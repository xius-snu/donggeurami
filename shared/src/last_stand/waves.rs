//! Last Stand's waves: who comes, how many, how strong, and what each
//! drops for whoever kills it. All of it is ours but "Zombies with increasing
//! difficulty/skill/creativity/health/attackdmg come in waves" and "random
//! chance droprate between silver gold sapphire amethyst that increases its
//! luck as progress goes" (Hajun), tuned by whole games of stand-in players
//! (this module's tests).

use serde::{Deserialize, Serialize};

use super::Knock;
use crate::Item;
use crate::bots::Dice;

/// How long after "Go!" the first wave comes, in seconds: time to take what
/// the generators have made and buy a sword. And the break between one wave
/// cleared and the next, to shop in.
pub const FIRST_WAVE: f32 = 20.0;
pub const BREAK: f32 = 25.0;

/// The most zombies standing at once, rising ones among them: the rest of a
/// wave waits below the rim until some are dead, so that a big wave comes as
/// a long fight, not one heap, and a phone draws no more than this many.
pub const MOST_STANDING: usize = 40;

/// What a zombie is. Each has its own size, pace and blow, and its own way
/// of coming at you; each comes from a wave on ([`Kind::first`]), and grows
/// stronger with every wave after ([`strength`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    /// The plain one: slow, and many of it.
    Walker,
    /// Fast and frail, from the third wave.
    Runner,
    /// A third bigger, slow, a heavy blow, and knocked a quarter as far.
    Brute,
    /// Keeps its distance and spits a slow ball that can be walked out of.
    Spitter,
    /// Leaps at whoever is a few metres off.
    Leaper,
    /// Twice as big, slow, a blow that hurts, knocked by nothing, and a hoard
    /// of things when it falls: every fifth wave, one more every ten.
    Giant,
}

impl Kind {
    /// Every kind, in the order they first come.
    pub const ALL: [Kind; 6] = [Kind::Walker, Kind::Runner, Kind::Brute, Kind::Giant, Kind::Spitter, Kind::Leaper];

    /// The first wave it comes in. The giant comes in every fifth wave and
    /// no other ([`giants`]). The design's, but the brute's, which is mine:
    /// from the fourth, not the fifth, so that each of the early waves
    /// brings one thing new and the fifth is the giant's alone.
    pub const fn first(self) -> u16 {
        match self {
            Kind::Walker => 1,
            Kind::Runner => 3,
            Kind::Brute => 4,
            Kind::Giant => 5,
            Kind::Spitter => 8,
            Kind::Leaper => 12,
        }
    }

    /// How big it is, against a body's size: what it is drawn as, what it
    /// takes up, and how far its blow reaches.
    pub const fn size(self) -> f32 {
        match self {
            Kind::Brute => 4.0 / 3.0,
            Kind::Giant => 2.0,
            _ => 1.0,
        }
    }

    /// What share of a knock it takes: a brute a quarter, a giant nothing.
    pub const fn knocked(self) -> f32 {
        match self {
            Kind::Brute => 0.25,
            Kind::Giant => 0.0,
            _ => 1.0,
        }
    }

    /// Its health, its blow's damage and its pace, in metres a second, in
    /// the first wave of all ([`strength`] for later ones). A spitter's blow
    /// is its ball's. The paces are the design's; the rest is mine:
    ///
    /// * the healths half the design's or less (40, 25, 160, 35, 50 and
    ///   1200 there: the brute's under a third, the giant's three eighths),
    ///   growing faster ([`HEALTH_GROWS`]), so that a spitter's comes to the
    ///   design's in the ninth wave, a walker's, a runner's and a leaper's in
    ///   the tenth, a giant's in the thirteenth (the fifteenth's giants are
    ///   the first past it) and a brute's in the sixteenth, and each outgrows
    ///   it after;
    /// * the blows lighter than the design's at first (8, 6, 22, 12, 14 and
    ///   40 there: three quarters of a walker's, five sixths of a runner's
    ///   and a spitter's, six sevenths of a leaper's, seven tenths of a
    ///   giant's and under two thirds of a brute's), growing faster
    ///   ([`DAMAGE_GROWS`]), so that a leaper's comes to the design's in the
    ///   tenth wave, a runner's and a spitter's in the eleventh, a walker's in
    ///   the seventeenth, a giant's in the twenty-first and a brute's in the
    ///   twenty-sixth.
    ///
    /// Tuned by the plain players of the module's tests, who land a blow a
    /// second (the waves each clears are in [`HEALTH_GROWS`]); a test holds
    /// these waves to the numbers.
    const fn base(self) -> (f32, f32, f32) {
        match self {
            Kind::Walker => (20.0, 6.0, 3.2),
            Kind::Runner => (12.0, 5.0, 6.0),
            Kind::Brute => (48.0, 14.0, 2.4),
            Kind::Spitter => (18.0, 10.0, 3.0),
            Kind::Leaper => (25.0, 12.0, 4.0),
            Kind::Giant => (450.0, 28.0, 2.0),
        }
    }

