//! Bakes the map the computer's players walk by in Bridge Race
//! (`roundtown_net::bridge_race::grid`, since 2026-11-18) out of Hajun's
//! `bridgeracemap.glb`, with the game's own collision, as `arena::nav_bake`
//! bakes Bridge Wars' (its flood, copied: the body's room, its footing, its
//! step), and checks that the one built into the game and the server is
//! the map's as it is now.
//!
//! The map is read as the game stands it, a quarter of a metre up
//! (`island::lift`), and walked over a grid half a metre to a square from
//! where everyone arrives and from the middle of the golden island, the way
//! a body walks. Three lands: each team's island, by team, and the golden
//! island, which the grid calls the middle (`Region::Middle`). What stands
//! on the teams' islands, the two market stands and the barrels by them,
//! and the palms, the chest and the trophy on the golden island, are walked
//! round as they are on Bridge Wars' map; the pads are a step up, walked
//! onto. The waiting room, 120 m up, is nobody's to walk in a game: it is
//! not on the grid.
//!
//! **After Hajun exports the map again**, `cargo test --lib` fails here
//! until the grid is baked again: `RT_BAKE_NAV=1 cargo test --lib race_bake`
//! writes `shared/src/race.nav`, the next build has it built in, and the
//! server has to be deployed with it.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use bevy::prelude::*;
use roundtown_net::bridge_race::{self, GOLDEN, ISLANDS, WAITING_FLOOR, WAITING_ROOM};
use roundtown_net::nav::{CELL, Grid, Nav, Region};

use crate::island::{self, Island, LATTICE, Venue, WATER_MATERIAL, glb, is_material};
use crate::{LAND_STEP_UP, PLAYER_HEIGHT, PLAYER_RADIUS};

/// The corners of the box the grid covers across the ground, in metres: the
/// three islands, which reach from x -90 to 10 and z -31 to 31, and
/// [`MARGIN`] round them, over the void. A bridge is ground the game knows
/// only where the grid is (`Nav::ground`): as first baked, with two metres
/// round the islands, a bridge laid out wide of them was nothing to the
/// game, a stick thrown onto it falling through it, and the computer's
/// players too. 57,120 squares, a fifth of Bridge Wars' map's, and the
/// void costs nothing baked.
const MARGIN: f32 = 20.0;
const LOW: Vec2 = Vec2::new(-90.0 - MARGIN, -31.0 - MARGIN);
const HIGH: Vec2 = Vec2::new(10.0 + MARGIN, 31.0 + MARGIN);
/// How much room a body is left beyond its own width, in metres, by what
/// stands on the map; how far each way from a body's middle there has to be
/// ground under it; and how far from land a body walks a floor still takes
/// that land's name: Bridge Wars' own (`arena::nav_bake`).
const ROOM: f32 = 0.1;
const FOOTING: f32 = 0.3;
const NAMED_WITHIN: f32 = 1.5;

/// The map's shape as the game has it, stood as high as the game stands it:
/// every mesh but the water, which it has none of, and the skinned, which
/// are bodies.
fn map() -> Island {
    let parts = glb::read_venue(Venue::BridgeRace);
    let corners: Vec<[Vec3; 3]> = parts
        .iter()
        .filter(|part| !part.skinned)
        .filter(|part| !part.material.as_deref().is_some_and(|material| is_material(material, WATER_MATERIAL)))
        .flat_map(|part| part.triangles.iter().copied())
        .collect();
    Island::new(&corners)
}

