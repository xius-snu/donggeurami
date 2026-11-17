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
//! **A thing drops onto its pad as it is made** (since late on 2026-11-11; Hajun:
//! "the minerals spawning animation is like a growing big animation. i
//! want it to be a drop from a certain height animation. so that even
//! when player is standing on top of the generator, u can see the mineral
//! for a bit before the player pick it up"): it appears the rules'
//! `DROP_FROM` over where it will hang, over the head of anyone standing
//! on the pad, and falls there in their `DROP_SECS`, faster as it comes
//! (`roundtown_net::bridge_wars::drop_height`, which is the rules' because
//! whoever runs the generators goes by where the thing is for who can
//! reach it). Until then it came up from nothing to its size where it
//! lay, and someone standing on the pad had it before it was seen.
//!
//! **It is yours as it comes within your reach, in the air** (since
//! 2026-11-12; Hajun: "the delay is a bit too much for the delay between
//! the item dropping from generator and the user collecting - user should
//! be able to collect in the air too maybe when they jump"): half a metre
//! over your head (the rules' `REACH_OVER`), 0.45 s after it is made if
//! you stand on its pad, and sooner if you jump for it; taken there, it
//! flies to you from where it is. For the evening before, nobody could
//! have it until it had landed and lain 0.3 s, 0.85 s after it was made.
//!
//! This is only how they look. What is made, where, and who has picked up
//! what are the rules' (`roundtown_net::bridge_wars::Mint`): the server's
//! online, where each thing is an entity it sends ([`Loot`]) and takes
//! away again the moment someone has it, so that everyone sees the same
//! things at the same times and nobody takes what someone else took; and
//! the game's own offline (`arena::gather`). Either way a thing here is
//! [`lay`]n as it is made and [`take`]n as it goes: taken, it flies to
//! whoever took it, shrinking, and is gone, which is all there is to the
//! magnet.
//!
//! **What someone who died on land held of sapphire and amethyst lies
//! here too** (since the night of 2026-11-15; Hajun: "they should drop the
//! sapphires and amethyst like small scatter explosion from their dead
//! body location like mineraft dropping items on death, and u only get it
//! if u pick it up"; the rules' `Mint::spill`): each flies out from the
//! middle of whoever died, up and over in the rules' `TOSS_SECS`
//! (`toss`), to where the rules had it come down, and bobs and turns
//! there as the rest do, over whatever it lies on, the land or a plank,
//! until someone walks into it; and nobody has it until it has lain a
//! moment (the rules' `SPILL_WAIT`), so that whoever stands over the dead
//! sees it lie before it flies to them. One that is more than one of its item,
//! which is what someone who held more than ten of it drops, is drawn
//! bigger ([`size`]). One that lay on a plank that has gone drops away
//! with it.
//!
//! Each is one entity, the model's scene under it, carrying its fight's
//! `Venue`, so that it is drawn only while you are there, and `InArena`,
//! so that it goes with the game. Only those near the camera are moved: a
//! full map has 240 of them, most of them a hundred metres off. At night
//! they glow a little in their own colours ([`glow`]), as the anchors do
//! (`lamps`): a sapphire is nearly black by moonlight, and the smaller
//! islands have no lamps.
//!
//! **In any fight** (since 2026-11-18, for Last Stand's generators and its
//! zombies' drops): a thing is laid on the venue of the game it is of, a
//! generator's over that game's pads' top ([`lay`]; Bridge Wars' own
//! `PAD_TOP` for its own, and [`PadTop`] for what the server sends of any
//! other's), and only whoever is on that venue is taken to have taken it.

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldAssetRoot, WorldInstanceReady};
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::{DROP_FROM, DROP_SECS, HOVER, MAGNET, TOSS_SECS, drop_height, toss};
use roundtown_net::{Item, Loot, Member, PAD_TOP};

use crate::arena::{Fighting, InArena};
use crate::island::Venue;
use crate::sky::Daylight;
use crate::{Dead, Player, ThirdPersonCamera};

