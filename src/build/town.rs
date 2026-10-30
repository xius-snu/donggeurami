//! The town's houses, each with a door that swings open and windows (Hajun,
//! 2026-10-30: "put doors that open and a few windows in each house in the
//! game (not in the blend) using the various door models").
//!
//! Every house in Hajun's town, `roundtownmap.glb`, is their Square House,
//! House Builder's own (`mybuilds/square_house.glb`), put down whole in
//! Blender: 64 of them as it is and 32 with a pink roof, a few to an object.
//! Nothing about where they are is in the code. Once the town has spawned,
//! each is found by its floor, a box the size of the Square House's
//! ([`floors`]), and turned the way its roof's ridge runs; the Square House's
//! walls, read off its own model, stand on it ([`Shape`]). The town's own
//! walls there are taken out of what is drawn and what is stood on
//! ([`walled`]), and each is made again as House Builder makes a wall, the
//! boxes left round the holes in it (`Wall`, `wall_mesh`): in the wall that
//! faces the nearest road, a door, one of Hajun's five (`DOORS`), with a
//! square window either side of it, and a window in each of the other three.
//! Which door and which windows goes by where the house stands, the same on
//! every device. The doors swing open for anyone who comes up to them, as
//! House Builder's do (`swing_doors`), and a window's hole is shut to bodies
//! by its pane (`pane`), as House Builder's are.
//!
//! Each house is its own, as a house in House Builder is ([`TownHouse`]):
//! its walls, a wall at a time, the frames of each wall's windows, its glass,
//! its door, and its roof and gables, the Square House's, in the town's own
//! colours there, taken out of what of the town is drawn. Standing in one,
//! whatever of it is between the camera and you is drawn see-through, as in
//! a house in House Builder (`see_through`; Hajun, 2026-10-30). Until then
//! the walls and windows of every house were drawn as one mesh to each
//! material, and roofs as the town's: one wall faded would have faded every
//! house's. A wall, a wall's frames or a house's glass the same in many
//! houses is one mesh, which the GPU draws in one go for all of them. The
//! House Builder stands at the door of the house nearest where the first
//! seat arrives (Hajun: "for now just put the build house battle minigame in
//! front of one house").

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::gltf::GltfMaterialName;
use bevy::math::Affine3A;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use super::{
    APART, Bounds, Height, Mount, OnWall, PIECES, PieceDef, Rooms, Span, WINDOW_SILL, Wall, YouAreIn, doorway,
    extent, pane, wall_mesh,
};
use crate::island::{self, Island, Islands, Land, Venue, WATER_MATERIAL};

/// Hajun's house that every house of the town is.
const HOUSE: &str = "square_house";
/// Their doors, one to a house, taken by turns.
const DOORS: [&str; 5] = [
    "classic_door",
    "classic_door_with_window",
    "classic_double_door",
    "glass_door",
    "glass_double_door",
];
/// Their windows: square ones either side of the door, and square or wide
/// in the other walls.
const SQUARE: &str = "square_window";
const WIDE: &str = "wide_window";
/// How near a box in the town has to be to the size of the Square House's
/// floor, every way, to be one, in metres.
const SAME_SIZE: f32 = 0.02;
/// How far out from a wall a road is looked for, for the door to face it,
/// in metres, and how finely.
const ROAD_REACH: f32 = 60.0;
const ROAD_STEP: f32 = 0.5;
/// How far a triangle can stick out of a wall's box, in metres, and still be
/// that wall: the town's walls are where the Square House's are to well
/// within this.
const SNUG: f32 = 0.02;
/// How far out from the front of their house, from the middle of its door,
/// the House Builder stands, in metres: clear of the leaves swinging out.
const AT_THE_DOOR: f32 = 2.5;

