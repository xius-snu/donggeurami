//! The computer's players in Bridge Race (since 2026-11-18; Hajun: "for
//! now, i want 2 ai bots to be on the same team, and the other team is real
//! players either 1~2 player"): one brain for the server's online and the
//! device's in a game played offline, as the rules are one
//! ([`super::Game`]). It plays by them: the metres it lays are the ones it
//! is given, the sticks it throws the ones it holds.
//!
//! Bridge Wars' brain (`bots::Squad`) is that game's, its map and its four
//! teams; this one is the race's own, round the same machinery: a body
//! moved over the map as a body moves ([`Body`]), the map with every
//! bridge on it ([`Nav`]), and seeded dice ([`Dice`]). What it walks and
//! fights by is copied from Bridge Wars' (its way finding, its laying, where
//! it stands to punch someone off the land, getting out from under a
//! stick), and what it sets out to do is the race's:
//!
//! * **Its team's way** to the golden island, one [`Span`] from its island
//!   to the golden one, thrown for the game, straight or bent, and shared by
//!   both of a team. Whoever of the team has metres goes to the end of it
//!   and lays on, a plank at a time, the first gap in it it can get to: the
//!   end, or a hole a stick has blown behind it. With none left it waits at
//!   the end for the next grant, or goes back to keep where its bridge
//!   begins, by who it is.
//! * **The golden island**, once it can get there: over, to somewhere on
//!   it, to stand; and to knock off whoever of the other team comes. A bold
//!   one a jump short of it jumps. One cut off from its team's way, left
//!   on the other team's bridge by a fight or across a hole it cannot jump,
//!   lays a short way of its own onto the golden island from the end of
//!   whatever bridge it can get to near it, or jumps from there.
//! * **Whoever of the other team comes near** it punches, standing where its
//!   punch throws them off the land, as Bridge Wars' do; and some go out to
//!   meet them, on its bridge or across onto theirs.
//! * **A stick of dynamite** it throws where it takes the most of the other
//!   team's bridge, or the planks from under one of them over the void;
//!   never its own team's planks, nor where one of its own would fall. When,
//!   is its kind's.
//!
//! **Nothing is a definite if-then** (Hajun: "if some thing becomes a
//! definite if-then, then its not a real human, but a program"): each is one
//! of [`KINDS`], each a little more or less so ([`Leanings`]), and every
//! threshold it goes by is a spread, thrown again each time.

use std::f32::consts::{PI, TAU};

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use crate::bots::{Body, Dice, Order, Seen, Stick};
use crate::bridge_wars::{
    BOT_PUNCH_EVERY, DYNAMITE_FURTHEST, DYNAMITE_NEAREST, DYNAMITE_RADIUS, REACH, RUN_SPEED, builder_speed,
};
use crate::nav::{BODY, Gap, Grid, Nav, PLANK_HALF_WIDTH, Region, Span};
use crate::{Kit, Upgrades, Utility};

use super::{GOLDEN, ISLANDS, TEAMS, team};

/// How high a rise a body walks up, in metres (Bridge Wars' brain's).
const STEP: f32 = 0.51;
/// This near a point of its way a bot has come to it, in metres.
const ARRIVED: f32 = 0.35;
/// How far ahead of its feet a bot's bridge is laid, in metres, at the
/// least (Bridge Wars' own).
const LAID_AHEAD: f32 = 1.6;
/// How near a bot stands off whoever it fights, as a share of its reach.
const PRESS_IN: f32 = 0.7;
/// How far a body knocked off an edge looks for ground to steer back to,
/// in metres.
const REACH_BACK: f32 = 8.0;
/// How far whoever a punch lands on is carried by it, standing and on the
/// run, in metres: what a bot goes by in choosing where to stand
/// ([`flung`]).
const CARRY: [f32; 2] = [3.0, 4.5];
/// How far out from whoever it fights a bot stands to punch them, and how
/// far back it steps between punches when it pokes at them, in metres.
const RING: f32 = 1.45;
const POKE_BACK: f32 = 3.6;
/// How much a bridge may slant where it leaves the land and where it meets
/// it, and how many times as long as the straight line between its ends it
/// may be, bends and all ([`Crew::bends`]): Bridge Wars' own.
const MOST_SLANT: f32 = 0.45;
const MOST_BENT: f32 = 1.05;
/// How long a body is in the air in a jump of its own, in seconds, and how
/// much of the ground it would cross in that while a bot keeps in hand over
/// a gap, in metres: a gap of 6.1 m at a run (Bridge Wars' own).
const JUMP_SECS: f32 = 0.7;
const JUMP_SPARE: f32 = 0.9;
/// How often a bot on its way somewhere looks along the straight walk to
/// the next point of its way, in seconds.
const LOOK_EVERY: f32 = 0.25;
/// How long between a bot's thoughts of what to do, in seconds, the least
/// and the most, thrown again each time; and the bonus the goal it is on
/// has over the rest, of a score of about 1.
const THINK_EVERY: (f32, f32) = (0.7, 1.3);
const STICKY: f32 = 0.15;
/// How long after coming into reach of someone a bot's first punch comes,
/// and how long between its punches after, in seconds (Bridge Wars' own).
const WINDUP: f32 = 0.3;
const PUNCH_EVERY: f32 = BOT_PUNCH_EVERY;
/// How long a knock carries a bot before it answers, and how much of its
/// pace it then steers by in the air: between these two, thrown again at
/// every knock (Bridge Wars' own).
const REEL: (f32, f32) = (0.06, 0.36);
const GRIP: (f32, f32) = (0.5, 1.0);
/// How near someone of the other team has to come for a bot minding its
/// own business to go for them, in metres, for a bot of a fighting kind
/// as most are; and how near on top of it they have to be for one laying
/// its bridge, or one out for something else, to turn on them.
const AGGRO: f32 = 6.5;
const ON_TOP_OF_IT: f32 = 2.8;
/// How much further off than it would go for someone a bot keeps after one
/// it is on already, as so many times as far.
const KEEPS_ON: f32 = 1.35;
/// How far either side of how near it lets people come it wanders from one
/// while to the next, and how long each while is, in seconds.
const KEEN: (f32, f32) = (0.75, 1.3);
const KEEN_LASTS: (f32, f32) = (4.0, 11.0);
/// How long a bot takes to wake up to a stick of dynamite thrown at it, in
/// seconds from the throw, the soonest and the latest, for the worst of
/// them at a fight, and how much sooner the best; how far past where its
/// blast reaches it gets out of its way, and how far short of it one waits
/// that was walking into it, in metres (Bridge Wars' own).
const STICK_WAKES: (f32, f32) = (0.75, 2.0);
const STICK_WAKES_SOONER: (f32, f32) = (0.4, 1.0);
const CLEAR_OF_BLAST: f32 = 1.0;
const SHORT_OF_BLAST: f32 = 3.0;
/// How often a bot with a stick of dynamite on it looks about for
/// somewhere worth throwing it from where it stands, in seconds, the least
/// and the most.
const STICK_LOOKS: (f32, f32) = (0.25, 0.6);
/// How long a bot takes to throw at something worth a stick once it has
/// seen it, in seconds, the least and the most, for one as good in a fight
/// as most, quicker the better: the moment anyone takes to see a chance
/// and take it.
const SIGHTED: (f32, f32) = (0.3, 1.8);
/// How much one plank of theirs taken counts toward a stick being worth
/// it, and one of them standing over the void on what it takes, who falls,
/// and one only knocked by it: so many planks' worth.
const WORTH_FALL: f32 = 5.0;
const WORTH_KNOCK: f32 = 1.0;
/// How far back from the end of its team's way a bot waits for metres, in
/// planks, and how far in from where its bridge leaves its island one
/// keeping it stands, in metres, and to the side: between these, thrown
/// each time it goes there.
const WAIT_BACK: (f32, f32) = (0.6, 3.0);
const GUARD_IN: (f32, f32) = (1.5, 4.5);
const GUARD_ASIDE: (f32, f32) = (-2.5, 2.5);
/// How far from where its bridge leaves its island a bot can be and go
/// back to keep it as readily as from there, in metres, and how much less
/// it wants to for each metre further: it keeps its island while its
/// bridge is short, and once it is out near the end of a long one it stays
/// out, rather than walk the length of it and back again for every grant.
const GUARD_NEAR: f32 = 12.0;
const GUARD_WALK: f32 = 0.012;
/// Where on the golden island a bot holding it stands, in metres from its
/// middle, and how long it stands there before it moves to another such
/// place, in seconds: inside its ring, well in from its edge, as people
/// would.
const HOLD_OUT: (f32, f32) = (0.5, 4.8);
const HOLD_STAYS: (f32, f32) = (3.0, 10.0);
/// How far from the end of its team's way, or from where it stands, the
/// golden island has to be for a bot to jump for it, in metres: a run at
/// the edge, and a jump.
const RUN_UP: f32 = 8.0;

/// One way of playing the race (mine): what each of the computer's players
/// is like, each a little more or less so ([`Leanings`]). Every number is a
/// leaning, from about 0 to about 1.5, for the dice to go by.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Kind {
    pub name: &'static str,
    /// How keen it is to be the one at the end of the way, laying.
    pub lays: f32,
    /// With nothing to lay, how much it waits at the end of the way for
    /// more, rather than go back to keep where its bridge begins.
    pub waits: f32,
    /// How near it lets someone of the other team come before it goes for
    /// them, as so many times as near as most.
    pub fights: f32,
    /// How keen it is to go out to meet someone of the other team, on its
    /// bridge or on theirs.
    pub meets: f32,
    /// How keen it is to throw a stick of dynamite, and how little it takes
    /// for one to be worth it.
    pub sticks: f32,
    /// How bold it is, how much it minds being thrown off the land, and how
    /// much it goes for throwing others off: where it stands to fight.
    pub bold: f32,
    pub minds: f32,
    pub shoves: f32,
    /// How good it is in a fight, from 0 to 1: the quicker its punches and
    /// the surer its feet.
    pub pvp: f32,
    /// How widely it weaves as it walks, and how much it jumps about.
    pub weave: f32,
    pub bouncy: f32,
    /// How ready it is to jump the last of the void onto the golden island,
    /// or across to another bridge, rather than lay the planks.
    pub leaps: f32,
}

