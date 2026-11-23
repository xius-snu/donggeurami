//! The made-up land: what fills a plot of the town when no real person's
//! land does (Hajun, 2026-11-23: "then, if there are still unfilled plots,
//! fill with fake plots that are premade"). One to each of the town's 48
//! plots, the same on every device and on the server, so that a plot
//! filled offline is filled the same online. Each has an owner of its own,
//! by a name of the computer's ([`crate::BOY_NAMES`], [`crate::GIRL_NAMES`])
//! and in a look of that side, whom someone of the computer's walks as on
//! the first sixteen plots ([`crate::land::Holder::Minded`]).
//!
//! **Each is a furnished plot**, as someone might have left theirs: one of
//! Hajun's four houses, standing somewhere on the plot and turned some way
//! ([`Plan`]), with a door and windows in its walls and a bed, a couch and
//! more on its ground floor ([`Room`]), on some a lamp or somewhere to sit
//! out on the lawn ([`LAWNS`]), and its own colours ([`Coat`]). No two are
//! the same, and the sixteen someone walks have the most in them.
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

/// The made-up lot of the plot numbered `plot` (of [`PLOTS`], round and
/// round past them). All of them are put together the first time one is
/// asked for, and every one handed out is a copy.
pub fn lot(plot: usize) -> Lot {
    static LOTS: OnceLock<Vec<Lot>> = OnceLock::new();
    LOTS.get_or_init(|| (0..PLOTS).map(made).collect())[plot % PLOTS].clone()
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

/// Hajun's four houses, by the names the game's pieces go by: the Square
/// House, the Flat House, the Two Floor House and the Roof Terrace House.
const HOUSES: [&str; 4] = [
    "square_house",
    "square_house_roofless",
    "square_house_two_floor",
    "square_house_roofless_two_floor",
];
const SQUARE: usize = 0;
const FLAT: usize = 1;
const TALL: usize = 2;
const TERRACE: usize = 3;

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

/// Their two windows, and what stands on a floor.
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
/// stands in the house stands on, and how much higher the floor over
/// that is, in metres.
const FLOOR: f32 = 0.1;
const STOREY: f32 = 5.0;
/// How many storeys of each house a door or a window goes in: the walls
/// of the Flat House and of the Roof Terrace House end at their tops.
const STOREYS: [u8; 4] = [1, 1, 2, 1];
/// How high over each house's foot the sill of a window on its ground
/// floor is, and of one on the floor over that, in the Two Floor House:
/// 0.22 of the way up its storey, where the game stands every window
/// (`build::WINDOW_SILL`). The Square House's one storey is 5 m of wall;
/// the others' ground storeys end under a floor, 4.9 m.
const SILLS: [f32; 4] = [1.2, 1.178, 1.178, 1.178];
const UPPER_SILL: f32 = 6.2;

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

/// Standing on the ground at (`x`, `y`, `z`), turned `turn` eighths.
fn ground(x: f32, y: f32, z: f32, turn: u8) -> At {
    let (yaw, ..) = EIGHTHS[usize::from(turn % 8)];
    At::Ground { x, y, z, yaw }
}

/// Something that stands on a floor or on the lawn: what it is, where
/// its own origin is, and how many eighths it is turned. Turned by
/// nothing, its front looks toward -z.
#[derive(Clone, Copy)]
struct Thing {
    kind: &'static str,
    x: f32,
    z: f32,
    turn: u8,
}

const fn thing(kind: &'static str, x: f32, z: f32, turn: u8) -> Thing {
    Thing { kind, x, z, turn }
}

/// A door or a window in a wall of a house: what it is, or `None` for the
/// door its plan has ([`Plan::door`]); which wall, as the game numbers
/// them (0 the front, 1 the one at x = -5.5, 2 the back and 3 the one at
/// x = 5.5); how far along the wall its middle is, x along the front and
/// the back and z along the other two; and which storey it is in.
#[derive(Clone, Copy)]
struct Opening {
    kind: Option<&'static str>,
    wall: u8,
    along: f32,
    storey: u8,
}

/// The door of the house, whichever its plan has.
const fn door(wall: u8, along: f32) -> Opening {
    Opening {
        kind: None,
        wall,
        along,
        storey: 0,
    }
}

/// A window on the ground floor.
const fn window(kind: &'static str, wall: u8, along: f32) -> Opening {
    Opening {
        kind: Some(kind),
        wall,
        along,
        storey: 0,
    }
}

/// A window on the floor over it, in a house whose walls go up that far:
/// the Two Floor House alone.
const fn upstairs(kind: &'static str, wall: u8, along: f32) -> Opening {
    Opening {
        kind: Some(kind),
        wall,
        along,
        storey: 1,
    }
}

/// What is in a house: the door and windows in its walls, and what
/// stands on its ground floor, the bed first, then a couch, and the
/// chairs last, which a plan may leave out ([`Plan::less`]).
struct Room {
    openings: &'static [Opening],
    things: &'static [Thing],
}

// ------------------------------------------------------------- the rooms

/// The rooms of the Square House and the Flat House, which have the whole
/// 11 m square of floor. Every one keeps its middle clear and a way in
/// from its door, and none has its door in the middle of the front wall
/// but the third, which is the Flat House's alone: the Square House's
/// ridge comes down there (the module's own account).
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
    },
];

