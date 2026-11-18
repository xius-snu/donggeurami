//! Last Stand's zombies, one mind for the server's online and the device's
//! own offline (Hajun: "Zombies with increasing
//! difficulty/skill/creativity/health/attackdmg come in waves from the
//! outer edges toward the anchor crystal in the middle and/or player").
//! Each:
//!
//! * **climbs up over the rim** where its wave comes from, out of the sea
//!   (out of the void until the evening of 2026-11-18), for a second or so
//!   ([`Climb`]): hurt as it climbs, but holding on, not knocked off;
//! * **goes for the anchor or for someone**, which is thrown as it rises
//!   (most for the anchor at first, fewer as the waves go on); turns on
//!   whoever hits it, or comes within a thrown few metres of it (at the
//!   crystal, only now and then, by a chance of its own); gives them up
//!   again past a thrown distance, or when they are unseen; and some come
//!   round the side, on an arc of their own, rather than straight on;
//! * **walks there** over the stone and the sand, round the stands, the
//!   palms and the crystal (seeking, keeping a little apart from the others,
//!   sliding round what is in its way, trying the other way round when
//!   stopped), at its own pace, and **never off the rim by itself**, but
//!   after someone who is in the sea: otherwise only a knock takes it over,
//!   and then it wades back, slower, and climbs out again as it first did
//!   (the void had it until the evening of 2026-11-18); and never past the
//!   ring round the sea ([`SHORE`](super::SHORE));
//! * **hits** what it reaches, no oftener than a thrown second or so: a
//!   spitter keeps its distance and spits a slow ball ([`Spittle`]) that
//!   can be walked out of, and a leaper leaps at whoever is a few metres off;
//! * and, one of the last few of its wave, with none still to come, it
//!   **hurries**, a spitter coming in close: the last of a wave come on,
//!   and none hangs back for ever (mine).
//!
//! **Hajun's rule for anything the computer plays: nothing is a definite
//! if-then; every threshold is a spread, thrown again** ([`Dice::around`]).

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::ANCHOR;
use super::game::Knock;
use super::map::{CRYSTAL_RADIUS, OBSTACLES, PLINTH_TOP, RIM, SAND_TOP, SEA_BED, WALK_EDGE, floor, ground, in_the_sea, keep_out, within_the_shore};
use super::waves::Kind;
use crate::bots::Dice;
use crate::bridge_wars::{AIR_DRAG, GROUND_DRAG, KNOCK_BACK, KNOCK_UP, REACH, SPRINT_BONUS, anchor_in_reach};
use crate::nav::BODY;

/// What a body falls by, going up and coming down, in metres a second a
/// second, and the most it walks up or down at a step, in metres: a body's
/// own (the game's, and the computer's players' in Bridge Wars).
const RISE_GRAVITY: f32 = 22.0;
const FALL_GRAVITY: f32 = 33.0;
const STEP: f32 = 0.51;
/// How far under the sea's bed a zombie has fallen when it is dead, in
/// metres: none ever is now that there is the sea under the rim
/// ([`SEA_BED`]), which every one comes down in; kept for one that some
/// day would (it was how far under the sand one gone over the rim into the
/// void had fallen, until the evening of 2026-11-18).
pub(super) const FALL_DEAD: f32 = 12.0;
/// How fast a zombie wades, as a share of how fast it walks: as fast as a
/// body in the town's sea goes (the game's `WATER_MOVE_SCALE`).
const WADE_PACE: f32 = 0.72;
/// How near whoever it goes for it comes, as a share of its reach; how far
/// over or under it they may be and still be hit, against its size.
const PRESS_IN: f32 = 0.7;
const HIT_OVER: f32 = 1.9;
/// How near another a zombie keeps from, against their sizes, and how much
/// of its pace it gives to that.
const APART: f32 = 1.3;
const APART_PACE: f32 = 0.6;
/// How near something in its way it begins to turn along it, in metres
/// past its own width; how far to one side of its way it turns when stopped,
/// in radians; how little of a step it has to have made to have been
/// stopped, and for how long, in seconds.
const FEEL: f32 = 1.2;
const ROUND_BY: f32 = 1.2;
const STOPPED: f32 = 0.35;
const STOPPED_FOR: f32 = 0.4;
/// Where a zombie climbing over the rim begins, under the sea's bed (in the
/// water, its head just out of it) and out past the rim, and where it ends,
/// in from the rim, all in metres, for one of a body's size. (Under the
/// sand's top by 1.9, in the void, until the evening of 2026-11-18.)
const CLIMB_UNDER: f32 = 1.0;
const CLIMB_OUT: f32 = 0.35;
const CLIMB_IN: f32 = 1.05;

/// How far across the ground from where it comes down a spitter's ball
/// splashes whoever is there, in metres (mine): a step to either side is
/// out of it.
pub const SPIT_RADIUS: f32 = 1.3;

/// What a zombie goes for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Goal {
    Anchor,
    Seat(usize),
}

/// A zombie climbing up over the rim: from where to where, how long it has
/// been at it and how long it takes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Climb {
    from: Vec3,
    to: Vec3,
    age: f32,
    secs: f32,
}

impl Climb {
    /// Where it is: up the side of the sand for the first three quarters,
    /// and in over the rim in the second half.
    fn at(&self) -> Vec3 {
        let share = (self.age / self.secs).clamp(0.0, 1.0);
        let up = (share / 0.75).min(1.0);
        let up = 1.0 - (1.0 - up) * (1.0 - up);
        let over = ((share - 0.5) / 0.5).max(0.0);
        let across = self.from.xz().lerp(self.to.xz(), over * over * (3.0 - 2.0 * over));
        Vec3::new(across.x, self.from.y + (self.to.y - self.from.y) * up, across.y)
    }
}

/// A leaper in the air at someone: from where to where, how long it has
/// been in the air and how long it takes, and at whom.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Leap {
    from: Vec3,
    to: Vec3,
    age: f32,
    secs: f32,
    seat: usize,
}

impl Leap {
    /// How high it goes over the straight line, in metres.
    const APEX: f32 = 1.6;

    fn at(&self) -> Vec3 {
        let share = (self.age / self.secs).clamp(0.0, 1.0);
        self.from.lerp(self.to, share) + Vec3::Y * (4.0 * Self::APEX * share * (1.0 - share))
    }
}

