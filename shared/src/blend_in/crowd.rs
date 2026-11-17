//! The crowd a game of Blend In's hider passes among (since 2026-11-18):
//! [`CROWD`](super::CROWD) of the town's people walking the middle of the
//! town, run by whoever runs the game (the server: Blend In is played
//! online only) so that every device sees the same crowd in the same
//! places, and drawn on every device exactly as a person is.
//!
//! **Their mind is the town AIs'** (the game's `ai`): every few seconds a
//! walk of 2 to 9 m ([`WALK_MIN`], [`WALK_MAX`]) in a random direction,
//! taken only if there is open ground all the way and a metre and a half
//! past its end ([`SHORE_MARGIN`]), never down or up more than a step
//! ([`STEP`]); walked straight at the walk's own speed ([`WALK_SPEED`]),
//! turning at once to face it ([`TURN_SPEED`]) and stopping on the spot;
//! then standing until the next. **A little less regular** (mine, so that
//! a person can pass for one: the town's think every 3 s exactly, which no
//! person keeps to), and each of them in its own way ([`Leanings`], mine,
//! after the check of 2026-11-18: "nothing is a definite if-then; every
//! threshold is a spread, thrown again", Hajun's rule for the computer's
//! players): each waits from one think to the next a while thrown between
//! its own shortest and longest, thrown as it is dressed, 2.4 s on average
//! and from 2 s for the busiest to 2.9 for the laziest ([`SHORTEST_WAIT`],
//! [`LONGEST_WAIT`]); one think in seven on average it stands where it is a
//! whole turn ([`STAND_A_TURN`]), and one in ten it only turns to face
//! somewhere ([`TURN_ONLY`]), each of them more or less often than that by
//! its leanings. So that the crowd walks about as often as the town's AIs
//! do, and is on the move about as much of the time (the game's
//! `blend_in_bake` measures both), but no two to one beat a seeker could
//! learn. Where the town's AIs ask the town's own shape whether the ground
//! is dry and how high it is, the crowd asks the town's middle baked
//! ([`grid`](super::grid)), the server having no map: dry, open and
//! with room for a body is a square a body can walk ([`Grid::clear`]),
//! which nothing past [`RING`](super::RING) is. Two differences come of
//! that, both mine: a walk never goes into a wall (the town's AIs press
//! against one until they next think), and never ends nearer where another
//! body stands or is going to than a body's width and a little, thrown
//! for each walk ([`APART`]), as the town's, which bump one another aside,
//! never stand in each other either.
//!
//! **Who they are** ([`Guise`](super::Guise)): each a boy or a girl by the
//! coin, one of that side's three looks, and a name of that side's, no two
//! alike and none a player's ([`thrown`](super::thrown)), thrown again
//! with fresh ids, and fresh leanings, every round as the hide begins
//! ([`Crowd::redress`]), so that nothing about them a seeker could have
//! noted in an earlier round (who wore what, how many wore it) says
//! anything of this one. All they do is thrown by sealed dice
//! ([`Sealed`]), so that nothing seen of them says what they will do next,
//! nor what they will be dressed as.
//!
//! **Struck** by a seeker's wrong guess, one staggers back a metre from
//! them as if punched, facing them ([`STAGGER`]), and stands a moment
//! ([`DAZED`]) before it thinks again.

use std::f32::consts::{PI, TAU};
use std::sync::Arc;

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::dice::Sealed;
use super::{CROWD_STREAM, Guise, fresh_ids, thrown};
use crate::nav::{CELL, Grid};

/// How fast one of them walks, in metres a second: the game's own walk
/// (`MOVE_SPEED`), the town AIs' and every person's.
pub const WALK_SPEED: f32 = 7.0;
/// How fast one turns to face its walk, in radians a second: the game's
/// `TURN_SPEED`, at which every body turns to face where it goes, a
/// person's too. Half a turn in a seventh of a second.
pub const TURN_SPEED: f32 = 22.0;
/// How far one walk goes, in metres: the town AIs'. Even the longest is
/// over (in 1.3 s) before the soonest next think ([`SHORTEST_WAIT`]), so
/// that one always stops between two walks.
pub const WALK_MIN: f32 = 2.0;
pub const WALK_MAX: f32 = 9.0;
/// How many random walks one looks at before it stands until it next
/// thinks: the town AIs' eight.
pub const TRIES: u32 = 8;
/// How much open ground a walk has to leave past where it ends, in metres:
/// the town AIs' margin from the water's edge, which keeps them, and the
/// crowd, off the very edge of anything.
pub const SHORE_MARGIN: f32 = 1.5;
/// How finely a walk is looked along, in metres: the town AIs'.
pub const PROBE: f32 = 0.25;
/// The most a body steps up or down without a jump, in metres: the game's
/// `LAND_STEP_UP`, half a metre and a hair under.
pub const STEP: f32 = 0.495;
/// How long one waits from one think to the next, in seconds (mine; the
/// town's AIs think every 3 s exactly): thrown afresh each time between its
/// own shortest and longest ([`Leanings::wait`]), which are thrown evenly
/// between these as it is dressed. The game was first designed with every
/// wait thrown from 2 to 4.5 s, and built with 1.5 to 3.5 once the first
/// was measured beside the town's own AIs: with a think in four standing
/// or only turning, it walked every 4.3 s on average to their 3.0, and was
/// on the move 18% of the time to their 27%, a crowd a third lazier than
/// the town it is drawn from, and so less cover for a hider walking among
/// it. These keep the second's average, 2.4 s, and give each its own, from
/// 2 s for the busiest to 2.9 for the laziest: the shortest never under
/// 1.4 s, a longest walk's time (1.3 s) and a moment's stand.
pub const SHORTEST_WAIT: (f32, f32) = (1.4, 1.9);
pub const LONGEST_WAIT: (f32, f32) = (2.5, 3.9);
/// How often a think is to stand where it is until the next (mine): one in
/// seven on average, each its own leaning, thrown evenly between these.
pub const STAND_A_TURN: (f32, f32) = (0.06, 0.22);
/// How often a think is only to turn where it stands, to face somewhere
/// (mine): one in ten on average, each its own leaning, as
/// [`STAND_A_TURN`].
pub const TURN_ONLY: (f32, f32) = (0.04, 0.16);
/// How far one struck by a seeker staggers back from them, in metres, and
/// over how long, in seconds: back a metre, as a punch would have it, the
/// last of it slower.
pub const STAGGER: f32 = 1.0;
pub const STAGGER_SECS: f32 = 0.35;
/// How long one stands once it has staggered, in seconds, before it thinks
/// again, thrown between these (mine).
pub const DAZED: (f32, f32) = (0.8, 1.6);
/// How near where another body stands, or is walking to, a walk may end, in
/// metres (mine): thrown evenly between these for each walk, never so near
/// that two bodies (0.9 m across) touch. Thrown, not the one number (1.2 m,
/// as it was first built), so that two standing nearer than that are no
/// sign that one of them is a person.
pub const APART: (f32, f32) = (1.0, 1.6);
/// How far apart they stand at the least as they are spread out at first,
/// in metres.
const SPREAD_APART: f32 = 2.0;
/// How many places are tried for each, spreading them out, before one is
/// taken however near the others it is.
const SPREAD_TRIES: usize = 100;

