//! Houses (since 2026-11-27; Hajun: "whenever someone visits your plot (stay
//! for more than 10 seconds) it will send a push notification to you like
//! "username is visiting your house. Go say hi!" after 15 seconds of
//! staying on the plot, if you've never rated that persons plot's version
//! before (if theyve modified the house after uve rated it, u can rate it
//! again) u will get 5 stars at the bottom middle of ur screen ... and in
//! the social tab ... a house tab ... a global leaderboard of the best rated
//! house that shows the top 100").
//!
//! **A visit** is someone standing on the plot of someone else's land in
//! the town ([`on_plot`]: the plot's own square, which the server goes by,
//! having no map) for [`VISIT_SECS`] together: its owner is told so, by a
//! push to their phone while they are away from the game and across their
//! screen while they are in it ([`VISITING`]), and it counts toward their
//! visitors. The same visitor counts again, and tells again, once
//! [`VISIT_AGAIN`] has gone by since they last did (mine).
//!
//! **A rating** is half a star to five, by halves ([`HALVES_MOST`]), and up
//! to [`WORDS_MOST`] of the [`WORDS`] words there are to say what the land
//! is like, each by its number (the game's `assets/ui.ron` has the words,
//! `words.house.adjectives`, in this order). Someone who has stood on
//! someone's plot [`RATE_SECS`] may rate their land **as it stands**: the
//! server keeps with each rating a print of the land it was given for
//! ([`land_print`]), and offers the stars again once what stands there, or
//! its paint, is no longer what was rated. **One rating to a rater and a
//! house**: a rating given again takes the place of the one before (mine),
//! so that someone who rates every change counts once.
//!
//! **The board** ranks every house that has been rated by its stars,
//! weighted toward [`PRIOR_STARS`] as though it had [`PRIOR`] ratings more
//! of that ([`score`]: mine, so that one rating of five is not the best
//! house there is), the top [`BOARD`] shown to everyone, and to whoever is
//! further down, their own place as well.
//!
//! What is asked and told is [`Ask`] and [`Tell`], inside
//! [`super::Ask::House`] and [`super::Tell::House`]: only to a device that
//! has asked to be told ([`Ask::Watch`]).

use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

use crate::At;
use crate::bag::{Paint, Standing};

/// How long someone has to stand on the plot of someone else's land, all
/// together, for it to be a visit, and for them to be offered to rate it, in
/// seconds: Hajun's numbers.
pub const VISIT_SECS: f32 = 10.0;
pub const RATE_SECS: f32 = 15.0;

/// How long after a visit that counted the same visitor counts again on the
/// same house, and its owner is told again, in seconds: five minutes, so
/// that a friend come back is told of again and someone stepping on and off
/// the edge is not (Hajun, 2026-11-28, of the hour it was: "too much"; "5
/// minutes instead of an hour").
pub const VISIT_AGAIN: f64 = 300.0;

/// The least time between two pushes to one owner about visits, in seconds.
/// None since 2026-11-28 (it was two minutes, which Hajun found "too much"):
/// three friends arriving together are three pushes, each with its name.
/// Each visit counts all the same.
pub const VISIT_PUSH_GAP: f64 = 0.0;

/// How long an offer to rate stands once its visitor has walked off the
/// plot, in seconds: a rating sent just as they went, or as the plot was
/// given to someone else, still counts.
pub const OFFER_STANDS: f64 = 120.0;

/// A rating's stars, in halves of a star: half a star at the least, five at
/// the most (Hajun: "u can tap again to modify ur star betwen 0.5~5star
/// with 0.5 increment").
pub const HALVES_LEAST: u8 = 1;
pub const HALVES_MOST: u8 = 10;

/// How many words there are to choose from, and how many one rating may
/// have ("u can choose 0~ or up to 3 adjectives").
pub const WORDS: usize = 16;
pub const WORDS_MOST: usize = 3;

/// How many houses the board shows everyone ("shows the top 100").
pub const BOARD: usize = 100;

