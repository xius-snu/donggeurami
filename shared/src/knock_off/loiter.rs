//! What the computer's players do on the ice while a game of Knock Off waits
//! for its people (mine; the spec: "the computer's players ... come in one
//! at a time and stand about (or wander a little)"): each, as it comes out
//! onto the ice, stands a while, looks round at someone or at the middle,
//! and now and then strolls a few metres somewhere else on it, well in from
//! the edge and clear of everyone, until the game stands everyone on their
//! spot. One piece for the server's computer players online and the
//! device's own offline, as `bots::mingle` is for Bridge Wars' waiting room.
//!
//! **Nothing here is a rule** (Hajun: "if some thing becomes a definite
//! if-then, then its not a real human, but a program"): how long one
//! stands, whether it then strolls, looks round or stands on, how far it
//! goes and how fast, are each thrown afresh every time.
//!
//! Only where each stands and which way it faces comes of it, which is all
//! anyone is sent of a body.

use std::f32::consts::{PI, TAU};

use bevy::math::Vec2;

use crate::bots::Dice;

/// How far from the middle of the ice a stroll ends, in metres, at the
/// most: well in from the edge, 12 out, so that nobody watching wonders
/// whether it will walk off.
const STROLL_WITHIN: f32 = 8.5;
/// How far a stroll goes, in metres, at the least and at the most.
const STROLL: (f32, f32) = (1.2, 5.0);
/// How far from everyone else a stroll ends, in metres, if there is room.
const CLEAR: f32 = 1.6;
/// How fast it strolls, in metres a second, at the least and at the most:
/// a walk, a good deal slower than a run, so that the legs are seen to
/// stroll.
const PACE: (f32, f32) = (1.4, 2.8);
/// How long it stands before it does anything else, in seconds, at the
/// least and at the most.
const STAND: (f32, f32) = (1.0, 4.5);
/// How fast it turns, in radians a second.
const TURN: f32 = 4.5;
/// This near where it was going, it is there, in metres.
const THERE: f32 = 0.05;

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

    /// The player in `seat`, `dt` seconds on, with everyone on the ice
    /// where `everyone` has them (by seat, it too; none for nobody): where
    /// it stands now (x, z) and which way it faces. None for a seat it does
    /// not have.
    pub fn step(&mut self, seat: usize, everyone: &[Option<Vec2>], dt: f32) -> Option<(Vec2, f32)> {
        let mut it = (*self.them.get(seat)?)?;
        let dt = dt.clamp(0.0, 0.25);
        match it.to {
            Some(to) => {
                let off = to - it.at;
                let far = off.length();
                let step = it.pace * dt;
                if far <= step.max(THERE) {
                    it.at = to;
                    it.to = None;
                    it.face = it.yaw;
                    it.wait = self.dice.around(STAND.0, STAND.1);
                } else {
                    it.at += off / far * step;
                    it.yaw = turned(it.yaw, facing(off), TURN * dt);
                }
            }
            None => {
                it.yaw = turned(it.yaw, it.face, TURN * dt);
                it.wait -= dt;
                if it.wait <= 0.0 {
                    self.next(seat, &mut it, everyone);
                }
            }
        }
        self.them[seat] = Some(it);
        Some((it.at, it.yaw))
    }

    /// What the player in `seat`, standing, does next: strolls somewhere
    /// else on the ice, looks round at someone or at the middle, or stands
    /// on, as the dice say, each about as often as the others but a stroll
    /// a little the oftenest.
    fn next(&mut self, seat: usize, it: &mut Stroller, everyone: &[Option<Vec2>]) {
        let throw = self.dice.roll();
        let stroll = self.dice.around(0.35, 0.6);
        let look = stroll + self.dice.around(0.2, 0.35);
        if throw < stroll
            && let Some(to) = self.somewhere(seat, it.at, everyone)
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
            let toward = others.get(pick).copied().unwrap_or(Vec2::ZERO) - it.at;
            if toward.length() > 0.3 {
                it.face = facing(toward);
            }
        }
        it.wait = self.dice.around(STAND.0, STAND.1);
    }

    /// Somewhere a stroll from `from` ends: a few metres off, well in from
    /// the edge and clear of everyone; the best of a few thrown, or none
    /// with nowhere near enough clear.
    fn somewhere(&mut self, seat: usize, from: Vec2, everyone: &[Option<Vec2>]) -> Option<Vec2> {
        let mut best: Option<(f32, Vec2)> = None;
        for _ in 0..8 {
            let angle = self.dice.between(0.0, TAU);
            let far = self.dice.between(STROLL.0, STROLL.1);
            let to = from + Vec2::from_angle(angle) * far;
            if to.length() > STROLL_WITHIN {
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
            if room >= CLEAR {
                break;
            }
        }
        best.filter(|(room, _)| *room >= CLEAR * 0.75).map(|(_, to)| to)
    }
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
    use super::*;

    #[test]
    fn they_stand_and_stroll_about_well_in_from_the_edge_and_apart() {
        for seed in 0..40 {
            let mut loiter = Loiter::new(seed);
            let mut everyone: Vec<Option<Vec2>> = (0..4).map(|seat| Some(Vec2::from_angle(0.8 + seat as f32 * 1.57) * 7.0)).collect();
            for seat in 1..4 {
                loiter.enter(seat, everyone[seat].expect("there"), 0.0);
            }
            assert!(!loiter.has(0) && loiter.has(2));
            let (mut walked, mut stood) = ([0.0f32; 4], [0.0f32; 4]);
            for _ in 0..(60 * 30) {
                for seat in 1..4 {
                    let (at, yaw) = loiter.step(seat, &everyone, 1.0 / 60.0).expect("on the ice");
                    assert!(yaw.is_finite());
                    // Never past where a stroll may end, but from where it
                    // stood on its spot, which is in from that.
                    assert!(at.length() <= STROLL_WITHIN.max(7.0) + 1e-3, "seed {seed} seat {seat} at {at}");
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
        }
        // Over many tables nearly every one of them strolls somewhere.
        let strolled = (0..60)
            .filter(|&seed| {
                let mut loiter = Loiter::new(seed);
                let mut everyone = vec![None, Some(Vec2::new(5.0, 5.0))];
                loiter.enter(1, Vec2::new(5.0, 5.0), 0.0);
                (0..(60 * 20)).any(|_| {
                    let (at, _) = loiter.step(1, &everyone, 1.0 / 60.0).expect("on the ice");
                    everyone[1] = Some(at);
                    at.distance(Vec2::new(5.0, 5.0)) > 0.5
                })
            })
            .count();
        assert!(strolled >= 54, "{strolled} of 60 strolled");
    }

    #[test]
    fn whoever_has_gone_is_not_stepped_and_the_same_seed_strolls_the_same() {
        let run = |seed: u64| {
            let mut loiter = Loiter::new(seed);
            loiter.enter(2, Vec2::new(-5.0, 4.0), 1.0);
            let everyone = vec![None, None, Some(Vec2::new(-5.0, 4.0))];
            (0..600).filter_map(|_| loiter.step(2, &everyone, 1.0 / 60.0)).last()
        };
        assert_eq!(run(4), run(4));
        let mut loiter = Loiter::new(1);
        loiter.enter(1, Vec2::ZERO, 0.0);
        loiter.leave(1);
        assert_eq!(loiter.step(1, &[], 0.1), None);
        assert_eq!(loiter.step(7, &[], 0.1), None);
        loiter.enter(3, Vec2::ONE, 0.0);
        loiter.clear();
        assert!(!loiter.has(3));
    }
}
