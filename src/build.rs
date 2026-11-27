//! Your house, in House Builder: everything the shop sells ([`PIECES`]), and
//! putting it down on your plot.
//!
//! Whatever you choose in the shop (`shop`) comes up in front of you as a
//! ghost — in nobody's way, and a house a little see-through — with the edit
//! menu open on it (`editor`). Drag it with your finger; the
//! tick puts it down where it is, solid, and the bin throws it away. Tap something once it is down and it is
//! picked back up, a ghost again until it is put down again; all but the house
//! you are standing in, which from inside nearly every tap would land on. Only
//! one thing is ever a ghost at once: the shop's button is put away until it is
//! down.
//!
//! How a piece moves depends on what it is ([`Mount`]):
//!
//! * A house stands on the land, one to a plot, wherever the whole of it fits.
//!   Its walls are made again when it loads, so that holes can be cut in them.
//! * A door or a window goes into a wall of the house, cutting the hole it
//!   needs ([`carve`]), and slides along whichever wall is under your finger:
//!   a door on the floor, a window at its sill. A house with a floor
//!   over another has a storey to each, and it goes in the one your finger is
//!   on, if that one is tall enough for it.
//! * A painting, a clock, a mirror or a shelf hangs on either face of a wall,
//!   wherever your finger is on it.
//!
//!   Neither kind ever reaches past the end of its wall, into another wall
//!   meeting it, or into a hole or onto something else hanging there: it
//!   stops as near your finger as it can, which can be right up to an edge.
//!   Whatever part of it you took hold of stays under your finger.
//! * Everything else is carried over the floor toward your finger the way a
//!   body walks: up a step, onto a bed, through a door, and right up to a
//!   wall, but never into one, any of it.
//!
//! What is down is part of the plot from then on: stood on and bumped into
//! like the island itself (`island::Islands::build_on`). Doors, windows and
//! what hangs on the walls are not: the hole in the wall is what counts.
//! Standing in a house, and nowhere else in the game, what stands between
//! the camera and you is see-through, and a tap goes through it
//! (`see_through`): zoomed out, the camera outside the house, you can tap
//! what is in there with you. When the time to build runs out, whatever is
//! still a ghost is put down where it is, and nothing can be changed any
//! more.
//!
//! The models, and the pictures the shop shows of them, are in `assets/build/`,
//! made by `blender/make_build_items.py`, which also says what the game
//! expects of each; or, for what Hajun models by hand, in `assets/mybuilds/`,
//! which they build to those same rules, so that the game loads them as they
//! are.
//!
//! **The same pieces are put down on your own plot of the town** (`stand`,
//! since 2026-11-21; Hajun: "it should close the panel and have the building
//! item in front of the character in the map similar to how the building
//! placing thing works in the house builder game"): out of your bag
//! (`bag`) rather than the shop, the same ghost with the same menu, moved by
//! the same rules. Where you are building is the [`Site`]: a plot of House
//! Builder's, an island all your own, or your plot of the town, which is a
//! square of the town's land ([`Yard`]) that whatever is put down has to
//! stay inside, the whole of it.

use std::collections::{HashMap, HashSet};

use bevy::asset::RenderAssetUsages;
use bevy::gltf::GltfMaterialName;
use bevy::light::NotShadowCaster;
use bevy::math::Affine3A;
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings};
use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldInstanceReady};

use bevy_replicon::prelude::Remote;
use roundtown_net::{At, Built};

use crate::builder::{Building, InGame, OnlinePlot};
use crate::editor::{self, CannotStand, Edit, EditMenu, Editable, Touch, Untappable};
use crate::hud::WantsPointer;
use crate::island::{self, Island, Islands, Venue};
use crate::lobby::Seat;
use crate::shop::Chosen;
use crate::{EYE_HEIGHT, PLAYER_RADIUS, Player, ThirdPersonCamera, WATER_Y};

pub(crate) mod paint;
#[cfg(test)]
mod premade_check;
pub(crate) mod stand;
mod town;

pub(crate) use town::TownHouse;

/// Everything the shop sells, in the order it shows them: only what Hajun
/// has modelled, in `assets/mybuilds/` (their call, 2026-10-03). The
/// script's pieces in `assets/build/` are no longer sold, nor packed into the
/// APK.
pub(crate) const PIECES: &[PieceDef] = &[
    my_house("square_house", "Square House"),
    my_house("square_house_roofless", "Flat House"),
    my_house("square_house_two_floor", "Square House 2F"),
    my_house("square_house_roofless_two_floor", "Roof Terrace 2F"),
    my_door("classic_door", "Classic Door", ONE_LEAF),
    my_door("classic_door_with_window", "Classic Window Door", ONE_LEAF),
    my_door("classic_double_door", "Classic Double Door", TWO_LEAVES),
    my_door("glass_door", "Glass Door", ONE_LEAF),
    my_door("glass_double_door", "Glass Double Door", TWO_LEAVES),
    my_window("square_window", "Square Window"),
    my_window("wide_window", "Wide Window"),
    my_floor("singlebed", "Single Bed", Tab::Furniture),
    my_floor("doublebed", "Double Bed", Tab::Furniture),
    my_floor("cafechair", "Cafe Chair", Tab::Furniture),
    my_floor("circletable", "Circle Table", Tab::Furniture),
    my_floor("cafedeskedge", "Cafe Desk Edge", Tab::Furniture),
    my_floor("cafedeskcorner", "Cafe Desk Corner", Tab::Furniture),
    my_floor("softcouch", "Soft Couch", Tab::Furniture),
    // Lit at night like the town's, which it is the model of (`lamps`).
    my_floor("streetlamp", "Street Lamp", Tab::Furniture),
    // For the town's plots alone, out of the bag (since 2026-11-25; Hajun:
    // "add all the new house models ... add all the new furnitures to the
    // item shop as well"): House Builder's shop sells none of what follows.
    town_house("wide_roof_house", "Wide Roof House"),
    town_house("square_house_three_floor", "Square House 3F"),
    town_house("square_house_four_floor", "Square House 4F"),
    town_house("square_house_five_floor", "Square House 5F"),
    town_house("square_house_six_floor", "Square House 6F"),
    town_house("square_house_seven_floor", "Square House 7F"),
    town_house("square_house_eight_floor", "Square House 8F"),
    town_house("square_house_nine_floor", "Square House 9F"),
    town_house("square_house_ten_floor", "Square House 10F"),
    town_house("square_house_roofless_three_floor", "Roof Terrace 3F"),
    town_house("square_house_roofless_four_floor", "Roof Terrace 4F"),
    town_house("square_house_roofless_five_floor", "Roof Terrace 5F"),
    town_house("square_house_roofless_six_floor", "Roof Terrace 6F"),
    town_house("square_house_roofless_seven_floor", "Roof Terrace 7F"),
    town_house("square_house_roofless_eight_floor", "Roof Terrace 8F"),
    town_house("square_house_roofless_nine_floor", "Roof Terrace 9F"),
    town_house("square_house_roofless_ten_floor", "Roof Terrace 10F"),
    // Hajun's own three, and the 56 they downloaded (by CreativeTrio, CC0),
    // made three times as big (two and a half by `blender/scale_downloads.py`,
    // then 1.2 by `grow_downloads.py`: "i meant 3x"), each under the name
    // its files have.
    new_one("end_table", "end_table", "End Table"),
    new_one("cabinet", "cabinet", "Cabinet"),
    new_one("bathtub", "bathtub", "Bathtub"),
    new_small("alarm_clock", "Alarm Clock", "Alarm Clock"),
    new_one("chair", "Chair", "Chair"),
    new_one("closed_umbrella", "Closed Umbrella", "Closed Umbrella"),
    new_small("computer_mouse", "Computer Mouse", "Computer Mouse"),
    new_small("cup_of_tea", "Cup Of Tea", "Cup Of Tea"),
    new_small("dumbbell", "Dumbbell", "Dumbbell"),
    new_small("empty_box", "Empty Box", "Empty Box"),
    new_small("frying_pan", "Frying Pan", "Frying Pan"),
    new_small("hair_dryer", "Hair Dryer", "Hair Dryer"),
    new_small("hamburger", "Hamburger", "Hamburger"),
    new_one("hand_rake", "Hand Rake", "Hand Rake"),
    new_small("mug_with_office_tool", "Mug With Office Tool", "Mug With Office Tool"),
    new_small("screwdriver", "Screwdriver", "Screwdriver"),
    new_small("stapler", "Stapler", "Stapler"),
    new_one("stool", "Stool", "Stool"),
    new_one("table", "Table", "Table"),
    new_small("tissue_box", "Tissue Box", "Tissue Box"),
    new_one("trash_bin", "Trash Bin", "Trash Bin"),
    new_small("watering_can", "Watering Can", "Watering Can"),
    new_small("wine_glass", "Wine Glass", "Wine Glass"),
    new_one("armchair", "Armchair", "Armchair"),
    new_one("bathroom_sink", "Bathroom Sink", "Bathroom Sink"),
    new_one("coat_rack", "Coat Rack", "Coat Rack"),
    new_one("desk", "Desk", "Desk"),
    new_small("desk_fan", "Desk Fan", "Desk Fan"),
    new_small("doll", "Doll", "Doll"),
    new_one("dresser", "Dresser", "Dresser"),
    new_small("flowers", "Flowers", "Flowers"),
    new_small("fruit_bowl", "Fruit Bowl", "Fruit Bowl"),
    new_small("globe", "Globe", "Globe"),
    new_small("headphones", "Headphones", "Headphones"),
    // Lit at night (`lamps::BY_KIND`).
    new_small("lamp_with_shade", "Lamp With Shade", "Lamp With Shade"),
    new_one("mailbox", "Mailbox", "Mailbox"),
    new_small("monitor", "Monitor", "Monitor"),
    new_hanging("painting", "Painting", "Painting"),
    new_small("piggy_bank", "Piggy Bank", "Piggy Bank"),
    new_small("potted_plant", "Potted Plant", "Potted Plant"),
    new_small("present", "Present", "Present"),
    new_small("printer", "Printer", "Printer"),
    new_one("propane_tank", "Propane Tank", "Propane Tank"),
    new_small("rubber_duck", "Rubber Duck", "Rubber Duck"),
    new_one("table_large_circular", "Table Large Circular", "Table Large Circular"),
    new_one("toilet", "Toilet", "Toilet"),
    new_one("barbecue", "Barbecue", "Barbecue"),
    new_one("bed", "Bed", "Bed"),
    // Hung from a ceiling, as the chandelier is (Hajun, 2026-11-25:
    // "chandelier should only be placeable on ceiling, and ceiling fan also
    // only on ceiling").
    new_ceiling("ceiling_fan", "Ceiling Fan", "Ceiling Fan"),
    new_small("computer", "Computer", "Computer"),
    new_one("couch", "Couch", "Couch"),
    // The file keeps the download's spelling.
    new_one("refrigerator", "Refrigirator", "Refrigerator"),
    new_one("rocking_chair", "Rocking Chair", "Rocking Chair"),
    new_one("stove", "Stove", "Stove"),
    new_one("treadmill", "Treadmill", "Treadmill"),
    new_one("grandfathers_clock", "Grandfathers Clock", "Grandfathers Clock"),
    new_small("safe", "Safe", "Safe"),
    new_one("telescope", "Telescope", "Telescope"),
    // Hung from a ceiling, and lit at night (`lamps::BY_KIND`; Hajun:
    // "chandelier should emit light").
    new_ceiling("chandelier", "Chandelier", "Chandelier"),
];

/// How far up its storey a window's hole starts, as a share of the storey's
/// height. It stays there: a window is dragged only along its wall, and up or
/// down only from one storey to another, the way a door is.
const WINDOW_SILL: f32 = 0.22;
/// How much wall two holes, or two things hung on one face, leave between
/// them.
const APART: f32 = 0.5;
/// How near a door, a window or a hanging comes to the edges of its wall: its
/// ends, another wall meeting it, and the floor and ceiling of its storey. A
/// little more than the made windows' sills reached out past their frames,
/// 5 cm.
const EDGE: f32 = 0.06;
/// How far in from the edge of its footprint a piece carried over the floor
/// is tested against walls, and anything else it could not be carried onto:
/// just inside it, so that it can stand flush against them, touching, which
/// a test on the edge itself would not allow. Until 2026-10-03 it was tested 2
/// cm out from its edge, which left it 2 to 6 cm short of whatever it was
/// pushed against, and two of Hajun's cafe desks would not go together.
const TOUCH: f32 = 0.001;
/// How near a piece carried over the floor has to come to lining up with
/// something before it snaps into line, in metres: flush against another
/// piece or a wall it is square to, and then, along another piece it is
/// flush against, with that one's ends or its middle (Hajun, 2026-10-03:
/// there was "no snapping at all").
const SNAP: f32 = 0.2;
/// How far a new piece comes up in front of you, from your edge to its.
const IN_FRONT: f32 = 0.6;
/// The same for a house, which is too big to work on at arm's length: at 0.6 m
/// its front wall filled the view. Out here you can see it whole, with room
/// to walk up to it.
pub(crate) const HOUSE_IN_FRONT: f32 = 3.0;
/// How far out of level the land under a house may be.
const HOUSE_TILT: f32 = 0.25;
/// A dragged house goes toward the finger in steps this long, and no further
/// than this in a frame, in metres.
const HOUSE_STEP: f32 = 0.5;
const HOUSE_REACH: f32 = 12.0;
/// How far apart the points round the edge of a piece that are kept out of
/// walls are, in metres: less than any wall is thick, 0.25 m at the least, so
/// that none slips between two.
const EDGE_STEP: f32 = 0.2;
/// A new piece faces you, turned to the nearest of these.
const TURN_STEP_DEG: f32 = 45.0;
/// How a piece turned into a wall or another piece is moved out of it: tried
/// this many ways round, at spots this far apart, in metres, out to this much
/// further than the corner of it furthest from its middle.
const ROOM_WAYS: u32 = 16;
const ROOM_STEP: f32 = 0.1;
const ROOM_SPARE: f32 = 0.5;
/// How much of a house is drawn while it is a ghost: its own colours, a little
/// see-through. Only a house is: everything else is drawn as it will be once
/// it is down (Hajun, 2026-10-01).
const GHOST_ALPHA: f32 = 0.65;
/// How near someone has to come to a door with a swinging leaf for it to
/// open, from the middle of the doorway along the ground: from outside, coming
/// in, and from inside, going out, which has to be nearer. Outside was 3.5 m
/// until 2026-10-03, when Hajun found it opening too far off.
const SWING_OUTSIDE: f32 = 3.0;
const SWING_INSIDE: f32 = 1.5;
/// How much further off than that someone can go before it shuts again, so
/// that it does not flap with them standing at the edge.
const SWING_HOLD: f32 = 0.5;
/// How long a door takes to swing all the way open, and shut, in seconds.
/// Open fast: running, someone comes from [`SWING_OUTSIDE`] to the door in
/// under half a second.
const SWING_OPEN_TIME: f32 = 0.35;
const SWING_SHUT_TIME: f32 = 0.8;

/// One thing the shop sells.
pub(crate) struct PieceDef {
    /// Where its files are, under `assets/`: [`MADE`] by the script, or
    /// [`MINE`], modelled by hand.
    pub dir: &'static str,
    /// What it goes by, over the wire and in a bag (`roundtown_net::bag`):
    /// never changed once anyone owns one.
    pub kind: &'static str,
    /// Its files: `<dir>/<file>.glb` and, for the shop, `<dir>/<file>.png`.
    /// Its kind, but for the 56 downloaded on 2026-11-24, which keep the
    /// names they came with ("Alarm Clock").
    pub file: &'static str,
    /// What the shop calls it. Plain ASCII: the built-in font has no more.
    pub name: &'static str,
    pub tab: Tab,
    pub mount: Mount,
    /// A part of it that swings open by itself, if one does.
    pub swing: Option<Swing>,
    /// Whether House Builder's shop sells it: the nineteen it always has,
    /// and none of what came for the town's plots alone on 2026-11-25.
    pub house_builder: bool,
    /// Whether it is a small thing that goes on top of others, a desk, a
    /// table, a shelf, a bed, as well as on a floor (since 2026-11-25; Hajun:
    /// "the things like hamburger, mug with office tool, etc. the desk
    /// gadgets - they are supposed to be easy to be placed on desks and
    /// tables,etc on other surfaces"): dragged over the top of something of
    /// yours, it goes up onto it ([`top_under`]), and it is in nothing it
    /// stands on ([`stops_on`]). Which they are is mine: what one would put
    /// on a desk or a table, and none of what stands on a floor however
    /// small (a stool, a bin, a gas bottle).
    pub small: bool,
}

/// A door's leaves that swing open by themselves as someone comes up to the
/// door, and shut again once they have gone ([`swing_doors`]).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Swing {
    /// The parts that swing, by their names in Blender, each with whatever
    /// hangs from it: each turns about the upright through its origin, its
    /// hinge.
    pub leaves: &'static [&'static str],
    /// How far each turns to be open, in degrees, modelled shut at 0:
    /// whichever way takes its far edge out of the house, Blender's -Y, which
    /// is the other way for a leaf hung on the other side.
    pub open_deg: f32,
}

/// Hajun's doors, hung on one side and swinging out to stand square to the
/// wall (their call, 2026-10-01, after trying 135, 180 and 170 degrees).
const ONE_LEAF: Swing = Swing {
    leaves: &["Door"],
    open_deg: 90.0,
};
/// Their double doors: the leaf hung on the other side is the first one's
/// copy, as Blender names it.
const TWO_LEAVES: Swing = Swing {
    leaves: &["Door", "Door.001"],
    open_deg: 90.0,
};

impl PieceDef {
    pub(crate) fn picture(&self) -> String {
        format!("{}/{}.png", self.dir, self.file)
    }

    fn model(&self) -> String {
        format!("{}/{}.glb", self.dir, self.file)
    }

    /// Whether it goes on a wall of the house, so that there has to be one.
    pub(crate) fn on_wall(&self) -> bool {
        matches!(self.mount, Mount::Door | Mount::Window | Mount::Hanging)
    }

    /// Whether it cuts a hole in the wall it is on.
    fn holed(&self) -> bool {
        self.mount.holed()
    }

    /// Whether, once down, it is part of what is stood on and bumped into:
    /// not what is in a wall, nor what hangs from a ceiling, which is over
    /// everyone's heads and which no other thing is to hang from.
    fn solid(&self) -> bool {
        !self.on_wall() && self.mount != Mount::Ceiling
    }
}

/// Made by `blender/make_build_items.py`.
const MADE: &str = "build";
/// Modelled by hand, to the rules the script's pieces keep.
const MINE: &str = "mybuilds";
/// What came to stand on the town's plots on 2026-11-25: Hajun's cabinet,
/// bathtub and end table, and the 56 things he downloaded.
const NEW_ONES: &str = "mybuilds/newones";

const fn house(kind: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        dir: MADE,
        kind,
        file: kind,
        name,
        tab: Tab::Foundation,
        mount: Mount::House,
        swing: None,
        house_builder: true,
        small: false,
    }
}

/// A house Hajun modelled, in `assets/mybuilds/`.
const fn my_house(kind: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        dir: MINE,
        ..house(kind, name)
    }
}

/// A house Hajun modelled for the town's plots alone (since 2026-11-25), in
/// `assets/mybuilds/`: House Builder's shop does not sell it.
const fn town_house(kind: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        house_builder: false,
        ..my_house(kind, name)
    }
}

/// Furniture or a decoration for the town's plots alone (since 2026-11-25),
/// in `assets/mybuilds/newones/`, under the name its files have.
const fn new_one(kind: &'static str, file: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        dir: NEW_ONES,
        file,
        house_builder: false,
        ..floor(kind, name, Tab::Furniture)
    }
}

/// The same, a small thing that goes on top of others as well
/// ([`PieceDef::small`]).
const fn new_small(kind: &'static str, file: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        small: true,
        ..new_one(kind, file, name)
    }
}

/// The same, hung on a wall of the house ([`Mount::Hanging`]): the painting,
/// which came lying down, its picture up, and was stood up in its `.blend`
/// to hang as the script's did (Hajun, 2026-11-25: "painting is supposed to
/// go on the wall, not the ground. rotate the blender asset for that one
/// and make it go on walls").
const fn new_hanging(kind: &'static str, file: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        tab: Tab::Decorations,
        mount: Mount::Hanging,
        ..new_one(kind, file, name)
    }
}

/// The same, hung from a ceiling ([`Mount::Ceiling`]).
const fn new_ceiling(kind: &'static str, file: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        mount: Mount::Ceiling,
        ..new_one(kind, file, name)
    }
}

/// Furniture or a decoration Hajun modelled, in `assets/mybuilds/`.
const fn my_floor(kind: &'static str, name: &'static str, tab: Tab) -> PieceDef {
    PieceDef {
        dir: MINE,
        ..floor(kind, name, tab)
    }
}

/// A window Hajun modelled, in `assets/mybuilds/`.
const fn my_window(kind: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        dir: MINE,
        ..opening(kind, name, Mount::Window)
    }
}

/// A door Hajun modelled, in `assets/mybuilds/`, whose leaf swings.
const fn my_door(kind: &'static str, name: &'static str, swing: Swing) -> PieceDef {
    PieceDef {
        dir: MINE,
        swing: Some(swing),
        ..opening(kind, name, Mount::Door)
    }
}

const fn opening(kind: &'static str, name: &'static str, mount: Mount) -> PieceDef {
    PieceDef {
        dir: MADE,
        kind,
        file: kind,
        name,
        tab: Tab::Openings,
        mount,
        swing: None,
        house_builder: true,
        small: false,
    }
}

/// Something Hajun modelled to hang on a wall, in `assets/mybuilds/`. None
/// does yet: the hangings the shop sold were the script's painting, clock,
/// mirror and wall shelf, until 2026-10-03, and a hanging goes on a wall as
/// they did.
#[allow(dead_code)]
const fn my_hanging(kind: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        dir: MINE,
        kind,
        file: kind,
        name,
        tab: Tab::Decorations,
        mount: Mount::Hanging,
        swing: None,
        house_builder: true,
        small: false,
    }
}

const fn floor(kind: &'static str, name: &'static str, tab: Tab) -> PieceDef {
    PieceDef {
        dir: MADE,
        kind,
        file: kind,
        name,
        tab,
        mount: Mount::Floor,
        swing: None,
        house_builder: true,
        small: false,
    }
}

/// The shop's tabs.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Tab {
    #[default]
    Foundation,
    Openings,
    Furniture,
    Decorations,
}

impl Tab {
    pub(crate) const ALL: [Self; 4] = [
        Self::Foundation,
        Self::Openings,
        Self::Furniture,
        Self::Decorations,
    ];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Foundation => "Foundation",
            Self::Openings => "Doors & Windows",
            Self::Furniture => "Furniture",
            Self::Decorations => "Decorations",
        }
    }

    /// Whether House Builder's shop sells anything under it. One that sells
    /// nothing is not shown, as Decorations is not while none of Hajun's
    /// pieces is one.
    pub(crate) fn sold(self) -> bool {
        PIECES.iter().any(|def| def.tab == self && def.house_builder)
    }
}

/// Where a piece goes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Mount {
    /// A whole house, standing on the land: one to a plot.
    House,
    /// In a wall of the house, standing on the floor of the storey it is in,
    /// cutting the hole its frame fills. Its model's origin is at the bottom
    /// middle of the hole, its +Y in Blender faces into the house, and its
    /// frame is the parts called "Frame...".
    Door,
    /// In a wall of the house like a door, but [`WINDOW_SILL`] of the way up
    /// the storey it is in rather than on its floor.
    Window,
    /// On a wall of the house. Its model's origin is the middle of its back,
    /// and it faces -Y in Blender, out of the wall.
    Hanging,
    /// Standing on whatever is under it.
    Floor,
    /// Hung from the ceiling of a room of a house (since 2026-11-25; Hajun:
    /// "chandelier should only be placeable on ceiling, and ceiling fan also
    /// only on ceiling"): its top against the underside of what is over the
    /// floor of the storey it is in, the floor upstairs or the roof, at least
    /// [`HEADROOM`] over that floor; carried along under it wherever the
    /// finger takes it, the whole of it on your plot, and put down only
    /// under a ceiling ([`hangs`]). In the town alone.
    Ceiling,
}

impl Mount {
    /// Whether it cuts a hole in the wall it is on.
    fn holed(self) -> bool {
        matches!(self, Self::Door | Self::Window)
    }
}

/// Every piece's model and picture, loaded when the app opens so that what
/// the shop shows and what it sells are there the moment they are wanted.
#[derive(Resource)]
pub(crate) struct Stock {
    scenes: Vec<Handle<WorldAsset>>,
    pictures: Vec<Handle<Image>>,
    /// A see-through copy of each material a piece is drawn with, made the
    /// first time something drawn with it is a ghost.
    ghosts: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
}

impl Stock {
    /// With nothing in it: for what only wants its ghosts.
    #[cfg(test)]
    fn for_tests() -> Self {
        Self {
            scenes: Vec::new(),
            pictures: Vec::new(),
            ghosts: HashMap::new(),
        }
    }

    fn index(def: &PieceDef) -> usize {
        PIECES
            .iter()
            .position(|piece| piece.kind == def.kind)
            .unwrap_or_default()
    }

    fn scene(&self, def: &PieceDef) -> Handle<WorldAsset> {
        self.scenes[Self::index(def)].clone()
    }

    pub(crate) fn picture(&self, def: &PieceDef) -> Handle<Image> {
        self.pictures[Self::index(def)].clone()
    }

    /// `solid` as a ghost: the same colours, only see-through. Blended, like
    /// everything else see-through in this world: both platforms draw it the
    /// same.
    fn ghost_of(
        &mut self,
        solid: &Handle<StandardMaterial>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.ghosts
            .entry(solid.id())
            .or_insert_with(|| {
                let mut look = materials.get(solid).cloned().unwrap_or_default();
                let alpha = look.base_color.alpha() * GHOST_ALPHA;
                look.base_color.set_alpha(alpha);
                look.alpha_mode = AlphaMode::Blend;
                materials.add(look)
            })
            .clone()
    }
}

/// Something on a plot, down or being put down.
#[derive(Component)]
pub(crate) struct Piece {
    pub def: &'static PieceDef,
}

/// Still being put down: in nobody's way, and a house see-through.
#[derive(Component)]
pub(crate) struct Ghost;

/// Just out of the shop, to be put in front of you once its model has loaded
/// and how big it is is known.
#[derive(Component)]
struct Fresh;

/// The box a piece's model fills, in its own space.
#[derive(Component, Clone, Copy)]
pub(crate) struct Bounds {
    pub(crate) low: Vec3,
    pub(crate) high: Vec3,
}

/// A mesh of a ghost, and the material it has once it is down.
#[derive(Component)]
struct Solid(Handle<StandardMaterial>);

/// What is inside a house: the boxes its `Floor...` objects fill, in its own
/// space.
#[derive(Component)]
struct Rooms(Vec<(Vec3, Vec3)>);

/// The house you are standing in, while you are ([`Rooms::hold`]): only in
/// a house that is down is anything in the way of the camera drawn
/// see-through, and any of its walls in the way takes its ceiling with it
/// (`see_through`).
#[derive(Component)]
pub(crate) struct YouAreIn;

impl Rooms {
    /// Whether someone standing at `at`, in the house's own space, is inside
    /// it: over one of its floors, and lower than the top of it, `top`.
    fn hold(&self, at: Vec3, top: f32) -> bool {
        at.y < top
            && self.0.iter().any(|(low, high)| {
                at.x >= low.x && at.x <= high.x && at.z >= low.z && at.z <= high.z
            })
    }
}

/// How much of a wall a door, a window or a hanging takes up, from its
/// origin: half its width, either way along the wall, and how far it reaches
/// below and above it. Read off its model once it has loaded: for a door or a
/// window, its frame, which is the hole it needs.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
struct Span {
    half: f32,
    below: f32,
    above: f32,
}

impl Span {
    /// Of a model filling the box from `low` to `high`, in its own space.
    fn of(low: Vec3, high: Vec3) -> Self {
        Self {
            half: (-low.x).max(high.x),
            below: -low.y,
            above: high.y,
        }
    }
}

/// Where on a wall of its house a door, a window or a hanging is.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) struct OnWall {
    /// The wall, which it disappears along with when the wall is in the way
    /// of the camera (`see_through`).
    pub(crate) wall: Entity,
    /// How far along the wall, and up it: to the middle of the bottom of a
    /// hole, to the middle of a hanging.
    along: f32,
    up: f32,
    /// Which face a hanging is on: 1 or -1, along the way the wall is thin.
    face: f32,
}

/// A door whose leaves swing ([`Swing`]), and how far open they are.
#[derive(Component)]
struct Doorway {
    leaves: Vec<Leaf>,
    /// How far along the doorway from its middle, either way, the middles of
    /// its leaves are: nowhere, for one leaf filling it. How near someone is
    /// to the door is how near they are to the nearest point between them.
    reach: f32,
    /// How far through their swing they are, from 0 shut to 1 open.
    through: f32,
    /// Whether someone is near enough to have it open.
    wanted: bool,
}

