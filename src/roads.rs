//! The town's roads, read off its model, and the way the car drives them
//! (`car`).
//!
//! A road is wherever the top of the town is the material [`ROAD`], with a
//! dashed line down its middle in [`MARKING`]. Nothing here knows where any
//! road is. Once the town has spawned, its dashes are read out of it
//! ([`dashes`]), and every dash in line with the next makes a road's middle,
//! round on itself or open at both ends ([`middles`]); how far the road
//! reaches either side of it, measured off the model, is how wide it is
//! ([`widths`]). A middle that runs out with its last dash goes on to the
//! middle of the road it meets, if it meets one within [`REACH`], and there
//! the two join. Where roads join, or cross, each is cut, and what is left
//! between two joins is one stretch of road ([`network`]). Reshape the roads
//! in Blender, paint their middles, and the car drives the new ones.
//!
//! The car keeps to the right, as Korea drives, so every stretch has two
//! lanes, one each way, each halfway out from the middle to the edge on its
//! side ([`lanes`]). It drives every lane of every stretch, both ways down
//! every road, in one loop through the town ([`plan`]), turning from lane to
//! lane where roads join and never back the way it came, but where a road
//! ends. A loop that drives each lane exactly once is not always to be had:
//! with only T-junctions, as Round Town has, it never is, as every way of
//! taking the turns leaves the lanes in an even number of loops. Then one
//! stretch is driven twice, each way, the shortest that makes it one loop.
//!
//! How fast the car goes along the loop is worked out from how sharply it
//! turns ([`Route::drive`]), and how long the loop takes is a whole number of
//! seconds, so that every device, whatever its arithmetic, puts the car in the
//! same place at the same time (`car`).

use bevy::prelude::*;

use crate::island::Island;

/// What a road's surface is called in the town's model.
pub(crate) const ROAD: &str = "road";
/// What the dashes down a road's middle are called, and the town's other
/// white road paint.
pub(crate) const MARKING: &str = "roadwhite";

/// How near two corners have to be to be one, in metres, in telling the
/// dashes of a mesh apart.
const WELD: f32 = 1e-3;
/// How many times as long as it is wide a mark has to be to be a dash.
const SLENDER: f32 = 1.5;
/// How far apart, middle to middle, two dashes of one road can be, as many
/// times as long as the longer is: the town's are 1.5 to 1.9 times.
const GAP: f32 = 3.0;
/// How nearly in line two dashes have to be to be one road's: lying within
/// 30° of each other, and each within 30° of the way to the other.
const IN_LINE: f32 = 0.866;
/// The fewest dashes that make a road.
const FEWEST_DASHES: usize = 3;
/// How many points a road's middle is drawn through between two dashes.
const BETWEEN_DASHES: usize = 8;
/// How far past its last dash a road's middle is followed to the middle of
/// the road it runs into, in metres: the straight roads' last dashes are 1.2
/// m short of the inner road's middle and 5.7 m short of the outer road's.
const REACH: f32 = 12.0;
/// How near two joins can be and be one, in metres.
const JOIN: f32 = 2.0;
/// How finely a road's width is felt for, in metres, before the edge is
/// found more finely between the last step on it and the first off it.
const STEP: f32 = 0.25;
/// The furthest out from its middle a road's edge is looked for, in metres.
const WIDEST: f32 = 12.0;
/// How far over a road's surface the top of the town can be and still be the
/// road, in metres: its paint, a centimetre thick, but not a kerb.
const PAINT: f32 = 0.02;
/// How far over a road's surface its edge is looked for, in metres: higher
/// than a kerb, and lower than the arm of a lamp over the road.
const OVER_THE_ROAD: f32 = 1.0;
/// How much further back than the edge of the road it meets a lane stops
/// short of a join, to turn from, in metres.
const ROOM_TO_TURN: f32 = 3.0;
/// The most of a lane that is cut off its two ends together to turn from.
const MOST_CUT: f32 = 0.7;
/// How far apart the points along the car's loop are, in metres.
const SPACING: f32 = 0.5;
/// How hard the car can turn, as the push it can take sideways, in metres a
/// second each second: round the inner road's inside lane, 15.5 m round,
/// nearly at its top speed, and into a side road at half of it.
const GRIP: f32 = 2.5;
/// How fast it speeds up and how hard it slows down, in metres a second each
/// second.
const SPEED_UP: f32 = 1.5;
const SLOW_DOWN: f32 = 2.5;
/// The slowest it goes, in metres a second.
const SLOWEST: f32 = 2.0;
/// How far round a loop the sharpness of its turns is taken over, either way,
/// in points: a metre and a half.
const SMOOTHING: usize = 3;
/// How many ways of taking the turns are tried for one loop through them all.
const TRIES: u32 = 4096;

/// One dash of a road's middle: where its middle is, which way it lies, flat
/// on the ground, how long it is, and how high its top is.
#[derive(Clone, Copy, Debug)]
struct Dash {
    at: Vec2,
    along: Vec2,
    length: f32,
    top: f32,
}

/// The dashes in `markings`, each one mesh's triangles, placed in the world,
/// as across the ground (x, z): every piece of each mesh that is longer than
/// it is wide. A piece is what its triangles make, joined at their corners.
fn dashes(markings: &[Vec<[Vec3; 3]>]) -> Vec<Dash> {
    let mut found = Vec::new();
    for mesh in markings {
        for corners in pieces(mesh) {
            let middle = corners.iter().map(|corner| corner.xz()).sum::<Vec2>() / corners.len() as f32;
            let along = long_way(&corners, middle);
            let spread = |way: Vec2| {
                corners.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), corner| {
                    let at = (corner.xz() - middle).dot(way);
                    (low.min(at), high.max(at))
                })
            };
            let (back, front) = spread(along);
            let (left, right) = spread(along.perp());
            let (length, width) = (front - back, right - left);
            if length < width.max(1e-3) * SLENDER {
                continue;
            }
            found.push(Dash {
                at: middle + along * ((back + front) * 0.5) + along.perp() * ((left + right) * 0.5),
                along,
                length,
                top: corners.iter().map(|corner| corner.y).fold(f32::NEG_INFINITY, f32::max),
            });
        }
    }
    found
}

/// The pieces of a mesh, its triangles joined where they share a corner:
/// each piece's corners, every one once, in the order of each piece's first
/// triangle, which every device reads alike.
fn pieces(triangles: &[[Vec3; 3]]) -> Vec<Vec<Vec3>> {
    let key = |corner: Vec3| {
        let at = (corner / WELD).round().as_ivec3();
        (at.x, at.y, at.z)
    };
    fn root(parent: &mut [usize], mut at: usize) -> usize {
        while parent[at] != at {
            parent[at] = parent[parent[at]];
            at = parent[at];
        }
        at
    }
    let mut parent: Vec<usize> = (0..triangles.len()).collect();
    let mut owner = std::collections::BTreeMap::new();
    for (index, triangle) in triangles.iter().enumerate() {
        for &corner in triangle {
            match owner.get(&key(corner)) {
                Some(&other) => {
                    let (a, b) = (root(&mut parent, other), root(&mut parent, index));
                    parent[a.max(b)] = a.min(b);
                }
                None => {
                    owner.insert(key(corner), index);
                }
            }
        }
    }
    let mut first_of: std::collections::BTreeMap<usize, usize> = Default::default();
    let mut pieces: Vec<(Vec<Vec3>, std::collections::BTreeSet<(i32, i32, i32)>)> = Vec::new();
    for (index, triangle) in triangles.iter().enumerate() {
        let top = root(&mut parent, index);
        let piece = *first_of.entry(top).or_insert_with(|| {
            pieces.push(Default::default());
            pieces.len() - 1
        });
        let (corners, seen) = &mut pieces[piece];
        for &corner in triangle {
            if seen.insert(key(corner)) {
                corners.push(corner);
            }
        }
    }
    pieces.into_iter().map(|(corners, _)| corners).collect()
}

