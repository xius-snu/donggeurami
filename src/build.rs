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
//! What is down is part of the plot from then on: stood on, bumped into and
//! kept out of by the camera like the island itself
//! (`island::Islands::build_on`). Doors, windows and what hangs on the walls are
//! not: the hole in the wall is what counts. When the time to build runs out,
//! whatever is still a ghost is put down where it is, and nothing can be
//! changed any more.
//!
//! The models, and the pictures the shop shows of them, are in `assets/build/`,
//! made by `blender/make_build_items.py`, which also says what the game
//! expects of each; or, for what Hajun models by hand, in `assets/mybuilds/`,
//! which they build to those same rules, so that the game loads them as they
//! are.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::math::Affine3A;
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::world_serialization::{WorldAsset, WorldInstanceReady};

use bevy_replicon::prelude::Remote;
use roundtown_net::{At, Built};

use crate::builder::{Building, InGame, OnlinePlot};
use crate::editor::{self, Edit, EditMenu, Editable, Touch, Untappable};
use crate::hud::WantsPointer;
use crate::island::{self, Island, Islands, Venue};
use crate::lobby::Seat;
use crate::shop::Chosen;
use crate::{EYE_HEIGHT, PLAYER_RADIUS, Player, ThirdPersonCamera, WATER_Y};

/// Everything the shop sells, in the order it shows them: only what Hajun
/// has modelled, in `assets/mybuilds/` (their call, 2026-10-03). The
/// script's pieces in `assets/build/` are no longer sold, nor packed into the
/// APK.
pub(crate) const PIECES: &[PieceDef] = &[
    my_house("square_house", "Square House"),
    my_house("square_house_roofless", "Flat House"),
    my_house("square_house_two_floor", "Two Floor House"),
    my_house("square_house_roofless_two_floor", "Roof Terrace House"),
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
    my_floor("threelegcircletable", "Three Leg Table", Tab::Furniture),
    my_floor("circletable", "Circle Table", Tab::Furniture),
    my_floor("cafedeskedge", "Cafe Desk Edge", Tab::Furniture),
    my_floor("cafedeskcorner", "Cafe Desk Corner", Tab::Furniture),
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
    /// Its files: `<dir>/<kind>.glb` and, for the shop, `<dir>/<kind>.png`.
    pub kind: &'static str,
    /// What the shop calls it. Plain ASCII: the built-in font has no more.
    pub name: &'static str,
    pub tab: Tab,
    pub mount: Mount,
    /// A part of it that swings open by itself, if one does.
    pub swing: Option<Swing>,
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
        format!("{}/{}.png", self.dir, self.kind)
    }

    fn model(&self) -> String {
        format!("{}/{}.glb", self.dir, self.kind)
    }

    /// Whether it goes on a wall of the house, so that there has to be one.
    pub(crate) fn on_wall(&self) -> bool {
        matches!(self.mount, Mount::Door | Mount::Window | Mount::Hanging)
    }

    /// Whether it cuts a hole in the wall it is on.
    fn holed(&self) -> bool {
        self.mount.holed()
    }

    /// Whether, once down, it is part of what is stood on and bumped into.
    fn solid(&self) -> bool {
        !self.on_wall()
    }
}

/// Made by `blender/make_build_items.py`.
const MADE: &str = "build";
/// Modelled by hand, to the rules the script's pieces keep.
const MINE: &str = "mybuilds";

const fn house(kind: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        dir: MADE,
        kind,
        name,
        tab: Tab::Foundation,
        mount: Mount::House,
        swing: None,
    }
}

/// A house Hajun modelled, in `assets/mybuilds/`.
const fn my_house(kind: &'static str, name: &'static str) -> PieceDef {
    PieceDef {
        dir: MINE,
        ..house(kind, name)
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
        name,
        tab: Tab::Openings,
        mount,
        swing: None,
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
        name,
        tab: Tab::Decorations,
        mount: Mount::Hanging,
        swing: None,
    }
}

