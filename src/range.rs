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

use crate::ui::{self, UiChanged};
use crate::{LAND_STEP_UP, Player, drive};

/// How high over your feet it lies, in metres: as high as a step, so that
/// nothing you could step up onto hides any of it.
const HEIGHT: f32 = LAND_STEP_UP;

/// How wide the outline is, in metres, just inside the sector's edge:
/// about three pixels on a phone with the camera at its usual 24 m over
/// you, one at its furthest and ten at its nearest.
const WIDTH: f32 = 0.1;

/// The outline lies this much over the fill, so that it is drawn after
/// it, over it.
const OVER: f32 = 0.002;

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
/// ground, and puts it away in the car, or with no body to stand round.
fn follow(
    driving: Res<drive::Driving>,
    me: Query<&Transform, With<Player>>,
    mut ranges: Query<(&mut Transform, &mut Visibility), (With<HitRange>, Without<Player>)>,
) {
    let Ok((mut place, mut shown)) = ranges.single_mut() else {
        return;
    };
    let body = me.single().ok().filter(|_| !driving.on());
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
fn turned_to(facing: Vec2) -> Quat {
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

/// What a sheet of it is made of: `colour`, as see-through as its alpha
/// says, unlit and drawn [`GAIN`] times as bright, from either side.
fn look(colour: Color) -> StandardMaterial {
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
fn bright(colour: Color) -> Color {
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
    let positions: Vec<[f32; 3]> = points.iter().map(|point| [point.x, 0.0, point.y]).collect();
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