/// How far up and down a thing bobs from where it hangs, the rules'
/// `HOVER` over its pad's top, in metres, how long one bob takes, in
/// seconds, and how fast it turns, in radians a second, a turn in seven
/// seconds (Hajun: "bob up and down and rotate slowly").
const BOB: f32 = 0.1;
const BOB_SECS: f32 = 2.4;
const SPIN: f32 = 0.9;
/// How long a thing takes to fly to whoever took it, in seconds; and how
/// small it is as it arrives, as a share.
const FLY_SECS: f32 = 0.16;
const FLY_SMALL: f32 = 0.25;
/// How high on a body a thing flies to, in metres over its feet.
const FLY_TO: f32 = 1.0;
/// How far past the magnet's reach whoever took a thing may be found, in
/// metres: online they are drawn a couple of updates behind where the
/// server had them as it gave it to them.
const TAKER_SLACK: f32 = 2.5;
/// How near the camera a thing has to be, in metres across the ground, to
/// be moved at all: fixed over you (`OrbitCamera::top_down`), the camera
/// sees 24 m to either side on the widest of phones (40 m at the furthest
/// it zoomed, until late on 2026-11-11, which is what this was set by).
const NEAR: f32 = 48.0;
/// How brightly a thing glows at night, as so many times its own colour
/// (`lamps::ANCHOR_GLOW`, which says how Bevy counts it): about as it
/// looks by day.
const GLOW: f32 = 0.9;
/// The biggest a thing that is more than one of its item is drawn, as so
/// many times its size ([`size`]).
const BIGGEST: f32 = 1.8;
/// How far a thing that was dropped falls away, in metres, when the plank
/// it lay on has gone and nobody took it.
const FALLS_AWAY: f32 = 1.6;

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

/// One thing lying on a pad, or where someone who died dropped it: which,
/// what it is, where it hangs at the middle of its bob, when it was made,
/// by the game's clock, and how far through its turn it began; where it
/// was thrown from, if it was dropped; and how many times its model's
/// size it is drawn ([`size`]).
#[derive(Component)]
pub(crate) struct Thing {
    pub(crate) key: Key,
    pub(crate) item: Item,
    at: Vec3,
    born: f32,
    phase: f32,
    from: Option<Vec3>,
    size: f32,
}

