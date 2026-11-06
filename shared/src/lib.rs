//! What 동그라미타운 and its server say to each other.
//!
//! The game (the root crate) and the server (`server/`) both build on this
//! crate, and both register it through [`protocol`], so they register the
//! same things in the same order. replicon hashes those registrations, and a
//! client whose hash differs from the server's is told it is out of date
//! instead of being let in.
//!
//! Three rules keep it cheap (`MULTIPLAYER.md`, "The one rule"):
//!
//! * A position is whole centimetres and a 16-bit turn ([`Spot`]): about ten
//!   bytes. Riding one of the town's cars, it is a place on the car, which
//!   moves with it ([`Spot::aboard`]), so that a rider is never seen beside
//!   it.
//! * Only what has changed is sent. replicon works off Bevy's change
//!   detection, the same idea as `set_if_neq` everywhere in the game, and a
//!   device sends its own position only while it changes.
//! * Who someone is, their name and their look ([`Member`]), goes once, when
//!   they come into your room, not with every move.
//!
//! What goes which way:
//!
//! * **Server to everyone in a room**, replicated: each player's [`Member`],
//!   [`Spot`] and [`Venue`]; and in a game of House Builder its [`Round`], its
//!   [`Seats`] and every piece put down, [`Built`].
//! * **Device to server**: where you are, [`Moved`], unreliable, since a lost
//!   one is replaced by the next; and everything you ask for, [`Ask`], in
//!   order.
//! * **Server to one device**: what concerns you alone, [`Tell`], in order.
//! * **Both ways, a few times as a device comes online and once a minute
//!   after**: what time it is, [`WhatTime`] and its answer [`TheTime`], for the
//!   town's clock, which the cars round the town, the fountain's jet and the
//!   day go by on every device. Nothing else about any of them is sent.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::replication::registry::ctx::WriteCtx;
use bevy_replicon::shared::replication::registry::rule_fns::{
    RuleFns, default_deserialize, default_serialize,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// netcode's protocol id: which game a packet is for. A packet with another
/// one is dropped without a word, so this names the game and never changes;
/// versions are told apart by replicon's protocol hash instead, which a
/// client is told about.
pub const PROTOCOL_ID: u64 = u64::from_be_bytes(*b"DGRMTOWN");

/// Bumped whenever what is sent changes: a field added, taken out or turned
/// to mean something else. replicon's protocol hash is made of the names of
/// the types [`protocol`] registers and their order, and nothing of what is
/// in them (`ProtocolHasher` in bevy_replicon 0.44), so it notices a type
/// added or renamed, and nothing else.
///
/// 2 since 2026-10-23: [`Spot::aboard`], for riding the town's car.
///
/// 3 since 2026-10-24: [`WhatTime`] and [`TheTime`], for the town's clock;
/// and the town's cars are two buses, so a place on car 0 is a place on a
/// bus, which an app with the cars put somewhere else entirely.
///
/// 4 since 2026-10-30: the town is Hajun's new map, `roundtownmap.glb`, 420 m
/// across where the old one was 140, so a place in the town is somewhere
/// else entirely in an app with the old one; and everyone arrives in it on
/// the path round its pond ([`TOWN_ARRIVAL_RADIUS`]).
///
/// Not bumped for [`Ask::Profile`] (2026-10-31), added last: an app from
/// before never sends it, and draws a look it has not heard of as its first.
///
/// 5 since 2026-11-07: the town is Hajun's `pvptownmap.glb`, four islands
/// at the ends of bridges round a town with no sea round it, and everyone
/// arrives on one of the islands ([`TOWN_ISLANDS`]); [`Ask::Respawn`], for
/// whoever falls off into the void; and [`Ask::Punch`] and [`Tell::Knocked`],
/// a punch landing on someone online.
pub const PROTOCOL_VERSION: u32 = 5;

/// The UDP port the game server listens on. 443, where QUIC goes, because a
/// network that lets any UDP through lets that through: school and office
/// networks often stop the rest.
pub const GAME_PORT: u16 = 443;

/// How many times a second the server sends what has changed, and how many
/// times a second a device sends its own position while it moves.
pub const TICK_HZ: f64 = 15.0;

/// How many players a town channel holds (`MULTIPLAYER.md`, "How many to a
/// room": 16 to start).
pub const TOWN_SEATS: usize = 16;

/// How many players a game of House Builder holds, bots included.
pub const GAME_SEATS: usize = 8;

/// The fewest bodies the town shows: below this, each device fills it out
/// with AIs of its own (Hajun's choice, 2026-10-08), one leaving for each
/// real player who arrives. 8 until 2026-11-01, when Hajun asked for "a
/// total of 32 ppl in the map including ai, so each new player join
/// replaces a ai". Not part of the protocol: the server never counts them.
pub const TOWN_BODIES: usize = 32;

/// The look a new account wears until its owner has chosen one of
/// [`PEOPLE_LOOKS`] (the game's `signup`, [`Ask::Profile`]), which the game
/// draws as the first of them. Every account from before there was a choice
/// (2026-10-31) wears it. Bots wear others.
pub const PLAYER_LOOK: &str = "blue";

/// The two sides a person chooses between on signing up, a boy and a girl:
/// the first look of [`BOY_LOOKS`] and of [`GIRL_LOOKS`], which were the
/// only two until 2026-11-02 (the game's `limbperson.glb` and
/// `limbpersongirl.glb` until 2026-11-01, then its `lobby::MODEL` and
/// `GIRL_MODEL`).
pub const PEOPLE_LOOKS: [&str; 2] = ["boy", "girl"];

/// Every look there is, by side: the boy and the girl, and Hajun's
/// recolourings of each (2026-11-02: "use variations of the colors. i made
/// various colors for both guy and girl for the ais"; and the same night,
/// for people too: "put some arrow thingy next to each guy and girl
/// character selection in the start so u can choose which color guy or
/// which color girl"). A person picks any of them on signing up
/// ([`profile::look`]); a bot is a boy or a girl by the toss of a coin and
/// then one of that side at random ([`ai_look`]), on the device for its own
/// AIs and on the server for House Builder's. The game maps each to its
/// model (`lobby::LOOKS`); an app that has not heard of one draws the boy.
pub const BOY_LOOKS: [&str; 3] = ["boy", "boy_blackbrown", "boy_whitejeans"];
pub const GIRL_LOOKS: [&str; 3] = ["girl", "girl_whiteblue", "girl_yellowpink"];

/// What the computer's players are called, the boys and the girls (Hajun,
/// 2026-11-02: "use the more manly ai names for the boy ais and girly cute
/// ai names for the girl ais from the names you already made"): the 48
/// names the game's AIs went by, split two ways. Players online go by the
/// same names with a number after (the server's).
pub const BOY_NAMES: [&str; 24] = [
    "Hodu", "Gamja", "Mandu", "Kimbap", "Maru", "Kong", "Hobak", "Bam", "Dori", "Bada", "Gureum",
    "Dongi", "Tori", "Jjang", "Pado", "Choco", "Kkami", "Ddol", "Ssal", "Bokki", "Tangi", "Sol",
    "Nuri", "Saem",
];
pub const GIRL_NAMES: [&str; 24] = [
    "Mochi", "Dubu", "Bori", "Yuja", "Nabi", "Podo", "Sagwa", "Gyul", "Dalgi", "Haru", "Byeol",
    "Bomi", "Nana", "Mint", "Coco", "Latte", "Bbori", "Yeppi", "Ggul", "Sseul", "Jelly", "Pudding",
    "Cookie", "Waffle",
];

/// One of the computer's looks: a boy if `boy`, else a girl, and of that
/// side the `pick`th, round and round. Both are the caller's to draw at
/// random, so that this crate needs no dice of its own.
pub fn ai_look(boy: bool, pick: usize) -> &'static str {
    let side: &[&str] = if boy { &BOY_LOOKS } else { &GIRL_LOOKS };
    side[pick % side.len()]
}