/// Which way across the ground `corners`, round `middle`, lie the longest: as
/// a direction a metre long.
fn long_way(corners: &[Vec3], middle: Vec2) -> Vec2 {
    let (mut xx, mut xz, mut zz) = (0.0, 0.0, 0.0);
    for corner in corners {
        let d = corner.xz() - middle;
        xx += d.x * d.x;
        xz += d.x * d.y;
        zz += d.y * d.y;
    }
    // The way of the larger of the two spreads they have across the ground.
    let most = (xx + zz) * 0.5 + (((xx - zz) * 0.5).powi(2) + xz * xz).sqrt();
    let (a, b) = (Vec2::new(xz, most - xx), Vec2::new(most - zz, xz));
    if a.length_squared() >= b.length_squared() {
        a.normalize_or(Vec2::X)
    } else {
        b.normalize_or(Vec2::X)
    }
}

/// A road's middle: a smooth line through its dashes, round on itself or open
/// at both ends, across the ground, and how high its paint is.
#[derive(Clone, Debug)]
struct Middle {
    points: Vec<Vec2>,
    closed: bool,
    /// The middles of its dashes, which its width is measured at, and which
    /// every [`BETWEEN_DASHES`]th of its points is.
    dashes: Vec<Vec2>,
    top: f32,
}

/// Every road's middle among `dashes`: each dash joined to the nearest dash
/// in line with it on either side of it, where that dash's nearest is it too.
fn middles(dashes: &[Dash]) -> Vec<Middle> {
    let n = dashes.len();
    // The nearest dash on each side of each dash, along it.
    let mut nearest = vec![[None::<(f32, usize)>; 2]; n];
    for (i, a) in dashes.iter().enumerate() {
        for (j, b) in dashes.iter().enumerate() {
            let apart = b.at - a.at;
            let far = apart.length();
            if i == j || far > GAP * a.length.max(b.length) || far < 1e-4 {
                continue;
            }
            let way = apart / far;
            if a.along.dot(b.along).abs() < IN_LINE || way.dot(a.along).abs() < IN_LINE {
                continue;
            }
            let side = usize::from(way.dot(a.along) < 0.0);
            if nearest[i][side].is_none_or(|(near, _)| far < near) {
                nearest[i][side] = Some((far, j));
            }
        }
    }
    let mut links: Vec<Vec<usize>> = vec![Vec::new(); n];
    for i in 0..n {
        for (_, j) in nearest[i].iter().flatten().copied() {
            if nearest[j].iter().flatten().any(|&(_, k)| k == i) && !links[i].contains(&j) {
                links[i].push(j);
                links[j].push(i);
            }
        }
    }
    for list in &mut links {
        list.sort_unstable();
        list.dedup();
    }
    // Walked from one end of each open line, then round each closed one, from
    // its first dash toward the first of the two it is joined to.
    let mut done = vec![false; n];
    let walk = |start: usize, done: &mut Vec<bool>| {
        let mut chain = vec![start];
        done[start] = true;
        let mut at = start;
        while let Some(&next) = links[at].iter().find(|&&next| !done[next]) {
            done[next] = true;
            chain.push(next);
            at = next;
        }
        let closed = chain.len() > 2 && links[at].contains(&start);
        (chain, closed)
    };
    let mut chains = Vec::new();
    for start in 0..n {
        if !done[start] && links[start].len() == 1 {
            chains.push(walk(start, &mut done));
        }
    }
    for start in 0..n {
        if !done[start] && links[start].len() == 2 {
            chains.push(walk(start, &mut done));
        }
    }
    chains
        .into_iter()
        .filter(|(chain, _)| chain.len() >= FEWEST_DASHES)
        .map(|(chain, closed)| {
            let at: Vec<Vec2> = chain.iter().map(|&dash| dashes[dash].at).collect();
            let mut tops: Vec<f32> = chain.iter().map(|&dash| dashes[dash].top).collect();
            tops.sort_by(f32::total_cmp);
            Middle {
                points: smooth(&at, closed),
                closed,
                top: tops[tops.len() / 2],
                dashes: at,
            }
        })
        .collect()
}

/// A smooth line through `points`, round from the last back to the first if
/// `closed`: [`BETWEEN_DASHES`] points from each to the next, the first of
/// them it.
fn smooth(points: &[Vec2], closed: bool) -> Vec<Vec2> {
    let n = points.len() as isize;
    let at = |i: isize| -> Vec2 {
        if closed {
            points[i.rem_euclid(n) as usize]
        } else if i < 0 {
            points[0] * 2.0 - points[1]
        } else if i >= n {
            points[n as usize - 1] * 2.0 - points[n as usize - 2]
        } else {
            points[i as usize]
        }
    };
    let spans = if closed { n } else { n - 1 };
    let mut line = Vec::with_capacity(spans as usize * BETWEEN_DASHES + 1);
    for span in 0..spans {
        let (p0, p1, p2, p3) = (at(span - 1), at(span), at(span + 1), at(span + 2));
        for step in 0..BETWEEN_DASHES {
            let t = step as f32 / BETWEEN_DASHES as f32;
            let (t2, t3) = (t * t, t * t * t);
            line.push(
                (p1 * 2.0
                    + (p2 - p0) * t
                    + (p0 * 2.0 - p1 * 5.0 + p2 * 4.0 - p3) * t2
                    + (p1 * 3.0 - p0 - p2 * 3.0 + p3) * t3)
                    * 0.5,
            );
        }
    }
    if !closed {
        line.push(points[n as usize - 1]);
    }
    line
}

/// How far from the start of `points` each of them is along it, and, if it is
/// `closed`, last, the whole way round, back at the first.
fn along(points: &[Vec2], closed: bool) -> Vec<f32> {
    let mut so_far = vec![0.0];
    for pair in points.windows(2) {
        so_far.push(so_far.last().unwrap() + pair[0].distance(pair[1]));
    }
    if closed && points.len() > 1 {
        so_far.push(so_far.last().unwrap() + points[points.len() - 1].distance(points[0]));
    }
    so_far
}

