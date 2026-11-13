//! The punch's hit range, drawn round you as Brawl Stars draws an attack's
//! (since 2026-11-07, Hajun: "right now there is no hit range indicator for
//! the melee. i want to make it similar to brawlstars - a default of 1/8
//! circular sector hit indicator with white outline and less white area
//! that is both semi transparent, projected only for the client's view -
//! which revolves around the player depending on player facing
//! direction"): a flat band lying on the ground in front of you, turned
//! the way you face, from the file's `melee.near` out to its `melee.reach`
//! (`assets/ui.ron`), `melee.width` across where it begins and opening out
//! by `melee.cone` from there: the sector of a circle whose sides meet
//! behind you, where an arc that wide at that angle has its middle
//! ([`Sector::of`]); with a `width` of 0, one coming to a point `near` in
//! front of you. It is the very range a punch lands in (`punch_lands` in
//! `lib.rs`, through [`Sector::holds`]), with a white outline just inside
//! its edge and a fainter white fill, both see-through, in the file's
//! colours (`colours.melee`), taken again as the file changes on desktop,
//! the shape too ([`restyle`]). For its first hour it was a wedge from a
//! point at your middle (Hajun: "its not a pointy corner near the player
//! so the hit region need to be in front of the player not connected to
//! it. the round edge near the player need to be wider"), then a ring
//! sector about your middle, until "make it much more wider near the
//! player". Yours alone, offline and online: nothing of it goes over the
//! network, and nobody else's is drawn, nor an AI's. Put away in the car,
//! where the fist is the reverse and there is no punch.
//!
//! It lies as high over your feet as a step ([`HEIGHT`]), so that under the
//! camera fixed over you the paving beside a road, a kerb or the strip of
//! grass never hides it; anything higher, a planter or the pond's rim,
//! draws over it, as the ground would. It is unlit and drawn brighter than
//! its colours ([`GAIN`]), as the clouds and the fountain's water are,
//! since Bevy's tonemapping shows plain white as a light grey. Its two
//! sheets are blended, which needs `NoIndirectDrawing` on the camera on the
//! Adreno, as the sea does (`RENDERING.md`); and being blended,
//! `see_through` never takes them for something in the way. Not yet seen
//! on a phone.

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::island::Venue;
use crate::ui::{self, UiChanged};
use crate::{Dead, LAND_STEP_UP, Player, drive};

/// How high over your feet it lies, in metres: as high as a step, so that
/// nothing you could step up onto hides any of it.
pub(crate) const HEIGHT: f32 = LAND_STEP_UP;

/// How wide the outline is, in metres, just inside the sector's edge:
/// about three pixels on a phone with the camera at its usual 24 m over
/// you, one at its furthest and ten at its nearest.
const WIDTH: f32 = 0.1;

/// The outline lies this much over the fill, so that it is drawn after
/// it, over it.
const OVER: f32 = 0.002;

/// How wide the dark rim round a band that is aimed is ([`rim`]), in
/// metres, just outside its edge: about three pixels on a phone under
/// Bridge Wars' camera.
const RIM: f32 = 0.07;

/// How much brighter than its colours it is drawn, as the fountain's
/// water is (`fountain::WATER_GAIN`; the clouds at 5): white at full alpha
/// comes out white rather than tonemapped to grey, and a fainter alpha
/// lightens the ground under it by about that share.
const GAIN: f32 = 2.5;

/// How far round an arc one triangle goes, in degrees.
const STEP_DEG: f32 = 3.0;

/// A cone this wide or wider, in degrees, is all round: a ring, or a
/// disc, no sides.
const ALL_ROUND: f32 = 359.9;

/// A near arc this close to where the sides meet, in metres, is none: the
/// sector comes to a point there.
const NO_EDGE: f32 = 1e-3;

/// The range a punch lands in, as the file's `melee` has it ([`Sector::of`]),
/// in the ground's metres: the sector of a circle about a point on the
/// line you face, `behind` your middle (in front of it, if less than
/// nothing), between the arcs `inner` and `outer` out from that point,
/// `half_cone` either side of straight ahead, in radians.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Sector {
    pub(crate) behind: f32,
    pub(crate) inner: f32,
    pub(crate) outer: f32,
    pub(crate) half_cone: f32,
}

