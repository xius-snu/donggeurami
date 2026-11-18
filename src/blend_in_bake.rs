//! Bakes the middle of the town as the crowd of a game of Blend In walks it
//! (`roundtown_net::blend_in`, since 2026-11-18) out of Hajun's
//! `roundtownmap.glb`, with the game's own collision, and checks that the
//! one built into the game and the server is the map's as it is now. And
//! watches that crowd walk beside the town's own AIs (`ai`), whose mind it
//! has.
//!
//! The game is played online only, the server runs its crowd so that every
//! device sees the same one, and the server has no map. So the town is read
//! here as the game reads it (`island::read_island`: every mesh but the
//! water and the skinned), and walked over a grid half a metre to a square
//! from where everyone arrives, the way the town's AIs may walk it
//! (`ai::dry_all_the_way`, `crate::dry_ground`): onto a square whose floor
//! is no more than a step up or down from the one it is on
//! ([`LAND_STEP_UP`]), dry, with no water over it, where nothing reaches
//! into a body standing there, and with ground under all of it; and never
//! past [`RING`], where the game is played to. Nothing past it is ever
//! looked at: a square is walked only if everything looked at for it is
//! inside the ring ([`REACH`]), so that nothing out there changes the
//! grid: Hajun's re-export of 2026-11-17, not committed when this was first
//! baked, lays plots for houses 76 to 172 m out. The town's houses and
//! trees from its save (`town.json`) are not in the map's file, and differ
//! from one device to the next: they are left out (there are none in the
//! map's middle).
//!
//! **After Hajun exports the town again**, `cargo test --lib` fails here
//! until the grid is baked again: `RT_BAKE_NAV=1 cargo test --lib
//! blend_in_bake` writes `shared/src/town.nav`, the next build has it built
//! in, and the server has to be deployed with it.

use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use roundtown_net::blend_in::{self, APART, Crowd, RING};
use roundtown_net::nav::{CELL, Grid, Region};
use roundtown_net::{TOWN_SEATS, profile};

use crate::island::{self, Island, Islands, LATTICE, Venue, WATER_MATERIAL, glb, is_material};
use crate::{Intent, LAND_STEP_UP, MOVE_SPEED, PLAYER_HEIGHT, PLAYER_RADIUS, WATER_Y, dry_ground};
use bevy::prelude::*;

/// How far out from the middle of the town the grid reaches each way, in
/// metres: past [`RING`], every square of which is in it.
const HALF: f32 = 72.0;
/// How much room a body is left beyond its own width, in metres, by what
/// stands in the town, as Bridge Wars' grid leaves it (`nav_bake`).
const ROOM: f32 = 0.1;
/// How far each way from a body's middle there has to be ground under it,
/// in metres: most of its width.
const FOOTING: f32 = 0.3;
/// How far from ground a body walks a floor nobody walks still takes that
/// ground's name, in metres: the pond's rim is the town's middle.
const NAMED_WITHIN: f32 = 1.5;
/// How far over a square's floor water is looked for, in metres: water
/// anywhere over it, a body standing there would be in.
const WATER_OVER: f32 = 3.0;
/// How far from the middle of a square the map is looked at to say whether
/// a body walks there, in metres: to the middle of the furthest square of
/// the lattice a body's room reaches into ([`PLAYER_RADIUS`] and [`ROOM`],
/// and half a square of it across), further than its footing
/// ([`FOOTING`]). Only a square with all that inside [`RING`] is walked
/// (the check of 2026-11-18 found squares at the ring's edge looked 0.8 m
/// past it): 0.73 m in from it, at 69.3 m from the middle.
const REACH: f32 = PLAYER_RADIUS + ROOM + LATTICE * std::f32::consts::FRAC_1_SQRT_2;

/// The town's shape as the game has it, every mesh but the water and the
/// skinned, and its water alone.
struct Map {
    land: Island,
    water: Island,
}

fn map(parts: &[glb::Part]) -> Map {
    let wet = |part: &&glb::Part| part.material.as_deref().is_some_and(|material| is_material(material, WATER_MATERIAL));
    let of = |water: bool| -> Vec<[Vec3; 3]> {
        parts
            .iter()
            .filter(|part| !part.skinned && wet(part) == water)
            .flat_map(|part| part.triangles.iter().copied())
            .collect()
    };
    Map {
        land: Island::new(&of(false)),
        water: Island::new(&of(true)),
    }
}

