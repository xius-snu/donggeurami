//! Round Town and your home, two islands played third person with WASD or
//! touch.
//!
//! The app opens in the town, with you and seven others already standing in
//! it (`lobby`). A button top right takes you home and back again (`hud`). The
//! House Builder, at the door of their house in the town, asks whether you
//! want to play their game (`builder`).
//!
//! `#[bevy_main]` generates the `android_main` entry point. Desktop and iOS
//! both reach [`main`] through `src/main.rs`; on iOS that binary *is* the app
//! executable, which `mobile/ios/build_rust.sh` drops into the .app bundle.

use bevy::audio::AudioPlugin;
use bevy::gilrs::GilrsPlugin;
use bevy::gltf::GltfPlugin;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::light::cluster::{ClusterConfig, GlobalClusterSettings};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::view::NoIndirectDrawing;
use bevy::window::WindowResolution;
use bevy::world_serialization::WorldInstanceReady;

mod account;
mod ai;
mod build;
mod builder;
mod editor;
mod fountain;
mod hud;
mod island;
mod log_file;
mod lobby;
mod login;
mod map;
mod net;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod pace;
mod shop;
mod sky;
mod tags;
use island::{Island, Islands, Venue};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
#[cfg(any(target_os = "android", target_os = "ios"))]
use bevy::window::{MonitorSelection, WindowMode};
#[cfg(target_os = "android")]
use bevy::window::PrimaryWindow;

const MOVE_SPEED: f32 = 7.0;
const TURN_SPEED: f32 = 22.0;
const JUMP_SPEED: f32 = 8.5;
const GRAVITY: f32 = 22.0;
/// Gravity on the way down: heavier than on the way up, so that a fall is
/// quick without a jump losing any of its height.
const FALL_GRAVITY: f32 = GRAVITY * 1.5;
const PLAYER_HEIGHT: f32 = 1.6;
const PLAYER_RADIUS: f32 = 0.45;
/// How far out from the middle of a body its legs reach, in metres: the
/// model's legs, side by side, are 0.42 m across, where its body and arms
/// are 0.86 ([`PLAYER_RADIUS`]). A body stands on whatever is under its legs
/// (`support`), and below [`HIP_HEIGHT`] its legs are all of it there is to
/// bump into anything (`resolve_player_solids`).
const LEG_RADIUS: f32 = 0.21;
/// How high over its feet a body's legs go before it widens into its body
/// and arms, in metres: the top of the model's legs. It is as wide as it
/// gets by [`WAIST_HEIGHT`].
const HIP_HEIGHT: f32 = 0.65;
const WAIST_HEIGHT: f32 = 0.8;
/// How much higher the ground under a body's legs can be than the ground
/// under the middle of it and still be the one slope, in metres: the bridge,
/// the steepest ground there is at 15°, rises 0.06 m across the legs. Higher
/// than that, the body is standing on an edge (`support`).
const EDGE_RISE: f32 = 0.15;
/// Points round the middle of a body's feet, as far out as its legs reach.
const UNDER_THE_LEGS: [Vec2; 8] = {
    let (r, d) = (LEG_RADIUS, LEG_RADIUS * std::f32::consts::FRAC_1_SQRT_2);
    [
        Vec2::new(r, 0.0),
        Vec2::new(d, d),
        Vec2::new(0.0, r),
        Vec2::new(-d, d),
        Vec2::new(-r, 0.0),
        Vec2::new(-d, -d),
        Vec2::new(0.0, -r),
        Vec2::new(d, -d),
    ]
};
const TREE_TRUNK_RADIUS: f32 = 1.1;
const TREE_TRUNK_HEIGHT: f32 = 6.5;
const WATER_JUMP_SLOP: f32 = 0.22;
const JUMP_COYOTE: f32 = 0.14;
/// How far under a ceiling a jump stops the head, in metres.
const HEAD_CLEARANCE: f32 = 0.02;
const JUMP_BUFFER: f32 = 0.12;
const LOOK_HEIGHT: f32 = PLAYER_HEIGHT * 0.75;
/// The height of the grass in `circlemap1.glb`, where the town stands.
const LAND_TOP: f32 = 0.0;
/// The sea, everywhere the island is not: z = -0.5 in Blender, which leaves the
/// lowest ground on either island — the town's roads, the home's sand, both at
/// z = -0.2 — just clear of it.
const WATER_Y: f32 = -0.5;
const WATER_WADE: f32 = 0.58;
const WATER_BOB_AMP: f32 = 0.07;
const WATER_BOB_SPEED: f32 = 1.7;
const WATER_MOVE_SCALE: f32 = 0.72;
/// The highest a body steps up onto without jumping, in metres: anything
/// under half a metre. Half a metre itself takes a jump, like the fountain's
/// rim out of its basin (Hajun, 2026-09-30); the 5 mm short of it keep a rise
/// of exactly that from going either way as its height rounds. It was 0.75
/// until 2026-09-30, when Hajun made it 0.5. It is also the height under which
/// nothing in the way counts as a wall (`Island::walls`), so that what is
/// taller than this has to be jumped onto or walked round.
const LAND_STEP_UP: f32 = 0.495;
const OCEAN_LIMIT: f32 = 580.0;
const CAMERA_DISTANCE: f32 = 10.0;
const CAMERA_DISTANCE_MIN: f32 = 2.5;
const CAMERA_DISTANCE_MAX: f32 = 15.0;
const ZOOM_WHEEL: f32 = 0.12;
const CAMERA_FAR: f32 = 4000.0;
const STICK_DEADZONE: f32 = 0.18;
const STICK_SIZE_VMIN: f32 = 22.0;
const STICK_LEFT_VW: f32 = 8.0;
const STICK_BOTTOM_VH: f32 = 11.0;
const JUMP_SIZE_VMIN: f32 = 18.0;
const JUMP_RIGHT_VW: f32 = 8.0;
const JUMP_BOTTOM_VH: f32 = 11.0;
const LOOK_SENSITIVITY: f32 = 0.0045;
const PITCH_MIN: f32 = -1.22;
const PITCH_MAX: f32 = 1.20;
const CAMERA_LOOK_UP_DISTANCE: f32 = 2.72;
const CAMERA_LOOK_UP_HEIGHT: f32 = 0.58;
const LOOK_UP_FOCUS_HEIGHT: f32 = PLAYER_HEIGHT * 0.9;
const LOOK_UP_EXTRA_PITCH: f32 = 0.52;
const CAMERA_FOV: f32 = std::f32::consts::FRAC_PI_4;
const CAMERA_LOOK_UP_FOV: f32 = 1.1;
/// How high over your feet the camera sees from when it is your eyes: in
/// first person, and in third person once it is brought in so near that you
/// are out of sight (`toward_your_eyes`). Higher than the top of your head,
/// 1.7 m, as a good deal taller person would see: Hajun asked for the view to
/// feel like that (2026-10-03). The height is mine, to be tried: Hajun's
/// houses and furniture are built big, a door 3.35 m high and a counter 1.5
/// m, and from here they look the size they would to someone of a person's
/// size. It was 1.44 m, nine tenths of a body's height, until then.
pub(crate) const EYE_HEIGHT: f32 = 2.2;
/// How far out from where you are out of sight the camera, brought in near
/// you, starts to rise toward your eyes, in metres. It rises the nearer it
/// comes, and is all the way up as you go out of sight.
const EYES_RISE: f32 = 0.6;
/// How much the camera takes in, top to bottom, in first person, until a pinch
/// or the wheel zooms it: about 66°, which is 98° across a 16:9 screen and
/// 110° across a 20:9 phone. Wide, so that while you build, a piece and the
/// room it goes in are on the screen together.
const FIRST_PERSON_FOV: f32 = 1.15;
/// How far first person zooms: in to about 23° top to bottom, which shows the
/// world three times the size, and out to about 77°. Any wider and the edges
/// of the screen stretch out of shape.
const FIRST_PERSON_FOV_MIN: f32 = 0.4;
const FIRST_PERSON_FOV_MAX: f32 = 1.35;
/// How far the camera keeps from the island, in metres: further than the
/// corners of its near plane reach from it, 0.18 m at the widest, looking up
/// on a 20:9 phone, so that a wall or a ceiling it is pressed up against is
/// never cut open on screen. No further, so that turning it into a wall you
/// stand against brings it in no faster than it has to.
const CAMERA_RADIUS: f32 = 0.2;
/// How quickly the camera backs out again once the way behind it is clear:
/// most of the way in this many seconds, and never slower than this many
/// metres a second, so that it keeps up with you turning it round a room.
const CAMERA_BACK_OUT_SECS: f32 = 0.3;
const CAMERA_BACK_OUT_SPEED: f32 = 12.0;
/// How quickly it comes in when something comes between it and you, the same
/// way: swept in over a moment rather than there at once. Hajun found it
/// jumping in front of you as a wall came between (2026-10-08). It only lags
/// where it has room to: in free air behind what is in the way, never in it.
const CAMERA_PULL_IN_SECS: f32 = 0.12;
const CAMERA_PULL_IN_SPEED: f32 = 10.0;
/// This close to as far out as you zoomed it, it is as far out as you zoomed
/// it.
const CAMERA_SETTLED: f32 = 1e-3;
/// How far past what you bump into with the camera still counts as inside
/// you, in metres: your arms and the top of your head reach that far.
const CAMERA_INSIDE: f32 = 0.15;
/// Further than this from one frame to the next, you went somewhere rather
/// than walked there, and the camera starts again behind you.
const CAMERA_CUT: f32 = 3.0;
const WALK_STRIDE_FREQ: f32 = 7.6;
const WALK_THIGH: f32 = 0.42;
const WALK_SHIN: f32 = 0.28;
const WALK_ARM: f32 = 0.55;
const WALK_ARM_HANG: f32 = 1.02;
const WALK_BLEND: f32 = 9.0;

/// The one player of the eight that this device steers: its stick and keys
/// move them, and the camera follows them.
#[derive(Component)]
struct Player;

/// What a body is trying to do this frame, whoever is steering it: this
/// device's stick and keys for [`Player`], an AI's
/// [`Steering`](ai::Steering) for the others. [`move_bodies`] carries it out
/// the same way for both.
#[derive(Component, Default)]
struct Intent {
    /// Which way to walk, flat on the ground. Its length is how fast, from 0
    /// for standing still to 1 for full speed.
    walk: Vec3,
    /// A jump, asked for this frame.
    jump: bool,
}

/// A body that will not walk off dry land into the sea, nor down anything it
/// could not step back up ([`strands`]). An AI does not know it could jump
/// back out, so it would stand in the water, or the fountain, for good.
#[derive(Component)]
struct StaysAshore;

#[derive(Component, Default)]
struct PlayerJump {
    velocity_y: f32,
    coyote: f32,
    buffer: f32,
}

#[derive(Component, Default)]
struct WalkCycle {
    phase: f32,
    weight: f32,
    speed: f32,
}

#[derive(Component, Clone, Copy)]
struct WalkBone {
    kind: WalkBoneKind,
    rest_rotation: Quat,
    /// The player this bone belongs to, whose [`WalkCycle`] swings it.
    owner: Entity,
}

