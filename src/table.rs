//! The tables the four games of 2026-11-18 are played at, on this device
//! (Hajun: "add these minigames: Blend In, Knock Off, Bridge Race, Last
//! Stand ... each of these games have a coin entrance fee and payout ...
//! Make these minigames, add them to the game selector panel"): one way in
//! and one way out for all four, whichever game it is, online or offline.
//! What each game is, and how it is played, is its own module's
//! (`blend_in`, `knock_off`, `bridge_race`, `last_stand`); how a table
//! fills, starts and pays is `roundtown_net::table`'s, which the server and
//! this device both go by.
//!
//! **In.** The game selector asks for a seat ([`Ask::Join`]). Online the
//! server is asked, and once it has sat you down (`Tell::Entered`) and
//! stood you on the game's venue, you are at its table ([`AtTable`]): the
//! entity the server sends to everyone in the table's room, with its
//! [`Board`] and its `Seats` on it, which [`TableView`] mirrors for
//! whatever draws from them. Offline this device sits you down itself, at a
//! table of its own that goes by the same rules: the fee comes off your
//! balance, you take the first seat, a moment later the computer takes the
//! seats left one at a time, as in Bridge Wars, if the game has the
//! computer; then the countdown, the game (its own module plays it, and
//! says how it came out: [`Finish`]), the results, and the prize on your
//! balance. Blend In needs people, and is not to be had offline.
//!
//! **While you are at one** the camera is fixed over you on a venue fought
//! as Bridge Wars is (`island::fights`), the buttons round the screen are
//! the game's own (`hud::Layout`), the game's name is said across the
//! screen as you come, and the board top centre says how the table fills
//! and counts down, with a button under it to leave ([`spawn_board`]).
//! Everything a table puts up is [`InTable`], each game's own too, and goes
//! with it. A game holds your body or the camera where it wants them with
//! `Frozen` and `HeldCamera`, and both are let go of as the table ends.
//!
//! **Out.** At the end the results are up a while, in a window as the
//! friends' panel is ([`ResultsWindow`]), your own row picked out, to leave
//! or play again from; then everyone still there is back in the town:
//! online as the server sends them (`Tell::Arrive` in the town), offline as
//! this device does. A connection gone for a while in the middle gives the
//! table up, back to the town offline, as in Bridge Wars.
//!
//! **The games' own modules** are told a table has begun with [`Entered`],
//! and begin afresh on every one, which can come while a table is up: one
//! table straight after another ("Play again", or your group's leader's,
//! once the one before is over), or the table you are at begun afresh
//! (`same_table`: a device put back in the game it was in, or your seat at
//! it moved by the server before play); and that you have left the tables
//! with [`Gone`]. Offline,
//! the computer's players' bodies are put up here as they come (lobby's
//! `spawn_bot_at`, marked [`ComputerSeat`] and `tags::Named`), standing
//! where each waits, and everyone, you too, is stood where their seat
//! arrives as play begins, as the server's own stubs do; each game moves
//! them from there.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::BOT_GAP;
use roundtown_net::table::{self as rules, Shape};
use roundtown_net::{Ask as NetAsk, Member, Minigame, Refusal, Seats, Tell, Venue};
use roundtown_net::{blend_in, bridge_race, knock_off, last_stand};

pub(crate) use roundtown_net::table::{Board, Results, Stage};

use crate::builder::{DIALOG_LAYER, Fill};
use crate::chat::Hushed;
use crate::drive::Driving;
use crate::games;
use crate::hud::{self, TakesPress};
use crate::island::{self, Islands};
use crate::lobby::{self, Balance, Lobby, Seat, Whereabouts};
use crate::net::Online;
use crate::tags::{HidesNames, Named};
use crate::ui::{UiChanged, fill, ui};
use crate::window::{self, Look};
use crate::{Carrying, Dead, Frozen, HeldCamera, OrbitCamera, Player, Team, View};

/// How long a table this device hosts waits, offline, before the computer
/// comes for the seats left or the countdown begins, in seconds: nobody
/// else is coming, and the moment is for seeing where you are (mine).
const ALONE_WAIT: f32 = 3.0;
/// How long the game's name is across the screen as you come to its table,
/// in seconds: Bridge Wars' own.
const NAME_SECS: f32 = 3.0;
/// How long a connection may be gone in the middle of a game played
/// online before the game is given up, in seconds: Bridge Wars' own.
const RECONNECT_WAIT: f32 = 15.0;
/// How long a seat asked of the server is waited on for its answer, in
/// seconds, for a refusal to be said of it: the server answers in a
/// moment, its database's fee too, and past this whatever is turned down
/// is something else (mine).
const ANSWER_MOST: f32 = 10.0;
/// How long the results wait to come up after the game is over, in
/// seconds: for the game's own word on how it ended to be read across the
/// screen first ("Mochi wins!"), which the window would stand over (mine).
const RESULTS_AFTER: f32 = 2.0;
/// Sizes, as shares of the short side of the screen: the board's name and
/// the line under it, as Bridge Wars' board's are; the results' window as
/// wide as the friends' panel, each row of it, and in a row the place, the
/// score and its word, the coins, and the coin's picture beside them.
const BOARD_TEXT: f32 = 4.8;
const BOARD_CAPTION: f32 = 3.4;
const RESULT_ROW: f32 = 7.4;
const PLACE_WIDTH: f32 = 10.0;
const SCORE_WIDTH: f32 = 32.0;
const PRIZE_WIDTH: f32 = 17.0;
const COIN: f32 = 3.8;
/// How wide each of the results' two buttons is.
const RESULT_CHOICE: f32 = 22.0;

/// Where you sit, at a table of one of the four games, if you do: which
/// game, your seat at it, whether the server runs it or this device does,
/// and the table's number (online the server's, from its board once that
/// has come, and 0 until then; offline this device's own).
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct AtTable(pub(crate) Option<Sitting>);

/// You at a table ([`AtTable`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Sitting {
    pub(crate) game: Minigame,
    pub(crate) seat: u8,
    pub(crate) online: bool,
    pub(crate) id: u32,
}

impl AtTable {
    /// Whether you sit at a table this device hosts, offline: nothing goes
    /// online in the middle of it (`net`).
    pub(crate) fn offline(&self) -> bool {
        self.0.is_some_and(|sitting| !sitting.online)
    }

    /// Where you sit, if it is at a table of `game`.
    pub(crate) fn of(&self, game: Minigame) -> Option<Sitting> {
        self.0.filter(|sitting| sitting.game == game)
    }
}

/// Something asked of the tables: a seat at one of `game` (the game
/// selector; offline this device's own table, online the server's); to
/// leave the one you are at (the board's button while it fills and counts
/// down, the results' at the end, or a game's own, as a "You're out!"
/// panel's), to be paid back what you paid for it if it has not begun; to
/// play again at another of the same game once it is over (the results');
/// and the wait over now, for trying a game out (a tap on the board: the
/// server allows it only with `RT_ALLOW_SKIP`, and offline it is always
/// allowed, as Bridge Wars' is).
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ask {
    Join(Minigame),
    Leave,
    Again,
    Skip,
}

/// A table of `game` has begun on this device, you in `seat` at it, the
/// server's (`online`) or this device's own: for the game's own module to
/// begin afresh, dropping whatever it had of a table before (everything
/// [`InTable`] has gone already, and the board is up again). One can come
/// while a table is up, with no [`Gone`] between: one table straight after
/// another ("Play again", or your group's leader's, once the one before is
/// over), or, `same_table`, the table you are at begun afresh: a device put
/// back in the game it was in (the server's `Rules::back`, whose own words
/// come straight after), or your seat at it moved by the server before
/// play, as the computer begins to come in Bridge Race or as a game of
/// people alone begins with a seat before yours left (whoever sits past
/// the seats a table settles on moves up into one it has). The game's
/// name is said across the screen only for a table you were not at, and a
/// group's follower told only of that (`friends`).
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Entered {
    pub(crate) game: Minigame,
    pub(crate) seat: u8,
    pub(crate) online: bool,
    pub(crate) same_table: bool,
}

/// You have left the tables, back to the town, from one of `game`: for its
/// module to give back whatever it held (the fight's state, the chat's
/// hush; the table lets go of your body, the camera and the hush itself,
/// and takes down everything [`InTable`]).
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Gone {
    pub(crate) game: Minigame,
}

/// How a game played offline came out, by seat (as `roundtown_net::table::
/// Results` has it, but for the prizes, which the table works out by the
/// game's own `prizes`): the place each came, 0 first, those tied the
/// same; the score each made; and the table's total, if the game counts
/// one. Written by the game's own module once its game is over; online the
/// server says how it came out, and this is not read.
#[derive(Message, Clone, PartialEq, Debug)]
pub(crate) struct Finish {
    pub(crate) places: Vec<u8>,
    pub(crate) scores: Vec<u32>,
    pub(crate) total: Option<u32>,
}

/// The table you are at as it is drawn from: the entity it is on (online
/// the server's, `Remote`; offline this device's own), its board and who
/// sits where. Written only when it changes.
#[derive(Resource, Default, Clone, PartialEq, Debug)]
pub(crate) struct TableView {
    pub(crate) shown: Option<Entity>,
    pub(crate) board: Option<Board>,
    pub(crate) members: Vec<Member>,
}

impl TableView {
    /// How far the table has got, once its board has come.
    pub(crate) fn stage(&self) -> Option<&Stage> {
        self.board.as_ref().map(|board| &board.stage)
    }

    /// Whether its game is being played.
    pub(crate) fn playing(&self) -> bool {
        matches!(self.stage(), Some(Stage::Playing))
    }

    /// Whether its game is over, its results up.
    pub(crate) fn over(&self) -> bool {
        matches!(self.stage(), Some(Stage::Over(_)))
    }

    /// How its game came out, once it is over.
    pub(crate) fn results(&self) -> Option<&Results> {
        match self.stage() {
            Some(Stage::Over(results)) => Some(results),
            _ => None,
        }
    }

    /// Whole seconds left of the stage it is at, 0 where nothing counts
    /// down.
    pub(crate) fn left(&self) -> u16 {
        self.board.as_ref().map_or(0, |board| board.left)
    }

    /// How many of its seats are taken, by people and the computer.
    pub(crate) fn seated(&self) -> usize {
        self.members.iter().filter(|member| member.id != 0).count()
    }
}

