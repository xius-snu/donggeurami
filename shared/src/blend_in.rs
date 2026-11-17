//! Blend In, hide and seek in the town (since 2026-11-18; Hajun: "1 random
//! hider chosen as the person walking around pretending to be an NPC/AI. 1~3
//! other people trying to find who is the person, by punching with a aura
//! sword. This game min player count is 2, so if there 2 ppl, it will not
//! fill the 2 slot with 2 ai's but rather start game with just 2 real ppl.
//! There is a time limit, and incorrect guesses cost you time by
//! stunning/blinding etc. ... Map = roundtownmap ... You swap until every one
//! has tried hiding atleast once"): two to four people and never the
//! computer, each hiding once among a crowd of the town's people while the
//! others look for them, on the town's own map with nobody of the town's in
//! it ([`Venue::BlendIn`]). Its rules are in this module, and the server and
//! the device both play by them, as by Bridge Wars' in
//! [`crate::bridge_wars`]; how its table fills, starts and pays is
//! [`crate::table`]'s.

use bevy::prelude::{App, Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::{Venue, table};

/// How a game of Blend In fills, starts and pays: two to four people, the
/// computer never ("This game min player count is 2 ... start game with
/// just 2 real ppl"), beginning 15 s after the second has sat down, or at
/// once with four; 50 coins to join.
pub const SHAPE: table::Shape = table::Shape {
    most: 4,
    fewest: 2,
    computer: false,
    wait: 15.0,
    countdown: 5.0,
    results: 15.0,
    fee: 50,
};

/// Something asked in a game of Blend In ([`Ask::BlendIn`](crate::Ask::BlendIn)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ask {
    /// A placeholder until the game's own asks come.
    Nothing,
}

/// Something told one device in a game of Blend In
/// ([`Tell::BlendIn`](crate::Tell::BlendIn)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// A placeholder until the game's own tells come.
    Nothing,
}

/// Registers this game's own replicated components (none yet).
pub fn register(_app: &mut App) {}

/// Where the player in `seat` stands as they arrive on the game's venue, and which way they
/// face: where they would in the town, on the path round its pond
/// ([`TOWN_ARRIVAL_RADIUS`](crate::TOWN_ARRIVAL_RADIUS)), facing the fountain.
pub fn arrival(seat: usize) -> (Vec3, Quat) {
    crate::arrival(Venue::Town, seat)
}

/// What each seat is paid, from how the game came out (`results.places`, `scores`, `total`);
/// `people[seat]` is false for the computer's seats, which are paid nothing. By place, as
/// many places as sat down at the table (a seat each, every one a person's): of two 100 and
/// nothing, of three 110, 40 and nothing, of four 120, 70, 10 and nothing; those who tie
/// share ([`table::split`]), so that two who draw have their fee back.
pub fn prizes(results: &table::Results, people: &[bool]) -> Vec<u32> {
    let by_place: &[u32] = match results.places.len() {
        0..=2 => &[100, 0],
        3 => &[110, 40, 0],
        _ => &[120, 70, 10, 0],
    };
    table::split(by_place, &results.places)
        .into_iter()
        .enumerate()
        .map(|(seat, coins)| if people.get(seat).copied().unwrap_or(false) { coins } else { 0 })
        .collect()
}