impl Sector {
    /// The file's range: from `near` in front of your middle out to
    /// `reach`, `width` across where it begins and opening out by `cone`
    /// from there, so its sides meet where an arc that wide at that angle
    /// has its middle; with no width, at a point `near` in front of you.
    pub(crate) fn of(melee: &ui::Melee) -> Sector {
        let half_cone = melee.cone.to_radians() * 0.5;
        let sine = half_cone.sin();
        let inner = if melee.width > 0.0 && sine > 1e-4 { melee.width / (2.0 * sine) } else { 0.0 };
        Sector {
            behind: inner - melee.near,
            inner,
            outer: inner + (melee.reach - melee.near),
            half_cone,
        }
    }

    /// Whether a body `radius` across, its middle `off` from yours along
    /// the ground with `facing` straight ahead, has any of itself in the
    /// range: its middle within the cone from where the sides meet, and no
    /// further than its own radius past the far arc or short of the near.
    pub(crate) fn holds(&self, facing: Vec2, off: Vec2, radius: f32) -> bool {
        let from_apex = off + facing * self.behind;
        let far = from_apex.length();
        far >= 1e-3
            && far - radius <= self.outer
            && far + radius >= self.inner
            && facing.angle_to(from_apex / far).abs() <= self.half_cone
    }

    /// Where its sides meet, in the ground's (x, z) about your middle with
    /// straight ahead -Z.
    fn apex(&self) -> Vec2 {
        Vec2::new(0.0, self.behind)
    }

    /// Whether it goes all the way round: no sides.
    fn all_round(&self) -> bool {
        self.half_cone >= ALL_ROUND.to_radians() * 0.5
    }
}

/// The range drawn round you: what it was built for, and what it is built
/// of, to build again as the file changes.
#[derive(Component)]
struct HitRange {
    shape: Sector,
    fill: Handle<Mesh>,
    outline: Handle<Mesh>,
    fill_look: Handle<StandardMaterial>,
    outline_look: Handle<StandardMaterial>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn)
        // Once the body has been moved this frame, in or out of the car.
        .add_systems(Update, (follow.after(drive::place), restyle));
}

/// Makes it, hidden until it is put at your feet ([`follow`]): the fill
/// and, a hair over it, the outline, under one root with no `Venue`, so
/// that it is drawn wherever you are and is in nothing's way.
fn spawn(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let file = ui::ui();
    let shape = Sector::of(&file.melee);
    let fill = meshes.add(sector(shape));
    let outline = meshes.add(outline(shape));
    let fill_look = materials.add(look(file.colours.melee.fill));
    let outline_look = materials.add(look(file.colours.melee.outline));
    commands.spawn((
        Name::new("Hit range"),
        HitRange {
            shape,
            fill: fill.clone(),
            outline: outline.clone(),
            fill_look: fill_look.clone(),
            outline_look: outline_look.clone(),
        },
        Transform::default(),
        Visibility::Hidden,
        children![
            (
                Name::new("Hit range fill"),
                Mesh3d(fill),
                MeshMaterial3d(fill_look),
                Transform::default(),
                NotShadowCaster,
                NotShadowReceiver,
            ),
            (
                Name::new("Hit range outline"),
                Mesh3d(outline),
                MeshMaterial3d(outline_look),
                Transform::from_xyz(0.0, OVER, 0.0),
                NotShadowCaster,
                NotShadowReceiver,
            ),
        ],
    ));
}

