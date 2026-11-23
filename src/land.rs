//! Land: the town's plots, and the sign on yours that says whose it is
//! (Hajun, 2026-11-20: "make it so each player, when they join the lobby,
//! they get assigned a plot and their plot has the plotsign in the right
//! side corner (from when looking from pond, bottom right corner so its
//! near the pond and facing the pond. or if the house is next to a street,
//! the fence should be bottom right corner from looking from that street
//! and facing the street. and the plotsign should just say "hajun's
//! land"."). The first of a land of one's own in the town (their thinking
//! that week: everyone has land from the start, and a lobby's plots are
//! given out among whoever is in it).
//!
//! **The plots are Hajun's**, the 48 planes of the material `plot` in the
//! town's map, read as the map spawns ([`read_map`], [`read_plots`]): each
//! one's four corners, its ring out from the pond, and its front, the side
//! its sign stands on. Nothing of them is kept by hand: move a plot in
//! Blender and its sign moves with it.
//!
//! **A plot's front** is the side with a street along it, the street's
//! surface within [`STREET_REACH`] of the side's middle (the map's `road`,
//! where nothing lies over it: the town's pavements do); with a street on
//! more than one side, the one more toward the pond (the ring road round
//! the pond before a street that runs out from it: mine); with none, the
//! side toward the pond. **The sign** stands inside the plot at the right
//! hand end of that side, as someone on the street, or at the pond,
//! looking at the plot has it, and faces them.
//!
//! **Whose plot is whose** (since 2026-11-23; Hajun: "i want to change it
//! so that at all times, there are 48 plots filled"): a seat of the town
//! is one of its 48 plots, numbered as Hajun numbered them
//! (`plotorder.png`, `roundtown_net::land::HOMES`, [`in_order`]), and the
//! seat the server gives you is your plot. Every other plot is someone's
//! too ([`Deeds`]): online as the server says (`roundtown_net::Tell::
//! Plots`: whoever else is in your channel, in the town or away in a game,
//! and the land of people who are not here, real and made up, that fills
//! the rest); offline each plot's own made-up owner
//! (`roundtown_net::premade`). A sign stands on every one, with its
//! owner's name ([`holders`]), and what they own stands there
//! (`build::stand`); someone of the computer's walks each of the first
//! sixteen that nobody here has (`resident`). Until then a seat was one of
//! sixteen, each device working a plot out for it, and only the plots of
//! whoever was in your channel had anything on them. `RT_LAND_SIGNS=all`
//! on desktop stands a sign with your own name on every plot, to look at
//! them.
//!
//! **The words** ("{name}'s land", `words.land.sign`, in
//! `colours.land.sign`) are a thing in the world, on both faces of the
//! board: drawn once into a picture with the game's own fonts
//! ([`draw_words`], [`words_picture`]), as big as fit the board, and hung a
//! few millimetres in front of each face on a flat sheet ([`keep_signs`]).
//! So they are seen as the board is, from any side and any distance, and
//! behind whatever stands in front of them; unlit, so that they are their
//! own colour in sun and shade, by day and by night. (For their first
//! two hours they were text on the screen, moved and slanted every frame
//! to lie over the board, as names over heads are: it could not narrow
//! toward the far end of a board seen from the side, and up close it came
//! apart. Hajun: "why dont u make like a in-world text obj rather than
//! trying to rotate/warp the text? even now, when the camera zooms in very
//! close to the text, the text warps weird.")
//!
//! **You arrive in the middle of your plot**, facing the fountain
//! (`island::arrival_of_a_person`, `roundtown_net::land`), as the server
//! has it: it has no map, and goes by the plots' middles written there,
//! which a test here holds to the map.
//!
//! **The sign is solid** (Hajun, 2026-11-21: "oh sign need to be solid
//! tho"): a box as wide and as tall as it and as deep as its posts,
//! standing where it stands, is laid over the town's own shape
//! ([`solid`], `Islands::lay_over`), which is what every body is kept out
//! of, the town's people and the car you drive with them. A box, and not
//! the model's own triangles: its board is thinner than a body is tested
//! against (`island::LATTICE`), and nobody passes under it or between its
//! chains. The town's shape is its model's, which a game of Blend In is
//! played on too: the boxes stand there unseen, out past the ring nobody
//! in that game can cross.

use bevy::asset::RenderAssetUsages;
use bevy::gltf::GltfMaterialName;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::light::NotShadowCaster;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::world_serialization::WorldInstanceReady;
use roundtown_net::Tell;
use roundtown_net::land::{Deed, HOMES};
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::{Format, Vector};

use crate::Player;
use crate::island::{self, Islands, Land, Venue, is_material};
use crate::lobby::Seat;
use crate::net::Online;
use crate::ui::{UiChanged, fill, ui};

/// What a plot is made of in the town's map, and what a street is: the
/// road, and the paint on it.
const PLOT_MATERIAL: &str = "plot";
const STREET_MATERIALS: [&str; 2] = ["road", "roadwhite"];
/// How far out from the middle of a plot's side a street may begin and the
/// plot still be next to it, in metres, and how far apart it is looked for:
/// the pavement between a plot and the street that runs out from the pond
/// is 7.5 m, to the ring road round the pond 10.5 to 12, to the road round
/// the town's edge 10 (and 14.5 and 20 behind the plots that stand further
/// in from it, which are not next to it).
const STREET_REACH: f32 = 13.0;
const STREET_STEP: f32 = 0.5;
/// How far over a street's surface something may lie and the street still
/// be what is on top there (its paint, 0.02 m up), and how far over it
/// anything is no longer lying on it at all (a lamp's head).
const PAINT_OVER: f32 = 0.05;
const OVERHEAD: f32 = 1.0;
/// How much further out than the one before a plot has to be to begin the
/// next ring, in metres: the town's three are 93, 126 and 156 to 166 m out.
const RING_GAP: f32 = 15.0;

/// The sign's model, Hajun's: two posts and a beam, and a board hung from
/// it by two chains.
pub(crate) const MODEL: &str = "plotsign.glb";
/// How wide the sign is, post to post, in metres; how far in from the
/// plot's corner its end stands, and how far in from the plot's edge its
/// middle: a metre and a half each. Half a metre and 0.6 for its first
/// hour, until Hajun asked for "more margin from the sign to the edges of
/// the corner"; two metres for the next, until "the sign can have slightly
/// less margin from the corner". The numbers are mine.
const SIGN_WIDE: f32 = 3.06;
/// How tall the sign is, and how deep the box that stands for it in the
/// town's shape, through from front to back: its posts are 0.27 m, and a
/// body is tested against the town a quarter of a metre at a time
/// (`island::LATTICE`), so anything thinner could be slipped through.
const SIGN_HIGH: f32 = 2.73;
const SIGN_DEEP: f32 = 0.3;
const FROM_CORNER: f32 = 1.5;
const FROM_EDGE: f32 = 1.5;
/// The board: how high its middle hangs over the sign's foot, how wide and
/// how tall the room for words inside its frame is (two to one, as their
/// picture is), and how far out from the sign's middle the words hang on
/// its front and on its back, a few millimetres clear of each face and
/// inside the frame, in metres. A test holds these to the model.
const BOARD_UP: f32 = 1.48;
const BOARD_WIDE: f32 = 1.3;
const BOARD_HIGH: f32 = 0.65;
const FRONT_OUT: f32 = 0.05;
const BACK_OUT: f32 = 0.032;

/// How much of the board's room the words' picture may fill, across and
/// down; how many pixels of their picture go to a metre of
/// the board (as many as the screen gives a metre a little over a metre
/// off: nearer than that they are drawn larger than they were made, and
/// soften); how much of the picture is left clear round them, in ems, so
/// that no letter is cut and its edge stays clean drawn small; and how many
/// times as bright as its colour the picture is drawn, being unlit, to come
/// out that colour (`range::GAIN`, `sky::UNLIT_GAIN`).
const WORDS_FILL: (f32, f32) = (0.94, 0.92);
const PIXELS_A_METRE: f32 = 640.0;
const PICTURE_PAD: f32 = 0.12;
const WORDS_GAIN: f32 = 2.5;
/// How near the camera a sign has to come for its words to be hung on it,
/// how far it has to go for them to be taken off again, in metres, and how
/// often that is looked at, in seconds. The words of the shortest name are
/// 0.22 m to the em: 45 m off they are two dots tall on a phone, and
/// further off they are nothing that can be read. The numbers are mine.
const WORDS_NEAR: f32 = 45.0;
const WORDS_FAR: f32 = 60.0;
const WORDS_EVERY: f32 = 0.25;

