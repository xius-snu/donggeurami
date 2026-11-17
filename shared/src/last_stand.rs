//! Last Stand, wave after wave round an anchor (since 2026-11-18; Hajun:
//! "Coop (2~4 players) Last Stand (Endless Zombie survival game). You are in
//! a circular map, with a spawn anchor crystal in the center, and 2
//! generators and item shops/upgrade shops. Zombies with increasing
//! difficulty/skill/creativity/health/attackdmg come in waves from the outer
//! edges toward the anchor crystal in the middle and/or player ... Endless
//! upgrades, endless waves of endlessly stronger and stronger enemies. You
//! get a numerical score at the end (individual scores of each person +
//! total combined team score). Map = laststandmap"): one to four people
//! together and never the computer, holding the anchor for as many waves as
//! they can, on `laststandmap.glb`
//! ([`Venue::LastStand`](crate::Venue::LastStand)). Its rules are in this
//! module, and the server and the device both play by them, as by Bridge
//! Wars' in [`crate::bridge_wars`]; how its table fills, starts and pays is
//! [`crate::table`]'s.
//!
//! **The whole game is a plain [`Game`]**, with no world, no network and no
//! map file: the server steps one online, a thin wrapper round it, and the
//! device one offline, the same game either way. Each feeds it where the
//! people stand ([`Game::sync`]) and what they do ([`Game::punch`],
//! [`Game::buy`], [`Game::use_item`], [`Game::fell`]...), steps it
//! ([`Game::step`]) and carries out what it says happened ([`Event`]);
//! everything else is the game's own: the zombies, the anchor, the
//! generators, everyone's health and purse.
//!
//! * **The map** ([`ground`], [`OBSTACLES`]): a stone disc 50 m across in a
//!   ring of sand 75 m across, over the void, with the anchor's crystal on
//!   its plinth in the middle, the two generators' pads either side of it,
//!   and the item shop's red stand and the upgrade shop's blue one in front
//!   of it and behind it. Lifted a quarter of a metre as the device loads it
//!   ([`LIFT`]), as every height here is.
//! * **The anchor** ([`ANCHOR_HEALTH`]): while it stands, whoever dies is
//!   back beside it [`RESPAWN_SECS`] later; broken, the dead stay dead, and
//!   **the game is over when the anchor is broken and nobody is standing**.
//! * **Waves** ([`count`], [`Kind`]): the first [`FIRST_WAVE`] seconds after
//!   "Go!", then [`BREAK`] between waves to shop; each bigger and harder,
//!   climbing up over the rim from a few places round it, walkers first and
//!   then runners, brutes, giants every fifth wave, spitters and leapers.
//!   Each goes for the anchor or for someone, turns on whoever hits it, and
//!   some come round the side.
//! * **Generators** ([`Forges`]): silver on both pads from the start, then
//!   gold, sapphire and amethyst as the team raises them, and faster, for
//!   ever; more of everything for more people. And what a zombie drops when
//!   it is killed ([`drop_of`]), better things as the waves go on, for
//!   whoever walks into it.
//! * **Two shops** ([`Ware`]): the item shop's swords, shields and things to
//!   use are yours and kept when you die; the upgrade shop's are the whole
//!   team's and have no end but two.
//! * **The score** ([`kill_score`]): each kill, and every ten damage; the
//!   team's is everyone's and [`WAVE_SCORE`] for each wave cleared. What the
//!   table pays is the waves cleared ([`prizes`]).
//!
//! Every number here not quoted from Hajun is ours ("the details i didn't
//! mention are yours to decide to make each game polished and fun"), tuned
//! by whole games played by stand-in players in this module's tests
//! (`cargo test -p roundtown_net stand_many -- --ignored --nocapture`).

use std::f32::consts::{PI, TAU};

use bevy::prelude::{App, Component, Quat, Vec2, Vec3};
use bevy_replicon::prelude::AppRuleExt;
use serde::{Deserialize, Serialize};

use crate::bridge_wars::WAIT_RING;
use crate::table;

mod forges;
mod game;
mod map;
mod shop;
mod waves;
mod zombies;

#[cfg(test)]
mod tests;

