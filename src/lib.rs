//! Third-person 3D world: an island, an ocean, a player, and WASD / touch movement.
//!
//! `#[bevy_main]` generates the `android_main` entry point. Desktop and iOS
//! both reach [`main`] through `src/main.rs`; on iOS that binary *is* the app
//! executable, which `mobile/ios/build_rust.sh` drops into the .app bundle.

use bevy::gltf::GltfPlugin;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::view::NoIndirectDrawing;
use bevy::window::WindowResolution;
use bevy::world_serialization::WorldInstanceReady;

mod day_night;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
#[cfg(any(target_os = "android", target_os = "ios"))]
use bevy::window::{MonitorSelection, WindowMode};

const MOVE_SPEED: f32 = 7.0;
const TURN_SPEED: f32 = 22.0;
const JUMP_SPEED: f32 = 8.5;
const GRAVITY: f32 = 22.0;
const PLAYER_HEIGHT: f32 = 1.6;
const PLAYER_RADIUS: f32 = 0.45;
const TREE_TRUNK_RADIUS: f32 = 1.1;
const TREE_TRUNK_HEIGHT: f32 = 6.5;
const WATER_JUMP_SLOP: f32 = 0.22;
const JUMP_COYOTE: f32 = 0.14;
const JUMP_BUFFER: f32 = 0.12;
const LOOK_HEIGHT: f32 = PLAYER_HEIGHT * 0.75;
const LAND_SIZE: f32 = 80.0;
const LAND_HEIGHT: f32 = 2.0;
const LAND_TOP: f32 = 0.0;
const LAND_HALF: f32 = LAND_SIZE * 0.5;
const WATER_Y: f32 = -1.0;
const WATER_WADE: f32 = 0.58;
const WATER_BOB_AMP: f32 = 0.07;
const WATER_BOB_SPEED: f32 = 1.7;
const WATER_MOVE_SCALE: f32 = 0.72;
const LAND_STEP_UP: f32 = 0.75;
const ISLAND_INLAND: f32 = 2.0;
const PLAYER_SPAWN: Vec3 = Vec3::new(0.0, LAND_TOP, 0.0);
const OCEAN_SIZE: f32 = 1200.0;
const OCEAN_LIMIT: f32 = 580.0;
const CAMERA_DISTANCE: f32 = 12.5;
const CAMERA_DISTANCE_MIN: f32 = 5.0;
const CAMERA_DISTANCE_MAX: f32 = 30.0;
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
const CAMERA_LOOK_UP_DISTANCE: f32 = 3.4;
const CAMERA_LOOK_UP_HEIGHT: f32 = 0.58;
const LOOK_UP_FOCUS_HEIGHT: f32 = PLAYER_HEIGHT * 0.9;
const LOOK_UP_EXTRA_PITCH: f32 = 0.52;
const CAMERA_FOV: f32 = std::f32::consts::FRAC_PI_4;
const CAMERA_LOOK_UP_FOV: f32 = 1.1;
const WALK_STRIDE_FREQ: f32 = 7.6;
const WALK_THIGH: f32 = 0.42;
const WALK_SHIN: f32 = 0.28;
const WALK_ARM: f32 = 0.55;
const WALK_ARM_HANG: f32 = 1.02;
const WALK_BLEND: f32 = 9.0;

#[derive(Component)]
struct Ocean;

#[derive(Resource)]
struct OceanMats {
    surface: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Player;

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
struct Collider {
    shape: ColliderShape,
}

#[derive(Clone, Copy)]
enum ColliderShape {
    Cylinder { radius: f32, height: f32 },
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
}

#[derive(Resource)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.38,
            distance: CAMERA_DISTANCE,
        }
    }
}