/// The grid of the town as the game's own file has it now.
fn bake() -> Grid {
    bake_parts(&glb::read("roundtownmap.glb"))
}

/// The grid of the town made of `parts`.
fn bake_parts(parts: &[glb::Part]) -> Grid {
    let map = map(parts);
    let side = (2.0 * HALF / CELL).round() as usize;
    let mut grid = Grid::blank(side, side, Vec2::splat(-HALF));
    let index_of = |at: Vec2| grid.index(at).expect("on the grid");
    // A square whose middle is in the ring, and one all of whose looks are.
    let inside = |at: Vec2| at.length() <= RING;
    let wholly = |at: Vec2| at.length() + REACH <= RING;

    // Whether anything in the town fills a square of the lattice bodies are
    // kept out of walls on, at a body's heights over a floor: asked once
    // for each square and floor, however many places near it are tried.
    let mut filled: HashMap<(i32, i32, i32), bool> = HashMap::new();
    let mut fits = |at: Vec2, floor: f32| -> bool {
        let reach = PLAYER_RADIUS + ROOM;
        let first = ((at - reach) / LATTICE).floor().as_ivec2();
        let last = ((at + reach) / LATTICE).floor().as_ivec2();
        for row in first.y..=last.y {
            for column in first.x..=last.x {
                let low = IVec2::new(column, row).as_vec2() * LATTICE;
                if at.distance(at.clamp(low, low + LATTICE)) >= reach {
                    continue;
                }
                let key = (column, row, (floor * 100.0).round() as i32);
                let full = *filled
                    .entry(key)
                    .or_insert_with(|| map.land.blocked(low + LATTICE * 0.5, floor + LAND_STEP_UP, floor + PLAYER_HEIGHT));
                if full {
                    return false;
                }
            }
        }
        true
    };
    let footed = |at: Vec2, floor: f32| -> bool {
        [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y].into_iter().all(|way| {
            map.land
                .floor(at + way * FOOTING, floor + LAND_STEP_UP)
                .is_some_and(|beside| (floor - beside).abs() <= LAND_STEP_UP)
        })
    };
    // Water over the floor, whichever way its face is wound.
    let wet = |at: Vec2, floor: f32| -> bool {
        map.water.floor(at, floor + WATER_OVER).is_some_and(|water| water >= floor - 0.01)
            || map.water.ceiling(at, floor - 0.01).is_some_and(|water| water <= floor + WATER_OVER)
    };
    // The dry ground a body standing at `floor` steps onto at `at`, as the
    // town's AIs have it (`dry_ground`): the highest within a step up.
    let dry = |at: Vec2, floor: f32| map.land.floor(at, floor + LAND_STEP_UP).filter(|&ground| ground > WATER_Y);

    // Where the walking begins: where everyone arrives in the town, on the
    // path round the pond.
    let mut walked: Vec<Option<f32>> = vec![None; side * side];
    let mut waiting: VecDeque<usize> = VecDeque::new();
    for seat in 0..TOWN_SEATS {
        let at = island::arrival(Venue::Town, seat).translation.xz();
        let index = index_of(at);
        let middle = grid.centre(index);
        let floor = dry(middle, 0.5).unwrap_or_else(|| panic!("nothing to stand on at {middle}, for seat {seat}"));
        assert!(fits(middle, floor) && footed(middle, floor) && !wet(middle, floor), "no room to stand at {middle}, for seat {seat}");
        if walked[index].is_none() {
            walked[index] = Some(floor);
            waiting.push_back(index);
        }
    }
    while let Some(index) = waiting.pop_front() {
        let floor = walked[index].expect("walked");
        let (column, row) = ((index % side) as i32, (index / side) as i32);
        for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (c, r) = (column + dc, row + dr);
            if c < 0 || r < 0 || c as usize >= side || r as usize >= side {
                continue;
            }
            let next = r as usize * side + c as usize;
            if walked[next].is_some() {
                continue;
            }
            let at = grid.centre(next);
            if !wholly(at) {
                continue;
            }
            // The highest dry floor a step reaches, and not down more than
            // a step: where a body walking can go and come back.
            let Some(there) = dry(at, floor) else {
                continue;
            };
            if floor - there > LAND_STEP_UP || !fits(at, there) || !footed(at, there) || wet(at, there) {
                continue;
            }
            walked[next] = Some(there);
            waiting.push_back(next);
        }
    }

    // Every square inside the ring: walked, or only a floor (its middle
    // alone looked at), which takes the name of the ground walked beside
    // it; or nothing. Past it, nothing at all.
    let within = (NAMED_WITHIN / CELL).ceil() as i32;
    for row in 0..side {
        for column in 0..side {
            let index = row * side + column;
            if let Some(floor) = walked[index] {
                grid.set(column, row, Some(floor), true, Some(Region::Middle));
                continue;
            }
            let at = grid.centre(index);
            if !inside(at) {
                continue;
            }
            let Some(floor) = map.land.floor(at, 1.0) else {
                continue;
            };
            let beside = (-within..=within).any(|dr| {
                (-within..=within).any(|dc| {
                    let (c, r) = (column as i32 + dc, row as i32 + dr);
                    c >= 0
                        && r >= 0
                        && (c as usize) < side
                        && (r as usize) < side
                        && walked[r as usize * side + c as usize].is_some()
                        && Vec2::new(dc as f32, dr as f32).length() * CELL <= NAMED_WITHIN
                })
            });
            grid.set(column, row, Some(floor), false, beside.then_some(Region::Middle));
        }
    }
    grid
}