pub use forges::{CAPS, Forges, INTERVALS, OVERCLOCK, SPREADS};
pub use game::{
    ANCHOR_HEALTH, BEAM_EVERY, BEAM_REACH, CREDIT_SECS, DYNAMITE_HIT, Event, FIREBALL_HIT, GRACE_SECS, Game, HEALTH,
    HURT_TIME, Knock, LEAP_SECS, PUNCH_SLACK, READY_SECS, REGEN_AFTER, REGEN_PER_SEC, RESPAWN_SECS, SPEED_SECS,
    UNSEEN_SECS, WAVE_SCORE,
};
pub use map::{
    CRYSTAL_RADIUS, ITEM_SHOP, OBSTACLES, Obstacle, PAD_HALF, PAD_TOP, PADS, PLINTH_HALF, PLINTH_TOP, RIM, SAND_RADIUS,
    SAND_TOP, SHOP_SLACK, STONE_RADIUS, STONE_TOP, UPGRADE_SHOP, WALK_EDGE, at_a_stand, disagreements, ground, keep_out,
};
pub use shop::{
    BEAM_DAMAGE, FORTIFY, LUCKY_MOST, PROTECTION, PROTECTION_MOST, SHARPNESS, SHIELD_SHARE, SWORD_DAMAGE, Ware, blow,
    buy, level, price, sharpened, sold_at, through,
};
pub use waves::{
    BREAK, DAMAGE_GROWS, FIRST_WAVE, HEALTH_GROWS, Kind, MOST_STANDING, PACE_GROWS, PACE_MOST, count, drop_chance,
    drop_of, first_in, giants, kill_score, mix, rising, strength, toughness, weights,
};
pub use zombies::{SPIT_RADIUS, Spittle};

/// How far up the map is stood as the device loads it, in metres (the
/// game's `island::lift`), as Bridge Race's is
/// ([`bridge_race::LIFT`](crate::bridge_race::LIFT)): Hajun's
/// `laststandmap.glb` has its stone at 0, where Bridge Wars' land is at
/// 0.25, which every rule written for Bridge Wars' heights goes by. Lifted,
/// its stone is at 0.25, its sand at 0.05, and its generators' pads' tops at
/// 0.5, as Bridge Wars' are ([`PAD_TOP`](crate::PAD_TOP)).
pub const LIFT: f32 = 0.25;

/// Where the anchor crystal stands, in metres from the middle of the map:
/// the middle of the pillar in the material `spawn` on its slab, 0.16 m off
/// the middle, as Hajun's map has it (a test in the game reads it off the
/// map). The pillar is Bridge Wars' anchors' own, as wide
/// ([`ANCHOR_RADIUS`](crate::ANCHOR_RADIUS)).
pub const ANCHOR: Vec2 = Vec2::new(0.0, -0.16);

/// The waiting room, Hajun's glass box high over the middle of the map, as
/// Bridge Wars' is over its: its middle across the ground, and its floor,
/// lifted with the rest of the map ([`LIFT`]; a test in the game reads it
/// off the map).
pub const WAITING_ROOM: Vec2 = Vec2::ZERO;
pub const WAITING_FLOOR: f32 = 150.5 + LIFT;

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

/// How a game of Last Stand fills, starts and pays: one to four people and
/// never the computer, beginning 15 s after the last person joined, or at
/// once with four (alone too, mine, so that nobody waits for a second
/// person who is not coming; Hajun said "2~4"); 50 coins to join.
pub const SHAPE: table::Shape = table::Shape {
    most: 4,
    fewest: 1,
    computer: false,
    wait: 15.0,
    countdown: 5.0,
    results: 20.0,
    fee: 50,
};

/// Something asked in a game of Last Stand ([`Ask::LastStand`](crate::Ask::LastStand)).
/// The rest of what a player does is asked as in Bridge Wars, and the
/// game's wrapper hands it to the [`Game`]: a punch that landed on a zombie
/// ([`Ask::Punch`](crate::Ask::Punch), the zombie's body's member),
/// something used off a button ([`Ask::Use`](crate::Ask::Use)), a fall into
/// the void ([`Ask::Respawn`](crate::Ask::Respawn)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ask {
    /// Sell me this, at the stand that sells it ([`Game::buy`]): its price
    /// comes out of what I hold, and [`Tell::Bought`] says it is mine; with
    /// too little, or nowhere near that stand, nothing happens.
    Buy(Ware),
    /// I am ready for the next wave ([`Game::ready`]): once everyone
    /// standing is, it comes in [`READY_SECS`] (mine: a break is for
    /// shopping, and whoever has done shopping need not stand about).
    Ready,
}

