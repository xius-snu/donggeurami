//! The town's car, driving round its roads by itself, which anyone can jump
//! onto and ride (Hajun, 2026-10-23).
//!
//! The car is Hajun's model, `assets/mybuilds/car.glb`, and it is the shape of
//! its model: what bodies stand on and bump into is every triangle of it,
//! read once it has spawned ([`Hull`]), and carried round with it as an
//! island of its own that moves. Seats, windows and a roof put in the model
//! are seats to stand on, windows to bump into and a roof over your head, as
//! a house's are. Its front is the end of its longer side that is Blender's
//! -X, as it is modelled, or -Y, Blender's front, as every other model faces.
//!
//! Where it goes is the town's roads, every lane of every one of them, read
//! off the town's model (`roads`). Where it is goes by the clock, as the
//! fountain's jet does: its way round takes a whole number of seconds, and it
//! is that far into it at every moment, by the time since 1970 ([`place_at`]),
//! so that everyone in the town sees it in the same place with nothing about
//! it going over the network. `RT_CAR_AT` (seconds into its way round) stops
//! it there, for trying things out.
//!
//! Standing on it, or in the air off it, a body is carried along with it
//! (`crate::move_bodies`, `crate::Riding`): every move it makes, the body
//! makes with it, turning as it turns, until the body comes down on anything
//! else. Online, where a rider is goes to everyone else as a place on the car
//! (`roundtown_net::Spot::aboard`), and they put the rider on the car as they
//! see it (`net`): two people riding it are seen riding it, and nobody is
//! seen gliding along the road beside it.

use std::f32::consts::TAU;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::ecs::system::SystemParam;
use bevy::gltf::GltfMaterialName;
use bevy::math::Affine3A;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use crate::island::{self, Island, Islands, Venue};
use crate::roads::{self, Paint, Route};

/// The car's model, under `assets/`.
pub(crate) const MODEL: &str = "mybuilds/car.glb";

/// What the car's wheels are called in its model: every part whose name
/// begins with it. Each wheel turns about its own middle as the car goes,
/// however many of them are one object in Blender.
const WHEEL: &str = "wheel";

/// The car: the number it goes by online.
#[derive(Component)]
pub(crate) struct Car {
    pub(crate) number: u8,
}

/// The car's shape, read off its model once it has spawned, in its own space.
#[derive(Component)]
pub(crate) struct Hull {
    /// Every triangle of it, to stand on and bump into.
    pub(crate) shape: Island,
    /// Which way its front faces, in its own space, across the ground.
    front: Vec2,
    /// The middle of its footprint, in its own space, across the ground: what
    /// goes down the middle of its lane.
    middle: Vec2,
    /// How high the lowest of it is, in its own space: the bottoms of its
    /// wheels, which stand on the road.
    lowest: f32,
    /// How far from its own middle any of it reaches, across the ground.
    pub(crate) reach: f32,
}

/// Where something stands and which way it is turned about the upright.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Place {
    pub(crate) at: Vec3,
    pub(crate) turn: Quat,
}

impl Default for Place {
    fn default() -> Self {
        Self {
            at: Vec3::ZERO,
            turn: Quat::IDENTITY,
        }
    }
}

impl Place {
    /// Where `point`, in the world, is in this place's own space.
    pub(crate) fn inward(&self, point: Vec3) -> Vec3 {
        self.turn.inverse() * (point - self.at)
    }

    /// Where `point`, in this place's own space, is in the world.
    pub(crate) fn outward(&self, point: Vec3) -> Vec3 {
        self.at + self.turn * point
    }

    /// `transform`, in the world, as it stands in this place's own space.
    pub(crate) fn local(&self, transform: &Transform) -> Transform {
        Transform::from_translation(self.inward(transform.translation))
            .with_rotation((self.turn.inverse() * transform.rotation).normalize())
    }

    /// `transform`, in this place's own space, as it stands in the world.
    pub(crate) fn world(&self, transform: &Transform) -> Transform {
        Transform::from_translation(self.outward(transform.translation))
            .with_rotation((self.turn * transform.rotation).normalize())
    }
}