/// A model read to raise the town's houses with: the house, or one of the
/// doors or windows put in them.
#[derive(Component)]
struct Model(&'static PieceDef);

/// One of the town's doors, put in by this module.
#[derive(Component)]
struct TownDoor(&'static PieceDef);

/// The Square House, in its own space: its floor, its walls, which way
/// across the ground its roof's ridge runs, the box round all of it, and its
/// roof and gables, each mesh with its material, where it is and the box it
/// fills.
#[derive(Clone, Debug)]
struct Shape {
    floor: (Vec3, Vec3),
    walls: Vec<(Vec3, Vec3)>,
    ridge: Vec2,
    bounds: (Vec3, Vec3),
    ceiling: Vec<Ceiling>,
}

/// A mesh of the Square House over its rooms, its roof or a gable, in its own
/// space, and the box it fills there.
#[derive(Clone, Debug)]
struct Ceiling {
    mesh: Handle<Mesh>,
    material: Option<Handle<StandardMaterial>>,
    place: Affine3A,
    fills: (Vec3, Vec3),
}

/// One of the town's houses, its parts its children, as a house in House
/// Builder is: four walls (`Wall`), its roof and gables, the frames of its
/// windows a wall at a time and its door (`OnWall`), and its glass. Standing
/// in one, whatever of it stands between the camera and you is drawn
/// see-through (`see_through`; Hajun, 2026-10-30: "when the player goes
/// inside one of the houses in roundtownmap it does the similar transparent
/// thing as in the build minigame"). Its floor is the town's.
#[derive(Component)]
pub(crate) struct TownHouse;

/// A wall of one of the town's houses, cut once round its door and windows,
/// as House Builder's are cut whenever theirs change (`carve_walls`).
#[derive(Component)]
pub(super) struct TownWall;

/// A town house's glass: drawn as one, `whole`, and while you are in the
/// house, a wall at a time instead, to fade with its wall: each wall's frames,
/// which it goes with, and its glass ([`open_the_house`]).
#[derive(Component)]
struct Panes {
    whole: Vec<Entity>,
    by_wall: Vec<(Entity, Handle<Mesh>, Handle<StandardMaterial>)>,
}

/// The glass of a wall of the town house you are in.
#[derive(Component)]
struct WallPane;

/// A door or a window, in its own space: what of a wall its frame takes up,
/// and each mesh it is drawn with, with its material and where it is.
struct Opening {
    span: Span,
    parts: Vec<(Handle<Mesh>, Option<Handle<StandardMaterial>>, Affine3A)>,
}

/// One mesh of the town, placed in the world, with its material.
struct Part {
    mesh: Handle<Mesh>,
    material: Option<Handle<StandardMaterial>>,
    place: Affine3A,
    road: bool,
}

/// What the town's houses are raised from, gathered as each model spawns.
#[derive(Resource, Default)]
struct Raising {
    house: Option<Shape>,
    openings: HashMap<&'static str, Opening>,
    town: Option<Vec<Part>>,
    done: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Raising>()
        .add_observer(town_ready)
        .add_systems(Startup, send_for_models)
        .add_systems(Update, (raise_houses, open_the_house));
}

/// The kind `kind` in the shop.
fn def(kind: &str) -> &'static PieceDef {
    PIECES
        .iter()
        .find(|def| def.kind == kind)
        .unwrap_or_else(|| panic!("{kind} is not in build::PIECES"))
}

/// Loads every model the town's houses are raised from, each to be read
/// once it has spawned, and then taken away again.
fn send_for_models(assets: Res<AssetServer>, mut commands: Commands) {
    for kind in std::iter::once(HOUSE).chain(DOORS).chain([SQUARE, WIDE]) {
        let def = def(kind);
        commands
            .spawn((
                Name::new(format!("Reading {kind}")),
                Model(def),
                WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(def.model()))),
                Transform::default(),
                Visibility::Hidden,
            ))
            .observe(model_ready);
    }
}

/// Reads a model once it has spawned, in its own space: the Square House's
/// floor, walls and ridge, or a door's or window's frame and meshes.
#[allow(clippy::too_many_arguments)]
fn model_ready(
    ready: On<WorldInstanceReady>,
    models: Query<&Model>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    parts: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>)>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    mut raising: ResMut<Raising>,
    mut commands: Commands,
) {
    let root = ready.entity;
    let (Ok(Model(def)), Ok(root_place)) = (models.get(root), placement.compute_global_transform(root)) else {
        return;
    };
    let into_root = root_place.affine().inverse();
    let called = |part: Entity, prefix: &str| {
        [Some(part), parents.get(part).ok().map(ChildOf::parent)]
            .into_iter()
            .flatten()
            .any(|entity| names.get(entity).is_ok_and(|name| name.as_str().starts_with(prefix)))
    };
    let grow = |sum: Option<(Vec3, Vec3)>, (low, high): (Vec3, Vec3)| {
        Some(sum.map_or((low, high), |(a, b): (Vec3, Vec3)| (a.min(low), b.max(high))))
    };
    let (mut floor, mut frame, mut walls, mut corners, mut drawn) = (None, None, Vec::new(), Vec::new(), Vec::new());
    let (mut bounds, mut ceiling) = (None, Vec::new());
    for part in children.iter_descendants(root) {
        let (Ok((mesh, material)), Ok(place)) = (parts.get(part), placement.compute_global_transform(part)) else {
            continue;
        };
        let Some(asset) = meshes.get(&mesh.0) else {
            continue;
        };
        let in_root = into_root * place.affine();
        let Some(filled) = extent(asset, in_root) else {
            continue;
        };
        if called(part, "Floor") {
            floor = grow(floor, filled);
        }
        if called(part, "Wall") {
            walls.push(filled);
        }
        if called(part, "Frame") {
            frame = grow(frame, filled);
        }
        bounds = grow(bounds, filled);
        if called(part, "Roof") || called(part, "Gable") {
            ceiling.push(Ceiling {
                mesh: mesh.0.clone(),
                material: material.map(|material| material.0.clone()),
                place: in_root,
                fills: filled,
            });
        }
        if let Some(VertexAttributeValues::Float32x3(points)) = asset.attribute(Mesh::ATTRIBUTE_POSITION) {
            corners.extend(points.iter().map(|&point| in_root.transform_point3(Vec3::from(point))));
        }
        drawn.push((mesh.0.clone(), material.map(|material| material.0.clone()), in_root));
    }
    commands.entity(root).despawn();
    if def.kind == HOUSE {
        let (Some(floor), Some(bounds), false) = (floor, bounds, walls.is_empty()) else {
            warn!("{}: no floor or no walls, so the town's houses get no doors", def.model());
            return;
        };
        raising.house = Some(Shape {
            floor,
            walls,
            ridge: ridge(&corners).unwrap_or(Vec2::Y),
            bounds,
            ceiling,
        });
    } else if let Some((low, high)) = frame {
        raising.openings.insert(
            def.kind,
            Opening {
                span: Span::of(low, high),
                parts: drawn,
            },
        );
    }
}