/// The most ratings sent with a house's stats, the newest first. Mine.
pub const LISTED_MOST: usize = 100;

/// A week, in seconds: what "the past 7 days" of visitors goes back.
pub const WEEK: f64 = 7.0 * 86_400.0;

/// How a house is weighted on the board ([`score`]): as though it had
/// [`PRIOR`] more ratings of [`PRIOR_STARS`]. Mine.
pub const PRIOR: f64 = 3.0;
pub const PRIOR_STARS: f64 = 3.0;

/// How far a plot reaches each way from its middle, along its own sides, in
/// metres: every plot of the town is 24 m square.
pub const PLOT_HALF: f32 = 12.0;

/// How far each plot is turned about the upright, in degrees, by Hajun's
/// numbers for them ([`crate::land::HOMES`]): its sides run along the
/// game's x and z axes turned this much. The twelve round the pond and the
/// plots along the streets are square to the world; the corners of each
/// ring are not. A test of the game's holds these to the map.
pub const TURNS: [f32; crate::land::PLOTS] = [
    // 1 to 12: round the pond, the four corner ones on the diagonals.
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 45.0, 45.0, 45.0, 45.0, //
    // 13 to 28.
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 55.0, 55.0, 55.0, 55.0, 35.0, 35.0, 35.0, 35.0, //
    // 29 to 48: the outer ring.
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 60.0, 60.0, 60.0, 60.0, 30.0, 30.0, 30.0, 30.0, 45.0, 45.0, 45.0, 45.0,
];

/// Whether someone standing `at`, across the ground (x and z), is on the
/// plot numbered `plot` ([`crate::land::HOMES`]): within its square, its
/// edges too. Nowhere is on two plots.
pub fn on_plot(plot: usize, at: Vec2) -> bool {
    let (Some(&home), Some(&turn)) = (crate::land::HOMES.get(plot), TURNS.get(plot)) else {
        return false;
    };
    let from = at - Vec2::from(home);
    let (sin, cos) = turn.to_radians().sin_cos();
    // Into the plot's own axes: turned back by its turn.
    let along = Vec2::new(from.x * cos + from.y * sin, -from.x * sin + from.y * cos);
    along.x.abs() <= PLOT_HALF && along.y.abs() <= PLOT_HALF
}

/// The plot someone standing `at` is on, if any ([`on_plot`]).
pub fn plot_under(at: Vec2) -> Option<usize> {
    (0..crate::land::PLOTS).find(|&plot| on_plot(plot, at))
}

/// Whether `halves` is a rating's stars: half a star to five.
pub fn halves_ok(halves: u8) -> bool {
    (HALVES_LEAST..=HALVES_MOST).contains(&halves)
}

/// The words chosen for a rating as they are kept, if they are some: each a
/// word there is, none twice, and no more than [`WORDS_MOST`]; in order.
pub fn chosen(words: &[u8]) -> Option<Vec<u8>> {
    let mut kept: Vec<u8> = words.to_vec();
    kept.sort_unstable();
    kept.dedup();
    (kept.len() == words.len() && kept.len() <= WORDS_MOST && kept.iter().all(|&word| usize::from(word) < WORDS))
        .then_some(kept)
}

/// The words `chosen` as one number, a bit to each: how they are kept in
/// the database.
pub fn words_to_bits(chosen: &[u8]) -> u32 {
    chosen.iter().filter(|&&word| usize::from(word) < WORDS).fold(0, |bits, &word| bits | 1 << word)
}

/// The words a number kept in the database stands for ([`words_to_bits`]),
/// in order.
pub fn bits_to_words(bits: u32) -> Vec<u8> {
    (0..WORDS as u8).filter(|&word| bits & 1 << word != 0).take(WORDS_MOST).collect()
}

/// The stars of `count` ratings whose halves come to `sum`, on average: 0
/// with none.
pub fn average(count: u32, sum: u32) -> f32 {
    if count == 0 { 0.0 } else { sum as f32 / count as f32 / 2.0 }
}

