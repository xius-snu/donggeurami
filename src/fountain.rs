//! The town's fountain, running. Water wells up out of the top of its tower
//! into the top bowl, spills over the brim of each bowl into the one under it
//! and from the lowest into the basin; and every [`JET_EVERY`] seconds a jet
//! shoots up out of the top, throwing anyone standing in the top bowl as high
//! as it goes (Hajun, 2026-10-08).
//!
//! The still water is the model's own: the basin's, and since 2026-10-08 the
//! two bowls', Blender objects with the `water` material that are drawn like
//! the sea (`island::read_island`). Everything here is read off the model once
//! the town has spawned ([`Fountain::read`]): where the tower stands and how
//! high its top is, every piece of water on its axis, and each bowl's brim,
//! the widest the tower is beside the bowl's water. Move or reshape the
//! fountain in Blender, and the water running down it follows.
//!
//! The running water has a material of its own, [`FlowingWater`]: see-through,
//! with streaks the GPU moves along by the frame's time, so that once it is
//! spawned nothing about it changes from one frame to the next. Only the jet
//! moves, while it is up. It is all in the sorted transparent phase, a few
//! thousand triangles in eight draws, and only drawn in the town.
//!
//! The jet goes up by the clock, not by the game's time, so that every
//! device's goes up at the same moment and everyone in the town sees the same
//! jet throw the same player. A throw is the device's own: it moves its own
//! player, as a jump does, and the server takes it as it takes a jump, since
//! it rises little faster than one (the server's `MOST_RISE` and `BURST`).

use std::f32::consts::{PI, TAU};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::asset::{RenderAssetUsages, uuid_handle};
use bevy::color::ColorToComponents;
use bevy::gltf::GltfMaterialName;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology, VertexAttributeValues};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::{Shader, ShaderRef};
use bevy::world_serialization::WorldInstanceReady;

use crate::island::{Venue, WATER_MATERIAL};
use crate::{GRAVITY, Intent, PlayerJump};

/// What the fountain's tower is called in the town's model.
const TOWER: &str = "Water_fountain_tower";
/// How far from the tower's axis a piece of water can be centred and still be
/// the fountain's, in metres.
const ON_THE_AXIS: f32 = 0.5;
/// How far over or under a bowl's water its brim is looked for, in metres.
const BRIM_REACH: f32 = 0.12;
/// How far over the stone running water runs, in metres, so as not to be
/// lost in it.
const OVER_STONE: f32 = 0.006;
/// How far over a pool its foam floats, in metres: enough for it to be drawn
/// after the pool, as see-through things are drawn furthest first.
const OVER_WATER: f32 = 0.006;
/// Running water falls the way water does. Bodies fall faster (`GRAVITY`):
/// the game's jump is snappier than life.
const WATER_GRAVITY: f32 = 9.81;
/// How far out from the axis the water welling up out of the top starts.
const SPOUT: f32 = 0.02;
/// How high the water wells up over the top before it falls, in metres.
const WELL_RISE: f32 = 0.22;
/// How far out the welling water lands in the top bowl, as a share of the way
/// to the edge of its water.
const WELL_LANDS: f32 = 0.6;
/// How far out spilled water lands at most, as a share of the way to the edge
/// of the pool it falls into.
const SPILL_LANDS: f32 = 0.85;
/// How fast spilled water leaves a brim, out from the axis, in metres a
/// second: at least this, so that it falls clear of the stone, and at most
/// this, so that it falls like water rather than being thrown.
const SPILL_OUT: (f32, f32) = (0.04, 0.35);
/// How many points along its fall a sheet of falling water is drawn with.
const FALL_POINTS: usize = 10;
/// How wide a ring of foam is either side of where water lands, in metres.
const FOAM: f32 = 0.06;
/// How many sides each sheet of water has round the axis: the tower's own 32.
const SIDES: u32 = 32;
/// The jet is narrower, and has fewer.
const JET_SIDES: u32 = 16;
/// How far apart the streaks on the running water are, round the fountain, in
/// metres: a few broad ones, as the rest of the town is drawn, rather than
/// many fine ones. Until Hajun found them "tooooo detailed" (2026-10-08)
/// every sheet had 40, however small.
const STREAK: f32 = 0.28;
/// The same for the foam where the water lands, in broad soft patches, and
/// for the narrow column of the jet.
const FOAM_PATCH: f32 = 0.45;
const JET_STREAK: f32 = 0.14;