/// One of the town's plots, as the map has it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Plot {
    /// Its middle, across the ground, and how high its ground is.
    pub middle: Vec2,
    pub ground: f32,
    /// Its four corners across the ground, in order round it.
    #[cfg_attr(not(test), allow(dead_code))]
    pub corners: [Vec2; 4],
    /// Which ring of plots out from the pond it is in, from 0.
    pub ring: usize,
    /// Which way its front looks, out over the side its sign stands on:
    /// at the street along that side, or at the pond.
    pub front: Vec2,
    /// Whether that side has a street along it.
    pub street: bool,
    /// Where the middle of its sign stands, across the ground.
    pub sign: Vec2,
}

impl Plot {
    /// Which way is right to someone in front of the plot looking at it,
    /// across the ground.
    #[cfg(test)]
    fn right(&self) -> Vec2 {
        right_of(-self.front)
    }
}

/// Which way is right to someone looking along `ahead`, across the ground
/// (x and z): looking north, the game's -Z, right is east, +X.
fn right_of(ahead: Vec2) -> Vec2 {
    Vec2::new(-ahead.y, ahead.x)
}

/// The town's plots, ring by ring out from the pond and round each ring,
/// and which of them each seat of the town is ([`in_order`]: a seat is a
/// plot by Hajun's number for it, less one). Empty until the town's map
/// has spawned.
#[derive(Resource, Default)]
pub(crate) struct Plots {
    pub all: Vec<Plot>,
    pub of_seat: Vec<usize>,
}

impl Plots {
    /// The plot that goes by the number `seat`, if the map has it.
    pub(crate) fn of(&self, seat: usize) -> Option<&Plot> {
        self.all.get(*self.of_seat.get(seat)?)
    }
}

/// Whose every plot of the town is but your own: its number
/// (`roundtown_net::land::HOMES`), its owner, the name on its sign, the
/// look of whoever walks it, and who is on it. Online as the server last
/// said ([`roundtown_net::Tell::Plots`]); offline, and until it has said,
/// each plot's own made-up owner (`roundtown_net::land::offline_deeds`).
#[derive(Resource, Default, PartialEq, Debug)]
pub(crate) struct Deeds(pub Vec<Deed>);

impl Deeds {
    /// The plot `who` has, by its number, if they have one here.
    pub(crate) fn plot_of(&self, who: u64) -> Option<usize> {
        self.0.iter().find(|deed| deed.who == who).map(|deed| usize::from(deed.plot))
    }

    /// Whether `who` has a plot here.
    pub(crate) fn has(&self, who: u64) -> bool {
        self.0.iter().any(|deed| deed.who == who)
    }
}

/// Hearing whose each plot is: whatever stands anything on them comes
/// after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DeedSystems;

/// One of the two sheets a sign's words hang on, while they do
/// ([`word_the_near`]).
#[derive(Component)]
struct Sheet;

/// A sign standing on a plot: which, and what it says.
#[derive(Component)]
struct Sign {

    plot: usize,
    says: String,
}

/// Whether a sign stands on every plot, yours or not, to look at them:
/// `RT_LAND_SIGNS=all` on desktop.
#[derive(Resource)]
struct Everywhere(bool);

pub(crate) fn plugin(app: &mut App) {
    let everywhere = std::env::var("RT_LAND_SIGNS").is_ok_and(|said| said.trim() == "all");
    app.init_resource::<Plots>()
        .init_resource::<Deeds>()
        .insert_resource(Everywhere(everywhere))
        .add_observer(read_map)
        .add_systems(Update, (keep_deeds.in_set(DeedSystems), keep_signs, word_the_near).chain());
}

/// Keeps [`Deeds`]: online, what the server says of your channel's plots,
/// yours left out; offline, and until it has said anything, every plot but
/// yours with its own made-up owner. Your plot being the seat you have in
/// the town, which a game changes for its own while you are in one, the
/// plots are left as they were anywhere but the town. Written only as it
/// changes.
fn keep_deeds(
    mut tells: MessageReader<Tell>,
    online: Res<Online>,
    me: Query<(&Seat, &Venue), With<Player>>,
    mut said: Local<Option<Vec<Deed>>>,
    mut deeds: ResMut<Deeds>,
) {
    for tell in tells.read() {
        if let Tell::Plots(all) = tell {
            *said = Some(all.clone());
        }
    }
    if !online.is() {
        *said = None;
    }
    let Ok((seat, venue)) = me.single() else {
        return;
    };
    if *venue != Venue::Town {
        return;
    }
    let mine = online.me().map(|me| me.id);
    let wanted = match said.as_ref() {
        Some(all) => all
            .iter()
            .filter(|deed| Some(deed.who) != mine && usize::from(deed.plot) != seat.0)
            .cloned()
            .collect(),
        None => roundtown_net::land::offline_deeds(seat.0),
    };
    if deeds.0 != wanted {
        deeds.0 = wanted;
    }
}

/// How high `triangle` is over or under `at`, across the ground, if it
/// lies there at all. A face that stands upright lies over nowhere.
fn height_at(triangle: &[Vec3; 3], at: Vec2) -> Option<f32> {
    let [a, b, c] = triangle.map(|corner| corner.xz());
    let whole = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if whole.abs() < 1e-9 {
        return None;
    }
    let u = ((b.y - c.y) * (at.x - c.x) + (c.x - b.x) * (at.y - c.y)) / whole;
    let v = ((c.y - a.y) * (at.x - c.x) + (a.x - c.x) * (at.y - c.y)) / whole;
    let w = 1.0 - u - v;
    (u >= -1e-4 && v >= -1e-4 && w >= -1e-4).then(|| u * triangle[0].y + v * triangle[1].y + w * triangle[2].y)
}

/// Whether the street is what is on top at `at`: one of `streets` lies
/// there, and nothing of `rest` lies on it (a pavement does, a quarter of
/// a metre up; a lamp's head high over it does not).
fn street_at(at: Vec2, streets: &[[Vec3; 3]], rest: &[[Vec3; 3]]) -> bool {
    let Some(street) = streets.iter().filter_map(|triangle| height_at(triangle, at)).reduce(f32::max) else {
        return false;
    };
    !rest
        .iter()
        .filter_map(|triangle| height_at(triangle, at))
        .any(|over| over > street + PAINT_OVER && over < street + OVERHEAD)
}

/// The corners of each separate piece of `triangles`: those that share a
/// corner are one piece's. A plot is a piece of four corners.
fn pieces(triangles: &[[Vec3; 3]]) -> Vec<Vec<Vec3>> {
    let same = |a: Vec3, b: Vec3| a.distance_squared(b) < 1e-4;
    let mut pieces: Vec<Vec<Vec3>> = Vec::new();
    for triangle in triangles {
        let mut joined: Vec<Vec3> = triangle.to_vec();
        let mut apart = Vec::new();
        for piece in pieces.drain(..) {
            if piece.iter().any(|&corner| triangle.iter().any(|&other| same(corner, other))) {
                joined.extend(piece);
            } else {
                apart.push(piece);
            }
        }
        let mut corners: Vec<Vec3> = Vec::new();
        for corner in joined {
            if !corners.iter().any(|&had| same(had, corner)) {
                corners.push(corner);
            }
        }
        apart.push(corners);
        pieces = apart;
    }
    pieces
}

/// The triangles of `all` that reach into the box from `low` to `high`,
/// across the ground.
fn within(all: &[[Vec3; 3]], low: Vec2, high: Vec2) -> Vec<[Vec3; 3]> {
    all.iter()
        .filter(|triangle| {
            let corners = triangle.map(|corner| corner.xz());
            let least = corners[0].min(corners[1]).min(corners[2]);
            let most = corners[0].max(corners[1]).max(corners[2]);
            least.cmple(high).all() && most.cmpge(low).all()
        })
        .copied()
        .collect()
}

/// One side of a plot: its two ends, which way it looks out, how far
/// toward the pond that is (1 straight at it), and whether a street runs
/// along it.
#[derive(Clone, Copy)]
struct Side {
    ends: [Vec2; 2],
    out: Vec2,
    toward_pond: f32,
    street: bool,
}

/// Whichever of `sides` looks most toward the pond, the first of them if
/// two look as much.
fn most_toward_the_pond<'a>(sides: impl Iterator<Item = &'a Side>) -> Option<Side> {
    sides.fold(None, |best: Option<Side>, &side| match best {
        Some(best) if best.toward_pond >= side.toward_pond - 1e-3 => Some(best),
        _ => Some(side),
    })
}

