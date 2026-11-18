//! Last Stand's two shops (Hajun: "You can buy items from item shop and also
//! buy upgrades from upgrade shop. I haven't rly thought of what the items
//! should be or what the upgrades should be, but they should make sense and
//! allow fun for long game so it makes sense. I.e. Endless upgrades"): what
//! each sells, at what price, and what a blow does with them. The catalogue
//! and every number are ours.
//!
//! * **The item shop**, at the red stand, sells what is yours, and kept
//!   when you die (mine: the game is played together against the zombies,
//!   and a death costs the wait and maybe the anchor already): Bridge Wars'
//!   three swords and three shields, each only better than what you have,
//!   and its things to use, on your buttons as in Bridge Wars, but for the
//!   Bridge Builder, there being no bridges.
//! * **The upgrade shop**, at the blue one, sells what is the whole team's,
//!   bought by anyone out of their own purse: six lines, every step dearer
//!   than the last by about half as much again; endless but for the
//!   protection and the luck, so that there is always something worth
//!   saving for. Each line is priced in a mineral of its own all the way
//!   ([`price`]), so that everything the generators make has a use all
//!   game: the anchor's mending and the protection in silver, the
//!   sharpness and the luck in gold, the beam in sapphire, the generators'
//!   own steps in each mineral in turn and then in amethyst. (The design had
//!   every line climb from silver to amethyst as it grew; in the stand-ins'
//!   games that left teams with thousands of silver and gold that bought
//!   nothing.)

use serde::{Deserialize, Serialize};

use super::StandUpgrades;
use crate::bridge_wars::{Shop, most_held};
use crate::{Item, Kit, Purse, Stack, Utility};

/// Something Last Stand's shops sell ([`Ask::Buy`](super::Ask::Buy)): its
/// own, not Bridge Wars' [`Ware`](crate::Ware), though its swords, shields
/// and things to use are Bridge Wars' own, on the same [`Kit`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ware {
    /// A sword, by its tier: 1 the basic one, 2 the silver, 3 the aura
    /// ([`SWORD_DAMAGE`]). Only one better than the one in hand.
    Sword(u8),
    /// A shield, by its tier, 1 to 3: a share of every blow kept off
    /// ([`SHIELD_SHARE`]). Only one better than the one worn.
    Shield(u8),
    /// One more of something to use, onto the button that holds its kind or
    /// the first empty one, as in Bridge Wars.
    Utility(Utility),
    /// The next step of both generators: gold, then sapphire, then amethyst,
    /// then faster and faster ([`Forges`](super::Forges)).
    Generator,
    /// Every blow of the team's [`SHARPNESS`] times as hard again: swords,
    /// fists, fireballs and sticks of dynamite.
    Sharpness,
    /// [`PROTECTION`] more off every zombie's blow on anyone of the team, up
    /// to [`PROTECTION_MOST`] steps.
    Protection,
    /// [`FORTIFY`] more health for the anchor, and all of it back.
    Fortify,
    /// The anchor's beam ([`BEAM_DAMAGE`] more for each step): it zaps the
    /// zombie nearest it.
    Beam,
    /// Every kill more likely to drop something, and better things, up to
    /// [`LUCKY_MOST`] steps ([`drop_chance`](super::drop_chance)).
    Lucky,
}

impl Ware {
    /// What the item shop sells, shelf by shelf: the swords, the shields,
    /// and the things to use.
    pub const ITEMS: [&'static [Ware]; 3] = [
        &[Ware::Sword(1), Ware::Sword(2), Ware::Sword(3)],
        &[Ware::Shield(1), Ware::Shield(2), Ware::Shield(3)],
        &[
            Ware::Utility(Utility::Healing),
            Ware::Utility(Utility::Speed),
            Ware::Utility(Utility::Fireball),
            Ware::Utility(Utility::Dynamite),
            Ware::Utility(Utility::Invisibility),
            Ware::Utility(Utility::Leap),
        ],
    ];

    /// What the upgrade shop sells, in the order its cards are shown.
    pub const UPGRADES: [Ware; 6] = [Ware::Generator, Ware::Sharpness, Ware::Protection, Ware::Fortify, Ware::Beam, Ware::Lucky];
}

/// Which shop sells `ware`: the item shop at the red stand, or the upgrade
/// shop at the blue.
pub fn sold_at(ware: Ware) -> Shop {
    match ware {
        Ware::Sword(_) | Ware::Shield(_) | Ware::Utility(_) => Shop::Items,
        Ware::Generator | Ware::Sharpness | Ware::Protection | Ware::Fortify | Ware::Beam | Ware::Lucky => Shop::Upgrades,
    }
}

