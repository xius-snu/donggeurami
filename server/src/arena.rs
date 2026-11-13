//! Bridge Wars, on the server (since 2026-11-08; Hajun: "right now the
//! bridge battle minigame doesnt allow queing with other players i think
//! even tho we join the game at the same time. fix that"): every game, from
//! the waiting room to the end, on the server's clock, so that people who
//! choose it together play it together.
//!
//! Devices show what it says — the stage, the time left, everyone's health,
//! each team's respawn anchor and who is out ([`ArenaRound`]), who is
//! playing ([`Seats`]) and every plank of every bridge ([`Plank`]) — send
//! what their player does ([`Ask::Arena`] to join, [`Ask::Punch`] and
//! [`Ask::HitAnchor`] for a punch that landed, [`Ask::Rung`] for a plank
//! laid, [`Ask::Respawn`] for a fall into the void, [`Ask::Leave`]), and are
//! moved by it: into the waiting room, to their islands, back where they
//! arrive when they stand again, and to the town at the end.
//!
//! A game is a room of its own ([`Room::Arena`]). Whoever wants to play goes
//! into the game filling now. After [`BOT_WAIT`] seconds with nobody joining
//! the computer takes every seat left at once, as the game does offline, and
//! the room counts down and goes. The computer's players are moved here
//! ([`Mind`]): the server has no map, so each keeps to the disc of its
//! island ([`ARENA_ISLAND_RADIUS`]), runs at whoever of another team comes
//! within its sight there, punches once in reach, runs away when nearly
//! beaten, and wanders when alone; knocked off its island it falls, and is
//! dead at the bottom. It never crosses a bridge.
//!
//! Health is the server's: a punch that lands ([`Ask::Punch`]) takes
//! [`DAMAGE`] off whoever it landed on, if they are not on the puncher's
//! team, and knocks them (`Tell::Knocked` to a person's device, which knocks
//! its own body; a bot is knocked here); at none a body is dead for
//! [`RESPAWN_SECS`] and then stands again where its seat arrives, so long as
//! its team's anchor stood when it died, and is out for good otherwise
//! (Hajun: "you only indefinitely respawn when you die, and at that point
//! when u died, the respawn anchor is still alive"). An anchor takes the
//! same damage from a punch of another team's ([`Ask::HitAnchor`]), twenty
//! of them ([`ANCHOR_HEALTH`]). The last team with anyone in play wins, or
//! nobody when the time runs out; the result is up for [`RESULTS_SECS`] and
//! everyone still there goes back to the town.
//!
//! **Since 2026-11-11 the generators, what they make, what everyone holds
//! and the shop are the server's too** (Hajun: "these generators/items in
//! map need to be server synced so other players see the same number of
//! items on map at the same times and other player taking the item from
//! the generators make it so you cant take it too"). Each game has a
//! [`Mint`], the generators' rules, which both sides share
//! (`roundtown_net::bridge_wars`); everything it makes is an entity the
//! room is sent ([`Loot`]), and whoever of the people in the game comes
//! within the magnet's reach of one, where they last said they stood, has
//! it: it is gone for everyone that tick, and theirs in the room's
//! [`Purses`]. A thing drops onto its pad from high over it as it is
//! made, and is within reach only once it has come down to whoever is
//! there, half a metre over their head ([`REACH_OVER`], since 2026-11-12;
//! for an evening before, not until it had landed and lain a moment), so
//! that someone standing on the pad sees it come, and someone who jumps
//! for it has it sooner, in the air. The computer's players pick up
//! nothing. [`Ask::Buy`] takes
//! a ware's price out of the asker's purse and tells them it is theirs
//! ([`Tell::Bought`]). And the game keeps Hajun's timetable ([`TIMETABLE`]):
//! the generators' tiers are the mint's own by the time; at ten minutes
//! every anchor left breaks; at fifteen, sudden death, the zombies come
//! ([`ZOMBIES`]), seated past the players' eight seats and moved as the
//! computer's other players are, each on the ground it keeps to
//! ([`zombie_home`]), going for anyone, and back up a while after it is
//! put down; and at twenty five it is a draw ([`PLAY_SECS`]).
//!
//! **Since 2026-11-12 the two shops' wares are the server's as well**
//! (Hajun's `bridgewarsgameevent.txt`). What everyone has and what each
//! team has bought is the game's [`Outfits`], the shops' rules, which
//! both sides share, and its room is sent them as [`Kits`] and
//! [`Upgraded`]: a punch takes more with a sword and less through a
//! shield ([`Game::wound`]); a fireball thrown ([`Ask::Use`]) is an entity
//! the room is sent ([`Fireball`]) that the server flies and bursts
//! ([`fireballs`]); a potion drunk is the kit's until it wears off; a
//! team's generator is as fast as the team has raised it; its heal pool
//! heals it on its island, and its traps blind whoever of another team
//! steps onto it ([`Outfits::tick`]). The computer's players buy nothing,
//! and do not go for anyone unseen.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::bots::{Seen, Skill, Squad, Stick, View};
use roundtown_net::bridge_wars::*;
use roundtown_net::nav::Nav;
use roundtown_net::{Blasted, Dynamite};
use roundtown_net::social::{Minigame, XP_PLAYED, XP_WON};
use roundtown_net::{
    ARENA_ANCHORS, ARENA_ISLAND_RADIUS, ARENA_ISLANDS, ARENA_SEATS, ARENA_TEAMS, ArenaRound,
    ArenaStage, Ask, Fireball, Kit, Kits, Loot, Member, Plank, Purse, Purses, Refusal, Seats, Spot,
    Tell, Upgraded, Upgrades, Use, Utility, Venue, Ware, arena_island, arena_team, arrival,
};

use crate::Config;
use crate::players::{Body, Everyone, Plays, Who, move_to, place, tell};
use crate::rooms::{InRoom, Room, Towns};
use crate::social::Social;

/// How long a game goes on with nobody connected to it, in seconds, before
/// it is given up.
const ABANDONED: f32 = 60.0;
/// The most rungs one seat can lay: far more than a game's worth, so that a
/// device gone wrong cannot fill the server's memory.
const MOST_RUNGS: usize = 600;
/// How far out a rung can be laid, in metres: the edge of the map and a
/// little.
const FURTHEST: f32 = 220.0;
/// How far apart a puncher and whoever they hit can be where each last said
/// they stood, in metres, for the punch to count: the reach, 2 m from the
/// puncher's middle, and the way each may have gone since.
const PUNCH_SLACK: f32 = 6.5;
/// How wide a body is, across the ground, in metres: the game's
/// `PLAYER_RADIUS`.
const BODY_RADIUS: f32 = 0.45;
/// Where a body stands on an island, in metres up: the islands' tops.
const ISLAND_TOP: f32 = 0.25;
/// How fast a body knocked off its island falls, in metres a second a
/// second: the game's `FALL_GRAVITY`.
const GRAVITY: f32 = 33.0;
/// How often a bot with nobody to go for picks somewhere new to walk, in
/// seconds, and how fast it walks there, in metres a second.
const THINK_EVERY: f32 = 3.0;
const STROLL: f32 = 7.0;
/// How near a bot stands off whoever it goes for, as a share of the reach.
const PRESS_IN: f32 = 0.7;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Arenas>()
        .add_systems(
            PreUpdate,
            hear.after(ServerSystems::Receive).run_if(in_state(ServerState::Running)),
        )
        .add_systems(Update, run.run_if(in_state(ServerState::Running)));
}

/// Every game under way, by id, and the one filling now.
#[derive(Resource, Default)]
pub(crate) struct Arenas {
    next: u32,
    games: HashMap<u32, Game>,
    filling: Option<u32>,
}

/// One game of Bridge Wars.
struct Game {
    id: u32,
    /// The entity its room is sent, with its [`ArenaRound`] and [`Seats`].
    shown: Entity,
    /// Everyone's body, in seat order.
    seats: Vec<Entity>,
    members: Vec<Member>,
    /// Which seats the computer is playing, and its mind for each.
    bot: Vec<bool>,
    minds: Vec<Mind>,
    /// Who has left it.
    gone: Vec<bool>,
    /// The seats nobody is sitting in: between two that are taken, as the
    /// seats are taken out of order ([`SEAT_ORDER`]), and whoever's they
    /// were who left before the fight began ([`Game::leave`]). Such a
    /// seat is gone and out, with no body, until someone takes it; the
    /// computer takes every one left as the wait ends, and again as the
    /// countdown does, so that the fight begins with all eight taken.
    vacant: Vec<bool>,
    stage: ArenaStage,
    /// Seconds left of this stage.
    left: f32,
    /// Seconds since the last to join did, in the waiting room.
    since_join: f32,
    /// Seconds with nobody connected to it.
    unwatched: f32,
    /// Everyone's health, how long since each was last hit, how long each
    /// is still reeling from the last punch that took ([`HURT_TIME`]), in
    /// which no other counts, how long each of the dead has left before
    /// standing again, and who is out for good.
    health: Vec<f32>,
    since_hit: Vec<f32>,
    hurt: Vec<f32>,
    dead_for: Vec<Option<f32>>,
    out: Vec<bool>,
    anchors: [f32; ARENA_TEAMS],
    anchor_hurt: [f32; ARENA_TEAMS],
    /// Every rung laid, and how many by each seat.
    planks: Vec<Entity>,
    rungs: Vec<usize>,
    /// How far into the fight it is, in seconds: what the timetable goes
    /// by ([`TIMETABLE`]), and the generators' tiers.
    elapsed: f32,
    /// The generators, and everything they have made that lies there yet,
    /// each by its number as the entity the room is sent ([`Loot`]).
    mint: Mint,
    loot: HashMap<u32, Entity>,
    /// What everyone holds, by seat.
    purses: Vec<Purse>,
    /// What everyone has and what each team has bought, by the shops'
    /// rules.
    outfits: Outfits,
    /// Every fireball in the air: its number, how far it has flown, and
    /// who threw it, by seat; and the entity its room is sent for each, by
    /// its number.
    balls: Vec<(u32, Flight, usize)>,
    next_ball: u32,
    shown_balls: HashMap<u32, Entity>,
    /// Every stick of dynamite thrown and not yet blown ([`Lobbed`]), and
    /// the entity its room is sent for each, by its number; and how many
    /// have blown, and how many planks they took with them, for reading a
    /// game back by.
    sticks: Vec<Lobbed>,
    next_stick: u32,
    shown_sticks: HashMap<u32, Entity>,
    blasts: u32,
    planks_blown: u32,
    /// Where everyone stands and which way they face, by seat: a
    /// person's where they last said, copied in each frame ([`sync`]); the
    /// computer's players' where the game has moved them, which is what
    /// their bodies are sent as ([`flush`]). Kept here, and not read off
    /// the bodies as it goes, so that a fight is a plain function of the
    /// game ([`Game::step`]) and whole games can be played in a test; and
    /// to the last of a metre, where a body's [`Spot`] is whole
    /// centimetres, which a slow step rounds to nothing.
    at: Vec<Vec3>,
    yaw: Vec<f32>,
    /// Whose body is in the game's room to be played with, by seat: a
    /// person's while it is there; the computer's always.
    present: Vec<bool>,
    /// What a step of the fight has done that the world is to be told or
    /// shown ([`Event`]), until it has been ([`flush`]).
    events: Vec<Event>,
    /// The map as the computer's players know it, with every bridge laid
    /// in this game on it ([`Nav`]): people's, as their rungs come
    /// ([`Ask::Rung`]), and the bots' own.
    nav: Nav,
    /// The computer's players in the players' seats ([`Squad`]): what
    /// each is doing, and its body, which is what moves it. The zombies
    /// are not theirs: they keep their minds ([`Mind`], [`think`]).
    squad: Squad,
    /// The computer's players as they were until 2026-11-12, each kept to
    /// its island by a [`Mind`] ([`think`]): for going back to without a
    /// deploy, should the new ones go wrong (`RT_ARENA_BOTS=old`).
    old_bots: bool,
    /// The game's own dice, for the generators and the computer's
    /// players: seeded, so that a game played in a test plays the same
    /// each time.
    dice: fastrand::Rng,
    /// What its room was last sent, to send again only what has changed.
    sent: Option<ArenaRound>,
    sent_seats: Vec<Member>,
    sent_purses: Vec<Purse>,
    sent_kits: Vec<Kit>,
    sent_upgrades: [Upgrades; ARENA_TEAMS],
    /// Its bots and planks have gone ([`close`]); everyone goes back to the
    /// town next tick ([`end`]).
    closing: bool,
}

/// The order a game's seats are taken in as people join (since 2026-11-12,
/// later that day; Hajun: "i want to change it so that this game is always
/// 2v2v2v2, with bots filling the remaining slots, and the 2 player become
/// same team. and if its 4 players, then those 2 players and 2 players are
/// same team too, and so on, so that real players will be teamed up with
/// each other first, and bots on remaining teams"): both seats of the
/// first island, both of the one across the map from it, and then both of
/// each of the two at its sides. So two people are a team, four are two
/// teams on opposite islands, and whoever is the odd one has the computer
/// for a teammate; pairs go by the order people join in. A seat is an
/// island and a team by its number ([`arena_team`]), so the seats between,
/// taken by nobody yet, are kept empty while the room fills
/// ([`Game::vacant`]), and the computer takes every one of them as the
/// wait ends. For half a day before, the order was an island each, 0, 2,
/// 1, 3, 4, 6, 5, 7, and two to four people began with no bots at all, a
/// 1v1 for two.
const SEAT_ORDER: [usize; ARENA_SEATS] = [0, 4, 2, 6, 1, 5, 3, 7];

/// Something a step of the fight has done ([`Game::step`], and what
/// players ask, [`hear`]) that takes the world to carry out: an entity for
/// the room to be sent, or no longer, or a word to someone's device. The
/// game only says so; [`flush`] does it, after.
#[derive(Clone, Debug)]
enum Event {
    /// A generator has made this: the room is sent it ([`Loot`]).
    Made(Piece),
    /// Someone has taken the thing numbered so: the room is sent it no
    /// longer.
    Taken(u32),
    /// A fireball, numbered so, has been thrown: the room is sent it.
    Thrown { ball: u32, shown: Fireball },
    /// The fireball numbered so has burst: the room is sent it no longer,
    /// which is how every device knows to draw the blast.
    Burst(u32),
    /// A stick of dynamite, numbered so, has been thrown: the room is
    /// sent it.
    Lobbed { stick: u32, shown: Dynamite },
    /// The stick numbered so has blown up: the room is sent it no longer,
    /// and is sent where it blew and which planks went with it, for good.
    Blew { stick: u32, shown: Blasted },
    /// The stick numbered so came down where there was nothing to lie
    /// on, and is gone into the void: the room is sent it no longer.
    Fell(u32),
    /// The person in `seat` has been knocked, by whoever has the id `by`
    /// from `from`, who was `running` if so: their device is told, and
    /// knocks its own body.
    Knocked { seat: usize, by: u64, from: Spot, running: bool },
    /// The person in this seat is sent to where they arrive on their
    /// island: as the fight begins, and each time they stand again.
    Land(usize),
    /// A zombie has risen in this seat: a body for it, for the room.
    Zombie(usize),
    /// One of the computer's players has laid this rung: the room is sent
    /// it, as it is a person's ([`Ask::Rung`]).
    Rung(Plank),
}

/// The turn about the upright a body facing as `turn` has, in radians:
/// what [`Game::yaw`] keeps.
fn yaw_of(turn: Quat) -> f32 {
    turn.to_euler(EulerRot::YXZ).0
}

/// Who is in a seat nobody is in: nobody, with no name and no look. A
/// device takes a member with no id for an empty seat.
fn nobody() -> Member {
    Member {
        id: 0,
        name: String::new(),
        look: String::new(),
        ai: true,
    }
}

/// A stick of dynamite thrown in a game: its number, how far through its
/// throw and its fuse it is ([`Lob`]), who threw it, by seat, and whether
/// it has come down and found ground to lie on.
#[derive(Clone, Copy, Debug)]
struct Lobbed {
    id: u32,
    lob: Lob,
    by: usize,
    lies: bool,
}

/// A bot's mind: how long until it may punch again, what a punch is carrying
/// it by across the ground and how fast it is rising from one (0 on the
/// ground), how fast it is falling once knocked off its island, when it
/// next thinks of somewhere to walk, and where.
#[derive(Default, Clone, Copy)]
struct Mind {
    cooldown: f32,
    shove: Vec2,
    lift: f32,
    falling: Option<f32>,
    think: f32,
    goal: Option<Vec2>,
}

/// Gravity on a body knocked up, on the way up and on the way down, in
/// metres a second a second: the game's `GRAVITY` and `FALL_GRAVITY`.
const RISE_GRAVITY: f32 = 22.0;

impl Game {
    /// A game numbered `id`, its waiting room empty, whose room is sent
    /// the entity `shown`; its dice thrown from `seed`.
    fn new(id: u32, shown: Entity, seed: u64) -> Self {
        let mut dice = fastrand::Rng::with_seed(seed);
        Self {
            id,
            shown,
            seats: Vec::new(),
            members: Vec::new(),
            bot: Vec::new(),
            minds: Vec::new(),
            gone: Vec::new(),
            vacant: Vec::new(),
            stage: ArenaStage::Waiting,
            left: 0.0,
            since_join: 0.0,
            unwatched: 0.0,
            health: Vec::new(),
            since_hit: Vec::new(),
            hurt: Vec::new(),
            dead_for: Vec::new(),
            out: Vec::new(),
            anchors: [ANCHOR_HEALTH; ARENA_TEAMS],
            anchor_hurt: [0.0; ARENA_TEAMS],
            planks: Vec::new(),
            rungs: Vec::new(),
            elapsed: 0.0,
            mint: Mint::new(0.0, &mut || dice.f32()),
            loot: HashMap::new(),
            purses: Vec::new(),
            outfits: Outfits::default(),
            balls: Vec::new(),
            next_ball: 0,
            shown_balls: HashMap::new(),
            sticks: Vec::new(),
            next_stick: 0,
            shown_sticks: HashMap::new(),
            blasts: 0,
            planks_blown: 0,
            at: Vec::new(),
            yaw: Vec::new(),
            present: Vec::new(),
            events: Vec::new(),
            nav: Nav::arena(),
            squad: Squad::new(seed, Skill::MODERATE),
            old_bots: false,
            dice,
            sent: None,
            sent_seats: Vec::new(),
            sent_purses: Vec::new(),
            sent_kits: Vec::new(),
            sent_upgrades: Default::default(),
            closing: false,
        }
    }

    fn room(&self) -> Room {
        Room::Arena(self.id)
    }

    fn seat_of(&self, body: Entity) -> Option<usize> {
        self.seats.iter().position(|&seat| seat == body)
    }

    fn over(&self) -> bool {
        matches!(self.stage, ArenaStage::Over { .. })
    }

    fn playing(&self) -> bool {
        self.stage == ArenaStage::Playing
    }

    /// Whether `seat` is standing: alive, and not gone or out.
    fn alive(&self, seat: usize) -> bool {
        self.health[seat] > 0.0 && !self.gone[seat] && !self.out[seat]
    }

    /// Whether `seat` is still in play: standing, or dead and coming back.
    fn in_play(&self, seat: usize) -> bool {
        !self.gone[seat] && !self.out[seat]
    }

