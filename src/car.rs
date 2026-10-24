//! The town's cars, driving round its roads by themselves, which anyone can
//! get on and ride (Hajun, 2026-10-23): two buses since 2026-10-24.
//!
//! The cars are Hajun's models, [`CARS`], and each is the shape of its
//! model: what bodies stand on and bump into is every triangle of it, read
//! once it has spawned ([`Hull`]), and carried round with it as an island of
//! its own that moves. Seats, windows and a roof put in a model are seats to
//! stand on, windows to bump into and a roof over your head, as a house's
//! are. A car's front is the end of its longer side that is Blender's -Y,
//! Blender's front, as the bus and every other model faces, or -X, as the
//! first car was modelled.
//!
//! Where they go is the town's roads, every lane of every one of them, read
//! off the town's model (`roads`), one after the other round the same loop.
//! Where each is goes by the town's clock (`clock`), as the fountain's jet
//! does: the loop takes a whole number of seconds, and a car is that far into
//! it at every moment, by the seconds since 1970, less how far behind the
//! first it goes ([`place_at`]). Online the town's clock is the server's, so
//! that everyone in the town sees each car in the same place with nothing
//! about the cars going over the network but the time. `RT_CAR_AT` (seconds
//! into the loop) stops them there, for trying things out.
//!
//! Standing on a car, or in the air off it, a body is carried along with it
//! (`crate::move_bodies`, `crate::Riding`): every move it makes, the body
//! makes with it, turning as it turns, until the body comes down on anything
//! else. Online, where a rider is goes to everyone else as a place on the car
//! (`roundtown_net::Spot::aboard`), which goes by its number, and they put
//! the rider on the car as they see it (`net`): two people riding one are
//! seen riding it, and nobody is seen gliding along the road beside it.

use std::f32::consts::TAU;

use bevy::ecs::system::SystemParam;
use bevy::gltf::GltfMaterialName;
use bevy::math::Affine3A;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use crate::clock::TownClock;
use crate::island::{self, Island, Islands, Venue};
use crate::roads::{self, Paint, Route};

/// The town's cars, Hajun's models under `assets/`, each going by the number
/// it has here: two of their bus, 12.5 m long, on the one way round, the
/// second half of it behind the first (Hajun, 2026-10-24: "remove the car and
/// car2 from the circlemap and put 2 bus instead"). Their first car and their
/// Roadster, `car.glb` and `car2.glb`, which drove the roads from 2026-10-23,
/// are still in `assets/mybuilds/`, left out of the APK. Every device online
/// has to have the same cars in the same order, as a rider is sent as a place
/// on one, by its number.
pub(crate) const CARS: [&str; 2] = ["mybuilds/bus.glb", "mybuilds/bus.glb"];

/// What the car's wheels are called in its model: every part with it in its
/// name, but a steering wheel. Each wheel turns about its own middle as the
/// car goes, however many of them are one object in Blender.
const WHEEL: &str = "wheel";

/// Whether a part called `name` is one of the car's wheels.
fn is_wheel(name: &str) -> bool {
    let name = name.to_lowercase();
    name.contains(WHEEL) && !name.contains("steering")
}

/// A car: the number it goes by online, its place in [`CARS`].
#[derive(Component)]
pub(crate) struct Car {
    pub(crate) number: u8,
}

impl Car {
    /// Its model, under `assets/`.
    fn model(&self) -> &'static str {
        CARS[usize::from(self.number)]
    }
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
    /// Half its length and half its width, along its front and across it:
    /// the room it takes on the road.
    half: Vec2,
    /// How far apart its front and back wheels are, along its front: both
    /// keep to its way round (`roads::Footprint`).
    base: f32,
    /// How far ahead of its footprint's middle the middle of its wheels is,
    /// along its front: the bus's front wheels are 3.8 m ahead of its middle,
    /// and its back ones 2.8 m behind.
    wheels_ahead: f32,
    /// Half the length and half the width of the box round its wheels, along
    /// its front and across it, about the middle of its wheels: the room they
    /// take on the road, which they keep to turning where all of it cannot
    /// (`roads::turning`).
    wheels: Vec2,
}