/// The plots of a map, given the triangles of its `plot` material, of its
/// `streets` and of the `rest` of its land, the pond at its middle: ring
/// by ring out from the pond, and round each ring from the game's +Z by
/// way of +X. A piece of `plot` that has not four corners is no plot.
pub(crate) fn read_plots(plot: &[[Vec3; 3]], streets: &[[Vec3; 3]], rest: &[[Vec3; 3]]) -> Vec<Plot> {
    let mut plots: Vec<Plot> = pieces(plot)
        .into_iter()
        .filter_map(|corners| {
            let corners: [Vec3; 4] = corners.try_into().ok()?;
            let middle = corners.iter().copied().sum::<Vec3>() / 4.0;
            let (ground, middle) = (middle.y, middle.xz());
            let angle = |corner: &Vec2| (corner.x - middle.x).atan2(corner.y - middle.y);
            let mut round = corners.map(|corner| corner.xz());
            round.sort_by(|a, b| angle(a).total_cmp(&angle(b)));
            // What of the map is near enough to be looked at from its sides.
            let reach = Vec2::splat(STREET_REACH + 1.0);
            let low = round.iter().copied().reduce(Vec2::min)? - reach;
            let high = round.iter().copied().reduce(Vec2::max)? + reach;
            let (streets, rest) = (within(streets, low, high), within(rest, low, high));
            let toward_pond = (-middle).normalize_or_zero();
            let sides: Vec<Side> = (0..4)
                .map(|side| {
                    let ends = [round[side], round[(side + 1) % 4]];
                    let from = (ends[0] + ends[1]) * 0.5;
                    let out = (from - middle).normalize_or_zero();
                    let steps = (STREET_REACH / STREET_STEP) as usize;
                    let street = (1..=steps)
                        .map(|step| step as f32 * STREET_STEP)
                        .any(|far| street_at(from + out * far, &streets, &rest));
                    Side {
                        ends,
                        out,
                        toward_pond: out.dot(toward_pond),
                        street,
                    }
                })
                .collect();
            // The side with a street that is most toward the pond; with no
            // street, the side most toward the pond.
            let front = most_toward_the_pond(sides.iter().filter(|side| side.street))
                .or_else(|| most_toward_the_pond(sides.iter()))?;
            // The right hand end of that side, to whoever looks at the
            // plot from in front of it, and the sign in from it.
            let right = right_of(-front.out);
            let [a, b] = front.ends;
            let corner = if (a - b).dot(right) > 0.0 { a } else { b };
            let sign = corner - right * (SIGN_WIDE * 0.5 + FROM_CORNER) - front.out * FROM_EDGE;
            Some(Plot {
                middle,
                ground,
                corners: round,
                ring: 0,
                front: front.out,
                street: front.street,
                sign,
            })
        })
        .collect();
    // Ring by ring out from the pond, and round each.
    plots.sort_by(|a, b| a.middle.length().total_cmp(&b.middle.length()));
    let (mut ring, mut last) = (0, plots.first().map_or(0.0, |plot| plot.middle.length()));
    for plot in &mut plots {
        let out = plot.middle.length();
        if out - last > RING_GAP {
            ring += 1;
        }
        last = out;
        plot.ring = ring;
    }
    let round = |plot: &Plot| plot.middle.x.atan2(plot.middle.y).rem_euclid(std::f32::consts::TAU);
    plots.sort_by(|a, b| a.ring.cmp(&b.ring).then(round(a).total_cmp(&round(b))));
    plots
}

/// Which of `plots` each seat of the town is: a seat is a plot by Hajun's
/// number for it less one, and its middle is written for the server, which
/// has no map (`roundtown_net::land::HOMES`): each written middle's plot
/// is the one of the map that stands there. As many as are found, in
/// order, up to the first that is not: a map whose plots have moved from
/// where they are written has to have them written again (a test says so).
pub(crate) fn in_order(plots: &[Plot]) -> Vec<usize> {
    HOMES
        .iter()
        .map_while(|&home| plots.iter().position(|plot| plot.middle.distance(Vec2::from(home)) < 0.5))
        .collect()
}

/// Reads the plots off the town's map once it has spawned.
fn read_map(
    ready: On<WorldInstanceReady>,
    lands: Query<&Venue, With<Land>>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, Option<&GltfMaterialName>, Has<SkinnedMesh>)>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    mut plots: ResMut<Plots>,
) {
    let Ok(&Venue::Town) = lands.get(ready.entity) else {
        return;
    };
    let (mut plot, mut streets, mut rest) = (Vec::new(), Vec::new(), Vec::new());
    for part in children.iter_descendants(ready.entity) {
        let Ok((mesh, material, skinned)) = parts.get(part) else {
            continue;
        };
        let named = |wanted: &str| material.is_some_and(|name| is_material(&name.0, wanted));
        // Neither water nor anyone modelled into the map is land.
        if skinned || named(island::WATER_MATERIAL) {
            continue;
        }
        let (Some(mesh), Ok(place)) = (meshes.get(&mesh.0), placement.compute_global_transform(part)) else {
            continue;
        };
        let into = if named(PLOT_MATERIAL) {
            &mut plot
        } else if STREET_MATERIALS.iter().any(|street| named(street)) {
            &mut streets
        } else {
            &mut rest
        };
        island::add_mesh(into, mesh, &place);
    }
    plots.all = read_plots(&plot, &streets, &rest);
    plots.of_seat = in_order(&plots.all);
    if plots.all.is_empty() {
        warn!("the town's map has no plots: nothing in it is of the material {PLOT_MATERIAL:?}");
        return;
    }
    info!(
        "the town's plots: {}, {} of them with a street at their front; a seat's plot for {} seats",
        plots.all.len(),
        plots.all.iter().filter(|plot| plot.street).count(),
        plots.of_seat.len()
    );
}

/// Whose sign stands on which plot: each of `plots` (a seat's plot, seat
/// by seat) with the name on it. Yours, in `seat`, by your own `name`; and
/// every other plot there is a deed for (`deeds`), by its owner's. A plot
/// said to be someone else's that is the one this device has you on is
/// yours here. In the order of the plots.
fn holders(plots: &[usize], (seat, name): (usize, &str), deeds: &[Deed]) -> Vec<(usize, String)> {
    let mut held: Vec<(usize, String)> = deeds
        .iter()
        .filter_map(|deed| Some((*plots.get(usize::from(deed.plot))?, deed.name.clone())))
        .collect();
    if let Some(&plot) = plots.get(seat) {
        held.retain(|(theirs, _)| *theirs != plot);
        held.push((plot, name.to_string()));
    }
    held.sort();
    held.dedup_by_key(|(plot, _)| *plot);
    held
}

/// Which of the signs `standing`, each its plot and its words, stay as they
/// are, those `wanted` now being what they are: each that is wanted still,
/// once. The rest go, and what is wanted and does not stand is stood, so
/// that someone coming into your channel or leaving it costs the one sign.
fn staying(standing: &[(usize, String)], wanted: &[(usize, String)]) -> Vec<bool> {
    let mut stays = Vec::with_capacity(standing.len());
    for (index, sign) in standing.iter().enumerate() {
        stays.push(wanted.contains(sign) && !standing[..index].contains(sign));
    }
    stays
}

