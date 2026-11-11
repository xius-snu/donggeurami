//! What Bridge Wars' generators have made, lying on their pads (since
//! 2026-11-11; Hajun: "make the generators functional in the pvpmap, using
//! the 4 item models in the asset folder as the items being generated -
//! they should drop in semi random location on top of the stone pad on
//! each island ... the items should bob up and down and rotate slowly, and
//! when the user can collect it by walking in it (short magnet)"): Hajun's
//! four models, `silver_item.glb`, `gold_item.glb`, `sapphire_item.glb` and
//! `amethyst_item.glb`, each a cut stone with its own number of sides,
//! hung a little over the pad it came up on, bobbing and turning.
//!
//! This is only how they look. What is made, where, and who has picked up
//! what are the rules' (`roundtown_net::bridge_wars::Mint`): the server's
//! online, where each thing is an entity it sends ([`Loot`]) and takes
//! away again the moment someone has it, so that everyone sees the same
//! things at the same times and nobody takes what someone else took; and
//! the game's own offline (`arena::gather`). Either way a thing here is
//! [`lay`]n as it comes up and [`take`]n as it goes: taken, it flies to
//! whoever took it, shrinking, and is gone, which is all there is to the
//! magnet.
//!
//! Each is one entity, the model's scene under it, carrying the arena's
//! `Venue`, so that it is drawn only while you are there, and `InArena`,
//! so that it goes with the game. Only those near the camera are moved: a
//! full map has 240 of them, most of them a hundred metres off. At night
//! they glow a little in their own colours ([`glow`]), as the anchors do
//! (`lamps`): a sapphire is nearly black by moonlight, and the smaller
//! islands have no lamps.

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldAssetRoot, WorldInstanceReady};
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::MAGNET;
use roundtown_net::{Item, Loot, Member, PAD_TOP};

use crate::arena::InArena;
use crate::island::Venue;
use crate::sky::Daylight;
use crate::{Dead, Player, ThirdPersonCamera};

/// How far over its pad's top a thing's own middle hangs, in metres, at
/// the middle of its bob: the models reach 0.18 m under theirs at the
/// most (the silver), so the lowest of them clears the pad by a hand.
const HOVER: f32 = 0.42;
/// How far up and down a thing bobs from there, in metres, how long one
/// bob takes, in seconds, and how fast it turns, in radians a second, a
/// turn in seven seconds (Hajun: "bob up and down and rotate slowly").
const BOB: f32 = 0.1;
const BOB_SECS: f32 = 2.4;
const SPIN: f32 = 0.9;
/// How long a thing takes to come up to its size as it is made, and to
/// fly to whoever took it, in seconds; and how small it is as it arrives,
/// as a share.
const POP_SECS: f32 = 0.22;
const FLY_SECS: f32 = 0.16;
const FLY_SMALL: f32 = 0.25;
/// How high on a body a thing flies to, in metres over its feet.
const FLY_TO: f32 = 1.0;
/// How far past the magnet's reach whoever took a thing may be found, in
/// metres: online they are drawn a couple of updates behind where the
/// server had them as it gave it to them.
const TAKER_SLACK: f32 = 2.5;
/// How near the camera a thing has to be, in metres across the ground, to
/// be moved at all: the camera sees 40 m at the furthest it zooms.
const NEAR: f32 = 48.0;
/// How brightly a thing glows at night, as so many times its own colour
/// (`lamps::ANCHOR_GLOW`, which says how Bevy counts it): about as it
/// looks by day.
const GLOW: f32 = 0.9;

/// The model of each item, in `assets/`.
pub(crate) fn model(item: Item) -> &'static str {
    match item {
        Item::Silver => "silver_item.glb",
        Item::Gold => "gold_item.glb",
        Item::Sapphire => "sapphire_item.glb",
        Item::Amethyst => "amethyst_item.glb",
    }
}

/// Which thing one lying here is, to whoever rules it: the server's
/// entity for it online, or the number the game's own generators gave it
/// offline.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Key {
    Own(u32),
    Server(Entity),
}

/// One thing lying on a pad: which, what it is, where it hangs at the
/// middle of its bob, when it came up, by the game's clock, and how far
/// through its bob and its turn it began.
#[derive(Component)]
pub(crate) struct Thing {
    pub(crate) key: Key,
    pub(crate) item: Item,
    at: Vec3,
    born: f32,
    phase: f32,
}

/// A thing that has been taken, on its way to whoever took it, if anyone
/// is known to have: for how long.
#[derive(Component)]
pub(crate) struct Flying {
    to: Option<Entity>,
    age: f32,
}

