//! What lies about in a game of Last Stand that is not a generator's or a
//! zombie's thing (`loot` draws those, and what the dead drop of the four
//! things): the swords the dead drop, and the treasure chests (both since
//! 2026-11-18).
//!
//! **A sword the dead dropped** (Hajun: "they should drop all their loot
//! (silver,gold,sapphire, amethyst, sword) ... for sword, can only pick up
//! if its greater than the sword they are already holding"): its model, the
//! sword in a hand's own (`gear::SWORDS`), lying flat over the ground, flat
//! side up for the camera over it, thrown out from whoever died as what
//! they held is (`loot`), and bobbing and turning there as those things do,
//! until someone holding a lesser one walks into it (it flies to them) or
//! it has lain its while (it sinks away).
//!
//! **A treasure chest** (Hajun: "make treasurechest.glb that sometimes
//! spawn in the sand area or the ocean, with no notification to the player
//! of its location or existence ... u need to go close to it and it will
//! show the proximitychatbubble and u press that and the chest pops/vanishes
//! and you get a random chance item and random quantity"): his model,
//! standing where the game put it, on the sand or down in the sea, said to
//! nobody; the town's bubble comes up over it near it (`talk::Opens`), and
//! its tap opens it: it pops, a little bigger and then gone, and what was
//! in it rises off where it stood in gold ([`Found`]).
//!
//! Offline the game here says what comes and goes (`play`); online the
//! server's entities do ([`LyingSword`], [`Chest`]), each drawn here as it
//! comes and taken away as it goes.

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::{HOVER, MAGNET, TOSS_SECS, toss};
use roundtown_net::last_stand::{self as rules, Chest, Find, LyingSword};
use roundtown_net::{Item, Utility};

use crate::arena::InArena;
use crate::gear::{SWORDS, SWORD_SCALE, pointing};
use crate::hud;
use crate::island::Venue;
use crate::talk::Opens;
use crate::ui::{fill, ui};
use crate::{Player, ThirdPersonCamera};

/// The chest's model, in `assets/`.
pub(super) const CHEST_MODEL: &str = "treasurechest.glb";
/// How far up and down a sword lying bobs, how long a bob takes, and how
/// fast it turns, as a thing lying does (`loot`'s); how long one taken
/// takes to fly to whoever took it, or to sink away, in seconds.
const BOB: f32 = 0.1;
const BOB_SECS: f32 = 2.4;
const SPIN: f32 = 0.9;
const GO_SECS: f32 = 0.18;
/// How far a sword that lay its while sinks as it goes, in metres.
const SINKS: f32 = 1.2;
/// How long a chest takes to pop as it is opened: up to a little bigger,
/// and then down to nothing, in seconds, and how much bigger.
const POP_UP: f32 = 0.1;
const POP_DOWN: f32 = 0.16;
const POP_BIGGER: f32 = 0.3;
/// What was in a chest, rising off where it stood: how big its words are,
/// as a share of the short side of the screen, how high over the chest it
/// begins and how far it rises, in metres, and how long it is up, the last
/// of it fading, in seconds.
const FOUND_TEXT: f32 = 4.2;
const FOUND_OVER: f32 = 1.2;
const FOUND_RISE: f32 = 1.4;
const FOUND_SECS: f32 = 1.8;
const FOUND_FADE: f32 = 0.6;
/// How far past the reach of a thing lying whoever took a sword may be
/// found, in metres: online they are drawn a couple of updates behind.
const TAKER_SLACK: f32 = 2.5;

/// Whether a treasure chest is opened at once by the town's bubble that
/// comes up over it near it (`talk::Opens`): not since the evening of
/// 2026-11-18 (Hajun: "the treasure chest, instead of proximitychatbubble
/// just make it so they have some hp (enough to 1 tap with aura sword, 2
/// tap with other swords, 3 tap with fist) and they explode like a player
/// dying of that item contents"), blows breaking it open now
/// ([`struck`], `rules::Game::hit_chest`); true puts the bubble back.
const BUBBLE_OPENS: bool = false;

/// Whether what was in a chest rises in words over where it stood ("+4
/// Gold", "Silver Sword!"): not since the evening of 2026-11-18, what was
/// in it being thrown out round it, to be seen and walked into, as what
/// the dead drop is; true says it again.
pub(super) const FOUND_WORDS: bool = false;

