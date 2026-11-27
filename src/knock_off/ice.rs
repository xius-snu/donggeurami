//! What is drawn on the ice in a game of Knock Off, as it is seen from the
//! camera over it (mine, every one of them; Hajun asked for the game, not
//! for how it looks):
//!
//! * **Your arrow**, while you aim: from your feet, as long as your slide
//!   would go on an empty island (`knock_off::reach`), a flat shaft and head
//!   lying on the ice in your colour, with a ring where you would stop, and
//!   a dark rim round both so that they read on white ice by day and by
//!   night. Where the slide would end in the sea, the shaft goes on in
//!   dashes past the edge and a cross stands where the ring would. Built
//!   again only when its length, or where it leaves the ice, changes;
//!   turned and moved, not built, as its way changes otherwise. Nobody
//!   else's is drawn until the reveal, when everyone's is, each in its
//!   owner's colour, for as long as it is shown.
//! * **A ring at everyone's feet**, in their colour, so that who is who is
//!   plain from over the ice, names or not; and **a streak behind whoever
//!   slides**, in their colour, fading, so that who went where can be
//!   followed.
//! * **The ice that will break**: while everyone aims, a thin dark ring of
//!   cracks on the ice where it will break at the end of the turn
//!   (`next_size`), with short cracks running out from it to the edge,
//!   and the band from there to the edge darker, as thin ice is. By night
//!   the ice that will stay has the moon on it, so that the bodies on it
//!   stand out and the band does too.
//! * **The ice breaking**: the island narrowed across the ground toward its
//!   middle to the ice that is left (its land's `Transform.scale`, x and z,
//!   put back as the game ends), while the band breaks into pieces of the
//!   ice's own material, of every size, the crack running round the rim from
//!   where it began and each piece tipping outward into the sea as it
//!   reaches it. From a quarter of it down the island is a small floe.
//! * **The island itself** (since 2026-11-27): the table's own outline
//!   (`rules::Floe`), its ice 3 m deep in the ice's own material off Hajun's
//!   `iceisland.glb`, whose own disc is put away; and the shape everyone
//!   walks about on while the table waits (`Islands::reshape`). Every ring
//!   that was round the disc (the cracks, the band of ice that will break,
//!   the moon's sheen, the pieces) is round the outline now.
//! * **A splash** where anyone goes into the sea, and a small one where some
//!   of the pieces do: a ring spreading on the water and drops thrown up out
//!   of it, gone within the break.
//! * **A burst of ice** where two meet in a slide, as big as they met hard:
//!   a flash spreading on the ice and a spray of it, with a shake of the
//!   camera (`hold_the_camera`).
//! * **Whoever won**, alone on the ice at the end: a ring of gold at their
//!   feet, beating, and a throw of every seat's colours and gold over them.
//!
//! Everything here hangs under one root, which alone carries the venue (so
//! that it is drawn on the ice and nowhere else, `island::show_where_i_am`
//! seeing to that), each piece shown or hidden here; and goes with the
//! table (`InTable`). Unlit, drawn brighter than its colours as the punch's
//! range is (`range::bright`), for the tonemapping greys a plain colour.

