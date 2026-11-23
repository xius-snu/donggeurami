//! The four games of 2026-11-18 at their tables, on the server (Hajun: "add
//! these minigames: Blend In, Knock Off, Bridge Race, Last Stand ... each of
//! these games have a coin entrance fee and payout"): one way for all four
//! from the join to the town again, which each game's own play fits into
//! ([`Rules`], each in its own module: `blend_in`, `knock_off`,
//! `bridge_race`, `last_stand`). The numbers each game's table goes by are
//! its shape (`roundtown_net::table::Shape`), both sides' alike.
//!
//! A table is a room of its own ([`Room::Table`]), and goes the way a game of
//! Bridge Wars goes (`arena`), from which it is copied:
//!
//! * **Joining** ([`Ask::Join`]): whoever asks goes in, and with them, if
//!   they lead a group, everyone of it who is here, free to and has the fee
//!   (`social`); in a group someone else leads, nobody does. One table for
//!   all of them: the one filling now if it has their seats free, and
//!   otherwise a new one. The seats are theirs from the moment they ask
//!   ([`Table::hold`]): with the database, the fee is taken there, and they
//!   sit down once it has been ([`paid`]), wherever each of them is answered
//!   first; without it, at once. Seated, each is told so
//!   ([`Tell::Entered`]) and put on the game's venue where the game has
//!   them wait ([`Rules::waiting_spot`]).
//! * **Waiting**: in a game with the computer (Knock Off, Bridge Race),
//!   once nobody has joined for its wait, how many seats it has is settled
//!   by how many people there are (`table::seats_for`) and the computer
//!   takes the seats left, one at a time, a throw of Bridge Wars'
//!   [`BOT_GAP`] apart (Hajun: "fill with ai like bridgewars"); in one
//!   without (Blend In, Last Stand), the countdown begins once it has as
//!   many people as it begins with and nobody has joined for its wait (as
//!   long as its game says, [`Rules::wait_from_join`]), or at once when it
//!   is full. Whoever sits past the seats a game settles on is moved up
//!   into one it has, and told so by being stood there afresh in it
//!   ([`close_ranks`]).
//! * **The countdown**, and then the play, which the game has to itself
//!   ([`Rules::begin`], [`Rules::step`]) until it says how everyone came
//!   ([`Table::finish`]).
//! * **The results**, up for the game's while: every person is paid by
//!   place, by the game's own table of prizes, people who left included
//!   (as House Builder pays them), and those who played it to its end earn
//!   toward their level; then everyone still there goes back to the town,
//!   over two frames, as every game here ends ([`close`], then [`end`]). A
//!   table nobody is left at closes too. Either way it closes only once
//!   whoever left it last has been sent that it is gone from their sight
//!   ([`Table::sent_since_anyone_went`]).
//!
//! **Leaving.** Before the play a seat left is anyone's again, and its fee
//! given back; once the play has begun it stays theirs, empty, and its game
//! hears of it ([`Rules::left`]); after, they only go. A game may say
//! someone is out of it before it is over ([`Seat::out`]): then it is over
//! for them, and they may go from it into another game, as from one over.
//! **A device that goes** keeps its seat while the game is played, its
//! body standing where it stood, as in Bridge Wars ([`Tables::keeps`],
//! [`Tables::returns`]); before the play it gives the seat up, fee and all.
//! **Who a body passes for**: a game may put someone else's name and look
//! on a person's body (`players::Disguised`, [`Ctx::disguise`]); every way
//! out of a table puts the account's own back.
//!
//! Each table is sent to its room as one entity: the [`Board`] (how far it
//! has got, the seconds left, how many people of how many seats, and at the
//! end how everyone came) and who sits where ([`Seats`]), beside whatever
//! the game puts there itself ([`Rules::show`]), each sent again only when
//! it has changed.

use std::collections::HashMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::server::server_tick::ServerTick;
use roundtown_net::bridge_wars::BOT_GAP;
use roundtown_net::social::{XP_PLAYED, XP_WON};
use roundtown_net::table::{self, Board, Results, Shape};
use roundtown_net::{Ask, Member, Minigame, Refusal, Seats, Spot, Tell, Venue, blend_in, bridge_race, knock_off, last_stand};

use crate::Config;
use crate::accounts::{Bank, Charged};
use crate::arena::{Arenas, wall_clock};
use crate::builder::Games;
use crate::fees::Paying;
use crate::players::{
    Accounts, Body, Disguised, Everyone, Plays, Who, move_to, place, stand_where_it_is, tell, undisguise,
};
use crate::rooms::{InRoom, Room, Towns};
use crate::social::Social;

/// How long a table goes on with nobody connected to it, in seconds, before
/// it is given up: Bridge Wars' own, the seats of whoever's devices have
/// gone waiting all the while.
const ABANDONED: f32 = 60.0;

/// The most seats a table has, and room: what its computer players' ids are
/// counted in ([`bot_id`]).
const MOST_SEATS: u64 = 64;

/// How long, at the most, a table whose countdown is over waits for whoever
/// sat down in its last moment and is still having their fee taken, in
/// seconds (mine): the database answers in a few thousandths of a second,
/// and they play; past this, the game begins without them, and they sit
/// down at another table as their fee is taken ([`paid`]).
const HELD_MOST: f32 = 2.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Tables>()
        .add_systems(
            PreUpdate,
            (
                hear.after(ServerSystems::Receive),
                // A fee is read after any prize asked for before it
                // (`fees::credited`).
                paid.after(crate::fees::credited),
            )
                .run_if(in_state(ServerState::Running)),
        )
        .add_systems(Update, run.run_if(in_state(ServerState::Running)));
}

/// Each of the four games' name, as the log says it.
pub(crate) fn name(game: Minigame) -> &'static str {
    match game {
        Minigame::BlendIn => "Blend In",
        Minigame::KnockOff => "Knock Off",
        Minigame::BridgeRace => "Bridge Race",
        Minigame::LastStand => "Last Stand",
        Minigame::BridgeWars => "Bridge Wars",
        Minigame::HouseBuilder => "House Builder",
    }
}

/// What coins change for at a table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Coins {
    Entry,
    Prize,
    Refund,
}

/// The line a change of coins at a table of `game` is kept with in the
/// database (`coin_changes.reason`), the table's number beside it.
fn why(game: Minigame, coins: Coins) -> &'static str {
    match (game, coins) {
        (Minigame::BlendIn, Coins::Entry) => "Blend In entry",
        (Minigame::BlendIn, Coins::Prize) => "Blend In prize",
        (Minigame::BlendIn, Coins::Refund) => "Blend In refund",
        (Minigame::KnockOff, Coins::Entry) => "Knock Off entry",
        (Minigame::KnockOff, Coins::Prize) => "Knock Off prize",
        (Minigame::KnockOff, Coins::Refund) => "Knock Off refund",
        (Minigame::BridgeRace, Coins::Entry) => "Bridge Race entry",
        (Minigame::BridgeRace, Coins::Prize) => "Bridge Race prize",
        (Minigame::BridgeRace, Coins::Refund) => "Bridge Race refund",
        (Minigame::LastStand, Coins::Entry) => "Last Stand entry",
        (Minigame::LastStand, Coins::Prize) => "Last Stand prize",
        (Minigame::LastStand, Coins::Refund) => "Last Stand refund",
        // Never at a table: theirs are their own (`builder`).
        (Minigame::BridgeWars | Minigame::HouseBuilder, _) => "a table's coins",
    }
}

/// What each of `game`'s seats is paid, from how it came out: the game's
/// own table of prizes, in its shared module.
fn prizes(game: Minigame, results: &Results, people: &[bool]) -> Vec<u32> {
    match game {
        Minigame::BlendIn => blend_in::prizes(results, people),
        Minigame::KnockOff => knock_off::prizes(results, people),
        Minigame::BridgeRace => bridge_race::prizes(results, people),
        Minigame::LastStand => last_stand::prizes(results, people),
        Minigame::BridgeWars | Minigame::HouseBuilder => vec![0; results.places.len()],
    }
}

/// The turn about the upright a body facing as `turn` has, in radians, as a
/// [`Spot`] keeps it: what [`Table::yaw`] holds.
pub(crate) fn yaw_of(turn: Quat) -> f32 {
    let forward = turn * Vec3::NEG_Z;
    (-forward.x).atan2(-forward.z)
}

/// The turn about the upright that faces from `from` toward `to`, across the
/// ground, in radians.
pub(crate) fn facing(from: Vec2, to: Vec2) -> f32 {
    let toward = to - from;
    (-toward.x).atan2(-toward.y)
}

/// Who the computer's player in `seat` of the table numbered `id` of `game`
/// is to everyone: clear of people's ids, which count up from 1, of House
/// Builder's bots, which count down from the top, of Bridge Wars' (from half
/// way down) and its builders' (from a quarter), and of every other table's
/// of every game: an eighth of the way down, less the game's place among
/// the four times 2^40 and the table's number times [`MOST_SEATS`] and the
/// seat, which reach no further than 2^42 below it.
pub(crate) fn bot_id(game: Minigame, id: u32, seat: usize) -> u64 {
    let game = Minigame::TABLES.iter().position(|&one| one == game).unwrap_or(0) as u64;
    let seat = (seat as u64).min(MOST_SEATS - 1);
    u64::MAX / 8 - ((game << 40) | (u64::from(id) * MOST_SEATS + seat))
}

/// Nobody: no id, no name and no look, as a seat nobody is in is sent; a
/// device takes a member with no id for an empty seat, as in Bridge Wars.
fn nobody() -> Member {
    Member {
        id: 0,
        name: String::new(),
        look: String::new(),
        ai: true,
    }
}

/// One seat at a table.
#[derive(Clone, Debug)]
pub(crate) struct Seat {
    /// The body in it: a person's, or the computer's (`Body::bot`); none
    /// (`Entity::PLACEHOLDER`) in a seat nobody is in, or one left for good.
    pub body: Entity,
    /// Who sits there as the room is told ([`Seats`]): a person's account,
    /// as its body's `Account` has it, or the computer's player; nobody, no
    /// id and no name, in a seat nobody is in. The table pays a person by
    /// its id. A game that has someone pass for someone else puts that on
    /// their body ([`Ctx::disguise`]), never here.
    pub member: Member,
    pub computer: bool,
    /// Left for good (to the town, or their device gone and given up): the
    /// seat stays theirs for the results, with no body. A seat nobody is in
    /// is gone too, as in Bridge Wars.
    pub gone: bool,
    /// Their device has gone and the seat waits for it (playing only).
    pub away: bool,
    /// What they paid to sit there, given back if the game never begins.
    pub paid: u32,
    /// Nobody sits here (mine, beside the spec's: Bridge Wars' `vacant`): a
    /// seat nobody has taken yet, or one left before the game began, for
    /// the next to come, a person or the computer. Never once the game is
    /// played: it begins with every one of its seats taken.
    pub vacant: bool,
    /// Out of the game before it is over, as the game says, which is the
    /// game's to set (Knock Off's in the sea: "If you are out, you can
    /// spectate or exit"; Last Stand's dead with the anchor broken): the
    /// game is over for whoever sits here, who may stay to watch, or go
    /// from it into another game of any kind, a group they lead with them,
    /// as one out of Bridge Wars may (`Arenas::done_with`); the seat stays
    /// theirs for the results either way.
    pub out: bool,
    /// Something was told to whoever sits here while their device was gone
    /// ([`Ctx::tell`]): back, it is told to begin the game afresh rather
    /// than carry on ([`Tables::returns`]).
    missed: bool,
}

impl Seat {
    /// A seat nobody is in.
    fn empty() -> Self {
        Self {
            body: Entity::PLACEHOLDER,
            member: nobody(),
            computer: false,
            gone: true,
            away: false,
            paid: 0,
            vacant: true,
            out: false,
            missed: false,
        }
    }

    /// Whether a person sits here, whether or not they have left since.
    pub(crate) fn person(&self) -> bool {
        !self.vacant && !self.computer
    }

    /// Whether a person sits here and has not left: their body is in the
    /// table's room, their device there or not.
    pub(crate) fn here(&self) -> bool {
        self.person() && !self.gone
    }
}

/// How far a table has got.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Stage {
    /// Waiting for people, `since_join` seconds since the last of them sat
    /// down; and, in a game with the computer, `coming`: the wait is over
    /// and the computer is taking the seats left, the next of its players
    /// in so many seconds.
    Waiting { since_join: f32, coming: Option<f32> },
    /// Counting down to the play, `left` seconds still.
    Countdown { left: f32 },
    /// Being played, `elapsed` seconds in.
    Playing { elapsed: f32 },
    /// Over, the results up `left` seconds more.
    Over { left: f32 },
}

/// One table of one of the four games: everything about it that is the
/// table's, whichever game is played at it. Its game's own state is its
/// [`Rules`], beside it in [`Tables`].
pub(crate) struct Table {
    pub game: Minigame,
    /// Its number, counted from 0 for each game.
    pub id: u32,
    pub room: Room,
    /// How its game fills, starts and pays (`table::shape`).
    pub shape: Shape,
    /// The entity its room is sent: `(Replicated, InRoom(room), Board,
    /// Seats)` and the game's own components ([`Rules::show`]).
    pub shown: Entity,
    /// Its seats, by number: as many as its game has at the most while it
    /// waits, those nobody is in among them ([`Seat::vacant`]); as many as
    /// it has once it is played ([`Table::size`]), every one taken.
    pub seats: Vec<Seat>,
    /// How many seats it will have (`table::seats_for`): with the computer,
    /// as many as its people make it while they wait, fixed once the
    /// computer begins coming; without, as many as it can have at the most
    /// while it waits (what "2/4" counts to), and as many as it has people
    /// once it begins.
    pub size: usize,
    pub stage: Stage,
    /// How it came out, once it is over ([`Table::finish`]).
    pub results: Option<Results>,
    /// Where each seat's body stands and which way it faces, by seat: a
    /// person's as their device last said, copied in each frame ([`sync`]);
    /// the computer's where the game puts it, which its body is sent as
    /// whenever it changes ([`flush`]).
    pub at: Vec<Vec3>,
    pub yaw: Vec<f32>,
    /// Seconds with nobody connected to it.
    pub unwatched: f32,
    /// Its computer players and its board have gone ([`close`]); everyone
    /// goes back to the town next frame ([`end`]).
    pub closing: bool,
    /// The game's own dice, seeded ([`Table::seed`]), so that a game played
    /// in a test plays the same each time.
    pub dice: fastrand::Rng,
    pub seed: u64,
    /// How long the play has left, in seconds, if the game keeps a clock it
    /// wants on the board (Bridge Race's six minutes): the game's to set as
    /// it plays. None, and the board says nothing counts down, for a game
    /// with none.
    pub play_left: Option<f32>,
    /// Someone has skipped the wait ([`Ask::Skip`], `RT_ALLOW_SKIP`): the
    /// computer takes every seat left at once, or the countdown begins.
    hurried: bool,
    /// The seats held for whoever's fee is being taken, by account, so that
    /// those who ask together sit together whenever each of them is
    /// answered ([`paid`]).
    holds: Vec<(usize, u64)>,
    /// Seats left for good since the game began, with the body that sat in
    /// each, for it to hear of ([`Rules::left`]), and seats whose devices
    /// are back, for it to be begun afresh on ([`welcome_back`]): what
    /// happened to the table from outside it (`players`) since it last ran.
    lefts: Vec<(usize, Entity)>,
    backs: Vec<usize>,
    /// Someone has gone out of its room since it last ran
    /// ([`Tables::walk_out`]); and the server's tick as it ran after the
    /// last of them went, with whether what its room is sent has gone out
    /// since ([`Table::sent_since_anyone_went`]).
    went_out: bool,
    gone_out: Option<(u32, bool)>,
    /// What its room was last sent, to send again only what has changed.
    sent_board: Option<Board>,
    sent_seats: Option<Vec<Member>>,
}

impl Table {
    /// A table of `game` numbered `id`, nobody at it yet, whose room is sent
    /// the entity `shown`; its dice thrown from `seed`. None for the two
    /// older games, which are not played at tables.
    pub(crate) fn new(game: Minigame, id: u32, shown: Entity, seed: u64) -> Option<Self> {
        let shape = table::shape(game)?;
        Some(Self {
            game,
            id,
            room: Room::Table(game, id),
            shape,
            shown,
            seats: (0..shape.most).map(|_| Seat::empty()).collect(),
            size: shape.most,
            stage: Stage::Waiting {
                since_join: 0.0,
                coming: None,
            },
            results: None,
            at: vec![Vec3::ZERO; shape.most],
            yaw: vec![0.0; shape.most],
            unwatched: 0.0,
            closing: false,
            dice: fastrand::Rng::with_seed(seed),
            seed,
            play_left: None,
            hurried: false,
            holds: Vec::new(),
            lefts: Vec::new(),
            backs: Vec::new(),
            went_out: false,
            gone_out: None,
            sent_board: None,
            sent_seats: None,
        })
    }

    /// Whether what its room is sent has gone out at least once since
    /// anyone last went out of its room (or nobody has), the server's tick
    /// being `tick` this frame: so that whoever went has been told that
    /// everything of it is gone from their sight before any of it is
    /// despawned, which it may be only then ([`close`]). An entity
    /// despawned in the same send as someone leaves its room is written to
    /// them neither as despawned nor as gone from sight (replicon 0.44's
    /// `collect_despawns`; the trap Bridge Wars fell into on 2026-11-09,
    /// there with everyone moved out as the game was despawned), and a
    /// device keeps it until its connection goes: the last table's board,
    /// on a device that has gone on to the next, beside the next one's.
    /// Whoever goes as the results run out for those still there holds the
    /// close back as much as the last of them to go does. A send goes out
    /// in the frame the tick moves, after every game has run: so the table
    /// may close the frame after it has been seen to move. Asked every
    /// frame, so that it sees the tick move.
    fn sent_since_anyone_went(&mut self, tick: u32) -> bool {
        if std::mem::take(&mut self.went_out) {
            self.gone_out = Some((tick, false));
        }
        match &mut self.gone_out {
            None => true,
            Some((_, true)) => {
                self.gone_out = None;
                true
            }
            Some((at, sent)) => {
                *sent = tick != *at;
                false
            }
        }
    }

