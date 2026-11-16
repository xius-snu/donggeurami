//! The minimap top right in a game of Bridge Wars (since 2026-11-09,
//! Hajun: "put a minimap that only shows the current player's location as
//! a red dot and your teammate as a green dot, and the full map in top
//! down view including the live bridges but not show the other players"):
//! a picture of the map from above ([`Minimap::image`]), the land in the
//! file's `colours.arena.map_land`, clear round it for the node's own
//! background to show, `map_back` (the picture was filled with `map_back`
//! as well until late on 2026-11-11, so that a see-through one was laid on twice
//! and came out more solid than the file said), drawn once from the
//! map's own triangles as they are read ([`draw_land`]), and the bridges
//! drawn over it again whenever they change ([`draw_bridges`], in the
//! planks' colour); and over the picture two kinds of dot, you in `map_me`
//! and your teammate in `map_mate` ([`place_dots`]), placed as you move.
//! Nobody else is shown. Under the dots, where each sapphire and each
//! amethyst generator is, the picture of what it makes, small
//! ([`mark_generators`], since 2026-11-12; Hajun: "in the minimap, add a
//! smaller sapphire and amethyst logo onto the minimap where the
//! generators are. so the icon need to be smaller so it fits inside the
//! smaller sapphire island in the minimap"). `hud` puts the picture on the screen
//! ([`MinimapNode`]); the top of it is north, as the top of the screen is
//! under the camera fixed over you.
//!
//! **Round each team's island there is a thin line in its team's colour**
//! ([`outline_islands`], the file's `colours.arena.team_red` and the
//! rest; since 2026-11-15, Hajun: "also in the minimap. put a small color
//! outline around each island to indicate which color that island is"),
//! drawn into the picture with the land, just outside the island's edge,
//! so that none of the island is covered. The four smaller islands and
//! the middle are no team's and have none (mine). A bridge built off an
//! island is drawn over its line where it leaves.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use roundtown_net::bridge_wars::{GENERATORS, generator, steady};
use roundtown_net::{ARENA_ISLAND_RADIUS, ARENA_ISLANDS};

use crate::arena::Fighting;
use crate::bridge::Bridges;
use crate::hud;
use crate::island::{Islands, Venue};
use crate::ui::{UiChanged, ui};
use crate::{Dead, Player, Team, WalkCycle};

/// How many pixels across the picture is.
const SIZE: u32 = 256;
/// How far from the middle of the map the picture reaches, in metres, each
/// way: the islands' far edges are 140.4 m out.
const HALF: f32 = 150.0;
/// How high over the ground a triangle may be and still be drawn as land,
/// in metres: the islands' tops are 0.25 m up, the barrels on them 1.43
/// (their desks 1.15, until the map of 2026-11-11); the market stands'
/// roofs, the trees' leaves, the lamps' heads and the waiting room, over
/// that, are not drawn.
const LAND_TOP: f32 = 2.0;
/// How big a dot is, as a share of the picture.
const DOT: f32 = 6.0;
/// How big the picture of what a generator makes is, over its pad on the
/// map, as a share of the picture: the smaller islands, where the
/// sapphire ones are, are 24 m across, 8 of the picture's 100, so that
/// each sits inside its island with the island showing round it. Under
/// two shares of the short side of the screen, the map being 26: a
/// five- and a six-sided dot of the item's colour, the stone its count
/// has top left, drawn for this size (`hud::item_mark`).
const MARK: f32 = 5.5;
/// How thick the line round each team's island is, outward from the
/// island's edge, in pixels of the picture ([`outline_islands`]): a pixel
/// of the picture is a little over a pixel of a screen 1,080 high, and an
/// island is 18 of them across. Mine ("a small color outline"): of 2.5,
/// 3, 3.5 and 4, looked at as a phone draws them, at three quarters size,
/// the thinnest whose yellow still reads against the land's sand.
const OUTLINE: f32 = 3.5;
/// How far past its island's own reach (`ARENA_ISLAND_RADIUS`) land is
/// still that island's, in metres: its edge is where the map has it.
const ISLAND_SLACK: f32 = 4.0;

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

