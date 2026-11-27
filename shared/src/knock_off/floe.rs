//! The island of ice each game of Knock Off is played on (since 2026-11-27;
//! Hajun: "right now, it always forms the same shape of island (circle). i
//! want it to be so that each game, a truly random shape of island forms.
//! check out blobmaker.app, and set both the two sliders all the way to the
//! right side and click random generate, i want that kind of style of shape
//! to be generated each game (truly random based on code, not just selecting
//! from a pre-generateed random array)"): a [`Floe`], thrown afresh for every
//! table from a seed nobody chose ([`super::Island`]), as blobmaker.app throws
//! its blobs.
//!
//! **Blobmaker's own way**, read out of the code its page runs on
//! 2026-11-27, with both its sliders all the way right: twelve points evenly
//! round a middle (its "complexity" at its most, 12), each as far out as a
//! throw between all but nothing and the most (its "contrast" at its most, 8:
//! `100 - (12.5 * 8 - 0.01)` to `100` of 100), and the outline the closed
//! curve those twelve pull smooth (d3's `curveBasisClosed`, a closed cubic
//! B-spline, which passes near each point without going through it). Every
//! blob it throws has a size of its own in its box; every island here is
//! made as big as the disc of [`AREA_RADIUS`] (mine), so that every game has
//! as much ice to play on, whatever its shape.
//!
//! **To the bit.** Whether someone's middle is past the edge is the slide's
//! to say on every machine alike ([`super::Slide`]), so the outline is made
//! by `+ - * /` and the square root alone, from the seed by whole numbers
//! ([`Dice`]), its points' ways out written down rather than worked out with
//! `cos` and `sin`, and it is asked of by them alone ([`Floe::holds`]): the
//! server sends the seed and every device makes the very same outline. What
//! the rest asks of it (how far in from the edge a spot is, where the ice is
//! deepest, where to stand someone) is the asking machine's own, and goes by
//! nothing it has to agree on with another.
//!
//! **One outline, every size.** The ice breaks from its edge at the end of
//! every turn ([`super::size_in`]); what is left is the same outline made
//! smaller toward the middle it was thrown round (mine). Every point of the
//! outline can be seen from that middle (it is star-shaped round it, which
//! is checked as it is made), so that what is left lies within what was, and
//! whichever way someone looks out from the middle, the edge is one place.

use std::f32::consts::{PI, TAU};

use bevy::math::Vec2;

use super::{BODY, ON_THE_LINE};
use crate::bots::Dice;

/// How many points blobmaker pulls its outline round: its "complexity" at
/// its most (it goes from 3 to 12).
pub const POINTS: usize = 12;

/// The least a point may be out from the middle, as a share of the most:
/// blobmaker's "contrast" at its most (8, of 1 to 8) throws each between
/// 0.01 and 100 of 100.
const LEAST_OUT: f32 = 0.0001;

/// How many points of the outline each span of its curve, between two of
/// blobmaker's points, is drawn with; and so how many there are round the
/// whole of it ([`RIM`]): close enough that the outline is a smooth curve
/// to look at and to play on, 4 to 15 cm apart round an island of the usual
/// size.
const STEPS: usize = 16;
pub const RIM: usize = POINTS * STEPS;

/// How big every island is, in metres: as big as the disc this far out
/// (mine). The disc Knock Off was played on until 2026-11-27 was 12 m out;
/// an island of blobmaker's reaches about one and a half times further from
/// its middle than the disc as big as it does, and is fitted on the screen
/// by how far it reaches, so the disc 12 m out would have stood its players
/// half as big again further off. At 10 a game is fitted from as far off as
/// it was, or a little further.
pub const AREA_RADIUS: f32 = 10.0;

/// How far in from the edge someone who was off the ice is stood back on
/// it, in metres (mine): half a body, so that their middle is not on the
/// very line.
pub const STOOD_BACK: f32 = 0.5 * BODY;

/// How the outline's points are filed by which way they lie from the middle
/// ([`turn_of`]), to find the two a way out passes between at once: in
/// buckets of a 64th of a quarter turn each.
const QUARTER_BUCKETS: f32 = 64.0;
const BUCKETS: usize = 256;

