//! What the computer's players do in the waiting room (since 2026-11-16;
//! Hajun: "make the bots just jump around and go around in different ways
//! according to their personality while in the waiting room, right now
//! they are just standing still in a circle"). Until then each stood where
//! its seat waits, on a ring round the middle, from the moment it came to
//! the moment the fight began.
//!
//! One piece for the server's bots online and the device's own in a game
//! played offline, as their brain is ([`super::Squad`]). It has nothing of
//! the game in it: the room is a floor with four walls, and each of them
//! ([`Waiter`]) is someone of its kind ([`Persona`]) passing the time:
//! standing and looking about, walking somewhere, running across, running
//! round the room, jumping on the spot, going up to someone, following
//! them, running rings round them, standing at the glass to look out over
//! the map, turning on the spot, pacing up and down, darting from side to
//! side. Which of those, how often and for how long goes by leanings read
//! off the kind's own ([`Nature`]): the free spirit is all over the room
//! and never on the ground for long, the troll is at someone's heels, the
//! defender stands, paces and looks out. **Nothing here is a rule**: what
//! it does next is thrown by those leanings, each of which is its own,
//! thrown about its kind's as it comes in, and so is how long it goes on.
//!
//! Only where it stands and which way it faces comes of it ([`Stood`]),
//! which is all anyone is sent of a body: no punch, nothing bought. It is
//! in nobody's way, as the computer's players are in the fight.

use std::f32::consts::{PI, TAU};

use bevy::math::{Vec2, Vec3};

use super::{Dice, FALL_GRAVITY, JUMP_UP, Persona, RISE_GRAVITY, pick};
use crate::bridge_wars::RUN_SPEED;

/// How far from the middle of the room a body's middle goes, along each
/// side, in metres: the walls' inner faces are 13.5 out in Hajun's map (a
/// box 30 m square with walls 1.5 m thick), a body is 0.45 across, and a
/// little is kept in hand. The game checks the map agrees.
pub const HALF: f32 = 12.6;
/// How far from the middle of the room anyone comes into it, in metres,
/// at the least and at the most; and how far from everyone already there,
/// if there is the room.
pub const ARRIVES: (f32, f32) = (1.5, 7.0);
const ARRIVES_CLEAR: f32 = 1.8;
/// This near where it was going, it is there, in metres.
const NEAR: f32 = 0.2;
/// The longest it goes on with one thing, in seconds, whatever comes of
/// it: nothing it does can hold it for good.
const LONGEST: f32 = 14.0;
/// How many things there are to do ([`Doing`]), and which is which in a
/// list of odds.
const WAYS: usize = 12;
const REST: usize = 0;
const AMBLE: usize = 1;
const DASH: usize = 2;
const LAP: usize = 3;
const HOP: usize = 4;
const VISIT: usize = 5;
const TAIL: usize = 6;
const RING: usize = 7;
const GAZE: usize = 8;
const SPIN: usize = 9;
const PACE: usize = 10;
const DART: usize = 11;
/// What each is called, for telling what they did over a while.
pub const WAY_NAMES: [&str; WAYS] =
    ["rest", "amble", "dash", "lap", "hop", "visit", "tail", "ring", "gaze", "spin", "pace", "dart"];

/// Where one of them stands in the room: across its floor from its middle,
/// how far over the floor, and its turn about the upright, in radians (-Z
/// turned by it is ahead, as a body's is).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Stood {
    pub at: Vec2,
    pub up: f32,
    pub yaw: f32,
}

impl Stood {
    /// Where that is in the world, the room's middle being over `centre`
    /// and its floor `floor` up.
    pub fn in_room(&self, centre: Vec2, floor: f32) -> Vec3 {
        Vec3::new(centre.x + self.at.x, floor + self.up, centre.y + self.at.y)
    }
}

/// How one of them is in a room with nothing to do, each from 0 to 1 but
/// where said: read off its kind's leanings ([`Persona`]) and thrown a
/// little either side of them as it comes in, so that no two of a kind
/// pass the time alike.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Nature {
    /// How much it is on the move rather than standing (the kind's
    /// `roams` and `drive`).
    lively: f32,
    /// How much it jumps (`bouncy`).
    springy: f32,
    /// How readily it goes up to someone (`bold`).
    sociable: f32,
    /// How much it is at someone's heels, or round and round them
    /// (`shoves`).
    pest: f32,
    /// How much it is one for standing at the glass, looking out over the
    /// map (`schemes`, `minds`).
    thoughtful: f32,
    /// How little it can stand still for (`patience`, the other way up).
    restless: f32,
    /// How quick it is on its feet (`pvp`).
    nimble: f32,
    /// How far its heading swings to either side as it walks, in radians
    /// (`weave`), and how much of the most that is.
    sway: f32,
    swings: f32,
    /// How fast it walks when it is in no hurry, in metres a second, and
    /// how fast it turns, in radians a second.
    pace: f32,
    turn: f32,
    /// Its own liking for each thing there is to do, about 1.
    tastes: [f32; WAYS],
}