impl Hull {
    /// The room it takes on the road, as its way round is worked out.
    fn footprint(&self) -> roads::Footprint {
        roads::Footprint {
            base: self.base,
            half: self.half,
            ahead: -self.wheels_ahead,
            wheels: self.wheels,
        }
    }
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
    /// The seconds since 1970 it was placed by this frame, by the town's
    /// clock.
    seconds: f64,
    /// How far it has come since 1970, for its wheels.
    distance: f64,
    placed: bool,
}

/// The clock the car goes by: the town's (`clock`), in seconds since 1970, or
/// stopped at one point of its way round, from `RT_CAR_AT`.
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

    /// The seconds the car goes by now, `town` being the town's clock.
    fn now(&self, town: &TownClock) -> f64 {
        self.stopped.unwrap_or_else(|| town.now())
    }

    /// The seconds it went by `ago` seconds before `now`: the same, if it is
    /// stopped.
    fn before(&self, now: f64, ago: f64) -> f64 {
        if self.stopped.is_some() { now } else { now - ago }
    }
}

/// What the town's model has to say about its roads, kept from the moment it
/// spawns until its shape and every car's have been read too, and the cars'
/// way round can be worked out.
#[derive(Resource)]
struct TownRoads {
    road: Vec<[Vec3; 3]>,
    markings: Vec<Vec<[Vec3; 3]>>,
}

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(CarClock::from_env())
        .add_observer(read_roads)
        .add_systems(Startup, spawn_cars)
        .add_systems(PreUpdate, drive)
        .add_systems(Update, (plan_the_way, turn_wheels).chain());
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

/// Works out the cars' way round the town once its roads, its shape and the
/// shape of every car have been read: turning, none of any of them is to go
/// off the road.
fn plan_the_way(
    roads: Option<Res<TownRoads>>,
    islands: Res<Islands>,
    hulls: Query<&Hull, With<Car>>,
    mut commands: Commands,
) {
    let Some(roads) = roads else {
        return;
    };
    let Some(land) = islands.ground(Venue::Town) else {
        return;
    };
    if hulls.iter().count() < CARS.len() {
        return;
    }
    // Worked out for the longest of them, which has the least room to turn.
    let Some(car) = hulls.iter().max_by(|a, b| a.half.x.total_cmp(&b.half.x)).map(Hull::footprint) else {
        return;
    };
    commands.remove_resource::<TownRoads>();
    let road = Island::new(&roads.road);
    let paint = Paint {
        land,
        road: &road,
        markings: &roads.markings,
    };
    let started = std::time::Instant::now();
    match Route::read(&paint, &car, CARS.len()) {
        Some(route) => {
            let behind: Vec<f64> = (0..CARS.len() as u8).map(|number| route.behind(number)).collect();
            info!(
                "the cars' way round the town: {} lanes, {:.0} m, in {} s, each car {:?} s behind the first \
                 (worked out in {} ms for {car:?})",
                route.lanes(),
                route.length(),
                route.lap(),
                behind,
                started.elapsed().as_millis()
            );
            commands.insert_resource(route);
        }
        None => warn!("no way round the town's roads, so no car drives them"),
    }
}