/// What a blow takes off a zombie by its tier of sword: a bare fist, the
/// basic sword, the silver, the aura (the design's numbers).
pub const SWORD_DAMAGE: [f32; 4] = [6.0, 14.0, 22.0, 34.0];
/// What share of a zombie's blow each tier of shield keeps off.
pub const SHIELD_SHARE: [f32; 4] = [0.0, 0.10, 0.20, 0.30];
/// How many times as hard each step of Sharpness makes every blow of the
/// team's, on top of the steps before (the design's +12%). The zombies'
/// health grows faster than that, 1.22 times a wave
/// ([`HEALTH_GROWS`](super::waves::HEALTH_GROWS)): a step a wave still
/// leaves a sword about 8% weaker against them each wave, and keeping up
/// takes about a step and three quarters a wave, each dearer than the last,
/// which no team does for long ("endlessly stronger and stronger enemies").
/// The design had it for swords and fists alone; mine, the fireball and the
/// stick too, which would be spent by the fifteenth wave otherwise.
pub const SHARPNESS: f32 = 1.12;
/// What share of a zombie's blow each step of Protection keeps off, after
/// the shield, and how many steps there are.
pub const PROTECTION: f32 = 0.08;
pub const PROTECTION_MOST: u8 = 5;
/// How much more health a Fortify gives the anchor, all of it back with it.
pub const FORTIFY: f32 = 250.0;
/// What the anchor's beam takes off a zombie for each step of it: mine, 12
/// where the design had 20. In the tuning's games with 20, a plain player
/// alone who got past the first giant was carried by the beam as far as a
/// team of four went, to the eleventh wave; with 12, to the ninth.
pub const BEAM_DAMAGE: f32 = 12.0;
/// How many steps of Lucky Loot there are.
pub const LUCKY_MOST: u8 = 5;

/// The price of step `step` of an upgrade line priced in `item`: `first` of
/// it, and `grows` hundredths as many again for every step (150 is half as
/// much again), rounded to the nearest, a half up, and never more than a
/// count holds. Worked out in whole numbers, a step at a time (in
/// millionths, exactly for the first steps, where a price is small and a
/// part matters), never with a float's `powf`: the server works out what
/// it takes and every device what it shows on the card, and `powf` is not
/// the same on every machine, where two of these steps come out at a half
/// exactly (the beam's third, 2 x 1.5 x 1.5, and the generator's seventh,
/// 4 x 1.5 x 1.5 x 1.5), which one machine could round up and another
/// down.
fn in_one(item: Item, first: u16, grows: u64, step: u16) -> (Item, u16) {
    /// What one is worked out in: millionths.
    const WHOLE: u64 = 1_000_000;
    let most = u64::from(u16::MAX) * WHOLE;
    let mut count = u64::from(first) * WHOLE;
    // (Every line grows, so the count passes the most a count holds in a
    // few dozen steps at the most, and the loop with it.)
    for _ in 0..step {
        count = count * grows.max(101) / 100;
        if count >= most {
            return (item, u16::MAX);
        }
    }
    (item, ((count + WHOLE / 2) / WHOLE) as u16)
}