/// A leaf of a door, that swings.
struct Leaf {
    part: Entity,
    /// Its turn when shut, as modelled.
    shut: Quat,
    /// How far it turns about the upright to be open, in radians: whichever
    /// way takes it out of the house.
    open: f32,
}

/// Where on a door, a window or a hanging being dragged the finger took hold
/// of it, from its origin: along its wall and up it. Kept from the first
/// moment of a drag to its end, so that what was pressed stays under the
/// finger.
#[derive(Component, Clone, Copy, Default)]
struct Grip {
    along: f32,
    up: f32,
}

/// How high up its wall a door, a window or a hanging is asked to go, which
/// also says which storey it goes in. A door always stands on the floor of
/// its storey.
#[derive(Clone, Copy, Debug)]
enum Height {
    /// `share` of the way up the storey at the height `near`: where it first
    /// goes in, and where a window is dragged.
    Share { share: f32, near: f32 },
    /// With its origin at this height, in the storey its middle is then in:
    /// where it is dragged.
    At(f32),
}

/// One wall of a house, made again from the box it was modelled as, with
/// holes in it for its doors and windows.
#[derive(Component)]
pub(crate) struct Wall {
    /// Its corners, in the house's own space.
    low: Vec3,
    high: Vec3,
    /// Which way into the house is from it.
    inside: Vec3,
    mesh: Handle<Mesh>,
    /// The holes it has, as [`carve`] takes them.
    holes: Vec<[f32; 4]>,
    /// How high each storey of the house runs up it, bottom to top. A door
    /// or a window goes in one of them, and a hanging hangs in one.
    storeys: Vec<(f32, f32)>,
    /// Where the house's other walls, and what else is built into it, like
    /// its staircase, stand against it, as [`carve`] takes a hole.
    joins: Vec<[f32; 4]>,
}

/// A wall of a house on a plot of the town that runs up past one of its
/// floors (each of the Square House 2F's four, modelled whole from the
/// ground to the roof), drawn in bands, a floor to each, so that each
/// floor's walls are painted by themselves (`paint`; since 2026-11-25):
/// the heights it is cut at, the tops of the floors it runs up past, and
/// the meshes of the bands over the first, each drawn by a [`Band`] under
/// it. The wall draws the band from its foot to the first cut itself, and
/// is the one wall to everything else, as it was: what its doors and
/// windows are in, and where it comes among its house's walls
/// ([`walls_of`]). A hole is cut in whichever band it is in
/// ([`carve_walls`]).
#[derive(Component)]
struct Banded {
    cuts: Vec<f32>,
    upper: Vec<Handle<Mesh>>,
}

/// A band of a [`Banded`] wall over its first, drawn by itself: as solid
/// as a wall is, its triangles a list of their own ([`reshape_plot`]).
#[derive(Component)]
struct Band;

impl Wall {
    /// The wall modelled as the box from `low` to `high`, in a house with
    /// `floors`, and walls and whatever else stands `against` them, this wall
    /// among them.
    fn new(low: Vec3, high: Vec3, floors: &[(Vec3, Vec3)], against: &[(Vec3, Vec3)]) -> Self {
        let mut wall = Self {
            low,
            high,
            inside: Vec3::ZERO,
            mesh: Handle::default(),
            holes: Vec::new(),
            storeys: Vec::new(),
            joins: Vec::new(),
        };
        wall.inside = wall.across() * wall.inward(floors);
        wall.storeys = wall.storeys_of(floors);
        wall.joins = wall.joins_of(against);
        wall
    }

    /// Where the other walls and fixtures among `against` stand against it,
    /// from corner to corner: at a corner of the house, wherever another wall
    /// runs into it, and where the staircase goes up beside it. A hole there
    /// would open into the end of the other wall, or the side of the stairs,
    /// so everything in or on it keeps [`EDGE`] from them, as from its own
    /// ends.
    fn joins_of(&self, against: &[(Vec3, Vec3)]) -> Vec<[f32; 4]> {
        // Walls modelled to meet can be a hair apart.
        const TOUCH: f32 = 0.01;
        let (start, end) = self.span();
        against
            .iter()
            .filter(|&&(low, high)| (low, high) != (self.low, self.high))
            .filter(|(low, high)| {
                low.x <= self.high.x + TOUCH
                    && high.x >= self.low.x - TOUCH
                    && low.z <= self.high.z + TOUCH
                    && high.z >= self.low.z - TOUCH
                    && low.y < self.high.y
                    && high.y > self.low.y
            })
            .filter_map(|(low, high)| {
                let (from, to) = if self.along_x() {
                    (low.x.max(start), high.x.min(end))
                } else {
                    (low.z.max(start), high.z.min(end))
                };
                (to - from > TOUCH).then_some([
                    from,
                    low.y.max(self.low.y),
                    to,
                    high.y.min(self.high.y),
                ])
            })
            .collect()
    }

    /// Its storeys: from its foot up to the underside of the first floor
    /// over it, from the top of that floor up to the next, and so on to its
    /// top. A floor counts where there is room to stand under it; a house
    /// with one floor has one storey, the whole wall. What is left of it over
    /// the top floor can be low, round a roof terrace, or nothing at all,
    /// under a flat top level with the wall's.
    fn storeys_of(&self, floors: &[(Vec3, Vec3)]) -> Vec<(f32, f32)> {
        let mut over: Vec<(f32, f32)> = floors
            .iter()
            .map(|(low, high)| (low.y, high.y))
            .filter(|&(_, top)| top <= self.high.y + 1e-3)
            .collect();
        over.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut storeys = Vec::new();
        let mut bottom = self.low.y;
        for (under, top) in over {
            if under - bottom >= crate::PLAYER_HEIGHT {
                storeys.push((bottom, under));
                bottom = top;
            }
        }
        storeys.push((bottom, self.high.y));
        storeys
    }

    /// Its storeys, the one at the height `up` first and then the others,
    /// nearest it first.
    fn storeys_from(&self, up: f32) -> Vec<(f32, f32)> {
        let away = |&(bottom, top): &(f32, f32)| (bottom - up).max(up - top).max(0.0);
        let mut storeys = self.storeys.clone();
        storeys.sort_by(|a, b| away(a).total_cmp(&away(b)));
        storeys
    }

    /// The storey at the height `up`, or the nearest.
    fn storey(&self, up: f32) -> (f32, f32) {
        self.storeys_from(up)
            .first()
            .copied()
            .unwrap_or((self.low.y, self.high.y))
    }

    /// It runs along x, and is thin along z.
    fn along_x(&self) -> bool {
        self.high.x - self.low.x >= self.high.z - self.low.z
    }

    /// The way through it, from one face to the other.
    fn across(&self) -> Vec3 {
        if self.along_x() { Vec3::Z } else { Vec3::X }
    }

    /// Where it starts and ends, along it.
    fn span(&self) -> (f32, f32) {
        if self.along_x() {
            (self.low.x, self.high.x)
        } else {
            (self.low.z, self.high.z)
        }
    }

    /// The middle of it, through its thickness, and half how thick it is.
    fn thin(&self) -> (f32, f32) {
        let (near, far) = if self.along_x() {
            (self.low.z, self.high.z)
        } else {
            (self.low.x, self.high.x)
        };
        ((near + far) * 0.5, (far - near) * 0.5)
    }

    /// The point `along` it and `up` it, `out` from its middle toward
    /// [`Self::across`].
    fn point(&self, along: f32, up: f32, out: f32) -> Vec3 {
        let middle = self.thin().0 + out;
        if self.along_x() {
            Vec3::new(along, up, middle)
        } else {
            Vec3::new(middle, up, along)
        }
    }

    /// How far along the ray from `from` going `way` it crosses the plane
    /// through the middle of this wall, ahead of it, if it does: where on the
    /// wall a finger that is off the wall itself is, beyond its edge.
    fn across_plane(&self, from: Vec3, way: Vec3) -> Option<f32> {
        let across = self.across();
        let facing = way.dot(across);
        let far = (self.thin().0 - from.dot(across)) / facing;
        (facing.abs() > 1e-4 && far > 0.0).then_some(far)
    }

    /// How far along it `at` is.
    fn along_of(&self, at: Vec3) -> f32 {
        if self.along_x() { at.x } else { at.z }
    }

    /// How far out of its middle `at` is, toward [`Self::across`].
    fn out_of(&self, at: Vec3) -> f32 {
        (if self.along_x() { at.z } else { at.x }) - self.thin().0
    }

    /// Which way into the house is from it, in the house's own space.
    pub(crate) fn way_in(&self) -> Vec3 {
        self.inside
    }

    /// The box it was modelled as, in the house's own space, with none of
    /// the holes its doors and windows have cut in it: what the camera
    /// stands clear of in the house you are in (`indoors`), so that it never
    /// goes out through a doorway or a window.
    pub(crate) fn whole(&self) -> (Vec3, Vec3) {
        (self.low, self.high)
    }

    /// Which way into the house is, along [`Self::across`]: toward the side
    /// with floor beside it, or failing that toward the house's middle.
    fn inward(&self, floors: &[(Vec3, Vec3)]) -> f32 {
        let middle = (self.low + self.high) * 0.5;
        let reach = self.thin().1 + 0.3;
        let floored = |side: f32| {
            let at = middle + self.across() * side * reach;
            floors.iter().any(|(low, high)| {
                at.x >= low.x && at.x <= high.x && at.z >= low.z && at.z <= high.z
            })
        };
        match (floored(1.0), floored(-1.0)) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ if middle.dot(self.across()) > 0.0 => -1.0,
            _ => 1.0,
        }
    }

    /// What its face is, from end to end and top to bottom, as [`carve`]
    /// takes it.
    fn face(&self) -> [f32; 4] {
        let (start, end) = self.span();
        [start, self.low.y, end, self.high.y]
    }

    /// Where on it a piece that goes as `mount` says and takes up `span` can
    /// go, as near `along` it and the `height` asked for as it can: a door on
    /// the floor of a storey, a window or a hanging (on `face`) anywhere up
    /// one, [`EDGE`] clear of its floor and ceiling. In the storey asked for
    /// if there is room there, or else the nearest that has some: one too
    /// low for it, like the parapet round a roof terrace, takes none. [`EDGE`]
    /// clear of its ends and of the walls meeting it, and [`APART`] from
    /// everything else in it or on it, `others`, each with its span and
    /// whether it is a hole. `None` if there is no room anywhere.
    fn fit(
        &self,
        mount: Mount,
        span: Span,
        height: Height,
        along: f32,
        face: f32,
        others: &[(OnWall, Span, bool)],
    ) -> Option<(f32, f32)> {
        let holed = mount.holed();
        let (start, end) = self.span();
        let room = (start + EDGE + span.half, end - EDGE - span.half);
        let near = match height {
            Height::Share { near, .. } => near,
            Height::At(up) => up + (span.above - span.below) * 0.5,
        };
        let on_storey = |(bottom, top): (f32, f32)| {
            let up = if mount == Mount::Door {
                if bottom + span.above > top + 1e-3 {
                    return None;
                }
                bottom
            } else {
                let lowest = bottom + EDGE + span.below;
                let highest = top - EDGE - span.above;
                if lowest > highest {
                    return None;
                }
                let want = match height {
                    Height::Share { share, .. } => bottom + (top - bottom) * share,
                    Height::At(up) => up,
                };
                want.clamp(lowest, highest)
            };
            let (low, high) = (up - span.below, up + span.above);
            // Everything in the way, as the stretches of the wall its middle
            // cannot be in. A hole is in the way of everything, going right
            // through the wall; a hanging only of what is on the same face;
            // another wall meeting this one, of both.
            let joined = self
                .joins
                .iter()
                .filter(|join| join[1] < high && join[3] > low)
                .map(|join| (join[0] - span.half - EDGE, join[2] + span.half + EDGE));
            let taken: Vec<(f32, f32)> = others
                .iter()
                .filter(|&&(other, _, other_holed)| holed || other_holed || other.face == face)
                .filter_map(|&(other, other_span, _)| {
                    let beside = other.up + other_span.above <= low
                        || other.up - other_span.below >= high;
                    let reach = span.half + other_span.half + APART;
                    (!beside).then_some((other.along - reach, other.along + reach))
                })
                .chain(joined)
                .collect();
            nearest_free(along, room, &taken).map(|along| (along, up))
        };
        self.storeys_from(near).into_iter().find_map(on_storey)
    }

    /// Where a piece standing at `on` goes, in its house's space: through the
    /// middle of the wall if it is `holed`, with its +Z side, Blender's +Y,
    /// into the house; otherwise its back on the face it hangs on, facing
    /// away from the wall.
    fn place(&self, holed: bool, on: OnWall) -> Transform {
        if holed {
            Transform::from_translation(self.point(on.along, on.up, 0.0))
                .looking_to(-self.inside, Vec3::Y)
        } else {
            let out = self.thin().1 * on.face;
            Transform::from_translation(self.point(on.along, on.up, out))
                .looking_to(self.across() * on.face, Vec3::Y)
        }
    }
}

/// The plot's shape has to be read again: something on it was put down,
/// picked up or thrown away.
#[derive(Resource, Default)]
struct Reshape(bool);

/// Where you are building just now, if anywhere: on your plot in a game of
/// House Builder while its time to build runs (`builder::Building`), an
/// island all your own; or, in the town, on your own plot of it
/// (`stand::Home`), a square of the town's land.
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Site(pub Option<Ground>);

/// Somewhere to build: the island it is on, and, where that island is not
/// all yours, the square of it that is ([`Yard`]).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Ground {
    pub venue: Venue,
    pub yard: Option<Yard>,
}

/// A square of the town's land that is someone's to build on, their plot
/// (`land::Plot`) as far as building goes: whatever is put down on it has to
/// stand inside it, the whole of it. It is also the space that what stands
/// there is kept in (`roundtown_net::bag::Standing`), so that it stands the
/// same on whichever plot its owner has ([`Yard::place`]).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Yard {
    /// Its middle, across the ground, and how high its ground is.
    pub middle: Vec2,
    pub ground: f32,
    /// Which way its front looks, across the ground: out over the side its
    /// sign stands on, a metre long.
    pub front: Vec2,
    /// How far each of its sides is from its middle, in metres.
    pub half: f32,
}

/// How far out of its yard a piece's corner may be and the piece still be
/// inside it, in metres: what a turn's rounding comes to.
const YARD_SLACK: f32 = 0.002;

impl Yard {
    /// The yard's own space, placed in the world: its origin the middle of
    /// the plot on its ground, and its -Z out over the plot's front.
    pub(crate) fn place(&self) -> Transform {
        Transform::from_translation(Vec3::new(self.middle.x, self.ground, self.middle.y))
            .looking_to(Vec3::new(self.front.x, 0.0, self.front.y), Vec3::Y)
    }

    /// The two ways its sides run, across the ground.
    fn axes(&self) -> [Vec2; 2] {
        [self.front, Vec2::new(-self.front.y, self.front.x)]
    }

    /// Whether `at`, across the ground, is on it, or within `margin` of it.
    pub(crate) fn holds(&self, at: Vec2, margin: f32) -> bool {
        let off = at - self.middle;
        self.axes().into_iter().all(|axis| off.dot(axis).abs() <= self.half + margin)
    }

    /// How far the footprint of a piece of `bounds`, turned by `turn`,
    /// reaches from the piece's own origin along each of the yard's two
    /// ways: the least and the most.
    fn reach(&self, bounds: Bounds, turn: Quat) -> [(f32, f32); 2] {
        let (low, high) = (bounds.low, bounds.high);
        let corners = [(low.x, low.z), (high.x, low.z), (high.x, high.z), (low.x, high.z)]
            .map(|(x, z)| (turn * Vec3::new(x, 0.0, z)).xz());
        self.axes().map(|axis| {
            corners
                .iter()
                .map(|corner| corner.dot(axis))
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(least, most), far| (least.min(far), most.max(far)))
        })
    }

    /// Whether the whole footprint of a piece of `bounds`, turned by `turn`
    /// and standing at `at`, is inside the yard.
    fn holds_piece(&self, at: Vec3, bounds: Bounds, turn: Quat) -> bool {
        let off = at.xz() - self.middle;
        self.axes()
            .into_iter()
            .zip(self.reach(bounds, turn))
            .all(|(axis, (least, most))| {
                let along = off.dot(axis);
                along + least >= -self.half - YARD_SLACK && along + most <= self.half + YARD_SLACK
            })
    }

    /// Where a piece of `bounds`, turned by `turn`, that would stand at
    /// `at` stands with the whole of it inside the yard: there, or as
    /// little further in as it takes. One too big for the yard stands in
    /// its middle.
    fn pull_in(&self, at: Vec3, bounds: Bounds, turn: Quat) -> Vec3 {
        let off = at.xz() - self.middle;
        let mut inside = self.middle;
        for (axis, (least, most)) in self.axes().into_iter().zip(self.reach(bounds, turn)) {
            let (from, to) = (-self.half - least, self.half - most);
            let along = if from <= to { off.dot(axis).clamp(from, to) } else { (from + to) * 0.5 };
            inside += axis * along;
        }
        Vec3::new(inside.x, at.y, inside.y)
    }
}

/// A piece asked to be put down, in the town, where it may not stand, and
/// why: not with the whole of it on your plot, in something else that
/// stands there, or, hung from a ceiling, under none. It is a ghost still,
/// with its menu open on it again (`stand` says why across the screen).
#[derive(Message, Clone, Copy)]
pub(crate) struct Misplaced {
    pub why: Misfit,
}

/// Why a piece cannot be put down where it is on a plot of the town.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Misfit {
    OffTheLand,
    InTheWay,
    /// Something hung from a ceiling, under no ceiling ([`hangs`]).
    NoCeiling,
}

/// How far over the floor under it a ceiling has to be for something to
/// hang from it, in metres: over a body's head (mine, 2026-11-25). What is
/// lower is a table's underside or a sign's board, and nothing hangs from
/// those.
const HEADROOM: f32 = 2.0;
/// How near the ceiling the top of what hangs from it is, in metres: it is
/// against it.
const AGAINST: f32 = 0.05;

/// Whether a piece on the plot `yard` can be put down where it is, standing
/// at `at` turned by `turn`: `None`, or why not. The whole of it on the
/// plot; furniture in no wall and in nothing else that stands there, by
/// their shapes on `island` ([`in_anything`]; by the boxes round them,
/// `stops`, until 2026-11-25); what hangs from a ceiling in nothing else
/// that hangs, by their boxes; a house only where a house fits
/// ([`house_fits_in`], on `ground`). What goes in a wall is put where it
/// fits as it is dragged.
#[allow(clippy::too_many_arguments)]
fn misfit(
    island: &Island,
    ground: &Island,
    yard: Yard,
    mount: Mount,
    at: Vec3,
    turn: Quat,
    bounds: Bounds,
    stops: &[Stop],
) -> Option<Misfit> {
    let in_furniture = || stops.iter().filter(|stop| stop.lines_up).any(|stop| overlapping(at, turn, bounds, stop));
    match mount {
        Mount::Door | Mount::Window | Mount::Hanging => None,
        _ if !yard.holds_piece(at, bounds, turn) => Some(Misfit::OffTheLand),
        Mount::House => house_fits_in(ground, Some(yard), bounds, turn, at.xz()).is_none().then_some(Misfit::InTheWay),
        Mount::Floor => in_anything(island, at, bounds, turn).then_some(Misfit::InTheWay),
        Mount::Ceiling if !hangs(island, at, bounds, turn) => Some(Misfit::NoCeiling),
        Mount::Ceiling => in_furniture().then_some(Misfit::InTheWay),
    }
}

/// How far over its foot something standing may have something else under
/// it, in metres: a rug, a mat, the seam of a floor. What is no higher it
/// stands over, or on.
const FOOTING: f32 = 0.05;

/// Whether a piece of `bounds`, turned by `turn` and standing at `at`, would
/// be in a wall or in anything else that stands there: by their shapes on
/// `island`, all over its footprint ([`EDGE_STEP`] apart, [`TOUCH`] in from
/// its edge), from just over its foot ([`FOOTING`]) to its own top.
///
/// **By the shapes since 2026-11-25** (Hajun: "the collision for preventing
/// the placement of objects over each other is tooo strong so even they are
/// not touching each other or the wall, the checkmark disappear and i cannot
/// place it next to each other. loosen the sensitivity of that a bit"): it
/// was by the boxes round things, and a box is bigger than what is in it,
/// a couch's round arms, a chair's legs, the empty corner of the L shaped
/// couch, so that two things that did not touch would not go together; and
/// between a step over its foot and a body's height, so that a stool would
/// not go under a table. What is on the floor inside its footprint and
/// between the points, as small as a pen, it may stand over.
fn in_anything(island: &Island, at: Vec3, bounds: Bounds, turn: Quat) -> bool {
    let margin = Vec2::splat(TOUCH);
    let low = bounds.low.xz() + margin;
    let size = (bounds.high.xz() - margin - low).max(Vec2::ZERO);
    let steps = (size / EDGE_STEP).ceil().max(Vec2::ONE).as_uvec2();
    let (bottom, top) = (at.y + FOOTING, at.y + bounds.high.y.max(2.0 * FOOTING));
    (0..=steps.x)
        .flat_map(|i| (0..=steps.y).map(move |j| (i, j)))
        .map(|(i, j)| low + size * Vec2::new(i as f32 / steps.x as f32, j as f32 / steps.y as f32))
        .map(|spot| at.xz() + (turn * Vec3::new(spot.x, 0.0, spot.y)).xz())
        .any(|spot| island.blocked(spot, bottom, top))
}

/// The middle of a piece of `bounds`, turned by `turn` and standing at
/// `at`, across the ground.
fn middle_of(at: Vec3, bounds: Bounds, turn: Quat) -> Vec2 {
    let middle = (bounds.low + bounds.high) * 0.5;
    at.xz() + (turn * Vec3::new(middle.x, 0.0, middle.z)).xz()
}

/// Whether a piece of `bounds`, turned by `turn` with its origin at `at`,
/// hangs from a ceiling there ([`Mount::Ceiling`]): over its middle, its top
/// against an underside of something on `island` ([`AGAINST`]) that is at
/// least [`HEADROOM`] over the floor under it; and none of it, round its
/// edge and at its middle, in a wall or in anything else between its foot
/// and a little under its top.
fn hangs(island: &Island, at: Vec3, bounds: Bounds, turn: Quat) -> bool {
    let (foot, top) = (at.y + bounds.low.y, at.y + bounds.high.y);
    let middle = middle_of(at, bounds, turn);
    let against = island.ceiling(middle, top - AGAINST).is_some_and(|ceiling| ceiling <= top + AGAINST);
    let room = island.floor(middle, foot).is_some_and(|floor| top - floor >= HEADROOM);
    against && room && caught_between(island, at, bounds, turn, foot + AGAINST, top - AGAINST).next().is_none()
}

/// Where the ceiling is that something hangs from over `spot`, in the house
/// whose floors are `storeys`: the lowest underside on `island` over the
/// floor there no higher than `reach`, and at least [`HEADROOM`] over it.
/// None, over no floor of a house or under open sky: nothing hangs over the
/// plot's own ground, nor from a roof terrace's open top.
fn ceiling_over(island: &Island, storeys: &[Storey], spot: Vec2, reach: f32) -> Option<f32> {
    let floor = storeys
        .iter()
        .filter(|storey| storey.under(spot))
        .map(|storey| storey.top)
        .filter(|&top| top <= reach)
        .reduce(f32::max)?;
    island.ceiling(spot, floor + HEADROOM)
}

/// Where a piece of `bounds`, turned by `turn`, hung over `spot` hangs: its
/// top against the ceiling over its middle ([`ceiling_over`], over the floor
/// no higher than `reach` there). None, with nothing over it there.
fn hang_at(island: &Island, storeys: &[Storey], spot: Vec3, bounds: Bounds, turn: Quat, reach: f32) -> Option<Vec3> {
    let ceiling = ceiling_over(island, storeys, middle_of(spot, bounds, turn), reach)?;
    Some(spot.with_y(ceiling - bounds.high.y))
}

/// Where a piece of `bounds`, turned by `turn`, that hangs from a ceiling
/// comes up out of the bag for someone standing at `you` who would have it
/// at `spot`: from the ceiling over the floor they stand on ([`hang_at`]);
/// on an upper floor of a house ([`upstairs`], since 2026-11-26), the whole
/// of it over that floor and from its ceiling, as a drag would leave it
/// ([`drag_hung`]). With no ceiling there, at `spot`, for its tick to be
/// grey.
fn hung_up(island: &Island, yard: Option<Yard>, storeys: &[Storey], you: Vec3, spot: Vec3, bounds: Bounds, turn: Quat) -> Vec3 {
    let reach = you.y + crate::LAND_STEP_UP;
    let floor = yard.map(|yard| upstairs(yard, storeys, you.xz(), reach)).unwrap_or_default();
    let (spot, under) = match floor.is_empty() {
        true => (spot, storeys),
        false => (kept_to(&floor, spot, bounds, turn), floor.as_slice()),
    };
    hang_at(island, under, spot, bounds, turn, reach).unwrap_or(spot)
}

/// Where a piece of `bounds`, turned by `turn`, hung from a ceiling at
/// `from` on your plot `yard` ends up dragged toward `to`: where the finger
/// takes it, the whole of it on the plot, its top against the ceiling
/// there; with no ceiling there, at the height it was, for its tick to be
/// grey (`mark_misfits`). Under the ceiling of the storey it is in: the
/// floor under its foot, or a step up from it, as a house's lowest floor is
/// from the ground it came out over, and never over its own top, which
/// would be the floor over the ceiling it hangs from. Over an upper floor
/// of a house, the whole of it over that floor and from its ceiling
/// ([`upstairs`], since 2026-11-26): never out over the Square House 2F's
/// stairwell, hung from whatever is over the ground floor there, the roof
/// where it comes down low, or a stair with an underside.
fn drag_hung(island: &Island, yard: Yard, storeys: &[Storey], from: Vec3, to: Vec3, bounds: Bounds, turn: Quat) -> Vec3 {
    let spot = yard.pull_in(Vec3::new(to.x, from.y, to.z), bounds, turn);
    let reach = (from.y + bounds.low.y + crate::LAND_STEP_UP).min(from.y + bounds.high.y);
    let floor = upstairs(yard, storeys, middle_of(from, bounds, turn), reach);
    let (spot, under) = match floor.is_empty() {
        true => (spot, storeys),
        false => (kept_to(&floor, spot, bounds, turn), floor.as_slice()),
    };
    hang_at(island, under, spot, bounds, turn, reach).unwrap_or(spot)
}

/// Whether a piece of `bounds`, turned by `turn` and standing at `at`, is in
/// `stop`: the two boxes overlap, across the floor and in height, by more
/// than a piece flush against another does.
fn overlapping(at: Vec3, turn: Quat, bounds: Bounds, stop: &Stop) -> bool {
    const SPARE: f32 = 0.01;
    let (bottom, top) = (at.y + bounds.low.y, at.y + bounds.high.y);
    let heights = [stop.low, stop.high].map(|corner| stop.place.transform_point3(corner).y);
    let (stop_bottom, stop_top) = (heights[0].min(heights[1]), heights[0].max(heights[1]));
    if top - SPARE <= stop_bottom || stop_top - SPARE <= bottom {
        return false;
    }
    let corners = |low: Vec3, high: Vec3| [(low.x, low.z), (high.x, low.z), (high.x, high.z), (low.x, high.z)];
    let mine = corners(bounds.low, bounds.high).map(|(x, z)| at.xz() + (turn * Vec3::new(x, 0.0, z)).xz());
    let theirs = corners(stop.low, stop.high).map(|(x, z)| stop.place.transform_point3(Vec3::new(x, 0.0, z)).xz());
    // Two boxes on the floor, each turned its own way, are apart if they
    // are apart along any of the four ways their sides run.
    let ways = [
        (turn * Vec3::X).xz(),
        (turn * Vec3::Z).xz(),
        stop.place.transform_vector3(Vec3::X).xz(),
        stop.place.transform_vector3(Vec3::Z).xz(),
    ];
    ways.into_iter().all(|way| {
        let way = way.normalize_or_zero();
        let span = |corners: &[Vec2; 4]| {
            corners
                .iter()
                .map(|corner| corner.dot(way))
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(least, most), far| (least.min(far), most.max(far)))
        };
        let (a, b) = (span(&mine), span(&theirs));
        way == Vec2::ZERO || (a.1 - SPARE > b.0 && b.1 - SPARE > a.0)
    })
}

/// A floor of a house that is down on a plot: the box it fills in the
/// house's own space, the way into that space from the world, and how high
/// its top is in the world.
#[derive(Clone, Copy)]
struct Storey {
    into: Affine3A,
    low: Vec3,
    high: Vec3,
    top: f32,
}

impl Storey {
    /// Whether `spot`, across the ground, is over it.
    fn under(&self, spot: Vec2) -> bool {
        let at = self.into.transform_point3(Vec3::new(spot.x, 0.0, spot.y));
        at.x >= self.low.x && at.x <= self.high.x && at.z >= self.low.z && at.z <= self.high.z
    }