/// Where the car stands this frame and where it stood the frame before, and
/// the clock it was placed by: a body on it moves as it did between them.
#[derive(Component, Clone, Copy, Default)]
pub(crate) struct Driven {
    pub(crate) now: Place,
    pub(crate) then: Place,
    /// The seconds since 1970 it was placed by this frame.
    seconds: f64,
    /// How far it has come since 1970, for its wheels.
    distance: f64,
    placed: bool,
}

/// The clock the car goes by: the time since 1970, or stopped at one point of
/// its way round, from `RT_CAR_AT`.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct CarClock {
    stopped: Option<f64>,
}

impl CarClock {
    fn from_env() -> Self {
        Self {
            stopped: std::env::var("RT_CAR_AT")
                .ok()
                .and_then(|said| said.trim().parse().ok()),
        }
    }

    /// The seconds the car goes by now.
    fn now(&self) -> f64 {
        self.stopped.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0.0, |since| since.as_secs_f64())
        })
    }

    /// The seconds it went by `ago` seconds before `now`: the same, if it is
    /// stopped.
    fn before(&self, now: f64, ago: f64) -> f64 {
        if self.stopped.is_some() { now } else { now - ago }
    }
}

/// What the town's model has to say about its roads, kept from the moment it
/// spawns until its shape has been read too and the car's way round can be
/// worked out.
#[derive(Resource)]
struct TownRoads {
    road: Vec<[Vec3; 3]>,
    markings: Vec<Vec<[Vec3; 3]>>,
}

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(CarClock::from_env())
        .add_observer(read_roads)
        .add_systems(PreUpdate, drive)
        .add_systems(Update, (plan_the_way, spawn_car, turn_wheels).chain());
}

/// Reads the town's roads out of its model once it has spawned: their
/// surfaces, and the paint down their middles, each mesh of it on its own.
fn read_roads(
    ready: On<WorldInstanceReady>,
    venues: Query<&Venue, Without<Car>>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, &GltfMaterialName), Without<SkinnedMesh>>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    route: Option<Res<Route>>,
    mut commands: Commands,
) {
    if route.is_some() || venues.get(ready.entity).ok() != Some(&Venue::Town) {
        return;
    }
    let mut road = Vec::new();
    let mut markings = Vec::new();
    for entity in children.iter_descendants(ready.entity) {
        let Ok((mesh, material)) = parts.get(entity) else {
            continue;
        };
        let (Some(mesh), Ok(place)) = (meshes.get(&mesh.0), placement.compute_global_transform(entity)) else {
            continue;
        };
        if material.0 == roads::ROAD {
            island::add_mesh(&mut road, mesh, &place);
        } else if material.0 == roads::MARKING {
            let mut corners = Vec::new();
            island::add_mesh(&mut corners, mesh, &place);
            markings.push(corners);
        }
    }
    // The town's players stand on it too, each a model of their own, with no
    // roads in it.
    if road.is_empty() || markings.is_empty() {
        return;
    }
    commands.insert_resource(TownRoads { road, markings });
}

/// Works out the car's way round the town once both its roads and its shape
/// have been read.
fn plan_the_way(roads: Option<Res<TownRoads>>, islands: Res<Islands>, mut commands: Commands) {
    let Some(roads) = roads else {
        return;
    };
    let Some(land) = islands.ground(Venue::Town) else {
        return;
    };
    commands.remove_resource::<TownRoads>();
    let road = Island::new(&roads.road);
    let paint = Paint {
        land,
        road: &road,
        markings: &roads.markings,
    };
    match Route::read(&paint) {
        Some(route) => {
            info!(
                "the car's way round the town: {} lanes, {:.0} m, in {} s",
                route.lanes(),
                route.length(),
                route.lap()
            );
            commands.insert_resource(route);
        }
        None => warn!("no way round the town's roads, so no car drives them"),
    }
}

/// The car, once it has its way round.
fn spawn_car(
    route: Option<Res<Route>>,
    cars: Query<(), With<Car>>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    if route.is_none() || !cars.is_empty() {
        return;
    }
    // On the town, drawn there and met there, once its shape has been read
    // and it has been put on the road: until then it is nowhere.
    commands
        .spawn((
            Name::new("Car"),
            Car { number: 0 },
            Driven::default(),
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(MODEL))),
            Transform::default(),
            Visibility::Hidden,
        ))
        .observe(read_car);
}