/// Puts it at your feet, [`HEIGHT`] up, turned the way you face along the
/// ground, in a game of Bridge Wars (`arena`, since the night of 2026-11-07; everywhere
/// for 2026-11-07, when the town was its map), and puts it away anywhere
/// else, in the car, while you are dead, or with no body to stand round.
fn follow(
    driving: Res<drive::Driving>,
    bridging: Res<crate::bridge::Bridging>,
    me: Query<(&Transform, &Venue, Has<Dead>, Has<crate::Drinking>), With<Player>>,
    mut ranges: Query<(&mut Transform, &mut Visibility), (With<HitRange>, Without<Player>)>,
) {
    let Ok((mut place, mut shown)) = ranges.single_mut() else {
        return;
    };
    // Nor while you carry a plank, there being no punch to show the range
    // of (`bridge`; Hajun, 2026-11-09: "remove the hit indicator"); nor
    // with a potion at your mouth, for the same reason (`utility`).
    let body = me
        .single()
        .ok()
        .filter(|&(_, &here, dead, drinking)| here == Venue::Arena && !driving.on() && !dead && !bridging.0 && !drinking)
        .map(|(body, ..)| body);
    shown.set_if_neq(if body.is_some() { Visibility::Inherited } else { Visibility::Hidden });
    let Some(body) = body else {
        return;
    };
    let facing = body.forward().xz().normalize_or(Vec2::NEG_Y);
    place.set_if_neq(Transform {
        translation: body.translation + Vec3::Y * HEIGHT,
        rotation: turned_to(facing),
        scale: Vec3::ONE,
    });
}

/// The turn about the ground's up that faces straight ahead (-Z) along
/// `facing`, in the ground's (x, z).
pub(crate) fn turned_to(facing: Vec2) -> Quat {
    Quat::from_rotation_y((-facing.x).atan2(-facing.y))
}

/// Takes the file's colours again, and its shape, as the file changes on
/// desktop (`ui::reread`): the sheets are built again at a new shape, and
/// recoloured either way.
fn restyle(
    changed: Res<UiChanged>,
    mut ranges: Query<&mut HitRange>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let file = ui::ui();
    let shape = Sector::of(&file.melee);
    for mut range in &mut ranges {
        if range.shape != shape {
            for (handle, mesh) in [(&range.fill, sector(shape)), (&range.outline, outline(shape))] {
                if meshes.insert(handle, mesh).is_err() {
                    warn!("the hit range has lost a mesh");
                }
            }
            range.shape = shape;
        }
        for (handle, colour) in [
            (&range.fill_look, file.colours.melee.fill),
            (&range.outline_look, file.colours.melee.outline),
        ] {
            if let Some(mut look) = materials.get_mut(handle) {
                look.base_color = bright(colour);
            }
        }
    }
}

/// A band lying on the ground in front of you, `width` across, from
/// `from` to `to` ahead of your middle (-Z), in metres: its fill, and its
/// outline, [`WIDTH`] wide just inside its edge, as the punch's range has
/// them. What is aimed in Bridge Wars shows where it will go with one
/// (`utility`).
pub(crate) fn band(width: f32, from: f32, to: f32) -> (Mesh, Mesh) {
    let corners = band_corners(width, from, to);
    let mut indices = Vec::new();
    triangle(&corners, &mut indices, 0, 1, 2);
    triangle(&corners, &mut indices, 0, 2, 3);
    let fill = flat(corners.clone(), indices);
    let mut points = corners.clone();
    points.extend(inset(&corners, WIDTH));
    let mut indices = Vec::new();
    for i in 0..4 {
        let j = (i + 1) % 4;
        triangle(&points, &mut indices, i, j, 4 + j);
        triangle(&points, &mut indices, i, 4 + j, 4 + i);
    }
    (fill, flat(points, indices))
}

/// The four corners of a band `width` across from `from` to `to` ahead of
/// your middle (-Z), in the ground's (x, z).
fn band_corners(width: f32, from: f32, to: f32) -> Vec<Vec2> {
    let half = width * 0.5;
    vec![
        Vec2::new(-half, -from),
        Vec2::new(-half, -to),
        Vec2::new(half, -to),
        Vec2::new(half, -from),
    ]
}

/// The rim round the same band ([`band`]): a line [`RIM`] wide just
/// outside its edge, all the way round, to be drawn dark. A white band is
/// all but the colour of the sky by day, and what is aimed in Bridge Wars
/// is aimed over the void as often as over land (`utility`; Hajun,
/// 2026-11-12: "the hit indicator should appear even on void area"): with
/// a dark line round it, it is seen over anything.
pub(crate) fn rim(width: f32, from: f32, to: f32) -> Mesh {
    let corners = band_corners(width, from, to);
    let mut points = corners.clone();
    points.extend(inset(&corners, -RIM));
    let mut indices = Vec::new();
    for i in 0..4 {
        let j = (i + 1) % 4;
        triangle(&points, &mut indices, i, j, 4 + j);
        triangle(&points, &mut indices, i, 4 + j, 4 + i);
    }
    flat(points, indices)
}