    /// Where a piece of `bounds`, turned by `turn`, that would stand at `at`
    /// stands with the whole of its footprint over this floor: there, or as
    /// little further in as it takes, along each of the house's own two
    /// ways; one too big for it, in its middle that way. As high as `at`.
    fn pull_in(&self, at: Vec3, bounds: Bounds, turn: Quat) -> Vec3 {
        let (low, high) = (bounds.low, bounds.high);
        let corners = [(low.x, low.z), (high.x, low.z), (high.x, high.z), (low.x, high.z)]
            .map(|(x, z)| self.into.transform_vector3(turn * Vec3::new(x, 0.0, z)));
        let here = self.into.transform_point3(at);
        let inside = |axis: usize| {
            let (least, most) = corners
                .iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(least, most), corner| (least.min(corner[axis]), most.max(corner[axis])));
            let (from, to) = (self.low[axis] - least, self.high[axis] - most);
            if from <= to { here[axis].clamp(from, to) } else { (from + to) * 0.5 }
        };
        self.into.inverse().transform_point3(Vec3::new(inside(0), here.y, inside(2))).with_y(at.y)
    }
}

/// How near in height two boxes of a house's floors are to be one floor, in
/// metres: what an L-shaped floor modelled as two comes to.
const ONE_FLOOR: f32 = 0.01;

/// The upper floor of a house that something at `spot`, on a floor no
/// higher than `reach`, stands on: the floor under it ([`floor_under`]),
/// where that is more than a step over the plot's own ground, as every
/// floor over a house's ground floor is; each box of it, at its height in
/// that house. None on the land or a house's ground floor.
///
/// **Nothing dragged comes down off an upper floor** (since 2026-11-26;
/// Hajun: "im on second floor, and im moving a furniture like stove around
/// the floor, and as it goes near the corner/edge near the wall, it falls
/// down to the ground. so i need to be so careful so it doesnt fall. fix
/// this."): such a floor's box ends at the inner faces of the walls, and in
/// the Square House 2F at the edge of the stairwell, and a thing taken past
/// it as far as the point it stands by (the stove's is near its middle)
/// stood on whatever was under that, the ground floor or the land, which it
/// could not be dragged back up from. Now the whole of it is kept over its
/// floor ([`kept_to`]), as the whole of it is on the plot. What hangs from
/// a ceiling is kept over its floor too, and a small thing goes up onto the
/// tops of only what stands on its floor (`edit_pieces`).
fn upstairs(yard: Yard, storeys: &[Storey], spot: Vec2, reach: f32) -> Vec<Storey> {
    let Some(floor) = storeys
        .iter()
        .filter(|storey| storey.top <= reach && storey.under(spot))
        .max_by(|a, b| a.top.total_cmp(&b.top))
    else {
        return Vec::new();
    };
    if floor.top - yard.ground <= crate::LAND_STEP_UP {
        return Vec::new();
    }
    storeys
        .iter()
        .filter(|storey| storey.into == floor.into && (storey.top - floor.top).abs() < ONE_FLOOR)
        .copied()
        .collect()
}

/// Whether two floors, as [`upstairs`] has them, are one: the one upper
/// floor of the one house, or neither, the land and a ground floor being
/// one.
fn one_floor(a: &[Storey], b: &[Storey]) -> bool {
    match (a.first(), b.first()) {
        (Some(a), Some(b)) => a.into == b.into && (a.top - b.top).abs() < ONE_FLOOR,
        (None, None) => true,
        _ => false,
    }
}

/// Where a piece of `bounds`, turned by `turn`, that would stand at `at`
/// stands kept to the upper floor `floor` ([`upstairs`]): the whole of its
/// footprint over the nearest box of it ([`Storey::pull_in`]). As high as
/// `at`.
fn kept_to(floor: &[Storey], at: Vec3, bounds: Bounds, turn: Quat) -> Vec3 {
    floor
        .iter()
        .map(|storey| storey.pull_in(at, bounds, turn))
        .min_by(|a, b| a.xz().distance_squared(at.xz()).total_cmp(&b.xz().distance_squared(at.xz())))
        .unwrap_or(at)
}

/// Every house that is down or being put down, with its floors: where it
/// stands, its floors, where it is and whether it is a ghost.
type Houses<'w, 's> = Query<'w, 's, (&'static GlobalTransform, &'static Rooms, Option<&'static Venue>, Has<Ghost>)>;

/// The floors of the houses down on `venue`, from where each stands in the
/// world: for what comes up out of the bag before anything is moved
/// ([`place_fresh`]).
fn storeys_in(houses: &Houses, venue: Venue) -> Vec<Storey> {
    houses
        .iter()
        .filter(|&(_, _, on, ghost)| !ghost && on == Some(&venue))
        .flat_map(|(place, rooms, ..)| {
            let (into, lift) = (place.affine().inverse(), place.translation().y);
            rooms.0.iter().map(move |&(low, high)| Storey {
                into,
                low,
                high,
                top: lift + high.y,
            })
        })
        .collect()
}

/// The floors of the houses down on `venue`.
fn storeys_on(pieces: &Moving, rooms: &Query<&Rooms>, venue: Venue) -> Vec<Storey> {
    pieces
        .iter()
        .filter(|&(.., on, ghost)| !ghost && on == Some(&venue))
        .filter_map(|(house, _, place, ..)| Some((place, rooms.get(house).ok()?)))
        .flat_map(|(place, rooms)| {
            let (into, lift) = (place.compute_affine().inverse(), place.translation.y);
            rooms.0.iter().map(move |&(low, high)| Storey {
                into,
                low,
                high,
                top: lift + high.y,
            })
        })
        .collect()
}

/// How far up a face has to point to be a top a small thing can stand on:
/// the cosine of its slope, 25 degrees at most.
const TOP_UP: f32 = 0.9;

/// Where a small thing of `bounds` stands on the top of whatever of yours
/// the finger's `ray` meets first, among the meshes `tops`: on the point it
/// meets, if that faces up ([`onto`]); or, met on its side, as a finger
/// over the front edge of a table meets it from the camera's low angle, on
/// its top straight down from a little way in from there. None, over
/// nothing of yours, where it goes on the floor as anything does.
fn top_under(ray_cast: &mut MeshRayCast, ray: Ray3d, tops: &HashSet<Entity>, bounds: Bounds) -> Option<Vec3> {
    if tops.is_empty() {
        return None;
    }
    let within = |entity: Entity| tops.contains(&entity);
    let settings = MeshRayCastSettings::default().with_filter(&within);
    let hit = ray_cast.cast_ray(ray, &settings).first().map(|(_, hit)| (hit.point, hit.normal))?;
    if let Some(at) = onto(hit.0, hit.1, bounds) {
        return Some(at);
    }
    const IN: f32 = 0.08;
    const OVER: f32 = 3.0;
    let inside = hit.0 + ray.direction * IN;
    let down = Ray3d::new(inside.with_y(hit.0.y + OVER), Dir3::NEG_Y);
    let (_, top) = ray_cast.cast_ray(down, &settings).first()?;
    onto(top.point, top.normal, bounds)
}

/// Where a thing of `bounds` stands on a surface met at `point`, facing
/// `normal`: its foot there, if the surface is a top ([`TOP_UP`]).
fn onto(point: Vec3, normal: Vec3, bounds: Bounds) -> Option<Vec3> {
    (normal.normalize_or_zero().y >= TOP_UP).then(|| point - Vec3::Y * bounds.low.y)
}

/// The floor under `spot` on the plot `yard`: the highest of the plot's own
/// ground and the floors of the house there, `storeys`, that is no higher
/// than `reach`. Not whatever else stands there: a thing dragged over a bed
/// goes through it, and not up onto it.
fn floor_under(yard: Yard, storeys: &[Storey], spot: Vec2, reach: f32) -> f32 {
    storeys
        .iter()
        .filter(|storey| storey.under(spot))
        .map(|storey| storey.top)
        .filter(|&top| top <= reach)
        .fold(yard.ground, f32::max)
}

/// Where a piece of `bounds`, turned by `turn`, dragged from `from` toward
/// `goal` on your plot of the town, `yard`, ends up (since 2026-11-22;
/// Hajun: "it seems like moving it around is a puzzle cuz u cant move it
/// through other objects due to collision, but i think it should be freely
/// able to move around even through other objects but it will just grey out
/// the check mark if its colliding/cant be applied"): where the finger
/// asks, the whole of it on the plot, on the floor that is under it there
/// ([`floor_under`]: the land, or a floor of the house no more than a step
/// up from where it is), through whatever else stands in between; and then
/// snapped into line with whatever among `stops` it has come up to, if it
/// is `clear` to stand there. On an upper floor of a house, on that floor,
/// the whole of it over it, however near the walls or past them the finger
/// goes ([`upstairs`], since 2026-11-26): it slides along them.
#[allow(clippy::too_many_arguments)]
fn drag_freely(
    yard: Yard,
    from: Vec3,
    goal: Vec2,
    bounds: Bounds,
    turn: Quat,
    stops: &[Stop],
    storeys: &[Storey],
    clear: impl Fn(Vec3) -> bool,
) -> Vec3 {
    let reach = from.y + crate::LAND_STEP_UP;
    let upstairs = upstairs(yard, storeys, from.xz(), reach);
    let on_its_floor = |spot: Vec3| match upstairs.first() {
        Some(floor) => kept_to(&upstairs, spot, bounds, turn).with_y(floor.top),
        None => spot.with_y(floor_under(yard, storeys, spot.xz(), reach)),
    };
    let at = on_its_floor(yard.pull_in(Vec3::new(goal.x, from.y, goal.y), bounds, turn));
    snaps(at, turn, bounds, stops)
        .into_iter()
        .map(on_its_floor)
        .find(|&spot| clear(spot))
        .unwrap_or(at)
}

/// Everything that puts pieces down and keeps the plots' shapes, after the
/// edit menu has said what was asked ([`editor::EditSystems`]).
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BuildSystems;

pub(crate) fn plugin(app: &mut App) {
    // The town's houses, with doors and windows put in them as here.
    town::plugin(app);
    app.init_resource::<Reshape>()
        .init_resource::<Site>()
        .init_resource::<NextPieceId>()
        .init_resource::<Untold>()
        .add_message::<Misplaced>()
        .add_observer(took)
        .add_observer(rebuilt)
        .add_observer(unbuilt)
        .add_systems(Startup, stock_up)
        .add_systems(Update, untappable_from_inside.before(editor::EditSystems))
        .add_systems(Update, swing_doors.after(crate::move_bodies))
        .add_systems(
            Update,
            (
                find_site,
                take_chosen,
                place_fresh,
                edit_pieces,
                mark_misfits,
                finish_building,
                report_pieces,
                raise_shadows,
                carve_walls,
                reshape_plot,
                want_pointer,
            )
                .chain()
                .in_set(BuildSystems)
                .after(editor::EditSystems),
        );
    // What stands on the town's plots, yours and everyone's (`stand`).
    stand::plugin(app);
    paint::plugin(app);
}

/// Says where you are building just now ([`Site`]): in a game of House
/// Builder, on your plot there while its time to build runs; otherwise, in
/// the town, on your own plot of it.
fn find_site(building: Res<Building>, home: Res<stand::Home>, mut site: ResMut<Site>) {
    let now = match building.0 {
        Some(plot) => Some(Ground {
            venue: Venue::Plot(plot),
            yard: None,
        }),
        None => home.yard.map(|yard| Ground {
            venue: Venue::Town,
            yard: Some(yard),
        }),
    };
    site.set_if_neq(Site(now));
}

fn stock_up(assets: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(Stock {
        scenes: PIECES
            .iter()
            .map(|def| assets.load(GltfAssetLabel::Scene(0).from_asset(def.model())))
            .collect(),
        pictures: PIECES
            .iter()
            .map(|def| assets.load(def.picture()))
            .collect(),
        ghosts: HashMap::new(),
    });
}

// ----------------------------------------------------------- coming and going

/// Puts what was chosen in the shop on your plot, as a ghost with the edit
/// menu open on it. Where exactly waits for its model ([`place_fresh`]).
fn take_chosen(
    mut chosen: MessageReader<Chosen>,
    building: Res<Building>,
    stock: Res<Stock>,
    me: Query<&Transform, With<Player>>,
    // A game's own: what stands on your plot of the town is not this
    // game's house (`stand`).
    pieces: Query<(Entity, &Piece), (Without<Shadow>, With<InGame>)>,
    mut menu: ResMut<EditMenu>,
    mut commands: Commands,
) {
    let Some(plot) = building.0 else {
        chosen.clear();
        return;
    };
    for &Chosen(def) in chosen.read() {
        let house = pieces
            .iter()
            .find(|(_, piece)| piece.def.mount == Mount::House)
            .map(|(house, _)| house);
        let at = me.single().map(|me| me.translation).unwrap_or_default();
        bring_out(&mut commands, &stock, &mut menu, def, house, Venue::Plot(plot), at, InGame);
    }
}

/// A piece of `def` just chosen, out of the shop or out of your bag: a
/// ghost with the edit menu open on it, and `tag` on it. On `venue`, at
/// `at` until its model has loaded and it is put where it comes up
/// ([`place_fresh`]); or, for what goes in or on a wall, in `house`, and
/// with no house, nothing.
#[allow(clippy::too_many_arguments)]
fn bring_out(
    commands: &mut Commands,
    stock: &Stock,
    menu: &mut EditMenu,
    def: &'static PieceDef,
    house: Option<Entity>,
    venue: Venue,
    at: Vec3,
    tag: impl Bundle,
) -> Option<Entity> {
    if def.on_wall() && house.is_none() {
        return None;
    }
    let mut piece = commands.spawn((
        Piece { def },
        Ghost,
        Fresh,
        tag,
        WorldAssetRoot(stock.scene(def)),
        // Until its size is known, a box a metre across.
        Editable {
            touch: Touch::Block {
                low: Vec3::splat(-0.5),
                high: Vec3::splat(0.5),
            },
            ring: 0.0,
            turns: !def.on_wall(),
            sticky: true,
        },
        Visibility::Inherited,
    ));
    match house.filter(|_| def.on_wall()) {
        // Its house carries it, and says where it is drawn.
        Some(house) => {
            piece.insert((ChildOf(house), Transform::default()));
        }
        None => {
            piece.insert((venue, Transform::from_translation(at)));
        }
    }
    piece.observe(piece_ready);
    let piece = piece.id();
    menu.open_on(piece);
    Some(piece)
}

/// Once a piece's model has spawned: how big it is, which is what a tap on it
/// is tested against, a house's walls made again to have holes cut in them,
/// a door's swinging leaves, and a ghost house's see-through look.
fn piece_ready(
    ready: On<WorldInstanceReady>,
    pieces: Query<(&Piece, Option<&Editable>, Has<Ghost>, Has<paint::Colours>)>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    parts: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>, Option<&GltfMaterialName>)>,
    turns: Query<&Transform>,
    placement: TransformHelper,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stock: ResMut<Stock>,
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    let root = ready.entity;
    let Ok((piece, editable, ghost, painted)) = pieces.get(root) else {
        return;
    };
    let is_house = piece.def.mount == Mount::House;
    // A mesh belongs to what its node is called in Blender.
    let called = |part: Entity, prefix: &str| {
        [Some(part), parents.get(part).ok().map(ChildOf::parent)]
            .into_iter()
            .flatten()
            .any(|entity| {
                names
                    .get(entity)
                    .is_ok_and(|name| name.as_str().starts_with(prefix))
            })
    };

    let grow = |sum: Option<(Vec3, Vec3)>, low: Vec3, high: Vec3| {
        Some(sum.map_or((low, high), |(a, b): (Vec3, Vec3)| {
            (a.min(low), b.max(high))
        }))
    };
    let mut bounds: Option<(Vec3, Vec3)> = None;
    let mut frame: Option<(Vec3, Vec3)> = None;
    // The box each mesh of it fills.
    let mut filled = HashMap::new();
    let mut walls = Vec::new();
    let mut floors = Vec::new();
    // Whatever else is built into a house, like its staircase: no door,
    // window or hanging goes where one stands against a wall.
    let mut fixtures = Vec::new();
    // On a plot of the town, what can be painted (`paint`): each mesh but a
    // house's walls, with the material it is drawn with, what that is
    // called, from how high to how high it runs and what it is; named once
    // all of a house's floors are known.
    let mut paintable = Vec::new();
    for part in children.iter_descendants(root) {
        let Ok((mesh, material, named)) = parts.get(part) else {
            continue;
        };
        let paint = material.zip(named).filter(|_| painted).map(|(material, named)| (material.0.clone(), named.0.clone()));
        // In the piece's own space, by how each node of it stands in the
        // one above ([`within`]): the same to the last bit wherever the
        // piece itself stands, which a house's walls are numbered by.
        let Some((low, high)) = meshes
            .get(&mesh.0)
            .zip(within(part, root, &parents, &turns))
            .and_then(|(mesh, place)| extent(mesh, place))
        else {
            continue;
        };
        filled.insert(part, (low, high));
        bounds = grow(bounds, low, high);
        if called(part, "Frame") {
            frame = grow(frame, low, high);
        }
        if is_house && called(part, "Wall") {
            walls.push((part, low, high, material.map(|material| material.0.clone()), paint));
            continue;
        }
        let floor = is_house && called(part, "Floor");
        if let Some((plain, named)) = paint {
            let shape = if floor { paint::Shape::Floor } else { paint::Shape::Other };
            paintable.push((part, plain, named, low.y, high.y, shape));
        }
        if floor {
            floors.push((low, high));
        } else if is_house && !called(part, "Roof") && !called(part, "Gable") {
            fixtures.push((low, high));
        }
        if ghost && is_house && let Some(material) = material {
            commands.entity(part).insert((
                Solid(material.0.clone()),
                MeshMaterial3d(stock.ghost_of(&material.0, &mut materials)),
                NotShadowCaster,
            ));
        }
    }

    // The parts to paint, named: in a house a material on a floor, and each
    // of its walls a band to each floor it runs up (`paint::parts`); the
    // walls' after the rest, in the order they were found.
    let drawn: Vec<paint::Drawn> = paintable
        .iter()
        .map(|(_, _, named, foot, top, shape)| (named.as_str(), *foot, *top, *shape))
        .chain(walls.iter().filter_map(|(_, low, high, _, paint)| {
            paint.as_ref().map(|(_, named)| (named.as_str(), low.y, high.y, paint::Shape::Wall))
        }))
        .collect();
    let mut named = paint::parts(is_house, &floors, &drawn).into_iter();
    for (part, plain, ..) in paintable {
        if let Some((_, name)) = named.next().and_then(|bands| bands.into_iter().next()) {
            commands.entity(part).insert(paint::Part::new(&name, plain));
        }
    }

    // Each wall again, as a box of the same size that holes can be cut in, in
    // the house's own space.
    let against: Vec<(Vec3, Vec3)> = walls
        .iter()
        .map(|(_, low, high, ..)| (*low, *high))
        .chain(fixtures)
        .collect();
    for (part, low, high, material, paint) in walls {
        commands.entity(part).despawn();
        let mut wall = Wall::new(low, high, &floors, &against);
        // Painted, a band to each floor it runs up, cut at the top of each
        // floor it runs past ([`Banded`]); otherwise the whole of it.
        let bands = match paint {
            Some(_) => named.next().unwrap_or_default(),
            None => Vec::new(),
        };
        let cuts: Vec<f32> = bands.iter().skip(1).map(|&(bottom, _)| bottom).collect();
        let mut shapes = wall_meshes(&wall, &cuts).into_iter().map(|mesh| meshes.add(mesh));
        let Some(mesh) = shapes.next() else {
            continue;
        };
        let upper: Vec<Handle<Mesh>> = shapes.collect();
        wall.mesh = mesh.clone();
        let material = material.unwrap_or_default();
        let made = commands.spawn((ChildOf(root), wall, Mesh3d(mesh), Transform::IDENTITY)).id();
        // What draws it: the wall itself, and a band under it for each
        // floor it runs up past.
        let mut drawers = vec![made];
        if !cuts.is_empty() {
            for mesh in &upper {
                drawers.push(commands.spawn((ChildOf(made), Band, Mesh3d(mesh.clone()), Transform::IDENTITY)).id());
            }
            commands.entity(made).insert(Banded { cuts, upper });
        }
        for (band, drawer) in drawers.into_iter().enumerate() {
            let mut drawer = commands.entity(drawer);
            // What is made in its place is the part, or the parts, the wall
            // was.
            if let (Some((plain, _)), Some((_, name))) = (&paint, bands.get(band)) {
                drawer.insert(paint::Part::new(name, plain.clone()));
            }
            if ghost {
                drawer.insert((
                    MeshMaterial3d(stock.ghost_of(&material, &mut materials)),
                    Solid(material.clone()),
                    NotShadowCaster,
                ));
            } else {
                drawer.insert(MeshMaterial3d(material.clone()));
            }
        }
    }
    if is_house {
        commands.entity(root).insert(Rooms(floors));
    }
    // A door's leaves that swing.
    if let Some(swing) = piece.def.swing
        && let Some(doorway) = doorway(root, swing, &filled, &children, &parents, &names, &turns, &placement)
    {
        commands.entity(root).insert(doorway);
    }

    if let Some((low, high)) = bounds {
        commands.entity(root).insert(Bounds { low, high });
        // Someone else's piece, from the server, is never edited here.
        if let Some(editable) = editable {
            commands.entity(root).insert(Editable {
                touch: Touch::Block { low, high },
                ..*editable
            });
        }
        // What it takes up of a wall: a door's or a window's frame, which is
        // the hole it needs, or failing a frame, the whole of it.
        if piece.def.on_wall() {
            let (low, high) = frame.filter(|_| piece.def.holed()).unwrap_or((low, high));
            commands.entity(root).insert(Span::of(low, high));
        }
    }
    if !ghost {
        reshape.0 = true;
    }
}

/// A door's leaves that swing, once its model has spawned under `root`: the
/// parts called what `swing` says, each turned as it hangs shut, and opening
/// about its hinge whichever way takes it out of the house, -Z here. `filled`
/// is the box each of the door's meshes fills, in its own space. `None` if
/// it has none of them.
#[allow(clippy::too_many_arguments)]
fn doorway(
    root: Entity,
    swing: Swing,
    filled: &HashMap<Entity, (Vec3, Vec3)>,
    children: &Query<&Children>,
    parents: &Query<&ChildOf>,
    names: &Query<&Name>,
    turns: &Query<&Transform>,
    placement: &TransformHelper,
) -> Option<Doorway> {
    let root_place = placement.compute_global_transform(root).ok()?;
    let into_root = root_place.affine().inverse();
    let mut leaves = Vec::new();
    let mut reach: f32 = 0.0;
    for &name in swing.leaves {
        let Some(part) = children
            .iter_descendants(root)
            .find(|&part| names.get(part).is_ok_and(|called| called.as_str() == name))
        else {
            continue;
        };
        // Its hinge, and the box its own meshes fill, here.
        let hinge = placement
            .compute_global_transform(part)
            .map(|place| into_root.transform_point3(place.translation()));
        let own = children
            .get(part)
            .iter()
            .flat_map(|own| own.iter())
            .filter_map(|mesh| filled.get(&mesh))
            .fold(None, |sum: Option<(Vec3, Vec3)>, &(low, high)| {
                Some(sum.map_or((low, high), |(a, b)| (a.min(low), b.max(high))))
            });
        // It turns in the space it hangs in, its parent's.
        let hung_in = parents
            .get(part)
            .ok()
            .and_then(|parent| placement.compute_global_transform(parent.parent()).ok())
            .map(|parent| parent.affine().inverse() * root_place.affine());
        let (Ok(hinge), Some((low, high)), Some(hung_in), Ok(hung)) =
            (hinge, own, hung_in, turns.get(part))
        else {
            continue;
        };
        let middle = (low + high) * 0.5;
        let far = hung_in.transform_vector3((middle - hinge).with_y(0.0));
        let out = hung_in.transform_vector3(Vec3::NEG_Z);
        leaves.push(Leaf {
            part,
            shut: hung.rotation,
            open: outward(far, out) * swing.open_deg.to_radians(),
        });
        reach = reach.max(middle.x.abs());
    }
    (!leaves.is_empty()).then_some(Doorway {
        leaves,
        reach,
        through: 0.0,
        wanted: false,
    })
}

/// Where `part` is in the space of `root`, which it is under: how each node
/// from it up to `root` stands in the one above, one after another.
///
/// Until 2026-11-22 this was worked out through the world, the part's
/// place there in the root's place there, which comes out a few millionths
/// of a metre off, by a different few wherever the root stands and
/// whichever way it is turned. Three walls of Hajun's houses begin at the
/// same x, and which of them counted as the first, by which a door or a
/// window in one is kept ([`walls_of`]), hung on those millionths: so a
/// door stood in one wall on the device that had put it there, and in
/// another on everyone else's, who met the house already where it stands
/// (Hajun: "the house rotation is different for another person like the
/// door was on the wrong wall on my friend's screen, but in my screen he
/// jumped through my window").
fn within(part: Entity, root: Entity, parents: &Query<&ChildOf>, turns: &Query<&Transform>) -> Option<Affine3A> {
    let mut place = Affine3A::IDENTITY;
    let mut at = part;
    while at != root {
        place = turns.get(at).ok()?.compute_affine() * place;
        at = parents.get(at).ok()?.parent();
    }
    Some(place)
}

/// The box round every corner of `mesh`, placed by `place`.
fn extent(mesh: &Mesh, place: Affine3A) -> Option<(Vec3, Vec3)> {
    let Some(VertexAttributeValues::Float32x3(corners)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return None;
    };
    corners
        .iter()
        .map(|&corner| place.transform_point3(Vec3::from(corner)))
        .map(|corner| (corner, corner))
        .reduce(|(low, high), (a, b)| (low.min(a), high.max(b)))
}

/// Puts a piece just out of the shop where you can see it, once how big it is
/// is known: in front of you, facing you, or on the wall of the house you are
/// looking at.
fn place_fresh(
    mut fresh: Query<
        (
            Entity,
            &Piece,
            &Bounds,
            Option<&Span>,
            &mut Transform,
            Option<&ChildOf>,
        ),
        (With<Fresh>, Without<Player>, Without<ThirdPersonCamera>),
    >,
    me: Query<&Transform, (With<Player>, Without<ThirdPersonCamera>)>,
    eye: Query<&Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    homes: Query<&GlobalTransform>,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    hung: Hung,
    houses: Houses,
    site: Res<Site>,
    islands: Res<Islands>,
    mut commands: Commands,
) {
    let (Some(Ground { venue, yard }), Ok(me), Ok(eye)) = (site.0, me.single(), eye.single()) else {
        return;
    };
    let ahead = crate::camera_ahead(eye);
    for (piece, kind, bounds, span, mut transform, house) in &mut fresh {
        commands.entity(piece).remove::<Fresh>();
        match kind.def.mount {
            Mount::Floor | Mount::House => {
                // Its front, Blender's -Y, toward you, and its near side a
                // step out of your way; a house's, a good few steps.
                let turn = facing(-ahead, yard);
                let gap = if kind.def.mount == Mount::House {
                    HOUSE_IN_FRONT
                } else {
                    IN_FRONT
                };
                let spot = me.translation + ahead * (PLAYER_RADIUS + gap - bounds.low.z);
                // On your plot of the town, nowhere but on it.
                let spot = yard.map_or(spot, |yard| yard.pull_in(spot, *bounds, turn));
                let at = if kind.def.mount == Mount::House {
                    islands
                        .ground(venue)
                        .and_then(|ground| house_near(ground, yard, *bounds, turn, spot.xz()))
                        .unwrap_or(spot)
                } else {
                    islands.get(venue).map_or(spot, |island| {
                        comes_up(island, yard, &storeys_in(&houses, venue), me.translation, spot, *bounds, turn)
                    })
                };
                *transform = Transform::from_translation(at).with_rotation(turn);
            }
            // A step out in front of you, its front toward you, hung from
            // the ceiling over the floor you stand on; with none there,
            // where you stand, for its tick to be grey.
            Mount::Ceiling => {
                let turn = facing(-ahead, yard);
                let spot = me.translation + ahead * (PLAYER_RADIUS + IN_FRONT - bounds.low.z);
                let spot = yard.map_or(spot, |yard| yard.pull_in(spot, *bounds, turn));
                let at = islands.get(venue).map_or(spot, |island| {
                    hung_up(island, yard, &storeys_in(&houses, venue), me.translation, spot, *bounds, turn)
                });
                *transform = Transform::from_translation(at).with_rotation(turn);
            }
            Mount::Door | Mount::Window | Mount::Hanging => {
                let (Some(house), Some(&span)) = (house.map(ChildOf::parent), span) else {
                    continue;
                };
                let Ok(home) = homes.get(house) else {
                    continue;
                };
                // Everything in the house's own space, and from your own
                // eyes: the camera can be looking in through a doorway, or
                // down at the floor.
                let into_house = home.affine().inverse();
                let from = into_house.transform_point3(me.translation + Vec3::Y * EYE_HEIGHT);
                let way = into_house.transform_vector3(ahead);
                let you = into_house.transform_point3(me.translation);
                // The wall you are facing first, then the others, nearest you
                // first.
                let mut choices: Vec<(f32, Entity, &Wall, Vec3)> = walls
                    .iter()
                    .filter(|(.., parent)| parent.parent() == house)
                    .map(
                        |(entity, wall, _)| match ray_box(from, way, wall.low, wall.high) {
                            Some(far) => (far, entity, wall, from + way * far),
                            None => {
                                let near = you.clamp(wall.low, wall.high);
                                (1e4 + near.distance(you), entity, wall, near)
                            }
                        },
                    )
                    .collect();
                choices.sort_by(|a, b| a.0.total_cmp(&b.0));
                let holed = kind.def.holed();
                // On the storey you are standing on, if there is room there:
                // a door on its floor, a window a little way up it, a hanging
                // half way.
                let share = if kind.def.mount == Mount::Window {
                    WINDOW_SILL
                } else {
                    0.5
                };
                let height = Height::Share { share, near: you.y };
                // On your storey of the wall you face, or failing that of the
                // next nearest wall, and only then on another storey: the
                // wall you face can have no room on yours, where the stairs
                // go up it.
                let found = [true, false].into_iter().find_map(|yours| {
                    choices.iter().find_map(|&(_, entity, wall, aim)| {
                        let face = if wall.out_of(from) >= 0.0 { 1.0 } else { -1.0 };
                        let others = others_on(&hung, entity, piece);
                        let mount = kind.def.mount;
                        wall.fit(mount, span, height, wall.along_of(aim), face, &others)
                            .filter(|&(_, up)| !yours || wall.storey(up) == wall.storey(you.y))
                            .map(|(along, up)| {
                                let on = OnWall {
                                    wall: entity,
                                    along,
                                    up,
                                    face,
                                };
                                (wall.place(holed, on), on)
                            })
                    })
                });
                match found {
                    Some((place, on)) => {
                        *transform = place;
                        commands.entity(piece).insert(on);
                    }
                    // Every wall is full.
                    None => commands.entity(piece).despawn(),
                }
            }
        }
    }
}

