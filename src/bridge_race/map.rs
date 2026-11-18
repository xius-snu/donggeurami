//! The minimap top right in a game of Bridge Race (`super`; since
//! 2026-11-18, mine, from a play-test that day: the camera 15.5 m over you
//! showed your own bridge and the void, the other team's bridge came into
//! sight about 140 s into a race of two and a half minutes, and until then
//! the race was a bar on the plate): the whole map from above, as Bridge
//! Wars' minimap draws its own (`minimap`, the same look: the plate's grey
//! round it, the land in `colours.arena.map_land`, the bridges in the
//! planks' colour, a thin line round each team's island in its colour), its
//! top north as the top of the screen is under the camera fixed over you.
//!
//! * **The land** is the three islands, each a disc 20 m across where the
//!   rules have it (`roundtown_net::bridge_race::ISLANDS`, `GOLDEN`): red's
//!   and blue's each with a line round it in its team's colour, the golden
//!   island's in its gold (`colours.bridge_race.accent`), drawn once.
//! * **The bridges**, both teams', drawn over it again whenever they change
//!   (`Bridges::version`), the holes blown in them with them: the race
//!   itself, both ways across, at a glance.
//! * **A dot for everyone in the race** where they stand, in their team's
//!   colour, from "Go!" on (in the waiting room nobody's team is out yet),
//!   yours bigger, with a white rim (`colours.bridge_race.map_you`); the
//!   dead not shown. Mine: the other team's too, where Bridge Wars' shows
//!   only you and your teammate (Hajun, for Bridge Wars: "not show the other
//!   players"): a race is run in the open, and where they stand is how far
//!   their bridge has got.
//!
//! The dots are placed by their `UiTransform`, which lays nothing out
//! again, and written only as they move.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use roundtown_net::bridge_race::{GOLDEN, ISLAND_RADIUS, ISLANDS};

use super::Racing;
use crate::bridge::Bridges;
use crate::island::Venue;
use crate::ui::{BridgeRaceColours, ui};
use crate::{Dead, Player, Team, WalkCycle};

/// What of the map the picture shows, in metres across the ground: from
/// the west to the east and from the north to the south (the game's -Z),
/// the three islands and room round them for a bridge laid wide, the same
/// way up and as wide as the map's own ground (`race.nav` reaches 20 m past
/// every island).
const WEST: f32 = -98.0;
const EAST: f32 = 18.0;
const NORTH: f32 = -44.0;
const SOUTH: f32 = 44.0;
/// How many pixels of the picture to a metre.
const PER_METRE: f32 = 2.0;
/// How wide the minimap is on the screen, in shares of its short side (as
/// tall as the picture is for it): a little wider than Bridge Wars' 26 is
/// tall, the race's map being wider than it is tall.
const WIDE: f32 = 30.0;
/// Its rim and its rounded corners, as Bridge Wars' are (`hud`'s).
const RIM: f32 = 0.35;
const ROUND: f32 = 1.6;
/// How thick the line round each island is, outward from its edge, in
/// pixels of the picture.
const OUTLINE: f32 = 2.5;
/// How big a dot is, in shares of the short side of the screen: everyone
/// else's, and yours, with its rim.
const DOT: f32 = 1.6;
const YOU: f32 = 2.1;
const YOU_RIM: f32 = 0.35;

/// The minimap on the screen: its picture, the land alone to draw the
/// bridges over again, and which bridges were last drawn.
#[derive(Component)]
pub(super) struct Minimap {
    image: Handle<Image>,
    land: Vec<u8>,
    drawn: Option<u32>,
}

/// One dot over the minimap: whose body it follows.
#[derive(Component)]
pub(super) struct MapDot {
    of: Entity,
}

/// How many pixels across and down the picture is.
fn picture_size() -> UVec2 {
    UVec2::new(((EAST - WEST) * PER_METRE).round() as u32, ((SOUTH - NORTH) * PER_METRE).round() as u32)
}

/// Where a point `at` on the ground is on the picture, as a share of it
/// across and down from its top left, which is the map's north-west.
fn share(at: Vec2) -> Vec2 {
    (at - Vec2::new(WEST, NORTH)) / Vec2::new(EAST - WEST, SOUTH - NORTH)
}

/// A colour as the picture's bytes.
fn bytes(colour: Color) -> [u8; 4] {
    colour.to_srgba().to_u8_array()
}