/// Which way across the ground the highest of `corners` lie, the ridge of a
/// roof: along x or along z, as a direction a metre long. `None` if they
/// lie no further one way than the other.
fn ridge(corners: &[Vec3]) -> Option<Vec2> {
    let top = corners.iter().map(|corner| corner.y).fold(f32::NEG_INFINITY, f32::max);
    let (low, high) = corners
        .iter()
        .filter(|corner| corner.y > top - 0.05)
        .fold((Vec2::INFINITY, Vec2::NEG_INFINITY), |(low, high), corner| {
            (low.min(corner.xz()), high.max(corner.xz()))
        });
    let size = high - low;
    if (size.x - size.y).abs() < 0.5 {
        return None;
    }
    Some(if size.x > size.y { Vec2::X } else { Vec2::Y })
}

/// Keeps the town's meshes once it has spawned, each placed in the world
/// with its material: what its houses are raised in.
fn town_ready(
    ready: On<WorldInstanceReady>,
    towns: Query<&Venue, With<Land>>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>, Option<&GltfMaterialName>), Without<SkinnedMesh>>,
    placement: TransformHelper,
    mut raising: ResMut<Raising>,
) {
    if towns.get(ready.entity).ok() != Some(&Venue::Town) || raising.town.is_some() {
        return;
    }
    let mut town = Vec::new();
    for entity in children.iter_descendants(ready.entity) {
        let (Ok((mesh, material, name)), Ok(place)) = (parts.get(entity), placement.compute_global_transform(entity))
        else {
            continue;
        };
        let called = |wanted: &str| name.is_some_and(|name| island::is_material(&name.0, wanted));
        if called(WATER_MATERIAL) {
            continue;
        }
        town.push(Part {
            mesh: mesh.0.clone(),
            material: material.map(|material| material.0.clone()),
            place: place.affine(),
            road: called(crate::roads::ROAD),
        });
    }
    raising.town = Some(town);
}

/// The triangles of `mesh`, placed by `place`, each with the numbers of its
/// corners: none if it is not a list of triangles.
fn triangles(mesh: &Mesh, place: &Affine3A) -> Vec<([Vec3; 3], [u32; 3])> {
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
        return Vec::new();
    };
    if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
        return Vec::new();
    }
    let points: Vec<Vec3> = points.iter().map(|&point| place.transform_point3(Vec3::from(point))).collect();
    let order: Vec<u32> = match mesh.indices() {
        Some(indices) => indices.iter().map(|at| at as u32).collect(),
        None => (0..points.len() as u32).collect(),
    };
    order
        .chunks_exact(3)
        .filter(|three| three.iter().all(|&at| (at as usize) < points.len()))
        .map(|three| {
            let at = [three[0], three[1], three[2]];
            (at.map(|at| points[at as usize]), at)
        })
        .collect()
}

/// A house of the town: where it stands, as the Square House's own space
/// placed in the world, and which of the Square House's walls faces the
/// nearest road.
struct House {
    place: Affine3A,
    middle: Vec2,
}

/// The town's houses among `town`, each one a floor the size of the Square
/// House's, `shape`, turned so that its ridge runs the way the ridge over it
/// does, and stood on that floor. In the order of where they stand, the same
/// on every device.
fn floors(town: &[Vec<[Vec3; 3]>], shape: &Shape) -> Vec<House> {
    let (low, high) = shape.floor;
    let size = high - low;
    let corners: Vec<Vec3> = town.iter().flatten().flatten().copied().collect();
    let mut houses = Vec::new();
    for triangles in town {
        for piece in pieces(triangles) {
            let (bottom, top) = piece.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(b, t), c| (b.min(c.y), t.max(c.y)));
            if piece.len() != 8 || ((top - bottom) - size.y).abs() > SAME_SIZE {
                continue;
            }
            let mut face: Vec<Vec2> = piece.iter().filter(|c| c.y > top - SAME_SIZE).map(|c| c.xz()).collect();
            if face.len() != 4 {
                continue;
            }
            let middle = face.iter().sum::<Vec2>() / 4.0;
            face.sort_by(|a, b| (*a - middle).to_angle().total_cmp(&(*b - middle).to_angle()));
            let (one, two) = (face[1] - face[0], face[2] - face[1]);
            if one.normalize_or_zero().dot(two.normalize_or_zero()).abs() > 0.01 {
                continue;
            }
            // The Square House's x along one side or the other, its ridge
            // the way the ridge over this floor runs.
            let over: Vec<Vec3> = corners
                .iter()
                .copied()
                .filter(|c| c.y > top && inside(c.xz(), middle, one, two, 0.5))
                .collect();
            let wanted = ridge_across(&over, middle, one);
            let mut best: Option<(f32, Quat)> = None;
            for quarter in 0..4 {
                let along = Vec2::from_angle(quarter as f32 * std::f32::consts::FRAC_PI_2).rotate(one.normalize());
                let (long, wide) = if quarter % 2 == 0 { (one.length(), two.length()) } else { (two.length(), one.length()) };
                if (long - size.x).abs() > SAME_SIZE || (wide - size.z).abs() > SAME_SIZE {
                    continue;
                }
                // About the upright, taking x to `along`.
                let turn = Quat::from_rotation_y((-along.y).atan2(along.x));
                let runs = (turn * Vec3::new(shape.ridge.x, 0.0, shape.ridge.y)).xz();
                let fit = wanted.map_or(0.0, |wanted| runs.dot(wanted).abs());
                if best.is_none_or(|(most, _)| fit > most + 1e-3) {
                    best = Some((fit, turn));
                }
            }
            let Some((_, turn)) = best else {
                continue;
            };
            let floor_middle = (low + high) * 0.5;
            let from = turn * Vec3::new(floor_middle.x, low.y, floor_middle.z);
            houses.push(House {
                place: Affine3A::from_rotation_translation(turn, Vec3::new(middle.x, bottom, middle.y) - from),
                middle,
            });
        }
    }
    houses.sort_by(|a, b| {
        let key = |house: &House| ((house.middle.x * 100.0).round() as i64, (house.middle.y * 100.0).round() as i64);
        key(a).cmp(&key(b))
    });
    houses
}

