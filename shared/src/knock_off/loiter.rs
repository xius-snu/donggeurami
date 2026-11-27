//! What the computer's players do on the ice while a game of Knock Off waits
//! for its people (mine; the spec: "the computer's players ... come in one
//! at a time and stand about (or wander a little)"): each, as it comes out
//! onto the ice, stands a while, looks round at someone or at the middle,
//! and now and then strolls a few metres somewhere else on it, well in from
//! the edge and clear of everyone, never across a bay of the sea, until the
//! game begins, wherever each then stands (since 2026-11-27, Hajun: "i dont
//! want fixed starting positions"). One piece for the server's computer
//! players online and the device's own offline, as `bots::mingle` is for
//! Bridge Wars' waiting room.
//!
//! **Nobody walks through anybody while the table waits** (Hajun: "collision
//! matters during waiting period too so u cant start on top of each other"):
//! one of them strolling stops short of whoever is in its way, and one
//! someone walks into steps back out of their way, onto the ice, never off
//! it, so that nobody begins on top of anyone. A person's own device keeps
//! their body clear of everyone's, theirs among them.
//!
//! **Nothing here is a rule** (Hajun: "if some thing becomes a definite
//! if-then, then its not a real human, but a program"): how long one
//! stands, whether it then strolls, looks round or stands on, how far it
//! goes and how fast, and how far in from the edge and how clear of
//! everyone it wants to end up, are each thrown afresh every time.
//!
//! Only where each stands and which way it faces comes of it, which is all
//! anyone is sent of a body.

use std::f32::consts::{PI, TAU};

use bevy::math::Vec2;

use super::{BODY, Floe, STOOD_BACK};
use crate::bots::Dice;

/// How far in from the edge of the ice a stroll may end, in metres, thrown
/// for each stroll from between these (no further in than most of the way
/// to the deepest spot, on an island that narrow): well in, so that nobody
/// watching wonders whether it will walk off.
const STROLL_IN: (f32, f32) = (1.6, 3.2);
/// How far a stroll goes, in metres, at the least and at the most.
const STROLL: (f32, f32) = (1.2, 5.0);
/// How much room from everyone else a stroll looks for where it ends, in
/// metres, thrown for each stroll from between these; and how little of
/// that it settles for, rather than stand on, a share of it thrown too.
const CLEAR: (f32, f32) = (1.6, 2.6);
const SETTLES_FOR: (f32, f32) = (0.55, 0.95);
/// How much room every body keeps from every other while the table waits,
/// middle to middle, in metres: two bodies as the slide has them
/// ([`BODY`]), so that wherever anyone begins, nobody begins on anyone.
pub const ROOM: f32 = 2.0 * BODY;
/// How fast it strolls, in metres a second, at the least and at the most:
/// a walk, a good deal slower than a run, so that the legs are seen to
/// stroll.
const PACE: (f32, f32) = (1.4, 2.8);
/// How fast it steps back out of the way of someone who has walked into
/// it, in metres a second at the most: a quick step, slower than they walk.
const GIVE_WAY: f32 = 3.0;
/// How long it stands before it does anything else, in seconds, at the
/// least and at the most.
const STAND: (f32, f32) = (1.0, 4.5);
/// How fast it turns, in radians a second.
const TURN: f32 = 4.5;
/// This near where it was going, it is there, in metres.
const THERE: f32 = 0.05;
/// How far apart along a stroll it looks that the ice is under it, in
/// metres.
const LOOK_ALONG: f32 = 0.4;

/// One of them on the ice.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Stroller {
    at: Vec2,
    /// Its turn about the upright, in radians (-Z turned by it is ahead).
    yaw: f32,
    /// Where it is strolling to, and how fast; none standing.
    to: Option<Vec2>,
    pace: f32,
    /// Which way it is turning to face, standing.
    face: f32,
    /// How much longer it stands before it does anything else.
    wait: f32,
}

/// The computer's players on the ice while a table waits, by seat.
#[derive(Clone, Debug)]
pub struct Loiter {
    dice: Dice,
    them: Vec<Option<Stroller>>,
}