/// How far out from its middle a treasure chest is to a blow, in metres: a
/// punch lands on it with any of this in its range, as on a body with any
/// of its own (mine, the model's half-width).
const CHEST_RADIUS: f32 = 0.55;

/// How a chest a blow lands on shakes and still stands (mine): for how
/// long, in seconds; how much it squashes down and grows wide, as a share
/// of its size; and how far it rocks about its upright, in radians.
const JOLT_SECS: f32 = 0.24;
const JOLT_SQUASH: f32 = 0.2;
const JOLT_ROCK: f32 = 0.22;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (follow_server, animate, shake, pop, rise_found).chain());
}

/// Which one a thing here is to whoever rules it: the game's own number for
/// it offline, or the server's entity online.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Key {
    Own(u32),
    Server(Entity),
}

/// A sword lying where someone who died dropped it (its model is its
/// tier's): which, where it hangs, where it was thrown from, when it was dropped (by this
/// device's time), and how far round it began.
#[derive(Component)]
pub(super) struct DroppedSword {
    pub(super) key: Key,
    rest: Vec3,
    from: Vec3,
    born: f32,
    phase: f32,
}

/// A sword on its way out of the game: to whoever took it, if anyone is
/// known to have, or sinking away; for how long.
#[derive(Component)]
struct Going {
    to: Option<Entity>,
    age: f32,
}

/// A treasure chest standing: which, and its number in the game (what it is
/// asked to open by), and, opened, how far through its pop it is.
#[derive(Component)]
pub(super) struct ChestLook {
    pub(super) key: Key,
    pub(super) id: u32,
    popping: Option<f32>,
    /// Where and how it stands, to shake about and come back to; and how
    /// much the blows on it had taken as last seen ([`Chest::hits`]).
    rest: Transform,
    hits: u8,
}

/// A chest a blow has just landed on, shaking, for so long so far
/// ([`shake`]).
#[derive(Component)]
struct Jolt(f32);

/// What was in a chest, rising off where it stood.
#[derive(Component)]
struct Found {
    at: Vec3,
    age: f32,
}

/// The sword of `tier` someone who died dropped, lying `rest` over the
/// ground there, thrown from `from`, now: drawn on the fight's venue.
pub(super) fn lay_sword(commands: &mut Commands, assets: &AssetServer, now: f32, key: Key, tier: u8, rest: Vec3, from: Vec3) -> Entity {
    let model = SWORDS[usize::from(tier.clamp(1, 3)) - 1];
    let phase = ((rest.x * 12.9898 + rest.z * 78.233).sin() * 43_758.547).rem_euclid(1.0) * TAU;
    commands
        .spawn((
            DroppedSword {
                key,
                rest,
                from,
                born: now,
                phase,
            },
            InArena,
            Venue::LastStand,
            Name::new("Dropped sword"),
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(model))),
            Transform::from_translation(from).with_scale(SWORD_SCALE),
            // Shown with the island it is on (`island::show_where_i_am`).
            Visibility::Hidden,
        ))
        .id()
}

/// The sword `thing` is gone: taken by `by`, it flies to them; with nobody,
/// it sinks away.
pub(super) fn take_sword(commands: &mut Commands, thing: Entity, by: Option<Entity>) {
    commands.entity(thing).try_insert(Going { to: by, age: 0.0 });
}

/// A treasure chest standing `at`, turned `yaw`, numbered `id` in its game:
/// the bubble comes up over it ([`Opens`]).
pub(super) fn place_chest(commands: &mut Commands, assets: &AssetServer, key: Key, id: u32, at: Vec3, yaw: f32) -> Entity {
    let rest = Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw));
    let mut chest = commands.spawn((
        ChestLook {
            key,
            id,
            popping: None,
            rest,
            hits: 0,
        },
        InArena,
        Venue::LastStand,
        Name::new("Treasure chest"),
        WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(CHEST_MODEL))),
        rest,
        Visibility::Hidden,
    ));
    if BUBBLE_OPENS {
        chest.insert(Opens);
    }
    chest.id()
}

/// A blow has landed on the chest `thing`, and it stands: it shakes
/// ([`shake`]).
pub(super) fn strike_chest(commands: &mut Commands, thing: Entity) {
    commands.entity(thing).try_insert(Jolt(0.0));
}

