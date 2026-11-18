//! Last Stand's map, `laststandmap.glb`, as numbers (read off Hajun's map of
//! 2026-11-18, lifted as the device lifts it, [`LIFT`]): the server has no
//! map, and everything a zombie walks over or round, and every place the
//! rules go by, is here. The game holds these to the map in a test of its
//! own ([`disagreements`]); a new export of the map that moves anything means
//! new numbers here, and the server deployed.

use bevy::math::{Vec2, Vec3};

use super::{ANCHOR, LIFT};
use crate::ANCHOR_RADIUS;
use crate::bridge_wars::Shop;

/// The stone disc: how far it reaches from the middle, and its top.
pub const STONE_RADIUS: f32 = 25.0;
pub const STONE_TOP: f32 = LIFT;
/// The ring of sand round it, a step under it: how far it reaches, which is
/// the rim, past which there is the sea, and its top.
pub const SAND_RADIUS: f32 = 37.5;
pub const SAND_TOP: f32 = LIFT - 0.2;
/// Where the sea begins, from the middle: the sand's edge, where the
/// zombies climb up out of the water (mine: "they come in waves from the
/// outer edges"; out of the void until the evening of 2026-11-18).
pub const RIM: f32 = SAND_RADIUS;
/// The furthest from the middle a zombie walks of itself, its middle half a
/// metre in from the rim: it never walks off into the sea but after someone
/// who is in it, and otherwise only a knock takes it over (mine).
pub const WALK_EDGE: f32 = SAND_RADIUS - 0.5;
/// The sea round the sand, since the evening of 2026-11-18 (Hajun: "in last
/// stand map, it shouldnt be void, but ocean with a tight invisible
/// circular boundary border so u can still go in the ocean but not too
/// far"): where the feet of a body in it come to, wading (the game's
/// `WATER_Y` less its `WATER_WADE`, the town's sea: the map is lifted, the
/// sea is not; a test in the game holds the two together); and the
/// furthest from the middle anybody's middle goes, a body, a zombie or a
/// giant, four metres out from the sand (mine): nobody goes past it, and
/// nothing knocks anyone past it. The sand's edge is too high to step up
/// onto from the sea: a body jumps up onto it, and a zombie climbs, as it
/// climbs out of the sea in the first place.
pub const SEA_BED: f32 = -1.08;
pub const SHORE: f32 = RIM + 4.0;

/// The anchor's slab, square round [`ANCHOR`]: half its width, and its top,
/// a step over the stone that anyone walks up (Hajun's map has a second,
/// narrower step on it, all under the crystal). The crystal on it, as wide
/// as Bridge Wars' anchors' pillars, from its middle: what a body is stopped
/// by, and what a blow reaches to.
pub const PLINTH_HALF: f32 = 1.0;
pub const PLINTH_TOP: f32 = LIFT + 0.25;
pub const CRYSTAL_RADIUS: f32 = ANCHOR_RADIUS;

/// The two generators' pads, either side of the anchor (`gen4`, `gen4.001`):
/// their middles across the ground, half their width, and their tops, which
/// lifted are Bridge Wars' own ([`PAD_TOP`](crate::PAD_TOP)), a step over
/// the stone.
pub const PADS: [Vec2; 2] = [Vec2::new(5.92, -0.16), Vec2::new(-6.08, -0.16)];
pub const PAD_HALF: f32 = 1.0;
pub const PAD_TOP: f32 = LIFT + 0.25;

/// The middles of the two shops' outlines (the material `activation`), in
/// front of their stands: the item shop's, at the red stand, toward -Z; the
/// upgrade shop's, at the blue one, toward +Z. Standing in an outline opens
/// its shop on the device.
pub const ITEM_SHOP: Vec2 = Vec2::new(0.13, -4.99);
pub const UPGRADE_SHOP: Vec2 = Vec2::new(0.13, 4.99);
/// How far from an outline's middle someone may be, where they last said
/// they stood, for the game to sell to them, in metres: the outline reaches
/// 1.45 m from its middle, and the rest is for the way they may have gone
/// since. Mine, and tighter than Bridge Wars' 5 ([`SHOP_SLACK`](crate::bridge_wars::SHOP_SLACK)):
/// here the two stands are 10 m apart with the anchor half way, which 5
/// would have had at both shops at once.
pub const SHOP_SLACK: f32 = 3.5;