/// The models, loaded as the app opens so that the first thing of a game
/// is there the moment it is made; each one's materials, as they turn up,
/// to glow at night; and how far the lamps were on when those were last
/// set, if they have been.
#[derive(Resource)]
pub(crate) struct Look {
    scenes: [Handle<WorldAsset>; 4],
    materials: [Vec<Handle<StandardMaterial>>; 4],
    lit: Option<f32>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, load)
        .add_systems(Update, (follow_server, animate, glow));
}

fn load(assets: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(Look {
        scenes: Item::ALL.map(|item| assets.load(GltfAssetLabel::Scene(0).from_asset(model(item)))),
        materials: Default::default(),
        lit: None,
    });
}

/// A thing of `item` come up at `at` across the ground, on a pad's top:
/// small, to come up to its size ([`animate`]), and turned and bobbing
/// out of step with whatever lies beside it, by where it lies.
pub(crate) fn lay(commands: &mut Commands, look: &Look, now: f32, key: Key, item: Item, at: Vec2) -> Entity {
    let place = Vec3::new(at.x, PAD_TOP + HOVER, at.y);
    let phase = ((at.x * 12.9898 + at.y * 78.233).sin() * 43_758.547).rem_euclid(1.0) * TAU;
    commands
        .spawn((
            Thing {
                key,
                item,
                at: place,
                born: now,
                phase,
            },
            InArena,
            Venue::Arena,
            Name::new(format!("{item:?}")),
            WorldAssetRoot(look.scenes[item.index()].clone()),
            Transform::from_translation(place)
                .with_rotation(Quat::from_rotation_y(phase))
                .with_scale(Vec3::splat(0.01)),
            // Shown with the island it is on (`island::show_where_i_am`).
            Visibility::Hidden,
        ))
        .observe(learn_look)
        .id()
}

/// `thing` has been taken, by `by` if anyone is known to have: it flies
/// to them and is gone.
pub(crate) fn take(commands: &mut Commands, thing: Entity, by: Option<Entity>) {
    commands.entity(thing).insert(Flying { to: by, age: 0.0 });
}

/// Notes the materials of a thing's model as it has spawned, for the night
/// ([`glow`]): every thing of an item shares them.
fn learn_look(
    ready: On<WorldInstanceReady>,
    things: Query<&Thing>,
    children: Query<&Children>,
    parts: Query<&MeshMaterial3d<StandardMaterial>>,
    mut look: ResMut<Look>,
) {
    let Ok(thing) = things.get(ready.entity) else {
        return;
    };
    let index = thing.item.index();
    for part in children.iter_descendants(ready.entity) {
        let Ok(material) = parts.get(part) else {
            continue;
        };
        if !look.materials[index].contains(&material.0) {
            look.materials[index].push(material.0.clone());
            look.lit = None;
        }
    }
}

/// Online, the things are the server's: each [`Loot`] it sends is laid
/// here as it comes, and one it has taken away is taken here, flying to
/// the nearest person to it, who is whoever the server gave it to.
fn follow_server(
    time: Res<Time>,
    look: Res<Look>,
    fresh: Query<(Entity, &Loot), (With<Remote>, Added<Loot>)>,
    sources: Query<(), With<Loot>>,
    things: Query<(Entity, &Thing), Without<Flying>>,
    people: Query<(Entity, &Transform, &Venue, Option<&Member>, Has<Player>), Without<Dead>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();
    for (source, loot) in &fresh {
        lay(&mut commands, &look, now, Key::Server(source), loot.item, Vec2::new(loot.x, loot.z));
    }
    for (thing, lying) in &things {
        let Key::Server(source) = lying.key else {
            continue;
        };
        if sources.contains(source) {
            continue;
        }
        let here = lying.at.xz();
        let taker = people
            .iter()
            .filter(|(_, _, venue, member, mine)| **venue == Venue::Arena && (*mine || member.is_some_and(|member| !member.ai)))
            .map(|(person, at, ..)| (person, at.translation.xz().distance(here)))
            .filter(|(_, away)| *away <= MAGNET + TAKER_SLACK)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(person, _)| person);
        take(&mut commands, thing, taker);
    }
}

