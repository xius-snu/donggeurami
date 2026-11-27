//! The computer's players in Knock Off (Hajun: "fill with ai like
//! bridgewars"): each one of six kinds ([`PERSONAS`], mine), playing the
//! whole game by it, the server's online and the device's own offline.
//!
//! Each turn one weighs some dozens of moves ([`Mind::plan`]): going at each
//! of the others, straight at where they stand and at where it guesses they
//! will go, at several powers; to the middle; aside out of someone's way;
//! staying put. It plays each out as a slide ([`Slide`], the very one the
//! game plays) a few times, against guesses of how the turn will go: each
//! of the others standing still, going at whoever is nearest them, going
//! for the middle, getting out of someone's way, or anything at all, as
//! likely as each has been seen to do so this game ([`Habits`]); and its own
//! hand off by as much as its hand is ([`Guess`]), so that a move that only
//! works dead on is seen for one. It scores how each came out, for whoever
//! else went in, for itself staying on, for where it ends (well inside the
//! next turn's ice is safe, right up against someone is not), and takes one
//! by chance, the better ones likelier, as hot as its kind's choosing is.
//! And then it misses a little, by its kind's aim and touch. With fewer
//! left it thinks harder, and its kind's leaning to one of the others over
//! another fades: with one other left there is no choosing whom.
//!
//! **Nothing a computer player does is a definite if-then** (Hajun: "if
//! some thing becomes a definite if-then, then its not a real human, but a
//! program"): its kind is leanings, each thrown again every turn
//! ([`Temper`]); which move it takes is thrown, the better likelier; how far
//! it misses by is thrown; even the powers it weighs are thrown about what
//! would reach.
//!
//! It thinks while everyone aims, a few slides at a time
//! ([`Mind::ponder`]), so that the device's frame is never all of it, and
//! makes up its mind at a moment thrown for it ([`Mind::decide`]); someone
//! leaving before then, it thinks again without them ([`Mind::replan`]).

use std::f32::consts::TAU;
use std::sync::Arc;

use bevy::math::Vec2;

use super::{AREA_RADIUS, BODY, Floe, METRES_A_POWER, MOST_POWER, launch};
use crate::bots::Dice;
use crate::knock_off::{Ending, Slide};

/// How much one of the computer's players makes of each way a slide can
/// come out, as it weighs its moves: going into the sea itself, and putting
/// someone else in (so much each, as much one way as the other, the
/// shove's being thrown about its kind's); being the one left; standing
/// well inside the next turn's ice, at the most; leaving someone else near
/// its edge, at the most; and ending right up against someone, at the most
/// ([`Persona::wary`]). All mine, to tune by whole games.
const LIFE: f32 = 15.0;
const KO: f32 = 2.0;
const WIN: f32 = 2.0;
const SAFE: f32 = 4.0;
const HEM: f32 = 0.6;
const CROWD: f32 = 1.5;

/// How much more going into the sea itself costs for each one fewer on the
/// ice than four, as a share of [`LIFE`], thrown each turn between these
/// (mine, 2026-11-18): with two left, going in is losing the game, and a
/// check of the game found finals decided by one of the two sliding off on
/// its own, its target having moved, which felt cheap (a quarter of all
/// games' finals were, over a thousand of them). The place one more turn
/// on the ice is worth grows the same way: 10 coins from fourth to third,
/// 70 from second to first.
const STAKES: (f32, f32) = (0.6, 1.2);

/// What anyone is guessed to do before they have been seen to do anything,
/// as so many times seen: standing still, going at whoever is nearest them,
/// going for the middle, getting out of the way of whoever is nearest them
/// (aside or back), and anything else; going at someone the likeliest, and
/// standing about the least (nobody who has played twice expects to be left
/// alone). And how much likelier going at someone is guessed to be for each
/// one fewer on the ice than four: with two left, there is nobody else to go
/// at.
const FIRST_GUESS: [f32; 5] = [0.5, 1.8, 1.0, 1.0, 0.6];
const FEWER_LEFT: f32 = 1.0;
/// How much of what someone was seen to do before each reveal counts
/// beside what they did at it: what they did lately, most.
const REMEMBERS: f32 = 0.8;
const STILL: usize = 0;
const AT_NEAREST: usize = 1;
const TO_MIDDLE: usize = 2;
const OUT_OF_THE_WAY: usize = 3;
const ANYTHING: usize = 4;

/// One kind of computer player: what it makes of each way a slide can come
/// out, what it leans to beside, how it chooses and how it misses. Every
/// number is a leaning, thrown again every turn ([`Temper`]), never a rule.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Persona {
    pub name: &'static str,
    /// How much it makes of putting someone else in the sea, of minding
    /// going in itself, of ending the turn well inside the next turn's ice,
    /// and of leaving the others near its edge: each about 1.
    pub shoves: f32,
    pub minds: f32,
    pub keeps_in: f32,
    pub hems: f32,
    /// How much it minds ending a turn close to someone still in, where
    /// they can hit it hardest the turn after, about 1.
    pub wary: f32,
    /// What it leans to beside how a move plays out, each about 0, a score
    /// of 1 being about half of putting someone in: going at whoever is
    /// nearest, going at whoever stands nearest the edge, going for the
    /// middle, standing still, going where it guesses someone will be rather
    /// than where they are, and power (this much for a slide of the most
    /// over one of none).
    pub nearest: f32,
    pub edgiest: f32,
    pub middle: f32,
    pub still: f32,
    pub leads: f32,
    pub hard: f32,
    /// How far off someone may be before its leaning to going at them
    /// falls away, in metres: a turtle has it for whoever comes close.
    pub range: f32,
    /// How hot its choice is, in score: near nothing it takes what it
    /// reckons best; the more, the more often a worse one.
    pub heat: f32,
    /// How far off its aim goes, in radians, as a bell spreads; how far off
    /// its power, in steps, likewise; and how much more power than it meant
    /// it puts in, on average, in steps.
    pub wobble: f32,
    pub misjudges: f32,
    pub overdoes: f32,
    /// How many guesses at how the turn will go it plays each move out
    /// against, with four on the ice (twice as many with two): the more, the
    /// better it reads them.
    pub reads: u8,
    /// When it makes up its mind, as a share of the aim, the soonest and the
    /// latest.
    pub decides: (f32, f32),
}