    /// How hard its blow knocks whoever it lands on: a brute's as a punch
    /// does, a giant's as one thrown on the run; the small ones' not at all
    /// (mine: a crowd of them would have the people holding the anchor
    /// batted about the stone, away from what they hold).
    pub const fn knocks(self) -> Knock {
        match self {
            Kind::Brute => Knock::Light,
            Kind::Giant => Knock::Heavy,
            _ => Knock::None,
        }
    }

    /// What share of its blow the anchor's crystal takes: a brute's three
    /// quarters (the design's "hits the anchor hard": half as hard again as
    /// the rest; with all of it, the first brutes broke the anchor in a third
    /// of a person's games alone), the rest only clawing at it, half (mine;
    /// a giant's blow is the biggest of all as it is, and two giants in the
    /// tenth wave with all of theirs broke every anchor there in the
    /// stand-ins' games).
    pub const fn on_the_anchor(self) -> f32 {
        match self {
            Kind::Brute => 0.75,
            _ => 0.5,
        }
    }

    /// What killing one is worth, before the wave counts ([`kill_score`]).
    pub const fn score(self) -> f32 {
        match self {
            Kind::Walker => 10.0,
            Kind::Runner => 15.0,
            Kind::Spitter => 25.0,
            Kind::Leaper => 30.0,
            Kind::Brute => 40.0,
            Kind::Giant => 250.0,
        }
    }

    /// Whether killing one is said in the chat, and by whom ("Mochi brought
    /// down a giant"): a giant alone (mine: a line for every walker would
    /// bury the chat). The one rule for both sides: the server tells the
    /// room of it ([`Tell::Slain`](super::Tell::Slain)) and a device playing
    /// offline says it of its own game's kill (since 2026-11-18; until then
    /// the rule was the server's alone, and offline said nothing).
    pub const fn announced(self) -> bool {
        matches!(self, Kind::Giant)
    }
}

/// How much stronger every zombie is for each wave past the first: its
/// health, its blow's damage and its pace, the last to at most three tenths
/// more than it began ("endless waves of endlessly stronger and stronger
/// enemies"). The pace's are the design's; the health's and the blow's are
/// mine (1.12 and 1.07 there), from lighter first waves ([`Kind::base`]):
/// the zombies outgrow every team in the end, a plain player alone (a blow
/// a second, the module's tests) mostly in the fifth to the tenth wave, six
/// or seven waves cleared in the middle of their games, and four plain
/// players together in the eleventh to the thirteenth, most in the twelfth.
/// As the game was first tuned, by a stand-in that landed two blows a
/// second, with 1.17 a wave and heavier first waves, a plain player alone
/// cleared three.
pub const HEALTH_GROWS: f32 = 1.22;
pub const DAMAGE_GROWS: f32 = 1.09;
pub const PACE_GROWS: f32 = 0.01;
pub const PACE_MOST: f32 = 1.3;

/// What one of `kind` has in wave `wave`: its health, its blow's damage,
/// and its pace, in metres a second.
pub fn strength(kind: Kind, wave: u16) -> (f32, f32, f32) {
    let (health, damage, pace) = kind.base();
    // (No more than two hundred waves of growth: well past where anyone is
    // still standing, and short of what a float holds.)
    let past = f32::from(wave.saturating_sub(1).min(200));
    (health * HEALTH_GROWS.powf(past), damage * DAMAGE_GROWS.powf(past), pace * (1.0 + PACE_GROWS * past).min(PACE_MOST))
}

/// How many times as much health one of `kind` has for `players` people in
/// the game: a giant, of which a wave has as many whoever comes, has a
/// five eighths of its health for one person and all of it for four
/// (mine: the rest of a wave grows with the people in its numbers,
/// [`count`]); every other kind, its own.
pub fn toughness(kind: Kind, players: usize) -> f32 {
    match kind {
        Kind::Giant => 0.5 + 0.125 * players.clamp(1, 4) as f32,
        _ => 1.0,
    }
}

