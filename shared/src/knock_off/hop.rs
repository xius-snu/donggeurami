//! Jumping in Knock Off (since 2026-11-27, Hajun: "i want to be able to
//! jump in the knock off games ... u should be able to jump all the time,
//! but jump will not go through during the time when the timer is up and u
//! are launched, but unblocked after again during any other time including
//! choosing the direction stage. jump is just for fun not rly for changing
//! the game mechanics", "and jump should be seen by other players too. and
//! also some bots can jump somtimes"): a hop on the spot, as high and as
//! long as the town's jump ([`hop_height`]), that nothing in the game goes
//! by: where everyone stands, slides and goes in is as it was.
//!
//! While the table waits a person's jump is the town's own, their body free
//! to move; once the game is played everyone's body is the game's, and a
//! jump is this hop, drawn over where the game has them ([`may_hop`]: not
//! once the aim's time is up, nor while everyone slides). The computer's
//! players hop too, now and then, each as keen as it is thrown to be
//! ([`Hops`], mine): one piece for the server's online and the device's
//! offline.

use crate::bots::Dice;
use crate::knock_off::KnockStage;

/// How fast a hop leaves the ice, in metres a second, and how hard it is
/// pulled back going up and coming down, in metres a second each second:
/// the town's jump (the game's `JUMP_SPEED`, `GRAVITY` and `FALL_GRAVITY`,
/// a test of the game's holds them the same), 1.64 m high, 0.7 s long.
pub const HOP_SPEED: f32 = 8.5;
pub const HOP_RISE_PULL: f32 = 22.0;
pub const HOP_FALL_PULL: f32 = 33.0;

/// How long a hop is, from leaving the ice to landing on it, in seconds.
pub fn hop_secs() -> f32 {
    let up = HOP_SPEED / HOP_RISE_PULL;
    let high = HOP_SPEED * HOP_SPEED / (2.0 * HOP_RISE_PULL);
    up + (2.0 * high / HOP_FALL_PULL).sqrt()
}

/// How high a hop is `secs` after it left the ice, in metres: up as the
/// town's jump goes up, and down as it comes down; nothing before it and
/// once it has landed.
pub fn hop_height(secs: f32) -> f32 {
    if !(secs > 0.0) {
        return 0.0;
    }
    let up = HOP_SPEED / HOP_RISE_PULL;
    if secs <= up {
        return HOP_SPEED * secs - 0.5 * HOP_RISE_PULL * secs * secs;
    }
    let high = HOP_SPEED * HOP_SPEED / (2.0 * HOP_RISE_PULL);
    let down = secs - up;
    (high - 0.5 * HOP_FALL_PULL * down * down).max(0.0)
}

/// Whether anyone on the ice may hop at `stage` of a game being played:
/// while everyone aims, as the ice breaks, and once it is over; not once the
/// aim's time is up and everyone's arrow is shown, nor while everyone
/// slides (Hajun: "jump will not go through during the time when the timer
/// is up and u are launched"). A hop under way as the aim's time runs out
/// lands as it would; it is never drawn in a slide.
pub fn may_hop(stage: KnockStage) -> bool {
    matches!(stage, KnockStage::Aim | KnockStage::Break | KnockStage::Over)
}

/// How many of the computer's players never hop at all, as a share (mine);
/// and how long the rest wait between hops on the whole, in seconds, each
/// thrown between these as it comes onto the ice; and how often a hop is
/// followed at once by another.
const NEVER: f32 = 0.4;
const KEEN: (f32, f32) = (3.0, 14.0);
const AGAIN: f32 = 0.22;

/// One of the computer's players, as keen to hop as it was thrown to be:
/// how long it waits between hops on the whole (none, never), how much
/// longer it waits now, and how long ago its hop under way began.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Hopper {
    keen: Option<f32>,
    wait: f32,
    since: Option<f32>,
}

/// The computer's players hopping now and then, by seat (mine; Hajun:
/// "some bots can jump somtimes"): some never do, the rest each as often as
/// it was thrown to, every wait thrown afresh about that, and now and then
/// a second hop straight after the first (Hajun's rule for the computer's
/// players: "if some thing becomes a definite if-then, then its not a real
/// human, but a program"). Only how high each is comes of it.
#[derive(Clone, Debug)]
pub struct Hops {
    dice: Dice,
    them: Vec<Option<Hopper>>,
}