impl Nature {
    fn of(persona: &Persona, dice: &mut Dice) -> Self {
        let mut near = |leaning: f32, spread: f32| (leaning + spread * dice.bell()).clamp(0.0, 1.0);
        let lively = near(0.15 + 0.5 * persona.roams + 0.35 * persona.drive, 0.08);
        let springy = near(persona.bouncy, 0.1);
        let sociable = near((persona.bold - 0.6) / 0.8, 0.1);
        let pest = near(persona.shoves - 0.8, 0.1);
        let thoughtful = near(0.7 * persona.schemes + 0.3 * (persona.minds - 0.5), 0.08);
        let restless = near((2.8 - persona.patience) / 1.8, 0.1);
        let nimble = near(persona.pvp, 0.1);
        let swings = near((persona.weave - 0.5) / 1.3, 0.1);
        let mut tastes = [1.0; WAYS];
        for taste in &mut tastes {
            *taste = dice.around(0.55, 1.45);
        }
        Self {
            lively,
            springy,
            sociable,
            pest,
            thoughtful,
            restless,
            nimble,
            sway: swings * dice.between(0.25, 0.6),
            swings,
            pace: dice.between(3.2, 4.8) + 2.0 * persona.drive,
            turn: dice.between(7.0, 13.0),
            tastes,
        }
    }

    /// How likely it is to take up each thing next, each as likely as its
    /// number is big: `soon`, the room is counting down, and the lively
    /// are on their toes; `company`, there is someone else in the room to
    /// go to.
    fn odds(&self, soon: bool, company: bool) -> [f32; WAYS] {
        let keen = if soon { 1.0 } else { 0.0 };
        let company = if company { 1.0 } else { 0.0 };
        let mut odds = [0.0; WAYS];
        odds[REST] = (0.45 + 1.7 * (1.0 - self.lively)) * (1.0 - 0.35 * keen * self.lively);
        odds[AMBLE] = 0.35 + 0.9 * self.lively;
        odds[DASH] = 0.05 + 0.95 * self.lively * self.lively;
        odds[LAP] = 0.02 + 0.6 * self.lively * self.lively;
        odds[HOP] = (0.04 + 1.3 * self.springy * self.springy) * (1.0 + 0.6 * keen);
        odds[VISIT] = company * (0.08 + 0.95 * self.sociable);
        odds[TAIL] = company * (0.02 + 0.9 * self.pest * self.sociable);
        odds[RING] = company * (0.01 + 0.8 * self.pest * self.springy);
        odds[GAZE] = 0.06 + self.thoughtful * (1.0 - 0.5 * self.lively);
        odds[SPIN] = 0.01 + 0.35 * self.springy * self.swings;
        odds[PACE] = 0.03 + 0.8 * self.restless * (1.0 - 0.7 * self.lively);
        odds[DART] = 0.02 + 0.7 * self.nimble * (0.4 + 0.6 * self.lively);
        for (odd, taste) in odds.iter_mut().zip(self.tastes) {
            *odd *= taste;
        }
        odds
    }
}

/// What it looks at, standing: someone, by their seat, or a way, as a
/// turn about the upright.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Look {
    Seat(usize),
    Way(f32),
}

/// What one of them is doing.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Doing {
    /// Standing, for `left` more seconds, looking at `at`, and at
    /// something else in `glance` seconds.
    Rest { left: f32, glance: f32, at: Option<Look> },
    /// Going `to`, at `speed`: a walk somewhere, or a run across the room.
    Go { to: Vec2, speed: f32, run: bool },
    /// Running round the room, `out` from its middle, one `way` or the
    /// other, through `left` more radians.
    Lap { way: f32, out: f32, speed: f32, left: f32 },
    /// Jumping where it stands, `left` more times, the next in `wait`
    /// seconds once it is down.
    Hop { left: u8, wait: f32 },
    /// Going up to whoever is in the seat `who`, to stand `off` from them
    /// and face them for `left` seconds: with `chase` seconds more to get
    /// to them in, and, once it has `met` them, over when they go off.
    Visit { who: usize, off: f32, speed: f32, left: f32, chase: f32, met: bool },
    /// At the heels of whoever is in `who`, `off` behind, for `left`
    /// seconds.
    Tail { who: usize, off: f32, left: f32 },
    /// Running rings round whoever is in `who`, `out` from them, one `way`
    /// or the other, through `left` more radians.
    Ring { who: usize, out: f32, way: f32, speed: f32, left: f32 },
    /// To the glass at `to`, and there looking out, the way `out`, for
    /// `left` seconds.
    Gaze { to: Vec2, out: f32, left: f32 },
    /// Turning where it stands, one `way` or the other, at `rate` radians
    /// a second, through `left` more.
    Spin { way: f32, rate: f32, left: f32 },
    /// Up and down between two `ends`, towards the one numbered `toward`,
    /// with `turns` more turns to make.
    Pace { ends: [Vec2; 2], toward: usize, turns: u8, speed: f32 },
    /// Darting `to`, with `left` more darts after it, the next after
    /// `pause` seconds.
    Dart { to: Vec2, left: u8, pause: f32 },
}