/// The point `s` along `points`, whose distances along are `so_far`.
fn point_at(points: &[Vec2], so_far: &[f32], s: f32) -> Vec2 {
    let last = so_far.len() - 1;
    let after = so_far.partition_point(|&d| d <= s).clamp(1, last);
    let (a, b) = (points[(after - 1) % points.len()], points[after % points.len()]);
    let span = so_far[after] - so_far[after - 1];
    let share = if span > 0.0 { ((s - so_far[after - 1]) / span).clamp(0.0, 1.0) } else { 0.0 };
    a.lerp(b, share)
}

/// The stretch of `points` from `from` to `to` along it: round past its start,
/// if it is `closed` and `to` comes before `from`.
fn stretch(points: &[Vec2], closed: bool, from: f32, to: f32) -> Vec<Vec2> {
    let so_far = along(points, closed);
    let whole = *so_far.last().unwrap();
    let to = if closed && to <= from { to + whole } else { to };
    let mut out = vec![point_at(points, &so_far, from)];
    let laps = if closed { 2 } else { 1 };
    for lap in 0..laps {
        for (index, &d) in so_far.iter().enumerate().take(points.len()) {
            let d = d + whole * lap as f32;
            if d > from && d < to {
                out.push(points[index]);
            }
        }
    }
    out.push(point_at(points, &so_far, if to > whole { to - whole } else { to }));
    out
}

/// How far along `points` the point of it nearest `at` is, and how near it
/// is.
fn locate(points: &[Vec2], closed: bool, at: Vec2) -> (f32, f32) {
    let so_far = along(points, closed);
    let spans = if closed { points.len() } else { points.len() - 1 };
    let mut best = (0.0, f32::INFINITY);
    for span in 0..spans {
        let (a, b) = (points[span], points[(span + 1) % points.len()]);
        let side = b - a;
        let long = side.length_squared();
        let u = if long > 0.0 { ((at - a).dot(side) / long).clamp(0.0, 1.0) } else { 0.0 };
        let near = (a + side * u).distance(at);
        if near < best.1 {
            best = (so_far[span] + u * long.sqrt(), near);
        }
    }
    best
}

/// Where the line from `from` along `way` meets `points`, if it does within
/// `reach`: how far along the line, and the point.
fn meet(from: Vec2, way: Vec2, reach: f32, points: &[Vec2], closed: bool) -> Option<(f32, Vec2)> {
    let spans = if closed { points.len() } else { points.len() - 1 };
    let mut best: Option<(f32, Vec2)> = None;
    for span in 0..spans {
        let (a, b) = (points[span], points[(span + 1) % points.len()]);
        let side = b - a;
        let turn = way.perp_dot(side);
        if turn.abs() < 1e-9 {
            continue;
        }
        let gap = a - from;
        let t = gap.perp_dot(side) / turn;
        let u = gap.perp_dot(way) / turn;
        if !(-1e-4..=1.0 + 1e-4).contains(&u) || !(0.0..=reach).contains(&t) {
            continue;
        }
        if best.is_none_or(|(near, _)| t < near) {
            best = Some((t, from + way * t));
        }
    }
    best
}

/// Where two roads' middles cross: how far along each, and the point.
fn crossings(a: &[Vec2], a_closed: bool, b: &[Vec2], b_closed: bool) -> Vec<(f32, f32, Vec2)> {
    let (a_far, b_far) = (along(a, a_closed), along(b, b_closed));
    let a_spans = if a_closed { a.len() } else { a.len() - 1 };
    let b_spans = if b_closed { b.len() } else { b.len() - 1 };
    let mut found = Vec::new();
    for i in 0..a_spans {
        let (p, r) = (a[i], a[(i + 1) % a.len()] - a[i]);
        for j in 0..b_spans {
            let (q, s) = (b[j], b[(j + 1) % b.len()] - b[j]);
            let turn = r.perp_dot(s);
            if turn.abs() < 1e-9 {
                continue;
            }
            let t = (q - p).perp_dot(s) / turn;
            let u = (q - p).perp_dot(r) / turn;
            if (0.0..1.0).contains(&t) && (0.0..1.0).contains(&u) {
                found.push((a_far[i] + t * r.length(), b_far[j] + u * s.length(), p + r * t));
            }
        }
    }
    found
}

/// What the town's model has to say about its roads, once it has spawned:
/// its whole shape, the shape of its roads alone, and its paint.
pub(crate) struct Paint<'a> {
    /// The town, everything in it.
    pub(crate) land: &'a Island,
    /// Only its roads' surfaces.
    pub(crate) road: &'a Island,
    /// Each mesh of its road paint, placed in the world.
    pub(crate) markings: &'a [Vec<[Vec3; 3]>],
}

impl Paint<'_> {
    /// Whether the top of the town at `at` is road, for a road painted at
    /// `top`.
    fn on_road(&self, at: Vec2, top: f32) -> bool {
        let reach = top + OVER_THE_ROAD;
        match (self.land.floor(at, reach), self.road.floor(at, reach)) {
            (Some(land), Some(road)) => land - road <= PAINT,
            _ => false,
        }
    }

    /// How far out from `at` toward `way` the road goes, to its edge, for a
    /// road painted at `top`: none if `at` is not on a road, and as far as
    /// [`WIDEST`] if it goes on further.
    fn edge(&self, at: Vec2, way: Vec2, top: f32) -> f32 {
        if !self.on_road(at, top) {
            return 0.0;
        }
        let (mut on, mut out) = (0.0, STEP);
        while out <= WIDEST {
            if !self.on_road(at + way * out, top) {
                let mut off = out;
                for _ in 0..6 {
                    let half = (on + off) * 0.5;
                    if self.on_road(at + way * half, top) {
                        on = half;
                    } else {
                        off = half;
                    }
                }
                return (on + off) * 0.5;
            }
            on = out;
            out += STEP;
        }
        WIDEST
    }
}

/// How far a road reaches either side of its middle, left and right as it
/// goes: the middle of what is measured at each of its dashes, so that where
/// another road joins it, and it seems to go on and on, does not count.
fn widths(middle: &Middle, paint: &Paint) -> (f32, f32) {
    let (mut lefts, mut rights) = (Vec::new(), Vec::new());
    let count = middle.points.len();
    for (index, &at) in middle.dashes.iter().enumerate() {
        // The line's own way at the dash, from its points either side.
        let here = index * BETWEEN_DASHES;
        let ahead = middle.points[if middle.closed { (here + 1) % count } else { (here + 1).min(count - 1) }];
        let behind = middle.points[if middle.closed { (here + count - 1) % count } else { here.saturating_sub(1) }];
        let way = (ahead - behind).normalize_or_zero();
        if way == Vec2::ZERO {
            continue;
        }
        rights.push(paint.edge(at, way.perp(), middle.top));
        lefts.push(paint.edge(at, -way.perp(), middle.top));
    }
    let typical = |mut list: Vec<f32>| {
        list.sort_by(f32::total_cmp);
        list.get(list.len() / 2).copied().unwrap_or(0.0)
    };
    (typical(lefts), typical(rights))
}