#[derive(Clone, Copy)]
enum WalkBoneKind {
    ThighL,
    ThighR,
    ShinL,
    ShinR,
    ArmL,
    ArmR,
}

#[derive(Component, Clone, Copy)]
pub(crate) struct Collider {
    shape: ColliderShape,
}

#[derive(Clone, Copy)]
pub(crate) enum ColliderShape {
    Cylinder { radius: f32, height: f32 },
    #[expect(
        dead_code,
        reason = "the island was the only box, and it is a model now; kept for box-shaped catalogue kinds"
    )]
    Aabb { half_extents: Vec3 },
}

#[derive(Component)]
struct ThirdPersonCamera;

/// How much nearer you the camera stands than you zoomed it, to keep the
/// island out from between it and you. It always points the way you turned
/// it: only how far out it stands along that way ever changes.
#[derive(Component, Default, Clone, Copy, PartialEq)]
struct CameraFit {
    /// How far out it stands from the point on you it looks at, while that is
    /// nearer than you zoomed it: eased in when something comes in the way,
    /// and back out once nothing is.
    distance: Option<f32>,
    /// What it looked at last frame.
    pivot: Option<Vec3>,
}

impl CameraFit {
    /// How far out the camera stands `dt` seconds on, now that the island
    /// leaves it `clear` of the `full` way out you zoomed it: all of that at
    /// once if it is starting again. Coming in, this can be further out than
    /// `clear`, behind what is in the way; [`place_camera`] sees that it is
    /// not in it.
    fn distance_toward(self, clear: f32, full: f32, cut: bool, dt: f32) -> Option<f32> {
        if cut {
            return (clear < full - CAMERA_SETTLED).then_some(clear);
        }
        match self.distance {
            None if clear >= full - CAMERA_SETTLED => None,
            shown => {
                let shown = shown.unwrap_or(full).min(full);
                let (secs, speed) = if clear < shown {
                    (CAMERA_PULL_IN_SECS, CAMERA_PULL_IN_SPEED)
                } else {
                    (CAMERA_BACK_OUT_SECS, CAMERA_BACK_OUT_SPEED)
                };
                let gap = (clear - shown).abs();
                let most = gap * (1.0 - (-dt / secs).exp());
                let step = most.max((speed * dt).min(gap));
                let eased = if clear < shown { shown - step } else { shown + step };
                (eased < full - CAMERA_SETTLED).then_some(eased)
            }
        }
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Component)]
struct JoystickKnob;

#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Component)]
struct JumpButtonVisual;

#[derive(Resource, Default)]
struct TouchControls {
    stick_id: Option<u64>,
    stick_value: Vec2,
    look_id: Option<u64>,
    look_last: Vec2,
    jump_id: Option<u64>,
    jump_just_pressed: bool,
    pinch_id: Option<u64>,
    pinch_last_dist: f32,
    /// The touch the editor is following. A press on one of the edit buttons
    /// claims one outright so that dragging an object does not also swing the
    /// camera; otherwise this is the same touch as `look_id`, and whether it
    /// turns out to be a tap or a look is decided when it lifts.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    edit_id: Option<u64>,
}

/// Whether the camera is your eyes rather than following you round: while you
/// build and while you rate houses in House Builder, if you have swapped to it
/// (`builder`).
#[derive(Resource, Default, PartialEq)]
pub(crate) struct FirstPerson(pub(crate) bool);

#[derive(Resource)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
    /// How much the camera takes in, top to bottom, while it is your eyes
    /// ([`FirstPerson`]): what a pinch or the wheel zooms there, the way it
    /// zooms `distance` in third person. Each keeps its own.
    fov: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.38,
            distance: CAMERA_DISTANCE,
            fov: FIRST_PERSON_FOV,
        }
    }
}

impl OrbitCamera {
    /// Behind `body`, looking the way it faces, at the starting tilt and
    /// distance.
    fn behind(body: &Transform) -> Self {
        let back = body.back();
        Self {
            yaw: back.x.atan2(back.z),
            ..default()
        }
    }

    /// Zooms in, for a `ratio` under 1, or out: in third person the camera
    /// stands that much nearer you or further off, and in first person it
    /// takes in that much less or more, so that the world on the screen grows
    /// or shrinks with the fingers.
    fn zoom_by_ratio(&mut self, ratio: f32, first_person: bool) {
        if first_person {
            let half = ((self.fov * 0.5).tan() * ratio).atan();
            self.fov = (half * 2.0).clamp(FIRST_PERSON_FOV_MIN, FIRST_PERSON_FOV_MAX);
        } else {
            self.distance =
                (self.distance * ratio).clamp(CAMERA_DISTANCE_MIN, CAMERA_DISTANCE_MAX);
        }
    }

    /// Turns the camera by a swipe or a move of the mouse of `delta` pixels.
    /// In first person, zoomed in, the same swipe turns it less, so that the
    /// world still moves across the screen as fast as it does unzoomed rather
    /// than racing past the finger.
    fn turn(&mut self, delta: Vec2, first_person: bool) {
        let zoom = if first_person {
            (self.fov * 0.5).tan() / (FIRST_PERSON_FOV * 0.5).tan()
        } else {
            1.0
        };
        let per_pixel = LOOK_SENSITIVITY * zoom;
        self.yaw -= delta.x * per_pixel;
        self.pitch = (self.pitch + delta.y * per_pixel).clamp(PITCH_MIN, PITCH_MAX);
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
struct HudLayout {
    stick_center: Vec2,
    stick_radius: f32,
    jump_center: Vec2,
    jump_radius: f32,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn hud_layout(window: &Window) -> HudLayout {
    let width = window.width();
    let height = window.height();
    let vmin = width.min(height);
    let stick_size = vmin * (STICK_SIZE_VMIN / 100.0);
    let stick_radius = stick_size * 0.5;
    let jump_size = vmin * (JUMP_SIZE_VMIN / 100.0);
    let jump_radius = jump_size * 0.5;
    HudLayout {
        stick_center: Vec2::new(
            width * (STICK_LEFT_VW / 100.0) + stick_radius,
            height - height * (STICK_BOTTOM_VH / 100.0) - stick_radius,
        ),
        stick_radius,
        jump_center: Vec2::new(
            width - width * (JUMP_RIGHT_VW / 100.0) - jump_radius,
            height - height * (JUMP_BOTTOM_VH / 100.0) - jump_radius,
        ),
        jump_radius,
    }
}

#[bevy_main]
pub fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window_settings()),
                ..default()
            })
            .set(GltfPlugin {
                // glTF calls +Z "forward"; Bevy calls -Z "forward". Without this
                // every imported model faces the opposite way from the Transform
                // that carries it. Turning it on means an object modelled facing
                // -Y in Blender (Numpad 1, the standard front view) also faces
                // `Transform::forward()` here.
                convert_coordinates: GltfConvertCoordinates {
                    rotate_scene_entity: true,
                    rotate_meshes: true,
                },
                ..default()
            })
            // The town makes no sound and reads no gamepad. Left in, the one
            // keeps an audio stream open playing silence for as long as the
            // app is, and the other wakes a thread every 8 ms on Windows to
            // look for controllers. Put them back when there is a use for them.
            .disable::<AudioPlugin>()
            .disable::<GilrsPlugin>()
            .set(LogPlugin {
                // An iPhone keeps its log where only a Mac can read it, so it
                // writes one of its own as well, which the Files app shows.
                #[cfg(target_os = "ios")]
                custom_layer: log_file::in_documents,
                ..default()
            }),
    )
    .init_resource::<TouchControls>()
    .init_resource::<OrbitCamera>()
    .init_resource::<FirstPerson>()
    .add_systems(Startup, (setup_world, cluster_lights_on_the_cpu));
    sky::plugin(&mut app);
    island::plugin(&mut app);
    // The town's fountain, running, and its jet.
    fountain::plugin(&mut app);
    // Everything standing on the town comes from the town save, so the map
    // has to be there before the editor can be pointed at any of it.
    map::plugin(&mut app);
    editor::plugin(&mut app);
    // Everyone in the town, you included, with the computer in every seat no
    // person has taken.
    lobby::plugin(&mut app);
    ai::plugin(&mut app);
    // Online: the server, reached in the background, and everyone else in
    // your room of it.
    net::plugin(&mut app);
    tags::plugin(&mut app);
    // Your account online, and the way to delete it.
    account::plugin(&mut app);
    // Your balance, the button that takes you home and back, and the welcome.
    hud::plugin(&mut app);
    // The House Builder in the town, and the game they ask you to play: the
    // shop to build with, and what is built.
    builder::plugin(&mut app);
    shop::plugin(&mut app);
    build::plugin(&mut app);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    app.add_systems(Startup, setup_hud).add_systems(
        Update,
        (
            // This is also where a touch is handed to the editor, so it has to
            // land before the editor reads the pointer.
            read_touch_controls.before(editor::EditSystems),
            update_joystick_knob,
            update_jump_visual,
        ),
    );

    #[cfg(target_os = "ios")]
    app.add_systems(Update, read_pinch_gesture);

    #[cfg(target_os = "android")]
    app.add_systems(PreUpdate, keep_ui_size);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        app.add_systems(Startup, lock_cursor).add_systems(
            Update,
            (
                // Around House Builder's keys: Escape is a dialog's no while
                // one is up, and the cursor is let go for as long as it is.
                grab_cursor.before(builder::GameSystems),
                cursor_for_dialogs.after(builder::GameSystems),
                read_mouse_look,
                read_mouse_zoom,
            ),
        );
        // No more frames than a fast screen needs.
        pace::plugin(&mut app);
    }

    app.add_systems(
        Update,
        (read_player_input, move_bodies, animate_walk, follow_camera).chain(),
    );
    // Last, so that every schedule the plugins above made is there to change.
    one_thread_per_world(&mut app);
    app.run();
}

/// Runs every schedule's systems one after another, on the thread that runs
/// the schedule, instead of handing them out to Bevy's pool of worker threads.
///
/// This world is small, and waking the workers, hundreds of times a frame, cost
/// more than the work they were woken for. At 120 frames a second the phone's
/// four workers took 72% of a core between them; with this the whole game
/// took 87% of a core instead of 121% (the laptop: 46% instead of 87%), and
/// both still made every frame. The game world and the render world still run
/// side by side, on a thread each, and systems that split their own work
/// across the pool (`par_iter`) still do.
fn one_thread_per_world(app: &mut App) {
    use bevy::ecs::schedule::{Schedules, SingleThreadedExecutor};
    fn each_schedule(world: &mut World) {
        for (_, schedule) in world.resource_mut::<Schedules>().iter_mut() {
            schedule.set_executor(SingleThreadedExecutor::new());
        }
    }
    each_schedule(app.world_mut());
    // Before `app.run()` moves the render world onto its own thread.
    if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
        each_schedule(render.world_mut());
    }
}

/// How much of the screen's width and height the game is drawn at on Android.
/// `MainActivity` has the phone draw it that much smaller and stretch it over
/// the screen (its `RENDER_SCALE`, which this must match), and hands touches
/// over scaled down to suit.
#[cfg(target_os = "android")]
const RENDER_SCALE: f32 = 0.75;