/// Whether someone standing at `at` across the ground is near enough to
/// `shop`'s stand to buy from it, as the game checks it.
pub fn at_a_stand(at: Vec2, shop: Shop) -> bool {
    let middle = match shop {
        Shop::Items => ITEM_SHOP,
        Shop::Upgrades => UPGRADE_SHOP,
    };
    middle.distance(at) <= SHOP_SLACK
}

/// Something on the map that a body cannot walk through, across the
/// ground: a circle, or a box square with the map's axes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Obstacle {
    Circle { at: Vec2, radius: f32 },
    Box { at: Vec2, half: Vec2 },
}

impl Obstacle {
    /// How far `at` is outside it across the ground, in metres (less than
    /// nothing inside it), and which way is straight out from its nearest
    /// edge there.
    pub fn outside(&self, at: Vec2) -> (f32, Vec2) {
        match *self {
            Obstacle::Circle { at: middle, radius } => {
                let off = at - middle;
                let far = off.length();
                let out = if far > 1e-5 { off / far } else { Vec2::X };
                (far - radius, out)
            }
            Obstacle::Box { at: middle, half } => {
                let off = at - middle;
                let over = off.abs() - half;
                let sign = Vec2::new(off.x.signum(), off.y.signum());
                if over.x > 0.0 || over.y > 0.0 {
                    let past = over.max(Vec2::ZERO);
                    let far = past.length();
                    (far, (past / far) * sign)
                } else if over.x > over.y {
                    (over.x, Vec2::new(sign.x, 0.0))
                } else {
                    (over.y, Vec2::new(0.0, sign.y))
                }
            }
        }
    }
}

/// Everything on Hajun's map that stands in a body's way, each as the
/// circle or box round what of it is between a body's knees and its head
/// (read off the map with the game's own rule for a wall, steeper than 50
/// degrees): the anchor's crystal; the red stand, from its back board to
/// its front posts (it is open between them, where whoever shops stands, in
/// the outline in front of it), reaching on to the barrel beside it, a body
/// not fitting between the two; that barrel; the blue stand and its barrel
/// likewise; and the trunks of the two palm trees, round their lean. The
/// slab, the pads and the stone's step to the sand are steps a body walks
/// up ([`ground`]).
pub const OBSTACLES: [Obstacle; 7] = [
    Obstacle::Circle {
        at: ANCHOR,
        radius: CRYSTAL_RADIUS,
    },
    Obstacle::Box {
        at: Vec2::new(0.28, -5.475),
        half: Vec2::new(1.6, 0.465),
    },
    Obstacle::Circle {
        at: Vec2::new(2.45, -5.2),
        radius: 0.6,
    },
    Obstacle::Box {
        at: Vec2::new(0.28, 5.475),
        half: Vec2::new(1.6, 0.465),
    },
    Obstacle::Circle {
        at: Vec2::new(2.45, 5.3),
        radius: 0.6,
    },
    Obstacle::Circle {
        at: Vec2::new(5.75, -4.83),
        radius: 0.6,
    },
    Obstacle::Circle {
        at: Vec2::new(-6.06, 4.99),
        radius: 0.6,
    },
];

/// What a body stands on at `at` across the ground: the land ([`ground`]),
/// or the sea's bed out to [`SHORE`], where it wades; nothing past that,
/// where nobody goes.
pub fn floor(at: Vec2) -> Option<f32> {
    ground(at).or_else(|| (at.length() <= SHORE + 1e-3).then_some(SEA_BED))
}

/// Whether `at` across the ground is out in the sea, past the sand.
pub fn in_the_sea(at: Vec2) -> bool {
    at.length() > RIM
}