/// How often the jet goes up, in seconds by the clock.
const JET_EVERY: f64 = 8.0;
/// How long the jet takes to shoot up, how long it stays up, and how long it
/// takes to fall back, in seconds. It throws only until it starts to fall,
/// which is sooner than anyone it throws comes back down
/// (`a_throw_comes_down_after_the_jet_stops_throwing`), so that one jet
/// throws nobody twice.
const JET_RISE: f32 = 0.2;
const JET_HOLD: f32 = 0.8;
const JET_FALL: f32 = 0.5;
/// How high the jet goes over the top of the tower, in metres.
const JET_HEIGHT: f32 = 6.0;
/// How high the jet throws whoever stands in the top bowl, over it, in
/// metres: out of the top of the jet and on up. Hajun asked for more than
/// the 4.5 m it was at first, the jet's own height then (2026-10-08). The
/// server takes a throw up to about 13 m as a move a body could make
/// (`MOST_RISE` and `BURST`, and a test of this in `server/src/players.rs`).
const THROW_HEIGHT: f32 = 9.0;
/// How much the jet's height wavers while it is up.
const JET_WAVER: f32 = 0.03;

/// The running water's shader, `fountain.wgsl`, built into the game rather
/// than loaded from `assets/`.
const SHADER: Handle<Shader> = uuid_handle!("5f3c1b8e-2a4d-4e7b-9c61-0d8a7e2f4b13");

pub(crate) fn plugin(app: &mut App) {
    app.world_mut()
        .resource_mut::<Assets<Shader>>()
        .insert(
            SHADER.id(),
            Shader::from_wgsl(include_str!("fountain.wgsl"), "donggeurami_town/fountain.wgsl"),
        )
        .expect("the fountain's shader has a handle of its own");
    app.add_plugins(MaterialPlugin::<FlowingWater>::default())
        .add_observer(find_fountain)
        .add_systems(Update, (spout, throw.before(crate::move_bodies)));
}

/// The fountain, as the town's model has it: its tower standing at `at`, and
/// everything else measured up and out from there.
#[derive(Resource, Clone, Debug, PartialEq)]
pub(crate) struct Fountain {
    at: Vec3,
    /// How high the top of the tower is.
    tip: f32,
    /// The water in it, from the top bowl down to the basin.
    pools: Vec<Pool>,
}

/// One piece of the fountain's water: how high it is, how far out from the
/// axis it reaches, and the brim it spills over, if any.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pool {
    height: f32,
    radius: f32,
    brim: Option<Brim>,
}

/// The rim of a bowl, which its water spills over: how far out it reaches and
/// how high its top is.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Brim {
    radius: f32,
    top: f32,
}

/// The line a sheet of running water runs along, out from the axis and up,
/// from where it starts to where it ends, and how solid it is at each point.
/// Turned round the axis, it is the sheet.
type Path = Vec<(Vec2, f32)>;

impl Fountain {
    /// The fountain whose tower stands at `at`, from the points of the tower's
    /// mesh and those of every piece of water in the town, all in the world.
    /// `None` if none of the water is on its axis.
    fn read(at: Vec3, tower: &[Vec3], waters: &[Vec<Vec3>]) -> Option<Self> {
        let out = |point: Vec3| ((point - at).xz().length(), point.y - at.y);
        let tip = tower.iter().map(|&point| out(point).1).reduce(f32::max)?;
        let mut pools: Vec<Pool> = waters
            .iter()
            .filter(|points| !points.is_empty())
            .filter_map(|points| {
                let middle = points.iter().sum::<Vec3>() / points.len() as f32;
                ((middle - at).xz().length() <= ON_THE_AXIS).then(|| Pool {
                    height: middle.y - at.y,
                    radius: points.iter().map(|&point| out(point).0).fold(0.0, f32::max),
                    brim: None,
                })
            })
            .collect();
        pools.sort_by(|a, b| b.height.total_cmp(&a.height));
        // Every pool but the lowest spills over the widest of the tower beside
        // it.
        let lowest = pools.len().checked_sub(1)?;
        for pool in &mut pools[..lowest] {
            let beside: Vec<(f32, f32)> = tower
                .iter()
                .map(|&point| out(point))
                .filter(|&(r, y)| (y - pool.height).abs() <= BRIM_REACH && r > pool.radius)
                .collect();
            let radius = beside.iter().map(|&(r, _)| r).reduce(f32::max);
            let top = beside.iter().map(|&(_, y)| y).reduce(f32::max);
            pool.brim = radius.zip(top).map(|(radius, top)| Brim { radius, top });
        }
        Some(Self { at, tip, pools })
    }

