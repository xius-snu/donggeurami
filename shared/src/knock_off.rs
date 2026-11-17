//! Knock Off, on an island of ice (since 2026-11-18; Hajun: "You spawn on a
//! tiny icy circular island with 2~4 other ppl (fill with ai like bridgewars
//! and its deathmatch so 1v1v1v1) ... Everyone is frozen in place, and before
//! every certain time interval like 10 second, you choose the direction and
//! intensity that you wanna slide. Then after the interval, everyone goes
//! toward that direction and intensity and collide, some ppl will be knocked
//! off. You keep going until last player surviving is the winner ... Map =
//! iceisland"): four seats, every one for themselves, the computer taking the
//! seats people leave, everyone aiming at once and then sliding at once, on
//! `iceisland.glb` ([`Venue::KnockOff`](crate::Venue::KnockOff)). Its rules
//! are in this module, and the server and the device both play by them, as
//! by Bridge Wars' in [`crate::bridge_wars`]; how its table fills, starts and
//! pays is [`crate::table`]'s.
//!
//! A game is a [`Game`], stepped by whoever runs it: the server online, the
//! device offline, each carrying out what it comes to ([`Event`]) and sending
//! or drawing what everyone sees of it ([`KnockRound`]). Turn after turn:
//!
//! 1. **Aim** ([`FIRST_AIM`] the first turn, [`AIM`] after): nobody moves.
//!    Each person sets which way and how hard ([`Ask::Aim`]), seen by
//!    nobody else, and may say they are ready ([`Ask::Ready`]); once every
//!    person still in is, the aim is over. The computer's players weigh
//!    what to do meanwhile, each making up its mind at a moment of its own.
//! 2. **Reveal** ([`REVEAL`]): everyone's arrow is shown.
//! 3. **Slide**: everyone sets off at once and slides until everything on
//!    the ice has stopped, bouncing off each other; whoever's middle passes
//!    the edge goes into the sea and is out ([`Slide`]: the same on every
//!    machine to the bit, from the same start).
//! 4. **Break** ([`BREAK`], mine): the edge of the ice breaks off, a metre
//!    a turn ([`radius_in`]), and takes whoever stands on what breaks.
//!
//! until one is left, who wins ("You keep going until last player surviving
//! is the winner"), or nobody. Places go by when you went in, the last first:
//! by the turn, and in a turn by the moment, to the step of the slide, so
//! that of two who knock each other in, whoever stayed on longer comes
//! first (mine); those the ice broke under after it, together; whoever went
//! in at the very same moment sharing a place ([`table::split`] pays them).

mod computer;
mod slide;

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

pub use computer::{PERSONAS, Persona};
pub use slide::{Ending, Hit, Slide};

use crate::bots::Dice;
use crate::table;
use computer::{Habits, Mind};

/// How far the ice reaches from the middle of the island, in metres: Hajun's
/// `iceisland.glb` is a disc of ice 24 m across round the origin, its top at
/// 0 (a test in the game reads it off the map). A body whose middle is
/// further out than this has gone over.
pub const RADIUS: f32 = 12.0;

/// How far from the middle everyone stands as a game begins, in metres: on
/// the diagonals, facing the middle ([`arrival`]).
pub const START_RING: f32 = 7.0;

/// How far out from its middle a body reaches as it slides and hits, in
/// metres: everyone is a disc this wide either way (mine: a little wider
/// than a body, 0.45, so that a hit that looks as though it lands does).
pub const BODY: f32 = 0.6;

/// One step of a slide, in seconds: the server and every device step it by
/// this and nothing else, so that each comes to the same place ([`Slide`]).
pub const STEP: f32 = 1.0 / 120.0;

/// How fast the ice slows whatever slides on it, in metres a second each
/// second: everyone alike, evenly, until they stop.
pub const ICE: f32 = 5.0;

/// How far a slide goes on an empty island for each step of power, in
/// metres, and the most power there is: power is distance (mine), so that
/// the arrow drawn while aiming is as long as the slide, and 10 all but
/// crosses the island (22 m, on ice 24 across). Hajun: "right side is a up
/// down arrow for controlling intensity". It was 2.6, 10 crossing the whole
/// island and more, until whole games of the computer's players were over
/// in two or three turns: a slide that long put whoever it hit in the sea
/// from anywhere, and whoever it missed too.
pub const METRES_A_POWER: f32 = 2.2;
pub const MOST_POWER: u8 = 10;

/// How much of the speed two bodies meet at they part at, along the line
/// between their middles (mine): 0.9, so that one sliding straight into
/// another standing still hands it nearly all of its speed and all but
/// stops, as a curling stone does, and a hit is worth aiming. (0.8 and 0.7
/// made games no longer, nor two who meet head on any likelier to stay on.)
pub const BOUNCE: f32 = 0.9;

/// The longest a slide goes on, in seconds; and how slow everything left on
/// the ice has to be going for it to be over before that, in metres a
/// second.
pub const SLIDE_MOST: f32 = 6.0;
pub const STOPPED: f32 = 0.05;

/// How fast whoever goes over the edge drops toward the sea, in metres a
/// second each second (the game's `FALL_GRAVITY`), and how far below the
/// ice's top they sink at the most, in metres: the ice's depth.
pub const FALL: f32 = 33.0;
pub const SUNK: f32 = 3.0;

/// How long everyone has to aim, in seconds: 12 the first turn, the game
/// being new (mine), and 10 after (Hajun: "before every certain time
/// interval like 10 second, you choose the direction and intensity that you
/// wanna slide").
pub const FIRST_AIM: f32 = 12.0;
pub const AIM: f32 = 10.0;

/// How long everyone's arrow is shown before everyone slides, in seconds.
pub const REVEAL: f32 = 0.8;

/// How long a slide is held once everything on the ice has stopped, in
/// seconds, so that whoever went over last is seen into the sea.
pub const SETTLE: f32 = 0.6;

/// How long the ice takes to break at the end of a turn, in seconds (mine).
pub const BREAK: f32 = 1.2;

/// The last turn there is: at its end the ice is gone, and whoever is on it
/// goes in with it ([`radius_in`]).
pub const LAST_TURN: u16 = 13;

/// How far past the edge of the ice someone's middle may be and they still
/// stand, in metres, in a slide and as the ice breaks: a millimetre, so that
/// whoever stands right on the line (as everyone who never moves stands on
/// the one it breaks along after the fifth turn, 7 m out) is not put in the
/// sea by how one sum was rounded.
const ON_THE_LINE: f32 = 0.001;

/// How many of the computer's moves its players play out, all of them
/// together, in each step of a game, while everyone aims: what a step of
/// theirs costs is about this many slides, on the device offline (a slide
/// played out is a few microseconds on this PC, and a whole turn's thinking
/// half a millisecond). Whatever one has not weighed by the time it makes up
/// its mind, it weighs then.
const THINK_A_STEP: usize = 24;

/// How far the ice reaches during `turn` (counted from 1), in metres: 12 the
/// first turn, a metre less each turn after to 3 in the tenth, then three
/// quarters of a metre less each turn to none (mine: the island's edge
/// cracks and breaks off at the end of every turn, as the shrinking ice of
/// the penguin games does, so that every game ends, and ends with everyone
/// close together). Before the first turn, the whole island.
pub fn radius_in(turn: u16) -> f32 {
    match turn {
        0..=1 => RADIUS,
        2..=10 => RADIUS + 1.0 - f32::from(turn),
        11 => 2.25,
        12 => 1.5,
        13 => 0.75,
        _ => 0.0,
    }
}

/// How far the ice reaches once `turn` is over and the edge has broken off:
/// what the next turn is played on.
pub fn radius_after(turn: u16) -> f32 {
    radius_in(turn.saturating_add(1))
}

/// How fast a slide of `power` sets off, in metres a second: as fast as the
/// ice takes `power` times [`METRES_A_POWER`] to stop.
pub fn speed(power: u8) -> f32 {
    (2.0 * ICE * METRES_A_POWER * f32::from(power.min(MOST_POWER))).sqrt()
}

/// How far a slide of `power` goes on an empty island, in metres: what the
/// arrow drawn while aiming is as long as.
pub fn reach(power: u8) -> f32 {
    METRES_A_POWER * f32::from(power.min(MOST_POWER))
}

/// How someone aiming `way` (across the ground, x and z; any length, and
/// nothing, or anything not a number, is staying put) with `power` sets off:
/// their velocity across the ice, in metres a second. By `+ - * /` and the
/// square root alone, as the slide goes ([`Slide`]).
pub fn launch(way: Vec2, power: u8) -> Vec2 {
    let length_squared = way.x * way.x + way.y * way.y;
    if power == 0 || !length_squared.is_finite() || length_squared < 1e-12 {
        return Vec2::ZERO;
    }
    let length = length_squared.sqrt();
    let speed = speed(power);
    Vec2::new(way.x / length * speed, way.y / length * speed)
}

/// How a game of Knock Off fills, starts and pays: four seats, the computer
/// taking those people leave 10 s after the last person joined, as in
/// Bridge Wars ("fill with ai like bridgewars"); 50 coins to join.
pub const SHAPE: table::Shape = table::Shape {
    most: 4,
    fewest: 1,
    computer: true,
    wait: 10.0,
    countdown: 5.0,
    results: 15.0,
    fee: 50,
};

/// Something asked in a game of Knock Off ([`Ask::KnockOff`](crate::Ask::KnockOff)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ask {
    /// My aim this turn as it stands: which way across the ice (`x`, `z`,
    /// the game's axes, any length; nothing is staying put) and how hard,
    /// 0 to [`MOST_POWER`]. Sent as it changes while everyone aims; the last
    /// one heard before the aim is over is the slide ([`Game::aim`]).
    Aim { x: f32, z: f32, power: u8 },
    /// I am ready: my aim is as it stands, and once every person still in
    /// is ready the aim is over for everyone ([`Game::ready`]).
    Ready,
}

