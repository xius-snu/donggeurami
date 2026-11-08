//! The minimap top right in a game of Bridge Wars (since 2026-11-09,
//! Hajun: "put a minimap that only shows the current player's location as
//! a red dot and your teammate as a green dot, and the full map in top
//! down view including the live bridges but not show the other players"):
//! a picture of the map from above ([`Minimap::image`]), the land in the
//! file's `colours.arena.map_land` over `map_back`, drawn once from the
//! map's own triangles as they are read ([`draw_land`]), and the bridges
//! drawn over it again whenever they change ([`draw_bridges`], in the
//! planks' colour); and over the picture two kinds of dot, you in `map_me`
//! and your teammate in `map_mate` ([`place_dots`]), placed as you move.
//! Nobody else is shown. `hud` puts the picture on the screen
//! ([`MinimapNode`]); the top of it is north, as the top of the screen is
//! under the camera fixed over you.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::bridge::Bridges;
use crate::island::{Islands, Venue};
use crate::ui::{UiChanged, ui};
use crate::{Dead, Player, Team, WalkCycle};

/// How many pixels across the picture is.
const SIZE: u32 = 256;
/// How far from the middle of the map the picture reaches, in metres, each
/// way: the islands' far edges are 140.4 m out.
const HALF: f32 = 150.0;
/// How high over the ground a triangle may be and still be drawn as land,
/// in metres: the islands' tops are 0.25 m up, their desks 1.15; the
/// lamps' heads and the waiting room, over that, are not drawn.
const LAND_TOP: f32 = 2.0;
/// How big a dot is, as a share of the picture.
const DOT: f32 = 6.0;

/// The picture, and what it was drawn from.
#[derive(Resource)]
pub(crate) struct Minimap {
    pub(crate) image: Handle<Image>,
    /// The land alone, to draw the bridges over again.
    land: Option<Vec<u8>>,
    /// Which bridges were last drawn (`Bridges::version`).
    drawn: u32,
}

/// The node the picture is on, which the dots are put over.
#[derive(Component)]
pub(crate) struct MinimapNode;

/// One dot over the picture: whose body it follows.
#[derive(Component)]
struct Dot {
    of: Entity,
}

pub(crate) fn plugin(app: &mut App) {
    // The picture is there from the start, clear, for `hud` to put on the
    // screen as the app opens; it is drawn once the map has been read.
    let image = Image::new_fill(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);
    app.insert_resource(Minimap {
        image: handle,
        land: None,
        drawn: u32::MAX,
    })
    .add_systems(Update, (draw_land, draw_bridges, place_dots).chain().after(crate::follow_camera));
}

/// Where a point `at` on the ground is in the picture, in pixels from its
/// top left, which is the map's north-west: the picture's top is north,
/// the game's -Z.
fn pixel(at: Vec2) -> Vec2 {
    (at + Vec2::splat(HALF)) / (2.0 * HALF) * SIZE as f32
}

/// The same, as a share of the picture, for a node over it.
fn share(at: Vec2) -> Vec2 {
    (at + Vec2::splat(HALF)) / (2.0 * HALF) * 100.0
}

/// A colour as the picture's bytes.
fn bytes(colour: Color) -> [u8; 4] {
    colour.to_srgba().to_u8_array()
}

/// Fills the convex polygon `corners`, in pixels, with `colour`.
fn fill(data: &mut [u8], corners: &[Vec2], colour: [u8; 4]) {
    let (low, high) = corners.iter().fold((Vec2::INFINITY, Vec2::NEG_INFINITY), |(low, high), c| (low.min(*c), high.max(*c)));
    let (x0, y0) = (low.x.floor().max(0.0) as u32, low.y.floor().max(0.0) as u32);
    let (x1, y1) = (high.x.ceil().min(SIZE as f32 - 1.0) as u32, high.y.ceil().min(SIZE as f32 - 1.0) as u32);
    if x0 > x1 || y0 > y1 {
        return;
    }
    // Which way round the polygon goes.
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
                let k = ((y * SIZE + x) * 4) as usize;
                data[k..k + 4].copy_from_slice(&colour);
            }
        }
    }
}

/// Draws the land once the map has been read: every triangle of it that
/// faces up and lies low, filled in the land's colour over the background.
fn draw_land(islands: Res<Islands>, mut minimap: ResMut<Minimap>, mut images: ResMut<Assets<Image>>) {
    if minimap.land.is_some() {
        return;
    }
    let Some(corners) = islands.corners(Venue::Arena) else {
        return;
    };
    let colours = ui().colours.arena;
    let mut data = bytes(colours.map_back).repeat((SIZE * SIZE) as usize);
    let land = bytes(colours.map_land);
    for [a, b, c] in corners {
        let normal = (*b - *a).cross(*c - *a);
        if normal.y <= 0.0 || a.y.max(b.y).max(c.y) > LAND_TOP {
            continue;
        }
        fill(&mut data, &[pixel(a.xz()), pixel(b.xz()), pixel(c.xz())], land);
    }
    if let Some(mut image) = images.get_mut(&minimap.image) {
        image.data = Some(data.clone());
    }
    minimap.land = Some(data);
    minimap.drawn = u32::MAX;
    info!("the minimap is drawn from {} triangles", corners.len());
}