/// Every kind there is (mine, from Hajun's "fill with ai like bridgewars":
/// as Bridge Wars' computer players each play as who they are), each a
/// game's own: four seats, six kinds, no two alike at a table.
pub const PERSONAS: [Persona; 6] = [
    // Goes for whoever is nearest, hard, and quick about it.
    Persona {
        name: "bully",
        shoves: 1.3,
        minds: 0.9,
        keeps_in: 0.8,
        hems: 0.6,
        wary: 0.6,
        nearest: 1.1,
        edgiest: 0.0,
        middle: 0.0,
        still: -0.4,
        leads: 0.0,
        hard: 0.8,
        range: 14.0,
        heat: 0.5,
        wobble: 0.08,
        misjudges: 0.7,
        overdoes: 0.4,
        reads: 6,
        decides: (0.08, 0.35),
    },
    // Goes for whoever stands nearest the edge, and does not miss much;
    // takes its time.
    Persona {
        name: "sniper",
        shoves: 1.5,
        minds: 1.1,
        keeps_in: 1.0,
        hems: 1.2,
        wary: 1.0,
        nearest: 0.0,
        edgiest: 1.0,
        middle: 0.0,
        still: 0.0,
        leads: 0.3,
        hard: 0.0,
        range: 30.0,
        heat: 0.2,
        wobble: 0.025,
        misjudges: 0.35,
        overdoes: 0.0,
        reads: 8,
        decides: (0.45, 0.9),
    },
    // Keeps to the middle with small moves, and shoves only whoever comes
    // close.
    Persona {
        name: "turtle",
        shoves: 0.8,
        minds: 1.5,
        keeps_in: 1.6,
        hems: 0.4,
        wary: 1.4,
        nearest: 0.6,
        edgiest: 0.0,
        middle: 0.8,
        still: 0.5,
        leads: 0.0,
        hard: -1.0,
        range: 4.5,
        heat: 0.35,
        wobble: 0.05,
        misjudges: 0.5,
        overdoes: -0.4,
        reads: 6,
        decides: (0.25, 0.7),
    },
    // Anything at all, and all over the place doing it.
    Persona {
        name: "wildcard",
        shoves: 1.0,
        minds: 0.8,
        keeps_in: 0.7,
        hems: 0.5,
        wary: 0.6,
        nearest: 0.0,
        edgiest: 0.0,
        middle: 0.0,
        still: 0.0,
        leads: 0.0,
        hard: 0.0,
        range: 30.0,
        heat: 2.0,
        wobble: 0.22,
        misjudges: 1.4,
        overdoes: 0.0,
        reads: 3,
        decides: (0.05, 0.95),
    },
    // Reads where the others will go, and is not there, or meets them
    // there; the last to make up its mind.
    Persona {
        name: "counter",
        shoves: 1.1,
        minds: 1.2,
        keeps_in: 1.2,
        hems: 0.8,
        wary: 1.2,
        nearest: 0.2,
        edgiest: 0.2,
        middle: 0.0,
        still: 0.0,
        leads: 0.8,
        hard: 0.0,
        range: 20.0,
        heat: 0.3,
        wobble: 0.05,
        misjudges: 0.5,
        overdoes: 0.0,
        reads: 10,
        decides: (0.5, 0.95),
    },
    // Big power, and often too much of it.
    Persona {
        name: "brawler",
        shoves: 1.4,
        minds: 0.7,
        keeps_in: 0.6,
        hems: 0.5,
        wary: 0.4,
        nearest: 0.4,
        edgiest: 0.2,
        middle: -0.2,
        still: -0.7,
        leads: 0.0,
        hard: 1.3,
        range: 30.0,
        heat: 0.6,
        wobble: 0.1,
        misjudges: 1.0,
        overdoes: 0.8,
        reads: 4,
        decides: (0.1, 0.5),
    },
];

/// One computer player's kind as it is this turn: each of its numbers
/// thrown about its kind's, so that nobody plays two turns the same.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct Temper {
    shoves: f32,
    minds: f32,
    keeps_in: f32,
    hems: f32,
    nearest: f32,
    edgiest: f32,
    middle: f32,
    still: f32,
    leads: f32,
    hard: f32,
    range: f32,
    heat: f32,
    wobble: f32,
    misjudges: f32,
    overdoes: f32,
    /// How far inside the next turn's edge is half as safe as the middle,
    /// in metres.
    cushion: f32,
    /// How much it minds ending the turn close to someone, and how close is
    /// close, in metres.
    wary: f32,
    close: f32,
    reads: usize,
    /// How much more going in costs for each one fewer on the ice than four
    /// ([`STAKES`]).
    stakes: f32,
}