/// A band `width` across along `line`, points in order, each as high over
/// the ground as its y says: its fill, its outline, [`WIDTH`] wide just
/// inside its edge all the way round, and the dark rim just outside it
/// ([`rim`]). What a stick of dynamite is aimed with in Bridge Wars
/// (`utility`): a band that rises and comes down as the stick will, an
/// arch standing over the ground, where a fireball's lies straight along
/// it. It is level from side to side all the way: across it is across the
/// ground. A line of fewer than two points is nothing: meshes with no
/// corner in them, which are not to be drawn (Bevy's mesh allocator logs
/// a use after free for one; `utility::aim` puts the band's sheets away
/// instead).
pub(crate) fn arch(line: &[Vec3], width: f32) -> (Mesh, Mesh, Mesh) {
    let half = width * 0.5;
    let n = line.len();
    if n < 2 {
        let nothing = || flat(Vec::new(), Vec::new());
        return (nothing(), nothing(), nothing());
    }
    let ground: Vec<Vec2> = line.iter().map(|point| point.xz()).collect();
    // Which way across the band is to the left of the way it runs, at
    // each point: square to the line through the points either side, or,
    // where those are over one another, as it was the point before.
    let mut across: Vec<Vec2> = Vec::with_capacity(n);
    for i in 0..n {
        let along = ground[(i + 1).min(n - 1)] - ground[i.saturating_sub(1)];
        let before = across.last().copied().unwrap_or(Vec2::X);
        across.push(if along.length() > 1e-4 { along.normalize().perp() } else { before });
    }
    let left: Vec<Vec2> = (0..n).map(|i| ground[i] + across[i] * half).collect();
    let right: Vec<Vec2> = (0..n).map(|i| ground[i] - across[i] * half).collect();
    let high: Vec<f32> = line.iter().map(|point| point.y).collect();
    // The fill: a strip between its two edges.
    let mut points = left.clone();
    points.extend(right.iter().copied());
    let mut heights = high.clone();
    heights.extend(high.iter().copied());
    let mut indices = Vec::new();
    for i in 0..n - 1 {
        triangle(&points, &mut indices, i, i + 1, n + i + 1);
        triangle(&points, &mut indices, i, n + i + 1, n + i);
    }
    let fill = raised(points, &heights, indices);
    // Its edge, all the way round: up the one side and back down the
    // other, each corner as high as the point of the line it is beside.
    let mut edge = left;
    edge.extend(right.into_iter().rev());
    let mut round = high.clone();
    round.extend(high.iter().rev().copied());
    (fill, edging_at(&edge, &round, WIDTH), edging_at(&edge, &round, -RIM))
}

/// A cross lying on the ground about the origin, its two strokes
/// `reach` long each way from its middle and `thick` either side of their
/// own middles, lying aslant as an X does: its fill, and the dark rim just
/// outside its edge ([`rim`]). Where a stick of dynamite will come down,
/// in the middle of the ring of its blast (`utility`; Hajun, 2026-11-13:
/// "some indicator of where the dynamite will land, maybe a x in the
/// middle of the circle").
pub(crate) fn cross(reach: f32, thick: f32) -> (Mesh, Mesh) {
    let (r, t) = (reach, thick);
    // A plus sign, round by its twelve corners, turned an eighth of a turn.
    let plus = [
        Vec2::new(t, r),
        Vec2::new(t, t),
        Vec2::new(r, t),
        Vec2::new(r, -t),
        Vec2::new(t, -t),
        Vec2::new(t, -r),
        Vec2::new(-t, -r),
        Vec2::new(-t, -t),
        Vec2::new(-r, -t),
        Vec2::new(-r, t),
        Vec2::new(-t, t),
        Vec2::new(-t, r),
    ];
    let turn = Vec2::from_angle(std::f32::consts::FRAC_PI_4);
    let edge: Vec<Vec2> = plus.iter().map(|corner| turn.rotate(*corner)).collect();
    // Every corner is seen from its middle: a fan from there fills it.
    let mut points = vec![Vec2::ZERO];
    points.extend(edge.iter().copied());
    let mut indices = Vec::new();
    for i in 0..edge.len() {
        triangle(&points, &mut indices, 0, 1 + i, 1 + (i + 1) % edge.len());
    }
    (flat(points, indices), edging(&edge, -RIM))
}

