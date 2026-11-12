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
    AIR_DRAG, BLIND_SIGHT, BOT_PUNCH_EVERY, BRIDGE_A_BUY, BUILDER_SPEED, DRINK_SECS, DRINK_SLOW, FIREBALL_RANGE,
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

    /// Moves it on by `dt` seconds: up and down as a knock threw it,
    /// landing on whatever ground it comes down onto and falling where
    /// there is none; carried across by a knock, dying away as it goes;
    /// and, on the ground and its own to move, walked by `walk`, metres a
    /// second across the ground, only where a body can walk. It never
    /// walks off an edge: a knock is what takes it off one.
    pub fn step(&mut self, nav: &Nav, walk: Vec2, dt: f32) {
        let here = self.at.xz();
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
            let across = here + self.shove * dt;
            self.at.x = across.x;
            self.at.z = across.y;
            let drag = if airborne { AIR_DRAG } else { GROUND_DRAG };
            self.shove *= drag.powf(dt * 20.0);
            if self.shove.length_squared() < 0.01 {
                self.shove = Vec2::ZERO;
            }
            return;
        }
        if airborne || walk == Vec2::ZERO {
            return;
        }
        // Where a body walks; or, off that already, knocked against a
        // stand or onto an island's rim, any ground, to get back to it.
        let astray = !nav.walkable_at(here);
        let open = |to: Vec2| {
            (nav.walkable_at(to) || astray) && nav.ground(to).is_some_and(|there| there - self.at.y <= STEP)
        };
        let whole = here + walk * dt;
        let to = [whole, Vec2::new(whole.x, here.y), Vec2::new(here.x, whole.y)]
            .into_iter()
            .find(|&to| to != here && open(to));
        if let Some(to) = to {
            self.at.x = to.x;
            self.at.z = to.y;
            if let Some(ground) = nav.ground(to) {
                // Down a step at once, as up one; down a drop, it falls.
                if self.at.y - ground <= STEP {
                    self.at.y = ground;
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
    /// Which of the two smaller islands beside its own it means to bridge
    /// to.
    sapphire: u8,
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
}

impl Bot {
    fn new(seat: usize, body: Body, dice: &mut Dice) -> Self {
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
            sapphire: (dice.roll() * 2.0) as u8 % 2,
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
        }
    }

    pub fn goal(&self) -> Goal {
        self.goal
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
    /// The potion on its second button is drunk.
    pub drink: bool,
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
            dice,
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

    /// The computer takes `seat`, standing `at`.
    pub fn seat(&mut self, seat: usize, at: Vec3, yaw: f32) {
        if seat < ARENA_SEATS {
            self.bots[seat] = Some(Bot::new(seat, Body::standing(at, yaw), &mut self.dice));
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
        bot.wants = [0; 4];
        // Dead on a raid, it stays home a while once it stands again.
        if matches!(bot.goal, Goal::Raid(_)) {
            bot.rest_until = now + rest;
        }
        bot.raiding_since = None;
        self.claims.retain(|claim| claim.2 != seat);
        self.call_off(seat, now);
    }

    /// The bot in `seat` has been knocked, from `from`.
    pub fn knock(&mut self, seat: usize, from: Vec2, running: bool) {
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.body.knock(from, running, 1.0);
        }
    }

    /// The bot in `seat` has been knocked by this much, across the ground
    /// and upward, as whoever runs the game has worked it out for itself:
    /// a game played offline, where a knock is the file's numbers and
    /// lands on the body the device draws. On top of half of whatever
    /// already carried it, as [`Body::knock`] has it.
    pub fn shoved(&mut self, seat: usize, across: Vec2, up: f32) {
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.body.shove = bot.body.shove * 0.5 + across;
            bot.body.lift = (bot.body.lift * 0.5 + up).min(up.max(0.0));
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
    /// whoever of a team wants them most is given them first.
    pub fn wants(&self, seat: usize) -> (Purse, u8) {
        self.bot(seat).map_or(([0; 4], 0), |bot| (bot.wants, bot.urgency))
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
        if bot.body.free(view.nav) {
            if view.elapsed >= bot.think_at {
                self.think(&mut bot, view);
            }
            let walk = self.act(&mut bot, view, dt, &mut order);
            bot.body.step(view.nav, walk, dt);
        } else {
            // In the air, or carried by a knock: nothing but that.
            bot.in_reach_since = None;
            bot.body.step(view.nav, Vec2::ZERO, dt);
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
        let mate_bot = mate_bot.is_some();
        bot.role = if with_person || !mate_bot || seat < ARENA_TEAMS {
            Role::Keeper
        } else {
            Role::Runner
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
        if me.health < (if mortal { skill.careful_below } else { skill.retreat_below }) && pressed {
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
        // What is left of one it is part way along; the whole of a new one.
        let short = span.map_or(0.0, |(_, _, span)| {
            let laid = bot.laying.map_or(0, |laying| laying.next.saturating_sub(1));
            let left = span.length() * (1.0 - laid as f32 / span.planks() as f32);
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
        let list = shopping(bot.role, alone, now, &purse, &kit, &upgrades, short);
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
        // The bridge to the middle comes before anything the keeper wants
        // silver for.
        let bridging = !nav.joined(home, Region::Middle) && now < 90.0;
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
        // A person's pad is theirs while they are at it.
        let person_at_pad = with_person && mate_seen.alive && mate_seen.at.xz().distance(ARENA_PADS[team]) < 3.5;
        if saving && can_go_home && !person_at_pad {
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
        let rested = now >= bot.rest_until && me.health >= 0.7 && kit.sword > 0;
        if now >= skill.raid_from && now < skill.hunt_from && rested && !with_person {
            let others = (0..ARENA_TEAMS).filter(|&other| other != team && in_play(other)).count();
            for victim in (0..ARENA_TEAMS).filter(|&victim| victim != team && in_play(victim)) {
                let mine = matches!(bot.goal, Goal::Raid(on) if usize::from(on) == victim);
                // Along with its teammate; or, the one that goes out, by
                // its own choice.
                let along = mate_raids == Some(victim) && (runner || now >= skill.keeper_raids_from);
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
                let score = 0.6 + 0.14 * ramp - here.distance(their_heart) / 2000.0 + chance * 0.05;
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

    /// The bridge the bot would lay next, if any: from its island to the
    /// middle, if nothing joins them yet; then, for the one that goes
    /// out, from the middle to one of the two smaller islands beside its
    /// own, if neither is joined to its island. Not one a teammate is
    /// laying.
    fn next_span(&mut self, bot: &Bot, view: &View, runner: bool, with_person: bool) -> Option<(Region, Region, Span)> {
        let nav = view.nav;
        let team = arena_team(bot.seat);
        let home = Region::Team(team);
        // The one it was laying, to be finished.
        if let Some(laying) = bot.laying {
            return Some((laying.from, laying.to, laying.span));
        }
        let wanted = if !nav.joined(home, Region::Middle) {
            // The one that goes out lays it; the keeper if nobody else
            // will, and beside a person only once they have had the time
            // to.
            if !runner && !(with_person && view.elapsed > 40.0) {
                return None;
            }
            (home, Region::Middle)
        } else {
            if !runner && !(with_person && view.elapsed > 120.0) {
                return None;
            }
            let beside = beside(team);
            let sapphire = (!beside.iter().any(|&island| nav.joined(home, Region::Sapphire(island))))
                .then(|| (Region::Middle, Region::Sapphire(beside[usize::from(bot.sapphire) % 2])));
            // Or, once the raids are on, to the island of a team that has
            // built no way to it: nobody sits a game out behind the void.
            let stranger = (runner && view.elapsed >= self.skill.raid_from + 90.0)
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
            sapphire.or(stranger)?
        };
        if self.claims.iter().any(|&(from, to, by)| (from, to) == wanted && by != bot.seat) {
            return None;
        }
        // Beside whatever was begun there and left.
        let tried = self.begun.iter().find(|begun| (begun.0, begun.1) == wanted).map_or(0, |begun| begun.2);
        let aside = [0.0, 2.9, -2.9][usize::from(tried) % 3];
        let span = nav.grid().span(wanted.0, wanted.1, aside)?;
        Some((wanted.0, wanted.1, span))
    }

    /// The fast layer: what the bot does this frame toward its goal, and
    /// which way it walks, in metres a second across the ground.
    fn act(&mut self, bot: &mut Bot, view: &View, dt: f32, order: &mut Order) -> Vec2 {
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
            if nav.ground(ahead).is_none_or(|ground| ground - bot.body.at.y > STEP) {
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
                order.drink = true;
            } else {
                order.drinking = true;
                pace *= DRINK_SLOW;
            }
        }

        // A fireball, at whoever of another team is a throw away and has
        // land under them: never at someone out on a bridge.
        if kit.utility == Some(Utility::Fireball) && kit.count > 0 && now >= bot.throw_at && bot.drinking.is_none() {
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
        bot.foe = self.foe(bot, view);
        if let Some(foe) = bot.foe.filter(|_| bot.drinking.is_none()) {
            let there = view.seats[foe].at.xz();
            let off = there - here;
            let away = off.length();
            bot.body.yaw = (-off.x).atan2(-off.y);
            if away <= REACH + BODY - 0.1 {
                let since = *bot.in_reach_since.get_or_insert(now);
                if now - since >= skill.windup && now >= bot.punch_at {
                    bot.punch_at = now + skill.punch_every;
                    order.punch = Some(foe);
                    // On the run only if it really was running in.
                    order.running = bot.speed > RUN_SPEED * 0.6;
                }
            } else if away > REACH + BODY + 0.5 {
                bot.in_reach_since = None;
            }
            if away <= REACH * PRESS_IN {
                return Vec2::ZERO;
            }
            // Straight at them where a body can walk it, and round what
            // stands between otherwise.
            if nav.line(here, there) {
                bot.path.clear();
                bot.path_to = None;
                return off / away * pace;
            }
            return self.walk_to(bot, view, there, pace, 0.5);
        }
        bot.in_reach_since = None;

        match bot.goal {
            Goal::Idle => {
                let at = if bot.role == Role::Keeper { spots.guard } else { spots.heart };
                self.walk_to(bot, view, at, pace * 0.7, 2.0)
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
                if kit.utility == Some(Utility::Healing) && kit.count > 0 && bot.drinking.is_none() && nearest > 6.0 {
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
                // There: no longer than a raid lasts.
                let since = *bot.raiding_since.get_or_insert(now);
                if now - since > skill.raid_secs {
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
                if view.anchors[victim] > 0.0 && now >= skill.anchors_from && !spared && free {
                    self.strikers[victim] = Some(seat);
                    let anchor = ARENA_ANCHORS[victim];
                    if here.distance(anchor) <= REACH + ANCHOR_RADIUS - 0.2 {
                        let off = anchor - here;
                        bot.body.yaw = (-off.x).atan2(-off.y);
                        if now >= bot.strike_at {
                            bot.strike_at = now + skill.anchor_every;
                            order.strike = Some(victim as u8);
                        }
                        return Vec2::ZERO;
                    }
                    return self.walk_to(bot, view, theirs.strike, pace, 2.0);
                }
                // Otherwise onto their pad, for what lies on it.
                self.walk_to(bot, view, theirs.pad, pace, 2.0)
            }
        }
    }

    /// Whom the bot fights this frame, if anyone: the nearest of another
    /// team it can see and get at, within how far it goes for people.
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
            _ => self.skill.aggro_at(now).max(if stung { 9.0 } else { 0.0 }).min(sight),
        };
        let home = Region::Team(arena_team(seat));
        let on_plank = nav.on_plank(here) && nav.grid().height(here).is_none();
        view.seats
            .iter()
            .enumerate()
            .filter(|&(other, seen)| other != seat && seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
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
    /// laid). Getting nowhere for a while, it thinks again.
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
                // No further than the point itself in a frame.
                off.normalize_or_zero() * pace.min(off.length() / (1.0 / 60.0))
            }
            None => Vec2::ZERO,
        }
    }

    /// Lays the bridge from `from` to `to`: to where it begins; then out
    /// along it at the pace anyone builds at, each plank laid ahead of
    /// its feet and paid for in metres; and it is done at the far end.
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
                    span,
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
        let (way, planks) = (span.way(), span.planks());
        let plank = span.length() / planks as f32;
        let along = (here - span.from).dot(way);
        let aside = (here - span.from).perp_dot(way).abs();
        // Not at its line yet: to the nearest a body can walk to where it
        // begins, or, taking it up again, to as far as it is laid.
        let reached = span.rung(laying.next.saturating_sub(1));
        let on_line = aside < 0.9 && along > -1.5 && along < span.length() + 1.0;
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
        while next <= planks && (next as f32 - 1.0) * plank < along + LAID_AHEAD + plank {
            let cost = if next == 0 { 0.0 } else { plank };
            if bot.metres + 1e-3 < cost {
                break;
            }
            bot.metres -= cost;
            order.rungs.push((laying.chain, next as u16, span.rung(next), span.heading()));
            next += 1;
        }
        bot.laying = Some(Laying { next, ..laying });
        order.carrying = true;
        // Laid to its end, and walked to it: done.
        if next > planks && along >= span.length() - ARRIVED {
            bot.laying = None;
            self.claims.retain(|claim| claim.2 != bot.seat);
            bot.think_at = now;
            return Vec2::ZERO;
        }
        // Out along it, no further than it is laid.
        let laid = (next.saturating_sub(1)) as f32 * plank;
        let limit = if next > planks { span.length() } else { (laid - 0.4).max(0.0) };
        let _ = pace;
        let speed = BUILDER_SPEED.min((limit - along).max(0.0) / dt.max(1e-3));
        // Onto its middle line, and on.
        let onto = span.from + way * along.clamp(0.0, span.length());
        let sideways = onto - here;
        if speed <= 0.0 && sideways.length() < 0.05 {
            // Out of metres on the way: nothing for it but to go and get
            // more.
            if bot.metres + 1e-3 < plank && next <= planks {
                bot.think_at = bot.think_at.min(now + 0.2);
            }
            return Vec2::ZERO;
        }
        way * speed + sideways.clamp_length_max(1.0) * 3.0
    }
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

/// What a bot would buy, in the order it would, by what it does on its
/// team, what it holds and has, and how many metres of bridge it is
/// `short` of for the span it means to lay.
fn shopping(role: Role, alone: bool, elapsed: f32, purse: &Purse, kit: &Kit, team: &Upgrades, short: f32) -> Vec<Ware> {
    let mut list = Vec::new();
    let has = |item: Item| purse[item.index()];
    // The metres for its bridge before anything.
    let buys = (short / BRIDGE_A_BUY).ceil() as usize;
    list.extend(std::iter::repeat_n(Ware::Bridge, buys.min(4)));
    // Sapphire is the team's: the heal pool, its generator, its swords,
    // its shields, a trap.
    if has(Item::Sapphire) > 0 {
        for ware in [Ware::HealPool, Ware::Forge, Ware::StrongerSwords, Ware::BetterShields, Ware::Trap] {
            if price(ware, kit, team).is_some() {
                list.push(ware);
            }
        }
    }
    // Amethyst is the best of its own arms.
    if has(Item::Amethyst) >= 4 && kit.sword < 3 {
        list.push(Ware::Sword(3));
    }
    if has(Item::Amethyst) >= 6 && kit.shield < 3 {
        list.push(Ware::Shield(3));
    }
    // A sword: the silver one once the gold for it is near, and until
    // then the basic one, which the keeper has first of all.
    let keeper = role == Role::Keeper && !alone;
    if kit.sword < 2 && (has(Item::Gold) >= 4 || kit.sword == 1) {
        list.push(Ware::Sword(2));
    }
    if kit.sword == 0 && has(Item::Gold) < 4 && (keeper || elapsed > 45.0) {
        list.push(Ware::Sword(1));
    }
    if kit.shield == 0 && (keeper || kit.sword > 0) {
        list.push(Ware::Shield(1));
    }
    // The second button: for the one that goes out, late on and with
    // silver to spare, a fireball; otherwise a healing potion, with spare
    // gold.
    let thrower = role == Role::Runner && elapsed > 300.0 && kit.sword > 0 && kit.shield > 0;
    if thrower && kit.count == 0 && has(Item::Silver) >= 40 {
        list.push(Ware::Utility(Utility::Fireball));
    } else if kit.sword > 0 && (kit.utility.is_none() || kit.utility == Some(Utility::Healing)) && kit.count == 0 {
        list.push(Ware::Utility(Utility::Healing));
    }
    if kit.sword >= 2 && kit.shield < 2 {
        list.push(Ware::Shield(2));
    }
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
    fn a_knock_carries_a_body_and_off_the_edge_it_falls() {
        let nav = island();
        // Knocked from the west while standing in the middle: up and
        // east, down again on the island, and its own to move after.
        let mut body = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        body.knock(Vec2::new(-1.0, 0.0), false, 1.0);
        assert!(!body.free(&nav));
        assert_eq!((body.shove, body.lift), (Vec2::X * KNOCK_BACK, KNOCK_UP));
        let mut highest = 0.0f32;
        for _ in 0..180 {
            body.step(&nav, Vec2::X * RUN_SPEED, 1.0 / 60.0);
            highest = highest.max(body.at.y);
        }
        assert!(highest > 1.2, "it rose to {highest}");
        assert!(body.free(&nav), "{body:?}");
        assert!(body.at.x > 2.0 && body.at.x < 5.6 && body.at.y == 0.25, "{}", body.at);
        // On the run from near the edge, it goes over it and falls, and
        // is never its own to move again.
        let mut body = Body::standing(Vec3::new(4.0, 0.25, 0.0), 0.0);
        body.knock(Vec2::new(3.0, 0.0), true, 1.0);
        for _ in 0..180 {
            body.step(&nav, Vec2::ZERO, 1.0 / 60.0);
            assert!(!body.free(&nav) || body.at.y > 0.0);
        }
        assert!(body.at.x > 6.5 && body.at.y < -20.0, "{}", body.at);
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

    #[test]
    fn what_a_bot_buys_goes_by_what_it_holds_and_what_it_does() {
        let (kit, team) = (Kit::default(), Upgrades::default());
        // Short of 44 m of bridge: two buys of it before anything.
        let list = shopping(Role::Runner, false, 10.0, &[8, 0, 0, 0], &kit, &team, 44.6);
        assert_eq!(&list[..2], &[Ware::Bridge, Ware::Bridge]);
        // The keeper: a basic sword, and a shield.
        let list = shopping(Role::Keeper, false, 10.0, &[10, 1, 0, 0], &kit, &team, 0.0);
        assert_eq!(list, vec![Ware::Sword(1), Ware::Shield(1)]);
        // The runner waits for the silver sword's gold, early on.
        let list = shopping(Role::Runner, false, 10.0, &[10, 1, 0, 0], &kit, &team, 0.0);
        assert!(list.is_empty(), "{list:?}");
        let list = shopping(Role::Runner, false, 60.0, &[10, 5, 0, 0], &kit, &team, 0.0);
        assert_eq!(list[0], Ware::Sword(2));
        // Sapphire goes on the team, the heal pool first.
        let list = shopping(Role::Runner, false, 200.0, &[0, 0, 3, 0], &Kit { sword: 2, shield: 1, ..kit }, &team, 0.0);
        assert_eq!(&list[..2], &[Ware::HealPool, Ware::Forge]);
        // With a sword, a healing potion for the second button.
        assert!(list.contains(&Ware::Utility(Utility::Healing)) && list.contains(&Ware::Shield(2)));
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