/// Draws the bridges over the land again whenever they change, or the
/// file's colours do.
fn draw_bridges(
    bridges: Res<Bridges>,
    changed: Res<UiChanged>,
    mut minimap: ResMut<Minimap>,
    mut images: ResMut<Assets<Image>>,
) {
    if changed.is_changed() && !changed.is_added() {
        minimap.land = None;
        return;
    }
    if minimap.drawn == bridges.version {
        return;
    }
    let Some(land) = minimap.land.clone() else {
        return;
    };
    let mut data = land;
    let plank = bytes(ui().colours.arena.plank);
    for quad in bridges.quads() {
        fill(&mut data, &quad.map(pixel), plank);
    }
    if let Some(mut image) = images.get_mut(&minimap.image) {
        image.data = Some(data);
    }
    minimap.drawn = bridges.version;
}

/// Puts a dot over the picture for you and for each teammate, where each
/// stands, and takes it away with its body; the dead are not shown.
fn place_dots(
    me: Query<(Entity, &Transform, &Venue, Option<&Team>, Has<Dead>), With<Player>>,
    others: Query<(Entity, &Transform, &Team, Has<Dead>), (With<WalkCycle>, Without<Player>)>,
    nodes: Query<Entity, With<MinimapNode>>,
    mut dots: Query<(Entity, &Dot, &mut Node, &mut Visibility)>,
    mut commands: Commands,
) {
    let Ok(node) = nodes.single() else {
        return;
    };
    let Ok((you, at, &here, team, dead)) = me.single() else {
        return;
    };
    let in_arena = here == Venue::Arena;
    // Everyone a dot is for, and where they stand.
    let mut bodies: Vec<(Entity, Vec2, bool, bool)> = vec![(you, at.translation.xz(), true, dead || !in_arena)];
    if in_arena && let Some(mine) = team {
        bodies.extend(
            others
                .iter()
                .filter(|(_, _, theirs, _)| *theirs == mine)
                .map(|(body, at, _, dead)| (body, at.translation.xz(), false, dead)),
        );
    }
    let colours = ui().colours.arena;
    for &(body, at, mine, hidden) in &bodies {
        let place = share(at);
        let shown = if hidden { Visibility::Hidden } else { Visibility::Inherited };
        match dots.iter_mut().find(|(_, dot, ..)| dot.of == body) {
            Some((_, _, mut node, mut visibility)) => {
                let (left, top) = (Val::Percent(place.x - DOT * 0.5), Val::Percent(place.y - DOT * 0.5));
                if node.left != left || node.top != top {
                    node.left = left;
                    node.top = top;
                }
                visibility.set_if_neq(shown);
            }
            None => {
                commands.entity(node).with_child((
                    Dot { of: body },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(place.x - DOT * 0.5),
                        top: Val::Percent(place.y - DOT * 0.5),
                        width: Val::Percent(DOT),
                        height: Val::Percent(DOT),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(if mine { colours.map_me } else { colours.map_mate }),
                    shown,
                    Pickable::IGNORE,
                ));
            }
        }
    }
    for (dot, of, ..) in &dots {
        if !bodies.iter().any(|(body, ..)| *body == of.of) {
            commands.entity(dot).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_picture_has_north_at_the_top_and_the_islands_on_it() {
        // The island at the game's -Z, the north one, is near the top of
        // the picture; the one at +X at its right.
        let north = pixel(Vec2::new(0.0, -130.0));
        assert!(north.y < SIZE as f32 * 0.1 && (north.x - SIZE as f32 * 0.5).abs() < 1.0, "{north}");
        let east = pixel(Vec2::new(130.0, 0.0));
        assert!(east.x > SIZE as f32 * 0.9, "{east}");
        let middle = share(Vec2::ZERO);
        assert!((middle - Vec2::splat(50.0)).length() < 1e-4);
        // A triangle fills the pixels inside it and no others.
        let mut data = vec![0u8; (SIZE * SIZE * 4) as usize];
        fill(&mut data, &[Vec2::new(10.0, 10.0), Vec2::new(30.0, 10.0), Vec2::new(10.0, 30.0)], [255, 255, 255, 255]);
        let at = |x: u32, y: u32| data[((y * SIZE + x) * 4 + 3) as usize];
        assert_eq!(at(12, 12), 255);
        assert_eq!(at(28, 28), 0);
        assert_eq!(at(5, 5), 0);
        // Wound the other way round too.
        let mut other = vec![0u8; (SIZE * SIZE * 4) as usize];
        fill(&mut other, &[Vec2::new(10.0, 30.0), Vec2::new(30.0, 10.0), Vec2::new(10.0, 10.0)], [255, 255, 255, 255]);
        assert_eq!(other[((12 * SIZE + 12) * 4 + 3) as usize], 255);
    }
}