impl Doing {
    fn way(&self) -> usize {
        match self {
            Doing::Rest { .. } => REST,
            Doing::Go { run: false, .. } => AMBLE,
            Doing::Go { run: true, .. } => DASH,
            Doing::Lap { .. } => LAP,
            Doing::Hop { .. } => HOP,
            Doing::Visit { .. } => VISIT,
            Doing::Tail { .. } => TAIL,
            Doing::Ring { .. } => RING,
            Doing::Gaze { .. } => GAZE,
            Doing::Spin { .. } => SPIN,
            Doing::Pace { .. } => PACE,
            Doing::Dart { .. } => DART,
        }
    }
}

/// One of the computer's players in the waiting room.
#[derive(Clone, Debug)]
struct Waiter {
    nature: Nature,
    at: Vec2,
    up: f32,
    /// How fast it is rising, in metres a second: 0 on the floor.
    lift: f32,
    yaw: f32,
    doing: Doing,
    /// How long it has been at what it is doing, in seconds.
    since: f32,
    /// How long until it next thinks of a jump as it goes, in seconds.
    hop_in: f32,
    /// Where it is in the swing of its walk, in radians, and how fast
    /// that goes round.
    swing: f32,
    swing_rate: f32,
}

/// The turn about the upright of a body headed `way` across the floor.
fn heading(way: Vec2) -> f32 {
    (-way.x).atan2(-way.y)
}

/// A step of `far` radians at the most from the turn `from` towards the
/// turn `to`, the short way round.
fn turned(from: f32, to: f32, far: f32) -> f32 {
    let off = (to - from + PI).rem_euclid(TAU) - PI;
    if off.abs() <= far { to } else { from + far * off.signum() }
}

/// `point`, kept inside the room.
fn inside(point: Vec2) -> Vec2 {
    point.clamp(Vec2::splat(-HALF), Vec2::splat(HALF))
}

/// The point `out` from `middle` at the angle `angle` round it.
fn round(middle: Vec2, angle: f32, out: f32) -> Vec2 {
    middle + Vec2::new(angle.cos(), angle.sin()) * out
}

impl Waiter {
    fn stood(&self) -> Stood {
        Stood {
            at: self.at,
            up: self.up,
            yaw: self.yaw,
        }
    }

    fn grounded(&self) -> bool {
        self.up == 0.0 && self.lift == 0.0
    }

    /// Jumps, as anyone jumps, if it is on the floor.
    fn jump(&mut self) {
        if self.grounded() {
            self.lift = JUMP_UP;
        }
    }

    /// Turns towards the turn `to`, as fast as it turns.
    fn turn_to(&mut self, to: f32, dt: f32) {
        self.yaw = turned(self.yaw, to, self.nature.turn * dt);
    }

    /// Turns to face `point`.
    fn face(&mut self, point: Vec2, dt: f32) {
        let off = point - self.at;
        if off.length_squared() > 1e-4 {
            self.turn_to(heading(off), dt);
        }
    }

    /// A step towards `to` at `speed`, weaving as it goes if it is one
    /// that does, and facing the way it goes. Whether it is there.
    fn walk(&mut self, to: Vec2, speed: f32, dt: f32) -> bool {
        let off = to - self.at;
        let far = off.length();
        if far <= NEAR {
            return true;
        }
        let mut way = off / far;
        // Not over the last of the way, which it would go round and round.
        if far > 2.5 && self.nature.sway > 0.0 {
            way = Vec2::from_angle(self.nature.sway * self.swing.sin()).rotate(way);
        }
        self.at = inside(self.at + way * (speed * dt).min(far));
        self.turn_to(heading(way), dt);
        false
    }

    /// Somewhere in the room from `near` to `far` off, if a throw or two
    /// finds one; anywhere in it otherwise.
    fn somewhere(&self, dice: &mut Dice, near: f32, far: f32) -> Vec2 {
        for _ in 0..8 {
            let to = Vec2::new(dice.between(-HALF, HALF), dice.between(-HALF, HALF));
            let off = to.distance(self.at);
            if off >= near && off <= far {
                return to;
            }
        }
        Vec2::new(dice.between(-HALF, HALF), dice.between(-HALF, HALF))
    }

    /// Someone else in the room, by their seat: the nearest, as often as
    /// not, or anyone.
    fn someone(&self, dice: &mut Dice, me: usize, everyone: &[Option<Vec2>]) -> Option<usize> {
        let there: Vec<(usize, f32)> = everyone
            .iter()
            .enumerate()
            .filter(|(seat, _)| *seat != me)
            .filter_map(|(seat, at)| Some((seat, at.as_ref()?.distance(self.at))))
            .collect();
        if there.is_empty() {
            return None;
        }
        if dice.roll() < 0.5 {
            return there.iter().min_by(|a, b| a.1.total_cmp(&b.1)).map(|(seat, _)| *seat);
        }
        let which = ((dice.roll() * there.len() as f32) as usize).min(there.len() - 1);
        Some(there[which].0)
    }

