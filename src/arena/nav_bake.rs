//! Bakes the map the computer's players walk by (`roundtown_net::nav`,
//! since 2026-11-12) out of Hajun's `pvptownmap.glb`, with the game's own
//! collision, and checks that the one built into the game and the server
//! is the map's as it is now.
//!
//! The server has no map, and a bot there has to know where it can stand,
//! where the stands, the anchors' slabs, the pond and the lamps are, and
//! where each island ends. So the map is read here as the game reads it
//! (`island::read_island`: every mesh but the water and the skinned), and
//! walked over a grid half a metre to a square from where everyone
//! arrives, the way a body walks: onto a square whose floor is no more
//! than a step up or down from the one it is on, where nothing reaches
//! into a body standing there, and where there is ground under all of it.
//! What that reaches is a body's to walk ([`Grid::clear`]); whatever other
//! floor there is holds a body knocked onto it, and no more.
//!
//! **After Hajun exports the map again**, `cargo test --lib` fails here
//! until the grid is baked again: `RT_BAKE_NAV=1 cargo test --lib nav_bake`
//! writes `shared/src/arena.nav`, the next build has it built in, and the
//! server has to be deployed with it, as with any other of the map's
//! numbers it goes by.

use std::collections::{HashMap, VecDeque};

use roundtown_net::nav::{CELL, Grid, Nav, Region};
use roundtown_net::{AMETHYST_PADS, ARENA_PADS, ARENA_SHOPS, ARENA_UPGRADE_SHOPS, SAPPHIRE_PADS};

use super::*;
use crate::LAND_STEP_UP;
use crate::island::{LATTICE, WATER_MATERIAL, glb};

/// How far out from the middle of the map the grid reaches each way, in
/// metres: the islands' far edges are 140.4 m out.
const HALF: f32 = 141.0;
/// How much room a body is left beyond its own width, in metres, by what
/// stands on the map, for a bot to walk past it without brushing it.
const ROOM: f32 = 0.1;
/// How far each way from a body's middle there has to be ground under it,
/// in metres: most of its width, so that it stands on the island and not
/// over its edge.
const FOOTING: f32 = 0.3;
/// How far from land a body walks, a floor still takes that land's name,
/// in metres: the rim of an island, which holds a body and is not walked,
/// is that island's.
const NAMED_WITHIN: f32 = 1.5;