/// Which treasure chest, of `chests` (each its number and where it stands),
/// a punch of yours thrown from `at`, `feet` up, facing `facing` lands on:
/// the nearest with any of itself ([`CHEST_RADIUS`]) in the range the
/// file's `melee` draws round you (`range::Sector`), as a body is, no
/// further up or down than a body's height.
pub(super) fn struck(at: Vec2, feet: f32, facing: Vec2, chests: impl Iterator<Item = (u32, Vec3)>) -> Option<u32> {
    let sector = crate::range::Sector::of(&ui().melee);
    chests
        .filter_map(|(id, there)| {
            let off = there.xz() - at;
            ((there.y - feet).abs() <= crate::PLAYER_HEIGHT && sector.holds(facing, off, CHEST_RADIUS)).then_some((off.length(), id))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id)
}

/// The chest `thing` is opened, or has stood its while: it pops, and is
/// gone; the bubble no longer comes up over it.
pub(super) fn pop_chest(commands: &mut Commands, thing: Entity) {
    commands.entity(thing).try_remove::<(Opens, Jolt)>().try_insert(Popping);
}

/// A chest popping: marked so, for [`pop`] to begin its pop.
#[derive(Component)]
struct Popping;

/// What was in a chest, said over where it stood, `at`: "+3 Sapphire",
/// "Silver Sword!", "+1 Fireball".
pub(super) fn say_found(commands: &mut Commands, at: Vec3, find: Find) {
    let colours = ui().colours;
    commands.spawn((
        InArena,
        Found { at, age: 0.0 },
        hud::words(found_words(find), FOUND_TEXT, colours.gold),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            ..default()
        },
        UiTransform::default(),
        Visibility::Hidden,
    ));
}

/// What was in a chest, in the file's words.
pub(super) fn found_words(find: Find) -> String {
    let ui = ui();
    let words = &ui.words;
    match find {
        Find::Things { item, count } => {
            let name = match item {
                Item::Silver => &words.arena.silver,
                Item::Gold => &words.arena.gold,
                Item::Sapphire => &words.arena.sapphire,
                Item::Amethyst => &words.arena.amethyst,
            };
            fill(&words.last_stand.found, &[("count", &count.to_string()), ("item", name)])
        }
        Find::Sword(tier) => {
            let name = &words.market.swords[usize::from(tier.clamp(1, 3)) - 1];
            fill(&words.last_stand.found_sword, &[("sword", name)])
        }
        Find::Utility(utility) => {
            let name = match utility {
                Utility::Fireball => &words.market.fireball,
                Utility::Healing => &words.market.healing,
                Utility::Speed => &words.market.speed,
                Utility::Invisibility => &words.market.invisibility,
                Utility::Dynamite => &words.market.dynamite,
                Utility::Leap => &words.market.leap,
                Utility::Builder => &words.market.builder,
            };
            fill(&words.last_stand.found_thing, &[("thing", name)])
        }
    }
}

/// Online, the swords and the chests are the server's: each drawn as it
/// comes, and, gone, a sword flying to whoever is nearest it of those
/// holding a lesser one (whoever the server gave it to), or sinking away
/// once it has lain its while; a chest popping.
#[allow(clippy::type_complexity)]
fn follow_server(
    time: Res<Time>,
    assets: Res<AssetServer>,
    fresh_swords: Query<(Entity, &LyingSword), (With<Remote>, Added<LyingSword>)>,
    fresh_chests: Query<(Entity, &Chest), (With<Remote>, Added<Chest>)>,
    struck_chests: Query<(Entity, &Chest), (With<Remote>, Changed<Chest>)>,
    swords: Query<(), With<LyingSword>>,
    chests: Query<(), With<Chest>>,
    lying: Query<(Entity, &DroppedSword), Without<Going>>,
    mut standing: Query<(Entity, &mut ChestLook), Without<Popping>>,
    people: Query<(Entity, &Transform, &Venue, Has<Player>), (With<crate::WalkCycle>, Without<crate::Dead>, Without<rules::Shambler>)>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();
    for (source, sword) in &fresh_swords {
        let rest = Vec3::new(sword.x, sword.ground + HOVER, sword.z);
        let from = Vec3::new(sword.from_x, sword.from_y, sword.from_z);
        lay_sword(&mut commands, &assets, now, Key::Server(source), sword.tier, rest, from);
    }
    for (source, chest) in &fresh_chests {
        place_chest(&mut commands, &assets, Key::Server(source), chest.id, Vec3::new(chest.x, chest.y, chest.z), chest.yaw);
    }
    for (thing, sword) in &lying {
        let Key::Server(source) = sword.key else {
            continue;
        };
        if swords.contains(source) {
            continue;
        }
        // Lain its while, it sinks; taken, it flies to the nearest of
        // those on the island within a reach of it.
        let lapsed = now - sword.born >= rules::LIES_FOR - 1.0;
        let taker = (!lapsed)
            .then(|| {
                people
                    .iter()
                    .filter(|(_, _, venue, _)| **venue == Venue::LastStand)
                    .map(|(person, at, ..)| (person, at.translation.xz().distance(sword.rest.xz())))
                    .filter(|(_, away)| *away <= MAGNET + TAKER_SLACK)
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(person, _)| person)
            })
            .flatten();
        take_sword(&mut commands, thing, taker);
    }
    for (thing, mut chest) in &mut standing {
        let Key::Server(source) = chest.key else {
            continue;
        };
        if !chests.contains(source) {
            pop_chest(&mut commands, thing);
        } else if let Ok((_, now)) = struck_chests.get(source)
            && now.hits > chest.hits
        {
            // A blow on it that it stood (since the evening of
            // 2026-11-18): it shakes.
            chest.hits = now.hits;
            strike_chest(&mut commands, thing);
        }
    }
}