/// What one of the crowd is doing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Doing {
    /// Standing where it is.
    Standing,
    /// Walking straight to `to`, `heading` its way across the ground (a
    /// unit, x and z).
    Walking { to: Vec2, heading: Vec2 },
    /// Turning where it stands to face `yaw`.
    Turning { yaw: f32 },
    /// Struck: carried back from `from` to `to`, `share` of the way there
    /// so far, facing `facing`, whoever struck it.
    Staggering { from: Vec2, to: Vec2, share: f32, facing: f32 },
}

/// How one of the crowd goes about things, its own (mine): what each of
/// them is thrown as it is dressed, about the crowd's middle, so that no
/// two keep the one rhythm a seeker could learn, where the town's AIs all
/// think every 3 s.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Leanings {
    /// The shortest and the longest it waits from one think to the next,
    /// in seconds ([`SHORTEST_WAIT`], [`LONGEST_WAIT`]).
    pub wait: (f32, f32),
    /// How often a think is to stand where it is ([`STAND_A_TURN`]), and
    /// how often only to turn ([`TURN_ONLY`]).
    pub stand: f32,
    pub turn: f32,
}

impl Leanings {
    /// One's own, thrown.
    pub fn thrown(dice: &mut Sealed) -> Self {
        Self {
            wait: (
                dice.between(SHORTEST_WAIT.0, SHORTEST_WAIT.1),
                dice.between(LONGEST_WAIT.0, LONGEST_WAIT.1),
            ),
            stand: dice.between(STAND_A_TURN.0, STAND_A_TURN.1),
            turn: dice.between(TURN_ONLY.0, TURN_ONLY.1),
        }
    }
}

/// The mind of one of the town's people as the crowd has it: what its body
/// does next, think by think. Its own body's to drive, whosever: the
/// crowd's, or in a test a person standing in for the hider who walks as
/// they do.
#[derive(Clone, Debug)]
pub struct Mind {
    /// Seconds until it next thinks.
    next: f32,
    doing: Doing,
    leanings: Leanings,
}

impl Mind {
    /// A mind of its own leanings, thrown, that first thinks a thrown while
    /// from now, so that a crowd does not all set off at once.
    pub fn new(dice: &mut Sealed) -> Self {
        let leanings = Leanings::thrown(dice);
        Self {
            next: dice.between(0.0, leanings.wait.1),
            doing: Doing::Standing,
            leanings,
        }
    }

    pub fn doing(&self) -> Doing {
        self.doing
    }

    pub fn leanings(&self) -> Leanings {
        self.leanings
    }

    /// Moves the body standing at `at`, facing `yaw`, on by `dt` seconds:
    /// thinking when it is time, walking a walk under way, turning,
    /// staggering. `free` says whether a walk may end at a spot, kept as far
    /// as the distance it is given from the other bodies there. Whether it
    /// thought.
    pub fn step(
        &mut self,
        at: &mut Vec3,
        yaw: &mut f32,
        grid: &Grid,
        dice: &mut Sealed,
        dt: f32,
        free: &dyn Fn(Vec2, f32) -> bool,
    ) -> bool {
        if let Doing::Staggering { from, to, share, facing } = &mut self.doing {
            *share = (*share + dt / STAGGER_SECS).min(1.0);
            let eased = 1.0 - (1.0 - *share) * (1.0 - *share);
            stand(at, from.lerp(*to, eased), grid);
            turn(yaw, *facing, dt);
            if *share >= 1.0 {
                self.doing = Doing::Standing;
                self.next = dice.between(DAZED.0, DAZED.1);
            }
            return false;
        }
        self.next -= dt;
        let thought = self.next <= 0.0;
        if thought {
            let (shortest, longest) = self.leanings.wait;
            self.next = (self.next + dice.between(shortest, longest)).max(0.0);
            self.doing = think(*at, grid, dice, &self.leanings, free);
        }
        match self.doing {
            Doing::Walking { to, heading } => {
                let here = at.xz();
                let stride = WALK_SPEED * dt;
                let there = if here.distance(to) <= stride {
                    self.doing = Doing::Standing;
                    to
                } else {
                    here + heading * stride
                };
                stand(at, there, grid);
                turn(yaw, yaw_of(heading), dt);
            }
            Doing::Turning { yaw: wanted } => {
                if turn(yaw, wanted, dt) {
                    self.doing = Doing::Standing;
                }
            }
            Doing::Standing | Doing::Staggering { .. } => {}
        }
        thought
    }