/// The rooms of the Two Floor House and the Roof Terrace House, beside
/// their stairs. Nothing goes in the wall the stairs go up, on the ground
/// floor, and the foot of the stairs is left clear.
const TWO_FLOORS: [Room; 4] = [
    // A single bed in the back right corner, a couch against the stairs, a
    // counter along the back and a table in the middle of the floor.
    Room {
        openings: &[
            door(0, 1.0), window(SQUARE_WINDOW, 0, -2.9), window(WIDE_WINDOW, 3, -2.0),
            upstairs(SQUARE_WINDOW, 0, -2.6), upstairs(SQUARE_WINDOW, 0, 2.6),
        ],
        things: &[
            thing(SINGLE_BED, 4.3, 3.22, 0), thing(COUCH, -2.53, 2.0, 6), thing(DESK, -2.47, 4.97, 0),
            thing(DESK, -0.47, 4.97, 0), thing(TABLE, 2.2, -0.8, 0), thing(CHAIR, 2.2, 0.6, 0),
        ],
    },
    // The bed out from the right wall, a couch against the stairs, a cupboard
    // at the back and a table by the right wall.
    Room {
        openings: &[
            door(0, 1.5), window(SQUARE_WINDOW, 0, -2.4), window(SQUARE_WINDOW, 3, -1.2),
            upstairs(SQUARE_WINDOW, 0, -2.6), upstairs(SQUARE_WINDOW, 3, 0.0),
        ],
        things: &[
            thing(DOUBLE_BED, 3.22, 3.9, 2), thing(COUCH, -2.53, -0.5, 6), thing(DESK, -2.47, 4.97, 0),
            thing(TABLE, 3.9, -1.2, 0), thing(CHAIR, 3.9, 0.2, 0),
        ],
    },
    // The door in the wall on the right, under the eaves: the bed in the
    // middle of the back, a couch against the stairs, a counter round the back
    // right corner and a table by the front wall.
    Room {
        openings: &[
            door(3, -1.5), window(WIDE_WINDOW, 0, 0.0), window(SQUARE_WINDOW, 2, 0.0),
            upstairs(SQUARE_WINDOW, 3, -2.6), upstairs(SQUARE_WINDOW, 3, 2.6),
        ],
        things: &[
            thing(DOUBLE_BED, 0.0, 3.22, 0), thing(COUCH, -2.53, -1.6, 6), thing(DESK, 3.47, 4.97, 0),
            thing(DESK_END, 4.97, 4.97, 0), thing(DESK, 4.97, 3.47, 2), thing(TABLE, 1.5, -3.2, 0),
            thing(CHAIR, 1.5, -1.8, 0),
        ],
    },
    // A single bed out from the stairs at the back, a couch by the right
    // wall, a counter along the back and a table toward the stairs.
    Room {
        openings: &[
            door(0, 2.0), window(SQUARE_WINDOW, 0, -2.0), window(SQUARE_WINDOW, 3, -2.6),
            upstairs(SQUARE_WINDOW, 0, -2.6), upstairs(SQUARE_WINDOW, 0, 2.6),
        ],
        things: &[
            thing(SINGLE_BED, -1.22, 3.9, 6), thing(COUCH, 4.53, 1.5, 2), thing(DESK, 2.47, 4.97, 0),
            thing(DESK, 4.47, 4.97, 0), thing(TABLE, -1.9, -1.2, 0), thing(CHAIR, -1.9, 0.2, 0),
        ],
    },
];

