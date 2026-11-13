//! The computer's players in Bridge Wars (since 2026-11-12; Hajun: "right
//! now the other bots dont do anything other than stay in their base. i
//! need the bots to be intelligent enough. think of a way to make this game
//! enjoyable. the difficulty need to moderate - not too hard, not too easy
//! ... maybe one way is every 1~2 seconds the bot scores a short list of
//! goals and does the highest one ... and a bonus for the current goal so
//! the bot doesn't flip between two every tick").
//!
//! One brain for the server's bots online and the device's own in a game
//! played offline, as the rules are one ([`crate::bridge_wars`]). It plays
//! by them: nothing is free, and everything is lost on death.
//!
//! * [`Body`]: a bot's body, moved over the map ([`Nav`]) as a body moves:
//!   walked where a body can walk, carried by a knock, landed on whatever
//!   ground is under it, land or plank, and fallen where there is none.
//! * [`Squad`]: every bot of a game. Each has a slow layer, which every
//!   second or so scores what it might be doing and takes the best, the
//!   one it is on getting a bonus ([`Goal`]); and a fast one, every frame,
//!   which does it ([`Squad::step`]): follows its way there, fights whoever
//!   comes into reach, lays its planks, and keeps to the ground.
//! * [`Order`]: what a frame of that comes to that the game has to carry
//!   out, whoever runs it: a punch, a purchase, a rung laid.
//! * [`Skill`]: every number of how hard they play, in one place.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use crate::bridge_wars::{
    AIR_DRAG, BLIND_SIGHT, BOT_PUNCH_EVERY, BRIDGE_A_BUY, BUILDER_METRES, BUILDER_SPEED, DRINK_SECS, DRINK_SLOW,
    DYNAMITE_FURTHEST, DYNAMITE_FUSE, DYNAMITE_NEAREST, DYNAMITE_RADIUS, FAST,
    FIREBALL_RANGE,
    GROUND_DRAG, HUNT_RADIUS, KNOCK_BACK, KNOCK_UP, Mint, Outfits, REACH, RUN_SPEED, SPRINT_BONUS, price,
};
use crate::nav::{BODY, Gap, Nav, Region, Span};
use crate::{
    AMETHYST_PADS, ANCHOR_RADIUS, ARENA_ANCHORS, ARENA_ISLANDS, ARENA_PADS, ARENA_SEATS, ARENA_SHOPS, ARENA_TEAMS,
    ARENA_UPGRADE_SHOPS, Item, Kit, Purse, SAPPHIRE_PADS, Upgrades, Utility, Ware, arena_team,
};

/// Gravity on a body knocked up, on the way up and on the way down, in
/// metres a second a second: the game's `GRAVITY` and `FALL_GRAVITY`.
const RISE_GRAVITY: f32 = 22.0;
const FALL_GRAVITY: f32 = 33.0;
/// How high a rise a body walks up, in metres: the game's step, and the
/// centimetre the map's floors are kept to.
const STEP: f32 = 0.51;
/// This near a point of its way a bot has come to it, in metres.
const ARRIVED: f32 = 0.35;
/// How far ahead of its feet a bot's bridge is laid, in metres, at the
/// least, as the Bridge Builder's is.
const LAID_AHEAD: f32 = 1.6;
/// How near a bot stands off whoever it fights, as a share of its reach.
const PRESS_IN: f32 = 0.7;
/// How fast a bot hops out of the pond, upward and across, in metres a
/// second: over its rim, 0.6 m over its floor.
const HOP_UP: f32 = 6.5;
const HOP_ACROSS: f32 = 5.0;
/// How fast a body leaves the ground in a jump of its own, in metres a
/// second: the game's `JUMP_SPEED`.
const JUMP_UP: f32 = 8.5;
/// How far a body knocked off an edge looks for ground to steer back to,
/// in metres.
const REACH_BACK: f32 = 8.0;
/// How far whoever a punch lands on is carried by it, in metres: what a
/// bot goes by in choosing where to stand ([`flung`]). A knock standing
/// carries about 3 and a running one more.
const CARRY: [f32; 2] = [3.0, 4.5];
/// How far out from whoever it fights a bot stands to punch them, how far
/// back it steps between punches when it pokes at them, and how far off
/// it stands when it waits for them to come, in metres.
const RING: f32 = 1.45;
const POKE_BACK: f32 = 3.6;
const HOLD_OFF: f32 = 4.2;
/// How much a bridge may slant where it leaves the land and where it
/// meets it, as so much to the side for each metre on (about 24
/// degrees); and how many times as long as the straight line between its
/// ends it may be, bends and all ([`Squad::bends`]).
const MOST_SLANT: f32 = 0.45;
const MOST_BENT: f32 = 1.05;
/// How long a body is in the air in a jump of its own, in seconds, at the
/// least (the game's `JUMP_SPEED` against its two gravities; a frame or
/// two more as it is stepped); and how much of the ground it would cross
/// in that while a bot keeps in hand when it jumps a gap, in metres: it
/// comes down that far past where the ground begins again, or does not
/// jump ([`Squad::reach_of_a_jump`]). At a run that is a gap of 5.8 m:
/// the four planks a stick of dynamite mostly takes out of a bridge
/// (5.45 m), and not the five it sometimes does (6.8), which a body at a
/// run does clear, but by too little to count on.
const JUMP_SECS: f32 = 0.7;
const JUMP_SPARE: f32 = 1.2;
/// How far ahead a bot reads where whoever it is after will be, at the
/// most, in seconds ([`Squad::meet`]).
const FORESEE: f32 = 1.6;
/// How far past where a stick of dynamite's blast reaches a bot gets out
/// of its way, and how far short of it one waits that was walking into
/// it, in metres ([`Squad::mind_sticks`]).
const CLEAR_OF_BLAST: f32 = 1.0;
const SHORT_OF_BLAST: f32 = 3.0;
/// And how much further off than that one walking at a stick takes note
/// of it, in metres: what it runs in the while it takes to wake up to it.
const NOTICED_FROM: f32 = 12.0;
/// How long a bot takes to wake up to a stick of dynamite thrown at it, in
/// seconds from the throw, at the soonest and at the latest: the worst of
/// them at a fight ([`Persona::pvp`] nothing), and how much sooner the
/// best. A stick is in the air about 0.6 s and lies 1.2 s, and getting out
/// of its blast takes a third of a second: the best are moving about as it
/// comes down, most by its second flash, and the worst are caught by one
/// thrown from close by that comes down on them about two times in five
/// (a test counts them, kind by kind). In a fight everyone is that much
/// later ([`STICK_BUSY`]), which is when a stick dropped at your own feet
/// catches them. One lying ahead on its way a bot has seen coming, and
/// takes this share of the time over ([`STICK_AHEAD`]).
const STICK_WAKES: (f32, f32) = (0.75, 2.0);
const STICK_WAKES_SOONER: (f32, f32) = (0.4, 1.0);
const STICK_BUSY: f32 = 0.25;
const STICK_AHEAD: f32 = 0.5;
/// A bot and someone who keeps coming at its base ([`Vigil`]), every
/// number of it a spread that each time is thrown from: how long it looks
/// after them when they step back, before it has had enough of them, in
/// seconds; how long it stands its ground once it has, before it either
/// goes at them or lets them be; how far from it they may be for it to go
/// at them from where it stands, in metres, for a bot as bold as most; how
/// far from the middle of its island they have to be to be gone, and for
/// how long; how far from them it stands, and how far in from the edge of
/// its island; and how long after they were last about it has forgotten
/// them. All mine: Hajun asked for the patience, and for it to differ from
/// one to the next ([`Persona::patience`]).
const VIGIL_LOOK: (f32, f32) = (0.8, 4.0);
const VIGIL_STAND: (f32, f32) = (12.0, 36.0);
const VIGIL_WATCH: (f32, f32) = (5.0, 12.0);
const VIGIL_GONE: (f32, f32) = (24.0, 38.0);
const VIGIL_CALM: (f32, f32) = (2.5, 8.0);
const VIGIL_OFF: (f32, f32) = (3.0, 5.0);
const VIGIL_INSIDE: (f32, f32) = (0.9, 2.2);
const VIGIL_FORGET: (f32, f32) = (50.0, 110.0);
/// What a bot's offer to stand its ground scores, of a score of 1
/// ([`Goal::Guard`]): over anything it does about its island, and, with
/// what the goal it is on has over the rest, still under backing off hurt.
const GUARD_SCORE: f32 = 0.79;
/// How much further off than it would go for someone a bot keeps after
/// one it is on already, as so many times as far; how far either side of
/// how near it lets people come it wanders from one while to the next
/// ([`Bot::keen`]), and for how long each while is, in seconds; how long a
/// blow smarts, in which it goes for whoever is near; and how far either
/// side of where it backs off hurt its nerve is from one fright to the
/// next, as a share of its health. Nothing a bot does begins or ends on a
/// line that is the same twice (Hajun, 2026-11-13: "if some thing becomes
/// a definite if-then, then its not a real human, but a program").
const KEEPS_ON: f32 = 1.3;
const KEEN: (f32, f32) = (0.72, 1.35);
const KEEN_LASTS: (f32, f32) = (4.0, 11.0);
const STING: (f32, f32) = (2.2, 6.5);
const NERVE: (f32, f32) = (-0.08, 0.08);
/// A bridge this short is a hole mended, and is laid straight, with no
/// bends of its own ([`Squad::lay`]).
const MEND: f32 = 12.0;
/// How far inside what stands, either side of a hole, the planks that
/// mend it begin and end, in metres: as a bridge's first and last rungs
/// lie inside the shores it joins.
const MEND_INSET: f32 = 0.5;

/// How hard the computer's players play: every number of their pacing, in
/// one place, to tune by.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Skill {
    /// How often a bot thinks again of what to do, in seconds, between
    /// these two.
    pub think_every: (f32, f32),
    /// The bonus the goal it is on has over the rest, of a score of 1.
    pub sticky: f32,
    /// How long after coming into reach of someone a bot's first punch
    /// comes, in seconds, and how long between its punches after.
    pub windup: f32,
    pub punch_every: f32,
    /// How long between a bot's punches on an anchor, in seconds.
    pub anchor_every: f32,
    /// How near someone of another team has to come for a bot minding its
    /// own business to go for them, in metres: this at first, growing to
    /// its full sight between the two times, in seconds into the fight.
    pub aggro: f32,
    pub aggro_grows: (f32, f32),
    /// Under this share of its health, with someone after it, a bot backs
    /// off; and it comes back once it has this much again.
    pub retreat_below: f32,
    pub recovered: f32,
    /// No bot goes to another team's island to fight before this, in
    /// seconds into the fight; none touches an anchor before
    /// `anchors_from`; and the one that keeps its team's base does not
    /// go along before `keeper_raids_from`.
    pub raid_from: f32,
    pub anchors_from: f32,
    pub keeper_raids_from: f32,
    /// Until this many seconds in, the bots break no more anchors once
    /// this many are broken, whoever broke them: nobody is to be out in
    /// the first minutes.
    pub anchors_spared: (usize, f32),
    /// How long a raid stays on an island at the most, in seconds; how
    /// long a bot that died on one, or came back from one, stays home
    /// after; and how long a team that was raided is left alone after.
    pub raid_secs: f32,
    pub raid_rest: f32,
    pub victim_rest: f32,
    /// The most of the raids that go to a team with a person on it, as a
    /// share of all of them, while there are other teams to go to: no
    /// more than a team's share of four.
    pub people_share: f32,
    /// From when everyone goes for everyone, wherever they are, in
    /// seconds: a while after the anchors break, so that the end of a
    /// game is minutes and not moments. Until then the raids go on as
    /// they were, each death the last.
    pub hunt_from: f32,
    /// Under this share of its health a bot backs off once its anchor is
    /// gone, when dying is for good.
    pub careful_below: f32,
    /// How long between a bot's fireballs, in seconds.
    pub throw_every: f32,
    /// How long after a knock lands a bot is carried by it and no more,
    /// before it answers, in seconds, between these two; and how much of
    /// its pace it then steers by in the air, between these two. Thrown
    /// again at every knock: now it is back on its feet at once, now it
    /// goes over the edge.
    pub reel: (f32, f32),
    pub grip: (f32, f32),
}

impl Skill {
    /// Not too hard, not too easy.
    pub const MODERATE: Skill = Skill {
        think_every: (1.0, 1.5),
        sticky: 0.15,
        windup: 0.3,
        punch_every: BOT_PUNCH_EVERY,
        anchor_every: 2.0,
        aggro: 6.0,
        aggro_grows: (150.0, 600.0),
        retreat_below: 0.35,
        recovered: 0.6,
        raid_from: 150.0,
        anchors_from: 240.0,
        keeper_raids_from: 330.0,
        anchors_spared: (2, 480.0),
        raid_secs: 45.0,
        raid_rest: 40.0,
        victim_rest: 90.0,
        people_share: 1.0 / 3.0,
        hunt_from: 780.0,
        careful_below: 0.5,
        throw_every: 6.0,
        reel: (0.06, 0.36),
        grip: (0.5, 1.0),
    };

    /// How near someone has to come for a bot to go for them unprovoked,
    /// `elapsed` seconds into the fight.
    pub fn aggro_at(&self, elapsed: f32) -> f32 {
        let (from, to) = self.aggro_grows;
        let share = ((elapsed - from) / (to - from).max(1.0)).clamp(0.0, 1.0);
        self.aggro + (HUNT_RADIUS - self.aggro).max(0.0) * share
    }
}

/// Dice of a bot's own: the same seed, the same game.
#[derive(Clone, Debug)]
pub struct Dice(u64);

impl Dice {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// A number from 0 up to 1.
    pub fn roll(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn between(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.roll()
    }

    /// A number about 0 as a bell rings them: two in three within 1 of
    /// it, and none further than 3.5.
    pub fn bell(&mut self) -> f32 {
        (self.roll() + self.roll() + self.roll() + self.roll() - 2.0) * 1.732
    }

    /// A number from `low` to `high`, mostly about the middle.
    pub fn around(&mut self, low: f32, high: f32) -> f32 {
        ((low + high) * 0.5 + (high - low) / 6.0 * self.bell()).clamp(low.min(high), high.max(low))
    }
}

/// A bot's body: where it is, which way it faces, what a knock is
/// carrying it by, across the ground and upward, and whether it is
/// jumping a gap ([`Body::leap`]).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Body {
    pub at: Vec3,
    /// Its turn about the upright, in radians: -Z turned by this is ahead.
    pub yaw: f32,
    pub shove: Vec2,
    pub lift: f32,
    /// It means to leave the ground it is on: walking, it goes on over any
    /// ground there is, to its very edge, and off that with a jump, as
    /// anyone jumps a gap, where otherwise it never walks off an edge
    /// (since 2026-11-13). Its brain's to say, each frame.
    pub leap: bool,
}

impl Body {
    pub fn standing(at: Vec3, yaw: f32) -> Self {
        Self {
            at,
            yaw,
            ..Self::default()
        }
    }

    /// Whether it is on the ground and its own to move: not in the air,
    /// nor being carried along by a knock.
    pub fn free(&self, nav: &Nav) -> bool {
        self.lift == 0.0
            && self.shove == Vec2::ZERO
            && nav.ground(self.at.xz()).is_some_and(|ground| (self.at.y - ground).abs() < 0.02)
    }

    /// Knocks it, by a punch or a blast from `from`, thrown on the run if
    /// `running`: up, and away from there, on top of half of whatever
    /// already carried it, as a punch knocks anyone. `share` is how much
    /// of the knock it takes.
    pub fn knock(&mut self, from: Vec2, running: bool, share: f32) {
        let away = (self.at.xz() - from).normalize_or(Vec2::NEG_Y);
        let push = (KNOCK_BACK + if running { SPRINT_BONUS } else { 0.0 }) * share;
        let up = KNOCK_UP * share;
        self.shove = self.shove * 0.5 + away * push;
        self.lift = (self.lift * 0.5 + up).min(up);
    }

    /// Jumps it `way`: up and across, for getting out of the pond.
    pub fn hop(&mut self, way: Vec2) {
        self.lift = HOP_UP;
        self.shove = way.normalize_or_zero() * HOP_ACROSS;
    }

    /// Jumps it, as anyone jumps: up, from the ground it stands on.
    /// Whether it did.
    pub fn jump(&mut self, nav: &Nav) -> bool {
        let grounded = self.lift == 0.0 && nav.ground(self.at.xz()).is_some_and(|ground| (self.at.y - ground).abs() < 0.02);
        if grounded {
            self.lift = JUMP_UP;
        }
        grounded
    }

    /// Moves it on by `dt` seconds: up and down as a knock or a jump threw
    /// it, landing on whatever ground it comes down onto and falling
    /// where there is none; carried across by a knock, dying away as it
    /// goes; and walked by `walk`, metres a second across the ground, all
    /// the while, as anyone's legs are their own in the air and with a
    /// knock on them (until 2026-11-12 a bot knocked was carried and no
    /// more until it had come to rest; Hajun: "the bot is like stunned
    /// ... so its so easy to knock them off the map"). On the ground it
    /// walks only where a body can walk; in the air only over ground, or,
    /// over the void already, anywhere, which is how one knocked off an
    /// edge steers back. It never walks off an edge: a knock is what
    /// takes it off one.
    pub fn step(&mut self, nav: &Nav, walk: Vec2, dt: f32) {
        let mut here = self.at.xz();
        let ground = nav.ground(here);
        // Stood a little under the ground there, as a game that stands it
        // where its seat arrives may stand it (the device's arrivals are
        // at the land's own height, under an island's top): onto it.
        if self.lift == 0.0
            && let Some(ground) = ground
            && self.at.y < ground
            && ground - self.at.y <= STEP
        {
            self.at.y = ground;
        }
        let floor = ground.unwrap_or(f32::NEG_INFINITY);
        // In the air: going up, over the ground, or under it, having
        // fallen past an edge and been carried in under the land or a
        // plank, where it goes on falling (until 2026-11-12 it hung there,
        // neither on the ground nor in the air, for the rest of the game).
        let airborne = self.lift > 0.0 || self.at.y > floor + 1e-3 || self.at.y < floor - 1e-3;
        if airborne {
            let before = self.at.y;
            self.at.y += self.lift * dt;
            self.lift -= if self.lift > 0.0 { RISE_GRAVITY } else { FALL_GRAVITY } * dt;
            // Down onto ground it was over: not up through an island from
            // under it.
            if self.at.y <= floor && before >= floor - 0.3 {
                self.at.y = floor;
                self.lift = 0.0;
            }
        } else if self.lift != 0.0 {
            // Come down to within a hair of the ground without quite
            // touching it: it is on it. (Until 2026-11-13 it stood there
            // still falling, as far as it knew, for as long as it stayed on
            // ground of that height: neither in the air nor its own to
            // move, so it never jumped nor drank.)
            self.at.y = floor;
            self.lift = 0.0;
        }
        if self.shove != Vec2::ZERO {
            here += self.shove * dt;
            self.at.x = here.x;
            self.at.z = here.y;
            let drag = if airborne { AIR_DRAG } else { GROUND_DRAG };
            self.shove *= drag.powf(dt * 20.0);
            if self.shove.length_squared() < 0.01 {
                self.shove = Vec2::ZERO;
            }
        }
        if walk == Vec2::ZERO {
            return;
        }
        // Where a body walks; or, off that already, knocked against a
        // stand or onto an island's rim, any ground, to get back to it.
        let astray = !nav.walkable_at(here);
        let whole = here + walk * dt;
        let tries = [whole, Vec2::new(whole.x, here.y), Vec2::new(here.x, whole.y)];
        if airborne {
            // In the air: over the void, wherever it steers, as it does
            // jumping a gap; over ground, only on over ground, and not out
            // of where a body walks.
            let over_void = nav.ground(here).is_none();
            let open = |to: Vec2| over_void || self.leap || ((nav.walkable_at(to) || astray) && nav.ground(to).is_some());
            if let Some(to) = tries.into_iter().find(|&to| to != here && open(to)) {
                self.at.x = to.x;
                self.at.z = to.y;
            }
        } else if self.leap {
            // Jumping a gap: on over any ground there is, to its very
            // edge, and off that with a jump.
            match nav.ground(whole) {
                Some(there) if there - self.at.y <= STEP => {
                    self.at.x = whole.x;
                    self.at.z = whole.y;
                    if self.at.y - there <= STEP {
                        self.at.y = there;
                    }
                }
                // Something too high to step up.
                Some(_) => {}
                None => {
                    self.lift = JUMP_UP;
                    self.at.x = whole.x;
                    self.at.z = whole.y;
                }
            }
        } else {
            let open = |to: Vec2| {
                (nav.walkable_at(to) || astray) && nav.ground(to).is_some_and(|there| there - self.at.y <= STEP)
            };
            if let Some(to) = tries.into_iter().find(|&to| to != here && open(to)) {
                self.at.x = to.x;
                self.at.z = to.y;
                if let Some(ground) = nav.ground(to) {
                    // Down a step at once, as up one; down a drop, it
                    // falls.
                    if self.at.y - ground <= STEP {
                        self.at.y = ground;
                    }
                }
            }
        }
        if walk.length_squared() > 1e-4 {
            self.yaw = (-walk.x).atan2(-walk.y);
        }
    }
}

/// What a bot is doing, as its slow layer last chose.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Goal {
    /// Nothing in particular: at home, by its anchor or its pad.
    Idle,
    /// On its team's pad, for what its generators make.
    Collect,
    /// To its island's shop for this, and buying it.
    Shop(Ware),
    /// Laying the bridge from the one land to the other.
    Lay(Region, Region),
    /// To a pad somewhere for what lies on it.
    Fetch(Vec2, Item),
    /// Home, where someone of another team is, to see them off.
    Defend,
    /// Between its base and someone who was at it and has only stepped
    /// back: looking after them a moment, or, having had enough of their
    /// coming and going, standing its ground until they are gone for good
    /// ([`Vigil`]).
    Guard,
    /// Away from whoever is after it, to get its health back.
    Retreat,
    /// To this team's island: to fight whoever is home, take what lies on
    /// their pad, and, once anchors are fair game, break theirs.
    Raid(u8),
    /// After whoever of another team is nearest, wherever they are: once
    /// the anchors are gone.
    Hunt,
}

impl Goal {
    /// Whether `other` is the same goal, whatever it is for.
    fn alike(&self, other: &Goal) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Which of a team's two a bot is: the one that keeps the base, or the
/// one that goes out from it. The lower seat keeps; and a bot whose
/// teammate is a person keeps, whichever its seat.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Keeper,
    Runner,
}

/// How a team of the computer's opens (since 2026-11-12; Hajun: "add
/// variation to strategy. some ai might choose to build bridge directly
/// to the center, some ai might choose to bridge from their base to the
/// diamond generators in the side, some ai might choose to bridge
/// later").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Opening {
    /// A bridge to the middle at once, and from the middle to a smaller
    /// island after: the only way they opened until then.
    Middle,
    /// A bridge from its own island straight to a smaller island beside
    /// it, 75 m and four buys, for its sapphire; the middle after, when
    /// it comes to that.
    Sapphire,
    /// Arms first: no bridge until a while in.
    Late,
}

/// What a team of the computer's means to do, as the one of its two that
/// goes out would have it ([`Persona::plan`]): how it opens; from when it
/// builds its first bridge, in seconds into the fight; which of the two
/// smaller islands beside its own it goes for; and from when it builds to
/// the middle, having opened to a smaller island.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Plan {
    pub opening: Opening,
    pub bridge_from: f32,
    pub side: u8,
    pub middle_from: f32,
}

impl Plan {
    /// As every team played until 2026-11-12: to the middle at once.
    const PLAIN: Plan = Plan {
        opening: Opening::Middle,
        bridge_from: 0.0,
        side: 0,
        middle_from: 0.0,
    };
}

/// A way of playing: who a bot is. Each of the computer's players takes
/// one by chance as it is seated, two of a game the same one as likely as
/// not, and plays the whole game by it (since 2026-11-12; Hajun: "at game
/// start the bots each take a certain personality/playstyle that is a
/// predefined set of probability distribution for the if ___ then do ___
/// thing for the various situations. for example like a estp personality
/// might u know, buy the action items, want to engage in combat in a
/// active way and yolo. or like a isfj personality might wanna play safe,
/// just bridging to the diamond islands and not the main land, and
/// upgrading his generator/defenses, or like intj might wanna optimize
/// their stack first and then go battle, or esfp might wanna just troll
/// and buy invisible potion and go to the enemy base and camp and knock
/// them off ... there should be more personality than 8, so that each
/// game feels new and each character shows some distinctive
/// characteristic"). Sixteen of them, by the four letters people sort
/// each other by ([`PERSONAS`]). Every number here is odds or a leaning,
/// not a rule: what it does at each turn is still thrown for.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Persona {
    pub name: &'static str,
    /// How much it wants to be the one of its team that goes out, from 0
    /// to 1: of a team's two, the one that wants it more goes, and the
    /// other keeps the base.
    pub drive: f32,
    /// Its odds of opening with a bridge to the middle, with one straight
    /// to a smaller island, and with none until later ([`Opening`]).
    pub opening: [f32; 3],
    /// Having opened to a smaller island, from when it builds to the
    /// middle of its own accord, in seconds into the fight: the careful
    /// ones leave it a long while.
    pub middle_from: f32,
    /// What it spends on first, each a leaning about 1: its sword, its
    /// shield, something for its second button, its team's generator,
    /// its team's defences (better shields, the heal pool, traps), and
    /// its team's swords.
    pub spends: [f32; 6],
    /// What it carries on its buttons for things to use (five of them
    /// since 2026-11-13; Hajun: "there is 4 more item slot, so the ai
    /// should be adjusted so it can buy up to 5 even. match it with the
    /// personality so more diverse interaction"): its leaning to each
    /// kind, as odds ([`KINDS`]: a fireball, a speed potion, an
    /// invisibility potion, a healing potion, a Bridge Builder and, since
    /// 2026-11-13, a stick of dynamite), and
    /// how many kinds it carries, at the least and at the most. Which of
    /// them, and in what order it buys them, is thrown by these each life
    /// ([`Persona::loadout`]): the likeliest mostly first. (One kind a
    /// life until then, there being one button.)
    pub carries: [f32; 6],
    pub pockets: [u8; 2],
    /// How much better than the rest it is in a fight, hand to hand, from
    /// 0 to 1 (since 2026-11-13; Hajun, having played them: "pvp seems
    /// moderate right now for all the bots equally ... i can always win
    /// even in 1v2 ... the bots, depending on personality, need to vary
    /// in pvp skill based on personality so some bots are like pro-er, and
    /// some bots are moderate like now"). At 0 it fights as they all did
    /// until then; at 1 its punches come as fast as they count, each
    /// thrown on the run, from out of your reach and back out of it; it
    /// is on its feet again sooner after a knock; it reads where you are
    /// going rather than where you are; and it is quicker to jump a gap
    /// after you and to get out from under a stick of dynamite
    /// ([`Temper::pvp`]).
    pub pvp: f32,
    /// How many times someone has to come at its base and step back
    /// before it stands its ground against them rather than go back to
    /// what it was doing, about: from 1, at once, to 3 (since 2026-11-13;
    /// Hajun, who had a team's anchor down by stepping onto their island
    /// and back onto his bridge over and over, the two of them coming at
    /// him and going back to their generator each time: "Real humans have
    /// like a patience(?) threshold that depends on the person - some
    /// people, after 1 agitation they choose to just stand between me and
    /// their generator ... some people who have stronger patience, maybe
    /// they will go back to their generator 1 time but after i come back
    /// again to attack they will be like oh this guy keep coming back, so
    /// ill just stay here until he rly rly leaves, and some ppl have more
    /// patience so maybe they will wait until the 3rd agitation"). Each
    /// bot's own is thrown about its kind's, and again each time it has
    /// forgotten whoever it was ([`Vigil`]).
    pub patience: f32,
    /// How readily it goes in, about 1 (how near it lets someone come
    /// before it goes for them, and how hurt it has to be to back off);
    /// how much it jumps about, from 0 to 1; and how widely it weaves as
    /// it walks, about 1.
    pub bold: f32,
    pub bouncy: f32,
    pub weave: f32,
    /// Its leaning to each way of fighting, about 1, on top of what the
    /// place calls for: pressing, circling, poking, waiting, lying in
    /// wait, and rushing ([`Stance`]).
    pub fights: [f32; 6],
    /// How much it goes for throwing people off the edge, and how much it
    /// minds being thrown off itself, each about 1.
    pub shoves: f32,
    pub minds: f32,
    /// How much sooner (under 0) or later than the rest it first goes
    /// raiding, in seconds; how much it likes a raid, added to how it
    /// scores one; how long it stays on one, as so many times the usual;
    /// and how likely it is to go for the anchor there rather than for
    /// whoever is home and what lies on their pad, from 0 to 1.
    pub eager: f32,
    pub raids: f32,
    pub lingers: f32,
    pub anchors: f32,
}

impl Persona {
    /// What it means to carry this life, in the order it buys them, and
    /// how many of each it keeps: as many kinds as its pockets have it,
    /// each thrown by its leanings from the kinds not yet taken. Fireballs
    /// by the handful for one that leans to them, two healing potions for
    /// one that leans on those, two sticks of dynamite for one a fifth of
    /// whose leanings are to it, and one of anything else.
    fn loadout(&self, dice: &mut Dice) -> Vec<(Utility, u8)> {
        // No more kinds than there are buttons for them.
        let all = (KINDS.len().min(crate::UTILITY_SLOTS)) as u8;
        let least = self.pockets[0].clamp(1, all);
        let most = self.pockets[1].clamp(least, all);
        let many = (least + (dice.roll() * f32::from(most - least + 1)) as u8).min(most);
        let whole = self.carries.iter().sum::<f32>().max(1e-3);
        let mut odds = self.carries;
        let mut carried = Vec::new();
        for _ in 0..many {
            if odds.iter().all(|odd| *odd <= 0.0) {
                break;
            }
            let which = pick(dice, &odds);
            let share = self.carries[which] / whole;
            let count = match KINDS[which] {
                Utility::Fireball if share >= 0.35 => 3,
                Utility::Fireball if share >= 0.15 => 2,
                Utility::Healing if share >= 0.45 => 2,
                Utility::Dynamite if share >= 0.2 => 2,
                _ => 1,
            };
            carried.push((KINDS[which], count));
            odds[which] = 0.0;
        }
        carried
    }

    /// How its team opens, by its odds, if it is the one that goes out.
    fn plan(&self, dice: &mut Dice) -> Plan {
        let opening = [Opening::Middle, Opening::Sapphire, Opening::Late][pick(dice, &self.opening)];
        Plan {
            opening,
            bridge_from: if opening == Opening::Late { dice.around(45.0, 120.0) } else { 0.0 },
            side: (dice.roll() * 2.0) as u8 % 2,
            middle_from: if opening == Opening::Sapphire { self.middle_from } else { 0.0 },
        }
    }
}

/// The kinds of thing there are to carry on a button, in the order a
/// kind's leanings to them are written ([`Persona::carries`]).
pub const KINDS: [Utility; 6] = [
    Utility::Fireball,
    Utility::Speed,
    Utility::Invisibility,
    Utility::Healing,
    Utility::Builder,
    Utility::Dynamite,
];