/// Something told one device in a game of Knock Off
/// ([`Tell::KnockOff`](crate::Tell::KnockOff)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// Your aim as the game holds it ([`Game::aim_of`]): sent to a device
    /// that comes back to the game while everyone aims, so that its arrow
    /// is drawn where it was left. Nobody's aim is sent to anyone else
    /// before the reveal.
    Aimed { x: f32, z: f32, power: u8 },
}

/// The base's test of what is sent (`what_was_sent_before_the_four_games_is_sent_as_it_was`,
/// in the crate's root) was written while each game's asks and tells were a placeholder
/// called `Nothing`, which these are not: until that test is written for them, it builds
/// with one of each by that name. Nothing outside the tests sees it.
#[cfg(test)]
#[allow(non_upper_case_globals)]
impl Ask {
    pub const Nothing: Ask = Ask::Ready;
}

#[cfg(test)]
#[allow(non_upper_case_globals)]
impl Tell {
    pub const Nothing: Tell = Tell::Aimed { x: 0.0, z: -1.0, power: 5 };
}

/// Registers this game's own replicated components, after the board
/// ([`table::Board`]): what everyone in a game sees of it ([`KnockRound`]).
pub fn register(app: &mut App) {
    app.replicate::<KnockRound>();
}

/// Where the player in `seat` stands as they arrive on the game's venue, and which way they
/// face: [`START_RING`] out from the middle on one of the diagonals, an eighth of the turn
/// round from +Z and a quarter on from one seat to the next, facing the middle.
pub fn arrival(seat: usize) -> (Vec3, Quat) {
    let angle = TAU / 8.0 + (seat % SHAPE.most) as f32 * TAU / 4.0;
    // A turn of `angle` puts +Z out where the seat stands, and points -Z
    // from there at the middle.
    let turn = Quat::from_rotation_y(angle);
    (turn * Vec3::new(0.0, 0.0, START_RING), turn)
}

/// What each seat is paid, from how the game came out (`results.places`, `scores`, `total`);
/// `people[seat]` is false for the computer's seats, which are paid nothing. By place, of
/// the four: 130, 60, 10 and nothing, those who went in at the very same moment sharing
/// ([`table::split`]).
pub fn prizes(results: &table::Results, people: &[bool]) -> Vec<u32> {
    table::split(&[130, 60, 10, 0], &results.places)
        .into_iter()
        .enumerate()
        .map(|(seat, coins)| if people.get(seat).copied().unwrap_or(false) { coins } else { 0 })
        .collect()
}

/// How far a turn of Knock Off has got.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum KnockStage {
    /// Everyone aims, nobody moves.
    Aim,
    /// Everyone's arrow is shown.
    Reveal,
    /// Everyone slides.
    Slide,
    /// The edge of the ice breaks off.
    Break,
    /// It is over: one is left, or nobody.
    Over,
}

/// A game of Knock Off as everyone in it sees it: on the entity its table's
/// room is sent, beside the board ([`table::Board`]), and sent to the room
/// whenever any of it changes (each stage, each tick of someone ready, each
/// body going in): a hundred bytes at the most, with four seats.
///
/// A device plays each slide itself: from `start` and `launches`, from
/// `since` by the town's clock (the [`Slide`] stepped to the clock less
/// `since`), the same to the bit as the server played it; and as the ice
/// breaks it settles everyone where the server has them (`start` again,
/// which by then is where the slide left them).
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct KnockRound {
    /// The turn, counted from 1; 0 before the game has begun.
    pub turn: u16,
    pub stage: KnockStage,
    /// When this stage began and when it ends, by the server's clock in
    /// seconds since 1970 (offline the device's own): the clock the game
    /// is stepped by ([`Game::step`]), which every device has for the
    /// town's, so that every device slides everyone at once. An aim ends
    /// early once everyone is ready; a game that is over ends when it
    /// began.
    pub since: f64,
    pub until: f64,
    /// How far the ice reaches now, in metres, and how far once this turn's
    /// break is over (drawn as cracks where it will break).
    pub radius: f32,
    pub next_radius: f32,
    /// Where each seat stood as this turn began, across the ground (x, z):
    /// the slide starts from here. None for whoever is out. While the ice
    /// breaks, where the slide left everyone, those it takes too (their
    /// `out` already says so), which is where the next turn begins.
    pub start: Vec<Option<(f32, f32)>>,
    /// Each seat's launch, a velocity across the ice (x, z) in metres a
    /// second, by seat: empty until the reveal, and again once the slide is
    /// over. Nobody's aim is anywhere in this before the reveal.
    pub launches: Vec<(f32, f32)>,
    /// Who is ready, for the ticks over their heads: people once they say
    /// so, the computer's players once they have made up their minds. Only
    /// while everyone aims.
    pub ready: Vec<bool>,
    /// The turn each seat went in, None while they are in: in a slide, from
    /// the moment they go over as the server plays it by the clock, which is
    /// when every device playing it sees them go; under the ice that breaks,
    /// as it begins to.
    pub out: Vec<Option<u16>>,
}

/// What a step of a game of Knock Off came to that whoever runs it carries
/// out ([`Game::step`]): the headlines, the lines for the chat, the splash.
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    /// A turn begins, counted from 1: everyone aims.
    Aim { turn: u16 },
    /// Everyone's aim is shown.
    Reveal,
    /// Everyone slides, now.
    Slide,
    /// `seat` has gone into the sea, and is out: over the edge in the
    /// slide, or with the ice that broke under them (`broke`); `by` whoever
    /// last hit them in this turn's slide, if anyone did ("Mochi knocked
    /// Dubu in").
    Fell { seat: usize, by: Option<usize>, broke: bool },
    /// The edge of the ice begins to break off: once it has, it reaches
    /// `radius`, the next turn's ice. Whoever stands on what breaks goes in
    /// with it ([`Event::Fell`], `broke`, just after).
    Breaks { radius: f32 },
    /// It is over: `first` came first, the one left on the ice, or whoever
    /// went in last (several, only if at the very same moment).
    Over { first: Vec<usize> },
}

/// A game of Knock Off: everything of it, as a plain thing stepped by
/// whoever runs it ([`Game::step`]), the server online and the device
/// offline, so that the two never differ. Its clock is the server's (the
/// device's own offline), in seconds since 1970, from [`Game::begin`] on:
/// told it at every step, and going by nothing else, so that it is never
/// behind the clock every device reads what it is sent by.
#[derive(Clone, Debug)]
pub struct Game {
    /// Each seat's, by seat: a person's (true) or the computer's.
    people: Vec<bool>,
    stage: KnockStage,
    turn: u16,
    /// The clock as it was last told ([`Game::step`]), and when this stage
    /// began and ends.
    now: f64,
    since: f64,
    until: f64,
    /// How far the ice reaches this turn.
    radius: f32,
    /// Where each seat stands across the ice as this turn began (while the
    /// ice breaks, as the slide left them); none once out.
    stand: Vec<Option<Vec2>>,
    /// Where each seat's body was last, whoever's, out or in; and which way
    /// it faces.
    last: Vec<Vec3>,
    yaw: Vec<f32>,
    /// Each seat's aim this turn: the way (of length one, or nothing) and
    /// the power.
    aims: Vec<(Vec2, u8)>,
    ready: Vec<bool>,
    /// The turn each went in, and when in it: the step of its slide they
    /// went over at, counted from 1; none, for whoever left before it; the
    /// most there is but one, for whoever the ice broke under after it; the
    /// most there is, for whoever left as it broke, after them.
    out: Vec<Option<u16>>,
    went: Vec<u32>,
    /// Everyone's launch this turn, from the reveal until the slide is over.
    launches: Vec<Vec2>,
    /// The slide being played, by the clock, and how it comes out, played
    /// out as it began.
    slide: Option<Slide>,
    ending: Option<Ending>,
    /// What everyone has been seen to do at the reveals so far, which the
    /// computer's players guess by.
    habits: Vec<Habits>,
    /// The computer's players, in their seats.
    minds: Vec<Option<Mind>>,
}

impl Game {
    /// A new game at a table of `seats` seats (four; fewer works), `people[seat]` false for
    /// the computer's, whose players are each one of [`PERSONAS`], no two alike, thrown from
    /// `seed`: the same seed, the same game. Nothing happens until [`Game::begin`].
    pub fn new(seats: usize, people: &[bool], seed: u64) -> Self {
        let mut dice = Dice::new(seed);
        // The computer's kinds, shuffled: each seat of theirs takes the next.
        let mut kinds: Vec<usize> = (0..PERSONAS.len()).collect();
        for at in (1..kinds.len()).rev() {
            let other = ((dice.roll() * (at + 1) as f32) as usize).min(at);
            kinds.swap(at, other);
        }
        let people: Vec<bool> = (0..seats).map(|seat| people.get(seat).copied().unwrap_or(false)).collect();
        let mut kinds = kinds.into_iter().cycle();
        let minds = (0..seats)
            .map(|seat| (!people[seat]).then(|| Mind::new(&PERSONAS[kinds.next().unwrap_or(0)], seed ^ (seat as u64 + 1).wrapping_mul(0xA24B_AED4_963E_E407))))
            .collect();
        let stand: Vec<Option<Vec2>> = (0..seats).map(|seat| Some(arrival(seat).0.xz())).collect();
        Self {
            last: stand.iter().map(|at| at.map_or(Vec3::ZERO, |at| Vec3::new(at.x, 0.0, at.y))).collect(),
            yaw: stand.iter().map(|at| at.map_or(0.0, |at| facing(-at).unwrap_or(0.0))).collect(),
            stand,
            stage: KnockStage::Aim,
            turn: 0,
            now: 0.0,
            since: 0.0,
            until: 0.0,
            radius: RADIUS,
            aims: vec![(Vec2::ZERO, 0); seats],
            ready: vec![false; seats],
            out: vec![None; seats],
            went: vec![0; seats],
            launches: Vec::new(),
            slide: None,
            ending: None,
            habits: vec![Habits::default(); seats],
            minds,
            people,
        }
    }

