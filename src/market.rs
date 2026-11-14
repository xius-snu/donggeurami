//! Bridge Wars' two shops, at the market stands on each island (the item
//! shop since 2026-11-11, selling only the bridge; both, with everything
//! in Hajun's `bridgewarsgameevent.txt`, since 2026-11-12): the **item
//! shop** at the red-roofed stand, which sells what is yours alone, the
//! bridge, three swords, three shields and five things for the second
//! button; and the **upgrade shop** at the blue one, which sells what is
//! your whole team's, a faster generator, stronger swords, better shields,
//! a heal pool and blindness traps.
//!
//! **Where the shops are is the map's**: Hajun drew a white outline on the
//! ground in front of every stand, in a material called `activation`
//! ([`ACTIVATION_MATERIAL`]); the one at a stand roofed in
//! [`RED_ROOF_MATERIAL`] is an item shop and the one at a stand roofed in
//! [`BLUE_ROOF_MATERIAL`] an upgrade shop ([`shop_zones`], read once the
//! map has spawned). Move a stand and its outline in Blender and the shop
//! follows. The server has no map, and sells only to someone near where
//! the shops stood when it was built (`roundtown_net::ARENA_SHOPS`,
//! `ARENA_UPGRADE_SHOPS`): a test keeps the two together.
//!
//! **Standing in an outline opens its shop** ([`watch`]): a window over
//! the middle of the screen with a card for everything it sells (Hajun:
//! "you may organize shoplayout in your own way, but keep it easy for
//! people to be able to understand right away without too much text").
//! A card is a picture, a name, one short line of what it does, and its
//! price, the picture of what it is paid in and how many; the swords, the
//! shields and the things for the second button are a row each, so that a
//! name is a word. What cannot be bought is blocked out, and says why in
//! place of a price: yours already, no more of it to have, or the second
//! button holding something else; what you cannot pay for is blocked out
//! with its price. An upgrade that goes by steps is one card, which shows
//! the next step, with a pip for each step taken. All of it white on
//! grey, but for the pictures, which are tinted as the things are.
//!
//! It is not a dialog: the fight goes on round it and you can walk, and
//! only a press on it is its own. A tap on a card buys it, at once
//! ([`press`], `arena::Want`): offline out of what you hold, by the
//! shops' rules; online asked of the server, which takes the price and
//! sends what everyone has. The window is put up again as what you hold
//! or have changes, which is how a card you can now pay for lights up.
//! The cross on its corner shuts it, and so does a press anywhere else on
//! the screen, which goes on to do whatever else it does, the stick
//! included ([`shut_by_hand`]), and Escape; shut that way it stays shut
//! until you have stepped out of the outline and in again. Stepping out
//! of it shuts it too, as does dying, and the game's end.

use bevy::gltf::GltfMaterialName;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use roundtown_net::bridge_wars::{self as rules, Shop, price};
use roundtown_net::{Item, Purse, Utility, Ware};

use crate::arena::{Fighting, InArena, Outfit, Resources, Want};
use crate::builder::{self, Fill};
use crate::hud::{self, DialogUp, Picture, Pictures, TakesPress};
use crate::island::{self, Land, Venue, is_material};
use crate::ui::{fill, ui};
use crate::{Dead, Player};

/// What the outlines on the ground in front of the stands are made of in
/// the map, and what the stands of the two shops are roofed with.
pub(crate) const ACTIVATION_MATERIAL: &str = "activation";
pub(crate) const RED_ROOF_MATERIAL: &str = "RoofTiles_Red";
pub(crate) const BLUE_ROOF_MATERIAL: &str = "bluemarket";
/// How far apart two corners can be and still be one outline's, or one
/// roof's, in metres: an outline is 1.5 m by 2.5, its long sides drawn
/// with a corner at each end and none between, and the two on an island
/// are 8.5 m apart.
const ONE_THING: f32 = 3.0;
/// How far the middle of a shop's outline can be from the middle of its
/// stand's roof, in metres: the outline lies under the front of it, 0.3 m
/// off; the other stand on the island is 10 m away.
const AT_ITS_STAND: f32 = 2.0;
/// How far over or under the islands' top your feet may be and still be
/// standing in an outline, in metres: not jumping over it.
const ON_THE_GROUND: f32 = 0.6;
/// Sizes, as shares of the short side of the screen: how far down the
/// screen each shop's window begins, as a share of the screen's height,
/// the item shop's, which is three rows, higher, both under the time and
/// the line under it; the window's title; a card, its picture, its name,
/// the line under it, its price with the picture of what it is paid in,
/// and a pip; the gap between cards and the window's own margin; and the
/// cross on its corner. An upgrade's card is taller: its name is two
/// words, on two lines, and it has its pips besides.
const TOP_ITEMS: f32 = 16.0;
const TOP_UPGRADES: f32 = 24.0;
const TITLE: f32 = 3.6;
const CARD_WIDTH: f32 = 15.5;
/// A card in a row of six, which the things for the buttons are since the
/// dynamite (2026-11-13): narrower, so that the row ends short of the
/// buttons on the right of a screen no wider than sixteen by nine, where
/// six cards of the other width reached under the second button.
const NARROW_CARD: f32 = 14.0;
const CARD_HEIGHT: f32 = 17.2;
const UPGRADE_CARD_HEIGHT: f32 = 21.5;
const PICTURE: f32 = 6.0;
const NAME: f32 = 2.3;
const STAT: f32 = 2.0;
const PRICE: f32 = 2.7;
const PRICE_ICON: f32 = 2.9;
const PIP: f32 = 1.1;
const GAP: f32 = 1.1;
const PAD: f32 = 1.8;
const CLOSE: f32 = 6.5;