impl Temper {
    fn throw(kind: &Persona, dice: &mut Dice) -> Self {
        let mut times = |of: f32, low: f32, high: f32| of * dice.around(low, high);
        let (shoves, minds, keeps_in, hems) = (times(kind.shoves, 0.7, 1.3), times(kind.minds, 0.75, 1.25), times(kind.keeps_in, 0.7, 1.3), times(kind.hems, 0.6, 1.4));
        let (range, heat, wobble, misjudges) = (times(kind.range, 0.8, 1.2), times(kind.heat, 0.6, 1.6), times(kind.wobble, 0.6, 1.4), times(kind.misjudges, 0.6, 1.4));
        let wary = times(kind.wary, 0.6, 1.4);
        let close = dice.around(2.5, 5.0);
        let mut about = |of: f32, by: f32| of + by * dice.bell();
        Self {
            shoves,
            minds,
            keeps_in,
            hems,
            wary,
            close,
            nearest: about(kind.nearest, 0.15),
            edgiest: about(kind.edgiest, 0.15),
            middle: about(kind.middle, 0.15),
            still: about(kind.still, 0.15),
            leads: about(kind.leads, 0.15),
            hard: about(kind.hard, 0.2),
            range,
            heat,
            wobble,
            misjudges,
            overdoes: about(kind.overdoes, 0.3),
            cushion: dice.around(1.0, 2.6),
            reads: (f32::from(kind.reads) + dice.bell()).round().clamp(1.0, 12.0) as usize,
            stakes: dice.around(STAKES.0, STAKES.1),
        }
    }
}

/// What a player has been seen to do at the reveals of a game, as the
/// computer's players guess by: how many times they stood still, went at
/// whoever was nearest them, went for the middle, got out of the way of
/// whoever was nearest them, or did anything else (a move that was partly
/// one and partly another counting partly to each), what they did lately
/// counting most.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(super) struct Habits([f32; 5]);

impl Habits {
    /// The player in `seat`, standing `at`, set off at `launch`, with
    /// everyone standing as `stand` has them and the middle of the ice
    /// ([`Floe::middle`]) at `middle`.
    pub(super) fn saw(&mut self, seat: usize, at: Vec2, launch: Vec2, stand: &[Option<Vec2>], middle: Vec2) {
        for seen in &mut self.0 {
            *seen *= REMEMBERS;
        }
        let Some(way) = launch.try_normalize() else {
            self.0[STILL] += 1.0;
            return;
        };
        // How nearly it went each way, as a number from 0 to 1 that falls
        // away quickly either side of dead on.
        let alike = |along: Option<Vec2>| along.map_or(0.0, |along| ((way.dot(along) + 1.0) * 0.5).powi(8));
        let to_nearest = nearest_to(seat, at, stand).and_then(|there| (there - at).try_normalize());
        let at_nearest = alike(to_nearest);
        let to_middle = alike((middle - at).try_normalize().filter(|_| at.distance(middle) > 0.5));
        let out_of_the_way = to_nearest.map_or(0.0, |to| alike(Some(to.perp())).max(alike(Some(-to.perp()))).max(alike(Some(-to))));
        let anything = 0.15;
        let all = at_nearest + to_middle + out_of_the_way + anything;
        self.0[AT_NEAREST] += at_nearest / all;
        self.0[TO_MIDDLE] += to_middle / all;
        self.0[OUT_OF_THE_WAY] += out_of_the_way / all;
        self.0[ANYTHING] += anything / all;
    }

    /// How likely each is, as so many times, with `in_play` on the ice:
    /// what it was guessed to be before anything was seen, and what has
    /// been; going at someone the likelier the fewer are left.
    fn odds(&self, in_play: usize) -> [f32; 5] {
        let mut odds: [f32; 5] = std::array::from_fn(|which| FIRST_GUESS[which] + self.0[which]);
        odds[AT_NEAREST] *= 1.0 + FEWER_LEFT * 4usize.saturating_sub(in_play) as f32;
        odds
    }
}

/// Where whoever stands nearest the player in `seat` (`at`) stands.
fn nearest_to(seat: usize, at: Vec2, stand: &[Option<Vec2>]) -> Option<Vec2> {
    stand
        .iter()
        .enumerate()
        .filter(|&(other, _)| other != seat)
        .filter_map(|(_, there)| *there)
        .min_by(|a, b| a.distance_squared(at).total_cmp(&b.distance_squared(at)))
}

/// Which of `odds` comes up: each as likely as it is big.
fn pick(dice: &mut Dice, odds: &[f32]) -> usize {
    let mut left = dice.roll() * odds.iter().sum::<f32>();
    for (which, weight) in odds.iter().enumerate() {
        if *weight > 0.0 && left < *weight {
            return which;
        }
        left -= weight;
    }
    odds.iter().rposition(|weight| *weight > 0.0).unwrap_or(0)
}

/// One move one of the computer's players weighs: which way (of length
/// one, or none), how hard, and what it leans to it for beside how it
/// plays out.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Move {
    way: Vec2,
    power: u8,
    leaning: f32,
}

/// One guess at how a turn will go: what everyone else does (a launch for
/// every seat; its own none), and how far its own hand will be off, as it
/// knows its hand: turned so many radians, and so many steps of power more.
/// Every move is played out against the same guesses, so that one move
/// does not look better than another by the luck of its guesses.
#[derive(Clone, Debug)]
struct Guess {
    launches: Vec<Vec2>,
    turned: f32,
    harder: f32,
}