    /// Standing a while: the livelier, the shorter, and nobody for long,
    /// the room being one they are in for a few seconds.
    fn rest(&self, dice: &mut Dice, soon: bool) -> Doing {
        let long = dice.around(0.5, 3.2) * (1.4 - 0.9 * self.nature.lively) * if soon { 0.7 } else { 1.0 };
        Doing::Rest {
            left: long,
            glance: dice.between(0.1, 1.0),
            at: None,
        }
    }

    /// Takes up the thing numbered `which`, if it can be done from here.
    fn begin(&self, which: usize, dice: &mut Dice, me: usize, everyone: &[Option<Vec2>], soon: bool) -> Option<Doing> {
        let nature = &self.nature;
        let either = |dice: &mut Dice| if dice.roll() < 0.5 { 1.0 } else { -1.0 };
        Some(match which {
            REST => self.rest(dice, soon),
            AMBLE => Doing::Go {
                to: self.somewhere(dice, 3.0, 13.0),
                speed: nature.pace,
                run: false,
            },
            DASH => Doing::Go {
                to: self.somewhere(dice, 9.0, 40.0),
                speed: RUN_SPEED,
                run: true,
            },
            LAP => Doing::Lap {
                way: either(dice),
                out: dice.between(7.0, HALF - 0.6),
                speed: if dice.roll() < 0.35 + 0.6 * nature.lively { RUN_SPEED } else { nature.pace },
                left: dice.between(1.2, 9.5),
            },
            HOP => Doing::Hop {
                left: 1 + (dice.roll() * (1.0 + 5.0 * nature.springy)) as u8,
                wait: 0.0,
            },
            VISIT => Doing::Visit {
                who: self.someone(dice, me, everyone)?,
                off: dice.between(1.3, 2.6),
                speed: if dice.roll() < 0.6 * nature.lively { RUN_SPEED } else { nature.pace },
                left: dice.around(1.0, 4.5),
                chase: dice.between(2.5, 6.0),
                met: false,
            },
            TAIL => Doing::Tail {
                who: self.someone(dice, me, everyone)?,
                off: dice.between(1.6, 3.2),
                left: dice.around(2.5, 8.0),
            },
            RING => Doing::Ring {
                who: self.someone(dice, me, everyone)?,
                out: dice.between(1.8, 3.4),
                way: either(dice),
                speed: RUN_SPEED * dice.between(0.7, 1.0),
                left: dice.between(4.0, 16.0),
            },
            GAZE => {
                // A point along one of the four walls, and the way out
                // through it.
                let along = dice.between(-HALF + 1.0, HALF - 1.0);
                let (to, out) = match (dice.roll() * 4.0) as u8 {
                    0 => (Vec2::new(HALF, along), Vec2::X),
                    1 => (Vec2::new(-HALF, along), Vec2::NEG_X),
                    2 => (Vec2::new(along, HALF), Vec2::Y),
                    _ => (Vec2::new(along, -HALF), Vec2::NEG_Y),
                };
                Doing::Gaze {
                    to,
                    out: heading(out),
                    left: dice.around(1.5, 6.0),
                }
            }
            SPIN => Doing::Spin {
                way: either(dice),
                rate: dice.between(5.0, 11.0),
                left: dice.between(TAU * 0.75, TAU * 3.0),
            },
            PACE => {
                let angle = dice.between(0.0, TAU);
                let reach = dice.between(1.5, 3.5);
                Doing::Pace {
                    ends: [inside(round(self.at, angle, reach)), inside(round(self.at, angle + PI, reach))],
                    toward: 0,
                    turns: 2 + (dice.roll() * 5.0) as u8,
                    speed: nature.pace * dice.between(0.75, 1.05),
                }
            }
            DART => Doing::Dart {
                to: inside(round(self.at, dice.between(0.0, TAU), dice.between(1.5, 3.5))),
                left: 2 + (dice.roll() * 5.0) as u8,
                pause: 0.0,
            },
            _ => return None,
        })
    }

    /// Takes up something else, thrown by what it leans to.
    fn choose(&mut self, dice: &mut Dice, me: usize, everyone: &[Option<Vec2>], soon: bool) {
        let company = everyone.iter().enumerate().any(|(seat, at)| seat != me && at.is_some());
        let mut odds = self.nature.odds(soon, company);
        // Never from standing to standing: the stillest of them does
        // something between two rests, so that none is seen to do nothing
        // for as long as it is in the room.
        if matches!(self.doing, Doing::Rest { .. }) {
            odds[REST] = 0.0;
        }
        let next = (0..4).find_map(|_| self.begin(pick(dice, &odds), dice, me, everyone, soon));
        self.doing = next.unwrap_or_else(|| self.rest(dice, soon));
        self.since = 0.0;
    }

