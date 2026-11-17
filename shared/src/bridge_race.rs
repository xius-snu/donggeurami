//! Bridge Race, to the golden island (since 2026-11-18; Hajun: "Like King of
//! the Ladder but with bridge building. You start on two islands next to each
//! other, and there is a golden island at equidistance from each island like
//! the tip of a 80deg 80deg 20deg isosceles triangle. Every interval you are
//! given more bridge meters, and sometimes u get dynamite as well, randomly.
//! Your goal is to reach the golden island first compared to the other team -
//! both of u need to make it across cuz its 2v2 ... for now, i want 2 ai bots
//! to be on the same team, and the other team is real players either 1~2
//! player ... Map = bridgeracemap"): two teams, each on its own island over
//! the void, bridging to the golden island, the computer making up the side
//! people do not, on `bridgeracemap.glb`
//! ([`Venue::BridgeRace`](crate::Venue::BridgeRace)). Its rules are in this
//! module, and the server and the device both play by them, as by Bridge
//! Wars' in [`crate::bridge_wars`]; how its table fills, starts and pays is
//! [`crate::table`]'s.
//!
//! **The game is a plain [`Game`]**, stepped by whoever runs it: the server
//! online, the device in a game played offline. Both feed it the same
//! things (where each person is, what each of them does) and carry out the
//! same [`Event`]s, so that the two never differ. In it:
//!
//! * **Metres**: everyone begins with [`START_METRES`] of bridge, and every
//!   [`GRANT_EVERY`] seconds each team is given [`TEAM_GRANT`] more, shared
//!   between those of its players still in the game ([`team_share`];
//!   "Every interval you are given more bridge meters"). A person's are
//!   counted on their own device, as in Bridge Wars, which lays their bridge
//!   and pays for it; the room is told how many grants there have been and
//!   what the last gave each team's players ([`RaceRound::grants`],
//!   [`RaceRound::share`]). The computer's the game counts itself.
//! * **Dynamite**, now and then ("sometimes u get dynamite as well,
//!   randomly"): with each grant one chance in five of a stick for each
//!   player, two at the most, on their button. Thrown as in Bridge Wars, and
//!   where it blows the planks its blast reaches go and **no more** (mine: no
//!   falling of the rest of the bridge after them, which in a race would
//!   decide it at the first stick), and whoever of the other team is in it
//!   is knocked away from it as a punch on the run knocks (mine: the other
//!   team's alone, as Bridge Wars' blast reaches only the other teams and a
//!   punch never lands on a teammate; the planks go from under anyone).
//! * **Punches** knock and do nothing else ("You can punch each other off the
//!   map"): nobody has health. The void is the only way to die: fall into it
//!   and you are back on your own island's middle a moment later ("ur
//!   respawn is the center of ur own player island"), with your metres and
//!   your sticks (mine).
//! * **Winning**: every one of a team on the golden island at once ("both of
//!   u need to make it across cuz its 2v2"), together for [`HOLD`] seconds
//!   (mine, "Like King of the Ladder": a ring round the island fills in the
//!   team's colour while they hold it, and empties the moment one of them is
//!   off it). At [`PLAY_SECS`] the team with more of its players on the
//!   golden island wins, or else the one whose bridge has got nearer it, or
//!   it is a draw.
//!
//! The computer's players are in [`brain`]: each one of several kinds, every
//! threshold of theirs a spread thrown again (Hajun: "if some thing becomes a
//! definite if-then, then its not a real human, but a program").

pub mod brain;
#[cfg(test)]
mod tests;

use std::f32::consts::TAU;
use std::sync::{Arc, LazyLock};

use bevy::prelude::{App, Component, Quat, Vec2, Vec3, Vec3Swizzles};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::bots::{Dice, Seen, Stick};
use crate::bridge_wars::{DYNAMITE_FUSE, DYNAMITE_RADIUS, FALL_TO_DIE, HURT_TIME, Lob, REACH, WAIT_RING, caught, in_dynamite};
use crate::chat::CREDIT_SECS;
use crate::nav::{BODY, Grid, Nav, PLANK_HALF_WIDTH, Region};
use crate::{Dynamite, Kit, Plank, Utility, table};

use brain::{Crew, View};

// ------------------------------------------------------------------ the map

/// The map as the computer's players know it, as last baked out of
/// `bridgeracemap.glb` by the game's own collision (the game's
/// `race_bake`, a test, which fails when the map has changed since).
static GRID: LazyLock<Arc<Grid>> = LazyLock::new(|| {
    Arc::new(
        Grid::decode(include_bytes!("race.nav"))
            .expect("race.nav is a baked grid: bake it again (RT_BAKE_NAV=1 cargo test --lib race_bake)"),
    )
});

/// Where a body can stand and walk on Hajun's `bridgeracemap.glb`, half a
/// metre to a square, lifted as the game stands it ([`LIFT`]): the two
/// teams' islands (`Region::Team` of each team) and the golden island
/// (`Region::Middle`), with the void between them. The server has no map,
/// and its game and its computer's players go by this; so does the device's
/// in a game played offline.
pub fn grid() -> Arc<Grid> {
    GRID.clone()
}

/// How far up the map is stood as the device loads it, in metres (the
/// game's `island::lift`): Hajun's `bridgeracemap.glb` has its land at 0
/// where Bridge Wars' `pvptownmap.glb` has it at 0.25, and every rule
/// written for Bridge Wars' heights (a plank's top, a pad's, how far a body
/// falls before it is dead) goes by 0.25. Lifted, its islands' tops are at
/// 0.25, its pads' at 0.5, and the waiting room's floor at
/// [`WAITING_FLOOR`].
pub const LIFT: f32 = 0.25;

/// How many teams there are: red and blue.
pub const TEAMS: usize = 2;

/// Which team the player in `seat` is on: 0, red, and 1, blue, seat by seat
/// in turn, so that the first two seats are one against one.
pub fn team(seat: usize) -> u8 {
    (seat % 2) as u8
}

/// Where each team's island is, by team, in metres from the middle of the
/// map: red's at (-80, -21) and blue's at (-80, 21), 42 m apart, as Hajun's
/// map has them (a test in the game reads them off it).
pub const ISLANDS: [Vec2; 2] = [Vec2::new(-80.0, -21.0), Vec2::new(-80.0, 21.0)];

/// Where the golden island is: the middle of the map, 83 m from each team's
/// island ("like the tip of a 80deg 80deg 20deg isosceles triangle").
pub const GOLDEN: Vec2 = Vec2::ZERO;

/// How far each of the three islands reaches from its middle, in metres:
/// discs 20 m across.
pub const ISLAND_RADIUS: f32 = 10.0;

/// How far it is from a team's island to the golden island, shore to
/// shore, in metres: the void each team bridges, 62.7 m. What how far a
/// bridge has got is a share of ([`RaceRound::across`]).
pub fn the_way_across(team: u8) -> f32 {
    ISLANDS[usize::from(team) % TEAMS].distance(GOLDEN) - 2.0 * ISLAND_RADIUS
}

/// Where whoever is in `seat` stands again after falling into the void, and
/// which way they face: the middle of their own team's island ("ur respawn
/// is the center of ur own player island"), where the first of the team
/// arrives, facing the golden island.
pub fn home(seat: usize) -> (Vec3, f32) {
    let (at, turn) = arrival(usize::from(team(seat)));
    (at, yaw_of(turn))
}