/// The picture of what a generator makes, over its pad on the map.
#[derive(Component)]
struct GeneratorMark;

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
    .add_systems(
        Update,
        (draw_land, draw_bridges, mark_generators, place_dots).chain().after(crate::follow_camera),
    );
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

/// The picture of the land, from every triangle of the map, `corners`:
/// each that faces up and lies low filled in `land`, the rest left clear
/// for the background behind the picture (`hud`); and round each team's
/// island a line in its team's colour (`team`, [`outline_islands`]).
fn land_picture(corners: &[[Vec3; 3]], land: [u8; 4], team: impl Fn(usize) -> [u8; 4]) -> Vec<u8> {
    // Clear, in the land's own colour, so that the land's edge, drawn
    // between a pixel of it and a clear one, shades to nothing rather
    // than to black.
    let mut data = [land[0], land[1], land[2], 0].repeat((SIZE * SIZE) as usize);
    for [a, b, c] in corners {
        let normal = (*b - *a).cross(*c - *a);
        if normal.y <= 0.0 || a.y.max(b.y).max(c.y) > LAND_TOP {
            continue;
        }
        fill(&mut data, &[pixel(a.xz()), pixel(b.xz()), pixel(c.xz())], land);
    }
    outline_islands(&mut data, OUTLINE, team);
    data
}

/// Draws a line round each team's island in its team's colour (`team`, by
/// the team's number), over a picture of the land: every clear pixel
/// outside the land that is within `thick` pixels ([`OUTLINE`]) of a pixel
/// of that island's land. Outside the land is what can be come to from the
/// picture's edge without crossing any, so that a gap in an island, under
/// something tall that is not drawn as land, is left as it is; and an
/// island's land is what lies within its reach of its middle
/// (`ARENA_ISLANDS`, `ARENA_ISLAND_RADIUS`, and [`ISLAND_SLACK`]).
fn outline_islands(data: &mut [u8], thick: f32, team: impl Fn(usize) -> [u8; 4]) {
    let size = SIZE as i32;
    let at = |x: i32, y: i32| (y * size + x) as usize;
    let land: Vec<bool> = data.chunks_exact(4).map(|pixel| pixel[3] != 0).collect();
    let mut outside = vec![false; land.len()];
    let mut edge: Vec<(i32, i32)> = (0..size).flat_map(|k| [(k, 0), (k, size - 1), (0, k), (size - 1, k)]).collect();
    while let Some((x, y)) = edge.pop() {
        if x < 0 || y < 0 || x >= size || y >= size || land[at(x, y)] || outside[at(x, y)] {
            continue;
        }
        outside[at(x, y)] = true;
        edge.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
    }
    let reach = thick.ceil() as i32;
    // (All of them found before any is drawn: a line is not land.)
    let mut lines = Vec::new();
    for (number, middle) in ARENA_ISLANDS.iter().enumerate() {
        let middle = pixel(*middle);
        let far = (ARENA_ISLAND_RADIUS + ISLAND_SLACK) / (2.0 * HALF) * SIZE as f32;
        let its = |x: i32, y: i32| {
            x >= 0
                && y >= 0
                && x < size
                && y < size
                && land[at(x, y)]
                && Vec2::new(x as f32 + 0.5, y as f32 + 0.5).distance(middle) <= far
        };
        let span = (far + thick).ceil() as i32 + 1;
        let (across, down) = (middle.x as i32, middle.y as i32);
        for y in (down - span).max(0)..=(down + span).min(size - 1) {
            for x in (across - span).max(0)..=(across + span).min(size - 1) {
                if !outside[at(x, y)] {
                    continue;
                }
                let near = (-reach..=reach).any(|dy| {
                    (-reach..=reach).any(|dx| ((dx * dx + dy * dy) as f32) <= thick * thick && its(x + dx, y + dy))
                });
                if near {
                    lines.push((at(x, y), number));
                }
            }
        }
    }
    for (k, number) in lines {
        data[k * 4..k * 4 + 4].copy_from_slice(&team(number));
    }
}

