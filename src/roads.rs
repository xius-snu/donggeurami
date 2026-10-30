//! The town's roads, read off its model, and the way the car drives them
//! (`car`).
//!
//! A road is wherever the top of the town is the material [`ROAD`], with a
//! dashed line down its middle in [`MARKING`]. Nothing here knows where any
//! road is. Once the town has spawned, its dashes are read out of it
//! ([`dashes`]), and every dash in line with the next makes a road's middle,
//! round on itself or open at both ends, drawn through them once the gaps
//! between them have been filled and they have been evened out ([`middles`],
//! [`bridge`], [`even_out`]); how far the road reaches either side of it,
//! measured off the model, is how wide it is ([`widths`]). A middle that runs
//! out with its last dash runs on for as long as there is road, across any
//! road it meets to the furthest it reaches, and there the two join; ends
//! that run off the road beside each other round an island in it meet round
//! the island, a roundabout ([`roundabout`]). Where roads join, or cross,
//! each is cut, and what is left between two joins is one stretch of road;
//! joins a few metres apart are one place ([`network`]). Reshape the roads in
//! Blender, paint their middles, and the car drives the new ones.
//!
//! The cars keep to the right, as Korea drives, so every stretch has two
//! lanes, one each way, each halfway out from the middle to the edge on its
//! side ([`lanes`]). Every turn from lane to lane at every join is worked
//! out first, as the car would go round it, its front wheels on the line and
//! its back ones following as a car's do ([`trailing`], [`turning`]): round
//! a corner as round as the road allows, swinging wide where it is tight, and
//! round a roundabout's island keeping it on the left. The cars drive every
//! lane they can get onto and off by turns they take gently, from and to
//! lanes they can, in one loop through the town ([`plan`]), never back the
//! way they came but where a road ends. A loop that drives each lane exactly
//! once is not always to be had: with only T-junctions, as the old town had,
//! it never is, as every way of taking the turns leaves the lanes in an even
//! number of loops. Then one stretch is driven twice, each way, the shortest
//! that makes it one loop.
//!
//! How fast a car goes along the loop is worked out from how sharply it
//! turns ([`Route::drive`]), and how long the loop takes is a whole number of
//! seconds, so that every device, whatever its arithmetic, puts each car in
//! the same place at the same time (`car`). The cars go round it one behind
//! the other, each an even share of it behind the one before, near enough
//! never to meet where it crosses itself ([`Route::spread`]).