/// Stands a sign on every plot of the town, yours and everyone else's
/// ([`Deeds`]), each to say whose land it is on both faces of its board,
/// and takes them away again as that changes: as the plots are read, as
/// you are seated or named, as whose a plot is changes, and as the file of
/// colours and words changes on desktop. Nothing is touched otherwise, nor
/// while you are anywhere but the town, where no sign is drawn. Each is
/// solid too ([`solid`]).
///
/// The words are a picture of them on two flat sheets, children of the
/// sign, so that they stand, turn and are hidden with it: unlit, their own
/// colour wherever the sun is, and drawn over the board where the two are
/// too near to tell apart from far off. They are hung on a sign only while
/// the camera is near it ([`word_the_near`]).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn keep_signs(
    plots: Res<Plots>,
    file: Res<UiChanged>,
    everywhere: Res<Everywhere>,
    deeds: Res<Deeds>,
    me: Query<(Ref<Name>, Ref<Seat>, Ref<Venue>), With<Player>>,
    signs: Query<(Entity, &Sign)>,
    assets: Res<AssetServer>,
    mut islands: ResMut<Islands>,
    mut commands: Commands,
) {
    let Ok((name, seat, venue)) = me.single() else {
        return;
    };
    let reworded = file.is_changed() && !file.is_added();
    let stirred = plots.is_changed()
        || reworded
        || name.is_changed()
        || seat.is_changed()
        || venue.is_changed()
        || deeds.is_changed();
    if !stirred {
        return;
    }
    if *venue != Venue::Town {
        return;
    }
    let ui = ui();
    let says = |name: &str| fill(&ui.words.land.sign, &[("name", name)]);
    let wanted: Vec<(usize, String)> = if everywhere.0 {
        (0..plots.all.len()).map(|plot| (plot, says(name.as_str()))).collect()
    } else {
        holders(&plots.of_seat, (seat.0, name.as_str()), &deeds.0)
            .into_iter()
            .map(|(plot, name)| (plot, says(&name)))
            .collect()
    };
    let standing: Vec<(usize, String)> = signs.iter().map(|(_, sign)| (sign.plot, sign.says.clone())).collect();
    // Each that stands and is still wanted stays as it is; with the file
    // changed, none, since its colours may have.
    let stays = if reworded { vec![false; standing.len()] } else { staying(&standing, &wanted) };
    if stays.iter().all(|stays| *stays) && standing.len() == wanted.len() {
        return;
    }
    let mut kept: Vec<&(usize, String)> = Vec::new();
    for (((sign, _), stays), stood) in signs.iter().zip(&stays).zip(&standing) {
        if *stays {
            kept.push(stood);
        } else {
            commands.entity(sign).despawn();
        }
    }
    // And each as something solid, in the town's shape: those that stand
    // now, in place of those that stood.
    let solids: Vec<Vec<[Vec3; 3]>> = wanted.iter().map(|&(index, _)| solid(&plots.all[index])).collect();
    islands.lay_over(Venue::Town, (!solids.is_empty()).then_some(solids));
    // Each stands bare: its words are hung on it while anyone is near
    // enough to read them ([`word_the_near`]).
    for (index, says) in wanted.iter().filter(|sign| !kept.contains(sign)).cloned() {
        let plot = &plots.all[index];
        let foot = Vec3::new(plot.sign.x, plot.ground, plot.sign.y);
        let front = Vec3::new(plot.front.x, 0.0, plot.front.y);
        commands.spawn((
            Sign { plot: index, says },
            Name::new("Plot sign"),
            // On the town, and drawn only there (`island::show_where_i_am`).
            Venue::Town,
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(MODEL))),
            // A model's front is its -Z as the loader has it.
            Transform::from_translation(foot).looking_to(front, Vec3::Y),
            Visibility::Hidden,
        ));
    }
}

/// Whether a sign `far` metres from the camera is to have its words on it,
/// having them now or not (`worded`): once it comes within [`WORDS_NEAR`],
/// until it is further than [`WORDS_FAR`].
fn words_wanted(far: f32, worded: bool) -> bool {
    far <= if worded { WORDS_FAR } else { WORDS_NEAR }
}

/// Hangs its words on each sign the camera is near, and takes them off
/// those it has gone far from ([`words_wanted`]), looked at
/// [`WORDS_EVERY`] seconds: a picture of them ([`draw_words`],
/// [`words_picture`]) on two flat sheets as big as it, one just clear of
/// each face of the board and looking out from it, children of the sign.
/// Since 2026-11-23, when every plot of the town came to have a sign: a
/// picture is a megabyte, kept here and on the graphics card, and each
/// sheet a draw of its own, where the words of a sign across the town are
/// smaller than a dot of the screen. One picture is drawn a look, the
/// nearest sign's first.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn word_the_near(
    time: Res<Time>,
    mut since: Local<f32>,
    cameras: Query<&GlobalTransform, With<crate::ThirdPersonCamera>>,
    me: Query<&Venue, With<Player>>,
    signs: Query<(Entity, &Sign, &Transform)>,
    words: Query<(Entity, &ChildOf), With<Sheet>>,
    (mut images, mut meshes, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    mut commands: Commands,
) {
    *since += time.delta_secs();
    if *since < WORDS_EVERY {
        return;
    }
    *since = 0.0;
    let (Ok(camera), Ok(&Venue::Town)) = (cameras.single(), me.single()) else {
        return;
    };
    let eye = camera.translation();
    let mut bare: Option<(f32, Entity, &Sign)> = None;
    for (sign, of, at) in &signs {
        let far = at.translation.distance(eye);
        let worded = words.iter().any(|(_, on)| on.parent() == sign);
        match (worded, words_wanted(far, worded)) {
            (true, false) => {
                for (sheet, _) in words.iter().filter(|(_, on)| on.parent() == sign) {
                    commands.entity(sheet).despawn();
                }
            }
            (false, true) if bare.is_none_or(|(nearest, ..)| far < nearest) => bare = Some((far, sign, of)),
            _ => {}
        }
    }
    let Some((_, sign, of)) = bare else {
        return;
    };
    let ink = ui().colours.land.sign.to_linear();
    let drawn = draw_words(&of.says);
    let sheet = meshes.add(Rectangle::from_size(drawn.metres()));
    let paint = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(ink.red * WORDS_GAIN, ink.green * WORDS_GAIN, ink.blue * WORDS_GAIN, ink.alpha),
        base_color_texture: Some(images.add(words_picture(&drawn))),
        // Their own colour on either face, in sun and in shade, by day and
        // by night, as words on the screen are.
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        // In front of the board it hangs on, however far off the two are
        // seen from.
        depth_bias: 4.0,
        ..default()
    });
    // A sheet looks out along its own +Z and reads along its own +X: on
    // the back of the board as it is, on the front turned half round,
    // which is right to whoever stands in front of the sign.
    let faces = [
        Transform::from_xyz(0.0, BOARD_UP, -FRONT_OUT).with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        Transform::from_xyz(0.0, BOARD_UP, BACK_OUT),
    ];
    for face in faces {
        commands.spawn((
            Sheet,
            ChildOf(sign),
            Mesh3d(sheet.clone()),
            MeshMaterial3d(paint.clone()),
            face,
            NotShadowCaster,
        ));
    }
}

/// The sign on `plot` as something solid, for the town's shape
/// (`Islands::lay_over`): the triangles of a closed box standing where it
/// stands, as wide as it post to post ([`SIGN_WIDE`]), [`SIGN_DEEP`]
/// through and [`SIGN_HIGH`] tall, its foot a little under the ground so
/// that nothing is left open beneath it. Every face looks outward, its top
/// up, as a solid's have to (`island`).
pub(crate) fn solid(plot: &Plot) -> Vec<[Vec3; 3]> {
    let across = right_of(-plot.front) * (SIGN_WIDE * 0.5);
    let through = plot.front * (SIGN_DEEP * 0.5);
    let (low, high) = (plot.ground - 0.1, plot.ground + SIGN_HIGH);
    let middle = Vec3::new(plot.sign.x, (low + high) * 0.5, plot.sign.y);
    // Its eight corners: left or right, back or front, bottom or top.
    let corner = |side: f32, face: f32, top: bool| {
        let at = plot.sign + across * side + through * face;
        Vec3::new(at.x, if top { high } else { low }, at.y)
    };
    let quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| [[a, b, c], [a, c, d]];
    let mut faces = Vec::with_capacity(12);
    // Top and bottom, front and back, and its two ends.
    faces.extend(quad(corner(-1.0, -1.0, true), corner(1.0, -1.0, true), corner(1.0, 1.0, true), corner(-1.0, 1.0, true)));
    faces.extend(quad(corner(-1.0, -1.0, false), corner(1.0, -1.0, false), corner(1.0, 1.0, false), corner(-1.0, 1.0, false)));
    for face in [-1.0, 1.0] {
        faces.extend(quad(corner(-1.0, face, false), corner(1.0, face, false), corner(1.0, face, true), corner(-1.0, face, true)));
    }
    for side in [-1.0, 1.0] {
        faces.extend(quad(corner(side, -1.0, false), corner(side, 1.0, false), corner(side, 1.0, true), corner(side, -1.0, true)));
    }
    // Each wound so that it looks away from the middle of the box.
    for [a, b, c] in &mut faces {
        let out = (*a + *b + *c) / 3.0 - middle;
        if (*b - *a).cross(*c - *a).dot(out) < 0.0 {
            std::mem::swap(b, c);
        }
    }
    faces
}