/// A spitter's ball on its way: where it left the spitter's mouth, where it
/// comes down, and how long it has been going. Whoever runs the game flies
/// it and has it splash as it comes down ([`SPIT_RADIUS`]); every device
/// draws it by the same rules ([`Spit`](super::Spit)). Slow, and aimed at
/// where they were: "which can be walked out of".
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Spittle {
    pub from: Vec3,
    pub to: Vec3,
    pub age: f32,
}

impl Spittle {
    /// One spat from `from` to come down at `to`.
    pub fn spat(from: Vec3, to: Vec3) -> Self {
        Self { from, to, age: 0.0 }
    }

    /// How far it goes across the ground.
    pub fn chord(&self) -> f32 {
        self.from.xz().distance(self.to.xz())
    }

    /// How long it is in the air, in seconds: well over a second from
    /// across the length of a spitter's distance (mine).
    pub fn secs(&self) -> f32 {
        0.7 + 0.065 * self.chord()
    }

    /// How high it rises over the straight line half way, in metres.
    pub fn apex(&self) -> f32 {
        1.0 + 0.15 * self.chord()
    }

    /// Where it is.
    pub fn at(&self) -> Vec3 {
        let share = (self.age / self.secs()).clamp(0.0, 1.0);
        self.from.lerp(self.to, share) + Vec3::Y * (4.0 * self.apex() * share * (1.0 - share))
    }

    /// Moves it on by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        self.age += dt;
    }

    /// Whether it has come down, where it splashes.
    pub fn landed(&self) -> bool {
        self.age >= self.secs()
    }
}

/// What a zombie sees of someone: where they are, whether they are standing
/// (alive and in the game), and whether it may go for them (not unseen,
/// and not just back beside the anchor).
#[derive(Clone, Copy, Debug)]
pub(super) struct Seen {
    pub at: Vec3,
    pub standing: bool,
    pub open: bool,
}

/// What a zombie goes by in a frame: everyone, by seat; whether the anchor
/// stands; every zombie, where it is and how big, itself at `index`; and
/// how many of its wave are left standing with none still to come (none
/// while some are still to come).
pub(super) struct View<'a> {
    pub people: &'a [Seen],
    pub anchor: bool,
    pub crowd: &'a [(Vec3, f32)],
    pub index: usize,
    pub last: Option<usize>,
}

/// How much faster the last few of a wave come on, and how near a spitter
/// of them comes ([`Zombie::hurries`]).
const HURRY: f32 = 1.4;
const HURRY_SPIT: f32 = 3.0;

/// What a zombie did in a frame, for the game to carry out.
#[derive(Clone, Copy, Debug)]
pub(super) enum Act {
    /// A blow on whoever is in `seat`, knocking them so hard
    /// ([`Kind::knocks`]; a leaper landing, hard), away from `from`: the
    /// zombie's middle, or, for a leaper landing on them, where it leapt
    /// from, so that they are knocked on the way it came (where it comes
    /// down is where they stood as it leapt, no way from them at all).
    Hit { seat: usize, knock: Knock, from: Vec3 },
    /// A blow on the anchor.
    HitAnchor,
    /// A ball spat.
    Spat(Spittle),
    /// Fallen far enough to be dead ([`FALL_DEAD`]): none does, over the
    /// sea.
    Fell,
}

/// One zombie of a game of Last Stand.
#[derive(Clone, Debug)]
pub(super) struct Zombie {
    /// Its number in the game, never another's.
    pub id: u32,
    pub kind: Kind,
    pub wave: u16,
    pub at: Vec3,
    /// Its turn about the upright: -Z turned by this is ahead.
    pub yaw: f32,
    pub health: f32,
    pub most: f32,
    /// What its blow takes, before a shield, and its pace, in metres a second.
    pub damage: f32,
    pub pace: f32,
    pub climb: Option<Climb>,
    pub leap: Option<Leap>,
    /// What a knock is carrying it by, across the ground and up.
    pub shove: Vec2,
    pub lift: f32,
    pub goal: Goal,
    /// Whether it went for someone as it rose, rather than the anchor: what
    /// it goes back to once it gives someone up.
    pub hunter: bool,
    /// How near someone has to come for it to turn on them, and how far
    /// they have to get for it to give them up, in metres: its own.
    pub aggro: f32,
    pub gives_up: f32,
    /// Going for the anchor, how near the crystal it holds to it, in
    /// metres past its reach (two to four), and how likely it is there,
    /// each time it looks round, to turn on someone within its `aggro` all
    /// the same (none to six in a hundred): its own, thrown as it rises
    /// (mine: were every one to turn, whoever held the anchor would draw
    /// the whole heap off it onto themselves; were none to, standing by the
    /// crystal would be safe from all that came for it).
    pub holds: f32,
    pub strays: f32,
    /// The arc it comes on by, in radians off straight at them (none for
    /// most), and which way round it goes round things.
    pub swing: f32,
    pub side: f32,
    /// A spitter's distance; a leaper's band of distances it leaps across.
    pub keep_off: f32,
    pub leaps: (f32, f32),
    /// Until it may hit or spit again, and leap again, in seconds.
    pub cooldown: f32,
    pub leap_wait: f32,
    /// With how few of its wave left standing, and none to come, it
    /// hurries: its own, thrown as it rises, and whether it does now.
    pub stragglers: f32,
    pub hurries: bool,
    /// Until it looks round again for whom to go for.
    pub think: f32,
    /// How much longer a blow by each seat would not count, in seconds.
    pub hurt: Vec<f32>,
    /// Who last hit it, by seat, and when, in seconds into the fight.
    pub last_hit: Option<(usize, f32)>,
    pub stopped: f32,
    pub detour: Option<(f32, f32)>,
    /// What it is going round, by its place in [`OBSTACLES`], and which
    /// way: kept to until it is past it, so that it never turns back half
    /// way round.
    pub rounding: Option<(usize, f32)>,
    /// Dead, to be buried: by whom, and whether into the void.
    pub dead: bool,
    pub killer: Option<usize>,
    pub void: bool,
}

