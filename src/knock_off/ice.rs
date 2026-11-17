//! What is drawn on the ice in a game of Knock Off, as it is seen from the
//! camera over it (mine, every one of them; Hajun asked for the game, not
//! for how it looks):
//!
//! * **Your arrow**, while you aim: from your feet, as long as your slide
//!   would go on an empty island (`knock_off::reach`), a flat shaft and head
//!   lying on the ice in your colour, with a ring where you would stop, and
//!   a dark rim round both so that they read on white ice by day and by
//!   night. Built again only when its length changes; turned and moved, not
//!   built, as its way changes. Nobody else's is drawn until the reveal,
//!   when everyone's is, each in its owner's colour, for as long as it is
//!   shown.
//! * **A ring at everyone's feet**, in their colour, so that who is who is
//!   plain from over the ice, names or not.
//! * **The cracks**: while everyone aims, a thin dark ring of cracks on the
//!   ice where it will break at the end of the turn (`next_radius`), with
//!   short cracks running out from it to the edge.
//! * **The ice breaking**: the island's model narrowed across the ground to
//!   the ice that is left (its land's `Transform.scale`, x and z, put back as
//!   the game ends), while a ring of slabs in the ice's own material, where
//!   the edge was, tips outward and drops into the sea. From 3 m down the
//!   island is a small floe.
//! * **A splash** where anyone goes into the sea, and a smaller one where a
//!   slab does: a ring spreading on the water and drops thrown up out of it.
//! * **A puff of ice dust** where two meet in a slide, with a small shake of
//!   the camera (`hold_the_camera`): a few centimetres, a tenth of a second.
//!
//! Everything here hangs under one root, which alone carries the venue (so
//! that it is drawn on the ice and nowhere else, `island::show_where_i_am`
//! seeing to that), each piece shown or hidden here; and goes with the
//! table (`InTable`). Unlit, drawn brighter than its colours as the punch's
//! range is (`range::bright`), for the tonemapping greys a plain colour.

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use roundtown_net::knock_off::{self as rules, KnockStage};

use super::{Knock, KnockSystems, bodies_by_seat, seat_colour};
use crate::island::Land;
use crate::table::{ComputerSeat, InTable, TableView};
use crate::ui::{UiChanged, ui};
use crate::{Dead, Player, Venue};

/// How long the camera shakes where two meet, in seconds.
pub(super) const SHAKE_SECS: f32 = 0.12;

/// How far over the ice what lies on it is drawn, in metres: clear of it
/// by enough that it never flickers through it from the camera's 30 m off,
/// and the rim a hair under the fill, so that the fill is drawn over it.
const ON_THE_ICE: f32 = 0.03;
const RIM_UNDER: f32 = 0.004;
/// The arrow, in metres: how far out from the middle of your feet it
/// begins (just outside a body), how wide its shaft is either side of its
/// middle, how wide its head is and how long; how wide its ring is, whose
/// outside is a body's reach (`knock_off::BODY`); and how wide the dark
/// rim round all of it is.
const ARROW_FROM: f32 = 0.75;
const SHAFT_HALF: f32 = 0.17;
const HEAD_HALF: f32 = 0.46;
const HEAD_LONG: f32 = 0.7;
const RING_WIDE: f32 = 0.14;
const RIM: f32 = 0.07;
/// The ring at everyone's feet: how far out its inside and outside are, in
/// metres.
const FEET_RING: (f32, f32) = (0.44, 0.56);
/// The cracks: how wide a line of them is, in metres; how far either way
/// across its line the ring of them wanders; and how long a crack running
/// out from it is, at the most.
const CRACK_WIDE: f32 = 0.07;
const CRACK_WANDER: f32 = 0.16;
const CRACK_OUT: f32 = 0.9;
/// The slabs the ice breaks into: how many at the most, how deep each is,
/// how fast they drop, in metres a second each second, and how far they
/// tip over as they go, in radians.
const SLABS: usize = 44;
const SLAB_DEEP: (f32, f32) = (0.45, 0.9);
const SLAB_DROP: f32 = 12.0;
const SLAB_TIP: f32 = 1.3;
/// How long each slab lies where it broke off before it goes, in seconds,
/// at the least and at the most: long enough to be seen as the edge, in
/// pieces, before it tips away.
const SLAB_WAIT: (f32, f32) = (0.12, 0.45);
/// A splash: how long it lasts, in seconds; how far its ring spreads, in
/// metres, for a body and for a slab; and how many drops it throws up.
const SPLASH_SECS: f32 = 0.9;
const SPLASH_SPREAD: (f32, f32) = (2.3, 0.75);
const SPLASH_DROPS: (usize, usize) = (12, 5);
/// A puff of ice dust: how long it lasts, in seconds, and how many motes.
const PUFF_SECS: f32 = 0.5;
const PUFF_MOTES: usize = 9;
/// How much the dust's and the drops' gravity is, in metres a second each
/// second.
const GRAVITY: f32 = 14.0;