/// How many zombies wave `wave` sends with `players` people in the game:
/// six, three more a wave and a quarter of the wave's square (the
/// design's), and more for more people, but not as many more as there are
/// people: three quarters and a quarter a person (mine; the design's 0.6
/// and 0.4 a person had four people at one anchor, who get in each other's
/// way, no further on than one alone). 9 in the first for one, 16 for four;
/// 61 in the tenth for one, 107 for four.
pub fn count(wave: u16, players: usize) -> u16 {
    let wave = f32::from(wave);
    let people = players.clamp(1, 4) as f32;
    ((6.0 + 3.0 * wave + wave * wave / 4.0) * (0.75 + 0.25 * people)).round().min(f32::from(u16::MAX)) as u16
}

/// How many of wave `wave` come over the rim a second, all its places
/// together, with `players` people in the game: 0.22 and a tenth for each
/// person in the first wave (0.32 for one, 0.62 for four), and 12% of that
/// more for every wave after (mine). A wave comes as a stream to be met,
/// not a heap at once; the rest waits below the rim. The later waves come
/// on faster than the first so that a game of four, which goes on longest,
/// is over within twenty minutes: at 7% a wave it took twenty and more.
pub fn rising(wave: u16, players: usize) -> f32 {
    (0.22 + 0.1 * players.clamp(1, 4) as f32) * (1.0 + 0.12 * f32::from(wave.saturating_sub(1).min(100)))
}

/// How many giants wave `wave` has: every fifth wave, one in the fifth and
/// the tenth, two in the fifteenth and the twentieth, and so on, one more
/// every ten waves; none in the waves between. Mine: the design's one
/// more every five waves made the tenth, with two, the end of half of all
/// the games of a person alone, whatever came before.
pub fn giants(wave: u16) -> u16 {
    if wave > 0 && wave % 5 == 0 { (wave / 5 + 1) / 2 } else { 0 }
}

/// The kinds that come for the first time in wave `wave`, for the headline
/// that says so ("Brutes!", "A giant!").
pub fn first_in(wave: u16) -> Vec<Kind> {
    Kind::ALL.into_iter().filter(|kind| kind.first() == wave && *kind != Kind::Walker).collect()
}

/// How likely each kind but the giant is to be the next of wave `wave`, as
/// weights: walkers always, and each harder kind once it has come, more of
/// it with every wave (mine).
pub fn weights(wave: u16) -> [(Kind, f32); 5] {
    let since = |kind: Kind| f32::from(wave) - f32::from(kind.first());
    let grows = |kind: Kind, from: f32, by: f32| if since(kind) >= 0.0 { from + by * since(kind) } else { 0.0 };
    [
        (Kind::Walker, 100.0),
        (Kind::Runner, grows(Kind::Runner, 22.0, 6.0)),
        (Kind::Brute, grows(Kind::Brute, 8.0, 2.5)),
        (Kind::Spitter, grows(Kind::Spitter, 10.0, 3.0)),
        (Kind::Leaper, grows(Kind::Leaper, 10.0, 3.0)),
    ]
}

/// Wave `wave`'s zombies, with `players` people in the game, in the order
/// they come over the rim: [`count`] of them, its giants among them (each
/// somewhere after the first half of the wave has come, to meet a team that
/// is already fighting), two at least of any kind that comes for the first
/// time (one brute), and the rest thrown by [`weights`], in a thrown order.
pub fn mix(wave: u16, players: usize, dice: &mut Dice) -> Vec<Kind> {
    let total = usize::from(count(wave, players));
    let giants = usize::from(giants(wave)).min(total);
    let mut kinds: Vec<Kind> = Vec::with_capacity(total);
    for kind in first_in(wave) {
        if kind != Kind::Giant {
            let least = if kind == Kind::Brute { 1 } else { 2 };
            kinds.extend(std::iter::repeat_n(kind, least));
        }
    }
    let table = weights(wave);
    let sum: f32 = table.iter().map(|(_, weight)| weight).sum();
    while kinds.len() < total - giants {
        let mut pick = dice.roll() * sum;
        let kind = table
            .iter()
            .find(|(_, weight)| {
                pick -= weight;
                pick < 0.0
            })
            .map_or(Kind::Walker, |(kind, _)| *kind);
        kinds.push(kind);
    }
    kinds.truncate(total - giants);
    // In a thrown order.
    for at in (1..kinds.len()).rev() {
        let other = ((dice.roll() * (at + 1) as f32) as usize).min(at);
        kinds.swap(at, other);
    }
    for _ in 0..giants {
        let at = ((dice.between(0.5, 0.8) * kinds.len() as f32) as usize).min(kinds.len());
        kinds.insert(at, Kind::Giant);
    }
    kinds
}

