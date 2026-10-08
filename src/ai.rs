//! The computer's players, who sit in every seat of the lobby that no person
//! has taken.
//!
//! An AI is a player like any other. Its body is moved by the same
//! [`move_bodies`](crate::move_bodies) as yours, so it stands on the same
//! ground, wades the same sea and bumps into the same trees and the same
//! people. Only what does the asking differs: where you have a stick, an AI has
//! [`Steering`] — walk this far that way, then stop — and a mind that asks for
//! a walk somewhere at random every [`THINK_EVERY`] seconds, always on the
//! island it is on.
//!
//! Offline, this device runs every AI. With a server, the server will, and
//! each client will see them the way it sees any other player.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::island::{Island, Islands, Venue};
use crate::lobby::SEATS;
use crate::{Intent, LAND_STEP_UP, MOVE_SPEED, StaysAshore, dry_ground};

/// How often an AI picks somewhere new to walk, in seconds.
const THINK_EVERY: f32 = 3.0;
/// How far one of those walks goes, in metres. Even the longest is over well
/// before the next one comes, so an AI stops and stands between them.
const WALK_MIN: f32 = 2.0;
const WALK_MAX: f32 = 9.0;
/// How many random walks an AI looks at before it gives up and stands still
/// until it next thinks. Only one over dry land all the way is taken.
const TRIES: u32 = 8;
/// A walk must leave this much dry land beyond where it ends, in metres, so
/// that the AIs do not spend their time at the water's edge.
const SHORE_MARGIN: f32 = 1.5;
/// How finely a walk is checked for dry land, in metres: finely enough not to
/// step clean over the fountain's rim, 0.3 m across, and miss the drop into
/// its basin behind it.
const PROBE: f32 = 0.25;
/// This close to the end of a walk is there, in metres.
const ARRIVED: f32 = 0.01;

/// Walks a body by instruction, the way a stick walks yours.
#[derive(Component, Default)]
pub(crate) struct Steering {
    leg: Option<Leg>,
}

/// One walk under way.
struct Leg {
    /// Which way, flat on the ground.
    heading: Vec3,
    /// How far, in metres.
    distance: f32,
    /// Where it began: filled in on the first frame the body acts on it, and
    /// everything after measured from there.
    start: Option<Vec3>,
}

impl Steering {
    /// Walk `distance` metres toward `direction`, then stop, in place of
    /// whatever walk was under way.
    pub(crate) fn walk(&mut self, direction: Vec3, distance: f32) {
        let heading = direction.with_y(0.0).normalize_or_zero();
        self.leg = (heading != Vec3::ZERO && distance > 0.0).then_some(Leg {
            heading,
            distance,
            start: None,
        });
    }

    /// Stop where it stands.
    pub(crate) fn stop(&mut self) {
        self.leg = None;
    }
}

/// An AI's whole mind, for now: a walk somewhere random every
/// [`THINK_EVERY`] seconds.
#[derive(Component)]
struct Wander {
    /// Seconds until it next decides where to go.
    until_next: f32,
}

/// Everything that makes the body in `seat` an AI rather than a person. The
/// seats think at different moments, so the AIs do not all set off at once.
pub(crate) fn brain(seat: usize) -> impl Bundle {
    (
        Intent::default(),
        Steering::default(),
        Wander {
            until_next: THINK_EVERY * seat as f32 / SEATS as f32,
        },
        StaysAshore,
    )
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, (wander, steer).chain().before(crate::move_bodies));
}

fn wander(
    time: Res<Time>,
    islands: Res<Islands>,
    mut ais: Query<(&Transform, &Venue, &mut Wander, &mut Steering)>,
) {
    let dt = time.delta_secs();
    for (transform, &venue, mut wander, mut steering) in &mut ais {
        let Some(island) = islands.get(venue) else {
            continue;
        };
        wander.until_next -= dt;
        if wander.until_next > 0.0 {
            continue;
        }
        wander.until_next += THINK_EVERY;
        match pick_walk(island, transform.translation) {
            Some((heading, distance)) => steering.walk(heading, distance),
            // Nowhere it looked was dry. Stand still, and look again next time.
            None => steering.stop(),
        }
    }
}

/// A random walk from `from` that stays on dry land all the way, if any of the
/// few tried does.
fn pick_walk(island: &Island, from: Vec3) -> Option<(Vec3, f32)> {
    (0..TRIES).find_map(|_| {
        let angle = fastrand::f32() * TAU;
        let heading = Vec3::new(angle.cos(), 0.0, angle.sin());
        let distance = WALK_MIN + fastrand::f32() * (WALK_MAX - WALK_MIN);
        dry_all_the_way(island, from, heading, distance + SHORE_MARGIN)
            .then_some((heading, distance))
    })
}

/// Whether there is dry land under every step of `distance` metres from `from`
/// along `heading`, following the ground up and down the way a body walking it
/// would, and never down anything it could not step back up.
fn dry_all_the_way(island: &Island, from: Vec3, heading: Vec3, distance: f32) -> bool {
    let steps = (distance / PROBE).ceil() as u32;
    let mut y = from.y;
    for step in 1..=steps {
        let spot = from + heading * (step as f32 * PROBE).min(distance);
        let Some(ground) = dry_ground(island, spot.with_y(y)) else {
            return false;
        };
        if y - ground > LAND_STEP_UP {
            return false;
        }
        y = ground;
    }
    true
}

/// Turns each body's walk under way into this frame's [`Intent`]: at full
/// speed until the last stretch, which is taken slower so the body stops where
/// it was sent rather than a stride past it.
fn steer(time: Res<Time>, mut bodies: Query<(&Transform, &mut Steering, &mut Intent)>) {
    let dt = time.delta_secs();
    for (transform, mut steering, mut intent) in &mut bodies {
        *intent = Intent::default();
        let Some(leg) = &mut steering.leg else {
            continue;
        };
        let here = transform.translation;
        let start = *leg.start.get_or_insert(here);
        // Measured along the heading, so being pushed aside by a tree or
        // another player on the way does not count as getting anywhere.
        let left = leg.distance - (here - start).dot(leg.heading);
        if left <= ARRIVED {
            steering.leg = None;
            continue;
        }
        intent.walk = leg.heading * (left / (MOVE_SPEED * dt)).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_walk_never_goes_down_into_the_fountain() {
        let fountain = crate::ground_tests::fountain();
        // From the grass straight at the middle, or from the ledge over the
        // rim: either would end in the basin, so neither is taken.
        let (grass, ledge) = (Vec3::new(4.0, 0.0, 0.0), Vec3::new(2.35, 0.3, 0.0));
        assert!(!dry_all_the_way(&fountain, grass, Vec3::NEG_X, 3.0));
        assert!(!dry_all_the_way(&fountain, ledge, Vec3::NEG_X, 1.0));
        // Up from the ledge onto the rim and no further can be, and so can a
        // walk along the grass.
        assert!(dry_all_the_way(&fountain, ledge, Vec3::NEG_X, 0.3));
        assert!(dry_all_the_way(&fountain, grass, Vec3::X, 3.0));
    }
}