/// Where the shops are, as the map has them: each the box round an
/// outline, across the ground, as its lowest and highest corners.
#[derive(Resource, Default, Debug)]
pub(crate) struct Shops {
    pub(crate) items: Vec<(Vec2, Vec2)>,
    pub(crate) upgrades: Vec<(Vec2, Vec2)>,
}

impl Shops {
    /// Which shop someone whose feet are at `at` stands in, if either.
    pub(crate) fn at(&self, at: Vec3) -> Option<Shop> {
        if in_a_shop(&self.items, at) {
            Some(Shop::Items)
        } else if in_a_shop(&self.upgrades, at) {
            Some(Shop::Upgrades)
        } else {
            None
        }
    }
}

/// Which shop's outline you are standing in, and whether you have shut it
/// by hand since you stepped into it.
#[derive(Resource, Default)]
struct Counter {
    inside: Option<Shop>,
    shut: bool,
}

/// A shop's window, and which shop's.
#[derive(Component)]
struct ShopWindow(Shop);
/// A card, and what a tap on it buys.
#[derive(Component)]
pub(crate) struct Card(pub(crate) Ware);
/// The cross on the window's corner.
#[derive(Component)]
struct CloseShop;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Counter>()
        .add_observer(read_map)
        .add_observer(press)
        .add_systems(Update, (shut_by_hand, watch).chain().after(crate::move_bodies));
}

/// Reads the shops off the map once it has spawned: every corner of every
/// outline and of every red and every blue roof, where the map puts them.
fn read_map(
    ready: On<WorldInstanceReady>,
    lands: Query<&Venue, With<Land>>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, &GltfMaterialName)>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    mut commands: Commands,
) {
    // The map's own model, not a body's on it.
    if lands.get(ready.entity).ok() != Some(&Venue::Arena) {
        return;
    }
    let (mut outlines, mut red, mut blue) = (Vec::new(), Vec::new(), Vec::new());
    for part in children.iter_descendants(ready.entity) {
        let Ok((mesh, material)) = parts.get(part) else {
            continue;
        };
        let corners = if is_material(&material.0, ACTIVATION_MATERIAL) {
            &mut outlines
        } else if is_material(&material.0, RED_ROOF_MATERIAL) {
            &mut red
        } else if is_material(&material.0, BLUE_ROOF_MATERIAL) {
            &mut blue
        } else {
            continue;
        };
        let (Some(mesh), Ok(place)) = (meshes.get(&mesh.0), placement.compute_global_transform(part)) else {
            continue;
        };
        let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            continue;
        };
        corners.extend(points.iter().map(|&point| place.transform_point(Vec3::from(point)).xz()));
    }
    let shops = Shops {
        items: shop_zones(&outlines, &red),
        upgrades: shop_zones(&outlines, &blue),
    };
    if shops.items.is_empty() || shops.upgrades.is_empty() {
        warn!(
            "Bridge Wars' map has {} outlines in {ACTIVATION_MATERIAL:?} at stands roofed in {RED_ROOF_MATERIAL:?} and {} at stands in {BLUE_ROOF_MATERIAL:?}",
            shops.items.len(),
            shops.upgrades.len()
        );
    } else {
        info!("Bridge Wars' shops: {} item shops and {} upgrade shops", shops.items.len(), shops.upgrades.len());
    }
    commands.insert_resource(shops);
}

/// The shops at the stands with these `roofs`, given every corner of every
/// outline on the ground and of every such roof, across the ground: the
/// box round each outline whose middle is at the middle of one of them.
pub(crate) fn shop_zones(outlines: &[Vec2], roofs: &[Vec2]) -> Vec<(Vec2, Vec2)> {
    let middle = |(low, high): &(Vec2, Vec2)| (*low + *high) * 0.5;
    let roofs: Vec<Vec2> = boxes(roofs).iter().map(middle).collect();
    boxes(outlines)
        .into_iter()
        .filter(|outline| roofs.iter().any(|roof| roof.distance(middle(outline)) <= AT_ITS_STAND))
        .collect()
}

/// The box round each lot of `corners` no more than [`ONE_THING`] apart,
/// as its lowest and highest corners.
fn boxes(corners: &[Vec2]) -> Vec<(Vec2, Vec2)> {
    let mut boxes: Vec<(Vec2, Vec2)> = Vec::new();
    for &corner in corners {
        let near = boxes
            .iter_mut()
            .find(|(low, high)| corner.cmpge(*low - ONE_THING).all() && corner.cmple(*high + ONE_THING).all());
        match near {
            Some((low, high)) => {
                *low = low.min(corner);
                *high = high.max(corner);
            }
            None => boxes.push((corner, corner)),
        }
    }
    // Two lots that grew to within reach of each other are one thing's.
    let mut merged = true;
    while merged {
        merged = false;
        'all: for i in 0..boxes.len() {
            for j in i + 1..boxes.len() {
                let ((a, b), (c, d)) = (boxes[i], boxes[j]);
                if (a - ONE_THING).cmple(d).all() && (c - ONE_THING).cmple(b).all() {
                    boxes[i] = (a.min(c), b.max(d));
                    boxes.swap_remove(j);
                    merged = true;
                    break 'all;
                }
            }
        }
    }
    boxes
}

/// Whether someone whose feet are at `at` stands in one of `zones`.
pub(crate) fn in_a_shop(zones: &[(Vec2, Vec2)], at: Vec3) -> bool {
    (at.y - island::LAND_TOP_OF_THE_ISLANDS).abs() <= ON_THE_GROUND
        && zones
            .iter()
            .any(|(low, high)| at.xz().cmpge(*low).all() && at.xz().cmple(*high).all())
}

