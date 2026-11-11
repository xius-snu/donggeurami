//! Round Town, played from behind you, or through your own eyes, with WASD or
//! touch.
//!
//! The app opens in the town, with you and seven others already standing in
//! it (`lobby`). Walk up to any of them and you can talk to them (`talk`).
//! The House Builder, at the door of their house in the town, asks whether
//! you want to play their game (`builder`).
//!
//! `#[bevy_main]` generates the `android_main` entry point. Desktop and iOS
//! both reach [`main`] through `src/main.rs`; on iOS that binary *is* the app
//! executable, which `mobile/ios/build_rust.sh` drops into the .app bundle.

use bevy::gilrs::GilrsPlugin;
use bevy::gltf::GltfPlugin;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::light::cluster::GlobalClusterSettings;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::view::NoIndirectDrawing;
use bevy::window::WindowResolution;
use bevy::world_serialization::WorldInstanceReady;


mod ai;
mod arena;
mod bridge;
mod build;
mod builder;
mod minimap;
mod car;
mod games;
mod clock;
mod drive;
mod editor;
mod fountain;
mod hud;
mod island;
mod lamps;
mod log_file;
mod lobby;
mod login;
mod fire;
mod loot;
mod map;
mod market;
mod net;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod pace;
mod range;
mod roads;
mod see_through;
mod settings;
mod shop;
mod signup;
mod sky;
mod sound;
mod tags;
mod talk;
mod ui;
mod utility;
mod gear;
use island::{Island, Islands, Venue};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
#[cfg(any(target_os = "android", target_os = "ios"))]
use bevy::window::{MonitorSelection, WindowMode};
#[cfg(target_os = "android")]
use bevy::window::PrimaryWindow;

const MOVE_SPEED: f32 = 7.0;
const TURN_SPEED: f32 = 22.0;
const JUMP_SPEED: f32 = 8.5;
const GRAVITY: f32 = 22.0;
/// Gravity on the way down: heavier than on the way up, so that a fall is
/// quick without a jump losing any of its height.
const FALL_GRAVITY: f32 = GRAVITY * 1.5;
const PLAYER_HEIGHT: f32 = 1.6;
const PLAYER_RADIUS: f32 = 0.45;
/// How far out from the middle of a body its legs reach, in metres: the
/// model's legs, side by side, are 0.42 m across, where its body and arms
/// are 0.86 ([`PLAYER_RADIUS`]). A body stands on whatever is under its legs
/// (`support`), and below [`HIP_HEIGHT`] its legs are all of it there is to
/// bump into anything (`resolve_player_solids`).
const LEG_RADIUS: f32 = 0.21;
/// How high over its feet a body's legs go before it widens into its body
/// and arms, in metres: the top of the model's legs. It is as wide as it
/// gets by [`WAIST_HEIGHT`].
const HIP_HEIGHT: f32 = 0.65;
const WAIST_HEIGHT: f32 = 0.8;
/// How much higher the ground under a body's legs can be than the ground
/// under the middle of it and still be the one slope, in metres: the bridge,
/// the steepest ground there is at 15°, rises 0.06 m across the legs. Higher
/// than that, the body is standing on an edge (`support`).
const EDGE_RISE: f32 = 0.15;
/// Points round the middle of a body's feet, as far out as its legs reach.
const UNDER_THE_LEGS: [Vec2; 8] = {
    let (r, d) = (LEG_RADIUS, LEG_RADIUS * std::f32::consts::FRAC_1_SQRT_2);
    [
        Vec2::new(r, 0.0),
        Vec2::new(d, d),
        Vec2::new(0.0, r),
        Vec2::new(-d, d),
        Vec2::new(-r, 0.0),
        Vec2::new(-d, -d),
        Vec2::new(0.0, -r),
        Vec2::new(d, -d),
    ]
};
const TREE_TRUNK_RADIUS: f32 = 1.1;
const TREE_TRUNK_HEIGHT: f32 = 6.5;
const WATER_JUMP_SLOP: f32 = 0.22;
const JUMP_COYOTE: f32 = 0.14;
/// How far under a ceiling a jump stops the head, in metres.
const HEAD_CLEARANCE: f32 = 0.02;
const JUMP_BUFFER: f32 = 0.12;
const LOOK_HEIGHT: f32 = PLAYER_HEIGHT * 0.75;
/// The height everyone arrives at, and the height of the path round the
/// pond in `roundtownmap.glb`, where they arrive in the town: they settle on
/// whatever is under them from there.
const LAND_TOP: f32 = 0.0;
/// The sea, everywhere the island is not: z = -0.5 in Blender, which leaves the
/// lowest ground on either island — the town's roads, the home's sand, both at
/// z = -0.2 — just clear of it.
const WATER_Y: f32 = -0.5;
const WATER_WADE: f32 = 0.58;
const WATER_BOB_AMP: f32 = 0.07;
const WATER_BOB_SPEED: f32 = 1.7;
const WATER_MOVE_SCALE: f32 = 0.72;
/// The highest a body steps up onto without jumping, in metres: anything
/// under half a metre. Half a metre itself takes a jump, like the fountain's
/// rim out of its basin (Hajun, 2026-09-30); the 5 mm short of it keep a rise
/// of exactly that from going either way as its height rounds. It was 0.75
/// until 2026-09-30, when Hajun made it 0.5. It is also the height under which
/// nothing in the way counts as a wall (`Island::walls`), so that what is
/// taller than this has to be jumped onto or walked round.
const LAND_STEP_UP: f32 = 0.495;
const OCEAN_LIMIT: f32 = 580.0;
/// How far down into the void a body falls before it is dead, in metres
/// (`arena::reap`, which puts it back where it arrived after its while):
/// the file's `arena.fall_to_die` below the land, 30 m as first written,
/// about a second and a half's fall, well under the map's underside, 3 m
/// down (15 m until the night of 2026-11-07, Hajun: "for falling make it
/// so u need to drop a bit more z to actually die"). Only over the void,
/// where there is no sea to come down in (`island::bottomless`).
pub(crate) fn void_y() -> f32 {
    LAND_TOP - ui::ui().arena.fall_to_die
}
const CAMERA_DISTANCE: f32 = 10.0;
/// The nearest the camera stands behind you, in metres. Zoomed in nearer, it
/// goes into your eyes ([`SWAP_ZOOM`]). 2.5 until 2026-11-01, when Hajun
/// found "the max zoom in is not zoomed in enough" and asked for the camera
/// to come closer before it goes into your eyes.
const CAMERA_DISTANCE_MIN: f32 = 1.5;
/// The nearest it stands looking up past you, in metres: looking up it comes
/// in to a share of how far out it stands ([`CAMERA_LOOK_UP_DISTANCE`] of
/// [`CAMERA_DISTANCE`]), which from [`CAMERA_DISTANCE_MIN`] would put it
/// inside you.
const CAMERA_LOOK_UP_NEAREST: f32 = 0.8;
/// The furthest it stands, in metres: 15 until Hajun asked for a little more
/// (2026-10-09).
const CAMERA_DISTANCE_MAX: f32 = 18.0;
/// How much further a zoom has to go than the camera can, as a ratio, before
/// it goes on into your eyes from as near as it stands behind you, or back
/// out behind you from your eyes: a tenth, a notch of the wheel, so that
/// fingers that waver as a pinch ends do not swap it back and forth.
const SWAP_ZOOM: f32 = 1.1;
const ZOOM_WHEEL: f32 = 0.12;
const CAMERA_FAR: f32 = 4000.0;
// How far over you the camera stands as you get into the car (`drive`),
// looking straight down, and the nearest and the furthest it zooms there,
// are the file's, `assets/ui.ron`'s `car.overhead`, `overhead_nearest` and
// `overhead_furthest` (24, 7 and 60 as first written: the car is 6.5 m long,
// and the roads 12 m wide).
const STICK_DEADZONE: f32 = 0.18;
/// The stick: 22 until 2026-11-01, when Hajun asked for "slightly" bigger.
/// It grows up and to the right from where its corner sits.
const STICK_SIZE_VMIN: f32 = 25.0;
/// 8 until Hajun asked for "very very slightly more left" (2026-11-01).
const STICK_LEFT_VW: f32 = 7.0;
const STICK_BOTTOM_VH: f32 = 11.0;
/// The jump button: nearly as big as the stick, and nearer the corner than
/// the stick is (Hajun, 2026-10-31: bigger, "especially jump button", and
/// "slightly closer to the bottom right corner"; 18, 8 and 11 until then;
/// then 5 and 7 for a night, until "very slightly more margin from the
/// bottom right corner", 2026-11-01, and then "move them up very very
/// slightly"). The three buttons round it are placed from it
/// (`hud::show_top_bar`).
const JUMP_SIZE_VMIN: f32 = 22.0;
const JUMP_RIGHT_VW: f32 = 6.0;
const JUMP_BOTTOM_VH: f32 = 9.0;
const LOOK_SENSITIVITY: f32 = 0.0045;
/// How much more a finger dragged across a phone's screen turns the camera
/// than the mouse does for the same distance (Hajun, 2026-11-01: "increase
/// the camera panning ... sensitivity slightly so each drag rotates the
/// camera bit more"; the same as the mouse until then).
#[cfg(any(target_os = "android", target_os = "ios"))]
const TOUCH_LOOK_GAIN: f32 = 1.25;
const PITCH_MIN: f32 = -1.22;
/// As far down as the camera tips: straight down, for a view of the tops of
/// things (Hajun, 2026-10-09). It stopped at 1.2, about 69°, until then.
const PITCH_MAX: f32 = std::f32::consts::FRAC_PI_2;
const CAMERA_LOOK_UP_DISTANCE: f32 = 2.72;
const CAMERA_LOOK_UP_HEIGHT: f32 = 0.58;
const LOOK_UP_FOCUS_HEIGHT: f32 = PLAYER_HEIGHT * 0.9;
const LOOK_UP_EXTRA_PITCH: f32 = 0.52;
/// How much the camera takes in, top to bottom: about 66°, which is 98°
/// across a 16:9 screen and 110° across a 20:9 phone. Wide, so that while you
/// build, a piece and the room it goes in are on the screen together. It was
/// first person's own until that became the only view (Hajun, 2026-10-09);
/// third person took in 45°.
const CAMERA_FOV: f32 = 1.15;
/// How high over your feet your eyes are, which the camera is in when it is
/// zoomed all the way in: inside your head, as high as the model's eyes, 1.54
/// to 1.6 m up, under the top of the head at 1.71 m (Hajun, 2026-10-09). It
/// was 2.2 m from 2026-10-03, over your head, for the view of a taller
/// person, and 1.44 m before that.
/// 1.38 since the evening of 2026-11-01: the eyes of `playercharacter.glb`
/// as it is, 1.64 m tall with a round head 0.53 m across; 1.57, the old
/// model's, until then.
pub(crate) const EYE_HEIGHT: f32 = 1.38;
/// How far your eyes keep from a ceiling lower than them, in metres: further
/// than the corners of the camera's near plane reach from it, 0.18 m at the
/// widest, looking up on a 20:9 phone, so that it is never cut open on screen.
const CAMERA_RADIUS: f32 = 0.2;
/// How far past what you bump into with the camera still counts as inside
/// you, in metres: your arms and the top of your head reach that far.
const CAMERA_INSIDE: f32 = 0.15;
/// How far down the camera looks at you as the app opens, in radians, and
/// once a double tap has swung it round behind you.
const CAMERA_PITCH: f32 = 0.38;
/// How long the camera has to have been left alone before walking with the
/// stick brings it round behind where you walk, in seconds (Hajun,
/// 2026-10-31: "~1.5 s"), or riding a bus brings it round behind the way
/// the bus faces (Hajun, 2026-11-01).
const RECENTER_AFTER: f32 = 1.5;
/// How long it takes to get going once it does, in seconds: it eases in,
/// from when the stick was pushed, or the bus got on, or the camera let go
/// of, whichever came later, rather than all at once.
const RECENTER_EASE_IN: f32 = 0.6;
/// How quickly it comes round, walking at full speed, or riding: this share
/// of the angle left, a second...
const RECENTER_RATE: f32 = 1.0;
/// ...and no faster than this, in radians a second (46°). The stick held to
/// one side walks you round a circle about 17 m across as it comes round, wide
/// enough to steer out of by pushing the stick up.
const RECENTER_MOST: f32 = 0.8;
/// Walking toward the camera it does not come round at all: as readily as
/// ever walking up to 100° off straight away from it, and not at all from
/// 150°, as running at it.
const RECENTER_SIDEWAYS: f32 = 1.745;
const RECENTER_BACKWARDS: f32 = 2.618;
/// How quickly a double tap swings the camera round behind you: this share
/// of the way left, a second, which is there in about a fifth of a second.
const SNAP_RATE: f32 = 16.0;
/// Two taps on nothing are a double tap if the second goes down no later
/// than this after the first came up, in seconds, and no further from it than
/// this share of the screen's short side.
const DOUBLE_TAP_SECS: f32 = 0.3;
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
const DOUBLE_TAP_NEAR: f32 = 0.08;
/// The longest a press can be held and still be a tap, in seconds.
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
const TAP_SECS: f32 = 0.3;
/// While you talk to someone (`talk`), the furthest from them the camera
/// looks at their face from, in metres: from further off it comes this near,
/// between you and them; from nearer it is your eyes.
const CLOSE_UP: f32 = 1.3;
/// How far under the face of someone you talk to the camera looks, in
/// metres: their face sits a little over the middle of the screen, clear of
/// what they say along the bottom of it.
const UNDER_THE_FACE: f32 = 0.25;
/// How long the camera takes to come in to a face, and to go back out, in
/// seconds.
const CLOSE_UP_IN: f32 = 0.45;
const CLOSE_UP_OUT: f32 = 0.35;
/// How fast the limbs swing as a body walks, in radians a second of stride
/// at full speed.
const WALK_STRIDE_FREQ: f32 = 7.6;
/// How far each limb swings, in radians: the thighs forward and back, the
/// shins bending back behind them, and the arms against the legs.
const WALK_THIGH: f32 = 0.42;
const WALK_SHIN: f32 = 0.28;
const WALK_ARM: f32 = 0.55;
/// How far under level the arms hang at the sides, in radians: 75 degrees,
/// 15 out from straight down, clear of the hips. Each arm is turned down
/// from the model's rest by whatever that takes ([`WalkBone::hang`], worked
/// out as it is bound, [`hang_from_rest`]): `playercharacter.glb` as it is
/// rests with its upper arms 60 degrees under level, its first version of
/// 2026-11-01 did 28, and the old models' stood straight out (turned 1.02
/// rad down from there, `WALK_ARM_HANG`, until that day).
const ARM_HANG_BELOW: f32 = 1.31;
/// How far out from straight down the legs stand, in radians: 3.5 degrees.
/// Each thigh is turned in from the model's rest by whatever that takes
/// ([`WalkBone::stance`], [`stance_from_rest`]), and never out:
/// `playercharacter.glb` rests with its legs 7 degrees out, its ankles 0.27
/// m apart, which Hajun found "too wide apart while standing/walking"
/// (2026-11-01); stood so, they are 0.19 m apart, the shoes a few
/// centimetres from each other.
const LEG_SPLAY: f32 = 0.06;
/// How quickly a walk comes and goes as a body starts and stops.
const WALK_BLEND: f32 = 9.0;

/// The one player of the eight that this device steers: its stick and keys
/// move them, and the camera follows them.
#[derive(Component)]
struct Player;

/// What a body is trying to do this frame, whoever is steering it: this
/// device's stick and keys for [`Player`], an AI's
/// [`Steering`](ai::Steering) for the others. [`move_bodies`] carries it out
/// the same way for both.
#[derive(Component)]
struct Intent {
    /// Which way to walk, flat on the ground. Its length is how fast, from 0
    /// for standing still to 1 for full speed.
    walk: Vec3,
    /// A jump, asked for this frame.
    jump: bool,
    /// How fast to go along `walk`, in metres a second, in place of a walk's
    /// own speed: the car's, driving it (`drive`), which the water does not
    /// slow and which keeps to where it could drive back out of.
    pace: Option<f32>,
    /// Which way to face, if not the way it walks: the way the car faces,
    /// reversing too.
    face: Option<Vec3>,
    /// How much faster than a walk the body goes, as a ratio: 1 walking,
    /// and in a game of Bridge Wars the file's `arena.run_speed` over a
    /// walk's, running (`arena`; the night of 2026-11-07, Hajun: "make it
    /// running rather than walking"). The legs swing that much faster and
    /// further ([`animate_walk`]).
    sprint: f32,
}

impl Default for Intent {
    fn default() -> Self {
        Self {
            walk: Vec3::ZERO,
            jump: false,
            pace: None,
            face: None,
            sprint: 1.0,
        }
    }
}

/// A body that is dead, in a game of Bridge Wars (`arena`): its health
/// gone to a punch, or fallen into the void. It stands where it died,
/// hidden, neither moving nor moved, in nobody's way and out of every
/// punch's reach, for this many seconds more, and is then put back where
/// its seat arrives with its health full again.
#[derive(Component, Debug)]
pub(crate) struct Dead {
    pub(crate) left: f32,
}

/// A punch of this device's that landed on a body it moves and took
/// ([`Shove::knock`]): who threw it and whom it landed on. Bridge Wars
/// deals its damage on it (`arena`).
#[derive(Message, Clone, Copy, Debug)]
pub(crate) struct Landed {
    pub(crate) by: Entity,
    pub(crate) victim: Entity,
}

/// A punch of yours, thrown ([`start_punch`]), whether or not it landed on
/// anyone: from where, how high your feet were, and which way you faced.
/// Bridge Wars lands it on a respawn anchor in its range (`arena`).
#[derive(Message, Clone, Copy, Debug)]
pub(crate) struct Swung {
    pub(crate) at: Vec2,
    pub(crate) feet: f32,
    pub(crate) facing: Vec2,
}

/// Which team a body is on in a game of Bridge Wars (`arena`, since
/// 2026-11-08: two to a team, a team to an island): a punch never lands on
/// a teammate (Hajun: "make it so u cannot hurt your own teammate"), and a
/// bot never goes for one. A body with none is on nobody's team.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Team(pub(crate) u8);

/// A body carrying a plank in front of it, for building a bridge (`bridge`,
/// since 2026-11-08; Hajun: "the charater should hold a small plank flat in
/// front of him with his hands like holding a baby style"): both arms held
/// out in front, a little under level, with the plank lying across them
/// ([`CARRY_REACH`]). The right arm still punches.
#[derive(Component, Default)]
pub(crate) struct Carrying;

/// How far the arms swing forward from hanging to carry a plank, in
/// radians: a little under level, so that the plank rests on them.
const CARRY_REACH: f32 = 1.45;

/// A body that is only there to be seen, like the builder a Bridge Builder
/// sends off (`bridge`, since 2026-11-12): it is in nobody's way, and no
/// punch lands on it.
#[derive(Component)]
pub(crate) struct ForShow;

/// A body drinking a potion, in Bridge Wars (`utility`, since 2026-11-12;
/// Hajun's `bridgewarsgameevent.txt`: "push it once and it shows 1.6s
/// timer running to 0 in front of ur character while ur movement is slowed
/// 50% while drinking ... sword unequips while drinking and short hand to
/// mouth animation while drinking,cannot attack while drinking"): for this
/// much longer, in seconds. It walks at half its pace
/// ([`read_player_input`]), throws no punch ([`start_punch`]), and its
/// right hand is at its mouth ([`drink_offset`]). This device's alone, as
/// a punch's swing is.
#[derive(Component, Debug)]
pub(crate) struct Drinking {
    pub(crate) left: f32,
}

impl Drinking {
    /// How far up to the mouth the hand is, from 0 at the side to 1 there:
    /// up as the drink begins and down again as it ends, each in
    /// [`DRINK_LIFT`] seconds.
    fn lifted(&self) -> f32 {
        let gone = roundtown_net::bridge_wars::DRINK_SECS - self.left;
        (gone.min(self.left) / DRINK_LIFT).clamp(0.0, 1.0)
    }
}

/// How long a drinker's hand takes to come up to its mouth, and to go back
/// down, in seconds; and which way the right arm points with the hand
/// there, in the body's space (its right is +x, up +y, and it faces -z):
/// across and up to the front of the face. The model's arms are short and
/// its head big: a straight arm reaches its mouth.
const DRINK_LIFT: f32 = 0.2;
const DRINK_REACH: Vec3 = Vec3::new(-0.62, 0.36, -0.7);

/// How a sword is held and swung, in Bridge Wars (`gear`, since
/// 2026-11-12; Hajun: "you may also need to make the sword animation for
/// the swords rather than using punch swing animation for it"), in
/// radians: the right arm a little forward of hanging, with the blade out
/// in front, at the ready; and for a slash, raised to a little under
/// level and swept about the body's upright from this far to the right of
/// straight ahead to this far to the left of it (a turn to the left is the
/// greater), in the time a punch takes to go out, and back to the ready in
/// the time it takes to come back.
const SWORD_READY: f32 = 0.45;
const SLASH_RAISE: f32 = 1.55;
pub(crate) const SLASH_FROM: f32 = -1.15;
pub(crate) const SLASH_TO: f32 = 1.0;

/// A body that will not walk off dry land into the sea, nor down anything it
/// could not step back up ([`strands`]). An AI does not know it could jump
/// back out, so it would stand in the water, or the fountain, for good.
#[derive(Component)]
struct StaysAshore;

#[derive(Component, Default)]
struct PlayerJump {
    velocity_y: f32,
    coyote: f32,
    buffer: f32,
}

/// The car a body is riding (`car`), if any: standing on it, or in the air
/// off it, and carried along with it until it comes down on anything else.
#[derive(Component, Default, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Riding(pub(crate) Option<Entity>);

#[derive(Component, Default)]
struct WalkCycle {
    phase: f32,
    weight: f32,
    speed: f32,
}

/// How far the arm swings forward from hanging, in radians: up to level in
/// front (1.5 rad until 2026-11-01, when `playercharacter.glb` came, whose
/// arms hang 12 degrees back of the shoulder). How long the punch takes,
/// out and back, is the file's `melee.swing_out` and `swing_back`
/// (`ui::Melee`; since 2026-11-07, Minecraft 1.8.9's six ticks; 0.07 and
/// 0.11 s until then, for 4 to 8 a second, Hajun, 2026-11-01).
const PUNCH_REACH: f32 = 1.7;

/// The fist button, or F on desktop, pressed: a punch is asked for.
#[derive(Message, Default)]
pub(crate) struct PunchPressed;

/// A body throwing a punch (Hajun, 2026-11-01: "a very basic animation for
/// the fist attack that activates each time u press the punch button"):
/// the right arm swings straight out in front and back ([`punch_offset`];
/// the two arms in turn for an hour, until Hajun asked for "only use right
/// hand"). A press mid-punch starts the next at once, from the side. The
/// swing is this device's alone; what it lands on is knocked back
/// ([`start_punch`], [`Shove`]), and that goes over the network as any move
/// does.
#[derive(Component, Default)]
struct Punching {
    /// Seconds into it.
    age: f32,
}

impl Punching {
    /// Starts a punch, whatever the last was doing.
    fn next() -> Punching {
        Punching { age: 0.0 }
    }

    /// How far out the arm is `age` seconds in, from 0 at the side to 1 at
    /// full reach: out, and back a little slower, in the file's times.
    fn reach(melee: &ui::Melee, age: f32) -> f32 {
        if age < melee.swing_out {
            (age / melee.swing_out).clamp(0.0, 1.0)
        } else {
            (1.0 - (age - melee.swing_out) / melee.swing_back).clamp(0.0, 1.0)
        }
    }
}

/// What a body is carried along by after a punch lands on it
/// ([`Shove::knock`]; since 2026-11-07, Hajun: "pressing melee (the punch)
/// actually deals 'knockback' (minecraft 1.8.9 bedwars style melee
/// knockback)"): across the ground, in metres a second, dying away as it
/// goes ([`Motion::step`]), and how long it is still reeling from the last
/// punch, in which no other moves it. Every body this device moves has one,
/// yours and the AIs'; everyone else online is knocked by their own device
/// (`net`), which this device tells through the server ([`Hit`]).
#[derive(Component, Default)]
pub(crate) struct Shove {
    across: Vec2,
    hurt: f32,
}

