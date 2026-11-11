//! Bridge Wars' fireballs, as they look (since 2026-11-12; Hajun's
//! `bridgewarsgameevent.txt`: "keep design minimal like do a red glowing
//! sphere for the fireball, and a minimal explosion effect"): a red ball
//! with a fainter glow round it, flying straight and level, and where it
//! bursts a ball of fire that swells to as far as the blast reaches and
//! fades, in a third of a second.
//!
//! This is only how they look, and where they are from frame to frame.
//! What a fireball hits and what its blast does are the rules'
//! (`roundtown_net::bridge_wars::Flight`): the server's online, where each
//! fireball is an entity it sends ([`Fireball`]) and takes away again the
//! moment it bursts, and this device's own offline
//! (`arena::fireballs_offline`). Either way a ball here is [`throw`]n as
//! it sets out and [`burst`] where it ends.

use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::Fireball;
use roundtown_net::bridge_wars::{BLAST_RADIUS, FIREBALL_RADIUS, FIREBALL_RANGE, Flight};

use crate::arena::InArena;
use crate::island::Venue;

/// How much bigger than the ball its glow is; how long a blast takes, in
/// seconds; and how big it is as it begins, in metres across its middle.
const GLOW: f32 = 1.8;
const BLAST_SECS: f32 = 0.35;
const BLAST_FROM: f32 = 0.6;
/// How far short of the end of its range a ball the server has burst can
/// be, in metres, and still be taken to have burst there: it sets out
/// here a moment after it does on the server, and is that far behind.
const BEHIND: f32 = 4.0;

/// A fireball in the air: how far it has flown; who threw it, by seat;
/// and, online, the server's entity for it, which is gone the moment the
/// server has it burst.
#[derive(Component)]
pub(crate) struct Ball {
    pub(crate) flight: Flight,
    pub(crate) seat: usize,
    pub(crate) source: Option<Entity>,
}

impl Ball {
    /// Whether it has flown its range.
    pub(crate) fn spent(&self) -> bool {
        self.flight.flown >= FIREBALL_RANGE
    }
}

/// A blast, and how long it has been going; and its fire, which is its
/// own, to fade.
#[derive(Component)]
struct Blast {
    age: f32,
}

/// The ball's shape and its colours, made as the app opens.
#[derive(Resource)]
pub(crate) struct Look {
    ball: Handle<Mesh>,
    fire: Handle<StandardMaterial>,
    glow: Handle<StandardMaterial>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, load).add_systems(Update, (follow_server, fly, blasts).chain());
}

fn load(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(Look {
        ball: meshes.add(Sphere::new(1.0).mesh().ico(3).expect("a sphere")),
        // Unlit, and brighter than its colour, as the sun's disc is: it
        // glows by day and by night alike.
        fire: materials.add(StandardMaterial {
            base_color: LinearRgba::rgb(2.6, 0.1, 0.04).into(),
            unlit: true,
            ..default()
        }),
        glow: materials.add(StandardMaterial {
            base_color: LinearRgba::new(1.0, 0.07, 0.02, 0.5).into(),
            unlit: true,
            alpha_mode: AlphaMode::Add,
            ..default()
        }),
    });
}

/// A fireball setting out on `flight`, thrown by whoever is in `seat`;
/// online, with the server's entity for it.
pub(crate) fn throw(commands: &mut Commands, look: &Look, flight: Flight, seat: usize, source: Option<Entity>) -> Entity {
    commands
        .spawn((
            Ball { flight, seat, source },
            InArena,
            Venue::Arena,
            Name::new("Fireball"),
            Mesh3d(look.ball.clone()),
            MeshMaterial3d(look.fire.clone()),
            Transform::from_translation(flight.at()).with_scale(Vec3::splat(FIREBALL_RADIUS)),
            // Shown with the island it is on (`island::show_where_i_am`).
            Visibility::Hidden,
            children![(
                Mesh3d(look.ball.clone()),
                MeshMaterial3d(look.glow.clone()),
                Transform::from_scale(Vec3::splat(GLOW)),
            )],
        ))
        .id()
}

/// A blast at `at`: a ball of fire that swells and fades ([`blasts`]).
pub(crate) fn burst(commands: &mut Commands, at: Vec3) {
    commands.spawn((
        Blast { age: 0.0 },
        InArena,
        Venue::Arena,
        Name::new("Blast"),
        Transform::from_translation(at).with_scale(Vec3::splat(BLAST_FROM)),
        Visibility::Hidden,
    ));
}