/// Opens a shop as you step into its outline in a game that is on, and
/// shuts it as you step out of it, die, or the game ends; shut by hand
/// ([`shut_by_hand`], [`press`]), it stays shut until you have stepped
/// out and in again. Up, it is put up again whenever what you hold or
/// have changes, so that every card says how it stands.
#[allow(clippy::too_many_arguments)]
fn watch(
    fighting: Res<Fighting>,
    shops: Option<Res<Shops>>,
    dialog: Res<DialogUp>,
    stock: Res<Resources>,
    outfit: Res<Outfit>,
    me: Query<(&Transform, &Venue, Has<Dead>), With<Player>>,
    windows: Query<(Entity, &ShopWindow)>,
    mut counter: ResMut<Counter>,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
    mut commands: Commands,
) {
    let inside = (fighting.playing && !dialog.0)
        .then(|| {
            let (at, &venue, dead) = me.single().ok()?;
            (venue == Venue::Arena && !dead).then(|| shops.as_ref()?.at(at.translation))?
        })
        .flatten();
    let up = windows.iter().next();
    let Some(shop) = inside else {
        if let Some((window, _)) = up {
            commands.entity(window).despawn();
        }
        if counter.inside.is_some() || counter.shut {
            *counter = Counter::default();
        }
        return;
    };
    if counter.inside != Some(shop) {
        *counter = Counter {
            inside: Some(shop),
            shut: false,
        };
    }
    if counter.shut {
        if let Some((window, _)) = up {
            commands.entity(window).despawn();
        }
        return;
    }
    let fresh = up.is_some_and(|(_, which)| which.0 == shop) && !stock.is_changed() && !outfit.is_changed();
    if fresh {
        return;
    }
    if let Some((window, _)) = up {
        commands.entity(window).despawn();
    }
    open(&mut commands, &mut images, &mut pictures, shop, &outfit, &stock.held);
}

/// What each shop sells, a row of cards to a kind of thing.
fn shelves(shop: Shop) -> Vec<Vec<Ware>> {
    match shop {
        // What builds bridges, and the swords; the shields, and the
        // tracker; and what is thrown, then what is drunk. (Seven things
        // go on the buttons since the leap potion, 2026-11-14: a row of
        // seven cards would reach under the buttons on the right, so the
        // Bridge Builder stands beside the bridge it builds.)
        Shop::Items => vec![
            vec![Ware::Bridge, Ware::Utility(Utility::Builder), Ware::Sword(1), Ware::Sword(2), Ware::Sword(3)],
            vec![Ware::Shield(1), Ware::Shield(2), Ware::Shield(3), Ware::Compass],
            [
                Utility::Fireball,
                Utility::Dynamite,
                Utility::Speed,
                Utility::Leap,
                Utility::Invisibility,
                Utility::Healing,
            ]
            .into_iter()
            .map(Ware::Utility)
            .collect(),
        ],
        Shop::Upgrades => vec![vec![
            Ware::Forge,
            Ware::StrongerSwords,
            Ware::BetterShields,
            Ware::HealPool,
            Ware::Trap,
            Ware::BuildSpeed,
        ]],
    }
}

/// How a card stands for you: to be bought, for so many of an item, which
/// you have or have not; yours already; with no more of it to have; or,
/// for something that goes on a button, with every button taken by
/// another kind.
#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    Buy { item: Item, cost: u16, able: bool },
    Owned,
    Max,
    Taken,
}

/// Everything a card shows: what it sells; its picture and the colour that
/// is tinted; its name and the line under it; how many of its steps have
/// been taken, of how many, for an upgrade that goes by steps; how many
/// of it are on your second button; and how it stands for you.
#[derive(Clone, PartialEq, Debug)]
struct Offer {
    ware: Ware,
    picture: Picture,
    tint: Color,
    name: String,
    stat: String,
    pips: Option<(u8, u8)>,
    held: u8,
    state: State,
}

/// The picture of something for the second button, and the colour it is
/// tinted: on its card here, and on the button (`utility`). The
/// dynamite's is in two colours of its own, and tinted white, which
/// leaves it as it is (`hud::Picture::in_its_own_colours`).
pub(crate) fn look_of(utility: Utility) -> (Picture, Color) {
    let colours = ui().colours;
    match utility {
        Utility::Fireball => (Picture::Fireball, colours.arena.fireball),
        Utility::Builder => (Picture::Builder, colours.hud.icon),
        Utility::Speed => (Picture::Flask, colours.arena.potion_speed),
        Utility::Invisibility => (Picture::Flask, colours.arena.potion_invisibility),
        Utility::Healing => (Picture::Flask, colours.arena.potion_healing),
        Utility::Dynamite => (Picture::Dynamite, Color::WHITE),
        Utility::Leap => (Picture::Flask, colours.arena.potion_leap),
    }
}