/// One stretch of road, between two joins, or a join and the end of the road:
/// its middle, from one to the other, how far it reaches either side of it,
/// and how high its paint is.
#[derive(Clone, Debug)]
struct Stretch {
    points: Vec<Vec2>,
    from: usize,
    to: usize,
    left: f32,
    right: f32,
    top: f32,
}

/// The town's roads: where they join, and the stretches between.
#[derive(Clone, Debug, Default)]
struct Network {
    joins: Vec<Vec2>,
    stretches: Vec<Stretch>,
}

/// The join at `at`, or a new one there: one within [`JOIN`] of it is the
/// same.
fn join_at(joins: &mut Vec<Vec2>, at: Vec2) -> usize {
    match joins.iter().position(|join| join.distance(at) <= JOIN) {
        Some(join) => join,
        None => {
            joins.push(at);
            joins.len() - 1
        }
    }
}

/// The roads in `paint`, joined where they meet and cut into stretches there.
fn network(paint: &Paint) -> Network {
    let mut middles = middles(&dashes(paint.markings));
    let sides: Vec<(f32, f32)> = middles.iter().map(|middle| widths(middle, paint)).collect();
    // Where each open end runs into the middle of another road, if it does,
    // found before any of them is made longer; then each goes on to it.
    let mut ends: Vec<(usize, bool, usize, Vec2)> = Vec::new();
    for (line, middle) in middles.iter().enumerate() {
        if middle.closed {
            continue;
        }
        for end in [false, true] {
            let points = &middle.points;
            let (tip, before) = if end {
                (points[points.len() - 1], points[points.len() - 2])
            } else {
                (points[0], points[1])
            };
            let way = (tip - before).normalize_or_zero();
            let met = middles
                .iter()
                .enumerate()
                .filter(|&(other, _)| other != line)
                .filter_map(|(other, road)| {
                    meet(tip, way, REACH, &road.points, road.closed).map(|(t, at)| (t, other, at))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            if let Some((_, other, at)) = met {
                ends.push((line, end, other, at));
            }
        }
    }
    for &(line, end, _, at) in &ends {
        let points = &mut middles[line].points;
        if end {
            points.push(at);
        } else {
            points.insert(0, at);
        }
    }
    let mut joins: Vec<Vec2> = Vec::new();
    let mut cuts: Vec<Vec<(f32, usize)>> = vec![Vec::new(); middles.len()];
    for &(line, end, other, at) in &ends {
        let join = join_at(&mut joins, at);
        cuts[line].push((if end { f32::INFINITY } else { 0.0 }, join));
        let road = &middles[other];
        cuts[other].push((locate(&road.points, road.closed, at).0, join));
    }
    // And roads that cross, each going on past the other.
    for a in 0..middles.len() {
        for b in a + 1..middles.len() {
            let (one, two) = (&middles[a], &middles[b]);
            for (sa, sb, at) in crossings(&one.points, one.closed, &two.points, two.closed) {
                let join = join_at(&mut joins, at);
                cuts[a].push((sa, join));
                cuts[b].push((sb, join));
            }
        }
    }
    let mut stretches = Vec::new();
    for (line, middle) in middles.iter().enumerate() {
        let whole = *along(&middle.points, middle.closed).last().unwrap();
        let mut marks: Vec<(f32, usize)> = cuts[line].iter().map(|&(s, join)| (s.min(whole), join)).collect();
        marks.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        // A join met twice along a line, at the end of one road and on the
        // middle of it, say, is one place.
        marks.dedup_by(|next, kept| next.1 == kept.1);
        if middle.closed && marks.len() > 1 && marks[0].1 == marks[marks.len() - 1].1 {
            marks.pop();
        }
        if !middle.closed {
            // An end that joins nothing is the end of the road.
            if marks.first().is_none_or(|first| first.0 > JOIN) {
                marks.insert(0, (0.0, join_at(&mut joins, middle.points[0])));
            }
            if marks.last().is_none_or(|last| last.0 < whole - JOIN) {
                marks.push((whole, join_at(&mut joins, *middle.points.last().unwrap())));
            }
        }
        let (left, right) = sides[line];
        let pairs = if middle.closed { marks.len() } else { marks.len().saturating_sub(1) };
        for index in 0..pairs {
            let (from, to) = (marks[index], marks[(index + 1) % marks.len()]);
            let long = if to.0 > from.0 { to.0 - from.0 } else { to.0 + whole - from.0 };
            if long <= JOIN && pairs > 1 {
                continue;
            }
            stretches.push(Stretch {
                points: stretch(&middle.points, middle.closed, from.0, to.0),
                from: from.1,
                to: to.1,
                left,
                right,
                top: middle.top,
            });
        }
    }
    // Too narrow for two lanes of the car is no road.
    stretches.retain(|stretch| stretch.left > 1.0 && stretch.right > 1.0);
    Network { joins, stretches }
}

/// One lane: one way down one stretch, kept to the right of its middle, with
/// its ends cut back from the joins to leave room to turn.
#[derive(Clone, Debug)]
struct Lane {
    points: Vec<Vec2>,
    from: usize,
    to: usize,
    /// Which stretch it is a lane of, and whether it goes the way the stretch
    /// does.
    stretch: usize,
    ahead: bool,
    /// How high the road's paint is.
    top: f32,
}

/// Both lanes of every stretch of `network`.
fn lanes(network: &Network) -> Vec<Lane> {
    // How far back from each join a lane stops: the half of the widest road
    // there, which a turn crosses, and some room.
    let mut room = vec![0.0f32; network.joins.len()];
    for stretch in &network.stretches {
        for join in [stretch.from, stretch.to] {
            room[join] = room[join].max(stretch.left.max(stretch.right) + ROOM_TO_TURN);
        }
    }
    let mut lanes = Vec::new();
    for (index, road) in network.stretches.iter().enumerate() {
        for ahead in [true, false] {
            let points = &road.points;
            let n = points.len();
            let mut lane: Vec<Vec2> = (0..n)
                .map(|at| {
                    let way = (points[(at + 1).min(n - 1)] - points[at.saturating_sub(1)]).normalize_or_zero();
                    // On the right of the middle, going the way the lane goes.
                    if ahead {
                        points[at] + way.perp() * (road.right * 0.5)
                    } else {
                        points[at] - way.perp() * (road.left * 0.5)
                    }
                })
                .collect();
            let (from, to) = if ahead { (road.from, road.to) } else { (road.to, road.from) };
            if !ahead {
                lane.reverse();
            }
            let whole = *along(&lane, false).last().unwrap();
            let (mut start, mut end) = (room[from], room[to]);
            if start + end > whole * MOST_CUT {
                let scale = whole * MOST_CUT / (start + end);
                start *= scale;
                end *= scale;
            }
            lanes.push(Lane {
                points: stretch(&lane, false, start, whole - end),
                from,
                to,
                stretch: index,
                ahead,
                top: road.top,
            });
        }
    }
    lanes
}

/// The lanes `drive` names, in an order that makes one loop through them:
/// each lane's end is where the next one's start is, and it never turns back
/// down the stretch it came along, but where nothing else is there to turn
/// into. A lane named twice is driven twice. `None` if there is no such loop.
///
/// It goes by the way the turns are taken at each join, which pairs each lane
/// coming into it with one going out of it: tried at random, from a seed of
/// its own, until one pairing at every join makes a single loop.
fn circuit(lanes: &[Lane], drive: &[usize], joins: usize) -> Option<Vec<usize>> {
    let n = drive.len();
    if n == 0 {
        return None;
    }
    let into: Vec<Vec<usize>> = (0..joins)
        .map(|join| (0..n).filter(|&k| lanes[drive[k]].to == join).collect())
        .collect();
    let out_of: Vec<Vec<usize>> = (0..joins)
        .map(|join| (0..n).filter(|&k| lanes[drive[k]].from == join).collect())
        .collect();
    let back = |a: usize, b: usize| {
        let (a, b) = (&lanes[drive[a]], &lanes[drive[b]]);
        a.stretch == b.stretch && a.ahead != b.ahead
    };
    let mut ways: Vec<Vec<Vec<usize>>> = Vec::with_capacity(joins);
    for join in 0..joins {
        let (ins, outs) = (&into[join], &out_of[join]);
        if ins.len() != outs.len() || ins.len() > 7 {
            return None;
        }
        let every = orders(outs);
        let mut turns: Vec<Vec<usize>> = every
            .iter()
            .filter(|order| ins.iter().zip(order.iter()).all(|(&i, &o)| !back(i, o)))
            .cloned()
            .collect();
        // At the end of a road the only way on is back.
        if turns.is_empty() {
            turns = every;
        }
        ways.push(turns);
    }
    let mut seed: u32 = 0x2545_F491;
    let mut random = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed as usize
    };
    let mut next = vec![0; n];
    for _ in 0..TRIES {
        for join in 0..joins {
            if into[join].is_empty() {
                continue;
            }
            let pick = &ways[join][random() % ways[join].len()];
            for (&i, &o) in into[join].iter().zip(pick.iter()) {
                next[i] = o;
            }
        }
        let mut order = vec![0];
        let mut at = next[0];
        while at != 0 && order.len() <= n {
            order.push(at);
            at = next[at];
        }
        if order.len() == n && at == 0 {
            return Some(order.into_iter().map(|k| drive[k]).collect());
        }
    }
    None
}

/// Every order of `items`.
fn orders(items: &[usize]) -> Vec<Vec<usize>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut all = Vec::new();
    for (index, &first) in items.iter().enumerate() {
        let rest: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|&(other, _)| other != index)
            .map(|(_, &item)| item)
            .collect();
        for mut order in orders(&rest) {
            order.insert(0, first);
            all.push(order);
        }
    }
    all
}

