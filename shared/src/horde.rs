//! The zombies of sudden death in Bridge Wars, as they have been since the
//! night of 2026-11-16 (Hajun, having looked on at a game's fifteenth
//! minute: "15:00 is when the zombies spawn all over the map, starting
//! slow, and more and more spawn, exponentially until like maybe max 50
//! zombies ... zombie should be invincible, and maybe deal like 20 dmg per
//! hit").
//!
//! One mind for the server's zombies online and the device's own in a game
//! played offline, as the computer's players have one ([`crate::bots`]).
//! Until then the two differed: the server's kept each to the island it
//! rose on, having had no map, and the device's walked wherever their
//! legs took them.
//!
//! * **They come one at a time** ([`Horde::rise`]), as many as the rules
//!   have by then ([`zombies_at`](crate::bridge_wars::zombies_at)), each
//!   where its number puts it ([`zombie_spot`]).
//! * **Each goes for whoever is nearest that it can see** ([`Horde::step`]):
//!   straight at them, at a zombie's pace, over whatever a body can walk,
//!   land or plank, and never off an edge ([`Body::step`]); stopped by
//!   something, it tries round it one way for a moment and then the other.
//!   It hits whoever is in its reach, no oftener than the computer's
//!   players do. With nobody in sight it shambles about where it is.
//! * **Nothing hurts one.** A blow, a blast or a bus only moves it
//!   ([`Horde::knock`], a share of what anyone is knocked by) or lays it
//!   flat; knocked into the void, or with its bridge gone from under it,
//!   it falls, and comes up again where it first did a while later
//!   ([`Horde::stand`]). The game that runs them sees to that, as it sees
//!   to whom a blow lands on.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use crate::bots::{Body, Dice};
use crate::bridge_wars::{BOT_PUNCH_EVERY, HUNT_RADIUS, REACH, ZOMBIE_KNOCK, ZOMBIE_SPEED, zombie_home, zombie_spot};
use crate::nav::{BODY, Nav};

/// How high the islands' tops are, in metres: where a zombie stands whose
/// place the map has no ground at, which is none of them.
const LAND_TOP: f32 = 0.25;
/// How near whoever it is after a zombie stops coming on, as a share of
/// its reach: pressed in on them, not inside them.
const PRESS_IN: f32 = 0.7;
/// How much higher or lower than a zombie someone may be and still be
/// gone for, and still be hit, in metres: not someone jumping a leap
/// potion's height over it, nor on the land above or below.
const SEEN_OVER: f32 = 3.0;
const HIT_OVER: f32 = 1.9;
/// With nobody in sight: how fast it shambles, in metres a second; how
/// far from where it stands it thinks of going; and how long it stands
/// before it thinks of somewhere, in seconds, thrown each time.
const SHAMBLE: f32 = 2.2;
const SHAMBLE_FAR: f32 = 9.0;
const SHAMBLE_EVERY: (f32, f32) = (1.5, 6.0);
/// Stopped by something on its way to someone: how far to one side of
/// straight at them it turns, in radians, for how long, in seconds, thrown
/// each time, before it tries straight at them again; and how little of a
/// frame's step it has to have made to have been stopped.
const ROUND_BY: f32 = 1.2;
const ROUND_FOR: (f32, f32) = (0.5, 1.6);
const STOPPED: f32 = 0.35;
/// How near another zombie one keeps from, in metres, and how much of
/// its pace it gives to getting that far from it: several after one
/// person come at them side by side, not as one.
const APART: f32 = 1.3;
const APART_PACE: f32 = 0.6;

/// Someone a zombie may go for: who, by seat, and where they are. Whoever
/// is dead, unseen or a zombie is not among them, which is the game's to
/// see to.
#[derive(Clone, Copy, Debug)]
pub struct Prey {
    pub seat: usize,
    pub at: Vec3,
}

/// One zombie: its body, how long until it may hit again, where it is
/// shambling to and how long until it thinks of somewhere else, and which
/// way round something it is going and for how much longer.
#[derive(Clone, Copy, Debug)]
struct Zombie {
    body: Body,
    cooldown: f32,
    goal: Option<Vec2>,
    think: f32,
    round: Option<(f32, f32)>,
}

/// Every zombie of a game, in the order they rose.
pub struct Horde {
    zombies: Vec<Zombie>,
    dice: Dice,
}

