//! Proof that the made-up lots (`roundtown_net::premade`, the land that
//! fills a plot of the town nobody who is here has) are lots the game
//! itself would let stand. They are written by hand, as numbers, in a
//! crate that has no models: here each of the 48 is stood on its own plot
//! of the town's map, as `stand::keep_theirs` stands it, with every
//! model's own size read out of its file, and held to the rules a thing is
//! put down by.
//!
//! **Of every lot** ([`faults`]):
//!
//! * everything on it is a piece there is, one of them a house, what goes
//!   in a wall in a wall and the rest on the ground, each under a number of
//!   its own, somewhere a thing may be said to stand (`bag::sound`);
//! * the house stands where a house may be put down (`misfit`,
//!   `house_fits_in`: the whole of it on the plot, on level land, in
//!   nothing), at the height of the land there, and a metre clear of the
//!   plot's sign;
//! * each door and window is in a wall the house has, where the wall has
//!   room for it by the rule it is dragged by (`Wall::fit`), and no two
//!   holes of a wall overlap;
//! * each thing that stands on a floor may be put down where it is with
//!   everything else down (`misfit`: on the plot, in no wall and in nothing
//!   else), on the floor that is under it, a metre clear of the sign and
//!   out of the way of a door;
//! * the middle of the plot, where its owner arrives, is free floor, and
//!   from there someone walks out of every door, on to the front edge of
//!   the plot, and to where the game offers each place of the bed and of
//!   every seat (`rest::offered`), a body's width at the least all the way;
//! * a bed and a seat are in the house, on its ground floor, with floor
//!   free beside the bed and in front of the seat ([`BESIDE`]); and so has
//!   every other seat, but a chair drawn up to a table, which has it behind
//!   it or beside it;
//! * what is painted is a part its model has, once;
//! * and it is no more than a phone draws easily ([`MOST_ON_A_PLOT`]).
//!
//! **Of all of them**: no more triangles than [`MOST_IN_ALL`], no two the
//! same, each unlike the one before it, and every house, door and window
//! Hajun has modelled somewhere among them. And every one of them stands
//! by the same rules on five other plots, turned other ways
//! ([`every_lot_stands_as_well_on_plots_turned_other_ways`]): none is
//! right only by where its own plot happens to lie.
//!
//! The town is its map as the game reads it, and a plot is the game's own
//! (`land::read_plots`), its front and its sign with it: nothing here
//! takes where a plot's sign stands for granted but
//! [`the_sign_stands_at_the_front_left_of_a_plot_s_own_space`], which says
//! where `premade` takes it to.

use bevy::prelude::*;
use roundtown_net::At;
use roundtown_net::bag::{self, Standing};
use roundtown_net::land::{Lot, PLOTS};
use roundtown_net::premade;

use super::stand::{placed, yard_of};
use super::{
    Bounds, Height, Mount, OnWall, PIECES, PieceDef, Rooms, Span, Stop, Storey, WINDOW_SILL, Wall, floor_under, house_fits_in, misfit, pane, wall_mesh,
    wall_order,
};
use crate::island::{self, Island, glb};
use crate::land::{self, Plot};
use crate::{LAND_STEP_UP, PLAYER_HEIGHT, PLAYER_RADIUS, rest};

/// How many triangles each piece is drawn with, which is what it costs a
/// phone that draws the town 120 times a second: a chair and a lamp are
/// the dear ones. [`the_pieces_are_drawn_with_as_many_triangles_as_counted`]
/// holds these to the models. The houses of three floors and more, and the
/// furniture and things that came on 2026-11-25, are the town's plots'
/// alone: nothing made-up stands any of them (an eight-floor house is
/// 3,400 triangles).
const TRIANGLES: [(&str, usize); 95] = [
    ("square_house", 144),
    ("square_house_roofless", 72),
    ("square_house_two_floor", 544),
    ("square_house_roofless_two_floor", 532),
    ("classic_door", 240),
    ("classic_door_with_window", 328),
    ("classic_double_door", 446),
    ("glass_door", 348),
    ("glass_double_door", 664),
    ("square_window", 70),
    ("wide_window", 82),
    ("singlebed", 412),
    ("doublebed", 520),
    ("cafechair", 1004),
    ("circletable", 220),
    ("cafedeskedge", 36),
    ("cafedeskcorner", 84),
    ("softcouch", 636),
    ("streetlamp", 1680),
    ("wide_roof_house", 144),
    ("square_house_three_floor", 1076),
    ("square_house_four_floor", 1542),
    ("square_house_five_floor", 2008),
    ("square_house_six_floor", 2474),
    ("square_house_seven_floor", 2940),
    ("square_house_eight_floor", 3406),
    ("square_house_nine_floor", 3872),
    ("square_house_ten_floor", 4338),
    ("square_house_roofless_three_floor", 1004),
    ("square_house_roofless_four_floor", 1476),
    ("square_house_roofless_five_floor", 1948),
    ("square_house_roofless_six_floor", 2420),
    ("square_house_roofless_seven_floor", 2892),
    ("square_house_roofless_eight_floor", 3364),
    ("square_house_roofless_nine_floor", 3836),
    ("square_house_roofless_ten_floor", 4308),
    ("end_table", 162),
    ("cabinet", 324),
    ("bathtub", 474),
    ("alarm_clock", 458),
    ("chair", 88),
    ("closed_umbrella", 436),
    ("computer_mouse", 100),
    ("cup_of_tea", 436),
    ("dumbbell", 492),
    ("empty_box", 76),
    ("frying_pan", 500),
    ("hair_dryer", 480),
    ("hamburger", 404),
    ("hand_rake", 356),
    ("mug_with_office_tool", 610),
    ("screwdriver", 344),
    ("stapler", 162),
    ("stool", 610),
    ("table", 86),
    ("tissue_box", 102),
    ("trash_bin", 238),
    ("watering_can", 554),
    ("wine_glass", 336),
    ("armchair", 162),
    ("bathroom_sink", 162),
    ("coat_rack", 378),
    ("desk", 166),
    ("desk_fan", 294),
    ("doll", 1234),
    ("dresser", 430),
    ("flowers", 496),
    ("fruit_bowl", 806),
    ("globe", 478),
    ("headphones", 472),
    ("lamp_with_shade", 786),
    ("mailbox", 242),
    ("monitor", 84),
    ("painting", 164),
    ("piggy_bank", 626),
    ("potted_plant", 390),
    ("present", 228),
    ("printer", 182),
    ("propane_tank", 1008),
    ("rubber_duck", 710),
    ("table_large_circular", 298),
    ("toilet", 404),
    ("barbecue", 655),
    ("bed", 726),
    ("ceiling_fan", 188),
    ("computer", 324),
    ("couch", 268),
    ("refrigerator", 190),
    ("rocking_chair", 668),
    ("stove", 858),
    ("treadmill", 220),
    ("grandfathers_clock", 547),
    ("safe", 222),
    ("telescope", 734),
    ("chandelier", 1204),
];
/// The most triangles one made-up plot is drawn with, and all 48 together.
const MOST_ON_A_PLOT: usize = 5_000;
const MOST_IN_ALL: usize = 190_000;
/// The most things on one of them, and the most plots with a lamp.
const MOST_THINGS: usize = 30;
const MOST_LIT: usize = 20;

