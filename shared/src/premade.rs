//! The made-up land: what fills a plot of the town when no real person's
//! land does (Hajun, 2026-11-23: "then, if there are still unfilled plots,
//! fill with fake plots that are premade"). Forty-eight of them, as many
//! as the town has plots, the same on every device and on the server; which
//! plot each is on is thrown as a channel opens (since 2026-11-24,
//! [`crate::land::Order`]; each plot had its own for the day before). Each
//! has an owner of its own, by a name of the computer's
//! ([`crate::BOY_NAMES`], [`crate::GIRL_NAMES`]) and in a look of that
//! side, whom someone of the computer's walks the land as
//! ([`crate::land::Holder::Minded`]).
//!
//! **Each is a furnished plot**, as someone might have left theirs: one of
//! seven houses, standing somewhere on the plot and turned some way
//! ([`Plan`]), with a door and windows in its walls and a bed, a couch and
//! more on its ground floor ([`Room`]), on some a lamp, a garden or
//! somewhere to sit out on the lawn ([`LAWNS`]), and its own colours
//! ([`Coat`]). No two are the same.
//!
//! **And each is its owner's, in their taste** (since 2026-11-25; Hajun:
//! "make each npc's plot have a coherent theme/personality for each one so
//! that its not just "diverse" randomly but its like the owner's design
//! taste is "diverse""): sixteen tastes, three owners each ([`TASTES`]),
//! each the things of its own put in their room's places and on its tables
//! and counters, and its own colours: the cook's stove and fridge and warm
//! walls, the gardener's plants and green, the stargazer's telescope up on
//! the roof terrace, the athlete's treadmill, the minimalist's next to
//! nothing. Their houses are low: two thirds of one floor, six of three, none
//! higher ("keep it not to rich so its not like 10 floor buildings").
//!
//! **All of it is written here by hand, in tables**, and put together by
//! sums that come out the same on every machine ([`made`]): a turn is so
//! many eighths of a whole one, which is what a tap on one of the game's
//! turn buttons turns a thing, each with its sine and cosine written out
//! ([`EIGHTHS`]); nothing is thrown, and nothing is measured as the app
//! runs. The 48 are made once ([`lot`]).
//!
//! **Where a thing may stand is the game's to say**, which has the models:
//! this crate has only their names. A test of the game's
//! (`build::premade_check`) stands every lot on its own plot of the town's
//! map and holds it to the game's own rules: the house where a house fits,
//! clear of the plot's sign; each door and window where its wall has room
//! for it; nothing in a wall or in anything else; the middle of the plot,
//! where its owner arrives, on free floor; a way on foot from there out of
//! the door, to the bed and to every seat; every painted part a part of
//! its model; and no more triangles than a phone draws easily (5,000 to a
//! plot, 190,000 in all). **Change a number here and run that test.**
//!
//! Two things about the game's shapes that the rooms keep clear of, for
//! whoever changes one. The Square House's front gable is modelled inside
//! out, and the line under its ridge, through the middle of its front
//! wall, can read as solid to a body (`build::reshape_plot`): a door in
//! that wall stands to one side of the middle. And the long edge of a
//! counter is bevelled, in triangles so thin that the game takes them to
//! run on a few millimetres past their ends: a counter set flush, end on,
//! to the side of another reads as standing in it, so a corner piece goes
//! between the two.
//!
//! A place in a room is in its house's own space, as the game has a
//! model's: the house's middle on its floor at the origin, its front wall
//! toward -z, and x to the right of someone inside looking out of that
//! wall. The rooms are 11 m square inside walls half a metre thick. A
//! house of two floors has its stairs up the wall at x = -5.5, from 4.25 m
//! in front of the middle to the back wall, which leaves 9 m of floor
//! across.

use std::f32::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4, PI};
use std::sync::OnceLock;

use crate::At;
use crate::bag::{LAMP, Paint, Standing};
use crate::land::{Lot, PLOTS, PREMADE};

/// The made-up lot numbered `plot` (of [`PLOTS`], round and round past
/// them): the one of that place of a channel
/// ([`Order`](crate::land::Order)), which is on another plot of the town
/// each time a channel opens (since 2026-11-24; the plot of that number's
/// own for the day before). All of them are put together the first time
/// one is asked for, and every one handed out is a copy.
pub fn lot(plot: usize) -> Lot {
    static LOTS: OnceLock<Vec<Lot>> = OnceLock::new();
    LOTS.get_or_init(|| (0..PLOTS).map(made).collect())[plot % PLOTS].clone()
}

/// The made-up lot that is `who`'s, if they are one of the made-up owners.
pub fn of(who: u64) -> Option<Lot> {
    crate::land::is_premade(who).then(|| lot((who - PREMADE) as usize))
}

/// Who the made-up lot of `plot` belongs to: a boy on the even plots and a
/// girl on the odd, each by a name of their side nobody else of them has,
/// in one of that side's looks.
fn owner(plot: usize) -> (&'static str, &'static str) {
    let boy = plot % 2 == 0;
    let nth = plot / 2;
    let names = crate::ai_names(boy);
    (names[nth % names.len()], crate::ai_look(boy, nth))
}

// ------------------------------------------------------------ the pieces

/// The houses the made-up owners have, by the names the game's pieces go
/// by: Hajun's first four (the Square House, the Flat House, the Two Floor
/// House and the Roof Terrace House), and since 2026-11-25 the Wide Roof
/// House and the Square House and the Roof Terrace of three floors. None
/// higher (Hajun: "keep it not to rich so its not like 10 floor buildings
/// ... floor height should be biased towards lower heights").
const HOUSES: [&str; 7] = [
    "square_house",
    "square_house_roofless",
    "square_house_two_floor",
    "square_house_roofless_two_floor",
    "wide_roof_house",
    "square_house_three_floor",
    "square_house_roofless_three_floor",
];
const SQUARE: usize = 0;
const FLAT: usize = 1;
const TALL: usize = 2;
const TERRACE: usize = 3;
const WIDE: usize = 4;
const TALL3: usize = 5;
const TERRACE3: usize = 6;

/// Their five doors.
const DOORS: [&str; 5] = [
    "classic_door",
    "classic_door_with_window",
    "classic_double_door",
    "glass_door",
    "glass_double_door",
];
const CLASSIC: usize = 0;
const WINDOWED: usize = 1;
const DOUBLE: usize = 2;
const GLASS: usize = 3;
const GLASS_DOUBLE: usize = 4;

/// Their two windows, and what of Hajun's own stands on a floor.
const SQUARE_WINDOW: &str = "square_window";
const WIDE_WINDOW: &str = "wide_window";
const SINGLE_BED: &str = "singlebed";
const DOUBLE_BED: &str = "doublebed";
const COUCH: &str = "softcouch";
const CHAIR: &str = "cafechair";
const TABLE: &str = "circletable";
const DESK: &str = "cafedeskedge";
const DESK_END: &str = "cafedeskcorner";

/// How high over a house's foot its ground floor is, which is what
/// stands in the house stands on, and how much higher each floor over
/// that is, in metres.
const FLOOR: f32 = 0.1;
const STOREY: f32 = 5.0;
/// How many storeys of each house a door or a window goes in: the walls
/// of the Flat House and of the roof terraces end under their tops.
const STOREYS: [u8; 7] = [1, 1, 2, 1, 1, 3, 2];
/// Whether each storey of a house has walls of its own, which the game
/// numbers storey by storey along each side (the houses of three floors),
/// rather than one wall a side all the way up.
const STACKED: [bool; 7] = [false, false, false, false, false, true, true];
/// How high over each house's foot the sill of a window is on each
/// storey: 0.22 of the way up the storey, where the game stands every
/// window (`build::WINDOW_SILL`). A storey under a roof is 5 m of wall; one
/// under another floor, 4.9.
const SILLS: [[f32; 3]; 7] = [
    [1.2, 0.0, 0.0],
    [1.178, 0.0, 0.0],
    [1.178, 6.2, 0.0],
    [1.178, 0.0, 0.0],
    [1.2, 0.0, 0.0],
    [1.178, 6.178, 11.2],
    [1.178, 6.178, 0.0],
];
/// How high over a house's foot the top of a roof terrace is, which what
/// stands up there stands on: for each house, if it has one.
const ROOF_TOP: [Option<f32>; 7] = [None, Some(5.1), None, Some(5.1), None, None, Some(10.1)];