/// Everything in or on a wall of a house.
type Hung<'w, 's> = Query<'w, 's, (Entity, &'static OnWall, &'static Span, &'static Piece)>;

/// Everything else in or on `wall`, besides `piece`: where each is, what it
/// takes up, and whether it is a hole.
fn others_on(hung: &Hung, wall: Entity, piece: Entity) -> Vec<(OnWall, Span, bool)> {
    hung.iter()
        .filter(|&(other, on, ..)| other != piece && on.wall == wall)
        .map(|(_, &on, &span, other)| (on, span, other.def.holed()))
        .collect()
}

/// Where a piece of `bounds`, turned by `turn`, comes up out of the bag for
/// someone standing at `you` who would have it at `spot`: [`ahead_of`] them;
/// and on an upper floor of a house in `storeys` ([`upstairs`], since
/// 2026-11-26), on nothing more than a step under that floor, never down
/// the stairwell in front of them, and the whole of it over the floor, as a
/// drag would leave it.
fn comes_up(island: &Island, yard: Option<Yard>, storeys: &[Storey], you: Vec3, spot: Vec3, bounds: Bounds, turn: Quat) -> Vec3 {
    let floor = yard
        .map(|yard| upstairs(yard, storeys, you.xz(), you.y + crate::LAND_STEP_UP))
        .unwrap_or_default();
    let lowest = floor.first().map(|storey| storey.top - crate::LAND_STEP_UP);
    let at = ahead_of(island, yard, you, spot, bounds, turn, lowest);
    match floor.first() {
        Some(storey) => kept_to(&floor, at, bounds, turn).with_y(at.y.max(storey.top)),
        None => at,
    }
}

/// Where a piece of `bounds`, turned by `turn`, comes up for someone standing
/// at `you` who would have it at `spot`: carried from them toward there as far
/// as the floor goes and no walls are in the way, then back toward them until
/// the whole of it is clear of the walls too — facing a wall, in the room
/// rather than through it. In a `yard`, the whole of it inside the yard
/// besides: as near where it would have come up as that leaves it. On
/// nothing lower than `lowest`, if that is said ([`comes_up`]).
fn ahead_of(island: &Island, yard: Option<Yard>, you: Vec3, spot: Vec3, bounds: Bounds, turn: Quat, lowest: Option<f32>) -> Vec3 {
    let point = Bounds {
        low: Vec3::ZERO,
        high: Vec3::ZERO,
    };
    let reached = editor::carry(island, you, spot.xz(), turn, |at| {
        !walled(island, at, point, turn)
    });
    let inside = |at: Vec3| yard.is_none_or(|yard| yard.holds_piece(at, bounds, turn));
    const TRIES: u32 = 12;
    (0..=TRIES)
        .map(|back| reached.xz().lerp(you.xz(), back as f32 / TRIES as f32))
        .filter_map(|at| {
            island
                .floor(at, you.y + crate::LAND_STEP_UP)
                .filter(|&y| lowest.is_none_or(|lowest| y >= lowest))
                .map(|y| Vec3::new(at.x, y, at.y))
        })
        .find(|&at| inside(at) && !walled(island, at, bounds, turn))
        .unwrap_or_else(|| yard.map_or(reached, |yard| yard.pull_in(reached, bounds, turn)))
}

/// Whether a piece of `bounds`, turned by `turn` and standing at `at`, would
/// be in a wall, or in anything else it could not be carried onto: whatever
/// fills the heights between a step over its feet and a body's height, all
/// round the edge of its footprint, [`TOUCH`] in from it, and at its middle
/// — what keeps a body out of the island. Tested at exactly those points
/// rather than on the lattice bodies are, so that a piece can stand flush
/// against a wall, or another piece.
fn walled(island: &Island, at: Vec3, bounds: Bounds, turn: Quat) -> bool {
    caught(island, at, bounds, turn).next().is_some()
}

/// Where round the edge of a piece of `bounds`, turned by `turn` and standing
/// at `at`, and at its middle, it would be in a wall or in anything else it
/// could not be carried onto, as [`walled`] tests it, on the ground.
fn caught(island: &Island, at: Vec3, bounds: Bounds, turn: Quat) -> impl Iterator<Item = Vec2> {
    caught_between(island, at, bounds, turn, at.y + crate::LAND_STEP_UP, at.y + crate::PLAYER_HEIGHT)
}

/// The same, between the heights `bottom` and `top`: for what hangs from a
/// ceiling, from its foot to just under the ceiling ([`hangs`]).
fn caught_between(island: &Island, at: Vec3, bounds: Bounds, turn: Quat, bottom: f32, top: f32) -> impl Iterator<Item = Vec2> {
    let margin = Vec2::splat(TOUCH);
    let (low, high) = (bounds.low.xz() + margin, bounds.high.xz() - margin);
    let size = high - low;
    let steps = (size / EDGE_STEP).ceil().max(Vec2::ONE);
    let across = (0..=steps.x as u32).flat_map(move |i| {
        let x = low.x + size.x * i as f32 / steps.x;
        [Vec2::new(x, low.y), Vec2::new(x, high.y)]
    });
    let along = (0..=steps.y as u32).flat_map(move |j| {
        let z = low.y + size.y * j as f32 / steps.y;
        [Vec2::new(low.x, z), Vec2::new(high.x, z)]
    });
    across
        .chain(along)
        .chain(std::iter::once((low + high) * 0.5))
        .map(move |edge| at.xz() + (turn * Vec3::new(edge.x, 0.0, edge.y)).xz())
        .filter(move |&spot| island.blocked(spot, bottom, top))
}

/// Where a piece of `bounds`, standing at `at` on `island`, stands once it is
/// turned to `turn`, so that it always turns (Hajun, 2026-10-09: up against a
/// wall or another piece, it would not turn at all). Where it is, if it is
/// clear there; otherwise as near as it is clear, out of whatever it would be
/// in, over the same floor and never through a wall, and snapped into line
/// among `stops` as a drag would leave it. Where it is, in whatever it would
/// be in, if there is nowhere near enough. In a `yard`, it is clear only
/// with the whole of it inside the yard.
fn turned_clear(island: &Island, yard: Option<Yard>, at: Vec3, bounds: Bounds, turn: Quat, stops: &[Stop]) -> Vec3 {
    let fits = |spot: Vec3| yard.is_none_or(|yard| yard.holds_piece(spot, bounds, turn)) && !walled(island, spot, bounds, turn);
    if fits(at) {
        return at;
    }
    // Turned out over the edge of its yard, and into nothing else: in.
    if let Some(yard) = yard
        && !walled(island, at, bounds, turn)
    {
        let pulled = yard.pull_in(at, bounds, turn);
        if fits(pulled) {
            return pulled;
        }
    }
    // Every way round, the way out of what it would be in first: away from
    // the middle of where it meets it.
    let (sum, count) = caught(island, at, bounds, turn)
        .fold((Vec2::ZERO, 0.0), |(sum, count), spot| (sum + spot, count + 1.0));
    let away = (at.xz() - sum / count).normalize_or_zero();
    let mut ways: Vec<Vec2> = (0..ROOM_WAYS)
        .map(|way| Vec2::from_angle(way as f32 * std::f32::consts::TAU / ROOM_WAYS as f32))
        .collect();
    ways.sort_by(|a, b| b.dot(away).total_cmp(&a.dot(away)));
    // `far` out `way`, on the floor it stands on.
    let out = |way: Vec2, far: f32| {
        let spot = at.xz() + way * far;
        editor::drop_height(island, spot, at.y)
            .filter(|y| (y - at.y).abs() <= crate::LAND_STEP_UP)
            .map(|y| Vec3::new(spot.x, y, spot.y))
    };
    // As little way out `way` as it takes, from somewhere it is clear `far`
    // out, found to within a millimetre by halving.
    let least = |way: Vec2, far: f32| {
        let (mut short, mut long) = (far - ROOM_STEP, far);
        for _ in 0..7 {
            let middle = (short + long) * 0.5;
            if out(way, middle).is_some_and(fits) {
                long = middle;
            } else {
                short = middle;
            }
        }
        long
    };
    // The nearest it is clear: of every way it is clear at the first step
    // out it is clear at all, the one it moves least to be.
    let reach = bounds.low.xz().abs().max(bounds.high.xz().abs()).length() + ROOM_SPARE;
    let found = (1..=(reach / ROOM_STEP).ceil() as u32).find_map(|step| {
        let far = step as f32 * ROOM_STEP;
        ways.iter()
            .filter(|&&way| {
                out(way, far).is_some_and(|spot| fits(spot) && open_between(island, at, spot))
            })
            .map(|&way| (way, least(way, far)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    });
    let Some((way, far)) = found else {
        return at;
    };
    let spot = out(way, far).unwrap_or(at);
    snaps(spot, turn, bounds, stops)
        .into_iter()
        .filter_map(|snap| editor::drop_height(island, snap.xz(), spot.y).map(|y| snap.with_y(y)))
        .find(|&snap| fits(snap))
        .unwrap_or(spot)
}

/// Whether the middle of a piece goes from `from` to `to` without going
/// through anything it could not be carried onto: never through a wall.
fn open_between(island: &Island, from: Vec3, to: Vec3) -> bool {
    let (bottom, top) = (from.y + crate::LAND_STEP_UP, from.y + crate::PLAYER_HEIGHT);
    let steps = (from.xz().distance(to.xz()) / EDGE_STEP).ceil().max(1.0);
    (1..=steps as u32)
        .map(|step| from.xz().lerp(to.xz(), step as f32 / steps))
        .all(|spot| !island.blocked(spot, bottom, top))
}

/// Something a piece carried over the floor can be pushed flush against:
/// another piece down on its plot, or a wall of the house, as the box it
/// fills in its own space and where that stands.
#[derive(Clone, Copy)]
struct Stop {
    place: Affine3A,
    low: Vec3,
    high: Vec3,
    /// Whether a piece flush against it lines up with its ends or its middle,
    /// as with another piece. A wall's ends are the corners of its room,
    /// which a piece gets into by going flush against the other wall too.
    lines_up: bool,
}

/// What a piece carried over the floor of `venue` can be pushed flush
/// against: every other piece down there that stands on the floor, and the
/// walls of the house there, if it is down. A small thing, the walls alone:
/// it goes up onto the rest ([`PieceDef::small`]), and is in none of them.
fn stops_on(
    pieces: &Moving,
    walls: &Query<(Entity, &Wall, &ChildOf)>,
    venue: Venue,
    carried: Entity,
) -> Vec<Stop> {
    let down = |on: Option<&Venue>, ghost: bool| !ghost && on == Some(&venue);
    // What hangs from a ceiling goes against what else does; anything else,
    // against what stands on a floor.
    let of = pieces.get(carried).ok().map(|(_, piece, ..)| piece.def);
    let hung = of.is_some_and(|def| def.mount == Mount::Ceiling);
    let small = of.is_some_and(|def| def.small);
    let like = if hung { Mount::Ceiling } else { Mount::Floor };
    let furniture = pieces
        .iter()
        .filter(|&(entity, piece, .., on, ghost)| {
            !small && entity != carried && piece.def.mount == like && down(on, ghost)
        })
        .filter_map(|(_, _, place, _, bounds, ..)| {
            let bounds = bounds?;
            Some(Stop {
                place: place.compute_affine(),
                low: bounds.low,
                high: bounds.high,
                lines_up: true,
            })
        });
    let house = walls.iter().filter_map(|(_, wall, house)| {
        let (_, _, place, .., on, ghost) = pieces.get(house.parent()).ok()?;
        down(on, ghost).then(|| Stop {
            place: place.compute_affine(),
            low: wall.low,
            high: wall.high,
            lines_up: false,
        })
    });
    furniture.chain(house).collect()
}

/// The box round a footprint on the floor, as a piece turned some way sees
/// it: from its lowest corner to its highest, x across the piece and y from
/// its front to its back.
#[derive(Clone, Copy, Debug)]
struct Patch {
    low: Vec2,
    high: Vec2,
}

impl Patch {
    fn middle(self) -> Vec2 {
        (self.low + self.high) * 0.5
    }

    fn moved(self, by: Vec2) -> Self {
        Self {
            low: self.low + by,
            high: self.high + by,
        }
    }

    /// How far it is beside `other` on the axis `axis`: how much the two
    /// overlap along it.
    fn beside(self, other: Self, axis: usize) -> f32 {
        self.high[axis].min(other.high[axis]) - self.low[axis].max(other.low[axis])
    }

    /// How far it has to go on the axis `axis` to be flush against `other`, on
    /// whichever side of it it is.
    fn to_flush(self, other: Self, axis: usize) -> f32 {
        if self.middle()[axis] > other.middle()[axis] {
            other.high[axis] - self.low[axis]
        } else {
            other.low[axis] - self.high[axis]
        }
    }
}

/// Where a piece of `bounds`, turned by `turn` and carried to `at`, would
/// snap to among `stops`, best first: flush against whatever it is square to
/// and within [`SNAP`] of, on either axis, and then lined up with an end, or
/// the middle, of another piece it is flush against, the nearest of them if
/// that is within [`SNAP`] too; failing that, only flush. Nothing, with
/// nothing near enough. Whether it is clear to stand there is for the caller
/// to test.
fn snaps(at: Vec3, turn: Quat, bounds: Bounds, stops: &[Stop]) -> Vec<Vec3> {
    // Nearer than these to square on, or to flush, it is; and it is beside
    // something only if it is beside it by more than it can be pressed into
    // it, which is [`TOUCH`].
    const SQUARE: f32 = 1e-3;
    const FLUSH: f32 = 2.0 * TOUCH;
    // Everything as the piece sees it, turned with it, so that what it is
    // square to is square on.
    let into = turn.inverse();
    let flat = |point: Vec3| (into * point).xz();
    let here = Patch {
        low: flat(at) + bounds.low.xz(),
        high: flat(at) + bounds.high.xz(),
    };
    // Only what it could not be carried onto, as [`walled`] tells it.
    let (bottom, top) = (at.y + crate::LAND_STEP_UP, at.y + crate::PLAYER_HEIGHT);
    let near: Vec<(Patch, bool)> = stops
        .iter()
        .filter(|stop| {
            stop.place.transform_point3(stop.high).y > bottom
                && stop.place.transform_point3(stop.low).y < top
        })
        .filter(|stop| {
            let across = flat(stop.place.transform_vector3(Vec3::X));
            across.x.abs().min(across.y.abs()) < SQUARE
        })
        .map(|stop| {
            let (low, high) = (stop.low, stop.high);
            let corners = [(low.x, low.z), (high.x, low.z), (low.x, high.z), (high.x, high.z)]
                .map(|(x, z)| flat(stop.place.transform_point3(Vec3::new(x, 0.0, z))));
            let patch = Patch {
                low: corners.into_iter().reduce(Vec2::min).unwrap_or_default(),
                high: corners.into_iter().reduce(Vec2::max).unwrap_or_default(),
            };
            (patch, stop.lines_up)
        })
        .collect();
    let nearest = |a: &f32, b: &f32| a.abs().total_cmp(&b.abs());
    // Flush against whatever it is beside, across and along.
    let flush = [0, 1].map(|axis| {
        near.iter()
            .filter(|(stop, _)| here.beside(*stop, 1 - axis) > FLUSH)
            .map(|(stop, _)| here.to_flush(*stop, axis))
            .filter(|by| by.abs() <= SNAP)
            .min_by(nearest)
    });
    let pushed = Vec2::new(flush[0].unwrap_or(0.0), flush[1].unwrap_or(0.0));
    let there = here.moved(pushed);
    // Then, on an axis it is not flush on, in line with another piece it is
    // flush against on the other.
    let lined = Vec2::from_array([0, 1].map(|axis| {
        let along = flush[axis].is_none().then(|| {
            near.iter()
                .filter(|&&(stop, lines_up)| {
                    lines_up
                        && there.beside(stop, axis) > FLUSH
                        && there.to_flush(stop, 1 - axis).abs() < FLUSH
                })
                .flat_map(|(stop, _)| {
                    [
                        stop.low[axis] - there.low[axis],
                        stop.high[axis] - there.high[axis],
                        stop.middle()[axis] - there.middle()[axis],
                    ]
                })
                .filter(|by| by.abs() <= SNAP)
                .min_by(nearest)
        });
        pushed[axis] + along.flatten().unwrap_or(0.0)
    }));
    let moved = |by: Vec2| at + turn * Vec3::new(by.x, 0.0, by.y);
    let mut spots = Vec::new();
    if lined != pushed {
        spots.push(moved(lined));
    }
    if flush.iter().any(Option::is_some) {
        spots.push(moved(pushed));
    }
    spots
}

/// Where a piece of `bounds`, turned by `turn`, dragged from `from` toward
/// `goal` over `island`, ends up: carried as far toward there as it goes, and
/// then snapped into line with whatever among `stops` it has come up to, if
/// it is clear to stand there. In a `yard`, it goes no further than the
/// whole of it is inside the yard.
fn drag_over_floor(
    island: &Island,
    yard: Option<Yard>,
    from: Vec3,
    goal: Vec2,
    bounds: Bounds,
    turn: Quat,
    stops: &[Stop],
) -> Vec3 {
    let fits = |at| yard.is_none_or(|yard| yard.holds_piece(at, bounds, turn)) && !walled(island, at, bounds, turn);
    let carried = editor::carry(island, from, goal, turn, fits);
    snaps(carried, turn, bounds, stops)
        .into_iter()
        .filter_map(|spot| {
            editor::drop_height(island, spot.xz(), carried.y).map(|y| spot.with_y(y))
        })
        .find(|&spot| fits(spot))
        .unwrap_or(carried)
}

/// The turn that faces a piece's front, Blender's -Y, along `way`, to the
/// nearest [`TURN_STEP_DEG`]: counted from the front of the plot it comes
/// up on, `yard`, so that its sides run along the plot's however the plot
/// is turned in the town (Hajun, 2026-11-25, of plot 43: "the edges of the
/// item liek for example the house edges or the couch's back need to be
/// able to be parallel to the edge of that plot"); and with none, as in
/// House Builder, from the world's axes. The turn button adds its 45° to
/// this, as it always has.
pub(crate) fn facing(way: Vec3, yard: Option<Yard>) -> Quat {
    let square = yard.map_or(Quat::IDENTITY, |yard| yard.place().rotation);
    let way = square.inverse() * way;
    let step = TURN_STEP_DEG.to_radians();
    let yaw = (-way.x).atan2(-way.z);
    square * Quat::from_rotation_y((yaw / step).round() * step)
}

/// Where near `spot` a house of `bounds`, turned by `turn`, fits: there, or
/// failing that as little further in toward the middle of the plot as it
/// takes: the middle of the island, or of the `yard`, where it fits only as
/// [`house_fits_in`] has it.
fn house_near(ground: &Island, yard: Option<Yard>, bounds: Bounds, turn: Quat, spot: Vec2) -> Option<Vec3> {
    let middle = yard.map_or(Vec2::ZERO, |yard| yard.middle);
    let inward = (middle - spot).normalize_or_zero();
    let reach = middle.distance(spot);
    (0..)
        .map(|step| step as f32 * 0.5)
        .take_while(|&back| back <= reach)
        .map(|back| spot + inward * back)
        .find_map(|at| house_fits_in(ground, yard, bounds, turn, at).map(|y| Vec3::new(at.x, y, at.y)))
}

/// Where a house of `bounds`, turned by `turn`, dragged from `from` toward
/// `goal`, gets to: as far along the way as the whole of it still fits on the
/// land, in short steps, sliding along the shore where it cannot go straight
/// on. One that does not fit where it is follows the finger freely until it
/// does.
fn house_toward(ground: &Island, yard: Option<Yard>, bounds: Bounds, turn: Quat, from: Vec3, goal: Vec2) -> Vec3 {
    let fit = |at: Vec2| house_fits_in(ground, yard, bounds, turn, at).map(|y| Vec3::new(at.x, y, at.y));
    if fit(from.xz()).is_none() {
        return fit(goal).unwrap_or(from);
    }
    let path = (goal - from.xz()).clamp_length_max(HOUSE_REACH);
    let steps = (path.length() / HOUSE_STEP).ceil().max(1.0);
    let step = path / steps;
    let mut at = from;
    for _ in 0..steps as u32 {
        let slides = [step, Vec2::new(step.x, 0.0), Vec2::new(0.0, step.y)];
        let Some(next) = slides.into_iter().find_map(|slide| fit(at.xz() + slide)) else {
            break;
        };
        at = next;
    }
    at
}

/// The height a house of `bounds`, turned by `turn`, stands at on `at`, as
/// [`house_fits`] has it; and in a `yard`, your plot of the town, only with
/// the whole of it inside the yard and nothing else standing where it
/// would: the plot's sign, or anything put down there already, which are
/// part of the town's shape as `ground` has it (`Islands::lay_layer`) and
/// are felt for all over its footprint ([`HOUSE_FEEL`]).
fn house_fits_in(ground: &Island, yard: Option<Yard>, bounds: Bounds, turn: Quat, at: Vec2) -> Option<f32> {
    let base = house_fits(ground, bounds, turn, at)?;
    let Some(yard) = yard else {
        return Some(base);
    };
    if !yard.holds_piece(Vec3::new(at.x, base, at.y), bounds, turn) {
        return None;
    }
    let (low, high) = (bounds.low.xz(), bounds.high.xz());
    let size = high - low;
    let steps = (size / HOUSE_FEEL).ceil().max(Vec2::ONE);
    let (bottom, top) = (base + crate::LAND_STEP_UP, base + crate::PLAYER_HEIGHT);
    let clear = (0..=steps.x as u32).all(|i| {
        (0..=steps.y as u32).all(|j| {
            let spot = low + size * Vec2::new(i as f32 / steps.x, j as f32 / steps.y);
            let under = at + (turn * Vec3::new(spot.x, 0.0, spot.y)).xz();
            !ground.blocked(under, bottom, top)
        })
    });
    clear.then_some(base)
}

/// How far apart the spots under a house are at which the ground is felt
/// for anything standing there already, on a plot of the town, in metres:
/// nearer than a chair is wide but for a hair, and than a sign is long.
const HOUSE_FEEL: f32 = 0.75;

/// The height a house of `bounds`, turned by `turn`, stands at on `at`, if the
/// land under the whole of it is dry and level enough.
fn house_fits(ground: &Island, bounds: Bounds, turn: Quat, at: Vec2) -> Option<f32> {
    let base = ground.floor(at, f32::INFINITY)?;
    if base <= WATER_Y + 0.25 {
        return None;
    }
    let (low, high) = (bounds.low, bounds.high);
    let middle = (low + high) * 0.5;
    let footprint = [
        (low.x, low.z),
        (high.x, low.z),
        (high.x, high.z),
        (low.x, high.z),
        (middle.x, low.z),
        (middle.x, high.z),
        (low.x, middle.z),
        (high.x, middle.z),
    ];
    footprint
        .into_iter()
        .all(|(x, z)| {
            let under = at + (turn * Vec3::new(x, 0.0, z)).xz();
            ground
                .floor(under, f32::INFINITY)
                .is_some_and(|y| (y - base).abs() <= HOUSE_TILT)
        })
        .then_some(base)
}

/// How far along the ray from `origin` going `way` it first meets the box from
/// `low` to `high`, if it does.
fn ray_box(origin: Vec3, way: Vec3, low: Vec3, high: Vec3) -> Option<f32> {
    let (mut near, mut far) = (f32::NEG_INFINITY, f32::INFINITY);
    for axis in 0..3 {
        let (from, step) = (origin[axis], way[axis]);
        if step.abs() < 1e-8 {
            if from < low[axis] || from > high[axis] {
                return None;
            }
            continue;
        }
        let (a, b) = ((low[axis] - from) / step, (high[axis] - from) / step);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far && far >= 0.0).then_some(near.max(0.0))
}

// ---------------------------------------------------------------- the menu

/// Everything that moves a piece.
type Moving<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Piece,
        &'static mut Transform,
        Option<&'static ChildOf>,
        Option<&'static Bounds>,
        Option<&'static Span>,
        Option<&'static Grip>,
        Option<&'static Venue>,
        Has<Ghost>,
    ),
>;

/// Does what the edit menu asks of a piece: moves it, turns it, puts it down,
/// throws it away, or, tapped once it is down, picks it back up.
///
/// On your plot of the town ([`Yard`]) nothing is moved or turned to where
/// any of it would be off the plot; and asked to be put down where it may
/// not stand, a piece stays a ghost, its menu open on it again
/// ([`Misplaced`]). **There a thing goes where the finger takes it, through
/// whatever else stands there** ([`drag_freely`], since 2026-11-22), and
/// turns where it is, where on a plot of House Builder it is carried up to
/// what is in its way and no further, as it always was; whether it can be
/// put down where it is, is said on it for the menu's tick to show
/// ([`mark_misfits`]). And there a tap somewhere else lets go of it
/// (`Edit::Away`): put down where it is if it can stand there, and put back
/// where it stood if not (`stand::PutBack`).
#[allow(clippy::too_many_arguments)]
fn edit_pieces(
    mut edits: MessageReader<Edit>,
    site: Res<Site>,
    islands: Res<Islands>,
    mut stock: ResMut<Stock>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pieces: Moving,
    homes: Query<&GlobalTransform>,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    hung: Hung,
    children: Query<&Children>,
    looks: Query<(&MeshMaterial3d<StandardMaterial>, Has<Solid>)>,
    (solids, rooms): (Query<&Solid>, Query<&Rooms>),
    (mut reshape, mut menu, mut misplaced, mut put_back): (
        ResMut<Reshape>,
        ResMut<EditMenu>,
        MessageWriter<Misplaced>,
        MessageWriter<stand::PutBack>,
    ),
    mut ray_cast: MeshRayCast,
    mut commands: Commands,
) {
    let Some(Ground { venue, yard }) = site.0 else {
        edits.clear();
        return;
    };
    for &edit in edits.read() {
        match edit {
            Edit::Opened(target) => {
                if let Ok((_, piece, .., false)) = pieces.get(target) {
                    if piece.def.mount == Mount::House {
                        ghost(target, &children, &looks, &mut stock, &mut materials, &mut commands);
                    }
                    commands.entity(target).insert(Ghost);
                    reshape.0 = true;
                }
            }
            Edit::Drag { target, to, ray } => {
                // What a piece carried over the floor can go flush against,
                // found before it is taken hold of to be moved.
                let stops = pieces
                    .get(target)
                    .is_ok_and(|(_, piece, ..)| matches!(piece.def.mount, Mount::Floor | Mount::Ceiling))
                    .then(|| stops_on(&pieces, &walls, venue, target))
                    .unwrap_or_default();
                // On your plot, the floors of the house there: what a thing
                // dragged freely stands on, and what hangs from a ceiling
                // hangs over.
                let storeys = yard.map(|_| storeys_on(&pieces, &rooms, venue)).unwrap_or_default();
                // And for a small thing, the meshes of what else stands
                // there on a floor, whose tops it may go up onto: what
                // stands on the floor it is on (the land and a ground floor
                // being one), never on another, as a finger over an upper
                // floor meets a table under it, through that floor, and over
                // a house's roof the things upstairs in it (since 2026-11-26).
                let floor_of = |at: Vec3| {
                    yard.map(|yard| upstairs(yard, &storeys, at.xz(), at.y + crate::LAND_STEP_UP))
                        .unwrap_or_default()
                };
                let tops: HashSet<Entity> = pieces
                    .get(target)
                    .ok()
                    .filter(|(_, piece, ..)| piece.def.small)
                    .map(|(_, _, own, ..)| {
                        let its_floor = floor_of(own.translation);
                        pieces
                            .iter()
                            .filter(|&(entity, piece, place, .., on, ghost)| {
                                entity != target
                                    && !ghost
                                    && on == Some(&venue)
                                    && piece.def.mount == Mount::Floor
                                    && yard.is_some_and(|yard| yard.holds(place.translation.xz(), 1.0))
                                    && one_floor(&its_floor, &floor_of(place.translation))
                            })
                            .flat_map(|(entity, ..)| children.iter_descendants(entity))
                            .collect()
                    })
                    .unwrap_or_default();
                let Ok((_, piece, mut transform, house, bounds, span, grip, ..)) =
                    pieces.get_mut(target)
                else {
                    continue;
                };
                match piece.def.mount {
                    Mount::Floor => {
                        let (Some(island), Some(&bounds)) = (islands.get(venue), bounds) else {
                            continue;
                        };
                        let (from, turn) = (transform.translation, transform.rotation);
                        // A small thing, on your plot, onto the top of what
                        // the finger is over, the whole of it on the plot.
                        let top = yard
                            .filter(|_| piece.def.small)
                            .and_then(|yard| top_under(&mut ray_cast, ray, &tops, bounds).filter(|&at| yard.holds_piece(at, bounds, turn)));
                        let at = match (top, to, yard) {
                            (Some(top), ..) => top,
                            (None, Some(to), Some(yard)) => {
                                // A small thing off a top goes where the
                                // finger meets the floor under it, not the
                                // level it was taken hold of at, which was the
                                // top's.
                                let goal = piece
                                    .def
                                    .small
                                    .then(|| {
                                        let floor = floor_under(yard, &storeys, from.xz(), from.y + crate::LAND_STEP_UP);
                                        let far = ray.intersect_plane(Vec3::Y * floor, InfinitePlane3d::new(Vec3::Y))?;
                                        Some(ray.get_point(far).xz())
                                    })
                                    .flatten()
                                    .unwrap_or(to.xz());
                                let clear = |spot| misfit(island, island, yard, Mount::Floor, spot, turn, bounds, &stops).is_none();
                                drag_freely(yard, from, goal, bounds, turn, &stops, &storeys, clear)
                            }
                            (None, Some(to), None) => drag_over_floor(island, yard, from, to.xz(), bounds, turn, &stops),
                            (None, None, _) => continue,
                        };
                        if transform.translation != at {
                            transform.translation = at;
                        }
                    }
                    // Where the finger takes it, its top against the
                    // ceiling there ([`drag_hung`]).
                    Mount::Ceiling => {
                        let (Some(to), Some(island), Some(&bounds), Some(yard)) = (to, islands.get(venue), bounds, yard) else {
                            continue;
                        };
                        let (from, turn) = (transform.translation, transform.rotation);
                        let at = drag_hung(island, yard, &storeys, from, to, bounds, turn);
                        if transform.translation != at {
                            transform.translation = at;
                        }
                    }
                    Mount::House => {
                        let (Some(to), Some(ground), Some(&bounds)) =
                            (to, islands.ground(venue), bounds)
                        else {
                            continue;
                        };
                        let at = match yard {
                            // On your plot it goes where the finger does,
                            // the whole of it on the plot, whatever stands
                            // there.
                            Some(yard) => yard.pull_in(Vec3::new(to.x, yard.ground, to.z), bounds, transform.rotation),
                            None => house_toward(
                                ground,
                                yard,
                                bounds,
                                transform.rotation,
                                transform.translation,
                                to.xz(),
                            ),
                        };
                        if transform.translation != at {
                            transform.translation = at;
                        }
                    }
                    Mount::Door | Mount::Window | Mount::Hanging => {
                        let (Some(house), Some(&span)) = (house.map(ChildOf::parent), span) else {
                            continue;
                        };
                        let Ok(home) = homes.get(house) else {
                            continue;
                        };
                        let into_house = home.affine().inverse();
                        let from = into_house.transform_point3(ray.origin);
                        let way = into_house.transform_vector3(*ray.direction);
                        // Whichever wall of the house the finger is over; or,
                        // off every wall, over its top, past its end or down
                        // on the floor, the one it is on, where the finger is
                        // across the plane of it, so that it keeps on after
                        // the finger up to the edge rather than stopping dead.
                        let under = walls
                            .iter()
                            .filter(|(.., parent)| parent.parent() == house)
                            .filter_map(|(entity, wall, _)| {
                                ray_box(from, way, wall.low, wall.high)
                                    .map(|far| (far, entity, wall))
                            })
                            .min_by(|a, b| a.0.total_cmp(&b.0))
                            .or_else(|| {
                                let (_, on, ..) = hung.get(target).ok()?;
                                let (entity, wall, _) = walls.get(on.wall).ok()?;
                                Some((wall.across_plane(from, way)?, entity, wall))
                            });
                        let Some((far, entity, wall)) = under else {
                            continue;
                        };
                        let aim = from + way * far;
                        let face = if wall.out_of(from) >= 0.0 { 1.0 } else { -1.0 };
                        let holed = piece.def.holed();
                        // The point pressed on it stays under the finger:
                        // where that was on it is taken as the drag starts.
                        let grip = grip.copied().unwrap_or_else(|| {
                            let grip = hung.get(target).map_or(Grip::default(), |(_, on, ..)| {
                                Grip {
                                    along: if on.wall == entity {
                                        on.along - wall.along_of(aim)
                                    } else {
                                        0.0
                                    },
                                    up: on.up - aim.y,
                                }
                            });
                            commands.entity(target).insert(grip);
                            grip
                        });
                        // On the storey the finger is on, if there is room
                        // there: a door on its floor, a window at its sill,
                        // a hanging where the finger is.
                        let others = others_on(&hung, entity, target);
                        let up = aim.y + grip.up;
                        let height = if piece.def.mount == Mount::Window {
                            Height::Share {
                                share: WINDOW_SILL,
                                near: up + (span.above - span.below) * 0.5,
                            }
                        } else {
                            Height::At(up)
                        };
                        let Some((along, up)) = wall.fit(
                            piece.def.mount,
                            span,
                            height,
                            wall.along_of(aim) + grip.along,
                            face,
                            &others,
                        ) else {
                            continue;
                        };
                        let on = OnWall {
                            wall: entity,
                            along,
                            up,
                            face,
                        };
                        let place = wall.place(holed, on);
                        if *transform != place {
                            *transform = place;
                        }
                        commands.entity(target).insert(on);
                    }
                }
            }
            Edit::Turn(target, angle) => {
                // What it can go flush against, as it can when dragged.
                let stops = pieces
                    .get(target)
                    .is_ok_and(|(_, piece, ..)| piece.def.mount == Mount::Floor)
                    .then(|| stops_on(&pieces, &walls, venue, target))
                    .unwrap_or_default();
                let Ok((_, piece, mut transform, _, bounds, ..)) = pieces.get_mut(target) else {
                    continue;
                };
                let turned = Quat::from_rotation_y(angle) * transform.rotation;
                let at = transform.translation;
                match (piece.def.mount, bounds, yard) {
                    // On your plot it turns where it is, the whole of it
                    // kept on the plot, into whatever stands there.
                    (Mount::House | Mount::Floor | Mount::Ceiling, Some(&bounds), Some(yard)) => {
                        transform.rotation = turned;
                        transform.translation = yard.pull_in(at, bounds, turned);
                    }
                    // A house turns only where it still fits on the land.
                    (Mount::House, Some(&bounds), None) => {
                        let fits = islands.ground(venue).is_some_and(|ground| {
                            house_fits_in(ground, yard, bounds, turned, at.xz()).is_some()
                        });
                        if fits {
                            transform.rotation = turned;
                        }
                    }
                    // Anything else always turns, out of a wall or another
                    // piece it would turn into.
                    (Mount::Floor, Some(&bounds), None) => {
                        if let Some(island) = islands.get(venue) {
                            transform.translation = turned_clear(island, yard, at, bounds, turned, &stops);
                        }
                        transform.rotation = turned;
                    }
                    (Mount::House | Mount::Floor | Mount::Ceiling, None, _) => transform.rotation = turned,
                    _ => {}
                }
            }
            Edit::Confirm(target) | Edit::Away(target) => {
                // Let go of by a tap somewhere else: only what stands on a
                // plot of the town. A game's piece waits for its tick.
                let away = matches!(edit, Edit::Away(_));
                if away && yard.is_none() {
                    continue;
                }
                let Ok((_, piece, transform, _, bounds, .., ghost)) = pieces.get(target) else {
                    continue;
                };
                if away && !ghost {
                    continue;
                }
                // On your plot of the town, only where it can stand: the
                // whole of it on the plot, in nothing else that stands
                // there, and a house only where a house fits.
                let why = yard.zip(bounds).and_then(|(yard, &bounds)| {
                    let (island, ground) = (islands.get(venue)?, islands.ground(venue)?);
                    let stops = stops_on(&pieces, &walls, venue, target);
                    let (at, turn) = (transform.translation, transform.rotation);
                    misfit(island, ground, yard, piece.def.mount, at, turn, bounds, &stops)
                });
                match why {
                    // By its tick: it stays in your hand, and why is said.
                    Some(why) if !away => {
                        menu.open_on(target);
                        misplaced.write(Misplaced { why });
                        continue;
                    }
                    // Let go of where it cannot stand, or before its model
                    // has said how big it is: back where it stood.
                    _ if away && (why.is_some() || bounds.is_none()) => {
                        put_back.write(stand::PutBack(target));
                        continue;
                    }
                    _ => {}
                }
                if away {
                    menu.let_go_of(target);
                }
                unghost(target, &children, &solids, &mut commands);
                commands.entity(target).remove::<(Ghost, Fresh, CannotStand)>();
                reshape.0 = true;
            }
            Edit::Trash(target) => {
                if pieces.contains(target) {
                    commands.entity(target).despawn();
                    reshape.0 = true;
                }
            }
            Edit::Dropped(target) => {
                if pieces.contains(target) {
                    commands.entity(target).remove::<Grip>();
                }
            }
        }
    }
}

/// Says of what is being put down on a plot of the town whether it can be
/// put down where it is (`editor::CannotStand`, which greys the menu's tick
/// on it): each thing in your hand, as it is moved or turned, as its model
/// says how big it is, and as what stands round it changes. Not what is
/// carried along in a house (`stand::Rides`), which is put down with it.
#[allow(clippy::type_complexity)]
fn mark_misfits(
    site: Res<Site>,
    islands: Res<Islands>,
    pieces: Moving,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    (marked, riders): (Query<(), With<CannotStand>>, Query<(), With<stand::Rides>>),
    mut tried: Local<HashMap<Entity, (Vec3, Quat, bool)>>,
    mut commands: Commands,
) {
    let Some(Ground { venue, yard: Some(yard) }) = site.0 else {
        tried.clear();
        return;
    };
    let in_hand = |&(entity, .., ghost): &(Entity, &Piece, &Transform, Option<&ChildOf>, Option<&Bounds>, Option<&Span>, Option<&Grip>, Option<&Venue>, bool)| {
        ghost && !riders.contains(entity)
    };
    if !pieces.iter().any(|item| in_hand(&item)) {
        tried.clear();
        return;
    }
    let (Some(island), Some(ground)) = (islands.get(venue), islands.ground(venue)) else {
        return;
    };
    let all = islands.is_changed();
    let mut now = HashMap::new();
    for item in pieces.iter().filter(in_hand) {
        let (entity, piece, place, _, bounds, ..) = item;
        let as_it_is = (place.translation, place.rotation, bounds.is_some());
        now.insert(entity, as_it_is);
        if !all && tried.get(&entity) == Some(&as_it_is) {
            continue;
        }
        let cannot = bounds.is_some_and(|&bounds| {
            let stops = stops_on(&pieces, &walls, venue, entity);
            misfit(island, ground, yard, piece.def.mount, place.translation, place.rotation, bounds, &stops).is_some()
        });
        match (cannot, marked.contains(entity)) {
            (true, false) => {
                commands.entity(entity).try_insert(CannotStand);
            }
            (false, true) => {
                commands.entity(entity).try_remove::<CannotStand>();
            }
            _ => {}
        }
    }
    *tried = now;
}

/// Makes everything drawn under `root` see-through, keeping what each was for
/// when it is put down.
fn ghost(
    root: Entity,
    children: &Query<&Children>,
    looks: &Query<(&MeshMaterial3d<StandardMaterial>, Has<Solid>)>,
    stock: &mut Stock,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    for part in std::iter::once(root).chain(children.iter_descendants(root)) {
        if let Ok((material, false)) = looks.get(part) {
            commands.entity(part).insert((
                Solid(material.0.clone()),
                MeshMaterial3d(stock.ghost_of(&material.0, materials)),
                NotShadowCaster,
            ));
        }
    }
}

/// Gives everything drawn under `root` back what it was before it was a ghost.
fn unghost(
    root: Entity,
    children: &Query<&Children>,
    solids: &Query<&Solid>,
    commands: &mut Commands,
) {
    for part in std::iter::once(root).chain(children.iter_descendants(root)) {
        if let Ok(solid) = solids.get(part) {
            commands
                .entity(part)
                .insert(MeshMaterial3d(solid.0.clone()))
                .remove::<(Solid, NotShadowCaster)>();
        }
    }
}

/// When the time to build runs out: what is still a ghost is put down where
/// it is, and nothing on the plot can be changed any more.
fn finish_building(
    building: Res<Building>,
    // A game's own pieces: what stands on your plot of the town (`stand`)
    // is yours to change whenever you are there.
    pieces: Query<(Entity, Has<Ghost>), (With<Piece>, With<Editable>, With<InGame>)>,
    children: Query<&Children>,
    solids: Query<&Solid>,
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    if building.0.is_some() {
        return;
    }
    for (piece, ghost) in &pieces {
        if ghost {
            unghost(piece, &children, &solids, &mut commands);
            commands.entity(piece).remove::<(Ghost, Fresh)>();
            reshape.0 = true;
        }
        commands.entity(piece).remove::<Editable>();
    }
}

/// A house you are standing in cannot be tapped. From inside, nearly every tap
/// lands on it — a wall, the floor, the ceiling — and picked back up it is a
/// ghost, no longer there to stand in or bump into. What is in there with you,
/// and the doors, windows and hangings in its walls, still can be. It is
/// also marked as the house you are in ([`YouAreIn`]). A town house is the
/// one you are in on Blend In's venue too, which is the town as it stands
/// (`island::scenery`, since 2026-11-18), and is see-through from inside
/// there as in the town (`see_through`).
///
/// **Nor can a house be tapped from your car** (since 2026-11-22, when your
/// own house came to stand on your plot of the town, `stand`): driving past
/// it, a stray tap would pick it up. Whatever else of yours stands there is
/// out of a tap's reach then too (`stand::hands_off`). **Nor is your own
/// house on your plot tapped but in edit mode, with the hand** (since
/// 2026-11-22, `customize`; Hajun: "with edit mode off, tapping objects
/// should do nothing"), as nothing else of yours there is.
fn untappable_from_inside(
    me: Query<(&Transform, &Venue), With<Player>>,
    driving: Res<crate::drive::Driving>,
    customize: Res<crate::customize::Customize>,
    houses: Query<(
        Entity,
        &Venue,
        &GlobalTransform,
        &Rooms,
        &Bounds,
        Has<Untappable>,
        Has<YouAreIn>,
        Has<stand::Mine>,
    )>,
    mut commands: Commands,
) {
    let me = me.single().ok();
    let driving = driving.on();
    for (house, venue, place, rooms, bounds, untappable, in_it, mine) in &houses {
        let inside = me.is_some_and(|(me, &on)| {
            let at = place.affine().inverse().transform_point3(me.translation);
            (on == *venue || island::scenery(on) == *venue) && rooms.hold(at, bounds.high.y)
        });
        let out_of_reach = inside || driving || (mine && !customize.moving());
        if out_of_reach && !untappable {
            commands.entity(house).insert(Untappable);
        } else if !out_of_reach && untappable {
            commands.entity(house).remove::<Untappable>();
        }
        if inside && !in_it {
            commands.entity(house).insert(YouAreIn);
        } else if !inside && in_it {
            commands.entity(house).remove::<YouAreIn>();
        }
    }
}

/// On desktop the cursor is let go for as long as something is being put
/// down, to drag it with and to press its buttons; and for as long as what
/// stands on your plot is being changed (`customize`), to tap it with.
fn want_pointer(
    site: Res<Site>,
    ghosts: Query<(), With<Ghost>>,
    customize: Res<crate::customize::Customize>,
    mut wants: ResMut<WantsPointer>,
) {
    wants.set_if_neq(WantsPointer(site.0.is_some() && (!ghosts.is_empty() || customize.on())));
}

// ---------------------------------------------------------------- the doors

/// Swings open each door with swinging leaves while anyone is near it, and
/// shut once nobody is: nearer it from inside, going out, than from outside,
/// coming in. Anyone is everyone on its plot, AIs as well, on its storey. A
/// door being put down stays shut. A double door's leaves swing together, and
/// open for someone going through either of them as soon as a door of one
/// leaf would. A town house's open for anyone on Blend In's venue as well,
/// which is the town as it stands (`island::scenery`, since 2026-11-18):
/// while you are there, for those there with you, and not for the town's
/// own people, who are not to be seen there.
fn swing_doors(
    time: Res<Time>,
    me: Query<&Venue, With<Player>>,
    bodies: Query<(&Transform, &Venue), With<Seat>>,
    mut doors: Query<(&mut Doorway, &GlobalTransform, &Span, &ChildOf, Has<Ghost>)>,
    venues: Query<&Venue>,
    mut leaves: Query<&mut Transform, Without<Seat>>,
) {
    let dt = time.delta_secs();
    let here = me.single().ok().copied();
    for (mut door, place, span, house, ghost) in &mut doors {
        // Whose door it is: its house's island's, or, while you stand on a
        // venue whose scenery that is, yours.
        let venue = venues.get(house.parent()).ok().map(|&venue| match here {
            Some(here) if here != venue && island::scenery(here) == venue => here,
            _ => venue,
        });
        // The middle of the bottom of the doorway, which way is in, and which
        // way along it.
        let (middle, inward, along) = (
            place.translation(),
            place.rotation() * Vec3::Z,
            place.rotation() * Vec3::X,
        );
        let hold = if door.wanted { SWING_HOLD } else { 0.0 };
        let wanted = !ghost
            && bodies.iter().any(|(body, &on)| {
                let at = body.translation;
                let nearest = nearest_leaf(at, middle, along, door.reach);
                Some(on) == venue && opens(at, nearest, inward, span.above, hold)
            });
        if door.wanted != wanted {
            door.wanted = wanted;
        }
        let step = dt / if wanted { SWING_OPEN_TIME } else { SWING_SHUT_TIME };
        let through = (door.through + if wanted { step } else { -step }).clamp(0.0, 1.0);
        if through == door.through {
            continue;
        }
        door.through = through;
        // Easing into the swing and out of it.
        let eased = through * through * (3.0 - 2.0 * through);
        for leaf in &door.leaves {
            if let Ok(mut part) = leaves.get_mut(leaf.part) {
                // About the upright through its hinge in the space it hangs
                // in: turned about its own, a leaf modelled as the mirror
                // image of another would swing the other way.
                part.rotation = Quat::from_rotation_y(leaf.open * eased) * leaf.shut;
            }
        }
    }
}

/// Which way, about the upright, a leaf turns to swing out of the house: 1 or
/// -1 for the turn that takes `far`, the way from its hinge to its middle,
/// round toward `out`.
fn outward(far: Vec3, out: Vec3) -> f32 {
    far.cross(out).y.signum()
}

/// The point on the bottom of a doorway nearest someone standing at `at`,
/// between the middles of its leaves: `reach` either way `along` it from its
/// middle, `middle`.
fn nearest_leaf(at: Vec3, middle: Vec3, along: Vec3, reach: f32) -> Vec3 {
    middle + along * (at - middle).dot(along).clamp(-reach, reach)
}

/// Whether someone standing at `at` is near enough a door to have it open:
/// the middle of the bottom of its doorway at `middle` (with two leaves, the
/// point between theirs nearest them, [`nearest_leaf`]), `inward` the way
/// into the house, `high` how tall it is, and `hold` how much further off
/// they can be because it is open already.
fn opens(at: Vec3, middle: Vec3, inward: Vec3, high: f32, hold: f32) -> bool {
    let off = at - middle;
    let reach = if off.dot(inward) > 0.0 {
        SWING_INSIDE
    } else {
        SWING_OUTSIDE
    };
    off.y > -1.0 && off.y < high && off.xz().length() < reach + hold
}

// ---------------------------------------------------------------- the walls

/// Cuts the holes the doors and windows in each wall need, whenever they
/// change: a house's built in House Builder, not one of the town's, cut once.
fn carve_walls(
    mut walls: Query<(Entity, &mut Wall, Option<&Banded>), Without<town::TownWall>>,
    openings: Query<(&OnWall, &Span, &Piece)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut wall, banded) in &mut walls {
        let mut holes: Vec<[f32; 4]> = openings
            .iter()
            .filter(|(on, _, piece)| on.wall == entity && piece.def.holed())
            .map(|(on, span, _)| {
                [
                    on.along - span.half,
                    on.up - span.below,
                    on.along + span.half,
                    on.up + span.above,
                ]
            })
            .collect();
        holes.sort_by(|a, b| a[0].total_cmp(&b[0]));
        if holes == wall.holes {
            continue;
        }
        wall.holes = holes;
        // A wall drawn in bands, each band cut round the holes in it.
        let Some(banded) = banded else {
            let _ = meshes.insert(&wall.mesh, wall_mesh(&wall));
            continue;
        };
        let drawn = std::iter::once(&wall.mesh).chain(&banded.upper);
        for (mesh, made) in drawn.zip(wall_meshes(&wall, &banded.cuts)) {
            let _ = meshes.insert(mesh, made);
        }
    }
}

/// What fills the hole a window taking up `span` cuts in `wall`, in the
/// window's own space: its glass, as far as bodies go. Drawn, the hole is
/// open; to them it is shut, so that nobody comes in through a window,
/// however low it is or however high they jump.
fn pane(span: Span, wall: &Wall) -> Mesh {
    let thick = wall.thin().1;
    let low = Vec3::new(-span.half, -span.below, -thick);
    let high = Vec3::new(span.half, span.above, thick);
    Mesh::from(Cuboid::from_corners(low, high)).translated_by((low + high) * 0.5)
}

/// A wall as a mesh: the boxes [`carve`] leaves of it.
fn wall_mesh(wall: &Wall) -> Mesh {
    wall_band(wall, wall.low.y, wall.high.y)
}

/// A wall as meshes, bottom first, cut at the heights `cuts` ([`Banded`]):
/// a band from its foot to the first, from each to the next, and from the
/// last to its top; with none, the whole of it.
fn wall_meshes(wall: &Wall, cuts: &[f32]) -> Vec<Mesh> {
    let edges: Vec<f32> = std::iter::once(wall.low.y).chain(cuts.iter().copied()).chain([wall.high.y]).collect();
    edges.windows(2).map(|band| wall_band(wall, band[0], band[1])).collect()
}

/// What of a wall is from `bottom` up to `top`, as a mesh: the boxes
/// [`carve`] leaves of that much of its face.
fn wall_band(wall: &Wall, bottom: f32, top: f32) -> Mesh {
    let (middle, half) = wall.thin();
    let [start, _, end, _] = wall.face();
    let parts = carve([start, bottom, end, top], &wall.holes);
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    for (n, [start, bottom, end, top]) in parts.into_iter().enumerate() {
        let (low, high) = if wall.along_x() {
            (
                Vec3::new(start, bottom, middle - half),
                Vec3::new(end, top, middle + half),
            )
        } else {
            (
                Vec3::new(middle - half, bottom, start),
                Vec3::new(middle + half, top, end),
            )
        };
        let part = Mesh::from(Cuboid::from_corners(low, high)).translated_by((low + high) * 0.5);
        if n == 0 {
            mesh = part;
        } else if let Err(error) = mesh.merge(&part) {
            warn!("could not make a wall: {error}");
        }
    }
    mesh
}

/// What is left of the face of a wall, `[along, up, along, up]` from corner to
/// corner, once `holes` like it are cut out of it: rectangles running from one
/// hole's edge to the next, as few as it takes.
fn carve(face: [f32; 4], holes: &[[f32; 4]]) -> Vec<[f32; 4]> {
    let mut cuts = vec![face[0], face[2]];
    for hole in holes {
        cuts.push(hole[0].clamp(face[0], face[2]));
        cuts.push(hole[2].clamp(face[0], face[2]));
    }
    cuts.sort_by(f32::total_cmp);
    cuts.dedup();

    let mut left: Vec<[f32; 4]> = Vec::new();
    for strip in cuts.windows(2) {
        let (start, end) = (strip[0], strip[1]);
        if end - start <= 1e-4 {
            continue;
        }
        let middle = (start + end) * 0.5;
        let mut through: Vec<(f32, f32)> = holes
            .iter()
            .filter(|hole| hole[0] < middle && middle < hole[2])
            .map(|hole| (hole[1], hole[3]))
            .collect();
        through.sort_by(|a, b| a.0.total_cmp(&b.0));
        // Up the strip, keeping what no hole takes.
        let mut kept = Vec::new();
        let mut up = face[1];
        for (bottom, top) in through {
            if bottom > up {
                kept.push((up, bottom.min(face[3])));
            }
            up = up.max(top);
        }
        if up < face[3] {
            kept.push((up, face[3]));
        }
        for (bottom, top) in kept {
            if top - bottom <= 1e-4 {
                continue;
            }
            // Straight on from the same rectangle in the strip before.
            match left
                .iter_mut()
                .find(|run| run[2] == start && run[1] == bottom && run[3] == top)
            {
                Some(run) => run[2] = end,
                None => left.push([start, bottom, end, top]),
            }
        }
    }
    left
}

/// The nearest point to `want` between `low` and `high` that is not inside any
/// of the stretches `taken`, if there is one.
fn nearest_free(want: f32, (low, high): (f32, f32), taken: &[(f32, f32)]) -> Option<f32> {
    if low > high {
        return None;
    }
    const SLACK: f32 = 1e-4;
    std::iter::once(want.clamp(low, high))
        .chain(taken.iter().flat_map(|&(start, end)| [start, end]))
        .filter(|&at| at >= low - SLACK && at <= high + SLACK)
        .filter(|&at| {
            taken
                .iter()
                .all(|&(start, end)| at <= start + SLACK || at >= end - SLACK)
        })
        .min_by(|a, b| (a - want).abs().total_cmp(&(b - want).abs()))
}

// ---------------------------------------------------------------- the plot

/// How long after reading the shape of what stands it is before it is read
/// again, in seconds (since 2026-11-23, when every plot of the town came
/// to have something on it: some 180,000 triangles are gone through each
/// time, and at the app's opening a piece's model loads nearly every frame
/// for a few seconds).
const RESHAPE_REST: f32 = 0.35;

/// Reads the plot's shape again when something on it has been put down,
/// picked up or thrown away: the island, and every piece that is down on it
/// and solid, with its walls as they are cut now.
fn reshape_plot(
    time: Res<Time>,
    mut rested: Local<Option<f32>>,
    mut reshape: ResMut<Reshape>,
    mut gone: RemovedComponents<Piece>,
    tops: Query<(Entity, &Piece, &Venue), Without<Ghost>>,
    children: Query<&Children>,
    ghosts: Query<(), With<Ghost>>,
    kinds: Query<&Piece>,
    panes: Query<(&OnWall, &Span)>,
    walls: Query<&Wall>,
    parts: Query<(&Mesh3d, Has<Band>)>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    mut islands: ResMut<Islands>,
) {
    if gone.read().count() > 0 {
        reshape.0 = true;
    }
    // No oftener than every [`RESHAPE_REST`]: one thing put down is read
    // at once, and a town's worth of plots coming in over a few seconds,
    // each piece as its model loads, a few times and not once a frame.
    let since = rested.map(|since| since + time.delta_secs());
    if !reshape.0 || since.is_some_and(|since| since < RESHAPE_REST) {
        *rested = since;
        return;
    }
    (*rested, reshape.0) = (Some(0.0), false);
    // Each plot's triangles, a list to each piece.
    let mut plots: Vec<(Venue, Vec<Vec<[Vec3; 3]>>)> = Vec::new();
    for (root, piece, &venue) in &tops {
        if !piece.def.solid() {
            continue;
        }
        let at = match plots.iter().position(|(on, _)| *on == venue) {
            Some(at) => at,
            None => {
                plots.push((venue, Vec::new()));
                plots.len() - 1
            }
        };
        let (_, pieces) = &mut plots[at];
        pieces.push(Vec::new());
        // Each wall of a house, made again round its holes, a list of its
        // own: closed, it is solid whatever of the house meets it. The
        // Square House's front gable is inside out, its underside facing up
        // at the very height of the top of the wall under it, and the two,
        // counted together, were taken for one surface and left that wall
        // open to walk through (found 2026-10-30, in the town's houses).
        let mut own: Vec<Vec<[Vec3; 3]>> = Vec::new();
        let Some(corners) = pieces.last_mut() else {
            continue;
        };
        // Down through it, leaving out a ghost, and what goes on the walls:
        // all but the glass of a window, which fills its hole.
        let mut next = vec![root];
        while let Some(entity) = next.pop() {
            let left_out = entity != root
                && (ghosts.contains(entity)
                    || kinds.get(entity).is_ok_and(|kind| !kind.def.solid()));
            if left_out {
                if !ghosts.contains(entity)
                    && let Ok(kind) = kinds.get(entity)
                    && kind.def.mount == Mount::Window
                    && let Ok((on, span)) = panes.get(entity)
                    && let Ok(wall) = walls.get(on.wall)
                    && let Ok(place) = placement.compute_global_transform(entity)
                {
                    island::add_mesh(corners, &pane(*span, wall), &place);
                }
                continue;
            }
            if let Ok((part, band)) = parts.get(entity)
                && let Some(mesh) = meshes.get(&part.0)
                && let Ok(place) = placement.compute_global_transform(entity)
            {
                // A band of a wall drawn in bands is as much a wall.
                if walls.contains(entity) || band {
                    let mut wall = Vec::new();
                    island::add_mesh(&mut wall, mesh, &place);
                    own.push(wall);
                } else {
                    island::add_mesh(corners, mesh, &place);
                }
            }
            if let Ok(below) = children.get(entity) {
                next.extend(below);
            }
        }
        pieces.extend(own);
    }
    for venue in islands.built_on() {
        if !plots.iter().any(|(on, _)| *on == venue) {
            islands.build_on(venue, &[]);
        }
    }
    // What stands on the town's plots is laid over the town's shape, a
    // layer of its own beside the plots' signs (`land`), rather than read
    // into it: the town is too big to read again each time anyone puts a
    // chair down.
    let mut town = None;
    for (venue, pieces) in plots {
        if venue == Venue::Town {
            town = Some(pieces);
        } else {
            islands.build_on(venue, &pieces);
        }
    }
    islands.lay_layer(Venue::Town, STOOD_LAYER, town);
}

/// The layer of the town's shape that what stands on its plots is
/// (`Islands::lay_layer`).
const STOOD_LAYER: &str = "stood";

// ---------------------------------------------------------------- online

/// Someone else's piece, down on their plot in a game played online, built
/// here from what the server says ([`raise_shadows`]): drawn, stood on and
/// bumped into like any other, and never edited.
#[derive(Component)]
pub(crate) struct Shadow;

/// On a piece the server has said is down, the piece built here for it.
#[derive(Component)]
struct Shadowed(Entity);

/// One of your pieces the server has been told is down: under which id, and
/// where.
#[derive(Component)]
struct Reported {
    id: u32,
    at: At,
}

/// The id the next of your pieces goes to the server under: never the same
/// twice, from one game to the next.
#[derive(Resource, Default)]
struct NextPieceId(u32);

/// Pieces of yours picked back up or thrown away while the connection was
/// gone: the server is told once it is back.
#[derive(Resource, Default)]
struct Untold(Vec<u32>);

/// Tells the server about every piece of yours as it is put down, or moved
/// once it is down — a tick, or the time running out with it still out — and,
/// once the connection is back after going, about all of them again. What is
/// picked back up or thrown away, [`took`] tells it.
#[allow(clippy::too_many_arguments)]
fn report_pieces(
    plot: Res<OnlinePlot>,
    mut ids: ResMut<NextPieceId>,
    mut untold: ResMut<Untold>,
    pieces: Query<
        (
            Entity,
            &Piece,
            &Transform,
            Option<&Venue>,
            Option<&OnWall>,
            Option<&ChildOf>,
            Option<&Reported>,
            Has<Ghost>,
        ),
        // A game's own: what stands on your plot of the town is told of by
        // `stand`.
        (Without<Shadow>, With<InGame>),
    >,
    venues: Query<&Venue>,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    mut asks: MessageWriter<roundtown_net::Ask>,
    mut commands: Commands,
) {
    let Some(mine) = plot.0 else {
        // The game is over: there is nothing left to tell.
        if pieces.is_empty() {
            untold.0.clear();
        }
        return;
    };
    let again = plot.is_changed();
    if again {
        for id in untold.0.drain(..) {
            asks.write(roundtown_net::Ask::Take { id });
        }
    }
    for (entity, piece, place, venue, on_wall, parent, reported, ghost) in &pieces {
        let on = venue
            .copied()
            .or_else(|| parent.and_then(|house| venues.get(house.parent()).ok().copied()));
        if on != Some(Venue::Plot(mine)) {
            continue;
        }
        if ghost {
            if reported.is_some() {
                commands.entity(entity).remove::<Reported>();
            }
            continue;
        }
        let at = match (piece.def.on_wall(), on_wall) {
            (false, _) => At::Ground {
                x: place.translation.x,
                y: place.translation.y,
                z: place.translation.z,
                yaw: place.rotation.to_euler(EulerRot::YXZ).0,
            },
            (true, Some(on)) => {
                let Some(wall) = wall_index(on.wall, &walls) else {
                    continue;
                };
                At::Wall {
                    wall,
                    along: on.along,
                    up: on.up,
                    face: on.face as i8,
                }
            }
            // Not on its wall yet.
            (true, None) => continue,
        };
        if !again && reported.is_some_and(|reported| reported.at == at) {
            continue;
        }
        let id = reported.map_or_else(
            || {
                ids.0 += 1;
                ids.0
            },
            |reported| reported.id,
        );
        asks.write(roundtown_net::Ask::Put {
            id,
            kind: piece.def.kind.into(),
            at,
        });
        commands.entity(entity).insert(Reported { id, at });
    }
}

/// One of your pieces is no longer down — picked back up, thrown away, or
/// gone with the game: the server is told, or, with the connection gone, told
/// once it is back.
fn took(
    remove: On<Remove, Reported>,
    reported: Query<&Reported>,
    plot: Res<OnlinePlot>,
    mut untold: ResMut<Untold>,
    mut asks: MessageWriter<roundtown_net::Ask>,
) {
    let Ok(reported) = reported.get(remove.entity) else {
        return;
    };
    if plot.0.is_some() {
        asks.write(roundtown_net::Ask::Take { id: reported.id });
    } else {
        untold.0.push(reported.id);
    }
}

/// Builds every piece the server says is down on someone else's plot, the
/// way it was put down there. What goes in a wall waits for its house, and if
/// the house is put down again somewhere else, goes back in it there.
#[allow(clippy::too_many_arguments)]
fn raise_shadows(
    plot: Res<OnlinePlot>,
    built: Query<(Entity, &Built, Option<&Shadowed>), With<Remote>>,
    pieces: Query<(), With<Piece>>,
    houses: Query<(Entity, &Piece, &Venue), (With<Shadow>, With<Rooms>)>,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    stock: Res<Stock>,
    mut commands: Commands,
) {
    let Some(mine) = plot.0 else {
        return;
    };
    for (entity, built, shadowed) in &built {
        if built.plot == mine || shadowed.is_some_and(|shadow| pieces.contains(shadow.0)) {
            continue;
        }
        // One a newer app sells, which this one has never heard of.
        let Some(def) = PIECES.iter().find(|def| def.kind == built.kind) else {
            continue;
        };
        let venue = Venue::Plot(built.plot);
        let mut shadow = match built.at {
            At::Ground { x, y, z, yaw } => commands.spawn((
                venue,
                Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_y(yaw)),
            )),
            At::Wall {
                wall,
                along,
                up,
                face,
            } => {
                let Some((house, ..)) = houses
                    .iter()
                    .find(|&(_, piece, on)| piece.def.mount == Mount::House && *on == venue)
                else {
                    continue;
                };
                let Some(&wall) = walls_of(house, &walls).get(usize::from(wall)) else {
                    continue;
                };
                let Ok((_, of, _)) = walls.get(wall) else {
                    continue;
                };
                let on = OnWall {
                    wall,
                    along,
                    up,
                    face: f32::from(face),
                };
                commands.spawn((ChildOf(house), of.place(def.holed(), on), on))
            }
        };
        shadow.insert((
            Piece { def },
            Shadow,
            InGame,
            WorldAssetRoot(stock.scene(def)),
            Visibility::Inherited,
        ));
        shadow.observe(piece_ready);
        let made = shadow.id();
        commands.entity(entity).insert(Shadowed(made));
    }
}