impl Shove {
    /// Knocks the body, standing at `at`, back from a punch thrown from
    /// `from`, as Minecraft 1.8.9 does: what it was carried by is halved
    /// and the file's `knock_back` away from the puncher added, and
    /// `sprint_bonus` more if the puncher was `running`; and it is thrown
    /// up at `knock_up`, whatever rise it had halved first, and the whole
    /// no more than that. Not within `hurt_time` of the last punch that
    /// landed, while it is still reeling (Minecraft's ten ticks of hurt
    /// resistance): whether this one took.
    pub(crate) fn knock(&mut self, jump: &mut PlayerJump, from: Vec2, at: Vec2, running: bool) -> bool {
        if self.hurt > 0.0 {
            return false;
        }
        let melee = ui::ui().melee;
        let away = (at - from).normalize_or(Vec2::NEG_Y);
        let push = melee.knock_back + if running { melee.sprint_bonus } else { 0.0 };
        self.across = self.across * 0.5 + away * push;
        jump.velocity_y = (jump.velocity_y * 0.5 + melee.knock_up).min(melee.knock_up);
        self.hurt = melee.hurt_time;
        true
    }
}

/// A punch of yours that landed on someone this device does not move,
/// someone online ([`start_punch`]): told to the server, which tells their
/// device to knock them (`net::relay_hits`). `running`, you were on the
/// move as you hit them.
#[derive(Message)]
pub(crate) struct Hit {
    pub(crate) victim: Entity,
    pub(crate) running: bool,
}

#[derive(Component, Clone, Copy)]
struct WalkBone {
    kind: WalkBoneKind,
    rest_rotation: Quat,
    /// How the bone lies at rest in its body's space: the turns of every
    /// node from the body down to it, its own last, so that it can be
    /// turned about the body's own axes whatever way its rig's go
    /// ([`WalkBone::turned`]).
    frame: Quat,
    /// How far the limb is turned about the body's front to back axis from
    /// the model's rest to stand as it should ([`stance_from_rest`]): an arm
    /// down, to hang at the side ([`ARM_HANG_BELOW`]), a thigh in, to stand
    /// under the body ([`LEG_SPLAY`]), the left one way and the right the
    /// other. 0 for a shin, which goes in with its thigh.
    stance: f32,
    /// The player this bone belongs to, whose [`WalkCycle`] swings it.
    owner: Entity,
}

impl WalkBone {
    /// The rotation to write to the bone for `turn`, a turn in its body's
    /// space about the bone's own head, on top of how it rests: its body
    /// sees it turned by `turn` from its rest, whatever way the rig has its
    /// axes. A bone under another that is turned too goes round with it.
    fn turned(&self, turn: Quat) -> Quat {
        self.rest_rotation * self.frame.inverse() * turn * self.frame
    }
}

#[derive(Clone, Copy)]
enum WalkBoneKind {
    ThighL,
    ThighR,
    ShinL,
    ShinR,
    ArmL,
    ArmR,
}

#[derive(Component, Clone, Copy)]
pub(crate) struct Collider {
    shape: ColliderShape,
}

#[derive(Clone, Copy)]
pub(crate) enum ColliderShape {
    Cylinder { radius: f32, height: f32 },
    #[expect(
        dead_code,
        reason = "the island was the only box, and it is a model now; kept for box-shaped catalogue kinds"
    )]
    Aabb { half_extents: Vec3 },
}

#[derive(Component)]
struct ThirdPersonCamera;

#[derive(Component)]
struct JoystickKnob;

#[derive(Component)]
struct JumpButtonVisual;

/// The stick or the jump button, which are put away while you talk to
/// someone.
#[derive(Component)]
struct TouchHud;

/// The mouse, as a finger on the stick or the jump button on desktop
/// ([`read_mouse_controls`]): an id no touch has.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const MOUSE_ID: u64 = u64::MAX;

/// The stick and the jump button, as fingers hold them on a phone, and the
/// mouse does on desktop (since 2026-11-06); what `read_player_input` and
/// `drive::steer` walk and drive you by, on every platform.
#[derive(Resource, Default)]
struct TouchControls {
    stick_id: Option<u64>,
    stick_value: Vec2,
    look_id: Option<u64>,
    look_last: Vec2,
    jump_id: Option<u64>,
    jump_just_pressed: bool,
    pinch_id: Option<u64>,
    pinch_last_dist: f32,
    /// The touch the editor is following. A press on one of the edit buttons
    /// claims one outright so that dragging an object does not also swing the
    /// camera; otherwise this is the same touch as `look_id`, and whether it
    /// turns out to be a tap or a look is decided when it lifts.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    edit_id: Option<u64>,
    /// When and where the touch looking round went down, for as long as it
    /// could still be a tap: until it has gone further than a tap does, or a
    /// second finger has come down to pinch.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    look_tap: Option<(f32, Vec2)>,
    /// The taps on nothing, to tell a double tap by.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    taps: DoubleTap,
}

/// Two taps on nothing, the second soon after the first and near it: a
/// double tap, which swings the camera round behind you.
#[derive(Default)]
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
struct DoubleTap {
    /// The last tap, while it could still be the first of two: when it came
    /// up, and where.
    last: Option<(f32, Vec2)>,
}

#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
impl DoubleTap {
    /// A tap at `at` that went down at `down` and came up at `up`, in
    /// seconds: whether it is the second of two, no more than `near` from
    /// the first. A third tap is the first of the next two.
    fn tap(&mut self, down: f32, up: f32, at: Vec2, near: f32) -> bool {
        let second = self
            .last
            .is_some_and(|(then, there)| down - then <= DOUBLE_TAP_SECS && at.distance(there) <= near);
        self.last = if second { None } else { Some((up, at)) };
        second
    }
}

/// What turns the camera besides your fingers, on a phone (Hajun,
/// 2026-10-31): walking with the stick, it comes round behind where you walk
/// once you have left it alone a while, and a double tap on nothing swings it
/// straight round behind you ([`assist_camera`]).
#[derive(Resource, Default)]
struct CameraAssist {
    /// Seconds since your fingers last turned or zoomed it.
    untouched: f32,
    /// Seconds the stick has been pushed for, without letting go.
    pushed: f32,
    /// Seconds you have been riding a bus for, without getting off.
    aboard: f32,
    /// A double tap is swinging it round to this yaw: behind you as you
    /// stood at the tap, and not as you turn after it. Read off you every
    /// frame, it chased you round and round: walking with the stick you face
    /// where the camera's ahead and the stick point, so the camera coming
    /// round behind you turned you with it (Hajun, 2026-10-31: "if you
    /// double tap the screen, and then u try to move right after, the
    /// camera starts spinning").
    snap: Option<f32>,
    /// Where you were the frame before, and the car you were riding then:
    /// on the car, in its own space, so that what the car carries you is not
    /// taken for walking.
    last: Option<(Vec3, Option<Entity>)>,
}

impl CameraAssist {
    /// Your fingers are on the camera, or the mouse moves it: it waits for
    /// them to have been off it for [`RECENTER_AFTER`] before coming round
    /// behind you again, and whatever swing a double tap began is over.
    fn touched(&mut self) {
        self.untouched = 0.0;
        self.snap = None;
    }

    /// A double tap: the camera is to swing round behind `body` as it stands
    /// now, and there it stops, however `body` turns meanwhile; and the
    /// wait before walking brings it round again ([`RECENTER_AFTER`], and
    /// its easing in) is taken as served, so that it comes round behind
    /// where you walk as soon as the swing is over (Hajun, 2026-11-01:
    /// "make that become 0s instantly so it starts autofollowing right
    /// away"; the taps, fingers on the camera, had started the wait over).
    #[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
    fn snap_behind(&mut self, body: &Transform) {
        self.snap = Some(OrbitCamera::behind(body).yaw);
        self.untouched = RECENTER_AFTER + RECENTER_EASE_IN;
    }

    /// How far `body` walked since the frame before, across the ground, or
    /// across the car it rides, `driven`, as the car is now: never how far
    /// the car carried it.
    fn walked(&mut self, body: &Transform, riding: &Riding, driven: Option<&car::Driven>) -> Vec3 {
        let car = riding.0.and_then(|car| Some((car, driven?)));
        let here = car.map_or(body.translation, |(_, driven)| driven.now.inward(body.translation));
        let on = car.map(|(car, _)| car);
        let walked = match self.last {
            Some((last, was_on)) if was_on == on => {
                let step = here - last;
                car.map_or(step, |(_, driven)| driven.now.turn * step)
            }
            _ => Vec3::ZERO,
        };
        self.last = Some((here, on));
        walked
    }

    /// How far round toward behind where you walk, or behind the way the
    /// bus you ride faces, the camera is ready to come, from 0 to 1: not at
    /// all until it has been left alone for [`RECENTER_AFTER`], then eased in
    /// over [`RECENTER_EASE_IN`], counted from then or from when the stick
    /// was pushed or the bus got on, whichever came later.
    fn ready(&self) -> f32 {
        let since = (self.untouched - RECENTER_AFTER).min(self.pushed.max(self.aboard));
        let ready = (since / RECENTER_EASE_IN).clamp(0.0, 1.0);
        ready * ready * (3.0 - 2.0 * ready)
    }
}

/// While you talk to someone (`talk`), the camera comes in from where you
/// turned and zoomed it to your eyes, turned to their face, and after, goes
/// back out to where it was (Hajun, 2026-10-31).
#[derive(Resource, Default)]
pub(crate) struct CloseUp {
    /// The face it is to come in to, for as long as you talk to them.
    pub(crate) on: Option<Vec3>,
    /// The face it came in to last, which it goes back out from.
    face: Vec3,
    /// How far in it has come, from 0, where you turned and zoomed it, to 1.
    blend: f32,
}

impl CloseUp {
    /// Whether it is on a face, or on its way to one or back from one.
    pub(crate) fn busy(&self) -> bool {
        self.on.is_some() || self.blend > 0.0
    }

    /// Back where you turned and zoomed it at once, rather than on its way:
    /// for going to another island, where it would sweep across the world
    /// from a face left behind.
    pub(crate) fn cut(&mut self) {
        self.on = None;
        self.blend = 0.0;
    }

    /// Moves it on by `dt` seconds, in to the face it is on, or back out, and
    /// says how far in it is now.
    fn step(&mut self, dt: f32) -> f32 {
        self.blend = match self.on {
            Some(face) => {
                self.face = face;
                (self.blend + dt / CLOSE_UP_IN).min(1.0)
            }
            None => (self.blend - dt / CLOSE_UP_OUT).max(0.0),
        };
        self.blend
    }
}

/// How you have turned and zoomed the camera: the one view there is, behind
/// you, or zoomed all the way in, through your own eyes (Hajun, 2026-10-09,
/// in place of a button that swapped between the two). On a phone the stick
/// and a double tap turn it too ([`assist_camera`]).
#[derive(Resource)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    /// How far out the camera stands behind you, from
    /// [`CAMERA_DISTANCE_MIN`] to [`CAMERA_DISTANCE_MAX`]; or 0, in your eyes.
    distance: f32,
    /// How much further the zoom has gone, as a ratio, than the camera could
    /// go with it: in, from as near as it stands behind you, or out, from
    /// your eyes. At [`SWAP_ZOOM`] either way it goes into your eyes, or back
    /// out of them.
    past: f32,
    /// Fixed straight down over you, as it is while you drive the car
    /// (`drive`; Hajun, 2026-11-05: "the camera gets fixed as top down (can
    /// still zoom in/out, but no auto follow rotate/direction/ no panning.
    /// its fixed just follow the car's plane position no rotation.)"):
    /// neither your fingers nor anything else turns it, and it zooms
    /// between the file's `car.overhead_nearest` and `overhead_furthest`,
    /// never into your eyes.
    overhead: bool,
    /// Fixed over you as high as it is, with nothing zooming it: in a game
    /// of Bridge Wars (since late on 2026-11-11; Hajun: "lock that zoom so u
    /// cannot zoom in or out"), the file's `arena.camera_height` up.
    locked: bool,
}

/// How the camera was turned and zoomed, to go back to
/// ([`OrbitCamera::back_to`]).
#[derive(Clone, Copy, Debug)]
pub(crate) struct View {
    yaw: f32,
    pitch: f32,
    distance: f32,
    past: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: CAMERA_PITCH,
            distance: CAMERA_DISTANCE,
            past: 1.0,
            overhead: false,
            locked: false,
        }
    }
}

impl OrbitCamera {
    /// Fixes it straight down over you, the file's `arena.camera_height`
    /// up, with the top of the screen north (Blender's +Y, the game's -Z),
    /// the same wherever you are and whichever way you face, so that the
    /// map is always the same way up: for a game of Bridge Wars (`arena`,
    /// since the night of 2026-11-07; the town itself for 2026-11-07,
    /// Hajun: "make the camera top down for even the player mode").
    /// Nothing turns it, and since late on 2026-11-11 nothing zooms it either
    /// (`locked`): it stands where an island just fits the height of the
    /// screen with you in its middle, on any screen wider than it is
    /// tall, the camera's view being so many degrees top to bottom
    /// ([`CAMERA_FOV`]); the car's `overhead`, 24 m, and zoomed between 7
    /// and 60, until then. How it was, to go back to, unless it was fixed
    /// over you already, in the car.
    fn top_down(&mut self) -> Option<View> {
        let was = (!self.overhead).then(|| self.overhead());
        self.yaw = 0.0;
        self.distance = ui::ui().arena.camera_height;
        self.locked = true;
        was
    }

    /// The file read again in a game of Bridge Wars: the camera goes to
    /// the height it says now.
    fn relock(&mut self) {
        let height = ui::ui().arena.camera_height;
        if self.locked && self.distance != height {
            self.distance = height;
        }
    }

    /// Behind `body`, looking the way it faces, at the starting tilt and
    /// distance.
    fn behind(body: &Transform) -> Self {
        let back = body.back();
        Self {
            yaw: back.x.atan2(back.z),
            ..default()
        }
    }

    /// Fixes it straight down over you, the file's `car.overhead` up, turned
    /// as it is: the top of the screen stays the way it was looking along
    /// the ground. How it was, to go back to.
    fn overhead(&mut self) -> View {
        let was = View {
            yaw: self.yaw,
            pitch: self.pitch,
            distance: self.distance,
            past: self.past,
        };
        self.pitch = PITCH_MAX;
        self.distance = ui::ui().car.overhead;
        self.past = 1.0;
        self.overhead = true;
        was
    }

    /// Back to `view`, as it was turned and zoomed before it was fixed over
    /// you.
    fn back_to(&mut self, view: View) {
        self.yaw = view.yaw;
        self.pitch = view.pitch;
        self.distance = view.distance;
        self.past = view.past;
        self.overhead = false;
        self.locked = false;
    }

    /// Zooms in, for a `ratio` under 1, or out: the camera stands that much
    /// nearer you or further off. Zoomed in on past as near as it stands, it
    /// goes into your eyes, and that is as far in as it zooms; zoomed out
    /// from there, it is straight back behind you, as near as it stands
    /// (Hajun, 2026-10-09). Over you, it stays over you, between the file's
    /// `car.overhead_nearest` and `overhead_furthest`; locked, in a game of
    /// Bridge Wars, it does not zoom at all.
    fn zoom_by_ratio(&mut self, ratio: f32) {
        if self.locked {
            return;
        }
        if self.overhead {
            let car = ui::ui().car;
            self.distance = (self.distance * ratio).clamp(car.overhead_nearest, car.overhead_furthest);
            self.past = 1.0;
            return;
        }
        if self.distance <= 0.0 {
            self.past = (self.past * ratio).max(1.0);
            if self.past >= SWAP_ZOOM {
                self.distance = CAMERA_DISTANCE_MIN;
                self.past = 1.0;
            }
            return;
        }
        let wanted = self.distance * ratio;
        if wanted >= CAMERA_DISTANCE_MIN {
            self.distance = wanted.min(CAMERA_DISTANCE_MAX);
            self.past = 1.0;
            return;
        }
        self.past *= wanted / CAMERA_DISTANCE_MIN;
        if self.past <= 1.0 / SWAP_ZOOM {
            self.distance = 0.0;
            self.past = 1.0;
        } else {
            self.distance = CAMERA_DISTANCE_MIN;
        }
    }

    /// Turns the camera by a swipe or a move of the mouse of `delta` pixels.
    /// Fixed over you, it does not turn.
    fn turn(&mut self, delta: Vec2) {
        if self.overhead {
            return;
        }
        self.yaw -= delta.x * LOOK_SENSITIVITY;
        self.pitch = (self.pitch + delta.y * LOOK_SENSITIVITY).clamp(PITCH_MIN, PITCH_MAX);
    }
}

/// Which way along the ground `camera` faces: the way it looks, or, looking
/// straight down, the way the top of the screen is. What the stick's up
/// walks you toward, and where a piece from the shop comes up.
pub(crate) fn camera_ahead(camera: &Transform) -> Vec3 {
    let looking = camera.forward().with_y(0.0);
    if looking.length_squared() > 1e-4 {
        looking.normalize()
    } else {
        camera.up().with_y(0.0).normalize_or(Vec3::NEG_Z)
    }
}

/// `angle`, in radians, as the same turn no more than half a turn either way.
fn wrap_angle(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// How fast a camera at `yaw` comes round behind someone walking at
/// `velocity` across the ground, in radians a second, `ready` of the way
/// eased in ([`CameraAssist::ready`]): by [`RECENTER_RATE`] of the angle left
/// a second at full speed, more slowly at less, never faster than
/// [`RECENTER_MOST`], and not at all walking toward it.
fn recenter_turn(yaw: f32, velocity: Vec2, ready: f32) -> f32 {
    let speed = velocity.length();
    if speed < 1e-3 {
        return 0.0;
    }
    let off = wrap_angle(yaw_behind(velocity) - yaw);
    let away = ((RECENTER_BACKWARDS - off.abs()) / (RECENTER_BACKWARDS - RECENTER_SIDEWAYS)).clamp(0.0, 1.0);
    turn_round(off, ready * away * (speed / MOVE_SPEED).min(1.0))
}

/// How fast a camera at `yaw` comes round behind the way the bus you ride
/// faces, `facing`, across the ground, in radians a second, `ready` of the
/// way eased in (Hajun, 2026-11-01: on a bus the camera is to come round
/// without your walking, "in direction of bus", so that you need not keep
/// turning it to look the way the bus goes). As walking brings it round
/// ([`recenter_turn`]), but standing still, and from in front of the bus as
/// readily as from beside it: you did not walk at the camera, the bus turned.
fn ride_turn(yaw: f32, facing: Vec2, ready: f32) -> f32 {
    if facing.length_squared() < 1e-6 {
        return 0.0;
    }
    turn_round(wrap_angle(yaw_behind(facing) - yaw), ready)
}

/// The yaw that puts the camera behind someone going `way` across the
/// ground, as [`OrbitCamera::behind`] puts it behind where you face.
fn yaw_behind(way: Vec2) -> f32 {
    (-way.x).atan2(-way.y)
}

/// How fast the camera comes round by `off` radians, at `strength` of full,
/// in radians a second: [`RECENTER_RATE`] of it a second, never faster than
/// [`RECENTER_MOST`].
fn turn_round(off: f32, strength: f32) -> f32 {
    if strength <= 0.0 {
        return 0.0;
    }
    (off * RECENTER_RATE * strength).clamp(-RECENTER_MOST, RECENTER_MOST)
}

/// Turns the camera for you (Hajun, 2026-10-31). Walking with the stick,
/// once your fingers have been off the camera for [`RECENTER_AFTER`], it
/// eases round behind where you walk ([`recenter_turn`]), so that one thumb
/// on the stick sees where it is going. Walked into a wall you go nowhere,
/// and it stays where it is; in your eyes it never turns: there you look
/// where you turned your eyes, whichever way you walk. Riding a bus, left
/// alone as long, it eases round behind the way the bus faces instead
/// ([`ride_turn`]), standing still too, and in your eyes as well (Hajun,
/// 2026-11-01: "riding bus is an exception where user doesnt need to move
/// ... camera will auto follow in direction of bus"); the bus turns under
/// you, and the camera comes round after it, so that you look the way it
/// goes without turning the camera yourself as it goes round. And a double
/// tap on nothing swings it straight round behind you as you stood at the
/// tap ([`CameraAssist::snap_behind`]), tipped as it is when the app opens,
/// or looking straight ahead from your eyes. A finger on the camera stops
/// any of them at once (`read_touch_controls`).
///
/// The stick and the double tap are a phone's alone: on desktop the keys
/// walk you and the mouse turns the camera, and neither brings it round.
/// Riding a bus brings it round on desktop too, once the mouse has been
/// still as long (`read_mouse_look`).
fn assist_camera(
    time: Res<Time>,
    controls: Res<TouchControls>,
    dialog: Res<hud::DialogUp>,
    cars: Query<(&car::Driven, &car::Hull)>,
    me: Query<(&Transform, &Riding), With<Player>>,
    mut assist: ResMut<CameraAssist>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let dt = time.delta_secs();
    let Ok((body, riding)) = me.single() else {
        return;
    };
    let bus = riding.0.and_then(|car| cars.get(car).ok());
    let walked = assist.walked(body, riding, bus.map(|(driven, _)| driven));
    assist.untouched += dt;
    let pushed = !dialog.0 && controls.stick_value.length() >= STICK_DEADZONE;
    assist.pushed = if pushed { assist.pushed + dt } else { 0.0 };
    let riding = !dialog.0 && bus.is_some();
    assist.aboard = if riding { assist.aboard + dt } else { 0.0 };

    // Fixed over you in the car, nothing brings it round.
    if orbit.overhead {
        assist.snap = None;
        return;
    }
    if let Some(yaw) = assist.snap {
        if snap_to(&mut orbit, yaw, dt) {
            assist.snap = None;
        }
        return;
    }
    if dt <= 0.0 {
        return;
    }
    let turn = match bus {
        Some((driven, hull)) if riding => ride_turn(orbit.yaw, hull.facing(&driven.now), assist.ready()),
        _ if pushed && orbit.distance > 0.0 => recenter_turn(orbit.yaw, walked.xz() / dt, assist.ready()),
        _ => 0.0,
    } * dt;
    if turn != 0.0 {
        orbit.yaw += turn;
    }
}

/// Swings `orbit` on round to `yaw` for `dt` seconds, as a double tap asks:
/// tipped as it is when the app opens, or in your eyes, looking straight
/// ahead. Whether it is there.
fn snap_to(orbit: &mut OrbitCamera, yaw: f32, dt: f32) -> bool {
    let pitch = if orbit.distance <= 0.0 { 0.0 } else { CAMERA_PITCH };
    let (yaw_left, pitch_left) = (wrap_angle(yaw - orbit.yaw), pitch - orbit.pitch);
    if yaw_left.abs() < 1e-3 && pitch_left.abs() < 1e-3 {
        orbit.yaw = yaw;
        orbit.pitch = pitch;
        return true;
    }
    let share = 1.0 - (-SNAP_RATE * dt).exp();
    orbit.yaw += yaw_left * share;
    orbit.pitch += pitch_left * share;
    false
}

struct HudLayout {
    stick_center: Vec2,
    stick_radius: f32,
    jump_center: Vec2,
    jump_radius: f32,
}

/// Where the stick and the jump button are on the screen, in logical
/// pixels: the jump button in its own place, or, in a game of Bridge Wars,
/// where the sword stands there, the smaller one beside the sword
/// (`hud::ARENA_JUMP`, `hud::Layout::Arena`).
fn hud_layout(window: &Window, layout: hud::Layout) -> HudLayout {
    let width = window.width();
    let height = window.height();
    let vmin = width.min(height);
    let stick_size = vmin * (STICK_SIZE_VMIN / 100.0);
    let stick_radius = stick_size * 0.5;
    let corner = Vec2::new(
        width - width * (JUMP_RIGHT_VW / 100.0),
        height - height * (JUMP_BOTTOM_VH / 100.0),
    );
    let (jump_center, jump_radius) = match layout {
        hud::Layout::Town => {
            let jump_radius = vmin * (JUMP_SIZE_VMIN / 100.0) * 0.5;
            (corner - Vec2::splat(jump_radius), jump_radius)
        }
        hud::Layout::Arena => {
            let (right, bottom, size) = hud::ARENA_JUMP;
            let jump_radius = vmin * (size / 100.0) * 0.5;
            (
                corner - Vec2::new(right, bottom) * vmin / 100.0 - Vec2::splat(jump_radius),
                jump_radius,
            )
        }
    };
    HudLayout {
        stick_center: Vec2::new(
            width * (STICK_LEFT_VW / 100.0) + stick_radius,
            height - height * (STICK_BOTTOM_VH / 100.0) - stick_radius,
        ),
        stick_radius,
        jump_center,
        jump_radius,
    }
}

#[bevy_main]
pub fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window_settings()),
                ..default()
            })
            .set(GltfPlugin {
                // glTF calls +Z "forward"; Bevy calls -Z "forward". Without this
                // every imported model faces the opposite way from the Transform
                // that carries it. Turning it on means an object modelled facing
                // -Y in Blender (Numpad 1, the standard front view) also faces
                // `Transform::forward()` here.
                convert_coordinates: GltfConvertCoordinates {
                    rotate_scene_entity: true,
                    rotate_meshes: true,
                },
                ..default()
            })
            // The town reads no gamepad. Left in, this wakes a thread every
            // 8 ms on Windows to look for controllers. Put it back when there
            // is a use for it.
            .disable::<GilrsPlugin>()
            .set(LogPlugin {
                // An iPhone keeps its log where only a Mac can read it, so it
                // writes one of its own as well, which the Files app shows.
                #[cfg(target_os = "ios")]
                custom_layer: log_file::in_documents,
                ..default()
            }),
    )
    .init_resource::<TouchControls>()
    .init_resource::<OrbitCamera>()
    .init_resource::<CameraAssist>()
    .init_resource::<CloseUp>()
    .add_message::<PunchPressed>()
    .add_message::<Hit>()
    .add_message::<Landed>()
    .add_message::<Swung>()
    .add_systems(Startup, (setup_world, cluster_lights_on_the_cpu));
    // The screen's colours and words, from `assets/ui.ron`, read again as
    // it changes on desktop.
    ui::plugin(&mut app);
    // The town's clock, the server's online, which the day, the fountain's
    // jet and the cars go by.
    clock::plugin(&mut app);
    sky::plugin(&mut app);
    // The street lamps, lit through the night.
    lamps::plugin(&mut app);
    island::plugin(&mut app);
    // The town's fountain, running, and its jet.
    fountain::plugin(&mut app);
    // The cars, two buses, that drive round the town's roads, and carry
    // whoever rides them.
    car::plugin(&mut app);
    // The car you drive yourself, at the press of the face button.
    drive::plugin(&mut app);
    // The punch's range, drawn round you.
    range::plugin(&mut app);
    // Everything standing on the town comes from the town save, so the map
    // has to be there before the editor can be pointed at any of it.
    map::plugin(&mut app);
    editor::plugin(&mut app);
    // Who you are: the name and the character chosen the first time the app
    // was run, on pages that come before the town then. Before the lobby,
    // which seats you by them.
    signup::plugin(&mut app);
    // Everyone in the town, you included, with the computer in every seat no
    // person has taken.
    lobby::plugin(&mut app);
    ai::plugin(&mut app);
    // Online: the server, reached in the background, and everyone else in
    // your room of it.
    net::plugin(&mut app);
    tags::plugin(&mut app);
    // The settings, behind the gear top right, with your account among them.
    settings::plugin(&mut app);
    // Your balance, and the welcome.
    hud::plugin(&mut app);
    // Talking to the town's AIs and the House Builder.
    talk::plugin(&mut app);
    // The House Builder in the town, and the game they ask you to play: the
    // shop to build with, and what is built.
    builder::plugin(&mut app);
    shop::plugin(&mut app);
    build::plugin(&mut app);
    // Bridge Wars, on the PvP map, the bridges built in it as you walk, and
    // the game selector both games are chosen from.
    arena::plugin(&mut app);
    fire::plugin(&mut app);
    loot::plugin(&mut app);
    market::plugin(&mut app);
    utility::plugin(&mut app);
    gear::plugin(&mut app);
    bridge::plugin(&mut app);
    minimap::plugin(&mut app);
    games::plugin(&mut app);
    // Standing in a house in House Builder, whatever stands between the
    // camera and you, see-through, and out of a tap's reach.
    see_through::plugin(&mut app);
    // The music.
    sound::plugin(&mut app);

    // The stick and the jump button, on every platform: fingers hold them
    // on a phone, the mouse on desktop (below).
    app.add_systems(Startup, setup_hud)
        .add_systems(Update, (update_joystick_knob, update_jump_visual, hide_touch_hud));

    #[cfg(any(target_os = "android", target_os = "ios"))]
    app.add_systems(
        Update,
        // This is also where a touch is handed to the editor, so it has to
        // land before the editor reads the pointer.
        read_touch_controls.before(editor::EditSystems),
    );

    #[cfg(target_os = "android")]
    app.add_systems(PreUpdate, keep_ui_size);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        app.add_systems(Startup, lock_cursor).add_systems(
            Update,
            (
                // Around House Builder's keys: Escape is a dialog's no while
                // one is up, and the cursor is let go for as long as it is.
                grab_cursor.before(builder::GameSystems),
                cursor_for_dialogs.after(builder::GameSystems),
                read_mouse_look,
                read_mouse_zoom,
                // The mouse on the stick and the jump button, before the
                // editor reads it for the town, so that a press on them is
                // not a tap there.
                read_mouse_controls.before(editor::read_desktop_pointer),
            ),
        );
        // No more frames than a fast screen needs.
        pace::plugin(&mut app);
    }

    app.add_systems(
        Update,
        (
            read_player_input,
            // Driving the car, the stick is the car's, and the car's move is
            // the body's.
            drive::steer,
            start_punch,
            // Building a bridge, a plank is laid under where you are about
            // to step before you step there.
            bridge::lay,
            move_bodies,
            drive::settle,
            drive::place,
            animate_walk,
            assist_camera,
            follow_camera,
        )
            .chain(),
    );
    // Last, so that every schedule the plugins above made is there to change.
    one_thread_per_world(&mut app);
    app.run();
}