const fn floor(kind: &'static str, name: &'static str, tab: Tab) -> PieceDef {
    PieceDef {
        dir: MADE,
        kind,
        name,
        tab,
        mount: Mount::Floor,
        swing: None,
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

    /// Whether the shop sells anything under it. One that sells nothing is
    /// not shown, as Decorations is not while none of Hajun's pieces is one.
    pub(crate) fn sold(self) -> bool {
        PIECES.iter().any(|def| def.tab == self)
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
struct Bounds {
    low: Vec3,
    high: Vec3,
}

/// A mesh of a ghost, and the material it has once it is down.
#[derive(Component)]
struct Solid(Handle<StandardMaterial>);

/// What is inside a house: the boxes its `Floor...` objects fill, in its own
/// space.
#[derive(Component)]
struct Rooms(Vec<(Vec3, Vec3)>);

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
struct OnWall {
    wall: Entity,
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
struct Wall {
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

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Reshape>()
        .init_resource::<NextPieceId>()
        .init_resource::<Untold>()
        .add_observer(took)
        .add_observer(rebuilt)
        .add_observer(unbuilt)
        .add_systems(Startup, stock_up)
        .add_systems(Update, untappable_from_inside.before(editor::EditSystems))
        .add_systems(Update, swing_doors.after(crate::move_bodies))
        .add_systems(
            Update,
            (
                take_chosen,
                place_fresh,
                edit_pieces,
                finish_building,
                report_pieces,
                raise_shadows,
                carve_walls,
                reshape_plot,
                want_pointer,
            )
                .chain()
                .after(editor::EditSystems),
        );
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
    pieces: Query<(Entity, &Piece), Without<Shadow>>,
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
        let mut piece = commands.spawn((
            Piece { def },
            Ghost,
            Fresh,
            InGame,
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
        if def.on_wall() {
            let Some(house) = house else {
                piece.despawn();
                continue;
            };
            // Its house carries it, and says where it is drawn.
            piece.insert((ChildOf(house), Transform::default()));
        } else {
            let at = me.single().map(|me| me.translation).unwrap_or_default();
            piece.insert((Venue::Plot(plot), Transform::from_translation(at)));
        }
        piece.observe(piece_ready);
        menu.open_on(piece.id());
    }
}

/// Once a piece's model has spawned: how big it is, which is what a tap on it
/// is tested against, a house's walls made again to have holes cut in them,
/// a door's swinging leaves, and a ghost house's see-through look.
fn piece_ready(
    ready: On<WorldInstanceReady>,
    pieces: Query<(&Piece, Option<&Editable>, Has<Ghost>)>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    parts: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>)>,
    turns: Query<&Transform>,
    placement: TransformHelper,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stock: ResMut<Stock>,
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    let root = ready.entity;
    let Ok((piece, editable, ghost)) = pieces.get(root) else {
        return;
    };
    let Ok(root_place) = placement.compute_global_transform(root) else {
        return;
    };
    let into_root = root_place.affine().inverse();
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
    for part in children.iter_descendants(root) {
        let Ok((mesh, material)) = parts.get(part) else {
            continue;
        };
        let Some((low, high)) = meshes
            .get(&mesh.0)
            .zip(placement.compute_global_transform(part).ok())
            .and_then(|(mesh, place)| extent(mesh, into_root * place.affine()))
        else {
            continue;
        };
        filled.insert(part, (low, high));
        bounds = grow(bounds, low, high);
        if called(part, "Frame") {
            frame = grow(frame, low, high);
        }
        if is_house && called(part, "Wall") {
            walls.push((part, low, high, material.map(|material| material.0.clone())));
            continue;
        }
        if is_house && called(part, "Floor") {
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

    // Each wall again, as a box of the same size that holes can be cut in, in
    // the house's own space.
    let against: Vec<(Vec3, Vec3)> = walls
        .iter()
        .map(|&(_, low, high, _)| (low, high))
        .chain(fixtures)
        .collect();
    for (part, low, high, material) in walls {
        commands.entity(part).despawn();
        let mut wall = Wall::new(low, high, &floors, &against);
        let mesh = meshes.add(wall_mesh(&wall));
        wall.mesh = mesh.clone();
        let material = material.unwrap_or_default();
        let mut made = commands.spawn((ChildOf(root), wall, Mesh3d(mesh), Transform::IDENTITY));
        if ghost {
            made.insert((
                MeshMaterial3d(stock.ghost_of(&material, &mut materials)),
                Solid(material),
                NotShadowCaster,
            ));
        } else {
            made.insert(MeshMaterial3d(material));
        }
    }
    if is_house {
        commands.entity(root).insert(Rooms(floors));
    }
    // A door's leaves that swing: the parts called what its piece says, each
    // turned as it hangs shut, and opening about its hinge whichever way
    // takes it out of the house, -Z here.
    if let Some(swing) = piece.def.swing {
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
                .fold(None, |sum, &(low, high)| grow(sum, low, high));
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
        if !leaves.is_empty() {
            commands.entity(root).insert(Doorway {
                leaves,
                reach,
                through: 0.0,
                wanted: false,
            });
        }
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
    building: Res<Building>,
    islands: Res<Islands>,
    mut commands: Commands,
) {
    let (Some(plot), Ok(me), Ok(eye)) = (building.0, me.single(), eye.single()) else {
        return;
    };
    let venue = Venue::Plot(plot);
    let ahead = eye.forward().with_y(0.0).normalize_or(Vec3::NEG_Z);
    for (piece, kind, bounds, span, mut transform, house) in &mut fresh {
        commands.entity(piece).remove::<Fresh>();
        match kind.def.mount {
            Mount::Floor | Mount::House => {
                // Its front, Blender's -Y, toward you, and its near side a
                // step out of your way; a house's, a good few steps.
                let turn = facing(-ahead);
                let gap = if kind.def.mount == Mount::House {
                    HOUSE_IN_FRONT
                } else {
                    IN_FRONT
                };
                let spot = me.translation + ahead * (PLAYER_RADIUS + gap - bounds.low.z);
                let at = if kind.def.mount == Mount::House {
                    islands
                        .ground(venue)
                        .and_then(|ground| house_near(ground, *bounds, turn, spot.xz()))
                        .unwrap_or(spot)
                } else {
                    islands.get(venue).map_or(spot, |island| {
                        ahead_of(island, me.translation, spot, *bounds, turn)
                    })
                };
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

/// Where a piece of `bounds`, turned by `turn`, comes up for someone standing
/// at `you` who would have it at `spot`: carried from them toward there as far
/// as the floor goes and no walls are in the way, then back toward them until
/// the whole of it is clear of the walls too — facing a wall, in the room
/// rather than through it.
fn ahead_of(island: &Island, you: Vec3, spot: Vec3, bounds: Bounds, turn: Quat) -> Vec3 {
    let point = Bounds {
        low: Vec3::ZERO,
        high: Vec3::ZERO,
    };
    let reached = editor::carry(island, you, spot.xz(), turn, |at| {
        !walled(island, at, point, turn)
    });
    const TRIES: u32 = 12;
    (0..=TRIES)
        .map(|back| reached.xz().lerp(you.xz(), back as f32 / TRIES as f32))
        .filter_map(|at| {
            island
                .floor(at, you.y + crate::LAND_STEP_UP)
                .map(|y| Vec3::new(at.x, y, at.y))
        })
        .find(|&at| !walled(island, at, bounds, turn))
        .unwrap_or(reached)
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
    let (bottom, top) = (at.y + crate::LAND_STEP_UP, at.y + crate::PLAYER_HEIGHT);
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
/// be in, if there is nowhere near enough.
fn turned_clear(island: &Island, at: Vec3, bounds: Bounds, turn: Quat, stops: &[Stop]) -> Vec3 {
    let fits = |spot: Vec3| !walled(island, spot, bounds, turn);
    if fits(at) {
        return at;
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
/// walls of the house there, if it is down.
fn stops_on(
    pieces: &Moving,
    walls: &Query<(Entity, &Wall, &ChildOf)>,
    venue: Venue,
    carried: Entity,
) -> Vec<Stop> {
    let down = |on: Option<&Venue>, ghost: bool| !ghost && on == Some(&venue);
    let furniture = pieces
        .iter()
        .filter(|&(entity, piece, .., on, ghost)| {
            entity != carried && piece.def.mount == Mount::Floor && down(on, ghost)
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
/// it is clear to stand there.
fn drag_over_floor(
    island: &Island,
    from: Vec3,
    goal: Vec2,
    bounds: Bounds,
    turn: Quat,
    stops: &[Stop],
) -> Vec3 {
    let fits = |at| !walled(island, at, bounds, turn);
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
/// nearest [`TURN_STEP_DEG`].
fn facing(way: Vec3) -> Quat {
    let step = TURN_STEP_DEG.to_radians();
    let yaw = (-way.x).atan2(-way.z);
    Quat::from_rotation_y((yaw / step).round() * step)
}

/// Where near `spot` a house of `bounds`, turned by `turn`, fits: there, or
/// failing that as little further in toward the middle of the plot as it
/// takes.
fn house_near(ground: &Island, bounds: Bounds, turn: Quat, spot: Vec2) -> Option<Vec3> {
    let inward = (-spot).normalize_or_zero();
    let reach = spot.length();
    (0..)
        .map(|step| step as f32 * 0.5)
        .take_while(|&back| back <= reach)
        .map(|back| spot + inward * back)
        .find_map(|at| house_fits(ground, bounds, turn, at).map(|y| Vec3::new(at.x, y, at.y)))
}

/// Where a house of `bounds`, turned by `turn`, dragged from `from` toward
/// `goal`, gets to: as far along the way as the whole of it still fits on the
/// land, in short steps, sliding along the shore where it cannot go straight
/// on. One that does not fit where it is follows the finger freely until it
/// does.
fn house_toward(ground: &Island, bounds: Bounds, turn: Quat, from: Vec3, goal: Vec2) -> Vec3 {
    let fit = |at: Vec2| house_fits(ground, bounds, turn, at).map(|y| Vec3::new(at.x, y, at.y));
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
fn edit_pieces(
    mut edits: MessageReader<Edit>,
    building: Res<Building>,
    islands: Res<Islands>,
    mut stock: ResMut<Stock>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pieces: Moving,
    homes: Query<&GlobalTransform>,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    hung: Hung,
    children: Query<&Children>,
    looks: Query<(&MeshMaterial3d<StandardMaterial>, Has<Solid>)>,
    solids: Query<&Solid>,
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    let Some(plot) = building.0 else {
        edits.clear();
        return;
    };
    let venue = Venue::Plot(plot);
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
                    .is_ok_and(|(_, piece, ..)| piece.def.mount == Mount::Floor)
                    .then(|| stops_on(&pieces, &walls, venue, target))
                    .unwrap_or_default();
                let Ok((_, piece, mut transform, house, bounds, span, grip, ..)) =
                    pieces.get_mut(target)
                else {
                    continue;
                };
                match piece.def.mount {
                    Mount::Floor => {
                        let (Some(to), Some(island), Some(&bounds)) =
                            (to, islands.get(venue), bounds)
                        else {
                            continue;
                        };
                        let at = drag_over_floor(
                            island,
                            transform.translation,
                            to.xz(),
                            bounds,
                            transform.rotation,
                            &stops,
                        );
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
                        let at = house_toward(
                            ground,
                            bounds,
                            transform.rotation,
                            transform.translation,
                            to.xz(),
                        );
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
                match (piece.def.mount, bounds) {
                    // A house turns only where it still fits on the land.
                    (Mount::House, Some(&bounds)) => {
                        let fits = islands.ground(venue).is_some_and(|ground| {
                            house_fits(ground, bounds, turned, at.xz()).is_some()
                        });
                        if fits {
                            transform.rotation = turned;
                        }
                    }
                    // Anything else always turns, out of a wall or another
                    // piece it would turn into.
                    (Mount::Floor, Some(&bounds)) => {
                        if let Some(island) = islands.get(venue) {
                            transform.translation = turned_clear(island, at, bounds, turned, &stops);
                        }
                        transform.rotation = turned;
                    }
                    (Mount::House | Mount::Floor, None) => transform.rotation = turned,
                    _ => {}
                }
            }
            Edit::Confirm(target) => {
                if pieces.contains(target) {
                    unghost(target, &children, &solids, &mut commands);
                    commands.entity(target).remove::<(Ghost, Fresh)>();
                    reshape.0 = true;
                }
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
    pieces: Query<(Entity, Has<Ghost>), (With<Piece>, With<Editable>)>,
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
/// and the doors, windows and hangings in its walls, still can be.
fn untappable_from_inside(
    me: Query<(&Transform, &Venue), With<Player>>,
    houses: Query<(
        Entity,
        &Venue,
        &GlobalTransform,
        &Rooms,
        &Bounds,
        Has<Untappable>,
    )>,
    mut commands: Commands,
) {
    let me = me.single().ok();
    for (house, venue, place, rooms, bounds, untappable) in &houses {
        let inside = me.is_some_and(|(me, on)| {
            let at = place.affine().inverse().transform_point3(me.translation);
            on == venue && rooms.hold(at, bounds.high.y)
        });
        if inside && !untappable {
            commands.entity(house).insert(Untappable);
        } else if !inside && untappable {
            commands.entity(house).remove::<Untappable>();
        }
    }
}

/// On desktop the cursor is let go for as long as something is being put
/// down, to drag it with and to press its buttons.
fn want_pointer(
    building: Res<Building>,
    ghosts: Query<(), With<Ghost>>,
    mut wants: ResMut<WantsPointer>,
) {
    wants.set_if_neq(WantsPointer(building.0.is_some() && !ghosts.is_empty()));
}

// ---------------------------------------------------------------- the doors

/// Swings open each door with swinging leaves while anyone is near it, and
/// shut once nobody is: nearer it from inside, going out, than from outside,
/// coming in. Anyone is everyone on its plot, AIs as well, on its storey. A
/// door being put down stays shut. A double door's leaves swing together, and
/// open for someone going through either of them as soon as a door of one
/// leaf would.
fn swing_doors(
    time: Res<Time>,
    bodies: Query<(&Transform, &Venue), With<Seat>>,
    mut doors: Query<(&mut Doorway, &GlobalTransform, &Span, &ChildOf, Has<Ghost>)>,
    venues: Query<&Venue>,
    mut leaves: Query<&mut Transform, Without<Seat>>,
) {
    let dt = time.delta_secs();
    for (mut door, place, span, house, ghost) in &mut doors {
        let venue = venues.get(house.parent()).ok();
        // The middle of the bottom of the doorway, which way is in, and which
        // way along it.
        let (middle, inward, along) = (
            place.translation(),
            place.rotation() * Vec3::Z,
            place.rotation() * Vec3::X,
        );
        let hold = if door.wanted { SWING_HOLD } else { 0.0 };
        let wanted = !ghost
            && bodies.iter().any(|(body, on)| {
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
/// change.
fn carve_walls(
    mut walls: Query<(Entity, &mut Wall)>,
    openings: Query<(&OnWall, &Span, &Piece)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut wall) in &mut walls {
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
        let _ = meshes.insert(&wall.mesh, wall_mesh(&wall));
    }
}

/// What fills the hole a window taking up `span` cuts in `wall`, in the
/// window's own space: its glass, as far as bodies and the camera go. Drawn,
/// the hole is open; to them it is shut, so that nobody comes in through a
/// window, however low it is or however high they jump.
fn pane(span: Span, wall: &Wall) -> Mesh {
    let thick = wall.thin().1;
    let low = Vec3::new(-span.half, -span.below, -thick);
    let high = Vec3::new(span.half, span.above, thick);
    Mesh::from(Cuboid::from_corners(low, high)).translated_by((low + high) * 0.5)
}

/// What shuts the hole a door taking up `span` cuts to the camera, the door
/// standing at `place`: a sheet across the hole, halfway through the wall.
/// Bodies walk through it; the camera, which would otherwise go out through
/// the doorway of a house you are in, and in through it when you are out,
/// stays on your side of it, as it does of the walls.
fn screen(span: Span, place: &GlobalTransform) -> [[Vec3; 3]; 2] {
    let corner = |x: f32, y: f32| place.transform_point(Vec3::new(x, y, 0.0));
    let (a, b) = (corner(-span.half, -span.below), corner(span.half, -span.below));
    let (c, d) = (corner(span.half, span.above), corner(-span.half, span.above));
    [[a, b, c], [a, c, d]]
}

/// A wall as a mesh: the boxes [`carve`] leaves of it.
fn wall_mesh(wall: &Wall) -> Mesh {
    let (middle, half) = wall.thin();
    let parts = carve(wall.face(), &wall.holes);
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

/// Reads the plot's shape again when something on it has been put down,
/// picked up or thrown away: the island, and every piece that is down on it
/// and solid, with its walls as they are cut now.
fn reshape_plot(
    mut reshape: ResMut<Reshape>,
    mut gone: RemovedComponents<Piece>,
    tops: Query<(Entity, &Piece, &Venue), Without<Ghost>>,
    children: Query<&Children>,
    ghosts: Query<(), With<Ghost>>,
    kinds: Query<&Piece>,
    panes: Query<(&OnWall, &Span)>,
    walls: Query<&Wall>,
    parts: Query<&Mesh3d>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    mut islands: ResMut<Islands>,
) {
    if gone.read().count() > 0 {
        reshape.0 = true;
    }
    if !std::mem::take(&mut reshape.0) {
        return;
    }
    // Each plot's triangles, a list to each piece, and those only the camera
    // meets.
    let mut plots: Vec<(Venue, Vec<Vec<[Vec3; 3]>>, Vec<[Vec3; 3]>)> = Vec::new();
    for (root, piece, &venue) in &tops {
        if !piece.def.solid() {
            continue;
        }
        let at = match plots.iter().position(|(on, ..)| *on == venue) {
            Some(at) => at,
            None => {
                plots.push((venue, Vec::new(), Vec::new()));
                plots.len() - 1
            }
        };
        let (_, pieces, screens) = &mut plots[at];
        pieces.push(Vec::new());
        let Some(corners) = pieces.last_mut() else {
            continue;
        };
        // Down through it, leaving out a ghost, and what goes on the walls:
        // all but the glass of a window, which fills its hole, and what
        // shuts a doorway to the camera.
        let mut next = vec![root];
        while let Some(entity) = next.pop() {
            let left_out = entity != root
                && (ghosts.contains(entity)
                    || kinds.get(entity).is_ok_and(|kind| !kind.def.solid()));
            if left_out {
                if !ghosts.contains(entity)
                    && let Ok(kind) = kinds.get(entity)
                    && let Ok((on, span)) = panes.get(entity)
                    && let Ok(wall) = walls.get(on.wall)
                    && let Ok(place) = placement.compute_global_transform(entity)
                {
                    match kind.def.mount {
                        Mount::Window => {
                            island::add_mesh(corners, &pane(*span, wall), &place);
                        }
                        Mount::Door => screens.extend(screen(*span, &place)),
                        _ => {}
                    }
                }
                continue;
            }
            if let Ok(part) = parts.get(entity)
                && let Some(mesh) = meshes.get(&part.0)
                && let Ok(place) = placement.compute_global_transform(entity)
            {
                island::add_mesh(corners, mesh, &place);
            }
            if let Ok(below) = children.get(entity) {
                next.extend(below);
            }
        }
    }
    for venue in islands.built_on() {
        if !plots.iter().any(|(on, ..)| *on == venue) {
            islands.build_on(venue, &[], &[]);
        }
    }
    for (venue, pieces, screens) in plots {
        islands.build_on(venue, &pieces, &screens);
    }
}

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
        Without<Shadow>,
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
/// every device, which their entities are not.
fn walls_of(house: Entity, walls: &Query<(Entity, &Wall, &ChildOf)>) -> Vec<Entity> {
    let mut of: Vec<(Entity, Vec3)> = walls
        .iter()
        .filter(|(.., parent)| parent.parent() == house)
        .map(|(entity, wall, _)| (entity, wall.low))
        .collect();
    of.sort_by(|a, b| {
        a.1.x
            .total_cmp(&b.1.x)
            .then(a.1.z.total_cmp(&b.1.z))
            .then(a.1.y.total_cmp(&b.1.y))
    });
    of.into_iter().map(|(entity, _)| entity).collect()
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
    fn the_camera_stays_on_your_side_of_a_doorway() {
        // A classic door in the middle of a front wall, the house toward +z.
        let mut wall = two_floor_wall(12.1);
        let on = at(0.0, 0.1, 1.0);
        wall.holes = vec![[-CLASSIC.half, on.up, CLASSIC.half, on.up + CLASSIC.above]];
        let mut corners = Vec::new();
        island::add_mesh(&mut corners, &wall_mesh(&wall), &GlobalTransform::IDENTITY);
        let screens = screen(CLASSIC, &GlobalTransform::from(wall.place(true, on)));
        let open = Island::new_for_tests(&corners);
        let shut = Island::screened_for_tests(&corners, &screens);
        // Someone just inside it, with the camera wanting to be behind them,
        // out through the doorway.
        let (you, behind) = (Vec3::new(0.0, 1.2, -4.5), Vec3::new(0.0, 2.0, -12.0));
        assert!(open.sweep(you, behind, 0.2).is_none(), "the doorway alone lets it out");
        let far = shut.sweep(you, behind, 0.2).expect("the screen keeps it in");
        let stops = you + (behind - you).normalize() * far;
        assert!(stops.z > -5.75 && stops.z < -5.5, "{stops}");
        // And anyone walks through it.
        let feet = on.up;
        let doorway = Vec2::new(0.0, -5.75);
        let (low, high) = (feet + crate::LAND_STEP_UP, feet + crate::PLAYER_HEIGHT);
        assert!(shut.walls(doorway, 0.0, low, high).next().is_none());
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
            let at = drag_over_floor(&island, Vec3::ZERO, goal, bounds, Quat::IDENTITY, &[wall]);
            assert!((at.x - flush).abs() < 1e-4, "{goal}: {at}");
        }
        // Not from further off.
        let at = drag_over_floor(&island, Vec3::ZERO, Vec2::new(1.2, 0.0), bounds, Quat::IDENTITY, &[wall]);
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
        let out = turned_clear(&island, at, DESK_EDGE, turned, &[]);
        assert!(!walled(&island, out, DESK_EDGE, turned), "{out}");
        let corner = FRAC_PI_4.cos() * 1.0 + FRAC_PI_4.sin() * 0.5;
        assert!((out.x - (2.0 - corner)).abs() < 0.005, "{out}");
        assert!(out.z.abs() < 1e-3 && out.y == 0.0, "{out}");
        // Clear where it is, it turns where it is.
        let free = Vec3::new(-3.0, 0.0, 0.0);
        assert_eq!(turned_clear(&island, free, DESK_EDGE, turned, &[]), free);
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
        let out = turned_clear(&island, at, DESK_CORNER, turned, &[desk]);
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
        let across = turned_clear(&corridor, Vec3::ZERO, DESK_EDGE, Quat::IDENTITY, &[]);
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

    #[test]
    fn two_desks_go_together_flush_and_in_line() {
        let (island, desk) = desk_down(DESK_EDGE, Quat::IDENTITY, Vec3::ZERO);
        let drag = |from: Vec3, goal: Vec2, bounds: Bounds, turn: Quat| {
            drag_over_floor(&island, from, goal, bounds, turn, &[desk])
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
            drag_over_floor(&island, from, goal, DESK_CORNER, slant, &[desk])
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
    }
}
