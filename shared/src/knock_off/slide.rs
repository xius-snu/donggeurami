//! The slide in Knock Off: everyone across the ice at once, bouncing off each
//! other, until everything on it has stopped ([`Slide`]). The server plays it
//! out the moment everyone's aim is fixed and sends where it began and how
//! everyone set off; every device plays it again from there by the town's
//! clock, and comes out where the server did.
//!
//! **To the bit.** Who goes over the edge can turn on one bit of one
//! number: a slide of four bodies hitting each other is chaotic, a hair
//! either way at the first hit is a body in the sea or not by the third. So
//! a step of it goes by `+ - * /` and the square root alone, which IEEE has
//! every machine round the same way (`sin`, `cos`, `atan2`, `exp` and `powf`
//! do not have to, and the server's x86 and a phone's ARM can differ in
//! them); on plain `f32`s written out, not through a library's vectors,
//! whose sums could some day be fused; with a step that is always
//! [`STEP`](super::STEP) and the pairs of bodies taken in one order. A
//! direction comes in as a vector, never an angle: the launches the server
//! sends ([`super::KnockRound::launches`]) are the very bits it played.
//!
//! **Power is distance** (mine): a body launched alone slides exactly as far
//! as its arrow was long, whatever the steps (a step it stops within is cut
//! to what it had left), so that the arrow drawn while aiming never lies.

use bevy::math::{Vec2, Vec3};

use super::{BODY, BOUNCE, FALL, ICE, ON_THE_LINE, SLIDE_MOST, STEP, STOPPED, SUNK};

/// The most steps a slide is played for before it is over whatever is
/// still moving: [`SLIDE_MOST`] of them.
const MOST_STEPS: u32 = (SLIDE_MOST / STEP) as u32 + 1;

/// How many times a step goes over every pair of bodies that touch, at the
/// most: one pair pushed apart can push one of them into a third, which the
/// next time round is pushed apart in its turn. A pass that finds nobody
/// touching is the last.
const PASSES: usize = 6;

/// How fast two bodies have to come together for it to count as a hit,
/// in metres a second ([`Slide::hits`]): one that only leans on another as
/// both come to rest is no hit to raise dust for, and one that only brushes
/// someone already sliding into the sea on their own is not what put them
/// there. Only a hit is anyone's to be credited with ([`Slide::hit_by`],
/// "Mochi knocked Dubu in"): the two part a little either way however
/// slowly they meet, but a brush slower than this changes how fast either
/// goes by less than a third of a metre a second, which moves a body
/// standing still a centimetre (mine).
const FELT: f32 = 0.3;

/// Where a body is in the slide.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Place {
    /// Not on the ice at all: out before the slide began.
    Gone,
    /// On the ice.
    On,
    /// Over the edge, at this step.
    Over(u32),
}

/// One body, in plain numbers: where across the ice, how fast and which
/// way, and, over the edge, how far it has dropped and how fast it drops.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Disc {
    x: f32,
    z: f32,
    vx: f32,
    vz: f32,
    drop: f32,
    sinking: f32,
    place: Place,
    /// Whoever hit it last in this slide: by a hit that counts as one
    /// ([`FELT`]), not a brush.
    by: Option<usize>,
}

/// Two bodies meeting in a slide: when, in seconds into it; who (the lower
/// seat first); where, between their middles; and how fast they came
/// together, in metres a second. For a puff of ice and a shake of the
/// camera, on the device.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Hit {
    pub time: f32,
    pub seats: (usize, usize),
    pub at: Vec2,
    pub speed: f32,
}

/// How a slide came out ([`Slide::play_out`]), by seat.
#[derive(Clone, PartialEq, Debug)]
pub struct Ending {
    /// Where each body came to rest: none for one that went over, or that
    /// was out before the slide.
    pub at: Vec<Option<Vec2>>,
    /// When each that went over did, in seconds into the slide.
    pub fell: Vec<Option<f32>>,
    /// Whoever hit each last in the slide, if anyone did (a hit, not a
    /// brush: [`Slide::hits`]).
    pub by: Vec<Option<usize>>,
    /// How long it was until everything still on the ice had stopped, in
    /// seconds: a whole number of steps, [`SLIDE_MOST`] at the most.
    pub secs: f32,
}