/// Every chest a blow has just landed on, and that stands: squashed down
/// and spread wide and rocked about its upright, less and less, and back
/// as it stood ([`JOLT_SECS`]).
fn shake(time: Res<Time>, mut chests: Query<(Entity, &ChestLook, &mut Jolt, &mut Transform)>, mut commands: Commands) {
    let dt = time.delta_secs();
    for (entity, chest, mut jolt, mut transform) in &mut chests {
        jolt.0 += dt;
        let done = jolt.0 / JOLT_SECS;
        if done >= 1.0 {
            transform.set_if_neq(chest.rest);
            commands.entity(entity).try_remove::<Jolt>();
            continue;
        }
        let left = 1.0 - done;
        let wave = (done * TAU * 2.0).sin() * left;
        let squash = JOLT_SQUASH * left * left;
        *transform = Transform {
            translation: chest.rest.translation,
            rotation: chest.rest.rotation * Quat::from_rotation_y(JOLT_ROCK * wave),
            scale: chest.rest.scale * Vec3::new(1.0 + squash * 0.5, 1.0 - squash, 1.0 + squash * 0.5),
        };
    }
}

/// Every sword lying: thrown out from whoever dropped it and down, then
/// bobbing and turning, flat side up; and one gone, flying to whoever took
/// it, shrinking, or sinking away.
fn animate(
    time: Res<Time>,
    mut swords: Query<(Entity, &DroppedSword, &mut Transform, Option<&mut Going>), Without<Player>>,
    bodies: Query<&Transform, With<Player>>,
    others: Query<&GlobalTransform>,
    mut commands: Commands,
) {
    let (now, dt) = (time.elapsed_secs(), time.delta_secs());
    for (entity, sword, mut transform, going) in &mut swords {
        let age = (now - sword.born).max(0.0);
        let here = if age < TOSS_SECS { toss(sword.from, sword.rest, age) } else { sword.rest + Vec3::Y * BOB * ((age - TOSS_SECS) * TAU / BOB_SECS).sin() };
        let turn = sword.phase + age * SPIN;
        let lying = pointing(Vec3::new(turn.cos(), 0.0, turn.sin()));
        let Some(mut going) = going else {
            let place = Transform {
                translation: here,
                rotation: lying,
                scale: SWORD_SCALE,
            };
            transform.set_if_neq(place);
            continue;
        };
        going.age += dt;
        let done = (going.age / GO_SECS).min(1.0);
        if done >= 1.0 {
            commands.entity(entity).try_despawn();
            continue;
        }
        let to = going
            .to
            .and_then(|taker| bodies.get(taker).map(|body| body.translation).ok().or_else(|| others.get(taker).ok().map(|at| at.translation())))
            .map_or(here - Vec3::Y * SINKS, |at| at + Vec3::Y * 1.0);
        *transform = Transform {
            translation: here.lerp(to, done * done),
            rotation: lying,
            scale: SWORD_SCALE * (1.0 - 0.75 * done),
        };
    }
}