#[test]
fn the_grid_built_in_is_the_town_s_as_it_is() {
    let fresh = bake();
    let bytes = fresh.encode();
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("shared/src/town.nav");
    if std::env::var_os("RT_BAKE_NAV").is_some() {
        std::fs::write(&file, &bytes).unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        println!("baked {} bytes into {}: build again to have it built in", bytes.len(), file.display());
        return;
    }
    assert!(
        *blend_in::grid() == fresh,
        "roundtownmap.glb is not the town shared/src/town.nav was baked from: bake it again with \
         `RT_BAKE_NAV=1 cargo test --lib blend_in_bake`, build, and deploy the server with it"
    );
}

/// How much of `grid` a body walks, in square metres.
fn area(grid: &Grid) -> f32 {
    let (width, height) = grid.size();
    let walked = (0..height)
        .flat_map(|row| (0..width).map(move |column| (column, row)))
        .filter(|&(column, row)| grid.clear(grid.centre_of(column, row)))
        .count();
    walked as f32 * CELL * CELL
}

#[test]
fn the_town_baked_is_the_ground_round_the_pond_out_to_the_ring() {
    let grid = bake();
    let (width, height) = grid.size();
    let mut floors: HashMap<i32, usize> = HashMap::new();
    let mut walked = Vec::new();
    for row in 0..height {
        for column in 0..width {
            let at = grid.centre_of(column, row);
            if at.length() > RING {
                assert_eq!(grid.height(at), None, "a floor past the ring at {at}");
                continue;
            }
            if grid.clear(at) {
                walked.push(at);
                *floors.entry((grid.height(at).expect("a floor where a body walks") * 100.0).round() as i32).or_default() += 1;
                assert_eq!(grid.region(at), Some(Region::Middle));
                // Nothing looked at for it past the ring.
                assert!(at.length() + REACH <= RING, "walked at {at}, {} m out", at.length());
            }
        }
    }
    assert!(FOOTING < REACH);
    // The ground from the pond's rim, 25 m out, to the ring, less the room
    // a body is kept from the rim and the lamps.
    let walked_area = walked.len() as f32 * CELL * CELL;
    let ring = std::f32::consts::PI * (RING * RING - 25.0 * 25.0);
    println!("the town's middle: {walked_area:.0} m2 walked of {ring:.0} between the pond and the ring; floors (cm, squares) {floors:?}");
    assert!(walked_area > ring * 0.95 && walked_area < ring, "{walked_area} m2 walked");
    // Every floor walked is the town's ground: the road, the paths, the
    // grass, the planters. Nothing higher than a planter.
    assert!(floors.keys().all(|&floor| (0..=50).contains(&floor)), "{floors:?}");
    // Where everyone arrives, and the path round the pond, the grass, the
    // road round the outside: all walked. The pond is not, nor the
    // fountain, nor the lamps.
    for seat in 0..TOWN_SEATS {
        let at = island::arrival(Venue::Town, seat).translation.xz();
        assert!(grid.clear(at), "seat {seat} arrives at {at}, which is not walked");
    }
    // (The edge: a square's width in from where the walking stops.)
    for out in [30.0, 40.0, 50.0, 65.0, RING - REACH - CELL] {
        for angle in 0..16 {
            let at = Vec2::from_angle(angle as f32 * std::f32::consts::TAU / 16.0 + 0.1) * out;
            assert!(grid.clear(at), "{at}, {out} m out, is not walked");
        }
    }
    for pond in [Vec2::ZERO, Vec2::new(0.0, 10.0), Vec2::new(-15.0, 8.0)] {
        assert!(!grid.clear(pond), "the pond at {pond} is walked");
    }
    for lamp in [Vec2::new(0.0, 43.28), Vec2::new(43.28, 0.0), Vec2::new(0.0, -43.28), Vec2::new(-43.28, 0.0)] {
        assert!(!grid.clear(lamp), "a lamp at {lamp} is walked through");
    }
    // All of it one piece: everywhere walked can be walked to from where
    // everyone arrives.
    let start = grid.index(island::arrival(Venue::Town, 0).translation.xz()).unwrap();
    let mut reached = vec![false; grid.squares()];
    let mut waiting = VecDeque::from([start]);
    reached[start] = true;
    while let Some(index) = waiting.pop_front() {
        let at = grid.centre(index);
        for way in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
            let next = at + way * CELL;
            if let Some(there) = grid.index(next)
                && !reached[there]
                && grid.clear(next)
            {
                reached[there] = true;
                waiting.push_back(there);
            }
        }
    }
    assert_eq!(reached.iter().filter(|&&reached| reached).count(), walked.len());
    println!("the grid: {} bytes baked", grid.encode().len());
}

