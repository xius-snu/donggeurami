//! Bridge Wars' sticks of dynamite, as they look (since 2026-11-13;
//! Hajun's `bridgewarsgameevent.md`: "it will throw an arched dynamite (2
//! minimalistic mini red cylinder shape with smaller brown cylinder
//! sticking out of one face which is the string), and after landing on the
//! ground/bridge, it will show a circular blast radius range around it and
//! it will shine bright red 3 time before it explode, and when it explode,
//! it will do 20 damage to player in that blast radius and destroy that
//! blast radius region of the bridge"): two red sticks side by side with a
//! brown fuse out of one end, tumbling end over end along an arch from the
//! hand to the ground ([`Lob`]: thrown straight at where they come down,
//! rising toward the camera over you, and drawn up the screen as they rise
//! and back down it as they fall, as an arch is on a map: never to one
//! side of the throw as such, Hajun, 2026-11-13: "its a arch projectile
//! not a boomarang"); lying where they came down with a red ring on the
//! ground round them as far as the blast will reach; shining three times,
//! the sticks themselves and a glow round them; and then the fireball's
//! own ball of fire (`fire::burst`).
//!
//! This is only how they look, and where they are from frame to frame.
//! What a blast does is the rules' (`roundtown_net::bridge_wars`): the
//! server's online, where each stick is an entity it sends ([`Dynamite`])
//! and takes away again as it blows, saying where it blew and which
//! planks went with it ([`Blasted`], [`hear_blasts`]); and this device's
//! own offline (`arena::dynamites_offline`). Either way a stick here is
//! [`throw`]n as it sets out. One that comes down where there is nothing
//! to lie on falls on into the void, and never blows.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::{DYNAMITE_RADIUS, Lob, caught};
use roundtown_net::{Blasted, Dynamite};

use crate::arena::InArena;
use crate::bridge::Bridges;
use crate::fire;
use crate::island::{Islands, Venue};
use crate::range;
use crate::ui::{UiChanged, ui};

/// How thick each of the two sticks is, from its middle, and how long, in
/// metres: "mini", and still to be seen from the camera 15 m over you,
/// where a metre is about fifty pixels of a phone's screen.
const STICK_RADIUS: f32 = 0.16;
const STICK_LENGTH: f32 = 0.8;
/// The fuse out of one end of them: how thick, from its middle, and how
/// long. A string, and brown as one (`colours.arena.dynamite_fuse`;
/// Hajun, 2026-11-13: "the dynamite's string part should be brown, like a
/// string"): thicker and longer than it was for its first day (0.045 and
/// 0.26), when from the camera over you it was a dark hair at the sticks'
/// end, and no colour at all.
const FUSE_RADIUS: f32 = 0.07;
const FUSE_LENGTH: f32 = 0.42;
/// How many times it turns end over end on its way.
const TUMBLES: f32 = 1.5;
/// How big the glow round it is at its brightest, in metres from its
/// middle, and how see-through even then.
const GLOW: f32 = 1.1;
const GLOW_ALPHA: f32 = 0.5;
/// How many times its own red the sticks shine at their brightest. (Bevy
/// takes no exposure to emitted light: a glow is so many times the colour,
/// as the anchors' is, `lamps`.)
const SHINE: f32 = 5.0;
/// How high over the ground it lies on the ring of its blast's reach is
/// drawn, in metres: clear of a plank's top and an island's.
const RING_OVER: f32 = 0.06;
/// How fast one that found nothing to lie on falls faster, in metres a
/// second a second, as anything falls into the void; and how far it falls
/// before it is taken away.
const FALL: f32 = 33.0;
const FALLEN: f32 = 35.0;
/// How near where the server says one blew a stick here has to be aimed to
/// be that one, in metres.
const SAME: f32 = 0.5;