/// The names the computer's players of one side go by.
pub fn ai_names(boy: bool) -> &'static [&'static str] {
    if boy { &BOY_NAMES } else { &GIRL_NAMES }
}

/// What a person says about themselves on signing up, as both sides check
/// it: a name to go by, and which of [`PEOPLE_LOOKS`] they wear.
pub mod profile {
    /// How long a name may be, in characters.
    pub const SHORTEST: usize = 2;
    pub const LONGEST: usize = 16;

    /// `text` as a name, if it is one: [`SHORTEST`] to [`LONGEST`]
    /// characters once the spaces round it are trimmed, all of them
    /// [`allowed`].
    pub fn name(text: &str) -> Option<String> {
        let name = text.trim();
        let fits = (SHORTEST..=LONGEST).contains(&name.len());
        (fits && name.chars().all(allowed)).then(|| name.to_string())
    }

    /// Whether `c` may be in a name: plain ASCII that the game's font can
    /// draw, letters, digits and what is on a keyboard, and a space.
    pub fn allowed(c: char) -> bool {
        c.is_ascii_graphic() || c == ' '
    }

    /// Whether `look` is one a person may choose: any of either side's
    /// (since 2026-11-02; the first of each until then).
    pub fn look(look: &str) -> bool {
        super::BOY_LOOKS.contains(&look) || super::GIRL_LOOKS.contains(&look)
    }
}