/// Every car, from the start: nowhere, until its shape has been read, the
/// way round worked out and it is put on the road (`drive`), and from then
/// on the town, drawn there and met there.
fn spawn_cars(assets: Res<AssetServer>, mut commands: Commands) {
    for (number, model) in (0..).zip(CARS) {
        commands
            .spawn((
                Name::new("Car"),
                Car { number },
                Driven::default(),
                WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(model))),
                Transform::default(),
                Visibility::Hidden,
            ))
            .observe(read_car);
    }
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
    cars: Query<&Car>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>), Without<SkinnedMesh>>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    placement: TransformHelper,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let car = ready.entity;
    let (Ok(model), Ok(root)) = (cars.get(car).map(Car::model), placement.compute_global_transform(car)) else {
        return;
    };
    let into_car = root.affine().inverse();
    // Each object's triangles, by the object.
    let mut pieces: Vec<(Option<Entity>, Vec<[Vec3; 3]>)> = Vec::new();
    let mut wheels: Vec<(Entity, Handle<Mesh>, Option<Handle<StandardMaterial>>, Affine3A)> = Vec::new();
    // The box round all its wheels.
    let mut wheel_box: Option<(Vec3, Vec3)> = None;
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
            warn!("{model}: could not read one of its meshes, so bodies will pass through it");
            continue;
        }
        // A wheel is a mesh of an object called that in Blender.
        let is_wheel = [Some(part), object].into_iter().flatten().any(|entity| {
            names
                .get(entity)
                .is_ok_and(|name| is_wheel(name.as_str()))
        });
        if is_wheel {
            wheels.push((part, mesh.0.clone(), material.map(|m| m.0.clone()), in_car.affine()));
            wheel_box = corners.iter().flatten().fold(wheel_box, |reach, &corner| {
                Some(reach.map_or((corner, corner), |(low, high)| (low.min(corner), high.max(corner))))
            });
        }
        match pieces.iter_mut().find(|(of, _)| *of == object) {
            Some((_, triangles)) => triangles.extend(corners),
            None => pieces.push((object, corners)),
        }
    }
    let all: Vec<Vec3> = pieces.iter().flat_map(|(_, triangles)| triangles.iter().flatten()).copied().collect();
    if all.is_empty() {
        warn!("{model} has nothing in it to stand on");
        return;
    }
    let (low, high) = all
        .iter()
        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), &corner| {
            (low.min(corner), high.max(corner))
        });
    let parts: Vec<&[[Vec3; 3]]> = pieces.iter().map(|(_, triangles)| triangles.as_slice()).collect();
    let turning = split_wheels(&wheels, &mut meshes);
    let (front, middle) = (front(low, high), ((low + high) * 0.5).xz());
    let (base, wheels_ahead) = axles(turning.iter().map(|wheel| (wheel.middle.xz() - middle).dot(front)));
    let hull = Hull {
        shape: Island::of_parts(&parts),
        front,
        middle,
        lowest: low.y,
        reach: all.iter().map(|corner| corner.xz().length()).fold(0.0, f32::max),
        half: half(low, high),
        base,
        wheels_ahead,
        wheels: wheel_room(low, high, wheel_box),
    };
    let across = hull.front.perp();
    let axle = Vec3::new(across.x, 0.0, across.y);
    for wheel in turning {
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
    commands.entity(car).insert(hull);
}

/// Which way the front of a car whose model fits between `low` and `high`, in
/// its own space, faces across the ground: along its longer side, toward
/// Blender's -X, the hood of Hajun's first car, or -Y, Blender's front, the
/// bus's windscreen, whichever it is longer along. Blender's -X is +x here
/// and its -Y is -z: the glTF loader turns models half round
/// (`GltfConvertCoordinates`).
fn front(low: Vec3, high: Vec3) -> Vec2 {
    let size = high - low;
    if size.x >= size.z { Vec2::X } else { Vec2::NEG_Y }
}

/// Half the length and half the width of a car whose model fits between
/// `low` and `high`, in its own space: along its front ([`front`]) and across.
fn half(low: Vec3, high: Vec3) -> Vec2 {
    let size = (high - low) * 0.5;
    if front(low, high) == Vec2::X { Vec2::new(size.x, size.z) } else { Vec2::new(size.z, size.x) }
}

/// Half the length and half the width of the box round the wheels of a car
/// whose model fits between `low` and `high`, in its own space, and whose
/// wheels fit in `wheels`: along its front, as long as they are from the
/// front of the front ones to the back of the back ones, and across it, as
/// far out from its middle as they go. With no wheels, all of it.
fn wheel_room(low: Vec3, high: Vec3, wheels: Option<(Vec3, Vec3)>) -> Vec2 {
    let Some((wheels_low, wheels_high)) = wheels else {
        return half(low, high);
    };
    let middle = (low + high) * 0.5;
    let long = (wheels_high - wheels_low) * 0.5;
    let out = (wheels_high - middle).abs().max((wheels_low - middle).abs());
    if front(low, high) == Vec2::X { Vec2::new(long.x, out.z) } else { Vec2::new(long.z, out.x) }
}