/// Runs every schedule's systems one after another, on the thread that runs
/// the schedule, instead of handing them out to Bevy's pool of worker threads.
///
/// This world is small, and waking the workers, hundreds of times a frame, cost
/// more than the work they were woken for. At 120 frames a second the phone's
/// four workers took 72% of a core between them; with this the whole game
/// took 87% of a core instead of 121% (the laptop: 46% instead of 87%), and
/// both still made every frame. The game world and the render world still run
/// side by side, on a thread each, and systems that split their own work
/// across the pool (`par_iter`) still do.
fn one_thread_per_world(app: &mut App) {
    use bevy::ecs::schedule::{Schedules, SingleThreadedExecutor};
    fn each_schedule(world: &mut World) {
        for (_, schedule) in world.resource_mut::<Schedules>().iter_mut() {
            schedule.set_executor(SingleThreadedExecutor::new());
        }
    }
    each_schedule(app.world_mut());
    // Before `app.run()` moves the render world onto its own thread. Not on
    // an iPhone, nor a Mac: there one of the render world's systems has to run
    // on the main thread, the one that makes the window's surface, which
    // touches its UIView. Bevy's own executor hands such a system back to the
    // main thread, and one thread cannot: it ran on the render thread, which
    // panicked on the first frame ("can only access UIView on the main
    // thread"), Bevy asked to quit, iOS does not let an app, and 1.0.1 to
    // 1.0.3 opened on a black screen and stayed there (2026-10-08, read off
    // the phone's own log.txt).
    #[cfg(not(target_vendor = "apple"))]
    {
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            each_schedule(render.world_mut());
        }
    }
}

/// How much of the screen's width and height the game is drawn at on Android.
/// `MainActivity` has the phone draw it that much smaller and stretch it over
/// the screen (its `RENDER_SCALE`, which this must match), and hands touches
/// over scaled down to suit.
#[cfg(target_os = "android")]
pub(crate) const RENDER_SCALE: f32 = 0.75;

/// Keeps every button and line of text the size it is on the full screen,
/// though the game is drawn smaller (`RENDER_SCALE`): there are that many
/// fewer pixels to each of the UI's.
#[cfg(target_os = "android")]
fn keep_ui_size(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    for mut window in &mut windows {
        let scale = window.resolution.base_scale_factor() * RENDER_SCALE;
        if window.resolution.scale_factor_override() != Some(scale) {
            window.resolution.set_scale_factor_override(Some(scale));
        }
    }
}

fn window_settings() -> Window {
    Window {
        title: "동그라미타운".into(),
        resolution: WindowResolution::new(1280, 720),
        #[cfg(any(target_os = "android", target_os = "ios"))]
        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
        #[cfg(any(target_os = "android", target_os = "ios"))]
        resizable: true,
        // No pinch or rotation recognizers on iOS. A UIKit gesture recognizer
        // cancels the touches it recognizes its gesture in (winit 0.30 leaves
        // `cancelsTouchesInView` as UIKit has it, on), and two thumbs, one on
        // the stick and one on the jump button or turning the camera, read as
        // a pinch or a turn once either moves: the stick goes dead, mid-jump
        // as like as not, until it is lifted and put down again. That is the
        // likeliest reason a running jump onto the fountain fell short on the
        // iPhone (Hajun, iOS 1.0.2 build 15), read off winit's source, not
        // yet seen on the phone. A pinch zooms through `read_touch_controls`,
        // as it always has on Android.
        #[cfg(target_os = "ios")]
        prefers_home_indicator_hidden: true,
        #[cfg(target_os = "ios")]
        prefers_status_bar_hidden: true,
        ..default()
    }
}

/// The camera. The players, you among them, come from the lobby
/// (`lobby::spawn_players`).
fn setup_world(mut commands: Commands) {
    commands.spawn((
        ThirdPersonCamera,
        Camera3d::default(),
        IsDefaultUiCamera,
        Projection::from(PerspectiveProjection {
            fov: CAMERA_FOV,
            far: CAMERA_FAR,
            ..default()
        }),
        Transform::from_xyz(0.0, 4.0, CAMERA_DISTANCE)
            .looking_at(Vec3::new(0.0, LOOK_HEIGHT, 0.0), Vec3::Y),
        // Bevy drives its render phases with indirect draws and GPU culling
        // wherever the adapter claims to support them. The Adreno 840 claims
        // it, but nothing queued into the sorted transparent phase then reaches
        // the screen — the sea, the clouds, the sun and moon glow and the stars
        // were all missing, with no validation error to show for it, while the
        // opaque phase drew correctly. This drops the camera back to direct
        // draws, which costs nothing at this scene's size and brings the
        // transparent pass back. Verified on an SM-S948N.
        #[cfg(any(target_os = "android", target_os = "ios"))]
        NoIndirectDrawing,
        // Bevy's own clusters: the view cut into a few thousand cells, each
        // told which street lamps reach it (`lamps`), so that a spot is lit
        // by the few lamps near it rather than every lamp in sight. Until the
        // lamps (2026-10-16) the world had no light to sort, and the camera
        // had one cell, `ClusterConfig::Single`. Not `ClusterConfig::None`:
        // Bevy 0.19 then fails to create its "clustering dummy texture" and
        // quits on the first frame.
    ));
}

/// Sorts lights into the camera's clusters on the CPU, as Bevy already does on
/// Android, rather than on the GPU, as it does everywhere else.
///
/// The only lights it sorts are the street lamps' (`lamps`), and only at
/// night: the sun, the moon and the fill are directional lights, which are
/// never clustered, and by day the lamps are out. So by day there is nothing
/// to sort, which on the CPU is nothing at all, where on the GPU it is
/// still compute and raster passes every frame; at night, a couple of dozen
/// lights. Every platform takes the one path Android always has. It was
/// turned off on 2026-10-08 as the suspect for the iPhone's black screen,
/// being the one path Bevy 0.19 takes on an iPhone but neither on Android
/// nor in the iOS simulator; it was not that (`one_thread_per_world` was),
/// and it stays off for what it saves.
fn cluster_lights_on_the_cpu(settings: Option<ResMut<GlobalClusterSettings>>) {
    if let Some(mut settings) = settings {
        settings.gpu_clustering = None;
    }
}

/// The stick and the jump button: a see-through grey disc each, with no rim,
/// and on them the stick's knob and the jump button's arrow in a see-through
/// white, a little greyed (Hajun, 2026-10-31; until then the stick was white
/// with a white rim, the jump button yellow with an arrow drawn in strokes).
/// Neither takes a press for itself: `read_touch_controls` knows where they
/// are, and on desktop `read_mouse_controls`, which drives them with the
/// mouse (since 2026-11-06, Hajun: "put on screen joystick and jump button
/// on windows version too"; until then they were a phone's alone).
fn setup_hud(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let colours = ui::ui().colours.hud;
    commands
        .spawn((
            TouchHud,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Vw(STICK_LEFT_VW),
                bottom: Val::Vh(STICK_BOTTOM_VH),
                width: Val::VMin(STICK_SIZE_VMIN),
                height: Val::VMin(STICK_SIZE_VMIN),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(colours.button),
            Pickable::IGNORE,
        ))
        .with_child((
            JoystickKnob,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(42.0),
                height: Val::Percent(42.0),
                left: Val::Percent(29.0),
                top: Val::Percent(29.0),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(colours.button_mark),
            Pickable::IGNORE,
        ));

    commands.spawn((
        TouchHud,
        JumpButtonVisual,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Vw(JUMP_RIGHT_VW),
            bottom: Val::Vh(JUMP_BOTTOM_VH),
            width: Val::VMin(JUMP_SIZE_VMIN),
            height: Val::VMin(JUMP_SIZE_VMIN),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(colours.button),
        Pickable::IGNORE,
        children![(
            ImageNode::new(images.add(hud::jump_arrow())).with_color(colours.button_mark),
            Node {
                width: Val::Percent(hud::JUMP_ARROW),
                height: Val::Percent(hud::JUMP_ARROW),
                ..default()
            },
            Pickable::IGNORE,
        )],
    ));
}

fn in_circle(point: Vec2, center: Vec2, radius: f32) -> bool {
    point.distance_squared(center) <= radius * radius
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[allow(clippy::too_many_arguments)]
fn read_touch_controls(
    time: Res<Time>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    mut controls: ResMut<TouchControls>,
    mut orbit: ResMut<OrbitCamera>,
    mut assist: ResMut<CameraAssist>,
    me: Query<&Transform, With<Player>>,
    menu: Res<editor::EditMenu>,
    presses: hud::Presses,
    mut pointer: ResMut<editor::EditPointer>,
    buttons: Res<hud::Layout>,
) {
    controls.jump_just_pressed = false;
    pointer.clear_edges();
    let Ok(window) = windows.single() else {
        return;
    };
    let layout = hud_layout(window, *buttons);
    let now = time.elapsed_secs();

    let released = |id: u64| {
        touches.just_released(id) || touches.just_canceled(id) || touches.get_pressed(id).is_none()
    };

    if let Some(id) = controls.stick_id
        && released(id)
    {
        controls.stick_id = None;
        controls.stick_value = Vec2::ZERO;
    }
    if let Some(id) = controls.look_id
        && released(id)
    {
        controls.look_id = None;
        // A tap on nothing, and the second of two: the camera swings round
        // behind you, as you stand at this moment.
        let near = window.width().min(window.height()) * DOUBLE_TAP_NEAR;
        if let Some((down, at)) = controls.look_tap.take()
            && now - down <= TAP_SECS
            && controls.taps.tap(down, now, at, near)
            && let Ok(body) = me.single()
        {
            assist.snap_behind(body);
        }
        if let Some(pinch_id) = controls.pinch_id
            && let Some(touch) = touches.get_pressed(pinch_id)
        {
            controls.look_id = Some(pinch_id);
            controls.look_last = touch.position();
            controls.pinch_id = None;
            controls.pinch_last_dist = 0.0;
        }
    }
    if let Some(id) = controls.pinch_id
        && released(id)
    {
        controls.pinch_id = None;
        controls.pinch_last_dist = 0.0;
        if let Some(look_id) = controls.look_id
            && let Some(touch) = touches.get_pressed(look_id)
        {
            controls.look_last = touch.position();
        }
    }
    if let Some(id) = controls.jump_id
        && released(id)
    {
        controls.jump_id = None;
    }
    if let Some(id) = controls.edit_id
        && released(id)
    {
        controls.edit_id = None;
        pointer.release();
    }

    for touch in touches.iter_just_pressed() {
        let pos = touch.position();
        let id = touch.id();
        // Every press dismisses an open menu unless it is on one of its
        // buttons, so the editor hears about all of them.
        pointer.note_press();
        // A press on a button over the world — the one top right, House
        // Builder's, the dark behind a dialog — is the button's alone. It
        // hears about it through picking, the same way it hears a mouse.
        if presses.claimed(window, pos) {
            continue;
        }
        // A press on the menu, or on what it is open on to drag it, belongs to
        // the editor alone. The menu's buttons come before the stick and the
        // jump button, and those before the thing itself: something as big as
        // a house right in front of you is drawn under both of them, and they
        // still have to work.
        let on_stick = in_circle(pos, layout.stick_center, layout.stick_radius * 1.15);
        let on_jump = in_circle(pos, layout.jump_center, layout.jump_radius * 1.2);
        let for_editor =
            menu.on_button(window, pos) || (!on_stick && !on_jump && menu.on_thing(pos));
        if controls.edit_id.is_none() && for_editor {
            controls.edit_id = Some(id);
            pointer.warp(pos);
            pointer.press();
        } else if controls.stick_id.is_none() && on_stick {
            controls.stick_id = Some(id);
        } else if controls.jump_id.is_none() && on_jump {
            controls.jump_id = Some(id);
            controls.jump_just_pressed = true;
        } else if controls.look_id.is_none() {
            controls.look_id = Some(id);
            controls.look_last = pos;
            controls.look_tap = Some((now, pos));
            // The same drag doubles as the editor's pointer: a short one is a
            // tap on whatever is under it, a long one is a look.
            if controls.edit_id.is_none() {
                controls.edit_id = Some(id);
                pointer.warp(pos);
                pointer.press();
            }
        } else if controls.pinch_id.is_none() {
            controls.pinch_id = Some(id);
            controls.pinch_last_dist = 0.0;
            controls.look_tap = None;
        }
    }

    if let Some(id) = controls.edit_id
        && let Some(touch) = touches.get_pressed(id)
    {
        pointer.track(touch.position());
    }

    if let Some(id) = controls.stick_id
        && let Some(touch) = touches.get_pressed(id)
    {
        let delta = touch.position() - layout.stick_center;
        let raw = Vec2::new(delta.x, -delta.y) / layout.stick_radius;
        controls.stick_value = raw.clamp_length_max(1.0);
    } else {
        controls.stick_value = Vec2::ZERO;
    }

    let look_pos = controls
        .look_id
        .and_then(|id| touches.get_pressed(id))
        .map(|touch| touch.position());
    let pinch_pos = controls
        .pinch_id
        .and_then(|id| touches.get_pressed(id))
        .map(|touch| touch.position());

    if let (Some(a), Some(b)) = (look_pos, pinch_pos) {
        let dist = a.distance(b);
        if dist > 8.0 && controls.pinch_last_dist > 8.0 {
            let ratio = (controls.pinch_last_dist / dist).clamp(0.82, 1.22);
            orbit.zoom_by_ratio(ratio);
        }
        controls.pinch_last_dist = dist;
        controls.look_last = a;
    } else {
        controls.pinch_last_dist = 0.0;
        if let Some(pos) = look_pos {
            let delta = pos - controls.look_last;
            controls.look_last = pos;
            orbit.turn(delta * TOUCH_LOOK_GAIN);
        }
    }
    // Gone further than a tap does, it is a look.
    if let Some((_, at)) = controls.look_tap
        && look_pos.is_some_and(|pos| pos.distance(at) > editor::TAP_SLOP)
    {
        controls.look_tap = None;
    }
    // A finger on the camera holds off its coming round behind you.
    if controls.look_id.is_some() || controls.pinch_id.is_some() {
        assist.touched();
    }
}

/// Takes the cursor for looking round, as the app opens. `RT_FREE_CURSOR=1`
/// leaves it alone, for running copies of the game side by side to try
/// playing online: each would otherwise take the mouse from the other. Nor
/// is it taken with the camera fixed over you, where there is nothing to
/// look round with ([`cursor_for_dialogs`] lets it go for as long as it
/// is).
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn lock_cursor(orbit: Res<OrbitCamera>, mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if std::env::var_os("RT_FREE_CURSOR").is_some() || orbit.overhead {
        return;
    }
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    cursor.visible = false;
    cursor.grab_mode = CursorGrabMode::Locked;
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn grab_cursor(
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Res<hud::DialogUp>,
    orbit: Res<OrbitCamera>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // With a dialog up, Escape is its no (`builder`), and the cursor is the
    // dialog's to give back when it goes.
    if !keyboard.just_pressed(KeyCode::Escape) || dialog.0 {
        return;
    }
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    let locked = cursor.grab_mode == CursorGrabMode::Locked;
    // Fixed over you, the mouse has no camera to turn: Escape only ever
    // lets the cursor go.
    cursor.grab_mode = if locked || orbit.overhead {
        CursorGrabMode::None
    } else {
        CursorGrabMode::Locked
    };
    cursor.visible = locked || orbit.overhead;
}

/// Lets the cursor go while a dialog is up, so that its buttons can be
/// clicked, or while a piece is being put down in House Builder, so that it
/// can be dragged, or while the camera is fixed over you, in the car or in
/// a game of Bridge Wars (`arena`), where there is nothing to look round
/// with and the stick and the buttons want the mouse; and takes it back
/// once none of them is, if it was taken before.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn cursor_for_dialogs(
    dialog: Res<hud::DialogUp>,
    wants: Res<hud::WantsPointer>,
    orbit: Res<OrbitCamera>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut was_taken: Local<bool>,
    mut let_go: Local<bool>,
) {
    let free = dialog.0 || wants.0 || orbit.overhead;
    if free == *let_go {
        return;
    }
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    *let_go = free;
    if free {
        *was_taken = cursor.grab_mode == CursorGrabMode::Locked;
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    } else if std::mem::take(&mut *was_taken) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

/// The mouse turns the camera, while the cursor is taken for it. Moving it
/// is a hand on the camera, as a finger is on a phone: riding a bus, the
/// camera comes round behind the way the bus faces only once the mouse has
/// been still a while ([`assist_camera`]).
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_mouse_look(
    cursors: Query<&CursorOptions, With<PrimaryWindow>>,
    mut motion: MessageReader<MouseMotion>,
    mut orbit: ResMut<OrbitCamera>,
    mut assist: ResMut<CameraAssist>,
) {
    let looking = cursors
        .single()
        .map(|cursor| cursor.grab_mode == CursorGrabMode::Locked)
        .unwrap_or(false);
    if !looking {
        motion.clear();
        return;
    }
    for event in motion.read() {
        if event.delta != Vec2::ZERO {
            orbit.turn(event.delta);
            assist.touched();
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_mouse_zoom(
    dialog: Res<hud::DialogUp>,
    mut scroll: MessageReader<MouseWheel>,
    mut orbit: ResMut<OrbitCamera>,
) {
    // With a dialog up the wheel is its: the shop scrolls with it.
    if dialog.0 {
        scroll.clear();
        return;
    }
    for event in scroll.read() {
        let lines = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
        };
        orbit.zoom_by_ratio((1.0 - lines * ZOOM_WHEEL).clamp(0.7, 1.4));
    }
}

/// The mouse on the stick and the jump button, with the cursor free
/// (Escape), as a finger is on a phone (since 2026-11-06, Hajun: "put on
/// screen joystick and jump button on windows version too"): pressed on the
/// stick's disc, the mouse holds the stick, pushing it toward wherever the
/// cursor goes until the button comes up, and pressed on the jump button it
/// jumps, or boosts the car for as long as it is held. A press on either
/// is theirs, not a tap on the town (`editor::read_desktop_pointer` asks
/// [`TouchControls::has_the_mouse`]), and one on a button over the world
/// is the button's. With the cursor taken for looking round there is no
/// cursor to press with: the keys walk you, as ever.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_mouse_controls(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cursors: Query<&CursorOptions, With<PrimaryWindow>>,
    presses: hud::Presses,
    buttons: Res<hud::Layout>,
    mut controls: ResMut<TouchControls>,
) {
    controls.jump_just_pressed = false;
    let free = cursors
        .single()
        .is_ok_and(|cursor| cursor.grab_mode == CursorGrabMode::None);
    let Ok(window) = windows.single() else {
        return;
    };
    let position = window.cursor_position().filter(|_| free);
    if !mouse.pressed(MouseButton::Left) || position.is_none() {
        if controls.stick_id == Some(MOUSE_ID) {
            controls.stick_id = None;
            controls.stick_value = Vec2::ZERO;
        }
        if controls.jump_id == Some(MOUSE_ID) {
            controls.jump_id = None;
        }
    }
    let Some(pos) = position else {
        return;
    };
    let layout = hud_layout(window, *buttons);
    if mouse.just_pressed(MouseButton::Left) && !presses.claimed(window, pos) {
        if controls.stick_id.is_none() && in_circle(pos, layout.stick_center, layout.stick_radius * 1.15) {
            controls.stick_id = Some(MOUSE_ID);
        } else if controls.jump_id.is_none() && in_circle(pos, layout.jump_center, layout.jump_radius * 1.2) {
            controls.jump_id = Some(MOUSE_ID);
            controls.jump_just_pressed = true;
        }
    }
    if controls.stick_id == Some(MOUSE_ID) {
        let delta = pos - layout.stick_center;
        let raw = Vec2::new(delta.x, -delta.y) / layout.stick_radius;
        controls.stick_value = raw.clamp_length_max(1.0);
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl TouchControls {
    /// Whether the mouse is on the stick or the jump button, so that its
    /// press is not also a tap on the town.
    pub(crate) fn has_the_mouse(&self) -> bool {
        self.stick_id == Some(MOUSE_ID) || self.jump_id == Some(MOUSE_ID)
    }
}

fn update_joystick_knob(
    controls: Res<TouchControls>,
    mut knobs: Query<&mut Node, With<JoystickKnob>>,
) {
    let Ok(mut node) = knobs.single_mut() else {
        return;
    };
    let max = 29.0;
    let left = Val::Percent(max + controls.stick_value.x * max);
    let top = Val::Percent(max - controls.stick_value.y * max);
    // Only when the knob has moved: a changed node has the whole screen's
    // layout worked out again.
    if node.left != left || node.top != top {
        node.left = left;
        node.top = top;
    }
}

/// Lights the jump button up while it is held: the one in its own place,
/// and in a game of Bridge Wars the one beside the sword (`hud`).
fn update_jump_visual(
    controls: Res<TouchControls>,
    mut buttons: Query<&mut BackgroundColor, With<JumpButtonVisual>>,
) {
    let held = controls.jump_id.is_some();
    let colours = ui::ui().colours.hud;
    for mut color in &mut buttons {
        color.set_if_neq(BackgroundColor(if held {
            colours.button_held
        } else {
            colours.button
        }));
    }
}

/// Puts the stick and the jump button away while the camera is on someone
/// you talk to (`talk`): what they say is along the bottom of the screen.
/// And the jump button alone in a game of Bridge Wars, where the sword
/// stands in its place (`hud::Layout::Arena`).
fn hide_touch_hud(
    close_up: Res<CloseUp>,
    layout: Res<hud::Layout>,
    mut huds: Query<(&mut Visibility, Has<JumpButtonVisual>), With<TouchHud>>,
) {
    for (mut visibility, jump) in &mut huds {
        let hidden = close_up.on.is_some() || (jump && *layout == hud::Layout::Arena);
        visibility.set_if_neq(if hidden { Visibility::Hidden } else { Visibility::Inherited });
    }
}

fn wading(pos: Vec3) -> bool {
    pos.y < WATER_Y + 0.25
}

/// What the feet come to rest on at `pos`: the highest ground on the island,
/// or on the car, within a step of them, or, with none, the sea bed the
/// player wades on; and which it is.
///
/// The ground under the middle of the body, unless the ground under its
/// legs, round that, is higher by more than a slope makes it ([`EDGE_RISE`]):
/// then it is standing on an edge, its middle out past it, and the edge holds
/// it up. Until 2026-10-08 only the middle counted. A body that came down
/// with its legs on an edge and its middle just past it went on down past it,
/// half into it, and once the edge was over its knees, was shoved out
/// sideways and dropped (Hajun: it "glitch steps").
fn support(ground: &Ground, pos: Vec3, elapsed: f32) -> (f32, Underfoot) {
    // Over the void there is no sea bed: a body off the land falls for
    // good, until it is dead and put back (`lobby::respawn_the_fallen`).
    let sea = if ground.bottomless {
        f32::NEG_INFINITY
    } else {
        water_rest_y(elapsed)
    };
    let reach = pos.y + LAND_STEP_UP;
    let under = |at: Vec2| match ground.floor(at, reach) {
        Some((height, on)) if height > sea => (height, on),
        _ => (sea, Underfoot::Island),
    };
    let middle = under(pos.xz());
    // Standing on it, as nearly everyone nearly always is, that is all there
    // is to it: only a body off it, in the air or out over an edge, looks
    // under its legs. Beside the fountain's tower that is eight more looks
    // through its thousands of triangles.
    if pos.y <= middle.0 {
        return middle;
    }
    let legs = UNDER_THE_LEGS
        .iter()
        .map(|&round| under(pos.xz() + round))
        .fold(middle, |highest, next| if next.0 > highest.0 { next } else { highest });
    if legs.0 > middle.0 + EDGE_RISE { legs } else { middle }
}

/// Something that moves on an island and is stood on and bumped into as the
/// island is: the car (`car`). Its shape, in its own space, where it stands
/// this frame and where it stood the frame before.
#[derive(Clone, Copy)]
pub(crate) struct Deck<'a> {
    car: Entity,
    shape: &'a Island,
    now: car::Place,
    then: car::Place,
    /// How far from its own middle any of it reaches, across the ground.
    reach: f32,
}

impl Deck<'_> {
    /// Whether any of it is within `margin` of `at`, across the ground: most
    /// of the town never asks it anything.
    fn near(&self, at: Vec2, margin: f32) -> bool {
        at.distance_squared(self.now.at.xz()) <= (self.reach + margin).powi(2)
    }

    /// Where `at`, across the ground, is in its own space. It only ever turns
    /// about the upright, so a line straight up is one in its space too.
    fn local(&self, at: Vec2) -> Vec2 {
        self.now.inward(Vec3::new(at.x, self.now.at.y, at.y)).xz()
    }

    /// The highest of it at `at` no higher than `reach`.
    fn floor(&self, at: Vec2, reach: f32) -> Option<f32> {
        let low = self.now.at.y;
        if !self.near(at, 0.0) {
            return None;
        }
        self.shape.floor(self.local(at), reach - low).map(|height| height + low)
    }

    /// The lowest underside of it over `at` higher than `above`.
    fn ceiling(&self, at: Vec2, above: f32) -> Option<f32> {
        let low = self.now.at.y;
        if !self.near(at, 0.0) {
            return None;
        }
        self.shape.ceiling(self.local(at), above - low).map(|height| height + low)
    }

    /// Moves `transform` as it moved since the frame before: along with it,
    /// and turned as it turned.
    fn carry(&self, transform: &mut Transform) {
        let turned = (self.now.turn * self.then.turn.inverse()).normalize();
        transform.translation = self.now.at + turned * (transform.translation - self.then.at);
        transform.rotation = (turned * transform.rotation).normalize();
    }
}

/// What a body moves over: its island, and whatever moves on it.
#[derive(Clone, Copy)]
pub(crate) struct Ground<'a> {
    island: &'a Island,
    decks: &'a [Deck<'a>],
    /// Nothing under the island but the void (`island::bottomless`): off
    /// it a body falls, rather than coming down in the sea.
    bottomless: bool,
}

/// What a body stands on: its island, or one of the decks on it, by where it
/// comes in [`Ground::decks`].
#[derive(Clone, Copy, PartialEq, Debug)]
enum Underfoot {
    Island,
    Deck(usize),
}

impl<'a> Ground<'a> {
    /// The island alone, with nothing moving on it.
    #[cfg(test)]
    fn bare(island: &'a Island) -> Self {
        Self {
            island,
            decks: &[],
            bottomless: false,
        }
    }

    /// The highest ground at `at` no higher than `reach`, and what it is.
    fn floor(&self, at: Vec2, reach: f32) -> Option<(f32, Underfoot)> {
        let mut highest = self.island.floor(at, reach).map(|height| (height, Underfoot::Island));
        for (index, deck) in self.decks.iter().enumerate() {
            if let Some(height) = deck.floor(at, reach)
                && highest.is_none_or(|(high, _)| height > high)
            {
                highest = Some((height, Underfoot::Deck(index)));
            }
        }
        highest
    }

    /// The lowest underside over `at` higher than `above`.
    fn ceiling(&self, at: Vec2, above: f32) -> Option<f32> {
        self.decks
            .iter()
            .filter_map(|deck| deck.ceiling(at, above))
            .chain(self.island.ceiling(at, above))
            .reduce(f32::min)
    }

    /// How far a ball of `radius` gets from `from` toward `to` before it
    /// touches anything, or `None` if it gets all the way.
    fn sweep(&self, from: Vec3, to: Vec3, radius: f32) -> Option<f32> {
        self.decks
            .iter()
            .filter(|deck| deck.near(from.xz(), radius) || deck.near(to.xz(), radius))
            .filter_map(|deck| deck.shape.sweep(deck.now.inward(from), deck.now.inward(to), radius))
            .chain(self.island.sweep(from, to, radius))
            .reduce(f32::min)
    }
}

/// The cars, as bodies meet them: something to stand on that moves.
type Decks<'w, 's> = Query<'w, 's, (Entity, &'static Venue, &'static car::Hull, &'static car::Driven)>;

/// The decks among `cars` that are on `venue`.
fn decks_on<'a>(cars: &'a Decks, venue: Venue) -> Vec<Deck<'a>> {
    cars.iter()
        .filter(|(_, on, ..)| **on == venue)
        .map(|(car, _, hull, driven)| Deck {
            car,
            shape: &hull.shape,
            now: driven.now,
            then: driven.then,
            reach: hull.reach,
        })
        .collect()
}

