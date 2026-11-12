//! Bridges built as you walk, in a game of Bridge Wars (`arena`; since
//! 2026-11-08, Hajun: "i want to add a 'autobridge' feature ... the user
//! need to build the bridge by pressing the bridge icon (maybe the 3rd
//! button above the sword icon) and it is a toggle button that enables
//! bridging mode vs normal mode. when u toggle it on, the charater should
//! hold a small plank flat in front of him with his hands like holding a
//! baby style ... and when u are in bridging mode and u walk off the edge
//! of the island, it will place small planks beneath your ground at the
//! target direction, and the bridge can curve if you walk angularly, and
//! also the vertices of the plank is gonna match perfectly with the edge
//! of the island/edge of the bridge if ur bridging from an existing
//! bridge, and the bridge width should be same as the 4 bridges in the map
//! now, and same thickness too"). The map's own bridges went that day;
//! every bridge is built.
//!
//! The plank over the sword ([`hud::BridgeButton`], or B on desktop) turns
//! it on and off ([`Bridging`]). On, you carry a plank in front of you
//! (`Carrying`, the arms held out, with a board lying across them,
//! [`HeldPlank`]), walk at half the pace ([`CARRY_SLOW`]), throw no punch
//! and show no hit range (2026-11-09); and the bridge follows your feet
//! ([`lay`]): a chain of rungs, a point and a heading each, with a plank
//! between every two ([`Chain`], [`Rung`]), a quad [`HALF_WIDTH`] out
//! either side of each rung and [`THICKNESS`] deep, its top at [`TOP`].
//!
//! **The bridge is a ribbon laid under where you walk** (2026-11-09, after
//! Hajun found the first version "weird and choppy" at the edges and in a
//! sharp turn: "think of a smart way to make briding feel good and look
//! good no matter what"). The chain you are building has a tip plank that
//! is not fixed yet: it runs from the last fixed rung to a little past
//! you ([`AHEAD`]), re-aimed at you every frame, so that whichever way you
//! step over the void there is a plank under you, and grows as you walk;
//! each time you are a plank's length ([`SPACING`]) past the last fixed
//! rung, a rung is fixed on the line toward you, so that the fixed planks
//! trace the path you actually walked, with a mitred joint at each rung,
//! turned no more than [`MAX_TURN`] a rung so that no plank folds. A chain
//! begins where you leave the ground, a rung on its very edge headed
//! straight out from it with its corners pulled onto the edge, or where you
//! step off the side of a plank, a rung flush against that side (a fork);
//! and where you step onto land it ends on the land's edge, a rung headed
//! into it. Walking back onto the ground you came from with nothing built
//! yet forgets the stub. Turning building off, or dying, fixes the tip
//! where it is.
//!
//! **A chain's root and its end are cut along the edge they meet, at
//! whatever slant** (2026-11-10, after Hajun's two screenshots of the first
//! ribbon: "unwanted gaps and bridge formation when the tilt angle is a bit
//! too large, especially when branching off of an existing bridge, and also
//! sometime when approaching the island at a weird angle (not tangent) then
//! it seemed to not connect properly to the island"). The first plank heads
//! where you walk, up to [`MAX_SLANT`] off straight out of the edge, and its
//! root corners lie on the edge of the ground it leaves ([`corners_of`]: a
//! corner over the void pushed back onto it, one over another bridge's
//! plank pulled in to that plank's edge, which a fork's first plank is made
//! long enough for, [`first_length`]); until then it stood square to the
//! edge and bent a plank later, a stub and a bend off the side of a bridge
//! with a wedge of void between. A fork's root is square to the plank's own
//! nearest edge ([`Chain::edge_normal`]), an island's to the shore felt for
//! round it ([`outward`]). Landing, a chain comes in no more than
//! [`MAX_END_SLANT`] off straight into the shore, turning toward it a rung
//! at a time first if it must ([`land_on`]), its end corners pushed up to
//! [`END_PUSH`] to meet the shore or pulled back onto a bridge's plank, and
//! the plank that lands runs at least [`MIN_LAND`] into the ground, under
//! the land, so that its end edge crosses the shore wherever its corners
//! fall. Heading at ground within a plank past the next rung, along the
//! plank or along your way, no rung is fixed ([`STRETCH`]): the tip
//! stretches to the ground, so that the plank that lands is a whole one.
//! The rungs aim [`LEAD`] ahead of you along your way, so that after a
//! sharp turn they settle onto your new line rather than swing past it.
//! Turning back over your own planks and off their side ends the chain
//! where it is and begins a fork there ([`Bridges::abandon`]; the tip
//! stayed frozen where you had turned back, and off your own bridge's side
//! there was nothing under you), and off them onto land or another bridge
//! ends it with no landing. **A turn too sharp for the tip to follow makes
//! an L corner** (later that day, Hajun: "it happens when i do a close to
//! 90 degree turn"): stepping off the tip through its side
//! ([`Chain::tip_side_exit`]), the tip is fixed as the chain's last plank,
//! cut off flush with the far edge of the fork that begins off its side
//! ([`Bridges::corner`]), so that nothing sticks out past the fork and
//! nothing of the fork hangs past the plank; and a fork landing within a
//! plank or two slants no more than that length can cut cleanly
//! ([`land_on`]), a square gusset rather than a triangle.
//!
//! **A bridge that is built stays as it was built** (later still that day,
//! Hajun, with four screenshots of an unfinished end reaching out for a
//! bridge built past it: "suddenly that unfinished end's vertices try to
//! connect to my current bridge. i think bridge that is already made should
//! not change"). A chain's corners are worked out when the chain itself
//! changes, a rung fixed or come from the server, or its tip moved, against
//! the ground as it stands then, and never again because another chain has
//! changed ([`Chain::stale`], [`Chain::cut`], [`Bridges::recut`]); its root
//! is cut for good once its third rung is fixed ([`ROOT_KEPT`]). Until then
//! every chain was worked out again, against the land and every other
//! chain's planks as they then stood, each time anything was fixed or a tip
//! had moved a little, so that whatever was built near an old root or end
//! afterwards recut it. And only an end that lands is cut along what it
//! lands on: one left over the void, let go of, turned back from or ended
//! at a corner, is square where it was left ([`corners_of`]); its corners
//! were pushed on like a landing's, up to [`END_PUSH`], and an end left
//! within that of a shore or a bridge grew a spike out to it.
//!
//! **A root and an end follow the edge they meet all the way across**
//! (the same day, Hajun: "fix the sliver gap on curved forks too"). Each
//! was one straight line between its two corners, and only the corners
//! were cut onto the edge: a chord across an edge that bends. Off the
//! inside of a curve of bridge that left void between the fork and the
//! bridge, a sliver on a gentle curve and 0.4 m deep in the middle on one
//! turning 20 degrees a plank, under whoever stepped off there; off the
//! outside the fork lay over the bridge instead. Now the line is tried
//! all the way across ([`cut_along`]), each place on it pushed or pulled
//! onto the edge as the corners are, more of them wherever the edge
//! bends, and the first plank's near edge, or the last one's far edge,
//! follows what it finds ([`Edge`]), the plank built in strips there
//! ([`strips`]); a straight edge is the one quad it always was. Each
//! place is set a centimetre and a half in under the ground it meets
//! ([`TUCK`]), so that no hairline is left along the cut.
//!
//! Every plank costs its length in metres of bridge, which you buy (Hajun:
//! "players will buy the bridge by meter count ... for now make each person
//! start with 100m"; 500 since 2026-11-10: "increase the default bridge
//! meter to 500m instead of 100"; and since 2026-11-11 none to start, and
//! 50 for every 4 silver at the shop, `market`): the file's
//! `arena.bridge_metres` to start (`arena::Resources`; by the log top left
//! until 2026-11-11, when the log went for what the generators make), and
//! the tip never reaches further than you have left. **The same number is written
//! in front of you while you build, and running out stops you** (later on
//! 2026-11-10, Hajun: "i need another number indicator in front of where
//! ur bridging that is the same '### m' number of bridge wood left, and
//! it should be white font but orange -> red when its 10m->5m left. and
//! when it become 0m, it should exit bridge mode (not able to go back to
//! bridge mode when u dont have any bridging m left) and pause (even if
//! the user is holding the joystick, so the user need to let go of the
//! screen and do the joystick again to move) so that they dont fall off
//! when their bridge ends"): [`show_left`] writes it [`LEFT_AHEAD`] ahead
//! of you the way you face, white, and orange shading to red from the
//! file's `arena.bridge_low_at` down to `bridge_red_at` ([`left_colour`]);
//! and the frame the plank being laid can reach no further for want of
//! metres ([`Grown::spent`]), or what is left is what is written "0m"
//! ([`OUT_BELOW`]), [`lay`] fixes what is being built, a plank's worth of
//! it still ahead of you, takes the metres to none, turns building off,
//! which [`toggle`] then keeps off, and stops you where you stand until
//! the stick has been let go ([`RanOut`], `read_player_input`). Until
//! then the tip ran out under you and you walked off the end of it; and
//! with less than a plank's worth left no chain began at all, building on
//! or not, where now one begins with anything that is not written "0m",
//! a stub that stops you at the edge.
//!
//! The planks are ground: their shape is laid over the island's
//! (`Islands::lay_over`), so that you, the bots and everyone stand on them
//! and fall off their sides. Online every rung you fix goes to the server
//! as it is fixed (`Ask::Rung`), and everyone's come back as [`Plank`]s,
//! from which the same bridges are built here ([`hear_planks`]), yours left
//! out; the tip is yours alone until it is fixed. Each chain is one mesh,
//! built again as it changes ([`rebuild`]), the planks' tops alternating
//! between the file's `colours.arena.plank` and a shade lighter, their
//! sides and undersides in `plank_side`, the ends open but for a tip's cap.
//! The width is Hajun's bridges' (2.726 m, their `bridge` object of
//! 2026-11-08, 0.25 m thick); a plank is half as long as it is wide, like
//! the boards of a dock, which follows a curve closely.

use std::f32::consts::{FRAC_PI_4, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::{Ask as NetAsk, Plank};

use crate::arena::{Fighting, InArena, Resources};
use crate::hud::{self, BridgeButton, DialogUp};
use crate::island::{Islands, Venue};
use crate::net::Online;
use crate::ui::{UiChanged, ui};
use crate::{Carrying, Dead, Intent, LAND_STEP_UP, MOVE_SPEED, PLAYER_RADIUS, Player, ThirdPersonCamera};

mod builder_bot;
pub(crate) use builder_bot::send_builder;

/// Half the width of a plank, in metres: Hajun's bridges in
/// `pvptownmap.glb` were 2.726 m wide. The computer's players lay theirs
/// by the same measures, which the server has too (`roundtown_net::nav`).
pub(crate) const HALF_WIDTH: f32 = roundtown_net::nav::PLANK_HALF_WIDTH;
/// How thick a plank is, in metres: Hajun's bridges' 0.25.
pub(crate) const THICKNESS: f32 = 0.25;
/// How far one fixed rung is from the next, in metres: half as far as a
/// plank is wide, so that the planks follow a curve closely, like the
/// boards of a dock.
pub(crate) const SPACING: f32 = roundtown_net::nav::PLANK_SPACING;
/// How high the top of every plank lies, in metres: the islands' tops,
/// 0.25 m up, less a seam, so that where a plank's root overlaps the
/// island the island shows, and the join is the island's own edge.
pub(crate) const TOP: f32 = roundtown_net::nav::PLANK_TOP;
/// How much slower you walk carrying a plank, as a share of your pace.
pub(crate) const CARRY_SLOW: f32 = 0.5;
/// The most a chain turns from one rung to the next, in radians: 35
/// degrees. The joint between two planks is mitred ([`corners_of`]), and
/// a mitre pulls each inner corner back along its plank by
/// `HALF_WIDTH * tan(turn / 2)`; a plank with a mitre at both ends folds,
/// its inner edge gone, once twice that passes [`SPACING`], and the first
/// plank, square to the edge it leaves, once `HALF_WIDTH * sin(turn)` and
/// one mitre do: 35 degrees leaves it a tenth of its length. A chain turns
/// round in about 2.3 m. The rung that lands a chain is clamped by it too.
const MAX_TURN: f32 = 0.6109;
/// The most the first plank of a chain slants from straight out of the
/// edge it leaves, in radians: 45 degrees (2026-11-10). The first plank
/// heads where you walk, and its root is cut along the edge it leaves
/// ([`corners_of`]), which pulls the corner on the inside of the slant
/// `HALF_WIDTH * tan(slant)` along the plank: a plank's width at 45
/// degrees. Until then the first plank stood square to the edge, and the
/// turn toward where you walked came a plank later: walking off a bridge
/// at a slant made a stub out of its side and a bend after it (Hajun:
/// "unwanted gaps and bridge formation when the tilt angle is a bit too
/// large, especially when branching off of an existing bridge").
const MAX_SLANT: f32 = FRAC_PI_4;
/// The most the plank that lands a chain slants from straight into the
/// shore, in radians: 50 degrees. Coming in at more than that, the chain
/// turns toward the shore first ([`land_on`]): the corner of a plank
/// landing at a slant has to reach `HALF_WIDTH * tan(slant)` further to
/// meet the shore, and on the round islands further still; past about 60
/// degrees it misses a 10 m island altogether, and the end was left
/// hanging off the shore (Hajun: "when approaching the island at a weird
/// angle ... it seemed to not connect properly").
const MAX_END_SLANT: f32 = 0.8727;
/// How many rungs a landing may add, each turned toward the shore, before
/// the plank that lands.
const LANDING_TURNS: usize = 3;
/// How far past your middle the tip plank reaches, in metres: your own
/// width, and a little, so that a plank is under your legs before they are
/// over the void. The tip is never shorter, however few metres are left.
const AHEAD: f32 = PLAYER_RADIUS + 0.6;
/// The shortest plank that lands a chain, or is left when a chain is let
/// go, in metres: shorter, the mitre at its start would reach past its
/// end. A chain landing nearer the shore than that runs that far into it
/// instead, under the land ([`land_on`]), so that its end edge always
/// crosses the shore; for a night it ended on its last rung, with its
/// corners pushed onto the shore, which missed the shore when the last
/// plank pointed past it.
const MIN_LAND: f32 = SPACING * 0.5;
/// How far a root's corner is pushed back to meet the edge the chain
/// leaves, at the most, in metres.
const ONTO_EDGE: f32 = 2.0 * SPACING;
/// How far past the next rung ground has to be, along the way to you, for
/// the rung to be fixed, in metres: nearer, the tip stretches on to the
/// ground instead ([`grow`]), so that the plank that lands is at least a
/// plank long, with room for the mitre at its start and its cut along the
/// edge it lands on. And how far ahead of a chain's last rung the shore is
/// looked for as it lands ([`LAND_REACH`]): as far as a stretched tip
/// reaches, a fork's first plank and that.
const STRETCH: f32 = SPACING;
const LAND_REACH: f32 = 4.5;
/// How far an end's corner is pushed on to meet the shore, at the most, in
/// metres: at [`MAX_END_SLANT`] on a straight shore a corner has 1.6 m to
/// go, and on the round islands up to about 2.7.
const END_PUSH: f32 = 3.0;
/// How much of a plank's length is left when its root or its end is cut
/// along an edge, as a share, past the mitre at its other end, so that no
/// plank folds; and a little room on top of the cut, in metres, which the
/// first plank of a fork is made long enough for ([`first_length`]).
const CUT_SHARE: f32 = 0.95;
const SLANT_ROOM: f32 = 0.1;
/// How far past the far edge of a fork the plank it leaves at an L corner
/// runs, in metres ([`Bridges::corner`]): a hair, so that the fork's far
/// root corner, cut back onto the plank's side, finds the plank there
/// rather than grazing its very corner and missing.
const CORNER_ROOM: f32 = 0.05;
/// How many fixed rungs a chain has to have for the cut of its root to be
/// kept for good ([`Chain::cut`]): three, its first plank and the joint at
/// that plank's far end both fixed. How far a root's corner may be pulled
/// in along the first plank goes by how far that joint's mitre reaches
/// back along it, which is not settled while the second plank is the tip,
/// swinging after its builder.
const ROOT_KEPT: usize = 3;
/// How a root or an end is cut along the edge of the ground it meets
/// ([`cut_along`]): at so many places across the plank to begin with,
/// evenly apart, and between any two of them again, up to so many times
/// over, wherever the edge halfway between them is found [`BENT`] metres
/// off the straight line from the one to the other; of all of those, the
/// ones it takes to follow the edge within [`STRAIGHT`] are kept, none for
/// a straight edge. Each is set [`TUCK`] in under the ground it meets, so
/// that no hairline of void is left along the cut: under the land, whose
/// top is higher, or a centimetre and a half under or over another
/// bridge's plank, whose top is a millimetre or more off this one's
/// ([`top_of`]). The tuck covers what the rest leaves: an edge is felt for
/// to 1.6 mm ([`EDGE`], halved in finding it); a joint of the edge near
/// one of two places, where halfway between them the edge is less than
/// `BENT` off the line, stands at most twice that off it, and one between
/// two places as near as they get, 4.3 cm, 7 mm at a joint of a bridge
/// turning as hard as it may; and `STRAIGHT`.
const STATIONS: usize = 8;
const REFINEMENTS: usize = 3;
const BENT: f32 = 0.003;
const STRAIGHT: f32 = 0.004;
const TUCK: f32 = 0.015;
/// How far from a chain's root, or from its last rung, the ground is ever
/// asked after in cutting it, in metres: as far as a corner is pushed or
/// pulled, and across the plank; other chains' planks further off than
/// that are left out of the asking ([`Bridges::recut`]).
const NEAR: f32 = 6.0;
/// How far off the first plank's line you may drift, in metres, before its
/// rung is fixed early, shorter than [`first_length`] asks, so that the
/// chain can turn after you: walking off a bridge nearly along its edge.
const DRIFT: f32 = 0.5 * HALF_WIDTH;
/// How far inside a plank's edge your middle has to be for you to be
/// standing on it, in metres, with a little room: a body stands on the
/// ground under its middle, or under its legs, 0.21 m out (`LEG_RADIUS`).
/// About to land with the tip stretched to the ground, you may drift that
/// near its side before a rung is fixed to turn it after you.
const STAND_IN: f32 = 0.3;
/// How far ahead of you, along the way you walk, the planks aim, in
/// metres: two planks. A rung fixed to one side of your line by a turn's
/// clamp aims the next at your line further on, which brings it onto the
/// line within a few degrees rather than swinging past it ([`grow`]).
const LEAD: f32 = 2.0 * SPACING;
/// How finely the shore is looked for ahead, in metres.
const SHORE_STEP: f32 = 0.1;
/// How far ahead of your middle the ground is looked at to begin a chain,
/// in metres, over where this frame's step takes you.
const LOOK: f32 = PLAYER_RADIUS + 0.4;
/// How far out past the edge a chain's root lies on you have to be, in
/// metres, to have left the ground it began on: the plank or the round
/// island behind it is all behind that line, so a little, for the edge
/// felt a hair off. Half a plank until 2026-11-10, which, stepping onto
/// another bridge half a metre from a fork's root at a slant, held the
/// landing back until the drift rule had fixed a rung on the way.
const GONE_OUT: f32 = 0.2;
/// How far back behind a chain's root you have to be, in metres, for a
/// chain with nothing built yet to be forgotten: further inland than
/// anyone begins one ([`LOOK`] and a step), or the chain is begun and
/// forgotten every other frame as you come up to the edge, its tip
/// flickering and a root sent to the server each time (so it was for a
/// day: a bridge's chain was the fifth begun).
const FORGET_BEHIND: f32 = LOOK + 0.7;
/// How finely an edge is felt for, in metres (2 cm until 2026-11-10, which
/// left a hairline of void along a cut now and then), and how far round a
/// point on it the ground is felt for which way is out.
const EDGE: f32 = 0.005;
const RING: f32 = 0.6;
/// How far the tip has to move, in metres, or turn, in radians, before the
/// planks' shape is laid over the island again, and the minimap drawn
/// again: the tip reaches [`AHEAD`] past you, so the shape may lag it by
/// this. The planks are drawn again whenever the tip moves at all.
const OVERLAY_MOVE: f32 = 0.2;
const OVERLAY_TURN: f32 = 0.05;
/// How many heights the planks' tops are spread over, a millimetre apart,
/// by chain, so that two chains crossing never flicker where they overlap
/// ([`top_of`]).
const TOPS: u32 = 8;
/// The most rungs one chain holds, and one game holds all told.
const MOST_RUNGS: usize = 600;
const MOST_CHAINS: usize = 200;
/// The plank you carry: how long, how thick and how wide, in metres, and
/// where it lies in front of you, up from your feet and forward.
const HELD: Vec3 = Vec3::new(1.1, 0.08, 0.5);
const HELD_AT: Vec3 = Vec3::new(0.0, 1.08, -0.56);
/// How much lighter every other plank's top is, as a share.
const LIGHTER: f32 = 0.08;
/// Under how many metres of bridge you have none, in metres: what is
/// written as "0m", the metres being written to the nearest whole one
/// (`Resources::metres`). With less, building is off and stays off ([`lay`],
/// [`toggle`]); with as much, a chain may begin, though it is a stub.
const OUT_BELOW: f32 = 0.5;
/// The metres left, written in front of you as you build ([`show_left`]):
/// how big, as a share of the short side of the screen; how far ahead of
/// your middle, in metres, past the end of the tip, which reaches
/// [`AHEAD`]; and how long the "0m" stays there after you have run out,
/// or have pressed the plank button with none left, in seconds.
const LEFT_TEXT: f32 = 3.6;
const LEFT_AHEAD: f32 = 2.4;
const OUT_SHOWN: f32 = 1.5;

/// Whether you are building a bridge as you walk: the plank button's
/// toggle.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct Bridging(pub bool);

/// The plank button pressed, or B on desktop.
#[derive(Message, Default)]
pub(crate) struct BridgePressed;

/// What running out of bridge has done to you (2026-11-10, Hajun: "when it
/// become 0m, it should exit bridge mode (not able to go back to bridge
/// mode when u dont have any bridging m left) and pause (even if the user
/// is holding the joystick, so the user need to let go of the screen and
/// do the joystick again to move) so that they dont fall off when their
/// bridge ends").
#[derive(Resource, Default)]
pub(crate) struct RanOut {
    /// You stand where the bridge ran out under you, whatever the stick
    /// says, until the stick has been let go (`read_player_input`).
    pub(crate) halted: bool,
    /// How much longer the "0m" stays written in front of you, in seconds.
    shown: f32,
}

/// One rung of a chain: where it lies across the ground, and which way the
/// chain heads there.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Rung {
    pub(crate) p: Vec2,
    pub(crate) d: Vec2,
}

