//! Knock Off, on an island of ice (since 2026-11-18; Hajun: "You spawn on a
//! tiny icy circular island with 2~4 other ppl (fill with ai like bridgewars
//! and its deathmatch so 1v1v1v1) ... Everyone is frozen in place, and before
//! every certain time interval like 10 second, you choose the direction and
//! intensity that you wanna slide. Then after the interval, everyone goes
//! toward that direction and intensity and collide, some ppl will be knocked
//! off. You keep going until last player surviving is the winner ... Map =
//! iceisland"): four seats, every one for themselves, the computer taking the
//! seats people leave, everyone aiming at once and then sliding at once, on
//! `iceisland.glb` ([`Venue::KnockOff`](crate::Venue::KnockOff)). Its rules
//! are in this module, and the server and the device both play by them, as
//! by Bridge Wars' in [`crate::bridge_wars`]; how its table fills, starts and
//! pays is [`crate::table`]'s.

use std::f32::consts::TAU;

use bevy::prelude::{App, Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::table;

/// How far the ice reaches from the middle of the island, in metres: Hajun's
/// `iceisland.glb` is a disc of ice 24 m across round the origin, its top at
/// 0 (a test in the game reads it off the map). A body whose middle is
/// further out than this has gone over.
pub const RADIUS: f32 = 12.0;

/// How far from the middle everyone stands as a game begins, in metres: on
/// the diagonals, facing the middle ([`arrival`]).
pub const START_RING: f32 = 7.0;

/// How a game of Knock Off fills, starts and pays: four seats, the computer
/// taking those people leave 10 s after the last person joined, as in
/// Bridge Wars ("fill with ai like bridgewars"); 50 coins to join.
pub const SHAPE: table::Shape = table::Shape {
    most: 4,
    fewest: 1,
    computer: true,
    wait: 10.0,
    countdown: 5.0,
    results: 15.0,
    fee: 50,
};

/// Something asked in a game of Knock Off ([`Ask::KnockOff`](crate::Ask::KnockOff)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ask {
    /// A placeholder until the game's own asks come.
    Nothing,
}

/// Something told one device in a game of Knock Off
/// ([`Tell::KnockOff`](crate::Tell::KnockOff)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// A placeholder until the game's own tells come.
    Nothing,
}

/// Registers this game's own replicated components (none yet).
pub fn register(_app: &mut App) {}

/// Where the player in `seat` stands as they arrive on the game's venue, and which way they
/// face: [`START_RING`] out from the middle on one of the diagonals, an eighth of the turn
/// round from +Z and a quarter on from one seat to the next, facing the middle.
pub fn arrival(seat: usize) -> (Vec3, Quat) {
    let angle = TAU / 8.0 + (seat % SHAPE.most) as f32 * TAU / 4.0;
    // A turn of `angle` puts +Z out where the seat stands, and points -Z
    // from there at the middle.
    let turn = Quat::from_rotation_y(angle);
    (turn * Vec3::new(0.0, 0.0, START_RING), turn)
}

/// What each seat is paid, from how the game came out (`results.places`, `scores`, `total`);
/// `people[seat]` is false for the computer's seats, which are paid nothing. By place, of
/// the four: 130, 60, 10 and nothing, those who went in in the same slide sharing
/// ([`table::split`]).
pub fn prizes(results: &table::Results, people: &[bool]) -> Vec<u32> {
    table::split(&[130, 60, 10, 0], &results.places)
        .into_iter()
        .enumerate()
        .map(|(seat, coins)| if people.get(seat).copied().unwrap_or(false) { coins } else { 0 })
        .collect()
}