/// How near two places have to be to be the same, in metres, as `stand`
/// has it.
const SAME: f32 = 1e-3;
/// How far everything on a plot keeps from its sign, in metres.
const CLEAR_OF_THE_SIGN: f32 = 1.0;
/// How far out from the foot of a door the ground is kept clear, in
/// metres: its leaves swing 1.8 m out.
const DOOR_FRONT: f32 = 2.0;
/// How far out from a bed's side, or from a seat's front, the middle of
/// someone standing there may be for there to be floor there, in metres:
/// from just clear of it to this, a little at a time, a body being kept
/// out of the town's shape a square at a time (`island::LATTICE`).
const BESIDE: f32 = 0.8;
/// How fine the squares are that a way on foot is looked for over, in
/// metres: as fine as a body is tested against the town
/// (`island::LATTICE`).
const CELL: f32 = 0.25;

/// One of the pieces as its model has it, read as the game reads it once
/// it has spawned (`super::piece_ready`): the box it fills; for what goes
/// in a wall, how much of the wall its frame takes; for a house, the boxes
/// its walls fill, in the order the game numbers them (`super::walls_of`),
/// its floors and whatever else is built into it; its triangles, and for a
/// house those of everything but its walls, which the game makes again
/// round their holes; and what its materials are called.
///
/// The game takes a mesh for a wall, a floor, a frame, a roof or a gable
/// by what it, or the object it is a mesh of, is called in Blender; this
/// goes by the object's name alone, which says the same of every piece of
/// Hajun's: none has a mesh called so under an object that is not.
struct Model {
    def: &'static PieceDef,
    bounds: Bounds,
    span: Option<Span>,
    walls: Vec<(Vec3, Vec3)>,
    floors: Vec<(Vec3, Vec3)>,
    fixtures: Vec<(Vec3, Vec3)>,
    triangles: Vec<[Vec3; 3]>,
    shell: Vec<[Vec3; 3]>,
    materials: Vec<String>,
}

fn read(def: &'static PieceDef) -> Model {
    let is_house = def.mount == Mount::House;
    let grow = |sum: Option<(Vec3, Vec3)>, (low, high): (Vec3, Vec3)| Some(sum.map_or((low, high), |(a, b): (Vec3, Vec3)| (a.min(low), b.max(high))));
    let (mut bounds, mut frame) = (None, None);
    let (mut walls, mut floors, mut fixtures) = (Vec::new(), Vec::new(), Vec::new());
    let (mut triangles, mut shell) = (Vec::new(), Vec::new());
    let mut materials: Vec<String> = Vec::new();
    for part in glb::read(&def.model()) {
        triangles.extend_from_slice(&part.triangles);
        if let Some(material) = &part.material
            && !materials.contains(material)
        {
            materials.push(material.clone());
        }
        let corners = part.triangles.iter().flatten().map(|&corner| (corner, corner));
        let Some(filled) = corners.reduce(|(low, high), (a, b)| (low.min(a), high.max(b))) else {
            continue;
        };
        bounds = grow(bounds, filled);
        let called = |prefix: &str| part.node.starts_with(prefix);
        if called("Frame") {
            frame = grow(frame, filled);
        }
        if is_house && called("Wall") {
            walls.push(filled);
            continue;
        }
        shell.extend_from_slice(&part.triangles);
        if is_house && called("Floor") {
            floors.push(filled);
        } else if is_house && !called("Roof") && !called("Gable") {
            fixtures.push(filled);
        }
    }
    let (low, high) = bounds.unwrap_or_else(|| panic!("{} has no triangles", def.kind));
    let span = def.on_wall().then(|| {
        let (low, high) = frame.filter(|_| def.holed()).unwrap_or((low, high));
        Span::of(low, high)
    });
    walls.sort_by_key(|&(low, _)| wall_order(low));
    Model {
        def,
        bounds: Bounds { low, high },
        span,
        walls,
        floors,
        fixtures,
        triangles,
        shell,
        materials,
    }
}

/// The town, to stand the lots on: its shape, as the game reads its map;
/// its plots, in Hajun's order, the plot of each number; and every piece
/// there is.
struct Town {
    island: Island,
    plots: Vec<Plot>,
    models: Vec<Model>,
}

fn town() -> Town {
    let (island, ..) = crate::roads::tests::round_town();
    let all = land::tests::plots();
    let plots: Vec<Plot> = land::in_order(&all).into_iter().map(|plot| all[plot]).collect();
    assert_eq!(plots.len(), PLOTS, "the map has not a plot for every number");
    Town {
        island,
        plots,
        models: PIECES.iter().map(read).collect(),
    }
}

/// The corners of the footprint of something as big as `bounds` that
/// stands as `place` has it, in order round it, across the ground.
fn footprint(place: &Transform, bounds: Bounds) -> [Vec2; 4] {
    let (low, high) = (bounds.low, bounds.high);
    [(low.x, low.z), (high.x, low.z), (high.x, high.z), (low.x, high.z)].map(|(x, z)| place.transform_point(Vec3::new(x, 0.0, z)).xz())
}