    /// The water always running down it: welling up out of the top into the
    /// top bowl, and spilling over each brim into the pool under it. Then the
    /// foam on each pool, where the water lands in it.
    fn running(&self) -> (Vec<Path>, Vec<Path>) {
        let top = &self.pools[0];
        let lands = top.radius * WELL_LANDS;
        let up = (2.0 * WATER_GRAVITY * WELL_RISE).sqrt();
        let start = Vec2::new(SPOUT, self.tip + OVER_STONE);
        let out = (lands - SPOUT) / flight(start.y - top.height, up);
        let mut falls = vec![solid_between(falling(start, out, up, top.height))];
        let mut foams = vec![foam(lands, top)];
        for pair in self.pools.windows(2) {
            let [pool, below] = pair else { continue };
            let Some(brim) = pool.brim else { continue };
            let (fall, lands) = spill(pool, &brim, below);
            falls.push(fall);
            foams.push(foam(lands, below));
        }
        (falls, foams)
    }

    /// Whether a body with its feet at `feet` stands in the top bowl, on its
    /// brim, on the edge of it with its legs, or on the spout over it: where
    /// the jet comes up.
    fn in_the_jet(&self, feet: Vec3) -> bool {
        let top = &self.pools[0];
        let reach = top.brim.map_or(top.radius, |brim| brim.radius) + crate::LEG_RADIUS;
        let from = feet - self.at;
        from.xz().length() <= reach && (top.height - 0.15..=self.tip + 0.3).contains(&from.y)
    }
}

/// How long water thrown up at `up` metres a second takes to come down to
/// `drop` metres under where it started, in seconds.
fn flight(drop: f32, up: f32) -> f32 {
    (up + (up * up + 2.0 * WATER_GRAVITY * drop.max(0.0)).sqrt()) / WATER_GRAVITY
}

/// Where water leaving `from` at `out` metres a second away from the axis and
/// `up` metres a second upward goes until it has fallen to `down_to`, evenly
/// in time.
fn falling(from: Vec2, out: f32, up: f32, down_to: f32) -> Vec<Vec2> {
    let time = flight(from.y - down_to, up);
    (0..=FALL_POINTS)
        .map(|step| {
            let t = time * step as f32 / FALL_POINTS as f32;
            Vec2::new(from.x + out * t, from.y + up * t - 0.5 * WATER_GRAVITY * t * t)
        })
        .collect()
}

/// `points` as a sheet that comes out of nothing at its start and goes into
/// nothing at its end, solid in between.
fn solid_between(points: Vec<Vec2>) -> Path {
    let last = points.len().saturating_sub(1);
    points
        .into_iter()
        .enumerate()
        .map(|(at, point)| (point, if at == 0 || at == last { 0.0 } else { 1.0 }))
        .collect()
}

/// The water spilling out of `pool` over `brim`: across the top of the brim
/// and down off its edge into `below`. Where it lands too, out from the axis.
fn spill(pool: &Pool, brim: &Brim, below: &Pool) -> (Path, f32) {
    let over = brim.top + OVER_STONE;
    let edge = Vec2::new(brim.radius + OVER_STONE, over);
    let time = flight(over - below.height, 0.0);
    let out = ((below.radius * SPILL_LANDS - edge.x) / time).clamp(SPILL_OUT.0, SPILL_OUT.1);
    // Thin over the brim, coming out of the water a little way in from its
    // edge, and only all there as it goes over.
    let across = (edge.x - pool.radius).max(0.0);
    let mut path = vec![
        (Vec2::new(pool.radius, over), 0.0),
        (Vec2::new(pool.radius + across.min(0.03), over), 0.5),
    ];
    let fall = falling(edge, out, 0.0, below.height);
    let lands = fall.last().map_or(edge.x, |point| point.x);
    let last = fall.len() - 1;
    path.extend(
        fall.into_iter()
            .enumerate()
            .map(|(at, point)| (point, if at == last { 0.0 } else { 1.0 })),
    );
    (path, lands)
}

/// A ring of foam on `pool`, round where water lands in it, `lands` out from
/// the axis, and never past the edge of its water.
fn foam(lands: f32, pool: &Pool) -> Path {
    let inner = (lands - FOAM).max(0.02);
    let outer = (lands + FOAM).min(pool.radius * 0.97).max(inner + 0.02);
    let height = pool.height + OVER_WATER;
    [0.0, 0.35, 0.65, 1.0]
        .into_iter()
        .map(|share| {
            let solid = if share == 0.0 || share == 1.0 { 0.0 } else { 1.0 };
            (Vec2::new(inner + (outer - inner) * share, height), solid)
        })
        .collect()
}