/// A turn's thinking, as far as it has got: every move it weighs, every
/// guess at how the turn will go, and what each move has scored so far
/// against those it has been played out against.
#[derive(Clone, Debug)]
struct Thinking {
    me: usize,
    start: Vec<Option<Vec2>>,
    /// The island, how big its ice is this turn, and how big once it has
    /// broken at its end ([`super::size_in`]).
    floe: Arc<Floe>,
    size: f32,
    next: f32,
    moves: Vec<Move>,
    guesses: Vec<Guess>,
    scores: Vec<f32>,
    played: usize,
    launches: Vec<Vec2>,
}

/// One of the computer's players in a game, in its seat: its kind, its own
/// dice (so that when it thinks has nothing to do with what it thinks), and
/// how far its thinking has got this turn.
#[derive(Clone, Debug)]
pub(super) struct Mind {
    persona: &'static Persona,
    dice: Dice,
    temper: Temper,
    decides_at: f64,
    thinking: Option<Thinking>,
    decided: bool,
}

impl Mind {
    pub(super) fn new(persona: &'static Persona, seed: u64) -> Self {
        Self {
            persona,
            dice: Dice::new(seed),
            temper: Temper::default(),
            decides_at: 0.0,
            thinking: None,
            decided: true,
        }
    }

    pub(super) fn persona(&self) -> &'static Persona {
        self.persona
    }

    /// Whether it has made up its mind this turn.
    pub(super) fn decided(&self) -> bool {
        self.decided
    }

    /// When it makes up its mind this turn, by the game's clock.
    pub(super) fn decides_at(&self) -> f64 {
        self.decides_at
    }

    /// Where everyone stood as it laid out what it weighs this turn, while
    /// it thinks: for the tests.
    #[cfg(test)]
    pub(super) fn thinking_on(&self) -> Option<&[Option<Vec2>]> {
        self.thinking.as_ref().map(|thinking| thinking.start.as_slice())
    }

    /// A turn's aim begins, `since`, `secs` long: it throws who it is this
    /// turn and when it will make up its mind ([`Mind::decides_at`]), and
    /// lays out what it will weigh, for the player in `me` with everyone
    /// standing as `stand` has them (none for whoever is out), on `floe`'s
    /// ice at `size` that will be `next` once the turn is over.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn plan(&mut self, me: usize, stand: &[Option<Vec2>], floe: &Arc<Floe>, size: f32, next: f32, habits: &[Habits], since: f64, secs: f32) {
        self.temper = Temper::throw(self.persona, &mut self.dice);
        let (soonest, latest) = self.persona.decides;
        self.decides_at = since + f64::from(secs * self.dice.around(soonest, latest).clamp(0.03, 0.97));
        self.decided = false;
        self.lay_out(me, stand, floe, size, next, habits);
    }

    /// Someone has left the game while it thinks ([`super::Game::leave`]): if it has not
    /// made up its mind, it lays out what it weighs again, with everyone standing as `stand`
    /// now has them, as who it is this turn and making up its mind when it meant to. Its
    /// moves sized to stop against whoever left, and its guesses of what they would do, go
    /// with them: a slide meant to end against someone who is no longer there carries on
    /// past where they stood, toward the edge, and a way out of the way of nobody is no
    /// way to weigh. Once it has made up its mind it keeps its aim, as a person who has
    /// said they are ready keeps theirs.
    pub(super) fn replan(&mut self, me: usize, stand: &[Option<Vec2>], floe: &Arc<Floe>, size: f32, next: f32, habits: &[Habits]) {
        if self.decided {
            return;
        }
        self.lay_out(me, stand, floe, size, next, habits);
    }

    /// Lays out what it weighs this turn, from the start: every move, and every guess at
    /// how the turn will go, none of them played out yet.
    fn lay_out(&mut self, me: usize, stand: &[Option<Vec2>], floe: &Arc<Floe>, size: f32, next: f32, habits: &[Habits]) {
        let moves = self.moves(me, stand, floe, size, next, habits);
        // The fewer are left, the harder it thinks: each guess is cheaper to
        // play out, and each counts for more.
        let left = stand.iter().flatten().count().max(1);
        let reads = self.temper.reads * 4 / left;
        let middle = floe.middle(size);
        let guesses: Vec<Guess> = (0..reads.max(1)).map(|_| self.guess(me, stand, habits, middle)).collect();
        self.thinking = Some(Thinking {
            me,
            start: stand.to_vec(),
            floe: Arc::clone(floe),
            size,
            next,
            scores: vec![0.0; moves.len()],
            moves,
            guesses,
            played: 0,
            launches: vec![Vec2::ZERO; stand.len()],
        });
    }

    /// Thinks on, playing out moves against guesses until it has weighed
    /// them all or `budget` slides are spent (each it plays out takes one).
    pub(super) fn ponder(&mut self, budget: &mut usize) {
        let Some(thinking) = &mut self.thinking else { return };
        let reads = thinking.guesses.len();
        let all = thinking.moves.len() * reads;
        while thinking.played < all && *budget > 0 {
            let (which, guess) = (thinking.played / reads, thinking.played % reads);
            let (guess, move_) = (&thinking.guesses[guess], thinking.moves[which]);
            thinking.launches.copy_from_slice(&guess.launches);
            thinking.launches[thinking.me] = shaken(move_.way, move_.power, guess.turned, guess.harder);
            let ending = Slide::play_out(&thinking.start, &thinking.launches, &thinking.floe, thinking.size);
            thinking.scores[which] += judge(thinking.me, &thinking.start, &ending, &thinking.floe, thinking.next, &self.temper);
            thinking.played += 1;
            *budget -= 1;
        }
    }

    /// Makes up its mind: weighs whatever it has not yet, takes one of its
    /// moves by chance, the better likelier as hot as it chooses, and misses
    /// by a little. Its aim: the way (of length one, or none) and the power.
    pub(super) fn decide(&mut self) -> (Vec2, u8) {
        let mut all = usize::MAX;
        self.ponder(&mut all);
        self.decided = true;
        let Some(thinking) = self.thinking.take() else {
            return (Vec2::ZERO, 0);
        };
        let reads = thinking.guesses.len().max(1) as f32;
        let totals: Vec<f32> = thinking.moves.iter().zip(&thinking.scores).map(|(move_, score)| score / reads + move_.leaning).collect();
        let best = totals.iter().copied().fold(f32::MIN, f32::max);
        let heat = self.temper.heat.max(0.02);
        let odds: Vec<f32> = totals.iter().map(|total| ((total - best) / heat).exp()).collect();
        let chosen = thinking.moves[pick(&mut self.dice, &odds)];
        if chosen.power == 0 {
            return (Vec2::ZERO, 0);
        }
        // And it misses, a little, by its aim and its touch.
        let (turned, harder) = self.miss();
        let way = turn(chosen.way, turned);
        (way, (f32::from(chosen.power) + harder).round().clamp(1.0, f32::from(MOST_POWER)) as u8)
    }

    /// How far its hand is off this time: turned so many radians, and so
    /// many steps of power more (or, less than nothing, fewer).
    fn miss(&mut self) -> (f32, f32) {
        (self.dice.bell() * self.temper.wobble, self.temper.overdoes + self.dice.bell() * self.temper.misjudges)
    }

    /// Every move it weighs this turn, for the player in `me` on `floe`'s ice
    /// at `size`, `next` once it has broken: staying put; at each of the
    /// others, as far as just reaches them and some way further, and the same
    /// at where it guesses they will be; for the middle of the ice that will
    /// be left ([`Floe::middle`], its deepest spot), about as far as it is;
    /// and out of the way of whoever is nearest, aside either way or back, as
    /// far as the ice leaves room for. The powers are thrown about each.
    fn moves(&mut self, me: usize, stand: &[Option<Vec2>], floe: &Floe, size: f32, next: f32, habits: &[Habits]) -> Vec<Move> {
        let temper = self.temper;
        let dice = &mut self.dice;
        let here = stand.get(me).copied().flatten().unwrap_or(Vec2::ZERO);
        let others: Vec<(usize, Vec2)> = stand
            .iter()
            .enumerate()
            .filter(|&(seat, _)| seat != me)
            .filter_map(|(seat, there)| there.map(|there| (seat, there)))
            .collect();
        let nearest = others.iter().min_by(|a, b| a.1.distance_squared(here).total_cmp(&b.1.distance_squared(here))).map(|&(seat, _)| seat);
        let edgiest = others
            .iter()
            .map(|&(seat, there)| (seat, floe.depth(there, size)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(seat, _)| seat);
        let middle = floe.middle(size);
        let mut moves = vec![Move {
            way: Vec2::ZERO,
            power: 0,
            leaning: temper.still - 0.5 * temper.hard,
        }];
        let mut add = |way: Vec2, power: f32, leaning: f32| {
            let power = power.round().clamp(1.0, f32::from(MOST_POWER)) as u8;
            if let Some(way) = way.try_normalize() {
                moves.push(Move {
                    way,
                    power,
                    leaning: leaning + temper.hard * (f32::from(power) / f32::from(MOST_POWER) - 0.5),
                });
            }
        };
        // The power that just brings it up against someone this far off, as
        // so many steps of it.
        let touching = |far: f32| (far - 2.0 * BODY).max(0.0) / METRES_A_POWER;
        // Its leanings to one of the others over the rest are for choosing
        // between them: with two others half of them, and with one, whom
        // there is no choosing, none (it goes at them or not by how that
        // would go).
        let choosing = (others.len().saturating_sub(1) as f32 / 2.0).min(1.0);
        for &(seat, there) in &others {
            let far = there.distance(here);
            if far < 1e-3 {
                continue;
            }
            let keen = (temper.range / far).min(1.0) * choosing;
            let lean = keen
                * (if Some(seat) == nearest { temper.nearest } else { 0.0 } + if Some(seat) == edgiest { temper.edgiest } else { 0.0 });
            for extra in [dice.around(0.3, 2.0), dice.around(2.0, 4.5), dice.around(4.5, 8.0)] {
                add(there - here, touching(far) + extra, lean);
            }
            if let Some(lead) = lead(seat, there, stand, habits, dice, middle) {
                let far = lead.distance(here);
                if far > 1e-3 {
                    for extra in [dice.around(0.5, 2.5), dice.around(2.5, 6.0)] {
                        add(lead - here, touching(far) + extra, lean + temper.leads);
                    }
                }
            }
        }
        // For the middle of what will be left from anywhere but the middle
        // itself, where there is no way to it: how near is near enough is
        // thrown each turn.
        let to_middle = floe.middle(next) - here;
        let out = to_middle.length();
        if out > dice.around(0.2, 0.6) {
            for extra in [-1.0, 0.0, 1.0] {
                add(to_middle, out / METRES_A_POWER + extra + 0.3 * dice.bell(), temper.middle);
            }
        }
        if let Some(there) = nearest.and_then(|seat| stand[seat]) {
            let back = (here - there).try_normalize().unwrap_or(Vec2::X);
            let aside = back.perp();
            // Nobody steps four metres aside on a floe six across: as much
            // room as the disc as big as the ice.
            let room = (AREA_RADIUS * size / 8.0).clamp(0.3, 1.0);
            for power in [dice.around(1.0, 3.0), dice.around(3.0, 5.5)] {
                add(aside, power * room, 0.0);
                add(-aside, power * room, 0.0);
            }
            add(back, dice.around(1.0, 3.0) * room, 0.0);
        }
        moves
    }

    /// A guess at how this turn goes for the player in `me`: what everyone
    /// else does, each as they have been seen to, and how far its own hand
    /// is off.
    fn guess(&mut self, me: usize, stand: &[Option<Vec2>], habits: &[Habits], middle: Vec2) -> Guess {
        let (turned, harder) = self.miss();
        Guess {
            launches: self.others(me, stand, habits, middle),
            turned,
            harder,
        }
    }

    /// What everyone but the player in `me` does this turn, as a guess,
    /// each as they have been seen to, the middle of the ice being `middle`:
    /// their launches, by seat, its own none.
    fn others(&mut self, me: usize, stand: &[Option<Vec2>], habits: &[Habits], middle: Vec2) -> Vec<Vec2> {
        let dice = &mut self.dice;
        let in_play = stand.iter().flatten().count();
        let mut launches = vec![Vec2::ZERO; stand.len()];
        for (seat, at) in stand.iter().enumerate() {
            let Some(at) = *at else { continue };
            if seat == me {
                continue;
            }
            let power = |steps: f32| steps.round().clamp(0.0, f32::from(MOST_POWER)) as u8;
            launches[seat] = match pick(dice, &habits[seat].odds(in_play)) {
                STILL => Vec2::ZERO,
                AT_NEAREST => match nearest_to(seat, at, stand) {
                    Some(there) => {
                        let reach = (there.distance(at) - 2.0 * BODY).max(0.0) / METRES_A_POWER;
                        launch(there - at, power(reach + dice.around(1.0, 5.0)).max(1))
                    }
                    None => Vec2::ZERO,
                },
                TO_MIDDLE => launch(middle - at, power(at.distance(middle) / METRES_A_POWER + 0.5 * dice.bell())),
                OUT_OF_THE_WAY => match nearest_to(seat, at, stand).and_then(|there| (at - there).try_normalize()) {
                    // Aside, either way, more often than back.
                    Some(back) => {
                        let way = match (dice.roll() * 5.0) as u8 {
                            0 => back,
                            1 | 2 => back.perp(),
                            _ => -back.perp(),
                        };
                        launch(way, power(dice.around(1.0, 4.5)).max(1))
                    }
                    None => Vec2::ZERO,
                },
                _ => {
                    let angle = dice.between(0.0, TAU);
                    launch(Vec2::new(angle.cos(), angle.sin()), power(dice.between(1.0, 8.0)))
                }
            };
        }
        launches
    }
}