/// Every kind there is (mine): a builder, keen to lay and slow to fight; a
/// bruiser, for the fight; a saboteur, for the dynamite; a steady one, a
/// little of each and careful; a rusher, bold, out in front and jumping for
/// it; and a keeper, who keeps where its bridge begins and goes out to meet
/// whoever comes.
pub const KINDS: [Kind; 6] = [
    Kind {
        name: "builder",
        lays: 1.0,
        waits: 0.85,
        fights: 0.75,
        meets: 0.2,
        sticks: 0.45,
        bold: 0.9,
        minds: 1.2,
        shoves: 0.9,
        pvp: 0.35,
        weave: 0.8,
        bouncy: 0.3,
        leaps: 0.35,
    },
    Kind {
        name: "bruiser",
        lays: 0.35,
        waits: 0.55,
        fights: 1.4,
        meets: 0.95,
        sticks: 0.3,
        bold: 1.3,
        minds: 0.8,
        shoves: 1.35,
        pvp: 0.85,
        weave: 1.0,
        bouncy: 0.7,
        leaps: 0.6,
    },
    Kind {
        name: "saboteur",
        lays: 0.55,
        waits: 0.65,
        fights: 0.9,
        meets: 0.4,
        sticks: 1.25,
        bold: 1.0,
        minds: 1.0,
        shoves: 1.1,
        pvp: 0.5,
        weave: 1.1,
        bouncy: 0.5,
        leaps: 0.45,
    },
    Kind {
        name: "steady",
        lays: 0.7,
        waits: 0.7,
        fights: 1.0,
        meets: 0.5,
        sticks: 0.6,
        bold: 1.0,
        minds: 1.35,
        shoves: 1.0,
        pvp: 0.6,
        weave: 0.6,
        bouncy: 0.2,
        leaps: 0.25,
    },
    Kind {
        name: "rusher",
        lays: 0.85,
        waits: 0.95,
        fights: 1.1,
        meets: 0.65,
        sticks: 0.7,
        bold: 1.4,
        minds: 0.7,
        shoves: 1.0,
        pvp: 0.6,
        weave: 1.3,
        bouncy: 0.9,
        leaps: 1.0,
    },
    Kind {
        name: "keeper",
        lays: 0.45,
        waits: 0.25,
        fights: 1.25,
        meets: 0.75,
        sticks: 0.65,
        bold: 0.95,
        minds: 1.2,
        shoves: 1.2,
        pvp: 0.7,
        weave: 0.7,
        bouncy: 0.3,
        leaps: 0.3,
    },
];

/// How one of the computer's players is, thrown as it takes its seat about
/// what its kind has, so that no two of a kind are quite alike: each
/// leaning its [`Kind`]'s of that name, a little more or less.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Leanings {
    pub lays: f32,
    pub waits: f32,
    pub fights: f32,
    pub meets: f32,
    pub sticks: f32,
    pub bold: f32,
    pub minds: f32,
    pub shoves: f32,
    pub pvp: f32,
    pub weave: f32,
    pub bouncy: f32,
    pub leaps: f32,
}

impl Leanings {
    /// `kind`, each leaning a little more or less so, as a bell rings it.
    fn of(kind: &Kind, dice: &mut Dice) -> Self {
        let mut near = |leaning: f32| (leaning * (1.0 + 0.12 * dice.bell())).max(0.0);
        Self {
            lays: near(kind.lays),
            waits: near(kind.waits).min(1.0),
            fights: near(kind.fights),
            meets: near(kind.meets).min(1.0),
            sticks: near(kind.sticks),
            bold: near(kind.bold),
            minds: near(kind.minds),
            shoves: near(kind.shoves),
            pvp: near(kind.pvp).min(1.0),
            weave: near(kind.weave),
            bouncy: near(kind.bouncy),
            leaps: near(kind.leaps),
        }
    }
}

/// What one of the computer's players has set out to do, as it last
/// thought.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Goal {
    /// Laying its team's way: to the first gap in it it can get to, and on
    /// out along it.
    Lay,
    /// At the end of its team's way, for the next grant.
    Wait,
    /// On its island, by where its team's way leaves it.
    Guard,
    /// Over onto the golden island, by its team's way or a jump, and
    /// standing there.
    Hold,
    /// After whoever of the other team is in this seat.
    Hunt(usize),
    /// To where a stick of dynamite of its reaches the other team's bridge.
    Sabotage,
}

/// A run of planks it is laying, by the rungs of `span`: its team's way, or
/// a short way of its own from the end of a bridge onto the golden island
/// ([`Crew::next_lay`]); its own bridge numbered `chain`, from the span's
/// rung `from` (its first rung, over ground already) to its rung `to`; and
/// the span's rung it lays next.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Laying {
    span: Span,
    chain: u16,
    from: usize,
    to: usize,
    next: usize,
}

impl Laying {
    /// The span's plank it lays next: the one its next rung ends, or, with
    /// nothing laid yet, the run's first.
    fn coming(&self) -> usize {
        if self.next == self.from { self.from } else { self.next - 1 }
    }

    /// The middle of the span's plank numbered `plank`.
    fn middle(&self, plank: usize) -> Vec2 {
        (self.span.rung(plank) + self.span.rung(plank + 1)) * 0.5
    }

    /// Where it takes the laying up from: just short of the run's first
    /// rung, with nothing laid yet; on the plank it laid last, after.
    fn resumes_at(&self) -> Vec2 {
        if self.next < self.from + 2 {
            let planks = self.span.planks() as f32;
            self.span.point(((self.from as f32 - 0.4) / planks).max(0.0))
        } else {
            self.middle(self.next - 2)
        }
    }
}

/// A run of planks of a team's way missing, by its planks: `from`, the
/// first missing, up to `to`, the first that stands again, or the end.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Run {
    from: usize,
    to: usize,
}

/// A jump a bot is making over a gap: where it means to come down, how
/// fast it runs at it, until when it keeps to it whatever comes of it, and
/// whether it has left the ground yet (Bridge Wars' own).
#[derive(Clone, Copy, PartialEq, Debug)]
struct Leap {
    to: Vec2,
    pace: f32,
    until: f32,
    aloft: bool,
}

/// A fight with one foe: where it means to stand and until when it keeps
/// to that, until when it is stepping back from a punch, and whether it
/// comes in again for its next punch on the run.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Duel {
    foe: usize,
    spot: Option<Vec2>,
    spot_until: f32,
    back_until: f32,
    lunge: bool,
}

/// One team's way to the golden island: the span, and for each of its
/// planks, between its rung of that number and the next, whether
/// something stands there now, as of the map's [`Nav::version`].
#[derive(Clone, Debug)]
struct Way {
    span: Span,
    rungs: Vec<Vec2>,
    standing: Vec<bool>,
    version: Option<u32>,
}

impl Way {
    fn new(span: Span) -> Self {
        let planks = span.planks();
        Self {
            span,
            rungs: (0..=planks).map(|rung| span.rung(rung)).collect(),
            standing: vec![false; planks],
            version: None,
        }
    }

    fn planks(&self) -> usize {
        self.standing.len()
    }

    /// The middle of its plank numbered `plank`.
    fn middle(&self, plank: usize) -> Vec2 {
        (self.rungs[plank] + self.rungs[plank + 1]) * 0.5
    }

    /// Where it is `back` planks short of its rung numbered `rung`, along
    /// it: no further back than its first rung.
    fn back_from(&self, rung: usize, back: f32) -> Vec2 {
        let share = ((rung as f32 - back) / self.planks() as f32).clamp(0.0, 1.0);
        self.span.point(share)
    }

    /// Looks again at which of its planks stand, if anything has been laid
    /// or taken since it last did.
    fn refresh(&mut self, nav: &Nav) {
        if self.version == Some(nav.version) {
            return;
        }
        self.version = Some(nav.version);
        for plank in 0..self.planks() {
            self.standing[plank] = nav.on_plank(self.middle(plank));
        }
    }
}

/// The game as the computer's players see it, for one frame.
pub struct View<'a> {
    /// Seconds of play.
    pub elapsed: f32,
    pub nav: &'a Nav,
    /// Everyone, by seat.
    pub seats: &'a [Seen],
    /// What everyone has on their buttons, by seat: their sticks.
    pub kits: &'a [Kit],
    /// Every stick of dynamite in the air or lying, not yet blown.
    pub sticks: &'a [Stick],
    /// How far each team's bridge has got toward the golden island, as a
    /// share of the way across ([`super::RaceRound::across`]): what tells a
    /// team that it is behind.
    pub across: [f32; TEAMS],
}

/// One of the computer's players.
#[derive(Clone, Debug)]
pub struct Racer {
    /// Its body: where it is, which way it faces, and what carries it.
    pub body: Body,
    seat: usize,
    side: u8,
    kind: &'static Kind,
    lean: Leanings,
    /// How many metres of bridge it has left to lay, how many bridges it has
    /// begun, and the run of its team's way it is laying.
    metres: f32,
    chains: u16,
    laying: Option<Laying>,
    goal: Goal,
    think_at: f32,
    /// Its way to where it is going: the points still ahead, where they
    /// lead, the map as it was, and when it was found.
    path: Vec<Vec2>,
    path_to: Option<Vec2>,
    path_version: u32,
    path_cuts: u32,
    path_at: f32,
    /// Where it was, and when, for telling that it is getting nowhere; when
    /// it last looked along its way, and was last on its way anywhere; how
    /// often it has got nowhere, and where it last did, heading where; and
    /// how many of those times it could not take a step at all.
    progress: (Vec2, f32),
    looked_at: f32,
    walked_at: f32,
    stalls: u32,
    stalled: Option<(Vec2, Vec2)>,
    pinned: u32,
    weave: f32,
    /// A hole in a bridge it means to jump on its way, and a jump it is
    /// making.
    cross: Option<Gap>,
    leaping: Option<Leap>,
    /// Whom it fights, since when they have been in its reach, when it may
    /// next punch, and how it goes about them.
    foe: Option<usize>,
    in_reach_since: Option<f32>,
    punch_at: f32,
    windup: f32,
    duel: Option<Duel>,
    /// How near it lets people come just now, as a share of its own, and
    /// until when that holds.
    keen: f32,
    keen_until: f32,
    /// A knock that has just landed, to be counted from its next frame;
    /// until when it carries it; and how much of its pace it steers by in
    /// the air this time.
    reel: Option<f32>,
    reel_until: f32,
    grip: f32,
    /// A stick of dynamite it has in mind, where and since when it woke up
    /// to it; when it next looks about for somewhere to throw its own, and
    /// how much a throw has to be worth to it then.
    alarm: Option<(Vec2, f32)>,
    look_at: f32,
    worth: f32,
    /// What it has seen worth throwing at, and when it will have taken
    /// that in and throws ([`SIGHTED`]).
    sighted: Option<(Vec2, f32)>,
    /// Where it means to throw from and at, out on a sabotage.
    aim: Option<(Vec2, Vec2)>,
    /// Where it stands waiting, keeping its island, or on the golden
    /// island, and until when: thrown each time.
    wait_back: f32,
    guard_at: Option<Vec2>,
    hold_at: Option<(Vec2, f32)>,
    /// When it next jumps for nothing, running; which way it faces this
    /// frame, if it means to; and how fast it went the frame before.
    hop_at: f32,
    face: Option<f32>,
    speed: f32,
}