/// Registers everything both sides send, in one order for both. Call it after
/// `RepliconPlugins` and before the renet client or server is made: its
/// channels are counted from what is registered here.
pub fn protocol(app: &mut App) {
    app.replicate::<Member>()
        .replicate::<Venue>()
        .replicate_with(RuleFns::new(default_serialize::<Spot>, deserialize_spot))
        .replicate::<Round>()
        .replicate::<Seats>()
        .replicate::<Built>()
        .add_client_message::<Moved>(Channel::Unreliable)
        .add_client_message::<Ask>(Channel::Ordered)
        .add_server_message::<Tell>(Channel::Ordered)
        .add_client_message::<WhatTime>(Channel::Unreliable)
        .add_server_message::<TheTime>(Channel::Unreliable)
        .make_message_independent::<TheTime>();
    app.world_mut()
        .resource_mut::<ProtocolHasher>()
        .add_custom(PROTOCOL_VERSION);
}

// ---------------------------------------------------------------- islands

/// Which island something is on. The game's `island::Venue` is this.
///
/// Every island stands at the origin, in the same place: what keeps them
/// apart is this, carried by everything that moves or collides.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Venue {
    /// `roundtownmap.glb`: Round Town, round a pond with a fountain in it,
    /// where everyone is.
    Town,
    /// Your home, where nobody else goes for now.
    Home,
    /// `lobbymap.glb`: where the players of a game of House Builder wait for
    /// it to fill.
    Lobby,
    /// One builder's own copy of `homemap.glb` in a game of House Builder, by
    /// their seat in the game.
    Plot(u8),
}

/// How far from the middle everyone stands on arriving on an island, in
/// metres: well inside the grass of the home and the lobby, which reaches 20 m
/// out. The town has its own ([`TOWN_ARRIVAL_RADIUS`]).
pub const ARRIVAL_RADIUS: f32 = 6.0;

/// How far from the middle everyone stood on arriving in the town, in
/// metres, from 2026-10-30 to 2026-11-07: on the middle of the path, 30 to
/// 33 m out, round the pond that fills the middle of `roundtownmap.glb`.
/// Since then everyone arrives on one of the four islands round the town
/// ([`TOWN_ISLANDS`]), and this is where the town's own name shows and the
/// House Builder stood.
pub const TOWN_ARRIVAL_RADIUS: f32 = 31.5;

/// The middles of the four islands at the ends of the town's bridges, in
/// metres from the middle of the town, where everyone arrives in Hajun's
/// `pvptownmap.glb` (2026-11-07: "make the player start in the center of
/// one of the 4 bridge connected island"): 200 m out along each axis, each
/// 32 m across, with a bridge 14 m wide from it to the town, 75 m out. In
/// the game's x and z: north (Blender's +Y) first, then clockwise seen from
/// above.
pub const TOWN_ISLANDS: [Vec2; 4] = [
    Vec2::new(0.0, -200.0),
    Vec2::new(200.0, 0.0),
    Vec2::new(0.0, 200.0),
    Vec2::new(-200.0, 0.0),
];

/// How far from its island's middle everyone on it but the first stands on
/// arriving, in metres: round the one in the middle, a few metres apart.
pub const ISLAND_RING: f32 = 4.0;

/// Which of the [`TOWN_ISLANDS`] the player in `seat` arrives on, and is
/// put back on when they fall off (the game's `lobby`): seat by seat round
/// the four, so that the first four seats of a town channel, four people,
/// are on four different islands (Hajun: "when there is 2 players+ you
/// will not spawn in the same bridge connected island").
pub fn town_island(seat: usize) -> usize {
    seat % TOWN_ISLANDS.len()
}

