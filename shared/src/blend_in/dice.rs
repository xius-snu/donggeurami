//! Blend In's dice ([`Sealed`], since 2026-11-18): thrown as the computer's
//! players' are ([`Dice`](crate::bots::Dice): `roll`, `between`, `bell`,
//! `around`), the same seed the same game, but sealed: nothing they throw
//! says what else they have thrown or will throw.
//!
//! Bridge Wars' dice are a xorshift, which keeps no secret: two of its
//! throws seen whole give away its state, and with it every throw it made
//! before and will make after. In Blend In what the dice throw is the very
//! secret the game is played for, which body is whose: the ids every body
//! goes by, which everyone is sent; the names and looks the crowd is dressed
//! in; where the hider is put; where each of the crowd walks next. With
//! Bridge Wars' dice, two of a round's ids gave away the dice that threw
//! them, and so which ids were the players' and where the hider was put
//! (the check of 2026-11-18). So each throw here is SipHash-2-4 (Aumasson
//! and Bernstein's keyed hash, which Rust's own hash maps were first built
//! on) of the throw's number, keyed by the seed: no number of throws seen
//! says anything of the rest without the seed, which nobody is sent, and
//! which trying every seed would take a computer centuries to find (mine).

/// Dice whose throws say nothing of one another without their seed: each
/// throw is SipHash-2-4, keyed by the seed, of which stream it is and how
/// many that stream has thrown. One seed throws as many streams as are
/// wanted, none of which says anything of the others.
#[derive(Clone, Debug)]
pub struct Sealed {
    key: (u64, u64),
    /// Which of a seed's streams this is, in the top sixteen bits of each
    /// throw's number.
    stream: u64,
    /// How many it has thrown.
    thrown: u64,
}

/// The second half of every key, the first being the seed: "Blend In", of
/// no meaning but to fill it.
const SALT: u64 = 0x426C_656E_6420_496E;
/// The bits of a throw's number that count the throws, below the stream's.
const COUNT: u64 = (1 << 48) - 1;

impl Sealed {
    /// Dice thrown from `seed`, the stream numbered `stream` of it.
    pub fn new(seed: u64, stream: u16) -> Self {
        Self {
            key: (seed, SALT),
            stream: u64::from(stream) << 48,
            thrown: 0,
        }
    }

    /// Sixty-four bits, every one as likely a 0 as a 1.
    pub fn bits(&mut self) -> u64 {
        let number = self.stream | (self.thrown & COUNT);
        self.thrown = self.thrown.wrapping_add(1);
        sip(self.key, number)
    }