    /// Whether whoever is in `seat` is one of the computer's players that
    /// plays the game ([`Squad`]): in a player's seat, and not, by the
    /// switch, as they were before.
    fn brained(&self, seat: usize) -> bool {
        !self.old_bots && seat < ARENA_SEATS && self.bot[seat]
    }

    /// Which zombie `seat` is, if it is one's: the seats past the players'.
    fn zombie(&self, seat: usize) -> Option<usize> {
        seat.checked_sub(ARENA_SEATS)
    }

    /// The ground the computer's player in `seat` keeps to, as the middle
    /// of a disc and how far it reaches: its island, or a zombie's home.
    fn home(&self, seat: usize) -> (Vec2, f32) {
        match self.zombie(seat) {
            Some(zombie) => zombie_home(zombie),
            None => (ARENA_ISLANDS[arena_island(seat)], ARENA_ISLAND_RADIUS),
        }
    }

    /// Where the computer's player in `seat` stands as it comes into the
    /// fight, and again each time it stands again, and which way it faces.
    fn bot_spot(&self, seat: usize) -> Spot {
        match self.zombie(seat) {
            Some(zombie) => {
                let at = zombie_spot(zombie);
                let (home, _) = zombie_home(zombie);
                let toward = (home - at).normalize_or(Vec2::NEG_Y);
                Spot::of(Vec3::new(at.x, ISLAND_TOP, at.y), Quat::from_rotation_y((-toward.x).atan2(-toward.y)))
            }
            None => {
                let (at, turn) = arrival(Venue::Arena, seat);
                Spot::of(at.with_y(ISLAND_TOP), turn)
            }
        }
    }

    /// Who the computer's player in `seat` is: a boy or a girl by the toss
    /// of a coin, in any of that side's colours, under one of that side's
    /// names that nobody in the game has.
    fn bot_member(&self, seat: usize) -> Member {
        let boy = fastrand::bool();
        let names: Vec<&str> = roundtown_net::ai_names(boy)
            .iter()
            .copied()
            .filter(|name| self.members.iter().all(|member| !member.name.starts_with(name)))
            .collect();
        Member {
            id: bot_id(self.id, seat),
            name: fastrand::choice(names).unwrap_or("Bot").into(),
            look: roundtown_net::ai_look(boy, fastrand::usize(..)).into(),
            ai: true,
        }
    }

    /// Whoever in `seat` has their feet at `at` picks up everything a
    /// generator has made that is within reach of them, dropping or lying
    /// (`Mint::pick_up`), if they are a person and standing: it is
    /// theirs, and gone from the pads. What they took, for the room to
    /// stop being sent it.
    fn gather(&mut self, seat: usize, at: Vec3) -> Vec<Piece> {
        if self.bot[seat] || !self.alive(seat) {
            return Vec::new();
        }
        let taken = self.mint.pick_up(at, self.elapsed);
        for piece in &taken {
            let held = &mut self.purses[seat][piece.item.index()];
            *held = held.saturating_add(1);
        }
        taken
    }

    /// Sells `ware` to whoever in `seat` stands at `at`, if the fight is
    /// on, they are standing, near the shop that sells it, it is to be had
    /// by them, and they hold its price, which comes out of their purse
    /// ([`Outfits::buy`]): whether it is theirs.
    fn buy(&mut self, seat: usize, ware: Ware, at: Vec2) -> bool {
        if !self.playing() || !self.alive(seat) || !at_a_shop(at, sold_at(ware)) {
            return false;
        }
        self.outfits.buy(seat, ware, &mut self.purses[seat])
    }

    /// What the room is shown of how it is going.
    fn round(&self) -> ArenaRound {
        ArenaRound {
            stage: self.stage,
            left: match self.stage {
                ArenaStage::Waiting => 0,
                _ => self.left.max(0.0).ceil() as u16,
            },
            health: self.health.iter().map(|health| health.max(0.0).round() as u16).collect(),
            anchors: self.anchors.map(|anchor| anchor.max(0.0).round() as u16),
            // The players' seats alone: a zombie is never out.
            out: self
                .out
                .iter()
                .enumerate()
                .take(ARENA_SEATS)
                .filter(|&(_, &out)| out)
                .fold(0u8, |bits, (seat, _)| bits | (1 << seat)),
        }
    }

    /// The next of the players' seats to be taken: the first, in the order
    /// they are taken in ([`SEAT_ORDER`]), that nobody has sat in.
    fn free_seat(&self) -> Option<usize> {
        SEAT_ORDER
            .into_iter()
            .find(|&seat| seat >= self.seats.len() || self.vacant[seat])
    }

    /// The seats `count` people take who go in together, in the order they
    /// take them (since 2026-11-13, a group of friends going in with its
    /// leader, `social`; Hajun: "everyone get queued at the same time in
    /// the same team (unless group is >2 and ur playing 2v2v2v2 bridge
    /// wars, then it will need to be 2 ppl same team 1 ppl other team i
    /// guess)"): two by two onto teams with both seats free, in the order
    /// the teams fill in ([`SEAT_ORDER`], which has each team's two seats
    /// together), and an odd one last, wherever the next to come alone
    /// would sit. One alone takes [`Game::free_seat`]. None if the room is
    /// no longer waiting, or has not the teams free.
    fn seats_for(&self, count: usize) -> Option<Vec<usize>> {
        if self.stage != ArenaStage::Waiting {
            return None;
        }
        let free = |seat: usize| seat >= self.seats.len() || self.vacant[seat];
        let mut seats = Vec::with_capacity(count);
        let mut teams = SEAT_ORDER.chunks(2);
        for _ in 0..count / 2 {
            let pair = teams.by_ref().find(|pair| pair.iter().all(|&seat| free(seat)))?;
            seats.extend_from_slice(pair);
        }
        if count % 2 == 1 {
            seats.push(SEAT_ORDER.into_iter().find(|&seat| free(seat) && !seats.contains(&seat))?);
        }
        Some(seats)
    }

    /// How many people are in it: not the computer's players, nor whoever
    /// has left.
    fn people(&self) -> usize {
        (0..self.seats.len())
            .filter(|&seat| !self.bot[seat] && !self.vacant[seat] && !self.gone[seat])
            .count()
    }

    /// Seats `member`, whose body is `body`, in the next of the players'
    /// seats ([`Game::free_seat`]); or, with those all taken or the fight
    /// begun, in the next seat past them, as a zombie is. Only the tests
    /// seat anyone so since 2026-11-13: people are seated together, each
    /// where [`Game::seats_for`] says ([`seat_together`]).
    #[cfg(test)]
    fn seat(&mut self, body: Entity, member: Member, bot: bool) -> usize {
        let seat = match self.free_seat() {
            Some(seat) if self.stage == ArenaStage::Waiting => seat,
            _ => self.seats.len().max(ARENA_SEATS),
        };
        self.put(seat, body, member, bot)
    }

    /// Seats `member`, whose body is `body`, in `seat`, with every seat
    /// before it that nobody is in yet kept empty.
    fn put(&mut self, seat: usize, body: Entity, member: Member, bot: bool) -> usize {
        while self.seats.len() <= seat {
            self.push_vacant();
        }
        self.seats[seat] = body;
        self.members[seat] = member;
        self.bot[seat] = bot;
        self.minds[seat] = Mind {
            think: THINK_EVERY * seat as f32 / ARENA_SEATS as f32,
            ..default()
        };
        self.gone[seat] = false;
        self.vacant[seat] = false;
        self.health[seat] = HEALTH;
        self.out[seat] = false;
        self.since_join = 0.0;
        // In the waiting room, until the game says otherwise: where a
        // person is put as they are seated, and where the computer's
        // waits.
        let (at, turn) = waiting_spot(seat);
        self.at[seat] = at;
        self.yaw[seat] = yaw_of(turn);
        self.present[seat] = true;
        if bot && seat < ARENA_SEATS {
            self.squad.seat(seat, at, yaw_of(turn));
        } else {
            self.squad.unseat(seat);
        }
        seat
    }

    /// Empties `seat`: nobody in it, gone and out, with no body, as a seat
    /// nobody has taken yet is ([`Game::push_vacant`]), for the next to
    /// come, a person or the computer, to take.
    fn vacate(&mut self, seat: usize) {
        self.seats[seat] = Entity::PLACEHOLDER;
        self.members[seat] = nobody();
        self.bot[seat] = false;
        self.minds[seat] = Mind::default();
        self.gone[seat] = true;
        self.vacant[seat] = true;
        self.health[seat] = 0.0;
        self.since_hit[seat] = 0.0;
        self.hurt[seat] = 0.0;
        self.dead_for[seat] = None;
        self.out[seat] = true;
        self.rungs[seat] = 0;
        self.purses[seat] = Purse::default();
        self.present[seat] = false;
        self.squad.unseat(seat);
        if let Some(kit) = self.outfits.kits.get_mut(seat) {
            *kit = Kit::default();
        }
    }

    /// Whoever was in `seat` has left the game: gone back to the town, or
    /// their connection has. Before the fight begins, in the waiting room
    /// or through the countdown, their seat is anyone's again
    /// ([`Game::vacate`]), a person's who comes or the computer's, so
    /// that no team begins short (until 2026-11-12 it stayed theirs,
    /// empty, a hole in a team for the whole game); once it has begun, it
    /// stays theirs and empty.
    fn leave(&mut self, seat: usize) {
        if seat < ARENA_SEATS && matches!(self.stage, ArenaStage::Waiting | ArenaStage::Countdown) {
            info!("Bridge Wars game {}: seat {seat} left before the fight, and is free again", self.id);
            self.vacate(seat);
        } else {
            self.gone[seat] = true;
        }
    }

    /// One more seat with nobody in it: gone and out, with no body.
    fn push_vacant(&mut self) {
        self.seats.push(Entity::PLACEHOLDER);
        self.members.push(nobody());
        self.bot.push(false);
        self.minds.push(Mind::default());
        self.gone.push(true);
        self.vacant.push(true);
        self.health.push(0.0);
        self.since_hit.push(0.0);
        self.hurt.push(0.0);
        self.dead_for.push(None);
        self.out.push(true);
        self.rungs.push(0);
        self.purses.push(Purse::default());
        self.at.push(Vec3::ZERO);
        self.yaw.push(0.0);
        self.present.push(false);
        self.outfits.seat(self.seats.len());
    }

    /// Makes the players' seats eight, with nobody in whichever nobody
    /// has taken: a zombie's seat is past the players' eight, whoever is
    /// in them ([`arena_team`]). The computer has taken every seat left
    /// by the time the fight begins, so this only keeps the count.
    fn pad(&mut self) {
        while self.seats.len() < ARENA_SEATS {
            self.push_vacant();
        }
    }

    /// `seat` is dead: for [`RESPAWN_SECS`], and then back, if its team's
    /// anchor stands now; otherwise for good. A zombie is always back, a
    /// while later ([`ZOMBIE_RESPAWN_SECS`]).
    fn die(&mut self, seat: usize) {
        self.health[seat] = 0.0;
        self.minds[seat] = Mind::default();
        // The metres of bridge one of the computer's players had go with
        // the rest, and the bridge it was laying is anyone's to finish.
        self.squad.died(seat, self.elapsed);
        // Their sword and what was on their second button go with them,
        // and everything they held of what the generators make (since
        // 2026-11-12; Hajun: "death make u lose all ur silver, gold,
        // sapphire, amethyst, bridge meters"; the metres of bridge are
        // their own device's to take). The room is sent the empty purse
        // with the rest (`Purses`).
        self.outfits.die(seat);
        if let Some(purse) = self.purses.get_mut(seat) {
            *purse = Purse::default();
        }
        if self.zombie(seat).is_some() {
            self.dead_for[seat] = Some(ZOMBIE_RESPAWN_SECS);
            return;
        }
        if self.anchors[usize::from(arena_team(seat))] > 0.0 {
            self.dead_for[seat] = Some(RESPAWN_SECS);
        } else {
            self.dead_for[seat] = None;
            self.out[seat] = true;
            info!("Bridge Wars game {}: seat {seat} is out", self.id);
        }
    }

    /// Takes a punch's damage off `victim`, hit from `from` by `hitter`, who
    /// was `running` if so: more with a sword in the hitter's hand, less
    /// through the victim's shield ([`Outfits::punch`]). Not while the
    /// victim is still reeling from the last punch that took
    /// ([`HURT_TIME`]): a device lands a punch at every press, several a
    /// second. A hitter who was unseen is seen again, whether or not the
    /// punch took ("being hit or attacking reveals u").
    fn wound(&mut self, victim: usize, hitter: usize, from: Spot, running: bool) {
        self.outfits.reveal(hitter);
        if self.hurt[victim] > 0.0 {
            return;
        }
        self.hurt[victim] = HURT_TIME;
        let damage = self.outfits.punch(DAMAGE, hitter, victim);
        self.harm(victim, hitter, from, running, damage);
    }

    /// Takes `damage` off `victim`, by `hitter`'s doing, and no more than
    /// that: dead at none. Whoever was unseen is seen again for being
    /// hit. Whether they are still standing. What a stick of dynamite's
    /// blast does to whoever it reaches: it knocks nobody.
    fn hurt_by(&mut self, victim: usize, hitter: usize, damage: f32) -> bool {
        self.outfits.reveal(victim);
        self.health[victim] = (self.health[victim] - damage).max(0.0);
        self.since_hit[victim] = 0.0;
        if self.health[victim] <= 0.0 {
            info!("Bridge Wars game {}: seat {hitter} killed seat {victim}", self.id);
            self.die(victim);
            return false;
        }
        debug!("Bridge Wars game {}: seat {hitter} hit seat {victim}, left with {}", self.id, self.health[victim]);
        true
    }

    /// Takes `damage` off `victim`, struck from `from` by `hitter`: dead at
    /// none, and knocked back from there otherwise, further if `running`,
    /// a person through their device ([`Event::Knocked`]) and a bot here.
    /// Whoever was unseen is seen again for being hit.
    fn harm(&mut self, victim: usize, hitter: usize, from: Spot, running: bool, damage: f32) {
        if !self.hurt_by(victim, hitter, damage) {
            return;
        }
        if self.brained(victim) {
            self.squad.knock(victim, from.translation().xz(), running);
            return;
        }
        if self.bot[victim] {
            let away = (self.at[victim].xz() - from.translation().xz()).normalize_or(Vec2::NEG_Y);
            // A zombie takes a share of the knock: it keeps coming.
            let taken = if self.zombie(victim).is_some() { ZOMBIE_KNOCK } else { 1.0 };
            let push = (KNOCK_BACK + if running { SPRINT_BONUS } else { 0.0 }) * taken;
            let up = KNOCK_UP * taken;
            let mind = &mut self.minds[victim];
            mind.shove = mind.shove * 0.5 + away * push;
            mind.lift = (mind.lift * 0.5 + up).min(up);
            return;
        }
        self.events.push(Event::Knocked {
            seat: victim,
            by: self.members[hitter].id,
            from,
            running,
        });
    }

    /// Which teams still have anyone in play: the players' teams, the
    /// zombies being nobody's.
    fn teams_in_play(&self) -> Vec<u8> {
        let mut teams: Vec<u8> = (0..self.seats.len().min(ARENA_SEATS))
            .filter(|&seat| self.in_play(seat))
            .map(arena_team)
            .collect();
        teams.sort_unstable();
        teams.dedup();
        teams
    }

    /// Puts `seat` where it arrives on its island, as the fight begins and
    /// each time it stands again: the computer's player there at once, a
    /// person through their device ([`Event::Land`]).
    fn stand(&mut self, seat: usize) {
        if self.vacant[seat] {
            return;
        }
        if self.bot[seat] {
            let spot = self.bot_spot(seat);
            self.at[seat] = spot.translation();
            self.yaw[seat] = yaw_of(spot.rotation());
            self.minds[seat] = Mind {
                think: THINK_EVERY * seat as f32 / ARENA_SEATS as f32,
                ..default()
            };
            self.squad.stand(seat, self.at[seat], self.yaw[seat], self.elapsed);
            return;
        }
        self.events.push(Event::Land(seat));
    }

    /// Something on the timetable happens ([`TIMETABLE`]): a tier is the
    /// mint's own, by the time; every anchor left breaks; or the zombies
    /// come.
    fn happen(&mut self, event: Happening) {
        info!("Bridge Wars game {}: {event:?}, {:.0} s in", self.id, self.elapsed);
        match event {
            Happening::Tier { .. } => {}
            Happening::AnchorsBreak => self.anchors = [0.0; ARENA_TEAMS],
            Happening::SuddenDeath => {
                for zombie in 0..ZOMBIES {
                    self.seat_zombie(zombie);
                }
            }
        }
    }

    /// A zombie, the one numbered `zombie`, in the next seat, past the
    /// players': standing where it comes up ([`zombie_spot`]), with
    /// everyone's health, and a mind as the computer's other players
    /// have. Its body is the world's to make ([`Event::Zombie`]).
    fn seat_zombie(&mut self, zombie: usize) {
        let seat = self.seats.len().max(ARENA_SEATS);
        debug_assert_eq!(seat, ARENA_SEATS + zombie, "the zombies come once the players' seats are full");
        let member = Member {
            id: bot_id(self.id, seat),
            name: ZOMBIE_NAME.into(),
            look: ZOMBIE_LOOK.into(),
            ai: true,
        };
        self.put(seat, Entity::PLACEHOLDER, member, true);
        self.stand(seat);
        self.events.push(Event::Zombie(seat));
    }

    /// Whoever in `seat`, standing at `at`, throws a fireball the way
    /// `heading` says, if there is one on their second button: it is in
    /// the air, the room is sent it ([`Event::Thrown`]), and the game
    /// flies it ([`Game::fireballs`]). Whether it was thrown.
    fn throw(&mut self, seat: usize, at: Vec3, heading: f32) -> bool {
        if !self.playing() || !self.alive(seat) || !heading.is_finite() || !self.outfits.spend(seat, Utility::Fireball) {
            return false;
        }
        let flight = Flight::thrown(at, heading);
        let ball = self.next_ball;
        self.next_ball = self.next_ball.wrapping_add(1);
        self.events.push(Event::Thrown {
            ball,
            shown: Fireball {
                seat: seat as u8,
                x: flight.from.x,
                y: flight.from.y,
                z: flight.from.z,
                heading,
            },
        });
        self.balls.push((ball, flight, seat));
        debug!("Bridge Wars game {}: seat {seat} threw a fireball", self.id);
        true
    }