/// How far apart two shapes on the ground are, each given by its corners
/// in order round it and neither with a dent in it: nothing, if they
/// overlap.
fn apart(a: &[Vec2], b: &[Vec2]) -> f32 {
    let sides = |shape: &[Vec2]| -> Vec<(Vec2, Vec2)> { (0..shape.len()).map(|at| (shape[at], shape[(at + 1) % shape.len()])).collect() };
    let reach = |shape: &[Vec2], way: Vec2| {
        shape
            .iter()
            .map(|corner| corner.dot(way))
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(least, most), far| (least.min(far), most.max(far)))
    };
    // Apart, they are apart along the way some side of one of them looks.
    let parted = sides(a).into_iter().chain(sides(b)).any(|(from, to)| {
        let way = (to - from).perp();
        let (one, other) = (reach(a, way), reach(b, way));
        one.1 < other.0 || other.1 < one.0
    });
    if !parted {
        return 0.0;
    }
    let to_side = |point: Vec2, (from, to): (Vec2, Vec2)| {
        let along = ((point - from).dot(to - from) / (to - from).length_squared()).clamp(0.0, 1.0);
        point.distance(from + (to - from) * along)
    };
    let nearest = |points: &[Vec2], shape: &[Vec2]| {
        points
            .iter()
            .flat_map(|&point| sides(shape).into_iter().map(move |side| to_side(point, side)))
            .fold(f32::INFINITY, f32::min)
    };
    nearest(a, b).min(nearest(b, a))
}

/// A door or a window of a lot: what stands, its model, how much of its
/// wall it takes, which wall, and where on it.
struct Hung<'a> {
    stood: &'a Standing,
    model: &'a Model,
    span: Span,
    wall: usize,
    on: OnWall,
}

impl Hung<'_> {
    /// The hole it needs, as `super::carve` takes one.
    fn hole(&self) -> [f32; 4] {
        let (on, span) = (self.on, self.span);
        [on.along - span.half, on.up - span.below, on.along + span.half, on.up + span.above]
    }
}

/// Something of a lot that stands on a floor: what stands, its model,
/// where it stands in the world, its triangles there, and whether it is in
/// the house, as the game says what rides along in one that is picked up
/// (`stand::note_edits`).
struct Down<'a> {
    stood: &'a Standing,
    model: &'a Model,
    place: Transform,
    part: Vec<[Vec3; 3]>,
    inside: bool,
}

impl Down<'_> {
    fn kind(&self) -> &str {
        &self.stood.kind
    }

    /// A point of its own space, across the ground.
    fn point(&self, x: f32, z: f32) -> Vec2 {
        self.place.transform_point(Vec3::new(x, 0.0, z)).xz()
    }
}

/// What a thing is called in a fault: its kind and its number.
fn called(stood: &Standing) -> String {
    format!("{} {}", stood.kind, stood.id)
}