/// Which way each of blobmaker's points lies from the middle: a twelfth of a
/// turn apart, from +x toward +z (blobmaker starts at the top of its box and
/// goes clockwise; any way round throws the same shapes). Written out, never
/// worked out with `cos` and `sin`, which two machines may round apart.
const WAYS: [(f32, f32); POINTS] = [
    (1.0, 0.0),
    (0.866_025_4, 0.5),
    (0.5, 0.866_025_4),
    (0.0, 1.0),
    (-0.5, 0.866_025_4),
    (-0.866_025_4, 0.5),
    (-1.0, 0.0),
    (-0.866_025_4, -0.5),
    (-0.5, -0.866_025_4),
    (0.0, -1.0),
    (0.5, -0.866_025_4),
    (0.866_025_4, -0.5),
];

/// What the pulls of a span's four points add up to ([`pulls`]).
const WHOLE: f32 = 6.0 * (STEPS * STEPS * STEPS) as f32;

/// How much each of four of blobmaker's points in a row pulls the outline,
/// `step` of [`STEPS`] along the span between the middle two: a uniform
/// cubic B-spline's weights, in [`WHOLE`]ths, whole numbers that an `f32`
/// holds exactly.
fn pulls(step: usize) -> [f32; 4] {
    let (j, n) = (step as i32, STEPS as i32);
    [
        (n - j).pow(3),
        3 * j.pow(3) - 6 * n * j * j + 4 * n.pow(3),
        -3 * j.pow(3) + 3 * n * j * j + 3 * n * n * j + n.pow(3),
        j.pow(3),
    ]
    .map(|weight| weight as f32)
}

/// Which way `(x, z)` points from the middle, as a number that rises with
/// the angle from +x toward +z, from 0 to all but 4 for a whole turn: in
/// each quarter, how much of the way's length along the two axes is across
/// it. By `+ - /` alone, for the slide. Not a number for no way at all.
fn turn_of(x: f32, z: f32) -> f32 {
    let (ax, az) = (x.abs(), z.abs());
    let across = az / (ax + az);
    match (x >= 0.0, z >= 0.0) {
        (true, true) => across,
        (false, true) => 2.0 - across,
        (false, false) => 2.0 + across,
        (true, false) => 4.0 - across,
    }
}

/// The island of ice a game of Knock Off is played on: its outline at its
/// whole size, round the middle of the world, which it is made smaller
/// toward as it breaks ([`super::size_in`]).
#[derive(Clone, PartialEq, Debug)]
pub struct Floe {
    /// The outline, [`RIM`] points round it, anticlockwise as x turns
    /// toward z, the first the one whose way from the middle has the least
    /// [`turn_of`]; and each one's way from the middle, rising all round.
    rim: Vec<Vec2>,
    turns: Vec<f32>,
    /// For each bucket of ways ([`BUCKETS`]), the last point of the outline
    /// whose way is not past the bucket's first (or the first point).
    first: Vec<u16>,
    /// How long each side of the outline is, from each point to the next.
    lengths: Vec<f32>,
    /// The deepest spot of the ice, the furthest in from every edge, and how
    /// far in it is, at its whole size; and how far from the middle the
    /// outline reaches at the furthest.
    heart: Vec2,
    deepest: f32,
    furthest: f32,
}

impl Floe {
    /// An island of blobmaker's, thrown from `seed`: the same seed, the same
    /// island, to the bit, on every machine. Should twelve throws ever make
    /// an outline that cannot be seen whole from its middle (none did in
    /// thousands tried), the next twelve are thrown, and so on; and should
    /// the dice somehow never make one, a round island.
    pub fn thrown(seed: u64) -> Self {
        let mut dice = Dice::new(seed);
        for _ in 0..64 {
            let out: [f32; POINTS] = std::array::from_fn(|_| LEAST_OUT + (1.0 - LEAST_OUT) * dice.roll());
            if let Some(floe) = Self::of(&out, AREA_RADIUS) {
                return floe;
            }
        }
        Self::round(AREA_RADIUS)
    }