/// How likely a zombie killed is to drop something, with the team's Lucky
/// Loot at `lucky`: one time in three, and a tenth more for each step.
/// Brutes and giants always drop ([`drop_of`]).
pub fn drop_chance(lucky: u8) -> f32 {
    (1.0 / 3.0 + 0.1 * f32::from(lucky)).min(0.95)
}

/// How likely each item is to be a thing a zombie drops in wave `wave`, as
/// weights (the design's): silver always, gold more and more, sapphire from
/// the third wave and amethyst from the fifth, each more with every wave.
fn item_weights(wave: f32) -> [f32; 4] {
    [
        100.0,
        10.0 + 4.0 * wave,
        if wave >= 3.0 { 2.0 * wave } else { 0.0 },
        if wave >= 5.0 { wave - 4.0 } else { 0.0 },
    ]
}

/// What a zombie of `kind` killed in wave `wave` drops, with the team's
/// Lucky Loot at `lucky`, each thing the item it is and how many of it it is
/// worth (one): of the plain kinds, one thing a [`drop_chance`] of the time;
/// a brute two or three; a giant a hoard of twelve. Each thing's item is
/// thrown by weights that move toward the better ones wave by wave, and
/// three waves further on for each step of luck. None for nothing dropped.
pub fn drop_of(kind: Kind, wave: u16, lucky: u8, dice: &mut Dice) -> Vec<(Item, u16)> {
    let things = match kind {
        Kind::Giant => 12,
        Kind::Brute => 2 + usize::from(dice.roll() < 0.5),
        _ => usize::from(dice.roll() < drop_chance(lucky)),
    };
    let weights = item_weights(f32::from(wave) + 3.0 * f32::from(lucky));
    let sum: f32 = weights.iter().sum();
    (0..things)
        .map(|_| {
            let mut pick = dice.roll() * sum;
            let item = Item::ALL
                .into_iter()
                .zip(weights)
                .find(|(_, weight)| {
                    pick -= weight;
                    pick < 0.0
                })
                .map_or(Item::Silver, |(item, _)| item);
            (item, 1)
        })
        .collect()
}

