//! The car you drive yourself (Hajun, 2026-11-05: "add this car1.glb model
//! in the asset folder into the game as a user spawnable ... make the emote
//! button to be a toggle on/off switch for car. when on, the player model
//! disappears and u are suddenly now a 'car'"). Press the face among the
//! buttons round the jump button, or C on desktop, and your body is gone
//! and you are the car, Hajun's `car1.glb` ([`MODEL`]), where you stood,
//! facing the way you faced; press it again and you are back on your feet
//! where the car was ([`toggle`]).
//!
//! The camera is fixed straight down over the car, `overhead` up: it zooms,
//! between `overhead_nearest` and `overhead_furthest`, and nothing turns it,
//! neither your fingers
//! nor the stick's coming round nor a double tap (Hajun: "the camera gets
//! fixed as top down (can still zoom in/out, but no auto follow
//! rotate/direction/ no panning. its fixed just follow the car's plane
//! position no rotation.)"; `OrbitCamera::overhead`). The top of the screen
//! stays the way the camera was looking along the ground as you got in, and
//! the stick's up is that way.
//!
//! The car drives as a car does ([`Drive::go`]): its front wheels steer and
//! its back wheels follow them, so it goes round at the radius its steering
//! gives, `tightest` at a crawl and wider the faster it goes, never on the
//! spot. The stick says which way to go, across the screen, and how hard:
//! the car steers toward it the shorter way round, so that pushed behind the
//! car it turns round in a circle (Hajun: "there is a pivot so when ur going
//! forward and then u turn the joystick the other way, you will 'turn' like
//! a real car in the closer direction so that it goes towards that
//! direction"). The fist held, or F, is the reverse: the car slows, and
//! backs away as fast as it boosts, straight back with the stick let go or
//! held where the nose points, and with the stick to one side its nose
//! comes round toward the stick as it backs, as a real car's does with
//! the wheel turned the other way (Hajun, that evening: "when u press the
//! reverse button without touching joystick, it should do a straight
//! reverse backward ... when reversing, the direction of the joystick
//! should be opposite ... it should just deaccellerate and then wheels
//! start towards rotating the other way. so straight back. so its like
//! real reverse"; for an hour it backed toward where the stick pointed,
//! so that held where you were driving it swung round). The jump button
//! held, or Space, is the boost: the car goes `boost` rather than
//! `cruise`, and goes without the stick, straight on (Hajun: "similar
//! to rocket league's car boosting - it just makes ur car faster, but u
//! dont need to press this to move it still goes when u just do the
//! joystick"). The stick let go, it brakes to a stop.
//!
//! The car is your own body, moved by the car ([`crate::Intent::pace`]): it
//! bumps into the island, the buses and everyone as you do, steps up what
//! you step up, and never goes down anything it could not drive back up,
//! into the pond or off the edge of the town into the sea
//! (`crate::strands`); and its nose and tail, which reach far past your
//! body, stop at anything taller than a kerb ([`settle`]), so that it does
//! not drive its front into a house before its middle meets the wall.
//! Nothing of it goes over the network: online, everyone else sees your
//! body go where the car goes, and the server lets a body go as fast as the
//! car boosts (`roundtown_net::cars::DRIVEN_TOP_SPEED`). The car's wheels
//! turn as it rolls and its front ones as it steers (`car::hang_wheels`).
//!
//! **Every number of how it goes is in `assets/ui.ron`, under `car`**
//! ([`Car`], since 2026-11-06; Hajun: "put all the constants related to
//! the car in the ron file. like related to movement. so the turning, the
//! boost/reverse, acc/speed, multipliers, etc. i want to modify the
//! numbers by myself while playing on pc"; until then constants here, my
//! numbers, which the file starts from): the speeds, how fast it gains and
//! loses them, how tightly it turns and how its steering answers the
//! stick, the camera's height over it, and how its nose and tail feel for
//! walls. [`steer`] and [`settle`] read the file as it stands every frame
//! (`ui()`, a handle), so a save on desktop changes the car at once, and
//! the camera's height over it goes to the new number while you drive
//! ([`retune`]). A phone drives by what the file said when its app was
//! built.
//!
//! **The gift button held, or G, is the gun** ([`shoot`], [`fly`], since
//! the same day; Hajun: "make the gift icon, while riding a car, so that it
//! shoots small fast spherical bullets out of the front of the car while
//! pressed at semifast intervals that disappear after a certain defined
//! distance range from initial starting point"): in the car, a bullet out
//! of its nose as the button goes down and then one every `bullet_every`
//! seconds, each a lit yellow ball of `bullet_radius`, going
//! `bullet_speed` level the way the car faces, and gone once
//! `bullet_range` from where it was fired; the four in the file with the
//! rest. A bullet hits nothing, and nothing of it goes over the network.

use std::f32::consts::TAU;

use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use crate::car::{self, Turns, WheelPart};
use crate::island::{self, Islands, Venue};
use crate::ui::{Car, UiChanged, ui};
use crate::{
    CloseUp, Decks, Intent, OrbitCamera, Player, STICK_DEADZONE, ThirdPersonCamera, TouchControls, View,
    camera_ahead, decks_on, hud,
};

