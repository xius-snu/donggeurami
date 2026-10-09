//! Round Town and your home, two islands played from behind you, or through
//! your own eyes, with WASD or touch.
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
mod see_through;
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
/// The nearest the camera stands behind you, in metres. Zoomed in nearer, it
/// goes into your eyes ([`SWAP_ZOOM`]).
const CAMERA_DISTANCE_MIN: f32 = 2.5;
/// The furthest it stands, in metres: 15 until Hajun asked for a little more
/// (2026-10-09).
const CAMERA_DISTANCE_MAX: f32 = 18.0;
/// How much further a zoom has to go than the camera can, as a ratio, before
/// it goes on into your eyes from as near as it stands behind you, or back
/// out behind you from your eyes: a tenth, a notch of the wheel, so that
/// fingers that waver as a pinch ends do not swap it back and forth.
const SWAP_ZOOM: f32 = 1.1;
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
/// As far down as the camera tips: straight down, for a view of the tops of
/// things (Hajun, 2026-10-09). It stopped at 1.2, about 69°, until then.
const PITCH_MAX: f32 = std::f32::consts::FRAC_PI_2;
const CAMERA_LOOK_UP_DISTANCE: f32 = 2.72;
const CAMERA_LOOK_UP_HEIGHT: f32 = 0.58;
const LOOK_UP_FOCUS_HEIGHT: f32 = PLAYER_HEIGHT * 0.9;
const LOOK_UP_EXTRA_PITCH: f32 = 0.52;
/// How much the camera takes in, top to bottom: about 66°, which is 98°
/// across a 16:9 screen and 110° across a 20:9 phone. Wide, so that while you
/// build, a piece and the room it goes in are on the screen together. It was
/// first person's own until that became the only view (Hajun, 2026-10-09);
/// third person took in 45°.
const CAMERA_FOV: f32 = 1.15;
/// How high over your feet your eyes are, which the camera is in when it is
/// zoomed all the way in: inside your head, as high as the model's eyes, 1.54
/// to 1.6 m up, under the top of the head at 1.71 m (Hajun, 2026-10-09). It
/// was 2.2 m from 2026-10-03, over your head, for the view of a taller
/// person, and 1.44 m before that.
pub(crate) const EYE_HEIGHT: f32 = 1.57;
/// How far your eyes keep from a ceiling lower than them, in metres: further
/// than the corners of the camera's near plane reach from it, 0.18 m at the
/// widest, looking up on a 20:9 phone, so that it is never cut open on screen.
const CAMERA_RADIUS: f32 = 0.2;
/// How far past what you bump into with the camera still counts as inside
/// you, in metres: your arms and the top of your head reach that far.
const CAMERA_INSIDE: f32 = 0.15;
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

/// How you have turned and zoomed the camera: the one view there is, behind
/// you, or zoomed all the way in, through your own eyes (Hajun, 2026-10-09,
/// in place of a button that swapped between the two).
#[derive(Resource)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    /// How far out the camera stands behind you, from
    /// [`CAMERA_DISTANCE_MIN`] to [`CAMERA_DISTANCE_MAX`]; or 0, in your eyes.
    distance: f32,
    /// How much further the zoom has gone, as a ratio, than the camera could
    /// go with it: in, from as near as it stands behind you, or out, from
    /// your eyes. At [`SWAP_ZOOM`] either way it goes into your eyes, or back
    /// out of them.
    past: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.38,
            distance: CAMERA_DISTANCE,
            past: 1.0,
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

    /// Zooms in, for a `ratio` under 1, or out: the camera stands that much
    /// nearer you or further off. Zoomed in on past as near as it stands, it
    /// goes into your eyes, and that is as far in as it zooms; zoomed out
    /// from there, it is straight back behind you, as near as it stands
    /// (Hajun, 2026-10-09).
    fn zoom_by_ratio(&mut self, ratio: f32) {
        if self.distance <= 0.0 {
            self.past = (self.past * ratio).max(1.0);
            if self.past >= SWAP_ZOOM {
                self.distance = CAMERA_DISTANCE_MIN;
                self.past = 1.0;
            }
            return;
        }
        let wanted = self.distance * ratio;
        if wanted >= CAMERA_DISTANCE_MIN {
            self.distance = wanted.min(CAMERA_DISTANCE_MAX);
            self.past = 1.0;
            return;
        }
        self.past *= wanted / CAMERA_DISTANCE_MIN;
        if self.past <= 1.0 / SWAP_ZOOM {
            self.distance = 0.0;
            self.past = 1.0;
        } else {
            self.distance = CAMERA_DISTANCE_MIN;
        }
    }

    /// Turns the camera by a swipe or a move of the mouse of `delta` pixels.
    fn turn(&mut self, delta: Vec2) {
        self.yaw -= delta.x * LOOK_SENSITIVITY;
        self.pitch = (self.pitch + delta.y * LOOK_SENSITIVITY).clamp(PITCH_MIN, PITCH_MAX);
    }
}