/// The card for `ware`, as it stands for someone with `outfit` holding
/// `held`: its words out of the file, its numbers out of the rules.
fn offer(ware: Ware, outfit: &Outfit, held: &Purse) -> Offer {
    let file = ui();
    let (words, colours) = (&file.words.market, file.colours);
    let (kit, team) = (&outfit.kit, &outfit.team);
    let whole = |number: f32| format!("{number:.0}");
    let percent = |share: f32| format!("{:.0}", share * 100.0);
    let tier = |tier: u8| usize::from(tier.clamp(1, 3));
    let (picture, tint, name, stat, pips) = match ware {
        Ware::Bridge => (
            Picture::Plank,
            colours.hud.icon,
            words.bridge.clone(),
            fill(&words.bridge_stat, &[("metres", &whole(rules::BRIDGE_A_BUY))]),
            None,
        ),
        Ware::Sword(sword) => (
            Picture::Sword,
            colours.arena.sword(sword),
            words.swords[tier(sword) - 1].clone(),
            fill(&words.sword_stat, &[("damage", &whole(rules::SWORD_DAMAGE[tier(sword)]))]),
            None,
        ),
        Ware::Shield(shield) => (
            Picture::Shield,
            // The ball of force's own colour, solid.
            colours.arena.shield(shield).with_alpha(1.0),
            words.shields[tier(shield) - 1].clone(),
            fill(&words.shield_stat, &[("share", &percent(rules::SHIELD_SHARE[tier(shield)]))]),
            None,
        ),
        Ware::Utility(utility) => {
            let (picture, tint) = look_of(utility);
            let (name, stat) = match utility {
                Utility::Fireball => (
                    &words.fireball,
                    fill(&words.fireball_stat, &[("damage", &whole(rules::FIREBALL_DAMAGE))]),
                ),
                Utility::Builder => (
                    &words.builder,
                    fill(&words.builder_stat, &[("metres", &whole(rules::BUILDER_METRES))]),
                ),
                Utility::Speed => (
                    &words.speed,
                    fill(
                        &words.speed_stat,
                        &[("more", &percent(rules::FAST - 1.0)), ("seconds", &whole(rules::FAST_SECS))],
                    ),
                ),
                Utility::Invisibility => (
                    &words.invisibility,
                    fill(&words.invisibility_stat, &[("seconds", &whole(rules::UNSEEN_SECS))]),
                ),
                Utility::Healing => (&words.healing, fill(&words.healing_stat, &[("share", &percent(rules::HEALING))])),
                Utility::Dynamite => (
                    &words.dynamite,
                    fill(&words.dynamite_stat, &[("damage", &whole(rules::DYNAMITE_DAMAGE))]),
                ),
                Utility::Leap => (
                    &words.leap,
                    fill(
                        &words.leap_stat,
                        &[("times", &whole(rules::LEAP)), ("seconds", &whole(rules::LEAP_SECS))],
                    ),
                ),
            };
            (picture, tint, name.clone(), stat, None)
        }
        Ware::Forge => {
            // The next step, or, with every step taken, the last.
            let step = usize::from(team.forge.min(rules::FORGE_MOST - 1));
            let tint = match step {
                2 => colours.arena.item_amethyst,
                3 => colours.arena.fireball,
                _ => colours.hud.icon,
            };
            (
                Picture::Forge,
                tint,
                words.forge[step].clone(),
                words.forge_stats[step].clone(),
                Some((team.forge, rules::FORGE_MOST)),
            )
        }
        Ware::StrongerSwords => (
            Picture::SwordPlus,
            colours.hud.icon,
            words.stronger_swords.clone(),
            fill(&words.stronger_swords_stat, &[("damage", &whole(rules::STRONGER_SWORDS))]),
            None,
        ),
        Ware::BetterShields => (
            Picture::ShieldPlus,
            colours.hud.icon,
            words.better_shields.clone(),
            fill(&words.better_shields_stat, &[("share", &percent(rules::BETTER_SHIELDS))]),
            Some((team.shields, rules::BETTER_SHIELDS_MOST)),
        ),
        Ware::HealPool => (
            Picture::Cross,
            colours.arena.heal_pool,
            words.heal_pool.clone(),
            fill(
                &words.heal_pool_stat,
                &[("health", &whole(rules::HEAL_POOL)), ("seconds", &format!("{}", rules::HEAL_POOL_EVERY))],
            ),
            None,
        ),
        Ware::Trap => (
            Picture::Eye,
            colours.arena.trap,
            words.trap.clone(),
            fill(&words.trap_stat, &[("seconds", &whole(rules::BLIND_SECS))]),
            Some((team.traps, rules::TRAPS_MOST)),
        ),
        Ware::Compass => (
            Picture::Compass,
            colours.arena.compass.with_alpha(1.0),
            words.compass.clone(),
            words.compass_stat.clone(),
            None,
        ),
        Ware::BuildSpeed => {
            // The next step, or, with every step taken, the last: how
            // fast the team builds once it has it.
            let step = usize::from(team.build_speed.min(rules::BUILD_SPEED_MOST - 1));
            (
                Picture::BuilderPlus,
                colours.hud.icon,
                words.build_speed[step].clone(),
                fill(&words.build_speed_stat, &[("percent", &percent(rules::BUILD_PACE[step + 1]))]),
                Some((team.build_speed, rules::BUILD_SPEED_MOST)),
            )
        }
    };
    let state = match price(ware, kit, team) {
        Some((item, cost)) => State::Buy {
            item,
            cost,
            able: held[item.index()] >= cost,
        },
        None => match ware {
            Ware::Sword(_) | Ware::Shield(_) | Ware::StrongerSwords | Ware::HealPool | Ware::Compass => State::Owned,
            // None of it, and not to be had: every button holds another
            // kind (five buttons since 2026-11-13; with the one there was,
            // any other kind on it).
            Ware::Utility(utility) if kit.slot_of(utility).is_none() => State::Taken,
            _ => State::Max,
        },
    };
    let on_the_button = match ware {
        Ware::Utility(utility) => kit.held(utility),
        _ => 0,
    };
    Offer {
        ware,
        picture,
        tint,
        name,
        stat,
        pips,
        held: on_the_button,
        state,
    }
}

/// How wide a card is in a row of `cards` of them, as a share of the short
/// side of the screen.
fn card_width(cards: usize) -> f32 {
    if cards > 5 { NARROW_CARD } else { CARD_WIDTH }
}

/// How wide a row of `cards` of them makes the window, margins and all.
#[cfg(test)]
fn row_width(cards: usize) -> f32 {
    cards as f32 * card_width(cards) + cards.saturating_sub(1) as f32 * GAP + 2.0 * PAD
}