/// The order the car drives `lanes` in, round the town: each lane once if
/// that makes one loop, and if not, one stretch driven twice, each way, the
/// shortest that does. `None` if no loop takes them all in.
fn plan(lanes: &[Lane], joins: usize) -> Option<Vec<usize>> {
    let every: Vec<usize> = (0..lanes.len()).collect();
    if let Some(order) = circuit(lanes, &every, joins) {
        return Some(order);
    }
    // By whole metres, so that stretches all but as long as each other are
    // tried in the order they come in, on every device.
    let stretches = lanes.iter().map(|lane| lane.stretch).max()? + 1;
    let length = |stretch: usize| -> i64 {
        lanes
            .iter()
            .filter(|lane| lane.stretch == stretch)
            .map(|lane| *along(&lane.points, false).last().unwrap())
            .sum::<f32>()
            .round() as i64
    };
    let mut twice: Vec<usize> = (0..stretches).collect();
    twice.sort_by_key(|&stretch| (length(stretch), stretch));
    twice.into_iter().find_map(|stretch| {
        let drive: Vec<usize> = every
            .iter()
            .copied()
            .chain((0..lanes.len()).filter(|&lane| lanes[lane].stretch == stretch))
            .collect();
        circuit(lanes, &drive, joins)
    })
}

/// A turn from the end of one lane, at `from` going `from_way`, to the start
/// of the next, at `to` going `to_way`: a curve leaving the one and joining
/// the other along them, about as round as a circle's arc through both. Its
/// points between the two, not them.
fn turn(from: Vec2, from_way: Vec2, to: Vec2, to_way: Vec2) -> Vec<Vec2> {
    let chord = from.distance(to);
    // How far out along each way a circle's arc would bulge: a third of the
    // way across for no turn at all, two thirds for turning right round.
    let half_turn = ((1.0 + from_way.dot(to_way)) * 0.5).max(0.0).sqrt();
    let handle = chord * 2.0 / (3.0 * (1.0 + half_turn));
    let (a, b) = (from + from_way * handle, to - to_way * handle);
    let steps = ((chord + handle) / SPACING).ceil().max(2.0) as usize;
    (1..steps)
        .map(|step| {
            let t = step as f32 / steps as f32;
            let u = 1.0 - t;
            from * (u * u * u) + a * (3.0 * u * u * t) + b * (3.0 * u * t * t) + to * (t * t * t)
        })
        .collect()
}

/// The way the car drives round the town: one loop through every lane, its
/// points [`SPACING`] apart, and when it passes each.
#[derive(Resource, Clone, Debug)]
pub(crate) struct Route {
    /// Where the car is at each point: the middle of its lane, on the road.
    points: Vec<Vec3>,
    /// Which way it is going there, across the ground.
    ways: Vec<Vec2>,
    /// When it passes each, in seconds into the loop, and, last, the whole
    /// loop, back at the first.
    times: Vec<f64>,
    /// How far along the loop each is, in metres, and, last, the whole way
    /// round.
    distances: Vec<f64>,
    /// The lanes driven, in order, as their stretch and which way along it.
    driven: Vec<(usize, bool)>,
}

impl Route {
    /// The car's way round the roads in `paint`, if they make one.
    pub(crate) fn read(paint: &Paint) -> Option<Self> {
        let network = network(paint);
        let lanes = lanes(&network);
        let order = plan(&lanes, network.joins.len())?;
        Some(Self::through(&lanes, &order, paint.road))
    }

    /// The loop through `lanes` in `order`, turning from each into the next,
    /// on the road `road`.
    fn through(lanes: &[Lane], order: &[usize], road: &Island) -> Self {
        let mut line: Vec<Vec2> = Vec::new();
        let mut tops: Vec<f32> = Vec::new();
        for (index, &lane) in order.iter().enumerate() {
            let this = &lanes[lane].points;
            let next = &lanes[order[(index + 1) % order.len()]].points;
            line.extend_from_slice(this);
            let end_way = (this[this.len() - 1] - this[this.len() - 2]).normalize_or_zero();
            let start_way = (next[1] - next[0]).normalize_or_zero();
            line.extend(turn(this[this.len() - 1], end_way, next[0], start_way));
            tops.resize(line.len(), lanes[lane].top);
        }
        // Evenly spaced round it, each on the road under it.
        let so_far = along(&line, true);
        let whole = *so_far.last().unwrap();
        let count = (whole / SPACING).round().max(3.0) as usize;
        let mut points: Vec<Vec3> = Vec::with_capacity(count);
        for step in 0..count {
            let s = step as f32 * whole / count as f32;
            let at = point_at(&line, &so_far, s);
            let near = so_far.partition_point(|&d| d <= s).saturating_sub(1).min(tops.len() - 1);
            let top = tops[near];
            let last = points.last().map_or(top, |point| point.y);
            let ground = road.floor(at, top + OVER_THE_ROAD).unwrap_or(last);
            points.push(Vec3::new(at.x, ground, at.y));
        }
        let ways: Vec<Vec2> = (0..count)
            .map(|at| {
                let (before, after) = (points[(at + count - 1) % count], points[(at + 1) % count]);
                (after - before).xz().normalize_or(Vec2::X)
            })
            .collect();
        let mut route = Self {
            points,
            ways,
            times: Vec::new(),
            distances: Vec::new(),
            driven: order.iter().map(|&lane| (lanes[lane].stretch, lanes[lane].ahead)).collect(),
        };
        route.drive();
        route
    }