    /// Whoever in `seat`, standing at `at`, throws a stick of dynamite the
    /// way `heading` says, to come down `far` from them, if there is one
    /// on their buttons: it is in the air, the room is sent it
    /// ([`Event::Lobbed`]), and the game lobs it on and blows it
    /// ([`Game::dynamites`]). It is thrown at the ground where it is to
    /// come down, as high as that is now, or, with none there, as high as
    /// the ground the thrower stands on. Whether it was thrown.
    fn lob(&mut self, seat: usize, at: Vec3, heading: f32, far: f32) -> bool {
        if !self.playing() || !self.alive(seat) || !heading.is_finite() || !far.is_finite() {
            return false;
        }
        if !self.outfits.spend(seat, Utility::Dynamite) {
            return false;
        }
        let aimed = Lob::thrown(at, heading, far, at.y);
        let floor = self.nav.ground(aimed.to.xz()).or(self.nav.ground(at.xz())).unwrap_or(at.y);
        let lob = Lob::thrown(at, heading, far, floor);
        let stick = self.next_stick;
        self.next_stick = self.next_stick.wrapping_add(1);
        self.events.push(Event::Lobbed {
            stick,
            shown: Dynamite {
                seat: seat as u8,
                x: lob.from.x,
                y: lob.from.y,
                z: lob.from.z,
                to_x: lob.to.x,
                to_y: lob.to.y,
                to_z: lob.to.z,
            },
        });
        self.sticks.push(Lobbed {
            id: stick,
            lob,
            by: seat,
            lies: false,
        });
        debug!("Bridge Wars game {}: seat {seat} threw a stick of dynamite {:.1} m", self.id, lob.chord());
        true
    }

    /// A punch by whoever is in `hitter`, standing at `from` across the
    /// ground, on the respawn anchor of `team`, if it is another team's,
    /// still stands and is within `reach` of them ([`anchor_in_reach`]):
    /// the punch's damage off it, more with a sword as off a body, and no
    /// oftener than a body takes one ([`HURT_TIME`]). Hitting an anchor is
    /// attacking: whoever was unseen is seen again, whether or not the hit
    /// takes. Whether it took.
    fn strike_anchor(&mut self, hitter: usize, team: usize, from: Vec2, reach: f32) -> bool {
        if !self.playing() || !self.alive(hitter) || team >= ARENA_TEAMS || team == usize::from(arena_team(hitter)) {
            return false;
        }
        if self.anchors[team] <= 0.0 || !anchor_in_reach(from, ARENA_ANCHORS[team], reach) {
            return false;
        }
        self.outfits.reveal(hitter);
        if self.anchor_hurt[team] > 0.0 {
            return false;
        }
        self.anchor_hurt[team] = HURT_TIME;
        // A sword breaks an anchor sooner, as it does a body.
        let damage = punch_damage(DAMAGE, &self.outfits.kit(hitter), &self.outfits.team(hitter));
        self.anchors[team] = (self.anchors[team] - damage).max(0.0);
        if self.anchors[team] <= 0.0 {
            info!("Bridge Wars game {}: seat {hitter} broke team {team}'s anchor", self.id);
        }
        true
    }

    /// One frame of the fight, `dt` seconds of it: the clock and the
    /// timetable, the generators, everyone's health and the dead standing
    /// again, the computer's players, the fireballs, the potions, traps
    /// and pools, and whether anyone has won. A plain function of the
    /// game: it reads where people are from [`Game::at`], and whatever it
    /// does that the world is to be shown or told it only says
    /// ([`Game::events`]), so that a whole game can be played in a test.
    fn step(&mut self, dt: f32) {
        self.left -= dt;
        let before = self.elapsed;
        self.elapsed += dt;
        let due: Vec<Happening> = happening_between(before, self.elapsed).collect();
        for event in due {
            self.happen(event);
        }
        self.mint_step();
        self.fight(dt);
        self.fireballs(dt);
        self.dynamites(dt);
        self.outfit(dt);
        let teams = self.teams_in_play();
        let winner = match (teams.len(), self.left <= 0.0) {
            (0, _) => Some(None),
            (1, _) => Some(Some(teams[0])),
            (_, true) => Some(None),
            _ => None,
        };
        if let Some(winner) = winner {
            self.stage = ArenaStage::Over { winner };
            self.left = RESULTS_SECS;
            info!("Bridge Wars game {}: over, team {winner:?} wins", self.id);
        }
    }

    /// The generators, for one frame of the fight: what they make comes up
    /// on its pad for the whole room ([`Event::Made`]), and what anyone
    /// has come near enough to is theirs, and gone from the room
    /// ([`Event::Taken`]).
    fn mint_step(&mut self) {
        // Each team's own generator as fast as the team has raised it.
        let forge = self.outfits.teams.map(|team| team.forge);
        let dice = &mut self.dice;
        let made = self.mint.run(self.elapsed, &forge, &mut || dice.f32());
        self.events.extend(made.into_iter().map(Event::Made));
        for seat in 0..self.seats.len() {
            if self.bot[seat] || self.gone[seat] || !self.present[seat] {
                continue;
            }
            let taken = self.gather(seat, self.at[seat]);
            if !taken.is_empty() {
                debug!("Bridge Wars game {}: seat {seat} picked up {}, and holds {:?}", self.id, taken.len(), self.purses[seat]);
            }
            self.events.extend(taken.into_iter().map(|piece| Event::Taken(piece.id)));
        }
        // And then the computer's players, each no more than its next
        // purchases need ([`Squad::wants`]): whoever of them it matters
        // most to first, so that a team's bridge comes before its
        // keeper's shield. People are served before any of them, whatever
        // their seats.
        let mut takers: Vec<(u8, usize)> = (0..self.seats.len().min(ARENA_SEATS))
            .filter(|&seat| self.brained(seat) && self.alive(seat))
            .map(|seat| (self.squad.wants(seat).1, seat))
            .collect();
        takers.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        for (_, seat) in takers {
            let (mut wants, _) = self.squad.wants(seat);
            if wants == [0; 4] {
                continue;
            }
            let taken = self.mint.pick_up_where(self.at[seat], self.elapsed, &mut |piece| {
                let wanted = &mut wants[piece.item.index()];
                let take = *wanted > 0;
                *wanted = wanted.saturating_sub(1);
                take
            });
            for piece in &taken {
                let held = &mut self.purses[seat][piece.item.index()];
                *held = held.saturating_add(1);
            }
            if !taken.is_empty() {
                debug!("Bridge Wars game {}: seat {seat} (bot) picked up {}, and holds {:?}", self.id, taken.len(), self.purses[seat]);
            }
            self.events.extend(taken.into_iter().map(|piece| Event::Taken(piece.id)));
        }
    }

    /// Everyone's health coming back, the dead standing again or going
    /// out, and the bots' moves, for one frame of the fight.
    fn fight(&mut self, dt: f32) {
        for hurt in self.hurt.iter_mut().chain(self.anchor_hurt.iter_mut()) {
            *hurt = (*hurt - dt).max(0.0);
        }
        for seat in 0..self.seats.len() {
            if !self.in_play(seat) {
                continue;
            }
            if let Some(left) = self.dead_for[seat] {
                let left = left - dt;
                if left > 0.0 {
                    self.dead_for[seat] = Some(left);
                } else {
                    self.dead_for[seat] = None;
                    self.health[seat] = HEALTH;
                    self.since_hit[seat] = 0.0;
                    self.stand(seat);
                    info!("Bridge Wars game {}: seat {seat} stands again", self.id);
                }
                continue;
            }
            if self.health[seat] <= 0.0 {
                continue;
            }
            self.since_hit[seat] += dt;
            if self.since_hit[seat] >= REGEN_AFTER && self.health[seat] < HEALTH {
                self.health[seat] = (self.health[seat] + REGEN_PER_SEC * dt).min(HEALTH);
            }
        }
        // The bots, each with everyone else where they stand this frame:
        // but for whoever is unseen, whom they do not go for.
        let standing: Vec<(usize, Vec2)> = (0..self.seats.len())
            .filter(|&seat| self.alive(seat) && self.present[seat] && self.minds[seat].falling.is_none())
            .filter(|&seat| !self.outfits.kit(seat).unseen(self.elapsed))
            .map(|seat| (seat, self.at[seat].xz()))
            .collect();
        // What the computer's players see of everyone, by seat.
        let mut seen: Vec<Seen> = (0..self.seats.len())
            .map(|seat| Seen {
                at: self.at[seat],
                alive: self.alive(seat) && self.present[seat],
                in_play: self.in_play(seat),
                unseen: self.outfits.kit(seat).unseen(self.elapsed),
                person: !self.bot[seat] && !self.vacant[seat],
                health: self.health[seat] / HEALTH,
            })
            .collect();
        for seat in 0..self.seats.len() {
            if !self.bot[seat] || !self.alive(seat) {
                continue;
            }
            if self.brained(seat) {
                self.play_bot(seat, &mut seen, dt);
                continue;
            }
            let was = Spot::of(self.at[seat], Quat::from_rotation_y(self.yaw[seat]));
            if let Some(victim) = think(self, seat, &standing, dt) {
                // A bot's punch is thrown on the run; a zombie's, which
                // comes on at a walk, knocks as one thrown standing does.
                let running = self.zombie(seat).is_none();
                self.wound(victim, seat, was, running);
            }
            if self.minds[seat].falling.is_some() && self.at[seat].y < ISLAND_TOP - FALL_TO_DIE {
                info!("Bridge Wars game {}: seat {seat} fell", self.id);
                self.die(seat);
            }
        }
    }

    /// One frame of the computer's player in `seat` ([`Squad::step`]): it
    /// thinks and moves, and what comes of it is carried out here, as it
    /// would be for a person who asked: a punch landed, a purchase made,
    /// a rung laid for the room to be sent, a potion drunk. Fallen far
    /// enough, it is dead.
    fn play_bot(&mut self, seat: usize, seen: &mut [Seen], dt: f32) {
        let was = Spot::of(self.at[seat], Quat::from_rotation_y(self.yaw[seat]));
        // Every stick of dynamite there is, as a bot sees one: where it
        // lies or will, how long until it blows, and whose it is.
        let sticks: Vec<Stick> = self
            .sticks
            .iter()
            .map(|stick| Stick {
                at: stick.lob.to.xz(),
                left: (stick.lob.secs() + DYNAMITE_FUSE - stick.lob.age).max(0.0),
                team: arena_team(stick.by),
            })
            .collect();
        let view = View {
            elapsed: self.elapsed,
            nav: &self.nav,
            seats: seen,
            purses: &self.purses,
            outfits: &self.outfits,
            mint: &self.mint,
            anchors: self.anchors,
            sticks: &sticks,
        };
        let order = self.squad.step(seat, &view, dt);
        if let Some(bot) = self.squad.bot(seat) {
            self.at[seat] = bot.body.at;
            self.yaw[seat] = bot.body.yaw;
            seen[seat].at = bot.body.at;
        }
        if let Some(victim) = order.punch
            && victim < self.seats.len()
            && self.alive(victim)
            && arena_team(victim) != arena_team(seat)
            && self.at[victim].xz().distance(self.at[seat].xz()) <= REACH + BODY_RADIUS + 0.5
        {
            self.wound(victim, seat, was, order.running);
        }
        if let Some(team) = order.strike {
            self.strike_anchor(seat, usize::from(team), self.at[seat].xz(), REACH);
        }
        if let Some(ware) = order.buy
            && self.buy(seat, ware, self.at[seat].xz())
        {
            debug!("Bridge Wars game {}: seat {seat} (bot) bought {ware:?}", self.id);
            self.squad.bought(seat, ware, self.elapsed);
        }
        for (chain, index, at, heading) in order.rungs {
            if self.rungs[seat] >= MOST_RUNGS {
                break;
            }
            self.rungs[seat] += 1;
            self.nav.hear(seat as u8, chain, index, at);
            self.events.push(Event::Rung(Plank {
                seat: seat as u8,
                chain,
                index,
                x: at.x,
                z: at.y,
                heading,
            }));
        }
        // A Bridge Builder sent off: one comes off its buttons, as off a
        // person's who sends one (`Use::Builder`), and its bridge comes
        // as rungs in the frames after, as a person's does.
        if order.build {
            if self.outfits.spend(seat, Utility::Builder) {
                debug!("Bridge Wars game {}: seat {seat} (bot) sent a Bridge Builder", self.id);
            } else {
                self.squad.unsend(seat);
            }
        }
        if let Some(heading) = order.throw {
            self.throw(seat, self.at[seat], heading);
        }
        if let Some((heading, far)) = order.blast {
            self.lob(seat, self.at[seat], heading, far);
        }
        if let Some(potion) = order.drink
            && self.outfits.drink(seat, potion, self.elapsed) == Some(Utility::Healing)
        {
            self.health[seat] = (self.health[seat] + HEALTH * HEALING).min(HEALTH);
        }
        if self.at[seat].y < ISLAND_TOP - FALL_TO_DIE {
            info!("Bridge Wars game {}: seat {seat} fell", self.id);
            self.die(seat);
        }
    }

    /// Every fireball in the air, for one frame of the fight: flown on,
    /// and burst where it meets someone of another team than its
    /// thrower's, or at the end of its range. Its blast takes its damage
    /// off everyone of another team it reaches, less what their shields
    /// keep off, and knocks them from where it burst; the room is no
    /// longer sent it ([`Event::Burst`]), which is how every device knows
    /// to draw the blast.
    fn fireballs(&mut self, dt: f32) {
        let mut flying = std::mem::take(&mut self.balls);
        flying.retain_mut(|(ball, flight, thrower)| {
            let spent = flight.fly(dt);
            let team = arena_team(*thrower);
            let others: Vec<(usize, Vec3)> = (0..self.seats.len())
                .filter(|&seat| arena_team(seat) != team && self.alive(seat) && self.present[seat])
                .map(|seat| (seat, self.at[seat]))
                .collect();
            if !spent && !others.iter().any(|(_, feet)| flight.meets(*feet)) {
                return true;
            }
            let burst = flight.at();
            let from = Spot::of(burst - Vec3::Y * FIREBALL_HEIGHT, Quat::IDENTITY);
            debug!("Bridge Wars game {}: seat {thrower}'s fireball burst at {burst:.1}, {:.1} m on", self.id, flight.flown);
            for (seat, feet) in others {
                if in_blast(burst, feet) {
                    let damage = self.outfits.through(seat, FIREBALL_DAMAGE);
                    self.harm(seat, *thrower, from, false, damage);
                }
            }
            self.events.push(Event::Burst(*ball));
            false
        });
        self.balls = flying;
    }

    /// Every stick of dynamite thrown, for one frame of the fight: lobbed
    /// on; as it comes down, lying there if there is ground for it to lie
    /// on ([`caught`]), a bridge's plank or the land, and gone into the
    /// void if there is not ([`Event::Fell`]); and once its fuse has burnt
    /// down, blown up. Its blast takes its damage off everyone of another
    /// team than its thrower's that it reaches, less what their shields
    /// keep off, knocking nobody (mine: Hajun's file gives the fireball a
    /// knock and this none); and every plank whose middle it reaches is
    /// gone, whoever laid it, from under whoever stands on it
    /// ([`Nav::blast`]): the computer's players fall with them, and a
    /// person's own device has them fall, told which planks went
    /// ([`Event::Blew`], [`Blasted`]). An anchor it does not touch.
    fn dynamites(&mut self, dt: f32) {
        let mut lying = std::mem::take(&mut self.sticks);
        lying.retain_mut(|stick| {
            stick.lob.tick(dt);
            if !stick.lob.down() {
                return true;
            }
            let at = stick.lob.to;
            if !stick.lies {
                if !caught(at.xz(), |spot| self.nav.ground(spot).is_some()) {
                    debug!("Bridge Wars game {}: seat {}'s dynamite fell into the void at {:.1}", self.id, stick.by, at.xz());
                    self.events.push(Event::Fell(stick.id));
                    return false;
                }
                stick.lies = true;
            }
            if !stick.lob.blown() {
                return true;
            }
            let team = arena_team(stick.by);
            let reached: Vec<usize> = (0..self.seats.len())
                .filter(|&seat| arena_team(seat) != team && self.alive(seat) && self.present[seat])
                .filter(|&seat| in_dynamite(at, self.at[seat]))
                .collect();
            for seat in reached {
                let damage = self.outfits.through(seat, DYNAMITE_DAMAGE);
                self.hurt_by(seat, stick.by, damage);
            }
            let planks = self.nav.blast(at.xz(), DYNAMITE_RADIUS);
            info!(
                "Bridge Wars game {}: seat {}'s dynamite blew at {:.1}, taking {} planks",
                self.id,
                stick.by,
                at.xz(),
                planks.len()
            );
            self.blasts += 1;
            self.planks_blown += planks.len() as u32;
            self.events.push(Event::Blew {
                stick: stick.id,
                shown: Blasted {
                    x: at.x,
                    y: at.y,
                    z: at.z,
                    planks,
                },
            });
            false
        });
        self.sticks = lying;
    }

    /// The potions, the traps and the heal pools, for one frame of the
    /// fight ([`Outfits::tick`]), with every living player where they
    /// stand: whoever a pool heals has that much health back.
    fn outfit(&mut self, dt: f32) {
        let standing: Vec<(usize, Vec2)> = (0..self.seats.len().min(ARENA_SEATS))
            .filter(|&seat| self.alive(seat) && self.present[seat])
            .map(|seat| (seat, self.at[seat].xz()))
            .collect();
        let ticked = self.outfits.tick(dt, self.elapsed, &standing);
        for seat in ticked.healed {
            self.health[seat] = (self.health[seat] + HEAL_POOL).min(HEALTH);
        }
        for (seat, island) in ticked.sprung {
            info!("Bridge Wars game {}: seat {seat} walked into team {island}'s trap", self.id);
        }
    }
}

impl Arenas {
    /// The game filling now, if it still has room for `count` who go in
    /// together ([`Game::seats_for`]); otherwise a new one, the old one
    /// left to fill with whoever is in it and the computer.
    fn filling(&mut self, commands: &mut Commands, old_bots: bool, count: usize) -> &mut Game {
        let open = self.filling.filter(|id| {
            self.games
                .get(id)
                .is_some_and(|game| !game.closing && game.seats_for(count).is_some())
        });
        let id = open.unwrap_or_else(|| {
            let id = self.next;
            self.next += 1;
            let shown = commands
                .spawn((Replicated, InRoom(Room::Arena(id)), Seats::default()))
                .id();
            let mut game = Game::new(id, shown, fastrand::u64(..));
            game.old_bots = old_bots;
            self.games.insert(id, game);
            info!("Bridge Wars game {id} opened");
            self.filling = Some(id);
            id
        });
        self.games.get_mut(&id).expect("just found or made")
    }

    /// The game in `room`, and `body`'s seat in it.
    fn of(&mut self, room: Room, body: Entity) -> Option<(&mut Game, usize)> {
        let Room::Arena(id) = room else {
            return None;
        };
        let game = self.games.get_mut(&id)?;
        let seat = game.seat_of(body)?;
        Some((game, seat))
    }

    /// `body` has left the game in `room` while it was going on: the
    /// connection went, or it went back to the town.
    pub(crate) fn walk_out(&mut self, room: Room, body: Entity) {
        if let Some((game, seat)) = self.of(room, body) {
            game.leave(seat);
        }
    }

    /// Whether the game in `room` has yet to begin its fight: in its
    /// waiting room still, or counting down. What someone's friends are
    /// told of where they are (`social`).
    pub(crate) fn queued(&self, room: Room) -> bool {
        let Room::Arena(id) = room else {
            return false;
        };
        self.games
            .get(&id)
            .is_some_and(|game| matches!(game.stage, ArenaStage::Waiting | ArenaStage::Countdown))
    }
}