/// Everything a table puts up, in the world and on the screen, each game's
/// own too: all of it goes as the table does.
#[derive(Component)]
pub(crate) struct InTable;

/// The body of the computer's player in this seat, at a table this device
/// hosts offline: put up here as it comes, standing where its seat waits,
/// and stood where its seat arrives as play begins; the game's own module
/// moves it from there.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ComputerSeat(pub(crate) u8);

/// The results at the end of a game, a window as the friends' panel is.
#[derive(Component)]
pub(crate) struct ResultsWindow;

/// The tables' systems, in order: what the server says, what is asked, the
/// table this device hosts, the connection, the sweep, the mirror, the
/// board and the results. Before you are moved, as Bridge Wars' are, and
/// before Bridge Wars' and House Builder's own: a table you are taken from
/// into one of theirs, by your group's leader, has given back the camera
/// and your body before their game takes them. A game's own systems that
/// read what these say go after them.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TableSystems;

/// The board top centre, the game's name on it, the line under it, and
/// what a press on any of a table's buttons does.
#[derive(Component)]
struct BoardRoot;
#[derive(Component)]
struct BoardName;
#[derive(Component)]
struct BoardCaption;
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum TableTap {
    Leave,
    Again,
    Skip,
    /// The cross on the results: they go, and you stay till the time is up.
    Shut,
}

/// The line under the results that counts down to the town.
#[derive(Component)]
struct BackIn;

/// What a table keeps to give back as it ends, whichever runs it: the
/// camera as you had it, your seat in the town, and how long the
/// connection has been gone; a seat asked of the server and not yet
/// answered, and for how long, for a refusal to be said of
/// ([`ANSWER_MOST`]); the table you were at before this one, over, whose
/// board may come a moment after this one's begins, and is not this one's;
/// and the number the next table this device hosts takes.
#[derive(Resource, Default)]
struct Kept {
    view: Option<View>,
    town_seat: usize,
    cut_off: Option<f32>,
    asked: Option<(Minigame, f32)>,
    passed: Option<(Minigame, u32)>,
    next: u32,
}

/// The table this device hosts, offline: how it goes ([`Host`]), its number,
/// the entity its board and seats are on, the computer's bodies by seat,
/// what you paid to sit at it, and the wait to be skipped.
#[derive(Resource)]
struct Hosted {
    host: Host,
    id: u32,
    entity: Entity,
    bodies: Vec<(usize, Entity)>,
    paid: u32,
    skip: bool,
}

/// Whether the results are up, and for which table: how long the game has
/// been over, and whether they have been shut by their cross (you stay till
/// the time is up) or are up already.
#[derive(Resource, Default)]
struct ResultsUp {
    table: Option<(Minigame, u32)>,
    since: f32,
    shut: bool,
    drawn: bool,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<AtTable>()
        .init_resource::<TableView>()
        .init_resource::<Kept>()
        .init_resource::<ResultsUp>()
        .add_message::<Ask>()
        .add_message::<Entered>()
        .add_message::<Gone>()
        .add_message::<Finish>()
        .add_observer(tap)
        .add_systems(
            Update,
            (hear, ask, host, cut_off, sweep, mirror, show_board, show_results)
                .chain()
                .in_set(TableSystems)
                .before(crate::read_player_input)
                .before(island::show_where_i_am)
                .before(crate::arena::ArenaSystems)
                .before(crate::builder::GameSystems),
        );
}

/// Away from the tables, nothing a table put up stands: whatever was put up
/// in the very frame a table ended, after its end took down what there was
/// (a table begun and ended in one frame, a game's last word on it).
fn sweep(at: Res<AtTable>, things: Query<Entity, With<InTable>>, mut commands: Commands) {
    if at.0.is_some() {
        return;
    }
    for thing in &things {
        commands.entity(thing).try_despawn();
    }
}

// ---------------------------------------------------------------- the table

/// How a table hosted on this device goes, stage by stage.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    /// Filling: how long since it was opened.
    Waiting { since: f32 },
    /// Every seat taken: how long until play.
    Countdown { left: f32 },
    /// The game's own module plays it, and says when it is over.
    Playing,
    /// Over, the results up, for so much longer.
    Over { left: f32 },
}

/// What a step of a table hosted on this device brought about, for the
/// world to be told of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Happened {
    /// The computer has taken this seat.
    Came(usize),
    /// Every seat is taken: the countdown begins.
    Counting,
    /// The countdown is over: the game is played.
    Begun,
    /// The results have been up as long as they are: back to the town.
    Done,
}

/// A table this device hosts, offline: the game, its shape, how far it has
/// got, who sits where (you in the first seat, `nobody` in a seat not yet
/// taken), when the computer's next player comes, if it is coming, and how
/// the game came out. Plain, for the tests: the dice and who the computer
/// is are the caller's ([`Host::step`]).
#[derive(Clone, Debug)]
struct Host {
    game: Minigame,
    shape: Shape,
    phase: Phase,
    seats: Vec<Member>,
    coming: Option<f32>,
    results: Option<Results>,
}

impl Host {
    /// A table of `game`, its shape `shape`, you (`me`) in the first of the
    /// seats a table of it has with one person at it
    /// (`roundtown_net::table::seats_for`).
    fn new(game: Minigame, shape: Shape, me: Member) -> Self {
        let mut seats = vec![nobody(); rules::seats_for(game, 1).max(1)];
        seats[0] = me;
        Self {
            game,
            shape,
            phase: Phase::Waiting { since: 0.0 },
            seats,
            coming: None,
            results: None,
        }
    }

    /// Whether the game has not begun: what was paid for a seat is paid
    /// back to whoever leaves it now.
    fn before_play(&self) -> bool {
        matches!(self.phase, Phase::Waiting { .. } | Phase::Countdown { .. })
    }

    /// How many people sit at it: you.
    fn people(&self) -> usize {
        self.seats.iter().filter(|member| member.id != 0 && !member.ai).count()
    }

    /// Moves it on by `dt` seconds, or past its wait or count at once if
    /// `skip`: once nobody has come for [`ALONE_WAIT`], the computer's
    /// players into the seats left, one each `gap()` seconds (all at once
    /// on a skip), the last of them first, as the server seats them
    /// ([`order`]), each as `who(seat, seats)` says, and with every seat
    /// taken and as many people as the game begins with, the countdown;
    /// then play; and at the end of the results, back to the town.
    fn step(
        &mut self,
        dt: f32,
        skip: bool,
        gap: &mut impl FnMut() -> f32,
        who: &mut impl FnMut(usize, &[Member]) -> Member,
    ) -> Vec<Happened> {
        let mut happened = Vec::new();
        match self.phase {
            Phase::Waiting { since } => {
                let since = if skip { since.max(ALONE_WAIT) } else { since + dt };
                self.phase = Phase::Waiting { since };
                if since < ALONE_WAIT {
                    return happened;
                }
                if self.shape.computer {
                    let left = self.coming.get_or_insert(0.0);
                    *left -= dt;
                    let due = if skip {
                        self.seats.len()
                    } else if *left <= 0.0 {
                        *left = gap();
                        1
                    } else {
                        0
                    };
                    for _ in 0..due {
                        let free = order(self.game, self.seats.len()).into_iter().rev().find(|&seat| self.seats[seat].id == 0);
                        let Some(seat) = free else {
                            break;
                        };
                        let member = who(seat, &self.seats);
                        self.seats[seat] = member;
                        happened.push(Happened::Came(seat));
                    }
                }
                let full = self.seats.iter().all(|member| member.id != 0);
                if full && self.people() >= self.shape.fewest {
                    self.phase = Phase::Countdown { left: self.shape.countdown };
                    happened.push(Happened::Counting);
                }
            }
            Phase::Countdown { left } => {
                if left > dt && !skip {
                    self.phase = Phase::Countdown { left: left - dt };
                } else {
                    self.phase = Phase::Playing;
                    happened.push(Happened::Begun);
                }
            }
            Phase::Playing => {}
            Phase::Over { left } => {
                if left > dt && !skip {
                    self.phase = Phase::Over { left: left - dt };
                } else {
                    happened.push(Happened::Done);
                }
            }
        }
        happened
    }

    /// The game is over, as `finish` says, if it is being played: the
    /// results, with what each person is paid by the game's own `prizes`,
    /// up for as long as the game has them up.
    fn finish(&mut self, finish: &Finish) -> Option<&Results> {
        if self.phase != Phase::Playing {
            return None;
        }
        let seats = self.seats.len();
        let places = (0..seats)
            .map(|seat| finish.places.get(seat).copied().unwrap_or(seat as u8))
            .collect();
        let scores = (0..seats).map(|seat| finish.scores.get(seat).copied().unwrap_or(0)).collect();
        let mut results = Results {
            places,
            scores,
            prizes: Vec::new(),
            total: finish.total,
        };
        let people: Vec<bool> = self.seats.iter().map(|member| member.id != 0 && !member.ai).collect();
        results.prizes = prizes(self.game, &results, &people);
        self.results = Some(results);
        self.phase = Phase::Over { left: self.shape.results };
        self.results.as_ref()
    }

    /// Its board, as the server would send it for a table numbered `id`.
    fn board(&self, id: u32) -> Board {
        let (stage, left) = match self.phase {
            Phase::Waiting { since } => (Stage::Waiting, if since < ALONE_WAIT { whole(ALONE_WAIT - since) } else { 0 }),
            Phase::Countdown { left } => (Stage::Countdown, whole(left)),
            Phase::Playing => (Stage::Playing, 0),
            Phase::Over { left } => (Stage::Over(self.results.clone().unwrap_or_default()), whole(left)),
        };
        Board {
            game: self.game,
            id,
            stage,
            left,
            people: self.people() as u8,
            seats: self.seats.len() as u8,
        }
    }
}

/// The order the `seats` seats at a table of `game` are taken in, as the
/// server takes them (its `Table::order`): people the first of them free,
/// and the computer the last. Seat by seat, but in Bridge Race, whose first
/// two people are one team (`bridge_race::PEOPLE_FIRST`).
fn order(game: Minigame, seats: usize) -> Vec<usize> {
    match game {
        Minigame::BridgeRace => bridge_race::PEOPLE_FIRST.iter().copied().filter(|&seat| seat < seats).collect(),
        _ => (0..seats).collect(),
    }
}