impl Racer {
    fn new(seat: usize, body: Body, kind: &'static Kind, dice: &mut Dice) -> Self {
        Self {
            body,
            seat,
            side: team(seat),
            kind,
            lean: Leanings::of(kind, dice),
            metres: super::START_METRES,
            chains: 0,
            laying: None,
            goal: Goal::Wait,
            think_at: 0.0,
            path: Vec::new(),
            path_to: None,
            path_version: 0,
            path_cuts: 0,
            path_at: f32::NEG_INFINITY,
            progress: (body.at.xz(), 0.0),
            looked_at: f32::NEG_INFINITY,
            walked_at: f32::NEG_INFINITY,
            stalls: 0,
            stalled: None,
            pinned: 0,
            weave: dice.between(0.0, TAU),
            cross: None,
            leaping: None,
            foe: None,
            in_reach_since: None,
            punch_at: 0.0,
            windup: WINDUP,
            duel: None,
            keen: 1.0,
            keen_until: 0.0,
            reel: None,
            reel_until: f32::NEG_INFINITY,
            grip: 1.0,
            alarm: None,
            look_at: 0.0,
            worth: 3.0,
            sighted: None,
            aim: None,
            wait_back: 1.0,
            guard_at: None,
            hold_at: None,
            hop_at: dice.between(1.0, 4.0),
            face: None,
            speed: 0.0,
        }
    }

    /// Which kind it is.
    pub fn kind(&self) -> &'static Kind {
        self.kind
    }

    /// How it is, its kind a little more or less so.
    pub fn leanings(&self) -> Leanings {
        self.lean
    }

    /// What it is about.
    pub fn goal(&self) -> Goal {
        self.goal
    }

    /// How many metres of bridge it has left to lay.
    pub fn metres(&self) -> f32 {
        self.metres
    }

    /// How often it has got nowhere on a way it had and thought again, and
    /// where it last did, heading where.
    pub fn stalls(&self) -> (u32, Option<(Vec2, Vec2)>) {
        (self.stalls, self.stalled)
    }

    /// How many of those times it could not take the next step at all.
    pub fn pinned(&self) -> u32 {
        self.pinned
    }

    /// Whether it is on its way somewhere, with a way there to walk.
    pub fn walking(&self) -> bool {
        !self.path.is_empty()
    }

    /// The run of its team's way it is laying, if any, by the way's rungs:
    /// where it begins, where it ends, and the rung it lays next.
    pub fn laying(&self) -> Option<(usize, usize, usize)> {
        self.laying.map(|laying| (laying.from, laying.to, laying.next))
    }
}

/// Where everyone was last seen moving, and how fast: what a bot reads
/// where someone will be from.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct Track {
    known: bool,
    at: Vec2,
    since: f32,
    velocity: Vec2,
}

/// Every one of the computer's players in a game, and each team's way to
/// the golden island.
#[derive(Clone, Debug)]
pub struct Crew {
    racers: Vec<Option<Racer>>,
    ways: [Option<Way>; TEAMS],
    tracks: Vec<Track>,
    watched_at: f32,
    dice: Dice,
}

impl Crew {
    /// The computer's players of a game on the map `grid`, none seated
    /// yet, and each team's way across thrown: from its island to the
    /// golden island, to one side of the straight line between their
    /// middles, a little more often away from the other team's than toward
    /// it, and bent as Bridge Wars' are, a little or a lot. Toward it, the
    /// two teams' bridges come within a throw of each other, and a jump,
    /// a long way out from the golden island.
    pub fn new(seed: u64, grid: &Grid) -> Self {
        let mut crew = Self {
            racers: Vec::new(),
            ways: [None, None],
            tracks: Vec::new(),
            watched_at: f32::NEG_INFINITY,
            dice: Dice::new(seed),
        };
        for side in 0..TEAMS {
            let along = (GOLDEN - ISLANDS[side]).normalize();
            let right = Vec2::new(-along.y, along.x);
            let away = (ISLANDS[side] - ISLANDS[1 - side]).normalize();
            let outward = if right.dot(away) >= 0.0 { 1.0 } else { -1.0 };
            let aside = crew.dice.around(-2.0, 2.6) * outward;
            let span = grid
                .span(Region::Team(side as u8), Region::Middle, aside)
                .or_else(|| grid.span(Region::Team(side as u8), Region::Middle, 0.0));
            if let Some(span) = span {
                let bent = crew.bends(span);
                crew.ways[side] = Some(Way::new(bent));
            }
        }
        crew
    }

    /// The computer takes `seat`, standing `at`: as one of [`KINDS`], by
    /// chance.
    pub fn seat(&mut self, seat: usize, at: Vec3, yaw: f32) {
        if self.racers.len() <= seat {
            self.racers.resize(seat + 1, None);
        }
        let which = ((self.dice.roll() * KINDS.len() as f32) as usize).min(KINDS.len() - 1);
        self.racers[seat] = Some(Racer::new(seat, Body::standing(at, yaw), &KINDS[which], &mut self.dice));
    }

    /// Has the one in `seat` be the kind called `name`: for trying one out.
    /// Whether there is such a one and such a kind.
    pub fn cast(&mut self, seat: usize, name: &str) -> bool {
        let Some(kind) = KINDS.iter().find(|kind| kind.name == name) else {
            return false;
        };
        let Some(racer) = self.racers.get_mut(seat).and_then(Option::as_mut) else {
            return false;
        };
        racer.kind = kind;
        racer.lean = Leanings::of(kind, &mut self.dice);
        true
    }

    /// The computer's player in `seat`, if it has one there.
    pub fn racer(&self, seat: usize) -> Option<&Racer> {
        self.racers.get(seat).and_then(Option::as_ref)
    }

    /// Its body.
    pub fn body(&self, seat: usize) -> Option<&Body> {
        self.racer(seat).map(|racer| &racer.body)
    }

    /// How many metres of bridge the one in `seat` has left to lay.
    pub fn metres(&self, seat: usize) -> f32 {
        self.racer(seat).map_or(0.0, |racer| racer.metres)
    }

    /// `team`'s way to the golden island.
    pub fn way(&self, team: u8) -> Option<Span> {
        self.ways.get(usize::from(team)).and_then(Option::as_ref).map(|way| way.span)
    }

    /// Which of the planks of `team`'s way stand, as it last looked, plank
    /// by plank from its island.
    pub fn standing(&self, team: u8) -> Vec<bool> {
        self.ways.get(usize::from(team)).and_then(Option::as_ref).map_or_else(Vec::new, |way| way.standing.clone())
    }

    /// The one in `seat` stands `at`: as the play begins, and each time it
    /// stands again after a fall. What it was about is behind it; the
    /// metres it had it keeps, and the bridge it was laying, to take up
    /// again if it is still to be had.
    pub fn stand(&mut self, seat: usize, at: Vec3, yaw: f32, now: f32) {
        let think = now + self.dice.between(0.1, 0.6);
        let Some(racer) = self.racers.get_mut(seat).and_then(Option::as_mut) else {
            return;
        };
        racer.body = Body::standing(at, yaw);
        racer.goal = Goal::Wait;
        racer.think_at = think;
        racer.path.clear();
        racer.path_to = None;
        racer.progress = (at.xz(), now);
        racer.cross = None;
        racer.leaping = None;
        racer.foe = None;
        racer.in_reach_since = None;
        racer.duel = None;
        racer.reel = None;
        racer.reel_until = f32::NEG_INFINITY;
        racer.alarm = None;
        racer.aim = None;
        racer.sighted = None;
        racer.guard_at = None;
        racer.hold_at = None;
    }

    /// The one in `seat` is given `metres` more of bridge, `now`: it
    /// thinks again a moment later, as anyone takes a moment to see what
    /// they have.
    pub fn grant(&mut self, seat: usize, metres: f32, now: f32) {
        let wakes = now + self.dice.around(0.15, 1.0);
        if let Some(racer) = self.racers.get_mut(seat).and_then(Option::as_mut) {
            racer.metres += metres;
            racer.think_at = racer.think_at.min(wakes);
        }
    }

    /// The one in `seat` has been knocked, from `from`, on the run if
    /// `running`: carried a while before it answers, and steering back by
    /// so much of its pace, each thrown again, the better it is in a fight
    /// the sooner and the surer.
    pub fn knock(&mut self, seat: usize, from: Vec2, running: bool) {
        let pvp = self.racer(seat).map_or(0.0, |racer| racer.lean.pvp);
        let slowest = REEL.0 + (REEL.1 - REEL.0) * (1.0 - 0.7 * pvp);
        let weakest = GRIP.0 + (GRIP.1 - GRIP.0) * 0.7 * pvp;
        let (reel, grip) = (self.dice.around(REEL.0, slowest), self.dice.around(weakest, GRIP.1));
        if let Some(racer) = self.racers.get_mut(seat).and_then(Option::as_mut) {
            racer.body.knock(from, running, 1.0);
            racer.reel = Some(reel);
            racer.grip = grip;
            racer.leaping = None;
        }
    }

    /// One frame of the one in `seat`: it thinks, if it is time; it does
    /// what it thought; its body is moved; and what the game is to carry
    /// out of it is given back.
    pub fn step(&mut self, seat: usize, view: &View, dt: f32) -> Order {
        let mut order = Order::default();
        self.watch(view);
        for way in self.ways.iter_mut().flatten() {
            way.refresh(view.nav);
        }
        let Some(mut racer) = self.racers.get_mut(seat).and_then(Option::take) else {
            return order;
        };
        let now = view.elapsed;
        let before = racer.body.at.xz();
        if let Some(reel) = racer.reel.take() {
            racer.reel_until = now + reel;
        }
        if now >= racer.keen_until {
            racer.keen = self.dice.around(KEEN.0, KEEN.1);
            racer.keen_until = now + self.dice.around(KEEN_LASTS.0, KEEN_LASTS.1);
        }
        if now >= racer.think_at {
            self.think(&mut racer, view);
        }
        let walk = self.act(&mut racer, view, dt, &mut order);
        racer.body.step(view.nav, walk, dt);
        if let Some(face) = racer.face.take() {
            racer.body.yaw = face;
        }
        racer.speed = racer.body.at.xz().distance(before) / dt.max(1e-4);
        self.racers[seat] = Some(racer);
        order
    }

    /// Looks at where everyone is, once a frame, whichever of them steps
    /// first: how fast and which way each is going, smoothed a little.
    fn watch(&mut self, view: &View) {
        let now = view.elapsed;
        if now <= self.watched_at {
            return;
        }
        self.watched_at = now;
        if self.tracks.len() < view.seats.len() {
            self.tracks.resize(view.seats.len(), Track::default());
        }
        for (track, seen) in self.tracks.iter_mut().zip(view.seats) {
            let at = seen.at.xz();
            let fresh = Track {
                known: seen.alive,
                at,
                since: now,
                velocity: Vec2::ZERO,
            };
            let (moved, dt) = (at - track.at, now - track.since);
            if !seen.alive || !track.known || dt < 1e-4 || moved.length() > 8.0 {
                *track = fresh;
                continue;
            }
            if moved.length_squared() < 1e-6 {
                if dt > 0.25 {
                    *track = fresh;
                }
                continue;
            }
            track.velocity += (moved / dt - track.velocity) * (1.0 - (-dt / 0.1).exp());
            track.at = at;
            track.since = now;
        }
    }