impl Zombie {
    /// A zombie of `kind` of wave `wave`, with these numbers, numbered `id`,
    /// beginning to climb over the rim at `angle` round it (radians from +X
    /// toward +Z), in a game of `seats` seats; going for the anchor unless
    /// `hunter`.
    #[allow(clippy::too_many_arguments)]
    pub fn rise(id: u32, kind: Kind, wave: u16, numbers: (f32, f32, f32), angle: f32, seats: usize, hunter: bool, dice: &mut Dice) -> Self {
        let (health, damage, pace) = numbers;
        let size = kind.size();
        let way = Vec2::from_angle(angle);
        let from = way * (RIM + CLIMB_OUT * size);
        let to = way * (RIM - CLIMB_IN - BODY * (size - 1.0));
        let climb = Climb {
            from: Vec3::new(from.x, SEA_BED - CLIMB_UNDER * size, from.y),
            to: Vec3::new(to.x, SAND_TOP, to.y),
            age: 0.0,
            secs: dice.around(1.0, 1.5) * size.sqrt(),
        };
        // Facing in, toward the middle.
        let yaw = way.x.atan2(way.y);
        // Some come round on an arc of their own, more of them as the
        // waves go on (mine).
        let flanks = dice.roll() < (0.15 + 0.02 * f32::from(wave)).min(0.5);
        let side = if dice.roll() < 0.5 { -1.0 } else { 1.0 };
        Self {
            id,
            kind,
            wave,
            at: climb.from,
            yaw,
            health,
            most: health,
            damage,
            pace,
            climb: Some(climb),
            leap: None,
            shove: Vec2::ZERO,
            lift: 0.0,
            goal: Goal::Anchor,
            hunter,
            aggro: dice.around(2.0, 4.5) * size.sqrt(),
            gives_up: dice.around(11.0, 22.0),
            holds: dice.around(2.0, 4.0),
            strays: dice.around(0.0, 0.06),
            swing: if flanks { side * dice.around(0.35, 0.9) } else { 0.0 },
            side,
            keep_off: dice.around(7.0, 9.0),
            leaps: (dice.around(3.5, 5.0), dice.around(7.5, 9.5)),
            cooldown: dice.around(0.3, 0.9),
            leap_wait: dice.around(0.5, 2.0),
            stragglers: dice.around(1.5, 5.0),
            hurries: false,
            think: 0.0,
            hurt: vec![0.0; seats],
            last_hit: None,
            stopped: 0.0,
            detour: None,
            rounding: None,
            dead: false,
            killer: None,
            void: false,
        }
    }

    /// How wide it is, from its middle, in metres.
    pub fn radius(&self) -> f32 {
        BODY * self.kind.size()
    }

    /// How far its blow reaches from its middle, in metres.
    pub fn reach(&self) -> f32 {
        REACH * self.kind.size()
    }

    /// Whether it is climbing over the rim.
    pub fn climbing(&self) -> bool {
        self.climb.is_some()
    }

    /// Where what it drops is thrown out from: its feet, or, while it is
    /// still climbing, where its climb comes up onto the sand, so that one
    /// killed on the side of the rim, its feet down in the water, drops what
    /// it would have on the sand above it, and not nothing; and one killed
    /// wading, on the sand nearest it, nothing lying in the sea.
    pub fn drops_from(&self) -> Vec3 {
        if let Some(climb) = self.climb {
            return climb.to;
        }
        if self.wading() {
            let way = self.at.xz().normalize_or(Vec2::X) * (RIM - 1.0);
            return Vec3::new(way.x, SAND_TOP, way.y);
        }
        self.at
    }

    /// Whether it is down in the sea, off the sand: its feet more than a
    /// step under the sand's top, out past the rim.
    pub fn wading(&self) -> bool {
        in_the_sea(self.at.xz()) && self.at.y < SAND_TOP - STEP
    }

    /// How fast it walks now: its pace, and [`HURRY`] times that once it is
    /// one of the last few of its wave (mine: the last of a wave come on,
    /// as in any game of waves, and none of them hangs back for ever, a
    /// spitter keeping its distance from people who keep out of its way);
    /// slower in the sea ([`WADE_PACE`]).
    fn pace_now(&self) -> f32 {
        self.pace * if self.hurries { HURRY } else { 1.0 } * if self.wading() { WADE_PACE } else { 1.0 }
    }

    /// Up out of the sea onto the sand from where it stands at the foot of
    /// the rim, as it first climbed out of it ([`Zombie::rise`]).
    fn climb_out(&mut self, dice: &mut Dice) {
        let size = self.kind.size();
        let way = self.at.xz().normalize_or(Vec2::X);
        let to = way * (RIM - CLIMB_IN - BODY * (size - 1.0));
        self.climb = Some(Climb {
            from: self.at,
            to: Vec3::new(to.x, SAND_TOP, to.y),
            age: 0.0,
            secs: dice.around(1.0, 1.5) * size.sqrt(),
        });
        self.shove = Vec2::ZERO;
        self.lift = 0.0;
        self.yaw = way.x.atan2(way.y);
    }

    /// How far a spitter keeps off whoever it spits at now: its own
    /// distance, or [`HURRY_SPIT`] once it hurries.
    fn keeps_off(&self) -> f32 {
        if self.hurries { self.keep_off.min(HURRY_SPIT) } else { self.keep_off }
    }

    /// Whether whoever has their feet at `at` is within its blow.
    fn reaches(&self, at: Vec3) -> bool {
        self.at.xz().distance(at.xz()) <= self.reach() + BODY && (at.y - self.at.y).abs() <= HIT_OVER * self.kind.size().max(1.0)
    }

    /// A blow or a blast from `from` knocks it, thrown on the run if
    /// `running`, by its kind's share of what it would knock anyone by: up,
    /// and away from there, on top of half of whatever already carried it.
    /// Nothing knocks one climbing over the rim, which holds on, nor a
    /// giant; a leaper is knocked out of its leap.
    pub fn knock(&mut self, from: Vec2, running: bool, share: f32) {
        let share = share * self.kind.knocked();
        if self.climb.is_some() || share <= 0.0 {
            return;
        }
        if let Some(leap) = self.leap.take() {
            // Out of its leap, with what the leap was carrying it by.
            self.at = leap.at();
        }
        let away = (self.at.xz() - from).normalize_or(Vec2::NEG_Y);
        let push = (KNOCK_BACK + if running { SPRINT_BONUS } else { 0.0 }) * share;
        let up = KNOCK_UP * share;
        self.shove = self.shove * 0.5 + away * push;
        self.lift = (self.lift * 0.5 + up).min(up);
    }