/// Hajun's car, under `assets/`: a red sports car 6.5 m long and 3.1 m wide
/// on four wheels, facing Blender's front, with 3.84 m between its axles.
pub(crate) const MODEL: &str = "car1.glb";

/// The car's numbers as the file has them now (`assets/ui.ron`, `car`).
fn tuning() -> Car {
    ui().car
}

/// The car's shape, read off its model once it has spawned ([`read_my_car`])
/// and, until then, the model's own numbers.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct Shape {
    /// Which way its front faces, in its own space, across the ground.
    front: Vec2,
    /// How high the lowest of it is, in its own space: the bottoms of its
    /// wheels, which stand on the ground.
    lowest: f32,
    /// How far its nose reaches ahead of its origin, and its tail behind,
    /// along its front.
    ahead: f32,
    behind: f32,
    half_width: f32,
    /// How far apart its front and back wheels are.
    base: f32,
    /// How far behind its origin its back wheels are: it turns about them.
    rear: f32,
}

impl Default for Shape {
    /// `car1.glb` as Hajun exported it on 2026-11-05, to the centimetre (a
    /// test keeps it so).
    fn default() -> Self {
        Self {
            front: Vec2::NEG_Y,
            lowest: 0.0,
            ahead: 3.71,
            behind: 2.84,
            half_width: 1.56,
            base: 3.84,
            rear: 1.75,
        }
    }
}

/// The car you drive: what its model hangs from, hidden until you get in.
#[derive(Component)]
pub(crate) struct MyCar;

/// The face button pressed: into the car, or out of it.
#[derive(Message, Default)]
pub(crate) struct CarPressed;

/// Whether you are in the car, and, in it, how it goes.
#[derive(Resource, Default)]
pub(crate) struct Driving(Option<Drive>);

impl Driving {
    pub(crate) fn on(&self) -> bool {
        self.0.is_some()
    }
}

/// The car as it goes.
#[derive(Debug)]
struct Drive {
    /// Which way it faces, across the ground.
    way: Vec2,
    /// How fast it goes that way, in metres a second: less than nothing,
    /// reversing.
    speed: f32,
    /// How far its front wheels are turned from the way it faces, in
    /// radians, to the left.
    steer: f32,
    /// How far it has rolled, in metres, for its wheels: back, reversing.
    rolled: f32,
    /// The camera as you had turned and zoomed it, to go back to.
    view: Option<View>,
    /// The island you got in on: sent anywhere else, you are out of it.
    venue: Option<Venue>,
    /// Where your body stood before this frame's move, and the move asked
    /// for, across the ground, to tell whether it was stopped short.
    from: Vec3,
    asked: Vec2,
}

impl Drive {
    /// Standing still, facing `way`.
    fn facing(way: Vec2) -> Self {
        Self {
            way,
            speed: 0.0,
            steer: 0.0,
            rolled: 0.0,
            view: None,
            venue: None,
            from: Vec3::ZERO,
            asked: Vec2::ZERO,
        }
    }

    /// Drives for `dt` seconds, by the file's numbers, `car`: toward
    /// `wanted`, if the stick is pushed, as hard as it is pushed,
    /// `strength`, from 0 to 1; boosted, or reversing. How far the car's
    /// origin goes across the ground.
    ///
    /// The speed goes toward what is asked: reversing, `reverse` back,
    /// whatever the stick says, braking first if it was going forward;
    /// boosted, `boost` forward whatever the stick says; otherwise `cruise`
    /// forward as hard as the stick is pushed, and with the stick let go, a
    /// stop. The stick says where the nose is to point, forward or back:
    /// the front wheels turn to bring it round toward the stick the shorter
    /// way, all the way at `full_lock_at` off it, and straight with the
    /// stick let go; no further than the speed allows ([`radius_at`]).
    /// Backing, that is the wheels turned the other way for the same stick,
    /// as a car reversing turns them: held where the nose points, the car
    /// backs straight. Then the car goes round as a car does: its back
    /// wheels roll the way it faces, and its origin, `shape.rear` ahead of
    /// them, swings with the turn; with the stick pushed, at the least as if
    /// creeping (`creep`), so that stopped against something it still comes
    /// round.
    #[allow(clippy::too_many_arguments)]
    fn go(
        &mut self,
        car: &Car,
        shape: &Shape,
        wanted: Option<Vec2>,
        strength: f32,
        boosting: bool,
        reversing: bool,
        dt: f32,
    ) -> Vec2 {
        let target = if reversing {
            -car.reverse
        } else if boosting {
            car.boost
        } else {
            car.cruise * strength
        };
        let faster = target.abs() > self.speed.abs() && target * self.speed >= 0.0;
        let rate = if !faster {
            car.brake
        } else if boosting || reversing {
            car.boost_accel
        } else {
            car.accel
        };
        self.speed += (target - self.speed).clamp(-rate * dt, rate * dt);

        let going = if self.speed < 0.0 || (self.speed == 0.0 && target < 0.0) { -1.0 } else { 1.0 };
        let lock = (shape.base / radius_at(car, self.speed)).atan();
        let full_lock_at = car.full_lock_at.to_radians();
        let steer_to = wanted.map_or(0.0, |way| {
            let off = left_of(self.way, way);
            going * (off / full_lock_at).clamp(-1.0, 1.0) * lock
        });
        let steer_rate = car.steer_rate.to_radians();
        self.steer += (steer_to - self.steer).clamp(-steer_rate * dt, steer_rate * dt);
        self.steer = self.steer.clamp(-lock, lock);

        let rolling = if wanted.is_some() { self.speed.abs().max(car.creep) * going } else { self.speed };
        let turning = rolling / shape.base * self.steer.tan();
        let step = self.speed * dt;
        self.rolled += step;
        let moved = self.way * step + left_of_way(self.way) * (turning * dt * shape.rear);
        self.way = turn_way(self.way, turning * dt);
        moved
    }
}