impl Hops {
    /// Nobody yet, its dice thrown from `seed`.
    pub fn new(seed: u64) -> Self {
        Self {
            dice: Dice::new(seed),
            them: Vec::new(),
        }
    }

    /// The computer's player in `seat`, `dt` seconds on: how high it is
    /// over where it stands, in metres. It begins a hop only when it `may`
    /// (on the ice, at a stage that has hops, [`may_hop`]); one under way
    /// lands as it would.
    pub fn step(&mut self, seat: usize, dt: f32, may: bool) -> f32 {
        if self.them.len() <= seat {
            self.them.resize(seat + 1, None);
        }
        let dice = &mut self.dice;
        let it = self.them[seat].get_or_insert_with(|| {
            let keen = (dice.roll() >= NEVER).then(|| dice.between(KEEN.0, KEEN.1));
            Hopper {
                keen,
                wait: keen.map_or(f32::INFINITY, |keen| keen * dice.around(0.3, 1.9)),
                since: None,
            }
        });
        let dt = dt.clamp(0.0, 0.25);
        if let Some(since) = &mut it.since {
            *since += dt;
            if *since < hop_secs() {
                return hop_height(*since);
            }
            it.since = None;
        }
        let Some(keen) = it.keen else {
            return 0.0;
        };
        it.wait -= dt;
        if it.wait <= 0.0 && may {
            it.since = Some(0.0);
            it.wait = if dice.roll() < AGAIN { dice.between(0.05, 0.3) } else { keen * dice.around(0.3, 1.9) };
        }
        0.0
    }

    /// Whether the player in `seat` has this very step begun a hop: for a
    /// body that hops by jumping itself, as the device's do while the
    /// table waits.
    pub fn rising(&self, seat: usize) -> bool {
        self.them.get(seat).copied().flatten().is_some_and(|it| it.since == Some(0.0))
    }

    /// The player in `seat` has gone: it hops no more, and whoever sits
    /// there next is thrown afresh.
    pub fn leave(&mut self, seat: usize) {
        if let Some(it) = self.them.get_mut(seat) {
            *it = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hop_is_the_town_s_jump_up_and_down_again() {
        let secs = hop_secs();
        assert!((secs - 0.701).abs() < 0.01, "{secs}");
        let top = (0..200).map(|k| hop_height(k as f32 * secs / 200.0)).fold(0.0f32, f32::max);
        assert!((top - 1.642).abs() < 0.01, "{top}");
        assert_eq!(hop_height(0.0), 0.0);
        assert_eq!(hop_height(-1.0), 0.0);
        assert_eq!(hop_height(secs + 0.01), 0.0);
        assert!(hop_height(secs - 0.05) > 0.0);
        assert!(!may_hop(KnockStage::Reveal) && !may_hop(KnockStage::Slide));
        assert!(may_hop(KnockStage::Aim) && may_hop(KnockStage::Break) && may_hop(KnockStage::Over));
    }

    #[test]
    fn some_of_the_computer_s_players_hop_now_and_then_and_some_never() {
        let mut hopped = 0;
        let mut never = 0;
        for seed in 0..60 {
            let mut hops = Hops::new(seed);
            let mut began = 0;
            let mut was = 0.0;
            for _ in 0..(60 * 60) {
                let high = hops.step(1, 1.0 / 60.0, true);
                assert!((0.0..=1.7).contains(&high));
                if was == 0.0 && high > 0.0 {
                    began += 1;
                }
                was = high;
            }
            // A minute: none, or a handful to a few dozen.
            assert!(began <= 60, "seed {seed}: {began}");
            if began == 0 {
                never += 1;
            } else {
                hopped += 1;
            }
        }
        assert!(hopped > 20 && never > 10, "{hopped} hopped, {never} never");
        // Not where it may not, but a hop under way lands.
        let mut hops = Hops::new(3);
        for _ in 0..(60 * 120) {
            assert_eq!(hops.step(0, 1.0 / 60.0, false), 0.0);
        }
        let mut hops = Hops::new(5);
        let mut up = None;
        for frame in 0..(60 * 120) {
            if hops.step(2, 1.0 / 60.0, true) > 0.0 {
                up = Some(frame);
                break;
            }
        }
        if up.is_some() {
            assert!(hops.step(2, 0.1, false) > 0.0, "a hop under way goes on");
        }
    }
}
