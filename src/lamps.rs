//! Street lamps, lit from dusk to dawn (Hajun, 2026-10-16). Every bulb in a
//! model, a part whose material is [`BULB_MATERIAL`], glows through the
//! night, and every lamp, its bulbs taken together ([`ONE_LAMP`]), throws
//! light from between them: the town's 12, round the ring round its pond,
//! and the shop's Street Lamp, wherever a builder puts one down. Nothing
//! knows where a lamp is but its model: a bulb added in Blender is a lamp,
//! and moving one moves its light.
//!
//! The light is a point light under each lamp's arm ([`LAMP_LUMENS`],
//! [`LAMP_RANGE`]), bright enough that under it the night is as light as
//! the day, and as the night's moonlight (`sky`) 16 m away, fading out to
//! nothing 40 m away, where the moonlight is all there is. The lamps come on
//! as the sun goes down and go
//! off as it comes up (`sky::Daylight::lamps`). By day there is no light at
//! all, so the day costs what it did before there were lamps; at night at
//! most [`MOST_LIT`] lamps throw light at once, the nearest the camera on
//! the island you are on. Lights cast no shadows: a shadow map for a point
//! light is the world drawn six times more.
//!
//! The lights are sorted into the camera's clusters on the CPU, on every
//! platform (`cluster_lights_on_the_cpu` in `lib.rs`).
//!
//! **Bridge Wars' respawn anchors are lit the same way** (since 2026-11-11,
//! Hajun, whose map of that day took the lamps off the islands: "make the
//! spawn anchor purple crystal thing emit some light so that its not
//! completely dark, while still remaining purple"): a part in the anchors'
//! material (`arena::SPAWN_MATERIAL`) glows in its own purple from dusk to
//! dawn ([`ANCHOR_GLOW`]) and throws a pale purple light from inside its
//! tip ([`ANCHOR_LUMENS`], [`ANCHOR_RANGE`], [`ANCHOR_COLOR`],
//! [`ANCHOR_LIFT`]), a fifth of a street lamp's, enough to see its island
//! by. The light is inside the pillar so that none of it falls on the
//! pillar's own faces, which are purple by their glow alone. What gives off
//! light, and how, is [`SOURCES`], by material. A broken anchor's pillar
//! is hidden (`arena::break_anchors`), and its light goes out with it, as
//! any lamp's does that is not drawn.

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
/// lit a little more brightly than by the noon sky; 16 m out, as brightly as
/// by the night's own light, the moon's. Two and a half times what it was
/// until 2026-10-30, when Hajun asked for each lamp to light a larger area,
/// "maybe 2~3x": at 2,000,000 the ground was lit as brightly as by the moon
/// out to 9 m, a third of the ground it is now.
const LAMP_LUMENS: f32 = 5_000_000.0;
/// How far a lamp's light reaches, in metres, fading out before it: two and
/// a half times the 16 m it was until 2026-10-30, with the light, so that it
/// carries across the gaps between the new town's lamps, 38 m apart round
/// the ring round its pond, and every spot along it is lit by two or three.
/// At 16 m, on the laptop's GPU the night cost a third more than the day, and
/// at 20, half as much again (2026-10-16): further costs more.
const LAMP_RANGE: f32 = 40.0;
/// A warm white, a little yellower than daylight.
const LAMP_COLOR: Color = Color::srgb(1.0, 0.87, 0.68);
/// How brightly a bulb glows when lit, as so many times its colour, added
/// to what the camera sees as it is: far past white, so a lit bulb is drawn
/// white. (Until 2026-11-11 this said the camera's exposure took it down to
/// four times its colour. It does not: Bevy 0.19 takes the exposure to
/// emitted light only by the material's `emissive_exposure_weight`, which
/// is 0, and never reads the alpha of `emissive`; read in its source,
/// `pbr_material.rs` and `pbr_functions.wgsl`, when the anchors' glow at
/// 1,500 came out white.)
const BULB_GLOW: f32 = 4_000.0;
/// The most lamps that throw light at once. More than the town has, so that
/// in the town every lamp does, and as many as Bridge Wars' map has with its
/// four anchors (20 lamps and the four); it stops a plot with a field of
/// lamps on it costing as many lights over every spot as it has lamps. The
/// rest still glow.
const MOST_LIT: usize = 24;