/// The tightest the car turns at `speed`, as the radius its back wheels go
/// round: `tightest` up to `easy_speed`, and wider in proportion from
/// there.
fn radius_at(car: &Car, speed: f32) -> f32 {
    car.tightest * (speed.abs() / car.easy_speed).max(1.0)
}

/// `way` turned `angle` radians to the left about the upright, as
/// `Quat::from_rotation_y` turns it.
fn turn_way(way: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(way.x * cos + way.y * sin, way.y * cos - way.x * sin)
}

/// The way to the left of `way`, across the ground.
fn left_of_way(way: Vec2) -> Vec2 {
    Vec2::new(way.y, -way.x)
}

/// How far round to the left of `from` the way `to` is, in radians, from
/// half a turn to the right to half a turn to the left.
fn left_of(from: Vec2, to: Vec2) -> f32 {
    to.dot(left_of_way(from)).atan2(to.dot(from))
}

/// A bullet from the car's gun ([`shoot`]): where it was fired from, which
/// way it goes and how fast, and how far it goes before it is gone
/// ([`fly`]).
#[derive(Component)]
struct Bullet {
    from: Vec3,
    velocity: Vec3,
    range: f32,
}

impl Bullet {
    /// Where it is `dt` seconds on from `at`, or nothing once that is its
    /// range from where it was fired: level and straight, nothing slows it
    /// or pulls it down.
    fn flown(&self, at: Vec3, dt: f32) -> Option<Vec3> {
        let next = at + self.velocity * dt;
        (next.distance(self.from) < self.range).then_some(next)
    }
}

/// The gun: its beat, and what a bullet is made of, made once.
#[derive(Resource, Default)]
struct Gun {
    /// Seconds since it last fired, while the button has been held; nothing
    /// with the button up, so that the next press fires at once.
    since: Option<f32>,
    look: Option<(Handle<Mesh>, Handle<StandardMaterial>)>,
}

impl Gun {
    /// Whether to fire now, `dt` seconds on, with the button held and a
    /// bullet due every `every` seconds: at once as the button goes down,
    /// and then on the beat.
    fn fires(&mut self, dt: f32, every: f32) -> bool {
        let since = self.since.map(|since| since + dt);
        if since.is_some_and(|since| since < every) {
            self.since = since;
            return false;
        }
        self.since = Some(0.0);
        true
    }

    /// The button is up, or you are out of the car.
    fn let_go(&mut self) {
        self.since = None;
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Driving>()
        .init_resource::<Gun>()
        .add_message::<CarPressed>()
        .add_systems(Startup, spawn_my_car)
        .add_systems(Update, (toggle.before(crate::read_player_input), retune))
        // The gun fires from where the car has been put this frame, and
        // its bullets fly on from there.
        .add_systems(Update, (shoot.after(place), fly.after(shoot)));
}

/// Fires the gun while the gift button, or G, is held in the car
/// (`hud::Held::gun`; not under a dialog): a bullet out of the car's nose,
/// from the height the nose is felt at, going `bullet_speed` level the way
/// the car faces, as the button goes down and then every `bullet_every`
/// seconds ([`Gun::fires`]). A bullet is a ball of `bullet_radius`, a lit
/// yellow, with no `Venue`: drawn wherever you are, and in nothing's way.
#[allow(clippy::too_many_arguments)]
fn shoot(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    held: Res<hud::Held>,
    dialog: Res<hud::DialogUp>,
    driving: Res<Driving>,
    cars: Query<&Shape, With<MyCar>>,
    me: Query<&Transform, With<Player>>,
    mut gun: ResMut<Gun>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let firing = !dialog.0 && (held.gun() || keyboard.pressed(KeyCode::KeyG));
    let (Some(drive), true) = (driving.0.as_ref(), firing) else {
        gun.let_go();
        return;
    };
    let car = tuning();
    if !gun.fires(time.delta_secs(), car.bullet_every) {
        return;
    }
    let Ok(body) = me.single() else {
        return;
    };
    let shape = cars.single().copied().unwrap_or_default();
    let way = Vec3::new(drive.way.x, 0.0, drive.way.y);
    let nose = body.translation + way * shape.ahead + Vec3::Y * ((car.felt_low + car.felt_high) * 0.5);
    let (mesh, material) = gun
        .look
        .get_or_insert_with(|| {
            (
                meshes.add(Sphere::new(1.0).mesh().uv(16, 8)),
                materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.85, 0.3),
                    emissive: LinearRgba::new(6.0, 4.0, 0.6, 1.0),
                    ..default()
                }),
            )
        })
        .clone();
    commands.spawn((
        Name::new("Bullet"),
        Bullet {
            from: nose,
            velocity: way * car.bullet_speed,
            range: car.bullet_range,
        },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(nose).with_scale(Vec3::splat(car.bullet_radius)),
    ));
}