    /// Works out when the car passes each point: as fast as it can take how
    /// sharply the road turns there, never faster than its top speed, and
    /// speeding up and slowing down gently between. Then a little faster or
    /// slower all the way round, for the loop to take a whole number of
    /// seconds.
    fn drive(&mut self) {
        let count = self.points.len();
        let gaps: Vec<f32> = (0..count)
            .map(|at| self.points[at].xz().distance(self.points[(at + 1) % count].xz()))
            .collect();
        // How sharply it turns at each point: the turn in its way, over the
        // distance it turns in.
        let sharp: Vec<f32> = (0..count)
            .map(|at| {
                let before = (at + count - 1) % count;
                let into = (self.points[at] - self.points[before]).xz().normalize_or_zero();
                let out = (self.points[(at + 1) % count] - self.points[at]).xz().normalize_or_zero();
                into.perp_dot(out).abs() / ((gaps[at] + gaps[before]) * 0.5).max(1e-3)
            })
            .collect();
        let mut speeds: Vec<f32> = (0..count)
            .map(|at| {
                let sharpness = (0..=2 * SMOOTHING)
                    .map(|k| sharp[(at + count + k - SMOOTHING) % count])
                    .sum::<f32>()
                    / (2 * SMOOTHING + 1) as f32;
                let round = if sharpness > 1e-6 { (GRIP / sharpness).sqrt() } else { f32::INFINITY };
                round.min(roundtown_net::cars::TOP_SPEED).max(SLOWEST)
            })
            .collect();
        // Twice round each way, so that slowing down for a turn just past
        // the first point and speeding up out of one just before the last
        // are both taken in.
        for _ in 0..2 {
            for at in 0..count {
                let before = (at + count - 1) % count;
                let most = (speeds[before] * speeds[before] + 2.0 * SPEED_UP * gaps[before]).sqrt();
                speeds[at] = speeds[at].min(most);
            }
            for at in (0..count).rev() {
                let after = (at + 1) % count;
                let most = (speeds[after] * speeds[after] + 2.0 * SLOW_DOWN * gaps[at]).sqrt();
                speeds[at] = speeds[at].min(most);
            }
        }
        let (mut times, mut distances) = (Vec::with_capacity(count + 1), Vec::with_capacity(count + 1));
        let (mut time, mut distance) = (0.0f64, 0.0f64);
        for at in 0..count {
            times.push(time);
            distances.push(distance);
            let pace = f64::from((speeds[at] + speeds[(at + 1) % count]) * 0.5);
            time += f64::from(gaps[at]) / pace;
            distance += f64::from(gaps[at]);
        }
        let lap = time.round().max(1.0);
        for when in &mut times {
            *when *= lap / time;
        }
        times.push(lap);
        distances.push(distance);
        self.times = times;
        self.distances = distances;
    }

    /// How long the loop takes, in seconds: a whole number of them.
    pub(crate) fn lap(&self) -> f64 {
        *self.times.last().unwrap()
    }

    /// How far it is round, in metres.
    pub(crate) fn length(&self) -> f64 {
        *self.distances.last().unwrap()
    }

    /// How many lanes it drives, counting one driven twice twice.
    pub(crate) fn lanes(&self) -> usize {
        self.driven.len()
    }

    /// Where the car is `seconds` into its loop, gone round again as often as
    /// it takes: on the road, which way it is going, and how far it has come
    /// in all that time, every time round.
    pub(crate) fn at(&self, seconds: f64) -> (Vec3, Vec2, f64) {
        let laps = seconds.div_euclid(self.lap());
        let into = seconds.rem_euclid(self.lap());
        let count = self.points.len();
        let after = self.times.partition_point(|&when| when <= into).clamp(1, count);
        let before = after - 1;
        let span = self.times[after] - self.times[before];
        let share = if span > 0.0 { ((into - self.times[before]) / span).clamp(0.0, 1.0) } else { 0.0 };
        let way = self.ways[before]
            .lerp(self.ways[after % count], share as f32)
            .normalize_or(self.ways[before]);
        let distance = self.distances[before] + (self.distances[after] - self.distances[before]) * share;
        let distance = laps * self.length() + distance;
        (self.points[before].lerp(self.points[after % count], share as f32), way, distance)
    }