/// The jet at its full height, up from the top of the tower: a column a
/// little wider at the top, rounded off.
fn column() -> Path {
    vec![
        (Vec2::new(0.05, 0.0), 0.9),
        (Vec2::new(0.07, 0.3), 1.0),
        (Vec2::new(0.1, JET_HEIGHT - 0.25), 1.0),
        (Vec2::new(0.08, JET_HEIGHT - 0.08), 0.8),
        (Vec2::new(0.0, JET_HEIGHT), 0.0),
    ]
}

/// The water bursting out of the top of the jet and falling away round it,
/// thinning as it goes.
fn crown() -> Path {
    let points = falling(Vec2::new(0.08, 0.0), 1.6, 1.0, -2.5);
    let last = points.len() - 1;
    points
        .into_iter()
        .enumerate()
        .map(|(at, point)| (point, if at == 0 { 0.6 } else { 1.0 - at as f32 / last as f32 }))
        .collect()
}

/// `path` turned round the axis, `sides` times, with streaks round it about
/// `streak` metres apart (`fountain.wgsl`): `uv.x` round it in streaks, one
/// to each whole number, as many whole streaks as fit round the middle of
/// the path, and at least three; `uv.y` along the path in metres; and the
/// colour's alpha how solid it is.
fn lathe(path: &Path, sides: u32, streak: f32) -> Mesh {
    let rings = path.len();
    let across = sides as usize + 1;
    let middle = path.iter().map(|&(point, _)| point.x).sum::<f32>() / rings.max(1) as f32;
    let streaks = (middle * TAU / streak).round().max(3.0);
    let mut positions = Vec::with_capacity(rings * across);
    let mut normals = Vec::with_capacity(rings * across);
    let mut uvs = Vec::with_capacity(rings * across);
    let mut colors = Vec::with_capacity(rings * across);
    let mut along = 0.0;
    for (at, &(point, solid)) in path.iter().enumerate() {
        if at > 0 {
            along += point.distance(path[at - 1].0);
        }
        let before = path[at.saturating_sub(1)].0;
        let after = path[(at + 1).min(rings - 1)].0;
        let way = (after - before).normalize_or(Vec2::Y);
        let facing = Vec2::new(-way.y, way.x);
        for side in 0..across {
            let share = side as f32 / sides as f32;
            let (sin, cos) = (share * TAU).sin_cos();
            positions.push([point.x * cos, point.y, point.x * sin]);
            normals.push([facing.x * cos, facing.y, facing.x * sin]);
            uvs.push([share * streaks, along]);
            colors.push([1.0, 1.0, 1.0, solid]);
        }
    }
    let mut indices = Vec::with_capacity((rings - 1) * sides as usize * 6);
    for ring in 0..rings.saturating_sub(1) {
        for side in 0..sides as usize {
            let a = (ring * across + side) as u32;
            let (b, c) = (a + 1, a + across as u32);
            indices.extend([a, c, b, b, c, c + 1]);
        }
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
}

// ---------------------------------------------------------------- the jet

/// The seconds since the jet last started to go up, by the clock: the same on
/// every device whose clock is right.
fn jet_phase() -> f32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |since| since.as_secs_f64());
    (now % JET_EVERY) as f32
}

/// How high the jet stands `phase` seconds after it started to go up, as a
/// share of [`JET_HEIGHT`], or `None` once it has fallen back.
fn jet_rise(phase: f32) -> Option<f32> {
    let held = phase - JET_RISE;
    let dropping = held - JET_HOLD;
    let rise = if phase < JET_RISE {
        let shot = phase / JET_RISE;
        1.0 - (1.0 - shot) * (1.0 - shot)
    } else if held < JET_HOLD {
        1.0 + JET_WAVER * (held / JET_HOLD * PI).sin() * (held * 23.0).sin()
    } else if dropping < JET_FALL {
        let fallen = dropping / JET_FALL;
        1.0 - fallen * fallen
    } else {
        return None;
    };
    Some(rise.max(0.02))
}

/// Whether the jet throws whoever stands in it, `phase` seconds after it
/// started to go up: until it starts to fall back.
fn throwing(phase: f32) -> bool {
    phase < JET_RISE + JET_HOLD
}

/// How fast the jet throws a body up: as fast as takes it [`THROW_HEIGHT`]
/// higher, as gravity takes bodies.
fn throw_speed() -> f32 {
    (2.0 * GRAVITY * THROW_HEIGHT).sqrt()
}