/// The dry ground a body at `pos` would stand on — the highest within a step
/// of its feet — or `None` over open sea.
fn dry_ground(island: &Island, pos: Vec3) -> Option<f32> {
    island
        .floor(pos.xz(), pos.y + LAND_STEP_UP)
        .filter(|&ground| ground > WATER_Y)
}

/// Whether a body that cannot jump, walked from `from` to `to`, would be
/// stranded there: off dry land into the sea, or down off something higher
/// than it could step back up, like the fountain's rim into its basin.
fn strands(island: &Island, from: Vec3, to: Vec3) -> bool {
    let Some(here) = dry_ground(island, from) else {
        return false;
    };
    dry_ground(island, to).is_none_or(|there| here - there > LAND_STEP_UP)
}

/// The lowest underside over a body standing at `at` that is higher than its
/// head, at `head`: over every square of the lattice its walls are tested on
/// (`Island::walls`), so that no ceiling it stops short of can still reach
/// into it at the side.
fn headroom(ground: &Ground, at: Vec2, head: f32) -> Option<f32> {
    let reach = PLAYER_RADIUS + island::LATTICE * 0.5;
    [-reach, 0.0, reach]
        .into_iter()
        .flat_map(|x| [-reach, 0.0, reach].map(|z| at + Vec2::new(x, z)))
        .filter_map(|spot| ground.ceiling(spot, head - HEAD_CLEARANCE))
        .reduce(f32::min)
}

fn water_rest_y(elapsed: f32) -> f32 {
    WATER_Y - WATER_WADE + (elapsed * WATER_BOB_SPEED).sin() * WATER_BOB_AMP
}

fn y_overlap(a0: f32, a1: f32, b0: f32, b1: f32) -> bool {
    a0 < b1 && a1 > b0
}

fn aabb_world(center: Vec3, half: Vec3) -> (Vec3, Vec3) {
    (center - half, center + half)
}

fn standing_on_aabb(pos: Vec3, min: Vec3, max: Vec3) -> bool {
    pos.x >= min.x
        && pos.x <= max.x
        && pos.z >= min.z
        && pos.z <= max.z
        && pos.y >= max.y - LAND_STEP_UP
}

fn resolve_circle_aabb(pos: &mut Vec3, radius: f32, min: Vec3, max: Vec3) {
    let closest_x = pos.x.clamp(min.x, max.x);
    let closest_z = pos.z.clamp(min.z, max.z);
    let dx = pos.x - closest_x;
    let dz = pos.z - closest_z;
    let d2 = dx * dx + dz * dz;
    if d2 > 1e-8 {
        let d = d2.sqrt();
        if d < radius {
            let scale = (radius - d) / d;
            pos.x += dx * scale;
            pos.z += dz * scale;
        }
        return;
    }
    let left = pos.x - min.x;
    let right = max.x - pos.x;
    let back = pos.z - min.z;
    let fwd = max.z - pos.z;
    let nearest = left.min(right).min(back).min(fwd);
    if nearest == left {
        pos.x = min.x - radius;
    } else if nearest == right {
        pos.x = max.x + radius;
    } else if nearest == back {
        pos.z = min.z - radius;
    } else {
        pos.z = max.z + radius;
    }
}

fn resolve_cylinders(pos: &mut Vec3, radius: f32, height: f32, other: Vec3, other_radius: f32, other_height: f32) {
    if !y_overlap(pos.y, pos.y + height, other.y, other.y + other_height) {
        return;
    }
    let dx = pos.x - other.x;
    let dz = pos.z - other.z;
    let min_dist = radius + other_radius;
    let d2 = dx * dx + dz * dz;
    if d2 >= min_dist * min_dist {
        return;
    }
    if d2 <= 1e-8 {
        pos.x += min_dist;
        return;
    }
    let d = d2.sqrt();
    let scale = (min_dist - d) / d;
    pos.x += dx * scale;
    pos.z += dz * scale;
}

fn resolve_player_solids(pos: &mut Vec3, solids: &[(Vec3, Collider)], ground: &Ground) {
    for &(center, collider) in solids {
        match collider.shape {
            ColliderShape::Aabb { half_extents } => {
                let (min, max) = aabb_world(center, half_extents);
                if standing_on_aabb(*pos, min, max) {
                    continue;
                }
                if !y_overlap(pos.y, pos.y + PLAYER_HEIGHT, min.y, max.y) {
                    continue;
                }
                resolve_circle_aabb(pos, PLAYER_RADIUS, min, max);
            }
            ColliderShape::Cylinder { radius, height } => {
                resolve_cylinders(pos, PLAYER_RADIUS, PLAYER_HEIGHT, center, radius, height);
            }
        }
    }
    // Whatever on the island stands higher than a step above the feet: the
    // shore, seen from the water, and fences and trunks from anywhere. Up to
    // the hips only the legs meet it, and from there up the body and arms
    // too: standing on the fountain's middle bowl, the top bowl is at the
    // thighs, clear of the legs, where the whole width of the body would have
    // been shoved off it. The hips are rounded off, half way between the two,
    // so that a body coming down just past an edge is eased off it rather
    // than shoved all at once as the edge reaches its waist.
    let shape = |feet: f32| {
        [
            (LEG_RADIUS, feet + LAND_STEP_UP, feet + HIP_HEIGHT),
            ((LEG_RADIUS + PLAYER_RADIUS) * 0.5, feet + HIP_HEIGHT, feet + WAIST_HEIGHT),
            (PLAYER_RADIUS, feet + WAIST_HEIGHT, feet + PLAYER_HEIGHT),
        ]
    };
    let body = shape(pos.y);
    resolve_walls(pos, ground.island, &body);
    // And the car, the same, in its own space, where its sides are square to
    // the lattice they are tested on. Moving into a body, it shoves the body
    // out of its way.
    for deck in ground.decks {
        if !deck.near(pos.xz(), PLAYER_RADIUS + island::LATTICE) {
            continue;
        }
        let mut local = deck.now.inward(*pos);
        let was = local;
        let body = shape(local.y);
        resolve_walls(&mut local, deck.shape, &body);
        if local != was {
            *pos = deck.now.outward(local);
        }
    }
    pos.x = pos.x.clamp(-OCEAN_LIMIT, OCEAN_LIMIT);
    pos.z = pos.z.clamp(-OCEAN_LIMIT, OCEAN_LIMIT);
}

/// How high over its feet the middle of a body walking on a car is, which
/// is swept through the car's walls as wide as the body (`keep_off_deck_walls`):
/// its body from the waist to the head.
const SWEPT_AT: f32 = (WAIST_HEIGHT + PLAYER_HEIGHT) * 0.5;
/// How far short of a wall a body swept into it stops, in metres.
const SWEPT_SHORT: f32 = 0.001;

/// Keeps a body walking from `from` to `pos` from going through the walls of
/// any car on the island, however thin. The lattice walls are tested on
/// (`resolve_walls`) has a square every 0.25 m, and the bus's walls are a
/// centimetre thick: a body walked out of the bus through its side, and in
/// the same way (Hajun, 2026-10-24). So the middle of the body, as wide as
/// it is, is swept from where it was to where it is going against every
/// triangle of each car near it, in the car's own space, and what it meets
/// it slides along (`slide_past`). Only the body's own walking is swept: a
/// car driving into someone standing still is no more than it was.
fn keep_off_deck_walls(from: Vec3, pos: &mut Vec3, ground: &Ground) {
    for deck in ground.decks {
        if !deck.near(pos.xz(), PLAYER_RADIUS + island::LATTICE) {
            continue;
        }
        let (start, end) = (deck.now.inward(from), deck.now.inward(*pos));
        let reached = slide_past(deck.shape, start, end);
        if reached != end {
            *pos = deck.now.outward(reached);
        }
    }
}

/// How far a body walking from `start` to `end`, at its feet, gets without
/// the middle of it, as wide as it is, going into anything of `shape`:
/// straight there, or failing that along each of the shape's two ways across
/// the ground in turn, so that it slides along a wall square to one of them,
/// as the bus's sides, front and back are, rather than stopping dead at it.
fn slide_past(shape: &Island, start: Vec3, end: Vec3) -> Vec3 {
    let middle = Vec3::Y * SWEPT_AT;
    let reach = |from: Vec3, to: Vec3| -> Vec3 {
        let length = from.distance(to);
        if length < 1e-6 {
            return to;
        }
        match shape.sweep(from + middle, to + middle, PLAYER_RADIUS) {
            None => to,
            Some(met) => from.lerp(to, (met - SWEPT_SHORT).max(0.0) / length),
        }
    };
    let straight = reach(start, end);
    if straight == end {
        return end;
    }
    let across = reach(start, Vec3::new(end.x, end.y, start.z));
    reach(across, Vec3::new(across.x, end.y, end.z))
}

/// Keeps a body at `pos`, of `shape` (`Island::walls_round`), out of
/// whatever on `island` stands higher than a step over its feet.
fn resolve_walls(pos: &mut Vec3, island: &Island, shape: &[(f32, f32, f32)]) {
    for (min, max, radius) in island.walls_round(pos.xz(), shape) {
        let (min, max) = (Vec3::new(min.x, 0.0, min.y), Vec3::new(max.x, 0.0, max.y));
        resolve_circle_aabb(pos, radius, min, max);
    }
}

/// A limb's turn `swing` of the way through a stride, `weight` of a walk,
/// in its body's space: forward is a turn about the body's side to side
/// axis, x, on top of how the limb stands, turned `stance` from the model's
/// rest about the body's front to back axis, z ([`stood`],
/// [`WalkBone::stance`]). The left arm swings against the left leg, as arms
/// do, and a knee bends back, the foot coming up behind as the leg goes
/// back.
fn walk_offset(kind: WalkBoneKind, swing: f32, weight: f32, stance: f32) -> Quat {
    let forward = |angle: f32| Quat::from_axis_angle(Vec3::X, angle) * stood(stance);
    match kind {
        WalkBoneKind::ThighL => forward(swing * WALK_THIGH * weight),
        WalkBoneKind::ThighR => forward(-swing * WALK_THIGH * weight),
        WalkBoneKind::ShinL => forward(swing.min(0.0) * WALK_SHIN * weight),
        WalkBoneKind::ShinR => forward(-swing.max(0.0) * WALK_SHIN * weight),
        WalkBoneKind::ArmL => forward(-swing * WALK_ARM * weight),
        WalkBoneKind::ArmR => forward(swing * WALK_ARM * weight),
    }
}

/// A limb stood as it should from the model's rest, in its body's space:
/// turned `stance` about the body's front to back axis
/// ([`stance_from_rest`]).
fn stood(stance: f32) -> Quat {
    Quat::from_axis_angle(Vec3::Z, stance)
}

/// How far a limb is turned about the body's front to back axis from how it
/// lies at rest (`frame`, the bone along its own y) to stand as it should:
/// an arm down, to hang [`ARM_HANG_BELOW`] under level, a thigh in, to stand
/// [`LEG_SPLAY`] out from straight down; the left, on the body's -x side,
/// turned one way and the right the other, and never up or out, however
/// the model rests them. 0 for a shin, which goes with its thigh.
fn stance_from_rest(kind: WalkBoneKind, frame: Quat) -> f32 {
    let (side, arm) = match kind {
        WalkBoneKind::ArmL => (1.0, true),
        WalkBoneKind::ArmR => (-1.0, true),
        WalkBoneKind::ThighL => (1.0, false),
        WalkBoneKind::ThighR => (-1.0, false),
        WalkBoneKind::ShinL | WalkBoneKind::ShinR => return 0.0,
    };
    let along = frame * Vec3::Y;
    if arm {
        // How far under level the arm is, seen from the front.
        let below = (-along.y).atan2(along.x.abs());
        side * (ARM_HANG_BELOW - below).max(0.0)
    } else {
        // How far out from straight down the leg is, seen from the front.
        let out = (-side * along.x).atan2(-along.y);
        side * (out - LEG_SPLAY).max(0.0)
    }
}

/// Where a punching arm is, `reach` of the way out, 0 to 1, in place of its
/// walk: hung as the walk hangs it, by `stance`, and swung straight out in
/// front of you about the shoulder, the whole way at full reach. Forward is
/// one turn for both arms in the body's space (the walk swings the right
/// one the other way only to swing it against the left).
fn punch_offset(kind: WalkBoneKind, stance: f32, reach: f32) -> Option<Quat> {
    if !matches!(kind, WalkBoneKind::ArmL | WalkBoneKind::ArmR) {
        return None;
    }
    Some(Quat::from_axis_angle(Vec3::X, PUNCH_REACH * reach) * stood(stance))
}

/// Where an arm carrying a plank is ([`Carrying`]): held out in front, a
/// little under level, in place of its walk, as a punch is held at
/// [`CARRY_REACH`] of its reach.
fn carry_offset(kind: WalkBoneKind, stance: f32) -> Option<Quat> {
    punch_offset(kind, stance, CARRY_REACH / PUNCH_REACH)
}

/// Where a drinker's right arm is ([`Drinking`]), `lifted` of the way up,
/// 0 to 1, in place of its walk: turned from where it hangs, as this
/// model's does (`frame`, how the bone lies at rest, stood by `stance`),
/// to point at the mouth ([`DRINK_REACH`]).
fn drink_offset(kind: WalkBoneKind, frame: Quat, stance: f32, lifted: f32) -> Option<Quat> {
    if !matches!(kind, WalkBoneKind::ArmR) {
        return None;
    }
    let hung = stood(stance) * frame * Vec3::Y;
    let up = Quat::from_rotation_arc(hung.normalize_or(Vec3::NEG_Y), DRINK_REACH.normalize());
    Some(Quat::IDENTITY.slerp(up, lifted.clamp(0.0, 1.0)) * stood(stance))
}

/// How far a slash has got `age` seconds in: how far through its sweep, 0
/// to 1, in the time a punch goes out (the file's `melee.swing_out`), and
/// how far back to the ready it has come since, 0 to 1, in the time a
/// punch comes back (`melee.swing_back`).
fn slash_at(melee: &ui::Melee, age: f32) -> (f32, f32) {
    let swept = (age / melee.swing_out).clamp(0.0, 1.0);
    let back = ((age - melee.swing_out) / melee.swing_back).clamp(0.0, 1.0);
    (swept, back)
}