    /// Struck by someone standing at `from`: whatever it was doing, it
    /// staggers back a metre from them, or half that, or not at all, as the
    /// ground behind it allows, turning to face them.
    pub fn stagger(&mut self, at: Vec3, from: Vec2, grid: &Grid, dice: &mut Sealed) {
        let here = at.xz();
        let away = (here - from).normalize_or_zero();
        let away = if away == Vec2::ZERO { Vec2::from_angle(dice.between(0.0, TAU)) } else { away };
        let back = [STAGGER, STAGGER * 0.5]
            .into_iter()
            .find(|&far| walks(grid, at, away, far))
            .unwrap_or(0.0);
        self.doing = Doing::Staggering {
            from: here,
            to: here + away * back,
            share: 0.0,
            facing: yaw_of(-away),
        };
    }

    /// Where its body will stand once what it is doing is done.
    fn end(&self, at: Vec3) -> Vec2 {
        match self.doing {
            Doing::Walking { to, .. } | Doing::Staggering { to, .. } => to,
            Doing::Standing | Doing::Turning { .. } => at.xz(),
        }
    }
}

/// What a think comes to: stand, turn, or walk, by the throw and the
/// leanings; a walk only if one of the few tried is open all the way and
/// ends clear of everyone else, by a distance thrown for it ([`APART`]),
/// else stand.
fn think(at: Vec3, grid: &Grid, dice: &mut Sealed, leanings: &Leanings, free: &dyn Fn(Vec2, f32) -> bool) -> Doing {
    let roll = dice.roll();
    if roll < leanings.stand {
        return Doing::Standing;
    }
    if roll < leanings.stand + leanings.turn {
        return Doing::Turning {
            yaw: dice.between(-PI, PI),
        };
    }
    let apart = dice.between(APART.0, APART.1);
    (0..TRIES)
        .find_map(|_| {
            let heading = Vec2::from_angle(dice.between(0.0, TAU));
            let distance = dice.between(WALK_MIN, WALK_MAX);
            let to = at.xz() + heading * distance;
            (walks(grid, at, heading, distance + SHORE_MARGIN) && free(to, apart)).then_some(Doing::Walking { to, heading })
        })
        .unwrap_or(Doing::Standing)
}

/// Whether a body at `from` can walk `distance` metres along `heading` (a
/// unit across the ground): every square of the way on ground a body walks
/// ([`Grid::clear`]), never down or up more than [`STEP`] from the one
/// before. The town AIs' `dry_all_the_way`, over the baked town, looked
/// along as finely as theirs ([`PROBE`]); and where the way crosses the
/// corner of a square between two looks, both squares beside the corner
/// have to be walked too, so that no walk ever clips one nobody walks (as
/// one by a lamp did, between two looks a quarter of a metre apart).
pub fn walks(grid: &Grid, from: Vec3, heading: Vec2, distance: f32) -> bool {
    let (width, _) = grid.size();
    let square = |at: Vec2| grid.index(at).map(|index| (index % width, index / width));
    // The floor of the square at `column`, `row`, if a body walks it and
    // can step onto it from a floor at `y`.
    let onto = |(column, row): (usize, usize), y: f32| -> Option<f32> {
        let at = grid.centre_of(column, row);
        grid.height(at).filter(|&ground| grid.clear(at) && (y - ground).abs() <= STEP)
    };
    let Some(mut was) = square(from.xz()) else {
        return false;
    };
    let mut y = grid.height(from.xz()).unwrap_or(from.y);
    let steps = (distance / PROBE).ceil() as u32;
    for step in 1..=steps {
        let spot = from.xz() + heading * (step as f32 * PROBE).min(distance);
        let Some(now) = square(spot) else {
            return false;
        };
        if now == was {
            continue;
        }
        if now.0 != was.0 && now.1 != was.1 && [(now.0, was.1), (was.0, now.1)].into_iter().any(|beside| onto(beside, y).is_none()) {
            return false;
        }
        let Some(ground) = onto(now, y) else {
            return false;
        };
        y = ground;
        was = now;
    }
    true
}

/// Stands `at` at `there` across the ground, on the ground there.
fn stand(at: &mut Vec3, there: Vec2, grid: &Grid) {
    at.x = there.x;
    at.z = there.y;
    if let Some(ground) = grid.height(there) {
        at.y = ground;
    }
}

/// Turns `yaw` toward `wanted` by as much as it turns in `dt`, the shorter
/// way round. Whether it is there.
fn turn(yaw: &mut f32, wanted: f32, dt: f32) -> bool {
    let off = (wanted - *yaw + PI).rem_euclid(TAU) - PI;
    let most = TURN_SPEED * dt;
    let there = off.abs() <= most;
    *yaw = if there { wanted } else { *yaw + most * off.signum() };
    *yaw = (*yaw + PI).rem_euclid(TAU) - PI;
    there
}