/// Which of `odds` comes up: each as likely as it is big.
fn pick(dice: &mut Dice, odds: &[f32]) -> usize {
    let mut left = dice.roll() * odds.iter().sum::<f32>();
    for (which, weight) in odds.iter().enumerate() {
        if *weight > 0.0 && left <= *weight {
            return which;
        }
        left -= weight;
    }
    odds.iter().rposition(|weight| *weight > 0.0).unwrap_or(0)
}

/// Every way of playing there is. My reading of each, from Hajun's four
/// examples on: the numbers are leanings to tune.
pub const PERSONAS: [Persona; 16] = [
    // The daredevil: the action items, into every fight, and never mind
    // the edge.
    Persona {
        name: "ESTP",
        drive: 0.9,
        opening: [0.8, 0.05, 0.15],
        middle_from: 0.0,
        spends: [1.3, 0.6, 1.6, 0.5, 0.5, 0.9],
        carries: [0.5, 0.35, 0.1, 0.05, 0.05, 0.3],
        pvp: 0.9,
        patience: 1.4,
        pockets: [3, 5],
        bold: 1.4,
        bouncy: 0.8,
        weave: 1.3,
        fights: [1.4, 1.0, 1.2, 0.3, 0.4, 1.8],
        shoves: 1.0,
        minds: 0.5,
        eager: -30.0,
        raids: 0.08,
        lingers: 1.0,
        anchors: 0.3,
    },
    // The troll: an invisibility potion, over to someone's island, and
    // there to lurk by the edge and knock them off it.
    Persona {
        name: "ESFP",
        drive: 0.85,
        opening: [0.7, 0.1, 0.2],
        middle_from: 60.0,
        spends: [0.9, 0.6, 1.8, 0.5, 0.5, 0.6],
        carries: [0.15, 0.2, 0.6, 0.05, 0.1, 0.35],
        pvp: 0.55,
        patience: 2.4,
        pockets: [2, 4],
        bold: 1.15,
        bouncy: 1.0,
        weave: 1.6,
        fights: [0.7, 1.3, 1.0, 1.0, 2.2, 1.0],
        shoves: 1.8,
        minds: 0.8,
        eager: -15.0,
        raids: 0.07,
        lingers: 1.8,
        anchors: 0.05,
    },
    // The commander: straight for the anchors, armed for it.
    Persona {
        name: "ENTJ",
        drive: 0.8,
        opening: [0.75, 0.05, 0.2],
        middle_from: 0.0,
        spends: [1.5, 1.0, 0.8, 1.1, 0.8, 1.5],
        carries: [0.4, 0.4, 0.05, 0.15, 0.15, 0.15],
        pvp: 0.8,
        patience: 1.5,
        pockets: [2, 4],
        bold: 1.25,
        bouncy: 0.4,
        weave: 0.8,
        fights: [1.4, 1.0, 1.0, 0.6, 0.8, 1.3],
        shoves: 1.1,
        minds: 1.0,
        eager: -10.0,
        raids: 0.08,
        lingers: 1.1,
        anchors: 0.9,
    },
    // The trickster: odd openings, a fireball, in and out, and a shove.
    Persona {
        name: "ENTP",
        drive: 0.7,
        opening: [0.35, 0.4, 0.25],
        middle_from: 90.0,
        spends: [1.0, 0.7, 1.5, 0.9, 0.7, 0.8],
        carries: [0.45, 0.2, 0.3, 0.05, 0.3, 0.4],
        pvp: 0.65,
        patience: 2.2,
        pockets: [3, 5],
        bold: 1.1,
        bouncy: 0.7,
        weave: 1.5,
        fights: [0.7, 1.4, 1.5, 0.9, 1.6, 1.1],
        shoves: 1.6,
        minds: 0.9,
        eager: 0.0,
        raids: 0.04,
        lingers: 1.3,
        anchors: 0.3,
    },
    // The captain: a bit of everything, and its team's swords and
    // shields.
    Persona {
        name: "ENFJ",
        drive: 0.65,
        opening: [0.7, 0.1, 0.2],
        middle_from: 60.0,
        spends: [1.1, 1.1, 0.8, 1.2, 1.3, 1.3],
        carries: [0.2, 0.3, 0.05, 0.45, 0.1, 0.05],
        pvp: 0.4,
        patience: 1.6,
        pockets: [2, 3],
        bold: 1.05,
        bouncy: 0.45,
        weave: 0.9,
        fights: [1.1, 1.1, 1.0, 0.9, 0.9, 1.0],
        shoves: 1.0,
        minds: 1.0,
        eager: 0.0,
        raids: 0.03,
        lingers: 1.0,
        anchors: 0.6,
    },
    // The free spirit: a speed potion, all over the map, jumping.
    Persona {
        name: "ENFP",
        drive: 0.8,
        opening: [0.55, 0.25, 0.2],
        middle_from: 45.0,
        spends: [0.9, 0.7, 1.5, 0.7, 0.7, 0.7],
        carries: [0.2, 0.55, 0.2, 0.05, 0.1, 0.15],
        pvp: 0.5,
        patience: 2.7,
        pockets: [2, 4],
        bold: 1.2,
        bouncy: 1.0,
        weave: 1.8,
        fights: [1.0, 1.5, 1.2, 0.6, 0.8, 1.5],
        shoves: 1.0,
        minds: 0.6,
        eager: -25.0,
        raids: 0.06,
        lingers: 0.8,
        anchors: 0.3,
    },
    // The enforcer: head on, and on to the anchor.
    Persona {
        name: "ESTJ",
        drive: 0.7,
        opening: [0.8, 0.05, 0.15],
        middle_from: 0.0,
        spends: [1.3, 1.2, 0.7, 1.0, 1.0, 1.2],
        carries: [0.35, 0.25, 0.0, 0.4, 0.05, 0.1],
        pvp: 0.7,
        patience: 1.3,
        pockets: [2, 3],
        bold: 1.2,
        bouncy: 0.3,
        weave: 0.6,
        fights: [1.6, 0.8, 0.8, 0.7, 0.7, 1.2],
        shoves: 1.0,
        minds: 1.0,
        eager: -5.0,
        raids: 0.05,
        lingers: 1.0,
        anchors: 0.8,
    },
    // The host: its own island first, with traps and a pool for whoever
    // comes.
    Persona {
        name: "ESFJ",
        drive: 0.45,
        opening: [0.65, 0.15, 0.2],
        middle_from: 120.0,
        spends: [1.0, 1.3, 0.9, 1.1, 1.6, 1.0],
        carries: [0.1, 0.15, 0.05, 0.7, 0.1, 0.2],
        pvp: 0.2,
        patience: 1.5,
        pockets: [1, 3],
        bold: 1.0,
        bouncy: 0.5,
        weave: 1.0,
        fights: [1.2, 1.0, 0.9, 1.1, 1.3, 0.8],
        shoves: 1.1,
        minds: 1.2,
        eager: 25.0,
        raids: -0.02,
        lingers: 0.9,
        anchors: 0.5,
    },
    // The mastermind: everything bought first, and then to battle.
    Persona {
        name: "INTJ",
        drive: 0.5,
        opening: [0.35, 0.25, 0.4],
        middle_from: 150.0,
        spends: [1.4, 1.3, 0.7, 1.6, 1.0, 1.4],
        carries: [0.3, 0.3, 0.1, 0.3, 0.3, 0.25],
        pvp: 0.6,
        patience: 1.8,
        pockets: [3, 5],
        bold: 1.0,
        bouncy: 0.25,
        weave: 0.6,
        fights: [1.0, 1.0, 1.3, 1.2, 1.3, 0.7],
        shoves: 1.3,
        minds: 1.3,
        eager: 55.0,
        raids: 0.05,
        lingers: 1.2,
        anchors: 0.8,
    },
    // The tinkerer: its generator, something clever for its button, and
    // no hurry to fight.
    Persona {
        name: "INTP",
        drive: 0.3,
        opening: [0.3, 0.3, 0.4],
        middle_from: 180.0,
        spends: [0.9, 0.9, 1.3, 1.7, 1.1, 0.9],
        carries: [0.4, 0.2, 0.25, 0.15, 0.45, 0.4],
        pvp: 0.2,
        patience: 2.5,
        pockets: [3, 5],
        bold: 0.8,
        bouncy: 0.3,
        weave: 1.1,
        fights: [0.7, 1.0, 1.4, 1.4, 1.2, 0.6],
        shoves: 1.2,
        minds: 1.2,
        eager: 40.0,
        raids: -0.02,
        lingers: 0.9,
        anchors: 0.5,
    },
    // The watchman: careful, and waiting for you beside your own bridge.
    Persona {
        name: "INFJ",
        drive: 0.25,
        opening: [0.4, 0.4, 0.2],
        middle_from: 240.0,
        spends: [0.9, 1.3, 0.9, 1.1, 1.6, 0.8],
        carries: [0.1, 0.1, 0.1, 0.7, 0.1, 0.3],
        pvp: 0.35,
        patience: 1.1,
        pockets: [1, 3],
        bold: 0.8,
        bouncy: 0.2,
        weave: 0.8,
        fights: [0.8, 0.9, 1.0, 1.4, 1.8, 0.5],
        shoves: 1.3,
        minds: 1.4,
        eager: 45.0,
        raids: -0.03,
        lingers: 0.8,
        anchors: 0.4,
    },
    // The dreamer: out of everyone's way for as long as it can be.
    Persona {
        name: "INFP",
        drive: 0.2,
        opening: [0.3, 0.45, 0.25],
        middle_from: 300.0,
        spends: [0.7, 1.2, 1.0, 1.2, 1.3, 0.6],
        carries: [0.05, 0.25, 0.3, 0.4, 0.1, 0.1],
        pvp: 0.0,
        patience: 2.8,
        pockets: [1, 2],
        bold: 0.65,
        bouncy: 0.5,
        weave: 1.5,
        fights: [0.6, 1.1, 1.1, 1.6, 1.3, 0.6],
        shoves: 0.8,
        minds: 1.5,
        eager: 70.0,
        raids: -0.08,
        lingers: 0.6,
        anchors: 0.2,
    },
    // The sentinel: by the book. The middle, a sword, a shield.
    Persona {
        name: "ISTJ",
        drive: 0.4,
        opening: [0.8, 0.05, 0.15],
        middle_from: 0.0,
        spends: [1.2, 1.3, 0.7, 1.2, 1.2, 1.0],
        carries: [0.15, 0.1, 0.0, 0.75, 0.05, 0.05],
        pvp: 0.4,
        patience: 1.2,
        pockets: [1, 2],
        bold: 0.95,
        bouncy: 0.15,
        weave: 0.5,
        fights: [1.3, 0.8, 0.9, 1.2, 1.0, 0.7],
        shoves: 0.9,
        minds: 1.3,
        eager: 20.0,
        raids: 0.0,
        lingers: 1.0,
        anchors: 0.7,
    },
    // The defender: safe. The smaller islands and not the middle, its
    // generator, and its team's defences.
    Persona {
        name: "ISFJ",
        drive: 0.15,
        opening: [0.1, 0.8, 0.1],
        middle_from: 420.0,
        spends: [0.8, 1.5, 0.9, 1.5, 1.7, 0.6],
        carries: [0.05, 0.05, 0.0, 0.9, 0.3, 0.25],
        pvp: 0.1,
        patience: 1.0,
        pockets: [1, 3],
        bold: 0.7,
        bouncy: 0.2,
        weave: 0.7,
        fights: [0.8, 0.8, 0.8, 1.6, 1.6, 0.5],
        shoves: 0.9,
        minds: 1.5,
        eager: 60.0,
        raids: -0.06,
        lingers: 0.7,
        anchors: 0.3,
    },
    // The lone wolf: a sword and its own two feet. In and out, round
    // you, and off the edge with you.
    Persona {
        name: "ISTP",
        drive: 0.6,
        opening: [0.6, 0.2, 0.2],
        middle_from: 60.0,
        spends: [1.5, 0.8, 1.0, 0.7, 0.6, 0.9],
        carries: [0.35, 0.35, 0.2, 0.1, 0.05, 0.15],
        pvp: 1.0,
        patience: 1.7,
        pockets: [1, 3],
        bold: 1.15,
        bouncy: 0.5,
        weave: 1.0,
        fights: [0.9, 1.4, 1.6, 0.9, 1.2, 1.0],
        shoves: 1.5,
        minds: 1.1,
        eager: -15.0,
        raids: 0.05,
        lingers: 1.0,
        anchors: 0.4,
    },
    // The drifter: here and there, a potion in hand, and never mind the
    // anchors.
    Persona {
        name: "ISFP",
        drive: 0.35,
        opening: [0.35, 0.45, 0.2],
        middle_from: 150.0,
        spends: [0.9, 1.0, 1.3, 0.9, 1.0, 0.7],
        carries: [0.1, 0.4, 0.3, 0.2, 0.1, 0.1],
        pvp: 0.25,
        patience: 2.6,
        pockets: [2, 4],
        bold: 0.85,
        bouncy: 0.7,
        weave: 1.6,
        fights: [0.8, 1.3, 1.2, 1.2, 1.0, 0.8],
        shoves: 0.9,
        minds: 1.2,
        eager: 30.0,
        raids: -0.03,
        lingers: 0.8,
        anchors: 0.2,
    },
];

/// How one bot is, thrown as it takes its seat about what its way of
/// playing has ([`Persona`]), so that no two of a kind are quite alike:
/// how readily it goes in, how much it jumps about, how widely it weaves,
/// how much it goes for throwing people off the edge and how much it
/// minds being thrown off.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Temper {
    pub bold: f32,
    pub bouncy: f32,
    pub weave: f32,
    pub shoves: f32,
    pub minds: f32,
    /// How much better than the rest it is in a fight, from 0 to 1: its
    /// kind's ([`Persona::pvp`]), a little more or less.
    pub pvp: f32,
}

/// A jump a bot is making over a gap ([`Body::leap`]): where it means to
/// come down, how fast it runs at it, until when it keeps to it whatever
/// comes of it, and whether it has left the ground yet.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Leap {
    to: Vec2,
    pace: f32,
    until: f32,
    aloft: bool,
}

/// Where someone was last seen to be and when they last moved, how fast
/// and which way they were going, and how fast that was turning, in
/// radians a second: what a bot reads where they will be from
/// ([`Squad::foresee`]). People's places come a few times a second
/// online, and every frame offline: both are taken as they come.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct Track {
    known: bool,
    at: Vec2,
    since: f32,
    velocity: Vec2,
    /// How they went between the last two places they were seen at, not
    /// smoothed: what the turning is read off.
    last: Vec2,
    turning: f32,
}

/// A stick of dynamite a bot can see: where it lies, or will come down;
/// how long until it blows, in seconds, its flight and its fuse together;
/// and whose it is, by team.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Stick {
    pub at: Vec2,
    pub left: f32,
    pub team: u8,
}

/// How a bot is fighting whoever it fights, for a few seconds at a time
/// (since 2026-11-12; Hajun: "think of some way to randomize this
/// interaction ... so that there is no 'optimal' way to kill bots, and
/// there is constantly surprise/different random fighting interactions").
/// Until then every bot ran straight at whoever it fought and stood in
/// front of them, wherever that was: on the last plank of a bridge, with
/// them to one side of its end, which is where one punch put it in the
/// void.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stance {
    /// Up to them, wherever it is best to stand ([`Squad::spot`]), and at
    /// them.
    Press,
    /// Round them as it punches, one way or the other.
    Circle,
    /// In for a punch and back out of their reach, to come in again on
    /// the run.
    Poke,
    /// Standing off out of their reach: let them come.
    Wait,
    /// On the land beside where their bridge comes ashore, to one side:
    /// whoever steps off it is thrown back over its side.
    Ambush,
    /// Off the bridge it is on and past them onto the land, at a run.
    Rush,
}

/// A fight with one foe: how the bot is going about it and until when,
/// which way round it goes, where it means to stand and until when it
/// keeps to that, until when it is stepping back from a punch, and
/// whether each of them was out on a bridge when it chose.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Duel {
    foe: usize,
    stance: Stance,
    until: f32,
    side: f32,
    spot: Option<Vec2>,
    spot_until: f32,
    back_until: f32,
    me_out: bool,
    foe_out: bool,
    /// When it makes up its mind about jumping a gap to get at them;
    /// until when, having thought better of it, it goes round by the land
    /// instead; and whether the way round was far, when it last looked,
    /// with where they stood then.
    leap_at: f32,
    round_until: f32,
    far: Option<(bool, Vec2)>,
    /// It stepped back from its last punch to come in again as the next
    /// falls due: that one is thrown the moment they are in its reach,
    /// on the run, with no waiting to take it in that they are.
    lunge: bool,
}

/// A bridge a bot is laying: the span, which of its bridges it is, which
/// rung comes next, and how many blasts the map had seen when it was last
/// looked over for holes ([`Nav::cuts`]).
#[derive(Clone, Copy, PartialEq, Debug)]
struct Laying {
    span: Span,
    from: Region,
    to: Region,
    chain: u16,
    next: usize,
    cuts: u32,
}

/// A Bridge Builder a bot has sent off: the bridge it is laying, straight
/// from the one land to the other, which of the bot's bridges that is,
/// which rung comes next, and how far along it the builder has walked.
/// Nobody sees the builder itself, as nobody sees anyone else's: its
/// bridge goes up by itself, a plank or two ahead of where it would be.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Sent {
    span: Span,
    from: Region,
    to: Region,
    chain: u16,
    next: usize,
    along: f32,
}

/// One of the computer's players.
#[derive(Clone, Debug)]
pub struct Bot {
    pub body: Body,
    seat: usize,
    role: Role,
    goal: Goal,
    goal_since: f32,
    think_at: f32,
    /// Its way to where its goal takes it: the points still ahead, where
    /// they lead, and the map as it was when the way was found.
    path: Vec<Vec2>,
    path_to: Option<Vec2>,
    path_version: u32,
    path_at: f32,
    /// Whom it is fighting, since when they have been in its reach, and
    /// when it may next punch.
    foe: Option<usize>,
    in_reach_since: Option<f32>,
    punch_at: f32,
    /// How many metres of bridge it has, how many bridges it has begun,
    /// and the one it is laying.
    metres: f32,
    chains: u16,
    laying: Option<Laying>,
    /// Until when it is drinking a potion, and which.
    drinking: Option<(f32, Utility)>,
    /// What its health was a frame ago, and when it last lost any.
    health_was: f32,
    hurt_at: f32,
    /// Where it was, and when, for telling that it is getting nowhere;
    /// and what it has given up on, until when.
    progress: (Vec2, f32),
    barred: Option<(Goal, f32)>,
    tries: u8,
    /// How many of each thing it would take from a pad now, and how much
    /// it matters.
    wants: Purse,
    urgency: u8,
    /// How fast it went, the frame before.
    speed: f32,
    /// Until when it stays home from raids; since when it has been on the
    /// island it is raiding; and when it may next hit an anchor, and
    /// throw a fireball.
    rest_until: f32,
    raiding_since: Option<f32>,
    strike_at: f32,
    throw_at: f32,
    /// Who it is ([`Persona`]), and how it is itself ([`Temper`]).
    persona: &'static Persona,
    temper: Temper,
    /// What it means to carry on its buttons this life, in the order it
    /// buys them, and how many of each ([`Persona::loadout`]); and, on a
    /// raid, whether it is there for the anchor.
    loadout: Vec<(Utility, u8)>,
    for_anchor: bool,
    /// The bridge its last thought settled on laying next, if any
    /// ([`Squad::next_span`]); and a Bridge Builder it has sent off, and
    /// how far that has got.
    planned: Option<(Region, Region, Span)>,
    sent: Option<Sent>,
    /// How it is fighting whoever it fights; and whom it has given up on
    /// for now, until when.
    duel: Option<Duel>,
    shun: Option<(usize, f32)>,
    /// How long the knock that has just landed carries it before it
    /// answers, to be counted from its next frame; until when that is;
    /// and how much of its pace it steers by in the air this time.
    reel: Option<f32>,
    reel_until: f32,
    grip: f32,
    /// How long after having someone in reach its next punch comes; when
    /// it may next jump; and how far round its weaving is as it walks.
    windup: f32,
    hop_at: f32,
    weave: f32,
    /// Which way it is to face this frame, if not the way it walks.
    face: Option<f32>,
    /// A person of its team is at their pad: what the pad makes is theirs
    /// ([`Squad::wants`]).
    yields: bool,
    /// With nothing to do, where on its island it is wandering to, and
    /// until when.
    loiter: Option<(Vec2, f32)>,
    /// A gap it is jumping ([`Leap`]); a hole in a bridge it is on its
    /// way to, to jump it ([`Nav::crossing`]); and how many blasts the map
    /// had seen when it last found its way ([`Nav::cuts`]).
    leaping: Option<Leap>,
    cross: Option<Gap>,
    path_cuts: u32,
    /// Whoever it is fighting is on their way over a gap to it, this
    /// frame: it jumps none itself until they are down ([`Squad::fight`]).
    awaits: bool,
    /// A stick of dynamite it is in the way of, and when it wakes up to
    /// it; and when it may next throw one of its own.
    alarm: Option<(Vec2, f32)>,
    blast_at: f32,
    /// Beside a person: since when it has been at its team's pad on this
    /// visit, and when it may next come for what lies there
    /// ([`Goal::Collect`]).
    pad_since: Option<f32>,
    pad_again: f32,
    /// Whoever has been coming at its base, and how it is taking it
    /// ([`Vigil`]); how many of their comings it takes before it stands
    /// its ground ([`Persona::patience`]); and what its team's anchor had
    /// left when it last thought.
    vigil: Option<Vigil>,
    patience: u8,
    anchor_was: f32,
    /// How near it lets people come before it goes for them, as so many
    /// times what it usually does, for the while until `keen_until`; how
    /// long the blow it last took smarts; and how far off its usual it
    /// backs off hurt this time ([`KEEN`], [`STING`], [`NERVE`]).
    keen: f32,
    keen_until: f32,
    sting: f32,
    nerve: f32,
}

impl Bot {
    fn new(seat: usize, body: Body, dice: &mut Dice, persona: &'static Persona) -> Self {
        let temper = Temper {
            bold: (persona.bold * (1.0 + 0.1 * dice.bell())).clamp(0.5, 1.6),
            bouncy: (persona.bouncy + 0.1 * dice.bell()).clamp(0.0, 1.0),
            weave: (persona.weave * (1.0 + 0.15 * dice.bell())).clamp(0.3, 2.0),
            shoves: persona.shoves,
            minds: persona.minds,
            pvp: (persona.pvp + 0.07 * dice.bell()).clamp(0.0, 1.0),
        };
        let loadout = persona.loadout(dice);
        Self {
            body,
            seat,
            role: if seat < ARENA_TEAMS { Role::Keeper } else { Role::Runner },
            goal: Goal::Idle,
            goal_since: 0.0,
            // Not all at once.
            think_at: dice.between(0.0, 1.0),
            path: Vec::new(),
            path_to: None,
            path_version: 0,
            path_at: 0.0,
            foe: None,
            in_reach_since: None,
            punch_at: 0.0,
            metres: 0.0,
            chains: 0,
            laying: None,
            drinking: None,
            health_was: 1.0,
            hurt_at: f32::NEG_INFINITY,
            progress: (body.at.xz(), 0.0),
            barred: None,
            tries: 0,
            wants: [0; 4],
            urgency: 0,
            speed: 0.0,
            rest_until: 0.0,
            raiding_since: None,
            strike_at: 0.0,
            throw_at: 0.0,
            persona,
            temper,
            loadout,
            for_anchor: false,
            planned: None,
            sent: None,
            duel: None,
            shun: None,
            reel: None,
            reel_until: f32::NEG_INFINITY,
            grip: 1.0,
            windup: 0.3,
            hop_at: dice.between(1.0, 4.0),
            weave: dice.between(0.0, std::f32::consts::TAU),
            face: None,
            yields: false,
            loiter: None,
            leaping: None,
            awaits: false,
            cross: None,
            path_cuts: 0,
            alarm: None,
            blast_at: 0.0,
            pad_since: None,
            pad_again: 0.0,
            vigil: None,
            patience: patience_of(persona, dice),
            anchor_was: f32::INFINITY,
            keen: dice.around(KEEN.0, KEEN.1),
            keen_until: dice.between(KEEN_LASTS.0, KEEN_LASTS.1),
            sting: 3.0,
            nerve: dice.around(NERVE.0, NERVE.1),
        }
    }

    /// Whoever it has in mind for coming at its base, if anyone.
    pub fn vigil(&self) -> Option<Vigil> {
        self.vigil
    }

    /// How many comings at its base it takes before it stands its ground.
    pub fn patience(&self) -> u8 {
        self.patience
    }

    /// Whether it is standing its ground against whoever keeps coming at
    /// its base, rather than only looking after them a moment.
    pub fn stands_its_ground(&self) -> bool {
        self.vigil.is_some_and(|vigil| vigil.stand.is_some_and(|stand| stand.resolved))
    }

    /// Whether it is in the middle of jumping a gap.
    pub fn leaping(&self) -> bool {
        self.leaping.is_some()
    }

    pub fn goal(&self) -> Goal {
        self.goal
    }

    /// Who it is.
    pub fn persona(&self) -> &'static Persona {
        self.persona
    }

    /// How it is itself.
    pub fn temper(&self) -> Temper {
        self.temper
    }

    /// How it is fighting whoever it fights, if it is fighting.
    /// What it means to carry on its buttons this life, in the order it
    /// buys them, and how many of each.
    pub fn loadout(&self) -> &[(Utility, u8)] {
        &self.loadout
    }

    /// Whether a Bridge Builder it sent off is still laying its bridge.
    pub fn building(&self) -> bool {
        self.sent.is_some()
    }

    pub fn stance(&self) -> Option<Stance> {
        self.duel.map(|duel| duel.stance)
    }

    pub fn role(&self) -> Role {
        self.role
    }

    /// How many metres of bridge it has to lay.
    pub fn metres(&self) -> f32 {
        self.metres
    }

    fn set_goal(&mut self, goal: Goal, now: f32) {
        if self.goal != goal {
            self.goal = goal;
            self.goal_since = now;
            self.path.clear();
            self.path_to = None;
            self.tries = 0;
            self.progress = (self.body.at.xz(), now);
        }
    }
}

/// A bot's mind of someone who has come at its base: on the land of its
/// island, or at its anchor as that lost health. Who, and how many times
/// they have come, counted each time they come again after having stepped
/// back; whether they are at it now; when they were last about; how long
/// after that it will have forgotten them; and how it stands toward them
/// while they are only stepped back ([`Stand`]).
///
/// Until 2026-11-13 a bot had no memory of anyone: it went home for
/// whoever stood on its island, and back to what it was doing the moment
/// they stepped off it, as often as they liked. Now it looks after them a
/// moment each time; and once they have come as many times as it has the
/// patience for, which is 1 to 3 by who it is and by chance, it stands
/// its ground: between them and its anchor, at them if they come near
/// enough, there until they are gone a good way off for a while. Standing
/// there long enough with them still about, it either goes out at them or
/// lets them be, as it is like, and it has not forgotten them either way:
/// the next time they come, it stands at once.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vigil {
    pub foe: usize,
    pub comings: u8,
    at_it: bool,
    about: f32,
    forget: f32,
    stand: Option<Stand>,
}

/// How a bot stands toward someone who was at its base and has stepped
/// back: where, between them and what it keeps; until when, at the
/// longest; whether it has had enough of them and stands its ground, or
/// only looks after them a moment; how far from it they may be for it to
/// go at them; how far from the middle of its island they have to be to
/// be gone, for how long, and since when they have been.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Stand {
    spot: Vec2,
    until: f32,
    resolved: bool,
    watch: f32,
    gone: f32,
    calm: f32,
    gone_since: Option<f32>,
    off: f32,
    inside: f32,
}

/// Where a bot stands between what its team keeps, at `keep`, and someone
/// `there` who has stepped back from it: on the land of its island
/// `home`, on the line from them to it, the nearest place to them that a
/// body can stand which is `off` or more from them and `inside` or more
/// in from where that line comes ashore; or, with too little land between
/// the shore and what it keeps for that, the last place on that land
/// before what it keeps, right in front of it. `None` with no land
/// between them at all: they are on the island still, or beside what it
/// keeps.
fn guard_spot(nav: &Nav, home: Region, keep: Vec2, there: Vec2, off: f32, inside: f32) -> Option<Vec2> {
    let line = keep - there;
    let far = line.length();
    if far < 1.0 {
        return None;
    }
    let way = line / far;
    let ashore = |at: Vec2| nav.grid().region(at) == Some(home) && nav.grid().height(at).is_some() && nav.walkable_at(at);
    let mut landed: Option<f32> = None;
    let mut last = None;
    let mut along = 0.0;
    // (No nearer what it keeps than a body and a half: that is in the
    // way of nothing.)
    while along <= far - 1.3 {
        let at = there + way * along;
        if ashore(at) {
            let from = *landed.get_or_insert(along);
            if along >= off && along - from >= inside {
                return Some(at);
            }
            // (Not on the very edge, at the least.)
            if along - from >= 0.5 {
                last = Some(at);
            }
        } else {
            landed = None;
        }
        along += 0.25;
    }
    last
}

/// How many times someone has to come at a bot's base before it stands
/// its ground, thrown about its kind's: 1, 2 or 3.
fn patience_of(persona: &Persona, dice: &mut Dice) -> u8 {
    (persona.patience + 0.6 * dice.bell()).round().clamp(1.0, 3.0) as u8
}

/// Someone in the game, as a bot sees them: where they are, whether they
/// are standing, in play, unseen, a person, and what share of their
/// health they have.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Seen {
    pub at: Vec3,
    pub alive: bool,
    pub in_play: bool,
    pub unseen: bool,
    pub person: bool,
    pub health: f32,
}

/// The game as the bots see it, for one frame.
pub struct View<'a> {
    /// Seconds into the fight.
    pub elapsed: f32,
    pub nav: &'a Nav,
    /// Everyone, by seat, the zombies past the players'.
    pub seats: &'a [Seen],
    pub purses: &'a [Purse],
    pub outfits: &'a Outfits,
    pub mint: &'a Mint,
    pub anchors: [f32; ARENA_TEAMS],
    /// Every stick of dynamite thrown and not yet blown.
    pub sticks: &'a [Stick],
}

/// What a frame of a bot comes to that the game is to carry out.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Order {
    /// A punch at whoever is in this seat, thrown on the run if `running`.
    pub punch: Option<usize>,
    pub running: bool,
    /// A punch at this team's anchor.
    pub strike: Option<u8>,
    /// Buy this, standing where it is.
    pub buy: Option<Ware>,
    /// Throw a fireball, as a plank heads.
    pub throw: Option<f32>,
    /// Throw a stick of dynamite, as a plank heads, to come down this far
    /// off, in metres.
    pub blast: Option<(f32, f32)>,
    /// This potion, off its button, is drunk.
    pub drink: Option<Utility>,
    /// A Bridge Builder is sent off: one comes off its button. Its bridge
    /// comes as `rungs`, a frame at a time, as one the bot walks does.
    pub build: bool,
    /// Rungs it lays: which of its bridges, which rung, where, and heading
    /// how.
    pub rungs: Vec<(u16, u16, Vec2, f32)>,
    /// How it is to be seen: carrying a plank, or with a potion at its
    /// mouth.
    pub carrying: bool,
    pub drinking: bool,
}