/// Flies every bullet on, and takes it away once it has gone its range.
fn fly(time: Res<Time>, mut bullets: Query<(Entity, &Bullet, &mut Transform)>, mut commands: Commands) {
    let dt = time.delta_secs();
    for (entity, bullet, mut transform) in &mut bullets {
        match bullet.flown(transform.translation, dt) {
            Some(at) => transform.translation = at,
            None => commands.entity(entity).despawn(),
        }
    }
}

/// The file read again while you drive (`ui::reread`): the camera goes to
/// the new height over you, if that is what changed, and stays between the
/// new nearest and furthest; the rest the car takes as it goes.
fn retune(changed: Res<UiChanged>, driving: Res<Driving>, mut orbit: ResMut<OrbitCamera>, mut last: Local<Option<f32>>) {
    if !changed.is_changed() {
        return;
    }
    let car = tuning();
    let was = last.replace(car.overhead);
    // Over you in and out of the car since 2026-11-07, so always.
    if !(orbit.overhead || driving.on()) || changed.is_added() {
        return;
    }
    if was.is_some_and(|was| was != car.overhead) {
        orbit.distance = car.overhead;
    }
    orbit.distance = orbit.distance.clamp(car.overhead_nearest, car.overhead_furthest);
}

/// The car, from the start: nowhere and hidden, until you get in.
fn spawn_my_car(assets: Res<AssetServer>, mut commands: Commands) {
    commands
        .spawn((
            Name::new("Your car"),
            MyCar,
            Shape::default(),
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(MODEL))),
            Transform::default(),
            Visibility::Hidden,
        ))
        .observe(read_my_car);
}

/// Reads the car's shape off its model once it has spawned, in the car's own
/// space, and hangs its wheels to turn, the front ones steering
/// (`car::hang_wheels`).
#[allow(clippy::too_many_arguments)]
fn read_my_car(
    ready: On<WorldInstanceReady>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>), Without<SkinnedMesh>>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    placement: TransformHelper,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cars: Query<&mut Shape, With<MyCar>>,
    mut commands: Commands,
) {
    let car = ready.entity;
    let (Ok(mut shape), Ok(root)) = (cars.get_mut(car), placement.compute_global_transform(car)) else {
        return;
    };
    let into_car = root.affine().inverse();
    let mut corners: Vec<[Vec3; 3]> = Vec::new();
    let mut wheels: Vec<WheelPart> = Vec::new();
    for part in children.iter_descendants(car) {
        let Ok((mesh, material)) = parts.get(part) else {
            continue;
        };
        let Ok(place) = placement.compute_global_transform(part) else {
            continue;
        };
        let in_car = GlobalTransform::from(into_car * place.affine());
        let read = meshes.get(&mesh.0).is_some_and(|asset| island::add_mesh(&mut corners, asset, &in_car));
        if !read {
            warn!("{MODEL}: could not read one of its meshes");
            continue;
        }
        let object = parents.get(part).ok().map(ChildOf::parent);
        let is_wheel = [Some(part), object]
            .into_iter()
            .flatten()
            .any(|entity| names.get(entity).is_ok_and(|name| car::is_wheel(name.as_str())));
        if is_wheel {
            wheels.push((part, mesh.0.clone(), material.map(|m| m.0.clone()), in_car.affine()));
        }
    }
    if corners.is_empty() {
        warn!("{MODEL}: nothing in it to drive");
        return;
    }
    let (low, high) = corners
        .iter()
        .flatten()
        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), &corner| (low.min(corner), high.max(corner)));
    let front = car::front(low, high);
    let turning = car::split_wheels(&wheels, &mut meshes);
    let (base, mid) = car::axles(turning.iter().map(|wheel| wheel.middle.xz().dot(front)));
    let along = |corner: &Vec3| corner.xz().dot(front);
    *shape = Shape {
        front,
        lowest: low.y,
        ahead: corners.iter().flatten().map(along).fold(f32::NEG_INFINITY, f32::max),
        behind: -corners.iter().flatten().map(along).fold(f32::INFINITY, f32::min),
        half_width: car::half(low, high).y,
        base,
        rear: base * 0.5 - mid,
    };
    info!(
        "{MODEL}: {} wheels, {:.2} m between the axles, {:.2} m long, {:.2} m wide",
        turning.len(),
        base,
        shape.ahead + shape.behind,
        shape.half_width * 2.0
    );
    let across = front.perp();
    let axle = Vec3::new(across.x, 0.0, across.y);
    car::hang_wheels(&mut commands, car, turning, &wheels, axle, |wheel| wheel.middle.xz().dot(front) > mid);
}