/// Where the player in `seat` stands on arriving on `venue`, and which way
/// they face. In the town, on their island ([`town_island`]): the first on
/// each island in its middle, and the rest round that [`ISLAND_RING`] out,
/// every one facing the town, down the bridge. Anywhere else, on a ring
/// round the middle, facing in: the first eight seats each an eighth of the
/// turn on from the one before, the next eight between them, and past
/// sixteen a ring further out. The server works out arrivals with this too,
/// so that everyone sees you arrive where you do.
pub fn arrival(venue: Venue, seat: usize) -> (Vec3, Quat) {
    if venue == Venue::Town {
        let island = TOWN_ISLANDS[town_island(seat)];
        let round = seat / TOWN_ISLANDS.len();
        // Seven round the one in the middle, and past them round again, a
        // little further out and turned on by half a place.
        let places = 7;
        let out = (round + places - 1) / places;
        let at = if round == 0 {
            island
        } else {
            let place = (round - 1) % places;
            let angle = (place as f32 + 0.5 * (out - 1) as f32) * TAU / places as f32;
            island + Vec2::new(angle.cos(), angle.sin()) * (ISLAND_RING + 2.0 * (out - 1) as f32)
        };
        let toward = -island.normalize();
        // A turn of `yaw` about the upright points -Z at (-sin yaw, -cos yaw).
        let yaw = (-toward.x).atan2(-toward.y);
        return (Vec3::new(at.x, 0.0, at.y), Quat::from_rotation_y(yaw));
    }
    let eighth = TAU / 8.0;
    let ring = seat / 16;
    let within = seat % 16;
    let angle = (within % 8) as f32 * eighth
        + if within >= 8 { eighth * 0.5 } else { 0.0 }
        + ring as f32 * eighth * 0.25;
    let radius = ARRIVAL_RADIUS + ring as f32 * 2.0;
    let turn = Quat::from_rotation_y(angle);
    // Facing in: a turn of `angle` about the upright points -Z at the middle
    // from where it puts +Z.
    (turn * Vec3::new(0.0, 0.0, radius), turn)
}

// ---------------------------------------------------------------- players

/// One player in a room, as the server describes them: sent once, when they
/// come into your room. Flat and primitive-only: a look is a name, never an
/// asset path.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[component(immutable)]
pub struct Member {
    /// Unique and stable for as long as they are connected, so that whatever
    /// is said about them can say who it is about.
    pub id: u64,
    /// What everyone else sees them called. Plain ASCII: the game's font has
    /// no more.
    pub name: String,
    /// Which of the game's looks they wear.
    pub look: String,
    /// The computer is playing this seat, not a person.
    pub ai: bool,
}

/// Where a body stands and which way it faces: whole centimetres, and a turn
/// in 65,536ths of a circle. About twelve bytes once postcard has packed it.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub struct Spot {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: u16,
    /// The car the body is riding, by its number, if it is: on it, or in the
    /// air off it and carried along with it until it comes down on anything
    /// else. The place and the turn are then in the car's own space, which
    /// moves with it, and not the island's: someone watching puts the body
    /// on the car as they see it, wherever that is (the game's `car`), so
    /// that a rider is never seen beside it, nor behind it by the time the
    /// update takes to come.
    pub aboard: Option<u8>,
    /// The body has come to a stop here. Someone watching draws it standing
    /// here from then on, rather than carrying it on the way it was going
    /// while they wait for the next update, which is not coming.
    pub still: bool,
    /// On a device, the server tick of the update that brought it, so that
    /// updates can be laid out in the time they were sent rather than the
    /// time they happened to arrive. Never sent.
    #[serde(skip)]
    pub tick: u32,
}

impl Spot {
    /// Where a body at `translation`, turned by `rotation`, stands.
    pub fn of(translation: Vec3, rotation: Quat) -> Self {
        let forward = rotation * Vec3::NEG_Z;
        let yaw = (-forward.x).atan2(-forward.z);
        Self {
            x: centimetres(translation.x),
            y: centimetres(translation.y),
            z: centimetres(translation.z),
            yaw: ((yaw / TAU).rem_euclid(1.0) * 65_536.0).round() as u32 as u16,
            aboard: None,
            still: false,
            tick: 0,
        }
    }