    /// Whoever is in `seat` has hit it: it goes for them now.
    pub fn turn_on(&mut self, seat: usize) {
        self.goal = Goal::Seat(seat);
        self.think = self.think.max(0.5);
    }

    /// A frame of it, `dt` seconds, with `view` of everyone: it is moved,
    /// and what it did, if anything, is given back.
    pub fn step(&mut self, view: &View, dice: &mut Dice, dt: f32) -> Option<Act> {
        self.cooldown = (self.cooldown - dt).max(0.0);
        self.leap_wait = (self.leap_wait - dt).max(0.0);
        for hurt in &mut self.hurt {
            *hurt = (*hurt - dt).max(0.0);
        }
        self.hurries = view.last.is_some_and(|left| left as f32 <= self.stragglers);
        if let Some(climb) = &mut self.climb {
            climb.age += dt;
            self.at = climb.at();
            if climb.age >= climb.secs {
                self.climb = None;
            }
            return None;
        }
        if let Some(leap) = &mut self.leap {
            leap.age += dt;
            self.at = leap.at();
            if leap.age < leap.secs {
                return None;
            }
            let (seat, from) = (leap.seat, leap.from);
            self.leap = None;
            self.cooldown = dice.around(0.8, 1.2);
            let seen = view.people.get(seat)?;
            return (seen.standing && self.reaches(seen.at)).then_some(Act::Hit { seat, knock: Knock::Heavy, from });
        }
        if self.carry(dt) {
            return (self.at.y < SEA_BED - FALL_DEAD).then_some(Act::Fell);
        }
        // Whom it goes for, looked at again every so often.
        self.think -= dt;
        if self.think <= 0.0 {
            self.think = dice.around(0.4, 1.0);
            self.choose(view, dice);
        }
        let here = self.at.xz();
        let Some((target, stop)) = self.aim(view) else {
            self.walk(view, Vec2::ZERO, here, dice, dt);
            return None;
        };
        let off = target - here;
        let far = off.length();
        let toward = if far > 1e-4 { off / far } else { Vec2::from_angle(self.yaw) };
        // Which way it means to walk: at them, or round on its arc; a
        // spitter only to its distance from them, and back from them if they
        // are nearer.
        let mut way = Vec2::ZERO;
        if self.kind == Kind::Spitter {
            if far > self.keeps_off() + 0.8 {
                way = toward;
            } else if far < self.keeps_off() - 1.5 {
                way = -toward;
            }
        } else if far > stop {
            way = toward;
        }
        if way != Vec2::ZERO && self.swing != 0.0 && way.dot(toward) > 0.0 {
            let bend = self.swing * ((far - 4.0) / 18.0).clamp(0.0, 1.0);
            way = Vec2::from_angle(bend).rotate(way);
        }
        self.walk(view, way, target, dice, dt);
        if way == Vec2::ZERO {
            // Standing at them, it faces them.
            self.yaw = (-toward.x).atan2(-toward.y);
        }
        self.strike(view, target, far, dice)
    }

    /// Who it goes for now: whoever it is after while they are still to be
    /// gone for and near enough; the anchor, or whoever is nearest of those
    /// to be gone for, as it rose, once it gives them up; or, going for the
    /// anchor, whoever comes within its reach of turning on them on its way
    /// there, but at the crystal (within its own `holds` of it) only by its
    /// own chance of straying, thrown at every look round, and otherwise
    /// only for a blow ([`Zombie::turn_on`]); and anyone, the anchor
    /// broken.
    fn choose(&mut self, view: &View, dice: &mut Dice) {
        let here = self.at.xz();
        let at_the_anchor = view.anchor && here.distance(ANCHOR) <= CRYSTAL_RADIUS + self.reach() + self.holds;
        let nearest = view
            .people
            .iter()
            .enumerate()
            .filter(|(_, seen)| seen.standing && seen.open)
            .map(|(seat, seen)| (seat, seen.at.xz().distance(here)))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let fallback = match nearest {
            Some((seat, _)) if self.hunter || !view.anchor => Goal::Seat(seat),
            _ => Goal::Anchor,
        };
        self.goal = match self.goal {
            Goal::Seat(seat) => {
                let keep = view
                    .people
                    .get(seat)
                    .is_some_and(|seen| seen.standing && seen.open && seen.at.xz().distance(here) <= self.gives_up);
                if keep { Goal::Seat(seat) } else { fallback }
            }
            Goal::Anchor => match nearest {
                Some((seat, _)) if !view.anchor || self.hunter => Goal::Seat(seat),
                Some((seat, far)) if far <= self.aggro && (!at_the_anchor || dice.roll() < self.strays) => Goal::Seat(seat),
                _ => Goal::Anchor,
            },
        };
    }

    /// Where it is making for, and how near it stops: the anchor's crystal,
    /// or whoever it goes for. None for someone it cannot see now, gone
    /// unseen or just back beside the anchor (it waits for its next look
    /// round, and goes for someone else, or the anchor).
    fn aim(&self, view: &View) -> Option<(Vec2, f32)> {
        match self.goal {
            Goal::Anchor if view.anchor => Some((ANCHOR, CRYSTAL_RADIUS + self.reach() * 0.55)),
            // The anchor broken and nobody to go for: about where it stood.
            Goal::Anchor => Some((ANCHOR, CRYSTAL_RADIUS + 3.0)),
            Goal::Seat(seat) => {
                let seen = view.people.get(seat)?;
                (seen.standing && seen.open).then(|| (seen.at.xz(), self.reach() * PRESS_IN))
            }
        }
    }

