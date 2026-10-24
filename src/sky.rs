//! The sky, the light it throws, and the sea, round a day and a night of
//! twenty minutes: ten minutes of day, a minute and a half of sunset, seven
//! of night and a minute and a half of sunrise ([`CYCLE_SECS`]). Hajun had
//! the day and the night back on 2026-10-16, with the town's street lamps
//! lit through the night (`lamps`); from 2026-09-23 until then it was always
//! mid-morning, the sun standing still.
//!
//! Noon is the town as it was then, which Hajun chose the light of on
//! 2026-10-13 ("Colour" in `RENDERING.md`): the sun 45° up, the same light,
//! the same sky and clouds. The sun climbs to it quickly and stays near it
//! for most of the day ([`sun_way`]), so that most of the day looks like
//! that, and goes down warm and orange into a pink and then a deep blue sky.
//! Through the night a moon crosses it, and stars come out.
//!
//! The day goes by the town's clock (`clock`), not by the game's time, as the
//! fountain's jet does, and online the town's clock is the server's: every
//! device is at the same time of day, so that everyone online in the town
//! sees the same sky. `RT_TIME_OF_DAY` stops it at one time, for trying
//! things out ([`Clock`]).
//!
//! The sun and the moon are one light ([`KeyLight`]), the sun's by day and
//! the moon's by night: one is always under the horizon, and its light out,
//! when the other is up. So the night costs no more light than the day, and
//! the day no more than it did when the sun stood still: two directional
//! lights, the key and the fill, and no shadows ([`SUN_SHADOWS`]).
//!
//! Nothing here is written to that has not changed: the sky is worked out
//! afresh [`STEP`] seconds at a time, and only what came out different from
//! the last time is set, so that through the day only the sun's way moves and
//! no material is touched.

use std::f32::consts::PI;

