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
/// arrives on one of the islands ([`ARENA_ISLANDS`], the town's that day); [`Ask::Respawn`], for
/// whoever falls off into the void; and [`Ask::Punch`] and [`Tell::Knocked`],
/// a punch landing on someone online. 6, that evening: the islands pulled
/// in to 130 m ([`ARENA_ISLANDS`]); an app with them at 200 m would arrive
/// over the void, fall, and have every move put back. 7, late that
/// evening: the first island the furnished one at +Z, its arrival at its
/// `spawn` marker. 8, at the day's end: every island's arrival at its own
/// marker, all four furnished.
///
/// 9 since the night of 2026-11-07: the town is `roundtownmap.glb` again, with the sea
/// round it and everyone arriving on the path round its pond
/// ([`TOWN_ARRIVAL_RADIUS`]), and the PvP map is a game of its own, Bridge
/// Wars, on [`Venue::Arena`] (Hajun: "i want the default map to open when
/// u open the app to be roundtownmap.glb ... when u press the game icon ui
/// button, it will open a popup game selector menu"); an app with the town
/// on the islands would arrive in the sea.
///
/// 10 since 2026-11-08: Bridge Wars is the server's online (Hajun: "the
/// bridge battle minigame doesnt allow queing with other players"): a game
/// is a room of its own with eight seats in four teams of two ([`ARENA_SEATS`],
/// [`arena_team`]), joined by [`Ask::Arena`] and followed through
/// [`ArenaRound`]; every punch, every hit on a respawn anchor
/// ([`Ask::HitAnchor`]) and every plank of a bridge ([`Ask::Rung`],
/// [`Plank`]) goes through the server, which keeps everyone's health.
///
/// Not bumped for the map of 2026-11-11, whose waiting room is 50 m higher
/// ([`bridge_wars::WAITING_FLOOR`]) and whose anchors stand up to 0.9 m from
/// where they did ([`ARENA_ANCHORS`]): an app with the map before is put
/// 50 m over its own room, which is open to the sky, and falls into it (the
/// server lets any fall go), and the anchors are well inside the slack a
/// hit on one is checked with.
///
/// 11 since the night of 2026-11-11: Bridge Wars has generators, a shop
/// and a timetable (Hajun: "i want to now make the generators functional
/// in the pvpmap ... these generators/items in map need to be server
/// synced"): everything a generator has made lies in the game's room as a
/// [`Loot`], what everyone holds is its [`Purses`], [`Ask::Buy`] and
/// [`Tell::Bought`] are the shop, a game lasts 25 minutes where it lasted
/// 10 ([`bridge_wars::PLAY_SECS`]), and past its eight seats a game may
/// seat zombies ([`bridge_wars::ZOMBIE_TEAM`]), whose seats an app from
/// before would take for a fifth pair of players.
pub const PROTOCOL_VERSION: u32 = 11;

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

/// How many players a game of Bridge Wars holds, bots included: two to each
/// of the four islands round the PvP map ([`ARENA_ISLANDS`]), a team to an
/// island ([`arena_team`]; since 2026-11-08, Hajun: "make it 8 player
/// rather than 4 player, so its 2v2v2v2"; one to an island until then).
pub const ARENA_SEATS: usize = 8;

/// How many teams a game of Bridge Wars has: one to an island.
pub const ARENA_TEAMS: usize = 4;

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
        .replicate::<ArenaRound>()
        .replicate::<Plank>()
        .replicate::<Loot>()
        .replicate::<Purses>()
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
    /// `pvptownmap.glb`: Bridge Wars' map, Round Town's middle in the sky
    /// with four islands at the ends of its bridges ([`ARENA_ISLANDS`]),
    /// where a game of Bridge Wars is played (since the night of 2026-11-07; the town
    /// itself on 2026-11-07). The game is the device's own for now, played
    /// with bots: online you are in a room of your own there, as at home,
    /// and nothing of where you stand is sent.
    Arena,
}

/// How far from the middle everyone stands on arriving on an island, in
/// metres: well inside the grass of the home and the lobby, which reaches 20 m
/// out. The town has its own ([`TOWN_ARRIVAL_RADIUS`]).
pub const ARRIVAL_RADIUS: f32 = 6.0;

/// How far from the middle everyone stands on arriving in the town, in
/// metres: on the middle of the path, 30 to 33 m out, round the pond that
/// fills the middle of `roundtownmap.glb`, where the town's own name shows.
/// Since 2026-10-30, but for 2026-11-07, when the town was the PvP map and
/// everyone arrived on its islands ([`ARENA_ISLANDS`]).
pub const TOWN_ARRIVAL_RADIUS: f32 = 31.5;

/// The arrival points of the four islands at the ends of the PvP map's
/// bridges ([`Venue::Arena`]), in metres from the middle of the map, where
/// everyone arrives in a game of Bridge Wars (Hajun, 2026-11-07, of the map
/// as the town: "make the player start in the center of one of the 4
/// bridge connected island"): the islands 130 m out along each axis, each
/// 20.8 m across (the islands' object scaled to 0.65 across the ground in
/// their map, its array of four with it; 200 m out and 32 m across that
/// morning), with a bridge 3.5 m wide from each to the middle, 75 m out,
/// until 2026-11-08, since when every bridge is built by walking; the
/// map's four smaller islands, 24 m across (`SapphireIslands`), are 125 m
/// out between them, on the diagonals. Each island's point is its middle (since the night of 2026-11-07, Hajun:
/// "make the spawnpoint in the center of the island so that u dont need
/// to hide the purple rotated cube"; for the evening before, the `spawn`
/// marker each island has, a purple pillar 5 to 6 m toward the middle,
/// which the game hid and now draws as it is): first the island at the
/// game's +Z (the bottom of the screen; Blender's +Y), then the others
/// clockwise from it seen from above. The islands' object is an array of
/// four about the map's middle, 130 m out on each axis (its mesh reaches
/// 140.4 m, the islands being 20.8 m across). Since Hajun's map of
/// 2026-11-11 the middle of each island is open ground between two market
/// stands, 4.65 m either side of it across the island (a desk lay across
/// it until then, which shoved whoever arrived to its edge), with the
/// anchor's slab 5 m toward the town and a low pad, `gen1` to `gen4`, 5 m
/// the other way. The server, which has no map, goes by this too.
pub const ARENA_ISLANDS: [Vec2; 4] = [
    Vec2::new(0.0, 130.0),
    Vec2::new(-130.0, 0.0),
    Vec2::new(0.0, -130.0),
    Vec2::new(130.0, 0.0),
];

/// How far from its island's middle everyone on it but the first stands on
/// arriving, in metres: round the one in the middle, a few metres apart.
pub const ISLAND_RING: f32 = 4.0;

/// How far across the ground each of the [`ARENA_ISLANDS`] reaches from its
/// middle, in metres: Hajun's islands are discs 20.8 m across (their object
/// scaled to 0.65). The server keeps its bots on them with this.
pub const ARENA_ISLAND_RADIUS: f32 = 10.4;