/// One slide, played a step at a time. The same start, the same launches
/// and the same ice give the same slide to the bit, on every machine.
#[derive(Clone, Debug)]
pub struct Slide {
    discs: Vec<Disc>,
    radius: f32,
    /// How many steps it has gone, and at which of them it was over
    /// (stepped on after that, only whatever went over moves, dropping).
    steps: u32,
    done: bool,
    done_at: u32,
    hits: Vec<Hit>,
}

impl Slide {
    /// A slide from where everyone stands (`start`, by seat: none for
    /// whoever is out already), each setting off at their launch
    /// (`launches`, by seat, a velocity across the ice in metres a second;
    /// one missing, or anything not a number in it, is standing still),
    /// on ice that reaches `radius` from the middle. Anyone already past
    /// its edge goes over as it begins.
    pub fn new(start: &[Option<Vec2>], launches: &[Vec2], radius: f32) -> Self {
        let discs = start
            .iter()
            .enumerate()
            .map(|(seat, at)| {
                let launch = launches.get(seat).copied().filter(|launch| launch.x.is_finite() && launch.y.is_finite()).unwrap_or(Vec2::ZERO);
                match at.filter(|at| at.x.is_finite() && at.y.is_finite()) {
                    Some(at) => Disc { x: at.x, z: at.y, vx: launch.x, vz: launch.y, drop: 0.0, sinking: 0.0, place: Place::On, by: None },
                    None => Disc { x: 0.0, z: 0.0, vx: 0.0, vz: 0.0, drop: 0.0, sinking: 0.0, place: Place::Gone, by: None },
                }
            })
            .collect();
        let mut slide = Self {
            discs,
            radius,
            steps: 0,
            done: false,
            done_at: 0,
            hits: Vec::new(),
        };
        slide.edge();
        slide.settle();
        slide
    }

    /// A slide played out to its end: where everyone came to rest, who went
    /// over and when, and how long it took.
    pub fn play_out(start: &[Option<Vec2>], launches: &[Vec2], radius: f32) -> Ending {
        let mut slide = Self::new(start, launches, radius);
        while !slide.done {
            slide.step();
        }
        slide.ending()
    }

    /// One step, [`STEP`] long: the ice slows everything on it and it moves
    /// on, whatever has come together bounces apart, whatever has passed
    /// the edge goes over, and whatever went over before drops on toward
    /// the sea. Once it is over ([`Slide::done`]) only that last goes on.
    pub fn step(&mut self) {
        self.steps += 1;
        let playing = !self.done;
        for disc in &mut self.discs {
            match disc.place {
                Place::On if playing => slow(disc),
                Place::Over(_) => {
                    // Off the ice nothing slows it across the ground: it
                    // carries on as it left, and drops.
                    disc.x += disc.vx * STEP;
                    disc.z += disc.vz * STEP;
                    disc.sinking += FALL * STEP;
                    disc.drop = (disc.drop + disc.sinking * STEP).min(SUNK);
                }
                _ => {}
            }
        }
        if playing {
            self.bounce();
            self.edge();
            self.settle();
        }
    }

