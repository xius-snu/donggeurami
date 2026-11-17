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

use std::f32::consts::{PI, TAU};

use bevy::prelude::{App, Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::bridge_wars::WAIT_RING;
use crate::table;

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
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ask {
    /// A placeholder until the game's own asks come.
    Nothing,
}

/// Something told one device in a game of Last Stand
/// ([`Tell::LastStand`](crate::Tell::LastStand)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// A placeholder until the game's own tells come.
    Nothing,
}

/// Registers this game's own replicated components (none yet).
pub fn register(_app: &mut App) {}

/// Where the player in `seat` stands as they arrive on the game's venue, and which way they
/// face: 3.2 m from the anchor, clear of its slab, on one of the diagonals, an eighth of the
/// turn round from +Z and a quarter on from one seat to the next, on the stone's top as
/// lifted ([`LIFT`]), facing out, toward where the zombies come from.
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