/// Where each team's respawn anchor stands, in metres from the middle of the
/// map, by team ([`arena_team`]): the purple pillar in the material `spawn`
/// on each island, 6 m from the island's middle toward the town, as
/// Hajun's `pvptownmap.glb` has them (a test in the game reads them off the
/// map). While it stands, whoever on its team dies comes back; broken, they
/// are out for good once they die (since 2026-11-08, Hajun: "make the purple
/// rotated cube thingy on each island the respawn anchor (similar to bed in
/// bedwars)"). As the map of 2026-11-11 has them, each a quarter turn on
/// from the last about the map's middle (the objects `spawn` to
/// `spawn.003`); until then, placed one by one, they were (0, 124.92),
/// (-123.84, 0.03), (-0.16, -124) and (123.83, -0.13).
pub const ARENA_ANCHORS: [Vec2; ARENA_TEAMS] = [
    Vec2::new(0.16, 124.0),
    Vec2::new(-124.0, 0.16),
    Vec2::new(-0.16, -124.0),
    Vec2::new(124.0, -0.16),
];

/// How far out from its middle an anchor reaches, across the ground, in
/// metres: the pillar is two pyramids base to base, 1.41 m square where
/// they meet, 0.71 m out at its sides and 1 m at its corners (a cube 1.4 m
/// across turned on its corner until 2026-11-11, about as wide). A punch
/// lands on it as on a body this wide.
pub const ANCHOR_RADIUS: f32 = 0.85;

/// Where the generators' pads are, in metres from the middle of the map, as
/// Hajun's `pvptownmap.glb` of 2026-11-11 has them (a test in the game
/// reads them off the map): the stone pad at the back of each team's
/// island, by team (the objects `gen1` to `gen4`), where its silver and its
/// gold come up; the pad in the middle of each of the four smaller islands
/// on the diagonals (`sapphiregen` to `sapphiregen4`), for sapphire; and
/// the four in the middle of the map, 35.8 m out on the axes, by the team
/// whose island each is toward (`amethystgen` to `.003`), for amethyst
/// (Hajun's `bridgewarsgameevent.txt`: "Silver and Gold spawn in each
/// player's personal island's square pad. Sapphire spawn in the 4 corner
/// islands's square pad. Amethyst spawn in the center map's 4 square
/// pads."). Each pad is 2 m square with its top [`PAD_TOP`] up.
pub const ARENA_PADS: [Vec2; ARENA_TEAMS] = [
    Vec2::new(0.16, 135.92),
    Vec2::new(-135.92, 0.16),
    Vec2::new(-0.16, -135.92),
    Vec2::new(135.92, -0.16),
];
pub const SAPPHIRE_PADS: [Vec2; 4] = [
    Vec2::new(88.39, 88.39),
    Vec2::new(-88.39, 88.39),
    Vec2::new(-88.39, -88.39),
    Vec2::new(88.39, -88.39),
];
pub const AMETHYST_PADS: [Vec2; 4] = [
    Vec2::new(0.16, 35.82),
    Vec2::new(-35.82, 0.16),
    Vec2::new(-0.16, -35.82),
    Vec2::new(35.82, -0.16),
];

/// How high a pad's top is, in metres, and how far it reaches from its
/// middle each way across the ground: a slab 2 m square, 0.25 m over the
/// island it lies on.
pub const PAD_TOP: f32 = 0.5;
pub const PAD_HALF: f32 = 1.0;

/// How far each of the four smaller islands reaches from its middle, in
/// metres: 24 m across (the map's `SapphireIslands`), its pad in the
/// middle of it.
pub const SAPPHIRE_ISLAND_RADIUS: f32 = 12.0;

/// Where each team's shop is, in metres from the middle of the map, by
/// team: the middle of the outline on the ground in front of the
/// red-roofed market stand on its island (the map's `activation`
/// material), to the right of whoever arrives there, 5 m from the island's
/// middle. Standing in the outline opens the shop on the device (the
/// game's `market`, which reads the outlines off the map); the server,
/// which has no map, takes a purchase only from someone near one of these
/// ([`bridge_wars::SHOP_SLACK`]). A test in the game keeps them where the
/// map has them.
pub const ARENA_SHOPS: [Vec2; ARENA_TEAMS] = [
    Vec2::new(5.0, 130.13),
    Vec2::new(-130.13, 5.0),
    Vec2::new(-5.0, -130.13),
    Vec2::new(130.13, -5.0),
];

/// Which of the [`ARENA_ISLANDS`] the player in `seat` arrives on, and is
/// put back on when they fall off (the game's `lobby`): seat by seat round
/// the four, so that the first four seats of a game of Bridge Wars are on
/// four different islands (Hajun: "when there is 2 players+ you will not
/// spawn in the same bridge connected island"), and the next four beside
/// them, a team to an island.
pub fn arena_island(seat: usize) -> usize {
    seat % ARENA_ISLANDS.len()
}

/// Which team the player in `seat` is on: the island's (2v2v2v2 since
/// 2026-11-08). Teammates cannot hurt each other, nor their own anchor.
/// A seat past the eight players' is a zombie's (since 2026-11-11,
/// [`bridge_wars::ZOMBIE_TEAM`]): on nobody's side but the zombies'.
pub fn arena_team(seat: usize) -> u8 {
    if seat >= ARENA_SEATS {
        return bridge_wars::ZOMBIE_TEAM;
    }
    arena_island(seat) as u8
}

/// What Bridge Wars' generators make, and its shop is paid in (since
/// 2026-11-11; Hajun's four item models, `silver_item.glb` and the rest):
/// silver and gold on each team's own island, sapphire on the four smaller
/// islands between them, amethyst in the middle of the map.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    Silver,
    Gold,
    Sapphire,
    Amethyst,
}

impl Item {
    /// Every item, in the order they are counted in ([`Purse`]) and shown
    /// down the screen.
    pub const ALL: [Item; 4] = [Item::Silver, Item::Gold, Item::Sapphire, Item::Amethyst];

    /// Its place in a [`Purse`].
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// How many of each [`Item`] someone holds, by [`Item::index`].
pub type Purse = [u16; 4];

/// Something Bridge Wars' shop sells.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ware {
    /// [`bridge_wars::BRIDGE_A_BUY`] metres of bridge to lay.
    Bridge,
}

/// Bridge Wars' rules, as the game plays them on the device (`arena`) and
/// the server does online (`server/src/arena.rs`): in one place, so that
/// the two never differ. The device's `assets/ui.ron` has the same numbers
/// for playing offline, where Hajun tunes them by hand; online the server's
/// rule.
pub mod bridge_wars {
    use super::{
        AMETHYST_PADS, ARENA_ISLAND_RADIUS, ARENA_ISLANDS, ARENA_PADS, ARENA_SEATS, ARENA_SHOPS, Item, PAD_HALF,
        Quat, SAPPHIRE_ISLAND_RADIUS, SAPPHIRE_PADS, TAU, Vec2, Vec3, Ware,
    };