    /// Bodies that have come together bounce apart, pair by pair, the lower
    /// seat first, a few times over: each pair closing on each other
    /// trades speed along the line between their middles as two equal
    /// weights that keep [`BOUNCE`] of it do, and any that overlap are
    /// pushed apart, half each, until they only touch.
    fn bounce(&mut self) {
        let reach = BODY + BODY;
        let reach_squared = reach * reach;
        let time = self.time();
        for _ in 0..PASSES {
            let mut touching = false;
            for one in 0..self.discs.len() {
                if self.discs[one].place != Place::On {
                    continue;
                }
                for other in one + 1..self.discs.len() {
                    if self.discs[other].place != Place::On {
                        continue;
                    }
                    let (a, b) = (self.discs[one], self.discs[other]);
                    let (dx, dz) = (b.x - a.x, b.z - a.z);
                    let apart_squared = dx * dx + dz * dz;
                    if apart_squared >= reach_squared {
                        continue;
                    }
                    touching = true;
                    // The line between their middles, from the one to the
                    // other; for two on the very same spot, along x, the
                    // one to the west.
                    let (nx, nz, apart) = if apart_squared > 0.0 {
                        let apart = apart_squared.sqrt();
                        (dx / apart, dz / apart, apart)
                    } else {
                        (1.0, 0.0, 0.0)
                    };
                    let (mut a, mut b) = (a, b);
                    let closing = (a.vx - b.vx) * nx + (a.vz - b.vz) * nz;
                    if closing > 0.0 {
                        let impulse = (1.0 + BOUNCE) * 0.5 * closing;
                        a.vx -= impulse * nx;
                        a.vz -= impulse * nz;
                        b.vx += impulse * nx;
                        b.vz += impulse * nz;
                        // A hit, which each is the other's to be credited
                        // with; a brush is neither's.
                        if closing > FELT {
                            a.by = Some(other);
                            b.by = Some(one);
                            self.hits.push(Hit {
                                time,
                                seats: (one, other),
                                at: Vec2::new((a.x + b.x) * 0.5, (a.z + b.z) * 0.5),
                                speed: closing,
                            });
                        }
                    }
                    let push = (reach - apart) * 0.5;
                    a.x -= nx * push;
                    a.z -= nz * push;
                    b.x += nx * push;
                    b.z += nz * push;
                    self.discs[one] = a;
                    self.discs[other] = b;
                }
            }
            if !touching {
                break;
            }
        }
    }

    /// Whatever on the ice has its middle past the edge goes over, this
    /// step (by more than a hair, [`ON_THE_LINE`]); on ice that is all gone,
    /// everything does.
    fn edge(&mut self) {
        let edge_squared = (self.radius + ON_THE_LINE) * (self.radius + ON_THE_LINE);
        let steps = self.steps;
        for disc in &mut self.discs {
            if disc.place == Place::On && (self.radius <= 0.0 || disc.x * disc.x + disc.z * disc.z > edge_squared) {
                disc.place = Place::Over(steps);
            }
        }
    }

    /// Whether everything still on the ice has all but stopped, or the
    /// slide has gone on as long as one does: then it is over, and what is
    /// left of anyone's speed is gone.
    fn settle(&mut self) {
        let stopped_squared = STOPPED * STOPPED;
        let still = self
            .discs
            .iter()
            .all(|disc| disc.place != Place::On || disc.vx * disc.vx + disc.vz * disc.vz < stopped_squared);
        if still || self.steps >= MOST_STEPS {
            self.done = true;
            self.done_at = self.steps;
            for disc in &mut self.discs {
                if disc.place == Place::On {
                    disc.vx = 0.0;
                    disc.vz = 0.0;
                }
            }
        }
    }

    /// Steps it on until it has gone `secs` (whole steps; never back): how a
    /// device plays a slide by the town's clock, `secs` being how long ago
    /// it began ([`super::KnockRound::since`]). However its frames fall, it
    /// takes the same steps.
    pub fn step_to(&mut self, secs: f32) {
        let wanted = (secs.max(0.0) / STEP).floor().min(u32::MAX as f32) as u32;
        // No further than everything ever is: a slide's length, and as long
        // again for whatever went over to be in the sea.
        let wanted = wanted.min(2 * MOST_STEPS);
        while self.steps < wanted {
            self.step();
        }
    }

    /// How many steps it has gone.
    pub fn steps(&self) -> u32 {
        self.steps
    }