    /// Carries it by whatever knock is on it, and by its weight: up and
    /// down, landing on whatever ground it comes down onto, the sea's bed
    /// off the sand; across, dying away as it goes, and out of whatever it
    /// was carried into, and never past the ring round the sea. Whether it
    /// is in the air after.
    fn carry(&mut self, dt: f32) -> bool {
        let here = self.at.xz();
        let under = floor(here).unwrap_or(f32::NEG_INFINITY);
        let airborne = self.lift > 0.0 || self.at.y > under + 1e-3 || self.at.y < under - 1e-3;
        if airborne {
            let before = self.at.y;
            self.at.y += self.lift * dt;
            self.lift -= if self.lift > 0.0 { RISE_GRAVITY } else { FALL_GRAVITY } * dt;
            // Down onto ground it was over, or up a step onto it.
            if self.at.y <= under && before >= under - 0.3 {
                self.at.y = under;
                self.lift = 0.0;
            }
        } else if self.lift != 0.0 {
            // Come down to within a hair of the ground without quite
            // touching it: it is on it (else it stood there still falling,
            // as far as it knew, for the rest of the game: Bridge Wars'
            // bodies did, until 2026-11-13).
            self.at.y = under;
            self.lift = 0.0;
        }
        if self.shove != Vec2::ZERO {
            let to = self.kept_off_the_sand(within_the_shore(keep_out(here + self.shove * dt, self.radius())));
            self.at.x = to.x;
            self.at.z = to.y;
            let drag = if airborne { AIR_DRAG } else { GROUND_DRAG };
            self.shove *= drag.powf(dt * 20.0);
            if self.shove.length_squared() < 0.01 {
                self.shove = Vec2::ZERO;
            }
        }
        let under = floor(self.at.xz());
        self.lift != 0.0 || under.is_none_or(|under| (self.at.y - under).abs() > 1e-3)
    }

    /// `to`, where it is going across the ground: kept at the foot of the
    /// sand while it is down in the sea, as a wall keeps a body (it climbs
    /// up, [`Zombie::climb_out`]; nothing carries it in under the sand).
    fn kept_off_the_sand(&self, to: Vec2) -> Vec2 {
        let foot = RIM + self.radius();
        if self.at.y < SAND_TOP - STEP && to.length() < foot {
            to.normalize_or(self.at.xz().normalize_or(Vec2::X)) * foot
        } else {
            to
        }
    }

    /// Walks it `way` for `dt` seconds at its pace, toward `target`: round
    /// whatever is in its way, a little apart from the others, the other
    /// way round for a moment when stopped, never off the rim of itself,
    /// and onto the ground where it steps.
    fn walk(&mut self, view: &View, way: Vec2, target: Vec2, dice: &mut Dice, dt: f32) {
        let here = self.at.xz();
        let radius = self.radius();
        let mut way = way;
        if way != Vec2::ZERO {
            // Round whatever is in its way: along its edge, on the side
            // nearer the way it means to go, or its own side, head on. Not
            // the crystal, going for the anchor, nor anything beyond
            // whoever it goes for.
            let far = target.distance(here);
            let mut rounding = None;
            for (which, obstacle) in OBSTACLES.iter().enumerate() {
                if which == 0 && self.goal == Goal::Anchor {
                    continue;
                }
                let (out_by, out) = obstacle.outside(here);
                let clear = out_by - radius;
                if clear > FEEL || way.dot(out) >= 0.0 || far < out_by + 0.5 {
                    continue;
                }
                let along = out.perp();
                // The way round it already taken, or the one nearer the
                // way it means to go, or its own, head on.
                let side = match self.rounding {
                    Some((round, side)) if round == which => side,
                    _ => {
                        let lean = along.dot(way);
                        if lean.abs() > 0.2 { lean.signum() } else { self.side }
                    }
                };
                rounding = Some((which, side));
                let share = (1.0 - clear.max(0.0) / FEEL).clamp(0.0, 1.0);
                way = (way * (1.0 - share) + along * side * share).normalize_or(along * side);
            }
            self.rounding = rounding;
            if let Some((side, left)) = self.detour {
                let left = left - dt;
                self.detour = (left > 0.0).then_some((side, left));
                way = Vec2::from_angle(side * ROUND_BY).rotate(way);
            }
        }
        // A little apart from the others, whatever it is at.
        let mut apart = Vec2::ZERO;
        for (other, &(at, size)) in view.crowd.iter().enumerate() {
            if other == view.index || (at.y - self.at.y).abs() > 1.5 {
                continue;
            }
            let room = APART * (self.kind.size() + size) * 0.5;
            let off = here - at.xz();
            let gap = off.length();
            if gap < room {
                // Two on the very same spot part by their numbers.
                let away = if gap > 1e-3 { off / gap } else { Vec2::from_angle(view.index as f32 * 2.4 - other as f32) };
                apart += away * (1.0 - gap / room);
            }
        }
        let mut walk = way * self.pace_now();
        if apart != Vec2::ZERO {
            walk = (walk + apart.clamp_length_max(1.0) * self.pace_now() * APART_PACE).clamp_length_max(self.pace_now());
        }
        if walk == Vec2::ZERO {
            return;
        }
        let mut to = here + walk * dt;
        // Never off the rim of its own walking: no further out than the
        // edge it keeps to, or than it is already; but after someone who is
        // in the sea, or in it itself, as far as the ring round it.
        let wading = self.wading();
        let after_someone_in_the_sea = match self.goal {
            Goal::Seat(seat) => view.people.get(seat).is_some_and(|seen| seen.standing && in_the_sea(seen.at.xz())),
            Goal::Anchor => false,
        };
        let out = to.length();
        if !wading && !after_someone_in_the_sea && out > WALK_EDGE && out > here.length() {
            to *= WALK_EDGE.max(here.length()) / out;
        }
        let to = within_the_shore(keep_out(to, radius));
        // Down in the sea at the foot of the sand, making in: up it, as it
        // climbed out of the sea to begin with; or held at its foot.
        if wading && to.length() < RIM + radius {
            if way != Vec2::ZERO && way.dot(-here.normalize_or_zero()) > 0.2 {
                self.climb_out(dice);
                return;
            }
        }
        let to = self.kept_off_the_sand(to);
        self.at.x = to.x;
        self.at.z = to.y;
        if let Some(under) = floor(to)
            && (under - self.at.y).abs() <= STEP
        {
            self.at.y = under;
        }
        if way != Vec2::ZERO {
            self.yaw = (-way.x).atan2(-way.y);
            // Stopped, it tries round the other way for a moment.
            let meant = walk.length() * dt;
            if to.distance(here) < meant * STOPPED {
                self.stopped += dt;
                if self.stopped > STOPPED_FOR && self.detour.is_none() {
                    self.side = -self.side;
                    self.detour = Some((self.side, dice.around(0.5, 1.4)));
                    self.stopped = 0.0;
                }
            } else {
                self.stopped = (self.stopped - dt).max(0.0);
            }
        }
    }

