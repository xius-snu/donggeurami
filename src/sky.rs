//! The sky, the light it throws, and the sea. It is always mid-morning: the sun
//! stands still, and only the clouds move.

use bevy::light::{CascadeShadowConfigBuilder, NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use crate::WATER_Y;

/// Roughly 10 am. Straight overhead, every shadow would sit hidden underneath
/// the thing casting it; at 45° each one reaches out about as far as its caster
/// is tall.
const SUN_ELEVATION_DEG: f32 = 45.0;
/// Measured from +X towards +Z. From where the camera starts, behind the player
/// at +Z looking towards -Z, the light comes over its right shoulder and the
/// shadows fall ahead and to the left, where they can be seen.
const SUN_AZIMUTH_DEG: f32 = 40.0;
/// Whether the sun casts shadows. Off since 2026-10-03, on trial: Hajun asked
/// to see how much cooler the phone runs without the two shadow maps, which
/// are the whole shadow-casting world drawn twice more every frame. With it
/// off Bevy makes no shadow maps at all.
const SUN_SHADOWS: bool = false;
const SUN_ILLUMINANCE: f32 = 7_000.0;
const SUN_COLOR: [f32; 3] = [1.0, 0.97, 0.90];
/// Skylight from above and a little away from the sun, so the side of anything
/// facing away from it is a cool blue rather than black. Casts no shadows.
const FILL_ILLUMINANCE: f32 = 3_600.0;
const FILL_COLOR: [f32; 3] = [0.80, 0.90, 1.0];
const AMBIENT_BRIGHTNESS: f32 = 340.0;

const SKY_DISTANCE: f32 = 170.0;
const LIGHT_DISTANCE: f32 = 48.0;
const SUN_DISC_RADIUS: f32 = 7.5;
const CLOUD_COUNT: u32 = 24;

const SKY: [f32; 3] = [0.53, 0.80, 0.92];
const SUN_DISC: [f32; 3] = [1.0, 0.93, 0.52];
const CLOUD: [f32; 3] = [0.96, 0.97, 1.0];
const CLOUD_ALPHA: f32 = 0.84;

/// Wide enough that its edge is never in sight from anywhere a player can get
/// to, with the camera pulled all the way back.
const SEA_SIZE: f32 = 1200.0;
/// A mid ocean blue, blended over whatever is below it. The same colour the
/// sea had before it was taken out, which was checked on the phone.
const SEA: [f32; 3] = [0.08, 0.30, 0.46];
const SEA_ALPHA: f32 = 0.58;
const SEA_BOB_AMP: f32 = 0.035;
const SEA_BOB_SPEED: f32 = 0.65;

#[derive(Component)]
struct CloudCluster {
    azimuth: f32,
    elevation: f32,
    distance: f32,
    speed: f32,
}

#[derive(Component, Clone)]
struct Sea;

/// What water looks like: the sea's material, which water that is part of a
/// map — the town's fountain — is drawn with too, so that the two match; and
/// the one sheet of sea that every island is laid on.
#[derive(Resource)]
pub(crate) struct Water {
    pub material: Handle<StandardMaterial>,
    sheet: Handle<Mesh>,
}

impl FromWorld for Water {
    fn from_world(world: &mut World) -> Self {
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(sea_material());
        let sheet = world.resource_mut::<Assets<Mesh>>().add(
            Plane3d::default()
                .mesh()
                .size(SEA_SIZE, SEA_SIZE)
                .subdivisions(48),
        );
        Self { material, sheet }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(ClearColor(srgb(SKY)))
        .init_resource::<Water>()
        .add_systems(Startup, setup_sky)
        .add_systems(Update, (drift_clouds, bob_sea));
}

/// The sea: a sheet of translucent blue at [`WATER_Y`], bobbing gently. Shared
/// by every platform, like everything else in here; it is drawn in the sorted
/// transparent phase, which is the one the phone's camera needs
/// `NoIndirectDrawing` for.
pub(crate) fn sea(water: &Water) -> impl Bundle {
    (
        Sea,
        Mesh3d(water.sheet.clone()),
        MeshMaterial3d(water.material.clone()),
        Transform::from_xyz(0.0, WATER_Y, 0.0),
        NotShadowCaster,
    )
}

fn bob_sea(time: Res<Time>, mut seas: Query<&mut Transform, With<Sea>>) {
    let bob = (time.elapsed_secs() * SEA_BOB_SPEED).sin() * SEA_BOB_AMP;
    for mut transform in &mut seas {
        transform.translation.y = WATER_Y + bob;
    }
}

fn setup_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    ambient.color = Color::WHITE;
    ambient.brightness = AMBIENT_BRIGHTNESS;

    let sun_dir = dir_from_elev_azim(
        SUN_ELEVATION_DEG.to_radians(),
        SUN_AZIMUTH_DEG.to_radians(),
    );
    let fill_dir = Vec3::new(-sun_dir.x * 0.45, 0.9, -sun_dir.z * 0.45).normalize();

    // Far enough to reach the buildings on the far side of the ring from
    // anywhere in the town: the ring is about 140 m across and the camera can
    // pull back 15 m behind the player. Beyond this, shadows are not drawn.
    //
    // Two shadow maps rather than Bevy's four: one for the 12 m round the
    // camera, where the players and the fountain are, and one for the rest.
    // Every map is the whole shadow-casting world drawn again, and on the
    // phone's GPU at 120 frames a second the four took about 40% of each
    // frame's work: with them it ran out of GPU once it warmed up and dropped
    // frames. Nearby shadows are the same as with four; distant ones are a
    // little softer.
    let sun_cascades = CascadeShadowConfigBuilder {
        num_cascades: 2,
        first_cascade_far_bound: 12.0,
        maximum_distance: 200.0,
        ..default()
    };

    commands.spawn((
        DirectionalLight {
            illuminance: SUN_ILLUMINANCE,
            color: srgb(SUN_COLOR),
            shadow_maps_enabled: SUN_SHADOWS,
            ..default()
        },
        light_transform(sun_dir),
        sun_cascades.build(),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: FILL_ILLUMINANCE,
            color: srgb(FILL_COLOR),
            ..default()
        },
        light_transform(fill_dir),
    ));

    commands
        .spawn((
            Mesh3d(meshes.add(ico(SUN_DISC_RADIUS, 1))),
            MeshMaterial3d(materials.add(sun_core_material())),
            Transform::from_translation(sun_dir * SKY_DISTANCE),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(ico(SUN_DISC_RADIUS * 1.55, 1))),
                MeshMaterial3d(materials.add(sun_glow_material(0.12, 0.9))),
                NotShadowCaster,
                NotShadowReceiver,
            ));
            parent.spawn((
                Mesh3d(meshes.add(ico(SUN_DISC_RADIUS * 2.35, 1))),
                MeshMaterial3d(materials.add(sun_glow_material(0.05, 0.35))),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        });

    let puff_mesh = meshes.add(ico(8.0, 1));
    let cloud = materials.add(cloud_material());
    spawn_clouds(&mut commands, puff_mesh, cloud);
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