/// The grid of the map as it is now.
fn bake() -> Grid {
    let map = map();
    let size = ((HIGH - LOW) / CELL).round().as_uvec2();
    let (width, height) = (size.x as usize, size.y as usize);
    let mut grid = Grid::blank(width, height, LOW);
    let index_of = |at: Vec2| grid.index(at).expect("on the grid");

    // Whether anything on the map fills a square of the lattice bodies are
    // kept out of walls on, at a body's heights over a floor: asked once for
    // each square and floor, however many places near it are tried.
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
                    .or_insert_with(|| map.blocked(low + LATTICE * 0.5, floor + LAND_STEP_UP, floor + PLAYER_HEIGHT));
                if full {
                    return false;
                }
            }
        }
        true
    };
    let footed = |at: Vec2, floor: f32| -> bool {
        [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y].into_iter().all(|way| {
            map.floor(at + way * FOOTING, floor + LAND_STEP_UP)
                .is_some_and(|beside| (floor - beside).abs() <= LAND_STEP_UP)
        })
    };

    // Where the walking begins: where everyone arrives, on the teams'
    // islands, and the middle of the golden island.
    let mut seeds: Vec<(Vec2, Region)> = (0..bridge_race::SHAPE.most)
        .map(|seat| (island::arrival(Venue::BridgeRace, seat).translation.xz(), Region::Team(bridge_race::team(seat))))
        .collect();
    seeds.push((GOLDEN, Region::Middle));

    let mut walked: Vec<Option<(f32, Region)>> = vec![None; width * height];
    let mut waiting: VecDeque<usize> = VecDeque::new();
    for (at, region) in seeds {
        let index = index_of(at);
        let middle = grid.centre(index);
        let floor = map.floor(middle, 3.0).unwrap_or_else(|| panic!("nothing to stand on at {middle}, for {region:?}"));
        assert!(fits(middle, floor) && footed(middle, floor), "no room to stand at {middle}, for {region:?}");
        if walked[index].is_none() {
            walked[index] = Some((floor, region));
            waiting.push_back(index);
        }
    }
    while let Some(index) = waiting.pop_front() {
        let (floor, region) = walked[index].expect("walked");
        let (column, row) = ((index % width) as i32, (index / width) as i32);
        for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (c, r) = (column + dc, row + dr);
            if c < 0 || r < 0 || c as usize >= width || r as usize >= height {
                continue;
            }
            let next = r as usize * width + c as usize;
            if walked[next].is_some() {
                continue;
            }
            let at = grid.centre(next);
            // The highest floor a step reaches, and not down more than a
            // step: where a body walking can go and come back.
            let Some(there) = map.floor(at, floor + LAND_STEP_UP) else {
                continue;
            };
            if floor - there > LAND_STEP_UP || !fits(at, there) || !footed(at, there) {
                continue;
            }
            walked[next] = Some((there, region));
            waiting.push_back(next);
        }
    }

    // Every square: walked, or only a floor, which takes the name of the
    // land beside it; or nothing.
    let within = (NAMED_WITHIN / CELL).ceil() as i32;
    for row in 0..height {
        for column in 0..width {
            let index = row * width + column;
            if let Some((floor, region)) = walked[index] {
                grid.set(column, row, Some(floor), true, Some(region));
                continue;
            }
            let at = grid.centre(index);
            let Some(floor) = map.floor(at, 1.0) else {
                continue;
            };
            let mut named: Option<(f32, Region)> = None;
            for dr in -within..=within {
                for dc in -within..=within {
                    let (c, r) = (column as i32 + dc, row as i32 + dr);
                    if c < 0 || r < 0 || c as usize >= width || r as usize >= height {
                        continue;
                    }
                    let Some((_, region)) = walked[r as usize * width + c as usize] else {
                        continue;
                    };
                    let away = Vec2::new(dc as f32, dr as f32).length() * CELL;
                    if away <= NAMED_WITHIN && named.is_none_or(|(least, _)| away < least) {
                        named = Some((away, region));
                    }
                }
            }
            grid.set(column, row, Some(floor), false, named.map(|(_, region)| region));
        }
    }
    // Where each land's middle is: what a span between two of them is
    // drawn through, and what each is walked from.
    grid.place_land(Region::Team(0), ISLANDS[0]);
    grid.place_land(Region::Team(1), ISLANDS[1]);
    grid.place_land(Region::Middle, GOLDEN);
    grid
}