    /// Whether its game is being played.
    pub(crate) fn playing(&self) -> bool {
        matches!(self.stage, Stage::Playing { .. })
    }

    /// Whether its game is over, its results up.
    pub(crate) fn over(&self) -> bool {
        matches!(self.stage, Stage::Over { .. })
    }

    /// How many people sit at it and have not left.
    pub(crate) fn people(&self) -> usize {
        self.seats.iter().filter(|seat| seat.here()).count()
    }

    /// The seat `body` is in, if it is in one.
    pub(crate) fn seat_of(&self, body: Entity) -> Option<usize> {
        if body == Entity::PLACEHOLDER {
            return None;
        }
        self.seats.iter().position(|seat| !seat.vacant && seat.body == body)
    }

    /// The order its seats are taken in: people take the first of them free,
    /// and the computer the last. Seat by seat, but in Bridge Race, where
    /// two people are a team (`bridge_race::PEOPLE_FIRST`: "if its 2 players
    /// then its a 2v2 against bots").
    pub(crate) fn order(&self) -> Vec<usize> {
        match self.game {
            Minigame::BridgeRace => bridge_race::PEOPLE_FIRST.iter().copied().filter(|&seat| seat < self.seats.len()).collect(),
            _ => (0..self.seats.len()).collect(),
        }
    }

    /// Whether how many seats it has is settled: once the computer has begun
    /// coming, in a game with it; once the play has, in one without.
    fn fixed(&self) -> bool {
        match self.stage {
            Stage::Waiting { coming, .. } => coming.is_some(),
            Stage::Countdown { .. } => self.shape.computer,
            Stage::Playing { .. } | Stage::Over { .. } => true,
        }
    }

    /// Whether `seat` is free for whoever comes next: one of the seats it
    /// will have, nobody in it, and not held for anyone.
    pub(crate) fn open(&self, seat: usize) -> bool {
        let seats = if self.fixed() { self.size } else { self.shape.most };
        seat < seats.min(self.seats.len()) && self.seats[seat].vacant && !self.held(seat)
    }

    /// Whether `seat` is held for someone whose fee is being taken.
    fn held(&self, seat: usize) -> bool {
        self.holds.iter().any(|&(held, _)| held == seat)
    }

    /// Whether anyone may still sit down at it: while it waits, and in a
    /// game without the computer, through its countdown too.
    fn joinable(&self) -> bool {
        !self.closing
            && match self.stage {
                Stage::Waiting { .. } => true,
                Stage::Countdown { .. } => !self.shape.computer,
                Stage::Playing { .. } | Stage::Over { .. } => false,
            }
    }

    /// Someone is coming to sit down while it waits for people (seated now,
    /// or their seat held while their fee is taken): it waits again from
    /// now, as long as its game says ([`Rules::wait_from_join`]), which is
    /// told how long it had still to wait if it was waiting already (with
    /// the computer, for it to come, which once it has begun is waited for
    /// no more; without, for the countdown, which it is only with as many
    /// people as the game begins with). Asked before they are counted.
    fn rewait(&mut self, rules: &dyn Rules) {
        let Stage::Waiting { since_join, coming } = self.stage else {
            return;
        };
        let before = self.people() + self.holds.len();
        let waiting = if self.shape.computer {
            coming.is_none() && before > 0
        } else {
            before >= self.shape.fewest
        };
        let left = waiting.then(|| (self.shape.wait - since_join).max(0.0));
        let wait = rules.wait_from_join(self, left).max(0.0);
        self.stage = Stage::Waiting {
            since_join: self.shape.wait - wait,
            coming,
        };
    }

    /// The seat held for `account`, no longer held, if it still was.
    fn unhold(&mut self, account: u64) -> Option<usize> {
        let at = self.holds.iter().position(|&(_, holder)| holder == account)?;
        Some(self.holds.swap_remove(at).0)
    }

    /// Whether anyone of the people at it but whoever is in `seat` is
    /// connected to it: in its room, with a device ([`sync`]).
    pub(crate) fn watched_but(&self, seat: usize) -> bool {
        (0..self.seats.len()).any(|other| other != seat && self.seats[other].here() && !self.seats[other].away)
    }

    /// Nobody sits in `seat` from now: it is anyone's.
    fn vacate(&mut self, seat: usize) {
        self.seats[seat] = Seat::empty();
    }

    /// Whoever is in `seat` leaves the table for good, however they leave.
    /// Before the play the seat is anyone's again, and a game without the
    /// computer counting down with too few people left waits again; the
    /// account and the fee to give back, if there is one. Once the play has
    /// begun the seat stays theirs with no body, and the game hears of it,
    /// and of the body, as the table next runs ([`Rules::left`]).
    fn leave(&mut self, seat: usize) -> Option<(u64, u32)> {
        if !self.seats[seat].here() {
            return None;
        }
        match self.stage {
            Stage::Waiting { .. } | Stage::Countdown { .. } => {
                let back = (self.seats[seat].member.id, self.seats[seat].paid);
                self.vacate(seat);
                if matches!(self.stage, Stage::Countdown { .. }) && !self.shape.computer && self.people() < self.shape.fewest {
                    info!("{} table {}: too few people left to begin, and it waits again", name(self.game), self.id);
                    self.stage = Stage::Waiting {
                        since_join: 0.0,
                        coming: None,
                    };
                }
                (back.1 > 0).then_some(back)
            }
            Stage::Playing { .. } | Stage::Over { .. } => {
                let left = &mut self.seats[seat];
                self.lefts.push((seat, left.body));
                left.gone = true;
                left.away = false;
                left.body = Entity::PLACEHOLDER;
                None
            }
        }
    }

    /// The game is over: how it came out (places and scores by seat, the
    /// table's total). The table works out the prizes with the game's own
    /// table of them, pays them as it next runs ([`run`]), and shows the
    /// results for the game's while (`shape.results`). Only while it is
    /// played, and once: a place or a score missing for a seat is the last
    /// place and nothing, and one too many is dropped.
    pub(crate) fn finish(&mut self, mut places: Vec<u8>, mut scores: Vec<u32>, total: Option<u32>) {
        if !self.playing() {
            return;
        }
        let seats = self.seats.len();
        places.resize(seats, seats.saturating_sub(1) as u8);
        scores.resize(seats, 0);
        let people: Vec<bool> = self.seats.iter().map(Seat::person).collect();
        let mut results = Results {
            places,
            scores,
            prizes: Vec::new(),
            total,
        };
        results.prizes = prizes(self.game, &results, &people);
        results.prizes.resize(seats, 0);
        self.results = Some(results);
        self.play_left = None;
        self.stage = Stage::Over {
            left: self.shape.results,
        };
    }

    /// What its room is shown of how it is going ([`Board`]).
    fn board(&self) -> Board {
        let people = self.people();
        let (stage, left) = match self.stage {
            Stage::Waiting { since_join, coming } => {
                // Until the computer comes, or the countdown begins: in a
                // game without the computer, only with enough people.
                let waits = if self.shape.computer {
                    coming.is_none() && people > 0
                } else {
                    people >= self.shape.fewest
                };
                let left = if waits { self.shape.wait - since_join } else { 0.0 };
                (table::Stage::Waiting, left)
            }
            Stage::Countdown { left } => (table::Stage::Countdown, left),
            Stage::Playing { .. } => (table::Stage::Playing, self.play_left.unwrap_or(0.0)),
            Stage::Over { left } => (table::Stage::Over(self.results.clone().unwrap_or_default()), left),
        };
        Board {
            game: self.game,
            id: self.id,
            stage,
            left: left.max(0.0).ceil().min(f32::from(u16::MAX)) as u16,
            people: people.min(usize::from(u8::MAX)) as u8,
            seats: self.size.min(usize::from(u8::MAX)) as u8,
        }
    }
}

/// What a table's game plays by: everything about the game that is its
/// own, each of the four in its own module. The table calls it as it goes
/// ([`run`], [`hear`]); a game acts on the world only through what it is
/// handed ([`Ctx`]), and is told everything it has to know.
///
/// A person's seat may change before the play: whoever sits past the seats
/// the game settles on is moved up into one it has, as the computer begins
/// to come, and as a game without the computer begins ([`close_ranks`]).
/// Their device is told the new seat by being stood afresh in it
/// (`Tell::Arrive`, its `seat`), not by `Tell::Entered` again, which a
/// device takes for a table begun afresh. From [`Rules::begin`] on, seats
/// do not change.
pub(crate) trait Rules: Send + Sync + 'static {
    /// Where someone just seated in `seat` stands while the table waits, and
    /// which way they face (radians about the upright, as [`yaw_of`]).
    fn waiting_spot(&mut self, table: &Table, seat: usize) -> (Vec3, f32);

    /// Who the computer is in `seat` (asked only of a game whose shape has
    /// the computer). Its id is the table's whatever this says ([`bot_id`]),
    /// and it is the computer's (`ai`). By default a boy or a girl by the
    /// coin, a look and a name of that side not taken at the table, as
    /// Bridge Wars' are (`arena::Game::bot_member`), thrown from the
    /// table's seed and the seat.
    fn computer(&mut self, table: &Table, seat: usize) -> Member {
        let mut dice = fastrand::Rng::with_seed(table.seed ^ (seat as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let boy = dice.bool();
        let names: Vec<&str> = roundtown_net::ai_names(boy)
            .iter()
            .copied()
            .filter(|name| table.seats.iter().all(|other| !other.member.name.starts_with(name)))
            .collect();
        Member {
            id: bot_id(table.game, table.id, seat),
            name: dice.choice(names).unwrap_or("Bot").into(),
            look: roundtown_net::ai_look(boy, dice.usize(..)).into(),
            ai: true,
        }
    }

    /// Which free seats `count` people going in together take, or None if
    /// they do not fit. By default the first free ones, in the order the
    /// table's seats are taken in ([`Table::order`], [`Table::open`]).
    fn seats_for(&self, table: &Table, count: usize) -> Option<Vec<usize>> {
        let seats: Vec<usize> = table.order().into_iter().filter(|&seat| table.open(seat)).take(count).collect();
        (seats.len() == count).then_some(seats)
    }

    /// How many seconds the table waits from now, someone having just sat
    /// down at it as it waits for people (or had their seat held while
    /// their fee is taken), before the computer comes (with the computer)
    /// or the countdown begins (without): `left` is how long it had still
    /// to wait, if it was waiting already, and None if it was not (nobody
    /// there yet, or too few people to begin). By default the game's whole
    /// wait again (`shape.wait`), from the last person in, as Bridge Wars'
    /// ("10second later it will fill with bots").
    fn wait_from_join(&self, table: &Table, _left: Option<f32>) -> f32 {
        table.shape.wait
    }

    /// Each frame before play (waiting, counting down).
    fn idle(&mut self, _ctx: &mut Ctx, _table: &mut Table, _dt: f32) {}

    /// The countdown is over: the play begins. Every seat it has is taken
    /// and its seats are numbered from 0 with nobody missing between: stand
    /// everyone where it begins ([`Ctx::place`]).
    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table);

    /// A frame of play, `dt` seconds on (`table.stage`'s `elapsed` counts
    /// this frame already). It ends the game with [`Table::finish`]; and it
    /// says when someone is out of it before then ([`Seat::out`]).
    fn step(&mut self, ctx: &mut Ctx, table: &mut Table, dt: f32);

    /// Something someone seated asked: any [`Ask`] but `Join` (and the two
    /// older games' joins, `Arena` and `Play`), `Leave`, `Chat`, `Hello`,
    /// `Profile`, `Social` and `Travel`, which the table and the rest of
    /// the server answer; and only while they are in the table's room.
    /// `Skip` only while it is played, and only with `RT_ALLOW_SKIP`
    /// (`config.allow_skip`): the table skips its own waits, and the play
    /// is the game's to skip as it will, or not. A punch (`Ask::Punch`) and
    /// a fall into the void (`Ask::Respawn`) at a table are their game's
    /// alone.
    fn hear(&mut self, _ctx: &mut Ctx, _table: &mut Table, _seat: usize, _ask: &Ask) {}

    /// `seat` has left for good since the play began, while it is played or
    /// with its results up (`table.over()`, when it only has to tidy up):
    /// back to the town, into another game, or their device gone and given
    /// up. `body` is the body that sat there, if it is still about (not if
    /// it went with its device), wherever it is now: for the game to take
    /// off it whatever it put on it (a sword, a glow), which it may have
    /// carried out of the room for the frame or two before this.
    fn left(&mut self, _ctx: &mut Ctx, _table: &mut Table, _seat: usize, _body: Option<Entity>) {}

    /// `seat`'s device is back after its connection went, and begins the
    /// game afresh (`Tell::Entered` has been sent; a `Tell::Arrive` where
    /// its body stands follows): tell it what only it needs.
    fn back(&mut self, _ctx: &mut Ctx, _table: &mut Table, _seat: usize) {}

    /// Whether nothing said in the table's room is passed on now
    /// ([`Tables::hushed`], which the chat asks): a game whose play has
    /// everyone say nothing while it lasts (Blend In from its hide to the
    /// end of its seek, since 2026-11-18). Never, by default.
    fn hushes(&self, _table: &Table) -> bool {
        false
    }

    /// Puts the game's own components on `table.shown`, each only when it
    /// changed.
    fn show(&mut self, commands: &mut Commands, table: &Table);

    /// The table is closing, everyone still at it in its room for one frame
    /// more: despawns everything the game spawned in its room (its computer
    /// players' bodies and its board are the table's), and takes off the
    /// bodies of the people still at it whatever it put on them (each
    /// seat's `body` where `here()`; those who left were [`Rules::left`]),
    /// who go back to the town the frame after.
    fn close(&mut self, commands: &mut Commands, table: &Table);
}

/// The game played at a new table of `game`, numbered as `table` is, its
/// dice thrown from `seed`.
pub(crate) fn rules_for(game: Minigame, table: &Table, seed: u64) -> Box<dyn Rules> {
    match game {
        Minigame::BlendIn => Box::new(crate::blend_in::BlendIn::new(table, seed)),
        Minigame::KnockOff => Box::new(crate::knock_off::KnockOff::new(table, seed)),
        Minigame::BridgeRace => Box::new(crate::bridge_race::BridgeRace::new(table, seed)),
        Minigame::LastStand => Box::new(crate::last_stand::LastStand::new(table, seed)),
        Minigame::BridgeWars | Minigame::HouseBuilder => {
            unreachable!("only the four are played at tables, and a table is opened only for them")
        }
    }
}

/// Every person's body, as the server keeps it beside what is sent of it.
pub(crate) type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Body,
        &'static mut Spot,
        &'static mut Venue,
        &'static Member,
    ),
>;

/// What a table acts on the world with, in each of its systems: what is
/// spawned and despawned, what devices are told, and every body.
#[derive(SystemParam)]
pub(crate) struct Hands<'w, 's> {
    commands: Commands<'w, 's>,
    tells: MessageWriter<'w, ToClients<Tell>>,
    bodies: Bodies<'w, 's>,
    accounts: Accounts<'w, 's>,
}

impl<'w, 's> Hands<'w, 's> {
    fn ctx<'a>(&'a mut self, now: f64, clock: f64, config: &'a Config) -> Ctx<'a, 'w, 's> {
        Ctx {
            commands: &mut self.commands,
            tells: &mut self.tells,
            bodies: &mut self.bodies,
            accounts: &self.accounts,
            now,
            clock,
            config,
        }
    }
}

/// What a table's game is handed to act on the world with: the server's
/// commands (anything it spawns goes in its room: [`Ctx::spawn_in_room`]),
/// what devices are told, every body as the server keeps it, every
/// account, the server's time (`now`, its own seconds) and the clock every
/// device has for the town's (`clock`, seconds since 1970), and how the
/// server was set up.
// Until the four games' own play comes, each in its own module, much of
// this is for them alone: their stand-ins and the tests use some of it.
#[allow(dead_code)]
pub(crate) struct Ctx<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub tells: &'a mut MessageWriter<'w, ToClients<Tell>>,
    pub bodies: &'a mut Bodies<'w, 's>,
    pub accounts: &'a Accounts<'w, 's>,
    pub now: f64,
    pub clock: f64,
    pub config: &'a Config,
}