/// The turn about the upright that `turn` is, as a body's yaw: -Z turned by
/// it is the way it faces.
fn yaw_of(turn: Quat) -> f32 {
    let ahead = turn * Vec3::NEG_Z;
    (-ahead.x).atan2(-ahead.z)
}

// ---------------------------------------------------------------- the table

/// How many seats a table has with `people` people at it once the computer
/// has come: two for one person, one against one ("If there is only 1 real
/// player, then its a 1v1 against bots"), and four for anyone else, two
/// against two ("if its 2 players then its a 2v2 against bots"); four for
/// nobody too, the computer playing both sides.
pub fn seats_for(people: usize) -> usize {
    if people == 1 { 2 } else { 4 }
}

/// The order people take seats in, the first to join first: the first two
/// on one team, so that two people are one team against the computer ("if
/// its 2 players then its a 2v2 against bots"), a third on the other, with
/// the computer, and a fourth beside them.
pub const PEOPLE_FIRST: [usize; 4] = [0, 2, 1, 3];

/// The waiting room, Hajun's glass box over the map between the two teams'
/// islands: its middle across the ground, and its floor, lifted with the
/// rest of the map ([`LIFT`]; a test in the game reads it off the map).
pub const WAITING_ROOM: Vec2 = Vec2::new(-40.0, 0.0);
pub const WAITING_FLOOR: f32 = 120.5 + LIFT;

/// Where the player in `seat` waits for the game: on the waiting room's
/// floor, on a ring round its middle as wide as Bridge Wars' room's, each a
/// quarter of the turn on from the last.
pub fn waiting_spot(seat: usize) -> Vec3 {
    let angle = seat as f32 * TAU / SHAPE.most as f32;
    Vec3::new(
        WAITING_ROOM.x + angle.sin() * WAIT_RING,
        WAITING_FLOOR,
        WAITING_ROOM.y + angle.cos() * WAIT_RING,
    )
}

/// How a game of Bridge Race fills, starts and pays: four seats at the
/// most, two teams of two, the computer making up what people do not 10 s
/// after the last person joined, as in Bridge Wars ("10second later it will
/// fill with bots, similar to how bridge wars is working now"); 50 coins
/// to join.
pub const SHAPE: table::Shape = table::Shape {
    most: 4,
    fewest: 1,
    computer: true,
    wait: 10.0,
    countdown: 5.0,
    results: 15.0,
    fee: 50,
};

// ---------------------------------------------------------------- the rules

/// How many metres of bridge everyone begins with, how many more each team
/// is given every so often, shared evenly between those of its players
/// still in the game, and how often, in seconds of play ("Every interval
/// you are given more bridge meters"; the numbers are ours, tuned by whole
/// games, `many_races`). A team's bridge comes on as fast with one of a
/// side as with two ([`team_share`]), and with one left of two, and a game
/// is about two and a half minutes at the median either way, the computer
/// laying its way all the way over: with each player given 5 m every 10 s
/// whatever their team, as first written, two against two was over in
/// under a minute, and one against one in under two.
pub const START_METRES: f32 = 10.0;
pub const TEAM_GRANT: f32 = 6.0;
pub const GRANT_EVERY: f32 = 16.0;

/// How many metres of bridge each of a team of `players` still in the game
/// is given with every grant ([`GRANT_EVERY`]): the team's [`TEAM_GRANT`]
/// shared between them, 6 m for one alone and 3 m each for two. Whoever
/// has left for good ([`Game::leave`]) leaves their share to whoever of
/// their team is left (mine: a team does not race at half pace for having
/// lost someone).
pub fn team_share(players: usize) -> f32 {
    TEAM_GRANT / players.max(1) as f32
}

/// How many metres of bridge each player of a table of `seats` seats is
/// given with every grant while everyone is still in the game: their
/// team's [`TEAM_GRANT`] shared between however many each team has, 6 m
/// each one against one and 3 m each two against two ([`team_share`]).
/// What each grant gives is in the round as it comes ([`RaceRound::share`]).
pub fn grant_metres(seats: usize) -> f32 {
    team_share(seats / TEAMS)
}

/// How likely each player is to be given a stick of dynamite with each
/// grant, and the most sticks anyone holds ("sometimes u get dynamite as
/// well, randomly"; the numbers are ours): about one every minute and a
/// third each, and not so many held that a team could take a whole bridge
/// down.
pub const DYNAMITE_ODDS: f32 = 0.2;
pub const DYNAMITE_MOST: u8 = 2;

/// How long every one of a team has to stand on the golden island together
/// to win it, in seconds (mine, "Like King of the Ladder": long enough for
/// the other team to come and knock one of them off, short enough that
/// getting there all together is most of it).
pub const HOLD: f32 = 3.0;

/// How long a game is played at the most, in seconds: six minutes (mine).
/// Then whoever has more of its players on the golden island wins, or else
/// the team whose bridge has got nearer it, or it is a draw.
pub const PLAY_SECS: f32 = 360.0;

/// How long after falling into the void someone is back on their island's
/// middle, in seconds, counted from when they are dead, [`FALL_TO_DIE`]
/// under the land (mine: a death costs the walk back, and that is enough).
pub const RESPAWN: f32 = 2.0;

/// How high the land of the islands is, as the game stands the map: what
/// a fall is measured from.
pub const LAND_TOP: f32 = LIFT;

/// How far over the golden island's top someone's feet may be for them to
/// be standing on it, in metres: a step's worth, and not in the air.
pub const ON_GOLD: f32 = 0.5;

/// How far apart a person who punches someone and whoever they hit may be
/// where each last said they stood, in metres, for the punch to count: the
/// reach, and the way each may have gone since (Bridge Wars' own).
pub const PUNCH_SLACK: f32 = 6.5;

/// How far from where a person last said they stood a rung they lay may be,
/// in metres: a plank or two ahead of their feet, and the way they may have
/// gone since, with plenty in hand, so that no rung of a real bridge is
/// ever turned away (the rest of the room would not see it, and its
/// layer would) while one laid from across the map is.
pub const RUNG_SLACK: f32 = 15.0;

/// The most rungs one seat can lay, and how far from the middle of the map
/// one can be: far more than a game's worth, and the edge of the map and a
/// good deal more, so that a device gone wrong cannot fill the game's
/// memory (Bridge Wars' own, near enough).
const MOST_RUNGS: usize = 600;
const MIDST: Vec2 = Vec2::new(-40.0, 0.0);
const FURTHEST: f32 = 120.0;
/// The longest a plank is counted as, in metres, of what each has laid: a
/// rung further than this from the one before it is no plank (the map's
/// own measure, `nav`).
const LONGEST_PLANK: f32 = 6.0;
/// The longest step of play, in seconds: a frame held up longer than this
/// plays on as if it were this long.
const LONGEST_STEP: f32 = 0.25;

/// How many grants have come by `elapsed` seconds of play: one every
/// [`GRANT_EVERY`], the first as the first sixteen seconds are up.
pub fn grants_by(elapsed: f32) -> u16 {
    (elapsed.max(0.0) / GRANT_EVERY).floor().min(f32::from(u16::MAX)) as u16
}