    /// How long the waiting room waits for someone else to join before the
    /// computer fills the seats left, in seconds, counted from the last to
    /// join (Hajun, the night of 2026-11-07: "if there is no player joining for 10sec,
    /// fill it with bots").
    pub const BOT_WAIT: f32 = 10.0;
    /// How long a full room counts down before everyone is sent to their
    /// island, in seconds ("it will count down from 5").
    pub const COUNTDOWN: f32 = 5.0;
    /// How long a game lasts, in seconds, after which it is a draw and
    /// everyone is back in the town: 25 minutes since 2026-11-11 (Hajun's
    /// `bridgewarsgameevent.txt`: "25:00  Game ends in a draw"; 10 until
    /// then, "a (small) timer of 10 minute"). What happens on the way is
    /// [`TIMETABLE`].
    pub const PLAY_SECS: f32 = 1500.0;
    /// How long the result is up at the end, in seconds, before everyone
    /// still there is sent back to the town.
    pub const RESULTS_SECS: f32 = 6.0;
    /// The health everyone starts with, what a punch takes off it, and
    /// what a respawn anchor starts with: ten punches' worth (Hajun,
    /// 2026-11-08: twenty, "make its default hp enough so that u need to
    /// hit it 20 times to break it"; then, 2026-11-09, "make the respawn
    /// anchor's hp to be 1/2 of now").
    pub const HEALTH: f32 = 100.0;
    pub const DAMAGE: f32 = 12.0;
    pub const ANCHOR_HEALTH: f32 = 120.0;
    /// How long after a punch lands on a body, or an anchor, no other
    /// counts, in seconds: Minecraft's ten ticks of hurt resistance, the
    /// game's `melee.hurt_time`. The server keeps it too (2026-11-09,
    /// Hajun: the bots "seem to get knocked back too much by me maybe cuz
    /// i can hit them multiple times before they process it"): every punch
    /// a device landed went through before.
    pub const HURT_TIME: f32 = 0.5;
    /// How long the dead stay dead before standing again, in seconds, and
    /// how long after the last hit health starts to come back, and how much
    /// a second.
    pub const RESPAWN_SECS: f32 = 4.0;
    pub const REGEN_AFTER: f32 = 5.0;
    pub const REGEN_PER_SEC: f32 = 4.0;
    /// How fast everyone runs there, in metres a second; how far a bot sees
    /// whoever it goes for, in metres; the least time between its punches,
    /// in seconds; under what share of its health it runs away; and how far
    /// from the puncher's middle a punch reaches.
    pub const RUN_SPEED: f32 = 10.0;
    pub const HUNT_RADIUS: f32 = 24.0;
    pub const BOT_PUNCH_EVERY: f32 = 0.8;
    pub const FLEE_BELOW: f32 = 0.2;
    pub const REACH: f32 = 2.0;
    /// How far below the map a body has fallen before it is dead, in
    /// metres.
    pub const FALL_TO_DIE: f32 = 30.0;
    /// How hard a punch knocks, in metres a second, away from the puncher,
    /// and that much more from a puncher on the move; and what is left of
    /// the knock each twentieth of a second on the ground: the game's
    /// `melee` numbers, Minecraft 1.8.9's, for the server's bots, which
    /// it knocks about itself.
    pub const KNOCK_BACK: f32 = 10.0;
    pub const SPRINT_BONUS: f32 = 10.0;
    pub const KNOCK_UP: f32 = 8.0;
    pub const GROUND_DRAG: f32 = 0.546;
    pub const AIR_DRAG: f32 = 0.91;

    /// The waiting room, Hajun's glass box over the middle of the map
    /// (`pvptownmap.glb`, the object `waiting room`): where its floor is,
    /// in metres up, and how far from its middle everyone stands in it,
    /// facing the middle. The game checks the map agrees. 150.5 since
    /// Hajun's map of 2026-11-11, which has the room 50 m higher; 100.5
    /// until then.
    pub const WAITING_FLOOR: f32 = 150.5;
    pub const WAIT_RING: f32 = 3.0;

    /// Where the player in `seat` waits for the game: on a ring round the
    /// middle of the waiting room, on its floor, facing in, each an eighth
    /// of the turn on from the last.
    pub fn waiting_spot(seat: usize) -> (Vec3, Quat) {
        let angle = seat as f32 * TAU / ARENA_SEATS as f32;
        let at = Vec3::new(angle.sin() * WAIT_RING, WAITING_FLOOR, angle.cos() * WAIT_RING);
        let toward = -at.with_y(0.0).normalize_or(Vec3::NEG_Z);
        (at, Quat::from_rotation_y((-toward.x).atan2(-toward.z)))
    }

    /// Whether a punch thrown from `from` could reach an anchor standing at
    /// `at`: within `reach` of the puncher's middle, counting the anchor's
    /// own width. The server checks a hit on an anchor with this, where
    /// the puncher last said they stood, with slack for the way since; the
    /// device aims one with its own range.
    pub fn anchor_in_reach(from: Vec2, at: Vec2, reach: f32) -> bool {
        from.distance(at) <= reach + super::ANCHOR_RADIUS
    }

    // ------------------------------------------------------- the timetable