/// How much light one of Bridge Wars' respawn anchors gives, in lumens, and
/// how far it reaches, in metres: a fifth of a street lamp's, from 3.5 m
/// up. It lights the middle of its island, 6 m away, about twice as
/// brightly as the moon does, the slab it stands on about as brightly as
/// the noon sky, and the island's far edge, 16 m away, hardly at all. My
/// numbers, for "some light so that its not completely dark".
const ANCHOR_LUMENS: f32 = 1_000_000.0;
const ANCHOR_RANGE: f32 = 25.0;
/// The colour of an anchor's light: a pale purple, pale enough that the
/// island under it keeps its own colours.
const ANCHOR_COLOR: Color = Color::srgb(0.78, 0.62, 1.0);
/// How brightly an anchor glows when lit, as so many times its colour,
/// added to what the camera sees as it is ([`BULB_GLOW`]): one and a half
/// times, which the tonemapping shows as a bright purple, about as it is
/// in daylight. At 1,500, for a first try, it was white.
const ANCHOR_GLOW: f32 = 1.5;
/// How far over the middle of an anchor its light is, in metres: inside
/// its tip, a quarter of a metre under the point, the pillar being 3 m
/// tall. **Inside it, so that the pillar stays purple**: no face of a
/// shape is lit from within it, and its glow alone colours it. For a first
/// try the light was 1.25 m over the tip, and lit the faces under it some
/// ten times over, the pale light's own shine with it: the pillar came out
/// white. And high in it, so that the slab under it is not lit several
/// times as brightly as the noon sky, as from its middle it would be
/// (three and a half times, worked out, not tried).
const ANCHOR_LIFT: f32 = 1.25;

/// Something in a model that gives off light from dusk to dawn, by the
/// material it is made of: how much light each one throws, how far and in
/// what colour, from how far over the middle of it, and how brightly the
/// thing itself glows, in its own colour.
struct Source {
    material: &'static str,
    lumens: f32,
    range: f32,
    color: Color,
    glow: f32,
    lift: f32,
}

/// Everything that gives off light: a street lamp's bulbs, and one of
/// Bridge Wars' respawn anchors.
const SOURCES: [Source; 2] = [
    Source {
        material: BULB_MATERIAL,
        lumens: LAMP_LUMENS,
        range: LAMP_RANGE,
        color: LAMP_COLOR,
        glow: BULB_GLOW,
        lift: 0.0,
    },
    Source {
        material: crate::arena::SPAWN_MATERIAL,
        lumens: ANCHOR_LUMENS,
        range: ANCHOR_RANGE,
        color: ANCHOR_COLOR,
        glow: ANCHOR_GLOW,
        lift: ANCHOR_LIFT,
    },
];

/// A lamp's light, and how much of it there is when it is on, in lumens.
#[derive(Component)]
struct Lamp {
    lumens: f32,
}