/// Whether `at` is within `grow` of the rectangle round `middle` with sides
/// `one` and `two`, each its full length.
fn inside(at: Vec2, middle: Vec2, one: Vec2, two: Vec2, grow: f32) -> bool {
    let off = at - middle;
    off.dot(one.normalize()).abs() <= one.length() * 0.5 + grow && off.dot(two.normalize()).abs() <= two.length() * 0.5 + grow
}

/// Which way the ridge of the roof over a floor runs, the highest of `over`:
/// along `one`, a side of the floor, or across it. `None` if neither.
fn ridge_across(over: &[Vec3], middle: Vec2, one: Vec2) -> Option<Vec2> {
    let one = one.normalize();
    let local: Vec<Vec3> = over
        .iter()
        .map(|c| {
            let off = c.xz() - middle;
            Vec3::new(off.dot(one), c.y, off.dot(one.perp()))
        })
        .collect();
    ridge(&local).map(|way| if way == Vec2::X { one } else { one.perp() })
}

/// The pieces `triangles` make, joined where they share a corner: each
/// piece's corners, every one once.
fn pieces(triangles: &[[Vec3; 3]]) -> Vec<Vec<Vec3>> {
    let key = |corner: Vec3| {
        let at = (corner * 1000.0).round().as_ivec3();
        (at.x, at.y, at.z)
    };
    fn root(parent: &mut [usize], mut at: usize) -> usize {
        while parent[at] != at {
            parent[at] = parent[parent[at]];
            at = parent[at];
        }
        at
    }
    let mut parent: Vec<usize> = (0..triangles.len()).collect();
    let mut owner: HashMap<(i32, i32, i32), usize> = HashMap::new();
    for (index, triangle) in triangles.iter().enumerate() {
        for &corner in triangle {
            match owner.get(&key(corner)) {
                Some(&other) => {
                    let (a, b) = (root(&mut parent, other), root(&mut parent, index));
                    parent[a.max(b)] = a.min(b);
                }
                None => {
                    owner.insert(key(corner), index);
                }
            }
        }
    }
    let mut by_root: HashMap<usize, Vec<Vec3>> = HashMap::new();
    let mut seen: HashMap<usize, std::collections::HashSet<(i32, i32, i32)>> = HashMap::new();
    let mut order = Vec::new();
    for (index, triangle) in triangles.iter().enumerate() {
        let top = root(&mut parent, index);
        if !by_root.contains_key(&top) {
            order.push(top);
        }
        let corners = by_root.entry(top).or_default();
        let known = seen.entry(top).or_default();
        for &corner in triangle {
            if known.insert(key(corner)) {
                corners.push(corner);
            }
        }
    }
    order.into_iter().map(|top| by_root.remove(&top).unwrap_or_default()).collect()
}

/// Whether `triangle`, in its house's own space, is part of the wall that
/// fills `wall`: its sides, wholly inside it, and its top and bottom, facing
/// out of it; not the underside of a gable on top of it, nor the floor
/// under it, which only touch it.
fn walled(triangle: &[Vec3; 3], (low, high): (Vec3, Vec3)) -> bool {
    let within = triangle.iter().all(|c| {
        c.x >= low.x - SNUG
            && c.x <= high.x + SNUG
            && c.z >= low.z - SNUG
            && c.z <= high.z + SNUG
            && c.y >= low.y - SNUG
            && c.y <= high.y + SNUG
    });
    if !within {
        return false;
    }
    let normal = (triangle[1] - triangle[0]).cross(triangle[2] - triangle[0]).normalize_or_zero();
    let level = |y: f32| triangle.iter().all(|c| (c.y - y).abs() <= SNUG);
    if level(high.y) {
        return normal.y > 0.5;
    }
    if level(low.y) {
        return normal.y < -0.5;
    }
    true
}