impl Loiter {
    /// Nobody yet, its dice thrown from `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            dice: Dice::new(seed),
            them: Vec::new(),
        }
    }

    /// The computer's player in `seat` has come out onto the ice, standing
    /// at `at` (across the ground: x, z) and facing `yaw`: it stands a
    /// moment first.
    pub fn enter(&mut self, seat: usize, at: Vec2, yaw: f32) {
        if self.them.len() <= seat {
            self.them.resize(seat + 1, None);
        }
        let wait = self.dice.around(STAND.0, STAND.1);
        self.them[seat] = Some(Stroller {
            at,
            yaw,
            to: None,
            pace: 0.0,
            face: yaw,
            wait,
        });
    }

    /// Whether the player in `seat` is on the ice here.
    pub fn has(&self, seat: usize) -> bool {
        self.them.get(seat).is_some_and(Option::is_some)
    }

    /// The player in `seat` is here no more.
    pub fn leave(&mut self, seat: usize) {
        if let Some(it) = self.them.get_mut(seat) {
            *it = None;
        }
    }

    /// Everyone has gone: the game has begun, and places them itself.
    pub fn clear(&mut self) {
        self.them.clear();
    }

    /// The player in `seat`, `dt` seconds on, on `floe` (whole: nothing
    /// breaks while a table waits), with everyone on the ice where
    /// `everyone` has them (by seat, it too; none for nobody): where it
    /// stands now (x, z) and which way it faces. None for a seat it does not
    /// have.
    pub fn step(&mut self, seat: usize, everyone: &[Option<Vec2>], floe: &Floe, dt: f32) -> Option<(Vec2, f32)> {
        let mut it = (*self.them.get(seat)?)?;
        let dt = dt.clamp(0.0, 0.25);
        let others: Vec<Vec2> = everyone
            .iter()
            .enumerate()
            .filter(|&(other, _)| other != seat)
            .filter_map(|(_, at)| *at)
            .collect();
        match it.to {
            Some(to) => {
                let off = to - it.at;
                let far = off.length();
                let step = it.pace * dt;
                let next = if far <= step.max(THERE) { to } else { it.at + off / far * step };
                // Someone in its way: it stops short of them and stands,
                // facing the way it was going.
                let in_the_way = others.iter().any(|&other| next.distance(other) < ROOM && next.distance(other) < it.at.distance(other));
                if in_the_way {
                    it.to = None;
                    it.face = facing(off);
                    it.wait = self.dice.around(STAND.0, STAND.1);
                } else if far <= step.max(THERE) {
                    it.at = to;
                    it.to = None;
                    it.face = it.yaw;
                    it.wait = self.dice.around(STAND.0, STAND.1);
                } else {
                    it.at = next;
                    it.yaw = turned(it.yaw, facing(off), TURN * dt);
                }
            }
            None => {
                it.yaw = turned(it.yaw, it.face, TURN * dt);
                it.wait -= dt;
                if it.wait <= 0.0 {
                    self.next(seat, &mut it, everyone, floe);
                }
            }
        }
        // Whoever has walked into it: it steps back out of their way, on
        // the ice.
        let mut away = Vec2::ZERO;
        for &other in &others {
            let apart = it.at - other;
            let far = apart.length();
            if far < ROOM {
                let way = apart.try_normalize().unwrap_or_else(|| (it.at - floe.middle(1.0)).normalize_or(Vec2::X));
                away += way * (ROOM - far);
            }
        }
        if away.length_squared() > 1e-10 {
            let step = away.clamp_length_max(GIVE_WAY * dt);
            it.at = floe.nearest_inside(it.at + step, 1.0, STOOD_BACK);
        }
        self.them[seat] = Some(it);
        Some((it.at, it.yaw))
    }

    /// What the player in `seat`, standing, does next: strolls somewhere
    /// else on the ice, looks round at someone or at the middle, or stands
    /// on, as the dice say, each about as often as the others but a stroll
    /// a little the oftenest.
    fn next(&mut self, seat: usize, it: &mut Stroller, everyone: &[Option<Vec2>], floe: &Floe) {
        let throw = self.dice.roll();
        let stroll = self.dice.around(0.35, 0.6);
        let look = stroll + self.dice.around(0.2, 0.35);
        if throw < stroll
            && let Some(to) = self.somewhere(seat, it.at, everyone, floe)
        {
            it.to = Some(to);
            it.pace = self.dice.around(PACE.0, PACE.1);
            return;
        }
        if throw < look {
            // At whoever of the others it throws, or at the middle.
            let others: Vec<Vec2> = everyone
                .iter()
                .enumerate()
                .filter(|&(other, _)| other != seat)
                .filter_map(|(_, at)| *at)
                .filter(|at| at.distance(it.at) > 0.5)
                .collect();
            let pick = (self.dice.roll() * (others.len() + 1) as f32) as usize;
            let toward = others.get(pick).copied().unwrap_or_else(|| floe.middle(1.0)) - it.at;
            if toward.length() > 0.3 {
                it.face = facing(toward);
            }
        }
        it.wait = self.dice.around(STAND.0, STAND.1);
    }

    /// Somewhere a stroll from `from` ends: a few metres off, well in from
    /// the edge, clear of everyone, and with ice all the way there (never
    /// across a bay of the sea); the best of a few thrown, or none with
    /// nowhere near enough clear. How far in, how clear and how near enough
    /// are each thrown afresh for every stroll (Hajun's rule for the
    /// computer's players), so that whether one strolls, and where to, is
    /// never one line drawn on the ice.
    fn somewhere(&mut self, seat: usize, from: Vec2, everyone: &[Option<Vec2>], floe: &Floe) -> Option<Vec2> {
        let within = self.dice.around(STROLL_IN.0, STROLL_IN.1).min(0.7 * floe.deepest(1.0));
        let clear = self.dice.around(CLEAR.0, CLEAR.1);
        let enough = clear * self.dice.around(SETTLES_FOR.0, SETTLES_FOR.1);
        let mut best: Option<(f32, Vec2)> = None;
        for _ in 0..10 {
            let angle = self.dice.between(0.0, TAU);
            let far = self.dice.between(STROLL.0, STROLL.1);
            let to = from + Vec2::from_angle(angle) * far;
            if floe.depth(to, 1.0) < within || !ice_all_the_way(floe, from, to) {
                continue;
            }
            let room = everyone
                .iter()
                .enumerate()
                .filter(|&(other, _)| other != seat)
                .filter_map(|(_, at)| *at)
                .map(|at| at.distance(to))
                .fold(f32::INFINITY, f32::min);
            if best.is_none_or(|(most, _)| room > most) {
                best = Some((room, to));
            }
            if room >= clear {
                break;
            }
        }
        best.filter(|(room, _)| *room >= enough).map(|(_, to)| to)
    }
}