    /// Something that happens at a set time into a game (Hajun's
    /// `bridgewarsgameevent.txt`, 2026-11-11), which every device says in a
    /// line across the upper half of the screen as it happens.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Happening {
        /// Every generator of `item` makes its things faster from now on:
        /// "Sapphire II".
        Tier { item: Item, tier: u8 },
        /// "Spawn Anchor destruction: all remaining spawn anchors break":
        /// from now on whoever dies is out.
        AnchorsBreak,
        /// "Sudden death: zombies spawn all over the map" ([`ZOMBIES`]).
        SuddenDeath,
    }

    /// What happens when, in seconds into the fight, in order: the file's
    /// timetable, from "2:00 Sapphire II" to "15:00 Sudden death". Its
    /// first line, both generators at their first tier, is how a game
    /// begins, and its last, "25:00 Game ends in a draw", is [`PLAY_SECS`].
    pub const TIMETABLE: [(f32, Happening); 6] = [
        (120.0, Happening::Tier { item: Item::Sapphire, tier: 2 }),
        (240.0, Happening::Tier { item: Item::Amethyst, tier: 2 }),
        (360.0, Happening::Tier { item: Item::Sapphire, tier: 3 }),
        (480.0, Happening::Tier { item: Item::Amethyst, tier: 3 }),
        (600.0, Happening::AnchorsBreak),
        (900.0, Happening::SuddenDeath),
    ];

    /// What happens later than `before` and no later than `now`, both in
    /// seconds into the fight: this frame's events, given last frame's
    /// time and this one's.
    pub fn happening_between(before: f32, now: f32) -> impl Iterator<Item = Happening> {
        TIMETABLE
            .iter()
            .filter(move |(at, _)| *at > before && *at <= now)
            .map(|&(_, event)| event)
    }

    /// Which tier `item`'s generators are at, `elapsed` seconds into the
    /// fight: 1 to begin, and as [`TIMETABLE`] has raised it since.
    pub fn tier(item: Item, elapsed: f32) -> u8 {
        TIMETABLE
            .iter()
            .filter(|(at, _)| *at <= elapsed)
            .filter_map(|&(_, event)| match event {
                Happening::Tier { item: raised, tier } if raised == item => Some(tier),
                _ => None,
            })
            .max()
            .unwrap_or(1)
    }

    // ------------------------------------------------------ the generators

    /// How many generators the map has: a silver one and a gold one on
    /// each team's pad, a sapphire one on each of the four smaller
    /// islands, and four amethyst ones in the middle ([`generator`]).
    pub const GENERATORS: usize = 16;

    /// What the generator numbered `index` makes, and the middle of the
    /// pad it makes it on.
    pub fn generator(index: usize) -> (Item, Vec2) {
        match index {
            0..=3 => (Item::Silver, ARENA_PADS[index]),
            4..=7 => (Item::Gold, ARENA_PADS[index - 4]),
            8..=11 => (Item::Sapphire, SAPPHIRE_PADS[index - 8]),
            _ => (Item::Amethyst, AMETHYST_PADS[(index - 12) % AMETHYST_PADS.len()]),
        }
    }

    /// How long a generator of `item` at `tier` takes to make one, in
    /// seconds, and how much more or less than that each one may take
    /// (Hajun's file: "Silver: baserate = 1 per ~2 sec (+- 0.5 sec variable
    /// interval)", "Gold: baserate = 1 per ~7.5sec (+-1 sec variable
    /// interval)", "Sapphire: base rate = 1 per 30 s (Tier I) ... Tier2 =
    /// 1per23s, Tier3 = 1per 12s", "Amethyst: base rate = 1 per 40 s (Tier
    /// I) ... Tier2 = 1 per 30s, Tier3 = 1 per 20s").
    pub fn interval(item: Item, tier: u8) -> (f32, f32) {
        match (item, tier) {
            (Item::Silver, _) => (2.0, 0.5),
            (Item::Gold, _) => (7.5, 1.0),
            (Item::Sapphire, 0 | 1) => (30.0, 0.0),
            (Item::Sapphire, 2) => (23.0, 0.0),
            (Item::Sapphire, _) => (12.0, 0.0),
            (Item::Amethyst, 0 | 1) => (40.0, 0.0),
            (Item::Amethyst, 2) => (30.0, 0.0),
            (Item::Amethyst, _) => (20.0, 0.0),
        }
    }

    /// The most a generator of `item` holds on its pad before it stops
    /// making more until some are taken (the file's "max held on
    /// generator": 48, 6, 4 and 2).
    pub fn cap(item: Item) -> usize {
        match item {
            Item::Silver => 48,
            Item::Gold => 6,
            Item::Sapphire => 4,
            Item::Amethyst => 2,
        }
    }

    /// How far from the middle of its pad a thing may come up, in metres
    /// each way across the ground: well inside the pad's edge, the things
    /// being 0.7 m across.
    pub const PAD_SPREAD: f32 = 0.7;
    /// How many places on its pad a generator looks at for each thing it
    /// makes: it takes the one furthest from everything lying there, so
    /// that a pad fills evenly, and no two things come up in one spot
    /// while there is room (Hajun: "semi random location").
    const PLACES_TRIED: usize = 6;
    /// How near someone has to come to a thing to pick it up, in metres
    /// across the ground from their middle to its (Hajun: "the user can
    /// collect it by walking in it (short magnet)"): a body's own width
    /// and a little over a metre, so that standing at a pad's edge takes
    /// what lies along it, and walking across takes it all.
    pub const MAGNET: f32 = 1.5;
    /// How far over or under a pad's top someone's feet may be and still
    /// pick up what lies on it, in metres: a jump over it, and not the
    /// waiting room nor a fall past it.
    pub const MAGNET_HEIGHT: f32 = 3.0;

    /// One thing a generator has made, lying on its pad until someone
    /// picks it up: a number of its own, which generator made it, what it
    /// is, and where it lies across the ground.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Piece {
        pub id: u32,
        pub generator: u8,
        pub item: Item,
        pub at: Vec2,
    }

    /// Every generator of a game and everything they have made that still
    /// lies there: the server's online, and the device's own in a game
    /// played offline, run by the same rules either way. It rolls no dice
    /// of its own: whoever runs it hands it some, numbers from 0 up to 1.
    #[derive(Clone, Debug)]
    pub struct Mint {
        /// Seconds until each generator next makes one. It counts only
        /// while the generator holds less than its [`cap`].
        until: [f32; GENERATORS],
        pieces: Vec<Piece>,
        next: u32,
    }

    impl Mint {
        /// Every generator at the start of a game: nothing made yet, and
        /// the first of each a whole wait away.
        pub fn new(dice: &mut dyn FnMut() -> f32) -> Self {
            let mut until = [0.0; GENERATORS];
            for (index, wait) in until.iter_mut().enumerate() {
                *wait = wait_for(generator(index).0, 1, dice);
            }
            Self {
                until,
                pieces: Vec::new(),
                next: 0,
            }
        }

        /// Everything lying on the pads.
        pub fn pieces(&self) -> &[Piece] {
            &self.pieces
        }

        /// How many things the generator numbered `index` holds.
        pub fn held(&self, index: usize) -> usize {
            self.pieces.iter().filter(|piece| usize::from(piece.generator) == index).count()
        }

        /// Runs every generator for `dt` seconds, `elapsed` seconds into
        /// the fight (which says what tier each is at): what they made.
        /// A generator holding its [`cap`] waits, its next thing a whole
        /// wait off from when one is taken.
        pub fn run(&mut self, dt: f32, elapsed: f32, dice: &mut dyn FnMut() -> f32) -> Vec<Piece> {
            let mut made = Vec::new();
            for index in 0..GENERATORS {
                let (item, pad) = generator(index);
                if self.held(index) >= cap(item) {
                    continue;
                }
                let level = tier(item, elapsed);
                // A tier raised mid-wait cuts the wait to the new tier's.
                let (every, spread) = interval(item, level);
                self.until[index] = self.until[index].min(every + spread) - dt;
                // One at the most a go: a frame is far shorter than a wait.
                if self.until[index] > 0.0 {
                    continue;
                }
                self.until[index] += wait_for(item, level, dice);
                let piece = Piece {
                    id: self.next,
                    generator: index as u8,
                    item,
                    at: self.place(pad, dice),
                };
                self.next = self.next.wrapping_add(1);
                self.pieces.push(piece);
                made.push(piece);
            }
            made
        }

        /// Somewhere on the pad round `pad` for a new thing: of a few
        /// places at random, the one furthest from everything already
        /// lying on that pad.
        fn place(&self, pad: Vec2, dice: &mut dyn FnMut() -> f32) -> Vec2 {
            let lying: Vec<Vec2> = self
                .pieces
                .iter()
                .map(|piece| piece.at)
                .filter(|at| (*at - pad).abs().max_element() <= PAD_HALF)
                .collect();
            let mut best = (f32::NEG_INFINITY, pad);
            for _ in 0..PLACES_TRIED {
                let off = Vec2::new(dice() * 2.0 - 1.0, dice() * 2.0 - 1.0) * PAD_SPREAD;
                let at = pad + off;
                let room = lying
                    .iter()
                    .map(|other| other.distance_squared(at))
                    .fold(f32::INFINITY, f32::min);
                if room > best.0 {
                    best = (room, at);
                }
            }
            best.1
        }

        /// Whoever stands at `at` across the ground picks up everything
        /// within [`MAGNET`] of them: what they took, no longer lying
        /// there for anyone else.
        pub fn pick_up(&mut self, at: Vec2) -> Vec<Piece> {
            let reach = MAGNET * MAGNET;
            let (taken, left): (Vec<Piece>, Vec<Piece>) = self
                .pieces
                .iter()
                .copied()
                .partition(|piece| piece.at.distance_squared(at) <= reach);
            self.pieces = left;
            taken
        }
    }

    /// How long a generator of `item` at `tier` waits before its next
    /// thing, in seconds: its [`interval`], more or less its spread.
    fn wait_for(item: Item, tier: u8, dice: &mut dyn FnMut() -> f32) -> f32 {
        let (every, spread) = interval(item, tier);
        every + (dice() * 2.0 - 1.0) * spread
    }

    // ------------------------------------------------------------ the shop

    /// What `ware` costs: so many of an item (Hajun, 2026-11-11: "right
    /// now just have 'bridge' as the option, which should cost 4 silver
    /// for 50m of bridge").
    pub fn price(ware: Ware) -> (Item, u16) {
        match ware {
            Ware::Bridge => (Item::Silver, 4),
        }
    }

    /// How many metres of bridge one [`Ware::Bridge`] is.
    pub const BRIDGE_A_BUY: f32 = 50.0;

    /// How far from a shop ([`ARENA_SHOPS`]) someone may be, where they
    /// last said they stood, for the server to sell to them, in metres:
    /// the outline in front of the stand reaches 1.5 m from its middle,
    /// and the rest is for the way they may have gone since.
    pub const SHOP_SLACK: f32 = 5.0;

    /// Whether someone standing at `at` across the ground is near enough
    /// to a shop to buy from it, as the server checks it.
    pub fn at_a_shop(at: Vec2) -> bool {
        ARENA_SHOPS.iter().any(|shop| shop.distance(at) <= SHOP_SLACK)
    }

    // --------------------------------------------------------- the zombies

    /// The team of the zombies of sudden death ([`Happening::SuddenDeath`]):
    /// nobody's, so everyone is theirs to go for and they are everyone's.
    /// They sit in the seats past the players' eight (`arena_team`).
    pub const ZOMBIE_TEAM: u8 = super::ARENA_TEAMS as u8;
    /// How many zombies sudden death brings: one to each team's island,
    /// one to each of the smaller islands between them, and four in the
    /// middle of the map ([`zombie_home`]; the file says only "zombies
    /// spawn all over the map": the number, and the rest about them, is
    /// not Hajun's).
    pub const ZOMBIES: usize = 12;
    /// What a zombie is called over its head, and the look it wears, which
    /// the game draws as the body everyone has, gone green.
    pub const ZOMBIE_NAME: &str = "Zombie";
    pub const ZOMBIE_LOOK: &str = "zombie";
    /// How long a zombie stays down before another stands in its place, in
    /// seconds: sudden death does not let up.
    pub const ZOMBIE_RESPAWN_SECS: f32 = 12.0;
    /// How fast a zombie comes at whoever it goes for, in metres a second:
    /// slower than anyone runs ([`RUN_SPEED`]), so that it can be got away
    /// from, though not for long on an island.
    pub const ZOMBIE_SPEED: f32 = 6.0;
    /// How much of a punch's knock a zombie takes, as a share: it is not
    /// to be batted off an island as a player is. Knocked as anyone is,
    /// the one that rose on each team's island, 4 m from its middle, was
    /// over the edge at the first punch thrown on the run, which carries a
    /// body 7 m: all four of a game's were gone within three seconds of
    /// rising, and again each time they stood again (seen on a server on
    /// this PC, 2026-11-11).
    pub const ZOMBIE_KNOCK: f32 = 0.3;

    /// The ground the zombie numbered `index` keeps to, as the middle of a
    /// disc and how far it reaches: a team's island, one of the smaller
    /// islands, or the middle of the map, out to the edge of its paving.
    /// The server has no map, and keeps each to its disc as it keeps its
    /// other players to their islands.
    pub fn zombie_home(index: usize) -> (Vec2, f32) {
        match index % ZOMBIES {
            at @ 0..=3 => (ARENA_ISLANDS[at], ARENA_ISLAND_RADIUS),
            at @ 4..=7 => (SAPPHIRE_PADS[at - 4], SAPPHIRE_ISLAND_RADIUS),
            _ => (Vec2::ZERO, TOWN_RADIUS),
        }
    }

    /// How far the middle of the map reaches for a zombie, in metres: the
    /// paving ends 76 m out.
    pub const TOWN_RADIUS: f32 = 74.0;

    /// Where the zombie numbered `index` comes up, and comes up again: on
    /// a team's island, 4 m to one side of its middle, clear of where its
    /// players stand again; on a smaller island, 4 m from its pad toward
    /// the middle of the map; and in the middle, on the path round the
    /// pond, between the roads out.
    pub fn zombie_spot(index: usize) -> Vec2 {
        let index = index % ZOMBIES;
        let (home, _) = zombie_home(index);
        match index {
            0..=3 => home + Vec2::new(home.y, -home.x).normalize() * -4.0,
            4..=7 => home - home.normalize() * 4.0,
            _ => {
                let angle = (index - 8) as f32 * TAU / 4.0 + TAU / 8.0;
                Vec2::new(angle.cos(), angle.sin()) * super::TOWN_ARRIVAL_RADIUS
            }
        }
    }
}

