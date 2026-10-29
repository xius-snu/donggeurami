//! Street lamps, lit from dusk to dawn (Hajun, 2026-10-16). Every bulb in a
//! model, a part whose material is [`BULB_MATERIAL`], glows through the
//! night, and every lamp, its bulbs taken together ([`ONE_LAMP`]), throws
//! light from between them: the town's 23, round the fountain and along
//! both roads, and the shop's Street Lamp, wherever a builder puts one down.
//! Nothing knows where a lamp is but its model: a bulb added in Blender is
//! a lamp, and moving one moves its light.
//!
//! The light is a point light under each lamp's arm ([`LAMP_LUMENS`],
//! [`LAMP_RANGE`]), bright enough that under it the night is nearly as light
//! as the day, and gone a stone's throw away, where the night's moonlight
//! (`sky`) is all there is. The lamps come on as the sun goes down and go
//! off as it comes up (`sky::Daylight::lamps`). By day there is no light at
//! all, so the day costs what it did before there were lamps; at night at
//! most [`MOST_LIT`] lamps throw light at once, the nearest the camera on
//! the island you are on. Lights cast no shadows: a shadow map for a point
//! light is the world drawn six times more.
//!
//! The lights are sorted into the camera's clusters on the CPU, on every
//! platform (`cluster_lights_on_the_cpu` in `lib.rs`).

use bevy::gltf::GltfMaterialName;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use crate::ThirdPersonCamera;
use crate::sky::{self, Daylight};

/// What a bulb's material is called in Blender: Hajun's street lamp's,
/// `lightbulb`, in the town's model and in the shop's.
pub(crate) const BULB_MATERIAL: &str = "lightbulb";
/// How far apart two bulbs can be and still be one lamp's, lit together from
/// between them, in metres. Hajun's lamp has two, either end of its arm, a
/// little over a metre apart; the lamps of the town stand at least 18 m
/// apart.
const ONE_LAMP: f32 = 1.5;
/// How much light a lamp gives, in lumens: under it, 6 m down, the ground is
/// lit about two thirds as brightly as by the noon sky; 8 m out, an eighth.
const LAMP_LUMENS: f32 = 2_000_000.0;
/// How far a lamp's light reaches, in metres, fading out before it: far
/// enough to light the ground between the lamps of the inner road, 19 m
/// apart, and no further, so that each spot is lit by a few lamps at most.
/// At 20 m the town at night looked the same, and on the laptop's GPU the
/// night cost half as much again as the day; at 16, a third (2026-10-16).
const LAMP_RANGE: f32 = 16.0;
/// A warm white, a little yellower than daylight.
const LAMP_COLOR: Color = Color::srgb(1.0, 0.87, 0.68);
/// How brightly a bulb glows when lit, in Bevy's units for emitted light: the
/// camera's exposure takes this down to four times its colour, which the
/// tonemapping shows as a bright, warm white.
const BULB_GLOW: f32 = 4_000.0;
/// The most lamps that throw light at once. More than the town has, so that
/// in the town every lamp does; it stops a plot with a field of lamps on it
/// costing as many lights over every spot as it has lamps. The rest still
/// glow.
const MOST_LIT: usize = 24;

/// A lamp's light.
#[derive(Component)]
struct Lamp;

/// A mesh of bulbs, which glows while the lamps are on.
#[derive(Component)]
struct Bulb;

pub(crate) fn plugin(app: &mut App) {
    app.add_observer(find_bulbs)
        .add_systems(Update, light_the_lamps.after(sky::DaySystems));
}