    /// How long it has been going, in seconds: so many steps.
    pub fn time(&self) -> f32 {
        self.steps as f32 * STEP
    }

    /// Whether it is over: everything on the ice stopped, or the slide as
    /// long as one goes.
    pub fn done(&self) -> bool {
        self.done
    }

    /// Where the body in `seat` is: across the ice, and how far below its
    /// top it has dropped, over the edge (0 on it). None for one that was
    /// out before the slide.
    pub fn at(&self, seat: usize) -> Option<Vec3> {
        self.discs
            .get(seat)
            .filter(|disc| disc.place != Place::Gone)
            .map(|disc| Vec3::new(disc.x, -disc.drop, disc.z))
    }

    /// How fast the body in `seat` is going across the ice, and which way.
    pub fn velocity(&self, seat: usize) -> Option<Vec2> {
        self.discs
            .get(seat)
            .filter(|disc| disc.place != Place::Gone)
            .map(|disc| Vec2::new(disc.vx, disc.vz))
    }

    /// When the body in `seat` went over the edge, in seconds into the
    /// slide, if it has.
    pub fn over_edge(&self, seat: usize) -> Option<f32> {
        match self.discs.get(seat)?.place {
            Place::Over(step) => Some(step as f32 * STEP),
            _ => None,
        }
    }

    /// Whoever last hit the body in `seat` in this slide, if anyone has: by
    /// a hit ([`Slide::hits`]), not a brush.
    pub fn hit_by(&self, seat: usize) -> Option<usize> {
        self.discs.get(seat)?.by
    }

    /// Every hit so far, in the order they came: two bodies that came
    /// together faster than a brush.
    pub fn hits(&self) -> &[Hit] {
        &self.hits
    }

    /// How it has come out so far: once it is over ([`Slide::done`]), how
    /// it came out, its time when it was over however long it has been
    /// stepped on since (as a device stepping it by the clock does, for
    /// whoever went over to be seen into the sea).
    pub fn ending(&self) -> Ending {
        let steps = if self.done { self.done_at } else { self.steps };
        Ending {
            at: self
                .discs
                .iter()
                .map(|disc| (disc.place == Place::On).then(|| Vec2::new(disc.x, disc.z)))
                .collect(),
            fell: (0..self.discs.len()).map(|seat| self.over_edge(seat)).collect(),
            by: self.discs.iter().map(|disc| disc.by).collect(),
            secs: steps as f32 * STEP,
        }
    }
}