use std::f32::consts::TAU;

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
/// How many times as long as it is wide a mark has to be to be a dash, and
/// how wide it can be at the most, in metres: a line, 0.4 m wide in the old
/// town and 0.53 in the new, not a stripe of a crossing, 1.6 m wide and as
/// long as a dash is, whose stripes lie along the road across it, two of
/// them on the very middle of the ring round the new town's pond.
const SLENDER: f32 = 1.5;
const WIDEST_DASH: f32 = 1.0;
/// How far apart, middle to middle, two dashes of one road can be, as many
/// times as long as the longer is: the old town's were 1.5 to 1.9 times, the
/// new town's 2.7 to 2.8, and either side of a crossing on the ring round
/// its pond, where the dashes stop for the crossing's stripes, up to 7.8.
/// Not as far as across a roundabout's island, 10 times.
const GAP: f32 = 9.0;
/// How nearly in line two dashes have to be to be one road's: lying within
/// 30° of each other, and each within 30° of the way to the other.
const IN_LINE: f32 = 0.866;
/// The fewest dashes that make a road.
const FEWEST_DASHES: usize = 3;
/// How far each dash's middle is moved toward the middle of the two either
/// side of it, and then back away from theirs, in evening a road's middle
/// out ([`even_out`]): halfway in takes out all that goes from one side to
/// the other and back from dash to dash, and a little more than that back
/// out puts back what it took off a bend (Taubin's smoothing).
const SHRINK: f32 = 0.5;
const SWELL: f32 = -0.53;
/// How many points a road's middle is drawn through between two dashes.
const BETWEEN_DASHES: usize = 8;
/// How far past its last dash a road's middle is followed to the middle of
/// the road it runs into, in metres, at the least: the old town's straight
/// roads' last dashes were 1.2 m short of the inner road's middle and 5.7 m
/// short of the outer road's. Further, for as long as the road goes on, up
/// to [`ACROSS`]: Hajun's new town's straight roads run 8 m to the middle of
/// the outer half of the ring round its pond and 23 m on, across it and the
/// gap in the strip down its middle, to the inner half's.
const REACH: f32 = 12.0;
const ACROSS: f32 = 40.0;
/// How near beside each other roads have to run off the road, in metres, to
/// run off it round one island, a roundabout ([`roundabout`]); how many ways
/// round its middle the island's edge and the road round it are felt for,
/// and how far out; and how big an island a roundabout's is, across from its
/// middle, in metres. The new town's are 9 m, with road 8 m wide round them
/// (6 m until Hajun made them bigger, 2026-10-30).
const ISLAND_GROUP: f32 = 30.0;
/// How near joins are to be one place, in metres ([`network`]): the new
/// town's ring's two halves meet the straight roads 15 m apart.
const ONE_PLACE: f32 = 20.0;
/// How far a car's wheels may run up onto the edge of a roundabout's island,
/// in metres, going round it: round the new town's while they were 6 m wide
/// for a bus 3.7 m wide, as a bus's do round a small roundabout. Round them
/// at 8 m the bus's keep off it.
const APRON: f32 = 1.0;
const ISLAND_WAYS: usize = 64;
const ISLAND_REACH: f32 = 30.0;
const SMALLEST_ISLAND: f32 = 2.0;
const LARGEST_ISLAND: f32 = 20.0;
/// How near two joins can be and be one, in metres.
const JOIN: f32 = 2.0;
/// How finely a road's width is felt for, in metres, before the edge is
/// found more finely between the last step on it and the first off it.
const STEP: f32 = 0.25;
/// The furthest out from its middle a road's edge is looked for, in metres.
const WIDEST: f32 = 12.0;
/// How far over a road's surface the top of the town can be and still be the
/// road, in metres: its paint, a centimetre thick in the old town and two in
/// the new, but not a kerb, which is a quarter of a metre at the least.
const PAINT: f32 = 0.03;
/// How far over a road's surface its edge is looked for, in metres: higher
/// than a kerb, and lower than the arm of a lamp over the road.
const OVER_THE_ROAD: f32 = 1.0;
/// How much further back than the edge of the road it meets a lane stops
/// short of a join, to turn from, in metres.
const ROOM_TO_TURN: f32 = 3.0;
/// The most of a lane that is cut off its two ends together to turn from.
const MOST_CUT: f32 = 0.7;
/// How much nearer a join than the room left at it a lane is followed before
/// it is turned from, a step at a time, in metres, where turning from the room
/// left would take the car off the road; and the nearest a turn leaves a
/// lane or joins one to the join.
const NEARER: f32 = 0.5;
const NEAREST: f32 = 1.0;
/// How far inside the road's edge the car keeps all of itself, turning, in
/// metres.
const KERB: f32 = 0.25;
/// How high over the road a car's underside is, in metres, and how far up
/// from there what it swings out over is looked for: over a kerb or paving,
/// a quarter of a metre high, the bus swings out clear; over the strip of
/// grass down the middle of the ring round the new town's pond, half a metre
/// high, it would swing into it.
const UNDERSIDE: f32 = 0.4;
const OVERHEAD: f32 = 3.6;
/// How far along a lane a car turning wide takes to move across between it
/// and the middle of the road, in metres ([`turning`]), and the most of a
/// lane that moving across and turning take, at either end of it.
const CROSS_OVER: f32 = 8.0;
const MOST_WIDE: f32 = 0.5;
/// The tightest a car's front wheels turn, as the radius they turn round
/// over how far apart its front and back wheels are: 8.5 m for the bus
/// ([`turning`]). Its back wheels follow its front ones as a car's do
/// ([`trailing`]), and turned tighter than its wheels are apart, they would
/// swing round about each other rather than roll. From 2026-10-24 to
/// 2026-10-30 both pairs of wheels kept to the line, and this was 0.7, at
/// which the bus slid up to 17° sideways round the old town's corners.
const TIGHTEST: f32 = 1.3;
/// How much further back from a join than the room left a turn is tried
/// from, and joins the next lane, in metres ([`corner_shapes`]).
const FURTHER: [f32; 4] = [3.0, 6.0, 10.0, 15.0];
/// How far off a lane, in metres, an arc round a corner can start or end
/// ([`fillets`]): no further than the car would be seen to jump.
const ON_THE_LINE: f32 = 0.05;
/// How gently a turn has to bend, at its tightest, as the radius over how far
/// apart the car's wheels are, for the car to take it as it comes, the first
/// that keeps it on the road: 10.4 m for the bus. At a tighter corner it
/// takes the gentlest that does, swinging wide into it as a bus does
/// ([`turning`]).
const RELAXED: f32 = 1.6;
/// How round the corners turned at an ordinary join are tried, as the radius
/// the front wheels turn round over how far apart the car's wheels are, the
/// roundest first ([`fillets`]): 19.6 m down to 8.5 m for the bus.
const ROUNDS: [f32; 7] = [3.0, 2.5, 2.0, 1.75, 1.5, 1.4, 1.3];
/// How far along the lane before a turn its back wheels are followed from,
/// and along the lane after it, as many times as far as its wheels are
/// apart: where its back wheels are, coming into a turn and coming out of
/// it, follows from where its front ones were ([`trailing`]).
const LEAD_IN: f32 = 3.0;
const LEAD_OUT: f32 = 2.0;
/// How round the arcs a car turns onto and off the road round a roundabout's
/// island by are, as the radius over how far apart its wheels are
/// ([`round_shapes`]).
const MERGES: [f32; 4] = [2.0, 1.6, 1.3, 1.1];
/// How far out from the island a car goes round it, as a share of the way to
/// the outside of the road round it: round the new town's, 8 m wide, the
/// bus's back wheels keep inside its front ones by 1.6 m, and only from a
/// little under halfway out are they clear of the island.
const ROUND_AT: [f32; 5] = [0.65, 0.6, 0.7, 0.55, 0.75];
/// The most turns weighed, gentlest first, where none keeps the car on the
/// road, for the one that takes it off least.
const ROUND_TRIES: usize = 200;
/// How far apart the points round the car are that are felt for the road
/// under them, in metres.
const FEEL: f32 = 0.25;
/// How near the cars come to each other at the least, in metres, where the
/// loop crosses itself and passing each other, where that can be had
/// ([`Route::spread`]), and how often, in seconds, how near they come is
/// looked at round it.
const APART: f32 = 1.0;
const LOOK_EVERY: f64 = 0.2;
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
/// The slowest it goes, in metres a second, but where it stops.
const SLOWEST: f32 = 2.0;
/// How many times its top speed a car goes along an open road, one from a
/// roundabout to the next: round the edge of Hajun's new town, 14 m/s for a
/// bus (Hajun, 2026-10-30: "make the bus go faster when going around the
/// outer circle (like 2x)"). The server lets a rider off a car as far from
/// where they got on as 18.2 m/s takes them (`MOST_RIDE` in
/// `server/src/players.rs`), which this is under.
pub(crate) const OPEN_ROAD: f32 = 2.0;
/// How far short of a crossing painted across the road a car stops, the
/// front of it, in metres, and how long it waits there before it goes on, in
/// seconds: at every other crossing it comes to, as if to let someone across
/// (Hajun, 2026-10-30: "make it pause before some of the crosswalks and then
/// go"). Hajun's new town's crossings are on the ring round its pond.
const STOP_SHORT: f32 = 1.5;
const PAUSE: f64 = 3.0;
/// How near the middles of two stripes side by side are, at the most, in
/// metres, for them to be one crossing's: Hajun's are 2.1 m apart. And the
/// fewest stripes that make a crossing.
const STRIPES_APART: f32 = 3.0;
const FEWEST_STRIPES: usize = 3;
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
        for (corners, _) in pieces(mesh) {
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
            if length < width.max(1e-3) * SLENDER || width > WIDEST_DASH {
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
/// triangle, which every device reads alike; and how much of the ground it
/// covers, in square metres.
fn pieces(triangles: &[[Vec3; 3]]) -> Vec<(Vec<Vec3>, f32)> {
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
    let mut pieces: Vec<(Vec<Vec3>, std::collections::BTreeSet<(i32, i32, i32)>, f32)> = Vec::new();
    for (index, triangle) in triangles.iter().enumerate() {
        let top = root(&mut parent, index);
        let piece = *first_of.entry(top).or_insert_with(|| {
            pieces.push(Default::default());
            pieces.len() - 1
        });
        let (corners, seen, area) = &mut pieces[piece];
        for &corner in triangle {
            if seen.insert(key(corner)) {
                corners.push(corner);
            }
        }
        *area += (triangle[1] - triangle[0]).xz().perp_dot((triangle[2] - triangle[0]).xz()).abs() * 0.5;
    }
    pieces.into_iter().map(|(corners, _, area)| (corners, area)).collect()
}

/// A crossing painted across a road, as the box round its stripes across the
/// ground: its middle, which way its stripes lie, and how far it goes each
/// way from its middle, along them and across them.
#[derive(Clone, Copy, Debug)]
struct Crossing {
    middle: Vec2,
    along: Vec2,
    half: Vec2,
}

impl Crossing {
    /// Whether `at` is on it.
    fn holds(&self, at: Vec2) -> bool {
        let off = at - self.middle;
        off.dot(self.along).abs() <= self.half.x && off.dot(self.along.perp()).abs() <= self.half.y
    }
}

/// The crossings painted across the roads in `markings`: their stripes are
/// every piece of paint that is no line but a patch, as wide all along as no
/// line is ([`WIDEST_DASH`]), each with every other lying the same way whose
/// middle is within [`STRIPES_APART`] of it. Hajun's new town has one at each
/// corner of the ring round its pond, across both its halves: 12 stripes, each
/// 8 m long and 1.2 m wide, lying along the road.
fn crossings_painted(markings: &[Vec<[Vec3; 3]>]) -> Vec<Crossing> {
    // Each stripe: its corners across the ground, its middle and the way it lies.
    let mut stripes: Vec<(Vec<Vec2>, Vec2, Vec2)> = Vec::new();
    for mesh in markings {
        for (corners, area) in pieces(mesh) {
            let middle = corners.iter().map(|corner| corner.xz()).sum::<Vec2>() / corners.len() as f32;
            let along = long_way(&corners, middle);
            let (back, front) = corners.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), corner| {
                let at = (corner.xz() - middle).dot(along);
                (low.min(at), high.max(at))
            });
            if front - back < 1e-3 || area / (front - back) <= WIDEST_DASH {
                continue;
            }
            stripes.push((corners.iter().map(|corner| corner.xz()).collect(), middle, along));
        }
    }
    let count = stripes.len();
    let mut crossing: Vec<usize> = (0..count).collect();
    fn root(crossing: &mut [usize], mut at: usize) -> usize {
        while crossing[at] != at {
            crossing[at] = crossing[crossing[at]];
            at = crossing[at];
        }
        at
    }
    for a in 0..count {
        for b in a + 1..count {
            let ((_, a_middle, a_along), (_, b_middle, b_along)) = (&stripes[a], &stripes[b]);
            if a_middle.distance(*b_middle) <= STRIPES_APART && a_along.dot(*b_along).abs() >= IN_LINE {
                let (ra, rb) = (root(&mut crossing, a), root(&mut crossing, b));
                crossing[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    // Each crossing's stripes' corners, the way the first of them lies, and how many.
    let mut found: Vec<(usize, Vec<Vec2>, Vec2, usize)> = Vec::new();
    for stripe in 0..count {
        let top = root(&mut crossing, stripe);
        match found.iter_mut().find(|(of, ..)| *of == top) {
            Some((_, corners, _, stripes_in_it)) => {
                corners.extend_from_slice(&stripes[stripe].0);
                *stripes_in_it += 1;
            }
            None => found.push((top, stripes[stripe].0.clone(), stripes[stripe].2, 1)),
        }
    }
    found
        .into_iter()
        .filter(|&(.., stripes_in_it)| stripes_in_it >= FEWEST_STRIPES)
        .map(|(_, corners, along, _)| {
            let across = along.perp();
            let (low, high) = corners.iter().fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(low, high), corner| {
                    let at = Vec2::new(corner.dot(along), corner.dot(across));
                    (low.min(at), high.max(at))
                },
            );
            let middle = (low + high) * 0.5;
            Crossing {
                middle: along * middle.x + across * middle.y,
                along,
                half: (high - low) * 0.5,
            }
        })
        .collect()
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
            let at = even_out(&bridge(&at, closed), closed);
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

/// The middles of a road's dashes, `points`, evened out. Hajun's do not all
/// lie on one smooth line: round the rings, every other dash is a few
/// centimetres in from the rest, 3.6 cm round the inner road and 11 round the
/// outer, and a line through every one of them wove in and out by as much,
/// the car on it turning out and in, out and in, all the way round. Each is
/// moved halfway to the middle of the two either side of it, then back away
/// from theirs by a little more ([`SHRINK`], [`SWELL`]): what went from side
/// to side between dashes is gone, a circle stays the circle it was, to a
/// centimetre, and a straight line stays straight. The ends of an open road
/// stay where they are.
fn even_out(points: &[Vec2], closed: bool) -> Vec<Vec2> {
    let n = points.len();
    let pass = |points: &[Vec2], by: f32| -> Vec<Vec2> {
        (0..n)
            .map(|at| {
                if n < 3 || !closed && (at == 0 || at == n - 1) {
                    return points[at];
                }
                let (before, after) = (points[(at + n - 1) % n], points[(at + 1) % n]);
                points[at] + ((before + after) * 0.5 - points[at]) * by
            })
            .collect()
    };
    pass(&pass(points, SHRINK), SWELL)
}

/// `points` with the gaps in them filled: where two are much further apart
/// than the rest, as the new town's ring round its pond's dashes are either
/// side of a crossing, as many points between them as fill it as evenly as
/// the rest, on the circle through them and the one before, and through
/// them and the one after, halfway between the two. A line drawn through
/// the dashes alone sagged 0.4 m in across each crossing, and the bus going
/// round it turned out and back in there.
fn bridge(points: &[Vec2], closed: bool) -> Vec<Vec2> {
    let n = points.len();
    if n < 4 {
        return points.to_vec();
    }
    let spans = if closed { n } else { n - 1 };
    let mut gaps: Vec<f32> = (0..spans).map(|at| points[at].distance(points[(at + 1) % n])).collect();
    gaps.sort_by(f32::total_cmp);
    let usual = gaps[gaps.len() / 2];
    let point = |at: isize| -> Option<Vec2> {
        if closed {
            Some(points[at.rem_euclid(n as isize) as usize])
        } else {
            (0..n as isize).contains(&at).then(|| points[at as usize])
        }
    };
    let mut out = Vec::with_capacity(n * 2);
    for at in 0..spans {
        let (a, b) = (points[at], points[(at + 1) % n]);
        out.push(a);
        let between = (a.distance(b) / usual).round() as usize;
        if between < 2 || a.distance(b) < usual * 1.6 {
            continue;
        }
        let (before, after) = (point(at as isize - 1), point(at as isize + 2));
        for k in 1..between {
            let share = k as f32 / between as f32;
            let on: Vec<Vec2> = [before, after]
                .into_iter()
                .flatten()
                .filter_map(|third| on_circle(third, a, b, share))
                .collect();
            out.push(if on.is_empty() { a.lerp(b, share) } else { on.iter().sum::<Vec2>() / on.len() as f32 });
        }
    }
    if !closed {
        out.push(points[n - 1]);
    }
    out
}

/// The point `share` of the way from `a` to `b` the short way round the
/// circle through them and `third`; `None` if the three lie in a line.
fn on_circle(third: Vec2, a: Vec2, b: Vec2, share: f32) -> Option<Vec2> {
    let (x, y, z) = (third, a, b);
    let d = 2.0 * (x.x * (y.y - z.y) + y.x * (z.y - x.y) + z.x * (x.y - y.y));
    if d.abs() < 1e-6 {
        return None;
    }
    let square = |v: Vec2| v.length_squared();
    let middle = Vec2::new(
        (square(x) * (y.y - z.y) + square(y) * (z.y - x.y) + square(z) * (x.y - y.y)) / d,
        (square(x) * (z.x - y.x) + square(y) * (x.x - z.x) + square(z) * (y.x - x.x)) / d,
    );
    let radius = middle.distance(a);
    if radius > 1e4 {
        return None;
    }
    let (from, to) = ((a - middle).to_angle(), (b - middle).to_angle());
    let mut turn = to - from;
    if turn > std::f32::consts::PI {
        turn -= TAU;
    } else if turn < -std::f32::consts::PI {
        turn += TAU;
    }
    Some(middle + Vec2::from_angle(from + turn * share) * radius)
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

/// The point `s` along `points`, whose distances along are `so_far`, and
/// which way they go there: from [`SPACING`] before it to as far after, so
/// that on a bend it is the way the bend goes at the point. The way of the
/// side of `points` it was on was the way the bend went half a side back, and
/// a turn that set off that way from a lane round a ring, or joined one so,
/// swung the car's front out at the start of it and back in.
fn heading_at(points: &[Vec2], so_far: &[f32], s: f32) -> (Vec2, Vec2) {
    let whole = *so_far.last().unwrap();
    let (back, on) = ((s - SPACING).max(0.0), (s + SPACING).min(whole));
    let way = (point_at(points, so_far, on) - point_at(points, so_far, back)).normalize_or_zero();
    (point_at(points, so_far, s), way)
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

/// The town's roads: where they join, and the stretches between, and which
/// joins are roundabouts.
#[derive(Clone, Debug, Default)]
struct Network {
    joins: Vec<Vec2>,
    stretches: Vec<Stretch>,
    /// A roundabout at each join that is one, by the join's number.
    rings: Vec<Option<Ring>>,
}

/// A roundabout: an island in the middle of where roads meet, which they
/// are driven round, keeping it on the left, as a roundabout is where cars
/// keep to the right. Its middle is the join's; `inside` is how far out from
/// it the island goes, and `outside` how far the road round it does.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Ring {
    inside: f32,
    outside: f32,
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
    // Where each open end runs on to along the road, found before any of
    // them is made longer: the middle of the furthest road it reaches while
    // it is still on the road, across one road to another where the road
    // goes on, as through a gap in a strip down the middle of a road
    // (Hajun's new town, 2026-10-30, whose ring round its pond is two roads
    // with a strip of grass between, open where the straight roads come in).
    // An end that runs off the road without reaching one, where it does.
    let mut ends: Vec<(usize, bool, usize, Vec2)> = Vec::new();
    let mut loose: Vec<(usize, bool, Vec2)> = Vec::new();
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
            let mut on = 0.0;
            while on < ACROSS && paint.on_road(tip + way * (on + STEP), middle.top) {
                on += STEP;
            }
            let reach = (on + STEP).max(REACH.min(on + STEP * 2.0));
            let met = middles
                .iter()
                .enumerate()
                .filter(|&(other, _)| other != line)
                .flat_map(|(other, road)| {
                    meetings(tip, way, reach, &road.points, road.closed)
                        .into_iter()
                        .map(move |(t, at)| (t, other, at))
                })
                .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
            match met {
                Some((_, other, at)) => ends.push((line, end, other, at)),
                None => loose.push((line, end, tip + way * (on + STEP))),
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
    let mut rings: Vec<Option<Ring>> = Vec::new();
    let mut cuts: Vec<Vec<(f32, usize)>> = vec![Vec::new(); middles.len()];
    for &(line, end, other, at) in &ends {
        let join = join_at(&mut joins, at);
        cuts[line].push((if end { f32::INFINITY } else { 0.0 }, join));
        let road = &middles[other];
        cuts[other].push((locate(&road.points, road.closed, at).0, join));
    }
    // Ends that run off the road beside each other, round an island in it,
    // join there: a roundabout. Each goes on to the middle of the island,
    // round the bend its road is on ([`on_to`]).
    let top = middles.iter().map(|middle| middle.top).fold(f32::NEG_INFINITY, f32::max);
    let mut taken = vec![false; loose.len()];
    for first in 0..loose.len() {
        if taken[first] {
            continue;
        }
        let mut group = vec![first];
        let mut at = 0;
        while at < group.len() {
            let here = loose[group[at]].2;
            for other in 0..loose.len() {
                if !taken[other] && !group.contains(&other) && loose[other].2.distance(here) <= ISLAND_GROUP {
                    group.push(other);
                }
            }
            at += 1;
        }
        if group.len() < 2 {
            continue;
        }
        let runs_off: Vec<Vec2> = group.iter().map(|&end| loose[end].2).collect();
        let Some((middle, ring)) = roundabout(paint, &runs_off, top) else {
            continue;
        };
        let join = joins.len();
        joins.push(middle);
        rings.resize(joins.len(), None);
        rings[join] = Some(ring);
        for &end in &group {
            taken[end] = true;
            let (line, end, _) = loose[end];
            let on = on_to(&middles[line], end, middle);
            let points = &mut middles[line].points;
            if end {
                points.extend(on);
            } else {
                points.splice(0..0, on.into_iter().rev());
            }
            cuts[line].push((if end { f32::INFINITY } else { 0.0 }, join));
        }
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
    rings.resize(joins.len(), None);
    // Joins nearer each other than [`ONE_PLACE`] are one place: the two
    // halves of the new town's ring either side of a gap in the strip
    // between them, where a straight road comes in. The way across between
    // them is no stretch of road, but turns across that place, from one half
    // to the other, which a car takes as it would change lanes, rather than
    // as two right angles a few metres apart, which the bus could not take
    // without swinging round on its back wheels.
    let mut place: Vec<usize> = (0..joins.len()).collect();
    fn root(place: &mut [usize], mut at: usize) -> usize {
        while place[at] != at {
            place[at] = place[place[at]];
            at = place[at];
        }
        at
    }
    for a in 0..joins.len() {
        for b in 0..a {
            if joins[a].distance(joins[b]) < ONE_PLACE && rings[a].is_none() && rings[b].is_none() {
                let (ra, rb) = (root(&mut place, a), root(&mut place, b));
                place[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    let mut number = vec![usize::MAX; joins.len()];
    let (mut placed, mut placed_rings, mut members): (Vec<Vec2>, Vec<Option<Ring>>, Vec<usize>) = Default::default();
    for join in 0..joins.len() {
        let top = root(&mut place, join);
        if number[top] == usize::MAX {
            number[top] = placed.len();
            placed.push(Vec2::ZERO);
            placed_rings.push(rings[top]);
            members.push(0);
        }
        number[join] = number[top];
        placed[number[join]] += joins[join];
        members[number[join]] += 1;
    }
    for (at, count) in placed.iter_mut().zip(&members) {
        *at /= *count as f32;
    }
    for stretch in &mut stretches {
        (stretch.from, stretch.to) = (number[stretch.from], number[stretch.to]);
    }
    stretches.retain(|stretch| stretch.from != stretch.to || *along(&stretch.points, false).last().unwrap() > ONE_PLACE * 2.0);
    Network {
        joins: placed,
        stretches,
        rings: placed_rings,
    }
}

/// Where the line from `from` along `way` meets `points`, every time it does
/// within `reach`: how far along the line, and the point.
fn meetings(from: Vec2, way: Vec2, reach: f32, points: &[Vec2], closed: bool) -> Vec<(f32, Vec2)> {
    let spans = if closed { points.len() } else { points.len() - 1 };
    let mut found = Vec::new();
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
        if (-1e-4..=1.0 + 1e-4).contains(&u) && (0.0..=reach).contains(&t) {
            found.push((t, from + way * t));
        }
    }
    found
}

/// The points on from the end of `road`'s middle, its last if `end`, else its
/// first, to `to`, the middle of the island of a roundabout it runs into,
/// `to` the last of them: round the circle through its last two dashes and
/// `to`, as far apart as the rest of its points, so that a road round a bend
/// goes on round it, and a straight one straight on. Hajun's new town's road
/// round its edge, 194 m round, stops its dashes 23 m short of the middle of
/// each roundabout's island since its roundabouts grew (2026-10-30), and a
/// straight line on from its last dash turned 3.4° off it there, as did its
/// lanes and the bus coming onto the roundabout along them.
fn on_to(road: &Middle, end: bool, to: Vec2) -> Vec<Vec2> {
    let dashes = &road.dashes;
    let n = dashes.len();
    let (tip, before) = if end { (dashes[n - 1], dashes[n - 2]) } else { (dashes[0], dashes[1]) };
    let apart = tip.distance(before) / BETWEEN_DASHES as f32;
    let pieces = (tip.distance(to) / apart.max(0.1)).ceil().max(1.0) as usize;
    (1..=pieces)
        .map(|k| {
            let share = k as f32 / pieces as f32;
            if k == pieces {
                to
            } else {
                on_circle(before, tip, to, share).unwrap_or_else(|| tip.lerp(to, share))
            }
        })
        .collect()
}

/// The roundabout round the island that roads running off it at `runs_off`
/// run off onto, if there is one: the middle of the island, and how far out
/// from it the island goes and the road round it, as the road's paint is at
/// `top`. The island's edge is felt for out from among where they run off,
/// all the way round, and a circle drawn through it; the road round it goes
/// out as far as it does out from the island in at least a quarter of the
/// ways round, which the roads coming in leave out.
fn roundabout(paint: &Paint, runs_off: &[Vec2], top: f32) -> Option<(Vec2, Ring)> {
    let from = runs_off.iter().sum::<Vec2>() / runs_off.len() as f32;
    let ways = |middle: Vec2| {
        (0..ISLAND_WAYS).map(move |k| (middle, Vec2::from_angle(k as f32 / ISLAND_WAYS as f32 * TAU)))
    };
    // The edge of the island each way round, where the road starts.
    let edges: Vec<Vec2> = ways(from)
        .filter_map(|(middle, way)| {
            let mut out = 0.0;
            while out < ISLAND_REACH {
                if paint.on_road(middle + way * out, top) {
                    return Some(middle + way * out);
                }
                out += FEEL;
            }
            None
        })
        .collect();
    if edges.len() < ISLAND_WAYS * 3 / 4 {
        return None;
    }
    // The circle nearest all of them: x * x + z * z + d x + e z + f = 0.
    let (mut m, mut v) = ([[0.0f64; 3]; 3], [0.0f64; 3]);
    for edge in &edges {
        let (x, z) = (f64::from(edge.x - from.x), f64::from(edge.y - from.y));
        let row = [x, z, 1.0];
        let rhs = -(x * x + z * z);
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] += row[i] * row[j];
            }
            v[i] += row[i] * rhs;
        }
    }
    let [d, e, f] = solve(m, v)?;
    let middle = from + Vec2::new((-d / 2.0) as f32, (-e / 2.0) as f32);
    let inside = ((d * d + e * e) / 4.0 - f).max(0.0).sqrt() as f32;
    let off = edges.iter().map(|edge| (edge.distance(middle) - inside).abs()).fold(0.0, f32::max);
    if !(SMALLEST_ISLAND..=LARGEST_ISLAND).contains(&inside) || off > inside * 0.25 {
        return None;
    }
    // How far the road round it goes, each way round from the island.
    let mut outs: Vec<f32> = ways(middle)
        .map(|(middle, way)| {
            let mut out = inside + FEEL;
            while out < inside + ISLAND_REACH && paint.on_road(middle + way * out, top) {
                out += FEEL;
            }
            out
        })
        .collect();
    outs.sort_by(f32::total_cmp);
    let outside = outs[outs.len() / 4];
    (outside > inside + 1.0).then_some((middle, Ring { inside, outside }))
}

/// `x` where `m x = v`, if `m` can be undone.
fn solve(m: [[f64; 3]; 3], v: [f64; 3]) -> Option<[f64; 3]> {
    let det = |m: &[[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let whole = det(&m);
    if whole.abs() < 1e-12 {
        return None;
    }
    let mut x = [0.0; 3];
    for (k, slot) in x.iter_mut().enumerate() {
        let mut swapped = m;
        for row in swapped.iter_mut() {
            row[k] = 0.0;
        }
        for (row, value) in swapped.iter_mut().zip(v) {
            row[k] = value;
        }
        *slot = det(&swapped) / whole;
    }
    Some(x)
}

/// One lane: one way down one stretch, kept to the right of its middle, from
/// the join it starts at to the one it ends at.
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
    /// How far short of each end it is left, to turn from: the half of the
    /// widest road at the join, which a turn crosses, and some room. A turn
    /// from there that would take the car off the road is turned from nearer
    /// the join ([`turning`]).
    room: (f32, f32),
    /// How far to the right of the road's middle it is: how far across a car
    /// in it moves to turn wide from the middle of the road.
    inside: f32,
    /// The fastest a car goes along it, in metres a second: its top speed, or
    /// on an open road, between two roundabouts, more ([`OPEN_ROAD`]).
    speed: f32,
}

impl Lane {
    /// How long it is, from join to join.
    fn length(&self) -> f32 {
        *along(&self.points, false).last().unwrap()
    }
}

/// Both lanes of every stretch of `network`.
fn lanes(network: &Network) -> Vec<Lane> {
    // How far back from each join a lane is left to turn from: the half of
    // the widest road there, which a turn crosses, and some room.
    let mut room = vec![0.0f32; network.joins.len()];
    for stretch in &network.stretches {
        for join in [stretch.from, stretch.to] {
            room[join] = room[join].max(stretch.left.max(stretch.right) + ROOM_TO_TURN);
        }
    }
    // At a roundabout, back from the road round its island.
    for (join, ring) in network.rings.iter().enumerate() {
        if let Some(ring) = ring {
            room[join] = room[join].max(ring.outside + ROOM_TO_TURN);
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
            let roundabout = |join: usize| network.rings.get(join).copied().flatten().is_some();
            let open = roundabout(from) && roundabout(to);
            let whole = *along(&lane, false).last().unwrap();
            let (mut start, mut end) = (room[from], room[to]);
            if start + end > whole * MOST_CUT {
                let scale = whole * MOST_CUT / (start + end);
                start *= scale;
                end *= scale;
            }
            lanes.push(Lane {
                points: lane,
                from,
                to,
                stretch: index,
                ahead,
                top: road.top,
                room: (start, end),
                inside: if ahead { road.right } else { road.left } * 0.5,
                speed: roundtown_net::cars::TOP_SPEED * if open { OPEN_ROAD } else { 1.0 },
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
/// coming into it with one going out of it, as far as `allowed` allows: tried
/// at random, from a seed of its own, until one pairing at every join makes
/// a single loop.
fn circuit(lanes: &[Lane], drive: &[usize], joins: usize, allowed: &dyn Fn(usize, usize) -> bool) -> Option<Vec<usize>> {
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
        let every: Vec<Vec<usize>> = orders(outs)
            .into_iter()
            .filter(|order| ins.iter().zip(order.iter()).all(|(&i, &o)| allowed(drive[i], drive[o])))
            .collect();
        let mut turns: Vec<Vec<usize>> = every
            .iter()
            .filter(|order| ins.iter().zip(order.iter()).all(|(&i, &o)| !back(i, o)))
            .cloned()
            .collect();
        // At the end of a road the only way on is back.
        if turns.is_empty() {
            turns = every;
        }
        if turns.is_empty() && !ins.is_empty() {
            return None;
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

/// The lanes, numbered up to `count`, that a car can go round from any of
/// them to any other, turning from one into the next where `turns` says it
/// can: each lot of them, every lane in one, in order.
fn round_together(count: usize, turns: &dyn Fn(usize, usize) -> bool) -> Vec<Vec<usize>> {
    // Where each can get to, and what can get to each.
    let reach = |from: usize, forward: bool| -> Vec<bool> {
        let mut seen = vec![false; count];
        let mut next = vec![from];
        seen[from] = true;
        while let Some(at) = next.pop() {
            for (other, been) in seen.iter_mut().enumerate() {
                let joined = if forward { turns(at, other) } else { turns(other, at) };
                if joined && !*been {
                    *been = true;
                    next.push(other);
                }
            }
        }
        seen
    };
    let mut placed = vec![false; count];
    let mut lots = Vec::new();
    for lane in 0..count {
        if placed[lane] {
            continue;
        }
        let (to, from) = (reach(lane, true), reach(lane, false));
        let lot: Vec<usize> = (0..count).filter(|&other| to[other] && from[other]).collect();
        for &other in &lot {
            placed[other] = true;
        }
        lots.push(lot);
    }
    lots
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

/// The order the car drives the lanes `every` names in, round the town,
/// turning only as `allowed` allows: each lane once if that makes one loop,
/// and if not, one stretch driven twice, each way, the shortest that does.
/// `None` if no loop takes them all in.
fn plan(lanes: &[Lane], every: &[usize], joins: usize, allowed: &dyn Fn(usize, usize) -> bool) -> Option<Vec<usize>> {
    if let Some(order) = circuit(lanes, every, joins, allowed) {
        return Some(order);
    }
    // By whole metres, so that stretches all but as long as each other are
    // tried in the order they come in, on every device.
    let stretches = lanes.iter().map(|lane| lane.stretch).max()? + 1;
    let length = |stretch: usize| -> i64 {
        every
            .iter()
            .map(|&lane| &lanes[lane])
            .filter(|lane| lane.stretch == stretch)
            .map(|lane| lane.length() - lane.room.0 - lane.room.1)
            .sum::<f32>()
            .round() as i64
    };
    let mut twice: Vec<usize> = (0..stretches).collect();
    twice.sort_by_key(|&stretch| (length(stretch), stretch));
    twice.into_iter().find_map(|stretch| {
        let doubled: Vec<usize> = every.iter().copied().filter(|&lane| lanes[lane].stretch == stretch).collect();
        if doubled.is_empty() {
            return None;
        }
        let drive: Vec<usize> = every.iter().copied().chain(doubled).collect();
        circuit(lanes, &drive, joins, allowed)
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

/// The room a car takes on the road, which its way round is worked out for.
///
/// Its front wheels keep to the line, and its back ones, `base` behind them,
/// follow them as a car's do ([`trailing`]): pulled along the way the car
/// faces, never sideways, so that round a bend they keep inside it, and the
/// car turns as a car turns. Its body faces from the back wheels to the
/// front ones, and only what of it is ahead of its front wheels and behind
/// its back ones swings out over the bend.
///
/// The box round all of it, `half` long and wide each way, has its middle
/// `ahead` of the middle of its wheels; the box round its wheels, `wheels`
/// long and wide each way, has the middle of its wheels for its middle.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Footprint {
    pub(crate) base: f32,
    pub(crate) half: Vec2,
    pub(crate) ahead: f32,
    pub(crate) wheels: Vec2,
}

impl Footprint {
    /// A car `half` long and wide each way that turns about its middle, its
    /// wheels all of it.
    #[cfg(test)]
    pub(crate) fn turning_about_its_middle(half: Vec2) -> Self {
        Self {
            base: 0.0,
            half,
            ahead: 0.0,
            wheels: half,
        }
    }
}

/// Where a car's front and back wheels are as its front ones go along
/// `points`, its back ones `base` behind them, as a car's follow: each step
/// the front ones take, the back ones are pulled after them along the way the
/// car faces, toward the front ones, and never sideways, so that round a bend
/// they keep inside it, and come back onto the line along the straight after
/// it. From the first point, the car facing along the line; or, round on
/// itself if `closed`, once it has been round, where it then is every time.
/// As the front wheels and the back ones at each point.
fn trailing(points: &[Vec2], closed: bool, base: f32) -> Vec<(Vec2, Vec2)> {
    let count = points.len();
    if count < 2 {
        return points.iter().map(|&point| (point, point)).collect();
    }
    // A car whose wheels are all in its middle turns about it: facing the way
    // from the point before each to the one after, its back wheels a hair
    // behind its front ones.
    if base <= 1e-3 {
        let mut way = (points[1] - points[0]).normalize_or(Vec2::X);
        return (0..count)
            .map(|at| {
                let (before, after) = if closed {
                    (points[(at + count - 1) % count], points[(at + 1) % count])
                } else {
                    (points[at.saturating_sub(1)], points[(at + 1).min(count - 1)])
                };
                way = (after - before).try_normalize().unwrap_or(way);
                (points[at], points[at] - way * 1e-3)
            })
            .collect();
    }
    let first = (points[1] - points[0]).normalize_or(Vec2::X);
    let (mut front, mut back) = (points[0], points[0] - first * base);
    let mut out = Vec::with_capacity(count);
    let laps = if closed { 2 } else { 1 };
    for lap in 0..laps {
        for &next in points {
            // A tenth of a metre at a time at the most.
            let steps = (next.distance(front) / 0.1).ceil().max(1.0) as usize;
            for step in 1..=steps {
                let at = front.lerp(next, step as f32 / steps as f32);
                back = at - (at - back).normalize_or(first) * base;
            }
            front = next;
            if lap + 1 == laps {
                out.push((front, back));
            }
        }
    }
    out
}

/// Where a car stands at each of `wheels`, its front and back wheels'
/// places: the middle of its wheels, and the way it faces, from its back
/// wheels to its front ones.
fn stands(wheels: &[(Vec2, Vec2)]) -> Vec<(Vec2, Vec2)> {
    wheels
        .iter()
        .map(|&(front, back)| ((front + back) * 0.5, (front - back).normalize_or_zero()))
        .collect()
}

/// A turn from one lane into the next: how far short of the end of the one
/// it leaves it, how far along the other it joins it, and its points between.
#[derive(Clone, Debug)]
struct Turn {
    leave: f32,
    join: f32,
    points: Vec<Vec2>,
}

/// The turn from the end of `this` lane into the start of `next`, at the
/// join between, for a car that takes the room `car` takes on the road, its
/// front wheels on the line and its back ones following ([`trailing`]).
///
/// At an ordinary join, turned round corners as round as the road allows
/// ([`fillets`]), between the lanes or swinging wide from the middle of the
/// road into the middle of the other; and from the room left at the end of
/// the one and the start of the other ([`Lane::room`]) and nearer the join,
/// and turning wide, as a curve between them ([`corner_shapes`]). At a
/// roundabout, round its island as well ([`round_shapes`]).
///
/// Of them all, the first tried that keeps all of the car [`KERB`] inside
/// the road's edge; failing that, the first that keeps its wheels so, or
/// failing that on the road, the rest of it swinging out over nothing higher
/// than its underside ([`UNDERSIDE`]), as a bus's does; and if none does
/// even that, the one that takes the fewest of the points round its wheels
/// over it. Where that bends tighter than [`RELAXED`], the gentlest instead
/// that does as well: at a tight corner the car swings wide into it, as a
/// bus does (Hajun, 2026-10-30: "try to do wide turns in tight corner so the
/// turning animation is not weird this time"). Never one tighter than it can
/// follow ([`TIGHTEST`]), if there is another way that keeps its wheels on
/// the road; with whether it is, the turn.
fn turning(this: &Lane, next: &Lane, paint: &Paint, car: &Footprint, network: &Network) -> (Turn, bool) {
    let next_whole = *along(&next.points, false).last().unwrap();
    let top = this.top.max(next.top);
    let on_road = |at: Vec2| paint.on_road(at, top);
    let ring = network.rings.get(this.to).copied().flatten();
    // Each turn tried, as the turn, all its front wheels go over that they
    // did not turning from the room left, and how far along `this` that
    // starts and along `next` it ends.
    let shapes: Vec<(Turn, Vec<Vec2>, f32, f32)> = match ring {
        Some(ring) => round_shapes(this, next, network.joins[this.to], ring, car.base),
        None => {
            let mut shapes = fillets(this, next, car);
            shapes.extend(corner_shapes(this, next));
            shapes
        }
    };
    // In the order tried; and the gentlest first, to swing wide by.
    let bends: Vec<f32> = shapes.iter().map(|(_, path, ..)| tightest(path)).collect();
    let tried: Vec<usize> = (0..shapes.len()).collect();
    let mut order = tried.clone();
    order.sort_by(|&a, &b| bends[b].total_cmp(&bends[a]).then(a.cmp(&b)));
    let followable: Vec<usize> = tried.iter().copied().filter(|&at| bends[at] >= car.base * TIGHTEST).collect();
    // Where the car stands at each point of all it goes over: from the lane
    // before, which only brings its back wheels to where they would be, to
    // the lane after, where they come back onto the line.
    let lead_in = car.base * LEAD_IN;
    let lead_out = car.base * LEAD_OUT;
    let stands_on = |at: usize| -> Vec<(Vec2, Vec2)> {
        let (_, path, from, to) = &shapes[at];
        let mut points = stretch(&this.points, false, (from - lead_in).max(0.0), *from);
        let lead = points.len();
        points.extend_from_slice(path);
        points.extend(stretch(&next.points, false, *to, (to + lead_out).min(next_whole)));
        stands(&trailing(&points, false, car.base)[lead..])
    };
    let all = (outline(car.half + KERB), car.ahead);
    let wheels = (outline(car.wheels + KERB), 0.0);
    let only_wheels = (car.wheels, 0.0) != (car.half, car.ahead);
    let just = (outline(car.wheels), 0.0);
    let mut stood: Vec<Option<Vec<(Vec2, Vec2)>>> = vec![None; shapes.len()];
    let mut stand = |at: usize| -> Vec<(Vec2, Vec2)> { stood[at].get_or_insert_with(|| stands_on(at)).clone() };
    // Round a roundabout's island, only the wheels, which may run up onto
    // its edge ([`APRON`]): the gentlest that keeps them on the road, or the
    // one that keeps them most.
    if let Some(ring) = ring {
        let middle = network.joins[this.to];
        let corners = &outline(car.wheels)[..4];
        let on_road = |at: Vec2| {
            let out = at.distance(middle);
            paint.on_road(at, top) || (ring.inside - APRON..=ring.inside + FEEL).contains(&out)
        };
        // Of those it can follow, if there are any.
        let weighed = if followable.is_empty() { &order } else { &followable };
        let (mut least, mut best) = (usize::MAX, weighed[0]);
        for &at in weighed.iter().take(ROUND_TRIES) {
            let off = off_the_road(&stand(at), corners, 0.0, on_road, least);
            if off < least {
                (least, best) = (off, at);
            }
            if least == 0 {
                break;
            }
        }
        return (shapes[best].0.clone(), bends[best] >= car.base * TIGHTEST);
    }
    // Its body, where its wheels are on the road and it is not, swings out
    // over nothing higher than its underside: a kerb or paving, not the
    // strip of grass down the middle of a road, a wall or a lamp.
    let body = outline(car.half);
    let over = |at: Vec2| on_road(at) || !paint.land.blocked(at, UNDERSIDE, UNDERSIDE + OVERHEAD);
    for (group, gentle) in [(&followable, true), (&tried, false)] {
        for (outline, ahead) in std::iter::once(&all).chain(only_wheels.then_some(&wheels)).chain([&just]) {
            let whole = std::ptr::eq(outline, &all.0);
            let mut clear = |at: usize| {
                let stands = stand(at);
                off_the_road(&stands, outline, *ahead, on_road, 1) == 0
                    && (whole || off_the_road(&stands, &body, car.ahead, over, 1) == 0)
            };
            let Some(first) = group.iter().copied().find(|&at| clear(at)) else {
                continue;
            };
            if bends[first] >= car.base * RELAXED {
                return (shapes[first].0.clone(), gentle);
            }
            // A tight corner: the gentlest that keeps the car as well.
            let gentlest = order
                .iter()
                .copied()
                .filter(|&at| bends[at] > bends[first] && group.contains(&at))
                .find(|&at| clear(at))
                .unwrap_or(first);
            return (shapes[gentlest].0.clone(), gentle);
        }
    }
    // None keeps even its wheels clear of the edge: the one that takes them
    // over it least, counting each no further than the least so far. All
    // round them, not only their corners: turning tightly, the side of a long
    // car between them cuts the corner it turns round.
    let (mut least, mut best) = (usize::MAX, order[0]);
    for &at in followable.iter().chain(&order).take(ROUND_TRIES) {
        let off = off_the_road(&stand(at), &wheels.0, 0.0, on_road, least);
        if off < least {
            (least, best) = (off, at);
        }
    }
    let middle = shapes[best].1.get(shapes[best].1.len() / 2).copied().unwrap_or_default();
    debug!("the car's turn at {middle} takes its wheels off the road, by {least} of the points round them");
    (shapes[best].0.clone(), false)
}

/// The turns from the end of `this` lane into the start of `next` round
/// corners of [`ROUNDS`] for a car that takes the room `car` does: the arc
/// of each round as it turns from the line of the one lane to the line of
/// the other, or from the middle of the road it leaves, or into the middle of
/// the road it joins, moving across between it and its lane over
/// [`CROSS_OVER`], as a bus turning wide does. None where the two go the
/// same way or turn right back. As [`corner_shapes`] has them.
fn fillets(this: &Lane, next: &Lane, car: &Footprint) -> Vec<(Turn, Vec<Vec2>, f32, f32)> {
    let (this_far, next_far) = (along(&this.points, false), along(&next.points, false));
    let (this_whole, next_whole) = (*this_far.last().unwrap(), *next_far.last().unwrap());
    let (end, end_way) = heading_at(&this.points, &this_far, this_whole);
    let (start, start_way) = heading_at(&next.points, &next_far, 0.0);
    let turned = end_way.perp_dot(start_way);
    let angle = end_way.angle_to(start_way).abs();
    if turned.abs() < 0.3 || angle > 2.8 {
        return Vec::new();
    }
    let mut shapes = Vec::new();
    for (wide_from, wide_to) in [(false, false), (true, false), (false, true), (true, true)] {
        // The two lines, the one along the lane or the middle of its road.
        let from_line = if wide_from { end - end_way.perp() * this.inside } else { end };
        let to_line = if wide_to { start - start_way.perp() * next.inside } else { start };
        // Where they cross, along each from its point.
        let gap = to_line - from_line;
        let along_from = gap.perp_dot(start_way) / end_way.perp_dot(start_way);
        let along_to = gap.perp_dot(end_way) / end_way.perp_dot(start_way);
        for round in ROUNDS {
            let radius = car.base * round;
            // A car with its wheels all in its middle turns round curves.
            if radius < NEAREST {
                continue;
            }
            let tangent = radius * (angle * 0.5).tan();
            // How far back from the end of the lane the arc starts, and how
            // far along the next it ends; turning wide, how far back the car
            // moves across to the middle of the road, and back into its lane.
            let leave = tangent - along_from;
            let join = along_to + tangent;
            let off_lane = leave + if wide_from { CROSS_OVER } else { 0.0 };
            let on_lane = join + if wide_to { CROSS_OVER } else { 0.0 };
            if leave < NEAREST * 0.5 || join < NEAREST * 0.5 || off_lane > this_whole * MOST_WIDE || on_lane > next_whole * MOST_WIDE {
                continue;
            }
            // The arc, round its middle, from where it leaves the one line
            // to where it meets the other: only where those are the lanes, or
            // their middles, as near as makes no odds. Off a lane round a
            // ring, a line along it from its end goes off it, 2 m in 15.
            let arc_from = from_line + end_way * (along_from - tangent);
            let arc_to = to_line + start_way * (along_to + tangent);
            let on_this = heading_at(&this.points, &this_far, this_whole - leave).0
                - if wide_from { end_way.perp() * this.inside } else { Vec2::ZERO };
            let on_next = heading_at(&next.points, &next_far, join).0
                - if wide_to { start_way.perp() * next.inside } else { Vec2::ZERO };
            if arc_from.distance(on_this) > ON_THE_LINE || arc_to.distance(on_next) > ON_THE_LINE {
                continue;
            }
            let middle = arc_from + end_way.perp() * radius * turned.signum();
            let pieces = ((radius * angle) / SPACING).ceil().max(2.0) as usize;
            let arc = |k: usize| {
                let swing = angle * turned.signum() * k as f32 / pieces as f32;
                middle + Vec2::from_angle(swing).rotate(arc_from - middle)
            };
            // Its ends are lane points, but where it turns wide, and the
            // lane's own are not the turn's.
            let mut points = Vec::new();
            if wide_from {
                let (lane, lane_way) = heading_at(&this.points, &this_far, this_whole - off_lane);
                points.extend(turn(lane, lane_way, arc_from, end_way));
            }
            let first = if wide_from { 0 } else { 1 };
            let last = if wide_to { pieces } else { pieces - 1 };
            points.extend((first..=last).map(arc));
            if wide_to {
                let (lane, lane_way) = heading_at(&next.points, &next_far, on_lane);
                points.extend(turn(arc(pieces), start_way, lane, lane_way));
            }
            let mut path = Vec::new();
            if off_lane <= this.room.1 {
                path = stretch(&this.points, false, this_whole - this.room.1, this_whole - off_lane);
            }
            path.extend_from_slice(&points);
            if on_lane <= next.room.0 {
                path.extend(stretch(&next.points, false, on_lane, next.room.0));
            }
            shapes.push((
                Turn {
                    leave: off_lane,
                    join: on_lane,
                    points,
                },
                path,
                this_whole - off_lane.max(this.room.1),
                on_lane.max(next.room.0),
            ));
        }
    }
    shapes
}

/// The turns from the end of `this` lane into the start of `next` at an
/// ordinary join, each tried ([`turning`]): how many steps nearer the join
/// than the room left it leaves the one lane and joins the other, and
/// whether it turns wide, from the middle of the road or onto it; every turn
/// from lane to lane first, so that a car that can turn that way does. Each
/// as the turn, all the car's front wheels go over that they did not turning
/// from the room left, and how far along `this` that starts and along `next`
/// it ends.
fn corner_shapes(this: &Lane, next: &Lane) -> Vec<(Turn, Vec<Vec2>, f32, f32)> {
    let (this_far, next_far) = (along(&this.points, false), along(&next.points, false));
    let (this_whole, next_whole) = (*this_far.last().unwrap(), *next_far.last().unwrap());
    // From the room left and nearer the join, or further back from it: a
    // car changing from one half of a ring to the other, through a gap in
    // the strip between, as in the new town, goes across gently the longer
    // it takes.
    let steps = |room: f32| -> Vec<f32> {
        let nearer = ((room - NEAREST) / NEARER).floor().clamp(0.0, 8.0) as usize;
        (0..=nearer)
            .map(|step| room - step as f32 * NEARER)
            .chain(FURTHER.iter().map(|further| room + further))
            .collect()
    };
    let mut tries: Vec<(f32, f32, bool, bool)> = Vec::new();
    for (wide_from, wide_to) in [(false, false), (false, true), (true, false), (true, true)] {
        for &leave in &steps(this.room.1) {
            for &join in &steps(next.room.0) {
                tries.push((leave, join, wide_from, wide_to));
            }
        }
    }
    tries
        .into_iter()
        .filter_map(|(leave, join, wide_from, wide_to)| {
            // Turning wide, it moves across from its lane to the middle of
            // the road, over [`CROSS_OVER`], before it turns, or from the
            // middle to its lane after: a bus turning into the far lane of a
            // road swings its front out over the far kerb, turning from lane
            // to lane, and turning into the middle of the road it has half as
            // far again to swing it in. Both ends of it no further into a
            // lane than [`MOST_WIDE`] of the way along it.
            let off_lane = leave + if wide_from { CROSS_OVER } else { 0.0 };
            let on_lane = join + if wide_to { CROSS_OVER } else { 0.0 };
            if off_lane > this_whole * MOST_WIDE || on_lane > next_whole * MOST_WIDE {
                return None;
            }
            let (from, from_way) = heading_at(&this.points, &this_far, this_whole - leave);
            let (to, to_way) = heading_at(&next.points, &next_far, join);
            let from = if wide_from { from - from_way.perp() * this.inside } else { from };
            let to = if wide_to { to - to_way.perp() * next.inside } else { to };
            let mut points = Vec::new();
            if wide_from {
                let (lane, lane_way) = heading_at(&this.points, &this_far, this_whole - off_lane);
                points.extend(turn(lane, lane_way, from, from_way));
                points.push(from);
            }
            points.extend(turn(from, from_way, to, to_way));
            if wide_to {
                let (lane, lane_way) = heading_at(&next.points, &next_far, on_lane);
                points.push(to);
                points.extend(turn(to, to_way, lane, lane_way));
            }
            // All the car goes over that it did not, turning from the room
            // left: the lanes further in, and the turn.
            let mut path = Vec::new();
            if off_lane <= this.room.1 {
                path = stretch(&this.points, false, this_whole - this.room.1, this_whole - off_lane);
            }
            path.extend_from_slice(&points);
            if on_lane <= next.room.0 {
                path.extend(stretch(&next.points, false, on_lane, next.room.0));
            }
            let from_s = this_whole - off_lane.max(this.room.1);
            Some((
                Turn {
                    leave: off_lane,
                    join: on_lane,
                    points,
                },
                path,
                from_s,
                on_lane.max(next.room.0),
            ))
        })
        .collect()
}

/// The turns from the end of `this` lane into the start of `next` round the
/// roundabout at their join, its middle at `middle` (Hajun's new town's four,
/// 2026-10-30), for a car whose wheels are `base` apart: off the one lane
/// round an arc to the right onto the road round the island, round the
/// island keeping it on the left, as a roundabout is driven where cars keep
/// to the right, and off it round another arc to the right onto the other
/// lane. Each arc runs on from the last along it, so that the car turns from
/// one into the next without a jolt; round at a few ways out from the island
/// ([`ROUND_AT`]), on and off round arcs of a few sizes ([`MERGES`]). As
/// [`corner_shapes`] has them.
fn round_shapes(this: &Lane, next: &Lane, middle: Vec2, ring: Ring, base: f32) -> Vec<(Turn, Vec<Vec2>, f32, f32)> {
    let (this_far, next_far) = (along(&this.points, false), along(&next.points, false));
    let (this_whole, next_whole) = (*this_far.last().unwrap(), *next_far.last().unwrap());
    let mut shapes = Vec::new();
    for share in ROUND_AT {
        let radius = ring.inside + (ring.outside - ring.inside) * share;
        for on in MERGES {
            for off in MERGES {
                let Some(onto) = merge(&this.points, &this_far, this_whole - this.room.1, middle, radius, base * on, false)
                else {
                    continue;
                };
                let Some(off_of) = merge(&next.points, &next_far, next.room.0, middle, radius, base * off, true) else {
                    continue;
                };
                let (leave, join) = (this_whole - onto.along, off_of.along);
                if leave < NEAREST || join < NEAREST || leave > this_whole * MOST_WIDE || join > next_whole * MOST_WIDE {
                    continue;
                }
                // Round the island from where the one arc meets the road
                // round it to where the other leaves it: the angle going down.
                let (on_at, off_at) = ((onto.ring - middle).to_angle(), (off_of.ring - middle).to_angle());
                let round = (on_at - off_at).rem_euclid(TAU);
                if round > TAU - 0.3 {
                    continue;
                }
                let mut points = arc(onto.centre, onto.lane, onto.ring);
                points.push(onto.ring);
                let pieces = ((round * radius) / SPACING).ceil().max(1.0) as usize;
                points.extend((1..pieces).map(|k| middle + Vec2::from_angle(on_at - round * k as f32 / pieces as f32) * radius));
                points.push(off_of.ring);
                points.extend(arc(off_of.centre, off_of.ring, off_of.lane));
                let mut path = vec![onto.lane];
                path.extend_from_slice(&points);
                path.push(off_of.lane);
                shapes.push((Turn { leave, join, points }, path, onto.along, join));
            }
        }
    }
    shapes
}

/// An arc to the right between a lane and the road round an island
/// ([`merge`]): how far along the lane it meets it, its middle, and where it
/// meets the lane and the road round the island.
struct Merge {
    along: f32,
    centre: Vec2,
    lane: Vec2,
    ring: Vec2,
}

/// The arc to the right, `bend` round, by which a car turns off a lane,
/// `points` (whose distances along are `so_far`), onto the road round an
/// island, keeping the island on its left, round its middle `middle` at
/// `radius`; or, `leaving`, by which one comes off the road round it onto
/// the lane. Its middle is on the right of the lane, `bend` from it, and
/// `radius + bend` from the island's: found from the lane's line at `from`
/// along it, and again from its line where the arc meets it, which on a
/// lane round a bend is a little further on, until it meets it where it
/// says. `None` if no arc that size meets both.
fn merge(points: &[Vec2], so_far: &[f32], from: f32, middle: Vec2, radius: f32, bend: f32, leaving: bool) -> Option<Merge> {
    let whole = *so_far.last()?;
    let mut at = from.clamp(0.0, whole);
    for _ in 0..6 {
        let (point, way) = heading_at(points, so_far, at);
        let side = point + way.perp() * bend - middle;
        let (b, c) = (side.dot(way), side.length_squared() - (radius + bend).powi(2));
        let roots = b * b - c;
        if roots < 0.0 {
            return None;
        }
        // Coming on, where it first can, going along the lane; going off,
        // where it last can.
        let step = if leaving { -b + roots.sqrt() } else { -b - roots.sqrt() };
        if !(0.0..=whole).contains(&(at + step)) {
            return None;
        }
        at += step;
        if step.abs() < 1e-3 {
            break;
        }
    }
    let (lane, way) = heading_at(points, so_far, at);
    let centre = lane + way.perp() * bend;
    if ((centre - middle).length() - (radius + bend)).abs() > ON_THE_LINE * 2.0 {
        return None;
    }
    Some(Merge {
        along: at,
        centre,
        lane,
        ring: middle + (centre - middle).normalize() * radius,
    })
}

/// The points of an arc to the right round `centre` from `from` to `to`,
/// neither of them.
fn arc(centre: Vec2, from: Vec2, to: Vec2) -> Vec<Vec2> {
    let swing = (from - centre).angle_to(to - centre).rem_euclid(TAU);
    let pieces = ((swing * from.distance(centre)) / SPACING).ceil().max(1.0) as usize;
    (1..pieces)
        .map(|k| centre + Vec2::from_angle(swing * k as f32 / pieces as f32).rotate(from - centre))
        .collect()
}

/// How tightly the line through `points` bends at its tightest, as the radius
/// of the circle through a point and the points either side of it, those
/// less than a tenth of a metre from the one before left out.
fn tightest(points: &[Vec2]) -> f32 {
    let mut apart: Vec<Vec2> = Vec::with_capacity(points.len());
    for &point in points {
        if apart.last().is_none_or(|last| last.distance(point) >= 0.1) {
            apart.push(point);
        }
    }
    apart
        .windows(3)
        .map(|three| {
            let (a, b, c) = (three[0], three[1], three[2]);
            let twice_the_area = (b - a).perp_dot(c - a).abs();
            if twice_the_area < 1e-6 {
                f32::INFINITY
            } else {
                a.distance(b) * b.distance(c) * c.distance(a) / (2.0 * twice_the_area)
            }
        })
        .fold(f32::INFINITY, f32::min)
}

/// Points all the way round a box `half` long and wide each way, along it and
/// across, no more than [`FEEL`] apart: its four corners first, which are what
/// goes off the road first, turning, and then the rest along its sides.
fn outline(half: Vec2) -> Vec<Vec2> {
    let corners = [
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
    ];
    let mut points = corners.to_vec();
    for side in 0..4 {
        let (a, b) = (corners[side], corners[(side + 1) % 4]);
        let steps = (a.distance(b) / FEEL).ceil().max(1.0) as usize;
        points.extend((1..steps).map(|step| a.lerp(b, step as f32 / steps as f32)));
    }
    points
}

/// How many of the points `outline` round a box, along it and across it, are
/// off the road, wherever the car is in `stands`, as the middle of its wheels
/// and the way it faces, the box's middle `ahead` of the middle of its
/// wheels: counted as far as `enough`, and no further. Its corners are looked
/// at all the way along first, so that a turn that takes one off the road is
/// found out in a few looks rather than a hundred at each point.
fn off_the_road(
    stands: &[(Vec2, Vec2)],
    outline: &[Vec2],
    ahead: f32,
    on_road: impl Fn(Vec2) -> bool,
    enough: usize,
) -> usize {
    let corners = outline.len().min(4);
    let mut off = 0;
    for points in [&outline[..corners], &outline[corners..]] {
        for &(wheels, way) in stands {
            if way == Vec2::ZERO {
                continue;
            }
            let middle = wheels + way * ahead;
            for corner in points {
                if !on_road(middle + way * corner.x + way.perp() * corner.y) {
                    off += 1;
                    if off >= enough {
                        return off;
                    }
                }
            }
        }
    }
    off
}

/// Whether two boxes `half` long and wide each way, along them and across,
/// one round `a` facing `a_way` and the other round `b` facing `b_way`,
/// overlap: neither has a side the other is wholly beyond.
fn boxes_meet(a: Vec2, a_way: Vec2, b: Vec2, b_way: Vec2, half: Vec2) -> bool {
    let reach = |way: Vec2, axis: Vec2| half.x * way.dot(axis).abs() + half.y * way.perp().dot(axis).abs();
    [a_way, a_way.perp(), b_way, b_way.perp()]
        .into_iter()
        .all(|axis| (b - a).dot(axis).abs() <= reach(a_way, axis) + reach(b_way, axis))
}

/// The way the car drives round the town: one loop through the lanes, its
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
    /// How long it stands at each before it goes on, in seconds: no time at
    /// all, but short of every other crossing it comes to ([`PAUSE`]).
    waits: Vec<f64>,
    /// How far along the loop each is, in metres, and, last, the whole way
    /// round.
    distances: Vec<f64>,
    /// The lanes driven, in order, as their stretch and which way along it.
    driven: Vec<(usize, bool)>,
    /// How far behind the first car each car goes round, in seconds.
    behind: Vec<f64>,
    /// Where a car's back wheels are as its front ones pass each point, its
    /// wheels [`Self::base`] apart: following them round ([`trailing`]).
    backs: Vec<Vec2>,
    base: f32,
}

impl Route {
    /// The way round the roads in `paint`, if they make one, for `cars` cars
    /// that each take the room `car` takes on the road.
    pub(crate) fn read(paint: &Paint, car: &Footprint, cars: usize) -> Option<Self> {
        let network = network(paint);
        let lanes = lanes(&network);
        // Every turn from a lane into another where they join, but back down
        // its own stretch, and whether the car takes it gently ([`turning`]).
        let mut turns: std::collections::BTreeMap<(usize, usize), (Turn, bool)> = Default::default();
        for (a, this) in lanes.iter().enumerate() {
            for (b, next) in lanes.iter().enumerate() {
                let back = this.stretch == next.stretch && this.ahead != next.ahead;
                // Back the way it came only at the end of a road.
                let dead_end = lanes.iter().filter(|lane| lane.from == this.to).count() == 1;
                if this.to == next.from && (!back || dead_end) {
                    turns.insert((a, b), turning(this, next, paint, car, &network));
                }
            }
        }
        // The lanes it can go round from any of them to any other, turning
        // gently, the most road of them: the rest it leaves be. In Hajun's
        // new town, the bus turns right into the ring round the pond and out
        // of it only swinging round on its back wheels, as a corner a right
        // turn goes round is sharp and the roads 12 m wide, and the inner
        // half of the ring the other way round it, clockwise, it would get
        // onto and off only so.
        let driven = round_together(lanes.len(), &|a, b| turns.get(&(a, b)).is_some_and(|&(_, gentle)| gentle))
            .into_iter()
            .max_by(|a, b| {
                let length = |lot: &Vec<usize>| lot.iter().map(|&lane| lanes[lane].length()).sum::<f32>();
                length(a).total_cmp(&length(b)).then(b[0].cmp(&a[0]))
            })
            .unwrap_or_default();
        let gently = |a: usize, b: usize| turns.get(&(a, b)).is_some_and(|&(_, gentle)| gentle);
        let any = |a: usize, b: usize| turns.contains_key(&(a, b));
        let every: Vec<usize> = (0..lanes.len()).collect();
        let order = plan(&lanes, &driven, network.joins.len(), &gently)
            .or_else(|| plan(&lanes, &driven, network.joins.len(), &any))
            .or_else(|| plan(&lanes, &every, network.joins.len(), &any))?;
        let mut route = Self::through(&lanes, &order, paint, car, &turns);
        route.behind = route.spread(cars, car);
        Some(route)
    }

    /// The loop through `lanes` in `order`, turning from each into the next,
    /// on the roads in `paint`, for a car that takes the room `car` takes.
    fn through(
        lanes: &[Lane],
        order: &[usize],
        paint: &Paint,
        car: &Footprint,
        turned: &std::collections::BTreeMap<(usize, usize), (Turn, bool)>,
    ) -> Self {
        let count = order.len();
        let turns: Vec<Turn> = (0..count)
            .map(|index| {
                let (this, next) = (order[index], order[(index + 1) % count]);
                turned.get(&(this, next)).map(|(turn, _)| turn.clone()).unwrap_or_else(|| Turn {
                    leave: lanes[this].room.1,
                    join: lanes[next].room.0,
                    points: Vec::new(),
                })
            })
            .collect();
        let mut line: Vec<Vec2> = Vec::new();
        let mut tops: Vec<f32> = Vec::new();
        // The fastest it goes from each point of it: along a lane, as fast as
        // the lane allows, and turning, its top speed.
        let mut limits: Vec<f32> = Vec::new();
        for (index, &lane) in order.iter().enumerate() {
            let lane = &lanes[lane];
            let start = turns[(index + count - 1) % count].join;
            line.extend(stretch(&lane.points, false, start, lane.length() - turns[index].leave));
            limits.resize(line.len(), lane.speed);
            line.extend_from_slice(&turns[index].points);
            limits.resize(line.len(), roundtown_net::cars::TOP_SPEED);
            tops.resize(line.len(), lane.top);
        }
        // Evenly spaced round it, each on the road under it.
        let so_far = along(&line, true);
        let whole = *so_far.last().unwrap();
        let count = (whole / SPACING).round().max(3.0) as usize;
        let mut points: Vec<Vec3> = Vec::with_capacity(count);
        let mut fastest: Vec<f32> = Vec::with_capacity(count);
        for step in 0..count {
            let s = step as f32 * whole / count as f32;
            let at = point_at(&line, &so_far, s);
            let near = so_far.partition_point(|&d| d <= s).saturating_sub(1).min(tops.len() - 1);
            let top = tops[near];
            let last = points.last().map_or(top, |point| point.y);
            let ground = paint.road.floor(at, top + OVER_THE_ROAD).unwrap_or(last);
            points.push(Vec3::new(at.x, ground, at.y));
            fastest.push(limits[near]);
        }
        // Where it stands, short of every other crossing it comes to, the
        // front of it [`STOP_SHORT`] short, its front that far ahead of where
        // the loop has it: as far as the middle of its wheels, which is where
        // the loop has it, is behind it.
        let crossings = crossings_painted(paint.markings);
        let on: Vec<bool> = points
            .iter()
            .map(|point| crossings.iter().any(|crossing| crossing.holds(point.xz())))
            .collect();
        let short = ((car.ahead + car.half.x + STOP_SHORT) / (whole / count as f32)).round() as usize % count;
        let mut waits = vec![0.0; count];
        for (passed, at) in (0..count).filter(|&at| on[at] && !on[(at + count - 1) % count]).enumerate() {
            if passed % 2 == 0 {
                waits[(at + count - short) % count] = PAUSE;
            }
        }
        let ways: Vec<Vec2> = (0..count)
            .map(|at| {
                let (before, after) = (points[(at + count - 1) % count], points[(at + 1) % count]);
                (after - before).xz().normalize_or(Vec2::X)
            })
            .collect();
        let fronts: Vec<Vec2> = points.iter().map(|point| point.xz()).collect();
        let backs = trailing(&fronts, true, car.base).into_iter().map(|(_, back)| back).collect();
        let mut route = Self {
            points,
            ways,
            times: Vec::new(),
            waits,
            distances: Vec::new(),
            driven: order.iter().map(|&lane| (lanes[lane].stretch, lanes[lane].ahead)).collect(),
            behind: vec![0.0],
            backs,
            base: car.base,
        };
        route.drive(&fastest);
        route
    }

    /// How far behind the first car each of `cars` cars goes round, in whole
    /// seconds, for cars that take the room `car` takes: an even share of the
    /// loop behind the one before, or, where that would have two of them meet
    /// where the loop crosses itself, or passing each other, the nearest to
    /// it, a second at a time, that keeps every two [`APART`] all the way
    /// round; or failing that, that keeps them out of each other. The two
    /// buses, turning wide into the middle of the road (`turning`), pass the
    /// other in the lane beside within a metre.
    fn spread(&self, cars: usize, car: &Footprint) -> Vec<f64> {
        let lap = self.lap();
        // Where a car is every [`LOOK_EVERY`] round the loop, worked out once:
        // a car `behind` seconds behind the first is where the first was then.
        let looks = (lap / LOOK_EVERY).round() as usize;
        let stands: Vec<(Vec2, Vec2)> = (0..looks)
            .map(|look| {
                let (wheels, way, _) = self.on_wheels(look as f64 * LOOK_EVERY, car.base);
                (wheels.xz() + way * car.ahead, way)
            })
            .collect();
        // Whether two cars, one `a` seconds behind the first and the other
        // `b`, ever come within `apart` of each other.
        let meet = |a: f64, b: f64, apart: f32| {
            let shift = (((b - a) / LOOK_EVERY).round() as i64).rem_euclid(looks as i64) as usize;
            (0..looks).any(|look| {
                let ((one, one_way), (other, other_way)) = (stands[look], stands[(look + shift) % looks]);
                boxes_meet(one, one_way, other, other_way, car.half + apart * 0.5)
            })
        };
        let mut behind: Vec<f64> = Vec::with_capacity(cars);
        for number in 0..cars {
            let even = (lap * number as f64 / cars as f64).round();
            let most = (lap / cars as f64 * 0.5).floor() as i64;
            let tries = || {
                std::iter::once(0)
                    .chain((1..=most).flat_map(|off| [off, -off]))
                    .map(|off| even + off as f64)
            };
            let clear = |apart: f32| tries().find(|&mine| behind.iter().all(|&other| !meet(mine, other, apart)));
            behind.push(clear(APART).or_else(|| clear(0.0)).unwrap_or_else(|| {
                warn!("car {number} meets another wherever on the loop it goes");
                even
            }));
        }
        behind
    }

    /// How far behind the first car the car numbered `car` goes round, in
    /// seconds.
    pub(crate) fn behind(&self, car: u8) -> f64 {
        self.behind.get(usize::from(car)).copied().unwrap_or(0.0)
    }

    /// Works out when the car passes each point: as fast as it can take how
    /// sharply the road turns there, never faster than `fastest` there, its
    /// top speed but along an open road, standing still where it waits, and
    /// speeding up and slowing down gently between. Then a little faster or
    /// slower all the way round, for the loop, its waits and all, to take a
    /// whole number of seconds.
    fn drive(&mut self, fastest: &[f32]) {
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
                round.min(fastest[at]).max(SLOWEST)
            })
            .collect();
        for (speed, &wait) in speeds.iter_mut().zip(&self.waits) {
            if wait > 0.0 {
                *speed = 0.0;
            }
        }
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
        // How long it takes to get to each point, going, and how long it has
        // stood still by then.
        let (mut going, mut stood, mut distances) =
            (Vec::with_capacity(count), Vec::with_capacity(count), Vec::with_capacity(count + 1));
        let (mut time, mut waited, mut distance) = (0.0f64, 0.0f64, 0.0f64);
        for at in 0..count {
            going.push(time);
            stood.push(waited);
            distances.push(distance);
            let pace = f64::from((speeds[at] + speeds[(at + 1) % count]) * 0.5).max(1e-3);
            time += f64::from(gaps[at]) / pace;
            waited += self.waits[at];
            distance += f64::from(gaps[at]);
        }
        let lap = (time + waited).round().max(waited.ceil() + 1.0);
        let quicker = (lap - waited) / time;
        let mut times: Vec<f64> = going.iter().zip(&stood).map(|(going, stood)| going * quicker + stood).collect();
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
        // Standing where it waits first, and only then on to the next point.
        let wait = self.waits[before];
        let span = self.times[after] - self.times[before] - wait;
        let share = if span > 0.0 { ((into - self.times[before] - wait) / span).clamp(0.0, 1.0) } else { 0.0 };
        let way = self.ways[before]
            .lerp(self.ways[after % count], share as f32)
            .normalize_or(self.ways[before]);
        let distance = self.distances[before] + (self.distances[after] - self.distances[before]) * share;
        let distance = laps * self.length() + distance;
        (self.points[before].lerp(self.points[after % count], share as f32), way, distance)
    }

    /// Where a car whose front and back wheels are `base` apart is `seconds`
    /// into its loop ([`Footprint`]): its front wheels half that ahead of
    /// where [`Self::at`] has the car, and its back ones following them
    /// ([`trailing`]). As the middle of its wheels, halfway between them, on
    /// the road; the way it faces, from its back wheels to its front ones;
    /// and how far it has come.
    pub(crate) fn on_wheels(&self, seconds: f64, base: f32) -> (Vec3, Vec2, f64) {
        let (at, way, distance) = self.at(seconds);
        if base <= 0.0 {
            return (at, way, distance);
        }
        let ahead = distance.rem_euclid(self.length()) + f64::from(base) * 0.5;
        let front = self.point_round(ahead);
        let back = self.back_round(ahead, base);
        ((front + back) * 0.5, (front - back).xz().normalize_or(way), distance)
    }

    /// Where the back wheels of a car `base` long between its wheels are,
    /// its front ones `distance` round the loop: following them, as worked
    /// out for the cars' own (`backs`), on the road. Any other car's are on
    /// the loop as far behind as its wheels are apart.
    fn back_round(&self, distance: f64, base: f32) -> Vec3 {
        let on_road = self.point_round(distance - f64::from(base));
        if (base - self.base).abs() > 1e-3 || self.backs.len() != self.points.len() {
            return on_road;
        }
        let distance = distance.rem_euclid(self.length());
        let count = self.points.len();
        let after = self.distances.partition_point(|&far| far <= distance).clamp(1, count);
        let before = after - 1;
        let span = self.distances[after] - self.distances[before];
        let share = if span > 0.0 { ((distance - self.distances[before]) / span).clamp(0.0, 1.0) } else { 0.0 };
        let back = self.backs[before].lerp(self.backs[after % count], share as f32);
        Vec3::new(back.x, on_road.y, back.y)
    }

    /// The point `distance` metres round the loop from its first point, gone
    /// round again as often as it takes.
    fn point_round(&self, distance: f64) -> Vec3 {
        let distance = distance.rem_euclid(self.length());
        let count = self.points.len();
        let after = self.distances.partition_point(|&far| far <= distance).clamp(1, count);
        let before = after - 1;
        let span = self.distances[after] - self.distances[before];
        let share = if span > 0.0 { ((distance - self.distances[before]) / span).clamp(0.0, 1.0) } else { 0.0 };
        self.points[before].lerp(self.points[after % count], share as f32)
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

    /// Half the length and half the width of a car as big as Hajun's first.
    pub(crate) const CAR: Vec2 = Vec2::new(3.1, 1.45);

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
        let route = Route::read(&paint, &Footprint::turning_about_its_middle(CAR), 2).expect("a way round");
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

    /// Round Town as `roundtownmap.glb` has it: its land, as the game reads
    /// it (`island::read_island`), its roads alone, and its paint.
    pub(crate) fn round_town() -> (Island, Island, Vec<Vec<[Vec3; 3]>>) {
        let parts = crate::island::glb::read("roundtownmap.glb");
        let made_of = |material: &str| -> Vec<Vec<[Vec3; 3]>> {
            parts
                .iter()
                .filter(|part| part.material.as_deref().is_some_and(|name| crate::island::is_material(name, material)))
                .map(|part| part.triangles.clone())
                .collect()
        };
        let land: Vec<[Vec3; 3]> = parts
            .iter()
            .filter(|part| {
                !part.skinned
                    && !part
                        .material
                        .as_deref()
                        .is_some_and(|name| crate::island::is_material(name, crate::island::WATER_MATERIAL))
            })
            .flat_map(|part| part.triangles.iter().copied())
            .collect();
        let road: Vec<[Vec3; 3]> = made_of(ROAD).into_iter().flatten().collect();
        (Island::new_for_tests(&land), Island::new_for_tests(&road), made_of(MARKING))
    }

    /// The bus, the longest of the cars, which the way round is worked out
    /// for.
    fn bus() -> Footprint {
        let cars = crate::car::tests::footprints();
        cars.iter().copied().max_by(|a, b| a.half.x.total_cmp(&b.half.x)).unwrap()
    }

    #[test]
    fn round_town_s_own_roads_meet_at_its_ring_and_round_its_roundabouts() {
        // Hajun's new town (2026-10-30): a ring round its pond that is two
        // roads with a strip of grass between, four straight roads out from
        // it, each to a roundabout, 8 m of road round an island 18 m across
        // (6 m until Hajun made them bigger, the same day), and a road round
        // the edge of the town from one roundabout to the next.
        let (land, road, markings) = round_town();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        let network = network(&paint);
        assert_eq!(network.joins.len(), 8, "{:?}", network.joins);
        assert_eq!(network.stretches.len(), 16);
        let rings: Vec<Ring> = network.rings.iter().flatten().copied().collect();
        assert_eq!(rings.len(), 4, "{:?}", network.rings);
        for ring in &rings {
            assert!((ring.inside - 9.1).abs() < 0.3 && (ring.outside - 17.1).abs() < 0.5, "{ring:?}");
        }
        for (join, ring) in network.joins.iter().zip(&network.rings) {
            // The roundabouts out at the edge, the four joins of the ring's
            // two halves with the straight roads between them, at the gaps.
            let out = join.length();
            if ring.is_some() {
                assert!((out - 194.0).abs() < 0.5, "{join}");
            } else {
                assert!((out - 57.4).abs() < 0.5, "{join}");
            }
        }
        for stretch in &network.stretches {
            assert!((stretch.left - 6.0).abs() < 0.3 && (stretch.right - 6.0).abs() < 0.3, "{stretch:?}");
        }
    }

    #[test]
    fn the_bus_drives_round_town_gently() {
        // Every lane of every road but where the bus would have to swing
        // round on its back wheels to get onto it: the inner half of the
        // ring, clockwise, which it gets onto and off only turning right
        // through a gap in the strip of grass. On the road all the way, never
        // turning back or more tightly than it can follow, its wheels on the
        // road but coming onto a roundabout and off it.
        let (land, road, markings) = round_town();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        let car = bus();
        let route = Route::read(&paint, &car, 2).expect("a way round the town");
        assert_eq!(route.lanes(), 28);
        let network = network(&paint);
        let count = route.points.len();
        for at in 0..count {
            let (a, b) = (route.points[at], route.points[(at + 1) % count]);
            assert!(a.distance(b) < SPACING * 1.2, "{a} to {b}");
            assert!(a.y.abs() < 0.03, "{a} not on the road");
            assert!(paint.on_road(a.xz(), 0.02), "{a} off the road");
            assert!(route.ways[at].dot(route.ways[(at + 1) % count]) > 0.9, "turned back at {a}");
        }
        let points: Vec<Vec2> = route.points.iter().map(|point| point.xz()).collect();
        assert!(tightest(&points) >= car.base * TIGHTEST * 0.95, "bends {} m round", tightest(&points));
        // Its wheels on the road, but onto the edge of an island.
        let on_road = |at: Vec2| {
            paint.on_road(at, 0.02)
                || network.joins.iter().zip(&network.rings).any(|(middle, ring)| {
                    ring.is_some_and(|ring| (ring.inside - APRON..=ring.outside + 1.0).contains(&at.distance(*middle)))
                })
        };
        // Coming onto the road round a roundabout's island and off it, 8 m
        // wide round an island 18 m across, its back wheels cut the square
        // corners where the roads meet it, by a metre at the most (0.85 m;
        // 1.05 m while it was 6 m wide, before Hajun made the roundabouts
        // bigger on 2026-10-30): the paving's, a step high, and on the edge
        // of the town, the sea's.
        let near_a_roundabout = |at: Vec2| {
            network.joins.iter().zip(&network.rings).any(|(middle, ring)| ring.is_some() && middle.distance(at) < 25.0)
        };
        let mut furthest: f32 = 0.0;
        for (wheels, way) in stands(&trailing(&points, true, car.base)) {
            for corner in &outline(car.wheels)[..4] {
                let at = wheels + way * corner.x + way.perp() * corner.y;
                if on_road(at) {
                    continue;
                }
                assert!(near_a_roundabout(at), "the bus's wheels go off the road at {at}");
                let off = (1..=60)
                    .map(|step| step as f32 * 0.05)
                    .find(|&far| (0..16).any(|k| on_road(at + Vec2::from_angle(k as f32 / 16.0 * TAU) * far)))
                    .unwrap_or(f32::INFINITY);
                furthest = furthest.max(off);
            }
        }
        assert!(furthest <= 1.2, "its wheels go {furthest} m off the road round a roundabout");
        println!("its wheels {furthest} m off the road at the most, round a roundabout");
        assert_eq!(route.lap().fract(), 0.0);
        assert!(route.fastest() <= roundtown_net::cars::TOP_SPEED * OPEN_ROAD * 1.01);
        println!("round town: {} m round in {} s, {} lanes", route.length().round(), route.lap(), route.lanes());
    }

    #[test]
    fn the_bus_waits_short_of_every_other_crossing_and_hurries_round_the_edge() {
        // Hajun, 2026-10-30: "make the bus go faster when going around the
        // outer circle (like 2x) and when in the inner circle, make it pause
        // before some of the crosswalks and then go".
        let (land, road, markings) = round_town();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        let crossings = crossings_painted(&markings);
        assert_eq!(crossings.len(), 4, "{crossings:?}");
        for crossing in &crossings {
            // At the corners of the ring round the pond, across both halves.
            assert!((crossing.middle.length() - 57.5).abs() < 0.5, "{crossing:?}");
            assert!((crossing.half - Vec2::new(4.0, 12.15)).abs().max_element() < 0.1, "{crossing:?}");
        }
        let car = bus();
        let route = Route::read(&paint, &car, 2).unwrap();
        let count = route.points.len();
        let on: Vec<bool> = route
            .points
            .iter()
            .map(|point| crossings.iter().any(|crossing| crossing.holds(point.xz())))
            .collect();
        // A crossing on every lane of the ring it drives, 12 a loop, and a
        // wait short of every other one.
        let passed = (0..count).filter(|&at| on[at] && !on[(at + count - 1) % count]).count();
        assert_eq!(passed, 12);
        let stops: Vec<usize> = (0..count).filter(|&at| route.waits[at] > 0.0).collect();
        assert_eq!(stops.len(), 6);
        let front = f64::from(car.ahead + car.half.x);
        for &stop in &stops {
            // Its front just short of the crossing, and standing still there
            // for as long as it waits, before it goes on.
            let short = route.point_round(route.distances[stop] + front).xz();
            let past = route.point_round(route.distances[stop] + front + f64::from(STOP_SHORT + SPACING)).xz();
            assert!(!crossings.iter().any(|crossing| crossing.holds(short)), "its front on a crossing at {short}");
            assert!(crossings.iter().any(|crossing| crossing.holds(past)), "no crossing ahead of {short}");
            let arrived = route.times[stop];
            let (still, ..) = route.at(arrived + 0.1);
            let (yet, ..) = route.at(arrived + PAUSE - 0.1);
            let (gone, ..) = route.at(arrived + PAUSE + 1.0);
            assert!(still.distance(yet) < 1e-4 && gone.distance(yet) > 0.1, "{still} {yet} {gone}");
        }
        // Twice its top speed along the road round the edge of the town, and
        // only there.
        let top = roundtown_net::cars::TOP_SPEED;
        assert!(route.fastest() > top * OPEN_ROAD * 0.98, "{} m/s at the most", route.fastest());
        for at in 0..count {
            let speed = (route.distances[at + 1] - route.distances[at]) / (route.times[at + 1] - route.times[at]);
            assert!(speed <= f64::from(top) * 1.01 || route.points[at].xz().length() > 180.0, "{speed} m/s at {}", route.points[at]);
        }
        assert_eq!(route.lap().fract(), 0.0);
        println!("round town: {} m round in {} s", route.length().round(), route.lap());
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
                    room: (0.0, 0.0),
                    inside: 0.0,
                    speed: roundtown_net::cars::TOP_SPEED,
                })
            })
            .collect();
        let every: Vec<usize> = (0..lanes.len()).collect();
        assert!(circuit(&lanes, &every, 8, &|_, _| true).is_none());
        let order = plan(&lanes, &every, 8, &|_, _| true).expect("a loop with one stretch twice");
        assert_eq!(order.len(), 26);
        for (index, &lane) in order.iter().enumerate() {
            let (a, b) = (&lanes[lane], &lanes[order[(index + 1) % order.len()]]);
            assert_eq!(a.to, b.from);
            assert!(!(a.stretch == b.stretch && a.ahead != b.ahead), "turned back");
        }
    }

    /// How far round from the way `points` go into each to the way they go out
    /// of it, round the loop they make, anticlockwise from above taken as less
    /// than none.
    fn turns(points: &[Vec2]) -> Vec<f32> {
        let n = points.len();
        (0..n)
            .map(|at| {
                let (before, here, after) = (points[(at + n - 1) % n], points[at], points[(at + 1) % n]);
                (here - before).normalize().perp_dot((after - here).normalize())
            })
            .collect()
    }

    #[test]
    fn dashes_every_other_one_in_from_the_rest_make_a_round_road() {
        // A ring of 96 dashes 57 m out, every other one 11 cm in, as Hajun's
        // outer road's are: evened out, all of them are on one circle, to a
        // centimetre, and the line through them turns one way all the way
        // round, by as much at every point, near enough.
        let dashes: Vec<Vec2> = (0..96)
            .map(|k| Vec2::from_angle(k as f32 / 96.0 * TAU) * (57.0 - 0.11 * (k % 2) as f32))
            .collect();
        let even = even_out(&dashes, true);
        let radius = even.iter().map(|at| at.length()).sum::<f32>() / 96.0;
        assert!((radius - 56.945).abs() < 0.01, "{radius}");
        for at in &even {
            assert!((at.length() - radius).abs() < 0.001, "{at} off the circle {radius}");
        }
        let round = turns(&smooth(&even, true));
        let mean = round.iter().sum::<f32>() / round.len() as f32;
        for turn in &round {
            assert!((turn - mean).abs() < mean.abs() * 0.1, "turned {turn}, not {mean}");
        }
        // Through the dashes as they are, it went back and forth.
        assert!(turns(&smooth(&dashes, true)).iter().any(|turn| turn.signum() != mean.signum()));
        // A straight road of them, every other one 2 cm to the side, is
        // straight, and its ends stay where they were.
        let dashes: Vec<Vec2> = (0..12).map(|k| Vec2::new(k as f32 * 3.0, 0.02 * (k % 2) as f32)).collect();
        let even = even_out(&dashes, false);
        assert_eq!((even[0], even[11]), (dashes[0], dashes[11]));
        for at in &even[2..10] {
            assert!((at.y - 0.01).abs() < 0.002, "{at}");
        }
    }

    #[test]
    fn round_each_of_round_town_s_rings_the_car_turns_one_way() {
        // Round each lane of the ring round the pond the bus turns the one
        // way, never back: through every dash it wove in and out by a few
        // centimetres, and the old town's cars' fronts swung out and in all
        // the way round (Hajun, 2026-10-23: "it shouldnt rotate outward
        // ever").
        let (land, road, markings) = round_town();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        let route = Route::read(&paint, &bus(), 2).unwrap();
        let points: Vec<Vec2> = route.points.iter().map(|point| point.xz()).collect();
        // Over a metre and a half at a time: the lanes are drawn through
        // points a metre apart, and the loop through points half that, which
        // between them wave in and out by a millimetre.
        let n = points.len();
        let round: Vec<f32> = (0..n)
            .map(|at| {
                let (before, here, after) = (points[(at + n - 3) % n], points[at], points[(at + 3) % n]);
                (here - before).normalize().perp_dot((after - here).normalize())
            })
            .collect();
        // The lanes of the ring's two halves, 2.65 m and 2.9 m either side
        // of their middles, 49.9 and 64.8 m out.
        let network = network(&paint);
        let near_a_join = |at: Vec2| network.joins.iter().any(|join| join.distance(at) < 30.0);
        // The lanes of the ring round the pond: of its two halves, 49.9 and
        // 64.8 m out, either side of their middles.
        let rings: Vec<f32> = lanes(&network)
            .iter()
            .map(|lane| lane.points[lane.points.len() / 2].length())
            .filter(|&out| out < 80.0)
            .collect();
        let lane_at = |at: Vec2| {
            rings
                .iter()
                .position(|&lane| (at.length() - lane).abs() < 0.05)
                .filter(|_| !near_a_join(at))
        };
        let mut runs: Vec<Vec<usize>> = Vec::new();
        for at in 0..points.len() {
            match (lane_at(points[at]), runs.last_mut()) {
                (Some(lane), Some(run)) if at > 0 && run.last() == Some(&(at - 1)) && lane_at(points[at - 1]) == Some(lane) => {
                    run.push(at)
                }
                (Some(_), _) => runs.push(vec![at]),
                _ => {}
            }
        }
        let runs: Vec<&[usize]> = runs.iter().filter(|run| run.len() > 10).map(|run| &run[3..run.len() - 3]).collect();
        assert!(runs.len() >= 8, "{} runs round the ring", runs.len());
        for run in runs {
            let way = round[run[0]].signum();
            for &at in run {
                assert!(round[at].signum() == way, "turned back at {}", points[at]);
            }
        }
    }

    #[test]
    fn round_town_s_buses_swing_into_nothing_and_never_meet() {
        // A bus, 12.5 m long on 12 m roads, cannot keep all of itself on them
        // turning from one into another, but what of it swings out over the
        // kerb swings over nothing taller than the kerb: no lamp, wall or
        // tree. Round a roundabout's island it swings out over the island,
        // which is a step high, and over the edge of the town into the sea.
        let (land, road, markings) = round_town();
        let paint = Paint { land: &land, road: &road, markings: &markings };
        let cars = crate::car::tests::footprints();
        let car = bus();
        let route = Route::read(&paint, &car, cars.len()).unwrap();
        let points: Vec<Vec2> = route.points.iter().map(|point| point.xz()).collect();
        for car in &cars {
            for (wheels, way) in stands(&trailing(&points, true, car.base)) {
                let middle = wheels + way * car.ahead;
                for corner in outline(car.half) {
                    let at = middle + way * corner.x + way.perp() * corner.y;
                    assert!(paint.on_road(at, 0.02) || !land.blocked(at, 0.4, 4.0), "{car:?} swings into something at {at}");
                }
            }
        }
        // And the two, half the loop or so apart, never meet where it crosses
        // itself, nor passing each other.
        let behind = route.behind(1);
        assert!((behind - route.lap() * 0.5).abs() < route.lap() * 0.25, "{behind} s behind");
        let looks = (route.lap() / 0.1) as usize;
        for look in 0..looks {
            let seconds = look as f64 * 0.1;
            let (one, one_way, _) = route.on_wheels(seconds, car.base);
            let (other, other_way, _) = route.on_wheels(seconds - behind, car.base);
            let (one, other) = (one.xz() + one_way * car.ahead, other.xz() + other_way * car.ahead);
            assert!(!boxes_meet(one, one_way, other, other_way, car.half), "they meet at {one}");
        }
    }

    #[test]
    fn the_back_wheels_follow_the_front_ones_inside_a_bend() {
        // Round a circle 20 m round, a car whose wheels are 6.5 m apart has
        // its back wheels on a circle inside it, as far round as the two can
        // be that far apart, and they go the way it faces: never sideways.
        let base = 6.5;
        let circle: Vec<Vec2> = (0..400).map(|k| Vec2::from_angle(k as f32 / 400.0 * TAU) * 20.0).collect();
        let wheels = trailing(&circle, true, base);
        let inside = (20.0f32 * 20.0 - base * base).sqrt();
        for (k, &(front, back)) in wheels.iter().enumerate() {
            assert!((front.distance(back) - base).abs() < 1e-3);
            assert!((back.length() - inside).abs() < 0.05, "{} for {inside}", back.length());
            let (next_back, facing) = (wheels[(k + 1) % wheels.len()].1, (front - back).normalize());
            assert!((next_back - back).normalize().dot(facing) > 0.999, "slid sideways at {back}");
        }
        // Along a straight, they come back onto the line behind the front.
        let line: Vec<Vec2> = (0..200).map(|k| Vec2::new(k as f32 * 0.5, 0.0)).collect();
        let (front, back) = *trailing(&line, false, base).last().unwrap();
        assert!(back.distance(front - Vec2::X * base) < 1e-3);
    }

}