// (As for `Ctx`: the four games' own play will call what the stand-ins do
// not.)
#[allow(dead_code)]
impl Ctx<'_, '_, '_> {
    /// Tells the person in `seat` `told`, if they are at the table, their
    /// body in its room: through their device; or, their device gone, they
    /// have missed it, and are told to begin the game afresh when they are
    /// back ([`Tables::returns`]). Tell a seat through this, not straight
    /// to its device, so that nothing is missed unawares.
    pub(crate) fn tell(&mut self, table: &mut Table, seat: usize, told: Tell) {
        let Some(it) = table.seats.get_mut(seat) else {
            return;
        };
        if !it.here() {
            return;
        }
        let Ok((body, ..)) = self.bodies.get(it.body) else {
            return;
        };
        if body.room != table.room {
            return;
        }
        match body.client {
            Some(client) => tell(self.tells, client, told),
            None => it.missed = true,
        }
    }

    /// Tells every person at the table `told`, as [`Ctx::tell`] does.
    pub(crate) fn tell_room(&mut self, table: &mut Table, told: Tell) {
        for seat in 0..table.seats.len() {
            if table.seats[seat].here() {
                self.tell(table, seat, told.clone());
            }
        }
    }

    /// Stands whoever is in `seat` at `at`, facing `yaw`: a person through
    /// their device, as a fresh arrival there (`players::place`, which
    /// drops what their device sent from before); the computer's player
    /// there as the frame ends, its body sent where it stands
    /// ([`Table::at`]).
    pub(crate) fn place(&mut self, table: &mut Table, seat: usize, at: Vec3, yaw: f32) {
        if seat >= table.seats.len() {
            return;
        }
        table.at[seat] = at;
        table.yaw[seat] = yaw;
        let it = &table.seats[seat];
        if !it.here() {
            return;
        }
        let body = it.body;
        let Ok((body_of, spot, venue, _)) = self.bodies.get_mut(body) else {
            return;
        };
        if body_of.room != table.room {
            return;
        }
        place(
            self.commands,
            self.tells,
            body,
            (body_of, spot, venue),
            table.room,
            table.game.venue(),
            seat as u8,
            Some(Spot::of(at, Quat::from_rotation_y(yaw))),
            self.now,
        );
    }

    /// Knocks the person in `seat` back from `from`, where whoever has the
    /// id `by` stood, who was `running` if so: their own device knocks its
    /// body ([`Tell::Knocked`]). The computer's players are the game's to
    /// knock.
    pub(crate) fn knock(&mut self, table: &mut Table, seat: usize, by: u64, from: Spot, running: bool) {
        self.tell(table, seat, Tell::Knocked { by, from, running });
    }

    /// Spawns `bundle` in the table's room: sent to everyone at it, and
    /// nobody else ([`InRoom`]). The game despawns it ([`Rules::close`]).
    pub(crate) fn spawn_in_room(&mut self, table: &Table, bundle: impl Bundle) -> Entity {
        self.commands.spawn((Replicated, InRoom(table.room), bundle)).id()
    }

    /// A body in the table's room that is in no seat, moved by the game
    /// alone ([`Ctx::move_body`]), as `member`, standing at `at` and facing
    /// `yaw`: drawn by every device as a person is (Blend In's crowd). The
    /// game despawns it.
    pub(crate) fn spawn_body(&mut self, table: &Table, member: Member, at: Vec3, yaw: f32) -> Entity {
        let body = Body::bot(table.room, 0, self.now);
        let spot = Spot::of(at, Quat::from_rotation_y(yaw));
        self.spawn_in_room(table, (member, spot, table.game.venue(), body))
    }

    /// Sends a body the game moves itself as standing at `at`, facing `yaw`,
    /// if that is anywhere new: and as standing still once it has stopped,
    /// so that nobody watching carries it on the way it was going while they
    /// wait for an update that is not coming ([`Spot::still`]), as Bridge
    /// Wars' are.
    pub(crate) fn move_body(&mut self, body: Entity, at: Vec3, yaw: f32) {
        let Ok((_, mut spot, ..)) = self.bodies.get_mut(body) else {
            return;
        };
        let mut next = Spot::of(at, Quat::from_rotation_y(yaw));
        next.still = (next.x, next.y, next.z) == (spot.x, spot.y, spot.z);
        if !next.same(&spot) {
            *spot = next;
        }
    }

    /// Where the body in `seat` is, as the server has it.
    pub(crate) fn spot(&self, table: &Table, seat: usize) -> Option<Spot> {
        let it = table.seats.get(seat)?;
        if it.vacant || it.body == Entity::PLACEHOLDER {
            return None;
        }
        self.bodies.get(it.body).ok().map(|(_, spot, ..)| *spot)
    }

    /// Has the person in `seat` pass for `member` (another name, look and
    /// id) until the game takes it off ([`Ctx::undisguise`]) or they leave
    /// the table, which takes it off whatever the game does: sent to
    /// everyone in the room as who they are, while their account keeps its
    /// own (`players::Account`).
    pub(crate) fn disguise(&mut self, table: &Table, seat: usize, member: Member) {
        let Some(it) = table.seats.get(seat) else {
            return;
        };
        if it.here() {
            self.commands.entity(it.body).insert((member, Disguised));
        }
    }

    /// The person in `seat` is who their account is again.
    pub(crate) fn undisguise(&mut self, table: &Table, seat: usize) {
        if let Some(it) = table.seats.get(seat)
            && it.here()
        {
            undisguise(self.commands, it.body, self.accounts);
        }
    }
}

/// A fee to be given back: whoever left a table before its game began, or
/// paid for one there was no seat at for them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Refund {
    account: u64,
    fee: u32,
    game: Minigame,
    table: u32,
}

/// Every table, by game and number, and each one's game beside it.
#[derive(Resource, Default)]
pub(crate) struct Tables {
    /// The next table's number, for each game.
    next: HashMap<Minigame, u32>,
    tables: HashMap<(Minigame, u32), Table>,
    /// Each table's game ([`rules_for`]), beside it rather than in it, so
    /// that a game is handed its table with nothing borrowed twice.
    games: HashMap<(Minigame, u32), Box<dyn Rules>>,
    /// The table filling now, for each game: where whoever joins next sits,
    /// if it has their seats free.
    filling: HashMap<Minigame, u32>,
    /// Fees to give back, as the tables next run ([`pay_back`]).
    refunds: Vec<Refund>,
}

impl Tables {
    /// The table in `room`, and `body`'s seat at it.
    pub(crate) fn of(&mut self, room: Room, body: Entity) -> Option<(&mut Table, usize)> {
        let Room::Table(game, id) = room else {
            return None;
        };
        let table = self.tables.get_mut(&(game, id))?;
        let seat = table.seat_of(body)?;
        Some((table, seat))
    }

    /// Whether what is said in `room` is held back now: it is a table's
    /// whose game hushes it ([`Rules::hushes`]). Anywhere else, never.
    pub(crate) fn hushed(&self, room: Room) -> bool {
        let Room::Table(game, id) = room else {
            return false;
        };
        let key = (game, id);
        self.tables
            .get(&key)
            .zip(self.games.get(&key))
            .is_some_and(|(table, rules)| rules.hushes(table))
    }

    /// The table in `room`, and `body`'s seat at it, to be looked at.
    pub(crate) fn seated(&self, room: Room, body: Entity) -> Option<(&Table, usize)> {
        let Room::Table(game, id) = room else {
            return None;
        };
        let table = self.tables.get(&(game, id))?;
        let seat = table.seat_of(body)?;
        Some((table, seat))
    }

    /// `body` has left the table in `room` for good: back to the town, to
    /// another game, or its device gone and given up ([`Table::leave`]),
    /// its fee given back as the tables next run if the game had not begun.
    /// Every way out of a table's room comes here first, so that the table
    /// does not close until whoever went has been sent that it is gone from
    /// their sight ([`Table::sent_since_anyone_went`]).
    pub(crate) fn walk_out(&mut self, room: Room, body: Entity) {
        let Room::Table(game, id) = room else {
            return;
        };
        let Some(table) = self.tables.get_mut(&(game, id)) else {
            return;
        };
        table.went_out = true;
        let Some(seat) = table.seat_of(body) else {
            return;
        };
        info!("{} table {id}: seat {seat} left it", name(game));
        if let Some((account, fee)) = table.leave(seat) {
            self.refunds.push(Refund {
                account,
                fee,
                game,
                table: id,
            });
        }
    }

    /// Whether `body`'s seat at the table in `room` waits for it, its device
    /// having gone: while the game is being played, and through its
    /// results, as a seat in Bridge Wars waits (`Arenas::keeps`), the body
    /// standing where it stood with nobody to move it. Not before: a game
    /// does not begin with a seat waiting for someone who may not come, and
    /// the seat is given up, its fee given back.
    pub(crate) fn keeps(&self, room: Room, body: Entity) -> bool {
        self.seated(room, body)
            .is_some_and(|(table, seat)| !table.closing && table.seats[seat].here() && (table.playing() || table.over()))
    }

    /// What becomes of whoever plays `body`, whose device is back, the body
    /// having been kept at the table in `room` ([`Tables::keeps`]), as
    /// Bridge Wars decides it (`Arenas::returns`). `still_there`, the device
    /// says it is on the table's venue yet: its app was only put away, and
    /// it has the game as it left it.
    ///
    /// - Still there, it carries on where it stands ([`Return::Resume`]):
    ///   unless it was to be told something meanwhile, which it was not
    ///   there to hear ([`Ctx::tell`]), and the game is being played: then
    ///   it is put back, as one that has forgotten is.
    /// - Not there, the app closed and opened again, it is put back in the
    ///   game ([`Return::Back`]), so long as the game is being played and
    ///   someone else of the people at it is connected to it (Hajun, of
    ///   Bridge Wars: "u should rejoin the game as long as there is atleast
    ///   1 real player in that game still"): begun afresh on it, as the
    ///   table next runs ([`welcome_back`]).
    /// - Otherwise the seat is given up, and they are in the town
    ///   ([`Return::GiveUp`]): the caller walks them out.
    pub(crate) fn returns(&mut self, room: Room, body: Entity, still_there: bool) -> Return {
        let Some((table, seat)) = self.of(room, body) else {
            return Return::GiveUp;
        };
        if table.closing || !table.seats[seat].here() {
            return Return::GiveUp;
        }
        let missed = std::mem::take(&mut table.seats[seat].missed);
        if still_there && (!missed || !table.playing()) {
            table.seats[seat].away = false;
            return Return::Resume;
        }
        if !table.playing() || !(still_there || table.watched_but(seat)) {
            return Return::GiveUp;
        }
        table.seats[seat].away = false;
        table.backs.push(seat);
        Return::Back { seat: seat as u8 }
    }

    /// Whether the table in `room` has yet to begin its game: waiting for
    /// people, or counting down. What someone's friends are told of where
    /// they are (`social`).
    pub(crate) fn queued(&self, room: Room) -> bool {
        let Room::Table(game, id) = room else {
            return false;
        };
        self.tables
            .get(&(game, id))
            .is_some_and(|table| matches!(table.stage, Stage::Waiting { .. } | Stage::Countdown { .. }))
    }

    /// Whether `body` is done with the table in `room`: its game is over, or
    /// it is out of it ([`Seat::out`]), and it is free to go into another
    /// game, of any kind, as one done with a game of Bridge Wars is.
    pub(crate) fn done_with(&self, room: Room, body: Entity) -> bool {
        self.seated(room, body)
            .is_some_and(|(table, seat)| table.over() || table.seats[seat].out)
    }

    /// How many people are at each of the four games' tables, theirs
    /// connected or not: what the server's log says once a minute.
    pub(crate) fn counts(&self) -> Vec<(Minigame, usize)> {
        Minigame::TABLES
            .iter()
            .map(|&game| {
                let people = self
                    .tables
                    .iter()
                    .filter(|((of, _), _)| *of == game)
                    .map(|(_, table)| table.people())
                    .sum();
                (game, people)
            })
            .collect()
    }

    /// A new table of `game`, its room sent its board: the one filling now.
    /// Its number; None for a game not played at tables.
    fn open_table(&mut self, commands: &mut Commands, game: Minigame) -> Option<u32> {
        table::shape(game)?;
        let next = self.next.entry(game).or_default();
        let id = *next;
        *next += 1;
        let room = Room::Table(game, id);
        let shown = commands.spawn((Replicated, InRoom(room), Seats::default())).id();
        let seed = fastrand::u64(..);
        let table = Table::new(game, id, shown, seed)?;
        let rules = rules_for(game, &table, seed);
        self.tables.insert((game, id), table);
        self.games.insert((game, id), rules);
        self.filling.insert(game, id);
        info!("{} table {id} opened", name(game));
        Some(id)
    }

    /// The table `count` people going into a game of `game` together sit at,
    /// and which seats, in their order: the one filling now, if anyone may
    /// still sit down at it and it has their seats free
    /// ([`Rules::seats_for`]); otherwise a new one, the old one left to
    /// fill with whoever is at it (and the computer). In a game without the
    /// computer, any of its tables anyone may still sit down at, the oldest
    /// first, before a new one (mine): nothing else fills a table there,
    /// and someone waiting alone for a second person at one that a group
    /// did not fit into would otherwise wait for good, everyone after them
    /// sat down at the group's table or at newer ones.
    fn filling(&mut self, commands: &mut Commands, game: Minigame, count: usize) -> Option<(u32, Vec<usize>)> {
        let shape = table::shape(game)?;
        let tries: Vec<u32> = if shape.computer {
            self.filling.get(&game).copied().into_iter().collect()
        } else {
            let mut ids: Vec<u32> = self.tables.keys().filter(|(of, _)| *of == game).map(|&(_, id)| id).collect();
            ids.sort_unstable();
            ids
        };
        for id in tries {
            if let (Some(table), Some(rules)) = (self.tables.get(&(game, id)), self.games.get(&(game, id)))
                && table.joinable()
                && let Some(seats) = rules.seats_for(table, count)
            {
                return Some((id, seats));
            }
        }
        let id = self.open_table(commands, game)?;
        let key = (game, id);
        let seats = self.games.get(&key)?.seats_for(self.tables.get(&key)?, count)?;
        Some((id, seats))
    }

    /// Holds `seat` at the table numbered `id` of `game` for the account
    /// `account`, whose fee is being taken: the table waits again from now,
    /// as it does when anyone sits down ([`Table::rewait`]).
    fn hold(&mut self, (game, id): (Minigame, u32), seat: usize, account: u64) {
        let (Some(table), Some(rules)) = (self.tables.get_mut(&(game, id)), self.games.get(&(game, id))) else {
            return;
        };
        table.rewait(rules.as_ref());
        table.holds.push((seat, account));
    }

    /// The seat held at the table numbered `id` of `game` for `account`,
    /// whose fee has been answered, no longer held; if it still was, and
    /// anyone may still sit down there.
    fn unhold(&mut self, game: Minigame, id: u32, account: u64) -> Option<usize> {
        let table = self.tables.get_mut(&(game, id))?;
        let seat = table.unhold(account)?;
        (table.joinable() && table.seats.get(seat).is_some_and(|it| it.vacant)).then_some(seat)
    }
}

/// What becomes of someone whose device is back while their body was kept
/// at a table ([`Tables::returns`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Return {
    /// The device never left the game: it carries on where it stands.
    Resume,
    /// It is put back in the game, in this seat, which is begun afresh on
    /// it ([`Tell::Entered`], [`Rules::back`]) as the table next runs.
    Back { seat: u8 },
    /// The seat is given up: they are in the town.
    GiveUp,
}

/// Whether the body `entity` is free to go into a game: connected, and in
/// the town, or done with whatever game it is in (over for it, or out of
/// it), whatever game that is.
fn free_to_join(body: &Body, entity: Entity, tables: &Tables, arenas: &Arenas, games: &Games) -> bool {
    body.client.is_some()
        && match body.room {
            Room::Town(_) => true,
            Room::Home(_) => false,
            Room::Game(_) => games.done_with(body.room, entity),
            Room::Arena(_) => arenas.done_with(body.room, entity),
            Room::Table(..) => tables.done_with(body.room, entity),
        }
}

/// Seats `body`, its fee paid, `paid`, in `seat` of the table `key`: out of
/// the town, or out of whatever game it was done with, and into the table's
/// room, its device told it is at the table ([`Tell::Entered`]) and then
/// where the game has it wait ([`Rules::waiting_spot`]).
#[allow(clippy::too_many_arguments)]
fn sit(
    ctx: &mut Ctx,
    tables: &mut Tables,
    towns: &mut Towns,
    arenas: &mut Arenas,
    games: &mut Games,
    (game, id): (Minigame, u32),
    seat: usize,
    body: Entity,
    paid: u32,
) {
    let Ok((body_of, _, _, member)) = ctx.bodies.get(body) else {
        return;
    };
    let old = body_of.room;
    let member = ctx.accounts.get(body).map_or_else(|_| member.clone(), |(account, _)| account.0.clone());
    // Out of whatever game it was done with, as itself again.
    match old {
        Room::Table(..) => tables.walk_out(old, body),
        Room::Arena(_) => arenas.walk_out(old, body),
        Room::Game(_) => games.walk_out(old, body),
        Room::Town(_) | Room::Home(_) => {}
    }
    undisguise(ctx.commands, body, ctx.accounts);
    towns.away(body);
    let (Some(table), Some(rules)) = (tables.tables.get_mut(&(game, id)), tables.games.get_mut(&(game, id))) else {
        return;
    };
    if seat >= table.seats.len() {
        return;
    }
    info!("{} sits at {} table {id}, in seat {seat}", member.name, name(game));
    table.rewait(rules.as_ref());
    table.seats[seat] = Seat {
        body,
        member,
        computer: false,
        gone: false,
        away: false,
        paid,
        vacant: false,
        out: false,
        missed: false,
    };
    let (at, yaw) = rules.waiting_spot(table, seat);
    table.at[seat] = at;
    table.yaw[seat] = yaw;
    let Ok((body_of, spot, venue, _)) = ctx.bodies.get_mut(body) else {
        return;
    };
    if let Some(client) = body_of.client {
        tell(ctx.tells, client, Tell::Entered { game, seat: seat as u8 });
    }
    place(
        ctx.commands,
        ctx.tells,
        body,
        (body_of, spot, venue),
        table.room,
        game.venue(),
        seat as u8,
        Some(Spot::of(at, Quat::from_rotation_y(yaw))),
        ctx.now,
    );
}

/// The computer, in `seat` of `table`: who the game says it is
/// ([`Rules::computer`]), a body in the table's room standing where the
/// game has it wait.
fn seat_computer(ctx: &mut Ctx, table: &mut Table, rules: &mut dyn Rules, seat: usize) {
    let mut member = rules.computer(table, seat);
    member.id = bot_id(table.game, table.id, seat);
    member.ai = true;
    let (at, yaw) = rules.waiting_spot(table, seat);
    let body = ctx
        .commands
        .spawn((
            Replicated,
            member.clone(),
            Spot::of(at, Quat::from_rotation_y(yaw)),
            table.game.venue(),
            InRoom(table.room),
            Body::bot(table.room, seat as u8, ctx.now),
        ))
        .id();
    table.seats[seat] = Seat {
        body,
        member,
        computer: true,
        gone: false,
        away: false,
        paid: 0,
        vacant: false,
        out: false,
        missed: false,
    };
    table.at[seat] = at;
    table.yaw[seat] = yaw;
}