/// How far apart a car's front and back wheels are, and how far ahead of its
/// footprint's middle the middle of them is, from how far ahead of it each of
/// its wheels' middles is (`ahead`), along its front: the front ones the
/// furthest ahead, the back ones the furthest behind. With no wheels, its
/// wheels are all in its middle.
fn axles(ahead: impl IntoIterator<Item = f32>) -> (f32, f32) {
    let (back, front) = ahead
        .into_iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(back, front), at| (back.min(at), front.max(at)));
    if front < back { (0.0, 0.0) } else { (front - back, (front + back) * 0.5) }
}

/// Where the car numbered `number` is, and which way it is turned, `seconds`
/// since 1970: its front and back wheels on the middle of its lane, its front
/// facing from the one to the other, and its wheels on the road. With how far
/// it has come.
pub(crate) fn place_at(route: &Route, hull: &Hull, number: u8, seconds: f64) -> (Place, f64) {
    let (on_road, way, distance) = route.on_wheels(seconds - route.behind(number), hull.base);
    let turn = Quat::from_rotation_y(heading(way) - heading(hull.front));
    let wheels = hull.middle + hull.front * hull.wheels_ahead;
    let at = on_road - turn * Vec3::new(wheels.x, 0.0, wheels.y) - Vec3::Y * hull.lowest;
    (Place { at, turn }, distance)
}

/// The turn about the upright that has -z face `way`, across the ground.
fn heading(way: Vec2) -> f32 {
    (-way.x).atan2(-way.y)
}

