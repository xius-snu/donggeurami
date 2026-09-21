//! Local day–night cycle. Starts at daytime when the app opens.
//!
//! A later server can own [`DayClock::time`] and skip [`tick_clock`].

use bevy::light::{CascadeShadowConfigBuilder, NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

/// 10 min day, 1.5 min sunset, 7 min night, 1.5 min sunrise.
pub const DAY_SECS: f32 = 10.0 * 60.0;
pub const SUNSET_SECS: f32 = 1.5 * 60.0;
pub const NIGHT_SECS: f32 = 7.0 * 60.0;
pub const SUNRISE_SECS: f32 = 1.5 * 60.0;
pub const CYCLE_SECS: f32 = DAY_SECS + SUNSET_SECS + NIGHT_SECS + SUNRISE_SECS;

/// Set above 1.0 to preview the cycle faster. Keep at 1.0 once a server owns the clock.
const TIME_SCALE: f32 = 20.0;
const SKY_DISTANCE: f32 = 170.0;
const LIGHT_DISTANCE: f32 = 48.0;
const SUN_DISC_RADIUS: f32 = 7.5;
const MOON_DISC_RADIUS: f32 = 5.2;
const CLOUD_COUNT: u32 = 24;
const STAR_COUNT: u32 = 160;
const STAR_DISTANCE: f32 = 185.0;

const SKY_DAY: [f32; 3] = [0.53, 0.80, 0.92];
const SKY_GOLDEN: [f32; 3] = [0.96, 0.55, 0.22];
const SKY_PINK: [f32; 3] = [0.95, 0.42, 0.36];
const SKY_DUSK: [f32; 3] = [0.62, 0.18, 0.10];
const SKY_NIGHT: [f32; 3] = [0.015, 0.02, 0.055];
const SKY_MOONLIT: [f32; 3] = [0.04, 0.055, 0.12];
const SKY_DAWN: [f32; 3] = [0.98, 0.52, 0.36];

const CLOUD_DAY: [f32; 3] = [0.96, 0.97, 1.0];
const CLOUD_GOLDEN: [f32; 3] = [1.0, 0.58, 0.28];
const CLOUD_PINK: [f32; 3] = [1.0, 0.46, 0.38];
const CLOUD_DUSK: [f32; 3] = [0.88, 0.32, 0.18];
const CLOUD_NIGHT: [f32; 3] = [0.16, 0.18, 0.28];
const CLOUD_DAWN: [f32; 3] = [1.0, 0.62, 0.48];

/// Seconds into the repeating cycle. `0` is the start of daytime.
#[derive(Resource)]
pub struct DayClock {
    pub time: f32,
}

impl Default for DayClock {
    fn default() -> Self {
        Self { time: 0.0 }
    }
}

impl DayClock {
    pub fn wrap(&mut self) {
        self.time = self.time.rem_euclid(CYCLE_SECS);
    }
}

#[derive(Resource)]
struct SkyVisuals {
    sun_core: Handle<StandardMaterial>,
    sun_halo: Handle<StandardMaterial>,
    sun_corona: Handle<StandardMaterial>,
    moon_core: Handle<StandardMaterial>,
    moon_halo: Handle<StandardMaterial>,
    cloud: Handle<StandardMaterial>,
    star: Handle<StandardMaterial>,
}

#[derive(Component)]
struct SunLight;

#[derive(Component)]
struct MoonLight;

#[derive(Component)]
struct SkyFill;

#[derive(Component)]
struct SunDisc;

#[derive(Component)]
struct MoonDisc;

#[derive(Component)]
struct StarField;

#[derive(Component)]
struct CloudCluster {
    azimuth: f32,
    elevation: f32,
    distance: f32,
    speed: f32,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<DayClock>()
        .add_systems(Startup, setup_sky)
        .add_systems(Update, (tick_clock, apply_sky).chain())
        .add_systems(Update, drift_clouds);
}

fn setup_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    clock: Res<DayClock>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let state = sky_state(clock.time);
    apply_ambient(&mut ambient, &state);

    let sun_cascades = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 12.0,
        maximum_distance: 70.0,
        ..default()
    };

    commands.spawn((
        SunLight,
        DirectionalLight {
            illuminance: state.sun_illuminance,
            color: state.sun_color,
            shadow_maps_enabled: true,
            ..default()
        },
        light_transform(state.sun_dir),
        sun_cascades.build(),
    ));

    commands.spawn((
        MoonLight,
        DirectionalLight {
            illuminance: state.moon_illuminance,
            color: state.moon_color,
            ..default()
        },
        light_transform(state.moon_dir),
    ));

    commands.spawn((
        SkyFill,
        DirectionalLight {
            illuminance: state.fill_illuminance,
            color: state.fill_color,
            ..default()
        },
        light_transform(state.fill_dir),
    ));

    let sun_core = materials.add(sun_core_material(state.sun_disc));
    let sun_halo = materials.add(sun_halo_material(state.sun_disc, 0.12, 0.9));
    let sun_corona = materials.add(sun_halo_material(state.sun_disc, 0.05, 0.35));
    let moon_core = materials.add(moon_core_material(state.moon_disc));
    let moon_halo = materials.add(moon_halo_material(state.moon_disc));
    let cloud = materials.add(cloud_material(state.cloud, state.cloud_alpha));
    let star = materials.add(star_material(state.stars));

    let sun_mesh = meshes.add(ico(SUN_DISC_RADIUS, 1));
    let sun_halo_mesh = meshes.add(ico(SUN_DISC_RADIUS * 1.55, 1));
    let sun_corona_mesh = meshes.add(ico(SUN_DISC_RADIUS * 2.35, 1));
    let moon_mesh = meshes.add(uv_ball(MOON_DISC_RADIUS, 36, 20));
    let moon_halo_mesh = meshes.add(uv_ball(MOON_DISC_RADIUS * 1.28, 28, 16));
    let puff_mesh = meshes.add(ico(8.0, 1));
    let star_mesh = meshes.add(ico(1.0, 0));

    commands
        .spawn((
            SunDisc,
            Mesh3d(sun_mesh),
            MeshMaterial3d(sun_core.clone()),
            disc_transform(state.sun_dir),
            disc_visibility(state.sun_visible),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(sun_halo_mesh),
                MeshMaterial3d(sun_halo.clone()),
                NotShadowCaster,
                NotShadowReceiver,
            ));
            parent.spawn((
                Mesh3d(sun_corona_mesh),
                MeshMaterial3d(sun_corona.clone()),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        });

    commands
        .spawn((
            MoonDisc,
            Mesh3d(moon_mesh),
            MeshMaterial3d(moon_core.clone()),
            disc_transform(state.moon_dir),
            disc_visibility(state.moon_visible),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(moon_halo_mesh),
                MeshMaterial3d(moon_halo.clone()),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        });

    spawn_clouds(&mut commands, puff_mesh, cloud.clone());
    spawn_stars(&mut commands, star_mesh, star.clone(), state.stars);

    commands.insert_resource(SkyVisuals {
        sun_core,
        sun_halo,
        sun_corona,
        moon_core,
        moon_halo,
        cloud,
        star,
    });
}