/// Reads the car's shape out of its model once it has spawned, every mesh of
/// it in the car's own space, and gives each of its wheels a middle of its
/// own to turn about.
///
/// Each object in Blender is one part of its shape, all its materials
/// together, as the glTF loader makes a mesh of each: the body is closed only
/// as a whole, and its paint alone, round the seats sunk into it, would be a
/// box with no top over them, and the seats taken for the inside of it
/// (`Island::blocked`).
#[allow(clippy::too_many_arguments)]
fn read_car(
    ready: On<WorldInstanceReady>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>), Without<SkinnedMesh>>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    placement: TransformHelper,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let car = ready.entity;
    let Ok(root) = placement.compute_global_transform(car) else {
        return;
    };
    let into_car = root.affine().inverse();
    // Each object's triangles, by the object.
    let mut pieces: Vec<(Option<Entity>, Vec<[Vec3; 3]>)> = Vec::new();
    let mut wheels: Vec<(Entity, Handle<Mesh>, Option<Handle<StandardMaterial>>, Affine3A)> = Vec::new();
    for part in children.iter_descendants(car) {
        let Ok((mesh, material)) = parts.get(part) else {
            continue;
        };
        let Ok(place) = placement.compute_global_transform(part) else {
            continue;
        };
        let in_car = GlobalTransform::from(into_car * place.affine());
        let object = parents.get(part).ok().map(ChildOf::parent);
        let mut corners = Vec::new();
        let read = meshes
            .get(&mesh.0)
            .is_some_and(|asset| island::add_mesh(&mut corners, asset, &in_car));
        if !read {
            warn!("{MODEL}: could not read one of its meshes, so bodies will pass through it");
            continue;
        }
        match pieces.iter_mut().find(|(of, _)| *of == object) {
            Some((_, triangles)) => triangles.extend(corners),
            None => pieces.push((object, corners)),
        }
        // A wheel is a mesh of an object called that in Blender.
        let is_wheel = [Some(part), object].into_iter().flatten().any(|entity| {
            names
                .get(entity)
                .is_ok_and(|name| name.as_str().to_lowercase().starts_with(WHEEL))
        });
        if is_wheel {
            wheels.push((part, mesh.0.clone(), material.map(|m| m.0.clone()), in_car.affine()));
        }
    }
    let all: Vec<Vec3> = pieces.iter().flat_map(|(_, triangles)| triangles.iter().flatten()).copied().collect();
    if all.is_empty() {
        warn!("{MODEL} has nothing in it to stand on");
        return;
    }
    let (low, high) = all
        .iter()
        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), &corner| {
            (low.min(corner), high.max(corner))
        });
    let parts: Vec<&[[Vec3; 3]]> = pieces.iter().map(|(_, triangles)| triangles.as_slice()).collect();
    let hull = Hull {
        shape: Island::of_parts(&parts),
        front: front(low, high),
        middle: ((low + high) * 0.5).xz(),
        lowest: low.y,
        reach: all.iter().map(|corner| corner.xz().length()).fold(0.0, f32::max),
    };
    let across = hull.front.perp();
    let axle = Vec3::new(across.x, 0.0, across.y);
    for wheel in split_wheels(&wheels, &mut meshes) {
        let Wheel { middle, radius, meshes: parts } = wheel;
        let pivot = commands
            .spawn((
                Name::new("Car wheel"),
                Turns { axle, radius },
                Transform::from_translation(middle),
                Visibility::Inherited,
                ChildOf(car),
            ))
            .id();
        for (mesh, material) in parts {
            let mut part = commands.spawn((Mesh3d(mesh), Transform::default(), ChildOf(pivot)));
            if let Some(material) = material {
                part.insert(MeshMaterial3d(material));
            }
        }
    }
    // The wheels as modelled are drawn no more: their turning copies are.
    for (part, ..) in &wheels {
        commands.entity(*part).insert(Visibility::Hidden);
    }
    commands.entity(car).insert((hull, Venue::Town));
}