/// Draws the land once the map has been read ([`land_picture`]).
fn draw_land(islands: Res<Islands>, mut minimap: ResMut<Minimap>, mut images: ResMut<Assets<Image>>) {
    if minimap.land.is_some() {
        return;
    }
    let Some(corners) = islands.corners(Venue::Arena) else {
        return;
    };
    let colours = ui().colours.arena;
    let data = land_picture(corners, bytes(colours.map_land), |team| bytes(colours.team(team as u8)));
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

/// Marks each sapphire and each amethyst generator on the map as the map
/// is put on the screen (`hud` puts it up afresh with the bar, as the
/// file's colours change too): the picture of what it makes, in the
/// item's colour, small enough to sit inside the smaller islands
/// ([`MARK`]), with its middle on the generator's pad.
fn mark_generators(
    nodes: Query<Entity, Added<MinimapNode>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let colours = ui().colours.arena;
    for node in &nodes {
        // One picture to an item, shared by its four generators.
        let mut pictures = Vec::new();
        for index in 0..GENERATORS {
            let (item, pad) = generator(index);
            if !steady(item) {
                continue;
            }
            let picture = match pictures.iter().find(|(of, _)| *of == item) {
                Some((_, picture)) => Handle::clone(picture),
                None => {
                    let picture = images.add(hud::item_mark(item));
                    pictures.push((item, picture.clone()));
                    picture
                }
            };
            let place = share(pad);
            commands.entity(node).with_child((
                GeneratorMark,
                ImageNode::new(picture).with_color(colours.item(item)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(place.x - MARK * 0.5),
                    top: Val::Percent(place.y - MARK * 0.5),
                    width: Val::Percent(MARK),
                    height: Val::Percent(MARK),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        }
    }
}

/// Puts a dot over the picture for you and for each teammate, where each
/// stands, and takes it away with its body; the dead are not shown. A dot
/// is over whatever else is on the map, the generators' marks among it.
/// Your teammate has none until the fight begins and the teams are out
/// (`Fighting::teams_out`, since 2026-11-16): in the waiting room it said
/// who was with you.
fn place_dots(
    fighting: Res<Fighting>,
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
    if in_arena && fighting.teams_out && let Some(mine) = team {
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
                    ZIndex(1),
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

    #[test]
    fn each_team_s_island_has_a_thin_line_round_it_in_its_team_s_colour() {
        // The map as it is, drawn as the game draws it, the land white
        // and each team's line a colour of its own.
        let parts = crate::island::glb::read("pvptownmap.glb");
        let corners: Vec<[Vec3; 3]> = parts.iter().filter(|part| !part.skinned).flat_map(|part| part.triangles.iter().copied()).collect();
        let (land, lines) = ([255u8, 255, 255, 255], [[255u8, 0, 0, 255], [0, 0, 255, 255], [0, 255, 0, 255], [255, 255, 0, 255]]);
        let data = land_picture(&corners, land, |team| lines[team]);
        let pixels: Vec<[u8; 4]> = data.chunks_exact(4).map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]]).collect();
        let middle_of = |k: usize| Vec2::new((k as u32 % SIZE) as f32 + 0.5, (k as u32 / SIZE) as f32 + 0.5);
        let metres = 2.0 * HALF / SIZE as f32;
        for (team, line) in lines.iter().enumerate() {
            let island = pixel(ARENA_ISLANDS[team]);
            let drawn: Vec<Vec2> = (0..pixels.len()).filter(|&k| pixels[k] == *line).map(middle_of).collect();
            // A ring of it: about as many pixels as a line that thick
            // round a disc that big comes to, all of them just outside
            // the island's edge, and all the way round it.
            let round = std::f32::consts::TAU * (ARENA_ISLAND_RADIUS / metres + OUTLINE * 0.5) * OUTLINE;
            assert!((drawn.len() as f32) > round * 0.6 && (drawn.len() as f32) < round * 1.6, "team {team}: {} pixels, about {round} looked for", drawn.len());
            for at in &drawn {
                let out = at.distance(island) * metres;
                assert!(out > ARENA_ISLAND_RADIUS - 2.5 && out < ARENA_ISLAND_RADIUS + ISLAND_SLACK + (OUTLINE + 1.0) * metres, "team {team}: a pixel of its line {out} m from its middle");
            }
            for eighth in 0..8 {
                let way = Vec2::from_angle(eighth as f32 * std::f32::consts::FRAC_PI_4);
                assert!(drawn.iter().any(|at| (*at - island).normalize().dot(way) > 0.95), "team {team}: no line toward {way}");
            }
            // The island itself is still land, under none of it.
            let k = (island.y as u32 * SIZE + island.x as u32) as usize;
            assert_eq!(pixels[k], land, "team {team}: the middle of its island");
        }
        // Nothing else has a line: every pixel is land, clear, or one of
        // the four lines', and the four smaller islands and the middle
        // have none near them.
        for (k, pixel_) in pixels.iter().enumerate() {
            assert!(*pixel_ == land || pixel_[3] == 0 || lines.contains(pixel_), "{pixel_:?}");
            if lines.contains(pixel_) {
                let at = (middle_of(k) / SIZE as f32 - Vec2::splat(0.5)) * 2.0 * HALF;
                assert!(ARENA_ISLANDS.iter().any(|island| island.distance(at) < ARENA_ISLAND_RADIUS + ISLAND_SLACK + 6.0), "a line at {at}");
            }
        }
        // A gap inside an island is not lined, only its outside.
        let mut data = [255u8, 255, 255, 0].repeat((SIZE * SIZE) as usize);
        let island = pixel(ARENA_ISLANDS[0]);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let from = Vec2::new(x as f32 + 0.5, y as f32 + 0.5).distance(island);
                if from < 8.0 && from > 2.0 {
                    data[((y * SIZE + x) * 4 + 3) as usize] = 255;
                }
            }
        }
        outline_islands(&mut data, OUTLINE, |team| lines[team]);
        let k = ((island.y as u32 * SIZE + island.x as u32) * 4) as usize;
        assert_eq!(data[k + 3], 0, "the gap in the middle is still clear");
        let lined = data.chunks_exact(4).filter(|pixel| *pixel == lines[0].as_slice()).count();
        assert!(lined > 80, "{lined}");
    }

    #[test]
    fn a_generator_s_mark_sits_inside_the_island_its_pad_is_on() {
        use roundtown_net::{AMETHYST_PADS, Item, SAPPHIRE_ISLAND_RADIUS, SAPPHIRE_PADS};
        // Eight marks: the four sapphire generators and the four amethyst
        // ones, each on the map, the sapphire ones out on the diagonals
        // and the amethyst ones round the middle.
        let marked: Vec<(Item, Vec2)> = (0..GENERATORS).map(generator).filter(|(item, _)| steady(*item)).collect();
        assert_eq!(marked.len(), 8);
        for (item, pad) in &marked {
            assert!(matches!(item, Item::Sapphire | Item::Amethyst));
            let place = share(*pad);
            assert!(place.min_element() > MARK && place.max_element() < 100.0 - MARK, "{item:?} at {place}");
        }
        assert!(SAPPHIRE_PADS.iter().chain(&AMETHYST_PADS).all(|pad| marked.iter().any(|(_, at)| at == pad)));
        // A mark is smaller than the smaller islands are on the map, with
        // some of the island showing all round it, and no bigger than a
        // player's dot.
        let island = 2.0 * SAPPHIRE_ISLAND_RADIUS / (2.0 * HALF) * 100.0;
        assert!(MARK < island * 0.75 && MARK > island * 0.5, "a mark {MARK} on an island {island} across");
        assert!(MARK <= DOT);
        // The amethyst generators' marks stand clear of each other.
        for (k, a) in AMETHYST_PADS.iter().enumerate() {
            for b in &AMETHYST_PADS[k + 1..] {
                assert!(share(*a).distance(share(*b)) > 2.0 * MARK);
            }
        }
    }
}