/// The column of the jet.
#[derive(Component)]
struct Jet;

/// The water bursting out of the top of the jet.
#[derive(Component)]
struct Crown;

/// Puts the jet up, and takes it down again, by the clock. Down, which it is
/// most of the time, it is hidden and nothing about it is written.
fn spout(
    fountain: Option<Res<Fountain>>,
    mut jets: Query<(&mut Transform, &mut Visibility), (With<Jet>, Without<Crown>)>,
    mut crowns: Query<(&mut Transform, &mut Visibility), (With<Crown>, Without<Jet>)>,
) {
    let Some(fountain) = fountain else {
        return;
    };
    let rise = jet_rise(jet_phase());
    let shown = if rise.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for (mut transform, mut visibility) in &mut jets {
        visibility.set_if_neq(shown);
        if let Some(rise) = rise {
            transform.set_if_neq(
                Transform::from_xyz(0.0, fountain.tip, 0.0).with_scale(Vec3::new(1.0, rise, 1.0)),
            );
        }
    }
    for (mut transform, mut visibility) in &mut crowns {
        visibility.set_if_neq(shown);
        if let Some(rise) = rise {
            let top = fountain.tip + JET_HEIGHT * rise;
            transform.set_if_neq(
                Transform::from_xyz(0.0, top, 0.0).with_scale(Vec3::splat(rise.min(1.0))),
            );
        }
    }
}

/// Throws up into the air whoever this device moves — you, or one of its AIs
/// — standing in the jet as it comes up. Each device throws its own: everyone
/// else is moved by theirs.
fn throw(
    fountain: Option<Res<Fountain>>,
    mut bodies: Query<(&Transform, &Venue, &mut PlayerJump), With<Intent>>,
) {
    let Some(fountain) = fountain else {
        return;
    };
    if !throwing(jet_phase()) {
        return;
    }
    for (body, venue, mut jump) in &mut bodies {
        if *venue == Venue::Town
            && jump.velocity_y <= 0.0
            && fountain.in_the_jet(body.translation)
        {
            jump.velocity_y = throw_speed();
        }
    }
}

// --------------------------------------------------------- finding it

/// Finds the fountain in the town's model once it has spawned, by the name of
/// its tower, and sets its water running.
#[allow(clippy::too_many_arguments)]
fn find_fountain(
    ready: On<WorldInstanceReady>,
    venues: Query<&Venue>,
    children: Query<&Children>,
    names: Query<&Name>,
    parts: Query<(&Mesh3d, Option<&GltfMaterialName>)>,
    placement: TransformHelper,
    found: Option<Res<Fountain>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FlowingWater>>,
    mut commands: Commands,
) {
    if found.is_some() || venues.get(ready.entity).ok() != Some(&Venue::Town) {
        return;
    }
    let Some(tower) = children
        .iter_descendants(ready.entity)
        .find(|&entity| names.get(entity).is_ok_and(|name| name.as_str() == TOWER))
    else {
        return;
    };
    // Every point of the meshes of `entity`, placed in the world.
    let points = |entity: Entity| -> Vec<Vec3> {
        let Ok((mesh, _)) = parts.get(entity) else {
            return Vec::new();
        };
        let Some(Ok(VertexAttributeValues::Float32x3(corners))) = meshes
            .get(&mesh.0)
            .map(|mesh| mesh.try_attribute(Mesh::ATTRIBUTE_POSITION))
        else {
            return Vec::new();
        };
        let Ok(place) = placement.compute_global_transform(entity) else {
            return Vec::new();
        };
        corners.iter().map(|&corner| place.transform_point(Vec3::from(corner))).collect()
    };
    let shape: Vec<Vec3> = std::iter::once(tower)
        .chain(children.iter_descendants(tower))
        .flat_map(points)
        .collect();
    let waters: Vec<Vec<Vec3>> = children
        .iter_descendants(ready.entity)
        .filter(|&entity| {
            parts
                .get(entity)
                .is_ok_and(|(_, material)| material.is_some_and(|name| name.0 == WATER_MATERIAL))
        })
        .map(points)
        .collect();
    let Ok(at) = placement.compute_global_transform(tower) else {
        return;
    };
    let Some(fountain) = Fountain::read(at.translation(), &shape, &waters) else {
        warn!("{TOWER} has no water on it, so nothing runs down it");
        return;
    };
    spawn_water(&mut commands, &mut meshes, &mut materials, &fountain);
    commands.insert_resource(fountain);
}