/// Which way the front of a car whose model fits between `low` and `high`, in
/// its own space, faces across the ground: along its longer side, toward
/// Blender's -X, the hood of Hajun's car, or -Y, Blender's front, whichever it
/// is longer along. Blender's -X is +x here and its -Y is -z: the glTF loader
/// turns models half round (`GltfConvertCoordinates`).
fn front(low: Vec3, high: Vec3) -> Vec2 {
    let size = high - low;
    if size.x >= size.z { Vec2::X } else { Vec2::NEG_Y }
}

/// Where the car is, and which way it is turned, `seconds` since 1970: its
/// footprint's middle on the middle of its lane, its front the way the lane
/// goes, and its wheels on the road.
pub(crate) fn place_at(route: &Route, hull: &Hull, seconds: f64) -> (Place, f64) {
    let (on_road, way, distance) = route.at(seconds);
    let turn = Quat::from_rotation_y(heading(way) - heading(hull.front));
    let middle = turn * Vec3::new(hull.middle.x, 0.0, hull.middle.y);
    let at = on_road - middle - Vec3::Y * hull.lowest;
    (Place { at, turn }, distance)
}

/// The turn about the upright that has -z face `way`, across the ground.
fn heading(way: Vec2) -> f32 {
    (-way.x).atan2(-way.y)
}

/// Puts the car where the clock says it is, keeping where it was the frame
/// before for whoever is riding it. Only what has changed is written.
fn drive(
    clock: Res<CarClock>,
    route: Option<Res<Route>>,
    mut cars: Query<(&Hull, &mut Transform, &mut Driven), With<Car>>,
) {
    let Some(route) = route else {
        return;
    };
    let seconds = clock.now();
    for (hull, mut transform, mut driven) in &mut cars {
        let (place, distance) = place_at(&route, hull, seconds);
        let then = if driven.placed { driven.now } else { place };
        *driven = Driven {
            now: place,
            then,
            seconds,
            distance,
            placed: true,
        };
        transform.set_if_neq(Transform::from_translation(place.at).with_rotation(place.turn));
    }
}

/// One of the car's wheels: where its middle is in the car's own space, how
/// big round it is, and its meshes, one to each material, made about its
/// middle.
struct Wheel {
    middle: Vec3,
    radius: f32,
    meshes: Vec<(Handle<Mesh>, Option<Handle<StandardMaterial>>)>,
}

/// A wheel's turning, about its axle, a line across the car through its
/// middle, as fast as the car goes over its radius.
#[derive(Component)]
struct Turns {
    axle: Vec3,
    radius: f32,
}

/// The car's wheels, among the meshes of its parts called [`WHEEL`]: each
/// mesh, placed in the car's own space by `in_car`, broken into the pieces
/// its triangles make, and the pieces of all of them that are round about the
/// same middle taken together as one wheel. Each gets a copy of each mesh
/// with only its own triangles in it, made about its middle.
fn split_wheels(
    parts: &[(Entity, Handle<Mesh>, Option<Handle<StandardMaterial>>, Affine3A)],
    meshes: &mut Assets<Mesh>,
) -> Vec<Wheel> {
    // Each piece: which part, its triangles by their corners' numbers, and
    // the box round it in the car's space.
    let mut pieces: Vec<(usize, Vec<u32>, Vec3, Vec3)> = Vec::new();
    for (index, (_, handle, _, in_car)) in parts.iter().enumerate() {
        let Some(mesh) = meshes.get(handle) else {
            continue;
        };
        let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            continue;
        };
        let points: Vec<Vec3> = positions
            .iter()
            .map(|&corner| in_car.transform_point3(Vec3::from(corner)))
            .collect();
        let order: Vec<u32> = match mesh.indices() {
            Some(indices) => indices.iter().map(|at| at as u32).collect(),
            None => (0..points.len() as u32).collect(),
        };
        for triangles in loose_pieces(&points, &order) {
            let (low, high) = triangles
                .iter()
                .map(|&corner| points[corner as usize])
                .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), p| (low.min(p), high.max(p)));
            pieces.push((index, triangles, low, high));
        }
    }
    let boxes: Vec<(Vec3, Vec3)> = pieces.iter().map(|&(_, _, low, high)| (low, high)).collect();
    gather(&boxes)
        .into_iter()
        .map(|(low, high, members)| {
            let middle = (low + high) * 0.5;
            let made = parts
                .iter()
                .enumerate()
                .filter_map(|(index, (_, handle, material, in_car))| {
                    let triangles: Vec<u32> = members
                        .iter()
                        .filter(|&&piece| pieces[piece].0 == index)
                        .flat_map(|&piece| pieces[piece].1.iter().copied())
                        .collect();
                    if triangles.is_empty() {
                        return None;
                    }
                    let mesh = meshes.get(handle)?;
                    let about = Affine3A::from_translation(-middle) * *in_car;
                    Some((meshes.add(part_of(mesh, &triangles, &about)), material.clone()))
                })
                .collect();
            Wheel {
                middle,
                radius: ((high.y - low.y) * 0.5).max(0.05),
                meshes: made,
            }
        })
        .collect()
}