use bevy::light::{CascadeShadowConfigBuilder, NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;

use crate::WATER_Y;
use crate::clock::TownClock;

/// How long each part of the day is, in seconds. The day starts with the sun
/// already up, past sunrise.
pub(crate) const DAY_SECS: f32 = 600.0;
pub(crate) const SUNSET_SECS: f32 = 90.0;
pub(crate) const NIGHT_SECS: f32 = 420.0;
pub(crate) const SUNRISE_SECS: f32 = 90.0;
/// A whole day and night: twenty minutes.
pub(crate) const CYCLE_SECS: f32 = DAY_SECS + SUNSET_SECS + NIGHT_SECS + SUNRISE_SECS;
const SUNSET_END: f32 = DAY_SECS + SUNSET_SECS;
const NIGHT_END: f32 = SUNSET_END + NIGHT_SECS;
/// How often the sky is worked out again, in seconds of the day. In a step
/// the sun moves a sixteenth of a degree, which is a fifth of a metre at the
/// sky's distance, against a disc 15 m across.
const STEP: f32 = 0.25;

/// How high the sun stands at noon, 45°: at noon each shadow reaches out
/// about as far as its caster is tall, and the light is as Hajun chose it on
/// 2026-10-13, when the sun stood there all day.
const SUN_ELEVATION_DEG: f32 = 45.0;
/// How high it is at the start and end of the day, as the sunrise turns into
/// the day and the day into the sunset.
const SUN_LOW_DEG: f32 = 16.0;
/// How far under the horizon the sun goes at the end of the sunset, and
/// comes up from at the start of the sunrise.
const SUN_DOWN_DEG: f32 = -8.0;
/// Where the sun is at noon, measured from +X towards +Z. From where the
/// camera starts, behind the player at +Z looking towards -Z, the light comes
/// over its right shoulder.
const SUN_AZIMUTH_DEG: f32 = 40.0;
/// How far round the sky the sun goes in the day, from the end of the
/// sunrise to the start of the sunset, and how much more in each of those.
const DAY_SWEEP_DEG: f32 = 150.0;
const TWILIGHT_SWEEP_DEG: f32 = 12.0;
/// How high the moon gets at midnight, and where it is then: across the sky
/// from where the sun is at noon. It rises as the night begins and is down as
/// it ends, under the horizon by this much.
const MOON_ELEVATION_DEG: f32 = 40.0;
const MOON_AZIMUTH_DEG: f32 = SUN_AZIMUTH_DEG + 180.0;
const MOON_DOWN_DEG: f32 = -6.0;

/// Whether the sun casts shadows. Off since 2026-10-03, on trial: Hajun asked
/// to see how much cooler the phone runs without the two shadow maps, which
/// are the whole shadow-casting world drawn twice more every frame. With it
/// off Bevy makes no shadow maps at all.
const SUN_SHADOWS: bool = false;
/// 7,000 lux until 2026-10-13, when the fill below was turned down: Hajun
/// found the whole world a greyish blue, "like a privacy screen filter".
const SUN_ILLUMINANCE: f32 = 7_800.0;
const SUN_COLOR: [f32; 3] = [1.0, 0.97, 0.90];
/// The sun's light low in the sky, at sunrise and sunset.
const SUN_LOW_COLOR: [f32; 3] = [1.0, 0.42, 0.16];
/// Skylight from above and a little away from the sun, so the side of anything
/// facing away from it is a cool blue rather than black. Casts no shadows.
/// It was 3,600 lux until 2026-10-13: at half the sun, its blue cancelled the
/// sun's warmth on everything facing up, and the paving, the roads and the
/// roofs all came out a cool blue-grey. Now what faces up is lit a little
/// warm of white, and only what faces away from the sun is cool.
const FILL_ILLUMINANCE: f32 = 2_000.0;
const FILL_COLOR: [f32; 3] = [0.80, 0.90, 1.0];
/// The fill as the sun goes down, warm, and at night, a moonlit blue.
const FILL_LOW_COLOR: [f32; 3] = [1.0, 0.74, 0.55];
const FILL_NIGHT_COLOR: [f32; 3] = [0.42, 0.52, 0.80];
/// 340 until 2026-10-13: raised with the fill turned down, so that what
/// faces away from the sun is no darker than it was.
const AMBIENT_BRIGHTNESS: f32 = 445.0;
/// The night's light, the moon's, the fill's and the ambient light, in a
/// blue: on the ground about a twentieth of the noon sky's. Dark enough that
/// the street lamps light the town, and light enough that an island without
/// any, like the home or a plot, can still be found your way round.
const MOON_ILLUMINANCE: f32 = 320.0;
const MOON_COLOR: [f32; 3] = [0.62, 0.72, 1.0];
const NIGHT_FILL_ILLUMINANCE: f32 = 130.0;
const NIGHT_AMBIENT: f32 = 55.0;
const NIGHT_AMBIENT_COLOR: [f32; 3] = [0.32, 0.42, 0.78];

const SKY_DISTANCE: f32 = 170.0;
const LIGHT_DISTANCE: f32 = 48.0;
const SUN_DISC_RADIUS: f32 = 7.5;
const MOON_DISC_RADIUS: f32 = 5.2;
const CLOUD_COUNT: u32 = 24;
const STAR_COUNT: u32 = 160;
const STAR_DISTANCE: f32 = 185.0;

/// A clear light blue: a little brighter than the (0.53, 0.80, 0.92) it was
/// until 2026-10-13, which looked greyish, and no bluer. A deeper blue,
/// (0.40, 0.68, 1.0), was tried that day, and Hajun found it too blue.
const SKY: [f32; 3] = [0.55, 0.82, 0.97];
/// The sky through the sunset, the night and the sunrise. As the cycle had
/// them before it was taken out (2026-09-23).
const SKY_GOLDEN: [f32; 3] = [0.96, 0.55, 0.22];
const SKY_PINK: [f32; 3] = [0.95, 0.42, 0.36];
const SKY_DUSK: [f32; 3] = [0.62, 0.18, 0.10];
const SKY_NIGHT: [f32; 3] = [0.015, 0.02, 0.055];
const SKY_MOONLIT: [f32; 3] = [0.04, 0.055, 0.12];
const SKY_PREDAWN: [f32; 3] = [0.20, 0.14, 0.22];
const SKY_DAWN: [f32; 3] = [0.98, 0.52, 0.36];
const SUN_DISC: [f32; 3] = [1.0, 0.93, 0.52];
const SUN_DISC_LOW: [f32; 3] = [1.0, 0.38, 0.08];
const MOON_DISC: [f32; 3] = [0.82, 0.85, 0.94];
const CLOUD: [f32; 3] = [0.96, 0.97, 1.0];
const CLOUD_GOLDEN: [f32; 3] = [1.0, 0.58, 0.28];
const CLOUD_PINK: [f32; 3] = [1.0, 0.46, 0.38];
const CLOUD_DUSK: [f32; 3] = [0.88, 0.32, 0.18];
const CLOUD_NIGHT: [f32; 3] = [0.16, 0.18, 0.28];
const CLOUD_DAWN: [f32; 3] = [1.0, 0.62, 0.48];
const CLOUD_ALPHA: f32 = 0.84;
const CLOUD_NIGHT_ALPHA: f32 = 0.42;
/// How many times brighter than their colours the sun's disc, the moon's and
/// the clouds are drawn. Unlit, they still go through the camera's
/// tonemapping, which shows white as a light grey: until 2026-10-13 the
/// clouds were grey and the sun a dull mustard yellow. At five times they
/// come out white and a pale yellow-white. (`ClearColor`, the sky itself, is
/// not tonemapped, and is shown as it is.)
const UNLIT_GAIN: f32 = 5.0;
/// The stars, likewise, which are small and added to the sky behind them.
const STAR_GAIN: f32 = 3.0;
/// How brightly something unlit that stands in the world, the fountain's
/// running water, is drawn at night, as a share of how it is at noon: as if
/// lit by the street lamps round the fountain, about a sixth of the noon
/// sky's light on it, and a little brighter for being water. By day, as much
/// of it as the sky lights the ground with, against noon.
const UNLIT_AT_NIGHT: f32 = 0.3;

/// Wide enough that its edge is never in sight from anywhere a player can get
/// to, with the camera pulled all the way back.
const SEA_SIZE: f32 = 1200.0;
/// A clear sea blue, blended over whatever is below it, which is mostly the
/// sky. It was (0.08, 0.30, 0.46) at 0.58 until 2026-10-13, the colour the
/// sea had before it was taken out and checked on the phone; so much of the
/// pale sky came through it that it showed a steel grey. It is lit, so it
/// darkens with the light at night by itself, and the lamps by the shore
/// shine in it.
const SEA: [f32; 3] = [0.10, 0.40, 0.56];
const SEA_ALPHA: f32 = 0.70;
const SEA_BOB_AMP: f32 = 0.035;
const SEA_BOB_SPEED: f32 = 0.65;

/// What time of day it is, for everything else that changes with it: the
/// street lamps (`lamps`) and the fountain's water (`fountain`). Changes
/// only when the sky does, [`STEP`] seconds at a time.
#[derive(Resource, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Daylight {
    /// How far into the day and night it is, in seconds, from the start of
    /// the day.
    pub time: f32,
    /// How far on the street lamps are: 0 all day, 1 all night, coming on as
    /// the sun goes down and going off as it comes up.
    pub lamps: f32,
    /// How brightly something unlit that stands in the world is drawn, as a
    /// share of how it is at noon: as brightly as the sky lights the ground,
    /// and never less than [`UNLIT_AT_NIGHT`], which it is all night. In
    /// steps of a sixty-fourth, so that it changes only now and then.
    pub unlit: f32,
}