/// The picture of the land: the three islands, each a disc of `land`, with
/// a line round it `OUTLINE` pixels thick in `lines` (by island: red's,
/// blue's, the golden island's); clear round them, for the minimap's own
/// background to show.
fn land_picture(land: [u8; 4], lines: [[u8; 4]; 3]) -> Vec<u8> {
    let size = picture_size();
    let mut data = [land[0], land[1], land[2], 0].repeat((size.x * size.y) as usize);
    let islands = [ISLANDS[0], ISLANDS[1], GOLDEN];
    for y in 0..size.y {
        for x in 0..size.x {
            let at = Vec2::new(WEST + (x as f32 + 0.5) / PER_METRE, NORTH + (y as f32 + 0.5) / PER_METRE);
            for (island, line) in islands.iter().zip(lines) {
                let out = at.distance(*island) * PER_METRE;
                let edge = ISLAND_RADIUS * PER_METRE;
                let colour = if out <= edge {
                    land
                } else if out <= edge + OUTLINE {
                    line
                } else {
                    continue;
                };
                let k = ((y * size.x + x) * 4) as usize;
                data[k..k + 4].copy_from_slice(&colour);
            }
        }
    }
    data
}

/// Fills the convex polygon `corners`, across the ground, in `colour`,
/// over a picture `size` across (Bridge Wars' minimap's own way, copied).
fn fill(data: &mut [u8], size: UVec2, corners: &[Vec2], colour: [u8; 4]) {
    let corners: Vec<Vec2> = corners.iter().map(|at| share(*at) * size.as_vec2()).collect();
    let (low, high) = corners.iter().fold((Vec2::INFINITY, Vec2::NEG_INFINITY), |(low, high), c| (low.min(*c), high.max(*c)));
    let (x0, y0) = (low.x.floor().max(0.0) as u32, low.y.floor().max(0.0) as u32);
    let (x1, y1) = (high.x.ceil().min(size.x as f32 - 1.0), high.y.ceil().min(size.y as f32 - 1.0));
    if x1 < 0.0 || y1 < 0.0 || x0 as f32 > x1 || y0 as f32 > y1 {
        return;
    }
    let (x1, y1) = (x1 as u32, y1 as u32);
    let turning: f32 = (0..corners.len()).map(|i| corners[i].perp_dot(corners[(i + 1) % corners.len()])).sum();
    let hand = if turning >= 0.0 { 1.0 } else { -1.0 };
    for y in y0..=y1 {
        for x in x0..=x1 {
            let at = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let inside = (0..corners.len()).all(|i| {
                let (a, b) = (corners[i], corners[(i + 1) % corners.len()]);
                (b - a).perp_dot(at - a) * hand >= -0.5
            });
            if inside {
                let k = ((y * size.x + x) * 4) as usize;
                data[k..k + 4].copy_from_slice(&colour);
            }
        }
    }
}