/// Where a team's bots stand for what is on their island.
#[derive(Clone, Copy, Debug, Default)]
struct Spots {
    heart: Vec2,
    pad: Vec2,
    items: Vec2,
    upgrades: Vec2,
    guard: Vec2,
    /// Where another team's bot stands to hit this island's anchor: on
    /// the side of it the bridges come from.
    strike: Vec2,
    /// What the team keeps: its anchor, where the map has it on the
    /// island, and the middle of the island otherwise.
    keep: Vec2,
}

/// Every one of the computer's players in a game.
#[derive(Clone, Debug)]
pub struct Squad {
    pub skill: Skill,
    bots: Vec<Option<Bot>>,
    /// Which spans are being laid, and by whom: one bot to a span.
    claims: Vec<(Region, Region, usize)>,
    /// How many bridges have been begun across each span, so that one
    /// begun after another was left unfinished lies beside it.
    begun: Vec<(Region, Region, u8)>,
    spots: Option<[Spots; ARENA_TEAMS]>,
    /// Who is raiding each team, by seat; until when each team is left
    /// alone, having been raided; how many raids there have been, and how
    /// many of them on a team with a person on it; and who is at each
    /// team's anchor. The raids are steered a little, so that nobody is
    /// mobbed and nobody is out in the first minutes.
    raids: [Vec<usize>; ARENA_TEAMS],
    spared_until: [f32; ARENA_TEAMS],
    raided: (u32, u32),
    strikers: [Option<usize>; ARENA_TEAMS],
    /// What this game's choices of whom to raid are thrown with, so that
    /// who goes for whom first is not the same in every game.
    salt: usize,
    /// How each team means to open ([`Plan`]).
    plans: [Plan; ARENA_TEAMS],
    /// How everyone in the game has been moving, by seat, and when that
    /// was last looked at ([`Squad::watch`]).
    tracks: Vec<Track>,
    watched_at: f32,
    dice: Dice,
}

/// The other seat of `seat`'s team.
fn mate_of(seat: usize) -> usize {
    (seat + ARENA_TEAMS) % ARENA_SEATS
}

impl Squad {
    pub fn new(seed: u64, skill: Skill) -> Self {
        let mut dice = Dice::new(seed);
        Self {
            skill,
            bots: vec![None; ARENA_SEATS],
            claims: Vec::new(),
            begun: Vec::new(),
            spots: None,
            raids: Default::default(),
            spared_until: [0.0; ARENA_TEAMS],
            raided: (0, 0),
            strikers: [None; ARENA_TEAMS],
            salt: (dice.roll() * 11.0) as usize,
            plans: [Plan::PLAIN; ARENA_TEAMS],
            tracks: Vec::new(),
            watched_at: f32::NEG_INFINITY,
            dice,
        }
    }

    /// How `team` means to open.
    pub fn plan(&self, team: usize) -> Plan {
        self.plans[team % ARENA_TEAMS]
    }

    /// Has `team` open as `plan` has it: for trying one out.
    pub fn set_plan(&mut self, team: usize, plan: Plan) {
        self.plans[team % ARENA_TEAMS] = plan;
    }