    /// A frame of it.
    fn step(&mut self, dice: &mut Dice, me: usize, everyone: &[Option<Vec2>], soon: bool, dt: f32) {
        let dt = dt.clamp(0.0, 0.1);
        // Up and down, as a jump goes.
        if !self.grounded() {
            self.up += self.lift * dt;
            self.lift -= if self.lift > 0.0 { RISE_GRAVITY } else { FALL_GRAVITY } * dt;
            if self.up <= 0.0 {
                self.up = 0.0;
                self.lift = 0.0;
            }
        }
        self.swing += self.swing_rate * dt;
        self.since += dt;
        let there = |who: usize| everyone.get(who).copied().flatten().filter(|_| who != me);
        let nature = self.nature;
        // Whether it is going somewhere this frame, and at a run: one that
        // jumps does so as it goes.
        let mut going = false;
        let mut doing = self.doing;
        let done = match &mut doing {
            Doing::Rest { left, glance, at } => {
                *left -= dt;
                *glance -= dt;
                if *glance <= 0.0 {
                    *glance = dice.between(0.7, 2.6);
                    // At someone, mostly, if anyone is there; or about.
                    *at = match self.someone(dice, me, everyone) {
                        Some(who) if dice.roll() < 0.35 + 0.5 * nature.sociable => Some(Look::Seat(who)),
                        _ => Some(Look::Way(dice.between(-PI, PI))),
                    };
                }
                match *at {
                    Some(Look::Seat(who)) => {
                        if let Some(point) = there(who) {
                            self.face(point, dt);
                        }
                    }
                    Some(Look::Way(way)) => self.turn_to(way, dt),
                    None => {}
                }
                *left <= 0.0
            }
            Doing::Go { to, speed, .. } => {
                going = true;
                self.walk(*to, *speed, dt)
            }
            Doing::Lap { way, out, speed, left } => {
                going = true;
                let before = self.at.y.atan2(self.at.x);
                // A little ahead of where it is, round the ring.
                let ahead = before + *way * (2.5 / *out).min(0.7);
                self.walk(inside(round(Vec2::ZERO, ahead, *out)), *speed, dt);
                let swept = (self.at.y.atan2(self.at.x) - before + PI).rem_euclid(TAU) - PI;
                *left -= swept.abs();
                *left <= 0.0
            }
            Doing::Hop { left, wait } => {
                if self.grounded() {
                    *wait -= dt;
                    if *left == 0 {
                        true
                    } else {
                        if *wait <= 0.0 {
                            self.jump();
                            *left -= 1;
                            *wait = dice.between(0.02, 0.45);
                        }
                        false
                    }
                } else {
                    false
                }
            }
            Doing::Visit { who, off, speed, left, chase, met } => match there(*who) {
                None => true,
                Some(point) => {
                    let far = point.distance(self.at);
                    if far <= *off + 0.6 {
                        *met = true;
                        self.face(point, dt);
                        *left -= dt;
                        // One that jumps, jumps at them.
                        if dice.roll() < 1.6 * nature.springy * nature.springy * dt {
                            self.jump();
                        }
                        *left <= 0.0
                    } else if *met && far > *off + 2.5 {
                        // They have gone off, and it does not trail after
                        // them: that is another thing to do.
                        true
                    } else {
                        going = true;
                        *chase -= dt;
                        let stand = point + (self.at - point).normalize_or(Vec2::X) * *off;
                        self.walk(inside(stand), *speed, dt);
                        *chase <= 0.0
                    }
                }
            },
            Doing::Tail { who, off, left } => match there(*who) {
                None => true,
                Some(point) => {
                    *left -= dt;
                    if point.distance(self.at) > *off {
                        going = true;
                        let stand = point + (self.at - point).normalize_or(Vec2::X) * *off;
                        self.walk(inside(stand), RUN_SPEED, dt);
                    } else {
                        self.face(point, dt);
                    }
                    *left <= 0.0
                }
            },
            Doing::Ring { who, out, way, speed, left } => match there(*who) {
                None => true,
                Some(point) => {
                    going = true;
                    let off = self.at - point;
                    let before = off.y.atan2(off.x);
                    self.walk(inside(round(point, before + *way * 0.8, *out)), *speed, dt);
                    let off = self.at - point;
                    let swept = (off.y.atan2(off.x) - before + PI).rem_euclid(TAU) - PI;
                    *left -= swept.abs();
                    *left <= 0.0
                }
            },
            Doing::Gaze { to, out, left } => {
                if self.at.distance(*to) > NEAR {
                    going = true;
                    self.walk(*to, nature.pace, dt);
                } else {
                    self.turn_to(*out, dt);
                    *left -= dt;
                }
                *left <= 0.0
            }
            Doing::Spin { way, rate, left } => {
                let far = *rate * dt;
                self.yaw = (self.yaw + *way * far + PI).rem_euclid(TAU) - PI;
                *left -= far;
                *left <= 0.0
            }
            Doing::Pace { ends, toward, turns, speed } => {
                if self.walk(ends[*toward], *speed, dt) {
                    *toward = 1 - *toward;
                    *turns = turns.saturating_sub(1);
                }
                *turns == 0
            }
            Doing::Dart { to, left, pause } => {
                if *pause > 0.0 {
                    *pause -= dt;
                    false
                } else if self.walk(*to, RUN_SPEED, dt) {
                    if *left == 0 {
                        true
                    } else {
                        *left -= 1;
                        *pause = dice.between(0.03, 0.3);
                        *to = inside(round(self.at, dice.between(0.0, TAU), dice.between(1.5, 3.5)));
                        false
                    }
                } else {
                    false
                }
            }
        };
        self.doing = doing;
        // Jumping as it goes: now and then for most, and all the way for
        // one that is all spring.
        if going {
            self.hop_in -= dt;
            if self.hop_in <= 0.0 {
                self.hop_in = dice.between(0.3, 1.5);
                if dice.roll() < nature.springy * nature.springy {
                    self.jump();
                }
            }
        }
        // Done, and down from any jump of it: on to something else.
        if (done || self.since > LONGEST) && self.grounded() {
            self.choose(dice, me, everyone, soon);
        }
    }
}