/// Finds every bulb in a model once it has spawned, and gives each lamp its
/// light, out until the lamps come on.
fn find_bulbs(
    ready: On<WorldInstanceReady>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, &GltfMaterialName)>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    mut commands: Commands,
) {
    for part in children.iter_descendants(ready.entity) {
        let Ok((mesh, material)) = parts.get(part) else {
            continue;
        };
        if !crate::island::is_material(&material.0, BULB_MATERIAL) {
            continue;
        }
        let (Some(mesh), Ok(place)) = (meshes.get(&mesh.0), placement.compute_global_transform(part))
        else {
            continue;
        };
        let Some(VertexAttributeValues::Float32x3(corners)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        // Told apart in the world, where a metre is a metre, and lit from
        // the mesh's own space, which the lights go with.
        let world: Vec<Vec3> = corners
            .iter()
            .map(|&corner| place.transform_point(Vec3::from(corner)))
            .collect();
        let into = place.affine().inverse();
        commands.entity(part).insert(Bulb);
        for middle in lamps_of(&world) {
            commands.spawn((
                ChildOf(part),
                Lamp,
                PointLight {
                    color: LAMP_COLOR,
                    intensity: 0.0,
                    range: LAMP_RANGE,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(into.transform_point3(middle)),
                Visibility::Hidden,
            ));
        }
    }
}

/// Where each lamp's light goes, among the corners of its bulbs, `bulbs`:
/// the middle of the box round each lot of bulbs no more than [`ONE_LAMP`]
/// apart.
fn lamps_of(bulbs: &[Vec3]) -> Vec<Vec3> {
    let mut boxes: Vec<(Vec3, Vec3)> = Vec::new();
    for &corner in bulbs {
        let near = boxes.iter_mut().find(|(low, high)| {
            corner.cmpge(*low - ONE_LAMP).all() && corner.cmple(*high + ONE_LAMP).all()
        });
        match near {
            Some((low, high)) => {
                *low = low.min(corner);
                *high = high.max(corner);
            }
            None => boxes.push((corner, corner)),
        }
    }
    // Two lots that grew to within reach of each other are one lamp's.
    let mut merged = true;
    while merged {
        merged = false;
        'all: for i in 0..boxes.len() {
            for j in i + 1..boxes.len() {
                let ((a, b), (c, d)) = (boxes[i], boxes[j]);
                if (a - ONE_LAMP).cmple(d).all() && (c - ONE_LAMP).cmple(b).all() {
                    boxes[i] = (a.min(c), b.max(d));
                    boxes.swap_remove(j);
                    merged = true;
                    break 'all;
                }
            }
        }
    }
    boxes.into_iter().map(|(low, high)| (low + high) * 0.5).collect()
}

/// Turns the lamps on and off with the time of day: each bulb's glow, and the
/// light of the nearest [`MOST_LIT`] lamps to the camera on the island you are
/// on. Only what has changed is written.
fn light_the_lamps(
    daylight: Res<Daylight>,
    cameras: Query<&GlobalTransform, With<ThirdPersonCamera>>,
    mut lamps: Query<(Entity, &ChildOf, &GlobalTransform, &mut PointLight, &mut Visibility), With<Lamp>>,
    drawn: Query<&InheritedVisibility>,
    bulbs: Query<&MeshMaterial3d<StandardMaterial>, With<Bulb>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut nearest: Local<Vec<(f32, Entity)>>,
) {
    let on = daylight.lamps;
    nearest.clear();
    if on > 0.0 {
        let eye = cameras.single().map(GlobalTransform::translation).unwrap_or_default();
        nearest.extend(
            lamps
                .iter()
                .filter(|(_, bulb, ..)| drawn.get(bulb.parent()).is_ok_and(|drawn| drawn.get()))
                .map(|(lamp, _, place, ..)| (place.translation().distance_squared(eye), lamp)),
        );
        nearest.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        nearest.truncate(MOST_LIT);
    }
    for (lamp, _, _, mut light, mut visibility) in &mut lamps {
        let lit = nearest.iter().any(|&(_, near)| near == lamp);
        visibility.set_if_neq(if lit {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let intensity = LAMP_LUMENS * on;
        if lit && light.intensity != intensity {
            light.intensity = intensity;
        }
    }
    // Every bulb of a model shares its material, so each is looked at once.
    let mut seen: Vec<AssetId<StandardMaterial>> = Vec::new();
    for material in &bulbs {
        let id = material.id();
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        let Some(glow) = materials.get(id).map(|bulb| glow(bulb, on)) else {
            continue;
        };
        if materials.get(id).is_some_and(|bulb| bulb.emissive != glow)
            && let Some(mut bulb) = materials.get_mut(id)
        {
            bulb.emissive = glow;
        }
    }
}

/// How a bulb of `material` glows with the lamps `on` that much: in its own
/// colour, out by day.
fn glow(material: &StandardMaterial, on: f32) -> LinearRgba {
    // Its alpha stays 1, which has the camera's exposure taken to it.
    let (own, strength) = (material.base_color.to_linear(), BULB_GLOW * on);
    LinearRgba::rgb(own.red * strength, own.green * strength, own.blue * strength)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hajun's lamp's two bulbs, as their meshes' boxes have them: either
    /// end of its arm, under its shades.
    fn bulbs_at(x: f32, z: f32) -> Vec<Vec3> {
        [-1.14, -0.60, 0.60, 1.14]
            .into_iter()
            .flat_map(|across| {
                [5.76, 6.04].map(move |up| Vec3::new(x + across, up, z + 0.27 * across.signum()))
            })
            .collect()
    }

    #[test]
    fn two_bulbs_of_a_lamp_are_one_light_between_them() {
        let lamps = lamps_of(&bulbs_at(0.0, 0.0));
        assert_eq!(lamps.len(), 1);
        assert!(lamps[0].distance(Vec3::new(0.0, 5.9, 0.0)) < 1e-4, "{}", lamps[0]);
    }

    #[test]
    fn lamps_apart_are_lights_of_their_own() {
        // The fountain's three lamps, 12 m out, and the inner road's, 19 m
        // apart along it: in any order of their corners.
        let mut corners: Vec<Vec3> = [(8.4, 8.4), (-11.4, 3.1), (3.1, -11.4), (22.0, 9.5), (8.8, 22.3)]
            .into_iter()
            .flat_map(|(x, z)| bulbs_at(x, z))
            .collect();
        corners.reverse();
        corners.rotate_left(7);
        let lamps = lamps_of(&corners);
        assert_eq!(lamps.len(), 5, "{lamps:?}");
        for (x, z) in [(8.4, 8.4), (-11.4, 3.1), (3.1, -11.4), (22.0, 9.5), (8.8, 22.3)] {
            assert!(
                lamps.iter().any(|lamp| lamp.distance(Vec3::new(x, 5.9, z)) < 1e-3),
                "no light at {x}, {z}: {lamps:?}"
            );
        }
    }

    #[test]
    fn a_bulb_glows_only_while_the_lamps_are_on() {
        let bulb = StandardMaterial {
            base_color: Color::linear_rgb(1.0, 0.74, 0.33),
            ..default()
        };
        assert_eq!(glow(&bulb, 0.0), LinearRgba::BLACK);
        let lit = glow(&bulb, 1.0);
        assert!((lit.red - BULB_GLOW).abs() < 1e-2 && (lit.blue - 0.33 * BULB_GLOW).abs() < 1e-2);
    }
}