/// The turn about the upright that faces `heading` (x and z): a turn of
/// `yaw` points -Z at (-sin yaw, -cos yaw), as [`Spot`](crate::Spot) has
/// it.
pub fn yaw_of(heading: Vec2) -> f32 {
    (-heading.x).atan2(-heading.y)
}

/// One of the crowd: who it is, where it stands and faces, and its mind.
#[derive(Clone, Debug)]
pub struct Walker {
    pub guise: Guise,
    pub at: Vec3,
    pub yaw: f32,
    mind: Mind,
}

impl Walker {
    pub fn doing(&self) -> Doing {
        self.mind.doing()
    }

    /// How it goes about things this round.
    pub fn leanings(&self) -> Leanings {
        self.mind.leanings()
    }

    /// Where it will stand once what it is doing is done.
    fn end(&self) -> Vec2 {
        self.mind.end(self.at)
    }
}

/// The crowd: its people, the town's middle they walk, and the dice they
/// go by, sealed ([`Sealed`]), the same seed the same crowd.
pub struct Crowd {
    grid: Arc<Grid>,
    dice: Sealed,
    walkers: Vec<Walker>,
    /// The middle of every square a body can walk with room round it (the
    /// squares beside it walked too), to throw a place to stand from.
    open: Vec<Vec2>,
}

impl Crowd {
    /// [`CROWD`](super::CROWD) of the town's people, spread over the town's
    /// middle ([`grid`](super::grid)): what a table of Blend In has walking
    /// from the moment it opens, while people come to it, and hands its
    /// game ([`Game::new`](super::Game::new)). `seed` is a secret, as the
    /// game's is: it says everything they will do.
    pub fn new(seed: u64) -> Self {
        Self::on(super::grid(), super::CROWD, seed)
    }

    /// `count` of them on `grid`, spread over it anywhere a body walks,
    /// none nearer another than two metres where there is room, each
    /// facing anywhere and first thinking a thrown while from now.
    pub fn on(grid: Arc<Grid>, count: usize, seed: u64) -> Self {
        let mut dice = Sealed::new(seed, CROWD_STREAM);
        let open = open_ground(&grid);
        let mut walkers: Vec<Walker> = Vec::with_capacity(count);
        for _ in 0..count {
            let mut there = None;
            for _ in 0..SPREAD_TRIES {
                let Some(&spot) = open.get(dice.pick(open.len())) else {
                    break;
                };
                there = Some(spot);
                if walkers.iter().all(|walker| walker.at.xz().distance(spot) >= SPREAD_APART) {
                    break;
                }
            }
            let there = there.unwrap_or(Vec2::ZERO);
            let at = Vec3::new(there.x, grid.height(there).unwrap_or(0.0), there.y);
            walkers.push(Walker {
                guise: Guise::default(),
                at,
                yaw: dice.between(-PI, PI),
                mind: Mind::new(&mut dice),
            });
        }
        let mut crowd = Self { grid, dice, walkers, open };
        let ids = fresh_ids(&mut crowd.dice, count, &[]);
        crowd.redress(&[], &ids);
        crowd
    }

    /// The ground they walk.
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn walkers(&self) -> &[Walker] {
        &self.walkers
    }

    pub fn len(&self) -> usize {
        self.walkers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.walkers.is_empty()
    }

    /// The middle of every square a body walks with room round it.
    pub fn open(&self) -> &[Vec2] {
        &self.open
    }

    /// Whether a body could stand `at` with room round it: on a square a
    /// body walks, the squares beside it walked too.
    pub fn roomy(&self, at: Vec2) -> bool {
        roomy(&self.grid, at)
    }

    /// Moves every one of them on by `dt` seconds, each as its mind has it.
    /// `people` are where the players' bodies stand, which no walk ends on.
    pub fn step(&mut self, dt: f32, people: &[Vec2]) {
        let mut ends: Vec<Vec2> = self.walkers.iter().map(Walker::end).collect();
        for index in 0..self.walkers.len() {
            let free = |spot: Vec2, apart: f32| {
                ends.iter()
                    .enumerate()
                    .all(|(other, end)| other == index || end.distance(spot) >= apart)
                    && people.iter().all(|person| person.distance(spot) >= apart)
            };
            let walker = &mut self.walkers[index];
            walker.mind.step(&mut walker.at, &mut walker.yaw, &self.grid, &mut self.dice, dt, &free);
            ends[index] = walker.end();
        }
    }

    /// The one numbered `index` has been struck by someone standing at
    /// `from`: it staggers back from them.
    pub fn stagger(&mut self, index: usize, from: Vec2) {
        if let Some(walker) = self.walkers.get_mut(index) {
            walker.mind.stagger(walker.at, from, &self.grid, &mut self.dice);
        }
    }

    /// Dresses every one of them afresh, by the coin, as the hide begins:
    /// a side, a look of it and a name of it that none of the others has
    /// and that is not one of `taken` (the players' names, small letters),
    /// and each the id of theirs in `ids`; and each with leanings of its
    /// own afresh, a new person to anyone watching.
    pub(super) fn redress(&mut self, taken: &[String], ids: &[u64]) {
        let mut used: Vec<String> = taken.to_vec();
        for (walker, &id) in self.walkers.iter_mut().zip(ids) {
            let (name, look) = thrown(&mut self.dice, &used);
            used.push(name.to_lowercase());
            walker.guise = Guise { id, name, look };
            walker.mind.leanings = Leanings::thrown(&mut self.dice);
        }
    }
}