/// `seconds` rounded up to whole ones, as a board counts them.
fn whole(seconds: f32) -> u16 {
    seconds.max(0.0).ceil().min(f32::from(u16::MAX)) as u16
}

/// Who is in a seat nobody is in yet: nobody, as the server has an empty
/// seat (`Seats`).
fn nobody() -> Member {
    Member {
        id: 0,
        name: String::new(),
        look: String::new(),
        ai: true,
    }
}

/// What each seat at a table of `game` is paid, by the game's own rules
/// (`roundtown_net::<game>::prizes`); `people[seat]` false for the
/// computer's.
fn prizes(game: Minigame, results: &Results, people: &[bool]) -> Vec<u32> {
    match game {
        Minigame::BlendIn => blend_in::prizes(results, people),
        Minigame::KnockOff => knock_off::prizes(results, people),
        Minigame::BridgeRace => bridge_race::prizes(results, people),
        Minigame::LastStand => last_stand::prizes(results, people),
        Minigame::BridgeWars | Minigame::HouseBuilder => vec![0; results.places.len()],
    }
}

/// What `game`'s score is called among its results: the game's own word
/// (`blend_in::score_word` and the rest).
fn score_word(game: Minigame) -> String {
    match game {
        Minigame::BlendIn => crate::blend_in::score_word(),
        Minigame::KnockOff => crate::knock_off::score_word(),
        Minigame::BridgeRace => crate::bridge_race::score_word(),
        Minigame::LastStand => crate::last_stand::score_word(),
        Minigame::BridgeWars | Minigame::HouseBuilder => String::new(),
    }
}

/// What the line under `game`'s results says of what the table made
/// together, `total`, in the game's own words, if it counts one
/// (`last_stand::total_words` and the rest).
fn total_words(game: Minigame, total: u32) -> Option<String> {
    match game {
        Minigame::BlendIn => crate::blend_in::total_words(total),
        Minigame::KnockOff => crate::knock_off::total_words(total),
        Minigame::BridgeRace => crate::bridge_race::total_words(total),
        Minigame::LastStand => crate::last_stand::total_words(total),
        Minigame::BridgeWars | Minigame::HouseBuilder => None,
    }
}

/// Where the player in `seat` waits at a table of `game` while it fills, as
/// the server stands them there (the shared rules): in Bridge Race's and
/// Last Stand's waiting rooms over their maps, facing the middle of the
/// room; and where each seat arrives elsewhere, on Knock Off's ice and on
/// the path round the town's pond.
fn waiting_place(game: Minigame, seat: usize) -> Transform {
    let in_room = |at: Vec3, middle: Vec2| {
        let toward = middle - at.xz();
        // A turn of `yaw` about the upright points -Z at (-sin yaw, -cos yaw).
        let yaw = (-toward.x).atan2(-toward.y);
        Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw))
    };
    match game {
        Minigame::BridgeRace => in_room(bridge_race::waiting_spot(seat), bridge_race::WAITING_ROOM),
        Minigame::LastStand => in_room(last_stand::waiting_spot(seat), last_stand::WAITING_ROOM),
        _ => starting_place(game.venue(), seat),
    }
}

/// Where the player in `seat` stands as play begins on `venue`, and which
/// way they face, at the height the shared rules give
/// (`roundtown_net::arrival`): on a lifted map, on its lifted land.
fn starting_place(venue: Venue, seat: usize) -> Transform {
    let (at, turn) = roundtown_net::arrival(venue, seat);
    Transform::from_translation(at).with_rotation(turn)
}

/// A seconds count as a clock shows it, `m:ss`, rounded up, so that the
/// last second reads `0:01`. (Bridge Wars and House Builder each have one
/// of their own, private.)
pub(crate) fn clock(seconds: f32) -> String {
    let seconds = seconds.max(0.0).ceil() as u32;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Says across the screen why a seat at a table was turned down: for want
/// of coins, or for your being in the middle of something else. That only
/// a group's leader starts a game is said by `friends`, for every game, and
/// is not said again here. (Bridge Wars' and House Builder's refusals are
/// their own, and say nothing across the screen, as before the tables.)
fn say_refused(commands: &mut Commands, refusal: Refusal) {
    let ui = ui();
    let words = &ui.words.games;
    let said = match refusal {
        Refusal::Coins => &words.refused_coins,
        Refusal::Busy => &words.refused_busy,
        Refusal::NotLeader => return,
    };
    hud::announce(commands, None, said, hud::CAPTION, 3.0);
}

/// Whether a table of `game` you are told you are at, the server's
/// (`online`), is the one you sit at already, `before`, whose game is not
/// over (`over`): the server tells a device it is at its table again as it
/// puts the device back in the game it was in, and as it moves the
/// device's seat before play; and sits it at another only once the one it
/// is at is over ("Play again", or a group's leader taking their group on
/// from it). Offline every table this device opens is another.
fn same_table(before: Sitting, game: Minigame, online: bool, over: bool) -> bool {
    before.online && online && before.game == game && !over
}

// ------------------------------------------------------------ in and out

/// What sitting down at a table, and getting up from one, changes.
#[derive(SystemParam)]
struct Seating<'w, 's> {
    commands: Commands<'w, 's>,
    me: Query<'w, 's, (Entity, &'static mut Seat, &'static mut Balance), With<Player>>,
    bodies: Whereabouts<'w, 's>,
    orbit: ResMut<'w, OrbitCamera>,
    driving: ResMut<'w, Driving>,
    held: ResMut<'w, HeldCamera>,
    hushed: ResMut<'w, Hushed>,
    at: ResMut<'w, AtTable>,
    kept: ResMut<'w, Kept>,
    view: ResMut<'w, TableView>,
    things: Query<'w, 's, Entity, With<InTable>>,
    entered: MessageWriter<'w, Entered>,
    gone: MessageWriter<'w, Gone>,
}

impl Seating<'_, '_> {
    /// You are at a table of `game`, in `seat`, the server's (`online`) or
    /// this device's own, numbered `id`: what a table before it put up goes;
    /// on a venue fought as Bridge Wars is, the camera is fixed over you,
    /// as it was before kept to go back to (and getting there gets you out
    /// of the car, `drive::toggle`, whose own view is the one kept); the
    /// board is up, the game's name across the screen, and the game told.
    /// Told again of the table you are at ([`same_table`]), it is begun
    /// afresh as it stands: under the number it has, its board as it was
    /// mirrored, and its name not said again.
    fn open(&mut self, game: Minigame, seat: u8, online: bool, id: u32) {
        let before = self.at.0;
        let same = before.is_some_and(|before| same_table(before, game, online, self.view.over()));
        for thing in &self.things {
            self.commands.entity(thing).try_despawn();
        }
        // The table you were at, over, which this one comes straight after:
        // its board may still be here a moment after this one's begins, and
        // is never this one's ([`mirror`]).
        if !same && let Some(before) = before.filter(|before| before.online && self.view.over()) {
            self.kept.passed = Some((before.game, before.id));
        }
        let id = match before {
            Some(before) if same => before.id,
            _ => id,
        };
        if island::fights(game.venue()) {
            let before_the_car = self.driving.take_view();
            let fixed = self.orbit.top_down();
            // A table straight after another keeps the view the first kept.
            if self.kept.view.is_none() {
                self.kept.view = fixed.or(before_the_car);
            }
        }
        if let Ok((you, mut sat, _)) = self.me.single_mut() {
            if sat.0 != usize::from(seat) {
                sat.0 = usize::from(seat);
            }
            self.commands.entity(you).remove::<(Dead, Frozen)>();
        }
        self.held.set_if_neq(HeldCamera(None));
        // Nothing of a table before is drawn while this one's board comes.
        if !same && *self.view != TableView::default() {
            *self.view = TableView::default();
        }
        spawn_board(&mut self.commands);
        if !same {
            hud::announce(&mut self.commands, None, &games::name_of(game), hud::HEADLINE, NAME_SECS).insert(InTable);
        }
        self.at.0 = Some(Sitting { game, seat, online, id });
        self.kept.cut_off = None;
        info!(
            "{game:?}: at table {id}, in seat {seat}{}{}",
            if online { " (online)" } else { "" },
            if same { ", the same table begun afresh" } else { "" }
        );
        self.entered.write(Entered {
            game,
            seat,
            online,
            same_table: same,
        });
    }

    /// The table is over for you: everything it put up goes; nothing of a
    /// game's holds your body, the camera or the chat; the camera is as it
    /// was before; and `move_you`, you are back in the town where your seat
    /// there arrives (offline, or the connection given up; online the server
    /// has moved you already). The game is told.
    fn end(&mut self, move_you: bool) {
        let Some(sitting) = self.at.0.take() else {
            return;
        };
        for thing in &self.things {
            self.commands.entity(thing).try_despawn();
        }
        self.commands.remove_resource::<Hosted>();
        self.held.set_if_neq(HeldCamera(None));
        self.hushed.set_if_neq(Hushed(false));
        let town_seat = self.kept.town_seat;
        let mut seat_now = town_seat;
        if let Ok((you, mut seat, _)) = self.me.single_mut() {
            self.commands
                .entity(you)
                .remove::<(Frozen, Dead, Team, Carrying, crate::arena::Health)>();
            if move_you {
                seat.0 = town_seat;
                lobby::send(&mut self.bodies, &mut self.orbit, you, Venue::Town, town_seat);
            }
            seat_now = seat.0;
        }
        match self.kept.view.take() {
            Some(view) => self.orbit.back_to(view),
            // Fixed over you with nothing to go back to: behind you, as you
            // stand in the town.
            None if self.orbit.locked => *self.orbit = OrbitCamera::behind(&island::arrival(Venue::Town, seat_now)),
            None => {}
        }
        if *self.view != TableView::default() {
            *self.view = TableView::default();
        }
        self.kept.cut_off = None;
        self.kept.passed = None;
        info!("{:?}: away from the table, back to the town", sitting.game);
        self.gone.write(Gone { game: sitting.game });
    }

    /// Sits you at a table of `game` this device hosts: offline, or once
    /// over at one of its own, again. Not Blend In's, which needs people;
    /// not before the game's map has loaded, which it waits for; not
    /// without the fee, which comes off your balance (as House Builder's
    /// does offline). You wait where your seat waits.
    fn sit_offline(&mut self, game: Minigame, islands: &Islands, lobby: &Lobby) -> Hosting {
        let Some(shape) = rules::shape(game) else {
            return Hosting::No;
        };
        if !shape.computer && shape.fewest > 1 {
            info!("{game:?} needs people to play with, and is not to be had offline");
            return Hosting::No;
        }
        let venue = game.venue();
        if islands.get(venue).is_none() {
            return Hosting::Wait;
        }
        let again = self.at.0.is_some();
        let you = {
            let Ok((you, seat, mut balance)) = self.me.single_mut() else {
                return Hosting::No;
            };
            if balance.0 < shape.fee {
                say_refused(&mut self.commands, Refusal::Coins);
                return Hosting::No;
            }
            balance.0 -= shape.fee;
            if !again {
                self.kept.town_seat = seat.0;
            }
            you
        };
        let id = self.kept.next;
        self.kept.next = self.kept.next.wrapping_add(1);
        let host = Host::new(game, shape, lobby.mine());
        let entity = self
            .commands
            .spawn((InTable, Name::new("Table"), host.board(id), Seats(host.seats.clone())))
            .id();
        self.open(game, 0, false, id);
        lobby::put(&mut self.bodies, &mut self.orbit, you, venue, waiting_place(game, 0));
        self.commands.insert_resource(Hosted {
            host,
            id,
            entity,
            bodies: Vec::new(),
            paid: shape.fee,
            skip: false,
        });
        Hosting::Sat
    }
}

/// What came of asking this device for a table of its own.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Hosting {
    Sat,
    /// Not yet: the game's map is still loading.
    Wait,
    No,
}