/// Moves whoever sits past the first `size` seats of `table`, a person or a
/// seat held for one, into the first seat before them nobody is in, in the
/// order the seats are taken ([`Table::order`]): as the computer begins to
/// come, so that someone left alone in Bridge Race's far seat is where one
/// against one is played; and as a table without the computer begins, so
/// that its people are numbered from 0 with nobody missing between. Whoever
/// is moved is stood afresh in their new seat, which their device is told
/// with it (`Tell::Arrive`, through [`Ctx::place`]): `stand`, where the
/// game has that seat wait; otherwise where they stand. Not with
/// `Tell::Entered`, which their device would take for a table begun afresh
/// on it, its name said across the screen again and its board put up anew
/// (mine).
fn close_ranks(ctx: &mut Ctx, table: &mut Table, rules: &mut dyn Rules, size: usize, stand: bool) {
    let order = table.order();
    while let Some(from) = (size..table.seats.len()).find(|&seat| !table.seats[seat].vacant || table.held(seat)) {
        let Some(to) = order.iter().copied().find(|&seat| seat < size && table.seats[seat].vacant && !table.held(seat)) else {
            break;
        };
        if let Some(hold) = table.holds.iter_mut().find(|(held, _)| *held == from) {
            hold.0 = to;
            continue;
        }
        table.seats.swap(from, to);
        table.at.swap(from, to);
        table.yaw.swap(from, to);
        info!("{} table {}: seat {from} moves up to seat {to}", name(table.game), table.id);
        if !table.seats[to].here() {
            continue;
        }
        let (at, yaw) = if stand {
            rules.waiting_spot(table, to)
        } else {
            (table.at[to], table.yaw[to])
        };
        ctx.place(table, to, at, yaw);
    }
}

/// What players ask of the four games' tables: to join one, to leave one,
/// to skip a wait, and whatever else they ask at one, which is its game's.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<&Plays>,
    paying: Query<(), With<Paying>>,
    mut hands: Hands,
    mut tables: ResMut<Tables>,
    (mut towns, mut arenas, mut games): (ResMut<Towns>, ResMut<Arenas>, ResMut<Games>),
    (everyone, mut social, bank): (Res<Everyone>, ResMut<Social>, Res<Bank>),
    config: Res<Config>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs_f64();
    let clock = wall_clock();
    for ask in asks.read() {
        let Some(client) = ask.client_id.entity() else {
            continue;
        };
        let Ok(&Plays(body)) = clients.get(client) else {
            continue;
        };
        let Ok((body_of, ..)) = hands.bodies.get(body) else {
            continue;
        };
        let room = body_of.room;
        let mut ctx = hands.ctx(now, clock, &config);
        match &ask.message {
            Ask::Join(game) => join(
                &mut ctx,
                &mut tables,
                (&mut towns, &mut arenas, &mut games),
                (&everyone, &mut social, &bank),
                &paying,
                (body, client),
                *game,
            ),
            // Back to the town, from any stage of any table: before its
            // game, with the fee given back.
            Ask::Leave | Ask::Travel(Venue::Town) if matches!(room, Room::Table(..)) => {
                tables.walk_out(room, body);
                undisguise(ctx.commands, body, ctx.accounts);
                let Ok((mut body_of, spot, venue, _)) = ctx.bodies.get_mut(body) else {
                    continue;
                };
                let (channel, town_seat) = towns.sit(body, body_of.who.0, body_of.town);
                body_of.town = Some(channel);
                move_to(
                    ctx.commands,
                    ctx.tells,
                    body,
                    (body_of, spot, venue),
                    Room::Town(channel),
                    Venue::Town,
                    town_seat,
                    now,
                );
            }
            // Home is by the town.
            Ask::Travel(_) if matches!(room, Room::Table(..)) => {
                tell(ctx.tells, client, Tell::Refused(Refusal::Busy));
            }
            // For trying the games out (`RT_ALLOW_SKIP`): the wait is over
            // now, and the computer comes all at once; or the countdown is.
            // The play is the game's, to skip as it offers (`Rules::hear`).
            Ask::Skip if matches!(room, Room::Table(..)) => {
                if !config.allow_skip {
                    continue;
                }
                let Room::Table(game, id) = room else {
                    continue;
                };
                let Tables { tables: all, games: rules, .. } = &mut *tables;
                let (Some(table), Some(rules)) = (all.get_mut(&(game, id)), rules.get_mut(&(game, id))) else {
                    continue;
                };
                let Some(seat) = table.seat_of(body).filter(|_| !table.closing) else {
                    continue;
                };
                match table.stage {
                    Stage::Waiting { .. } => table.hurried = true,
                    Stage::Countdown { left } => table.stage = Stage::Countdown { left: left.min(0.0) },
                    Stage::Playing { .. } => rules.hear(&mut ctx, table, seat, &ask.message),
                    Stage::Over { .. } => {}
                }
            }
            // What the table and the rest of the server answer: the two
            // older games' joins are theirs (`arena`, `builder`).
            Ask::Hello { .. }
            | Ask::Profile { .. }
            | Ask::Social(_)
            | Ask::Chat(_)
            | Ask::Leave
            | Ask::Skip
            | Ask::Travel(_)
            | Ask::Arena
            | Ask::Play => {}
            // Everything else asked at a table is its game's.
            other => {
                let Room::Table(game, id) = room else {
                    continue;
                };
                let Tables { tables: all, games: rules, .. } = &mut *tables;
                let (Some(table), Some(rules)) = (all.get_mut(&(game, id)), rules.get_mut(&(game, id))) else {
                    continue;
                };
                if table.closing {
                    continue;
                }
                if let Some(seat) = table.seat_of(body) {
                    rules.hear(&mut ctx, table, seat, other);
                }
            }
        }
    }
}

/// `body`, its device `client`, asks to join a game of `game`: in it goes,
/// and with it, if it leads a group, everyone of the group who is here,
/// free to and has the fee (`social`); in a group someone else leads, nobody
/// does. Its seats held at one table for all of them, each fee is taken,
/// and each is seated as theirs is: at once without the database, and once
/// it has been taken with it ([`paid`]).
#[allow(clippy::too_many_arguments)]
fn join(
    ctx: &mut Ctx,
    tables: &mut Tables,
    (towns, arenas, games): (&mut Towns, &mut Arenas, &mut Games),
    (everyone, social, bank): (&Everyone, &mut Social, &Bank),
    paying: &Query<(), With<Paying>>,
    (body, client): (Entity, Entity),
    game: Minigame,
) {
    // The two older games are joined their own ways (`Ask::Arena`,
    // `Ask::Play`): refused, as the ask says, so that a device waiting on
    // its answer is not left waiting.
    let Some(shape) = table::shape(game) else {
        tell(ctx.tells, client, Tell::Refused(Refusal::Busy));
        return;
    };
    let Ok((body_of, ..)) = ctx.bodies.get(body) else {
        return;
    };
    let Some(party) = social.party(body_of.who.0) else {
        tell(ctx.tells, client, Tell::Refused(Refusal::NotLeader));
        return;
    };
    // Free to: in the town, or done with whatever game they are in, and not
    // paying for another already. And able to pay.
    let (mut going, mut staying) = (Vec::new(), Vec::new());
    let mut cannot_pay = false;
    for id in party {
        let entity = everyone.0.get(&Who(id)).copied();
        let able = entity.and_then(|entity| {
            let (body_of, ..) = ctx.bodies.get(entity).ok()?;
            let free = free_to_join(body_of, entity, tables, arenas, games) && !paying.contains(entity);
            Some((free, body_of.coins >= shape.fee))
        });
        match (entity, able) {
            (Some(entity), Some((true, true))) => going.push(entity),
            (Some(entity), Some((true, false))) if entity == body => cannot_pay = true,
            _ => staying.push(id),
        }
    }
    if cannot_pay {
        tell(ctx.tells, client, Tell::Refused(Refusal::Coins));
        return;
    }
    // The one who asked is first of them, or it is nobody's to go.
    if going.first() != Some(&body) {
        tell(ctx.tells, client, Tell::Refused(Refusal::Busy));
        return;
    }
    for id in staying {
        social.left_behind(id);
    }
    // One table for all of them: the one filling now, if it has their
    // seats, or a new one.
    let Some((id, seats)) = tables.filling(ctx.commands, game, going.len()) else {
        tell(ctx.tells, client, Tell::Refused(Refusal::Busy));
        return;
    };
    for (&entity, seat) in going.iter().zip(seats) {
        let Ok((mut body_of, ..)) = ctx.bodies.get_mut(entity) else {
            continue;
        };
        let account = body_of.who.0;
        // Kept in the database: taken there, in one transaction with the
        // line saying why, and seated in the seat held for them once it has
        // been ([`paid`]).
        if bank.kept() {
            tables.hold((game, id), seat, account);
            bank.charge(account, shape.fee, why(game, Coins::Entry), game, Some(id));
            ctx.commands.entity(entity).insert(Paying { game, into: Some(id) });
            continue;
        }
        body_of.coins -= shape.fee;
        if let Some(client) = body_of.client {
            tell(ctx.tells, client, Tell::Balance(body_of.coins));
        }
        sit(ctx, tables, towns, arenas, games, (game, id), seat, entity, shape.fee);
    }
}

/// A table's fee taken in the database: whoever paid it is seated where
/// their seat was held, or, that gone, wherever there is a seat for one
/// now; turned away without the coins; and with nowhere to go, or no longer
/// here, given it back. The other games' fees are their own (`builder`).
#[allow(clippy::too_many_arguments)]
fn paid(
    mut charged: MessageReader<Charged>,
    mut hands: Hands,
    mut tables: ResMut<Tables>,
    (mut towns, mut arenas, mut games): (ResMut<Towns>, ResMut<Arenas>, ResMut<Games>),
    everyone: Res<Everyone>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs_f64();
    let clock = wall_clock();
    for answer in charged.read() {
        let game = answer.game;
        let Some(shape) = table::shape(game) else {
            continue;
        };
        let mut ctx = hands.ctx(now, clock, &config);
        let held = answer
            .into
            .and_then(|id| tables.unhold(game, id, answer.id).map(|seat| (id, seat)));
        let body = everyone.0.get(&Who(answer.id)).copied();
        if let Some(body) = body {
            ctx.commands.entity(body).try_remove::<Paying>();
        }
        let device = body.and_then(|body| ctx.bodies.get(body).ok()).and_then(|(body_of, ..)| body_of.client);
        let Some(left) = answer.left else {
            if let Some(client) = device {
                tell(ctx.tells, client, Tell::Refused(Refusal::Coins));
            }
            continue;
        };
        let mut free = false;
        if let Some(body) = body
            && let Ok((mut body_of, ..)) = ctx.bodies.get_mut(body)
        {
            body_of.coins = left;
            if let Some(client) = body_of.client {
                tell(ctx.tells, client, Tell::Balance(left));
            }
            free = free_to_join(&body_of, body, &tables, &arenas, &games);
        }
        let seat = match (body, free) {
            (Some(_), true) => held.or_else(|| {
                tables
                    .filling(ctx.commands, game, 1)
                    .and_then(|(id, seats)| seats.first().map(|&seat| (id, seat)))
            }),
            _ => None,
        };
        match (body, seat) {
            (Some(body), Some((id, seat))) => {
                sit(&mut ctx, &mut tables, &mut towns, &mut arenas, &mut games, (game, id), seat, body, shape.fee);
            }
            _ => {
                info!("{}: {}'s fee is given back, with no seat for them", name(game), answer.id);
                tables.refunds.push(Refund {
                    account: answer.id,
                    fee: shape.fee,
                    game,
                    table: answer.into.unwrap_or(0),
                });
            }
        }
    }
}

/// Moves every table on: fills it, counts down, has its game played
/// ([`Rules::step`]), pays out, and ends it once its results have been up
/// their while, or once nobody is left.
#[allow(clippy::too_many_arguments)]
fn run(
    time: Res<Time>,
    tick: Res<ServerTick>,
    config: Res<Config>,
    mut tables: ResMut<Tables>,
    mut hands: Hands,
    (mut everyone, mut towns): (ResMut<Everyone>, ResMut<Towns>),
    (mut social, bank): (ResMut<Social>, Res<Bank>),
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs_f64();
    // The time by this machine's clock, which every device has for the
    // town's: what a game stamps what everyone is to see at once with.
    let clock = wall_clock();
    let Tables {
        tables: all,
        games,
        filling,
        refunds,
        ..
    } = &mut *tables;
    for refund in refunds.drain(..) {
        pay_back(&mut hands.ctx(now, clock, &config), &everyone, &bank, refund);
    }
    let mut ended = Vec::new();
    for (&key, table) in all.iter_mut() {
        let Some(rules) = games.get_mut(&key) else {
            continue;
        };
        let rules = rules.as_mut();
        let mut ctx = hands.ctx(now, clock, &config);
        // What happened to it from outside since it last ran: seats left
        // for good since the play began, for its game to hear of, with the
        // body that sat in each if it is still about, and devices back for
        // it, to begin it afresh on.
        if !table.closing {
            for (seat, body) in std::mem::take(&mut table.lefts) {
                let body = Some(body).filter(|&body| ctx.bodies.contains(body));
                rules.left(&mut ctx, table, seat, body);
            }
            for seat in std::mem::take(&mut table.backs) {
                welcome_back(&mut ctx, table, rules, seat);
            }
        }
        let watched = table.seats.iter().any(|seat| {
            seat.here()
                && ctx
                    .bodies
                    .get(seat.body)
                    .is_ok_and(|(body, ..)| body.client.is_some() && body.room == table.room)
        });
        table.unwatched = if watched { 0.0 } else { table.unwatched + dt };
        if table.closing {
            end(&mut ctx, table, &mut everyone, &mut towns);
            ended.push(key);
            continue;
        }
        // A game its last person left, over as it heard of it (Knock Off's:
        // "your place kept and paid"), is paid for before it closes.
        if table.over() && social.first_time(key.0, key.1) {
            finish_up(&mut ctx, table, &everyone, &bank, &mut social);
        }
        // Its results up their while, nobody left at it, or nobody watching
        // it for a minute: closed, but only once whoever went out of its
        // room last has been told it is gone from their sight
        // ([`Table::sent_since_anyone_went`], asked every frame).
        let told = table.sent_since_anyone_went(tick.get());
        let everyone_gone = table.people() == 0 && table.holds.is_empty();
        let done = matches!(table.stage, Stage::Over { left } if left <= 0.0);
        if (done || everyone_gone || table.unwatched > ABANDONED) && told {
            close(&mut ctx, table, rules);
            continue;
        }

        sync(&mut ctx, table);
        match table.stage {
            Stage::Waiting { .. } => {
                wait(&mut ctx, table, rules, dt);
                rules.idle(&mut ctx, table, dt);
            }
            Stage::Countdown { left } => {
                let left = left - dt;
                table.stage = Stage::Countdown { left };
                rules.idle(&mut ctx, table, dt);
                // Counted down: the game begins; a moment more, though, for
                // whoever sat down in its last moment, their fee still being
                // taken, so that they play ([`HELD_MOST`]).
                if left <= 0.0 && (table.holds.is_empty() || left <= -HELD_MOST) {
                    begin(&mut ctx, table, rules, &mut everyone, refunds);
                }
            }
            Stage::Playing { elapsed } => {
                table.stage = Stage::Playing { elapsed: elapsed + dt };
                rules.step(&mut ctx, table, dt);
            }
            Stage::Over { left } => table.stage = Stage::Over { left: left - dt },
        }
        flush(&mut ctx, table);

        // Over: everyone is who their account is again, is paid by their
        // place, and, if they played it to its end, earns toward their
        // level (`social`). Once a table.
        if table.over() && social.first_time(key.0, key.1) {
            finish_up(&mut ctx, table, &everyone, &bank, &mut social);
        }

        // Only what has changed goes out again.
        rules.show(ctx.commands, table);
        show_board(&mut ctx, table);
    }
    for (game, id) in ended {
        all.remove(&(game, id));
        games.remove(&(game, id));
        if filling.get(&game) == Some(&id) {
            filling.remove(&game);
        }
    }
}

/// A frame of a table waiting for people, `dt` seconds on. With the
/// computer: once nobody has joined for its wait (or someone has skipped
/// it, or its people fill it), how many seats it has is settled and the
/// computer comes for the seats left, one at a time, a throw of
/// [`BOT_GAP`] apart (as Bridge Wars' come, since 2026-11-16: Hajun, "make
/// them join slightly delay one by one rather than all at once"), whoever
/// joins or leaves meanwhile; with every seat taken, the countdown. Without
/// it: once it has as many people as it begins with and nobody has joined
/// for its wait (or someone has skipped it), or it is full, the countdown.
fn wait(ctx: &mut Ctx, table: &mut Table, rules: &mut dyn Rules, dt: f32) {
    let Stage::Waiting { since_join, coming } = table.stage else {
        return;
    };
    let since_join = since_join + dt;
    let people = table.people();
    let shape = table.shape;
    if !shape.computer {
        table.size = shape.most;
        let due = people >= shape.fewest && (since_join >= shape.wait || table.hurried);
        if due || people >= shape.most {
            info!("{} table {}: {people} people, and it counts down", name(table.game), table.id);
            table.stage = Stage::Countdown { left: shape.countdown };
        } else {
            table.stage = Stage::Waiting { since_join, coming };
        }
        return;
    }
    let mut coming = coming;
    if coming.is_none() {
        let coming_people = people + table.holds.len();
        table.size = table::seats_for(table.game, coming_people).min(table.seats.len());
        let due = since_join >= shape.wait || table.hurried || people >= table.size;
        if coming_people > 0 && due {
            // Two people at Bridge Race: on one team, one on each, or one
            // against the other, a third of the time each, thrown for the
            // table (`bridge_race::pairing`, since the evening of
            // 2026-11-18): the one in red's far seat moved to blue's first
            // for the last two, as `close_ranks` moves anyone past the
            // seats a game has.
            let pairing = (table.game == Minigame::BridgeRace && coming_people == 2)
                .then(|| bridge_race::pairing(table.dice.f32()));
            match pairing {
                Some(bridge_race::Pairing::Apart) => close_ranks(ctx, table, rules, 2, true),
                Some(bridge_race::Pairing::Alone) => table.size = 2,
                Some(bridge_race::Pairing::Together) | None => {}
            }
            if let Some(pairing) = pairing {
                info!("{} table {}: two people, {pairing:?}", name(table.game), table.id);
            }
            let size = table.size;
            close_ranks(ctx, table, rules, size, true);
            coming = Some(0.0);
            info!(
                "{} table {}: {people} people, and the computer comes for the seats left of {}",
                name(table.game),
                table.id,
                table.size
            );
        }
    }
    let Some(mut next) = coming else {
        table.stage = Stage::Waiting { since_join, coming };
        return;
    };
    let mut due = if table.hurried {
        usize::MAX
    } else {
        next -= dt;
        if next <= 0.0 {
            next = BOT_GAP.0 + (BOT_GAP.1 - BOT_GAP.0) * table.dice.f32();
            1
        } else {
            0
        }
    };
    let order = table.order();
    while due > 0
        && let Some(seat) = order.iter().rev().copied().find(|&seat| seat < table.size && table.open(seat))
    {
        seat_computer(ctx, table, rules, seat);
        due -= 1;
    }
    let full = (0..table.size).all(|seat| !table.seats[seat].vacant) && table.holds.is_empty();
    table.stage = if full {
        info!("{} table {}: every seat taken, and it counts down", name(table.game), table.id);
        Stage::Countdown { left: shape.countdown }
    } else {
        Stage::Waiting {
            since_join,
            coming: Some(next),
        }
    };
}