/// Puts the minimap up under `parent` (the right of the bar across the top,
/// where Bridge Wars has its own), its land drawn, its bridges to come.
pub(super) fn spawn(commands: &mut Commands, parent: Entity, images: &mut Assets<Image>) {
    let colours = ui().colours;
    let line = |colour: Color| bytes(BridgeRaceColours::solid(colour, 1.0));
    let land = land_picture(bytes(colours.arena.map_land), [line(colours.arena.team(0)), line(colours.arena.team(1)), line(colours.bridge_race.accent)]);
    let size = picture_size();
    let mut image = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.data = Some(land.clone());
    let image = images.add(image);
    commands.spawn((
        ChildOf(parent),
        Minimap {
            image: image.clone(),
            land,
            drawn: None,
        },
        Name::new("Bridge Race's minimap"),
        ImageNode::new(image),
        Node {
            width: Val::VMin(WIDE),
            height: Val::VMin(WIDE * (SOUTH - NORTH) / (EAST - WEST)),
            border: UiRect::all(Val::VMin(RIM)),
            border_radius: BorderRadius::all(Val::VMin(ROUND)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(colours.arena.map_back),
        BorderColor::all(colours.arena.hp_rim),
        Pickable::IGNORE,
    ));
}

/// Draws the bridges over the land again whenever they change: every
/// plank's top that stands, the tips being laid with them.
pub(super) fn draw(bridges: Res<Bridges>, mut maps: Query<&mut Minimap>, mut images: ResMut<Assets<Image>>) {
    for mut map in &mut maps {
        if map.drawn == Some(bridges.version) {
            continue;
        }
        map.drawn = Some(bridges.version);
        let size = picture_size();
        let mut data = map.land.clone();
        let plank = bytes(ui().colours.arena.plank);
        for quad in bridges.quads() {
            fill(&mut data, size, &quad, plank);
        }
        if let Some(mut image) = images.get_mut(&map.image) {
            image.data = Some(data);
        }
    }
}

/// Where a dot `dot` across (logical pixels) stands for `at` on a minimap
/// `map` across: its top left, over the picture inside the minimap's rim
/// (`rim`, logical pixels).
fn dot_at(at: Vec2, map: Vec2, rim: f32, dot: f32) -> Vec2 {
    let inside = map - Vec2::splat(2.0 * rim);
    Vec2::splat(rim) + share(at) * inside - Vec2::splat(dot * 0.5)
}

/// A dot over the minimap for everyone in the race, from "Go!" on, where
/// each stands, in their team's colour, yours bigger and rimmed, put away
/// while they are dead or off the race's map; each put up as its body
/// comes and taken away with it.
#[allow(clippy::type_complexity)]
pub(super) fn dots(
    racing: Res<Racing>,
    windows: Query<&Window>,
    me: Query<(Entity, &Transform, &Venue, Option<&Team>, Has<Dead>), With<Player>>,
    others: Query<(Entity, &Transform, &Venue, &Team, Has<Dead>), (With<WalkCycle>, Without<Player>)>,
    maps: Query<(Entity, &ComputedNode), With<Minimap>>,
    mut placed: Query<(Entity, &MapDot, &mut UiTransform, &mut Visibility)>,
    mut commands: Commands,
) {
    let Ok((map, node)) = maps.single() else {
        return;
    };
    let begun = racing.0.as_ref().is_some_and(|race| race.begun);
    let Ok(window) = windows.single() else {
        return;
    };
    let vmin = window.width().min(window.height()) / 100.0;
    let size = node.size() * node.inverse_scale_factor();
    let rim = RIM * vmin;
    // Everyone a dot is for, where they stand, their team, whether it is
    // you, and whether it is shown.
    let mut bodies: Vec<(Entity, Vec2, u8, bool, bool)> = Vec::new();
    if let Ok((you, at, &venue, team, dead)) = me.single() {
        let shown = begun && venue == Venue::BridgeRace && !dead;
        bodies.push((you, at.translation.xz(), team.map_or(0, |team| team.0), true, shown));
    }
    if begun {
        bodies.extend(
            others
                .iter()
                .filter(|(_, _, venue, ..)| **venue == Venue::BridgeRace)
                .map(|(body, at, _, team, dead)| (body, at.translation.xz(), team.0, false, !dead)),
        );
    }
    let colours = ui().colours;
    for &(body, at, team, yours, shown) in &bodies {
        let across = (if yours { YOU + 2.0 * YOU_RIM } else { DOT }) * vmin;
        let corner = dot_at(at, size, rim, across);
        let wanted = UiTransform::from_translation(Val2::px(corner.x.round(), corner.y.round()));
        let visible = if shown { Visibility::Inherited } else { Visibility::Hidden };
        match placed.iter_mut().find(|(_, dot, ..)| dot.of == body) {
            Some((_, _, mut place, mut visibility)) => {
                place.set_if_neq(wanted);
                visibility.set_if_neq(visible);
            }
            None => {
                let ink = BridgeRaceColours::solid(colours.arena.team(team), 1.0);
                let (wide, rim) = if yours { (YOU + 2.0 * YOU_RIM, YOU_RIM) } else { (DOT, 0.0) };
                commands.spawn((
                    ChildOf(map),
                    MapDot { of: body },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::ZERO,
                        top: Val::ZERO,
                        width: Val::VMin(wide),
                        height: Val::VMin(wide),
                        border: UiRect::all(Val::VMin(rim)),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    wanted,
                    BackgroundColor(ink),
                    BorderColor::all(colours.bridge_race.map_you),
                    // Yours over everyone else's.
                    ZIndex(if yours { 2 } else { 1 }),
                    visible,
                    Pickable::IGNORE,
                ));
            }
        }
    }
    for (dot, of, ..) in &placed {
        if !bodies.iter().any(|(body, ..)| *body == of.of) {
            commands.entity(dot).despawn();
        }
    }
}

/// Your dot takes your team's colour once you are on it (in the waiting
/// room you are on none, and it was put up then in red's).
#[allow(clippy::type_complexity)]
pub(super) fn recolour(me: Query<(Entity, &Team), (With<Player>, Changed<Team>)>, mut dots: Query<(&MapDot, &mut BackgroundColor)>) {
    let Ok((you, team)) = me.single() else {
        return;
    };
    let ink = BridgeRaceColours::solid(ui().colours.arena.team(team.0), 1.0);
    for (dot, mut colour) in &mut dots {
        if dot.of == you {
            colour.set_if_neq(BackgroundColor(ink));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_minimap_has_north_at_the_top_and_the_three_islands_on_it_lined_in_their_colours() {
        let size = picture_size();
        // As wide for its height as the minimap is on the screen, and
        // every island inside it with room round it.
        assert_eq!(size, UVec2::new(232, 176));
        for island in [ISLANDS[0], ISLANDS[1], GOLDEN] {
            let at = share(island);
            assert!(at.min_element() > 0.05 && at.max_element() < 0.95, "{island} at {at}");
        }
        // Red's island (the map's -Z, north) is over blue's on the
        // picture, and the golden island to their right.
        assert!(share(ISLANDS[0]).y < share(ISLANDS[1]).y);
        assert!(share(GOLDEN).x > share(ISLANDS[0]).x + 0.5);
        let (land, lines) = ([200u8, 190, 160, 255], [[255u8, 0, 0, 255], [0, 0, 255, 255], [255, 210, 0, 255]]);
        let data = land_picture(land, lines);
        let pixel = |at: Vec2| {
            let p = (share(at) * size.as_vec2()).floor();
            let k = ((p.y as u32 * size.x + p.x as u32) * 4) as usize;
            [data[k], data[k + 1], data[k + 2], data[k + 3]]
        };
        for (island, line) in [ISLANDS[0], ISLANDS[1], GOLDEN].into_iter().zip(lines) {
            assert_eq!(pixel(island), land, "the middle of {island}");
            assert_eq!(pixel(island + Vec2::new(ISLAND_RADIUS + 0.6, 0.0)), line, "the line round {island}");
            assert_eq!(pixel(island + Vec2::new(ISLAND_RADIUS + 4.0, 0.0))[3], 0, "clear past {island}");
        }
        // The void between them clear.
        assert_eq!(pixel(Vec2::new(-40.0, 0.0))[3], 0);
    }

    #[test]
    fn a_plank_is_drawn_where_it_lies_and_a_dot_stands_where_its_body_does() {
        let size = picture_size();
        let mut data = vec![0u8; (size.x * size.y * 4) as usize];
        // A plank 2 m wide along the way from red's island.
        let quad = [Vec2::new(-60.0, -16.0), Vec2::new(-60.0, -14.0), Vec2::new(-56.0, -14.0), Vec2::new(-56.0, -16.0)];
        fill(&mut data, size, &quad, [1, 2, 3, 255]);
        let at = (share(Vec2::new(-58.0, -15.0)) * size.as_vec2()).floor();
        let k = ((at.y as u32 * size.x + at.x as u32) * 4) as usize;
        assert_eq!(&data[k..k + 4], &[1, 2, 3, 255]);
        let away = (share(Vec2::new(-58.0, -10.0)) * size.as_vec2()).floor();
        assert_eq!(data[((away.y as u32 * size.x + away.x as u32) * 4 + 3) as usize], 0);
        // A dot for the golden island's middle on a minimap 300 by 228,
        // its rim 4: its middle on the island's place in the picture.
        let (map, rim, dot) = (Vec2::new(300.0, 228.0), 4.0, 16.0);
        let middle = dot_at(GOLDEN, map, rim, dot) + Vec2::splat(dot * 0.5);
        let expected = Vec2::splat(rim) + share(GOLDEN) * (map - Vec2::splat(2.0 * rim));
        assert!((middle - expected).length() < 1e-3);
        assert!(middle.x > map.x * 0.8 && (middle.y - map.y * 0.5).abs() < 1.0, "{middle}");
    }
}