/// A mesh of bulbs, which glows while the lamps are on, this brightly.
#[derive(Component)]
struct Bulb {
    glow: f32,
}

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
        let Some(source) = SOURCES
            .iter()
            .find(|source| crate::island::is_material(&material.0, source.material))
        else {
            continue;
        };
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
        commands.entity(part).insert(Bulb { glow: source.glow });
        for middle in lamps_of(&world) {
            commands.spawn((
                ChildOf(part),
                Lamp { lumens: source.lumens },
                PointLight {
                    color: source.color,
                    intensity: 0.0,
                    range: source.range,
                    shadow_maps_enabled: false,
                    ..default()
                },
                Transform::from_translation(into.transform_point3(middle + Vec3::Y * source.lift)),
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
    mut lamps: Query<(Entity, &Lamp, &ChildOf, &GlobalTransform, &mut PointLight, &mut Visibility)>,
    drawn: Query<&InheritedVisibility>,
    bulbs: Query<(&MeshMaterial3d<StandardMaterial>, &Bulb)>,
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
                .filter(|(_, _, bulb, ..)| drawn.get(bulb.parent()).is_ok_and(|drawn| drawn.get()))
                .map(|(lamp, _, _, place, ..)| (place.translation().distance_squared(eye), lamp)),
        );
        nearest.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        nearest.truncate(MOST_LIT);
    }
    for (lamp, source, _, _, mut light, mut visibility) in &mut lamps {
        let lit = nearest.iter().any(|&(_, near)| near == lamp);
        visibility.set_if_neq(if lit {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let intensity = source.lumens * on;
        if lit && light.intensity != intensity {
            light.intensity = intensity;
        }
    }
    // Every bulb of a model shares its material, so each is looked at once.
    let mut seen: Vec<AssetId<StandardMaterial>> = Vec::new();
    for (material, source) in &bulbs {
        let id = material.id();
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        let Some(glow) = materials.get(id).map(|bulb| glow(bulb, source.glow * on)) else {
            continue;
        };
        if materials.get(id).is_some_and(|bulb| bulb.emissive != glow)
            && let Some(mut bulb) = materials.get_mut(id)
        {
            bulb.emissive = glow;
        }
    }
}

/// How a bulb of `material` glows at `strength`, its own ([`Bulb::glow`])
/// by how far the lamps are on: in its own colour, out by day.
fn glow(material: &StandardMaterial, strength: f32) -> LinearRgba {
    // Added to what the camera sees as it is: the camera's exposure is
    // taken to it only by the material's `emissive_exposure_weight`, 0.
    let own = material.base_color.to_linear();
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
        let lit = glow(&bulb, BULB_GLOW);
        assert!((lit.red - BULB_GLOW).abs() < 1e-2 && (lit.blue - 0.33 * BULB_GLOW).abs() < 1e-2);
    }

    #[test]
    fn each_of_bridge_wars_anchors_is_one_light_inside_its_tip_and_glows_purple() {
        // The pillars as Hajun's map has them, each a part of its own in
        // the anchors' material: one light to each, at the anchor
        // (`roundtown_net::ARENA_ANCHORS`), inside the pillar's upper half
        // and on its axis, where none of its faces is lit by it.
        let source = SOURCES
            .iter()
            .find(|source| source.material == crate::arena::SPAWN_MATERIAL)
            .expect("the anchors give off light");
        let parts = crate::island::glb::read("pvptownmap.glb");
        let pillars: Vec<_> = parts
            .iter()
            .filter(|part| {
                part.material
                    .as_deref()
                    .is_some_and(|name| crate::island::is_material(name, source.material))
            })
            .collect();
        assert_eq!(pillars.len(), roundtown_net::ARENA_ANCHORS.len());
        for pillar in pillars {
            let corners: Vec<Vec3> = pillar.triangles.iter().flatten().copied().collect();
            let lights = lamps_of(&corners);
            assert_eq!(lights.len(), 1, "{}: {lights:?}", pillar.node);
            let light = lights[0] + Vec3::Y * source.lift;
            let tip = corners.iter().map(|corner| corner.y).fold(f32::MIN, f32::max);
            assert!(light.y > lights[0].y && light.y < tip - 0.1, "{}: light {} m up, tip {tip}", pillar.node, light.y);
            // Behind every face of it: inside.
            for [a, b, c] in &pillar.triangles {
                let out = (*b - *a).cross(*c - *a);
                assert!(out.dot(light - *a) < 0.0, "{}: its light is in front of one of its faces", pillar.node);
            }
            let over = roundtown_net::ARENA_ANCHORS
                .iter()
                .map(|anchor| anchor.distance(light.xz()))
                .fold(f32::MAX, f32::min);
            assert!(over < 0.1, "{}: its light is {over} m from the nearest anchor", pillar.node);
        }
        // Reaching the middle of its island, 6 m away, and its far edge,
        // and not as far as the town's lamps or another island.
        assert!(source.range > 16.4 && source.range < 40.0);
        // Its glow is its own colour, as purple as by day: more blue than
        // red and hardly any green, and no brighter than twice its colour:
        // the tonemapping takes anything much brighter to white.
        let crystal = StandardMaterial {
            base_color: Color::linear_rgb(0.544, 0.066, 0.8),
            ..default()
        };
        assert_eq!(glow(&crystal, 0.0), LinearRgba::BLACK);
        let lit = glow(&crystal, source.glow);
        assert!(lit.blue > lit.red && lit.red > 4.0 * lit.green, "{lit:?}");
        assert!(source.glow <= 2.0);
        assert!(lit.red <= 2.0 && lit.green <= 2.0 && lit.blue <= 2.0, "{lit:?}");
        // And its light is purple too: more blue than red, more red than
        // green.
        let light = source.color.to_linear();
        assert!(light.blue > light.red && light.red > light.green, "{light:?}");
    }
}