/// A stick of dynamite thrown: how far through its throw and its fuse it
/// is ([`Lob`]); who threw it, by seat; whether the server runs it, and
/// then its entity for it, which is gone the moment the server has it
/// blow or fall; what has become of it ([`Lies`]); and which way it lies.
#[derive(Component)]
pub(crate) struct Stick {
    pub(crate) lob: Lob,
    pub(crate) seat: usize,
    pub(crate) served: bool,
    source: Option<Entity>,
    pub(crate) lies: Lies,
    yaw: f32,
}

/// What has become of a stick: still in the air; down, on ground, with
/// its fuse burning; or down where there was nothing to lie on, and
/// falling, this fast.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Lies {
    Flying,
    Lying,
    Falling(f32),
}

impl Stick {
    /// Whether it lies with its fuse burnt down: for whoever runs the
    /// game to blow.
    pub(crate) fn blows(&self) -> bool {
        self.lies == Lies::Lying && self.lob.blown()
    }

    /// Whether it is still to blow, as far as anyone can tell: in the air,
    /// or lying.
    pub(crate) fn live(&self) -> bool {
        !matches!(self.lies, Lies::Falling(_))
    }
}

/// A stick's own looks, to shine: its red, and the glow round it.
#[derive(Component)]
struct Shining {
    red: Handle<StandardMaterial>,
    glow: Handle<StandardMaterial>,
}

/// The part of a stick that tumbles; the glow round it; and the ring on
/// the ground round where it lies.
#[derive(Component)]
struct Tumbling;
#[derive(Component)]
struct Glow;
#[derive(Component)]
struct Reach;

/// The sticks' shapes and what they share of their looks, made as the app
/// opens.
#[derive(Resource)]
struct Look {
    stick: Handle<Mesh>,
    fuse: Handle<Mesh>,
    ball: Handle<Mesh>,
    brown: Handle<StandardMaterial>,
    ring: [Handle<Mesh>; 3],
    ring_fill: Handle<StandardMaterial>,
    ring_outline: Handle<StandardMaterial>,
    ring_rim: Handle<StandardMaterial>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, load)
        .add_systems(Update, ((follow_server, hear_blasts, fit_out, fly).chain(), recolour));
}

fn load(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let colours = ui().colours.arena;
    let (fill, outline, rim) = range::disc(DYNAMITE_RADIUS);
    commands.insert_resource(Look {
        stick: meshes.add(Cylinder::new(STICK_RADIUS, STICK_LENGTH)),
        fuse: meshes.add(Cylinder::new(FUSE_RADIUS, FUSE_LENGTH)),
        ball: meshes.add(Sphere::new(1.0).mesh().ico(3).expect("a sphere")),
        brown: materials.add(StandardMaterial {
            base_color: colours.dynamite_fuse,
            perceptual_roughness: 0.9,
            ..default()
        }),
        ring: [meshes.add(fill), meshes.add(outline), meshes.add(rim)],
        ring_fill: materials.add(range::look(colours.blast_fill)),
        ring_outline: materials.add(range::look(colours.blast_outline)),
        ring_rim: materials.add(range::look(colours.aim_rim)),
    });
}

/// A stick of dynamite setting out on `lob`, thrown by whoever is in
/// `seat`; online, with the server's entity for it. What it looks like is
/// put on it a frame later ([`fit_out`]).
pub(crate) fn throw(commands: &mut Commands, lob: Lob, seat: usize, source: Option<Entity>) -> Entity {
    let along = lob.to - lob.from;
    commands
        .spawn((
            Stick {
                lob,
                seat,
                served: source.is_some(),
                source,
                lies: Lies::Flying,
                yaw: (-along.x).atan2(-along.z),
            },
            InArena,
            Venue::Arena,
            Name::new("Dynamite"),
            Transform::from_translation(place(&lob)),
            // Shown with the island it is on (`island::show_where_i_am`).
            Visibility::Hidden,
        ))
        .id()
}

/// Where a stick on `lob` is drawn: where it is, and as high over that as
/// it is thick, so that lying it lies on the ground and not in it.
fn place(lob: &Lob) -> Vec3 {
    lob.at() + Vec3::Y * STICK_RADIUS
}