/// The ice slows a body on it by [`ICE`] for one step, and it moves on: by
/// as far as it goes in the step slowing evenly, or, if it stops within the
/// step, by exactly as far as it had left to go.
fn slow(disc: &mut Disc) {
    let speed_squared = disc.vx * disc.vx + disc.vz * disc.vz;
    if speed_squared <= 0.0 {
        return;
    }
    let speed = speed_squared.sqrt();
    let slowing = ICE * STEP;
    if speed <= slowing {
        // What it had left: its speed squared over twice the slowing, along
        // the way it was going.
        let share = speed / (ICE + ICE);
        disc.x += disc.vx * share;
        disc.z += disc.vz * share;
        disc.vx = 0.0;
        disc.vz = 0.0;
    } else {
        let keep = (speed - slowing) / speed;
        let (vx, vz) = (disc.vx * keep, disc.vz * keep);
        disc.x += (disc.vx + vx) * (0.5 * STEP);
        disc.z += (disc.vz + vz) * (0.5 * STEP);
        disc.vx = vx;
        disc.vz = vz;
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3Swizzles;

    use super::*;
    use crate::bots::Dice;
    use crate::knock_off::{METRES_A_POWER, MOST_POWER, RADIUS, launch};

    /// Every number of a slide as it stands, as bits: two slides that give
    /// the same are the same to the bit.
    fn bits(slide: &Slide) -> Vec<u32> {
        slide
            .discs
            .iter()
            .flat_map(|disc| [disc.x, disc.z, disc.vx, disc.vz, disc.drop, disc.sinking])
            .map(f32::to_bits)
            .collect()
    }

    #[test]
    fn a_body_alone_slides_as_far_as_its_power_says() {
        for power in 1..=MOST_POWER {
            for way in [Vec2::X, Vec2::NEG_Y, Vec2::new(0.6, 0.8), Vec2::new(-0.28, 0.96)] {
                // From the far side, so that even the most stays on.
                let from = -way * 13.0;
                let ending = Slide::play_out(&[Some(from)], &[launch(way, power)], 40.0);
                let went = ending.at[0].expect("on the ice").distance(from);
                let meant = f32::from(power) * METRES_A_POWER;
                assert!((went - meant).abs() < 0.02 * meant, "power {power} went {went} m, not {meant}");
                // Along the way it was launched.
                let along = (ending.at[0].unwrap() - from).normalize();
                assert!(along.distance(way.normalize()) < 1e-4, "power {power}: went {along}");
                // And it took as long as slowing evenly from that speed takes,
                // all but the last crawl.
                let secs = launch(way, power).length() / ICE;
                assert!((ending.secs - secs).abs() < 3.0 * STEP, "power {power}: {} s, not {secs}", ending.secs);
            }
        }
        // Standing still, it stays, and that is over at once.
        let ending = Slide::play_out(&[Some(Vec2::new(3.0, -2.0))], &[Vec2::ZERO], RADIUS);
        assert_eq!(ending.at[0], Some(Vec2::new(3.0, -2.0)));
        assert_eq!(ending.secs, 0.0);
    }

    #[test]
    fn a_hit_keeps_the_speed_there_was_between_the_two() {
        let mut dice = Dice::new(7);
        let mut hits = 0;
        for _ in 0..200 {
            let one = Vec2::new(dice.between(-3.0, 3.0), dice.between(-3.0, 3.0));
            let other = one + Vec2::new(dice.bell(), dice.bell()).normalize_or(Vec2::X) * dice.between(1.3, 5.0);
            // The one at the other, give or take; the other going anywhere.
            let at = (other - one).normalize() + Vec2::new(dice.bell(), dice.bell()) * 0.15;
            let launches = [launch(at, 3 + (dice.roll() * 8.0) as u8), Vec2::new(dice.between(-3.0, 3.0), dice.between(-3.0, 3.0))];
            let mut slide = Slide::new(&[Some(one), Some(other)], &launches, 100.0);
            let both = |slide: &Slide| slide.velocity(0).unwrap() + slide.velocity(1).unwrap();
            while !slide.done() {
                // What the two would have between them after the ice had
                // slowed each this step, had they not met.
                let slowed = {
                    let mut alone = slide.clone();
                    alone.discs.iter_mut().for_each(slow);
                    Vec2::new(alone.discs[0].vx + alone.discs[1].vx, alone.discs[0].vz + alone.discs[1].vz)
                };
                let met = slide.hits().len();
                slide.step();
                if slide.hits().len() > met {
                    // The hit itself takes nothing from it, nor adds.
                    let after = both(&slide);
                    assert!(after.distance(slowed) < 1e-4 * (1.0 + slowed.length()), "{slowed} became {after}");
                    hits += 1;
                    break;
                }
            }
        }
        assert!(hits > 150, "only {hits} of 200 met");
    }

    #[test]
    fn a_body_hitting_one_standing_head_on_hands_it_nearly_all_its_speed() {
        // One sliding at another standing still, straight at it: just
        // before they meet, and just after.
        let mut slide = Slide::new(&[Some(Vec2::new(-5.0, 0.0)), Some(Vec2::ZERO)], &[launch(Vec2::X, 6), Vec2::ZERO], RADIUS);
        let mut coming = 0.0;
        while slide.hits().is_empty() {
            coming = slide.velocity(0).unwrap().x;
            slide.step();
        }
        let (after, hit) = (slide.velocity(1).unwrap().x, slide.velocity(0).unwrap().x);
        assert!(after >= 0.8 * coming, "it came at {coming} and the other left at {after}");
        assert!(hit.abs() < 0.1 * coming, "it went on at {hit}");
        assert_eq!(slide.hit_by(1), Some(0));
        assert_eq!(slide.hits()[0].seats, (0, 1));
    }

    #[test]
    fn a_brush_is_no_hit_and_nobody_s_to_be_credited_with() {
        // One sliding up to another standing still, from 2 m off, at a
        // speed that brings it there at `arriving`: 0.8 m to go before the
        // two touch.
        let meet = |arriving: f32| {
            let speed = (arriving * arriving + 2.0 * ICE * 0.8).sqrt();
            let mut slide = Slide::new(&[Some(Vec2::new(-2.0, 0.0)), Some(Vec2::ZERO)], &[Vec2::new(speed, 0.0), Vec2::ZERO], RADIUS);
            while !slide.done() {
                slide.step();
            }
            slide
        };
        // Barely moving as they touch: the other is nudged, but nobody hit
        // anyone.
        let brushed = meet(0.15);
        assert!(brushed.at(1).unwrap().x > 0.0, "it was not even nudged");
        assert!(brushed.hits().is_empty(), "{:?}", brushed.hits());
        assert_eq!((brushed.hit_by(0), brushed.hit_by(1)), (None, None));
        assert_eq!(brushed.ending().by, vec![None, None]);
        // Coming on at a run: a hit, each the other's.
        let hit = meet(1.5);
        assert_eq!(hit.hits().len(), 1);
        assert_eq!((hit.hit_by(0), hit.hit_by(1)), (Some(1), Some(0)));
    }

    #[test]
    fn a_slide_stepped_on_past_its_end_still_says_when_it_ended() {
        // One knocked in, and the slide stepped on long after, as a device
        // stepping it by the clock steps it while whoever went in drops.
        let start = [Some(Vec2::new(8.0, 0.0)), Some(Vec2::new(10.5, 0.0))];
        let launches = [launch(Vec2::X, 4), Vec2::ZERO];
        let ending = Slide::play_out(&start, &launches, RADIUS);
        let mut slide = Slide::new(&start, &launches, RADIUS);
        slide.step_to(SLIDE_MOST * 2.0);
        assert!(slide.time() > ending.secs + 1.0, "{} s, over at {} s", slide.time(), ending.secs);
        assert_eq!(slide.ending(), ending);
    }

    #[test]
    fn nothing_overlaps_after_a_step() {
        let mut dice = Dice::new(11);
        for _ in 0..150 {
            // Four apart, as a turn always begins, anywhere on the ice and
            // often close together, going anywhere.
            let mut start: Vec<Option<Vec2>> = Vec::new();
            while start.len() < 4 {
                let at = Vec2::new(dice.between(-5.0, 5.0), dice.between(-5.0, 5.0));
                if start.iter().flatten().all(|other| other.distance(at) >= 2.0 * BODY) {
                    start.push(Some(at));
                }
            }
            let launches: Vec<Vec2> = (0..4).map(|_| Vec2::new(dice.between(-14.0, 14.0), dice.between(-14.0, 14.0))).collect();
            let mut slide = Slide::new(&start, &launches, RADIUS);
            for _ in 0..MOST_STEPS {
                slide.step();
                for one in 0..4 {
                    for other in one + 1..4 {
                        if let (Some(a), Some(b)) = (slide.at(one), slide.at(other))
                            && slide.over_edge(one).is_none()
                            && slide.over_edge(other).is_none()
                        {
                            assert!(a.distance(b) > 2.0 * BODY - 1e-3, "{a} and {b} overlap after {} s", slide.time());
                        }
                    }
                }
                if slide.done() {
                    break;
                }
            }
        }
    }

    #[test]
    fn a_body_past_the_edge_goes_over_and_nothing_hits_it_after() {
        // One shoved off by another, and a third coming after them both
        // along the same line, which would have hit it on the ice.
        let start = [Some(Vec2::new(8.0, 0.0)), Some(Vec2::new(10.5, 0.0)), Some(Vec2::new(2.0, 0.0))];
        let launches = [launch(Vec2::X, 4), Vec2::ZERO, launch(Vec2::X, 10)];
        let mut slide = Slide::new(&start, &launches, RADIUS);
        while !slide.done() {
            slide.step();
        }
        let fell = slide.over_edge(1).expect("knocked over");
        assert!(fell > 0.0);
        assert_eq!(slide.hit_by(1), Some(0));
        // Once over, nobody touches it again, though the one that came
        // after went over the same way, and it has dropped.
        assert!(slide.over_edge(0).is_some() || slide.over_edge(2).is_some(), "nobody followed it: {:?}", slide.ending());
        assert!(
            slide.hits().iter().all(|hit| (hit.seats.0 != 1 && hit.seats.1 != 1) || hit.time <= fell),
            "{:?}",
            slide.hits()
        );
        let at = slide.at(1).unwrap();
        assert!(at.xz().length() > RADIUS && at.y < 0.0, "{at}");
        let ending = slide.ending();
        assert_eq!(ending.at[1], None);
        assert_eq!(ending.fell[1], Some(fell));
        // And past its end it goes on dropping, and nothing on the ice moves.
        let resting: Vec<Option<Vec3>> = (0..3).map(|seat| slide.at(seat).filter(|_| slide.over_edge(seat).is_none())).collect();
        assert!(resting.iter().any(Option::is_some), "everyone went over: {:?}", slide.ending());
        for _ in 0..60 {
            slide.step();
        }
        for (seat, at) in resting.iter().enumerate() {
            if at.is_some() {
                assert_eq!(slide.at(seat), *at, "seat {seat} moved");
            }
        }
        assert!(slide.at(1).unwrap().y <= at.y);
        assert!(slide.at(1).unwrap().y >= -SUNK);
    }

    #[test]
    fn the_same_start_is_the_same_slide_to_the_bit() {
        let mut dice = Dice::new(23);
        for _ in 0..40 {
            let start: Vec<Option<Vec2>> = (0..4)
                .map(|seat| (seat != 3 || dice.roll() < 0.7).then(|| Vec2::new(dice.between(-8.0, 8.0), dice.between(-8.0, 8.0))))
                .collect();
            let launches: Vec<Vec2> = (0..4).map(|_| Vec2::new(dice.between(-15.0, 15.0), dice.between(-15.0, 15.0))).collect();
            let radius = dice.between(3.0, 12.0);
            let (mut one, mut other) = (Slide::new(&start, &launches, radius), Slide::new(&start, &launches, radius));
            for _ in 0..MOST_STEPS + 30 {
                one.step();
                other.step();
                assert_eq!(bits(&one), bits(&other));
            }
            assert_eq!(Slide::play_out(&start, &launches, radius), Slide::play_out(&start, &launches, radius));
        }
        // And a slide written down to the bit, four sliding into a pile in
        // the middle: what it came to on the machine the game was made on,
        // which every machine has to come to, the server and the phones
        // alike. A slide that comes out otherwise is a slide that has been
        // changed: an app from before plays every slide otherwise than the
        // server, and `PROTOCOL_VERSION` goes up with it.
        let start = [Some(Vec2::new(4.95, 4.95)), Some(Vec2::new(4.95, -4.95)), Some(Vec2::new(-4.95, -4.95)), Some(Vec2::new(-4.95, 4.95))];
        let launches = [
            launch(Vec2::new(-1.0, -1.0), 5),
            launch(Vec2::new(-1.0, 1.0), 6),
            launch(Vec2::new(1.0, 1.1), 5),
            launch(Vec2::new(1.0, -0.8), 7),
        ];
        let mut slide = Slide::new(&start, &launches, RADIUS);
        while !slide.done() {
            slide.step();
        }
        let ending = slide.ending();
        let written: Vec<u32> = ending
            .at
            .iter()
            .map(|at| at.unwrap_or(Vec2::splat(f32::MAX)))
            .chain(ending.fell.iter().map(|fell| Vec2::splat(fell.unwrap_or(-1.0))))
            .flat_map(|at| [at.x.to_bits(), at.y.to_bits()])
            .chain([ending.secs.to_bits()])
            .collect();
        assert_eq!(slide.hits().len(), 3, "{:?}", slide.hits());
        assert_eq!(ending.by, [Some(3), Some(3), Some(3), Some(2)]);
        assert_eq!(written, GOLDEN_ENDING.to_vec(), "the slide came out otherwise: {written:#x?} {ending:?}");
    }

    /// How the slide of [`the_same_start_is_the_same_slide_to_the_bit`]
    /// came out on 2026-11-18, as bits: where each came to rest (`f32::MAX`
    /// twice for the one that went over), when each went over (-1 twice for
    /// those that did not), and how long it took. Everyone was hit, and the
    /// last of them knocked in.
    const GOLDEN_ENDING: [u32; 17] = [
        0x40a03ab9, 0xbed643fb, 0x3f6d0cbd, 0xc03280e1, 0xbd26b7bd, 0xbf79d342, 0x7f7fffff, 0x7f7fffff, 0xbf800000, 0xbf800000,
        0xbf800000, 0xbf800000, 0xbf800000, 0xbf800000, 0x3ff77778, 0x3ff77778, 0x3ff77778,
    ];

    #[test]
    fn a_thousand_slides_never_come_to_anything_that_is_not_a_number() {
        let mut dice = Dice::new(99);
        for _ in 0..1000 {
            let count = 2 + (dice.roll() * 3.0) as usize;
            let start: Vec<Option<Vec2>> = (0..count)
                .map(|_| (dice.roll() < 0.9).then(|| Vec2::new(dice.between(-3.0, 3.0), dice.between(-3.0, 3.0)) * dice.between(0.0, 4.0)))
                .collect();
            // Some on the very same spot, some still, some flat out.
            let mut start = start;
            if dice.roll() < 0.2 && count > 2 {
                start[1] = start[0];
            }
            let launches: Vec<Vec2> = (0..count)
                .map(|_| match (dice.roll() * 4.0) as u8 {
                    0 => Vec2::ZERO,
                    1 => launch(Vec2::new(dice.bell(), dice.bell()), MOST_POWER),
                    _ => Vec2::new(dice.between(-16.0, 16.0), dice.between(-16.0, 16.0)),
                })
                .collect();
            let radius = [RADIUS, 3.0, 0.75, 0.0][(dice.roll() * 4.0) as usize];
            let mut slide = Slide::new(&start, &launches, radius);
            for _ in 0..MOST_STEPS + 10 {
                slide.step();
            }
            assert!(slide.done());
            for seat in 0..count {
                if let Some(at) = slide.at(seat) {
                    assert!(at.is_finite() && slide.velocity(seat).unwrap().is_finite(), "{start:?} {launches:?}: {at}");
                }
            }
            assert!(slide.ending().secs <= SLIDE_MOST + STEP);
        }
    }

    #[test]
    fn whoever_stands_past_the_edge_as_it_begins_goes_over_and_all_go_on_no_ice() {
        let ending = Slide::play_out(&[Some(Vec2::new(5.0, 0.0)), Some(Vec2::new(1.0, 0.0)), None], &[], 3.0);
        assert_eq!(ending.at, vec![None, Some(Vec2::new(1.0, 0.0)), None]);
        assert_eq!(ending.fell, vec![Some(0.0), None, None]);
        let ending = Slide::play_out(&[Some(Vec2::ZERO), Some(Vec2::new(0.3, 0.0))], &[], 0.0);
        assert_eq!(ending.at, vec![None, None]);
    }
}