    // ---------------------------------------------------------- thinking

    /// What the racer sets out to do, every second or so: each thing it
    /// might be doing scored, the one it is on a little more, and the best
    /// taken. Each score a spread, thrown again each time.
    fn think(&mut self, racer: &mut Racer, view: &View) {
        let (nav, now) = (view.nav, view.elapsed);
        racer.think_at = now + self.dice.around(THINK_EVERY.0, THINK_EVERY.1);
        let here = racer.body.at.xz();
        let lean = racer.lean;
        let mut offers: Vec<(f32, Goal)> = Vec::new();
        let gold = nav.heart(Region::Middle);
        if Self::reachable(nav, here, gold) {
            // Every one of the team has to be there: whatever else, over.
            offers.push((1.3 + 0.05 * self.dice.bell(), Goal::Hold));
        } else {
            if let Some((span, ..)) = self.next_lay(racer, nav)
                && racer.metres + 1e-3 >= span.length() / span.planks() as f32
            {
                offers.push((0.8 + 0.35 * lean.lays + 0.08 * self.dice.bell(), Goal::Lay));
            }
            offers.push((0.55 + 0.3 * lean.waits + 0.08 * self.dice.bell(), Goal::Wait));
            let home = here.distance(self.root(racer.side, nav, 1.5));
            let walk = GUARD_WALK * (home - GUARD_NEAR).max(0.0);
            offers.push((0.5 + 0.3 * (1.0 - lean.waits) - walk + 0.08 * self.dice.bell(), Goal::Guard));
            // A jump short of the golden island, the bold jump.
            if self.gold_leap(racer, nav).is_some() {
                let keen = 0.55 + 0.6 * lean.leaps * lean.bold.min(1.3) + 0.1 * self.dice.bell();
                offers.push((keen, Goal::Hold));
            }
        }
        if view.kits.get(racer.seat).is_some_and(|kit| kit.held(Utility::Dynamite) > 0)
            && let Some((stand, at, worth)) = self.sabotage(racer, view)
        {
            racer.aim = Some((stand, at));
            let keen = 0.45 + 0.45 * lean.sticks + 0.04 * worth + 0.3 * (urgency(racer, view) - 1.0) + 0.08 * self.dice.bell();
            offers.push((keen, Goal::Sabotage));
        }
        if let Some(foe) = self.prey(racer, view) {
            let keen = 0.5 + 0.55 * lean.meets + 0.08 * self.dice.bell();
            offers.push((keen, Goal::Hunt(foe)));
        }
        for offer in &mut offers {
            if offer.1 == racer.goal {
                offer.0 += STICKY;
            }
        }
        let Some(&(_, goal)) = offers.iter().max_by(|a, b| a.0.total_cmp(&b.0)) else {
            return;
        };
        if goal != racer.goal {
            racer.goal = goal;
            racer.path.clear();
            racer.path_to = None;
            racer.guard_at = None;
            racer.wait_back = self.dice.around(WAIT_BACK.0, WAIT_BACK.1);
        }
    }

    /// How long one plank of `team`'s way is, in metres: what each of its
    /// rungs costs.
    fn plank_length(&self, side: u8) -> f32 {
        self.ways[usize::from(side)]
            .as_ref()
            .map_or(f32::INFINITY, |way| way.span.length() / way.planks() as f32)
    }

    /// The first run of planks missing from its team's way that the racer
    /// can get to the near end of, and that no teammate is laying with the
    /// metres to: the way's end, or a hole a stick has blown between it and
    /// its island.
    fn next_run(&self, racer: &Racer, nav: &Nav) -> Option<Run> {
        self.first_run(racer, nav, true)
    }

    /// The first run of planks missing from its team's way that the racer
    /// can get to the near end of; `free`, only one no teammate is laying.
    fn first_run(&self, racer: &Racer, nav: &Nav, free: bool) -> Option<Run> {
        let way = self.ways[usize::from(racer.side)].as_ref()?;
        let here = racer.body.at.xz();
        let planks = way.planks();
        let mut plank = 0;
        while plank < planks {
            if way.standing[plank] {
                plank += 1;
                continue;
            }
            let to = (plank..planks).find(|&next| way.standing[next]).unwrap_or(planks);
            let near = if plank == 0 { way.rungs[0] } else { way.middle(plank - 1) };
            // A teammate is on it: laying, with the metres for the plank
            // that comes next, and that plank this one.
            let taken = free
                && self.racers.iter().flatten().any(|mate| {
                    mate.seat != racer.seat
                        && mate.side == racer.side
                        && mate.goal == Goal::Lay
                        && mate.metres + 1e-3 >= self.plank_length(racer.side)
                        && mate.laying.is_some_and(|laying| laying.span == way.span && laying.coming() == plank)
                });
            if !taken && nav.connected(here, near) {
                return Some(Run { from: plank, to });
            }
            plank = to;
        }
        None
    }

    /// What the racer would lay next, as a span and the run of its rungs:
    /// the first gap in its team's way it can get to ([`Crew::next_run`]);
    /// or, cut off from all of that, as one left on the other team's bridge
    /// by a fight is, a short way of its own straight onto the golden
    /// island from the end of whatever bridge it can get to near it
    /// ([`Crew::near_end`]).
    fn next_lay(&self, racer: &Racer, nav: &Nav) -> Option<(Span, usize, usize)> {
        if let Some(run) = self.next_run(racer, nav) {
            let way = self.ways[usize::from(racer.side)].as_ref()?;
            return Some((way.span, run.from, run.to));
        }
        let (end, _) = self.near_end(racer, nav)?;
        let shore = GOLDEN + (end - GOLDEN).normalize_or_zero() * (super::ISLAND_RADIUS - 0.5);
        if end.distance(shore) < 0.5 {
            return None;
        }
        let span = Span::straight(end, shore);
        Some((span, 0, span.planks()))
    }

    /// The end of a bridge, any team's, out over the void a little way off
    /// the golden island and heading for it, that the racer can get to:
    /// the one nearest the island, as its last rung and the rung before it.
    /// Where it jumps from, or lays a short way from, cut off from its own
    /// team's way.
    fn near_end(&self, racer: &Racer, nav: &Nav) -> Option<(Vec2, Vec2)> {
        let here = racer.body.at.xz();
        let grid = nav.grid();
        let mut best: Option<(f32, Vec2, Vec2)> = None;
        for (a, b) in nav.planks() {
            for (end, back) in [(b, a), (a, b)] {
                let off = end.distance(GOLDEN) - super::ISLAND_RADIUS;
                if !(0.0..=RUN_UP + 2.0).contains(&off) || grid.height(end).is_some() {
                    continue;
                }
                let ahead = (end - back).normalize_or_zero();
                if ahead.dot((GOLDEN - end).normalize_or_zero()) < 0.3 || nav.on_plank(end + ahead * 0.6) {
                    continue;
                }
                if best.is_some_and(|(least, ..)| off >= least) || !nav.connected(here, (a + b) * 0.5) {
                    continue;
                }
                best = Some((off, end, back));
            }
        }
        best.map(|(_, end, back)| (end, back))
    }

    /// Whether a bot at a run can get from `a` to `b`: walking, or walking
    /// and jumping one hole in a bridge on the way ([`Crew::crossing`]).
    fn reachable(nav: &Nav, a: Vec2, b: Vec2) -> bool {
        nav.connected(a, b) || Self::crossing(nav, a, b, Self::reach_of_a_jump(RUN_SPEED)).is_some()
    }

    /// A hole to jump on the way from `a` to `b` ([`Nav::crossing`]), with
    /// somewhere to stand to jump it and to come down past it, either side
    /// of it. ([`Nav::crossing`] has the near side of a hole as one can get
    /// to from anywhere within a couple of metres of it: one left on a
    /// plank or two between two holes, a blast or two having taken the
    /// planks either side of it, was told the golden island was a jump
    /// away, and stood on its planks, its metres in hand, going nowhere.)
    fn crossing(nav: &Nav, a: Vec2, b: Vec2, most: f32) -> Option<Gap> {
        nav.crossing(a, b, most).filter(|gap| nav.walkable_at(gap.a_in) && nav.walkable_at(gap.b_in))
    }

    /// How far across a gap a bot going at `pace` jumps.
    fn reach_of_a_jump(pace: f32) -> f32 {
        pace * JUMP_SECS - JUMP_SPARE
    }

    /// A jump onto the golden island from the end of a bridge the racer can
    /// get to near it ([`Crew::near_end`]), if that is a jump short of it:
    /// where the ground ends and where it comes down.
    fn gold_leap(&self, racer: &Racer, nav: &Nav) -> Option<(Vec2, Vec2)> {
        let (end, back) = self.near_end(racer, nav)?;
        let from = (end + back) * 0.5;
        let aim = GOLDEN + (end - GOLDEN).normalize_or_zero() * (super::ISLAND_RADIUS - 2.5);
        nav.leap(from, aim, Self::reach_of_a_jump(RUN_SPEED))
    }

    /// Whoever of the other team the racer would go out to meet: someone
    /// on its own team's bridge, or on the golden island, that it can get
    /// to, not too far off.
    fn prey(&self, racer: &Racer, view: &View) -> Option<usize> {
        let nav = view.nav;
        let here = racer.body.at.xz();
        let way = self.ways[usize::from(racer.side)].as_ref()?;
        // How far it goes out to meet them, by who it is, and by how keen
        // it is just now ([`KEEN`]).
        let within = (12.0 + 22.0 * racer.lean.meets) * racer.keen;
        view.seats
            .iter()
            .enumerate()
            .filter(|&(seat, seen)| seen.alive && team(seat) != racer.side)
            .filter(|(_, seen)| {
                let there = seen.at.xz();
                let on_ours = way.rungs.windows(2).any(|pair| over_plank(there, pair[0], pair[1]))
                    && nav.grid().height(there).is_none()
                    && nav.on_plank(there);
                let on_gold = nav.grid().region(there) == Some(Region::Middle);
                (on_ours || on_gold) && there.distance(here) <= within && Self::reachable(nav, here, there)
            })
            .min_by(|a, b| a.1.at.xz().distance(here).total_cmp(&b.1.at.xz().distance(here)))
            .map(|(seat, _)| seat)
    }