/// Into the car, at the press of the face button or C, where you stand and
/// facing the way you face, with the camera fixed over you; or out of it,
/// standing where the car is, with the camera back as you had it. Not
/// while a dialog is up or the camera is on someone you talk to; and out
/// of it, whatever is pressed, once you are sent to another island.
#[allow(clippy::too_many_arguments)]
fn toggle(
    mut pressed: MessageReader<CarPressed>,
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Res<hud::DialogUp>,
    close_up: Res<CloseUp>,
    me: Query<(&Transform, &Venue), With<Player>>,
    mut cars: Query<&mut Visibility, With<MyCar>>,
    mut driving: ResMut<Driving>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let asked = pressed.read().count() > 0 || keyboard.just_pressed(KeyCode::KeyC);
    let Ok((body, &venue)) = me.single() else {
        return;
    };
    let sent = driving.0.as_ref().is_some_and(|drive| drive.venue != Some(venue));
    if !sent && !(asked && !dialog.0 && !close_up.busy()) {
        return;
    }
    let shown = match driving.0.take() {
        Some(drive) => {
            if let Some(view) = drive.view {
                orbit.back_to(view);
            }
            false
        }
        None => {
            let way = body.forward().xz().normalize_or(Vec2::NEG_Y);
            // Fixed over you already, out of the car too (since 2026-11-07,
            // Hajun: "make the camera top down for even the player mode"),
            // there is nothing to go back to.
            let view = if orbit.overhead { None } else { Some(orbit.overhead()) };
            driving.0 = Some(Drive {
                view,
                venue: Some(venue),
                from: body.translation,
                ..Drive::facing(way)
            });
            true
        }
    };
    for mut visibility in &mut cars {
        visibility.set_if_neq(if shown { Visibility::Inherited } else { Visibility::Hidden });
    }
}

/// Drives the car by the stick, or the keys, the fist and the jump button,
/// and asks your body to go where it goes ([`Intent::pace`]): before
/// `move_bodies`, in place of the walk the stick would ask for. With a
/// dialog up, the car brakes to a stop.
#[allow(clippy::too_many_arguments)]
pub(crate) fn steer(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<TouchControls>,
    held: Res<hud::Held>,
    dialog: Res<hud::DialogUp>,
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    cars: Query<&Shape, With<MyCar>>,
    mut me: Query<(&Transform, &mut Intent), With<Player>>,
    mut driving: ResMut<Driving>,
) {
    let Some(drive) = driving.0.as_mut() else {
        return;
    };
    let Ok((body, mut intent)) = me.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let shape = cars.single().copied().unwrap_or_default();
    let car = tuning();
    let (wanted, strength, boosting, reversing) = if dialog.0 {
        (None, 0.0, false, false)
    } else {
        // The stick, or the keys, across the screen: its up is the way the
        // top of the screen is along the ground.
        let ahead = cameras.single().map_or(Vec3::NEG_Z, camera_ahead);
        let right = ahead.cross(Vec3::Y).xz();
        let stick = controls.stick_value;
        let push = if stick.length() >= STICK_DEADZONE {
            stick
        } else {
            let key = |code: KeyCode| if keyboard.pressed(code) { 1.0 } else { 0.0 };
            Vec2::new(key(KeyCode::KeyD) - key(KeyCode::KeyA), key(KeyCode::KeyW) - key(KeyCode::KeyS))
        };
        let way = right * push.x + ahead.xz() * push.y;
        let boosting = controls.jump_id.is_some() || keyboard.pressed(KeyCode::Space);
        let reversing = held.fist() || keyboard.pressed(KeyCode::KeyF);
        (way.try_normalize(), push.length().min(1.0), boosting, reversing)
    };
    drive.from = body.translation;
    let moved = drive.go(&car, &shape, wanted, strength, boosting, reversing, dt);
    drive.asked = moved;
    let face = Some(Vec3::new(drive.way.x, 0.0, drive.way.y));
    *intent = match moved.try_normalize() {
        Some(dir) => Intent {
            walk: Vec3::new(dir.x, 0.0, dir.y),
            jump: false,
            pace: Some(moved.length() / dt),
            face,
        },
        None => Intent {
            walk: Vec3::ZERO,
            jump: false,
            pace: Some(0.0),
            face,
        },
    };
}