/// Every one of the computer's players in a waiting room, by seat.
#[derive(Clone, Debug)]
pub struct Mingle {
    waiters: Vec<Option<Waiter>>,
    dice: Dice,
}

impl Mingle {
    /// A room with nobody of the computer's in it, its dice thrown from
    /// `seed`: its own dice, so that nothing done here changes what a game
    /// played from a seed comes to.
    pub fn new(seed: u64) -> Self {
        Self {
            waiters: Vec::new(),
            dice: Dice::new(seed ^ 0x6D69_6E67_6C65),
        }
    }

    /// Where the next to come into the room stands, a person or one of
    /// the computer's: somewhere about its middle, thrown, clear of
    /// `everyone` there already if there is the room, facing the middle.
    /// By nothing of their seat, which is their team: until 2026-11-16
    /// each stood at their seat's own place on a ring, and where anyone
    /// stood said which team they were on.
    pub fn arrive(&mut self, everyone: &[Option<Vec2>]) -> Stood {
        let dice = &mut self.dice;
        let mut throw = || round(Vec2::ZERO, dice.between(0.0, TAU), dice.between(ARRIVES.0, ARRIVES.1));
        let mut at = throw();
        for _ in 0..12 {
            if everyone.iter().flatten().all(|other| other.distance(at) >= ARRIVES_CLEAR) {
                break;
            }
            at = throw();
        }
        Stood {
            at,
            up: 0.0,
            yaw: heading(-at),
        }
    }

    /// One of the computer's players, of the kind `persona`, has come
    /// into the room in `seat`, standing as `stood`: it stands a moment,
    /// as someone who has just come in does, and is off.
    pub fn enter(&mut self, seat: usize, persona: &Persona, stood: Stood) {
        if self.waiters.len() <= seat {
            self.waiters.resize(seat + 1, None);
        }
        let nature = Nature::of(persona, &mut self.dice);
        let dice = &mut self.dice;
        self.waiters[seat] = Some(Waiter {
            nature,
            at: inside(stood.at),
            up: 0.0,
            lift: 0.0,
            yaw: stood.yaw,
            doing: Doing::Rest {
                left: dice.between(0.15, 1.1) * (1.3 - nature.lively),
                glance: dice.between(0.1, 0.6),
                at: None,
            },
            since: 0.0,
            hop_in: dice.between(0.1, 1.0),
            swing: dice.between(0.0, TAU),
            swing_rate: dice.between(2.0, 4.5),
        });
    }

    /// Nobody of the computer's is in `seat` any more.
    pub fn leave(&mut self, seat: usize) {
        if let Some(waiter) = self.waiters.get_mut(seat) {
            *waiter = None;
        }
    }

    /// Nobody of the computer's is in the room any more: the fight has
    /// begun.
    pub fn clear(&mut self) {
        self.waiters.clear();
    }

    /// Whether the computer's player in `seat` is in the room.
    pub fn has(&self, seat: usize) -> bool {
        self.waiters.get(seat).is_some_and(Option::is_some)
    }

    /// Where the computer's player in `seat` stands, if it is in the
    /// room.
    pub fn stood(&self, seat: usize) -> Option<Stood> {
        Some(self.waiters.get(seat)?.as_ref()?.stood())
    }

    /// A frame of the one in `seat`, with `everyone` in the room where
    /// they stand across its floor from its middle, by seat (nobody in a
    /// seat nobody is in), and `soon`, the room counting down: where it
    /// stands after it.
    pub fn step(&mut self, seat: usize, everyone: &[Option<Vec2>], soon: bool, dt: f32) -> Option<Stood> {
        let waiter = self.waiters.get_mut(seat)?.as_mut()?;
        waiter.step(&mut self.dice, seat, everyone, soon, dt);
        Some(waiter.stood())
    }