    /// The same, as a place on the car numbered `car`, in its own space, or,
    /// with none, on the island.
    pub fn aboard(self, car: Option<u8>) -> Self {
        Self { aboard: car, ..self }
    }

    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32) / 100.0
    }

    /// The turn about the upright, in radians.
    pub fn yaw(&self) -> f32 {
        f32::from(self.yaw) / 65_536.0 * TAU
    }

    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw())
    }

    /// The same place, the same way and as still, on the same car or none,
    /// whenever it was sent.
    pub fn same(&self, other: &Self) -> bool {
        (self.x, self.y, self.z, self.yaw, self.aboard, self.still)
            == (other.x, other.y, other.z, other.yaw, other.aboard, other.still)
    }
}

fn centimetres(metres: f32) -> i32 {
    (metres * 100.0).round() as i32
}

/// The cars that drive round the town, two buses since 2026-10-24 (the
/// game's `car`), as far as the server checks a rider's moves against them:
/// it cannot tell where they are, having no islands, but it knows how fast a
/// car can take anyone.
pub mod cars {
    /// The fastest a car goes, in metres a second, along the straight and
    /// gentle stretches of the town's roads: as fast as a body runs, so that
    /// someone running beside it keeps up and no more, and can get on.
    pub const TOP_SPEED: f32 = 7.0;
    /// How far from a car's own middle a body riding it can be, in metres:
    /// on it, which is 12.5 m long, its corners 6.5 m out, or in the air off
    /// it, carried along with it until it comes down, which a run takes
    /// another 5 m or so.
    pub const REACH: f32 = 15.0;
    /// The fastest the car anyone can drive themselves goes, in metres a
    /// second (the game's `drive`, since 2026-11-05): boosted, or reversing.
    /// Nothing says over the network that a body is in a car, so the server
    /// lets every body go this fast across the ground
    /// (`server/src/players.rs`). The car's own speeds are in the game's
    /// `assets/ui.ron`, and a test there keeps them no faster than this:
    /// 40 since 2026-11-06 (Hajun: "change the max speed to 40m/s"), 20
    /// until then.
    pub const DRIVEN_TOP_SPEED: f32 = 40.0;
}

/// A [`Spot`] as it arrives, stamped with the tick of the update it came in.
fn deserialize_spot(ctx: &mut WriteCtx, message: &mut Bytes) -> Result<Spot> {
    let mut spot: Spot = default_deserialize(ctx, message)?;
    spot.tick = ctx.message_tick.get();
    Ok(spot)
}

// ------------------------------------------------------------ House Builder

/// House Builder's rules, which the server plays by online and the game by
/// itself offline: in one place, so that the two never differ.
pub mod rules {
    /// What every player starts with, in coins.
    pub const STARTING_BALANCE: u32 = 1000;
    /// What a game costs to join, in coins.
    pub const ENTRY: u32 = 100;
    /// How long the lobby waits for people before the computer starts taking
    /// the empty seats, in seconds: online only (Hajun's choice, 2026-10-08).
    /// Offline there is nobody to wait for.
    pub const PEOPLE_WAIT: f32 = 10.0;
    /// How long the computer takes to fill the seats left, in seconds: one at
    /// a time, evenly over it, the last as it runs out.
    pub const QUEUE_FILL: f32 = 5.0;
    /// How long a full room shows 8/8 before everyone leaves for their plot.
    pub const FULL_HOLD: f32 = 2.0;
    /// How long everyone has to build, in seconds.
    pub const BUILD_SECS: f32 = 300.0;
    /// How long everyone spends at each house rating it, in seconds.
    pub const VISIT_SECS: f32 = 15.0;
    /// The most stars a house can be given.
    pub const MOST_STARS: u8 = 5;
    /// What each place pays, in coins, first to last (Hajun's, 2026-10-08).
    pub const PRIZES: [u32; super::GAME_SEATS] = [500, 400, 300, 200, 100, 50, 25, 0];
    /// How long the results are up, in seconds, before everyone who has not
    /// asked to play again is sent back to the town.
    pub const RESULTS_SECS: f32 = 15.0;
}