/// The eight ways a thing is turned about the upright: so many eighths of
/// a whole turn, each with its angle as [`At::Ground`] has it, its cosine
/// and its sine. Written out, and not worked out as the app runs, so that
/// what is turned stands at the same numbers on every machine.
const EIGHTHS: [(f32, f32, f32); 8] = [
    (0.0, 1.0, 0.0),
    (FRAC_PI_4, FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (FRAC_PI_2, 0.0, 1.0),
    (3.0 * FRAC_PI_4, -FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (PI, -1.0, 0.0),
    (-3.0 * FRAC_PI_4, -FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    (-FRAC_PI_2, 0.0, -1.0),
    (-FRAC_PI_4, FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
];

/// Where the point (`x`, `z`) of something turned `turn` eighths comes
/// to, across the ground, as the game turns a thing about the upright.
fn turned(turn: u8, x: f32, z: f32) -> (f32, f32) {
    let (_, cos, sin) = EIGHTHS[usize::from(turn % 8)];
    (x * cos + z * sin, z * cos - x * sin)
}

/// Standing at (`x`, `y`, `z`), turned `turn` eighths.
fn ground(x: f32, y: f32, z: f32, turn: u8) -> At {
    let (yaw, ..) = EIGHTHS[usize::from(turn % 8)];
    At::Ground { x, y, z, yaw }
}

/// Something that stands: what it is, where its own origin is, how high
/// over its house's foot (nothing: on the floor it is on), and how many
/// eighths it is turned. Turned by nothing, its front looks toward -z.
#[derive(Clone, Copy)]
struct Thing {
    kind: &'static str,
    x: f32,
    y: f32,
    z: f32,
    turn: u8,
}

const fn thing(kind: &'static str, x: f32, z: f32, turn: u8) -> Thing {
    Thing { kind, x, y: 0.0, z, turn }
}

/// A small thing standing on top of something else, `y` over the house's
/// foot ([`TOPS`]).
const fn up(kind: &'static str, x: f32, y: f32, z: f32, turn: u8) -> Thing {
    Thing { kind, x, y, z, turn }
}

/// A door or a window in a wall of a house: what it is, or `None` for the
/// door its plan has ([`Plan::door`]); which side of the house, as the game
/// numbers its walls (0 the front, 1 the left, 2 the back, 3 the right);
/// how far along the wall its middle is, x along the front and the back
/// and z along the other two; and which storey it is in.
#[derive(Clone, Copy)]
struct Opening {
    kind: Option<&'static str>,
    side: u8,
    along: f32,
    storey: u8,
}

/// The door of the house, whichever its plan has.
const fn door(side: u8, along: f32) -> Opening {
    Opening {
        kind: None,
        side,
        along,
        storey: 0,
    }
}

/// A window on the ground floor.
const fn window(kind: &'static str, side: u8, along: f32) -> Opening {
    Opening {
        kind: Some(kind),
        side,
        along,
        storey: 0,
    }
}

/// A window on the floor over it, in a house whose walls go up that far.
const fn upstairs(kind: &'static str, side: u8, along: f32) -> Opening {
    Opening {
        kind: Some(kind),
        side,
        along,
        storey: 1,
    }
}

/// A window on the floor over that, in the Square House of three floors.
const fn top_floor(kind: &'static str, side: u8, along: f32) -> Opening {
    Opening {
        kind: Some(kind),
        side,
        along,
        storey: 2,
    }
}

/// Which of a house's walls is on `side` (0 the front, 1 the left, 2 the
/// back, 3 the right) on `storey`, as the game numbers them
/// (`build::walls_of`): one a side, or a side's storeys one after another.
fn wall_of(house: usize, side: u8, storey: u8) -> u8 {
    if STACKED[house] { side * STOREYS[house] + storey } else { side }
}

/// What is in a house: the door and windows in its walls, and what
/// stands on its ground floor, the bed first, then a couch, and the
/// chairs last, which a plan may leave out ([`Plan::less`]); and the spots
/// it leaves for what its owner's taste adds ([`Taste::extra`]).
struct Room {
    openings: &'static [Opening],
    things: &'static [Thing],
    spots: &'static [Spot],
}

/// Somewhere a room leaves for something its owner adds: the middle of the
/// back of what goes there, against a wall or with its back to the room,
/// and the way it faces, in eighths.
#[derive(Clone, Copy)]
struct Spot {
    x: f32,
    z: f32,
    turn: u8,
}

const fn spot(x: f32, z: f32, turn: u8) -> Spot {
    Spot { x, z, turn }
}

// -------------------------------------------- what each thing takes up

/// How much floor each thing that a taste swaps in or adds takes, in its
/// own space as the game has its model: from its lowest x to its highest,
/// and from its front (-z) to its back. Read off the models
/// (`build::premade_check` reads them again and holds the lots to them).
const FOOTPRINTS: [(&str, f32, f32, f32, f32); 33] = [
    (DESK, -1.0, 1.0, -0.5, 0.5),
    (DESK_END, -0.5, 0.5, -0.5, 0.5),
    (CHAIR, -0.42, 0.42, -0.42, 0.42),
    (COUCH, -1.88, 1.88, -1.06, 0.94),
    ("stove", -0.56, 0.56, -0.82, 0.56),
    ("refrigerator", -0.82, 0.58, -0.73, 0.65),
    ("cabinet", -0.74, 0.74, -0.04, 0.86),
    ("bathroom_sink", -1.32, 0.45, -0.67, 0.44),
    ("chair", -0.44, 0.44, -0.27, 0.56),
    ("stool", -0.27, 0.27, -0.27, 0.27),
    ("armchair", -0.91, 0.56, -0.56, 0.79),
    ("rocking_chair", -0.62, 0.62, -0.89, 0.71),
    ("grandfathers_clock", -0.38, 0.38, -0.18, 0.38),
    ("coat_rack", -0.38, 0.38, -0.38, 0.38),
    ("closed_umbrella", -0.39, 0.42, -0.41, 0.41),
    ("potted_plant", -0.46, 0.42, -0.33, 0.36),
    ("trash_bin", -0.26, 0.26, -0.35, 0.34),
    ("propane_tank", -0.31, 0.31, -0.32, 0.30),
    ("treadmill", -0.80, 0.80, -1.53, 1.56),
    ("dumbbell", -0.58, 0.58, -0.23, 0.23),
    ("empty_box", -0.68, 0.50, -0.50, 0.68),
    ("safe", -0.36, 0.25, -0.20, 0.46),
    ("toilet", -0.32, 0.32, -0.78, 0.38),
    ("bathtub", -2.01, 2.24, -0.95, 0.95),
    ("bed", -1.17, 2.52, -2.44, 2.44),
    ("couch", -1.88, 2.27, -2.03, 1.09),
    ("desk", -2.36, 0.76, -0.79, 0.76),
    ("end_table", -0.77, 0.18, -0.83, 0.77),
    ("table_large_circular", -1.59, 1.59, -1.59, 1.59),
    ("telescope", -0.53, 0.75, -0.94, 0.54),
    ("computer", 0.03, 0.44, -0.42, 0.42),
    ("watering_can", -0.20, 0.20, -0.67, 0.31),
    ("hair_dryer", -0.34, 0.07, -0.33, 0.11),
];

/// The middle of what a thing takes up, in its own space: where its
/// origin is not its middle, the room turned the other way round puts its
/// middle where the other's was.
fn middle_of(kind: &str) -> (f32, f32) {
    FOOTPRINTS
        .iter()
        .find(|(of, ..)| *of == kind)
        .map_or((0.0, 0.0), |&(_, low_x, high_x, low_z, high_z)| ((low_x + high_x) * 0.5, (low_z + high_z) * 0.5))
}

/// The middle of a thing's width and the back of it, in its own space.
fn back_of(kind: &str) -> (f32, f32) {
    FOOTPRINTS
        .iter()
        .find(|(of, ..)| *of == kind)
        .map_or((0.0, 0.0), |&(_, low_x, high_x, _, high_z)| ((low_x + high_x) * 0.5, high_z))
}

/// The middle of a thing's width and the front of it, in its own space.
fn front_of(kind: &str) -> (f32, f32) {
    FOOTPRINTS
        .iter()
        .find(|(of, ..)| *of == kind)
        .map_or((0.0, 0.0), |&(_, low_x, high_x, low_z, _)| ((low_x + high_x) * 0.5, low_z))
}

/// How high over its foot the top of each thing a small thing can stand on
/// is, measured off its model at its middle: a table's, a counter's.
const TOPS: [(&str, f32); 10] = [
    (TABLE, 1.2),
    (DESK, 1.5),
    (DESK_END, 1.5),
    ("table", 1.186),
    ("desk", 1.082),
    ("table_large_circular", 1.129),
    ("dresser", 1.564),
    ("cabinet", 1.477),
    ("end_table", 1.421),
    ("stove", 1.282),
];

fn top_of(kind: &str) -> Option<f32> {
    TOPS.iter().find(|(of, _)| *of == kind).map(|&(_, high)| high)
}

/// What a small thing may be stood on, as a taste names it: a table of any
/// kind, or a counter of any kind.
const TABLES: [&str; 4] = [TABLE, "table", "table_large_circular", "desk"];
const COUNTERS: [&str; 6] = [DESK, DESK_END, "cabinet", "dresser", "end_table", "stove"];

// ------------------------------------------------------------- the rooms

/// The rooms of the Square House and the Flat House, which have the whole
/// 11 m square of floor. Every one keeps its middle clear and a way in
/// from its door, and none has its door in the middle of the front wall
/// but the third, which is the Flat House's alone: the Square House's
/// ridge comes down there (the module's own account). The front of each,
/// where the owners of most of the plots it is on arrive, a metre in from
/// the front wall, is left clear, and so is its middle.
const ONE_FLOOR: [Room; 6] = [
    // The bed in the back right corner, a couch by the left wall, and a kitchen
    // along the back with a table before it.
    Room {
        openings: &[
            door(0, -2.6), window(SQUARE_WINDOW, 0, 2.4), window(WIDE_WINDOW, 3, -2.2),
            window(SQUARE_WINDOW, 2, 3.9), window(SQUARE_WINDOW, 1, 2.9),
        ],
        things: &[
            thing(DOUBLE_BED, 3.95, 3.22, 0), thing(COUCH, -4.53, -0.6, 6), thing(DESK, -4.47, 4.97, 0),
            thing(DESK, -2.47, 4.97, 0), thing(TABLE, -1.0, 2.6, 0), thing(CHAIR, -1.0, 1.1, 4),
        ],
        spots: &[spot(5.47, -1.2, 2), spot(0.4, 5.47, 0)],
    },
    // The bed out from the left wall, a couch at the back, a counter down the
    // right wall, and a table with two chairs by the left.
    Room {
        openings: &[
            door(0, 2.6), window(SQUARE_WINDOW, 0, -2.4), window(SQUARE_WINDOW, 1, -1.2),
            window(SQUARE_WINDOW, 2, -3.2), window(SQUARE_WINDOW, 3, 3.4),
        ],
        things: &[
            thing(SINGLE_BED, -3.22, 3.4, 6), thing(COUCH, 2.6, 4.53, 0), thing(DESK, 4.97, 0.5, 2),
            thing(DESK, 4.97, -1.5, 2), thing(TABLE, -3.4, -1.2, 0), thing(CHAIR, -3.4, 0.2, 0),
            thing(CHAIR, -1.95, -1.25, 2),
        ],
        spots: &[spot(5.47, 2.5, 2), spot(-3.6, 5.47, 0)],
    },
    // The Flat House alone: its door in the middle of the front, the bed in
    // the middle of the back between two cupboards, a couch on the left and a
    // table with two chairs on the right.
    Room {
        openings: &[
            door(0, 0.0), window(SQUARE_WINDOW, 0, -3.9), window(SQUARE_WINDOW, 0, 3.9),
            window(SQUARE_WINDOW, 1, 3.0), window(SQUARE_WINDOW, 3, 3.0),
        ],
        things: &[
            thing(DOUBLE_BED, 0.0, 3.22, 0), thing(COUCH, -4.53, -0.8, 6), thing(DESK, -4.47, 4.97, 0),
            thing(DESK, 4.47, 4.97, 0), thing(TABLE, 3.9, -1.0, 0), thing(CHAIR, 3.9, 0.4, 0),
            thing(CHAIR, 2.45, -1.0, 6),
        ],
        spots: &[spot(-3.3, -5.47, 4), spot(5.47, 2.4, 2)],
    },
    // The bed out from the right wall, a couch at the back, a counter down the
    // left wall, and a table by the right.
    Room {
        openings: &[
            door(0, -2.6), window(SQUARE_WINDOW, 0, 2.4), window(SQUARE_WINDOW, 3, -1.5),
            window(SQUARE_WINDOW, 1, 4.0),
        ],
        things: &[
            thing(DOUBLE_BED, 3.22, 3.9, 2), thing(COUCH, -2.6, 4.53, 0), thing(DESK, -4.97, -0.5, 6),
            thing(DESK, -4.97, 1.5, 6), thing(TABLE, 3.6, -1.4, 0), thing(CHAIR, 3.6, 0.0, 0),
        ],
        spots: &[spot(5.47, 1.4, 2), spot(-5.47, 3.6, 6)],
    },
    // A single bed tucked along the back wall, a couch by the right wall, a
    // counter with a round end at the back, and a table with two chairs.
    Room {
        openings: &[
            door(0, 2.6), window(SQUARE_WINDOW, 0, -2.4), window(SQUARE_WINDOW, 1, -0.6),
            window(WIDE_WINDOW, 3, -2.3), window(SQUARE_WINDOW, 2, -3.2),
        ],
        things: &[
            thing(SINGLE_BED, -3.22, 4.33, 6), thing(COUCH, 4.53, 2.0, 2), thing(DESK, 2.0, 4.97, 0),
            thing(DESK_END, 0.47, 4.97, 0), thing(TABLE, -3.2, -0.6, 0), thing(CHAIR, -3.2, 0.8, 0),
            thing(CHAIR, -1.75, -0.6, 2),
        ],
        spots: &[spot(-5.47, 2.2, 6), spot(4.4, 5.47, 0)],
    },
    // The door in the wall on the right, under the eaves: the bed in the back
    // left corner, a couch against the front wall, a counter at the back and a
    // table before it.
    Room {
        openings: &[
            door(3, -1.5), window(SQUARE_WINDOW, 3, 3.2), window(SQUARE_WINDOW, 0, 2.5),
            window(SQUARE_WINDOW, 2, -3.9), window(SQUARE_WINDOW, 1, -1.5),
        ],
        things: &[
            thing(DOUBLE_BED, -3.95, 3.22, 0), thing(COUCH, -2.5, -4.53, 4), thing(DESK, 1.5, 4.97, 0),
            thing(DESK, 3.5, 4.97, 0), thing(TABLE, 2.6, 1.8, 0), thing(CHAIR, 2.6, 0.3, 4),
        ],
        spots: &[spot(-5.47, -1.0, 6)],
    },
];

/// The rooms of the houses of two and three floors, beside their stairs.
/// Nothing goes in the wall the stairs go up, on the ground floor, and the
/// foot of the stairs is left clear.
const TWO_FLOORS: [Room; 4] = [
    // A single bed in the back right corner, a couch against the stairs, a
    // counter along the back and a table in the middle of the floor.
    Room {
        openings: &[
            door(0, 1.0), window(SQUARE_WINDOW, 0, -2.9), window(WIDE_WINDOW, 3, -2.0),
            upstairs(SQUARE_WINDOW, 0, -2.6), upstairs(SQUARE_WINDOW, 0, 2.6),
            top_floor(SQUARE_WINDOW, 0, 0.0), top_floor(SQUARE_WINDOW, 3, 0.0),
        ],
        things: &[
            thing(SINGLE_BED, 4.3, 3.22, 0), thing(COUCH, -2.53, 2.0, 6), thing(DESK, -2.47, 4.97, 0),
            thing(DESK, -0.47, 4.97, 0), thing(TABLE, 2.2, -0.8, 0), thing(CHAIR, 2.2, 0.6, 0),
        ],
        spots: &[spot(5.47, -2.4, 2)],
    },
    // The bed out from the right wall, a couch against the stairs, a cupboard
    // at the back and a table by the right wall.
    Room {
        openings: &[
            door(0, 1.5), window(SQUARE_WINDOW, 0, -2.4), window(SQUARE_WINDOW, 3, -1.2),
            upstairs(SQUARE_WINDOW, 0, -2.6), upstairs(SQUARE_WINDOW, 3, 0.0),
            top_floor(SQUARE_WINDOW, 0, 2.6), top_floor(SQUARE_WINDOW, 3, 0.0),
        ],
        things: &[
            thing(DOUBLE_BED, 3.22, 3.9, 2), thing(COUCH, -2.53, -0.5, 6), thing(DESK, -2.47, 4.97, 0),
            thing(TABLE, 3.9, -1.2, 0), thing(CHAIR, 3.9, 0.2, 0),
        ],
        spots: &[spot(0.4, 5.47, 0)],
    },
    // The door in the wall on the right, under the eaves: the bed in the
    // middle of the back, a couch against the stairs, a counter round the back
    // right corner and a table by the front wall.
    Room {
        openings: &[
            door(3, -1.5), window(WIDE_WINDOW, 0, 0.0), window(SQUARE_WINDOW, 2, 0.0),
            upstairs(SQUARE_WINDOW, 3, -2.6), upstairs(SQUARE_WINDOW, 3, 2.6),
            top_floor(SQUARE_WINDOW, 3, 0.0), top_floor(SQUARE_WINDOW, 0, 0.0),
        ],
        things: &[
            thing(DOUBLE_BED, 0.0, 3.22, 0), thing(COUCH, -2.53, -1.6, 6), thing(DESK, 3.47, 4.97, 0),
            thing(DESK_END, 4.97, 4.97, 0), thing(DESK, 4.97, 3.47, 2), thing(TABLE, 1.5, -3.2, 0),
            thing(CHAIR, 1.5, -1.8, 0),
        ],
        spots: &[spot(5.47, 0.6, 2)],
    },
    // A single bed out from the stairs at the back, a couch by the right
    // wall, a counter along the back and a table toward the stairs.
    Room {
        openings: &[
            door(0, 2.0), window(SQUARE_WINDOW, 0, -2.0), window(SQUARE_WINDOW, 3, -2.6),
            upstairs(SQUARE_WINDOW, 0, -2.6), upstairs(SQUARE_WINDOW, 0, 2.6),
            top_floor(SQUARE_WINDOW, 0, 0.0), top_floor(SQUARE_WINDOW, 3, 2.6),
        ],
        things: &[
            thing(SINGLE_BED, -1.22, 3.9, 6), thing(COUCH, 4.53, 1.5, 2), thing(DESK, 2.47, 4.97, 0),
            thing(DESK, 4.47, 4.97, 0), thing(TABLE, -1.9, -1.2, 0), thing(CHAIR, -1.9, 0.2, 0),
        ],
        spots: &[spot(5.47, -1.6, 2)],
    },
];

/// The rooms of the Wide Roof House (since 2026-11-25): 8.8 m across inside
/// and 13.2 deep, its door in the front wall to one side of the middle. Its
/// owners arrive in its middle, or four metres in front of it, or there two
/// and a half metres to one side: the front of the room is left clear.
const WIDE_ROOMS: [Room; 3] = [
    // The bathhouse: the bathtub along the back wall, the sink and the toilet
    // down the right, the bed by the left wall, an armchair at the front right.
    Room {
        openings: &[
            door(0, -2.2), window(SQUARE_WINDOW, 0, 2.4), window(SQUARE_WINDOW, 1, -3.0),
            window(SQUARE_WINDOW, 3, 4.6),
        ],
        things: &[
            thing(DOUBLE_BED, -2.87, 1.5, 0), thing("armchair", 3.58, -1.8, 2), thing("bathtub", -0.115, 5.62, 0),
            thing("bathroom_sink", 3.93, 2.2, 2), thing("toilet", 3.99, 0.5, 2), thing("potted_plant", -3.6, 5.8, 0),
        ],
        spots: &[],
    },
    // The party room: the L shaped couch by the left wall, its end coming
    // forward, a big round table in the back left with a chair before it and
    // one behind, and the bed in the back right corner, a way to it between.
    Room {
        openings: &[
            door(0, 2.2), window(SQUARE_WINDOW, 0, -2.4), window(SQUARE_WINDOW, 1, 4.6),
            window(SQUARE_WINDOW, 3, -2.0), window(SQUARE_WINDOW, 2, 2.0),
        ],
        things: &[
            thing(DOUBLE_BED, 2.87, 4.32, 0), thing("couch", -3.28, -2.5, 6), thing("table_large_circular", -2.2, 3.6, 0),
            thing("chair", -2.2, 1.69, 4), thing("chair", -2.2, 5.51, 0),
        ],
        spots: &[],
    },
    // The study: the big bed in the back right, an armchair by the left wall
    // with a lamp on a table beside it, a desk down the right wall and a chair
    // at it, and a coat rack and an umbrella by the door.
    Room {
        openings: &[
            door(0, -2.2), window(SQUARE_WINDOW, 0, 2.4), window(SQUARE_WINDOW, 1, -2.5),
            window(SQUARE_WINDOW, 2, -2.0),
        ],
        things: &[
            thing("bed", 1.85, 4.07, 0), thing("armchair", -3.58, 1.0, 6), thing("end_table", -3.6, 2.75, 6),
            thing("desk", 3.61, -1.5, 2), thing("chair", 2.52, -0.7, 6), thing("coat_rack", -3.9, -5.9, 0),
        ],
        spots: &[],
    },
];

// -------------------------------------------------------------- the lawns

/// What stands out on the lawn, in the plot's own space
/// ([`crate::bag::Standing`]: -z out over the plot's front, where its sign
/// stands a metre and a half in from the front left corner). Each goes
/// with the plans that have it, whose houses stand clear of it.
const LAWNS: [&[Thing]; 25] = [
    // 0: nothing.
    &[],
    // 1: a lamp by the front right corner.
    &[thing(LAMP, 8.8, -9.8, 0)],
    // 2: a lamp beside the sign.
    &[thing(LAMP, -4.4, -10.0, 0)],
    // 3: a lamp by the left edge.
    &[thing(LAMP, -10.2, 3.0, 2)],
    // 4: a lamp by the right edge.
    &[thing(LAMP, 10.2, -3.0, 2)],
    // 5: a couch looking out over the front, on the right.
    &[thing(COUCH, 8.5, -8.6, 0)],
    // 6: a couch looking out over the front, on the left.
    &[thing(COUCH, -8.5, -5.5, 0)],
    // 7: a table and a chair in front on the right.
    &[thing(TABLE, 8.5, -8.0, 0), thing(CHAIR, 8.5, -6.55, 0)],
    // 8: a table and a chair on the left.
    &[thing(TABLE, -9.2, 2.0, 0), thing(CHAIR, -9.2, 3.45, 0)],
    // 9: a couch on the left, looking at the house.
    &[thing(COUCH, -10.0, 1.0, 6)],
    // 10: a couch on the right, looking at the house.
    &[thing(COUCH, 10.0, 1.0, 2)],
    // 11: a stand in front on the right.
    &[thing(DESK_END, 5.5, -9.0, 0), thing(DESK, 7.0, -9.0, 0), thing(DESK, 9.0, -9.0, 0)],
    // 12: a stand and a lamp.
    &[thing(DESK_END, 5.5, -9.0, 0), thing(DESK, 7.0, -9.0, 0), thing(DESK, 9.0, -9.0, 0), thing(LAMP, -4.4, -10.0, 0)],
    // 13: a stand on the left.
    &[thing(DESK, -9.0, 4.0, 6), thing(DESK, -9.0, 6.0, 6), thing(DESK_END, -9.0, 7.5, 6)],
    // 14: a couch at the back.
    &[thing(COUCH, 0.0, 10.4, 0)],
    // 15: a table and a chair at the back left.
    &[thing(TABLE, -8.5, 9.4, 0), thing(CHAIR, -8.5, 7.9, 4)],
    // 16: a lamp at the back right.
    &[thing(LAMP, 9.0, 10.2, 0)],
    // 17: a couch and a table at the back.
    &[thing(COUCH, -3.0, 10.4, 0), thing(TABLE, 2.5, 9.6, 0)],
    // 18: a couch in the front right corner.
    &[thing(COUCH, 8.6, -8.6, 3)],
    // 19 (since 2026-11-25): a garden in front on the right: a plant in a
    // pot and flowers.
    &[thing("potted_plant", 8.2, -8.8, 0), thing("flowers", 9.4, -8.0, 0)],
    // 20: a garden by the left edge.
    &[thing("potted_plant", -9.8, 1.0, 0), thing("flowers", -9.8, 2.4, 0)],
    // 21: a barbecue in front on the right.
    &[thing("barbecue", 8.4, -8.2, 0)],
    // 22: a mailbox beside the sign.
    &[thing("mailbox", -4.4, -10.2, 0)],
    // 23: a garden at the back left.
    &[thing("potted_plant", -8.5, 9.6, 0), thing("flowers", -8.5, 8.2, 0)],
    // 24: a garden by the right edge.
    &[thing("potted_plant", 10.1, -3.6, 0), thing("flowers", 10.1, -2.3, 0)],
];

// -------------------------------------------------------------- the tastes

/// A small thing a taste puts on top of something its room has: on the
/// `nth` of the room's tables or counters ([`TABLES`], [`COUNTERS`]), where
/// in that one's own space, and turned so many eighths from it.
#[derive(Clone, Copy)]
struct Top {
    on: &'static [&'static str],
    nth: usize,
    kind: &'static str,
    x: f32,
    z: f32,
    turn: u8,
}

const fn on_table(nth: usize, kind: &'static str, x: f32, z: f32, turn: u8) -> Top {
    Top { on: &TABLES, nth, kind, x, z, turn }
}

const fn on_counter(nth: usize, kind: &'static str, x: f32, z: f32, turn: u8) -> Top {
    Top { on: &COUNTERS, nth, kind, x, z, turn }
}

/// An owner's taste, which everything of theirs keeps to (Hajun,
/// 2026-11-25: "make each npc's plot have a coherent theme/personality for
/// each one so that its not just "diverse" randomly but its like the
/// owner's design taste is "diverse" so the each of the entire owner's
/// creation should be coherent with their personality"): what of their
/// room's own they have another thing of (each of a kind in turn, aligned
/// at its back and its middle; `None` to have nothing there), what small
/// things they keep on its tables and counters, what they add at the spots
/// the room leaves, and what stands on their roof terrace, if they have
/// one. Their colours are their plot's coat ([`COATS`]), and their lawn
/// their plan's.
struct Taste {
    swaps: &'static [(&'static str, &'static [Option<&'static str>])],
    tops: &'static [Top],
    extra: &'static [&'static str],
    roof: &'static [Thing],
}