/// Something told one device in a game of Last Stand
/// ([`Tell::LastStand`](crate::Tell::LastStand)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// The shop has sold you this ([`Ask::Buy`]): its price is out of what
    /// you hold, as the room's [`Purses`](crate::Purses) will show, and
    /// what it was is in your kit ([`Kits`](crate::Kits)) or the team's
    /// ([`StandUpgrades`]), each as it comes.
    Bought(Ware),
    /// Someone of the team, by seat, has bought this at the upgrade shop for
    /// everyone, which the team has to `level` now: told to everyone, for a
    /// line in the chat ("Mochi bought Sharpness Lvl 3").
    Upgraded { by: u8, ware: Ware, level: u16 },
    /// The anchor's beam has zapped the zombie standing here
    /// ([`Event::Zap`]): told to everyone, for a line to be drawn from the
    /// crystal to it. What it took shows as the zombie's own health
    /// ([`Shambler`]) goes down.
    Zap { x: f32, y: f32, z: f32 },
}

/// Registers this game's own replicated components, appended after
/// everything else in this order: how the game stands ([`StandRound`]), the
/// team's upgrades ([`StandUpgrades`]), each zombie's kind and health on its
/// body ([`Shambler`]), and a spitter's ball in the air ([`Spit`]). What
/// everyone holds and has goes as Bridge Wars' does, in
/// [`Purses`](crate::Purses) and [`Kits`](crate::Kits) on the table's
/// entity, and what lies about as its [`Loot`](crate::Loot), each thing of
/// its own.
pub fn register(app: &mut App) {
    app.replicate::<StandRound>()
        .replicate::<StandUpgrades>()
        .replicate::<Shambler>()
        .replicate::<Spit>();
}

/// Where the player in `seat` stands as they arrive on the game's venue, and which way they
/// face: 3.2 m from the anchor, clear of its slab, on one of the diagonals, an eighth of the
/// turn round from +Z and a quarter on from one seat to the next, on the stone's top as
/// lifted ([`LIFT`]), facing out, toward where the zombies come from. Where the dead stand
/// again too, while the anchor stands.
pub fn arrival(seat: usize) -> (Vec3, Quat) {
    let angle = TAU / 8.0 + (seat % SHAPE.most) as f32 * TAU / 4.0;
    let out = Quat::from_rotation_y(angle) * Vec3::new(0.0, 0.0, 3.2);
    // Half a turn on from facing the anchor, which a turn of `angle` does.
    (Vec3::new(ANCHOR.x, LIFT, ANCHOR.y) + out, Quat::from_rotation_y(angle + PI))
}

/// What each seat is paid, from how the game came out (`results.places`, `scores`, `total`);
/// `people[seat]` is false for the computer's seats, which are paid nothing. Every person
/// alike, 10 coins for every wave the table cleared together (`total`), up to 400: five
/// waves give the fee back.
pub fn prizes(results: &table::Results, people: &[bool]) -> Vec<u32> {
    let each = results.total.unwrap_or(0).saturating_mul(10).min(400);
    (0..results.places.len())
        .map(|seat| if people.get(seat).copied().unwrap_or(false) { each } else { 0 })
        .collect()
}

/// The team's score at the end, as the results show it beside everyone's own
/// (Hajun: "individual scores of each person + total combined team score"):
/// everyone's added up, and [`WAVE_SCORE`] for each wave the team cleared.
pub fn team_score(scores: &[u32], waves: u32) -> u32 {
    scores.iter().fold(0u32, |sum, score| sum.saturating_add(*score)).saturating_add(waves.saturating_mul(WAVE_SCORE))
}