fn tick_clock(time: Res<Time>, mut clock: ResMut<DayClock>) {
    clock.time += time.delta_secs() * TIME_SCALE;
    clock.wrap();
}

fn apply_sky(
    clock: Res<DayClock>,
    visuals: Res<SkyVisuals>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sun_lights: Query<
        (&mut DirectionalLight, &mut Transform),
        (
            With<SunLight>,
            Without<MoonLight>,
            Without<SkyFill>,
            Without<SunDisc>,
            Without<MoonDisc>,
        ),
    >,
    mut moon_lights: Query<
        (&mut DirectionalLight, &mut Transform),
        (
            With<MoonLight>,
            Without<SunLight>,
            Without<SkyFill>,
            Without<SunDisc>,
            Without<MoonDisc>,
        ),
    >,
    mut fill_lights: Query<
        (&mut DirectionalLight, &mut Transform),
        (
            With<SkyFill>,
            Without<SunLight>,
            Without<MoonLight>,
            Without<SunDisc>,
            Without<MoonDisc>,
        ),
    >,
    mut sun_discs: Query<
        (&mut Transform, &mut Visibility),
        (
            With<SunDisc>,
            Without<MoonDisc>,
            Without<SunLight>,
            Without<MoonLight>,
            Without<SkyFill>,
            Without<StarField>,
        ),
    >,
    mut moon_discs: Query<
        (&mut Transform, &mut Visibility),
        (
            With<MoonDisc>,
            Without<SunDisc>,
            Without<SunLight>,
            Without<MoonLight>,
            Without<SkyFill>,
            Without<StarField>,
        ),
    >,
    mut starfields: Query<
        &mut Visibility,
        (With<StarField>, Without<SunDisc>, Without<MoonDisc>),
    >,
) {
    let state = sky_state(clock.time);
    clear.0 = mix_srgb_color(state.sky);
    apply_ambient(&mut ambient, &state);

    for (mut light, mut transform) in &mut sun_lights {
        light.illuminance = state.sun_illuminance;
        light.color = state.sun_color;
        *transform = light_transform(state.sun_dir);
    }

    for (mut light, mut transform) in &mut moon_lights {
        light.illuminance = state.moon_illuminance;
        light.color = state.moon_color;
        *transform = light_transform(state.moon_dir);
    }

    for (mut light, mut transform) in &mut fill_lights {
        light.illuminance = state.fill_illuminance;
        light.color = state.fill_color;
        *transform = light_transform(state.fill_dir);
    }

    for (mut transform, mut visibility) in &mut sun_discs {
        *transform = disc_transform(state.sun_dir);
        *visibility = disc_visibility(state.sun_visible);
    }

    for (mut transform, mut visibility) in &mut moon_discs {
        *transform = disc_transform(state.moon_dir);
        *visibility = disc_visibility(state.moon_visible);
    }

    if let Some(mut material) = materials.get_mut(&visuals.sun_core) {
        apply_sun_core(&mut material, state.sun_disc);
    }
    if let Some(mut material) = materials.get_mut(&visuals.sun_halo) {
        apply_sun_halo(&mut material, state.sun_disc, 0.12, 0.9);
    }
    if let Some(mut material) = materials.get_mut(&visuals.sun_corona) {
        apply_sun_halo(&mut material, state.sun_disc, 0.05, 0.35);
    }
    if let Some(mut material) = materials.get_mut(&visuals.moon_core) {
        apply_moon_core(&mut material, state.moon_disc);
    }
    if let Some(mut material) = materials.get_mut(&visuals.moon_halo) {
        apply_moon_halo(&mut material, state.moon_disc);
    }
    if let Some(mut material) = materials.get_mut(&visuals.cloud) {
        apply_cloud_material(&mut material, state.cloud, state.cloud_alpha);
    }
    if let Some(mut material) = materials.get_mut(&visuals.star) {
        apply_star_material(&mut material, state.stars);
    }
    for mut visibility in &mut starfields {
        *visibility = disc_visibility(state.stars > 0.02);
    }
}