// ------------------------------------------------------------ what is sent

/// A game of Bridge Race as the room is sent it: on the table's own entity
/// beside its board ([`table::Board`]), its seats and everyone's kit
/// ([`crate::Kits`], where each stick of dynamite shows on its owner's
/// button), and re-inserted only when it changes: once a second for the
/// clock, a tenth of [`HOLD`] at a time while a team holds the golden
/// island, and as bridges get further across. A dozen bytes.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct RaceRound {
    /// Whole seconds of play left, rounded up.
    pub left: u16,
    /// How many grants of metres have come ([`GRANT_EVERY`]): a person's
    /// device adds what each gives its team's players ([`RaceRound::share`])
    /// to what it has, for each it has not counted yet.
    pub grants: u16,
    /// What the last grant gave each of each team's players, by team, in
    /// tenths of a metre ([`team_share`]: 30 two against two, 60 one against
    /// one or for one left of two); before the first, what it will give.
    /// [`RaceRound::share_of`] has it in metres.
    pub share: [u8; 2],
    /// Each team's ring round the golden island, in tenths of [`HOLD`],
    /// from 0 to 10: full is won.
    pub hold: [u8; 2],
    /// How many of each team stand on the golden island.
    pub on_gold: [u8; 2],
    /// The team that won, once it is over; none while it is played, and none
    /// for a draw ([`RaceRound::over`]).
    pub winner: Option<u8>,
    /// Whether it is over.
    pub over: bool,
    /// How far each team's bridge has got toward the golden island, in
    /// hundredths of the way across ([`the_way_across`]), by its plank
    /// nearest it: 100 once it is there. What the time limit goes by when
    /// nobody is on the island (mine: so that what everyone sees on their
    /// screen is what decides it).
    pub across: [u8; 2],
}

impl RaceRound {
    /// What the last grant gave each player of `team`, in metres
    /// ([`RaceRound::share`]): what a person's device adds for each grant
    /// it has not yet counted.
    pub fn share_of(&self, team: u8) -> f32 {
        f32::from(self.share[usize::from(team) % TEAMS]) / 10.0
    }
}

/// Something asked in a game of Bridge Race ([`Ask::BridgeRace`](crate::Ask::BridgeRace)).
/// A punch, a rung, a stick thrown and a fall go as they do in Bridge Wars
/// ([`crate::Ask::Punch`], [`crate::Ask::Rung`], [`crate::Ask::Use`],
/// [`crate::Ask::Respawn`]).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ask {
    /// Nothing: the placeholder this game had before its own asks came,
    /// kept first so that whatever was written against it still reads as
    /// it did. No device sends it, and the game takes it for nothing.
    Nothing,
    /// How many metres of bridge I have left to lay, and how many of the
    /// game's grants ([`RaceRound::grants`]) they count: my own device
    /// counts them, as Bridge Wars' does, laying my bridge a plank at a
    /// time and adding each grant as it comes ([`RaceRound::share_of`] my
    /// team). Said as they change, no oftener than once a second. The game
    /// keeps it to give back to me if I come back into the game on a device
    /// that has forgotten ([`Tell::Metres`]); nothing else goes by it.
    Metres { metres: f32, grants: u16 },
}

/// Something told one device in a game of Bridge Race
/// ([`Tell::BridgeRace`](crate::Tell::BridgeRace)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// Nothing: the placeholder this game had before its own tells came,
    /// kept first as [`Ask::Nothing`] is. Never sent.
    Nothing,
    /// You are back in your game after your connection went: the metres
    /// of bridge you had left as your device last said, with every grant
    /// since added, and how many grants that counts ([`Game::kept`]).
    Metres { metres: f32, grants: u16 },
    /// Something has happened in your game that its chat says ([`Feed`]):
    /// told every device in the room that is connected, as it happens, and
    /// to nobody whose device has gone (a line of the chat missed is no
    /// reason to be put back in the game afresh).
    Feed(Feed),
}

/// What the chat of a game of Bridge Race says as it goes (the game's
/// `bridge_race`, which writes each in the file's words, names in their
/// teams' colours): online the server tells each device ([`Tell::Feed`]),
/// offline the device's own game says the same, both by [`feed`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Feed {
    /// Whoever is in `seat` has fallen into the void, knocked off by whoever
    /// is in `by` if anyone of the other team had a hand in it
    /// ([`Event::Fell`]).
    Fell { seat: u8, by: Option<u8> },
    /// The stick of dynamite of whoever is in `by` has blown, taking
    /// `planks` planks, most of them of `team`'s bridges (of the thrower's
    /// own only if it took none of the other team's); none, if it took
    /// none ([`Event::Blew`]).
    Blew { by: u8, team: Option<u8>, planks: u16 },
}

/// What the chat says of `event`, if it says anything of it: a fall, and a
/// stick blown. The grants, the sticks given, the bridges joined to the
/// golden island and a team holding it are said by every device off the
/// round and the kits it is sent ([`RaceRound`], [`crate::Kits`]), which
/// come to everyone alike.
pub fn feed(event: &Event) -> Option<Feed> {
    match event {
        Event::Fell { seat, by } => Some(Feed::Fell {
            seat: *seat as u8,
            by: by.map(|by| by as u8),
        }),
        Event::Blew { by, planks, .. } => {
            let mut of = [0u16; TEAMS];
            for &(seat, ..) in planks {
                of[usize::from(team(usize::from(seat)))] += 1;
            }
            let theirs = usize::from(1 - team(*by));
            let side = if of[theirs] > 0 { theirs } else { 1 - theirs };
            Some(Feed::Blew {
                by: *by as u8,
                team: (of[side] > 0).then_some(side as u8),
                planks: planks.len().min(usize::from(u16::MAX)) as u16,
            })
        }
        _ => None,
    }
}

/// Registers this game's own replicated components: its round.
pub fn register(app: &mut App) {
    app.replicate::<RaceRound>();
}

// ---------------------------------------------------------------- the game

/// How a game of Bridge Race came to be over.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Finish {
    /// A team held the golden island, every one of it, for [`HOLD`]; or
    /// both did in the same moment, a draw.
    Held,
    /// The time ran out ([`PLAY_SECS`]).
    Time,
    /// Every one of a team left the game, which the other wins; or of both.
    Left,
}