/// A flat plane of `size` metres across at `at`, `y` up, facing up.
fn plane(at: Vec2, size: f32, y: f32) -> Vec<[Vec3; 3]> {
    let half = size * 0.5;
    let corner = |x: f32, z: f32| Vec3::new(at.x + x, y, at.y + z);
    let (a, b, c, d) = (corner(-half, -half), corner(half, -half), corner(half, half), corner(-half, half));
    // Wound to face up, as the island reads a top (counter-clockwise from
    // above, in the game's x and z).
    vec![[a, d, c], [a, c, b]]
}

#[test]
fn nothing_past_the_ring_changes_the_grid() {
    let plain = bake();
    // Hajun's re-export of 2026-11-17: plots for houses, flat planes at
    // 0.27 m, from 76 m out, and whatever else may come out there, a box
    // as tall as a house a tenth of a metre past the ring's edge, which the
    // squares at the edge looked into before the check of 2026-11-18.
    let mut parts = glb::read("roundtownmap.glb");
    let mut extra: Vec<[Vec3; 3]> = Vec::new();
    for at in [
        Vec2::new(-25.0, 90.0),
        Vec2::new(66.0, 66.0),
        Vec2::new(-66.0, 66.0),
        Vec2::new(66.0, -66.0),
        Vec2::new(-66.0, -66.0),
        Vec2::new(-90.0, -25.0),
    ] {
        extra.extend(plane(at, 24.0, 0.27));
    }
    let low = Vec3::new(-3.0, 0.0, RING + 0.1);
    let high = low + Vec3::new(6.0, 4.0, 3.0);
    let corner = |x: bool, y: bool, z: bool| {
        Vec3::new(if x { high.x } else { low.x }, if y { high.y } else { low.y }, if z { high.z } else { low.z })
    };
    for (a, b, c, d) in [
        (corner(false, true, false), corner(false, true, true), corner(true, true, true), corner(true, true, false)),
        (corner(false, false, false), corner(true, false, false), corner(true, false, true), corner(false, false, true)),
        (corner(false, false, false), corner(false, false, true), corner(false, true, true), corner(false, true, false)),
        (corner(true, false, false), corner(true, true, false), corner(true, true, true), corner(true, false, true)),
        (corner(false, false, false), corner(false, true, false), corner(true, true, false), corner(true, false, false)),
        (corner(false, false, true), corner(true, false, true), corner(true, true, true), corner(false, true, true)),
    ] {
        extra.push([a, b, c]);
        extra.push([a, c, d]);
    }
    parts.push(glb::Part {
        node: "beyond".into(),
        material: Some("plot".into()),
        skinned: false,
        triangles: extra,
    });
    assert!(bake_parts(&parts) == plain, "something past the ring changed the grid");
}