impl OrbitCamera {
    fn zoom_by_ratio(&mut self, ratio: f32) {
        self.distance = (self.distance * ratio).clamp(CAMERA_DISTANCE_MIN, CAMERA_DISTANCE_MAX);
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
            }),
    )
    .init_resource::<TouchControls>()
    .init_resource::<OrbitCamera>()
    .insert_resource(ClearColor(day_night::default_clear_color()))
    .add_systems(Startup, (setup_world, spawn_props));
    day_night::plugin(&mut app);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    app.add_systems(Startup, setup_hud).add_systems(
        Update,
        (
            read_touch_controls,
            update_joystick_knob,
            update_jump_visual,
        ),
    );

    #[cfg(target_os = "ios")]
    app.add_systems(Update, read_pinch_gesture);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    app.add_systems(Startup, lock_cursor)
        .add_systems(Update, (grab_cursor, read_mouse_look, read_mouse_zoom));

    app.add_systems(
        Update,
        (
            tint_ocean,
            animate_ocean,
            (move_player, animate_walk, follow_camera).chain(),
        ),
    )
    .run();
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

fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ocean_mesh = meshes.add(
        Plane3d::default()
            .mesh()
            .size(OCEAN_SIZE, OCEAN_SIZE)
            .subdivisions(48),
    );
    let palette = day_night::water_palette(0.0);
    let surface = materials.add(ocean_surface_material(&palette));
    commands
        .spawn((
            Ocean,
            Transform::from_xyz(0.0, WATER_Y, 0.0),
            Visibility::default(),
            NotShadowCaster,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(ocean_mesh),
                MeshMaterial3d(surface.clone()),
                NotShadowCaster,
            ));
        });
    commands.insert_resource(OceanMats { surface });

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(LAND_SIZE, LAND_HEIGHT, LAND_SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.36, 0.55, 0.34),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, LAND_TOP - LAND_HEIGHT * 0.5, 0.0),
        Collider {
            shape: ColliderShape::Aabb {
                half_extents: Vec3::new(LAND_HALF, LAND_HEIGHT * 0.5, LAND_HALF),
            },
        },
    ));

    commands.spawn((
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("tree.glb"))),
        Transform::from_xyz(7.0, 0.0, -9.0),
        Collider {
            shape: ColliderShape::Cylinder {
                radius: TREE_TRUNK_RADIUS,
                height: TREE_TRUNK_HEIGHT,
            },
        },
    ));

    commands
        .spawn((
            Player,
            PlayerJump::default(),
            WalkCycle::default(),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("limbperson.glb"))),
            Transform::from_translation(PLAYER_SPAWN),
            Collider {
                shape: ColliderShape::Cylinder {
                    radius: PLAYER_RADIUS,
                    height: PLAYER_HEIGHT,
                },
            },
        ))
        .observe(bind_walk_bones);

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
    ));
}

/// One model authored in Blender and exported by `blender/export_glb.py`.
///
/// There is deliberately no scale field: a metre in Blender is a metre here, so
/// anything the wrong size gets resized in Blender and re-exported rather than
/// scaled at spawn time. See MODELING.md.
struct Prop {
    /// Path under `assets/`, e.g. `"models/my_object.glb"`.
    path: &'static str,
    /// Where the model's Blender origin lands. y = 0.0 is the island top.
    position: Vec3,
    /// Turn about the world Y axis. At 0 the model faces the way it faced
    /// along -Y in Blender.
    yaw_deg: f32,
}

const PROPS: &[Prop] = &[];