/// The fountain's running water, and its jet, down for now: in the town, and
/// only drawn there.
fn spawn_water(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<FlowingWater>,
    fountain: &Fountain,
) {
    let falls = materials.add(FlowingWater::falls());
    let foam = materials.add(FlowingWater::foam());
    let jet = materials.add(FlowingWater::jet());
    let (sheets, rings) = fountain.running();
    commands
        .spawn((
            Name::new("Fountain water"),
            Venue::Town,
            Transform::from_translation(fountain.at),
            Visibility::Hidden,
        ))
        .with_children(|water| {
            for sheet in &sheets {
                water.spawn((Mesh3d(meshes.add(lathe(sheet, SIDES, STREAK))), MeshMaterial3d(falls.clone())));
            }
            for ring in &rings {
                water.spawn((Mesh3d(meshes.add(lathe(ring, SIDES, FOAM_PATCH))), MeshMaterial3d(foam.clone())));
            }
            water.spawn((
                Jet,
                Mesh3d(meshes.add(lathe(&column(), JET_SIDES, JET_STREAK))),
                MeshMaterial3d(jet.clone()),
                Transform::from_xyz(0.0, fountain.tip, 0.0),
                Visibility::Hidden,
            ));
            water.spawn((
                Crown,
                Mesh3d(meshes.add(lathe(&crown(), SIDES, STREAK))),
                MeshMaterial3d(jet),
                Transform::from_xyz(0.0, fountain.tip + JET_HEIGHT, 0.0),
                Visibility::Hidden,
            ));
        });
}

// ------------------------------------------------------- what it looks like

/// Running water: see-through, with white streaks running along it
/// (`fountain.wgsl`).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub(crate) struct FlowingWater {
    #[uniform(0)]
    look: Look,
}

/// What [`FlowingWater`] hands its shader.
#[derive(Clone, Copy, Debug, ShaderType)]
struct Look {
    /// The water between the streaks, linear, and in `w` how solid it is.
    tint: Vec4,
    /// The streaks, linear, and in `w` how solid they are at most.
    streak: Vec4,
    /// How fast the water runs, in metres a second; how far apart the swells
    /// passing down a streak are, in metres; how wide a streak is, as a share
    /// of the room it has; and nothing.
    flow: Vec4,
}

impl FlowingWater {
    /// Spilling over the brims and welling out of the top.
    fn falls() -> Self {
        Self::new(
            Color::srgba(0.78, 0.91, 1.0, 0.22),
            Color::srgba(1.0, 1.0, 1.0, 0.6),
            [1.5, 0.5, 0.8, 0.0],
        )
    }

    /// Foam where the falling water lands.
    fn foam() -> Self {
        Self::new(
            Color::srgba(0.92, 0.97, 1.0, 0.3),
            Color::srgba(1.0, 1.0, 1.0, 0.5),
            [0.2, 0.3, 0.95, 0.0],
        )
    }

    /// The jet.
    fn jet() -> Self {
        Self::new(
            Color::srgba(0.8, 0.93, 1.0, 0.4),
            Color::srgba(1.0, 1.0, 1.0, 0.7),
            [6.0, 1.0, 0.8, 0.0],
        )
    }

    /// `flow` as [`Look::flow`] has it.
    fn new(tint: Color, streak: Color, flow: [f32; 4]) -> Self {
        Self {
            look: Look {
                tint: tint.to_linear().to_vec4(),
                streak: streak.to_linear().to_vec4(),
                flow: Vec4::from_array(flow),
            },
        }
    }
}

impl Material for FlowingWater {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Seen from both sides: from outside a sheet, and through it.
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FALL_GRAVITY;

    /// The tower in `circlemap1.glb`: every ring of it, out from the axis and
    /// up, as Blender has them.
    const TOWER_RINGS: [(f32, f32); 32] = [
        (0.200, 0.050), (0.242, 0.050), (0.200, 0.091), (0.242, 0.091),
        (0.180, 0.174), (0.200, 0.257), (0.107, 0.350), (0.165, 0.500),
        (0.267, 0.650), (0.466, 0.733), (0.638, 0.830), (0.098, 0.850),
        (0.500, 0.850), (0.736, 0.913), (0.601, 0.925), (0.727, 0.941),
        (0.049, 0.950), (0.066, 1.050), (0.098, 1.150), (0.252, 1.245),
        (0.396, 1.355), (0.089, 1.400), (0.239, 1.400), (0.340, 1.450),
        (0.491, 1.450), (0.066, 1.550), (0.080, 1.600), (0.064, 1.650),
        (0.035, 1.750), (0.048, 1.750), (0.021, 1.761), (0.015, 1.800),
    ];