/// Someone else's piece, put down again somewhere else: built again there.
fn rebuilt(insert: On<Insert, Built>, shadowed: Query<&Shadowed>, mut commands: Commands) {
    if let Ok(shadow) = shadowed.get(insert.entity) {
        commands.entity(shadow.0).try_despawn();
        commands.entity(insert.entity).try_remove::<Shadowed>();
    }
}

/// Someone else's piece, picked back up or thrown away: gone here too.
fn unbuilt(remove: On<Remove, Built>, shadowed: Query<&Shadowed>, mut commands: Commands) {
    if let Ok(shadow) = shadowed.get(remove.entity) {
        commands.entity(shadow.0).try_despawn();
    }
}

/// A house's walls, in order of where they stand in it: the same order on
/// every device, which their entities are not. By each one's low corner to
/// the millimetre ([`wall_order`]), across the house, then front to back,
/// then up: of Hajun's square houses, the front wall, the right, the back
/// and the left.
fn walls_of(house: Entity, walls: &Query<(Entity, &Wall, &ChildOf)>) -> Vec<Entity> {
    let mut of: Vec<(Entity, [i64; 3])> = walls
        .iter()
        .filter(|(.., parent)| parent.parent() == house)
        .map(|(entity, wall, _)| (entity, wall_order(wall.low)))
        .collect();
    of.sort_by_key(|&(_, order)| order);
    of.into_iter().map(|(entity, _)| entity).collect()
}