/// The countdown is over: the game begins. Whoever's device went while the
/// table waited, and is not back, has given their seat up, their fee given
/// back; a table without the computer that has too few people for it now
/// waits again; a seat still held for whoever's fee is being taken is held
/// no more, and they sit down at another table as it is taken ([`paid`]);
/// the computer takes every seat left in a game with it, and in one without
/// its people are numbered from 0 ([`close_ranks`]); and the game stands
/// everyone where it begins ([`Rules::begin`]).
fn begin(ctx: &mut Ctx, table: &mut Table, rules: &mut dyn Rules, everyone: &mut Everyone, refunds: &mut Vec<Refund>) {
    for seat in 0..table.seats.len() {
        if !table.seats[seat].here() {
            continue;
        }
        let body = table.seats[seat].body;
        let Ok((body_of, ..)) = ctx.bodies.get(body) else {
            continue;
        };
        if body_of.client.is_some() {
            continue;
        }
        info!("{} table {}: seat {seat}'s device is not back as the game begins, and the seat is free again", name(table.game), table.id);
        everyone.0.remove(&body_of.who);
        ctx.commands.entity(body).try_despawn();
        if let Some((account, fee)) = table.leave(seat) {
            refunds.push(Refund {
                account,
                fee,
                game: table.game,
                table: table.id,
            });
        }
    }
    let people = table.people();
    if !table.shape.computer && people < table.shape.fewest {
        table.stage = Stage::Waiting {
            since_join: 0.0,
            coming: None,
        };
        return;
    }
    if people == 0 {
        return;
    }
    // Before anyone is moved up: a seat held for someone who is not seated
    // yet would keep a person past the seats the game has out of the one
    // left for them, and the game would begin without them, and a seat
    // short.
    if !table.holds.is_empty() {
        info!("{} table {}: begins without whoever's fee is still being taken", name(table.game), table.id);
        table.holds.clear();
    }
    if table.shape.computer {
        let order = table.order();
        for seat in order {
            if seat < table.size && table.seats[seat].vacant {
                seat_computer(ctx, table, rules, seat);
            }
        }
    } else {
        table.size = people;
        close_ranks(ctx, table, rules, people, false);
    }
    let size = table.size.min(table.seats.len());
    table.seats.truncate(size);
    table.at.truncate(size);
    table.yaw.truncate(size);
    table.stage = Stage::Playing { elapsed: 0.0 };
    table.play_left = None;
    table.hurried = false;
    let cast: Vec<String> = table
        .seats
        .iter()
        .enumerate()
        .map(|(seat, it)| format!("{seat} {}{}", it.member.name, if it.computer { " (the computer)" } else { "" }))
        .collect();
    info!("{} table {}: the game begins, {}", name(table.game), table.id, cast.join(", "));
    rules.begin(ctx, table);
}

/// Copies into `table` where each person at it last said they stood
/// ([`Table::at`]), and whether their device has gone ([`Seat::away`]):
/// what its game goes by. The computer's players are where the game has
/// them.
fn sync(ctx: &mut Ctx, table: &mut Table) {
    for seat in 0..table.seats.len() {
        if !table.seats[seat].here() {
            continue;
        }
        let Ok((body, spot, ..)) = ctx.bodies.get(table.seats[seat].body) else {
            continue;
        };
        if body.room == table.room {
            table.at[seat] = spot.translation();
            table.yaw[seat] = spot.yaw();
        }
        table.seats[seat].away = body.client.is_none();
    }
}

/// Sends each of the computer's players at `table` where its game has moved
/// it, if it has ([`Ctx::move_body`]).
fn flush(ctx: &mut Ctx, table: &Table) {
    for (seat, it) in table.seats.iter().enumerate() {
        if it.computer && it.body != Entity::PLACEHOLDER {
            ctx.move_body(it.body, table.at[seat], table.yaw[seat]);
        }
    }
}

/// `seat`'s device is back for the game at `table`, which is begun afresh
/// on it: it is told it is at the table ([`Tell::Entered`]), told by the
/// game what only it needs ([`Rules::back`]), and stood where its body
/// stands.
fn welcome_back(ctx: &mut Ctx, table: &mut Table, rules: &mut dyn Rules, seat: usize) {
    let Some(it) = table.seats.get(seat) else {
        return;
    };
    if !it.here() {
        return;
    }
    let body = it.body;
    let Ok((body_of, ..)) = ctx.bodies.get(body) else {
        return;
    };
    if body_of.room != table.room {
        return;
    }
    if let Some(client) = body_of.client {
        tell(ctx.tells, client, Tell::Entered { game: table.game, seat: seat as u8 });
    }
    rules.back(ctx, table, seat);
    let Ok((mut body_of, spot, venue, _)) = ctx.bodies.get_mut(body) else {
        return;
    };
    let (spot, venue) = (*spot, *venue);
    stand_where_it_is(ctx.tells, &mut body_of, spot, venue, ctx.now);
}

/// A table's game is over, the first frame of its results: everyone at it
/// is who their account is again, every person is paid what their place is
/// worth by the game's own table of prizes (`results.prizes`), wherever
/// they are now (someone who left still came where they came), into their
/// account in the database, whose answer tells their device
/// (`fees::credited`), or without one, onto their body; and whoever played
/// it to its end earns toward their level, more for coming first
/// (`social`): whoever is placed first, those tied there alike, but nobody
/// when every seat is placed first, a draw (Bridge Race's; two people at
/// Blend In who both found the other, or neither), as nobody does in a
/// game of Bridge Wars that is drawn; and so nobody playing alone (mine).
fn finish_up(ctx: &mut Ctx, table: &Table, everyone: &Everyone, bank: &Bank, social: &mut Social) {
    let Some(results) = &table.results else {
        return;
    };
    for seat in 0..table.seats.len() {
        ctx.undisguise(table, seat);
    }
    for (it, &prize) in table.seats.iter().zip(&results.prizes) {
        if !it.person() || prize == 0 {
            continue;
        }
        if bank.kept() {
            bank.credit(it.member.id, prize, why(table.game, Coins::Prize), Some(table.id));
            continue;
        }
        let Some(&body) = everyone.0.get(&Who(it.member.id)) else {
            continue;
        };
        let Ok((mut body_of, ..)) = ctx.bodies.get_mut(body) else {
            continue;
        };
        body_of.coins = body_of.coins.saturating_add(prize);
        if let Some(client) = body_of.client {
            tell(ctx.tells, client, Tell::Balance(body_of.coins));
        }
    }
    let anyone_won = results.places.iter().any(|&place| place > 0);
    for (it, &place) in table.seats.iter().zip(&results.places) {
        // (Nor whoever's device has gone, their seat waiting.)
        if it.here() && !it.away {
            let won = anyone_won && place == 0;
            social.award(it.member.id, XP_PLAYED + if won { XP_WON } else { 0 });
        }
    }
    info!(
        "{} table {}: over, places {:?}, scores {:?}, total {:?}, paid {:?}",
        name(table.game),
        table.id,
        results.places,
        results.scores,
        results.total,
        results.prizes
    );
}

/// A fee given back, into the account in the database, whose answer tells
/// its device (`fees::credited`), or without one, onto its body, if it is
/// still here.
fn pay_back(ctx: &mut Ctx, everyone: &Everyone, bank: &Bank, refund: Refund) {
    info!(
        "{} table {}: {}'s fee of {} is given back",
        name(refund.game),
        refund.table,
        refund.account,
        refund.fee
    );
    if bank.kept() {
        bank.credit(refund.account, refund.fee, why(refund.game, Coins::Refund), Some(refund.table));
        return;
    }
    let Some(&body) = everyone.0.get(&Who(refund.account)) else {
        return;
    };
    let Ok((mut body_of, ..)) = ctx.bodies.get_mut(body) else {
        return;
    };
    body_of.coins = body_of.coins.saturating_add(refund.fee);
    if let Some(client) = body_of.client {
        tell(ctx.tells, client, Tell::Balance(body_of.coins));
    }
}

/// Sends the table's room its board and who sits where, each only when it
/// has changed.
fn show_board(ctx: &mut Ctx, table: &mut Table) {
    let board = table.board();
    if table.sent_board.as_ref() != Some(&board) {
        ctx.commands.entity(table.shown).insert(board.clone());
        table.sent_board = Some(board);
    }
    let same = table.sent_seats.as_ref().is_some_and(|sent| {
        sent.len() == table.seats.len() && sent.iter().zip(&table.seats).all(|(sent, it)| *sent == it.member)
    });
    if !same {
        let members: Vec<Member> = table.seats.iter().map(|it| it.member.clone()).collect();
        ctx.commands.entity(table.shown).insert(Seats(members.clone()));
        table.sent_seats = Some(members);
    }
}

/// The first frame of a table's end: its computer players, whatever its
/// game spawned and its board go, while everyone at it is still in its room
/// to be told so. Told the frame after, as [`end`] moves them out, they
/// would never be: an entity despawned in the very tick a device stops
/// seeing its room is written to that device neither as despawned nor as
/// lost from sight (replicon 0.44's `collect_despawns`; Bridge Wars' bots
/// stood on in the next game for a night, 2026-11-09).
fn close(ctx: &mut Ctx, table: &mut Table, rules: &mut dyn Rules) {
    for it in &table.seats {
        if it.computer && it.body != Entity::PLACEHOLDER {
            ctx.commands.entity(it.body).try_despawn();
        }
    }
    rules.close(ctx.commands, table);
    ctx.commands.entity(table.shown).try_despawn();
    table.closing = true;
    info!("{} table {} closing", name(table.game), table.id);
}