    /// What the one in `seat` is doing, by its name ([`WAY_NAMES`]): for
    /// telling what they did over a while.
    pub fn doing(&self, seat: usize) -> Option<&'static str> {
        let waiter = self.waiters.get(seat)?.as_ref()?;
        Some(WAY_NAMES[waiter.doing.way()])
    }
}

#[cfg(test)]
mod tests {
    use super::super::PERSONAS;
    use super::*;

    /// How high a jump goes, in metres: the game's 1.64.
    fn jump_height() -> f32 {
        JUMP_UP * JUMP_UP / (2.0 * RISE_GRAVITY)
    }

    /// What one of a kind did over a while in a room with two others
    /// standing in it: how far it went, how many times it left the floor,
    /// what share of the time it stood, and how long it spent at each
    /// thing.
    struct Told {
        went: f32,
        jumps: u32,
        stood: f32,
        ways: [f32; WAYS],
    }

    fn watch(persona: &Persona, seed: u64, secs: f32) -> Told {
        let mut mingle = Mingle::new(seed);
        let mut everyone = vec![None; 8];
        everyone[0] = Some(Vec2::new(-3.0, 2.0));
        everyone[5] = Some(Vec2::new(6.0, -7.0));
        let stood = mingle.arrive(&everyone);
        mingle.enter(2, persona, stood);
        let dt = 1.0 / 60.0;
        let mut told = Told {
            went: 0.0,
            jumps: 0,
            stood: 0.0,
            ways: [0.0; WAYS],
        };
        let (mut was, mut aloft) = (stood, false);
        for frame in 0..(secs / dt) as usize {
            // Someone walking about, for whoever follows them.
            let turn = frame as f32 * dt * 0.4;
            everyone[0] = Some(Vec2::new(turn.cos(), turn.sin()) * 5.0);
            let now = mingle.step(2, &everyone, false, dt).expect("it is in the room");
            assert!(now.at.x.abs() <= HALF + 1e-3 && now.at.y.abs() <= HALF + 1e-3, "{} out of the room at {}", persona.name, now.at);
            // (A jump stepped a frame at a time tops out a frame's rise
            // over what the numbers come to.)
            assert!(now.up >= 0.0 && now.up <= jump_height() + 0.15, "{} is {} m up", persona.name, now.up);
            assert!(now.at.is_finite() && now.yaw.is_finite());
            // No faster than a run, and a frame's slack.
            let step = now.at.distance(was.at);
            assert!(step <= RUN_SPEED * dt * 1.05 + 1e-4, "{} went {step} m in a frame", persona.name);
            told.went += step;
            if step < 1e-4 && now.up == 0.0 {
                told.stood += dt;
            }
            if now.up > 0.0 && !aloft {
                told.jumps += 1;
            }
            aloft = now.up > 0.0;
            let way = WAY_NAMES.iter().position(|name| Some(*name) == mingle.doing(2)).unwrap();
            told.ways[way] += dt;
            was = now;
        }
        told
    }

    /// Over many of a kind, a minute each: metres a minute, jumps a
    /// minute, and the share of the time stood still.
    fn over_many(persona: &Persona) -> (f32, f32, f32, [f32; WAYS]) {
        let many = 40;
        let (mut went, mut jumps, mut stood, mut ways) = (0.0, 0.0, 0.0, [0.0; WAYS]);
        for seed in 0..many {
            let told = watch(persona, 1000 + seed, 60.0);
            went += told.went;
            jumps += told.jumps as f32;
            stood += told.stood / 60.0;
            for (sum, way) in ways.iter_mut().zip(told.ways) {
                *sum += way / 60.0;
            }
        }
        let many = many as f32;
        (went / many, jumps / many, stood / many, ways.map(|way| way / many))
    }

    #[test]
    fn everyone_keeps_to_the_room_and_nobody_only_stands_or_never_does() {
        for persona in &PERSONAS {
            let (went, jumps, stood, ways) = over_many(persona);
            let told: Vec<String> = WAY_NAMES.iter().zip(ways).map(|(name, share)| format!("{name} {:.0}%", share * 100.0)).collect();
            println!("{}: {went:.0} m a minute, {jumps:.1} jumps, stood {:.0}% | {}", persona.name, stood * 100.0, told.join(", "));
            // Even the stillest of them goes somewhere, and the liveliest
            // stands now and then.
            assert!(went > 40.0, "{} went {went} m in a minute", persona.name);
            assert!(stood > 0.03 && stood < 0.8, "{} stood {stood} of the time", persona.name);
            // Nobody does one thing only.
            assert!(ways.iter().filter(|share| **share > 0.02).count() >= 4, "{}: {ways:?}", persona.name);
        }
    }