/// Online, the fireballs are the server's: each it sends is thrown here as
/// it comes.
fn follow_server(look: Res<Look>, fresh: Query<(Entity, &Fireball), (With<Remote>, Added<Fireball>)>, mut commands: Commands) {
    for (source, ball) in &fresh {
        let flight = Flight {
            from: Vec3::new(ball.x, ball.y, ball.z),
            heading: ball.heading,
            flown: 0.0,
        };
        throw(&mut commands, &look, flight, usize::from(ball.seat), Some(source));
    }
}

/// Flies every ball on, and bursts the server's where they are once the
/// server has taken them away.
pub(crate) fn fly(
    time: Res<Time>,
    sources: Query<(), With<Fireball>>,
    mut balls: Query<(Entity, &mut Ball, &mut Transform)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut ball, mut transform) in &mut balls {
        ball.flight.fly(dt);
        transform.translation = ball.flight.at();
        let Some(source) = ball.source else {
            continue;
        };
        if sources.contains(source) {
            continue;
        }
        // Burst, on the server: at the end of its range, if it was nearly
        // there; otherwise where it is, on someone.
        if FIREBALL_RANGE - ball.flight.flown < BEHIND {
            ball.flight.flown = FIREBALL_RANGE;
        }
        burst(&mut commands, ball.flight.at());
        commands.entity(entity).despawn();
    }
}

/// Swells and fades every blast, each with a fire of its own, and takes it
/// away once it is spent.
fn blasts(
    time: Res<Time>,
    look: Res<Look>,
    mut going: Query<(Entity, &mut Blast, &mut Transform, Option<&MeshMaterial3d<StandardMaterial>>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut blast, mut transform, fire) in &mut going {
        let Some(fire) = fire else {
            commands.entity(entity).insert((
                Mesh3d(look.ball.clone()),
                // Blended, not added: added to whatever is under it, a
                // fire over the island's pale top came out a pale pink,
                // and over the night sky a tan.
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: fire_at(0.0),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
            ));
            continue;
        };
        blast.age += dt;
        let done = blast.age / BLAST_SECS;
        if done >= 1.0 {
            materials.remove(&fire.0);
            commands.entity(entity).despawn();
            continue;
        }
        transform.scale = Vec3::splat(radius_at(done));
        if let Some(mut material) = materials.get_mut(&fire.0) {
            material.base_color = fire_at(done);
        }
    }
}

/// How big a blast is, `done` of the way through, 0 to 1: out fast and
/// then slowing, to as far as the blast reaches.
fn radius_at(done: f32) -> f32 {
    let out = 1.0 - (1.0 - done.clamp(0.0, 1.0)).powi(3);
    BLAST_FROM + (BLAST_RADIUS - BLAST_FROM) * out
}

/// A blast's fire, `done` of the way through: a yellow orange going to
/// red, and fading to nothing. Brighter than its colour, as everything
/// unlit is drawn, to come out of the tonemapping as fire rather than as
/// brick.
fn fire_at(done: f32) -> Color {
    let done = done.clamp(0.0, 1.0);
    let (hot, cool) = (Vec3::new(2.4, 1.25, 0.14), Vec3::new(1.5, 0.2, 0.02));
    let fire = hot.lerp(cool, done.powf(0.6));
    LinearRgba::new(fire.x, fire.y, fire.z, 0.92 * (1.0 - done).powf(1.5)).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_blast_swells_to_as_far_as_it_reaches_and_fades_to_nothing() {
        assert_eq!(radius_at(0.0), BLAST_FROM);
        assert!((radius_at(1.0) - BLAST_RADIUS).abs() < 1e-5);
        let (mut last, mut alpha) = (radius_at(0.0), fire_at(0.0).alpha());
        for step in 1..=20 {
            let done = step as f32 / 20.0;
            assert!(radius_at(done) >= last, "it shrinks");
            assert!(fire_at(done).alpha() <= alpha, "it brightens again");
            (last, alpha) = (radius_at(done), fire_at(done).alpha());
        }
        assert_eq!(fire_at(1.0).alpha(), 0.0);
        // A ball that has flown its range is spent.
        let mut ball = Ball {
            flight: Flight::thrown(Vec3::ZERO, 0.0),
            seat: 0,
            source: None,
        };
        assert!(!ball.spent());
        while !ball.flight.fly(0.05) {}
        assert!(ball.spent());
    }
}