/// The pieces whose boxes, from their lowest corner to their highest, are
/// `boxes`, taken together where they are about the same middle: as near to
/// it as half the size of the bigger box. Each lot as the box round it all
/// and which of the pieces it is.
fn gather(boxes: &[(Vec3, Vec3)]) -> Vec<(Vec3, Vec3, Vec<usize>)> {
    let mut lots: Vec<(Vec3, Vec3, Vec<usize>)> = Vec::new();
    for (piece, &(low, high)) in boxes.iter().enumerate() {
        let middle = (low + high) * 0.5;
        let near = lots.iter_mut().find(|(l, h, _)| {
            let size = (*h - *l).max_element().max((high - low).max_element());
            ((*l + *h) * 0.5).distance(middle) < size * 0.5
        });
        match near {
            Some((l, h, members)) => {
                *l = l.min(low);
                *h = h.max(high);
                members.push(piece);
            }
            None => lots.push((low, high, vec![piece])),
        }
    }
    lots
}

/// The pieces the triangles `order` make of the corners `points`, joined
/// where they share a corner, or one in the same place: each as the numbers
/// of its triangles' corners, three to a triangle.
fn loose_pieces(points: &[Vec3], order: &[u32]) -> Vec<Vec<u32>> {
    let key = |at: u32| {
        let p = (points[at as usize] * 1e4).round().as_ivec3();
        (p.x, p.y, p.z)
    };
    let triangles: Vec<&[u32]> = order.chunks_exact(3).collect();
    let mut parent: Vec<usize> = (0..triangles.len()).collect();
    fn root(parent: &mut [usize], mut at: usize) -> usize {
        while parent[at] != at {
            parent[at] = parent[parent[at]];
            at = parent[at];
        }
        at
    }
    let mut owner = std::collections::HashMap::new();
    for (index, triangle) in triangles.iter().enumerate() {
        for &corner in *triangle {
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
    let mut pieces: Vec<(usize, Vec<u32>)> = Vec::new();
    for (index, triangle) in triangles.iter().enumerate() {
        let top = root(&mut parent, index);
        match pieces.iter_mut().find(|(first, _)| *first == top) {
            Some((_, corners)) => corners.extend_from_slice(triangle),
            None => pieces.push((top, triangle.to_vec())),
        }
    }
    pieces.into_iter().map(|(_, corners)| corners).collect()
}

/// A copy of `mesh` with only the triangles `order` names in it, every corner
/// moved by `about`: its points, its normals and its tangents, and whatever
/// else it has of each corner as it is.
fn part_of(mesh: &Mesh, order: &[u32], about: &Affine3A) -> Mesh {
    let mut kept: Vec<u32> = order.to_vec();
    kept.sort_unstable();
    kept.dedup();
    let renumber = |at: u32| kept.binary_search(&at).unwrap() as u32;
    // Normals go by the inverse of the turn and stretch, transposed, so that
    // a part stretched one way still has them square to its faces.
    let normals = about.matrix3.inverse().transpose();
    let mut part = Mesh::new(mesh.primitive_topology(), mesh.asset_usage);
    for (attribute, values) in mesh.attributes() {
        let picked = match values {
            VertexAttributeValues::Float32x3(all) => {
                let pick = kept.iter().map(|&at| Vec3::from(all[at as usize]));
                VertexAttributeValues::Float32x3(if attribute.id == Mesh::ATTRIBUTE_POSITION.id {
                    pick.map(|p| about.transform_point3(p).to_array()).collect()
                } else if attribute.id == Mesh::ATTRIBUTE_NORMAL.id {
                    pick.map(|n| (normals * Vec3A::from(n)).normalize_or_zero().to_array()).collect()
                } else {
                    pick.map(|value| value.to_array()).collect()
                })
            }
            VertexAttributeValues::Float32x2(all) => {
                VertexAttributeValues::Float32x2(kept.iter().map(|&at| all[at as usize]).collect())
            }
            VertexAttributeValues::Float32x4(all) => {
                let pick = kept.iter().map(|&at| all[at as usize]);
                VertexAttributeValues::Float32x4(if attribute.id == Mesh::ATTRIBUTE_TANGENT.id {
                    pick.map(|[x, y, z, w]| {
                        let way = about.transform_vector3(Vec3::new(x, y, z)).normalize_or_zero();
                        [way.x, way.y, way.z, w]
                    })
                    .collect()
                } else {
                    pick.collect()
                })
            }
            _ => continue,
        };
        part.insert_attribute(attribute.clone(), picked);
    }
    part.insert_indices(Indices::U32(order.iter().map(|&at| renumber(at)).collect()));
    part
}

/// Turns each wheel as far round as the car has gone over its radius. Only
/// what has changed is written.
fn turn_wheels(
    cars: Query<(&Driven, &Children), With<Car>>,
    mut wheels: Query<(&Turns, &mut Transform)>,
) {
    for (driven, children) in &cars {
        for child in children.iter() {
            let Ok((turns, mut transform)) = wheels.get_mut(child) else {
                continue;
            };
            let round = TAU as f64 * f64::from(turns.radius);
            let angle = (driven.distance.rem_euclid(round) / f64::from(turns.radius)) as f32;
            // Rolling forward, the top of the wheel goes the way the car does.
            let rotation = Quat::from_axis_angle(turns.axle, -angle);
            if transform.rotation != rotation {
                transform.rotation = rotation;
            }
        }
    }
}

/// The cars, for everything that has to say where a rider is on one: where
/// each is now, and where it was a moment ago.
#[derive(SystemParam)]
pub(crate) struct Cars<'w, 's> {
    route: Option<Res<'w, Route>>,
    clock: Res<'w, CarClock>,
    cars: Query<'w, 's, (Entity, &'static Car, &'static Hull, &'static Driven)>,
}

impl Cars<'_, '_> {
    /// The car numbered `number`, and where it is now.
    pub(crate) fn numbered(&self, number: u8) -> Option<(Entity, Place)> {
        self.cars
            .iter()
            .find(|(_, car, ..)| car.number == number)
            .map(|(entity, .., driven)| (entity, driven.now))
    }

    /// The number of the car `entity`, and where it is now.
    pub(crate) fn of(&self, entity: Entity) -> Option<(u8, Place)> {
        self.cars
            .get(entity)
            .ok()
            .map(|(_, car, _, driven)| (car.number, driven.now))
    }

    /// Where the car numbered `number` was `ago` seconds before now.
    pub(crate) fn before(&self, number: u8, ago: f64) -> Option<Place> {
        let route = self.route.as_ref()?;
        let (_, _, hull, driven) = self.cars.iter().find(|(_, car, ..)| car.number == number)?;
        Some(place_at(route, hull, self.clock.before(driven.seconds, ago)).0)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Hajun's car, as `car.glb` has it, in its own space.
    pub(crate) fn hajuns_car() -> (Vec<Vec<[Vec3; 3]>>, Vec3, Vec3) {
        let pieces = crate::island::glb::objects(&crate::island::glb::read(MODEL));
        let (low, high) = pieces
            .iter()
            .flatten()
            .flatten()
            .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), &corner| {
                (low.min(corner), high.max(corner))
            });
        (pieces, low, high)
    }

    #[test]
    fn a_car_faces_along_its_longer_side() {
        // Modelled along Blender's x, as Hajun's is, its hood at -x: +x here.
        assert_eq!(front(Vec3::new(-3.0, -0.3, -1.75), Vec3::new(3.0, 1.0, 1.75)), Vec2::X);
        // Turned to face Blender's -Y, as every other model faces: -z here.
        assert_eq!(front(Vec3::new(-1.75, -0.3, -3.0), Vec3::new(1.75, 1.0, 3.0)), Vec2::NEG_Y);
        // And Hajun's is one or the other, whatever has been done to it.
        let (_, low, high) = hajuns_car();
        let size = high - low;
        let way = front(low, high);
        assert!(if way == Vec2::X { size.x >= size.z } else { size.z > size.x });
    }

    #[test]
    fn the_car_stands_on_the_road_facing_the_way_it_goes() {
        let (land, road, markings) = crate::roads::tests::round_town();
        let route = Route::read(&Paint { land: &land, road: &road, markings: &markings }).unwrap();
        let (pieces, low, high) = hajuns_car();
        let parts: Vec<&[[Vec3; 3]]> = pieces.iter().map(Vec::as_slice).collect();
        let hull = Hull {
            shape: Island::of_parts_for_tests(&parts),
            front: front(low, high),
            middle: ((low + high) * 0.5).xz(),
            lowest: low.y,
            reach: 3.6,
        };
        for step in 0..40 {
            let seconds = 1.76e9 + step as f64 * 4.9;
            let (place, _) = place_at(&route, &hull, seconds);
            let (soon, _) = place_at(&route, &hull, seconds + 0.05);
            // Its wheels on the road, -0.2 m, and its front the way it goes.
            assert!((place.outward(Vec3::Y * low.y).y + 0.2).abs() < 0.02, "{place:?}");
            let ahead = place.turn * Vec3::X;
            let went = (soon.at - place.at).normalize_or_zero();
            if went != Vec3::ZERO {
                assert!(ahead.dot(went) > 0.9, "facing {ahead}, going {went}");
            }
            // Never faster than its top speed.
            let speed = soon.at.distance(place.at) / 0.05;
            assert!(speed < roundtown_net::cars::TOP_SPEED * 1.05, "{speed} m/s");
        }
    }

    #[test]
    fn a_place_and_its_own_space_undo_each_other() {
        let place = Place {
            at: Vec3::new(20.0, 0.1, -4.0),
            turn: Quat::from_rotation_y(0.7),
        };
        let body = Transform::from_xyz(21.0, 1.1, -3.5).with_rotation(Quat::from_rotation_y(2.0));
        let back = place.world(&place.local(&body));
        assert!(back.translation.distance(body.translation) < 1e-5);
        assert!(back.rotation.angle_between(body.rotation) < 1e-5);
    }

    #[test]
    fn the_wheels_are_four_pieces_round_their_own_middles() {
        let parts = crate::island::glb::read(MODEL);
        let wheel: Vec<&crate::island::glb::Part> =
            parts.iter().filter(|part| part.node.starts_with(WHEEL)).collect();
        assert!(!wheel.is_empty());
        // Each of its meshes broken into its pieces, and the pieces about the
        // same middle taken together: four wheels, at the car's corners.
        let mut boxes: Vec<(Vec3, Vec3)> = Vec::new();
        for part in &wheel {
            let points: Vec<Vec3> = part.triangles.iter().flatten().copied().collect();
            let order: Vec<u32> = (0..points.len() as u32).collect();
            for piece in loose_pieces(&points, &order) {
                boxes.push(
                    piece
                        .iter()
                        .map(|&at| points[at as usize])
                        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(l, h), p| (l.min(p), h.max(p))),
                );
            }
        }
        let wheels = gather(&boxes);
        assert_eq!(wheels.len(), 4, "{wheels:?}");
        for (low, high, _) in wheels {
            // Round, side on, and out at a corner of the car.
            let (tall, long) = (high.y - low.y, (high.x - low.x).max(high.z - low.z));
            assert!((tall - long).abs() < tall * 0.2, "{tall} tall, {long} long");
            let middle = (low + high) * 0.5;
            assert!(middle.x.abs() > 1.0 && middle.z.abs() > 1.0, "{middle}");
        }
    }
}