/// One bridge: whose it is, which of theirs, its fixed rungs in order, the
/// tip not fixed yet, and what is drawn of it.
struct Chain {
    seat: u8,
    id: u16,
    rungs: Vec<Rung>,
    /// The end of the plank being laid, past the last fixed rung, aimed at
    /// its builder: yours alone, never sent, fixed once you are a plank's
    /// length on.
    tip: Option<Rung>,
    /// How many rungs it had fixed, and the tip it had, when its corners
    /// were last worked out ([`Chain::cut`]): they are worked out again
    /// once either has changed, the chain itself, and never because
    /// another bridge has ([`Chain::stale`]). And the tip as last laid over
    /// the island, to lay it again once it has moved enough.
    built_rungs: usize,
    built_tip: Option<Rung>,
    overlaid_tip: Option<Rung>,
    /// Rungs that came ahead of their turn, online, to go on once those
    /// before them have.
    waiting: Vec<(u16, Rung)>,
    /// The corners of each rung, left and right, as last built, the tip's
    /// last: the plain ones, but for a root on an island's edge and an end
    /// on one, pulled onto the edge ([`shape_of`]).
    corners: Vec<(Vec2, Vec2)>,
    /// Its root's edge between its two corners, and its end's, as last
    /// built, where they follow a bent edge of the ground they meet rather
    /// than run straight across ([`Edge`]).
    root_edge: Edge,
    end_edge: Edge,
    /// Whether its last rung lands on ground, as last built.
    complete: bool,
    /// Its root as cut along the edge it left, its corners and the edge
    /// between them, kept from when its third rung was fixed
    /// ([`ROOT_KEPT`]) and never cut again.
    root_cut: Option<((Vec2, Vec2), Edge)>,
    /// The triangles bodies stand on, as last built ([`planks`]).
    triangles: Vec<[Vec3; 3]>,
    /// Whether its root is on another bridge's plank rather than on land:
    /// a fork, whose first plank is made long enough to be cut along the
    /// plank's edge ([`first_length`]). Yours alone.
    forked: bool,
    /// Whether its builder stood back over one of its fixed planks last
    /// frame, having turned back: off them onto land or another bridge the
    /// chain ends where it is, and off their side a fork begins (`lay`).
    /// Yours alone.
    behind: bool,
    mesh: Entity,
    handle: Handle<Mesh>,
}

impl Chain {
    fn new(seat: u8, id: u16, rungs: Vec<Rung>) -> Self {
        Self {
            seat,
            id,
            rungs,
            tip: None,
            built_rungs: usize::MAX,
            built_tip: None,
            overlaid_tip: None,
            waiting: Vec::new(),
            corners: Vec::new(),
            root_edge: Edge::new(),
            end_edge: Edge::new(),
            complete: false,
            root_cut: None,
            triangles: Vec::new(),
            forked: false,
            behind: false,
            mesh: Entity::PLACEHOLDER,
            handle: Handle::default(),
        }
    }

    /// Its rungs with the tip after them: what is built.
    fn all(&self) -> Vec<Rung> {
        self.rungs.iter().copied().chain(self.tip).collect()
    }

    /// Whether the plank `k`, between its `k`th and `k+1`th rungs as built,
    /// is a fixed one, rather than the tip's.
    fn fixed(&self, k: usize) -> bool {
        k + 1 < self.rungs.len()
    }

    /// Its fixed planks' tops as built, each a quad of its corners, the
    /// tip's left out: the ground another chain may begin on or land on.
    fn fixed_quads(&self) -> impl Iterator<Item = [Vec2; 4]> + '_ {
        (0..self.corners.len().saturating_sub(1)).filter(|&k| self.fixed(k)).map(|k| {
            let (l0, r0) = self.corners[k];
            let (l1, r1) = self.corners[k + 1];
            [l0, r0, r1, l1]
        })
    }

    /// Its fixed planks' tops as built, the tip's left out, the first and
    /// the last in strips where the root or the end follows a bent edge
    /// ([`strips`]): the ground another chain stands on, begins on or
    /// lands on, exactly as it is drawn.
    fn ground_quads(&self) -> impl Iterator<Item = [Vec2; 4]> + '_ {
        let n = self.corners.len();
        (0..n.saturating_sub(1)).filter(|&k| self.fixed(k)).flat_map(move |k| {
            let root: &[(f32, Vec2)] = if k == 0 { &self.root_edge } else { &[] };
            let end: &[(f32, Vec2)] = if k + 2 == n { &self.end_edge } else { &[] };
            let (near, far) = (self.corners[k], self.corners[k + 1]);
            // Nearly every plank is the one quad, and takes no list.
            let whole = (root.is_empty() && end.is_empty()).then_some([near.0, near.1, far.1, far.0]);
            let cut = if whole.is_none() { strips(near, root, far, end) } else { Vec::new() };
            whole.into_iter().chain(cut)
        })
    }

    /// Where a step from `from` to `to` leaves its tip plank, as last
    /// built, through one of the tip's two sides, and which way is out of
    /// that side: a turn too sharp for the tip to follow (`lay`). None
    /// with no tip built, or for a step that stays on it or leaves it
    /// over its end.
    fn tip_side_exit(&self, from: Vec2, to: Vec2) -> Option<(Vec2, Vec2)> {
        let n = self.corners.len();
        if self.tip.is_none() || self.built_tip.is_none() || n < 2 {
            return None;
        }
        let (l0, r0) = self.corners[n - 2];
        let (l1, r1) = self.corners[n - 1];
        let middle = (l0 + r0 + l1 + r1) * 0.25;
        [(l0, l1), (r0, r1)].into_iter().find_map(|(a, b)| {
            let e = crossing(from, to, a, b)?;
            let out = perp(b - a).normalize_or(Vec2::X);
            Some((e, if out.dot(a - middle) >= 0.0 { out } else { -out }))
        })
    }

    /// Which way is off its fixed planks at `e`, a point on their edge:
    /// square to whichever edge of theirs is nearest, outward; the sides,
    /// and the ends of the first and the last, never the joints between.
    /// None with no fixed plank.
    fn edge_normal(&self, e: Vec2) -> Option<Vec2> {
        let quads: Vec<[Vec2; 4]> = self.fixed_quads().collect();
        let mut best: Option<(f32, Vec2)> = None;
        for (k, quad) in quads.iter().enumerate() {
            let middle = (quad[0] + quad[1] + quad[2] + quad[3]) * 0.25;
            let mut edges = vec![(quad[1], quad[2]), (quad[3], quad[0])];
            if k == 0 {
                edges.push((quad[0], quad[1]));
            }
            if k + 1 == quads.len() {
                edges.push((quad[2], quad[3]));
            }
            for (a, b) in edges {
                let ab = b - a;
                let t = ((e - a).dot(ab) / ab.length_squared().max(1e-8)).clamp(0.0, 1.0);
                let gap = e.distance(a + ab * t);
                if best.is_none_or(|(nearest, _)| gap < nearest) {
                    let square = perp(ab).normalize_or(Vec2::X);
                    best = Some((gap, if square.dot(a - middle) >= 0.0 { square } else { -square }));
                }
            }
        }
        best.map(|(_, square)| square)
    }

    /// Whether it has changed since its corners were last worked out: a
    /// rung fixed, or come from the server, or its tip moved, dropped or
    /// fixed. Its rungs are only ever added to, so how many there are and
    /// the tip say all there is to say.
    fn stale(&self) -> bool {
        self.rungs.len() != self.built_rungs || self.tip != self.built_tip
    }

    /// Works its shape out again, against the ground as it is now, the
    /// `land` and every other bridge's planks ([`shape_of`]): called only
    /// once the chain itself has changed, so that a bridge that is built
    /// stays as it was built whatever is built beside it afterwards
    /// (2026-11-10, Hajun, of an unfinished end whose corners reached for
    /// a bridge built past it: "i think bridge that is already made should
    /// not change"). Its root is cut for good once its third rung is fixed
    /// ([`ROOT_KEPT`]), and kept from then on as the chain grows, not
    /// worked out again.
    fn cut(&mut self, land: &dyn Fn(Vec2) -> bool, ground: &dyn Fn(Vec2) -> bool) {
        let rungs = self.all();
        let shape = shape_of(&rungs, self.tip.is_some(), self.root_cut.as_ref(), land, ground);
        if self.root_cut.is_none() && self.rungs.len() >= ROOT_KEPT {
            self.root_cut = Some((shape.corners[0], shape.root.clone()));
        }
        self.corners = shape.corners;
        self.root_edge = shape.root;
        self.end_edge = shape.end;
        self.complete = shape.complete;
        self.built_rungs = self.rungs.len();
        self.built_tip = self.tip;
    }

    /// How high its planks' tops lie.
    fn top(&self) -> f32 {
        top_of(self.seat, self.id)
    }
}

/// How high the tops of `seat`'s `id`th chain lie, in metres: [`TOP`], or
/// up to [`TOPS`] millimetres under it, the same on every device, so that
/// where two chains cross one lies on the other rather than flickering
/// through it.
fn top_of(seat: u8, id: u16) -> f32 {
    TOP - 0.001 * ((u32::from(seat) * 3 + u32::from(id)) % TOPS) as f32
}

/// Every bridge in the game.
#[derive(Resource, Default)]
pub(crate) struct Bridges {
    chains: Vec<Chain>,
    next_id: u16,
    /// The chain you are building now, if any.
    active: Option<usize>,
    /// Something has changed since the planks were last built.
    dirty: bool,
    /// Counts up each time the planks are built again, for the minimap.
    pub(crate) version: u32,
    /// The game they belong to (`Fighting::game`): another game, and they
    /// are cleared.
    game: u32,
    /// The meshes of chains forgotten, to be taken down ([`rebuild`]).
    orphans: Vec<Entity>,
}

/// What became of the chain you were building, this frame ([`grow`]).
#[derive(Default, Debug, PartialEq)]
struct Grown {
    /// Rungs fixed, each with its index, to send.
    added: Vec<(u16, Rung)>,
    /// The chain ended on land.
    landed: bool,
    /// The chain is done with, landed or left.
    done: bool,
    /// The tip can no longer keep [`AHEAD`] of its builder for want of
    /// metres: the bridge ends here.
    spent: bool,
}

impl Bridges {
    /// The chain `seat`'s `id`th, if it is here.
    fn chain(&mut self, seat: u8, id: u16) -> Option<&mut Chain> {
        self.chains.iter_mut().find(|chain| chain.seat == seat && chain.id == id)
    }

    /// Takes in a rung that is not this device's player's to lay: the
    /// `index`th of the bridge numbered `chain` of whoever is in `seat`,
    /// as the server says someone has laid one ([`hear_planks`]), or as
    /// one of the computer's players lays its own in a game played
    /// offline. In order, one that comes early waiting for those before
    /// it ([`take`]).
    pub(crate) fn lay_remote(&mut self, seat: u8, chain: u16, index: u16, rung: Rung) {
        if self.chain(seat, chain).is_none() {
            if self.chains.len() >= MOST_CHAINS {
                return;
            }
            self.chains.push(Chain::new(seat, chain, Vec::new()));
        }
        let kept = self.chain(seat, chain).expect("just found or made");
        take(kept, index, rung);
        self.dirty = true;
    }

    /// Every rung fixed so far, of every bridge: whose it is, by seat,
    /// which of theirs, which of its rungs, and where. What the computer's
    /// players know of the bridges in a game played offline
    /// (`roundtown_net::nav::Nav::hear`).
    pub(crate) fn laid(&self) -> impl Iterator<Item = (u8, u16, u16, Vec2)> + '_ {
        self.chains.iter().flat_map(|chain| {
            chain
                .rungs
                .iter()
                .enumerate()
                .map(move |(index, rung)| (chain.seat, chain.id, index as u16, rung.p))
        })
    }

    /// Which chain has a fixed plank under `at`, across the ground, if
    /// any has. Never a tip.
    fn plank_under(&self, at: Vec2) -> Option<usize> {
        self.chains.iter().position(|chain| chain.ground_quads().any(|quad| inside_quad(at, quad)))
    }

    /// Every fixed plank of every chain but the `except`th, as quads: the
    /// ground a chain of yours may land on, its own planks left out (its
    /// last rung lies on its own last plank's end, and on its tip).
    fn ground_but(&self, except: usize) -> Vec<[Vec2; 4]> {
        self.chains
            .iter()
            .enumerate()
            .filter(|(c, _)| *c != except)
            .flat_map(|(_, chain)| chain.ground_quads())
            .collect()
    }

    /// Every plank's top, as a quad of its corners, tips included: for the
    /// minimap.
    pub(crate) fn quads(&self) -> impl Iterator<Item = [Vec2; 4]> + '_ {
        self.chains.iter().flat_map(|chain| {
            chain.corners.windows(2).map(|pair| {
                let ((l0, r0), (l1, r1)) = (pair[0], pair[1]);
                [l0, r0, r1, l1]
            })
        })
    }

    /// Forgets the `c`th chain altogether, one of nothing but its root:
    /// what was drawn of it, its tip, is taken down with it. (Until
    /// 2026-11-10 the chain alone went, and the plank its tip had drawn
    /// stayed where it was, out over the edge, with nothing to stand on.)
    fn forget(&mut self, c: usize) {
        let chain = self.chains.remove(c);
        if chain.mesh != Entity::PLACEHOLDER {
            self.orphans.push(chain.mesh);
        }
        self.dirty = true;
    }

    /// Works out again the corners of every chain that has itself changed
    /// since they last were ([`Chain::stale`], [`Chain::cut`]), each
    /// against the `land` and every other chain's planks as they stand,
    /// in the order the chains were begun, so that a fork come in the same
    /// frame as the bridge it leaves is cut against it; and says which.
    /// A chain that has not changed is left exactly as it was built.
    fn recut(&mut self, land: &dyn Fn(Vec2) -> bool) -> Vec<usize> {
        let mut cut = Vec::new();
        for c in 0..self.chains.len() {
            if !self.chains[c].stale() {
                continue;
            }
            // Only what is near its root or its end is ever asked after
            // (`NEAR`): the rest of a game's planks are left out of it.
            let ends: Vec<Vec2> = {
                let rungs = &self.chains[c].rungs;
                rungs.first().into_iter().chain(rungs.last()).map(|rung| rung.p).collect()
            };
            let others: Vec<[Vec2; 4]> = self
                .ground_but(c)
                .into_iter()
                .filter(|quad| {
                    let middle = (quad[0] + quad[1] + quad[2] + quad[3]) * 0.25;
                    let reach = quad.iter().map(|corner| corner.distance(middle)).fold(0.0, f32::max);
                    ends.iter().any(|end| end.distance(middle) <= NEAR + reach)
                })
                .collect();
            let ground = |at: Vec2| land(at) || others.iter().any(|&quad| inside_quad(at, quad));
            self.chains[c].cut(land, &ground);
            cut.push(c);
        }
        cut
    }

    /// Begins a chain of `seat`'s at `root`, a fork off another bridge if
    /// `forked`, and says which of theirs it is.
    fn begin(&mut self, seat: u8, root: Rung, forked: bool) -> u16 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let mut chain = Chain::new(seat, id, vec![root]);
        chain.forked = forked;
        self.chains.push(chain);
        self.active = Some(self.chains.len() - 1);
        id
    }

    /// Fixes the `c`th chain's tip as its last plank and ends the chain
    /// there, for a fork about to begin off its side: an L corner. Its
    /// builder stands at `here`, about to step off the tip through the side
    /// whose way out is `out`, walking `dir`. The plank runs straight on
    /// from the last rung the way the chain was heading before the turn
    /// (not the way the tip had swung toward it, which left a kink and a
    /// slanted cap), and the fork's root lies on that straight side level
    /// with `here`. The fork's first plank heads `dir`, within
    /// [`MAX_SLANT`] of square to the side, and its root edge, cut along
    /// the side, reaches `HALF_WIDTH / cos(slant)` either way from its root;
    /// the plank ends at the far one, and a hair past ([`CORNER_ROOM`]), so
    /// that nothing sticks out past the fork and nothing of the fork hangs
    /// past the plank. Charges `metres`; none with less than a plank's
    /// worth, or with no tip. The fork's root, and the rung fixed with the
    /// chain's id and its index, to send.
    fn corner(&mut self, c: usize, here: Vec2, out: Vec2, dir: Vec2, metres: &mut f32) -> Option<(Rung, (u16, u16, Rung))> {
        let chain = &mut self.chains[c];
        chain.tip?;
        let last = *chain.rungs.last()?;
        if chain.rungs.len() >= MOST_RUNGS {
            return None;
        }
        let stem = last.d;
        let square = perp(stem);
        let side = if square.dot(out) >= 0.0 { square } else { -square };
        let along = (here - last.p).dot(stem).max(0.0);
        let heading = clamp_turn(dir, side, MAX_SLANT);
        let span = HALF_WIDTH / heading.angle_to(side).cos().max(0.1);
        let length = (along + span + CORNER_ROOM).max(MIN_LAND).min(*metres);
        if length < MIN_LAND {
            return None;
        }
        *metres -= length;
        let rung = Rung { p: last.p + stem * length, d: stem };
        let index = chain.rungs.len() as u16;
        let id = chain.id;
        chain.rungs.push(rung);
        self.abandon(c);
        let root = Rung { p: last.p + stem * along + side * HALF_WIDTH, d: side };
        Some((root, (id, index, rung)))
    }

    /// Ends the `c`th chain, yours, where its fixed planks got to: its tip
    /// dropped, and a chain of nothing but its root forgotten.
    fn abandon(&mut self, c: usize) {
        let chain = &mut self.chains[c];
        chain.tip = None;
        chain.overlaid_tip = None;
        chain.behind = false;
        if chain.rungs.len() < 2 {
            self.forget(c);
        }
        self.active = None;
        self.dirty = true;
    }

    /// Lets go of the chain you are building: its tip fixed where it is, if
    /// it has one worth keeping, and a chain of nothing but its root
    /// forgotten. The rung fixed, if any, to send.
    fn let_go(&mut self, metres: &mut f32) -> Option<(u16, u16, Rung)> {
        let c = self.active.take()?;
        let chain = &mut self.chains[c];
        let id = chain.id;
        let mut grown = Grown::default();
        fix_tip(chain, metres, &mut grown);
        if chain.rungs.len() < 2 {
            self.forget(c);
        }
        self.dirty = true;
        grown.added.pop().map(|(index, rung)| (id, index, rung))
    }
}