/// What the server says of the tables: that you are at one
/// (`Tell::Entered`: as you are seated, again as a device is put back in
/// the game it was in, and as your seat at it is moved before play); where
/// it stands you while you are at one (`Tell::Arrive`): in another of its
/// seats, which is yours from then, or on any venue but its own, in the
/// town as everyone is sent at its end, or into another game, as your
/// group's leader takes you on from a table over for you, either of which
/// ends it here; and that it has turned a seat down.
fn hear(mut tells: MessageReader<Tell>, mut seating: Seating) {
    for tell in tells.read() {
        match tell {
            Tell::Entered { game, seat } => {
                if !game.at_a_table() {
                    continue;
                }
                seating.kept.asked = None;
                seating.open(*game, *seat, true, 0);
            }
            Tell::Arrive { venue, seat, .. } => {
                let Some(sitting) = seating.at.0.filter(|sitting| sitting.online) else {
                    continue;
                };
                if *venue != sitting.game.venue() {
                    seating.end(false);
                } else if *seat != sitting.seat && !seating.view.over() {
                    // Stood where another of its seats waits or begins: the
                    // server has moved you up into it, which it says with
                    // `Tell::Entered` too, as it is sent now; and should it
                    // say it only by standing you there, it is the same.
                    // (Seats move only before the play and as it begins;
                    // over, the next table comes with its own word.)
                    seating.open(sitting.game, *seat, true, sitting.id);
                }
            }
            Tell::Refused(refusal) => {
                if seating.kept.asked.take().is_some() {
                    info!("the server turned a seat down: {refusal:?}");
                    say_refused(&mut seating.commands, *refusal);
                }
            }
            _ => {}
        }
    }
}

/// What is asked of the tables ([`Ask`]): online, asked of the server, the
/// table being its; offline, done here. A seat asked of the server and not
/// answered is given up on with the connection, which takes its answer
/// with it, or after [`ANSWER_MOST`]: whatever is turned down after that is
/// not it, and not said as if it were.
#[allow(clippy::too_many_arguments)]
fn ask(
    time: Res<Time>,
    mut asks: MessageReader<Ask>,
    online: Res<Online>,
    mut requests: MessageWriter<NetAsk>,
    hosted: Option<ResMut<Hosted>>,
    mut seating: Seating,
    islands: Res<Islands>,
    lobby: Res<Lobby>,
    mut waiting: Local<Option<Minigame>>,
) {
    if let Some((game, waited)) = seating.kept.asked {
        let waited = waited + time.delta_secs();
        seating.kept.asked = (online.is() && waited < ANSWER_MOST).then_some((game, waited));
        if seating.kept.asked.is_none() {
            info!("{game:?}: no answer to the seat asked of the server");
        }
    }
    let mut hosted = hosted;
    for &ask in asks.read() {
        let sitting = seating.at.0;
        let over = seating.view.over();
        match ask {
            Ask::Join(game) => {
                if !game.at_a_table() || sitting.is_some() {
                    continue;
                }
                if online.is() {
                    requests.write(NetAsk::Join(game));
                    seating.kept.asked = Some((game, 0.0));
                    if let Ok((_, seat, _)) = seating.me.single() {
                        seating.kept.town_seat = seat.0;
                    }
                    info!("{game:?}: asking the server for a seat");
                } else {
                    *waiting = Some(game);
                }
            }
            Ask::Again => {
                let Some(sitting) = sitting.filter(|_| over) else {
                    continue;
                };
                if !sitting.online {
                    *waiting = Some(sitting.game);
                } else if online.is() {
                    requests.write(NetAsk::Join(sitting.game));
                    seating.kept.asked = Some((sitting.game, 0.0));
                }
            }
            Ask::Leave => {
                let Some(sitting) = sitting else {
                    continue;
                };
                if sitting.online && online.is() {
                    requests.write(NetAsk::Leave);
                    continue;
                }
                // Offline, what you paid back if the game has not begun.
                if let Some(hosted) = hosted.as_deref()
                    && hosted.host.before_play()
                    && let Ok((_, _, mut balance)) = seating.me.single_mut()
                {
                    balance.0 = balance.0.saturating_add(hosted.paid);
                }
                seating.end(true);
            }
            Ask::Skip => match sitting {
                Some(sitting) if sitting.online => {
                    requests.write(NetAsk::Skip);
                }
                Some(_) => {
                    if let Some(hosted) = hosted.as_deref_mut() {
                        hosted.skip = true;
                    }
                }
                None => {}
            },
        }
    }
    // A table of this device's own, once its map is there to stand on.
    if let Some(game) = *waiting
        && seating.sit_offline(game, &islands, &lobby) != Hosting::Wait
    {
        *waiting = None;
    }
}

/// Moves on the table this device hosts, offline: the computer's players in
/// as they come, put up where each waits; the countdown; everyone where
/// their seat arrives as play begins; the game's own word that it is over,
/// and the prize; and at the end of the results, back to the town. Its
/// board and seats written as they change.
#[allow(clippy::too_many_arguments)]
fn host(
    time: Res<Time>,
    mut finishes: MessageReader<Finish>,
    hosted: Option<ResMut<Hosted>>,
    mut seating: Seating,
    lobby: Res<Lobby>,
    assets: Res<AssetServer>,
    mut tables: Query<(&mut Board, &mut Seats), Without<Remote>>,
) {
    let Some(mut hosted) = hosted else {
        finishes.clear();
        return;
    };
    if !seating.at.offline() {
        finishes.clear();
        return;
    }
    let game = hosted.host.game;
    let venue = game.venue();
    for finish in finishes.read() {
        let Some(results) = hosted.host.finish(finish) else {
            continue;
        };
        // You are the first seat.
        let prize = results.prizes.first().copied().unwrap_or(0);
        info!("{game:?}: over, places {:?}, paid {prize}", results.places);
        if let Ok((_, _, mut balance)) = seating.me.single_mut() {
            balance.0 = balance.0.saturating_add(prize);
        }
    }
    let skip = std::mem::take(&mut hosted.skip);
    let others = lobby.seats.clone();
    let happened = hosted.host.step(
        time.delta_secs(),
        skip,
        &mut || BOT_GAP.0 + (BOT_GAP.1 - BOT_GAP.0) * fastrand::f32(),
        // The computer as each seat's number and one, as Bridge Wars' are
        // offline: yours, the first seat's, is 1.
        &mut |seat: usize, seats: &[Member]| lobby::ai_member(seat, seat as u64 + 1, seats, &others),
    );
    for happening in happened {
        match happening {
            Happened::Came(seat) => {
                let member = hosted.host.seats[seat].clone();
                let body = lobby::spawn_bot_at(&mut seating.commands, &assets, &member, seat, venue, waiting_place(game, seat));
                seating.commands.entity(body).insert((InTable, ComputerSeat(seat as u8), Named));
                hosted.bodies.push((seat, body));
                info!("{game:?}: the computer takes seat {seat}, as {}", member.name);
            }
            Happened::Counting => info!("{game:?}: every seat taken, counting down"),
            Happened::Begun => {
                // Everyone where their seat arrives, as the server's stubs
                // stand them: the game moves them from there.
                if let Ok((you, ..)) = seating.me.single() {
                    lobby::put(&mut seating.bodies, &mut seating.orbit, you, venue, starting_place(venue, 0));
                }
                for &(seat, body) in &hosted.bodies {
                    lobby::put(&mut seating.bodies, &mut seating.orbit, body, venue, starting_place(venue, seat));
                }
                info!("{game:?}: begun");
            }
            Happened::Done => {
                seating.end(true);
                return;
            }
        }
    }
    if let Ok((mut board, mut seats)) = tables.get_mut(hosted.entity) {
        let now = hosted.host.board(hosted.id);
        if *board != now {
            *board = now;
        }
        if seats.0 != hosted.host.seats {
            seats.0.clone_from(&hosted.host.seats);
        }
    }
}

/// Online, the connection gone in the middle of a table: it waits a moment
/// for it to come back, and then gives the table up, back to the town as
/// it is offline, as Bridge Wars does.
fn cut_off(time: Res<Time>, online: Res<Online>, mut seating: Seating) {
    let Some(sitting) = seating.at.0 else {
        return;
    };
    if !sitting.online || online.is() {
        if seating.kept.cut_off.is_some() {
            seating.kept.cut_off = None;
        }
        return;
    }
    let gone = match seating.kept.cut_off {
        Some(gone) => gone + time.delta_secs(),
        None => {
            hud::announce(&mut seating.commands, None, &ui().words.house_builder.reconnecting, hud::CAPTION, 3.0)
                .insert(InTable);
            0.0
        }
    };
    seating.kept.cut_off = Some(gone);
    if gone < RECONNECT_WAIT {
        return;
    }
    seating.end(true);
    hud::announce(&mut seating.commands, None, &ui().words.house_builder.lost_connection, hud::CAPTION, 3.0);
}