    /// Has the bot in `seat` be who is called `name` ([`PERSONAS`]), and
    /// its team open as the one of its two that goes out would: for
    /// trying one out. Whether there is such a bot and such a one.
    pub fn cast(&mut self, seat: usize, name: &str) -> bool {
        let Some(persona) = PERSONAS.iter().find(|persona| persona.name == name) else {
            return false;
        };
        let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) else {
            return false;
        };
        let body = bot.body;
        *bot = Bot::new(seat, body, &mut self.dice, persona);
        self.settle(usize::from(arena_team(seat)));
        true
    }

    /// Works out again how `team` means to open: as the one of its bots
    /// that goes out would have it, the one of the two that wants to
    /// more.
    fn settle(&mut self, team: usize) {
        let lead = [team, team + ARENA_TEAMS]
            .into_iter()
            .filter_map(|seat| self.bots.get(seat).and_then(Option::as_ref).map(|bot| (bot.persona, seat)))
            .max_by(|a, b| a.0.drive.total_cmp(&b.0.drive).then(a.1.cmp(&b.1)));
        if let Some((persona, _)) = lead {
            self.plans[team % ARENA_TEAMS] = persona.plan(&mut self.dice);
        }
    }

    /// The bot in `seat` is raiding nobody any more, `now`: a team nobody
    /// is raiding any longer is left alone for a while.
    fn call_off(&mut self, seat: usize, now: f32) {
        for team in 0..ARENA_TEAMS {
            let before = self.raids[team].len();
            self.raids[team].retain(|&raider| raider != seat);
            if before > 0 && self.raids[team].is_empty() {
                self.spared_until[team] = now + self.skill.victim_rest;
            }
            if self.strikers[team] == Some(seat) {
                self.strikers[team] = None;
            }
        }
    }

    /// The computer takes `seat`, standing `at`: as someone of its own,
    /// by chance ([`PERSONAS`]).
    pub fn seat(&mut self, seat: usize, at: Vec3, yaw: f32) {
        if seat < ARENA_SEATS {
            let who = ((self.dice.roll() * PERSONAS.len() as f32) as usize).min(PERSONAS.len() - 1);
            self.bots[seat] = Some(Bot::new(seat, Body::standing(at, yaw), &mut self.dice, &PERSONAS[who]));
            self.settle(usize::from(arena_team(seat)));
        }
    }

    /// Nobody of the computer's is in `seat` any more.
    pub fn unseat(&mut self, seat: usize) {
        if let Some(bot) = self.bots.get_mut(seat) {
            *bot = None;
        }
        self.claims.retain(|claim| claim.2 != seat);
        self.call_off(seat, 0.0);
    }

    /// The bot in `seat`, if the computer has it.
    pub fn bot(&self, seat: usize) -> Option<&Bot> {
        self.bots.get(seat).and_then(Option::as_ref)
    }

    /// The bot in `seat` stands `at`: as the fight begins, and each time
    /// it stands again.
    pub fn stand(&mut self, seat: usize, at: Vec3, yaw: f32, now: f32) {
        let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) else {
            return;
        };
        bot.body = Body::standing(at, yaw);
        bot.health_was = 1.0;
        bot.foe = None;
        bot.in_reach_since = None;
        bot.drinking = None;
        bot.barred = None;
        bot.duel = None;
        bot.shun = None;
        bot.reel = None;
        bot.reel_until = f32::NEG_INFINITY;
        bot.face = None;
        bot.leaping = None;
        bot.cross = None;
        bot.alarm = None;
        if let Some(vigil) = &mut bot.vigil {
            vigil.stand = None;
        }
        // Another life, and perhaps other things for its buttons.
        bot.loadout = bot.persona.loadout(&mut self.dice);
        bot.planned = None;
        bot.goal = Goal::Idle;
        bot.goal_since = now;
        bot.path.clear();
        bot.path_to = None;
        bot.progress = (at.xz(), now);
        bot.think_at = now + self.dice.between(0.1, 0.6);
    }

    /// The bot in `seat` has died: the metres of bridge it had are gone
    /// with the rest of what it held, and the bridge it was laying is
    /// anyone's to finish.
    pub fn died(&mut self, seat: usize, now: f32) {
        let rest = self.skill.raid_rest;
        let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) else {
            return;
        };
        bot.metres = 0.0;
        bot.laying = None;
        // A builder it had sent off goes with it, its bridge left as far
        // as it got.
        bot.sent = None;
        bot.planned = None;
        bot.drinking = None;
        bot.foe = None;
        bot.duel = None;
        bot.leaping = None;
        bot.cross = None;
        bot.alarm = None;
        // Whoever it stood against it has not forgotten; where it stood
        // it has.
        if let Some(vigil) = &mut bot.vigil {
            vigil.stand = None;
        }
        bot.wants = [0; 4];
        // Dead on a raid, it stays home a while once it stands again.
        if matches!(bot.goal, Goal::Raid(_)) {
            bot.rest_until = now + rest;
        }
        bot.raiding_since = None;
        self.claims.retain(|claim| claim.2 != seat);
        self.call_off(seat, now);
    }

    /// The bot in `seat` has been knocked, from `from`. How long it is
    /// carried before it answers, and how well it steers when it does,
    /// are thrown again each time ([`Skill::reel`], [`Skill::grip`]).
    pub fn knock(&mut self, seat: usize, from: Vec2, running: bool) {
        let (reel, grip) = self.reeling(seat);
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.body.knock(from, running, 1.0);
            bot.reel = Some(reel);
            bot.grip = grip;
            // A jump it was making is whatever the knock makes of it.
            bot.leaping = None;
        }
    }

    /// How long a knock landing now carries the bot in `seat` before it
    /// answers, and how much of its pace it then steers by in the air:
    /// the better it is in a fight, the sooner and the surer
    /// ([`Temper::pvp`]): one that is as good as they come is never
    /// carried longer than the quickest third of what the rest may be,
    /// and never steers by less than most of its pace.
    fn reeling(&mut self, seat: usize) -> (f32, f32) {
        let skill = self.skill;
        let pvp = self.bot(seat).map_or(0.0, |bot| bot.temper.pvp);
        let slowest = skill.reel.0 + (skill.reel.1 - skill.reel.0) * (1.0 - 0.7 * pvp);
        let weakest = skill.grip.0 + (skill.grip.1 - skill.grip.0) * 0.7 * pvp;
        (self.dice.around(skill.reel.0, slowest), self.dice.around(weakest, skill.grip.1))
    }

    /// The bot in `seat` has been knocked by this much, across the ground
    /// and upward, as whoever runs the game has worked it out for itself:
    /// a game played offline, where a knock is the file's numbers and
    /// lands on the body the device draws. On top of half of whatever
    /// already carried it, as [`Body::knock`] has it.
    pub fn shoved(&mut self, seat: usize, across: Vec2, up: f32) {
        let (reel, grip) = self.reeling(seat);
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.body.shove = bot.body.shove * 0.5 + across;
            bot.body.lift = (bot.body.lift * 0.5 + up).min(up.max(0.0));
            bot.reel = Some(reel);
            bot.grip = grip;
            bot.leaping = None;
        }
    }

    /// The Bridge Builder the bot in `seat` meant to send off
    /// ([`Order::build`]) was not on its buttons after all, as whoever
    /// runs the game found: no bridge is laid for it. Nothing is free.
    pub fn unsend(&mut self, seat: usize) {
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.sent = None;
        }
    }

    /// The bot in `seat` has what it asked the shop for.
    pub fn bought(&mut self, seat: usize, ware: Ware, now: f32) {
        let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) else {
            return;
        };
        if ware == Ware::Bridge {
            bot.metres += BRIDGE_A_BUY;
        }
        bot.tries = 0;
        bot.think_at = now;
    }

    /// How many of each thing the bot in `seat` would take from a pad it
    /// stands on, and how much it matters to its team that it has them:
    /// whoever of a team wants them most is given them first. Nothing
    /// while a person of its team is at their pad, though it stands on it
    /// with them: what the pad makes is theirs (Hajun, 2026-11-12: "just
    /// let it stay on the generator even while ur collecting but the item
    /// collection goes to the player"; until then it stepped off as they
    /// came and back on as they left).
    pub fn wants(&self, seat: usize) -> (Purse, u8) {
        self.bot(seat)
            .filter(|bot| !bot.yields)
            .map_or(([0; 4], 0), |bot| (bot.wants, bot.urgency))
    }

    /// Where each team's bots stand on their island, worked out once.
    fn spots(&mut self, nav: &Nav) -> [Spots; ARENA_TEAMS] {
        if let Some(spots) = self.spots {
            return spots;
        }
        let grid = nav.grid();
        let mut spots = [Spots::default(); ARENA_TEAMS];
        for (team, spot) in spots.iter_mut().enumerate() {
            let island = ARENA_ISLANDS[team];
            let near = |at: Vec2| grid.nearest_clear(at, 4.0).unwrap_or(at);
            spot.heart = nav.heart(Region::Team(team as u8));
            spot.pad = near(ARENA_PADS[team]);
            spot.items = near(ARENA_SHOPS[team]);
            spot.upgrades = near(ARENA_UPGRADE_SHOPS[team]);
            // Between the anchor and the island's middle, a little to one
            // side: by the anchor, and clear of where its team arrives.
            let anchor = ARENA_ANCHORS[team];
            let inward = (island - anchor).normalize_or_zero();
            spot.guard = near(anchor + inward * 2.4 + Vec2::new(-inward.y, inward.x) * 1.6);
            spot.strike = near(anchor - inward * 1.9);
            let on_it = grid.nearest_clear(anchor, 4.0).is_some_and(|by| grid.region(by) == Some(Region::Team(team as u8)));
            spot.keep = if on_it { anchor } else { spot.heart };
        }
        self.spots = Some(spots);
        spots
    }

    /// One frame of the bot in `seat`: it thinks, if it is time; it acts
    /// on what it thought; its body is moved; and what the game is to
    /// carry out of it is given back.
    pub fn step(&mut self, seat: usize, view: &View, dt: f32) -> Order {
        let mut order = Order::default();
        self.watch(view);
        let Some(mut bot) = self.bots.get_mut(seat).and_then(Option::take) else {
            return order;
        };
        let me = view.seats.get(seat).copied().unwrap_or_default();
        if me.health < bot.health_was - 1e-3 {
            bot.hurt_at = view.elapsed;
            // How long this one smarts: not the same as the last.
            bot.sting = self.dice.around(STING.0, STING.1);
            // A drink is spilt by a blow.
            bot.drinking = None;
        }
        bot.health_was = me.health;
        let before = bot.body.at.xz();
        if !me.alive {
            self.bots[seat] = Some(bot);
            return order;
        }
        // What its pad makes is a person's while they are at it.
        let mate = view.seats.get(mate_of(seat)).copied().unwrap_or_default();
        let pad = ARENA_PADS[usize::from(arena_team(seat)) % ARENA_TEAMS];
        bot.yields = mate.person && mate.alive && mate.at.xz().distance(pad) < 3.5;
        // A knock that has landed since its last frame carries it for a
        // moment before it answers.
        if let Some(reel) = bot.reel.take() {
            bot.reel_until = view.elapsed + reel;
        }
        // A builder it sent off lays on, wherever the bot itself is.
        build_on(&mut bot, dt, &mut order);
        if view.elapsed >= bot.think_at {
            self.think(&mut bot, view);
        }
        // Its legs are its own in the air and with a knock on it, as
        // anyone's are.
        let walk = self.act(&mut bot, view, dt, &mut order);
        bot.body.step(view.nav, walk, dt);
        if let Some(face) = bot.face.take() {
            bot.body.yaw = face;
        }
        bot.speed = bot.body.at.xz().distance(before) / dt.max(1e-4);
        self.bots[seat] = Some(bot);
        order
    }

    /// Looks at where everyone is, once a frame, whichever bot's turn it
    /// is first: how fast and which way each is going, and how fast that
    /// is turning, each smoothed a little, from where they were when they
    /// last moved ([`Track`]). What a bot reads where someone will be
    /// from ([`Squad::foresee`]).
    fn watch(&mut self, view: &View) {
        let now = view.elapsed;
        if now <= self.watched_at {
            return;
        }
        self.watched_at = now;
        if self.tracks.len() < view.seats.len() {
            self.tracks.resize(view.seats.len(), Track::default());
        }
        for (track, seen) in self.tracks.iter_mut().zip(view.seats) {
            let at = seen.at.xz();
            let begun = Track {
                known: seen.alive,
                at,
                since: now,
                ..Track::default()
            };
            if !seen.alive || !track.known {
                *track = begun;
                continue;
            }
            let moved = at - track.at;
            let dt = now - track.since;
            if moved.length_squared() < 1e-6 {
                // Nothing new for a while is standing still: online a
                // person's place comes only while it changes.
                if dt > 0.25 {
                    *track = begun;
                }
                continue;
            }
            // Sent somewhere, not walked there: begun again from it.
            if dt < 1e-4 || moved.length() > 8.0 {
                *track = begun;
                continue;
            }
            let fresh = moved / dt;
            if track.last.length() > 2.0 && fresh.length() > 2.0 {
                let rate = (track.last.angle_to(fresh) / dt).clamp(-6.0, 6.0);
                track.turning += (rate - track.turning) * (1.0 - (-dt / 0.25).exp());
            } else {
                track.turning *= 0.5;
            }
            track.velocity += (fresh - track.velocity) * (1.0 - (-dt / 0.1).exp());
            track.last = fresh;
            track.at = at;
            track.since = now;
        }
    }

    /// Where whoever is in `seat`, now `at`, will be in `secs` seconds if
    /// they go on as they are going: on round the bend they are on, if
    /// they are turning, and no further than there is ground under them.
    /// `at` itself for someone standing, or not seen moving.
    fn foresee(&self, nav: &Nav, seat: usize, at: Vec2, secs: f32) -> Vec2 {
        let Some(track) = self.tracks.get(seat).filter(|track| track.known && track.velocity.length() >= 1.0) else {
            return at;
        };
        let mut velocity = track.velocity;
        let mut place = at;
        let mut left = secs;
        while left > 1e-3 {
            let step = left.min(0.05);
            velocity = Vec2::from_angle(track.turning * step).rotate(velocity);
            let next = place + velocity * step;
            if nav.ground(next).is_none() {
                break;
            }
            place = next;
            left -= step;
        }
        place
    }

    /// How fast whoever is in `seat` is going, in metres a second.
    fn speed_of(&self, seat: usize) -> f32 {
        self.tracks.get(seat).filter(|track| track.known).map_or(0.0, |track| track.velocity.length())
    }

    /// Where the bot heads to get at whoever is in `foe`, now `there`,
    /// going at `pace`: not where they are but where it can be on them,
    /// the soonest it could be within a punch of where they will be then
    /// ([`Squad::foresee`], no further ahead than [`FORESEE`]). Someone
    /// running round and round is cut off across the round, where running
    /// at where they are only ever follows them (Hajun, 2026-11-13:
    /// "instead of attacking i just run around in a circle, the AI always
    /// is one step behind so he just keeps following in a circle behind
    /// me in an endless loop"). How much of that it goes by is how good
    /// it is in a fight: all of it for the best, a little over half for
    /// the rest.
    fn meet(&self, nav: &Nav, bot: &Bot, foe: usize, there: Vec2, pace: f32) -> Vec2 {
        let here = bot.body.at.xz();
        if self.speed_of(foe) < 1.0 {
            return there;
        }
        let mut meet = there;
        let mut secs = 0.1;
        while secs <= FORESEE {
            meet = self.foresee(nav, foe, there, secs);
            if here.distance(meet) - RING <= pace.max(1.0) * secs {
                break;
            }
            secs += 0.1;
        }
        there.lerp(meet, 0.55 + 0.45 * bot.temper.pvp)
    }

    /// How far across a gap a bot going at `pace` jumps: what it crosses
    /// in the air, less what it keeps in hand ([`JUMP_SPARE`]).
    fn reach_of_a_jump(pace: f32) -> f32 {
        pace * JUMP_SECS - JUMP_SPARE
    }

    /// Whether a bot at a run can get from `a` to `b`: walking, or walking
    /// and jumping one hole in a bridge on the way ([`Nav::crossing`]).
    fn reachable(nav: &Nav, a: Vec2, b: Vec2) -> bool {
        nav.connected(a, b) || nav.crossing(a, b, Self::reach_of_a_jump(RUN_SPEED)).is_some()
    }

    /// What a stick of dynamite makes of the way the bot was going to
    /// walk ([`Stick`]). In the way of one, a plank of a bridge under it
    /// and the stick near enough to take it, or the stick another team's
    /// and near enough to hurt, it runs for the nearest ground clear of
    /// the blast, once it has woken up to it, which the better ones do
    /// sooner and anyone in a fight later ([`STICK_WAKES`]): the worst of
    /// them are still in it now and then as it blows. Out of it, it
    /// keeps out until it has blown, whatever it was doing there: it does
    /// not walk back in after whoever it was fighting. Walking toward one
    /// that lies on a bridge, it waits short of it. The three times a
    /// stick shines are there for this. A stick of its own team's on the
    /// land is nothing to it.
    fn mind_sticks(&mut self, bot: &mut Bot, view: &View, walk: Vec2) -> Vec2 {
        if view.sticks.is_empty() {
            bot.alarm = None;
            return walk;
        }
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let team = arena_team(bot.seat);
        let out = out_over_the_void(nav, here);
        let clear = DYNAMITE_RADIUS + BODY + CLEAR_OF_BLAST;
        // The nearest it has to mind: one that would take the floor from
        // under it or hurt it where it stands, or one lying on a bridge
        // that it is walking at; and one it has in mind already it keeps
        // in mind until it has blown, from as far off as it waits for one.
        // (One it is walking at it takes note of from further off than it
        // waits at, so that it has woken up to it by the time it is there.)
        let heading = walk.normalize_or_zero();
        let in_mind = bot.alarm.map(|(at, _)| at);
        let minded = view
            .sticks
            .iter()
            .filter(|stick| {
                let away = stick.at.distance(here);
                let harms = stick.team != team || out;
                let on_a_bridge = nav.on_plank(stick.at) && nav.grid().height(stick.at).is_none();
                let walking_at = heading.dot((stick.at - here).normalize_or_zero()) > 0.3;
                let known = in_mind.is_some_and(|at| at.distance(stick.at) < 0.2);
                (away < clear && harms)
                    || (away < clear + SHORT_OF_BLAST + NOTICED_FROM && on_a_bridge && walking_at)
                    || (away < clear + SHORT_OF_BLAST && harms && known)
            })
            .min_by(|a, b| a.at.distance_squared(here).total_cmp(&b.at.distance_squared(here)))
            .copied();
        let Some(stick) = minded else {
            bot.alarm = None;
            return walk;
        };
        // It wakes up to it a while after it is thrown.
        let woken = match bot.alarm {
            Some((at, when)) if at.distance(stick.at) < 0.2 => when,
            _ => {
                let pvp = bot.temper.pvp;
                let busy = if bot.duel.is_some() { STICK_BUSY } else { 0.0 };
                let soonest = STICK_WAKES.0 - STICK_WAKES_SOONER.0 * pvp + busy;
                let latest = STICK_WAKES.1 - STICK_WAKES_SOONER.1 * pvp + busy;
                let share = if stick.at.distance(here) >= clear { STICK_AHEAD } else { 1.0 };
                let when = now + self.dice.around(soonest, latest) * share;
                bot.alarm = Some((stick.at, when));
                when
            }
        };
        if now < woken {
            return walk;
        }
        let pace = RUN_SPEED * if bot.drinking.is_some() { DRINK_SLOW } else { 1.0 };
        let away = stick.at.distance(here);
        if away >= clear {
            // Clear of it: no nearer it than a little short of the blast
            // until it has blown, whether it lies on the bridge ahead or
            // where the fight was; any other way it goes as it would.
            let nearer = heading.dot((stick.at - here).normalize_or_zero()) > 0.0;
            return if away < clear + SHORT_OF_BLAST && nearer { Vec2::ZERO } else { walk };
        }
        // Out of it: to the nearest place clear of it that can be walked
        // straight to, or failing that straight away from it for as far
        // as there is ground.
        let ring = clear + 0.3;
        let best = (0..16)
            .map(|k| stick.at + Vec2::from_angle(k as f32 * std::f32::consts::TAU / 16.0) * ring)
            .filter(|&to| nav.walkable_at(to) && nav.line(here, to))
            .min_by(|a, b| a.distance_squared(here).total_cmp(&b.distance_squared(here)));
        bot.path.clear();
        bot.path_to = None;
        match best {
            Some(to) => (to - here).normalize_or_zero() * pace,
            None => (here - stick.at).normalize_or(-heading) * pace,
        }
    }

    /// The slow layer: what the bot might be doing, each with a score,
    /// the best of them taken, the one it is on getting a bonus.
    fn think(&mut self, bot: &mut Bot, view: &View) {
        let skill = self.skill;
        let now = view.elapsed;
        bot.think_at = now + self.dice.between(skill.think_every.0, skill.think_every.1);
        let (seat, nav) = (bot.seat, view.nav);
        let team = usize::from(arena_team(seat));
        let home = Region::Team(team as u8);
        let spots = self.spots(nav)[team];
        let me = view.seats[seat];
        let here = bot.body.at.xz();
        let mate = mate_of(seat);
        let mate_seen = view.seats.get(mate).copied().unwrap_or_default();
        let with_person = mate_seen.person && mate_seen.in_play;
        let mate_bot = self.bots[mate].as_ref().filter(|_| mate_seen.in_play);
        // Whom its teammate is raiding, if it is one of the computer's and
        // on a raid.
        let mate_raids = mate_bot.and_then(|mate| match mate.goal {
            Goal::Raid(victim) => Some(usize::from(victim)),
            _ => None,
        });
        // Of a team's two, the one that wants to go out more goes, and the
        // other keeps the base: beside a person, the bot keeps.
        let mate_drive = mate_bot.map(|mate| mate.persona.drive);
        let mate_bot = mate_bot.is_some();
        let mine = bot.persona.drive;
        bot.role = match mate_drive {
            Some(theirs) if !with_person && (mine > theirs || (mine == theirs && seat > mate)) => Role::Runner,
            _ => Role::Keeper,
        };
        // A bot with nobody beside it does both parts.
        let alone = !with_person && !mate_bot;
        let (keeper, runner) = (bot.role == Role::Keeper, bot.role == Role::Runner || alone);
        let purse = view.purses.get(seat).copied().unwrap_or_default();
        let kit = view.outfits.kit(seat);
        let upgrades = view.outfits.team(seat);
        // Walking, or over a hole in a bridge that it can jump.
        let can_go_home = Self::reachable(nav, here, spots.heart);
        let on_home = nav.grid().region(here) == Some(home) && !nav.on_plank(here);
        let foes: Vec<Seen> = view
            .seats
            .iter()
            .enumerate()
            .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
            .map(|(_, seen)| *seen)
            .collect();
        let nearest_foe = foes.iter().map(|foe| foe.at.xz().distance(here)).fold(f32::INFINITY, f32::min);

        let mut offers: Vec<(f32, Goal)> = vec![(0.05, Goal::Idle)];

        // How near it lets people come, for the next while: nobody keeps
        // to the same line all day ([`KEEN`]).
        if now >= bot.keen_until {
            bot.keen = self.dice.around(KEEN.0, KEEN.1);
            bot.keen_until = now + self.dice.around(KEEN_LASTS.0, KEEN_LASTS.1);
        }

        // Hurt, with someone after it: away, until it has its health back.
        let pressed = nearest_foe < 10.0 || now - bot.hurt_at < bot.sting;
        let mortal = view.anchors[team] <= 0.0;
        // The bolder, the longer it stays in; and a little sooner or later
        // each time than the last ([`NERVE`]).
        let give_way = (if mortal { skill.careful_below } else { skill.retreat_below }) + (1.0 - bot.temper.bold) * 0.15 + bot.nerve;
        if me.health < give_way && pressed {
            offers.push((0.95, Goal::Retreat));
        } else if bot.goal == Goal::Retreat && me.health < skill.recovered + bot.nerve && nearest_foe < 16.0 {
            offers.push((0.9, Goal::Retreat));
        }

        // Someone of another team on its island, standing on the land of
        // it: home, to see them off.
        let on_its_land = |at: Vec3| {
            let across = at.xz();
            nav.grid().region(across) == Some(home) && !nav.on_plank(across) && at.y > -1.0
        };
        let intruder = foes.iter().any(|foe| on_its_land(foe.at));
        if intruder && can_go_home {
            offers.push((if on_home || keeper { 0.88 } else { 0.6 }, Goal::Defend));
        }

        // Whoever is at its base: on the land of its island, or by its
        // anchor as that loses health. It has them in mind from then on,
        // and once they have only stepped back it looks after them, or
        // stands its ground ([`Vigil`]).
        let anchor_left = view.anchors[team];
        let struck = anchor_left < bot.anchor_was - 1e-3;
        bot.anchor_was = anchor_left;
        let caller = view
            .seats
            .iter()
            .enumerate()
            .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
            .filter(|(_, seen)| on_its_land(seen.at) || (struck && seen.at.xz().distance(spots.keep) < 5.0))
            .min_by(|a, b| a.1.at.xz().distance_squared(spots.keep).total_cmp(&b.1.at.xz().distance_squared(spots.keep)))
            .map(|(other, _)| other);
        self.keep_vigil(bot, view, caller, on_home, &spots);
        if bot.vigil.is_some_and(|vigil| vigil.stand.is_some()) && can_go_home {
            offers.push((GUARD_SCORE, Goal::Guard));
        }

        // The bridge it means to lay next, and the metres it is short of
        // for it.
        let span = self.next_span(bot, view, runner, with_person);
        // With none it has to build, one worth a Bridge Builder, for one
        // that carries them: bought for it when it has the amethyst, and
        // laid by the builder alone. Not beside a person, whose base it
        // keeps.
        let holding = kit.held(Utility::Builder) > 0;
        let carries_builders = bot.loadout.iter().any(|(kind, _)| *kind == Utility::Builder);
        let spare = (span.is_none()
            && runner
            && !with_person
            && bot.laying.is_none()
            && bot.sent.is_none()
            && (holding || carries_builders))
            .then(|| self.spare_span(bot, view))
            .flatten();
        let span = span.or(spare.filter(|_| holding));
        bot.planned = span;
        // A Bridge Builder on one of its buttons lays a new one for it,
        // straight, if it is no longer than a builder goes: no metres of
        // its own.
        let builder = bot.laying.is_none()
            && holding
            && span.is_some_and(|(_, _, span)| span.length() <= BUILDER_METRES - 2.0);
        // What is left of one it is part way along; the whole of a new
        // one, and a little over, for the bends it will have.
        let short = span.filter(|_| !builder).map_or(0.0, |(_, _, span)| {
            let laid = bot.laying.map_or(0, |laying| laying.next.saturating_sub(1));
            let bends = if bot.laying.is_some() { 1.0 } else { MOST_BENT };
            let left = span.length() * bends * (1.0 - laid as f32 / span.planks() as f32);
            (left + 0.5 - bot.metres).max(0.0)
        });
        if let Some((from, to, span)) = span
            && short == 0.0
            && Self::reachable(nav, here, span.from)
        {
            offers.push((0.78, Goal::Lay(from, to)));
        }

        // What it would buy, in the order it would: the first it can pay
        // for now it goes to the shop for; what it cannot yet, it saves
        // for at its pad.
        // A team that means to build later arms itself first: both of its
        // two buy as the one that keeps the base does.
        let building_later = self.plans[team].opening == Opening::Late && now < self.plans[team].bridge_from;
        let buys_as = if building_later { Role::Keeper } else { bot.role };
        let list = shopping(
            bot.persona,
            &bot.loadout,
            buys_as,
            alone,
            now,
            &purse,
            &kit,
            &upgrades,
            short,
            bot.laying.is_some(),
            spare.is_some() && !holding,
        );
        let payable = |ware: Ware| price(ware, &kit, &upgrades).is_some_and(|(item, cost)| purse[item.index()] >= cost);
        if let Some(&ware) = list.iter().find(|&&ware| payable(ware)) {
            let score = if on_home {
                0.72
            } else if can_go_home {
                0.55
            } else {
                0.0
            };
            offers.push((score, Goal::Shop(ware)));
        }
        // What it would take from a pad: the price of the first few
        // things on its list, less what it holds.
        let mut wants = [0u16; 4];
        for &ware in list.iter().take(3) {
            if let Some((item, cost)) = price(ware, &kit, &upgrades) {
                wants[item.index()] = wants[item.index()].saturating_add(cost);
            }
        }
        for (item, wanted) in wants.iter_mut().enumerate() {
            *wanted = wanted.saturating_sub(purse[item]);
        }
        // Its team's first bridge comes before anything the keeper wants
        // silver for, from when the team means to build it.
        let plan = self.plans[team];
        let unbridged = !nav.joined(home, Region::Middle)
            && !(plan.opening == Opening::Sapphire
                && beside(team as u8).iter().any(|&island| nav.joined(home, Region::Sapphire(island))));
        let bridging = unbridged && now >= plan.bridge_from && now < plan.bridge_from + 100.0;
        if keeper && !alone && !with_person && bridging {
            wants[Item::Silver.index()] = 0;
        }
        bot.urgency = if short > 0.0 {
            3
        } else if kit.sword == 0 {
            2
        } else {
            1
        };
        let saving = wants[Item::Silver.index()] > 0 || wants[Item::Gold.index()] > 0;
        // On its pad for them. **Beside a person it is no keeper of the
        // pad**: it comes for what lies there once enough of it does,
        // and they are not there for it themselves, picks it up, and is
        // off again, to the shop, a bridge, another pad or a raid, and not
        // back for a while (since 2026-11-13; Hajun: "the bot stays on the
        // iron/gold generator for toooo long ... just come back to pick up
        // resource and leave again quickly not stay"; until then it stood
        // on the pad for as long as it was saving for anything, which was
        // nearly always).
        if saving && can_go_home {
            let stock = view.mint.held(team) + view.mint.held(ARENA_TEAMS + team);
            if with_person {
                let worth = usize::from(wants[Item::Silver.index()] + wants[Item::Gold.index()]).clamp(1, 6);
                let there = bot.goal == Goal::Collect;
                if !bot.yields && now >= bot.pad_again && (there || stock >= worth) {
                    offers.push((0.56, Goal::Collect));
                }
            } else {
                offers.push((if stock > 0 { 0.52 } else { 0.48 }, Goal::Collect));
            }
        }

        // Out to a pad for what lies on it: the amethyst in the middle,
        // and the sapphire on the smaller islands. The keeper stays home,
        // unless it is alone, or its teammate is a person: then it goes
        // out for them as the one that goes out does, from early on and
        // with nobody of another team near its island.
        // Its own: the amethyst pad toward its island, and the two smaller
        // islands beside it. The rest are other teams' to go to, until
        // late.
        let quiet = nearest_foe > 25.0;
        let late = now >= skill.keeper_raids_from;
        if runner || (with_person && quiet && now > 45.0) {
            let base = 0.5;
            for (pad, &at) in AMETHYST_PADS.iter().enumerate() {
                let stock = view.mint.held(12 + pad);
                if stock > 0 && (pad == team || late) && Self::reachable(nav, here, at) {
                    let score = base + 0.06 + 0.08 * stock.min(2) as f32 - here.distance(at) / 300.0;
                    offers.push((score, Goal::Fetch(at, Item::Amethyst)));
                }
            }
            let beside = beside(team as u8);
            for (pad, &at) in SAPPHIRE_PADS.iter().enumerate() {
                let stock = view.mint.held(8 + pad);
                if stock > 0 && (beside.contains(&(pad as u8)) || late) && Self::reachable(nav, here, at) {
                    let score = base + 0.07 * stock.min(4) as f32 - here.distance(at) / 300.0;
                    offers.push((score, Goal::Fetch(at, Item::Sapphire)));
                }
            }
        }

        // The anchors gone, everyone goes for everyone: whoever of another
        // team is nearest that it can get at.
        let in_play = |team: usize| (0..ARENA_SEATS).any(|other| usize::from(arena_team(other)) == team && view.seats.get(other).is_some_and(|seen| seen.in_play));
        if now >= skill.hunt_from && foes.iter().any(|foe| Self::reachable(nav, here, foe.at.xz())) {
            offers.push((0.8, Goal::Hunt));
        }

        // A raid: to another team's island, armed and whole, when its
        // turn comes. The one that goes out goes; the keeper with it,
        // late on; and a person's teammate by itself, a little after the
        // rest have begun and with nobody of another team near its own
        // island (since 2026-11-13, Hajun: "make it go out more ... or
        // raiding an island more"; never by itself until then).
        // Sooner or later than the rest, by who it is.
        let rested = now >= bot.rest_until && me.health >= 0.7 && kit.sword > 0;
        let goes_out = runner || (with_person && quiet && now >= skill.raid_from + bot.persona.eager.max(0.0) + 20.0);
        if now >= skill.raid_from + bot.persona.eager && now < skill.hunt_from && rested {
            let others = (0..ARENA_TEAMS).filter(|&other| other != team && in_play(other)).count();
            for victim in (0..ARENA_TEAMS).filter(|&victim| victim != team && in_play(victim)) {
                let mine = matches!(bot.goal, Goal::Raid(on) if usize::from(on) == victim);
                // Along with its teammate, unless it is one for staying
                // at home; or, the one that goes out, by its own choice.
                let along = mate_raids == Some(victim) && (runner || (now >= skill.keeper_raids_from && bot.persona.raids > -0.05));
                if !mine && !along && !goes_out {
                    continue;
                }
                // One team's raid on a team at a time, a rest after each,
                // and a team with a person on it no more than its share.
                let theirs = self.raids[victim].iter().all(|&raider| usize::from(arena_team(raider)) == team);
                let people = (0..ARENA_SEATS).any(|other| usize::from(arena_team(other)) == victim && view.seats.get(other).is_some_and(|seen| seen.person && seen.in_play));
                let share = (self.raided.1 + 1) as f32 <= skill.people_share * (self.raided.0 + 1) as f32 + 1.0;
                let allowed = mine || along || (theirs && now >= self.spared_until[victim] && (!people || others < 2 || share));
                let their_heart = self.spots(nav)[victim].heart;
                if !allowed || !Self::reachable(nav, here, their_heart) {
                    continue;
                }
                let ramp = ((now - skill.raid_from) / (skill.hunt_from - skill.raid_from)).clamp(0.0, 1.0);
                // Which of them, by chance as much as by how far: thrown
                // once a minute for each, not again at every thought, and
                // differently in every game (until 2026-11-12 the same in
                // all of them: the first raids always fell on the same
                // two islands).
                let chance = ((seat * 7 + victim * 13 + (now / 60.0) as usize * 5 + self.salt) % 11) as f32 / 11.0;
                let score = 0.6 + 0.14 * ramp - here.distance(their_heart) / 2000.0 + chance * 0.05 + bot.persona.raids;
                offers.push((score, Goal::Raid(victim as u8)));
            }
        }

        // The best of them, what it is on having a bonus, and what it has
        // given up on left out for a while.
        if bot.barred.is_some_and(|(_, until)| now >= until) {
            bot.barred = None;
        }
        let best = offers
            .into_iter()
            .filter(|(_, goal)| !bot.barred.is_some_and(|(barred, _)| barred.alike(goal)))
            .map(|(score, goal)| (if goal == bot.goal { score + skill.sticky } else { score }, goal))
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(Goal::Idle, |(_, goal)| goal);
        // One bot to a span.
        self.claims.retain(|claim| claim.2 != seat);
        if let Goal::Lay(from, to) = best {
            self.claims.push((from, to, seat));
        }
        // The raids' ledger: who is on whom.
        if let Goal::Raid(victim) = best {
            let victim = usize::from(victim);
            if !self.raids[victim].contains(&seat) {
                if self.raids[victim].is_empty() {
                    let people = (0..ARENA_SEATS).any(|other| usize::from(arena_team(other)) == victim && view.seats.get(other).is_some_and(|seen| seen.person && seen.in_play));
                    self.raided = (self.raided.0 + 1, self.raided.1 + u32::from(people));
                }
                // Off whoever else it was on.
                self.call_off(seat, now);
                self.raids[victim].push(seat);
            }
        } else if matches!(bot.goal, Goal::Raid(_)) {
            // Back from one: home for a while.
            self.call_off(seat, now);
            bot.rest_until = bot.rest_until.max(now + skill.raid_rest);
            bot.raiding_since = None;
        }
        // What it takes from a pad: what it is saving for at its own,
        // whatever lies on one it has gone out to, and on a raid what
        // its next purchases need of what the raided have.
        bot.wants = match best {
            Goal::Fetch(_, item) => {
                let mut all = [0; 4];
                all[item.index()] = 9;
                all
            }
            Goal::Raid(_) => [20u16.saturating_sub(purse[0]), 7u16.saturating_sub(purse[1]), 0, 0],
            _ => wants,
        };
        // Over a fright: the next one finds it a little braver, or less.
        if bot.goal == Goal::Retreat && best != Goal::Retreat {
            bot.nerve = self.dice.around(NERVE.0, NERVE.1);
        }
        bot.set_goal(best, now);
    }

    /// Keeps the bot's mind of whoever comes at its base ([`Vigil`]), as
    /// often as it thinks. `caller` is whoever is at it now, if anyone;
    /// `at_home` whether the bot is on its island. With someone at it: it
    /// has them in mind, and counts their coming if they had been away.
    /// With them stepped back: it takes its stand toward them, if it is
    /// home or was seeing them off, for a look or, having had enough, for
    /// good; keeps between them and what its team keeps as they move; and
    /// ends it once they are gone a good way off for a while, or it has
    /// been long enough, when it either goes out at them or lets them be.
    /// Left alone long enough, it forgets them, and how patient it is
    /// with the next is thrown again.
    fn keep_vigil(&mut self, bot: &mut Bot, view: &View, caller: Option<usize>, at_home: bool, spots: &Spots) {
        let (nav, now) = (view.nav, view.elapsed);
        let home = Region::Team(arena_team(bot.seat));
        if let Some(foe) = caller {
            let mut vigil = bot.vigil.unwrap_or(Vigil {
                foe,
                comings: 0,
                at_it: false,
                about: now,
                forget: self.dice.around(VIGIL_FORGET.0, VIGIL_FORGET.1),
                stand: None,
            });
            if !vigil.at_it {
                vigil.comings = vigil.comings.saturating_add(1);
            }
            vigil.foe = foe;
            vigil.at_it = true;
            vigil.about = now;
            vigil.stand = None;
            bot.vigil = Some(vigil);
            return;
        }
        let Some(mut vigil) = bot.vigil else {
            return;
        };
        let them = view.seats.get(vigil.foe).copied().unwrap_or_default();
        let there = them.at.xz();
        let seen = them.alive && !them.unseen;
        let here = bot.body.at.xz();
        if vigil.at_it {
            // They have just stepped back from it.
            vigil.at_it = false;
            if at_home || bot.goal == Goal::Defend {
                let resolved = vigil.comings >= bot.patience;
                let lasts = if resolved { VIGIL_STAND } else { VIGIL_LOOK };
                vigil.stand = Some(Stand {
                    spot: here,
                    until: now + self.dice.around(lasts.0, lasts.1),
                    resolved,
                    watch: self.dice.around(VIGIL_WATCH.0, VIGIL_WATCH.1) * bot.temper.bold,
                    gone: self.dice.around(VIGIL_GONE.0, VIGIL_GONE.1),
                    calm: self.dice.around(VIGIL_CALM.0, VIGIL_CALM.1),
                    gone_since: None,
                    off: self.dice.around(VIGIL_OFF.0, VIGIL_OFF.1),
                    inside: self.dice.between(VIGIL_INSIDE.0, VIGIL_INSIDE.1) * bot.temper.minds.clamp(0.6, 1.5),
                });
            }
        }
        let away = there.distance(spots.heart);
        if seen && away < VIGIL_GONE.1 + 8.0 {
            vigil.about = now;
        }
        if let Some(mut stand) = vigil.stand {
            let gone = !seen || away > stand.gone;
            let gone_for = if gone { now - *stand.gone_since.get_or_insert(now) } else { 0.0 };
            if !gone {
                stand.gone_since = None;
            }
            let over = now >= stand.until || (stand.resolved && gone_for >= stand.calm);
            vigil.stand = if !over {
                // Between them and what it keeps, as they move; the two of
                // a team a little apart.
                let side = if bot.seat < ARENA_TEAMS { 0.9 } else { -0.9 };
                if let Some(spot) = guard_spot(nav, home, spots.keep, there, stand.off, stand.inside) {
                    let across = (spots.keep - there).normalize_or_zero().perp() * side;
                    let apart = spot + across;
                    let stands = nav.grid().region(apart) == Some(home) && nav.grid().height(apart).is_some() && nav.walkable_at(apart);
                    stand.spot = if stands { apart } else { spot };
                } else if stand.spot == here {
                    stand.spot = spots.guard;
                }
                Some(stand)
            } else if stand.resolved && !gone && self.dice.roll() < (0.25 + 0.3 * bot.temper.pvp + 0.35 * (bot.temper.bold - 1.0)).clamp(0.1, 0.85) {
                // It has stood there long enough with them still about:
                // out at them, for as far as they are and a while.
                stand.watch = stand.spot.distance(there) + 6.0;
                stand.until = now + self.dice.around(6.0, 14.0);
                Some(stand)
            } else {
                // Or it lets them be: back to what it was doing, with
                // them still in mind.
                bot.think_at = bot.think_at.min(now + 0.2);
                None
            };
        }
        // Left alone long enough, it has forgotten them.
        if vigil.stand.is_none() && now - vigil.about > vigil.forget {
            bot.vigil = None;
            bot.patience = patience_of(bot.persona, &mut self.dice);
            return;
        }
        bot.vigil = Some(vigil);
    }

    /// The bridge the bot would lay next, if any, as its team means to
    /// open ([`Plan`]): from its island to the middle, if nothing joins
    /// them yet, or first, for a team that opens so, from its island
    /// straight to a smaller island beside it, with the middle left for
    /// as long as it leaves it; then from the middle to one of the two
    /// smaller islands beside its own, if neither is joined to its
    /// island; none before its team means to build. The one that goes out
    /// lays them; beside a person the bot does, once they have had the
    /// time to. Not one a teammate is laying. The line of it between the
    /// two lands is a little to one side or the other, and straight: it
    /// is bent as it is begun ([`Squad::lay`]).
    fn next_span(&mut self, bot: &Bot, view: &View, runner: bool, with_person: bool) -> Option<(Region, Region, Span)> {
        let (nav, now) = (view.nav, view.elapsed);
        let team = arena_team(bot.seat);
        let home = Region::Team(team);
        // The one it was laying, to be finished.
        if let Some(laying) = bot.laying {
            return Some((laying.from, laying.to, laying.span));
        }
        let plan = self.plans[usize::from(team) % ARENA_TEAMS];
        let beside = beside(team);
        let own = Region::Sapphire(beside[usize::from(plan.side) % 2]);
        let to_middle = nav.joined(home, Region::Middle);
        let to_sapphire = beside.iter().any(|&island| nav.joined(home, Region::Sapphire(island)));
        // A hole blown in its team's way to the middle is either of the
        // team's to mend, whichever of them goes out and whoever is
        // beside it, from whichever side of it they stand on: a few
        // planks, where a bridge is the one that goes out's to lay.
        if !to_middle && let Some(mend) = self.mend_for(bot, view, (home, Region::Middle)) {
            return Some(mend);
        }
        let wanted = if with_person {
            // Beside a person: the bridge to the middle once they have
            // had the time to build it, unless they are out over the void
            // building one now; and a little later one to a smaller
            // island. (After 40 s and two minutes until 2026-11-13, when
            // Hajun asked for the bot beside them to be out and about
            // more: "bridging or raiding an island more".)
            let mate = view.seats.get(mate_of(bot.seat)).copied().unwrap_or_default();
            let theirs = mate.alive && out_over_the_void(nav, mate.at.xz());
            if !to_middle && now > 30.0 && !theirs {
                (home, Region::Middle)
            } else if to_middle && !to_sapphire && now > 75.0 {
                (Region::Middle, own)
            } else {
                return None;
            }
        } else {
            if !runner || now < plan.bridge_from {
                return None;
            }
            let direct = (plan.opening == Opening::Sapphire && !to_sapphire && !to_middle).then_some((home, own));
            let middle = (!to_middle && now >= plan.middle_from).then_some((home, Region::Middle));
            let sapphire = (to_middle && !to_sapphire).then_some((Region::Middle, own));
            // Or, once the raids are on, to the island of a team that has
            // built no way to it: nobody sits a game out behind the void.
            let stranger = (to_middle && now >= self.skill.raid_from + 90.0)
                .then(|| {
                    (0..ARENA_TEAMS as u8).find(|&other| {
                        other != team
                            && !nav.joined(Region::Team(other), Region::Middle)
                            && (0..ARENA_SEATS)
                                .any(|seat| arena_team(seat) == other && view.seats.get(seat).is_some_and(|seen| seen.in_play))
                    })
                })
                .flatten()
                .map(|other| (Region::Middle, Region::Team(other)));
            direct.or(middle).or(sapphire).or(stranger)?
        };
        self.span_for(bot, view, wanted)
    }

    /// The bridge from the one land of `wanted` to the other, for `bot` to
    /// lay: none if a teammate is laying it, or a Bridge Builder is, its
    /// own or anyone's. Beside whatever was begun there and left; and not
    /// on the very line between the two lands' middles, but a little to
    /// one side of it or the other, by the bot and by the bridge.
    fn span_for(&mut self, bot: &Bot, view: &View, wanted: (Region, Region)) -> Option<(Region, Region, Span)> {
        let nav = view.nav;
        if self.claims.iter().any(|&(from, to, by)| (from, to) == wanted && by != bot.seat) {
            return None;
        }
        let being_built = |sent: Option<Sent>| sent.is_some_and(|sent| (sent.from, sent.to) == wanted);
        if being_built(bot.sent) || self.bots.iter().flatten().any(|other| being_built(other.sent)) {
            return None;
        }
        // A bridge there with one hole blown in it is mended, not built
        // again beside itself.
        if let Some(mend) = self.mend_for(bot, view, wanted) {
            return Some(mend);
        }
        let tried = self.begun.iter().find(|begun| (begun.0, begun.1) == wanted).map_or(0, |begun| begun.2);
        let off = ((bot.seat * 31 + usize::from(bot.chains) * 17 + self.salt * 7) % 13) as f32 / 6.0 - 1.0;
        let aside = [0.0, 2.9, -2.9][usize::from(tried) % 3] + off * 1.4;
        let span = nav.grid().span(wanted.0, wanted.1, aside).or_else(|| nav.grid().span(wanted.0, wanted.1, 0.0))?;
        // From its far end, for a bot that stands on that side of the
        // water and cannot get round to this one.
        let here = bot.body.at.xz();
        if !Self::reachable(nav, here, span.from) && Self::reachable(nav, here, span.to) {
            let back = nav.grid().span(wanted.1, wanted.0, -aside).or_else(|| nav.grid().span(wanted.1, wanted.0, 0.0))?;
            return Some((wanted.0, wanted.1, back));
        }
        Some((wanted.0, wanted.1, span))
    }

    /// The planks that would mend the way between the two lands of
    /// `wanted`, for `bot` to lay: across the one hole a blast has left
    /// in a bridge between them ([`Nav::crossing`]), from a little inside
    /// what stands on the side the bot can get to, to a little inside
    /// what stands on the other ([`MEND_INSET`]), straight. None if they
    /// are joined, if no one hole lies between them (a bridge blown in
    /// two places has a stretch of itself standing in the middle of
    /// nowhere: a new bridge, then), if the bot can get to neither side
    /// of it, or if a teammate is at it already.
    fn mend_for(&mut self, bot: &Bot, view: &View, wanted: (Region, Region)) -> Option<(Region, Region, Span)> {
        let nav = view.nav;
        if nav.gaps().is_empty() || self.claims.iter().any(|&(from, to, by)| (from, to) == wanted && by != bot.seat) {
            return None;
        }
        let here = bot.body.at.xz();
        let gap = nav.crossing(nav.heart(wanted.0), nav.heart(wanted.1), f32::INFINITY)?;
        let gap = [gap, gap.turned()].into_iter().find(|gap| nav.connected(here, gap.a_in))?;
        let inside = |end: Vec2, step_back: Vec2| end + (step_back - end).normalize_or_zero() * MEND_INSET;
        Some((wanted.0, wanted.1, Span::straight(inside(gap.a, gap.a_in), inside(gap.b, gap.b_in))))
    }

    /// A bridge worth a Bridge Builder, to a bot with none it has to
    /// build ([`Squad::next_span`]): one its team could do without, and
    /// which it lays only by sending a builder, never with metres of its
    /// own. From the middle to the other of the two smaller islands
    /// beside its own, for twice the sapphire; or, once the raids are on,
    /// **a side door**: from a smaller island its team has reached
    /// straight onto the island of the other team that island lies
    /// beside, to come at them from where no bridge of theirs lands. One
    /// that likes a raid goes for the side door first, and the rest for
    /// the sapphire. No side door twice over the same water.
    fn spare_span(&mut self, bot: &Bot, view: &View) -> Option<(Region, Region, Span)> {
        let (nav, now) = (view.nav, view.elapsed);
        let team = arena_team(bot.seat);
        let home = Region::Team(team);
        let own = beside(team);
        let reached = |island: u8| nav.joined(home, Region::Sapphire(island));
        let in_play = |other: u8| {
            (0..ARENA_SEATS).any(|seat| arena_team(seat) == other && view.seats.get(seat).is_some_and(|seen| seen.in_play))
        };
        let second = nav
            .joined(home, Region::Middle)
            .then(|| own.into_iter().find(|&island| !reached(island)))
            .flatten()
            .map(|island| (Region::Middle, Region::Sapphire(island)));
        let side_door = (now >= self.skill.raid_from)
            .then(|| {
                own.into_iter().filter(|&island| reached(island)).find_map(|island| {
                    let from = Region::Sapphire(island);
                    (0..ARENA_TEAMS as u8)
                        .find(|&other| {
                            other != team
                                && in_play(other)
                                && beside(other).contains(&island)
                                && !self.begun.iter().any(|begun| (begun.0, begun.1) == (from, Region::Team(other)))
                        })
                        .map(|other| (from, Region::Team(other)))
                })
            })
            .flatten();
        let wanted = if bot.persona.raids > 0.0 { side_door.or(second) } else { second.or(side_door) }?;
        self.span_for(bot, view, wanted).filter(|(_, _, span)| span.length() <= BUILDER_METRES - 2.0)
    }

    /// The fast layer: which way the bot walks this frame, in metres a
    /// second across the ground, and what it does. A knock that has just
    /// landed carries it for a moment and no more ([`Skill::reel`]); then,
    /// in the air over the void, it steers back for the nearest ground,
    /// and, carried toward an edge, against what carries it, each as hard
    /// as it steers this time ([`Skill::grip`]); otherwise it goes about
    /// what it was doing ([`Squad::intend`]), fighting whoever it fights
    /// all the while, and now and then, running, it jumps.
    fn act(&mut self, bot: &mut Bot, view: &View, dt: f32, order: &mut Order) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        bot.face = None;
        bot.body.leap = false;
        bot.awaits = false;
        if now < bot.reel_until {
            return Vec2::ZERO;
        }
        let ground = nav.ground(here);
        let in_air = bot.body.lift != 0.0 || ground.is_none_or(|ground| (bot.body.at.y - ground).abs() > 0.02);
        // Jumping a gap: at where it means to come down and nothing else,
        // off the edge with a jump ([`Body::leap`]), until it is down
        // again; or, still on the ground, has come to where it was going
        // with no gap on the way; or where it meant to come down is gone,
        // to a blast; or it has come to nothing. **In the air it keeps on
        // for where it means to come down, and stops over that very
        // spot**: until 2026-11-14 the jump was over as soon as it was
        // within 0.4 m of there, in the air or not, which over the void,
        // with the place to come down on the very edge of a hole, left it
        // to steer for the nearest ground from there, and it fell
        // ([`Nav::nearest_walkable_within`]); and going at its pace all
        // the way, it would have been to one side of the spot and the
        // other of it, frame by frame, as it came down.
        if let Some(mut leap) = bot.leaping {
            leap.aloft |= in_air;
            let down = leap.aloft && !in_air;
            let there = !in_air && here.distance(leap.to) < 0.4;
            let gone = in_air && nav.ground(leap.to).is_none();
            if down || there || gone || now >= leap.until {
                bot.leaping = None;
                bot.path.clear();
                bot.path_to = None;
                bot.progress = (here, now);
            } else {
                bot.leaping = Some(leap);
                bot.body.leap = true;
                let off = leap.to - here;
                return off.normalize_or_zero() * leap.pace.min(off.length() / dt.max(1e-4));
            }
        }
        if in_air && ground.is_none() {
            return nav
                .nearest_walkable_within(here, REACH_BACK)
                .map_or(Vec2::ZERO, |land| (land - here).normalize_or_zero() * RUN_SPEED * bot.grip);
        }
        if bot.body.shove != Vec2::ZERO {
            // How far what carries it has still to carry it: further in
            // the air than along the ground.
            let carried = here + bot.body.shove * if in_air { 0.5 } else { 0.11 };
            if nav.ground(carried).is_none() {
                return -bot.body.shove.normalize_or_zero() * RUN_SPEED * bot.grip;
            }
        }
        let walk = self.intend(bot, view, dt, order);
        // A jump it has just begun is next frame's; and a stick of
        // dynamite comes before wherever it was going.
        if bot.leaping.is_some() {
            return walk;
        }
        let walk = self.mind_sticks(bot, view, walk);
        // Running somewhere, a jump now and then, as people run: the more
        // the bouncier it is. Not with a plank in its arms or a potion at
        // its mouth, nor in a fight, which has jumps of its own.
        let running = walk.length() > RUN_SPEED * 0.6;
        if running && !in_air && bot.duel.is_none() && !order.carrying && bot.drinking.is_none() && now >= bot.hop_at {
            bot.hop_at = now + self.dice.around(0.7, 3.2) / (0.25 + bot.temper.bouncy);
            if self.dice.roll() < 0.6 {
                bot.body.jump(nav);
            }
        }
        walk
    }

    /// What the bot does this frame toward its goal, and which way it
    /// walks for it.
    fn intend(&mut self, bot: &mut Bot, view: &View, dt: f32, order: &mut Order) -> Vec2 {
        let skill = self.skill;
        let now = view.elapsed;
        let (seat, nav) = (bot.seat, view.nav);
        let team = usize::from(arena_team(seat));
        let spots = self.spots(nav)[team];
        let here = bot.body.at.xz();
        let kit = view.outfits.kit(seat);

        // In the pond, where a knock put it: to its rim, and over it.
        if nav.grid().region(here) == Some(Region::Pocket) && !nav.walkable_at(here) {
            let out = nav.grid().nearest_clear(here, 32.0).unwrap_or(spots.heart);
            let way = (out - here).normalize_or_zero();
            let ahead = here + way * 0.6;
            if nav.ground(ahead).is_none_or(|ground| ground - bot.body.at.y > STEP) && bot.body.free(nav) {
                bot.body.hop(way);
                return Vec2::ZERO;
            }
            return way * RUN_SPEED;
        }

        // A potion at its mouth: slowly, and nothing else, until it is
        // drunk.
        let mut pace = RUN_SPEED;
        if let Some((until, potion)) = bot.drinking {
            if now >= until {
                bot.drinking = None;
                // The one it put to its mouth, of however many kinds it
                // carries.
                order.drink = Some(potion);
            } else {
                order.drinking = true;
                pace *= DRINK_SLOW;
            }
        }

        // Faster while a speed potion lasts.
        if kit.fast(now) {
            pace *= FAST;
        }
        // A potion, of whichever kinds it carries, each in its own time
        // and with nobody near enough to spill it: an invisibility potion
        // as it comes up to the island it is raiding, to walk onto it
        // unseen; a speed potion as it sets out on a raid or a hunt, or
        // for home from far off when someone is on its island; and a
        // healing potion whenever it is hurt and has the room to drink it.
        // Not in the frame one has just been drunk: it is on its button
        // still, as this frame sees its kit, and for want of this a bot
        // put the same potion to its mouth a second time, for nothing.
        if bot.drinking.is_none() && order.drink.is_none() && bot.body.free(nav) {
            let nearest = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| seen.alive && arena_team(other) != arena_team(seat))
                .map(|(_, seen)| seen.at.xz().distance(here))
                .fold(f32::INFINITY, f32::min);
            let health = view.seats.get(seat).map_or(1.0, |me| me.health);
            let speed = kit.held(Utility::Speed) > 0 && !kit.fast(now);
            let unseen = kit.held(Utility::Invisibility) > 0 && !kit.unseen(now);
            let potion = match bot.goal {
                _ if health < 0.55 && kit.held(Utility::Healing) > 0 => Some(Utility::Healing),
                Goal::Raid(victim)
                    if unseen && here.distance(ARENA_ISLANDS[usize::from(victim) % ARENA_ISLANDS.len()]) < 62.0 =>
                {
                    Some(Utility::Invisibility)
                }
                Goal::Raid(_) | Goal::Hunt if speed => Some(Utility::Speed),
                Goal::Defend if speed && here.distance(spots.heart) > 30.0 => Some(Utility::Speed),
                _ => None,
            };
            if let Some(potion) = potion
                && nearest > 9.0
            {
                bot.drinking = Some((now + DRINK_SECS, potion));
            }
        }

        // A fireball, at whoever of another team is a throw away and has
        // land under them: never at someone out on a bridge.
        if kit.held(Utility::Fireball) > 0 && now >= bot.throw_at && bot.drinking.is_none() {
            let target = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
                .map(|(_, seen)| seen.at)
                .filter(|at| (4.0..FIREBALL_RANGE - 1.5).contains(&at.xz().distance(here)) && (at.y - bot.body.at.y).abs() < 1.0)
                .filter(|at| nav.grid().height(at.xz()).is_some() && !nav.on_plank(at.xz()))
                .min_by(|a, b| a.xz().distance_squared(here).total_cmp(&b.xz().distance_squared(here)));
            if let Some(target) = target {
                let off = target.xz() - here;
                order.throw = Some((-off.x).atan2(-off.y));
                bot.throw_at = now + skill.throw_every;
            }
        }

        // A stick of dynamite, at whoever of another team is out on a
        // bridge a throw away: at where they will be by the time it
        // blows, if that is on a bridge still, and otherwise at where
        // they are, to take the bridge from under them or from in front
        // of them. From the land, or from out on a bridge itself if what
        // would go is well clear of where it stands; never where one of
        // its own team stands on the planks that would go, and not with a
        // plank in its arms. Not the moment it could, but soon.
        let ashore = nav.grid().height(here).is_some();
        if kit.held(Utility::Dynamite) > 0
            && now >= bot.blast_at
            && bot.drinking.is_none()
            && bot.laying.is_none()
            && bot.body.free(nav)
        {
            let team = arena_team(seat);
            let mates: Vec<Vec2> = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| other != seat && seen.alive && arena_team(other) == team)
                .map(|(_, seen)| seen.at.xz())
                .collect();
            let aim = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != team)
                .filter(|(_, seen)| (seen.at.y - bot.body.at.y).abs() < 1.5 && out_over_the_void(nav, seen.at.xz()))
                .filter_map(|(other, seen)| {
                    let there = seen.at.xz();
                    let away = there.distance(here);
                    if !(3.5..=DYNAMITE_FURTHEST + 2.0).contains(&away) {
                        return None;
                    }
                    // In the air for about as long as it is far, and
                    // then most of its fuse.
                    let ahead = self.foresee(nav, other, there, 0.45 + 0.04 * away + 0.6 * DYNAMITE_FUSE);
                    let at = if out_over_the_void(nav, ahead) { ahead } else { there };
                    let far = at.distance(here);
                    let nearest = if ashore { DYNAMITE_NEAREST + 0.3 } else { DYNAMITE_RADIUS + 2.0 };
                    let reached = (nearest..=DYNAMITE_FURTHEST).contains(&far);
                    let spares = mates.iter().all(|mate| !nav.on_plank(*mate) || mate.distance(at) > DYNAMITE_RADIUS + 1.5);
                    (reached && spares).then_some((away, at))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, at)| at);
            if let Some(at) = aim
                && self.dice.roll() < dt * 4.0
            {
                let off = at - here;
                order.blast = Some(((-off.x).atan2(-off.y), off.length()));
                bot.blast_at = now + self.dice.around(6.0, 12.0);
            }
        }

        // Whoever it fights, if anyone.
        if bot.shun.is_some_and(|(_, until)| now >= until) {
            bot.shun = None;
        }
        bot.foe = self.foe(bot, view);
        if let Some(foe) = bot.foe.filter(|_| bot.drinking.is_none()) {
            return self.fight(bot, view, foe, pace, dt, order);
        }
        bot.in_reach_since = None;
        bot.duel = None;

        match bot.goal {
            Goal::Idle => {
                // Nothing to do: about its island, from one thing on it
                // to another and not straight to any of them, as people
                // wander when there is nothing to do; the one that keeps
                // the base mostly by its anchor. (Until 2026-11-12 it
                // stood on one spot.)
                if bot.loiter.is_none_or(|(_, until)| now >= until) {
                    let places = [spots.guard, spots.heart, spots.pad, spots.items, spots.upgrades];
                    // (Beside a person, not by the pad: what it makes is
                    // theirs to stand on.)
                    let mate = view.seats.get(mate_of(seat)).copied().unwrap_or_default();
                    let odds = if mate.person && mate.in_play {
                        [3.5, 3.0, 0.0, 1.5, 1.5]
                    } else if bot.role == Role::Keeper {
                        [5.0, 1.5, 1.5, 1.0, 1.0]
                    } else {
                        [1.0, 3.0, 2.0, 1.5, 1.5]
                    };
                    let about = Vec2::from_angle(self.dice.roll() * std::f32::consts::TAU) * self.dice.between(0.0, 1.6);
                    let to = places[pick(&mut self.dice, &odds)] + about;
                    let to = nav.nearest_walkable(to).filter(|to| nav.connected(here, *to)).unwrap_or(spots.heart);
                    bot.loiter = Some((to, now + self.dice.around(2.5, 8.0)));
                }
                let to = bot.loiter.map_or(spots.heart, |(to, _)| to);
                self.walk_to(bot, view, to, pace * 0.6, 2.0)
            }
            Goal::Collect => {
                // Beside a person, a visit and no more: once it has been
                // at the pad a moment, or there is nothing left lying
                // there, it is off, and not back for a while.
                let mate = view.seats.get(mate_of(seat)).copied().unwrap_or_default();
                if mate.person && mate.in_play {
                    let lying = view.mint.held(team) + view.mint.held(ARENA_TEAMS + team);
                    let since = (here.distance(spots.pad) < 1.3).then(|| *bot.pad_since.get_or_insert(now));
                    if since.is_some_and(|since| now - since > 1.4 || lying == 0) || bot.yields {
                        bot.pad_since = None;
                        bot.pad_again = now + self.dice.around(9.0, 16.0);
                        bot.think_at = now;
                        return Vec2::ZERO;
                    }
                }
                self.walk_to(bot, view, spots.pad, pace, 2.0)
            }
            Goal::Shop(ware) => {
                let shop = if matches!(crate::bridge_wars::sold_at(ware), crate::bridge_wars::Shop::Items) {
                    spots.items
                } else {
                    spots.upgrades
                };
                if here.distance(shop) <= 1.2 {
                    order.buy = Some(ware);
                    bot.tries += 1;
                    // Asked for and not had: something else, for a while.
                    if bot.tries > 2 {
                        bot.barred = Some((bot.goal, now + 6.0));
                    }
                    bot.think_at = now;
                    return Vec2::ZERO;
                }
                self.walk_to(bot, view, shop, pace, 2.0)
            }
            Goal::Fetch(pad, _) => {
                // Given long enough there, it thinks of something else.
                if now - bot.goal_since > 45.0 {
                    bot.barred = Some((bot.goal, now + 15.0));
                    bot.think_at = now;
                }
                self.walk_to(bot, view, pad, pace, 2.0)
            }
            Goal::Defend => self.walk_to(bot, view, spots.guard, pace, 1.0),
            Goal::Guard => {
                // Where it stands, between them and what its team keeps,
                // turned to them.
                let Some((spot, foe)) = bot.vigil.and_then(|vigil| vigil.stand.map(|stand| (stand.spot, vigil.foe))) else {
                    bot.think_at = bot.think_at.min(now);
                    return Vec2::ZERO;
                };
                if here.distance(spot) <= 1.5
                    && let Some(them) = view.seats.get(foe)
                {
                    let off = them.at.xz() - here;
                    if off.length() > 0.5 {
                        bot.face = Some((-off.x).atan2(-off.y));
                    }
                }
                self.walk_to(bot, view, spot, pace, 1.0)
            }
            Goal::Retreat => {
                // Home if there is a way, to its pad, the far side of its
                // island from anyone coming; and a healing potion there,
                // or anywhere nobody is near.
                let nearest = view
                    .seats
                    .iter()
                    .enumerate()
                    .filter(|&(other, seen)| seen.alive && arena_team(other) != arena_team(seat))
                    .map(|(_, seen)| seen.at.xz().distance(here))
                    .fold(f32::INFINITY, f32::min);
                if bot.drinking.is_none() && order.drink.is_none() && bot.body.free(nav) {
                    if kit.held(Utility::Healing) > 0 && nearest > 6.0 {
                        bot.drinking = Some((now + DRINK_SECS, Utility::Healing));
                    } else if kit.held(Utility::Speed) > 0 && !kit.fast(now) && (8.0..22.0).contains(&nearest) {
                        // Or a speed potion, to get away on, with whoever
                        // is after it far enough off for it to be drunk.
                        bot.drinking = Some((now + DRINK_SECS, Utility::Speed));
                    }
                }
                if nav.connected(here, spots.pad) {
                    self.walk_to(bot, view, spots.pad, pace, 2.0)
                } else {
                    Vec2::ZERO
                }
            }
            Goal::Lay(from, to) => self.lay(bot, view, from, to, pace, order, dt),
            Goal::Hunt => {
                // After whoever of another team is nearest that it can
                // get at; with nobody, it thinks of something else.
                let prey = view
                    .seats
                    .iter()
                    .enumerate()
                    .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
                    .map(|(_, seen)| seen.at.xz())
                    .filter(|&at| Self::reachable(nav, here, at))
                    .min_by(|a, b| a.distance_squared(here).total_cmp(&b.distance_squared(here)));
                match prey {
                    Some(prey) => self.walk_to(bot, view, prey, pace, 1.0),
                    None => {
                        bot.think_at = bot.think_at.min(now + 0.5);
                        Vec2::ZERO
                    }
                }
            }
            Goal::Raid(victim) => {
                let victim = usize::from(victim);
                let theirs = self.spots(nav)[victim];
                let on_island = nav.grid().region(here) == Some(Region::Team(victim as u8)) && !nav.on_plank(here);
                if !on_island {
                    bot.raiding_since = None;
                    return self.walk_to(bot, view, theirs.heart, pace, 2.0);
                }
                // There: no longer than a raid lasts, which for some is
                // longer than for others; and for the anchor or not, as
                // it is like, thrown as it steps ashore.
                if bot.raiding_since.is_none() {
                    bot.for_anchor = self.dice.roll() < bot.persona.anchors;
                }
                let since = *bot.raiding_since.get_or_insert(now);
                if now - since > skill.raid_secs * bot.persona.lingers {
                    bot.barred = Some((bot.goal, now + skill.raid_rest));
                    bot.think_at = now;
                    return Vec2::ZERO;
                }
                // Their anchor, once anchors are fair game and while few
                // enough are broken: one bot at it at a time, and at half
                // a bot's pace.
                let broken = view.anchors.iter().filter(|anchor| **anchor <= 0.0).count();
                let spared = broken >= skill.anchors_spared.0 && now < skill.anchors_spared.1;
                let free = self.strikers[victim].is_none_or(|striker| striker == seat);
                if bot.for_anchor && view.anchors[victim] > 0.0 && now >= skill.anchors_from && !spared && free {
                    self.strikers[victim] = Some(seat);
                    let anchor = ARENA_ANCHORS[victim];
                    if here.distance(anchor) <= REACH + ANCHOR_RADIUS - 0.2 {
                        let off = anchor - here;
                        bot.face = Some((-off.x).atan2(-off.y));
                        if now >= bot.strike_at {
                            bot.strike_at = now + skill.anchor_every;
                            order.strike = Some(victim as u8);
                        }
                        return Vec2::ZERO;
                    }
                    return self.walk_to(bot, view, theirs.strike, pace, 2.0);
                }
                // Otherwise onto their pad, for what lies on it, and for
                // whoever comes back to it.
                self.walk_to(bot, view, theirs.pad, pace, 2.0)
            }
        }
    }

    /// A frame of the bot's fight with whoever is in `foe`: its punch,
    /// when they are in its reach and it is due, now and then with a jump;
    /// and where it goes, by how it is fighting them for now ([`Stance`]),
    /// which it thinks of again every few seconds and whenever either of
    /// them steps onto a bridge or off one. It faces them whichever way
    /// it walks.
    #[allow(clippy::too_many_arguments)]
    fn fight(&mut self, bot: &mut Bot, view: &View, foe: usize, pace: f32, dt: f32, order: &mut Order) -> Vec2 {
        let skill = self.skill;
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let there = view.seats[foe].at.xz();
        let off = there - here;
        let away = off.length();
        bot.face = Some((-off.x).atan2(-off.y));

        // How good it is in a fight ([`Temper::pvp`]): the better, the
        // sooner its first punch comes once it has someone in reach, and
        // the nearer its punches come to as fast as they count (a body is
        // past hurting for half a second after a blow that took).
        let pvp = bot.temper.pvp;
        let every = skill.punch_every - (skill.punch_every - 0.53).max(0.0) * pvp;
        let windup = skill.windup * (1.0 - 0.65 * pvp);
        // Coming in again from a step back timed to its next punch, that
        // punch is thrown as they come into its reach: it knew they would.
        let lunging = bot.duel.is_some_and(|duel| duel.foe == foe && duel.lunge);
        let mut struck = false;
        if away <= REACH + BODY - 0.1 {
            let since = *bot.in_reach_since.get_or_insert(now);
            if (lunging || now - since >= bot.windup) && now >= bot.punch_at {
                // Not to the beat: a little sooner, a little later; the
                // best of them nearly to it.
                bot.punch_at = now + self.dice.around(every * (0.75 + 0.2 * pvp), every * (1.3 - 0.24 * pvp));
                bot.windup = self.dice.around(windup * 0.5, windup * 1.6);
                order.punch = Some(foe);
                // On the run only if it really was running in.
                order.running = bot.speed > RUN_SPEED * 0.6;
                struck = true;
                // And now and then with a jump, as people throw theirs.
                if self.dice.roll() < 0.1 + 0.3 * bot.temper.bouncy {
                    bot.body.jump(nav);
                }
            }
        } else if away > REACH + BODY + 0.5 {
            bot.in_reach_since = None;
        }
        // A jump of no use at all, now and then, close in.
        if away < 6.0 && now >= bot.hop_at {
            bot.hop_at = now + self.dice.around(0.8, 3.4) / (0.3 + bot.temper.bouncy);
            if self.dice.roll() < 0.5 {
                bot.body.jump(nav);
            }
        }

        // How it goes about this one, for the next while.
        let (me_out, foe_out) = (out_over_the_void(nav, here), out_over_the_void(nav, there));
        let stale = bot
            .duel
            .is_none_or(|duel| duel.foe != foe || now >= duel.until || duel.me_out != me_out || duel.foe_out != foe_out);
        if stale {
            // Having stood off someone on the land from out on a bridge
            // long enough, now and then it gives them up for a while.
            let stood_off = bot
                .duel
                .is_some_and(|duel| duel.foe == foe && duel.stance == Stance::Wait && duel.me_out && !duel.foe_out && now >= duel.until);
            if stood_off && me_out && !foe_out && self.dice.roll() < 0.3 {
                bot.shun = Some((foe, now + self.dice.around(3.0, 8.0)));
                bot.duel = None;
                bot.think_at = now;
                return Vec2::ZERO;
            }
            bot.duel = Some(self.take_stance(bot, view, foe, me_out, foe_out));
        }
        let Some(mut duel) = bot.duel else {
            return Vec2::ZERO;
        };
        if struck {
            // In and out: back out of their reach until its next punch is
            // nearly due, to come in again on the run, where a punch
            // knocks further. Poking is that; and the better a bot is in a
            // fight, the oftener it does so however it is fighting, and
            // the closer to the moment it cuts it (Hajun, 2026-11-13:
            // "they seem to get knocked back more than they knock me
            // back": a bot stood where it was between its punches, and a
            // punch thrown standing knocks half as far as one thrown
            // running in).
            // (Back in from where it steps back to, to where they come
            // into its reach, as its next punch falls due.)
            let timed = (bot.punch_at - (POKE_BACK - (REACH + BODY - 0.1)) / pace.max(1.0)).max(now + 0.12);
            duel.lunge = false;
            match duel.stance {
                Stance::Poke if self.dice.roll() >= pvp => duel.back_until = now + self.dice.around(0.45, 1.0),
                // It has had its punch from where it lay in wait: at them.
                Stance::Ambush => {
                    duel.stance = Stance::Press;
                    duel.spot = None;
                    duel.until = now + self.dice.around(1.5, 3.5);
                }
                Stance::Poke => (duel.back_until, duel.lunge) = (timed, true),
                Stance::Press | Stance::Circle if self.dice.roll() < 0.9 * pvp => (duel.back_until, duel.lunge) = (timed, true),
                _ => {}
            }
            // Wherever it stood to punch, it is somewhere else next.
            duel.spot_until = now;
        }
        // Where it goes for them: at where they will be, not where they
        // are ([`Squad::meet`]).
        let aim = self.meet(nav, bot, foe, there, pace);
        let brisk = self.speed_of(foe) > 3.0;
        // Someone on their way over a gap to it, in the air over the void
        // or running at it with no straight way between them, is waited
        // for: it jumps no gap itself until they are down or have stopped
        // (seen in a game of bots on 2026-11-14: two of them after each
        // other across a hole a blast had left in a bridge jumped it at
        // the same moment, passing each other in the air, three times
        // running, and the third time one of them fell).
        let coming = !nav.line(here, there) && {
            let aloft = nav.ground(there).is_none_or(|ground| view.seats[foe].at.y - ground > 0.3);
            let closing = self
                .tracks
                .get(foe)
                .is_some_and(|track| track.known && track.velocity.dot(-off / away.max(1e-3)) > 3.0);
            aloft || closing
        };
        bot.awaits = coming;
        // A gap between it and them that it could jump, with no walking
        // straight to them and the way round far, or none: now and then
        // it jumps it, the likelier the bolder and the better it is, as
        // someone after you would (Hajun, 2026-11-13: "i jump across to
        // the other bridge, and instead of the ai also jumping across, it
        // goes all the way back to the island and then go on the other
        // bridge - so i can jump back and forth between the 2 bridge").
        // It stands where it is for the moment it takes to make up its
        // mind, rather than set off round and be out of reach of the jump
        // by then; and having thought better of it, it goes round, and
        // thinks of it again in a while.
        if bot.leaping.is_none()
            && !coming
            && pace >= RUN_SPEED * 0.9
            && now >= duel.round_until
            && bot.body.free(nav)
            && !nav.line(here, there)
            && let Some((_, down)) = nav.leap(here, there, Self::reach_of_a_jump(pace))
        {
            let far = match duel.far {
                Some((far, stood)) if stood.distance(there) < 2.0 => far,
                _ => {
                    let round = nav.path(here, there).map(|path| {
                        let mut from = here;
                        path.iter().fold(0.0, |walked, &point| {
                            let step = from.distance(point);
                            from = point;
                            walked + step
                        })
                    });
                    let far = round.is_none_or(|round| round > 2.0 * away + 5.0);
                    duel.far = Some((far, there));
                    far
                }
            };
            if far && now < duel.leap_at {
                bot.duel = Some(duel);
                return Vec2::ZERO;
            }
            if far {
                let keen = (0.3 + 0.3 * bot.temper.bold + 0.35 * pvp).clamp(0.3, 0.95);
                if self.dice.roll() < keen {
                    bot.leaping = Some(Leap {
                        to: down,
                        pace,
                        until: now + 2.0,
                        aloft: false,
                    });
                    bot.duel = Some(duel);
                    return (down - here).normalize_or_zero() * pace;
                }
                duel.round_until = now + self.dice.around(1.5, 3.0);
                duel.leap_at = duel.round_until + self.dice.between(0.1, 0.4);
                duel.far = None;
            }
        }
        // Straight to somewhere near, no further than it is in a frame;
        // round whatever stands between otherwise.
        let toward = |squad: &mut Self, bot: &mut Bot, to: Vec2, pace: f32| {
            let stretch = to - here;
            if stretch.length() <= 0.15 {
                Vec2::ZERO
            } else if nav.line(here, to) {
                bot.path.clear();
                bot.path_to = None;
                stretch.normalize_or_zero() * pace.min(stretch.length() / dt.max(1e-3))
            } else {
                squad.walk_to(bot, view, to, pace, 0.5)
            }
        };
        let walk = match duel.stance {
            Stance::Wait => {
                if away < HOLD_OFF - 1.4 {
                    // They have come: at them, then.
                    duel.stance = if self.dice.roll() < 0.5 { Stance::Press } else { Stance::Poke };
                    duel.spot = None;
                    duel.until = now + self.dice.around(1.5, 3.5);
                    Vec2::ZERO
                } else if away > HOLD_OFF + 1.2 {
                    toward(self, bot, aim, pace)
                } else {
                    Vec2::ZERO
                }
            }
            Stance::Ambush => match duel.spot {
                Some(lair) => toward(self, bot, lair, pace),
                None => Vec2::ZERO,
            },
            Stance::Press | Stance::Circle | Stance::Poke | Stance::Rush => {
                let backing = duel.stance != Stance::Rush && now < duel.back_until;
                if duel.spot.is_none() || now >= duel.spot_until {
                    duel.spot = if backing {
                        // Straight back from them, as far as there is
                        // ground to stand on that way.
                        let back = here - off.normalize_or_zero() * (POKE_BACK - away).max(0.0);
                        (nav.walkable_at(back) && flung(nav, there, back) == 0.0).then_some(back)
                    } else {
                        self.spot(bot, view, aim, &duel)
                    };
                    // Sooner again for someone on the move, who is not
                    // where they were by then; and, backing off, the
                    // moment that is over.
                    let again = now + if brisk { self.dice.between(0.1, 0.22) } else { self.dice.between(0.3, 0.7) };
                    duel.spot_until = if backing { again.min(duel.back_until) } else { again };
                }
                // Circling, a little slower; but not after someone who is
                // running, which would never catch them.
                let pace = if duel.stance == Stance::Circle && !brisk { pace * 0.85 } else { pace };
                match duel.spot {
                    Some(spot) => toward(self, bot, spot, pace),
                    // Nowhere to stand round them: up to them as it used
                    // to go, or, backing, where it is.
                    None if backing || away <= REACH * PRESS_IN => Vec2::ZERO,
                    None => toward(self, bot, aim, pace),
                }
            }
        };
        // Off the bridge and past them: on the land, it fights on as it
        // would there.
        if duel.stance == Stance::Rush && !me_out {
            duel.until = now;
        }
        bot.duel = Some(duel);
        walk
    }

    /// How the bot goes about whoever is in `foe` for the next while, by
    /// chance: weighted by where the two of them stand, on the land or
    /// out on a bridge, by how bold it is, and by who it is
    /// ([`Persona::fights`]). On the land it presses,
    /// circles or pokes; with them coming over a bridge it may lie in
    /// wait beside where the bridge comes ashore, or wait, or go to meet
    /// them; out on a bridge with them on the land ahead it may stop short
    /// of their reach, or run for the land past them, rather than stand
    /// on the last plank to be thrown off it; and with both of them out
    /// on a bridge it presses, pokes or waits. Having waited once, it does
    /// not wait again.
    fn take_stance(&mut self, bot: &Bot, view: &View, foe: usize, me_out: bool, foe_out: bool) -> Duel {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let there = view.seats[foe].at.xz();
        let bold = bot.temper.bold;
        let shy = 2.0 - bold;
        let side = if self.dice.roll() < 0.5 { 1.0 } else { -1.0 };
        // Press, circle, poke, wait, ambush, rush.
        let mut odds = match (me_out, foe_out) {
            (false, false) => [3.0 * bold, 3.0, 2.5 * bold, 0.4 * shy, 0.0, 0.0],
            (false, true) => [2.0 * bold, 0.0, 1.5 * bold, 1.5 * shy, 4.5 * shy, 0.0],
            (true, false) => [1.0 * bold, 0.0, 1.5, 3.5 * shy, 0.0, 4.0 * bold],
            (true, true) => [3.0 * bold, 0.0, 3.0, 2.0 * shy, 0.0, 0.0],
        };
        for (odd, leaning) in odds.iter_mut().zip(bot.persona.fights) {
            *odd *= leaning;
        }
        if bot.duel.is_some_and(|was| was.foe == foe && matches!(was.stance, Stance::Wait | Stance::Ambush)) {
            odds[3] = 0.0;
            odds[4] = 0.0;
        }
        let lair = (odds[4] > 0.0).then(|| lair(nav, here, there, side)).flatten();
        if lair.is_none() {
            odds[4] = 0.0;
        }
        let stances = [Stance::Press, Stance::Circle, Stance::Poke, Stance::Wait, Stance::Ambush, Stance::Rush];
        let stance = stances[pick(&mut self.dice, &odds)];
        let lasts = match stance {
            Stance::Press | Stance::Circle | Stance::Poke => self.dice.around(1.6, 4.2),
            Stance::Wait => self.dice.around(1.4, 3.6),
            Stance::Ambush => 5.0,
            Stance::Rush => 3.0,
        };
        Duel {
            foe,
            stance,
            until: now + lasts,
            side,
            spot: if stance == Stance::Ambush { lair } else { None },
            spot_until: 0.0,
            back_until: 0.0,
            me_out,
            foe_out,
            // A moment to take in that there is a gap between them.
            leap_at: now + self.dice.between(0.15, 0.5),
            round_until: 0.0,
            far: None,
            lunge: false,
        }
    }

    /// Where the bot stands to fight someone `there`: the best of twelve
    /// places round them a punch's length out, each a place a body can
    /// walk that it can get to. Better for being near; much better, by
    /// how much it goes for that, if a punch from there would carry them
    /// off the land ([`flung`]), and worse, by how much it minds, if one
    /// of theirs would carry it off; circling, better for being a little further round them the
    /// way it is going; and all of it a little by chance, so that it is
    /// not the same place twice. A bot running for the land past them
    /// takes only a place on the land. `None` with nowhere to stand: out
    /// on a bridge's middle line there are two places at the most.
    fn spot(&mut self, bot: &Bot, view: &View, there: Vec2, duel: &Duel) -> Option<Vec2> {
        let nav = view.nav;
        let here = bot.body.at.xz();
        let (shoves, minds) = (bot.temper.shoves, bot.temper.minds);
        let mine = (here - there).to_angle();
        let turn = self.dice.roll() * std::f32::consts::TAU / 12.0;
        let mut best: Option<(f32, Vec2)> = None;
        for place in 0..12 {
            let angle = turn + place as f32 * std::f32::consts::TAU / 12.0;
            let at = there + Vec2::from_angle(angle) * RING;
            if !nav.walkable_at(at) || !nav.connected(here, at) {
                continue;
            }
            if duel.stance == Stance::Rush && out_over_the_void(nav, at) {
                continue;
            }
            let mut score = -0.09 * here.distance(at) + shoves * flung(nav, at, there) - 1.1 * minds * flung(nav, there, at);
            if duel.stance == Stance::Circle {
                // How far round them from where it is, the way it goes:
                // best about a sixth of a turn on.
                let round = (angle - mine + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                score += 0.5 - (round * duel.side - 1.0).abs() * 0.35;
            }
            if !nav.line(here, at) {
                score -= 0.5;
            }
            score += 0.18 * self.dice.bell();
            if best.is_none_or(|(most, _)| score > most) {
                best = Some((score, at));
            }
        }
        best.map(|(_, at)| at)
    }

    /// Whom the bot fights this frame, if anyone: the nearest of another
    /// team it can see and get at, within how far it goes for people,
    /// which is further the bolder it is; but for whoever it has given up
    /// on for now, unless they have just hit it.
    fn foe(&self, bot: &Bot, view: &View) -> Option<usize> {
        let (seat, nav, now) = (bot.seat, view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let me = view.seats[seat];
        let kit = view.outfits.kit(seat);
        let sight = if kit.blind { BLIND_SIGHT } else { HUNT_RADIUS };
        let stung = now - bot.hurt_at < bot.sting;
        let reach = match bot.goal {
            // Backing off, it fights only whoever is on top of it.
            Goal::Retreat => 2.6,
            Goal::Defend | Goal::Guard | Goal::Hunt => sight,
            // On the island it is raiding, whoever is home.
            Goal::Raid(victim) if nav.grid().region(here) == Some(Region::Team(victim)) => sight,
            // As near as it lets people come just now ([`KEEN`]).
            _ => (self.skill.aggro_at(now) * bot.temper.bold * bot.keen).max(if stung { 9.0 } else { 0.0 }).min(sight),
        };
        // Standing toward someone who has stepped back from its base, it
        // goes at them too, for as far from where it stands as it means
        // to this time: not from wherever it has got to after them, or it
        // would be after them for as long as they kept walking (as one
        // was, 30 m down its own bridge, the first time this was tried).
        let watched = bot
            .vigil
            .filter(|_| bot.goal == Goal::Guard)
            .and_then(|vigil| vigil.stand.map(|stand| (vigil.foe, stand.spot, stand.watch)));
        let home = Region::Team(arena_team(seat));
        let on_plank = out_over_the_void(nav, here);
        view.seats
            .iter()
            .enumerate()
            .filter(|&(other, seen)| other != seat && seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
            .filter(|&(other, _)| stung || !bot.shun.is_some_and(|(shunned, _)| shunned == other))
            .filter_map(|(other, seen)| {
                let there = seen.at.xz();
                let away = there.distance(here);
                // Whoever it is on already it keeps after a little
                // further than it would have gone for them.
                let reach = if bot.foe == Some(other) { (reach * KEEPS_ON).min(sight.max(reach)) } else { reach };
                if away > reach || (seen.at.y - bot.body.at.y).abs() > 3.0 {
                    return None;
                }
                // Seeing someone off its island, it meets them on the
                // land: it does not hold the end of a bridge against
                // them, but for whoever it stands toward.
                if matches!(bot.goal, Goal::Defend | Goal::Guard) && away > 2.6 && nav.grid().region(there) != Some(home) {
                    let at_them = watched.is_some_and(|(foe, spot, watch)| foe == other && there.distance(spot) <= watch);
                    if !at_them {
                        return None;
                    }
                }
                // Hurt and out on a bridge, it starts nothing.
                if on_plank && me.health < 0.5 && away > 3.0 {
                    return None;
                }
                // Someone it can walk to; or, with only the void between
                // them, someone a jump away, over a hole in a bridge or
                // from the one bridge to the next.
                let jump = Self::reach_of_a_jump(RUN_SPEED);
                if !nav.connected(here, there) && nav.crossing(here, there, jump).is_none() && nav.leap(here, there, jump).is_none() {
                    return None;
                }
                // Whoever it is on already, a little sooner than another.
                Some((if bot.foe == Some(other) { away * 0.8 } else { away }, other))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, other)| other)
    }

    /// Which way the bot walks, at `pace`, to get to `to`: along its way
    /// there, found again when where it is going has moved, every few
    /// times `again` seconds, and, having no way, as soon as a plank is
    /// laid anywhere; nowhere once it is there. A plank only ever adds
    /// ground, so a way it has stays good while bridges are built (until
    /// 2026-11-12 every bot found its way again at every plank anyone
    /// laid). Getting nowhere for a while, it thinks again. **It does not
    /// walk the way as a ruler would**: it weaves a little to either side
    /// as it goes, the more the wider it weaves by nature, less on a
    /// bridge and none at all close to where it is going (Hajun,
    /// 2026-11-12: "more nonlinear/curve/slightly random not efficient
    /// movements to make it seem like a real player").
    fn walk_to(&mut self, bot: &mut Bot, view: &View, to: Vec2, pace: f32, again: f32) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        if here.distance(to) <= ARRIVED {
            bot.path.clear();
            bot.progress = (here, now);
            return Vec2::ZERO;
        }
        // A plank laid only adds to the ways there are; a blast takes
        // some away, and a way found before one may lead over a hole now.
        let stale = bot.path_to.is_none_or(|was| was.distance(to) > 1.0)
            || (bot.path.is_empty() && (bot.path_version != nav.version || now - bot.path_at > 0.5))
            || now - bot.path_at > again.max(0.5) * 4.0
            || bot.path_cuts != nav.cuts;
        if stale {
            bot.cross = None;
            bot.path = match nav.path(here, to) {
                Some(path) => path,
                // No way there: to the edge of a hole in a bridge that it
                // can jump, if one lies on the way, and over it.
                None => match nav.crossing(here, to, Self::reach_of_a_jump(pace)).filter(|_| pace >= RUN_SPEED * 0.9) {
                    Some(gap) => {
                        bot.cross = Some(gap);
                        nav.path(here, gap.a_in).unwrap_or_default()
                    }
                    None => Vec::new(),
                },
            };
            bot.path_to = Some(to);
            bot.path_version = nav.version;
            bot.path_at = now;
            bot.path_cuts = nav.cuts;
        }
        if let Some(gap) = bot.cross
            && here.distance(gap.a_in) <= 0.6
            && bot.body.free(nav)
            && !bot.awaits
        {
            bot.cross = None;
            bot.path.clear();
            bot.path_to = None;
            bot.progress = (here, now);
            bot.leaping = Some(Leap {
                to: gap.b_in,
                pace,
                until: now + 2.0,
                aloft: false,
            });
            return (gap.b_in - here).normalize_or_zero() * pace;
        }
        while bot.path.first().is_some_and(|next| next.distance(here) <= ARRIVED) {
            bot.path.remove(0);
        }
        // Getting nowhere: something else.
        if here.distance(bot.progress.0) > 0.75 {
            bot.progress = (here, now);
        } else if now - bot.progress.1 > 3.0 {
            bot.progress = (here, now);
            bot.path.clear();
            bot.path_to = None;
            if !matches!(bot.goal, Goal::Idle) {
                bot.barred = Some((bot.goal, now + 8.0));
                bot.think_at = now;
            }
        }
        match bot.path.first() {
            Some(&next) => {
                let off = next - here;
                let far = off.length();
                // To one side and the other of the straight way, slowly,
                // and not in time with anyone else.
                bot.weave += (1.0 / 60.0) * (1.9 + 0.6 * bot.temper.weave);
                let room = if out_over_the_void(nav, here) { 0.35 } else { 1.0 };
                let swing = (bot.weave.sin() + 0.5 * (bot.weave * 2.3).sin()) * 0.16 * bot.temper.weave * room * (far / 3.0).min(1.0);
                // No further than the point itself in a frame.
                Vec2::from_angle(swing).rotate(off.normalize_or_zero()) * pace.min(far / (1.0 / 60.0))
            }
            None => Vec2::ZERO,
        }
    }

    /// How a bridge of `chord` metres bends on its way, thrown as it is
    /// begun ([`Span::bend`]): a sweep to one side, a wave and a ripple,
    /// each as a bell rings it, and all of them by how straight this
    /// bridge is to be, which is thrown as a bell rings too: most a
    /// little wavy, a few all but straight, a few all over the place
    /// (Hajun, 2026-11-12: "add some random bell curve variation to the
    /// 'straightness' of their bridging. straight line bridge look too
    /// ai"). Never so bent that it leaves or meets the land at more than
    /// a slant of [`MOST_SLANT`], nor that it is more than [`MOST_BENT`]
    /// times as long as the straight one.
    fn bends(&mut self, span: Span) -> Span {
        let chord = span.chord();
        let wavy = (1.0 + 0.5 * self.dice.bell()).clamp(0.1, 2.4) * chord / 44.0;
        let mut bend = [1.3, 0.7, 0.35].map(|far| far * wavy * self.dice.bell());
        for _ in 0..12 {
            // How it slants at its two ends, as so much to the side for
            // each metre on.
            let turn = std::f32::consts::PI / chord.max(1.0);
            let slants = [bend[0] + 2.0 * bend[1] + 3.0 * bend[2], bend[0] - 2.0 * bend[1] + 3.0 * bend[2]].map(|sum| (sum * turn).abs());
            let bent = span.bent(bend);
            if slants[0] <= MOST_SLANT && slants[1] <= MOST_SLANT && bent.length() <= chord * MOST_BENT - 0.2 {
                return bent;
            }
            bend = bend.map(|far| far * 0.8);
        }
        span
    }

    /// Lays the bridge from `from` to `to`: to where it begins; then out
    /// along it, bends and all, at the pace anyone builds at, each plank
    /// laid ahead of its feet and paid for in metres; and it is done at
    /// the far end.
    #[allow(clippy::too_many_arguments)]
    fn lay(&mut self, bot: &mut Bot, view: &View, from: Region, to: Region, pace: f32, order: &mut Order, dt: f32) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        // A blast has taken planks since it last looked: if any were of
        // the bridge it is laying, behind it, there is no walking on along
        // it. It leaves it as it is and thinks again: of mending the hole,
        // or of another bridge.
        if let Some(laying) = bot.laying
            && laying.cuts != nav.cuts
        {
            let holed = (0..laying.next.saturating_sub(1)).any(|plank| nav.broken(bot.seat as u8, laying.chain, plank as u16));
            if holed {
                bot.laying = None;
                bot.planned = None;
                self.claims.retain(|claim| claim.2 != bot.seat);
                bot.think_at = now;
                return Vec2::ZERO;
            }
            bot.laying = Some(Laying { cuts: nav.cuts, ..laying });
        }
        let laying = match bot.laying {
            Some(laying) => laying,
            None => {
                // The bridge its last thought settled on, if that is the
                // one it is here to lay. (Until 2026-11-13 this asked for
                // it again, as if for the one of two bots that goes out:
                // a person's teammate whose kind opens to a smaller
                // island, or late, was given that bridge, or none yet, in
                // place of the one to the middle it had the metres for.)
                let Some((_, _, span)) = bot.planned.filter(|planned| (planned.0, planned.1) == (from, to)) else {
                    bot.think_at = now;
                    return Vec2::ZERO;
                };
                // With a Bridge Builder on a button: to where the bridge
                // is to begin, and the builder is sent off from there to
                // lay it, straight, while the bot goes about something
                // else.
                if view.outfits.kit(bot.seat).held(Utility::Builder) > 0 && span.length() <= BUILDER_METRES - 2.0 {
                    let near = nav.nearest_walkable(span.from).unwrap_or(span.from);
                    if here.distance(near) > ARRIVED + 0.3 {
                        return self.walk_to(bot, view, near, pace, 2.0);
                    }
                    order.build = true;
                    bot.sent = Some(Sent {
                        span,
                        from,
                        to,
                        chain: bot.chains,
                        next: 0,
                        along: 0.0,
                    });
                    bot.chains += 1;
                    match self.begun.iter_mut().find(|begun| (begun.0, begun.1) == (from, to)) {
                        Some(begun) => begun.2 += 1,
                        None => self.begun.push((from, to, 1)),
                    }
                    self.claims.retain(|claim| claim.2 != bot.seat);
                    bot.planned = None;
                    bot.think_at = now;
                    return Vec2::ZERO;
                }
                let laying = Laying {
                    // A hole mended is a few planks, laid straight.
                    span: if span.chord() < MEND { span } else { self.bends(span) },
                    from,
                    to,
                    chain: bot.chains,
                    next: 0,
                    cuts: nav.cuts,
                };
                bot.chains += 1;
                match self.begun.iter_mut().find(|begun| (begun.0, begun.1) == (from, to)) {
                    Some(begun) => begun.2 += 1,
                    None => self.begun.push((from, to, 1)),
                }
                bot.laying = Some(laying);
                laying
            }
        };
        let span = laying.span;
        let (way, planks, chord) = (span.way(), span.planks(), span.chord());
        // A rung to so much of the straight line, and so much bridge.
        let (step, plank) = (chord / planks as f32, span.length() / planks as f32);
        let along = (here - span.from).dot(way);
        let onto = span.point(along / chord.max(1e-3));
        let aside = here.distance(onto);
        // Not at its line yet: to the nearest a body can walk to where it
        // begins, or, taking it up again, to as far as it is laid.
        let reached = span.rung(laying.next.saturating_sub(1));
        let on_line = aside < 0.9 && along > -1.5 && along < chord + 1.0;
        if !on_line || (laying.next == 0 && here.distance(span.from) > 2.2) {
            let to = if laying.next == 0 { span.from } else { reached };
            let near = nav.nearest_walkable(to).unwrap_or(to);
            if here.distance(near) > ARRIVED + 0.3 {
                return self.walk_to(bot, view, near, pace, 2.0);
            }
        }
        // Its planks, a rung at a time, kept ahead of its feet: the first
        // rung is the bridge's root, and costs nothing.
        let mut next = laying.next;
        while next <= planks && (next as f32 - 1.0) * step < along + LAID_AHEAD + step {
            let cost = if next == 0 { 0.0 } else { plank };
            if bot.metres + 1e-3 < cost {
                break;
            }
            bot.metres -= cost;
            order.rungs.push((laying.chain, next as u16, span.rung(next), span.heading_at(next)));
            next += 1;
        }
        bot.laying = Some(Laying { next, ..laying });
        order.carrying = true;
        // Laid to its end, and walked to it: done.
        if next > planks && along >= chord - ARRIVED {
            bot.laying = None;
            self.claims.retain(|claim| claim.2 != bot.seat);
            bot.think_at = now;
            return Vec2::ZERO;
        }
        // Out along it, no further than it is laid.
        let laid = (next.saturating_sub(1)) as f32 * step;
        let limit = if next > planks { chord } else { (laid - 0.4).max(0.0) };
        let speed = BUILDER_SPEED.min((limit - along).max(0.0) / dt.max(1e-3));
        // Onto its middle line, and on along it as it bends.
        let sideways = onto - here;
        if speed <= 0.0 && sideways.length() < 0.05 {
            // Out of metres on the way: nothing for it but to go and get
            // more.
            if bot.metres + 1e-3 < plank && next <= planks {
                bot.think_at = bot.think_at.min(now + 0.2);
            }
            return Vec2::ZERO;
        }
        span.tangent(along / chord.max(1e-3)) * speed + sideways.clamp_length_max(1.0) * 3.0
    }
}