/// Fixes `chain`'s tip where it is, if it is a plank's worth ([`MIN_LAND`])
/// and the chain has room, charging `metres`; a shorter tip is dropped.
fn fix_tip(chain: &mut Chain, metres: &mut f32, grown: &mut Grown) {
    chain.overlaid_tip = None;
    let Some(tip) = chain.tip.take() else {
        return;
    };
    let Some(last) = chain.rungs.last() else {
        return;
    };
    let length = last.p.distance(tip.p);
    if length >= MIN_LAND && chain.rungs.len() < MOST_RUNGS {
        *metres = (*metres - length).max(0.0);
        grown.added.push((chain.rungs.len() as u16, tip));
        chain.rungs.push(tip);
    }
}

/// The mesh of one chain.
#[derive(Component)]
struct PlankMesh;

/// The plank you carry, hung in front of you.
#[derive(Component)]
struct HeldPlank;

/// The plank in the arms of one of the computer's players while it lays
/// its bridge, in a game played offline (`arena::drive_bots` gives it
/// `Carrying`, and takes it away): which entity the plank is, to take it
/// down by.
#[derive(Component)]
struct BotPlank(Entity);

/// What the planks are drawn with: one material, coloured by vertex.
#[derive(Resource)]
struct Look(Handle<StandardMaterial>);

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<BridgePressed>()
        .init_resource::<Bridges>()
        .init_resource::<Bridging>()
        .init_resource::<RanOut>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (toggle, hear_planks).before(crate::read_player_input),
                (rebuild, carry, carry_for_bots, light_button).after(lay),
                // A Bridge Builder's builders, whose planks are built with
                // the rest.
                (builder_bot::equip, builder_bot::walk).chain().after(lay).before(rebuild),
                // The camera's place for this frame is set by then.
                show_left.after(crate::follow_camera),
            ),
        );
}

fn setup(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(Look(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    })));
}

/// Turns building on and off at the plank button, or B, in a game of Bridge
/// Wars while it is on; off anywhere else, and as a game ends. With no
/// metres of bridge left it is not turned on: the "0m" shows in front of
/// you for a moment instead ([`show_left`]).
#[allow(clippy::too_many_arguments)]
fn toggle(
    mut presses: MessageReader<BridgePressed>,
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Res<DialogUp>,
    fighting: Res<Fighting>,
    stock: Res<Resources>,
    me: Query<&Venue, With<Player>>,
    mut bridging: ResMut<Bridging>,
    mut ran_out: ResMut<RanOut>,
) {
    let pressed = presses.read().count() > 0 || keyboard.just_pressed(KeyCode::KeyB);
    let in_a_game = fighting.playing && me.single().is_ok_and(|&here| here == Venue::Arena);
    if !in_a_game {
        bridging.set_if_neq(Bridging(false));
        if ran_out.halted || ran_out.shown > 0.0 {
            *ran_out = RanOut::default();
        }
        return;
    }
    if pressed && !dialog.0 {
        if !bridging.0 && stock.bridge_metres < OUT_BELOW {
            ran_out.shown = OUT_SHOWN;
            info!("building a bridge: no metres left");
            return;
        }
        bridging.0 = !bridging.0;
        info!("building a bridge: {}", if bridging.0 { "on" } else { "off" });
    }
}

/// The colour the metres of bridge left are written in, in front of you:
/// `plenty` with more than `low_at` of them, `low` at `low_at`, shading to
/// `out` at `red_at` and under (Hajun: "white font but orange -> red when
/// its 10m->5m left"). By the number as it is written, to the nearest
/// whole metre.
fn left_colour(metres: f32, low_at: f32, red_at: f32, plenty: Color, low: Color, out: Color) -> Color {
    let written = metres.max(0.0).round();
    if written > low_at {
        plenty
    } else if written <= red_at || low_at <= red_at {
        out
    } else {
        let share = (low_at - written) / (low_at - red_at);
        Color::from(low.to_srgba().mix(&out.to_srgba(), share))
    }
}

/// The metres of bridge left, written in front of you as you build.
#[derive(Component)]
struct LeftAhead;

/// Writes the metres of bridge you have left in front of you while
/// building is on, [`LEFT_AHEAD`] ahead of you the way you face, past the
/// end of the plank being laid, where you are looking as you build
/// ("50m"; the number that was by the log top left until 2026-11-11, and is
/// written nowhere else now but in the shop), in the file's
/// `colours.arena.bridge_left`, white, and from `arena.bridge_low_at`
/// metres down to `arena.bridge_red_at` in `bridge_low` shading to
/// `bridge_out`, orange to red ([`left_colour`]; 2026-11-10, Hajun: "i
/// need another number indicator in front of where ur bridging that is the
/// same '### m' number of bridge wood left, and it should be white font
/// but orange -> red when its 10m->5m left"). Once you have run out, the
/// "0m" stays for as long as you stand stopped, and a moment; and it shows
/// for a moment when the plank button is pressed with none left. Yours
/// alone, like the plank you carry.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_left(
    time: Res<Time>,
    bridging: Res<Bridging>,
    fighting: Res<Fighting>,
    stock: Res<Resources>,
    mut ran_out: ResMut<RanOut>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    me: Query<(&Transform, &Venue, Has<Dead>), With<Player>>,
    mut written: Query<(&mut Text, &mut TextColor, &mut Visibility, &mut UiTransform, &ComputedNode), With<LeftAhead>>,
    mut commands: Commands,
) {
    if ran_out.shown > 0.0 {
        ran_out.shown = (ran_out.shown - time.delta_secs()).max(0.0);
    }
    let style = ui();
    let colours = style.colours.arena;
    let Ok((mut text, mut colour, mut shown, mut place, node)) = written.single_mut() else {
        if bridging.0 {
            commands.spawn((
                InArena,
                LeftAhead,
                Name::new("Metres of bridge left"),
                hud::words(String::new(), LEFT_TEXT, colours.bridge_left),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    top: Val::ZERO,
                    ..default()
                },
                UiTransform::default(),
                Visibility::Hidden,
            ));
        }
        return;
    };
    let you = me.single().ok().filter(|&(_, &venue, dead)| fighting.playing && venue == Venue::Arena && !dead);
    let wanted = bridging.0 || ran_out.halted || ran_out.shown > 0.0;
    let point = you.filter(|_| wanted).and_then(|(body, _, _)| {
        let (camera, eye) = cameras.single().ok()?;
        let facing = (body.rotation * Vec3::NEG_Z).with_y(0.0).normalize_or(Vec3::NEG_Z);
        let at = body.translation + facing * LEFT_AHEAD + Vec3::Y * TOP;
        camera.world_to_viewport(&GlobalTransform::from(*eye), at).ok()
    });
    let Some(point) = point else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    let metres = stock.metres();
    if text.0 != metres {
        text.0 = metres;
    }
    let arena = style.arena;
    colour.set_if_neq(TextColor(left_colour(
        stock.bridge_metres,
        arena.bridge_low_at,
        arena.bridge_red_at,
        colours.bridge_left,
        colours.bridge_low,
        colours.bridge_out,
    )));
    let size = node.size() * node.inverse_scale_factor();
    place.set_if_neq(UiTransform {
        translation: Val2::px(point.x - size.x * 0.5, point.y - size.y * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
    shown.set_if_neq(Visibility::Inherited);
}

/// Lights the plank button while building is on.
fn light_button(bridging: Res<Bridging>, mut buttons: Query<&mut BackgroundColor, With<BridgeButton>>) {
    let colours = ui().colours.hud;
    for mut fill in &mut buttons {
        fill.set_if_neq(BackgroundColor(if bridging.0 { colours.button_held } else { colours.button }));
    }
}

/// Has you carry a plank while building is on and you are alive, and not
/// otherwise: the arms held out (`Carrying`), and the board across them.
fn carry(
    bridging: Res<Bridging>,
    look: Res<Look>,
    me: Query<(Entity, Has<Dead>, Has<Carrying>), With<Player>>,
    mut held: Query<&mut Visibility, With<HeldPlank>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let Ok((you, dead, carrying)) = me.single() else {
        return;
    };
    let wanted = bridging.0 && !dead;
    if wanted != carrying {
        if wanted {
            commands.entity(you).insert(Carrying);
        } else {
            commands.entity(you).remove::<Carrying>();
        }
    }
    let shown = if wanted { Visibility::Inherited } else { Visibility::Hidden };
    match held.single_mut() {
        Ok(mut visibility) => {
            visibility.set_if_neq(shown);
        }
        Err(_) => {
            commands.entity(you).with_child((HeldPlank, Name::new("Held plank"), board(&mut meshes, &look), shown));
        }
    }
}

/// Puts a plank in the arms of each of the computer's players as it begins
/// to lay a bridge, in a game played offline, and takes it away as it
/// stops (since 2026-11-12): whoever carries one (`Carrying`) and is
/// neither you, whose plank is [`HeldPlank`], nor a Bridge Builder's
/// builder, a body for show with a plank of its own (`builder_bot::equip`).
#[allow(clippy::type_complexity)]
fn carry_for_bots(
    look: Res<Look>,
    begun: Query<Entity, (Added<Carrying>, Without<Player>, Without<crate::ForShow>, Without<BotPlank>)>,
    done: Query<(Entity, &BotPlank), Without<Carrying>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for body in &begun {
        let plank = commands
            .spawn((Name::new("Bot's plank"), board(&mut meshes, &look), ChildOf(body)))
            .id();
        commands.entity(body).insert(BotPlank(plank));
    }
    for (body, plank) in &done {
        commands.entity(plank.0).try_despawn();
        commands.entity(body).remove::<BotPlank>();
    }
}

/// The plank someone building carries, as it hangs in front of them: the
/// one in your arms ([`carry`]), the one in a builder's (`builder_bot`),
/// and the one in the arms of one of the computer's players
/// ([`carry_for_bots`]).
fn board(meshes: &mut Assets<Mesh>, look: &Look) -> impl Bundle {
    let colour = ui().colours.arena.plank.to_linear();
    let mut mesh = Mesh::from(Cuboid::new(HELD.x, HELD.y, HELD.z));
    let count = mesh.count_vertices();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[colour.red, colour.green, colour.blue, 1.0]; count]);
    (
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(look.0.clone()),
        Transform::from_translation(HELD_AT),
        NotShadowCaster,
        NotShadowReceiver,
    )
}

/// Lays the bridge under you as you build: before you move, so that it is
/// there when your feet are.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lay(
    time: Res<Time>,
    fighting: Res<Fighting>,
    online: Res<Online>,
    islands: Res<Islands>,
    mut bridging: ResMut<Bridging>,
    mut ran_out: ResMut<RanOut>,
    mut me: Query<(&Transform, &Venue, &mut Intent, Has<Dead>), With<Player>>,
    mut bridges: ResMut<Bridges>,
    mut stock: ResMut<Resources>,
    mut requests: MessageWriter<NetAsk>,
) {
    let Ok((body, &venue, mut intent, dead)) = me.single_mut() else {
        return;
    };
    let Some(seat) = fighting.seat else {
        return;
    };
    let mut sent: Vec<(u16, u16, Rung)> = Vec::new();
    let building = bridging.0 && fighting.playing && !dead && venue == Venue::Arena;
    // Dead, you are stopped no longer: you come back free to walk.
    if dead && ran_out.halted {
        ran_out.halted = false;
    }
    // The plank being laid can reach no further for want of metres.
    let mut spent = false;
    if !building {
        // Hands empty, or dead: whatever was being built is left as it is.
        if bridges.active.is_some()
            && let Some(fixed) = bridges.let_go(&mut stock.bridge_metres)
        {
            sent.push(fixed);
        }
    } else if let Some(island) = islands.get(Venue::Arena) {
        let feet = body.translation.y;
        let here = body.translation.xz();
        let reach = feet + LAND_STEP_UP;
        let ground = |at: Vec2| island.floor(at, reach).is_some();
        let land = |at: Vec2| island.floor_base(at, reach).is_some();
        // Where this frame's step, and a look ahead of it, takes you, if
        // you are walking.
        let probe = (intent.walk.length() >= 0.1).then(|| {
            let dir = intent.walk.xz().normalize();
            let speed = MOVE_SPEED * intent.sprint * intent.walk.length().min(1.0);
            (dir, here + dir * (speed * time.delta_secs() + LOOK))
        });
        // Where a fork begins this frame, if one does: its root, and the
        // way off the edge it begins on.
        let mut start: Option<(Vec2, Vec2)> = None;
        if let Some(c) = bridges.active {
            let under = bridges.plank_under(here);
            let on_own = under.map(|other| other == c);
            let was_behind = bridges.chains[c].behind;
            bridges.chains[c].behind = on_own == Some(true);
            // On the tip and about to step off its side: a turn too sharp
            // for the tip to follow. The tip is fixed as the chain's last
            // plank, cut off flush with the far edge of the fork that
            // begins off its side, below: an L corner (2026-11-10, after
            // Hajun's "close to 90 degree turn" screenshots; until then a
            // rung was fixed a plank on, past where you had turned, and
            // the fork began off that plank's side, leaving the rest of it
            // sticking out past the fork, and the fork's root cut hanging
            // past the plank's end).
            let corner = if on_own.is_none()
                && stock.bridge_metres >= SPACING + MIN_LAND
                && let Some((dir, probe)) = probe
                && let Some((_, out)) = bridges.chains[c].tip_side_exit(here, probe)
            {
                bridges.corner(c, here, out, dir, &mut stock.bridge_metres)
            } else {
                None
            };
            if let Some((root, (id, index, rung))) = corner {
                sent.push((id, index, rung));
                start = Some((root.p, root.d));
            } else if was_behind && (land(here) || on_own == Some(false)) {
                // Back over the chain's own planks and off them onto land
                // or another bridge: the chain ends where it got to, with
                // no landing (2026-11-10).
                bridges.abandon(c);
            } else if on_own == Some(true) && probe.is_some_and(|(_, at)| !ground(at)) {
                // Back over the chain's own planks and about to step off
                // their side: the chain ends where it got to, and a fork
                // begins off that side, below (2026-11-10; until then the
                // tip stayed frozen where you had turned back, and off the
                // side of your own bridge there was nothing under you).
                bridges.abandon(c);
            } else {
                let others = bridges.ground_but(c);
                let ground_but_mine = |at: Vec2| land(at) || others.iter().any(|&quad| inside_quad(at, quad));
                let heading = probe.map(|(dir, _)| dir);
                let grown = grow(&mut bridges.chains[c], here, heading, on_own, &mut stock.bridge_metres, &land, &ground_but_mine);
                let id = bridges.chains[c].id;
                sent.extend(grown.added.iter().map(|&(index, rung)| (id, index, rung)));
                spent = grown.spent && !grown.done;
                if grown.done {
                    bridges.active = None;
                    if bridges.chains[c].rungs.len() < 2 {
                        bridges.forget(c);
                    }
                    bridges.dirty = true;
                } else if !grown.added.is_empty() {
                    bridges.dirty = true;
                }
            }
        }
        if bridges.active.is_none()
            && let Some((dir, probe)) = probe
            && stock.bridge_metres >= OUT_BELOW
            && bridges.chains.len() < MOST_CHAINS
        {
            // Nothing being built, and about to step over the void: off
            // the edge of the ground, the island's or a plank's (a fork),
            // the root on the very edge where your way crosses it, headed
            // straight out of it (square to the plank's own edge, or to
            // the land's as felt for round it); or off the side of the
            // tip just fixed at a corner, above. The first plank heads
            // where you walk, within `MAX_SLANT` of that.
            if start.is_none() && ground(here) && !ground(probe) {
                let e = bisect(here, probe, &ground);
                let square = match bridges.plank_under(here) {
                    Some(c) => bridges.chains[c].edge_normal(e),
                    None => outward(e, &ground),
                };
                start = Some((e, square.unwrap_or(dir)));
            }
        } else {
            start = None;
        }
        if let Some((e, square)) = start
            && let Some((dir, _)) = probe
        {
            let root = Rung { p: e, d: square };
            let id = bridges.begin(seat, root, !land(here));
            sent.push((id, 0, root));
            let c = bridges.chains.len() - 1;
            // The tip, reaching past you from the start.
            let d = clamp_turn(dir, root.d, MAX_SLANT);
            let length = (AHEAD + (here - root.p).dot(d).max(0.0)).min(stock.bridge_metres.max(AHEAD));
            bridges.chains[c].tip = Some(Rung { p: root.p + d * length, d });
            bridges.dirty = true;
        }
    }
    // Out of bridge: the plank being laid can reach no further, or what
    // is left is what is written as "0m". Whatever is being built is
    // fixed where it is, a plank's worth of it still ahead of you (the
    // tip is never shorter than `AHEAD`); the metres are none; building
    // is off, and stays off (`toggle`); and you stop where you are,
    // whatever the stick says, until it has been let go
    // (`read_player_input`), so that you do not walk off the end of it
    // (2026-11-10, Hajun: "so that they dont fall off when their bridge
    // ends"). Until then the tip ran out under you and you walked off it.
    if building && (spent || stock.bridge_metres < OUT_BELOW) {
        if bridges.active.is_some()
            && let Some(fixed) = bridges.let_go(&mut stock.bridge_metres)
        {
            sent.push(fixed);
        }
        stock.bridge_metres = 0.0;
        bridging.0 = false;
        ran_out.halted = true;
        ran_out.shown = OUT_SHOWN;
        intent.walk = Vec3::ZERO;
        info!("building a bridge: out of metres");
    }
    if online.is() && fighting.online {
        for (chain, index, rung) in sent {
            requests.write(NetAsk::Rung {
                chain,
                index,
                x: rung.p.x,
                z: rung.p.y,
                heading: yaw_of(rung.d),
            });
        }
    }
}