/// What a wall whose low corner is `low`, in its house's own space, is
/// ordered by among the house's walls: the corner to the millimetre,
/// across, front to back and up. To the millimetre, so that two walls
/// that begin at the same place across the house are told apart by where
/// they begin front to back, and never by the last bits of a number
/// ([`within`]).
fn wall_order(low: Vec3) -> [i64; 3] {
    [low.x, low.z, low.y].map(|metres| (f64::from(metres) * 1000.0).round() as i64)
}

/// Where `wall` comes among its house's walls ([`walls_of`]).
fn wall_index(wall: Entity, walls: &Query<(Entity, &Wall, &ChildOf)>) -> Option<u8> {
    let (_, _, house) = walls.get(wall).ok()?;
    walls_of(house.parent(), walls)
        .iter()
        .position(|&of| of == wall)
        .and_then(|at| u8::try_from(at).ok())
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

    use super::*;

    fn area(parts: &[[f32; 4]]) -> f32 {
        parts.iter().map(|p| (p[2] - p[0]) * (p[3] - p[1])).sum()
    }

    /// How far off the nearest of `step`s `angle` is, in radians.
    fn off_the_step(angle: f32, step: f32) -> f32 {
        let over = angle.rem_euclid(step);
        over.min(step - over)
    }

    #[test]
    fn a_new_thing_comes_up_square_to_its_plot_however_the_plot_is_turned() {
        // Every plot of the town, from every way you may be looking (Hajun,
        // 2026-11-25: "if i am on plot 43 for example, i cannnot place any
        // object parallel to the edges of the plot").
        let step = TURN_STEP_DEG.to_radians();
        let plots = crate::land::tests::plots();
        assert_eq!(plots.len(), roundtown_net::land::PLOTS);
        let mut crooked = Vec::new();
        for plot in &plots {
            let yard = stand::yard_of(plot);
            let square = yard.place().rotation;
            let plot_yaw = square.to_euler(EulerRot::YXZ).0;
            if off_the_step(plot_yaw, step) > 1e-3 {
                crooked.push(plot.middle);
            }
            for way in 0..72 {
                let way = Vec3::new((way as f32 * 0.0873).sin(), 0.0, (way as f32 * 0.0873).cos());
                let turn = facing(way, Some(yard));
                // Turned in the plot's own space by a whole number of 45°.
                let within = (square.inverse() * turn).to_euler(EulerRot::YXZ).0;
                assert!(off_the_step(within, step) < 1e-4, "{:?}: {within}", plot.middle);
                // And its front, Blender's -Y, the nearest of those to the way.
                let front = turn * Vec3::NEG_Z;
                assert!(front.angle_between(way) <= step * 0.5 + 1e-4, "{:?}: {front} for {way}", plot.middle);
            }
            // Its back along the plot's front edge, looking out of it.
            let out = Vec3::new(yard.front.x, 0.0, yard.front.y);
            assert!((facing(out, Some(yard)) * Vec3::NEG_Z).distance(out) < 1e-4, "{:?}", plot.middle);
        }
        // Some of the plots are on no 45° of the world's, plot 43 among
        // them, which is what this is for.
        let [x, z] = roundtown_net::land::HOMES[42];
        assert!(crooked.iter().any(|middle| middle.distance(Vec2::new(x, z)) < 1.0), "{crooked:?}");
        // With no plot, as in House Builder, to the world's own 45°.
        let turn = facing(Vec3::new(0.3, 0.0, 1.0), None);
        assert!(off_the_step(turn.to_euler(EulerRot::YXZ).0, step) < 1e-4);
    }

    #[test]
    fn a_door_leaves_the_wall_round_it() {
        let face = [0.0, 0.0, 6.0, 2.8];
        let door = [2.0, 0.0, 3.4, 2.3];
        let parts = carve(face, &[door]);
        assert_eq!(parts.len(), 3, "{parts:?}");
        assert!((area(&parts) - (6.0 * 2.8 - 1.4 * 2.3)).abs() < 1e-4);
        assert!(
            parts.contains(&[2.0, 2.3, 3.4, 2.8]),
            "the lintel: {parts:?}"
        );
    }

    #[test]
    fn a_door_and_a_window_share_a_wall() {
        let face = [-3.0, 0.0, 3.0, 2.8];
        let door = [-2.0, 0.0, -0.6, 2.3];
        let window = [0.5, 0.9, 1.7, 2.0];
        let parts = carve(face, &[door, window]);
        let expected = 6.0 * 2.8 - 1.4 * 2.3 - 1.2 * 1.1;
        assert!((area(&parts) - expected).abs() < 1e-4, "{parts:?}");
        // No two pieces overlap.
        for (i, a) in parts.iter().enumerate() {
            for b in &parts[i + 1..] {
                let apart = a[2] <= b[0] || b[2] <= a[0] || a[3] <= b[1] || b[3] <= a[1];
                assert!(apart, "{a:?} and {b:?} overlap");
            }
        }
        // And nothing is left where either hole is.
        for p in &parts {
            for hole in [door, window] {
                let apart =
                    p[2] <= hole[0] || hole[2] <= p[0] || p[3] <= hole[1] || hole[3] <= p[1];
                assert!(apart, "{p:?} fills the hole {hole:?}");
            }
        }
    }

    #[test]
    fn an_uncut_wall_is_one_piece() {
        assert_eq!(carve([0.0, 0.0, 4.0, 2.8], &[]), vec![[0.0, 0.0, 4.0, 2.8]]);
    }

    #[test]
    fn free_room_is_nearest_the_finger() {
        let room = (0.0, 10.0);
        assert_eq!(nearest_free(4.0, room, &[]), Some(4.0));
        assert_eq!(nearest_free(-3.0, room, &[]), Some(0.0));
        // Pushed out of the way to whichever side is nearer.
        assert_eq!(nearest_free(4.0, room, &[(3.0, 6.0)]), Some(3.0));
        assert_eq!(nearest_free(5.5, room, &[(3.0, 6.0)]), Some(6.0));
        // Nowhere at all.
        assert_eq!(nearest_free(4.0, room, &[(-1.0, 11.0)]), None);
        assert_eq!(nearest_free(4.0, (5.0, 4.0), &[]), None);
    }

    /// The cottage's walls in Bevy's space, 2.8 m high and 0.3 thick: its
    /// front and back running the whole 6 m, its sides between them.
    fn cottage_walls() -> [(Vec3, Vec3); 4] {
        [
            (Vec3::new(-3.0, 0.0, -3.0), Vec3::new(3.0, 2.8, -2.7)),
            (Vec3::new(-3.0, 0.0, 2.7), Vec3::new(3.0, 2.8, 3.0)),
            (Vec3::new(-3.0, 0.0, -2.7), Vec3::new(-2.7, 2.8, 2.7)),
            (Vec3::new(2.7, 0.0, -2.7), Vec3::new(3.0, 2.8, 2.7)),
        ]
    }

    /// One of the cottage's walls, [`cottage_walls`]`[which]`.
    fn cottage_wall(which: usize) -> Wall {
        let floors = [(Vec3::new(-2.7, 0.0, -2.7), Vec3::new(2.7, 0.1, 2.7))];
        let walls = cottage_walls();
        let (low, high) = walls[which];
        Wall::new(low, high, &floors, &walls)
    }

    /// The cottage's front wall: 6 m along x, with the house behind it toward
    /// +z.
    fn front_wall() -> Wall {
        cottage_wall(0)
    }

    fn at(along: f32, up: f32, face: f32) -> OnWall {
        OnWall {
            wall: Entity::PLACEHOLDER,
            along,
            up,
            face,
        }
    }

    /// The door's frame, 1.4 m by 2.3, from the bottom middle of the hole.
    const DOOR: Span = Span {
        half: 0.7,
        below: 0.0,
        above: 2.3,
    };

    /// The window's frame, 1.2 m by 1.1.
    const WINDOW: Span = Span {
        half: 0.6,
        below: 0.0,
        above: 1.1,
    };

    /// The painting, 1 m by 0.75, from its middle.
    const PAINTING: Span = Span {
        half: 0.5,
        below: 0.375,
        above: 0.375,
    };

    /// Where a window goes first: [`WINDOW_SILL`] of the way up the storey at
    /// `near`.
    fn first(near: f32) -> Height {
        Height::Share {
            share: WINDOW_SILL,
            near,
        }
    }

    #[test]
    fn a_wall_knows_its_inside() {
        let wall = front_wall();
        assert!(wall.along_x());
        assert_eq!(wall.inside, Vec3::Z);
        // A door's +Z faces into the house, and it stands in the middle of
        // the wall's thickness.
        let place = wall.place(true, at(0.0, 0.0, 1.0));
        assert!((place.rotation * Vec3::Z).distance(Vec3::Z) < 1e-5);
        assert!((place.translation - Vec3::new(0.0, 0.0, -2.85)).length() < 1e-5);
        // Something hung on the inside faces into the house, its back on the
        // wall.
        let place = wall.place(false, at(0.0, 1.5, 1.0));
        assert!((place.forward().as_vec3() - Vec3::Z).length() < 1e-5);
        assert!((place.translation - Vec3::new(0.0, 1.5, -2.7)).length() < 1e-5);
    }

    #[test]
    fn a_wall_knows_where_the_others_meet_it() {
        // The front runs on behind the sides, into both corners.
        assert_eq!(
            front_wall().joins,
            vec![[-3.0, 0.0, -2.7, 2.8], [2.7, 0.0, 3.0, 2.8]]
        );
        // A side stops at the front and back, so nothing runs into it.
        assert!(cottage_wall(2).joins.is_empty());
    }

    #[test]
    fn doors_keep_clear_of_the_ends_and_each_other() {
        let wall = front_wall();
        let door = |along, others: &[(OnWall, Span, bool)]| {
            wall.fit(Mount::Door, DOOR, Height::At(0.0), along, 1.0, others)
        };
        // Asked for past the end, it stops just short of the side wall that
        // meets it there, on the floor.
        let (along, up) = door(20.0, &[]).unwrap();
        let end = 2.7 - EDGE - DOOR.half;
        assert!((along - end).abs() < 1e-4, "{along}");
        assert_eq!(up, 0.0);
        // Asked for on top of another door, it goes beside it.
        let other = (at(-end, 0.0, 1.0), DOOR, true);
        let (along, _) = door(-end + 0.5, &[other]).unwrap();
        assert!((along - (-end + DOOR.half * 2.0 + APART)).abs() < 1e-4, "{along}");
        // With no room beside it, there is nowhere.
        let wide = Span { half: 2.0, ..DOOR };
        assert_eq!(door(0.5, &[(at(0.0, 0.0, 1.0), wide, true)]), None);
    }

    #[test]
    fn a_window_goes_right_up_to_the_edges() {
        let window = |wall: &Wall, height, along| {
            wall.fit(Mount::Window, WINDOW, height, along, 1.0, &[])
                .unwrap()
        };
        // First a little way up its wall.
        let (_, up) = window(&front_wall(), first(0.0), 0.0);
        assert!((up - 2.8 * WINDOW_SILL).abs() < 1e-4, "{up}");
        // Then wherever it is dragged, as near the floor or the ceiling as
        // EDGE.
        let (_, up) = window(&front_wall(), Height::At(1.3), 0.0);
        assert!((up - 1.3).abs() < 1e-4, "{up}");
        let (_, up) = window(&front_wall(), Height::At(-5.0), 0.0);
        assert!((up - EDGE).abs() < 1e-4, "{up}");
        let (_, up) = window(&front_wall(), Height::At(5.0), 0.0);
        assert!((up - (2.8 - EDGE - WINDOW.above)).abs() < 1e-4, "{up}");
        // And as near the walls either side, on a wall that runs on into the
        // corners behind them as on one that stops at them.
        for wall in [cottage_wall(0), cottage_wall(2)] {
            for way in [1.0, -1.0] {
                let (along, _) = window(&wall, Height::At(1.0), 20.0 * way);
                let end = 2.7 - EDGE - WINDOW.half;
                assert!((along - end * way).abs() < 1e-4, "{along}");
            }
        }
    }

    #[test]
    fn holes_and_hangings_keep_out_of_each_others_way() {
        let wall = front_wall();
        // A painting hung on the far face, where a window would go.
        let hung = (at(0.0, 1.5, -1.0), PAINTING, false);
        // A hole goes right through, so the window keeps clear of it.
        let (along, _) = wall
            .fit(Mount::Window, WINDOW, first(0.0), 0.0, 1.0, &[hung])
            .unwrap();
        assert!(
            along.abs() >= WINDOW.half + PAINTING.half + APART - 1e-4,
            "{along}"
        );
        // Another painting on the near face is not in its way.
        let painting = |up, others: &[(OnWall, Span, bool)]| {
            wall.fit(Mount::Hanging, PAINTING, Height::At(up), 0.0, 1.0, others)
                .unwrap()
        };
        let (along, up) = painting(1.5, &[hung]);
        assert!(
            along.abs() < 1e-4 && (up - 1.5).abs() < 1e-4,
            "{along} {up}"
        );
        // And none hangs off the top of the wall.
        let (_, up) = painting(50.0, &[]);
        assert!((up - (2.8 - EDGE - PAINTING.above)).abs() < 1e-4, "{up}");
    }

    /// The front wall of one of Hajun's two floor houses, from the top of the
    /// ground floor's slab up to `top`, with the upper floor 6 m up.
    fn two_floor_wall(top: f32) -> Wall {
        let floors = [
            (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.1, 6.0)),
            (Vec3::new(-5.5, 6.0, -5.5), Vec3::new(3.5, 6.1, 5.5)),
        ];
        let walls = [
            (Vec3::new(-6.0, 0.1, -6.0), Vec3::new(6.0, top, -5.5)),
            (Vec3::new(-6.0, 0.1, 5.5), Vec3::new(6.0, top, 6.0)),
            (Vec3::new(-6.0, 0.1, -5.5), Vec3::new(-5.5, top, 5.5)),
            (Vec3::new(5.5, 0.1, -5.5), Vec3::new(6.0, top, 5.5)),
        ];
        Wall::new(walls[0].0, walls[0].1, &floors, &walls)
    }

    #[test]
    fn each_storey_is_a_wall_of_its_own() {
        // The two floor house: 12 m of wall, 6 m to each storey.
        let wall = two_floor_wall(12.1);
        assert_eq!(wall.storeys, vec![(0.1, 6.0), (6.1, 12.1)]);
        let window = |height, others: &[(OnWall, Span, bool)]| {
            wall.fit(Mount::Window, WINDOW, height, 0.0, 1.0, others)
                .unwrap()
        };
        // A window goes first in the storey you are on, as far up it as it
        // would be on a wall of its own.
        let (_, up) = window(first(0.0), &[]);
        assert!((up - (0.1 + 5.9 * WINDOW_SILL)).abs() < 1e-4, "{up}");
        let (_, up) = window(first(6.1), &[]);
        assert!((up - (6.1 + 6.0 * WINDOW_SILL)).abs() < 1e-4, "{up}");
        // Dragged, it goes in the storey its middle is in, clear of the floor
        // between them.
        let (_, up) = window(Height::At(5.0), &[]);
        assert!((up - (6.0 - EDGE - WINDOW.above)).abs() < 1e-4, "{up}");
        let (_, up) = window(Height::At(5.6), &[]);
        assert!((up - (6.1 + EDGE)).abs() < 1e-4, "{up}");
        // A door stands on the floor of either.
        let door = |up| wall.fit(Mount::Door, DOOR, Height::At(up), 0.0, 1.0, &[]);
        assert_eq!(door(6.5), Some((0.0, 6.1)));
        assert_eq!(door(3.0), Some((0.0, 0.1)));
        // A window over a door on the storey below is not in its way.
        let under = (at(0.0, 0.1, 1.0), DOOR, true);
        let (along, _) = window(first(6.1), &[under]);
        assert_eq!(along, 0.0);
        // A hanging dragged just under the upper floor stays under it, one
        // just over it stays over it, and one up at the roof stays under that.
        let painting = |up| {
            wall.fit(Mount::Hanging, PAINTING, Height::At(up), 0.0, 1.0, &[])
                .unwrap()
                .1
        };
        let up = painting(5.9);
        assert!((up - (6.0 - EDGE - PAINTING.above)).abs() < 1e-4, "{up}");
        let up = painting(6.2);
        assert!((up - (6.1 + EDGE + PAINTING.below)).abs() < 1e-4, "{up}");
        let up = painting(50.0);
        assert!((up - (12.1 - EDGE - PAINTING.above)).abs() < 1e-4, "{up}");
        // One house with one floor is one storey, the whole wall.
        assert_eq!(front_wall().storeys, vec![(0.0, 2.8)]);
    }

    /// How high the corners of `mesh` go, from the lowest to the highest,
    /// and how many corners it has.
    fn heights(mesh: &Mesh) -> (f32, f32, usize) {
        let Some(VertexAttributeValues::Float32x3(corners)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("no corners");
        };
        let low = corners.iter().map(|corner| corner[1]).fold(f32::INFINITY, f32::min);
        let high = corners.iter().map(|corner| corner[1]).fold(f32::NEG_INFINITY, f32::max);
        (low, high, corners.len())
    }

    #[test]
    fn a_wall_up_two_floors_is_drawn_a_floor_at_a_time_and_cut_where_its_holes_are() {
        // The Square House 2F's walls run whole from the ground to the roof
        // (Hajun, 2026-11-25: "each floor need to be its own color/material
        // when painting"): cut at the top of the upper floor, a band below
        // and a band above, which together are the whole wall.
        let mut wall = two_floor_wall(12.1);
        let levels = paint::Levels::of(&[
            (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.1, 6.0)),
            (Vec3::new(-5.5, 6.0, -5.5), Vec3::new(3.5, 6.1, 5.5)),
        ]);
        let cuts = levels.cuts(wall.low.y, wall.high.y);
        assert_eq!(cuts, vec![6.1]);
        // From how high to how high, and whether one box, of a band.
        let is = |mesh: &Mesh, low: f32, high: f32| {
            let (from, to, corners) = heights(mesh);
            (from - low).abs() < 1e-4 && (to - high).abs() < 1e-4 && corners == 24
        };
        let bands = wall_meshes(&wall, &cuts);
        assert_eq!(bands.len(), 2);
        assert!(is(&bands[0], 0.1, 6.1) && is(&bands[1], 6.1, 12.1));
        assert!(is(&wall_meshes(&wall, &[])[0], 0.1, 12.1) && is(&wall_mesh(&wall), 0.1, 12.1));
        // A door upstairs is a hole in the band above alone, and the band
        // below is as it was.
        wall.holes = vec![[-0.6, 6.1, 0.6, 8.2]];
        let bands = wall_meshes(&wall, &cuts);
        assert!(is(&bands[0], 0.1, 6.1));
        let (low, high, corners) = heights(&bands[1]);
        assert!((low - 6.1).abs() < 1e-4 && (high - 12.1).abs() < 1e-4, "{low} to {high}");
        assert!(corners > 24, "{corners}: no hole cut");
        // A wall of one floor, every wall of a house of three or more, is
        // not cut.
        assert!(levels.cuts(0.1, 6.1).is_empty());
        assert!(levels.cuts(6.1, 12.1).is_empty());
    }

    #[test]
    fn a_roof_terrace_has_no_room_for_a_window() {
        // The roof terrace house: its walls stop 1 m over the upper floor,
        // a parapet round it.
        let wall = two_floor_wall(7.1);
        assert_eq!(wall.storeys, vec![(0.1, 6.0), (6.1, 7.1)]);
        // Asked for from up there, a window or a door goes in downstairs,
        let (_, up) = wall
            .fit(Mount::Window, WINDOW, first(6.1), 0.0, 1.0, &[])
            .unwrap();
        assert!((up - (0.1 + 5.9 * WINDOW_SILL)).abs() < 1e-4, "{up}");
        assert_eq!(
            wall.fit(Mount::Door, DOOR, first(6.1), 0.0, 1.0, &[]),
            Some((0.0, 0.1))
        );
        // and a window dragged up there stays under the upper floor.
        let (_, up) = wall
            .fit(Mount::Window, WINDOW, Height::At(6.5), 0.0, 1.0, &[])
            .unwrap();
        assert!((up - (6.0 - EDGE - WINDOW.above)).abs() < 1e-4, "{up}");
        // A painting is small enough to hang on the parapet.
        let (_, up) = wall
            .fit(Mount::Hanging, PAINTING, Height::At(6.6), 0.0, 1.0, &[])
            .unwrap();
        assert!((up - 6.6).abs() < 1e-4, "{up}");
        // With the wall downstairs full, there is nowhere for a window: never
        // the parapet.
        let full = Span {
            half: 6.0,
            below: 0.0,
            above: 5.9,
        };
        let taken = (at(0.0, 0.1, 1.0), full, true);
        assert_eq!(
            wall.fit(Mount::Window, WINDOW, first(6.1), 0.0, 1.0, &[taken]),
            None
        );
    }

    /// Hajun's two floor house as they reshaped it on 2026-10-01, in Bevy's
    /// space: 10 m walls, the upper floor 5 m up, and the staircase up the
    /// right-hand wall (Blender's +x, -x here) from 1.25 m in from the front
    /// to the back wall. Its walls: front, back, the left, the right.
    fn stair_house() -> [Wall; 4] {
        let v = Vec3::new;
        let floors = [
            (v(-6.0, 0.0, -6.0), v(6.0, 0.1, 6.0)),
            (v(-3.5, 5.0, -5.5), v(5.5, 5.1, 5.5)),
        ];
        let walls = [
            (v(-6.0, 0.1, -6.0), v(6.0, 10.1, -5.5)),
            (v(-6.0, 0.1, 5.5), v(6.0, 10.1, 6.0)),
            (v(5.5, 0.1, -5.5), v(6.0, 10.1, 5.5)),
            (v(-6.0, 0.1, -5.5), v(-5.5, 10.1, 5.5)),
        ];
        let stairs = (v(-5.5, 0.1, -4.25), v(-3.5, 5.1, 5.5));
        let mut against = walls.to_vec();
        against.push(stairs);
        walls.map(|(low, high)| Wall::new(low, high, &floors, &against))
    }

    /// Hajun's square window's frame, 2 m by 2, and their classic door's, 2 m
    /// by 3.35.
    const SQUARE: Span = Span {
        half: 1.0,
        below: 0.0,
        above: 2.0,
    };
    const CLASSIC: Span = Span {
        half: 1.0,
        below: 0.0,
        above: 3.35,
    };

    #[test]
    fn nothing_goes_in_the_wall_the_stairs_go_up() {
        let [front, back, _, right] = stair_house();
        assert_eq!(right.storeys, vec![(0.1, 5.0), (5.1, 10.1)]);
        // Downstairs, the stairs take the whole of the right-hand wall a
        // window or a door would need: asked for there, either can only go
        // upstairs, over the stairs.
        let (_, up) = right
            .fit(Mount::Window, SQUARE, first(0.0), 0.0, 1.0, &[])
            .unwrap();
        assert_ne!(right.storey(up), right.storey(0.0), "{up}");
        let (_, up) = right
            .fit(Mount::Door, CLASSIC, Height::At(0.1), 0.0, 1.0, &[])
            .unwrap();
        assert_eq!(up, 5.1);
        // The back wall keeps its windows clear of the top of the stairs,
        // where they meet it, and the front wall has them where they are
        // asked for.
        let (along, _) = back
            .fit(Mount::Window, SQUARE, first(0.0), -5.0, 1.0, &[])
            .unwrap();
        assert!((along - (-3.5 + EDGE + SQUARE.half)).abs() < 1e-4, "{along}");
        let (along, up) = front
            .fit(Mount::Window, SQUARE, first(0.0), -2.0, 1.0, &[])
            .unwrap();
        assert!(along == -2.0 && front.storey(up) == front.storey(0.0), "{along} {up}");
    }

    #[test]
    fn a_flat_top_leaves_one_storey_under_it() {
        // The roofless house: 5 m walls, and a floor on top level with them.
        let floors = [
            (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.1, 6.0)),
            (Vec3::new(-5.5, 5.0, -5.5), Vec3::new(5.5, 5.1, 5.5)),
        ];
        let low = Vec3::new(-6.0, 0.1, -6.0);
        let high = Vec3::new(6.0, 5.1, -5.5);
        let wall = Wall::new(low, high, &floors, &[(low, high)]);
        assert_eq!(wall.storeys, vec![(0.1, 5.0), (5.1, 5.1)]);
        // A window dragged to the top stays under the floor there.
        let (_, up) = wall
            .fit(Mount::Window, SQUARE, Height::At(9.0), 0.0, 1.0, &[])
            .unwrap();
        assert!((up - (5.0 - EDGE - SQUARE.above)).abs() < 1e-4, "{up}");
    }

    #[test]
    fn anyone_walks_through_a_doorway() {
        // A classic door in the middle of a front wall, the house toward +z.
        let mut wall = two_floor_wall(12.1);
        let on = at(0.0, 0.1, 1.0);
        wall.holes = vec![[-CLASSIC.half, on.up, CLASSIC.half, on.up + CLASSIC.above]];
        let mut corners = Vec::new();
        island::add_mesh(&mut corners, &wall_mesh(&wall), &GlobalTransform::IDENTITY);
        let island = Island::new_for_tests(&corners);
        // Nothing in the doorway, and the wall either side of it.
        let feet = on.up;
        let (low, high) = (feet + crate::LAND_STEP_UP, feet + crate::PLAYER_HEIGHT);
        assert!(island.walls(Vec2::new(0.0, -5.75), 0.0, low, high).next().is_none());
        assert!(island.walls(Vec2::new(3.0, -5.75), 0.0, low, high).next().is_some());
    }

    #[test]
    fn a_finger_off_the_wall_still_drags_along_it() {
        let wall = front_wall();
        // From the middle of the cottage, a finger over the top of its front
        // wall: past the box of it, so not over it.
        let (from, way) = (Vec3::new(0.5, 1.5, 0.0), Vec3::new(0.0, 3.5, -3.0).normalize());
        assert!(ray_box(from, way, wall.low, wall.high).is_none());
        // It still says where on the wall's plane the finger is, and a window
        // dragged there goes as far up as it can, under the top.
        let far = wall.across_plane(from, way).unwrap();
        let aim = from + way * far;
        assert!((wall.out_of(aim)).abs() < 1e-4 && aim.y > 2.8, "{aim}");
        let (along, up) = wall
            .fit(Mount::Window, WINDOW, Height::At(aim.y), wall.along_of(aim), 1.0, &[])
            .unwrap();
        assert!((along - 0.5).abs() < 1e-4, "{along}");
        assert!((up - (2.8 - EDGE - WINDOW.above)).abs() < 1e-4, "{up}");
        // Looking along the wall, never across it, there is nowhere.
        assert!(wall.across_plane(from, Vec3::X).is_none());
    }

    #[test]
    fn nobody_gets_in_through_a_window() {
        // Hajun's square window as it first goes in on the ground floor of
        // the two floor house: taller than anyone, and low enough to jump
        // onto its sill.
        let square = SQUARE;
        let mut wall = two_floor_wall(12.1);
        let (_, up) = wall
            .fit(Mount::Window, square, first(0.0), 0.0, 1.0, &[])
            .unwrap();
        let on = at(0.0, up, 1.0);
        wall.holes = vec![[-square.half, up, square.half, up + square.above]];
        let mut open = Vec::new();
        island::add_mesh(&mut open, &wall_mesh(&wall), &GlobalTransform::IDENTITY);
        let mut shut = open.clone();
        let place = GlobalTransform::from(wall.place(true, on));
        island::add_mesh(&mut shut, &pane(square, &wall), &place);
        // Someone standing on the sill, in the middle of the wall.
        let in_the_way = |corners: &[[Vec3; 3]]| {
            Island::new_for_tests(corners)
                .walls(
                    Vec2::new(0.0, -5.75),
                    0.0,
                    up + crate::LAND_STEP_UP,
                    up + crate::PLAYER_HEIGHT,
                )
                .next()
                .is_some()
        };
        assert!(!in_the_way(&open), "the hole alone lets them through");
        assert!(in_the_way(&shut), "the glass does not");
    }

    /// The triangles of a closed box from `low` to `high`, wound facing out.
    fn block(low: Vec3, high: Vec3) -> Vec<[Vec3; 3]> {
        let p = |x: bool, y: bool, z: bool| {
            Vec3::new(
                if x { high.x } else { low.x },
                if y { high.y } else { low.y },
                if z { high.z } else { low.z },
            )
        };
        let (o, i) = (false, true);
        let quads = [
            [p(o, o, o), p(i, o, o), p(i, o, i), p(o, o, i)],
            [p(o, i, o), p(o, i, i), p(i, i, i), p(i, i, o)],
            [p(o, o, o), p(o, o, i), p(o, i, i), p(o, i, o)],
            [p(i, o, o), p(i, i, o), p(i, i, i), p(i, o, i)],
            [p(o, o, o), p(o, i, o), p(i, i, o), p(i, o, o)],
            [p(o, o, i), p(i, o, i), p(i, i, i), p(o, i, i)],
        ];
        quads
            .iter()
            .flat_map(|q| [[q[0], q[1], q[2]], [q[0], q[2], q[3]]])
            .collect()
    }

    #[test]
    fn furniture_goes_right_up_to_a_wall() {
        // A floor, and a wall across it at x = 2.
        let mut shape = block(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0));
        shape.extend(block(Vec3::new(2.0, 0.0, -5.0), Vec3::new(2.3, 2.8, 5.0)));
        let island = Island::new_for_tests(&shape);
        // A table a metre square.
        let bounds = Bounds {
            low: Vec3::new(-0.5, 0.0, -0.5),
            high: Vec3::new(0.5, 0.8, 0.5),
        };
        let fits = |at| !walled(&island, at, bounds, Quat::IDENTITY);
        let flush = 2.0 - bounds.high.x;
        let carry = |goal| editor::carry(&island, Vec3::ZERO, goal, Quat::IDENTITY, fits);
        // Dragged straight at the wall, it stops up against it.
        let at = carry(Vec2::new(10.0, 0.0));
        assert!(at.x <= flush + TOUCH && at.x > flush - 0.01, "{at}");
        // Dragged at it slantwise, it stops up against it and slides along.
        let at = carry(Vec2::new(10.0, 3.0));
        assert!(at.x <= flush + TOUCH && at.x > flush - 0.01, "{at}");
        assert!(at.z > 1.0, "{at}");
        // And snapped, it is flush against it, as it is from a little way off.
        let wall = Stop {
            place: Affine3A::IDENTITY,
            low: Vec3::new(2.0, 0.0, -5.0),
            high: Vec3::new(2.3, 2.8, 5.0),
            lines_up: false,
        };
        for goal in [Vec2::new(10.0, 0.0), Vec2::new(10.0, 3.0), Vec2::new(1.35, 0.0)] {
            let at = drag_over_floor(&island, None, Vec3::ZERO, goal, bounds, Quat::IDENTITY, &[wall]);
            assert!((at.x - flush).abs() < 1e-4, "{goal}: {at}");
        }
        // Not from further off.
        let at = drag_over_floor(&island, None, Vec3::ZERO, Vec2::new(1.2, 0.0), bounds, Quat::IDENTITY, &[wall]);
        assert!((at - Vec3::new(1.2, 0.0, 0.0)).length() < 1e-4, "{at}");
    }

    #[test]
    fn furniture_always_turns_out_of_a_wall() {
        // A floor, and a wall across it at x = 2.
        let mut shape = block(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0));
        shape.extend(block(Vec3::new(2.0, 0.0, -5.0), Vec3::new(2.3, 2.8, 5.0)));
        let island = Island::new_for_tests(&shape);
        // The edge desk with its long side flush against the wall, turned 45°
        // more: a corner of it would be 0.56 m into the wall, so it comes
        // straight out from the wall that far, and no further.
        let along = Quat::from_rotation_y(FRAC_PI_2);
        let at = Vec3::new(1.5, 0.0, 0.0);
        assert!(!walled(&island, at, DESK_EDGE, along));
        let turned = Quat::from_rotation_y(FRAC_PI_4) * along;
        assert!(walled(&island, at, DESK_EDGE, turned));
        let out = turned_clear(&island, None, at, DESK_EDGE, turned, &[]);
        assert!(!walled(&island, out, DESK_EDGE, turned), "{out}");
        let corner = FRAC_PI_4.cos() * 1.0 + FRAC_PI_4.sin() * 0.5;
        assert!((out.x - (2.0 - corner)).abs() < 0.005, "{out}");
        assert!(out.z.abs() < 1e-3 && out.y == 0.0, "{out}");
        // Clear where it is, it turns where it is.
        let free = Vec3::new(-3.0, 0.0, 0.0);
        assert_eq!(turned_clear(&island, None, free, DESK_EDGE, turned, &[]), free);
        // It never goes through a wall to get anywhere.
        assert!(open_between(&island, at, Vec3::new(1.0, 0.0, 4.0)));
        assert!(!open_between(&island, at, Vec3::new(3.0, 0.0, 0.0)));
    }

    #[test]
    fn furniture_always_turns_out_of_another_piece() {
        // The corner desk flush against the end of the edge desk, turned 45°:
        // a corner of it would be in the edge desk, so it comes out away from
        // it, as far as it takes.
        let (island, desk) = desk_down(DESK_EDGE, Quat::IDENTITY, Vec3::ZERO);
        let at = Vec3::new(1.5, 0.0, 0.0);
        let turned = Quat::from_rotation_y(FRAC_PI_4);
        assert!(walled(&island, at, DESK_CORNER, turned));
        let out = turned_clear(&island, None, at, DESK_CORNER, turned, &[desk]);
        assert!(!walled(&island, out, DESK_CORNER, turned), "{out}");
        let corner = FRAC_PI_4.cos();
        assert!((out.x - (1.0 + corner)).abs() < 0.005 && out.z.abs() < 1e-3, "{out}");
        // Between two walls too near for it to turn between, it turns where
        // it is, rather than go through either.
        let mut shape = block(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0));
        shape.extend(block(Vec3::new(-0.85, 0.0, -10.0), Vec3::new(-0.6, 2.8, 10.0)));
        shape.extend(block(Vec3::new(0.6, 0.0, -10.0), Vec3::new(0.85, 2.8, 10.0)));
        let corridor = Island::new_for_tests(&shape);
        let along = Quat::from_rotation_y(FRAC_PI_2);
        assert!(!walled(&corridor, Vec3::ZERO, DESK_EDGE, along));
        let across = turned_clear(&corridor, None, Vec3::ZERO, DESK_EDGE, Quat::IDENTITY, &[]);
        assert_eq!(across, Vec3::ZERO);
    }

    #[test]
    fn furniture_on_one_floor_leaves_the_other_free() {
        let ground = block(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0));
        // A house with an upper floor 5 m up, as the two floor houses have.
        let mut house = block(Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.1, 6.0));
        house.extend(block(Vec3::new(-5.5, 5.0, -5.5), Vec3::new(5.5, 5.1, 5.5)));
        // A bed on the floor `y`: a closed frame, with a blanket over it that
        // is only a sheet, so that the bed is not a closed shape.
        let bed = |y: f32| {
            let mut bed = block(Vec3::new(-1.0, y, -1.0), Vec3::new(1.0, y + 0.5, 1.0));
            let p = |x: f32, z: f32| Vec3::new(x, y + 0.55, z);
            bed.push([p(-1.0, -1.0), p(-1.0, 1.0), p(1.0, 1.0)]);
            bed.push([p(-1.0, -1.0), p(1.0, 1.0), p(1.0, -1.0)]);
            bed
        };
        let (upstairs, downstairs) = (bed(5.1), bed(0.1));
        let body = |island: &Island, at: Vec2, feet: f32| {
            island.blocked(at, feet + crate::LAND_STEP_UP, feet + crate::PLAYER_HEIGHT)
        };
        let under = Island::of_parts_for_tests(&[&ground, &house, &upstairs]);
        let over = Island::of_parts_for_tests(&[&ground, &house, &downstairs]);
        // Read as one, the sheet throws out the count for all under it.
        let as_one: Vec<_> = [&ground, &house, &upstairs].into_iter().flatten().copied().collect();
        assert!(body(&Island::new_for_tests(&as_one), Vec2::ZERO, 0.1));
        // Each piece apart, the room under the bed and the floor over the
        // other are free, and each bed is still in the way on its own floor.
        assert!(!body(&under, Vec2::ZERO, 0.1));
        assert!(!body(&over, Vec2::ZERO, 5.1));
        assert!(body(&under, Vec2::ZERO, 5.1));
        assert!(body(&over, Vec2::ZERO, 0.1));
        // A table carried across the upper floor goes over the bed under it
        // without catching on anything.
        let table = Bounds {
            low: Vec3::new(-0.5, 0.0, -0.5),
            high: Vec3::new(0.5, 0.8, 0.5),
        };
        let fits = |at| !walled(&over, at, table, Quat::IDENTITY);
        let from = Vec3::new(-2.5, 5.1, 0.0);
        let at = editor::carry(&over, from, Vec2::new(2.5, 0.0), Quat::IDENTITY, fits);
        assert!((at - Vec3::new(2.5, 5.1, 0.0)).length() < 1e-4, "{at}");
        // Something left open at one end is still solid within its own
        // heights: a post with no lid.
        let mut post = block(Vec3::new(3.0, 0.1, 3.0), Vec3::new(3.5, 3.0, 3.5));
        post.drain(2..4);
        let posted = Island::of_parts_for_tests(&[&ground, &house, &post]);
        assert!(body(&posted, Vec2::new(3.25, 3.25), 0.1));
        assert!(!body(&posted, Vec2::new(3.25, 3.25), 5.1));
    }

    /// Hajun's cafe desks, as the game reads them: the edge 2 m across and 1 m
    /// deep, the corner 1 m square, both 1.5 m high.
    const DESK_EDGE: Bounds = Bounds {
        low: Vec3::new(-1.0, 0.0, -0.5),
        high: Vec3::new(1.0, 1.5, 0.5),
    };
    const DESK_CORNER: Bounds = Bounds {
        low: Vec3::new(-0.5, 0.0, -0.5),
        high: Vec3::new(0.5, 1.5, 0.5),
    };

    /// A floor with a desk of `bounds` down on it, turned by `turn` at `at`,
    /// and that desk as what a piece carried over the floor stops against.
    fn desk_down(bounds: Bounds, turn: Quat, at: Vec3) -> (Island, Stop) {
        let place = Affine3A::from_rotation_translation(turn, at);
        let mut shape = block(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0));
        shape.extend(
            block(bounds.low, bounds.high)
                .into_iter()
                .map(|tri| tri.map(|corner| place.transform_point3(corner))),
        );
        let stop = Stop {
            place,
            low: bounds.low,
            high: bounds.high,
            lines_up: true,
        };
        (Island::new_for_tests(&shape), stop)
    }

    /// A plot 24 m square about the origin, its front toward -Z, its ground
    /// at nothing.
    fn plot() -> Yard {
        Yard {
            middle: Vec2::ZERO,
            ground: 0.0,
            front: Vec2::NEG_Y,
            half: 12.0,
        }
    }

    /// A table a metre square.
    const TABLE: Bounds = Bounds {
        low: Vec3::new(-0.5, 0.0, -0.5),
        high: Vec3::new(0.5, 0.8, 0.5),
    };

    #[test]
    fn on_your_plot_a_thing_goes_where_the_finger_takes_it_through_whatever_stands_there() {
        // A desk down in the middle of the plot, and a table dragged about
        // it.
        let (island, desk) = desk_down(DESK_EDGE, Quat::IDENTITY, Vec3::ZERO);
        let yard = plot();
        let why = |spot| misfit(&island, &island, yard, Mount::Floor, spot, Quat::IDENTITY, TABLE, &[desk]);
        let drag = |from: Vec3, goal: Vec2| {
            drag_freely(yard, from, goal, TABLE, Quat::IDENTITY, &[desk], &[], |spot| why(spot).is_none())
        };
        let near = |at: Vec3, want: Vec3| (at - want).length() < 1e-4;
        // From one side of the desk to the far side of it: there, through
        // the desk, where carried over the floor as a piece in House
        // Builder is it stops up against it (Hajun, 2026-11-22: "moving it
        // around is a puzzle cuz u cant move it through other objects").
        let from = Vec3::new(-5.0, 0.0, 0.0);
        let at = drag(from, Vec2::new(5.0, 0.0));
        assert!(near(at, Vec3::new(5.0, 0.0, 0.0)), "{at}");
        let carried = drag_over_floor(&island, Some(yard), from, Vec2::new(5.0, 0.0), TABLE, Quat::IDENTITY, &[desk]);
        assert!(carried.x < -1.4, "{carried}");
        // Into the desk: there, in it, where it cannot be put down ("it
        // will just grey out the check mark if its colliding").
        let at = drag(from, Vec2::new(0.2, 0.1));
        assert!(near(at, Vec3::new(0.2, 0.0, 0.1)), "{at}");
        assert_eq!(why(at), Some(Misfit::InTheWay));
        // Brought up to its end, a little off line, it snaps flush against
        // it and in line with it, as it always did, and can be put down.
        let at = drag(from, Vec2::new(-1.62, 0.05));
        assert!(near(at, Vec3::new(-1.5, 0.0, 0.0)), "{at}");
        assert_eq!(why(at), None);
        // Past the edge of the plot: as far as the whole of it is on the
        // plot, and no further.
        let at = drag(from, Vec2::new(40.0, 3.0));
        assert!(near(at, Vec3::new(11.5, 0.0, 3.0)), "{at}");
        assert_eq!(why(at), None);
        assert_eq!(why(Vec3::new(11.8, 0.0, 3.0)), Some(Misfit::OffTheLand));
        // What goes in a wall is put where it fits as it is dragged: never
        // in the way.
        assert_eq!(misfit(&island, &island, yard, Mount::Door, Vec3::ZERO, Quat::IDENTITY, TABLE, &[desk]), None);
    }

    #[test]
    fn a_thing_is_in_the_way_of_another_only_where_their_shapes_meet() {
        // Hajun, 2026-11-25: "even they are not touching each other or the
        // wall, the checkmark disappear". An L shaped desk, two blocks 0.8
        // high: one 3 m along x, and one 1 m wide coming 2 m forward from
        // its left end; the box round it is 3 by 3 m, and empty at its
        // front right.
        let ground = block(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0));
        let along = block(Vec3::new(-1.5, 0.0, 0.5), Vec3::new(1.5, 0.8, 1.5));
        let forward = block(Vec3::new(-1.5, 0.0, -1.5), Vec3::new(-0.5, 0.8, 0.5));
        let island = Island::new_for_tests(&[ground, along, forward].concat());
        let yard = plot();
        let why = |at: Vec3, bounds: Bounds| misfit(&island, &island, yard, Mount::Floor, at, Quat::IDENTITY, bounds, &[]);
        // A table a metre square in the empty corner, inside the box round
        // the desk and touching none of it: it goes down.
        assert_eq!(why(Vec3::new(0.5, 0.0, -0.5), TABLE), None);
        // Pushed half into the desk: not.
        assert_eq!(why(Vec3::new(0.0, 0.0, 0.5), TABLE), Some(Misfit::InTheWay));
        // Flush against it, in the corner of the L and beside its end: it goes
        // down.
        assert_eq!(why(Vec3::new(0.0, 0.0, 0.0), TABLE), None);
        assert_eq!(why(Vec3::new(0.0, 0.0, -1.0), TABLE), None);
        // A table top on its own, a slab 0.9 to 1.0 m up on no legs, and a
        // stool 0.6 m high under it: it goes down; a chair 1.5 m high, whose
        // back would be in the top, does not.
        let slab = block(Vec3::new(-1.0, 0.9, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let island = Island::new_for_tests(&[block(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0)), slab].concat());
        let stool = Bounds {
            low: Vec3::new(-0.25, 0.0, -0.25),
            high: Vec3::new(0.25, 0.6, 0.25),
        };
        let chair = Bounds {
            low: Vec3::new(-0.4, 0.0, -0.4),
            high: Vec3::new(0.4, 1.5, 0.4),
        };
        let why = |bounds: Bounds| misfit(&island, &island, yard, Mount::Floor, Vec3::ZERO, Quat::IDENTITY, bounds, &[]);
        assert_eq!((why(stool), why(chair)), (None, Some(Misfit::InTheWay)));
        // And on the slab, a mug: on it, not in it.
        let mug = Bounds {
            low: Vec3::new(-0.1, 0.0, -0.1),
            high: Vec3::new(0.1, 0.3, 0.1),
        };
        let on_it = misfit(&island, &island, yard, Mount::Floor, Vec3::new(0.2, 1.0, 0.3), Quat::IDENTITY, mug, &[]);
        assert_eq!(on_it, None);
    }

    #[test]
    fn a_top_is_what_a_small_thing_goes_up_onto_and_a_side_is_not() {
        let mug = Bounds {
            low: Vec3::new(-0.1, 0.0, -0.1),
            high: Vec3::new(0.1, 0.3, 0.1),
        };
        // A desk's top, met at (1, 1.08, 2): the mug's foot there.
        assert_eq!(onto(Vec3::new(1.0, 1.08, 2.0), Vec3::Y, mug), Some(Vec3::new(1.0, 1.08, 2.0)));
        // A slope of 20 degrees is a top; its front, or a slope of 40, not.
        let tilted = |degrees: f32| Quat::from_rotation_x(degrees.to_radians()) * Vec3::Y;
        assert!(onto(Vec3::ONE, tilted(20.0), mug).is_some());
        assert!(onto(Vec3::ONE, tilted(40.0), mug).is_none());
        assert!(onto(Vec3::ONE, Vec3::NEG_Z, mug).is_none());
        // One whose origin is under its foot stands with its foot there.
        let sunk = Bounds {
            low: Vec3::new(-0.1, 0.11, -0.1),
            high: Vec3::new(0.1, 0.4, 0.1),
        };
        assert_eq!(onto(Vec3::new(0.0, 1.0, 0.0), Vec3::Y, sunk), Some(Vec3::new(0.0, 0.89, 0.0)));
        // Which they are: only things from the town's plots of 2026-11-25,
        // and none that stands on a floor however small.
        let small: Vec<&str> = PIECES.iter().filter(|def| def.small).map(|def| def.kind).collect();
        assert_eq!(small.len(), 29);
        for kind in ["hamburger", "mug_with_office_tool", "lamp_with_shade", "monitor", "computer", "cup_of_tea"] {
            assert!(small.contains(&kind), "{kind}");
        }
        for kind in ["stool", "trash_bin", "propane_tank", "closed_umbrella", "desk", "chair", "streetlamp"] {
            assert!(!small.contains(&kind), "{kind}");
        }
        assert!(PIECES.iter().filter(|def| def.small).all(|def| def.mount == Mount::Floor && !def.house_builder));
    }

    #[test]
    fn two_things_are_in_each_other_only_if_their_boxes_overlap_by_more_than_a_touch() {
        let (_, desk) = desk_down(DESK_EDGE, Quat::IDENTITY, Vec3::ZERO);
        let in_it = |at: Vec3, turn: Quat| overlapping(at, turn, TABLE, &desk);
        // Flush against its end, and against its front: not in it. A little
        // way in, or right in it: in it.
        assert!(!in_it(Vec3::new(1.5, 0.0, 0.0), Quat::IDENTITY));
        assert!(!in_it(Vec3::new(0.0, 0.0, -1.0), Quat::IDENTITY));
        assert!(in_it(Vec3::new(1.45, 0.0, 0.0), Quat::IDENTITY));
        assert!(in_it(Vec3::ZERO, Quat::IDENTITY));
        // Turned an eighth, its corner reaches further: in it where square
        // on it was clear, and clear a little further off.
        let eighth = Quat::from_rotation_y(FRAC_PI_4);
        assert!(in_it(Vec3::new(1.6, 0.0, 0.0), eighth));
        assert!(!in_it(Vec3::new(1.75, 0.0, 0.0), eighth));
        // Off its corner: clear.
        assert!(!in_it(Vec3::new(1.6, 0.0, 1.1), Quat::IDENTITY));
        // On the floor over it, or the one under: not in it.
        assert!(!in_it(Vec3::new(0.0, 1.5, 0.0), Quat::IDENTITY));
        assert!(!in_it(Vec3::new(0.0, -0.8, 0.0), Quat::IDENTITY));
        // A desk turned a quarter and stood off to one side is where it is:
        // in it there, and clear where it would have been unturned.
        let (_, turned) = desk_down(DESK_EDGE, Quat::from_rotation_y(FRAC_PI_2), Vec3::new(5.0, 0.0, 0.0));
        assert!(overlapping(Vec3::new(5.0, 0.0, 1.2), Quat::IDENTITY, TABLE, &turned));
        assert!(!overlapping(Vec3::new(6.2, 0.0, 0.0), Quat::IDENTITY, TABLE, &turned));
    }

    #[test]
    fn a_thing_dragged_on_your_plot_stands_on_the_floor_under_it() {
        let yard = Yard { ground: 0.27, ..plot() };
        // A house turned a quarter, off the middle of the plot: a ground
        // floor 8 by 6 and a tenth of a metre thick, and a floor over half
        // of it, three metres up.
        let house = Affine3A::from_rotation_translation(Quat::from_rotation_y(FRAC_PI_2), Vec3::new(2.0, 0.27, -3.0));
        let storey = |low: Vec3, high: Vec3| Storey {
            into: house.inverse(),
            low,
            high,
            top: 0.27 + high.y,
        };
        let storeys = [
            storey(Vec3::new(-4.0, 0.0, -3.0), Vec3::new(4.0, 0.1, 3.0)),
            storey(Vec3::new(-4.0, 3.0, -3.0), Vec3::new(0.0, 3.1, 3.0)),
        ];
        let in_house = |x: f32, z: f32| house.transform_point3(Vec3::new(x, 0.0, z)).xz();
        let step = crate::LAND_STEP_UP;
        let under = |spot: Vec2, from: f32| floor_under(yard, &storeys, spot, from + step);
        let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
        // Out on the plot: the land. Into the house off the land: its
        // floor, a step up, and not the floor over that.
        assert!(close(under(Vec2::new(-9.0, 9.0), 0.27), 0.27));
        assert!(close(under(in_house(2.0, 0.0), 0.27), 0.37));
        assert!(close(under(in_house(-2.0, 0.0), 0.37), 0.37));
        // From upstairs, the upper floor for as long as it is under the
        // spot, and off the edge of it the one below; which a thing
        // dragged upstairs is never let go down to (the test after this).
        assert!(close(under(in_house(-2.0, 0.0), 3.37), 3.37));
        assert!(close(under(in_house(2.0, 0.0), 3.37), 0.37));
        // And out of the house from in it: the land again.
        assert!(close(under(Vec2::new(-9.0, 9.0), 0.37), 0.27));
    }

    /// The Square House 2F's floors as its model has them, the ground floor
    /// out under its walls, 12 m across, and the upper floor inside them
    /// but for the stairwell along one wall, from 3.5 m to that side of its
    /// middle; the house turned an eighth of a turn, as on a crooked plot,
    /// on a plot whose ground is 0.27 up. The plot, where the house stands,
    /// and its floors.
    fn two_floors() -> (Yard, Affine3A, [Storey; 2]) {
        let yard = Yard { ground: 0.27, ..plot() };
        let house = Affine3A::from_rotation_translation(Quat::from_rotation_y(FRAC_PI_4), Vec3::new(1.0, 0.27, -2.0));
        let storey = |low: Vec3, high: Vec3| Storey {
            into: house.inverse(),
            low,
            high,
            top: 0.27 + high.y,
        };
        let storeys = [
            storey(Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.1, 6.0)),
            storey(Vec3::new(-3.5, 5.0, -5.5), Vec3::new(5.5, 5.1, 5.5)),
        ];
        (yard, house, storeys)
    }

    /// The Square House 2F's floors and the slabs of them, a flat ceiling
    /// over its upper floor, and in its stairwell a stair with an underside,
    /// as a staircase open beneath would have (Hajun's has none: what is
    /// over his stairwell is the roof).
    fn two_floors_built() -> (Yard, Affine3A, [Storey; 2], Island) {
        let (yard, house, storeys) = two_floors();
        let mut shape = block(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.27, 20.0));
        for (low, high) in [
            (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.1, 6.0)),
            (Vec3::new(-3.5, 5.0, -5.5), Vec3::new(5.5, 5.1, 5.5)),
            (Vec3::new(-6.0, 10.0, -6.0), Vec3::new(6.0, 10.1, 6.0)),
            (Vec3::new(-5.5, 2.4, -4.25), Vec3::new(-3.5, 2.6, 4.25)),
        ] {
            shape.extend(block(low, high).into_iter().map(|tri| tri.map(|corner| house.transform_point3(corner))));
        }
        (yard, house, storeys, Island::new_for_tests(&shape))
    }

    /// Hajun's stove, as its model has it: its origin off its middle.
    const STOVE: Bounds = Bounds {
        low: Vec3::new(-0.675, 0.0, -0.98),
        high: Vec3::new(0.675, 2.02, 0.675),
    };

    #[test]
    fn a_thing_dragged_upstairs_stays_upstairs_however_near_the_walls_the_finger_takes_it() {
        // Hajun, 2026-11-26: "im on second floor, and im moving a furniture
        // like stove around the floor, and as it goes near the corner/edge
        // near the wall, it falls down to the ground. so i need to be so
        // careful so it doesnt fall. fix this."
        let (yard, house, storeys) = two_floors();
        let in_house = |x: f32, y: f32, z: f32| house.transform_point3(Vec3::new(x, y, z));
        let own = |at: Vec3| house.inverse().transform_point3(at);
        let upstairs_at = 0.27 + 5.1;
        let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
        // Upstairs, and every corner of it over the upper floor.
        let over_the_floor = |at: Vec3, turn: Quat| {
            let (low, high) = (STOVE.low, STOVE.high);
            close(at.y, upstairs_at)
                && [(low.x, low.z), (high.x, low.z), (high.x, high.z), (low.x, high.z)].iter().all(|&(x, z)| {
                    let corner = own(at + turn * Vec3::new(x, 0.0, z));
                    corner.x >= -3.5 - 1e-3 && corner.x <= 5.5 + 1e-3 && corner.z >= -5.5 - 1e-3 && corner.z <= 5.5 + 1e-3
                })
        };
        // Square to the house, turned with the plot's edges, and crooked
        // to it; dragged from the middle of the upper floor up to a wall,
        // past it, into a corner and past that, over the stairwell, and off
        // the plot altogether: upstairs every time, the whole of it over
        // the floor, where its middle past the floor's edge had it down on
        // the ground floor or out on the land.
        let goals = [
            in_house(5.2, 5.1, 0.0),
            in_house(9.0, 5.1, 0.3),
            in_house(5.3, 5.1, -5.4),
            in_house(8.0, 5.1, -8.0),
            in_house(-4.0, 5.1, 1.0),
            in_house(-9.0, 5.1, 6.0),
            Vec3::new(40.0, 0.0, 40.0),
            Vec3::new(-40.0, 0.0, 3.0),
        ];
        for turn in [Quat::from_rotation_y(FRAC_PI_4), Quat::from_rotation_y(-FRAC_PI_4), Quat::IDENTITY, Quat::from_rotation_y(1.0)] {
            for goal in goals {
                let at = drag_freely(yard, in_house(1.0, 5.1, 0.0), goal.xz(), STOVE, turn, &[], &storeys, |_| true);
                assert!(over_the_floor(at, turn), "turned {turn:?}, to {goal}: at {at}, {} in the house", own(at));
            }
        }
        // Square to the house: flush against the wall it was taken past,
        // as near the finger along it as that leaves it; in the corner,
        // against both; at the edge of the stairwell; and within the floor,
        // where the finger asks, as ever.
        let turn = Quat::from_rotation_y(FRAC_PI_4);
        let drag = |goal: Vec3| own(drag_freely(yard, in_house(1.0, 5.1, 0.0), goal.xz(), STOVE, turn, &[], &storeys, |_| true));
        let at = drag(in_house(9.0, 5.1, 0.3));
        assert!(close(at.x, 5.5 - 0.675) && close(at.z, 0.3), "{at}");
        let at = drag(in_house(8.0, 5.1, -8.0));
        assert!(close(at.x, 5.5 - 0.675) && close(at.z, -5.5 + 0.98), "{at}");
        let at = drag(in_house(-4.0, 5.1, 1.0));
        assert!(close(at.x, -3.5 + 0.675) && close(at.z, 1.0), "{at}");
        let at = drag(in_house(2.0, 5.1, -1.5));
        assert!(close(at.x, 2.0) && close(at.z, -1.5) && close(at.y, 5.1), "{at}");
        // On the ground floor, as before: out through the wall onto the
        // land, and in again onto the floor, a step up.
        let drag = |from: Vec3, goal: Vec3| drag_freely(yard, from, goal.xz(), STOVE, turn, &[], &storeys, |_| true);
        let out = drag(in_house(1.0, 0.1, 0.0), in_house(8.0, 0.1, 0.0));
        assert!(close(out.y, 0.27) && close(own(out).x, 8.0), "{out}");
        let back = drag(out, in_house(1.0, 0.1, 0.0));
        assert!(close(back.y, 0.37) && close(own(back).x, 1.0), "{back}");
    }

    #[test]
    fn upstairs_is_a_floor_over_a_house_s_ground_floor_and_an_l_shaped_one_is_one_floor() {
        let (yard, house, storeys) = two_floors();
        let in_house = |x: f32, y: f32, z: f32| house.transform_point3(Vec3::new(x, y, z));
        let floor_of = |at: Vec3| upstairs(yard, &storeys, at.xz(), at.y + crate::LAND_STEP_UP);
        // The land and the ground floor: nothing to keep to, and one.
        let (land, ground) = (floor_of(Vec3::new(-9.0, 0.27, 9.0)), floor_of(in_house(1.0, 0.1, 0.0)));
        assert!(land.is_empty() && ground.is_empty());
        assert!(one_floor(&land, &ground));
        // The upper floor, and a desk's top on it: the one floor, and not
        // a table's top on the ground floor under it, which a finger over
        // the upper floor meets through it.
        let up = floor_of(in_house(1.0, 5.1, 0.0));
        assert_eq!(up.len(), 1);
        assert!((up[0].top - 5.37).abs() < 1e-4);
        assert!(one_floor(&up, &floor_of(in_house(-2.0, 6.4, 3.0))));
        assert!(!one_floor(&up, &floor_of(in_house(1.0, 1.4, 0.0))));
        assert!(!one_floor(&up, &ground));

        // An upper floor modelled as two boxes, an L: the one floor, and a
        // thing dragged along it goes from one wing into the other, and
        // round the corner the L has not, along the wing nearer the finger.
        let storey = |low: Vec3, high: Vec3| Storey {
            into: house.inverse(),
            low,
            high,
            top: 0.27 + high.y,
        };
        let l = [
            storey(Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.1, 6.0)),
            storey(Vec3::new(-5.5, 5.0, 0.0), Vec3::new(5.5, 5.1, 5.5)),
            storey(Vec3::new(0.0, 5.0, -5.5), Vec3::new(5.5, 5.1, 0.0)),
        ];
        let floor_of = |at: Vec3| upstairs(yard, &l, at.xz(), at.y + crate::LAND_STEP_UP);
        let wing = floor_of(in_house(-3.0, 5.1, 3.0));
        assert_eq!(wing.len(), 2);
        assert!(one_floor(&wing, &floor_of(in_house(3.0, 5.1, -3.0))));
        let turn = Quat::from_rotation_y(FRAC_PI_4);
        let own = |at: Vec3| house.inverse().transform_point3(at);
        let drag = |goal: Vec3| own(drag_freely(yard, in_house(-3.0, 5.1, 3.0), goal.xz(), STOVE, turn, &[], &l, |_| true));
        let at = drag(in_house(3.0, 5.1, -3.0));
        assert!((at - Vec3::new(3.0, 5.1, -3.0)).length() < 1e-3, "{at}");
        let at = drag(in_house(-3.0, 5.1, -1.0));
        assert!((at - Vec3::new(-3.0, 5.1, 0.98)).length() < 1e-3, "{at}");
    }

    /// A lamp a metre square and a metre high, its origin under its middle,
    /// to hang from a ceiling.
    const LAMP: Bounds = Bounds {
        low: Vec3::new(-0.5, 0.0, -0.5),
        high: Vec3::new(0.5, 1.0, 0.5),
    };

    #[test]
    fn something_hung_upstairs_stays_upstairs_over_the_stairwell_and_past_the_walls() {
        let (yard, house, storeys, island) = two_floors_built();
        let in_house = |x: f32, y: f32, z: f32| house.transform_point3(Vec3::new(x, y, z));
        let own = |at: Vec3| house.inverse().transform_point3(at);
        let close = |a: Vec3, b: Vec3| (a - b).length() < 1e-3;
        let turn = Quat::from_rotation_y(FRAC_PI_4);
        // Hung from the ceiling over the upper floor, its top against it,
        // ten metres up; moved about under it.
        let from = in_house(1.0, 9.0, 0.0);
        let drag = |to: Vec3| own(drag_hung(&island, yard, &storeys, from, to, LAMP, turn));
        assert!(close(drag(in_house(2.0, 9.0, 1.0)), Vec3::new(2.0, 9.0, 1.0)));
        // Over the stairwell: at its edge, from that ceiling still; where by
        // the floor under its middle, the ground floor, it hung from what
        // is over that there, the stair, under 3 m up.
        let reach = from.y + crate::LAND_STEP_UP;
        let was = hang_at(&island, &storeys, in_house(-4.5, 9.0, 0.0), LAMP, turn, reach);
        assert!(was.is_some_and(|was| was.y < 3.0), "{was:?}");
        let at = drag(in_house(-4.5, 9.0, 0.0));
        assert!(close(at, Vec3::new(-3.0, 9.0, 0.0)), "{at}");
        // Past a wall: against it, inside.
        let at = drag(in_house(8.0, 9.0, -2.0));
        assert!(close(at, Vec3::new(5.0, 9.0, -2.0)), "{at}");
        // Brought out upstairs facing the stairwell: the same.
        let you = in_house(-2.4, 5.1, 0.0);
        let at = own(hung_up(&island, Some(yard), &storeys, you, in_house(-4.4, 5.1, 0.0), LAMP, turn));
        assert!(close(at, Vec3::new(-3.0, 9.0, 0.0)), "{at}");
        // And on the ground floor, as before: from the ceiling over it.
        let at = own(hung_up(&island, Some(yard), &storeys, in_house(1.0, 0.1, 0.0), in_house(2.0, 0.1, 0.0), LAMP, turn));
        assert!(close(at, Vec3::new(2.0, 4.0, 0.0)), "{at}");
    }

    #[test]
    fn a_thing_brought_out_upstairs_comes_up_on_that_floor_and_not_down_the_stairwell() {
        let (yard, house, storeys, island) = two_floors_built();
        let in_house = |x: f32, y: f32, z: f32| house.transform_point3(Vec3::new(x, y, z));
        let own = |at: Vec3| house.inverse().transform_point3(at);
        let turn = Quat::from_rotation_y(FRAC_PI_4);
        // Upstairs near the stairwell, facing it.
        let (you, spot) = (in_house(-2.4, 5.1, 0.0), in_house(-4.6, 5.1, 0.0));
        // Bound by nothing, as it was, and as House Builder's pieces still
        // are: down there, on the stairs.
        let there = ahead_of(&island, Some(yard), you, spot, STOVE, turn, None);
        assert!(own(there).y < 3.0, "{}", own(there));
        // On your plot of the town: on your floor, the whole of it over it,
        // at the edge of the stairwell.
        let at = own(comes_up(&island, Some(yard), &storeys, you, spot, STOVE, turn));
        assert!((at - Vec3::new(-3.5 + 0.675, 5.1, 0.0)).length() < 1e-3, "{at}");
        // On the ground floor and the land, as before.
        let (you, spot) = (in_house(1.0, 0.1, 0.0), in_house(3.0, 0.1, 0.0));
        assert_eq!(comes_up(&island, Some(yard), &storeys, you, spot, STOVE, turn), ahead_of(&island, Some(yard), you, spot, STOVE, turn, None));
        let (you, spot) = (in_house(9.0, 0.0, 0.0), in_house(11.0, 0.0, 0.0));
        assert_eq!(comes_up(&island, Some(yard), &storeys, you, spot, STOVE, turn), ahead_of(&island, Some(yard), you, spot, STOVE, turn, None));
    }


    #[test]
    fn a_house_s_walls_are_numbered_the_same_however_little_their_corners_are_off() {
        // Hajun's square house, the low corner of each wall: three of them
        // begin at the same place across it.
        let walls = [
            ("back", Vec3::new(-6.0, 0.1, 5.5)),
            ("front", Vec3::new(-6.0, 0.1, -6.0)),
            ("left", Vec3::new(5.5, 0.1, -5.5)),
            ("right", Vec3::new(-6.0, 0.1, -5.5)),
        ];
        let numbered = |off: [f32; 4]| {
            let mut walls: Vec<(&str, Vec3)> = walls.iter().zip(off).map(|(&(name, low), off)| (name, low + Vec3::X * off)).collect();
            walls.sort_by_key(|&(_, low)| wall_order(low));
            walls.into_iter().map(|(name, _)| name).collect::<Vec<_>>()
        };
        // As it is modelled, and with each corner a few millionths of a
        // metre off this way or that, as it came out worked through the
        // world: the front wall, the right, the back and the left.
        let offs = [[0.0; 4], [0.0, 2e-6, 0.0, 0.0], [-5e-7, 0.0, 0.0, -5e-7], [3e-6, -2e-6, 1e-6, 4e-6], [-4e-5, 5e-5, 0.0, -3e-5]];
        for off in offs {
            assert_eq!(numbered(off), ["front", "right", "back", "left"], "{off:?}");
        }
        // By the numbers to their last bit, as they were until 2026-11-22,
        // the front wall of the house on one plot of the town, its corner
        // -5.999998 across, came after the two beside it: a door kept as
        // in the first wall was in the front one on one device and in the
        // right-hand one on another.
        let mut by_bits: Vec<(&str, Vec3)> = walls.iter().map(|&(name, low)| (name, low + Vec3::X * if name == "front" { 2e-6 } else { 0.0 })).collect();
        by_bits.sort_by(|a, b| a.1.x.total_cmp(&b.1.x).then(a.1.z.total_cmp(&b.1.z)));
        assert_eq!(by_bits.iter().map(|(name, _)| *name).collect::<Vec<_>>(), ["right", "back", "front", "left"]);
    }

    #[test]
    fn two_desks_go_together_flush_and_in_line() {
        let (island, desk) = desk_down(DESK_EDGE, Quat::IDENTITY, Vec3::ZERO);
        let drag = |from: Vec3, goal: Vec2, bounds: Bounds, turn: Quat| {
            drag_over_floor(&island, None, from, goal, bounds, turn, &[desk])
        };
        let near = |at: Vec3, want: Vec3| (at - want).length() < 1e-4;
        // The corner pushed into the end of the edge, a little off line: flush
        // against it and in line with it.
        let at = drag(Vec3::new(3.0, 0.0, 0.1), Vec2::new(0.0, 0.1), DESK_CORNER, Quat::IDENTITY);
        assert!(near(at, Vec3::new(1.5, 0.0, 0.0)), "{at}");
        // Brought up to it and not pushed, the same.
        let at = drag(Vec3::new(3.0, 0.0, 0.0), Vec2::new(1.62, -0.15), DESK_CORNER, Quat::IDENTITY);
        assert!(near(at, Vec3::new(1.5, 0.0, 0.0)), "{at}");
        // Slid along its front, it goes level with either end, or its middle,
        // whichever is nearest, and in between it stays where it is put.
        for (x, want) in [(0.4, 0.5), (0.1, 0.0), (-0.65, -0.5), (0.25, 0.25)] {
            let at = drag(Vec3::new(x, 0.0, -3.0), Vec2::new(x, 0.0), DESK_CORNER, Quat::IDENTITY);
            assert!(near(at, Vec3::new(want, 0.0, -1.0)), "{x}: {at}");
        }
        // Another edge, turned to run front to back, against its front: square
        // to it, so flush against it and level with its end.
        let across = Quat::from_rotation_y(FRAC_PI_2);
        let at = drag(Vec3::new(0.45, 0.0, -4.0), Vec2::new(0.45, 0.0), DESK_EDGE, across);
        assert!(near(at, Vec3::new(0.5, 0.0, -1.5)), "{at}");
        // Turned half that, it is not square to it, and is left where it is
        // put; and so is one far enough off.
        let slant = Quat::from_rotation_y(FRAC_PI_4);
        let at = drag(Vec3::new(4.0, 0.0, 0.0), Vec2::new(1.75, 0.0), DESK_CORNER, slant);
        assert!(near(at, Vec3::new(1.75, 0.0, 0.0)), "{at}");
        let at = drag(Vec3::new(4.0, 0.0, 0.0), Vec2::new(1.8, 0.1), DESK_CORNER, Quat::IDENTITY);
        assert!(near(at, Vec3::new(1.8, 0.0, 0.1)), "{at}");
    }

    #[test]
    fn turned_desks_go_together_and_slide_along_each_other() {
        // Both turned half a right angle, in a house that is.
        let slant = Quat::from_rotation_y(FRAC_PI_4);
        let (island, desk) = desk_down(DESK_EDGE, slant, Vec3::ZERO);
        let drag = |from: Vec3, goal: Vec2| {
            drag_over_floor(&island, None, from, goal, DESK_CORNER, slant, &[desk])
        };
        // In the desks' own space: the corner out past the edge's end, pushed
        // in at it, and lined up with it.
        let world = |x: f32, z: f32| slant * Vec3::new(x, 0.0, z);
        let at = drag(world(3.0, 0.15), world(0.0, 0.15).xz());
        assert!((at - world(1.5, 0.0)).length() < 1e-4, "{at}");
        // Pushed slantwise into the edge's front, it slides along it, flush,
        // to about where the finger is level with.
        let at = drag(world(-0.25, -3.0), world(0.25, 2.0).xz());
        let mine = slant.inverse() * at;
        assert!((mine.z + 1.0).abs() < 1e-4, "{mine}");
        assert!((mine.x - 0.25).abs() < 0.05, "{mine}");
    }

    #[test]
    fn a_door_opens_sooner_from_outside_than_from_inside() {
        // A door in the origin, the house toward +z, 3.35 m tall.
        let near = |x: f32, y: f32, z: f32, hold: f32| {
            opens(Vec3::new(x, y, z), Vec3::ZERO, Vec3::Z, 3.35, hold)
        };
        // Two and a half metres out, coming in, it opens, but not three; as
        // far in, going out, it does not, not until a metre from it.
        assert!(near(0.0, 0.0, -2.5, 0.0));
        assert!(!near(0.0, 0.0, -3.1, 0.0));
        assert!(!near(0.0, 0.0, 2.5, 0.0));
        assert!(near(0.0, 0.0, 1.0, 0.0));
        // Walking past it outside, as near as coming straight at it.
        assert!(near(2.0, 0.0, -2.0, 0.0));
        // Further off than it opens at, it stays open a little way on.
        assert!(!near(0.0, 0.0, -3.3, 0.0));
        assert!(near(0.0, 0.0, -3.3, SWING_HOLD));
        // Upstairs, over it, is not near it.
        assert!(!near(0.0, 6.1, -1.0, 0.0));
    }

    #[test]
    fn a_double_door_opens_for_either_leaf_as_a_single_door_would() {
        // Hajun's double doors, the house toward +z: two leaves 1.8 m wide,
        // their middles 0.9 m either side of the doorway's.
        let near = |x: f32, z: f32| {
            let at = Vec3::new(x, 0.0, z);
            opens(at, nearest_leaf(at, Vec3::ZERO, Vec3::X, 0.9), Vec3::Z, 3.35, 0.0)
        };
        for x in [-0.9, 0.9] {
            // Going out through either leaf, or coming in, it opens where a
            // door of one leaf does for someone coming straight at it.
            assert!(near(x, 1.4) && !near(x, 1.6), "{x}");
            assert!(near(x, -2.9) && !near(x, -3.1), "{x}");
        }
        // Going out right beside the frame, too: from the middle of the
        // doorway, that is too far to the side to open it at all.
        assert!(near(1.8, 1.1));
        // A door of one leaf is as near as the middle of its doorway.
        let at = Vec3::new(0.9, 0.0, 1.2);
        assert_eq!(nearest_leaf(at, Vec3::ZERO, Vec3::X, 0.0), Vec3::ZERO);
    }

    #[test]
    fn every_leaf_swings_out_of_the_house() {
        // In a door's own space the house is toward +z. Hajun's doors are
        // hung on the left seen from outside, Blender's -x, +x here, the leaf
        // reaching across the doorway; their double doors have another leaf
        // hung on the right, and each reaches to the middle. Hinged a little
        // further out than the middle of its leaf.
        for (hinge, middle) in [(0.9, 0.0), (1.8, 0.9), (-1.8, -0.9)] {
            let far = Vec3::new(middle - hinge, 0.0, 0.05);
            let open = Quat::from_rotation_y(outward(far, Vec3::NEG_Z) * 90f32.to_radians());
            let edge = open * (far * 2.0);
            assert!(edge.z < -1.0, "hinged at {hinge}: {edge}");
        }
        // The classic door turns as it always has: -90 degrees about
        // Blender's Z.
        assert_eq!(outward(Vec3::new(-0.9, 0.0, 0.05), Vec3::NEG_Z), -1.0);
    }

    #[test]
    fn inside_is_over_a_floor_and_under_the_roof() {
        // Two floors, like the L-shaped house's: the corner between its wings
        // is in the box round the house, but outside it.
        let rooms = Rooms(vec![
            (Vec3::new(-3.7, 0.0, 0.3), Vec3::new(3.7, 0.1, 3.7)),
            (Vec3::new(0.3, 0.0, -3.7), Vec3::new(3.7, 0.1, 0.3)),
        ]);
        let top = 4.9;
        assert!(rooms.hold(Vec3::new(-2.0, 0.1, 2.0), top));
        assert!(rooms.hold(Vec3::new(2.0, 0.1, -2.0), top));
        assert!(!rooms.hold(Vec3::new(-2.0, 0.1, -2.0), top), "between the wings");
        assert!(!rooms.hold(Vec3::new(-2.0, 5.0, 2.0), top), "on the roof");
    }

    #[test]
    fn every_piece_has_its_files() {
        let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        for def in PIECES {
            assert!(
                assets.join(def.model()).exists(),
                "{} has no model",
                def.kind
            );
            assert!(
                assets.join(def.picture()).exists(),
                "{} has no picture",
                def.kind
            );
        }
    }

    #[test]
    fn no_piece_is_left_out_of_the_apk() {
        let list = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("mobile/android/app/assets-left-out.txt");
        let list = std::fs::read_to_string(list).expect("the APK's list of what it leaves out");
        let patterns: Vec<&str> = list
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        // As aapt matches them: the whole name, or a name with one `*` at its
        // start or its end, a folder's or a file's.
        let matches = |pattern: &str, name: &str| {
            let pattern = pattern.trim_start_matches("<dir>").trim_start_matches("<file>");
            if let Some(end) = pattern.strip_prefix('*') {
                name.ends_with(end)
            } else if let Some(start) = pattern.strip_suffix('*') {
                name.starts_with(start)
            } else {
                name == pattern
            }
        };
        for def in PIECES {
            for path in [def.model(), def.picture()] {
                for name in path.split('/') {
                    if let Some(pattern) = patterns.iter().find(|pattern| matches(pattern, name)) {
                        panic!(
                            "{path} is sold in the shop, but `{pattern}` in \
                             mobile/android/app/assets-left-out.txt leaves it out of the APK"
                        );
                    }
                }
            }
        }
        // Nor either of the town's cars, whichever drives its roads.
        for car in crate::car::CARS {
            for name in car.split('/') {
                if let Some(pattern) = patterns.iter().find(|pattern| matches(pattern, name)) {
                    panic!(
                        "{car} can drive round the town, but `{pattern}` in \
                         mobile/android/app/assets-left-out.txt leaves it out of the APK"
                    );
                }
            }
        }
        // Nor any car you drive, nor its picture in the shop.
        for car in crate::drive::CARS {
            for path in [car.model, car.picture] {
                for name in path.split('/') {
                    if let Some(pattern) = patterns.iter().find(|pattern| matches(pattern, name)) {
                        panic!(
                            "{path} is a car you drive, but `{pattern}` in \
                             mobile/android/app/assets-left-out.txt leaves it out of the APK"
                        );
                    }
                }
            }
        }
        // Nor the model every body is drawn with.
        for model in crate::lobby::models() {
            for name in model.split('/') {
                if let Some(pattern) = patterns.iter().find(|pattern| matches(pattern, name)) {
                    panic!(
                        "{model} is what every player is drawn with, but `{pattern}` in \
                         mobile/android/app/assets-left-out.txt leaves it out of the APK"
                    );
                }
            }
        }
        // Nor the music.
        for sound in [crate::sound::MUSIC] {
            for name in sound.split('/') {
                if let Some(pattern) = patterns.iter().find(|pattern| matches(pattern, name)) {
                    panic!(
                        "{sound} is played in the town, but `{pattern}` in \
                         mobile/android/app/assets-left-out.txt leaves it out of the APK"
                    );
                }
            }
        }
    }
}