    /// A blow, a ball or a leap at what it goes for, at `target`, `far` off,
    /// if it is ready to and near enough.
    fn strike(&mut self, view: &View, target: Vec2, far: f32, dice: &mut Dice) -> Option<Act> {
        // A leaper leaps when it is ready to leap, whatever its blows.
        if let Goal::Seat(seat) = self.goal
            && self.kind == Kind::Leaper
            && self.leap_wait <= 0.0
            && (self.leaps.0..=self.leaps.1).contains(&far)
            && view.people.get(seat).is_some_and(|seen| seen.standing && seen.open)
        {
            let to = target.clamp_length_max(WALK_EDGE);
            let to = Vec3::new(to.x, ground(to).unwrap_or(self.at.y), to.y);
            self.leap = Some(Leap {
                from: self.at,
                to,
                age: 0.0,
                secs: 0.45 + 0.035 * far,
                seat,
            });
            self.leap_wait = dice.around(3.0, 5.0);
            self.leaps = (dice.around(3.5, 5.0), dice.around(7.5, 9.5));
            return None;
        }
        if self.cooldown > 0.0 {
            return None;
        }
        let mouth = self.at + Vec3::Y * 1.4 * self.kind.size();
        match self.goal {
            Goal::Seat(seat) => {
                // Nobody it cannot see is hit, even one it was hitting a
                // moment ago who has just drunk the invisibility.
                let seen = *view.people.get(seat)?;
                if !seen.standing || !seen.open {
                    return None;
                }
                match self.kind {
                    Kind::Spitter if far <= self.keeps_off() + 2.5 => {
                        self.cooldown = dice.around(2.0, 3.0);
                        Some(Act::Spat(Spittle::spat(mouth, seen.at)))
                    }
                    Kind::Spitter => None,
                    _ if self.reaches(seen.at) => {
                        self.cooldown = self.wind_up(dice);
                        Some(Act::Hit {
                            seat,
                            knock: self.kind.knocks(),
                            from: self.at,
                        })
                    }
                    _ => None,
                }
            }
            Goal::Anchor if view.anchor => {
                if self.kind == Kind::Spitter {
                    if far > self.keeps_off() + 2.5 {
                        return None;
                    }
                    // At the crystal's side, chest high.
                    let side = ANCHOR + (self.at.xz() - ANCHOR).normalize_or(Vec2::Y) * CRYSTAL_RADIUS;
                    self.cooldown = dice.around(2.0, 3.0);
                    return Some(Act::Spat(Spittle::spat(mouth, Vec3::new(side.x, PLINTH_TOP + 1.2, side.y))));
                }
                anchor_in_reach(self.at.xz(), ANCHOR, self.reach()).then(|| {
                    self.cooldown = self.wind_up(dice);
                    Act::HitAnchor
                })
            }
            Goal::Anchor => None,
        }
    }