/// Moves on the Bridge Builder `bot` has sent off, if any: along its
/// bridge at the pace anyone builds at, each plank laid a rung or two
/// ahead of it, to the far shore, where it is done.
fn build_on(bot: &mut Bot, dt: f32, order: &mut Order) {
    let Some(mut sent) = bot.sent else {
        return;
    };
    let (planks, chord) = (sent.span.planks(), sent.span.chord());
    let step = chord / planks as f32;
    sent.along += BUILDER_SPEED * dt;
    while sent.next <= planks && (sent.next as f32 - 1.0) * step < sent.along + LAID_AHEAD + step {
        order
            .rungs
            .push((sent.chain, sent.next as u16, sent.span.rung(sent.next), sent.span.heading_at(sent.next)));
        sent.next += 1;
    }
    bot.sent = (sent.next <= planks).then_some(sent);
}

/// Whether someone `at` is out on a bridge with nothing but the void
/// under it: on a plank, and over no land.
fn out_over_the_void(nav: &Nav, at: Vec2) -> bool {
    nav.grid().height(at).is_none() && nav.on_plank(at)
}

/// How surely a punch thrown from `from` carries whoever stands `at` off
/// the land and into the void, from 0 to 1: by whether there is ground
/// where a knock would put them down, as far on as one carries
/// ([`CARRY`]). What a bot goes by in choosing where to stand
/// ([`Squad::spot`]): where its punch throws them off, and theirs does
/// not throw it off.
fn flung(nav: &Nav, from: Vec2, at: Vec2) -> f32 {
    let away = (at - from).normalize_or_zero();
    if away == Vec2::ZERO {
        return 0.0;
    }
    CARRY.iter().filter(|far| nav.ground(at + away * **far).is_none()).count() as f32 / CARRY.len() as f32
}