/// Grows `chain`, which its builder standing at `here`, walking along
/// `heading` if they are walking, is building, by this frame: lands it if
/// they are on `land`, or on another bridge (`ground` is land and planks
/// together), past where it began; forgets it if they have gone back onto
/// what it began on with nothing built; fixes a rung each plank's length
/// they are past the last, on the line toward a point [`LEAD`] ahead of
/// them along their heading, as far as `metres` of bridge allow; and aims
/// the tip the same way, past them. `on_own` says whether they stand on
/// one of its fixed planks, on another chain's, or on no plank.
fn grow(
    chain: &mut Chain,
    here: Vec2,
    heading: Option<Vec2>,
    on_own: Option<bool>,
    metres: &mut f32,
    land: &dyn Fn(Vec2) -> bool,
    ground: &dyn Fn(Vec2) -> bool,
) -> Grown {
    let mut grown = Grown::default();
    let root = chain.rungs[0];
    let mut last = *chain.rungs.last().expect("a chain has a root");
    // Where the planks aim: ahead of you along the way you walk, so that
    // after a turn the rungs settle onto your new line rather than swing
    // past it (aimed at you, a rung fixed to one side of your line by the
    // turn's clamp sent the next one 25 degrees past your heading, and
    // the one after back: a kink after every sharp turn).
    let aim = here + heading.map_or(Vec2::ZERO, |f| f * LEAD);
    // Past where it began: out past the edge it left (the ground it left
    // is all behind that line, being a plank or a round island), or well
    // away from the root with a plank fixed (a chain that curves back to
    // its own shore); or still at its root, on the shore or the plank it
    // began on; or gone back behind the root.
    let out = (here - root.p).dot(root.d);
    let gone_out = out > GONE_OUT || (chain.rungs.len() >= 2 && here.distance(root.p) > 2.0 * SPACING);
    let back_inland = out < -FORGET_BEHIND;
    let on_land = land(here);
    if on_land || on_own == Some(false) {
        if gone_out {
            // The far shore, or another bridge: the chain ends on its
            // edge.
            let edge: &dyn Fn(Vec2) -> bool = if on_land { land } else { ground };
            land_on(chain, here, edge, metres, &mut grown);
            chain.tip = None;
            grown.done = true;
            return grown;
        }
        if back_inland {
            chain.tip = None;
            grown.done = true;
            return grown;
        }
        // At the root still, coming up to the edge: the tip waits past it.
    } else if on_own == Some(true) {
        // Back over the chain's own planks: the tip stays as it is.
        return grown;
    }
    // Over the void, or coming up to the edge: a rung fixed on the line
    // toward you each plank's length you are past the last, and the tip
    // aimed at you past that; behind the last rung still, the tip reaches
    // straight on from it. The first plank may slant `MAX_SLANT` from
    // straight out of the edge, every other turn `MAX_TURN` from the last;
    // the first plank of a fork is as long as its slant needs
    // (`first_length`). Heading at ground within a plank past the next
    // rung, no rung is fixed: the tip stretches to the ground, so that the
    // plank that lands is a whole one, with room for its mitre and its cut
    // (`STRETCH`); unless you drift off the tip's line, when a rung is
    // fixed where there is room for one, so that the chain can turn after
    // you.
    loop {
        let to = here - last.p;
        let far = to.length();
        if far < 1e-3 || to.dot(last.d) <= 0.0 || *metres < SPACING || chain.rungs.len() >= MOST_RUNGS {
            break;
        }
        let first = chain.rungs.len() == 1;
        let toward = (aim - last.p).normalize_or(to / far);
        let d = clamp_turn(toward, last.d, if first { MAX_SLANT } else { MAX_TURN });
        let along = to.dot(d);
        let off = to.perp_dot(d).abs();
        let drifting = off > DRIFT;
        let whole = if first && chain.forked { first_length(d.angle_to(root.d)) } else { SPACING };
        // Ground within reach along the plank, or ahead of you along your
        // way: the chain is about to land, and the tip stretches to the
        // ground for as long as it is under you.
        let ahead = shore_ahead(last.p, d, ground, whole + STRETCH);
        let landing = ahead.is_some() || heading.is_some_and(|f| shore_ahead(here, f, ground, STRETCH + SPACING).is_some());
        let length = if landing && off < HALF_WIDTH - STAND_IN {
            break;
        } else if let Some(shore) = ahead {
            let short = (shore - MIN_LAND).min(whole);
            if short < MIN_LAND {
                break;
            }
            short
        } else if along >= whole {
            whole
        } else if drifting && far >= SPACING {
            along.clamp(SPACING, whole)
        } else {
            break;
        };
        let length = length.min(*metres);
        let rung = Rung { p: last.p + d * length, d };
        *metres -= length;
        fix(chain, rung, &mut grown);
        last = rung;
    }
    let to = here - last.p;
    let most = if chain.rungs.len() == 1 { MAX_SLANT } else { MAX_TURN };
    let (d, length) = if to.dot(last.d) > 0.1 {
        let far = to.length();
        let toward = (aim - last.p).normalize_or(to / far);
        (clamp_turn(toward, last.d, most), far + AHEAD)
    } else {
        (last.d, AHEAD)
    };
    // As far as the metres go, and never shorter than `AHEAD`: once it
    // can no longer keep that far ahead of you, the bridge ends here.
    let most = metres.max(AHEAD);
    grown.spent = length > most + 1e-3;
    let length = length.min(most);
    chain.tip = Some(Rung { p: last.p + d * length, d });
    grown
}

/// Fixes `rung` on the end of `chain`, to be sent.
fn fix(chain: &mut Chain, rung: Rung, grown: &mut Grown) {
    grown.added.push((chain.rungs.len() as u16, rung));
    chain.rungs.push(rung);
}

/// Ends `chain`, whose builder stands at `here` on `edge` ground past its
/// root, on that ground's edge: a plank from the last rung to the shore
/// between them, headed at where they stepped onto it, as long as that
/// comes into the shore within [`MAX_END_SLANT`] of straight in and turns
/// no more than a joint may; coming in at more of a slant, or the shore
/// not within reach that way, a plank's worth of rung turned toward the
/// shore first, up to [`LANDING_TURNS`] of them, as `metres` allow. The
/// plank that lands is never shorter than [`MIN_LAND`]: nearer the shore
/// than that, it runs that far into the ground, under the land, so that
/// its end edge crosses the shore wherever its corners fall. Over ground
/// already, the chain is left where it got to, its tip fixed if it is a
/// plank's worth.
fn land_on(chain: &mut Chain, here: Vec2, edge: &dyn Fn(Vec2) -> bool, metres: &mut f32, grown: &mut Grown) {
    let mut last = *chain.rungs.last().expect("a chain has a root");
    for turn in 0..=LANDING_TURNS {
        // Over ground already (felt a hair ahead: a root lies on the very
        // edge of the ground it left).
        if edge(last.p + last.d * 0.05) {
            fix_tip(chain, metres, grown);
            return;
        }
        if chain.rungs.len() >= MOST_RUNGS {
            return;
        }
        let target = bisect(here, last.p, edge);
        let toward = (target - last.p).normalize_or(last.d);
        let inward = outward(target, edge).map(|out| -out).unwrap_or(toward);
        let slant = inward.angle_to(toward);
        let want = if slant.abs() <= MAX_END_SLANT {
            toward
        } else {
            Vec2::from_angle(slant.signum() * MAX_END_SLANT).rotate(inward)
        };
        let first = chain.rungs.len() == 1;
        let mut most = if first { MAX_SLANT } else { MAX_TURN };
        let mut h = clamp_turn(want, last.d, most);
        let mut found = shore_ahead(last.p, h, edge, LAND_REACH);
        // A fork landing within a plank or two slants no more than that
        // length can cut cleanly along the plank it leaves (`CUT_SHARE`):
        // slanted 45 degrees, a short one came out a triangle, its root
        // cut hanging past its end.
        if first && chain.forked {
            for _ in 0..3 {
                let Some(gap) = found else { break };
                let room = CUT_SHARE * gap.max(MIN_LAND) - SLANT_ROOM;
                if HALF_WIDTH * h.angle_to(last.d).abs().tan() <= room {
                    break;
                }
                most = (room.max(0.0) / HALF_WIDTH).atan();
                h = clamp_turn(want, last.d, most);
                found = shore_ahead(last.p, h, edge, LAND_REACH);
            }
        }
        if let Some(gap) = found {
            let length = gap.max(MIN_LAND);
            *metres = (*metres - length).max(0.0);
            fix(chain, Rung { p: last.p + h * length, d: h }, grown);
            grown.landed = true;
            return;
        }
        if turn == LANDING_TURNS || *metres < SPACING {
            return;
        }
        let rung = Rung { p: last.p + h * SPACING, d: h };
        *metres -= SPACING;
        fix(chain, rung, grown);
        last = rung;
    }
}

/// How long the first plank of a fork has to be, in metres, slanting
/// `slant` radians from straight out of the plank it leaves: room for its
/// root to be cut along that plank's edge, which takes `HALF_WIDTH *
/// tan(slant)` of the corner on the inside of the slant, and for the mitre
/// at its far end, within [`CUT_SHARE`] of it; never less than a plank.
fn first_length(slant: f32) -> f32 {
    SPACING.max(2.0 * (HALF_WIDTH * slant.abs().tan() + SLANT_ROOM))
}

/// How far ahead of `from`, along `d`, `ground` begins, to within
/// [`EDGE`], if within `within`; `from` is over nothing.
fn shore_ahead(from: Vec2, d: Vec2, ground: &dyn Fn(Vec2) -> bool, within: f32) -> Option<f32> {
    let steps = (within / SHORE_STEP).ceil() as usize;
    let on = (1..=steps).map(|k| from + d * (k as f32 * SHORE_STEP)).find(|&at| ground(at))?;
    let e = bisect(on, on - d * SHORE_STEP, ground);
    Some((e - from).dot(d).max(0.0))
}

/// How far ahead of `from`, along `d`, `ground` ends, to within [`EDGE`],
/// if within `within`; `from` is over ground.
fn edge_ahead(from: Vec2, d: Vec2, ground: &dyn Fn(Vec2) -> bool, within: f32) -> Option<f32> {
    let steps = (within / SHORE_STEP).floor() as usize;
    let off = (1..=steps).map(|k| from + d * (k as f32 * SHORE_STEP)).find(|&at| !ground(at))?;
    let e = bisect(off - d * SHORE_STEP, off, ground);
    Some((e - from).dot(d).max(0.0))
}

/// `heading` turned no more than `most` radians from `from`, both unit.
fn clamp_turn(heading: Vec2, from: Vec2, most: f32) -> Vec2 {
    let turn = from.angle_to(heading);
    if turn.abs() <= most {
        heading
    } else {
        Vec2::from_angle(turn.signum() * most).rotate(from)
    }
}

/// The point between `on` and `off` where `ground` ends, to within
/// [`EDGE`]: `on` is over ground and `off` is not.
fn bisect(on: Vec2, off: Vec2, ground: &dyn Fn(Vec2) -> bool) -> Vec2 {
    let (mut a, mut b) = (on, off);
    while a.distance(b) > EDGE {
        let middle = (a + b) * 0.5;
        if ground(middle) {
            a = middle;
        } else {
            b = middle;
        }
    }
    (a + b) * 0.5
}

/// Which way is off the ground at `e`, a point on its edge: square to the
/// edge, where a ring [`RING`] out crosses it twice (the two crossings
/// felt for to a fraction of a degree, and the way off square to the line
/// between them), or else toward where the ground is not round the ring;
/// or none, with ground, or none, all round. Until 2026-11-10 it was the
/// latter from 16 points, which is out by up to 11 degrees.
fn outward(e: Vec2, ground: &dyn Fn(Vec2) -> bool) -> Option<Vec2> {
    const SAMPLES: usize = 32;
    let step = TAU / SAMPLES as f32;
    let at = |angle: f32| e + Vec2::from_angle(angle) * RING;
    let on: Vec<bool> = (0..SAMPLES).map(|k| ground(at(k as f32 * step))).collect();
    let rough: Vec2 = (0..SAMPLES).filter(|&k| !on[k]).map(|k| Vec2::from_angle(k as f32 * step)).sum();
    let rough = rough.try_normalize()?;
    let crossings: Vec<Vec2> = (0..SAMPLES)
        .filter(|&k| on[k] != on[(k + 1) % SAMPLES])
        .map(|k| {
            let (mut a, mut b) = (k as f32 * step, (k + 1) as f32 * step);
            for _ in 0..8 {
                let middle = (a + b) * 0.5;
                if ground(at(middle)) == on[k] {
                    a = middle;
                } else {
                    b = middle;
                }
            }
            at((a + b) * 0.5)
        })
        .collect();
    if crossings.len() != 2 {
        return Some(rough);
    }
    let Some(square) = perp(crossings[1] - crossings[0]).try_normalize() else {
        return Some(rough);
    };
    Some(if square.dot(rough) >= 0.0 { square } else { -square })
}

/// Perpendicular to `d`, to its right as the chain heads.
fn perp(d: Vec2) -> Vec2 {
    Vec2::new(-d.y, d.x)
}

/// The turn about the upright that heads `d`, as `Spot` has a turn: the
/// game's -Z turned by it is `d`.
fn yaw_of(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}

/// The heading a turn of `yaw` about the upright gives.
pub(crate) fn heading_of(yaw: f32) -> Vec2 {
    Vec2::new(-yaw.sin(), -yaw.cos())
}

/// Where the segment from `p` to `q` crosses the segment from `a` to `b`,
/// if it does.
fn crossing(p: Vec2, q: Vec2, a: Vec2, b: Vec2) -> Option<Vec2> {
    let r = q - p;
    let s = b - a;
    let denominator = r.perp_dot(s);
    if denominator.abs() < 1e-9 {
        return None;
    }
    let t = (a - p).perp_dot(s) / denominator;
    let u = (a - p).perp_dot(r) / denominator;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then(|| p + r * t)
}

/// Whether `at` is inside the convex quad `q`, its corners in order round
/// it either way.
fn inside_quad(at: Vec2, q: [Vec2; 4]) -> bool {
    let mut sign = 0.0f32;
    for k in 0..4 {
        let edge = q[(k + 1) % 4] - q[k];
        let side = edge.perp_dot(at - q[k]);
        if side.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = side.signum();
        } else if side.signum() != sign {
            return false;
        }
    }
    true
}

/// A root's or an end's edge between its two corners, where it does not
/// run straight from the one to the other: the points of it, in order
/// from the left corner to the right, each with how far across the plank
/// it lies, from 0 at the left corner to 1 at the right. Empty for a
/// straight edge.
type Edge = Vec<(f32, Vec2)>;

/// A chain as it is built ([`shape_of`]): the left and right corners of
/// each of its rungs, its root's edge and its end's between their
/// corners, and whether it ends on ground.
struct Shape {
    corners: Vec<(Vec2, Vec2)>,
    root: Edge,
    end: Edge,
    complete: bool,
}

/// The left and right corners of each rung of `rungs`, and whether the
/// chain ends on ground ([`shape_of`], with no root kept, and its root's
/// and its end's edges between their corners left out).
#[cfg(test)]
fn corners_of(
    rungs: &[Rung],
    building: bool,
    land: &dyn Fn(Vec2) -> bool,
    ground: &dyn Fn(Vec2) -> bool,
) -> (Vec<(Vec2, Vec2)>, bool) {
    let shape = shape_of(rungs, building, None, land, ground);
    (shape.corners, shape.complete)
}

/// The shape of the chain of `rungs`. The left and right corners of each
/// rung, the ribbon's two edges: [`HALF_WIDTH`] out either side, square to
/// the plank at the root and at the end, and mitred at every rung between,
/// across the bisector of the two planks that meet there and as much wider
/// as the mitre needs (`1 / cos(turn / 2)`), so that the planks share their
/// edges and neither gap nor overlap however the chain curves; a mitre
/// that would reach past a short plank's end is drawn in instead. The
/// root and the end are cut along the edge of the `ground` there (the
/// `land`, and every other bridge's planks; [`cut_along`]): a corner over
/// the void is pushed out, back along the first plank or on along the
/// last, onto that edge, so that the bridge meets the shore exactly
/// however it curves and at whatever slant; a corner over another bridge's
/// plank is pulled in along its own plank onto that plank's edge, no
/// further than leaves the plank its share ([`CUT_SHARE`]) past the mitre
/// at its other end, so that a fork is cut flush along the bridge it
/// leaves or lands on and no plank folds (until 2026-11-10 only the push,
/// and only onto land: a fork's root stood square to the fork, one corner
/// over the void, a wedge of void between it and the bridge it left); and
/// a corner over land stays, hidden under the land, whose top is higher.
/// **And so is every point of the root and of the end between its
/// corners** (2026-11-10, Hajun: "fix the sliver gap on curved forks
/// too"): where the edge it meets is bent, the inside of a curve of
/// bridge, a joint, a plank's corner, the root's own edge bends with it,
/// [`Shape::root`], and the end's, [`Shape::end`]. Until then each ran
/// straight from the one corner to the other, a chord across a bent edge:
/// off the inside of a curve a sliver of void was left between a fork and
/// the bridge it left, 0.4 m deep at its middle on a bridge turning 20
/// degrees a plank, right where you stepped off; and off the outside the
/// fork lay 0.18 m over the bridge. A `root` kept from before
/// ([`ROOT_KEPT`]) is put back as it was, not cut again. And whether the
/// chain ends on ground: its last rung on ground, or ground just ahead of
/// it. Only such an end is cut: a chain still `building`, its tip the last
/// of the rungs, has no end yet, however near the shore its tip reaches;
/// and one left over the void, let go of, turned back from or ended at a
/// corner, ends square where it was left, whatever ground is ahead of it
/// (until 2026-11-10 its corners were pushed on like a landing's, up to
/// [`END_PUSH`], so that an end left within that of a shore, or of a
/// bridge, grew a spike out to it).
fn shape_of(
    rungs: &[Rung],
    building: bool,
    root: Option<&((Vec2, Vec2), Edge)>,
    land: &dyn Fn(Vec2) -> bool,
    ground: &dyn Fn(Vec2) -> bool,
) -> Shape {
    let n = rungs.len();
    let mut shape = Shape {
        corners: Vec::new(),
        root: Edge::new(),
        end: Edge::new(),
        complete: false,
    };
    let Some(last) = rungs.last() else {
        return shape;
    };
    // Each plank's direction and length, from its rung to the next.
    let planks: Vec<(Vec2, f32)> = rungs
        .windows(2)
        .map(|pair| {
            let along = pair[1].p - pair[0].p;
            let length = along.length();
            (if length > 1e-4 { along / length } else { pair[0].d }, length)
        })
        .collect();
    // How far along its planks each rung's mitre reaches.
    let mut reach = vec![0.0f32; n];
    shape.corners = (0..n)
        .map(|k| {
            let across = match (k.checked_sub(1).map(|j| planks[j]), planks.get(k)) {
                (None, None) => perp(rungs[k].d) * HALF_WIDTH,
                (None, Some(&(out, _))) => perp(out) * HALF_WIDTH,
                (Some((into, _)), None) => perp(into) * HALF_WIDTH,
                (Some((into, before)), Some(&(out, after))) => {
                    let bisector = (perp(into) + perp(out)).normalize_or(perp(out));
                    let half_turn = into.angle_to(out).abs() * 0.5;
                    let mut length = HALF_WIDTH / half_turn.cos().max(0.1);
                    // Along the planks the mitre reaches `length * sin`;
                    // no further than most of the shorter plank.
                    let most = 0.45 * before.min(after);
                    if half_turn.sin() > 1e-4 && length * half_turn.sin() > most {
                        length = most / half_turn.sin();
                    }
                    reach[k] = length * half_turn.sin();
                    bisector * length
                }
            };
            (rungs[k].p - across, rungs[k].p + across)
        })
        .collect();
    if n > 1 {
        match root {
            Some((corners, edge)) => {
                shape.corners[0] = *corners;
                shape.root = edge.clone();
            }
            None => {
                let (out, length) = planks[0];
                let pull = CUT_SHARE * length - reach[1];
                let (l, r) = shape.corners[0];
                (shape.corners[0], shape.root) = cut_along(l, r, -out, ONTO_EDGE, pull, land, ground);
            }
        }
    }
    shape.complete = !building && n > 1 && {
        let on = planks[n - 2].0;
        ground(last.p) || shore_ahead(last.p, on, ground, 0.15).is_some()
    };
    if shape.complete {
        let (on, length) = planks[n - 2];
        let mut pull = CUT_SHARE * length - reach[n - 2];
        if n == 2 {
            // One plank, cut at both its ends: its end is pulled back no
            // further than its root was pulled in leaves it.
            let (l, r) = shape.corners[0];
            let drawn_in = [l, r]
                .into_iter()
                .chain(shape.root.iter().map(|point| point.1))
                .map(|at| (at - rungs[0].p).dot(on))
                .fold(0.0, f32::max);
            pull -= drawn_in;
        }
        let (l, r) = shape.corners[n - 1];
        (shape.corners[n - 1], shape.end) = cut_along(l, r, on, END_PUSH, pull, land, ground);
    }
    shape
}