fn spawn_props(mut commands: Commands, asset_server: Res<AssetServer>) {
    for prop in PROPS {
        commands.spawn((
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(prop.path))),
            Transform::from_translation(prop.position)
                .with_rotation(Quat::from_rotation_y(prop.yaw_deg.to_radians())),
        ));
    }
}

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
) {
    controls.jump_just_pressed = false;
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

    for touch in touches.iter_just_pressed() {
        let pos = touch.position();
        let id = touch.id();
        if controls.stick_id.is_none() && in_circle(pos, layout.stick_center, layout.stick_radius * 1.15)
        {
            controls.stick_id = Some(id);
        } else if controls.jump_id.is_none()
            && in_circle(pos, layout.jump_center, layout.jump_radius * 1.2)
        {
            controls.jump_id = Some(id);
            controls.jump_just_pressed = true;
        } else if controls.look_id.is_none() {
            controls.look_id = Some(id);
            controls.look_last = pos;
        } else if controls.pinch_id.is_none() {
            controls.pinch_id = Some(id);
            controls.pinch_last_dist = 0.0;
        }
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
            orbit.zoom_by_ratio((controls.pinch_last_dist / dist).clamp(0.82, 1.22));
        }
        controls.pinch_last_dist = dist;
        controls.look_last = a;
    } else {
        controls.pinch_last_dist = 0.0;
        if let Some(pos) = look_pos {
            let delta = pos - controls.look_last;
            controls.look_last = pos;
            orbit.yaw -= delta.x * LOOK_SENSITIVITY;
            orbit.pitch = (orbit.pitch + delta.y * LOOK_SENSITIVITY).clamp(PITCH_MIN, PITCH_MAX);
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn lock_cursor(mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn grab_cursor(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
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
        orbit.yaw -= event.delta.x * LOOK_SENSITIVITY;
        orbit.pitch = (orbit.pitch + event.delta.y * LOOK_SENSITIVITY).clamp(PITCH_MIN, PITCH_MAX);
    }
}

#[cfg(target_os = "ios")]
fn read_pinch_gesture(
    mut pinch: MessageReader<bevy::input::gestures::PinchGesture>,
    mut orbit: ResMut<OrbitCamera>,
) {
    for event in pinch.read() {
        orbit.zoom_by_ratio((1.0 - event.0).clamp(0.7, 1.4));
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_mouse_zoom(mut scroll: MessageReader<MouseWheel>, mut orbit: ResMut<OrbitCamera>) {
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
    node.left = Val::Percent(max + controls.stick_value.x * max);
    node.top = Val::Percent(max - controls.stick_value.y * max);
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
    color.0 = if held {
        Color::srgba(1.0, 0.82, 0.2, 0.55)
    } else {
        Color::srgba(1.0, 0.82, 0.2, 0.32)
    };
}

fn over_island(pos: Vec3) -> bool {
    pos.x.abs() <= LAND_HALF && pos.z.abs() <= LAND_HALF
}

fn inland_island(pos: Vec3) -> bool {
    pos.x.abs() <= LAND_HALF - ISLAND_INLAND && pos.z.abs() <= LAND_HALF - ISLAND_INLAND
}

fn on_land(pos: Vec3) -> bool {
    over_island(pos) && pos.y >= LAND_TOP - 0.05
}

fn wading(pos: Vec3) -> bool {
    !on_land(pos) && pos.y < WATER_Y + 0.25
}

fn land_or_water_support(pos: Vec3, elapsed: f32) -> f32 {
    if inland_island(pos) || (over_island(pos) && pos.y >= LAND_TOP - LAND_STEP_UP) {
        LAND_TOP
    } else {
        water_rest_y(elapsed)
    }
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

fn resolve_player_solids(pos: &mut Vec3, solids: &[(Vec3, Collider)]) {
    for &(center, collider) in solids {
        match collider.shape {
            ColliderShape::Aabb { half_extents } => {
                let (min, max) = aabb_world(center, half_extents);
                if standing_on_aabb(*pos, min, max) || inland_island(*pos) {
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
    let mut bound = 0;
    for entity in children.iter_descendants(ready.entity) {
        let Ok((name, transform)) = named.get(entity) else {
            continue;
        };
        let Some(kind) = walk_bone_kind(name.as_str()) else {
            continue;
        };
        commands.entity(entity).insert(WalkBone {
            kind,
            rest_rotation: transform.rotation,
        });
        bound += 1;
    }
    if bound < 6 {
        warn!("walk bones: bound {bound}/6 on limbperson");
    }
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

fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<TouchControls>,
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    solids: Query<(&Transform, &Collider), Without<Player>>,
    mut players: Query<(&mut Transform, &mut PlayerJump, &mut WalkCycle), With<Player>>,
) {
    let Ok((mut transform, mut jump, mut walk)) = players.single_mut() else {
        return;
    };

    let solids: Vec<(Vec3, Collider)> = solids
        .iter()
        .map(|(solid, collider)| (solid.translation, *collider))
        .collect();

    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let support_y = land_or_water_support(transform.translation, elapsed);
    let grounded = transform.translation.y <= support_y + WATER_JUMP_SLOP;
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
    walk.speed = if speed_scale > 0.0 {
        if slowed {
            speed_scale * WATER_MOVE_SCALE
        } else {
            speed_scale
        }
    } else {
        0.0
    };
    if speed_scale > 0.0 {
        let speed = if slowed {
            MOVE_SPEED * WATER_MOVE_SCALE
        } else {
            MOVE_SPEED
        };
        apply_planar_move(&mut transform, wish, speed * speed_scale * dt, dt);
        resolve_player_solids(&mut transform.translation, &solids);
    }

    if keyboard.just_pressed(KeyCode::Space) || controls.jump_just_pressed {
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

    jump.velocity_y -= GRAVITY * dt;
    transform.translation.y += jump.velocity_y * dt;
    resolve_player_solids(&mut transform.translation, &solids);

    let support_y = land_or_water_support(transform.translation, elapsed);
    if jump.velocity_y <= 0.0 && transform.translation.y <= support_y {
        transform.translation.y = support_y;
        jump.velocity_y = 0.0;
    }
}

fn animate_walk(
    time: Res<Time>,
    mut walks: Query<&mut WalkCycle, With<Player>>,
    mut bones: Query<(&WalkBone, &mut Transform)>,
) {
    let Ok(mut walk) = walks.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    let target = if walk.speed > 0.04 { 1.0 } else { 0.0 };
    walk.weight = walk.weight.lerp(target, (dt * WALK_BLEND).min(1.0));
    if walk.weight < 0.01 {
        walk.weight = 0.0;
    } else {
        walk.phase += dt * WALK_STRIDE_FREQ * walk.speed.max(0.35);
    }
    let swing = walk.phase.sin();
    let weight = walk.weight;
    for (bone, mut transform) in &mut bones {
        transform.rotation = bone.rest_rotation * walk_offset(bone.kind, swing, weight);
    }
}

fn animate_ocean(time: Res<Time>, mut oceans: Query<&mut Transform, With<Ocean>>) {
    let bob = (time.elapsed_secs() * 0.65).sin() * 0.035;
    for mut transform in &mut oceans {
        transform.translation.y = WATER_Y + bob;
    }
}

/// One sea on every platform: a translucent surface blended over whatever sits
/// behind it. Both builds run the same transparent pass, so a single material
/// here is what keeps the phone and the desktop showing the same water.
fn ocean_surface_material(palette: &day_night::WaterPalette) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(palette.rgb[0], palette.rgb[1], palette.rgb[2], palette.alpha),
        perceptual_roughness: 0.38,
        reflectance: 0.28,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

fn tint_ocean(
    clock: Res<day_night::DayClock>,
    mats: Res<OceanMats>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let palette = day_night::water_palette(clock.time);
    if let Some(mut material) = materials.get_mut(&mats.surface) {
        *material = ocean_surface_material(&palette);
    }
}

fn follow_camera(
    orbit: Res<OrbitCamera>,
    players: Query<&Transform, With<Player>>,
    mut cameras: Query<
        (&mut Transform, &mut Projection),
        (With<ThirdPersonCamera>, Without<Player>),
    >,
) {
    let Ok(player) = players.single() else {
        return;
    };
    let Ok((mut camera, mut projection)) = cameras.single_mut() else {
        return;
    };

    let look_up = if orbit.pitch < 0.0 {
        (orbit.pitch / PITCH_MIN).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if let Projection::Perspective(perspective) = &mut *projection {
        perspective.fov = CAMERA_FOV + (CAMERA_LOOK_UP_FOV - CAMERA_FOV) * look_up;
        perspective.far = CAMERA_FAR;
    }

    let yaw = Quat::from_rotation_y(orbit.yaw);
    if orbit.pitch >= 0.0 {
        let look_at = player.translation + Vec3::Y * LOOK_HEIGHT;
        let offset =
            yaw * Quat::from_rotation_x(-orbit.pitch) * Vec3::new(0.0, 0.0, orbit.distance);
        camera.translation = look_at + offset;
        camera.look_at(look_at, Vec3::Y);
        return;
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
}