/// What `ware` costs someone with `kit` on a team with `team`: so many of an
/// item. Nothing if it is not to be had: a sword or a shield no better than
/// the one they have; something to use with as many of it on its button as
/// that holds, or a new kind with every button taken, or the Bridge
/// Builder, which is no use here; or the protection or the luck at its last
/// step. The rest have no last step.
pub fn price(ware: Ware, kit: &Kit, team: &StandUpgrades) -> Option<(Item, u16)> {
    match ware {
        Ware::Sword(1) if kit.sword < 1 => Some((Item::Silver, 12)),
        Ware::Sword(2) if kit.sword < 2 => Some((Item::Gold, 8)),
        Ware::Sword(3) if kit.sword < 3 => Some((Item::Sapphire, 6)),
        Ware::Shield(1) if kit.shield < 1 => Some((Item::Silver, 20)),
        Ware::Shield(2) if kit.shield < 2 => Some((Item::Gold, 10)),
        Ware::Shield(3) if kit.shield < 3 => Some((Item::Sapphire, 5)),
        Ware::Utility(Utility::Builder) => None,
        Ware::Utility(utility)
            if match kit.slot_of(utility) {
                Some(_) => kit.held(utility) < most_held(utility),
                None => kit.free_slot().is_some(),
            } =>
        {
            Some(match utility {
                Utility::Healing => (Item::Gold, 4),
                Utility::Speed => (Item::Sapphire, 1),
                Utility::Fireball => (Item::Silver, 30),
                Utility::Dynamite => (Item::Amethyst, 1),
                Utility::Invisibility => (Item::Sapphire, 2),
                Utility::Leap => (Item::Sapphire, 1),
                Utility::Builder => return None,
            })
        }
        // Gold, sapphire and amethyst are the design's steps; after them
        // the overclocking, from four amethyst, half as much again a step.
        Ware::Generator => Some(match team.generator {
            0 => (Item::Silver, 24),
            1 => (Item::Gold, 12),
            2 => (Item::Sapphire, 8),
            step => in_one(Item::Amethyst, 4, 150, step - 3),
        }),
        Ware::Sharpness => Some(in_one(Item::Gold, 6, 145, team.sharpness)),
        Ware::Protection if team.protection < PROTECTION_MOST => Some(in_one(Item::Silver, 20, 160, u16::from(team.protection))),
        // Mine: 1.6 times a step, where it was 1.35. A Fortify mends the
        // whole anchor, and at 1.35 a plain player alone, whose anchor is
        // hit hardest, bought one after about every wave, the anchor whole
        // again each time; at 1.6, after about two waves in three.
        Ware::Fortify => Some(in_one(Item::Silver, 24, 160, team.fortify)),
        Ware::Beam => Some(in_one(Item::Sapphire, 2, 150, team.beam)),
        Ware::Lucky if team.lucky < LUCKY_MOST => Some(in_one(Item::Gold, 5, 160, u16::from(team.lucky))),
        _ => None,
    }
}

/// Sells `ware` to someone with `purse` and `kit` on a team with `team`, if
/// it is to be had and they hold its [`price`], which comes out of the
/// purse: whether it is theirs, or the team's. What it does to the anchor
/// is the game's ([`Game::buy`](super::Game::buy)).
pub fn buy(ware: Ware, purse: &mut Purse, kit: &mut Kit, team: &mut StandUpgrades) -> bool {
    let Some((item, cost)) = price(ware, kit, team) else {
        return false;
    };
    let held = &mut purse[item.index()];
    if *held < cost {
        return false;
    }
    *held -= cost;
    match ware {
        Ware::Sword(tier) => kit.sword = tier,
        Ware::Shield(tier) => kit.shield = tier,
        Ware::Utility(utility) => {
            // Onto the button its kind is on, or the first empty one: the
            // price was only there with one of the two.
            if let Some(slot) = kit.slot_of(utility).or(kit.free_slot()) {
                kit.slots[slot].get_or_insert(Stack { utility, count: 0 }).count += 1;
            }
        }
        Ware::Generator => team.generator = team.generator.saturating_add(1),
        Ware::Sharpness => team.sharpness = team.sharpness.saturating_add(1),
        Ware::Protection => team.protection += 1,
        Ware::Fortify => team.fortify = team.fortify.saturating_add(1),
        Ware::Beam => team.beam = team.beam.saturating_add(1),
        Ware::Lucky => team.lucky += 1,
    }
    true
}

/// How far the team has taken `ware`, an upgrade, for the chat ("Sharpness
/// Lvl 3"); of an item, its tier, or none.
pub fn level(ware: Ware, team: &StandUpgrades) -> u16 {
    match ware {
        Ware::Sword(tier) | Ware::Shield(tier) => u16::from(tier),
        Ware::Utility(_) => 0,
        Ware::Generator => team.generator,
        Ware::Sharpness => team.sharpness,
        Ware::Protection => u16::from(team.protection),
        Ware::Fortify => team.fortify,
        Ware::Beam => team.beam,
        Ware::Lucky => u16::from(team.lucky),
    }
}

/// `damage` dealt by anyone of a team with `team`: [`SHARPNESS`] times as
/// hard for every step of it.
pub fn sharpened(damage: f32, team: &StandUpgrades) -> f32 {
    damage * SHARPNESS.powf(f32::from(team.sharpness.min(400)))
}

/// What a blow by someone with `kit` on a team with `team` takes off a
/// zombie: their sword's, or a fist's, sharpened.
pub fn blow(kit: &Kit, team: &StandUpgrades) -> f32 {
    sharpened(SWORD_DAMAGE[usize::from(kit.sword.min(3))], team)
}