/// The same grid from another export of the town, if `RT_TOWN_GLB` names
/// one: `RT_TOWN_GLB=C:\Users\Hajun\Desktop\RoundTown\assets\roundtownmap.glb
/// cargo test --lib blend_in_bake -- --nocapture`, to see whether an
/// export not yet committed would need the grid baked again.
#[test]
fn another_export_of_the_town_bakes_the_same_grid() {
    let Some(other) = std::env::var_os("RT_TOWN_GLB") else {
        return;
    };
    let other = other.to_string_lossy().into_owned();
    let theirs = bake_parts(&glb::read(&other));
    let ours = bake();
    let (width, height) = ours.size();
    let differ = (0..height)
        .flat_map(|row| (0..width).map(move |column| (column, row)))
        .map(|(column, row)| ours.centre_of(column, row))
        .filter(|&at| (ours.clear(at), ours.height(at)) != (theirs.clear(at), theirs.height(at)))
        .count();
    println!("{other}: {differ} squares differ from the grid of assets/roundtownmap.glb");
    assert!(theirs == ours, "{other} bakes another grid: {differ} squares differ");
}

/// How bodies moved, frame by frame, as anyone watching them sees it: how
/// long they were on the move, every walk's length, and how long from the
/// start of one walk to the start of the next.
#[derive(Default, Debug)]
struct Paces {
    moving: f32,
    still: f32,
    walks: Vec<f32>,
    gaps: Vec<f32>,
}

impl Paces {
    /// Takes `tracks` in, each a body's place on the ground frame by frame,
    /// `dt` apart. A walk under way as the watching began is no walk, nor
    /// is where it began the start of one: only its time on the move counts.
    fn of(tracks: &[Vec<Vec2>], dt: f32) -> Self {
        let mut paces = Self::default();
        for track in tracks {
            let mut walk: Option<Vec2> = None;
            let mut last_start: Option<usize> = None;
            let mut stood = false;
            for (frame, pair) in track.windows(2).enumerate() {
                let moving = pair[0].distance(pair[1]) > 1e-4;
                if moving {
                    paces.moving += dt;
                    if walk.is_none() && stood {
                        walk = Some(pair[0]);
                        if let Some(last) = last_start {
                            paces.gaps.push((frame - last) as f32 * dt);
                        }
                        last_start = Some(frame);
                    }
                } else {
                    paces.still += dt;
                    stood = true;
                    if let Some(from) = walk.take() {
                        paces.walks.push(from.distance(pair[0]));
                    }
                }
            }
        }
        paces
    }

    fn share(&self) -> f32 {
        self.moving / (self.moving + self.still)
    }

    fn mean(all: &[f32]) -> f32 {
        all.iter().sum::<f32>() / all.len().max(1) as f32
    }

    fn line(&self) -> String {
        let least = self.gaps.iter().copied().fold(f32::INFINITY, f32::min);
        format!(
            "walking {:.1}% of the time; {} walks, {:.1} m on average; {:.2} s at the least from one walk to the next, {:.2} on average",
            self.share() * 100.0,
            self.walks.len(),
            Self::mean(&self.walks),
            least,
            Self::mean(&self.gaps),
        )
    }
}

/// Moves each of the town's AIs as its intent has it, at a walk, with no
/// one else in the way: the motion `move_bodies` gives them, but for
/// bumping into one another.
fn walk_the_town_s(time: Res<Time>, islands: Res<Islands>, mut bodies: Query<(&mut Transform, &Intent, &Venue)>) {
    let dt = time.delta_secs();
    for (mut transform, intent, &venue) in &mut bodies {
        let Some(island) = islands.get(venue) else {
            continue;
        };
        let to = transform.translation + intent.walk * MOVE_SPEED * dt;
        transform.translation = match dry_ground(island, to) {
            Some(ground) => to.with_y(ground),
            None => to,
        };
    }
}