/// Keeps every button and line of text the size it is on the full screen,
/// though the game is drawn smaller (`RENDER_SCALE`): there are that many
/// fewer pixels to each of the UI's.
#[cfg(target_os = "android")]
fn keep_ui_size(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    for mut window in &mut windows {
        let scale = window.resolution.base_scale_factor() * RENDER_SCALE;
        if window.resolution.scale_factor_override() != Some(scale) {
            window.resolution.set_scale_factor_override(Some(scale));
        }
    }
}

fn window_settings() -> Window {
    Window {
        title: "동그라미타운".into(),
        resolution: WindowResolution::new(1280, 720),
        #[cfg(any(target_os = "android", target_os = "ios"))]
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        #[cfg(any(target_os = "android", target_os = "ios"))]
        resizable: true,
        #[cfg(target_os = "ios")]
        recognize_pinch_gesture: true,
        #[cfg(target_os = "ios")]
        recognize_rotation_gesture: true,
        #[cfg(target_os = "ios")]
        prefers_home_indicator_hidden: true,
        #[cfg(target_os = "ios")]
        prefers_status_bar_hidden: true,
        ..default()
    }
}

/// The camera. The players, you among them, come from the lobby
/// (`lobby::spawn_players`).
fn setup_world(mut commands: Commands) {
    commands.spawn((
        ThirdPersonCamera,
        CameraFit::default(),
        Camera3d::default(),
        IsDefaultUiCamera,
        Projection::from(PerspectiveProjection {
            fov: CAMERA_FOV,
            far: CAMERA_FAR,
            ..default()
        }),
        Transform::from_xyz(0.0, 4.0, CAMERA_DISTANCE)
            .looking_at(Vec3::new(0.0, LOOK_HEIGHT, 0.0), Vec3::Y),
        // Bevy drives its render phases with indirect draws and GPU culling
        // wherever the adapter claims to support them. The Adreno 840 claims
        // it, but nothing queued into the sorted transparent phase then reaches
        // the screen — the sea, the clouds, the sun and moon glow and the stars
        // were all missing, with no validation error to show for it, while the
        // opaque phase drew correctly. This drops the camera back to direct
        // draws, which costs nothing at this scene's size and brings the
        // transparent pass back. Verified on an SM-S948N.
        #[cfg(any(target_os = "android", target_os = "ios"))]
        NoIndirectDrawing,
        // Clustering sorts point and spot lights, light probes and decals into
        // cells of the view, and the world has none of them: the sun and the
        // fill are directional lights, which light everything and are never
        // clustered. One cell instead of Bevy's thousands is the same picture
        // for less work. `ClusterConfig::None` would be less still, but Bevy
        // 0.19 then fails to create its "clustering dummy texture" and quits
        // on the first frame. A point light added later needs this taken out.
        ClusterConfig::Single,
    ));
}

/// Sorts lights into the camera's clusters on the CPU, as Bevy already does on
/// Android, rather than on the GPU, as it does everywhere else.
///
/// There is nothing to sort: the world's only lights are the sun and the fill,
/// directional lights, which are never clustered (see `ClusterConfig::Single`
/// in `setup_world`). On the GPU the sorting is still compute and raster
/// passes every frame, and it is a path Bevy 0.19 takes on an iPhone but
/// neither on Android nor in the iOS simulator. The iPhone's 1.0.1 never drew
/// a frame (2026-10-08): the screen stayed black, and the server's log shows
/// that it made its login and never joined the game, which is what a render
/// thread that dies on its first frame does (Bevy then asks to quit, which
/// iOS does not allow, and stops). That path is the suspect, not yet proven
/// on the phone; with it off, every platform takes the one Android takes.
fn cluster_lights_on_the_cpu(settings: Option<ResMut<GlobalClusterSettings>>) {
    if let Some(mut settings) = settings {
        settings.gpu_clustering = None;
    }
}

/// The stick and the jump button.
#[cfg(any(target_os = "android", target_os = "ios"))]
fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Vw(STICK_LEFT_VW),
                bottom: Val::Vh(STICK_BOTTOM_VH),
                width: Val::VMin(STICK_SIZE_VMIN),
                height: Val::VMin(STICK_SIZE_VMIN),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.18)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.35)),
        ))
        .with_child((
            JoystickKnob,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(42.0),
                height: Val::Percent(42.0),
                left: Val::Percent(29.0),
                top: Val::Percent(29.0),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.42)),
        ));

    commands
        .spawn((
            JumpButtonVisual,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Vw(JUMP_RIGHT_VW),
                bottom: Val::Vh(JUMP_BOTTOM_VH),
                width: Val::VMin(JUMP_SIZE_VMIN),
                height: Val::VMin(JUMP_SIZE_VMIN),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                overflow: Overflow::clip(),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.82, 0.2, 0.32)),
            BorderColor::all(Color::srgba(1.0, 0.85, 0.3, 0.5)),
        ))
        .with_children(|parent| {
            parent.spawn(jump_icon_bar(16.0, 32.0, 38.0, 11.0, -40.0));
            parent.spawn(jump_icon_bar(46.0, 32.0, 38.0, 11.0, 40.0));
            parent.spawn(jump_icon_bar(44.5, 40.0, 11.0, 28.0, 0.0));
        });
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn jump_icon_bar(left: f32, top: f32, width: f32, height: f32, rotation_deg: f32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(left),
            top: Val::Percent(top),
            width: Val::Percent(width),
            height: Val::Percent(height),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        UiTransform::from_rotation(Rot2::degrees(rotation_deg)),
    )
}