/// Something that has happened in a game, for whoever runs it to carry
/// out: the server online, telling and sending the room what it has to; the
/// device offline, drawing it.
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    /// A rung of a bridge, the game's from now on: whoever is to see the
    /// plank between it and the one before it is to be sent it (online a
    /// replicated [`Plank`]). Of the computer's players, and of a person,
    /// whose own device has it already.
    Plank(Plank),
    /// Whoever is in `victim` has been knocked by whoever is in `by`, from
    /// `from`, on the run if `running`: by a punch, or by `by`'s stick of
    /// dynamite blowing at `from`. A person's own device knocks their body
    /// ([`crate::Tell::Knocked`]); the computer's the game has knocked
    /// already.
    Knocked { victim: usize, by: usize, from: Vec3, running: bool },
    /// A stick of dynamite has been thrown, the game's number `stick`: the
    /// room is to be sent it as it is ([`Dynamite`]), every device lobbing
    /// it on from there.
    Thrown { stick: u32, shown: Dynamite },
    /// The stick numbered so came down where there was no ground for it,
    /// and is gone into the void, unblown.
    Lost { stick: u32 },
    /// The stick numbered so, whoever in `by` threw it, has blown at `at`,
    /// taking these planks, each as whose, which of their bridges and which
    /// plank of that one ([`Nav::blast`]): no more, with nothing after them
    /// (online a [`crate::Blasted`] with no `later`).
    Blew { stick: u32, by: usize, at: Vec3, planks: Vec<(u8, u16, u16)> },
    /// The grant numbered `grant` has come: [`grant_metres`] for everyone,
    /// and a stick of dynamite for each of `sticks`, by seat, now on their
    /// button.
    Granted { grant: u16, sticks: Vec<usize> },
    /// Whoever is in `seat` has fallen into the void, knocked off by whoever
    /// is in `by` if they last hit them no longer ago than
    /// [`CREDIT_SECS`].
    Fell { seat: usize, by: Option<usize> },
    /// Whoever is in `seat` stands again, `at` on their island's middle,
    /// facing `yaw`: a person is to be put there, as they are by the server
    /// arriving anywhere; the computer's is there.
    Back { seat: usize, at: Vec3, yaw: f32 },
    /// `team` has every one of its players on the golden island, and its
    /// ring begins to fill; or, not `holding`, no longer, and it is empty.
    Holding { team: u8, holding: bool },
    /// The game is over: won by `winner`, or a draw, so.
    Over { winner: Option<u8>, finish: Finish },
}

/// A stick of dynamite on its way, or lying with its fuse burning: its
/// number, its lob, whose it is, by seat, and whether it has come down
/// onto ground.
#[derive(Clone, Copy, Debug)]
struct Lobbed {
    id: u32,
    lob: Lob,
    by: usize,
    lies: bool,
}

/// How a game went, for telling, over whole games, how they go: counts of
/// what happened in it, and when things first did.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Tally {
    /// Punches and blasts that landed on someone.
    pub knocks: u32,
    /// Falls into the void, all told, and how many of them someone of the
    /// other team had a hand in.
    pub falls: u32,
    pub knocked_off: u32,
    /// Sticks of dynamite given, thrown, gone into the void unblown, and
    /// blown; and how many planks the ones that blew took.
    pub given: u32,
    pub thrown: u32,
    pub lost: u32,
    pub blown: u32,
    pub planks_blown: u32,
    /// When each team's island was first joined to the golden island, in
    /// seconds of play; and when each team first stood on it all together.
    pub joined: [Option<f32>; 2],
    pub first_hold: [Option<f32>; 2],
    /// How many times each team has begun to hold the golden island.
    pub holds: [u32; 2],
}

/// A game of Bridge Race at a table of two or four seats, from the
/// countdown's end ([`Game::begin`]) to its own ([`Game::outcome`]): a
/// plain function of what it is fed, so that a whole game is a test.
#[derive(Clone, Debug)]
pub struct Game {
    /// Whose each seat is: a person's, or the computer's.
    people: Vec<bool>,
    /// The seats whose people have left for good: no longer anyone the
    /// golden island waits for.
    gone: Vec<bool>,
    /// Where each body stands, feet, and which way it faces: a person's as
    /// their device last said ([`Game::sync`]), the computer's as the game
    /// moves it.
    at: Vec<Vec3>,
    yaw: Vec<f32>,
    /// How long until whoever has fallen into the void stands again, for
    /// each seat that has.
    down: Vec<Option<f32>>,
    /// How long until a punch takes on each again ([`HURT_TIME`]), and who
    /// last hit each and when, in seconds of play.
    hurt: Vec<f32>,
    last_hit: Vec<Option<(usize, f32)>>,
    /// What each has on their buttons: their sticks of dynamite.
    kits: Vec<Kit>,
    /// What a person's device last said of their metres, and of how many
    /// grants ([`Ask::Metres`]).
    said: Vec<(f32, u16)>,
    /// How many metres of bridge each has laid, all told: their score.
    laid: Vec<f32>,
    /// How many rungs each has had taken.
    rungs: Vec<usize>,
    /// Every bridge heard of, as whose, by seat, and which of theirs; and
    /// for each, how many of its planks have been counted in
    /// [`Game::laid`].
    chains: Vec<(u8, u16)>,
    counted: Vec<usize>,
    /// What each grant so far gave each of each team's players, by grant
    /// and by team ([`team_share`]): what a person's metres are reckoned
    /// by, coming back ([`Game::kept`]).
    shares: Vec<[f32; TEAMS]>,
    /// Whether each of the computer's players carried a plank, the frame
    /// before.
    carrying: Vec<bool>,
    nav: Nav,
    crew: Crew,
    sticks: Vec<Lobbed>,
    next_stick: u32,
    dice: Dice,
    begun: bool,
    elapsed: f32,
    grants: u16,
    /// How long each team has held the golden island, all of it together,
    /// in seconds; whether it does now; and how many of each are on it.
    hold: [f32; TEAMS],
    holding: [bool; TEAMS],
    on_gold: [u8; TEAMS],
    /// How far each team's bridge has got ([`RaceRound::across`]), and the
    /// map's [`Nav::version`] that was worked out as of.
    across: [u8; TEAMS],
    across_of: Option<u32>,
    /// Who won, how, and when, once it is over.
    over: Option<(Option<u8>, Finish, f32)>,
    events: Vec<Event>,
    tally: Tally,
}

impl Game {
    /// A new game at a table of `seats` seats, two or four; `people[seat]`
    /// false for the computer's, which take their seats now, each as one of
    /// its kinds ([`brain::KINDS`]). The same `seed`, the same game.
    pub fn new(seats: usize, people: &[bool], seed: u64) -> Self {
        let seats = seats.clamp(1, SHAPE.most);
        let people: Vec<bool> = (0..seats).map(|seat| people.get(seat).copied().unwrap_or(false)).collect();
        let nav = Nav::new(grid());
        let mut crew = Crew::new(seed ^ 0x5851_F42D_4C95_7F2D, nav.grid());
        let mut at = Vec::with_capacity(seats);
        let mut yaw = Vec::with_capacity(seats);
        for (seat, &person) in people.iter().enumerate() {
            let (spot, turn) = arrival(seat);
            at.push(spot);
            yaw.push(yaw_of(turn));
            if !person {
                crew.seat(seat, spot, yaw_of(turn));
            }
        }
        Self {
            gone: vec![false; seats],
            at,
            yaw,
            down: vec![None; seats],
            hurt: vec![0.0; seats],
            last_hit: vec![None; seats],
            kits: vec![Kit::default(); seats],
            said: vec![(START_METRES, 0); seats],
            laid: vec![0.0; seats],
            rungs: vec![0; seats],
            chains: Vec::new(),
            counted: Vec::new(),
            shares: Vec::new(),
            carrying: vec![false; seats],
            people,
            nav,
            crew,
            sticks: Vec::new(),
            next_stick: 0,
            dice: Dice::new(seed),
            begun: false,
            elapsed: 0.0,
            grants: 0,
            hold: [0.0; TEAMS],
            holding: [false; TEAMS],
            on_gold: [0; TEAMS],
            across: [0; TEAMS],
            across_of: None,
            over: None,
            events: Vec::new(),
            tally: Tally::default(),
        }
    }

