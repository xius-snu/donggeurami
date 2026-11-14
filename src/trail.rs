//! The sparks someone on a potion leaves behind them in Bridge Wars (since
//! 2026-11-14; Hajun: "its also not noticeable when ur effect runs out ...
//! for speed potion maybe add a blue particle effect trail and for jump
//! potion maybe green particle effect trail"): blue ones from whoever runs
//! fast on a speed potion, green ones from whoever leaps on a leap potion,
//! for as long as the potion lasts, so that its running out is seen: the
//! sparks stop.
//!
//! Everyone's are drawn, not only yours: every device has what every seat
//! has drunk (`roundtown_net::Kits`, `arena::Wears`), so an enemy coming
//! fast is seen to be. Never those of someone unseen to you, an enemy on
//! an invisibility potion, whom they would give away; your own, and a
//! teammate's, are drawn as you both are.
//!
//! A spark is a small bright ball left where its body was, around its
//! legs, or under its feet while it is in the air, that rises a little,
//! shrinks, and is gone in half a second or so. Moving, a body leaves
//! twenty-two a second for each potion; standing, a few a second,
//! so that the potion is seen to be on. One mesh and one look to each
//! colour, in the file's `colours.arena.trail_speed` and `trail_leap`:
//! a spark is moved and shrunk, and nothing of it is made each frame.

use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use crate::arena::{InArena, Wears};
use crate::island::Venue;
use crate::range;
use crate::ui::{UiChanged, ui};
use crate::{Dead, Player, Team};

/// How long a spark lasts, in seconds, at the least and at the most.
const LIFE: (f32, f32) = (0.4, 0.75);
/// How far across a spark is as it is left, in metres, at the least and at
/// the most; it shrinks to nothing from there. (0.16 to 0.3, and sixteen a
/// second, as first made: from the camera over you a thin line of dots,
/// each a millimetre on a phone.)
const SIZE: (f32, f32) = (0.22, 0.42);
/// How long between two sparks of one potion from one body, in seconds:
/// on the move, and standing.
const EVERY_MOVING: f32 = 1.0 / 22.0;
const EVERY_STANDING: f32 = 1.0 / 5.0;
/// How fast a body has to be going to be on the move, in metres a second.
const MOVING: f32 = 1.0;
/// How fast a spark rises, in metres a second, at the least and the most.
const RISE: (f32, f32) = (0.3, 1.1);
/// How far from the middle of its body a spark is left, across the
/// ground, in metres; and how high over its feet, at the least and the
/// most: round its legs. In the air, round and under its feet instead.
const SPREAD: f32 = 0.34;
const HEIGHT: (f32, f32) = (0.1, 0.95);
const HEIGHT_IN_THE_AIR: (f32, f32) = (-0.25, 0.45);
/// How far up from where it stood a moment ago a body has to have gone to
/// be in the air, in metres a second.
const RISING: f32 = 1.5;

/// A spark, and which potion it is of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Speed,
    Leap,
}

/// What the sparks are drawn with: one ball, and a look to each kind.
#[derive(Resource)]
struct Look {
    ball: Handle<Mesh>,
    speed: Handle<StandardMaterial>,
    leap: Handle<StandardMaterial>,
}

/// One spark: how long it has been there, how long it lasts, how fast it
/// rises, and how far across it was as it was left.
#[derive(Component)]
struct Spark {
    age: f32,
    life: f32,
    rise: f32,
    size: f32,
}

/// A body that leaves sparks: how long until the next of each kind, and
/// where it stood last frame, to tell how fast it goes.
#[derive(Component, Default)]
struct Sparkling {
    speed_in: f32,
    leap_in: f32,
    was: Option<Vec3>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, ((leave, fly).chain().after(crate::move_bodies), recolour));
}

fn setup(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let colours = ui().colours.arena;
    commands.insert_resource(Look {
        ball: meshes.add(Sphere::new(0.5).mesh().ico(1).expect("a sphere")),
        speed: materials.add(range::look(colours.trail_speed)),
        leap: materials.add(range::look(colours.trail_leap)),
    });
}

/// Whether a body wearing `wears`, on `team`, is to leave sparks for
/// someone on `mine` to see, and of which kinds: fast, leaping, or both;
/// none from someone dead, nor from someone of another team who is
/// unseen.
fn kinds(wears: &Wears, team: Option<u8>, mine: Option<u8>, dead: bool) -> (bool, bool) {
    if dead || (wears.unseen && team != mine) {
        return (false, false);
    }
    (wears.fast, wears.leaping)
}

/// How long until the next spark of a body going `speed` metres a second:
/// oftener on the move.
fn every(speed: f32) -> f32 {
    if speed >= MOVING { EVERY_MOVING } else { EVERY_STANDING }
}