/// Where a bot standing `here` on the land lies in wait for someone
/// coming over a bridge from `there`: beside where their bridge comes
/// ashore, on the way between the two of them, a little inland and well
/// to one side (`side`, or the other if there is no standing there), so
/// that a punch at whoever steps ashore throws them back over the side of
/// their bridge. `None` if they are far off, the landfall is far from the
/// bot, or there is nowhere to stand.
fn lair(nav: &Nav, here: Vec2, there: Vec2, side: f32) -> Option<Vec2> {
    let whole = here.distance(there);
    if !(2.0..=16.0).contains(&whole) {
        return None;
    }
    let way = (here - there) / whole;
    let landfall = (1..=(whole / 0.4) as usize)
        .map(|step| there + way * (step as f32 * 0.4))
        .find(|&at| nav.grid().clear(at))?;
    if landfall.distance(here) > 9.0 {
        return None;
    }
    [side, -side]
        .into_iter()
        .map(|side| landfall + way * 1.2 + Vec2::new(-way.y, way.x) * side * 1.9)
        .find(|&at| nav.walkable_at(at) && nav.grid().height(at).is_some())
}

/// The two smaller islands beside `team`'s own, by their pads' numbers.
fn beside(team: u8) -> [u8; 2] {
    let island = ARENA_ISLANDS[usize::from(team) % ARENA_ISLANDS.len()];
    let mut nearest: Vec<u8> = (0..SAPPHIRE_PADS.len() as u8).collect();
    nearest.sort_by(|&a, &b| {
        SAPPHIRE_PADS[usize::from(a)]
            .distance_squared(island)
            .total_cmp(&SAPPHIRE_PADS[usize::from(b)].distance_squared(island))
    });
    [nearest[0], nearest[1]]
}