/// Everything that is wrong with `lot` stood on `plot` of the town, whose
/// shape is `town`: nothing, for a lot the game would let stand.
fn faults(lot: &Lot, plot: &Plot, town: &mut Island, models: &[Model]) -> Vec<String> {
    let mut faults: Vec<String> = Vec::new();
    let yard = yard_of(plot);
    let model_of = |kind: &str| models.iter().find(|model| model.def.kind == kind);

    // ------------------------------------------- what there is on it
    let mut ids: Vec<u32> = lot.stands.iter().map(|stood| stood.id).collect();
    ids.sort_unstable();
    if ids != (1..=lot.stands.len() as u32).collect::<Vec<_>>() {
        faults.push(format!("its things are not numbered from 1 up, each once: {ids:?}"));
    }
    if lot.stands.len() > MOST_THINGS {
        faults.push(format!("{} things stand on it", lot.stands.len()));
    }
    let (mut houses, mut hung, mut standing) = (Vec::new(), Vec::new(), Vec::new());
    for stood in &lot.stands {
        let Some(model) = model_of(&stood.kind) else {
            faults.push(format!("{} is no piece of the game's", called(stood)));
            continue;
        };
        if !bag::sound(&stood.at) {
            faults.push(format!("{} stands nowhere a thing may: {:?}", called(stood), stood.at));
        }
        match (model.def.mount, stood.at) {
            (Mount::House, At::Ground { x, y, z, yaw }) => houses.push((stood, model, placed(&yard, x, y, z, yaw))),
            (Mount::Floor, At::Ground { x, y, z, yaw }) => standing.push((stood, model, placed(&yard, x, y, z, yaw))),
            (Mount::Door | Mount::Window, At::Wall { wall, along, up, face }) => {
                let on = OnWall {
                    wall: Entity::PLACEHOLDER,
                    along,
                    up,
                    face: f32::from(face),
                };
                match model.span {
                    Some(span) => hung.push(Hung {
                        stood,
                        model,
                        span,
                        wall: usize::from(wall),
                        on,
                    }),
                    None => faults.push(format!("{} has no frame", called(stood))),
                }
            }
            _ => faults.push(format!("{} stands as what it is not: {:?}", called(stood), stood.at)),
        }
    }
    let &[(_, house, home)] = houses.as_slice() else {
        faults.push(format!("{} houses stand on it", houses.len()));
        return faults;
    };
    let into_house = home.compute_affine().inverse();
    let house_shape = footprint(&home, house.bounds);

    // ------------------------------------------------------ its walls
    let against: Vec<(Vec3, Vec3)> = house.walls.iter().chain(&house.fixtures).copied().collect();
    let mut walls: Vec<Wall> = house.walls.iter().map(|&(low, high)| Wall::new(low, high, &house.floors, &against)).collect();
    for (index, opening) in hung.iter().enumerate() {
        let Some(wall) = walls.get(opening.wall) else {
            faults.push(format!("{} is in wall {}, which the house has not", called(opening.stood), opening.wall));
            continue;
        };
        let others: Vec<(OnWall, Span, bool)> = hung
            .iter()
            .enumerate()
            .filter(|&(other, beside)| other != index && beside.wall == opening.wall)
            .map(|(_, beside)| (beside.on, beside.span, true))
            .collect();
        // Dragged to where it is, with everything else in its wall, it
        // stays where it is: a door on the floor of its storey, a window
        // at its sill.
        let (on, span, mount) = (opening.on, opening.span, opening.model.def.mount);
        let height = match mount {
            Mount::Window => Height::Share {
                share: WINDOW_SILL,
                near: on.up + (span.above - span.below) * 0.5,
            },
            _ => Height::At(on.up),
        };
        let fits = wall.fit(mount, span, height, on.along, on.face, &others);
        if !fits.is_some_and(|(along, up)| (along - on.along).abs() < SAME && (up - on.up).abs() < SAME) {
            faults.push(format!(
                "{} does not fit wall {} at {} along and {} up: the wall would have it at {fits:?}",
                called(opening.stood),
                opening.wall,
                on.along,
                on.up
            ));
        }
        for beside in hung[index + 1..].iter().filter(|beside| beside.wall == opening.wall) {
            let (a, b) = (opening.hole(), beside.hole());
            if a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3] {
                faults.push(format!("{} and {} overlap in wall {}", called(opening.stood), called(beside.stood), opening.wall));
            }
        }
    }
    // Each wall cut round its holes, as the game cuts it
    // (`super::carve_walls`).
    for (number, wall) in walls.iter_mut().enumerate() {
        let mut holes: Vec<[f32; 4]> = hung.iter().filter(|opening| opening.wall == number).map(Hung::hole).collect();
        holes.sort_by(|a, b| a[0].total_cmp(&b[0]));
        wall.holes = holes;
    }

    // ------------------------------ everything as the town's shape has it
    // The house as `super::reshape_plot` reads it: all of it but its walls
    // in one, with the glass of its windows, and each wall on its own.
    let mut shell: Vec<[Vec3; 3]> = house.shell.iter().map(|triangle| triangle.map(|corner| home.transform_point(corner))).collect();
    for window in hung.iter().filter(|opening| opening.model.def.mount == Mount::Window) {
        if let Some(wall) = walls.get(window.wall) {
            let place = GlobalTransform::from(home * wall.place(true, window.on));
            island::add_mesh(&mut shell, &pane(window.span, wall), &place);
        }
    }
    let mut house_parts = vec![shell];
    for wall in &walls {
        let mut part = Vec::new();
        island::add_mesh(&mut part, &wall_mesh(wall), &GlobalTransform::from(home));
        house_parts.push(part);
    }
    let rooms = Rooms(house.floors.clone());
    let things: Vec<Down> = standing
        .into_iter()
        .map(|(stood, model, place)| Down {
            stood,
            model,
            place,
            part: model.triangles.iter().map(|triangle| triangle.map(|corner| place.transform_point(corner))).collect(),
            inside: rooms.hold(into_house.transform_point3(place.translation), house.bounds.high.y),
        })
        .collect();
    let storeys: Vec<Storey> = house
        .floors
        .iter()
        .map(|&(low, high)| Storey {
            into: into_house,
            low,
            high,
            top: home.translation.y + high.y,
        })
        .collect();
    // The plot's sign: the box the game stands for it, and its footprint.
    let sign = land::solid(plot);
    let mut sign_shape: Vec<Vec2> = Vec::new();
    for corner in sign.iter().flatten().map(|corner| corner.xz()) {
        if sign_shape.iter().all(|had| had.distance(corner) > SAME) {
            sign_shape.push(corner);
        }
    }
    let middle_of_sign = sign_shape.iter().copied().sum::<Vec2>() / sign_shape.len().max(1) as f32;
    let round = |corner: &Vec2| (corner.x - middle_of_sign.x).atan2(corner.y - middle_of_sign.y);
    sign_shape.sort_by(|a, b| round(a).total_cmp(&round(b)));
    if sign_shape.len() != 4 {
        faults.push(format!("the sign's box has {} corners across the ground", sign_shape.len()));
        return faults;
    }

    // ------------------------------------------------------ the house
    // Where a house may be put down, with the sign and whatever stands
    // out on the lawn there already: what stands in it is carried in with
    // it (`stand::Rides`).
    let lawn: Vec<&[[Vec3; 3]]> = std::iter::once(sign.as_slice())
        .chain(things.iter().filter(|thing| !thing.inside).map(|thing| thing.part.as_slice()))
        .collect();
    town.lay_over(Some(Island::of_parts_for_tests(&lawn)));
    let (at, turn) = (home.translation, home.rotation);
    if !yard.holds_piece(at, house.bounds, turn) {
        faults.push("the house is not wholly on the plot".into());
    }
    if let Some(why) = misfit(&*town, &*town, yard, Mount::House, at, turn, house.bounds, &[]) {
        faults.push(format!("the house may not be put down where it is: {why:?}"));
    }
    match house_fits_in(town, Some(yard), house.bounds, turn, at.xz()) {
        Some(base) if (base - at.y).abs() < 0.02 => {}
        Some(base) => faults.push(format!("the house stands {} up, where the land is {base} up", at.y)),
        None => faults.push("the house does not fit where it stands".into()),
    }
    let from_the_sign = apart(&house_shape, &sign_shape);
    if from_the_sign < CLEAR_OF_THE_SIGN {
        faults.push(format!("the house is {from_the_sign} m from the sign"));
    }

    // -------------------------------------- what stands on the floors
    for (index, thing) in things.iter().enumerate() {
        // The town's shape with everything else down, as it is while this
        // one is in someone's hand.
        let others = things.iter().enumerate().filter(|&(other, _)| other != index).map(|(_, other)| other);
        let parts: Vec<&[[Vec3; 3]]> = std::iter::once(sign.as_slice())
            .chain(house_parts.iter().map(Vec::as_slice))
            .chain(others.clone().map(|other| other.part.as_slice()))
            .collect();
        town.lay_over(Some(Island::of_parts_for_tests(&parts)));
        // What it can be pushed flush against, as `super::stops_on` has
        // it: everything else on a floor, and the house's walls.
        let stops: Vec<Stop> = others
            .map(|other| Stop {
                place: other.place.compute_affine(),
                low: other.model.bounds.low,
                high: other.model.bounds.high,
                lines_up: true,
            })
            .chain(walls.iter().map(|wall| Stop {
                place: home.compute_affine(),
                low: wall.low,
                high: wall.high,
                lines_up: false,
            }))
            .collect();
        let (at, turn, bounds) = (thing.place.translation, thing.place.rotation, thing.model.bounds);
        if let Some(why) = misfit(&*town, &*town, yard, Mount::Floor, at, turn, bounds, &stops) {
            faults.push(format!("{} may not be put down where it is: {why:?}", called(thing.stood)));
        }
        let floor = floor_under(yard, &storeys, at.xz(), at.y + LAND_STEP_UP);
        if (floor - at.y).abs() > SAME {
            faults.push(format!("{} stands {} up, where the floor under it is {floor} up", called(thing.stood), at.y));
        }
        let shape = footprint(&thing.place, bounds);
        let from_the_sign = apart(&shape, &sign_shape);
        if from_the_sign < CLEAR_OF_THE_SIGN {
            faults.push(format!("{} is {from_the_sign} m from the sign", called(thing.stood)));
        }
        // On the lawn, it is clear of the house, eaves and all.
        if !thing.inside && apart(&shape, &house_shape) < 0.25 {
            faults.push(format!("{} stands on the lawn under the house", called(thing.stood)));
        }
    }

    // -------------------------------------------- walking about on it
    // The town's shape with all of it down.
    let parts: Vec<&[[Vec3; 3]]> = std::iter::once(sign.as_slice())
        .chain(house_parts.iter().map(Vec::as_slice))
        .chain(things.iter().map(|thing| thing.part.as_slice()))
        .collect();
    town.lay_over(Some(Island::of_parts_for_tests(&parts)));
    let town = &*town;
    // Where someone at `at` has their feet, if a body has room there: on
    // the land, or on the ground floor of the house.
    let stands = |at: Vec2| {
        let feet = floor_under(yard, &storeys, at, yard.ground + LAND_STEP_UP);
        let body = [(PLAYER_RADIUS, feet + LAND_STEP_UP, feet + PLAYER_HEIGHT)];
        town.walls_round(at, &body).is_empty().then_some(feet)
    };
    if stands(yard.middle).is_none() {
        faults.push("the middle of the plot, where its owner arrives, is in something".into());
    }
    // Everywhere on the plot that is walked to from its middle, a square
    // at a time: each square's middle, where a body has room.
    let across = (2.0 * yard.half / CELL).round() as usize;
    let own = yard.place();
    let into_plot = own.compute_affine().inverse();
    let spot = |column: usize, row: usize| {
        let at = |nth: usize| -yard.half + (nth as f32 + 0.5) * CELL;
        own.transform_point(Vec3::new(at(column), 0.0, at(row))).xz()
    };
    let square = |at: Vec2| {
        let here = into_plot.transform_point3(Vec3::new(at.x, yard.ground, at.y));
        let nth = |far: f32| ((far + yard.half) / CELL).floor();
        let (column, row) = (nth(here.x), nth(here.z));
        (column >= 0.0 && row >= 0.0 && column < across as f32 && row < across as f32).then_some((column as usize, row as usize))
    };
    let mut walked: Vec<Option<f32>> = vec![None; across * across];
    let mut tried = vec![false; across * across];
    let mut next: Vec<(usize, usize)> = Vec::new();
    for (column, row) in [(across / 2 - 1, across / 2 - 1), (across / 2, across / 2 - 1), (across / 2 - 1, across / 2), (across / 2, across / 2)] {
        tried[row * across + column] = true;
        if let Some(feet) = stands(spot(column, row)) {
            walked[row * across + column] = Some(feet);
            next.push((column, row));
        }
    }
    while let Some((column, row)) = next.pop() {
        let from = walked[row * across + column].unwrap_or(yard.ground);
        let beside = [(column.wrapping_sub(1), row), (column + 1, row), (column, row.wrapping_sub(1)), (column, row + 1)];
        for (column, row) in beside.into_iter().filter(|&(column, row)| column < across && row < across) {
            if std::mem::replace(&mut tried[row * across + column], true) {
                continue;
            }
            if let Some(feet) = stands(spot(column, row)).filter(|feet| (feet - from).abs() <= LAND_STEP_UP) {
                walked[row * across + column] = Some(feet);
                next.push((column, row));
            }
        }
    }
    // Most of a plot is lawn: a walk that got nowhere would say nothing.
    let afoot = walked.iter().flatten().count();
    if afoot < across * across / 4 {
        faults.push(format!("only {afoot} squares of the plot are walked to from its middle"));
    }
    // Whether the square `at` is in has been walked to, or one beside it.
    let walked_to = |at: Vec2| {
        square(at).is_some_and(|(column, row)| {
            let about = |nth: usize| nth.saturating_sub(1)..=(nth + 1).min(across - 1);
            about(row).any(|row| about(column).any(|column| walked[row * across + column].is_some()))
        })
    };
    // Out of every door, over ground that is clear and the plot's own; and
    // on to the front edge of the plot.
    let doors: Vec<&Hung> = hung.iter().filter(|opening| opening.model.def.mount == Mount::Door).collect();
    if doors.is_empty() {
        faults.push("the house has no door".into());
    }
    for door in &doors {
        let Some(wall) = walls.get(door.wall) else {
            continue;
        };
        // In the house's space, so far out from the wall's outer face.
        let out = |along: f32, far: f32| {
            let at = wall.point(along, 0.0, 0.0) - wall.way_in() * (wall.thin().1 + far);
            home.transform_point(at).xz()
        };
        for far in [1.0, DOOR_FRONT] {
            let at = out(door.on.along, far);
            if !yard.holds(at, -PLAYER_RADIUS) {
                faults.push(format!("{} opens off the plot", called(door.stood)));
            } else if !walked_to(at) {
                faults.push(format!("nobody walks from the middle of the plot to {far} m out of {}", called(door.stood)));
            }
        }
        let half = door.span.half;
        let front = [out(door.on.along - half, 0.0), out(door.on.along + half, 0.0), out(door.on.along + half, DOOR_FRONT), out(door.on.along - half, DOOR_FRONT)];
        for thing in things.iter().filter(|thing| !thing.inside) {
            if apart(&footprint(&thing.place, thing.model.bounds), &front) <= 0.0 {
                faults.push(format!("{} stands in front of {}", called(thing.stood), called(door.stood)));
            }
        }
        if apart(&sign_shape, &front) <= 0.0 {
            faults.push(format!("the sign stands in front of {}", called(door.stood)));
        }
    }
    if !(0..across).any(|column| walked[column].is_some()) {
        faults.push("nobody walks from the middle of the plot to its front edge".into());
    }
    // Nothing stands between the sign and the front edge it is read from.
    let ahead = |corner: &Vec2| (*corner - yard.middle).dot(yard.front);
    let mut fore = sign_shape.clone();
    fore.sort_by(|a, b| ahead(a).total_cmp(&ahead(b)));
    let to_the_edge = yard.front * (yard.half - ahead(&fore[0]));
    let before_the_sign = [fore[0], fore[1], fore[1] + to_the_edge, fore[0] + to_the_edge];
    for thing in &things {
        if apart(&footprint(&thing.place, thing.model.bounds), &before_the_sign) <= 0.0 {
            faults.push(format!("{} stands in front of the sign", called(thing.stood)));
        }
    }

    // --------------------------------------------- resting, and floor
    // Whether there is floor beside a thing for someone who has walked
    // there to stand on: out from the point (`x`, `z`) of its edge, in its
    // own space, the way `out`, with their middle from just clear of it to
    // [`BESIDE`] out.
    let floor_beside = |thing: &Down, (x, z): (f32, f32), out: (f32, f32)| {
        (1..=7).map(|step| PLAYER_RADIUS + 0.05 * step as f32).take_while(|far| *far <= BESIDE + 1e-4).any(|far| {
            let at = thing.point(x + out.0 * far, z + out.1 * far);
            stands(at).is_some() && walked_to(at)
        })
    };
    let clear = |from: Vec3, to: Vec3| town.sweep(from, to, 0.05).is_none();
    let (mut beds, mut seats, mut fronted) = (0, 0, 0);
    for thing in &things {
        let places = rest::spots(thing.kind());
        if places.is_empty() {
            continue;
        }
        let bounds = thing.model.bounds;
        let is_bed = places.iter().any(|place| place.pose == roundtown_net::Pose::Lie);
        // In the house, on its ground floor.
        let housed = thing.inside && thing.place.translation.y - home.translation.y < 1.0;
        // The game offers each of its places to someone who has walked up
        // to it (`rest::offered`): from some square walked to.
        let mut offered = vec![false; places.len()];
        for row in 0..across {
            for column in 0..across {
                let Some(feet) = walked[row * across + column] else {
                    continue;
                };
                let at = spot(column, row);
                if at.distance(thing.place.translation.xz()) > 6.0 {
                    continue;
                }
                let you = Vec3::new(at.x, feet, at.y);
                for place in rest::offered(you, &thing.place, &bounds, places, false, &[], &clear) {
                    offered[place] = true;
                }
            }
        }
        if offered.contains(&false) {
            faults.push(format!("nobody gets to rest on {}: of its places, the game offers {offered:?}", called(thing.stood)));
        }
        if is_bed {
            // Floor beside it, along one of its long sides.
            if housed {
                beds += 1;
            }
            if !(floor_beside(thing, (bounds.high.x, 0.0), (1.0, 0.0)) || floor_beside(thing, (bounds.low.x, 0.0), (-1.0, 0.0))) {
                faults.push(format!("there is no floor beside {}", called(thing.stood)));
            }
            continue;
        }
        if housed {
            seats += 1;
        }
        // Floor in front of each of its places; or, of a chair drawn up to
        // a table, behind it or beside it.
        let in_front = places.iter().all(|place| floor_beside(thing, (place.at.x, bounds.low.z), (0.0, -1.0)));
        let before = thing.point(0.0, bounds.low.z - 0.1);
        let at_a_table = things.iter().any(|other| {
            let own = other.place.compute_affine().inverse().transform_point3(Vec3::new(before.x, other.place.translation.y, before.y));
            let (low, high) = (other.model.bounds.low, other.model.bounds.high);
            other.kind() == "circletable" && own.x > low.x && own.x < high.x && own.z > low.z && own.z < high.z
        });
        let about = floor_beside(thing, (0.0, bounds.high.z), (0.0, 1.0))
            || floor_beside(thing, (bounds.high.x, 0.0), (1.0, 0.0))
            || floor_beside(thing, (bounds.low.x, 0.0), (-1.0, 0.0));
        if in_front && housed {
            fronted += 1;
        }
        if !(in_front || (at_a_table && about)) {
            faults.push(format!("there is no floor in front of {}", called(thing.stood)));
        }
    }
    if beds != 1 {
        faults.push(format!("{beds} beds stand in the house"));
    }
    if seats == 0 || fronted == 0 {
        faults.push(format!("{seats} seats stand in the house, {fronted} of them with floor in front"));
    }

    // -------------------------------------------- paint, and triangles
    for (index, paint) in lot.paints.iter().enumerate() {
        let Some(stood) = lot.stands.iter().find(|stood| stood.id == paint.id) else {
            faults.push(format!("{:?} is painted on nothing that stands", paint.part));
            continue;
        };
        if !model_of(&stood.kind).is_some_and(|model| model.materials.contains(&paint.part)) {
            faults.push(format!("{} has no part called {:?} to paint", called(stood), paint.part));
        }
        if lot.paints[..index].iter().any(|other| other.id == paint.id && other.part == paint.part) {
            faults.push(format!("{:?} of {} is painted twice", paint.part, called(stood)));
        }
    }
    let drawn = triangles(lot);
    if drawn > MOST_ON_A_PLOT {
        faults.push(format!("{drawn} triangles are drawn for it"));
    }
    faults
}