    /// How long until it may hit again, thrown each time: a second and a
    /// little more, longer for the big ones (mine: slower than Bridge
    /// Wars' bots, 0.8, as a crowd of them is the game).
    fn wind_up(&self, dice: &mut Dice) -> f32 {
        match self.kind {
            Kind::Brute => dice.around(1.4, 2.0),
            Kind::Giant => dice.around(2.0, 2.8),
            _ => dice.around(1.0, 1.5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::last_stand::map::{SAND_RADIUS, SHORE, STONE_TOP};
    use crate::last_stand::waves::strength;

    fn walker(dice: &mut Dice, angle: f32, hunter: bool) -> Zombie {
        Zombie::rise(0, Kind::Walker, 1, strength(Kind::Walker, 1), angle, 2, hunter, dice)
    }

    fn view<'a>(people: &'a [Seen], crowd: &'a [(Vec3, f32)], anchor: bool) -> View<'a> {
        View { people, anchor, crowd, index: 0, last: None }
    }

    #[test]
    fn a_zombie_climbs_over_the_rim_held_on_and_then_walks_to_the_anchor_and_hits_it() {
        let mut dice = Dice::new(1);
        let mut zombie = walker(&mut dice, 0.7, false);
        let start = zombie.at;
        assert!(start.xz().length() > SAND_RADIUS && start.y < SAND_TOP - 1.0, "{start}");
        // Knocked as it climbs, it holds on.
        zombie.knock(Vec2::ZERO, true, 1.0);
        assert_eq!((zombie.shove, zombie.lift), (Vec2::ZERO, 0.0));
        let nobody: [Seen; 0] = [];
        let dt = 1.0 / 30.0;
        let mut frames = 0;
        while zombie.climbing() {
            assert!(zombie.step(&view(&nobody, &[(zombie.at, 1.0)], true), &mut dice, dt).is_none());
            frames += 1;
            assert!(frames < 100, "still climbing");
        }
        assert!(zombie.at.xz().length() < SAND_RADIUS && (zombie.at.y - SAND_TOP).abs() < 1e-4, "{}", zombie.at);
        // Then to the anchor, never faster than its pace, and blows on it no
        // oftener than a second or so.
        let mut hits = Vec::new();
        let mut last = zombie.at.xz();
        for frame in 0..(30 * 30) {
            let crowd = [(zombie.at, 1.0)];
            if let Some(act) = zombie.step(&view(&nobody, &crowd, true), &mut dice, dt) {
                assert!(matches!(act, Act::HitAnchor), "{act:?}");
                hits.push(frame as f32 * dt);
            }
            assert!(zombie.at.xz().distance(last) <= zombie.pace * dt + 1e-3);
            last = zombie.at.xz();
        }
        assert!(hits.len() >= 10, "{} blows", hits.len());
        assert!(hits.windows(2).all(|pair| pair[1] - pair[0] >= 0.8 - 1e-3), "{hits:?}");
        assert!(anchor_in_reach(zombie.at.xz(), ANCHOR, zombie.reach()));
        // Outside the crystal, as every body is.
        assert!(OBSTACLES[0].outside(zombie.at.xz()).0 >= zombie.radius() - 1e-3);
    }

    #[test]
    fn a_zombie_turns_on_whoever_comes_near_and_gives_them_up_when_they_are_unseen() {
        let mut dice = Dice::new(2);
        let mut zombie = walker(&mut dice, 0.0, false);
        zombie.climb = None;
        zombie.at = Vec3::new(20.0, STONE_TOP, 0.0);
        let dt = 1.0 / 30.0;
        // Someone far off: it keeps on for the anchor.
        let far = [Seen { at: Vec3::new(20.0, STONE_TOP, 15.0), standing: true, open: true }];
        for _ in 0..30 {
            zombie.step(&view(&far, &[(zombie.at, 1.0)], true), &mut dice, dt);
        }
        assert_eq!(zombie.goal, Goal::Anchor);
        // Someone right by it: it turns on them, and hits them.
        let near = [Seen { at: zombie.at + Vec3::new(1.2, 0.0, 0.0), standing: true, open: true }];
        let mut hit = false;
        for _ in 0..90 {
            if let Some(Act::Hit { seat: 0, knock: Knock::None, from }) = zombie.step(&view(&near, &[(zombie.at, 1.0)], true), &mut dice, dt) {
                assert_eq!(from, zombie.at);
                hit = true;
            }
        }
        assert_eq!(zombie.goal, Goal::Seat(0));
        assert!(hit);
        // Unseen, they are given up, and it is for the anchor again.
        let unseen = [Seen { open: false, ..near[0] }];
        for _ in 0..60 {
            assert!(zombie.step(&view(&unseen, &[(zombie.at, 1.0)], true), &mut dice, dt).is_none());
        }
        assert_eq!(zombie.goal, Goal::Anchor);
    }

    #[test]
    fn a_zombie_walks_off_the_rim_only_after_someone_in_the_sea_and_one_knocked_in_climbs_back_out() {
        let mut dice = Dice::new(3);
        let mut zombie = walker(&mut dice, 1.0, true);
        zombie.climb = None;
        let edge = Vec2::from_angle(1.0) * (WALK_EDGE - 0.2);
        zombie.at = Vec3::new(edge.x, SAND_TOP, edge.y);
        let dt = 1.0 / 30.0;
        // Someone on the sand at the edge beyond it, round the rim: it
        // keeps to the edge, and never steps off of itself.
        let along = Vec2::from_angle(1.25) * (WALK_EDGE - 0.1);
        let by_the_edge = [Seen { at: Vec3::new(along.x, SAND_TOP, along.y), standing: true, open: true }];
        for _ in 0..120 {
            zombie.step(&view(&by_the_edge, &[(zombie.at, 1.0)], true), &mut dice, dt);
            assert!(zombie.at.xz().length() <= WALK_EDGE + 1e-3 && !zombie.wading(), "{}", zombie.at);
        }
        // Someone out in the sea: it goes in after them, down into the
        // water, slower there, and no further out than the ring round it.
        let out_there = Vec2::from_angle(1.0) * (SHORE + 3.0);
        let in_the_water = [Seen { at: Vec3::new(out_there.x, SEA_BED, out_there.y), standing: true, open: true }];
        let mut waded = false;
        for _ in 0..300 {
            assert!(zombie.step(&view(&in_the_water, &[(zombie.at, 1.0)], true), &mut dice, dt).is_none_or(|act| !matches!(act, Act::Fell)));
            assert!(zombie.at.xz().length() <= SHORE + 1e-3, "{}", zombie.at);
            waded |= zombie.wading() && (zombie.at.y - SEA_BED).abs() < 1e-3;
        }
        assert!(waded, "it went into the sea after them: {}", zombie.at);
        // A blow on the run, on the sand, from inside: over the rim, down in
        // the sea, and not dead; and then, going for the anchor, it wades
        // back to the sand and climbs out, as it first climbed out.
        let mut zombie = walker(&mut dice, 1.0, false);
        zombie.climb = None;
        zombie.at = Vec3::new(edge.x, SAND_TOP, edge.y);
        zombie.knock(zombie.at.xz() * 0.9, true, 1.0);
        let nobody: [Seen; 0] = [];
        let (mut was_in, mut climbed, mut back) = (false, false, false);
        for _ in 0..600 {
            let act = zombie.step(&view(&nobody, &[(zombie.at, 1.0)], true), &mut dice, dt);
            assert!(!matches!(act, Some(Act::Fell)), "the sea has it no more than a step down");
            was_in |= zombie.wading();
            climbed |= was_in && zombie.climbing();
            if climbed && !zombie.climbing() && zombie.at.xz().length() < RIM && (zombie.at.y - SAND_TOP).abs() < 1e-3 {
                back = true;
                break;
            }
        }
        assert!(was_in && climbed && back, "{} {was_in} {climbed} {back}", zombie.at);
        // What it drops, killed in the sea, lies on the sand in from it.
        let mut wet = walker(&mut dice, 0.5, false);
        wet.climb = None;
        let off = Vec2::from_angle(0.5) * (RIM + 2.0);
        wet.at = Vec3::new(off.x, SEA_BED, off.y);
        assert!(wet.wading());
        let drop = wet.drops_from();
        assert!(drop.xz().length() < RIM && ground(drop.xz()) == Some(SAND_TOP), "{drop}");
        // A giant is knocked by nothing.
        let mut giant = Zombie::rise(1, Kind::Giant, 5, strength(Kind::Giant, 5), 0.0, 2, false, &mut dice);
        giant.climb = None;
        giant.knock(Vec2::ZERO, true, 1.0);
        assert_eq!(giant.shove, Vec2::ZERO);
    }

    #[test]
    fn a_zombie_come_down_a_hair_over_the_ground_is_on_it_and_walks_on() {
        // Coming down from a knock, it stopped a hair over the stone, still
        // falling as far as it knew (a whole wave of a game of three waited
        // on one such for an hour, in the stand-ins' games).
        let mut dice = Dice::new(8);
        let mut zombie = walker(&mut dice, 0.0, false);
        zombie.climb = None;
        zombie.at = Vec3::new(-12.88, STONE_TOP + 0.0005, -0.37);
        zombie.lift = -1.0;
        let nobody: [Seen; 0] = [];
        let from = zombie.at.xz();
        for _ in 0..30 {
            zombie.step(&view(&nobody, &[(zombie.at, 1.0)], true), &mut dice, 1.0 / 30.0);
        }
        assert_eq!(zombie.lift, 0.0);
        assert!(zombie.at.xz().distance(from) > 1.0, "it has not walked: {}", zombie.at);
    }

    #[test]
    fn a_zombie_gets_round_a_stand_in_its_way() {
        // Coming at the anchor from straight behind the red stand.
        let mut dice = Dice::new(4);
        let mut zombie = walker(&mut dice, -std::f32::consts::FRAC_PI_2, false);
        zombie.climb = None;
        zombie.at = Vec3::new(0.85, STONE_TOP, -12.0);
        let nobody: [Seen; 0] = [];
        let dt = 1.0 / 30.0;
        let mut reached = None;
        for frame in 0..(30 * 20) {
            zombie.step(&view(&nobody, &[(zombie.at, 1.0)], true), &mut dice, dt);
            for obstacle in &OBSTACLES {
                assert!(obstacle.outside(zombie.at.xz()).0 >= zombie.radius() - 1e-3, "frame {frame}: in {obstacle:?} at {}", zombie.at);
            }
            if anchor_in_reach(zombie.at.xz(), ANCHOR, zombie.reach()) {
                reached = Some(frame);
                break;
            }
        }
        // About as long as walking round it takes: twelve metres and a
        // little round, at its pace.
        let reached = reached.expect("it got to the anchor") as f32 * dt;
        assert!(reached < 7.0, "{reached} s");
    }

    #[test]
    fn a_spitter_keeps_its_distance_and_its_ball_comes_down_where_they_were() {
        let mut dice = Dice::new(5);
        let mut spitter = Zombie::rise(0, Kind::Spitter, 8, strength(Kind::Spitter, 8), 0.0, 1, true, &mut dice);
        spitter.climb = None;
        spitter.at = Vec3::new(25.0, SAND_TOP, 0.0);
        let them = [Seen { at: Vec3::new(5.0, STONE_TOP, 0.0), standing: true, open: true }];
        let dt = 1.0 / 30.0;
        let mut balls = Vec::new();
        for _ in 0..(30 * 12) {
            if let Some(Act::Spat(ball)) = spitter.step(&view(&them, &[(spitter.at, 1.0)], true), &mut dice, dt) {
                balls.push(ball);
            }
        }
        let away = spitter.at.xz().distance(them[0].at.xz());
        assert!(away > spitter.keep_off - 1.6 && away < spitter.keep_off + 1.0, "{away} m off, keeping {}", spitter.keep_off);
        assert!(balls.len() >= 3, "{} balls", balls.len());
        let mut ball = balls[0];
        assert_eq!(ball.to, them[0].at);
        // Slow: over a second in the air from its distance, and high.
        assert!(ball.secs() > 1.1, "{}", ball.secs());
        let mut highest: f32 = 0.0;
        while !ball.landed() {
            ball.tick(dt);
            highest = highest.max(ball.at().y);
        }
        assert!(highest > ball.from.y + 1.0);
        assert!(ball.at().distance(them[0].at) < 1e-3);
    }

    #[test]
    fn a_leaper_leaps_at_whoever_is_a_few_metres_off_and_lands_a_heavy_blow() {
        let mut dice = Dice::new(6);
        let mut leaper = Zombie::rise(0, Kind::Leaper, 12, strength(Kind::Leaper, 12), 0.0, 1, true, &mut dice);
        leaper.climb = None;
        leaper.at = Vec3::new(16.0, STONE_TOP, 0.0);
        leaper.leap_wait = 0.0;
        let them = [Seen { at: Vec3::new(9.0, STONE_TOP, 0.0), standing: true, open: true }];
        let dt = 1.0 / 30.0;
        let (mut leapt, mut landed) = (None, None);
        for _ in 0..(30 * 4) {
            let act = leaper.step(&view(&them, &[(leaper.at, 1.0)], true), &mut dice, dt);
            if let Some(leap) = leaper.leap {
                leapt.get_or_insert(leap.from);
            }
            if let Some(Act::Hit { seat: 0, knock: Knock::Heavy, from }) = act {
                landed = Some(from);
                break;
            }
        }
        let (Some(leapt), Some(from)) = (leapt, landed) else {
            panic!("leapt {leapt:?}, landed {landed:?}");
        };
        // Down on them, and knocking them on the way it came: from where
        // it leapt, not from where it came down, which is where they stand.
        assert!(leaper.at.xz().distance(them[0].at.xz()) < 1.0);
        assert_eq!(from, leapt);
        let away = (them[0].at.xz() - from.xz()).normalize();
        assert!(away.dot(Vec2::NEG_X) > 0.95, "knocked {away}");
    }

    #[test]
    fn a_zombie_at_the_crystal_holds_to_it_but_now_and_then_one_strays_onto_whoever_stands_by() {
        // Many zombies, each its own throw, at the crystal, with someone
        // standing a step from each: some turn on them in a few seconds,
        // most keep at the crystal; and a blow turns every one.
        let mut dice = Dice::new(9);
        let dt = 1.0 / 30.0;
        let mut strayed = 0;
        for _ in 0..200 {
            let mut zombie = walker(&mut dice, 0.0, false);
            zombie.climb = None;
            let side = ANCHOR + Vec2::new(CRYSTAL_RADIUS + zombie.reach() * 0.55, 0.0);
            zombie.at = Vec3::new(side.x, ground(side).expect("by the crystal"), side.y);
            let by = [Seen { at: zombie.at + Vec3::new(0.0, 0.0, 1.4), standing: true, open: true }];
            for _ in 0..(30 * 3) {
                zombie.step(&view(&by, &[(zombie.at, 1.0)], true), &mut dice, dt);
            }
            strayed += usize::from(zombie.goal == Goal::Seat(0));
            zombie.turn_on(0);
            assert_eq!(zombie.goal, Goal::Seat(0));
        }
        assert!((10..=120).contains(&strayed), "{strayed} of 200 strayed");
    }
}