/// Puts each car where the clock says it is, keeping where it was the frame
/// before for whoever is riding it, and on the town, the first time. Only
/// what has changed is written.
fn drive(
    clock: Res<CarClock>,
    town: Res<TownClock>,
    route: Option<Res<Route>>,
    mut cars: Query<(Entity, &Car, &Hull, &mut Transform, &mut Driven, Has<Venue>)>,
    mut commands: Commands,
) {
    let Some(route) = route else {
        return;
    };
    let seconds = clock.now(&town);
    for (entity, car, hull, mut transform, mut driven, on_the_town) in &mut cars {
        let (place, distance) = place_at(&route, hull, car.number, seconds);
        let then = if driven.placed { driven.now } else { place };
        *driven = Driven {
            now: place,
            then,
            seconds,
            distance,
            placed: true,
        };
        transform.set_if_neq(Transform::from_translation(place.at).with_rotation(place.turn));
        if !on_the_town {
            commands.entity(entity).insert(Venue::Town);
        }
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
    /// The car numbered `number`, and where it is now, once it is on the
    /// road.
    pub(crate) fn numbered(&self, number: u8) -> Option<(Entity, Place)> {
        self.cars
            .iter()
            .find(|(_, car, _, driven)| car.number == number && driven.placed)
            .map(|(entity, .., driven)| (entity, driven.now))
    }

    /// The number of the car `entity`, and where it is now, once it is on
    /// the road.
    pub(crate) fn of(&self, entity: Entity) -> Option<(u8, Place)> {
        self.cars
            .get(entity)
            .ok()
            .filter(|(.., driven)| driven.placed)
            .map(|(_, car, _, driven)| (car.number, driven.now))
    }

    /// Where the car numbered `number` was `ago` seconds before now.
    pub(crate) fn before(&self, number: u8, ago: f64) -> Option<Place> {
        let route = self.route.as_ref()?;
        let (_, _, hull, driven) = self
            .cars
            .iter()
            .find(|(_, car, _, driven)| car.number == number && driven.placed)?;
        Some(place_at(route, hull, number, self.clock.before(driven.seconds, ago)).0)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// One of Hajun's cars, as its file `model` has it, in its own space:
    /// each object's triangles, and the box round them all.
    pub(crate) fn hajuns_car(model: &str) -> (Vec<Vec<[Vec3; 3]>>, Vec3, Vec3) {
        let pieces = crate::island::glb::objects(&crate::island::glb::read(model));
        let (low, high) = pieces
            .iter()
            .flatten()
            .flatten()
            .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), &corner| {
                (low.min(corner), high.max(corner))
            });
        (pieces, low, high)
    }

    /// The wheels of the car `model`, in its own space, as the game takes
    /// them apart ([`split_wheels`]): each one's box, the pieces of its wheel
    /// objects round one middle.
    fn wheels_of(model: &str) -> Vec<(Vec3, Vec3)> {
        let mut boxes: Vec<(Vec3, Vec3)> = Vec::new();
        for part in crate::island::glb::read(model).iter().filter(|part| is_wheel(&part.node)) {
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
        gather(&boxes).into_iter().map(|(low, high, _)| (low, high)).collect()
    }

    /// The car `model`'s shape, as the game reads it.
    pub(crate) fn hull_of(model: &str) -> Hull {
        let (pieces, low, high) = hajuns_car(model);
        let parts: Vec<&[[Vec3; 3]]> = pieces.iter().map(Vec::as_slice).collect();
        let (front, middle) = (front(low, high), ((low + high) * 0.5).xz());
        let wheels = wheels_of(model);
        let (base, wheels_ahead) =
            axles(wheels.iter().map(|&(low, high)| (((low + high) * 0.5).xz() - middle).dot(front)));
        let wheel_box = wheels.iter().copied().reduce(|(l, h), (a, b)| (l.min(a), h.max(b)));
        Hull {
            shape: Island::of_parts_for_tests(&parts),
            front,
            middle,
            lowest: low.y,
            reach: pieces.iter().flatten().flatten().map(|corner| corner.xz().length()).fold(0.0, f32::max),
            half: half(low, high),
            base,
            wheels_ahead,
            wheels: wheel_room(low, high, wheel_box),
        }
    }

    /// The room each of the cars takes on the road, in the order of [`CARS`].
    pub(crate) fn footprints() -> Vec<roads::Footprint> {
        CARS.iter().map(|model| hull_of(model).footprint()).collect()
    }

    #[test]
    fn the_bus_has_its_wheels_where_they_are_modelled() {
        // Its front wheels 3.77 m ahead of its middle and its back ones 2.75
        // m behind, as Hajun's export of 2026-10-24 has them, give or take a
        // centimetre for whatever is done to it after.
        let bus = hull_of("mybuilds/bus.glb").footprint();
        assert!((bus.base - 6.52).abs() < 0.3, "{bus:?}");
        assert!((bus.ahead + 0.51).abs() < 0.3, "{bus:?}");
        assert!(bus.wheels.x < bus.half.x && (bus.wheels.y - bus.half.y).abs() < 0.1, "{bus:?}");
        // A car with no wheels turns about its middle.
        assert_eq!(axles([]), (0.0, 0.0));
        assert_eq!(axles([1.5, -1.0, 1.5, -1.0]), (2.5, 0.25));
    }

    #[test]
    fn a_car_faces_along_its_longer_side() {
        // Modelled along Blender's x, as Hajun's first is, its hood at -x: +x
        // here.
        let (low, high) = (Vec3::new(-3.0, -0.3, -1.75), Vec3::new(3.0, 1.0, 1.75));
        assert_eq!(front(low, high), Vec2::X);
        assert_eq!(half(low, high), Vec2::new(3.0, 1.75));
        // Turned to face Blender's -Y, as every other model faces: -z here.
        let (low, high) = (Vec3::new(-1.75, -0.3, -3.0), Vec3::new(1.75, 1.0, 3.0));
        assert_eq!(front(low, high), Vec2::NEG_Y);
        assert_eq!(half(low, high), Vec2::new(3.0, 1.75));
        // And each of Hajun's is one or the other, whatever has been done to
        // it.
        for model in CARS {
            let (_, low, high) = hajuns_car(model);
            let size = high - low;
            let way = front(low, high);
            assert!(if way == Vec2::X { size.x >= size.z } else { size.z > size.x }, "{model}");
        }
    }

    #[test]
    fn the_cars_stand_on_the_road_facing_the_way_they_go() {
        let (land, road, markings) = crate::roads::tests::round_town();
        let hulls: Vec<Hull> = CARS.iter().map(|model| hull_of(model)).collect();
        let car = hulls.iter().max_by(|a, b| a.half.x.total_cmp(&b.half.x)).unwrap().footprint();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        let route = Route::read(&paint, &car, CARS.len()).unwrap();
        // The loop, a twentieth of a second apart, and how far off it a point is.
        let line: Vec<Vec2> = (0..(route.lap() / 0.05) as usize).map(|k| route.at(k as f64 * 0.05).0.xz()).collect();
        let off_the_line = |at: Vec2| {
            (0..line.len())
                .map(|k| {
                    let (a, b) = (line[k], line[(k + 1) % line.len()]);
                    let along = ((at - a).dot(b - a) / (b - a).length_squared().max(1e-9)).clamp(0.0, 1.0);
                    (a + (b - a) * along).distance(at)
                })
                .fold(f32::INFINITY, f32::min)
        };
        for (number, hull) in (0..).zip(&hulls) {
            for step in 0..40 {
                let seconds = 1.76e9 + step as f64 * 4.9;
                let (place, _) = place_at(&route, hull, number, seconds);
                // Its front wheels and its back ones on the loop, as a train's
                // on its rails.
                for wheels in [hull.wheels_ahead + hull.base * 0.5, hull.wheels_ahead - hull.base * 0.5] {
                    let at = hull.middle + hull.front * wheels;
                    let on = place.outward(Vec3::new(at.x, 0.0, at.y)).xz();
                    assert!(off_the_line(on) < 0.05, "car {number}'s wheels {} m off its way", off_the_line(on));
                }
                // Its wheels on the road, -0.2 m.
                assert!((place.outward(Vec3::Y * hull.lowest).y + 0.2).abs() < 0.02, "{place:?}");
            }
            // All the way round, its front the way it goes, near enough: its
            // middle goes a little to the side of the way it faces turning, as
            // a bus's does, its back wheels following its front ones round
            // the bend, but never more than 30° (`roads::TIGHTEST`). And never
            // faster than its top speed.
            let mut seconds = 0.0;
            while seconds < route.lap() {
                let (place, _) = place_at(&route, hull, number, seconds);
                let (soon, _) = place_at(&route, hull, number, seconds + 0.05);
                let ahead = place.turn * Vec3::new(hull.front.x, 0.0, hull.front.y);
                let went = (soon.at - place.at).normalize_or_zero();
                assert!(went == Vec3::ZERO || ahead.dot(went) > 0.866, "car {number} facing {ahead}, going {went}");
                let speed = soon.at.distance(place.at) / 0.05;
                assert!(speed < roundtown_net::cars::TOP_SPEED * 1.05, "{speed} m/s");
                seconds += 0.05;
            }
        }
        // The second car where the first was, as far behind it as it goes.
        let behind = route.behind(1);
        assert!(behind > 0.0 && behind < route.lap());
        let (first, _) = place_at(&route, &hulls[0], 0, 1.76e9);
        let (second, _) = place_at(&route, &hulls[1], 1, 1.76e9 + behind);
        let middle = |place: &Place, hull: &Hull| place.outward(Vec3::new(hull.middle.x, 0.0, hull.middle.y)).xz();
        assert!(middle(&first, &hulls[0]).distance(middle(&second, &hulls[1])) < 1e-3);
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
        for model in CARS {
            let parts = crate::island::glb::read(model);
            let wheel: Vec<&crate::island::glb::Part> = parts.iter().filter(|part| is_wheel(&part.node)).collect();
            assert!(!wheel.is_empty(), "{model}");
            // Each of its meshes broken into its pieces, and the pieces about
            // the same middle taken together: four wheels, at the car's
            // corners.
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
            assert_eq!(wheels.len(), 4, "{model}: {wheels:?}");
            for (low, high, _) in wheels {
                // Round, side on, and out toward a corner of the car.
                let (tall, long) = (high.y - low.y, (high.x - low.x).max(high.z - low.z));
                assert!((tall - long).abs() < tall * 0.2, "{model}: {tall} tall, {long} long");
                let middle = (low + high) * 0.5;
                assert!(middle.x.abs() > 0.5 && middle.z.abs() > 0.5, "{model}: {middle}");
            }
        }
        // Called a wheel, however else it is called, but a steering wheel.
        assert!(is_wheel("wheels") && is_wheel("Roadster wheel front left") && is_wheel("Wheel.003"));
        assert!(!is_wheel("Roadster") && !is_wheel("steering wheel") && !is_wheel("Steering_Wheel"));
    }
}