/// How many triangles `lot` is drawn with ([`TRIANGLES`]).
fn triangles(lot: &Lot) -> usize {
    lot.stands
        .iter()
        .map(|stood| TRIANGLES.iter().find(|(kind, _)| *kind == stood.kind).map_or(usize::MAX / 64, |&(_, count)| count))
        .sum()
}

/// Holds the lots of the plots numbered `from` and after, up to `to`, each
/// on its own plot of the town.
fn check(from: usize, to: usize) {
    let mut town = town();
    let mut wrong: Vec<String> = Vec::new();
    for number in from..to {
        let lot = premade::lot(number);
        for fault in faults(&lot, &town.plots[number], &mut town.island, &town.models) {
            wrong.push(format!("plot {number} ({}'s): {fault}", lot.name));
        }
    }
    assert!(wrong.is_empty(), "{} things wrong with the made-up lots:\n{}", wrong.len(), wrong.join("\n"));
}

#[test]
fn the_lots_round_the_pond_stand_by_the_game_s_own_rules() {
    check(0, 12);
}

#[test]
fn the_lots_of_the_ring_behind_them_stand_by_the_game_s_own_rules() {
    check(12, 28);
}

#[test]
fn the_lots_of_the_outer_ring_stand_by_the_game_s_own_rules() {
    check(28, PLOTS);
}

