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
    AIR_DRAG, BLIND_SIGHT, BOT_PUNCH_EVERY, BRIDGE_A_BUY, BUILDER_SPEED, DRINK_SECS, DRINK_SLOW, FAST, FIREBALL_RANGE,
    GROUND_DRAG, HUNT_RADIUS, KNOCK_BACK, KNOCK_UP, Mint, Outfits, REACH, RUN_SPEED, SPRINT_BONUS, price,
};
use crate::nav::{BODY, Nav, Region, Span};
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

/// A bot's body: where it is, which way it faces, and what a knock is
/// carrying it by, across the ground and upward.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Body {
    pub at: Vec3,
    /// Its turn about the upright, in radians: -Z turned by this is ahead.
    pub yaw: f32,
    pub shove: Vec2,
    pub lift: f32,
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
            // In the air: over the void, wherever it steers; over ground,
            // only on over ground, and not out of where a body walks.
            let over_void = nav.ground(here).is_none();
            let open = |to: Vec2| over_void || ((nav.walkable_at(to) || astray) && nav.ground(to).is_some());
            if let Some(to) = tries.into_iter().find(|&to| to != here && open(to)) {
                self.at.x = to.x;
                self.at.z = to.y;
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
    /// What it puts on its second button, as odds: a fireball, a speed
    /// potion, an invisibility potion, a healing potion.
    pub button: [f32; 4],
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
    /// What it puts on its second button this life, by its odds.
    fn fancies(&self, dice: &mut Dice) -> Utility {
        let kinds = [Utility::Fireball, Utility::Speed, Utility::Invisibility, Utility::Healing];
        kinds[pick(dice, &self.button)]
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
        button: [0.5, 0.35, 0.1, 0.05],
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
        button: [0.15, 0.2, 0.6, 0.05],
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
        button: [0.4, 0.4, 0.05, 0.15],
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
        button: [0.45, 0.2, 0.3, 0.05],
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
        button: [0.2, 0.3, 0.05, 0.45],
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
        button: [0.2, 0.55, 0.2, 0.05],
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
        button: [0.35, 0.25, 0.0, 0.4],
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
        button: [0.1, 0.15, 0.05, 0.7],
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
        button: [0.3, 0.3, 0.1, 0.3],
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
        button: [0.4, 0.2, 0.25, 0.15],
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
        button: [0.1, 0.1, 0.1, 0.7],
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
        button: [0.05, 0.25, 0.3, 0.4],
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
        button: [0.15, 0.1, 0.0, 0.75],
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
        button: [0.05, 0.05, 0.0, 0.9],
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
        button: [0.35, 0.35, 0.2, 0.1],
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
        button: [0.1, 0.4, 0.3, 0.2],
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
}

/// A bridge a bot is laying: the span, which of its bridges it is, and
/// which rung comes next.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Laying {
    span: Span,
    from: Region,
    to: Region,
    chain: u16,
    next: usize,
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
    /// Until when it is drinking a potion.
    drinking: Option<f32>,
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
    /// What it puts on its second button this life; and, on a raid,
    /// whether it is there for the anchor.
    fancy: Utility,
    for_anchor: bool,
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
}