    /// The countdown is over, at `clock` (by the clock every step after is
    /// told the time by, [`Game::step`]): everyone stands on their spot
    /// ([`arrival`]), facing the middle, and the first turn's aim begins.
    pub fn begin(&mut self, clock: f64) -> Vec<Event> {
        let mut events = Vec::new();
        if self.turn != 0 {
            return events;
        }
        let seats = self.people.len();
        self.now = clock;
        self.since = clock;
        self.until = clock;
        self.turn = 1;
        self.radius = radius_in(1);
        for seat in 0..seats {
            if self.out[seat].is_none() {
                let at = arrival(seat).0;
                self.stand[seat] = Some(at.xz());
                self.last[seat] = at;
            }
        }
        if self.in_play() <= 1 {
            self.finish(&mut events);
        } else {
            self.aim_begins(&mut events);
        }
        events
    }

    /// Whether the player in `seat` is still on the ice.
    pub fn in_play_at(&self, seat: usize) -> bool {
        self.out.get(seat).is_some_and(Option::is_none)
    }

    /// How many are still on the ice.
    fn in_play(&self) -> usize {
        self.out.iter().filter(|out| out.is_none()).count()
    }

    /// Whether `seat` is a person's, still on the ice.
    fn person_in(&self, seat: usize) -> bool {
        self.people.get(seat).copied().unwrap_or(false) && self.in_play_at(seat)
    }

    /// The person in `seat` aims `way` (across the ground, x and z; any length, and nothing is
    /// staying put) with `power` (0 to [`MOST_POWER`]; more is the most): heard only while
    /// everyone aims, from someone still on the ice who has not yet said they are ready, and
    /// only a way that is a number. Their aim until they aim again. Not aiming at all is
    /// staying where you are.
    pub fn aim(&mut self, seat: usize, way: Vec2, power: u8) {
        if self.stage != KnockStage::Aim || self.turn == 0 || !self.person_in(seat) || self.ready[seat] {
            return;
        }
        if !way.x.is_finite() || !way.y.is_finite() {
            return;
        }
        let length = way.length();
        let way = if length > 1e-6 { way / length } else { Vec2::ZERO };
        self.aims[seat] = (way, power.min(MOST_POWER));
    }

    /// The person in `seat` is ready: their aim stays as it is, and once every person still on
    /// the ice is ready the aim is over (the computer is always ready). Heard only while
    /// everyone aims.
    pub fn ready(&mut self, seat: usize) {
        if self.stage != KnockStage::Aim || self.turn == 0 || !self.person_in(seat) {
            return;
        }
        self.ready[seat] = true;
        self.aim_may_end(self.now);
    }

    /// The aim is over, from `from` on, once every person still on the ice is ready (the
    /// computer always is); and with no person on it, once every one of the computer's
    /// players still on it has made up its mind, each at the moment thrown for it, so that
    /// whoever looks on sees them take their time.
    fn aim_may_end(&mut self, from: f64) {
        if self.stage != KnockStage::Aim || self.turn == 0 {
            return;
        }
        let seats = self.people.len();
        if (0..seats).any(|seat| self.person_in(seat)) {
            if (0..seats).filter(|&seat| self.person_in(seat)).all(|seat| self.ready[seat]) {
                self.until = self.until.min(from);
            }
        } else {
            let minds_done = (0..seats)
                .filter(|&seat| self.in_play_at(seat))
                .filter_map(|seat| self.minds[seat].as_ref())
                .map(Mind::decides_at)
                .fold(from, f64::max);
            self.until = self.until.min(minds_done);
        }
    }

    /// The aim the person in `seat` has this turn, as the game holds it: for a device that
    /// comes back while everyone aims ([`Tell::Aimed`]). None while nobody aims, or for the
    /// computer's.
    pub fn aim_of(&self, seat: usize) -> Option<(Vec2, u8)> {
        (self.stage == KnockStage::Aim && self.person_in(seat)).then(|| self.aims[seat])
    }

    /// The player in `seat` has left for good: they are out now, in this turn, as though they
    /// went in the moment they left (as the ice breaks, after whoever it has begun to take).
    /// Taken off the ice at once while everyone aims or looks at the arrows, and while
    /// everyone aims, the computer's players who have not made up their minds think again
    /// without them; in the slide, theirs goes on as it began (everyone plays it so) and they
    /// are gone once it is over.
    pub fn leave(&mut self, seat: usize) {
        if seat >= self.people.len() || self.out[seat].is_some() || self.stage == KnockStage::Over {
            return;
        }
        self.out[seat] = Some(self.turn);
        self.went[seat] = match self.stage {
            KnockStage::Slide => self.slide.as_ref().map_or(0, |slide| slide.steps() + 1),
            KnockStage::Break => u32::MAX,
            _ => 0,
        };
        self.aims[seat] = (Vec2::ZERO, 0);
        self.ready[seat] = false;
        match self.stage {
            KnockStage::Aim | KnockStage::Reveal | KnockStage::Break => {
                self.stand[seat] = None;
                if let Some(launch) = self.launches.get_mut(seat) {
                    *launch = Vec2::ZERO;
                }
            }
            _ => {}
        }
        if self.stage == KnockStage::Aim && self.turn != 0 {
            let next = radius_after(self.turn);
            for other in 0..self.people.len() {
                if self.out[other].is_none()
                    && let Some(mind) = &mut self.minds[other]
                {
                    mind.replan(other, &self.stand, self.radius, next, &self.habits);
                }
            }
        }
        // Those left may all be ready now.
        self.aim_may_end(self.now);
    }

    /// A step of play, at `clock`: the time by the clock [`Game::begin`] was given, the very
    /// one every device reads [`KnockRound::since`] and `until` by (online the server's time
    /// of day, which every device has for the town's clock; offline the device's own, as its
    /// town's clock is). The computer's players think and make up their minds, a slide goes
    /// on, and each stage gives way to the next when its time is up. The game goes by this
    /// alone, never by how long a frame took: a frame that stalls, or a phone put away and
    /// brought back, leaves it where the clock is, not behind it for the rest of the game
    /// (a frame's own time is held to a quarter of a second by the engine, however long it
    /// stalled). Never back: a clock put back a little holds it where it was; one that is
    /// not a number is not heard. Everything that happened, for whoever runs the game to
    /// carry out.
    pub fn step(&mut self, clock: f64) -> Vec<Event> {
        let mut events = Vec::new();
        if self.turn == 0 || self.stage == KnockStage::Over {
            return events;
        }
        if clock.is_finite() && clock > self.now {
            self.now = clock;
        }
        if self.stage == KnockStage::Aim {
            self.think();
        }
        // Each stage over gives way to the next, on the minute it was to:
        // a long step passes through as many as it covers, a whole game's
        // at the most (four a turn).
        let mut stages = 0;
        while self.stage != KnockStage::Over && stages <= 4 * usize::from(LAST_TURN) {
            stages += 1;
            if matches!(self.stage, KnockStage::Aim | KnockStage::Reveal) && self.in_play() <= 1 {
                self.since = self.now.min(self.until);
                self.finish(&mut events);
                break;
            }
            if self.now < self.until {
                break;
            }
            match self.stage {
                KnockStage::Aim => self.reveal(&mut events),
                KnockStage::Reveal => self.slide_begins(&mut events),
                KnockStage::Slide => self.slide_over(&mut events),
                KnockStage::Break => self.broken(&mut events),
                KnockStage::Over => {}
            }
        }
        // Whatever is sliding, as far as the clock has come: a slide that
        // began in this step too.
        if self.stage == KnockStage::Slide {
            self.slide_on(&mut events);
        }
        events
    }

    /// A new turn's aim begins: everyone's aim is nothing again, everyone
    /// faces the middle, and the computer's players begin to think. With
    /// nobody's person on the ice it is over once every one of the computer's
    /// has made up its mind, so that whoever looks on sees them take their
    /// time.
    fn aim_begins(&mut self, events: &mut Vec<Event>) {
        let seats = self.people.len();
        self.stage = KnockStage::Aim;
        self.since = self.until;
        let secs = if self.turn == 1 { FIRST_AIM } else { AIM };
        self.until = self.since + f64::from(secs);
        self.aims = vec![(Vec2::ZERO, 0); seats];
        self.ready = vec![false; seats];
        self.launches.clear();
        self.slide = None;
        self.ending = None;
        for seat in 0..seats {
            if let Some(at) = self.stand[seat]
                && let Some(yaw) = facing(-at)
            {
                self.yaw[seat] = yaw;
            }
        }
        let next = radius_after(self.turn);
        for seat in 0..seats {
            if self.out[seat].is_some() {
                continue;
            }
            if let Some(mind) = &mut self.minds[seat] {
                mind.plan(seat, &self.stand, self.radius, next, &self.habits, self.since, secs);
            }
        }
        self.aim_may_end(self.since);
        events.push(Event::Aim { turn: self.turn });
    }