/// A disc lying on the ground about the origin, `radius` across from its
/// middle: its fill, its outline just inside its edge and the dark rim
/// just outside it, as [`arch`] has them. How far a stick of dynamite's
/// blast reaches, where it is aimed and where it lies (`utility`,
/// `dynamite`).
pub(crate) fn disc(radius: f32) -> (Mesh, Mesh, Mesh) {
    let steps = (360.0 / STEP_DEG) as usize;
    let edge: Vec<Vec2> = (0..steps)
        .map(|i| Vec2::from_angle(i as f32 * std::f32::consts::TAU / steps as f32) * radius)
        .collect();
    let mut points = vec![Vec2::ZERO];
    points.extend(edge.iter().copied());
    let mut indices = Vec::new();
    for i in 0..steps {
        triangle(&points, &mut indices, 0, 1 + i, 1 + (i + 1) % steps);
    }
    (flat(points, indices), edging(&edge, WIDTH), edging(&edge, -RIM))
}

/// A line `width` wide along the closed `edge`: just inside it, or, for
/// a `width` less than nothing, just outside it ([`inset`]).
fn edging(edge: &[Vec2], width: f32) -> Mesh {
    edging_at(edge, &vec![0.0; edge.len()], width)
}

/// The same, each corner of `edge` as high over the ground as `heights`
/// has it, and the line beside it as high as its corner.
fn edging_at(edge: &[Vec2], heights: &[f32], width: f32) -> Mesh {
    let n = edge.len();
    let mut points = edge.to_vec();
    points.extend(inset(edge, width));
    let mut high = heights.to_vec();
    high.extend(heights.iter().copied());
    let mut indices = Vec::new();
    for i in 0..n {
        let j = (i + 1) % n;
        triangle(&points, &mut indices, i, j, n + j);
        triangle(&points, &mut indices, i, n + j, n + i);
    }
    raised(points, &high, indices)
}

/// What a sheet of it is made of: `colour`, as see-through as its alpha
/// says, unlit and drawn [`GAIN`] times as bright, from either side.
pub(crate) fn look(colour: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: bright(colour),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        fog_enabled: false,
        ..default()
    }
}

/// `colour` [`GAIN`] times as bright, its alpha as it is.
pub(crate) fn bright(colour: Color) -> Color {
    let linear = colour.to_linear();
    Color::linear_rgba(linear.red * GAIN, linear.green * GAIN, linear.blue * GAIN, linear.alpha)
}

/// The fill: the sector between its near arc and its far arc, a strip of
/// triangles lying flat and facing up; or, with no near arc, a fan from
/// the point where its sides meet.
fn sector(shape: Sector) -> Mesh {
    let outer = arc(shape, shape.outer);
    let mut indices = Vec::new();
    let mut points;
    if shape.inner <= NO_EDGE {
        points = vec![shape.apex()];
        points.extend(outer);
        for i in 1..points.len() - 1 {
            triangle(&points, &mut indices, 0, i, i + 1);
        }
    } else {
        let n = outer.len();
        points = outer;
        points.extend(arc(shape, shape.inner));
        for i in 0..n - 1 {
            triangle(&points, &mut indices, i, i + 1, n + i + 1);
            triangle(&points, &mut indices, i, n + i + 1, n + i);
        }
    }
    flat(points, indices)
}

/// The outline: a band [`WIDTH`] wide just inside the sector's edge, round
/// its far arc, down its two sides and round its near arc, its outer edge
/// the sector's own; all round, a ring at each arc.
fn outline(shape: Sector) -> Mesh {
    let mut outer = arc(shape, shape.outer);
    let mut inner = arc(shape, shape.inner);
    // Each closed edge, and whether its band lies inside it or round it.
    let mut edges: Vec<(Vec<Vec2>, bool)> = Vec::new();
    if shape.all_round() {
        // An arc all the way round ends where it began.
        outer.pop();
        inner.pop();
        edges.push((outer, true));
        if shape.inner > NO_EDGE {
            edges.push((inner, false));
        }
    } else {
        let mut edge = outer;
        if shape.inner > NO_EDGE {
            edge.extend(inner.into_iter().rev());
        } else {
            edge.push(shape.apex());
        }
        edges.push((edge, true));
    }
    let mut points = Vec::new();
    let mut indices = Vec::new();
    for (edge, inside) in edges {
        let n = edge.len();
        let first = points.len();
        let band = inset(&edge, if inside { WIDTH } else { -WIDTH });
        points.extend(edge);
        points.extend(band);
        for i in 0..n {
            let j = (i + 1) % n;
            triangle(&points, &mut indices, first + i, first + j, first + n + j);
            triangle(&points, &mut indices, first + i, first + n + j, first + n + i);
        }
    }
    flat(points, indices)
}