impl Bot {
    fn new(seat: usize, body: Body, dice: &mut Dice, persona: &'static Persona) -> Self {
        let temper = Temper {
            bold: (persona.bold * (1.0 + 0.1 * dice.bell())).clamp(0.5, 1.6),
            bouncy: (persona.bouncy + 0.1 * dice.bell()).clamp(0.0, 1.0),
            weave: (persona.weave * (1.0 + 0.15 * dice.bell())).clamp(0.3, 2.0),
            shoves: persona.shoves,
            minds: persona.minds,
        };
        let fancy = persona.fancies(dice);
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
            fancy,
            for_anchor: false,
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
        }
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
    /// This potion, off its button, is drunk.
    pub drink: Option<Utility>,
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
        // Another life, and perhaps another thing for its second button.
        bot.fancy = bot.persona.fancies(&mut self.dice);
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
        bot.drinking = None;
        bot.foe = None;
        bot.duel = None;
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
        let (reel, grip) = self.reeling();
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.body.knock(from, running, 1.0);
            bot.reel = Some(reel);
            bot.grip = grip;
        }
    }

    /// How long a knock landing now carries a bot before it answers, and
    /// how much of its pace it then steers by in the air.
    fn reeling(&mut self) -> (f32, f32) {
        let skill = self.skill;
        (self.dice.around(skill.reel.0, skill.reel.1), self.dice.around(skill.grip.0, skill.grip.1))
    }

    /// The bot in `seat` has been knocked by this much, across the ground
    /// and upward, as whoever runs the game has worked it out for itself:
    /// a game played offline, where a knock is the file's numbers and
    /// lands on the body the device draws. On top of half of whatever
    /// already carried it, as [`Body::knock`] has it.
    pub fn shoved(&mut self, seat: usize, across: Vec2, up: f32) {
        let (reel, grip) = self.reeling();
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.body.shove = bot.body.shove * 0.5 + across;
            bot.body.lift = (bot.body.lift * 0.5 + up).min(up.max(0.0));
            bot.reel = Some(reel);
            bot.grip = grip;
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
        }
        self.spots = Some(spots);
        spots
    }

    /// One frame of the bot in `seat`: it thinks, if it is time; it acts
    /// on what it thought; its body is moved; and what the game is to
    /// carry out of it is given back.
    pub fn step(&mut self, seat: usize, view: &View, dt: f32) -> Order {
        let mut order = Order::default();
        let Some(mut bot) = self.bots.get_mut(seat).and_then(Option::take) else {
            return order;
        };
        let me = view.seats.get(seat).copied().unwrap_or_default();
        if me.health < bot.health_was - 1e-3 {
            bot.hurt_at = view.elapsed;
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
        let can_go_home = nav.connected(here, spots.heart);
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

        // Hurt, with someone after it: away, until it has its health back.
        let pressed = nearest_foe < 10.0 || now - bot.hurt_at < 3.0;
        let mortal = view.anchors[team] <= 0.0;
        // The bolder, the longer it stays in.
        let give_way = (if mortal { skill.careful_below } else { skill.retreat_below }) + (1.0 - bot.temper.bold) * 0.15;
        if me.health < give_way && pressed {
            offers.push((0.95, Goal::Retreat));
        } else if bot.goal == Goal::Retreat && me.health < skill.recovered && nearest_foe < 16.0 {
            offers.push((0.9, Goal::Retreat));
        }

        // Someone of another team on its island, standing on the land of
        // it: home, to see them off.
        let intruder = foes.iter().any(|foe| {
            let at = foe.at.xz();
            nav.grid().region(at) == Some(home) && !nav.on_plank(at) && foe.at.y > -1.0
        });
        if intruder && can_go_home {
            offers.push((if on_home || keeper { 0.88 } else { 0.6 }, Goal::Defend));
        }

        // The bridge it means to lay next, and the metres it is short of
        // for it.
        let span = self.next_span(bot, view, runner, with_person);
        // What is left of one it is part way along; the whole of a new
        // one, and a little over, for the bends it will have.
        let short = span.map_or(0.0, |(_, _, span)| {
            let laid = bot.laying.map_or(0, |laying| laying.next.saturating_sub(1));
            let bends = if bot.laying.is_some() { 1.0 } else { MOST_BENT };
            let left = span.length() * bends * (1.0 - laid as f32 / span.planks() as f32);
            (left + 0.5 - bot.metres).max(0.0)
        });
        if let Some((from, to, span)) = span
            && short == 0.0
            && nav.connected(here, span.from)
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
        let list = shopping(bot.persona, bot.fancy, buys_as, alone, now, &purse, &kit, &upgrades, short);
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
        // On its pad for them, a person of its team there or not: while
        // they are, what it makes is theirs ([`Squad::wants`]).
        if saving && can_go_home {
            let stock = view.mint.held(team) + view.mint.held(ARENA_TEAMS + team);
            offers.push((if stock > 0 { 0.52 } else { 0.48 }, Goal::Collect));
        }

        // Out to a pad for what lies on it: the amethyst in the middle,
        // and the sapphire on the smaller islands. The keeper stays home,
        // unless it is alone.
        // Its own: the amethyst pad toward its island, and the two smaller
        // islands beside it. The rest are other teams' to go to, until
        // late.
        let quiet = nearest_foe > 40.0;
        let late = now >= skill.keeper_raids_from;
        if runner || (with_person && quiet && now > 120.0) {
            let base = if runner { 0.5 } else { 0.3 };
            for (pad, &at) in AMETHYST_PADS.iter().enumerate() {
                let stock = view.mint.held(12 + pad);
                if stock > 0 && (pad == team || late) && nav.connected(here, at) {
                    let score = base + 0.06 + 0.08 * stock.min(2) as f32 - here.distance(at) / 300.0;
                    offers.push((score, Goal::Fetch(at, Item::Amethyst)));
                }
            }
            let beside = beside(team as u8);
            for (pad, &at) in SAPPHIRE_PADS.iter().enumerate() {
                let stock = view.mint.held(8 + pad);
                if stock > 0 && (beside.contains(&(pad as u8)) || late) && nav.connected(here, at) {
                    let score = base + 0.07 * stock.min(4) as f32 - here.distance(at) / 300.0;
                    offers.push((score, Goal::Fetch(at, Item::Sapphire)));
                }
            }
        }

        // The anchors gone, everyone goes for everyone: whoever of another
        // team is nearest that it can get at.
        let in_play = |team: usize| (0..ARENA_SEATS).any(|other| usize::from(arena_team(other)) == team && view.seats.get(other).is_some_and(|seen| seen.in_play));
        if now >= skill.hunt_from && foes.iter().any(|foe| nav.connected(here, foe.at.xz())) {
            offers.push((0.8, Goal::Hunt));
        }

        // A raid: to another team's island, armed and whole, when its
        // turn comes. The one that goes out goes; the keeper with it,
        // late on; a person's teammate never by itself.
        // Sooner or later than the rest, by who it is.
        let rested = now >= bot.rest_until && me.health >= 0.7 && kit.sword > 0;
        if now >= skill.raid_from + bot.persona.eager && now < skill.hunt_from && rested && !with_person {
            let others = (0..ARENA_TEAMS).filter(|&other| other != team && in_play(other)).count();
            for victim in (0..ARENA_TEAMS).filter(|&victim| victim != team && in_play(victim)) {
                let mine = matches!(bot.goal, Goal::Raid(on) if usize::from(on) == victim);
                // Along with its teammate, unless it is one for staying
                // at home; or, the one that goes out, by its own choice.
                let along = mate_raids == Some(victim) && (runner || (now >= skill.keeper_raids_from && bot.persona.raids > -0.05));
                if !mine && !along && !runner {
                    continue;
                }
                // One team's raid on a team at a time, a rest after each,
                // and a team with a person on it no more than its share.
                let theirs = self.raids[victim].iter().all(|&raider| usize::from(arena_team(raider)) == team);
                let people = (0..ARENA_SEATS).any(|other| usize::from(arena_team(other)) == victim && view.seats.get(other).is_some_and(|seen| seen.person && seen.in_play));
                let share = (self.raided.1 + 1) as f32 <= skill.people_share * (self.raided.0 + 1) as f32 + 1.0;
                let allowed = mine || along || (theirs && now >= self.spared_until[victim] && (!people || others < 2 || share));
                let their_heart = self.spots(nav)[victim].heart;
                if !allowed || !nav.connected(here, their_heart) {
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
        bot.set_goal(best, now);
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
        let wanted = if with_person {
            // Beside a person: the bridge to the middle once they have
            // had the time to build it, and later one to a smaller island.
            if !to_middle && now > 40.0 {
                (home, Region::Middle)
            } else if to_middle && !to_sapphire && now > 120.0 {
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
        if self.claims.iter().any(|&(from, to, by)| (from, to) == wanted && by != bot.seat) {
            return None;
        }
        // Beside whatever was begun there and left; and not on the very
        // line between the two lands' middles, but a little to one side of
        // it or the other, by the bot and by the bridge.
        let tried = self.begun.iter().find(|begun| (begun.0, begun.1) == wanted).map_or(0, |begun| begun.2);
        let off = ((bot.seat * 31 + usize::from(bot.chains) * 17 + self.salt * 7) % 13) as f32 / 6.0 - 1.0;
        let aside = [0.0, 2.9, -2.9][usize::from(tried) % 3] + off * 1.4;
        let span = nav.grid().span(wanted.0, wanted.1, aside).or_else(|| nav.grid().span(wanted.0, wanted.1, 0.0))?;
        Some((wanted.0, wanted.1, span))
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
        if now < bot.reel_until {
            return Vec2::ZERO;
        }
        let ground = nav.ground(here);
        let in_air = bot.body.lift != 0.0 || ground.is_none_or(|ground| (bot.body.at.y - ground).abs() > 0.02);
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
        if let Some(until) = bot.drinking {
            if now >= until {
                bot.drinking = None;
                // The one it has: a bot keeps to one kind of thing a life.
                order.drink = kit.potion();
            } else {
                order.drinking = true;
                pace *= DRINK_SLOW;
            }
        }

        // Faster while a speed potion lasts.
        if kit.fast(now) {
            pace *= FAST;
        }
        // A potion for the road, with nobody near enough to spill it: a
        // speed potion as it sets out on a raid or a hunt, and an
        // invisibility potion as it comes up to the island it is raiding,
        // to walk onto it unseen.
        if let Some(potion @ (Utility::Speed | Utility::Invisibility)) = kit.potion()
            && bot.drinking.is_none()
            && bot.body.free(nav)
        {
            let nearest = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| seen.alive && arena_team(other) != arena_team(seat))
                .map(|(_, seen)| seen.at.xz().distance(here))
                .fold(f32::INFINITY, f32::min);
            let due = match (potion, bot.goal) {
                (Utility::Speed, Goal::Raid(_) | Goal::Hunt) => !kit.fast(now),
                (Utility::Invisibility, Goal::Raid(victim)) => {
                    !kit.unseen(now) && here.distance(ARENA_ISLANDS[usize::from(victim) % ARENA_ISLANDS.len()]) < 62.0
                }
                _ => false,
            };
            if due && nearest > 9.0 {
                bot.drinking = Some(now + DRINK_SECS);
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
                    let odds = if bot.role == Role::Keeper { [5.0, 1.5, 1.5, 1.0, 1.0] } else { [1.0, 3.0, 2.0, 1.5, 1.5] };
                    let about = Vec2::from_angle(self.dice.roll() * std::f32::consts::TAU) * self.dice.between(0.0, 1.6);
                    let to = places[pick(&mut self.dice, &odds)] + about;
                    let to = nav.nearest_walkable(to).filter(|to| nav.connected(here, *to)).unwrap_or(spots.heart);
                    bot.loiter = Some((to, now + self.dice.around(2.5, 8.0)));
                }
                let to = bot.loiter.map_or(spots.heart, |(to, _)| to);
                self.walk_to(bot, view, to, pace * 0.6, 2.0)
            }
            Goal::Collect => self.walk_to(bot, view, spots.pad, pace, 2.0),
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
                if kit.held(Utility::Healing) > 0 && bot.drinking.is_none() && nearest > 6.0 {
                    bot.drinking = Some(now + DRINK_SECS);
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
                    .filter(|&at| nav.connected(here, at))
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

        let mut struck = false;
        if away <= REACH + BODY - 0.1 {
            let since = *bot.in_reach_since.get_or_insert(now);
            if now - since >= bot.windup && now >= bot.punch_at {
                // Not to the beat: a little sooner, a little later.
                bot.punch_at = now + self.dice.around(skill.punch_every * 0.75, skill.punch_every * 1.3);
                bot.windup = self.dice.around(skill.windup * 0.5, skill.windup * 1.6);
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
            match duel.stance {
                // In and out: back out of their reach for a moment.
                Stance::Poke => duel.back_until = now + self.dice.around(0.45, 1.0),
                // It has had its punch from where it lay in wait: at them.
                Stance::Ambush => {
                    duel.stance = Stance::Press;
                    duel.spot = None;
                    duel.until = now + self.dice.around(1.5, 3.5);
                }
                _ => {}
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
                    toward(self, bot, there, pace)
                } else {
                    Vec2::ZERO
                }
            }
            Stance::Ambush => match duel.spot {
                Some(lair) => toward(self, bot, lair, pace),
                None => Vec2::ZERO,
            },
            Stance::Press | Stance::Circle | Stance::Poke | Stance::Rush => {
                let backing = duel.stance == Stance::Poke && now < duel.back_until;
                if duel.spot.is_none() || now >= duel.spot_until {
                    duel.spot = if backing {
                        // Straight back from them, as far as there is
                        // ground to stand on that way.
                        let back = here - off.normalize_or_zero() * (POKE_BACK - away).max(0.0);
                        (nav.walkable_at(back) && flung(nav, there, back) == 0.0).then_some(back)
                    } else {
                        self.spot(bot, view, there, &duel)
                    };
                    duel.spot_until = now + self.dice.between(0.3, 0.7);
                }
                let pace = if duel.stance == Stance::Circle { pace * 0.85 } else { pace };
                match duel.spot {
                    Some(spot) => toward(self, bot, spot, pace),
                    // Nowhere to stand round them: up to them as it used
                    // to go, or, backing, where it is.
                    None if backing || away <= REACH * PRESS_IN => Vec2::ZERO,
                    None => toward(self, bot, there, pace),
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
        let stung = now - bot.hurt_at < 3.0;
        let reach = match bot.goal {
            // Backing off, it fights only whoever is on top of it.
            Goal::Retreat => 2.6,
            Goal::Defend | Goal::Hunt => sight,
            // On the island it is raiding, whoever is home.
            Goal::Raid(victim) if nav.grid().region(here) == Some(Region::Team(victim)) => sight,
            _ => (self.skill.aggro_at(now) * bot.temper.bold).max(if stung { 9.0 } else { 0.0 }).min(sight),
        };
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
                if away > reach || (seen.at.y - bot.body.at.y).abs() > 3.0 {
                    return None;
                }
                // Seeing someone off its island, it meets them on the
                // land: it does not hold the end of a bridge against them.
                if bot.goal == Goal::Defend && away > 2.6 && nav.grid().region(there) != Some(home) {
                    return None;
                }
                // Hurt and out on a bridge, it starts nothing.
                if on_plank && me.health < 0.5 && away > 3.0 {
                    return None;
                }
                if !nav.connected(here, there) {
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
        let stale = bot.path_to.is_none_or(|was| was.distance(to) > 1.0)
            || (bot.path.is_empty() && (bot.path_version != nav.version || now - bot.path_at > 0.5))
            || now - bot.path_at > again.max(0.5) * 4.0;
        if stale {
            bot.path = nav.path(here, to).unwrap_or_default();
            bot.path_to = Some(to);
            bot.path_version = nav.version;
            bot.path_at = now;
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
        let laying = match bot.laying {
            Some(laying) => laying,
            None => {
                let Some((_, _, span)) = self.next_span(bot, view, true, false) else {
                    bot.think_at = now;
                    return Vec2::ZERO;
                };
                let laying = Laying {
                    span: self.bends(span),
                    from,
                    to,
                    chain: bot.chains,
                    next: 0,
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
/// and a third buys as they all did. What goes on its second button is
/// what it fancies this life (`fancy`, [`Persona::button`]), one at a
/// time.
#[allow(clippy::too_many_arguments)]
fn shopping(
    persona: &Persona,
    fancy: Utility,
    role: Role,
    alone: bool,
    elapsed: f32,
    purse: &Purse,
    kit: &Kit,
    team: &Upgrades,
    short: f32,
) -> Vec<Ware> {
    let mut list = Vec::new();
    let has = |item: Item| purse[item.index()];
    // The metres for its bridge before anything.
    let buys = (short / BRIDGE_A_BUY).ceil() as usize;
    list.extend(std::iter::repeat_n(Ware::Bridge, buys.min(4)));
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
    // Its second button: what it fancies, once it is armed, or at once if
    // that is what it is like; a fireball late on and with silver to
    // spare, unless it is one for fireballs. A keeper with nobody to
    // fetch it amethyst makes do with a healing potion.
    let fancy = match fancy {
        Utility::Speed | Utility::Invisibility if keeper && has(Item::Amethyst) == 0 => Utility::Healing,
        other => other,
    };
    let keen = button > 1.2;
    let ready = match fancy {
        Utility::Fireball => (elapsed > 300.0 && kit.sword > 0 && kit.shield > 0) || (keen && elapsed > 45.0),
        Utility::Healing => kit.sword > 0 || keen,
        Utility::Speed | Utility::Invisibility => has(Item::Amethyst) > 0 || (keen && elapsed > 60.0),
        Utility::Builder => false,
    };
    // One kind of thing a life, as when there was one button for them
    // (five since 2026-11-13; mine, so that the games they were tuned by
    // are the games they play): nothing on any button but what it fancies,
    // and none of that left.
    let bare = kit.slots.iter().flatten().all(|stack| stack.utility == fancy && stack.count == 0);
    if ready && bare && price(Ware::Utility(fancy), kit, team).is_some() {
        wanted.push((0.76 * button, Ware::Utility(fancy)));
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
        let mut nav = Nav::new(Arc::new(grid));
        for index in 0..=span.planks() {
            nav.hear(1, 0, index as u16, span.rung(index));
        }
        assert!(nav.joined(Region::Team(1), Region::Middle));
        nav
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
        let list = |role, elapsed, purse: &Purse, kit: &Kit, short| {
            shopping(&plain, Utility::Healing, role, false, elapsed, purse, kit, &team, short)
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
        let daring = shopping(persona("ESTP"), Utility::Fireball, Role::Runner, false, 60.0, &silver, &kit, &team, 0.0);
        assert_eq!(daring[0], Ware::Utility(Utility::Fireball), "{daring:?}");
        let steady = shopping(persona("ISTJ"), Utility::Fireball, Role::Runner, false, 60.0, &silver, &kit, &team, 0.0);
        assert!(matches!(steady[0], Ware::Sword(1) | Ware::Shield(1)), "{steady:?}");
        assert!(!steady.contains(&Ware::Utility(Utility::Fireball)), "not before it is armed, and late: {steady:?}");
        // The defender spends its team's sapphire on defences, the
        // commander on swords.
        let sapphire: Purse = [0, 0, 9, 0];
        let safe = shopping(persona("ISFJ"), Utility::Healing, Role::Keeper, false, 200.0, &sapphire, &armed, &team, 0.0);
        assert_eq!(&safe[..3], &[Ware::HealPool, Ware::BetterShields, Ware::Trap], "{safe:?}");
        let bent_on = shopping(persona("ENTJ"), Utility::Healing, Role::Runner, false, 200.0, &sapphire, &armed, &team, 0.0);
        assert_eq!(bent_on[0], Ware::StrongerSwords, "{bent_on:?}");
        // The troll has an invisibility potion as soon as it has the
        // amethyst for it, before the best of swords; a keeper who
        // fancies one, with nobody to fetch amethyst, a healing potion.
        let amethyst: Purse = [0, 0, 0, 4];
        let sly = shopping(persona("ESFP"), Utility::Invisibility, Role::Runner, false, 200.0, &amethyst, &armed, &team, 0.0);
        assert_eq!(sly[0], Ware::Utility(Utility::Invisibility), "{sly:?}");
        let home = shopping(persona("ESFP"), Utility::Invisibility, Role::Keeper, false, 200.0, &[0, 9, 0, 0], &armed, &team, 0.0);
        assert!(home.contains(&Ware::Utility(Utility::Healing)) && !home.contains(&Ware::Utility(Utility::Invisibility)));
        // Sixteen of them, each by its own name, and none with odds of
        // nothing at all.
        let mut names: Vec<&str> = PERSONAS.iter().map(|persona| persona.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 16);
        for persona in &PERSONAS {
            assert!(persona.opening.iter().sum::<f32>() > 0.0 && persona.button.iter().sum::<f32>() > 0.0, "{}", persona.name);
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
}