fn ico(radius: f32, subdivisions: u32) -> Mesh {
    Sphere::new(radius)
        .mesh()
        .ico(subdivisions)
        .expect("icosphere")
}

fn spawn_clouds(
    commands: &mut Commands,
    puff_mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
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

fn sun_core_material() -> StandardMaterial {
    let rgb = SUN_DISC;
    StandardMaterial {
        base_color: srgb(rgb),
        emissive: LinearRgba::rgb(rgb[0] * 2.2, rgb[1] * 1.9, rgb[2] * 1.1),
        unlit: true,
        fog_enabled: false,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    }
}

fn sun_glow_material(alpha: f32, glow: f32) -> StandardMaterial {
    let rgb = SUN_DISC;
    StandardMaterial {
        unlit: true,
        double_sided: true,
        cull_mode: None,
        reflectance: 0.0,
        fog_enabled: false,
        base_color: Color::srgba(rgb[0], rgb[1], rgb[2], alpha),
        emissive: LinearRgba::rgb(rgb[0] * glow, rgb[1] * glow * 0.82, rgb[2] * glow * 0.4),
        alpha_mode: AlphaMode::Add,
        ..default()
    }
}

fn sea_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(SEA[0], SEA[1], SEA[2], SEA_ALPHA),
        perceptual_roughness: 0.38,
        reflectance: 0.28,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

fn cloud_material() -> StandardMaterial {
    StandardMaterial {
        emissive: LinearRgba::BLACK,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        fog_enabled: false,
        base_color: Color::srgba(CLOUD[0], CLOUD[1], CLOUD[2], CLOUD_ALPHA),
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn srgb(rgb: [f32; 3]) -> Color {
    Color::srgb(rgb[0], rgb[1], rgb[2])
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