/// Leaves sparks behind every body on a speed potion or a leap potion.
#[allow(clippy::type_complexity)]
fn leave(
    time: Res<Time>,
    look: Res<Look>,
    me: Query<Option<&Team>, With<Player>>,
    mut bodies: Query<(Entity, &Transform, &Wears, &Venue, Option<&Team>, Has<Dead>, Option<&mut Sparkling>)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let mine = me.single().ok().flatten().map(|team| team.0);
    for (body, at, wears, &venue, team, dead, sparkling) in &mut bodies {
        let (fast, leaping) = kinds(wears, team.map(|team| team.0), mine, dead);
        let Some(mut sparkling) = sparkling else {
            // Only a body with a potion on is given the count.
            if (fast || leaping) && venue == Venue::Arena {
                commands.entity(body).insert(Sparkling::default());
            }
            continue;
        };
        let here = at.translation;
        let moved = sparkling.was.map_or(Vec3::ZERO, |was| here - was) / dt;
        sparkling.was = Some(here);
        if venue != Venue::Arena || !(fast || leaping) {
            continue;
        }
        let (speed, rising) = (moved.xz().length(), moved.y.abs() > RISING);
        let wait = every(speed.max(moved.y.abs()));
        for (on, kind) in [(fast, Kind::Speed), (leaping, Kind::Leap)] {
            let due = match kind {
                Kind::Speed => &mut sparkling.speed_in,
                Kind::Leap => &mut sparkling.leap_in,
            };
            if !on {
                *due = 0.0;
                continue;
            }
            *due -= dt;
            // (One at the most a frame: a frame is far shorter than the
            // wait, but for one that comes late.)
            if *due > 0.0 {
                continue;
            }
            *due = wait;
            let between = |(low, high): (f32, f32)| low + (high - low) * fastrand::f32();
            let round = Vec2::from_angle(fastrand::f32() * std::f32::consts::TAU) * SPREAD * fastrand::f32().sqrt();
            let up = between(if rising { HEIGHT_IN_THE_AIR } else { HEIGHT });
            let size = between(SIZE);
            commands.spawn((
                InArena,
                Spark {
                    age: 0.0,
                    life: between(LIFE),
                    rise: between(RISE),
                    size,
                },
                Mesh3d(look.ball.clone()),
                MeshMaterial3d(match kind {
                    Kind::Speed => look.speed.clone(),
                    Kind::Leap => look.leap.clone(),
                }),
                Transform::from_translation(here + Vec3::new(round.x, up, round.y)).with_scale(Vec3::splat(size)),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    }
}

/// How far across a spark `age` seconds old is, that lasts `life` and was
/// `size` across as it was left: shrinking evenly to nothing.
fn shrunk(size: f32, age: f32, life: f32) -> f32 {
    size * (1.0 - age / life.max(1e-3)).clamp(0.0, 1.0)
}

/// Raises and shrinks every spark, and takes each away as it is spent.
fn fly(time: Res<Time>, mut sparks: Query<(Entity, &mut Spark, &mut Transform)>, mut commands: Commands) {
    let dt = time.delta_secs();
    for (entity, mut spark, mut place) in &mut sparks {
        spark.age += dt;
        if spark.age >= spark.life {
            commands.entity(entity).despawn();
            continue;
        }
        place.translation.y += spark.rise * dt;
        place.scale = Vec3::splat(shrunk(spark.size, spark.age, spark.life));
    }
}

/// Takes the sparks' colours again as the file changes on desktop.
fn recolour(changed: Res<UiChanged>, look: Res<Look>, mut materials: ResMut<Assets<StandardMaterial>>) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let colours = ui().colours.arena;
    for (handle, colour) in [(&look.speed, colours.trail_speed), (&look.leap, colours.trail_leap)] {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color = range::bright(colour);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparks_come_from_whoever_is_on_a_potion_and_never_give_away_someone_unseen() {
        let on = |fast: bool, leaping: bool, unseen: bool| Wears {
            fast,
            leaping,
            unseen,
            ..Wears::default()
        };
        // Nothing drunk: none. A speed potion: blue ones; a leap potion:
        // green ones; both: both.
        assert_eq!(kinds(&on(false, false, false), Some(1), Some(0), false), (false, false));
        assert_eq!(kinds(&on(true, false, false), Some(1), Some(0), false), (true, false));
        assert_eq!(kinds(&on(false, true, false), Some(1), Some(0), false), (false, true));
        assert_eq!(kinds(&on(true, true, false), Some(0), Some(0), false), (true, true));
        // An enemy who is unseen leaves none, which would say where they
        // are; you and your own team, unseen, are drawn to each other,
        // sparks and all.
        assert_eq!(kinds(&on(true, true, true), Some(1), Some(0), false), (false, false));
        assert_eq!(kinds(&on(true, true, true), Some(0), Some(0), false), (true, true));
        // The dead leave none.
        assert_eq!(kinds(&on(true, true, false), Some(0), Some(0), true), (false, false));
    }

    #[test]
    fn a_spark_shrinks_to_nothing_and_comes_oftener_on_the_move() {
        assert!((shrunk(0.3, 0.0, 0.6) - 0.3).abs() < 1e-6);
        assert!((shrunk(0.3, 0.3, 0.6) - 0.15).abs() < 1e-6);
        assert_eq!(shrunk(0.3, 0.6, 0.6), 0.0);
        assert_eq!(shrunk(0.3, 5.0, 0.6), 0.0);
        assert!(every(10.0) < every(0.0) && every(MOVING) == EVERY_MOVING);
        // No more than three dozen in the air from one body on both
        // potions at a run.
        let most = 2.0 * LIFE.1 / EVERY_MOVING;
        assert!(most <= 36.0, "{most}");
        assert!(SIZE.0 > 0.1 && SIZE.1 < 0.5 && LIFE.0 > 0.2);
    }
}
