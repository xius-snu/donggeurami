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
///
/// 12 since late on 2026-11-11: [`Tell::Bought`] of a [`Ware::Bridge`] is 25 m of
/// bridge where it was 50 ([`bridge_wars::BRIDGE_A_BUY`]; Hajun: "make it
/// so 4 silver gets you 25m of bridge"), which an app from before would
/// still count as 50. With it, though no app from before would have
/// minded: a thing a generator makes drops onto its pad and cannot be
/// picked up until it has lain there a moment, and a generator's count to
/// its next thing runs on while it is full ([`bridge_wars::next_at`]), so
/// that every device can write the same count over it.
///
/// Not bumped on 2026-11-12, when a thing came to be picked up as it
/// drops, as soon as it is within reach ([`bridge_wars::REACH_OVER`]),
/// rather than once it has lain a moment: who has what, and when, is the
/// server's to say online, and an app from before shows it as the server
/// says it.
///
/// 13 since 2026-11-12: Bridge Wars' two shops sell swords, shields, things
/// to use and upgrades for a team (Hajun's `bridgewarsgameevent.txt`):
/// [`Ware`] has them all, what everyone has is the game's [`Kits`] and
/// what each team has bought its [`Upgraded`], a fireball in the air is a
/// [`Fireball`], and [`Ask::Use`] uses what is on the second button.
pub const PROTOCOL_VERSION: u32 = 13;

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
        .replicate::<Kits>()
        .replicate::<Upgraded>()
        .replicate::<Fireball>()
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

/// Where each team's upgrade shop is, as [`ARENA_SHOPS`] are: the middle
/// of the outline in front of the blue-roofed stand on its island (the
/// map's `bluemarket`), to the left of whoever arrives there, across the
/// island's middle from the red one.
pub const ARENA_UPGRADE_SHOPS: [Vec2; ARENA_TEAMS] = [
    Vec2::new(-5.0, 130.13),
    Vec2::new(-130.13, -5.0),
    Vec2::new(5.0, -130.13),
    Vec2::new(130.13, 5.0),
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

/// Something Bridge Wars' shops sell (Hajun's `bridgewarsgameevent.txt`;
/// only the bridge until 2026-11-12). The item shop, at the red stand,
/// sells what is yours alone; the upgrade shop, at the blue one, what is
/// your whole team's ([`bridge_wars::sold_at`]). What each costs, and
/// whether it can be bought at all, is [`bridge_wars::price`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ware {
    /// [`bridge_wars::BRIDGE_A_BUY`] metres of bridge to lay.
    Bridge,
    /// A sword, by its tier: 1 the basic one, 2 the silver, 3 the aura.
    /// Only one better than the one in hand; lost on dying.
    Sword(u8),
    /// A shield, by its tier, 1 to 3: a share of every blow kept off.
    /// Only one better than the one worn; kept for the game.
    Shield(u8),
    /// One more of something to use, for the second button, which holds
    /// one kind of thing at a time.
    Utility(Utility),
    /// The next step of the team's own generator: faster, faster again,
    /// amethyst too, and fastest.
    Forge,
    /// Ten more damage on every sword the team swings. Once.
    StrongerSwords,
    /// The next step of the team's shields: so much more kept off, four
    /// times over.
    BetterShields,
    /// The team heals while it stands on its own island.
    HealPool,
    /// One more blindness trap waiting on the team's island, three at the
    /// most.
    Trap,
}

/// Something to use in Bridge Wars, bought at the item shop and put on the
/// second button, however many of it ([`Kit::count`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Utility {
    /// Aimed and thrown: it flies straight and bursts at the end of its
    /// range, or on whoever it meets ([`bridge_wars::Flight`]).
    Fireball,
    /// Aimed and sent off: a builder who walks that way laying a bridge.
    Builder,
    /// Drunk: [`bridge_wars::FAST_SECS`] of running faster.
    Speed,
    /// Drunk: [`bridge_wars::UNSEEN_SECS`] of not being seen.
    Invisibility,
    /// Drunk: a quarter of your health back, at once.
    Healing,
}

impl Utility {
    pub const ALL: [Utility; 5] =
        [Utility::Fireball, Utility::Builder, Utility::Speed, Utility::Invisibility, Utility::Healing];

    /// Whether it is aimed, by holding its button and dragging, rather
    /// than pressed.
    pub const fn aimed(self) -> bool {
        matches!(self, Utility::Fireball | Utility::Builder)
    }

    /// Whether it takes [`bridge_wars::DRINK_SECS`] to drink before it
    /// does anything: every potion (the healing one too since Hajun's
    /// file of 2026-11-12, "after drinking time 1.6s, speed reduct, same
    /// as speed potion").
    pub const fn drunk(self) -> bool {
        !self.aimed()
    }

    /// Whether what it does lasts a while once it is drunk, with the time
    /// left written over the button.
    pub const fn lasts(self) -> bool {
        matches!(self, Utility::Speed | Utility::Invisibility)
    }
}

/// What someone in a game of Bridge Wars has, besides what is in their
/// [`Purse`]: the sword in their hand and the shield round them, by tier
/// (0 is none); what is on their second button and how many of it; until
/// when, in whole seconds into the fight, they run fast and go unseen (0
/// is not at all); and whether a trap has blinded them.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Kit {
    pub sword: u8,
    pub shield: u8,
    pub utility: Option<Utility>,
    pub count: u8,
    pub fast_until: u16,
    pub unseen_until: u16,
    pub blind: bool,
}

impl Kit {
    /// Whether they are running fast, `elapsed` seconds into the fight.
    pub fn fast(&self, elapsed: f32) -> bool {
        f32::from(self.fast_until) > elapsed
    }

    /// Whether they are unseen then.
    pub fn unseen(&self, elapsed: f32) -> bool {
        f32::from(self.unseen_until) > elapsed
    }
}

/// What a team has bought at the upgrade shop: how far its generator has
/// been raised (0 to [`bridge_wars::FORGE_MOST`]), whether its swords are
/// stronger, how far its shields have been bettered (0 to
/// [`bridge_wars::BETTER_SHIELDS_MOST`]), whether it has a heal pool, and
/// how many blindness traps wait on its island.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Upgrades {
    pub forge: u8,
    pub swords: bool,
    pub shields: u8,
    pub heal_pool: bool,
    pub traps: u8,
}