impl Default for Daylight {
    fn default() -> Self {
        Self::at(0.0)
    }
}

impl Daylight {
    fn at(time: f32) -> Self {
        let state = sky_at(time);
        Self {
            time,
            lamps: state.lamps,
            unlit: state.unlit,
        }
    }
}

/// Where the day is kept: by the town's clock, or stopped at one time.
#[derive(Resource, Clone, Copy, PartialEq, Debug, Default)]
pub(crate) struct Clock {
    /// The time of day it stays at, if `RT_TIME_OF_DAY` says one: seconds
    /// into the day, or `day`, `sunset`, `night` or `sunrise`, the middle
    /// of each.
    pinned: Option<f32>,
}

impl Clock {
    fn from_env() -> Self {
        Self {
            pinned: std::env::var("RT_TIME_OF_DAY").ok().and_then(|said| pinned(&said)),
        }
    }

    /// How far into the day and night it is now, in seconds, `town` being the
    /// town's clock.
    fn now(&self, town: &TownClock) -> f32 {
        self.pinned.unwrap_or_else(|| time_of_day(town.now()))
    }
}

/// How far into the day and night it is at `since` seconds since 1970: the
/// same on every device in the town, whose clock is the server's.
fn time_of_day(since: f64) -> f32 {
    (since.rem_euclid(f64::from(CYCLE_SECS))) as f32
}

/// The time of day `said` names, as `RT_TIME_OF_DAY` has it.
fn pinned(said: &str) -> Option<f32> {
    match said.trim() {
        "day" | "noon" => Some(DAY_SECS * 0.5),
        "sunset" => Some(DAY_SECS + SUNSET_SECS * 0.5),
        "night" | "midnight" => Some(SUNSET_END + NIGHT_SECS * 0.5),
        "sunrise" => Some(NIGHT_END + SUNRISE_SECS * 0.5),
        number => number
            .parse::<f32>()
            .ok()
            .filter(|time| time.is_finite())
            .map(|time| time.rem_euclid(CYCLE_SECS)),
    }
}

#[derive(Component)]
struct CloudCluster {
    azimuth: f32,
    elevation: f32,
    distance: f32,
    speed: f32,
}

/// The sun's light by day and the moon's by night.
#[derive(Component)]
struct KeyLight;

/// The light from the rest of the sky.
#[derive(Component)]
struct FillLight;

#[derive(Component)]
struct SunDisc;

#[derive(Component)]
struct MoonDisc;

#[derive(Component)]
struct Stars;

#[derive(Component, Clone)]
struct Sea;

/// The sky's own materials, which change with the time of day, and the sky
/// as it was last set.
#[derive(Resource)]
struct Sky {
    sun: Handle<StandardMaterial>,
    sun_glow: [Handle<StandardMaterial>; 2],
    cloud: Handle<StandardMaterial>,
    stars: Handle<StandardMaterial>,
    last: SkyState,
}

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

/// The town's clock (`clock`) has to be there first: the sky starts at the
/// time of day it says.
pub(crate) fn plugin(app: &mut App) {
    let clock = Clock::from_env();
    let state = sky_at(clock.now(app.world().resource::<TownClock>()));
    app.insert_resource(ClearColor(srgb(state.sky)))
        .insert_resource(clock)
        .insert_resource(Daylight::at(state.time))
        .init_resource::<Water>()
        .add_systems(Startup, setup_sky)
        .add_systems(
            Update,
            (pass_the_day.in_set(DaySystems), drift_clouds, bob_sea),
        );
}