/// The sixteen tastes, three owners each ([`PLANS`]).
const TASTES: [Taste; 16] = [
    // 0, the cook: a stove and a fridge where the counters were, a pan on the
    // stove, and a barbecue out on the lawn.
    Taste {
        swaps: &[(DESK, &[Some("stove"), Some("refrigerator")]), (CHAIR, &[Some("chair")])],
        tops: &[on_counter(0, "frying_pan", 0.0, -0.12, 0)],
        extra: &[],
        roof: &[],
    },
    // 1, the bookworm: an armchair for the couch, a reading lamp on a counter,
    // a cup of tea on the table, a grandfather's clock.
    Taste {
        swaps: &[(COUCH, &[Some("armchair")]), (CHAIR, &[Some("chair")])],
        tops: &[on_counter(0, "lamp_with_shade", 0.5, 0.1, 0), on_table(0, "cup_of_tea", -0.45, -0.3, 0)],
        extra: &["grandfathers_clock"],
        roof: &[],
    },
    // 2, the gamer: a computer, its screen and mouse on a counter, headphones
    // on the next.
    Taste {
        swaps: &[(CHAIR, &[Some("chair")])],
        tops: &[
            on_counter(0, "monitor", -0.2, 0.25, 0), on_counter(0, "computer", 0.45, 0.0, 0),
            on_counter(0, "computer_mouse", -0.45, -0.25, 0), on_counter(1, "headphones", 0.0, 0.0, 0),
        ],
        extra: &["trash_bin"],
        roof: &[],
    },
    // 3, the gardener: cupboards for the counters, flowers on the table, a
    // plant on the floor, and a garden out on the lawn.
    Taste {
        swaps: &[(DESK, &[Some("cabinet"), Some("cabinet")]), (CHAIR, &[Some("chair"), Some("chair")])],
        tops: &[on_table(0, "flowers", -0.35, -0.3, 0)],
        extra: &["potted_plant"],
        roof: &[],
    },
    // 4, the stargazer: a globe on the table, and a telescope up on the roof
    // terrace.
    Taste {
        swaps: &[(CHAIR, &[Some("chair")])],
        tops: &[on_table(0, "globe", 0.0, 0.1, 0)],
        extra: &[],
        roof: &[thing("telescope", 1.5, 1.0, 4)],
    },
    // 5, the one who never grew up: a stool, a doll and a present on the
    // table, a box of toys.
    Taste {
        swaps: &[(CHAIR, &[Some("stool"), Some("chair")])],
        tops: &[on_table(0, "doll", 0.35, 0.2, 0), on_table(0, "present", -0.4, -0.3, 0)],
        extra: &["empty_box"],
        roof: &[],
    },
    // 6, the tea drinker: a cupboard, tea on the table, and a rocking chair.
    Taste {
        swaps: &[(DESK, &[Some("cabinet")]), (CHAIR, &[Some("chair")])],
        tops: &[on_table(0, "cup_of_tea", 0.45, -0.3, 0)],
        extra: &["rocking_chair"],
        roof: &[],
    },
    // 7, the tinkerer: a stool, tools on the workbench, a bin and a box.
    Taste {
        swaps: &[(CHAIR, &[Some("stool"), Some("chair")])],
        tops: &[on_counter(0, "screwdriver", 0.3, 0.0, 2), on_counter(0, "stapler", -0.45, 0.1, 0)],
        extra: &["empty_box", "trash_bin"],
        roof: &[],
    },
    // 8, the one who loves a bath: in the bathhouse, a duck in the tub and
    // flowers; anywhere else an armchair, a bath at the front and a sink.
    Taste {
        swaps: &[(COUCH, &[Some("armchair")]), (DESK, &[Some("bathroom_sink")]), (CHAIR, &[Some("chair"), Some("chair")])],
        tops: &[on_table(0, "flowers", 0.0, 0.0, 0), on_counter(0, "tissue_box", 0.3, 0.2, 0)],
        extra: &["bathtub"],
        roof: &[],
    },
    // 9, the night owl, who reads late: an armchair for the couch, a lamp on a
    // counter, and a lamp out on the lawn.
    Taste {
        swaps: &[(COUCH, &[Some("armchair")]), (CHAIR, &[Some("chair")])],
        tops: &[on_counter(0, "lamp_with_shade", 0.4, 0.1, 0)],
        extra: &[],
        roof: &[],
    },
    // 10, the traveller: a globe, a coat rack and an umbrella by the wall, a
    // mailbox out front.
    Taste {
        swaps: &[(CHAIR, &[Some("chair")])],
        tops: &[on_table(0, "globe", 0.0, 0.0, 0)],
        extra: &["coat_rack", "closed_umbrella"],
        roof: &[],
    },
    // 11, the athlete: a treadmill, dumbbells on the floor, tea on the table.
    Taste {
        swaps: &[(CHAIR, &[Some("stool")])],
        tops: &[on_table(0, "cup_of_tea", 0.0, 0.0, 0)],
        extra: &["treadmill", "dumbbell"],
        roof: &[],
    },
    // 12, the party host: a glass and a burger on the table, a barbecue out on
    // the lawn, and up on the roof terrace a big table with a present on it.
    Taste {
        swaps: &[(CHAIR, &[Some("chair")])],
        tops: &[on_table(0, "wine_glass", 0.45, 0.35, 0), on_table(0, "hamburger", -0.4, -0.4, 0)],
        extra: &[],
        roof: &[thing("table_large_circular", 1.0, 0.5, 0), up("present", 1.0, FLOOR + 1.129, 0.0, 0)],
    },
    // 13, the minimalist: as little as will do, and nothing on it.
    Taste {
        swaps: &[(CHAIR, &[Some("chair")])],
        tops: &[],
        extra: &[],
        roof: &[],
    },
    // 14, the office worker: a computer, its screen and a mouse on one counter,
    // a printer and a stapler on the next, tissues on the table, and a bin.
    Taste {
        swaps: &[(CHAIR, &[Some("chair"), Some("chair")])],
        tops: &[
            on_counter(0, "monitor", -0.2, 0.25, 0), on_counter(0, "computer", 0.45, 0.0, 0),
            on_counter(0, "computer_mouse", -0.45, -0.25, 0), on_counter(1, "printer", 0.0, 0.0, 2),
            on_counter(1, "stapler", 0.6, -0.1, 0), on_table(0, "tissue_box", 0.0, 0.0, 0),
        ],
        extra: &["trash_bin"],
        roof: &[],
    },
    // 15, the collector: a cupboard, a globe on the table, a clock on the
    // cupboard, a grandfather's clock and a safe.
    Taste {
        swaps: &[(DESK, &[Some("cabinet")]), (CHAIR, &[Some("chair")])],
        tops: &[on_table(0, "globe", 0.2, 0.0, 0), on_counter(0, "alarm_clock", 0.3, 0.42, 0)],
        extra: &["grandfathers_clock", "safe"],
        roof: &[],
    },
];
const COOK: usize = 0;
const BOOKWORM: usize = 1;
const GAMER: usize = 2;
const GARDENER: usize = 3;
const STARGAZER: usize = 4;
const CHILD: usize = 5;
const TEA: usize = 6;
const TINKERER: usize = 7;
const BATHER: usize = 8;
const NIGHT_OWL: usize = 9;
const TRAVELLER: usize = 10;
const ATHLETE: usize = 11;
const HOST: usize = 12;
const MINIMALIST: usize = 13;
const OFFICE: usize = 14;
const COLLECTOR: usize = 15;