fn flatten_basis(transform: &Transform) -> (Vec3, Vec3) {
    let mut forward = *transform.forward();
    forward.y = 0.0;
    let forward = forward.normalize_or_zero();
    let mut right = *transform.right();
    right.y = 0.0;
    let right = right.normalize_or_zero();
    if forward.length_squared() <= f32::EPSILON {
        (Vec3::NEG_Z, Vec3::X)
    } else {
        (forward, right)
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn in_circle(point: Vec2, center: Vec2, radius: f32) -> bool {
    point.distance_squared(center) <= radius * radius
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn read_touch_controls(
    touches: Res<Touches>,
    windows: Query<&Window>,
    mut controls: ResMut<TouchControls>,
    mut orbit: ResMut<OrbitCamera>,
    first_person: Res<FirstPerson>,
    menu: Res<editor::EditMenu>,
    presses: hud::Presses,
    mut pointer: ResMut<editor::EditPointer>,
) {
    controls.jump_just_pressed = false;
    pointer.clear_edges();
    let Ok(window) = windows.single() else {
        return;
    };
    let layout = hud_layout(window);

    let released = |id: u64| {
        touches.just_released(id) || touches.just_canceled(id) || touches.get_pressed(id).is_none()
    };

    if let Some(id) = controls.stick_id
        && released(id)
    {
        controls.stick_id = None;
        controls.stick_value = Vec2::ZERO;
    }
    if let Some(id) = controls.look_id
        && released(id)
    {
        controls.look_id = None;
        if let Some(pinch_id) = controls.pinch_id
            && let Some(touch) = touches.get_pressed(pinch_id)
        {
            controls.look_id = Some(pinch_id);
            controls.look_last = touch.position();
            controls.pinch_id = None;
            controls.pinch_last_dist = 0.0;
        }
    }
    if let Some(id) = controls.pinch_id
        && released(id)
    {
        controls.pinch_id = None;
        controls.pinch_last_dist = 0.0;
        if let Some(look_id) = controls.look_id
            && let Some(touch) = touches.get_pressed(look_id)
        {
            controls.look_last = touch.position();
        }
    }
    if let Some(id) = controls.jump_id
        && released(id)
    {
        controls.jump_id = None;
    }
    if let Some(id) = controls.edit_id
        && released(id)
    {
        controls.edit_id = None;
        pointer.release();
    }

    for touch in touches.iter_just_pressed() {
        let pos = touch.position();
        let id = touch.id();
        // Every press dismisses an open menu unless it is on one of its
        // buttons, so the editor hears about all of them.
        pointer.note_press();
        // A press on a button over the world — the one top right, House
        // Builder's, the dark behind a dialog — is the button's alone. It
        // hears about it through picking, the same way it hears a mouse.
        if presses.claimed(window, pos) {
            continue;
        }
        // A press on the menu, or on what it is open on to drag it, belongs to
        // the editor alone. The menu's buttons come before the stick and the
        // jump button, and those before the thing itself: something as big as
        // a house right in front of you is drawn under both of them, and they
        // still have to work.
        let on_stick = in_circle(pos, layout.stick_center, layout.stick_radius * 1.15);
        let on_jump = in_circle(pos, layout.jump_center, layout.jump_radius * 1.2);
        let for_editor =
            menu.on_button(window, pos) || (!on_stick && !on_jump && menu.on_thing(pos));
        if controls.edit_id.is_none() && for_editor {
            controls.edit_id = Some(id);
            pointer.warp(pos);
            pointer.press();
        } else if controls.stick_id.is_none() && on_stick {
            controls.stick_id = Some(id);
        } else if controls.jump_id.is_none() && on_jump {
            controls.jump_id = Some(id);
            controls.jump_just_pressed = true;
        } else if controls.look_id.is_none() {
            controls.look_id = Some(id);
            controls.look_last = pos;
            // The same drag doubles as the editor's pointer: a short one is a
            // tap on whatever is under it, a long one is a look.
            if controls.edit_id.is_none() {
                controls.edit_id = Some(id);
                pointer.warp(pos);
                pointer.press();
            }
        } else if controls.pinch_id.is_none() {
            controls.pinch_id = Some(id);
            controls.pinch_last_dist = 0.0;
        }
    }

    if let Some(id) = controls.edit_id
        && let Some(touch) = touches.get_pressed(id)
    {
        pointer.track(touch.position());
    }

    if let Some(id) = controls.stick_id
        && let Some(touch) = touches.get_pressed(id)
    {
        let delta = touch.position() - layout.stick_center;
        let raw = Vec2::new(delta.x, -delta.y) / layout.stick_radius;
        controls.stick_value = raw.clamp_length_max(1.0);
    } else {
        controls.stick_value = Vec2::ZERO;
    }

    let look_pos = controls
        .look_id
        .and_then(|id| touches.get_pressed(id))
        .map(|touch| touch.position());
    let pinch_pos = controls
        .pinch_id
        .and_then(|id| touches.get_pressed(id))
        .map(|touch| touch.position());

    if let (Some(a), Some(b)) = (look_pos, pinch_pos) {
        let dist = a.distance(b);
        if dist > 8.0 && controls.pinch_last_dist > 8.0 {
            let ratio = (controls.pinch_last_dist / dist).clamp(0.82, 1.22);
            orbit.zoom_by_ratio(ratio, first_person.0);
        }
        controls.pinch_last_dist = dist;
        controls.look_last = a;
    } else {
        controls.pinch_last_dist = 0.0;
        if let Some(pos) = look_pos {
            let delta = pos - controls.look_last;
            controls.look_last = pos;
            orbit.turn(delta, first_person.0);
        }
    }
}

/// Takes the cursor for looking round, as the app opens. `RT_FREE_CURSOR=1`
/// leaves it alone, for running copies of the game side by side to try
/// playing online: each would otherwise take the mouse from the other.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn lock_cursor(mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if std::env::var_os("RT_FREE_CURSOR").is_some() {
        return;
    }
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn grab_cursor(
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Res<hud::DialogUp>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // With a dialog up, Escape is its no (`builder`), and the cursor is the
    // dialog's to give back when it goes.
    if !keyboard.just_pressed(KeyCode::Escape) || dialog.0 {
        return;
    }
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    let locked = cursor.grab_mode == CursorGrabMode::Locked;
    cursor.grab_mode = if locked {
        CursorGrabMode::None
    } else {
        CursorGrabMode::Locked
    };
    cursor.visible = locked;
}

/// Lets the cursor go while a dialog is up, so that its buttons can be
/// clicked, or while a piece is being put down in House Builder, so that it
/// can be dragged; and takes it back once neither is, if it was taken before.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn cursor_for_dialogs(
    dialog: Res<hud::DialogUp>,
    wants: Res<hud::WantsPointer>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut was_taken: Local<bool>,
    mut let_go: Local<bool>,
) {
    let free = dialog.0 || wants.0;
    if free == *let_go {
        return;
    }
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    *let_go = free;
    if free {
        *was_taken = cursor.grab_mode == CursorGrabMode::Locked;
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    } else if std::mem::take(&mut *was_taken) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_mouse_look(
    cursors: Query<&CursorOptions, With<PrimaryWindow>>,
    first_person: Res<FirstPerson>,
    mut motion: MessageReader<MouseMotion>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let looking = cursors
        .single()
        .map(|cursor| cursor.grab_mode == CursorGrabMode::Locked)
        .unwrap_or(false);
    if !looking {
        motion.clear();
        return;
    }
    for event in motion.read() {
        orbit.turn(event.delta, first_person.0);
    }
}

#[cfg(target_os = "ios")]
fn read_pinch_gesture(
    first_person: Res<FirstPerson>,
    mut pinch: MessageReader<bevy::input::gestures::PinchGesture>,
    mut orbit: ResMut<OrbitCamera>,
) {
    for event in pinch.read() {
        orbit.zoom_by_ratio((1.0 - event.0).clamp(0.7, 1.4), first_person.0);
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_mouse_zoom(
    dialog: Res<hud::DialogUp>,
    first_person: Res<FirstPerson>,
    mut scroll: MessageReader<MouseWheel>,
    mut orbit: ResMut<OrbitCamera>,
) {
    // With a dialog up the wheel is its: the shop scrolls with it.
    if dialog.0 {
        scroll.clear();
        return;
    }
    for event in scroll.read() {
        let lines = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
        };
        orbit.zoom_by_ratio((1.0 - lines * ZOOM_WHEEL).clamp(0.7, 1.4), first_person.0);
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn update_joystick_knob(
    controls: Res<TouchControls>,
    mut knobs: Query<&mut Node, With<JoystickKnob>>,
) {
    let Ok(mut node) = knobs.single_mut() else {
        return;
    };
    let max = 29.0;
    let left = Val::Percent(max + controls.stick_value.x * max);
    let top = Val::Percent(max - controls.stick_value.y * max);
    // Only when the knob has moved: a changed node has the whole screen's
    // layout worked out again.
    if node.left != left || node.top != top {
        node.left = left;
        node.top = top;
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn update_jump_visual(
    controls: Res<TouchControls>,
    mut buttons: Query<&mut BackgroundColor, With<JumpButtonVisual>>,
) {
    let Ok(mut color) = buttons.single_mut() else {
        return;
    };
    let held = controls.jump_id.is_some();
    color.set_if_neq(BackgroundColor(if held {
        Color::srgba(1.0, 0.82, 0.2, 0.55)
    } else {
        Color::srgba(1.0, 0.82, 0.2, 0.32)
    }));
}

fn wading(pos: Vec3) -> bool {
    pos.y < WATER_Y + 0.25
}

/// What the feet come to rest on at `pos`: the highest ground on the island
/// within a step of them, or, with none, the sea bed the player wades on.
///
/// The ground under the middle of the body, unless the ground under its
/// legs, round that, is higher by more than a slope makes it ([`EDGE_RISE`]):
/// then it is standing on an edge, its middle out past it, and the edge holds
/// it up. Until 2026-10-08 only the middle counted. A body that came down
/// with its legs on an edge and its middle just past it went on down past it,
/// half into it, and once the edge was over its knees, was shoved out
/// sideways and dropped (Hajun: it "glitch steps").
fn support(island: &Island, pos: Vec3, elapsed: f32) -> f32 {
    let sea = water_rest_y(elapsed);
    let reach = pos.y + LAND_STEP_UP;
    let under = |at: Vec2| island.floor(at, reach).map_or(sea, |ground| ground.max(sea));
    let middle = under(pos.xz());
    // Standing on it, as nearly everyone nearly always is, that is all there
    // is to it: only a body off it, in the air or out over an edge, looks
    // under its legs. Beside the fountain's tower that is eight more looks
    // through its thousands of triangles.
    if pos.y <= middle {
        return middle;
    }
    let legs = UNDER_THE_LEGS
        .iter()
        .map(|&round| under(pos.xz() + round))
        .fold(middle, f32::max);
    if legs > middle + EDGE_RISE { legs } else { middle }
}

/// The dry ground a body at `pos` would stand on — the highest within a step
/// of its feet — or `None` over open sea.
fn dry_ground(island: &Island, pos: Vec3) -> Option<f32> {
    island
        .floor(pos.xz(), pos.y + LAND_STEP_UP)
        .filter(|&ground| ground > WATER_Y)
}

/// Whether a body that cannot jump, walked from `from` to `to`, would be
/// stranded there: off dry land into the sea, or down off something higher
/// than it could step back up, like the fountain's rim into its basin.
fn strands(island: &Island, from: Vec3, to: Vec3) -> bool {
    let Some(here) = dry_ground(island, from) else {
        return false;
    };
    dry_ground(island, to).is_none_or(|there| here - there > LAND_STEP_UP)
}

/// The lowest underside over a body standing at `at` that is higher than its
/// head, at `head`: over every square of the lattice its walls are tested on
/// (`Island::walls`), so that no ceiling it stops short of can still reach
/// into it at the side.
fn headroom(island: &Island, at: Vec2, head: f32) -> Option<f32> {
    let reach = PLAYER_RADIUS + island::LATTICE * 0.5;
    [-reach, 0.0, reach]
        .into_iter()
        .flat_map(|x| [-reach, 0.0, reach].map(|z| at + Vec2::new(x, z)))
        .filter_map(|spot| island.ceiling(spot, head - HEAD_CLEARANCE))
        .reduce(f32::min)
}

fn water_rest_y(elapsed: f32) -> f32 {
    WATER_Y - WATER_WADE + (elapsed * WATER_BOB_SPEED).sin() * WATER_BOB_AMP
}

fn y_overlap(a0: f32, a1: f32, b0: f32, b1: f32) -> bool {
    a0 < b1 && a1 > b0
}

fn aabb_world(center: Vec3, half: Vec3) -> (Vec3, Vec3) {
    (center - half, center + half)
}

fn standing_on_aabb(pos: Vec3, min: Vec3, max: Vec3) -> bool {
    pos.x >= min.x
        && pos.x <= max.x
        && pos.z >= min.z
        && pos.z <= max.z
        && pos.y >= max.y - LAND_STEP_UP
}

fn resolve_circle_aabb(pos: &mut Vec3, radius: f32, min: Vec3, max: Vec3) {
    let closest_x = pos.x.clamp(min.x, max.x);
    let closest_z = pos.z.clamp(min.z, max.z);
    let dx = pos.x - closest_x;
    let dz = pos.z - closest_z;
    let d2 = dx * dx + dz * dz;
    if d2 > 1e-8 {
        let d = d2.sqrt();
        if d < radius {
            let scale = (radius - d) / d;
            pos.x += dx * scale;
            pos.z += dz * scale;
        }
        return;
    }
    let left = pos.x - min.x;
    let right = max.x - pos.x;
    let back = pos.z - min.z;
    let fwd = max.z - pos.z;
    let nearest = left.min(right).min(back).min(fwd);
    if nearest == left {
        pos.x = min.x - radius;
    } else if nearest == right {
        pos.x = max.x + radius;
    } else if nearest == back {
        pos.z = min.z - radius;
    } else {
        pos.z = max.z + radius;
    }
}

fn resolve_cylinders(pos: &mut Vec3, radius: f32, height: f32, other: Vec3, other_radius: f32, other_height: f32) {
    if !y_overlap(pos.y, pos.y + height, other.y, other.y + other_height) {
        return;
    }
    let dx = pos.x - other.x;
    let dz = pos.z - other.z;
    let min_dist = radius + other_radius;
    let d2 = dx * dx + dz * dz;
    if d2 >= min_dist * min_dist {
        return;
    }
    if d2 <= 1e-8 {
        pos.x += min_dist;
        return;
    }
    let d = d2.sqrt();
    let scale = (min_dist - d) / d;
    pos.x += dx * scale;
    pos.z += dz * scale;
}

fn resolve_player_solids(pos: &mut Vec3, solids: &[(Vec3, Collider)], island: &Island) {
    for &(center, collider) in solids {
        match collider.shape {
            ColliderShape::Aabb { half_extents } => {
                let (min, max) = aabb_world(center, half_extents);
                if standing_on_aabb(*pos, min, max) {
                    continue;
                }
                if !y_overlap(pos.y, pos.y + PLAYER_HEIGHT, min.y, max.y) {
                    continue;
                }
                resolve_circle_aabb(pos, PLAYER_RADIUS, min, max);
            }
            ColliderShape::Cylinder { radius, height } => {
                resolve_cylinders(pos, PLAYER_RADIUS, PLAYER_HEIGHT, center, radius, height);
            }
        }
    }
    // Whatever on the island stands higher than a step above the feet: the
    // shore, seen from the water, and fences and trunks from anywhere. Up to
    // the hips only the legs meet it, and from there up the body and arms
    // too: standing on the fountain's middle bowl, the top bowl is at the
    // thighs, clear of the legs, where the whole width of the body would have
    // been shoved off it. The hips are rounded off, half way between the two,
    // so that a body coming down just past an edge is eased off it rather
    // than shoved all at once as the edge reaches its waist.
    let feet = pos.y;
    let shape = [
        (LEG_RADIUS, feet + LAND_STEP_UP, feet + HIP_HEIGHT),
        ((LEG_RADIUS + PLAYER_RADIUS) * 0.5, feet + HIP_HEIGHT, feet + WAIST_HEIGHT),
        (PLAYER_RADIUS, feet + WAIST_HEIGHT, feet + PLAYER_HEIGHT),
    ];
    for (min, max, radius) in island.walls_round(pos.xz(), &shape) {
        let (min, max) = (Vec3::new(min.x, 0.0, min.y), Vec3::new(max.x, 0.0, max.y));
        resolve_circle_aabb(pos, radius, min, max);
    }
    pos.x = pos.x.clamp(-OCEAN_LIMIT, OCEAN_LIMIT);
    pos.z = pos.z.clamp(-OCEAN_LIMIT, OCEAN_LIMIT);
}

fn walk_offset(kind: WalkBoneKind, swing: f32, weight: f32) -> Quat {
    match kind {
        WalkBoneKind::ThighL | WalkBoneKind::ThighR => {
            Quat::from_axis_angle(Vec3::Y, swing * WALK_THIGH * weight)
        }
        WalkBoneKind::ShinL => {
            Quat::from_axis_angle(Vec3::Y, (-swing).max(0.0) * WALK_SHIN * weight)
        }
        WalkBoneKind::ShinR => {
            Quat::from_axis_angle(Vec3::Y, swing.max(0.0) * WALK_SHIN * weight)
        }
        WalkBoneKind::ArmL => {
            let hang = Quat::from_axis_angle(Vec3::Z, -WALK_ARM_HANG);
            let stride = Quat::from_axis_angle(Vec3::X, swing * WALK_ARM * weight);
            hang * stride
        }
        WalkBoneKind::ArmR => {
            let hang = Quat::from_axis_angle(Vec3::Z, WALK_ARM_HANG);
            let stride = Quat::from_axis_angle(Vec3::X, -swing * WALK_ARM * weight);
            hang * stride
        }
    }
}

fn walk_bone_kind(name: &str) -> Option<WalkBoneKind> {
    Some(match name {
        // Limb meshes are skinned to the .001 / .004 chains, not Bone.002.
        "Bone.005.L.001" => WalkBoneKind::ThighL,
        "Bone.005.R.001" => WalkBoneKind::ThighR,
        "Bone.006.L" => WalkBoneKind::ShinL,
        "Bone.006.R" => WalkBoneKind::ShinR,
        "Bone.004.L" => WalkBoneKind::ArmL,
        "Bone.004.R" => WalkBoneKind::ArmR,
        _ => return None,
    })
}

fn bind_walk_bones(
    ready: On<WorldInstanceReady>,
    children: Query<&Children>,
    named: Query<(&Name, &Transform)>,
    mut commands: Commands,
) {
    let bound = bind_limbs(ready.entity, &children, &named, &mut commands);
    if bound < 6 {
        warn!("walk bones: bound {bound}/6 on player {:?}", ready.entity);
    }
}

/// Hands the limbs under `owner` to its [`WalkCycle`], which swings them as it
/// walks and, standing, hangs its arms at its sides. Returns how many were
/// found.
pub(crate) fn bind_limbs(
    owner: Entity,
    children: &Query<&Children>,
    named: &Query<(&Name, &Transform)>,
    commands: &mut Commands,
) -> usize {
    let mut bound = 0;
    for entity in children.iter_descendants(owner) {
        let Ok((name, transform)) = named.get(entity) else {
            continue;
        };
        let Some(kind) = walk_bone_kind(name.as_str()) else {
            continue;
        };
        commands.entity(entity).insert(WalkBone {
            kind,
            rest_rotation: transform.rotation,
            owner,
        });
        bound += 1;
    }
    bound
}

fn apply_planar_move(transform: &mut Transform, world_dir: Vec3, distance: f32, dt: f32) {
    let mut dir = world_dir;
    dir.y = 0.0;
    let Some(dir) = dir.try_normalize() else {
        return;
    };
    let target = Transform::from_translation(transform.translation)
        .looking_at(transform.translation + dir, Vec3::Y)
        .rotation;
    let angle = transform.rotation.angle_between(target);
    if angle <= f32::EPSILON {
        transform.rotation = target;
    } else {
        let step = (TURN_SPEED * dt / angle).min(1.0);
        transform.rotation = transform.rotation.slerp(target, step);
    }
    transform.translation += dir * distance;
}

/// This device's stick or keys, as the [`Intent`] of its [`Player`].
fn read_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<TouchControls>,
    dialog: Res<hud::DialogUp>,
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    mut players: Query<(&Transform, &mut Intent, &mut WalkCycle), With<Player>>,
) {
    let Ok((transform, mut intent, mut walk)) = players.single_mut() else {
        return;
    };
    // A dialog is up: you stand and wait for it to be answered.
    if dialog.0 {
        *intent = Intent::default();
        walk.speed = 0.0;
        return;
    }
    let slowed = wading(transform.translation);
    let (cam_forward, cam_right) = cameras
        .single()
        .map(flatten_basis)
        .unwrap_or((Vec3::NEG_Z, Vec3::X));

    let mut wish = Vec3::ZERO;
    let stick = controls.stick_value;
    if stick.length() >= STICK_DEADZONE {
        wish += cam_right * stick.x + cam_forward * stick.y;
    } else {
        if keyboard.pressed(KeyCode::KeyW) {
            wish += cam_forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            wish -= cam_forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            wish -= cam_right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            wish += cam_right;
        }
    }

    let speed_scale = if stick.length() >= STICK_DEADZONE {
        stick.length().clamp(0.0, 1.0)
    } else if wish.length_squared() > 0.0 {
        1.0
    } else {
        0.0
    };
    // Your own legs follow the stick rather than where you end up, so they
    // answer the moment it moves. Everyone else's are read off how their body
    // moves (`lobby::walk_from_motion`).
    walk.speed = if speed_scale > 0.0 {
        if slowed {
            speed_scale * WATER_MOVE_SCALE
        } else {
            speed_scale
        }
    } else {
        0.0
    };
    intent.walk = wish.normalize_or_zero() * speed_scale;
    intent.jump = keyboard.just_pressed(KeyCode::Space) || controls.jump_just_pressed;
}

/// Every body that moves under its own power, and what moving it needs.
type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static Collider,
        &'static Venue,
        &'static Intent,
        &'static mut PlayerJump,
        Has<StaysAshore>,
    ),
>;

/// Walks, jumps and drops every body that moves under its own power — yours
/// and the AIs' alike — as its [`Intent`] asks, and keeps each one out of its
/// island, the trees on it and every other player there. Bodies on different
/// islands pass through each other: they are never in the same place.
pub(crate) fn move_bodies(
    time: Res<Time>,
    islands: Res<Islands>,
    fixed: Query<(&Transform, &Collider, &Venue), Without<Intent>>,
    mut bodies: Bodies,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let fixed: Vec<(Vec3, Collider, Venue)> = fixed
        .iter()
        .map(|(solid, collider, venue)| (solid.translation, *collider, *venue))
        .collect();

    let movers: Vec<(Entity, Venue)> = bodies
        .iter()
        .map(|(entity, _, _, venue, ..)| (entity, *venue))
        .collect();
    for (mover, venue) in movers {
        // Until its island has loaded there is nothing to stand on, so the
        // body waits where it is instead of dropping into the sea.
        let Some(island) = islands.get(venue) else {
            continue;
        };
        // Everyone else here as they stand now, which for those already moved
        // this frame is where they have just got to, so no two end up inside
        // each other.
        let mut solids: Vec<(Vec3, Collider)> = fixed
            .iter()
            .filter(|(.., at)| *at == venue)
            .map(|&(center, collider, _)| (center, collider))
            .collect();
        solids.extend(
            bodies
                .iter()
                .filter(|(other, _, _, at, ..)| *other != mover && **at == venue)
                .map(|(_, other, collider, ..)| (other.translation, *collider)),
        );
        let Ok((_, mut body, _, _, intent, mut jump, stays_ashore)) = bodies.get_mut(mover) else {
            continue;
        };
        // Worked out on a copy and only written back if it has changed, so
        // that someone standing still is not taken for someone who moved,
        // which would have their whole skeleton placed again every frame.
        let mut transform = *body;
        let motion = Motion {
            island,
            solids: &solids,
            stays_ashore,
            dt,
            elapsed,
        };
        motion.step(&mut transform, &mut jump, intent);
        body.set_if_neq(transform);
    }
}

/// What one body's own motion over a frame depends on, besides the body.
struct Motion<'a> {
    island: &'a Island,
    /// Everyone and everything else on the island to bump into.
    solids: &'a [(Vec3, Collider)],
    stays_ashore: bool,
    dt: f32,
    /// The game's time, which the sea bobs by.
    elapsed: f32,
}

impl Motion<'_> {
    /// Walks, jumps and drops `transform`, as `intent` asks, for one frame.
    fn step(&self, transform: &mut Transform, jump: &mut PlayerJump, intent: &Intent) {
        let Motion {
            island,
            solids,
            stays_ashore,
            dt,
            elapsed,
        } = *self;
        let support_y = support(island, transform.translation, elapsed);
        let grounded = transform.translation.y <= support_y + WATER_JUMP_SLOP;
        let slowed = wading(transform.translation);

        let speed_scale = intent.walk.length().min(1.0);
        if speed_scale > 0.0 {
            let speed = if slowed {
                MOVE_SPEED * WATER_MOVE_SCALE
            } else {
                MOVE_SPEED
            };
            // In steps no longer than the lattice the island is tested on, so
            // that one slow frame cannot carry the player clean through a fence.
            let distance = speed * speed_scale * dt;
            let steps = (distance / island::LATTICE).ceil().max(1.0);
            for _ in 0..steps as u32 {
                let from = transform.translation;
                apply_planar_move(transform, intent.walk, distance / steps, dt / steps);
                resolve_player_solids(&mut transform.translation, solids, island);
                if stays_ashore && strands(island, from, transform.translation) {
                    transform.translation = from;
                    break;
                }
            }
        }

        if intent.jump {
            jump.buffer = JUMP_BUFFER;
        }
        if grounded {
            jump.coyote = JUMP_COYOTE;
        } else {
            jump.coyote = (jump.coyote - dt).max(0.0);
        }
        jump.buffer = (jump.buffer - dt).max(0.0);
        if jump.buffer > 0.0 && jump.coyote > 0.0 {
            jump.velocity_y = JUMP_SPEED;
            jump.buffer = 0.0;
            jump.coyote = 0.0;
        }

        let gravity = if jump.velocity_y > 0.0 {
            GRAVITY
        } else {
            FALL_GRAVITY
        };
        jump.velocity_y -= gravity * dt;
        let before = transform.translation.y;
        transform.translation.y += jump.velocity_y * dt;
        // Going up, the head stops at whatever is over it — a ceiling, the top
        // of a doorway — rather than going into it, where it would read as a
        // wall at the height of the head and shove the body out sideways.
        if jump.velocity_y > 0.0
            && let Some(ceiling) = headroom(island, transform.translation.xz(), before + PLAYER_HEIGHT)
            && transform.translation.y + PLAYER_HEIGHT > ceiling - HEAD_CLEARANCE
        {
            transform.translation.y = (ceiling - HEAD_CLEARANCE - PLAYER_HEIGHT).max(before);
            jump.velocity_y = 0.0;
        }

        // Looked for from the higher of this frame's two heights, so that one
        // long frame cannot drop the feet clean through the ground they started
        // on.
        let highest = transform.translation.with_y(before.max(transform.translation.y));
        let support_y = support(island, highest, elapsed);
        // Walking down the bridge, keep the feet on it rather than stepping off
        // into the air every frame and dropping back onto it a moment later.
        let downhill = grounded && transform.translation.y - support_y <= LAND_STEP_UP;
        if jump.velocity_y <= 0.0 && (transform.translation.y <= support_y || downhill) {
            transform.translation.y = support_y;
            jump.velocity_y = 0.0;
        }
        // Only once the feet have settled, so that the ground they are about to
        // land on does not first read as a wall at the height of their shins.
        resolve_player_solids(&mut transform.translation, solids, island);
    }
}

pub(crate) fn animate_walk(
    time: Res<Time>,
    mut walks: Query<&mut WalkCycle>,
    mut bones: Query<(&WalkBone, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for mut walk in &mut walks {
        let target = if walk.speed > 0.04 { 1.0 } else { 0.0 };
        walk.weight = walk.weight.lerp(target, (dt * WALK_BLEND).min(1.0));
        if walk.weight < 0.01 {
            walk.weight = 0.0;
        } else {
            walk.phase += dt * WALK_STRIDE_FREQ * walk.speed.max(0.35);
        }
    }
    for (bone, mut transform) in &mut bones {
        let Ok(walk) = walks.get(bone.owner) else {
            continue;
        };
        let rotation = bone.rest_rotation * walk_offset(bone.kind, walk.phase.sin(), walk.weight);
        // Standing still, a limb keeps the pose it had, and is left alone.
        if transform.rotation != rotation {
            transform.rotation = rotation;
        }
    }
}


/// Puts the camera where you turned it, or as near there as the island
/// allows: never inside a wall or a ceiling, and with one between it and you
/// only for the moment it takes to come in past it. It always points the way
/// you turned it, so that every bit of a swipe turns it by as much wherever
/// you are. Whatever is in the way only brings it nearer, straight in along
/// that line, swept in rather than there at once, and once the way is clear
/// it eases back out. Brought in near you, it rises straight up toward your eyes, and
/// once it is so near that it is inside you, it sees what you would, from as
/// high as in first person, and you are not drawn.
///
/// In [`FirstPerson`] it is your eyes, turned the same way, wider unless it is
/// zoomed in, and with you never drawn.
pub(crate) fn follow_camera(
    time: Res<Time>,
    orbit: Res<OrbitCamera>,
    first_person: Res<FirstPerson>,
    islands: Res<Islands>,
    mut players: Query<(&Transform, &Venue, &mut Visibility), With<Player>>,
    mut cameras: Query<
        (&mut Transform, &mut Projection, &mut CameraFit),
        (With<ThirdPersonCamera>, Without<Player>),
    >,
) {
    let Ok((player, &venue, mut body)) = players.single_mut() else {
        return;
    };
    let Ok((mut eye, mut projection, mut fit)) = cameras.single_mut() else {
        return;
    };

    let look_up = if orbit.pitch < 0.0 {
        (orbit.pitch / PITCH_MIN).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Both only written when they change, so that a camera at rest is not
    // taken for one that moved.
    let fov = if first_person.0 {
        orbit.fov
    } else {
        CAMERA_FOV + (CAMERA_LOOK_UP_FOV - CAMERA_FOV) * look_up
    };
    if let Projection::Perspective(perspective) = &*projection
        && (perspective.fov != fov || perspective.far != CAMERA_FAR)
    {
        *projection = Projection::Perspective(PerspectiveProjection {
            fov,
            far: CAMERA_FAR,
            ..perspective.clone()
        });
    }
    if first_person.0 {
        body.set_if_neq(Visibility::Hidden);
        // Back out of your eyes, it starts again behind you rather than easing
        // out from wherever it last stood.
        fit.set_if_neq(CameraFit::default());
        eye.set_if_neq(eye_view(&orbit, player, islands.get(venue)));
        return;
    }
    let (view, next) = place_camera(
        &orbit,
        player,
        look_up,
        islands.get(venue),
        *fit,
        time.delta_secs(),
    );
    body.set_if_neq(if inside_you(view.translation, player) {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    fit.set_if_neq(next);
    eye.set_if_neq(view);
}

/// Whether the camera at `at` is inside you, or so near your arms or head that
/// you would fill the screen from within: anywhere over where you stand up to
/// your eyes, which are higher than your head.
fn inside_you(at: Vec3, player: &Transform) -> bool {
    let feet = player.translation;
    at.xz().distance(feet.xz()) < PLAYER_RADIUS + CAMERA_INSIDE
        && at.y > feet.y - CAMERA_INSIDE
        && at.y < feet.y + PLAYER_HEIGHT.max(EYE_HEIGHT) + CAMERA_INSIDE
}

/// Your own eyes, looking the way `orbit` is turned. In third person the same
/// yaw and pitch swing the camera round you to look the same way, so a swipe
/// turns the view alike in both. They are higher than your head, so under a
/// ceiling lower than them they stop short of it, the way a jump stops your
/// head.
fn eye_view(orbit: &OrbitCamera, player: &Transform, island: Option<&Island>) -> Transform {
    let chest = player.translation + Vec3::Y * LOOK_HEIGHT;
    Transform::from_translation(rise(chest, EYE_HEIGHT - LOOK_HEIGHT, island))
        .with_rotation(Quat::from_rotation_y(orbit.yaw) * Quat::from_rotation_x(-orbit.pitch))
}

/// Where the camera at `at`, in third person, stands once it has risen toward
/// your eyes: not at all out where it sees you whole, a little more the
/// nearer it comes to you, and as high as [`EYE_HEIGHT`] by the time you are
/// out of sight, so that it then sees what you would in first person. Only
/// ever up, straight up, so that it still looks the way you turned it.
fn toward_your_eyes(at: Vec3, player: &Transform, island: Option<&Island>) -> Vec3 {
    let out = at.xz().distance(player.translation.xz()) - (PLAYER_RADIUS + CAMERA_INSIDE);
    let near = 1.0 - (out / EYES_RISE).clamp(0.0, 1.0);
    let share = near * near * (3.0 - 2.0 * near);
    let short = player.translation.y + EYE_HEIGHT - at.y;
    if share <= 0.0 || short <= 0.0 {
        return at;
    }
    rise(at, short * share, island)
}

/// As far as `up` straight up from `at` as the camera gets before it meets a
/// ceiling, and no further.
fn rise(at: Vec3, up: f32, island: Option<&Island>) -> Vec3 {
    let top = at + Vec3::Y * up;
    let clear = island
        .and_then(|island| island.sweep(at, top, CAMERA_RADIUS))
        .unwrap_or(up);
    at + Vec3::Y * clear
}

/// Where the camera stands this frame, `dt` seconds after it stood as `fit`
/// says, turned as `orbit` is round `player` on `island`, and how much nearer
/// that is than you zoomed it: on the line you turned it along, or, near you,
/// risen straight up from a point of it toward your eyes. Coming in, it can be
/// behind something for a moment, but never in it. Until the island has
/// loaded, nothing is in the way.
fn place_camera(
    orbit: &OrbitCamera,
    player: &Transform,
    look_up: f32,
    island: Option<&Island>,
    fit: CameraFit,
    dt: f32,
) -> (Transform, CameraFit) {
    let (mut view, aim) = camera_view(orbit, player, look_up);
    let (way, full) = (view.translation - aim).normalize_and_length();
    let clear = island
        .filter(|_| full > 1e-3)
        .and_then(|island| island.sweep(aim, view.translation, CAMERA_RADIUS))
        .unwrap_or(full);
    let pivot = player.translation + Vec3::Y * LOOK_HEIGHT;
    let cut = fit.pivot.is_none_or(|last| last.distance(pivot) > CAMERA_CUT);
    let mut distance = fit.distance_toward(clear, full, cut, dt);
    // Still on its way in, it is behind what is in the way. Should that put it
    // in it, or touching it, it goes the rest of the way in at once rather than
    // show it from inside.
    if let (Some(island), Some(behind)) = (island, distance)
        && behind > clear
        && !island.room_for(aim + way * behind, CAMERA_RADIUS)
    {
        distance = Some(clear);
    }
    let next = CameraFit {
        distance,
        pivot: Some(pivot),
    };
    // In along the same line, so that it still looks the same way.
    if let Some(distance) = next.distance {
        view.translation = aim + way * distance;
    }
    view.translation = toward_your_eyes(view.translation, player, island);
    (view, next)
}

/// Where the camera stands and which way it looks, for `orbit` round `player`,
/// and the point on you it looks toward, which nothing may come between.
fn camera_view(orbit: &OrbitCamera, player: &Transform, look_up: f32) -> (Transform, Vec3) {
    let mut camera = Transform::default();
    let yaw = Quat::from_rotation_y(orbit.yaw);
    if orbit.pitch >= 0.0 {
        let look_at = player.translation + Vec3::Y * LOOK_HEIGHT;
        let offset =
            yaw * Quat::from_rotation_x(-orbit.pitch) * Vec3::new(0.0, 0.0, orbit.distance);
        camera.translation = look_at + offset;
        camera.look_at(look_at, Vec3::Y);
        return (camera, look_at);
    }

    let back = yaw * Vec3::Z;
    let cam_height = LOOK_HEIGHT + (CAMERA_LOOK_UP_HEIGHT - LOOK_HEIGHT) * look_up;
    let look_up_distance = orbit.distance * (CAMERA_LOOK_UP_DISTANCE / CAMERA_DISTANCE);
    let distance = orbit.distance + (look_up_distance - orbit.distance) * look_up;
    let focus_height = LOOK_HEIGHT + (LOOK_UP_FOCUS_HEIGHT - LOOK_HEIGHT) * look_up;
    camera.translation = player.translation + back * distance + Vec3::Y * cam_height;
    camera.translation.y = camera.translation.y.max(player.translation.y + CAMERA_LOOK_UP_HEIGHT);

    let focus = player.translation + Vec3::Y * focus_height;
    camera.look_at(focus, Vec3::Y);
    camera.rotate_local_x(LOOK_UP_EXTRA_PITCH * look_up);
    (camera, focus)
}

#[cfg(test)]
mod camera_tests {
    use super::*;

    const DT: f32 = 1.0 / 120.0;

    fn orbit(yaw: f32, pitch: f32, distance: f32) -> OrbitCamera {
        OrbitCamera {
            yaw,
            pitch,
            distance,
            ..default()
        }
    }

    fn look_up(pitch: f32) -> f32 {
        if pitch < 0.0 {
            (pitch / PITCH_MIN).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Closed boxes, from their lowest to their highest corners, wound out.
    pub(super) fn island(boxes: &[(Vec3, Vec3)]) -> Island {
        let mut corners = Vec::new();
        for &(l, h) in boxes {
            let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
            let quads = [
                [p(l.x, l.y, l.z), p(h.x, l.y, l.z), p(h.x, l.y, h.z), p(l.x, l.y, h.z)],
                [p(l.x, h.y, l.z), p(l.x, h.y, h.z), p(h.x, h.y, h.z), p(h.x, h.y, l.z)],
                [p(l.x, l.y, l.z), p(l.x, l.y, h.z), p(l.x, h.y, h.z), p(l.x, h.y, l.z)],
                [p(h.x, l.y, l.z), p(h.x, h.y, l.z), p(h.x, h.y, h.z), p(h.x, l.y, h.z)],
                [p(l.x, l.y, l.z), p(l.x, h.y, l.z), p(h.x, h.y, l.z), p(h.x, l.y, l.z)],
                [p(l.x, l.y, h.z), p(h.x, l.y, h.z), p(h.x, h.y, h.z), p(l.x, h.y, h.z)],
            ];
            for q in quads {
                corners.push([q[0], q[1], q[2]]);
                corners.push([q[0], q[2], q[3]]);
            }
        }
        Island::new_for_tests(&corners)
    }

    const GROUND: (Vec3, Vec3) = (Vec3::new(-40.0, -2.0, -40.0), Vec3::new(40.0, 0.0, 40.0));
    /// A tall wall from x = 6.
    const WALL: (Vec3, Vec3) = (Vec3::new(6.0, -0.1, -20.0), Vec3::new(6.5, 6.0, 20.0));

    /// The room in `testhomemap.glb`: 3 m high inside, its doorway facing +z.
    fn room() -> Vec<(Vec3, Vec3)> {
        let v = Vec3::new;
        vec![
            GROUND,
            (v(-4.0, -0.1, -8.5), v(4.0, 3.3, -8.2)),
            (v(-4.0, -0.1, -8.5), v(-3.7, 3.3, 0.0)),
            (v(3.7, -0.1, -8.5), v(4.0, 3.3, 0.0)),
            (v(-4.0, -0.1, -0.3), v(-0.8, 3.3, 0.0)),
            (v(0.8, -0.1, -0.3), v(4.0, 3.3, 0.0)),
            (v(-0.8, 2.3, -0.3), v(0.8, 3.3, 0.0)),
            (v(-3.7, 3.0, -8.2), v(3.7, 3.3, -0.3)),
        ]
    }

    /// How many frames in a row the camera may be unable to see you while it
    /// comes in past what came between: half a second.
    const CATCH_UP: u32 = 60;

    /// One frame of the camera, checked: it looks exactly the way you turned
    /// it, from along the line you turned it along or straight over a point of
    /// it, risen no higher than your eyes, no further out than you zoomed it,
    /// clear of every box, and able to see you, or at least not unable to for
    /// longer than [`CATCH_UP`]. `hidden` counts the frames in a row it has
    /// not seen you.
    fn check(
        view: &Transform,
        you: &OrbitCamera,
        player: &Transform,
        island: &Island,
        boxes: &[(Vec3, Vec3)],
        hidden: &mut u32,
        when: &str,
    ) {
        let (yours, aim) = camera_view(you, player, look_up(you.pitch));
        assert_eq!(view.rotation, yours.rotation, "{when}: not turned the way you turned it");
        let (way, full) = (yours.translation - aim).normalize_and_length();
        // The point of the line it stands on, or straight over.
        let out = (view.translation - aim).xz().dot(way.xz()) / way.xz().length_squared();
        let under = aim + way * out;
        assert!(out <= full + 1e-4, "{when}: {out} out, further than {full}");
        let off = view.translation.xz().distance(under.xz());
        assert!(off < 1e-3, "{when}: {off} off the line you turned it along");
        let risen = view.translation.y - under.y;
        let eyes = player.translation.y + EYE_HEIGHT;
        assert!(risen > -1e-4, "{when}: {risen} under the line you turned it along");
        assert!(risen < 1e-4 || view.translation.y < eyes + 1e-4, "{when}: over your eyes");
        let seen = island
            .sweep(aim, under, CAMERA_RADIUS)
            .is_none_or(|clear| clear >= out - 0.02)
            && island
                .sweep(under, view.translation, CAMERA_RADIUS)
                .is_none_or(|clear| clear >= risen - 0.02);
        *hidden = if seen { 0 } else { *hidden + 1 };
        assert!(*hidden <= CATCH_UP, "{when}: has not seen you for {hidden} frames, from {}", view.translation);
        for &(low, high) in &boxes[1..] {
            let off = view.translation.distance(view.translation.clamp(low, high));
            assert!(off >= CAMERA_RADIUS - 0.02, "{when}: {off} from a wall at {}", view.translation);
        }
    }

    #[test]
    fn open_ground_leaves_it_alone() {
        let island = island(&[GROUND]);
        let player = Transform::default();
        let you = orbit(0.4, 0.38, 10.0);
        let (view, fit) = place_camera(&you, &player, 0.0, Some(&island), CameraFit::default(), DT);
        assert_eq!(view, camera_view(&you, &player, 0.0).0);
        assert_eq!(fit.distance, None);
    }

    #[test]
    fn turning_it_into_a_wall_only_brings_it_in() {
        // Pressed against the wall, and turned round into it and out the
        // other side, level and from high up.
        let boxes = [GROUND, WALL];
        let island = island(&boxes);
        let player = Transform::from_xyz(5.55, 0.0, 0.0);
        for pitch in [0.0, 0.38, 1.2] {
            let (mut fit, mut hidden) = (CameraFit::default(), 0);
            for frame in 0..480 {
                let you = orbit(frame as f32 * DT, pitch, 10.0);
                let (view, next) = place_camera(&you, &player, 0.0, Some(&island), fit, DT);
                fit = next;
                let when = format!("pitch {pitch}, frame {frame}");
                check(&view, &you, &player, &island, &boxes, &mut hidden, &when);
            }
        }
    }

    #[test]
    fn turning_it_round_a_room_follows_your_hand() {
        // In the middle of the room and tucked into a corner of it: turned all
        // the way round, fast, and tipped up and down as it goes.
        let boxes = room();
        let island = island(&boxes);
        for (x, z) in [(0.0, -4.0), (-3.2, -7.7), (3.2, -0.8)] {
            let player = Transform::from_xyz(x, 0.0, z);
            let (mut fit, mut hidden) = (CameraFit::default(), 0);
            for frame in 0..480 {
                let time = frame as f32 * DT;
                let you = orbit(time * 3.0, 0.6 + 0.6 * (time * 2.0).sin(), 15.0);
                let look = look_up(you.pitch);
                let (view, next) = place_camera(&you, &player, look, Some(&island), fit, DT);
                fit = next;
                let when = format!("at {x} {z}, frame {frame}");
                check(&view, &you, &player, &island, &boxes, &mut hidden, &when);
            }
        }
    }

    #[test]
    fn looking_up_in_a_room_stays_inside_it() {
        let boxes = room();
        let island = island(&boxes);
        let player = Transform::from_xyz(-3.2, 0.0, -7.7);
        let (mut fit, mut hidden) = (CameraFit::default(), 0);
        for frame in 0..480 {
            let time = frame as f32 * DT;
            let you = orbit(time * 2.0, -0.6 - 0.5 * (time * 3.0).sin(), 10.0);
            let (view, next) =
                place_camera(&you, &player, look_up(you.pitch), Some(&island), fit, DT);
            fit = next;
            check(&view, &you, &player, &island, &boxes, &mut hidden, &format!("frame {frame}"));
        }
    }

    #[test]
    fn walking_into_a_room_from_the_sky_view_and_out() {
        let boxes = room();
        let island = island(&boxes);
        let you = orbit(0.0, 1.2, 15.0);
        for x in [0.0, 0.2, -0.5] {
            let (mut fit, mut hidden) = (CameraFit::default(), 0);
            let path = (0..=230).map(|step| 6.0 - step as f32 * 7.0 * DT);
            for (step, z) in path.clone().chain(path.rev()).enumerate() {
                let player = Transform::from_xyz(x, 0.0, z);
                let (view, next) = place_camera(&you, &player, 0.0, Some(&island), fit, DT);
                fit = next;
                let when = format!("x {x}, step {step}");
                check(&view, &you, &player, &island, &boxes, &mut hidden, &when);
                // Seeing you, unless it is still on its way down over the roof.
                if z < -1.0 && hidden == 0 {
                    let at = view.translation;
                    let in_room = at.x.abs() < 3.7 && at.z > -8.2 && at.z < -0.3 && at.y < 3.0;
                    assert!(in_room, "x {x}, z {z}: camera not in the room, at {at}");
                }
            }
            // Back outside, it is as far out as you zoomed it again.
            for _ in 0..240 {
                let player = Transform::from_xyz(x, 0.0, 6.0);
                fit = place_camera(&you, &player, 0.0, Some(&island), fit, DT).1;
            }
            assert_eq!(fit.distance, None, "x {x}");
        }
    }

    #[test]
    fn brought_in_to_you_it_sees_from_your_eyes() {
        // Your back to the wall, and the camera turned round behind you into
        // it: it comes in to you, and up to your eyes, with you out of sight.
        let open = [GROUND, WALL];
        let player = Transform::from_xyz(5.55, 0.0, 0.0);
        let you = orbit(std::f32::consts::FRAC_PI_2, 0.38, 10.0);
        let place = |boxes: &[(Vec3, Vec3)]| {
            let island = island(boxes);
            let (view, _) =
                place_camera(&you, &player, 0.0, Some(&island), CameraFit::default(), DT);
            check(&view, &you, &player, &island, boxes, &mut 0, "at the wall");
            (view.translation, eye_view(&you, &player, Some(&island)).translation)
        };
        let (behind, eyes) = place(&open);
        assert!((behind.y - EYE_HEIGHT).abs() < 1e-3, "{behind}");
        assert!(inside_you(behind, &player), "{behind}");
        assert_eq!(eyes.y, EYE_HEIGHT);
        // Under a ceiling lower than your eyes, as high as it goes under it,
        // in third person and in first.
        let low = (Vec3::new(-20.0, 2.3, -20.0), Vec3::new(6.0, 2.6, 20.0));
        let (behind, eyes) = place(&[GROUND, WALL, low]);
        for at in [behind, eyes] {
            assert!((at.y - (2.3 - CAMERA_RADIUS)).abs() < 1e-3, "{at}");
        }
    }

    #[test]
    fn eases_back_out() {
        let fit = |distance| CameraFit { distance, pivot: None };
        // Clear again: out, but not all at once.
        let out = fit(Some(3.0)).distance_toward(15.0, 15.0, false, 0.1).unwrap();
        assert!(out > 3.0 && out < 15.0, "{out}");
        // Something back in the way: in, but not all at once either.
        let back = fit(Some(out)).distance_toward(3.0, 15.0, false, DT).unwrap();
        assert!(back > 3.0 && back < out, "{back}");
        // A little more room is taken at once, rather than lagged behind, and
        // a little less given up at once.
        assert_eq!(fit(Some(3.0)).distance_toward(3.05, 15.0, false, DT), Some(3.05));
        assert_eq!(fit(Some(3.05)).distance_toward(3.0, 15.0, false, DT), Some(3.0));
        // Given long enough, it is as far out as you zoomed it.
        let mut distance = Some(3.0);
        for _ in 0..120 {
            distance = fit(distance).distance_toward(15.0, 15.0, false, DT);
        }
        assert_eq!(distance, None);
        // And as far in as the way is clear.
        for _ in 0..60 {
            distance = fit(distance).distance_toward(3.0, 15.0, false, DT);
        }
        assert_eq!(distance, Some(3.0));
        // Zoomed in nearer than it stood, it comes in with you.
        assert_eq!(fit(Some(8.0)).distance_toward(5.0, 5.0, false, DT), None);
        // Starting again, it is where it can see you at once.
        assert_eq!(fit(None).distance_toward(3.0, 15.0, true, DT), Some(3.0));
    }

    #[test]
    fn a_wall_coming_between_sweeps_it_in() {
        // Turned round past the end of a wall 2 m off, so that the wall comes
        // between it and you all at once: from behind the wall, it comes in
        // over a moment rather than in one frame, and is never in the wall.
        let end = (Vec3::new(2.0, -0.1, -20.0), Vec3::new(2.5, 6.0, 1.0));
        let boxes = [GROUND, end];
        let island = island(&boxes);
        let player = Transform::default();
        let (mut fit, mut hidden, mut longest) = (CameraFit::default(), 0, 0);
        let (mut was, mut was_clear) = (10.0, 10.0);
        let (mut biggest, mut biggest_clear) = (0.0f32, 0.0f32);
        for frame in 0..180 {
            let you = orbit(frame as f32 * DT, 0.38, 10.0);
            let (view, next) = place_camera(&you, &player, 0.0, Some(&island), fit, DT);
            fit = next;
            check(&view, &you, &player, &island, &boxes, &mut hidden, &format!("frame {frame}"));
            longest = longest.max(hidden);
            let (yours, aim) = camera_view(&you, &player, 0.0);
            let clear = island.sweep(aim, yours.translation, CAMERA_RADIUS).unwrap_or(10.0);
            let shown = fit.distance.unwrap_or(10.0);
            biggest = biggest.max(was - shown);
            biggest_clear = biggest_clear.max(was_clear - clear);
            (was, was_clear) = (shown, clear);
        }
        // The way in was cut short by metres in one frame.
        assert!(biggest_clear > 5.0, "{biggest_clear}");
        // It was behind the wall for a moment, and came in a little at a time.
        assert!(longest > 5, "{longest}");
        assert!(biggest < biggest_clear * 0.3, "{biggest} in one frame");
        // And it ends up in front of the wall, seeing you.
        assert_eq!(hidden, 0);
    }

    #[test]
    fn zooming_in_first_person_narrows_the_view() {
        let mut you = OrbitCamera::default();
        // Fingers twice as far apart: the world twice the size on the screen.
        you.zoom_by_ratio(0.5, true);
        let half = (FIRST_PERSON_FOV * 0.5).tan() * 0.5;
        assert!(((you.fov * 0.5).tan() - half).abs() < 1e-5, "{}", you.fov);
        // And back again.
        you.zoom_by_ratio(2.0, true);
        assert!((you.fov - FIRST_PERSON_FOV).abs() < 1e-5, "{}", you.fov);
        // Third person keeps its own zoom.
        assert_eq!(you.distance, CAMERA_DISTANCE);
        // No further than it goes, either way.
        for _ in 0..50 {
            you.zoom_by_ratio(0.8, true);
        }
        assert_eq!(you.fov, FIRST_PERSON_FOV_MIN);
        for _ in 0..50 {
            you.zoom_by_ratio(1.25, true);
        }
        assert_eq!(you.fov, FIRST_PERSON_FOV_MAX);
    }

    #[test]
    fn zoomed_in_a_swipe_turns_less() {
        let swipe = Vec2::new(40.0, -25.0);
        let mut unzoomed = OrbitCamera::default();
        unzoomed.turn(swipe, true);
        let mut zoomed = OrbitCamera::default();
        zoomed.zoom_by_ratio(0.5, true);
        zoomed.turn(swipe, true);
        // The world twice the size, half the turn: it moves as far on the
        // screen either way.
        let start = OrbitCamera::default();
        let turned = |you: &OrbitCamera| Vec2::new(you.yaw - start.yaw, you.pitch - start.pitch);
        assert!((turned(&zoomed) * 2.0 - turned(&unzoomed)).length() < 1e-5);
        // In third person, zooming never changes how far a swipe turns it.
        let mut behind = OrbitCamera::default();
        behind.zoom_by_ratio(0.5, false);
        behind.turn(swipe, false);
        assert_eq!(turned(&behind), turned(&unzoomed));
    }
}

#[cfg(test)]
mod ground_tests {
    use super::camera_tests::island;
    use super::*;

    /// The town's fountain, across one side of it, as `circlemap1.glb` has
    /// it: the floor of its basin 0.05 m up, under the water; the rim 0.55;
    /// the ledge outside it 0.3; and the grass.
    pub(super) fn fountain() -> Island {
        let v = Vec3::new;
        island(&[
            (v(-3.0, -1.0, -3.0), v(1.9, 0.05, 3.0)),
            (v(1.9, -1.0, -3.0), v(2.2, 0.55, 3.0)),
            (v(2.2, -1.0, -3.0), v(2.5, 0.3, 3.0)),
            (v(2.5, -1.0, -3.0), v(8.0, 0.0, 3.0)),
        ])
    }

    #[test]
    fn the_fountain_rim_is_a_jump_out_of_the_basin() {
        let fountain = fountain();
        let rim = Vec2::new(2.05, 0.0);
        let walled = |feet: f32| {
            fountain
                .walls(rim, PLAYER_RADIUS, feet + LAND_STEP_UP, feet + PLAYER_HEIGHT)
                .next()
                .is_some()
        };
        // Half a metre up out of the basin: in the way, and not underfoot.
        assert!(walled(0.05));
        assert!(support(&fountain, rim.extend(0.05).xzy(), 0.0) < 0.55);
        // A quarter of a metre up from the ledge outside: a step.
        assert!(!walled(0.3));
        assert_eq!(support(&fountain, rim.extend(0.3).xzy(), 0.0), 0.55);
    }

    #[test]
    fn an_ai_never_steps_down_into_the_basin() {
        let fountain = fountain();
        let on_rim = Vec3::new(2.05, 0.55, 0.0);
        // Down into the basin, which it could not step back up out of.
        assert!(strands(&fountain, on_rim, Vec3::new(1.5, 0.55, 0.0)));
        // Down onto the ledge, and off that onto the grass, which it could.
        assert!(!strands(&fountain, on_rim, Vec3::new(2.35, 0.55, 0.0)));
        assert!(!strands(
            &fountain,
            Vec3::new(2.35, 0.3, 0.0),
            Vec3::new(3.0, 0.3, 0.0)
        ));
        // Nor off the edge of the land into the sea.
        assert!(strands(
            &fountain,
            Vec3::new(7.5, 0.0, 0.0),
            Vec3::new(9.0, 0.0, 0.0)
        ));
    }
}

#[cfg(test)]
mod body_tests {
    use super::camera_tests::island;
    use super::*;

    const DT: f32 = 1.0 / 120.0;

    /// A body on `island` for `frames` frames, from `at`, walking `walk` and
    /// jumping on frame `jump_on`: where it is after each.
    fn frames(island: &Island, at: Vec3, walk: Vec3, jump_on: Option<usize>, frames: usize) -> Vec<Vec3> {
        let mut transform = Transform::from_translation(at);
        let mut jump = PlayerJump::default();
        (0..frames)
            .map(|frame| {
                let motion = Motion {
                    island,
                    solids: &[],
                    stays_ashore: false,
                    dt: DT,
                    elapsed: frame as f32 * DT,
                };
                let intent = Intent {
                    walk,
                    jump: jump_on == Some(frame),
                };
                motion.step(&mut transform, &mut jump, &intent);
                transform.translation
            })
            .collect()
    }

    /// The furthest a body went sideways in one frame.
    fn biggest_shove(path: &[Vec3]) -> f32 {
        path.windows(2)
            .map(|pair| pair[0].xz().distance(pair[1].xz()))
            .fold(0.0, f32::max)
    }

    const GROUND: (Vec3, Vec3) = (Vec3::new(-20.0, -2.0, -20.0), Vec3::new(20.0, 0.0, 20.0));
    /// A block a metre high, its near edge at x = 1.
    const BLOCK: (Vec3, Vec3) = (Vec3::new(1.0, -0.1, -3.0), Vec3::new(3.0, 1.0, 3.0));

    #[test]
    fn coming_down_with_the_legs_on_an_edge_stands_on_it() {
        let island = island(&[GROUND, BLOCK]);
        // The middle of the body 0.15 m short of the edge, the legs over it.
        let path = frames(&island, Vec3::new(0.85, 2.0, 0.0), Vec3::ZERO, None, 120);
        let end = *path.last().unwrap();
        assert!((end.y - 1.0).abs() < 1e-4, "came down at {end}, not on the block");
        assert!(biggest_shove(&path) < 1e-4, "shoved {} sideways", biggest_shove(&path));
    }

    #[test]
    fn jumping_onto_a_ledge_from_beside_it_lands_on_it() {
        // A long one, so that a second of running stays on it.
        let island = island(&[GROUND, (BLOCK.0, BLOCK.1.with_x(15.0))]);
        // Running at the block and jumping just short of it.
        let path = frames(&island, Vec3::new(0.2, 0.0, 0.0), Vec3::X, Some(1), 120);
        let end = *path.last().unwrap();
        assert!((end.y - 1.0).abs() < 1e-4, "ended at {end}, not on the block");
        // Never pushed back the way it came.
        for pair in path.windows(2) {
            assert!(pair[1].x >= pair[0].x - 1e-4, "pushed back from {} to {}", pair[0], pair[1]);
        }
    }

    #[test]
    fn standing_on_a_slope_is_not_standing_on_an_edge() {
        // The bridge's slope, 15°, the steepest ground there is: the feet are
        // on the ground under the middle of them, not on the ground uphill.
        let rise = 15f32.to_radians().tan();
        let corner = |x: f32, z: f32| Vec3::new(x, x * rise, z);
        let ramp = Island::new_for_tests(&[
            [corner(-5.0, -5.0), corner(-5.0, 5.0), corner(5.0, 5.0)],
            [corner(-5.0, -5.0), corner(5.0, 5.0), corner(5.0, -5.0)],
        ]);
        let feet = support(&ramp, Vec3::new(1.0, rise, 0.0), 0.0);
        assert!((feet - rise).abs() < 1e-4, "{feet} on the slope at {rise}");
    }

    /// The fountain across its middle, as boxes: the floor of the basin, the
    /// stem, the middle bowl to 0.736 m out and 0.94 m up, and over it the top
    /// bowl, to 0.491 m out, 1.15 to 1.45 m up.
    fn fountain() -> Island {
        let v = Vec3::new;
        island(&[
            (v(-3.0, -1.0, -3.0), v(3.0, 0.05, 3.0)),
            (v(-0.1, 0.05, -3.0), v(0.1, 1.8, 3.0)),
            (v(-0.736, 0.83, -3.0), v(0.736, 0.94, 3.0)),
            (v(-0.491, 1.15, -3.0), v(0.491, 1.45, 3.0)),
        ])
    }

    #[test]
    fn the_middle_bowl_of_the_fountain_can_be_stood_on() {
        let fountain = fountain();
        // Down onto it from a jump, with the legs over its brim and clear of
        // the top bowl over it.
        for x in [0.72, 0.8, 0.9] {
            let path = frames(&fountain, Vec3::new(x, 2.5, 0.0), Vec3::ZERO, None, 180);
            let end = *path.last().unwrap();
            assert!((end.y - 0.94).abs() < 1e-4, "from {x}: ended at {end}, not on the middle bowl");
            assert!(biggest_shove(&path) < 0.12, "from {x}: shoved {}", biggest_shove(&path));
        }
    }
}