// -------------------------------------------------------------- the lawns

/// What stands out on the lawn, in the plot's own space
/// ([`crate::bag::Standing`]: -z out over the plot's front, where its sign
/// stands a metre and a half in from the front left corner). Each goes
/// with the plans that have it, whose houses stand clear of it.
const LAWNS: [&[Thing]; 19] = [
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
];

// -------------------------------------------------------------- the plans

/// One plot's house and what goes with it: which house; where on the
/// plot its middle stands, and how many eighths it is turned (turned by
/// nothing, its front wall looks out over the plot's front); which room
/// it has, of its kind's ([`ONE_FLOOR`], [`TWO_FLOORS`]), whether the room
/// is the other way round, left for right, and how many of the room's
/// last things it goes without; which door; and which lawn.
struct Plan {
    house: usize,
    at: (f32, f32),
    turn: u8,
    flipped: bool,
    room: usize,
    less: usize,
    door: usize,
    lawn: usize,
}

#[allow(clippy::too_many_arguments)]
const fn plan(house: usize, at: (f32, f32), turn: u8, flipped: bool, room: usize, less: usize, door: usize, lawn: usize) -> Plan {
    Plan {
        house,
        at,
        turn,
        flipped,
        room,
        less,
        door,
        lawn,
    }
}

impl Plan {
    /// The room it has.
    fn room(&self) -> &'static Room {
        match self.house {
            TALL | TERRACE => &TWO_FLOORS[self.room],
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

/// Every plot's plan, in Hajun's order of the plots ([`crate::land::HOMES`]).
const PLANS: [Plan; PLOTS] = [
    // 1 to 12: round the pond.
    plan(SQUARE, BACK, 0, false, 0, 0, WINDOWED, 1),
    plan(FLAT, MIDDLE, 0, false, 2, 0, GLASS_DOUBLE, 11),
    plan(TALL, BACK_RIGHT, 0, false, 3, 0, CLASSIC, 8),
    plan(SQUARE, LEFT, 6, true, 3, 0, GLASS, 4),
    plan(TERRACE, BACK, 0, false, 1, 0, DOUBLE, 6),
    plan(SQUARE, BACK_LEFT, 0, false, 1, 1, CLASSIC, 4),
    plan(FLAT, MIDDLE, 7, true, 4, 0, DOUBLE, 18),
    plan(TALL, MIDDLE, 2, false, 3, 0, GLASS_DOUBLE, 14),
    plan(FLAT, BACK_RIGHT, 2, false, 0, 0, GLASS, 2),
    plan(SQUARE, RIGHT, 2, false, 5, 0, CLASSIC, 3),
    plan(TERRACE, MIDDLE, 1, false, 2, 0, WINDOWED, 18),
    plan(SQUARE, BACK_RIGHT, 0, false, 4, 0, GLASS_DOUBLE, 11),
    // 13 to 28: the ring behind them.
    plan(FLAT, LEFT, 6, true, 1, 0, CLASSIC, 10),
    plan(SQUARE, MIDDLE, 0, true, 0, 0, CLASSIC, 1),
    plan(SQUARE, BACK, 2, false, 5, 0, GLASS_DOUBLE, 7),
    plan(FLAT, BACK_LEFT, 6, false, 3, 0, WINDOWED, 4),
    plan(SQUARE, RIGHT, 2, true, 1, 2, DOUBLE, 9),
    plan(TALL, BACK, 0, false, 1, 1, GLASS, 5),
    plan(FLAT, BACK_RIGHT, 0, true, 0, 0, WINDOWED, 3),
    plan(SQUARE, MIDDLE, 6, false, 3, 0, DOUBLE, 0),
    plan(SQUARE, BACK_LEFT, 0, true, 0, 0, GLASS, 10),
    plan(FLAT, BACK, 0, true, 2, 1, CLASSIC, 5),
    plan(TALL, (0.0, 2.5), 7, false, 0, 1, WINDOWED, 0),
    plan(TALL, LEFT, 6, false, 3, 1, DOUBLE, 17),
    plan(TERRACE, BACK_RIGHT, 2, false, 3, 1, GLASS_DOUBLE, 13),
    plan(SQUARE, BACK, 6, true, 5, 0, WINDOWED, 2),
    plan(FLAT, FRONT_RIGHT, 2, false, 4, 2, GLASS, 15),
    plan(SQUARE, BACK_RIGHT, 2, true, 3, 1, CLASSIC, 6),
    // 29 to 48: the outer ring.
    plan(TALL, MIDDLE, 0, false, 2, 1, WINDOWED, 12),
    plan(FLAT, BACK_LEFT, 0, false, 0, 1, DOUBLE, 1),
    plan(SQUARE, MIDDLE, 2, false, 1, 2, GLASS, 8),
    plan(TERRACE, LEFT, 6, false, 0, 1, CLASSIC, 16),
    plan(FLAT, BACK, 0, false, 3, 1, GLASS_DOUBLE, 0),
    plan(SQUARE, BACK_RIGHT, 0, true, 1, 2, DOUBLE, 5),
    plan(TALL, BACK_LEFT, 6, false, 1, 1, CLASSIC, 2),
    plan(FLAT, RIGHT, 2, false, 2, 2, WINDOWED, 14),
    plan(SQUARE, (2.5, 0.0), 1, true, 4, 2, CLASSIC, 0),
    plan(TERRACE, BACK, 2, false, 2, 1, GLASS, 4),
    plan(TERRACE, LEFT, 6, false, 3, 1, GLASS_DOUBLE, 0),
    plan(FLAT, MIDDLE, 1, false, 5, 0, DOUBLE, 18),
    plan(SQUARE, BACK, 0, true, 3, 0, GLASS, 11),
    plan(TALL, RIGHT, 2, false, 0, 1, WINDOWED, 0),
    plan(FLAT, BACK_RIGHT, 0, false, 1, 2, CLASSIC, 9),
    plan(SQUARE, FRONT_RIGHT, 4, false, 4, 2, WINDOWED, 17),
    plan(FLAT, (-2.0, 2.0), 7, false, 3, 0, GLASS_DOUBLE, 10),
    plan(SQUARE, BACK_LEFT, 6, false, 4, 1, DOUBLE, 0),
    plan(TERRACE, MIDDLE, 6, false, 1, 1, GLASS, 3),
    plan(TALL, MIDDLE, 1, false, 1, 1, DOUBLE, 2),
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
const SLATE: Tint = Some([92, 113, 137]);
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
/// door, frames, blanket, seats, lamp. A third of the houses are as Hajun
/// modelled them, white under a grey roof, and no two plots near each
/// other have walls of one colour.
const COATS: [Coat; PLOTS] = [
    // 1 to 12: round the pond.
    coat(CREAM, TERRACOTTA, OAK, GREEN, OWN, NAVY, OWN, PINE),
    coat(SKY, OWN, BIRCH, NAVY, OWN, EMBER, EMBER, OWN),
    coat(OWN, OWN, OWN, RED, OWN, OWN, OWN, OWN),
    coat(BUTTER, CHARCOAL, OWN, WHITE, OWN, AQUA, AQUA, INK),
    coat(OWN, OWN, OWN, OWN, OWN, HONEY, OWN, OWN),
    coat(SAGE, BROWN, WALNUT, RED, BROWN, PLUM, PLUM, PINE),
    coat(PEACH, OWN, OAK, TEAL, OWN, AZURE, AZURE, OWN),
    coat(OWN, OWN, OWN, OWN, OWN, PINK, OWN, OWN),
    coat(MINT, OWN, OWN, WHITE, OWN, FOREST, OWN, BLACK),
    coat(OWN, OWN, OWN, GREEN, OWN, OWN, OWN, OWN),
    coat(LAVENDER, OWN, BIRCH, WHITE, OWN, LILAC, OWN, OWN),
    coat(POWDER, NAVY, OWN, NAVY, OWN, EMBER, NAVY, OWN),
    // 13 to 28: the ring behind them.
    coat(APRICOT, OWN, OAK, TEAL, OWN, AQUA, AQUA, OWN),
    coat(OWN, OWN, OWN, NAVY, OWN, MOSS, OWN, PINE),
    coat(BLUSH, SLATE, OWN, WHITE, OWN, NAVY, NAVY, OWN),
    coat(OWN, OWN, OWN, YELLOW, OWN, HONEY, OWN, INK),
    coat(SEAFOAM, TEAL, OAK, ORANGE, OWN, PINK, OWN, OWN),
    coat(OWN, OWN, OWN, BLACK, OWN, OWN, OWN, OWN),
    coat(DOVE, OWN, STONE, YELLOW, CHARCOAL, AZURE, OWN, BLACK),
    coat(SHELL, PLUM, OWN, PLUM, OWN, FOREST, OWN, OWN),
    coat(OWN, OWN, OWN, OWN, OWN, LILAC, LILAC, OWN),
    coat(PISTACHIO, OWN, OAK, GREEN, OWN, EMBER, EMBER, OWN),
    coat(SAND, FOREST, WALNUT, GREEN, BROWN, MOSS, OWN, OWN),
    coat(ROSE, CHARCOAL, OWN, WHITE, OWN, NAVY, OWN, OWN),
    coat(SKY, OWN, WALNUT, BLACK, OWN, HONEY, OWN, OWN),
    coat(LILAC, NAVY, OWN, YELLOW, OWN, AQUA, OWN, OWN),
    coat(OWN, OWN, OWN, OWN, CHARCOAL, OWN, OWN, OWN),
    coat(MINT, BROWN, OWN, RED, OWN, PINK, OWN, OWN),
    // 29 to 48: the outer ring.
    coat(SAGE, TERRACOTTA, OAK, NAVY, OWN, PLUM, OWN, BLACK),
    coat(BLUSH, OWN, BIRCH, WHITE, OWN, AZURE, OWN, WHITE),
    coat(OWN, OWN, OWN, RED, OWN, OWN, OWN, OWN),
    coat(APRICOT, OWN, WALNUT, TEAL, OWN, FOREST, OWN, INK),
    coat(OWN, OWN, OWN, OWN, OWN, EMBER, OWN, OWN),
    coat(PEACH, SLATE, OWN, NAVY, OWN, LILAC, OWN, OWN),
    coat(POWDER, BRICK, OWN, WHITE, OWN, HONEY, OWN, PINE),
    coat(OWN, OWN, OWN, ORANGE, OWN, NAVY, OWN, OWN),
    coat(CORAL, CHARCOAL, OWN, WHITE, OWN, AQUA, AQUA, OWN),
    coat(LAVENDER, OWN, BIRCH, INK, OWN, OWN, OWN, OWN),
    coat(OWN, OWN, OWN, OWN, OWN, PINK, OWN, OWN),
    coat(CREAM, OWN, OAK, GREEN, OWN, MOSS, MOSS, OWN),
    coat(OWN, OWN, OWN, OWN, OWN, AZURE, OWN, OWN),
    coat(SEAFOAM, NAVY, OWN, RED, OWN, OWN, OWN, OWN),
    coat(BUTTER, OWN, OWN, NAVY, OWN, PLUM, OWN, OWN),
    coat(OWN, OWN, OWN, TEAL, OWN, HONEY, OWN, OWN),
    coat(SAND, OWN, STONE, CHARCOAL, CHARCOAL, EMBER, EMBER, OWN),
    coat(OWN, OWN, OWN, GREEN, OWN, OWN, OWN, OWN),
    coat(LILAC, OWN, WALNUT, WHITE, OWN, FOREST, OWN, BLACK),
    coat(DOVE, SLATE, OWN, YELLOW, CHARCOAL, LILAC, OWN, INK),
];

/// The parts of a thing of `kind` that `coat` has a colour for, each by
/// the name its material has in the thing's model, which is what a part
/// is painted by ([`Paint::part`]).
fn parts(kind: &str, coat: &Coat) -> Vec<(&'static str, Tint)> {
    let roofed = kind == HOUSES[SQUARE] || kind == HOUSES[TALL];
    match kind {
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

/// The made-up lot of `plot`, put together from its plan and its coat:
/// the house under the number 1, then what is in its walls, what stands
/// on its floor and what stands on the lawn, each under the next number.
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
        let (wall, along) = match (plan.flipped, opening.wall) {
            (true, 1) => (3, opening.along),
            (true, 3) => (1, opening.along),
            (true, wall) => (wall, -opening.along),
            (false, wall) => (wall, opening.along),
        };
        // A door stands on the floor of its storey, a window at its sill.
        let (kind, up) = match (opening.kind, opening.storey) {
            (None, storey) => (DOORS[plan.door], FLOOR + STOREY * f32::from(storey)),
            (Some(window), 0) => (window, SILLS[plan.house]),
            (Some(window), _) => (window, UPPER_SILL),
        };
        stand(kind, At::Wall {
            wall,
            along,
            up,
            face: 1,
        });
    }
    // What stands on its floor, where the house's place and turn bring
    // each thing's place in the room.
    let kept = room.things.len().saturating_sub(plan.less);
    for thing in &room.things[..kept] {
        let (across, turn) = if plan.flipped { (-thing.x, (8 - thing.turn) % 8) } else { (thing.x, thing.turn) };
        let (right, back) = turned(plan.turn, across, thing.z);
        stand(thing.kind, ground(x + right, FLOOR, z + back, plan.turn + turn));
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
            assert!((1..=4).contains(&count(made, &[SQUARE_WINDOW, WIDE_WINDOW])), "plot {plot}");
            assert_eq!(count(made, &[SINGLE_BED, DOUBLE_BED]), 1, "plot {plot}");
            assert!(count(made, &[COUCH, CHAIR]) >= 1, "plot {plot}");
            // What goes in a wall is in one of the house's four, and
            // everything else stands on the ground.
            for stood in &made.stands {
                let in_a_wall = DOORS.contains(&stood.kind.as_str()) || [SQUARE_WINDOW, WIDE_WINDOW].contains(&stood.kind.as_str());
                match stood.at {
                    At::Wall { wall, face, .. } => assert!(in_a_wall && wall < 4 && face == 1, "plot {plot}: {stood:?}"),
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
            let roofed = plan.house == SQUARE || plan.house == TALL;
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
            for opening in plan.room().openings.iter().filter(|opening| opening.kind.is_none() && opening.wall == 0) {
                assert!(opening.along.abs() - HALF_A_DOOR >= RIDGE, "plot {plot}: its door is {} m along the front wall", opening.along);
            }
        }
        // Every room is one its plan's house has, and goes without no more
        // than leaves it its bed and its couch.
        for (plot, plan) in PLANS.iter().enumerate() {
            let rooms = if matches!(plan.house, TALL | TERRACE) { TWO_FLOORS.len() } else { ONE_FLOOR.len() };
            assert!(plan.room < rooms && plan.door < DOORS.len() && plan.lawn < LAWNS.len() && plan.turn < 8, "plot {plot}");
            assert!(plan.room().things.len() - plan.less >= 2, "plot {plot}");
        }
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