    /// The fastest the car goes anywhere round the loop, in metres a second.
    #[cfg(test)]
    fn fastest(&self) -> f32 {
        (0..self.points.len())
            .map(|at| ((self.distances[at + 1] - self.distances[at]) / (self.times[at + 1] - self.times[at])) as f32)
            .fold(0.0, f32::max)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::f32::consts::{FRAC_PI_2, TAU};

    use super::*;

    /// A flat ring of ground from `inner` to `outer` out from the middle,
    /// `top` high, with a bottom at `bottom`, in `sides` pieces round.
    fn ring(inner: f32, outer: f32, bottom: f32, top: f32, sides: usize) -> Vec<[Vec3; 3]> {
        let corner = |r: f32, angle: f32, y: f32| Vec3::new(r * angle.cos(), y, r * angle.sin());
        let mut triangles = Vec::new();
        for side in 0..sides {
            let (a, b) = (side as f32 / sides as f32 * TAU, (side + 1) as f32 / sides as f32 * TAU);
            let up = [corner(inner, a, top), corner(inner, b, top), corner(outer, b, top), corner(outer, a, top)];
            let down = [
                corner(inner, a, bottom),
                corner(outer, a, bottom),
                corner(outer, b, bottom),
                corner(inner, b, bottom),
            ];
            for quad in [up, down] {
                triangles.push([quad[0], quad[1], quad[2]]);
                triangles.push([quad[0], quad[2], quad[3]]);
            }
        }
        triangles
    }

    /// The paving between two straight roads `half` wide either side of the
    /// x and z axes, from `inner` to `outer` out from the middle, in the
    /// quarter where both are over `half`: its top at `top`.
    fn quarter(inner: f32, outer: f32, half: f32, top: f32) -> Vec<[Vec3; 3]> {
        let arc = |r: f32, share: f32| {
            let (from, to) = ((half / r).asin(), (half / r).acos());
            let angle = from + (to - from) * share;
            (r * angle.cos(), r * angle.sin())
        };
        let steps = 96;
        let mut triangles = Vec::new();
        for step in 0..steps {
            let (s, t) = (step as f32 / steps as f32, (step + 1) as f32 / steps as f32);
            let corners = [arc(inner, s), arc(inner, t), arc(outer, t), arc(outer, s)];
            for y in [top, top - 0.3] {
                let p = |k: usize| Vec3::new(corners[k].0, y, corners[k].1);
                if y == top {
                    triangles.push([p(0), p(1), p(2)]);
                    triangles.push([p(0), p(2), p(3)]);
                } else {
                    triangles.push([p(0), p(2), p(1)]);
                    triangles.push([p(0), p(3), p(2)]);
                }
            }
        }
        triangles
    }

    /// A dash, `length` long and 0.4 m wide, its middle at `at`, lying along
    /// `way`, a centimetre over the road at `y`.
    fn dash(at: Vec2, way: Vec2, length: f32, y: f32) -> Vec<[Vec3; 3]> {
        let turn = Quat::from_rotation_arc(Vec3::X, Vec3::new(way.x, 0.0, way.y).normalize());
        let half = Vec3::new(length * 0.5, 0.005, 0.2);
        let middle = Vec3::new(at.x, y + 0.005, at.y);
        crate::camera_tests::boxes(&[(-half, half)])
            .into_iter()
            .map(|corners| corners.map(|corner| middle + turn * corner))
            .collect()
    }

    /// A town laid out as Round Town is: its inner road a ring from 13 to 23
    /// m out, its middle 18 m out, and its outer road from 52 to 62; four
    /// straight roads, 10 m wide, from one to the other; paving 0.2 m higher
    /// in between and round both; and every road's middle dashed, as Hajun's
    /// is. Its land, its roads alone, and its paint, a mesh to each road.
    pub(crate) fn town() -> (Vec<[Vec3; 3]>, Vec<[Vec3; 3]>, Vec<Vec<[Vec3; 3]>>) {
        let road_y = -0.2;
        // The roads go on under the paving, as Hajun's do: the inner road a
        // disc to 23 m and the outer one to 62.
        let mut road = ring(0.0, 23.0, -0.5, road_y, 64);
        road.extend(ring(0.0, 62.0, -0.5, road_y, 128));
        for k in 0..4 {
            let turn = Quat::from_rotation_y(k as f32 * FRAC_PI_2);
            for corners in crate::camera_tests::boxes(&[(Vec3::new(22.0, -0.5, -5.0), Vec3::new(52.0, road_y, 5.0))]) {
                road.push(corners.map(|corner| turn * corner));
            }
        }
        let mut land = road.clone();
        land.extend(ring(0.0, 10.0, -0.3, 0.0, 64));
        land.extend(ring(10.0, 13.0, -0.3, -0.1, 64));
        land.extend(ring(62.0, 70.0, -2.0, 0.0, 128));
        for k in 0..4 {
            let turn = Quat::from_rotation_y(k as f32 * FRAC_PI_2);
            for corners in quarter(23.0, 52.0, 5.0, 0.0) {
                land.push(corners.map(|corner| turn * corner));
            }
        }
        let mut inner = Vec::new();
        for k in 0..32 {
            let at = Vec2::from_angle(k as f32 / 32.0 * TAU) * 18.0;
            inner.extend(dash(at, at.perp().normalize(), 2.0, road_y));
        }
        let mut outer = Vec::new();
        for k in 0..96 {
            let at = Vec2::from_angle(k as f32 / 96.0 * TAU) * 57.0;
            outer.extend(dash(at, at.perp().normalize(), 2.0, road_y));
        }
        let mut cross = Vec::new();
        for k in 0..4 {
            let way = Vec2::from_angle(k as f32 * FRAC_PI_2);
            for d in 0..12 {
                cross.extend(dash(way * (19.2 + d as f32 * 2.919), way, 2.0, road_y));
            }
        }
        land.extend(inner.iter().chain(&outer).chain(&cross).copied());
        (land, road, vec![inner, outer, cross])
    }

    /// The town's paint, its land and its roads, ready to read.
    fn read_town() -> (Island, Island, Vec<Vec<[Vec3; 3]>>) {
        let (land, road, markings) = town();
        (Island::new_for_tests(&land), Island::new_for_tests(&road), markings)
    }

    #[test]
    fn a_dash_is_where_its_middle_is_and_lies_along_its_length() {
        let way = Vec2::new(0.6, 0.8);
        let found = dashes(&[dash(Vec2::new(3.0, -2.0), way, 2.0, -0.2)]);
        assert_eq!(found.len(), 1);
        assert!(found[0].at.distance(Vec2::new(3.0, -2.0)) < 1e-4, "{}", found[0].at);
        assert!(found[0].along.dot(way).abs() > 0.9999, "{}", found[0].along);
        assert!((found[0].length - 2.0).abs() < 1e-3);
        // Two dashes in one mesh are two dashes; a square is none.
        let mut two = dash(Vec2::ZERO, Vec2::X, 2.0, 0.0);
        two.extend(dash(Vec2::new(4.0, 0.0), Vec2::X, 2.0, 0.0));
        assert_eq!(dashes(&[two]).len(), 2);
        let square = crate::camera_tests::boxes(&[(Vec3::ZERO, Vec3::new(1.0, 0.01, 1.0))]);
        assert!(dashes(&[square]).is_empty());
    }

    #[test]
    fn the_dashes_make_the_middles_of_the_roads() {
        let (_, _, markings) = town();
        let middles = middles(&dashes(&markings));
        let rings: Vec<&Middle> = middles.iter().filter(|middle| middle.closed).collect();
        let straight: Vec<&Middle> = middles.iter().filter(|middle| !middle.closed).collect();
        assert_eq!((rings.len(), straight.len()), (2, 4), "{} middles", middles.len());
        for ring in rings {
            let radius = ring.dashes[0].length();
            for point in &ring.points {
                assert!((point.length() - radius).abs() < 0.05, "{point} off the ring {radius}");
            }
        }
        for road in straight {
            assert_eq!(road.dashes.len(), 12);
        }
    }

    #[test]
    fn round_town_is_twelve_stretches_between_eight_joins() {
        let (land, road, markings) = read_town();
        let network = network(&Paint { land: &land, road: &road, markings: &markings });
        assert_eq!(network.joins.len(), 8, "{:?}", network.joins);
        assert_eq!(network.stretches.len(), 12);
        for stretch in &network.stretches {
            // Every road 10 m wide, its middle down the middle of it.
            assert!((stretch.left - 5.0).abs() < 0.1, "left {}", stretch.left);
            assert!((stretch.right - 5.0).abs() < 0.1, "right {}", stretch.right);
        }
        // Where the straight roads meet the middles of the rings.
        for join in &network.joins {
            let out = join.length();
            assert!((out - 18.0).abs() < 0.1 || (out - 57.0).abs() < 0.1, "{join}");
        }
    }

    #[test]
    fn each_lane_keeps_to_the_right_halfway_out_to_the_edge() {
        let (land, road, markings) = read_town();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        for lane in lanes(&network(&paint)) {
            let n = lane.points.len();
            let (a, b) = (lane.points[n / 2], lane.points[n / 2 + 1]);
            let right = (b - a).normalize().perp();
            let out = a.length();
            let on = |r: f32| (out - r).abs() < 0.2;
            if on(15.5) || on(20.5) || on(54.5) || on(59.5) {
                // Round a ring: on its outside, going round the way that has
                // the outside on the right.
                let outside = right.dot(a.normalize()) > 0.0;
                assert_eq!(outside, on(20.5) || on(59.5), "at {a}");
            } else {
                // Down a straight road, 2.5 m to the right of its middle.
                let off = a.x.abs().min(a.y.abs());
                assert!((off - 2.5).abs() < 0.2, "at {a}");
                let middle = if a.x.abs() < a.y.abs() { Vec2::new(0.0, a.y) } else { Vec2::new(a.x, 0.0) };
                assert!(right.dot(a - middle) > 0.0, "{a} left of the middle");
            }
            for point in &lane.points {
                assert!(paint.on_road(*point, -0.19), "{point} off the road");
            }
        }
    }

    #[test]
    fn the_car_drives_every_lane_of_every_road_both_ways() {
        let (land, road, markings) = read_town();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        let route = Route::read(&paint).expect("a way round");
        for stretch in 0..12 {
            for ahead in [true, false] {
                assert!(route.driven.contains(&(stretch, ahead)), "stretch {stretch}, {ahead}");
            }
        }
        // Only T-junctions: one stretch twice, each way.
        assert_eq!(route.driven.len(), 26);
        // A loop with no gaps in it, on the road all the way, and never a
        // turn back.
        let count = route.points.len();
        for at in 0..count {
            let (a, b) = (route.points[at], route.points[(at + 1) % count]);
            assert!(a.distance(b) < SPACING * 1.2, "{a} to {b}");
            assert!((a.y + 0.2).abs() < 0.02, "{a} not on the road");
            assert!(paint.on_road(a.xz(), -0.19), "{a} off the road");
            assert!(route.ways[at].dot(route.ways[(at + 1) % count]) > 0.9, "turned back at {a}");
        }
        // A whole number of seconds round, never faster than the top speed.
        assert_eq!(route.lap().fract(), 0.0);
        assert!(route.fastest() <= roundtown_net::cars::TOP_SPEED * 1.01, "{}", route.fastest());
        // Where it is, by the time, round and round.
        let (start, ..) = route.at(0.0);
        let (again, ..) = route.at(route.lap() * 3.0);
        assert!(start.distance(again) < 1e-3);
        let (half, ..) = route.at(route.lap() * 0.5);
        assert!(half.distance(start) > 1.0);
    }

    /// Round Town as `circlemap1.glb` has it: its land, as the game reads it
    /// (`island::read_island`), its roads alone, and its paint.
    pub(crate) fn round_town() -> (Island, Island, Vec<Vec<[Vec3; 3]>>) {
        let parts = crate::island::glb::read("circlemap1.glb");
        let made_of = |material: &str| -> Vec<Vec<[Vec3; 3]>> {
            parts
                .iter()
                .filter(|part| part.material.as_deref() == Some(material))
                .map(|part| part.triangles.clone())
                .collect()
        };
        let land: Vec<[Vec3; 3]> = parts
            .iter()
            .filter(|part| !part.skinned && part.material.as_deref() != Some(crate::island::WATER_MATERIAL))
            .flat_map(|part| part.triangles.iter().copied())
            .collect();
        let road: Vec<[Vec3; 3]> = made_of(ROAD).into_iter().flatten().collect();
        (Island::new_for_tests(&land), Island::new_for_tests(&road), made_of(MARKING))
    }

    #[test]
    fn round_town_s_own_roads_are_driven_every_lane_both_ways() {
        let (land, road, markings) = round_town();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        let network = network(&paint);
        assert_eq!(network.joins.len(), 8, "{:?}", network.joins);
        assert_eq!(network.stretches.len(), 12);
        for stretch in &network.stretches {
            assert!((stretch.left - 5.0).abs() < 0.2, "left {}", stretch.left);
            assert!((stretch.right - 5.0).abs() < 0.2, "right {}", stretch.right);
        }
        let route = Route::read(&paint).expect("a way round the town");
        for stretch in 0..12 {
            for ahead in [true, false] {
                assert!(route.driven.contains(&(stretch, ahead)), "stretch {stretch}, {ahead}");
            }
        }
        for point in &route.points {
            assert!(paint.on_road(point.xz(), -0.19), "{point} off the road");
            assert!((point.y + 0.2).abs() < 0.02, "{point} not on the road");
        }
        assert!(route.fastest() <= roundtown_net::cars::TOP_SPEED * 1.01);
        println!(
            "round town: {} m round in {} s, {} lanes",
            route.length().round(),
            route.lap(),
            route.driven.len()
        );
    }

    #[test]
    fn a_turn_leaves_one_lane_and_joins_the_next_along_them() {
        let points = turn(Vec2::new(0.0, 8.0), Vec2::NEG_Y, Vec2::new(8.0, 0.0), Vec2::X);
        let (first, last) = (points[0], *points.last().unwrap());
        assert!((first - Vec2::new(0.0, 8.0)).normalize().dot(Vec2::NEG_Y) > 0.99);
        assert!((Vec2::new(8.0, 0.0) - last).normalize().dot(Vec2::X) > 0.99);
        // Near a quarter circle round (8, 8).
        for point in &points {
            let out = point.distance(Vec2::new(8.0, 8.0));
            assert!((out - 8.0).abs() < 0.3, "{point}, {out} out");
        }
    }

    #[test]
    fn with_only_t_junctions_no_loop_drives_each_lane_once() {
        // Round Town's shape: a ring of four inside a ring of four, joined
        // across, every join three ways.
        let mut stretches = Vec::new();
        for k in 0..4 {
            stretches.push((k, (k + 1) % 4));
            stretches.push((4 + k, 4 + (k + 1) % 4));
            stretches.push((k, 4 + k));
        }
        let lanes: Vec<Lane> = stretches
            .iter()
            .enumerate()
            .flat_map(|(stretch, &(a, b))| {
                [(a, b, true), (b, a, false)].map(|(from, to, ahead)| Lane {
                    points: vec![Vec2::ZERO, Vec2::X],
                    from,
                    to,
                    stretch,
                    ahead,
                    top: 0.0,
                })
            })
            .collect();
        let every: Vec<usize> = (0..lanes.len()).collect();
        assert!(circuit(&lanes, &every, 8).is_none());
        let order = plan(&lanes, 8).expect("a loop with one stretch twice");
        assert_eq!(order.len(), 26);
        for (index, &lane) in order.iter().enumerate() {
            let (a, b) = (&lanes[lane], &lanes[order[(index + 1) % order.len()]]);
            assert_eq!(a.to, b.from);
            assert!(!(a.stretch == b.stretch && a.ahead != b.ahead), "turned back");
        }
    }
}