/// Cuts the point `at` of a plank's end along the edge of the ground,
/// `out` being the way off the plank's end there: over the void, pushed
/// out along `out` onto ground within `push`; over another plank, pulled
/// in along `-out` to where that plank ends, within `pull`; over land,
/// left. Either way it comes to lie [`TUCK`] in from the edge, over the
/// ground.
fn to_edge(at: Vec2, out: Vec2, push: f32, pull: f32, land: &dyn Fn(Vec2) -> bool, ground: &dyn Fn(Vec2) -> bool) -> Vec2 {
    if !ground(at) {
        return match shore_ahead(at, out, ground, push) {
            Some(gap) => at + out * (gap + TUCK),
            None => at,
        };
    }
    if land(at) {
        return at;
    }
    match edge_ahead(at, -out, ground, pull) {
        Some(gap) => at - out * (gap - TUCK),
        None => at,
    }
}

/// Cuts a plank's end, the straight line from its left corner `l` to its
/// right corner `r`, along the edge of the ground it meets, `out` being
/// the way off the plank there: the two corners, each moved along `out` or
/// back onto that edge ([`to_edge`]), and the points of the edge between
/// them, moved the same way, that it takes to follow it ([`Edge`]). The
/// line is tried at [`STATIONS`] places evenly across, and again halfway
/// between any two of them, [`REFINEMENTS`] times over, wherever the edge
/// there is found more than [`BENT`] off the straight line between them: a
/// bend in the edge, or the corner of a plank, is found to a few
/// centimetres across. Of those, only the ones the edge is more than
/// [`STRAIGHT`] off a straight line without are kept ([`bends`]): none,
/// along a straight edge, where the plank's end is the one quad it always
/// was.
fn cut_along(
    l: Vec2,
    r: Vec2,
    out: Vec2,
    push: f32,
    pull: f32,
    land: &dyn Fn(Vec2) -> bool,
    ground: &dyn Fn(Vec2) -> bool,
) -> ((Vec2, Vec2), Edge) {
    // The line `across` of the way from `l` to `r`, cut: how far across,
    // where it comes to lie, and how far along `out` it was moved.
    let cut = |across: f32| {
        let plain = l.lerp(r, across);
        let at = to_edge(plain, out, push, pull, land, ground);
        (across, at, (at - plain).dot(out))
    };
    // The places between `a` and `b` where the edge leaves the straight
    // line between them, in order.
    fn between(a: (f32, Vec2, f32), b: (f32, Vec2, f32), deeper: usize, cut: &dyn Fn(f32) -> (f32, Vec2, f32), line: &mut Vec<(f32, Vec2)>) {
        if deeper == 0 {
            return;
        }
        let middle = cut((a.0 + b.0) * 0.5);
        if (middle.2 - (a.2 + b.2) * 0.5).abs() <= BENT {
            return;
        }
        between(a, middle, deeper - 1, cut, line);
        line.push((middle.0, middle.1));
        between(middle, b, deeper - 1, cut, line);
    }
    let stations: Vec<(f32, Vec2, f32)> = (0..=STATIONS).map(|k| cut(k as f32 / STATIONS as f32)).collect();
    let mut line: Vec<(f32, Vec2)> = Vec::new();
    for pair in stations.windows(2) {
        line.push((pair[0].0, pair[0].1));
        between(pair[0], pair[1], REFINEMENTS, &cut, &mut line);
    }
    line.push((1.0, stations[STATIONS].1));
    ((line[0].1, line[line.len() - 1].1), bends(&line))
}