#[test]
fn every_lot_stands_as_well_on_plots_turned_other_ways() {
    // Each is for its own plot, and stands on no other. But a plot is a
    // plot: on Hajun's 1, 6, 10, 21 and 45, which lie square to the map
    // and aslant, round the pond, behind it and in a corner of the town,
    // every one of the 48 stands by the same rules. A thread to a plot,
    // each with the town to itself.
    let wrong: Vec<String> = std::thread::scope(|scope| {
        let plots = [0, 5, 9, 20, 44].map(|on| {
            scope.spawn(move || {
                let mut town = town();
                let mut wrong = Vec::new();
                for number in 0..PLOTS {
                    let lot = premade::lot(number);
                    for fault in faults(&lot, &town.plots[on], &mut town.island, &town.models) {
                        wrong.push(format!("the lot of plot {number} on plot {on}: {fault}"));
                    }
                }
                wrong
            })
        });
        plots.into_iter().flat_map(|plot| plot.join().unwrap_or_else(|_| vec!["a plot's check did not finish".into()])).collect()
    });
    assert!(wrong.is_empty(), "{} things wrong with the made-up lots on other plots:
{}", wrong.len(), wrong.join("
"));
}

#[test]
fn a_lot_the_game_would_not_let_stand_is_found_out() {
    // The same lot with one thing wrong at a time: the rules above are
    // not rules that anything passes.
    let mut town = town();
    let good = premade::lot(0);
    let mut said = |lot: &Lot| faults(lot, &town.plots[0], &mut town.island, &town.models).join("; ");
    assert_eq!(said(&good), "");
    let at = |lot: &Lot, kind: &str| lot.stands.iter().position(|stood| stood.kind == kind).unwrap_or_else(|| panic!("plot 0 has no {kind}"));
    let moved = |kind: &str, to: At| {
        let mut lot = good.clone();
        let which = at(&lot, kind);
        lot.stands[which].at = to;
        lot
    };
    let on_the_ground = |x: f32, y: f32, z: f32| At::Ground { x, y, z, yaw: 0.0 };
    // The house over the sign, and half off the plot.
    assert!(said(&moved("square_house", on_the_ground(-5.5, 0.0, -5.5))).contains("the house"));
    assert!(said(&moved("square_house", on_the_ground(9.0, 0.0, 0.0))).contains("not wholly on the plot"));
    // The bed in the house's back wall, in the couch, off the floor, and
    // out on the lawn.
    assert!(said(&moved("doublebed", on_the_ground(3.95, 0.1, 8.5))).contains("may not be put down"));
    assert!(said(&moved("doublebed", on_the_ground(-4.0, 0.1, 3.9))).contains("may not be put down"));
    assert!(said(&moved("doublebed", on_the_ground(3.95, 0.6, 7.72))).contains("where the floor under it is"));
    assert!(said(&moved("doublebed", on_the_ground(-8.0, 0.0, 0.0))).contains("0 beds stand in the house"));
    // The lamp against the sign, and at the foot of the door.
    assert!(said(&moved("streetlamp", on_the_ground(-6.0, 0.0, -10.5))).contains("from the sign"));
    assert!(said(&moved("streetlamp", on_the_ground(-2.6, 0.0, -2.5))).contains("in front of"));
    // The door where the window is, half out of the end of its wall, and
    // in a wall there is not; and a window off its sill.
    let in_a_wall = |wall: u8, along: f32, up: f32| At::Wall { wall, along, up, face: 1 };
    assert!(said(&moved("classic_door_with_window", in_a_wall(0, 2.0, 0.1))).contains("does not fit"));
    assert!(said(&moved("classic_door_with_window", in_a_wall(0, 5.2, 0.1))).contains("does not fit"));
    assert!(said(&moved("classic_door_with_window", in_a_wall(7, 0.0, 0.1))).contains("which the house has not"));
    assert!(said(&moved("wide_window", in_a_wall(3, -2.2, 2.5))).contains("does not fit"));
    // A wall where the door was: nobody gets out of the house, whose
    // floor is all that is walked to.
    let mut shut = good.clone();
    let door = at(&shut, "classic_door_with_window");
    shut.stands[door].kind = "square_window".into();
    shut.stands[door].at = in_a_wall(2, -3.0, 1.2);
    let wrong = said(&shut);
    assert!(wrong.contains("the house has no door") && wrong.contains("to its front edge") && wrong.contains("squares of the plot are walked to"), "{wrong}");
    // The couch across the doorway, inside: nobody gets out either.
    let wrong = said(&moved("softcouch", on_the_ground(-2.6, 0.1, 0.1)));
    assert!(wrong.contains("m out of classic_door_with_window") && wrong.contains("to its front edge"), "{wrong}");
    // Paint on a part its model has not, and more chairs than a phone
    // draws easily.
    let mut painted = good.clone();
    painted.paints.push(bag::Paint {
        id: 1,
        part: "plain_door".into(),
        colour: [1, 2, 3],
    });
    assert!(said(&painted).contains("no part called"));
    let mut crowded = good.clone();
    for nth in 0..2 {
        let id = crowded.stands.len() as u32 + 1;
        crowded.stands.push(Standing {
            id,
            kind: "cafechair".into(),
            at: on_the_ground(-9.0, 0.0, 2.0 + nth as f32 * 2.0),
        });
    }
    assert!(said(&crowded).contains("triangles are drawn"));
}

#[test]
fn the_sign_stands_at_the_front_left_of_a_plot_s_own_space() {
    // What `premade` takes a plot for: 24 m square about its own origin,
    // its ground at nothing, its front toward -z, and its sign a metre
    // and a half in from the front edge and from the corner on the left
    // of someone in the middle looking out over the front.
    let town = town();
    for (number, plot) in town.plots.iter().enumerate() {
        let yard = yard_of(plot);
        assert!((yard.half - 12.0).abs() < 0.01, "plot {number}: {} m to a side", 2.0 * yard.half);
        let into = yard.place().compute_affine().inverse();
        let middle = into.transform_point3(Vec3::new(plot.sign.x, plot.ground, plot.sign.y));
        assert!(middle.distance(Vec3::new(-8.97, 0.0, -10.5)) < 0.02, "plot {number}: its sign at {middle}");
        let corners: Vec<Vec3> = land::solid(plot).iter().flatten().map(|&corner| into.transform_point3(corner)).collect();
        let low = corners.iter().copied().reduce(Vec3::min).unwrap_or_default();
        let high = corners.iter().copied().reduce(Vec3::max).unwrap_or_default();
        assert!(low.distance(Vec3::new(-10.5, -0.1, -10.65)) < 0.02 && high.distance(Vec3::new(-7.44, 2.73, -10.35)) < 0.02, "plot {number}: {low} to {high}");
        // Out over the front is out of the plot, toward its street or the
        // pond: -z, and nothing of the town stands on the plot.
        let before = placed(&yard, 0.0, 0.0, -13.0, 0.0).translation.xz();
        assert!(!yard.holds(before, 0.0) && (before - plot.middle).normalize().distance(plot.front) < 1e-4, "plot {number}");
        let body = [(PLAYER_RADIUS, plot.ground + LAND_STEP_UP, plot.ground + PLAYER_HEIGHT)];
        for (x, z) in [(0.0, 0.0), (-11.0, 11.0), (11.0, 11.0), (11.0, -11.0), (-11.0, -11.0), (6.0, -6.0)] {
            let at = placed(&yard, x, 0.0, z, 0.0).translation;
            let ground = town.island.floor(at.xz(), f32::INFINITY);
            assert!(ground.is_some_and(|ground| (ground - plot.ground).abs() < 0.01), "plot {number}: {ground:?} up at {at}");
            assert!(town.island.walls_round(at.xz(), &body).is_empty(), "plot {number}: something of the town stands at {at}");
        }
    }
}

#[test]
fn the_pieces_are_drawn_with_as_many_triangles_as_counted() {
    for def in PIECES {
        let model = read(def);
        let counted = TRIANGLES.iter().find(|(kind, _)| *kind == def.kind).map(|&(_, count)| count);
        assert_eq!(Some(model.triangles.len()), counted, "{}", def.kind);
        // A house has four walls, each a box of its own, and a floor: four
        // to a storey, in the houses of three floors and more (Hajun's own,
        // 2026-11-24), but for the roof terrace's two lowest storeys, which
        // are walled as one, as in the Roof Terrace House of two.
        if def.mount == Mount::House {
            let (walls, floors) = (model.walls.len(), model.floors.len());
            assert!(walls >= 4 && walls % 4 == 0 && walls <= 4 * floors && floors > 0, "{}: {walls} walls, {floors} floors", def.kind);
        }
        assert_eq!(model.span.is_some(), def.on_wall(), "{}", def.kind);
    }
    assert_eq!(TRIANGLES.len(), PIECES.len());
}

#[test]
fn all_the_made_up_lots_together_are_light_to_draw_and_no_two_alike() {
    let lots: Vec<Lot> = (0..PLOTS).map(premade::lot).collect();
    let drawn: Vec<usize> = lots.iter().map(triangles).collect();
    let all: usize = drawn.iter().sum();
    let heaviest = drawn.iter().enumerate().max_by_key(|&(_, count)| *count).unwrap_or((0, &0));
    println!("the made-up lots: {all} triangles in all, the most on plot {}, {}", heaviest.0, heaviest.1);
    assert!(all <= MOST_IN_ALL, "{all} triangles in all");
    assert!(drawn.iter().all(|&count| count <= MOST_ON_A_PLOT), "{drawn:?}");
    let has = |lot: &Lot, kind: &str| lot.stands.iter().any(|stood| stood.kind == kind);
    let lit = lots.iter().filter(|lot| has(lot, bag::LAMP)).count();
    assert!(lit <= MOST_LIT, "{lit} plots with a lamp");
    // Every house, door and window there is stands somewhere, and so does
    // every other piece of House Builder's: what came for the town's plots
    // alone on 2026-11-25 stands on nobody's made-up land.
    for def in PIECES.iter().filter(|def| def.house_builder) {
        assert!(lots.iter().any(|lot| has(lot, def.kind)), "no made-up lot has a {}", def.kind);
    }
    // No two alike, whoever they are, and each unlike the one before it in
    // at least three ways: its house, where the house stands, which way it
    // is turned, its door, and its walls' colour.
    let house = |lot: &Lot| lot.stands.iter().find(|stood| stood.kind.starts_with("square_house")).cloned();
    let door = |lot: &Lot| lot.stands.iter().find(|stood| stood.kind.contains("door")).map(|stood| stood.kind.clone());
    let walls = |lot: &Lot| lot.paints.iter().find(|paint| paint.part == "plain_wall").map(|paint| paint.colour);
    let (mut flat, mut eighths) = (0, 0);
    for (number, lot) in lots.iter().enumerate() {
        assert!(lots[..number].iter().all(|other| other.stands != lot.stands), "plot {number} is another over again");
        let Some(Standing { kind, at: At::Ground { x, z, yaw, .. }, .. }) = house(lot) else {
            panic!("plot {number} has no house on the ground");
        };
        // A quarter turn or none, or an eighth.
        let quarters = yaw / std::f32::consts::FRAC_PI_2;
        if (quarters - quarters.round()).abs() > 0.01 {
            eighths += 1;
        }
        flat += usize::from(kind == "square_house_roofless");
        let Some(before) = number.checked_sub(1).map(|before| &lots[before]) else {
            continue;
        };
        let Some(Standing { kind: kind_before, at: At::Ground { x: x_before, z: z_before, yaw: yaw_before, .. }, .. }) = house(before) else {
            continue;
        };
        let differences = [kind != kind_before, (x, z) != (x_before, z_before), yaw != yaw_before, door(lot) != door(before), walls(lot) != walls(before)];
        let unlike = differences.iter().filter(|differs| **differs).count();
        assert!(unlike >= 3, "plots {} and {number} are alike in all but {unlike} ways", number - 1);
    }
    // Some turned an eighth, and the plain house and the flat one the
    // most of them.
    let square = lots.iter().filter(|lot| has(lot, "square_house")).count();
    assert!(eighths >= 4 && square >= 14 && flat >= 12, "{eighths} turned an eighth, {square} Square Houses, {flat} Flat Houses");
    // The sixteen someone walks have the most in them.
    let things = |lots: &[Lot]| lots.iter().map(|lot| lot.stands.len()).sum::<usize>() as f32 / lots.len() as f32;
    assert!(things(&lots[..16]) > things(&lots[16..]), "{} things a plot at the front, {} behind", things(&lots[..16]), things(&lots[16..]));
}