/// Points along the arc `radius` out from where the sector's sides meet,
/// from its left side round to its right, one every [`STEP_DEG`] or so,
/// in the ground's (x, z).
fn arc(shape: Sector, radius: f32) -> Vec<Vec2> {
    let whole = shape.half_cone * 2.0;
    let steps = ((whole.to_degrees() / STEP_DEG).ceil() as usize).max(4);
    let apex = shape.apex();
    (0..=steps)
        .map(|i| {
            let t = -shape.half_cone + whole * i as f32 / steps as f32;
            apex + Vec2::new(t.sin() * radius, -t.cos() * radius)
        })
        .collect()
}

/// The closed polygon `edge` drawn `width` in, corner for corner, or out,
/// for a `width` less than nothing (the band round a hole): each corner
/// moved along the bisector of its two sides as far as keeps the band
/// that wide along both (a mitre), and never more than four widths, at a
/// corner too sharp for that. Which side is in is read off which way
/// round the polygon goes.
fn inset(edge: &[Vec2], width: f32) -> Vec<Vec2> {
    let n = edge.len();
    // Twice the polygon's signed area: which way round it goes, and so on
    // which hand its inside lies.
    let turning: f32 = (0..n).map(|i| edge[i].perp_dot(edge[(i + 1) % n])).sum();
    let hand = if turning >= 0.0 { 1.0 } else { -1.0 };
    let inward = |i: usize| -> Vec2 { (edge[(i + 1) % n] - edge[i]).normalize_or(Vec2::X).perp() * hand };
    (0..n)
        .map(|i| {
            let before = inward((i + n - 1) % n);
            let after = inward(i);
            let mitre = (before + after) / (1.0 + before.dot(after)).max(1e-3);
            edge[i] + mitre.clamp_length_max(4.0) * width
        })
        .collect()
}

/// Adds the triangle of `points[a]`, `[b]` and `[c]`, wound to face up.
fn triangle(points: &[Vec2], indices: &mut Vec<u32>, a: usize, b: usize, c: usize) {
    let (p, q, r) = (points[a], points[b], points[c]);
    // In (x, z) a triangle faces up (+Y) when its corners go round the
    // other way than they would seen on a page.
    let (b, c) = if (q - p).perp_dot(r - p) > 0.0 { (c, b) } else { (b, c) };
    indices.extend([a as u32, b as u32, c as u32]);
}

/// A flat mesh of `points` in the ground's (x, z), facing up.
fn flat(points: Vec<Vec2>, indices: Vec<u32>) -> Mesh {
    let level = vec![0.0; points.len()];
    raised(points, &level, indices)
}