/// Where the time of day moves on ([`Daylight`]): whatever follows it runs
/// after this.
#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct DaySystems;

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
    clock: Res<Clock>,
    town: Res<TownClock>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let state = sky_at(clock.now(&town));
    ambient.color = srgb(state.ambient_color);
    ambient.brightness = state.ambient;

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
        KeyLight,
        DirectionalLight {
            illuminance: state.key,
            color: srgb(state.key_color),
            shadow_maps_enabled: SUN_SHADOWS,
            ..default()
        },
        light_transform(state.key_dir),
        sun_cascades.build(),
    ));

    commands.spawn((
        FillLight,
        DirectionalLight {
            illuminance: state.fill,
            color: srgb(state.fill_color),
            ..default()
        },
        light_transform(state.fill_dir),
    ));

    let sun = materials.add(sun_core_material(state.sun_disc));
    let sun_glow = [
        materials.add(glow_material(state.sun_disc, 0.12, 0.9)),
        materials.add(glow_material(state.sun_disc, 0.05, 0.35)),
    ];
    commands
        .spawn((
            SunDisc,
            Mesh3d(meshes.add(ico(SUN_DISC_RADIUS, 1))),
            MeshMaterial3d(sun.clone()),
            Transform::from_translation(state.sun_dir * SKY_DISTANCE),
            shown(state.sun_up),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .with_children(|parent| {
            for (radius, glow) in [(1.55, &sun_glow[0]), (2.35, &sun_glow[1])] {
                parent.spawn((
                    Mesh3d(meshes.add(ico(SUN_DISC_RADIUS * radius, 1))),
                    MeshMaterial3d(glow.clone()),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
            }
        });

    commands
        .spawn((
            MoonDisc,
            Mesh3d(meshes.add(Sphere::new(MOON_DISC_RADIUS).mesh().uv(36, 20))),
            MeshMaterial3d(materials.add(moon_core_material())),
            Transform::from_translation(state.moon_dir * SKY_DISTANCE),
            shown(state.moon_up),
            NotShadowCaster,
            NotShadowReceiver,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(meshes.add(Sphere::new(MOON_DISC_RADIUS * 1.28).mesh().uv(28, 16))),
                MeshMaterial3d(materials.add(glow_material(MOON_DISC, 0.1, 0.55))),
                NotShadowCaster,
                NotShadowReceiver,
            ));
        });

    let stars = materials.add(star_material(state.stars));
    commands.spawn((
        Stars,
        Mesh3d(meshes.add(star_field())),
        MeshMaterial3d(stars.clone()),
        Transform::default(),
        shown(state.stars > 0.0),
        NotShadowCaster,
        NotShadowReceiver,
    ));

    let puff_mesh = meshes.add(ico(8.0, 1));
    let cloud = materials.add(cloud_material(state.cloud, state.cloud_alpha));
    spawn_clouds(&mut commands, puff_mesh, cloud.clone());

    commands.insert_resource(Sky {
        sun,
        sun_glow,
        cloud,
        stars,
        last: state,
    });
}

/// Moves the sky on to the time of day it is now, [`STEP`] seconds at a
/// time, setting only what has changed since the last step.
#[allow(clippy::too_many_arguments)]
fn pass_the_day(
    clock: Res<Clock>,
    town: Res<TownClock>,
    sky: Option<ResMut<Sky>>,
    mut daylight: ResMut<Daylight>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut keys: Query<(&mut DirectionalLight, &mut Transform), (With<KeyLight>, Without<FillLight>)>,
    mut fills: Query<(&mut DirectionalLight, &mut Transform), (With<FillLight>, Without<KeyLight>)>,
    mut discs: Query<
        (&mut Transform, &mut Visibility, Has<SunDisc>),
        (Or<(With<SunDisc>, With<MoonDisc>)>, Without<KeyLight>, Without<FillLight>),
    >,
    mut stars: Query<&mut Visibility, (With<Stars>, Without<SunDisc>, Without<MoonDisc>)>,
) {
    let Some(mut sky) = sky else {
        return;
    };
    let time = (clock.now(&town) / STEP).floor() * STEP;
    if time == sky.last.time {
        return;
    }
    let now = sky_at(time);
    let was = sky.last;

    if now.sky != was.sky {
        clear.0 = srgb(now.sky);
    }
    if now.ambient != was.ambient || now.ambient_color != was.ambient_color {
        ambient.brightness = now.ambient;
        ambient.color = srgb(now.ambient_color);
    }
    for (mut light, mut place) in &mut keys {
        if now.key != was.key || now.key_color != was.key_color {
            light.illuminance = now.key;
            light.color = srgb(now.key_color);
        }
        if now.key_dir != was.key_dir {
            *place = light_transform(now.key_dir);
        }
    }
    for (mut light, mut place) in &mut fills {
        if now.fill != was.fill || now.fill_color != was.fill_color {
            light.illuminance = now.fill;
            light.color = srgb(now.fill_color);
        }
        if now.fill_dir != was.fill_dir {
            *place = light_transform(now.fill_dir);
        }
    }
    for (mut place, mut visibility, sun) in &mut discs {
        let (dir, up) = if sun {
            (now.sun_dir, now.sun_up)
        } else {
            (now.moon_dir, now.moon_up)
        };
        visibility.set_if_neq(shown(up));
        if up {
            place.set_if_neq(Transform::from_translation(dir * SKY_DISTANCE));
        }
    }
    for mut visibility in &mut stars {
        visibility.set_if_neq(shown(now.stars > 0.0));
    }
    if now.sun_disc != was.sun_disc {
        if let Some(mut material) = materials.get_mut(&sky.sun) {
            material.base_color = bright(now.sun_disc, 1.0);
        }
        for (handle, (alpha, glow)) in sky.sun_glow.iter().zip([(0.12, 0.9), (0.05, 0.35)]) {
            if let Some(mut material) = materials.get_mut(handle) {
                *material = glow_material(now.sun_disc, alpha, glow);
            }
        }
    }
    if now.cloud != was.cloud || now.cloud_alpha != was.cloud_alpha {
        if let Some(mut material) = materials.get_mut(&sky.cloud) {
            material.base_color = bright(now.cloud, now.cloud_alpha);
        }
    }
    if now.stars != was.stars && now.stars > 0.0 {
        if let Some(mut material) = materials.get_mut(&sky.stars) {
            material.base_color = star_color(now.stars);
        }
    }
    daylight.set_if_neq(Daylight {
        time,
        lamps: now.lamps,
        unlit: now.unlit,
    });
    sky.last = now;
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

/// Everything about the sky at one time of day.
#[derive(Clone, Copy, PartialEq, Debug)]
struct SkyState {
    time: f32,
    /// The way to the sun and to the moon, and whether each is up.
    sun_dir: Vec3,
    moon_dir: Vec3,
    sun_up: bool,
    moon_up: bool,
    /// The sun's light by day, the moon's by night: how strong, what colour,
    /// and the way to it.
    key: f32,
    key_color: [f32; 3],
    key_dir: Vec3,
    fill: f32,
    fill_color: [f32; 3],
    fill_dir: Vec3,
    ambient: f32,
    ambient_color: [f32; 3],
    /// `ClearColor`, in sRGB.
    sky: [f32; 3],
    sun_disc: [f32; 3],
    cloud: [f32; 3],
    cloud_alpha: f32,
    /// How bright the stars are, from 0, out, to 1.
    stars: f32,
    lamps: f32,
    unlit: f32,
}

/// The sky `time` seconds into the day and night.
fn sky_at(time: f32) -> SkyState {
    let t = time.rem_euclid(CYCLE_SECS);
    let (d, s, n, r) = (DAY_SECS, SUNSET_SECS, NIGHT_SECS, SUNRISE_SECS);
    let sun_dir = sun_way(t);
    let moon_dir = moon_way(t);
    let (sun_height, moon_height) = (sun_dir.y, moon_dir.y);
    // How far the sun's light is up, and how warm, from white overhead to an
    // orange low down.
    let sunlit = smoothstep(0.0, 0.35, sun_height);
    let warmth = 1.0 - smoothstep(0.12, 0.48, sun_height);
    let moonlit = smoothstep(0.0, 0.25, moon_height);
    let (key, key_color, key_dir) = if sun_height > 0.0 || moon_height <= 0.0 {
        (
            SUN_ILLUMINANCE * sunlit,
            mix3(SUN_COLOR, SUN_LOW_COLOR, warmth),
            sun_dir,
        )
    } else {
        (MOON_ILLUMINANCE * moonlit, MOON_COLOR, moon_dir)
    };
    let fill_dir = fill_way(key_dir);
    let (fill, fill_color) = if sun_height > 0.0 || moon_height <= 0.0 {
        (
            lerp(NIGHT_FILL_ILLUMINANCE, FILL_ILLUMINANCE, sunlit),
            mix3(FILL_NIGHT_COLOR, mix3(FILL_COLOR, FILL_LOW_COLOR, warmth), sunlit),
        )
    } else {
        (NIGHT_FILL_ILLUMINANCE, FILL_NIGHT_COLOR)
    };

    let sky = sample_rgb(
        t,
        &[
            (0.0, SKY),
            (d * 0.88, SKY),
            (d, mix3(SKY, SKY_GOLDEN, 0.45)),
            (d + s * 0.3, SKY_GOLDEN),
            (d + s * 0.55, SKY_PINK),
            (d + s * 0.8, SKY_DUSK),
            (SUNSET_END, SKY_NIGHT),
            (SUNSET_END + n * 0.45, SKY_MOONLIT),
            (NIGHT_END, SKY_NIGHT),
            (NIGHT_END + r * 0.2, SKY_PREDAWN),
            (NIGHT_END + r * 0.45, SKY_DAWN),
            (NIGHT_END + r * 0.75, mix3(SKY, SKY_DAWN, 0.4)),
            (CYCLE_SECS, SKY),
        ],
    );
    let cloud = sample_rgb(
        t,
        &[
            (0.0, CLOUD),
            (d * 0.88, CLOUD),
            (d, mix3(CLOUD, CLOUD_GOLDEN, 0.45)),
            (d + s * 0.3, CLOUD_GOLDEN),
            (d + s * 0.55, CLOUD_PINK),
            (d + s * 0.8, CLOUD_DUSK),
            (SUNSET_END, CLOUD_NIGHT),
            (NIGHT_END, CLOUD_NIGHT),
            (NIGHT_END + r * 0.45, CLOUD_DAWN),
            (NIGHT_END + r * 0.75, mix3(CLOUD, CLOUD_DAWN, 0.4)),
            (CYCLE_SECS, CLOUD),
        ],
    );
    let cloud_alpha = sample(
        t,
        &[
            (0.0, CLOUD_ALPHA),
            (d, CLOUD_ALPHA),
            (SUNSET_END, CLOUD_NIGHT_ALPHA),
            (NIGHT_END, CLOUD_NIGHT_ALPHA),
            (CYCLE_SECS, CLOUD_ALPHA),
        ],
    );
    let ambient = sample(
        t,
        &[
            (0.0, AMBIENT_BRIGHTNESS),
            (d * 0.88, AMBIENT_BRIGHTNESS),
            (d, 320.0),
            (d + s * 0.6, 170.0),
            (SUNSET_END, NIGHT_AMBIENT),
            (NIGHT_END, NIGHT_AMBIENT),
            (NIGHT_END + r * 0.5, 200.0),
            (CYCLE_SECS, AMBIENT_BRIGHTNESS),
        ],
    );
    let ambient_color = sample_rgb(
        t,
        &[
            (0.0, [1.0, 1.0, 1.0]),
            (d * 0.88, [1.0, 1.0, 1.0]),
            (d, [1.0, 0.82, 0.64]),
            (d + s * 0.5, [1.0, 0.52, 0.36]),
            (SUNSET_END, NIGHT_AMBIENT_COLOR),
            (NIGHT_END, NIGHT_AMBIENT_COLOR),
            (NIGHT_END + r * 0.5, [0.85, 0.62, 0.52]),
            (CYCLE_SECS, [1.0, 1.0, 1.0]),
        ],
    );
    let stars = sample(
        t,
        &[
            (0.0, 0.0),
            (d + s * 0.6, 0.0),
            (SUNSET_END, 1.0),
            (NIGHT_END, 1.0),
            (NIGHT_END + r * 0.45, 0.0),
            (CYCLE_SECS, 0.0),
        ],
    );

    SkyState {
        time,
        sun_dir,
        moon_dir,
        sun_up: sun_height > -0.02,
        moon_up: moon_height > -0.02,
        key,
        key_color,
        key_dir,
        fill,
        fill_color,
        fill_dir,
        ambient,
        ambient_color,
        sky,
        sun_disc: mix3(SUN_DISC, SUN_DISC_LOW, warmth),
        cloud,
        cloud_alpha,
        stars,
        // On as the sun goes down from 11° up to the horizon, as the light
        // goes, and off again as it comes back up through them.
        lamps: 1.0 - smoothstep(0.0, 0.2, sun_height),
        unlit: ((on_the_ground(key, key_dir, fill, fill_dir, ambient) / noon_on_the_ground()
            * 64.0)
            .round()
            / 64.0)
            .clamp(UNLIT_AT_NIGHT, 1.0),
    }
}

/// The way to the fill light, with the key light, the sun or the moon, the
/// way `key` is: from above, and a little away from it.
fn fill_way(key: Vec3) -> Vec3 {
    Vec3::new(-key.x * 0.45, 0.9, -key.z * 0.45).normalize()
}

/// How much light the sky throws on the ground, in lux: the key light's and
/// the fill's, as high as each is, and the ambient light's, a brightness,
/// which lights what faces it as π times as many lux would.
fn on_the_ground(key: f32, key_dir: Vec3, fill: f32, fill_dir: Vec3, ambient: f32) -> f32 {
    key * key_dir.y.max(0.0) + fill * fill_dir.y.max(0.0) + ambient * PI
}

/// As much at noon.
fn noon_on_the_ground() -> f32 {
    let sun = sun_way(DAY_SECS * 0.5);
    on_the_ground(SUN_ILLUMINANCE, sun, FILL_ILLUMINANCE, fill_way(sun), AMBIENT_BRIGHTNESS)
}

/// The way to the sun `t` seconds into the day and night: up at
/// [`SUN_LOW_DEG`] as the day begins, climbing quickly to its noon and
/// staying near it ([`plateau`]), down to the same height as the day ends,
/// under the horizon through the sunset, and up through it again in the
/// sunrise, on the other side. Through the night it waits under the horizon
/// where it went down.
fn sun_way(t: f32) -> Vec3 {
    let half = 0.5;
    let twilight = TWILIGHT_SWEEP_DEG / DAY_SWEEP_DEG;
    let (elevation, along) = if t < DAY_SECS {
        let s = t / DAY_SECS;
        (lerp(SUN_LOW_DEG, SUN_ELEVATION_DEG, plateau(s)), s - half)
    } else if t < SUNSET_END {
        let u = smoothstep(DAY_SECS, SUNSET_END, t);
        (lerp(SUN_LOW_DEG, SUN_DOWN_DEG, u), half + twilight * u)
    } else if t < NIGHT_END {
        (SUN_DOWN_DEG, half + twilight)
    } else {
        let u = smoothstep(NIGHT_END, CYCLE_SECS, t);
        (lerp(SUN_DOWN_DEG, SUN_LOW_DEG, u), -half - twilight * (1.0 - u))
    };
    dir_from_elev_azim(
        elevation.to_radians(),
        (SUN_AZIMUTH_DEG + along * DAY_SWEEP_DEG).to_radians(),
    )
}

/// The way to the moon `t` seconds into the day and night: up through the
/// night, across the sky from the sun's noon, and under the horizon the rest
/// of the time.
fn moon_way(t: f32) -> Vec3 {
    let (elevation, along) = if (SUNSET_END..NIGHT_END).contains(&t) {
        let n = (t - SUNSET_END) / NIGHT_SECS;
        (lerp(MOON_DOWN_DEG, MOON_ELEVATION_DEG, plateau(n)), n - 0.5)
    } else {
        (MOON_DOWN_DEG, 0.5)
    };
    dir_from_elev_azim(
        elevation.to_radians(),
        (MOON_AZIMUTH_DEG + along * DAY_SWEEP_DEG).to_radians(),
    )
}

/// How far up from its lowest to its highest the sun is, `s` of the way
/// through the day, or the moon through the night: 0 at either end and 1 in
/// the middle, climbing quickly and staying near the top. The sun is over
/// 30° from a tenth of the way into the day to nine tenths of it, and over
/// 40° for the middle three fifths. It climbs no faster at the start than it
/// went on climbing through the sunrise.
fn plateau(s: f32) -> f32 {
    let arc = (PI * s).sin().clamp(0.0, 1.0);
    1.0 - (1.0 - arc) * (1.0 - arc)
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

fn shown(up: bool) -> Visibility {
    if up {
        Visibility::Inherited
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

/// Every star, in one mesh, so that they are one draw: small, scattered over
/// the sky above the horizon, a few bigger than the rest.
fn star_field() -> Mesh {
    let star = ico(1.0, 0);
    let (Some(bevy::mesh::VertexAttributeValues::Float32x3(corners)), Some(indices)) =
        (star.attribute(Mesh::ATTRIBUTE_POSITION), star.indices())
    else {
        unreachable!("an icosphere has positions and indices");
    };
    let mut positions = Vec::with_capacity(corners.len() * STAR_COUNT as usize);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut order = Vec::with_capacity(indices.len() * STAR_COUNT as usize);
    for i in 0..STAR_COUNT {
        let azimuth = hash01(i, 1) * std::f32::consts::TAU;
        let y = lerp(0.08, 0.98, hash01(i, 2));
        let across = (1.0 - y * y).max(0.0).sqrt();
        let dir = Vec3::new(across * azimuth.cos(), y, across * azimuth.sin());
        let size = lerp(0.09, 0.36, hash01(i, 3).powf(1.8));
        let first = positions.len() as u32;
        for &corner in corners {
            let corner = Vec3::from(corner);
            positions.push((dir * STAR_DISTANCE + corner * size).to_array());
            normals.push(corner.to_array());
        }
        order.extend(indices.iter().map(|index| first + index as u32));
    }
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(bevy::mesh::Indices::U32(order))
}

fn sun_core_material(rgb: [f32; 3]) -> StandardMaterial {
    StandardMaterial {
        base_color: bright(rgb, 1.0),
        unlit: true,
        fog_enabled: false,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    }
}

/// A glow round the sun or the moon, added to the sky behind it.
fn glow_material(rgb: [f32; 3], alpha: f32, glow: f32) -> StandardMaterial {
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

fn moon_core_material() -> StandardMaterial {
    StandardMaterial {
        base_color: bright(MOON_DISC, 1.0),
        unlit: true,
        fog_enabled: false,
        perceptual_roughness: 0.95,
        reflectance: 0.0,
        ..default()
    }
}

fn star_material(opacity: f32) -> StandardMaterial {
    StandardMaterial {
        unlit: true,
        double_sided: true,
        cull_mode: None,
        reflectance: 0.0,
        fog_enabled: false,
        base_color: star_color(opacity),
        alpha_mode: AlphaMode::Add,
        ..default()
    }
}

/// The stars, as bright as `opacity` of them, added to the sky.
fn star_color(opacity: f32) -> Color {
    let linear = srgb([0.92, 0.95, 1.0]).to_linear();
    Color::linear_rgba(
        linear.red * STAR_GAIN,
        linear.green * STAR_GAIN,
        linear.blue * STAR_GAIN,
        opacity.clamp(0.0, 1.0),
    )
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

fn cloud_material(rgb: [f32; 3], alpha: f32) -> StandardMaterial {
    StandardMaterial {
        unlit: true,
        double_sided: true,
        cull_mode: None,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        fog_enabled: false,
        base_color: bright(rgb, alpha),
        alpha_mode: AlphaMode::Blend,
        ..default()
    }
}

/// `rgb`, as much as `alpha` of it, as bright as something unlit is drawn so
/// that it comes out that colour on screen ([`UNLIT_GAIN`]).
fn bright(rgb: [f32; 3], alpha: f32) -> Color {
    let linear = srgb(rgb).to_linear();
    Color::linear_rgba(
        linear.red * UNLIT_GAIN,
        linear.green * UNLIT_GAIN,
        linear.blue * UNLIT_GAIN,
        alpha,
    )
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)]
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The colour `keys` give at `time`, eased from one key to the next.
fn sample_rgb(time: f32, keys: &[(f32, [f32; 3])]) -> [f32; 3] {
    for pair in keys.windows(2) {
        let ((t0, a), (t1, b)) = (pair[0], pair[1]);
        if time <= t1 {
            return mix3(a, b, smoothstep(t0, t1, time));
        }
    }
    keys.last().map_or([0.0; 3], |key| key.1)
}

/// The number `keys` give at `time`, eased from one key to the next.
fn sample(time: f32, keys: &[(f32, f32)]) -> f32 {
    for pair in keys.windows(2) {
        let ((t0, a), (t1, b)) = (pair[0], pair[1]);
        if time <= t1 {
            return lerp(a, b, smoothstep(t0, t1, time));
        }
    }
    keys.last().map_or(0.0, |key| key.1)
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

#[cfg(test)]
mod tests {
    use super::*;

    const NOON: f32 = DAY_SECS * 0.5;
    const MIDNIGHT: f32 = SUNSET_END + NIGHT_SECS * 0.5;

    /// Every step of a whole day and night.
    fn whole_day() -> impl Iterator<Item = f32> {
        (0..(CYCLE_SECS / STEP) as u32).map(|step| step as f32 * STEP)
    }

    #[test]
    fn noon_is_the_town_as_hajun_chose_its_light() {
        let noon = sky_at(NOON);
        let approved = dir_from_elev_azim(
            SUN_ELEVATION_DEG.to_radians(),
            SUN_AZIMUTH_DEG.to_radians(),
        );
        assert!(noon.sun_dir.distance(approved) < 1e-5, "{}", noon.sun_dir);
        assert_eq!(noon.key_dir, noon.sun_dir);
        assert_eq!(noon.key, SUN_ILLUMINANCE);
        assert_eq!(noon.key_color, SUN_COLOR);
        assert_eq!(noon.fill, FILL_ILLUMINANCE);
        assert_eq!(noon.fill_color, FILL_COLOR);
        let fill = Vec3::new(-approved.x * 0.45, 0.9, -approved.z * 0.45).normalize();
        assert!(noon.fill_dir.distance(fill) < 1e-5);
        assert_eq!(noon.ambient, AMBIENT_BRIGHTNESS);
        assert_eq!(noon.ambient_color, [1.0, 1.0, 1.0]);
        assert_eq!(noon.sky, SKY);
        assert_eq!(noon.cloud, CLOUD);
        assert_eq!(noon.cloud_alpha, CLOUD_ALPHA);
        assert_eq!(noon.sun_disc, SUN_DISC);
        assert_eq!((noon.stars, noon.lamps, noon.unlit), (0.0, 0.0, 1.0));
        assert!(noon.sun_up && !noon.moon_up);
    }

    #[test]
    fn most_of_the_day_the_sun_is_high_and_the_light_unchanged() {
        for step in 0..=100 {
            let t = DAY_SECS * (0.1 + 0.78 * step as f32 / 100.0);
            let sky = sky_at(t);
            let elevation = sky.sun_dir.y.asin().to_degrees();
            assert!(elevation > 30.0, "{elevation} at {t}");
            assert_eq!(sky.sky, SKY, "at {t}");
            assert_eq!(sky.lamps, 0.0, "at {t}");
        }
    }

    #[test]
    fn the_sun_is_up_by_day_and_the_moon_by_night() {
        for t in whole_day() {
            let sky = sky_at(t);
            if t < DAY_SECS {
                assert!(sky.sun_dir.y > 0.25 && sky.moon_dir.y < 0.0, "at {t}");
            }
            if (SUNSET_END..NIGHT_END).contains(&t) {
                assert!(sky.sun_dir.y < 0.0, "at {t}");
            }
            if (SUNSET_END + 20.0..NIGHT_END - 20.0).contains(&t) {
                assert!(sky.moon_dir.y > 0.0, "at {t}");
            }
            // Never both: the one light is the sun's or the moon's.
            assert!(sky.sun_dir.y <= 0.0 || sky.moon_dir.y <= 0.0, "at {t}");
        }
        let midnight = sky_at(MIDNIGHT);
        assert_eq!(midnight.key_dir, midnight.moon_dir);
        assert!(midnight.moon_up && !midnight.sun_up);
        assert!((midnight.moon_dir.y.asin().to_degrees() - MOON_ELEVATION_DEG).abs() < 1e-3);
    }

    #[test]
    fn the_lamps_are_off_all_day_and_on_all_night() {
        for t in whole_day() {
            let sky = sky_at(t);
            assert!((UNLIT_AT_NIGHT..=1.0).contains(&sky.unlit), "at {t}");
            if t < DAY_SECS {
                assert_eq!(sky.lamps, 0.0, "at {t}");
            }
            if (DAY_SECS * 0.3..DAY_SECS * 0.7).contains(&t) {
                assert!(sky.unlit > 0.9, "{} at {t}", sky.unlit);
            }
            if (SUNSET_END..NIGHT_END).contains(&t) {
                assert_eq!(sky.lamps, 1.0, "at {t}");
                assert_eq!(sky.unlit, UNLIT_AT_NIGHT, "at {t}");
                assert!(sky.stars > 0.99 || t < SUNSET_END + 1.0, "at {t}");
            }
        }
        // They come on while the sun is going down, before it is dark.
        let on = whole_day()
            .find(|&t| t > DAY_SECS && sky_at(t).lamps > 0.0)
            .unwrap();
        assert!(on < SUNSET_END && sky_at(on).sun_dir.y > 0.0, "{on}");
    }

    #[test]
    fn the_day_goes_by_without_a_jump() {
        // From one step to the next, nothing the eye can follow jumps: not
        // the light, the sky or the clouds, and not the way the light comes
        // from or its colour while there is any of it. (Where the sun's
        // light hands over to the moon's, both are out.)
        let mut was = sky_at(0.0);
        for t in whole_day().skip(1).chain([0.0]) {
            let now = sky_at(t);
            let close = |a: f32, b: f32, most: f32| (a - b).abs() <= most;
            assert!(close(now.key, was.key, 100.0), "key light {} to {} at {t}", was.key, now.key);
            assert!(close(now.fill, was.fill, 25.0), "fill at {t}");
            assert!(close(now.ambient, was.ambient, 3.0), "ambient at {t}");
            for k in 0..3 {
                assert!(close(now.sky[k], was.sky[k], 0.02), "sky at {t}");
                assert!(close(now.cloud[k], was.cloud[k], 0.02), "clouds at {t}");
                assert!(close(now.fill_color[k], was.fill_color[k], 0.02), "fill colour at {t}");
            }
            assert!(close(now.lamps, was.lamps, 0.05), "lamps at {t}");
            assert!(close(now.unlit, was.unlit, 0.04), "unlit at {t}");
            if now.key > 1.0 && was.key > 1.0 {
                assert!(now.key_dir.angle_between(was.key_dir) < 0.01, "light's way at {t}");
                for k in 0..3 {
                    assert!(close(now.key_color[k], was.key_color[k], 0.02), "key colour at {t}");
                }
            }
            was = now;
        }
    }

    #[test]
    fn the_time_of_day_is_the_clock_s() {
        assert_eq!(time_of_day(0.0), 0.0);
        assert_eq!(time_of_day(f64::from(CYCLE_SECS) * 1000.0 + 61.5), 61.5);
        // Today, by the clock: still well inside one day and night.
        let now = time_of_day(1_792_000_000.25);
        assert!((0.0..CYCLE_SECS).contains(&now));
    }

    #[test]
    fn rt_time_of_day_names_a_time() {
        assert_eq!(pinned("night"), Some(MIDNIGHT));
        assert_eq!(pinned("day"), Some(NOON));
        assert_eq!(pinned(" sunset "), Some(DAY_SECS + SUNSET_SECS * 0.5));
        assert_eq!(pinned("1250"), Some(50.0));
        assert_eq!(pinned("-10"), Some(CYCLE_SECS - 10.0));
        assert_eq!(pinned("tea time"), None);
        assert_eq!(pinned("NaN"), None);
    }

    #[test]
    fn the_stars_are_one_mesh_above_the_horizon() {
        let (field, star) = (star_field(), ico(1.0, 0));
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(corners)) =
            field.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("no positions");
        };
        assert_eq!(corners.len(), star.count_vertices() * STAR_COUNT as usize);
        assert_eq!(
            field.indices().map(|indices| indices.len()),
            star.indices().map(|indices| indices.len() * STAR_COUNT as usize)
        );
        assert!(corners.iter().all(|corner| corner[1] > 0.0));
    }
}