/// Seats `going`, who go in together, the first of them having asked, in
/// the game filling now, or in a new one if that has not the room for them:
/// two by two onto one team ([`Game::seats_for`]), each out of the town, or
/// out of the game that was over for them, and into the waiting room.
#[allow(clippy::too_many_arguments)]
fn seat_together(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    arenas: &mut Arenas,
    towns: &mut Towns,
    old_bots: bool,
    going: &[Entity],
    now: f64,
) {
    for &body in going {
        let Ok((body_of, ..)) = bodies.get(body) else {
            continue;
        };
        if let Some((old, seat)) = arenas.of(body_of.room, body) {
            old.leave(seat);
        }
        towns.stand(body);
    }
    let game = arenas.filling(commands, old_bots, going.len());
    let Some(seats) = game.seats_for(going.len()) else {
        return;
    };
    let room = game.room();
    for (&body, seat) in going.iter().zip(seats) {
        let Ok((body_of, spot, venue, member)) = bodies.get_mut(body) else {
            continue;
        };
        game.put(seat, body, member.clone(), false);
        if let Some(client) = body_of.client {
            tell(tells, client, Tell::Seated { seat: seat as u8 });
        }
        let (at, turn) = waiting_spot(seat);
        place(
            commands,
            tells,
            body,
            (body_of, spot, venue),
            room,
            Venue::Arena,
            seat as u8,
            Some(Spot::of(at, turn)),
            now,
        );
    }
}

type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Body,
        &'static mut Spot,
        &'static mut Venue,
        &'static Member,
    ),
>;

/// What players ask of Bridge Wars.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<&Plays>,
    mut bodies: Bodies,
    mut arenas: ResMut<Arenas>,
    mut towns: ResMut<Towns>,
    (everyone, mut social): (Res<Everyone>, ResMut<Social>),
    config: Res<Config>,
    time: Res<Time>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    for ask in asks.read() {
        let Some(client) = ask.client_id.entity() else {
            continue;
        };
        let Ok(&Plays(body)) = clients.get(client) else {
            continue;
        };
        let Ok((body_of, ..)) = bodies.get(body) else {
            continue;
        };
        let room = body_of.room;
        match &ask.message {
            Ask::Arena => {
                // Whoever asks goes in, and with them, if they lead a
                // group, everyone of it who is here and free to (`social`,
                // since 2026-11-13); in a group someone else leads, nobody
                // does: only its leader starts a game.
                let Some(party) = social.party(body_of.who.0) else {
                    tell(&mut tells, client, Tell::Refused(Refusal::NotLeader));
                    continue;
                };
                // Free to: in the town, or in a game that is over for them.
                let (mut going, mut staying) = (Vec::new(), Vec::new());
                for id in party {
                    let entity = everyone.0.get(&Who(id)).copied();
                    let free = entity.is_some_and(|entity| {
                        bodies.get(entity).is_ok_and(|(body_of, ..)| {
                            body_of.client.is_some()
                                && match body_of.room {
                                    Room::Town(_) => true,
                                    Room::Arena(_) => arenas
                                        .of(body_of.room, entity)
                                        .is_some_and(|(game, seat)| game.over() || game.out[seat]),
                                    Room::Home(_) | Room::Game(_) => false,
                                }
                        })
                    });
                    match entity {
                        Some(entity) if free => going.push(entity),
                        _ => staying.push(id),
                    }
                }
                // The one who asked is first of them, or it is nobody's to
                // go.
                if going.first() != Some(&body) {
                    tell(&mut tells, client, Tell::Refused(Refusal::Busy));
                    continue;
                }
                for id in staying {
                    social.left_behind(id);
                }
                seat_together(
                    &mut commands,
                    &mut tells,
                    &mut bodies,
                    &mut arenas,
                    &mut towns,
                    config.arena_old_bots,
                    &going,
                    now,
                );
            }
            Ask::Leave | Ask::Travel(Venue::Town) if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                game.leave(seat);
                let Ok((mut body_of, spot, venue, _)) = bodies.get_mut(body) else {
                    continue;
                };
                let (channel, town_seat) = towns.sit(body, body_of.town);
                body_of.town = Some(channel);
                move_to(
                    &mut commands,
                    &mut tells,
                    body,
                    (body_of, spot, venue),
                    Room::Town(channel),
                    Venue::Town,
                    town_seat,
                    now,
                );
            }
            Ask::Punch { who, running } if matches!(room, Room::Arena(_)) => {
                let Some((game, hitter)) = arenas.of(room, body) else {
                    continue;
                };
                if !game.playing() || !game.alive(hitter) {
                    continue;
                }
                let Some(victim) = game.members.iter().position(|member| member.id == *who) else {
                    continue;
                };
                if !game.alive(victim) || arena_team(victim) == arena_team(hitter) {
                    continue;
                }
                let (Ok((_, from, ..)), Ok((_, at, ..))) = (bodies.get(game.seats[hitter]), bodies.get(game.seats[victim]))
                else {
                    continue;
                };
                let (from, at) = (*from, *at);
                if from.translation().xz().distance(at.translation().xz()) > PUNCH_SLACK {
                    continue;
                }
                // Knocked from where they stand now, as their body says,
                // and told so once the frame's fight is done ([`flush`]).
                if !game.bot[victim] {
                    game.at[victim] = at.translation();
                }
                game.wound(victim, hitter, from, *running);
            }
            Ask::HitAnchor { team } if matches!(room, Room::Arena(_)) => {
                let Some((game, hitter)) = arenas.of(room, body) else {
                    continue;
                };
                let Ok((_, from, ..)) = bodies.get(game.seats[hitter]) else {
                    continue;
                };
                game.strike_anchor(hitter, usize::from(*team), from.translation().xz(), PUNCH_SLACK);
            }
            Ask::Rung {
                chain,
                index,
                x,
                z,
                heading,
            } if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                let sane = [*x, *z, *heading].iter().all(|n| n.is_finite()) && x.abs() < FURTHEST && z.abs() < FURTHEST;
                if !game.playing() || !sane || game.rungs[seat] >= MOST_RUNGS {
                    continue;
                }
                let plank = commands
                    .spawn((
                        Replicated,
                        InRoom(game.room()),
                        Plank {
                            seat: seat as u8,
                            chain: *chain,
                            index: *index,
                            x: *x,
                            z: *z,
                            heading: *heading,
                        },
                    ))
                    .id();
                game.planks.push(plank);
                game.rungs[seat] += 1;
                game.nav.hear(seat as u8, *chain, *index, Vec2::new(*x, *z));
            }
            // Fallen into the void: dead, and back if the anchor stands.
            Ask::Respawn if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                if game.playing() && game.alive(seat) {
                    info!("Bridge Wars game {}: seat {seat} fell", game.id);
                    game.die(seat);
                }
            }
            Ask::Buy(ware) if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                let Ok((_, at, ..)) = bodies.get(game.seats[seat]) else {
                    continue;
                };
                if game.buy(seat, *ware, at.translation().xz()) {
                    info!("Bridge Wars game {}: seat {seat} bought {ware:?}", game.id);
                    tell(&mut tells, client, Tell::Bought(*ware));
                }
            }
            Ask::Use(used) if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                if !game.playing() || !game.alive(seat) {
                    continue;
                }
                match used {
                    Use::Fireball { heading } => {
                        let Ok((_, at, ..)) = bodies.get(game.seats[seat]) else {
                            continue;
                        };
                        game.throw(seat, at.translation(), *heading);
                    }
                    Use::Builder => {
                        game.outfits.spend(seat, Utility::Builder);
                    }
                    Use::Dynamite { heading, far } => {
                        let Ok((_, at, ..)) = bodies.get(game.seats[seat]) else {
                            continue;
                        };
                        game.lob(seat, at.translation(), *heading, *far);
                    }
                    Use::Potion(potion) => {
                        let drunk = game.outfits.drink(seat, *potion, game.elapsed);
                        if drunk == Some(Utility::Healing) {
                            game.health[seat] = (game.health[seat] + HEALTH * HEALING).min(HEALTH);
                        }
                        debug!("Bridge Wars game {}: seat {seat} drank {drunk:?}", game.id);
                    }
                }
            }
            Ask::Skip if matches!(room, Room::Arena(_)) => {
                if let Some((game, _)) = arenas.of(room, body)
                    && config.allow_skip
                {
                    match game.stage {
                        ArenaStage::Waiting => game.since_join = BOT_WAIT,
                        _ => game.left = 0.0,
                    }
                }
            }
            _ => {}
        }
    }
}

/// Moves every game on: fills its room, counts down, sends everyone to their
/// islands, runs the fight ([`Game::step`]) and does in the world what the
/// game says is to be done there ([`flush`]), and ends it once the result
/// has been up its while, or once nobody is left.
#[allow(clippy::too_many_arguments)]
fn run(
    time: Res<Time>,
    config: Res<Config>,
    mut arenas: ResMut<Arenas>,
    mut bodies: Bodies,
    mut everyone: ResMut<Everyone>,
    mut social: ResMut<Social>,
    mut towns: ResMut<Towns>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs_f64();
    let mut ended = Vec::new();
    for game in arenas.games.values_mut() {
        let room = game.room();
        let watched = (0..game.seats.len()).any(|seat| {
            !game.gone[seat]
                && !game.bot[seat]
                && bodies
                    .get(game.seats[seat])
                    .is_ok_and(|(body, ..)| body.client.is_some() && body.room == room)
        });
        game.unwatched = if watched { 0.0 } else { game.unwatched + dt };
        let everyone_gone = (0..game.seats.len()).all(|seat| game.gone[seat] || game.bot[seat]);
        if game.closing {
            end(&mut commands, &mut tells, game, &mut bodies, &mut everyone, &mut towns, now);
            ended.push(game.id);
            continue;
        }
        if (game.over() && game.left <= 0.0) || everyone_gone || game.unwatched > ABANDONED {
            close(&mut commands, game);
            continue;
        }

        sync(game, &bodies);
        match game.stage {
            ArenaStage::Waiting => {
                game.since_join += dt;
                let waited = game.since_join >= BOT_WAIT && game.free_seat().is_some();
                if waited {
                    // The computer takes every seat left, however many
                    // people there are: every game is two to a team on
                    // four islands, the people teamed up with each other
                    // first ([`SEAT_ORDER`]).
                    let people = game.people();
                    while game.free_seat().is_some() {
                        seat_bot(&mut commands, game, now, false);
                    }
                    info!(
                        "Bridge Wars game {}: nobody joined in {BOT_WAIT} s: {people} people, and the computer in the seats left",
                        game.id
                    );
                }
                if waited || game.free_seat().is_none() {
                    game.pad();
                    game.stage = ArenaStage::Countdown;
                    game.left = COUNTDOWN;
                }
            }
            ArenaStage::Countdown => {
                game.left -= dt;
                if game.left <= 0.0 {
                    // Whoever left during the countdown has the computer
                    // in their seat, on its island from the start.
                    while game.free_seat().is_some() {
                        seat_bot(&mut commands, game, now, true);
                    }
                    for seat in 0..game.seats.len() {
                        game.stand(seat);
                    }
                    game.stage = ArenaStage::Playing;
                    // From the start, or, for trying the timetable out,
                    // from as far into it as the server was told to.
                    game.elapsed = config.arena_from.clamp(0.0, PLAY_SECS - 1.0);
                    game.left = PLAY_SECS - game.elapsed;
                    let dice = &mut game.dice;
                    game.mint = Mint::new(game.elapsed, &mut || dice.f32());
                    // Nothing in anyone's purse, but for trying the shops
                    // out (`RT_ARENA_PURSE`).
                    if config.arena_purse > 0 {
                        for seat in 0..game.purses.len().min(ARENA_SEATS) {
                            if !game.bot[seat] {
                                game.purses[seat] = [config.arena_purse; 4];
                            }
                        }
                    }
                    info!("Bridge Wars game {}: fight, {} s to play", game.id, game.left);
                    // Who the computer's players are this time, and how
                    // their teams mean to open: for reading a game back by.
                    let cast: Vec<String> = (0..game.seats.len().min(ARENA_SEATS))
                        .filter_map(|seat| {
                            game.squad
                                .bot(seat)
                                .map(|bot| format!("{} {} is {}", seat, game.members[seat].name, bot.persona().name))
                        })
                        .collect();
                    let openings: Vec<String> = (0..ARENA_TEAMS).map(|team| format!("{:?}", game.squad.plan(team).opening)).collect();
                    info!("Bridge Wars game {}: {}; the teams open {}", game.id, cast.join(", "), openings.join(", "));
                    // Begun late, everything before then has happened.
                    let before: Vec<Happening> = happening_between(-1.0, game.elapsed).collect();
                    for event in before {
                        game.happen(event);
                    }
                }
            }
            ArenaStage::Playing => game.step(dt),
            ArenaStage::Over { .. } => game.left -= dt,
        }
        flush(&mut commands, &mut tells, &mut bodies, game, now);

        // Whoever of the people in it played it to its end has earned
        // toward their level (`social`, since 2026-11-13): more, on the
        // team that won. Once a game.
        if let ArenaStage::Over { winner } = game.stage
            && social.first_time(Minigame::BridgeWars, game.id)
        {
            for seat in 0..game.seats.len().min(ARENA_SEATS) {
                if game.bot[seat] || game.vacant[seat] || game.gone[seat] {
                    continue;
                }
                let won = winner == Some(arena_team(seat));
                social.award(game.members[seat].id, XP_PLAYED + if won { XP_WON } else { 0 });
            }
        }

        // Only what has changed goes out again.
        let round = game.round();
        if game.sent.as_ref() != Some(&round) {
            commands.entity(game.shown).insert(round.clone());
            game.sent = Some(round);
        }
        if game.sent_seats != game.members {
            commands.entity(game.shown).insert(Seats(game.members.clone()));
            game.sent_seats.clone_from(&game.members);
        }
        if game.sent_purses != game.purses {
            commands.entity(game.shown).insert(Purses(game.purses.clone()));
            game.sent_purses.clone_from(&game.purses);
        }
        if game.sent_kits != game.outfits.kits {
            commands.entity(game.shown).insert(Kits(game.outfits.kits.clone()));
            game.sent_kits.clone_from(&game.outfits.kits);
        }
        if game.sent_upgrades != game.outfits.teams {
            commands.entity(game.shown).insert(Upgraded(game.outfits.teams));
            game.sent_upgrades = game.outfits.teams;
        }
    }
    for id in ended {
        arenas.games.remove(&id);
        if arenas.filling == Some(id) {
            arenas.filling = None;
        }
    }
}

/// Copies into `game` where each person in it last said they stood, and
/// whether their body is in its room to be played with ([`Game::at`],
/// [`Game::present`]): what a step of the fight goes by. The computer's
/// players are where the game has them.
fn sync(game: &mut Game, bodies: &Bodies) {
    let room = game.room();
    for seat in 0..game.seats.len() {
        if game.bot[seat] || game.vacant[seat] {
            continue;
        }
        match bodies.get(game.seats[seat]) {
            Ok((body, spot, ..)) => {
                game.present[seat] = body.room == room && spot.aboard.is_none();
                game.at[seat] = spot.translation();
                game.yaw[seat] = yaw_of(spot.rotation());
            }
            Err(_) => game.present[seat] = false,
        }
    }
}

/// Does in the world what `game` has said is to be done there
/// ([`Game::events`]): the room is sent what the generators have made and
/// the fireballs thrown, and no longer what has been taken or has burst;
/// whoever was knocked is told; whoever stands again is sent to their
/// island; a zombie risen has its body. And each of the computer's players
/// is sent where the game has moved it, if it has.
fn flush(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    game: &mut Game,
    now: f64,
) {
    let room = game.room();
    for event in std::mem::take(&mut game.events) {
        match event {
            Event::Made(piece) => {
                let entity = commands
                    .spawn((
                        Replicated,
                        InRoom(room),
                        Loot {
                            item: piece.item,
                            x: piece.at.x,
                            z: piece.at.y,
                        },
                    ))
                    .id();
                game.loot.insert(piece.id, entity);
            }
            Event::Taken(piece) => {
                if let Some(entity) = game.loot.remove(&piece) {
                    commands.entity(entity).try_despawn();
                }
            }
            Event::Thrown { ball, shown } => {
                let entity = commands.spawn((Replicated, InRoom(room), shown)).id();
                game.shown_balls.insert(ball, entity);
            }
            Event::Burst(ball) => {
                if let Some(entity) = game.shown_balls.remove(&ball) {
                    commands.entity(entity).try_despawn();
                }
            }
            Event::Lobbed { stick, shown } => {
                let entity = commands.spawn((Replicated, InRoom(room), shown)).id();
                game.shown_sticks.insert(stick, entity);
            }
            Event::Blew { stick, shown } => {
                if let Some(entity) = game.shown_sticks.remove(&stick) {
                    commands.entity(entity).try_despawn();
                }
                // Kept to the end of the game with the rungs: which planks
                // are gone is part of every bridge from now on.
                let entity = commands.spawn((Replicated, InRoom(room), shown)).id();
                game.planks.push(entity);
            }
            Event::Fell(stick) => {
                if let Some(entity) = game.shown_sticks.remove(&stick) {
                    commands.entity(entity).try_despawn();
                }
            }
            Event::Knocked { seat, by, from, running } => {
                if let Ok((body, ..)) = bodies.get(game.seats[seat])
                    && let Some(client) = body.client
                {
                    tell(tells, client, Tell::Knocked { by, from, running });
                }
            }
            Event::Land(seat) => land(commands, tells, bodies, game, seat, now),
            Event::Rung(plank) => {
                let entity = commands.spawn((Replicated, InRoom(room), plank)).id();
                game.planks.push(entity);
            }
            Event::Zombie(seat) => {
                let body = commands
                    .spawn((
                        Replicated,
                        game.members[seat].clone(),
                        Spot::of(game.at[seat], Quat::from_rotation_y(game.yaw[seat])),
                        Venue::Arena,
                        InRoom(room),
                        Body::bot(room, seat as u8, now),
                    ))
                    .id();
                game.seats[seat] = body;
            }
        }
    }
    for seat in 0..game.seats.len() {
        if !game.bot[seat] {
            continue;
        }
        let Ok((_, mut spot, ..)) = bodies.get_mut(game.seats[seat]) else {
            continue;
        };
        let mut next = Spot::of(game.at[seat], Quat::from_rotation_y(game.yaw[seat]));
        // Come to a stop, it is sent once more as standing still, so that
        // nobody watching carries it on the way it was going while they
        // wait for an update that is not coming ([`Spot::still`]).
        next.still = (next.x, next.y, next.z) == (spot.x, spot.y, spot.z);
        if !next.same(&spot) {
            *spot = next;
        }
    }
}