/// A cheap mix of where a house stands, to choose its door and windows by:
/// the same everywhere for a house in the same place.
fn mix(at: Vec2) -> u32 {
    let (x, z) = ((at.x * 10.0).round() as i32 as u32, (at.y * 10.0).round() as i32 as u32);
    let mut h = x.wrapping_mul(0x9E37_79B1) ^ z.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

/// Meshes being gathered, one to a material: corners, which way each faces,
/// and triangles.
#[derive(Default)]
struct Gathered {
    points: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    order: Vec<u32>,
}

impl Gathered {
    /// Adds `mesh`, placed by `place`.
    fn add(&mut self, mesh: &Mesh, place: &Affine3A) {
        let (Some(VertexAttributeValues::Float32x3(points)), Some(VertexAttributeValues::Float32x3(normals))) =
            (mesh.attribute(Mesh::ATTRIBUTE_POSITION), mesh.attribute(Mesh::ATTRIBUTE_NORMAL))
        else {
            return;
        };
        let start = self.points.len() as u32;
        self.points.extend(points.iter().map(|&p| place.transform_point3(Vec3::from(p)).to_array()));
        self.normals.extend(
            normals
                .iter()
                .map(|&n| place.transform_vector3(Vec3::from(n)).normalize_or_zero().to_array()),
        );
        match mesh.indices() {
            Some(indices) => self.order.extend(indices.iter().map(|at| start + at as u32)),
            None => self.order.extend((0..points.len() as u32).map(|at| start + at)),
        }
    }

    fn mesh(self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.points)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_indices(Indices::U32(self.order))
    }
}