/// How many times its model's size a thing that is `worth` of its item is
/// drawn: as it is for one, and for more than one as much bigger as holds
/// that many, up to [`BIGGEST`].
fn size(worth: u16) -> f32 {
    f32::from(worth.max(1)).cbrt().min(BIGGEST)
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

/// How high the tops of the generators' pads are in the fight being
/// played, in metres: where a thing the server sends from a generator
/// hangs over (a [`Loot`] says nothing of its height, only a dropped one
/// does, `Spilt::ground`), on any map but Bridge Wars', whose things hang
/// over its own `PAD_TOP` whatever this says, as they always did. Bridge
/// Wars' `PAD_TOP` as the app opens, and out of any other game. **Any
/// fight may own it**, as it does `arena::Fighting` (since 2026-11-18):
/// another game whose pads stand otherwise sets it as it begins, and sets
/// it back to Bridge Wars' once as it ends. (Last Stand's map is stood up
/// a quarter of a metre so that its pads' tops are where Bridge Wars' are:
/// none of its own needed. What the server sends in the frame or two
/// before this device knows where the fight is, as it is put back into a
/// game, is hung over Bridge Wars' pads, where it was put, and keeps that
/// height as it is moved onto the fight's venue, `arena::settle`: right
/// for every fight there is.)
#[derive(Resource, Clone, Copy, PartialEq, Debug)]
pub(crate) struct PadTop(pub(crate) f32);

impl PadTop {
    /// The top of the pads a thing the server sends from a generator on
    /// `venue` hangs over: Bridge Wars' own `PAD_TOP` on its map, so that
    /// nothing another game leaves here moves its things; this anywhere
    /// else.
    pub(crate) fn on(self, venue: Venue) -> f32 {
        if venue == Venue::Arena { PAD_TOP } else { self.0 }
    }
}

impl Default for PadTop {
    fn default() -> Self {
        Self(PAD_TOP)
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<PadTop>()
        .add_systems(Startup, load)
        .add_systems(Update, (follow_server, animate, glow));
}

fn load(assets: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(Look {
        scenes: Item::ALL.map(|item| assets.load(GltfAssetLabel::Scene(0).from_asset(model(item)))),
        materials: Default::default(),
        lit: None,
    });
}

/// The thing `loot` says, made now, on `venue`, the fight's: one a
/// generator made, over a pad's top, `pad_top` up (Bridge Wars' `PAD_TOP`
/// for its own), high over where it will hang, to drop there
/// ([`animate`]); or one that someone who died dropped, at the middle of
/// them, to fly out to where it lies, over the ground it says. Turned out
/// of step with whatever lies beside it, by where it lies.
pub(crate) fn lay(commands: &mut Commands, look: &Look, now: f32, key: Key, loot: Loot, venue: Venue, pad_top: f32) -> Entity {
    let item = loot.item;
    let ground = loot.spilt.map_or(pad_top, |spilt| spilt.ground);
    let place = Vec3::new(loot.x, ground + HOVER, loot.z);
    let from = loot.spilt.map(|spilt| Vec3::new(spilt.from_x, spilt.from_y, spilt.from_z));
    let size = size(loot.spilt.map_or(1, |spilt| spilt.worth));
    let phase = ((loot.x * 12.9898 + loot.z * 78.233).sin() * 43_758.547).rem_euclid(1.0) * TAU;
    commands
        .spawn((
            Thing {
                key,
                item,
                at: place,
                born: now,
                phase,
                from,
                size,
            },
            InArena,
            venue,
            Name::new(format!("{item:?}")),
            WorldAssetRoot(look.scenes[item.index()].clone()),
            Transform::from_translation(from.unwrap_or(place + Vec3::Y * DROP_FROM))
                .with_rotation(Quat::from_rotation_y(phase))
                .with_scale(Vec3::splat(size)),
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
/// whoever is nearest it, who is whoever the server gave it to: a person,
/// or, since 2026-11-12, one of the computer's players, which pick up
/// what their purchases need (`roundtown_net::bots`). Until then only
/// people were looked for, the computer's picking up nothing, and a
/// thing one of them took would have flown to the nearest person, or
/// nowhere. Each laid on the fight's venue (`Fighting::served_on`; one
/// sent before this device knew where the fight was moved there as soon
/// as it does, `arena::settle`), a generator's at the fight's pads' top
/// ([`PadTop::on`]), and taken by whoever is nearest of those on the
/// thing's own venue (since 2026-11-18; Bridge Wars' map alone until
/// then).
#[allow(clippy::too_many_arguments)]
fn follow_server(
    time: Res<Time>,
    look: Res<Look>,
    (fighting, pad_top): (Res<Fighting>, Res<PadTop>),
    fresh: Query<(Entity, &Loot), (With<Remote>, Added<Loot>)>,
    sources: Query<(), With<Loot>>,
    things: Query<(Entity, &Thing, &Venue), Without<Flying>>,
    people: Query<(Entity, &Transform, &Venue, Option<&Member>, Has<Player>), Without<Dead>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();
    if !fresh.is_empty() {
        let mine = people.iter().find(|(.., mine)| *mine).map(|(_, _, venue, ..)| *venue);
        let venue = fighting.served_on(mine);
        for (source, loot) in &fresh {
            let thing = lay(&mut commands, &look, now, Key::Server(source), *loot, venue, pad_top.on(venue));
            fighting.settle_later(&mut commands, thing);
        }
    }
    for (thing, lying, &on) in &things {
        let Key::Server(source) = lying.key else {
            continue;
        };
        if sources.contains(source) {
            continue;
        }
        let here = lying.at.xz();
        let taker = people
            .iter()
            // Anyone but a zombie, which picks up nothing.
            .filter(|(_, _, venue, member, mine)| {
                **venue == on && (*mine || member.is_some_and(|member| member.look != roundtown_net::bridge_wars::ZOMBIE_LOOK))
            })
            .map(|(person, at, ..)| (person, at.translation.xz().distance(here)))
            .filter(|(_, away)| *away <= MAGNET + TAKER_SLACK)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(person, _)| person);
        take(&mut commands, thing, taker);
    }
}

/// Bobs and turns every thing near the camera, drops a new one onto its
/// pad, and flies a taken one to whoever took it, where it is gone.
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
        let here = place(thing.at, thing.from, age);
        let turn = Quat::from_rotation_y(thing.phase + age * SPIN);
        if let Some(mut flying) = flying {
            flying.age += dt;
            let done = (flying.age / FLY_SECS).min(1.0);
            if done >= 1.0 {
                commands.entity(entity).despawn();
                continue;
            }
            // To whoever took it; one that nobody did, up and gone, or,
            // dropped and its plank gone from under it, down with that.
            let nowhere = here + Vec3::Y * if thing.from.is_some() { -FALLS_AWAY } else { 0.5 };
            let to = flying
                .to
                .and_then(|taker| bodies.get(taker).ok())
                .map_or(nowhere, |taker| taker.translation + Vec3::Y * FLY_TO);
            *transform = Transform {
                translation: here.lerp(to, done * done),
                rotation: turn,
                scale: Vec3::splat(thing.size * (1.0 + (FLY_SMALL - 1.0) * done)),
            };
            continue;
        }
        // Far from the camera it is left as it was, out of sight, once it
        // has landed.
        if eye.is_some_and(|eye| eye.distance(thing.at.xz()) > NEAR) && age > DROP_SECS.max(TOSS_SECS) {
            continue;
        }
        *transform = Transform {
            translation: here,
            rotation: turn,
            scale: Vec3::splat(thing.size),
        };
    }
}

/// Where a thing that hangs at `at` is, `age` seconds after it was made:
/// one a generator made over that by its [`height`]; one that was dropped,
/// thrown from `from`, on its way from there as the rules have it
/// (`toss`), and from the moment it comes down bobbing about `at` as the
/// rest do.
fn place(at: Vec3, from: Option<Vec3>, age: f32) -> Vec3 {
    match from {
        Some(from) if age < TOSS_SECS => toss(from, at, age),
        Some(_) => at + Vec3::Y * bob(age - TOSS_SECS),
        None => at + Vec3::Y * height(age),
    }
}

/// How far over where it hangs a thing a generator made is, `age` seconds
/// after it was made, in metres: dropping as the rules have it
/// (`drop_height`), and from the moment it lands bobbing about there.
fn height(age: f32) -> f32 {
    if age < DROP_SECS {
        return drop_height(age);
    }
    bob(age - DROP_SECS)
}

/// How far over where it hangs a thing is, `since` seconds after it came
/// down, in metres: up first.
fn bob(since: f32) -> f32 {
    BOB * (since * TAU / BOB_SECS).sin()
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
    fn a_thing_is_laid_on_its_fight_s_venue_over_its_pads_top() {
        // Since 2026-11-18 on whichever venue its game is played, over that
        // game's pads: Last Stand's here, made up as half a metre up.
        let mut world = World::new();
        let look = Look {
            scenes: Default::default(),
            materials: Default::default(),
            lit: None,
        };
        let loot = Loot {
            item: Item::Gold,
            x: 6.0,
            z: 1.0,
            spilt: None,
        };
        let thing = {
            let mut commands = world.commands();
            lay(&mut commands, &look, 0.0, Key::Own(1), loot, Venue::LastStand, 0.5)
        };
        world.flush();
        assert_eq!(world.get::<Venue>(thing), Some(&Venue::LastStand));
        assert_eq!(world.get::<Thing>(thing).expect("laid").at, Vec3::new(6.0, 0.5 + HOVER, 1.0));
        assert!(world.get::<InArena>(thing).is_some(), "to go with its game");
        // And what the server sends hangs over Bridge Wars' pads, until a
        // fight's game says its own are elsewhere; on Bridge Wars' own map
        // over its own, whatever another game has left said.
        assert_eq!(PadTop::default(), PadTop(PAD_TOP));
        assert_eq!(PadTop::default().on(Venue::LastStand), PAD_TOP);
        assert_eq!((PadTop(0.9).on(Venue::LastStand), PadTop(0.9).on(Venue::Arena)), (0.9, PAD_TOP));
    }

    #[test]
    fn a_thing_drops_onto_its_pad_and_then_bobs_there() {
        // It begins over the head of anyone standing on the pad, comes
        // down all the way, faster as it comes, and never under where it
        // hangs on the way.
        assert_eq!(height(0.0), DROP_FROM);
        assert!(PAD_TOP + HOVER + DROP_FROM > PAD_TOP + crate::PLAYER_HEIGHT + 1.0);
        let (mut last, mut last_step) = (height(0.0), 0.0);
        for step in 1..=20 {
            let at = height(DROP_SECS * step as f32 / 20.0 - 1e-4);
            assert!(at < last && at >= 0.0, "{at} after {last}");
            assert!(last - at > last_step, "it falls no faster as it comes");
            (last, last_step) = (at, last - at);
        }
        assert!(height(DROP_SECS - 1e-4) < 0.01);
        // Landed, it is where it hangs, and bobs no further than its bob
        // from there, up first.
        assert_eq!(height(DROP_SECS), 0.0);
        assert!(height(DROP_SECS + 0.1) > 0.0);
        for step in 0..200 {
            assert!(height(DROP_SECS + step as f32 * 0.05).abs() <= BOB + 1e-5);
        }
        // It begins well out of the reach of someone standing on the pad,
        // so that they see it come, and is within it wherever it bobs.
        let reach = roundtown_net::bridge_wars::REACH_OVER;
        assert!(HOVER + DROP_FROM > reach + 2.0);
        assert!(HOVER + BOB < reach);
    }

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