    /// The countdown is over: everyone stands where they arrive on their
    /// team's island ([`arrival`]), a person put there by whoever runs the
    /// game as the play begins, and the clock starts.
    pub fn begin(&mut self) {
        if self.begun {
            return;
        }
        self.begun = true;
        for seat in 0..self.seats() {
            let (spot, turn) = arrival(seat);
            self.at[seat] = spot;
            self.yaw[seat] = yaw_of(turn);
            if !self.people[seat] {
                self.crew.stand(seat, spot, self.yaw[seat], 0.0);
            }
        }
    }

    /// How many seats the table has.
    pub fn seats(&self) -> usize {
        self.people.len()
    }

    /// Whether whoever is in `seat` is a person.
    pub fn person(&self, seat: usize) -> bool {
        self.people.get(seat).copied().unwrap_or(false)
    }

    /// Whether the game is being played: begun, and not over.
    pub fn playing(&self) -> bool {
        self.begun && self.over.is_none()
    }

    /// Seconds of play so far.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Whether whoever is in `seat` is in the game and on their feet: not
    /// left for good, and not fallen into the void.
    pub fn standing(&self, seat: usize) -> bool {
        seat < self.seats() && !self.gone[seat] && self.down[seat].is_none()
    }

    /// Where a person's body is now, as their device last said, and which
    /// way it faces. The computer's are the game's own, and nothing anyone
    /// says moves them.
    pub fn sync(&mut self, seat: usize, at: Vec3, yaw: f32) {
        if seat < self.seats() && self.people[seat] && at.is_finite() && yaw.is_finite() {
            self.at[seat] = at;
            self.yaw[seat] = yaw;
        }
    }

    /// Whoever is in `seat` has laid this rung of a bridge ([`crate::Ask::Rung`]):
    /// the `index`th of their bridge numbered `chain`, `at` across the
    /// ground, heading so. Taken, if it could be theirs: while the game is
    /// played, from a person in it, no further from where they last said
    /// they stood than [`RUNG_SLACK`], on the map, and within the most
    /// anyone lays ([`Event::Plank`]). A person's metres are their device's
    /// to count, as in Bridge Wars. Whether it was taken.
    pub fn rung(&mut self, seat: usize, chain: u16, index: u16, at: Vec2, heading: f32) -> bool {
        if !self.playing() || seat >= self.seats() || !self.people[seat] || self.gone[seat] {
            return false;
        }
        if !at.is_finite() || !heading.is_finite() || self.rungs[seat] >= MOST_RUNGS || at.distance(MIDST) > FURTHEST {
            return false;
        }
        if at.distance(self.at[seat].xz()) > RUNG_SLACK {
            return false;
        }
        self.take_rung(seat, chain, index, at, heading);
        true
    }

    /// A rung laid by whoever is in `seat`, taken: onto the map, counted in
    /// what they have laid, and told; one the map has had already (a device
    /// back after its connection went sends again what it had not heard was
    /// sent) is nothing new, and is not told again.
    fn take_rung(&mut self, seat: usize, chain: u16, index: u16, at: Vec2, heading: f32) {
        if usize::from(index) < self.nav.rungs(seat as u8, chain).len() {
            return;
        }
        self.rungs[seat] += 1;
        self.nav.hear(seat as u8, chain, index, at);
        self.events.push(Event::Plank(Plank {
            seat: seat as u8,
            chain,
            index,
            x: at.x,
            z: at.y,
            heading,
        }));
        // What it adds to their bridge, counted once: the planks it has
        // brought in, with any that waited for it.
        let found = match self.chains.iter().position(|&(of, number)| usize::from(of) == seat && number == chain) {
            Some(found) => found,
            None => {
                self.chains.push((seat as u8, chain));
                self.counted.push(0);
                self.chains.len() - 1
            }
        };
        let rungs = self.nav.rungs(seat as u8, chain);
        let counted = self.counted[found];
        let planks = rungs.len().saturating_sub(1);
        for plank in counted..planks {
            let length = rungs[plank].distance(rungs[plank + 1]);
            if length <= LONGEST_PLANK {
                self.laid[seat] += length;
            }
        }
        self.counted[found] = planks.max(counted);
    }

    /// A punch by the person in `hitter` landed on whoever is in `victim`,
    /// `running` if they were on the move ([`crate::Ask::Punch`]): they are
    /// knocked back from where the hitter stands, as in Bridge Wars, and
    /// nothing more. Not a teammate, never; not someone fallen, or the
    /// hitter fallen; not from further than [`PUNCH_SLACK`]; and not while
    /// the last punch that took still smarts ([`HURT_TIME`]: a device lands
    /// one at every press). Whether it took.
    pub fn punch(&mut self, hitter: usize, victim: usize, running: bool) -> bool {
        if !self.playing() || hitter == victim || hitter >= self.seats() || victim >= self.seats() || !self.people[hitter] {
            return false;
        }
        if !self.standing(hitter) || !self.standing(victim) || team(hitter) == team(victim) {
            return false;
        }
        if self.at[hitter].xz().distance(self.at[victim].xz()) > PUNCH_SLACK {
            return false;
        }
        self.knock(victim, hitter, self.at[hitter], running, false)
    }

    /// Knocks whoever is in `victim` from `from`, by whoever is in `by`:
    /// the computer's here, a person's by their own device
    /// ([`Event::Knocked`]). A punch takes only once the last has stopped
    /// smarting; a blast whatever.
    fn knock(&mut self, victim: usize, by: usize, from: Vec3, running: bool, blast: bool) -> bool {
        if !blast && self.hurt[victim] > 0.0 {
            return false;
        }
        self.hurt[victim] = HURT_TIME;
        self.last_hit[victim] = Some((by, self.elapsed));
        self.tally.knocks += 1;
        if !self.people[victim] {
            self.crew.knock(victim, from.xz(), running);
        }
        self.events.push(Event::Knocked {
            victim,
            by,
            from,
            running,
        });
        true
    }

    /// The person in `seat` throws a stick of dynamite the way `heading`
    /// says, to come down `far` from them ([`crate::Use::Dynamite`]), if
    /// they have one on their button and are on their feet: it is in the
    /// air ([`Event::Thrown`]), and the game lobs it on and blows it.
    /// Whether it was thrown.
    pub fn throw(&mut self, seat: usize, heading: f32, far: f32) -> bool {
        if !self.playing() || seat >= self.seats() || !self.people[seat] || !self.standing(seat) {
            return false;
        }
        self.lob(seat, heading, far)
    }