/// Every plot in the order it came, first to last: by its stars, most first,
/// a tie settled by `lot`. The seats are shuffled by the lot before they are
/// put in order of their stars, and the order keeps those with the same stars
/// as the shuffle left them.
///
/// The server's lot is its game's [`Round::theme`], drawn at random when the
/// game opens and sent to everyone in it, so that every device works out the
/// same places from the stars as the server pays out on, without being told
/// them.
pub fn standings(stars: &[u32; GAME_SEATS], lot: u8) -> [u8; GAME_SEATS] {
    let mut order: [u8; GAME_SEATS] = std::array::from_fn(|seat| seat as u8);
    // Fisher and Yates' shuffle, drawing on xorshift32 seeded from the lot:
    // odd, so never the zero xorshift cannot leave.
    let mut state = u32::from(lot).wrapping_mul(0x9E37_79B9) | 1;
    for last in (1..GAME_SEATS).rev() {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        order.swap(last, (state % (last as u32 + 1)) as usize);
    }
    // A stable sort: those with the same stars stay as the shuffle put them.
    order.sort_by_key(|&plot| std::cmp::Reverse(stars[usize::from(plot)]));
    order
}

/// A game of House Builder as everyone in it sees it: how far it has got, how
/// long is left of that, and the theme.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Round {
    pub stage: Stage,
    /// Whole seconds left of this stage, rounded up: what the board shows.
    /// Once the game is over, how long the results are up for.
    pub left: u16,
    /// Which of the game's themes, by its place in `builder::THEMES`, going
    /// round again past the last. Also the lot its ties are settled by
    /// ([`standings`]).
    pub theme: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Stage {
    /// In the lobby, waiting for the room to fill.
    Queue,
    /// Full, and about to go.
    Full,
    /// Everyone on their own plot, building.
    Build,
    /// Everyone at the house on this plot, rating it.
    Visit(u8),
    /// Done: the house on `winner` came first, and these are every house's
    /// stars, by plot, which the rest of the places are worked out from
    /// ([`standings`]).
    Over { winner: u8, stars: [u32; GAME_SEATS] },
}

/// Everyone in a game of House Builder, in seat order: a builder's plot is
/// their seat.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Seats(pub Vec<Member>);

/// A piece put down on a plot in a game of House Builder: a house, a door, a
/// window, a bed. Sent once it is down, so that everyone visiting sees it;
/// never while it is being dragged.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[component(immutable)]
pub struct Built {
    /// Whose plot it is on.
    pub plot: u8,
    /// What it is: `build::PIECES`' name for it, never an asset path.
    pub kind: String,
    pub at: At,
}

/// Where a piece is on its plot.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum At {
    /// Standing on the plot: where its origin is, and its turn about the
    /// upright, in radians.
    Ground { x: f32, y: f32, z: f32, yaw: f32 },
    /// In or on a wall of the plot's house: which of its walls, counted in
    /// order of where they stand in the house, how far along it and up it,
    /// and which face, 1 or -1, for what hangs on one.
    Wall {
        wall: u8,
        along: f32,
        up: f32,
        face: i8,
    },
}

// --------------------------------------------------------------- messages

/// Where you are now: sent while it changes, 15 times a second, and nothing
/// while you stand still. Unreliable: a lost one is replaced by the next.
#[derive(Message, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Moved {
    /// How many times the server has moved you, as of this position. One
    /// sent before the server last moved you is from where you were before,
    /// and is dropped rather than taken for a leap.
    pub arrival: u8,
    pub spot: Spot,
}

/// Something you ask of the server. In order, and never lost.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub enum Ask {
    /// The first thing a device says: where it already is, so that it is let
    /// in there rather than moved.
    Hello { venue: Venue, spot: Spot },
    /// Take me home, or back to the town.
    Travel(Venue),
    /// Pay, and join a game of House Builder: from the House Builder's offer,
    /// or after one game for another.
    Play,
    /// Back to the town after a game.
    Leave,
    /// Give the house on `plot` this many stars.
    Rate { plot: u8, stars: u8 },
    /// End the wait early: for trying the game out, and only allowed by a
    /// server that says so.
    Skip,
    /// This piece is down on my plot, here. `id` is mine to choose, and says
    /// the same piece again if it is picked up and put down again.
    Put { id: u32, kind: String, at: At },
    /// This piece of mine is no longer down: picked back up, or thrown away.
    Take { id: u32 },
    /// Who I am, as chosen on signing up (the game's `signup`): the name to
    /// go by, and which of [`PEOPLE_LOOKS`] I wear. The server keeps both
    /// with the account and tells everyone in the room. Sent on coming online
    /// if the account has them otherwise, and whenever they change. One that
    /// is no name ([`profile::name`]) or no look is ignored. Last, since
    /// 2026-10-31: an app from before never sends it.
    Profile { name: String, look: String },
    /// I have fallen off the town into the void (since 2026-11-07, when the
    /// sea round it went): put me back where my seat arrives, as a fresh
    /// arrival ([`Tell::Arrive`]), so that the leap back is not taken for
    /// a move I could not have made.
    Respawn,
    /// My punch landed on the player whose [`Member::id`] this is: they are
    /// to be knocked back from me ([`Tell::Knocked`]). `running`, I was on
    /// the move as I hit them, which knocks them further, as a sprinting
    /// hit does in Minecraft.
    Punch { who: u64, running: bool },
}

