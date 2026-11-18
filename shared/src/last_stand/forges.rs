//! Last Stand's two generators (Hajun: "You can upgrade generators
//! progressively, at first u just start with silver and u gradually also
//! get the other 3 minerals (gold, sapphire, amethyst)"), and everything
//! lying about with what they make: on Bridge Wars' own [`Pile`], by the
//! same rules for coming up, lying there and being picked up.

use super::map::{PAD_HALF, PAD_TOP, PADS};
use crate::Item;
use crate::bridge_wars::{Pile, Piece};

/// How long each generator takes over one of each item, in seconds, by
/// [`Item::index`], and how much more or less each one may take: silver as
/// Bridge Wars' does, the rest a little slower than its (the numbers are
/// ours; two pads make them, where Bridge Wars' teams have one each).
pub const INTERVALS: [f32; 4] = [2.0, 8.0, 30.0, 60.0];
pub const SPREADS: [f32; 4] = [0.5, 1.5, 4.0, 8.0];
/// The most of each item lying on one pad before it makes no more until
/// some are taken: half of Bridge Wars' silver, and its gold, a sapphire
/// fewer and its amethyst.
pub const CAPS: [usize; 4] = [24, 6, 3, 2];
/// How many times as fast each step of the generator past amethyst makes
/// everything (the design's Overclock, endless).
pub const OVERCLOCK: f32 = 1.25;

/// Both pads' generators, and everything lying anywhere in the game, theirs
/// and what zombies drop: the server's online and the device's own offline,
/// run by the same rules. It rolls no dice of its own.
#[derive(Clone, Debug, Default)]
pub struct Forges {
    pile: Pile,
    /// When each pad next comes to make each item, in seconds into the
    /// fight, by [`Item::index`]: none for an item it does not make yet.
    due: [[Option<f32>; 4]; 2],
}

impl Forges {
    /// Both generators, nothing made yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Everything lying about, and what picks it up.
    pub fn pile(&self) -> &Pile {
        &self.pile
    }

    pub fn pile_mut(&mut self) -> &mut Pile {
        &mut self.pile
    }

    /// Which items the generators make at `level` of the upgrade shop's
    /// Generator, by [`Item::index`]: silver from the start, then gold,
    /// sapphire and amethyst, one more a step.
    pub fn makes(level: u16) -> [bool; 4] {
        [true, level >= 1, level >= 2, level >= 3]
    }

    /// How many times as fast as at first they make everything at `level`:
    /// [`OVERCLOCK`] for every step past amethyst.
    pub fn speed(level: u16) -> f32 {
        OVERCLOCK.powf(f32::from(level.saturating_sub(3).min(60)))
    }

    /// How many times as fast they make everything, and as much more they
    /// hold, with `people` people in the game: four tenths more for each
    /// after the first (mine: swords and shields are each person's own, and
    /// four people living off what one would have were each a quarter as
    /// well armed, and got no further than two).
    pub fn shared(people: usize) -> f32 {
        0.6 + 0.4 * people.clamp(1, 4) as f32
    }

    /// The number a thing made by `pad`'s generator of `item` is said to be
    /// of ([`Piece::generator`]), so that each pad's each item is counted on
    /// its own against its [`CAPS`].
    pub fn generator(pad: usize, item: Item) -> u8 {
        (pad * Item::ALL.len() + item.index()) as u8
    }

    /// Runs both generators up to `elapsed` seconds into the fight, the team
    /// having the Generator at `level`, with `people` people in the game:
    /// what they made. Each item a pad makes comes round a whole wait on
    /// from the last time, or from when it began to make it, and makes one
    /// unless the pad holds its cap; either way its next wait begins (Bridge
    /// Wars' rule: one at the most a go, and never a wait that has already
    /// passed).
    pub fn run(&mut self, elapsed: f32, level: u16, people: usize, dice: &mut dyn FnMut() -> f32) -> Vec<Piece> {
        let (makes, shared) = (Self::makes(level), Self::shared(people));
        let speed = Self::speed(level) * shared;
        let mut made = Vec::new();
        for (pad, at) in PADS.iter().enumerate() {
            for item in Item::ALL {
                let index = item.index();
                if !makes[index] {
                    self.due[pad][index] = None;
                    continue;
                }
                let wait = |dice: &mut dyn FnMut() -> f32| (INTERVALS[index] + (dice() * 2.0 - 1.0) * SPREADS[index]) / speed;
                let due = match self.due[pad][index] {
                    Some(due) => due,
                    None => *self.due[pad][index].insert(elapsed + wait(dice)),
                };
                if elapsed < due {
                    continue;
                }
                self.due[pad][index] = Some((due + wait(dice)).max(elapsed));
                let generator = Self::generator(pad, item);
                if self.pile.held(generator) as f32 >= (CAPS[index] as f32 * shared).round() {
                    continue;
                }
                made.push(self.pile.make(generator, item, *at, PAD_HALF, PAD_TOP, elapsed, dice));
            }
        }
        made
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;

    use super::*;
    use crate::bots::Dice;

    #[test]
    fn silver_from_the_start_and_each_step_brings_the_next_mineral_in_order_and_then_more_of_everything() {
        let mut dice = Dice::new(3);
        let mut roll = || dice.roll();
        // Ten minutes at each level, taking nothing: what lies on the pads.
        let lying = |level: u16, roll: &mut dyn FnMut() -> f32| {
            let mut forges = Forges::new();
            let mut made = [0usize; 4];
            let mut elapsed = 0.0;
            while elapsed < 600.0 {
                elapsed += 0.1;
                for piece in forges.run(elapsed, level, 1, roll) {
                    made[piece.item.index()] += 1;
                    assert_eq!(piece.ground, PAD_TOP);
                    assert!(PADS.iter().any(|pad| (piece.at - *pad).abs().max_element() < PAD_HALF), "{:?}", piece.at);
                }
            }
            for pad in 0..2 {
                for item in Item::ALL {
                    assert!(forges.pile().held(Forges::generator(pad, item)) <= CAPS[item.index()]);
                }
            }
            made
        };
        let mut at = |level| lying(level, &mut roll);
        let none = at(0);
        assert!(none[0] > 0 && none[1..] == [0, 0, 0], "{none:?}");
        let gold = at(1);
        assert!(gold[1] > 0 && gold[2..] == [0, 0], "{gold:?}");
        let sapphire = at(2);
        assert!(sapphire[2] > 0 && sapphire[3] == 0, "{sapphire:?}");
        let amethyst = at(3);
        assert!(amethyst[3] > 0, "{amethyst:?}");
        // Taking everything as it comes: an overclocked generator makes more
        // of everything than one that is not.
        let taken = |level: u16, roll: &mut dyn FnMut() -> f32| {
            let mut forges = Forges::new();
            let mut taken = [0usize; 4];
            let mut elapsed = 0.0;
            while elapsed < 300.0 {
                elapsed += 0.1;
                forges.run(elapsed, level, 1, roll);
                for pad in PADS {
                    for piece in forges.pile_mut().pick_up(Vec3::new(pad.x, PAD_TOP, pad.y), elapsed) {
                        taken[piece.item.index()] += 1;
                    }
                }
            }
            taken
        };
        let (plain, fast) = (taken(3, &mut roll), taken(7, &mut roll));
        for item in 0..4 {
            assert!(fast[item] > plain[item], "item {item}: {fast:?} against {plain:?}");
        }
        // Silver about every two seconds on each pad: 300 in five minutes.
        assert!((250..=350).contains(&plain[0]), "{plain:?}");
        assert!(Forges::speed(u16::MAX).is_finite());
    }
}