/// Every chest opened, or stood its while, pops: a little bigger, and then
/// down to nothing, and is gone.
fn pop(time: Res<Time>, mut chests: Query<(Entity, &mut ChestLook, &mut Transform, Has<Popping>)>, mut commands: Commands) {
    let dt = time.delta_secs();
    for (entity, mut chest, mut transform, popping) in &mut chests {
        if !popping {
            continue;
        }
        let age = chest.popping.unwrap_or(0.0) + dt;
        chest.popping = Some(age);
        let size = if age < POP_UP {
            1.0 + POP_BIGGER * (age / POP_UP)
        } else {
            (1.0 + POP_BIGGER) * (1.0 - (age - POP_UP) / POP_DOWN)
        };
        if size <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }
        transform.scale = Vec3::splat(size);
    }
}

/// What was in a chest, rising off where it stood, on the screen over it,
/// and fading at the last.
fn rise_found(
    time: Res<Time>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    mut found: Query<(Entity, &mut Found, &mut Visibility, &mut UiTransform, &ComputedNode, &mut TextColor, &mut TextShadow)>,
    mut commands: Commands,
) {
    let Ok((camera, eye)) = cameras.single() else {
        return;
    };
    let eye = GlobalTransform::from(*eye);
    let (gold, shadow) = (ui().colours.gold, ui().colours.text_shadow);
    for (entity, mut words, mut shown, mut place, node, mut colour, mut under) in &mut found {
        words.age += time.delta_secs();
        if words.age >= FOUND_SECS {
            commands.entity(entity).try_despawn();
            continue;
        }
        let up = FOUND_OVER + FOUND_RISE * (words.age / FOUND_SECS);
        let Ok(point) = camera.world_to_viewport(&eye, words.at + Vec3::Y * up) else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let size = node.size() * node.inverse_scale_factor();
        place.set_if_neq(UiTransform {
            translation: Val2::px(point.x - size.x * 0.5, point.y - size.y * 0.5),
            ..default()
        });
        let solid = ((FOUND_SECS - words.age) / FOUND_FADE).clamp(0.0, 1.0);
        colour.set_if_neq(TextColor(gold.with_alpha(gold.alpha() * solid)));
        under.color = shadow.with_alpha(shadow.alpha() * solid);
        shown.set_if_neq(Visibility::Inherited);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_punch_strikes_the_nearest_chest_in_front_within_its_range_and_none_behind() {
        let (at, feet, facing) = (Vec2::ZERO, 0.0, Vec2::NEG_Y);
        let ahead = |far: f32| Vec3::new(0.0, 0.0, -far);
        // In front, near: struck; the nearer of two.
        assert_eq!(struck(at, feet, facing, [(7, ahead(1.6))].into_iter()), Some(7));
        assert_eq!(struck(at, feet, facing, [(7, ahead(2.4)), (8, ahead(1.4))].into_iter()), Some(8));
        // Behind, beside out of the cone, far beyond the reach, or a body's
        // height up: not.
        assert_eq!(struck(at, feet, facing, [(7, Vec3::new(0.0, 0.0, 1.6))].into_iter()), None);
        assert_eq!(struck(at, feet, facing, [(7, Vec3::new(3.0, 0.0, 0.0))].into_iter()), None);
        assert_eq!(struck(at, feet, facing, [(7, ahead(9.0))].into_iter()), None);
        assert_eq!(struck(at, feet, facing, [(7, ahead(1.6) + Vec3::Y * 2.5)].into_iter()), None);
        // The bubble and the words over it are put away for the blows.
        assert!(!BUBBLE_OPENS && !FOUND_WORDS);
    }

    #[test]
    fn what_was_in_a_chest_is_said_in_the_file_s_words() {
        assert_eq!(found_words(Find::Things { item: Item::Sapphire, count: 3 }), "+3 Sapphire");
        assert_eq!(found_words(Find::Sword(2)), "Silver Sword!");
        assert_eq!(found_words(Find::Utility(Utility::Fireball)), "+1 Fireball");
        // Every word it says is one the fonts draw.
        for find in [
            Find::Things { item: Item::Silver, count: 25 },
            Find::Things { item: Item::Amethyst, count: 1 },
            Find::Sword(1),
            Find::Sword(3),
            Find::Utility(Utility::Healing),
        ] {
            assert!(found_words(find).chars().all(roundtown_net::text::allowed), "{find:?}");
        }
        // A sword lies for the taking no sooner than a thing does.
        assert!(roundtown_net::bridge_wars::SPILL_WAIT > TOSS_SECS);
    }
}