    /// Where the racer, with a stick, might throw it from and at, and what
    /// it is worth: from somewhere it can walk to on its own team's way or
    /// the golden island, at the other team's bridge within a throw.
    fn sabotage(&self, racer: &Racer, view: &View) -> Option<(Vec2, Vec2, f32)> {
        let nav = view.nav;
        let here = racer.body.at.xz();
        let way = self.ways[usize::from(racer.side)].as_ref()?;
        // Where it could stand: along its team's way, as far as it can get,
        // and round the golden island's shore if it can get there.
        let mut stands: Vec<Vec2> = (0..way.planks()).filter(|&plank| way.standing[plank]).map(|plank| way.middle(plank)).collect();
        if Self::reachable(nav, here, nav.heart(Region::Middle)) {
            stands.extend((0..16).map(|k| GOLDEN + Vec2::from_angle(k as f32 * TAU / 16.0) * (super::ISLAND_RADIUS - 1.5)));
        }
        stands.retain(|&at| nav.walkable_at(at) && nav.connected(here, at));
        let targets: Vec<Vec2> = nav
            .planks()
            .map(|(a, b)| (a + b) * 0.5)
            .filter(|&at| nav.grid().height(at).is_none())
            .collect();
        let mut best: Option<(f32, Vec2, Vec2, f32)> = None;
        for &at in &targets {
            let Some(worth) = self.worth(racer, view, at) else {
                continue;
            };
            let Some(&stand) = stands
                .iter()
                .filter(|stand| (DYNAMITE_NEAREST + 0.5..=DYNAMITE_FURTHEST - 0.8).contains(&stand.distance(at)))
                .min_by(|a, b| a.distance(here).total_cmp(&b.distance(here)))
            else {
                continue;
            };
            let score = worth - 0.08 * stand.distance(here);
            if best.is_none_or(|(most, ..)| score > most) {
                best = Some((score, stand, at, worth));
            }
        }
        best.map(|(_, stand, at, worth)| (stand, at, worth))
    }

    /// What a stick of the racer's coming down `at` is worth to it, in
    /// planks of the other team's taken, more for each of them it would
    /// send into the void and for each it would knock; none if it takes a
    /// plank of its own team's, or the ground from under one of its own,
    /// itself among them.
    fn worth(&self, racer: &Racer, view: &View, at: Vec2) -> Option<f32> {
        let nav = view.nav;
        let taken = nav.would_blast(at, DYNAMITE_RADIUS);
        if taken.is_empty() || taken.iter().any(|&(seat, ..)| team(usize::from(seat)) == racer.side) {
            return None;
        }
        let going: Vec<(Vec2, Vec2)> = taken
            .iter()
            .filter_map(|&(seat, chain, plank)| {
                let rungs = nav.rungs(seat, chain);
                Some((*rungs.get(usize::from(plank))?, *rungs.get(usize::from(plank) + 1)?))
            })
            .collect();
        let falls = |there: Vec2| nav.grid().height(there).is_none() && going.iter().any(|&(a, b)| over_plank(there, a, b));
        let mut worth = taken.len() as f32;
        for (seat, seen) in view.seats.iter().enumerate() {
            if !seen.alive {
                continue;
            }
            let there = seen.at.xz();
            if team(seat) == racer.side {
                if falls(there) {
                    return None;
                }
                continue;
            }
            if falls(there) {
                worth += WORTH_FALL;
            } else if there.distance(at) <= DYNAMITE_RADIUS + BODY {
                worth += WORTH_KNOCK;
            }
        }
        Some(worth)
    }

    // ------------------------------------------------------------- acting