/// Whether a body could stand `at` on `grid` with room round it: on a
/// square a body walks, and the four squares beside it walked too.
fn roomy(grid: &Grid, at: Vec2) -> bool {
    [Vec2::ZERO, Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
        .into_iter()
        .all(|way| grid.clear(at + way * CELL))
}

/// The middle of every square of `grid` a body walks with room round it.
fn open_ground(grid: &Grid) -> Vec<Vec2> {
    let (width, height) = grid.size();
    (0..height)
        .flat_map(|row| (0..width).map(move |column| (column, row)))
        .map(|(column, row)| grid.centre_of(column, row))
        .filter(|&at| roomy(grid, at))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::{CROWD, RING, grid};
    use super::*;
    use crate::nav::Region;

    /// A flat disc of ground `radius` across, half a metre to a square.
    fn disc(radius: f32) -> Arc<Grid> {
        let side = (2.0 * radius / CELL).ceil() as usize + 2;
        let corner = Vec2::splat(-(side as f32) * CELL * 0.5);
        let mut grid = Grid::blank(side, side, corner);
        for row in 0..side {
            for column in 0..side {
                let at = grid.centre_of(column, row);
                if at.length() <= radius {
                    grid.set(column, row, Some(0.25), true, Some(Region::Middle));
                }
            }
        }
        Arc::new(grid)
    }

    #[test]
    fn a_walk_is_taken_only_over_open_ground_all_the_way() {
        let grid = disc(10.0);
        let middle = Vec3::new(0.0, 0.25, 0.0);
        assert!(walks(&grid, middle, Vec2::X, 9.0));
        // Off the edge, or ending too near it for the margin past its end.
        assert!(!walks(&grid, middle, Vec2::X, 11.0));
        assert!(!walks(&grid, middle, Vec2::X, 9.0 + SHORE_MARGIN + 0.5));
        // A wall across the way: a square nobody walks.
        let mut walled = (*grid).clone();
        let column = grid.index(Vec2::new(4.0, 0.0)).unwrap() % grid.size().0;
        for row in 0..grid.size().1 {
            walled.set(column, row, Some(1.2), false, None);
        }
        assert!(!walks(&walled, middle, Vec2::X, 6.0));
        assert!(walks(&walled, middle, Vec2::NEG_X, 6.0));
        // A step up or down of more than half a metre, though walked on
        // either side of it.
        let mut stepped = (*grid).clone();
        for row in 0..grid.size().1 {
            for column in 0..grid.size().0 {
                let at = grid.centre_of(column, row);
                if at.length() <= 10.0 && at.x > 3.0 {
                    stepped.set(column, row, Some(0.85), true, Some(Region::Middle));
                }
            }
        }
        assert!(!walks(&stepped, middle, Vec2::X, 6.0));
        assert!(walks(&stepped, Vec3::new(5.0, 0.85, 0.0), Vec2::X, 3.0));
        // A square nobody walks whose corner the way clips between two
        // looks along it: from (1.6, 0) slantwise, the looks fall at
        // (1.95, 0.35) and (2.13, 0.53), either side of the square from
        // (2, 0) to (2.5, 0.5), which the way crosses for 0.14 m.
        let slant = Vec2::new(1.0, 1.0).normalize();
        let from = Vec3::new(1.6, 0.25, 0.0);
        assert!(walks(&grid, from, slant, 3.0));
        let mut clipped = (*grid).clone();
        let index = grid.index(Vec2::new(2.25, 0.25)).unwrap();
        clipped.set(index % grid.size().0, index / grid.size().0, Some(0.25), false, None);
        for look in 1..=3 {
            assert_ne!(clipped.index(from.xz() + slant * PROBE * look as f32), Some(index), "look {look}");
        }
        assert!(!walks(&clipped, from, slant, 3.0));
    }

    #[test]
    fn a_turn_takes_the_shorter_way_round_and_stops_where_it_was_going() {
        let mut yaw = 3.0;
        // From just short of half a turn one way to just past it: through
        // the half turn, not back round the whole circle.
        assert!(!turn(&mut yaw, -3.0, 0.001));
        assert!(yaw > 3.0 || yaw < -3.0, "{yaw}");
        assert!(turn(&mut yaw, -3.0, 0.1));
        assert!((yaw + 3.0).abs() < 1e-6, "{yaw}");
        // A heading's yaw faces it, as a Spot turns.
        for heading in [Vec2::X, Vec2::NEG_Y, Vec2::new(0.6, 0.8)] {
            let yaw = yaw_of(heading);
            let ahead = Vec2::new(-yaw.sin(), -yaw.cos());
            assert!(ahead.distance(heading) < 1e-5, "{heading}: {ahead}");
        }
    }

    #[test]
    fn one_struck_staggers_back_from_whoever_struck_it_and_stands_a_moment() {
        let grid = disc(10.0);
        let mut dice = Sealed::new(3, 0);
        let mut mind = Mind::new(&mut dice);
        let (mut at, mut yaw) = (Vec3::new(2.0, 0.25, 0.0), 0.0);
        mind.stagger(at, Vec2::new(0.5, 0.0), &grid, &mut dice);
        let free = |_: Vec2, _: f32| true;
        let mut secs = 0.0;
        while matches!(mind.doing(), Doing::Staggering { .. }) {
            mind.step(&mut at, &mut yaw, &grid, &mut dice, 1.0 / 60.0, &free);
            secs += 1.0 / 60.0;
        }
        assert!((secs - STAGGER_SECS).abs() < 0.05, "{secs} s");
        assert!(at.distance(Vec3::new(3.0, 0.25, 0.0)) < 1e-4, "{at}");
        // Facing back the way it was struck from.
        assert!((Vec2::new(-yaw.sin(), -yaw.cos()) - Vec2::NEG_X).length() < 1e-4, "{yaw}");
        assert!(mind.next >= DAZED.0 && mind.next <= DAZED.1);
        // At the edge, it goes back only as far as there is ground.
        let at = Vec3::new(9.4, 0.25, 0.0);
        mind.stagger(at, Vec2::new(8.0, 0.0), &grid, &mut dice);
        let Doing::Staggering { to, .. } = mind.doing() else { panic!() };
        assert!(to.length() <= 10.0 && grid.clear(to), "{to}");
    }

    /// What a crowd did over a while, one by one.
    #[derive(Default, Debug)]
    struct Watched {
        /// Seconds walked, and stood, all of them together.
        walking: f32,
        standing: f32,
        /// Every wait between one think and the next, and every walk's
        /// length.
        waits: Vec<f32>,
        walks: Vec<f32>,
        /// Each one's waits, by its number.
        theirs: Vec<Vec<f32>>,
        /// Thinks that came to standing, to turning, and to walking.
        stood: u32,
        turned: u32,
        walked: u32,
    }

    /// Watches `crowd` for `secs`, `dt` at a time, after a while for it to
    /// settle into its stride, checking every step that each of them stands
    /// where a body walks and inside the ring.
    fn watch(crowd: &mut Crowd, secs: f32, dt: f32) -> Watched {
        let mut watched = Watched {
            theirs: vec![Vec::new(); crowd.len()],
            ..Default::default()
        };
        let mut since: Vec<Option<f32>> = vec![None; crowd.len()];
        let mut clock = 0.0;
        let settle = 15.0;
        while clock < settle + secs {
            let before: Vec<f32> = crowd.walkers.iter().map(|walker| walker.mind.next).collect();
            let doing: Vec<Doing> = crowd.walkers.iter().map(Walker::doing).collect();
            let stood: Vec<Vec2> = crowd.walkers.iter().map(|walker| walker.at.xz()).collect();
            let faced: Vec<f32> = crowd.walkers.iter().map(|walker| walker.yaw).collect();
            crowd.step(dt, &[]);
            clock += dt;
            for (index, walker) in crowd.walkers.iter().enumerate() {
                let at = walker.at.xz();
                assert!(crowd.grid.clear(at), "{index} stands at {at}, where nobody walks, {clock} s in");
                assert!(at.length() <= RING, "{index} is out at {at}");
                assert_eq!(Some(walker.at.y), crowd.grid.height(at), "{index} at {}", walker.at);
                if clock < settle {
                    continue;
                }
                // A think: the wait to the next one thrown on top of what
                // was left of the last.
                if walker.mind.next > before[index] {
                    assert!(!matches!(doing[index], Doing::Walking { .. }), "{index} still walking as it next thought");
                    if let Some(last) = since[index] {
                        watched.waits.push(clock - last);
                        watched.theirs[index].push(clock - last);
                    }
                    since[index] = Some(clock);
                    match walker.doing() {
                        // Measured from where it stood as it thought, the
                        // walk's first stride taken since.
                        Doing::Walking { to, .. } => {
                            watched.walked += 1;
                            watched.walks.push(to.distance(stood[index]));
                        }
                        // A turn as small as one frame's is over already.
                        Doing::Standing if walker.yaw != faced[index] => watched.turned += 1,
                        Doing::Turning { .. } => watched.turned += 1,
                        Doing::Standing => watched.stood += 1,
                        Doing::Staggering { .. } => unreachable!("a think is never a stagger"),
                    }
                }
                if matches!(walker.doing(), Doing::Walking { .. }) || matches!(doing[index], Doing::Walking { .. }) {
                    watched.walking += dt;
                } else {
                    watched.standing += dt;
                }
            }
        }
        watched
    }

    #[test]
    fn the_crowd_walks_the_middle_of_the_town_as_the_town_s_people_do_a_little_less_regularly() {
        let mut crowd = Crowd::new(7);
        assert_eq!(crowd.len(), CROWD);
        // Ten minutes, at a frame rate a server keeps.
        let watched = watch(&mut crowd, 600.0, 1.0 / 60.0);
        let thinks = (watched.stood + watched.turned + watched.walked) as f32;
        let share = |n: u32| n as f32 / thinks;
        let walking = watched.walking / (watched.walking + watched.standing);
        let mean = |all: &[f32]| all.iter().sum::<f32>() / all.len() as f32;
        let least = |all: &[f32]| all.iter().copied().fold(f32::INFINITY, f32::min);
        let most = |all: &[f32]| all.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        println!(
            "{} thinks in ten minutes: {:.0}% walked, {:.0}% turned, {:.0}% stood; walking {:.1}% of the time; \
             waits {:.2} to {:.2} s, {:.2} on average; walks {:.1} to {:.1} m, {:.1} on average",
            thinks,
            share(watched.walked) * 100.0,
            share(watched.turned) * 100.0,
            share(watched.stood) * 100.0,
            walking * 100.0,
            least(&watched.waits),
            most(&watched.waits),
            mean(&watched.waits),
            least(&watched.walks),
            most(&watched.walks),
            mean(&watched.walks),
        );
        // Every wait between two thinks within the one who waited's own
        // leanings, and the crowd's within all of theirs.
        let dt = 1.0 / 60.0;
        for (index, waits) in watched.theirs.iter().enumerate() {
            let (shortest, longest) = crowd.walkers[index].leanings().wait;
            assert!(waits.iter().all(|&wait| wait >= shortest - dt && wait <= longest + dt), "{index}: {shortest} to {longest}");
        }
        assert!(least(&watched.waits) >= SHORTEST_WAIT.0 - dt && most(&watched.waits) <= LONGEST_WAIT.1 + dt);
        // About the middle of what they lean to, all together: a wait of
        // 2.4 s, as the crowd was first built with.
        let middle = (SHORTEST_WAIT.0 + SHORTEST_WAIT.1 + LONGEST_WAIT.0 + LONGEST_WAIT.1) * 0.25;
        assert!((mean(&watched.waits) - middle).abs() < 0.12, "{}", mean(&watched.waits));
        // But each to a beat of its own: the busiest waits on average a
        // good while less than the laziest, and no two wait alike.
        let beats: Vec<f32> = watched.theirs.iter().map(|waits| mean(waits)).collect();
        println!("each one's wait on average, {:.2} to {:.2} s", least(&beats), most(&beats));
        assert!(most(&beats) - least(&beats) > 0.4, "{beats:?}");
        let mut sorted = beats.clone();
        sorted.sort_by(f32::total_cmp);
        assert!(sorted.windows(2).all(|pair| pair[0] != pair[1]), "{sorted:?}");
        // Every walk 2 to 9 m, and spread over that.
        assert!(least(&watched.walks) >= WALK_MIN - 0.01 && most(&watched.walks) <= WALK_MAX + 0.01);
        assert!(least(&watched.walks) < WALK_MIN + 0.3 && most(&watched.walks) > WALK_MAX - 0.3);
        // About one think in seven stands, one in ten turns, and nearly all
        // the rest walk: a walk is turned down only near an edge, or where
        // another stands.
        let half = |(low, high): (f32, f32)| (low + high) * 0.5;
        assert!((share(watched.turned) - half(TURN_ONLY)).abs() < 0.03, "{}", share(watched.turned));
        assert!((share(watched.stood) - half(STAND_A_TURN)).abs() < 0.06, "{}", share(watched.stood));
        // Each walks a good share of the time and stands the rest: the
        // shares as the numbers have them, a 5.5 m walk (0.79 s), three
        // thinks in four, a think every 2.4 s: on the move 25% of the time.
        assert!((0.2..0.31).contains(&walking), "walking {walking}");
        assert!(thinks > 600.0 / middle * CROWD as f32 * 0.9, "{thinks} thinks");
    }

    #[test]
    fn every_one_of_them_walks_and_stands_and_none_gets_stuck() {
        let mut crowd = Crowd::new(11);
        let start: Vec<Vec2> = crowd.walkers.iter().map(|walker| walker.at.xz()).collect();
        let mut walked = vec![0.0f32; crowd.len()];
        let mut stood = vec![0.0f32; crowd.len()];
        let dt = 1.0 / 30.0;
        for _ in 0..(300.0 / dt) as usize {
            crowd.step(dt, &[]);
            for (index, walker) in crowd.walkers.iter().enumerate() {
                if matches!(walker.doing(), Doing::Walking { .. }) {
                    walked[index] += dt;
                } else {
                    stood[index] += dt;
                }
            }
        }
        let shares: Vec<f32> = (0..crowd.len()).map(|index| walked[index] / (walked[index] + stood[index])).collect();
        for (index, &share) in shares.iter().enumerate() {
            // The laziest of them, by its leanings, walks a sixth of the
            // time; the busiest a third.
            assert!((0.1..0.42).contains(&share), "{index} walked {share} of the time");
            // And got somewhere in five minutes.
            assert!(crowd.walkers[index].at.xz().distance(start[index]) > 0.0, "{index} never moved");
        }
        println!(
            "each on the move {:.0}% to {:.0}% of the time",
            shares.iter().copied().fold(f32::INFINITY, f32::min) * 100.0,
            shares.iter().copied().fold(0.0, f32::max) * 100.0
        );
        // Wherever they went, no two stand in one another.
        for (index, walker) in crowd.walkers.iter().enumerate() {
            for other in &crowd.walkers[index + 1..] {
                if !matches!(walker.doing(), Doing::Walking { .. }) && !matches!(other.doing(), Doing::Walking { .. }) {
                    assert!(walker.at.xz().distance(other.at.xz()) >= APART.0 - 0.01);
                }
            }
        }
    }

    #[test]
    fn they_end_their_walks_apart_by_a_thrown_distance_never_touching() {
        // A crowd packed close on a small square, so that walks end near
        // others often: some nearer than the one number it used to be,
        // none nearer than two bodies' width and a little.
        let mut crowd = Crowd::on(disc(7.0), 14, 3);
        let (mut closest, mut nearer) = (f32::INFINITY, 0);
        let mut was: Vec<bool> = vec![false; crowd.len()];
        let dt = 1.0 / 30.0;
        for _ in 0..(240.0 / dt) as usize {
            crowd.step(dt, &[]);
            let standing: Vec<bool> = crowd.walkers.iter().map(|walker| !matches!(walker.doing(), Doing::Walking { .. })).collect();
            for index in 0..crowd.len() {
                // Each walk that has just ended, against everyone standing.
                if !standing[index] || was[index] {
                    continue;
                }
                for other in (0..crowd.len()).filter(|&other| other != index && standing[other]) {
                    let apart = crowd.walkers[index].at.xz().distance(crowd.walkers[other].at.xz());
                    closest = closest.min(apart);
                    nearer += usize::from(apart < 1.2);
                }
            }
            was = standing;
        }
        println!("the nearest a walk ended to another standing: {closest:.2} m; {nearer} walks ended nearer than 1.2 m");
        assert!(closest >= APART.0 - 0.01, "{closest}");
        assert!(nearer > 0);
    }

    #[test]
    fn they_are_spread_over_the_middle_and_no_two_go_by_one_name() {
        for seed in 0..20 {
            let crowd = Crowd::new(seed);
            let names: Vec<String> = crowd.walkers.iter().map(|walker| walker.guise.name.to_lowercase()).collect();
            let mut unique = names.clone();
            unique.sort();
            unique.dedup();
            assert_eq!(unique.len(), names.len(), "{names:?}");
            let mut ids: Vec<u64> = crowd.walkers.iter().map(|walker| walker.guise.id).collect();
            ids.sort();
            ids.dedup();
            assert_eq!(ids.len(), crowd.len());
            for walker in &crowd.walkers {
                // A boy's look and a boy's name, or a girl's and a girl's.
                let boy = crate::BOY_LOOKS.contains(&walker.guise.look.as_str());
                assert!(boy || crate::GIRL_LOOKS.contains(&walker.guise.look.as_str()), "{:?}", walker.guise);
                assert!(crate::ai_names(boy).contains(&walker.guise.name.as_str()), "{:?}", walker.guise);
                assert!(crowd.roomy(walker.at.xz()), "{}", walker.at);
            }
            for (index, walker) in crowd.walkers.iter().enumerate() {
                for other in &crowd.walkers[index + 1..] {
                    assert!(walker.at.xz().distance(other.at.xz()) >= SPREAD_APART);
                }
            }
        }
        // Spread over the whole of it: as many on each half as the other,
        // near enough, and some on the road round the outside, some on the
        // path by the pond.
        let mut out = [0usize; 2];
        let (mut road, mut path) = (0, 0);
        for seed in 0..40 {
            for walker in Crowd::new(seed).walkers {
                let at = walker.at.xz();
                out[usize::from(at.x > 0.0)] += 1;
                road += usize::from(at.length() > 60.0);
                path += usize::from(at.length() < 33.0);
            }
        }
        let all = (40 * CROWD) as f32;
        assert!((out[0] as f32 / all - 0.5).abs() < 0.05, "{out:?}");
        assert!(road as f32 / all > 0.2 && path as f32 / all > 0.05, "{road} on the road, {path} by the pond");
        // The same seed, the same crowd.
        let (a, b) = (Crowd::new(5), Crowd::new(5));
        for (a, b) in a.walkers.iter().zip(&b.walkers) {
            assert_eq!((&a.guise, a.at, a.yaw), (&b.guise, b.at, b.yaw));
        }
        assert!(grid().squares() > 0);
    }

    #[test]
    fn dressed_again_they_take_no_name_of_the_players_and_new_ids() {
        let mut crowd = Crowd::new(2);
        let taken: Vec<String> = ["Mochi", "Dubu", "Hodu", "Kong"].iter().map(|name| name.to_lowercase()).collect();
        let before: Vec<Guise> = crowd.walkers.iter().map(|walker| walker.guise.clone()).collect();
        let leaned: Vec<Leanings> = crowd.walkers.iter().map(Walker::leanings).collect();
        let mut dice = Sealed::new(9, 0);
        let ids = fresh_ids(&mut dice, crowd.len(), &[]);
        crowd.redress(&taken, &ids);
        for (walker, id) in crowd.walkers.iter().zip(&ids) {
            assert_eq!(walker.guise.id, *id);
            assert!(!taken.contains(&walker.guise.name.to_lowercase()), "{}", walker.guise.name);
        }
        // Most of them by other names, and in other looks, than before; and
        // every one of them of other leanings.
        let renamed = crowd.walkers.iter().zip(&before).filter(|(now, was)| now.guise.name != was.name).count();
        let changed = crowd.walkers.iter().zip(&before).filter(|(now, was)| now.guise.look != was.look).count();
        assert!(renamed > CROWD / 2 && changed > CROWD / 2, "{renamed} renamed, {changed} in another look");
        assert!(crowd.walkers.iter().zip(&leaned).all(|(now, was)| now.leanings() != *was));
        for walker in &crowd.walkers {
            let Leanings { wait, stand, turn } = walker.leanings();
            assert!((SHORTEST_WAIT.0..=SHORTEST_WAIT.1).contains(&wait.0) && (LONGEST_WAIT.0..=LONGEST_WAIT.1).contains(&wait.1));
            assert!((STAND_A_TURN.0..=STAND_A_TURN.1).contains(&stand) && (TURN_ONLY.0..=TURN_ONLY.1).contains(&turn));
            // The longest walk over before the soonest think after it.
            assert!(wait.0 > WALK_MAX / WALK_SPEED + 0.1);
        }
        // Even with every boy's name but a few taken, nobody goes by one
        // twice: the rest are girls.
        let all_but: Vec<String> = crate::BOY_NAMES[..20].iter().map(|name| name.to_lowercase()).collect();
        crowd.redress(&all_but, &ids);
        let mut names: Vec<&str> = crowd.walkers.iter().map(|walker| walker.guise.name.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), CROWD);
    }
}