/// Someone, or a slab of ice (`big` false), going into the sea at `at`.
#[derive(Message, Clone, Copy, Debug)]
pub(super) struct Splashed {
    pub at: Vec3,
    pub big: bool,
}

/// Two meeting in a slide, at `at` on the ice, `speed` metres a second the
/// faster for it.
#[derive(Message, Clone, Copy, Debug)]
pub(super) struct Met {
    pub at: Vec3,
}

/// How far the camera shakes where two meet at `speed`, in metres: a few
/// centimetres, more the harder they met.
pub(super) fn shake_for(speed: f32) -> f32 {
    (speed / 140.0).clamp(0.015, 0.06)
}

/// The root everything drawn on the ice hangs under, while you are at a
/// table of Knock Off.
#[derive(Component)]
struct IceRoot;

/// The arrow of the player in a seat: how long it was built for (its
/// meshes are its own), and its fill's and rim's meshes.
#[derive(Component)]
struct Arrow {
    seat: usize,
    built: Option<f32>,
    fill: Handle<Mesh>,
    rim: Handle<Mesh>,
}

/// The ring at the feet of whoever is in a seat.
#[derive(Component)]
struct FeetRing(usize);

/// The cracks, and where they were drawn for: the radius and the turn.
#[derive(Component)]
struct Cracks {
    built: Option<(u16, f32)>,
    mesh: Handle<Mesh>,
}

/// A slab of the ice's edge breaking off: where it began, which way is
/// out, how long it waits before it goes, its age, how deep it is, and
/// whether it has splashed.
#[derive(Component)]
struct Slab {
    from: Vec3,
    out: Vec2,
    turn: Quat,
    wait: f32,
    age: f32,
    splashed: bool,
}

/// A splash, how old, and its ring's material, which fades.
#[derive(Component)]
struct Splash {
    age: f32,
    ring: Handle<StandardMaterial>,
}

/// A splash's ring spreading on the water, and how far it spreads.
#[derive(Component)]
struct Ripple {
    spread: f32,
    age: f32,
}

/// One of a splash's drops, or of a puff's motes: how fast it goes, how big
/// it began, how old it is and how long it lasts.
#[derive(Component)]
struct Drop {
    velocity: Vec3,
    size: f32,
    age: f32,
    lasts: f32,
}

/// A puff of ice dust, and how old.
#[derive(Component)]
struct Puff {
    age: f32,
}

/// What everything on the ice is made of: each seat's colour, the dark rim,
/// the cracks, the dust, the water's white, and, until the ice's own is
/// found on its model, a pale ice for the slabs; and the shapes shared by
/// many: a ball, a box and a flat ring.
#[derive(Resource)]
struct IceLook {
    seats: [Handle<StandardMaterial>; 4],
    rim: Handle<StandardMaterial>,
    cracks: Handle<StandardMaterial>,
    dust: Handle<StandardMaterial>,
    white: Handle<StandardMaterial>,
    ice: Handle<StandardMaterial>,
    found_ice: bool,
    ball: Handle<Mesh>,
    slab: Handle<Mesh>,
    ring: Handle<Mesh>,
    feet_fill: Handle<Mesh>,
    feet_rim: Handle<Mesh>,
}

impl FromWorld for IceLook {
    fn from_world(world: &mut World) -> Self {
        let colours = ui().colours.knock_off;
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let seats = [0, 1, 2, 3].map(|seat| materials.add(flat(seat_colour(seat))));
        let rim = materials.add(flat(colours.rim));
        let cracks = materials.add(flat(colours.cracks));
        let dust = materials.add(flat(colours.dust));
        let white = materials.add(flat(colours.splash));
        let ice = materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.93, 1.0),
            perceptual_roughness: 0.3,
            ..default()
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let ball = meshes.add(Sphere::new(1.0).mesh().ico(1).expect("a ball"));
        let slab = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let ring = meshes.add(annulus(0.82, 1.0, 0.0));
        let feet_fill = meshes.add(annulus(FEET_RING.0, FEET_RING.1, 0.0));
        let feet_rim = meshes.add(rimmed_annulus(FEET_RING.0, FEET_RING.1));
        Self {
            seats,
            rim,
            cracks,
            dust,
            white,
            ice,
            found_ice: false,
            ball,
            slab,
            ring,
            feet_fill,
            feet_rim,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_message::<Splashed>()
        .add_message::<Met>()
        .init_resource::<IceLook>()
        .add_systems(
            Update,
            (
                put_up.after(KnockSystems),
                (find_the_ice, recolour, arrows, feet_rings, cracks, break_off, splash, puff)
                    .chain()
                    .after(super::place)
                    .before(crate::follow_camera),
                (fall_off, drip).after(crate::move_bodies),
            ),
        );
}

/// How much brighter than its colour what lies on the ice is drawn: the
/// tonemapping greys a plain colour, as it does the punch's range
/// (`range::GAIN`, 2.5, for a sheet as see-through as that one); less than
/// that for these solid ones, whose colours keep their own then, a red red
/// and not a coral, as the file has them.
const GAIN: f32 = 1.6;

/// `colour`, solid, [`GAIN`] times as bright.
fn bright(colour: Color) -> Color {
    let linear = colour.to_linear();
    Color::linear_rgb(linear.red * GAIN, linear.green * GAIN, linear.blue * GAIN)
}

/// A material for what lies on the ice: `colour`, unlit, solid, a little
/// brighter than it is ([`bright`]).
fn flat(colour: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: bright(colour),
        unlit: true,
        fog_enabled: false,
        ..default()
    }
}