/// What the bot in `seat` does this frame: it is moved ([`Game::at`]), and
/// whom it punches, if anyone, is given back. It keeps to the disc of its
/// island, having no map: knocked off it, it falls.
fn think(game: &mut Game, seat: usize, standing: &[(usize, Vec2)], dt: f32) -> Option<usize> {
    let (island, reach) = game.home(seat);
    let edge = reach - BODY_RADIUS;
    // A zombie comes on slower than anyone runs, and never runs away.
    let zombie = game.zombie(seat).is_some();
    let pace = if zombie { ZOMBIE_SPEED } else { RUN_SPEED };
    let beaten = !zombie && game.health[seat] / HEALTH < FLEE_BELOW;
    let mut at = game.at[seat];
    let mind = &mut game.minds[seat];
    mind.cooldown = (mind.cooldown - dt).max(0.0);

    // Knocked off the island, it falls.
    if let Some(speed) = mind.falling {
        let speed = speed - GRAVITY * dt;
        mind.falling = Some(speed);
        at.y += speed * dt;
        if mind.shove != Vec2::ZERO {
            at += Vec3::new(mind.shove.x, 0.0, mind.shove.y) * dt;
        }
        game.at[seat] = at;
        return None;
    }

    // A punch throws it up and carries it, dying away as it goes, slowly
    // in the air and quickly once it is down, off the edge if it must.
    let airborne = mind.lift > 0.0 || at.y > ISLAND_TOP + 1e-3;
    if airborne {
        at.y += mind.lift * dt;
        mind.lift -= if mind.lift > 0.0 { RISE_GRAVITY } else { GRAVITY } * dt;
        if at.y <= ISLAND_TOP {
            at.y = ISLAND_TOP;
            mind.lift = 0.0;
        }
    }
    if mind.shove != Vec2::ZERO {
        at += Vec3::new(mind.shove.x, 0.0, mind.shove.y) * dt;
        let drag = if airborne { AIR_DRAG } else { GROUND_DRAG };
        mind.shove *= drag.powf(dt * 20.0);
        if mind.shove.length_squared() < 0.01 {
            mind.shove = Vec2::ZERO;
        }
        if at.xz().distance(island) > reach + 0.2 {
            mind.falling = Some(mind.lift.min(0.0));
            game.at[seat] = at;
            return None;
        }
    }
    if airborne {
        // Knocked into the air, it is carried, and does nothing else.
        game.at[seat] = at;
        return None;
    }

    let here = at.xz();
    let team = arena_team(seat);
    // Whoever of another team is nearest within sight, on or beside its
    // island, where it can get at them.
    let target = standing
        .iter()
        .filter(|&&(other, there)| {
            other != seat
                && arena_team(other) != team
                && there.distance(here) <= HUNT_RADIUS
                && there.distance(island) <= reach + 1.5
        })
        .min_by(|a, b| a.1.distance_squared(here).total_cmp(&b.1.distance_squared(here)))
        .copied();
    let mut step = Vec2::ZERO;
    let mut facing = Vec2::ZERO;
    let mut punch = None;
    match target {
        Some((other, there)) => {
            mind.goal = None;
            let off = there - here;
            let toward = off.normalize_or_zero();
            if beaten {
                // Nearly beaten, it runs for it: away, or round the edge.
                let away = -toward;
                let ahead = here + away * 3.0;
                step = if ahead.distance(island) <= edge {
                    away
                } else {
                    let round = Vec2::new(-away.y, away.x);
                    if (here + round * 3.0).distance(island) <= edge { round } else { -round }
                } * RUN_SPEED * dt;
                facing = step;
            } else {
                facing = toward;
                if off.length() > REACH * PRESS_IN {
                    step = toward * pace * dt;
                }
                if off.length() <= REACH + BODY_RADIUS && mind.cooldown <= 0.0 {
                    mind.cooldown = BOT_PUNCH_EVERY;
                    punch = Some(other);
                }
            }
        }
        None => {
            // Nobody to go for: a stroll somewhere on the island now and
            // then.
            mind.think -= dt;
            if mind.think <= 0.0 {
                mind.think += THINK_EVERY;
                let angle = game.dice.f32() * std::f32::consts::TAU;
                let out = game.dice.f32() * (edge - 1.5);
                mind.goal = Some(island + Vec2::new(angle.cos(), angle.sin()) * out);
            }
            if let Some(goal) = mind.goal {
                let off = goal - here;
                if off.length() < 0.2 {
                    mind.goal = None;
                } else {
                    step = off.normalize() * (STROLL * dt).min(off.length());
                    facing = off;
                }
            }
        }
    }
    let mut next = here + step;
    if next.distance(island) > edge {
        next = island + (next - island).normalize_or(Vec2::X) * edge;
    }
    if facing.length_squared() > 1e-6 {
        game.yaw[seat] = (-facing.x).atan2(-facing.y);
    }
    game.at[seat] = Vec3::new(next.x, ISLAND_TOP, next.y);
    punch
}

/// Sends the person in `seat` to where they arrive on their island,
/// through their device.
fn land(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    game: &Game,
    seat: usize,
    now: f64,
) {
    let room = game.room();
    let Ok((body_of, spot, venue, _)) = bodies.get_mut(game.seats[seat]) else {
        return;
    };
    if body_of.room != room {
        return;
    }
    move_to(commands, tells, game.seats[seat], (body_of, spot, venue), room, Venue::Arena, seat as u8, now);
}

/// The computer, in the next free seat of `game` ([`Game::free_seat`]),
/// standing in the waiting room, or, `landed`, where its seat arrives on
/// its island, the fight being about to begin: a boy or a girl by the
/// toss of a coin, in any of that side's colours, under one of that
/// side's names that nobody there has, as House Builder's are
/// (`builder::seat_bot`).
fn seat_bot(commands: &mut Commands, game: &mut Game, now: f64, landed: bool) {
    let Some(seat) = game.free_seat() else {
        return;
    };
    let member = game.bot_member(seat);
    let spot = if landed {
        game.bot_spot(seat)
    } else {
        let (at, turn) = waiting_spot(seat);
        Spot::of(at, turn)
    };
    let body = commands
        .spawn((
            Replicated,
            member.clone(),
            spot,
            Venue::Arena,
            InRoom(game.room()),
            Body::bot(game.room(), seat as u8, now),
        ))
        .id();
    game.put(seat, body, member, true);
}

/// Who the computer's player in `seat` of the game numbered `game` is to
/// everyone: clear of people's ids, which count up from 1, of House
/// Builder's bots, which count down from the top, and of every other
/// game's, a game having no more than [`MOST_SEATS`] seats, its zombies
/// counted.
fn bot_id(game: u32, seat: usize) -> u64 {
    u64::MAX / 2 - u64::from(game) * MOST_SEATS - seat as u64
}

/// The most seats a game has: its players' and its zombies', and room.
const MOST_SEATS: u64 = 32;

/// The first tick of a game's end: its planks, its computer players and
/// its board go, while everyone in it is still in its room to be told so.
/// Told the tick after, as [`end`] moves them out, they would never be: an
/// entity despawned in the very tick a device stops seeing its room is
/// written to that device neither as despawned (it is hidden by then) nor
/// as lost from sight (it is gone by then), read in replicon 0.44's
/// `collect_despawns`; and the bots stood on in the next game, their
/// bodies never taken down (seen 2026-11-09).
fn close(commands: &mut Commands, game: &mut Game) {
    for (seat, &body) in game.seats.iter().enumerate() {
        if game.bot[seat] && body != Entity::PLACEHOLDER {
            commands.entity(body).try_despawn();
        }
    }
    for plank in game.planks.drain(..) {
        commands.entity(plank).try_despawn();
    }
    for (_, thing) in game.loot.drain() {
        commands.entity(thing).try_despawn();
    }
    for (_, ball) in game.shown_balls.drain() {
        commands.entity(ball).try_despawn();
    }
    for (_, stick) in game.shown_sticks.drain() {
        commands.entity(stick).try_despawn();
    }
    // Whatever the game had still to say goes unsaid: there is no room
    // left to be shown it.
    game.balls.clear();
    game.sticks.clear();
    game.events.clear();
    commands.entity(game.shown).try_despawn();
    game.closing = true;
}