    /// The computer's players weigh their moves, so many slides a step
    /// between them, and each makes up its mind at its moment.
    fn think(&mut self) {
        let mut budget = THINK_A_STEP;
        for seat in 0..self.people.len() {
            if self.out[seat].is_some() {
                continue;
            }
            let Some(mind) = &mut self.minds[seat] else { continue };
            if mind.decided() {
                continue;
            }
            mind.ponder(&mut budget);
            if self.now >= mind.decides_at() {
                self.aims[seat] = mind.decide();
                self.ready[seat] = true;
            }
        }
    }

    /// The aim is over: whoever of the computer's has not made up its mind
    /// does, everyone's launch is fixed and shown, and what everyone did is
    /// marked for the computer's players to go by.
    fn reveal(&mut self, events: &mut Vec<Event>) {
        let seats = self.people.len();
        for seat in 0..seats {
            if self.out[seat].is_none()
                && let Some(mind) = &mut self.minds[seat]
                && !mind.decided()
            {
                self.aims[seat] = mind.decide();
            }
        }
        self.launches = (0..seats)
            .map(|seat| if self.out[seat].is_none() && self.stand[seat].is_some() { launch(self.aims[seat].0, self.aims[seat].1) } else { Vec2::ZERO })
            .collect();
        for seat in 0..seats {
            if let Some(at) = self.stand[seat].filter(|_| self.out[seat].is_none()) {
                self.habits[seat].saw(seat, at, self.launches[seat], &self.stand);
                if let Some(yaw) = facing(self.launches[seat]) {
                    self.yaw[seat] = yaw;
                }
            }
        }
        self.ready = vec![false; seats];
        self.stage = KnockStage::Reveal;
        self.since = self.until;
        self.until = self.since + f64::from(REVEAL);
        events.push(Event::Reveal);
    }

    /// Everyone sets off: the slide is played out at once, for how it ends
    /// and how long it takes, and again by the clock, for where everyone is
    /// as it goes.
    fn slide_begins(&mut self, events: &mut Vec<Event>) {
        let start: Vec<Option<Vec2>> = (0..self.people.len()).map(|seat| self.stand[seat].filter(|_| self.out[seat].is_none())).collect();
        let ending = Slide::play_out(&start, &self.launches, self.radius);
        self.stage = KnockStage::Slide;
        self.since = self.until;
        self.until = self.since + f64::from(ending.secs + SETTLE);
        self.slide = Some(Slide::new(&start, &self.launches, self.radius));
        self.ending = Some(ending);
        events.push(Event::Slide);
    }

    /// The slide goes on by the clock: whoever goes over is out the moment
    /// they do.
    fn slide_on(&mut self, events: &mut Vec<Event>) {
        let Some(slide) = &mut self.slide else { return };
        slide.step_to((self.now.min(self.until) - self.since) as f32);
        let mut fell = Vec::new();
        for seat in 0..self.people.len() {
            if let Some(at) = slide.at(seat) {
                self.last[seat] = at;
            }
            if let Some(when) = slide.over_edge(seat)
                && self.out[seat].is_none()
            {
                fell.push((when, seat, slide.hit_by(seat)));
            }
        }
        self.fell(fell, events);
    }

    /// Whoever of `fell` (when in the slide, who, who hit them last) is not
    /// out yet is, now: in the order they went over, however the steps
    /// fell, so that the same game tells the same story at any frame rate.
    fn fell(&mut self, mut fell: Vec<(f32, usize, Option<usize>)>, events: &mut Vec<Event>) {
        fell.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for (when, seat, by) in fell {
            if self.out[seat].is_none() {
                self.out[seat] = Some(self.turn);
                self.went[seat] = (when / STEP).round() as u32 + 1;
                events.push(Event::Fell { seat, by, broke: false });
            }
        }
    }

    /// The slide is over: everyone is where it left them, whoever went over
    /// is out, and the ice breaks; or, with one left or nobody, the game is
    /// over.
    fn slide_over(&mut self, events: &mut Vec<Event>) {
        // The slide played on to its end by the clock, whoever went over
        // seen into the sea where they went, however few steps it was
        // stepped by (a frame that stalled may have gone past all of it).
        self.slide_on(events);
        let ending = self.ending.take().unwrap_or_else(|| Slide::new(&self.stand, &[], self.radius).ending());
        // Catching up with whatever the clock did not get to.
        let fell = (0..self.people.len()).filter_map(|seat| ending.fell[seat].map(|when| (when, seat, ending.by[seat]))).collect();
        self.fell(fell, events);
        for seat in 0..self.people.len() {
            match ending.at[seat] {
                Some(at) if self.out[seat].is_none() => {
                    self.stand[seat] = Some(at);
                    self.last[seat] = Vec3::new(at.x, 0.0, at.y);
                }
                _ => {
                    self.stand[seat] = None;
                    self.last[seat].y = -SUNK;
                }
            }
        }
        self.launches.clear();
        self.slide = None;
        if self.in_play() <= 1 {
            self.since = self.until;
            self.finish(events);
            return;
        }
        // The ice breaks, and takes whoever stands where it breaks.
        self.stage = KnockStage::Break;
        self.since = self.until;
        self.until = self.since + f64::from(BREAK);
        let next = radius_after(self.turn);
        events.push(Event::Breaks { radius: next });
        for seat in 0..self.people.len() {
            if let Some(at) = self.stand[seat]
                && self.out[seat].is_none()
                && (next <= 0.0 || at.length_squared() > (next + ON_THE_LINE) * (next + ON_THE_LINE))
            {
                self.out[seat] = Some(self.turn);
                // After every fall of the slide, and before whoever leaves
                // while it breaks ([`Game::leave`]).
                self.went[seat] = u32::MAX - 1;
                events.push(Event::Fell { seat, by: ending.by[seat], broke: true });
            }
        }
    }

    /// The ice has broken: whoever it took is in the sea, and the next turn
    /// begins; or, with one left or nobody, the game is over.
    fn broken(&mut self, events: &mut Vec<Event>) {
        self.radius = radius_after(self.turn);
        for seat in 0..self.people.len() {
            if self.out[seat].is_some() && self.stand[seat].take().is_some() {
                self.last[seat].y = -SUNK;
            }
        }
        if self.in_play() <= 1 {
            self.since = self.until;
            self.finish(events);
            return;
        }
        self.turn += 1;
        self.aim_begins(events);
    }

    /// It is over: whoever is left on the ice came first, or, with nobody
    /// left, whoever went in last.
    fn finish(&mut self, events: &mut Vec<Event>) {
        self.stage = KnockStage::Over;
        self.until = self.since;
        self.launches.clear();
        self.slide = None;
        self.ending = None;
        self.ready = vec![false; self.people.len()];
        let (places, _) = self.places();
        events.push(Event::Over {
            first: (0..places.len()).filter(|&seat| places[seat] == 0).collect(),
        });
    }

    /// Everyone's place, 0 first, by when they went in, last first: by the
    /// turn, and in a turn by the moment (those who went in at the very same
    /// one sharing a place), whoever is still on the ice before them all; and
    /// how many turns each was still on the ice at the end of.
    fn places(&self) -> (Vec<u8>, Vec<u32>) {
        let lasted = |seat: usize| self.out[seat].map_or((u32::MAX, u32::MAX), |turn| (u32::from(turn), self.went[seat]));
        let places = (0..self.out.len())
            .map(|seat| (0..self.out.len()).filter(|&other| lasted(other) > lasted(seat)).count() as u8)
            .collect();
        let survived = (0..self.out.len())
            .map(|seat| self.out[seat].map_or(u32::from(self.turn), |turn| u32::from(turn).saturating_sub(1)))
            .collect();
        (places, survived)
    }

    /// Where the body in `seat` is and which way it faces (a turn about the upright, from -Z):
    /// the computer's, which the game alone moves, and everyone's, in play. On the ice, as the
    /// turn began, or as the slide has them; in the sea, under it.
    pub fn at(&self, seat: usize) -> (Vec3, f32) {
        let yaw = self.yaw.get(seat).copied().unwrap_or(0.0);
        if self.stage == KnockStage::Slide
            && let Some(at) = self.slide.as_ref().and_then(|slide| slide.at(seat))
        {
            return (at, yaw);
        }
        match self.stand.get(seat).copied().flatten() {
            Some(at) => (Vec3::new(at.x, 0.0, at.y), yaw),
            None => (self.last.get(seat).copied().unwrap_or(Vec3::new(0.0, -SUNK, 0.0)), yaw),
        }
    }

    /// What everyone in the game is sent of it ([`KnockRound`]): nothing of anyone's aim before
    /// the reveal. Cheap to make and to compare: whoever sends it sends it when it changes.
    pub fn round(&self) -> KnockRound {
        let shown = matches!(self.stage, KnockStage::Reveal | KnockStage::Slide);
        KnockRound {
            turn: self.turn,
            stage: self.stage,
            since: self.since,
            until: self.until,
            radius: self.radius,
            next_radius: if self.stage == KnockStage::Over { self.radius } else { radius_after(self.turn) },
            start: self.stand.iter().map(|at| at.map(|at| (at.x, at.y))).collect(),
            launches: if shown { self.launches.iter().map(|launch| (launch.x, launch.y)).collect() } else { Vec::new() },
            ready: if self.stage == KnockStage::Aim { self.ready.clone() } else { vec![false; self.ready.len()] },
            out: self.out.clone(),
        }
    }

    /// Over: each seat's place, 0 first (those who went in at the very same moment tied), and
    /// how many turns each was still on the ice at the end of (the results' score); no total.
    /// None while it is played.
    pub fn outcome(&self) -> Option<(Vec<u8>, Vec<u32>, Option<u32>)> {
        (self.stage == KnockStage::Over).then(|| {
            let (places, survived) = self.places();
            (places, survived, None)
        })
    }

    /// How far it has got, and in which turn (0 before it has begun).
    pub fn stage(&self) -> KnockStage {
        self.stage
    }