/// How far round the body's upright a slash has turned, `swept` of the way
/// through, 0 to 1: eased, so that it whips through the middle.
fn slash_yaw(swept: f32) -> f32 {
    let eased = swept * swept * (3.0 - 2.0 * swept);
    SLASH_FROM + (SLASH_TO - SLASH_FROM) * eased
}

/// Where the right arm of a body with a sword is, in place of its walk
/// and of a punch: at the ready, a little forward of hanging; or, in a
/// `slash` ([`slash_at`]), raised and swept across in front of it, and
/// then back to the ready.
fn sword_offset(kind: WalkBoneKind, stance: f32, slash: Option<(f32, f32)>) -> Option<Quat> {
    if !matches!(kind, WalkBoneKind::ArmR) {
        return None;
    }
    let ready = Quat::from_axis_angle(Vec3::X, SWORD_READY) * stood(stance);
    let Some((swept, back)) = slash else {
        return Some(ready);
    };
    let out = Quat::from_rotation_y(slash_yaw(swept)) * Quat::from_axis_angle(Vec3::X, SLASH_RAISE) * stood(stance);
    Some(out.slerp(ready, back))
}

/// Which way the blade of the sword in a body's right hand points, in the
/// body's space (`gear`): at the ready, out in front, a little up and out
/// to the right, where the camera over you sees all of it; in a `slash`,
/// level, the way the arm reaches as it sweeps ([`sword_offset`]), and
/// back to the ready after.
pub(crate) fn blade_way(slash: Option<(f32, f32)>) -> Vec3 {
    let ready = Vec3::new(0.14, 0.26, -1.0).normalize();
    let Some((swept, back)) = slash else {
        return ready;
    };
    let out = Quat::from_rotation_y(slash_yaw(swept)) * Vec3::NEG_Z;
    out.lerp(ready, back).normalize_or(ready)
}

/// How far into its slash a body with a sword is, if it is in one
/// ([`slash_at`]), for whoever draws the sword (`gear`).
pub(crate) fn slashing(melee: &ui::Melee, punch: Option<&Punching>) -> Option<(f32, f32)> {
    punch.filter(|punch| punch.age < melee.swing()).map(|punch| slash_at(melee, punch.age))
}

/// The limb a node of a body's model is, by its name: the bones of
/// `playercharacter.glb`, a Rigify metarig's (Hajun's, 2026-11-01; until
/// then `limbperson.glb`'s own, `Bone.005.L.001` and the like). The limbs'
/// meshes hang off these: the thighs and shins, and the upper arms, which
/// take the forearms and hands round with them.
fn walk_bone_kind(name: &str) -> Option<WalkBoneKind> {
    Some(match name {
        "thigh.L" => WalkBoneKind::ThighL,
        "thigh.R" => WalkBoneKind::ThighR,
        "shin.L" => WalkBoneKind::ShinL,
        "shin.R" => WalkBoneKind::ShinR,
        "upper_arm.L" => WalkBoneKind::ArmL,
        "upper_arm.R" => WalkBoneKind::ArmR,
        _ => return None,
    })
}

fn bind_walk_bones(
    ready: On<WorldInstanceReady>,
    children: Query<&Children>,
    parts: Query<(Option<&Name>, &Transform)>,
    mut commands: Commands,
) {
    let bound = bind_limbs(ready.entity, &children, &parts, &mut commands);
    if bound < 6 {
        warn!("walk bones: bound {bound}/6 on player {:?}", ready.entity);
    }
}

/// Hands the limbs under `owner` to its [`WalkCycle`], which swings them as it
/// walks and, standing, hangs its arms at its sides. Each is bound with how
/// it lies at rest in the body's space, the turns of every node from the
/// body down to it (the model's half turn round among them,
/// `GltfConvertCoordinates`), so that it can be swung about the body's axes
/// whatever way its rig's go. Returns how many were found.
pub(crate) fn bind_limbs(
    owner: Entity,
    children: &Query<&Children>,
    parts: &Query<(Option<&Name>, &Transform)>,
    commands: &mut Commands,
) -> usize {
    let mut bound = 0;
    let mut below: Vec<(Entity, Quat)> = vec![(owner, Quat::IDENTITY)];
    while let Some((above, frame)) = below.pop() {
        let Ok(nodes) = children.get(above) else {
            continue;
        };
        for entity in nodes.iter() {
            let Ok((name, transform)) = parts.get(entity) else {
                continue;
            };
            let frame = frame * transform.rotation;
            below.push((entity, frame));
            let Some(kind) = name.and_then(|name| walk_bone_kind(name.as_str())) else {
                continue;
            };
            commands.entity(entity).insert(WalkBone {
                kind,
                rest_rotation: transform.rotation,
                frame,
                stance: stance_from_rest(kind, frame),
                owner,
            });
            bound += 1;
        }
    }
    bound
}

/// Moves `transform` `distance` along `world_dir`, across the ground, turning
/// it to face that way over `dt`; or to face `face`, if there is one, at
/// once: the body in the car faces the way the car does, reversing too, and
/// the car turns as smoothly as it steers.
fn apply_planar_move(transform: &mut Transform, world_dir: Vec3, distance: f32, dt: f32, face: Option<Vec3>) {
    let mut dir = world_dir;
    dir.y = 0.0;
    let Some(dir) = dir.try_normalize() else {
        return;
    };
    let facing = face.and_then(|face| face.with_y(0.0).try_normalize()).unwrap_or(dir);
    let target = Transform::from_translation(transform.translation)
        .looking_at(transform.translation + facing, Vec3::Y)
        .rotation;
    let angle = transform.rotation.angle_between(target);
    if face.is_some() || angle <= f32::EPSILON {
        transform.rotation = target;
    } else {
        let step = (TURN_SPEED * dt / angle).min(1.0);
        transform.rotation = transform.rotation.slerp(target, step);
    }
    transform.translation += dir * distance;
}

/// This device's stick or keys, as the [`Intent`] of its [`Player`].
fn read_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<TouchControls>,
    dialog: Res<hud::DialogUp>,
    bridging: Res<bridge::Bridging>,
    mut ran_out: ResMut<bridge::RanOut>,
    (outfit, clock): (Res<arena::Outfit>, Res<arena::FightTime>),
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    mut players: Query<(&Transform, &Venue, &mut Intent, &mut WalkCycle, Has<Dead>, Has<Drinking>), With<Player>>,
) {
    let Ok((transform, &venue, mut intent, mut walk, dead, drinking)) = players.single_mut() else {
        return;
    };
    // A dialog is up: you stand and wait for it to be answered. Dead, you
    // stand where you died until you are put back (`arena`).
    if dialog.0 || dead {
        *intent = Intent::default();
        walk.speed = 0.0;
        return;
    }
    // In a game of Bridge Wars everyone runs (`arena`); carrying a plank,
    // at half the pace (`bridge`; Hajun, 2026-11-09: "when bride mode is
    // toggled, decrease player speed to 1/2").
    let sprint = if venue == Venue::Arena { ui::ui().arena.run_speed / MOVE_SPEED } else { 1.0 };
    let sprint = if bridging.0 { sprint * bridge::CARRY_SLOW } else { sprint };
    // The speed potion, while it lasts; and drinking any potion, at half
    // the pace (`utility`, `arena::Outfit`).
    let rules_fast = roundtown_net::bridge_wars::FAST;
    let sprint = if outfit.kit.fast(clock.0) { sprint * rules_fast } else { sprint };
    let sprint = if drinking { sprint * roundtown_net::bridge_wars::DRINK_SLOW } else { sprint };
    let slowed = wading(transform.translation);
    let cam_forward = cameras.single().map_or(Vec3::NEG_Z, camera_ahead);
    let cam_right = cam_forward.cross(Vec3::Y);

    let mut wish = Vec3::ZERO;
    let stick = controls.stick_value;
    if stick.length() >= STICK_DEADZONE {
        wish += cam_right * stick.x + cam_forward * stick.y;
    } else {
        if keyboard.pressed(KeyCode::KeyW) {
            wish += cam_forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            wish -= cam_forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            wish -= cam_right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            wish += cam_right;
        }
    }

    let speed_scale = if stick.length() >= STICK_DEADZONE {
        stick.length().clamp(0.0, 1.0)
    } else if wish.length_squared() > 0.0 {
        1.0
    } else {
        0.0
    };
    // Out of bridge, you were stopped where it ran out under you
    // (`bridge::lay`): you stand there, whatever the stick says, until it
    // has been let go, and walk again once it is pushed again (Hajun,
    // 2026-11-10: "the user need to let go of the screen and do the
    // joystick again to move").
    let speed_scale = if !ran_out.halted {
        speed_scale
    } else {
        if speed_scale == 0.0 {
            ran_out.halted = false;
        }
        0.0
    };
    // Your own legs follow the stick rather than where you end up, so they
    // answer the moment it moves. Everyone else's are read off how their body
    // moves (`lobby::walk_from_motion`).
    walk.speed = if speed_scale > 0.0 {
        if slowed {
            speed_scale * WATER_MOVE_SCALE
        } else {
            speed_scale * sprint
        }
    } else {
        0.0
    };
    intent.walk = wish.normalize_or_zero() * speed_scale;
    intent.sprint = sprint;
    intent.jump = keyboard.just_pressed(KeyCode::Space) || controls.jump_just_pressed;
    // A walk's own speed, facing the way it goes: the car writes its own
    // over these every frame you drive (`drive::steer`, after this), and
    // nothing else does. Until 2026-11-07 they were left as the car last
    // wrote them, so out of it you walked at its last speed facing its last
    // way, or, out of a car standing still, not at all (Hajun: "if you
    // leave the car u glitch").
    intent.pace = None;
    intent.face = None;
}

/// Every body that moves under its own power, and what moving it needs.
type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Transform,
        &'static Collider,
        &'static Venue,
        &'static Intent,
        &'static mut PlayerJump,
        &'static mut Riding,
        &'static mut Shove,
        Has<StaysAshore>,
        Has<Dead>,
    ),
>;

/// Walks, jumps and drops every body that moves under its own power — yours
/// and the AIs' alike — as its [`Intent`] asks, and keeps each one out of its
/// island, the trees on it and every other player there. Bodies on different
/// islands pass through each other: they are never in the same place.
///
/// A body riding the car is first carried along with it, as it moved since
/// the frame before, and then moves on it as it would on the ground: it rides
/// it for as long as it stands on it, or is in the air off it, and gets off
/// once it comes down on anything else (`car`).
pub(crate) fn move_bodies(
    time: Res<Time>,
    islands: Res<Islands>,
    fixed: Query<(&Transform, &Collider, &Venue), Without<Intent>>,
    cars: Decks,
    mut bodies: Bodies,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let melee = ui::ui().melee;
    let fixed: Vec<(Vec3, Collider, Venue)> = fixed
        .iter()
        .map(|(solid, collider, venue)| (solid.translation, *collider, *venue))
        .collect();

    // The dead stand where they died, neither moving nor in anyone's way
    // (`arena`).
    let movers: Vec<(Entity, Venue)> = bodies
        .iter()
        .filter(|(.., dead)| !dead)
        .map(|(entity, _, _, venue, ..)| (entity, *venue))
        .collect();
    for (mover, venue) in movers {
        // Until its island has loaded there is nothing to stand on, so the
        // body waits where it is instead of dropping into the sea.
        let Some(island) = islands.get(venue) else {
            continue;
        };
        let decks = decks_on(&cars, venue);
        // Everyone else here as they stand now, which for those already moved
        // this frame is where they have just got to, so no two end up inside
        // each other.
        let mut solids: Vec<(Vec3, Collider)> = fixed
            .iter()
            .filter(|(.., at)| *at == venue)
            .map(|&(center, collider, _)| (center, collider))
            .collect();
        solids.extend(
            bodies
                .iter()
                .filter(|(other, _, _, at, .., dead)| *other != mover && **at == venue && !dead)
                .map(|(_, other, collider, ..)| (other.translation, *collider)),
        );
        let Ok((_, mut body, _, _, intent, mut jump, mut riding, mut shove, stays_ashore, _)) = bodies.get_mut(mover)
        else {
            continue;
        };
        // Worked out on a copy and only written back if it has changed, so
        // that someone standing still is not taken for someone who moved,
        // which would have their whole skeleton placed again every frame.
        let mut transform = *body;
        let mut on = riding.0;
        if let Some(car) = on {
            match decks.iter().find(|deck| deck.car == car) {
                Some(deck) => deck.carry(&mut transform),
                None => on = None,
            }
        }
        let motion = Motion {
            ground: Ground {
                island,
                decks: &decks,
                bottomless: island::bottomless(venue),
            },
            solids: &solids,
            stays_ashore,
            dt,
            elapsed,
            melee,
        };
        match motion.step(&mut transform, &mut jump, &mut shove, intent) {
            Some(Underfoot::Deck(deck)) => on = Some(decks[deck].car),
            Some(Underfoot::Island) => on = None,
            None => {}
        }
        riding.set_if_neq(Riding(on));
        body.set_if_neq(transform);
    }
}

/// What one body's own motion over a frame depends on, besides the body.
struct Motion<'a> {
    /// The island, and the car on it.
    ground: Ground<'a>,
    /// Everyone and everything else on the island to bump into.
    solids: &'a [(Vec3, Collider)],
    stays_ashore: bool,
    dt: f32,
    /// The game's time, which the sea bobs by.
    elapsed: f32,
    /// How a knock dies away (`air_drag`, `ground_drag`).
    melee: ui::Melee,
}

impl Motion<'_> {
    /// Walks, jumps and drops `transform`, as `intent` asks, and carries it
    /// along by whatever `shove` a punch left it with, for one frame: and
    /// what it ends the frame standing on, if anything, or `None` in the
    /// air.
    fn step(
        &self,
        transform: &mut Transform,
        jump: &mut PlayerJump,
        shove: &mut Shove,
        intent: &Intent,
    ) -> Option<Underfoot> {
        let Motion {
            ground,
            solids,
            stays_ashore,
            dt,
            elapsed,
            melee,
        } = *self;
        let (support_y, _) = support(&ground, transform.translation, elapsed);
        let grounded = transform.translation.y <= support_y + WATER_JUMP_SLOP;
        let slowed = wading(transform.translation);

        let speed_scale = intent.walk.length().min(1.0);
        if speed_scale > 0.0 {
            let speed = match intent.pace {
                Some(pace) => pace,
                None if slowed => MOVE_SPEED * WATER_MOVE_SCALE,
                None => MOVE_SPEED * intent.sprint,
            };
            // A body driven keeps to where it could drive back out of, as an
            // AI keeps to where it could walk back: neither can jump.
            let ashore = stays_ashore || intent.pace.is_some();
            // In steps no longer than the lattice the island is tested on, so
            // that one slow frame cannot carry the player clean through a fence.
            let distance = speed * speed_scale * dt;
            let steps = (distance / island::LATTICE).ceil().max(1.0);
            for _ in 0..steps as u32 {
                let from = transform.translation;
                apply_planar_move(transform, intent.walk, distance / steps, dt / steps, intent.face);
                keep_off_deck_walls(from, &mut transform.translation, &ground);
                resolve_player_solids(&mut transform.translation, solids, &ground);
                if ashore && strands(ground.island, from, transform.translation) {
                    transform.translation = from;
                    break;
                }
            }
        }

        // Knocked by a punch, the body is carried along through the same
        // walls and bodies as a walk, without turning to face the way it
        // goes, and off any edge: nothing keeps an AI on its island from a
        // punch, which is what knocks one into the void. What carries it
        // dies away as Minecraft's does, by the tick, faster on the ground.
        if shove.across != Vec2::ZERO {
            let along = Vec3::new(shove.across.x, 0.0, shove.across.y) * dt;
            let steps = (along.length() / island::LATTICE).ceil().max(1.0);
            for _ in 0..steps as u32 {
                let from = transform.translation;
                transform.translation += along / steps;
                keep_off_deck_walls(from, &mut transform.translation, &ground);
                resolve_player_solids(&mut transform.translation, solids, &ground);
            }
            let drag = if grounded { melee.ground_drag } else { melee.air_drag };
            shove.across *= drag.powf(dt * 20.0);
            if shove.across.length_squared() < 0.01 {
                shove.across = Vec2::ZERO;
            }
        }
        shove.hurt = (shove.hurt - dt).max(0.0);

        if intent.jump {
            jump.buffer = JUMP_BUFFER;
        }
        if grounded {
            jump.coyote = JUMP_COYOTE;
        } else {
            jump.coyote = (jump.coyote - dt).max(0.0);
        }
        jump.buffer = (jump.buffer - dt).max(0.0);
        if jump.buffer > 0.0 && jump.coyote > 0.0 {
            jump.velocity_y = JUMP_SPEED;
            jump.buffer = 0.0;
            jump.coyote = 0.0;
        }

        let gravity = if jump.velocity_y > 0.0 {
            GRAVITY
        } else {
            FALL_GRAVITY
        };
        jump.velocity_y -= gravity * dt;
        let before = transform.translation.y;
        transform.translation.y += jump.velocity_y * dt;
        // Going up, the head stops at whatever is over it — a ceiling, the top
        // of a doorway — rather than going into it, where it would read as a
        // wall at the height of the head and shove the body out sideways.
        if jump.velocity_y > 0.0
            && let Some(ceiling) = headroom(&ground, transform.translation.xz(), before + PLAYER_HEIGHT)
            && transform.translation.y + PLAYER_HEIGHT > ceiling - HEAD_CLEARANCE
        {
            transform.translation.y = (ceiling - HEAD_CLEARANCE - PLAYER_HEIGHT).max(before);
            jump.velocity_y = 0.0;
        }

        // Looked for from the higher of this frame's two heights, so that one
        // long frame cannot drop the feet clean through the ground they started
        // on.
        let highest = transform.translation.with_y(before.max(transform.translation.y));
        let (support_y, under) = support(&ground, highest, elapsed);
        // Walking down the bridge, keep the feet on it rather than stepping off
        // into the air every frame and dropping back onto it a moment later.
        let downhill = grounded && transform.translation.y - support_y <= LAND_STEP_UP;
        let mut footing = None;
        if jump.velocity_y <= 0.0 && (transform.translation.y <= support_y || downhill) {
            transform.translation.y = support_y;
            jump.velocity_y = 0.0;
            footing = Some(under);
        }
        // Only once the feet have settled, so that the ground they are about to
        // land on does not first read as a wall at the height of their shins.
        resolve_player_solids(&mut transform.translation, solids, &ground);
        footing
    }
}

/// Starts a punch each time the fist button is pressed, or F on desktop,
/// unless a dialog is up: the next arm, from the start, whatever the last
/// was doing. In the car the fist is the reverse (`drive`), and there is no
/// punch.
///
/// And lands it (since 2026-11-07, Hajun: "deals 'knockback' ... if done
/// while both facing the other player/ai and pressed when within range"):
/// on the nearest body with any of itself in the range the file's `melee`
/// draws round you (`range::Sector`: from `near` to `reach` in front of
/// you, `width` across where it begins, opening out by `cone`), and no
/// higher or lower than a body's height (Minecraft 1.8.9's three blocks
/// from the eyes to the hitbox, with the crosshair on it), at the moment
/// of the press, as Minecraft lands a click. An AI, or anyone this device
/// moves, is knocked back on the spot ([`Shove::knock`]); someone online
/// is told through the server ([`Hit`], `net::relay_hits`), and their own
/// device knocks them. Hit on the move, they go further, as from a
/// sprinting hit.
#[allow(clippy::too_many_arguments)]
fn start_punch(
    mut pressed: MessageReader<PunchPressed>,
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Res<hud::DialogUp>,
    driving: Res<drive::Driving>,
    bridging: Res<bridge::Bridging>,
    me: Query<(Entity, &Transform, &Venue, &Intent, Has<Dead>, Option<&Team>, Has<Drinking>), With<Player>>,
    others: Query<
        (Entity, &Transform, &Venue, Option<&Team>),
        (With<WalkCycle>, Without<Player>, Without<Dead>, Without<ForShow>),
    >,
    mut shoves: Query<(&mut Shove, &mut PlayerJump), Without<Player>>,
    mut hits: MessageWriter<Hit>,
    mut landed: MessageWriter<Landed>,
    mut swung: MessageWriter<Swung>,
    mut commands: Commands,
) {
    // Carrying a plank, your hands are full: no punch (`bridge`; Hajun,
    // 2026-11-09: "dont allow punching").
    let asked = pressed.read().count() > 0 || keyboard.just_pressed(KeyCode::KeyF);
    if !asked || dialog.0 || driving.on() || bridging.0 {
        return;
    }
    let Ok((you, body, &venue, intent, dead, team, drinking)) = me.single() else {
        return;
    };
    // Nor with a potion at your mouth (`utility`).
    if dead || drinking {
        return;
    }
    commands.entity(you).insert(Punching::next());
    let melee = ui::ui().melee;
    let at = body.translation.xz();
    let facing = body.forward().xz().normalize_or(Vec2::NEG_Y);
    debug!("punch thrown from {at}, facing {facing}");
    swung.write(Swung {
        at,
        feet: body.translation.y,
        facing,
    });
    // Never on a teammate (`Team`, in Bridge Wars): the punch passes them
    // by, and lands on whoever is behind them.
    let Some((victim, there)) = punch_lands(
        &melee,
        at,
        body.translation.y,
        facing,
        others
            .iter()
            .filter(|(_, _, on, theirs)| **on == venue && !(team.is_some() && *theirs == team))
            .map(|(other, transform, ..)| (other, transform.translation)),
    ) else {
        return;
    };
    let running = intent.walk.length() >= 0.5;
    match shoves.get_mut(victim) {
        Ok((mut shove, mut jump)) => {
            let took = shove.knock(&mut jump, at, there, running);
            debug!("punch landed on {victim}, {:.2} m off{}", at.distance(there), if took { "" } else { ", still reeling" });
            if took {
                landed.write(Landed { by: you, victim });
            }
        }
        Err(_) => {
            debug!("punch landed on {victim} online, {:.2} m off", at.distance(there));
            hits.write(Hit { victim, running });
        }
    }
}