/// What the server tells you alone. In order, and never lost.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub enum Tell {
    /// You are in, and this is who you are to everyone else.
    Welcome { me: Member },
    /// The server has put you on `venue`, where `seat` arrives there, or, with
    /// a `spot`, where you already stood. `arrival` is what to send with your
    /// moves from now on.
    Arrive {
        venue: Venue,
        seat: u8,
        spot: Option<Spot>,
        arrival: u8,
    },
    /// The last move was not one a body could make: back to here.
    PutBack { arrival: u8, spot: Spot },
    /// What you have, now that it has changed.
    Balance(u32),
    /// You are in a game of House Builder, in this seat: your plot.
    Joined { seat: u8 },
    /// The player whose [`Member::id`] is `by` punched you, standing at
    /// `from` (in the island's space, or on a bus, in the bus's): your own
    /// device knocks your body back from there, and sends where it goes as
    /// it does any move. `running`, they hit you on the move.
    Knocked { by: u64, from: Spot, running: bool },
    /// What you asked for cannot be done.
    Refused(Refusal),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// You have not got the coins.
    Coins,
    /// You are in the middle of something else.
    Busy,
}

/// What time it is, asked of the server to set the town's clock by (the
/// game's `clock`): when it was asked, by the device's own steady clock,
/// which comes back with the answer, so that the device can tell how long the
/// answer took. Unreliable: one that is lost is never answered, and the next
/// is asked in its place.
#[derive(Message, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct WhatTime {
    pub asked: f64,
}