impl Horde {
    /// None yet; its dice thrown from `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            zombies: Vec::new(),
            dice: Dice::new(seed),
        }
    }

    /// How many have risen.
    pub fn len(&self) -> usize {
        self.zombies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.zombies.is_empty()
    }

    /// The body of the zombie numbered `index`.
    pub fn body(&self, index: usize) -> Option<&Body> {
        self.zombies.get(index).map(|zombie| &zombie.body)
    }

    /// Where the zombie numbered `index` comes up on `nav`, and which way
    /// it faces: its place ([`zombie_spot`]) on whatever ground is there,
    /// toward the middle of the ground it comes up on.
    pub fn spot(nav: &Nav, index: usize) -> Body {
        let at = zombie_spot(index);
        let (home, _) = zombie_home(index);
        let toward = (home - at).normalize_or(Vec2::NEG_Y);
        let height = nav.ground(at).unwrap_or(LAND_TOP);
        Body::standing(Vec3::new(at.x, height, at.y), (-toward.x).atan2(-toward.y))
    }

    /// The next zombie rises: its number, the next after the last.
    pub fn rise(&mut self, nav: &Nav) -> usize {
        let index = self.zombies.len();
        let think = self.dice.around(SHAMBLE_EVERY.0, SHAMBLE_EVERY.1);
        self.zombies.push(Zombie {
            body: Self::spot(nav, index),
            cooldown: 0.0,
            goal: None,
            think,
            round: None,
        });
        index
    }

    /// The zombie numbered `index` comes up again where it first did,
    /// having gone into the void.
    pub fn stand(&mut self, nav: &Nav, index: usize) {
        let think = self.dice.around(SHAMBLE_EVERY.0, SHAMBLE_EVERY.1);
        if let Some(zombie) = self.zombies.get_mut(index) {
            *zombie = Zombie {
                body: Self::spot(nav, index),
                cooldown: 0.0,
                goal: None,
                think,
                round: None,
            };
        }
    }

    /// A blow or a blast from `from` knocks the zombie numbered `index`,
    /// thrown on the run if `running`: by a share of what it would knock
    /// anyone by ([`ZOMBIE_KNOCK`]). It keeps coming.
    pub fn knock(&mut self, index: usize, from: Vec2, running: bool) {
        if let Some(zombie) = self.zombies.get_mut(index) {
            zombie.body.knock(from, running, ZOMBIE_KNOCK);
        }
    }

    /// A knock that has already landed on the body of the zombie numbered
    /// `index`, as the device lands one on a body it draws, `across` the
    /// ground and `up`, in metres a second: its to be carried by, by the
    /// same share.
    pub fn shoved(&mut self, index: usize, across: Vec2, up: f32) {
        if let Some(zombie) = self.zombies.get_mut(index) {
            zombie.body.shove = zombie.body.shove * 0.5 + across * ZOMBIE_KNOCK;
            let up = up.max(0.0) * ZOMBIE_KNOCK;
            if up > 0.0 {
                zombie.body.lift = (zombie.body.lift * 0.5 + up).min(up);
            }
        }
    }

    /// A frame of the zombie numbered `index`, `dt` seconds of it, over
    /// `nav`, with everyone it may go for where they are (`prey`): it is
    /// moved, and whom it hits, if anyone, is given back, by seat. `flat`,
    /// a bus has run it over: it lies where it is and does nothing.
    pub fn step(&mut self, index: usize, nav: &Nav, prey: &[Prey], flat: bool, dt: f32) -> Option<usize> {
        // Which way the others near it are, to keep a little apart from
        // them.
        let me = self.zombies.get(index)?.body.at;
        let crowded = self
            .zombies
            .iter()
            .enumerate()
            .filter(|&(other, zombie)| other != index && (zombie.body.at.y - me.y).abs() < 1.0)
            .filter_map(|(other, zombie)| {
                let off = me.xz() - zombie.body.at.xz();
                let away = off.length();
                // Two on the very same spot part by their numbers.
                let way = if away > 1e-3 { off / away } else { Vec2::from_angle(index as f32 * 2.4 - other as f32) };
                (away < APART).then(|| way * (1.0 - away / APART))
            })
            .fold(Vec2::ZERO, |sum, push| sum + push);
        let zombie = self.zombies.get_mut(index)?;
        zombie.cooldown = (zombie.cooldown - dt).max(0.0);
        if flat {
            zombie.body.shove = Vec2::ZERO;
            zombie.body.lift = 0.0;
            zombie.round = None;
            return None;
        }
        let here = zombie.body.at.xz();
        let height = zombie.body.at.y;
        // Whoever is nearest within its sight, and about as high as it is.
        let after = prey
            .iter()
            .filter(|prey| (prey.at.y - height).abs() <= SEEN_OVER)
            .map(|prey| (prey.at.xz().distance_squared(here), prey))
            .filter(|(away, _)| *away <= HUNT_RADIUS * HUNT_RADIUS)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, prey)| *prey);
        let mut walk = Vec2::ZERO;
        let mut hit = None;
        let mut face = None;
        match after {
            Some(prey) => {
                zombie.goal = None;
                let off = prey.at.xz() - here;
                let toward = off.normalize_or_zero();
                face = Some(toward);
                if off.length() > REACH * PRESS_IN {
                    walk = toward * ZOMBIE_SPEED;
                }
                let in_reach = off.length() <= REACH + BODY && (prey.at.y - height).abs() <= HIT_OVER;
                if in_reach && zombie.cooldown <= 0.0 && zombie.body.free(nav) {
                    zombie.cooldown = BOT_PUNCH_EVERY;
                    hit = Some(prey.seat);
                }
            }
            None => {
                zombie.round = None;
                zombie.think -= dt;
                if zombie.think <= 0.0 {
                    zombie.think = self.dice.around(SHAMBLE_EVERY.0, SHAMBLE_EVERY.1);
                    let to = here + Vec2::from_angle(self.dice.roll() * std::f32::consts::TAU) * self.dice.between(2.0, SHAMBLE_FAR);
                    zombie.goal = nav.walkable_at(to).then_some(to);
                }
                if let Some(goal) = zombie.goal {
                    let off = goal - here;
                    if off.length() < 0.3 {
                        zombie.goal = None;
                    } else {
                        walk = off.normalize() * SHAMBLE.min(off.length() / dt.max(1e-4));
                    }
                }
            }
        }
        // A little apart from the others, whatever it is at.
        if crowded != Vec2::ZERO {
            walk = (walk + crowded.clamp_length_max(1.0) * ZOMBIE_SPEED * APART_PACE).clamp_length_max(ZOMBIE_SPEED);
        }
        // Round whatever stopped it, for a moment.
        let straight = walk;
        if let Some((side, left)) = zombie.round {
            let left = left - dt;
            zombie.round = (left > 0.0 && straight != Vec2::ZERO).then_some((side, left));
            if zombie.round.is_some() {
                walk = Vec2::from_angle(side * ROUND_BY).rotate(straight);
            }
        }
        let own = zombie.body.free(nav);
        zombie.body.step(nav, walk, dt);
        let moved = zombie.body.at.xz().distance(here);
        if walk == Vec2::ZERO {
            // Standing at someone, it faces them.
            if let Some(toward) = face.filter(|toward| *toward != Vec2::ZERO) {
                zombie.body.yaw = (-toward.x).atan2(-toward.y);
            }
        } else if own && moved < walk.length() * dt * STOPPED {
            // Stopped: the other way round from the last time, or either.
            let side = match zombie.round {
                Some((side, _)) => -side,
                None if self.dice.roll() < 0.5 => 1.0,
                None => -1.0,
            };
            if after.is_some() {
                zombie.round = Some((side, self.dice.around(ROUND_FOR.0, ROUND_FOR.1)));
            } else {
                zombie.goal = None;
            }
        }
        hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge_wars::{ZOMBIES, zombies_at};

    fn someone(seat: usize, at: Vec2) -> Prey {
        Prey {
            seat,
            at: Vec3::new(at.x, LAND_TOP, at.y),
        }
    }

    #[test]
    fn every_zombie_comes_up_on_ground_a_body_walks_and_no_two_in_one_place() {
        let nav = Nav::arena();
        let mut horde = Horde::new(7);
        let mut stood: Vec<Vec2> = Vec::new();
        for index in 0..ZOMBIES {
            assert_eq!(horde.rise(&nav), index);
            let body = *horde.body(index).expect("risen");
            let at = body.at.xz();
            assert!(nav.walkable_at(at), "zombie {index} at {at}");
            assert!(body.free(&nav), "zombie {index}: {body:?}");
            assert!(stood.iter().all(|other| other.distance(at) > 2.0), "zombie {index} at {at}");
            stood.push(at);
        }
        assert_eq!(horde.len(), ZOMBIES);
        // As many as the rules have by the end, and none before they come.
        assert_eq!(zombies_at(crate::bridge_wars::PLAY_SECS), ZOMBIES);
        assert_eq!(zombies_at(crate::bridge_wars::SUDDEN_DEATH - 0.1), 0);
    }

    #[test]
    fn a_zombie_goes_for_whoever_is_nearest_in_sight_at_its_own_pace_and_hits_them_in_reach() {
        let nav = Nav::arena();
        let mut horde = Horde::new(3);
        // The ninth rises on a team's island.
        for _ in 0..9 {
            horde.rise(&nav);
        }
        let index = 8;
        let from = horde.body(index).expect("risen").at.xz();
        let (island, _) = zombie_home(index);
        let there = island + (island - from).normalize() * 3.5;
        assert!(nav.walkable_at(there) && from.distance(there) > 5.0);
        let dt = 1.0 / 60.0;
        // Nobody in sight: no nearer anyone, and nobody hit.
        let far = [someone(0, from + Vec2::new(HUNT_RADIUS + 5.0, 0.0))];
        for _ in 0..30 {
            assert_eq!(horde.step(index, &nav, &far, false, dt), None);
        }
        // Someone in sight: at them, no faster than a zombie goes, and a
        // blow once they are in its reach, and the next no sooner than
        // the computer's players throw theirs.
        horde.stand(&nav, index);
        let prey = [someone(3, there), someone(5, there + (there - from).normalize() * 2.5)];
        let (mut hits, mut first, mut last) = (Vec::new(), None, from);
        for frame in 0..600 {
            let hit = horde.step(index, &nav, &prey, false, dt);
            let at = horde.body(index).expect("risen").at.xz();
            assert!(at.distance(last) <= ZOMBIE_SPEED * dt + 1e-3, "frame {frame}: {} m", at.distance(last));
            last = at;
            if let Some(seat) = hit {
                assert_eq!(seat, 3, "the nearer of the two");
                first.get_or_insert(frame);
                hits.push(frame as f32 * dt);
            }
        }
        let first = first.expect("it hit them") as f32 * dt;
        assert!(first > 0.3 && first < 2.5, "first hit after {first} s");
        assert!(last.distance(there) <= REACH + BODY && last.distance(there) > 0.5, "stood {} m off", last.distance(there));
        assert!(hits.windows(2).all(|pair| pair[1] - pair[0] >= BOT_PUNCH_EVERY - 1e-3), "{hits:?}");
        assert!(hits.len() >= 8, "{} hits in ten seconds", hits.len());
        // Flat under a bus it does nothing; and knocked, it is carried a
        // share of what anyone is, and its legs are its own after.
        let lay = horde.body(index).expect("risen").at;
        for _ in 0..60 {
            assert_eq!(horde.step(index, &nav, &prey, true, dt), None);
        }
        assert_eq!(horde.body(index).expect("risen").at, lay);
        horde.knock(index, there, true);
        let knocked = *horde.body(index).expect("risen");
        assert!((knocked.shove.length() - 20.0 * ZOMBIE_KNOCK).abs() < 1e-3 && knocked.lift > 0.0, "{knocked:?}");
    }

    #[test]
    fn a_zombie_never_walks_off_an_edge_after_someone_and_tries_round_what_stops_it() {
        let nav = Nav::arena();
        let mut horde = Horde::new(11);
        for _ in 0..9 {
            horde.rise(&nav);
        }
        let index = 8;
        let (island, _) = zombie_home(index);
        // Someone out over the void beside its island, in its sight: it
        // comes to the edge and no further, for as long as they are there.
        let out = island + island.normalize() * 16.0;
        assert!(nav.ground(out).is_none());
        let prey = [Prey {
            seat: 1,
            at: Vec3::new(out.x, LAND_TOP, out.y),
        }];
        let dt = 1.0 / 60.0;
        let mut sides = Vec::new();
        for _ in 0..1200 {
            assert_eq!(horde.step(index, &nav, &prey, false, dt), None);
            let body = *horde.body(index).expect("risen");
            assert!(nav.ground(body.at.xz()).is_some() && (body.at.y - LAND_TOP).abs() < 0.6, "{body:?}");
            sides.push((body.at.xz() - island).perp_dot(out - island).signum());
        }
        // It has been to both sides of the straight way at them: round
        // one way, and the other.
        assert!(sides.iter().any(|side| *side > 0.0) && sides.iter().any(|side| *side < 0.0));
    }
}