/// The end of a game, the tick after [`close`]: anyone still in it is sent
/// back to the town, or, if they are not connected, forgotten.
fn end(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    game: &mut Game,
    bodies: &mut Bodies,
    everyone: &mut Everyone,
    towns: &mut Towns,
    now: f64,
) {
    let room = game.room();
    for (seat, &body) in game.seats.iter().enumerate() {
        if game.bot[seat] {
            continue;
        }
        let Ok((mut body_of, spot, venue, _)) = bodies.get_mut(body) else {
            continue;
        };
        if body_of.room != room {
            continue;
        }
        if body_of.client.is_none() {
            everyone.0.remove(&body_of.who);
            commands.entity(body).despawn();
            continue;
        }
        let (channel, town_seat) = towns.sit(body, body_of.town);
        body_of.town = Some(channel);
        move_to(
            commands,
            tells,
            body,
            (body_of, spot, venue),
            Room::Town(channel),
            Venue::Town,
            town_seat,
            now,
        );
    }
    for plank in game.planks.drain(..) {
        commands.entity(plank).try_despawn();
    }
    info!("Bridge Wars game {} closed", game.id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_game() -> Game {
        let mut game = Game::new(0, Entity::PLACEHOLDER, 7);
        game.stage = ArenaStage::Playing;
        game.left = PLAY_SECS;
        game.mint = Mint::new(0.0, &mut || 0.5);
        for seat in 0..ARENA_SEATS {
            let member = Member {
                id: seat as u64 + 1,
                name: format!("Seat {seat}"),
                look: "boy".into(),
                ai: seat > 0,
            };
            game.put(seat, Entity::from_bits(seat as u64 + 1), member, seat > 0);
        }
        game
    }

    /// A waiting room with nobody in it yet.
    fn empty_room() -> Game {
        let mut game = full_game();
        game.stage = ArenaStage::Waiting;
        for list in [&mut game.bot, &mut game.gone, &mut game.vacant, &mut game.out] {
            list.clear();
        }
        game.seats.clear();
        game.members.clear();
        game.minds.clear();
        game.health.clear();
        game.since_hit.clear();
        game.hurt.clear();
        game.dead_for.clear();
        game.rungs.clear();
        game.purses.clear();
        game.at.clear();
        game.yaw.clear();
        game.present.clear();
        game.outfits = Outfits::default();
        game
    }

    fn person(n: u64) -> Member {
        Member {
            id: n,
            name: format!("Person {n}"),
            look: "boy".into(),
            ai: false,
        }
    }

    /// The computer in every seat of `game` nobody is in, as the wait's end
    /// puts it there, without the entities.
    fn fill_with_bots(game: &mut Game) {
        while let Some(seat) = game.free_seat() {
            let member = game.bot_member(seat);
            game.put(seat, Entity::from_bits(100 + seat as u64), member, true);
        }
        game.pad();
    }

    #[test]
    fn people_are_teamed_up_first_and_the_computer_takes_every_seat_left() {
        // Two people are a team, and four are two teams on opposite
        // islands (Hajun, 2026-11-12: "the 2 player become same team. and
        // if its 4 players, then those 2 players and 2 players are same
        // team too, and so on").
        assert_eq!(arena_team(SEAT_ORDER[0]), arena_team(SEAT_ORDER[1]));
        assert_eq!(arena_team(SEAT_ORDER[2]), arena_team(SEAT_ORDER[3]));
        assert_eq!(ARENA_ISLANDS[arena_island(SEAT_ORDER[0])], -ARENA_ISLANDS[arena_island(SEAT_ORDER[2])]);
        let mut order = SEAT_ORDER;
        order.sort_unstable();
        assert_eq!(order, [0, 1, 2, 3, 4, 5, 6, 7], "every seat is taken once");
        // However many come, each team is filled before the next is
        // begun, and the computer has every seat left once the wait is
        // over: all eight are taken, every team is two, and nobody is out.
        for people in 1..=ARENA_SEATS {
            let mut game = empty_room();
            for n in 0..people {
                let seat = game.seat(Entity::from_bits(n as u64 + 1), person(n as u64 + 1), false);
                assert_eq!(seat, SEAT_ORDER[n], "{people} people: the {n}th to come");
            }
            assert_eq!(game.people(), people);
            let teams: Vec<u8> = (0..people).map(|n| arena_team(SEAT_ORDER[n])).collect();
            for pair in teams.chunks_exact(2) {
                assert_eq!(pair[0], pair[1], "{people} people: teams {teams:?}");
            }
            fill_with_bots(&mut game);
            assert_eq!(game.seats.len(), ARENA_SEATS);
            assert!(game.vacant.iter().all(|vacant| !vacant), "{people} people: a seat left empty");
            assert_eq!(game.free_seat(), None);
            assert_eq!(game.people(), people);
            assert_eq!(game.bot.iter().filter(|bot| **bot).count(), ARENA_SEATS - people);
            assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
            assert_eq!(game.round().out, 0);
            assert!((0..ARENA_SEATS).all(|seat| game.health[seat] == HEALTH && !game.gone[seat]));
            // Where a person and the computer are a team, the person has
            // the lower seat, and so is the first to be given what lies
            // on their pad.
            for team in 0..ARENA_TEAMS {
                let (first, second) = (team, team + ARENA_TEAMS);
                assert!(!game.bot[first] || game.bot[second], "{people} people: team {team}");
            }
            // And no two of the computer's players go by one name.
            let mut names: Vec<&str> = game.members.iter().map(|member| member.name.as_str()).collect();
            names.sort_unstable();
            names.dedup();
            assert_eq!(names.len(), ARENA_SEATS, "{people} people: {names:?}");
        }
        // The fight on, whoever is seated is past the players, as a
        // zombie is.
        let mut game = empty_room();
        game.seat(Entity::from_bits(1), person(1), false);
        fill_with_bots(&mut game);
        game.stage = ArenaStage::Playing;
        let zombie = game.seat(Entity::from_bits(99), person(99), true);
        assert_eq!(zombie, ARENA_SEATS);
        assert_eq!(game.zombie(zombie), Some(0));
    }

    #[test]
    fn a_group_goes_in_two_by_two_onto_one_team_and_an_odd_one_onto_another() {
        // Each team's two seats are together in the order seats are taken,
        // the lower first: what going in two by two goes by.
        for pair in SEAT_ORDER.chunks(2) {
            assert_eq!(arena_team(pair[0]), arena_team(pair[1]), "{pair:?}");
            assert!(pair[0] < pair[1], "{pair:?}");
        }
        let teams = |seats: &[usize]| -> Vec<u8> { seats.iter().map(|&seat| arena_team(seat)).collect() };
        // Into an empty room: one where one alone sits; two on one team;
        // three, two on one team and the third on another; four, two
        // teams, on opposite islands.
        let room = empty_room();
        assert_eq!(room.seats_for(1), Some(vec![SEAT_ORDER[0]]));
        assert_eq!(room.seats_for(1).map(|seats| seats[0]), room.free_seat());
        assert_eq!(room.seats_for(2), Some(vec![0, 4]));
        assert_eq!(room.seats_for(3), Some(vec![0, 4, 2]));
        assert_eq!(room.seats_for(4), Some(vec![0, 4, 2, 6]));
        assert_eq!(teams(&room.seats_for(4).unwrap()), vec![0, 0, 2, 2]);
        assert_eq!(ARENA_ISLANDS[arena_island(0)], -ARENA_ISLANDS[arena_island(2)]);
        // With one person in it already, alone on their team: two go onto
        // the next team whole, and never beside the stranger; three, two
        // onto a team of their own and the third beside the stranger.
        let mut room = empty_room();
        room.seat(Entity::from_bits(1), person(1), false);
        assert_eq!(room.seats_for(2), Some(vec![2, 6]));
        assert_eq!(room.seats_for(3), Some(vec![2, 6, 4]));
        assert_eq!(room.seats_for(4), Some(vec![2, 6, 1, 5]));
        // Seated so, each pair is one team, and the next to come alone
        // sits where there is room.
        for (n, seat) in room.seats_for(4).unwrap().into_iter().enumerate() {
            room.put(seat, Entity::from_bits(10 + n as u64), person(10 + n as u64), false);
        }
        assert_eq!(room.people(), 5);
        assert_eq!(arena_team(room.seat_of(Entity::from_bits(10)).unwrap()), arena_team(room.seat_of(Entity::from_bits(11)).unwrap()));
        assert_eq!(arena_team(room.seat_of(Entity::from_bits(12)).unwrap()), arena_team(room.seat_of(Entity::from_bits(13)).unwrap()));
        assert_eq!(room.seats_for(1), Some(vec![4]));
        // One team is left whole: two more fit, four do not, nor three
        // once the stranger's teammate has come.
        assert_eq!(room.seats_for(2), Some(vec![3, 7]));
        assert_eq!(room.seats_for(3), Some(vec![3, 7, 4]));
        assert_eq!(room.seats_for(4), None);
        room.seat(Entity::from_bits(2), person(2), false);
        assert_eq!(room.seats_for(3), None);
        assert_eq!(room.seats_for(2), Some(vec![3, 7]));
        // Full, or no longer waiting: nobody.
        fill_with_bots(&mut room);
        assert_eq!(room.seats_for(1), None);
        let mut begun = empty_room();
        begun.stage = ArenaStage::Countdown;
        assert_eq!(begun.seats_for(2), None);
        // The computer still takes every seat a group left.
        let mut room = empty_room();
        for (n, seat) in room.seats_for(3).unwrap().into_iter().enumerate() {
            room.put(seat, Entity::from_bits(1 + n as u64), person(1 + n as u64), false);
        }
        fill_with_bots(&mut room);
        assert_eq!(room.people(), 3);
        assert_eq!(room.bot.iter().filter(|bot| **bot).count(), ARENA_SEATS - 3);
        assert!(room.vacant.iter().all(|vacant| !vacant));
        assert_eq!(room.teams_in_play(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn a_seat_left_before_the_fight_is_anyone_s_again_and_one_left_after_stays_empty() {
        let mut game = empty_room();
        for n in 1..=3 {
            game.seat(Entity::from_bits(n), person(n), false);
        }
        // Three people, in seats 0, 4 and 2. The second leaves the
        // waiting room: their seat is empty, and the next to come has it.
        assert_eq!(game.people(), 3);
        game.leave(4);
        assert!(game.vacant[4] && game.gone[4] && game.out[4] && game.members[4].id == 0);
        assert_eq!(game.seats[4], Entity::PLACEHOLDER);
        assert_eq!(game.people(), 2);
        assert_eq!(game.free_seat(), Some(4));
        assert_eq!(game.seat(Entity::from_bits(9), person(9), false), 4);
        assert_eq!((game.people(), game.members[4].id), (3, 9));
        assert!(!game.vacant[4] && !game.gone[4] && !game.out[4] && game.health[4] == HEALTH);
        // Through the countdown the same, and the computer takes it as
        // the fight begins.
        game.stage = ArenaStage::Countdown;
        game.leave(2);
        assert_eq!(game.free_seat(), Some(2));
        fill_with_bots(&mut game);
        assert!(game.bot[2] && !game.vacant[2]);
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
        // The fight on, a seat left stays its own, empty: nobody takes it,
        // and its team is one short.
        game.stage = ArenaStage::Playing;
        game.leave(0);
        assert!(game.gone[0] && !game.vacant[0] && !game.in_play(0));
        assert_eq!(game.free_seat(), None);
        assert_eq!(game.people(), 1);
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3], "seat 4 still plays for team 0");
        // With everyone gone from an empty waiting room, nobody is left.
        let mut game = empty_room();
        game.seat(Entity::from_bits(1), person(1), false);
        game.leave(0);
        assert!((0..game.seats.len()).all(|seat| game.gone[seat] || game.bot[seat]));
        assert_eq!(game.free_seat(), Some(0));
    }

    #[test]
    fn dead_with_the_anchor_standing_comes_back_and_without_it_is_out() {
        let mut game = full_game();
        game.die(1);
        assert_eq!(game.health[1], 0.0);
        assert_eq!(game.dead_for[1], Some(RESPAWN_SECS));
        assert!(!game.out[1] && game.in_play(1) && !game.alive(1));
        assert_eq!(game.round().anchors[1], ANCHOR_HEALTH as u16);
        game.anchors[usize::from(arena_team(2))] = 0.0;
        game.die(2);
        assert!(game.out[2] && !game.in_play(2));
        assert_eq!(game.dead_for[2], None);
        // Teams 0 to 3 are all still in play through their other seat.
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
        for seat in [0, 4, 1, 5, 3, 7] {
            game.anchors[usize::from(arena_team(seat))] = 0.0;
            game.die(seat);
        }
        // Team 2 is left: its seat 6 stands and its seat 2 is out.
        assert_eq!(game.teams_in_play(), vec![2]);
        let round = game.round();
        assert!(round.is_out(2) && round.is_out(0) && !round.is_out(6));
        assert_eq!(round.health[6], HEALTH as u16);
        assert_eq!(round.health[0], 0);
        assert_eq!(round.anchors, [0; ARENA_TEAMS]);
    }

    #[test]
    fn a_bot_keeps_to_its_island_runs_at_an_enemy_and_falls_when_knocked_off() {
        // The computer's players as they were until 2026-11-12, which the
        // server can be switched back to (`RT_ARENA_BOTS=old`), and as
        // the zombies still are.
        let mut game = full_game();
        game.old_bots = true;
        let seat = 5;
        let island = ARENA_ISLANDS[arena_island(seat)];
        game.stand(seat);
        assert_eq!(game.at[seat].y, ISLAND_TOP);
        // Alone, it strolls, and never off the island.
        for _ in 0..600 {
            let here = game.at[seat].xz();
            let punch = think(&mut game, seat, &[(seat, here)], 1.0 / 60.0);
            assert_eq!(punch, None);
            assert!(game.at[seat].xz().distance(island) <= ARENA_ISLAND_RADIUS - BODY_RADIUS + 1e-3);
            assert_eq!(game.at[seat].y, ISLAND_TOP);
        }
        // An enemy 6 m off on its island: it runs at them and, in reach,
        // punches, no oftener than it may.
        let enemy = island + Vec2::new(6.0, 0.0);
        let mut punches = 0;
        for _ in 0..600 {
            let standing = [(seat, game.at[seat].xz()), (0, enemy)];
            punches += usize::from(think(&mut game, seat, &standing, 1.0 / 60.0).is_some());
        }
        assert!(game.at[seat].xz().distance(enemy) <= REACH + BODY_RADIUS + 0.1, "{}", game.at[seat]);
        assert!(punches >= 8 && punches <= 14, "{punches} punches in ten seconds");
        // A punch that lands takes its damage off a bot and knocks it,
        // away from whoever threw it and up; a body reeling from one
        // takes no other for a while, however many land.
        let mut hit = full_game();
        hit.old_bots = true;
        hit.stand(1);
        let from = Spot::of(hit.at[1] + Vec3::X * 2.0, Quat::IDENTITY);
        hit.wound(1, 0, from, true);
        assert_eq!(hit.health[1], HEALTH - DAMAGE);
        assert!(hit.minds[1].shove.x < -(KNOCK_BACK + SPRINT_BONUS) + 0.01 && hit.minds[1].lift == KNOCK_UP);
        hit.wound(1, 0, from, true);
        assert_eq!(hit.health[1], HEALTH - DAMAGE, "the second, while it reels, takes nothing");
        hit.hurt[1] = 0.0;
        hit.wound(1, 0, from, false);
        assert_eq!(hit.health[1], HEALTH - 2.0 * DAMAGE);
        assert!(hit.events.is_empty(), "a bot is knocked here, with nobody to tell");
        // A person hit is knocked by their own device, which is told who
        // by and from where.
        hit.wound(0, 1, from, false);
        assert_eq!(hit.health[0], HEALTH - DAMAGE);
        match hit.events.as_slice() {
            [Event::Knocked { seat: 0, by, running: false, .. }] => assert_eq!(*by, hit.members[1].id),
            other => panic!("{other:?}"),
        }
        // A teammate right beside it is nobody to go for: never punched.
        let mut idle = full_game();
        idle.old_bots = true;
        idle.stand(seat);
        for _ in 0..600 {
            let here = idle.at[seat].xz();
            let punch = think(&mut idle, seat, &[(seat, here), (1, here + Vec2::new(1.0, 0.0))], 1.0 / 60.0);
            assert_eq!(punch, None, "punched a teammate");
        }
        // Knocked hard toward the edge, up and across as a punch knocks,
        // it goes over and falls.
        game.minds[seat].shove = Vec2::new(KNOCK_BACK + SPRINT_BONUS, 0.0) * 2.0;
        game.minds[seat].lift = KNOCK_UP;
        let mut fell = false;
        for _ in 0..240 {
            let standing = [(seat, game.at[seat].xz()), (0, enemy)];
            think(&mut game, seat, &standing, 1.0 / 60.0);
            if game.minds[seat].falling.is_some() && game.at[seat].y < ISLAND_TOP - 1.0 {
                fell = true;
                break;
            }
        }
        assert!(fell, "still on the island at {}", game.at[seat]);
    }

    #[test]

    fn the_room_is_shown_only_what_has_changed() {
        let game = full_game();
        let round = game.round();
        assert_eq!(round.stage, ArenaStage::Playing);
        assert_eq!(round.left, PLAY_SECS as u16);
        assert_eq!(round.health.len(), ARENA_SEATS);
        assert_eq!(round, game.round());
    }

    #[test]
    fn what_a_generator_makes_is_whoever_comes_near_its_and_nobody_else_s() {
        use roundtown_net::{ARENA_PADS, ARENA_SHOPS, Item, PAD_TOP};
        let mut game = full_game();
        // Twenty seconds and a bit of every generator: ten silver and two
        // gold on each team's pad.
        let (dt, mut dice) = (0.05, || 0.5);
        for _ in 0..410 {
            game.elapsed += dt;
            game.mint.run(game.elapsed, &[0; ARENA_TEAMS], &mut dice);
        }
        // The tenth silver of each pad has only just been made. A fifth
        // of a second after, still dropping, it is out of reach over the
        // head of someone standing on the pad, who has the other eleven;
        // someone at the top of a jump a moment later has it too, in the
        // air, with what lies under their feet.
        let pad = ARENA_PADS[0];
        let on_pad = Vec3::new(pad.x, PAD_TOP, pad.y);
        let last = game.mint.pieces().iter().map(|piece| piece.born).fold(0.0, f32::max);
        assert!(game.elapsed - last < DROP_SECS + 0.1, "the tenth silver was made {last} s in");
        assert_eq!(game.mint.clone().pick_up(on_pad, last + 0.2).len(), 11, "the one still dropping is left");
        assert_eq!(game.mint.clone().pick_up(on_pad + Vec3::Y * 1.6, last + 0.36).len(), 12);
        // A second on, with nothing more made, all of them lie there.
        game.elapsed += 1.0;
        let lying = game.mint.pieces().len();
        assert_eq!(lying, 4 * (10 + 2));
        // The computer's player in seat 4, on the pad, picks up nothing;
        // nor does the person in seat 0 while dead, or high over it.
        assert!(game.gather(4, on_pad).is_empty());
        game.health[0] = 0.0;
        assert!(game.gather(0, on_pad).is_empty());
        game.health[0] = HEALTH;
        assert!(game.gather(0, on_pad + Vec3::Y * (WAITING_FLOOR - PAD_TOP)).is_empty());
        assert_eq!(game.mint.pieces().len(), lying);
        // Standing on it, they have all of it, and it is gone: whoever
        // comes after finds nothing there.
        let taken = game.gather(0, on_pad);
        assert_eq!(taken.len(), 12);
        assert_eq!(game.purses[0], [10, 2, 0, 0]);
        assert_eq!(game.mint.pieces().len(), lying - 12);
        game.bot[4] = false;
        assert!(game.gather(4, on_pad).is_empty());
        assert_eq!(game.purses[4], [0; 4]);
        // The shop sells a bridge for 4 silver to someone standing at it,
        // as often as they can pay, and not from the pad.
        let shop = ARENA_SHOPS[0];
        assert!(!game.buy(0, Ware::Bridge, pad));
        assert!(game.buy(0, Ware::Bridge, shop));
        assert!(game.buy(0, Ware::Bridge, shop));
        assert_eq!(game.purses[0][Item::Silver.index()], 2);
        assert!(!game.buy(0, Ware::Bridge, shop), "with 2 silver");
        assert_eq!(game.purses[0], [2, 2, 0, 0]);
        // Nor to the dead, nor to someone with nothing.
        assert!(!game.buy(4, Ware::Bridge, ARENA_SHOPS[0]));
        game.purses[0][0] = 4;
        game.health[0] = 0.0;
        assert!(!game.buy(0, Ware::Bridge, shop));
    }

    #[test]
    fn the_two_shops_sell_their_own_wares_and_dying_loses_the_sword() {
        use roundtown_net::{ARENA_SHOPS, ARENA_UPGRADE_SHOPS};
        let mut game = full_game();
        game.purses[0] = [60, 20, 20, 10];
        let (items, upgrades) = (ARENA_SHOPS[0], ARENA_UPGRADE_SHOPS[0]);
        // A sword at the item shop, not at the upgrade shop; an upgrade
        // the other way round.
        assert!(!game.buy(0, Ware::Sword(1), upgrades));
        assert!(game.buy(0, Ware::Sword(1), items));
        assert!(!game.buy(0, Ware::Forge, items));
        assert!(game.buy(0, Ware::Forge, upgrades));
        assert_eq!(game.purses[0], [50, 20, 18, 10]);
        assert_eq!((game.outfits.kits[0].sword, game.outfits.teams[0].forge), (1, 1));
        // Never the sword in hand again, nor a lesser one; a better one,
        // a shield, a fireball for the second button, and a potion beside
        // it, on the next (since 2026-11-13: the one button there was
        // took no potion with a fireball on it).
        assert!(!game.buy(0, Ware::Sword(1), items));
        assert!(game.buy(0, Ware::Sword(3), items) && game.buy(0, Ware::Shield(2), items));
        assert!(game.buy(0, Ware::Utility(Utility::Fireball), items));
        assert!(game.buy(0, Ware::Utility(Utility::Speed), items));
        assert_eq!(game.purses[0], [10, 8, 18, 5]);
        let kit = game.outfits.kit(0);
        assert_eq!((kit.slot_of(Utility::Fireball), kit.slot_of(Utility::Speed)), (Some(0), Some(1)));
        // The team's upgrade is its other seat's too: seat 4 swings a
        // stronger sword once it has one, and seat 0's punch takes what a
        // sword's does, less what a shield keeps off.
        assert!(game.buy(0, Ware::StrongerSwords, upgrades));
        assert_eq!(game.outfits.team(4), game.outfits.team(0));
        assert_eq!(game.outfits.punch(DAMAGE, 0, 1), DAMAGE + 20.0 + 5.0);
        assert_eq!(game.outfits.punch(DAMAGE, 4, 1), DAMAGE);
        assert!((game.outfits.punch(DAMAGE, 1, 0) - DAMAGE * 0.86).abs() < 1e-4);
        // Dead, the sword, the fireball and the potion are gone, and everything held
        // of what the generators make; the shield and the team's upgrades
        // are not, nor anyone else's purse.
        game.purses[4] = [3, 2, 1, 0];
        assert_ne!(game.purses[0], [0; 4]);
        game.die(0);
        assert_eq!(game.outfits.kits[0], Kit { shield: 2, ..Kit::default() });
        assert!(game.outfits.teams[0].swords && game.outfits.teams[0].forge == 1);
        assert_eq!((game.purses[0], game.purses[4]), ([0; 4], [3, 2, 1, 0]));
        // A potion drunk heals at once, or lasts its while.
        game.health[0] = 40.0;
        game.dead_for[0] = None;
        game.purses[0] = [0, 6, 0, 1];
        assert!(game.buy(0, Ware::Utility(Utility::Healing), items));
        assert_eq!(game.outfits.drink(0, Utility::Healing, game.elapsed), Some(Utility::Healing));
        assert!(game.buy(0, Ware::Utility(Utility::Healing), items), "the button is the healing's until it is ticked empty");
        // Unseen, seat 0 is seen again the moment it is hit.
        game.outfits.kits[0] = Kit { unseen_until: 600, ..Kit::default() };
        assert!(game.outfits.kit(0).unseen(game.elapsed));
        assert!(game.outfits.reveal(0) && !game.outfits.kit(0).unseen(game.elapsed));
    }

    #[test]
    fn the_zombies_sit_past_the_players_keep_to_their_ground_and_always_come_back() {
        let mut game = full_game();
        // Seated as sudden death seats them: a seat each past the
        // players', standing where it rises, and a body asked of the
        // world for each.
        game.happen(Happening::SuddenDeath);
        for zombie in 0..ZOMBIES {
            let seat = ARENA_SEATS + zombie;
            assert_eq!(game.zombie(seat), Some(zombie));
            assert_eq!(arena_team(seat), ZOMBIE_TEAM);
            assert_eq!(game.home(seat), zombie_home(zombie));
            assert!(game.at[seat].xz().distance(zombie_spot(zombie)) < 0.01);
            assert_eq!(game.at[seat].y, ISLAND_TOP);
            assert!(game.bot[seat] && game.alive(seat) && game.members[seat].name == ZOMBIE_NAME);
            assert_eq!(game.members[seat].id, bot_id(game.id, seat));
        }
        let risen: Vec<usize> = game
            .events
            .iter()
            .filter_map(|event| match event {
                Event::Zombie(seat) => Some(*seat),
                _ => None,
            })
            .collect();
        assert_eq!(risen, (ARENA_SEATS..ARENA_SEATS + ZOMBIES).collect::<Vec<_>>());
        assert_eq!(game.seats.len(), ARENA_SEATS + ZOMBIES);
        assert_eq!(game.purses.len(), game.seats.len());
        // Every id in the game its own, and none another game's.
        let mut ids: Vec<u64> = (0..game.seats.len()).map(|seat| bot_id(game.id, seat)).collect();
        ids.extend((0..game.seats.len()).map(|seat| bot_id(game.id + 1, seat)));
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 2 * game.seats.len());
        // They are nobody's team: the four teams are still what is in
        // play, and with every player out, nobody is.
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
        // A zombie put down is back after its while, whatever the anchors.
        game.anchors = [0.0; ARENA_TEAMS];
        let zombie = ARENA_SEATS + 5;
        game.die(zombie);
        assert_eq!(game.dead_for[zombie], Some(ZOMBIE_RESPAWN_SECS));
        assert!(!game.out[zombie] && game.in_play(zombie) && !game.alive(zombie));
        for seat in 0..ARENA_SEATS {
            game.die(seat);
        }
        assert!(game.teams_in_play().is_empty());
        let round = game.round();
        assert_eq!(round.out, 0xff);
        assert_eq!(round.health.len(), ARENA_SEATS + ZOMBIES);
        // The one on team 0's island goes for the player there, slower
        // than anyone runs, punches in reach, and never leaves the island.
        let mut game = full_game();
        game.happen(Happening::SuddenDeath);
        let seat = ARENA_SEATS;
        let (island, reach) = game.home(seat);
        let prey = island + Vec2::new(6.0, 3.0);
        let from = game.at[seat].xz();
        let standing = [(seat, from), (0, prey)];
        think(&mut game, seat, &standing, 1.0 / 60.0);
        // A sixtieth of a second's worth at its pace: well short of a run.
        let step = game.at[seat].xz().distance(from);
        assert!((step - ZOMBIE_SPEED / 60.0).abs() < 1e-3, "a step of {step} m");
        let mut punches = 0;
        for _ in 0..600 {
            punches += usize::from(think(&mut game, seat, &standing, 1.0 / 60.0) == Some(0));
            assert!(game.at[seat].xz().distance(island) <= reach - BODY_RADIUS + 1e-3);
        }
        assert!(punches >= 5, "{punches} punches in ten seconds");
        // Nearly beaten, it still comes on.
        game.health[seat] = HEALTH * FLEE_BELOW * 0.5;
        game.at[seat] = Vec3::new(island.x, ISLAND_TOP, island.y);
        let far = [(seat, island), (0, island + Vec2::new(5.0, 0.0))];
        think(&mut game, seat, &far, 0.1);
        assert!(game.at[seat].x > island.x, "it ran away");
    }

    #[test]
    fn at_ten_minutes_every_anchor_breaks_and_whoever_dies_after_is_out() {
        let mut game = full_game();
        game.anchors[2] = 0.0;
        game.happen(Happening::Tier {
            item: roundtown_net::Item::Sapphire,
            tier: 2,
        });
        assert_eq!(game.anchors, [ANCHOR_HEALTH, ANCHOR_HEALTH, 0.0, ANCHOR_HEALTH]);
        game.happen(Happening::AnchorsBreak);
        assert_eq!(game.anchors, [0.0; ARENA_TEAMS]);
        game.die(1);
        assert!(game.out[1]);
    }

    #[test]
    fn an_anchor_takes_a_punch_from_another_team_within_reach_and_no_oftener_than_a_body() {
        use roundtown_net::ANCHOR_RADIUS;
        let mut game = full_game();
        let anchor = ARENA_ANCHORS[1];
        let beside = anchor + Vec2::new(1.5, 0.0);
        // Not one's own, nor from out of reach, nor a team there is none
        // of, nor by the dead.
        assert!(!game.strike_anchor(1, 1, beside, PUNCH_SLACK));
        assert!(!game.strike_anchor(5, 1, beside, PUNCH_SLACK));
        assert!(!game.strike_anchor(0, 1, anchor + Vec2::new(PUNCH_SLACK + ANCHOR_RADIUS + 0.5, 0.0), PUNCH_SLACK));
        assert!(!game.strike_anchor(0, 9, beside, PUNCH_SLACK));
        game.health[2] = 0.0;
        assert!(!game.strike_anchor(2, 1, beside, PUNCH_SLACK));
        assert_eq!(game.anchors[1], ANCHOR_HEALTH);
        // A punch's damage off it, and nothing while it reels from one.
        assert!(game.strike_anchor(0, 1, beside, PUNCH_SLACK));
        assert_eq!(game.anchors[1], ANCHOR_HEALTH - DAMAGE);
        assert!(!game.strike_anchor(0, 1, beside, PUNCH_SLACK));
        assert_eq!(game.anchors[1], ANCHOR_HEALTH - DAMAGE);
        game.anchor_hurt[1] = 0.0;
        // A sword breaks it sooner, as it does a body; and whoever was
        // unseen is seen again for hitting it.
        game.outfits.kits[0] = Kit {
            sword: 2,
            unseen_until: 900,
            ..Kit::default()
        };
        assert!(game.strike_anchor(0, 1, beside, PUNCH_SLACK));
        assert_eq!(game.anchors[1], ANCHOR_HEALTH - 2.0 * DAMAGE - SWORD_DAMAGE[2]);
        assert!(!game.outfits.kit(0).unseen(game.elapsed));
        // Broken, it takes no more.
        game.anchors[1] = 0.0;
        game.anchor_hurt[1] = 0.0;
        assert!(!game.strike_anchor(0, 1, beside, PUNCH_SLACK));
    }

    #[test]
    fn a_fireball_thrown_flies_and_bursts_on_whoever_it_meets_and_the_room_is_shown_both() {
        let mut game = full_game();
        // Seat 0 on its island with one fireball; seat 1, a bot of
        // another team, 6 m ahead of it the way it throws.
        game.outfits.kits[0] = Kit::default().holding(Utility::Fireball, 1);
        let from = Vec3::new(0.0, ISLAND_TOP, 130.0);
        game.at[0] = from;
        game.at[1] = from + Vec3::new(0.0, 0.0, -6.0);
        assert!(game.throw(0, from, 0.0));
        assert!(!game.throw(0, from, 0.0), "with none left");
        assert!(!game.throw(1, game.at[1], 0.0), "with none at all");
        assert!(matches!(game.events.as_slice(), [Event::Thrown { ball: 0, shown }] if shown.seat == 0));
        game.events.clear();
        let mut frames = 0;
        while !game.balls.is_empty() {
            game.fireballs(1.0 / 60.0);
            frames += 1;
            assert!(frames < 120, "still flying");
        }
        // It met them well short of its range, took its damage and
        // knocked them on the way it flew; a teammate of the thrower's
        // beside them would have taken nothing.
        assert!(frames < 20, "{frames} frames");
        assert_eq!(game.health[1], HEALTH - FIREBALL_DAMAGE);
        let knocked = game.squad.bot(1).expect("the computer's").body;
        assert!(knocked.shove.y < -1.0 && knocked.lift > 0.0, "{knocked:?}");
        assert!(matches!(game.events.as_slice(), [Event::Burst(0)]));
        assert_eq!(game.health[4], HEALTH);
    }

    #[test]
    fn a_stick_of_dynamite_comes_down_blows_hurts_other_teams_and_takes_the_planks_under_it() {
        use roundtown_net::nav::{PLANK_TOP, Region};
        let mut game = full_game();
        // A bridge from seat 0's island to the middle, its teammate's.
        let span = game.nav.grid().span(Region::Team(0), Region::Middle, 0.0).expect("a span");
        for index in 0..=span.planks() {
            game.nav.hear(4, 0, index as u16, span.rung(index));
        }
        assert!(game.nav.joined(Region::Team(0), Region::Middle));
        // Seat 0 on the island at the bridge's end, with two sticks; the
        // bridge runs north from there. Seat 1, a bot of another team
        // with the best shield there is, stands on it 8 m out, and seat
        // 4, of the thrower's own team, beside it; seat 2 well off.
        game.outfits.kits[0] = Kit::default().holding(Utility::Dynamite, 2);
        game.outfits.kits[1].shield = 3;
        let from = Vec3::new(span.from.x, ISLAND_TOP, span.from.y + 1.0);
        let down = Vec3::new(from.x, PLANK_TOP, from.z - 8.0);
        game.at[0] = from;
        game.at[2] = down + Vec3::new(0.0, 0.0, -12.0);
        for seat in [1, 4] {
            game.at[seat] = down + Vec3::new(0.3, 0.0, -1.0);
            game.squad.stand(seat, game.at[seat], 0.0, 0.0);
        }
        assert!(!game.lob(1, game.at[1], 0.0, 8.0), "with none at all");
        assert!(game.lob(0, from, 0.0, 8.0));
        let [Event::Lobbed { stick: 0, shown }] = game.events.as_slice() else {
            panic!("the room is not shown it: {:?}", game.events);
        };
        assert_eq!(shown.seat, 0);
        assert!(Vec3::new(shown.to_x, shown.to_y, shown.to_z).distance(down) < 1e-3, "{shown:?}");
        assert!(Vec3::new(shown.x, shown.y, shown.z).distance(from + Vec3::new(0.0, DYNAMITE_HAND, -DYNAMITE_AHEAD)) < 1e-3);
        game.events.clear();
        assert_eq!(game.outfits.kit(0).held(Utility::Dynamite), 1);
        // It is in the air for most of a second, lies for its fuse, and
        // blows: nothing happens to anyone until then.
        let mut seconds = 0.0;
        while !game.sticks.is_empty() {
            assert_eq!((game.health[1], game.blasts), (HEALTH, 0));
            assert!(game.events.is_empty() && game.nav.joined(Region::Team(0), Region::Middle));
            game.dynamites(1.0 / 60.0);
            seconds += 1.0 / 60.0;
            assert!(seconds < 4.0, "it never blew");
        }
        assert!((seconds - (0.45 + 0.04 * 8.0 + DYNAMITE_FUSE)).abs() < 0.05, "{seconds} s");
        // Its damage off the one of another team within its reach, less
        // what their shield keeps off, and nobody knocked; nothing off the
        // thrower's teammate beside them, nor off anyone further away.
        assert!((game.health[1] - (HEALTH - DYNAMITE_DAMAGE * (1.0 - SHIELD_SHARE[3]))).abs() < 1e-3, "{}", game.health[1]);
        assert_eq!((game.health[4], game.health[2], game.health[0]), (HEALTH, HEALTH, HEALTH));
        let body = game.squad.bot(1).expect("the computer's").body;
        assert_eq!((body.shove, body.lift), (Vec2::ZERO, 0.0), "a stick of dynamite knocks nobody");
        // The planks whose middles it reached are gone, one after another
        // of that one bridge, and the room is told which, with where it
        // blew; the island is parted from the middle again.
        let [Event::Blew { stick: 0, shown }] = game.events.as_slice() else {
            panic!("the room is not shown the blast: {:?}", game.events);
        };
        assert!(Vec3::new(shown.x, shown.y, shown.z).distance(down) < 1e-3);
        assert!((4..=5).contains(&shown.planks.len()), "{:?}", shown.planks);
        assert!(shown.planks.iter().all(|&(seat, chain, _)| (seat, chain) == (4, 0)));
        assert!(shown.planks.windows(2).all(|pair| pair[1].2 == pair[0].2 + 1));
        assert_eq!((game.blasts, game.planks_blown), (1, shown.planks.len() as u32));
        assert!(!game.nav.joined(Region::Team(0), Region::Middle));
        assert_eq!(game.nav.ground(down.xz()), None);
        game.events.clear();
        // Whoever of the computer's stood on them falls with them, and is
        // dead of it soon enough.
        for _ in 0..150 {
            game.step(1.0 / 60.0);
        }
        assert!(!game.alive(1) && !game.alive(4), "they stand on nothing still: {} {}", game.at[1], game.at[4]);
        game.events.clear();
        // One thrown where there is nothing to lie on is gone into the
        // void as it comes down, and nothing blows.
        let west = std::f32::consts::FRAC_PI_2;
        assert!(game.lob(0, from, west, 9.0));
        game.events.clear();
        while !game.sticks.is_empty() {
            game.dynamites(1.0 / 60.0);
        }
        assert!(matches!(game.events.as_slice(), [Event::Fell(1)]), "{:?}", game.events);
        assert_eq!((game.blasts, game.outfits.kit(0).held(Utility::Dynamite)), (1, 0));
        assert!(!game.lob(0, from, 0.0, 5.0), "with none left");
        // And one thrown onto the land blows there and takes no plank.
        game.outfits.kits[0] = Kit::default().holding(Utility::Dynamite, 1);
        assert!(game.lob(0, from, std::f32::consts::PI, 5.0));
        game.events.clear();
        while !game.sticks.is_empty() {
            game.dynamites(1.0 / 60.0);
        }
        assert!(matches!(game.events.as_slice(), [Event::Blew { stick: 2, shown }] if shown.planks.is_empty()), "{:?}", game.events);
    }

    /// Plays `game` on by itself for `secs` seconds, or until it is over,
    /// a thirtieth of a second at a time, with nobody to show or tell
    /// what it says is to be.
    fn play(game: &mut Game, secs: f32) {
        let dt = 1.0 / 30.0;
        let until = game.elapsed + secs;
        while game.playing() && game.elapsed < until {
            game.step(dt);
            game.events.clear();
        }
    }

    /// A game of eight of the computer's players, on their islands, the
    /// fight just begun: those that play the game, or, `old`, those kept
    /// to their islands.
    fn bots_alone(seed: u64, old: bool) -> Game {
        let mut game = Game::new(3, Entity::PLACEHOLDER, seed);
        game.old_bots = old;
        fill_with_bots(&mut game);
        for seat in 0..ARENA_SEATS {
            game.stand(seat);
        }
        game.stage = ArenaStage::Playing;
        game.left = PLAY_SECS;
        game
    }

    #[test]
    fn a_whole_game_of_the_bots_as_they_were_plays_itself_out() {
        let mut game = bots_alone(11, true);
        assert_eq!((game.people(), game.seats.len()), (0, ARENA_SEATS));
        // Just short of ten minutes: every anchor stands, and everyone is
        // where they can be.
        play(&mut game, 599.0);
        assert!(game.playing());
        assert_eq!(game.anchors, [ANCHOR_HEALTH; ARENA_TEAMS]);
        assert!(game.at.iter().all(|at| at.is_finite()));
        assert!(!game.mint.pieces().is_empty());
        // Past ten the anchors have broken, and past fifteen the zombies
        // have come.
        play(&mut game, 310.0);
        assert_eq!(game.anchors, [0.0; ARENA_TEAMS]);
        assert_eq!(game.seats.len(), ARENA_SEATS + ZOMBIES);
        // And it is over by its time at the latest, with a winner or a
        // draw.
        play(&mut game, PLAY_SECS);
        assert!(game.over(), "still {:?} at {} s", game.stage, game.elapsed);
        assert!(game.elapsed <= PLAY_SECS + 0.1);
        assert!(game.at.iter().all(|at| at.is_finite()));
        // The same dice, the same game.
        let (mut one, mut other) = (bots_alone(5, true), bots_alone(5, true));
        play(&mut one, 60.0);
        play(&mut other, 60.0);
        assert_eq!(one.at, other.at);
        assert_eq!(one.mint.pieces(), other.mint.pieces());
    }

    /// What everyone in `game` is up to, for looking at one played in a
    /// test.
    fn report(game: &Game) -> String {
        use roundtown_net::nav::Region;
        let joined: Vec<String> = Region::LANDS
            .iter()
            .filter(|&&land| land != Region::Middle && game.nav.joined(land, Region::Middle))
            .map(|land| format!("{land:?}"))
            .collect();
        let mut lines = vec![format!(
            "t={:>4.0} anchors {:?} joined to the middle: {}",
            game.elapsed,
            game.anchors.map(|anchor| anchor as i32),
            joined.join(" ")
        )];
        for seat in 0..game.seats.len().min(ARENA_SEATS) {
            let Some(bot) = game.squad.bot(seat) else {
                continue;
            };
            let (at, kit) = (game.at[seat], game.outfits.kit(seat));
            let state = if game.out[seat] {
                "OUT "
            } else if game.alive(seat) {
                "    "
            } else {
                "dead"
            };
            lines.push(format!(
                "  seat {seat} team {} {:<6} hp {:>3.0} {state} ({:>6.1},{:>5.1},{:>6.1}) {:<14} purse {:?} sword {} shield {} buttons {} {:>3.0}m  {:?}",
                arena_team(seat),
                format!("{:?}", bot.role()),
                game.health[seat],
                at.x,
                at.y,
                at.z,
                game.nav.grid().region(at.xz()).map_or(String::from("-"), |land| format!("{land:?}")),
                game.purses[seat],
                kit.sword,
                kit.shield,
                kit.slots
                    .iter()
                    .flatten()
                    .map(|stack| format!("{:?}x{}", stack.utility, stack.count))
                    .collect::<Vec<_>>()
                    .join(" "),
                bot.metres(),
                bot.goal(),
            ));
        }
        let teams: Vec<String> = game
            .outfits
            .teams
            .iter()
            .map(|team| format!("forge {} swords {} shields {} pool {} traps {}", team.forge, team.swords, team.shields, team.heal_pool, team.traps))
            .collect();
        lines.push(format!("  upgrades: {}", teams.join(" | ")));
        lines.join("\n")
    }

    /// Not a test of anything: a game of the computer's players alone,
    /// told as it goes, for looking at what they do. `cargo test -p
    /// roundtown_server watch -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn watch_a_game_of_bots() {
        let seed = std::env::var("RT_SEED").ok().and_then(|seed| seed.parse().ok()).unwrap_or(1);
        let brief = std::env::var_os("RT_BRIEF").is_some();
        let mut game = bots_alone(seed, false);
        let mut next = 0.0;
        let mut alive = [true; ARENA_SEATS];
        let mut anchors = game.anchors;
        let mut goals = [roundtown_net::bots::Goal::Idle; ARENA_SEATS];
        while game.playing() {
            if game.elapsed >= next && !brief {
                println!("{}", report(&game));
                next += if game.elapsed < 300.0 { 20.0 } else { 60.0 };
            }
            let was = game.at.clone();
            game.step(1.0 / 30.0);
            game.events.clear();
            for seat in 0..ARENA_SEATS {
                let bot = game.squad.bot(seat).expect("the computer's");
                if alive[seat] && !game.alive(seat) {
                    let how = if game.at[seat].y < -5.0 { "FELL" } else { "KILLED" };
                    println!(
                        "    {:>4.0} s: seat {seat} (team {}) {how} from ({:.0},{:.0}) {:?} on a plank {} doing {:?}{}",
                        game.elapsed,
                        arena_team(seat),
                        was[seat].x,
                        was[seat].z,
                        game.nav.grid().region(was[seat].xz()),
                        game.nav.on_plank(was[seat].xz()),
                        goals[seat],
                        if game.out[seat] { ", and is OUT" } else { "" },
                    );
                }
                alive[seat] = game.alive(seat);
                if game.alive(seat) {
                    if goals[seat] != bot.goal() && brief && matches!(bot.goal(), roundtown_net::bots::Goal::Raid(_) | roundtown_net::bots::Goal::Hunt | roundtown_net::bots::Goal::Defend) {
                        println!("    {:>4.0} s: seat {seat} (team {}) -> {:?}", game.elapsed, arena_team(seat), bot.goal());
                    }
                    goals[seat] = bot.goal();
                }
            }
            for team in 0..ARENA_TEAMS {
                if anchors[team] > 0.0 && game.anchors[team] <= 0.0 {
                    println!("    {:>4.0} s: team {team}'s anchor is gone", game.elapsed);
                }
            }
            anchors = game.anchors;
        }
        println!("{}", report(&game));
        println!("over: {:?} at {:.0} s", game.stage, game.elapsed);
    }

    /// What happened in a game of the computer's players alone, for
    /// telling whether they play it sensibly.
    #[derive(Debug)]
    struct Chronicle {
        seed: u64,
        /// When each team's island was joined to the middle, and when the
        /// first of the smaller islands was.
        joined: [Option<f32>; ARENA_TEAMS],
        sapphire: Option<f32>,
        /// When the first team upgrade was bought, and the first sword.
        upgrade: Option<f32>,
        sword: Option<f32>,
        /// When each anchor went, and when each team was out.
        anchor: [Option<f32>; ARENA_TEAMS],
        out: [Option<f32>; ARENA_TEAMS],
        /// Deaths before the anchors break, and after: killed, and fallen.
        killed: (u32, u32),
        fell: (u32, u32),
        /// Whoever fell with nobody having hit them in the three seconds
        /// before: walked off, which must never be.
        walked_off: u32,
        /// The most seconds any one of them stood with nothing to do
        /// while standing, before the anchors broke.
        idlest: f32,
        /// How it ended.
        over: f32,
        winner: Option<u8>,
        /// Who each of the eight was, how each team opened, and how often
        /// each of them died.
        who: [&'static str; ARENA_SEATS],
        openings: [char; ARENA_TEAMS],
        deaths: [u32; ARENA_SEATS],
        /// How many of each kind of thing for a button they came by, all
        /// of them together, and how many they used (`bots::KINDS`: a
        /// fireball, a speed potion, an invisibility potion, a healing
        /// potion, a Bridge Builder, a stick of dynamite); and the most
        /// kinds any one of them had on its buttons at once.
        carried: [u32; 6],
        used: [u32; 6],
        fullest: usize,
        /// How many sticks of dynamite blew, and how many planks they
        /// took with them.
        blasts: u32,
        planks_blown: u32,
    }

    impl Default for Chronicle {
        fn default() -> Self {
            Self {
                seed: 0,
                joined: [None; ARENA_TEAMS],
                sapphire: None,
                upgrade: None,
                sword: None,
                anchor: [None; ARENA_TEAMS],
                out: [None; ARENA_TEAMS],
                killed: (0, 0),
                fell: (0, 0),
                walked_off: 0,
                idlest: 0.0,
                over: 0.0,
                winner: None,
                who: ["-"; ARENA_SEATS],
                openings: ['M'; ARENA_TEAMS],
                deaths: [0; ARENA_SEATS],
                carried: [0; 6],
                used: [0; 6],
                fullest: 0,
                blasts: 0,
                planks_blown: 0,
            }
        }
    }

    /// Plays a game of eight of the computer's players to its end, and
    /// says what happened in it.
    fn chronicle(seed: u64) -> Chronicle {
        use roundtown_net::bots::{Goal, Role};
        use roundtown_net::nav::Region;
        let mut game = bots_alone(seed, false);
        let mut told = Chronicle {
            seed,
            who: std::array::from_fn(|seat| game.squad.bot(seat).map_or("-", |bot| bot.persona().name)),
            openings: std::array::from_fn(|team| match game.squad.plan(team).opening {
                roundtown_net::bots::Opening::Middle => 'M',
                roundtown_net::bots::Opening::Sapphire => 'S',
                roundtown_net::bots::Opening::Late => 'L',
            }),
            ..Chronicle::default()
        };
        let dt = 1.0 / 30.0;
        let mut alive = [true; ARENA_SEATS];
        let mut idle = [0.0f32; ARENA_SEATS];
        let mut held = [[0u8; 6]; ARENA_SEATS];
        while game.playing() {
            game.step(dt);
            game.events.clear();
            let now = game.elapsed;
            let early = now < 600.0;
            for team in 0..ARENA_TEAMS {
                if told.joined[team].is_none() && game.nav.joined(Region::Team(team as u8), Region::Middle) {
                    told.joined[team] = Some(now);
                }
                if told.anchor[team].is_none() && game.anchors[team] <= 0.0 {
                    told.anchor[team] = Some(now);
                }
                if told.out[team].is_none() && !game.teams_in_play().contains(&(team as u8)) {
                    told.out[team] = Some(now);
                }
                let bought = game.outfits.teams[team];
                if told.upgrade.is_none() && (bought.forge > 0 || bought.heal_pool || bought.swords || bought.shields > 0 || bought.traps > 0) {
                    told.upgrade = Some(now);
                }
            }
            if told.sapphire.is_none() && (0..4).any(|island| game.nav.joined(Region::Sapphire(island), Region::Middle)) {
                told.sapphire = Some(now);
            }
            if told.sword.is_none() && game.outfits.kits.iter().any(|kit| kit.sword > 0) {
                told.sword = Some(now);
            }
            for seat in 0..ARENA_SEATS {
                let standing = game.alive(seat);
                // What it has on its buttons: more of a kind is bought,
                // and less of one, while it stands, is used.
                let kit = game.outfits.kit(seat);
                for (kind, utility) in roundtown_net::bots::KINDS.into_iter().enumerate() {
                    let (now_held, was) = (kit.held(utility), held[seat][kind]);
                    if now_held > was {
                        told.carried[kind] += u32::from(now_held - was);
                    } else if now_held < was && standing && alive[seat] {
                        told.used[kind] += u32::from(was - now_held);
                    }
                    held[seat][kind] = now_held;
                }
                told.fullest = told.fullest.max(kit.slots.iter().flatten().filter(|stack| stack.count > 0).count());
                if alive[seat] && !standing {
                    told.deaths[seat] += 1;
                    let fallen = game.at[seat].y < -5.0;
                    let count = if fallen { &mut told.fell } else { &mut told.killed };
                    if early {
                        count.0 += 1;
                    } else {
                        count.1 += 1;
                    }
                    if fallen && game.since_hit[seat] > 3.0 {
                        told.walked_off += 1;
                    }
                }
                alive[seat] = standing;
                let bot = game.squad.bot(seat).expect("the computer's");
                // The one that goes out with nothing to do, while it
                // stands, before the anchors break. The keeper's part is
                // to stand by its anchor.
                if standing && early && bot.role() == Role::Runner && bot.goal() == Goal::Idle {
                    idle[seat] += dt;
                    told.idlest = told.idlest.max(idle[seat]);
                } else {
                    idle[seat] = 0.0;
                }
            }
            assert!(game.at.iter().all(|at| at.is_finite()), "seed {seed}: somebody is nowhere at {now} s");
        }
        told.over = game.elapsed;
        told.winner = match game.stage {
            ArenaStage::Over { winner } => winner,
            _ => None,
        };
        told.blasts = game.blasts;
        told.planks_blown = game.planks_blown;
        told
    }

    /// The games of `seeds`, each told in a line.
    fn told(seeds: std::ops::Range<u64>) -> Vec<Chronicle> {
        let games: Vec<Chronicle> = seeds.map(chronicle).collect();
        let time = |at: Option<f32>| at.map_or(String::from("   -"), |at| format!("{at:>4.0}"));
        println!("seed | joined to the middle | sapph | sword | upgr | anchors gone        | out                 | killed fell (early) (late) | off | idle | over winner");
        for game in &games {
            println!(
                "{:>4} | {} {} {} {}  | {}  | {}  | {} | {} {} {} {} | {} {} {} {} | {:>2} {:>2}   {:>2} {:>2}            | {:>3} | {:>4.0} | {:>4.0} {:?}",
                game.seed,
                time(game.joined[0]),
                time(game.joined[1]),
                time(game.joined[2]),
                time(game.joined[3]),
                time(game.sapphire),
                time(game.sword),
                time(game.upgrade),
                time(game.anchor[0]),
                time(game.anchor[1]),
                time(game.anchor[2]),
                time(game.anchor[3]),
                time(game.out[0]),
                time(game.out[1]),
                time(game.out[2]),
                time(game.out[3]),
                game.killed.0,
                game.fell.0,
                game.killed.1,
                game.fell.1,
                game.walked_off,
                game.idlest,
                game.over,
                game.winner,
            );
            if std::env::var("RT_WHO").is_ok() {
                println!("       {} | opened {}", game.who.join(" "), game.openings.iter().collect::<String>());
            }
        }
        games
    }

    #[test]
    fn the_computer_s_players_play_whole_games_sensibly() {
        let games = told(1..9);
        let mut winners = [0u32; ARENA_TEAMS];
        for game in &games {
            let seed = game.seed;
            // Every team's island is joined to the middle before long, by
            // its own bridge or, a team that keeps to itself, by someone
            // else's; a smaller island is reached, arms are bought and
            // the teams' upgrades begun. Not all in the first half
            // minute, as it was when they all opened alike: some build
            // to a smaller island first, and some arm themselves first.
            for team in 0..ARENA_TEAMS {
                assert!(game.joined[team].is_some_and(|at| at < 330.0), "seed {seed}: team {team} joined at {:?}", game.joined[team]);
            }
            assert!(game.joined.iter().flatten().any(|at| *at < 60.0), "seed {seed}: nobody across in the first minute");
            assert!(game.sapphire.is_some_and(|at| at < 240.0), "seed {seed}: a smaller island at {:?}", game.sapphire);
            assert!(game.sword.is_some_and(|at| at < 120.0), "seed {seed}: a sword at {:?}", game.sword);
            assert!(game.upgrade.is_some_and(|at| at < 300.0), "seed {seed}: an upgrade at {:?}", game.upgrade);
            // Nobody walks off an edge, ever.
            assert_eq!(game.walked_off, 0, "seed {seed}");
            // No anchor goes before anchors are fair game, and nobody is
            // out before then.
            for team in 0..ARENA_TEAMS {
                assert!(game.anchor[team].is_none_or(|at| at >= 240.0), "seed {seed}: team {team}'s anchor at {:?}", game.anchor[team]);
                assert!(game.out[team].is_none_or(|at| at >= 240.0), "seed {seed}: team {team} out at {:?}", game.out[team]);
            }
            // It ends, somebody having won it, well before its time is up.
            assert!(game.over < 1200.0, "seed {seed}: over at {}", game.over);
            if let Some(winner) = game.winner {
                winners[usize::from(winner)] += 1;
            }
        }
        // And no island wins them all.
        println!("wins by team: {winners:?}");
        assert!(winners.iter().filter(|wins| **wins > 0).count() >= 3, "{winners:?}");
        // The same dice, the same game.
        let (one, other) = (chronicle(5), chronicle(5));
        assert_eq!((one.over, one.winner, one.killed, one.fell), (other.over, other.winner, other.killed, other.fell));
    }

    /// More of the same, for tuning by: `cargo test -p roundtown_server
    /// many -- --ignored --nocapture`, sixty games, or as many as
    /// `RT_GAMES` says.
    #[test]
    #[ignore]
    fn many_games_of_bots() {
        let count: u64 = std::env::var("RT_GAMES").ok().and_then(|count| count.parse().ok()).unwrap_or(60);
        let games = told(100..100 + count);
        let mean = |pick: &dyn Fn(&Chronicle) -> f32| games.iter().map(pick).sum::<f32>() / games.len() as f32;
        println!(
            "over at {:.0} s on average; killed early {:.1}, fell early {:.1}; killed late {:.1}, fell late {:.1}; anchors broken before ten minutes {:.1}",
            mean(&|game| game.over),
            mean(&|game| game.killed.0 as f32),
            mean(&|game| game.fell.0 as f32),
            mean(&|game| game.killed.1 as f32),
            mean(&|game| game.fell.1 as f32),
            mean(&|game| game.anchor.iter().filter(|at| at.is_some_and(|at| at < 599.0)).count() as f32),
        );
        // Who won, by island, and how many were drawn; and whose anchor
        // went first, and who was out first.
        let mut winners = [0u32; ARENA_TEAMS];
        let mut first_anchor = [0u32; ARENA_TEAMS];
        let mut first_out = [0u32; ARENA_TEAMS];
        let first = |times: &[Option<f32>; ARENA_TEAMS]| {
            (0..ARENA_TEAMS)
                .filter(|&team| times[team].is_some_and(|at| at < 599.0))
                .min_by(|&a, &b| times[a].unwrap_or(f32::MAX).total_cmp(&times[b].unwrap_or(f32::MAX)))
        };
        for game in &games {
            if let Some(team) = game.winner {
                winners[usize::from(team)] += 1;
            }
            if let Some(team) = first(&game.anchor) {
                first_anchor[team] += 1;
            }
            if let Some(team) = first(&game.out) {
                first_out[team] += 1;
            }
        }
        let drawn = games.iter().filter(|game| game.winner.is_none()).count();
        println!("of {} games: won by island {winners:?}, drawn {drawn}; first anchor broken before ten minutes {first_anchor:?}; first out before ten minutes {first_out:?}", games.len());
        // And by who they were: how many of each there were, how often
        // their team won, and how often each died in a game.
        let mut kinds: Vec<(&str, u32, u32, u32)> = Vec::new();
        let mut opened: [(u32, u32); 3] = [(0, 0); 3];
        for game in &games {
            for seat in 0..ARENA_SEATS {
                let name = game.who[seat];
                let won = game.winner == Some(arena_team(seat));
                match kinds.iter_mut().find(|kind| kind.0 == name) {
                    Some(kind) => {
                        kind.1 += 1;
                        kind.2 += u32::from(won);
                        kind.3 += game.deaths[seat];
                    }
                    None => kinds.push((name, 1, u32::from(won), game.deaths[seat])),
                }
            }
            for team in 0..ARENA_TEAMS {
                let which = "MSL".find(game.openings[team]).unwrap_or(0);
                opened[which].0 += 1;
                opened[which].1 += u32::from(game.winner == Some(team as u8));
            }
        }
        kinds.sort_by(|a, b| a.0.cmp(b.0));
        for (name, played, won, died) in kinds {
            println!("{name}: {played} played, their team won {:.0}%, died {:.1} times a game", 100.0 * won as f32 / played as f32, died as f32 / played as f32);
        }
        for (which, (teams, won)) in ["to the middle", "to a smaller island", "later"].into_iter().zip(opened) {
            println!("opened {which}: {teams} teams, won {:.0}%", 100.0 * won as f32 / teams.max(1) as f32);
        }
        // What they had on their buttons, all eight together, a game.
        let each = |pick: &dyn Fn(&Chronicle) -> [u32; 6]| {
            let mut sum = [0.0f32; 6];
            for game in &games {
                for (kind, count) in pick(game).into_iter().enumerate() {
                    sum[kind] += count as f32 / games.len() as f32;
                }
            }
            sum.map(|mean| format!("{mean:.1}")).join(", ")
        };
        println!(
            "a game, of fireballs, speed, invisibility and healing potions, Bridge Builders and sticks of dynamite: bought {}; used {}",
            each(&|game| game.carried),
            each(&|game| game.used)
        );
        println!(
            "a game: {:.1} sticks of dynamite blew, taking {:.1} planks; the most in one game {} sticks and {} planks",
            mean(&|game| game.blasts as f32),
            mean(&|game| game.planks_blown as f32),
            games.iter().map(|game| game.blasts).max().unwrap_or(0),
            games.iter().map(|game| game.planks_blown).max().unwrap_or(0),
        );
        let mut fullest = [0u32; 6];
        for game in &games {
            fullest[game.fullest.min(5)] += 1;
        }
        println!("the most kinds on one of them at once, by how many games (none to five): {fullest:?}");
    }

    /// A game with people in `people` seats who do nothing but stand where
    /// `stands` puts each, and the computer in the rest: the fight just
    /// begun.
    fn with_people(seed: u64, people: &[usize], stands: impl Fn(usize) -> Vec3) -> Game {
        let mut game = Game::new(4, Entity::PLACEHOLDER, seed);
        for &seat in people {
            game.put(seat, Entity::from_bits(seat as u64 + 1), person(seat as u64 + 1), false);
        }
        fill_with_bots(&mut game);
        for seat in 0..ARENA_SEATS {
            game.stand(seat);
        }
        for &seat in people {
            game.at[seat] = stands(seat);
        }
        game.events.clear();
        game.stage = ArenaStage::Playing;
        game.left = PLAY_SECS;
        game
    }

    /// Plays `game` on a thirtieth of a second, with the people in it put
    /// back where `stands` has them each time they stand again.
    fn step_with_people(game: &mut Game, stands: &impl Fn(usize) -> Vec3) {
        game.step(1.0 / 30.0);
        for event in std::mem::take(&mut game.events) {
            if let Event::Land(seat) = event {
                game.at[seat] = stands(seat);
            }
        }
    }

    #[test]
    fn a_team_of_people_is_raided_no_more_than_its_share_and_its_anchor_left_alone_at_first() {
        use roundtown_net::bots::Goal;
        // Two people on team 0 who only stand in the middle of their
        // island: the worst defenders there could be.
        let home = |seat: usize| {
            let (at, _) = arrival(Venue::Arena, seat);
            at.with_y(ISLAND_TOP)
        };
        let (mut on_people, mut on_bots) = (0u32, 0u32);
        for seed in [3, 8, 21] {
            let mut game = with_people(seed, &[0, 4], home);
            assert_eq!((game.people(), game.squad.bot(0).is_none(), game.squad.bot(1).is_some()), (2, true, true));
            let mut goals = [Goal::Idle; ARENA_SEATS];
            let mut first_raid_on_people = None;
            while game.playing() && game.elapsed < 600.0 {
                step_with_people(&mut game, &home);
                let now = game.elapsed;
                for seat in 0..ARENA_SEATS {
                    let Some(bot) = game.squad.bot(seat) else {
                        continue;
                    };
                    if let Goal::Raid(victim) = bot.goal()
                        && goals[seat] != bot.goal()
                    {
                        if victim == 0 {
                            on_people += 1;
                            first_raid_on_people.get_or_insert(now);
                        } else {
                            on_bots += 1;
                        }
                    }
                    goals[seat] = bot.goal();
                }
                // Nobody comes for them before raids begin, and their
                // anchor is not touched before anchors are fair game.
                if now < 150.0 {
                    assert!(first_raid_on_people.is_none(), "seed {seed}: raided at {now} s");
                }
                if now < 240.0 {
                    assert_eq!(game.anchors[0], ANCHOR_HEALTH, "seed {seed}: their anchor hit at {now} s");
                }
            }
            println!(
                "seed {seed}: the people first raided at {first_raid_on_people:?}; their anchor {} at the end, in play {:?}",
                game.anchors[0],
                game.teams_in_play()
            );
        }
        // Over the three games, the raids on them are no more than about
        // a team's share of all of them.
        println!("raids on the people {on_people}, on the computer's teams {on_bots}");
        assert!(on_people > 0, "nobody ever came for them");
        assert!(on_people as f32 <= 0.45 * (on_people + on_bots) as f32, "{on_people} of {} raids were on the people", on_people + on_bots);
    }

    #[test]
    fn a_person_s_bot_teammate_comes_to_the_pad_for_what_lies_there_and_is_off_again() {
        use roundtown_net::bots::{Goal, Role};
        use roundtown_net::nav::Region;
        use roundtown_net::{ARENA_PADS, PAD_TOP};
        // A person in seat 0 who stands on their pad for the first twenty
        // seconds and in the middle of their island after, never going
        // near the pad again; the computer in seat 4 beside them. Seven
        // minutes of it, three times over.
        let (pad, island) = (ARENA_PADS[0], ARENA_ISLANDS[0]);
        let dt = 1.0 / 30.0;
        let (mut went_out, mut went_raiding, mut fetched) = (0, 0, 0);
        for seed in [9, 2, 5] {
            let mut game = with_people(seed, &[0], |_| Vec3::new(pad.x, PAD_TOP, pad.y));
            let (mut armed, mut raided, mut fetching) = (false, false, false);
            let mut given = [0; 4];
            // How long it has been by the pad this time, the longest it
            // ever was, how long in all, and how many times it came; and
            // how long it was off its own island.
            let (mut stay, mut longest, mut by_the_pad, mut visits, mut away) = (0.0f32, 0.0f32, 0.0f32, 0u32, 0.0f32);
            let mut alive_for = 0.0f32;
            while game.playing() && game.elapsed < 420.0 {
                let now = game.elapsed;
                let stands = |_: usize| {
                    if now < 20.0 {
                        Vec3::new(pad.x, PAD_TOP, pad.y)
                    } else {
                        Vec3::new(0.0, ISLAND_TOP, 130.0)
                    }
                };
                game.at[0] = stands(0);
                step_with_people(&mut game, &stands);
                let bot = game.squad.bot(4).expect("the computer's");
                // Once it has thought at all, it is the one that keeps, as
                // far as who goes for whom is concerned.
                if now > 2.0 {
                    assert_eq!(bot.role(), Role::Keeper, "at {now} s");
                }
                raided |= matches!(bot.goal(), Goal::Raid(_));
                fetching |= matches!(bot.goal(), Goal::Fetch(..));
                // A sword is lost with a death: had at any time, it was
                // bought.
                armed |= game.outfits.kit(4).sword > 0;
                // While the person is on the pad, the bot is not there for
                // what it makes: that is theirs.
                if now < 19.0 && now > 2.0 {
                    given = game.purses[0];
                    assert_ne!(bot.goal(), Goal::Collect, "seed {seed}: at {now} s it is at the pad with them");
                    assert_eq!(game.purses[4], [0; 4], "seed {seed}: at {now} s the bot holds {:?}", game.purses[4]);
                }
                if !game.alive(4) || now < 30.0 {
                    stay = 0.0;
                    continue;
                }
                alive_for += dt;
                let at = game.at[4].xz();
                if at.distance(pad) < 2.0 {
                    if stay == 0.0 {
                        visits += 1;
                    }
                    stay += dt;
                    by_the_pad += dt;
                    longest = longest.max(stay);
                } else {
                    stay = 0.0;
                }
                if at.distance(island) > ARENA_ISLAND_RADIUS + 1.0 {
                    away += dt;
                }
            }
            println!(
                "seed {seed}: the bot beside a person is {}: by the pad {visits} times, {by_the_pad:.0} s of {alive_for:.0} in all and {longest:.1} s at the longest; off its island {away:.0} s; raided {raided}, fetched {fetching}; kit {:?}, upgrades {:?}",
                game.squad.bot(4).unwrap().persona().name,
                game.outfits.kit(4),
                game.outfits.teams[0],
            );
            assert!(given[0] >= 7 && given[1] >= 1, "seed {seed}: the person on the pad was given what it made: {given:?}");
            // It comes for what lies there, again and again, and is never
            // there long: a few seconds at the most, and a small part of
            // its time in all (until 2026-11-13 it stood there whenever it
            // was saving for anything, most of the game).
            assert!(visits >= 6, "seed {seed}: it came to the pad {visits} times in seven minutes");
            assert!(longest < 5.0, "seed {seed}: it stood by the pad for {longest} s");
            assert!(by_the_pad < 0.15 * alive_for, "seed {seed}: by the pad {by_the_pad} s of {alive_for}");
            // With no bridge built by the person, it has collected for
            // one, bought it and laid it; and it has a sword.
            assert!(game.nav.joined(Region::Team(0), Region::Middle), "seed {seed}: no bridge to the middle by seven minutes");
            assert!(armed, "seed {seed}: it never had a sword in seven minutes");
            went_out += u32::from(away > 0.2 * alive_for);
            went_raiding += u32::from(raided);
            fetched += u32::from(fetching);
        }
        // And it is out and about: off its island for a good part of the
        // time, out to the pads in the middle and on the smaller islands,
        // and off on a raid of its own, where until 2026-11-13 it never
        // went on one by itself.
        assert!(went_out >= 2, "off its island for a fifth of the time in {went_out} games of three");
        assert!(fetched >= 2, "it went out for amethyst or sapphire in {fetched} games of three");
        assert!(went_raiding >= 1, "it went on a raid in {went_raiding} games of three");
    }
}