#[test]
fn the_grid_built_in_is_the_race_map_s_as_it_is() {
    let fresh = bake();
    let bytes = fresh.encode();
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("shared/src/race.nav");
    if std::env::var_os("RT_BAKE_NAV").is_some() {
        std::fs::write(&file, &bytes).unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        println!("baked {} bytes into {}: build again to have it built in", bytes.len(), file.display());
        return;
    }
    assert!(
        *bridge_race::grid() == fresh,
        "bridgeracemap.glb is not the map shared/src/race.nav was baked from: bake it again with \
         `RT_BAKE_NAV=1 cargo test --lib race_bake`, build, and deploy the server with it"
    );
}

#[test]
fn the_race_map_baked_is_three_lands_a_body_walks_and_the_way_across_is_63_metres() {
    let grid = bake();
    let nav = Nav::new(Arc::new(grid.clone()));
    // How much of each land a body walks: discs 20 m across, 314 m2, less
    // their rims and what stands on them.
    let (width, height) = grid.size();
    let mut squares: HashMap<Region, usize> = HashMap::new();
    for row in 0..height {
        for column in 0..width {
            let at = grid.centre_of(column, row);
            if grid.clear(at) {
                *squares.entry(grid.region(at).expect("walked land has a name")).or_default() += 1;
            }
        }
    }
    let area = |region: Region| squares.get(&region).copied().unwrap_or(0) as f32 * CELL * CELL;
    for region in [Region::Team(0), Region::Team(1), Region::Middle] {
        let walked = area(region);
        assert!((220.0..300.0).contains(&walked), "{region:?}: {walked} m2 walked");
    }
    assert_eq!(squares.len(), 3, "{:?}", squares.keys().collect::<Vec<_>>());
    // Each where the rules have it, and none joined to another until
    // something is built.
    for (region, middle) in [(Region::Team(0), ISLANDS[0]), (Region::Team(1), ISLANDS[1]), (Region::Middle, GOLDEN)] {
        assert_eq!(grid.land(region), middle);
        assert_eq!(grid.region(middle), Some(region), "{middle}");
        assert!(nav.heart(region).distance(middle) < 1.0, "{region:?}'s heart is {}", nav.heart(region));
    }
    for (one, other) in [(Region::Team(0), Region::Team(1)), (Region::Team(0), Region::Middle), (Region::Team(1), Region::Middle)] {
        assert!(!nav.joined(one, other), "{one:?} and {other:?} are one");
    }
    // Everyone arrives where a body walks, on their own team's island, on
    // its top as the game stands it (or the shop's outline painted 2 cm
    // over it), and can walk from there to where their way across leaves.
    for seat in 0..bridge_race::SHAPE.most {
        let at = island::arrival(Venue::BridgeRace, seat).translation.xz();
        assert!(grid.clear(at), "seat {seat} arrives at {at}, which is not walked");
        assert_eq!(grid.region(at), Some(Region::Team(bridge_race::team(seat))), "seat {seat}");
        let floor = grid.height(at).expect("a floor where a body walks");
        assert!((floor - bridge_race::LIFT).abs() < 0.03, "seat {seat} arrives {floor} m up");
        let span = grid.span(Region::Team(bridge_race::team(seat)), Region::Middle, 0.0).expect("a span");
        assert!(nav.path(at, span.from).is_some(), "seat {seat}: no way from {at} to {}", span.from);
    }
    // The golden island is walked, its middle at its top as the game stands
    // it, and its ring, a band 6 m out and 0.2 m high, a step up; round the
    // chest and the trophy and the palms. The stands and the barrels on the
    // teams' islands are in the way; the pads are a step up, and walked.
    assert!((grid.height(GOLDEN).unwrap() - bridge_race::LIFT).abs() < 0.02, "{:?}", grid.height(GOLDEN));
    for ring in [Vec2::new(5.5, 0.0), Vec2::new(-6.0, 0.0), Vec2::new(0.0, -6.0)] {
        assert!(grid.clear(ring), "the ring at {ring} is not walked");
        assert!((grid.height(ring).unwrap() - (bridge_race::LIFT + 0.2)).abs() < 0.02, "the ring at {ring}: {:?}", grid.height(ring));
    }
    assert!(!grid.clear(Vec2::new(0.0, 3.8)), "the chest is walked through");
    for island in ISLANDS {
        let side = island.y.signum();
        assert!(!grid.clear(Vec2::new(-80.9, island.y - side * 5.5)), "a stand on {island} is walked through");
        assert!(!grid.clear(Vec2::new(-83.2, island.y - side * 5.0)), "a barrel on {island} is walked through");
        let pad = Vec2::new(-86.65, island.y);
        assert!(grid.clear(pad), "the pad on {island} is not walked");
        assert!((grid.height(pad).unwrap() - (bridge_race::LIFT + 0.25)).abs() < 0.02);
    }
    // The waiting room is 120 m up, nobody's in a game: over the void, on
    // the grid, though the map has its floor where the rules say.
    assert_eq!(grid.height(WAITING_ROOM), None);
    let floor = map().floor(WAITING_ROOM, WAITING_FLOOR + 1.0).expect("the waiting room's floor");
    assert!((floor - WAITING_FLOOR).abs() < 0.01, "{floor}");
    // Each team's way across, straight: from just inside its island's shore
    // to just inside the golden island's, 62.7 m of void and a metre of
    // shore; and to either side of it.
    for team in 0..2u8 {
        let span = grid.span(Region::Team(team), Region::Middle, 0.0).expect("a span to the golden island");
        assert!((span.length() - 63.7).abs() < 0.8, "team {team}: {} m", span.length());
        assert_eq!(grid.region(span.from), Some(Region::Team(team)));
        assert_eq!(grid.region(span.to), Some(Region::Middle));
        for aside in [-3.0, 3.0] {
            assert!(grid.span(Region::Team(team), Region::Middle, aside).is_some(), "team {team}, {aside} aside");
        }
    }
    // Laid, a span joins the two, and the way from where the team arrives
    // to the golden island's middle goes over it.
    let mut built = nav.clone();
    let span = grid.span(Region::Team(0), Region::Middle, 0.0).unwrap();
    for index in 0..=span.planks() {
        built.hear(0, 0, index as u16, span.rung(index));
    }
    assert!(built.joined(Region::Team(0), Region::Middle) && !built.joined(Region::Team(1), Region::Middle));
    let from = island::arrival(Venue::BridgeRace, 0).translation.xz();
    let way = built.path(from, GOLDEN).expect("a way to the golden island");
    let mut at = from;
    let mut walked = 0.0;
    for &point in &way {
        assert!(built.line(at, point), "{at} to {point}");
        walked += at.distance(point);
        at = point;
    }
    assert!((82.0..90.0).contains(&walked), "{walked} m to the golden island's middle");
    // The grid reaches far out over the void all round each island, and a
    // bridge laid out there, wide of everything, is ground a body stands
    // on and walks to from the island.
    for middle in [ISLANDS[0], ISLANDS[1], GOLDEN] {
        for spoke in 0..16 {
            let out = middle + Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / 16.0) * (bridge_race::ISLAND_RADIUS + MARGIN - 1.0);
            assert!(grid.index(out).is_some(), "{out}, out from {middle}, is off the grid");
        }
    }
    let mut wide = nav.clone();
    let (shore, out) = (ISLANDS[0] - Vec2::X * (bridge_race::ISLAND_RADIUS - 0.5), ISLANDS[0] - Vec2::X * (bridge_race::ISLAND_RADIUS + MARGIN - 2.0));
    let planks = (shore.distance(out) / 1.4).ceil() as usize;
    for index in 0..=planks {
        wide.hear(0, 0, index as u16, shore.lerp(out, index as f32 / planks as f32));
    }
    let far = out + Vec2::X * 1.0;
    assert!(wide.ground(far).is_some() && wide.walkable_at(far), "a plank {} m out is not ground", far.distance(ISLANDS[0]) - bridge_race::ISLAND_RADIUS);
    assert!(wide.connected(ISLANDS[0], far));
    println!(
        "the race grid: {} bytes baked, {}x{} squares; the teams' islands {:.0} and {:.0} m2 walked, the golden {:.0}; a way across {:.1} m, {} planks",
        grid.encode().len(),
        width,
        height,
        area(Region::Team(0)),
        area(Region::Team(1)),
        area(Region::Middle),
        span.length(),
        span.planks()
    );
}
