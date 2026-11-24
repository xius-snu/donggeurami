//! What stands on the town's plots and does something (since 2026-11-25;
//! Hajun: "make some of the assets "functional" for the ones u can for
//! example the treadmill could actually move the player backward when they
//! get on it like a real treadmill so that they need to keep walking forward
//! to stay on it").
//!
//! - **The treadmill's belt** ([`run_belts`]) carries whoever stands on it
//!   toward its open end, at [`BELT_SPEED`], less than a walk: walking toward
//!   its handles keeps you on it, as on a real one, and standing still puts
//!   you off its end. Every body that moves under its own power (yours, the
//!   computer's people), before it moves this frame; everyone else sees it
//!   by where your body is, as ever.
//! - **The ceiling fan** ([`turn_fans`]) turns, all of it about the rod it
//!   hangs from, which is over its origin: its model turned under the piece,
//!   which stays where it was put.
//!
//! The rest of what came that day is still: one mesh each, as it was
//! downloaded, with no part of it to move by itself (the desk fan's blades
//! are its stand too). What lights up is `lamps::BY_KIND`, what is sat on
//! `rest`.

use bevy::prelude::*;

use crate::build::{Ghost, Piece};
use crate::island::Venue;
use crate::rest::Resting;
use crate::{Dead, Intent};

/// How fast the belt goes, in metres a second: under a walk's 7, so that
/// walking on it gets you on toward its handles, slowly, and standing still
/// takes you off its end in half a second. Mine.
const BELT_SPEED: f32 = 5.0;
/// The belt, in the treadmill's own space (`mybuilds/newones/Treadmill.glb`,
/// set on its origin): between its side rails, from its open end at its
/// front (-Z) to its handles; and how far over its origin feet on it are, it
/// being 0.28 m up at its open end and 0.40 at its handles.
const BELT_HALF_WIDTH: f32 = 0.55;
const BELT_FROM: f32 = -1.52;
const BELT_TO: f32 = 1.06;
const BELT_LOW: f32 = 0.15;
const BELT_HIGH: f32 = 0.6;
/// How fast the ceiling fan turns, in radians a second: about once in two
/// seconds. Mine.
const FAN_TURN: f32 = 3.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            run_belts.after(crate::read_player_input).before(crate::move_bodies),
            turn_fans,
        ),
    );
}

/// Whether feet at `at`, in a treadmill's own space, are on its belt.
fn on_the_belt(at: Vec3) -> bool {
    at.x.abs() <= BELT_HALF_WIDTH && (BELT_FROM..=BELT_TO).contains(&at.z) && (BELT_LOW..=BELT_HIGH).contains(&at.y)
}

/// Carries everyone standing on a treadmill's belt toward its open end, as
/// far as the belt goes this frame: before they move under their own power
/// (`move_bodies`), which keeps them out of whatever they are carried into.
/// Only whoever is on a belt is written.
#[allow(clippy::type_complexity)]
fn run_belts(
    time: Res<Time>,
    treadmills: Query<(&Piece, &Transform, &Venue), (Without<Ghost>, Without<Intent>)>,
    mut bodies: Query<(&mut Transform, &Venue), (With<Intent>, Without<Piece>, Without<Resting>, Without<Dead>)>,
) {
    let step = BELT_SPEED * time.delta_secs();
    if step <= 0.0 {
        return;
    }
    let belts: Vec<(Transform, Venue)> = treadmills
        .iter()
        .filter(|(piece, ..)| piece.def.kind == "treadmill")
        .map(|(_, place, venue)| (*place, *venue))
        .collect();
    if belts.is_empty() {
        return;
    }
    for (mut body, venue) in &mut bodies {
        let carried = belts.iter().find(|(belt, on)| *on == *venue && on_the_belt(belt.compute_affine().inverse().transform_point3(body.translation)));
        if let Some((belt, _)) = carried {
            body.translation += belt.rotation * Vec3::NEG_Z * step;
        }
    }
}

/// The turn of a ceiling fan's model under its piece, as its scene made it.
#[derive(Component)]
struct Fan(Quat);

/// Turns every ceiling fan about the rod it hangs from: each part of its
/// scene directly under the piece, about the piece's upright.
#[allow(clippy::type_complexity)]
fn turn_fans(
    time: Res<Time>,
    fans: Query<(&Piece, &Children)>,
    mut parts: Query<(Entity, &mut Transform, Option<&Fan>), Without<Piece>>,
    mut commands: Commands,
) {
    let spin = Quat::from_rotation_y((time.elapsed_secs() * FAN_TURN) % std::f32::consts::TAU);
    for (piece, children) in &fans {
        if piece.def.kind != "ceiling_fan" {
            continue;
        }
        for &child in children {
            let Ok((part, mut place, fan)) = parts.get_mut(child) else {
                continue;
            };
            let rest = match fan {
                Some(&Fan(rest)) => rest,
                None => {
                    commands.entity(part).insert(Fan(place.rotation));
                    place.rotation
                }
            };
            place.rotation = spin * rest;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_belt_is_between_its_rails_from_its_open_end_to_its_handles() {
        // On it, anywhere along it, at its height.
        for at in [Vec3::new(0.0, 0.3, -1.4), Vec3::new(0.5, 0.4, 1.0), Vec3::new(-0.5, 0.35, 0.0)] {
            assert!(on_the_belt(at), "{at}");
        }
        // Beside it, past its ends, on the floor by it, or jumping over it.
        for at in [Vec3::new(0.7, 0.35, 0.0), Vec3::new(0.0, 0.3, -1.7), Vec3::new(0.0, 0.4, 1.2), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.2, 0.0)] {
            assert!(!on_the_belt(at), "{at}");
        }
        // Slower than a walk, so that walking on it gets one on.
        assert!(BELT_SPEED < 7.0 && BELT_SPEED > 3.0);
    }

    #[test]
    fn the_belt_carries_whoever_stands_on_it_toward_its_open_end_and_nobody_else() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        {
            let mut time = world.resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(100));
        }
        let treadmill = crate::build::PIECES.iter().find(|def| def.kind == "treadmill").unwrap();
        // A treadmill turned a quarter, its front toward the world's -X.
        let turned = Transform::from_xyz(10.0, 0.0, 5.0).with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
        world.spawn((Piece { def: treadmill }, turned, Venue::Town));
        let on = world.spawn((Transform::from_xyz(10.0, 0.35, 5.0), Venue::Town, Intent::default())).id();
        let off = world.spawn((Transform::from_xyz(12.0, 0.0, 5.0), Venue::Town, Intent::default())).id();
        world.run_system_once(run_belts).unwrap();
        // Half a metre in a tenth of a second, toward its open end.
        let moved = world.get::<Transform>(on).unwrap().translation;
        assert!(moved.distance(Vec3::new(9.5, 0.35, 5.0)) < 1e-4, "{moved}");
        assert_eq!(world.get::<Transform>(off).unwrap().translation, Vec3::new(12.0, 0.0, 5.0));
    }
}