/// Stops the car at what it met, after `move_bodies`: stopped short by what
/// your body bumped into, it loses its speed; and its nose, going forward,
/// or its tail, reversing, is kept out of anything taller than a kerb, of
/// the island or a bus on it, as your body is kept out of walls
/// (`resolve_walls`), the whole car moving back with it, and the car loses
/// its speed by how squarely it went in: all of it head on, none grazing
/// along a wall.
pub(crate) fn settle(
    islands: Res<Islands>,
    cars: Decks,
    shapes: Query<&Shape, With<MyCar>>,
    mut me: Query<(&mut Transform, &Venue), With<Player>>,
    mut driving: ResMut<Driving>,
) {
    let Some(drive) = driving.0.as_mut() else {
        return;
    };
    let Ok((mut body, &venue)) = me.single_mut() else {
        return;
    };
    let asked = drive.asked.length();
    if asked < 1e-5 {
        return;
    }
    let car = tuning();
    let went = (body.translation - drive.from).xz().dot(drive.asked / asked);
    if went < asked * car.stopped_short {
        drive.speed = 0.0;
    }
    let Some(island) = islands.get(venue) else {
        return;
    };
    let decks = decks_on(&cars, venue);
    let shape = shapes.single().copied().unwrap_or_default();
    let forward = drive.asked.dot(drive.way) >= 0.0;
    let (reach, way) = if forward { (shape.ahead, drive.way) } else { (shape.behind, -drive.way) };
    let end = body.translation.xz() + way * (reach - car.end_back);
    let feet = body.translation.y;
    let mut point = Vec3::new(end.x, feet, end.y);
    let felt = |feet: f32| [(car.end_radius, feet + car.felt_low, feet + car.felt_high)];
    crate::resolve_walls(&mut point, island, &felt(feet));
    for deck in &decks {
        if !deck.near(point.xz(), car.end_radius + island::LATTICE) {
            continue;
        }
        let mut local = deck.now.inward(point);
        let was = local;
        let band = felt(local.y);
        crate::resolve_walls(&mut local, deck.shape, &band);
        if local != was {
            point = deck.now.outward(local);
        }
    }
    let push = point.xz() - end;
    if push.length_squared() > 1e-10 {
        body.translation.x += push.x;
        body.translation.z += push.y;
        let squarely = -push.normalize().dot(way);
        drive.speed *= (1.0 - squarely).clamp(0.0, 1.0);
    }
}