/// How bright a stick's red is, shining `shine`, from 0 to 1: what it
/// gives off, on top of what lights it.
fn shining(red: Color, shine: f32) -> LinearRgba {
    let red = red.to_linear();
    let times = SHINE * shine.clamp(0.0, 1.0);
    LinearRgba::rgb(red.red * times, red.green * times, red.blue * times)
}

/// Online, the sticks are the server's: each it sends is thrown here as
/// it comes.
fn follow_server(fresh: Query<(Entity, &Dynamite), (With<Remote>, Added<Dynamite>)>, mut commands: Commands) {
    for (source, stick) in &fresh {
        let lob = Lob {
            from: Vec3::new(stick.x, stick.y, stick.z),
            to: Vec3::new(stick.to_x, stick.to_y, stick.to_z),
            age: 0.0,
        };
        throw(&mut commands, lob, usize::from(stick.seat), Some(source));
    }
}

/// Online, a stick the server has blown ([`Blasted`]): the planks it says
/// went with it are gone (`bridge::Bridges::break_planks`), and so is the
/// plank you are laying if the blast reached its middle, which only this
/// device knows of; the blast is drawn where it blew; and the stick drawn
/// here for it is taken away.
fn hear_blasts(
    blasts: Query<&Blasted, (With<Remote>, Added<Blasted>)>,
    sticks: Query<(Entity, &Stick)>,
    mut bridges: ResMut<Bridges>,
    mut commands: Commands,
) {
    for blast in &blasts {
        let at = Vec3::new(blast.x, blast.y, blast.z);
        bridges.break_planks(&blast.planks);
        bridges.blast_tip(at.xz(), DYNAMITE_RADIUS);
        fire::burst(&mut commands, at + Vec3::Y * STICK_RADIUS);
        info!("a stick of dynamite blew at {:.1}, taking {} planks", at.xz(), blast.planks.len());
        if let Some((stick, _)) = sticks
            .iter()
            .filter(|(_, stick)| stick.served && stick.lob.to.distance(at) < SAME)
            .min_by(|a, b| b.1.lob.age.total_cmp(&a.1.lob.age))
        {
            commands.entity(stick).try_despawn();
        }
    }
}