fn drift_clouds(time: Res<Time>, mut clouds: Query<(&mut CloudCluster, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut cloud, mut transform) in &mut clouds {
        cloud.azimuth += cloud.speed * dt;
        let bob = 0.025 * (cloud.azimuth * 1.7).sin();
        transform.translation =
            dir_from_elev_azim(cloud.elevation + bob, cloud.azimuth) * cloud.distance;
    }
}

struct SkyState {
    sun_dir: Vec3,
    moon_dir: Vec3,
    sun_illuminance: f32,
    moon_illuminance: f32,
    fill_illuminance: f32,
    fill_dir: Vec3,
    sun_color: Color,
    moon_color: Color,
    fill_color: Color,
    sky: [f32; 3],
    ambient_brightness: f32,
    ambient_color: Color,
    sun_visible: bool,
    moon_visible: bool,
    sun_disc: [f32; 3],
    moon_disc: [f32; 3],
    cloud: [f32; 3],
    cloud_alpha: f32,
    stars: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct WaterPalette {
    pub rgb: [f32; 3],
    pub alpha: f32,
}

/// Day water is a mid ocean blue; night water is near-black. Unlit water
/// looked brighter at night because it kept a daylight color against a dark sky.
pub(crate) fn water_palette(time: f32) -> WaterPalette {
    let state = sky_state(time);
    let sun_h = state.sun_dir.y.max(0.0);
    let moon_h = state.moon_dir.y.max(0.0);
    let day = smoothstep(0.02, 0.32, sun_h);
    let dusk = smoothstep(0.0, 0.16, sun_h);
    let moon = smoothstep(0.0, 0.22, moon_h);
    let day_rgb = [0.08, 0.30, 0.46];
    let dusk_rgb = [0.07, 0.14, 0.22];
    let night_rgb = mix3([0.010, 0.020, 0.045], [0.025, 0.045, 0.080], moon);
    let rgb = mix3(mix3(night_rgb, dusk_rgb, dusk), day_rgb, day);
    let alpha = lerp(0.80, 0.58, day);
    WaterPalette { rgb, alpha }
}

fn sky_state(time: f32) -> SkyState {
    let t = time.rem_euclid(CYCLE_SECS);
    let sunset_end = DAY_SECS + SUNSET_SECS;
    let night_end = sunset_end + NIGHT_SECS;

    let sky = sample_rgb(
        t,
        &[
            (0.0, SKY_DAY),
            (DAY_SECS * 0.76, SKY_DAY),
            (DAY_SECS, SKY_GOLDEN),
            (DAY_SECS + SUNSET_SECS * 0.28, SKY_PINK),
            (DAY_SECS + SUNSET_SECS * 0.62, SKY_DUSK),
            (sunset_end, SKY_NIGHT),
            (sunset_end + NIGHT_SECS * 0.45, SKY_MOONLIT),
            (night_end, SKY_NIGHT),
            (night_end + SUNRISE_SECS * 0.42, SKY_DAWN),
            (CYCLE_SECS, SKY_DAY),
        ],
    );
    let cloud = sample_rgb(
        t,
        &[
            (0.0, CLOUD_DAY),
            (DAY_SECS * 0.76, CLOUD_DAY),
            (DAY_SECS, CLOUD_GOLDEN),
            (DAY_SECS + SUNSET_SECS * 0.28, CLOUD_PINK),
            (DAY_SECS + SUNSET_SECS * 0.62, CLOUD_DUSK),
            (sunset_end, CLOUD_NIGHT),
            (night_end, CLOUD_NIGHT),
            (night_end + SUNRISE_SECS * 0.42, CLOUD_DAWN),
            (CYCLE_SECS, CLOUD_DAY),
        ],
    );
    let ambient_rgb = sample_rgb(
        t,
        &[
            (0.0, [1.0, 1.0, 1.0]),
            (DAY_SECS * 0.76, [1.0, 1.0, 1.0]),
            (DAY_SECS, [1.0, 0.72, 0.48]),
            (DAY_SECS + SUNSET_SECS * 0.45, [1.0, 0.42, 0.28]),
            (sunset_end, [0.55, 0.22, 0.16]),
            (night_end, [0.32, 0.42, 0.78]),
            (night_end + SUNRISE_SECS * 0.45, [0.85, 0.62, 0.52]),
            (CYCLE_SECS, [1.0, 1.0, 1.0]),
        ],
    );
    let ambient_brightness = sample_scalar(
        t,
        &[
            (0.0, 340.0),
            (DAY_SECS * 0.76, 340.0),
            (DAY_SECS, 200.0),
            (sunset_end, 42.0),
            (sunset_end + NIGHT_SECS * 0.5, 50.0),
            (night_end, 42.0),
            (CYCLE_SECS, 340.0),
        ],
    );
    let cloud_alpha = sample_scalar(
        t,
        &[
            (0.0, 0.84),
            (DAY_SECS, 0.78),
            (sunset_end, 0.42),
            (night_end, 0.42),
            (CYCLE_SECS, 0.84),
        ],
    );
    let stars = sample_scalar(
        t,
        &[
            (0.0, 0.0),
            (DAY_SECS + SUNSET_SECS * 0.32, 0.0),
            (sunset_end, 1.0),
            (night_end, 1.0),
            (night_end + SUNRISE_SECS * 0.55, 0.0),
            (CYCLE_SECS, 0.0),
        ],
    );

    let sun_dir = dir_from_plane_angle(sun_plane_angle(t));
    let moon_dir = dir_from_plane_angle(sun_plane_angle(t) + std::f32::consts::PI);
    let sun_height = sun_dir.y.max(0.0);
    let moon_height = moon_dir.y.max(0.0);
    let warmth = 1.0 - smoothstep(0.12, 0.48, sun_height);
    let day_fill = smoothstep(0.02, 0.18, sun_height);

    let key_dir = mix_dir(moon_dir, sun_dir, day_fill);
    let fill_dir = {
        let mut dir = Vec3::new(-key_dir.x * 0.45, 0.9, 0.0);
        if dir.length_squared() < 0.05 {
            dir = Vec3::new(0.35, 0.94, 0.0);
        }
        dir.normalize_or_zero()
    };
    let fill_illuminance = lerp(
        140.0 * smoothstep(0.0, 0.22, moon_height),
        3_600.0 * smoothstep(0.0, 0.35, sun_height),
        day_fill,
    );
    let fill_color = mix_srgb_color(mix3(
        [0.42, 0.52, 0.80],
        mix3([0.80, 0.90, 1.0], [1.0, 0.74, 0.55], warmth),
        day_fill,
    ));

    SkyState {
        sun_dir,
        moon_dir,
        fill_dir,
        sun_illuminance: 7_000.0 * smoothstep(0.0, 0.35, sun_height),
        moon_illuminance: 180.0 * smoothstep(0.0, 0.22, moon_height),
        fill_illuminance,
        sun_color: mix_srgb_color(mix3([1.0, 0.97, 0.90], [1.0, 0.42, 0.16], warmth)),
        moon_color: Color::srgb(0.62, 0.72, 1.0),
        fill_color,
        sky,
        ambient_brightness,
        ambient_color: mix_srgb_color(ambient_rgb),
        sun_visible: sun_dir.y > -0.02,
        moon_visible: moon_dir.y > -0.02 && sun_height < 0.55,
        sun_disc: mix3([1.0, 0.93, 0.52], [1.0, 0.38, 0.08], warmth),
        moon_disc: [0.82, 0.85, 0.94],
        cloud,
        cloud_alpha,
        stars,
    }
}

fn sun_plane_angle(t: f32) -> f32 {
    let sunset_end = DAY_SECS + SUNSET_SECS;
    let night_end = sunset_end + NIGHT_SECS;
    if t <= DAY_SECS {
        lerp(0.40, std::f32::consts::PI - 0.40, t / DAY_SECS)
    } else if t <= sunset_end {
        lerp(
            std::f32::consts::PI - 0.40,
            std::f32::consts::PI + 0.16,
            smoothstep(DAY_SECS, sunset_end, t),
        )
    } else if t <= night_end {
        lerp(
            std::f32::consts::PI + 0.16,
            std::f32::consts::TAU - 0.16,
            (t - sunset_end) / NIGHT_SECS,
        )
    } else {
        lerp(-0.16, 0.40, smoothstep(night_end, CYCLE_SECS, t))
    }
}

fn dir_from_plane_angle(angle: f32) -> Vec3 {
    Vec3::new(angle.cos(), angle.sin(), 0.0).normalize()
}

fn dir_from_elev_azim(elev: f32, azim: f32) -> Vec3 {
    Vec3::new(elev.cos() * azim.cos(), elev.sin(), elev.cos() * azim.sin()).normalize()
}

fn light_transform(dir: Vec3) -> Transform {
    let up = if dir.y.abs() > 0.94 {
        Vec3::X
    } else {
        Vec3::Y
    };
    Transform::from_translation(dir * LIGHT_DISTANCE).looking_at(Vec3::ZERO, up)
}

fn disc_transform(dir: Vec3) -> Transform {
    Transform::from_translation(dir * SKY_DISTANCE)
}

fn disc_visibility(visible: bool) -> Visibility {
    if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

fn ico(radius: f32, subdivisions: u32) -> Mesh {
    Sphere::new(radius)
        .mesh()
        .ico(subdivisions)
        .expect("icosphere")
}

fn uv_ball(radius: f32, sectors: u32, stacks: u32) -> Mesh {
    Sphere::new(radius).mesh().uv(sectors, stacks)
}

fn spawn_clouds(commands: &mut Commands, puff_mesh: Handle<Mesh>, material: Handle<StandardMaterial>) {
    // Fibonacci/golden spiral over the upper hemisphere so clouds cover
    // overhead as well as the sides, instead of a single ring.
    const GOLDEN_ANGLE: f32 = 2.399_963_2;
    for i in 0..CLOUD_COUNT {
        let t = (i as f32 + 0.5) / CLOUD_COUNT as f32;
        let height = lerp(0.16, 0.995, t);
        let elevation = height.asin();
        let azimuth = i as f32 * GOLDEN_ANGLE + hash01(i, 1) * 0.2;
        let distance = lerp(92.0, 138.0, hash01(i, 2));
        let speed = lerp(0.016, 0.042, hash01(i, 3)) * if i % 2 == 0 { 1.0 } else { -0.7 };
        let cluster_scale = lerp(0.7, 1.35, hash01(i, 4));

        commands
            .spawn((
                CloudCluster {
                    azimuth,
                    elevation,
                    distance,
                    speed,
                },
                Transform::from_translation(dir_from_elev_azim(elevation, azimuth) * distance),
                NotShadowCaster,
                NotShadowReceiver,
                Visibility::default(),
            ))
            .with_children(|parent| {
                for (offset, scale) in cloud_puffs(i) {
                    parent.spawn((
                        Mesh3d(puff_mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::from_translation(offset * cluster_scale).with_scale(
                            Vec3::new(scale.x, scale.y, scale.z) * cluster_scale,
                        ),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                }
            });
    }
}

fn spawn_stars(
    commands: &mut Commands,
    star_mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    stars: f32,
) {
    commands
        .spawn((
            StarField,
            Transform::default(),
            disc_visibility(stars > 0.02),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .with_children(|parent| {
            for i in 0..STAR_COUNT {
                let u = hash01(i, 1);
                let v = hash01(i, 2);
                let azimuth = u * std::f32::consts::TAU;
                let y = lerp(0.08, 0.98, v);
                let radius = (1.0 - y * y).max(0.0).sqrt();
                let dir = Vec3::new(radius * azimuth.cos(), y, radius * azimuth.sin());
                let size = lerp(0.09, 0.36, hash01(i, 3).powf(1.8));
                parent.spawn((
                    Mesh3d(star_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(dir * STAR_DISTANCE).with_scale(Vec3::splat(size)),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
            }
        });
}

fn cloud_puffs(seed: u32) -> [(Vec3, Vec3); 6] {
    let jitter = |lane: u32| hash01(seed, 10 + lane) - 0.5;
    [
        (Vec3::new(jitter(1) * 3.0, 0.0, jitter(2) * 3.0), Vec3::new(1.55, 0.40, 1.15)),
        (Vec3::new(9.0 + jitter(3) * 2.0, 0.5, 2.0), Vec3::new(1.15, 0.34, 0.95)),
        (Vec3::new(-8.5, -0.2, 1.8 + jitter(4) * 2.0), Vec3::new(1.25, 0.36, 1.0)),
        (Vec3::new(4.2, 0.7, -7.5), Vec3::new(0.95, 0.30, 0.85)),
        (Vec3::new(-5.8, 0.35, -6.2), Vec3::new(1.05, 0.33, 0.9)),
        (Vec3::new(1.2 + jitter(5) * 2.0, 0.9, 6.8), Vec3::new(0.82, 0.28, 0.78)),
    ]
}

fn sun_core_material(rgb: [f32; 3]) -> StandardMaterial {
    let mut material = StandardMaterial::default();
    apply_sun_core(&mut material, rgb);
    material
}

fn sun_halo_material(rgb: [f32; 3], alpha: f32, glow: f32) -> StandardMaterial {
    let mut material = StandardMaterial::default();
    apply_sun_halo(&mut material, rgb, alpha, glow);
    material
}

fn moon_core_material(rgb: [f32; 3]) -> StandardMaterial {
    let mut material = StandardMaterial::default();
    apply_moon_core(&mut material, rgb);
    material
}

fn moon_halo_material(rgb: [f32; 3]) -> StandardMaterial {
    let mut material = StandardMaterial::default();
    apply_moon_halo(&mut material, rgb);
    material
}

fn cloud_material(rgb: [f32; 3], alpha: f32) -> StandardMaterial {
    let mut material = StandardMaterial::default();
    apply_cloud_material(&mut material, rgb, alpha);
    material
}

fn star_material(opacity: f32) -> StandardMaterial {
    let mut material = StandardMaterial::default();
    apply_star_material(&mut material, opacity);
    material
}

fn apply_sun_core(material: &mut StandardMaterial, rgb: [f32; 3]) {
    material.base_color = mix_srgb_color(rgb);
    material.emissive = LinearRgba::rgb(rgb[0] * 2.2, rgb[1] * 1.9, rgb[2] * 1.1);
    material.unlit = true;
    material.fog_enabled = false;
    material.perceptual_roughness = 1.0;
    material.reflectance = 0.0;
}

fn apply_sun_halo(material: &mut StandardMaterial, rgb: [f32; 3], alpha: f32, glow: f32) {
    apply_sky_glow(material, rgb, alpha, glow);
}

fn apply_moon_core(material: &mut StandardMaterial, rgb: [f32; 3]) {
    material.base_color = mix_srgb_color(rgb);
    material.emissive = LinearRgba::rgb(rgb[0] * 0.35, rgb[1] * 0.38, rgb[2] * 0.5);
    material.unlit = true;
    material.fog_enabled = false;
    material.perceptual_roughness = 0.95;
    material.reflectance = 0.0;
}

fn apply_moon_halo(material: &mut StandardMaterial, rgb: [f32; 3]) {
    apply_sky_glow(material, rgb, 0.1, 0.55);
}

fn apply_sky_glow(material: &mut StandardMaterial, rgb: [f32; 3], alpha: f32, glow: f32) {
    material.unlit = true;
    material.double_sided = true;
    material.cull_mode = None;
    material.reflectance = 0.0;
    material.fog_enabled = false;
    material.base_color = Color::srgba(rgb[0], rgb[1], rgb[2], alpha);
    material.emissive = LinearRgba::rgb(rgb[0] * glow, rgb[1] * glow * 0.82, rgb[2] * glow * 0.4);
    material.alpha_mode = AlphaMode::Add;
}

fn apply_cloud_material(material: &mut StandardMaterial, rgb: [f32; 3], alpha: f32) {
    material.emissive = LinearRgba::BLACK;
    material.unlit = true;
    material.double_sided = true;
    material.cull_mode = None;
    material.perceptual_roughness = 1.0;
    material.reflectance = 0.0;
    material.fog_enabled = false;
    material.base_color = Color::srgba(rgb[0], rgb[1], rgb[2], alpha);
    material.alpha_mode = AlphaMode::Blend;
}

fn apply_star_material(material: &mut StandardMaterial, opacity: f32) {
    let glow = opacity.clamp(0.0, 1.0);
    material.unlit = true;
    material.double_sided = true;
    material.cull_mode = None;
    material.reflectance = 0.0;
    material.base_color = Color::srgba(0.92, 0.95, 1.0, glow);
    material.emissive = LinearRgba::rgb(0.55 * glow, 0.6 * glow, 0.75 * glow);
    material.alpha_mode = AlphaMode::Add;
}

fn apply_ambient(ambient: &mut GlobalAmbientLight, state: &SkyState) {
    ambient.color = state.ambient_color;
    ambient.brightness = state.ambient_brightness;
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn mix_dir(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a.lerp(b, t.clamp(0.0, 1.0)).normalize_or_zero()
}

fn sample_rgb(time: f32, keys: &[(f32, [f32; 3])]) -> [f32; 3] {
    for pair in keys.windows(2) {
        let (t0, a) = pair[0];
        let (t1, b) = pair[1];
        if time <= t1 {
            return mix3(a, b, smoothstep(t0, t1, time));
        }
    }
    keys.last().map(|key| key.1).unwrap_or([0.0; 3])
}

fn sample_scalar(time: f32, keys: &[(f32, f32)]) -> f32 {
    for pair in keys.windows(2) {
        let (t0, a) = pair[0];
        let (t1, b) = pair[1];
        if time <= t1 {
            return lerp(a, b, smoothstep(t0, t1, time));
        }
    }
    keys.last().map(|key| key.1).unwrap_or(0.0)
}

pub(crate) fn default_clear_color() -> Color {
    mix_srgb_color(SKY_DAY)
}

fn mix_srgb_color(rgb: [f32; 3]) -> Color {
    Color::srgb(rgb[0], rgb[1], rgb[2])
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hash01(n: u32, lane: u32) -> f32 {
    let mut x = n
        .wrapping_add(1)
        .wrapping_mul(0x9E37_79B9)
        ^ lane.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    (x >> 8) as f32 / 16_777_216.0
}