/// Where the player in `seat` stands on arriving on `venue`, and which way
/// they face. On the PvP map, on their island ([`arena_island`]): the first
/// on each island at its marker, and the rest round that [`ISLAND_RING`]
/// out, every one facing the middle, down the bridge. Anywhere else, on a
/// ring round the middle, facing in — round the pond, in the town
/// ([`TOWN_ARRIVAL_RADIUS`]) — the first eight seats each an eighth of the
/// turn on from the one before, the next eight between them, and past
/// sixteen a ring further out. The server works out arrivals with this too,
/// so that everyone sees you arrive where you do.
pub fn arrival(venue: Venue, seat: usize) -> (Vec3, Quat) {
    if venue == Venue::Arena {
        let island = ARENA_ISLANDS[arena_island(seat)];
        let round = seat / ARENA_ISLANDS.len();
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
    let first = if venue == Venue::Town { TOWN_ARRIVAL_RADIUS } else { ARRIVAL_RADIUS };
    let radius = first + ring as f32 * 2.0;
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

// ------------------------------------------------------------- Bridge Wars

/// A game of Bridge Wars as everyone in it sees it (since 2026-11-08, when
/// the server took the game over so that people could play it together):
/// how far it has got, how long is left of that, everyone's health by
/// seat, each team's respawn anchor, and who is out for good. Sent to the
/// room whenever any of it changes: a few dozen bytes, a few times a second
/// while anyone's health is coming back.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ArenaRound {
    pub stage: ArenaStage,
    /// Whole seconds left of this stage, rounded up: what the board shows.
    pub left: u16,
    /// Everyone's health by seat, in whole points of [`bridge_wars::HEALTH`];
    /// 0 is dead, until they stand again.
    pub health: Vec<u16>,
    /// Each team's respawn anchor, in whole points of
    /// [`bridge_wars::ANCHOR_HEALTH`]; 0 is broken.
    pub anchors: [u16; ARENA_TEAMS],
    /// The seats that are out for good, a bit to a seat: dead with their
    /// anchor broken.
    pub out: u8,
}

impl ArenaRound {
    /// Whether the player in `seat` is out for good.
    pub fn is_out(&self, seat: usize) -> bool {
        seat < 8 && self.out & (1 << seat) != 0
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum ArenaStage {
    /// In the waiting room, waiting for the room to fill.
    Waiting,
    /// Full, counting down to go.
    Countdown,
    /// Everyone on their island, fighting.
    Playing,
    /// Done: `winner` is the team left standing, or none, the time having
    /// run out with more than one.
    Over { winner: Option<u8> },
}

/// One rung of a bridge someone has built in a game of Bridge Wars (the
/// game's `bridge`): whose, which of their chains of planks, which rung of
/// it, where it lies across the ground and which way it heads. Sent once
/// as it is laid, so that everyone in the game sees the plank between it
/// and the rung before; never moved.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[component(immutable)]
pub struct Plank {
    pub seat: u8,
    pub chain: u16,
    pub index: u16,
    pub x: f32,
    pub z: f32,
    /// Which way the rung heads, across the ground, in radians from the
    /// game's -Z, as a turn about the upright.
    pub heading: f32,
}

/// One thing a generator has made in a game of Bridge Wars, lying on its
/// pad ([`bridge_wars::Mint`], since 2026-11-11): what it is, and where it
/// lies across the ground, on a pad's top ([`PAD_TOP`]). Sent once as it
/// comes up, to everyone in the game, and gone for everyone the moment
/// the server has someone pick it up (Hajun: "other player taking the
/// item from the generators make it so you cant take it too"); never
/// moved.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[component(immutable)]
pub struct Loot {
    pub item: Item,
    pub x: f32,
    pub z: f32,
}

/// What everyone in a game of Bridge Wars holds, by seat: how many of each
/// [`Item`]. On the game's own entity beside its [`ArenaRound`], and sent
/// to the room whenever anyone's changes: something picked up, or
/// something bought.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Purses(pub Vec<Purse>);

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
    /// hit does in Minecraft. In a game of Bridge Wars the server takes the
    /// punch's damage off them too, if they are not on my team.
    Punch { who: u64, running: bool },
    /// Join a game of Bridge Wars: from the town, or from a game that is
    /// over for me, for another ([`Tell::Seated`] answers).
    Arena,
    /// My punch landed on `team`'s respawn anchor ([`ARENA_ANCHORS`]): the
    /// server takes the damage off it, if it is not my own.
    HitAnchor { team: u8 },
    /// I have laid this rung of a bridge, as [`Plank`] has it: everyone in
    /// the game is to see the plank.
    Rung {
        chain: u16,
        index: u16,
        x: f32,
        z: f32,
        heading: f32,
    },
    /// Sell me this, from Bridge Wars' shop: its price comes out of what I
    /// hold ([`bridge_wars::price`]), and [`Tell::Bought`] says it is
    /// mine; with too little, or nowhere near a shop, nothing happens.
    Buy(Ware),
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
    /// You are in a game of Bridge Wars, in this seat: your island is
    /// [`arena_island`] of it, your team [`arena_team`].
    Seated { seat: u8 },
    /// The player whose [`Member::id`] is `by` punched you, standing at
    /// `from` (in the island's space, or on a bus, in the bus's): your own
    /// device knocks your body back from there, and sends where it goes as
    /// it does any move. `running`, they hit you on the move.
    Knocked { by: u64, from: Spot, running: bool },
    /// What you asked for cannot be done.
    Refused(Refusal),
    /// The shop has sold you this ([`Ask::Buy`]): its price is out of what
    /// you hold, as the room's [`Purses`] will show.
    Bought(Ware),
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
    fn the_first_four_of_a_game_of_bridge_wars_arrive_on_four_islands_facing_the_middle() {
        for seat in 0..ARENA_TEAMS {
            let (at, turn) = arrival(Venue::Arena, seat);
            let island = ARENA_ISLANDS[seat];
            assert!(at.xz().distance(island) < 1e-4, "seat {seat}: {at} for {island}");
            assert_eq!(at.y, 0.0);
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(-island.normalize()) < 1e-4, "seat {seat} faces {facing}");
            assert_eq!(arena_island(seat), seat);
        }
        // Past them, on the islands too, near the middle, each on the
        // island its seat says: the second of each team beside the first.
        for seat in ARENA_TEAMS..TOWN_SEATS * 2 {
            let (at, _) = arrival(Venue::Arena, seat);
            let island = ARENA_ISLANDS[arena_island(seat)];
            let out = at.xz().distance(island);
            assert!(out > 3.0 && out < 8.0, "seat {seat}: {out} m from its island's middle");
            assert!(out < ARENA_ISLAND_RADIUS - 2.0, "seat {seat}: off the island");
        }
        // Eight seats, two to a team, a team to an island.
        assert_eq!(ARENA_SEATS, 2 * ARENA_TEAMS);
        for seat in 0..ARENA_SEATS {
            assert_eq!(arena_team(seat), arena_island(seat) as u8);
            assert_eq!(arena_team(seat), arena_team((seat + ARENA_TEAMS) % ARENA_SEATS));
            assert_ne!(arena_team(seat), arena_team((seat + 1) % ARENA_SEATS));
        }
        // Each team's anchor stands on its island, toward the town.
        for team in 0..ARENA_TEAMS {
            let (anchor, island) = (ARENA_ANCHORS[team], ARENA_ISLANDS[team]);
            let out = anchor.distance(island);
            assert!(out > 4.0 && out < 7.0, "team {team}: anchor {out} m from its island's middle");
            assert!(anchor.length() < island.length(), "team {team}: anchor beyond its island");
            assert!(bridge_wars::anchor_in_reach(anchor + Vec2::X * 2.5, anchor, bridge_wars::REACH));
            assert!(!bridge_wars::anchor_in_reach(anchor + Vec2::X * 4.0, anchor, bridge_wars::REACH));
        }
    }

    #[test]
    fn everyone_waits_in_the_room_facing_its_middle_and_apart() {
        let spots: Vec<(Vec3, Quat)> = (0..ARENA_SEATS).map(bridge_wars::waiting_spot).collect();
        for (seat, &(at, turn)) in spots.iter().enumerate() {
            assert_eq!(at.y, bridge_wars::WAITING_FLOOR);
            assert!((at.xz().length() - bridge_wars::WAIT_RING).abs() < 1e-4, "seat {seat} at {at}");
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(-at.xz().normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
        for (a, &(at, _)) in spots.iter().enumerate() {
            for &(other, _) in &spots[a + 1..] {
                assert!(at.distance(other) > 1.0, "{at} and {other}");
            }
        }
        let round = ArenaRound {
            stage: ArenaStage::Playing,
            left: 10,
            health: vec![100; ARENA_SEATS],
            anchors: [240; ARENA_TEAMS],
            out: 0b0000_0101,
        };
        assert!(round.is_out(0) && round.is_out(2) && !round.is_out(1) && !round.is_out(9));
    }

    #[test]
    fn in_the_town_everyone_arrives_on_the_path_round_the_pond_facing_in() {
        for seat in 0..TOWN_SEATS * 2 {
            let (at, turn) = arrival(Venue::Town, seat);
            let out = at.xz().length();
            let ring = if seat < 16 { TOWN_ARRIVAL_RADIUS } else { TOWN_ARRIVAL_RADIUS + 2.0 };
            assert!((out - ring).abs() < 1e-3, "seat {seat}: {out} m out");
            assert_eq!(at.y, 0.0);
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(-at.xz().normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
    }

    #[test]
    fn a_full_town_arrives_apart() {
        for venue in [Venue::Town, Venue::Lobby, Venue::Arena] {
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

    /// Dice that always come up the same: 0.5 is every wait at its
    /// middle and every thing in the middle of its pad.
    fn steady(value: f32) -> impl FnMut() -> f32 {
        move || value
    }

    /// Dice that go round a few values, for places apart on a pad.
    fn rolling() -> impl FnMut() -> f32 {
        let mut state = 0x2545_F491u32;
        move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state >> 8) as f32 / (1u32 << 24) as f32
        }
    }

    #[test]
    fn the_generators_make_what_the_file_says_as_fast_as_it_says_and_hold_no_more_than_it_says() {
        use bridge_wars::{GENERATORS, Mint, PAD_SPREAD, cap, generator};
        let mut dice = steady(0.5);
        let mut mint = Mint::new(&mut dice);
        // A silver every 2 s and a gold every 7.5 s from each team's pad,
        // a sapphire every 30 s and an amethyst every 40 s: after 29 s,
        // 14 silver and 3 gold a pad, and neither of the others yet.
        let (dt, mut elapsed) = (1.0 / 60.0, 0.0);
        let mut made = Vec::new();
        while elapsed < 29.0 {
            elapsed += dt;
            made.extend(mint.run(dt, elapsed, &mut dice));
        }
        let count = |made: &[bridge_wars::Piece], item: Item| made.iter().filter(|piece| piece.item == item).count();
        assert_eq!(count(&made, Item::Silver), 14 * 4);
        assert_eq!(count(&made, Item::Gold), 3 * 4);
        assert_eq!(count(&made, Item::Sapphire) + count(&made, Item::Amethyst), 0);
        // Each lies on its own generator's pad, well inside its edge, at
        // a number nobody else has.
        for piece in &made {
            let (item, pad) = generator(usize::from(piece.generator));
            assert_eq!(item, piece.item);
            assert!((piece.at - pad).abs().max_element() <= PAD_SPREAD + 1e-4, "{piece:?} off its pad at {pad}");
        }
        let mut ids: Vec<u32> = made.iter().map(|piece| piece.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), made.len());
        // Left alone for two minutes, every generator holds what it may
        // and no more: 48, 6, 4 and 2.
        while elapsed < 125.0 {
            elapsed += dt;
            mint.run(dt, elapsed, &mut dice);
        }
        for index in 0..GENERATORS {
            let (item, _) = generator(index);
            assert_eq!(mint.held(index), cap(item), "generator {index}, {item:?}");
        }
        assert_eq!(mint.pieces().len(), 4 * (48 + 6 + 4 + 2));
        // Emptied, a pad's next thing is a whole wait off, not at once.
        let pad = ARENA_PADS[0];
        let taken = mint.pick_up(pad);
        assert_eq!(taken.len(), 48 + 6, "everything on a team's pad is within reach of its middle");
        assert!(mint.run(dt, elapsed + dt, &mut dice).is_empty());
        let mut waited = 0.0;
        let next = loop {
            waited += dt;
            let made = mint.run(dt, elapsed + waited, &mut dice);
            if let Some(piece) = made.first() {
                break *piece;
            }
            assert!(waited < 3.0, "nothing after 3 s");
        };
        assert_eq!(next.item, Item::Silver);
        assert!(waited > 1.9, "the next silver after only {waited} s");
    }

    #[test]
    fn the_generators_speed_up_as_the_timetable_says() {
        use bridge_wars::{TIMETABLE, Happening, Mint, PLAY_SECS, happening_between, interval, tier};
        assert_eq!(tier(Item::Sapphire, 0.0), 1);
        assert_eq!(tier(Item::Sapphire, 119.9), 1);
        assert_eq!(tier(Item::Sapphire, 120.0), 2);
        assert_eq!(tier(Item::Amethyst, 239.0), 1);
        assert_eq!(tier(Item::Amethyst, 240.0), 2);
        assert_eq!(tier(Item::Sapphire, 360.0), 3);
        assert_eq!(tier(Item::Amethyst, 480.0), 3);
        assert_eq!(tier(Item::Sapphire, PLAY_SECS), 3);
        assert_eq!(tier(Item::Silver, PLAY_SECS), 1);
        assert_eq!(tier(Item::Gold, PLAY_SECS), 1);
        assert_eq!(interval(Item::Sapphire, 1).0, 30.0);
        assert_eq!(interval(Item::Sapphire, 2).0, 23.0);
        assert_eq!(interval(Item::Sapphire, 3).0, 12.0);
        assert_eq!(interval(Item::Amethyst, 1).0, 40.0);
        assert_eq!(interval(Item::Amethyst, 2).0, 30.0);
        assert_eq!(interval(Item::Amethyst, 3).0, 20.0);
        // In order, each once, all before the draw; and an event is this
        // frame's only in the frame its time passes.
        assert!(TIMETABLE.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(TIMETABLE.iter().all(|(at, _)| *at > 0.0 && *at < PLAY_SECS));
        assert_eq!(
            happening_between(119.99, 120.01).collect::<Vec<_>>(),
            vec![Happening::Tier { item: Item::Sapphire, tier: 2 }]
        );
        assert_eq!(happening_between(120.01, 120.03).count(), 0);
        assert_eq!(happening_between(0.0, PLAY_SECS).count(), TIMETABLE.len());
        assert_eq!(happening_between(599.0, 601.0).collect::<Vec<_>>(), vec![Happening::AnchorsBreak]);
        assert_eq!(happening_between(899.0, 901.0).collect::<Vec<_>>(), vec![Happening::SuddenDeath]);
        // Someone standing on a sapphire pad the whole game, taking each
        // as it comes: 4 in the first two minutes, one every 23 s for the
        // next four, and one every 12 s after.
        let mut dice = steady(0.5);
        let mut mint = Mint::new(&mut dice);
        let (dt, mut elapsed) = (1.0 / 30.0, 0.0);
        let mut taken = [0usize; 3];
        while elapsed < 600.0 {
            elapsed += dt;
            mint.run(dt, elapsed, &mut dice);
            let got = mint.pick_up(SAPPHIRE_PADS[0]);
            assert!(got.iter().all(|piece| piece.item == Item::Sapphire));
            let stretch = if elapsed <= 120.5 {
                0
            } else if elapsed <= 360.5 {
                1
            } else {
                2
            };
            taken[stretch] += got.len();
        }
        assert_eq!(taken[0], 4, "{taken:?}");
        assert!((10..=11).contains(&taken[1]), "{taken:?}");
        assert!((19..=20).contains(&taken[2]), "{taken:?}");
    }

    #[test]
    fn a_thing_is_picked_up_from_near_it_once_and_things_come_up_apart() {
        use bridge_wars::{MAGNET, Mint, generator};
        let mut dice = rolling();
        let mut mint = Mint::new(&mut dice);
        let (dt, mut elapsed) = (0.05, 0.0);
        while elapsed < 20.0 {
            elapsed += dt;
            mint.run(dt, elapsed, &mut dice);
        }
        // Ten silver or so and two gold on each team's pad by now, no two
        // of a pad's in one spot.
        let pad = ARENA_PADS[2];
        let lying: Vec<Vec2> = mint
            .pieces()
            .iter()
            .filter(|piece| generator(usize::from(piece.generator)).1 == pad)
            .map(|piece| piece.at)
            .collect();
        assert!(lying.len() >= 10, "{} things", lying.len());
        for (k, a) in lying.iter().enumerate() {
            for b in &lying[k + 1..] {
                assert!(a.distance(*b) > 0.02, "two things at {a}");
            }
        }
        // From 3 m off the pad nothing is in reach; from its middle all
        // of it is, once: a second go, or someone else's, finds nothing.
        let all = mint.pieces().len();
        assert!(mint.pick_up(pad + Vec2::new(MAGNET + 1.0 + 0.5, 0.0)).is_empty());
        assert_eq!(mint.pieces().len(), all);
        let taken = mint.pick_up(pad);
        assert_eq!(taken.len(), lying.len());
        assert!(mint.pick_up(pad).is_empty());
        assert_eq!(mint.pieces().len(), all - taken.len());
        // Another team's pad is untouched by it.
        assert!(!mint.pick_up(ARENA_PADS[0]).is_empty());
    }

    #[test]
    fn the_pads_the_shops_and_the_zombies_are_where_the_map_has_room_for_them() {
        use bridge_wars::{
            BRIDGE_A_BUY, SHOP_SLACK, TOWN_RADIUS, ZOMBIE_TEAM, ZOMBIES, at_a_shop, price, zombie_home, zombie_spot,
        };
        for team in 0..ARENA_TEAMS {
            let island = ARENA_ISLANDS[team];
            // Each team's pad is on its island, behind its middle, the
            // anchor being before it; and its shop beside its middle.
            let pad = ARENA_PADS[team];
            assert!(pad.distance(island) + PAD_HALF * 1.5 < ARENA_ISLAND_RADIUS, "team {team}'s pad");
            assert!(pad.length() > island.length() && ARENA_ANCHORS[team].length() < island.length());
            let shop = ARENA_SHOPS[team];
            assert!((shop.distance(island) - 5.0).abs() < 0.1, "team {team}'s shop");
            // Bought from at the shop and a few steps off it, and not
            // from the pad, nor from the anchor.
            assert!(at_a_shop(shop) && at_a_shop(shop + Vec2::new(SHOP_SLACK - 0.1, 0.0)));
            assert!(!at_a_shop(pad) && !at_a_shop(ARENA_ANCHORS[team]), "team {team}");
            // Its amethyst pad is the one toward it, in the middle.
            assert!(AMETHYST_PADS[team].normalize().dot(island.normalize()) > 0.99);
            assert!(AMETHYST_PADS[team].length() < 40.0);
        }
        for pad in SAPPHIRE_PADS {
            assert!((pad.length() - 125.0).abs() < 0.1, "{pad}");
            assert!(ARENA_ISLANDS.iter().all(|island| island.distance(pad) > ARENA_ISLAND_RADIUS + SAPPHIRE_ISLAND_RADIUS));
        }
        assert_eq!(price(Ware::Bridge), (Item::Silver, 4));
        assert_eq!(BRIDGE_A_BUY, 50.0);
        // The seats past the players' are the zombies', on nobody's team
        // but their own; each comes up inside the ground it keeps to,
        // with room round it, and no two in one place.
        for seat in 0..ARENA_SEATS {
            assert!(arena_team(seat) < ZOMBIE_TEAM);
        }
        assert_eq!(arena_team(ARENA_SEATS), ZOMBIE_TEAM);
        assert_eq!(arena_team(ARENA_SEATS + ZOMBIES - 1), ZOMBIE_TEAM);
        let spots: Vec<Vec2> = (0..ZOMBIES).map(zombie_spot).collect();
        for (index, spot) in spots.iter().enumerate() {
            let (home, reach) = zombie_home(index);
            assert!(spot.distance(home) < reach - 2.0, "zombie {index} at {spot}, home {home} reaching {reach}");
            assert!(spots.iter().filter(|other| other.distance(*spot) < 3.0).count() == 1, "zombie {index}");
        }
        // The four in the middle are on the path round the pond, clear
        // of the amethyst pads.
        for index in 8..ZOMBIES {
            assert_eq!(zombie_home(index), (Vec2::ZERO, TOWN_RADIUS));
            assert!(AMETHYST_PADS.iter().all(|pad| pad.distance(spots[index]) > 10.0));
        }
    }

    #[test]
    fn a_thing_on_a_pad_is_a_few_bytes_and_a_purse_is_four_counts() {
        let loot = Loot {
            item: Item::Sapphire,
            x: -88.39,
            z: 88.39,
        };
        let bytes = postcard::to_allocvec(&loot).unwrap();
        assert!(bytes.len() <= 9, "{} bytes", bytes.len());
        let back: Loot = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, loot);
        let mut purse: Purse = [0; 4];
        purse[Item::Silver.index()] = 7;
        purse[Item::Amethyst.index()] = 1;
        assert_eq!(purse, [7, 0, 0, 1]);
        assert_eq!(Item::ALL.map(Item::index), [0, 1, 2, 3]);
    }
}