/// The server's answer to [`WhatTime`], written the moment it is asked: the
/// time by its clock as it answers, in seconds since 1970, and when it was
/// asked, as the device said. Independent of replication, as nothing in it
/// waits on anything replicated: it goes out at once, rather than with the
/// next tick, which would hold it up to a fifteenth of a second, and put
/// the town's clock out by half that.
#[derive(Message, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TheTime {
    pub asked: f64,
    pub now: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spot_keeps_where_and_which_way() {
        for (at, yaw) in [
            (Vec3::new(5.55, 0.0, -3.21), 0.0),
            (Vec3::new(-580.0, 12.34, 580.0), 1.0),
            (Vec3::new(0.004, -0.7, 0.0), -2.5),
            (Vec3::ZERO, std::f32::consts::PI),
        ] {
            let rotation = Quat::from_rotation_y(yaw);
            let spot = Spot::of(at, rotation);
            assert!(spot.translation().distance(at) < 0.01, "{at} came back {}", spot.translation());
            let back = spot.rotation() * Vec3::NEG_Z;
            let was = rotation * Vec3::NEG_Z;
            assert!(back.distance(was) < 1e-3, "{yaw}: {back} for {was}");
        }
    }

    #[test]
    fn the_first_eight_arrive_where_they_did_offline() {
        let venue = Venue::Lobby;
        for seat in 0..8 {
            let (at, turn) = arrival(venue, seat);
            let was = Quat::from_rotation_y(seat as f32 * TAU / 8.0) * Vec3::new(0.0, 0.0, ARRIVAL_RADIUS);
            assert!(at.distance(was) < 1e-4, "seat {seat}: {at} for {was}");
            let facing = turn * Vec3::NEG_Z;
            assert!(facing.distance(-was.normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
    }

    #[test]
    fn the_first_four_in_the_town_arrive_on_four_islands_facing_it() {
        for seat in 0..4 {
            let (at, turn) = arrival(Venue::Town, seat);
            let island = TOWN_ISLANDS[seat];
            assert!(at.xz().distance(island) < 1e-4, "seat {seat}: {at} for {island}");
            assert_eq!(at.y, 0.0);
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(-island.normalize()) < 1e-4, "seat {seat} faces {facing}");
            assert_eq!(town_island(seat), seat);
        }
        // The rest stand on the islands too, near the middle, and each on
        // the island its seat says.
        for seat in 4..TOWN_SEATS * 2 {
            let (at, _) = arrival(Venue::Town, seat);
            let island = TOWN_ISLANDS[town_island(seat)];
            let out = at.xz().distance(island);
            assert!(out > 3.0 && out < 8.0, "seat {seat}: {out} m from its island's middle");
        }
    }

    #[test]
    fn a_full_town_arrives_apart() {
        for venue in [Venue::Town, Venue::Lobby] {
            let spots: Vec<Vec3> = (0..TOWN_SEATS * 2).map(|seat| arrival(venue, seat).0).collect();
            for (i, a) in spots.iter().enumerate() {
                for b in &spots[i + 1..] {
                    assert!(a.distance(*b) > 2.0, "{venue:?}: {a} and {b}");
                }
            }
        }
    }

    #[test]
    fn the_most_stars_come_first() {
        let stars = [12, 30, 7, 25, 18, 3, 22, 9];
        for lot in 0..=u8::MAX {
            assert_eq!(standings(&stars, lot), [1, 3, 6, 4, 0, 7, 2, 5], "lot {lot}");
        }
    }

    #[test]
    fn a_tie_goes_either_way_by_lot() {
        let mut stars = [5; GAME_SEATS];
        stars[1] = 20;
        stars[6] = 20;
        let firsts: Vec<u8> = (0..=u8::MAX).map(|lot| standings(&stars, lot)[0]).collect();
        assert!(firsts.iter().all(|&first| first == 1 || first == 6));
        assert!(firsts.contains(&1) && firsts.contains(&6));
        // All eight tied: every one of them comes first by some lot, and last
        // by another.
        let level = [10; GAME_SEATS];
        for plot in 0..GAME_SEATS as u8 {
            assert!((0..=u8::MAX).any(|lot| standings(&level, lot)[0] == plot), "{plot} first");
            assert!((0..=u8::MAX).any(|lot| standings(&level, lot)[7] == plot), "{plot} last");
        }
    }

    #[test]
    fn every_plot_has_a_place() {
        for lot in 0..=u8::MAX {
            let mut order = standings(&[4, 4, 9, 0, 4, 9, 1, 4], lot);
            order.sort_unstable();
            assert_eq!(order, [0, 1, 2, 3, 4, 5, 6, 7], "lot {lot}");
        }
    }

    #[test]
    fn a_name_is_two_to_sixteen_plain_characters() {
        assert_eq!(profile::name("  Hajun  ").as_deref(), Some("Hajun"));
        assert_eq!(profile::name("Mochi 482").as_deref(), Some("Mochi 482"));
        assert_eq!(profile::name("ab").as_deref(), Some("ab"));
        assert_eq!(profile::name("sixteen chars ok").as_deref(), Some("sixteen chars ok"));
        assert_eq!(profile::name("a"), None);
        assert_eq!(profile::name(" a "), None);
        assert_eq!(profile::name("seventeen chars!!"), None);
        // Nothing the game's font cannot draw.
        assert_eq!(profile::name("하준"), None);
        assert_eq!(profile::name("tab\there"), None);
        assert!(profile::look("boy") && profile::look("girl"));
        assert!(!profile::look("blue") && !profile::look("red") && !profile::look(""));
    }

    #[test]
    fn a_spot_is_about_ten_bytes() {
        let spot = Spot::of(Vec3::new(-234.56, 1.2, 345.67), Quat::from_rotation_y(2.0));
        let bytes = postcard::to_allocvec(&spot).unwrap();
        assert!(bytes.len() <= 13, "{} bytes", bytes.len());
        // On the car, a place on it, which is never far from its middle.
        let on = Spot::of(Vec3::new(-1.2, 1.0, 2.5), Quat::from_rotation_y(2.0)).aboard(Some(0));
        let bytes = postcard::to_allocvec(&on).unwrap();
        assert!(bytes.len() <= 12, "{} bytes", bytes.len());
    }

    #[test]
    fn a_spot_on_the_car_is_not_the_same_place_off_it() {
        let off = Spot::of(Vec3::new(1.0, 1.0, 2.0), Quat::IDENTITY);
        let on = off.aboard(Some(0));
        assert!(off.same(&off) && on.same(&on));
        assert!(!off.same(&on));
        let back: Spot = postcard::from_bytes(&postcard::to_allocvec(&on).unwrap()).unwrap();
        assert!(back.same(&on), "{back:?}");
    }
}