/// What ranks a house on the board, of `count` ratings whose halves come to
/// `sum`: its stars on average, as though it had [`PRIOR`] more ratings of
/// [`PRIOR_STARS`] (the "Bayesian average" of the stores' charts), so that
/// a house rated once, five stars, is not above one rated forty times, four
/// and a half.
pub fn score(count: u32, sum: u32) -> f64 {
    let stars = f64::from(sum) / 2.0;
    (stars + PRIOR * PRIOR_STARS) / (f64::from(count) + PRIOR)
}

/// `houses`, each an account and the count and sum of its ratings, in the
/// board's order: the best [`score`] first, then the most ratings, then the
/// oldest account. Those never rated are left out.
pub fn ranked(mut houses: Vec<(u64, u32, u32)>) -> Vec<(u64, u32, u32)> {
    houses.retain(|&(_, count, _)| count > 0);
    houses.sort_by(|a, b| {
        score(b.1, b.2)
            .total_cmp(&score(a.1, a.2))
            .then(b.1.cmp(&a.1))
            .then(a.0.cmp(&b.0))
    });
    houses
}

/// A print of what stands on a plot, and its paint: the same for land that
/// is the same, and another, nearly always, for land any of which has been
/// put down, moved, turned, taken back or painted. FNV-1a over everything
/// of it, in order, so that the server finds the same print for the same
/// land from one run to the next.
pub fn land_print(stands: &[Standing], paints: &[Paint]) -> u64 {
    let mut print = Fnv::new();
    print.number(stands.len() as u64);
    for stood in stands {
        print.number(u64::from(stood.id));
        print.bytes(stood.kind.as_bytes());
        match stood.at {
            At::Ground { x, y, z, yaw } => {
                print.number(0);
                for n in [x, y, z, yaw] {
                    print.number(u64::from(n.to_bits()));
                }
            }
            At::Wall { wall, along, up, face } => {
                print.number(1);
                print.number(u64::from(wall));
                print.number(u64::from(along.to_bits()));
                print.number(u64::from(up.to_bits()));
                print.number(face as u8 as u64);
            }
        }
    }
    print.number(paints.len() as u64);
    for paint in paints {
        print.number(u64::from(paint.id));
        print.bytes(paint.part.as_bytes());
        print.bytes(&paint.colour);
    }
    print.0
}

/// FNV-1a, 64 bits.
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn byte(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
    }

    fn number(&mut self, n: u64) {
        for byte in n.to_le_bytes() {
            self.byte(byte);
        }
    }

    /// Bytes, and how many, so that two strings one after the other are
    /// never taken for another two.
    fn bytes(&mut self, bytes: &[u8]) {
        self.number(bytes.len() as u64);
        for &byte in bytes {
            self.byte(byte);
        }
    }
}

/// What the words of a push say ([`VISITING`], [`RATED`]), the server's to
/// send, which has no `assets/ui.ron`: Hajun's words. `{name}` is the
/// visitor's.
pub const VISITING: &str = "{name} is visiting your house. Go say hi!";
pub const RATED: &str = "Someone just rated your house. Open the app to see what rating they gave you!";

/// A phone's maker, whose service a push to it goes by: Google's Firebase
/// Cloud Messaging for an Android phone, Apple's Push Notification service
/// for an iPhone.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Platform {
    Android,
    Ios,
}

/// The longest token a device's push service gives it, in bytes: an FCM
/// token is about 160, an APNs one 64 hex digits.
pub const TOKEN_LONGEST: usize = 4_096;

/// Whether `token` could be one a push service gave: something, not too
/// long, and of what such tokens are written in.
pub fn token_ok(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= TOKEN_LONGEST
        && token.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
}