/// Bobs and turns every thing near the camera, brings a new one up to its
/// size, and flies a taken one to whoever took it, where it is gone.
fn animate(
    time: Res<Time>,
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Thing>)>,
    mut things: Query<(Entity, &Thing, &mut Transform, Option<&mut Flying>), Without<ThirdPersonCamera>>,
    bodies: Query<&Transform, (Without<Thing>, Without<ThirdPersonCamera>)>,
    mut commands: Commands,
) {
    let (now, dt) = (time.elapsed_secs(), time.delta_secs());
    let eye = cameras.single().map(|camera| camera.translation.xz()).ok();
    for (entity, thing, mut transform, flying) in &mut things {
        let age = (now - thing.born).max(0.0);
        let bob = Vec3::Y * (BOB * (age * TAU / BOB_SECS + thing.phase).sin());
        let turn = Quat::from_rotation_y(thing.phase + age * SPIN);
        if let Some(mut flying) = flying {
            flying.age += dt;
            let done = (flying.age / FLY_SECS).min(1.0);
            if done >= 1.0 {
                commands.entity(entity).despawn();
                continue;
            }
            let from = thing.at + bob;
            let to = flying
                .to
                .and_then(|taker| bodies.get(taker).ok())
                .map_or(from + Vec3::Y * 0.5, |taker| taker.translation + Vec3::Y * FLY_TO);
            *transform = Transform {
                translation: from.lerp(to, done * done),
                rotation: turn,
                scale: Vec3::splat(1.0 + (FLY_SMALL - 1.0) * done),
            };
            continue;
        }
        // Far from the camera it is left as it was, out of sight.
        if eye.is_some_and(|eye| eye.distance(thing.at.xz()) > NEAR) && age > POP_SECS {
            continue;
        }
        *transform = Transform {
            translation: thing.at + bob,
            rotation: turn,
            scale: Vec3::splat((age / POP_SECS).clamp(0.01, 1.0)),
        };
    }
}

/// Has the things glow in their own colours from dusk to dawn, as far as
/// the lamps are on (`sky::Daylight::lamps`): each item's materials, which
/// all of its things share, written only as that changes, or as a
/// material turns up.
fn glow(daylight: Res<Daylight>, mut look: ResMut<Look>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let on = daylight.lamps;
    if look.lit == Some(on) {
        return;
    }
    for handle in look.materials.iter().flatten() {
        let Some(own) = materials.get(handle).map(|material| material.base_color.to_linear()) else {
            continue;
        };
        let lit = LinearRgba::rgb(own.red * GLOW * on, own.green * GLOW * on, own.blue * GLOW * on);
        if materials.get(handle).is_some_and(|material| material.emissive != lit)
            && let Some(mut material) = materials.get_mut(handle)
        {
            material.emissive = lit;
        }
    }
    look.lit = Some(on);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_item_has_a_model_of_its_own_in_the_apk_that_sits_over_its_pad() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("mobile/android/app/assets-left-out.txt"))
            .expect("the APK's list of what it leaves out");
        let patterns: Vec<&str> = list
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.trim_start_matches("<dir>").trim_start_matches("<file>"))
            .collect();
        let mut seen = Vec::new();
        for item in Item::ALL {
            let name = model(item);
            assert!(!seen.contains(&name), "{name} twice");
            seen.push(name);
            let left_out = patterns.iter().find(|pattern| {
                let pattern: &str = pattern;
                if let Some(end) = pattern.strip_prefix('*') {
                    name.ends_with(end)
                } else if let Some(start) = pattern.strip_suffix('*') {
                    name.starts_with(start)
                } else {
                    pattern == name
                }
            });
            assert!(left_out.is_none(), "{name} is left out of the APK by `{}`", left_out.unwrap());
            // The model as the game places it: one material, a stone well
            // under a metre across with as many sides as the item has
            // (three, four, five, six), wider than it is tall, and no
            // lower under its own middle than it hangs over the pad at
            // the bottom of its bob.
            let parts = crate::island::glb::read(name);
            assert_eq!(parts.len(), 1, "{name}: one mesh of one material");
            let corners: Vec<Vec3> = parts[0].triangles.iter().flatten().copied().collect();
            let (low, high) = corners
                .iter()
                .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), corner| (low.min(*corner), high.max(*corner)));
            let size = high - low;
            assert!(size.x > 0.5 && size.x < 0.9 && size.z > 0.5 && size.z < 0.9, "{name}: {size} across");
            assert!(size.y < size.x, "{name}: {size}");
            assert!(low.y > -(HOVER - BOB) + 0.05, "{name}: it reaches {} under its middle", -low.y);
            let sides = 3 + item.index();
            assert_eq!(parts[0].triangles.len(), 8 * sides - 4, "{name}: a stone of {sides} sides");
        }
    }
}