/// The points of `line` between its first and its last that it takes to
/// follow it within [`STRAIGHT`], in order: none, for a straight line.
fn bends(line: &[(f32, Vec2)]) -> Edge {
    fn keep(line: &[(f32, Vec2)], from: usize, to: usize, kept: &mut Vec<usize>) {
        let (a, b) = (line[from].1, line[to].1);
        let ab = b - a;
        let furthest = (from + 1..to)
            .map(|k| {
                let share = ((line[k].1 - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
                (k, line[k].1.distance(a + ab * share))
            })
            .max_by(|x, y| x.1.total_cmp(&y.1));
        if let Some((k, off)) = furthest
            && off > STRAIGHT
        {
            keep(line, from, k, kept);
            kept.push(k);
            keep(line, k, to, kept);
        }
    }
    let mut kept = Vec::new();
    if line.len() > 2 {
        keep(line, 0, line.len() - 1, &mut kept);
    }
    kept.into_iter().map(|k| line[k]).collect()
}

/// The point of `line`, an edge with its two corners at 0 and 1, `across`
/// of the way from the one to the other.
fn across_edge(line: &[(f32, Vec2)], across: f32) -> Vec2 {
    let k = line.partition_point(|point| point.0 < across).clamp(1, line.len() - 1);
    let (a, b) = (line[k - 1], line[k]);
    let span = b.0 - a.0;
    if span.abs() < 1e-6 { a.1 } else { a.1.lerp(b.1, ((across - a.0) / span).clamp(0.0, 1.0)) }
}

/// One plank's top, from its `near` corners to its `far` ones, left and
/// right, as quads side by side across it, each with its corners in order
/// round it, near left, near right, far right, far left: one, between
/// straight edges; and where the near edge or the far one bends
/// ([`Edge`]), a strip between each two of the places either does, so that
/// the plank's edge follows the bends.
fn strips(near: (Vec2, Vec2), near_edge: &[(f32, Vec2)], far: (Vec2, Vec2), far_edge: &[(f32, Vec2)]) -> Vec<[Vec2; 4]> {
    if near_edge.is_empty() && far_edge.is_empty() {
        return vec![[near.0, near.1, far.1, far.0]];
    }
    let line = |ends: (Vec2, Vec2), edge: &[(f32, Vec2)]| -> Vec<(f32, Vec2)> {
        std::iter::once((0.0, ends.0)).chain(edge.iter().copied()).chain(std::iter::once((1.0, ends.1))).collect()
    };
    let (near, far) = (line(near, near_edge), line(far, far_edge));
    let mut places: Vec<f32> = near.iter().chain(&far).map(|point| point.0).collect();
    places.sort_by(f32::total_cmp);
    places.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
    places
        .windows(2)
        .map(|pair| {
            [
                across_edge(&near, pair[0]),
                across_edge(&near, pair[1]),
                across_edge(&far, pair[1]),
                across_edge(&far, pair[0]),
            ]
        })
        .collect()
}

/// Takes in every rung the server says someone has laid ([`Plank`]), but
/// your own, which are laid here as you lay them, and builds their bridges.
fn hear_planks(
    fighting: Res<Fighting>,
    planks: Query<&Plank, (With<Remote>, Added<Plank>)>,
    mut bridges: ResMut<Bridges>,
) {
    for plank in &planks {
        if fighting.seat == Some(plank.seat) {
            continue;
        }
        let rung = Rung {
            p: Vec2::new(plank.x, plank.z),
            d: heading_of(plank.heading),
        };
        bridges.lay_remote(plank.seat, plank.chain, plank.index, rung);
    }
}

/// Puts `rung`, the `index`th of `chain`, in its place: on the end if it is
/// next, or waiting until those before it have come.
fn take(chain: &mut Chain, index: u16, rung: Rung) {
    if usize::from(index) < chain.rungs.len() || chain.rungs.len() >= MOST_RUNGS {
        return;
    }
    chain.waiting.push((index, rung));
    chain.waiting.sort_by_key(|(index, _)| *index);
    while chain.waiting.first().is_some_and(|(index, _)| usize::from(*index) == chain.rungs.len()) {
        let (_, rung) = chain.waiting.remove(0);
        chain.rungs.push(rung);
    }
}

/// Builds a chain's mesh again as the chain itself changes, a rung fixed
/// or its tip moved (and every chain's as the file's colours change), and
/// lays the planks' shape over the island; and clears everything out of
/// the arena, or as another game begins. A chain that has not changed is
/// neither worked out nor drawn again: until 2026-11-10 every chain was,
/// against the ground as it then stood, each time anything changed, and
/// the end of a bridge left unfinished reached out for a bridge built past
/// it afterwards.
#[allow(clippy::too_many_arguments)]
fn rebuild(
    fighting: Res<Fighting>,
    changed: Res<UiChanged>,
    look: Res<Look>,
    me: Query<&Venue, With<Player>>,
    mut bridges: ResMut<Bridges>,
    mut islands: ResMut<Islands>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let here = me.single().ok().copied();
    if !bridges.orphans.is_empty() {
        for mesh in bridges.orphans.drain(..) {
            commands.entity(mesh).try_despawn();
        }
    }
    if here != Some(Venue::Arena) || fighting.game != bridges.game {
        if !bridges.chains.is_empty() {
            for chain in bridges.chains.drain(..) {
                if chain.mesh != Entity::PLACEHOLDER {
                    commands.entity(chain.mesh).try_despawn();
                }
            }
            islands.lay_over(Venue::Arena, None);
            bridges.version += 1;
            info!("the bridges are cleared away");
        }
        bridges.active = None;
        bridges.next_id = 0;
        bridges.dirty = false;
        bridges.game = fighting.game;
        return;
    }
    // A chain is worked out and drawn again once it has itself changed, a
    // rung fixed or its tip moved at all, and no other chain with it; all
    // of them are laid over the island again, and the minimap drawn again,
    // once anything is fixed or a tip has moved enough.
    let stale = bridges.chains.iter().any(Chain::stale);
    let overlay_due = bridges.chains.iter().any(|chain| match (chain.tip, chain.overlaid_tip) {
        (Some(tip), Some(laid)) => tip.p.distance(laid.p) > OVERLAY_MOVE || tip.d.angle_to(laid.d).abs() > OVERLAY_TURN,
        (None, None) => false,
        _ => true,
    });
    let restyled = changed.is_changed() && !changed.is_added();
    if !bridges.dirty && !stale && !restyled {
        return;
    }
    let Some(island) = islands.get(Venue::Arena) else {
        return;
    };
    let relay = bridges.dirty || overlay_due || restyled;
    let land = |at: Vec2| island.floor_base(at, TOP + LAND_STEP_UP).is_some();
    let colours = ui().colours.arena;
    let cut = bridges.recut(&land);
    let mut parts = Vec::new();
    for (c, chain) in bridges.chains.iter_mut().enumerate() {
        let redraw = restyled || cut.contains(&c);
        if relay {
            chain.overlaid_tip = chain.tip;
        }
        if chain.corners.len() < 2 {
            chain.triangles.clear();
            if redraw
                && chain.mesh != Entity::PLACEHOLDER
                && meshes.insert(&chain.handle, Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())).is_err()
            {
                warn!("a bridge has lost its mesh");
            }
            continue;
        }
        if !redraw && chain.mesh != Entity::PLACEHOLDER {
            if relay {
                parts.push(chain.triangles.clone());
            }
            continue;
        }
        let (mesh, triangles) = planks(
            &chain.corners,
            &chain.root_edge,
            &chain.end_edge,
            chain.complete,
            chain.top(),
            colours.plank,
            colours.plank_side,
        );
        chain.triangles = triangles;
        if relay {
            parts.push(chain.triangles.clone());
        }
        if chain.mesh == Entity::PLACEHOLDER {
            chain.handle = meshes.add(mesh);
            chain.mesh = commands
                .spawn((
                    InArena,
                    PlankMesh,
                    Name::new(format!("Bridge {} of seat {}", chain.id, chain.seat)),
                    Venue::Arena,
                    Mesh3d(chain.handle.clone()),
                    MeshMaterial3d(look.0.clone()),
                    Transform::default(),
                    NotShadowCaster,
                ))
                .id();
        } else if meshes.insert(&chain.handle, mesh).is_err() {
            warn!("a bridge has lost its mesh");
        }
    }
    bridges.dirty = false;
    if relay {
        bridges.version += 1;
        islands.lay_over(Venue::Arena, (!parts.is_empty()).then_some(parts));
    }
}

/// The planks between the rungs with these `corners`, their tops `height`
/// up, as a mesh coloured by vertex, `top` on top (every other plank a
/// shade lighter) and `side` everywhere else, and as the triangles bodies
/// stand on: each plank's top, underside, left side and right side, and a
/// cap across the tip if the chain is not `complete`. The first plank's
/// near edge follows `root` and the last one's far edge `end`, where those
/// bend, its top and its underside in strips ([`strips`]).
fn planks(
    corners: &[(Vec2, Vec2)],
    root: &[(f32, Vec2)],
    end: &[(f32, Vec2)],
    complete: bool,
    height: f32,
    top: Color,
    side: Color,
) -> (Mesh, Vec<[Vec3; 3]>) {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colours: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut triangles = Vec::new();
    let tint = |colour: Color, lighter: f32| {
        let linear = colour.to_linear();
        [
            (linear.red * (1.0 + lighter)).min(1.0),
            (linear.green * (1.0 + lighter)).min(1.0),
            (linear.blue * (1.0 + lighter)).min(1.0),
            1.0,
        ]
    };
    let side_tint = tint(side, 0.0);
    let up = |at: Vec2| Vec3::new(at.x, height, at.y);
    let down = |at: Vec2| Vec3::new(at.x, height - THICKNESS, at.y);
    // One face: four corners round it, facing `out`, wound to face that way.
    let mut face = |a: Vec3, b: Vec3, c: Vec3, d: Vec3, out: Vec3, colour: [f32; 4]| {
        let (b, d) = if (b - a).cross(c - a).dot(out) < 0.0 { (d, b) } else { (b, d) };
        let first = positions.len() as u32;
        for corner in [a, b, c, d] {
            positions.push(corner.to_array());
            normals.push(out.to_array());
            colours.push(colour);
            uvs.push([0.0, 0.0]);
        }
        indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        triangles.push([a, b, c]);
        triangles.push([a, c, d]);
    };
    for k in 0..corners.len().saturating_sub(1) {
        let (l0, r0) = corners[k];
        let (l1, r1) = corners[k + 1];
        let along = ((l1 + r1) - (l0 + r0)).normalize_or(Vec2::X);
        let left_out = -perp(along);
        let top_tint = tint(top, if k % 2 == 1 { LIGHTER } else { 0.0 });
        let near_edge = if k == 0 { root } else { &[] };
        let far_edge = if k + 2 == corners.len() { end } else { &[] };
        for [a, b, c, d] in strips((l0, r0), near_edge, (l1, r1), far_edge) {
            face(up(a), up(b), up(c), up(d), Vec3::Y, top_tint);
            face(down(a), down(b), down(c), down(d), Vec3::NEG_Y, side_tint);
        }
        face(up(l0), up(l1), down(l1), down(l0), Vec3::new(left_out.x, 0.0, left_out.y), side_tint);
        face(up(r0), up(r1), down(r1), down(r0), Vec3::new(-left_out.x, 0.0, -left_out.y), side_tint);
        if k + 2 == corners.len() && !complete {
            face(up(l1), up(r1), down(r1), down(l1), Vec3::new(along.x, 0.0, along.y), side_tint);
        }
    }
    let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices));
    (mesh, triangles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::island::Island;

    /// A disc of land `radius` across from its middle at `centre`, its
    /// top at 0.25 m, as the arena's islands are.
    fn disc(centre: Vec2, radius: f32) -> Vec<[Vec3; 3]> {
        let at = |angle: f32| Vec3::new(centre.x + angle.cos() * radius, 0.25, centre.y + angle.sin() * radius);
        let middle = Vec3::new(centre.x, 0.25, centre.y);
        (0..64)
            .map(|k| {
                let (a, b) = (k as f32 * TAU / 64.0, (k + 1) as f32 * TAU / 64.0);
                // Wound to face up: in (x, z) the other way round than on a
                // page.
                [middle, at(b), at(a)]
            })
            .collect()
    }

    fn is_land(island: &Island) -> impl Fn(Vec2) -> bool + '_ {
        move |at| island.floor_base(at, 1.0).is_some()
    }

    /// Where the planks of `rungs` reach, as the quads of their corners,
    /// the last of them a tip still being laid if `building`.
    fn quads(rungs: &[Rung], building: bool, land: &dyn Fn(Vec2) -> bool) -> Vec<[Vec2; 4]> {
        let (corners, _) = corners_of(rungs, building, land, land);
        corners
            .windows(2)
            .map(|pair| {
                let ((l0, r0), (l1, r1)) = (pair[0], pair[1]);
                [l0, r0, r1, l1]
            })
            .collect()
    }

    #[test]
    fn a_span_the_computer_lays_is_a_bridge_like_any_other_on_hajuns_map() {
        use roundtown_net::nav::{Grid, Region};
        // The map as the game reads it, and the land a bridge is cut
        // against (`rebuild`).
        let parts = crate::island::glb::read("pvptownmap.glb");
        let corners: Vec<[Vec3; 3]> = parts
            .iter()
            .filter(|part| !part.skinned)
            .filter(|part| {
                !part
                    .material
                    .as_deref()
                    .is_some_and(|material| crate::island::is_material(material, crate::island::WATER_MATERIAL))
            })
            .flat_map(|part| part.triangles.iter().copied())
            .collect();
        let map = Island::new(&corners);
        let land = |at: Vec2| map.floor_base(at, TOP + LAND_STEP_UP).is_some();
        // Every span of the star, the islands' to either side of the
        // middle line as well.
        let grid = Grid::arena();
        let mut spans = Vec::new();
        for land in 0..4u8 {
            for aside in [0.0, -2.5, 2.5] {
                spans.push(grid.span(Region::Team(land), Region::Middle, aside).unwrap());
            }
            spans.push(grid.span(Region::Middle, Region::Sapphire(land), 0.0).unwrap());
            spans.push(grid.span(Region::Middle, Region::Team(land), 0.0).unwrap());
        }
        for (n, span) in spans.iter().enumerate() {
            let mut bridges = Bridges::default();
            let d = span.way();
            // The rungs come one at a time, as the server sends them, and
            // the bridge is cut again at each as the game cuts it: no
            // plank folded, and landed only by the last.
            for index in 0..=span.planks() {
                bridges.lay_remote(5, n as u16, index as u16, Rung { p: span.rung(index), d });
                assert_eq!(bridges.recut(&land), vec![0]);
                let chain = &bridges.chains[0];
                if index >= 1 {
                    assert_unfolded(&chain.corners, chain.top(), chain.complete);
                }
                assert_eq!(chain.complete, index == span.planks(), "span {n}: rung {index} of {}", span.planks());
            }
            let chain = &bridges.chains[0];
            assert_eq!(chain.rungs.len(), span.planks() + 1);
            assert_eq!(bridges.laid().count(), span.planks() + 1);
            // Its root and its end meet the land: every corner of both on
            // it, hidden under it or cut onto its edge.
            let (root, end) = (chain.corners[0], chain.corners[span.planks()]);
            for corner in [root.0, root.1, end.0, end.1] {
                assert!(land(corner), "span {n}: a corner over the void at {corner}");
            }
            // And from shore to shore there is something under a body
            // walking it, down its middle and as far to either side as
            // the computer's players walk.
            let quads: Vec<[Vec2; 4]> = chain.ground_quads().collect();
            let steps = (span.length() / 0.1) as usize;
            for k in 0..=steps {
                let at = span.from.lerp(span.to, k as f32 / steps as f32);
                for side in [-0.9, 0.0, 0.9] {
                    let under = at + perp(d) * side;
                    assert!(
                        land(under) || quads.iter().any(|&quad| inside_quad(under, quad)),
                        "span {n}: nothing under {under}, {:.1} m along",
                        at.distance(span.from)
                    );
                }
            }
        }
    }

    #[test]
    fn a_turn_is_clamped_and_a_heading_goes_round_trip() {
        let east = Vec2::X;
        assert!(clamp_turn(Vec2::new(1.0, 0.2).normalize(), east, MAX_TURN).distance(Vec2::new(1.0, 0.2).normalize()) < 1e-6);
        let turned = clamp_turn(-east, east, MAX_TURN);
        assert!((east.angle_to(turned).abs() - MAX_TURN).abs() < 1e-5, "{turned}");
        for d in [Vec2::X, Vec2::NEG_Y, Vec2::new(0.6, 0.8), Vec2::new(-0.3, -0.954).normalize()] {
            assert!(heading_of(yaw_of(d)).distance(d) < 1e-5, "{d}");
        }
        assert!(inside_quad(Vec2::new(0.5, 0.5), [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]));
        assert!(!inside_quad(Vec2::new(1.5, 0.5), [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]));
        assert!(inside_quad(Vec2::new(0.5, 0.5), [Vec2::Y, Vec2::ONE, Vec2::X, Vec2::ZERO]));
    }

    #[test]
    fn leaving_an_island_the_first_rung_lies_on_its_edge_and_meets_it_at_the_corners() {
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        // Walking east from 9 m out: the edge is at 10.
        let e = bisect(Vec2::new(9.0, 0.0), Vec2::new(11.0, 0.0), &land);
        assert!((e.x - 10.0).abs() < EDGE && e.y.abs() < 1e-6, "{e}");
        let out = outward(e, &land).unwrap();
        assert!(out.distance(Vec2::X) < 0.05, "{out}");
        // Its corners, plain, would stand off the circle; pulled onto it,
        // each is on the edge.
        let root = Rung { p: e, d: out };
        let (corners, complete) = corners_of(&[root, Rung { p: e + out * SPACING, d: out }], false, &land, &land);
        assert!(!complete);
        for corner in [corners[0].0, corners[0].1] {
            assert!((corner.length() - 10.0).abs() < 0.03, "{corner} is off the edge");
            assert!((corner.y.abs() - HALF_WIDTH).abs() < 0.02, "{corner} is not a plank's width out");
        }
        // The next rung's corners are the plain ones.
        assert!((corners[1].0 - (root.p + out * SPACING - perp(out) * HALF_WIDTH)).length() < 1e-5);
    }

    #[test]
    fn the_bridge_follows_your_feet_round_a_curve_and_lands_on_the_far_shore() {
        let mut corners = disc(Vec2::ZERO, 10.0);
        corners.extend(disc(Vec2::new(30.0, 8.0), 10.0));
        let island = Island::new(&corners);
        let land = is_land(&island);
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        let mut chain = Chain::new(0, 0, vec![root]);
        let mut metres = 100.0;
        // You walk from the edge in an arc, 0.1 m a frame, bending north
        // and then east toward the far island: every frame the tip reaches
        // past you, the fixed rungs trace where you walked, a plank's
        // length apart, and the plank under you is never missing.
        let mut here = Vec2::new(10.05, 0.0);
        let mut heading = 0.0f32;
        let mut frames = 0;
        let grown = loop {
            frames += 1;
            assert!(frames < 2000, "never landed");
            heading = if frames < 90 { heading + 0.012 } else { (heading - 0.012).max(-0.25) };
            here += Vec2::from_angle(heading) * 0.1;
            let grown = grow(&mut chain, here, Some(Vec2::from_angle(heading)), None, &mut metres, &land, &land);
            if grown.done {
                break grown;
            }
            let rungs = chain.all();
            let under = quads(&rungs, true, &land).into_iter().any(|quad| inside_quad(here, quad));
            assert!(under, "frame {frames}: nothing under {here}, rungs {:?}", rungs.len());
            let tip = chain.tip.expect("a tip while building");
            assert!((tip.p - here).dot(tip.d) >= AHEAD - 0.5, "the tip stops short at {here}");
        };
        assert!(grown.landed && chain.complete || grown.landed, "did not land");
        assert!(chain.rungs.len() > 8, "{} rungs", chain.rungs.len());
        for pair in chain.rungs.windows(2) {
            let apart = pair[0].p.distance(pair[1].p);
            assert!(apart <= SPACING * 1.5 + 1e-4 || pair[1] == *chain.rungs.last().unwrap(), "{apart} m between rungs");
            assert!(pair[0].d.angle_to(pair[1].d).abs() <= MAX_TURN + 1e-4);
        }
        let turned = chain.rungs.iter().map(|rung| Vec2::X.angle_to(rung.d).abs()).fold(0.0f32, f32::max);
        assert!(turned > 0.5, "never bent round: {turned}");
        // The last rung lies on the far island's edge, headed into it, or,
        // the shore nearer than a plank's worth, into it by up to that;
        // every other rung is over the void; the metres laid are the
        // bridge's length.
        let last = chain.rungs.last().unwrap();
        let off = last.p.distance(Vec2::new(30.0, 8.0)) - 10.0;
        assert!((-MIN_LAND - 0.03..EDGE + 0.01).contains(&off), "{} is {off} m off the shore", last.p);
        assert!(
            land(last.p) || shore_ahead(last.p, last.d, &land, ONTO_EDGE).is_some_and(|gap| gap < 0.1),
            "the last rung does not head into the shore"
        );
        for rung in &chain.rungs[1..chain.rungs.len() - 1] {
            assert!(!land(rung.p), "{} is over land", rung.p);
        }
        let length: f32 = chain.rungs.windows(2).map(|pair| pair[0].p.distance(pair[1].p)).sum();
        assert!((100.0 - metres - length).abs() < 1e-3, "laid {length} m for {} m of bridge", 100.0 - metres);
        // Built, the chain reaches the shore, the planks face out and stand
        // at the right height, and none is folded.
        let (built, complete) = corners_of(&chain.rungs, false, &land, &land);
        assert!(complete);
        let end = built.len() - 1;
        for corner in [built[end].0, built[end].1] {
            assert!(land(corner) || (corner.distance(Vec2::new(30.0, 8.0)) - 10.0).abs() < 0.05, "{corner} is off the far shore");
        }
        for corner in [built[0].0, built[0].1] {
            assert!(land(corner) || (corner.length() - 10.0).abs() < 0.05, "{corner} is off the near shore");
        }
        assert_unfolded(&built, TOP, complete);
        // An unfinished chain is capped at its tip.
        let (open, _) = planks(&built[..3], &[], &[], false, TOP, Color::WHITE, Color::BLACK);
        assert_eq!(open.count_vertices(), 2 * 16 + 4);
    }

    /// Every plank of `corners` faces up on top, down underneath and level
    /// at its sides, and none is folded: each plank's two edges run on the
    /// way the plank goes.
    fn assert_unfolded(corners: &[(Vec2, Vec2)], height: f32, complete: bool) {
        let (mesh, triangles) = planks(corners, &[], &[], complete, height, Color::WHITE, Color::BLACK);
        let caps = if complete { 0 } else { 2 };
        assert_eq!(triangles.len(), (corners.len() - 1) * 8 + caps);
        assert_eq!(mesh.count_vertices(), (corners.len() - 1) * 16 + caps * 2);
        for [a, b, c] in &triangles {
            let normal = (*b - *a).cross(*c - *a);
            let middle = (*a + *b + *c) / 3.0;
            if (middle.y - height).abs() < 1e-4 {
                assert!(normal.y > 1e-3, "a top faces down at {middle}");
            } else if (middle.y - (height - THICKNESS)).abs() < 1e-4 {
                assert!(normal.y < -1e-3, "an underside faces up at {middle}");
            } else {
                assert!(normal.y.abs() < 1e-4);
            }
        }
        for pair in corners.windows(2) {
            let ((l0, r0), (l1, r1)) = (pair[0], pair[1]);
            let along = ((l1 + r1) - (l0 + r0)).normalize();
            assert!((l1 - l0).dot(along) > 0.02, "the left edge folds between {l0} and {l1}");
            assert!((r1 - r0).dot(along) > 0.02, "the right edge folds between {r0} and {r1}");
        }
    }

    #[test]
    fn a_chain_turning_as_hard_as_it_may_never_folds() {
        // Off the shore square, and then every rung turned the most it may,
        // one way, then the other, then back and forth.
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        let mut rungs = vec![root];
        let turns = [1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 0.0];
        for sign in turns {
            let last = *rungs.last().unwrap();
            let d = Vec2::from_angle(sign * MAX_TURN).rotate(last.d);
            rungs.push(Rung { p: last.p + d * SPACING, d });
        }
        // Built over the void, with a tip and without.
        let (corners, complete) = corners_of(&rungs, true, &land, &land);
        assert!(!complete);
        assert_unfolded(&corners, TOP, complete);
        let (corners, _) = corners_of(&rungs, false, &land, &land);
        assert_unfolded(&corners, top_of(3, 5), false);
        // The mitres make the ribbon wider at a turn, by 1 / cos(17.5
        // degrees), never more; the root's corners are on the shore.
        for (k, (l, r)) in corners.iter().enumerate().skip(1) {
            let width = l.distance(*r) / (2.0 * HALF_WIDTH);
            assert!(width > 0.999 && width < 1.0 / (MAX_TURN * 0.5).cos() + 1e-3, "rung {k} is {width} wide");
        }
        for corner in [corners[0].0, corners[0].1] {
            assert!(land(corner) || (corner.length() - 10.0).abs() < 0.05, "{corner} is off the shore");
        }
        // A short plank after a turn, as a landing or a tip left over, has
        // its mitre drawn in rather than reaching past its end.
        let last = *rungs.last().unwrap();
        let d = Vec2::from_angle(MAX_TURN).rotate(last.d);
        rungs.push(Rung { p: last.p + d * MIN_LAND, d });
        let (corners, _) = corners_of(&rungs, false, &land, &land);
        assert_unfolded(&corners, TOP, false);
        // Two chains' tops are a millimetre or more apart, or the same.
        assert!(top_of(0, 0) <= TOP && top_of(0, 0) > TOP - 0.001 * TOPS as f32);
        assert!((top_of(0, 0) - top_of(0, 1)).abs() > 0.0009);
    }

    #[test]
    fn the_bridge_runs_out_with_the_metres_and_a_stub_is_forgotten() {
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        let mut chain = Chain::new(0, 0, vec![root]);
        // Three metres of bridge: two rungs fixed and a tip to the end of
        // them, and no more however far you walk.
        let mut metres = 3.0;
        for step in 1..60 {
            let here = Vec2::new(10.0 + step as f32 * 0.1, 0.0);
            grow(&mut chain, here, Some(Vec2::X), None, &mut metres, &land, &land);
        }
        assert_eq!(chain.rungs.len(), 3);
        assert!(metres < SPACING, "{metres} m left");
        let tip = chain.tip.unwrap();
        assert!((tip.p.x - (10.0 + 2.0 * SPACING + AHEAD)).abs() < 1e-3, "the tip reaches {}", tip.p.x);
        // Stepping back onto the island with nothing fixed yet forgets the
        // chain; letting go with a tip out fixes it.
        let mut bridges = Bridges::default();
        bridges.begin(0, root, false);
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(11.5, 0.0), d: Vec2::X });
        let mut metres = 10.0;
        assert_eq!(bridges.let_go(&mut metres).map(|(_, index, _)| index), Some(1));
        assert!((metres - 8.5).abs() < 1e-4);
        assert_eq!(bridges.chains[0].rungs.len(), 2);
        assert_eq!(bridges.active, None);
        let mut stub = Bridges::default();
        stub.begin(0, root, false);
        assert_eq!(stub.let_go(&mut metres), None);
        assert!(stub.chains.is_empty());
        // Coming up to the edge from inside the island, from as far in as
        // a chain is begun, the chain just begun keeps its tip waiting over
        // the void; back inland with only a root, it is done, and nothing
        // kept.
        for x in [10.0 - LOOK - 0.2, 9.7] {
            let mut fresh = Chain::new(0, 0, vec![root]);
            let grown = grow(&mut fresh, Vec2::new(x, 0.0), Some(Vec2::X), None, &mut metres, &land, &land);
            assert!(!grown.done && fresh.tip.is_some_and(|tip| tip.p.x > 10.5), "from {x}: {grown:?} {:?}", fresh.tip);
        }
        let mut back = Chain::new(0, 0, vec![root]);
        let grown = grow(&mut back, Vec2::new(10.0 - FORGET_BEHIND - 0.1, 0.0), Some(Vec2::NEG_X), None, &mut metres, &land, &land);
        assert!(grown.done && !grown.landed && grown.added.is_empty());
    }

    /// A bridge running east, its top from `-HALF_WIDTH` to `HALF_WIDTH`
    /// across and from x = -6 to about 6 along, as the quads another chain
    /// is cut against.
    fn parent() -> Vec<[Vec2; 4]> {
        (0..9)
            .map(|k| {
                let x0 = -6.0 + k as f32 * SPACING;
                let x1 = x0 + SPACING;
                [Vec2::new(x0, -HALF_WIDTH), Vec2::new(x0, HALF_WIDTH), Vec2::new(x1, HALF_WIDTH), Vec2::new(x1, -HALF_WIDTH)]
            })
            .collect()
    }

    /// Whether `at` is under one of the planks of `corners`.
    fn under(at: Vec2, corners: &[(Vec2, Vec2)]) -> bool {
        corners.windows(2).any(|pair| inside_quad(at, [pair[0].0, pair[0].1, pair[1].1, pair[1].0]))
    }

    #[test]
    fn a_fork_heads_where_you_walk_and_is_cut_flush_along_the_plank_it_leaves() {
        let parent = parent();
        let ground = |at: Vec2| parent.iter().any(|&quad| inside_quad(at, quad));
        let no_land = |_: Vec2| false;
        for slant in [0.0f32, 0.3, 0.6, MAX_SLANT] {
            // Standing on it half a metre from its far side, walking off
            // `slant` from square to the side.
            let dir = Vec2::from_angle(slant).rotate(Vec2::Y);
            let here = Vec2::new(0.0, HALF_WIDTH - 0.5);
            let probe = here + dir * LOOK;
            assert!(ground(here) && !ground(probe));
            // The root is where your way crosses the edge, headed square
            // out of it.
            let e = bisect(here, probe, &ground);
            assert!((e.y - HALF_WIDTH).abs() < EDGE, "slant {slant}: the root is at {e}");
            let out = outward(e, &ground).unwrap();
            assert!(out.distance(Vec2::Y) < 0.02, "slant {slant}: out is {out}");
            let root = Rung { p: e, d: out };
            let mut chain = Chain::new(0, 0, vec![root]);
            chain.forked = true;
            let mut metres = 100.0;
            let mut at = here;
            for frame in 0..80 {
                at += dir * 0.1;
                let on_own = if ground(at) { Some(false) } else { None };
                let grown = grow(&mut chain, at, Some(dir), on_own, &mut metres, &no_land, &ground);
                assert!(!grown.done, "slant {slant}, frame {frame}: {grown:?}");
                let (corners, _) = corners_of(&chain.all(), true, &no_land, &ground);
                assert!(ground(at) || under(at, &corners), "slant {slant}, frame {frame}: nothing under {at}");
                if chain.rungs.len() >= 3 {
                    break;
                }
            }
            assert!(chain.rungs.len() >= 3, "slant {slant}: {} rungs", chain.rungs.len());
            // The first plank heads where you walked, as long as its slant
            // needs; built, both corners of its root lie on the parent's
            // edge, and nothing folds.
            let first = chain.rungs[1].p - root.p;
            assert!(first.normalize().distance(dir) < 0.02, "slant {slant}: the first plank heads {first}");
            assert!((first.length() - first_length(slant)).abs() < 1e-3, "slant {slant}: a first plank of {} m", first.length());
            let (corners, _) = corners_of(&chain.rungs, false, &no_land, &ground);
            for corner in [corners[0].0, corners[0].1] {
                assert!((corner.y - HALF_WIDTH).abs() < 0.03, "slant {slant}: {corner} is off the parent's edge");
            }
            assert_unfolded(&corners, TOP, false);
        }
        assert!((first_length(0.0) - SPACING).abs() < 1e-6);
        assert!(first_length(MAX_SLANT) > 2.0 * HALF_WIDTH);
    }

    #[test]
    fn walking_off_a_bridge_nearly_along_its_edge_keeps_a_plank_under_you() {
        // Off the parent's side at 75 degrees from square: the first plank
        // may slant only `MAX_SLANT`, so its rung is fixed early as you
        // drift off its line, and the chain turns after you.
        let parent = parent();
        let ground = |at: Vec2| parent.iter().any(|&quad| inside_quad(at, quad));
        let no_land = |_: Vec2| false;
        let dir = Vec2::from_angle(75.0f32.to_radians()).rotate(Vec2::Y);
        let here = Vec2::new(2.0, HALF_WIDTH - 0.1);
        assert!(ground(here) && !ground(here + dir * LOOK), "the probe is not over the void");
        let e = bisect(here, here + dir * LOOK, &ground);
        let root = Rung { p: e, d: outward(e, &ground).unwrap() };
        let mut chain = Chain::new(0, 0, vec![root]);
        chain.forked = true;
        let mut metres = 100.0;
        let mut at = here;
        for frame in 0..70 {
            at += dir * 0.1;
            let on_own = if ground(at) { Some(false) } else { None };
            let grown = grow(&mut chain, at, Some(dir), on_own, &mut metres, &no_land, &ground);
            assert!(!grown.done, "frame {frame}: {grown:?}");
            let (corners, _) = corners_of(&chain.all(), true, &no_land, &ground);
            assert!(ground(at) || under(at, &corners), "frame {frame}: nothing under {at}, {} rungs", chain.rungs.len());
        }
        assert!(chain.rungs.len() >= 3, "{} rungs", chain.rungs.len());
        let first = chain.rungs[1].p - root.p;
        assert!(first.length() < first_length(MAX_SLANT) - 0.5, "the first plank was not fixed early: {} m", first.length());
        let last = chain.rungs.last().unwrap();
        assert!(last.d.angle_to(dir).abs() < 0.2, "the chain never turned after you: {:?}", last.d);
        let (corners, _) = corners_of(&chain.all(), true, &no_land, &ground);
        assert_unfolded(&corners, TOP, false);
    }

    #[test]
    fn landing_at_a_slant_the_bridge_turns_in_and_meets_the_shore_whole() {
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        for degrees in [0.0f32, 30.0, 50.0, 70.0, 85.0] {
            // Coming at the island's east shore over the void, that far off
            // straight in.
            let hit = Vec2::new(10.0, 0.0);
            let toward = Vec2::from_angle(degrees.to_radians()).rotate(Vec2::NEG_X);
            let start = hit - toward * 12.0;
            let mut chain = Chain::new(0, 0, vec![Rung { p: start, d: toward }]);
            let mut metres = 100.0;
            let mut here = start;
            let mut grown = Grown::default();
            for _ in 0..400 {
                here += toward * 0.1;
                grown = grow(&mut chain, here, Some(toward), None, &mut metres, &land, &land);
                if grown.done {
                    break;
                }
            }
            assert!(grown.done && grown.landed, "{degrees} degrees: never landed, {grown:?}");
            let (corners, complete) = corners_of(&chain.rungs, false, &land, &land);
            assert!(complete, "{degrees} degrees");
            assert_unfolded(&corners, TOP, complete);
            // The plank that lands comes in no more slanted than it may;
            // every rung but the last is over the void, and the last is on
            // the shore or into it by less than a plank's worth.
            let n = chain.rungs.len();
            let last = chain.rungs[n - 1].p;
            let on = (last - chain.rungs[n - 2].p).normalize();
            let end_slant = on.angle_to(-last.normalize()).abs();
            assert!(end_slant <= MAX_END_SLANT + 0.05, "{degrees} degrees: lands {} degrees off square", end_slant.to_degrees());
            for rung in &chain.rungs[1..n - 1] {
                assert!(!land(rung.p), "{degrees} degrees: {} is over land", rung.p);
            }
            assert!((10.0 - MIN_LAND - 0.03..10.0 + EDGE + 0.01).contains(&last.length()), "{degrees} degrees: the last rung is {} m out", last.length());
            // The end edge meets the shore whole: every point of it is on
            // land or on the shore.
            let (l, r) = corners[n - 1];
            for s in 0..=10 {
                let at = l.lerp(r, s as f32 / 10.0);
                assert!(land(at) || (at.length() - 10.0).abs() < 0.05, "{degrees} degrees: {at} on the end edge is off the shore");
            }
        }
    }

    #[test]
    fn after_a_sharp_turn_the_planks_settle_onto_your_line_without_swinging_past_it() {
        // Over the void with nothing near: east 6 m, a turn of 63 degrees,
        // and 10 m on. A plank is always under you; the rungs after the
        // turn never head more than 10 degrees past your new heading, and
        // the last of them are on it.
        let nothing = |_: Vec2| false;
        let mut chain = Chain::new(0, 0, vec![Rung { p: Vec2::ZERO, d: Vec2::X }]);
        let mut metres = 100.0;
        let mut here = Vec2::ZERO;
        let turned = Vec2::from_angle(63.0f32.to_radians());
        for frame in 0..160 {
            let f = if frame < 60 { Vec2::X } else { turned };
            here += f * 0.1;
            let grown = grow(&mut chain, here, Some(f), None, &mut metres, &nothing, &nothing);
            assert!(!grown.done);
            let (corners, _) = corners_of(&chain.all(), true, &nothing, &nothing);
            assert!(under(here, &corners), "frame {frame}: nothing under {here}");
        }
        let after: Vec<&Rung> = chain.rungs.iter().filter(|rung| rung.p.y > 1.0).collect();
        assert!(after.len() >= 5, "{} rungs after the turn", after.len());
        for rung in &after {
            let past = turned.angle_to(rung.d);
            assert!(past < 10.0f32.to_radians(), "a rung heads {} degrees past the turn", past.to_degrees());
        }
        for rung in &after[after.len() - 3..] {
            assert!(turned.angle_to(rung.d).abs() < 3.0f32.to_radians(), "{:?} is off the line", rung.d);
        }
        for pair in chain.rungs.windows(2) {
            assert!(pair[0].d.angle_to(pair[1].d).abs() <= MAX_TURN + 1e-4);
        }
        let (corners, _) = corners_of(&chain.rungs, false, &nothing, &nothing);
        assert_unfolded(&corners, TOP, false);
    }

    #[test]
    fn a_plank_s_edge_normal_is_square_to_its_nearest_side_or_end() {
        let nothing = |_: Vec2| false;
        let rungs: Vec<Rung> = (0..6).map(|k| Rung { p: Vec2::new(k as f32 * SPACING, 0.0), d: Vec2::X }).collect();
        let mut chain = Chain::new(0, 0, rungs.clone());
        chain.corners = corners_of(&rungs, false, &nothing, &nothing).0;
        let close = |a: Vec2, b: Vec2| a.distance(b) < 1e-4;
        assert!(close(chain.edge_normal(Vec2::new(2.0, HALF_WIDTH)).unwrap(), Vec2::Y));
        assert!(close(chain.edge_normal(Vec2::new(2.0, -HALF_WIDTH)).unwrap(), Vec2::NEG_Y));
        assert!(close(chain.edge_normal(Vec2::new(0.0, 0.5)).unwrap(), Vec2::NEG_X));
        assert!(close(chain.edge_normal(Vec2::new(5.0 * SPACING, 0.3)).unwrap(), Vec2::X));
        // At a joint's corner the side wins, never the joint between planks.
        assert!(close(chain.edge_normal(Vec2::new(SPACING, HALF_WIDTH)).unwrap(), Vec2::Y));
        assert!(Chain::new(0, 0, vec![rungs[0]]).edge_normal(Vec2::ZERO).is_none());
    }

    #[test]
    fn a_sharp_turn_on_the_tip_makes_an_l_corner_with_nothing_sticking_out() {
        let nothing = |_: Vec2| false;
        for degrees in [0.0f32, 30.0, 60.0] {
            // A chain east from x = -4, three planks fixed and the tip 3 m
            // on; standing on the tip 1 m past the last rung, you step off
            // its +y side, `degrees` off square.
            let root = Rung { p: Vec2::new(-4.0, 0.0), d: Vec2::X };
            let mut bridges = Bridges::default();
            bridges.begin(0, root, false);
            for k in 1..=3 {
                bridges.chains[0].rungs.push(Rung { p: Vec2::new(-4.0 + k as f32 * SPACING, 0.0), d: Vec2::X });
            }
            let last_x = -4.0 + 3.0 * SPACING;
            let tip = Rung { p: Vec2::new(last_x + 3.0, 0.0), d: Vec2::X };
            bridges.chains[0].tip = Some(tip);
            bridges.chains[0].built_tip = Some(tip);
            bridges.chains[0].corners = corners_of(&bridges.chains[0].all(), true, &nothing, &nothing).0;
            let dir = Vec2::from_angle(degrees.to_radians()).rotate(Vec2::Y);
            let here = Vec2::new(last_x + 1.0, HALF_WIDTH - 0.3);
            let probe = here + dir * LOOK;
            let (e, out) = bridges.chains[0].tip_side_exit(here, probe).expect("off the side");
            assert!((e.y - HALF_WIDTH).abs() < 1e-4 && out.distance(Vec2::Y) < 1e-5, "{degrees}: {e} {out}");
            assert!(bridges.chains[0].tip_side_exit(here, here + Vec2::X * 0.5).is_none(), "a step along the tip leaves it");
            let mut metres = 50.0;
            let (root, (id, index, rung)) = bridges.corner(0, here, out, dir, &mut metres).expect("a corner");
            assert_eq!((id, index), (0, 4));
            // The fork's root is on the straight side, level with you; the
            // plank ends flush with the far edge of the fork's root, and a
            // hair past it; the fork slants `MAX_SLANT` at the most.
            assert!((root.p - Vec2::new(here.x, HALF_WIDTH)).length() < 1e-4 && root.d.distance(Vec2::Y) < 1e-5, "{degrees}: the root is {root:?}");
            let heading = clamp_turn(dir, out, MAX_SLANT);
            let span = HALF_WIDTH / heading.angle_to(out).cos();
            assert!((rung.p.x - (here.x + span + CORNER_ROOM)).abs() < 1e-3 && rung.p.y.abs() < 1e-4, "{degrees}: the cap is at {}", rung.p);
            assert!((50.0 - metres - (rung.p.x - last_x)).abs() < 1e-3);
            assert!(bridges.active.is_none() && bridges.chains[0].tip.is_none());
            // The fork, cut against the chain, has both root corners on
            // the chain's side and none past its end.
            let old = corners_of(&bridges.chains[0].rungs, false, &nothing, &nothing).0;
            let quads: Vec<[Vec2; 4]> = old.windows(2).map(|pair| [pair[0].0, pair[0].1, pair[1].1, pair[1].0]).collect();
            let ground = |at: Vec2| quads.iter().any(|&quad| inside_quad(at, quad));
            let fork = [root, Rung { p: root.p + heading * first_length(heading.angle_to(out)), d: heading }];
            let (corners, _) = corners_of(&fork, false, &nothing, &ground);
            for corner in [corners[0].0, corners[0].1] {
                assert!((corner.y - HALF_WIDTH).abs() < 0.03, "{degrees}: {corner} is off the side");
                assert!(corner.x <= rung.p.x + 0.03, "{degrees}: {corner} hangs past the cap at {}", rung.p.x);
            }
            assert_unfolded(&corners, TOP, false);
        }
    }

    #[test]
    fn a_short_fork_lands_with_a_slant_its_length_can_cut() {
        // Off the parent's +y side toward a shore 0.6 m out, walking 40
        // degrees off square: the plank that lands slants far less than
        // that, so that both its root corners are cut onto the parent's
        // side.
        let parent = parent();
        let ground = |at: Vec2| parent.iter().any(|&quad| inside_quad(at, quad));
        let land = |at: Vec2| at.y >= HALF_WIDTH + 0.6;
        let both = |at: Vec2| ground(at) || land(at);
        let dir = Vec2::from_angle(40.0f32.to_radians()).rotate(Vec2::Y);
        let here = Vec2::new(0.0, HALF_WIDTH - 0.3);
        let e = bisect(here, here + dir * LOOK, &ground);
        let mut chain = Chain::new(0, 0, vec![Rung { p: e, d: Vec2::Y }]);
        chain.forked = true;
        let mut metres = 50.0;
        let mut at = here;
        let mut grown = Grown::default();
        for _ in 0..60 {
            at += dir * 0.1;
            let on_own = if ground(at) { Some(false) } else { None };
            grown = grow(&mut chain, at, Some(dir), on_own, &mut metres, &land, &both);
            if grown.done {
                break;
            }
        }
        assert!(grown.done && grown.landed, "{grown:?}");
        assert_eq!(chain.rungs.len(), 2, "{:?}", chain.rungs);
        let slant = chain.rungs[1].d.angle_to(Vec2::Y).abs();
        assert!(slant < 30.0f32.to_radians() && slant > 1.0f32.to_radians(), "slants {} degrees", slant.to_degrees());
        let (corners, complete) = corners_of(&chain.rungs, false, &land, &both);
        assert!(complete);
        for corner in [corners[0].0, corners[0].1] {
            assert!((corner.y - HALF_WIDTH).abs() < 0.03, "{corner} is off the parent's side");
        }
        assert_unfolded(&corners, TOP, complete);
    }

    #[test]
    fn landing_on_another_bridge_the_end_is_cut_flush_along_it() {
        // Walking from the void onto the parent's side, square and at a
        // slant: the chain ends on the parent's edge, both end corners on
        // it.
        let parent = parent();
        let ground = |at: Vec2| parent.iter().any(|&quad| inside_quad(at, quad));
        let no_land = |_: Vec2| false;
        for degrees in [0.0f32, 35.0] {
            let toward = Vec2::from_angle(degrees.to_radians()).rotate(Vec2::NEG_Y);
            let start = Vec2::new(0.0, HALF_WIDTH) - toward * 8.0;
            let mut chain = Chain::new(0, 0, vec![Rung { p: start, d: toward }]);
            let mut metres = 100.0;
            let mut here = start;
            let mut grown = Grown::default();
            for _ in 0..200 {
                here += toward * 0.1;
                let on_own = if ground(here) { Some(false) } else { None };
                grown = grow(&mut chain, here, Some(toward), on_own, &mut metres, &no_land, &ground);
                if grown.done {
                    break;
                }
            }
            assert!(grown.done && grown.landed, "{degrees} degrees: never landed, {grown:?}");
            let (corners, complete) = corners_of(&chain.rungs, false, &no_land, &ground);
            assert!(complete, "{degrees} degrees");
            assert_unfolded(&corners, TOP, complete);
            let n = chain.rungs.len();
            for corner in [corners[n - 1].0, corners[n - 1].1] {
                assert!((corner.y - HALF_WIDTH).abs() < 0.03, "{degrees} degrees: {corner} is off the parent's edge");
            }
        }
    }

    /// A chain of `seat`'s with these rungs and nothing being laid, as one
    /// let go of, or come from the server.
    fn left(seat: u8, id: u16, rungs: Vec<Rung>) -> Chain {
        Chain::new(seat, id, rungs)
    }

    /// `count` planks' worth of rungs from `from`, heading `d`.
    fn straight(from: Vec2, d: Vec2, count: usize) -> Vec<Rung> {
        (0..=count).map(|k| Rung { p: from + d * (k as f32 * SPACING), d }).collect()
    }

    #[test]
    fn a_bridge_already_built_does_not_change_when_another_is_built_past_it() {
        // Hajun's four screenshots of 2026-11-10: a bridge off the island's
        // east shore, left unfinished five planks out over the void.
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        let mut bridges = Bridges::default();
        bridges.chains.push(left(0, 0, straight(Vec2::new(10.0, 0.0), Vec2::X, 5)));
        assert_eq!(bridges.recut(&land), vec![0]);
        let before = bridges.chains[0].corners.clone();
        let end = Vec2::new(10.0 + 5.0 * SPACING, 0.0);
        // Its end is square, a plank's width across, and capped.
        assert!(!bridges.chains[0].complete);
        assert!(before[5].0.distance(end - Vec2::Y * HALF_WIDTH) < 1e-5 && before[5].1.distance(end + Vec2::Y * HALF_WIDTH) < 1e-5, "{:?}", before[5]);
        // Nothing has changed, so nothing is worked out again.
        assert!(bridges.recut(&land).is_empty());
        // Another bridge is built across its way, a metre past its end; a
        // third right over its end; a fourth beside its root. Each time
        // only the new one is worked out, and the first is as it was, to
        // the last bit.
        let past = Vec2::new(end.x + 1.0 + HALF_WIDTH, -8.0);
        let over = Vec2::new(end.x + 0.8, 4.0);
        let beside = Vec2::new(9.5 + HALF_WIDTH, HALF_WIDTH + 0.4);
        for (c, (from, d)) in [(past, Vec2::Y), (over, Vec2::NEG_Y), (beside, Vec2::Y)].into_iter().enumerate() {
            bridges.chains.push(left(1, c as u16, straight(from, d, 8)));
            assert_eq!(bridges.recut(&land), vec![c + 1], "bridge {}", c + 1);
            assert_eq!(bridges.chains[0].corners, before, "the first bridge changed as bridge {} was built", c + 1);
            assert!(!bridges.chains[0].complete);
        }
        // Worked out afresh against the ground as it stands now, it would
        // not be: its end is over the third bridge, and would be cut along
        // it, and its root's corner beside the fourth pulled along that.
        let others = bridges.ground_but(0);
        let ground = |at: Vec2| land(at) || others.iter().any(|&quad| inside_quad(at, quad));
        let (afresh, complete) = corners_of(&bridges.chains[0].rungs, false, &land, &ground);
        assert!(complete && afresh[5] != before[5], "the end would not have changed: {:?}", afresh[5]);
        // A rung of its own, though, and it is worked out again, and it
        // alone; the root, cut for good by its third rung, stays.
        let more = Rung { p: end + Vec2::X * SPACING, d: Vec2::X };
        take(&mut bridges.chains[0], 6, more);
        assert_eq!(bridges.recut(&land), vec![0]);
        assert_eq!(bridges.chains[0].corners[0], before[0], "the root was cut again");
        assert_eq!(bridges.chains[0].corners.len(), 7);
    }

    #[test]
    fn an_end_left_short_of_a_shore_stays_square() {
        // Toward a second island, let go of two metres short of its shore:
        // the end is square where it was left, not pushed on to the shore.
        let mut corners = disc(Vec2::ZERO, 10.0);
        corners.extend(disc(Vec2::new(30.0, 3.0), 10.0));
        let island = Island::new(&corners);
        let land = is_land(&island);
        let far_shore = 30.0 - (100.0f32 - 9.0).sqrt();
        let count = ((far_shore - 2.0 - 10.0) / SPACING).floor() as usize;
        let rungs = straight(Vec2::new(10.0, 0.0), Vec2::X, count);
        let last = *rungs.last().unwrap();
        assert!(far_shore - last.p.x > 0.5 && far_shore - last.p.x < END_PUSH, "{} m short", far_shore - last.p.x);
        let (built, complete) = corners_of(&rungs, false, &land, &land);
        assert!(!complete);
        let end = built[built.len() - 1];
        assert!(end.0.distance(last.p - Vec2::Y * HALF_WIDTH) < 1e-5 && end.1.distance(last.p + Vec2::Y * HALF_WIDTH) < 1e-5, "{end:?}");
        assert_unfolded(&built, TOP, complete);
        // Landed, a rung on the shore, it is cut along the shore as ever.
        let mut landed = rungs.clone();
        landed.push(Rung { p: Vec2::new(far_shore, 0.0), d: Vec2::X });
        let (built, complete) = corners_of(&landed, false, &land, &land);
        assert!(complete);
        let end = built[built.len() - 1];
        for corner in [end.0, end.1] {
            assert!(land(corner) || (corner.distance(Vec2::new(30.0, 3.0)) - 10.0).abs() < 0.05, "{corner} is off the far shore");
        }
    }

    #[test]
    fn a_root_is_cut_afresh_until_its_third_rung_and_kept_from_then_on() {
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        let mut bridges = Bridges::default();
        bridges.begin(0, root, false);
        // A root and a tip, and then one plank fixed: cut, on the shore,
        // and not kept yet.
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(11.0, 0.0), d: Vec2::X });
        assert_eq!(bridges.recut(&land), vec![0]);
        bridges.chains[0].rungs.push(Rung { p: Vec2::new(10.0 + SPACING, 0.0), d: Vec2::X });
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(12.5, 0.0), d: Vec2::X });
        assert_eq!(bridges.recut(&land), vec![0]);
        assert!(bridges.chains[0].root_cut.is_none());
        let on_shore = bridges.chains[0].corners[0];
        for corner in [on_shore.0, on_shore.1] {
            assert!((corner.length() - 10.0).abs() < 0.03, "{corner} is off the shore");
        }
        // The tip moving is the chain changing; standing still is not.
        assert!(bridges.recut(&land).is_empty());
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(12.6, 0.0), d: Vec2::X });
        assert_eq!(bridges.recut(&land), vec![0]);
        // The third rung: kept.
        bridges.chains[0].rungs.push(Rung { p: Vec2::new(10.0 + 2.0 * SPACING, 0.0), d: Vec2::X });
        assert_eq!(bridges.recut(&land), vec![0]);
        assert_eq!(bridges.chains[0].root_cut.as_ref().map(|kept| kept.0), Some(on_shore));
        // Someone's plank comes to lie over one of its corners; the chain
        // grows on, worked out again every frame, and its root stays as it
        // was cut. Afresh, that corner would have been cut along the plank.
        let corner = on_shore.1;
        let plank = vec![
            Rung { p: Vec2::new(corner.x + 0.3, corner.y - 0.4), d: Vec2::Y },
            Rung { p: Vec2::new(corner.x + 0.3, corner.y + 2.0), d: Vec2::Y },
        ];
        bridges.chains.push(left(1, 0, plank));
        assert_eq!(bridges.recut(&land), vec![1]);
        bridges.chains[0].rungs.push(Rung { p: Vec2::new(10.0 + 3.0 * SPACING, 0.0), d: Vec2::X });
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(16.0, 0.0), d: Vec2::X });
        assert_eq!(bridges.recut(&land), vec![0]);
        assert_eq!(bridges.chains[0].corners[0], on_shore);
        let others = bridges.ground_but(0);
        let ground = |at: Vec2| land(at) || others.iter().any(|&quad| inside_quad(at, quad));
        let (afresh, _) = corners_of(&bridges.chains[0].all(), true, &land, &ground);
        assert!(afresh[0].1.distance(on_shore.1) > 0.05, "the root would not have changed: {:?}", afresh[0]);
    }

    #[test]
    fn a_chain_forgotten_takes_what_was_drawn_of_it_down() {
        // A root and a tip drawn, and back inland: the chain goes, and its
        // mesh is handed over to be taken down.
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        let mesh = Entity::from_bits(7);
        let mut bridges = Bridges::default();
        bridges.begin(0, root, false);
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(11.0, 0.0), d: Vec2::X });
        bridges.chains[0].mesh = mesh;
        bridges.abandon(0);
        assert!(bridges.chains.is_empty() && bridges.active.is_none());
        assert_eq!(bridges.orphans, vec![mesh]);
        // Let go of with a tip too short to keep, the same; one never
        // drawn leaves nothing to take down.
        let mut bridges = Bridges::default();
        bridges.begin(0, root, false);
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(10.3, 0.0), d: Vec2::X });
        bridges.chains[0].mesh = mesh;
        let mut metres = 10.0;
        assert_eq!(bridges.let_go(&mut metres), None);
        assert!(bridges.chains.is_empty());
        assert_eq!(bridges.orphans, vec![mesh]);
        let mut bridges = Bridges::default();
        bridges.begin(0, root, false);
        bridges.abandon(0);
        assert!(bridges.orphans.is_empty());
        // A tip dropped from a chain that stays is the chain changing: it
        // is worked out again, without the tip's plank.
        let nothing = |_: Vec2| false;
        let mut bridges = Bridges::default();
        bridges.begin(0, root, false);
        bridges.chains[0].rungs.push(Rung { p: Vec2::new(10.0 + SPACING, 0.0), d: Vec2::X });
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(13.0, 0.0), d: Vec2::X });
        assert_eq!(bridges.recut(&nothing), vec![0]);
        assert_eq!(bridges.chains[0].corners.len(), 3);
        bridges.abandon(0);
        assert_eq!(bridges.recut(&nothing), vec![0]);
        assert_eq!(bridges.chains[0].corners.len(), 2);
        assert!(bridges.recut(&nothing).is_empty());
    }

    /// A bridge east from the origin: three planks straight, `planks` more
    /// each turned `turn` radians from the last, and three straight on.
    fn bent(turn: f32, planks: usize) -> Vec<Rung> {
        let mut rungs = vec![Rung { p: Vec2::ZERO, d: Vec2::X }];
        let turns = std::iter::repeat_n(0.0, 3).chain(std::iter::repeat_n(turn, planks)).chain(std::iter::repeat_n(0.0, 3));
        for by in turns {
            let last = *rungs.last().unwrap();
            let d = Vec2::from_angle(by).rotate(last.d);
            rungs.push(Rung { p: last.p + d * SPACING, d });
        }
        rungs
    }

    /// Points all along an edge, from its left corner to its right through
    /// the places it bends.
    fn along_edge(corners: (Vec2, Vec2), edge: &Edge) -> Vec<Vec2> {
        let line: Vec<Vec2> = std::iter::once(corners.0).chain(edge.iter().map(|point| point.1)).chain(std::iter::once(corners.1)).collect();
        line.windows(2).flat_map(|pair| (0..=12).map(move |k| pair[0].lerp(pair[1], k as f32 / 12.0))).collect()
    }

    /// The edge lies along the edge of `ground` all the way across: every
    /// point of it is over the ground, tucked just under it, and a hand's
    /// width `off` it, the way the plank lies, is not. So there is no void
    /// between the plank and the ground it meets, and the plank does not
    /// lie over that ground either.
    fn assert_flush(what: &str, corners: (Vec2, Vec2), edge: &Edge, off: Vec2, ground: &dyn Fn(Vec2) -> bool) {
        for at in along_edge(corners, edge) {
            assert!(ground(at), "{what}: {at} on the edge is over the void");
            assert!(!ground(at + off * 0.04), "{what}: the plank lies over the ground at {at}");
        }
    }

    /// Every strip of the chain's first and last planks runs on the way
    /// the plank goes, none folded, and its top faces up.
    fn assert_strips_unfolded(chain: &Chain) {
        let n = chain.corners.len();
        for k in [0, n - 2] {
            let root: &[(f32, Vec2)] = if k == 0 { &chain.root_edge } else { &[] };
            let end: &[(f32, Vec2)] = if k + 2 == n { &chain.end_edge } else { &[] };
            let (near, far) = (chain.corners[k], chain.corners[k + 1]);
            let on = ((far.0 + far.1) - (near.0 + near.1)).normalize();
            for [a, b, c, d] in strips(near, root, far, end) {
                assert!((d - a).dot(on) > 0.02 && (c - b).dot(on) > 0.02, "a strip folds: {a} {b} {c} {d}");
                assert!((b - a).perp_dot(d - a) > 0.0 && (c - b).perp_dot(d - c) < 0.0 || (b - a).perp_dot(d - a) < 0.0, "a strip is twisted");
            }
        }
        let (_, triangles) = planks(&chain.corners, &chain.root_edge, &chain.end_edge, chain.complete, TOP, Color::WHITE, Color::BLACK);
        for [a, b, c] in &triangles {
            let normal = (*b - *a).cross(*c - *a);
            let middle = (*a + *b + *c) / 3.0;
            if (middle.y - TOP).abs() < 1e-4 {
                assert!(normal.y > 1e-6, "a top faces down at {middle}");
            } else if (middle.y - (TOP - THICKNESS)).abs() < 1e-4 {
                assert!(normal.y < -1e-6, "an underside faces up at {middle}");
            }
        }
    }

    #[test]
    fn a_fork_off_a_curve_of_bridge_follows_its_bent_edge_and_leaves_no_void() {
        // Hajun, 2026-11-10: "fix the sliver gap on curved forks too". A
        // bridge turning 20 degrees a plank, and a fork straight out of the
        // middle of the bend: off its inside, and off its outside.
        let nothing = |_: Vec2| false;
        for inside in [true, false] {
            let what = if inside { "off the inside" } else { "off the outside" };
            let mut bridges = Bridges::default();
            bridges.chains.push(left(0, 0, bent(20.0f32.to_radians(), 6)));
            assert_eq!(bridges.recut(&nothing), vec![0]);
            let parent = &bridges.chains[0];
            assert!(parent.root_edge.is_empty() && parent.end_edge.is_empty());
            // Its inside is the side it turns toward, its right corners.
            let side = |k: usize| if inside { parent.corners[k].1 } else { parent.corners[k].0 };
            let e = (side(6) + side(7)) * 0.5;
            let d = parent.edge_normal(e).unwrap();
            let inward = perp(parent.rungs[6].d);
            assert!(d.dot(inward) * if inside { 1.0 } else { -1.0 } > 0.9, "{what}: the way off is {d}");
            let ground: Vec<[Vec2; 4]> = parent.ground_quads().collect();
            let on_parent = |at: Vec2| ground.iter().any(|&quad| inside_quad(at, quad));
            bridges.chains.push(left(0, 1, straight(e, d, 3)));
            assert_eq!(bridges.recut(&nothing), vec![1], "{what}");
            let fork = &bridges.chains[1];
            // The root's edge bends with the parent's, and lies along it
            // all the way across.
            assert!(!fork.root_edge.is_empty(), "{what}: the root is straight");
            assert_flush(what, fork.corners[0], &fork.root_edge, d, &on_parent);
            assert_strips_unfolded(fork);
            // The straight line between its corners, as the root was until
            // then, leaves the edge by far more than a sliver: off the
            // inside it stands 0.4 m out over the void in the middle, with
            // nothing under whoever steps off there, and off the outside it
            // lies 0.18 m in over the parent.
            let middle = (fork.corners[0].0 + fork.corners[0].1) * 0.5;
            let out = (middle - e).dot(d);
            let just_off = e + d * 0.08;
            let under = |quads: &[[Vec2; 4]]| quads.iter().any(|&quad| inside_quad(just_off, quad));
            let now: Vec<[Vec2; 4]> = fork.ground_quads().collect();
            let chord: Vec<[Vec2; 4]> = fork.fixed_quads().collect();
            if inside {
                assert!(out > 0.3, "the chord is only {out} m out");
                assert!(!on_parent(middle) && !on_parent(just_off));
                assert!(under(&now) && !under(&chord), "nothing under a step off the edge at {just_off}");
            } else {
                assert!(out < -0.12, "the chord is only {out} m in");
                assert!(on_parent(middle + d * 0.1));
                assert!(under(&now));
            }
        }
    }

    #[test]
    fn wherever_you_step_off_a_curve_of_bridge_there_is_no_void_under_the_step() {
        // Bends from gentle to as hard as a bridge turns; stepping off near
        // a joint and between two; off the inside and the outside; square
        // to the edge and at a slant either way. Round where you step off,
        // out to half a metre and 0.6 m either side, every point is over
        // the bridge you left or the fork, and no strip of the fork folds.
        let nothing = |_: Vec2| false;
        for degrees in [10.0f32, 20.0, 30.0, 35.0] {
            for share in [0.15f32, 0.5, 0.85] {
                for slant in [-0.6f32, 0.0, 0.6] {
                    for inside in [true, false] {
                        let what = format!("{degrees} degrees, {share} along, slant {slant}, inside {inside}");
                        let mut bridges = Bridges::default();
                        bridges.chains.push(left(0, 0, bent(degrees.to_radians(), 6)));
                        bridges.recut(&nothing);
                        let parent = &bridges.chains[0];
                        let side = |k: usize| if inside { parent.corners[k].1 } else { parent.corners[k].0 };
                        let e = side(6).lerp(side(7), share);
                        let d = parent.edge_normal(e).unwrap();
                        let ground: Vec<[Vec2; 4]> = parent.ground_quads().collect();
                        let on_parent = |at: Vec2| ground.iter().any(|&quad| inside_quad(at, quad));
                        let h = Vec2::from_angle(slant).rotate(d);
                        let first = first_length(slant);
                        let rungs = vec![Rung { p: e, d }, Rung { p: e + h * first, d: h }, Rung { p: e + h * (first + SPACING), d: h }];
                        bridges.chains.push(left(0, 1, rungs));
                        assert_eq!(bridges.recut(&nothing), vec![1], "{what}");
                        let fork = &bridges.chains[1];
                        assert_strips_unfolded(fork);
                        let planks: Vec<[Vec2; 4]> = fork.ground_quads().collect();
                        for across in [-0.6f32, -0.3, 0.0, 0.3, 0.6] {
                            for out in [0.03f32, 0.1, 0.2, 0.35, 0.5] {
                                let at = e + perp(d) * across + d * out;
                                assert!(
                                    on_parent(at) || planks.iter().any(|&quad| inside_quad(at, quad)),
                                    "{what}: nothing under {at}, {across} across and {out} out from the step at {e}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn landing_on_a_curve_of_bridge_the_end_follows_its_bent_edge() {
        // Over the void straight at the inside of the bend, and at its
        // outside: the end is cut along the bent edge all the way across.
        let nothing = |_: Vec2| false;
        for inside in [true, false] {
            let what = if inside { "onto the inside" } else { "onto the outside" };
            let mut bridges = Bridges::default();
            bridges.chains.push(left(0, 0, bent(20.0f32.to_radians(), 6)));
            bridges.recut(&nothing);
            let parent = &bridges.chains[0];
            let side = |k: usize| if inside { parent.corners[k].1 } else { parent.corners[k].0 };
            let e = (side(6) + side(7)) * 0.5;
            let d = parent.edge_normal(e).unwrap();
            let ground: Vec<[Vec2; 4]> = parent.ground_quads().collect();
            let on_parent = |at: Vec2| ground.iter().any(|&quad| inside_quad(at, quad));
            bridges.chains.push(left(1, 0, straight(e + d * (4.0 * SPACING), -d, 4)));
            assert_eq!(bridges.recut(&nothing), vec![1], "{what}");
            let chain = &bridges.chains[1];
            assert!(chain.complete, "{what}: it does not land");
            assert!(chain.root_edge.is_empty() && !chain.end_edge.is_empty(), "{what}: the end is straight");
            assert_flush(what, chain.corners[4], &chain.end_edge, d, &on_parent);
            assert_strips_unfolded(chain);
        }
    }

    #[test]
    fn a_straight_edge_is_one_quad_and_a_bent_one_strips_that_share_their_sides() {
        // Off a straight bridge's side, square and at a slant: the root's
        // edge is straight, nothing between its corners, the plank the one
        // quad it always was.
        let nothing = |_: Vec2| false;
        for slant in [0.0f32, 0.5] {
            let mut bridges = Bridges::default();
            bridges.chains.push(left(0, 0, straight(Vec2::new(-6.0, 0.0), Vec2::X, 9)));
            bridges.recut(&nothing);
            let d = Vec2::from_angle(slant).rotate(Vec2::Y);
            let root = Rung { p: Vec2::new(0.0, HALF_WIDTH), d: Vec2::Y };
            let rungs = vec![root, Rung { p: root.p + d * first_length(slant), d }, Rung { p: root.p + d * (first_length(slant) + SPACING), d }];
            bridges.chains.push(left(0, 1, rungs));
            assert_eq!(bridges.recut(&nothing), vec![1]);
            let fork = &bridges.chains[1];
            assert!(fork.root_edge.is_empty(), "slant {slant}: {:?}", fork.root_edge);
            for corner in [fork.corners[0].0, fork.corners[0].1] {
                assert!((corner.y - HALF_WIDTH).abs() < 0.03, "slant {slant}: {corner} is off the side");
            }
            assert_eq!(fork.ground_quads().count(), 2);
        }
        // A plank whose near edge bends twice is three strips, side by
        // side, each sharing its sides with the next, and their corners the
        // plank's own; one whose far edge bends too, somewhere else, four.
        let near = (Vec2::new(0.0, 0.0), Vec2::new(4.0, 0.0));
        let far = (Vec2::new(0.0, 2.0), Vec2::new(4.0, 2.0));
        let bent: Edge = vec![(0.25, Vec2::new(1.0, -0.3)), (0.5, Vec2::new(2.0, 0.1))];
        assert_eq!(strips(near, &[], far, &[]), vec![[near.0, near.1, far.1, far.0]]);
        let three = strips(near, &bent, far, &[]);
        assert_eq!(three.len(), 3);
        assert_eq!((three[0][0], three[2][1], three[2][2], three[0][3]), (near.0, near.1, far.1, far.0));
        for pair in three.windows(2) {
            assert!(pair[0][1].distance(pair[1][0]) < 1e-6 && pair[0][2].distance(pair[1][3]) < 1e-6);
        }
        assert!(three[0][1].distance(Vec2::new(1.0, -0.3)) < 1e-6 && three[1][2].distance(Vec2::new(2.0, 2.0)) < 1e-6);
        let four = strips(near, &bent, far, &vec![(0.75, Vec2::new(3.0, 2.4))]);
        assert_eq!(four.len(), 4);
        assert!(four[2][1].distance(Vec2::new(3.0, 0.05)) < 1e-6 && four[2][2].distance(Vec2::new(3.0, 2.4)) < 1e-6);
        // A line bent by less than a hair is straight; one with a corner
        // keeps the corner alone.
        let line: Vec<(f32, Vec2)> = (0..=8).map(|k| (k as f32 / 8.0, Vec2::new(k as f32, if k % 2 == 0 { 0.0 } else { 0.002 }))).collect();
        assert!(bends(&line).is_empty());
        let line: Vec<(f32, Vec2)> = (0..=8).map(|k| (k as f32 / 8.0, Vec2::new(k as f32, (k as f32 - 3.0).max(0.0) * 0.1))).collect();
        assert_eq!(bends(&line), vec![(0.375, Vec2::new(3.0, 0.0))]);
    }

    #[test]
    fn out_of_metres_the_bridge_ends_with_a_plank_s_worth_still_ahead_of_you() {
        // Off the shore east with 4.5 m of bridge, 12 m, and none to speak
        // of: every frame there is a plank under you, and the frame the
        // tip can reach no further for want of metres, the chain says so,
        // with the tip's end still about `AHEAD` in front of you (a step
        // or two less: the tip is aimed at you only once you are a hand's
        // width past its rung). Let go of there, as `lay` does, the tip
        // is fixed, the metres are none, and the bridge ends that far
        // ahead of where you were stopped.
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        for start in [4.5f32, 12.0, 0.6] {
            let mut bridges = Bridges::default();
            bridges.begin(0, root, false);
            let mut metres = start;
            let mut here = Vec2::new(9.9, 0.0);
            let mut stopped = None;
            for frame in 0..400 {
                here += Vec2::X * 0.04;
                let grown = grow(&mut bridges.chains[0], here, Some(Vec2::X), None, &mut metres, &land, &land);
                assert!(!grown.done, "{start} m, frame {frame}: {grown:?}");
                let (corners, _) = corners_of(&bridges.chains[0].all(), true, &land, &land);
                assert!(land(here) || under(here, &corners), "{start} m, frame {frame}: nothing under {here}");
                if grown.spent {
                    stopped = Some(here);
                    break;
                }
                assert!(metres >= OUT_BELOW || land(here), "{start} m, frame {frame}: {metres} m left and not spent");
            }
            let here = stopped.unwrap_or_else(|| panic!("{start} m: the bridge never ran out"));
            let tip = bridges.chains[0].tip.unwrap();
            assert!((tip.p.x - here.x) > AHEAD - 0.15, "{start} m: only {} m of plank ahead", tip.p.x - here.x);
            // What is left is less than a plank and the reach ahead of
            // it: all of it goes into the last plank.
            assert!(metres < SPACING + AHEAD, "{start} m: spent with {metres} m left");
            let fixed = bridges.let_go(&mut metres);
            assert!(fixed.is_some() && bridges.active.is_none(), "{start} m");
            assert_eq!(metres, 0.0, "{start} m");
            let end = bridges.chains[0].rungs.last().unwrap().p.x;
            assert!(end - here.x > AHEAD - 0.15, "{start} m: the bridge ends {} m ahead", end - here.x);
            // No more than was paid for, but for a last plank never
            // shorter than `AHEAD`.
            assert!(end - 10.0 <= start.max(AHEAD) + 1e-3, "{start} m of bridge laid {} m", end - 10.0);
        }
        // With plenty left the tip is never spent.
        let mut chain = Chain::new(0, 0, vec![root]);
        let mut metres = 100.0;
        for step in 1..200 {
            let grown = grow(&mut chain, Vec2::new(10.0 + step as f32 * 0.05, 0.0), Some(Vec2::X), None, &mut metres, &land, &land);
            assert!(!grown.spent, "spent with {metres} m left");
        }
    }

    #[test]
    fn the_metres_left_are_written_white_then_orange_shading_to_red() {
        let arena = ui().arena;
        let colours = ui().colours.arena;
        let of = |metres: f32| left_colour(metres, arena.bridge_low_at, arena.bridge_red_at, colours.bridge_left, colours.bridge_low, colours.bridge_out);
        // Hajun's numbers: orange at 10 m, red at 5.
        assert_eq!((arena.bridge_low_at, arena.bridge_red_at), (10.0, 5.0));
        let (white, orange, red) = (colours.bridge_left, colours.bridge_low, colours.bridge_out);
        assert_eq!(of(500.0), white);
        assert_eq!(of(10.6), white, "11m is written");
        // By the number as written: 10.4 is "10m".
        for metres in [10.4, 10.0, 9.6] {
            let close = of(metres).to_srgba();
            assert!((close.red - orange.to_srgba().red).abs() < 1e-4 && (close.green - orange.to_srgba().green).abs() < 1e-4, "{metres} m is {close:?}");
        }
        for metres in [5.4, 5.0, 3.0, 0.4, 0.0, -1.0] {
            assert_eq!(of(metres), red, "{metres} m");
        }
        // Between, between: greener than red and redder than orange, more
        // so the fewer are left.
        let green = |metres: f32| of(metres).to_srgba().green;
        assert!(green(8.0) < orange.to_srgba().green && green(8.0) > green(6.0) && green(6.0) > red.to_srgba().green);
        // The three are told apart, and the first is white.
        assert_eq!(white.to_srgba(), Srgba::WHITE);
        assert!(orange.to_srgba().green > red.to_srgba().green + 0.2 && orange.to_srgba().blue < 0.5);
        // The same thresholds inside out are red from the first of them.
        assert_eq!(left_colour(7.0, 5.0, 10.0, white, orange, red), white);
        assert_eq!(left_colour(4.0, 5.0, 10.0, white, orange, red), red);
    }

    #[test]
    fn rungs_arriving_out_of_turn_wait_for_those_before_them() {
        let mut chain = Chain::new(3, 7, Vec::new());
        let rung = |index: u16| Rung {
            p: Vec2::new(index as f32 * SPACING, 0.0),
            d: Vec2::X,
        };
        take(&mut chain, 2, rung(2));
        take(&mut chain, 0, rung(0));
        assert_eq!(chain.rungs.len(), 1);
        take(&mut chain, 1, rung(1));
        assert_eq!(chain.rungs.len(), 3);
        assert!(chain.waiting.is_empty());
        assert!((chain.rungs[2].p.x - 2.0 * SPACING).abs() < 1e-4);
    }
}