/// Whom a punch thrown from `at`, `feet` up, facing `facing`, lands on,
/// of the `bodies` there, each by where its feet are: the nearest with any
/// of itself in the range the file's `melee` draws round the puncher
/// (`range::Sector::holds`, a body being `PLAYER_RADIUS` across), no
/// further up or down than a body's height; and where it stands.
fn punch_lands(
    melee: &ui::Melee,
    at: Vec2,
    feet: f32,
    facing: Vec2,
    bodies: impl Iterator<Item = (Entity, Vec3)>,
) -> Option<(Entity, Vec2)> {
    let sector = range::Sector::of(melee);
    bodies
        .filter_map(|(body, there)| {
            let off = there.xz() - at;
            if (there.y - feet).abs() > PLAYER_HEIGHT || !sector.holds(facing, off, PLAYER_RADIUS) {
                return None;
            }
            Some((off.length(), body, there.xz()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, body, there)| (body, there))
}

pub(crate) fn animate_walk(
    time: Res<Time>,
    mut walks: Query<(
        Entity,
        &mut WalkCycle,
        Option<&mut Punching>,
        Has<Carrying>,
        Option<&Drinking>,
        Option<&arena::Wears>,
    )>,
    mut bones: Query<(&WalkBone, &mut Transform)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let melee = ui::ui().melee;
    let swing = melee.swing();
    for (body, mut walk, punch, ..) in &mut walks {
        let target = if walk.speed > 0.04 { 1.0 } else { 0.0 };
        walk.weight = walk.weight.lerp(target, (dt * WALK_BLEND).min(1.0));
        if walk.weight < 0.01 {
            walk.weight = 0.0;
        } else {
            walk.phase += dt * WALK_STRIDE_FREQ * walk.speed.max(0.35);
        }
        if let Some(mut punch) = punch {
            punch.age += dt;
            if punch.age >= swing {
                commands.entity(body).remove::<Punching>();
            }
        }
    }
    for (bone, mut transform) in &mut bones {
        let Ok((_, walk, punch, carrying, drinking, wears)) = walks.get(bone.owner) else {
            continue;
        };
        // The right arm punches; carrying a plank, both arms hold it out
        // in front; every other limb walks. Running, faster than a walk
        // (`Intent::sprint`), the limbs swing that much further as well as
        // faster, up to half as far again.
        let stride = 1.0 + (walk.speed - 1.0).clamp(0.0, 0.7);
        // With a sword in it (`arena::Wears`), the right arm holds it at
        // the ready and slashes with it in place of the punch; drinking,
        // it is at the mouth, whatever else it holds (`Drinking`).
        let armed = wears.is_some_and(|wears| wears.sword > 0) && !carrying;
        let striking = punch.filter(|punch| punch.age < swing && matches!(bone.kind, WalkBoneKind::ArmR));
        let offset = drinking
            .and_then(|drinking| drink_offset(bone.kind, bone.frame, bone.stance, drinking.lifted()))
            .or_else(|| {
                if armed {
                    sword_offset(bone.kind, bone.stance, striking.map(|punch| slash_at(&melee, punch.age)))
                } else {
                    striking.and_then(|punch| punch_offset(bone.kind, bone.stance, Punching::reach(&melee, punch.age)))
                }
            })
            .or_else(|| carrying.then(|| carry_offset(bone.kind, bone.stance)).flatten())
            .unwrap_or_else(|| walk_offset(bone.kind, walk.phase.sin() * stride, walk.weight, bone.stance));
        let rotation = bone.turned(offset);
        // Standing still, a limb keeps the pose it had, and is left alone.
        if transform.rotation != rotation {
            transform.rotation = rotation;
        }
    }
}


/// Puts the camera where you turned and zoomed it, whatever is in the way:
/// nothing ever brings it in nearer you. Standing in a house in House
/// Builder, whatever stands between it and you is drawn see-through instead
/// (`see_through`); anywhere else it hides you. Behind you it circles you,
/// looking down at you, as far as straight down, or looking up past you from
/// low behind you. Zoomed all the way in it is your eyes, turned the same
/// way, and you are not drawn.
///
/// Riding the car, it goes along with you, and turns only as you turn it:
/// the car turns under you, as the world does.
///
/// While you talk to someone it comes in to their face from your eyes
/// ([`close_up_view`]), and after, goes back out to where it was.
pub(crate) fn follow_camera(
    time: Res<Time>,
    orbit: Res<OrbitCamera>,
    mut close_up: ResMut<CloseUp>,
    islands: Res<Islands>,
    cars: Decks,
    driving: Res<drive::Driving>,
    mut players: Query<(&Transform, &Venue, &mut Visibility, Has<Dead>), With<Player>>,
    mut cameras: Query<&mut Transform, (With<ThirdPersonCamera>, Without<Player>)>,
) {
    let Ok((player, &venue, mut body, dead)) = players.single_mut() else {
        return;
    };
    let Ok(mut eye) = cameras.single_mut() else {
        return;
    };
    let decks = decks_on(&cars, venue);
    let ground = islands.get(venue).map(|island| Ground {
        island,
        decks: &decks,
        bottomless: island::bottomless(venue),
    });
    let mut view = camera_view(&orbit, player, ground);
    let blend = close_up.step(time.delta_secs());
    if blend > 0.0 {
        let near = close_up_view(player, close_up.face, ground);
        let eased = blend * blend * (3.0 - 2.0 * blend);
        view = Transform::from_translation(view.translation.lerp(near.translation, eased))
            .with_rotation(view.rotation.slerp(near.rotation, eased));
    }
    // Both only written when they change, so that a camera at rest is not
    // taken for one that moved. In the car, you are the car (`drive`);
    // dead, you are not there (`arena`).
    body.set_if_neq(if driving.on() || dead || inside_you(view.translation, player) {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    eye.set_if_neq(view);
}

/// Whether the camera at `at` is inside you, or so near your arms or head that
/// you would fill the screen from within: anywhere over where you stand up to
/// the top of your head.
fn inside_you(at: Vec3, player: &Transform) -> bool {
    let feet = player.translation;
    at.xz().distance(feet.xz()) < PLAYER_RADIUS + CAMERA_INSIDE
        && at.y > feet.y - CAMERA_INSIDE
        && at.y < feet.y + PLAYER_HEIGHT.max(EYE_HEIGHT) + CAMERA_INSIDE
}

/// Where the camera stands and which way it looks, turned and zoomed as
/// `orbit` is round `player` on `ground`: as far out behind you as you zoomed
/// it, or in your eyes.
fn camera_view(orbit: &OrbitCamera, player: &Transform, ground: Option<Ground>) -> Transform {
    if orbit.distance <= 0.0 {
        return eye_view(orbit, player, ground);
    }
    let look_up = if orbit.pitch < 0.0 {
        (orbit.pitch / PITCH_MIN).clamp(0.0, 1.0)
    } else {
        0.0
    };
    behind_you(orbit, player, look_up, orbit.distance)
}

/// Your own eyes, inside your head, looking the way `orbit` is turned: the
/// same yaw and pitch swing the camera round you to look the same way when it
/// is behind you, so a swipe turns the view alike in both. Under a ceiling
/// lower than them they stop short of it, the way a jump stops your head: a
/// room's, or the car's roof.
fn eye_view(orbit: &OrbitCamera, player: &Transform, ground: Option<Ground>) -> Transform {
    let chest = player.translation + Vec3::Y * LOOK_HEIGHT;
    Transform::from_translation(rise(chest, EYE_HEIGHT - LOOK_HEIGHT, ground))
        .with_rotation(Quat::from_rotation_y(orbit.yaw) * Quat::from_rotation_x(-orbit.pitch))
}

/// Looking at `face` from as near as [`CLOSE_UP`] to it, while you talk to
/// whoever it is: from your eyes, or from further off than that, as far
/// toward them as that, at the height of your eyes. A little under their face,
/// which puts it over the middle of the screen, clear of what they say.
fn close_up_view(player: &Transform, face: Vec3, ground: Option<Ground>) -> Transform {
    let eyes = rise(player.translation + Vec3::Y * LOOK_HEIGHT, EYE_HEIGHT - LOOK_HEIGHT, ground);
    let across = (face - eyes).with_y(0.0);
    let far = across.length();
    // Never, as bodies keep apart; but looking straight down from right over
    // them would say no way round to turn.
    if far < 0.1 {
        return Transform::from_translation(eyes).with_rotation(player.rotation);
    }
    let at = eyes + across * ((far - CLOSE_UP).max(0.0) / far);
    Transform::from_translation(at).looking_at(face - Vec3::Y * UNDER_THE_FACE, Vec3::Y)
}

/// As far as `up` straight up from `at` as the camera gets before it meets a
/// ceiling, and no further.
fn rise(at: Vec3, up: f32, ground: Option<Ground>) -> Vec3 {
    let top = at + Vec3::Y * up;
    let clear = ground
        .and_then(|ground| ground.sweep(at, top, CAMERA_RADIUS))
        .unwrap_or(up);
    at + Vec3::Y * clear
}

/// Where the camera stands `distance` out from you, turned as `orbit` is, and
/// which way it looks: down at the point on you it circles, straight down
/// from right over it at the most, or, looking up, from low behind you and
/// nearer, up past you.
fn behind_you(orbit: &OrbitCamera, player: &Transform, look_up: f32, distance: f32) -> Transform {
    let mut camera = Transform::default();
    let yaw = Quat::from_rotation_y(orbit.yaw);
    if orbit.pitch >= 0.0 {
        // Turned as your eyes would be, which is looking at that point from
        // out here, and still says which way is ahead looking straight down.
        let turn = yaw * Quat::from_rotation_x(-orbit.pitch);
        let look_at = player.translation + Vec3::Y * LOOK_HEIGHT;
        camera.translation = look_at + turn * Vec3::new(0.0, 0.0, distance);
        camera.rotation = turn;
        return camera;
    }

    let back = yaw * Vec3::Z;
    let cam_height = LOOK_HEIGHT + (CAMERA_LOOK_UP_HEIGHT - LOOK_HEIGHT) * look_up;
    let look_up_distance = (distance * (CAMERA_LOOK_UP_DISTANCE / CAMERA_DISTANCE))
        .max(CAMERA_LOOK_UP_NEAREST)
        .min(distance);
    let distance = distance + (look_up_distance - distance) * look_up;
    let focus_height = LOOK_HEIGHT + (LOOK_UP_FOCUS_HEIGHT - LOOK_HEIGHT) * look_up;
    camera.translation = player.translation + back * distance + Vec3::Y * cam_height;
    camera.translation.y = camera.translation.y.max(player.translation.y + CAMERA_LOOK_UP_HEIGHT);

    let focus = player.translation + Vec3::Y * focus_height;
    camera.look_at(focus, Vec3::Y);
    camera.rotate_local_x(LOOK_UP_EXTRA_PITCH * look_up);
    camera
}

#[cfg(test)]
mod camera_tests {
    use super::*;

    fn orbit(yaw: f32, pitch: f32, distance: f32) -> OrbitCamera {
        OrbitCamera {
            yaw,
            pitch,
            distance,
            ..default()
        }
    }

    /// The triangles of closed boxes, from their lowest to their highest
    /// corners, wound out.
    pub(crate) fn boxes(list: &[(Vec3, Vec3)]) -> Vec<[Vec3; 3]> {
        let mut corners = Vec::new();
        for &(l, h) in list {
            let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
            let quads = [
                [p(l.x, l.y, l.z), p(h.x, l.y, l.z), p(h.x, l.y, h.z), p(l.x, l.y, h.z)],
                [p(l.x, h.y, l.z), p(l.x, h.y, h.z), p(h.x, h.y, h.z), p(h.x, h.y, l.z)],
                [p(l.x, l.y, l.z), p(l.x, l.y, h.z), p(l.x, h.y, h.z), p(l.x, h.y, l.z)],
                [p(h.x, l.y, l.z), p(h.x, h.y, l.z), p(h.x, h.y, h.z), p(h.x, l.y, h.z)],
                [p(l.x, l.y, l.z), p(l.x, h.y, l.z), p(h.x, h.y, l.z), p(h.x, l.y, l.z)],
                [p(l.x, l.y, h.z), p(h.x, l.y, h.z), p(h.x, h.y, h.z), p(l.x, h.y, h.z)],
            ];
            for q in quads {
                corners.push([q[0], q[1], q[2]]);
                corners.push([q[0], q[2], q[3]]);
            }
        }
        corners
    }

    /// Closed boxes, from their lowest to their highest corners, wound out.
    pub(super) fn island(list: &[(Vec3, Vec3)]) -> Island {
        Island::new_for_tests(&boxes(list))
    }

    const GROUND: (Vec3, Vec3) = (Vec3::new(-40.0, -2.0, -40.0), Vec3::new(40.0, 0.0, 40.0));

    /// The room in `testhomemap.glb`: 3 m high inside, its doorway facing +z.
    fn room() -> Vec<(Vec3, Vec3)> {
        let v = Vec3::new;
        vec![
            GROUND,
            (v(-4.0, -0.1, -8.5), v(4.0, 3.3, -8.2)),
            (v(-4.0, -0.1, -8.5), v(-3.7, 3.3, 0.0)),
            (v(3.7, -0.1, -8.5), v(4.0, 3.3, 0.0)),
            (v(-4.0, -0.1, -0.3), v(-0.8, 3.3, 0.0)),
            (v(0.8, -0.1, -0.3), v(4.0, 3.3, 0.0)),
            (v(-0.8, 2.3, -0.3), v(0.8, 3.3, 0.0)),
            (v(-3.7, 3.0, -8.2), v(3.7, 3.3, -0.3)),
        ]
    }

    #[test]
    fn walls_never_bring_it_in() {
        // In the middle of the room, tucked into a corner of it, and outside
        // with the room between the camera and you: turned all the way round
        // and tipped up and down, it stands as far out as you zoomed it,
        // room or no room, and you are in sight of it.
        let island = island(&room());
        for (x, z) in [(0.0, -4.0), (-3.2, -7.7), (0.0, 3.0)] {
            let player = Transform::from_xyz(x, 0.0, z);
            for step in 0..96 {
                let turn = step as f32 / 96.0;
                let tilt = (1.2 * (turn * 9.0).sin()).clamp(PITCH_MIN, PITCH_MAX);
                let you = orbit(turn * std::f32::consts::TAU * 2.0, tilt, 15.0);
                let view = camera_view(&you, &player, Some(Ground::bare(&island)));
                assert_eq!(view, camera_view(&you, &player, None), "at {x} {z}");
                if you.pitch >= 0.0 {
                    let middle = player.translation + Vec3::Y * LOOK_HEIGHT;
                    let out = view.translation.distance(middle);
                    assert!((out - 15.0).abs() < 1e-3, "at {x} {z}: {out} out");
                }
                assert!(!inside_you(view.translation, &player), "at {x} {z}");
            }
        }
    }

    #[test]
    fn behind_you_you_are_always_in_sight() {
        // As near as it stands, looking up past you and down at you, as far
        // as straight down.
        let player = Transform::default();
        for pitch in [PITCH_MIN, -0.6, 0.0, 0.38, 1.2, PITCH_MAX] {
            let you = orbit(0.3, pitch, CAMERA_DISTANCE_MIN);
            let view = camera_view(&you, &player, None);
            assert!(!inside_you(view.translation, &player), "pitch {pitch}");
        }
    }

    #[test]
    fn zoomed_all_the_way_in_it_is_your_eyes() {
        let player = Transform::from_xyz(1.0, 0.5, -2.0);
        for pitch in [PITCH_MIN, 0.0, 0.38, PITCH_MAX] {
            let you = orbit(0.7, pitch, 0.0);
            let view = camera_view(&you, &player, None);
            assert_eq!(view, eye_view(&you, &player, None));
            // Inside your head, at the height of your eyes, and you not drawn.
            let eyes = view.translation - player.translation;
            assert_eq!(eyes.xz(), Vec2::ZERO);
            assert!((eyes.y - EYE_HEIGHT).abs() < 1e-5, "{eyes}");
            // In the head: playercharacter.glb's is from 1.11 m to 1.64.
            assert!(eyes.y > 1.11 && eyes.y < 1.64, "{eyes}");
            assert!(inside_you(view.translation, &player));
        }
    }

    #[test]
    fn it_looks_straight_down_from_right_overhead() {
        // Tipped down as far as it goes.
        let mut you = orbit(0.9, 0.38, 12.0);
        you.turn(Vec2::new(0.0, 1e4));
        assert_eq!(you.pitch, std::f32::consts::FRAC_PI_2);
        let player = Transform::from_xyz(3.0, 0.0, -1.0);
        let view = camera_view(&you, &player, None);
        let middle = player.translation + Vec3::Y * LOOK_HEIGHT;
        assert!(view.translation.xz().distance(middle.xz()) < 1e-4, "{}", view.translation);
        assert!((view.translation.y - (middle.y + 12.0)).abs() < 1e-4);
        assert!(view.forward().dot(Vec3::NEG_Y) > 1.0 - 1e-6);
        // Ahead is still the way it was turned: the top of the screen, which
        // is where the stick's up walks you.
        let turned = Quat::from_rotation_y(0.9) * Vec3::NEG_Z;
        assert!(camera_ahead(&view).distance(turned) < 1e-4);
        let nearly = camera_view(&orbit(0.9, PITCH_MAX - 0.05, 12.0), &player, None);
        assert!(camera_ahead(&nearly).distance(turned) < 1e-4);
        // And so it is in your eyes, looking at your feet.
        let feet = camera_view(&OrbitCamera { distance: 0.0, ..you }, &player, None);
        assert!(camera_ahead(&feet).distance(turned) < 1e-4);
    }

    #[test]
    fn in_bridge_wars_it_is_fixed_over_you_and_nothing_zooms_it() {
        // Turned and zoomed some way, then fixed over you for a game: the
        // top of the screen north, straight down, as high as the file says.
        let mut you = orbit(0.9, 0.38, 12.0);
        let was = you.top_down().expect("it was behind you");
        let height = ui::ui().arena.camera_height;
        assert!(you.overhead && you.locked);
        assert_eq!((you.yaw, you.pitch, you.distance), (0.0, PITCH_MAX, height));
        // Neither a pinch nor the wheel moves it, in or out, and nothing
        // turns it.
        for ratio in [0.5, 0.88, 1.12, 2.0] {
            you.zoom_by_ratio(ratio);
            assert_eq!(you.distance, height);
        }
        you.turn(Vec2::new(300.0, -200.0));
        assert_eq!((you.yaw, you.pitch), (0.0, PITCH_MAX));
        // Out of the game it is as it was, and zooms again.
        you.back_to(was);
        assert!(!you.overhead && !you.locked);
        assert_eq!((you.yaw, you.pitch, you.distance), (0.9, 0.38, 12.0));
        you.zoom_by_ratio(1.1);
        assert!(you.distance > 12.0);
        // Over you already, in the car, and then into a game: there is
        // nothing to go back to, and it is locked at the game's height all
        // the same.
        let mut you = orbit(0.3, 0.38, 9.0);
        you.overhead();
        you.zoom_by_ratio(1.5);
        assert!(you.top_down().is_none());
        assert!(you.locked);
        assert_eq!(you.distance, height);
    }

    #[test]
    fn from_15_5_m_up_an_island_just_fits_the_height_of_the_screen() {
        // What the file's `arena.camera_height` was chosen by (the file's
        // own number is Hajun's to change, and is not held to this): with
        // you in the middle of your island, the island's edge is just
        // inside the top and the bottom of the screen, the island taking
        // up all of its height but a twenty-fifth, on any screen wider
        // than it is tall, the camera seeing so many degrees top to bottom.
        let you = OrbitCamera {
            distance: 15.5,
            ..orbit(0.0, PITCH_MAX, 15.5)
        };
        let player = Transform::from_xyz(0.0, 0.25, 130.0);
        let view = camera_view(&you, &player, None);
        assert!(view.forward().dot(Vec3::NEG_Y) > 1.0 - 1e-6);
        let over = view.translation.y - player.translation.y;
        let seen = 2.0 * over * (CAMERA_FOV * 0.5).tan();
        let share = 2.0 * roundtown_net::ARENA_ISLAND_RADIUS / seen;
        assert!(share > 0.95 && share < 0.97, "the island is {share} of the screen's height");
    }

    #[test]
    fn your_eyes_stop_under_a_low_ceiling() {
        let player = Transform::default();
        let you = orbit(0.0, 0.38, 0.0);
        let eyes =
            |boxes: &[(Vec3, Vec3)]| eye_view(&you, &player, Some(Ground::bare(&island(boxes)))).translation;
        assert!((eyes(&[GROUND]).y - EYE_HEIGHT).abs() < 1e-6);
        // Under a ceiling lower than your eyes, as high as they go under it.
        let top = EYE_HEIGHT - 0.05 + CAMERA_RADIUS;
        let low = (Vec3::new(-20.0, top, -20.0), Vec3::new(20.0, top + 0.3, 20.0));
        let under = eyes(&[GROUND, low]);
        assert!((under.y - (EYE_HEIGHT - 0.05)).abs() < 1e-3, "{under}");
    }

    #[test]
    fn zoomed_in_past_as_near_as_it_stands_it_is_in_your_eyes() {
        let mut you = OrbitCamera::default();
        // In, as near as it stands behind you and never nearer, and then a
        // little further in, into your eyes...
        while you.distance > 0.0 {
            you.zoom_by_ratio(0.9);
            assert!(you.distance == 0.0 || you.distance >= CAMERA_DISTANCE_MIN, "{}", you.distance);
        }
        // ...which is as far in as it goes.
        for _ in 0..20 {
            you.zoom_by_ratio(0.8);
        }
        assert_eq!(you.distance, 0.0);
        // A pinch out that wavers back stays in your eyes; out a tenth
        // further than that, it is straight back behind you, as near as it
        // stands.
        you.zoom_by_ratio(1.05);
        you.zoom_by_ratio(0.95);
        you.zoom_by_ratio(1.05);
        assert_eq!(you.distance, 0.0);
        you.zoom_by_ratio(1.05);
        assert_eq!(you.distance, CAMERA_DISTANCE_MIN);
        // A notch of the wheel each way, from as near as it stands.
        you.zoom_by_ratio(0.88);
        assert_eq!(you.distance, 0.0);
        you.zoom_by_ratio(1.12);
        assert_eq!(you.distance, CAMERA_DISTANCE_MIN);
        // Out as far as it goes, and no further.
        for _ in 0..30 {
            you.zoom_by_ratio(1.25);
        }
        assert_eq!(you.distance, CAMERA_DISTANCE_MAX);
    }

    #[test]
    fn a_swipe_turns_it_alike_however_far_out() {
        let swipe = Vec2::new(40.0, -25.0);
        let turned = |distance: f32| {
            let mut you = orbit(0.0, 0.38, distance);
            you.turn(swipe);
            Vec2::new(you.yaw, you.pitch)
        };
        assert_eq!(turned(0.0), turned(CAMERA_DISTANCE));
        assert_eq!(turned(CAMERA_DISTANCE_MIN), turned(CAMERA_DISTANCE_MAX));
    }

    #[test]
    fn walking_with_the_stick_brings_it_round_behind_where_you_walk() {
        // Behind you looking down -z, walking off to the right, +x, at full
        // speed: it comes round, never faster than it may, and stops behind
        // where you walk, the way it stands behind where you face.
        let right = Vec2::new(MOVE_SPEED, 0.0);
        let behind = OrbitCamera::behind(&Transform::default().looking_to(Vec3::X, Vec3::Y)).yaw;
        let dt = 1.0 / 120.0;
        let mut yaw = 0.0;
        for _ in 0..1200 {
            let turn = recenter_turn(yaw, right, 1.0);
            assert!(turn.abs() <= RECENTER_MOST + 1e-6, "{turn}");
            assert!(wrap_angle(behind - yaw).abs() >= wrap_angle(behind - yaw - turn * dt).abs());
            yaw += turn * dt;
        }
        assert!(wrap_angle(behind - yaw).abs() < 0.01, "{yaw} for {behind}");
        // Walking straight away from it, it stays; walking at it, or nearly,
        // it never comes round.
        assert_eq!(recenter_turn(0.0, Vec2::new(0.0, -MOVE_SPEED), 1.0), 0.0);
        assert_eq!(recenter_turn(0.0, Vec2::new(0.0, MOVE_SPEED), 1.0), 0.0);
        assert_eq!(recenter_turn(0.0, Vec2::new(1.0, MOVE_SPEED), 1.0), 0.0);
        // Slower walking at a small angle, more slowly; and not before it
        // is ready.
        let off = Vec2::new(1.0, -3.0);
        assert!(recenter_turn(0.0, off * 0.5, 1.0).abs() < recenter_turn(0.0, off * 2.0, 1.0).abs());
        assert_eq!(recenter_turn(0.0, right, 0.0), 0.0);
    }

    #[test]
    fn it_comes_round_only_once_left_alone_and_eases_in() {
        let ready = |untouched: f32, pushed: f32| {
            CameraAssist {
                untouched,
                pushed,
                ..default()
            }
            .ready()
        };
        // Touched too lately, or the stick only just pushed: not yet.
        assert_eq!(ready(RECENTER_AFTER - 0.1, 5.0), 0.0);
        assert_eq!(ready(5.0, 0.0), 0.0);
        // Then by degrees, from whichever came later.
        let easing = ready(RECENTER_AFTER + RECENTER_EASE_IN * 0.5, 5.0);
        assert!(easing > 0.0 && easing < 1.0, "{easing}");
        assert!((easing - ready(5.0, RECENTER_EASE_IN * 0.5)).abs() < 1e-5);
        assert_eq!(ready(5.0, 5.0), 1.0);
        // Riding a bus counts as the stick does, standing still.
        let aboard = |untouched: f32, aboard: f32| {
            CameraAssist {
                untouched,
                aboard,
                ..default()
            }
            .ready()
        };
        assert_eq!(aboard(5.0, 0.0), 0.0);
        assert_eq!(aboard(RECENTER_AFTER - 0.1, 5.0), 0.0);
        assert!((aboard(5.0, RECENTER_EASE_IN * 0.5) - easing).abs() < 1e-5);
        assert_eq!(aboard(5.0, 5.0), 1.0);
    }

    #[test]
    fn riding_a_bus_it_comes_round_behind_the_way_the_bus_faces() {
        // Behind you looking down -z, on a bus facing +x: it comes round
        // behind the way the bus faces, never faster than walking brings it
        // round, and stops there, the way it stands behind where you face.
        let facing = Vec2::X;
        let behind = OrbitCamera::behind(&Transform::default().looking_to(Vec3::X, Vec3::Y)).yaw;
        let dt = 1.0 / 120.0;
        let mut yaw = 0.0;
        for _ in 0..1200 {
            let turn = ride_turn(yaw, facing, 1.0);
            assert!(turn.abs() <= RECENTER_MOST + 1e-6, "{turn}");
            assert!(wrap_angle(behind - yaw).abs() >= wrap_angle(behind - yaw - turn * dt).abs());
            yaw += turn * dt;
        }
        assert!(wrap_angle(behind - yaw).abs() < 0.01, "{yaw} for {behind}");
        // Already behind it, it stays; turned to look out of the front of
        // the bus, or nearly, it still comes round, as walking at the
        // camera would not; and not before it is ready.
        assert_eq!(ride_turn(0.0, Vec2::NEG_Y, 1.0), 0.0);
        assert!(ride_turn(0.0, Vec2::new(0.1, 1.0), 1.0).abs() > 0.5);
        assert_eq!(recenter_turn(0.0, Vec2::new(0.1, 1.0) * MOVE_SPEED, 1.0), 0.0);
        assert_eq!(ride_turn(0.0, facing, 0.0), 0.0);
        // Its turning follows the bus round: facing a little further round
        // each frame, the camera is never left more than a bend's worth
        // behind it, and catches up once the bend is over.
        let mut yaw = behind;
        let mut heading = 0.0_f32;
        for _ in 0..600 {
            heading += 0.4 * dt;
            let facing = Vec2::new(heading.cos(), heading.sin());
            yaw += ride_turn(yaw, facing, 1.0) * dt;
            assert!(wrap_angle(yaw_behind(facing) - yaw).abs() < 0.5, "{yaw} behind {heading}");
        }
        let facing = Vec2::new(heading.cos(), heading.sin());
        for _ in 0..600 {
            yaw += ride_turn(yaw, facing, 1.0) * dt;
        }
        assert!(wrap_angle(yaw_behind(facing) - yaw).abs() < 0.02);
    }

    #[test]
    fn a_punch_goes_out_and_back_in_about_minecraft_s_six_ticks() {
        let melee = ui::ui().melee;
        let swing = melee.swing();
        // Minecraft 1.8.9's swing is six ticks, 0.3 s; and still fast
        // enough to throw several a second.
        assert!((swing - 0.3).abs() <= 0.1, "{swing}");
        assert_eq!(Punching::reach(&melee, 0.0), 0.0);
        assert_eq!(Punching::reach(&melee, melee.swing_out), 1.0);
        assert_eq!(Punching::reach(&melee, swing), 0.0);
        let half = melee.swing_out * 0.5;
        assert!(Punching::reach(&melee, half) > 0.0 && Punching::reach(&melee, half) < 1.0);
        assert!(Punching::reach(&melee, melee.swing_out + melee.swing_back * 0.5) > 0.0);
        assert_eq!(Punching::next().age, 0.0);
        // At full reach both arms are swung forward by the same turn, hung as
        // the walk hangs them; at none they are as they hang, standing still.
        let hang = 0.3;
        let left = punch_offset(WalkBoneKind::ArmL, hang, 0.0).unwrap();
        let right = punch_offset(WalkBoneKind::ArmR, -hang, 0.0).unwrap();
        assert!(left.angle_between(walk_offset(WalkBoneKind::ArmL, 0.0, 0.0, hang)) < 1e-6);
        assert!(right.angle_between(walk_offset(WalkBoneKind::ArmR, 0.0, 0.0, -hang)) < 1e-6);
        let out = punch_offset(WalkBoneKind::ArmL, hang, 1.0).unwrap();
        assert!((left.inverse() * out).to_axis_angle().1 - PUNCH_REACH < 1e-5);
        assert_eq!(punch_offset(WalkBoneKind::ThighL, 0.0, 1.0), None);
    }

    #[test]
    fn a_limb_is_turned_about_its_body_s_axes_whatever_way_its_bone_lies() {
        // A bone lying some way, under nodes turned some other way: turned
        // forward about the body's x, its body sees it turned just so.
        let above = Quat::from_rotation_y(2.0) * Quat::from_rotation_x(0.4);
        let rest = Quat::from_rotation_z(-1.1) * Quat::from_rotation_y(0.7);
        let bone = WalkBone {
            kind: WalkBoneKind::ThighL,
            rest_rotation: rest,
            frame: above * rest,
            stance: 0.0,
            owner: Entity::PLACEHOLDER,
        };
        let turn = Quat::from_rotation_x(0.5);
        let (seen, meant) = (above * bone.turned(turn), turn * above * rest);
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            assert!((seen * axis).abs_diff_eq(meant * axis, 1e-5), "{:?} {:?}", seen * axis, meant * axis);
        }
        // Not turned at all, it rests.
        let rested = bone.turned(Quat::IDENTITY);
        for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
            assert!((rested * axis).abs_diff_eq(rest * axis, 1e-5), "{:?} {:?}", rested * axis, rest * axis);
        }
    }

    #[test]
    fn every_limb_of_the_player_s_model_is_bound_and_swings_the_right_way() {
        for model in crate::lobby::models() {
            limbs_of(model);
        }
    }

    /// The limbs of `model`, each found once, stand and swing the right way.
    fn limbs_of(model: &str) {
        // The model's bones, as they lie at rest in the body's space; a bone
        // points along its own y.
        let frames = crate::island::glb::frames(model);
        let frame = |kind: WalkBoneKind| -> Quat {
            let found: Vec<Quat> = frames
                .iter()
                .filter(|(name, _)| walk_bone_kind(name).is_some_and(|found| found as u8 == kind as u8))
                .map(|(_, frame)| *frame)
                .collect();
            assert_eq!(found.len(), 1, "{} of the kind in {model}", found.len());
            found[0]
        };
        let points = |frame: Quat, turn: Quat| (turn * frame) * Vec3::Y;
        let rest = Quat::IDENTITY;
        for (thigh, shin, swing, side) in [
            (WalkBoneKind::ThighL, WalkBoneKind::ShinL, 1.0, -1.0),
            (WalkBoneKind::ThighR, WalkBoneKind::ShinR, -1.0, 1.0),
        ] {
            // A thigh and a shin hang down; the thigh rests out on its own
            // side (the left on the body's -x) and is stood LEG_SPLAY out,
            // the shin going with it; the thigh swung forward points forward
            // of where it rested (the body faces -z); the shin bends back as
            // its leg goes back, and not as it comes forward.
            let (thigh_frame, shin_frame) = (frame(thigh), frame(shin));
            let rested = points(thigh_frame, rest);
            assert!(rested.y < -0.9 && rested.x * side > 0.05, "{rested:?}");
            assert!(points(shin_frame, rest).y < -0.9, "{:?}", points(shin_frame, rest));
            let stance = stance_from_rest(thigh, thigh_frame);
            assert!(stance * -side > 0.0, "{stance}");
            let stood = points(thigh_frame, walk_offset(thigh, 0.0, 0.0, stance));
            let out = (stood.x * side).atan2(-stood.y);
            assert!((out - LEG_SPLAY).abs() < 1e-3, "{stood:?}");
            assert_eq!(stance_from_rest(shin, shin_frame), 0.0);
            let forward = points(thigh_frame, walk_offset(thigh, swing, 1.0, stance));
            assert!(forward.z < stood.z - 0.3, "{forward:?}");
            let bent = points(shin_frame, walk_offset(shin, -swing, 1.0, 0.0));
            assert!(bent.z > points(shin_frame, rest).z + 0.2, "{bent:?}");
            assert!(points(shin_frame, walk_offset(shin, swing, 1.0, 0.0)).abs_diff_eq(points(shin_frame, rest), 1e-5));
        }
        for (arm, side) in [(WalkBoneKind::ArmL, -1.0), (WalkBoneKind::ArmR, 1.0)] {
            // Standing, an arm hangs ARM_HANG_BELOW under level, out a little
            // on its own side (the left is on the body's -x), however the
            // model rests it; walking, both swing forward and back; punching,
            // the right reaches level in front.
            let arm_frame = frame(arm);
            let stance = stance_from_rest(arm, arm_frame);
            assert!(stance * -side > 0.0, "{stance}");
            let hung = points(arm_frame, walk_offset(arm, 0.0, 0.0, stance));
            let below = (-hung.y).atan2(hung.x.abs());
            assert!((below - ARM_HANG_BELOW).abs() < 1e-3 && hung.x * side > 0.1, "{hung:?}");
            let swung = points(arm_frame, walk_offset(arm, 1.0, 1.0, stance));
            assert!((swung.z - hung.z).abs() > 0.3, "{swung:?}");
        }
        let right = frame(WalkBoneKind::ArmR);
        let stance = stance_from_rest(WalkBoneKind::ArmR, right);
        let punch = points(right, punch_offset(WalkBoneKind::ArmR, stance, 1.0).unwrap());
        assert!(punch.z < -0.9 && punch.y.abs() < 0.2, "{punch:?}");
        // Drinking, the right arm points across and up at the mouth, and
        // comes up to there and goes back down; the left is left to walk.
        let hung = points(right, walk_offset(WalkBoneKind::ArmR, 0.0, 0.0, stance));
        let at_mouth = points(right, drink_offset(WalkBoneKind::ArmR, right, stance, 1.0).unwrap());
        assert!(at_mouth.distance(DRINK_REACH.normalize()) < 1e-3, "{at_mouth:?}");
        assert!(at_mouth.x < -0.3 && at_mouth.y > 0.2 && at_mouth.z < -0.5, "{at_mouth:?}");
        assert!(points(right, drink_offset(WalkBoneKind::ArmR, right, stance, 0.0).unwrap()).distance(hung) < 1e-3);
        assert!(drink_offset(WalkBoneKind::ArmL, right, stance, 1.0).is_none());
        let (begun, half, done) = (Drinking { left: 1.6 }, Drinking { left: 0.8 }, Drinking { left: 0.0 });
        assert_eq!((begun.lifted(), half.lifted(), done.lifted()), (0.0, 1.0, 0.0));
        // With a sword the right arm is a little forward of hanging, at the
        // ready; a slash raises it to about level and sweeps it from the
        // right of straight ahead across to the left, and brings it back.
        let ready = points(right, sword_offset(WalkBoneKind::ArmR, stance, None).unwrap());
        assert!(ready.z < hung.z - 0.3 && ready.y < -0.7, "{ready:?}");
        let melee = ui::ui().melee;
        let pose = |age: f32| points(right, sword_offset(WalkBoneKind::ArmR, stance, Some(slash_at(&melee, age))).unwrap());
        let (from, through, to) = (pose(0.0), pose(melee.swing_out * 0.5), pose(melee.swing_out));
        assert!(from.x > 0.6 && to.x < -0.5 && through.z < -0.8, "{from:?} {through:?} {to:?}");
        for swung in [from, through, to] {
            assert!(swung.y.abs() < 0.4 && swung.z < 0.0, "{swung:?}");
        }
        assert!(pose(melee.swing()).distance(ready) < 1e-3);
        assert!(sword_offset(WalkBoneKind::ArmL, stance, None).is_none());
        // The blade points out in front at the ready, and the way the arm
        // reaches through a slash: from the right, across, to the left.
        assert!(blade_way(None).z < -0.9 && blade_way(None).y > 0.1);
        let edge = |age: f32| blade_way(Some(slash_at(&melee, age)));
        assert!(edge(0.0).x > 0.8 && edge(melee.swing_out).x < -0.7 && edge(melee.swing_out * 0.5).z < -0.9);
        assert!(edge(0.0).dot(from) > 0.9 && edge(melee.swing_out).dot(to) > 0.9, "the blade goes with the arm");
        assert!(edge(melee.swing()).distance(blade_way(None)) < 1e-3);
        // An arm that rests lower than it hangs, and a leg that rests
        // straighter than it stands, are left as they rest: never turned up,
        // in or out.
        let low = Quat::from_rotation_z(3.04);
        assert!((low * Vec3::Y).y < -0.99 && (low * Vec3::Y).x < 0.0);
        assert_eq!(stance_from_rest(WalkBoneKind::ArmL, low), 0.0);
        let straight = Quat::from_rotation_z(3.1) * Quat::from_rotation_x(0.05);
        assert_eq!(stance_from_rest(WalkBoneKind::ThighL, straight), 0.0);
        assert_eq!(stance_from_rest(WalkBoneKind::ThighR, Quat::from_rotation_z(-3.1)), 0.0);
    }

    #[test]
    fn two_quick_taps_near_each_other_are_a_double_tap() {
        let mut taps = DoubleTap::default();
        let near = 50.0;
        assert!(!taps.tap(1.0, 1.1, Vec2::new(100.0, 100.0), near));
        assert!(taps.tap(1.3, 1.35, Vec2::new(120.0, 110.0), near));
        // A third is the first of the next two.
        assert!(!taps.tap(1.5, 1.55, Vec2::new(120.0, 110.0), near));
        // Too long after, or too far off, and it is the first again.
        assert!(!taps.tap(2.0, 2.05, Vec2::new(120.0, 110.0), near));
        assert!(!taps.tap(2.2, 2.25, Vec2::new(300.0, 110.0), near));
        assert!(taps.tap(2.4, 2.45, Vec2::new(310.0, 100.0), near));
    }

    #[test]
    fn a_double_tap_swings_it_round_behind_you_in_a_moment() {
        // Facing +x, the camera off to the side and looking down steeply.
        let body = Transform::default().looking_to(Vec3::X, Vec3::Y);
        // Tapped just after a finger left the camera, the stick held.
        let mut assist = CameraAssist {
            untouched: 0.0,
            pushed: 5.0,
            ..default()
        };
        assist.snap_behind(&body);
        let yaw = assist.snap.expect("swinging");
        // And walking brings it round again at once, with no wait.
        assert_eq!(assist.ready(), 1.0);
        let mut you = orbit(2.5, 1.3, 9.0);
        let dt = 1.0 / 120.0;
        let mut took = 0.0;
        while !snap_to(&mut you, yaw, dt) {
            took += dt;
            assert!(took < 1.0, "never got there");
        }
        assert!(took < 0.6, "{took} s");
        assert!(wrap_angle(you.yaw - OrbitCamera::behind(&body).yaw).abs() < 1e-6);
        assert_eq!(you.pitch, CAMERA_PITCH);
        assert_eq!(you.distance, 9.0);
        // In your eyes it looks straight ahead.
        let mut eyes = orbit(-1.0, -0.5, 0.0);
        while !snap_to(&mut eyes, yaw, dt) {}
        let view = camera_view(&eyes, &body, None);
        assert!(view.forward().distance(Vec3::X) < 1e-4, "{}", view.forward());
        // A finger on the camera stops it.
        assist.touched();
        assert_eq!(assist.snap, None);
    }

    #[test]
    fn a_double_tap_goes_behind_where_you_stood_at_the_tap_though_you_turn_after() {
        // Walking with the stick held to the side, you face a quarter turn
        // off the camera's ahead, wherever the camera turns to. Swung round
        // behind you as you face each frame, it would chase you round for
        // ever; swung behind you as you stood at the tap, it gets there.
        let dt = 1.0 / 120.0;
        let mut you = orbit(0.0, CAMERA_PITCH, 9.0);
        let facing = |you: &OrbitCamera| {
            let ahead = camera_ahead(&camera_view(you, &Transform::default(), None));
            Transform::default().looking_to(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * ahead, Vec3::Y)
        };
        // Chased: behind you as you face each frame, which is a quarter turn
        // off wherever it has got to.
        let mut chased = orbit(0.0, CAMERA_PITCH, 9.0);
        let mut took = 0.0;
        loop {
            let mark = OrbitCamera::behind(&facing(&chased)).yaw;
            if took >= 2.0 || snap_to(&mut chased, mark, dt) {
                break;
            }
            took += dt;
        }
        assert!(took >= 2.0, "got there in {took} s chasing you round");
        // Behind you as you stood at the tap, and you turn as it comes round.
        let mut assist = CameraAssist::default();
        assist.snap_behind(&facing(&you));
        let fixed = assist.snap.unwrap();
        took = 0.0;
        while let Some(yaw) = assist.snap {
            assert_eq!(yaw, fixed, "the mark moved");
            if snap_to(&mut you, yaw, dt) {
                assist.snap = None;
            }
            took += dt;
            assert!(took < 1.0, "never got there");
        }
        assert!(wrap_angle(you.yaw - fixed).abs() < 1e-6);
        assert!(wrap_angle(OrbitCamera::behind(&facing(&you)).yaw - you.yaw).abs() > 1.0, "still facing off it");
    }

    #[test]
    fn talking_it_looks_at_their_face_from_near_you() {
        let you = Transform::default();
        let eyes = Vec3::Y * EYE_HEIGHT;
        // Someone just in front of you: from your eyes.
        let face = Vec3::new(0.3, 1.52, -1.0);
        let view = close_up_view(&you, face, None);
        assert!(view.translation.distance(eyes) < 1e-4, "{}", view.translation);
        // Someone further off: from as near them as it comes, at the height
        // of your eyes, between you and them.
        let face = Vec3::new(3.0, 1.52, -4.0);
        let view = close_up_view(&you, face, None);
        assert!(((face - view.translation).xz().length() - CLOSE_UP).abs() < 1e-3);
        assert!((view.translation.y - EYE_HEIGHT).abs() < 1e-4);
        assert!(view.translation.xz().dot(face.xz()) > 0.0);
        // Looking at them, their face a little over the middle of the
        // screen.
        let toward = (face - view.translation).normalize();
        assert!(view.forward().dot(toward) > 0.97);
        assert!(view.up().dot(toward) > 0.02);
    }

    #[test]
    fn the_close_up_comes_in_goes_back_out_and_cuts() {
        let mut close_up = CloseUp::default();
        assert!(!close_up.busy());
        close_up.on = Some(Vec3::ONE);
        let dt = 1.0 / 120.0;
        let mut took = 0.0;
        while close_up.step(dt) < 1.0 {
            took += dt;
        }
        assert!((took - CLOSE_UP_IN).abs() < 0.02, "{took}");
        close_up.on = None;
        assert!(close_up.busy());
        let mut took = 0.0;
        while close_up.step(dt) > 0.0 {
            took += dt;
            // Going back out from the face it was on.
            assert_eq!(close_up.face, Vec3::ONE);
        }
        assert!((took - CLOSE_UP_OUT).abs() < 0.02, "{took}");
        assert!(!close_up.busy());
        close_up.on = Some(Vec3::ONE);
        close_up.step(dt);
        close_up.cut();
        assert!(!close_up.busy());
    }
}

#[cfg(test)]
mod ground_tests {
    use super::camera_tests::island;
    use super::*;

    #[test]
    fn over_the_void_a_body_falls_for_ever_and_over_the_sea_it_wades() {
        let nothing = island::Island::new(&[]);
        let step = |bottomless: bool| {
            let motion = Motion {
                ground: Ground {
                    island: &nothing,
                    decks: &[],
                    bottomless,
                },
                solids: &[],
                stays_ashore: false,
                dt: 1.0 / 60.0,
                elapsed: 0.0,
                melee: ui::ui().melee,
            };
            let mut transform = Transform::from_xyz(0.0, 0.25, 0.0);
            let mut jump = PlayerJump::default();
            let mut shove = Shove::default();
            let mut footing = None;
            for _ in 0..120 {
                footing = motion.step(&mut transform, &mut jump, &mut shove, &Intent::default());
            }
            (transform.translation.y, footing)
        };
        // Two seconds off the edge of the town: dead, and never stood on
        // anything.
        let (y, footing) = step(true);
        assert!(y < void_y(), "{y}");
        assert_eq!(footing, None);
        // Off any other island: wading on the sea bed.
        let (y, footing) = step(false);
        assert!(y > WATER_Y - 1.5 && y < WATER_Y, "{y}");
        assert_eq!(footing, Some(Underfoot::Island));
    }

    #[test]
    fn a_punch_knocks_a_body_back_and_up_and_not_again_while_it_reels() {
        let melee = ui::ui().melee;
        let mut shove = Shove::default();
        let mut jump = PlayerJump::default();
        let (from, at) = (Vec2::ZERO, Vec2::new(2.0, 0.0));
        assert!(shove.knock(&mut jump, from, at, false));
        assert!((shove.across - Vec2::new(melee.knock_back, 0.0)).length() < 1e-4, "{}", shove.across);
        assert_eq!(jump.velocity_y, melee.knock_up);
        // Reeling, the next punch does nothing; after, it halves what is
        // left and adds its own, more for a punch on the move, and the
        // rise is never more than one knock's.
        assert!(!shove.knock(&mut jump, from, at, true));
        shove.hurt = 0.0;
        assert!(shove.knock(&mut jump, from, at, true));
        let expected = melee.knock_back * 0.5 + melee.knock_back + melee.sprint_bonus;
        assert!((shove.across.x - expected).abs() < 1e-3, "{}", shove.across);
        assert!(jump.velocity_y <= melee.knock_up);

        // Carried along, it dies away: in the air slowly, Minecraft's 0.91
        // a tick, and on the ground within a few frames.
        let flat = island::Island::new(&[
            [Vec3::new(-50.0, 0.0, -50.0), Vec3::new(-50.0, 0.0, 50.0), Vec3::new(50.0, 0.0, 50.0)],
            [Vec3::new(-50.0, 0.0, -50.0), Vec3::new(50.0, 0.0, 50.0), Vec3::new(50.0, 0.0, -50.0)],
        ]);
        let motion = Motion {
            ground: Ground::bare(&flat),
            solids: &[],
            stays_ashore: false,
            dt: 1.0 / 60.0,
            elapsed: 0.0,
            melee,
        };
        let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);
        let mut jump = PlayerJump::default();
        let mut shove = Shove {
            across: Vec2::new(melee.knock_back, 0.0),
            hurt: 0.0,
        };
        jump.velocity_y = melee.knock_up;
        let mut rose = false;
        for _ in 0..180 {
            motion.step(&mut transform, &mut jump, &mut shove, &Intent::default());
            rose |= transform.translation.y > 0.5;
        }
        assert!(rose, "never left the ground");
        assert_eq!(shove.across, Vec2::ZERO, "still carried after three seconds");
        let went = transform.translation.x;
        assert!(went > 1.5 && went < 6.0, "{went} m from one punch standing still");
    }

    #[test]
    fn out_of_the_car_your_legs_are_your_own_again() {
        use bevy::ecs::system::RunSystemOnce;
        // The car left the body pacing at 40 m/s facing east; a frame of
        // input with nothing pressed takes both back.
        let mut world = World::new();
        world.insert_resource(ButtonInput::<KeyCode>::default());
        world.insert_resource(TouchControls::default());
        world.insert_resource(hud::DialogUp::default());
        world.insert_resource(bridge::Bridging::default());
        world.insert_resource(bridge::RanOut::default());
        world.insert_resource(arena::Outfit::default());
        world.insert_resource(arena::FightTime::default());
        let you = world
            .spawn((
                Player,
                Transform::default(),
                Venue::Town,
                Intent {
                    walk: Vec3::ZERO,
                    jump: false,
                    pace: Some(40.0),
                    face: Some(Vec3::X),
                    sprint: 1.0,
                },
                WalkCycle::default(),
            ))
            .id();
        world.run_system_once(read_player_input).expect("the input is read");
        let intent = world.get::<Intent>(you).unwrap();
        assert_eq!(intent.pace, None, "still paced as the car was");
        assert_eq!(intent.face, None, "still facing the car's way");
    }

    #[test]
    fn out_of_bridge_you_stand_until_the_stick_is_let_go() {
        use bevy::ecs::system::RunSystemOnce;
        // Stopped where the bridge ran out, with the stick still pushed:
        // you stand. The stick let go, you still stand, and are stopped no
        // longer; pushed again, you walk. With keys as with the stick.
        for keys in [false, true] {
            let mut world = World::new();
            world.insert_resource(ButtonInput::<KeyCode>::default());
            world.insert_resource(TouchControls::default());
            world.insert_resource(hud::DialogUp::default());
            world.insert_resource(bridge::Bridging::default());
            world.insert_resource(arena::Outfit::default());
            world.insert_resource(arena::FightTime::default());
            let mut ran_out = bridge::RanOut::default();
            ran_out.halted = true;
            world.insert_resource(ran_out);
            let you = world.spawn((Player, Transform::default(), Venue::Arena, Intent::default(), WalkCycle::default())).id();
            let push = |world: &mut World, on: bool| {
                if keys {
                    let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
                    if on {
                        keyboard.press(KeyCode::KeyW);
                    } else {
                        keyboard.release(KeyCode::KeyW);
                    }
                } else {
                    world.resource_mut::<TouchControls>().stick_value = if on { Vec2::new(0.0, 1.0) } else { Vec2::ZERO };
                }
            };
            let walking = |world: &mut World| {
                world.run_system_once(read_player_input).expect("the input is read");
                (world.get::<Intent>(you).unwrap().walk.length(), world.get::<WalkCycle>(you).unwrap().speed)
            };
            push(&mut world, true);
            for _ in 0..3 {
                assert_eq!(walking(&mut world), (0.0, 0.0), "keys {keys}: walking while stopped");
                assert!(world.resource::<bridge::RanOut>().halted, "keys {keys}: let go of with the stick pushed");
            }
            push(&mut world, false);
            assert_eq!(walking(&mut world), (0.0, 0.0));
            assert!(!world.resource::<bridge::RanOut>().halted, "keys {keys}: still stopped with the stick let go");
            push(&mut world, true);
            let (walk, legs) = walking(&mut world);
            assert!(walk > 0.99 && legs > 0.0, "keys {keys}: not walking with the stick pushed again: {walk}");
        }
    }

    #[test]
    fn a_punch_lands_on_the_nearest_body_in_front_and_in_reach() {
        let melee = ui::ui().melee;
        let a = Entity::from_bits(1);
        let b = Entity::from_bits(2);
        let c = Entity::from_bits(3);
        let d = Entity::from_bits(4);
        let facing = Vec2::NEG_Y;
        let nearer = (melee.near + melee.reach) * 0.5;
        let bodies = || {
            [
                // Ahead, at the range's reach; ahead, nearer; behind; and
                // ahead but up on a roof.
                (a, Vec3::new(0.0, 0.0, -melee.reach)),
                (b, Vec3::new(0.1, 0.0, -nearer)),
                (c, Vec3::new(0.0, 0.0, melee.reach + 2.0)),
                (d, Vec3::new(0.0, 4.0, -nearer)),
            ]
            .into_iter()
        };
        let hit = punch_lands(&melee, Vec2::ZERO, 0.0, facing, bodies());
        assert_eq!(hit.map(|(body, _)| body), Some(b));
        // Out of reach: nobody.
        let far = punch_lands(&melee, Vec2::ZERO, 0.0, facing, [(a, Vec3::new(0.0, 0.0, -(melee.reach + 1.0)))].into_iter());
        assert_eq!(far, None);
        // Facing straight away from them all: nobody.
        let away = punch_lands(&melee, Vec2::ZERO, 0.0, Vec2::Y, bodies());
        assert_eq!(away, None);
    }

    #[test]
    fn a_wall_counted_on_its_own_is_solid_under_a_gable_inside_out() {
        // A wall of one of the town's houses, made again round its holes,
        // standing on the house's floor on the ground, and the Square
        // House's front gable on top of it, inside out: its underside facing
        // up, at the very height of the top of the wall. Counted with the
        // rest, the two were taken for one surface, and the wall was open to
        // walk through (2026-10-30); counted on its own, it is solid.
        let v = Vec3::new;
        let boxes = super::camera_tests::boxes;
        let wall = boxes(&[(v(0.0, 0.35, 0.0), v(4.0, 5.35, 0.5))]);
        let floor = boxes(&[(v(-1.0, 0.25, -1.0), v(5.0, 0.35, 1.5))]);
        let ground = vec![
            [v(-9.0, 0.25, -9.0), v(-9.0, 0.25, 9.0), v(9.0, 0.25, 9.0)],
            [v(-9.0, 0.25, -9.0), v(9.0, 0.25, 9.0), v(9.0, 0.25, -9.0)],
        ];
        let gable: Vec<[Vec3; 3]> = boxes(&[(v(0.0, 5.35, 0.0), v(4.0, 7.0, 0.5))])
            .into_iter()
            .map(|[a, b, c]| [a, c, b])
            .collect();
        let rest: Vec<[Vec3; 3]> = floor.iter().chain(&ground).chain(&gable).copied().collect();
        let at = Vec2::new(2.0, 0.25);
        let together: Vec<[Vec3; 3]> = rest.iter().chain(&wall).copied().collect();
        assert!(!island::Island::new(&together).blocked(at, 0.6, 1.6), "counted together, the wall is open");
        assert!(island::Island::of_parts(&[&rest, &wall]).blocked(at, 0.6, 1.6));
        // And the room beside it stays open.
        assert!(!island::Island::of_parts(&[&rest, &wall]).blocked(Vec2::new(2.0, 1.2), 0.6, 1.6));
    }

    /// The town's fountain, across one side of it, as `circlemap1.glb` has
    /// it: the floor of its basin 0.05 m up, under the water; the rim 0.55;
    /// the ledge outside it 0.3; and the grass.
    pub(super) fn fountain() -> Island {
        let v = Vec3::new;
        island(&[
            (v(-3.0, -1.0, -3.0), v(1.9, 0.05, 3.0)),
            (v(1.9, -1.0, -3.0), v(2.2, 0.55, 3.0)),
            (v(2.2, -1.0, -3.0), v(2.5, 0.3, 3.0)),
            (v(2.5, -1.0, -3.0), v(8.0, 0.0, 3.0)),
        ])
    }

    #[test]
    fn the_fountain_rim_is_a_jump_out_of_the_basin() {
        let fountain = fountain();
        let rim = Vec2::new(2.05, 0.0);
        let walled = |feet: f32| {
            fountain
                .walls(rim, PLAYER_RADIUS, feet + LAND_STEP_UP, feet + PLAYER_HEIGHT)
                .next()
                .is_some()
        };
        // Half a metre up out of the basin: in the way, and not underfoot.
        assert!(walled(0.05));
        assert!(support(&Ground::bare(&fountain), rim.extend(0.05).xzy(), 0.0).0 < 0.55);
        // A quarter of a metre up from the ledge outside: a step.
        assert!(!walled(0.3));
        assert_eq!(support(&Ground::bare(&fountain), rim.extend(0.3).xzy(), 0.0).0, 0.55);
    }

    #[test]
    fn an_ai_never_steps_down_into_the_basin() {
        let fountain = fountain();
        let on_rim = Vec3::new(2.05, 0.55, 0.0);
        // Down into the basin, which it could not step back up out of.
        assert!(strands(&fountain, on_rim, Vec3::new(1.5, 0.55, 0.0)));
        // Down onto the ledge, and off that onto the grass, which it could.
        assert!(!strands(&fountain, on_rim, Vec3::new(2.35, 0.55, 0.0)));
        assert!(!strands(
            &fountain,
            Vec3::new(2.35, 0.3, 0.0),
            Vec3::new(3.0, 0.3, 0.0)
        ));
        // Nor off the edge of the land into the sea.
        assert!(strands(
            &fountain,
            Vec3::new(7.5, 0.0, 0.0),
            Vec3::new(9.0, 0.0, 0.0)
        ));
    }
}

#[cfg(test)]
mod body_tests {
    use super::camera_tests::island;
    use super::*;

    const DT: f32 = 1.0 / 120.0;

    /// A body on `island` for `frames` frames, from `at`, walking `walk` and
    /// jumping on frame `jump_on`: where it is after each.
    fn frames(island: &Island, at: Vec3, walk: Vec3, jump_on: Option<usize>, frames: usize) -> Vec<Vec3> {
        let mut transform = Transform::from_translation(at);
        let mut jump = PlayerJump::default();
        (0..frames)
            .map(|frame| {
                let motion = Motion {
                    ground: Ground::bare(island),
                    solids: &[],
                    stays_ashore: false,
                    dt: DT,
                    elapsed: frame as f32 * DT,
                    melee: ui::ui().melee,
                };
                let intent = Intent {
                    walk,
                    jump: jump_on == Some(frame),
                    ..default()
                };
                motion.step(&mut transform, &mut jump, &mut Shove::default(), &intent);
                transform.translation
            })
            .collect()
    }

    /// The furthest a body went sideways in one frame.
    fn biggest_shove(path: &[Vec3]) -> f32 {
        path.windows(2)
            .map(|pair| pair[0].xz().distance(pair[1].xz()))
            .fold(0.0, f32::max)
    }

    const GROUND: (Vec3, Vec3) = (Vec3::new(-20.0, -2.0, -20.0), Vec3::new(20.0, 0.0, 20.0));
    /// A block a metre high, its near edge at x = 1.
    const BLOCK: (Vec3, Vec3) = (Vec3::new(1.0, -0.1, -3.0), Vec3::new(3.0, 1.0, 3.0));

    #[test]
    fn coming_down_with_the_legs_on_an_edge_stands_on_it() {
        let island = island(&[GROUND, BLOCK]);
        // The middle of the body 0.15 m short of the edge, the legs over it.
        let path = frames(&island, Vec3::new(0.85, 2.0, 0.0), Vec3::ZERO, None, 120);
        let end = *path.last().unwrap();
        assert!((end.y - 1.0).abs() < 1e-4, "came down at {end}, not on the block");
        assert!(biggest_shove(&path) < 1e-4, "shoved {} sideways", biggest_shove(&path));
    }

    #[test]
    fn jumping_onto_a_ledge_from_beside_it_lands_on_it() {
        // A long one, so that a second of running stays on it.
        let island = island(&[GROUND, (BLOCK.0, BLOCK.1.with_x(15.0))]);
        // Running at the block and jumping just short of it.
        let path = frames(&island, Vec3::new(0.2, 0.0, 0.0), Vec3::X, Some(1), 120);
        let end = *path.last().unwrap();
        assert!((end.y - 1.0).abs() < 1e-4, "ended at {end}, not on the block");
        // Never pushed back the way it came.
        for pair in path.windows(2) {
            assert!(pair[1].x >= pair[0].x - 1e-4, "pushed back from {} to {}", pair[0], pair[1]);
        }
    }

    #[test]
    fn standing_on_a_slope_is_not_standing_on_an_edge() {
        // The bridge's slope, 15°, the steepest ground there is: the feet are
        // on the ground under the middle of them, not on the ground uphill.
        let rise = 15f32.to_radians().tan();
        let corner = |x: f32, z: f32| Vec3::new(x, x * rise, z);
        let ramp = Island::new_for_tests(&[
            [corner(-5.0, -5.0), corner(-5.0, 5.0), corner(5.0, 5.0)],
            [corner(-5.0, -5.0), corner(5.0, 5.0), corner(5.0, -5.0)],
        ]);
        let (feet, _) = support(&Ground::bare(&ramp), Vec3::new(1.0, rise, 0.0), 0.0);
        assert!((feet - rise).abs() < 1e-4, "{feet} on the slope at {rise}");
    }

    /// The fountain across its middle, as boxes: the floor of the basin, the
    /// stem, the middle bowl to 0.736 m out and 0.94 m up, and over it the top
    /// bowl, to 0.491 m out, 1.15 to 1.45 m up.
    fn fountain() -> Island {
        let v = Vec3::new;
        island(&[
            (v(-3.0, -1.0, -3.0), v(3.0, 0.05, 3.0)),
            (v(-0.1, 0.05, -3.0), v(0.1, 1.8, 3.0)),
            (v(-0.736, 0.83, -3.0), v(0.736, 0.94, 3.0)),
            (v(-0.491, 1.15, -3.0), v(0.491, 1.45, 3.0)),
        ])
    }

    /// A body on `ground`, with `decks` moving on it as `place_at` says each
    /// frame, for `frames` frames, from `at`, walking `walk` and jumping on
    /// frame `jump_on`, carried along by whatever deck it rides as
    /// `move_bodies` carries it: where it is after each, and what it stands
    /// on.
    fn ride(
        ground: &Island,
        shape: &Island,
        place_at: impl Fn(usize) -> car::Place,
        at: Vec3,
        walk: Vec3,
        jump_on: Option<usize>,
        frames: usize,
    ) -> Vec<(Vec3, Option<Underfoot>)> {
        let mut transform = Transform::from_translation(at);
        let mut jump = PlayerJump::default();
        let mut riding = false;
        (0..frames)
            .map(|frame| {
                let deck = Deck {
                    car: Entity::PLACEHOLDER,
                    shape,
                    now: place_at(frame + 1),
                    then: place_at(frame),
                    // Further than any of Hajun's cars reaches from its
                    // middle: the bus, 12.5 m long, 6.5 m to its corners.
                    reach: 8.0,
                };
                if riding {
                    deck.carry(&mut transform);
                }
                let decks = [deck];
                let motion = Motion {
                    ground: Ground {
                        island: ground,
                        decks: &decks,
                        bottomless: false,
                    },
                    solids: &[],
                    stays_ashore: false,
                    dt: DT,
                    elapsed: frame as f32 * DT,
                    melee: ui::ui().melee,
                };
                let intent = Intent {
                    walk,
                    jump: jump_on == Some(frame),
                    ..default()
                };
                let footing = motion.step(&mut transform, &mut jump, &mut Shove::default(), &intent);
                match footing {
                    Some(Underfoot::Deck(_)) => riding = true,
                    Some(Underfoot::Island) => riding = false,
                    None => {}
                }
                (transform.translation, footing)
            })
            .collect()
    }

    /// The road, as the town's are, 0.2 m under the paving.
    const ROAD: (Vec3, Vec3) = (Vec3::new(-80.0, -2.0, -80.0), Vec3::new(80.0, -0.2, 80.0));

    #[test]
    fn a_body_dropped_onto_the_car_anywhere_stays_on_it() {
        // Each of Hajun's cars, as its model is now, standing on the road:
        // dropped onto it anywhere a body is wholly over it, half a metre
        // apart, into its seats or onto its hood or its back, the body comes
        // to rest on it, and is never shoved off through its sides. Each
        // object of the model is one shape, all its materials together: the
        // first car's body alone, without the seats sunk into it, was a box
        // with no top over them, which a body in them was shoved out of.
        for model in car::CARS {
            let (pieces, low, high) = car::tests::hajuns_car(model);
            let slices: Vec<&[[Vec3; 3]]> = pieces.iter().map(Vec::as_slice).collect();
            let shape = Island::of_parts_for_tests(&slices);
            let road = island(&[ROAD]);
            let standing = car::Place {
                at: Vec3::new(0.0, -0.2 - low.y, 0.0),
                turn: Quat::IDENTITY,
            };
            let over = standing.at.y + high.y + 0.5;
            let margin = PLAYER_RADIUS + 0.3;
            let mut dropped = 0;
            let mut x = low.x + margin;
            while x <= high.x - margin {
                let mut z = low.z + margin;
                while z <= high.z - margin {
                    let path = ride(&road, &shape, |_| standing, Vec3::new(x, over, z), Vec3::ZERO, None, 180);
                    let (end, footing) = *path.last().unwrap();
                    assert_eq!(footing, Some(Underfoot::Deck(0)), "{model}: dropped at {x}, {z}, ended at {end}");
                    assert!(
                        end.x > low.x && end.x < high.x && end.z > low.z && end.z < high.z,
                        "{model}: {end} out of it"
                    );
                    dropped += 1;
                    z += 0.5;
                }
                x += 0.5;
            }
            assert!(dropped >= 10, "{model}: only {dropped} drops");
        }
    }

    #[test]
    fn standing_on_the_car_it_carries_you_round_with_it() {
        // A flat car 6 m long, its top 1.3 m up, driving along x at 7 m/s and
        // turning a little: whoever stands on it stays where they stand on
        // it, and turns with it.
        let road = island(&[ROAD]);
        let shape = island(&[(Vec3::new(-3.0, 0.0, -1.5), Vec3::new(3.0, 1.3, 1.5))]);
        let place_at = |frame: usize| {
            let t = frame as f32 * DT;
            car::Place {
                at: Vec3::new(7.0 * t, -0.2, 0.0),
                turn: Quat::from_rotation_y(0.4 * t),
            }
        };
        let on = Vec3::new(1.0, 1.3, 0.5);
        let path = ride(&road, &shape, place_at, place_at(0).outward(on) + Vec3::Y * 0.01, Vec3::ZERO, None, 240);
        // Coming down on it, it went on a frame's way without you; from then
        // on, wherever you stand on it is where you stay.
        let landed = place_at(1).inward(path[0].0);
        assert!(landed.distance(on) < 0.1, "came down at {landed} on it");
        for (frame, &(at, footing)) in path.iter().enumerate() {
            assert_eq!(footing, Some(Underfoot::Deck(0)), "frame {frame}");
            let on_it = place_at(frame + 1).inward(at);
            assert!(on_it.distance(landed) < 1e-3, "frame {frame}: at {on_it} on it, not {landed}");
        }
        // Jumping on it, you come down on it where you went up.
        let path = ride(&road, &shape, place_at, place_at(0).outward(on), Vec3::ZERO, Some(10), 120);
        let went_up = place_at(1).inward(path[0].0);
        assert!(path[40].1.is_none(), "not in the air");
        let (end, footing) = *path.last().unwrap();
        assert_eq!(footing, Some(Underfoot::Deck(0)));
        let came_down = place_at(120).inward(end);
        assert!(came_down.distance(went_up) < 1e-3, "went up at {went_up} on it, came down at {came_down}");
        // And walking off the side of it, the way across the way it goes,
        // you come down on the road beside it, and are carried along it no
        // further as it drives on.
        let path = ride(&road, &shape, place_at, place_at(0).outward(on), Vec3::Z, None, 240);
        let (end, footing) = path[119];
        assert_eq!(footing, Some(Underfoot::Island), "ended at {end}");
        assert!((end.y + 0.2).abs() < 1e-3);
        let (later, _) = path[239];
        assert!((later.x - end.x).abs() < 1e-3, "carried on from {end} to {later}");
    }

    #[test]
    fn running_beside_the_car_you_can_jump_on() {
        // The car going along x at 5 m/s, its top 1.3 m over the road; you
        // running beside it, keeping up and closing in, and jumping for it.
        let road = island(&[ROAD]);
        let shape = island(&[(Vec3::new(-3.0, 0.0, -1.5), Vec3::new(3.0, 1.3, 1.5))]);
        let place_at = |frame: usize| car::Place {
            at: Vec3::new(5.0 * frame as f32 * DT, -0.2, 0.0),
            turn: Quat::IDENTITY,
        };
        let run = Vec3::new(5.0, 0.0, -3.5).normalize();
        let path = ride(&road, &shape, place_at, Vec3::new(0.0, -0.2, 2.3), run, Some(4), 120);
        let landed = path.iter().position(|&(_, footing)| footing == Some(Underfoot::Deck(0)));
        let Some(landed) = landed else {
            panic!("never came down on it: {:?}", path.last());
        };
        assert!((path[landed].0.y - 1.1).abs() < 1e-3, "came down at {}", path[landed].0);
    }

    #[test]
    fn the_car_driving_into_you_shoves_you_out_of_its_way() {
        // Standing on the road in its way, as it comes at you.
        let road = island(&[ROAD]);
        let shape = island(&[(Vec3::new(-3.0, 0.0, -1.5), Vec3::new(3.0, 1.3, 1.5))]);
        let place_at = |frame: usize| car::Place {
            at: Vec3::new(-10.0 + 7.0 * frame as f32 * DT, -0.2, 0.0),
            turn: Quat::IDENTITY,
        };
        let path = ride(&road, &shape, place_at, Vec3::new(0.0, -0.2, 1.0), Vec3::ZERO, None, 240);
        for (frame, &(at, _)) in path.iter().enumerate() {
            let on_it = place_at(frame + 1).inward(at);
            let inside = on_it.x.abs() < 3.0 && on_it.z.abs() < 1.5;
            assert!(!inside || on_it.z.abs() > 1.5 - LEG_RADIUS, "frame {frame}: inside the car at {on_it}");
            assert!((at.y + 0.2).abs() < 1e-3, "frame {frame}: lifted to {}", at.y);
        }
    }

    #[test]
    fn the_bus_s_thin_walls_keep_you_in_and_out_but_its_doorways_let_you_through() {
        // Hajun, 2026-10-24: walking out of the bus, you went out through its
        // side, and in through it the same way: its walls are a centimetre
        // thick, and bodies are kept out of walls on a quarter-metre lattice.
        let (pieces, low, high) = car::tests::hajuns_car("mybuilds/bus.glb");
        let slices: Vec<&[[Vec3; 3]]> = pieces.iter().map(Vec::as_slice).collect();
        let shape = Island::of_parts_for_tests(&slices);
        let road = island(&[ROAD]);
        let standing = car::Place {
            at: Vec3::new(0.0, -0.2 - low.y, 0.0),
            turn: Quat::IDENTITY,
        };
        let floor = shape.floor(Vec2::ZERO, low.y + 1.0).expect("a floor in the bus");
        // Along its right side, where the middle of a body sweeps straight out
        // of it and where it does not: the middles of the longest run of each,
        // a doorway and a wall.
        let gets_out = |z: f32| {
            let middle = Vec3::new(0.0, floor + SWEPT_AT, z);
            shape.sweep(middle, middle + Vec3::X * (high.x + 1.0), PLAYER_RADIUS).is_none()
        };
        let along: Vec<f32> = (0..=((high.z - low.z) / 0.1) as usize)
            .map(|k| low.z + k as f32 * 0.1)
            .collect();
        let middle_of_longest = |open: bool| {
            let mut runs: Vec<(usize, usize)> = Vec::new();
            for (k, &z) in along.iter().enumerate() {
                if gets_out(z) != open {
                    continue;
                }
                match runs.last_mut() {
                    Some((_, end)) if *end + 1 == k => *end = k,
                    _ => runs.push((k, k)),
                }
            }
            let (start, end) = runs.into_iter().max_by_key(|&(start, end)| end - start).expect("a run");
            (along[start] + along[end]) * 0.5
        };
        let (doorway, wall) = (middle_of_longest(true), middle_of_longest(false));
        let inside = |z: f32| standing.outward(Vec3::new(0.0, floor + 0.01, z));
        let outside = |z: f32| Vec3::new(high.x + 1.5, -0.2, z);
        let walk = |at: Vec3, way: Vec3, frames: usize| *ride(&road, &shape, |_| standing, at, way, None, frames).last().unwrap();
        // Walking at its side from inside, you stay in it, on its floor; and
        // at it from outside, out of it.
        let (end, footing) = walk(inside(wall), Vec3::X, 120);
        assert_eq!(footing, Some(Underfoot::Deck(0)), "walked out through the wall to {end}");
        assert!(end.x < high.x, "walked out through the wall to {end}");
        let (end, _) = walk(outside(wall), Vec3::NEG_X, 120);
        assert!(end.x > high.x, "walked in through the wall to {end}");
        // Slanting into it, you slide along it.
        let (end, _) = walk(inside(wall), Vec3::new(1.0, 0.0, 1.0).normalize(), 30);
        assert!(end.x < high.x && end.z > wall + 0.5, "stuck at the wall at {end}");
        // Its doorway lets you out, onto the road, and in, onto its floor.
        let (end, footing) = walk(inside(doorway), Vec3::X, 120);
        assert_eq!(footing, Some(Underfoot::Island), "not out at {end}");
        assert!(end.x > high.x + 0.5, "not out at {end}");
        let (end, footing) = walk(outside(doorway), Vec3::NEG_X, 60);
        assert_eq!(footing, Some(Underfoot::Deck(0)), "not in at {end}");
        assert!(end.x < high.x - 0.5, "not in at {end}");
    }

    #[test]
    fn the_middle_bowl_of_the_fountain_can_be_stood_on() {
        let fountain = fountain();
        // Down onto it from a jump, with the legs over its brim and clear of
        // the top bowl over it.
        for x in [0.72, 0.8, 0.9] {
            let path = frames(&fountain, Vec3::new(x, 2.5, 0.0), Vec3::ZERO, None, 180);
            let end = *path.last().unwrap();
            assert!((end.y - 0.94).abs() < 1e-4, "from {x}: ended at {end}, not on the middle bowl");
            assert!(biggest_shove(&path) < 0.12, "from {x}: shoved {}", biggest_shove(&path));
        }
    }
}