/// What a bot would buy, in the order it would: by what it does on its
/// team, what it holds and has, how many metres of bridge it is `short`
/// of for the span it means to lay, **and who it is**
/// ([`Persona::spends`]): everything it might buy next has a place in the
/// order they all bought in until 2026-11-12 (the team's upgrades, the
/// heal pool first; the aura sword and shield; a sword; a shield;
/// something for the second button; a better shield), and its leaning to
/// each kind of thing moves that up or down. So one puts a fireball
/// before its sword, another its team's defences before its generator,
/// and a third buys as they all did. What goes on its buttons is what it
/// means to carry this life (`loadout`, [`Persona::loadout`]): each kind
/// in its turn, the first before the rest, as many of each as it keeps,
/// and a second or a third of a kind only after the things it has none
/// of. A bridge it is short of (`short`) is bought as metres, or, by one
/// that carries Bridge Builders and has the amethyst, as a builder to lay
/// it, unless it is part way along laying it itself (`laying`). And with
/// a bridge worth a builder to be had (`spare`, [`Squad::spare_span`]),
/// one that carries them buys one in its turn among the rest.
#[allow(clippy::too_many_arguments)]
fn shopping(
    persona: &Persona,
    loadout: &[(Utility, u8)],
    role: Role,
    alone: bool,
    elapsed: f32,
    purse: &Purse,
    kit: &Kit,
    team: &Upgrades,
    short: f32,
    laying: bool,
    spare: bool,
) -> Vec<Ware> {
    let mut list = Vec::new();
    let has = |item: Item| purse[item.index()];
    // Its bridge before anything: a builder to lay it, or the metres.
    let builder = Ware::Utility(Utility::Builder);
    let builds = loadout.iter().any(|(kind, _)| *kind == Utility::Builder)
        && has(Item::Amethyst) >= 1
        && kit.held(Utility::Builder) == 0
        && price(builder, kit, team).is_some();
    if short > 0.0 {
        if builds && !laying && short <= BUILDER_METRES - 2.0 {
            list.push(builder);
        } else {
            let buys = (short / BRIDGE_A_BUY).ceil() as usize;
            list.extend(std::iter::repeat_n(Ware::Bridge, buys.min(4)));
        }
    }
    let [sword, shield, button, generator, defences, swords] = persona.spends;
    let keeper = role == Role::Keeper && !alone;
    let mut wanted: Vec<(f32, Ware)> = Vec::new();
    // Sapphire is the team's: the heal pool, its generator, its swords,
    // its shields, a trap.
    if has(Item::Sapphire) > 0 {
        for (rank, leaning, ware) in [
            (1.0, defences, Ware::HealPool),
            (0.98, generator, Ware::Forge),
            (0.96, swords, Ware::StrongerSwords),
            (0.94, defences, Ware::BetterShields),
            (0.92, defences, Ware::Trap),
        ] {
            if price(ware, kit, team).is_some() {
                wanted.push((rank * leaning, ware));
            }
        }
    }
    // Amethyst is the best of its own arms.
    if has(Item::Amethyst) >= 4 && kit.sword < 3 {
        wanted.push((0.9 * sword, Ware::Sword(3)));
    }
    if has(Item::Amethyst) >= 6 && kit.shield < 3 {
        wanted.push((0.88 * shield, Ware::Shield(3)));
    }
    // A sword: the silver one once the gold for it is near, and until
    // then the basic one, which the keeper has first of all.
    if kit.sword < 2 && (has(Item::Gold) >= 4 || kit.sword == 1) {
        wanted.push((0.86 * sword, Ware::Sword(2)));
    }
    if kit.sword == 0 && has(Item::Gold) < 4 && (keeper || elapsed > 45.0) {
        wanted.push((0.84 * sword, Ware::Sword(1)));
    }
    if kit.shield == 0 && (keeper || kit.sword > 0 || shield > 1.25) {
        wanted.push((0.8 * shield, Ware::Shield(1)));
    }
    // Its buttons: each kind it means to carry, in its turn, once it is
    // armed, or at once if that is what it is like; a fireball late on and
    // with silver to spare, unless it is one for fireballs. A keeper with
    // nobody to fetch it amethyst makes do with a healing potion in place
    // of what amethyst buys. As many of a kind as it keeps, the second and
    // the third of one well after the first of the next.
    let keen = button > 1.2;
    let stuck = keeper && has(Item::Amethyst) == 0;
    // A builder for a bridge its team could do without, in its turn.
    if spare && builds && short == 0.0 {
        wanted.push((0.74 * button, builder));
    }
    let mut kinds: Vec<Utility> = Vec::new();
    for &(kind, keeps) in loadout {
        let kind = match kind {
            Utility::Speed | Utility::Invisibility | Utility::Builder | Utility::Dynamite if stuck => Utility::Healing,
            other => other,
        };
        if kinds.contains(&kind) {
            continue;
        }
        kinds.push(kind);
        let ready = match kind {
            Utility::Fireball => (elapsed > 300.0 && kit.sword > 0 && kit.shield > 0) || (keen && elapsed > 45.0),
            Utility::Healing => kit.sword > 0 || keen,
            Utility::Speed | Utility::Invisibility | Utility::Dynamite => has(Item::Amethyst) > 0 || (keen && elapsed > 60.0),
            // Bought for a bridge it is short of, before anything.
            Utility::Builder => false,
        };
        let held = kit.held(kind);
        if ready && held < keeps && price(Ware::Utility(kind), kit, team).is_some() {
            let turn = 0.76 - 0.05 * (kinds.len() - 1) as f32;
            let again = if held > 0 { 0.6 } else { 1.0 };
            wanted.push((turn * again * button, Ware::Utility(kind)));
        }
    }
    if kit.sword >= 2 && kit.shield < 2 {
        wanted.push((0.72 * shield, Ware::Shield(2)));
    }
    wanted.sort_by(|a, b| b.0.total_cmp(&a.0));
    list.extend(wanted.into_iter().map(|(_, ware)| ware));
    list
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::nav::Grid;

    /// An island 12 m across at the origin, its top 0.25 m up, `Team(0)`.
    fn island() -> Nav {
        let mut grid = Grid::blank(60, 60, Vec2::new(-15.0, -15.0));
        for row in 0..60 {
            for column in 0..60 {
                let away = grid.centre_of(column, row).length();
                if away <= 6.0 {
                    grid.set(column, row, Some(0.25), away <= 5.5, Some(Region::Team(0)));
                }
            }
        }
        grid.place_land(Region::Team(0), Vec2::ZERO);
        Nav::new(Arc::new(grid))
    }

    /// Two islands 12 m across, 30 m apart on the x axis, `Team(1)` at the
    /// origin and `Middle` at (30, 0), with a straight bridge between
    /// them: for trying a fight at a bridge's end.
    fn bridged() -> Nav {
        let (mut nav, span) = two_islands();
        for index in 0..=span.planks() {
            nav.hear(1, 0, index as u16, span.rung(index));
        }
        assert!(nav.joined(Region::Team(1), Region::Middle));
        nav
    }

    /// The two islands of [`bridged`] with nothing between them yet, and
    /// the straight bridge there would be.
    fn two_islands() -> (Nav, Span) {
        let mut grid = Grid::blank(100, 40, Vec2::new(-10.0, -10.0));
        for row in 0..40 {
            for column in 0..100 {
                let at = grid.centre_of(column, row);
                for (centre, region) in [(Vec2::ZERO, Region::Team(1)), (Vec2::new(30.0, 0.0), Region::Middle)] {
                    let away = at.distance(centre);
                    if away <= 6.0 {
                        grid.set(column, row, Some(0.25), away <= 5.5, Some(region));
                    }
                }
            }
        }
        grid.place_land(Region::Team(1), Vec2::ZERO);
        grid.place_land(Region::Middle, Vec2::new(30.0, 0.0));
        let span = grid.span(Region::Team(1), Region::Middle, 0.0).expect("a span between them");
        (Nav::new(Arc::new(grid)), span)
    }

    /// What became of a bot sent over the bridge at someone standing on
    /// the far land beside its end, who punches it whenever it is in
    /// their reach, twice a second, and is never moved themselves: whether
    /// it fell, when, whether it ever stood on the far land, and how it
    /// went about them.
    struct Crossing {
        fell: Option<f32>,
        landed: bool,
        stances: Vec<Stance>,
        knocks: u32,
    }

    fn cross_to_someone_waiting(seed: u64, who: Option<&str>, waits_at: Vec2, secs: f32) -> Crossing {
        let nav = bridged();
        let mut squad = Squad::new(seed, Skill::MODERATE);
        let seat = 5;
        squad.seat(seat, Vec3::new(4.0, 0.25, 0.0), 0.0);
        if let Some(who) = who {
            assert!(squad.cast(seat, who));
        }
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        let mut seen = vec![Seen::default(); ARENA_SEATS];
        seen[0] = Seen {
            at: Vec3::new(waits_at.x, 0.25, waits_at.y),
            alive: true,
            in_play: true,
            unseen: false,
            person: true,
            health: 1.0,
        };
        let mut told = Crossing {
            fell: None,
            landed: false,
            stances: Vec::new(),
            knocks: 0,
        };
        let dt = 1.0 / 60.0;
        let mut cooldown = 0.0f32;
        // Late in a game, when everyone hunts everyone.
        let mut now = Skill::MODERATE.hunt_from + 5.0;
        let until = now + secs;
        while now < until {
            now += dt;
            cooldown -= dt;
            let body = squad.bot(seat).expect("seated").body;
            seen[seat] = Seen {
                at: body.at,
                alive: true,
                in_play: true,
                unseen: false,
                person: false,
                health: 1.0,
            };
            if body.at.y < -5.0 {
                told.fell = Some(now - (until - secs));
                break;
            }
            let view = View {
                elapsed: now,
                nav: &nav,
                seats: &seen,
                purses: &purses,
                outfits: &outfits,
                mint: &mint,
                anchors: [120.0; ARENA_TEAMS],
                sticks: &[],
            };
            squad.step(seat, &view, dt);
            let bot = squad.bot(seat).expect("seated");
            if let Some(stance) = bot.stance()
                && told.stances.last() != Some(&stance)
            {
                told.stances.push(stance);
            }
            let at = bot.body.at;
            told.landed |= nav.grid().region(at.xz()) == Some(Region::Middle) && !nav.on_plank(at.xz()) && at.y <= 0.26;
            if cooldown <= 0.0 && at.xz().distance(waits_at) <= REACH + BODY && (at.y - 0.25).abs() < 2.0 {
                squad.knock(seat, waits_at, false);
                told.knocks += 1;
                cooldown = 0.5;
            }
        }
        told
    }

    /// For tuning by: `cargo test -p roundtown_net crossings -- --ignored
    /// --nocapture`. How a bot of each kind fares against someone waiting
    /// beside the far end of its bridge.
    #[test]
    #[ignore]
    fn crossings_to_someone_waiting_at_a_bridge_s_end() {
        // Beside where the bridge comes ashore, to one side: where a
        // punch at whoever arrives throws them off the side of it.
        let waits_at = Vec2::new(26.2, 1.9);
        let mut all = (0, 0, 0);
        for persona in &PERSONAS {
            let (mut fell, mut landed, mut knocks) = (0, 0, 0);
            let mut when = 0.0;
            let trials = 60;
            for seed in 0..trials {
                let told = cross_to_someone_waiting(seed, Some(persona.name), waits_at, 30.0);
                fell += u32::from(told.fell.is_some());
                when += told.fell.unwrap_or(30.0);
                landed += u32::from(told.landed);
                knocks += told.knocks;
            }
            println!(
                "{}: fell {fell} of {trials} (after {:.1} s on average), stood on the far land {landed}, knocked {:.1} times",
                persona.name,
                when / trials as f32,
                knocks as f32 / trials as f32
            );
            all = (all.0 + fell, all.1 + landed, all.2 + trials as u32);
        }
        println!("all: fell {} of {}, stood on the far land {}", all.0, all.2, all.1);
        let told = cross_to_someone_waiting(3, None, waits_at, 30.0);
        println!("one of them: fell {:?}, landed {}, {:?}, {} knocks", told.fell, told.landed, told.stances, told.knocks);
    }

    #[test]
    fn a_body_walks_where_it_can_and_never_off_an_edge() {
        let nav = island();
        let mut body = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        assert!(body.free(&nav));
        // East at a run for two seconds: it is stopped short of the edge,
        // on the last of the land a body walks.
        for _ in 0..120 {
            body.step(&nav, Vec2::X * RUN_SPEED, 1.0 / 60.0);
        }
        assert!(body.at.x > 5.0 && body.at.x < 5.6, "{}", body.at);
        assert_eq!(body.at.y, 0.25);
        assert!(body.free(&nav));
        // Facing the way it walked: east.
        assert!((body.yaw + std::f32::consts::FRAC_PI_2).abs() < 1e-3, "{}", body.yaw);
        // Walked at a slant into the edge, it slides along it.
        let before = body.at;
        for _ in 0..30 {
            body.step(&nav, Vec2::new(1.0, 1.0).normalize() * RUN_SPEED, 1.0 / 60.0);
        }
        assert!(body.at.z > before.z + 0.5 && nav.walkable_at(body.at.xz()), "{}", body.at);
    }

    #[test]
    fn a_knock_carries_a_body_and_off_the_edge_it_falls_unless_its_legs_bring_it_back() {
        let nav = island();
        let frame = 1.0 / 60.0;
        // Knocked from the west while standing in the middle: up and
        // east, down again on the island, and its own to move after.
        let mut body = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        body.knock(Vec2::new(-1.0, 0.0), false, 1.0);
        assert!(!body.free(&nav));
        assert_eq!((body.shove, body.lift), (Vec2::X * KNOCK_BACK, KNOCK_UP));
        let mut highest = 0.0f32;
        for _ in 0..180 {
            body.step(&nav, Vec2::ZERO, frame);
            highest = highest.max(body.at.y);
        }
        assert!(highest > 1.2, "it rose to {highest}");
        assert!(body.free(&nav), "{body:?}");
        assert!(body.at.x > 2.0 && body.at.x < 5.6 && body.at.y == 0.25, "{}", body.at);
        let carried = body.at.x;
        // Its legs are its own all the while: running back at whoever
        // knocked it, it is carried hardly at all.
        let mut back = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        back.knock(Vec2::new(-1.0, 0.0), false, 1.0);
        let mut furthest = 0.0f32;
        for _ in 0..50 {
            back.step(&nav, Vec2::NEG_X * RUN_SPEED, frame);
            furthest = furthest.max(back.at.x);
        }
        assert!(furthest < carried - 1.5, "carried {furthest} m against {carried}");
        assert!(back.at.x < 0.0, "and back past where it stood: {}", back.at);
        // On the run from near the edge, it goes over it and falls, and
        // is never its own to move again.
        let mut body = Body::standing(Vec3::new(4.0, 0.25, 0.0), 0.0);
        body.knock(Vec2::new(3.0, 0.0), true, 1.0);
        for _ in 0..180 {
            body.step(&nav, Vec2::ZERO, frame);
            assert!(!body.free(&nav) || body.at.y > 0.0);
        }
        assert!(body.at.x > 6.5 && body.at.y < -20.0, "{}", body.at);
        // A knock that takes it a little way out over the void: with its
        // legs still, it is gone; steering back for the land, it lands on
        // it. And in the air over the land, its legs do not take it out
        // over the void: only the knock does.
        let mut gone = Body::standing(Vec3::new(3.2, 0.25, 0.0), 0.0);
        gone.knock(Vec2::new(2.0, 0.0), false, 1.0);
        let mut saved = gone;
        let mut out = false;
        for _ in 0..180 {
            gone.step(&nav, Vec2::ZERO, frame);
            // Its legs still until it is out over the void; then back
            // for the land.
            out |= nav.ground(saved.at.xz()).is_none();
            saved.step(&nav, if out { Vec2::NEG_X * RUN_SPEED } else { Vec2::ZERO }, frame);
        }
        assert!(gone.at.y < -20.0, "{}", gone.at);
        assert!(out, "it was carried out over the void");
        assert!(saved.free(&nav) && saved.at.y == 0.25 && saved.at.x < 5.6, "{saved:?}");
        let mut jumper = Body::standing(Vec3::new(4.5, 0.25, 0.0), 0.0);
        assert!(jumper.jump(&nav) && !jumper.jump(&nav), "one jump from the ground");
        let mut highest = 0.0f32;
        for _ in 0..120 {
            jumper.step(&nav, Vec2::X * RUN_SPEED, frame);
            highest = highest.max(jumper.at.y);
        }
        assert!(highest > 1.6, "a jump as high as anyone's: {highest}");
        assert!(jumper.free(&nav) && jumper.at.x < 5.6, "it came down on the land at its edge: {}", jumper.at);
        // A zombie takes a share of it.
        let mut zombie = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        zombie.knock(Vec2::new(-1.0, 0.0), true, 0.3);
        assert!((zombie.shove.x - 6.0).abs() < 1e-4 && (zombie.lift - 2.4).abs() < 1e-4);
    }

    #[test]
    fn a_body_stood_under_the_island_s_top_is_put_onto_it_and_one_fallen_under_it_goes_on_falling() {
        let nav = island();
        // Where a device's arrival stands it: at the land's own height, a
        // quarter of a metre under the island's top. It is put onto it,
        // and is its own to move.
        let mut body = Body::standing(Vec3::new(1.0, 0.0, 1.0), 0.0);
        assert!(!body.free(&nav));
        body.step(&nav, Vec2::ZERO, 1.0 / 60.0);
        assert_eq!(body.at.y, 0.25);
        assert!(body.free(&nav));
        for _ in 0..30 {
            body.step(&nav, Vec2::X * RUN_SPEED, 1.0 / 60.0);
        }
        assert!(body.at.x > 4.0 && body.at.y == 0.25, "{}", body.at);
        // Knocked off the edge and carried back in under the island as it
        // falls, it goes on falling: it does not hang there.
        let mut fallen = Body::standing(Vec3::new(5.0, -3.0, 0.0), 0.0);
        fallen.lift = -6.0;
        fallen.shove = Vec2::new(-4.0, 0.0);
        for _ in 0..180 {
            fallen.step(&nav, Vec2::ZERO, 1.0 / 60.0);
            assert!(!fallen.free(&nav));
        }
        assert!(fallen.at.y < -30.0, "{}", fallen.at);
        // Just under the edge, it catches it.
        let mut caught = Body::standing(Vec3::new(5.0, 0.1, 0.0), 0.0);
        caught.lift = -1.0;
        caught.step(&nav, Vec2::ZERO, 1.0 / 60.0);
        assert_eq!(caught.at.y, 0.25);
    }

    /// Someone with no leaning to anything.
    fn plain() -> Persona {
        Persona {
            name: "plain",
            spends: [1.0; 6],
            ..PERSONAS[0]
        }
    }

    fn persona(name: &str) -> &'static Persona {
        PERSONAS.iter().find(|persona| persona.name == name).expect("one of them")
    }

    #[test]
    fn what_a_bot_buys_goes_by_what_it_holds_and_what_it_does_and_who_it_is() {
        let (kit, team) = (Kit::default(), Upgrades::default());
        let plain = plain();
        let healing = [(Utility::Healing, 1)];
        let list = |role, elapsed, purse: &Purse, kit: &Kit, short| {
            shopping(&plain, &healing, role, false, elapsed, purse, kit, &team, short, false, false)
        };
        // Short of 44 m of bridge: two buys of it before anything.
        assert_eq!(&list(Role::Runner, 10.0, &[8, 0, 0, 0], &kit, 44.6)[..2], &[Ware::Bridge, Ware::Bridge]);
        // The keeper: a basic sword, and a shield.
        assert_eq!(list(Role::Keeper, 10.0, &[10, 1, 0, 0], &kit, 0.0), vec![Ware::Sword(1), Ware::Shield(1)]);
        // The runner waits for the silver sword's gold, early on.
        assert!(list(Role::Runner, 10.0, &[10, 1, 0, 0], &kit, 0.0).is_empty());
        assert_eq!(list(Role::Runner, 60.0, &[10, 5, 0, 0], &kit, 0.0)[0], Ware::Sword(2));
        // Sapphire goes on the team, the heal pool first.
        let armed = Kit { sword: 2, shield: 1, ..kit };
        let bought = list(Role::Runner, 200.0, &[0, 0, 3, 0], &armed, 0.0);
        assert_eq!(&bought[..2], &[Ware::HealPool, Ware::Forge]);
        // With a sword, a healing potion for the second button.
        assert!(bought.contains(&Ware::Utility(Utility::Healing)) && bought.contains(&Ware::Shield(2)));

        // Who it is moves things up and down. The daredevil has its
        // fireball before its sword; the sentinel, by the book, its sword.
        let silver: Purse = [60, 0, 0, 0];
        let fireballs = [(Utility::Fireball, 3)];
        let daring = shopping(persona("ESTP"), &fireballs, Role::Runner, false, 60.0, &silver, &kit, &team, 0.0, false, false);
        assert_eq!(daring[0], Ware::Utility(Utility::Fireball), "{daring:?}");
        let steady = shopping(persona("ISTJ"), &fireballs, Role::Runner, false, 60.0, &silver, &kit, &team, 0.0, false, false);
        assert!(matches!(steady[0], Ware::Sword(1) | Ware::Shield(1)), "{steady:?}");
        assert!(!steady.contains(&Ware::Utility(Utility::Fireball)), "not before it is armed, and late: {steady:?}");
        // The defender spends its team's sapphire on defences, the
        // commander on swords.
        let sapphire: Purse = [0, 0, 9, 0];
        let safe = shopping(persona("ISFJ"), &healing, Role::Keeper, false, 200.0, &sapphire, &armed, &team, 0.0, false, false);
        assert_eq!(&safe[..3], &[Ware::HealPool, Ware::BetterShields, Ware::Trap], "{safe:?}");
        let bent_on = shopping(persona("ENTJ"), &healing, Role::Runner, false, 200.0, &sapphire, &armed, &team, 0.0, false, false);
        assert_eq!(bent_on[0], Ware::StrongerSwords, "{bent_on:?}");
        // The troll has an invisibility potion as soon as it has the
        // amethyst for it, before the best of swords; a keeper who
        // fancies one, with nobody to fetch amethyst, a healing potion.
        let amethyst: Purse = [0, 0, 0, 4];
        let unseen = [(Utility::Invisibility, 1)];
        let sly = shopping(persona("ESFP"), &unseen, Role::Runner, false, 200.0, &amethyst, &armed, &team, 0.0, false, false);
        assert_eq!(sly[0], Ware::Utility(Utility::Invisibility), "{sly:?}");
        let home = shopping(persona("ESFP"), &unseen, Role::Keeper, false, 200.0, &[0, 9, 0, 0], &armed, &team, 0.0, false, false);
        assert!(home.contains(&Ware::Utility(Utility::Healing)) && !home.contains(&Ware::Utility(Utility::Invisibility)));
        // Sixteen of them, each by its own name, and none with odds of
        // nothing at all.
        let mut names: Vec<&str> = PERSONAS.iter().map(|persona| persona.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 16);
        for persona in &PERSONAS {
            assert!(persona.opening.iter().sum::<f32>() > 0.0 && persona.carries.iter().sum::<f32>() > 0.0, "{}", persona.name);
            let [least, most] = persona.pockets;
            assert!(1 <= least && least <= most && usize::from(most) <= KINDS.len(), "{}", persona.name);
            assert!(persona.fights.iter().all(|leaning| *leaning > 0.0), "{}", persona.name);
        }
    }

    #[test]
    fn they_go_for_people_from_further_off_as_the_game_goes_on() {
        let skill = Skill::MODERATE;
        assert_eq!(skill.aggro_at(0.0), 6.0);
        assert_eq!(skill.aggro_at(150.0), 6.0);
        assert!((skill.aggro_at(375.0) - 15.0).abs() < 0.01);
        assert_eq!(skill.aggro_at(600.0), HUNT_RADIUS);
        assert_eq!(skill.aggro_at(1400.0), HUNT_RADIUS);
        // The same dice, the same throws.
        let (mut a, mut b) = (Dice::new(4), Dice::new(4));
        for _ in 0..50 {
            let roll = a.roll();
            assert!((0.0..1.0).contains(&roll));
            assert_eq!(roll, b.roll());
        }
        assert_ne!(Dice::new(5).roll(), Dice::new(6).roll());
    }

    #[test]
    fn what_each_carries_on_its_buttons_goes_by_who_it_is() {
        let place = |kind: Utility| KINDS.iter().position(|other| *other == kind).expect("one of the six");
        let mut dice = Dice::new(11);
        let mut builders = std::collections::HashMap::new();
        let mut sticks = std::collections::HashMap::new();
        // Every kind there is to carry is one of them, each once.
        assert_eq!(KINDS.len(), Utility::ALL.len());
        assert!(Utility::ALL.iter().all(|kind| KINDS.contains(kind)));
        for persona in &PERSONAS {
            let [least, most] = persona.pockets;
            let (mut firsts, mut fullest) = ([0u32; 6], 0);
            for _ in 0..600 {
                let loadout = persona.loadout(&mut dice);
                // As many kinds as its pockets have it, and no more than
                // there are buttons; none of them twice, none it has no
                // leaning to, some of each, and no more fireballs or
                // sticks of dynamite than a button holds.
                assert!((usize::from(least)..=usize::from(most)).contains(&loadout.len()), "{}: {loadout:?}", persona.name);
                assert!(loadout.len() <= crate::UTILITY_SLOTS, "{}: {loadout:?}", persona.name);
                for (turn, (kind, keeps)) in loadout.iter().enumerate() {
                    assert!(loadout[..turn].iter().all(|(other, _)| other != kind), "{}: {loadout:?}", persona.name);
                    assert!(persona.carries[place(*kind)] > 0.0, "{}: {loadout:?}", persona.name);
                    assert!(*keeps >= 1 && *keeps <= crate::bridge_wars::most_held(*kind).min(3), "{}: {loadout:?}", persona.name);
                    assert!(*keeps == 1 || matches!(kind, Utility::Fireball | Utility::Healing | Utility::Dynamite));
                }
                firsts[place(loadout[0].0)] += 1;
                fullest = fullest.max(loadout.len());
                *builders.entry(persona.name).or_insert(0u32) += u32::from(loadout.iter().any(|(kind, _)| *kind == Utility::Builder));
                *sticks.entry(persona.name).or_insert(0u32) += u32::from(loadout.iter().any(|(kind, _)| *kind == Utility::Dynamite));
            }
            // What it leans to most is what it mostly buys first (or as
            // good as, where it leans to two alike); and over enough
            // lives it fills as many buttons as it has pockets.
            let leaning_most = persona.carries.iter().copied().fold(0.0, f32::max);
            let most_often = (0..6).max_by_key(|kind| firsts[*kind]).unwrap();
            assert!(persona.carries[most_often] >= 0.75 * leaning_most, "{}: {firsts:?}", persona.name);
            assert_eq!(fullest, usize::from(most), "{}", persona.name);
        }
        // Some fill all five buttons, and some keep to one or two.
        assert!(PERSONAS.iter().any(|persona| persona.pockets[1] == 5));
        assert!(PERSONAS.iter().any(|persona| persona.pockets[1] <= 2));
        // The tinkerer has a Bridge Builder far oftener than the sentinel.
        assert!(builders["INTP"] > 3 * builders["ISTJ"], "{builders:?}");
        // And the trickster a stick of dynamite far oftener than the
        // captain; everyone has one now and then.
        assert!(sticks["ENTP"] > 3 * sticks["ENFJ"], "{sticks:?}");
        assert!(sticks.values().all(|lives| *lives > 0), "{sticks:?}");
        // The daredevil's fireballs come by the handful; the sentinel has
        // one, when it has any.
        let keeps = |name: &str, kind: Utility, seed: u64| {
            let mut dice = Dice::new(seed);
            (0..200).find_map(|_| persona(name).loadout(&mut dice).into_iter().find(|(carried, _)| *carried == kind)).map(|(_, keeps)| keeps)
        };
        assert_eq!(keeps("ESTP", Utility::Fireball, 3), Some(3));
        assert_eq!(keeps("ISTJ", Utility::Fireball, 3), Some(1));
        assert_eq!(keeps("ISFJ", Utility::Healing, 3), Some(2));
        // The defender never goes unseen: it has no leaning to that.
        assert_eq!(keeps("ISFJ", Utility::Invisibility, 3), None);
    }

    #[test]
    fn a_bot_buys_each_kind_it_carries_in_its_turn_and_a_builder_for_its_bridge() {
        let team = Upgrades::default();
        let armed = Kit {
            sword: 2,
            shield: 2,
            ..Kit::default()
        };
        let plain = plain();
        let carried = [(Utility::Fireball, 2), (Utility::Healing, 1), (Utility::Speed, 1), (Utility::Builder, 1)];
        let rich: Purse = [200, 20, 0, 5];
        let buy = |kit: &Kit, purse: &Purse, short: f32, laying: bool| {
            shopping(&plain, &carried, Role::Runner, false, 400.0, purse, kit, &team, short, laying, false)
        };
        let things = |list: &[Ware]| -> Vec<Utility> {
            list.iter()
                .filter_map(|ware| match ware {
                    Ware::Utility(kind) => Some(*kind),
                    _ => None,
                })
                .collect()
        };
        // Nothing on its buttons: each kind it carries, in its turn, and
        // no builder with no bridge to lay.
        assert_eq!(things(&buy(&armed, &rich, 0.0, false)), vec![Utility::Fireball, Utility::Healing, Utility::Speed]);
        // With one fireball, the second comes after the first of the rest.
        let one = armed.holding(Utility::Fireball, 1);
        assert_eq!(things(&buy(&one, &rich, 0.0, false)), vec![Utility::Healing, Utility::Speed, Utility::Fireball]);
        // As many of each as it keeps, and no more.
        let full = armed.holding(Utility::Fireball, 2).holding(Utility::Healing, 1).holding(Utility::Speed, 1);
        assert!(things(&buy(&full, &rich, 0.0, false)).is_empty());
        // Short of a bridge: a builder to lay it, with the amethyst for
        // one, in place of the metres; the metres without the amethyst,
        // part way along laying it itself, or with a builder in hand
        // that this bridge is not for.
        let short = buy(&armed, &rich, 44.6, false);
        assert_eq!(short[0], Ware::Utility(Utility::Builder), "{short:?}");
        assert!(!short.contains(&Ware::Bridge), "{short:?}");
        assert_eq!(&buy(&armed, &[200, 20, 0, 0], 44.6, false)[..2], &[Ware::Bridge, Ware::Bridge]);
        assert_eq!(buy(&armed, &rich, 20.0, true)[0], Ware::Bridge);
        assert_eq!(buy(&armed.holding(Utility::Builder, 1), &rich, 44.6, false)[0], Ware::Bridge);
        // One that carries none buys the metres, whatever amethyst it has.
        let none = shopping(&plain, &[(Utility::Healing, 1)], Role::Runner, false, 400.0, &rich, &armed, &team, 44.6, false, false);
        assert_eq!(&none[..2], &[Ware::Bridge, Ware::Bridge]);
        // A keeper with nobody to fetch it amethyst makes do with healing
        // potions in place of what amethyst buys, once.
        let home = shopping(&plain, &carried, Role::Keeper, false, 400.0, &[200, 20, 0, 0], &armed, &team, 0.0, false, false);
        assert_eq!(things(&home), vec![Utility::Fireball, Utility::Healing]);
        // With a bridge worth a builder to be had, one that carries them
        // buys one in its turn among the rest; with one in hand, with no
        // amethyst, or carrying none, it does not.
        let spare = |kit: &Kit, purse: &Purse, carried: &[(Utility, u8)]| {
            things(&shopping(&plain, carried, Role::Runner, false, 400.0, purse, kit, &team, 0.0, false, true))
        };
        assert_eq!(spare(&armed, &rich, &carried), vec![Utility::Fireball, Utility::Builder, Utility::Healing, Utility::Speed]);
        assert!(!spare(&armed.holding(Utility::Builder, 1), &rich, &carried).contains(&Utility::Builder));
        assert!(!spare(&armed, &[200, 20, 0, 0], &carried).contains(&Utility::Builder));
        assert!(!spare(&armed, &rich, &[(Utility::Healing, 1)]).contains(&Utility::Builder));
    }

    #[test]
    fn a_bot_with_a_builder_and_nothing_it_has_to_build_opens_a_way_its_team_could_do_without() {
        // Hajun's map, with the island at the bottom joined to the middle
        // and to the smaller island on its one side, as a team has them
        // a minute or two in.
        let mut nav = Nav::arena();
        let (home, middle) = (Region::Team(0), Region::Middle);
        let [first, other] = beside(0);
        let mut chain = 0;
        let mut lay = |nav: &mut Nav, from: Region, to: Region| {
            let span = nav.grid().span(from, to, 0.0).expect("a span between them");
            for index in 0..=span.planks() {
                nav.hear(0, chain, index as u16, span.rung(index));
            }
            chain += 1;
            assert!(nav.joined(from, to), "{from:?} to {to:?}");
            span
        };
        lay(&mut nav, home, middle);
        lay(&mut nav, middle, Region::Sapphire(first));
        let outfits = {
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            outfits
        };
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        let mut seen = vec![Seen::default(); ARENA_SEATS];
        for (seat, one) in seen.iter_mut().enumerate() {
            *one = Seen {
                at: Vec3::new(ARENA_ISLANDS[seat % ARENA_TEAMS].x, 0.25, ARENA_ISLANDS[seat % ARENA_TEAMS].y),
                alive: true,
                in_play: true,
                unseen: false,
                person: false,
                health: 1.0,
            };
        }
        let spare = |nav: &Nav, name: &str, elapsed: f32| {
            let mut squad = Squad::new(3, Skill::MODERATE);
            squad.seat(4, Vec3::new(0.0, 0.25, 130.0), 0.0);
            assert!(squad.cast(4, name));
            let bot = squad.bot(4).expect("seated").clone();
            let view = View {
                elapsed,
                nav,
                seats: &seen,
                purses: &purses,
                outfits: &outfits,
                mint: &mint,
                anchors: [120.0; ARENA_TEAMS],
                sticks: &[],
            };
            squad.spare_span(&bot, &view)
        };
        // Early on, whoever it is: the other of the two smaller islands
        // beside its own, from the middle. No side door before the raids.
        for name in ["ISFJ", "ESTP"] {
            let (from, to, span) = spare(&nav, name, 100.0).expect("a bridge worth a builder");
            assert_eq!((from, to), (middle, Region::Sapphire(other)), "{name}");
            assert!(span.length() <= BUILDER_METRES - 2.0);
        }
        // Once the raids are on, one that likes a raid opens a side door:
        // from the smaller island its team has reached onto the island of
        // the other team that island lies beside. One that does not goes
        // for the sapphire still.
        let raids_on = Skill::MODERATE.raid_from + 10.0;
        assert!(persona("ESTP").raids > 0.0 && persona("ISFJ").raids <= 0.0);
        let (from, to, door) = spare(&nav, "ESTP", raids_on).expect("a side door");
        let Region::Team(theirs) = to else {
            panic!("not onto a team's island: {to:?}");
        };
        assert_eq!(from, Region::Sapphire(first));
        assert!(theirs != 0 && beside(theirs).contains(&first));
        assert!(door.length() <= BUILDER_METRES - 2.0, "{} m", door.length());
        assert_eq!(spare(&nav, "ISFJ", raids_on).map(|(from, to, _)| (from, to)), Some((middle, Region::Sapphire(other))));
        // A builder's bridge over that water is one a body walks: laid
        // rung by rung it joins the two lands, though nothing else did.
        let mut alone = Nav::arena();
        for index in 0..=door.planks() {
            alone.hear(4, 0, index as u16, door.rung(index));
        }
        assert!(alone.joined(from, to), "the side door does not land");
        // With both smaller islands reached and no raids yet, there is
        // nothing worth a builder.
        lay(&mut nav, middle, Region::Sapphire(other));
        assert!(spare(&nav, "ISFJ", 100.0).is_none());
    }

    /// One frame after another of the bot in `seat` alone in `nav`, with
    /// its kit as `outfits` has it and its health `health`, until `secs`
    /// are up or `done` says so: what it is ordered to do is done as a
    /// game would do it (a builder off its buttons, its rungs onto the
    /// map, a potion drunk), and each frame's order is shown to `each`.
    #[allow(clippy::too_many_arguments)]
    fn alone(
        squad: &mut Squad,
        nav: &mut Nav,
        outfits: &mut Outfits,
        seat: usize,
        foe: Option<Vec3>,
        mut health: f32,
        from: f32,
        secs: f32,
        mut each: impl FnMut(f32, &Order, &Nav, &Bot) -> bool,
    ) {
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        let mut seen = vec![Seen::default(); ARENA_SEATS];
        if let Some(at) = foe {
            seen[0] = Seen {
                at,
                alive: true,
                in_play: true,
                unseen: false,
                person: true,
                health: 1.0,
            };
        }
        let dt = 1.0 / 60.0;
        let mut now = from;
        while now < from + secs {
            now += dt;
            seen[seat] = Seen {
                at: squad.bot(seat).expect("seated").body.at,
                alive: true,
                in_play: true,
                unseen: false,
                person: false,
                health,
            };
            let order = {
                let view = View {
                    elapsed: now,
                    nav,
                    seats: &seen,
                    purses: &purses,
                    outfits,
                    mint: &mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &[],
                };
                squad.step(seat, &view, dt)
            };
            if order.build && !outfits.spend(seat, Utility::Builder) {
                squad.unsend(seat);
            }
            for &(chain, index, at, _) in &order.rungs {
                nav.hear(seat as u8, chain, index, at);
            }
            if let Some(potion) = order.drink
                && outfits.drink(seat, potion, now) == Some(Utility::Healing)
            {
                health = (health + crate::bridge_wars::HEALING).min(1.0);
            }
            if each(now, &order, nav, squad.bot(seat).expect("seated")) {
                break;
            }
        }
    }

    #[test]
    fn a_bridge_builder_sent_off_lays_its_bridge_while_the_bot_stays_ashore() {
        let (mut nav, span) = two_islands();
        let mut squad = Squad::new(5, Skill::MODERATE);
        let seat = 5;
        squad.seat(seat, Vec3::new(0.0, 0.25, 0.0), 0.0);
        assert!(squad.cast(seat, "ISTJ"));
        squad.set_plan(1, Plan::PLAIN);
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        outfits.kits[seat] = Kit::default().holding(Utility::Builder, 1);
        let (mut sent, mut joined, mut rungs, mut carried, mut furthest) = (None, None, Vec::new(), false, 0.0f32);
        alone(&mut squad, &mut nav, &mut outfits, seat, None, 1.0, 1.0, 40.0, |now, order, nav, bot| {
            if order.build {
                sent = Some(now);
            }
            carried |= order.carrying;
            rungs.extend(order.rungs.iter().map(|rung| (rung.1, rung.2)));
            furthest = furthest.max(bot.body.at.x);
            if sent.is_some() && nav.joined(Region::Team(1), Region::Middle) {
                joined = Some(now);
            }
            joined.is_some()
        });
        let (sent, joined) = (sent.expect("it sent its builder off"), joined.expect("its bridge reached the far land"));
        // Every rung of one straight bridge, in its order, from its own
        // shore to the far one, a little to one side of the line between
        // the two lands' middles or on it, as any bridge of a bot's is.
        let (first, last) = (rungs[0].1, rungs[rungs.len() - 1].1);
        let way = (last - first).normalize();
        assert!(rungs.len() >= 10 && rungs.len() <= span.planks() + 3, "{} rungs", rungs.len());
        for (index, (rung, at)) in rungs.iter().enumerate() {
            assert_eq!(usize::from(*rung), index);
            assert!(way.perp_dot(*at - first).abs() < 1e-3, "rung {index} is off the line");
            assert!(index == 0 || at.distance(rungs[index - 1].1) <= crate::nav::PLANK_SPACING + 1e-3);
        }
        assert!(first.x < 6.0 && last.x > 24.0 && first.y.abs() < 5.0 && (first.y - last.y).abs() < 0.5, "{first} to {last}");
        // The void between them at the pace anyone builds at, and no
        // quicker: the bridge goes up a rung at a time.
        let across = first.distance(last) / BUILDER_SPEED;
        assert!(joined - sent < across + 1.0 && joined - sent > across * 0.5, "{} s for {} m", joined - sent, first.distance(last));
        // And nothing of it the bot's own: no plank in its arms, no
        // metres spent, the builder gone from its button, and its feet on
        // its own island.
        assert!(!carried);
        let bot = squad.bot(seat).expect("seated");
        assert_eq!(bot.metres(), 0.0);
        assert!(!bot.building());
        assert_eq!(outfits.kit(seat).held(Utility::Builder), 0);
        assert!(furthest < 6.01, "it walked out to {furthest}");
        // And it lays no second bridge beside the one being built: with
        // the two joined there is none to lay.
        assert!(nav.joined(Region::Team(1), Region::Middle));
    }

    #[test]
    fn it_drinks_the_potion_the_moment_calls_for_of_those_it_carries() {
        // Late in a game, when everyone hunts everyone, with someone to
        // go for on the far land, and a speed potion, a healing potion
        // and an invisibility potion on its buttons.
        let drunk = |health: f32| {
            let mut nav = bridged();
            let mut squad = Squad::new(9, Skill::MODERATE);
            let seat = 5;
            squad.seat(seat, Vec3::new(0.0, 0.25, 0.0), 0.0);
            assert!(squad.cast(seat, "ISTJ"));
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            outfits.kits[seat] = Kit::default()
                .holding(Utility::Invisibility, 1)
                .holding(Utility::Speed, 1)
                .holding(Utility::Healing, 1);
            let mut drinks = Vec::new();
            let foe = Some(Vec3::new(32.0, 0.25, 0.0));
            let from = Skill::MODERATE.hunt_from + 5.0;
            alone(&mut squad, &mut nav, &mut outfits, seat, foe, health, from, 4.5, |_, order, _, _| {
                drinks.extend(order.drink);
                false
            });
            (drinks, outfits.kit(seat))
        };
        // Whole: the speed potion for the chase, and nothing else. An
        // invisibility potion is for walking onto an island it raids.
        let (drinks, kit) = drunk(1.0);
        assert_eq!(drinks, vec![Utility::Speed]);
        assert_eq!((kit.held(Utility::Healing), kit.held(Utility::Invisibility)), (1, 1));
        // Hurt: the healing potion first, and then the speed potion.
        let (drinks, kit) = drunk(0.45);
        assert_eq!(drinks, vec![Utility::Healing, Utility::Speed]);
        assert_eq!(kit.held(Utility::Invisibility), 1);
    }

    /// A round island `radius` across from its middle at the origin, its
    /// top 0.25 m up, `Team(1)`: room to run round and round on.
    fn round_island(radius: f32) -> Nav {
        let side = ((radius + 4.0) * 4.0) as usize;
        let mut grid = Grid::blank(side, side, Vec2::splat(-(radius + 4.0)));
        for row in 0..side {
            for column in 0..side {
                let away = grid.centre_of(column, row).length();
                if away <= radius {
                    grid.set(column, row, Some(0.25), away <= radius - 0.5, Some(Region::Team(1)));
                }
            }
        }
        grid.place_land(Region::Team(1), Vec2::ZERO);
        Nav::new(Arc::new(grid))
    }

    /// One of the computer's players, in seat 5, with someone of another
    /// team in seat 0 and, if `mate` says where, one of its own in seat 1,
    /// on a map of their own, a frame at a time: where the someone is each
    /// frame is the caller's to say, and so is every stick of dynamite
    /// there is; the bot's rungs go onto the map as a game would lay them.
    struct Rig {
        nav: Nav,
        squad: Squad,
        outfits: Outfits,
        mint: Mint,
        seat: usize,
        now: f32,
        sticks: Vec<Stick>,
        mate: Option<Vec2>,
        rungs: usize,
    }

    impl Rig {
        const FRAME: f32 = 1.0 / 60.0;

        /// `who` standing `at` on `nav`, `now` seconds into a fight.
        fn new(nav: Nav, seed: u64, who: &str, at: Vec2, now: f32) -> Self {
            let mut squad = Squad::new(seed, Skill::MODERATE);
            let seat = 5;
            squad.seat(seat, Vec3::new(at.x, 0.25, at.y), 0.0);
            assert!(squad.cast(seat, who), "nobody is called {who}");
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            Self {
                nav,
                squad,
                outfits,
                mint: Mint::new(0.0, &mut || 0.5),
                seat,
                now,
                sticks: Vec::new(),
                mate: None,
                rungs: 0,
            }
        }

        /// Late in a game, when everyone goes for everyone they can get at.
        fn hunting(nav: Nav, seed: u64, who: &str, at: Vec2) -> Self {
            Self::new(nav, seed, who, at, Skill::MODERATE.hunt_from + 5.0)
        }

        fn bot(&self) -> &Bot {
            self.squad.bot(self.seat).expect("seated")
        }

        fn at(&self) -> Vec2 {
            self.bot().body.at.xz()
        }

        /// Whether the bot has fallen off everything.
        fn fallen(&self) -> bool {
            self.bot().body.at.y < -2.0
        }

        /// One frame, with the someone at `person` if they are there: what
        /// the bot is ordered to do.
        fn step(&mut self, person: Option<Vec2>) -> Order {
            self.now += Self::FRAME;
            let standing = |at: Vec2, person: bool| Seen {
                at: Vec3::new(at.x, 0.25, at.y),
                alive: true,
                in_play: true,
                unseen: false,
                person,
                health: 1.0,
            };
            let mut seen = vec![Seen::default(); ARENA_SEATS];
            if let Some(at) = person {
                seen[0] = standing(at, true);
            }
            if let Some(at) = self.mate {
                seen[1] = standing(at, false);
            }
            seen[self.seat] = Seen {
                at: self.bot().body.at,
                ..standing(Vec2::ZERO, false)
            };
            let purses = [[0u16; 4]; ARENA_SEATS];
            let order = {
                let view = View {
                    elapsed: self.now,
                    nav: &self.nav,
                    seats: &seen,
                    purses: &purses,
                    outfits: &self.outfits,
                    mint: &self.mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &self.sticks,
                };
                self.squad.step(self.seat, &view, Self::FRAME)
            };
            for &(chain, index, at, _) in &order.rungs {
                self.nav.hear(self.seat as u8, chain, index, at);
                self.rungs += 1;
            }
            for stick in &mut self.sticks {
                stick.left -= Self::FRAME;
            }
            order
        }
    }

    #[test]
    fn a_body_jumping_a_gap_goes_off_the_edge_it_would_never_walk_off_and_comes_down_past_it() {
        // A bridge with four planks blown out of it, 5.4 m of nothing.
        let mut nav = bridged();
        let span = two_islands().1;
        let taken = nav.blast(span.rung(6), 3.0);
        assert_eq!(taken.len(), 4, "{taken:?}");
        let gap = nav.gaps()[0];
        assert!(gap.length() < Squad::reach_of_a_jump(RUN_SPEED), "{} m", gap.length());
        let frame = 1.0 / 60.0;
        let run = (gap.b_in - gap.a_in).normalize() * RUN_SPEED;
        // Walked at it, a body stops at the end of what stands.
        let mut walker = Body::standing(Vec3::new(gap.a_in.x, PLANK, gap.a_in.y), 0.0);
        for _ in 0..90 {
            walker.step(&nav, run, frame);
        }
        assert!(walker.free(&nav) && walker.at.x <= gap.a.x + 0.1, "{}", walker.at);
        // Jumping it, it leaves the ground at the end of what stands, and
        // is down again on the far side well inside what stands there.
        let mut jumper = Body::standing(Vec3::new(gap.a_in.x, PLANK, gap.a_in.y), 0.0);
        jumper.leap = true;
        let (mut left_at, mut highest, mut down_at) = (None, 0.0f32, None);
        for _ in 0..90 {
            let before = jumper;
            jumper.step(&nav, run, frame);
            if left_at.is_none() && jumper.lift > 0.0 {
                left_at = Some(before.at.x);
            }
            highest = highest.max(jumper.at.y);
            if left_at.is_some() && down_at.is_none() && jumper.free(&nav) {
                down_at = Some(jumper.at.x);
                break;
            }
        }
        let (left_at, down_at) = (left_at.expect("it jumped"), down_at.expect("it came down"));
        assert!((left_at - gap.a.x).abs() < 0.3, "it left the ground at {left_at}, the edge being at {}", gap.a.x);
        assert!(highest > 1.6, "as high as anyone jumps: {highest}");
        assert!(down_at > gap.b.x + 0.6, "down at {down_at}, the far edge being at {}", gap.b.x);
        // Five planks gone, 6.8 m, a body at a run clears too, by less: a
        // bot jumps no gap it has not a good deal in hand over
        // ([`JUMP_SPARE`]), and this is not one it jumps.
        let mut wide = bridged();
        assert_eq!(wide.blast((span.rung(6) + span.rung(7)) * 0.5, 3.0).len(), 5);
        let hole = wide.gaps()[0];
        assert!(hole.length() > Squad::reach_of_a_jump(RUN_SPEED) && hole.length() < RUN_SPEED * JUMP_SECS);
        let mut close = Body::standing(Vec3::new(hole.a_in.x, PLANK, hole.a_in.y), 0.0);
        close.leap = true;
        let mut down_at = None;
        for _ in 0..90 {
            close.step(&wide, run, frame);
            if close.at.x > hole.a.x + 1.0 && close.free(&wide) {
                down_at = Some(close.at.x);
                break;
            }
        }
        let down_at = down_at.expect("it clears it");
        assert!(down_at - hole.b.x < JUMP_SPARE, "it came down {} m past the far edge", down_at - hole.b.x);
        // Six planks gone is further than a jump goes.
        let mut wider = bridged();
        assert_eq!(wider.blast(span.rung(7), 3.7).len(), 6);
        let hole = wider.gaps()[0];
        let mut gone = Body::standing(Vec3::new(hole.a_in.x, PLANK, hole.a_in.y), 0.0);
        gone.leap = true;
        for _ in 0..180 {
            gone.step(&wider, run, frame);
        }
        assert!(gone.at.y < -10.0, "{}", gone.at);
    }

    /// How high a plank's top is: where a body stands on a bridge.
    const PLANK: f32 = crate::nav::PLANK_TOP;

    /// How many punches a bot called `who` lands in `secs` on someone who
    /// runs round and round a ring `radius` across from the middle of an
    /// island and never hits back, and how near it gets to them at its
    /// nearest.
    fn punches_on_a_runner(who: &str, seed: u64, radius: f32, secs: f32) -> (u32, f32) {
        let mut rig = Rig::hunting(round_island(10.0), seed, who, Vec2::new(-7.0, 0.5));
        let (mut punches, mut nearest) = (0, f32::INFINITY);
        for frame in 0..(secs / Rig::FRAME) as usize {
            let angle = frame as f32 * Rig::FRAME * RUN_SPEED / radius;
            let person = Vec2::from_angle(angle) * radius;
            let order = rig.step(Some(person));
            let away = rig.at().distance(person);
            nearest = nearest.min(away);
            punches += u32::from(order.punch == Some(0) && away <= REACH + BODY + 0.5);
        }
        assert!(!rig.fallen());
        (punches, nearest)
    }

    #[test]
    fn someone_running_round_and_round_is_cut_off_and_not_followed_for_ever() {
        // Round a ring 3.5 m across from its middle and round one 6 m, at
        // a run, for ten seconds: every kind of bot lands punches on them,
        // a few seeds of each together, where one that ran at where they
        // were would follow them round for ever a step behind.
        for radius in [3.5, 6.0] {
            for persona in &PERSONAS {
                let landed: u32 = (0..4).map(|seed| punches_on_a_runner(persona.name, seed, radius, 10.0).0).sum();
                assert!(landed >= 12, "{} landed {landed} punches in four tries on someone running a ring of {radius} m", persona.name);
            }
        }
        // And the best of them land the most.
        let of = |who: &str| (0..6).map(|seed| punches_on_a_runner(who, seed, 3.5, 10.0).0).sum::<u32>();
        let (best, plain) = (of("ISTP"), of("INFP"));
        assert!(best > plain, "the lone wolf landed {best}, the dreamer {plain}");
    }

    #[test]
    fn the_better_a_bot_is_in_a_fight_the_faster_its_punches_come_and_the_more_of_them_on_the_run() {
        // Someone standing still on an island, never hitting back, for
        // twelve seconds: how many punches, and how many thrown running.
        let fight = |who: &str, seed: u64| {
            let mut rig = Rig::hunting(round_island(10.0), seed, who, Vec2::new(-6.0, 0.0));
            let (mut punches, mut running) = (0u32, 0u32);
            for _ in 0..(12.0 / Rig::FRAME) as usize {
                let order = rig.step(Some(Vec2::ZERO));
                if order.punch == Some(0) {
                    punches += 1;
                    running += u32::from(order.running);
                }
            }
            (punches, running)
        };
        let sum = |who: &str| (0..5).map(|seed| fight(who, seed)).fold((0, 0), |all, one| (all.0 + one.0, all.1 + one.1));
        let (best, plain) = (sum("ISTP"), sum("INFP"));
        assert!(persona("ISTP").pvp > 0.9 && persona("INFP").pvp < 0.1);
        // The dreamer as they all were: a punch every 0.8 s or so, once it
        // is there. The lone wolf nearly as fast as punches count, two a
        // second, and most of them thrown running in.
        assert!((45..=75).contains(&plain.0), "the dreamer threw {} in a minute", plain.0);
        assert!(best.0 as f32 > plain.0 as f32 * 1.3, "the lone wolf threw {} against {}", best.0, plain.0);
        assert!(best.1 as f32 > 0.6 * best.0 as f32, "{} of its {} on the run", best.1, best.0);
        assert!((best.1 as f32 / best.0 as f32) > (plain.1 as f32 / plain.0 as f32) + 0.2, "{best:?} against {plain:?}");
        // Every kind is somewhere between, by how good it is.
        assert!(PERSONAS.iter().all(|persona| (0.0..=1.0).contains(&persona.pvp)));
        assert!(PERSONAS.iter().filter(|persona| persona.pvp >= 0.6).count() >= 4, "some are good at it");
        assert!(PERSONAS.iter().filter(|persona| persona.pvp <= 0.25).count() >= 4, "and some are as they all were");
    }

    /// The two islands with two bridges between them, side by side, 7 m
    /// from the middle of the one to the middle of the other: 4.3 m of
    /// nothing between their edges. And how far to each side of the line
    /// between the islands' middles they lie.
    fn two_bridges() -> (Nav, f32) {
        let (mut nav, _) = two_islands();
        let aside = 3.5;
        for (chain, side) in [aside, -aside].into_iter().enumerate() {
            let span = nav.grid().span(Region::Team(1), Region::Middle, side).expect("a span to one side");
            for index in 0..=span.planks() {
                nav.hear(1, chain as u16, index as u16, span.rung(index));
            }
        }
        (nav, aside)
    }

    #[test]
    fn a_bot_jumps_from_one_bridge_to_the_next_after_someone_and_back_after_them() {
        let (nav, aside) = two_bridges();
        let across = 2.0 * aside - 2.0 * crate::nav::PLANK_HALF_WIDTH;
        assert!(across > 4.0 && across < Squad::reach_of_a_jump(RUN_SPEED));
        // Halfway out on the one bridge, with someone halfway out on the
        // other: 7 m off as the crow flies, and 30 m round by the island.
        let (mut jumped, mut went_round) = (0, 0);
        for seed in 0..6 {
            let mut rig = Rig::hunting(nav.clone(), seed, "ISTP", Vec2::new(15.0, aside));
            let mut side = -aside;
            let mut crossings = 0;
            for _ in 0..(12.0 / Rig::FRAME) as usize {
                rig.step(Some(Vec2::new(15.0, side)));
                assert!(!rig.fallen(), "seed {seed}: it fell");
                let at = rig.at();
                // On their bridge and down, having come over the gap and
                // not round by an island: they jump back to the other.
                if (at.y - side).abs() < 1.2 && rig.bot().body.free(&rig.nav) {
                    crossings += 1;
                    side = -side;
                }
                // Back on an island: it went round. It is put back out on
                // the bridge across from them, to try again.
                if rig.nav.grid().height(at).is_some() {
                    went_round += 1;
                    let now = rig.now;
                    rig.squad.stand(rig.seat, Vec3::new(15.0, PLANK, -side), 0.0, now);
                }
            }
            jumped += crossings;
        }
        // Over and back and over again, nearly every time: now and then
        // it thinks better of it and goes round.
        assert!(jumped >= 30, "{jumped} jumps in six tries of twelve seconds");
        assert!(went_round * 5 <= jumped, "round by the island {went_round} times, over the gap {jumped}");
    }

    #[test]
    fn a_bot_gets_out_from_under_a_stick_of_dynamite_and_waits_short_of_one_on_its_way() {
        let span = two_islands().1;
        let middle = span.rung(7);
        // Standing on a bridge, a stick comes down beside it: the quick
        // ones are clear of the blast by the time it blows, on what still
        // stands, whichever team threw it.
        for (who, team) in [("ISTP", 0), ("ESTP", 1), ("ENTJ", 0)] {
            for seed in 0..4 {
                let mut rig = Rig::new(bridged(), seed, who, middle, 200.0);
                rig.sticks.push(Stick {
                    at: middle + Vec2::X * 0.8,
                    left: 0.6 + DYNAMITE_FUSE,
                    team,
                });
                while rig.sticks[0].left > 0.0 {
                    rig.step(None);
                }
                let stick = rig.sticks.remove(0);
                let away = rig.at().distance(stick.at);
                assert!(away > DYNAMITE_RADIUS + BODY, "{who}, seed {seed}: {away} m from it as it blew");
                rig.nav.blast(stick.at, DYNAMITE_RADIUS);
                for _ in 0..60 {
                    rig.step(None);
                }
                assert!(!rig.fallen() && rig.nav.ground(rig.at()).is_some(), "{who}, seed {seed}: it went with the planks");
            }
        }
        // On the land, one of its own team's is nothing to it: it stays
        // where it is. Another team's it gets away from.
        for (team, moves) in [(1, false), (0, true)] {
            let mut rig = Rig::new(bridged(), 3, "ISTP", Vec2::new(1.0, 1.0), 200.0);
            let from = rig.at();
            rig.sticks.push(Stick {
                at: from + Vec2::new(0.8, 0.0),
                left: 1.6,
                team,
            });
            let mut furthest = 0.0f32;
            while rig.sticks[0].left > 0.0 {
                rig.step(None);
                furthest = furthest.max(rig.at().distance(rig.sticks[0].at));
            }
            assert_eq!(furthest > DYNAMITE_RADIUS + BODY, moves, "a stick of team {team}'s: {furthest} m at the furthest");
        }
        // How soon one is out of a blast goes by how good it is at a fight.
        // A stick of another team's thrown from close by to come down right
        // where it stands, in the open: the best of them is clear of it
        // every time, and the worst is still in it now and then, more
        // often than not when it comes down in the middle of a fight with
        // someone.
        let caught = |who: &str, fighting: bool| -> usize {
            (0..80)
                .filter(|&seed| {
                    let at = Vec2::new(-2.0, 1.0);
                    let mut rig = if fighting {
                        Rig::hunting(bridged(), seed, who, at)
                    } else {
                        Rig::new(bridged(), seed, who, at, 200.0)
                    };
                    let foe = fighting.then_some(at + Vec2::new(0.0, 1.6));
                    // (A moment of it first, to have the fight begun.)
                    for _ in 0..30 {
                        rig.step(foe);
                    }
                    rig.sticks.push(Stick {
                        at: rig.at(),
                        left: 0.55 + DYNAMITE_FUSE,
                        team: 0,
                    });
                    while rig.sticks[0].left > 0.0 {
                        rig.step(foe);
                    }
                    rig.at().distance(rig.sticks[0].at) <= DYNAMITE_RADIUS + BODY
                })
                .count()
        };
        assert_eq!((caught("ISTP", false), caught("ISTP", true)), (0, 0));
        assert_eq!(caught("ESTJ", false), 0);
        let (worst, worst_fighting) = (caught("INFP", false), caught("INFP", true));
        assert!((12..=48).contains(&worst), "the worst of them was caught {worst} times in eighty");
        assert!(worst_fighting > worst && worst_fighting >= 40, "and {worst_fighting} times in a fight");
        let middling = caught("ENFJ", true);
        assert!(middling > 0 && middling < worst_fighting, "one in the middle, in a fight: {middling} times in eighty");
        // (As it stood on 2026-11-13, of eighty, in the open and in a
        // fight: ISTP, ESTP, ENTJ and ESTJ never; ENTP 0 and 1, INTJ 0 and
        // 4, ESFP 0 and 1, ENFP 1 and 10, ENFJ 0 and 14, ISTJ 2 and 15,
        // INFJ 5 and 14, ISFP 12 and 30, INTP 12 and 36, ESFJ 14 and 31,
        // ISFJ 17 and 49, INFP 34 and 55.)
        // After someone on the far land, with a stick lying on the bridge
        // between: it waits short of the blast, and does not run into it.
        let far = Vec2::new(30.0, 0.5);
        let mut rig = Rig::hunting(bridged(), 2, "ENTJ", Vec2::new(4.0, 0.0));
        rig.sticks.push(Stick {
            at: span.rung(8),
            left: 0.7 + DYNAMITE_FUSE,
            team: 0,
        });
        while rig.sticks[0].left > 0.0 {
            rig.step(Some(far));
            assert!(rig.at().distance(rig.sticks[0].at) > DYNAMITE_RADIUS, "it ran into the blast: {}", rig.at());
        }
        assert!(rig.at().x < span.rung(8).x, "and it is on its own side of it");
    }

    /// Someone comes at the base of `who` and steps back onto their
    /// bridge, five times over: three seconds on its island, seven on the
    /// bridge three and a half metres out. How patient the bot was thrown;
    /// at which coming it first stood its ground; and whether, having not
    /// yet had enough of them, it ever went back to what it was doing with
    /// them still there.
    fn baited(who: &str, seed: u64) -> (u8, Option<u8>, bool) {
        let mut rig = Rig::new(bridged(), seed, who, Vec2::new(-3.0, 0.5), 200.0);
        let patience = rig.bot().patience();
        let (on, off) = (Vec2::new(3.5, 0.0), Vec2::new(9.5, 0.0));
        let mut went_back = false;
        for coming in 1..=5u8 {
            for _ in 0..180 {
                rig.step(Some(on));
                assert!(!rig.bot().stands_its_ground(), "they are on its island: it is at them");
            }
            assert_eq!(rig.bot().vigil().map(|vigil| vigil.comings), Some(coming), "{who}, seed {seed}");
            let mut stood = false;
            for _ in 0..420 {
                rig.step(Some(off));
                assert!(!rig.fallen(), "{who}, seed {seed}: it went off its island after them");
                stood |= rig.bot().stands_its_ground();
                went_back |= !stood && !matches!(rig.bot().goal(), Goal::Guard | Goal::Defend);
            }
            if stood {
                return (patience, Some(coming), went_back);
            }
        }
        (patience, None, went_back)
    }

    #[test]
    fn someone_who_keeps_coming_at_a_bot_s_base_is_stood_against_sooner_or_later_by_who_it_is() {
        // Where it stands: on its island, between what it keeps and them,
        // so far from them and so far in from where their bridge lands.
        let nav = bridged();
        let spot = guard_spot(&nav, Region::Team(1), Vec2::ZERO, Vec2::new(9.5, 0.0), 3.5, 1.0).expect("a place to stand");
        assert!((spot.x - 4.5).abs() < 0.35 && spot.y.abs() < 1e-3, "{spot}");
        assert_eq!(guard_spot(&nav, Region::Team(1), Vec2::ZERO, Vec2::new(0.6, 0.0), 3.5, 1.0), None);
        // Each of them stands its ground at the coming it was thrown the
        // patience for, the first, the second or the third, and not
        // before: until then it looks after them a moment and goes back
        // to what it was doing. Nobody's patience is the same every time,
        // and one kind is not as patient as another.
        let mut mean = std::collections::HashMap::new();
        for persona in PERSONAS.iter() {
            let mut stood_at = [0u32; 4];
            for seed in 0..24 {
                let (patience, stood, went_back) = baited(persona.name, seed);
                assert!((1..=3).contains(&patience));
                assert_eq!(stood, Some(patience), "{}, seed {seed}", persona.name);
                assert_eq!(went_back, patience > 1, "{}, seed {seed}: patient for {patience}", persona.name);
                stood_at[usize::from(patience)] += 1;
            }
            assert!(stood_at.iter().filter(|times| **times > 0).count() >= 2, "{}: always at the same coming, {stood_at:?}", persona.name);
            let comings = (stood_at[1] + 2 * stood_at[2] + 3 * stood_at[3]) as f32 / 24.0;
            assert!((comings - persona.patience).abs() < 0.5, "{}: at {comings} on average, for {}", persona.name, persona.patience);
            // (As it stood on 2026-11-14, of twenty-four: the defender at
            // the first coming 21 times and the second 3; the mastermind
            // 2, 21 and 1 at the first, the second and the third; the
            // free spirit at the second 8 times and the third 16.)
            mean.insert(persona.name, comings);
        }
        assert!(mean["ISFJ"] + 1.0 < mean["INFP"] && mean["INFJ"] + 1.0 < mean["ENFP"], "{mean:?}");
        assert!(PERSONAS.iter().all(|persona| (1.0..=3.0).contains(&persona.patience)));
    }

    #[test]
    fn a_bot_standing_its_ground_keeps_between_them_and_its_base_until_they_are_really_gone() {
        let (on, off) = (Vec2::new(3.5, 0.0), Vec2::new(9.5, 0.0));
        for (who, seed) in [("ISFJ", 1), ("INFJ", 4), ("ISTJ", 2), ("ESTJ", 7), ("ISFJ", 9)] {
            let mut rig = Rig::new(bridged(), seed, who, Vec2::new(-3.0, 0.5), 200.0);
            // Baited until it stands.
            let mut comings = 0;
            while !rig.bot().stands_its_ground() {
                comings += 1;
                assert!(comings <= 3, "{who}, seed {seed}");
                for _ in 0..180 {
                    rig.step(Some(on));
                }
                for _ in 0..120 {
                    rig.step(Some(off));
                }
            }
            // With them a good way down their bridge, further than it
            // goes at anyone from where it stands and nearer than gone,
            // it is on its island on their side of what it keeps, turned
            // to them; and it does not go back to anything else.
            let down = Vec2::new(22.5, 0.0);
            for frame in 0..(6.0 / Rig::FRAME) as usize {
                rig.step(Some(down));
                if frame > 150 && rig.bot().stands_its_ground() && rig.bot().foe.is_none() {
                    let at = rig.at();
                    assert!(at.x > 0.8 && at.x < 5.6 && at.y.abs() < 2.5, "{who}, seed {seed}: it stands at {at}");
                    assert_eq!(rig.bot().goal(), Goal::Guard);
                }
            }
            // They come back onto its island: it is at them at once, and
            // it has counted another coming.
            let mut struck = false;
            for _ in 0..(4.0 / Rig::FRAME) as usize {
                struck |= rig.step(Some(on)).punch == Some(0);
            }
            assert!(struck, "{who}, seed {seed}: they came back and it did nothing");
            assert_eq!(rig.bot().vigil().map(|vigil| vigil.comings), Some(comings + 1));
            // They step back, and it stands again, having had enough long
            // since; they go for good, and within the while it takes to
            // believe it, it stands there no longer. It has not forgotten
            // them: the next time they come and step back, it stands at
            // once, however patient it was thrown.
            for _ in 0..120 {
                rig.step(Some(off));
            }
            assert!(rig.bot().stands_its_ground(), "{who}, seed {seed}");
            let mut stood_down = None;
            for frame in 0..(12.0 / Rig::FRAME) as usize {
                rig.step(None);
                if !rig.bot().stands_its_ground() {
                    stood_down = Some(frame as f32 * Rig::FRAME);
                    break;
                }
            }
            let after = stood_down.expect("it never stood down");
            assert!(after > 1.0 && after < VIGIL_CALM.1 + 2.0, "{who}, seed {seed}: after {after} s");
            for _ in 0..180 {
                rig.step(Some(on));
            }
            for _ in 0..120 {
                rig.step(Some(off));
            }
            assert!(rig.bot().stands_its_ground(), "{who}, seed {seed}: it had forgotten them");
            // Out on their bridge near enough for it to go at them, and
            // then away to the far land: it goes no further after them
            // than it means to from where it stands, and is on its own
            // island again, whatever it was doing to them on the bridge.
            for _ in 0..(5.0 / Rig::FRAME) as usize {
                rig.step(Some(off));
                assert!(!rig.fallen(), "{who}, seed {seed}");
            }
            for _ in 0..(10.0 / Rig::FRAME) as usize {
                rig.step(Some(Vec2::new(28.0, 0.0)));
                assert!(!rig.fallen(), "{who}, seed {seed}");
            }
            assert!(rig.at().x < 6.3, "{who}, seed {seed}: it is after them still, at {}", rig.at());
        }
    }

    #[test]
    fn a_bot_does_not_stand_its_ground_for_ever_and_what_it_does_then_is_not_the_same_twice() {
        // Someone who has had a bot stand its ground, and then stays out
        // on their bridge where it does not go at them from where it
        // stands: for a minute and a half. Sooner or later it has stood
        // there long enough, and either goes out at them or lets them be
        // and goes back to what it was doing; which, and when, differs
        // from one time to the next, and the good at a fight go out at
        // them oftener than the rest.
        let (on, off, down) = (Vec2::new(3.5, 0.0), Vec2::new(9.5, 0.0), Vec2::new(22.5, 0.0));
        let outcomes = |who: &str| -> (u32, u32, Vec<f32>) {
            let (mut went_out, mut let_be, mut whens) = (0, 0, Vec::new());
            for seed in 0..30 {
                let mut rig = Rig::new(bridged(), seed, who, Vec2::new(-3.0, 0.5), 200.0);
                for _ in 0..4 {
                    for _ in 0..180 {
                        rig.step(Some(on));
                    }
                    for _ in 0..90 {
                        rig.step(Some(off));
                    }
                    if rig.bot().stands_its_ground() {
                        break;
                    }
                }
                assert!(rig.bot().stands_its_ground(), "{who}, seed {seed}");
                let (mut out, mut back) = (false, false);
                for frame in 0..(90.0 / Rig::FRAME) as usize {
                    rig.step(Some(down));
                    assert!(!rig.fallen(), "{who}, seed {seed}");
                    // Out on the bridge after them; or about its island
                    // again with nobody in mind to stand against.
                    let now_out = rig.at().x > 8.0;
                    let now_back = !rig.bot().stands_its_ground() && rig.bot().goal() != Goal::Guard;
                    if (now_out && !out) || (now_back && !back) {
                        whens.push(frame as f32 * Rig::FRAME);
                    }
                    out |= now_out;
                    back |= now_back;
                }
                assert!(out || back, "{who}, seed {seed}: it stood there for a minute and a half");
                went_out += u32::from(out);
                let_be += u32::from(back && !out);
            }
            (went_out, let_be, whens)
        };
        let (bold_out, bold_be, whens) = outcomes("ISTP");
        let (meek_out, meek_be, _) = outcomes("INFP");
        // (As it stood on 2026-11-14, of thirty: the lone wolf out at them
        // 19 times and letting them be 11, the dreamer 4 and 26.)
        assert!(bold_out > 0 && bold_be > 0, "the lone wolf: out at them {bold_out} times, let them be {bold_be}");
        assert!(meek_out > 0 && meek_be > 0, "the dreamer: out at them {meek_out} times, let them be {meek_be}");
        assert!(bold_out > meek_out, "{bold_out} against {meek_out}");
        // And not on the stroke of anything: the first thing it did came
        // anywhere from a quarter of a minute on to well over half of one.
        let (soonest, latest) = whens.iter().fold((f32::INFINITY, 0.0f32), |(low, high), when| (low.min(*when), high.max(*when)));
        assert!(soonest < 20.0 && latest > 30.0, "between {soonest} and {latest} s");
    }

    #[test]
    fn a_hole_blown_in_a_bridge_is_jumped_if_it_is_short_and_mended_if_it_is_not() {
        let span = two_islands().1;
        let far = Vec2::new(30.0, 0.5);
        // Four planks gone, 5.4 m: after someone on the far land, a bot
        // runs up to the hole and jumps it.
        let mut over = 0;
        for seed in 0..5 {
            let mut rig = Rig::hunting(bridged(), seed, "ESTJ", Vec2::new(2.0, 0.0));
            assert_eq!(rig.nav.blast(span.rung(7), DYNAMITE_RADIUS).len(), 4);
            let gap = rig.nav.gaps()[0];
            for _ in 0..(8.0 / Rig::FRAME) as usize {
                rig.step(Some(far));
                assert!(!rig.fallen(), "seed {seed}: it fell into the hole");
                if rig.at().x > gap.b.x + 1.0 && rig.bot().body.free(&rig.nav) {
                    over += 1;
                    break;
                }
            }
            assert_eq!(rig.rungs, 0, "it laid planks where it could jump");
        }
        assert_eq!(over, 5, "over the hole {over} times of five");
        // Five planks gone, 6.8 m, is further than it jumps. With nobody
        // it can get at, and the metres for it, it lays planks across the
        // hole, from the end of what stands on its side to the end of what
        // stands on the other: a few planks, not a bridge.
        let mut rig = Rig::new(bridged(), 4, "ISTJ", Vec2::new(2.0, 0.0), 400.0);
        let hole = (span.rung(7) + span.rung(8)) * 0.5;
        assert_eq!(rig.nav.blast(hole, DYNAMITE_RADIUS).len(), 5);
        let gap = rig.nav.gaps()[0];
        assert!(!rig.nav.joined(Region::Team(1), Region::Middle));
        // Nothing to lay them with yet: it stays on its own side.
        for _ in 0..(4.0 / Rig::FRAME) as usize {
            rig.step(Some(far));
            assert!(!rig.fallen() && rig.at().x < gap.a.x + 0.3, "{}", rig.at());
        }
        assert_eq!(rig.rungs, 0);
        // 25 m, as the shop sells them.
        let now = rig.now;
        rig.squad.bought(rig.seat, Ware::Bridge, now);
        let mut mended = None;
        for _ in 0..(25.0 / Rig::FRAME) as usize {
            rig.step(Some(far));
            assert!(!rig.fallen(), "it fell mending the hole");
            if rig.nav.joined(Region::Team(1), Region::Middle) {
                mended = Some(rig.now - now);
                break;
            }
        }
        let took = mended.expect("the hole was not mended");
        assert!(took < 20.0, "{took} s");
        // (Joined as its last plank but one comes within a body's reach
        // of what stands: it lays the last, and walks to the end of it.)
        for _ in 0..(3.0 / Rig::FRAME) as usize {
            rig.step(Some(far));
        }
        assert!(!rig.fallen());
        // One rung to begin on and a plank's worth after it for each plank
        // of the hole, and one over for the ends: not the fifteen of a
        // bridge begun again.
        assert!((6..=9).contains(&rig.rungs), "{} rungs", rig.rungs);
        let left = rig.bot().metres();
        assert!(left > 14.0 && left < 19.5, "{left} m left of 25");
        // Its planks begin inside what stands on the near side and end
        // inside what stands on the far side, straight along the bridge.
        let laid = rig.nav.rungs(rig.seat as u8, 0);
        assert!(laid[0].x < gap.a.x && laid[0].x > gap.a.x - 1.0 && laid[0].y.abs() < 0.3, "{:?}", laid[0]);
        let last = laid[laid.len() - 1];
        assert!(last.x > gap.b.x && last.x < gap.b.x + 1.0 && last.y.abs() < 0.3, "{last:?}");
    }

    #[test]
    fn a_bot_comes_down_past_someone_on_the_edge_of_a_hole_and_waits_for_someone_coming_over_it() {
        let span = two_islands().1;
        let holed = || {
            let mut nav = bridged();
            assert_eq!(nav.blast(span.rung(7), DYNAMITE_RADIUS).len(), 4);
            let gap = nav.gaps()[0];
            (nav, gap)
        };
        // Someone standing on the very edge of what is left past a hole
        // of four planks. A bot after them jumps it and comes down on the
        // planks behind them, whoever it is, and never in the hole: for a
        // night the place to come down was where they stood, the edge
        // itself, the jump was over a step short of it, in the air, and
        // what it then steered for was a square with nothing under it.
        let mut over = 0;
        for who in ["ESTP", "ISTP", "ENTJ", "INFP"] {
            for seed in 0..6 {
                let (nav, gap) = holed();
                let mut rig = Rig::hunting(nav, seed, who, Vec2::new(2.0, 0.0));
                for _ in 0..(9.0 / Rig::FRAME) as usize {
                    rig.step(Some(gap.b));
                    assert!(!rig.fallen(), "{who}, seed {seed}: it fell into the hole");
                    if rig.at().x > gap.b.x - 0.1 && rig.bot().body.free(&rig.nav) {
                        over += 1;
                        break;
                    }
                }
            }
        }
        assert!(over >= 20, "over the hole {over} times of twenty-four");
        // Someone running at it from the far land, off the edge and over
        // the hole: it lets them come. It is not jumping the hole itself
        // while they are on their way, to pass them in the air.
        for seed in 0..10 {
            let (nav, gap) = holed();
            let mut rig = Rig::hunting(nav, seed, "ESTP", gap.a_in - Vec2::X * 2.0);
            // From the far land, 16 m past the hole, to a step in from the
            // edge on the bot's side: a little over two seconds of it.
            let from = gap.b.x + 16.0;
            let frames = ((from - gap.a.x + 0.8) / (RUN_SPEED * Rig::FRAME)) as usize;
            let mut was = false;
            for frame in 0..frames {
                let run = frame as f32 * RUN_SPEED * Rig::FRAME;
                rig.step(Some(Vec2::new(from - run, 0.0)));
                assert!(!rig.fallen(), "seed {seed}: it fell");
                // (For the first quarter of a second it cannot know which
                // way they are going.)
                let leaping = rig.bot().leaping();
                assert!(frame < 15 || was || !leaping, "seed {seed}: it jumped at someone on their way over, {run} m into their run");
                was = leaping;
            }
            // And they are met where it stood, on its own side.
            assert!(rig.at().x < gap.a.x + 0.1, "seed {seed}: it is at {}", rig.at());
        }
    }

    #[test]
    fn a_bot_on_the_land_throws_dynamite_at_whoever_comes_over_a_bridge_and_at_nobody_else() {
        // Someone coming over the bridge at a walk, 9 m off and nearer, a
        // few minutes into a game, with the bot minding its own island and
        // a stick of dynamite on a button.
        let thrown = |at: Vec2, mate: Option<Vec2>, holds: u8, seed: u64, walks: f32| {
            let mut rig = Rig::new(bridged(), seed, "INTP", at, 200.0);
            rig.outfits.kits[5] = Kit::default().holding(Utility::Dynamite, holds);
            rig.mate = mate;
            let mut person = Vec2::new(15.0, 0.0);
            for _ in 0..(2.5 / Rig::FRAME) as usize {
                person.x -= walks * Rig::FRAME;
                let order = rig.step(Some(person));
                if let Some((heading, far)) = order.blast {
                    let way = Vec2::new(-heading.sin(), -heading.cos());
                    return Some((rig.at() + way * far, far, person, rig.nav.clone()));
                }
            }
            None
        };
        let home = Vec2::new(4.0, 0.3);
        for seed in 0..5 {
            let (down, far, person, nav) = thrown(home, None, 1, seed, 3.0).expect("it never threw it");
            // Onto the bridge, at where they are going, no further than
            // one can be thrown.
            assert!((DYNAMITE_NEAREST..=DYNAMITE_FURTHEST).contains(&far), "{far} m");
            assert!(out_over_the_void(&nav, down), "it comes down off the bridge, at {down}");
            assert!(down.x <= person.x + 0.5 && down.x > 6.0, "at {down}, with them at {person}");
        }
        // Not with none to throw; nor at where one of its own stands on
        // the planks that would go.
        assert!(thrown(home, None, 0, 1, 3.0).is_none());
        assert!(thrown(home, Some(Vec2::new(9.0, 0.0)), 1, 1, 3.0).is_none());
        // From out on a bridge itself, only at what is well clear of
        // where it stands: at someone standing 7 m along it, and not at
        // someone coming at it who is nearly on it.
        let out = Vec2::new(8.0, 0.0);
        let (down, far, _, _) = thrown(out, None, 1, 1, 0.0).expect("it never threw it from the bridge");
        assert!(far > DYNAMITE_RADIUS + 1.9 && down.distance(Vec2::new(15.0, 0.0)) < 0.5, "{far} m, at {down}");
        assert!(thrown(Vec2::new(11.0, 0.0), None, 1, 1, 2.0).is_none());
    }
}