/// A mesh of `points` in the ground's (x, z), each as high over it as
/// `heights` has it, facing up.
fn raised(points: Vec<Vec2>, heights: &[f32], indices: Vec<u32>) -> Mesh {
    let positions: Vec<[f32; 3]> = points.iter().zip(heights).map(|(point, high)| [point.x, *high, point.y]).collect();
    let normals = vec![[0.0, 1.0, 0.0]; points.len()];
    let uvs = vec![[0.0, 0.0]; points.len()];
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PLAYER_RADIUS, punch_lands};

    /// Every corner of `mesh`, and the three corners of each of its
    /// triangles.
    fn corners(mesh: &Mesh) -> (Vec<Vec3>, Vec<[Vec3; 3]>) {
        let points: Vec<Vec3> = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|positions| positions.as_float3())
            .expect("positions")
            .iter()
            .map(|&point| Vec3::from(point))
            .collect();
        let order: Vec<usize> = mesh.indices().expect("indices").iter().collect();
        let triangles = order.chunks(3).map(|tri| [points[tri[0]], points[tri[1]], points[tri[2]]]).collect();
        (points, triangles)
    }

    fn faces_up(triangles: &[[Vec3; 3]]) -> bool {
        triangles.iter().all(|[a, b, c]| (*b - *a).cross(*c - *a).y > 0.0)
    }

    #[test]
    fn a_band_is_that_wide_from_here_to_there_ahead_of_you_with_its_outline_inside_it() {
        let (fill, outline) = band(0.8, 0.5, 20.0);
        let (points, triangles) = corners(&fill);
        assert_eq!((points.len(), triangles.len()), (4, 2));
        assert!(faces_up(&triangles));
        // Ahead is -Z: from half a metre in front of you to twenty.
        let (low, high) = points.iter().fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), p| (low.min(*p), high.max(*p)));
        assert!(low.abs_diff_eq(Vec3::new(-0.4, 0.0, -20.0), 1e-5) && high.abs_diff_eq(Vec3::new(0.4, 0.0, -0.5), 1e-5));
        let (points, triangles) = corners(&outline);
        assert_eq!((points.len(), triangles.len()), (8, 8));
        assert!(faces_up(&triangles));
        // The outline's own edge is the band's, and it reaches WIDTH in.
        for point in &points {
            let (across, along) = (point.x.abs(), -point.z);
            assert!(across <= 0.4 + 1e-5 && (0.5 - 1e-5..=20.0 + 1e-5).contains(&along), "{point:?}");
            let in_from_edge = (0.4 - across).min(along - 0.5).min(20.0 - along);
            assert!(in_from_edge < 1e-4 || (in_from_edge - WIDTH).abs() < 1e-4, "{point:?}");
        }
        // The rim's inner edge is the band's too, and it reaches RIM out
        // from it, all the way round: nothing of it is inside the band.
        let (points, triangles) = corners(&rim(0.8, 0.5, 20.0));
        assert_eq!((points.len(), triangles.len()), (8, 8));
        assert!(faces_up(&triangles));
        for point in &points {
            let (across, along) = (point.x.abs(), -point.z);
            let out_from_edge = (across - 0.4).max(0.5 - along).max(along - 20.0);
            assert!(out_from_edge.abs() < 1e-4 || (out_from_edge - RIM).abs() < 1e-4, "{point:?}");
        }
        let (low, high) = points.iter().fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), p| (low.min(*p), high.max(*p)));
        assert!(low.abs_diff_eq(Vec3::new(-0.4 - RIM, 0.0, -20.0 - RIM), 1e-5) && high.abs_diff_eq(Vec3::new(0.4 + RIM, 0.0, -0.5 + RIM), 1e-5));
    }

    #[test]
    fn the_file_s_range_begins_near_you_that_wide_and_reaches_that_far() {
        let melee = ui::ui().melee;
        let shape = Sector::of(&melee);
        // The near arc's middle is `near` in front of you, the far arc's
        // `reach`, and the near arc's corners `width` apart.
        assert!((shape.inner - shape.behind - melee.near).abs() < 1e-5);
        assert!((shape.outer - shape.behind - melee.reach).abs() < 1e-5);
        assert!((2.0 * shape.inner * shape.half_cone.sin() - melee.width).abs() < 1e-5);
        // With no width the sides meet `near` in front of you.
        let pointed = Sector::of(&ui::Melee { width: 0.0, ..melee });
        assert_eq!(pointed.inner, 0.0);
        assert!((pointed.behind + melee.near).abs() < 1e-6);
    }

    #[test]
    fn the_range_drawn_is_the_punch_s_own_and_faces_up() {
        let melee = ui::ui().melee;
        let shape = Sector::of(&melee);
        let apex = shape.apex();
        for mesh in [sector(shape), outline(shape)] {
            let (points, triangles) = corners(&mesh);
            let (mut nearest, mut furthest) = (f32::MAX, 0.0f32);
            let (mut left, mut right) = (0.0f32, 0.0f32);
            for point in &points {
                assert_eq!(point.y, 0.0);
                let from_apex = point.xz() - apex;
                let far = from_apex.length();
                nearest = nearest.min(far);
                furthest = furthest.max(far);
                assert!(far <= shape.outer + 1e-4, "{point} is past the far arc");
                assert!(far >= shape.inner - 1e-4, "{point} is short of the near arc");
                if far > 1e-4 {
                    assert!(Vec2::NEG_Y.angle_to(from_apex).abs() <= shape.half_cone + 1e-4, "{point} is outside the cone");
                }
                if (far - shape.inner).abs() < 1e-4 {
                    left = left.min(point.x);
                    right = right.max(point.x);
                }
            }
            assert!((furthest - shape.outer).abs() < 1e-4, "reaches {furthest} m from where the sides meet");
            assert!((nearest - shape.inner).abs() < 1e-4, "begins {nearest} m from where the sides meet");
            assert!((right - left - melee.width).abs() < 1e-3, "{} m wide at its near arc", right - left);
            assert!(faces_up(&triangles));
        }
        // And whoever stands at its far corner is hit; just outside its
        // side, their own width past its far arc, or their own width short
        // of its near arc, they are not.
        let at = |off: f32, far: f32| {
            let there = apex + Vec2::new(off.sin() * far, -off.cos() * far);
            Vec3::new(there.x, 0.0, there.y)
        };
        let a = Entity::from_bits(1);
        let lands = |there: Vec3| {
            punch_lands(&melee, Vec2::ZERO, 0.0, Vec2::NEG_Y, [(a, there)].into_iter()).is_some()
        };
        assert!(lands(at(shape.half_cone - 0.01, shape.outer)));
        assert!(!lands(at(shape.half_cone + 0.05, shape.outer)));
        assert!(lands(at(0.0, shape.outer + PLAYER_RADIUS - 0.05)));
        assert!(!lands(at(0.0, shape.outer + PLAYER_RADIUS + 0.05)));
        let short = shape.inner - PLAYER_RADIUS;
        assert!(lands(at(0.0, short.max(0.0) + 0.05)));
        if short > 0.06 {
            assert!(!lands(at(0.0, short - 0.05)));
        }
    }

    #[test]
    fn the_outline_is_a_band_just_inside_the_edge() {
        let melee = ui::ui().melee;
        let shape = Sector::of(&melee);
        let (points, _) = corners(&outline(shape));
        let between = |low: f32, high: f32| {
            points.iter().map(|point| (point.xz() - shape.apex()).length()).filter(move |&far| far > low && far < high)
        };
        // The band along the far arc runs WIDTH in from it, and the one
        // along the near arc WIDTH out from it.
        let far_band = between(shape.outer - WIDTH * 2.0, shape.outer - 1e-4).fold(f32::MAX, f32::min);
        assert!((far_band - (shape.outer - WIDTH)).abs() < 0.01, "the far band reaches in to {far_band} m");
        let near_band = between(shape.inner + 1e-4, shape.inner + WIDTH * 2.0).fold(0.0f32, f32::max);
        assert!((near_band - (shape.inner + WIDTH)).abs() < 0.01, "the near band reaches out to {near_band} m");
        // All round, a ring at each arc and nothing between them.
        let round = Sector { half_cone: 180f32.to_radians(), ..shape };
        let (rings, triangles) = corners(&outline(round));
        assert!(rings.iter().all(|point| {
            let far = (point.xz() - round.apex()).length();
            far > round.outer - WIDTH * 1.01 || far < round.inner + WIDTH * 1.01
        }));
        assert!(faces_up(&triangles));
        // With no width, both come to the point where the sides meet.
        let pointed = Sector::of(&ui::Melee { width: 0.0, ..melee });
        for mesh in [sector(pointed), outline(pointed)] {
            let (points, triangles) = corners(&mesh);
            assert!(points.iter().any(|point| point.xz().distance(pointed.apex()) < 1e-6));
            assert!(faces_up(&triangles));
        }
    }

    #[test]
    fn it_turns_to_face_the_way_you_face() {
        for facing in [Vec2::NEG_Y, Vec2::X, Vec2::Y, Vec2::new(-0.6, -0.8)] {
            let ahead = (turned_to(facing) * Vec3::NEG_Z).xz();
            assert!(ahead.distance(facing) < 1e-5, "{facing} came out {ahead}");
        }
    }
}