/// Bridge Wars' rules, as the game plays them on the device (`arena`) and
/// the server does online (`server/src/arena.rs`): in one place, so that
/// the two never differ. The device's `assets/ui.ron` has the same numbers
/// for playing offline, where Hajun tunes them by hand; online the server's
/// rule.
pub mod bridge_wars {
    use super::{
        AMETHYST_PADS, ARENA_ISLAND_RADIUS, ARENA_ISLANDS, ARENA_PADS, ARENA_SEATS, ARENA_SHOPS, ARENA_TEAMS,
        ARENA_UPGRADE_SHOPS, Item, Kit, PAD_HALF, PAD_TOP, Purse, Quat, SAPPHIRE_ISLAND_RADIUS, SAPPHIRE_PADS, TAU,
        Upgrades, Utility, Vec2, Vec3, Ware, arena_team,
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
    /// pad it makes it on. Past the map's own sixteen are the four a
    /// team's own generator becomes once it is raised to make amethyst
    /// too ([`FORGE_AMETHYST`]): on the team's pad, by team.
    pub fn generator(index: usize) -> (Item, Vec2) {
        match index {
            0..=3 => (Item::Silver, ARENA_PADS[index]),
            4..=7 => (Item::Gold, ARENA_PADS[index - 4]),
            8..=11 => (Item::Sapphire, SAPPHIRE_PADS[index - 8]),
            12..=15 => (Item::Amethyst, AMETHYST_PADS[index - 12]),
            _ => (Item::Amethyst, ARENA_PADS[(index - GENERATORS) % ARENA_TEAMS]),
        }
    }

    /// How many times as fast a team's own generator makes its silver and
    /// its gold, by how far the team has raised it at the upgrade shop
    /// (Hajun's file: "Better Generator ... (faster base generator 150%)
    /// -> Advanced Generator ... (even faster base generator 200%) ->
    /// Amethyst Generator ... (starts spawning amethyst too) -> Molten
    /// Generator ... (fastest 300%)"; that the third step keeps the
    /// second's pace is mine).
    pub const FORGE_SPEED: [f32; 5] = [1.0, 1.5, 2.0, 2.0, 3.0];
    /// From which step a team's generator makes amethyst too, how long each
    /// takes, in seconds, and how many lie on the pad at the most: as slow
    /// as the map's own amethyst generators begin, and as few.
    pub const FORGE_AMETHYST: u8 = 3;
    pub const FORGE_AMETHYST_SECS: f32 = 60.0;
    pub const FORGE_AMETHYST_CAP: usize = 2;

    /// How long a generator of `item` at `tier` takes to make one, in
    /// seconds, and how much more or less than that each one may take
    /// (Hajun's file: "Silver: baserate = 1 per ~2 sec (+- 0.5 sec variable
    /// interval)", "Gold: baserate = 1 per ~7.5sec (+-1 sec variable
    /// interval)", "Sapphire: base rate = 1 per 30 s (Tier I) ... Tier2 =
    /// 1per23s, Tier3 = 1per 12s", "Amethyst: base rate = 1 per 60 s (Tier
    /// I) ... Tier2 = 1 per 40s, Tier3 = 1 per 25s": 40, 30 and 20 until
    /// 2026-11-12).
    pub fn interval(item: Item, tier: u8) -> (f32, f32) {
        match (item, tier) {
            (Item::Silver, _) => (2.0, 0.5),
            (Item::Gold, _) => (7.5, 1.0),
            (Item::Sapphire, 0 | 1) => (30.0, 0.0),
            (Item::Sapphire, 2) => (23.0, 0.0),
            (Item::Sapphire, _) => (12.0, 0.0),
            (Item::Amethyst, 0 | 1) => (60.0, 0.0),
            (Item::Amethyst, 2) => (40.0, 0.0),
            (Item::Amethyst, _) => (25.0, 0.0),
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
    /// How far over someone's feet a thing may be and still be picked up
    /// by them, in metres: half a metre over the top of their head, which
    /// is 1.64 m up, as Minecraft picks a thing up half a block over a
    /// player's; and how far under their feet: a jump's height over what
    /// lies on a pad, so that someone jumping across a pad takes what
    /// lies on it, and neither the waiting room nor a fall past the island
    /// does. A thing is where it is as it drops ([`Piece::height`]), so
    /// one still coming down is out of reach until it is that near, and
    /// someone who jumps for it has it sooner, in the air (Hajun,
    /// 2026-11-12: "the delay is a bit too much for the delay between the
    /// item dropping from generator and the user collecting - user should
    /// be able to collect in the air too maybe when they jump"). For an
    /// evening before that a thing could not be picked up until it had
    /// dropped and lain 0.3 s, 0.85 s after it was made, however near it
    /// came; standing on its pad it is yours 0.45 s after it is made now,
    /// and at the top of a jump 0.32 s after.
    pub const REACH_OVER: f32 = 2.14;
    pub const REACH_UNDER: f32 = 1.5;
    /// How far over its pad's top a thing's own middle hangs once it has
    /// dropped, in metres (the device bobs it about there, `loot`): the
    /// models reach 0.18 m under their middles at the most, so the lowest
    /// of them clears the pad by a hand.
    pub const HOVER: f32 = 0.42;
    /// How far over where it will hang a thing appears as it is made, to
    /// drop from there, in metres, and how long the drop takes, in
    /// seconds (Hajun, 2026-11-11: "i want it to be a drop from a certain
    /// height animation. so that even when player is standing on top of
    /// the generator, u can see the mineral for a bit before the player
    /// pick it up"): from over the head of anyone standing on the pad.
    /// Both numbers are mine. The device draws the drop (`loot`), and
    /// whoever runs the generators goes by it for who can reach the thing
    /// ([`Mint::pick_up`]).
    pub const DROP_FROM: f32 = 5.0;
    pub const DROP_SECS: f32 = 0.55;

    /// How far over where it hangs a thing is, `age` seconds after it was
    /// made, in metres: dropping from [`DROP_FROM`] for [`DROP_SECS`], as
    /// a thing let go falls, slowly and then fast, and there from then on.
    pub fn drop_height(age: f32) -> f32 {
        if age >= DROP_SECS {
            return 0.0;
        }
        let fallen = age.max(0.0) / DROP_SECS;
        DROP_FROM * (1.0 - fallen * fallen)
    }

    /// One thing a generator has made, dropping onto its pad or lying on
    /// it until someone picks it up: a number of its own, which generator
    /// made it, what it is, where it lies across the ground, and how far
    /// into the fight it was made, in seconds.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Piece {
        pub id: u32,
        pub generator: u8,
        pub item: Item,
        pub at: Vec2,
        pub born: f32,
    }

    impl Piece {
        /// How high its middle is, `elapsed` seconds into the fight, in
        /// the map's own heights: over its pad's top where it hangs, and
        /// as far over that as it has still to drop.
        pub fn height(&self, elapsed: f32) -> f32 {
            PAD_TOP + HOVER + drop_height(elapsed - self.born)
        }
    }

    /// Whether a generator of `item` makes its things to the second, with
    /// nothing more or less to each wait ([`interval`]): the sapphire and
    /// the amethyst ones, whose next thing every device can count down to
    /// by the time alone ([`next_at`]).
    pub fn steady(item: Item) -> bool {
        interval(item, 1).1 == 0.0
    }

    /// When a [`steady`] generator of `item` next comes to make one, later
    /// than `after`, both in seconds into the fight: a whole wait on from
    /// the last time it did, counted from the start of the fight, and
    /// sooner where a tier raised meanwhile has a wait of its own that
    /// ends sooner, counted from when it was raised (so the sapphire
    /// ones, every 30 s to 120, then every 23 s to 350, then at 372, the
    /// third tier having come at 360, and every 12 s from there). It
    /// comes round whether or not the generator is full, in which case
    /// nothing is made and the count starts over: so the count is the
    /// time's alone, and the same on every device, which is how each
    /// writes "Next Amethyst in 24sec..." over a generator with nothing
    /// sent for it (the time is the round's, to the second).
    pub fn next_at(item: Item, after: f32) -> f32 {
        let mut at = 0.0;
        loop {
            let next = following(item, at);
            if next > after {
                return next;
            }
            at = next;
        }
    }

    /// The time a [`steady`] generator of `item` comes to make one after
    /// the time at `at`.
    fn following(item: Item, at: f32) -> f32 {
        let mut next = at + interval(item, tier(item, at)).0;
        for &(raised_at, happening) in &TIMETABLE {
            if let Happening::Tier { item: raised, tier } = happening
                && raised == item
                && raised_at > at
                && raised_at < next
            {
                next = next.min(raised_at + interval(item, tier).0);
            }
        }
        next
    }

    /// Every generator of a game and everything they have made that still
    /// lies there: the server's online, and the device's own in a game
    /// played offline, run by the same rules either way. It rolls no dice
    /// of its own: whoever runs it hands it some, numbers from 0 up to 1.
    #[derive(Clone, Debug)]
    pub struct Mint {
        /// When each generator next comes to make one, in seconds into
        /// the fight. It comes round whether or not the generator is
        /// full ([`cap`]): full, nothing is made, and the count starts
        /// over.
        due: [f32; GENERATORS],
        /// When each team's own generator next makes an amethyst, once it
        /// has been raised to ([`FORGE_AMETHYST`]): none until then.
        forge_due: [Option<f32>; ARENA_TEAMS],
        pieces: Vec<Piece>,
        next: u32,
    }

    impl Mint {
        /// Every generator as a fight begins, `elapsed` seconds into its
        /// timetable (none, but for trying the timetable out): nothing
        /// made yet, and the first of each a whole wait away, or, for the
        /// [`steady`] ones of a fight begun late, when the time says.
        pub fn new(elapsed: f32, dice: &mut dyn FnMut() -> f32) -> Self {
            let mut due = [0.0; GENERATORS];
            for (index, due) in due.iter_mut().enumerate() {
                let item = generator(index).0;
                *due = if steady(item) {
                    next_at(item, elapsed)
                } else {
                    elapsed + wait_for(item, tier(item, elapsed), dice)
                };
            }
            Self {
                due,
                forge_due: [None; ARENA_TEAMS],
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

        /// Runs every generator up to `elapsed` seconds into the fight
        /// (which says what tier each is at): what they made. A
        /// generator whose time has come makes one, unless it holds its
        /// [`cap`], and either way begins its next wait: full, it makes
        /// nothing until something has been taken and its time comes
        /// round again (until late on 2026-11-11 its count stood still while it
        /// was full, and the next thing was a whole wait on from when one
        /// was taken; the count is written over the sapphire and the
        /// amethyst ones now, and one that stood still at nothing would
        /// say so for as long as nobody came).
        ///
        /// `forge` is how far each team has raised its own generator
        /// ([`Upgrades::forge`]): its silver and its gold come that many
        /// times as fast ([`FORGE_SPEED`]), from its next wait on, and
        /// from [`FORGE_AMETHYST`] it makes amethyst on its pad too.
        pub fn run(&mut self, elapsed: f32, forge: &[u8; ARENA_TEAMS], dice: &mut dyn FnMut() -> f32) -> Vec<Piece> {
            let mut made = Vec::new();
            for index in 0..GENERATORS {
                let (item, pad) = generator(index);
                // One at the most a go: a frame is far shorter than a wait.
                if elapsed < self.due[index] {
                    continue;
                }
                self.due[index] = if steady(item) {
                    next_at(item, self.due[index])
                } else {
                    // A team's own, as fast as the team has made it; and
                    // never a wait that has already passed, should a go
                    // come late.
                    let speed = FORGE_SPEED[usize::from(forge[index % ARENA_TEAMS]).min(FORGE_SPEED.len() - 1)];
                    (self.due[index] + wait_for(item, tier(item, elapsed), dice) / speed).max(elapsed)
                };
                if self.held(index) >= cap(item) {
                    continue;
                }
                made.push(self.make(index, item, pad, elapsed, dice));
            }
            for team in 0..ARENA_TEAMS {
                if forge[team] < FORGE_AMETHYST {
                    continue;
                }
                // Its first a whole wait on from when it was raised.
                let due = *self.forge_due[team].get_or_insert(elapsed + FORGE_AMETHYST_SECS);
                if elapsed < due {
                    continue;
                }
                self.forge_due[team] = Some(due + FORGE_AMETHYST_SECS);
                let index = GENERATORS + team;
                if self.held(index) < FORGE_AMETHYST_CAP {
                    let (item, pad) = generator(index);
                    made.push(self.make(index, item, pad, elapsed, dice));
                }
            }
            made
        }

        /// One more thing from the generator numbered `index`, on its pad.
        fn make(&mut self, index: usize, item: Item, pad: Vec2, elapsed: f32, dice: &mut dyn FnMut() -> f32) -> Piece {
            let piece = Piece {
                id: self.next,
                generator: index as u8,
                item,
                at: self.place(pad, dice),
                born: elapsed,
            };
            self.next = self.next.wrapping_add(1);
            self.pieces.push(piece);
            piece
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

        /// Whoever's feet are at `at`, `elapsed` seconds into the fight,
        /// picks up everything within reach of them: within [`MAGNET`]
        /// across the ground, and, where the thing is now, dropping or
        /// lying ([`Piece::height`]), no further over their feet than
        /// [`REACH_OVER`] nor under them than [`REACH_UNDER`]. What they
        /// took, no longer there for anyone else.
        pub fn pick_up(&mut self, at: Vec3, elapsed: f32) -> Vec<Piece> {
            let (across, reach) = (Vec2::new(at.x, at.z), MAGNET * MAGNET);
            let (taken, left): (Vec<Piece>, Vec<Piece>) = self.pieces.iter().copied().partition(|piece| {
                let over = piece.height(elapsed) - at.y;
                piece.at.distance_squared(across) <= reach && (-REACH_UNDER..=REACH_OVER).contains(&over)
            });
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

    /// Which of an island's two shops: the item shop at its red stand, or
    /// the upgrade shop at its blue one.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Shop {
        Items,
        Upgrades,
    }

    /// Which shop sells `ware`.
    pub fn sold_at(ware: Ware) -> Shop {
        match ware {
            Ware::Bridge | Ware::Sword(_) | Ware::Shield(_) | Ware::Utility(_) => Shop::Items,
            Ware::Forge | Ware::StrongerSwords | Ware::BetterShields | Ware::HealPool | Ware::Trap => Shop::Upgrades,
        }
    }

    /// How much more a punch takes with each sword in hand, by its tier
    /// (none, basic, silver, aura: Hajun's file, "+5 dmg", "+10 dmg", "+20
    /// dmg"), and what share of every blow each shield keeps off ("6% dmg
    /// reduction", 14% and 22%).
    pub const SWORD_DAMAGE: [f32; 4] = [0.0, 5.0, 10.0, 20.0];
    pub const SHIELD_SHARE: [f32; 4] = [0.0, 0.06, 0.14, 0.22];
    /// What the upgrade shop's Stronger Swords adds to every sword of the
    /// team ("permanent +5 dmg on all swords"), and each step of its
    /// Better Shields to every shield ("permanent +6% shield dmg
    /// reduction"), of which there are four. Both go by having a sword or
    /// a shield: bare hands hit no harder, and nothing is kept off someone
    /// with no shield (mine: the file says "on all swords" and "shield
    /// dmg reduction").
    pub const STRONGER_SWORDS: f32 = 5.0;
    pub const BETTER_SHIELDS: f32 = 0.06;
    pub const BETTER_SHIELDS_MOST: u8 = 4;
    /// How far a team's generator can be raised, and how many traps can
    /// wait on its island ("1st 2nd, 3rd queued").
    pub const FORGE_MOST: u8 = 4;
    pub const TRAPS_MOST: u8 = 3;

    /// How many of `utility` the second button holds: three fireballs
    /// (Hajun's file: "can buy multiple - cap at 3 stack"), and nine of
    /// anything else (mine: one figure in the button's corner).
    pub const fn most_held(utility: Utility) -> u8 {
        match utility {
            Utility::Fireball => 3,
            _ => 9,
        }
    }

    /// What `ware` costs someone with `kit` on a team with `team`: so many
    /// of an item, from Hajun's file. Nothing if it is not to be had: a
    /// sword or a shield no better than the one they have ("you cannot buy
    /// a same/lower tier sword/shield, only greater"); something for the
    /// second button while it holds another thing ("you can only buy if
    /// the 2nd ui button is either empty or its filled by the same item u
    /// already have"), or as many as it holds; or an upgrade the team has
    /// to its last step.
    pub fn price(ware: Ware, kit: &Kit, team: &Upgrades) -> Option<(Item, u16)> {
        let step = |prices: &[u16], taken: u8| prices.get(usize::from(taken)).map(|cost| (Item::Sapphire, *cost));
        match ware {
            Ware::Bridge => Some((Item::Silver, 4)),
            Ware::Sword(1) if kit.sword < 1 => Some((Item::Silver, 10)),
            Ware::Sword(2) if kit.sword < 2 => Some((Item::Gold, 7)),
            Ware::Sword(3) if kit.sword < 3 => Some((Item::Amethyst, 4)),
            Ware::Shield(1) if kit.shield < 1 => Some((Item::Silver, 24)),
            Ware::Shield(2) if kit.shield < 2 => Some((Item::Gold, 12)),
            Ware::Shield(3) if kit.shield < 3 => Some((Item::Amethyst, 6)),
            Ware::Utility(utility)
                if kit.utility.is_none_or(|held| held == utility) && kit.count < most_held(utility) =>
            {
                Some(match utility {
                    Utility::Fireball => (Item::Silver, 40),
                    Utility::Builder => (Item::Amethyst, 1),
                    Utility::Speed => (Item::Amethyst, 1),
                    Utility::Invisibility => (Item::Amethyst, 2),
                    Utility::Healing => (Item::Gold, 3),
                })
            }
            Ware::Forge => step(&[2, 4, 6, 8], team.forge),
            Ware::StrongerSwords if !team.swords => Some((Item::Sapphire, 4)),
            Ware::BetterShields => step(&[2, 3, 4, 6], team.shields),
            Ware::HealPool if !team.heal_pool => Some((Item::Sapphire, 1)),
            Ware::Trap => step(&[1, 2, 4], team.traps),
            _ => None,
        }
    }

    /// Sells `ware` to someone with `purse` and `kit` on a team with
    /// `team`, if it is to be had and they hold its [`price`], which comes
    /// out of the purse: whether it is theirs. A bridge is nothing in the
    /// kit: whoever sells it gives the metres ([`BRIDGE_A_BUY`]).
    pub fn buy(ware: Ware, purse: &mut Purse, kit: &mut Kit, team: &mut Upgrades) -> bool {
        let Some((item, cost)) = price(ware, kit, team) else {
            return false;
        };
        let held = &mut purse[item.index()];
        if *held < cost {
            return false;
        }
        *held -= cost;
        match ware {
            Ware::Bridge => {}
            Ware::Sword(tier) => kit.sword = tier,
            Ware::Shield(tier) => kit.shield = tier,
            Ware::Utility(utility) => {
                kit.utility = Some(utility);
                kit.count += 1;
            }
            Ware::Forge => team.forge += 1,
            Ware::StrongerSwords => team.swords = true,
            Ware::BetterShields => team.shields += 1,
            Ware::HealPool => team.heal_pool = true,
            Ware::Trap => team.traps += 1,
        }
        true
    }

    /// How many metres of bridge one [`Ware::Bridge`] is: 25 since
    /// late on 2026-11-11, 50 for the evening before.
    pub const BRIDGE_A_BUY: f32 = 25.0;

    /// What a punch takes, from `base` (a bare hand's, [`DAMAGE`]): more
    /// with a sword, and more again if the team's swords are stronger.
    pub fn punch_damage(base: f32, kit: &Kit, team: &Upgrades) -> f32 {
        if kit.sword == 0 {
            return base;
        }
        base + SWORD_DAMAGE[usize::from(kit.sword.min(3))] + if team.swords { STRONGER_SWORDS } else { 0.0 }
    }

    /// What share of a blow the shield of someone with `kit` keeps off: its
    /// own, and the team's Better Shields on top.
    pub fn kept_off(kit: &Kit, team: &Upgrades) -> f32 {
        if kit.shield == 0 {
            return 0.0;
        }
        SHIELD_SHARE[usize::from(kit.shield.min(3))] + BETTER_SHIELDS * f32::from(team.shields.min(BETTER_SHIELDS_MOST))
    }

    /// How long a potion takes to drink, in seconds, and how much of their
    /// pace someone keeps while they drink ("shows 1.6s timer running to 0
    /// in front of ur character while ur movement is slowed 50% while
    /// drinking"); how long the speed potion lasts and how much faster it
    /// makes them ("45 second 40% speed boost"); how long the invisibility
    /// potion lasts ("30second complete invisibility ... being hit or
    /// attacking reveals u", [`Outfits::reveal`]); and what share of
    /// their health the healing potion gives back once it is drunk
    /// ("instantly heal 25% of your health").
    pub const DRINK_SECS: f32 = 1.6;
    pub const DRINK_SLOW: f32 = 0.5;
    pub const FAST_SECS: f32 = 45.0;
    pub const FAST: f32 = 1.4;
    pub const UNSEEN_SECS: f32 = 30.0;
    pub const HEALING: f32 = 0.25;
    /// What a team's heal pool gives each of the team on its own island,
    /// and how often, in seconds ("heals 5hp every 2.5sec").
    pub const HEAL_POOL: f32 = 5.0;
    pub const HEAL_POOL_EVERY: f32 = 2.5;
    /// How long a trap blinds whoever springs it, in seconds ("for fixed
    /// 8 seconds"; until Hajun's file of 2026-11-12 it was until they had
    /// been off that island five); and how far round themselves they see
    /// meanwhile, in metres ("a very small circle around their character":
    /// the number is mine).
    pub const BLIND_SECS: f32 = 8.0;
    pub const BLIND_SIGHT: f32 = 2.6;
    /// How far a builder lays its bridge, in metres ("creating a bridge in
    /// a straight line for 100m or until it reaches a island/land"), and
    /// how fast it walks, as anyone building does, half a run.
    pub const BUILDER_METRES: f32 = 100.0;
    pub const BUILDER_SPEED: f32 = 5.0;

    /// A fireball: what its blast takes off everyone it reaches ("20
    /// dmg"); how fast it flies and how far, in metres a second and
    /// metres; how big the ball is; how high over the thrower's feet it
    /// flies and how far in front of them it sets out; and how far its
    /// blast reaches from where it bursts, across the ground and up or
    /// down. All but the damage are mine: a range that ends about where
    /// the screen does under the camera over you, which shows 11 m up and
    /// down from you, so that whoever throws it sees it burst (20 m, for
    /// its first hours, burst off the screen); a ball that can be seen
    /// coming; a blast three bodies wide.
    pub const FIREBALL_DAMAGE: f32 = 20.0;
    pub const FIREBALL_SPEED: f32 = 22.0;
    pub const FIREBALL_RANGE: f32 = 12.0;
    pub const FIREBALL_RADIUS: f32 = 0.4;
    pub const FIREBALL_HEIGHT: f32 = 1.0;
    pub const FIREBALL_AHEAD: f32 = 0.9;
    pub const BLAST_RADIUS: f32 = 3.0;
    pub const BLAST_HEIGHT: f32 = 3.0;
    /// How wide a body is to a fireball, in metres from its middle: the
    /// game's bodies', at the shoulders.
    const BODY: f32 = 0.45;

    /// A fireball on its way: where it set out from, which way it flies, as
    /// a `Plank` heads (radians from the game's -Z, about the upright), and
    /// how far it has flown. The server flies the ones thrown online and
    /// the device its own offline, by the same rules: straight and level,
    /// through whatever stands on the map, the server having no map, until
    /// it [`Flight::meets`] someone or has flown its range, where it
    /// bursts.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Flight {
        pub from: Vec3,
        pub heading: f32,
        pub flown: f32,
    }

    impl Flight {
        /// One thrown by someone whose feet are at `feet`, that way.
        pub fn thrown(feet: Vec3, heading: f32) -> Self {
            let mut flight = Self {
                from: feet + Vec3::Y * FIREBALL_HEIGHT,
                heading,
                flown: 0.0,
            };
            flight.from += flight.way() * FIREBALL_AHEAD;
            flight
        }

        /// Which way it flies, across the ground.
        pub fn way(&self) -> Vec3 {
            Vec3::new(-self.heading.sin(), 0.0, -self.heading.cos())
        }

        /// Where it is.
        pub fn at(&self) -> Vec3 {
            self.from + self.way() * self.flown
        }

        /// Flies it on for `dt` seconds: whether it has come to the end of
        /// its range, where it stops.
        pub fn fly(&mut self, dt: f32) -> bool {
            self.flown = (self.flown + FIREBALL_SPEED * dt).min(FIREBALL_RANGE);
            self.flown >= FIREBALL_RANGE
        }

        /// Whether it has met a body whose feet are at `feet`.
        pub fn meets(&self, feet: Vec3) -> bool {
            let at = self.at();
            let across = Vec2::new(at.x - feet.x, at.z - feet.z).length();
            across <= FIREBALL_RADIUS + BODY && at.y >= feet.y - 0.2 && at.y <= feet.y + 2.0
        }
    }

    /// Whether a fireball bursting at `centre` reaches a body whose feet
    /// are at `feet`.
    pub fn in_blast(centre: Vec3, feet: Vec3) -> bool {
        let across = Vec2::new(centre.x - feet.x, centre.z - feet.z).length();
        across <= BLAST_RADIUS + BODY && (centre.y - (feet.y + 1.0)).abs() <= BLAST_HEIGHT
    }

    /// What has happened in a go of [`Outfits::tick`]: who a heal pool has
    /// healed, by seat, and whose trap has blinded whom, by seat and the
    /// team whose island it was.
    #[derive(Clone, PartialEq, Debug, Default)]
    pub struct Ticked {
        pub healed: Vec<usize>,
        pub sprung: Vec<(usize, u8)>,
    }

    /// What everyone in a game has and what every team has bought, with
    /// the rules that change them: the server's online, and the device's
    /// own in a game played offline, as the [`Mint`] is.
    #[derive(Clone, PartialEq, Debug, Default)]
    pub struct Outfits {
        pub kits: Vec<Kit>,
        pub teams: [Upgrades; ARENA_TEAMS],
        /// How much longer each seat is blind, in seconds, and whose
        /// island it last sprang a trap on and has not stepped off since.
        blind_left: Vec<f32>,
        trapped_on: Vec<Option<u8>>,
        /// How long each team's heal pool has been filling.
        pool: [f32; ARENA_TEAMS],
    }

    impl Outfits {
        /// Room for everyone up to `seats`, with nothing.
        pub fn seat(&mut self, seats: usize) {
            if self.kits.len() < seats {
                self.kits.resize(seats, Kit::default());
                self.blind_left.resize(seats, 0.0);
                self.trapped_on.resize(seats, None);
            }
        }

        /// What the one in `seat` has: nothing, for a seat nobody is in.
        pub fn kit(&self, seat: usize) -> Kit {
            self.kits.get(seat).copied().unwrap_or_default()
        }

        /// What the team of the one in `seat` has bought: nothing, for a
        /// zombie, which is on nobody's team.
        pub fn team(&self, seat: usize) -> Upgrades {
            self.teams.get(usize::from(arena_team(seat))).copied().unwrap_or_default()
        }

        /// Sells `ware` to the one in `seat`, out of `purse` ([`buy`]).
        pub fn buy(&mut self, seat: usize, ware: Ware, purse: &mut Purse) -> bool {
            self.seat(seat + 1);
            let Some(team) = self.teams.get_mut(usize::from(arena_team(seat))) else {
                return false;
            };
            buy(ware, purse, &mut self.kits[seat], team)
        }

        /// The one in `seat` uses one `utility` off their second button,
        /// if that is what is on it: whether they had one. The button is
        /// empty once none is left and nothing drunk from it still lasts
        /// ([`Outfits::tick`]).
        pub fn spend(&mut self, seat: usize, utility: Utility) -> bool {
            let Some(kit) = self.kits.get_mut(seat) else {
                return false;
            };
            if kit.utility != Some(utility) || kit.count == 0 {
                return false;
            }
            kit.count -= 1;
            true
        }

        /// The one in `seat` has drunk the potion on their second button,
        /// `elapsed` seconds into the fight: which it was, if they had
        /// one. The speed and the invisibility last from now; the healing
        /// is the caller's to give ([`HEALING`]). Whoever asks has taken
        /// [`DRINK_SECS`] over it already: the drinking is the drinker's
        /// own device's.
        pub fn drink(&mut self, seat: usize, elapsed: f32) -> Option<Utility> {
            let potion = self.kits.get(seat)?.utility?;
            if potion.aimed() || !self.spend(seat, potion) {
                return None;
            }
            let kit = &mut self.kits[seat];
            match potion {
                Utility::Speed => kit.fast_until = (elapsed + FAST_SECS).ceil() as u16,
                Utility::Invisibility => kit.unseen_until = (elapsed + UNSEEN_SECS).ceil() as u16,
                _ => {}
            }
            Some(potion)
        }

        /// The one in `seat` has died: their sword, what was on their
        /// second button and whatever they had drunk are gone ("all items
        /// are lost upon death"), and they see again. Their shield stays
        /// ("sword is lost on death, shield carries on"), as what their
        /// team has bought does.
        pub fn die(&mut self, seat: usize) {
            let Some(kit) = self.kits.get_mut(seat) else {
                return;
            };
            *kit = Kit {
                shield: kit.shield,
                ..Kit::default()
            };
            self.blind_left[seat] = 0.0;
            self.trapped_on[seat] = None;
        }

        /// The one in `seat` has hit someone or something, or been hit:
        /// if they were unseen, they are seen again ("being hit or
        /// attacking reveals u"). Whether they were.
        pub fn reveal(&mut self, seat: usize) -> bool {
            match self.kits.get_mut(seat) {
                Some(kit) if kit.unseen_until != 0 => {
                    kit.unseen_until = 0;
                    true
                }
                _ => false,
            }
        }

        /// What a punch by the one in `hitter` takes off the one in
        /// `victim`, from a bare hand's `base`: more with a sword, less
        /// through a shield.
        pub fn punch(&self, base: f32, hitter: usize, victim: usize) -> f32 {
            self.through(victim, punch_damage(base, &self.kit(hitter), &self.team(hitter)))
        }

        /// What gets through to the one in `victim` of a blow of `damage`.
        pub fn through(&self, victim: usize, damage: f32) -> f32 {
            damage * (1.0 - kept_off(&self.kit(victim), &self.team(victim)))
        }

        /// Moves the potions, the traps and the heal pools on by `dt`
        /// seconds, `elapsed` seconds into the fight, with every living
        /// player where they stand, by seat (the computer's too; never a
        /// zombie): a potion that has run out is over, and the button
        /// empty with it once none is left; an enemy stepping onto the
        /// island of a team with a trap waiting is blinded for
        /// [`BLIND_SECS`], the trap spent, and springs no other there
        /// until they have stepped off it and on again (mine: the file
        /// says only that a trap is used up by one enemy); and every
        /// [`HEAL_POOL_EVERY`] a team's pool heals whoever of the team
        /// stands on its island.
        pub fn tick(&mut self, dt: f32, elapsed: f32, standing: &[(usize, Vec2)]) -> Ticked {
            let mut ticked = Ticked::default();
            for kit in &mut self.kits {
                if kit.fast_until != 0 && !kit.fast(elapsed) {
                    kit.fast_until = 0;
                }
                if kit.unseen_until != 0 && !kit.unseen(elapsed) {
                    kit.unseen_until = 0;
                }
                if kit.utility.is_some() && kit.count == 0 && kit.fast_until == 0 && kit.unseen_until == 0 {
                    kit.utility = None;
                }
            }
            for (kit, left) in self.kits.iter_mut().zip(&mut self.blind_left) {
                if *left > 0.0 {
                    *left -= dt;
                    if *left <= 0.0 {
                        *left = 0.0;
                        kit.blind = false;
                    }
                }
            }
            let island_of = |at: Vec2| (0..ARENA_TEAMS).find(|&team| ARENA_ISLANDS[team].distance(at) <= ARENA_ISLAND_RADIUS);
            for &(seat, at) in standing {
                if seat >= ARENA_SEATS || seat >= self.kits.len() {
                    continue;
                }
                let on = island_of(at);
                if self.trapped_on[seat].is_some_and(|island| on != Some(usize::from(island))) {
                    self.trapped_on[seat] = None;
                }
                if let Some(island) = on
                    && island != usize::from(arena_team(seat))
                    && self.trapped_on[seat].is_none()
                    && self.teams[island].traps > 0
                {
                    self.teams[island].traps -= 1;
                    self.trapped_on[seat] = Some(island as u8);
                    self.blind_left[seat] = BLIND_SECS;
                    self.kits[seat].blind = true;
                    ticked.sprung.push((seat, island as u8));
                }
            }
            for team in 0..ARENA_TEAMS {
                if !self.teams[team].heal_pool {
                    continue;
                }
                self.pool[team] += dt;
                if self.pool[team] < HEAL_POOL_EVERY {
                    continue;
                }
                self.pool[team] -= HEAL_POOL_EVERY;
                ticked.healed.extend(
                    standing
                        .iter()
                        .filter(|(seat, at)| {
                            *seat < ARENA_SEATS && usize::from(arena_team(*seat)) == team && island_of(*at) == Some(team)
                        })
                        .map(|(seat, _)| *seat),
                );
            }
            ticked
        }
    }

    /// How far from a shop ([`ARENA_SHOPS`], [`ARENA_UPGRADE_SHOPS`])
    /// someone may be, where they last said they stood, for the server to
    /// sell to them, in metres: the outline in front of the stand reaches
    /// 1.5 m from its middle, and the rest is for the way they may have
    /// gone since.
    pub const SHOP_SLACK: f32 = 5.0;

    /// Whether someone standing at `at` across the ground is near enough
    /// to a `shop` to buy from it, as the server checks it. An island's
    /// two shops are 10 m apart: at either, someone is not at the other.
    pub fn at_a_shop(at: Vec2, shop: Shop) -> bool {
        let shops = match shop {
            Shop::Items => &ARENA_SHOPS,
            Shop::Upgrades => &ARENA_UPGRADE_SHOPS,
        };
        shops.iter().any(|shop| shop.distance(at) < SHOP_SLACK)
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

/// What everyone in a game of Bridge Wars has, by seat ([`Kit`]): beside
/// the [`Purses`], and sent to the room whenever anyone's changes, so that
/// every device draws the sword in everyone's hand and the shield round
/// them, and hides whoever is unseen.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Kits(pub Vec<Kit>);

/// What each team of a game of Bridge Wars has bought at the upgrade
/// shop, by team: beside the [`Kits`].
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub struct Upgraded(pub [Upgrades; ARENA_TEAMS]);

/// A fireball in the air in a game of Bridge Wars: who threw it, by seat,
/// where it set out from, and which way it flies, as a [`Plank`] heads.
/// Sent once as it is thrown; every device flies it on from there at
/// [`bridge_wars::FIREBALL_SPEED`], and it is gone, for everyone, the
/// moment the server has it burst, where each device then has it.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[component(immutable)]
pub struct Fireball {
    pub seat: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub heading: f32,
}

/// Using what is on the second button in Bridge Wars ([`Ask::Use`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Use {
    /// Throw a fireball this way, as a [`Plank`] heads.
    Fireball { heading: f32 },
    /// A builder has been sent off: its bridge comes as [`Ask::Rung`]s.
    Builder,
    /// The potion on the button has been drunk.
    Potion,
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
    /// I have used what is on my second button in Bridge Wars: one fewer
    /// of it, and what it does is done (since 2026-11-12).
    Use(Use),
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
        use bridge_wars::{DROP_SECS, GENERATORS, Mint, PAD_SPREAD, WAITING_FLOOR, cap, generator};
        let mut dice = steady(0.5);
        let mut mint = Mint::new(0.0, &mut dice);
        // A silver every 2 s and a gold every 7.5 s from each team's pad,
        // a sapphire every 30 s and an amethyst every 40 s: after 29 s,
        // 14 silver and 3 gold a pad, and neither of the others yet.
        let (dt, mut elapsed) = (1.0 / 60.0, 0.0);
        let mut made = Vec::new();
        while elapsed < 29.0 {
            elapsed += dt;
            made.extend(mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice));
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
            mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice);
        }
        for index in 0..GENERATORS {
            let (item, _) = generator(index);
            assert_eq!(mint.held(index), cap(item), "generator {index}, {item:?}");
        }
        assert_eq!(mint.pieces().len(), 4 * (48 + 6 + 4 + 2));
        // Full, a generator makes nothing however long it is left; and
        // emptied, its next thing comes as its time next comes round,
        // within a wait, and not all at once.
        while elapsed < 140.0 {
            elapsed += dt;
            assert!(mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice).is_empty(), "something made on a full pad, {elapsed} s in");
        }
        let pad = ARENA_PADS[0];
        let taken = mint.pick_up(on(pad), elapsed);
        assert_eq!(taken.len(), 48 + 6, "everything on a team's pad is within reach of its middle");
        let mut waited = 0.0;
        let next = loop {
            waited += dt;
            let made = mint.run(elapsed + waited, &[0; ARENA_TEAMS], &mut dice);
            assert!(made.len() <= 1, "{} things at once on an emptied pad", made.len());
            if let Some(piece) = made.first() {
                break *piece;
            }
            assert!(waited < 2.1, "nothing after {waited} s");
        };
        assert_eq!(next.item, Item::Silver);
        // And that one, dropping from high over the pad, is nobody's
        // while it is out of reach over their head. It is theirs as it
        // comes down to them, before it has landed, and sooner for
        // someone at the top of a jump, who has it in the air.
        elapsed += waited;
        let standing = on(pad);
        let jumping = standing + Vec3::Y * 1.6;
        let had = |by: Vec3, after: f32| mint.clone().pick_up(by, elapsed + after);
        assert!(had(standing, 0.0).is_empty() && had(jumping, 0.0).is_empty());
        assert!(had(standing, 0.25).is_empty() && had(jumping, 0.25).is_empty());
        assert!(had(standing, 0.35).is_empty());
        assert_eq!(had(jumping, 0.35), vec![next]);
        assert!(had(standing, 0.42).is_empty());
        assert_eq!(had(standing, 0.47), vec![next]);
        assert!(0.47 < DROP_SECS, "it is still dropping then");
        // Landed, it is to be had at once, from the pad and from a jump
        // over it; and not from far under the island, nor from the
        // waiting room over the map.
        let landed = DROP_SECS + 0.01;
        assert_eq!(had(standing, landed), vec![next]);
        assert_eq!(had(jumping, landed), vec![next]);
        assert!(had(standing - Vec3::Y * 8.0, landed).is_empty());
        assert!(had(Vec3::new(pad.x, WAITING_FLOOR, pad.y), landed).is_empty());
    }

    /// Standing on the pad at `pad`: where someone's feet are there.
    fn on(pad: Vec2) -> Vec3 {
        Vec3::new(pad.x, PAD_TOP, pad.y)
    }

    #[test]
    fn a_sapphire_and_an_amethyst_generator_s_next_thing_is_the_time_s_alone() {
        use bridge_wars::{GENERATORS, Mint, PLAY_SECS, generator, next_at, steady as to_the_second};
        assert!(to_the_second(Item::Sapphire) && to_the_second(Item::Amethyst));
        assert!(!to_the_second(Item::Silver) && !to_the_second(Item::Gold));
        // Every time one comes round, up to `until`.
        let times = |item: Item, until: f32| {
            let (mut at, mut times) = (0.0, Vec::new());
            loop {
                at = next_at(item, at);
                if at > until {
                    return times;
                }
                times.push(at);
            }
        };
        // A sapphire every 30 s to two minutes, every 23 s from then, and
        // from six minutes, when the wait in hand had 13 s to go, every
        // 12 s.
        let sapphire = times(Item::Sapphire, 400.0);
        assert_eq!(sapphire[..6], [30.0, 60.0, 90.0, 120.0, 143.0, 166.0]);
        assert_eq!(sapphire[sapphire.len() - 4..], [350.0, 372.0, 384.0, 396.0]);
        // An amethyst every 60 s to four minutes, every 40 s to eight,
        // and every 25 s after.
        let amethyst = times(Item::Amethyst, 530.0);
        assert_eq!(amethyst[..3], [60.0, 120.0, 180.0]);
        assert_eq!(amethyst[3..6], [240.0, 280.0, 320.0]);
        assert_eq!(amethyst[amethyst.len() - 4..], [440.0, 480.0, 505.0, 530.0]);
        // What is written over a generator: the next is always ahead,
        // never further than a wait, and at the moment one comes it is
        // the one after.
        assert_eq!(next_at(Item::Sapphire, 0.0), 30.0);
        assert_eq!(next_at(Item::Sapphire, 119.0), 120.0);
        assert_eq!(next_at(Item::Sapphire, 120.0), 143.0);
        assert_eq!(next_at(Item::Sapphire, 360.0), 372.0);
        assert_eq!(next_at(Item::Amethyst, 239.5), 240.0);
        assert_eq!(next_at(Item::Amethyst, 240.0), 280.0);
        assert_eq!(next_at(Item::Amethyst, 480.0), 505.0);
        for item in [Item::Sapphire, Item::Amethyst] {
            let mut at = 0.0;
            while at < PLAY_SECS {
                let next = next_at(item, at);
                assert!(next > at && next - at <= 60.0, "{item:?}: {next} after {at}");
                at += 0.37;
            }
        }
        // The generators keep those times, to the frame, taken from or
        // not: one emptied as each thing comes, and one left to fill.
        let mut dice = steady(0.5);
        let mut mint = Mint::new(0.0, &mut dice);
        let (dt, mut elapsed) = (1.0 / 30.0, 0.0);
        let mut made_at = Vec::new();
        let mut left_alone = 0;
        while elapsed < 400.0 {
            elapsed += dt;
            for piece in mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice) {
                if piece.generator == 8 {
                    made_at.push(elapsed);
                }
                left_alone += usize::from(piece.generator == 9);
            }
            mint.pick_up(on(SAPPHIRE_PADS[0]), elapsed);
        }
        assert_eq!(made_at.len(), sapphire.len(), "{made_at:?}");
        for (made, due) in made_at.iter().zip(&sapphire) {
            assert!(*made >= *due && made - due < 2.0 * dt, "made {made} s in, due at {due}");
        }
        assert_eq!(left_alone, 4, "a full generator makes nothing as its time comes round");
        // A fight begun part way through its timetable keeps them too.
        let mut late = Mint::new(595.0, &mut dice);
        let mut elapsed = 595.0;
        let mut first = [None; GENERATORS];
        while elapsed < 625.0 {
            elapsed += dt;
            for piece in late.run(elapsed, &[0; ARENA_TEAMS], &mut dice) {
                first[usize::from(piece.generator)].get_or_insert(elapsed);
            }
        }
        for (index, first) in first.iter().enumerate() {
            let (item, _) = generator(index);
            let first = first.unwrap_or_else(|| panic!("generator {index} made nothing in 30 s"));
            if to_the_second(item) {
                let due = next_at(item, 595.0);
                assert!(first >= due && first - due < 2.0 * dt, "{item:?}: first {first} s in, due at {due}");
            }
        }
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
        assert_eq!(interval(Item::Amethyst, 1).0, 60.0);
        assert_eq!(interval(Item::Amethyst, 2).0, 40.0);
        assert_eq!(interval(Item::Amethyst, 3).0, 25.0);
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
        let mut mint = Mint::new(0.0, &mut dice);
        let (dt, mut elapsed) = (1.0 / 30.0, 0.0);
        let mut taken = [0usize; 3];
        while elapsed < 600.0 {
            elapsed += dt;
            mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice);
            let got = mint.pick_up(on(SAPPHIRE_PADS[0]), elapsed);
            assert!(got.iter().all(|piece| piece.item == Item::Sapphire));
            // Each is to be had about half a second after it is made, as
            // it comes down.
            let stretch = if elapsed <= 121.5 {
                0
            } else if elapsed <= 361.5 {
                1
            } else {
                2
            };
            taken[stretch] += got.len();
        }
        assert_eq!(taken, [4, 10, 19]);
    }

