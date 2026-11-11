//! Bridge Wars' shop, at the red-roofed market stand on each island (since
//! 2026-11-11; Hajun: "make it so that the red shop if the user goes inside
//! the activation, it opens a closable popup shop menu, and right now just
//! have 'bridge' as the option, which should cost 4 silver for 50m of
//! bridge. make it so u just tap on the bridge and it buys 1 time right
//! away no confirmation needed. and u can close the shop panel tapping on
//! close button in top right corner or just tapping anywhere else on
//! screen").
//!
//! **Where the shops are is the map's**: Hajun drew a white outline on the
//! ground in front of every stand, in a material called `activation`
//! ([`ACTIVATION_MATERIAL`]), and a shop is each of those that lies at a
//! stand whose roof is red ([`RED_ROOF_MATERIAL`]; [`shop_zones`], read
//! once the map has spawned). Move a stand and its outline in Blender and
//! the shop follows; the blue stands' outlines are nothing yet. The server
//! has no map, and sells only to someone near where the shops stood when
//! it was built (`roundtown_net::ARENA_SHOPS`): a test keeps the two
//! together.
//!
//! **Standing in the outline opens the shop** ([`watch`]): a small window
//! over the middle of the screen with what it sells on a card, what that
//! costs, and how much bridge you have, which is written nowhere else but
//! in front of you as you build. It is not a dialog: the fight goes on
//! round it and you can walk, and only a press on it is its own. A tap on
//! the card buys one, at once, if you hold its price: offline out of what
//! you hold, here; online asked of the server, which takes the price and
//! says so (`Tell::Bought`, `arena::hear`). The cross on its corner shuts
//! it, and so does a press anywhere else on the screen, which goes on to
//! do whatever else it does, the stick included ([`shut_by_hand`]), and
//! Escape; shut that way it stays shut until you have stepped out of the
//! outline and in again. Stepping out of it shuts it too, as does dying,
//! and the game's end.

use bevy::gltf::GltfMaterialName;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use roundtown_net::bridge_wars::{BRIDGE_A_BUY, price};
use roundtown_net::{Ask as NetAsk, Ware};

use crate::arena::{Fighting, InArena, Resources};
use crate::builder::{self, Fill};
use crate::hud::{self, DialogUp, TakesPress};
use crate::island::{self, Land, Venue, is_material};
use crate::ui::{fill, ui};
use crate::{Dead, Player};

/// What the outlines on the ground in front of the stands are made of in
/// the map, and what a stand that is a shop is roofed with.
pub(crate) const ACTIVATION_MATERIAL: &str = "activation";
pub(crate) const RED_ROOF_MATERIAL: &str = "RoofTiles_Red";
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
/// Sizes, as shares of the short side of the screen: the window, and how
/// far down the screen its top is, as a share of the screen's height; its
/// title; the card's picture, the name of what it sells and its price,
/// with the picture of what it is paid in; the line under the card; and
/// the cross on its corner.
const WIDTH: f32 = 54.0;
const TOP: f32 = 24.0;
const TITLE: f32 = 4.6;
const PICTURE: f32 = 8.5;
const NAME: f32 = 4.0;
const PRICE: f32 = 3.8;
const PRICE_ICON: f32 = 4.0;
const HAVE: f32 = 3.0;
const CLOSE: f32 = 7.5;

/// Where the shops are, as the map has them: each the box round an
/// outline, across the ground, as its lowest and highest corners.
#[derive(Resource, Default, Debug)]
pub(crate) struct Shops {
    pub(crate) zones: Vec<(Vec2, Vec2)>,
}

/// Whether you are standing in a shop's outline, and whether you have shut
/// the shop by hand since you stepped into it.
#[derive(Resource, Default)]
struct Counter {
    inside: bool,
    shut: bool,
}

/// The shop's window.
#[derive(Component)]
struct ShopWindow;
/// The card that sells the bridge, its name and its price on it.
#[derive(Component)]
struct BridgeCard;
#[derive(Component)]
struct CardName;
#[derive(Component)]
struct CardPrice;
/// The line under the card: how much bridge you have.
#[derive(Component)]
struct HaveLine;
/// The cross on the window's corner.
#[derive(Component)]
struct CloseShop;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Counter>()
        .add_observer(read_map)
        .add_observer(press)
        .add_systems(
            Update,
            (shut_by_hand, watch, refresh).chain().after(crate::move_bodies),
        );
}