    pub fn turn(&self) -> u16 {
        self.turn
    }

    /// Which of [`PERSONAS`] the computer's player in `seat` is, by name; None for a person's.
    pub fn persona(&self, seat: usize) -> Option<&'static str> {
        self.minds.get(seat)?.as_ref().map(|mind| mind.persona().name)
    }
}

/// Which way something faces that looks along `way` across the ground: a
/// turn about the upright from -Z, as a body's. None for no way at all.
fn facing(way: Vec2) -> Option<f32> {
    (way.length_squared() > 1e-8).then(|| (-way.x).atan2(-way.y))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    /// The clock a test game begins by: a day in November 2026, in seconds
    /// since 1970, as the server's would be.
    const CLOCK: f64 = 1_795_000_000.0;

    /// Steps `game` on once, `dt` seconds on by its clock from the last
    /// time it was told it.
    fn tick(game: &mut Game, dt: f64) -> Vec<Event> {
        let clock = game.now + dt;
        game.step(clock)
    }

    /// Steps `game` on, `dt` at a time, until it is over or `secs` have
    /// passed; everything it came to, in order.
    fn run(game: &mut Game, dt: f64, secs: f64) -> Vec<Event> {
        let mut events = Vec::new();
        let mut gone = 0.0;
        while game.outcome().is_none() && gone < secs {
            events.extend(tick(game, dt));
            gone += dt;
        }
        events
    }

    /// Steps `game` on until it is at `stage` (or over), `dt` at a time.
    fn run_to(game: &mut Game, stage: KnockStage, dt: f64) -> Vec<Event> {
        let mut events = Vec::new();
        for _ in 0..100_000 {
            if game.stage() == stage || game.outcome().is_some() {
                break;
            }
            events.extend(tick(game, dt));
        }
        events
    }

    #[test]
    fn the_ice_breaks_a_metre_a_turn_and_then_to_nothing() {
        let radii: Vec<f32> = (1..=LAST_TURN + 1).map(radius_in).collect();
        assert_eq!(radii, [12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.25, 1.5, 0.75, 0.0]);
        assert_eq!(radius_in(0), RADIUS);
        assert_eq!(radius_after(LAST_TURN), 0.0);
        assert_eq!(radius_after(u16::MAX), 0.0);
        for turn in 1..=LAST_TURN {
            assert!(radius_after(turn) < radius_in(turn), "turn {turn}");
        }
        // The arrow is as long as the slide; ten all but crosses the island.
        assert_eq!(reach(10), 10.0 * METRES_A_POWER);
        assert!(reach(MOST_POWER) > 1.8 * RADIUS && reach(MOST_POWER) < 2.0 * RADIUS);
        assert_eq!(reach(200), reach(MOST_POWER));
        assert!((speed(4) * speed(4) / (2.0 * ICE) - reach(4)).abs() < 1e-4);
        assert_eq!(launch(Vec2::new(3.0, 4.0), 0), Vec2::ZERO);
        assert_eq!(launch(Vec2::ZERO, 7), Vec2::ZERO);
        assert_eq!(launch(Vec2::new(f32::NAN, 1.0), 7), Vec2::ZERO);
        assert!((launch(Vec2::new(3.0, 4.0), 7).length() - speed(7)).abs() < 1e-5);
    }

    #[test]
    fn turns_run_aim_reveal_slide_break_one_after_another() {
        // Two people and two of the computer's; the people are ready at
        // once, and aim nowhere.
        let mut game = Game::new(4, &[true, false, true, false], 3);
        assert_eq!(game.round().turn, 0);
        assert_eq!(game.step(CLOCK - 0.1), vec![]);
        let begun = game.begin(CLOCK);
        assert_eq!(begun, vec![Event::Aim { turn: 1 }]);
        let round = game.round();
        assert_eq!((round.stage, round.since, round.until), (KnockStage::Aim, CLOCK, CLOCK + f64::from(FIRST_AIM)));
        assert_eq!((round.radius, round.next_radius), (12.0, 11.0));
        // Everyone on their spot, facing the middle.
        for seat in 0..4 {
            let (at, yaw) = game.at(seat);
            assert_eq!(at, arrival(seat).0);
            let facing = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
            assert!(facing.xz().distance(-at.xz().normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
        // The stages go by in order, each beginning as the last ended.
        let mut seen = Vec::new();
        for _ in 0..20_000 {
            if game.stage() == KnockStage::Aim {
                game.ready(0);
                game.ready(2);
            }
            let last = game.round();
            let events = tick(&mut game, 1.0 / 60.0);
            let round = game.round();
            if round.stage != last.stage || round.turn != last.turn {
                assert_eq!(round.since, last.until, "{last:?} then {round:?}");
                assert!(round.until >= round.since);
                seen.push((round.turn, round.stage));
            }
            for event in &events {
                if let Event::Fell { seat, .. } = event {
                    assert_eq!(round.out[*seat], Some(round.turn), "{event:?}");
                }
            }
            if game.outcome().is_some() {
                break;
            }
        }
        assert!(game.outcome().is_some(), "never over: {seen:?}");
        let turns = game.turn();
        for (index, &(turn, stage)) in seen.iter().enumerate() {
            let expected = [KnockStage::Reveal, KnockStage::Slide, KnockStage::Break, KnockStage::Aim][index % 4];
            if (turn, stage) == (turns, KnockStage::Over) {
                assert_eq!(index, seen.len() - 1);
                break;
            }
            assert_eq!(stage, expected, "{seen:?}");
            assert_eq!(turn, 1 + (index as u16 + 1) / 4, "{seen:?}");
        }
    }

    #[test]
    fn nobody_s_aim_is_in_the_round_before_the_reveal() {
        let mut game = Game::new(4, &[true, true, false, false], 5);
        game.begin(CLOCK);
        game.aim(0, Vec2::new(-1.0, -1.0), 4);
        game.aim(1, Vec2::new(0.0, 1.0), 12);
        run(&mut game, 0.05, 5.0);
        assert_eq!(game.stage(), KnockStage::Aim);
        let round = game.round();
        assert!(round.launches.is_empty(), "{round:?}");
        // Nothing in it moves, whoever aimed what, and the computer's
        // players face the middle however they have made up their minds.
        let mut other = Game::new(4, &[true, true, false, false], 5);
        other.begin(CLOCK);
        run(&mut other, 0.05, 5.0);
        let theirs = other.round();
        assert_eq!((round.start.clone(), round.out.clone()), (theirs.start.clone(), theirs.out.clone()));
        for seat in 0..4 {
            assert_eq!(game.at(seat), other.at(seat), "seat {seat}");
        }
        // Only your own is yours to know.
        let (way, power) = game.aim_of(0).unwrap();
        assert!(way.distance(Vec2::new(-1.0, -1.0).normalize()) < 1e-6 && power == 4, "{way} {power}");
        assert_eq!(game.aim_of(1), Some((Vec2::Y, MOST_POWER)));
        assert_eq!(game.aim_of(2), None);
        // At the reveal, everyone's, exactly as each will set off.
        run_to(&mut game, KnockStage::Reveal, 0.05);
        let round = game.round();
        assert_eq!(round.launches.len(), 4);
        let launched = launch(way, power);
        assert_eq!(round.launches[0], (launched.x, launched.y));
        assert_eq!(round.launches[1], (0.0, speed(MOST_POWER)));
        assert!(round.ready.iter().all(|ready| !ready));
        // And aiming once it is over is heard by nobody.
        game.aim(0, Vec2::X, 9);
        assert_eq!(game.round().launches[0], (launched.x, launched.y));
        assert_eq!(game.aim_of(0), None);
    }

    #[test]
    fn the_aim_ends_early_only_once_every_person_is_ready() {
        let mut game = Game::new(4, &[true, false, true, false], 8);
        game.begin(CLOCK);
        run(&mut game, 0.1, 1.0);
        game.ready(0);
        // One of two: the aim goes on.
        run(&mut game, 0.1, 2.0);
        assert_eq!(game.stage(), KnockStage::Aim);
        assert!(game.round().ready[0] && !game.round().ready[2]);
        // Being ready, your aim stays as it was.
        game.aim(0, Vec2::X, 5);
        assert_eq!(game.aim_of(0), Some((Vec2::ZERO, 0)));
        // Ready for nobody else's seat, nor the computer's.
        game.ready(1);
        game.ready(9);
        run(&mut game, 0.1, 1.0);
        assert_eq!(game.stage(), KnockStage::Aim);
        // Both: over at the next step, well before its time, the computer's
        // players having made up their minds then whenever they meant to.
        game.ready(2);
        let when = game.now;
        let events = tick(&mut game, 0.1);
        assert!(events.contains(&Event::Reveal), "{events:?}");
        assert_eq!(game.round().since, when);
        assert!(when < CLOCK + f64::from(FIRST_AIM) - 5.0);
        // A person who never says so holds it to its time.
        let mut game = Game::new(4, &[true, false, false, false], 8);
        game.begin(CLOCK);
        run_to(&mut game, KnockStage::Reveal, 0.1);
        assert!((game.round().since - (CLOCK + f64::from(FIRST_AIM))).abs() < 0.11);
        // The computer alone: over once the last of them has made up its
        // mind, which is a while, and never the whole of it.
        for seed in 0..20 {
            let mut game = Game::new(4, &[false; 4], seed);
            game.begin(CLOCK);
            run_to(&mut game, KnockStage::Reveal, 0.05);
            let took = game.round().since - CLOCK;
            assert!(took > 1.0 && took < f64::from(FIRST_AIM) - 0.3, "seed {seed}: {took} s");
        }
    }

    #[test]
    fn standing_still_the_ice_takes_everyone_and_they_share_first() {
        // Four people who never aim: on their spots, 7 m out, until the
        // ice breaks to 6 m at the end of the sixth turn and takes them all.
        let mut game = Game::new(4, &[true; 4], 1);
        let mut events = game.begin(CLOCK);
        events.extend(run(&mut game, 0.1, 1000.0));
        let (places, scores, total) = game.outcome().expect("over");
        assert_eq!(game.turn(), 6);
        assert_eq!(places, vec![0; 4]);
        assert_eq!(scores, vec![5; 4]);
        assert_eq!(total, None);
        let fell: Vec<&Event> = events.iter().filter(|event| matches!(event, Event::Fell { .. })).collect();
        assert_eq!(fell.len(), 4);
        assert!(fell.iter().all(|event| matches!(event, Event::Fell { broke: true, by: None, .. })), "{fell:?}");
        assert_eq!(events.last(), Some(&Event::Over { first: vec![0, 1, 2, 3] }));
        // Shared between them, 200 coins four ways.
        let results = table::Results { places, ..default() };
        assert_eq!(prizes(&results, &[true; 4]), vec![50; 4]);
    }

    #[test]
    fn the_last_of_the_ice_takes_whoever_is_left_on_it() {
        // Two people who have kept to the middle of the last floe, touching,
        // standing still: the ice gives out under both at the end of the last
        // turn, at the same moment, and they share first.
        let mut game = Game::new(4, &[true; 4], 12);
        game.begin(CLOCK);
        game.leave(2);
        game.leave(3);
        game.turn = LAST_TURN;
        game.radius = radius_in(LAST_TURN);
        game.stand[0] = Some(Vec2::new(0.6, 0.0));
        game.stand[1] = Some(Vec2::new(-0.6, 0.0));
        let events = run(&mut game, 0.1, 100.0);
        assert!(events.contains(&Event::Breaks { radius: 0.0 }), "{events:?}");
        assert!(events.contains(&Event::Fell { seat: 0, by: None, broke: true }) && events.contains(&Event::Fell { seat: 1, by: None, broke: true }));
        let (places, scores, _) = game.outcome().expect("over");
        assert_eq!(game.turn(), LAST_TURN);
        assert_eq!(places, vec![0, 0, 2, 2]);
        assert_eq!(scores, vec![u32::from(LAST_TURN) - 1, u32::from(LAST_TURN) - 1, 0, 0]);
    }

    #[test]
    fn places_go_by_when_you_went_in_last_first() {
        let mut game = Game::new(4, &[true; 4], 2);
        game.begin(CLOCK);
        // Seat 3 leaves in the first turn's aim, and is out in it.
        game.leave(3);
        assert_eq!(game.round().out, vec![None, None, None, Some(1)]);
        assert_eq!(game.round().start[3], None);
        // Seat 1 slides off the edge in the first turn: in it too, but
        // later, and so above seat 3.
        game.aim(1, arrival(1).0.xz(), MOST_POWER);
        let events = run_to(&mut game, KnockStage::Break, 0.05);
        assert!(events.contains(&Event::Fell { seat: 1, by: None, broke: false }), "{events:?}");
        assert_eq!(game.round().out, vec![None, Some(1), None, Some(1)]);
        // Seat 2 slides to the middle the second turn; seat 0 stays, and the
        // ice takes it at the end of the sixth.
        run_to(&mut game, KnockStage::Aim, 0.05);
        assert_eq!(game.turn(), 2);
        game.aim(2, -arrival(2).0.xz(), 3);
        run(&mut game, 0.05, 1000.0);
        let (places, scores, _) = game.outcome().expect("over");
        assert_eq!(game.turn(), 6);
        assert_eq!(places, vec![1, 2, 0, 3]);
        assert_eq!(scores, vec![5, 0, 6, 0]);
        let results = table::Results { places, ..default() };
        assert_eq!(prizes(&results, &[true; 4]), vec![60, 10, 130, 0]);
        // Whoever came first is the one still on the ice, and over is over.
        assert!(game.in_play_at(2) && !game.in_play_at(0));
        game.leave(2);
        game.aim(2, Vec2::X, 3);
        assert_eq!(game.outcome().unwrap().0, vec![1, 2, 0, 3]);
        assert_eq!(tick(&mut game, 1.0), vec![]);
    }

    #[test]
    fn of_two_who_go_in_in_one_slide_whoever_stayed_on_longer_comes_first() {
        // The last two slide off the ice in the same slide, one from nearer
        // the edge than the other: the further one lasts longer and wins.
        let mut game = Game::new(2, &[true, true], 9);
        game.begin(CLOCK);
        game.aim(0, arrival(0).0.xz(), MOST_POWER);
        game.aim(1, arrival(1).0.xz(), 2 * MOST_POWER / 3);
        let events = run(&mut game, 0.05, 100.0);
        let fell: Vec<usize> = events.iter().filter_map(|event| if let Event::Fell { seat, .. } = event { Some(*seat) } else { None }).collect();
        assert_eq!(fell, vec![0, 1]);
        assert_eq!(game.outcome().unwrap().0, vec![1, 0]);
        assert_eq!(events.last(), Some(&Event::Over { first: vec![1] }));
        // And two who go in at the very same moment share it.
        let mut game = Game::new(2, &[true, true], 9);
        game.begin(CLOCK);
        game.aim(0, arrival(0).0.xz(), MOST_POWER);
        game.aim(1, arrival(1).0.xz(), MOST_POWER);
        run(&mut game, 0.05, 100.0);
        assert_eq!(game.outcome().unwrap().0, vec![0, 0]);
    }

    #[test]
    fn the_last_to_leave_but_one_leaves_the_game_to_them() {
        let mut game = Game::new(4, &[true, false, true, false], 4);
        game.begin(CLOCK);
        run(&mut game, 0.1, 0.5);
        // Two go (the game minds nobody's seat over another's); the two
        // people left play on.
        game.leave(1);
        game.leave(3);
        let events = run(&mut game, 0.1, 0.5);
        assert!(!events.iter().any(|event| matches!(event, Event::Over { .. })));
        // The one's opponent goes; they have won, there and then.
        game.leave(2);
        let events = tick(&mut game, 0.1);
        assert_eq!(events, vec![Event::Over { first: vec![0] }]);
        let (places, scores, _) = game.outcome().unwrap();
        assert_eq!(places, vec![0, 1, 1, 1]);
        assert_eq!(scores, vec![1, 0, 0, 0]);
        // Leaving twice changes nothing.
        game.leave(2);
        assert_eq!(game.outcome().unwrap().0, vec![0, 1, 1, 1]);
    }

    #[test]
    fn someone_leaving_in_the_slide_goes_on_sliding_in_it() {
        // Seat 0 slides at seat 1 hard; seat 1 leaves as everyone slides:
        // the slide is played as it began, everyone's devices playing the
        // same, and seat 1 is gone at its end.
        let mut game = Game::new(4, &[true, true, true, true], 6);
        game.begin(CLOCK);
        game.aim(0, arrival(1).0.xz() - arrival(0).0.xz(), 6);
        for seat in 0..4 {
            game.ready(seat);
        }
        run_to(&mut game, KnockStage::Slide, 0.05);
        let launches = game.round().launches.clone();
        run(&mut game, 0.05, 0.2);
        game.leave(1);
        assert_eq!(game.round().launches, launches);
        assert_eq!(game.round().out[1], Some(1));
        run_to(&mut game, KnockStage::Break, 0.05);
        assert_eq!(game.round().start[1], None);
    }

    #[test]
    fn leaving_as_the_ice_breaks_is_going_in_after_whoever_it_takes() {
        // Three people: two slide out to 11.4 m, past where the ice breaks
        // to 11 at the end of the turn, and it takes them; the third, the
        // one left on it, leaves as it breaks. They went in last of the
        // three, and come first.
        let mut game = Game::new(3, &[true; 3], 7);
        game.begin(CLOCK);
        game.aim(0, arrival(0).0.xz(), 2);
        game.aim(1, arrival(1).0.xz(), 2);
        for seat in 0..3 {
            game.ready(seat);
        }
        let events = run_to(&mut game, KnockStage::Break, 0.05);
        assert!(events.contains(&Event::Fell { seat: 0, by: None, broke: true }) && events.contains(&Event::Fell { seat: 1, by: None, broke: true }), "{events:?}");
        assert_eq!(game.stage(), KnockStage::Break);
        game.leave(2);
        let events = run(&mut game, 0.05, 10.0);
        assert_eq!(events.last(), Some(&Event::Over { first: vec![2] }));
        let (places, scores, _) = game.outcome().expect("over");
        assert_eq!(places, vec![1, 1, 0]);
        assert_eq!(scores, vec![0, 0, 0]);
        let results = table::Results { places, ..default() };
        assert_eq!(prizes(&results, &[true; 3]), vec![35, 35, 130]);
        // With more left on the ice, whoever leaves as it breaks places
        // below those still on it, and above whoever it took.
        let mut game = Game::new(4, &[true; 4], 7);
        game.begin(CLOCK);
        game.aim(0, arrival(0).0.xz(), 2);
        for seat in 0..4 {
            game.ready(seat);
        }
        run_to(&mut game, KnockStage::Break, 0.05);
        assert_eq!(game.round().out, vec![Some(1), None, None, None]);
        game.leave(3);
        run_to(&mut game, KnockStage::Aim, 0.05);
        game.leave(1);
        run(&mut game, 0.05, 1000.0);
        assert_eq!(game.outcome().expect("over").0, vec![3, 1, 0, 2]);
    }

    #[test]
    fn whoever_leaves_as_everyone_aims_is_thought_no_more_of() {
        // A person and three of the computer's on their spots; the person
        // leaves before any of the computer's has made up its mind.
        for seed in 0..20 {
            let mut game = Game::new(4, &[true, false, false, false], seed);
            game.begin(CLOCK);
            tick(&mut game, 0.05);
            let when: Vec<Option<f64>> = (0..4).map(|seat| game.minds[seat].as_ref().map(|mind| mind.decides_at())).collect();
            game.leave(0);
            // Each thinks again on the ice as it is now, without them (no
            // move sized to stop against them, no guess of what they will
            // do), and makes up its mind when it meant to.
            for (seat, meant) in when.iter().enumerate().skip(1) {
                let mind = game.minds[seat].as_ref().expect("the computer's");
                assert!(!mind.decided(), "seed {seed}, seat {seat}");
                assert_eq!(mind.thinking_on().map(|stand| stand[0]), Some(None), "seed {seed}, seat {seat}");
                assert_eq!(Some(mind.decides_at()), *meant, "seed {seed}, seat {seat}");
            }
            // And the aim, nobody's person left on the ice, is over once the
            // last of them has made up its mind.
            let last = when.iter().flatten().copied().fold(f64::MIN, f64::max);
            assert_eq!(game.round().until, last, "seed {seed}");
        }
    }

    #[test]
    fn a_round_goes_whole_and_small() {
        // As long as one gets: four seats, every one of them something.
        let round = KnockRound {
            turn: LAST_TURN,
            stage: KnockStage::Slide,
            since: CLOCK + 600.123,
            until: CLOCK + 605.456,
            radius: 0.75,
            next_radius: 0.0,
            start: vec![Some((-3.21, 4.56)); 4],
            launches: vec![(-12.3, 4.5); 4],
            ready: vec![true; 4],
            out: vec![Some(LAST_TURN); 4],
        };
        let bytes = postcard::to_allocvec(&round).unwrap();
        assert!(bytes.len() <= 110, "{} bytes", bytes.len());
        assert_eq!(postcard::from_bytes::<KnockRound>(&bytes).unwrap(), round);
        // And as everyone aims, less: nobody's launch.
        let mut game = Game::new(4, &[true, false, false, false], 1);
        game.begin(CLOCK);
        let bytes = postcard::to_allocvec(&game.round()).unwrap();
        assert!(bytes.len() <= 76, "{} bytes", bytes.len());
        // What is asked and told, as small.
        for ask in [Ask::Aim { x: -0.6, z: 0.8, power: 10 }, Ask::Ready] {
            let bytes = postcard::to_allocvec(&crate::Ask::KnockOff(ask.clone())).unwrap();
            assert!(bytes.len() <= 12, "{ask:?} is {} bytes", bytes.len());
            let crate::Ask::KnockOff(back) = postcard::from_bytes::<crate::Ask>(&bytes).unwrap() else { panic!("not Knock Off's") };
            assert_eq!(back, ask);
        }
        let told = Tell::Aimed { x: 0.6, z: -0.8, power: 3 };
        let bytes = postcard::to_allocvec(&crate::Tell::KnockOff(told.clone())).unwrap();
        let crate::Tell::KnockOff(back) = postcard::from_bytes::<crate::Tell>(&bytes).unwrap() else { panic!("not Knock Off's") };
        assert_eq!(back, told);
    }

    #[test]
    fn a_device_playing_the_slide_from_the_round_comes_out_where_the_game_does() {
        for seed in 20..30 {
            let mut game = Game::new(4, &[false; 4], seed);
            game.begin(CLOCK);
            let mut slides = 0;
            for _ in 0..100_000 {
                if game.outcome().is_some() {
                    break;
                }
                let before = game.stage();
                tick(&mut game, 1.0 / 30.0);
                if before != KnockStage::Slide && game.stage() == KnockStage::Slide {
                    // What a device is sent, and plays by the clock as its
                    // frames fall, a little behind.
                    let round = game.round();
                    let start: Vec<Option<Vec2>> = round.start.iter().map(|at| at.map(|(x, z)| Vec2::new(x, z))).collect();
                    let launches: Vec<Vec2> = round.launches.iter().map(|&(x, z)| Vec2::new(x, z)).collect();
                    let mut device = Slide::new(&start, &launches, round.radius);
                    let mut clock = round.since + 0.013;
                    while clock < round.until {
                        clock += 1.0 / 57.0;
                        device.step_to((clock - round.since) as f32);
                    }
                    // Where the game has everyone once it is over: about to
                    // break the ice, or, the slide having left one or nobody,
                    // the game over.
                    run_to(&mut game, KnockStage::Break, 1.0 / 30.0);
                    let settled = game.round();
                    for seat in 0..4 {
                        let theirs = device.at(seat).filter(|_| device.over_edge(seat).is_none()).map(|at| (at.x, at.z));
                        if round.out[seat].is_none() {
                            assert_eq!(theirs, settled.start[seat], "seed {seed}, seat {seat}");
                        }
                    }
                    slides += 1;
                }
            }
            assert!(slides >= 1, "seed {seed}");
        }
    }

    /// A game of four of the computer's players, as it went.
    #[derive(Clone, Debug, Default)]
    struct Told {
        seed: u64,
        turns: u16,
        kinds: Vec<&'static str>,
        first: Vec<usize>,
        /// Every fall: who, in which turn, who hit them last, and whether
        /// the ice broke under them.
        fell: Vec<(usize, u16, Option<usize>, bool)>,
        /// How many turns each seat aimed in.
        turns_in: Vec<u16>,
        /// How long the computer took thinking, all told.
        thought: Duration,
        events: Vec<Event>,
        /// Each turn's round as it was revealed (everyone's start and
        /// launch, to the bit, and when it was), and the round once it was
        /// over.
        reveals: Vec<KnockRound>,
        ended: Option<KnockRound>,
    }

    /// A game of four of the computer's players from `seed`, stepped `dt`
    /// at a time.
    fn told(seed: u64, dt: f64) -> Told {
        told_by(seed, |step| CLOCK + f64::from(step) * dt)
    }

    /// A game of four of the computer's players from `seed`, stepped at
    /// the times `clock` gives for its steps, counted from 1. After every
    /// step, the stage it is at began by that time and ends after it: the
    /// game is where the clock is.
    fn told_by(seed: u64, mut clock: impl FnMut(u32) -> f64) -> Told {
        let mut game = Game::new(4, &[false; 4], seed);
        let mut events = game.begin(CLOCK);
        let mut thought = Duration::ZERO;
        let mut reveals = Vec::new();
        for step in 1..1_000_000 {
            if game.outcome().is_some() {
                break;
            }
            let aiming = game.stage() == KnockStage::Aim;
            let now = clock(step);
            let began = Instant::now();
            let stepped = game.step(now);
            if aiming {
                thought += began.elapsed();
            }
            let round = game.round();
            assert!(
                round.stage == KnockStage::Over || (round.since <= now && now < round.until),
                "seed {seed}: at {} the round is {:?} from {} to {}",
                now - CLOCK,
                round.stage,
                round.since - CLOCK,
                round.until - CLOCK
            );
            if stepped.contains(&Event::Reveal) {
                reveals.push(round);
            }
            events.extend(stepped);
        }
        assert!(game.outcome().is_some(), "seed {seed} never ended");
        let fell = events
            .iter()
            .filter_map(|event| match *event {
                Event::Fell { seat, by, broke } => Some((seat, game.round().out[seat].unwrap(), by, broke)),
                _ => None,
            })
            .collect();
        let first = match events.last() {
            Some(Event::Over { first }) => first.clone(),
            other => panic!("seed {seed} ended with {other:?}"),
        };
        Told {
            seed,
            turns: game.turn(),
            kinds: (0..4).map(|seat| game.persona(seat).unwrap()).collect(),
            first,
            fell,
            turns_in: (0..4).map(|seat| game.round().out[seat].unwrap_or(game.turn())).collect(),
            thought,
            events,
            reveals,
            ended: Some(game.round()),
        }
    }

    /// Each seat's move at a reveal, in words: where it stood (how far out),
    /// and how hard it went where (the middle, at someone, or elsewhere).
    fn moves_of(round: &KnockRound, kinds: &[&str]) -> String {
        let start: Vec<Option<Vec2>> = round.start.iter().map(|at| at.map(|(x, z)| Vec2::new(x, z))).collect();
        (0..start.len())
            .filter_map(|seat| {
                let here = start[seat]?;
                let launched = Vec2::new(round.launches[seat].0, round.launches[seat].1);
                let power = launched.length_squared() / (2.0 * ICE * METRES_A_POWER);
                let toward = match launched.try_normalize() {
                    None => String::from("still"),
                    Some(way) => {
                        let at = (0..start.len())
                            .filter(|&other| other != seat)
                            .filter_map(|other| start[other].map(|there| (other, (there - here).normalize_or_zero().dot(way))))
                            .max_by(|a, b| a.1.total_cmp(&b.1));
                        let middle = (-here).normalize_or_zero().dot(way);
                        match at {
                            Some((other, close)) if close > 0.97 && close >= middle => format!("at {other}"),
                            _ if middle > 0.97 => String::from("middle"),
                            _ => format!("{:+.0}deg", way.y.atan2(way.x).to_degrees()),
                        }
                    }
                };
                Some(format!("{seat}:{}@{:.1}m p{power:.0} {toward}", kinds[seat], here.length()))
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// What many games come to, by the numbers: how long they go, who wins
    /// by kind, how often one goes in by its own move alone.
    #[derive(Debug, Default)]
    struct Tally {
        games: usize,
        turns: Vec<u16>,
        shared_first: usize,
        /// By kind, in `PERSONAS`' order: games played, won outright, won
        /// shared, turns aimed in, slid into the sea by its own move alone
        /// (nobody hit it in that slide: a brush is no hit), gone in at all,
        /// and others it put in.
        played: [u32; 6],
        won: [u32; 6],
        shared: [u32; 6],
        aimed: [u32; 6],
        alone: [u32; 6],
        gone: [u32; 6],
        shoved: [u32; 6],
        broke: u32,
        slid: u32,
        /// How many went in in each turn, from the first, all games told.
        by_turn: [u32; LAST_TURN as usize],
        thought: Duration,
    }

    fn tally(games: &[Told]) -> Tally {
        let kind = |name: &str| PERSONAS.iter().position(|persona| persona.name == name).unwrap();
        let mut tally = Tally { games: games.len(), ..default() };
        for game in games {
            tally.turns.push(game.turns);
            tally.thought += game.thought;
            if game.first.len() > 1 {
                tally.shared_first += 1;
            }
            for seat in 0..4 {
                let of = kind(game.kinds[seat]);
                tally.played[of] += 1;
                tally.aimed[of] += u32::from(game.turns_in[seat]);
                if game.first.contains(&seat) {
                    if game.first.len() == 1 {
                        tally.won[of] += 1;
                    } else {
                        tally.shared[of] += 1;
                    }
                }
            }
            for &(seat, turn, by, broke) in &game.fell {
                let of = kind(game.kinds[seat]);
                tally.gone[of] += 1;
                tally.by_turn[usize::from(turn.clamp(1, LAST_TURN) - 1)] += 1;
                if by.is_none() && !broke {
                    tally.alone[of] += 1;
                }
                if let Some(by) = by {
                    tally.shoved[kind(game.kinds[by])] += 1;
                }
                if broke {
                    tally.broke += 1;
                } else {
                    tally.slid += 1;
                }
            }
        }
        tally.turns.sort_unstable();
        tally
    }

    fn print(tally: &Tally) {
        let median = tally.turns[tally.turns.len() / 2];
        let mean = tally.turns.iter().map(|&turns| f32::from(turns)).sum::<f32>() / tally.games as f32;
        let mut by_turns = [0u32; LAST_TURN as usize + 1];
        for &turns in &tally.turns {
            by_turns[usize::from(turns)] += 1;
        }
        println!(
            "{} games: over in {mean:.1} turns on average, {median} the median, by turns {:?}; first shared in {} ({:.0}%); in the sea {} by the slide, {} by the ice breaking; the computer thought {:.2} ms a turn each",
            tally.games,
            &by_turns[1..],
            tally.shared_first,
            100.0 * tally.shared_first as f32 / tally.games as f32,
            tally.slid,
            tally.broke,
            tally.thought.as_secs_f64() * 1000.0 / tally.aimed.iter().sum::<u32>().max(1) as f64,
        );
        println!("in the sea by turn, all games told: {:?}", tally.by_turn);
        println!("kind     | games | won  (rate) | shared | turns | slid in alone (of turns) | in at all | put in");
        for (of, persona) in PERSONAS.iter().enumerate() {
            println!(
                "{:<8} | {:>5} | {:>4} ({:>3.0}%) | {:>6} | {:>5} | {:>4} ({:>4.1}%)             | {:>9} | {:>6}",
                persona.name,
                tally.played[of],
                tally.won[of],
                100.0 * tally.won[of] as f32 / tally.played[of].max(1) as f32,
                tally.shared[of],
                tally.aimed[of],
                tally.alone[of],
                100.0 * tally.alone[of] as f32 / tally.aimed[of].max(1) as f32,
                tally.gone[of],
                tally.shoved[of],
            );
        }
    }

    #[test]
    fn the_computer_plays_whole_games_that_end_close_and_varied() {
        // Four hundred games: a shared first comes about one game in a
        // hundred, and in two hundred it may well not come at all.
        let games: Vec<Told> = (0..400).map(|seed| told(seed, 1.0 / 30.0)).collect();
        let tally = tally(&games);
        print(&tally);
        // Every game ends, by the last turn there is.
        for game in &games {
            assert!(game.turns >= 1 && game.turns <= LAST_TURN, "seed {}: {} turns", game.seed, game.turns);
            assert!(!game.first.is_empty(), "seed {}", game.seed);
        }
        // Not too short and not too long: half of them by about the middle
        // of the ice's breaking.
        let median = tally.turns[tally.turns.len() / 2];
        assert!((5..=9).contains(&median), "median {median} turns");
        // Every kind wins some, and none more than half of the games it is in.
        for (of, persona) in PERSONAS.iter().enumerate() {
            assert!(tally.won[of] > 0, "{} never won", persona.name);
            assert!(2 * tally.won[of] <= tally.played[of], "{} won {} of {}", persona.name, tally.won[of], tally.played[of]);
        }
        // Sliding into the sea by your own move alone, nobody having hit
        // you, is a mistake now and then, not a habit: for every kind.
        for (of, persona) in PERSONAS.iter().enumerate() {
            assert!(
                tally.alone[of] * 100 < 15 * tally.aimed[of],
                "{}: {} of {} turns ended in the sea alone",
                persona.name,
                tally.alone[of],
                tally.aimed[of]
            );
        }
        // A shared first happens, but seldom.
        assert!(tally.shared_first > 0, "first never shared");
        assert!(tally.shared_first * 10 < tally.games, "first shared in {} of {}", tally.shared_first, tally.games);
        // Both ways in happen.
        assert!(tally.slid > 0 && tally.broke > 0, "{tally:?}");
    }

    #[test]
    fn the_same_seed_is_the_same_game_however_its_steps_fall() {
        for seed in [3, 77, 150] {
            let (one, other, finer) = (told(seed, 1.0 / 30.0), told(seed, 1.0 / 30.0), told(seed, 1.0 / 120.0));
            assert_eq!(one.events, other.events, "seed {seed}");
            assert_eq!(one.events, finer.events, "seed {seed}, at 120 a second");
            // Every slide from the same start and launches, to the bit, at
            // the same moments; and the same where it ended.
            assert!(!one.reveals.is_empty(), "seed {seed}");
            assert_eq!(one.reveals, other.reveals, "seed {seed}");
            assert_eq!(one.reveals, finer.reveals, "seed {seed}, at 120 a second");
            assert_eq!(one.ended, finer.ended, "seed {seed}, at 120 a second");
        }
    }

    #[test]
    fn a_game_stepped_through_stalls_is_where_the_clock_is_and_comes_out_the_same() {
        // Frames that now and then stall for seconds, as a server's can, or
        // a phone's put away and brought back: the game goes by the clock it
        // is told, never by how long a frame was, so after every step it is
        // at the stage the clock is in (`told_by` checks), and it comes out
        // as it does stepped evenly.
        for seed in [5, 41, 160] {
            let mut dice = Dice::new(seed + 7);
            let mut at = CLOCK;
            let stalled = told_by(seed, |_| {
                at += if dice.roll() < 0.03 { f64::from(dice.between(0.3, 40.0)) } else { 1.0 / 30.0 };
                at
            });
            let even = told(seed, 1.0 / 30.0);
            assert_eq!(stalled.events, even.events, "seed {seed}");
            assert_eq!(stalled.ended, even.ended, "seed {seed}");
        }
        // A stall longer than a whole game: over in that one step, as it
        // would have come out.
        let mut game = Game::new(4, &[false; 4], 41);
        let mut events = game.begin(CLOCK);
        events.extend(game.step(CLOCK + 3600.0));
        assert!(game.outcome().is_some());
        let even = told(41, 1.0 / 30.0);
        assert_eq!(events, even.events);
        assert_eq!(Some(game.round()), even.ended);
        // A clock put back a little holds the game where it was, and one
        // that is not a number is not heard.
        let mut game = Game::new(4, &[false; 4], 2);
        game.begin(CLOCK);
        game.step(CLOCK + 2.0);
        let round = game.round();
        game.step(CLOCK + 1.5);
        game.step(f64::NAN);
        game.step(f64::INFINITY);
        assert_eq!(game.now, CLOCK + 2.0);
        assert_eq!(game.round(), round);
    }

    /// More games, for tuning by: `cargo test -p roundtown_net knock_off::tests::many
    /// -- --ignored --nocapture`, two hundred games, or as many as `RT_GAMES` says, from
    /// the seed 1000, or from the one `RT_FROM` says (one game over again: `RT_FROM=1006
    /// RT_GAMES=1`, which prints how it went).
    #[test]
    #[ignore]
    fn many_games_of_the_computer() {
        let count: u64 = std::env::var("RT_GAMES").ok().and_then(|count| count.parse().ok()).unwrap_or(200);
        let from: u64 = std::env::var("RT_FROM").ok().and_then(|from| from.parse().ok()).unwrap_or(1000);
        let games: Vec<Told> = (from..from + count).map(|seed| told(seed, 1.0 / 30.0)).collect();
        if count <= 6 {
            for game in &games {
                println!("seed {}: {:?}", game.seed, game.kinds);
                let mut reveals = game.reveals.iter();
                for event in &game.events {
                    if *event == Event::Reveal
                        && let Some(round) = reveals.next()
                    {
                        println!("  turn {} on {} m: {}", round.turn, round.radius, moves_of(round, &game.kinds));
                    } else if !matches!(event, Event::Slide | Event::Aim { .. }) {
                        println!("  {event:?}");
                    }
                }
            }
        }
        println!("seed | turns | first          | in the sea (seat, turn, by, broke)");
        for game in &games {
            println!(
                "{:>4} | {:>5} | {:<14} | {:?}",
                game.seed,
                game.turns,
                game.first.iter().map(|&seat| game.kinds[seat]).collect::<Vec<_>>().join(","),
                game.fell,
            );
        }
        print(&tally(&games));
    }
}