    /// What the racer does this frame, and which way it walks.
    fn act(&mut self, racer: &mut Racer, view: &View, dt: f32, order: &mut Order) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = racer.body.at.xz();
        racer.face = None;
        racer.body.leap = false;
        racer.body.high = false;
        if now < racer.reel_until {
            return Vec2::ZERO;
        }
        let ground = nav.ground(here);
        let in_air = racer.body.lift != 0.0 || ground.is_none_or(|ground| (racer.body.at.y - ground).abs() > 0.02);
        // Jumping a gap: at where it means to come down and nothing else
        // (Bridge Wars' own).
        if let Some(mut leap) = racer.leaping {
            leap.aloft |= in_air;
            let down = leap.aloft && !in_air;
            let there = !in_air && here.distance(leap.to) < 0.4;
            let gone = in_air && nav.ground(leap.to).is_none();
            if down || there || gone || now >= leap.until {
                racer.leaping = None;
                racer.path.clear();
                racer.path_to = None;
                racer.progress = (here, now);
            } else {
                racer.leaping = Some(leap);
                racer.body.leap = true;
                let off = leap.to - here;
                return off.normalize_or_zero() * leap.pace.min(off.length() / dt.max(1e-4));
            }
        }
        // Knocked out over the void: back for the nearest ground, by as
        // much of its pace as it steers by this time.
        if in_air && ground.is_none() {
            return nav
                .nearest_walkable_within(here, REACH_BACK)
                .map_or(Vec2::ZERO, |land| (land - here).normalize_or_zero() * RUN_SPEED * racer.grip);
        }
        if racer.body.shove != Vec2::ZERO {
            let carried = here + racer.body.shove * if in_air { 0.5 } else { 0.11 };
            if nav.ground(carried).is_none() {
                return -racer.body.shove.normalize_or_zero() * RUN_SPEED * racer.grip;
            }
        }
        let walk = self.intend(racer, view, dt, order);
        if racer.leaping.is_some() {
            return walk;
        }
        self.mind_throw(racer, view, order);
        let walk = self.mind_sticks(racer, view, walk);
        // Running somewhere, a jump now and then, as people run; not with a
        // plank in its arms, in a fight, or on the golden island, where its
        // feet in the air would empty its team's ring.
        let running = walk.length() > RUN_SPEED * 0.6;
        let on_gold = nav.grid().region(here) == Some(Region::Middle);
        if running && !in_air && racer.duel.is_none() && !order.carrying && !on_gold && now >= racer.hop_at {
            racer.hop_at = now + self.dice.around(0.7, 3.2) / (0.25 + racer.lean.bouncy);
            if self.dice.roll() < 0.5 {
                racer.body.jump(nav);
            }
        }
        walk
    }

    /// What the racer does toward its goal this frame, a fight with whoever
    /// of the other team comes near it before anything.
    fn intend(&mut self, racer: &mut Racer, view: &View, dt: f32, order: &mut Order) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = racer.body.at.xz();
        let foe = self.foe(racer, view);
        if foe != racer.foe {
            racer.in_reach_since = None;
        }
        racer.foe = foe;
        if let Some(foe) = foe {
            // Laying, with nobody right on top of it, it lays on.
            let laying = racer.goal == Goal::Lay && racer.laying.is_some() && view.seats[foe].at.xz().distance(here) > ON_TOP_OF_IT;
            if !laying {
                return self.fight(racer, view, foe, RUN_SPEED, dt, order);
            }
        } else {
            racer.duel = None;
        }
        match racer.goal {
            Goal::Lay => self.lay(racer, view, dt, order),
            Goal::Wait => {
                let to = self.wait_spot(racer, nav);
                self.walk_to(racer, view, to, RUN_SPEED, 1.0)
            }
            Goal::Guard => {
                let to = match racer.guard_at {
                    Some(at) => at,
                    None => {
                        let at = self.guard_spot(racer, nav);
                        racer.guard_at = Some(at);
                        at
                    }
                };
                self.walk_to(racer, view, to, RUN_SPEED, 1.0)
            }
            Goal::Hold => self.hold(racer, view, dt),
            Goal::Hunt(foe) => {
                let Some(seen) = view.seats.get(foe).filter(|seen| seen.alive) else {
                    racer.think_at = now;
                    return Vec2::ZERO;
                };
                self.walk_to(racer, view, seen.at.xz(), RUN_SPEED, 0.5)
            }
            Goal::Sabotage => {
                let Some((stand, at)) = racer.aim else {
                    racer.think_at = now;
                    return Vec2::ZERO;
                };
                if view.kits.get(racer.seat).is_none_or(|kit| kit.held(Utility::Dynamite) == 0) {
                    racer.aim = None;
                    racer.think_at = now;
                    return Vec2::ZERO;
                }
                // There, or near enough to throw from where it is: it stands
                // and throws ([`Crew::mind_throw`]).
                let far = here.distance(at);
                if (DYNAMITE_NEAREST + 0.3..=DYNAMITE_FURTHEST - 0.3).contains(&far) && here.distance(stand) < 3.0 {
                    racer.path.clear();
                    racer.path_to = None;
                    return Vec2::ZERO;
                }
                self.walk_to(racer, view, stand, RUN_SPEED, 1.0)
            }
        }
    }

    /// Where the racer waits for metres: a little short of the end of the
    /// first gap in its team's way that it can get to, on what stands,
    /// behind whoever of its team is laying it (until this was so, one
    /// whose teammate was laying the end of the way went back to its
    /// island to wait, and out again once they were done); or, with
    /// nothing laid there yet, by where it leaves its island.
    fn wait_spot(&self, racer: &Racer, nav: &Nav) -> Vec2 {
        let Some(way) = self.ways[usize::from(racer.side)].as_ref() else {
            return nav.heart(Region::Team(racer.side));
        };
        match self.first_run(racer, nav, false) {
            Some(run) if run.from > 0 => {
                let at = way.back_from(run.from, racer.wait_back);
                nav.nearest_walkable(at).unwrap_or(at)
            }
            Some(_) => self.root(racer.side, nav, 1.5),
            // Cut off from its team's way: at the end of the bridge it is
            // on near the golden island, if it is on one, for the metres to
            // lay the rest; or where it is.
            None => match self.near_end(racer, nav) {
                Some((end, back)) => nav.nearest_walkable((end + back) * 0.5).unwrap_or(end),
                None if nav.connected(racer.body.at.xz(), nav.heart(Region::Team(racer.side))) => self.root(racer.side, nav, 1.5),
                None => racer.body.at.xz(),
            },
        }
    }

    /// Where `team`'s way leaves its island, `back` metres in from it, on
    /// ground a body walks.
    fn root(&self, side: u8, nav: &Nav, back: f32) -> Vec2 {
        let Some(way) = self.ways[usize::from(side)].as_ref() else {
            return nav.heart(Region::Team(side));
        };
        let inward = (ISLANDS[usize::from(side)] - way.span.from).normalize_or_zero();
        let at = way.span.from + inward * back;
        nav.grid().nearest_clear(at, 3.0).unwrap_or_else(|| nav.heart(Region::Team(side)))
    }

    /// Where the racer stands keeping its island: in from where its team's
    /// way leaves it, a little to one side, thrown each time.
    fn guard_spot(&mut self, racer: &Racer, nav: &Nav) -> Vec2 {
        let side = racer.side;
        let Some(way) = self.ways[usize::from(side)].as_ref() else {
            return nav.heart(Region::Team(side));
        };
        let inward = (ISLANDS[usize::from(side)] - way.span.from).normalize_or_zero();
        let across = Vec2::new(-inward.y, inward.x);
        let at = way.span.from + inward * self.dice.around(GUARD_IN.0, GUARD_IN.1) + across * self.dice.around(GUARD_ASIDE.0, GUARD_ASIDE.1);
        nav.grid().nearest_clear(at, 3.0).unwrap_or_else(|| self.root(side, nav, 2.0))
    }

    /// Over onto the golden island and standing there: walked to, if it can
    /// be; jumped onto from the end of its team's way, if it is a jump
    /// short; and once there, somewhere on it, inside its ring, moving to
    /// another such place now and then.
    fn hold(&mut self, racer: &mut Racer, view: &View, dt: f32) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = racer.body.at.xz();
        let gold = nav.heart(Region::Middle);
        if !Self::reachable(nav, here, gold) {
            let Some((edge, down)) = self.gold_leap(racer, nav) else {
                racer.think_at = racer.think_at.min(now + 0.2);
                return Vec2::ZERO;
            };
            // To the edge's run-up, and off it at a run.
            let run_from = edge + (edge - down).normalize_or_zero() * 3.0;
            let ready = here.distance(edge) <= 3.5 && (down - here).normalize_or_zero().dot((down - edge).normalize_or_zero()) > 0.9;
            if ready && racer.body.free(nav) {
                racer.leaping = Some(Leap {
                    to: down,
                    pace: RUN_SPEED,
                    until: now + 2.0,
                    aloft: false,
                });
                return (down - here).normalize_or_zero() * RUN_SPEED;
            }
            let near = nav.nearest_walkable(run_from).unwrap_or(run_from);
            return self.walk_to(racer, view, near, RUN_SPEED, 0.5);
        }
        let spot = match racer.hold_at {
            Some((at, until)) if now < until => at,
            _ => {
                let mut at = gold;
                for _ in 0..8 {
                    let out = self.dice.around(HOLD_OUT.0, HOLD_OUT.1);
                    let tried = GOLDEN + Vec2::from_angle(self.dice.between(0.0, TAU)) * out;
                    if nav.grid().clear(tried) && nav.connected(here, tried) {
                        at = tried;
                        break;
                    }
                }
                racer.hold_at = Some((at, now + self.dice.around(HOLD_STAYS.0, HOLD_STAYS.1)));
                at
            }
        };
        if here.distance(spot) <= ARRIVED {
            return Vec2::ZERO;
        }
        if nav.line(here, spot) {
            racer.path.clear();
            racer.path_to = None;
            let off = spot - here;
            let walk = off.normalize_or_zero() * RUN_SPEED.min(off.length() / dt.max(1e-3));
            if Self::slit_ahead(nav, here, spot)
                && let Some(over) = self.over_a_slit(racer, nav, spot, now)
            {
                return over;
            }
            return walk;
        }
        self.walk_to(racer, view, spot, RUN_SPEED, 0.5)
    }

    /// Lays the run of its team's way it is on, or the next one it can get
    /// to: to where it begins; then out along it at the pace anyone builds
    /// at, each plank laid ahead of its feet and paid for in metres; done
    /// at its end, or with no metres left (Bridge Wars' `Squad::lay`, by
    /// the way's own rungs).
    fn lay(&mut self, racer: &mut Racer, view: &View, dt: f32, order: &mut Order) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = racer.body.at.xz();
        // Still to be laid as it was: the plank it laid last still stands
        // (its rung `next` - 1 begins the plank to come, and the one before
        // ends the last laid), the plank it lays next is still missing, and
        // it can still get to where it takes it up from. (Knocked off and
        // back on its island, with a hole blown in its bridge since, it
        // cannot, and lays whatever it can get to first: until this was so
        // it stood on its side of the hole for the rest of the game.)
        if let Some(laying) = racer.laying {
            let planks = laying.span.planks();
            let done = laying.next > laying.to;
            let laid_stands = laying.next < laying.from + 2 || nav.on_plank(laying.middle(laying.next - 2));
            let coming = laying.coming();
            let still_missing = done || coming >= planks || !nav.on_plank(laying.middle(coming));
            if done || !laid_stands || !still_missing || !nav.connected(here, laying.resumes_at()) {
                racer.laying = None;
            }
        }
        let laying = match racer.laying {
            Some(laying) => laying,
            None => {
                let Some((span, from, to)) = self.next_lay(racer, nav) else {
                    racer.think_at = now;
                    return Vec2::ZERO;
                };
                if racer.metres + 1e-3 < span.length() / span.planks() as f32 {
                    racer.think_at = now;
                    return Vec2::ZERO;
                }
                let laying = Laying {
                    span,
                    chain: racer.chains,
                    from,
                    to,
                    next: from,
                };
                racer.chains += 1;
                racer.laying = Some(laying);
                laying
            }
        };
        let span = laying.span;
        let planks = span.planks();
        let chord = span.chord();
        let step = chord / planks as f32;
        let plank_cost = span.length() / planks as f32;
        let along = (here - span.from).dot(span.way());
        let onto = span.point(along / chord.max(1e-3));
        let aside = here.distance(onto);
        // Not at its line yet: to the nearest a body can walk to where the
        // run begins, or, taking it up again, to as far as it is laid.
        let start = span.rung(laying.from);
        let begin_along = laying.from as f32 * step;
        let on_line = aside < 0.9 && along > begin_along - 2.0 && along < chord + 1.0;
        if !on_line || (laying.next == laying.from && here.distance(start) > 2.2) {
            let to = laying.resumes_at();
            let near = nav.nearest_walkable(to).unwrap_or(to);
            if here.distance(near) > ARRIVED + 0.3 {
                return self.walk_to(racer, view, near, RUN_SPEED, 2.0);
            }
        }
        // On its line, and walking by it, not by a way found.
        racer.path.clear();
        racer.path_to = None;
        // Its planks, a rung at a time, kept ahead of its feet: the first
        // rung, over ground already, costs nothing.
        let mut next = laying.next;
        while next <= laying.to && ((next as f32 - 1.0) * step < along + LAID_AHEAD + step || next == planks) {
            let cost = if next == laying.from { 0.0 } else { span.rung(next).distance(span.rung(next - 1)) };
            if racer.metres + 1e-3 < cost {
                break;
            }
            racer.metres = (racer.metres - cost).max(0.0);
            order.rungs.push((laying.chain, (next - laying.from) as u16, span.rung(next), span.heading_at(next)));
            next += 1;
        }
        racer.laying = Some(Laying { next, ..laying });
        order.carrying = true;
        // Laid to the end of the run, and walked to it: done.
        let end_along = laying.to as f32 * step;
        if next > laying.to && along >= end_along - 1.0 {
            racer.laying = None;
            racer.think_at = now;
            return Vec2::ZERO;
        }
        // Out along it, no further than it is laid.
        let laid = (next.saturating_sub(1)) as f32 * step;
        let limit = if next > planks { chord } else { (laid - 0.4).max(0.0) };
        let speed = builder_speed(&Upgrades::default()).min((limit - along).max(0.0) / dt.max(1e-3));
        let sideways = onto - here;
        if speed <= 0.0 && sideways.length() < 0.05 {
            // Out of metres on the way: it waits here for more.
            if racer.metres + 1e-3 < plank_cost {
                racer.think_at = racer.think_at.min(now + 0.2);
            }
            return Vec2::ZERO;
        }
        span.tangent(along / chord.max(1e-3)) * speed + sideways.clamp_length_max(1.0) * 3.0
    }

    /// Bends a span thrown for a way as Bridge Wars' are bent
    /// (`Squad::bends`, copied): a sweep to one side, a wave and a ripple,
    /// each as a bell rings it, all by how straight this one is to be;
    /// never so bent that it leaves or meets the land at more than a slant
    /// of [`MOST_SLANT`], nor that it is more than [`MOST_BENT`] times as
    /// long as the straight one.
    fn bends(&mut self, span: Span) -> Span {
        let chord = span.chord();
        let wavy = (1.0 + 0.5 * self.dice.bell()).clamp(0.1, 2.4) * chord / 44.0;
        let mut bend = [1.3, 0.7, 0.35].map(|far| far * wavy * self.dice.bell());
        for _ in 0..12 {
            let turn = PI / chord.max(1.0);
            let slants = [bend[0] + 2.0 * bend[1] + 3.0 * bend[2], bend[0] - 2.0 * bend[1] + 3.0 * bend[2]].map(|sum| (sum * turn).abs());
            let bent = span.bent(bend);
            if slants[0] <= MOST_SLANT && slants[1] <= MOST_SLANT && bent.length() <= chord * MOST_BENT - 0.2 {
                return bent;
            }
            bend = bend.map(|far| far * 0.8);
        }
        span
    }

    // ----------------------------------------------------------- fighting

    /// Whom the racer fights this frame, if anyone: the nearest of the
    /// other team it can get at, within how near it lets people come just
    /// now: on the golden island anyone there, after someone it is hunting
    /// them, and laying or out to throw a stick only whoever is on top of
    /// it. Whoever it is on already it keeps after a little further.
    fn foe(&self, racer: &Racer, view: &View) -> Option<usize> {
        let nav = view.nav;
        let here = racer.body.at.xz();
        let on_gold = |at: Vec2| nav.grid().region(at) == Some(Region::Middle) && nav.grid().height(at).is_some();
        let aggro = AGGRO * racer.lean.fights * racer.keen;
        let reach = match racer.goal {
            Goal::Hold if on_gold(here) => 11.0 * racer.keen,
            Goal::Hunt(_) => aggro * 1.4,
            Goal::Lay | Goal::Sabotage => ON_TOP_OF_IT,
            Goal::Hold | Goal::Wait | Goal::Guard => aggro,
        };
        let jump = Self::reach_of_a_jump(RUN_SPEED);
        view.seats
            .iter()
            .enumerate()
            .filter(|&(seat, seen)| seen.alive && team(seat) != racer.side)
            .filter_map(|(seat, seen)| {
                let there = seen.at.xz();
                let away = there.distance(here);
                let mut reach = if racer.foe == Some(seat) { reach * KEEPS_ON } else { reach };
                if racer.goal == Goal::Hunt(seat) {
                    reach = reach.max(40.0);
                }
                // Holding the golden island, whoever is on it.
                if racer.goal == Goal::Hold && on_gold(here) && on_gold(there) {
                    reach = reach.max(away);
                }
                if away > reach || (seen.at.y - racer.body.at.y).abs() > 3.0 {
                    return None;
                }
                if !nav.connected(here, there) && Self::crossing(nav, here, there, jump).is_none() && nav.leap(here, there, jump).is_none() {
                    return None;
                }
                Some((if racer.foe == Some(seat) { away * 0.8 } else { away }, seat))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, seat)| seat)
    }

    /// A fight with whoever is in `foe` (Bridge Wars' `Squad::fight`, the
    /// most of it): punched when in its reach, the better in a fight the
    /// quicker; stood off where its punch throws them off the land and
    /// theirs does not throw it off ([`Crew::spot`]), now and then stepping
    /// back to come in again on the run; and a gap between them jumped, the
    /// likelier the bolder.
    fn fight(&mut self, racer: &mut Racer, view: &View, foe: usize, pace: f32, dt: f32, order: &mut Order) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = racer.body.at.xz();
        let there = view.seats[foe].at.xz();
        let off = there - here;
        let away = off.length();
        racer.face = Some((-off.x).atan2(-off.y));
        let pvp = racer.lean.pvp;
        let every = PUNCH_EVERY - (PUNCH_EVERY - 0.53).max(0.0) * pvp;
        let windup = WINDUP * (1.0 - 0.65 * pvp);
        let on_gold = nav.grid().region(here) == Some(Region::Middle);
        let lunging = racer.duel.is_some_and(|duel| duel.foe == foe && duel.lunge);
        let mut struck = false;
        if away <= REACH + BODY - 0.1 {
            let since = *racer.in_reach_since.get_or_insert(now);
            if (lunging || now - since >= racer.windup) && now >= racer.punch_at {
                racer.punch_at = now + self.dice.around(every * (0.75 + 0.2 * pvp), every * (1.3 - 0.24 * pvp));
                racer.windup = self.dice.around(windup * 0.5, windup * 1.6);
                order.punch = Some(foe);
                order.running = racer.speed > RUN_SPEED * 0.6;
                struck = true;
                // Now and then with a jump, as people throw theirs; but not
                // holding the golden island.
                if !on_gold && self.dice.roll() < 0.1 + 0.3 * racer.lean.bouncy {
                    racer.body.jump(nav);
                }
            }
        } else if away > REACH + BODY + 0.5 {
            racer.in_reach_since = None;
        }
        let mut duel = match racer.duel {
            Some(duel) if duel.foe == foe => duel,
            _ => Duel {
                foe,
                spot: None,
                spot_until: 0.0,
                back_until: 0.0,
                lunge: false,
            },
        };
        if struck {
            duel.lunge = false;
            if self.dice.roll() < 0.35 + 0.55 * pvp {
                // Back out of their reach until its next punch is nearly
                // due, to come in again on the run.
                let timed = (racer.punch_at - (POKE_BACK - (REACH + BODY - 0.1)) / pace.max(1.0)).max(now + 0.12);
                duel.back_until = timed;
                duel.lunge = true;
            }
            duel.spot_until = now;
        }
        // A gap between them it could jump, with no walking to them: now
        // and then it jumps it, the likelier the bolder and the better.
        if racer.leaping.is_none()
            && racer.body.free(nav)
            && !nav.connected(here, there)
            && let Some((_, down)) = nav.leap(here, there, Self::reach_of_a_jump(pace))
            && self.dice.roll() < dt * (0.6 + 1.2 * racer.lean.leaps * racer.lean.bold)
        {
            racer.leaping = Some(Leap {
                to: down,
                pace,
                until: now + 2.0,
                aloft: false,
            });
            racer.duel = Some(duel);
            return (down - here).normalize_or_zero() * pace;
        }
        let backing = now < duel.back_until;
        if duel.spot.is_none() || now >= duel.spot_until {
            duel.spot = if backing {
                let back = here - off.normalize_or_zero() * (POKE_BACK - away).max(0.0);
                (nav.walkable_at(back) && flung(nav, there, back) == 0.0).then_some(back)
            } else {
                self.spot(racer, view, there)
            };
            let brisk = self.speed_of(foe) > 3.0;
            let again = now + if brisk { self.dice.between(0.1, 0.22) } else { self.dice.between(0.3, 0.7) };
            duel.spot_until = if backing { again.min(duel.back_until) } else { again };
        }
        let aim = self.lead(foe, there, here, pace, racer.lean.pvp);
        let toward = |crew: &mut Self, racer: &mut Racer, to: Vec2| {
            let stretch = to - here;
            if stretch.length() <= 0.15 {
                Vec2::ZERO
            } else if nav.line(here, to) {
                racer.path.clear();
                racer.path_to = None;
                let walk = stretch.normalize_or_zero() * pace.min(stretch.length() / dt.max(1e-3));
                if Self::slit_ahead(nav, here, to) {
                    crew.over_a_slit(racer, nav, to, now).unwrap_or(walk)
                } else {
                    walk
                }
            } else {
                crew.walk_to(racer, view, to, pace, 0.5)
            }
        };
        let walk = match duel.spot {
            Some(spot) => toward(self, racer, spot),
            None if backing || away <= REACH * PRESS_IN => Vec2::ZERO,
            None => toward(self, racer, aim),
        };
        racer.duel = Some(duel);
        walk
    }

    /// Where whoever is in `foe`, now `there`, will be by the time the
    /// racer, `here`, can be on them going at `pace`: a little ahead of
    /// where they are, the more so the better it is in a fight.
    fn lead(&self, foe: usize, there: Vec2, here: Vec2, pace: f32, pvp: f32) -> Vec2 {
        let Some(track) = self.tracks.get(foe).filter(|track| track.known && track.velocity.length() >= 1.0) else {
            return there;
        };
        let secs = (here.distance(there) / pace.max(1.0)).min(1.2);
        there + track.velocity * secs * (0.4 + 0.5 * pvp)
    }

    /// How fast whoever is in `seat` is going, in metres a second.
    fn speed_of(&self, seat: usize) -> f32 {
        self.tracks.get(seat).filter(|track| track.known).map_or(0.0, |track| track.velocity.length())
    }

    /// Where the racer stands to fight someone `there`: the best of twelve
    /// places round them a punch's length out, each a place a body can
    /// walk that it can get to; better for being near; much better, by how
    /// much it goes for that, if a punch from there would carry them off
    /// the land ([`flung`]), and worse, by how much it minds, if one of
    /// theirs would carry it off; and all of it a little by chance (Bridge
    /// Wars' `Squad::spot`).
    fn spot(&mut self, racer: &Racer, view: &View, there: Vec2) -> Option<Vec2> {
        let nav = view.nav;
        let here = racer.body.at.xz();
        let (shoves, minds) = (racer.lean.shoves, racer.lean.minds);
        let turn = self.dice.roll() * TAU / 12.0;
        let mut best: Option<(f32, Vec2)> = None;
        for place in 0..12 {
            let at = there + Vec2::from_angle(turn + place as f32 * TAU / 12.0) * RING;
            if !nav.walkable_at(at) || !nav.connected(here, at) {
                continue;
            }
            let mut score = -0.09 * here.distance(at) + shoves * flung(nav, at, there) - 1.1 * minds * flung(nav, there, at);
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

    // ---------------------------------------------------- sticks of dynamite

    /// With a stick on it, every so often: if a throw from where it stands
    /// is worth as much as it takes this time, it throws it, once it has
    /// taken it in that it is ([`SIGHTED`]); out to throw one
    /// ([`Goal::Sabotage`]), at what it went for, once it is within a throw
    /// of it. How much it takes is thrown again each time it looks, the
    /// keener the kind, and the further its team is behind, the less
    /// ([`urgency`]). (Without the moment it takes to see it, the two teams'
    /// bridges coming within a throw of each other had every one of the
    /// four throwing in the same half second.)
    fn mind_throw(&mut self, racer: &mut Racer, view: &View, order: &mut Order) {
        let now = view.elapsed;
        if now < racer.look_at || view.kits.get(racer.seat).is_none_or(|kit| kit.held(Utility::Dynamite) == 0) {
            return;
        }
        racer.look_at = now + self.dice.around(STICK_LOOKS.0, STICK_LOOKS.1);
        if !racer.body.free(view.nav) || racer.leaping.is_some() {
            return;
        }
        let here = racer.body.at.xz();
        let keen = racer.lean.sticks.max(0.05) * urgency(racer, view);
        racer.worth = self.dice.around(2.0, 9.0) / keen;
        let mut best: Option<(f32, Vec2)> = None;
        let mut targets: Vec<Vec2> = view
            .nav
            .planks()
            .map(|(a, b)| (a + b) * 0.5)
            .filter(|at| (DYNAMITE_NEAREST + 0.3..=DYNAMITE_FURTHEST - 0.3).contains(&at.distance(here)))
            .collect();
        // And at the feet of whoever of them stands on a bridge within a
        // throw.
        targets.extend(
            view.seats
                .iter()
                .enumerate()
                .filter(|&(seat, seen)| seen.alive && team(seat) != racer.side)
                .map(|(_, seen)| seen.at.xz())
                .filter(|at| view.nav.on_plank(*at) && (DYNAMITE_NEAREST + 0.3..=DYNAMITE_FURTHEST - 0.3).contains(&at.distance(here))),
        );
        for at in targets {
            if let Some(worth) = self.worth(racer, view, at)
                && best.is_none_or(|(most, _)| worth > most)
            {
                best = Some((worth, at));
            }
        }
        let aimed = racer.goal == Goal::Sabotage && racer.aim.is_some_and(|(_, at)| best.is_some_and(|(_, best)| best.distance(at) < 4.0));
        let Some((_, at)) = best.filter(|&(worth, _)| aimed || worth >= racer.worth) else {
            racer.sighted = None;
            return;
        };
        // Seen just now: it throws in a moment, if it is still there to
        // throw at.
        let when = match racer.sighted {
            Some((seen, when)) if seen.distance(at) < 3.0 => when,
            _ => {
                let when = now + self.dice.around(SIGHTED.0, SIGHTED.1) * (1.3 - 0.6 * racer.lean.pvp);
                racer.sighted = Some((at, when));
                when
            }
        };
        if now < when {
            return;
        }
        let off = at - here;
        order.blast = Some(((-off.x).atan2(-off.y), off.length()));
        racer.aim = None;
        racer.sighted = None;
        if racer.goal == Goal::Sabotage {
            racer.think_at = now;
        }
    }

    /// What a stick of dynamite makes of the way the racer was going to
    /// walk (Bridge Wars' `Squad::mind_sticks`, the most of it): in the way
    /// of one, the plank under it within its blast, or the stick the other
    /// team's and near enough to knock it, it runs for the nearest ground
    /// clear of it once it has woken up to it, which the better do sooner;
    /// walking toward one lying on a bridge, it waits short of it.
    fn mind_sticks(&mut self, racer: &mut Racer, view: &View, walk: Vec2) -> Vec2 {
        if view.sticks.is_empty() {
            racer.alarm = None;
            return walk;
        }
        let (nav, now) = (view.nav, view.elapsed);
        let here = racer.body.at.xz();
        let out = nav.grid().height(here).is_none() && nav.on_plank(here);
        let clear = DYNAMITE_RADIUS + BODY + CLEAR_OF_BLAST;
        let heading = walk.normalize_or_zero();
        let in_mind = racer.alarm.map(|(at, _)| at);
        let minded = view
            .sticks
            .iter()
            .filter(|stick| {
                let away = stick.at.distance(here);
                let harms = stick.team != racer.side || out;
                let on_a_bridge = nav.on_plank(stick.at) && nav.grid().height(stick.at).is_none();
                let walking_at = heading.dot((stick.at - here).normalize_or_zero()) > 0.3;
                let known = in_mind.is_some_and(|at| at.distance(stick.at) < 0.2);
                (away < clear && harms) || (away < clear + SHORT_OF_BLAST + 6.0 && on_a_bridge && walking_at) || (away < clear + SHORT_OF_BLAST && harms && known)
            })
            .min_by(|a, b| a.at.distance_squared(here).total_cmp(&b.at.distance_squared(here)))
            .copied();
        let Some(stick) = minded else {
            racer.alarm = None;
            return walk;
        };
        let woken = match racer.alarm {
            Some((at, when)) if at.distance(stick.at) < 0.2 => when,
            _ => {
                let pvp = racer.lean.pvp;
                let soonest = STICK_WAKES.0 - STICK_WAKES_SOONER.0 * pvp;
                let latest = STICK_WAKES.1 - STICK_WAKES_SOONER.1 * pvp;
                let share = if stick.at.distance(here) >= clear { 0.5 } else { 1.0 };
                let when = now + self.dice.around(soonest, latest) * share;
                racer.alarm = Some((stick.at, when));
                when
            }
        };
        if now < woken {
            return walk;
        }
        let away = stick.at.distance(here);
        if away >= clear {
            let nearer = heading.dot((stick.at - here).normalize_or_zero()) > 0.0;
            return if away < clear + SHORT_OF_BLAST && nearer { Vec2::ZERO } else { walk };
        }
        let ring = clear + 0.3;
        let best = (0..16)
            .map(|k| stick.at + Vec2::from_angle(k as f32 * TAU / 16.0) * ring)
            .filter(|&to| nav.walkable_at(to) && nav.line(here, to))
            .min_by(|a, b| a.distance_squared(here).total_cmp(&b.distance_squared(here)));
        racer.path.clear();
        racer.path_to = None;
        match best {
            Some(to) => (to - here).normalize_or_zero() * RUN_SPEED,
            None => (here - stick.at).normalize_or(-heading) * RUN_SPEED,
        }
    }

    // ------------------------------------------------------------ walking

    /// Which way the racer walks, at `pace`, to get to `to`, along its way
    /// there, found again when where it is going has moved, every few
    /// times `again` seconds, when the map has lost a plank, and when the
    /// straight walk to the next point of it cannot be walked; a hole in a
    /// bridge on the way jumped, if it is short enough; weaving a little as
    /// it goes; and, getting nowhere for a while, thinking again (Bridge
    /// Wars' `Squad::walk_to`).
    fn walk_to(&mut self, racer: &mut Racer, view: &View, to: Vec2, pace: f32, again: f32) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = racer.body.at.xz();
        if now - racer.walked_at > 0.5 {
            racer.progress = (here, now);
        }
        racer.walked_at = now;
        if here.distance(to) <= ARRIVED {
            racer.path.clear();
            racer.progress = (here, now);
            return Vec2::ZERO;
        }
        let lost = now - racer.looked_at >= LOOK_EVERY && {
            racer.looked_at = now;
            let from = nav.nearest_walkable(here);
            racer.path.first().is_some_and(|&next| from.is_some_and(|from| !nav.line(from, next)))
        };
        let stale = racer.path_to.is_none_or(|was| was.distance(to) > 1.0)
            || (racer.path.is_empty() && (racer.path_version != nav.version || now - racer.path_at > 0.5))
            || now - racer.path_at > again.max(0.5) * 4.0
            || racer.path_cuts != nav.cuts
            || lost;
        if stale {
            racer.cross = None;
            racer.path = match nav.path(here, to) {
                Some(path) => path,
                None => match Self::crossing(nav, here, to, Self::reach_of_a_jump(pace)).filter(|_| pace >= RUN_SPEED * 0.9) {
                    Some(gap) => {
                        racer.cross = Some(gap);
                        nav.path(here, gap.a_in).unwrap_or_default()
                    }
                    None => Vec::new(),
                },
            };
            racer.path_to = Some(to);
            racer.path_version = nav.version;
            racer.path_at = now;
            racer.path_cuts = nav.cuts;
        }
        if let Some(gap) = racer.cross
            && here.distance(gap.a_in) <= 0.6
            && racer.body.free(nav)
        {
            racer.cross = None;
            racer.path.clear();
            racer.path_to = None;
            racer.progress = (here, now);
            if gap.length() > Self::reach_of_a_jump(pace) {
                return Vec2::ZERO;
            }
            racer.leaping = Some(Leap {
                to: gap.b_in,
                pace,
                until: now + 2.0,
                aloft: false,
            });
            return (gap.b_in - here).normalize_or_zero() * pace;
        }
        while racer.path.first().is_some_and(|next| next.distance(here) <= ARRIVED) {
            racer.path.remove(0);
        }
        if here.distance(racer.progress.0) > 0.75 {
            racer.progress = (here, now);
        } else if now - racer.progress.1 > 3.0 {
            racer.progress = (here, now);
            if let Some(&next) = racer.path.first() {
                racer.stalls += 1;
                racer.stalled = Some((here, next));
                if !Self::gets_on(nav, &racer.body, (next - here).normalize_or_zero() * pace) {
                    racer.pinned += 1;
                }
            }
            racer.path.clear();
            racer.path_to = None;
            racer.think_at = now;
        }
        match racer.path.first() {
            Some(&next) => {
                let off = next - here;
                let far = off.length();
                racer.weave += (1.0 / 60.0) * (1.9 + 0.6 * racer.lean.weave);
                let room = if nav.grid().height(here).is_none() { 0.35 } else { 1.0 };
                let swing = (racer.weave.sin() + 0.5 * (racer.weave * 2.3).sin()) * 0.16 * racer.lean.weave * room * (far / 3.0).min(1.0);
                let straight = off.normalize_or_zero() * pace.min(far / (1.0 / 60.0));
                let woven = Vec2::from_angle(swing).rotate(straight);
                if Self::slit_ahead(nav, here, next)
                    && let Some(over) = self.over_a_slit(racer, nav, next, now)
                {
                    over
                } else if Self::gets_on(nav, &racer.body, woven) {
                    woven
                } else {
                    if !Self::gets_on(nav, &racer.body, straight) {
                        racer.looked_at = f32::MIN;
                    }
                    straight
                }
            }
            None => Vec2::ZERO,
        }
    }

    /// Whether the straight walk from `here` to `to` comes, in its next
    /// stride, to a stretch the map's squares have as walked and that has
    /// no ground under it: a slit between two bridges ([`Crew::over_a_slit`]).
    fn slit_ahead(nav: &Nav, here: Vec2, to: Vec2) -> bool {
        let way = (to - here).normalize_or_zero();
        let far = here.distance(to);
        [0.25, 0.5, 0.75].into_iter().filter(|ahead| *ahead < far + 0.5).any(|ahead| {
            let at = here + way * ahead;
            nav.walkable_at(at) && nav.ground(at).is_none()
        })
    }

    /// Over a slit of nothing in the racer's way to `to`, with a jump, if
    /// there is one a jump across with ground past it: the way it goes,
    /// having set off. Where two bridges meet that do not share a rung, the
    /// one coming to the other at a slant or stopping short of it, there
    /// can be a hand's width of the void between their planks that the
    /// map's squares, half a metre across, have as walked (since
    /// 2026-11-18: one of the computer's stood at such a slit on its way
    /// onto the golden island, walking at it, for the rest of a game).
    /// Anyone would step over it.
    fn over_a_slit(&mut self, racer: &mut Racer, nav: &Nav, to: Vec2, now: f32) -> Option<Vec2> {
        let here = racer.body.at.xz();
        if !racer.body.free(nav) {
            return None;
        }
        let ahead = here + (to - here).normalize_or_zero() * (here.distance(to) + 1.5).max(2.5);
        let (_, down) = nav.leap(here, ahead, Self::reach_of_a_jump(RUN_SPEED))?;
        racer.leaping = Some(Leap {
            to: down,
            pace: RUN_SPEED,
            until: now + 2.0,
            aloft: false,
        });
        racer.path.clear();
        racer.path_to = None;
        Some((down - here).normalize_or_zero() * RUN_SPEED)
    }

    /// Whether `body`, standing where one can walk, gets anywhere by
    /// walking `walk` for a frame (Bridge Wars' own).
    fn gets_on(nav: &Nav, body: &Body, walk: Vec2) -> bool {
        let here = body.at.xz();
        if !nav.walkable_at(here) || !body.free(nav) {
            return true;
        }
        let whole = here + walk * (1.0 / 30.0);
        [whole, Vec2::new(whole.x, here.y), Vec2::new(here.x, whole.y)]
            .into_iter()
            .any(|to| to != here && nav.walkable_at(to) && nav.ground(to).is_some_and(|there| there - body.at.y <= STEP))
    }
}