/// Keeps [`TableView`] as the table you are at stands: online the server's
/// entity of your game (the newest, and never the table over that you have
/// just gone on from, whose board may still be here a moment: `passed`),
/// and its number into [`AtTable`]; offline this device's own. Written only
/// when it changes.
fn mirror(
    mut at: ResMut<AtTable>,
    kept: Res<Kept>,
    hosted: Option<Res<Hosted>>,
    tables: Query<(Entity, &Board, &Seats, Has<Remote>)>,
    mut view: ResMut<TableView>,
) {
    let Some(sitting) = at.0 else {
        if *view != TableView::default() {
            *view = TableView::default();
        }
        return;
    };
    let shown = if sitting.online {
        tables
            .iter()
            .filter(|(_, board, _, remote)| {
                *remote && board.game == sitting.game && kept.passed != Some((board.game, board.id))
            })
            .max_by_key(|(_, board, ..)| board.id)
    } else {
        hosted.and_then(|hosted| tables.get(hosted.entity).ok())
    };
    let Some((entity, board, seats, _)) = shown else {
        return;
    };
    if sitting.online && sitting.id != board.id {
        at.0 = Some(Sitting { id: board.id, ..sitting });
    }
    if view.shown != Some(entity) || view.board.as_ref() != Some(board) || view.members != seats.0 {
        *view = TableView {
            shown: Some(entity),
            board: Some(board.clone()),
            members: seats.0.clone(),
        };
    }
}

// ---------------------------------------------------------------- the board