// -------------------------------------------------------------- the plans

/// One plot's house and what goes with it: which house; where on the
/// plot its middle stands, and how many eighths it is turned (turned by
/// nothing, its front wall looks out over the plot's front); which room
/// it has, of its kind's ([`ONE_FLOOR`], [`TWO_FLOORS`], [`WIDE_ROOMS`]),
/// whether the room is the other way round, left for right, and how many
/// of the room's last things it goes without; which door; whose taste it
/// is ([`TASTES`]); and which lawn.
struct Plan {
    house: usize,
    at: (f32, f32),
    turn: u8,
    flipped: bool,
    room: usize,
    less: usize,
    door: usize,
    taste: usize,
    lawn: usize,
}

#[allow(clippy::too_many_arguments)]
const fn plan(house: usize, at: (f32, f32), turn: u8, flipped: bool, room: usize, less: usize, door: usize, taste: usize, lawn: usize) -> Plan {
    Plan {
        house,
        at,
        turn,
        flipped,
        room,
        less,
        door,
        taste,
        lawn,
    }
}

impl Plan {
    /// The room it has.
    fn room(&self) -> &'static Room {
        match self.house {
            TALL | TERRACE | TALL3 | TERRACE3 => &TWO_FLOORS[self.room],
            WIDE => &WIDE_ROOMS[self.room],
            _ => &ONE_FLOOR[self.room],
        }
    }
}