/// What killing a zombie of `kind` in wave `wave` scores for whoever dealt
/// the last blow: its kind's worth, and a tenth more for each wave.
pub fn kill_score(kind: Kind, wave: u16) -> u32 {
    (kind.score() * (1.0 + f32::from(wave) / 10.0)).round() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waves_grow_with_the_wave_and_with_the_people_but_less_than_one_for_one() {
        assert_eq!(count(1, 1), 9);
        assert_eq!(count(1, 4), 16);
        assert_eq!(count(5, 1), 27);
        assert_eq!(count(10, 1), 61);
        assert_eq!(count(10, 4), 107);
        for wave in 1..60 {
            assert!(count(wave + 1, 1) > count(wave, 1), "wave {wave}");
            for people in 1..4 {
                let (fewer, more) = (count(wave, people), count(wave, people + 1));
                assert!(more > fewer && f32::from(more) < f32::from(fewer) * 2.0, "wave {wave}, {people} people");
            }
        }
        // Saturating, never overflowing.
        assert_eq!(count(u16::MAX, 4), u16::MAX);
        assert_eq!(count(1, 0), count(1, 1));
    }

    #[test]
    fn each_kind_comes_from_its_wave_on_and_grows_stronger_every_wave() {
        let mut dice = Dice::new(5);
        for wave in 1..30u16 {
            let kinds = mix(wave, 2, &mut dice);
            assert_eq!(kinds.len(), usize::from(count(wave, 2)), "wave {wave}");
            for kind in Kind::ALL {
                let had = kinds.iter().filter(|other| **other == kind).count();
                if wave < kind.first() {
                    assert_eq!(had, 0, "wave {wave}: {kind:?} before its time");
                }
                if wave == kind.first() {
                    assert!(had >= 1, "wave {wave}: no {kind:?} in its first wave");
                }
            }
            assert_eq!(kinds.iter().filter(|kind| **kind == Kind::Giant).count(), usize::from(giants(wave)), "wave {wave}");
            // Never a giant first: the team meets the wave before its giant.
            assert_ne!(kinds.first(), Some(&Kind::Giant), "wave {wave}");
        }
        assert_eq!((giants(4), giants(5), giants(10), giants(11), giants(15), giants(20), giants(25)), (0, 1, 1, 0, 2, 2, 3));
        assert_eq!(first_in(5), vec![Kind::Giant]);
        assert!(first_in(1).is_empty());
        // Stronger every wave, the pace to a third more and no more.
        for kind in Kind::ALL {
            let mut last = strength(kind, 1);
            for wave in 2..80 {
                let now = strength(kind, wave);
                assert!(now.0 > last.0 && now.1 > last.1 && now.2 >= last.2, "{kind:?} in wave {wave}");
                assert!(now.2 <= strength(kind, 1).2 * PACE_MOST + 1e-4);
                last = now;
            }
            let (health, damage, pace) = strength(kind, u16::MAX);
            assert!(health.is_finite() && damage.is_finite() && pace.is_finite(), "{kind:?}");
        }
        // The harder kinds take more of a wave as the waves go on.
        let share = |wave: u16| {
            let table = weights(wave);
            1.0 - table[0].1 / table.iter().map(|(_, weight)| weight).sum::<f32>()
        };
        assert!(share(3) < share(8) && share(8) < share(15) && share(15) < share(30));
        assert_eq!(share(1), 0.0);
    }

    #[test]
    fn drops_come_more_often_and_better_with_the_waves_and_with_luck() {
        let mut dice = Dice::new(9);
        // Of a thousand walkers in each of these waves, with no luck and with
        // all of it: how many drop, and what share of what drops is better
        // than silver.
        let tell = |wave: u16, lucky: u8, dice: &mut Dice| {
            let things: Vec<(Item, u16)> = (0..3000).flat_map(|_| drop_of(Kind::Walker, wave, lucky, dice)).collect();
            let better = things.iter().filter(|(item, _)| *item != Item::Silver).count() as f32 / things.len() as f32;
            (things.len(), better, things.iter().any(|(item, _)| *item == Item::Amethyst))
        };
        let (dropped, better, amethyst) = tell(1, 0, &mut dice);
        assert!((850..=1150).contains(&dropped), "{dropped} of 3000");
        assert!(better < 0.2 && !amethyst, "{better}");
        let (later, later_better, later_amethyst) = tell(12, 0, &mut dice);
        assert!((850..=1150).contains(&later) && later_better > 0.4 && later_amethyst, "{later}, {later_better}");
        let (lucky, lucky_better, _) = tell(12, LUCKY, &mut dice);
        assert!(lucky > 2 * later && lucky_better > later_better + 0.05, "{lucky}, {lucky_better}");
        // Brutes always drop two or three; giants a hoard.
        for _ in 0..50 {
            assert!((2..=3).contains(&drop_of(Kind::Brute, 6, 0, &mut dice).len()));
            assert_eq!(drop_of(Kind::Giant, 5, 0, &mut dice).len(), 12);
        }
        // And a kill is worth more later.
        assert_eq!(kill_score(Kind::Walker, 1), 11);
        assert_eq!(kill_score(Kind::Giant, 10), 500);
        assert!(kill_score(Kind::Brute, 9) > kill_score(Kind::Brute, 8));
    }

    #[test]
    fn each_kind_comes_to_the_design_s_health_and_blow_in_the_waves_its_doc_says() {
        // The design's first wave of each kind, health and blow, growing
        // 1.12 and 1.07 a wave; and the first wave each of ours is as much
        // or more ([`Kind::base`] says which).
        let design = [
            (Kind::Walker, 40.0, 8.0),
            (Kind::Runner, 25.0, 6.0),
            (Kind::Brute, 160.0, 22.0),
            (Kind::Spitter, 35.0, 12.0),
            (Kind::Leaper, 50.0, 14.0),
            (Kind::Giant, 1200.0, 40.0),
        ];
        let reaches = |kind: Kind, ours: fn((f32, f32, f32)) -> f32, first: f32, grows: f32| {
            (1..100u16).find(|wave| ours(strength(kind, *wave)) >= first * grows.powi(i32::from(*wave) - 1)).expect("in time")
        };
        let health: Vec<u16> = design.iter().map(|(kind, health, _)| reaches(*kind, |numbers| numbers.0, *health, 1.12)).collect();
        let blow: Vec<u16> = design.iter().map(|(kind, _, blow)| reaches(*kind, |numbers| numbers.1, *blow, 1.07)).collect();
        assert_eq!(health, [10, 10, 16, 9, 10, 13]);
        assert_eq!(blow, [17, 11, 26, 11, 10, 21]);
    }

    const LUCKY: u8 = super::super::LUCKY_MOST;
}