/// `way` turned `by` radians about the upright, anticlockwise seen from
/// above as x turns toward z.
fn turn(way: Vec2, by: f32) -> Vec2 {
    let (sin, cos) = by.sin_cos();
    Vec2::new(way.x * cos - way.y * sin, way.x * sin + way.y * cos)
}

/// The launch of a move `way` with `power` by a hand that is off: turned
/// `turned` radians, and `harder` steps of power more. Standing still is
/// standing still, however a hand shakes.
fn shaken(way: Vec2, power: u8, turned: f32, harder: f32) -> Vec2 {
    if power == 0 {
        return Vec2::ZERO;
    }
    launch(turn(way, turned), (f32::from(power) + harder).round().clamp(1.0, f32::from(MOST_POWER)) as u8)
}

/// Where it guesses the player in `seat`, standing `there`, will be once
/// they have slid a while, by one of the things they have been seen to do,
/// thrown as likely as each has been ([`Habits::odds`], as its guesses of
/// the whole turn are): some way toward whoever is nearest them, or toward
/// the middle of the ice, `middle`. None if the throw is standing still,
/// getting out of the way or anything at all.
fn lead(seat: usize, there: Vec2, stand: &[Option<Vec2>], habits: &[Habits], dice: &mut Dice, middle: Vec2) -> Option<Vec2> {
    let odds = habits[seat].odds(stand.iter().flatten().count());
    let share = dice.around(0.2, 0.5);
    match pick(dice, &odds) {
        AT_NEAREST => nearest_to(seat, there, stand).map(|target| there + (target - there) * share),
        TO_MIDDLE => Some(there + (middle - there) * share),
        _ => None,
    }
}