/// Something asked of the server about houses.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum Ask {
    /// Tell me about houses: that I may rate the one I am visiting, and
    /// that someone visits mine or has rated it. Said once, on coming
    /// online, by an app that knows how to show it.
    Watch,
    /// This device's push token, from its maker's service, for word of my
    /// house while the app is away; said whenever the app has one, as it
    /// comes online. With none ever said, nothing is pushed.
    Push { platform: Platform, token: String },
    /// My rating of the house of the account `owner` I am visiting, as it
    /// stands: `halves` of a star ([`halves_ok`]) and the words chosen by
    /// their numbers ([`chosen`]). Taken only from someone offered to rate
    /// it ([`Tell::MayRate`]) a moment ago at the most ([`OFFER_STANDS`]).
    Rate { owner: u64, halves: u8, words: Vec<u8> },
    /// The stats and the ratings of the house of the account `who`, mine
    /// with none ([`Tell::Stats`] answers).
    Stats(Option<u64>),
    /// The board ([`Tell::Board`] answers).
    Board,
}

/// What the server tells a device about houses, only one that has asked
/// ([`Ask::Watch`]).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// You may rate this house now, having stood on its plot a while, and
    /// not having rated it as it stands; or, with none, no longer: you have
    /// rated it, walked off, or it is someone else's plot now.
    MayRate(Option<Rateable>),
    /// Someone, by the name they go by, is visiting your house: said while
    /// you are in the game, where a push would not show.
    Visiting { name: String },
    /// Someone has rated your house: said while you are in the game.
    Rated,
    /// Your rating is kept.
    Thanks,
    /// A house's stats and its ratings ([`Ask::Stats`]).
    Stats(Stats),
    /// The board ([`Ask::Board`]).
    Board(Board),
}

/// A house you may rate: whose, and the name on its sign.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Rateable {
    pub owner: u64,
    pub name: String,
}

/// A house as its stats show it: whose, by their account, name and tag;
/// what stands on it is worth at the shop's prices, in coins
/// ([`crate::land::worth`]), and how many things that is; how many visits
/// it has had, ever and in the past [`WEEK`]; its ratings, how many and
/// their halves together; and the ratings themselves, the newest first, up
/// to [`LISTED_MOST`].
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct Stats {
    pub who: u64,
    pub name: String,
    pub tag: u16,
    pub worth: u32,
    pub things: u32,
    pub visits: u32,
    pub visits_week: u32,
    pub count: u32,
    pub sum: u32,
    pub ratings: Vec<Rating>,
}

/// One rating, as a house's stats list it: who gave it, by their account,
/// name and tag; its halves of a star; and its words, by their numbers.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Rating {
    pub who: u64,
    pub name: String,
    pub tag: u16,
    pub halves: u8,
    pub words: Vec<u8>,
}

/// The board: the top [`BOARD`] houses, the best first; and, for whoever
/// asked, their own house's place, if it is rated and further down than
/// those (Hajun: "if ur like top 101+ then it just shows u that ur place is
/// top 101 on your side at the bottom, but not to anyone else").
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct Board {
    pub top: Vec<Ranked>,
    pub mine: Option<Ranked>,
}