    /// A round island as big as the disc `radius` out (its outline within
    /// a tenth of a per cent of that disc's), for trying things on: made as
    /// every island is, of blobmaker's twelve points, all as far out.
    pub fn round(radius: f32) -> Self {
        Self::of(&[1.0; POINTS], radius).expect("a round island can be seen whole from its middle")
    }

    /// The island whose twelve points are each `out[i]` out (as shares of
    /// the most) along [`WAYS`], as big as the disc `area_radius` out; none
    /// if it cannot be seen whole from its middle.
    pub fn of(out: &[f32; POINTS], area_radius: f32) -> Option<Self> {
        let points: [(f32, f32); POINTS] = std::array::from_fn(|i| (out[i] * WAYS[i].0, out[i] * WAYS[i].1));
        let mut rim: Vec<(f32, f32)> = Vec::with_capacity(RIM);
        for i in 0..POINTS {
            let four = [points[(i + POINTS - 1) % POINTS], points[i], points[(i + 1) % POINTS], points[(i + 2) % POINTS]];
            for step in 0..STEPS {
                let pull = pulls(step);
                let x = (pull[0] * four[0].0 + pull[1] * four[1].0 + pull[2] * four[2].0 + pull[3] * four[3].0) / WHOLE;
                let z = (pull[0] * four[0].1 + pull[1] * four[1].1 + pull[2] * four[2].1 + pull[3] * four[3].1) / WHOLE;
                rim.push((x, z));
            }
        }
        // As big as the disc: its area, twice over, round the outline.
        let mut twice = 0.0f32;
        for i in 0..RIM {
            let (a, b) = (rim[i], rim[(i + 1) % RIM]);
            twice += a.0 * b.1 - b.0 * a.1;
        }
        if twice.is_nan() || twice <= 0.0 || !area_radius.is_finite() || area_radius <= 0.0 {
            return None;
        }
        let scale = (2.0 * PI * area_radius * area_radius / twice).sqrt();
        let mut rim: Vec<Vec2> = rim.into_iter().map(|(x, z)| Vec2::new(x * scale, z * scale)).collect();
        // From the point whose way has the least turn, so that the ways rise
        // all round.
        let turn = |at: Vec2| turn_of(at.x, at.y);
        let least = (0..RIM).min_by(|&a, &b| turn(rim[a]).total_cmp(&turn(rim[b])))?;
        rim.rotate_left(least);
        let turns: Vec<f32> = rim.iter().map(|&at| turn(at)).collect();
        // Seen whole from the middle: each point a little further round than
        // the one before it, all the way round and back to the first.
        for i in 0..RIM {
            let (a, b) = (rim[i], rim[(i + 1) % RIM]);
            let turning = a.x * b.y - a.y * b.x;
            let next_on = i + 1 == RIM || (turns[i + 1].is_finite() && turns[i + 1] > turns[i]);
            if turning.is_nan() || turning <= 0.0 || !turns[i].is_finite() || !next_on {
                return None;
            }
        }
        let first = (0..BUCKETS)
            .map(|bucket| {
                let from = bucket as f32 / QUARTER_BUCKETS;
                turns.iter().rposition(|&turn| turn <= from).unwrap_or(0) as u16
            })
            .collect();
        let lengths = (0..RIM)
            .map(|i| {
                let (a, b) = (rim[i], rim[(i + 1) % RIM]);
                let (ex, ez) = (b.x - a.x, b.y - a.y);
                (ex * ex + ez * ez).sqrt()
            })
            .collect();
        let furthest = rim.iter().map(|at| at.length()).fold(0.0, f32::max);
        let mut floe = Self {
            rim,
            turns,
            first,
            lengths,
            heart: Vec2::ZERO,
            deepest: 0.0,
            furthest,
        };
        (floe.heart, floe.deepest) = floe.find_the_heart();
        Some(floe)
    }

    /// Its outline at its whole size, [`RIM`] points, anticlockwise as x
    /// turns toward z.
    pub fn rim(&self) -> &[Vec2] {
        &self.rim
    }