    /// Whoever is in `seat` throws a stick, if they have one: thrown at the
    /// ground where it is to come down, as high as that is, or, with none
    /// there, as high as the ground they stand on (Bridge Wars' own).
    fn lob(&mut self, seat: usize, heading: f32, far: f32) -> bool {
        if !heading.is_finite() || !far.is_finite() {
            return false;
        }
        let held = self.kits[seat].held(Utility::Dynamite);
        if held == 0 {
            return false;
        }
        self.kits[seat] = with_sticks(self.kits[seat], held - 1);
        let feet = self.at[seat];
        let aimed = Lob::thrown(feet, heading, far, feet.y);
        let floor = self.nav.ground(aimed.to.xz()).or(self.nav.ground(feet.xz())).unwrap_or(feet.y);
        let lob = Lob::thrown(feet, heading, far, floor);
        let stick = self.next_stick;
        self.next_stick = self.next_stick.wrapping_add(1);
        self.events.push(Event::Thrown {
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
        self.tally.thrown += 1;
        true
    }

    /// The person in `seat` has fallen into the void, as their own device
    /// says ([`crate::Ask::Respawn`]): they stand on their island's middle
    /// again [`RESPAWN`] seconds from now ([`Event::Back`]).
    pub fn fell(&mut self, seat: usize) {
        if self.playing() && seat < self.seats() && self.people[seat] && self.standing(seat) {
            self.fall(seat);
        }
    }

    /// Whoever is in `seat` is dead in the void: told, credited to whoever
    /// last hit them if that was lately, and stood again in a moment.
    fn fall(&mut self, seat: usize) {
        let by = self.last_hit[seat]
            .filter(|&(hitter, when)| hitter != seat && team(hitter) != team(seat) && self.elapsed - when <= CREDIT_SECS)
            .map(|(hitter, _)| hitter);
        self.down[seat] = Some(RESPAWN);
        self.last_hit[seat] = None;
        self.tally.falls += 1;
        self.tally.knocked_off += u32::from(by.is_some());
        self.events.push(Event::Fell { seat, by });
    }

    /// What the device of the person in `seat` last said of their metres
    /// ([`Ask::Metres`]): kept, if it could be so, to give back to them if
    /// they come back to the game ([`Game::kept`]). No more than they could
    /// have, and no grant that has not come.
    pub fn said(&mut self, seat: usize, metres: f32, grants: u16) {
        if seat >= self.seats() || !self.people[seat] || !metres.is_finite() || grants > self.grants {
            return;
        }
        let most = START_METRES + self.granted(seat, 0, grants);
        self.said[seat] = (metres.clamp(0.0, most), grants);
    }

    /// What the device of the person in `seat`, come back to the game, is
    /// to have ([`Tell::Metres`]): the metres they had as it last said, with
    /// every grant since, and how many grants that counts.
    pub fn kept(&self, seat: usize) -> (f32, u16) {
        let (metres, counted) = self.said.get(seat).copied().unwrap_or((START_METRES, 0));
        (metres + self.granted(seat, counted, self.grants), self.grants)
    }

    /// How many metres the grants after the `after`th, up to the
    /// `until`th, gave whoever is in `seat`: what each gave their team's
    /// players as it came ([`Game::shares`]).
    fn granted(&self, seat: usize, after: u16, until: u16) -> f32 {
        let side = usize::from(team(seat));
        let (after, until) = (usize::from(after), usize::from(until).min(self.shares.len()));
        self.shares.get(after..until).map_or(0.0, |shares| shares.iter().map(|share| share[side]).sum())
    }

    /// What a grant now gives each of `side`'s players still in the game:
    /// the team's grant, shared between them ([`team_share`]).
    fn share(&self, side: usize) -> f32 {
        let players = (0..self.seats()).filter(|&seat| usize::from(team(seat)) == side && !self.gone[seat]).count();
        team_share(players)
    }

    /// Whoever is in `seat` has left for good: they no longer count for
    /// their team on the golden island, and the grants from now on are
    /// shared between whoever of their team is left. A team with nobody
    /// left loses.
    pub fn leave(&mut self, seat: usize) {
        if seat < self.seats() {
            self.gone[seat] = true;
            self.down[seat] = None;
        }
    }

    /// A step of play, `dt` seconds of it: the clock and its grants, the
    /// fallen standing again, the computer's players, the sticks, and the
    /// golden island. Everything that happened, for whoever runs the game
    /// to carry out.
    pub fn step(&mut self, dt: f32) -> Vec<Event> {
        if !self.playing() || dt <= 0.0 || !dt.is_finite() {
            return std::mem::take(&mut self.events);
        }
        let dt = dt.min(LONGEST_STEP);
        self.elapsed += dt;
        while self.grants < grants_by(self.elapsed) {
            self.grant();
        }
        for hurt in &mut self.hurt {
            *hurt = (*hurt - dt).max(0.0);
        }
        for seat in 0..self.seats() {
            let Some(left) = self.down[seat] else {
                continue;
            };
            if left - dt > 0.0 {
                self.down[seat] = Some(left - dt);
            } else {
                self.down[seat] = None;
                self.stand(seat);
            }
        }
        self.play_computer(dt);
        self.tick_sticks(dt);
        self.reckon_across();
        self.holds(dt);
        if self.over.is_none() && self.elapsed >= PLAY_SECS {
            self.time_up();
        }
        std::mem::take(&mut self.events)
    }

    /// A grant comes: metres for everyone, each team's shared between
    /// those of it still in the game (the computer's counted here, a
    /// person's by their device), and for each, by a throw of the dice, a
    /// stick of dynamite, if they have room for it.
    fn grant(&mut self) {
        self.grants += 1;
        let share = [self.share(0), self.share(1)];
        self.shares.push(share);
        let mut given = Vec::new();
        for seat in 0..self.seats() {
            if self.gone[seat] {
                continue;
            }
            if !self.people[seat] {
                self.crew.grant(seat, share[usize::from(team(seat))], self.elapsed);
            }
            if self.dice.roll() < DYNAMITE_ODDS {
                let held = self.kits[seat].held(Utility::Dynamite);
                if held < DYNAMITE_MOST {
                    self.kits[seat] = with_sticks(self.kits[seat], held + 1);
                    given.push(seat);
                    self.tally.given += 1;
                }
            }
        }
        self.events.push(Event::Granted {
            grant: self.grants,
            sticks: given,
        });
    }

    /// Whoever is in `seat` stands again on their island's middle.
    fn stand(&mut self, seat: usize) {
        let (at, yaw) = home(seat);
        self.at[seat] = at;
        self.yaw[seat] = yaw;
        self.hurt[seat] = 0.0;
        if !self.people[seat] {
            self.crew.stand(seat, at, yaw, self.elapsed);
        }
        self.events.push(Event::Back { seat, at, yaw });
    }

    /// Everyone as the computer's players see them this frame.
    fn seen(&self) -> Vec<Seen> {
        (0..self.seats())
            .map(|seat| Seen {
                at: self.at[seat],
                alive: self.standing(seat),
                in_play: !self.gone[seat],
                unseen: false,
                person: self.people[seat],
                health: 1.0,
                flat: false,
            })
            .collect()
    }

    /// One frame of each of the computer's players ([`Crew::step`]): it
    /// thinks and moves, and what comes of it is carried out here, as it
    /// would be for a person: a punch at someone within its reach, rungs
    /// laid, a stick thrown. Fallen far enough, it is dead.
    fn play_computer(&mut self, dt: f32) {
        let mut seen = self.seen();
        let sticks: Vec<Stick> = self
            .sticks
            .iter()
            .map(|stick| Stick {
                at: stick.lob.to.xz(),
                left: (stick.lob.secs() + DYNAMITE_FUSE - stick.lob.age).max(0.0),
                team: team(stick.by),
            })
            .collect();
        for seat in 0..self.seats() {
            if self.people[seat] || !self.standing(seat) {
                continue;
            }
            let view = View {
                elapsed: self.elapsed,
                nav: &self.nav,
                seats: &seen,
                kits: &self.kits,
                sticks: &sticks,
                bridges: &self.chains,
                across: self.across.map(|across| f32::from(across) / 100.0),
                holding: self.hold.map(|held| (held / HOLD).clamp(0.0, 1.0)),
                on_gold: self.on_gold,
            };
            let order = self.crew.step(seat, &view, dt);
            if let Some(body) = self.crew.body(seat) {
                self.at[seat] = body.at;
                self.yaw[seat] = body.yaw;
                seen[seat].at = body.at;
            }
            self.carrying[seat] = order.carrying;
            if let Some(victim) = order.punch
                && victim < self.seats()
                && self.standing(victim)
                && team(victim) != team(seat)
                && self.at[victim].xz().distance(self.at[seat].xz()) <= REACH + BODY + 0.5
            {
                self.knock(victim, seat, self.at[seat], order.running, false);
            }
            for (chain, index, at, heading) in order.rungs {
                if self.rungs[seat] < MOST_RUNGS {
                    self.take_rung(seat, chain, index, at, heading);
                }
            }
            if let Some((heading, far)) = order.blast {
                self.lob(seat, heading, far);
            }
            if self.at[seat].y < LAND_TOP - FALL_TO_DIE {
                self.fall(seat);
                seen[seat].alive = false;
            }
        }
    }

    /// The sticks of dynamite, for one frame: each comes down, onto ground
    /// or past it into the void; and each lying one whose fuse has burnt
    /// down blows, taking the planks its blast reaches and no more, and
    /// knocking whoever of the other team it reaches away from it as a
    /// punch on the run knocks.
    fn tick_sticks(&mut self, dt: f32) {
        let mut lying = std::mem::take(&mut self.sticks);
        lying.retain_mut(|stick| {
            stick.lob.tick(dt);
            if !stick.lob.down() {
                return true;
            }
            let at = stick.lob.to;
            if !stick.lies {
                if !caught(at.xz(), |spot| self.nav.ground(spot).is_some()) {
                    self.events.push(Event::Lost { stick: stick.id });
                    self.tally.lost += 1;
                    return false;
                }
                stick.lies = true;
            }
            if !stick.lob.blown() {
                return true;
            }
            let side = team(stick.by);
            // Whoever of the other team stands on a plank it takes, out over
            // the void, falls by the thrower's doing, knocked by it or not.
            let taken = self.nav.would_blast(at.xz(), DYNAMITE_RADIUS);
            let stranded: Vec<usize> = (0..self.seats())
                .filter(|&seat| team(seat) != side && self.standing(seat))
                .filter(|&seat| {
                    let feet = self.at[seat].xz();
                    self.nav.grid().height(feet).is_none()
                        && taken.iter().any(|&(of, chain, plank)| {
                            let rungs = self.nav.rungs(of, chain);
                            let plank = usize::from(plank);
                            plank + 1 < rungs.len() && over_plank(feet, rungs[plank], rungs[plank + 1])
                        })
                })
                .collect();
            for seat in stranded {
                self.last_hit[seat] = Some((stick.by, self.elapsed));
            }
            let planks = self.nav.blast(at.xz(), DYNAMITE_RADIUS);
            let reached: Vec<usize> = (0..self.seats())
                .filter(|&seat| team(seat) != side && self.standing(seat) && in_dynamite(at, self.at[seat]))
                .collect();
            for seat in reached {
                self.knock(seat, stick.by, at, true, true);
            }
            self.tally.blown += 1;
            self.tally.planks_blown += planks.len() as u32;
            self.events.push(Event::Blew {
                stick: stick.id,
                by: stick.by,
                at,
                planks,
            });
            false
        });
        lying.append(&mut self.sticks);
        self.sticks = lying;
    }

    /// Works out again how far each team's bridge has got
    /// ([`RaceRound::across`]), when a plank has been laid or taken since.
    fn reckon_across(&mut self) {
        if self.across_of == Some(self.nav.version) {
            return;
        }
        self.across_of = Some(self.nav.version);
        for side in 0..TEAMS as u8 {
            let share = if self.nav.joined(Region::Team(side), Region::Middle) {
                1.0
            } else {
                let way = the_way_across(side);
                let mut best: f32 = 0.0;
                for &(seat, chain) in self.chains.iter().filter(|(seat, _)| team(usize::from(*seat)) == side) {
                    let rungs = self.nav.rungs(seat, chain);
                    for (plank, pair) in rungs.windows(2).enumerate() {
                        if self.nav.broken(seat, chain, plank as u16) || pair[0].distance(pair[1]) > LONGEST_PLANK {
                            continue;
                        }
                        for end in pair {
                            let short = (end.distance(GOLDEN) - ISLAND_RADIUS).max(0.0);
                            best = best.max(1.0 - short / way);
                        }
                    }
                }
                best.clamp(0.0, 1.0)
            };
            let before = self.across[usize::from(side)];
            self.across[usize::from(side)] = (share * 100.0).floor() as u8;
            if share >= 1.0 && before < 100 {
                self.tally.joined[usize::from(side)].get_or_insert(self.elapsed);
            }
        }
    }

    /// Whether whoever is in `seat` stands on the golden island: on their
    /// feet, over its land, their feet within [`ON_GOLD`] of its top; and,
    /// the computer's, not in the air, as the game knows.
    pub fn on_gold(&self, seat: usize) -> bool {
        if !self.standing(seat) {
            return false;
        }
        let at = self.at[seat];
        let grid = self.nav.grid();
        let over = grid.region(at.xz()) == Some(Region::Middle)
            && grid.height(at.xz()).is_some_and(|top| at.y >= top - 0.3 && at.y <= top + ON_GOLD);
        over && (self.people[seat] || self.crew.body(seat).is_some_and(|body| body.lift == 0.0))
    }

    /// The golden island, for one frame: how many of each team stand on it;
    /// each team with every one of its players on it holds it, its ring
    /// filling, and empties the moment one of them is off it; and a team
    /// whose ring is full has won.
    fn holds(&mut self, dt: f32) {
        let mut on = [0u8; TEAMS];
        let mut all = [true; TEAMS];
        let mut anyone = [false; TEAMS];
        for seat in 0..self.seats() {
            if self.gone[seat] {
                continue;
            }
            let side = usize::from(team(seat));
            anyone[side] = true;
            if self.on_gold(seat) {
                on[side] += 1;
            } else {
                all[side] = false;
            }
        }
        self.on_gold = on;
        if self.over.is_some() {
            return;
        }
        for side in 0..TEAMS {
            let now = anyone[side] && all[side];
            if now != self.holding[side] {
                self.holding[side] = now;
                self.events.push(Event::Holding {
                    team: side as u8,
                    holding: now,
                });
                if now {
                    self.tally.holds[side] += 1;
                    self.tally.first_hold[side].get_or_insert(self.elapsed);
                }
            }
            self.hold[side] = if now { self.hold[side] + dt } else { 0.0 };
        }
        // A team with nobody left in the game has lost it.
        match anyone {
            [false, false] => return self.finish(None, Finish::Left),
            [true, false] => return self.finish(Some(0), Finish::Left),
            [false, true] => return self.finish(Some(1), Finish::Left),
            [true, true] => {}
        }
        match self.hold.map(|held| held >= HOLD) {
            [true, true] => self.finish(None, Finish::Held),
            [true, false] => self.finish(Some(0), Finish::Held),
            [false, true] => self.finish(Some(1), Finish::Held),
            [false, false] => {}
        }
    }

    /// The time is up now, whatever the clock says, if the game is being
    /// played: it is over by the same rule as at [`PLAY_SECS`]. For trying
    /// the game out, when someone skips the rest of it (`Ask::Skip`, which
    /// only a server that allows it hears; mine).
    pub fn call_time(&mut self) {
        if self.playing() {
            self.time_up();
        }
    }

    /// The time is up: the team with more of its players on the golden
    /// island wins; with as many, the one whose bridge has got further
    /// across, as everyone's screen shows it ([`RaceRound::across`]); and
    /// with that the same too, it is a draw.
    fn time_up(&mut self) {
        let [red, blue] = self.on_gold;
        let winner = if red != blue {
            Some(if red > blue { 0 } else { 1 })
        } else if self.across[0] != self.across[1] {
            Some(if self.across[0] > self.across[1] { 0 } else { 1 })
        } else {
            None
        };
        self.finish(winner, Finish::Time);
    }

    fn finish(&mut self, winner: Option<u8>, finish: Finish) {
        if self.over.is_none() {
            self.over = Some((winner, finish, self.elapsed));
            self.events.push(Event::Over { winner, finish });
        }
    }

    /// Where the body in `seat` is, feet, and which way it faces: the
    /// computer's as the game moves them, a person's as their device last
    /// said.
    pub fn at(&self, seat: usize) -> (Vec3, f32) {
        (self.at.get(seat).copied().unwrap_or_default(), self.yaw.get(seat).copied().unwrap_or_default())
    }

    /// Whether the computer's player in `seat` carries a plank, laying its
    /// bridge: for drawing it so.
    pub fn carrying(&self, seat: usize) -> bool {
        self.carrying.get(seat).copied().unwrap_or(false)
    }

    /// What everyone has on their buttons, by seat: their sticks of
    /// dynamite. What the room is sent as its [`crate::Kits`].
    pub fn kits(&self) -> &[Kit] {
        &self.kits
    }

    /// How many metres of bridge whoever is in `seat` has left to lay: the
    /// computer's as it counts them, a person's as their device last said.
    pub fn metres(&self, seat: usize) -> f32 {
        if self.person(seat) { self.said.get(seat).map_or(0.0, |said| said.0) } else { self.crew.metres(seat) }
    }

    /// How many metres of bridge each has laid, all told, by seat.
    pub fn laid(&self) -> &[f32] {
        &self.laid
    }

    /// The map with every bridge laid on it so far, and the holes blown in
    /// them: what the computer's players walk by, and what whoever runs the
    /// game asks of where there is ground (one whose device has gone stands
    /// where they stood, and falls if what was under them is gone).
    pub fn nav(&self) -> &Nav {
        &self.nav
    }

    /// The computer's players.
    pub fn crew(&self) -> &Crew {
        &self.crew
    }

    /// How it has gone so far.
    pub fn tally(&self) -> Tally {
        self.tally
    }

    /// What the room is sent of it ([`RaceRound`]), cheap to compare.
    pub fn round(&self) -> RaceRound {
        let share = self.shares.last().copied().unwrap_or_else(|| [self.share(0), self.share(1)]);
        RaceRound {
            left: (PLAY_SECS - self.elapsed).max(0.0).ceil() as u16,
            grants: self.grants,
            share: share.map(|metres| (metres * 10.0).round().clamp(0.0, 255.0) as u8),
            hold: self.hold.map(|held| ((held / HOLD) * 10.0).floor().clamp(0.0, 10.0) as u8),
            on_gold: self.on_gold,
            winner: self.over.and_then(|(winner, ..)| winner),
            over: self.over.is_some(),
            across: self.across,
        }
    }

    /// Once it is over: who won, how, and when, in seconds of play.
    pub fn result(&self) -> Option<(Option<u8>, Finish, f32)> {
        self.over
    }

    /// Once it is over, how it came out, by seat (`table::Results` without
    /// its prizes): the winning team first, 0, and the other 1 (a team
    /// comes in a place together; in a draw everyone is first); each one's
    /// score, the whole metres of bridge they laid; and nothing for the
    /// table as a whole.
    pub fn outcome(&self) -> Option<(Vec<u8>, Vec<u32>, Option<u32>)> {
        let (winner, ..) = self.over?;
        let places = (0..self.seats())
            .map(|seat| match winner {
                Some(won) if team(seat) != won => 1,
                _ => 0,
            })
            .collect();
        let scores = self.laid.iter().map(|laid| laid.round() as u32).collect();
        Some((places, scores, None))
    }
}

/// Whether `at` is over the plank between the rungs `a` and `b`: within its
/// width of its middle line, between the two or a step past either.
fn over_plank(at: Vec2, a: Vec2, b: Vec2) -> bool {
    let along = b - a;
    let length = along.length();
    if length < 1e-4 {
        return false;
    }
    let way = along / length;
    let off = at - a;
    (-0.3..=length + 0.3).contains(&off.dot(way)) && off.perp_dot(way).abs() <= PLANK_HALF_WIDTH
}

/// `kit` with `count` sticks of dynamite on their button, and the button
/// empty with none, so that it is not left up, faint.
fn with_sticks(kit: Kit, count: u8) -> Kit {
    if count > 0 {
        return kit.holding(Utility::Dynamite, count);
    }
    let mut kit = kit;
    if let Some(slot) = kit.slot_of(Utility::Dynamite) {
        kit.slots[slot] = None;
    }
    kit
}

// ------------------------------------------------------ arrival and prizes

/// Where the player in `seat` stands as they arrive on the game's venue, and which way they
/// face: the first of each team on the middle of its island, the second 3 m on toward the
/// golden island; both facing the golden island, on the island's top as lifted ([`LIFT`]).
pub fn arrival(seat: usize) -> (Vec3, Quat) {
    let island = ISLANDS[usize::from(team(seat))];
    let at = if seat < 2 { island } else { island + (GOLDEN - island).normalize() * 3.0 };
    let toward = (GOLDEN - at).normalize();
    // A turn of `yaw` about the upright points -Z at (-sin yaw, -cos yaw).
    let yaw = (-toward.x).atan2(-toward.y);
    (Vec3::new(at.x, LIFT, at.y), Quat::from_rotation_y(yaw))
}

/// What each seat is paid, from how the game came out (`results.places`, `scores`, `total`);
/// `people[seat]` is false for the computer's seats, which are paid nothing. The winning
/// team's people, those who came first, 110 each, and the rest nothing; a draw, everyone
/// first, the fee back to each.
pub fn prizes(results: &table::Results, people: &[bool]) -> Vec<u32> {
    let draw = results.places.iter().all(|&place| place == 0);
    results
        .places
        .iter()
        .enumerate()
        .map(|(seat, &place)| match people.get(seat).copied().unwrap_or(false) {
            false => 0,
            true if draw => SHAPE.fee,
            true if place == 0 => 110,
            true => 0,
        })
        .collect()
}