/// How much it matters to the racer's team just now to set the other team's
/// bridge back, as so many times as most: one with the race even or its own
/// team ahead; more the further the other team is ahead of it, and the
/// nearer the golden island the other team's bridge has got (mine: the team
/// behind is the one with something to cut, which keeps a race close).
fn urgency(racer: &Racer, view: &View) -> f32 {
    let side = usize::from(racer.side) % TEAMS;
    let (mine, theirs) = (view.across[side], view.across[1 - side]);
    let behind = ((theirs - mine) / 0.15).clamp(0.0, 1.0);
    let close = ((theirs - 0.6) / 0.3).clamp(0.0, 1.0);
    1.0 + 0.6 * behind + 0.4 * close
}

/// Whether `at` is over the plank between the rungs `a` and `b`: within its
/// width of its middle line, between the two or a step past either.
fn over_plank(at: Vec2, a: Vec2, b: Vec2) -> bool {
    let along = b - a;
    let length = along.length();
    if length < 1e-4 {
        return false;
    }
    let way = along / length;
    let off = at - a;
    (-0.3..=length + 0.3).contains(&off.dot(way)) && off.perp_dot(way).abs() <= PLANK_HALF_WIDTH
}

/// How surely a punch thrown from `from` carries whoever stands `at` off
/// the land and into the void, from 0 to 1: by whether there is ground
/// where a knock would put them down, as far on as one carries ([`CARRY`];
/// Bridge Wars' own).
fn flung(nav: &Nav, from: Vec2, at: Vec2) -> f32 {
    let away = (at - from).normalize_or_zero();
    if away == Vec2::ZERO {
        return 0.0;
    }
    CARRY.iter().filter(|far| nav.ground(at + away * **far).is_none()).count() as f32 / CARRY.len() as f32
}