/// Where a house stands: at the back of its plot, in a back corner, to
/// one side, in the middle, and toward the front on the right, where the
/// sign is not. No further from the middle of the plot than leaves the
/// middle on the house's floor, a metre in from its walls: whoever the
/// plot belongs to arrives there.
const BACK: (f32, f32) = (0.0, 4.5);
const BACK_RIGHT: (f32, f32) = (4.5, 4.5);
const BACK_LEFT: (f32, f32) = (-4.5, 4.5);
const RIGHT: (f32, f32) = (4.5, 0.0);
const LEFT: (f32, f32) = (-4.5, 0.0);
const MIDDLE: (f32, f32) = (0.0, 0.0);
const FRONT_RIGHT: (f32, f32) = (4.5, -3.0);
/// And the Wide Roof House, 10 by 15 m, whose long side goes front to back
/// unturned: further in from the plot's edges along it.
const WIDE_BACK: (f32, f32) = (0.0, 4.0);
const WIDE_BACK_RIGHT: (f32, f32) = (2.5, 4.0);
const WIDE_BACK_LEFT: (f32, f32) = (-2.5, 4.0);
const WIDE_RIGHT: (f32, f32) = (4.0, 0.0);
const WIDE_LEFT: (f32, f32) = (-4.0, 0.0);

/// Every plot's plan, in Hajun's order of the plots ([`crate::land::HOMES`]).
/// Two thirds of the houses are of one floor, the rest of two but six of
/// three; sixteen are as Hajun modelled them, white, where minimalists,
/// office workers, bathers, athletes and gamers live.
const PLANS: [Plan; PLOTS] = [
    // 1 to 12: round the pond.
    plan(SQUARE, BACK, 0, false, 0, 0, WINDOWED, COOK, 21),
    plan(FLAT, MIDDLE, 0, false, 2, 0, GLASS_DOUBLE, BOOKWORM, 0),
    plan(TALL, BACK_RIGHT, 0, false, 3, 0, CLASSIC, GAMER, 0),
    plan(SQUARE, LEFT, 6, true, 3, 0, GLASS, GARDENER, 24),
    plan(TERRACE, BACK, 0, false, 1, 0, DOUBLE, STARGAZER, 6),
    plan(SQUARE, BACK_LEFT, 0, false, 1, 1, CLASSIC, CHILD, 0),
    plan(FLAT, MIDDLE, 7, true, 4, 0, DOUBLE, TEA, 0),
    plan(TALL, MIDDLE, 2, false, 3, 0, GLASS_DOUBLE, GAMER, 0),
    plan(FLAT, BACK_RIGHT, 2, false, 0, 0, GLASS, TINKERER, 13),
    plan(WIDE, WIDE_BACK, 0, false, 0, 0, CLASSIC, BATHER, 0),
    plan(TERRACE, MIDDLE, 1, false, 2, 0, WINDOWED, STARGAZER, 18),
    plan(SQUARE, BACK_RIGHT, 0, false, 4, 0, GLASS_DOUBLE, NIGHT_OWL, 0),
    // 13 to 28: the ring behind them.
    plan(FLAT, LEFT, 6, true, 1, 0, CLASSIC, TRAVELLER, 22),
    plan(SQUARE, MIDDLE, 0, true, 0, 0, CLASSIC, ATHLETE, 0),
    plan(WIDE, MIDDLE, 0, false, 1, 0, WINDOWED, HOST, 21),
    plan(FLAT, BACK_LEFT, 6, false, 3, 3, WINDOWED, MINIMALIST, 0),
    plan(SQUARE, RIGHT, 2, true, 1, 2, DOUBLE, COOK, 0),
    plan(TALL3, BACK, 0, false, 1, 1, GLASS, OFFICE, 0),
    plan(FLAT, BACK_RIGHT, 0, true, 0, 0, WINDOWED, GARDENER, 20),
    plan(SQUARE, MIDDLE, 6, false, 3, 0, DOUBLE, COLLECTOR, 0),
    plan(SQUARE, BACK_LEFT, 0, true, 0, 0, GLASS, ATHLETE, 0),
    plan(FLAT, BACK, 0, true, 2, 1, CLASSIC, CHILD, 5),
    plan(WIDE, WIDE_BACK_RIGHT, 0, false, 2, 0, WINDOWED, TRAVELLER, 22),
    plan(TALL, LEFT, 6, false, 3, 1, DOUBLE, BOOKWORM, 17),
    plan(TERRACE3, BACK_RIGHT, 2, false, 3, 1, GLASS_DOUBLE, STARGAZER, 13),
    plan(WIDE, WIDE_BACK, 0, true, 2, 0, CLASSIC, TEA, 18),
    plan(WIDE, WIDE_RIGHT, 2, true, 0, 0, GLASS, BATHER, 23),
    plan(SQUARE, BACK_RIGHT, 2, true, 3, 1, CLASSIC, TINKERER, 6),
    // 29 to 48: the outer ring.
    plan(TALL, MIDDLE, 0, false, 2, 1, WINDOWED, NIGHT_OWL, 1),
    plan(FLAT, BACK_LEFT, 0, false, 0, 1, DOUBLE, COOK, 21),
    plan(SQUARE, MIDDLE, 2, false, 1, 2, GLASS, OFFICE, 0),
    plan(WIDE, WIDE_LEFT, 6, false, 1, 0, CLASSIC, GARDENER, 19),
    plan(FLAT, BACK, 0, false, 3, 3, GLASS_DOUBLE, MINIMALIST, 0),
    plan(SQUARE, BACK_RIGHT, 0, true, 1, 2, DOUBLE, COLLECTOR, 5),
    plan(TALL3, MIDDLE, 6, false, 1, 1, CLASSIC, BOOKWORM, 0),
    plan(FLAT, RIGHT, 2, false, 2, 2, WINDOWED, BATHER, 14),
    plan(WIDE, MIDDLE, 2, true, 1, 0, CLASSIC, CHILD, 0),
    plan(TERRACE, BACK, 2, false, 2, 1, GLASS, HOST, 21),
    plan(TERRACE, LEFT, 6, false, 3, 1, GLASS_DOUBLE, COLLECTOR, 0),
    plan(WIDE, WIDE_BACK_LEFT, 0, true, 2, 0, WINDOWED, TRAVELLER, 22),
    plan(SQUARE, BACK, 0, true, 3, 3, GLASS, MINIMALIST, 0),
    plan(TALL, RIGHT, 2, false, 0, 1, WINDOWED, NIGHT_OWL, 1),
    plan(FLAT, BACK_RIGHT, 0, false, 1, 2, CLASSIC, TINKERER, 9),
    plan(SQUARE, FRONT_RIGHT, 4, false, 4, 2, WINDOWED, OFFICE, 0),
    plan(FLAT, (-2.0, 2.0), 7, false, 3, 0, GLASS_DOUBLE, TEA, 10),
    plan(SQUARE, BACK_LEFT, 6, false, 4, 1, DOUBLE, ATHLETE, 0),
    plan(TERRACE3, MIDDLE, 6, false, 1, 1, GLASS, HOST, 0),
    plan(TALL, MIDDLE, 1, false, 1, 1, DOUBLE, GAMER, 0),
];