/// How a game of Last Stand stands, sent to its room on the table's entity
/// beside its [`Board`](table::Board) whenever any of it changes: which wave
/// it is, and whether it is coming or here; the anchor's health and the most
/// it has; how many of the wave are not yet dead; and everyone's health,
/// when the dead stand again and score, by seat. Some sixty bytes for four
/// ([`Game::round`]), sent a few times a second while health comes back and
/// the zombies are hit.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StandRound {
    /// The wave on now, or the last one, between waves: 0 before the first.
    pub wave: u16,
    pub stage: StandStage,
    /// The anchor's health, in whole points, and the most it has
    /// ([`ANCHOR_HEALTH`], and more for each Fortify): 0 is broken.
    pub anchor: u16,
    pub anchor_most: u16,
    /// How many of this wave are not yet dead: still to come over the rim,
    /// coming, or standing. 0 between waves.
    pub left: u16,
    /// Everyone's health by seat, in whole points of [`HEALTH`] (rounded up,
    /// so that anyone standing has some): 0 is dead.
    pub health: Vec<u16>,
    /// When each seat stands again, by the room's clock ([`Game::clock`]):
    /// 0 for anyone standing, and infinity for anyone out for good (dead
    /// with the anchor broken) or gone.
    pub back_at: Vec<f64>,
    /// Everyone's score by seat ([`kill_score`]).
    pub scores: Vec<u32>,
    /// Who is ready for the next wave ([`Ask::Ready`]), a bit to a seat:
    /// during a break only.
    pub ready: u8,
}

impl StandRound {
    /// Whether whoever is in `seat` is ready for the next wave.
    pub fn is_ready(&self, seat: usize) -> bool {
        seat < 8 && self.ready & (1 << seat) != 0
    }
}

/// How far a game of Last Stand has got.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum StandStage {
    /// Between waves, or before the first: the next comes at `until`, by the
    /// room's clock, which every device counts down to with nothing more
    /// sent ("Next wave in 0:18"). Sooner, once everyone is ready.
    Break { until: f64 },
    /// A wave is on.
    Wave,
    /// The anchor is broken and nobody is standing: the last stand fell.
    Over,
}

/// What the team of a game of Last Stand has bought at the upgrade shop,
/// for everyone: how far each line has been taken ([`Ware`]). On the
/// table's entity, sent as it changes. Endless but for two: the protection
/// ([`PROTECTION_MOST`]) and the luck ([`LUCKY_MOST`]).
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct StandUpgrades {
    /// The generators: gold from 1, sapphire from 2, amethyst from 3, and
    /// faster for every step past that ([`OVERCLOCK`]).
    pub generator: u16,
    /// How many times every blow is [`SHARPNESS`] harder.
    pub sharpness: u16,
    /// How many [`PROTECTION`]s off every hit, up to [`PROTECTION_MOST`].
    pub protection: u8,
    /// How many times [`FORTIFY`] has been added to the anchor.
    pub fortify: u16,
    /// The anchor's beam: [`BEAM_DAMAGE`] for each step.
    pub beam: u16,
    /// How lucky the team's kills are, up to [`LUCKY_MOST`]
    /// ([`drop_chance`], [`drop_of`]).
    pub lucky: u8,
}

/// On each zombie's body in a game of Last Stand: what kind it is (which the
/// device draws it as, by its size and tint), and its health and the most
/// it had, in whole points, for the bar over it; a hit shows as the health
/// going down. Sent as it rises and whenever its health changes.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shambler {
    pub kind: Kind,
    pub health: u16,
    pub most: u16,
}

/// A spitter's ball on its way in a game of Last Stand ([`Spittle`]): where
/// it left the spitter's mouth and where it comes down. Sent once as it is
/// spat; every device flies it on from there by the same rules, and it is
/// gone, for everyone, the moment it lands, where it bursts in a splash of
/// green ([`SPIT_RADIUS`]).
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[component(immutable)]
pub struct Spit {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub to_x: f32,
    pub to_y: f32,
    pub to_z: f32,
}

impl Spit {
    /// What the room is sent of a ball on its way.
    pub fn of(spittle: &Spittle) -> Self {
        Self {
            x: spittle.from.x,
            y: spittle.from.y,
            z: spittle.from.z,
            to_x: spittle.to.x,
            to_y: spittle.to.y,
            to_z: spittle.to.z,
        }
    }

    /// The ball it is, just spat: for a device to fly it on.
    pub fn spittle(&self) -> Spittle {
        Spittle::spat(Vec3::new(self.x, self.y, self.z), Vec3::new(self.to_x, self.to_y, self.to_z))
    }
}