use std::collections::VecDeque;
use std::f32::consts::{PI, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use roundtown_net::knock_off::{self as rules, Floe, KnockStage};

use super::{Knock, KnockSystems, bodies_by_seat, seat_colour, winner_on_the_ice};
use crate::clock::TownClock;
use crate::island::{Islands, Land};
use crate::sky::Daylight;
use crate::table::{ComputerSeat, InTable, TableView};
use crate::ui::{UiChanged, ui};
use crate::{Dead, Player, Venue};

/// How long the camera shakes where two meet, in seconds.
pub(super) const SHAKE_SECS: f32 = 0.28;

/// How far over the ice what lies on it is drawn, in metres: clear of it
/// by enough that it never flickers through it from the camera's 30 m off,
/// and the rim a hair under the fill, so that the fill is drawn over it;
/// the streaks behind those sliding under the arrows and rings; and the
/// band of ice that will break and the moon's sheen under the cracks.
const ON_THE_ICE: f32 = 0.03;
const RIM_UNDER: f32 = 0.004;
const TRAIL_UP: f32 = 0.02;
const THIN_UP: f32 = 0.012;
const SHEEN_UP: f32 = 0.008;
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
/// Past the edge, in metres: how long each dash of the shaft is, and the
/// gap between two; and how wide each bar of the cross where it would stop
/// (mine, from a check of the game: a slide straight into the sea had been
/// drawn as a safe one, and was taken).
const DASH: f32 = 0.42;
const DASH_GAP: f32 = 0.3;
const CROSS_WIDE: f32 = 0.2;
/// The ring at everyone's feet: how far out its inside and outside are, in
/// metres.
const FEET_RING: (f32, f32) = (0.44, 0.56);
/// The streak behind whoever slides: how long it lasts behind them, in
/// seconds; how wide it is at its newest, in metres; how strong, at its
/// newest; and how fast they have to be going to leave one, in metres a
/// second.
const TRAIL_SECS: f32 = 0.4;
const TRAIL_WIDE: f32 = 0.55;
const TRAIL_ALPHA: f32 = 0.6;
const TRAIL_FROM: f32 = 1.2;
/// The cracks: how wide a line of them is, in metres; how far either way
/// across its line the ring of them wanders; and how long a crack running
/// out from it is, at the most.
const CRACK_WIDE: f32 = 0.07;
const CRACK_WANDER: f32 = 0.16;
const CRACK_OUT: f32 = 0.9;
/// The pieces the ice breaks into (mine; boxes all alike, all at once, had
/// read as the teeth of a cog, a check of the game found): how long each is
/// round the rim, in metres, thrown between these; the crack between two,
/// in metres; how far either way its broken inside wanders; how deep each
/// is; how fast they drop, in metres a second each second, and how far they
/// tip over as they go, in radians; how long the crack takes to run round
/// to the far side of the rim from where it began, in seconds, each piece
/// going as it reaches it; and how many of them splash, as a share.
const PIECE_LONG: (f32, f32) = (1.1, 3.4);
const PIECE_GAP: f32 = 0.07;
const PIECE_JAG: f32 = 0.14;
const SLAB_DEEP: (f32, f32) = (0.45, 0.9);
const SLAB_DROP: f32 = 12.0;
const SLAB_TIP: f32 = 1.3;
const SLAB_RUN: f32 = 0.42;
const SLAB_SPLASHES: f32 = 0.22;
/// A splash, for a body and for a piece of ice: how long it lasts, in
/// seconds; how far its ring spreads, in metres; how many drops it throws
/// up; and how strong its ring is at first. A piece's is small and quick,
/// all of them gone before the next aim (a ring of them had stood round the
/// island into it, brighter than anything by night).
const SPLASH_SECS: (f32, f32) = (0.9, 0.4);
const SPLASH_SPREAD: (f32, f32) = (2.3, 0.6);
const SPLASH_DROPS: (usize, usize) = (16, 4);
const SPLASH_ALPHA: (f32, f32) = (0.75, 0.4);
/// The burst where two meet: how long it lasts, in seconds; how far its
/// flash spreads, in metres, met softly and met as hard as two meet; how
/// many motes of ice it throws, likewise; and how hard is hard, in metres a
/// second.
const BURST_SECS: f32 = 0.55;
const BURST_SPREAD: (f32, f32) = (0.7, 2.4);
const BURST_MOTES: (f32, f32) = (6.0, 20.0);
const HARD: f32 = 20.0;
/// How much of the dust's and the drops' gravity is, in metres a second
/// each second.
const GRAVITY: f32 = 14.0;
/// How far under the water a piece of ice shrinks away to nothing, in
/// metres from where it begins to: the sea is see-through, and a piece
/// gone at once there would be seen to go.
const SINKS: (f32, f32) = (0.3, 1.4);
/// By night: how much dimmer the splashes and the dust are, all night, as a
/// share (unlit, they had been the brightest things on the screen); and how
/// strong the moon's sheen on the ice is.
const NIGHT_DIM: f32 = 0.55;
const SHEEN: f32 = 0.16;
/// Whoever won: how far out the ring of gold at their feet reaches, inside
/// and out, in metres; how far it beats either way, as a share, and how
/// fast, in radians a second; and how many bits of colour are thrown over
/// them, and for how long at the most, in seconds.
const HALO: (f32, f32) = (0.62, 0.86);
const HALO_BEAT: (f32, f32) = (0.14, 7.0);
const CONFETTI: usize = 54;
const CONFETTI_SECS: f32 = 2.8;

/// Someone, or a piece of ice (`big` false), going into the sea at `at`.
#[derive(Message, Clone, Copy, Debug)]
pub(super) struct Splashed {
    pub at: Vec3,
    pub big: bool,
}

/// Two meeting in a slide, the players in `seats`, at `at` on the ice,
/// `speed` metres a second the faster for it.
#[derive(Message, Clone, Copy, Debug)]
pub(super) struct Met {
    pub at: Vec3,
    pub speed: f32,
    pub seats: (usize, usize),
}

/// How far the camera shakes where two meet at `speed`, as a share of how
/// far it is from the ice (as much on the screen however near it has come):
/// from a fifth of a per cent for a nudge to a little over one for two who
/// meet as hard as they can, a third of a metre from 30 m off, a dozen
/// pixels on a desktop's screen.
pub(super) fn shake_for(speed: f32) -> f32 {
    (speed * 0.0006).clamp(0.002, 0.011)
}

/// The root everything drawn on the ice hangs under, while you are at a
/// table of Knock Off.
#[derive(Component)]
struct IceRoot;

/// The arrow of the player in a seat: what it was built for (how long, and
/// how far along it leaves the ice, if it does: its meshes are its own),
/// and its fill's and rim's meshes.
#[derive(Component)]
struct Arrow {
    seat: usize,
    built: Option<(f32, Option<f32>)>,
    fill: Handle<Mesh>,
    rim: Handle<Mesh>,
}

/// The ring at the feet of whoever is in a seat.
#[derive(Component)]
struct FeetRing(usize);

/// The streak behind whoever is in a seat as they slide, and its mesh.
#[derive(Component)]
struct Trail {
    seat: usize,
    mesh: Handle<Mesh>,
}

/// The cracks, and what they were drawn for: the turn, how big the ice
/// will be once it breaks, and the island.
#[derive(Component)]
struct Cracks {
    built: Option<(u16, f32, u64)>,
    mesh: Handle<Mesh>,
}

/// The band of ice that will break, and what it was built for: from how big
/// the ice will be to how big it is, and the island.
#[derive(Component)]
struct ThinIce {
    built: Option<(f32, f32, u64)>,
    mesh: Handle<Mesh>,
}

/// The moon on the ice that will stay, by night, and what it was built
/// for: how big that ice is, and the island.
#[derive(Component)]
struct Sheen {
    built: Option<(f32, u64)>,
    mesh: Handle<Mesh>,
}

/// The table's island's own ice, under the island's model (whose own disc
/// is put away), and which island it was shaped for.
#[derive(Component)]
struct IslandIce {
    island: u64,
}

/// How deep the island's ice goes under its top, in metres: as deep as
/// Hajun's disc was, its sides down into the sea.
const ICE_DEEP: f32 = rules::SUNK;

/// The ring of gold at the feet of whoever won.
#[derive(Component)]
struct Halo;

/// A piece of the ice's edge breaking off: where it began, which way is
/// out, when the ice began to break by the town's clock and how long after
/// it goes (by the clock, as the break is, and not by the frames, which a
/// slow one would have holding the pieces up into the next aim), and
/// whether it has splashed (or never will).
#[derive(Component)]
struct Slab {
    from: Vec3,
    out: Vec2,
    began: f64,
    wait: f32,
    splashed: bool,
}

/// A splash, or the burst where two meet: how old, how long it lasts, and
/// its ring's material, which fades from how strong it was at first.
#[derive(Component)]
struct Splash {
    age: f32,
    lasts: f32,
    ring: Handle<StandardMaterial>,
    strong: f32,
}

/// A splash's ring spreading on the water, or a burst's on the ice: how far
/// it spreads, and over how long.
#[derive(Component)]
struct Ripple {
    spread: f32,
    age: f32,
    lasts: f32,
}

/// One of a splash's drops, or of a burst's motes: how fast it goes, how
/// big it began, how old it is and how long it lasts.
#[derive(Component)]
struct Drop {
    velocity: Vec3,
    size: f32,
    age: f32,
    lasts: f32,
}

/// A bit of colour thrown over whoever won: how fast it goes, how it
/// spins (about its axis, radians a second), and how old it is.
#[derive(Component)]
struct Confetto {
    velocity: Vec3,
    spin: Vec3,
    age: f32,
}

/// What everything on the ice is made of: each seat's colour, the dark rim,
/// the cracks, the band of ice that will break and the moon's sheen, the
/// streaks' (coloured by each streak's own), the dust, the water's white,
/// gold, and, until the ice's own is found on its model, a pale ice for
/// the pieces it breaks into (the file's `slab`); and the shapes shared by
/// many: a ball, a box and flat rings.
#[derive(Resource)]
struct IceLook {
    seats: [Handle<StandardMaterial>; 4],
    rim: Handle<StandardMaterial>,
    cracks: Handle<StandardMaterial>,
    thin: Handle<StandardMaterial>,
    sheen: Handle<StandardMaterial>,
    trail: Handle<StandardMaterial>,
    dust: Handle<StandardMaterial>,
    white: Handle<StandardMaterial>,
    gold: Handle<StandardMaterial>,
    ice: Handle<StandardMaterial>,
    found_ice: bool,
    ball: Handle<Mesh>,
    slab: Handle<Mesh>,
    ring: Handle<Mesh>,
    feet_fill: Handle<Mesh>,
    feet_rim: Handle<Mesh>,
    halo: Handle<Mesh>,
    flash: Handle<Mesh>,
}

impl FromWorld for IceLook {
    fn from_world(world: &mut World) -> Self {
        let colours = ui().colours.knock_off;
        let gold = ui().colours.gold;
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let seats = [0, 1, 2, 3].map(|seat| materials.add(flat(seat_colour(seat))));
        let rim = materials.add(flat(colours.rim));
        let cracks = materials.add(flat(colours.cracks));
        let thin = materials.add(StandardMaterial {
            base_color: colours.thin_ice,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            fog_enabled: false,
            ..default()
        });
        let sheen = materials.add(StandardMaterial {
            base_color: Color::NONE,
            unlit: true,
            alpha_mode: AlphaMode::Add,
            fog_enabled: false,
            ..default()
        });
        // Coloured by its mesh, a seat's colour fading along it.
        let trail = materials.add(StandardMaterial {
            base_color: Color::linear_rgb(GAIN, GAIN, GAIN),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            fog_enabled: false,
            ..default()
        });
        let dust = materials.add(flat(colours.dust));
        let white = materials.add(flat(colours.splash));
        let gold = materials.add(flat(gold));
        let ice = materials.add(StandardMaterial {
            base_color: colours.slab,
            perceptual_roughness: 0.3,
            ..default()
        });
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let ball = meshes.add(Sphere::new(1.0).mesh().ico(1).expect("a ball"));
        let slab = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
        let ring = meshes.add(annulus(0.82, 1.0, 0.0));
        let feet_fill = meshes.add(annulus(FEET_RING.0, FEET_RING.1, 0.0));
        let feet_rim = meshes.add(rimmed_annulus(FEET_RING.0, FEET_RING.1));
        let halo = meshes.add(annulus(HALO.0, HALO.1, 0.0));
        let flash = meshes.add(annulus(0.5, 1.0, 0.0));
        Self {
            seats,
            rim,
            cracks,
            thin,
            sheen,
            trail,
            dust,
            white,
            gold,
            ice,
            found_ice: false,
            ball,
            slab,
            ring,
            feet_fill,
            feet_rim,
            halo,
            flash,
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
                shape_the_island.after(KnockSystems).before(crate::move_bodies),
                (find_the_ice, recolour, arrows, feet_rings, trails, cracks, thin_ice, break_off, splash, burst, celebrate)
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

/// How bright the splashes and the dust are by `daylight`: as their colours
/// by day, [`NIGHT_DIM`] dimmer all night.
fn by_night(daylight: &Daylight) -> f32 {
    1.0 - NIGHT_DIM * daylight.lamps.clamp(0.0, 1.0)
}

/// `colour` as bright as [`flat`] has it, `dim` of that.
fn dimmed(colour: Color, dim: f32) -> Color {
    let linear = bright(colour).to_linear();
    Color::linear_rgb(linear.red * dim, linear.green * dim, linear.blue * dim)
}

/// Puts up the root and everything that lives under it as you sit down at
/// a table of Knock Off (the table took down everything of the one
/// before): an arrow, a ring for the feet and a streak for each seat, the
/// cracks, the band of ice that will break, the moon's sheen and the ring
/// of gold, all hidden until wanted.
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
        let streak = meshes.add(annulus(0.1, 0.2, 0.0));
        commands.spawn((
            ChildOf(root),
            Trail { seat, mesh: streak.clone() },
            Mesh3d(streak),
            MeshMaterial3d(look.trail.clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
            NotShadowReceiver,
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
    let thin = meshes.add(annulus(1.0, 1.1, 0.0));
    commands.spawn((
        ChildOf(root),
        ThinIce {
            built: None,
            mesh: thin.clone(),
        },
        Mesh3d(thin),
        MeshMaterial3d(look.thin.clone()),
        Transform::from_xyz(0.0, THIN_UP, 0.0),
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
    ));
    let sheen = meshes.add(annulus(0.0, 1.0, 0.0));
    commands.spawn((
        ChildOf(root),
        Sheen {
            built: None,
            mesh: sheen.clone(),
        },
        Mesh3d(sheen),
        MeshMaterial3d(look.sheen.clone()),
        Transform::from_xyz(0.0, SHEEN_UP, 0.0),
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
    ));
    commands.spawn((
        ChildOf(root),
        Halo,
        Mesh3d(look.halo.clone()),
        MeshMaterial3d(look.gold.clone()),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

/// The ice's own material, off its model, for the pieces that break off
/// it and the island's own ice, once the model is there.
fn find_the_ice(
    mut look: ResMut<IceLook>,
    lands: Query<(Entity, &Venue), With<Land>>,
    children: Query<&Children>,
    materials: Query<&MeshMaterial3d<StandardMaterial>, Without<IslandIce>>,
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
/// desktop (`ui::reread`); and as the day turns to night and back, the
/// splashes and the dust dimmer by night, and the moon's sheen on the ice.
fn recolour(changed: Res<UiChanged>, daylight: Res<Daylight>, look: Res<IceLook>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let reread = changed.is_changed() && !changed.is_added();
    if !reread && !daylight.is_changed() {
        return;
    }
    let colours = ui().colours.knock_off;
    let dim = by_night(&daylight);
    let mut paint = |handle: &Handle<StandardMaterial>, colour: Color| {
        if let Some(mut material) = materials.get_mut(handle)
            && material.base_color != colour
        {
            material.base_color = colour;
        }
    };
    if reread {
        for seat in 0..4 {
            paint(&look.seats[seat], bright(seat_colour(seat)));
        }
        paint(&look.rim, bright(colours.rim));
        paint(&look.cracks, bright(colours.cracks));
        paint(&look.thin, colours.thin_ice);
        paint(&look.gold, bright(ui().colours.gold));
        // The pieces' own ice, lit as the island is, until the island's is
        // found (then theirs is the model's, which is not the file's to
        // paint).
        if !look.found_ice {
            paint(&look.ice, colours.slab);
        }
    }
    paint(&look.dust, dimmed(colours.dust, dim));
    paint(&look.white, dimmed(colours.splash, dim));
    let moon = colours.sheen.to_linear();
    let strength = SHEEN * daylight.lamps.clamp(0.0, 1.0);
    paint(&look.sheen, Color::linear_rgb(moon.red * strength, moon.green * strength, moon.blue * strength));
}

/// Your arrow while you aim, and everyone's while they are shown: from the
/// feet, as long as the slide would go on an empty island, the ring where
/// it would stop, or past the edge, dashes and a cross; put away as
/// everyone sets off. Each built again only as it changes.
fn arrows(
    knock: Res<Knock>,
    mut arrows: Query<(&mut Arrow, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let round = knock.playing();
    let sliding = knock.sliding();
    let floe = knock.floe.clone();
    for (mut arrow, mut place, mut shown) in &mut arrows {
        let seat = arrow.seat;
        let start = round.and_then(|round| round.start.get(seat).copied().flatten()).map(|(x, z)| Vec2::new(x, z));
        let in_play = round.is_some_and(|round| round.out.get(seat).is_some_and(Option::is_none));
        // Which way and how far: yours while you aim; anyone's once shown,
        // until the slide has begun here (`place`).
        let wanted = match (round, start) {
            (Some(round), Some(at)) if in_play => match round.stage {
                KnockStage::Aim if knock.seat == Some(seat as u8) && knock.aim.turn == round.turn => {
                    (knock.aim.power > 0 && knock.aim.way != Vec2::ZERO).then(|| (at, knock.aim.way, rules::reach(knock.aim.power), round.size))
                }
                KnockStage::Reveal if !sliding => {
                    let launch = round.launches.get(seat).map_or(Vec2::ZERO, |&(x, z)| Vec2::new(x, z));
                    let speed = launch.length();
                    (speed > 1e-3).then(|| (at, launch / speed, speed * speed / (2.0 * rules::ICE), round.size))
                }
                _ => None,
            },
            _ => None,
        };
        let (Some((at, way, reach, size)), Some(floe)) = (wanted, &floe) else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        // Where its middle first leaves the ice, if it would.
        let sea = floe.leaves(at, way, reach, size);
        let same = arrow.built.is_some_and(|(built, left)| {
            (built - reach).abs() <= 1e-3
                && match (left, sea) {
                    (None, None) => true,
                    (Some(left), Some(sea)) => (left - sea).abs() <= 0.02,
                    _ => false,
                }
        });
        if !same {
            let (fill, rim) = arrow_meshes(reach, sea);
            if meshes.insert(&arrow.fill, fill).is_err() || meshes.insert(&arrow.rim, rim).is_err() {
                warn!("Knock Off: an arrow has lost its mesh");
            }
            arrow.built = Some((reach, sea));
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

/// The streak behind whoever slides, in their colour: where they were over
/// the last [`TRAIL_SECS`], on the ice, narrowing and fading behind them;
/// built afresh each frame only while there is one to draw. By the time
/// that passes (`Time<Real>`), as every flourish here is, not the engine's
/// time of the frames, which it holds to a quarter of a second each: a
/// device that slows has them kept up with the clock the game goes by.
fn trails(
    time: Res<Time<Real>>,
    knock: Res<Knock>,
    mut trails: Query<(&Trail, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut kept: Local<(Option<u16>, Vec<VecDeque<(f32, Vec2)>>)>,
) {
    let now = time.elapsed_secs();
    let turn = knock.playing().map(|round| round.turn);
    if kept.0 != turn || kept.1.len() != 4 {
        *kept = (turn, vec![VecDeque::new(); 4]);
    }
    let played = knock.slide.as_ref().filter(|played| Some(played.turn) == turn);
    for (seat, points) in kept.1.iter_mut().enumerate() {
        let going = played.and_then(|played| played.slide.velocity(seat)).map_or(0.0, |velocity| velocity.length());
        if let Some(at) = knock.drawn.get(seat).copied().flatten().filter(|at| at.y > -0.05 && going > TRAIL_FROM) {
            points.push_back((now, at.xz()));
        }
        while points.front().is_some_and(|(when, _)| now - when > TRAIL_SECS) {
            points.pop_front();
        }
    }
    for (trail, mut shown) in &mut trails {
        let points = &kept.1[trail.seat];
        if points.len() < 2 {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        }
        if meshes.insert(&trail.mesh, trail_mesh(points, now, seat_colour(trail.seat))).is_err() {
            warn!("Knock Off: a streak has lost its mesh");
        }
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// The cracks where the ice will break at the end of the turn, while
/// everyone aims, looks and slides: built again for each turn.
fn cracks(knock: Res<Knock>, mut cracks: Query<(&mut Cracks, &mut Visibility)>, mut meshes: ResMut<Assets<Mesh>>) {
    let wanted = knock
        .playing()
        .filter(|round| matches!(round.stage, KnockStage::Aim | KnockStage::Reveal | KnockStage::Slide))
        .filter(|round| round.next_size < round.size - 1e-3)
        .map(|round| (round.turn, round.next_size, round.size))
        .zip(knock.floe.as_ref().zip(knock.island));
    for (mut cracked, mut shown) in &mut cracks {
        let Some(((turn, next, now), (floe, island))) = wanted else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        if cracked.built != Some((turn, next, island)) {
            if meshes.insert(&cracked.mesh, crack_mesh(floe, next, now, u64::from(turn) ^ island)).is_err() {
                warn!("Knock Off: the cracks have lost their mesh");
            }
            cracked.built = Some((turn, next, island));
        }
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// The band of ice that will break, darker, while everyone aims, looks and
/// slides (mine, from a check of the game: a line of cracks alone did not
/// say at a glance who stood on it, and Bridge Wars marks its doomed planks
/// too); and by night the moon on the ice that will stay. Each built again
/// only as it changes.
#[allow(clippy::type_complexity)]
fn thin_ice(
    knock: Res<Knock>,
    daylight: Res<Daylight>,
    mut bands: Query<(&mut ThinIce, &mut Visibility), Without<Sheen>>,
    mut sheens: Query<(&mut Sheen, &mut Visibility), Without<ThinIce>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let round = knock.playing();
    let island = knock.floe.as_ref().zip(knock.island);
    let doomed = round
        .filter(|round| matches!(round.stage, KnockStage::Aim | KnockStage::Reveal | KnockStage::Slide))
        .filter(|round| round.next_size < round.size - 1e-3)
        .map(|round| (round.next_size.max(0.0), round.size))
        .zip(island);
    for (mut band, mut shown) in &mut bands {
        let Some(((inside, edge), (floe, island))) = doomed else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        if band.built != Some((inside, edge, island)) {
            if meshes.insert(&band.mesh, band_mesh(floe, inside, edge)).is_err() {
                warn!("Knock Off: the thin ice has lost its mesh");
            }
            band.built = Some((inside, edge, island));
        }
        shown.set_if_neq(Visibility::Inherited);
    }
    // The ice that stays: all of it while the table waits, and from the
    // first turn on what will be left once this turn's edge has gone.
    let stays = round.map_or(1.0, |round| round.next_size);
    let lit = knock.seat.is_some() && daylight.lamps > 0.02 && stays > 0.005;
    for (mut sheen, mut shown) in &mut sheens {
        let Some((floe, island)) = island.filter(|_| lit) else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        if sheen.built != Some((stays, island)) {
            if meshes.insert(&sheen.mesh, band_mesh(floe, 0.0, stays)).is_err() {
                warn!("Knock Off: the sheen has lost its mesh");
            }
            sheen.built = Some((stays, island));
        }
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// The table's island, shaped as it comes ([`Knock::floe`]): its own ice
/// under the island's model, in the ice's own material, the model's disc
/// put away; and the shape everyone walks about on while the table waits,
/// in place of the disc's, the frame it comes, before anyone moves on it.
/// Away from the tables, the last island stays as it was, drawn nowhere.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn shape_the_island(
    knock: Res<Knock>,
    look: Res<IceLook>,
    lands: Query<(Entity, &Venue), With<Land>>,
    children: Query<&Children>,
    mut models: Query<&mut Visibility, (With<Mesh3d>, Without<IslandIce>)>,
    mut ices: Query<(Entity, &IslandIce, &mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut islands: ResMut<Islands>,
    mut commands: Commands,
) {
    let Some(land) = lands.iter().find(|(_, venue)| **venue == Venue::KnockOff).map(|(land, _)| land) else {
        return;
    };
    // The model's own disc, put away for good once there is an island.
    let shaped = !ices.is_empty() || knock.island.is_some();
    if shaped {
        for part in children.iter_descendants(land) {
            if let Ok(mut shown) = models.get_mut(part) {
                shown.set_if_neq(Visibility::Hidden);
            }
        }
    }
    for (_, _, _, mut material) in &mut ices {
        if material.0 != look.ice {
            material.0 = look.ice.clone();
        }
    }
    let (Some(floe), Some(island)) = (&knock.floe, knock.island) else {
        return;
    };
    if ices.iter().any(|(_, ice, ..)| ice.island == island) {
        return;
    }
    let mesh = meshes.add(island_mesh(floe));
    match ices.iter_mut().next() {
        Some((entity, _, mut drawn, _)) => {
            drawn.0 = mesh;
            commands.entity(entity).insert(IslandIce { island });
        }
        None => {
            commands.spawn((
                ChildOf(land),
                IslandIce { island },
                Mesh3d(mesh),
                MeshMaterial3d(look.ice.clone()),
                Transform::default(),
                Visibility::Inherited,
                Name::new("Knock Off: the table's island"),
            ));
        }
    }
    islands.reshape(Venue::KnockOff, island_triangles(floe), &[]);
    info!("Knock Off: the island shaped, {:#x}", island);
}

/// Every triangle of the island's ice at its whole size, across the ground
/// round the middle of the world, its top at 0 and [`ICE_DEEP`] deep: its
/// top, facing up, a fan from the middle it was thrown round (every point
/// of its outline can be seen from there); its bottom, facing down; and its
/// sides, facing out.
fn island_triangles(floe: &Floe) -> Vec<[Vec3; 3]> {
    let rim = floe.rim();
    let count = rim.len();
    let (top, bottom) = (|at: Vec2| Vec3::new(at.x, 0.0, at.y), |at: Vec2| Vec3::new(at.x, -ICE_DEEP, at.y));
    let mut corners = Vec::with_capacity(count * 4);
    for i in 0..count {
        let (a, b) = (rim[i], rim[(i + 1) % count]);
        corners.push([Vec3::ZERO, top(b), top(a)]);
        corners.push([Vec3::new(0.0, -ICE_DEEP, 0.0), bottom(a), bottom(b)]);
        corners.push([top(a), top(b), bottom(a)]);
        corners.push([top(b), bottom(b), bottom(a)]);
    }
    corners
}

/// The island's ice as it is drawn: its top, flat, lit as the ice is;
/// its sides, smooth round the outline; and its bottom. Its colour is its
/// material's, the model's own ice.
fn island_mesh(floe: &Floe) -> Mesh {
    let rim = floe.rim();
    let count = rim.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(count * 4 + 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(count * 4 + 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(count * 4 + 2);
    let mut indices: Vec<u32> = Vec::with_capacity(count * 12);
    let across = 2.0 * floe.furthest().max(1.0);
    let uv = |at: Vec2| [at.x / across + 0.5, at.y / across + 0.5];
    // The top: the middle, and every point of the outline.
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    for &at in rim {
        positions.push([at.x, 0.0, at.y]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push(uv(at));
    }
    for i in 0..count as u32 {
        let (a, b) = (1 + i, 1 + (i + 1) % count as u32);
        indices.extend([0, b, a]);
    }
    // The sides: each point of the outline twice, at the top and down at
    // the bottom, facing out from both sides it is between.
    let sides = positions.len() as u32;
    for i in 0..count {
        let (before, after) = (rim[(i + count - 1) % count], rim[(i + 1) % count]);
        let out = |from: Vec2, to: Vec2| {
            let along = (to - from).normalize_or(Vec2::X);
            Vec2::new(along.y, -along.x)
        };
        let facing = (out(before, rim[i]) + out(rim[i], after)).normalize_or(rim[i].normalize_or(Vec2::X));
        for y in [0.0, -ICE_DEEP] {
            positions.push([rim[i].x, y, rim[i].y]);
            normals.push([facing.x, 0.0, facing.y]);
            uvs.push(uv(rim[i]));
        }
    }
    for i in 0..count as u32 {
        let j = (i + 1) % count as u32;
        let (top_i, bottom_i, top_j, bottom_j) = (sides + 2 * i, sides + 2 * i + 1, sides + 2 * j, sides + 2 * j + 1);
        indices.extend([top_i, top_j, bottom_i, top_j, bottom_j, bottom_i]);
    }
    // The bottom.
    let under = positions.len() as u32;
    positions.push([0.0, -ICE_DEEP, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    for &at in rim {
        positions.push([at.x, -ICE_DEEP, at.y]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push(uv(at));
    }
    for i in 0..count as u32 {
        let (a, b) = (under + 1 + i, under + 1 + (i + 1) % count as u32);
        indices.extend([under, a, b]);
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

/// The ice breaking at the end of a turn: the island's model narrowed to
/// the ice that is left (and whole again away from the game), and as it
/// begins to break, the band where the edge was in pieces of every size,
/// the crack running round the rim both ways from where it began, each
/// piece tipping out and dropping into the sea as it reaches it, a few of
/// them with a splash.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn break_off(
    clock: Res<TownClock>,
    knock: Res<Knock>,
    look: Res<IceLook>,
    roots: Query<Entity, With<IceRoot>>,
    mut lands: Query<(&Venue, &mut Transform), (With<Land>, Without<Slab>)>,
    mut slabs: Query<(Entity, &mut Slab, &mut Transform), Without<Land>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut splashes: MessageWriter<Splashed>,
    mut broke: Local<Option<u16>>,
    mut commands: Commands,
) {
    let round = knock.playing();
    let size = match round {
        Some(round) if matches!(round.stage, KnockStage::Break | KnockStage::Over) => round.next_size,
        Some(round) => round.size,
        None => 1.0,
    };
    let across = size.max(0.02);
    for (venue, mut land) in &mut lands {
        if *venue == Venue::KnockOff {
            let scale = Vec3::new(across, 1.0, across);
            if land.scale != scale {
                land.scale = scale;
            }
        }
    }
    // As it begins to break: the pieces, once a turn.
    if let Some(round) = round.filter(|round| round.stage == KnockStage::Break)
        && *broke != Some(round.turn)
        && let Ok(root) = roots.single()
        && let Some(floe) = &knock.floe
    {
        *broke = Some(round.turn);
        let (outer, inner) = (round.size, round.next_size.max(0.0));
        let mut dice = fastrand::Rng::with_seed(u64::from(round.turn) * 7919);
        let began = dice.f32() * TAU;
        for (from, to) in pieces_round(floe, (outer + inner) * 0.5, began, &mut dice) {
            let out = floe.reach_toward(Vec2::from_angle((from + to) * 0.5)) * outer;
            let gap = (PIECE_GAP * 0.5 / out.max(0.3)).min((to - from) * 0.2);
            let (from, to) = (from + gap, to - gap);
            let deep = SLAB_DEEP.0 + (SLAB_DEEP.1 - SLAB_DEEP.0) * dice.f32();
            let (mesh, centre) = piece_mesh(floe, inner, outer, from, to, deep, &mut dice);
            let half_way = (from + to) * 0.5;
            // How far round the rim from where the crack began: it gets
            // there in its time, and the piece goes.
            let round_from = (half_way - began).rem_euclid(TAU);
            let round_from = round_from.min(TAU - round_from);
            let wait = SLAB_RUN * round_from / PI + dice.f32() * 0.06;
            let at = Vec3::new(centre.x, -0.004 - dice.f32() * 0.02, centre.y);
            commands.spawn((
                ChildOf(root),
                Slab {
                    from: at,
                    out: Vec2::from_angle(half_way),
                    began: round.since,
                    wait,
                    splashed: dice.f32() >= SLAB_SPLASHES,
                },
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(look.ice.clone()),
                Transform::from_translation(at),
                NotShadowCaster,
            ));
        }
    }
    // Away from a game being played, the next game's first break is its own.
    if round.is_none() {
        *broke = None;
    }
    // Each piece tips out and drops, and shrinks away under the sea.
    let now = clock.now();
    for (slab, mut thing, mut place) in &mut slabs {
        let going = ((now - thing.began) as f32 - thing.wait).max(0.0);
        let drop = 0.5 * SLAB_DROP * going * going;
        let under = -(thing.from.y - drop) + crate::WATER_Y;
        let left = 1.0 - ((under - SINKS.0) / (SINKS.1 - SINKS.0)).clamp(0.0, 1.0);
        if left <= 0.0 || going > 2.0 || round.is_none() {
            commands.entity(slab).despawn();
            continue;
        }
        let tip = (going * 1.8).min(1.0) * SLAB_TIP;
        let at = thing.from + Vec3::new(thing.out.x, 0.0, thing.out.y) * going * 1.2 - Vec3::Y * drop;
        let axis = Vec3::new(-thing.out.y, 0.0, thing.out.x);
        place.set_if_neq(
            Transform::from_translation(at)
                .with_rotation(Quat::from_axis_angle(axis, -tip))
                .with_scale(Vec3::splat(left)),
        );
        if !thing.splashed && at.y < crate::WATER_Y - 0.3 {
            thing.splashed = true;
            splashes.write(Splashed {
                at: Vec3::new(at.x, crate::WATER_Y, at.z),
                big: false,
            });
        }
    }
}

/// The pieces the band of `floe` round its outline at `middle` (a share of
/// the island, between the ice that breaks and what is left) breaks into:
/// from where to where round it each one is, in radians from +x toward +z,
/// all the way round from `began`, where the crack began; each its own
/// length along the band ([`PIECE_LONG`]), as far round the island as that
/// is where the band is (further round where the outline comes in close,
/// nearer where it reaches out), and the last never a sliver.
fn pieces_round(floe: &Floe, middle: f32, began: f32, dice: &mut fastrand::Rng) -> Vec<(f32, f32)> {
    let out = |angle: f32| (floe.reach_toward(Vec2::from_angle(angle)) * middle).max(0.3);
    let mut pieces = Vec::new();
    let mut from = 0.0;
    while from < TAU - 1e-3 {
        let long = PIECE_LONG.0 + (PIECE_LONG.1 - PIECE_LONG.0) * dice.f32();
        let mut to = from + (long / out(began + from)).min(TAU * 0.4);
        if TAU - to < 0.4 * PIECE_LONG.0 / out(began + to.min(TAU)) {
            to = TAU;
        }
        let to = to.min(TAU);
        pieces.push((began + from, began + to));
        from = to;
    }
    pieces
}

/// A splash where anything goes into the sea: a ring spreading out on the
/// water, fading as it goes, and drops thrown up out of it.
fn splash(
    mut splashed: MessageReader<Splashed>,
    roots: Query<Entity, With<IceRoot>>,
    look: Res<IceLook>,
    daylight: Res<Daylight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok(root) = roots.single() else {
        splashed.clear();
        return;
    };
    let white = dimmed(ui().colours.knock_off.splash, by_night(&daylight));
    for one in splashed.read() {
        let (spread, drops, lasts, strong) = if one.big {
            (SPLASH_SPREAD.0, SPLASH_DROPS.0, SPLASH_SECS.0, SPLASH_ALPHA.0)
        } else {
            (SPLASH_SPREAD.1, SPLASH_DROPS.1, SPLASH_SECS.1, SPLASH_ALPHA.1)
        };
        let ring = materials.add(StandardMaterial {
            base_color: white.with_alpha(strong),
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
                    lasts,
                    ring: ring.clone(),
                    strong,
                },
                Transform::from_translation(one.at + Vec3::Y * 0.05),
                Visibility::default(),
            ))
            .id();
        commands.spawn((
            ChildOf(splash),
            Ripple { spread, age: 0.0, lasts },
            Mesh3d(look.ring.clone()),
            MeshMaterial3d(ring),
            Transform::from_scale(Vec3::splat(0.3)),
            NotShadowCaster,
            NotShadowReceiver,
        ));
        let mut dice = fastrand::Rng::with_seed((one.at.x.to_bits() as u64) << 32 | u64::from(one.at.z.to_bits()));
        for k in 0..drops {
            let angle = (k as f32 + dice.f32()) * TAU / drops as f32;
            let out = Vec2::from_angle(angle) * (0.6 + dice.f32() * 2.0) * spread * 0.5;
            let up = if one.big { 4.0 + dice.f32() * 3.5 } else { 2.0 + dice.f32() * 1.6 };
            let size = (0.06 + dice.f32() * 0.08) * if one.big { 1.0 } else { 0.7 };
            commands.spawn((
                ChildOf(splash),
                Drop {
                    velocity: Vec3::new(out.x, up, out.y),
                    size,
                    age: 0.0,
                    lasts,
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

/// A burst of ice where two meet, as big as they met hard (mine, from a
/// check of the game: a few motes of dust had read as dirt, and a hit is
/// what every aim is for): a flash spreading on the ice, and a spray of
/// motes thrown out low over it, shrinking away.
fn burst(
    mut met: MessageReader<Met>,
    roots: Query<Entity, With<IceRoot>>,
    look: Res<IceLook>,
    daylight: Res<Daylight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Ok(root) = roots.single() else {
        met.clear();
        return;
    };
    let white = dimmed(ui().colours.knock_off.splash, by_night(&daylight));
    for one in met.read() {
        let hard = (one.speed / HARD).clamp(0.0, 1.0);
        let flash = materials.add(StandardMaterial {
            base_color: white.with_alpha(0.9),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            fog_enabled: false,
            ..default()
        });
        let burst = commands
            .spawn((
                ChildOf(root),
                Splash {
                    age: 0.0,
                    lasts: BURST_SECS,
                    ring: flash.clone(),
                    strong: 0.95,
                },
                Transform::from_translation(one.at + Vec3::Y * (ON_THE_ICE + 0.01)),
                Visibility::default(),
            ))
            .id();
        commands.spawn((
            ChildOf(burst),
            Ripple {
                spread: BURST_SPREAD.0 + (BURST_SPREAD.1 - BURST_SPREAD.0) * hard,
                age: 0.0,
                lasts: BURST_SECS * 0.5,
            },
            Mesh3d(look.flash.clone()),
            MeshMaterial3d(flash),
            Transform::from_scale(Vec3::splat(0.3)),
            NotShadowCaster,
            NotShadowReceiver,
        ));
        // The spray: white, and the two who met's own colours, so that it
        // is seen on white ice whose it was.
        let mut dice = fastrand::Rng::with_seed(u64::from(one.at.x.to_bits()) ^ u64::from(one.at.z.to_bits()).rotate_left(17));
        let motes = (BURST_MOTES.0 + (BURST_MOTES.1 - BURST_MOTES.0) * hard).round() as usize;
        for k in 0..motes {
            let angle = (k as f32 + dice.f32()) * TAU / motes as f32;
            let out = Vec2::from_angle(angle) * (1.8 + dice.f32() * 2.6) * (0.7 + 0.8 * hard);
            let size = (0.1 + dice.f32() * 0.12) * (0.8 + 0.6 * hard);
            let material = match k % 3 {
                0 => look.dust.clone(),
                1 => look.seats[one.seats.0 % 4].clone(),
                _ => look.seats[one.seats.1 % 4].clone(),
            };
            commands.spawn((
                ChildOf(burst),
                Drop {
                    velocity: Vec3::new(out.x, 1.4 + dice.f32() * 2.4 * (0.6 + hard), out.y),
                    size,
                    age: 0.0,
                    lasts: BURST_SECS,
                },
                Transform::from_xyz(0.0, 0.3, 0.0).with_scale(Vec3::splat(size)),
                Mesh3d(look.ball.clone()),
                MeshMaterial3d(material),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    }
}

/// Whoever won, alone on the ice at the end: the ring of gold at their feet,
/// beating; and once, as it is over, a throw of every seat's colours and
/// gold over them, the bits spinning and drifting down.
#[allow(clippy::type_complexity)]
fn celebrate(
    time: Res<Time<Real>>,
    knock: Res<Knock>,
    look: Res<IceLook>,
    roots: Query<Entity, With<IceRoot>>,
    mut halos: Query<(&mut Transform, &mut Visibility), (With<Halo>, Without<Confetto>)>,
    mut bits: Query<(Entity, &mut Confetto, &mut Transform), Without<Halo>>,
    mut thrown: Local<Option<u16>>,
    mut commands: Commands,
) {
    let round = knock.playing();
    let won = round.and_then(winner_on_the_ice);
    for (mut place, mut shown) in &mut halos {
        let Some((_, at)) = won else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let beat = 1.0 + HALO_BEAT.0 * (time.elapsed_secs() * HALO_BEAT.1).sin();
        place.set_if_neq(Transform::from_xyz(at.x, ON_THE_ICE + 0.004, at.y).with_scale(Vec3::new(beat, 1.0, beat)));
        shown.set_if_neq(Visibility::Inherited);
    }
    if let (Some((seat, at)), Some(round), Ok(root)) = (won, round, roots.single())
        && *thrown != Some(round.turn)
    {
        *thrown = Some(round.turn);
        let mut dice = fastrand::Rng::with_seed(u64::from(round.turn) * 31 + seat as u64);
        for k in 0..CONFETTI {
            let angle = dice.f32() * TAU;
            let out = Vec2::from_angle(angle) * (0.8 + dice.f32() * 2.6);
            let material = match k % 5 {
                4 => look.gold.clone(),
                colour => look.seats[(seat + colour) % 4].clone(),
            };
            let spin = Vec3::new(dice.f32() - 0.5, dice.f32() - 0.5, dice.f32() - 0.5).normalize_or(Vec3::Y) * (6.0 + dice.f32() * 10.0);
            commands.spawn((
                ChildOf(root),
                Confetto {
                    velocity: Vec3::new(out.x, 4.0 + dice.f32() * 4.0, out.y),
                    spin,
                    age: dice.f32() * 0.4,
                },
                Mesh3d(look.slab.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(at.x, 1.4, at.y).with_scale(Vec3::new(0.16, 0.02, 0.1)),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    }
    if won.is_none() {
        *thrown = None;
    }
    let dt = time.delta_secs();
    for (bit, mut thing, mut place) in &mut bits {
        thing.age += dt;
        if thing.age > CONFETTI_SECS || place.translation.y < crate::WATER_Y || round.is_none() {
            commands.entity(bit).despawn();
            continue;
        }
        // Light, as paper is: it slows in the air and drifts down.
        thing.velocity.y -= 7.0 * dt;
        let slowed = thing.velocity * (1.0 - 1.6 * dt).max(0.0);
        thing.velocity = slowed;
        place.translation += slowed * dt;
        let turned = Quat::from_scaled_axis(thing.spin * dt);
        place.rotation = turned * place.rotation;
    }
}

/// The splashes' and bursts' rings spread and fade, and every drop and mote
/// flies, falls and shrinks away; each gone at its end.
#[allow(clippy::type_complexity)]
fn drip(
    time: Res<Time<Real>>,
    daylight: Res<Daylight>,
    mut splashes: Query<(Entity, &mut Splash)>,
    mut ripples: Query<(&mut Ripple, &mut Transform), Without<Drop>>,
    mut drops: Query<(&mut Drop, &mut Transform), Without<Ripple>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let white = dimmed(ui().colours.knock_off.splash, by_night(&daylight));
    for (entity, mut splash) in &mut splashes {
        splash.age += dt;
        let share = splash.age / splash.lasts;
        if share >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(mut ring) = materials.get_mut(&splash.ring) {
            ring.base_color = white.with_alpha((1.0 - share) * splash.strong);
        }
    }
    for (mut ripple, mut place) in &mut ripples {
        ripple.age += dt;
        let share = (ripple.age / ripple.lasts).min(1.0);
        place.set_if_neq(Transform::from_scale(Vec3::splat(0.3 + share.sqrt() * ripple.spread)));
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
/// the feet: its fill and the dark rim just outside all of it. On the ice
/// (`sea` none): the shaft and head as one, and the ring where it stops,
/// `reach` out. Leaving it `sea` metres out: the shaft whole to there and
/// in dashes past it, the head, and a cross where the ring would be.
fn arrow_meshes(reach: f32, sea: Option<f32>) -> (Mesh, Mesh) {
    let ring_out = rules::BODY;
    let ring_in = ring_out - RING_WIDE;
    let tip = (reach - ring_out).max(ARROW_FROM + 0.2);
    let base = (tip - HEAD_LONG).max(ARROW_FROM + 0.05);
    let head_long = tip - base;
    let head_half = HEAD_HALF.min(head_long * 0.9).max(SHAFT_HALF + 0.08);
    let stop = Vec2::new(0.0, -reach);
    let mut fill = Sheet::default();
    let mut rim = Sheet::default();
    match sea {
        None => {
            let outline = [
                Vec2::new(-SHAFT_HALF, -ARROW_FROM),
                Vec2::new(-SHAFT_HALF, -base),
                Vec2::new(-head_half, -base),
                Vec2::new(0.0, -tip),
                Vec2::new(head_half, -base),
                Vec2::new(SHAFT_HALF, -base),
                Vec2::new(SHAFT_HALF, -ARROW_FROM),
            ];
            // The shaft, and the head.
            fill.quad(outline[0], outline[1], outline[5], outline[6]);
            fill.triangle(outline[2], outline[3], outline[4]);
            rim.band(&outline, RIM);
            // The ring where it stops, with its rim inside and out.
            fill.ring(stop, ring_in, ring_out);
            rim.ring(stop, ring_out, ring_out + RIM);
            rim.ring(stop, (ring_in - RIM).max(0.0), ring_in);
        }
        Some(edge) => {
            let shaft = |from: f32, to: f32| {
                [
                    Vec2::new(-SHAFT_HALF, -from),
                    Vec2::new(-SHAFT_HALF, -to),
                    Vec2::new(SHAFT_HALF, -to),
                    Vec2::new(SHAFT_HALF, -from),
                ]
            };
            let whole = edge.clamp(ARROW_FROM, base);
            if whole > ARROW_FROM + 0.02 {
                let piece = shaft(ARROW_FROM, whole);
                fill.quad(piece[0], piece[1], piece[2], piece[3]);
                rim.band(&piece, RIM);
            }
            let mut from = if whole > ARROW_FROM + 0.02 { whole + DASH_GAP } else { ARROW_FROM };
            while from < base - 0.04 {
                let to = (from + DASH).min(base);
                let piece = shaft(from, to);
                fill.quad(piece[0], piece[1], piece[2], piece[3]);
                rim.band(&piece, RIM);
                from = to + DASH_GAP;
            }
            let head = [Vec2::new(-head_half, -base), Vec2::new(0.0, -tip), Vec2::new(head_half, -base)];
            fill.triangle(head[0], head[1], head[2]);
            rim.band(&head, RIM);
            for turn in [PI * 0.25, -PI * 0.25] {
                let along = Vec2::from_angle(turn) * ring_out;
                let across = along.perp().normalize_or(Vec2::X) * CROSS_WIDE * 0.5;
                let bar = [stop - along - across, stop + along - across, stop + along + across, stop - along + across];
                fill.quad(bar[0], bar[1], bar[2], bar[3]);
                rim.band(&bar, RIM);
            }
        }
    }
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

/// The streak behind a body that slides: through `points` (when, and where
/// across the ice), oldest first, as wide as [`TRAIL_WIDE`] and as strong
/// as [`TRAIL_ALPHA`] at its newest, narrowing and fading to nothing
/// [`TRAIL_SECS`] behind `now`, in `colour`. Coloured by its corners
/// (`ATTRIBUTE_COLOR`), its material white.
fn trail_mesh(points: &VecDeque<(f32, Vec2)>, now: f32, colour: Color) -> Mesh {
    let linear = colour.to_linear();
    let count = points.len();
    let mut positions = Vec::with_capacity(count * 2);
    let mut colours = Vec::with_capacity(count * 2);
    for k in 0..count {
        let (when, at) = points[k];
        let before = points[k.saturating_sub(1)].1;
        let after = points[(k + 1).min(count - 1)].1;
        let across = (after - before).normalize_or(Vec2::X).perp();
        let fresh = (1.0 - (now - when) / TRAIL_SECS).clamp(0.0, 1.0);
        let half = TRAIL_WIDE * 0.5 * fresh.sqrt();
        for side in [-1.0f32, 1.0] {
            let corner = at + across * half * side;
            positions.push([corner.x, TRAIL_UP, corner.y]);
            colours.push([linear.red, linear.green, linear.blue, TRAIL_ALPHA * fresh * fresh]);
        }
    }
    let mut indices = Vec::with_capacity(count.saturating_sub(1) * 6);
    for k in 0..count.saturating_sub(1) as u32 {
        let (a, b, c, d) = (2 * k, 2 * k + 1, 2 * k + 2, 2 * k + 3);
        indices.extend([a, b, c, b, d, c]);
    }
    let corners = positions.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; corners])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; corners])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
}

/// A piece of the ice's edge broken off: the band from `inner` to `outer`
/// out, from `from` to `to` round (radians), `deep` deep, its broken inside
/// wandering a little either way of its line, thrown from `dice`; built
/// round its own middle, which is returned (across the ground) for it to
/// be put there and to tip about.
fn piece_mesh(floe: &Floe, inner: f32, outer: f32, from: f32, to: f32, deep: f32, dice: &mut fastrand::Rng) -> (Mesh, Vec2) {
    let reach = |angle: f32| floe.reach_toward(Vec2::from_angle(angle));
    let steps = (((to - from) * reach((from + to) * 0.5) * outer / 0.6).ceil() as usize).clamp(2, 12);
    let angles: Vec<f32> = (0..=steps).map(|k| from + (to - from) * k as f32 / steps as f32).collect();
    let outside: Vec<Vec2> = angles.iter().map(|&angle| Vec2::from_angle(angle) * reach(angle) * outer).collect();
    let inside: Vec<Vec2> = angles
        .iter()
        .map(|&angle| {
            let (out, edge) = (reach(angle) * inner, reach(angle) * outer);
            let jag = if out > 0.3 { (dice.f32() - 0.5) * 2.0 * PIECE_JAG } else { 0.0 };
            Vec2::from_angle(angle) * (out + jag).clamp(0.0, (edge - 0.1).max(0.0))
        })
        .collect();
    let middle = outside.iter().chain(&inside).copied().sum::<Vec2>() / (2 * (steps + 1)) as f32;
    let at = |point: Vec2, y: f32| Vec3::new(point.x - middle.x, y, point.y - middle.y);
    let flat = |way: Vec2| Vec3::new(way.x, 0.0, way.y);
    let mut solid = Solid::default();
    for k in 0..steps {
        let (o0, o1, i0, i1) = (outside[k], outside[k + 1], inside[k], inside[k + 1]);
        let out = Vec2::from_angle((angles[k] + angles[k + 1]) * 0.5);
        // Its top and its bottom, the rim it had, and the crack it broke
        // along.
        solid.face([at(o0, 0.0), at(o1, 0.0), at(i1, 0.0), at(i0, 0.0)], Vec3::Y);
        solid.face([at(o0, -deep), at(o1, -deep), at(i1, -deep), at(i0, -deep)], Vec3::NEG_Y);
        solid.face([at(o0, 0.0), at(o1, 0.0), at(o1, -deep), at(o0, -deep)], flat(out));
        solid.face([at(i0, 0.0), at(i1, 0.0), at(i1, -deep), at(i0, -deep)], flat(-out));
    }
    // Its two ends, where it broke from the pieces either side.
    for (k, side) in [(0, -1.0), (steps, 1.0)] {
        let along = Vec2::from_angle(angles[k]).perp() * side;
        solid.face([at(inside[k], 0.0), at(outside[k], 0.0), at(outside[k], -deep), at(inside[k], -deep)], flat(along));
    }
    (solid.mesh(), middle)
}

/// The cracks round `floe`'s ice where it will break, its outline at
/// `next` (a share of the island) on ice at `size`: a ring of them along it
/// that wanders a little either side of the line, and short ones running
/// out from it toward the edge, thrown from `seed` (the turn's and the
/// island's) so that each turn's are their own; on ice that will be gone,
/// cracks running out from the middle.
fn crack_mesh(floe: &Floe, next: f32, size: f32, seed: u64) -> Mesh {
    let mut dice = fastrand::Rng::with_seed(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x6372_6163);
    let mut sheet = Sheet::default();
    let reach = |way: Vec2| floe.reach_toward(way);
    if floe.deepest(next) > 0.15 {
        // Along the outline, a point every 0.45 m or so, each wandering.
        let line: Vec<Vec2> = floe.rim_at(next).collect();
        let mut points = Vec::new();
        for k in 0..line.len() {
            let (a, b) = (line[k], line[(k + 1) % line.len()]);
            let pieces = (a.distance(b) / 0.45).ceil().max(1.0) as usize;
            for piece in 0..pieces {
                points.push(a.lerp(b, piece as f32 / pieces as f32));
            }
        }
        let wander = CRACK_WANDER.min(0.25 * floe.deepest(next));
        let points: Vec<Vec2> = points.into_iter().map(|at| at + at.normalize_or_zero() * (dice.f32() - 0.5) * 2.0 * wander).collect();
        let steps = points.len().max(3);
        if points.len() >= 3 {
            for k in 0..steps {
                sheet.line(points[k], points[(k + 1) % steps], CRACK_WIDE);
            }
        }
        // Short cracks out from it, toward the edge.
        let runs = (steps / 4).max(4);
        for _ in 0..runs {
            let Some(&from) = points.get(dice.usize(..points.len().max(1))) else {
                break;
            };
            let out = from.normalize_or(Vec2::X);
            let room = (reach(out) * size - from.length()).max(0.0);
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
            let angle = (k as f32 + dice.f32() * 0.6) * TAU / 7.0;
            let way = Vec2::from_angle(angle);
            let out = way * reach(way) * size * 0.9;
            let middle = out * 0.5 + out.perp() * (dice.f32() - 0.5) * 0.3;
            sheet.line(Vec2::ZERO, middle, CRACK_WIDE);
            sheet.line(middle, out, CRACK_WIDE * 0.7);
        }
    }
    sheet.mesh()
}

/// The band of `floe`'s ice from its outline at `inner` to its outline at
/// `outer` (shares of the island), lying on it: the ice that will break
/// off, or, from none, all of the ice at `outer`.
fn band_mesh(floe: &Floe, inner: f32, outer: f32) -> Mesh {
    let mut sheet = Sheet::default();
    let rim = floe.rim();
    for k in 0..rim.len() {
        let (a, b) = (rim[k], rim[(k + 1) % rim.len()]);
        if inner > 0.0 {
            sheet.quad(a * inner, a * outer, b * outer, b * inner);
        } else {
            sheet.triangle(Vec2::ZERO, a * outer, b * outer);
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
                Vec2::from_angle(k as f32 / steps as f32 * TAU),
                Vec2::from_angle((k + 1) as f32 / steps as f32 * TAU),
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

/// A solid of flat faces, each its own corners facing its own way: the
/// pieces the ice breaks into.
#[derive(Default)]
struct Solid {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

impl Solid {
    /// The flat four-sided face with `corners` in order round it, facing
    /// `out`, whichever way round they go.
    fn face(&mut self, corners: [Vec3; 4], out: Vec3) {
        let first = self.positions.len() as u32;
        let normal = out.normalize_or(Vec3::Y);
        for corner in corners {
            self.positions.push(corner.to_array());
            self.normals.push(normal.to_array());
        }
        // Anticlockwise seen from the side it faces, which is its front.
        let turning = (corners[1] - corners[0]).cross(corners[2] - corners[0]) + (corners[2] - corners[0]).cross(corners[3] - corners[0]);
        if turning.dot(normal) >= 0.0 {
            self.indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        } else {
            self.indices.extend([first, first + 2, first + 1, first, first + 3, first + 2]);
        }
    }

    fn mesh(self) -> Mesh {
        let count = self.positions.len();
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
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
            for sea in [None, Some(reach * 0.5), Some(0.2)] {
                let (fill, rim) = arrow_meshes(reach, sea);
                for mesh in [&fill, &rim] {
                    let all = triangles(mesh);
                    assert!(!all.is_empty(), "never a mesh with nothing in it");
                    for [a, b, c] in all {
                        // Facing up: the corners go round the other way than
                        // on a page, in (x, z).
                        assert!((b - a).perp_dot(c - a) <= 1e-6, "power {power}, {sea:?}");
                    }
                }
                // The ring's far side, or the cross's, is the slide's end and a
                // body's reach, or within it.
                let furthest = triangles(&fill).iter().flatten().map(|at| -at.y).fold(0.0f32, f32::max);
                assert!(furthest <= reach + rules::BODY + 0.02, "power {power}: {furthest}");
                if sea.is_none() {
                    assert!((furthest - (reach + rules::BODY)).abs() < 0.02, "power {power}: {furthest}");
                } else {
                    assert!(furthest > reach + rules::BODY * 0.6, "power {power}: the cross where it stops");
                }
                // The head's tip short of the ring.
                assert!(triangles(&fill).iter().flatten().all(|at| at.y <= 0.0));
            }
        }
        // Past the edge, the shaft is in dashes: gaps in it along the way.
        let (dashed, _) = arrow_meshes(rules::reach(10), Some(3.0));
        let (whole, _) = arrow_meshes(rules::reach(10), None);
        // Whether the point `along` the middle of the arrow is in any of its
        // triangles.
        let covered = |mesh: &Mesh, along: f32| {
            let at = Vec2::new(0.0, -along);
            triangles(mesh).iter().any(|&[a, b, c]| {
                let sides = [(b - a).perp_dot(at - a), (c - b).perp_dot(at - b), (a - c).perp_dot(at - c)];
                sides.iter().all(|&side| side <= 1e-6) || sides.iter().all(|&side| side >= -1e-6)
            })
        };
        let gaps = (0..60).map(|k| 3.2 + k as f32 * 0.25).filter(|&along| along < rules::reach(10) - 2.0 && !covered(&dashed, along)).count();
        assert!(gaps > 3, "{gaps} gaps");
        assert!((0..60).map(|k| 1.0 + k as f32 * 0.25).filter(|&along| along < rules::reach(10) - 2.0).all(|along| covered(&whole, along)));
    }

    #[test]
    fn the_cracks_lie_where_the_ice_will_break_and_never_past_its_edge() {
        for seed in 0..12 {
            let floe = Floe::thrown(seed);
            for turn in 1..=rules::LAST_TURN {
                let (next, now) = (rules::size_after(turn), rules::size_in(turn));
                let mesh = crack_mesh(&floe, next, now, u64::from(turn));
                let all = triangles(&mesh);
                assert!(!all.is_empty(), "seed {seed} turn {turn}");
                for at in all.iter().flatten() {
                    // On the ice there is now, but for how far a crack wanders
                    // and how wide it is.
                    assert!(floe.depth(*at, now) > -(CRACK_WANDER + CRACK_WIDE), "seed {seed} turn {turn}: {at}");
                }
            }
        }
    }

    #[test]
    fn the_band_that_will_break_is_the_ice_between_the_two_outlines() {
        for seed in 0..12 {
            let floe = Floe::thrown(seed);
            let (inner, outer) = (rules::size_after(3), rules::size_in(3));
            let band = triangles(&band_mesh(&floe, inner, outer));
            assert!(!band.is_empty());
            // Its middle is on the ice there is, and not on what is left.
            for [a, b, c] in band.iter().step_by(7) {
                let middle = (*a + *b + *c) / 3.0;
                assert!(floe.depth(middle, outer) > -1e-3 && floe.depth(middle, inner) < 1e-3, "seed {seed}: {middle}");
            }
            // Facing up, as everything lying on the ice is.
            for [a, b, c] in &band {
                assert!((*b - *a).perp_dot(*c - *a) <= 1e-6, "seed {seed}");
            }
        }
    }

    #[test]
    fn the_island_is_its_outline_three_metres_deep_closed_and_facing_out() {
        let floe = Floe::thrown(4);
        let corners = island_triangles(&floe);
        let shape = crate::island::Island::new(&corners);
        // Ice to stand on, at its top, on the outline's side of it; sea past.
        for k in 0..64 {
            let way = Vec2::from_angle(k as f32 * TAU / 64.0);
            let edge = floe.reach_toward(way);
            assert_eq!(shape.floor(way * (edge - 0.05), 1.0), Some(0.0), "{way}");
            assert_eq!(shape.floor(way * (edge + 0.05), 1.0), None, "{way}");
        }
        // Closed: each side of every triangle is a side of another, the
        // other way round.
        let mut sides = std::collections::HashMap::new();
        for [a, b, c] in &corners {
            for (from, to) in [(a, b), (b, c), (c, a)] {
                let key = |at: &Vec3| (at.x.to_bits(), at.y.to_bits(), at.z.to_bits());
                *sides.entry((key(from), key(to))).or_insert(0) += 1;
            }
        }
        for ((from, to), count) in &sides {
            assert_eq!(sides.get(&(*to, *from)), Some(count), "an open side");
        }
        // As drawn: a mesh with something in it, every corner with a normal.
        let mesh = island_mesh(&floe);
        assert!(!triangles(&mesh).is_empty());
        assert_eq!(mesh.count_vertices(), 4 * floe.rim().len() + 2);
    }

    #[test]
    fn the_ice_breaks_into_pieces_of_every_size_that_go_all_the_way_round() {
        let floe = Floe::thrown(11);
        for turn in 1..=rules::LAST_TURN {
            let (inner, outer) = (rules::size_after(turn), rules::size_in(turn));
            let middle = (outer + inner) * 0.5;
            let mut dice = fastrand::Rng::with_seed(u64::from(turn));
            let began = 1.3;
            let pieces = pieces_round(&floe, middle, began, &mut dice);
            assert!(pieces.len() >= 2, "turn {turn}");
            // Round the whole rim, one after another, none a sliver.
            assert_eq!(pieces[0].0, began);
            assert!((pieces.last().unwrap().1 - began - TAU).abs() < 1e-4, "turn {turn}");
            for pair in pieces.windows(2) {
                assert_eq!(pair[0].1, pair[1].0);
            }
            let out = |angle: f32| (floe.reach_toward(Vec2::from_angle(angle)) * middle).max(0.3);
            let lengths: Vec<f32> = pieces.iter().map(|(from, to)| (to - from) * out((from + to) * 0.5)).collect();
            assert!(lengths.iter().all(|&long| long > 0.2 * PIECE_LONG.0), "turn {turn}: {lengths:?}");
            if pieces.len() > 6 {
                let (least, most) = lengths.iter().fold((f32::MAX, 0.0f32), |(least, most), &long| (least.min(long), most.max(long)));
                assert!(most > 1.5 * least, "turn {turn}: all alike, {lengths:?}");
            }
            // And each piece is solid, never a mesh with nothing in it,
            // facing out all round.
            let (from, to) = pieces[0];
            let (mesh, middle) = piece_mesh(&floe, inner, outer, from, to, 0.6, &mut dice);
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
                panic!("positions");
            };
            assert!(!points.is_empty());
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) else {
                panic!("normals");
            };
            let Some(Indices::U32(indices)) = mesh.indices() else {
                panic!("indices");
            };
            for three in indices.chunks(3) {
                let [a, b, c] = [0, 1, 2].map(|k| Vec3::from_array(points[three[k] as usize]));
                let facing = Vec3::from_array(normals[three[0] as usize]);
                let turning = (b - a).cross(c - a);
                assert!(turning.dot(facing) >= -1e-6, "turn {turn}: a face turned inward");
            }
            let way = middle.normalize_or(Vec2::X);
            let edge = floe.reach_toward(way);
            assert!(middle.length() > edge * inner * 0.8 - 0.2 && middle.length() < edge * outer + 0.05, "turn {turn}: {middle}");
        }
    }

    #[test]
    fn a_streak_narrows_and_fades_behind_whoever_slides() {
        let points: VecDeque<(f32, Vec2)> = (0..10).map(|k| (k as f32 * 0.04, Vec2::new(k as f32 * 0.5, 0.0))).collect();
        let mesh = trail_mesh(&points, 0.36, Color::WHITE);
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(colours)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("colours");
        };
        assert_eq!(colours.len(), 20);
        // The oldest faint, the newest strongest.
        assert!(colours[0][3] < colours[19][3] && (colours[19][3] - TRAIL_ALPHA).abs() < 1e-4);
        assert!(!triangles(&mesh).is_empty());
    }

    #[test]
    fn a_shake_is_felt_and_no_more() {
        assert!((0.0015..=0.011).contains(&shake_for(0.3)));
        assert!((0.0015..=0.011).contains(&shake_for(30.0)));
        assert!(shake_for(10.0) > shake_for(2.0));
        // As hard as two meet, a little over a per cent of the way to the
        // ice: a third of a metre from 30 m off.
        assert!(shake_for(rules::speed(rules::MOST_POWER) * 2.0) * 30.0 > 0.25);
    }
}