/// The end of a table, the frame after [`close`]: everyone still in its
/// room, seated or not (nobody should be there unseated, but a body left
/// there would otherwise be there for good), is who their account is again
/// and goes back to the town, or, if their device has gone, is forgotten.
fn end(ctx: &mut Ctx, table: &Table, everyone: &mut Everyone, towns: &mut Towns) {
    let inside: Vec<(Who, Entity)> = everyone
        .0
        .iter()
        .filter(|&(_, &body)| ctx.bodies.get(body).is_ok_and(|(body_of, ..)| body_of.room == table.room))
        .map(|(&who, &body)| (who, body))
        .collect();
    for (who, body) in inside {
        let Ok((body_of, ..)) = ctx.bodies.get(body) else {
            continue;
        };
        if body_of.client.is_none() {
            everyone.0.remove(&who);
            ctx.commands.entity(body).try_despawn();
            continue;
        }
        undisguise(ctx.commands, body, ctx.accounts);
        let Ok((mut body_of, spot, venue, _)) = ctx.bodies.get_mut(body) else {
            continue;
        };
        let (channel, town_seat) = towns.sit(body, body_of.who.0, body_of.town);
        body_of.town = Some(channel);
        move_to(
            ctx.commands,
            ctx.tells,
            body,
            (body_of, spot, venue),
            Room::Town(channel),
            Venue::Town,
            town_seat,
            ctx.now,
        );
    }
    info!("{} table {} closed", name(table.game), table.id);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use bevy::ecs::message::Messages;
    use bevy::ecs::system::{RunSystemOnce, SystemId};

    use super::*;
    use crate::accounts::{Credited, Done, Job, Loaded, Snapshot};
    use crate::players::Account;
    use crate::rooms::ClientRoom;

    const BLEND_IN: Minigame = Minigame::BlendIn;
    const KNOCK_OFF: Minigame = Minigame::KnockOff;
    const BRIDGE_RACE: Minigame = Minigame::BridgeRace;
    const LAST_STAND: Minigame = Minigame::LastStand;

    /// The server with no network and no app: a world of its own holding
    /// everything the tables go by, and their systems run on it frame by
    /// frame in the order the server runs them (with House Builder's fees
    /// beside theirs); with a database, its thread played by the test,
    /// which sees each job asked of it and answers as it likes.
    struct Server {
        world: World,
        systems: Vec<SystemId>,
        jobs: Option<Receiver<Job>>,
        answers: Option<Sender<Done>>,
        /// Everything every device was told, in order.
        told: Vec<(Entity, Tell)>,
        /// Everything the database was asked to do, in order.
        asked: Vec<Job>,
    }

    impl Server {
        fn new(kept: bool) -> Self {
            let mut world = World::new();
            world.insert_resource(Time::<()>::default());
            world.init_resource::<ServerTick>();
            world.insert_resource(Config {
                port: 0,
                key: None,
                public: Vec::new(),
                db: None,
                allow_skip: true,
                arena_from: 0.0,
                arena_purse: 0,
                arena_old_bots: false,
            });
            let (mut jobs, mut answers) = (None, None);
            let bank = if kept {
                let (to_db, asked) = channel();
                let (answer, from_db) = channel();
                jobs = Some(asked);
                answers = Some(answer);
                Bank::Database {
                    jobs: to_db,
                    done: Mutex::new(from_db),
                }
            } else {
                Bank::Memory
            };
            world.insert_resource(bank);
            world.insert_resource(Social::new(Snapshot::default()));
            world.init_resource::<Everyone>();
            world.init_resource::<Towns>();
            world.init_resource::<Tables>();
            world.init_resource::<Arenas>();
            world.init_resource::<Games>();
            world.init_resource::<Messages<FromClient<Ask>>>();
            world.init_resource::<Messages<ToClients<Tell>>>();
            world.init_resource::<Messages<Loaded>>();
            world.init_resource::<Messages<Charged>>();
            world.init_resource::<Messages<Credited>>();
            world.init_resource::<Messages<crate::accounts::Dealt>>();
            let systems = vec![
                world.register_system(crate::accounts::answers),
                world.register_system(crate::fees::credited),
                world.register_system(paid),
                world.register_system(crate::builder::paid),
                world.register_system(hear),
                world.register_system(run),
            ];
            Self {
                world,
                systems,
                jobs,
                answers,
                told: Vec::new(),
                asked: Vec::new(),
            }
        }

        /// Someone in the town, the account `id` with `coins` to spend: their
        /// device and their body, as `players::admit` lets them in, known
        /// to `social` by their name, with nothing earned yet.
        fn person(&mut self, id: u64, coins: u32) -> (Entity, Entity) {
            let client = self.world.spawn_empty().id();
            let body = self.world.spawn_empty().id();
            let (channel, seat) = self.world.resource_mut::<Towns>().sit(body, id, None);
            let room = Room::Town(channel);
            let member = Member {
                id,
                name: format!("Person {id}"),
                look: "boy".into(),
                ai: false,
            };
            self.world.resource_mut::<Social>().enroll(id, &member.name, None, 0);
            let (at, turn) = roundtown_net::arrival(Venue::Town, usize::from(seat));
            self.world.entity_mut(body).insert((
                Replicated,
                member.clone(),
                Account(member),
                Spot::of(at, turn),
                Venue::Town,
                InRoom(room),
                Body::person(client, Who(id), room, seat, Some(channel), coins, 0.0),
            ));
            self.world.entity_mut(client).insert((Plays(body), ClientRoom(room)));
            self.world.resource_mut::<Everyone>().0.insert(Who(id), body);
            (client, body)
        }

        fn ask(&mut self, client: Entity, ask: Ask) {
            self.world.write_message(FromClient {
                client_id: ClientId::Client(client),
                message: ask,
            });
        }

        /// The database's answer to a charge, as its thread would give it.
        fn charged(&mut self, answer: Charged) {
            self.answers
                .as_ref()
                .expect("a database")
                .send(Done::Charged(answer))
                .expect("the server is listening");
        }

        /// One frame, `dt` seconds on, with a send of the rooms in it (the
        /// tick moves, as replicon's does on the server's fixed steps): what
        /// devices were told and what the database was asked kept to be
        /// looked at.
        fn frame(&mut self, dt: f32) {
            self.world.resource_mut::<Time>().advance_by(Duration::from_secs_f32(dt));
            self.world.resource_mut::<ServerTick>().increment();
            for system in self.systems.clone() {
                self.world.run_system(system).expect("a system the tables run by");
            }
            let told: Vec<ToClients<Tell>> = self.world.resource_mut::<Messages<ToClients<Tell>>>().drain().collect();
            for told in told {
                if let SendTargets::Single(ClientId::Client(client)) = told.targets {
                    self.told.push((client, told.message));
                }
            }
            if let Some(jobs) = &self.jobs {
                self.asked.extend(jobs.try_iter());
            }
            self.world.resource_mut::<Messages<FromClient<Ask>>>().update();
            self.world.resource_mut::<Messages<Loaded>>().update();
            self.world.resource_mut::<Messages<Charged>>().update();
            self.world.resource_mut::<Messages<Credited>>().update();
            self.world.resource_mut::<Messages<crate::accounts::Dealt>>().update();
        }

        /// Frames of a tenth of a second until `until` holds, for at most
        /// `most` seconds: how long it took.
        fn until(&mut self, most: f32, until: impl Fn(&Self) -> bool) -> f32 {
            let mut took = 0.0;
            while !until(self) {
                assert!(took < most, "not within {most} s");
                self.frame(0.1);
                took += 0.1;
            }
            took
        }

        fn table(&self, game: Minigame, id: u32) -> &Table {
            &self.world.resource::<Tables>().tables[&(game, id)]
        }

        fn table_mut(&mut self, game: Minigame, id: u32) -> Mut<'_, Table> {
            self.world
                .resource_mut::<Tables>()
                .map_unchanged(|tables| tables.tables.get_mut(&(game, id)).expect("the table"))
        }

        fn body(&self, body: Entity) -> &Body {
            self.world.get::<Body>(body).expect("a body")
        }

        /// Everything `client` was told, in order.
        fn heard(&self, client: Entity) -> Vec<&Tell> {
            self.told.iter().filter(|(to, _)| *to == client).map(|(_, tell)| tell).collect()
        }

        /// The game and the seat `client` was last told it sits in.
        fn entered(&self, client: Entity) -> Option<(Minigame, u8)> {
            self.heard(client).iter().rev().find_map(|tell| match tell {
                Tell::Entered { game, seat } => Some((*game, *seat)),
                _ => None,
            })
        }

        /// How many times `client` was told it is at a table.
        fn times_entered(&self, client: Entity) -> usize {
            self.heard(client).iter().filter(|tell| matches!(tell, Tell::Entered { .. })).count()
        }

        /// The venue and the seat `client` was last stood in afresh.
        fn arrived(&self, client: Entity) -> Option<(Venue, u8)> {
            self.heard(client).iter().rev().find_map(|tell| match tell {
                Tell::Arrive { venue, seat, .. } => Some((*venue, *seat)),
                _ => None,
            })
        }

        /// What `client` was last told it has.
        fn balance(&self, client: Entity) -> Option<u32> {
            self.heard(client).iter().rev().find_map(|tell| match tell {
                Tell::Balance(coins) => Some(*coins),
                _ => None,
            })
        }

        /// What the account `id` has earned toward its level.
        fn xp(&self, id: u64) -> u32 {
            self.world.resource::<Social>().person(id).map_or(0, |person| person.xp)
        }

        /// How many of the computer's players sit at a table.
        fn computers(&self, game: Minigame, id: u32) -> usize {
            self.table(game, id).seats.iter().filter(|seat| seat.computer).count()
        }

        /// Through the countdown and into the game, skipping what can be.
        fn skip_to_the_game(&mut self, client: Entity, game: Minigame, id: u32) {
            self.ask(client, Ask::Skip);
            self.frame(0.1);
            self.until(10.0, |server| server.table(game, id).playing());
        }

        /// Has the game at a table do what its own play would, with what a
        /// game is handed ([`Ctx`]).
        fn act(&mut self, game: Minigame, id: u32, deed: impl Fn(&mut Ctx, &mut Table) + Send + Sync + 'static) {
            self.world
                .run_system_once(move |mut hands: Hands, mut tables: ResMut<Tables>, config: Res<Config>| {
                    let mut ctx = hands.ctx(0.0, 0.0, &config);
                    let table = tables.tables.get_mut(&(game, id)).expect("the table");
                    deed(&mut ctx, table);
                })
                .expect("the deed");
        }
    }

    #[test]
    fn someone_alone_plays_knock_off_with_the_computer_from_the_join_to_the_town() {
        let mut server = Server::new(false);
        let (client, body) = server.person(1, 1000);
        server.ask(client, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        // Seated at once with no database: the fee taken and told, at the
        // first table of the game, seat 0, on the ice, in the table's room.
        let room = Room::Table(KNOCK_OFF, 0);
        assert_eq!(server.entered(client), Some((KNOCK_OFF, 0)));
        assert_eq!((server.body(body).coins, server.balance(client)), (950, Some(950)));
        assert_eq!(server.body(body).room, room);
        assert_eq!(server.world.get::<Venue>(body), Some(&Venue::KnockOff));
        assert_eq!(server.world.get::<ClientRoom>(client), Some(&ClientRoom(room)));
        let spot = server.world.get::<Spot>(body).expect("a spot").translation();
        assert!((spot - knock_off::arrival(0).0).length() < 0.02, "{spot}");
        assert!(matches!(
            server.heard(client)[..],
            [Tell::Balance(950), Tell::Entered { .. }, Tell::Arrive { venue: Venue::KnockOff, seat: 0, .. }]
        ));
        let shown = server.table(KNOCK_OFF, 0).shown;
        let board = server.world.get::<Board>(shown).expect("the board").clone();
        assert_eq!((board.stage, board.left, board.people, board.seats), (table::Stage::Waiting, 10, 1, 4));
        // Nobody else comes: ten seconds after them, the computer, for the
        // three seats left, one at a time, the last seats first.
        let waited = server.until(30.0, |server| server.computers(KNOCK_OFF, 0) > 0);
        assert!((9.5..=10.5).contains(&waited), "{waited}");
        assert!(server.table(KNOCK_OFF, 0).seats[3].computer);
        let mut came = vec![server.computers(KNOCK_OFF, 0)];
        while !matches!(server.table(KNOCK_OFF, 0).stage, Stage::Countdown { .. }) {
            assert!(came.len() < 100, "{came:?}");
            server.frame(0.1);
            came.push(server.computers(KNOCK_OFF, 0));
        }
        assert_eq!(came.last(), Some(&3));
        assert!(came.windows(2).all(|pair| pair[1] <= pair[0] + 1), "one at a time: {came:?}");
        assert!(came.iter().filter(|&&count| count == 2).count() >= 2, "a while apart: {came:?}");
        let table = server.table(KNOCK_OFF, 0);
        let computers: Vec<Entity> = table.seats.iter().filter(|seat| seat.computer).map(|seat| seat.body).collect();
        for seat in 1..4 {
            let it = &table.seats[seat];
            let member = server.world.get::<Member>(it.body).expect("the computer's body");
            assert_eq!((member.id, member.ai, it.paid), (bot_id(KNOCK_OFF, 0, seat), true, 0));
            assert_eq!(server.world.get::<InRoom>(it.body), Some(&InRoom(room)));
            assert_eq!(server.world.get::<Venue>(it.body), Some(&Venue::KnockOff));
        }
        // Five seconds counted down, and the game.
        let counted = server.until(10.0, |server| server.table(KNOCK_OFF, 0).playing());
        assert!((4.5..=5.5).contains(&counted), "{counted}");
        assert_eq!(server.table(KNOCK_OFF, 0).seats.len(), 4);
        // The game is Knock Off's own, played by the clock (`knock_off`,
        // whose own tests play it through): here it is over as it says it
        // is, seat 0 first: the person is paid first's 130, the computer
        // nothing.
        server.frame(0.1);
        assert!(server.table(KNOCK_OFF, 0).playing());
        server.table_mut(KNOCK_OFF, 0).finish(vec![0, 1, 2, 3], vec![4, 3, 2, 1], None);
        server.frame(0.1);
        let results = server.table(KNOCK_OFF, 0).results.clone().expect("how it came out");
        assert_eq!((results.places, results.prizes), (vec![0, 1, 2, 3], vec![130, 0, 0, 0]));
        assert_eq!((server.body(body).coins, server.balance(client)), (1080, Some(1080)));
        let board = server.world.get::<Board>(shown).expect("the board").clone();
        assert!(matches!(&board.stage, table::Stage::Over(shown) if shown.prizes == vec![130, 0, 0, 0]), "{board:?}");
        assert_eq!(board.left, 15);
        // Fifteen seconds of results, and it closes: the computer's bodies
        // and the board go while the person is still in the room...
        server.until(20.0, |server| server.table(KNOCK_OFF, 0).closing);
        assert!(computers.iter().all(|&body| server.world.get_entity(body).is_err()));
        assert!(server.world.get_entity(shown).is_err());
        assert_eq!(server.body(body).room, room);
        // ...and the frame after they are back in the town, the table gone.
        server.frame(0.1);
        assert!(matches!(server.body(body).room, Room::Town(_)));
        assert!(matches!(server.heard(client).last(), Some(Tell::Arrive { venue: Venue::Town, .. })));
        assert!(!server.world.resource::<Tables>().tables.contains_key(&(KNOCK_OFF, 0)));
        assert_eq!(server.body(body).coins, 1080);
    }

    #[test]
    fn blend_in_never_begins_with_one_person_and_does_with_two_once_nobody_has_joined_for_its_wait() {
        let mut server = Server::new(false);
        let (one, _) = server.person(1, 1000);
        server.ask(one, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert_eq!(server.entered(one), Some((BLEND_IN, 0)));
        // Alone, however long: nothing counts down, and the computer never
        // comes ("This game min player count is 2").
        for _ in 0..600 {
            server.frame(0.1);
        }
        let table = server.table(BLEND_IN, 0);
        assert!(matches!(table.stage, Stage::Waiting { .. }));
        assert_eq!((table.board().left, table.board().people, table.board().seats), (0, 1, 4));
        assert_eq!(server.computers(BLEND_IN, 0), 0);
        // A second: the wait counts from them, and once it is over the
        // countdown, and the game, the two of them alone.
        let (two, _) = server.person(2, 1000);
        server.ask(two, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert_eq!(server.entered(two), Some((BLEND_IN, 1)));
        assert_eq!(server.table(BLEND_IN, 0).board().left, 15);
        let waited = server.until(30.0, |server| matches!(server.table(BLEND_IN, 0).stage, Stage::Countdown { .. }));
        assert!((14.5..=15.5).contains(&waited), "{waited}");
        server.until(10.0, |server| server.table(BLEND_IN, 0).playing());
        let table = server.table(BLEND_IN, 0);
        assert_eq!((table.size, table.seats.len(), table.people()), (2, 2, 2));
        assert!(table.seats.iter().all(Seat::here));
        assert_eq!(table.board().seats, 2);
    }

    #[test]
    fn bridge_race_alone_is_one_against_one_and_two_people_are_one_team() {
        let mut server = Server::new(false);
        let (alone, _) = server.person(1, 1000);
        server.ask(alone, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        assert_eq!(server.entered(alone), Some((BRIDGE_RACE, 0)));
        assert_eq!(server.table(BRIDGE_RACE, 0).board().seats, 2, "one against one");
        server.until(30.0, |server| server.table(BRIDGE_RACE, 0).playing());
        let table = server.table(BRIDGE_RACE, 0);
        assert_eq!(table.seats.len(), 2);
        assert!(table.seats[0].here() && table.seats[1].computer);
        assert_ne!(bridge_race::team(0), bridge_race::team(1));
        // Two people, one after the other, at the next table: red's two
        // seats as they wait; and as the computer comes, as the table
        // throws it (`bridge_race::pairing`, since the evening of
        // 2026-11-18), on one team against two of the computer's, one on
        // each team with one of the computer's, or one against the other.
        let (first, first_body) = server.person(2, 1000);
        let (second, _) = server.person(3, 1000);
        server.ask(first, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        server.ask(second, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        assert_eq!(server.body(first_body).room, Room::Table(BRIDGE_RACE, 1));
        assert_eq!((server.entered(first), server.entered(second)), (Some((BRIDGE_RACE, 0)), Some((BRIDGE_RACE, 2))));
        assert_eq!(server.table(BRIDGE_RACE, 1).board().seats, 4, "two against two, as they wait");
        server.until(30.0, |server| server.table(BRIDGE_RACE, 1).playing());
        let table = server.table(BRIDGE_RACE, 1);
        let people: Vec<usize> = (0..table.seats.len()).filter(|&seat| table.seats[seat].here()).collect();
        let computer: Vec<usize> = (0..table.seats.len()).filter(|&seat| table.seats[seat].computer).collect();
        match table.seats.len() {
            // One against the other: red's first seat and blue's.
            2 => assert_eq!((people.as_slice(), computer.as_slice()), (&[0, 1][..], &[][..])),
            // On one team, or one on each.
            4 if people == [0, 2] => assert_eq!(computer, [1, 3]),
            4 => assert_eq!((people.as_slice(), computer.as_slice()), (&[0, 1][..], &[2, 3][..])),
            seats => panic!("{seats} seats"),
        }
    }

    #[test]
    fn a_group_goes_into_bridge_race_two_by_two_onto_a_team() {
        // Three: two on red, the third on blue with the computer.
        let mut server = Server::new(false);
        let (a, _) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        let (c, _) = server.person(3, 1000);
        server.world.resource_mut::<Social>().band(&[1, 2, 3]);
        // Only the leader starts a game.
        server.ask(b, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        assert!(matches!(server.heard(b).last(), Some(Tell::Refused(Refusal::NotLeader))));
        server.ask(a, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        let seats = [server.entered(a), server.entered(b), server.entered(c)];
        assert_eq!(seats, [Some((BRIDGE_RACE, 0)), Some((BRIDGE_RACE, 2)), Some((BRIDGE_RACE, 1))]);
        server.until(30.0, |server| server.table(BRIDGE_RACE, 0).playing());
        assert!(server.table(BRIDGE_RACE, 0).seats[3].computer);
        // Someone alone, and then a group of two: the group blue together.
        let mut server = Server::new(false);
        let (alone, _) = server.person(1, 1000);
        let (leader, _) = server.person(2, 1000);
        let (friend, _) = server.person(3, 1000);
        server.world.resource_mut::<Social>().band(&[2, 3]);
        server.ask(alone, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        server.ask(leader, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        let seats = [server.entered(alone), server.entered(leader), server.entered(friend)];
        assert_eq!(seats, [Some((BRIDGE_RACE, 0)), Some((BRIDGE_RACE, 1)), Some((BRIDGE_RACE, 3))]);
    }

    #[test]
    fn whoever_is_left_past_the_seats_a_game_has_is_moved_up_before_it_begins() {
        // Bridge Race: two people, and the first leaves. The second, alone
        // in red's far seat, is moved to its first as the computer comes,
        // stood where that seat waits, and plays one against one; told so
        // by being stood there, not by being told again that it is at the
        // table, which its device would take for a table begun afresh.
        let mut server = Server::new(false);
        let (a, _) = server.person(1, 1000);
        let (b, body_b) = server.person(2, 1000);
        server.ask(a, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        server.ask(b, Ask::Join(BRIDGE_RACE));
        server.frame(0.1);
        assert_eq!(server.entered(b), Some((BRIDGE_RACE, 2)));
        server.ask(a, Ask::Leave);
        server.frame(0.1);
        server.until(30.0, |server| server.computers(BRIDGE_RACE, 0) > 0);
        assert_eq!(server.arrived(b), Some((Venue::BridgeRace, 0)));
        assert_eq!(server.body(body_b).seat, 0);
        let spot = server.world.get::<Spot>(body_b).expect("a spot").translation();
        assert!((spot - bridge_race::waiting_spot(0)).length() < 0.02, "{spot}");
        server.until(30.0, |server| server.table(BRIDGE_RACE, 0).playing());
        assert_eq!((server.entered(b), server.times_entered(b)), (Some((BRIDGE_RACE, 2)), 1));
        let table = server.table(BRIDGE_RACE, 0);
        assert_eq!(table.seats.len(), 2);
        assert!(table.seats[0].body == body_b && table.seats[1].computer);
        assert_eq!(server.body(body_b).seat, 0);
        // Last Stand: three people, and the second leaves. The third is
        // numbered 1 as the game begins, with two seats, and stood afresh
        // where they stand in it, whether or not the game stands them
        // anywhere as it begins (this one does not).
        let mut server = Server::new(false);
        let (a, _) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        let (c, body_c) = server.person(3, 1000);
        for client in [a, b, c] {
            server.ask(client, Ask::Join(LAST_STAND));
            server.frame(0.1);
        }
        server.ask(b, Ask::Leave);
        server.frame(0.1);
        assert_eq!(server.entered(c), Some((LAST_STAND, 2)));
        server.world.resource_mut::<Tables>().games.insert((LAST_STAND, 0), Box::new(OverAsTheyGo));
        let stood = server.world.get::<Spot>(body_c).expect("a spot").translation();
        server.until(30.0, |server| server.table(LAST_STAND, 0).playing());
        assert_eq!((server.entered(c), server.times_entered(c)), (Some((LAST_STAND, 2)), 1));
        let table = server.table(LAST_STAND, 0);
        assert_eq!((table.size, table.seats.len()), (2, 2));
        assert_eq!(table.seats[1].body, body_c);
        assert_eq!(server.body(body_c).seat, 1);
        match server.heard(c).last() {
            Some(Tell::Arrive {
                venue: Venue::LastStand,
                seat: 1,
                spot: Some(spot),
                ..
            }) => assert!((spot.translation() - stood).length() < 0.02),
            other => panic!("not stood afresh in seat 1: {other:?}"),
        }
    }

    #[test]
    fn whoever_leaves_before_the_game_has_their_fee_back_and_their_seat_is_anyone_s() {
        // With no database: on their body, at once.
        let mut server = Server::new(false);
        let (a, body) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        server.ask(a, Ask::Join(LAST_STAND));
        server.frame(0.1);
        server.ask(b, Ask::Join(LAST_STAND));
        server.frame(0.1);
        assert_eq!((server.entered(b), server.body(body).coins), (Some((LAST_STAND, 1)), 950));
        server.ask(a, Ask::Leave);
        server.frame(0.1);
        assert_eq!((server.body(body).coins, server.balance(a)), (1000, Some(1000)));
        assert!(matches!(server.body(body).room, Room::Town(_)));
        assert!(server.table(LAST_STAND, 0).seats[0].vacant);
        // The next to come has the seat.
        let (c, _) = server.person(3, 1000);
        server.ask(c, Ask::Join(LAST_STAND));
        server.frame(0.1);
        assert_eq!(server.entered(c), Some((LAST_STAND, 0)));
        // With one: taken there and seated once it has been; given back
        // there, with the line saying why.
        let mut server = Server::new(true);
        let (a, body) = server.person(1, 1000);
        server.ask(a, Ask::Join(LAST_STAND));
        server.frame(0.1);
        assert!(
            matches!(
                server.asked[..],
                [Job::Charge { id: 1, amount: 50, why: "Last Stand entry", game: LAST_STAND, into: Some(0) }]
            ),
            "{} jobs",
            server.asked.len()
        );
        assert_eq!(server.entered(a), None, "seated once the fee is taken");
        assert!(server.world.get::<Paying>(body).is_some());
        server.charged(Charged {
            id: 1,
            left: Some(950),
            game: LAST_STAND,
            into: Some(0),
        });
        server.frame(0.1);
        assert_eq!(server.entered(a), Some((LAST_STAND, 0)));
        assert_eq!((server.body(body).coins, server.balance(a)), (950, Some(950)));
        assert!(server.world.get::<Paying>(body).is_none());
        server.ask(a, Ask::Leave);
        server.frame(0.1);
        assert!(
            matches!(server.asked.last(), Some(Job::Credit { id: 1, amount: 50, why: "Last Stand refund", game: Some(0) })),
            "{} jobs",
            server.asked.len()
        );
        // Turned away by the database: told so, and nobody sits there.
        let (d, body) = server.person(4, 50);
        server.ask(d, Ask::Join(LAST_STAND));
        server.frame(0.1);
        let into = match server.asked.last() {
            Some(Job::Charge { id: 4, into, .. }) => *into,
            _ => panic!("no charge for the fourth"),
        };
        server.charged(Charged {
            id: 4,
            left: None,
            game: LAST_STAND,
            into,
        });
        server.frame(0.1);
        assert!(matches!(server.heard(d).last(), Some(Tell::Refused(Refusal::Coins))));
        assert_eq!(server.entered(d), None);
        assert!(server.world.get::<Paying>(body).is_none());
    }

    #[test]
    fn two_people_tied_first_at_knock_off_share_first_s_and_second_s() {
        let mut server = Server::new(false);
        let (a, body_a) = server.person(1, 1000);
        let (b, body_b) = server.person(2, 1000);
        server.ask(a, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        server.ask(b, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        // Skipped: the computer all at once.
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        assert_eq!(server.computers(KNOCK_OFF, 0), 2);
        server.skip_to_the_game(a, KNOCK_OFF, 0);
        server.table_mut(KNOCK_OFF, 0).finish(vec![0, 0, 2, 3], vec![9, 9, 4, 0], None);
        server.frame(0.1);
        let results = server.table(KNOCK_OFF, 0).results.clone().expect("how it came out");
        assert_eq!(results.prizes, vec![95, 95, 0, 0]);
        assert_eq!((server.body(body_a).coins, server.body(body_b).coins), (950 + 95, 950 + 95));
    }

    #[test]
    fn last_stand_pays_everyone_ten_a_wave_cleared_up_to_four_hundred() {
        for (waves, each) in [(7, 70), (45, 400), (0, 0)] {
            let mut server = Server::new(false);
            let (a, body_a) = server.person(1, 1000);
            let (b, body_b) = server.person(2, 1000);
            server.ask(a, Ask::Join(LAST_STAND));
            server.frame(0.1);
            server.ask(b, Ask::Join(LAST_STAND));
            server.frame(0.1);
            server.ask(a, Ask::Skip);
            server.frame(0.1);
            server.skip_to_the_game(a, LAST_STAND, 0);
            assert_eq!(server.computers(LAST_STAND, 0), 0);
            server.table_mut(LAST_STAND, 0).finish(vec![0, 0], vec![310, 120], Some(waves));
            server.frame(0.1);
            let coins = (server.body(body_a).coins, server.body(body_b).coins);
            assert_eq!(coins, (950 + each, 950 + each), "{waves} waves");
        }
    }

    #[test]
    fn the_computer_s_ids_are_nobody_else_s() {
        // People's count up from 1. House Builder's bots come down from the
        // top, Bridge Wars' from half way and its builders' from a quarter,
        // each no further than its game's number reaches.
        let house_builder = u64::MAX - u64::from(u32::MAX) * 8 - 8;
        let bridge_wars = (u64::MAX / 2 - u64::from(u32::MAX) * 64 - 64, u64::MAX / 2);
        let builders = u64::MAX / 4 - (u64::from(u32::MAX) * 64 + 64) * 256 - 256;
        let mut seen = HashSet::new();
        for game in Minigame::TABLES {
            for id in [0, 1, 2, 63, 64, 1000, 16_384, 1 << 20, u32::MAX] {
                for seat in 0..4 {
                    let bot = bot_id(game, id, seat);
                    assert!(seen.insert(bot), "{game:?} table {id} seat {seat} has another's id");
                    assert!(bot > 1 << 40, "{bot}");
                    assert!(bot < builders && bot < house_builder, "{bot}");
                    assert!(!(bridge_wars.0..=bridge_wars.1).contains(&bot), "{bot}");
                }
            }
        }
    }

    #[test]
    fn a_fee_for_house_builder_seats_nobody_at_a_table_and_a_table_s_nobody_in_house_builder() {
        let mut server = Server::new(true);
        let (a, body_a) = server.person(1, 1000);
        let (b, body_b) = server.person(2, 1000);
        // House Builder's answer for someone whose seat at a table is held:
        // House Builder's to seat, and the table's seat stays held.
        server.ask(a, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        server.charged(Charged {
            id: 1,
            left: Some(900),
            game: Minigame::HouseBuilder,
            into: Some(0),
        });
        server.frame(0.1);
        assert_eq!(server.entered(a), None);
        assert!(server.heard(a).iter().any(|tell| matches!(tell, Tell::Joined { .. })));
        assert_eq!(server.body(body_a).room, Room::Game(0));
        assert!(server.table(KNOCK_OFF, 0).held(0));
        // The table's own answer for them, now in House Builder: no seat for
        // them, and the fee given back.
        server.charged(Charged {
            id: 1,
            left: Some(850),
            game: KNOCK_OFF,
            into: Some(0),
        });
        server.frame(0.1);
        assert_eq!(server.entered(a), None);
        assert!(matches!(server.asked.last(), Some(Job::Credit { id: 1, amount: 50, why: "Knock Off refund", .. })));
        // A table's answer: the table seats them, and House Builder nobody.
        server.ask(b, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        let into = match server.asked.last() {
            Some(Job::Charge { id: 2, game: KNOCK_OFF, into, .. }) => *into,
            _ => panic!("no charge for the second"),
        };
        server.charged(Charged {
            id: 2,
            left: Some(950),
            game: KNOCK_OFF,
            into,
        });
        server.frame(0.1);
        let id = into.expect("a table");
        assert_eq!(server.entered(b), Some((KNOCK_OFF, 0)));
        assert!(!server.heard(b).iter().any(|tell| matches!(tell, Tell::Joined { .. })));
        assert_eq!(server.body(body_b).room, Room::Table(KNOCK_OFF, id));
        assert!(!server.table(KNOCK_OFF, id).held(0), "the seat is theirs, no longer held");
    }

    /// A game over the moment the last of its people leaves it, each placed
    /// as they stood: as Knock Off's own may be ("your place kept and
    /// paid").
    struct OverAsTheyGo;

    impl Rules for OverAsTheyGo {
        fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
            let (at, turn) = knock_off::arrival(seat);
            (at, yaw_of(turn))
        }
        fn begin(&mut self, _ctx: &mut Ctx, _table: &mut Table) {}
        fn step(&mut self, _ctx: &mut Ctx, _table: &mut Table, _dt: f32) {}
        fn left(&mut self, _ctx: &mut Ctx, table: &mut Table, _seat: usize, _body: Option<Entity>) {
            if table.people() == 0 {
                table.finish(vec![1, 0, 2, 3], vec![0; 4], None);
            }
        }
        fn show(&mut self, _commands: &mut Commands, _table: &Table) {}
        fn close(&mut self, _commands: &mut Commands, _table: &Table) {}
    }

    #[test]
    fn whoever_leaves_a_game_that_is_over_as_they_go_is_paid_for_their_place() {
        let mut server = Server::new(false);
        let (a, body) = server.person(1, 1000);
        server.ask(a, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        server.skip_to_the_game(a, KNOCK_OFF, 0);
        server.world.resource_mut::<Tables>().games.insert((KNOCK_OFF, 0), Box::new(OverAsTheyGo));
        server.ask(a, Ask::Leave);
        server.frame(0.1);
        // Second, behind the computer: second's 60, in the town.
        assert_eq!((server.body(body).coins, server.balance(a)), (950 + 60, Some(950 + 60)));
        assert!(matches!(server.body(body).room, Room::Town(_)));
        server.until(1.0, |server| !server.world.resource::<Tables>().tables.contains_key(&(KNOCK_OFF, 0)));
    }

    #[test]
    fn a_table_left_empty_closes_only_once_whoever_left_last_has_been_sent_that_it_is_gone() {
        let mut server = Server::new(false);
        let (a, body) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        server.ask(a, Ask::Join(BLEND_IN));
        server.frame(0.1);
        server.ask(b, Ask::Join(BLEND_IN));
        server.frame(0.1);
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        server.skip_to_the_game(a, BLEND_IN, 0);
        server.table_mut(BLEND_IN, 0).finish(vec![0, 1], vec![1, 0], None);
        server.frame(0.1);
        let shown = server.table(BLEND_IN, 0).shown;
        // Both play again, one after the other, from the results: the first
        // into a new table at once, and the old one goes on for the second.
        server.ask(a, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert_eq!(server.body(body).room, Room::Table(BLEND_IN, 1));
        assert!(!server.table(BLEND_IN, 0).closing);
        server.ask(b, Ask::Join(BLEND_IN));
        server.frame(0.1);
        // Nobody at the old one now: it stands through a send of its room,
        // the one that tells the second it is gone from their sight, and
        // only after that is it despawned.
        let tick = server.world.resource::<ServerTick>().get();
        let table = server.table(BLEND_IN, 0);
        assert_eq!((table.people(), table.closing, table.gone_out), (0, false, Some((tick, false))));
        assert!(server.world.get_entity(shown).is_ok());
        server.frame(0.1);
        assert!(!server.table(BLEND_IN, 0).closing);
        assert!(server.world.get_entity(shown).is_ok());
        server.frame(0.1);
        assert!(server.table(BLEND_IN, 0).closing);
        assert!(server.world.get_entity(shown).is_err());
        server.frame(0.1);
        assert!(!server.world.resource::<Tables>().tables.contains_key(&(BLEND_IN, 0)));
        // However many frames go by without a send, it waits: the tick has
        // to be seen to move, and the close is the frame after; with nobody
        // gone, nothing holds it; and whoever goes meanwhile holds it again.
        let mut table = Table::new(BLEND_IN, 9, Entity::PLACEHOLDER, 1).expect("a table");
        assert!(table.sent_since_anyone_went(5));
        table.went_out = true;
        assert!(!table.sent_since_anyone_went(5) && !table.sent_since_anyone_went(5) && !table.sent_since_anyone_went(5));
        assert!(!table.sent_since_anyone_went(6));
        assert!(table.sent_since_anyone_went(6) && table.sent_since_anyone_went(6));
        table.went_out = true;
        assert!(!table.sent_since_anyone_went(6) && !table.sent_since_anyone_went(7));
        table.went_out = true;
        assert!(!table.sent_since_anyone_went(7) && !table.sent_since_anyone_went(8));
        assert!(table.sent_since_anyone_went(8));
    }

    #[test]
    fn a_table_at_its_results_closes_only_once_whoever_just_left_it_has_been_sent_that_it_is_gone() {
        let mut server = Server::new(false);
        let (a, _) = server.person(1, 1000);
        let (b, body_b) = server.person(2, 1000);
        server.ask(a, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        server.ask(b, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        server.skip_to_the_game(a, KNOCK_OFF, 0);
        server.table_mut(KNOCK_OFF, 0).finish(vec![0, 1, 2, 3], vec![0; 4], None);
        server.frame(0.1);
        let shown = server.table(KNOCK_OFF, 0).shown;
        let computers: Vec<Entity> = server.table(KNOCK_OFF, 0).seats.iter().filter(|it| it.computer).map(|it| it.body).collect();
        // The results' last moment, and the second goes to the town in it,
        // the first staying: nothing of the table goes in the send that
        // tells the second it is gone from their sight...
        server.table_mut(KNOCK_OFF, 0).stage = Stage::Over { left: 0.0 };
        server.ask(b, Ask::Leave);
        server.frame(0.1);
        assert!(matches!(server.body(body_b).room, Room::Town(_)));
        assert!(!server.table(KNOCK_OFF, 0).closing);
        assert!(server.world.get_entity(shown).is_ok());
        assert!(computers.iter().all(|&body| server.world.get_entity(body).is_ok()));
        server.frame(0.1);
        assert!(!server.table(KNOCK_OFF, 0).closing);
        // ...but after it, and the first goes back to the town as ever.
        server.frame(0.1);
        assert!(server.table(KNOCK_OFF, 0).closing);
        assert!(server.world.get_entity(shown).is_err());
        assert!(computers.iter().all(|&body| server.world.get_entity(body).is_err()));
        server.frame(0.1);
        assert!(matches!(server.heard(a).last(), Some(Tell::Arrive { venue: Venue::Town, .. })));
        assert!(!server.world.resource::<Tables>().tables.contains_key(&(KNOCK_OFF, 0)));
    }

    #[test]
    fn a_seat_waits_while_the_game_is_played_and_whoever_s_device_is_back_is_put_back_in_it() {
        let mut server = Server::new(false);
        let (a, body_a) = server.person(1, 1000);
        let (b, body_b) = server.person(2, 1000);
        server.ask(a, Ask::Join(BLEND_IN));
        server.frame(0.1);
        server.ask(b, Ask::Join(BLEND_IN));
        server.frame(0.1);
        let room = Room::Table(BLEND_IN, 0);
        // Before the game a seat does not wait for a device that has gone.
        assert!(!server.world.resource::<Tables>().keeps(room, body_a));
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        server.skip_to_the_game(b, BLEND_IN, 0);
        // A game that tells a device back nothing of its own (Blend In's
        // tells it what it wears).
        server.world.resource_mut::<Tables>().games.insert((BLEND_IN, 0), Box::new(OverAsTheyGo));
        assert!(server.world.resource::<Tables>().keeps(room, body_a));
        // Its device goes (as `players::gone` has it): the body stands in its
        // seat, away; what it is told meanwhile it misses.
        server.world.get_mut::<Body>(body_a).expect("a body").client = None;
        server.frame(0.1);
        assert!(server.table(BLEND_IN, 0).seats[0].away);
        assert!(!server.table(BLEND_IN, 0).watched_but(1));
        server.act(BLEND_IN, 0, |ctx, table| ctx.tell(table, 0, Tell::Balance(1)));
        // Back on a device that still had the game, with something missed:
        // begun afresh on it, someone else there or not.
        let back = server.world.spawn(Plays(body_a)).id();
        server.world.get_mut::<Body>(body_a).expect("a body").client = Some(back);
        let returned = server.world.resource_mut::<Tables>().returns(room, body_a, true);
        assert_eq!(returned, Return::Back { seat: 0 });
        server.frame(0.1);
        assert!(matches!(
            server.heard(back)[..],
            [Tell::Entered { game: BLEND_IN, seat: 0 }, Tell::Arrive { venue: Venue::BlendIn, seat: 0, .. }, Tell::PutBack { .. }]
        ));
        assert!(!server.table(BLEND_IN, 0).seats[0].away);
        // An app only put away, with nothing missed, carries on.
        assert_eq!(server.world.resource_mut::<Tables>().returns(room, body_a, true), Return::Resume);
        // One closed and opened again is put back while someone else of the
        // people at it is connected, and gives the seat up when nobody is.
        assert_eq!(server.world.resource_mut::<Tables>().returns(room, body_a, false), Return::Back { seat: 0 });
        server.world.get_mut::<Body>(body_b).expect("a body").client = None;
        server.frame(0.1);
        assert_eq!(server.world.resource_mut::<Tables>().returns(room, body_a, false), Return::GiveUp);
    }

    #[test]
    fn whoever_a_game_has_pass_for_someone_else_is_themselves_again_on_the_way_out() {
        let mut server = Server::new(false);
        let (a, body_a) = server.person(1, 1000);
        let (b, body_b) = server.person(2, 1000);
        server.ask(a, Ask::Join(BLEND_IN));
        server.frame(0.1);
        server.ask(b, Ask::Join(BLEND_IN));
        server.frame(0.1);
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        server.skip_to_the_game(a, BLEND_IN, 0);
        // A game that plays on with one person left (Blend In's own ends a
        // game of two as one leaves, paid by points so far).
        server.world.resource_mut::<Tables>().games.insert((BLEND_IN, 0), Box::new(OverAsTheyGo));
        let mochi = |id: u64| Member {
            id,
            name: "Mochi".into(),
            look: "girl_whiteblue".into(),
            ai: false,
        };
        server.act(BLEND_IN, 0, move |ctx, table| {
            ctx.disguise(table, 0, mochi(77));
            ctx.disguise(table, 1, mochi(78));
        });
        assert_eq!(server.world.get::<Member>(body_a).map(|member| member.id), Some(77));
        assert!(server.world.get::<Disguised>(body_a).is_some());
        // The seat is still the account's, as the room is told.
        assert_eq!(server.table(BLEND_IN, 0).seats[0].member.id, 1);
        // Leaving mid-game: their own again, in the town, the seat theirs
        // and empty.
        server.ask(a, Ask::Leave);
        server.frame(0.1);
        assert_eq!(server.world.get::<Member>(body_a).map(|member| member.id), Some(1));
        assert!(server.world.get::<Disguised>(body_a).is_none());
        assert!(matches!(server.body(body_a).room, Room::Town(_)));
        let seat = &server.table(BLEND_IN, 0).seats[0];
        assert!(seat.gone && !seat.vacant && seat.member.id == 1);
        // The game over: everyone is their own again.
        server.table_mut(BLEND_IN, 0).finish(vec![1, 0], vec![0, 1], None);
        server.frame(0.1);
        assert_eq!(server.world.get::<Member>(body_b).map(|member| member.id), Some(2));
        assert!(server.world.get::<Disguised>(body_b).is_none());
        // Who left still came where they came, and is paid for it: of two,
        // second's nothing; the one who stayed, first's 100.
        assert_eq!((server.body(body_a).coins, server.body(body_b).coins), (950, 1050));
    }

    #[test]
    fn a_body_the_game_moves_is_sent_only_where_it_has_gone_and_as_still_once_it_stops() {
        let mut server = Server::new(false);
        let (a, _) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        server.ask(a, Ask::Join(BLEND_IN));
        server.frame(0.1);
        server.ask(b, Ask::Join(BLEND_IN));
        server.frame(0.1);
        let crowd = Member {
            id: 4242,
            name: "Dubu".into(),
            look: "girl".into(),
            ai: false,
        };
        server.act(BLEND_IN, 0, move |ctx, table| {
            ctx.spawn_body(table, crowd.clone(), Vec3::new(10.0, 0.0, 5.0), 0.0);
        });
        let mut bodies = server.world.query::<(Entity, &Member, &InRoom, &Venue)>();
        let (body, room, venue) = bodies
            .iter(&server.world)
            .find(|(_, member, ..)| member.id == 4242)
            .map(|(body, _, room, venue)| (body, *room, *venue))
            .expect("the crowd's body");
        assert_eq!((room, venue), (InRoom(Room::Table(BLEND_IN, 0)), Venue::BlendIn));
        // A step: sent there, on the move.
        server.act(BLEND_IN, 0, move |ctx, _| ctx.move_body(body, Vec3::new(11.0, 0.0, 5.0), 0.0));
        let moved = *server.world.get::<Spot>(body).expect("a spot");
        assert!((moved.translation().x - 11.0).abs() < 0.01 && !moved.still);
        // Where it is already: sent once more as standing still, and then
        // nothing more.
        server.act(BLEND_IN, 0, move |ctx, _| ctx.move_body(body, Vec3::new(11.0, 0.0, 5.0), 0.0));
        assert!(server.world.get::<Spot>(body).expect("a spot").still);
        let before = server.world.entity(body).get_ref::<Spot>().expect("a spot").last_changed();
        server.act(BLEND_IN, 0, move |ctx, _| ctx.move_body(body, Vec3::new(11.0, 0.0, 5.0), 0.0));
        let after = server.world.entity(body).get_ref::<Spot>().expect("a spot").last_changed();
        assert_eq!(before, after, "written again for nothing");
    }

    /// Blend In's first table, with the database: three people seated, the
    /// countdown begun, the second gone from it as it counts, a fourth
    /// asking in then, into the second's seat, held while their fee is
    /// taken, and the countdown over before it has been. The third's device
    /// and body, and the fourth's.
    fn counted_down_with_a_seat_held() -> (Server, (Entity, Entity), (Entity, Entity)) {
        let mut server = Server::new(true);
        let (a, _) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        let third = server.person(3, 1000);
        let fourth = server.person(4, 1000);
        for (client, id) in [(a, 1), (b, 2), (third.0, 3)] {
            server.ask(client, Ask::Join(BLEND_IN));
            server.frame(0.1);
            server.charged(Charged {
                id,
                left: Some(950),
                game: BLEND_IN,
                into: Some(0),
            });
            server.frame(0.1);
        }
        assert_eq!(server.entered(third.0), Some((BLEND_IN, 2)));
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        assert!(matches!(server.table(BLEND_IN, 0).stage, Stage::Countdown { .. }));
        server.ask(b, Ask::Leave);
        server.frame(0.1);
        server.ask(fourth.0, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert!(server.table(BLEND_IN, 0).held(1));
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        (server, third, fourth)
    }

    #[test]
    fn nobody_is_left_out_as_a_game_without_the_computer_begins_while_a_fee_is_being_taken() {
        // The fee taken a moment after the countdown is over: the game waits
        // that moment, and the fourth plays, in the seat they were given.
        let (mut server, (_, body_c), (d, body_d)) = counted_down_with_a_seat_held();
        server.frame(0.5);
        assert!(matches!(server.table(BLEND_IN, 0).stage, Stage::Countdown { .. }));
        assert_eq!(server.table(BLEND_IN, 0).board().left, 0);
        server.charged(Charged {
            id: 4,
            left: Some(950),
            game: BLEND_IN,
            into: Some(0),
        });
        server.frame(0.1);
        assert_eq!(server.entered(d), Some((BLEND_IN, 1)));
        let table = server.table(BLEND_IN, 0);
        assert!(table.playing());
        assert_eq!((table.size, table.seats.len(), table.people()), (3, 3, 3));
        assert_eq!((table.seats[1].body, table.seats[2].body), (body_d, body_c));
        // Taken too late: the game begins without them, everyone seated
        // still seated (the third moved up into the second's seat, and
        // stood afresh in it), and they sit down at a table of their own
        // as their fee is taken.
        let (mut server, (c, body_c), (d, body_d)) = counted_down_with_a_seat_held();
        let waited = server.until(5.0, |server| server.table(BLEND_IN, 0).playing());
        assert!((1.7..=2.3).contains(&waited), "{waited}");
        let table = server.table(BLEND_IN, 0);
        assert_eq!((table.size, table.seats.len(), table.people()), (2, 2, 2));
        assert!(table.seats.iter().all(Seat::here));
        assert_eq!(table.seats[1].body, body_c);
        assert_eq!(server.arrived(c), Some((Venue::BlendIn, 1)));
        assert_eq!(server.body(body_c).room, Room::Table(BLEND_IN, 0));
        server.charged(Charged {
            id: 4,
            left: Some(950),
            game: BLEND_IN,
            into: Some(0),
        });
        server.frame(0.1);
        assert_eq!(server.body(body_d).room, Room::Table(BLEND_IN, 1));
        assert_eq!(server.entered(d), Some((BLEND_IN, 0)));
    }

    /// What a game was told, for the tests that look at it.
    #[derive(Clone, Debug, PartialEq)]
    enum Heard {
        /// Who left, by seat, and their body, if it was still about.
        Left(usize, Option<Entity>),
        /// The bodies of the people still at the table as it closed.
        Closed(Vec<Entity>),
    }

    /// A game that does nothing but note what it is told.
    struct Notes(Arc<Mutex<Vec<Heard>>>);

    impl Rules for Notes {
        fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
            let (at, turn) = knock_off::arrival(seat);
            (at, yaw_of(turn))
        }
        fn begin(&mut self, _ctx: &mut Ctx, _table: &mut Table) {}
        fn step(&mut self, _ctx: &mut Ctx, _table: &mut Table, _dt: f32) {}
        fn left(&mut self, _ctx: &mut Ctx, _table: &mut Table, seat: usize, body: Option<Entity>) {
            self.0.lock().expect("the notes").push(Heard::Left(seat, body));
        }
        fn show(&mut self, _commands: &mut Commands, _table: &Table) {}
        fn close(&mut self, _commands: &mut Commands, table: &Table) {
            let here = table.seats.iter().filter(|it| it.here()).map(|it| it.body).collect();
            self.0.lock().expect("the notes").push(Heard::Closed(here));
        }
    }

    #[test]
    fn a_game_hears_whose_body_left_it_and_closes_with_the_bodies_still_at_it_in_hand() {
        let mut server = Server::new(false);
        let (a, body_a) = server.person(1, 1000);
        let (b, body_b) = server.person(2, 1000);
        let (c, body_c) = server.person(3, 1000);
        let (d, body_d) = server.person(4, 1000);
        for client in [a, b, c, d] {
            server.ask(client, Ask::Join(KNOCK_OFF));
            server.frame(0.1);
        }
        server.until(30.0, |server| server.table(KNOCK_OFF, 0).playing());
        assert_eq!(server.computers(KNOCK_OFF, 0), 0);
        let notes = Arc::new(Mutex::new(Vec::new()));
        server.world.resource_mut::<Tables>().games.insert((KNOCK_OFF, 0), Box::new(Notes(notes.clone())));
        // The first goes to the town: their body, for the game to take off
        // it whatever it put on it.
        server.ask(a, Ask::Leave);
        server.frame(0.1);
        // The second's device goes and the seat is given up, the body with
        // it (as `players::gone` has it): no body to take anything off.
        server.world.resource_mut::<Tables>().walk_out(Room::Table(KNOCK_OFF, 0), body_b);
        server.world.despawn(body_b);
        server.frame(0.1);
        // Over: the third goes as the results are up, their body too.
        server.table_mut(KNOCK_OFF, 0).finish(vec![2, 1, 0, 3], vec![0; 4], None);
        server.frame(0.1);
        server.ask(c, Ask::Leave);
        server.frame(0.1);
        // The results' while over: the fourth's body in hand as it closes.
        server.table_mut(KNOCK_OFF, 0).stage = Stage::Over { left: 0.0 };
        server.until(1.0, |server| server.table(KNOCK_OFF, 0).closing);
        let heard = notes.lock().expect("the notes").clone();
        let all = [
            Heard::Left(0, Some(body_a)),
            Heard::Left(1, None),
            Heard::Left(2, Some(body_c)),
            Heard::Closed(vec![body_d]),
        ];
        assert_eq!(heard, all);
    }

    #[test]
    fn whoever_is_out_of_a_game_may_go_from_it_into_another_and_nobody_still_in_it_may() {
        let mut server = Server::new(false);
        let (a, body_a) = server.person(1, 1000);
        let (b, body_b) = server.person(2, 1000);
        server.ask(a, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        server.ask(b, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        server.skip_to_the_game(a, KNOCK_OFF, 0);
        let notes = Arc::new(Mutex::new(Vec::new()));
        server.world.resource_mut::<Tables>().games.insert((KNOCK_OFF, 0), Box::new(Notes(notes.clone())));
        let room = Room::Table(KNOCK_OFF, 0);
        // In the game: into nothing else.
        assert!(!server.world.resource::<Tables>().done_with(room, body_a));
        server.ask(a, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        assert!(matches!(server.heard(a).last(), Some(Tell::Refused(Refusal::Busy))));
        assert_eq!(server.body(body_a).room, room);
        // Out of it, in the sea: into the next game, at a new table, the
        // seat they had theirs still, empty, for the results, and the game
        // they were in told they have gone.
        server.table_mut(KNOCK_OFF, 0).seats[0].out = true;
        assert!(server.world.resource::<Tables>().done_with(room, body_a));
        assert!(!server.world.resource::<Tables>().done_with(room, body_b));
        server.ask(a, Ask::Join(KNOCK_OFF));
        server.frame(0.1);
        assert_eq!((server.body(body_a).room, server.entered(a)), (Room::Table(KNOCK_OFF, 1), Some((KNOCK_OFF, 0))));
        let seat = &server.table(KNOCK_OFF, 0).seats[0];
        assert!(seat.out && seat.gone && !seat.vacant && seat.member.id == 1);
        assert_eq!(*notes.lock().expect("the notes"), [Heard::Left(0, Some(body_a))]);
        // And along with the group they are in, whose leader is in the town.
        let (leader, _) = server.person(3, 1000);
        server.world.resource_mut::<Social>().band(&[3, 2]);
        server.table_mut(KNOCK_OFF, 0).seats[1].out = true;
        server.ask(leader, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert_eq!((server.body(body_b).room, server.entered(b)), (Room::Table(BLEND_IN, 0), Some((BLEND_IN, 1))));
    }

    #[test]
    fn whoever_goes_from_a_game_they_are_out_of_into_house_builder_goes_as_themselves() {
        let mut server = Server::new(true);
        let (a, body_a) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        for (client, id) in [(a, 1), (b, 2)] {
            server.ask(client, Ask::Join(BLEND_IN));
            server.frame(0.1);
            server.charged(Charged {
                id,
                left: Some(950),
                game: BLEND_IN,
                into: Some(0),
            });
            server.frame(0.1);
        }
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        server.skip_to_the_game(a, BLEND_IN, 0);
        let mochi = Member {
            id: 77,
            name: "Mochi".into(),
            look: "girl".into(),
            ai: false,
        };
        server.act(BLEND_IN, 0, move |ctx, table| ctx.disguise(table, 0, mochi.clone()));
        server.table_mut(BLEND_IN, 0).seats[0].out = true;
        // Out, and passing for someone else: House Builder's fee taken for
        // them (as its own join asks it, free to as they are), and seated
        // there as their account, wearing it again.
        server.charged(Charged {
            id: 1,
            left: Some(850),
            game: Minigame::HouseBuilder,
            into: Some(0),
        });
        server.frame(0.1);
        let room = server.body(body_a).room;
        assert!(matches!(room, Room::Game(_)), "{room:?}");
        assert_eq!(server.world.get::<Member>(body_a).map(|member| member.id), Some(1));
        assert!(server.world.get::<Disguised>(body_a).is_none());
        let seated: Vec<u64> = server.world.resource::<Games>().members(room).expect("the game").iter().map(|member| member.id).collect();
        assert_eq!(seated, [1]);
        let seat = &server.table(BLEND_IN, 0).seats[0];
        assert!(seat.out && seat.gone && seat.member.id == 1);
    }

    #[test]
    fn someone_waiting_alone_for_blend_in_sits_with_the_next_to_come_whatever_table_a_group_took() {
        let mut server = Server::new(false);
        let (alone, _) = server.person(1, 1000);
        server.ask(alone, Ask::Join(BLEND_IN));
        server.frame(0.1);
        // A group of four, which does not fit beside them: a table of its
        // own, full, counting down at once.
        let (leader, body_leader) = server.person(2, 1000);
        for id in 3..=5 {
            server.person(id, 1000);
        }
        server.world.resource_mut::<Social>().band(&[2, 3, 4, 5]);
        server.ask(leader, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert_eq!(server.body(body_leader).room, Room::Table(BLEND_IN, 1));
        assert!(matches!(server.table(BLEND_IN, 1).stage, Stage::Countdown { .. }));
        // The next to come sits with the one who has been waiting, not at a
        // newer table of their own, and the two of them play.
        let (next, body_next) = server.person(6, 1000);
        server.ask(next, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert_eq!((server.body(body_next).room, server.entered(next)), (Room::Table(BLEND_IN, 0), Some((BLEND_IN, 1))));
        server.until(30.0, |server| server.table(BLEND_IN, 0).playing());
        assert_eq!(server.table(BLEND_IN, 0).people(), 2);
    }

    #[test]
    fn whoever_comes_first_earns_the_winner_s_share_toward_their_level_but_nobody_on_a_draw() {
        let won = XP_PLAYED + XP_WON;
        for (places, earned) in [(vec![0, 1], (won, XP_PLAYED)), (vec![1, 0], (XP_PLAYED, won)), (vec![0, 0], (XP_PLAYED, XP_PLAYED))] {
            let mut server = Server::new(false);
            let (a, _) = server.person(1, 1000);
            let (b, _) = server.person(2, 1000);
            server.ask(a, Ask::Join(BLEND_IN));
            server.frame(0.1);
            server.ask(b, Ask::Join(BLEND_IN));
            server.frame(0.1);
            server.ask(a, Ask::Skip);
            server.frame(0.1);
            server.skip_to_the_game(a, BLEND_IN, 0);
            server.table_mut(BLEND_IN, 0).finish(places.clone(), vec![0, 0], None);
            server.frame(0.1);
            assert_eq!((server.xp(1), server.xp(2)), earned, "{places:?}");
        }
    }

    #[test]
    fn blend_in_waits_its_whole_wait_from_the_second_person_and_at_least_eight_seconds_from_each_after() {
        let mut server = Server::new(false);
        let (a, _) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        let (c, _) = server.person(3, 1000);
        server.ask(a, Ask::Join(BLEND_IN));
        server.frame(0.1);
        server.ask(b, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert_eq!(server.table(BLEND_IN, 0).board().left, 15);
        for _ in 0..100 {
            server.frame(0.1);
        }
        assert_eq!(server.table(BLEND_IN, 0).board().left, 5);
        // A third with five seconds to go: eight from them, not fifteen.
        server.ask(c, Ask::Join(BLEND_IN));
        server.frame(0.1);
        assert_eq!(server.table(BLEND_IN, 0).board().left, 8);
        let waited = server.until(20.0, |server| matches!(server.table(BLEND_IN, 0).stage, Stage::Countdown { .. }));
        assert!((7.5..=8.5).contains(&waited), "{waited}");
        // Last Stand waits its whole wait again from each who comes.
        let mut server = Server::new(false);
        let (a, _) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        server.ask(a, Ask::Join(LAST_STAND));
        server.frame(0.1);
        assert_eq!(server.table(LAST_STAND, 0).board().left, 15);
        for _ in 0..100 {
            server.frame(0.1);
        }
        server.ask(b, Ask::Join(LAST_STAND));
        server.frame(0.1);
        assert_eq!(server.table(LAST_STAND, 0).board().left, 15);
    }

    #[test]
    fn a_table_s_room_is_hushed_while_its_game_has_everyone_say_nothing() {
        let mut server = Server::new(false);
        let (a, _) = server.person(1, 1000);
        let (b, _) = server.person(2, 1000);
        server.ask(a, Ask::Join(BLEND_IN));
        server.frame(0.1);
        server.ask(b, Ask::Join(BLEND_IN));
        server.frame(0.1);
        let room = Room::Table(BLEND_IN, 0);
        let hushed = |server: &Server, room: Room| server.world.resource::<Tables>().hushed(room);
        assert!(!hushed(&server, room), "waiting");
        server.ask(a, Ask::Skip);
        server.frame(0.1);
        server.skip_to_the_game(a, BLEND_IN, 0);
        assert!(!hushed(&server, room), "choosing");
        // Everyone ready: Blend In's hide, and nothing said is passed on.
        for client in [a, b] {
            server.ask(client, Ask::BlendIn(roundtown_net::blend_in::Ask::Ready));
        }
        server.frame(0.1);
        server.frame(0.1);
        assert!(hushed(&server, room), "the hide");
        // Nowhere else: the town, another table, a table that is not there.
        assert!(!hushed(&server, Room::Town(0)));
        assert!(!hushed(&server, Room::Table(BLEND_IN, 1)));
        assert!(!hushed(&server, Room::Table(KNOCK_OFF, 0)));
    }
}