/// What gets through to someone with `kit` on a team with `team` of a
/// zombie's blow of `damage`: their shield's share kept off, and then the
/// team's protection's.
pub fn through(damage: f32, kit: &Kit, team: &StandUpgrades) -> f32 {
    let shield = SHIELD_SHARE[usize::from(kit.shield.min(3))];
    let protection = PROTECTION * f32::from(team.protection.min(PROTECTION_MOST));
    damage * (1.0 - shield) * (1.0 - protection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swords_and_shields_are_only_ever_better_and_things_to_use_fill_the_buttons() {
        let team = StandUpgrades::default();
        let mut kit = Kit::default();
        assert_eq!(price(Ware::Sword(1), &kit, &team), Some((Item::Silver, 12)));
        assert_eq!(price(Ware::Sword(3), &kit, &team), Some((Item::Sapphire, 6)));
        kit.sword = 2;
        assert_eq!(price(Ware::Sword(1), &kit, &team), None);
        assert_eq!(price(Ware::Sword(2), &kit, &team), None);
        assert!(price(Ware::Sword(3), &kit, &team).is_some());
        kit.shield = 3;
        assert!((1..=3).all(|tier| price(Ware::Shield(tier), &kit, &team).is_none()));
        // No bridges, so no Bridge Builder; nothing past the third tier.
        assert_eq!(price(Ware::Utility(Utility::Builder), &kit, &team), None);
        assert_eq!(price(Ware::Sword(4), &Kit::default(), &team), None);
        // Three fireballs on a button at the most; five kinds on five.
        let mut purse: Purse = [999; 4];
        let mut team = StandUpgrades::default();
        let mut kit = Kit::default();
        for _ in 0..3 {
            assert!(buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team));
        }
        assert!(!buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team));
        for utility in [Utility::Healing, Utility::Speed, Utility::Dynamite, Utility::Invisibility] {
            assert!(buy(Ware::Utility(utility), &mut purse, &mut kit, &mut team));
        }
        assert!(!buy(Ware::Utility(Utility::Leap), &mut purse, &mut kit, &mut team), "a sixth kind with five buttons full");
        assert_eq!(purse[Item::Silver.index()], 999 - 90);
        // Not without the price.
        let mut poor: Purse = [11, 0, 0, 0];
        assert!(!buy(Ware::Sword(1), &mut poor, &mut Kit::default(), &mut StandUpgrades::default()));
        assert_eq!(poor, [11, 0, 0, 0]);
    }

    #[test]
    fn every_upgrade_s_steps_are_dearer_than_the_last_each_in_its_own_mineral_and_without_end() {
        for ware in Ware::UPGRADES {
            let mut team = StandUpgrades::default();
            let mut kit = Kit::default();
            let mut last: Option<(Item, u16)> = None;
            let mut steps = 0;
            while let Some(cost) = price(ware, &kit, &team) {
                if let Some(last) = last {
                    // Dearer, in its own mineral: but for the generator's
                    // first steps, each in a better one.
                    assert!(cost.0 == last.0 || ware == Ware::Generator, "{ware:?} step {steps}: {cost:?} after {last:?}");
                    assert!(cost.0.index() > last.0.index() || cost.1 > last.1 || cost.1 == u16::MAX, "{ware:?} step {steps}: {cost:?} after {last:?}");
                }
                last = Some(cost);
                let mut purse: Purse = [u16::MAX; 4];
                assert!(buy(ware, &mut purse, &mut kit, &mut team), "{ware:?} step {steps}");
                steps += 1;
                if steps > 60 {
                    break;
                }
            }
            match ware {
                Ware::Protection => assert_eq!(steps, usize::from(PROTECTION_MOST)),
                Ware::Lucky => assert_eq!(steps, usize::from(LUCKY_MOST)),
                // Endless: still something to buy.
                _ => assert!(steps > 60, "{ware:?} ended at {steps}"),
            }
        }
        // Every mineral has its lines: silver the anchor's mending and the
        // protection, gold the sharpness and the luck, sapphire the beam.
        let team = StandUpgrades::default();
        let firsts = Ware::UPGRADES.map(|ware| price(ware, &Kit::default(), &team).map(|(item, _)| item));
        assert_eq!(
            firsts,
            [Some(Item::Silver), Some(Item::Gold), Some(Item::Silver), Some(Item::Silver), Some(Item::Sapphire), Some(Item::Gold)]
        );
        // A long way on, the price saturates rather than overflowing.
        let far = StandUpgrades {
            generator: u16::MAX,
            sharpness: u16::MAX,
            fortify: 300,
            beam: 1000,
            ..default()
        };
        let at = |ware: Ware| price(ware, &Kit::default(), &far);
        assert_eq!(at(Ware::Generator), Some((Item::Amethyst, u16::MAX)));
        assert_eq!(at(Ware::Sharpness), Some((Item::Gold, u16::MAX)));
        assert_eq!(at(Ware::Beam), Some((Item::Sapphire, u16::MAX)));
        assert_eq!(at(Ware::Fortify), Some((Item::Silver, u16::MAX)));
        let mut purse: Purse = [u16::MAX; 4];
        let mut team = far;
        assert!(buy(Ware::Sharpness, &mut purse, &mut Kit::default(), &mut team));
        assert_eq!(team.sharpness, u16::MAX);
        let steps = [1, 2, 3].map(|generator| price(Ware::Generator, &Kit::default(), &StandUpgrades { generator, ..default() }));
        assert_eq!(steps, [Some((Item::Gold, 12)), Some((Item::Sapphire, 8)), Some((Item::Amethyst, 4))]);
    }

    fn default<T: Default>() -> T {
        T::default()
    }

    #[test]
    fn prices_are_worked_out_in_whole_numbers_to_the_same_coin_everywhere() {
        // Each line's first ten steps, as worked out in exact fractions
        // (first x grows^step, rounded half up) outside the game: the
        // server's and every device's.
        let steps = |ware: Ware, team: &dyn Fn(u16) -> StandUpgrades| -> Vec<u16> {
            (0..10).map(|step| price(ware, &Kit::default(), &team(step)).map_or(0, |(_, cost)| cost)).collect()
        };
        assert_eq!(steps(Ware::Beam, &|beam| StandUpgrades { beam, ..default() }), [2, 3, 5, 7, 10, 15, 23, 34, 51, 77]);
        assert_eq!(steps(Ware::Sharpness, &|sharpness| StandUpgrades { sharpness, ..default() }), [6, 9, 13, 18, 27, 38, 56, 81, 117, 170]);
        assert_eq!(steps(Ware::Fortify, &|fortify| StandUpgrades { fortify, ..default() }), [24, 38, 61, 98, 157, 252, 403, 644, 1031, 1649]);
        assert_eq!(
            steps(Ware::Generator, &|step| StandUpgrades { generator: step + 3, ..default() }),
            [4, 6, 9, 14, 20, 30, 46, 68, 103, 154]
        );
        let capped = |ware: Ware, team: &dyn Fn(u8) -> StandUpgrades| -> Vec<u16> {
            (0..5).map(|step| price(ware, &Kit::default(), &team(step)).map_or(0, |(_, cost)| cost)).collect()
        };
        assert_eq!(capped(Ware::Protection, &|protection| StandUpgrades { protection, ..default() }), [20, 32, 51, 82, 131]);
        assert_eq!(capped(Ware::Lucky, &|lucky| StandUpgrades { lucky, ..default() }), [5, 8, 13, 20, 33]);
        // The two that come out at a half exactly, rounded up: the beam's
        // third step (2 x 1.5 x 1.5) and the generator's seventh
        // (4 x 1.5 x 1.5 x 1.5).
        assert_eq!(price(Ware::Beam, &Kit::default(), &StandUpgrades { beam: 2, ..default() }), Some((Item::Sapphire, 5)));
        assert_eq!(price(Ware::Generator, &Kit::default(), &StandUpgrades { generator: 6, ..default() }), Some((Item::Amethyst, 14)));
    }

    #[test]
    fn swords_and_sharpness_hit_harder_and_shields_and_protection_keep_off_more() {
        let mut team = StandUpgrades::default();
        let mut kit = Kit::default();
        assert_eq!(blow(&kit, &team), 6.0);
        kit.sword = 3;
        assert_eq!(blow(&kit, &team), 34.0);
        team.sharpness = 2;
        assert!((blow(&kit, &team) - 34.0 * 1.12 * 1.12).abs() < 1e-3);
        assert_eq!(through(100.0, &Kit::default(), &StandUpgrades::default()), 100.0);
        kit.shield = 3;
        team.protection = PROTECTION_MOST;
        // The aura shield and all five steps: 42 of a hundred get through.
        assert!((through(100.0, &kit, &team) - 42.0).abs() < 1e-3);
        team.protection = 200;
        assert!((through(100.0, &kit, &team) - 42.0).abs() < 1e-3);
        assert!(blow(&kit, &StandUpgrades { sharpness: u16::MAX, ..team }).is_finite());
    }
}