/// The board top centre, as Bridge Wars' is: on the plate's grey, the
/// game's name, and under it how the table fills and counts down
/// ([`caption`]); and under that a button to leave. Up while the table
/// fills and counts down; each game draws its own over play. A tap on the
/// plate moves the table on, for trying a game out ([`Ask::Skip`]).
fn spawn_board(commands: &mut Commands) {
    let ui = ui();
    let colours = ui.colours;
    commands
        .spawn((
            InTable,
            BoardRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(hud::EDGE),
                left: Val::ZERO,
                right: Val::ZERO,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(0.8),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|board| {
            board.spawn((
                TableTap::Skip,
                TakesPress,
                Button,
                HidesNames,
                Node {
                    padding: UiRect::axes(Val::VMin(3.0), Val::VMin(0.7)),
                    border_radius: BorderRadius::all(Val::VMin(3.0)),
                    ..default()
                },
                BackgroundColor(colours.panel),
                Fill {
                    idle: colours.panel,
                    lit: colours.panel_lit,
                },
                children![(BoardName, hud::words("", BOARD_TEXT, colours.text))],
            ));
            board.spawn((BoardCaption, hud::words("", BOARD_CAPTION, colours.text)));
            board.spawn((
                TableTap::Leave,
                HidesNames,
                window::choice(ui.words.table.leave.clone(), Look::No),
            ));
        });
}

/// What the board says under the game's name, `board` as it stands with
/// `seated` of its seats taken: while it fills, the time before the
/// computer comes for the seats left; then how many of them are taken as
/// it comes; or, in a game of people alone, how many more it wants, and
/// once it has them, how many it has with the time before it begins; and
/// then the countdown. Nothing while it is played, or over.
fn caption(board: &Board, seated: usize) -> String {
    let ui = ui();
    let words = &ui.words.table;
    let shape = rules::shape(board.game);
    let computer = shape.is_some_and(|shape| shape.computer);
    let fewest = shape.map_or(1, |shape| shape.fewest);
    // Of how many: with the computer, the seats it will have; of people
    // alone, as many as may come.
    let of = if computer { usize::from(board.seats) } else { shape.map_or(usize::from(board.seats), |shape| shape.most) };
    let (count, seats, time) = (seated.to_string(), of.to_string(), clock(f32::from(board.left)));
    match board.stage {
        Stage::Waiting if computer && board.left > 0 => fill(&words.bots_join, &[("time", &time)]),
        Stage::Waiting if !computer && usize::from(board.people) < fewest => match fewest - usize::from(board.people) {
            1 => words.one_more.clone(),
            short => fill(&words.more, &[("count", &short.to_string())]),
        },
        Stage::Waiting if !computer && board.left > 0 => {
            fill(&words.players_for, &[("count", &count), ("seats", &seats), ("time", &time)])
        }
        Stage::Waiting => fill(&words.players, &[("count", &count), ("seats", &seats)]),
        Stage::Countdown => fill(&words.starting, &[("seconds", &board.left.to_string())]),
        Stage::Playing | Stage::Over(_) => String::new(),
    }
}

/// Shows the board while the table fills and counts down, and writes on
/// it what it says, only as that changes; puts it away for play. Drawn
/// again in the file's colours as the file changes on desktop.
#[allow(clippy::type_complexity)]
fn show_board(
    view: Res<TableView>,
    at: Res<AtTable>,
    changed: Res<UiChanged>,
    boards: Query<Entity, With<BoardRoot>>,
    mut roots: Query<&mut Visibility, With<BoardRoot>>,
    mut names: Query<&mut Text, (With<BoardName>, Without<BoardCaption>)>,
    mut captions: Query<&mut Text, (With<BoardCaption>, Without<BoardName>)>,
    mut commands: Commands,
) {
    let Some(sitting) = at.0 else {
        return;
    };
    if changed.is_changed() && !changed.is_added() {
        for board in &boards {
            commands.entity(board).despawn();
        }
        spawn_board(&mut commands);
        return;
    }
    let up = matches!(view.stage(), Some(Stage::Waiting | Stage::Countdown));
    for mut root in &mut roots {
        root.set_if_neq(if up { Visibility::Inherited } else { Visibility::Hidden });
    }
    if !up {
        return;
    }
    let name = games::name_of(sitting.game);
    let said = view.board.as_ref().map(|board| caption(board, view.seated())).unwrap_or_default();
    for mut text in &mut names {
        if text.0 != name {
            text.0.clone_from(&name);
        }
    }
    for mut text in &mut captions {
        if text.0 != said {
            text.0.clone_from(&said);
        }
    }
}

/// A press on one of a table's buttons: the board's and the results'. Only
/// what the press landed on.
fn tap(
    click: On<Pointer<Click>>,
    taps: Query<&TableTap>,
    windows: Query<Entity, With<ResultsWindow>>,
    mut results: ResMut<ResultsUp>,
    mut asks: MessageWriter<Ask>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    let Ok(&tapped) = taps.get(click.entity) else {
        return;
    };
    match tapped {
        TableTap::Leave => {
            asks.write(Ask::Leave);
        }
        TableTap::Again => {
            asks.write(Ask::Again);
        }
        TableTap::Skip => {
            asks.write(Ask::Skip);
        }
        TableTap::Shut => {
            results.shut = true;
            for window in &windows {
                commands.entity(window).despawn();
            }
        }
    }
}

// -------------------------------------------------------------- the results

/// One row of the results, as it is drawn: whose seat, the place it came,
/// the name it goes by, its score, what it was paid (people only; the
/// computer is paid nothing, and says nothing of coins), and whether it is
/// yours.
#[derive(Clone, PartialEq, Debug)]
struct Row {
    seat: usize,
    place: u8,
    name: String,
    score: u32,
    prize: Option<u32>,
    mine: bool,
}

/// The rows of `results`, one to each seat taken (`members`), first place
/// first and those tied by seat; `me` your seat.
fn rows(results: &Results, members: &[Member], me: Option<u8>) -> Vec<Row> {
    let mut rows: Vec<Row> = members
        .iter()
        .enumerate()
        .filter(|(_, member)| member.id != 0)
        .map(|(seat, member)| Row {
            seat,
            place: results.places.get(seat).copied().unwrap_or(u8::MAX),
            name: member.name.clone(),
            score: results.scores.get(seat).copied().unwrap_or(0),
            prize: (!member.ai).then(|| results.prizes.get(seat).copied().unwrap_or(0)),
            mine: me.is_some_and(|me| usize::from(me) == seat),
        })
        .collect();
    rows.sort_by_key(|row| (row.place, row.seat));
    rows
}

/// A place as the results say it: the file's first to fourth, and past
/// them the fifth and on.
fn place_word(place: u8) -> String {
    let ui = ui();
    match ui.words.table.places.get(usize::from(place)) {
        Some(word) => word.clone(),
        None => format!("{}th", u32::from(place) + 1),
    }
}

/// Puts the results up a moment after the game is over ([`RESULTS_AFTER`]),
/// unless their cross has shut them, and writes the time to the town under
/// them as it counts down; takes them down once the table is not over, and
/// draws them again in the file's colours as it changes on desktop.
#[allow(clippy::too_many_arguments)]
fn show_results(
    time: Res<Time>,
    view: Res<TableView>,
    at: Res<AtTable>,
    changed: Res<UiChanged>,
    mut up: ResMut<ResultsUp>,
    windows: Query<Entity, With<ResultsWindow>>,
    mut backs: Query<&mut Text, With<BackIn>>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let over = at.0.zip(view.board.as_ref()).zip(view.results());
    let Some(((sitting, board), results)) = over else {
        for window in &windows {
            commands.entity(window).despawn();
        }
        if up.table.is_some() {
            *up = ResultsUp::default();
        }
        return;
    };
    let table = (sitting.game, board.id);
    if up.table != Some(table) {
        *up = ResultsUp {
            table: Some(table),
            ..default()
        };
    }
    up.since += time.delta_secs();
    if changed.is_changed() && !changed.is_added() && up.drawn {
        for window in &windows {
            commands.entity(window).despawn();
        }
        up.drawn = false;
    }
    if !up.shut && !up.drawn && up.since >= RESULTS_AFTER {
        spawn_results(&mut commands, &assets, sitting, results, &view.members);
        up.drawn = true;
    }
    let back = fill(&ui().words.table.back_in, &[("time", &clock(f32::from(view.left())))]);
    for mut text in &mut backs {
        if text.0 != back {
            text.0.clone_from(&back);
        }
    }
}

/// The results, a window as the friends' panel is, in the middle of the
/// screen over the game, which goes on under it: the game's name over
/// them; a row to each seat, first place first, with its place, its name,
/// its score with the game's own word for it, and what each person was
/// paid, with the coin; your own row picked out; the table's total in the
/// game's own words, if it counts one; the time to the town; and Leave and
/// Play again. Its cross shuts it, and you stay till the time is up.
fn spawn_results(commands: &mut Commands, assets: &AssetServer, sitting: Sitting, results: &Results, members: &[Member]) {
    let ui = ui();
    let (colours, words) = (ui.colours.window, &ui.words.table);
    let title = fill(&words.results, &[("game", &games::name_of(sitting.game))]);
    let pane = commands
        .spawn((
            ResultsWindow,
            InTable,
            HidesNames,
            window::pane(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::VMin(window::WIDTH),
                padding: UiRect::all(Val::VMin(window::PAD)),
                ..default()
            }),
            UiTransform::from_translation(Val2::new(Val::Percent(-50.0), Val::Percent(-50.0))),
            GlobalZIndex(DIALOG_LAYER - 1),
        ))
        .id();
    commands.spawn((
        ChildOf(pane),
        window::header(),
        children![window::words(title, window::TITLE, colours.title)],
    ));
    commands.spawn((ChildOf(pane), window::cross(assets.load("icons/close.png"), TableTap::Shut)));
    let list = commands
        .spawn((
            ChildOf(pane),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::VMin(0.6),
                margin: UiRect::top(Val::VMin(1.2)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let word = score_word(sitting.game);
    for row in rows(results, members, Some(sitting.seat)) {
        let line = commands
            .spawn((
                ChildOf(list),
                Node {
                    height: Val::VMin(RESULT_ROW),
                    align_items: AlignItems::Center,
                    column_gap: Val::VMin(1.6),
                    padding: UiRect::horizontal(Val::VMin(1.6)),
                    border_radius: BorderRadius::all(Val::VMin(1.2)),
                    ..default()
                },
                BackgroundColor(if row.mine { colours.row_mine } else { Color::NONE }),
                Pickable::IGNORE,
            ))
            .id();
        commands.spawn((
            ChildOf(line),
            Node {
                width: Val::VMin(PLACE_WIDTH),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
            children![window::words(place_word(row.place), window::NAME, colours.name)],
        ));
        commands.spawn((
            ChildOf(line),
            Node {
                flex_grow: 1.0,
                min_width: Val::ZERO,
                overflow: Overflow::clip_x(),
                ..default()
            },
            Pickable::IGNORE,
            children![window::words(row.name.clone(), window::NAME, colours.name)],
        ));
        // The score, and smaller after it, fainter, what it is.
        commands.spawn((
            ChildOf(line),
            Node {
                width: Val::VMin(SCORE_WIDTH),
                flex_shrink: 0.0,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Baseline,
                column_gap: Val::VMin(1.0),
                ..default()
            },
            Pickable::IGNORE,
            children![
                window::words(hud::money(row.score), window::NAME, colours.name),
                window::words(word.clone(), window::STATUS, colours.status),
            ],
        ));
        let coins = commands
            .spawn((
                ChildOf(line),
                Node {
                    width: Val::VMin(PRIZE_WIDTH),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Center,
                    column_gap: Val::VMin(1.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        if let Some(prize) = row.prize {
            let paid = if prize > 0 { format!("+{}", hud::money(prize)) } else { "0".into() };
            commands.spawn((ChildOf(coins), window::words(paid, window::NAME, colours.name)));
            commands.spawn((
                ChildOf(coins),
                ImageNode::new(assets.load(hud::COIN_ICON)),
                Node {
                    width: Val::VMin(COIN),
                    height: Val::VMin(COIN),
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ));
        }
    }
    // What the whole table made, in the game's own words, if it counts it.
    if let Some(total) = results.total.and_then(|total| total_words(sitting.game, total)) {
        commands.spawn((
            ChildOf(pane),
            Node {
                margin: UiRect::top(Val::VMin(1.6)),
                padding: UiRect::horizontal(Val::VMin(1.6)),
                ..default()
            },
            Pickable::IGNORE,
            children![window::words(total, window::NAME, colours.name)],
        ));
    }
    commands.spawn((
        ChildOf(pane),
        Node {
            margin: UiRect::top(Val::VMin(1.6)),
            padding: UiRect::horizontal(Val::VMin(1.6)),
            ..default()
        },
        Pickable::IGNORE,
        children![(BackIn, window::words("", window::STATUS, colours.status))],
    ));
    commands.spawn((
        ChildOf(pane),
        Node {
            justify_content: JustifyContent::Center,
            column_gap: Val::VMin(3.0),
            margin: UiRect::top(Val::VMin(2.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (TableTap::Leave, window::choice_wide(words.leave.clone(), Look::No, RESULT_CHOICE)),
            (TableTap::Again, window::choice_wide(words.play_again.clone(), Look::Yes, RESULT_CHOICE)),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(id: u64, name: &str, ai: bool) -> Member {
        Member {
            id,
            name: name.into(),
            look: "boy".into(),
            ai,
        }
    }

    /// A table of `game` played offline step by step, the computer's
    /// players a half second apart, until it is over or `most` seconds have
    /// gone: what happened, and when.
    fn played(game: Minigame, finish_after: f32) -> (Host, Vec<(f32, Happened)>) {
        let mut host = Host::new(game, rules::shape(game).unwrap(), member(1, "You", false));
        let mut happened = Vec::new();
        let dt = 0.05;
        let mut clock = 0.0;
        let mut playing_since = None;
        let mut gap = || 0.5;
        let mut who = |seat: usize, _: &[Member]| member(seat as u64 + 1, "Bot", true);
        while clock < 200.0 {
            clock += dt;
            for happening in host.step(dt, false, &mut gap, &mut who) {
                if happening == Happened::Begun {
                    playing_since = Some(clock);
                }
                happened.push((clock, happening));
                if happening == Happened::Done {
                    return (host, happened);
                }
            }
            if playing_since.is_some_and(|since| clock - since >= finish_after) && host.phase == Phase::Playing {
                let seats = host.seats.len();
                host.finish(&Finish {
                    places: (0..seats as u8).collect(),
                    scores: vec![0; seats],
                    total: Some(2),
                });
            }
        }
        (host, happened)
    }

    #[test]
    fn a_table_played_offline_goes_wait_computer_one_at_a_time_countdown_play_results_town() {
        let (host, happened) = played(Minigame::KnockOff, 20.0);
        let what: Vec<Happened> = happened.iter().map(|(_, what)| *what).collect();
        // Three of the computer's players into the three seats left, one
        // at a time, the last seat first as the server seats them, then
        // the countdown, play and back to the town.
        assert_eq!(
            what,
            [
                Happened::Came(3),
                Happened::Came(2),
                Happened::Came(1),
                Happened::Counting,
                Happened::Begun,
                Happened::Done,
            ]
        );
        let at = |wanted: Happened| happened.iter().find(|(_, what)| *what == wanted).unwrap().0;
        // Nobody before the wait is out, then one each half second (to a
        // step or two of the 0.05 s ones taken).
        let near = 0.11;
        assert!((at(Happened::Came(3)) - ALONE_WAIT).abs() < near, "{}", at(Happened::Came(3)));
        assert!((at(Happened::Came(2)) - at(Happened::Came(3)) - 0.5).abs() < near);
        assert!((at(Happened::Came(1)) - at(Happened::Came(2)) - 0.5).abs() < near);
        // The countdown as the last comes, as long as the game's.
        let shape = knock_off::SHAPE;
        assert!((at(Happened::Counting) - at(Happened::Came(1))).abs() < 1e-3);
        assert!((at(Happened::Begun) - at(Happened::Counting) - shape.countdown).abs() < near);
        // The results as long as the game has them up after it is over.
        assert!((at(Happened::Done) - at(Happened::Begun) - 20.0 - shape.results).abs() < 2.0 * near);
        // And everyone is paid as the game pays: you first, 130.
        let results = host.results.as_ref().unwrap();
        assert_eq!(results.places, [0, 1, 2, 3]);
        assert_eq!(results.prizes, [130, 0, 0, 0]);
        assert_eq!(host.seats.iter().filter(|member| member.ai).count(), 3);
    }

    #[test]
    fn a_table_of_people_alone_counts_down_once_nobody_comes_and_bridge_race_alone_is_one_against_one() {
        // Last Stand, alone: one seat, nobody else, the countdown once the
        // wait is out, and paid by the waves cleared.
        let (host, happened) = played(Minigame::LastStand, 5.0);
        let what: Vec<Happened> = happened.iter().map(|(_, what)| *what).collect();
        assert_eq!(what, [Happened::Counting, Happened::Begun, Happened::Done]);
        assert!((happened[0].0 - ALONE_WAIT).abs() < 0.11);
        assert_eq!(host.seats.len(), 1);
        assert_eq!(host.results.as_ref().unwrap().prizes, [20]);
        // Bridge Race, alone: you and one of the computer's, red and blue.
        let (host, happened) = played(Minigame::BridgeRace, 5.0);
        let what: Vec<Happened> = happened.iter().map(|(_, what)| *what).collect();
        assert_eq!(what, [Happened::Came(1), Happened::Counting, Happened::Begun, Happened::Done]);
        assert_eq!(host.seats.len(), 2);
        assert_eq!((bridge_race::team(0), bridge_race::team(1)), (0, 1));
        assert_eq!(host.results.as_ref().unwrap().prizes, [110, 0]);
        // Blend In has nobody to begin with offline: it never counts down.
        let mut lonely = Host::new(Minigame::BlendIn, blend_in::SHAPE, member(1, "You", false));
        let mut gap = || 0.5;
        let mut who = |seat: usize, _: &[Member]| member(seat as u64 + 1, "Bot", true);
        for _ in 0..1000 {
            let happened = lonely.step(0.1, false, &mut gap, &mut who);
            assert!(happened.is_empty(), "{happened:?}");
        }
    }

    #[test]
    fn a_skip_brings_the_computer_all_at_once_and_ends_each_count() {
        let mut host = Host::new(Minigame::KnockOff, knock_off::SHAPE, member(1, "You", false));
        let mut gap = || 0.5;
        let mut who = |seat: usize, _: &[Member]| member(seat as u64 + 1, "Bot", true);
        let happened = host.step(0.016, true, &mut gap, &mut who);
        assert_eq!(happened, [Happened::Came(3), Happened::Came(2), Happened::Came(1), Happened::Counting]);
        assert_eq!(host.step(0.016, true, &mut gap, &mut who), [Happened::Begun]);
        // Play is the game's: a skip does nothing to it.
        assert!(host.step(0.016, true, &mut gap, &mut who).is_empty());
        assert_eq!(host.phase, Phase::Playing);
        host.finish(&Finish { places: vec![2, 0, 1, 3], scores: vec![], total: None });
        assert_eq!(host.step(0.016, true, &mut gap, &mut who), [Happened::Done]);
        // A finish that comes when nothing is played is nothing.
        assert!(host.clone().finish(&Finish { places: vec![], scores: vec![], total: None }).is_none());
        // The seats in the order the server takes them, the computer from
        // the far end: seat by seat, and Bridge Race's two people first one
        // team (so the computer takes 3 and then 1 beside two people).
        assert_eq!(order(Minigame::KnockOff, 4), [0, 1, 2, 3]);
        assert_eq!(order(Minigame::BridgeRace, 4), [0, 2, 1, 3]);
        assert_eq!(order(Minigame::BridgeRace, 2), [0, 1]);
    }

    #[test]
    fn the_board_of_a_table_played_offline_says_what_the_server_s_would() {
        let mut host = Host::new(Minigame::KnockOff, knock_off::SHAPE, member(1, "You", false));
        let mut gap = || 0.5;
        let mut who = |seat: usize, _: &[Member]| member(seat as u64 + 1, "Bot", true);
        let board = host.board(7);
        assert_eq!((board.game, board.id, board.stage.clone()), (Minigame::KnockOff, 7, Stage::Waiting));
        assert_eq!((board.left, board.people, board.seats), (3, 1, 4));
        host.step(1.2, false, &mut gap, &mut who);
        assert_eq!(host.board(7).left, 2, "rounded up");
        // Every seat taken: the countdown, in whole seconds.
        host.step(0.0, true, &mut gap, &mut who);
        let board = host.board(7);
        assert_eq!((board.stage, board.left, board.people), (Stage::Countdown, 5, 1));
    }

    #[test]
    fn the_results_are_in_place_order_ties_by_seat_people_paid_and_yours_picked_out() {
        let members = vec![
            member(11, "Mochi", false),
            member(12, "Bot", true),
            member(0, "", true),
            member(14, "Dubu", false),
        ];
        let results = Results {
            places: vec![2, 0, 3, 0],
            scores: vec![5, 9, 0, 9],
            prizes: vec![10, 0, 0, 95],
            total: None,
        };
        let rows = rows(&results, &members, Some(0));
        let order: Vec<(usize, u8)> = rows.iter().map(|row| (row.seat, row.place)).collect();
        // The two tied first by seat, then the third; the empty seat not at
        // all.
        assert_eq!(order, [(1, 0), (3, 0), (0, 2)]);
        // People with what they were paid, the computer with nothing said.
        assert_eq!(rows.iter().map(|row| row.prize).collect::<Vec<_>>(), [None, Some(95), Some(10)]);
        assert_eq!(rows.iter().map(|row| row.mine).collect::<Vec<_>>(), [false, false, true]);
        assert_eq!(rows[1].name, "Dubu");
        assert_eq!(rows[1].score, 9);
        // Tied places read the same: 1st and 1st, then 3rd.
        assert_eq!(place_word(rows[0].place), place_word(rows[1].place));
        assert_eq!(place_word(2), ui().words.table.places[2]);
        assert_eq!(place_word(5), "6th");
    }

    #[test]
    fn the_board_says_how_the_table_fills_and_counts_down() {
        let file = ui();
        let words = &file.words.table;
        let board = |game: Minigame, stage: Stage, left: u16, people: u8, seats: u8| Board {
            game,
            id: 1,
            stage,
            left,
            people,
            seats,
        };
        // With the computer: the time before it comes, then the seats.
        let waiting = board(Minigame::KnockOff, Stage::Waiting, 7, 1, 4);
        assert_eq!(caption(&waiting, 1), fill(&words.bots_join, &[("time", "0:07")]));
        let coming = board(Minigame::KnockOff, Stage::Waiting, 0, 1, 4);
        assert_eq!(caption(&coming, 3), fill(&words.players, &[("count", "3"), ("seats", "4")]));
        // People alone: how many more Blend In wants, then how many it has
        // of as many as may come, and the time before it begins.
        let alone = board(Minigame::BlendIn, Stage::Waiting, 0, 1, 1);
        assert_eq!(caption(&alone, 1), words.one_more);
        let two = board(Minigame::BlendIn, Stage::Waiting, 12, 2, 2);
        assert_eq!(caption(&two, 2), fill(&words.players_for, &[("count", "2"), ("seats", "4"), ("time", "0:12")]));
        // Counting down, and nothing while it is played.
        let counting = board(Minigame::LastStand, Stage::Countdown, 3, 1, 1);
        assert_eq!(caption(&counting, 1), fill(&words.starting, &[("seconds", "3")]));
        assert_eq!(caption(&board(Minigame::LastStand, Stage::Playing, 0, 1, 1), 1), "");
        assert_eq!(clock(0.2), "0:01");
        assert_eq!(clock(75.0), "1:15");
    }

    /// A world with what sitting at a table and getting up from one needs,
    /// and you in it, in the town's seat 5.
    fn a_world() -> World {
        let mut world = World::new();
        world.init_resource::<Messages<Tell>>();
        world.init_resource::<Messages<Entered>>();
        world.init_resource::<Messages<Gone>>();
        world.init_resource::<OrbitCamera>();
        world.init_resource::<Driving>();
        world.init_resource::<HeldCamera>();
        world.init_resource::<Hushed>();
        world.init_resource::<AtTable>();
        world.init_resource::<Kept>();
        world.init_resource::<TableView>();
        world.spawn((
            Player,
            Seat(5),
            Balance(1000),
            Transform::default(),
            Venue::Town,
            crate::PlayerJump::default(),
        ));
        world
    }

    /// The server says `tell`: heard, as a frame would hear it, and then
    /// what is left of a table swept, as at the end of the frame. (A reader
    /// made afresh reads every message still kept, so each goes once read.)
    fn told(world: &mut World, tell: Tell) {
        use bevy::ecs::system::RunSystemOnce;
        world.resource_mut::<Messages<Tell>>().write(tell);
        world.run_system_once(hear).expect("heard");
        world.run_system_once(sweep).expect("swept");
        world.resource_mut::<Messages<Tell>>().drain().for_each(drop);
    }

    #[test]
    fn online_the_server_s_seat_opens_the_table_and_the_town_ends_it() {
        let mut world = a_world();
        // Seated at a table of Bridge Race: you are at it, the game is
        // told, and the camera is fixed over you, as in Bridge Wars.
        told(&mut world, Tell::Entered { game: Minigame::BridgeRace, seat: 2 });
        let sitting = Sitting {
            game: Minigame::BridgeRace,
            seat: 2,
            online: true,
            id: 0,
        };
        assert_eq!(world.resource::<AtTable>().0, Some(sitting));
        let entered: Vec<Entered> = world.resource_mut::<Messages<Entered>>().drain().collect();
        assert_eq!(
            entered,
            [Entered {
                game: Minigame::BridgeRace,
                seat: 2,
                online: true,
                same_table: false
            }]
        );
        assert!(world.resource::<OrbitCamera>().locked);
        assert!(world.resource::<Kept>().view.is_some(), "the view before kept to go back to");
        let things = world.query_filtered::<(), With<InTable>>().iter(&world).count();
        assert!(things >= 2, "the board and the game's name: {things}");
        // Held by the game meanwhile, and hushed.
        world.resource_mut::<HeldCamera>().0 = Some(Transform::IDENTITY);
        world.resource_mut::<Hushed>().0 = true;
        // A seat of another game's is not a table's, and nothing comes of it.
        told(&mut world, Tell::Entered { game: Minigame::BridgeWars, seat: 1 });
        assert_eq!(world.resource::<AtTable>().0, Some(sitting));
        // Back in the town, as the server sends everyone at the end: the
        // table is over here, all of it taken down, the camera as it was,
        // nothing of the game's holding you, and the game told.
        told(
            &mut world,
            Tell::Arrive {
                venue: Venue::Town,
                seat: 5,
                spot: None,
                arrival: 3,
            },
        );
        assert_eq!(world.resource::<AtTable>().0, None);
        let gone: Vec<Gone> = world.resource_mut::<Messages<Gone>>().drain().collect();
        assert_eq!(gone, [Gone { game: Minigame::BridgeRace }]);
        assert!(!world.resource::<OrbitCamera>().locked);
        assert_eq!(world.resource::<HeldCamera>().0, None);
        assert!(!world.resource::<Hushed>().0);
        assert_eq!(world.query_filtered::<(), With<InTable>>().iter(&world).count(), 0);
        // Nothing to end twice.
        told(&mut world, Tell::Arrive { venue: Venue::Town, seat: 5, spot: None, arrival: 4 });
        assert_eq!(world.resource_mut::<Messages<Gone>>().drain().count(), 0);
    }

    #[test]
    fn online_the_board_is_the_newest_of_your_game_and_never_the_table_you_left() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.init_resource::<AtTable>();
        world.init_resource::<Kept>();
        world.init_resource::<TableView>();
        let board = |id: u32, stage: Stage| Board {
            game: Minigame::KnockOff,
            id,
            stage,
            left: 9,
            people: 2,
            seats: 4,
        };
        let seats = Seats(vec![member(7, "Mochi", false), member(8, "Dubu", false)]);
        // The table you were at, over; another game's; and the one you are
        // at now, a moment after.
        world.spawn((Remote, board(3, Stage::Over(Results::default())), seats.clone()));
        world.spawn((Remote, Board { game: Minigame::LastStand, ..board(9, Stage::Waiting) }, seats.clone()));
        let new = world.spawn((Remote, board(5, Stage::Waiting), seats.clone())).id();
        world.resource_mut::<AtTable>().0 = Some(Sitting {
            game: Minigame::KnockOff,
            seat: 1,
            online: true,
            id: 0,
        });
        world.resource_mut::<Kept>().passed = Some((Minigame::KnockOff, 3));
        world.run_system_once(mirror).expect("mirrored");
        let view = world.resource::<TableView>().clone();
        assert_eq!(view.shown, Some(new));
        assert_eq!(view.board, Some(board(5, Stage::Waiting)));
        assert_eq!((view.seated(), view.left(), view.playing(), view.over()), (2, 9, false, false));
        assert_eq!(world.resource::<AtTable>().0.map(|sitting| sitting.id), Some(5));
        // Away from the tables, nothing is mirrored.
        world.resource_mut::<AtTable>().0 = None;
        world.run_system_once(mirror).expect("mirrored");
        assert_eq!(*world.resource::<TableView>(), TableView::default());
    }

    #[test]
    fn online_the_table_you_are_at_told_again_is_begun_afresh_and_keeps_its_board() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = a_world();
        let board = |id: u32, stage: Stage| Board {
            game: Minigame::BridgeRace,
            id,
            stage,
            left: 0,
            people: 1,
            seats: 2,
        };
        let seats = Seats(vec![member(7, "Mochi", false), member(0, "", true)]);
        let table = world.spawn((Remote, board(4, Stage::Waiting), seats.clone())).id();
        let things = |world: &mut World| world.query_filtered::<(), With<InTable>>().iter(world).count();
        let entered = |world: &mut World| world.resource_mut::<Messages<Entered>>().drain().collect::<Vec<_>>();
        let told_again = |seat: u8| Entered {
            game: Minigame::BridgeRace,
            seat,
            online: true,
            same_table: true,
        };
        // Seated in Bridge Race's far seat, its board mirrored.
        told(&mut world, Tell::Entered { game: Minigame::BridgeRace, seat: 2 });
        world.run_system_once(mirror).expect("mirrored");
        assert_eq!(world.resource::<TableView>().shown, Some(table));
        assert_eq!(things(&mut world), 2, "the board and the game's name");
        entered(&mut world);
        // Moved up into the first seat as the computer begins to come (the
        // server's `close_ranks`): the same table, begun afresh in its new
        // seat under its own number, its board still the one shown, and
        // its name not said again.
        told(&mut world, Tell::Entered { game: Minigame::BridgeRace, seat: 0 });
        let sitting = Sitting {
            game: Minigame::BridgeRace,
            seat: 0,
            online: true,
            id: 4,
        };
        assert_eq!(world.resource::<AtTable>().0, Some(sitting));
        assert_eq!(entered(&mut world), [told_again(0)]);
        assert_eq!(world.resource::<Kept>().passed, None);
        assert_eq!(things(&mut world), 1, "the board again, and no name across the screen");
        world.run_system_once(mirror).expect("mirrored");
        assert_eq!(world.resource::<TableView>().shown, Some(table));
        assert_eq!(world.resource::<TableView>().board, Some(board(4, Stage::Waiting)));
        // Moved by being stood in another of its seats alone: the same.
        told(&mut world, Tell::Arrive { venue: Venue::BridgeRace, seat: 1, spot: None, arrival: 4 });
        assert_eq!(world.resource::<AtTable>().0, Some(Sitting { seat: 1, ..sitting }));
        assert_eq!(entered(&mut world), [told_again(1)]);
        // Stood where it has you already: nothing.
        told(&mut world, Tell::Arrive { venue: Venue::BridgeRace, seat: 1, spot: None, arrival: 5 });
        assert!(entered(&mut world).is_empty());
        // Played, and this device put back in it, its app having been put
        // away while something was said to it: still its board, played.
        *world.get_mut::<Board>(table).expect("a board") = board(4, Stage::Playing);
        world.run_system_once(mirror).expect("mirrored");
        told(&mut world, Tell::Entered { game: Minigame::BridgeRace, seat: 1 });
        assert_eq!(entered(&mut world), [told_again(1)]);
        world.run_system_once(mirror).expect("mirrored");
        assert!(world.resource::<TableView>().playing());
        assert_eq!(world.resource::<AtTable>().0, Some(Sitting { seat: 1, ..sitting }));
        // Over, and played again: another table, its name said, and the
        // one over never shown for it, though its board is here a moment.
        *world.get_mut::<Board>(table).expect("a board") = board(4, Stage::Over(Results::default()));
        world.run_system_once(mirror).expect("mirrored");
        assert!(world.resource::<TableView>().over());
        // (Stood elsewhere on its venue while it is over: no seat moved.)
        told(&mut world, Tell::Arrive { venue: Venue::BridgeRace, seat: 3, spot: None, arrival: 6 });
        assert_eq!(world.resource::<AtTable>().0, Some(Sitting { seat: 1, ..sitting }));
        assert!(entered(&mut world).is_empty());
        told(&mut world, Tell::Entered { game: Minigame::BridgeRace, seat: 0 });
        let another = Entered {
            same_table: false,
            ..told_again(0)
        };
        assert_eq!(entered(&mut world), [another]);
        assert_eq!(world.resource::<Kept>().passed, Some((Minigame::BridgeRace, 4)));
        assert_eq!(things(&mut world), 2, "the board and the game's name");
        world.run_system_once(mirror).expect("mirrored");
        assert_eq!(*world.resource::<TableView>(), TableView::default());
        let next = world.spawn((Remote, board(5, Stage::Waiting), seats)).id();
        world.run_system_once(mirror).expect("mirrored");
        assert_eq!(world.resource::<TableView>().shown, Some(next));
        assert_eq!(world.resource::<AtTable>().0.map(|sitting| sitting.id), Some(5));
    }

    #[test]
    fn online_a_table_is_over_here_once_the_server_stands_you_anywhere_but_on_its_venue() {
        // Taken into Bridge Wars from a table of Last Stand over for you, as
        // your group's leader starts one: the table is over here, as in the
        // town, and the camera is given back for Bridge Wars to take.
        let mut world = a_world();
        told(&mut world, Tell::Entered { game: Minigame::LastStand, seat: 1 });
        assert!(world.resource::<OrbitCamera>().locked);
        told(&mut world, Tell::Arrive { venue: Venue::Arena, seat: 6, spot: None, arrival: 2 });
        assert_eq!(world.resource::<AtTable>().0, None);
        let gone: Vec<Gone> = world.resource_mut::<Messages<Gone>>().drain().collect();
        assert_eq!(gone, [Gone { game: Minigame::LastStand }]);
        assert_eq!(world.query_filtered::<(), With<InTable>>().iter(&world).count(), 0);
        assert!(!world.resource::<OrbitCamera>().locked);
        // Offline, nothing the server says ends a table this device hosts.
        world.resource_mut::<AtTable>().0 = Some(Sitting {
            game: Minigame::KnockOff,
            seat: 0,
            online: false,
            id: 3,
        });
        told(&mut world, Tell::Arrive { venue: Venue::Town, seat: 5, spot: None, arrival: 3 });
        assert!(world.resource::<AtTable>().0.is_some());
    }

    #[test]
    fn a_seat_asked_of_the_server_is_waited_on_while_connected_and_only_for_a_while() {
        use bevy::ecs::system::RunSystemOnce;
        use std::time::Duration;
        let mut world = a_world();
        world.init_resource::<Messages<Ask>>();
        world.init_resource::<Messages<NetAsk>>();
        world.init_resource::<Islands>();
        world.init_resource::<Time>();
        world.insert_resource(Lobby {
            seats: Vec::new(),
            me: 1,
        });
        world.insert_resource(Online::for_tests(member(1, "You", false)));
        let asked = |world: &World| world.resource::<Kept>().asked.map(|(game, _)| game);
        // Each step a frame `secs` long, with whatever is asked in it.
        let step = |world: &mut World, secs: f32, asks: &[Ask]| {
            for &ask in asks {
                world.resource_mut::<Messages<Ask>>().write(ask);
            }
            world.resource_mut::<Time>().advance_by(Duration::from_secs_f32(secs));
            world.run_system_once(super::ask).expect("asked");
            world.resource_mut::<Messages<Ask>>().drain().for_each(drop);
        };
        // Asked of the server, and waited on.
        step(&mut world, 0.0, &[Ask::Join(Minigame::KnockOff)]);
        let sent: Vec<NetAsk> = world.resource_mut::<Messages<NetAsk>>().drain().collect();
        assert!(matches!(sent[..], [NetAsk::Join(Minigame::KnockOff)]), "{sent:?}");
        assert_eq!(asked(&world), Some(Minigame::KnockOff));
        step(&mut world, ANSWER_MOST * 0.6, &[]);
        assert_eq!(asked(&world), Some(Minigame::KnockOff));
        // Unanswered past the while: given up on, and what is turned down
        // next is not said as if it were the seat.
        step(&mut world, ANSWER_MOST * 0.6, &[]);
        assert_eq!(asked(&world), None);
        told(&mut world, Tell::Refused(Refusal::Coins));
        assert_eq!(world.query::<&Text>().iter(&world).count(), 0, "nothing said");
        // Asked again, and the connection gone: given up on at once.
        step(&mut world, 0.0, &[Ask::Join(Minigame::LastStand)]);
        assert_eq!(asked(&world), Some(Minigame::LastStand));
        world.insert_resource(Online::default());
        step(&mut world, 0.02, &[]);
        assert_eq!(asked(&world), None);
        // And answered, said once.
        world.insert_resource(Online::for_tests(member(1, "You", false)));
        step(&mut world, 0.0, &[Ask::Join(Minigame::BridgeRace)]);
        told(&mut world, Tell::Refused(Refusal::Coins));
        assert_eq!(asked(&world), None);
        let said: Vec<String> = world.query::<&Text>().iter(&world).map(|text| text.0.clone()).collect();
        assert_eq!(said, [ui().words.games.refused_coins.clone()]);
    }

    #[test]
    fn everyone_waits_and_begins_where_the_server_stands_them() {
        // Waiting in Bridge Race's and Last Stand's rooms, facing in; on
        // Knock Off's ice where play begins too.
        for (game, room, floor) in [
            (Minigame::BridgeRace, bridge_race::WAITING_ROOM, bridge_race::WAITING_FLOOR),
            (Minigame::LastStand, last_stand::WAITING_ROOM, last_stand::WAITING_FLOOR),
        ] {
            for seat in 0..4 {
                let place = waiting_place(game, seat);
                assert_eq!(place.translation.y, floor);
                let ahead = (place.rotation * Vec3::NEG_Z).xz();
                let inward = (room - place.translation.xz()).normalize();
                assert!(ahead.dot(inward) > 0.999, "{game:?} {seat}: {ahead} against {inward}");
            }
        }
        for seat in 0..4 {
            assert_eq!(waiting_place(Minigame::KnockOff, seat), starting_place(Venue::KnockOff, seat));
            // And play begins on the land as lifted.
            assert_eq!(starting_place(Venue::BridgeRace, seat).translation.y, bridge_race::LIFT);
            assert_eq!(starting_place(Venue::LastStand, seat).translation.y, last_stand::LIFT);
        }
    }
}