    /// A number from 0 up to 1, in steps of a 2^24th, as Bridge Wars' dice
    /// throw it.
    pub fn roll(&mut self) -> f32 {
        (self.bits() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn between(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.roll()
    }

    /// A number about 0 as a bell rings them: two in three within 1 of it,
    /// and none further than 3.5.
    pub fn bell(&mut self) -> f32 {
        (self.roll() + self.roll() + self.roll() + self.roll() - 2.0) * 1.732
    }

    /// A number from `low` to `high`, mostly about the middle.
    pub fn around(&mut self, low: f32, high: f32) -> f32 {
        ((low + high) * 0.5 + (high - low) / 6.0 * self.bell()).clamp(low.min(high), high.max(low))
    }

    /// One of `count` things, by its number: 0 with none.
    pub fn pick(&mut self, count: usize) -> usize {
        ((self.roll() * count as f32) as usize).min(count.saturating_sub(1))
    }
}

/// SipHash-2-4 of the eight bytes of `message`, the lowest first, keyed by
/// `key`: as its makers' reference has it, two rounds for each block (the
/// message, then the last, which holds only the message's length) and four
/// to finish.
fn sip(key: (u64, u64), message: u64) -> u64 {
    let mut v = [
        key.0 ^ 0x736f_6d65_7073_6575,
        key.1 ^ 0x646f_7261_6e64_6f6d,
        key.0 ^ 0x6c79_6765_6e65_7261,
        key.1 ^ 0x7465_6462_7974_6573,
    ];
    fn rounds(v: &mut [u64; 4], count: usize) {
        for _ in 0..count {
            v[0] = v[0].wrapping_add(v[1]);
            v[1] = v[1].rotate_left(13) ^ v[0];
            v[0] = v[0].rotate_left(32);
            v[2] = v[2].wrapping_add(v[3]);
            v[3] = v[3].rotate_left(16) ^ v[2];
            v[0] = v[0].wrapping_add(v[3]);
            v[3] = v[3].rotate_left(21) ^ v[0];
            v[2] = v[2].wrapping_add(v[1]);
            v[1] = v[1].rotate_left(17) ^ v[2];
            v[2] = v[2].rotate_left(32);
        }
    }
    for block in [message, 8 << 56] {
        v[3] ^= block;
        rounds(&mut v, 2);
        v[0] ^= block;
    }
    v[2] ^= 0xff;
    rounds(&mut v, 4);
    v[0] ^ v[1] ^ v[2] ^ v[3]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_throw_is_siphash_2_4_as_its_makers_and_rust_have_it() {
        // The reference's own: the key the bytes 0 to 15, the message the
        // bytes 0 to 7.
        assert_eq!(sip((0x0706_0504_0302_0100, 0x0f0e_0d0c_0b0a_0908), 0x0706_0504_0302_0100), 0x93f5_f579_9a93_2462);
        // And as Rust's own SipHash-2-4 hashes eight bytes, for many keys
        // and messages.
        let mut dice = Sealed::new(5, 0);
        for _ in 0..200 {
            let (k0, k1, message) = (dice.bits(), dice.bits(), dice.bits());
            #[allow(deprecated)]
            let mut theirs = std::hash::SipHasher::new_with_keys(k0, k1);
            std::hash::Hasher::write(&mut theirs, &message.to_le_bytes());
            assert_eq!(sip((k0, k1), message), std::hash::Hasher::finish(&theirs));
        }
    }

    #[test]
    fn the_same_seed_throws_the_same_and_streams_and_seeds_throw_apart() {
        let throws = |seed: u64, stream: u16| -> Vec<u64> {
            let mut dice = Sealed::new(seed, stream);
            (0..64).map(|_| dice.bits()).collect()
        };
        assert_eq!(throws(9, 1), throws(9, 1));
        assert_ne!(throws(9, 1), throws(9, 2));
        assert_ne!(throws(9, 1), throws(10, 1));
        // A copy throws on as the dice it was copied from.
        let mut dice = Sealed::new(3, 1);
        dice.bits();
        let mut copy = dice.clone();
        assert_eq!(dice.bits(), copy.bits());
    }

    #[test]
    fn rolls_are_spread_evenly_and_every_bit_is_a_coin() {
        let mut dice = Sealed::new(77, 4);
        let rolls: Vec<f32> = (0..20_000).map(|_| dice.roll()).collect();
        assert!(rolls.iter().all(|roll| (0.0..1.0).contains(roll)));
        let mut tenths = [0u32; 10];
        for roll in &rolls {
            tenths[(roll * 10.0) as usize] += 1;
        }
        assert!(tenths.iter().all(|&count| (1_800..2_200).contains(&count)), "{tenths:?}");
        let mut ones = [0u32; 64];
        for _ in 0..4_000 {
            let bits = dice.bits();
            for (bit, count) in ones.iter_mut().enumerate() {
                *count += ((bits >> bit) & 1) as u32;
            }
        }
        assert!(ones.iter().all(|&count| (1_800..2_200).contains(&count)), "{ones:?}");
        // As Bridge Wars' dice throw the rest.
        for _ in 0..1_000 {
            assert!((2.0..=5.0).contains(&dice.between(2.0, 5.0)));
            assert!((2.0..=5.0).contains(&dice.around(2.0, 5.0)));
            assert!(dice.bell().abs() <= 3.5);
            assert!(dice.pick(3) < 3);
        }
        assert_eq!(dice.pick(0), 0);
    }
}