// ------------------------------------------------------------ the colours

/// A colour to paint a part, red, green and blue as a screen has them, or
/// `None` to leave it as its model has it.
type Tint = Option<[u8; 3]>;

/// As the model has it.
const OWN: Tint = None;
/// Soft ones, for walls.
const CREAM: Tint = Some([244, 227, 193]);
const BUTTER: Tint = Some([246, 231, 168]);
const PEACH: Tint = Some([246, 205, 176]);
const BLUSH: Tint = Some([242, 198, 201]);
const ROSE: Tint = Some([233, 180, 184]);
const SAGE: Tint = Some([200, 217, 191]);
const MINT: Tint = Some([201, 230, 216]);
const SEAFOAM: Tint = Some([183, 220, 211]);
const SKY: Tint = Some([198, 223, 240]);
const POWDER: Tint = Some([185, 207, 230]);
const LAVENDER: Tint = Some([217, 205, 234]);
const LILAC: Tint = Some([203, 190, 224]);
const SAND: Tint = Some([230, 211, 179]);
const APRICOT: Tint = Some([248, 212, 154]);
const CORAL: Tint = Some([243, 181, 165]);
const PISTACHIO: Tint = Some([219, 232, 185]);
const DOVE: Tint = Some([214, 217, 220]);
const SHELL: Tint = Some([238, 222, 210]);
/// Deeper ones, for roofs, and for a door or a blanket.
const TERRACOTTA: Tint = Some([181, 87, 63]);
const BRICK: Tint = Some([157, 74, 60]);
const CHARCOAL: Tint = Some([74, 79, 87]);
const FOREST: Tint = Some([95, 125, 93]);
const BROWN: Tint = Some([122, 90, 69]);
const TEAL: Tint = Some([63, 124, 122]);
const PLUM: Tint = Some([122, 86, 115]);
const NAVY: Tint = Some([61, 79, 115]);
/// Wood and stone, for floors.
const OAK: Tint = Some([201, 165, 122]);
const WALNUT: Tint = Some([160, 118, 84]);
const BIRCH: Tint = Some([216, 195, 160]);
const STONE: Tint = Some([190, 196, 200]);
/// And the rest, for doors, blankets, seats and lamps.
const RED: Tint = Some([179, 64, 46]);
const GREEN: Tint = Some([47, 93, 80]);
const YELLOW: Tint = Some([242, 200, 75]);
const WHITE: Tint = Some([245, 243, 238]);
const ORANGE: Tint = Some([217, 121, 60]);
const BLACK: Tint = Some([43, 43, 43]);
const AQUA: Tint = Some([42, 157, 143]);
const HONEY: Tint = Some([233, 196, 106]);
const EMBER: Tint = Some([231, 111, 81]);
const MOSS: Tint = Some([132, 169, 140]);
const PINK: Tint = Some([217, 134, 155]);
const AZURE: Tint = Some([127, 183, 230]);
const PINE: Tint = Some([47, 79, 63]);
const INK: Tint = Some([44, 62, 87]);

/// One plot's colours: its house's walls, roof and floors (a flat top is
/// a floor); its door, and the frames of its door and windows; its bed's
/// blanket, its chairs' seats, and its lamp's post. Each is painted on
/// whatever of the plot has such a part ([`parts`]).
struct Coat {
    wall: Tint,
    roof: Tint,
    floor: Tint,
    door: Tint,
    trim: Tint,
    blanket: Tint,
    seat: Tint,
    lamp: Tint,
}

#[allow(clippy::too_many_arguments)]
const fn coat(wall: Tint, roof: Tint, floor: Tint, door: Tint, trim: Tint, blanket: Tint, seat: Tint, lamp: Tint) -> Coat {
    Coat {
        wall,
        roof,
        floor,
        door,
        trim,
        blanket,
        seat,
        lamp,
    }
}

/// Every plot's colours, in the order of the plots: walls, roof, floors,
/// door, frames, blanket, seats, lamp. Each owner's in their taste's way
/// (the cook's warm, the gardener's green, the stargazer's night blue, the
/// tea drinker's rose), and no two plots near each other with walls of one
/// colour; a third of the houses as Hajun modelled them, white under a
/// grey roof.
const COATS: [Coat; PLOTS] = [
    // 1 to 12: round the pond.
    coat(BUTTER, TERRACOTTA, OAK, RED, OWN, EMBER, RED, OWN),
    coat(CREAM, BROWN, WALNUT, GREEN, BROWN, PLUM, OWN, PINE),
    coat(OWN, OWN, OWN, BLACK, OWN, INK, OWN, OWN),
    coat(SAGE, FOREST, OAK, GREEN, OWN, MOSS, GREEN, PINE),
    coat(OWN, OWN, OWN, INK, OWN, NAVY, INK, BLACK),
    coat(BLUSH, TERRACOTTA, BIRCH, YELLOW, OWN, PINK, AQUA, OWN),
    coat(ROSE, BROWN, WALNUT, PLUM, OWN, PINK, PLUM, OWN),
    coat(OWN, OWN, OWN, BLACK, CHARCOAL, AZURE, OWN, OWN),
    coat(SAND, CHARCOAL, STONE, ORANGE, CHARCOAL, NAVY, ORANGE, BLACK),
    coat(OWN, OWN, OWN, TEAL, OWN, AQUA, OWN, WHITE),
    coat(LAVENDER, OWN, BIRCH, INK, OWN, NAVY, INK, OWN),
    coat(POWDER, NAVY, STONE, BLACK, CHARCOAL, INK, NAVY, BLACK),
    // 13 to 28: the ring behind them.
    coat(APRICOT, OWN, OAK, TEAL, OWN, TEAL, TEAL, OWN),
    coat(OWN, OWN, OWN, RED, OWN, AZURE, OWN, PINE),
    coat(CORAL, TERRACOTTA, OAK, YELLOW, OWN, EMBER, YELLOW, OWN),
    coat(OWN, OWN, OWN, BLACK, OWN, WHITE, WHITE, OWN),
    coat(PEACH, TERRACOTTA, OAK, RED, OWN, HONEY, RED, OWN),
    coat(OWN, OWN, OWN, NAVY, CHARCOAL, NAVY, OWN, BLACK),
    coat(MINT, OWN, OAK, GREEN, OWN, MOSS, OWN, OWN),
    coat(LILAC, PLUM, WALNUT, PLUM, BROWN, PLUM, PLUM, INK),
    coat(OWN, OWN, OWN, YELLOW, OWN, AZURE, OWN, OWN),
    coat(SKY, OWN, BIRCH, YELLOW, OWN, PINK, PINK, OWN),
    coat(SAND, TERRACOTTA, OAK, TEAL, OWN, TEAL, TEAL, OWN),
    coat(SHELL, BROWN, WALNUT, GREEN, BROWN, PLUM, OWN, OWN),
    coat(LILAC, OWN, OWN, INK, OWN, NAVY, OWN, OWN),
    coat(SHELL, BRICK, WALNUT, PLUM, OWN, PINK, PLUM, OWN),
    coat(OWN, OWN, OWN, TEAL, OWN, AQUA, OWN, OWN),
    coat(DOVE, BRICK, STONE, ORANGE, CHARCOAL, NAVY, OWN, OWN),
    // 29 to 48: the outer ring.
    coat(DOVE, NAVY, STONE, BLACK, CHARCOAL, INK, OWN, BLACK),
    coat(APRICOT, OWN, OAK, RED, OWN, EMBER, RED, OWN),
    coat(OWN, OWN, OWN, CHARCOAL, OWN, NAVY, CHARCOAL, OWN),
    coat(PISTACHIO, FOREST, OAK, GREEN, OWN, MOSS, OWN, OWN),
    coat(OWN, OWN, OWN, WHITE, OWN, WHITE, OWN, OWN),
    coat(ROSE, PLUM, WALNUT, PLUM, OWN, PLUM, PLUM, OWN),
    coat(CREAM, BROWN, WALNUT, GREEN, BROWN, PLUM, OWN, PINE),
    coat(OWN, OWN, OWN, TEAL, OWN, AQUA, OWN, OWN),
    coat(SKY, OWN, BIRCH, YELLOW, OWN, PINK, OWN, OWN),
    coat(CORAL, OWN, OAK, YELLOW, OWN, EMBER, YELLOW, OWN),
    coat(OWN, OWN, OWN, PLUM, BROWN, PLUM, OWN, OWN),
    coat(SAND, TERRACOTTA, OAK, TEAL, OWN, TEAL, OWN, OWN),
    coat(OWN, OWN, OWN, BLACK, OWN, WHITE, OWN, OWN),
    coat(SEAFOAM, NAVY, STONE, BLACK, CHARCOAL, INK, NAVY, BLACK),
    coat(SAND, CHARCOAL, STONE, ORANGE, CHARCOAL, NAVY, ORANGE, OWN),
    coat(OWN, OWN, OWN, INK, OWN, NAVY, OWN, OWN),
    coat(PEACH, BROWN, WALNUT, PLUM, OWN, PINK, PLUM, OWN),
    coat(OWN, OWN, OWN, RED, OWN, AZURE, OWN, OWN),
    coat(BLUSH, OWN, BIRCH, YELLOW, OWN, EMBER, YELLOW, OWN),
    coat(POWDER, CHARCOAL, STONE, BLACK, CHARCOAL, INK, OWN, OWN),
];