/// Raises the town's houses once the town and every model they are raised
/// from have spawned: their walls made again round the holes for their doors
/// and windows, the doors put in to swing, the windows drawn, and the
/// town's shape read again with all of that.
#[allow(clippy::too_many_arguments)]
fn raise_houses(
    mut raising: ResMut<Raising>,
    mut islands: ResMut<Islands>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if raising.done
        || raising.house.is_none()
        || raising.town.is_none()
        || raising.openings.len() < DOORS.len() + 2
        || islands.ground(Venue::Town).is_none()
    {
        return;
    }
    raising.done = true;
    let started = std::time::Instant::now();
    let Raising { house, openings, town, .. } = &*raising;
    let (Some(shape), Some(town)) = (house.clone(), town.as_ref()) else {
        return;
    };
    // Every triangle of the town, mesh by mesh.
    let read: Vec<Vec<([Vec3; 3], [u32; 3])>> = town
        .iter()
        .map(|part| meshes.get(&part.mesh).map(|mesh| triangles(mesh, &part.place)).unwrap_or_default())
        .collect();
    let plain: Vec<Vec<[Vec3; 3]>> = read.iter().map(|tris| tris.iter().map(|(t, _)| *t).collect()).collect();
    let houses = floors(&plain, &shape);
    if houses.is_empty() {
        info!("the town has no Square Houses in it, so no doors or windows go in");
        return;
    }
    let road: Vec<[Vec3; 3]> = town.iter().zip(&plain).filter(|(part, _)| part.road).flat_map(|(_, tris)| tris.iter().copied()).collect();
    let road = Island::new(&road);
    let Some(land) = islands.ground(Venue::Town) else {
        return;
    };
    let on_road = |at: Vec2| match (land.floor(at, 1.0), road.floor(at, 1.0)) {
        (Some(land), Some(road)) => (land - road).abs() <= 0.02,
        _ => false,
    };
    let floors_in = [shape.floor];
    let walls: Vec<Wall> = shape.walls.iter().map(|&(low, high)| Wall::new(low, high, &floors_in, &shape.walls)).collect();

    // Which of the town's triangles are a house's: its walls, which go, made
    // again round the holes for its door and windows; its roof and gables,
    // drawn from now on as the house's own, the Square House's, and so taken
    // out of what of the town is drawn, though still stood on; and what each
    // of those is made of there, which the house's own are made in. Its floor
    // stays the town's.
    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    let cell = |at: Vec2| ((at.x / 16.0).floor() as i32, (at.y / 16.0).floor() as i32);
    for (index, house) in houses.iter().enumerate() {
        grid.entry(cell(house.middle)).or_default().push(index);
    }
    let reach = (shape.bounds.1 - shape.bounds.0).xz().length() * 0.5 + 1.0;
    let into: Vec<Affine3A> = houses.iter().map(|house| house.place.inverse()).collect();
    let mut gone: Vec<Vec<bool>> = read.iter().map(|tris| vec![false; tris.len()]).collect();
    let mut hidden = gone.clone();
    // The pieces of its ceiling, the smallest first: a gable's triangle, in
    // the box round the roof as well, is the gable's.
    let volume = |(low, high): (Vec3, Vec3)| (high - low).max(Vec3::ZERO).element_product();
    let mut smallest: Vec<usize> = (0..shape.ceiling.len()).collect();
    smallest.sort_by(|&a, &b| volume(shape.ceiling[a].fills).total_cmp(&volume(shape.ceiling[b].fills)));
    let fits = |local: &[Vec3; 3], (low, high): (Vec3, Vec3)| {
        local
            .iter()
            .all(|corner| corner.cmpge(low - SNUG).all() && corner.cmple(high + SNUG).all())
    };
    // Each house's, by how much of it: its walls', then each piece of its
    // ceiling's.
    let mut made_of: Vec<Vec<MadeOf>> = vec![vec![HashMap::new(); 1 + shape.ceiling.len()]; houses.len()];
    for (part_index, tris) in read.iter().enumerate() {
        let Some(material) = &town[part_index].material else {
            continue;
        };
        for (tri_index, (tri, _)) in tris.iter().enumerate() {
            let middle = (tri[0] + tri[1] + tri[2]).xz() / 3.0;
            let (cx, cz) = cell(middle);
            let near = (-1..=1)
                .flat_map(|dx| (-1..=1).map(move |dz| (cx + dx, cz + dz)))
                .filter_map(|key| grid.get(&key))
                .flatten()
                .copied()
                .filter(|&house| houses[house].middle.distance(middle) <= reach);
            for house in near {
                let local = tri.map(|c| into[house].transform_point3(c));
                let of = if shape.walls.iter().any(|&wall| walled(&local, wall)) {
                    gone[part_index][tri_index] = true;
                    0
                } else if let Some(&piece) = smallest.iter().find(|&&piece| fits(&local, shape.ceiling[piece].fills)) {
                    hidden[part_index][tri_index] = true;
                    1 + piece
                } else {
                    continue;
                };
                let area = (tri[1] - tri[0]).cross(tri[2] - tri[0]).length() * 0.5;
                made_of[house][of].entry(material.id()).or_insert((0.0, material.clone())).0 += area;
                break;
            }
        }
    }

    // Each house, with its parts: its roof and gables, its walls with holes
    // for its door and windows, a wall at a time, the frames of each wall's
    // windows, its glass and its door. Meshes the same in many houses are
    // made once and shared, so that they are drawn together.
    let mut shared: HashMap<String, Handle<Mesh>> = HashMap::new();
    let mut solids: Vec<Vec<[Vec3; 3]>> = Vec::new();
    let arrives = island::arrival(Venue::Town, 0).translation.xz();
    let mut builder_at: Option<(f32, Transform)> = None;
    let mut doors = 0;
    for (index, house) in houses.iter().enumerate() {
        let Some(material) = most(&made_of[index][0]) else {
            continue;
        };
        // The wall nearest a road, out from the middle of its outer face.
        let front = (0..walls.len())
            .filter_map(|w| {
                let wall = &walls[w];
                let (start, end) = wall.span();
                let out = -wall.way_in();
                let from = wall.point((start + end) * 0.5, wall.low.y, wall.thin().1 * out.dot(wall.across()));
                let (from, way) = (house.place.transform_point3(from).xz(), house.place.transform_vector3(out).xz());
                (1..=(ROAD_REACH / ROAD_STEP) as usize)
                    .map(|step| step as f32 * ROAD_STEP)
                    .find(|&far| on_road(from + way * far))
                    .map(|far| (far, w))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
            .map_or(0, |(_, w)| w);
        let picks = mix(house.middle);
        let door = def(DOORS[picks as usize % DOORS.len()]);
        let aside = if (picks >> 8).is_multiple_of(2) { SQUARE } else { WIDE };
        let root = commands
            .spawn((
                Name::new("Town house"),
                TownHouse,
                Venue::Town,
                Transform::from_matrix(Mat4::from(house.place)),
                Visibility::Hidden,
                Rooms(vec![shape.floor]),
                Bounds {
                    low: shape.bounds.0,
                    high: shape.bounds.1,
                },
            ))
            .id();
        // Its roof and gables, the Square House's, in the town's own colours
        // there.
        for (piece, ceiling) in shape.ceiling.iter().enumerate() {
            let Some(own) = most(&made_of[index][1 + piece]).or_else(|| ceiling.material.clone()) else {
                continue;
            };
            commands.spawn((
                Mesh3d(ceiling.mesh.clone()),
                MeshMaterial3d(own),
                Transform::from_matrix(Mat4::from(ceiling.place)),
                ChildOf(root),
            ));
        }
        let mut glass: HashMap<AssetId<StandardMaterial>, (Handle<StandardMaterial>, Gathered)> = HashMap::new();
        let mut by_wall = Vec::new();
        let mut layout = Vec::new();
        for (w, wall) in walls.iter().enumerate() {
            let (start, end) = wall.span();
            let middle = (start + end) * 0.5;
            let mut asks: Vec<(&'static PieceDef, f32)> = Vec::new();
            if w == front {
                let half = openings[door.kind].span.half + openings[SQUARE].span.half + APART + 0.5;
                asks.extend([(door, middle), (def(SQUARE), middle - half), (def(SQUARE), middle + half)]);
            } else {
                let opposite = walls[front].way_in().dot(wall.way_in()) < -0.5;
                asks.push((def(if opposite { WIDE } else { aside }), middle));
            }
            let mut others: Vec<(OnWall, Span, bool)> = Vec::new();
            let mut holes: Vec<[f32; 4]> = Vec::new();
            let mut frames: HashMap<AssetId<StandardMaterial>, (Handle<StandardMaterial>, Gathered)> = HashMap::new();
            let mut panes: HashMap<AssetId<StandardMaterial>, (Handle<StandardMaterial>, Gathered)> = HashMap::new();
            let mut here: Vec<String> = Vec::new();
            let mut door_here: Option<(&'static PieceDef, Transform, OnWall)> = None;
            for (piece, along) in asks {
                let opening = &openings[piece.kind];
                let height = if piece.mount == Mount::Door {
                    Height::Share { share: 0.0, near: wall.low.y }
                } else {
                    Height::Share { share: WINDOW_SILL, near: wall.low.y + 1.0 }
                };
                let Some((along, up)) = wall.fit(piece.mount, opening.span, height, along, 1.0, &others) else {
                    continue;
                };
                let on = OnWall { wall: Entity::PLACEHOLDER, along, up, face: 1.0 };
                others.push((on, opening.span, true));
                holes.push([along - opening.span.half, up - opening.span.below, along + opening.span.half, up + opening.span.above]);
                here.push(format!("{} {along:?} {up:?}", piece.kind));
                let local = wall.place(true, on).compute_affine();
                let place = house.place * local;
                if piece.mount == Mount::Door {
                    let world = Transform::from_matrix(Mat4::from(place));
                    let stands = place.transform_point3(Vec3::NEG_Z * (wall.thin().1 + AT_THE_DOOR));
                    let away = house.middle.distance(arrives);
                    if builder_at.is_none_or(|(nearest, _)| away < nearest - 1e-3) {
                        builder_at = Some((away, world.with_translation(stands)));
                    }
                    door_here = Some((piece, Transform::from_matrix(Mat4::from(local)), on));
                    continue;
                }
                // A window: its frame and glass, in the house's own space, and
                // its pane shut to bodies.
                for (mesh, part_material, in_window) in &opening.parts {
                    let (Some(asset), Some(part_material)) = (meshes.get(mesh), part_material) else {
                        continue;
                    };
                    let at = local * *in_window;
                    let blended = materials.get(part_material).is_some_and(|look| look.alpha_mode == AlphaMode::Blend);
                    if blended {
                        for gathering in [&mut panes, &mut glass] {
                            gathering
                                .entry(part_material.id())
                                .or_insert_with(|| (part_material.clone(), Gathered::default()))
                                .1
                                .add(asset, &at);
                        }
                    } else {
                        frames
                            .entry(part_material.id())
                            .or_insert_with(|| (part_material.clone(), Gathered::default()))
                            .1
                            .add(asset, &at);
                    }
                }
                let pane = pane(opening.span, wall);
                solids.push(triangles(&pane, &place).into_iter().map(|(t, _)| t).collect());
            }
            // The wall, cut round them.
            holes.sort_by(|a, b| a[0].total_cmp(&b[0]));
            let mut cut = Wall {
                low: wall.low,
                high: wall.high,
                inside: wall.inside,
                mesh: Handle::default(),
                holes,
                storeys: wall.storeys.clone(),
                joins: wall.joins.clone(),
            };
            let made = wall_mesh(&cut);
            solids.push(triangles(&made, &house.place).into_iter().map(|(t, _)| t).collect());
            cut.mesh = shared_mesh(&mut shared, &mut meshes, format!("wall {w} {:?}", cut.holes), || made);
            let wall_entity = commands
                .spawn((
                    Mesh3d(cut.mesh.clone()),
                    cut,
                    TownWall,
                    MeshMaterial3d(material.clone()),
                    Transform::IDENTITY,
                    ChildOf(root),
                ))
                .id();
            // The frames of its windows, which are in the way as the wall is,
            // and go see-through with it (`see_through`), and their glass.
            let here = here.join(", ");
            if !frames.is_empty() || !panes.is_empty() {
                let windows = commands
                    .spawn((
                        Name::new("Windows"),
                        OnWall { wall: wall_entity, along: 0.0, up: 0.0, face: 1.0 },
                        Transform::IDENTITY,
                        Visibility::default(),
                        ChildOf(root),
                    ))
                    .id();
                for (part_material, gathered) in frames.into_values() {
                    let key = format!("frames {w} {:?} {here}", part_material.id());
                    let mesh = shared_mesh(&mut shared, &mut meshes, key, || gathered.mesh());
                    commands.spawn((Mesh3d(mesh), MeshMaterial3d(part_material), Transform::IDENTITY, ChildOf(windows)));
                }
                for (part_material, gathered) in panes.into_values() {
                    let key = format!("glass {w} {:?} {here}", part_material.id());
                    by_wall.push((windows, shared_mesh(&mut shared, &mut meshes, key, || gathered.mesh()), part_material));
                }
            }
            if let Some((piece, at, on)) = door_here {
                commands
                    .spawn((
                        Name::new(piece.name),
                        TownDoor(piece),
                        WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(piece.model()))),
                        at,
                        OnWall { wall: wall_entity, ..on },
                        ChildOf(root),
                    ))
                    .observe(door_ready);
                doors += 1;
            }
            layout.push(format!("{w}: {here}"));
        }
        // Its glass, as one, but wall by wall while you are in it.
        let layout = layout.join("; ");
        let whole = glass
            .into_values()
            .map(|(part_material, gathered)| {
                let key = format!("glass {:?} {layout}", part_material.id());
                let mesh = shared_mesh(&mut shared, &mut meshes, key, || gathered.mesh());
                commands
                    .spawn((Mesh3d(mesh), MeshMaterial3d(part_material), Transform::IDENTITY, ChildOf(root)))
                    .id()
            })
            .collect();
        commands.entity(root).insert(Panes { whole, by_wall });
    }

    // The town's own walls taken out of what of it is drawn and stood on, and
    // its houses' roofs and gables out of what of it is drawn.
    let mut corners: Vec<[Vec3; 3]> = Vec::new();
    for (part_index, part) in town.iter().enumerate() {
        let mut drawn: Vec<[u32; 3]> = Vec::new();
        for ((&(tri, at), &went), &covered) in read[part_index].iter().zip(&gone[part_index]).zip(&hidden[part_index]) {
            if went {
                continue;
            }
            corners.push(tri);
            if !covered {
                drawn.push(at);
            }
        }
        if drawn.len() < read[part_index].len()
            && let Some(mut mesh) = meshes.get_mut(&part.mesh)
        {
            mesh.insert_indices(Indices::U32(drawn.into_iter().flatten().collect()));
        }
    }
    let taken = gone.iter().flatten().filter(|gone| **gone).count();
    let roofed = hidden.iter().flatten().filter(|hidden| **hidden).count();
    islands.reshape(Venue::Town, corners, &solids);
    if let Some((_, at)) = builder_at {
        crate::builder::stand_house_builder(&mut commands, &assets, at);
    }
    info!(
        "the town's houses: {} of them, {doors} doors, their own walls' {taken} triangles made again round the \
         holes and their roofs' and gables' {roofed} drawn as each house's own, from {} meshes (in {} ms)",
        houses.len(),
        shared.len(),
        started.elapsed().as_millis()
    );
}

/// What a house's walls, or a piece of its ceiling, are made of in the town:
/// each material, by how much of them it covers, in square metres.
type MadeOf = HashMap<AssetId<StandardMaterial>, (f32, Handle<StandardMaterial>)>;

/// The material that covers the most of `made`.
fn most(made: &MadeOf) -> Option<Handle<StandardMaterial>> {
    made.values().max_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, material)| material.clone())
}