/// `at` brought no further from the middle than [`SHORE`]: the ring round
/// the sea that nobody is let past.
pub fn within_the_shore(at: Vec2) -> Vec2 {
    at.clamp_length_max(SHORE)
}

/// How high the land is at `at` across the ground, lifted: the anchor's
/// slab, a pad, the stone or the sand; none past the rim, where there is
/// the sea ([`floor`] has its bed). What things dropped lie on, and a
/// stick of dynamite: one that comes down in the sea sinks.
pub fn ground(at: Vec2) -> Option<f32> {
    let out = at.length();
    if out > SAND_RADIUS {
        return None;
    }
    if (at - ANCHOR).abs().max_element() <= PLINTH_HALF {
        return Some(PLINTH_TOP);
    }
    if PADS.iter().any(|pad| (at - *pad).abs().max_element() <= PAD_HALF) {
        return Some(PAD_TOP);
    }
    Some(if out <= STONE_RADIUS { STONE_TOP } else { SAND_TOP })
}

/// `at` moved out of whatever obstacle a body `radius` wide there would be
/// inside of ([`OBSTACLES`]), straight out from it, as little as takes; and
/// out of the next, where two meet, until it is in none.
pub fn keep_out(at: Vec2, radius: f32) -> Vec2 {
    let mut at = at;
    for _ in 0..4 {
        let mut moved = false;
        for obstacle in &OBSTACLES {
            let (far, out) = obstacle.outside(at);
            if far < radius - 1e-4 {
                at += out * (radius - far);
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    at
}

/// How steep a face may be and be ground, as the game counts it: the
/// upright share of its facing, for 50 degrees.
const GROUND_FACING: f32 = 0.643;
/// A body's band over the ground under it, which a wall in has to be inside
/// one of the [`OBSTACLES`]: from over a step to its head (mine).
const BAND: (f32, f32) = (0.3, 1.7);
/// How near the numbers here have to be to the map's, in metres: a place,
/// a height, and an obstacle's edge round a wall.
const NEAR: f32 = 0.1;
const LEVEL: f32 = 0.06;
const ROUND: f32 = 0.15;

/// What of Hajun's map disagrees with the numbers here, each said in a line;
/// none, as they agree. `parts` is every mesh of the map as the game reads it
/// (`glb::read_venue(Venue::LastStand)`): the name of its object, its
/// material's, and its triangles in the game's axes, lifted ([`LIFT`]). The
/// game's test of the map calls this; it is here, and not in that test, so
/// that the numbers and what holds them to the map are read together.
///
/// It holds: [`ground`] to the map's floors, a body's width in from every
/// edge, everywhere a zombie can walk; every wall in a body's band inside
/// one of the [`OBSTACLES`], and each of them round some wall; the pads, the
/// crystal and the two outlines where [`PADS`], [`ANCHOR`], [`ITEM_SHOP`]
/// and [`UPGRADE_SHOP`] say; and the pads' tops at [`PAD_TOP`].
pub fn disagreements(parts: &[(&str, Option<&str>, &[[Vec3; 3]])]) -> Vec<String> {
    let mut wrong = Vec::new();
    let (mut floors, mut walls): (Vec<[Vec3; 3]>, Vec<[Vec3; 3]>) = (Vec::new(), Vec::new());
    for &(node, _, triangles) in parts {
        if node == "waiting room" {
            continue;
        }
        for triangle in triangles {
            let normal = (triangle[1] - triangle[0]).cross(triangle[2] - triangle[0]);
            let length = normal.length();
            if length < 1e-9 {
                continue;
            }
            if normal.y / length >= GROUND_FACING {
                floors.push(*triangle);
            } else if (normal.y / length).abs() < GROUND_FACING {
                walls.push(*triangle);
            }
        }
    }
    // The floors, everywhere a body stands clear of every edge and every
    // obstacle: the highest one at a body's height or under it.
    let floor_at = |at: Vec2| -> Option<f32> {
        floors
            .iter()
            .filter_map(|triangle| height_in(triangle, at))
            .filter(|height| *height < 3.0)
            .fold(None, |best: Option<f32>, height| Some(best.map_or(height, |best| best.max(height))))
    };
    let edges = |at: Vec2| {
        let out = at.length();
        (out - STONE_RADIUS).abs() < 0.25
            || (out - SAND_RADIUS).abs() < 0.25
            || ((at - ANCHOR).abs().max_element() - PLINTH_HALF).abs() < ROUND
            || PADS.iter().any(|pad| ((at - *pad).abs().max_element() - PAD_HALF).abs() < ROUND)
            || OBSTACLES.iter().any(|obstacle| obstacle.outside(at).0 < 0.3)
    };
    let mut off = 0;
    for x in -38..=38 {
        for z in -38..=38 {
            let at = Vec2::new(x as f32, z as f32) + Vec2::splat(0.25);
            if edges(at) {
                continue;
            }
            let (map, ours) = (floor_at(at), ground(at));
            let agree = match (map, ours) {
                (Some(map), Some(ours)) => (map - ours).abs() <= LEVEL,
                (None, None) => true,
                _ => false,
            };
            if !agree {
                off += 1;
                if off <= 8 {
                    wrong.push(format!("the ground at {at} is {map:?} on the map, {ours:?} here"));
                }
            }
        }
    }
    if off > 8 {
        wrong.push(format!("and {} more places where the ground disagrees", off - 8));
    }
    // Every wall in a body's band is inside an obstacle, and each obstacle
    // is round some wall.
    let mut round = [false; OBSTACLES.len()];
    let mut loose = Vec::new();
    for triangle in &walls {
        let [a, b, c] = *triangle;
        for point in [a, b, c, (a + b) * 0.5, (b + c) * 0.5, (c + a) * 0.5, (a + b + c) / 3.0] {
            let across = Vec2::new(point.x, point.z);
            let under = ground(across).unwrap_or(SAND_TOP);
            if point.y <= under + BAND.0 || point.y >= under + BAND.1 {
                continue;
            }
            match OBSTACLES.iter().position(|obstacle| obstacle.outside(across).0 <= ROUND) {
                Some(which) => round[which] = true,
                None => loose.push(across),
            }
        }
    }
    if let Some(first) = loose.first() {
        wrong.push(format!("{} points of walls in a body's way are in no obstacle, the first at {first}", loose.len()));
    }
    for (which, found) in round.iter().enumerate() {
        if !found {
            wrong.push(format!("obstacle {which} ({:?}) is round no wall of the map", OBSTACLES[which]));
        }
    }
    // The pads, the crystal and the outlines.
    let middle = |points: &[Vec3]| -> Option<(Vec2, f32)> {
        let (low, high) = points
            .iter()
            .fold((Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)), |(low, high), point| (low.min(*point), high.max(*point)));
        (!points.is_empty()).then(|| (Vec2::new(low.x + high.x, low.z + high.z) * 0.5, high.y))
    };
    let mut pads: Vec<(Vec2, f32)> = parts
        .iter()
        .filter(|(node, ..)| node.starts_with("gen"))
        .filter_map(|(_, _, triangles)| middle(&triangles.iter().flatten().copied().collect::<Vec<Vec3>>()))
        .collect();
    pads.sort_by(|a, b| b.0.x.total_cmp(&a.0.x));
    if pads.len() != PADS.len() {
        wrong.push(format!("{} pads on the map, {} here", pads.len(), PADS.len()));
    }
    for (pad, (found, top)) in PADS.iter().zip(&pads) {
        if pad.distance(*found) > NEAR || (top - PAD_TOP).abs() > LEVEL {
            wrong.push(format!("a pad is at {found}, its top {top}, on the map: {pad} and {PAD_TOP} here"));
        }
    }
    let material = |name: &str| -> Vec<Vec3> {
        parts
            .iter()
            .filter(|(_, material, _)| material.is_some_and(|material| material.split('.').next() == Some(name)))
            .flat_map(|(_, _, triangles)| triangles.iter().flatten().copied())
            .collect()
    };
    match middle(&material("spawn")) {
        Some((found, _)) if found.distance(ANCHOR) <= NEAR => {}
        found => wrong.push(format!("the crystal is at {found:?} on the map, {ANCHOR} here")),
    }
    let outlines = material("activation");
    for (shop, front) in [(ITEM_SHOP, -1.0), (UPGRADE_SHOP, 1.0)] {
        let side: Vec<Vec3> = outlines.iter().copied().filter(|point| point.z * front > 0.0).collect();
        match middle(&side) {
            Some((found, _)) if found.distance(shop) <= NEAR => {}
            found => wrong.push(format!("a shop's outline is at {found:?} on the map, {shop} here")),
        }
    }
    wrong
}

/// How high `triangle` is over `at` across the ground, if it is over it.
fn height_in(triangle: &[Vec3; 3], at: Vec2) -> Option<f32> {
    let [a, b, c] = *triangle;
    let (low, high) = (a.min(b).min(c), a.max(b).max(c));
    if at.x < low.x - 1e-4 || at.x > high.x + 1e-4 || at.y < low.z - 1e-4 || at.y > high.z + 1e-4 {
        return None;
    }
    let (ab, ac, ap) = (Vec2::new(b.x - a.x, b.z - a.z), Vec2::new(c.x - a.x, c.z - a.z), at - Vec2::new(a.x, a.z));
    let area = ab.perp_dot(ac);
    if area.abs() < 1e-9 {
        return None;
    }
    let (u, v) = (ap.perp_dot(ac) / area, ab.perp_dot(ap) / area);
    (u >= -1e-4 && v >= -1e-4 && u + v <= 1.0 + 1e-4).then(|| a.y + u * (b.y - a.y) + v * (c.y - a.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ground_is_stone_then_sand_then_nothing_with_steps_on_it_a_body_walks_up() {
        assert_eq!(ground(Vec2::new(12.0, 3.0)), Some(STONE_TOP));
        assert_eq!(ground(Vec2::new(0.0, 30.0)), Some(SAND_TOP));
        assert_eq!(ground(Vec2::new(-27.0, -27.0)), None);
        assert_eq!(ground(Vec2::new(-26.0, -26.0)), Some(SAND_TOP));
        assert_eq!(ground(ANCHOR + Vec2::new(0.9, -0.9)), Some(PLINTH_TOP));
        for pad in PADS {
            assert_eq!(ground(pad + Vec2::new(0.5, 0.7)), Some(PAD_TOP));
        }
        // Every step is one a body walks up: half a metre takes a jump.
        for (low, high) in [(SAND_TOP, STONE_TOP), (STONE_TOP, PLINTH_TOP), (STONE_TOP, PAD_TOP)] {
            assert!(high - low < 0.495, "{low} to {high}");
        }
        // Lifted: the stone where Bridge Wars' islands are, the pads' tops
        // where its pads' are.
        assert_eq!(STONE_TOP, 0.25);
        assert_eq!(PAD_TOP, crate::PAD_TOP);
        // A zombie walks no further out than a step from the rim.
        assert!(WALK_EDGE < RIM && WALK_EDGE > RIM - 1.0);
    }

    #[test]
    fn a_body_is_kept_out_of_every_obstacle_the_short_way() {
        let radius = crate::nav::BODY;
        for (which, obstacle) in OBSTACLES.iter().enumerate() {
            let middle = match *obstacle {
                Obstacle::Circle { at, .. } | Obstacle::Box { at, .. } => at,
            };
            // From inside, every way round: out to its edge and the body's
            // width, and no further than takes.
            for spoke in 0..16 {
                let way = Vec2::from_angle(spoke as f32 * TAU_16);
                let inside = middle + way * 0.3;
                let out = keep_out(inside, radius);
                let (far, _) = obstacle.outside(out);
                assert!(far >= radius - 1e-3, "obstacle {which}, from {inside}: {far} m out at {out}");
                assert!(OBSTACLES.iter().all(|other| other.outside(out).0 >= radius - 1e-3), "obstacle {which}, from {inside}: into another");
            }
            // And nowhere near it, left as it is.
            let clear = middle + Vec2::new(0.0, 4.0);
            if OBSTACLES.iter().all(|other| other.outside(clear).0 > radius) {
                assert_eq!(keep_out(clear, radius), clear);
            }
        }
        // No two obstacles so near that a body could be held between them:
        // either they meet, one solid, or there is a body's width and more
        // between them.
        let edge = |obstacle: &Obstacle| -> Vec<Vec2> {
            match *obstacle {
                Obstacle::Circle { at, radius } => (0..64).map(|spoke| at + Vec2::from_angle(spoke as f32 * TAU_16 / 4.0) * radius).collect(),
                Obstacle::Box { at, half } => (0..=40)
                    .flat_map(|step| {
                        let along = step as f32 / 40.0 * 2.0 - 1.0;
                        [Vec2::new(along, -1.0), Vec2::new(along, 1.0), Vec2::new(-1.0, along), Vec2::new(1.0, along)]
                    })
                    .map(|corner| at + corner * half)
                    .collect(),
            }
        };
        for (a, one) in OBSTACLES.iter().enumerate() {
            for other in &OBSTACLES[a + 1..] {
                let gap = edge(one).into_iter().map(|point| other.outside(point).0).fold(f32::INFINITY, f32::min);
                assert!(gap <= 0.0 || gap > 2.0 * radius, "{one:?} and {other:?}: {gap} m between");
            }
        }
    }

    const TAU_16: f32 = std::f32::consts::TAU / 16.0;

    #[test]
    fn a_shop_sells_only_at_its_own_stand_and_nowhere_else() {
        assert!(at_a_stand(ITEM_SHOP, Shop::Items) && !at_a_stand(ITEM_SHOP, Shop::Upgrades));
        assert!(at_a_stand(UPGRADE_SHOP, Shop::Upgrades) && !at_a_stand(UPGRADE_SHOP, Shop::Items));
        // From the anchor, half way between them, neither.
        assert!(!at_a_stand(ANCHOR, Shop::Items) && !at_a_stand(ANCHOR, Shop::Upgrades));
        // Anywhere in an outline, its own: 2.5 m by 1.5.
        for corner in [Vec2::new(-1.24, -0.75), Vec2::new(1.25, -0.75), Vec2::new(-1.24, 0.75), Vec2::new(1.25, 0.75)] {
            assert!(at_a_stand(ITEM_SHOP + corner, Shop::Items));
            assert!(at_a_stand(UPGRADE_SHOP + corner, Shop::Upgrades));
        }
    }

    /// A made map with every number here as it is: the stone and the sand
    /// as discs, the slab and the pads as boxes, the crystal, a box of walls
    /// round each obstacle, and the two outlines.
    fn made_map(shift: Vec2) -> Vec<(String, Option<String>, Vec<[Vec3; 3]>)> {
        let mut parts = Vec::new();
        let disc = |radius: f32, top: f32| -> Vec<[Vec3; 3]> {
            (0..64)
                .map(|spoke| {
                    let (a, b) = (spoke as f32 * TAU_64, (spoke + 1) as f32 * TAU_64);
                    let corner = |turn: f32| Vec3::new(turn.cos() * radius, top, turn.sin() * radius);
                    // Wound to face up.
                    [Vec3::new(0.0, top, 0.0), corner(b), corner(a)]
                })
                .collect()
        };
        parts.push(("Circle".to_string(), Some("stone.001".to_string()), disc(STONE_RADIUS, STONE_TOP)));
        parts.push(("Circle.001".to_string(), Some("sand".to_string()), disc(SAND_RADIUS, SAND_TOP)));
        parts.push(("Cube.006".to_string(), Some("stone.001".to_string()), solid(ANCHOR, Vec2::splat(PLINTH_HALF), STONE_TOP, PLINTH_TOP)));
        for (index, pad) in PADS.iter().enumerate() {
            parts.push((format!("gen{index}"), Some("stone.001".to_string()), solid(*pad, Vec2::splat(PAD_HALF), STONE_TOP, PAD_TOP)));
        }
        parts.push(("spawn.003".to_string(), Some("spawn".to_string()), solid(ANCHOR, Vec2::splat(0.5), PLINTH_TOP + 0.25, PLINTH_TOP + 3.25)));
        for (index, obstacle) in OBSTACLES.iter().enumerate().skip(1) {
            let (middle, half) = match *obstacle {
                Obstacle::Circle { at, radius } => (at, Vec2::splat(radius * 0.7)),
                Obstacle::Box { at, half } => (at, half - Vec2::splat(0.05)),
            };
            parts.push((format!("thing{index}"), Some("Wood".to_string()), solid(middle + shift, half, STONE_TOP, STONE_TOP + 2.0)));
        }
        for shop in [ITEM_SHOP, UPGRADE_SHOP] {
            let outline = solid(shop, Vec2::new(1.24, 0.75), STONE_TOP + 0.02, STONE_TOP + 0.02);
            parts.push(("Plane.006".to_string(), Some("activation".to_string()), outline));
        }
        parts
    }

    const TAU_64: f32 = std::f32::consts::TAU / 64.0;

    /// A box from `bottom` to `top`, its faces facing out: its top and its
    /// four sides (no bottom, as the map's have none to stand on).
    fn solid(middle: Vec2, half: Vec2, bottom: f32, top: f32) -> Vec<[Vec3; 3]> {
        let corner = |x: f32, z: f32, y: f32| Vec3::new(middle.x + x * half.x, y, middle.y + z * half.y);
        let mut faces = vec![
            [corner(-1.0, -1.0, top), corner(-1.0, 1.0, top), corner(1.0, 1.0, top)],
            [corner(-1.0, -1.0, top), corner(1.0, 1.0, top), corner(1.0, -1.0, top)],
        ];
        if top > bottom {
            let sides = [(-1.0, -1.0, 1.0, -1.0), (1.0, -1.0, 1.0, 1.0), (1.0, 1.0, -1.0, 1.0), (-1.0, 1.0, -1.0, -1.0)];
            for (x0, z0, x1, z1) in sides {
                faces.push([corner(x0, z0, bottom), corner(x1, z1, bottom), corner(x1, z1, top)]);
                faces.push([corner(x0, z0, bottom), corner(x1, z1, top), corner(x0, z0, top)]);
            }
        }
        faces
    }

    #[test]
    fn the_map_s_numbers_are_held_to_a_map_and_what_disagrees_is_said() {
        let as_parts = |map: &[(String, Option<String>, Vec<[Vec3; 3]>)]| -> Vec<String> {
            let parts: Vec<(&str, Option<&str>, &[[Vec3; 3]])> =
                map.iter().map(|(node, material, triangles)| (node.as_str(), material.as_deref(), triangles.as_slice())).collect();
            disagreements(&parts)
        };
        let agrees = made_map(Vec2::ZERO);
        assert_eq!(as_parts(&agrees), Vec::<String>::new());
        // Its walls a metre and a half from where the obstacles are: walls in
        // no obstacle, and obstacles round no wall.
        let moved = made_map(Vec2::new(1.5, 1.5));
        let said = as_parts(&moved);
        assert!(said.iter().any(|line| line.contains("in no obstacle")), "{said:?}");
        assert!(said.iter().any(|line| line.contains("round no wall")), "{said:?}");
        // The stone a hand higher: the ground disagrees.
        let mut higher = made_map(Vec2::ZERO);
        for corner in higher[0].2.iter_mut().flatten() {
            corner.y += 0.1;
        }
        assert!(as_parts(&higher).iter().any(|line| line.contains("the ground at")));
        // A pad where the other is not.
        let mut pad = made_map(Vec2::ZERO);
        for corner in pad[3].2.iter_mut().flatten() {
            corner.x += 0.5;
        }
        assert!(as_parts(&pad).iter().any(|line| line.contains("a pad is at")));
    }
}
