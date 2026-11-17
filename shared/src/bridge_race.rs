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

use std::f32::consts::TAU;

use bevy::prelude::{App, Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::bridge_wars::WAIT_RING;
use crate::table;

/// How far up the map is stood as the device loads it, in metres (the
/// game's `island::lift`): Hajun's `bridgeracemap.glb` has its land at 0
/// where Bridge Wars' `pvptownmap.glb` has it at 0.25, and every rule
/// written for Bridge Wars' heights (a plank's top, a pad's, how far a body
/// falls before it is dead) goes by 0.25. Lifted, its islands' tops are at
/// 0.25, its pads' at 0.5, and the waiting room's floor at
/// [`WAITING_FLOOR`].
pub const LIFT: f32 = 0.25;

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

/// Something asked in a game of Bridge Race ([`Ask::BridgeRace`](crate::Ask::BridgeRace)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ask {
    /// A placeholder until the game's own asks come.
    Nothing,
}

/// Something told one device in a game of Bridge Race
/// ([`Tell::BridgeRace`](crate::Tell::BridgeRace)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// A placeholder until the game's own tells come.
    Nothing,
}

/// Registers this game's own replicated components (none yet).
pub fn register(_app: &mut App) {}

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