/// The mesh made by `make`, or the one made already for the same `key`: the
/// same mesh in many houses is one, drawn together.
fn shared_mesh(
    shared: &mut HashMap<String, Handle<Mesh>>,
    meshes: &mut Assets<Mesh>,
    key: String,
    make: impl FnOnce() -> Mesh,
) -> Handle<Mesh> {
    shared.entry(key).or_insert_with(|| meshes.add(make())).clone()
}

/// While you are in one of the town's houses, its glass a wall at a time, so
/// that each wall's goes see-through with the wall (`see_through`), and as
/// one again once you are not.
fn open_the_house(
    opened: Query<&Panes, (With<TownHouse>, Added<YouAreIn>)>,
    mut left: RemovedComponents<YouAreIn>,
    houses: Query<&Panes, With<TownHouse>>,
    children: Query<&Children>,
    wall_panes: Query<(), With<WallPane>>,
    mut commands: Commands,
) {
    for panes in &opened {
        for &whole in &panes.whole {
            commands.entity(whole).insert(Visibility::Hidden);
        }
        for (windows, mesh, material) in &panes.by_wall {
            commands.spawn((
                WallPane,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::IDENTITY,
                ChildOf(*windows),
            ));
        }
    }
    for house in left.read() {
        let Ok(panes) = houses.get(house) else {
            continue;
        };
        for &whole in &panes.whole {
            commands.entity(whole).insert(Visibility::Inherited);
        }
        for (windows, ..) in &panes.by_wall {
            for pane in children.iter_descendants(*windows).filter(|&pane| wall_panes.contains(pane)) {
                commands.entity(pane).despawn();
            }
        }
    }
}