    /// A ring of points `r` out from `at` and `y` up, as many as the tower's
    /// sides.
    fn ring(at: Vec3, r: f32, y: f32) -> Vec<Vec3> {
        (0..32)
            .map(|side| {
                let (sin, cos) = (side as f32 / 32.0 * TAU).sin_cos();
                at + Vec3::new(r * cos, y, r * sin)
            })
            .collect()
    }

    /// The town's fountain, standing at `at`: its tower, its basin and its
    /// two bowls' water, and a pond somewhere else in the town.
    fn town(at: Vec3) -> Fountain {
        let tower: Vec<Vec3> = TOWER_RINGS.iter().flat_map(|&(r, y)| ring(at, r, y)).collect();
        let waters = vec![
            ring(at, 1.897, 0.45),
            ring(at, 0.3197, 1.44),
            ring(at + Vec3::new(20.0, 0.0, -9.0), 3.0, 0.1),
            ring(at, 0.5871, 0.915),
        ];
        Fountain::read(at, &tower, &waters).expect("a fountain")
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn reads_the_fountain_off_the_model() {
        let at = Vec3::new(3.0, 0.2, -1.0);
        let fountain = town(at);
        assert_eq!(fountain.at, at);
        assert!(close(fountain.tip, 1.8), "{}", fountain.tip);
        // The bowls top to bottom, then the basin; not the pond.
        let heights: Vec<f32> = fountain.pools.iter().map(|pool| pool.height).collect();
        assert_eq!(heights.len(), 3, "{heights:?}");
        for (height, want) in heights.iter().zip([1.44, 0.915, 0.45]) {
            assert!(close(*height, want), "{heights:?}");
        }
        // Each bowl spills over the widest of the tower beside it; the basin
        // over nothing.
        let brims: Vec<Option<Brim>> = fountain.pools.iter().map(|pool| pool.brim).collect();
        let top = brims[0].expect("the top bowl's brim");
        assert!(close(top.radius, 0.491) && close(top.top, 1.45), "{top:?}");
        let middle = brims[1].expect("the middle bowl's brim");
        assert!(close(middle.radius, 0.736) && close(middle.top, 0.941), "{middle:?}");
        assert_eq!(brims[2], None);
    }

    #[test]
    fn water_lands_in_the_water_under_it() {
        let fountain = town(Vec3::ZERO);
        let (falls, foams) = fountain.running();
        // Out of the top, and over each bowl's brim.
        assert_eq!(falls.len(), 3);
        assert_eq!(foams.len(), 3);
        for (fall, pool) in falls.iter().zip(&fountain.pools) {
            let (end, solid) = *fall.last().expect("a path");
            assert!(close(end.y, pool.height), "ends at {end}, not on the water at {}", pool.height);
            assert!(end.x < pool.radius * 0.9, "lands at {}, past the water's {}", end.x, pool.radius);
            assert_eq!(solid, 0.0, "goes into the water rather than ending");
            assert_eq!(fall[0].1, 0.0, "comes out of the water rather than starting");
        }
        // Welling out of the top, it falls clear of the spout: wider than
        // the tower at every height over the top bowl.
        let spout = TOWER_RINGS
            .iter()
            .filter(|&&(_, y)| y > 1.46)
            .map(|&(r, _)| r)
            .fold(0.0, f32::max);
        for &(point, _) in &falls[0] {
            assert!(point.y > 1.43, "{point} under the top bowl's water");
            if point.y < fountain.tip {
                assert!(point.x > spout, "{point} inside the spout, {spout} wide");
            }
        }
        // Spilling, it goes over the brim and only ever outward and down
        // from its edge: away from the stone under it.
        for (fall, pool) in falls[1..].iter().zip(&fountain.pools) {
            let brim = pool.brim.expect("a brim");
            let edge = fall
                .iter()
                .position(|&(point, _)| point.x > brim.radius)
                .expect("over the edge");
            assert!(fall[..edge].iter().all(|&(point, _)| point.y > brim.top), "{fall:?}");
            for pair in fall[edge..].windows(2) {
                let (a, b) = (pair[0].0, pair[1].0);
                assert!(b.x >= a.x && b.y < a.y, "{a} to {b}");
            }
        }
    }

    #[test]
    fn foam_floats_on_its_water() {
        let fountain = town(Vec3::ZERO);
        let (_, foams) = fountain.running();
        for (foam, pool) in foams.iter().zip(&fountain.pools) {
            for &(point, _) in foam {
                assert!(close(point.y, pool.height + OVER_WATER), "{point}");
                assert!(point.x > 0.0 && point.x <= pool.radius, "{point} off the water");
            }
        }
    }

    #[test]
    fn the_jet_comes_and_goes() {
        // Shooting up, from next to nothing.
        let first = jet_rise(0.0).expect("up");
        let partway = jet_rise(JET_RISE * 0.5).expect("up");
        assert!(first < 0.05 && partway > first && partway < 1.0, "{first} {partway}");
        // Up, wavering.
        for step in 0..=20 {
            let rise = jet_rise(JET_RISE + JET_HOLD * step as f32 / 20.0).expect("up");
            assert!((rise - 1.0).abs() <= JET_WAVER, "{rise}");
        }
        // Falling back, and gone.
        let falling = jet_rise(JET_RISE + JET_HOLD + JET_FALL * 0.5).expect("falling");
        assert!(falling < 1.0, "{falling}");
        assert_eq!(jet_rise(JET_RISE + JET_HOLD + JET_FALL + 0.01), None);
        assert_eq!(jet_rise(JET_EVERY as f32 - 0.01), None);
        // It throws while it is up, and not once it falls.
        assert!(throwing(0.0) && throwing(JET_RISE + JET_HOLD - 0.01));
        assert!(!throwing(JET_RISE + JET_HOLD) && !throwing(JET_EVERY as f32 - 0.01));
    }

    #[test]
    fn a_throw_comes_down_after_the_jet_stops_throwing() {
        // As high as it throws, and higher than the jet goes.
        let speed = throw_speed();
        assert!(close(speed * speed / (2.0 * GRAVITY), THROW_HEIGHT));
        assert!(THROW_HEIGHT > JET_HEIGHT);
        // Up, then down from there: all of it longer than the jet throws, so
        // nobody it throws lands back in it while it can throw them again.
        let flight = speed / GRAVITY + (2.0 * THROW_HEIGHT / FALL_GRAVITY).sqrt();
        assert!(flight > JET_RISE + JET_HOLD, "{flight}");
    }

    #[test]
    fn only_the_top_bowl_is_in_the_jet() {
        let at = Vec3::new(3.0, 0.2, -1.0);
        let fountain = town(at);
        let feet = |x: f32, y: f32, z: f32| at + Vec3::new(x, y, z);
        // Standing in the top bowl, on its brim, on its edge with the legs,
        // and on the spout.
        assert!(fountain.in_the_jet(feet(0.15, 1.40, 0.1)));
        assert!(fountain.in_the_jet(feet(0.0, 1.45, -0.45)));
        assert!(fountain.in_the_jet(feet(0.6, 1.45, 0.0)));
        assert!(fountain.in_the_jet(feet(0.02, 1.80, 0.0)));
        // Past its edge, in the bowl under it, in the basin, and high over it.
        assert!(!fountain.in_the_jet(feet(0.75, 1.45, 0.0)));
        assert!(!fountain.in_the_jet(feet(0.0, 0.85, 0.6)));
        assert!(!fountain.in_the_jet(feet(1.2, 0.05, 0.0)));
        assert!(!fountain.in_the_jet(feet(0.1, 3.0, 0.0)));
    }

    #[test]
    fn a_path_turned_round_the_axis() {
        let path: Path = vec![
            (Vec2::new(0.5, 1.0), 0.0),
            (Vec2::new(0.5, 0.7), 1.0),
            (Vec2::new(0.6, 0.3), 0.0),
        ];
        // Streaks a metre apart round a path 3.35 m round its middle: three.
        let mesh = lathe(&path, 8, 1.0);
        assert_eq!(mesh.count_vertices(), 3 * 9);
        assert_eq!(mesh.indices().map(|indices| indices.len()), Some(2 * 8 * 6));
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("no uvs");
        };
        // Round in streaks, from 0 to 3, along in metres.
        assert_eq!(uvs[4], [1.5, 0.0]);
        assert_eq!(uvs[8], [3.0, 0.0]);
        assert!(close(uvs[9][1], 0.3));
        assert!(close(uvs[18][1], 0.3 + 0.1f32.hypot(0.4)));
        let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!("no colours");
        };
        assert_eq!((colors[0][3], colors[9][3], colors[18][3]), (0.0, 1.0, 0.0));
    }
}