/// Puts up the root and everything that lives under it as you sit down at
/// a table of Knock Off (the table took down everything of the one
/// before): an arrow for each seat, a ring for each seat's feet, and the
/// cracks, all hidden until wanted.
fn put_up(
    knock: Res<Knock>,
    roots: Query<(), With<IceRoot>>,
    look: Res<IceLook>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if knock.seat.is_none() || !roots.is_empty() {
        return;
    }
    let root = commands
        .spawn((IceRoot, InTable, Venue::KnockOff, Transform::default(), Visibility::default(), Name::new("Knock Off: the ice")))
        .id();
    for seat in 0..4 {
        let fill = meshes.add(annulus(0.1, 0.2, 0.0));
        let rim = meshes.add(annulus(0.2, 0.3, 0.0));
        commands.spawn((
            ChildOf(root),
            Arrow {
                seat,
                built: None,
                fill: fill.clone(),
                rim: rim.clone(),
            },
            Transform::default(),
            Visibility::Hidden,
            children![
                (Mesh3d(fill), MeshMaterial3d(look.seats[seat].clone()), Transform::from_xyz(0.0, ON_THE_ICE, 0.0), NotShadowCaster, NotShadowReceiver),
                (Mesh3d(rim), MeshMaterial3d(look.rim.clone()), Transform::from_xyz(0.0, ON_THE_ICE - RIM_UNDER, 0.0), NotShadowCaster, NotShadowReceiver),
            ],
        ));
        commands.spawn((
            ChildOf(root),
            FeetRing(seat),
            Transform::default(),
            Visibility::Hidden,
            children![
                (Mesh3d(look.feet_fill.clone()), MeshMaterial3d(look.seats[seat].clone()), Transform::from_xyz(0.0, ON_THE_ICE, 0.0), NotShadowCaster, NotShadowReceiver),
                (Mesh3d(look.feet_rim.clone()), MeshMaterial3d(look.rim.clone()), Transform::from_xyz(0.0, ON_THE_ICE - RIM_UNDER, 0.0), NotShadowCaster, NotShadowReceiver),
            ],
        ));
    }
    let crack_mesh = meshes.add(annulus(1.0, 1.1, 0.0));
    commands.spawn((
        ChildOf(root),
        Cracks {
            built: None,
            mesh: crack_mesh.clone(),
        },
        Mesh3d(crack_mesh),
        MeshMaterial3d(look.cracks.clone()),
        Transform::from_xyz(0.0, ON_THE_ICE * 0.5, 0.0),
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

/// The ice's own material, off its model, for the slabs that break off it,
/// once the model is there.
fn find_the_ice(
    mut look: ResMut<IceLook>,
    lands: Query<(Entity, &Venue), With<Land>>,
    children: Query<&Children>,
    materials: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    if look.found_ice {
        return;
    }
    for (land, venue) in &lands {
        if *venue != Venue::KnockOff {
            continue;
        }
        if let Some(found) = children.iter_descendants(land).find_map(|part| materials.get(part).ok()) {
            look.ice = found.0.clone();
            look.found_ice = true;
        }
    }
}

/// Everything takes the file's colours again as the file changes on
/// desktop (`ui::reread`).
fn recolour(changed: Res<UiChanged>, look: Res<IceLook>, mut materials: ResMut<Assets<StandardMaterial>>) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let colours = ui().colours.knock_off;
    let mut paint = |handle: &Handle<StandardMaterial>, colour: Color| {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color = bright(colour);
        }
    };
    for seat in 0..4 {
        paint(&look.seats[seat], seat_colour(seat));
    }
    paint(&look.rim, colours.rim);
    paint(&look.cracks, colours.cracks);
    paint(&look.dust, colours.dust);
    paint(&look.white, colours.splash);
}

/// Your arrow while you aim, and everyone's while they are shown: from the
/// feet, as long as the slide would go on an empty island, the ring where
/// it would stop. Each built again only as its length changes.
fn arrows(
    knock: Res<Knock>,
    mut arrows: Query<(&mut Arrow, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let round = knock.playing();
    for (mut arrow, mut place, mut shown) in &mut arrows {
        let seat = arrow.seat;
        let start = round.and_then(|round| round.start.get(seat).copied().flatten()).map(|(x, z)| Vec2::new(x, z));
        let in_play = round.is_some_and(|round| round.out.get(seat).is_some_and(Option::is_none));
        // Which way and how far: yours while you aim; anyone's once shown.
        let wanted = match (round, start) {
            (Some(round), Some(at)) if in_play => match round.stage {
                KnockStage::Aim if knock.seat == Some(seat as u8) && knock.aim.turn == round.turn => {
                    (knock.aim.power > 0 && knock.aim.way != Vec2::ZERO).then(|| (at, knock.aim.way, rules::reach(knock.aim.power)))
                }
                KnockStage::Reveal => {
                    let launch = round.launches.get(seat).map_or(Vec2::ZERO, |&(x, z)| Vec2::new(x, z));
                    let speed = launch.length();
                    (speed > 1e-3).then(|| (at, launch / speed, speed * speed / (2.0 * rules::ICE)))
                }
                _ => None,
            },
            _ => None,
        };
        let Some((at, way, reach)) = wanted else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        if arrow.built.is_none_or(|built| (built - reach).abs() > 1e-3) {
            let (fill, rim) = arrow_meshes(reach);
            if meshes.insert(&arrow.fill, fill).is_err() || meshes.insert(&arrow.rim, rim).is_err() {
                warn!("Knock Off: an arrow has lost its mesh");
            }
            arrow.built = Some(reach);
        }
        place.set_if_neq(Transform::from_xyz(at.x, 0.0, at.y).with_rotation(crate::range::turned_to(way)));
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// The ring at the feet of everyone on the ice, in their colour, under them
/// wherever they are on it; put away once they are off it.
#[allow(clippy::type_complexity)]
fn feet_rings(
    knock: Res<Knock>,
    view: Res<TableView>,
    me: Query<Entity, With<Player>>,
    remotes: Query<(Entity, &roundtown_net::Member), With<bevy_replicon::prelude::Remote>>,
    computers: Query<(Entity, &ComputerSeat)>,
    bodies: Query<(&Transform, &Venue, Has<Dead>), Without<FeetRing>>,
    mut rings: Query<(&FeetRing, &mut Transform, &mut Visibility)>,
) {
    let by_seat = bodies_by_seat(&knock, &view, me.single().ok(), &remotes, &computers);
    let taken = |seat: usize| view.members.get(seat).is_some_and(|member| member.id != 0) || knock.seat == Some(seat as u8);
    for (ring, mut place, mut shown) in &mut rings {
        let body = by_seat
            .get(ring.0)
            .copied()
            .flatten()
            .filter(|_| taken(ring.0))
            .and_then(|body| bodies.get(body).ok())
            .filter(|(at, venue, dead)| **venue == Venue::KnockOff && !dead && at.translation.y > -0.2 && at.translation.y < 0.5);
        let Some((at, ..)) = body else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        place.set_if_neq(Transform::from_xyz(at.translation.x, at.translation.y.min(0.0), at.translation.z));
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// The cracks where the ice will break at the end of the turn, while
/// everyone aims, looks and slides: built again for each turn.
fn cracks(knock: Res<Knock>, mut cracks: Query<(&mut Cracks, &mut Visibility)>, mut meshes: ResMut<Assets<Mesh>>) {
    let wanted = knock
        .playing()
        .filter(|round| matches!(round.stage, KnockStage::Aim | KnockStage::Reveal | KnockStage::Slide))
        .filter(|round| round.next_radius < round.radius - 0.01)
        .map(|round| (round.turn, round.next_radius, round.radius));
    for (mut cracked, mut shown) in &mut cracks {
        let Some((turn, next, now)) = wanted else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        if cracked.built != Some((turn, next)) {
            if meshes.insert(&cracked.mesh, crack_mesh(next, now, u64::from(turn))).is_err() {
                warn!("Knock Off: the cracks have lost their mesh");
            }
            cracked.built = Some((turn, next));
        }
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// The ice breaking at the end of a turn: the island's model narrowed to
/// the ice that is left (and whole again away from the game), and as it
/// begins to break, a ring of slabs where the edge was, tipping out and
/// dropping into the sea, a few of them with a splash.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn break_off(
    time: Res<Time>,
    knock: Res<Knock>,
    look: Res<IceLook>,
    roots: Query<Entity, With<IceRoot>>,
    mut lands: Query<(&Venue, &mut Transform), (With<Land>, Without<Slab>)>,
    mut slabs: Query<(Entity, &mut Slab, &mut Transform), Without<Land>>,
    mut splashes: MessageWriter<Splashed>,
    mut broke: Local<Option<u16>>,
    mut commands: Commands,
) {
    let round = knock.playing();
    let reach = match round {
        Some(round) if matches!(round.stage, KnockStage::Break | KnockStage::Over) => round.next_radius,
        Some(round) => round.radius,
        None => rules::RADIUS,
    };
    let across = (reach / rules::RADIUS).max(0.02);
    for (venue, mut land) in &mut lands {
        if *venue == Venue::KnockOff {
            let scale = Vec3::new(across, 1.0, across);
            if land.scale != scale {
                land.scale = scale;
            }
        }
    }
    // As it begins to break: the slabs, once a turn.
    if let Some(round) = round.filter(|round| round.stage == KnockStage::Break)
        && *broke != Some(round.turn)
        && let Ok(root) = roots.single()
    {
        *broke = Some(round.turn);
        let (outer, inner) = (round.radius, round.next_radius.max(0.0));
        let mut dice = fastrand::Rng::with_seed(u64::from(round.turn) * 7919);
        let middle = (outer + inner) * 0.5;
        let count = ((std::f32::consts::TAU * middle / 1.25).ceil() as usize).clamp(5, SLABS);
        let wide = (outer - inner).max(0.3);
        let spacing = std::f32::consts::TAU * middle / count as f32;
        for k in 0..count {
            let angle = (k as f32 + (dice.f32() - 0.5) * 0.3) * std::f32::consts::TAU / count as f32;
            let out = Vec2::from_angle(angle);
            let at = out * (middle + (dice.f32() - 0.5) * wide * 0.15);
            let deep = SLAB_DEEP.0 + (SLAB_DEEP.1 - SLAB_DEEP.0) * dice.f32();
            let long = (spacing * (1.0 + dice.f32() * 0.15)).max(0.3);
            let size = Vec3::new(long, deep, wide * (0.85 + dice.f32() * 0.2));
            let from = Vec3::new(at.x, -deep * 0.5 - dice.f32() * 0.04, at.y);
            let turn = crate::range::turned_to(Vec2::from_angle((dice.f32() - 0.5) * 0.25).rotate(out.perp()));
            commands.spawn((
                ChildOf(root),
                Slab {
                    from,
                    out,
                    turn,
                    wait: SLAB_WAIT.0 + (SLAB_WAIT.1 - SLAB_WAIT.0) * dice.f32(),
                    age: 0.0,
                    splashed: k % 3 != 0,
                },
                Mesh3d(look.slab.clone()),
                MeshMaterial3d(look.ice.clone()),
                Transform::from_translation(from).with_rotation(turn).with_scale(size),
                NotShadowCaster,
            ));
        }
    }
    // Away from a game being played, the next game's first break is its own.
    if round.is_none() {
        *broke = None;
    }
    // Each slab tips out and drops, and is gone under the sea.
    let dt = time.delta_secs();
    for (slab, mut thing, mut place) in &mut slabs {
        thing.age += dt;
        let going = (thing.age - thing.wait).max(0.0);
        if going > 2.0 || round.is_none() {
            commands.entity(slab).despawn();
            continue;
        }
        let drop = 0.5 * SLAB_DROP * going * going;
        let tip = (going * 1.8).min(1.0) * SLAB_TIP;
        let at = thing.from + Vec3::new(thing.out.x, 0.0, thing.out.y) * going * 1.2 - Vec3::Y * drop;
        let axis = Vec3::new(-thing.out.y, 0.0, thing.out.x);
        let turned = Quat::from_axis_angle(axis, -tip) * thing.turn;
        let scale = place.scale;
        place.set_if_neq(Transform::from_translation(at).with_rotation(turned).with_scale(scale));
        if !thing.splashed && at.y < crate::WATER_Y - 0.3 {
            thing.splashed = true;
            splashes.write(Splashed {
                at: Vec3::new(at.x, crate::WATER_Y, at.z),
                big: false,
            });
        }
    }
}

/// A splash where anything goes into the sea: a ring spreading out on the
/// water, fading as it goes, and drops thrown up out of it.
fn splash(
    mut splashed: MessageReader<Splashed>,
    roots: Query<Entity, With<IceRoot>>,
    look: Res<IceLook>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok(root) = roots.single() else {
        splashed.clear();
        return;
    };
    for one in splashed.read() {
        let (spread, drops) = if one.big { (SPLASH_SPREAD.0, SPLASH_DROPS.0) } else { (SPLASH_SPREAD.1, SPLASH_DROPS.1) };
        let ring = materials.add(StandardMaterial {
            base_color: crate::range::bright(ui().colours.knock_off.splash),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            fog_enabled: false,
            ..default()
        });
        let splash = commands
            .spawn((
                ChildOf(root),
                Splash {
                    age: 0.0,
                    ring: ring.clone(),
                },
                Transform::from_translation(one.at + Vec3::Y * 0.05),
                Visibility::default(),
            ))
            .id();
        commands.spawn((
            ChildOf(splash),
            Ripple { spread, age: 0.0 },
            Mesh3d(look.ring.clone()),
            MeshMaterial3d(ring),
            Transform::from_scale(Vec3::splat(0.3)),
            NotShadowCaster,
            NotShadowReceiver,
        ));
        let mut dice = fastrand::Rng::with_seed((one.at.x.to_bits() as u64) << 32 | u64::from(one.at.z.to_bits()));
        for k in 0..drops {
            let angle = (k as f32 + dice.f32()) * std::f32::consts::TAU / drops as f32;
            let out = Vec2::from_angle(angle) * (1.0 + dice.f32() * 2.0) * spread * 0.5;
            let up = (3.0 + dice.f32() * 3.0) * if one.big { 1.0 } else { 0.6 };
            let size = (0.06 + dice.f32() * 0.07) * if one.big { 1.0 } else { 0.7 };
            commands.spawn((
                ChildOf(splash),
                Drop {
                    velocity: Vec3::new(out.x, up, out.y),
                    size,
                    age: 0.0,
                    lasts: SPLASH_SECS,
                },
                Mesh3d(look.ball.clone()),
                MeshMaterial3d(look.white.clone()),
                Transform::from_scale(Vec3::splat(size)),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    }
}

/// A puff of ice dust where two meet: a few motes thrown out low over the
/// ice, shrinking away.
fn puff(mut met: MessageReader<Met>, roots: Query<Entity, With<IceRoot>>, look: Res<IceLook>, mut commands: Commands) {
    let Ok(root) = roots.single() else {
        met.clear();
        return;
    };
    for one in met.read() {
        let puff = commands
            .spawn((ChildOf(root), Puff { age: 0.0 }, Transform::from_translation(one.at + Vec3::Y * 0.35), Visibility::default()))
            .id();
        let mut dice = fastrand::Rng::with_seed(u64::from(one.at.x.to_bits()) ^ u64::from(one.at.z.to_bits()).rotate_left(17));
        for k in 0..PUFF_MOTES {
            let angle = (k as f32 + dice.f32()) * std::f32::consts::TAU / PUFF_MOTES as f32;
            let out = Vec2::from_angle(angle) * (1.6 + dice.f32() * 2.2);
            let size = 0.1 + dice.f32() * 0.1;
            commands.spawn((
                ChildOf(puff),
                Drop {
                    velocity: Vec3::new(out.x, 1.2 + dice.f32() * 2.0, out.y),
                    size,
                    age: 0.0,
                    lasts: PUFF_SECS,
                },
                Mesh3d(look.ball.clone()),
                MeshMaterial3d(look.dust.clone()),
                Transform::from_scale(Vec3::splat(size)),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    }
}

/// The splashes' rings spread and fade, and every drop and mote flies, falls
/// and shrinks away; each gone at its end.
#[allow(clippy::type_complexity)]
fn drip(
    time: Res<Time>,
    mut splashes: Query<(Entity, &mut Splash)>,
    mut puffs: Query<(Entity, &mut Puff)>,
    mut ripples: Query<(&mut Ripple, &mut Transform), Without<Drop>>,
    mut drops: Query<(&mut Drop, &mut Transform), Without<Ripple>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let white = crate::range::bright(ui().colours.knock_off.splash);
    for (entity, mut splash) in &mut splashes {
        splash.age += dt;
        let share = splash.age / SPLASH_SECS;
        if share >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(mut ring) = materials.get_mut(&splash.ring) {
            ring.base_color = white.with_alpha((1.0 - share) * 0.85);
        }
    }
    for (mut ripple, mut place) in &mut ripples {
        ripple.age += dt;
        let share = (ripple.age / SPLASH_SECS).min(1.0);
        place.set_if_neq(Transform::from_scale(Vec3::splat(0.3 + share.sqrt() * ripple.spread)));
    }
    for (entity, mut puff) in &mut puffs {
        puff.age += dt;
        if puff.age >= PUFF_SECS {
            commands.entity(entity).despawn();
        }
    }
    for (mut drop, mut place) in &mut drops {
        drop.age += dt;
        drop.velocity.y -= GRAVITY * dt;
        let share = (drop.age / drop.lasts).min(1.0);
        let size = drop.size * (1.0 - share * share).max(0.0);
        let at = place.translation + drop.velocity * dt;
        place.set_if_neq(Transform::from_translation(at).with_scale(Vec3::splat(size.max(1e-3))));
    }
}

/// Whoever walks off the ice while the table waits is seen into the sea
/// with a splash too.
fn fall_off(knock: Res<Knock>, me: Query<&Transform, With<Player>>, mut splashes: MessageWriter<Splashed>, mut was: Local<Option<f32>>) {
    if knock.seat.is_none() || knock.playing().is_some() {
        *was = None;
        return;
    }
    let Ok(at) = me.single() else {
        return;
    };
    let y = at.translation.y;
    if was.is_some_and(|was| was >= crate::WATER_Y + 0.2) && y < crate::WATER_Y + 0.2 {
        splashes.write(Splashed {
            at: Vec3::new(at.translation.x, crate::WATER_Y, at.translation.z),
            big: true,
        });
    }
    *was = Some(y);
}

// ------------------------------------------------------------- the shapes

/// The arrow's two sheets, lying on the ground along -Z from the middle of
/// the feet: its fill (shaft, head and the ring where it stops, `reach`
/// out) and the dark rim just outside all of it.
fn arrow_meshes(reach: f32) -> (Mesh, Mesh) {
    let ring_out = rules::BODY;
    let ring_in = ring_out - RING_WIDE;
    let tip = (reach - ring_out).max(ARROW_FROM + 0.2);
    let base = (tip - HEAD_LONG).max(ARROW_FROM + 0.05);
    let head_long = tip - base;
    let head_half = HEAD_HALF.min(head_long * 0.9).max(SHAFT_HALF + 0.08);
    let outline = [
        Vec2::new(-SHAFT_HALF, -ARROW_FROM),
        Vec2::new(-SHAFT_HALF, -base),
        Vec2::new(-head_half, -base),
        Vec2::new(0.0, -tip),
        Vec2::new(head_half, -base),
        Vec2::new(SHAFT_HALF, -base),
        Vec2::new(SHAFT_HALF, -ARROW_FROM),
    ];
    let mut fill = Sheet::default();
    // The shaft, and the head.
    fill.quad(outline[0], outline[1], outline[5], outline[6]);
    fill.triangle(outline[2], outline[3], outline[4]);
    let mut rim = Sheet::default();
    rim.band(&outline, RIM);
    // The ring where it stops, with its rim inside and out.
    let stop = Vec2::new(0.0, -reach);
    fill.ring(stop, ring_in, ring_out);
    rim.ring(stop, ring_out, ring_out + RIM);
    rim.ring(stop, (ring_in - RIM).max(0.0), ring_in);
    (fill.mesh(), rim.mesh())
}

/// A flat ring lying on the ground round the origin, from `inner` to
/// `outer` out, `up` over it.
fn annulus(inner: f32, outer: f32, up: f32) -> Mesh {
    let mut sheet = Sheet::default();
    sheet.ring(Vec2::ZERO, inner, outer);
    sheet.raised(up)
}

/// The dark rim round a ring at the feet: just outside it and just inside.
fn rimmed_annulus(inner: f32, outer: f32) -> Mesh {
    let mut sheet = Sheet::default();
    sheet.ring(Vec2::ZERO, outer, outer + RIM);
    sheet.ring(Vec2::ZERO, (inner - RIM).max(0.0), inner);
    sheet.mesh()
}

/// The cracks round the ice where it will break, `radius` out on ice
/// reaching `edge`: a ring of them that wanders a little either side of
/// the line, and short ones running out from it toward the edge, thrown
/// from `seed` (the turn's) so that each turn's are their own; on ice that
/// will be gone, cracks running out from the middle.
fn crack_mesh(radius: f32, edge: f32, seed: u64) -> Mesh {
    let mut dice = fastrand::Rng::with_seed(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x6372_6163);
    let mut sheet = Sheet::default();
    if radius > 0.3 {
        let steps = ((radius * std::f32::consts::TAU / 0.45).ceil() as usize).max(12);
        let points: Vec<Vec2> = (0..steps)
            .map(|k| {
                let angle = k as f32 / steps as f32 * std::f32::consts::TAU;
                Vec2::from_angle(angle) * (radius + (dice.f32() - 0.5) * 2.0 * CRACK_WANDER)
            })
            .collect();
        for k in 0..steps {
            sheet.line(points[k], points[(k + 1) % steps], CRACK_WIDE);
        }
        // Short cracks out from it, toward the edge.
        let runs = (steps / 4).max(4);
        for _ in 0..runs {
            let from = points[dice.usize(..steps)];
            let out = from.normalize_or(Vec2::X);
            let room = (edge - from.length()).max(0.0);
            let long = (CRACK_OUT * (0.4 + dice.f32() * 0.6)).min(room * 0.8);
            if long > 0.1 {
                let bend = out.perp() * (dice.f32() - 0.5) * long * 0.6;
                let middle = from + out * long * 0.5 + bend;
                sheet.line(from, middle, CRACK_WIDE * 0.8);
                sheet.line(middle, from + out * long, CRACK_WIDE * 0.6);
            }
        }
    } else {
        // Nothing will be left: cracks from the middle out.
        for k in 0..7 {
            let angle = (k as f32 + dice.f32() * 0.6) * std::f32::consts::TAU / 7.0;
            let out = Vec2::from_angle(angle) * edge * 0.9;
            let middle = out * 0.5 + out.perp() * (dice.f32() - 0.5) * 0.3;
            sheet.line(Vec2::ZERO, middle, CRACK_WIDE);
            sheet.line(middle, out, CRACK_WIDE * 0.7);
        }
    }
    sheet.mesh()
}

/// A flat sheet lying on the ground (x, z), built of triangles facing up.
#[derive(Default)]
struct Sheet {
    points: Vec<Vec2>,
    indices: Vec<u32>,
}

impl Sheet {
    /// The triangle `a`, `b`, `c`, whichever way round, facing up.
    fn triangle(&mut self, a: Vec2, b: Vec2, c: Vec2) {
        let first = self.points.len() as u32;
        // In (x, z) a triangle faces up (+Y) when its corners go round the
        // other way than they would seen on a page.
        let (b, c) = if (b - a).perp_dot(c - a) > 0.0 { (c, b) } else { (b, c) };
        self.points.extend([a, b, c]);
        self.indices.extend([first, first + 1, first + 2]);
    }

    /// The four-sided `a`, `b`, `c`, `d`, its corners in order round it.
    fn quad(&mut self, a: Vec2, b: Vec2, c: Vec2, d: Vec2) {
        self.triangle(a, b, c);
        self.triangle(a, c, d);
    }

    /// A line `wide` across from `from` to `to`.
    fn line(&mut self, from: Vec2, to: Vec2, wide: f32) {
        let across = (to - from).normalize_or(Vec2::X).perp() * wide * 0.5;
        self.quad(from - across, to - across, to + across, from + across);
    }

    /// A ring round `middle`, from `inner` to `outer` out.
    fn ring(&mut self, middle: Vec2, inner: f32, outer: f32) {
        let steps = 48;
        for k in 0..steps {
            let (a, b) = (
                Vec2::from_angle(k as f32 / steps as f32 * std::f32::consts::TAU),
                Vec2::from_angle((k + 1) as f32 / steps as f32 * std::f32::consts::TAU),
            );
            self.quad(middle + a * inner, middle + a * outer, middle + b * outer, middle + b * inner);
        }
    }

    /// A band `wide` just outside the closed `edge`, all the way round: each
    /// side's band, and at each corner a fan round it, so that it is whole
    /// round a corner going either way.
    fn band(&mut self, edge: &[Vec2], wide: f32) {
        let n = edge.len();
        // Which hand the outside is on, by which way round the edge goes.
        let turning: f32 = (0..n).map(|i| edge[i].perp_dot(edge[(i + 1) % n])).sum();
        let hand = if turning >= 0.0 { -1.0 } else { 1.0 };
        let outward = |i: usize| (edge[(i + 1) % n] - edge[i]).normalize_or(Vec2::X).perp() * hand;
        for i in 0..n {
            let j = (i + 1) % n;
            let out = outward(i) * wide;
            self.quad(edge[i], edge[j], edge[j] + out, edge[i] + out);
            // Round the corner at j, from this side's outward to the next's.
            let next = outward(j) * wide;
            self.triangle(edge[j], edge[j] + out, edge[j] + (out + next).normalize_or(out) * wide);
            self.triangle(edge[j], edge[j] + (out + next).normalize_or(out) * wide, edge[j] + next);
        }
    }

    /// As a mesh lying on the ground.
    fn mesh(self) -> Mesh {
        self.raised(0.0)
    }

    /// As a mesh lying `up` over the ground.
    fn raised(self, up: f32) -> Mesh {
        let positions: Vec<[f32; 3]> = self.points.iter().map(|point| [point.x, up, point.y]).collect();
        let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
        let uvs = vec![[0.0, 0.0]; positions.len()];
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(self.indices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The corners of every triangle of `mesh`, across the ground.
    fn triangles(mesh: &Mesh) -> Vec<[Vec2; 3]> {
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("positions");
        };
        let Some(Indices::U32(indices)) = mesh.indices() else {
            panic!("indices");
        };
        indices
            .chunks(3)
            .map(|three| {
                let at = |i: u32| Vec2::new(points[i as usize][0], points[i as usize][2]);
                [at(three[0]), at(three[1]), at(three[2])]
            })
            .collect()
    }

    #[test]
    fn every_arrow_is_as_long_as_its_slide_faces_up_and_has_something_in_it() {
        for power in 1..=rules::MOST_POWER {
            let reach = rules::reach(power);
            let (fill, rim) = arrow_meshes(reach);
            for mesh in [&fill, &rim] {
                let all = triangles(mesh);
                assert!(!all.is_empty(), "never a mesh with nothing in it");
                for [a, b, c] in all {
                    // Facing up: the corners go round the other way than on
                    // a page, in (x, z).
                    assert!((b - a).perp_dot(c - a) <= 1e-6, "power {power}");
                }
            }
            // The ring's far side is the slide's end and a body's reach.
            let furthest = triangles(&fill).iter().flatten().map(|at| -at.y).fold(0.0f32, f32::max);
            assert!((furthest - (reach + rules::BODY)).abs() < 0.02, "power {power}: {furthest}");
            // The head's tip short of the ring.
            assert!(triangles(&fill).iter().flatten().all(|at| at.y <= 0.0));
        }
    }

    #[test]
    fn the_cracks_lie_where_the_ice_will_break_and_never_past_its_edge() {
        for turn in 1..=rules::LAST_TURN {
            let (next, now) = (rules::radius_after(turn), rules::radius_in(turn));
            let mesh = crack_mesh(next, now, u64::from(turn));
            let all = triangles(&mesh);
            assert!(!all.is_empty(), "turn {turn}");
            for at in all.iter().flatten() {
                assert!(at.length() <= now + CRACK_WANDER + CRACK_WIDE, "turn {turn}: {at}");
            }
        }
    }

    #[test]
    fn a_shake_is_a_few_centimetres() {
        assert!((0.01..=0.06).contains(&shake_for(0.3)));
        assert!((0.01..=0.06).contains(&shake_for(30.0)));
        assert!(shake_for(10.0) >= shake_for(2.0));
    }
}