/// Which way along the ground `camera` faces: the way it looks, or, looking
/// straight down, the way the top of the screen is. What the stick's up
/// walks you toward, and where a piece from the shop comes up.
pub(crate) fn camera_ahead(camera: &Transform) -> Vec3 {
    let looking = camera.forward().with_y(0.0);
    if looking.length_squared() > 1e-4 {
        looking.normalize()
    } else {
        camera.up().with_y(0.0).normalize_or(Vec3::NEG_Z)
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
    // Whatever stands between the camera and you, see-through, and out of a
    // tap's reach.
    see_through::plugin(&mut app);

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
    // Before `app.run()` moves the render world onto its own thread. Not on
    // an iPhone, nor a Mac: there one of the render world's systems has to run
    // on the main thread, the one that makes the window's surface, which
    // touches its UIView. Bevy's own executor hands such a system back to the
    // main thread, and one thread cannot: it ran on the render thread, which
    // panicked on the first frame ("can only access UIView on the main
    // thread"), Bevy asked to quit, iOS does not let an app, and 1.0.1 to
    // 1.0.3 opened on a black screen and stayed there (2026-10-08, read off
    // the phone's own log.txt).
    #[cfg(not(target_vendor = "apple"))]
    {
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            each_schedule(render.world_mut());
        }
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
        // No pinch or rotation recognizers on iOS. A UIKit gesture recognizer
        // cancels the touches it recognizes its gesture in (winit 0.30 leaves
        // `cancelsTouchesInView` as UIKit has it, on), and two thumbs, one on
        // the stick and one on the jump button or turning the camera, read as
        // a pinch or a turn once either moves: the stick goes dead, mid-jump
        // as like as not, until it is lifted and put down again. That is the
        // likeliest reason a running jump onto the fountain fell short on the
        // iPhone (Hajun, iOS 1.0.2 build 15), read off winit's source, not
        // yet seen on the phone. A pinch zooms through `read_touch_controls`,
        // as it always has on Android.
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
/// passes every frame, for nothing; on the CPU it is nothing at all, and every
/// platform takes the one path Android always has. It was turned off on
/// 2026-10-08 as the suspect for the iPhone's black screen, being the one
/// path Bevy 0.19 takes on an iPhone but neither on Android nor in the iOS
/// simulator; it was not that (`one_thread_per_world` was), and it stays off
/// for what it saves.
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
            orbit.zoom_by_ratio(ratio);
        }
        controls.pinch_last_dist = dist;
        controls.look_last = a;
    } else {
        controls.pinch_last_dist = 0.0;
        if let Some(pos) = look_pos {
            let delta = pos - controls.look_last;
            controls.look_last = pos;
            orbit.turn(delta);
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
        orbit.turn(event.delta);
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_mouse_zoom(
    dialog: Res<hud::DialogUp>,
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
        orbit.zoom_by_ratio((1.0 - lines * ZOOM_WHEEL).clamp(0.7, 1.4));
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
    let cam_forward = cameras.single().map_or(Vec3::NEG_Z, camera_ahead);
    let cam_right = cam_forward.cross(Vec3::Y);

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


/// Puts the camera where you turned and zoomed it, whatever is in the way:
/// nothing ever brings it in nearer you. Whatever stands between it and you
/// is drawn see-through instead (`see_through`). Behind you it circles you,
/// looking down at you, as far as straight down, or looking up past you from
/// low behind you. Zoomed all the way in it is your eyes, turned the same
/// way, and you are not drawn.
pub(crate) fn follow_camera(
    orbit: Res<OrbitCamera>,
    islands: Res<Islands>,
    mut players: Query<(&Transform, &Venue, &mut Visibility), With<Player>>,
    mut cameras: Query<&mut Transform, (With<ThirdPersonCamera>, Without<Player>)>,
) {
    let Ok((player, &venue, mut body)) = players.single_mut() else {
        return;
    };
    let Ok(mut eye) = cameras.single_mut() else {
        return;
    };
    let view = camera_view(&orbit, player, islands.get(venue));
    // Both only written when they change, so that a camera at rest is not
    // taken for one that moved.
    body.set_if_neq(if inside_you(view.translation, player) {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    eye.set_if_neq(view);
}

/// Whether the camera at `at` is inside you, or so near your arms or head that
/// you would fill the screen from within: anywhere over where you stand up to
/// the top of your head.
fn inside_you(at: Vec3, player: &Transform) -> bool {
    let feet = player.translation;
    at.xz().distance(feet.xz()) < PLAYER_RADIUS + CAMERA_INSIDE
        && at.y > feet.y - CAMERA_INSIDE
        && at.y < feet.y + PLAYER_HEIGHT.max(EYE_HEIGHT) + CAMERA_INSIDE
}

/// Where the camera stands and which way it looks, turned and zoomed as
/// `orbit` is round `player` on `island`: as far out behind you as you zoomed
/// it, or in your eyes.
fn camera_view(orbit: &OrbitCamera, player: &Transform, island: Option<&Island>) -> Transform {
    if orbit.distance <= 0.0 {
        return eye_view(orbit, player, island);
    }
    let look_up = if orbit.pitch < 0.0 {
        (orbit.pitch / PITCH_MIN).clamp(0.0, 1.0)
    } else {
        0.0
    };
    behind_you(orbit, player, look_up, orbit.distance)
}

/// Your own eyes, inside your head, looking the way `orbit` is turned: the
/// same yaw and pitch swing the camera round you to look the same way when it
/// is behind you, so a swipe turns the view alike in both. Under a ceiling
/// lower than them they stop short of it, the way a jump stops your head.
fn eye_view(orbit: &OrbitCamera, player: &Transform, island: Option<&Island>) -> Transform {
    let chest = player.translation + Vec3::Y * LOOK_HEIGHT;
    Transform::from_translation(rise(chest, EYE_HEIGHT - LOOK_HEIGHT, island))
        .with_rotation(Quat::from_rotation_y(orbit.yaw) * Quat::from_rotation_x(-orbit.pitch))
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

/// Where the camera stands `distance` out from you, turned as `orbit` is, and
/// which way it looks: down at the point on you it circles, straight down
/// from right over it at the most, or, looking up, from low behind you and
/// nearer, up past you.
fn behind_you(orbit: &OrbitCamera, player: &Transform, look_up: f32, distance: f32) -> Transform {
    let mut camera = Transform::default();
    let yaw = Quat::from_rotation_y(orbit.yaw);
    if orbit.pitch >= 0.0 {
        // Turned as your eyes would be, which is looking at that point from
        // out here, and still says which way is ahead looking straight down.
        let turn = yaw * Quat::from_rotation_x(-orbit.pitch);
        let look_at = player.translation + Vec3::Y * LOOK_HEIGHT;
        camera.translation = look_at + turn * Vec3::new(0.0, 0.0, distance);
        camera.rotation = turn;
        return camera;
    }

    let back = yaw * Vec3::Z;
    let cam_height = LOOK_HEIGHT + (CAMERA_LOOK_UP_HEIGHT - LOOK_HEIGHT) * look_up;
    let look_up_distance = distance * (CAMERA_LOOK_UP_DISTANCE / CAMERA_DISTANCE);
    let distance = distance + (look_up_distance - distance) * look_up;
    let focus_height = LOOK_HEIGHT + (LOOK_UP_FOCUS_HEIGHT - LOOK_HEIGHT) * look_up;
    camera.translation = player.translation + back * distance + Vec3::Y * cam_height;
    camera.translation.y = camera.translation.y.max(player.translation.y + CAMERA_LOOK_UP_HEIGHT);

    let focus = player.translation + Vec3::Y * focus_height;
    camera.look_at(focus, Vec3::Y);
    camera.rotate_local_x(LOOK_UP_EXTRA_PITCH * look_up);
    camera
}

#[cfg(test)]
mod camera_tests {
    use super::*;

    fn orbit(yaw: f32, pitch: f32, distance: f32) -> OrbitCamera {
        OrbitCamera {
            yaw,
            pitch,
            distance,
            ..default()
        }
    }

    /// The triangles of closed boxes, from their lowest to their highest
    /// corners, wound out.
    pub(crate) fn boxes(list: &[(Vec3, Vec3)]) -> Vec<[Vec3; 3]> {
        let mut corners = Vec::new();
        for &(l, h) in list {
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
        corners
    }

    /// Closed boxes, from their lowest to their highest corners, wound out.
    pub(super) fn island(list: &[(Vec3, Vec3)]) -> Island {
        Island::new_for_tests(&boxes(list))
    }

    const GROUND: (Vec3, Vec3) = (Vec3::new(-40.0, -2.0, -40.0), Vec3::new(40.0, 0.0, 40.0));

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

    #[test]
    fn walls_never_bring_it_in() {
        // In the middle of the room, tucked into a corner of it, and outside
        // with the room between the camera and you: turned all the way round
        // and tipped up and down, it stands as far out as you zoomed it,
        // room or no room, and you are in sight of it.
        let island = island(&room());
        for (x, z) in [(0.0, -4.0), (-3.2, -7.7), (0.0, 3.0)] {
            let player = Transform::from_xyz(x, 0.0, z);
            for step in 0..96 {
                let turn = step as f32 / 96.0;
                let tilt = (1.2 * (turn * 9.0).sin()).clamp(PITCH_MIN, PITCH_MAX);
                let you = orbit(turn * std::f32::consts::TAU * 2.0, tilt, 15.0);
                let view = camera_view(&you, &player, Some(&island));
                assert_eq!(view, camera_view(&you, &player, None), "at {x} {z}");
                if you.pitch >= 0.0 {
                    let middle = player.translation + Vec3::Y * LOOK_HEIGHT;
                    let out = view.translation.distance(middle);
                    assert!((out - 15.0).abs() < 1e-3, "at {x} {z}: {out} out");
                }
                assert!(!inside_you(view.translation, &player), "at {x} {z}");
            }
        }
    }

    #[test]
    fn behind_you_you_are_always_in_sight() {
        // As near as it stands, looking up past you and down at you, as far
        // as straight down.
        let player = Transform::default();
        for pitch in [PITCH_MIN, -0.6, 0.0, 0.38, 1.2, PITCH_MAX] {
            let you = orbit(0.3, pitch, CAMERA_DISTANCE_MIN);
            let view = camera_view(&you, &player, None);
            assert!(!inside_you(view.translation, &player), "pitch {pitch}");
        }
    }

    #[test]
    fn zoomed_all_the_way_in_it_is_your_eyes() {
        let player = Transform::from_xyz(1.0, 0.5, -2.0);
        for pitch in [PITCH_MIN, 0.0, 0.38, PITCH_MAX] {
            let you = orbit(0.7, pitch, 0.0);
            let view = camera_view(&you, &player, None);
            assert_eq!(view, eye_view(&you, &player, None));
            // Inside your head, at the height of your eyes, and you not drawn.
            let eyes = view.translation - player.translation;
            assert_eq!(eyes.xz(), Vec2::ZERO);
            assert!((eyes.y - EYE_HEIGHT).abs() < 1e-5, "{eyes}");
            assert!(eyes.y > 1.5 && eyes.y < 1.71, "{eyes}");
            assert!(inside_you(view.translation, &player));
        }
    }

    #[test]
    fn it_looks_straight_down_from_right_overhead() {
        // Tipped down as far as it goes.
        let mut you = orbit(0.9, 0.38, 12.0);
        you.turn(Vec2::new(0.0, 1e4));
        assert_eq!(you.pitch, std::f32::consts::FRAC_PI_2);
        let player = Transform::from_xyz(3.0, 0.0, -1.0);
        let view = camera_view(&you, &player, None);
        let middle = player.translation + Vec3::Y * LOOK_HEIGHT;
        assert!(view.translation.xz().distance(middle.xz()) < 1e-4, "{}", view.translation);
        assert!((view.translation.y - (middle.y + 12.0)).abs() < 1e-4);
        assert!(view.forward().dot(Vec3::NEG_Y) > 1.0 - 1e-6);
        // Ahead is still the way it was turned: the top of the screen, which
        // is where the stick's up walks you.
        let turned = Quat::from_rotation_y(0.9) * Vec3::NEG_Z;
        assert!(camera_ahead(&view).distance(turned) < 1e-4);
        let nearly = camera_view(&orbit(0.9, PITCH_MAX - 0.05, 12.0), &player, None);
        assert!(camera_ahead(&nearly).distance(turned) < 1e-4);
        // And so it is in your eyes, looking at your feet.
        let feet = camera_view(&OrbitCamera { distance: 0.0, ..you }, &player, None);
        assert!(camera_ahead(&feet).distance(turned) < 1e-4);
    }

    #[test]
    fn your_eyes_stop_under_a_low_ceiling() {
        let player = Transform::default();
        let you = orbit(0.0, 0.38, 0.0);
        let eyes =
            |boxes: &[(Vec3, Vec3)]| eye_view(&you, &player, Some(&island(boxes))).translation;
        assert!((eyes(&[GROUND]).y - EYE_HEIGHT).abs() < 1e-6);
        // Under a ceiling lower than your eyes, as high as they go under it.
        let low = (Vec3::new(-20.0, 1.7, -20.0), Vec3::new(20.0, 2.0, 20.0));
        let under = eyes(&[GROUND, low]);
        assert!((under.y - (1.7 - CAMERA_RADIUS)).abs() < 1e-3, "{under}");
    }

    #[test]
    fn zoomed_in_past_as_near_as_it_stands_it_is_in_your_eyes() {
        let mut you = OrbitCamera::default();
        // In, as near as it stands behind you and never nearer, and then a
        // little further in, into your eyes...
        while you.distance > 0.0 {
            you.zoom_by_ratio(0.9);
            assert!(you.distance == 0.0 || you.distance >= CAMERA_DISTANCE_MIN, "{}", you.distance);
        }
        // ...which is as far in as it goes.
        for _ in 0..20 {
            you.zoom_by_ratio(0.8);
        }
        assert_eq!(you.distance, 0.0);
        // A pinch out that wavers back stays in your eyes; out a tenth
        // further than that, it is straight back behind you, as near as it
        // stands.
        you.zoom_by_ratio(1.05);
        you.zoom_by_ratio(0.95);
        you.zoom_by_ratio(1.05);
        assert_eq!(you.distance, 0.0);
        you.zoom_by_ratio(1.05);
        assert_eq!(you.distance, CAMERA_DISTANCE_MIN);
        // A notch of the wheel each way, from as near as it stands.
        you.zoom_by_ratio(0.88);
        assert_eq!(you.distance, 0.0);
        you.zoom_by_ratio(1.12);
        assert_eq!(you.distance, CAMERA_DISTANCE_MIN);
        // Out as far as it goes, and no further.
        for _ in 0..30 {
            you.zoom_by_ratio(1.25);
        }
        assert_eq!(you.distance, CAMERA_DISTANCE_MAX);
    }

    #[test]
    fn a_swipe_turns_it_alike_however_far_out() {
        let swipe = Vec2::new(40.0, -25.0);
        let turned = |distance: f32| {
            let mut you = orbit(0.0, 0.38, distance);
            you.turn(swipe);
            Vec2::new(you.yaw, you.pitch)
        };
        assert_eq!(turned(0.0), turned(CAMERA_DISTANCE));
        assert_eq!(turned(CAMERA_DISTANCE_MIN), turned(CAMERA_DISTANCE_MAX));
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