/// Puts on each stick what it looks like, the frame after it is thrown:
/// its two sticks and its fuse, which tumble together; the glow round it;
/// and the ring on the ground, put away until it lies there.
fn fit_out(
    look: Res<Look>,
    fresh: Query<Entity, Added<Stick>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let colours = ui().colours.arena;
    for stick in &fresh {
        let red = materials.add(StandardMaterial {
            base_color: colours.dynamite,
            perceptual_roughness: 0.7,
            ..default()
        });
        let glow = materials.add(StandardMaterial {
            base_color: colours.dynamite.with_alpha(0.0),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        // A cylinder stands up its own y: laid along the stick's z.
        let lying = Quat::from_rotation_x(FRAC_PI_2);
        commands
            .entity(stick)
            .insert(Shining {
                red: red.clone(),
                glow: glow.clone(),
            })
            .with_children(|parts| {
                parts.spawn((Tumbling, Transform::default(), Visibility::Inherited)).with_children(|body| {
                    for side in [-1.0, 1.0] {
                        body.spawn((
                            Mesh3d(look.stick.clone()),
                            MeshMaterial3d(red.clone()),
                            Transform::from_xyz(side * STICK_RADIUS, 0.0, 0.0).with_rotation(lying),
                            NotShadowCaster,
                        ));
                    }
                    // The fuse, out of the end that leads.
                    body.spawn((
                        Mesh3d(look.fuse.clone()),
                        MeshMaterial3d(look.brown.clone()),
                        Transform::from_xyz(0.0, 0.0, -(STICK_LENGTH + FUSE_LENGTH) * 0.5).with_rotation(lying),
                        NotShadowCaster,
                    ));
                });
                parts.spawn((
                    Glow,
                    Mesh3d(look.ball.clone()),
                    MeshMaterial3d(glow),
                    Transform::from_scale(Vec3::splat(0.01)),
                    Visibility::Hidden,
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                parts
                    .spawn((Reach, Transform::from_xyz(0.0, RING_OVER - STICK_RADIUS, 0.0), Visibility::Hidden))
                    .with_children(|ring| {
                        let sheets = [(&look.ring[2], &look.ring_rim, 0.0), (&look.ring[0], &look.ring_fill, 0.0), (&look.ring[1], &look.ring_outline, 0.002)];
                        for (mesh, material, over) in sheets {
                            ring.spawn((
                                Mesh3d(mesh.clone()),
                                MeshMaterial3d(material.clone()),
                                Transform::from_xyz(0.0, over, 0.0),
                                NotShadowCaster,
                                NotShadowReceiver,
                            ));
                        }
                    });
            });
    }
}

/// Moves every stick on: along its arch, tumbling; as it comes down, onto
/// the ground if there is any for it to lie on ([`caught`]), with the ring
/// of its blast's reach round it, and on down into the void if there is
/// not; shining as its fuse burns ([`Lob::shine`]). One of the server's
/// that the server has taken away without a blast ([`hear_blasts`] takes
/// those) has fallen into the void there: here it falls on from where it
/// is, or, if this device had it lying on something, is gone.
#[allow(clippy::type_complexity)]
fn fly(
    time: Res<Time>,
    islands: Res<Islands>,
    sources: Query<(), With<Dynamite>>,
    mut sticks: Query<(Entity, &mut Stick, &mut Transform, Option<&Shining>, Option<&Children>)>,
    mut tumbling: Query<&mut Transform, (With<Tumbling>, Without<Stick>, Without<Glow>)>,
    mut glows: Query<(&mut Transform, &mut Visibility), (With<Glow>, Without<Stick>, Without<Tumbling>, Without<Reach>)>,
    mut rings: Query<&mut Visibility, (With<Reach>, Without<Glow>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let colours = ui().colours.arena;
    let island = islands.get(Venue::Arena);
    for (entity, mut stick, mut transform, looks, children) in &mut sticks {
        // Taken away by the server with no blast: it found nothing to lie
        // on there.
        if stick.served && stick.source.is_some_and(|source| !sources.contains(source)) {
            if stick.lies == Lies::Lying {
                commands.entity(entity).despawn();
                continue;
            }
            stick.source = None;
        }
        match stick.lies {
            Lies::Flying => {
                stick.lob.tick(dt);
                transform.translation = place(&stick.lob);
                if stick.lob.down() {
                    let to = stick.lob.to;
                    let ground = |spot: Vec2| island.and_then(|island| island.floor(spot, to.y + 1.0)).is_some_and(|floor| (floor - to.y).abs() < 1.0);
                    // One of the server's it has taken away already fell
                    // there, whatever this device makes of the ground.
                    let gone = stick.served && stick.source.is_none();
                    stick.lies = if !gone && caught(to.xz(), ground) {
                        Lies::Lying
                    } else {
                        // On down as fast as it was coming.
                        Lies::Falling(4.0 * stick.lob.apex() / stick.lob.secs())
                    };
                }
            }
            Lies::Lying => {
                // This device's own are blown by its game, once their
                // fuse has burnt down (`arena::dynamites_offline`); the
                // server's by the server.
                stick.lob.tick(dt);
                transform.translation = place(&stick.lob);
            }
            Lies::Falling(speed) => {
                transform.translation.y -= speed * dt;
                stick.lies = Lies::Falling(speed + FALL * dt);
                if transform.translation.y < stick.lob.to.y - FALLEN {
                    commands.entity(entity).despawn();
                    continue;
                }
            }
        }
        let lying = stick.lies == Lies::Lying;
        let shine = if lying { stick.lob.shine() } else { 0.0 };
        let turn = match stick.lies {
            // End over end on its way, and on as it falls.
            Lies::Flying => (stick.lob.age / stick.lob.secs()).clamp(0.0, 1.0) * TUMBLES * TAU,
            Lies::Lying => 0.0,
            Lies::Falling(speed) => speed * 0.6,
        };
        let turned = Quat::from_rotation_y(stick.yaw) * Quat::from_rotation_x(turn);
        for &child in children.into_iter().flatten() {
            if let Ok(mut body) = tumbling.get_mut(child) {
                if body.rotation != turned {
                    body.rotation = turned;
                }
            } else if let Ok((mut glow, mut shown)) = glows.get_mut(child) {
                shown.set_if_neq(if shine > 0.01 { Visibility::Inherited } else { Visibility::Hidden });
                let size = Vec3::splat((GLOW * (0.5 + 0.5 * shine)).max(0.01));
                if glow.scale != size {
                    glow.scale = size;
                }
            } else if let Ok(mut shown) = rings.get_mut(child) {
                shown.set_if_neq(if lying { Visibility::Inherited } else { Visibility::Hidden });
            }
        }
        if let Some(looks) = looks {
            let glowing = shining(colours.dynamite, shine);
            if let Some(mut red) = materials.get_mut(&looks.red)
                && red.emissive != glowing
            {
                red.emissive = glowing;
            }
            let alpha = GLOW_ALPHA * shine;
            if let Some(mut glow) = materials.get_mut(&looks.glow)
                && (glow.base_color.alpha() - alpha).abs() > 1e-3
            {
                glow.base_color = range::bright(colours.dynamite).with_alpha(alpha);
            }
        }
    }
}

/// Takes the ring's colours and the fuse's again as the file changes on
/// desktop; a stick's own red is taken as it next shines ([`fly`]).
fn recolour(changed: Res<UiChanged>, look: Res<Look>, mut materials: ResMut<Assets<StandardMaterial>>) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let colours = ui().colours.arena;
    for (handle, colour) in [
        (&look.ring_fill, colours.blast_fill),
        (&look.ring_outline, colours.blast_outline),
        (&look.ring_rim, colours.aim_rim),
    ] {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color = range::bright(colour);
        }
    }
    if let Some(mut brown) = materials.get_mut(&look.brown) {
        brown.base_color = colours.dynamite_fuse;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stick_is_drawn_on_the_ground_it_lies_on_and_shines_only_as_its_fuse_burns() {
        let feet = Vec3::new(0.0, 0.25, 130.0);
        let mut lob = Lob::thrown(feet, 0.0, 6.0, 0.24);
        // In the air it gives off nothing; lying, at the height of each
        // flash, several times its own red.
        let red = Color::srgb(0.9, 0.2, 0.15);
        assert_eq!(shining(red, lob.shine()), LinearRgba::rgb(0.0, 0.0, 0.0));
        let mut brightest = 0.0f32;
        while !lob.blown() {
            lob.tick(1.0 / 60.0);
            let glowing = shining(red, lob.shine());
            assert!(lob.down() || glowing.red == 0.0);
            brightest = brightest.max(glowing.red);
        }
        assert!(brightest > 3.0 * red.to_linear().red, "{brightest}");
        // Lying, its sticks rest on the ground: their middles as high over
        // it as they are thick.
        assert!((place(&lob).y - (0.24 + STICK_RADIUS)).abs() < 1e-5);
        assert!(place(&lob).xz().distance(Vec2::new(0.0, 124.0)) < 1e-4);
        // The sticks are small, and still wider across than the fuse,
        // which is long enough to be seen for the string it is.
        assert!(STICK_LENGTH < 1.0 && 4.0 * STICK_RADIUS < 1.0 && FUSE_RADIUS < 0.5 * STICK_RADIUS);
        assert!(FUSE_LENGTH > 0.4 * STICK_LENGTH && FUSE_LENGTH < STICK_LENGTH);
        // One that lies with its fuse burnt down is for the game to blow;
        // one falling never is.
        let mut stick = Stick {
            lob,
            seat: 0,
            served: false,
            source: None,
            lies: Lies::Lying,
            yaw: 0.0,
        };
        assert!(stick.blows() && stick.live());
        stick.lies = Lies::Falling(3.0);
        assert!(!stick.blows() && !stick.live());
        stick.lies = Lies::Flying;
        assert!(!stick.blows() && stick.live());
    }
}