/// Reads the shops off the map once it has spawned: every corner of every
/// outline and of every red roof, where the map puts them.
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
    let (mut outlines, mut roofs) = (Vec::new(), Vec::new());
    for part in children.iter_descendants(ready.entity) {
        let Ok((mesh, material)) = parts.get(part) else {
            continue;
        };
        let corners = if is_material(&material.0, ACTIVATION_MATERIAL) {
            &mut outlines
        } else if is_material(&material.0, RED_ROOF_MATERIAL) {
            &mut roofs
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
    let zones = shop_zones(&outlines, &roofs);
    if zones.is_empty() {
        warn!("Bridge Wars' map has no outline in {ACTIVATION_MATERIAL:?} at a stand roofed in {RED_ROOF_MATERIAL:?}: no shop");
    } else {
        info!("Bridge Wars' shops: {} of them, the first from {} to {}", zones.len(), zones[0].0, zones[0].1);
    }
    commands.insert_resource(Shops { zones });
}

/// The shops, given every corner of every outline on the ground and of
/// every red roof, across the ground: the box round each outline whose
/// middle is at the middle of a red roof.
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

/// Opens the shop as you step into an outline in a game that is on, and
/// shuts it as you step out of one, die, or the game ends; shut by hand
/// ([`shut_by_hand`], [`press`]), it stays shut until you have stepped
/// out and in again.
#[allow(clippy::too_many_arguments)]
fn watch(
    fighting: Res<Fighting>,
    shops: Option<Res<Shops>>,
    dialog: Res<DialogUp>,
    stock: Res<Resources>,
    me: Query<(&Transform, &Venue, Has<Dead>), With<Player>>,
    windows: Query<Entity, With<ShopWindow>>,
    mut counter: ResMut<Counter>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let inside = fighting.playing
        && !dialog.0
        && shops.is_some_and(|shops| {
            me.single().is_ok_and(|(at, &venue, dead)| {
                venue == Venue::Arena && !dead && in_a_shop(&shops.zones, at.translation)
            })
        });
    let up = windows.iter().next();
    if !inside {
        if let Some(window) = up {
            commands.entity(window).despawn();
        }
        if counter.inside || counter.shut {
            *counter = Counter::default();
        }
        return;
    }
    if !counter.inside {
        counter.inside = true;
    }
    match (counter.shut, up) {
        (false, None) => open(&mut commands, &mut images, &stock),
        (true, Some(window)) => commands.entity(window).despawn(),
        _ => {}
    }
}

/// The window: its title, the bridge on a card with what it costs, how
/// much bridge you have, and the cross on its corner.
fn open(commands: &mut Commands, images: &mut Assets<Image>, stock: &Resources) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.arena);
    let (item, cost) = price(Ware::Bridge);
    let able = stock.held[item.index()] >= cost;
    let (card, ink, pay) = card_look(able);
    commands
        .spawn((
            InArena,
            ShopWindow,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Vh(TOP),
                width: Val::VMin(WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(1.8),
                padding: UiRect::all(Val::VMin(3.0)),
                border_radius: BorderRadius::all(Val::VMin(3.5)),
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
            window.spawn(builder::label(words.shop.clone(), TITLE, colours.gold));
            window
                .spawn((
                    BridgeCard,
                    TakesPress,
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        column_gap: Val::VMin(3.0),
                        padding: UiRect::axes(Val::VMin(3.0), Val::VMin(2.2)),
                        border_radius: BorderRadius::all(Val::VMin(2.6)),
                        ..default()
                    },
                    BackgroundColor(card.idle),
                    card,
                ))
                .with_children(|card| {
                    card.spawn((
                        ImageNode::new(images.add(hud::plank())).with_color(colours.hud.icon),
                        Node {
                            width: Val::VMin(PICTURE),
                            height: Val::VMin(PICTURE),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    card.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            row_gap: Val::VMin(1.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|lines| {
                        let name = fill(&words.shop_bridge, &[("metres", &format!("{BRIDGE_A_BUY:.0}"))]);
                        lines.spawn((CardName, builder::label(name, NAME, ink)));
                        lines
                            .spawn((
                                Node {
                                    align_items: AlignItems::Center,
                                    column_gap: Val::VMin(1.2),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|row| {
                                row.spawn((
                                    ImageNode::new(images.add(hud::item_picture(item))).with_color(colours.arena.item(item)),
                                    Node {
                                        width: Val::VMin(PRICE_ICON),
                                        height: Val::VMin(PRICE_ICON),
                                        flex_shrink: 0.0,
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                                row.spawn((CardPrice, builder::label(cost.to_string(), PRICE, pay)));
                            });
                    });
                });
            window.spawn((HaveLine, builder::label(have(stock), HAVE, colours.text_soft)));
            window.spawn(hud::close_cross(CLOSE, CloseShop));
        });
}

/// How the card looks when you can pay for it and when you cannot: its
/// fill, lit under the pointer only if it can be bought; the name of what
/// it sells; and its price, a warning with too little.
fn card_look(able: bool) -> (Fill, Color, Color) {
    let colours = ui().colours;
    if able {
        (
            Fill {
                idle: colours.shop.card,
                lit: colours.shop.card_lit,
            },
            colours.text,
            colours.text,
        )
    } else {
        (
            Fill {
                idle: colours.shop.card_off,
                lit: colours.shop.card_off,
            },
            colours.shop.name_off,
            colours.warn_lit,
        )
    }
}

/// "You have 50m of bridge".
fn have(stock: &Resources) -> String {
    fill(&ui().words.arena.shop_have, &[("metres", &format!("{:.0}", stock.bridge_metres.max(0.0).round()))])
}

/// Keeps the window as what you hold changes: the card lit or blocked out
/// by whether you can pay, and the line under it.
#[allow(clippy::type_complexity)]
fn refresh(
    stock: Res<Resources>,
    mut cards: Query<(&mut Fill, &mut BackgroundColor), With<BridgeCard>>,
    mut names: Query<&mut TextColor, (With<CardName>, Without<CardPrice>)>,
    mut prices: Query<&mut TextColor, (With<CardPrice>, Without<CardName>)>,
    mut lines: Query<&mut Text, With<HaveLine>>,
) {
    if !stock.is_changed() || cards.is_empty() {
        return;
    }
    let (item, cost) = price(Ware::Bridge);
    let (look, ink, pay) = card_look(stock.held[item.index()] >= cost);
    for (mut fill, mut colour) in &mut cards {
        if fill.idle != look.idle {
            *fill = look;
            colour.set_if_neq(BackgroundColor(look.idle));
        }
    }
    for mut colour in &mut names {
        colour.set_if_neq(TextColor(ink));
    }
    for mut colour in &mut prices {
        colour.set_if_neq(TextColor(pay));
    }
    let line = have(&stock);
    for mut text in &mut lines {
        if text.0 != line {
            text.0.clone_from(&line);
        }
    }
}

/// A tap on the card buys a bridge, at once, if you hold its price; a tap
/// on the cross shuts the shop.
fn press(
    click: On<Pointer<Click>>,
    cards: Query<(), With<BridgeCard>>,
    closes: Query<(), With<CloseShop>>,
    fighting: Res<Fighting>,
    mut stock: ResMut<Resources>,
    mut counter: ResMut<Counter>,
    mut requests: MessageWriter<NetAsk>,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if closes.contains(click.entity) {
        counter.shut = true;
        return;
    }
    if !cards.contains(click.entity) || !fighting.playing {
        return;
    }
    let (item, cost) = price(Ware::Bridge);
    if stock.held[item.index()] < cost {
        return;
    }
    if fighting.online {
        // The server takes the price and says it is yours (`arena::hear`).
        requests.write(NetAsk::Buy(Ware::Bridge));
    } else {
        stock.held[item.index()] -= cost;
        stock.bridge_metres += BRIDGE_A_BUY;
        info!("bought {BRIDGE_A_BUY} m of bridge");
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
    use roundtown_net::{ARENA_ISLAND_RADIUS, ARENA_ISLANDS, ARENA_SHOPS, ARENA_TEAMS};

    /// Every corner of the map's parts in `material`, across the ground.
    fn corners_of(parts: &[island::glb::Part], material: &str) -> Vec<Vec2> {
        parts
            .iter()
            .filter(|part| part.material.as_deref().is_some_and(|name| is_material(name, material)))
            .flat_map(|part| part.triangles.iter().flatten().map(|corner| corner.xz()))
            .collect()
    }

    #[test]
    fn the_shops_are_the_outlines_at_the_red_stands_where_the_server_has_them() {
        let parts = island::glb::read("pvptownmap.glb");
        let outlines = corners_of(&parts, ACTIVATION_MATERIAL);
        let roofs = corners_of(&parts, RED_ROOF_MATERIAL);
        // Two outlines to an island, one of them at a red stand.
        assert_eq!(boxes(&outlines).len(), 2 * ARENA_TEAMS);
        assert_eq!(boxes(&roofs).len(), ARENA_TEAMS);
        let zones = shop_zones(&outlines, &roofs);
        assert_eq!(zones.len(), ARENA_TEAMS, "{zones:?}");
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
}