/// A shop's window: its title, a row of cards to each of its shelves, and
/// the cross on its corner.
fn open(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    pictures: &mut Pictures,
    shop: Shop,
    outfit: &Outfit,
    held: &Purse,
) {
    let file = ui();
    let (colours, words) = (file.colours, &file.words.market);
    let (title, top) = match shop {
        Shop::Items => (&words.items, TOP_ITEMS),
        Shop::Upgrades => (&words.upgrades, TOP_UPGRADES),
    };
    commands
        .spawn((
            InArena,
            ShopWindow(shop),
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Vh(top),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(GAP),
                padding: UiRect::all(Val::VMin(PAD)),
                border_radius: BorderRadius::all(Val::VMin(3.0)),
                ..default()
            },
            // Half its own width back, so that it is centred.
            UiTransform::from_translation(Val2::new(Val::Percent(-50.0), Val::ZERO)),
            BackgroundColor(colours.shop.window),
            // Under a dialog, and under the line that says what the
            // game's timetable has just done (`arena::alert`).
            GlobalZIndex(builder::DIALOG_LAYER - 2),
        ))
        .with_children(|window| {
            window.spawn(builder::label(title.clone(), TITLE, colours.text));
            for shelf in shelves(shop) {
                window
                    .spawn((
                        Node {
                            column_gap: Val::VMin(GAP),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|row| {
                        let height = match shop {
                            Shop::Items => CARD_HEIGHT,
                            Shop::Upgrades => UPGRADE_CARD_HEIGHT,
                        };
                        let width = card_width(shelf.len());
                        for ware in shelf {
                            card(row, images, pictures, &offer(ware, outfit, held), width, height);
                        }
                    });
            }
            window.spawn(hud::close_cross(CLOSE, CloseShop));
        });
}

/// One card, `width` wide and `height` tall: the picture, the name, the
/// line under it, the pips of an upgrade that goes by steps, and the
/// price, or why there is none. Lit under the pointer only if it can be
/// bought; blocked out otherwise.
fn card(
    row: &mut ChildSpawnerCommands,
    images: &mut Assets<Image>,
    pictures: &mut Pictures,
    offer: &Offer,
    width: f32,
    height: f32,
) {
    let file = ui();
    let (colours, words) = (file.colours, &file.words.market);
    let able = matches!(offer.state, State::Buy { able: true, .. });
    let (look, ink, soft, tint) = if able {
        (
            Fill {
                idle: colours.shop.card,
                lit: colours.shop.card_lit,
            },
            colours.text,
            colours.text_soft,
            offer.tint,
        )
    } else {
        (
            Fill {
                idle: colours.shop.card_off,
                lit: colours.shop.card_off,
            },
            colours.shop.name_off,
            colours.shop.name_off,
            // Yours already, it keeps its colour; out of reach, it is
            // blocked out with the rest.
            if offer.state == State::Owned { offer.tint } else { offer.tint.with_alpha(offer.tint.alpha() * 0.35) },
        )
    };
    row.spawn((
        Card(offer.ware),
        TakesPress,
        Button,
        Node {
            width: Val::VMin(width),
            height: Val::VMin(height),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::axes(Val::VMin(0.6), Val::VMin(1.0)),
            border_radius: BorderRadius::all(Val::VMin(2.0)),
            ..default()
        },
        BackgroundColor(look.idle),
        look,
    ))
    .with_children(|card| {
        card.spawn((
            ImageNode::new(pictures.of(images, offer.picture)).with_color(tint),
            Node {
                width: Val::VMin(PICTURE),
                height: Val::VMin(PICTURE),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
        ));
        card.spawn((
            builder::label(offer.name.clone(), NAME, ink),
            Node {
                max_width: Val::Percent(100.0),
                ..default()
            },
        ));
        card.spawn(builder::label(offer.stat.clone(), STAT, soft));
        // Its steps, a pip to each: those taken lit.
        if let Some((taken, of)) = offer.pips {
            {
                card.spawn((
                    Node {
                        column_gap: Val::VMin(PIP * 0.6),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|pips| {
                    for pip in 0..of {
                        pips.spawn((
                            Node {
                                width: Val::VMin(PIP),
                                height: Val::VMin(PIP),
                                border_radius: BorderRadius::all(Val::Percent(50.0)),
                                ..default()
                            },
                            BackgroundColor(if pip < taken { colours.arena.pip } else { colours.arena.pip_off }),
                            Pickable::IGNORE,
                        ));
                    }
                });
            }
        }
        match offer.state {
            State::Buy { item, cost, .. } => {
                card.spawn((
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::VMin(0.8),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|price| {
                    let coin = colours.arena.item(item);
                    price.spawn((
                        ImageNode::new(images.add(hud::item_picture(item)))
                            .with_color(if able { coin } else { coin.with_alpha(0.4) }),
                        Node {
                            width: Val::VMin(PRICE_ICON),
                            height: Val::VMin(PRICE_ICON),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    price.spawn(builder::label(cost.to_string(), PRICE, ink));
                });
            }
            State::Owned => {
                card.spawn(builder::label(words.owned.clone(), STAT, soft));
            }
            State::Max => {
                card.spawn(builder::label(words.max.clone(), STAT, soft));
            }
            State::Taken => {
                card.spawn(builder::label(words.taken.clone(), STAT, soft));
            }
        }
        // How many of it are on your second button, in the card's corner.
        if offer.held > 0 {
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::VMin(1.0),
                    top: Val::VMin(0.6),
                    ..default()
                },
                Pickable::IGNORE,
                children![builder::label(format!("x{}", offer.held), STAT, colours.text)],
            ));
        }
    });
}

/// A tap on a card asks for what it sells, at once (`arena::sell`, which
/// knows whether you can have it); a tap on the cross shuts the shop.
fn press(
    click: On<Pointer<Click>>,
    cards: Query<&Card>,
    closes: Query<(), With<CloseShop>>,
    fighting: Res<Fighting>,
    mut counter: ResMut<Counter>,
    mut wants: MessageWriter<Want>,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if closes.contains(click.entity) {
        counter.shut = true;
        return;
    }
    if let Ok(card) = cards.get(click.entity)
        && fighting.playing
    {
        wants.write(Want(card.0));
    }
}

/// Shuts the shop at a press anywhere on the screen but on its window, a
/// finger's or the mouse's, and at Escape. The press is not taken: it goes
/// on to whatever it was on, so that touching the stick walks you away.
fn shut_by_hand(
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    screens: Query<&Window>,
    windows: Query<(&ComputedNode, &UiGlobalTransform), With<ShopWindow>>,
    mut counter: ResMut<Counter>,
) {
    let (Ok((node, place)), Ok(screen)) = (windows.single(), screens.single()) else {
        return;
    };
    // Not before it has been laid out: it is nowhere yet.
    if node.size().min_element() <= 0.0 {
        return;
    }
    let mut presses: Vec<Vec2> = touches.iter_just_pressed().map(|touch| touch.position()).collect();
    if mouse.just_pressed(MouseButton::Left)
        && let Some(cursor) = screen.cursor_position()
    {
        presses.push(cursor);
    }
    let elsewhere = presses
        .iter()
        .any(|&press| !node.contains_point(*place, press * screen.scale_factor()));
    if elsewhere || keyboard.just_pressed(KeyCode::Escape) {
        counter.shut = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roundtown_net::{ARENA_ISLAND_RADIUS, ARENA_ISLANDS, ARENA_SHOPS, ARENA_TEAMS, ARENA_UPGRADE_SHOPS};

    /// Every corner of the map's parts in `material`, across the ground.
    fn corners_of(parts: &[island::glb::Part], material: &str) -> Vec<Vec2> {
        parts
            .iter()
            .filter(|part| part.material.as_deref().is_some_and(|name| is_material(name, material)))
            .flat_map(|part| part.triangles.iter().flatten().map(|corner| corner.xz()))
            .collect()
    }

    #[test]
    fn the_shops_are_the_outlines_at_the_red_and_the_blue_stands_where_the_server_has_them() {
        let parts = island::glb::read("pvptownmap.glb");
        let outlines = corners_of(&parts, ACTIVATION_MATERIAL);
        let roofs = corners_of(&parts, RED_ROOF_MATERIAL);
        let blue = corners_of(&parts, BLUE_ROOF_MATERIAL);
        // Two outlines to an island, one at a red stand and one at a blue.
        assert_eq!(boxes(&outlines).len(), 2 * ARENA_TEAMS);
        assert_eq!(boxes(&roofs).len(), ARENA_TEAMS);
        assert_eq!(boxes(&blue).len(), ARENA_TEAMS);
        let zones = shop_zones(&outlines, &roofs);
        assert_eq!(zones.len(), ARENA_TEAMS, "{zones:?}");
        let upgrades = shop_zones(&outlines, &blue);
        assert_eq!(upgrades.len(), ARENA_TEAMS, "{upgrades:?}");
        let shops = Shops {
            items: zones.clone(),
            upgrades: upgrades.clone(),
        };
        let land: Vec<[Vec3; 3]> = parts
            .iter()
            .filter(|part| !part.skinned)
            .flat_map(|part| part.triangles.iter().copied())
            .collect();
        let map = island::Island::new(&land);
        for team in 0..ARENA_TEAMS {
            // Where the server takes each team's shop to be is the middle
            // of an outline, 1.5 m by 2.5, on that team's island.
            let shop = ARENA_SHOPS[team];
            let (low, high) = zones
                .iter()
                .find(|(low, high)| ((*low + *high) * 0.5).distance(shop) < 0.1)
                .unwrap_or_else(|| panic!("team {team}: no outline at {shop}, of {zones:?}"));
            let size = *high - *low;
            assert!((size.min_element() - 1.5).abs() < 0.1 && (size.max_element() - 2.49).abs() < 0.1, "{size}");
            assert!(shop.distance(ARENA_ISLANDS[team]) < ARENA_ISLAND_RADIUS - 2.0);
            // Someone can stand in it: there is ground under its nearer
            // half, which the stand's counter leaves clear, and standing
            // there is standing in the shop; and nobody arrives in one,
            // nor comes back up in one, nor is one where a zombie rises.
            let nearer = shop + (ARENA_ISLANDS[team] - shop).normalize() * 0.45;
            let ground = map.floor(nearer, 1.0).expect("ground in the outline");
            assert!((ground - island::LAND_TOP_OF_THE_ISLANDS).abs() < 0.03, "{ground}");
            let body = [(crate::PLAYER_RADIUS, ground + crate::LAND_STEP_UP, ground + crate::PLAYER_HEIGHT)];
            assert!(map.walls_round(nearer, &body).is_empty(), "team {team}: the stand fills its own outline");
            assert!(in_a_shop(&zones, Vec3::new(nearer.x, ground, nearer.y)));
            assert!(!in_a_shop(&zones, Vec3::new(nearer.x, ground + 1.5, nearer.y)), "jumping over it");
            assert_eq!(shops.at(Vec3::new(nearer.x, ground, nearer.y)), Some(Shop::Items));
            // And its upgrade shop the same, at the blue stand across the
            // island's middle: an outline of its own, with room to stand.
            let shop = ARENA_UPGRADE_SHOPS[team];
            assert!(
                upgrades.iter().any(|(low, high)| ((*low + *high) * 0.5).distance(shop) < 0.1),
                "team {team}: no outline at {shop}, of {upgrades:?}"
            );
            let nearer = shop + (ARENA_ISLANDS[team] - shop).normalize() * 0.45;
            let ground = map.floor(nearer, 1.0).expect("ground in the outline");
            let body = [(crate::PLAYER_RADIUS, ground + crate::LAND_STEP_UP, ground + crate::PLAYER_HEIGHT)];
            assert!(map.walls_round(nearer, &body).is_empty(), "team {team}: the blue stand fills its own outline");
            assert_eq!(shops.at(Vec3::new(nearer.x, ground, nearer.y)), Some(Shop::Upgrades));
            assert_eq!(shops.at(Vec3::new(ARENA_ISLANDS[team].x, ground, ARENA_ISLANDS[team].y)), None);
        }
        for seat in 0..roundtown_net::ARENA_SEATS {
            let at = island::arrival(Venue::Arena, seat).translation;
            assert!(!in_a_shop(&zones, at.with_y(island::LAND_TOP_OF_THE_ISLANDS)), "seat {seat} arrives in a shop");
        }
        for zombie in 0..roundtown_net::bridge_wars::ZOMBIES {
            let at = roundtown_net::bridge_wars::zombie_spot(zombie);
            assert!(!in_a_shop(&zones, Vec3::new(at.x, island::LAND_TOP_OF_THE_ISLANDS, at.y)), "zombie {zombie}");
        }
    }

    #[test]
    fn an_outline_with_no_red_roof_at_it_is_no_shop() {
        let outline = |x: f32| vec![Vec2::new(x - 0.75, -1.25), Vec2::new(x + 0.75, -1.25), Vec2::new(x + 0.75, 1.25), Vec2::new(x - 0.75, 1.25)];
        let mut outlines = outline(5.0);
        outlines.extend(outline(-5.0));
        let roof = vec![Vec2::new(4.7, -1.4), Vec2::new(5.9, 1.4)];
        let zones = shop_zones(&outlines, &roof);
        assert_eq!(zones, vec![(Vec2::new(4.25, -1.25), Vec2::new(5.75, 1.25))]);
        assert!(shop_zones(&outlines, &[]).is_empty());
        assert!(shop_zones(&[], &roof).is_empty());
    }

    #[test]
    fn every_card_says_how_it_stands_in_a_word_or_a_price() {
        use roundtown_net::{Kit, Upgrades};
        let bare = Outfit::default();
        // Every ware of both shops is on a shelf once, the item shop's in
        // three rows and the upgrade shop's in one, no row longer than
        // six cards: a row of six is of narrower cards, so that it is no
        // more than a card's width wider than a row of five was, and
        // ends short of the second button on a screen of sixteen by nine.
        // Seven things go on the buttons: the Bridge Builder stands
        // beside the bridge, and the other six are a row.
        let (items, upgrades) = (shelves(Shop::Items), shelves(Shop::Upgrades));
        assert_eq!((items.len(), upgrades.len()), (3, 1));
        assert_eq!(items.iter().map(Vec::len).sum::<usize>(), 15);
        assert_eq!(upgrades[0].len(), 6);
        for utility in Utility::ALL {
            assert!(items.iter().flatten().any(|ware| *ware == Ware::Utility(utility)), "{utility:?} is not sold");
        }
        assert_eq!(items[0][..2], [Ware::Bridge, Ware::Utility(Utility::Builder)]);
        for (shop, rows) in [(Shop::Items, &items), (Shop::Upgrades, &upgrades)] {
            for ware in rows.iter().flatten() {
                assert_eq!(rules::sold_at(*ware), shop, "{ware:?}");
                assert!(rows.iter().flatten().filter(|other| *other == ware).count() == 1);
            }
            assert!(rows.iter().all(|row| row.len() <= 6));
            let widest = rows.iter().map(|row| row_width(row.len())).fold(0.0, f32::max);
            assert!(widest < row_width(5) + 0.6 * CARD_WIDTH, "{widest}");
            // (The second button's left edge there, from the middle of the
            // screen, where the window is: half the screen's width, less
            // the jump button's margin, 6% of it, and the second button's
            // own place and width in the box round the jump button, 18.5
            // and 14, `hud::ARENA_SLOT`.)
            let second_button = 1600.0 / 9.0 * (0.5 - 0.06) - 32.5;
            assert!(widest * 0.5 - PAD < second_button, "{widest} against {second_button}");
        }
        // A stick of dynamite, the last of the things for the buttons: 1
        // amethyst, three to a button, and not to be had with five other
        // kinds on the five buttons.
        assert!(items[2].contains(&Ware::Utility(Utility::Dynamite)));
        let stick = offer(Ware::Utility(Utility::Dynamite), &bare, &[0, 0, 0, 1]);
        assert_eq!((stick.name.as_str(), stick.picture), ("Dynamite", Picture::Dynamite));
        assert_eq!(stick.state, State::Buy { item: Item::Amethyst, cost: 1, able: true });
        let three = Outfit {
            kit: Kit::default().holding(Utility::Dynamite, 3),
            ..bare
        };
        let full = offer(Ware::Utility(Utility::Dynamite), &three, &[0, 0, 0, 9]);
        assert_eq!((full.state, full.held), (State::Max, 3));
        let five = Outfit {
            kit: Kit::default()
                .holding(Utility::Fireball, 1)
                .holding(Utility::Builder, 1)
                .holding(Utility::Speed, 1)
                .holding(Utility::Invisibility, 1)
                .holding(Utility::Healing, 1),
            ..bare
        };
        assert_eq!(offer(Ware::Utility(Utility::Dynamite), &five, &[0, 0, 0, 9]).state, State::Taken);
        // With nothing, everything has a price, which you cannot pay; a
        // name and a line that are short and plain; and nothing filled in
        // is left with its braces.
        for ware in items.iter().chain(&upgrades).flatten() {
            let card = offer(*ware, &bare, &[0; 4]);
            assert!(matches!(card.state, State::Buy { able: false, .. }), "{ware:?}: {:?}", card.state);
            let drawn = |words: &str| words.chars().all(roundtown_net::text::allowed);
            assert!(!card.name.is_empty() && drawn(&card.name) && drawn(&card.stat));
            assert!(!card.name.contains('{') && !card.stat.contains('{'), "{ware:?}: {} / {}", card.name, card.stat);
            assert!(card.stat.len() <= 16, "{ware:?}: {:?} is a long line for a card", card.stat);
            assert!(card.name.split(' ').all(|word| word.len() <= 10), "{ware:?}: {:?} will not fit a card", card.name);
        }
        // The numbers are the rules': 25 m of bridge for 4 silver, a sword
        // of +5 for 10.
        let bridge = offer(Ware::Bridge, &bare, &[4, 0, 0, 0]);
        assert_eq!((bridge.stat.as_str(), bridge.state), ("+25m", State::Buy { item: Item::Silver, cost: 4, able: true }));
        assert_eq!(offer(Ware::Sword(1), &bare, &[9, 0, 0, 0]).stat, "+5 dmg");
        assert_eq!(offer(Ware::Shield(3), &bare, &[0; 4]).stat, "-22% dmg");
        assert_eq!(offer(Ware::Utility(Utility::Speed), &bare, &[0; 4]).stat, "+40% 45s");
        // The compass, beside the shields: 2 amethyst, and yours until you
        // die, with nothing more of it to buy.
        assert!(items[1].contains(&Ware::Compass));
        let compass = offer(Ware::Compass, &bare, &[0, 0, 0, 2]);
        assert_eq!(compass.state, State::Buy { item: Item::Amethyst, cost: 2, able: true });
        let tracking = Outfit {
            kit: Kit { compass: true, ..Kit::default() },
            ..bare
        };
        assert_eq!(offer(Ware::Compass, &tracking, &[0, 0, 0, 9]).state, State::Owned);
        // With the silver sword, the basic and the silver are yours and
        // the aura is still to buy; with fireballs on a button, a potion
        // is to be had beside them, on the next, and the fireball's card
        // counts them, up to the three a button holds.
        let armed = Outfit {
            kit: Kit {
                sword: 2,
                ..Kit::default()
            }
            .holding(Utility::Fireball, 2),
            team: Upgrades {
                forge: 4,
                shields: 2,
                traps: 3,
                swords: true,
                heal_pool: false,
                build_speed: 1,
            },
        };
        let rich: Purse = [99; 4];
        assert_eq!(offer(Ware::Sword(1), &armed, &rich).state, State::Owned);
        assert_eq!(offer(Ware::Sword(2), &armed, &rich).state, State::Owned);
        assert!(matches!(offer(Ware::Sword(3), &armed, &rich).state, State::Buy { able: true, .. }));
        let speed = offer(Ware::Utility(Utility::Speed), &armed, &rich);
        assert!(matches!(speed.state, State::Buy { able: true, .. }) && speed.held == 0);
        let fireball = offer(Ware::Utility(Utility::Fireball), &armed, &rich);
        assert!(matches!(fireball.state, State::Buy { able: true, .. }) && fireball.held == 2);
        let full = Outfit {
            kit: armed.kit.holding(Utility::Fireball, 3),
            ..armed
        };
        let fireball = offer(Ware::Utility(Utility::Fireball), &full, &rich);
        assert_eq!((fireball.state, fireball.held), (State::Max, 3));
        // With every button taken by another kind, there is nowhere for
        // one more kind to go.
        let crowded = Outfit {
            kit: Kit {
                slots: [Some(roundtown_net::Stack { utility: Utility::Fireball, count: 1 }); roundtown_net::UTILITY_SLOTS],
                ..Kit::default()
            },
            ..armed
        };
        assert_eq!(offer(Ware::Utility(Utility::Speed), &crowded, &rich).state, State::Taken);
        // An upgrade taken to its last step has no more to sell, and
        // shows every pip lit; one part way shows the next step's price.
        let forge = offer(Ware::Forge, &armed, &rich);
        assert_eq!((forge.state, forge.pips), (State::Max, Some((4, 4))));
        assert_eq!(forge.name, ui().words.market.forge[3]);
        let shields = offer(Ware::BetterShields, &armed, &rich);
        assert_eq!((shields.state, shields.pips), (State::Buy { item: Item::Sapphire, cost: 4, able: true }, Some((2, 4))));
        assert_eq!(offer(Ware::Trap, &armed, &rich).state, State::Max);
        assert_eq!(offer(Ware::StrongerSwords, &armed, &rich).state, State::Owned);
        assert!(matches!(offer(Ware::HealPool, &armed, &rich).state, State::Buy { cost: 1, .. }));
        assert_eq!(offer(Ware::Forge, &bare, &rich).name, ui().words.market.forge[0]);
        // The leap potion, 1 amethyst, five times as high for 45 s; and
        // the team's building, three steps for 4, 8 and 16 sapphire, each
        // card saying how fast the team builds once it has it.
        let leap = offer(Ware::Utility(Utility::Leap), &bare, &[0, 0, 0, 1]);
        assert_eq!((leap.name.as_str(), leap.stat.as_str(), leap.picture), ("Leap", "x5 jump 45s", Picture::Flask));
        assert_eq!(leap.state, State::Buy { item: Item::Amethyst, cost: 1, able: true });
        let first = offer(Ware::BuildSpeed, &bare, &rich);
        assert_eq!((first.name.as_str(), first.stat.as_str()), ("Advanced Builder", "75% speed"));
        assert_eq!((first.state, first.pips), (State::Buy { item: Item::Sapphire, cost: 4, able: true }, Some((0, 3))));
        let second = offer(Ware::BuildSpeed, &armed, &rich);
        assert_eq!((second.name.as_str(), second.stat.as_str()), ("Master Builder", "100% speed"));
        assert_eq!(second.state, State::Buy { item: Item::Sapphire, cost: 8, able: true });
        let done = Outfit {
            team: Upgrades { build_speed: 3, ..armed.team },
            ..armed
        };
        let last = offer(Ware::BuildSpeed, &done, &rich);
        assert_eq!((last.name.as_str(), last.stat.as_str(), last.state), ("Grand Builder", "125% speed", State::Max));
    }
}