#[test]
fn the_crowd_walks_as_the_town_s_own_ais_do_a_little_less_regularly() {
    let (count, secs, dt) = (blend_in::CROWD, 600.0, 1.0 / 30.0);
    let settle = 15.0;
    let frames = ((settle + secs) / dt) as usize;
    let first = (settle / dt) as usize;

    // The crowd, as the server runs it.
    let mut crowd = Crowd::new(21);
    let mut tracks: Vec<Vec<Vec2>> = vec![Vec::new(); count];
    for frame in 0..frames {
        crowd.step(dt, &[]);
        if frame >= first {
            for (track, walker) in tracks.iter_mut().zip(crowd.walkers()) {
                track.push(walker.at.xz());
            }
        }
    }
    let theirs = Paces::of(&tracks, dt);

    // As many of the town's own AIs, by their own mind (`ai`), spread as
    // the crowd is, on the town's own shape.
    let parts = glb::read("roundtownmap.glb");
    let corners: Vec<[Vec3; 3]> = parts
        .iter()
        .filter(|part| !part.skinned && !part.material.as_deref().is_some_and(|material| is_material(material, WATER_MATERIAL)))
        .flat_map(|part| part.triangles.iter().copied())
        .collect();
    let mut islands = Islands::default();
    islands.reshape(Venue::Town, corners, &[]);
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(islands);
    app.add_plugins(crate::ai::plugin);
    app.add_systems(Update, walk_the_town_s.after(crate::ai::steer));
    let open = crowd.open();
    let mut town = Vec::new();
    for seat in 0..count {
        let at = open[(seat * 7919) % open.len()];
        let ground = blend_in::grid().height(at).unwrap_or(0.0);
        town.push(app.world_mut().spawn((Transform::from_xyz(at.x, ground, at.y), Venue::Town, crate::ai::brain(seat))).id());
    }
    let mut tracks: Vec<Vec<Vec2>> = vec![Vec::new(); count];
    for frame in 0..frames {
        app.world_mut().resource_mut::<Time>().advance_by(Duration::from_secs_f32(dt));
        app.update();
        if frame >= first {
            for (track, &body) in tracks.iter_mut().zip(&town) {
                track.push(app.world().get::<Transform>(body).unwrap().translation.xz());
            }
        }
    }
    let ours = Paces::of(&tracks, dt);
    println!("the town's own AIs: {}", ours.line());
    println!("Blend In's crowd:   {}", theirs.line());

    // Walks of the same lengths, about as often and about as much of the
    // time, but not all at the one beat.
    let (mean_ours, mean_theirs) = (Paces::mean(&ours.walks), Paces::mean(&theirs.walks));
    assert!((mean_theirs - mean_ours).abs() < mean_ours * 0.15, "{mean_theirs} m against {mean_ours}");
    assert!((theirs.share() - ours.share()).abs() < ours.share() * 0.25, "{} against {}", theirs.share(), ours.share());
    let (often_ours, often_theirs) = (Paces::mean(&ours.gaps), Paces::mean(&theirs.gaps));
    assert!((often_theirs - often_ours).abs() < often_ours * 0.2, "a walk every {often_theirs} s against {often_ours}");
    let soon = |paces: &Paces| paces.gaps.iter().filter(|&&gap| gap < 2.9).count() as f32 / paces.gaps.len() as f32;
    assert_eq!(soon(&ours), 0.0, "the town's think every 3 s");
    assert!(soon(&theirs) > 0.2, "{}", soon(&theirs));
    assert!(theirs.gaps.iter().all(|&gap| gap >= blend_in::SHORTEST_WAIT.0 - 2.0 * dt));
    // And nobody of the crowd ends a walk in anyone else.
    for (index, walker) in crowd.walkers().iter().enumerate() {
        for other in &crowd.walkers()[index + 1..] {
            if !matches!(walker.doing(), blend_in::Doing::Walking { .. }) && !matches!(other.doing(), blend_in::Doing::Walking { .. }) {
                assert!(walker.at.xz().distance(other.at.xz()) >= APART.0 - 0.01);
            }
        }
    }
    // The town's AIs go by names a person may have, which the crowd's are.
    assert!(crowd.walkers().iter().all(|walker| profile::name(&walker.guise.name).as_deref() == Some(walker.guise.name.as_str())));
}

#[test]
fn the_grid_has_a_square_for_every_spot_the_crowd_is_spread_over() {
    let grid = blend_in::grid();
    assert!(area(&grid) > 0.0);
    let crowd = Crowd::new(1);
    for walker in crowd.walkers() {
        assert!(grid.clear(walker.at.xz()));
    }
}