/// One house on the board: its place, from 1; whose; and its ratings, how
/// many and their halves together.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Ranked {
    pub place: u32,
    pub who: u64,
    pub name: String,
    pub tag: u16,
    pub count: u32,
    pub sum: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plot_is_its_own_square_however_it_is_turned_and_nowhere_is_on_two() {
        for (plot, (&home, turn)) in crate::land::HOMES.iter().zip(TURNS).enumerate() {
            let middle = Vec2::from(home);
            let (sin, cos) = turn.to_radians().sin_cos();
            // Its own axes in the world's.
            let (across, down) = (Vec2::new(cos, sin), Vec2::new(-sin, cos));
            for (a, b) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                let corner = middle + (across * a + down * b) * PLOT_HALF;
                assert!(on_plot(plot, corner.lerp(middle, 0.01)), "{plot}: inside its corner");
                assert!(!on_plot(plot, middle + (corner - middle) * 1.03), "{plot}: past its corner");
                assert_eq!(plot_under(corner.lerp(middle, 0.01)), Some(plot));
            }
            assert!(on_plot(plot, middle));
            assert!(!on_plot(plot, middle + across * (PLOT_HALF + 0.5)));
        }
        // The pond and the path round it are nobody's.
        assert_eq!(plot_under(Vec2::ZERO), None);
        assert_eq!(plot_under(Vec2::new(40.0, 0.0)), None);
        // And every square lies within what the server took for a plot
        // before ([`crate::land::ON_A_PLOT`]).
        for plot in 0..crate::land::PLOTS {
            assert_eq!(crate::land::plot_at(Vec2::from(crate::land::HOMES[plot])), Some(plot));
        }
    }

    #[test]
    fn words_are_a_few_of_those_there_are_none_twice() {
        assert_eq!(chosen(&[]), Some(vec![]));
        assert_eq!(chosen(&[5, 1]), Some(vec![1, 5]));
        assert_eq!(chosen(&[1, 2, 15]), Some(vec![1, 2, 15]));
        assert_eq!(chosen(&[1, 2, 3, 4]), None, "four");
        assert_eq!(chosen(&[2, 2]), None, "twice");
        assert_eq!(chosen(&[16]), None, "no such word");
        for words in [vec![], vec![0], vec![3, 9, 15]] {
            assert_eq!(bits_to_words(words_to_bits(&words)), words);
        }
        assert!(halves_ok(1) && halves_ok(10));
        assert!(!halves_ok(0) && !halves_ok(11));
    }

    #[test]
    fn the_board_goes_by_stars_weighted_toward_three() {
        // One rating of five is no match for forty of four and a half.
        let once = (1, 1, 10);
        let often = (2, 40, 40 * 9);
        let ranked = ranked(vec![once, often, (3, 0, 0)]);
        assert_eq!(ranked, vec![often, once], "the unrated left out");
        assert!((average(40, 360) - 4.5).abs() < 1e-6);
        assert_eq!(average(0, 0), 0.0);
        // As good as each other: the more ratings first, then the older.
        let ranked = super::ranked(vec![(9, 2, 16), (8, 2, 16), (7, 4, 32)]);
        assert_eq!(ranked.iter().map(|house| house.0).collect::<Vec<_>>(), vec![7, 8, 9]);
        // More ratings of the same stars rank higher the better they are.
        assert!(score(10, 100) > score(1, 10));
        assert!(score(10, 20) < score(1, 2));
    }

    #[test]
    fn a_land_s_print_changes_with_anything_of_it() {
        let chair = |x: f32| Standing {
            id: 3,
            kind: "Chair".into(),
            at: At::Ground { x, y: 0.0, z: 1.0, yaw: 0.5 },
        };
        let paint = Paint {
            id: 3,
            part: "seat".into(),
            colour: [10, 20, 30],
        };
        let print = land_print(&[chair(1.0)], &[]);
        assert_eq!(print, land_print(&[chair(1.0)], &[]), "the same land, the same print");
        assert_ne!(print, land_print(&[chair(1.001)], &[]), "moved");
        assert_ne!(print, land_print(&[], &[]), "taken back");
        assert_ne!(print, land_print(&[chair(1.0)], std::slice::from_ref(&paint)), "painted");
        let repainted = Paint { colour: [10, 20, 31], ..paint.clone() };
        assert_ne!(land_print(&[chair(1.0)], &[paint]), land_print(&[chair(1.0)], &[repainted]));
        // The same on every run: a number fixed here.
        assert_eq!(land_print(&[], &[]), land_print(&[], &[]));
    }

    #[test]
    fn a_token_is_what_push_services_write() {
        assert!(token_ok("dGVzdA:APA91bH-x_y.z"));
        assert!(token_ok(&"ab12".repeat(16)));
        assert!(!token_ok(""));
        assert!(!token_ok("has space"));
        assert!(!token_ok(&"a".repeat(TOKEN_LONGEST + 1)));
    }
}