/// The map's shape as the game has it: every mesh of it but the water,
/// which a body goes through, and the skinned, which are bodies.
fn map() -> Island {
    let parts = glb::read("pvptownmap.glb");
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
    let side = (2.0 * HALF / CELL).round() as usize;
    let mut grid = Grid::blank(side, side, Vec2::splat(-HALF));
    let index_of = |at: Vec2| grid.index(at).expect("on the grid");

    // Whether anything on the map fills a square of the lattice bodies are
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

    // Where the walking begins: where everyone arrives, the smaller
    // islands' open ground, the path round the pond, and the pond's own
    // floor, which is a pocket nobody walks into or out of.
    let mut seeds: Vec<(Vec2, Region)> = Vec::new();
    for seat in 0..ARENA_SEATS {
        let at = island::arrival(Venue::Arena, seat).translation.xz();
        seeds.push((at, Region::Team(arena_team(seat))));
    }
    for island in 0..4 {
        seeds.push((rules::zombie_spot(4 + island), Region::Sapphire(island as u8)));
        seeds.push((rules::zombie_spot(8 + island), Region::Middle));
    }
    seeds.push((Vec2::ZERO, Region::Pocket));

    let mut walked: Vec<Option<(f32, Region)>> = vec![None; side * side];
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
    for row in 0..side {
        for column in 0..side {
            let index = row * side + column;
            if let Some((floor, region)) = walked[index] {
                grid.set(column, row, Some(floor), region != Region::Pocket, Some(region));
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
                    if c < 0 || r < 0 || c as usize >= side || r as usize >= side {
                        continue;
                    }
                    let Some((_, region)) = walked[r as usize * side + c as usize] else {
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
    grid
}

#[test]
fn the_grid_built_in_is_the_map_s_as_it_is() {
    let fresh = bake();
    let bytes = fresh.encode();
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("shared/src/arena.nav");
    if std::env::var_os("RT_BAKE_NAV").is_some() {
        std::fs::write(&file, &bytes).unwrap_or_else(|error| panic!("{}: {error}", file.display()));
        println!("baked {} bytes into {}: build again to have it built in", bytes.len(), file.display());
        return;
    }
    assert!(
        *Grid::arena() == fresh,
        "pvptownmap.glb is not the map shared/src/arena.nav was baked from: bake it again with \
         `RT_BAKE_NAV=1 cargo test --lib nav_bake`, build, and deploy the server with it"
    );
}

#[test]
fn the_map_baked_is_nine_lands_a_body_walks_with_the_pond_shut_off() {
    let grid = bake();
    let nav = Nav::new(std::sync::Arc::new(grid.clone()));
    // How much of each land a body walks: the teams' islands, 20.8 m
    // across, less their edges, their stands, slabs and barrels; the
    // smaller islands, 24 m across; and the middle, out to 76 m, without
    // its pond.
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
    for team in 0..4 {
        let walked = area(Region::Team(team));
        assert!((240.0..335.0).contains(&walked), "team {team}'s island: {walked} m2 walked");
        let smaller = area(Region::Sapphire(team));
        assert!((360.0..450.0).contains(&smaller), "sapphire island {team}: {smaller} m2 walked");
    }
    let middle = area(Region::Middle);
    assert!((14_500.0..16_300.0).contains(&middle), "the middle: {middle} m2 walked");
    assert_eq!(area(Region::Pocket), 0.0, "nobody walks the pond");
    assert_eq!(squares.len(), 9, "{:?}", squares.keys().collect::<Vec<_>>());
    // The pond is its own: its floor holds a body, and there is no walking
    // out of it.
    assert_eq!(grid.region(Vec2::ZERO), Some(Region::Pocket));
    assert!(grid.height(Vec2::ZERO).is_some() && !grid.clear(Vec2::ZERO));
    assert!(!nav.walkable_at(Vec2::new(10.0, -8.0)));
    // No land is joined to any other until something is built.
    for (n, &a) in Region::LANDS.iter().enumerate() {
        for &b in &Region::LANDS[n + 1..] {
            assert!(!nav.joined(a, b), "{a:?} and {b:?} are one");
        }
        assert_eq!(grid.region(nav.heart(a)), Some(a));
    }
    // Everywhere the game sends a body, it can walk: where everyone
    // arrives, every pad, and beside every shop.
    for seat in 0..ARENA_SEATS {
        let at = island::arrival(Venue::Arena, seat).translation.xz();
        assert!(grid.clear(at), "seat {seat} arrives at {at}, which is not walked");
        // The island's top, or the shop's outline painted 2 cm over it.
        let floor = grid.height(at).expect("a floor where a body walks");
        assert!((floor - island::LAND_TOP_OF_THE_ISLANDS).abs() < 0.03, "seat {seat} arrives {floor} m up");
    }
    for (pads, name) in [(ARENA_PADS, "a team's"), (SAPPHIRE_PADS, "a sapphire"), (AMETHYST_PADS, "an amethyst")] {
        for pad in pads {
            assert!(grid.clear(pad), "{name} pad at {pad} is not walked");
            assert_eq!(grid.height(pad), Some(PAD_TOP));
        }
    }
    for team in 0..ARENA_TEAMS {
        for shop in [ARENA_SHOPS[team], ARENA_UPGRADE_SHOPS[team]] {
            let stand = grid.nearest_clear(shop, 3.0).unwrap_or_else(|| panic!("nowhere to stand by the shop at {shop}"));
            assert!(stand.distance(shop) < rules::SHOP_SLACK - 1.0, "the shop at {shop} is served from {stand}");
            assert_eq!(grid.region(stand), Some(Region::Team(team as u8)));
            // And from where the team arrives there is a way to it, to
            // the pad, and to beside the anchor.
            let from = island::arrival(Venue::Arena, team).translation.xz();
            for to in [stand, ARENA_PADS[team]] {
                let way = nav.path(from, to).unwrap_or_else(|| panic!("no way from {from} to {to}"));
                assert_eq!(way.last(), Some(&to));
            }
        }
        let anchor = ARENA_ANCHORS[team];
        assert!(!grid.clear(anchor), "team {team}'s anchor is walked through");
        let beside = grid.nearest_clear(anchor, 3.0).unwrap_or_else(|| panic!("nowhere to stand by the anchor at {anchor}"));
        assert!(beside.distance(anchor) < rules::REACH + ANCHOR_RADIUS, "the anchor at {anchor} is {} m from {beside}", beside.distance(anchor));
    }
    // What stands on the map is walked round: the stands either side of
    // an island's middle, and the lamp on each axis in the middle.
    assert!(!grid.clear(Vec2::new(5.3, 130.0)) && !grid.clear(Vec2::new(-5.3, 130.0)));
    assert!(!grid.clear(Vec2::new(0.0, 43.3)));
    // The star's eight spans: each team's island to the middle, 43.6 m of
    // void and a metre of shore, and each smaller island to it, 37 m and
    // the same; all of a length, by the map's four-fold symmetry.
    for team in 0..4u8 {
        let span = grid.span(Region::Team(team), Region::Middle, 0.0).expect("a span to the middle");
        assert!((span.length() - 44.6).abs() < 0.6, "team {team}: {} m", span.length());
        assert_eq!(grid.region(span.from), Some(Region::Team(team)));
        assert_eq!(grid.region(span.to), Some(Region::Middle));
        assert!(span.planks() <= 34 && span.length() <= 2.0 * rules::BRIDGE_A_BUY);
        let back = grid.span(Region::Middle, Region::Sapphire(team), 0.0).expect("a span to a smaller island");
        assert!((back.length() - 38.0).abs() < 0.9, "sapphire {team}: {} m", back.length());
        assert!(back.length() <= 2.0 * rules::BRIDGE_A_BUY);
        // To either side of the middle line as well.
        for aside in [-2.5, 2.5] {
            assert!(grid.span(Region::Team(team), Region::Middle, aside).is_some());
        }
    }
    // Laid, a span joins its two lands, and the way from the island to a
    // pad in the middle goes over it and round the lamp.
    let mut built = nav.clone();
    let span = grid.span(Region::Team(0), Region::Middle, 0.0).unwrap();
    for index in 0..=span.planks() {
        built.hear(4, 0, index as u16, span.rung(index));
    }
    assert!(built.joined(Region::Team(0), Region::Middle) && !built.joined(Region::Team(1), Region::Middle));
    let from = island::arrival(Venue::Arena, 0).translation.xz();
    let way = built.path(from, AMETHYST_PADS[0]).expect("a way to the pad in the middle");
    let mut at = from;
    let mut walked = 0.0;
    for &point in &way {
        assert!(built.line(at, point), "{at} to {point}");
        walked += at.distance(point);
        at = point;
    }
    assert!((94.0..104.0).contains(&walked), "{walked} m to the pad, as the crow flies {}", from.distance(AMETHYST_PADS[0]));
    println!(
        "the grid: {} bytes baked; a team's island {:.0} m2, a smaller one {:.0}, the middle {:.0}; a span {:.1} m",
        grid.encode().len(),
        area(Region::Team(0)),
        area(Region::Sapphire(0)),
        middle,
        span.length()
    );
}