/// Whether there is ice under a body all the way from `from` to `to`, its
/// middle never nearer the edge than half a body.
fn ice_all_the_way(floe: &Floe, from: Vec2, to: Vec2) -> bool {
    let steps = (from.distance(to) / LOOK_ALONG).ceil().max(1.0) as usize;
    (1..=steps).all(|k| floe.depth(from.lerp(to, k as f32 / steps as f32), 1.0) >= 0.5 * BODY)
}

/// Which way something faces that looks along `way` across the ground (x,
/// z): its turn about the upright from -Z.
fn facing(way: Vec2) -> f32 {
    (-way.x).atan2(-way.y)
}

/// `from` turned toward `to` by no more than `most`, the short way round.
fn turned(from: f32, to: f32, most: f32) -> f32 {
    let off = (to - from + PI).rem_euclid(TAU) - PI;
    from + off.clamp(-most, most)
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3Swizzles;

    use super::*;

    #[test]
    fn they_stand_and_stroll_about_well_in_from_the_edge_and_apart() {
        for seed in 0..40 {
            let floe = Floe::thrown(seed + 500);
            let mut loiter = Loiter::new(seed);
            let mut everyone: Vec<Option<Vec2>> = (0..4).map(|seat| Some(super::super::waiting_spot(seat, &floe).0.xz())).collect();
            for seat in 1..4 {
                loiter.enter(seat, everyone[seat].expect("there"), 0.0);
            }
            assert!(!loiter.has(0) && loiter.has(2));
            let (mut walked, mut stood) = ([0.0f32; 4], [0.0f32; 4]);
            for _ in 0..(60 * 30) {
                for seat in 1..4 {
                    let (at, yaw) = loiter.step(seat, &everyone, &floe, 1.0 / 60.0).expect("on the ice");
                    assert!(yaw.is_finite());
                    // Never off the ice, nor on anyone.
                    assert!(floe.depth(at, 1.0) >= 0.0, "seed {seed} seat {seat} at {at}, {} in", floe.depth(at, 1.0));
                    let was = everyone[seat].expect("there");
                    if at.distance(was) > 1e-4 {
                        walked[seat] += at.distance(was);
                    } else {
                        stood[seat] += 1.0 / 60.0;
                    }
                    everyone[seat] = Some(at);
                }
            }
            for seat in 1..4 {
                // Half a minute: each stood a good while, and few never
                // strolled at all.
                assert!(stood[seat] > 8.0, "seed {seed} seat {seat} stood {}", stood[seat]);
                assert!(walked[seat] < 60.0, "seed {seed} seat {seat} walked {}", walked[seat]);
            }
            for one in 1..4 {
                for other in one + 1..4 {
                    let apart = everyone[one].unwrap().distance(everyone[other].unwrap());
                    assert!(apart > ROOM - 0.1, "seed {seed}: {one} and {other} {apart} apart");
                }
            }
        }
        // Over many tables nearly every one of them strolls somewhere.
        let strolled = (0..60)
            .filter(|&seed| {
                let floe = Floe::thrown(seed);
                let mut loiter = Loiter::new(seed);
                let from = floe.middle(1.0);
                let mut everyone = vec![None, Some(from)];
                loiter.enter(1, from, 0.0);
                (0..(60 * 20)).any(|_| {
                    let (at, _) = loiter.step(1, &everyone, &floe, 1.0 / 60.0).expect("on the ice");
                    everyone[1] = Some(at);
                    at.distance(from) > 0.5
                })
            })
            .count();
        assert!(strolled >= 54, "{strolled} of 60 strolled");
    }

    #[test]
    fn one_walked_into_steps_out_of_the_way_and_one_strolling_stops_short() {
        let floe = Floe::round(12.0);
        let mut loiter = Loiter::new(3);
        loiter.enter(1, Vec2::ZERO, 0.0);
        // Someone walks right up to it, and stands there: it steps back out
        // of their way, as quickly as a step, and no further than room.
        let you = Vec2::new(0.5, 0.0);
        let everyone = [Some(you), Some(Vec2::ZERO)];
        let (first, _) = loiter.step(1, &everyone, &floe, 0.1).unwrap();
        assert!(first.x < 0.0 && first.distance(Vec2::ZERO) <= GIVE_WAY * 0.1 + 1e-4, "{first}");
        let mut at = first;
        for _ in 0..30 {
            at = loiter.step(1, &[Some(you), Some(at)], &floe, 0.1).unwrap().0;
        }
        assert!((at.distance(you) - ROOM).abs() < 0.05, "{at}, {} from you", at.distance(you));
        // Against the edge, it is pushed along it, never off.
        let mut loiter = Loiter::new(4);
        let edge = Vec2::new(12.0 - STOOD_BACK, 0.0);
        loiter.enter(1, edge, 0.0);
        let mut at = edge;
        for _ in 0..40 {
            at = loiter.step(1, &[Some(Vec2::new(11.0, 0.05)), Some(at)], &floe, 0.1).unwrap().0;
            assert!(floe.depth(at, 1.0) >= STOOD_BACK - 0.01, "{at}");
        }
        // One strolling stops short of whoever stands in its way.
        let mut loiter = Loiter::new(5);
        loiter.enter(1, Vec2::new(-4.0, 0.0), 0.0);
        if let Some(it) = &mut loiter.them[1] {
            it.to = Some(Vec2::new(4.0, 0.0));
            it.pace = 2.0;
        }
        let blocker = Vec2::new(0.0, 0.0);
        let mut at = Vec2::new(-4.0, 0.0);
        for _ in 0..200 {
            at = loiter.step(1, &[Some(blocker), Some(at)], &floe, 1.0 / 30.0).unwrap().0;
            assert!(at.distance(blocker) >= ROOM - 0.02, "{at}");
        }
        assert!(at.x < 0.0, "it went through: {at}");
    }

    #[test]
    fn a_stroll_never_crosses_the_sea() {
        for seed in 0..60 {
            let floe = Floe::thrown(seed * 13 + 1);
            let mut loiter = Loiter::new(seed);
            let from = floe.middle(1.0);
            for _ in 0..30 {
                if let Some(to) = loiter.somewhere(1, from, &[None, Some(from)], &floe) {
                    assert!(ice_all_the_way(&floe, from, to), "seed {seed}: {from} to {to}");
                }
            }
        }
    }

    #[test]
    fn whoever_has_gone_is_not_stepped_and_the_same_seed_strolls_the_same() {
        let floe = Floe::thrown(9);
        let run = |seed: u64| {
            let mut loiter = Loiter::new(seed);
            let from = floe.middle(1.0);
            loiter.enter(2, from, 1.0);
            let everyone = vec![None, None, Some(from)];
            (0..600).filter_map(|_| loiter.step(2, &everyone, &floe, 1.0 / 60.0)).last()
        };
        assert_eq!(run(4), run(4));
        let mut loiter = Loiter::new(1);
        loiter.enter(1, Vec2::ZERO, 0.0);
        loiter.leave(1);
        assert_eq!(loiter.step(1, &[], &floe, 0.1), None);
        assert_eq!(loiter.step(7, &[], &floe, 0.1), None);
        loiter.enter(3, Vec2::ONE, 0.0);
        loiter.clear();
        assert!(!loiter.has(3));
    }
}