    #[test]
    fn each_passes_the_time_as_who_it_is() {
        let of = |name: &str| over_many(PERSONAS.iter().find(|persona| persona.name == name).unwrap());
        let (spirit, defender, troll, tinkerer) = (of("ENFP"), of("ISFJ"), of("ESFP"), of("INTP"));
        // The free spirit is all over the room, jumping; the defender
        // mostly stands, and hardly leaves the floor.
        // (Not that the defender is rooted to the spot: it paces, and goes
        // to the glass, at a walk.)
        assert!(spirit.0 > 1.5 * defender.0, "the free spirit went {} m, the defender {}", spirit.0, defender.0);
        assert!(spirit.1 > 5.0 * defender.1.max(0.2), "the free spirit jumped {}, the defender {}", spirit.1, defender.1);
        assert!(defender.2 > 1.5 * spirit.2, "the defender stood {}, the free spirit {}", defender.2, spirit.2);
        // The troll is at someone's heels or round and round them far
        // more than the defender is; the tinkerer is at the glass far
        // more than the free spirit.
        let pestering = |told: &(f32, f32, f32, [f32; WAYS])| told.3[TAIL] + told.3[RING];
        assert!(pestering(&troll) > 4.0 * pestering(&defender).max(0.005), "{} and {}", pestering(&troll), pestering(&defender));
        assert!(tinkerer.3[GAZE] > 2.0 * spirit.3[GAZE], "{} and {}", tinkerer.3[GAZE], spirit.3[GAZE]);
        // The defender paces, which the free spirit has no time for.
        assert!(defender.3[PACE] > 2.0 * spirit.3[PACE], "{} and {}", defender.3[PACE], spirit.3[PACE]);
    }

    #[test]
    fn two_of_a_kind_are_not_alike_and_a_seed_is_the_same_room_again() {
        let persona = &PERSONAS[5];
        let run = |seed: u64| {
            let mut mingle = Mingle::new(seed);
            let everyone = vec![None; 8];
            let stood = mingle.arrive(&everyone);
            mingle.enter(1, persona, stood);
            let mut path = Vec::new();
            for _ in 0..600 {
                path.push(mingle.step(1, &everyone, false, 1.0 / 60.0).unwrap());
            }
            path
        };
        assert_eq!(run(7), run(7));
        assert_ne!(run(7), run(8));
    }

    #[test]
    fn whoever_comes_in_stands_clear_of_everyone_about_the_middle_facing_it() {
        let mut mingle = Mingle::new(3);
        let mut everyone: Vec<Option<Vec2>> = vec![None; 8];
        let mut angles = Vec::new();
        for seat in 0..8 {
            let stood = mingle.arrive(&everyone);
            let out = stood.at.length();
            assert!(out >= ARRIVES.0 - 1e-3 && out <= ARRIVES.1 + 1e-3, "{out} m from the middle");
            for other in everyone.iter().flatten() {
                assert!(other.distance(stood.at) >= ARRIVES_CLEAR, "{} m from someone", other.distance(stood.at));
            }
            // Facing the middle: ahead is -Z turned by its yaw.
            let ahead = Vec2::new(-stood.yaw.sin(), -stood.yaw.cos());
            assert!(ahead.dot(-stood.at.normalize()) > 0.999, "seat {seat} faces {ahead}");
            angles.push(stood.at.y.atan2(stood.at.x));
            everyone[seat] = Some(stood.at);
        }
        // And not on a ring an eighth of a turn apart, as they were.
        let mut gaps: Vec<f32> = angles.windows(2).map(|pair| (pair[1] - pair[0]).rem_euclid(TAU)).collect();
        gaps.sort_by(f32::total_cmp);
        assert!(gaps[gaps.len() - 1] - gaps[0] > 0.3, "{gaps:?}");
    }

    #[test]
    fn someone_who_goes_up_to_another_gets_there_and_one_alone_goes_to_nobody() {
        // Alone in the room, nothing it takes up needs anyone.
        let mut mingle = Mingle::new(11);
        let nobody = vec![None; 8];
        let stood = mingle.arrive(&nobody);
        mingle.enter(3, &PERSONAS[1], stood);
        for _ in 0..3600 {
            mingle.step(3, &nobody, true, 1.0 / 60.0);
            assert!(!matches!(mingle.doing(3), Some("visit" | "tail" | "ring")), "{:?} with nobody there", mingle.doing(3));
        }
        // With someone standing in a corner, the troll is beside them
        // before long.
        let mut everyone = vec![None; 8];
        let corner = Vec2::new(10.0, -10.0);
        everyone[6] = Some(corner);
        let mut nearest = f32::INFINITY;
        for _ in 0..7200 {
            let now = mingle.step(3, &everyone, false, 1.0 / 60.0).unwrap();
            nearest = nearest.min(now.at.distance(corner));
        }
        assert!(nearest < 3.6, "never nearer than {nearest} m");
    }
}