/// The parts of a thing of `kind` that `coat` has a colour for, each by
/// the name its material has in the thing's model, which is what a part
/// is painted by ([`Paint::part`]). The things that came for the plots on
/// 2026-11-25 are each one material on a picture of many colours, and are
/// left as they are.
fn parts(kind: &str, coat: &Coat) -> Vec<(&'static str, Tint)> {
    let roofed = [SQUARE, TALL, WIDE, TALL3].map(|house| HOUSES[house]).contains(&kind);
    match kind {
        _ if kind == HOUSES[TERRACE3] => vec![("plain_wall", coat.wall), ("plain_wall.001", coat.wall), ("plain_floor", coat.floor)],
        _ if HOUSES.contains(&kind) => vec![
            ("plain_wall", coat.wall),
            ("plain_roof", if roofed { coat.roof } else { OWN }),
            ("plain_floor", coat.floor),
        ],
        _ if DOORS.contains(&kind) => vec![("door", coat.door), ("doorframe", coat.trim)],
        SQUARE_WINDOW | WIDE_WINDOW => vec![("window_frame", coat.trim)],
        SINGLE_BED | DOUBLE_BED => vec![("blanket", coat.blanket)],
        CHAIR => vec![("chair", coat.seat)],
        LAMP => vec![("lampostbody", coat.lamp)],
        _ => Vec::new(),
    }
}

// ------------------------------------------------------- putting together

/// What of its room a plan has, in the room's own space, before it is
/// turned the other way round: the room's things but the last few it goes
/// without, each its taste's thing in its place ([`Taste::swaps`]); what
/// its taste adds at the room's spots; and the small things its taste
/// keeps on the room's tables and counters.
fn furnished(plan: &Plan) -> Vec<Thing> {
    let (room, taste) = (plan.room(), &TASTES[plan.taste]);
    let kept = room.things.len().saturating_sub(plan.less);
    let mut seen: Vec<(&str, usize)> = Vec::new();
    let mut things: Vec<Thing> = Vec::new();
    for &old in &room.things[..kept] {
        let nth = match seen.iter_mut().find(|(kind, _)| *kind == old.kind) {
            Some((_, count)) => {
                *count += 1;
                *count - 1
            }
            None => {
                seen.push((old.kind, 1));
                0
            }
        };
        let swapped = taste
            .swaps
            .iter()
            .find(|(from, _)| *from == old.kind)
            .and_then(|(_, to)| to.get(nth).copied());
        match swapped {
            None => things.push(old),
            Some(None) => {}
            Some(Some(kind)) => {
                // Its back where the old one's was, against its wall, its
                // middle across the old one's middle; a chair's front, at its
                // table.
                let ((was_x, was_back), (now_x, now_back)) =
                    if old.kind == CHAIR { (front_of(old.kind), front_of(kind)) } else { (back_of(old.kind), back_of(kind)) };
                let (right, back) = turned(old.turn, was_x - now_x, was_back - now_back);
                things.push(Thing {
                    kind,
                    x: old.x + right,
                    z: old.z + back,
                    ..old
                });
            }
        }
    }
    for (kind, spot) in taste.extra.iter().zip(room.spots) {
        let (middle, back) = back_of(kind);
        let (right, behind) = turned(spot.turn, middle, back);
        things.push(thing(kind, spot.x - right, spot.z - behind, spot.turn));
    }
    let mut tops: Vec<Thing> = Vec::new();
    for top in taste.tops {
        let Some(host) = things.iter().filter(|host| top.on.contains(&host.kind)).nth(top.nth) else {
            continue;
        };
        let Some(high) = top_of(host.kind) else {
            continue;
        };
        let (right, back) = turned(host.turn, top.x, top.z);
        tops.push(up(top.kind, host.x + right, FLOOR + high, host.z + back, host.turn + top.turn));
    }
    things.extend(tops);
    things
}