/// A town door once its model has spawned: what of its wall its frame takes
/// up, and its leaves, to swing (`swing_doors`).
#[allow(clippy::too_many_arguments)]
fn door_ready(
    ready: On<WorldInstanceReady>,
    doors: Query<&TownDoor>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    turns: Query<&Transform>,
    parts: Query<&Mesh3d>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    mut commands: Commands,
) {
    let root = ready.entity;
    let (Ok(TownDoor(piece)), Ok(root_place)) = (doors.get(root), placement.compute_global_transform(root)) else {
        return;
    };
    let into_root = root_place.affine().inverse();
    let mut filled = HashMap::new();
    let mut frame: Option<(Vec3, Vec3)> = None;
    for part in children.iter_descendants(root) {
        let (Ok(mesh), Ok(place)) = (parts.get(part), placement.compute_global_transform(part)) else {
            continue;
        };
        let Some((low, high)) = meshes.get(&mesh.0).and_then(|mesh| extent(mesh, into_root * place.affine())) else {
            continue;
        };
        filled.insert(part, (low, high));
        let framed = [Some(part), parents.get(part).ok().map(ChildOf::parent)]
            .into_iter()
            .flatten()
            .any(|entity| names.get(entity).is_ok_and(|name| name.as_str().starts_with("Frame")));
        if framed {
            frame = Some(frame.map_or((low, high), |(a, b)| (a.min(low), b.max(high))));
        }
    }
    let Some((low, high)) = frame else {
        return;
    };
    commands.entity(root).insert(Span::of(low, high));
    if let Some(swing) = piece.swing
        && let Some(doorway) = doorway(root, swing, &filled, &children, &parents, &names, &turns, &placement)
    {
        commands.entity(root).insert(doorway);
    }
}