    /// Its outline at `size`.
    pub fn rim_at(&self, size: f32) -> impl Iterator<Item = Vec2> + '_ {
        self.rim.iter().map(move |&at| at * size)
    }

    /// How far from the middle its outline reaches at the furthest, at its
    /// whole size, in metres.
    pub fn furthest(&self) -> f32 {
        self.furthest
    }

    /// The middle of the ice at `size`, as anyone on it would take it: its
    /// deepest spot, the furthest in from every edge (where the ice it was
    /// thrown round is, the island may reach no more than a few metres
    /// out from on one side).
    pub fn middle(&self, size: f32) -> Vec2 {
        self.heart * size.max(0.0)
    }

    /// How far in from every edge the deepest spot of the ice is, at `size`,
    /// in metres.
    pub fn deepest(&self, size: f32) -> f32 {
        self.deepest * size.max(0.0)
    }

    /// Which side of the outline a way out from the middle passes through:
    /// the one from that point to the next, by `turn`, a way's
    /// [`turn_of`].
    fn side_at(&self, turn: f32) -> usize {
        let last = RIM - 1;
        if turn.is_nan() || turn < self.turns[0] || turn >= self.turns[last] {
            return last;
        }
        let bucket = ((turn * QUARTER_BUCKETS) as usize).min(BUCKETS - 1);
        let mut at = usize::from(self.first[bucket]);
        while at < last && self.turns[at + 1] <= turn {
            at += 1;
        }
        at
    }

    /// Whether `(x, z)`, on the island at its whole size, is on it, its
    /// middle no further past the edge than `slack`: by `+ - * /` alone, so
    /// that every machine says the same of the same numbers ([`super::Slide`]
    /// asks it, at every step). The very middle is on it; anything not a
    /// number, or past every number, is not.
    pub(super) fn holds_whole(&self, x: f32, z: f32, slack: f32) -> bool {
        let reach = x.abs() + z.abs();
        if reach == 0.0 {
            return true;
        }
        if !reach.is_finite() || reach <= 0.0 {
            return false;
        }
        let side = self.side_at(turn_of(x, z));
        let (a, b) = (self.rim[side], self.rim[(side + 1) % RIM]);
        let (ex, ez) = (b.x - a.x, b.y - a.y);
        // Inside an anticlockwise outline is on the left of each side.
        ex * (z - a.y) - ez * (x - a.x) >= -slack * self.lengths[side]
    }

    /// Whether `at` (across the ground: x, z) stands on the ice at `size`
    /// (a share of its whole, [`super::size_in`]): its middle no further
    /// past the edge than [`ON_THE_LINE`]. Nothing stands on ice that is
    /// gone. The same to the bit as the slide asks it ([`super::Slide`]).
    pub fn holds(&self, at: Vec2, size: f32) -> bool {
        if size.is_nan() || size <= 0.0 {
            return false;
        }
        let inv = 1.0 / size;
        self.holds_whole(at.x * inv, at.y * inv, ON_THE_LINE * inv)
    }

    /// The nearest point of the outline at its whole size to `at` (at its
    /// whole size too), and the side it is on.
    fn nearest_on_rim(&self, at: Vec2) -> (Vec2, usize) {
        let mut best = (f32::INFINITY, Vec2::ZERO, 0);
        for side in 0..RIM {
            let (a, b) = (self.rim[side], self.rim[(side + 1) % RIM]);
            let along = b - a;
            let share = ((at - a).dot(along) / along.length_squared().max(1e-12)).clamp(0.0, 1.0);
            let on = a + along * share;
            let far = on.distance_squared(at);
            if far < best.0 {
                best = (far, on, side);
            }
        }
        (best.1, best.2)
    }

    /// How far in from the edge of the ice at `size` `at` is, in metres:
    /// less than nothing past it; on ice that is gone, as far as it is from
    /// the middle, less than nothing.
    pub fn depth(&self, at: Vec2, size: f32) -> f32 {
        if size.is_nan() || size <= 0.0 {
            return -at.length();
        }
        let whole = at / size;
        let (on, _) = self.nearest_on_rim(whole);
        let far = on.distance(whole) * size;
        if self.holds_whole(whole.x, whole.y, 0.0) { far } else { -far }
    }

    /// The nearest spot to `at` on the ice at `size` at least `margin` in
    /// from its edge (no further in than the ice is deep, though): `at`
    /// itself if it is; or in from the nearest point of the edge, square to
    /// it; or, where the edge turns or narrows too much for that, as near
    /// as there is on the way from there to the deepest spot of the ice.
    /// Nowhere, on ice that is gone: the middle.
    pub fn nearest_inside(&self, at: Vec2, size: f32, margin: f32) -> Vec2 {
        if size.is_nan() || size <= 0.0 || !at.is_finite() {
            return Vec2::ZERO;
        }
        let margin = margin.clamp(0.0, 0.9 * self.deepest(size));
        if self.depth(at, size) >= margin {
            return at;
        }
        let (on, side) = self.nearest_on_rim(at / size);
        let along = (self.rim[(side + 1) % RIM] - self.rim[side]).normalize_or(Vec2::X);
        // In, square to the edge: the left of an anticlockwise outline.
        let on = on * size;
        let tried = on + Vec2::new(-along.y, along.x) * margin;
        if self.depth(tried, size) >= margin * 0.98 {
            return tried;
        }
        // Where the edge turns or narrows too much for that (the tip of a
        // thin arm): the nearest to `at` of the spots deep enough on the
        // first ring round that point of the edge that has any, ring after
        // ring a fifth of a metre further out.
        let reach = 2.0 * self.furthest * size;
        let mut ring = 1;
        while ring as f32 * 0.2 <= reach && ring <= 500 {
            let out = ring as f32 * 0.2;
            let count = (24 + 6 * ring).min(240);
            let found = (0..count)
                .map(|k| on + Vec2::from_angle(k as f32 * TAU / count as f32) * out)
                .filter(|&spot| self.depth(spot, size) >= margin)
                .min_by(|a, b| a.distance_squared(at).total_cmp(&b.distance_squared(at)));
            if let Some(found) = found {
                return found;
            }
            ring += 1;
        }
        self.middle(size)
    }

    /// The nearest spot to `near` on the ice at `size` at least `margin` in
    /// from its edge and at least `clear` from every one of `others`: the
    /// nearest on the ice ([`Floe::nearest_inside`]), if that is clear;
    /// otherwise the first clear one looking round it, ring after ring
    /// further out; or, nowhere being clear, that nearest on the ice.
    pub fn free_spot(&self, near: Vec2, others: &[Vec2], size: f32, margin: f32, clear: f32) -> Vec2 {
        let start = self.nearest_inside(near, size, margin);
        let margin = margin.clamp(0.0, 0.9 * self.deepest(size));
        let fits = |at: Vec2| others.iter().all(|other| other.distance_squared(at) >= clear * clear) && self.depth(at, size) >= margin;
        if fits(start) {
            return start;
        }
        let reach = 2.0 * self.furthest * size.max(0.0) + clear;
        let mut ring = 1;
        while (ring as f32) * 0.25 <= reach && ring <= 400 {
            let out = ring as f32 * 0.25;
            let count = (6 * ring).clamp(8, 96);
            for k in 0..count {
                let at = start + Vec2::from_angle(k as f32 * TAU / count as f32) * out;
                if fits(at) {
                    return at;
                }
            }
            ring += 1;
        }
        start
    }

    /// How far from the middle the outline is along `way` (of any length),
    /// at its whole size, in metres.
    pub fn reach_toward(&self, way: Vec2) -> f32 {
        let Some(way) = way.try_normalize() else {
            return 0.0;
        };
        let side = self.side_at(turn_of(way.x, way.y));
        let (a, b) = (self.rim[side], self.rim[(side + 1) % RIM]);
        let edge = b - a;
        // Where `way * t` meets the side from `a` to `b`.
        let across = way.perp_dot(edge);
        if across.abs() < 1e-9 {
            return a.length().min(b.length());
        }
        (a.perp_dot(edge) / across).max(0.0)
    }

    /// Where a slide from `at` along `way` (of length one) first takes its
    /// middle off the ice at `size`, in metres along it, if it goes no
    /// further than `reach`: none if it stays on all the way. (A slide that
    /// would cross a bay of the sea and come back onto the ice goes in where
    /// it first leaves it.)
    pub fn leaves(&self, at: Vec2, way: Vec2, reach: f32, size: f32) -> Option<f32> {
        if size.is_nan() || size <= 0.0 {
            return Some(0.0);
        }
        if !self.holds(at, size) {
            return Some(0.0);
        }
        let mut first: Option<f32> = None;
        for side in 0..RIM {
            let (a, b) = (self.rim[side] * size, self.rim[(side + 1) % RIM] * size);
            let edge = b - a;
            let across = way.perp_dot(edge);
            if across.abs() < 1e-9 {
                continue;
            }
            let to_a = a - at;
            let along = to_a.perp_dot(edge) / across;
            let on_edge = to_a.perp_dot(way) / across;
            if along > 0.0 && along <= reach && (0.0..=1.0).contains(&on_edge) && first.is_none_or(|first| along < first) {
                first = Some(along);
            }
        }
        first
    }

    /// The deepest spot of the island at its whole size, and how deep it is:
    /// the best of spots out along every few of the outline's points, then
    /// walked uphill to where it is deepest, ever finer. This machine's own
    /// sum, which nothing has to agree on with another's.
    fn find_the_heart(&self) -> (Vec2, f32) {
        let mut best = (Vec2::ZERO, self.depth(Vec2::ZERO, 1.0));
        for side in (0..RIM).step_by(4) {
            for k in 1..16 {
                let at = self.rim[side] * (k as f32 / 16.0);
                let deep = self.depth(at, 1.0);
                if deep > best.1 {
                    best = (at, deep);
                }
            }
        }
        let mut step = 0.05 * self.furthest;
        while step > 0.002 {
            let mut moved = false;
            for k in 0..8 {
                let at = best.0 + Vec2::from_angle(k as f32 * TAU / 8.0) * step;
                let deep = self.depth(at, 1.0);
                if deep > best.1 {
                    best = (at, deep);
                    moved = true;
                }
            }
            if !moved {
                step *= 0.5;
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every bit of an outline, as one number: two outlines that give the
    /// same are the same to the bit.
    fn bits(floe: &Floe) -> u64 {
        floe.rim.iter().flat_map(|at| [at.x.to_bits(), at.y.to_bits()]).fold(0xcbf2_9ce4_8422_2325, |hash, bits| {
            (hash ^ u64::from(bits)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    }

    /// Whether `at` is inside `rim` (a closed outline), by counting how often
    /// a line from it toward +x crosses it: an asking of its own, to check
    /// [`Floe::holds`] by.
    fn inside(rim: &[Vec2], at: Vec2) -> bool {
        let mut inside = false;
        for i in 0..rim.len() {
            let (a, b) = (rim[i], rim[(i + 1) % rim.len()]);
            if (a.y > at.y) != (b.y > at.y) && at.x < a.x + (b.x - a.x) * (at.y - a.y) / (b.y - a.y) {
                inside = !inside;
            }
        }
        inside
    }

    fn area(rim: &[Vec2]) -> f32 {
        0.5 * (0..rim.len()).map(|i| rim[i].perp_dot(rim[(i + 1) % rim.len()])).sum::<f32>()
    }

    #[test]
    fn every_island_is_as_big_as_the_disc_and_seen_whole_from_its_middle() {
        for seed in 0..400 {
            let floe = Floe::thrown(seed);
            assert_eq!(floe.rim.len(), RIM);
            let wanted = PI * AREA_RADIUS * AREA_RADIUS;
            assert!((area(&floe.rim) - wanted).abs() < 1e-3 * wanted, "seed {seed}: {} m²", area(&floe.rim));
            for i in 0..RIM {
                let (a, b) = (floe.rim[i], floe.rim[(i + 1) % RIM]);
                assert!(a.perp_dot(b) > 0.0, "seed {seed}: the outline turns back at {i}");
            }
            // As far out as blobmaker's reach from their middles, for their
            // size: never a disc, never further than twice its radius.
            assert!(floe.furthest > 1.05 * AREA_RADIUS && floe.furthest < 2.6 * AREA_RADIUS, "seed {seed}: {}", floe.furthest);
            // The deepest spot is on the ice, and as deep as it says.
            assert!(floe.holds(floe.heart, 1.0) && (floe.depth(floe.heart, 1.0) - floe.deepest).abs() < 1e-4);
            assert!(floe.deepest > 1.5, "seed {seed}: the deepest is only {} in", floe.deepest);
        }
    }

    #[test]
    fn no_two_tables_play_on_the_same_island_and_one_seed_is_one_island() {
        let mut seen = std::collections::HashSet::new();
        for seed in 0..300 {
            assert!(seen.insert(bits(&Floe::thrown(seed))), "seed {seed} threw an island thrown before");
        }
        assert_eq!(Floe::thrown(77), Floe::thrown(77));
        // Shapes of every kind: some all but round, some with arms reaching
        // twice as far as their bays go in.
        let spreads: Vec<f32> = (0..300)
            .map(|seed| {
                let floe = Floe::thrown(seed);
                let near = (0..360).map(|k| floe.reach_toward(Vec2::from_angle(k as f32 * TAU / 360.0))).fold(f32::MAX, f32::min);
                floe.furthest / near
            })
            .collect();
        assert!(spreads.iter().any(|&spread| spread < 2.0), "{spreads:?}");
        assert!(spreads.iter().filter(|&&spread| spread > 4.0).count() > 60, "{spreads:?}");
    }

    #[test]
    fn an_island_is_made_alike_to_the_bit_on_every_machine() {
        // What the island thrown from these seeds came to on the machine the
        // game was made on, as one number of all its bits: every machine
        // has to come to the same, the server and the phones alike. An
        // island that comes out otherwise has been changed: an app from
        // before would play every slide on another island than the server,
        // and `PROTOCOL_VERSION` goes up with it.
        let made: Vec<u64> = [1, 2, 0xDEAD_BEEF].map(|seed| bits(&Floe::thrown(seed))).to_vec();
        assert_eq!(made, GOLDEN_ISLANDS.to_vec(), "{made:#x?}");
        assert_eq!(bits(&Floe::round(12.0)), GOLDEN_ROUND, "{:#x}", bits(&Floe::round(12.0)));
    }

    /// The islands of [`an_island_is_made_alike_to_the_bit_on_every_machine`],
    /// as they came out on 2026-11-27.
    const GOLDEN_ISLANDS: [u64; 3] = [0x217c_f3ec_62f2_5722, 0xb3f7_c649_d43f_8d60, 0x8b2d_0151_fd09_b680];
    const GOLDEN_ROUND: u64 = 0xfab4_a63d_2e97_a3cd;

    #[test]
    fn whether_a_spot_is_on_the_ice_is_what_the_outline_says_at_every_size() {
        let mut dice = Dice::new(5);
        for seed in 0..60 {
            let floe = Floe::thrown(seed * 31 + 7);
            for size in [1.0, 11.0 / 12.0, 0.5, 0.1875, 0.0625] {
                let rim: Vec<Vec2> = floe.rim_at(size).collect();
                let far = floe.furthest * size * 1.2;
                for _ in 0..400 {
                    let at = Vec2::new(dice.between(-far, far), dice.between(-far, far));
                    let deep = floe.depth(at, size);
                    // Within a hair of the line either answer will do.
                    if deep.abs() < 2.0 * ON_THE_LINE {
                        continue;
                    }
                    assert_eq!(floe.holds(at, size), inside(&rim, at), "seed {seed} size {size} at {at}, {deep} in");
                    assert_eq!(floe.holds(at, size), deep > 0.0);
                }
            }
            // No ice at all holds nobody; the middle is held to the last.
            assert!(!floe.holds(Vec2::ZERO, 0.0));
            assert!(floe.holds(Vec2::ZERO, 0.0625));
            assert!(!floe.holds(Vec2::new(f32::NAN, 0.0), 1.0) && !floe.holds(Vec2::new(f32::INFINITY, 1.0), 1.0));
        }
    }

    #[test]
    fn the_ice_that_is_left_lies_within_what_was() {
        for seed in 0..80 {
            let floe = Floe::thrown(seed);
            let sizes: Vec<f32> = (1..=super::super::LAST_TURN + 1).map(super::super::size_in).collect();
            for pair in sizes.windows(2) {
                for at in floe.rim_at(pair[1]) {
                    // A hair in from each point of the smaller outline, on the
                    // bigger.
                    assert!(pair[1] <= 0.0 || floe.holds(at * 0.999, pair[0]), "seed {seed}: {at} at {} not on {}", pair[1], pair[0]);
                }
            }
        }
    }

    #[test]
    fn someone_off_the_ice_is_stood_back_on_it_nearest_where_they_went_in() {
        for seed in 0..60 {
            let floe = Floe::thrown(seed);
            for k in 0..24 {
                let way = Vec2::from_angle(k as f32 * TAU / 24.0);
                // Just past the edge, and well out in the sea.
                for past in [0.3, 4.0] {
                    let off = way * (floe.reach_toward(way) + past);
                    let back = floe.nearest_inside(off, 1.0, STOOD_BACK);
                    let deep = floe.depth(back, 1.0);
                    assert!(deep >= STOOD_BACK * 0.97, "seed {seed}: {back} is {deep} in");
                    // Near where they went in: no further than half the
                    // island, and as near as the edge is, give or take a
                    // turn of it.
                    let edge = -floe.depth(off, 1.0);
                    assert!(back.distance(off) <= edge + 3.0 * STOOD_BACK + 0.5 * floe.furthest, "seed {seed}: {off} to {back}");
                }
            }
            // Someone on the ice stays where they are.
            let on = floe.middle(1.0) * 0.5;
            assert_eq!(floe.nearest_inside(on, 1.0, 0.0), on);
        }
    }

    #[test]
    fn a_free_spot_is_on_the_ice_and_clear_of_everyone() {
        for seed in 0..40 {
            let floe = Floe::thrown(seed);
            let heart = floe.middle(1.0);
            let others = [heart, heart + Vec2::X * 0.7, heart - Vec2::Y * 0.9];
            let spot = floe.free_spot(heart, &others, 1.0, STOOD_BACK, 2.0 * BODY);
            assert!(floe.depth(spot, 1.0) >= STOOD_BACK * 0.97, "seed {seed}");
            assert!(others.iter().all(|other| other.distance(spot) >= 2.0 * BODY - 1e-4), "seed {seed}: {spot}");
            assert!(spot.distance(heart) < 3.0, "seed {seed}: {spot} is far from {heart}");
        }
    }

    #[test]
    fn a_slide_leaves_the_ice_where_its_middle_first_crosses_the_edge() {
        let round = Floe::round(12.0);
        let at = Vec2::new(7.0, 0.0);
        assert_eq!(round.leaves(at, Vec2::X, 4.0, 1.0), None);
        assert!((round.leaves(at, Vec2::X, 8.0, 1.0).unwrap() - 5.0).abs() < 0.05);
        assert!((round.leaves(at, -Vec2::X, 22.0, 1.0).unwrap() - 19.0).abs() < 0.05);
        // Half the ice, half as far.
        assert!((round.leaves(Vec2::ZERO, Vec2::Y, 22.0, 0.5).unwrap() - 6.0).abs() < 0.05);
        // On any island, where it leaves is where the slide's own steps
        // would find its middle off.
        for seed in 0..30 {
            let floe = Floe::thrown(seed);
            let from = floe.middle(1.0);
            for k in 0..16 {
                let way = Vec2::from_angle(k as f32 * TAU / 16.0 + 0.1);
                if let Some(left) = floe.leaves(from, way, 40.0, 1.0) {
                    assert!(floe.holds(from + way * (left - 0.02), 1.0), "seed {seed}");
                    assert!(!floe.holds(from + way * (left + 0.02), 1.0), "seed {seed}");
                }
            }
        }
    }

    #[test]
    fn a_round_island_is_the_disc() {
        let round = Floe::round(12.0);
        for k in 0..360 {
            let way = Vec2::from_angle(k as f32 * TAU / 360.0);
            let reach = round.reach_toward(way);
            assert!((reach - 12.0).abs() < 0.02, "{way}: {reach}");
            assert!(round.holds(way * 11.97, 1.0) && !round.holds(way * 12.03, 1.0), "{way}");
        }
        assert!(round.middle(1.0).length() < 0.05 && (round.deepest(1.0) - 12.0).abs() < 0.05);
    }
}