/// The made-up lot of `plot`, put together from its plan and its coat:
/// the house under the number 1, then what is in its walls, what stands
/// on its floor and roof and what stands on the lawn, each under the next
/// number.
fn made(plot: usize) -> Lot {
    let (name, look) = owner(plot);
    let (plan, coat) = (&PLANS[plot], &COATS[plot]);
    let room = plan.room();
    let (x, z) = plan.at;
    let mut stands: Vec<Standing> = Vec::new();
    let mut stand = |kind: &str, at: At| {
        let id = stands.len() as u32 + 1;
        stands.push(Standing {
            id,
            kind: kind.into(),
            at,
        });
    };
    stand(HOUSES[plan.house], ground(x, 0.0, z, plan.turn));
    // What is in its walls, each in the wall it is in once the room is
    // the other way round, and only on a storey the house has.
    for opening in room.openings.iter().filter(|opening| opening.storey < STOREYS[plan.house]) {
        let (side, along) = match (plan.flipped, opening.side) {
            (true, 1) => (3, opening.along),
            (true, 3) => (1, opening.along),
            (true, side) => (side, -opening.along),
            (false, side) => (side, opening.along),
        };
        // A door stands on the floor of its storey, a window at its sill.
        let (kind, up) = match opening.kind {
            None => (DOORS[plan.door], FLOOR + STOREY * f32::from(opening.storey)),
            Some(window) => (window, SILLS[plan.house][usize::from(opening.storey)]),
        };
        stand(kind, At::Wall {
            wall: wall_of(plan.house, side, opening.storey),
            along,
            up,
            face: 1,
        });
    }
    // What stands on its floor and on its roof, where the house's place and
    // turn bring each thing's place in the room.
    let roof = ROOF_TOP[plan.house].map_or(&[][..], |_| TASTES[plan.taste].roof);
    let things = furnished(plan).into_iter().map(|thing| (thing, FLOOR)).chain(roof.iter().map(|&thing| (thing, ROOF_TOP[plan.house].unwrap_or(FLOOR))));
    for (thing, floor) in things {
        // The other way round, left for right: its middle where the
        // middle of the other was, mirrored, facing the mirrored way.
        let (across, along, turn) = if plan.flipped {
            let turn = (8 - thing.turn % 8) % 8;
            let (middle_x, middle_z) = middle_of(thing.kind);
            let (was_x, was_z) = turned(thing.turn, middle_x, middle_z);
            let (now_x, now_z) = turned(turn, middle_x, middle_z);
            (-(thing.x + was_x) - now_x, thing.z + was_z - now_z, turn)
        } else {
            (thing.x, thing.z, thing.turn)
        };
        let (right, back) = turned(plan.turn, across, along);
        let high = if thing.y > 0.0 { thing.y - FLOOR + floor } else { floor };
        stand(thing.kind, ground(x + right, high, z + back, plan.turn + turn));
    }
    for thing in LAWNS[plan.lawn] {
        stand(thing.kind, ground(thing.x, 0.0, thing.z, thing.turn));
    }
    let paints = stands
        .iter()
        .flat_map(|stood| {
            parts(&stood.kind, coat).into_iter().filter_map(|(part, tint)| {
                Some(Paint {
                    id: stood.id,
                    part: part.into(),
                    colour: tint?,
                })
            })
        })
        .collect();
    Lot {
        who: PREMADE + plot as u64,
        name: name.into(),
        look: look.into(),
        stands,
        paints,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a bed is, and what a seat.
    const BEDS: [&str; 3] = [SINGLE_BED, DOUBLE_BED, "bed"];
    const SEATS: [&str; 7] = [COUCH, CHAIR, "chair", "armchair", "couch", "rocking_chair", "stool"];

    #[test]
    fn every_plot_has_a_made_up_lot_of_its_own() {
        let lots: Vec<Lot> = (0..PLOTS).map(lot).collect();
        for (plot, made) in lots.iter().enumerate() {
            assert_eq!(made.who, PREMADE + plot as u64);
            assert!(made.premade() && !made.stands.is_empty());
            assert!(crate::profile::name(&made.name).is_some() && crate::profile::look(&made.look));
            // As a bag would keep it: everything a thing there is, each
            // under a number of its own, where a thing may stand.
            let mut ids: Vec<u32> = made.stands.iter().map(|stood| stood.id).collect();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), made.stands.len(), "plot {plot}");
            for stood in &made.stands {
                assert!(crate::bag::thing(&stood.kind).is_some_and(|thing| thing.stands()), "plot {plot}: {}", stood.kind);
                assert!(crate::bag::sound(&stood.at), "plot {plot}: {:?}", stood.at);
            }
            assert!(made.stands.len() <= crate::bag::MOST_STANDING);
            for paint in &made.paints {
                assert!(made.stands.iter().any(|stood| stood.id == paint.id) && crate::bag::part_named(&paint.part));
            }
        }
        let mut names: Vec<&str> = lots.iter().map(|made| made.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), PLOTS);
        assert_eq!(lot(PLOTS + 3), lot(3));
    }

    #[test]
    fn every_made_up_lot_is_a_house_with_a_door_a_bed_and_a_seat() {
        let lots: Vec<Lot> = (0..PLOTS).map(lot).collect();
        let count = |made: &Lot, kinds: &[&str]| made.stands.iter().filter(|stood| kinds.contains(&stood.kind.as_str())).count();
        for (plot, made) in lots.iter().enumerate() {
            // Numbered from 1 up, the house first, and no more of them
            // than a plot is told of easily.
            let ids: Vec<u32> = made.stands.iter().map(|stood| stood.id).collect();
            assert_eq!(ids, (1..=made.stands.len() as u32).collect::<Vec<_>>(), "plot {plot}");
            assert!(made.stands.len() <= 30, "plot {plot}: {} things", made.stands.len());
            assert_eq!(count(made, &HOUSES), 1, "plot {plot}");
            assert!(HOUSES.contains(&made.stands[0].kind.as_str()), "plot {plot}");
            assert!(count(made, &DOORS) >= 1, "plot {plot}");
            assert!((1..=8).contains(&count(made, &[SQUARE_WINDOW, WIDE_WINDOW])), "plot {plot}");
            assert_eq!(count(made, &BEDS), 1, "plot {plot}");
            assert!(count(made, &SEATS) >= 1, "plot {plot}");
            // What goes in a wall is in one of the house's, and everything
            // else stands on the ground.
            for stood in &made.stands {
                let in_a_wall = DOORS.contains(&stood.kind.as_str()) || [SQUARE_WINDOW, WIDE_WINDOW].contains(&stood.kind.as_str());
                match stood.at {
                    At::Wall { wall, face, .. } => assert!(in_a_wall && wall < 12 && face == 1, "plot {plot}: {stood:?}"),
                    At::Ground { .. } => assert!(!in_a_wall, "plot {plot}: {stood:?}"),
                }
            }
            // One colour to a part.
            for (n, paint) in made.paints.iter().enumerate() {
                assert!(made.paints[..n].iter().all(|other| (other.id, &other.part) != (paint.id, &paint.part)), "plot {plot}: {paint:?}");
            }
            assert!(made.paints.len() <= crate::bag::MOST_PAINTED);
        }
        // No two the same, whoever they belong to, and each unlike the
        // one numbered before it: another house, or the house somewhere
        // else or turned another way.
        for (plot, made) in lots.iter().enumerate() {
            assert!(lots[..plot].iter().all(|other| other.stands != made.stands), "plot {plot} is another's over again");
            if let Some(before) = plot.checked_sub(1).map(|before| &lots[before]) {
                assert_ne!(before.stands[0], made.stands[0], "plots {} and {plot} have one house in one place", plot - 1);
            }
        }
        // Every house, every door and both windows are somewhere, and a
        // lamp, the dearest thing to draw, on no more than twenty plots.
        let somewhere = |kind: &str| lots.iter().any(|made| count(made, &[kind]) > 0);
        assert!(HOUSES.iter().chain(&DOORS).chain(&[SQUARE_WINDOW, WIDE_WINDOW]).all(|kind| somewhere(kind)));
        let lit = lots.iter().filter(|made| count(made, &[LAMP]) > 0).count();
        assert!((1..=20).contains(&lit), "{lit} plots with a lamp");
        // About a third of the houses are as their models have them.
        let painted = |made: &Lot| made.paints.iter().any(|paint| paint.id == 1);
        let plain = lots.iter().filter(|made| !painted(made)).count();
        assert!((12..=20).contains(&plain), "{plain} houses left as they are");
    }

    #[test]
    fn most_houses_are_low_and_none_higher_than_three_floors() {
        // Hajun: "keep it not to rich so its not like 10 floor buildings but
        // pretty diverse ... floor height should be biased towards lower
        // heights for the npc plots".
        let floors = |house: usize| match house {
            TALL | TERRACE => 2,
            TALL3 | TERRACE3 => 3,
            _ => 1,
        };
        let one = PLANS.iter().filter(|plan| floors(plan.house) == 1).count();
        let three = PLANS.iter().filter(|plan| floors(plan.house) == 3).count();
        assert!(one * 3 >= PLOTS * 2 && three <= 6, "{one} of one floor, {three} of three");
        for house in 0..HOUSES.len() {
            assert!(PLANS.iter().any(|plan| plan.house == house), "nobody has a {}", HOUSES[house]);
        }
    }

    #[test]
    fn every_taste_is_three_owners_and_no_two_neighbours_share_one() {
        for taste in 0..TASTES.len() {
            assert_eq!(PLANS.iter().filter(|plan| plan.taste == taste).count(), 3, "taste {taste}");
        }
        for pair in PLANS.windows(2) {
            assert_ne!(pair[0].taste, pair[1].taste);
        }
        // What a taste swaps in or adds, and what it keeps on tops, is
        // somewhere on the lot of at least one of its owners.
        let lots: Vec<Lot> = (0..PLOTS).map(lot).collect();
        for (taste, of) in TASTES.iter().enumerate() {
            let theirs: Vec<&Lot> = PLANS.iter().zip(&lots).filter(|(plan, _)| plan.taste == taste).map(|(_, made)| made).collect();
            let has = |kind: &str| theirs.iter().any(|made| made.stands.iter().any(|stood| stood.kind == kind));
            for kind in of.extra.iter().chain(of.tops.iter().map(|top| &top.kind)) {
                assert!(has(kind), "taste {taste}: none of its owners has a {kind}");
            }
        }
    }

    #[test]
    fn no_two_plots_near_each_other_have_walls_of_one_colour() {
        use bevy::math::Vec2;
        // The next plot along a street is 35 m off, and the one across a
        // street 50.
        const NEAR: f32 = 52.0;
        let walls: Vec<Tint> = (0..PLOTS)
            .map(|plot| lot(plot).paints.iter().find(|paint| paint.id == 1 && paint.part == "plain_wall").map(|paint| paint.colour))
            .collect();
        let homes = crate::land::HOMES.map(Vec2::from);
        for (plot, home) in homes.iter().enumerate() {
            for (other, there) in homes.iter().enumerate().skip(plot + 1) {
                assert!(home.distance(*there) > NEAR || walls[plot] != walls[other], "plots {plot} and {other}: {:?}", walls[plot]);
            }
        }
        // And a roof is painted only on a house that has one.
        for (plot, plan) in PLANS.iter().enumerate() {
            let roofed = [SQUARE, TALL, WIDE, TALL3].contains(&plan.house);
            let roof = lot(plot).paints.iter().any(|paint| paint.part == "plain_roof");
            assert_eq!(roof, roofed && COATS[plot].roof.is_some(), "plot {plot}");
        }
    }

    #[test]
    fn no_door_of_a_square_house_stands_under_its_ridge() {
        // The widest door, a double one, is 3.8 m across; and no door
        // comes nearer the line under the ridge than half a metre.
        const HALF_A_DOOR: f32 = 1.9;
        const RIDGE: f32 = 0.5;
        for (plot, plan) in PLANS.iter().enumerate().filter(|(_, plan)| plan.house == SQUARE) {
            for opening in plan.room().openings.iter().filter(|opening| opening.kind.is_none() && opening.side == 0) {
                assert!(opening.along.abs() - HALF_A_DOOR >= RIDGE, "plot {plot}: its door is {} m along the front wall", opening.along);
            }
        }
        // Every room is one its plan's house has, and goes without no more
        // than leaves it its bed and its couch.
        for (plot, plan) in PLANS.iter().enumerate() {
            let rooms = match plan.house {
                TALL | TERRACE | TALL3 | TERRACE3 => TWO_FLOORS.len(),
                WIDE => WIDE_ROOMS.len(),
                _ => ONE_FLOOR.len(),
            };
            assert!(plan.room < rooms && plan.door < DOORS.len() && plan.lawn < LAWNS.len() && plan.turn < 8, "plot {plot}");
            assert!(plan.room().things.len() - plan.less >= 2, "plot {plot}");
            assert!(plan.taste < TASTES.len(), "plot {plot}");
        }
    }

    #[test]
    fn the_walls_of_a_house_of_three_floors_are_numbered_storey_by_storey() {
        // As the game numbers them: the front's three storeys, then the
        // left's, the back's and the right's.
        assert_eq!(wall_of(TALL3, 0, 0), 0);
        assert_eq!(wall_of(TALL3, 0, 2), 2);
        assert_eq!(wall_of(TALL3, 1, 0), 3);
        assert_eq!(wall_of(TALL3, 3, 1), 10);
        assert_eq!(wall_of(TERRACE3, 2, 1), 5);
        // And one a side, all the way up, in the rest.
        assert_eq!(wall_of(TALL, 3, 1), 3);
        assert_eq!(wall_of(SQUARE, 2, 0), 2);
    }

    #[test]
    fn a_thing_turned_stands_where_the_game_turns_it_to() {
        use bevy::math::{Quat, Vec3};
        // The written sines and cosines are those of the written angles,
        // and a quarter turn moves a point to exactly where it should.
        for (turn, &(yaw, cos, sin)) in EIGHTHS.iter().enumerate() {
            assert!((yaw.cos() - cos).abs() < 1e-6 && (yaw.sin() - sin).abs() < 1e-6, "{turn}");
            let (x, z) = turned(turn as u8, 3.0, -2.0);
            let there = Quat::from_rotation_y(yaw) * Vec3::new(3.0, 0.0, -2.0);
            assert!((there.x - x).abs() < 1e-5 && (there.z - z).abs() < 1e-5, "{turn}: {x} {z} for {there}");
        }
        assert_eq!(turned(2, 3.0, -2.0), (-2.0, -3.0));
        assert_eq!(turned(4, 3.0, -2.0), (-3.0, 2.0));
        assert_eq!(turned(6, 3.0, -2.0), (2.0, 3.0));
        // Turned past a whole turn, a thing is turned by what is left.
        assert_eq!(ground(0.0, 0.0, 0.0, 9), ground(0.0, 0.0, 0.0, 1));
    }
}