/// How a slide came out for the player in `me`, as it weighs it: so much
/// for each of the others in the sea or standing where the ice will break
/// (it will take them, as the game's own break does: off `floe`'s ice at
/// `next`, [`Floe::holds`]), and for leaving the rest near the edge; so much
/// off for being one of those itself, and otherwise so much for standing
/// well inside the next turn's ice, and for being the one left.
fn judge(me: usize, start: &[Option<Vec2>], ending: &Ending, floe: &Floe, next: f32, temper: &Temper) -> f32 {
    let gone = |seat: usize| ending.at[seat].is_none_or(|at| !floe.holds(at, next));
    let safety = |seat: usize| {
        let margin = ending.at[seat].map_or(0.0, |at| floe.depth(at, next));
        if margin > 0.0 { margin / (margin + temper.cushion) } else { 0.0 }
    };
    let mut score = 0.0;
    let mut standing = 0;
    let mut crowded = 0.0;
    for seat in 0..start.len() {
        if seat == me || start[seat].is_none() {
            continue;
        }
        if gone(seat) {
            score += KO * temper.shoves;
        } else {
            standing += 1;
            score += HEM * temper.hems * (1.0 - safety(seat));
            // Ending up against someone is ending where they can hit you
            // hardest the turn after.
            if let (Some(mine), Some(theirs)) = (ending.at[me], ending.at[seat]) {
                let near = (1.0 - mine.distance(theirs) / temper.close).max(0.0);
                crowded += near * near;
            }
        }
    }
    if gone(me) {
        // The fewer are left, the more it costs.
        let left = start.iter().flatten().count();
        score -= LIFE * temper.minds * (1.0 + temper.stakes * 4usize.saturating_sub(left) as f32);
    } else {
        score += SAFE * temper.keeps_in * safety(me) - CROWD * temper.wary * crowded;
        if standing == 0 {
            score += WIN;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use bevy::math::Vec3Swizzles;

    use super::*;
    use crate::knock_off::{RADIUS, size_after, waiting_spot};

    /// The round island 12 m out, as the game was played on until
    /// 2026-11-27.
    fn round() -> Arc<Floe> {
        Arc::new(Floe::round(RADIUS))
    }

    /// What a computer player of `kind` in seat 0 does from `stand`, on
    /// `floe`'s ice at `size` and then `next`, over so many minds of it,
    /// nothing having been seen of anyone yet: how many times it stands
    /// still, goes at someone, goes for the middle, gets out of the way, or
    /// anything else.
    fn what_it_does(kind: &'static Persona, stand: &[Option<Vec2>], floe: &Arc<Floe>, size: f32, next: f32, minds: u64) -> [u32; 5] {
        let habits = vec![Habits::default(); stand.len()];
        let here = stand[0].unwrap();
        let mut done = [0u32; 5];
        for seed in 0..minds {
            let mut mind = Mind::new(kind, seed * 7919 + 13);
            mind.plan(0, stand, floe, size, next, &habits, 0.0, AIM_SECS);
            let (way, power) = mind.decide();
            let at_someone = stand.iter().skip(1).flatten().map(|there| (*there - here).normalize().dot(way)).fold(-1.0f32, f32::max);
            let to_middle = (floe.middle(next) - here).try_normalize().map_or(-1.0, |to| to.dot(way));
            let which = match () {
                _ if power == 0 => 0,
                _ if at_someone > 0.95 => 1,
                _ if to_middle > 0.95 => 2,
                _ if at_someone < 0.4 => 3,
                _ => 4,
            };
            done[which] += 1;
        }
        done
    }

    const AIM_SECS: f32 = 10.0;

    #[test]
    fn with_one_other_left_the_careful_get_out_of_the_way_and_the_rough_go_at_them() {
        // Two left, either side of the middle of ice of 7 m that breaks to
        // 6: going at the other is going at someone who may well be going
        // at you, and two who meet head on there both go in.
        let stand = [Some(Vec2::new(2.0, 0.5)), Some(Vec2::new(-2.0, -0.5))];
        let done = |name: &str| {
            let kind = PERSONAS.iter().find(|kind| kind.name == name).unwrap();
            let done = what_it_does(kind, &stand, &round(), 7.0 / RADIUS, 6.0 / RADIUS, 100);
            println!("{name:<8} still {:>3} at them {:>3} middle {:>3} out of the way {:>3} else {:>3}", done[0], done[1], done[2], done[3], done[4]);
            done
        };
        for careful in ["turtle", "sniper", "counter"] {
            let done = done(careful);
            assert!(done[3] > 60, "{careful}: {done:?}");
        }
        let (brawler, turtle) = (done("brawler"), done("turtle"));
        assert!(brawler[1] > 2 * turtle[1] + 10, "the brawler {brawler:?}, the turtle {turtle:?}");
        // And nobody stands still to be hit, much.
        for kind in &PERSONAS {
            assert!(what_it_does(kind, &stand, &round(), 7.0 / RADIUS, 6.0 / RADIUS, 100)[0] < 25, "{}", kind.name);
        }
    }

    #[test]
    fn on_the_first_turn_nobody_slides_off_the_ice_on_purpose() {
        // Everyone on their spot as a game begins: whatever each does, it
        // does not set out for the sea, though its hand may carry it there.
        // On the round island, and on islands of blobmaker's, everyone where
        // they arrive.
        let floes: Vec<Arc<Floe>> = std::iter::once(round()).chain((0..6).map(|seed| Arc::new(Floe::thrown(seed + 100)))).collect();
        for (which, floe) in floes.iter().enumerate() {
            let stand: Vec<Option<Vec2>> = (0..4).map(|seat| Some(waiting_spot(seat, floe).0.xz())).collect();
            let here = stand[0].unwrap();
            let out = (here - floe.middle(1.0)).normalize_or_zero();
            for kind in &PERSONAS {
                let habits = vec![Habits::default(); 4];
                let mut outward = 0;
                for seed in 0..60 {
                    let mut mind = Mind::new(kind, seed + 1);
                    mind.plan(0, &stand, floe, 1.0, size_after(1), &habits, 0.0, 12.0);
                    let (way, power) = mind.decide();
                    // Out from where it stands, away from the middle, and a
                    // slide alone that would take it 3 m past the edge.
                    let reach = f32::from(power) * METRES_A_POWER;
                    if power > 0 && way.dot(out) > 0.5 && floe.leaves(here, way, reach, 1.0).is_some_and(|left| reach - left > 3.0) {
                        outward += 1;
                    }
                }
                // (The wildcard, all over the place, now and then a fourth.)
                assert!(outward <= 4, "island {which}, {}: {outward} of 60 set out for the sea", kind.name);
            }
        }
    }

    #[test]
    fn a_turn_s_thinking_is_a_matter_of_a_millisecond_or_so() {
        // The most a turn takes: four on the ice, the kind that reads the
        // others hardest, every move against every guess. Here, in a test
        // built with little optimising; a phone's build is optimised fully.
        let stand: Vec<Option<Vec2>> = vec![Some(Vec2::new(3.0, 2.0)), Some(Vec2::new(-4.0, 1.0)), Some(Vec2::new(1.0, -5.0)), Some(Vec2::new(-2.0, -2.0))];
        let habits = vec![Habits::default(); 4];
        let counter = PERSONAS.iter().find(|kind| kind.name == "counter").unwrap();
        let mut took = Vec::new();
        for seed in 0..20 {
            let mut mind = Mind::new(counter, seed);
            let began = Instant::now();
            mind.plan(0, &stand, &round(), 9.0 / RADIUS, 8.0 / RADIUS, &habits, 0.0, AIM_SECS);
            mind.decide();
            took.push(began.elapsed().as_secs_f64());
        }
        took.sort_by(f64::total_cmp);
        let (middle, most) = (took[took.len() / 2], took[took.len() - 1]);
        println!("a turn's thinking took {:.2} ms in the middle, {:.2} ms at the longest", middle * 1000.0, most * 1000.0);
        // The middle of twenty, not the longest: a test run beside a build,
        // as the tests are on this PC while others work, has the odd turn
        // held up by whatever else the machine is doing (once 20 ms and
        // more, 2026-11-18), which says nothing of the thinking itself.
        assert!(middle < 0.02, "{:.2} ms", middle * 1000.0);
    }

    #[test]
    fn thinking_again_without_someone_leaves_nothing_of_them() {
        let floe = round();
        let stand: Vec<Option<Vec2>> = (0..4).map(|seat| Some(waiting_spot(seat, &floe).0.xz())).collect();
        let habits = vec![Habits::default(); 4];
        let mut left = stand.clone();
        left[0] = None;
        for kind in &PERSONAS {
            let mut mind = Mind::new(kind, 17);
            mind.plan(1, &stand, &floe, 1.0, size_after(1), &habits, 100.0, AIM_SECS);
            let (when, temper) = (mind.decides_at, mind.temper);
            let mut budget = 5;
            mind.ponder(&mut budget);
            mind.replan(1, &left, &floe, 1.0, size_after(1), &habits);
            // Who it is this turn, and when it makes up its mind, as they were.
            assert_eq!((mind.decides_at, mind.temper, mind.decided), (when, temper, false), "{}", kind.name);
            let thinking = mind.thinking.as_ref().unwrap();
            assert_eq!(thinking.start, left, "{}", kind.name);
            assert_eq!(thinking.played, 0, "{}", kind.name);
            assert!(thinking.scores.iter().all(|score| *score == 0.0), "{}", kind.name);
            // Nobody guessed at for the seat that is empty.
            assert!(thinking.guesses.iter().all(|guess| guess.launches[0] == Vec2::ZERO), "{}", kind.name);
            // Once it has made up its mind, it keeps it.
            mind.decide();
            mind.replan(1, &stand, &floe, 1.0, size_after(1), &habits);
            assert!(mind.decided && mind.thinking.is_none(), "{}", kind.name);
        }
    }

    #[test]
    fn what_someone_does_is_seen_for_what_it_is() {
        // Seat 0 at (5, 0), its nearest an eighth of the turn off the way to
        // the middle: each way it might go is mostly one thing.
        let stand = [Some(Vec2::new(5.0, 0.0)), Some(Vec2::new(5.0 - 2.12, 2.12)), Some(Vec2::new(-6.0, 0.0))];
        let seen = |launch: Vec2| {
            let mut habits = Habits::default();
            habits.saw(0, stand[0].unwrap(), launch, &stand, Vec2::ZERO);
            habits.0
        };
        let mostly = |seen: [f32; 5]| (0..5).max_by(|&a, &b| seen[a].total_cmp(&seen[b])).unwrap();
        assert_eq!(seen(Vec2::ZERO)[STILL], 1.0);
        let at = seen(Vec2::new(-6.0, 6.0));
        assert!(mostly(at) == AT_NEAREST && at[AT_NEAREST] > 0.6, "{at:?}");
        let middle = seen(Vec2::new(-8.0, 0.0));
        assert!(mostly(middle) == TO_MIDDLE && middle[TO_MIDDLE] > 0.5, "{middle:?}");
        let aside = seen(Vec2::new(4.0, 4.0));
        assert!(mostly(aside) == OUT_OF_THE_WAY && aside[OUT_OF_THE_WAY] > 0.8, "{aside:?}");
        // What was seen lately counts most.
        let mut habits = Habits::default();
        habits.saw(0, stand[0].unwrap(), Vec2::ZERO, &stand, Vec2::ZERO);
        habits.saw(0, stand[0].unwrap(), Vec2::new(-6.0, 6.0), &stand, Vec2::ZERO);
        assert!(habits.0[STILL] < 1.0 && habits.0[AT_NEAREST] > 0.6, "{:?}", habits.0);
        // With fewer left, going at someone is guessed likelier.
        assert!(habits.odds(2)[AT_NEAREST] > habits.odds(4)[AT_NEAREST]);
    }
}