/// The words of a sign, drawn: how much of each pixel of their picture
/// they cover, row by row from the top, and how many pixels the picture is
/// across and down. [`PIXELS_A_METRE`] of them go to a metre of the board.
struct Words {
    cover: Vec<u8>,
    wide: usize,
    high: usize,
}

impl Words {
    /// How big the picture is on the board, across and up, in metres.
    fn metres(&self) -> Vec2 {
        Vec2::new(self.wide as f32, self.high as f32) / PIXELS_A_METRE
    }
}

/// `words` as a picture to hang on the board: white, and as see-through as
/// the letters leave it, with every smaller copy of itself down to a
/// pixel, each half the one before, so that from far off the words are
/// drawn from one about their size on the screen and do not sparkle.
fn words_picture(words: &Words) -> Image {
    let (wide, high) = (words.wide, words.high);
    let mut cover = words.cover.clone();
    let (mut wide_now, mut high_now) = (wide, high);
    let mut data = Vec::with_capacity(wide * high * 4 * 4 / 3 + 16);
    let mut levels = 0;
    loop {
        data.extend(cover.iter().flat_map(|&alpha| [255, 255, 255, alpha]));
        levels += 1;
        if wide_now == 1 && high_now == 1 {
            break;
        }
        (cover, wide_now, high_now) = halved(&cover, wide_now, high_now);
    }
    let rest = data.split_off(wide * high * 4);
    let mut image = Image::new(
        Extent3d {
            width: wide as u32,
            height: high as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        // Kept on this side too, as the game's other pictures made in code
        // are (`minimap`, `editor`), to be sent again should the phone ever
        // want it again.
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.mip_level_count = levels;
    if let Some(data) = image.data.as_mut() {
        data.extend(rest);
    }
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mipmap_filter: ImageFilterMode::Linear,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

/// `cover`, `wide` by `high`, at half the size each way (never less than a
/// pixel): each of its pixels the mean of the four, or the two, it stands
/// for. Returns it with its size.
fn halved(cover: &[u8], wide: usize, high: usize) -> (Vec<u8>, usize, usize) {
    let (half_wide, half_high) = ((wide / 2).max(1), (high / 2).max(1));
    let mut half = Vec::with_capacity(half_wide * half_high);
    for y in 0..half_high {
        for x in 0..half_wide {
            let (mut sum, mut count) = (0u32, 0u32);
            for (from_x, from_y) in [(2 * x, 2 * y), (2 * x + 1, 2 * y), (2 * x, 2 * y + 1), (2 * x + 1, 2 * y + 1)] {
                if from_x < wide && from_y < high {
                    sum += u32::from(cover[from_y * wide + from_x]);
                    count += 1;
                }
            }
            half.push((sum / count.max(1)) as u8);
        }
    }
    (half, half_wide, half_high)
}

/// Draws the words `says` for a sign's board: its lines (a `\n` between
/// them) one under another, each in the middle across, in a picture no
/// bigger than they are with [`PICTURE_PAD`] left clear round them, and as
/// big as leaves that picture inside [`WORDS_FILL`] of the board's room
/// both ways. In the game's
/// own fonts (`font::files`), each letter from the first of them that has
/// it, Hangul from the second, set one after another as far apart as its
/// font says. A letter no font has is left out, which what the game lets
/// be written never is; with nothing to draw, a single clear pixel.
fn draw_words(says: &str) -> Words {
    let nothing = || Words {
        cover: vec![0],
        wide: 1,
        high: 1,
    };
    let fonts: Vec<swash::FontRef> = crate::font::files()
        .iter()
        .filter_map(|file| swash::FontRef::from_index(file, 0))
        .collect();
    let Some(first) = fonts.first() else {
        return nothing();
    };
    // Each line's letters, as the font and the glyph of each and how far it
    // carries the pen, in ems.
    let lines: Vec<Vec<(&swash::FontRef, swash::GlyphId, f32)>> = says
        .lines()
        .map(|line| {
            line.chars()
                .filter_map(|letter| {
                    fonts.iter().find_map(|font| {
                        let glyph = font.charmap().map(letter);
                        (glyph != 0).then(|| (font, glyph, font.glyph_metrics(&[]).scale(1.0).advance_width(glyph)))
                    })
                })
                .collect()
        })
        .collect();
    let long = |line: &Vec<(&swash::FontRef, swash::GlyphId, f32)>| line.iter().map(|(.., carries)| carries).sum::<f32>();
    let longest = lines.iter().map(long).fold(0.0, f32::max);
    if longest <= 0.0 {
        return nothing();
    }
    // A line's height in ems, by the first font: from its top to its
    // bottom and the room it asks for between lines.
    let metrics = first.metrics(&[]).scale(1.0);
    let line_high = metrics.ascent + metrics.descent + metrics.leading;
    let all_high = line_high * lines.len() as f32;
    // An em on the board, in metres, and so in the picture's pixels: the
    // picture, with what is left clear round the words, fills the room.
    let clear = 2.0 * PICTURE_PAD;
    let em = (BOARD_WIDE * WORDS_FILL.0 / (longest + clear)).min(BOARD_HIGH * WORDS_FILL.1 / (all_high + clear));
    let size = em * PIXELS_A_METRE;
    let pad = (PICTURE_PAD * size).ceil();
    let wide = (longest * size + 2.0 * pad).ceil() as usize;
    let high = (all_high * size + 2.0 * pad).ceil() as usize;
    let mut cover = vec![0u8; wide * high];
    let mut context = ScaleContext::new();
    for (row, line) in lines.iter().enumerate() {
        let baseline = pad + (row as f32 * line_high + metrics.leading * 0.5 + metrics.ascent) * size;
        let mut pen = (wide as f32 - long(line) * size) * 0.5;
        for &(font, glyph, carries) in line {
            let mut scaler = context.builder(*font).size(size).hint(false).build();
            // Drawn where the pen is to a part of a pixel, so that letters
            // stand as far apart as they should.
            let drawn = Render::new(&[Source::Outline])
                .format(Format::Alpha)
                .offset(Vector::new(pen.fract(), baseline.fract()))
                .render(&mut scaler, glyph);
            if let Some(drawn) = drawn {
                let (left, up) = (pen.floor() as i32 + drawn.placement.left, baseline.floor() as i32 - drawn.placement.top);
                let across = (drawn.placement.width as usize).max(1);
                for (at, &alpha) in drawn.data.iter().enumerate() {
                    let (x, y) = (left + (at % across) as i32, up + (at / across) as i32);
                    if x >= 0 && y >= 0 && (x as usize) < wide && (y as usize) < high {
                        let pixel = &mut cover[y as usize * wide + x as usize];
                        *pixel = (*pixel).max(alpha);
                    }
                }
            }
            pen += carries * size;
        }
    }
    Words { cover, wide, high }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::island::glb;

    /// The town's map as the game reads it: the triangles of its plots, of
    /// its streets and of the rest of its land.
    fn town() -> (Vec<[Vec3; 3]>, Vec<[Vec3; 3]>, Vec<[Vec3; 3]>) {
        let (mut plot, mut streets, mut rest) = (Vec::new(), Vec::new(), Vec::new());
        for part in glb::read_venue(Venue::Town) {
            let named = |wanted: &str| part.material.as_deref().is_some_and(|name| is_material(name, wanted));
            if part.skinned || named(island::WATER_MATERIAL) {
                continue;
            }
            if named(PLOT_MATERIAL) {
                plot.extend(part.triangles);
            } else if STREET_MATERIALS.iter().any(|street| named(street)) {
                streets.extend(part.triangles);
            } else {
                rest.extend(part.triangles);
            }
        }
        (plot, streets, rest)
    }

    pub(crate) fn plots() -> Vec<Plot> {
        let (plot, streets, rest) = town();
        read_plots(&plot, &streets, &rest)
    }

    /// A square of two triangles on the ground, `half` to each side of
    /// `middle`.
    fn square(middle: Vec2, half: f32, up: f32) -> Vec<[Vec3; 3]> {
        let at = |x: f32, z: f32| Vec3::new(middle.x + x * half, up, middle.y + z * half);
        vec![[at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0)], [at(-1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)]]
    }

    #[test]
    fn the_town_has_forty_eight_plots_in_three_rings() {
        let plots = plots();
        assert_eq!(plots.len(), 48);
        let in_ring = |ring: usize| plots.iter().filter(|plot| plot.ring == ring).count();
        assert_eq!([in_ring(0), in_ring(1), in_ring(2)], [12, 16, 20]);
        for (index, plot) in plots.iter().enumerate() {
            // Each a square 24 m to a side, flat on the ground.
            for side in 0..4 {
                let long = plot.corners[side].distance(plot.corners[(side + 1) % 4]);
                assert!((long - 24.0).abs() < 0.05, "plot {index}: a side {long} m long");
            }
            assert!((plot.ground - 0.27).abs() < 0.02, "plot {index}: {} m up", plot.ground);
            // In order: ring by ring, and round each.
            if let Some(next) = plots.get(index + 1) {
                assert!(next.ring >= plot.ring);
            }
        }
        // The first is the first round from +Z by way of +X: at (25, 90).
        assert!(plots[0].middle.distance(Vec2::new(25.0, 90.0)) < 0.1, "{:?}", plots[0].middle);
    }

    #[test]
    fn a_plot_s_front_is_its_street_or_the_pond() {
        let plots = plots();
        let toward_pond = |plot: &Plot| plot.front.dot(-plot.middle.normalize());
        // Round the pond, all twelve look at it over the ring road, those
        // beside a street that runs out from the pond too.
        for plot in plots.iter().filter(|plot| plot.ring == 0) {
            assert!(plot.street && toward_pond(plot) > 0.95, "{:?}", plot.middle);
        }
        // In the middle ring, the eight beside a street that runs out from
        // the pond look at it, across it from each other; the eight between
        // have no street, and look at the pond.
        let middle: Vec<&Plot> = plots.iter().filter(|plot| plot.ring == 1).collect();
        let (street, pond): (Vec<&Plot>, Vec<&Plot>) = middle.iter().partition(|plot| plot.street);
        assert_eq!((street.len(), pond.len()), (8, 8));
        for plot in street {
            assert!(toward_pond(plot) < 0.3, "{:?}", plot.middle);
            // At the street, which runs along an axis.
            let axis = if plot.middle.x.abs() < plot.middle.y.abs() { Vec2::X } else { Vec2::Y };
            assert!((plot.front.dot(axis).abs() - 1.0).abs() < 1e-3, "{:?} looks {:?}", plot.middle, plot.front);
            assert!(plot.front.dot(plot.middle) < 0.0, "{:?} looks away from its street", plot.middle);
        }
        for plot in pond {
            assert!(toward_pond(plot) > 0.95, "{:?}", plot.middle);
        }
        // In the outer ring: eight at a street that runs out from the
        // pond, eight at the road round the town's edge, behind them from
        // the pond, and the four at the corners, with no street near, at
        // the pond.
        let outer: Vec<&Plot> = plots.iter().filter(|plot| plot.ring == 2).collect();
        assert_eq!(outer.iter().filter(|plot| plot.street && toward_pond(plot).abs() < 0.3).count(), 8);
        assert_eq!(outer.iter().filter(|plot| plot.street && toward_pond(plot) < -0.95).count(), 8);
        assert_eq!(outer.iter().filter(|plot| !plot.street && toward_pond(plot) > 0.95).count(), 4);
    }

    #[test]
    fn the_sign_stands_inside_the_right_hand_corner_seen_from_the_front() {
        // A plot north of the pond with no street: seen from the pond,
        // looking north (the game's -Z), right is east (+X), and its near
        // side is its south one.
        let lone = read_plots(&square(Vec2::new(0.0, -90.0), 12.0, 0.27), &[], &[]);
        assert_eq!(lone.len(), 1);
        assert!(lone[0].front.distance(Vec2::new(0.0, 1.0)) < 1e-4, "{:?}", lone[0].front);
        assert!(!lone[0].street);
        let stands = Vec2::new(12.0 - FROM_CORNER - SIGN_WIDE * 0.5, -78.0 - FROM_EDGE);
        assert!(lone[0].sign.distance(stands) < 1e-3, "{:?}", lone[0].sign);
        // With a street along its east side, 7.5 m of pavement between: it
        // looks east at the street, and seen from there, looking west,
        // right is north: the corner away from the pond.
        let street = square(Vec2::new(12.0 + 7.5 + 5.0, -90.0), 5.0, 0.0);
        let pavement = square(Vec2::new(0.0, -90.0), 40.0, -0.01);
        let by_street = read_plots(&square(Vec2::new(0.0, -90.0), 12.0, 0.27), &street, &pavement);
        assert!(by_street[0].street && by_street[0].front.distance(Vec2::new(1.0, 0.0)) < 1e-4, "{:?}", by_street[0].front);
        let stands = Vec2::new(12.0 - FROM_EDGE, -102.0 + FROM_CORNER + SIGN_WIDE * 0.5);
        assert!(by_street[0].sign.distance(stands) < 1e-3, "{:?}", by_street[0].sign);
        // A street under a pavement is no street.
        let paved = read_plots(&square(Vec2::new(0.0, -90.0), 12.0, 0.27), &street, &square(Vec2::new(0.0, -90.0), 40.0, 0.25));
        assert!(!paved[0].street && paved[0].front.distance(Vec2::new(0.0, 1.0)) < 1e-4);

        // On the town's own: every sign inside its plot, along its front,
        // and at the right hand end of it.
        for plot in plots() {
            let right = plot.right();
            let from_middle = plot.sign - plot.middle;
            assert!((from_middle.dot(plot.front) - (12.0 - FROM_EDGE)).abs() < 0.05, "{:?}", plot.middle);
            assert!((from_middle.dot(right) - (12.0 - FROM_CORNER - SIGN_WIDE * 0.5)).abs() < 0.05, "{:?}", plot.middle);
        }
    }

    #[test]
    fn a_seat_of_the_town_is_a_plot_by_hajun_s_number_for_it() {
        let plots = plots();
        let of_seat = in_order(&plots);
        // Every plot of the map is one of them, each once.
        assert_eq!(of_seat.len(), roundtown_net::land::PLOTS);
        let mut distinct = of_seat.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), plots.len());
        // His first twelve are the twelve round the pond, his next sixteen
        // the ring behind them, and the rest the outer one.
        assert!(of_seat[..12].iter().all(|&plot| plots[plot].ring == 0));
        assert!(of_seat[12..28].iter().all(|&plot| plots[plot].ring == 1));
        assert!(of_seat[28..].iter().all(|&plot| plots[plot].ring == 2));
        // His plot 1: west of the pond as the game has it, just south of
        // the street that runs out from it there, looking at the pond.
        let first = &plots[of_seat[0]];
        assert!(first.middle.distance(Vec2::new(-90.0, 25.0)) < 0.1, "{:?}", first.middle);
        assert!(first.front.distance(Vec2::new(1.0, 0.0)) < 1e-3, "{:?}", first.front);
        // Whoever is in a channel has one of the first sixteen: the twelve
        // round the pond and the four behind his 1 to 4.
        assert!(of_seat[12..roundtown_net::land::FRONT].iter().all(|&plot| plots[plot].middle.x.abs().min(plots[plot].middle.y.abs()) < 26.0));
        // A map with a plot somewhere else than it is written: as far as
        // the first that is not found.
        assert_eq!(in_order(&plots[..0]).len(), 0);
        let mut moved = plots.clone();
        let third = of_seat[2];
        moved[third].middle += Vec2::splat(3.0);
        assert_eq!(in_order(&moved).len(), 2);
    }

    #[test]
    fn the_sign_s_model_is_the_size_the_game_takes_it_for() {
        let parts = glb::read(MODEL);
        let reach = |material: &str, of: fn(Vec3) -> f32| {
            let all: Vec<f32> = parts
                .iter()
                .filter(|part| part.material.as_deref() == Some(material))
                .flat_map(|part| part.triangles.iter().flatten().copied())
                .map(of)
                .collect();
            assert!(!all.is_empty(), "{MODEL} has nothing of {material}");
            (all.iter().copied().fold(f32::INFINITY, f32::min), all.iter().copied().fold(f32::NEG_INFINITY, f32::max))
        };
        // Post to post, about its middle, standing on the ground.
        let (left, right) = reach("wood", |corner| corner.x);
        assert!((right - left - SIGN_WIDE).abs() < 0.05 && (right + left).abs() < 0.05, "{left} to {right}");
        let (foot, top) = reach("wood", |corner| corner.y);
        assert!(foot.abs() < 0.02 && (top - SIGN_HIGH).abs() < 0.05, "{foot} to {top}");
        // As deep as its posts, which the box that stands for it is no
        // thinner than, nor thinner than a body is tested against.
        let (back, fore) = reach("wood", |corner| corner.z);
        assert!(fore - back <= SIGN_DEEP && SIGN_DEEP > island::LATTICE, "{back} to {fore}");
        // The board's frame: the room for words is inside it, about its
        // middle, and in front of it.
        let (left, right) = reach("border", |corner| corner.x);
        let (low, high) = reach("border", |corner| corner.y);
        let (back, fore) = reach("border", |corner| corner.z);
        assert!(((low + high) * 0.5 - BOARD_UP).abs() < 0.03, "{low} to {high}");
        assert!(BOARD_WIDE < right - left - 0.15 && BOARD_HIGH < high - low - 0.15);
        assert!((right + left).abs() < 0.03);
        // The words hang clear of each face of the board itself, the wood
        // inside the frame, and inside the frame's own depth: its front is
        // the sign's -Z.
        let panel: Vec<f32> = parts
            .iter()
            .filter(|part| part.material.as_deref() == Some("wood"))
            .flat_map(|part| part.triangles.iter().flatten().copied())
            .filter(|corner| corner.x.abs() < 0.74 && corner.y > low && corner.y < high)
            .map(|corner| corner.z)
            .collect();
        let (panel_front, panel_back) = (panel.iter().copied().fold(f32::INFINITY, f32::min), panel.iter().copied().fold(f32::NEG_INFINITY, f32::max));
        assert!(back < -FRONT_OUT && -FRONT_OUT < panel_front - 0.002, "the frame from {back}, the board from {panel_front}");
        assert!(panel_back + 0.002 < BACK_OUT && BACK_OUT < fore, "the board to {panel_back}, the frame to {fore}");
        // And it goes into the APK.
        let list = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("mobile/android/app/assets-left-out.txt");
        let list = std::fs::read_to_string(list).expect("the APK's list of what it leaves out");
        assert!(!list.lines().any(|line| line.trim() == MODEL), "{MODEL} is left out of the APK");
    }

    /// The box round what is drawn in `words`: its left, top, right and
    /// bottom pixels.
    fn ink(words: &Words) -> (usize, usize, usize, usize) {
        let drawn: Vec<(usize, usize)> = words
            .cover
            .iter()
            .enumerate()
            .filter(|&(_, &alpha)| alpha > 40)
            .map(|(at, _)| (at % words.wide, at / words.wide))
            .collect();
        assert!(!drawn.is_empty(), "nothing is drawn");
        (
            drawn.iter().map(|at| at.0).min().unwrap(),
            drawn.iter().map(|at| at.1).min().unwrap(),
            drawn.iter().map(|at| at.0).max().unwrap(),
            drawn.iter().map(|at| at.1).max().unwrap(),
        )
    }

    #[test]
    fn the_words_are_drawn_as_big_as_the_board_has_room_for() {
        let room = Vec2::new(BOARD_WIDE, BOARD_HIGH);
        // Two lines of a short name: as tall as the room leaves them, in a
        // picture no bigger than they are, inside the room.
        let words = draw_words("hajun's\nland");
        assert_eq!(words.cover.len(), words.wide * words.high);
        let on_board = words.metres();
        assert!(on_board.cmple(room).all(), "{on_board} on a board with room for {room}");
        assert!(on_board.y > BOARD_HIGH * WORDS_FILL.1 - 0.01 && on_board.x < BOARD_WIDE * 0.75, "{on_board}");
        let (left, top, right, bottom) = ink(&words);
        assert!(((left + right) as f32 * 0.5 - words.wide as f32 * 0.5).abs() < 6.0, "{left} to {right} of {}", words.wide);
        assert!(left > 2 && top > 2 && right + 3 < words.wide && bottom + 3 < words.high, "cut at an edge: {left} {top} {right} {bottom}");
        assert!((right - left) as f32 > words.wide as f32 * 0.75 && (bottom - top) as f32 > words.high as f32 * 0.7);
        // Both lines are there: ink in the upper half and in the lower.
        let half = words.wide * (words.high / 2);
        assert!(words.cover[..half].iter().any(|&alpha| alpha > 200) && words.cover[half..].iter().any(|&alpha| alpha > 200));
        // The longest name there can be reaches the room's width and no
        // further: smaller letters.
        let long = draw_words("WWWWWWWWWWWWWWWW's\nland");
        let on_board = long.metres();
        assert!(on_board.cmple(room).all() && on_board.x > BOARD_WIDE * WORDS_FILL.0 - 0.01 && on_board.y < words.metres().y * 0.6, "{on_board}");
        let (left, top, right, bottom) = ink(&long);
        assert!(left > 2 && top > 2 && right + 3 < long.wide && bottom + 3 < long.high, "cut at an edge: {left} {top} {right} {bottom}");
        // Hangul is drawn, from the second font, and Vietnamese from the
        // first, as wide as their letters; and so on a line of their own.
        for name in ["하준", "Dũng", "하준's\nland"] {
            let drawn = draw_words(name);
            let (left, _, right, _) = ink(&drawn);
            assert!(right - left > drawn.wide / 2 && drawn.metres().cmple(room).all(), "{name}: {left} to {right} of {}", drawn.wide);
        }
        // Nothing to say, or nothing the fonts have, is a clear pixel.
        for nothing in ["", "\n", "\u{1F600}"] {
            let drawn = draw_words(nothing);
            assert_eq!((drawn.wide, drawn.high, drawn.cover.as_slice()), (1, 1, [0u8].as_slice()), "{nothing:?}");
        }
    }

    #[test]
    fn the_picture_has_every_smaller_copy_of_itself() {
        let words = draw_words("hajun's\nland");
        let image = words_picture(&words);
        // Each half of the one before, down to a pixel.
        let mut pixels = 0;
        let (mut wide, mut high, mut levels) = (words.wide, words.high, 0);
        loop {
            pixels += wide * high;
            levels += 1;
            if wide == 1 && high == 1 {
                break;
            }
            (wide, high) = ((wide / 2).max(1), (high / 2).max(1));
        }
        assert_eq!(levels, usize::BITS - words.wide.max(words.high).leading_zeros());
        assert_eq!(image.texture_descriptor.mip_level_count, levels);
        assert_eq!(image.data.as_ref().map(Vec::len), Some(pixels * 4));
        assert_eq!((image.texture_descriptor.size.width, image.texture_descriptor.size.height), (words.wide as u32, words.high as u32));
        // White everywhere, the letters in how see-through it is.
        let data = image.data.as_ref().unwrap();
        let first = words.wide * words.high * 4;
        assert!(data.chunks_exact(4).all(|pixel| pixel[..3] == [255, 255, 255]));
        assert!(data[..first].chunks_exact(4).any(|pixel| pixel[3] == 255));
        assert!(data[..first].chunks_exact(4).any(|pixel| pixel[3] == 0));
        // A half is the mean of what it stands for.
        assert_eq!(halved(&[0, 255, 255, 0, 10, 20, 30, 40], 4, 2), (vec![71, 81], 2, 1));
        assert_eq!(halved(&[100, 200, 50], 3, 1), (vec![150], 1, 1));
        // Nothing drawn is a picture of a pixel, which can still be hung.
        let nothing = words_picture(&draw_words(""));
        assert_eq!((nothing.texture_descriptor.mip_level_count, nothing.data.as_ref().map(Vec::len)), (1, Some(4)));
    }

    #[test]
    fn a_sign_for_every_plot_and_for_yours() {
        use roundtown_net::land::Holder;
        let plots = [10, 11, 12, 13];
        let deed = |plot: u8, who: u64, name: &str| Deed {
            plot,
            who,
            name: name.into(),
            look: "boy".into(),
            holder: Holder::Here,
        };
        // With nothing known of anyone else: you alone, on the plot you
        // have.
        assert_eq!(holders(&plots, (0, "Hajun"), &[]), [(10, "Hajun".to_string())]);
        assert_eq!(holders(&plots, (2, "Hajun"), &[]), [(12, "Hajun".to_string())]);
        // Everyone a deed names, by the name on it, and you by your own,
        // in the order of the plots.
        let deeds = [deed(3, 8, "Dubu"), deed(1, 7, "Mina")];
        assert_eq!(
            holders(&plots, (0, "Hajun"), &deeds),
            [(10, "Hajun".to_string()), (11, "Mina".to_string()), (13, "Dubu".to_string())]
        );
        // A plot past those the map has has no sign; and the plot this
        // device has you on is yours here, whoever else is said to have it.
        let deeds = [deed(9, 9, "Bora"), deed(1, 7, "Mina"), deed(2, 8, "Dubu")];
        assert_eq!(holders(&plots, (1, "Hajun"), &deeds), [(11, "Hajun".to_string()), (12, "Dubu".to_string())]);
        // Whose a plot is, by who they are.
        let deeds = Deeds(deeds.to_vec());
        assert_eq!((deeds.plot_of(8), deeds.plot_of(5)), (Some(2), None));
        assert!(deeds.has(7) && !deeds.has(5));
    }

    #[test]
    fn a_sign_has_its_words_while_the_camera_is_near_it() {
        // They come as it comes near, and stay until it is well away, so
        // that a step either way at the edge changes nothing.
        assert!(words_wanted(10.0, false) && words_wanted(WORDS_NEAR, false));
        assert!(!words_wanted(WORDS_NEAR + 1.0, false));
        assert!(words_wanted(WORDS_NEAR + 1.0, true) && words_wanted(WORDS_FAR, true));
        assert!(!words_wanted(WORDS_FAR + 1.0, true));
        // The smallest words there are, the longest name's, are still a
        // few dots tall on a phone where they come: a phone's screen is
        // about 400 dots tall, and the view 60 degrees up and down.
        let dots = |metres: f32, far: f32| metres / (2.0 * far * (30.0f32).to_radians().tan()) * 400.0;
        let short = draw_words("hajun's\nland");
        let line = short.metres().y / 2.0;
        assert!(dots(line, WORDS_NEAR) < 3.5 && dots(line, WORDS_NEAR) > 1.0, "{}", dots(line, WORDS_NEAR));
    }

    #[test]
    fn a_sign_still_wanted_stays_as_it_is() {
        let sign = |plot: usize, name: &str| (plot, format!("{name}'s\nland"));
        let standing = [sign(10, "Hajun"), sign(11, "Mina"), sign(13, "Dubu")];
        // Nothing changed: all stay.
        assert_eq!(staying(&standing, &standing), [true, true, true]);
        // Someone left: theirs alone goes. Someone came: nothing that stood
        // goes.
        assert_eq!(staying(&standing, &[sign(10, "Hajun"), sign(13, "Dubu")]), [true, false, true]);
        let more = [sign(10, "Hajun"), sign(11, "Mina"), sign(12, "Bora"), sign(13, "Dubu")];
        assert_eq!(staying(&standing, &more), [true, true, true]);
        // Named anew, or another's plot now: that one goes, to be stood
        // again with its new words.
        assert_eq!(staying(&standing, &[sign(10, "Hajun"), sign(11, "Minji"), sign(13, "Dubu")]), [true, false, true]);
        // Two the same, which there should never be: one of them stays.
        assert_eq!(staying(&[sign(10, "Hajun"), sign(10, "Hajun")], &[sign(10, "Hajun")]), [true, false]);
        assert!(staying(&[], &standing).is_empty());
    }

    #[test]
    fn the_sign_is_solid_where_it_stands_and_nowhere_else() {
        use crate::island::Island;
        for plot in plots().iter().filter(|plot| plot.ring == 0) {
            let faces = solid(plot);
            // A closed box: twelve triangles, each looking away from its
            // middle, its top up and its bottom down.
            assert_eq!(faces.len(), 12);
            let middle = Vec3::new(plot.sign.x, plot.ground + SIGN_HIGH * 0.5 - 0.05, plot.sign.y);
            for [a, b, c] in &faces {
                let normal = (*b - *a).cross(*c - *a);
                assert!(normal.length() > 1e-3 && normal.dot((*a + *b + *c) / 3.0 - middle) > 0.0, "{:?}", plot.middle);
            }
            let (ups, downs) = faces.iter().fold((0, 0), |(ups, downs), [a, b, c]| {
                let up = (*b - *a).cross(*c - *a).normalize().y;
                (ups + usize::from(up > 0.99), downs + usize::from(up < -0.99))
            });
            assert_eq!((ups, downs), (2, 2));
            // Laid over an island, it fills where the sign stands from the
            // ground to over a head, from one post to the other, and
            // nothing a stride in front of it, behind it or past its ends.
            let island = Island::of_parts_for_tests(&[&faces]);
            let (feet, head) = (plot.ground + crate::LAND_STEP_UP, plot.ground + crate::PLAYER_HEIGHT);
            let right = plot.right();
            for along in [-1.4, -0.7, 0.0, 0.7, 1.4] {
                assert!(island.blocked(plot.sign + right * along, feet, head), "{:?} at {along}", plot.middle);
            }
            for clear in [plot.front * 1.0, plot.front * -1.0, right * 2.2, right * -2.2] {
                assert!(!island.blocked(plot.sign + clear, feet, head), "{:?} at {clear}", plot.middle);
            }
            // A body is kept out of it from either face: however it is
            // turned, no square of the lattice a body is tested on is left
            // open along it.
            let body = [(crate::PLAYER_RADIUS, feet, head)];
            for along in (-14..=14).map(|step| step as f32 * 0.1) {
                assert!(
                    !island.walls_round(plot.sign + right * along, &body).is_empty(),
                    "{:?}: a way through {along} m along its sign",
                    plot.middle
                );
            }
            // And where you arrive, in the middle of the plot, is clear of it.
            assert!(island.walls_round(plot.middle, &body).is_empty());
        }
        // Laid over the town's shape, and taken off it again.
        let plots = plots();
        let mut islands = Islands::default();
        islands.set_for_tests(Venue::Town);
        let at = plots[0].sign;
        assert!(!islands.get(Venue::Town).unwrap().blocked(at, 0.8, 1.8));
        islands.lay_over(Venue::Town, Some(vec![solid(&plots[0])]));
        assert!(islands.get(Venue::Town).unwrap().blocked(at, 0.8, 1.8));
        islands.lay_over(Venue::Town, None);
        assert!(!islands.get(Venue::Town).unwrap().blocked(at, 0.8, 1.8));
    }

    #[test]
    fn a_person_arrives_in_the_middle_of_the_plot_their_seat_is() {
        // What the server goes by, having no map (`roundtown_net::land`),
        // is what the map says.
        let plots = plots();
        let of_seat = in_order(&plots);
        assert_eq!(of_seat.len(), roundtown_net::land::HOMES.len());
        for (seat, &plot) in of_seat.iter().enumerate() {
            let plot = &plots[plot];
            let home = Vec2::from(roundtown_net::land::HOMES[seat]);
            assert!(home.distance(plot.middle) < 0.02, "seat {seat}: {home} is written, {} is its plot's middle", plot.middle);
            // You stand there, on its ground, facing the fountain, which is
            // the middle of the town.
            let arrives = island::arrival_of_a_person(Venue::Town, seat);
            assert!(arrives.translation.xz().distance(plot.middle) < 0.02);
            assert!((plot.ground - arrives.translation.y).abs() < crate::LAND_STEP_UP, "seat {seat}");
            let ahead = (arrives.rotation * Vec3::NEG_Z).xz();
            assert!(ahead.distance(-plot.middle.normalize()) < 1e-3, "seat {seat} faces {ahead}");
            // Clear of its own sign, and on no plot but its own.
            assert!(plot.sign.distance(plot.middle) > 5.0);
            assert!(roundtown_net::land::just_arrived(arrives.translation.xz()));
            assert_eq!(roundtown_net::land::plot_at(arrives.translation.xz()), Some(seat));
            // Anywhere on the plot is on it, as far as the server can
            // tell, and nowhere on it is on another.
            for corner in plot.corners {
                assert_eq!(roundtown_net::land::plot_at(corner.lerp(plot.middle, 0.01)), Some(seat));
            }
        }
        // The path round the pond is on nobody's plot, and so is a person
        // anywhere but the town where they always were.
        assert!(island::arrival(Venue::Town, 0).translation.xz().length() < 40.0);
        assert!(!roundtown_net::land::just_arrived(island::arrival(Venue::Town, 0).translation.xz()));
        assert_eq!(roundtown_net::land::plot_at(island::arrival(Venue::Town, 0).translation.xz()), None);
        assert_eq!(island::arrival_of_a_person(Venue::Arena, 3), island::arrival(Venue::Arena, 3));
        assert_eq!(island::arrival_of_a_person(Venue::Town, 60), island::arrival(Venue::Town, 60));
    }

    #[test]
    fn right_is_east_to_someone_looking_north() {
        assert_eq!(right_of(Vec2::new(0.0, -1.0)), Vec2::new(1.0, 0.0));
        assert_eq!(right_of(Vec2::new(1.0, 0.0)), Vec2::new(0.0, 1.0));
    }
}