/// The car, to place it.
type MyCars<'w, 's> = Query<'w, 's, (&'static Shape, &'static mut Transform, &'static Children), (With<MyCar>, Without<Player>)>;
/// Its wheels, to turn them.
type MyWheels<'w, 's> = Query<'w, 's, (&'static Turns, &'static mut Transform), (Without<MyCar>, Without<Player>)>;

/// Puts the car where your body is, facing the way it faces, its wheels on
/// the ground, rolled as far as it has gone and the front ones turned as it
/// steers. Only what has changed is written.
pub(crate) fn place(
    driving: Res<Driving>,
    me: Query<&Transform, With<Player>>,
    mut cars: MyCars,
    mut wheels: MyWheels,
) {
    let Some(drive) = driving.0.as_ref() else {
        return;
    };
    let Ok(body) = me.single() else {
        return;
    };
    let Ok((shape, mut transform, children)) = cars.single_mut() else {
        return;
    };
    let turn = Quat::from_rotation_y(car::heading(drive.way) - car::heading(shape.front));
    transform.set_if_neq(Transform::from_translation(body.translation - Vec3::Y * shape.lowest).with_rotation(turn));
    for child in children.iter() {
        let Ok((turns, mut wheel)) = wheels.get_mut(child) else {
            continue;
        };
        let round = TAU * turns.radius;
        let angle = drive.rolled.rem_euclid(round) / turns.radius;
        // Rolling forward, the top of the wheel goes the way the car does.
        let roll = Quat::from_axis_angle(turns.axle, -angle);
        let rotation = if turns.front { Quat::from_rotation_y(drive.steer) * roll } else { roll };
        if wheel.rotation != rotation {
            wheel.rotation = rotation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 120.0;

    /// Drives `drive` for `secs` with the stick `wanted`, pushed all the
    /// way, by the file's numbers as built in, and says how far it went
    /// across the ground.
    fn driven(drive: &mut Drive, wanted: Option<Vec2>, boosting: bool, reversing: bool, secs: f32) -> Vec2 {
        let car = tuning();
        let mut at = Vec2::ZERO;
        let mut left = secs;
        while left > 0.0 {
            let strength = if wanted.is_some() { 1.0 } else { 0.0 };
            at += drive.go(&car, &Shape::default(), wanted, strength, boosting, reversing, DT);
            left -= DT;
        }
        at
    }

    #[test]
    fn the_car_goes_no_faster_than_the_server_lets_a_body_go() {
        // Nothing tells the server a body is in the car: it lets every body
        // go as fast as the car boosts and no faster, with some slack
        // (`MOST_SPEED` in server/src/players.rs). Faster in the file, and
        // online the server would put the car back every second or so: raise
        // `DRIVEN_TOP_SPEED` in shared/src/lib.rs with it, and deploy the
        // server.
        let car = tuning();
        let most = roundtown_net::cars::DRIVEN_TOP_SPEED;
        for (name, speed) in [("cruise", car.cruise), ("boost", car.boost), ("reverse", car.reverse)] {
            assert!(
                speed <= most,
                "assets/ui.ron: car.{name} is {speed} m/s, faster than the server lets a body go ({most}): \
                 raise roundtown_net::cars::DRIVEN_TOP_SPEED with it and deploy the server"
            );
        }
    }

    #[test]
    fn turning_left_is_the_engine_s_turn_about_the_upright() {
        for (way, angle) in [(Vec2::NEG_Y, 0.4), (Vec2::X, -1.2), (Vec2::new(0.6, 0.8), 2.9)] {
            let turned = turn_way(way, angle);
            let engine = (Quat::from_rotation_y(angle) * Vec3::new(way.x, 0.0, way.y)).xz();
            assert!(turned.distance(engine) < 1e-5, "{way} by {angle}: {turned} against {engine}");
            // The left of the way is a quarter turn that way, and the angle
            // round to the left comes back out.
            assert!(left_of_way(way).distance(turn_way(way, std::f32::consts::FRAC_PI_2)) < 1e-5);
            assert!((left_of(way, turned) - angle).abs() < 1e-4, "{way} by {angle}");
        }
    }

    #[test]
    fn it_steers_toward_the_stick_the_shorter_way_round() {
        // Facing -z, the stick to the left (-x): it comes round to the left.
        let mut drive = Drive::facing(Vec2::NEG_Y);
        let went = driven(&mut drive, Some(Vec2::NEG_X), false, false, 3.0);
        assert!(drive.way.distance(Vec2::NEG_X) < 0.05, "facing {}", drive.way);
        assert!(went.x < -5.0 && went.y < 0.0, "went {went}");
        // The stick straight behind, a hair to the right: round to the right,
        // never turning back on itself in a frame (a hair of wobble as the
        // steering settles on the stick, a hundredth of a radian, is not
        // that: with a tight `tightest` in the file it overshoots by a few
        // thousandths; the long way round would be tens of thousandths).
        let mut drive = Drive::facing(Vec2::NEG_Y);
        let mut most_turn: f32 = 0.0;
        let car = tuning();
        for _ in 0..(4.0 / DT) as usize {
            let before = drive.way;
            drive.go(&car, &Shape::default(), Some(Vec2::new(0.05, 1.0).normalize()), 1.0, false, false, DT);
            let turn = left_of(before, drive.way);
            assert!(turn <= 1e-2, "turned left {turn}");
            most_turn = most_turn.max(turn.abs());
        }
        assert!(drive.way.distance(Vec2::Y) < 0.05, "facing {}", drive.way);
        assert!(most_turn < 0.1);
    }

    #[test]
    fn it_turns_no_tighter_than_its_back_wheels_can_at_a_crawl_and_wider_boosting() {
        // Round and round to the left with the stick behind it: a circle,
        // whose radius is read off how far round the back wheels go in a
        // lap, which is `tightest` once the speed is off the stick's cruise.
        let shape = Shape::default();
        let car = tuning();
        let lap = |boosting: bool| -> (f32, f32) {
            let mut drive = Drive::facing(Vec2::NEG_Y);
            // Up to speed and round to full lock first: as long as the
            // file's boost takes to reach, and 3 s over.
            let warm_up = car.boost / car.boost_accel.max(car.accel) + 3.0;
            for _ in 0..(warm_up / DT) as usize {
                drive.go(&car, &shape, Some(left_of_way(drive.way)), 1.0, boosting, false, DT);
            }
            let mut turned = 0.0;
            let mut rolled = 0.0;
            while turned < TAU {
                let before = drive.way;
                let was = drive.rolled;
                drive.go(&car, &shape, Some(left_of_way(drive.way)), 1.0, boosting, false, DT);
                turned += left_of(before, drive.way);
                rolled += drive.rolled - was;
            }
            (rolled / TAU, drive.speed)
        };
        let (radius, speed) = lap(false);
        assert!((speed - car.cruise).abs() < 0.01, "{speed} m/s");
        assert!((radius - radius_at(&car, car.cruise)).abs() < 0.1, "{radius} m round at {speed} m/s");
        let (wide, fast) = lap(true);
        assert!((fast - car.boost).abs() < 0.01, "{fast} m/s");
        assert!(wide > radius * 1.3, "{wide} m round boosting against {radius} cruising");
        assert_eq!(radius_at(&car, 3.0), car.tightest);
        // Tuned in the file, the car goes by the new numbers: twice as
        // tight a circle with `tightest` halved.
        let tighter = Car { tightest: car.tightest * 0.5, ..car };
        assert_eq!(radius_at(&tighter, 3.0), car.tightest * 0.5);
        assert_eq!(radius_at(&tighter, car.cruise), radius_at(&car, car.cruise) * 0.5);
    }

    #[test]
    fn reversing_backs_straight_and_brings_the_nose_round_to_the_stick() {
        // Driving forward along -z with the stick that way, the reverse
        // pressed and the stick held: it slows, and backs straight along +z,
        // as fast as it boosts, still facing -z.
        let car = tuning();
        let mut drive = Drive::facing(Vec2::NEG_Y);
        driven(&mut drive, Some(Vec2::NEG_Y), false, false, 3.0);
        assert!((drive.speed - car.cruise).abs() < 0.01);
        let went = driven(&mut drive, Some(Vec2::NEG_Y), false, true, 4.0);
        assert!((drive.speed + car.reverse).abs() < 0.01, "{drive:?}");
        assert!(went.y > 20.0 && went.x.abs() < 1e-3, "went {went}");
        assert!(drive.way.distance(Vec2::NEG_Y) < 1e-4 && drive.steer == 0.0, "{drive:?}");
        // With no stick at all, straight back from standing.
        let mut drive = Drive::facing(Vec2::X);
        let went = driven(&mut drive, None, false, true, 3.0);
        assert!(went.x < -20.0 && went.y.abs() < 1e-3 && drive.speed < 0.0, "{drive:?} went {went}");
        // Facing -z, reversing with the stick to the left of the nose (-x):
        // the nose comes round to the left as it backs, the way a car's
        // does reversing with its wheels turned right, until it faces the
        // stick, and from then it backs straight, away from it.
        let mut drive = Drive::facing(Vec2::NEG_Y);
        let went = driven(&mut drive, Some(Vec2::NEG_X), false, true, 5.0);
        assert!(drive.way.distance(Vec2::NEG_X) < 0.05, "facing {}", drive.way);
        assert!(went.x > 10.0, "went {went}");
        // The stick let go: it comes to a stop, in as long as the file's
        // brake takes from its reverse speed, and a second over.
        driven(&mut drive, None, false, false, car.reverse / car.brake + 1.0);
        assert_eq!(drive.speed, 0.0);
        assert_eq!(drive.steer, 0.0);
        // Boosting with no stick drives it straight on, faster than cruising.
        let mut drive = Drive::facing(Vec2::X);
        let went = driven(&mut drive, None, true, false, 4.0);
        assert!(drive.speed > car.cruise && went.y.abs() < 1e-3 && went.x > 40.0, "{drive:?} went {went}");
    }

    #[test]
    fn a_bullet_flies_straight_and_level_and_is_gone_at_its_range() {
        let car = tuning();
        let from = Vec3::new(1.0, 0.9, -2.0);
        let bullet = Bullet {
            from,
            velocity: Vec3::NEG_Z * car.bullet_speed,
            range: car.bullet_range,
        };
        let mut at = from;
        let mut flown = 0.0;
        while let Some(next) = bullet.flown(at, DT) {
            assert!((next.y - from.y).abs() < 1e-5 && (next.x - from.x).abs() < 1e-5, "{next}");
            assert!(next.z < at.z);
            at = next;
            flown += DT;
        }
        // Gone within a frame of its range, having flown at its speed.
        let short = car.bullet_range - at.distance(from);
        assert!(short >= 0.0 && short <= car.bullet_speed * DT + 1e-3, "{short} m short");
        assert!((flown - car.bullet_range / car.bullet_speed).abs() <= 2.0 * DT, "{flown} s");
    }

    #[test]
    fn the_gun_fires_as_the_button_goes_down_and_then_on_the_beat() {
        let every = 0.12;
        let mut gun = Gun::default();
        assert!(gun.fires(DT, every));
        let mut shots = 0;
        for _ in 0..(1.2 / DT) as usize {
            if gun.fires(DT, every) {
                shots += 1;
            }
        }
        // Ten beats in 1.2 s, give or take a frame's rounding.
        assert!((9..=10).contains(&shots), "{shots} shots");
        // Let go and pressed again: at once.
        gun.let_go();
        assert!(gun.fires(DT, every));
        assert!(!gun.fires(DT, every));
    }

    #[test]
    fn the_model_is_what_the_shape_says_with_four_wheels_facing_blender_s_front() {
        let (pieces, low, high) = car::tests::hajuns_car(MODEL);
        assert_eq!(car::front(low, high), Vec2::NEG_Y);
        let wheels = car::tests::wheels_of(MODEL);
        assert_eq!(wheels.len(), 4, "{wheels:?}");
        let front = Vec2::NEG_Y;
        let (base, mid) = car::axles(wheels.iter().map(|&(low, high)| ((low + high) * 0.5).xz().dot(front)));
        let along = |corner: &Vec3| corner.xz().dot(front);
        let read = Shape {
            front,
            lowest: low.y,
            ahead: pieces.iter().flatten().flatten().map(along).fold(f32::NEG_INFINITY, f32::max),
            behind: -pieces.iter().flatten().flatten().map(along).fold(f32::INFINITY, f32::min),
            half_width: car::half(low, high).y,
            base,
            rear: base * 0.5 - mid,
        };
        let said = Shape::default();
        for (name, a, b) in [
            ("lowest", read.lowest, said.lowest),
            ("ahead", read.ahead, said.ahead),
            ("behind", read.behind, said.behind),
            ("half_width", read.half_width, said.half_width),
            ("base", read.base, said.base),
            ("rear", read.rear, said.rear),
        ] {
            assert!((a - b).abs() < 0.015, "{name}: the model says {a}, the shape {b}");
        }
        // Its wheels stand on the ground, and its front wheels are the two
        // ahead of its origin.
        assert!(read.lowest.abs() < 0.01);
        let ahead: Vec<bool> = wheels.iter().map(|&(l, h)| ((l + h) * 0.5).xz().dot(front) > mid).collect();
        assert_eq!(ahead.iter().filter(|&&front| front).count(), 2);
    }
}