    #[test]
    fn a_thing_is_picked_up_from_near_it_once_and_things_come_up_apart() {
        use bridge_wars::{MAGNET, Mint, generator};
        let mut dice = rolling();
        let mut mint = Mint::new(0.0, &mut dice);
        let (dt, mut elapsed) = (0.05, 0.0);
        while elapsed < 20.0 {
            elapsed += dt;
            mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice);
        }
        // A second on, the last of them has dropped.
        let elapsed = elapsed + 1.0;
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
        assert!(mint.pick_up(on(pad + Vec2::new(MAGNET + 1.0 + 0.5, 0.0)), elapsed).is_empty());
        assert_eq!(mint.pieces().len(), all);
        let taken = mint.pick_up(on(pad), elapsed);
        assert_eq!(taken.len(), lying.len());
        assert!(mint.pick_up(on(pad), elapsed).is_empty());
        assert_eq!(mint.pieces().len(), all - taken.len());
        // Another team's pad is untouched by it.
        assert!(!mint.pick_up(on(ARENA_PADS[0]), elapsed).is_empty());
    }

    #[test]
    fn the_shops_sell_what_the_file_says_for_what_it_says() {
        use bridge_wars::{Shop, buy, price, sold_at};
        let (bare, none) = (Kit::default(), Upgrades::default());
        // The item shop: the bridge, three swords, three shields and five
        // things for the second button.
        for (ware, cost) in [
            (Ware::Bridge, (Item::Silver, 4)),
            (Ware::Sword(1), (Item::Silver, 10)),
            (Ware::Sword(2), (Item::Gold, 7)),
            (Ware::Sword(3), (Item::Amethyst, 4)),
            (Ware::Shield(1), (Item::Silver, 24)),
            (Ware::Shield(2), (Item::Gold, 12)),
            (Ware::Shield(3), (Item::Amethyst, 6)),
            (Ware::Utility(Utility::Fireball), (Item::Silver, 40)),
            (Ware::Utility(Utility::Builder), (Item::Amethyst, 1)),
            (Ware::Utility(Utility::Speed), (Item::Amethyst, 1)),
            (Ware::Utility(Utility::Invisibility), (Item::Amethyst, 2)),
            (Ware::Utility(Utility::Healing), (Item::Gold, 3)),
        ] {
            assert_eq!(price(ware, &bare, &none), Some(cost), "{ware:?}");
            assert_eq!(sold_at(ware), Shop::Items);
        }
        assert_eq!(price(Ware::Sword(4), &bare, &none), None);
        assert_eq!(price(Ware::Shield(0), &bare, &none), None);
        // Only a better sword or shield than the one in hand; any tier
        // from none.
        let silver = Kit { sword: 2, shield: 1, ..bare };
        assert_eq!(price(Ware::Sword(1), &silver, &none), None);
        assert_eq!(price(Ware::Sword(2), &silver, &none), None);
        assert!(price(Ware::Sword(3), &silver, &none).is_some());
        assert_eq!(price(Ware::Shield(1), &silver, &none), None);
        assert!(price(Ware::Shield(2), &silver, &none).is_some() && price(Ware::Shield(3), &silver, &none).is_some());
        // The second button holds one kind of thing: three fireballs at
        // the most, and nine of anything else.
        let fire = Kit { utility: Some(Utility::Fireball), count: 2, ..bare };
        assert!(price(Ware::Utility(Utility::Fireball), &fire, &none).is_some());
        assert_eq!(price(Ware::Utility(Utility::Speed), &fire, &none), None);
        let full = Kit { count: 3, ..fire };
        assert_eq!(price(Ware::Utility(Utility::Fireball), &full, &none), None);
        let potions = Kit { utility: Some(Utility::Healing), count: 8, ..bare };
        assert!(price(Ware::Utility(Utility::Healing), &potions, &none).is_some());
        assert_eq!(price(Ware::Utility(Utility::Healing), &Kit { count: 9, ..potions }, &none), None);
        // What is aimed and what is drunk; and of what is drunk, what
        // lasts.
        for utility in Utility::ALL {
            assert_ne!(utility.aimed(), utility.drunk(), "{utility:?}");
            assert!(!utility.lasts() || utility.drunk());
        }
        assert!(Utility::Fireball.aimed() && Utility::Builder.aimed() && Utility::Healing.drunk());
        assert!(Utility::Speed.lasts() && Utility::Invisibility.lasts() && !Utility::Healing.lasts());
        // The upgrade shop: each step dearer than the last, to its last.
        let steps = |ware: Ware, raise: &dyn Fn(&mut Upgrades)| {
            let (mut team, mut costs) = (Upgrades::default(), Vec::new());
            while let Some((item, cost)) = price(ware, &bare, &team) {
                assert_eq!((item, sold_at(ware)), (Item::Sapphire, Shop::Upgrades));
                costs.push(cost);
                raise(&mut team);
            }
            costs
        };
        assert_eq!(steps(Ware::Forge, &|team| team.forge += 1), [2, 4, 6, 8]);
        assert_eq!(steps(Ware::BetterShields, &|team| team.shields += 1), [2, 3, 4, 6]);
        assert_eq!(steps(Ware::Trap, &|team| team.traps += 1), [1, 2, 4]);
        assert_eq!(steps(Ware::StrongerSwords, &|team| team.swords = true), [4]);
        assert_eq!(steps(Ware::HealPool, &|team| team.heal_pool = true), [1]);
        // Buying takes the price and gives the thing; without the price,
        // nothing changes.
        let (mut purse, mut kit, mut team): (Purse, Kit, Upgrades) = ([9, 7, 3, 0], bare, none);
        assert!(!buy(Ware::Sword(1), &mut purse, &mut kit, &mut team), "9 silver is not 10");
        assert_eq!((purse, kit), ([9, 7, 3, 0], bare));
        assert!(buy(Ware::Sword(2), &mut purse, &mut kit, &mut team));
        assert_eq!((purse, kit.sword), ([9, 0, 3, 0], 2));
        assert!(!buy(Ware::Sword(2), &mut purse, &mut kit, &mut team));
        assert!(buy(Ware::Forge, &mut purse, &mut kit, &mut team) && buy(Ware::Trap, &mut purse, &mut kit, &mut team));
        assert_eq!((purse, team.forge, team.traps), ([9, 0, 0, 0], 1, 1));
        purse[0] = 100;
        assert!(buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team));
        assert!(buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team));
        assert_eq!((purse[0], kit.utility, kit.count), (20, Some(Utility::Fireball), 2));
    }

    #[test]
    fn a_sword_hits_harder_and_a_shield_keeps_some_of_it_off() {
        use bridge_wars::{DAMAGE, Outfits, kept_off, punch_damage};
        let (bare, none) = (Kit::default(), Upgrades::default());
        // A bare hand's 12, and 5, 10 and 20 more by the sword.
        assert_eq!(punch_damage(DAMAGE, &bare, &none), 12.0);
        for (tier, more) in [(1, 5.0), (2, 10.0), (3, 20.0)] {
            assert_eq!(punch_damage(DAMAGE, &Kit { sword: tier, ..bare }, &none), 12.0 + more);
        }
        // Stronger Swords is five more on a sword, and nothing on a hand.
        let strong = Upgrades { swords: true, ..none };
        assert_eq!(punch_damage(DAMAGE, &Kit { sword: 3, ..bare }, &strong), 37.0);
        assert_eq!(punch_damage(DAMAGE, &bare, &strong), 12.0);
        // 6%, 14% and 22% kept off by the shield, and 6% more for each of
        // the team's four Better Shields; nothing with no shield.
        for (tier, share) in [(1, 0.06), (2, 0.14), (3, 0.22)] {
            assert!((kept_off(&Kit { shield: tier, ..bare }, &none) - share).abs() < 1e-6);
        }
        let better = Upgrades { shields: 4, ..none };
        assert!((kept_off(&Kit { shield: 3, ..bare }, &better) - 0.46).abs() < 1e-6);
        assert_eq!(kept_off(&bare, &better), 0.0);
        // In a game: seat 0, with the aura sword on a team with stronger
        // swords, hits seat 1, with the basic shield.
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        outfits.kits[0].sword = 3;
        outfits.kits[1].shield = 1;
        outfits.teams[0].swords = true;
        assert!((outfits.punch(DAMAGE, 0, 1) - 37.0 * 0.94).abs() < 1e-4);
        assert_eq!(outfits.punch(DAMAGE, 1, 0), 12.0);
        // A zombie, in a seat past the players', has nothing.
        assert_eq!(outfits.punch(DAMAGE, ARENA_SEATS, 2), 12.0);
        assert!((outfits.through(1, 20.0) - 18.8).abs() < 1e-4);
        // Dying loses the sword and what was on the second button, and
        // keeps the shield.
        outfits.kits[1] = Kit { sword: 2, shield: 1, utility: Some(Utility::Speed), count: 2, fast_until: 90, unseen_until: 0, blind: true };
        outfits.die(1);
        assert_eq!(outfits.kits[1], Kit { shield: 1, ..Kit::default() });
    }

    #[test]
    fn a_team_s_generator_is_as_fast_as_the_team_has_made_it() {
        use bridge_wars::{FORGE_AMETHYST, GENERATORS, Mint, generator};
        // Three minutes of every generator, team 0's raised to each step:
        // its silver half as fast again, twice, and three times as fast as
        // team 1's, which is left as it was; and from the third step
        // amethyst on its own pad too.
        let made = |forge: u8| {
            let mut dice = steady(0.5);
            let mut mint = Mint::new(0.0, &mut dice);
            let (dt, mut elapsed) = (1.0 / 30.0, 0.0);
            let mut counts = [0usize; 3];
            while elapsed < 180.0 {
                elapsed += dt;
                for piece in mint.run(elapsed, &[forge, 0, 0, 0], &mut dice) {
                    match usize::from(piece.generator) {
                        0 => counts[0] += 1,
                        1 => counts[1] += 1,
                        index if index == GENERATORS => {
                            assert_eq!((piece.item, generator(index).1), (Item::Amethyst, ARENA_PADS[0]));
                            counts[2] += 1;
                        }
                        _ => {}
                    }
                }
                // Everything taken as it comes, so that nothing fills.
                mint.pick_up(on(ARENA_PADS[0]), elapsed);
                mint.pick_up(on(ARENA_PADS[1]), elapsed);
            }
            counts
        };
        assert_eq!(made(0), [90, 90, 0]);
        let better = made(1);
        assert!((133..=136).contains(&better[0]) && better[1] == 90 && better[2] == 0, "{better:?}");
        let advanced = made(2);
        assert!((178..=180).contains(&advanced[0]) && advanced[2] == 0, "{advanced:?}");
        // An amethyst a minute from when it was raised: at 60, 120 and 180.
        let amethyst = made(FORGE_AMETHYST);
        assert!((178..=180).contains(&amethyst[0]) && (2..=3).contains(&amethyst[2]), "{amethyst:?}");
        let molten = made(4);
        assert!((266..=270).contains(&molten[0]) && (2..=3).contains(&molten[2]), "{molten:?}");
    }

    #[test]
    fn a_fireball_flies_straight_to_the_end_of_its_range_or_whoever_it_meets() {
        use bridge_wars::{BLAST_RADIUS, FIREBALL_AHEAD, FIREBALL_HEIGHT, FIREBALL_RANGE, FIREBALL_SPEED, Flight, in_blast};
        // Thrown north from the middle of an island: it sets out in front
        // of the thrower, at their chest, and flies north, level.
        let feet = Vec3::new(0.0, 0.25, 130.0);
        let mut ball = Flight::thrown(feet, 0.0);
        assert!(ball.at().distance(feet + Vec3::new(0.0, FIREBALL_HEIGHT, -FIREBALL_AHEAD)) < 1e-5);
        assert!(ball.way().distance(Vec3::NEG_Z) < 1e-6);
        // Heading a quarter turn to the left of north is west, as a plank
        // heads.
        assert!(Flight::thrown(feet, std::f32::consts::FRAC_PI_2).way().distance(Vec3::NEG_X) < 1e-6);
        // Someone 8 m north is met on the way; someone 8 m north and 2 m
        // to the side is not, nor someone far under it.
        let (ahead, beside) = (feet + Vec3::new(0.0, 0.0, -8.0), feet + Vec3::new(2.0, 0.0, -8.0));
        let (dt, mut met, mut seconds) = (1.0 / 60.0, None, 0.0);
        while !ball.fly(dt) {
            seconds += dt;
            assert!(!ball.meets(beside) && !ball.meets(ahead - Vec3::Y * 6.0));
            if met.is_none() && ball.meets(ahead) {
                met = Some(ball.flown);
            }
        }
        let met = met.expect("it passed through someone in its way");
        assert!(met > 6.0 && met < 7.5, "met {met} m out");
        // Left to fly, it stops at the end of its range, in the time that
        // takes.
        assert_eq!(ball.flown, FIREBALL_RANGE);
        assert!((seconds - FIREBALL_RANGE / FIREBALL_SPEED).abs() < 0.05);
        assert!(ball.at().distance(feet + Vec3::new(0.0, FIREBALL_HEIGHT, -FIREBALL_AHEAD - FIREBALL_RANGE)) < 1e-3);
        // Its blast reaches whoever is within three metres of it across
        // the ground, at about its height, and nobody further.
        let burst = ball.at();
        let under = burst - Vec3::Y * FIREBALL_HEIGHT;
        assert!(in_blast(burst, under) && in_blast(burst, under + Vec3::X * BLAST_RADIUS));
        assert!(!in_blast(burst, under + Vec3::X * (BLAST_RADIUS + 0.6)));
        assert!(!in_blast(burst, under - Vec3::Y * 8.0));
    }

    #[test]
    fn potions_wear_off_traps_blind_and_pools_heal() {
        use bridge_wars::{BLIND_SECS, FAST_SECS, HEAL_POOL_EVERY, Outfits, UNSEEN_SECS};
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        // A speed potion: nothing until it is drunk; drunk, 45 s of being
        // fast, the button kept for as long, and empty after.
        let mut purse: Purse = [0, 3, 0, 6];
        assert!(outfits.buy(0, Ware::Utility(Utility::Speed), &mut purse));
        assert!(!outfits.spend(0, Utility::Fireball) && !outfits.kits[0].fast(10.0));
        assert_eq!(outfits.drink(0, 10.0), Some(Utility::Speed));
        assert_eq!(outfits.drink(0, 10.0), None, "there was only one");
        assert!(outfits.kits[0].fast(10.0) && outfits.kits[0].fast(10.0 + FAST_SECS - 0.5));
        assert!(!outfits.buy(0, Ware::Utility(Utility::Healing), &mut purse), "the button still holds the speed");
        outfits.tick(0.1, 30.0, &[]);
        assert_eq!(outfits.kits[0].utility, Some(Utility::Speed));
        outfits.tick(0.1, 10.0 + FAST_SECS + 0.1, &[]);
        assert_eq!(outfits.kits[0], Kit::default());
        // The invisibility lasts its 30 s, or until they hit or are hit;
        // the healing is at once, and the button empty the moment the
        // last is gone.
        assert!(outfits.buy(0, Ware::Utility(Utility::Invisibility), &mut purse));
        assert_eq!(outfits.drink(0, 100.0), Some(Utility::Invisibility));
        assert!(outfits.kits[0].unseen(100.0 + UNSEEN_SECS - 0.5) && !outfits.kits[0].unseen(100.0 + UNSEEN_SECS));
        outfits.tick(0.1, 200.0, &[]);
        assert_eq!(outfits.kits[0], Kit::default());
        assert!(outfits.buy(0, Ware::Utility(Utility::Invisibility), &mut purse));
        assert_eq!(outfits.drink(0, 150.0), Some(Utility::Invisibility));
        assert!(outfits.kits[0].unseen(151.0) && !outfits.reveal(1), "seat 1 was in plain sight");
        assert!(outfits.reveal(0) && !outfits.kits[0].unseen(151.0) && !outfits.reveal(0));
        outfits.tick(0.1, 151.0, &[]);
        assert_eq!(outfits.kits[0], Kit::default(), "and the button is empty again");
        assert!(outfits.buy(0, Ware::Utility(Utility::Healing), &mut purse));
        assert_eq!(outfits.drink(0, 200.0), Some(Utility::Healing));
        outfits.tick(0.1, 200.1, &[]);
        assert_eq!(outfits.kits[0].utility, None);
        // A fireball is not drunk.
        outfits.kits[0] = Kit { utility: Some(Utility::Fireball), count: 1, ..Kit::default() };
        assert_eq!(outfits.drink(0, 200.0), None);
        assert!(outfits.spend(0, Utility::Fireball) && !outfits.spend(0, Utility::Fireball));

        // Team 1 has two traps waiting and a heal pool. Seat 0, of team 0,
        // steps onto its island: blinded, one trap spent; its own seat 1
        // there is not, nor a zombie; and seat 2, coming after, takes the
        // second.
        outfits.teams[1] = Upgrades { traps: 2, heal_pool: true, ..Upgrades::default() };
        let (theirs, mine) = (ARENA_ISLANDS[1], ARENA_ISLANDS[0]);
        let ticked = outfits.tick(0.1, 300.0, &[(0, theirs), (1, theirs + Vec2::X), (ARENA_SEATS, theirs)]);
        assert_eq!(ticked.sprung, vec![(0, 1)]);
        assert!(outfits.kits[0].blind && !outfits.kits[1].blind && outfits.teams[1].traps == 1);
        let ticked = outfits.tick(0.1, 300.1, &[(0, theirs), (2, theirs)]);
        assert_eq!(ticked.sprung, vec![(2, 1)]);
        assert_eq!(outfits.teams[1].traps, 0);
        // With none left, seat 3 walks on unharmed.
        assert!(outfits.tick(0.1, 300.2, &[(3, theirs)]).sprung.is_empty() && !outfits.kits[3].blind);
        // Blind for eight seconds, wherever they go: seat 0 for 7.8 more
        // off the island, and then no longer.
        let off = theirs + Vec2::new(40.0, 0.0);
        for _ in 0..77 {
            outfits.tick(0.1, 301.0, &[(0, off)]);
        }
        assert!(outfits.kits[0].blind, "7.9 seconds in");
        for _ in 0..2 {
            outfits.tick(0.1, 308.0, &[(0, off)]);
        }
        assert!(!outfits.kits[0].blind, "{BLIND_SECS} seconds in");
        // Dying, seat 2 sees again at once.
        outfits.die(2);
        assert!(!outfits.kits[2].blind);
        // A trap springs on someone stepping onto the island, not on
        // someone who has stood on it since the last one: seat 0 stays on
        // team 1's island through a trap bought under their feet, and
        // springs it only once they have stepped off and on again.
        outfits.teams[1].traps = 1;
        outfits.tick(0.1, 320.0, &[(0, theirs)]);
        assert!(outfits.kits[0].blind && outfits.teams[1].traps == 0);
        for _ in 0..90 {
            outfits.tick(0.1, 321.0, &[(0, theirs)]);
        }
        assert!(!outfits.kits[0].blind);
        outfits.teams[1].traps = 1;
        assert!(outfits.tick(0.1, 330.0, &[(0, theirs)]).sprung.is_empty(), "still standing on it");
        outfits.tick(0.1, 330.1, &[(0, off)]);
        assert_eq!(outfits.tick(0.1, 330.2, &[(0, theirs)]).sprung, vec![(0, 1)]);
        assert!(outfits.kits[0].blind && outfits.teams[1].traps == 0);
        // The pool heals whoever of team 1 is on its island every two and
        // a half seconds: seat 1 there, not seat 5 away from it, nor seat
        // 0 of another team standing on it.
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        outfits.teams[1].heal_pool = true;
        let mut healed = Vec::new();
        for _ in 0..((HEAL_POOL_EVERY * 2.0 / 0.05) as usize + 2) {
            healed.extend(outfits.tick(0.05, 0.0, &[(1, theirs), (5, mine), (0, theirs)]).healed);
        }
        assert_eq!(healed, vec![1, 1]);
    }

    #[test]
    fn the_pads_the_shops_and_the_zombies_are_where_the_map_has_room_for_them() {
        use bridge_wars::{
            BRIDGE_A_BUY, SHOP_SLACK, Shop, TOWN_RADIUS, ZOMBIE_TEAM, ZOMBIES, at_a_shop, price, zombie_home,
            zombie_spot,
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
            assert!(at_a_shop(shop, Shop::Items) && at_a_shop(shop + Vec2::new(SHOP_SLACK - 0.1, 0.0), Shop::Items));
            assert!(!at_a_shop(pad, Shop::Items) && !at_a_shop(ARENA_ANCHORS[team], Shop::Items), "team {team}");
            // Its upgrade shop is across the island's middle from it, as
            // far out, and at either nobody is at the other.
            let upgrades = ARENA_UPGRADE_SHOPS[team];
            assert!((upgrades.distance(island) - 5.0).abs() < 0.1 && (upgrades.distance(shop) - 10.0).abs() < 0.1);
            assert!(at_a_shop(upgrades, Shop::Upgrades) && !at_a_shop(upgrades, Shop::Items));
            assert!(!at_a_shop(shop, Shop::Upgrades) && !at_a_shop(pad, Shop::Upgrades));
            // Its amethyst pad is the one toward it, in the middle.
            assert!(AMETHYST_PADS[team].normalize().dot(island.normalize()) > 0.99);
            assert!(AMETHYST_PADS[team].length() < 40.0);
        }
        for pad in SAPPHIRE_PADS {
            assert!((pad.length() - 125.0).abs() < 0.1, "{pad}");
            assert!(ARENA_ISLANDS.iter().all(|island| island.distance(pad) > ARENA_ISLAND_RADIUS + SAPPHIRE_ISLAND_RADIUS));
        }
        assert_eq!(price(Ware::Bridge, &Kit::default(), &Upgrades::default()), Some((Item::Silver, 4)));
        assert_eq!(BRIDGE_A_BUY, 25.0);
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
