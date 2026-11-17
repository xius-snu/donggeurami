//! What the four games of 2026-11-18 have in common (Hajun: "add these
//! minigames: Blend In, Knock Off, Bridge Race, Last Stand ... each of these
//! games have a coin entrance fee and payout. Generally should be above 50%
//! to win net positive coin, and if not, net negative coin"): how a game of
//! one of them fills, starts and pays, and the board every device in its
//! room is sent ([`Board`]).
//!
//! A game of one of them is played at a table: a room of its own on the
//! server online, and the device's own offline, for the games played
//! offline. Whoever joins ([`Ask::Join`](crate::Ask::Join)) pays the fee and
//! takes a seat ([`Tell::Entered`](crate::Tell::Entered)). In the games that
//! have the computer, it takes the seats left one at a time once nobody has
//! joined for a while, as in Bridge Wars; a table with as many people as it
//! begins with counts down once nobody has joined for a while, or at once
//! when it is full; then the game is played, by its own rules, each in its
//! own module ([`crate::blend_in`], [`crate::knock_off`],
//! [`crate::bridge_race`], [`crate::last_stand`]); and at the end how
//! everyone came is up a while ([`Results`]) before everyone still there
//! goes back to the town. Each game's numbers for all of this are its
//! [`Shape`] (its own `SHAPE`), [`shape`] by the game.
//!
//! **Coins.** The fee is paid on joining, and given back to whoever leaves,
//! or is left, before the game begins. At the end each game pays by place,
//! by its own table of what a place is worth (each game's `prizes`); seats
//! that tie share what the places they take up pay, evenly and rounded down
//! ([`split`]); the computer is paid nothing. The fees and what each place
//! pays are ours, by Hajun's rule above: whoever beats more than half of the
//! others comes away with more than they paid, and everyone else with less.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::social::Minigame;
use crate::{ARENA_SEATS, GAME_SEATS, blend_in, bridge_race, knock_off, last_stand};

/// How a game of one of the four fills, starts and pays (each game's own `SHAPE`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Shape {
    /// The most seats it has.
    pub most: usize,
    /// The fewest people it begins with (Blend In 2: "This game min player count is 2").
    pub fewest: usize,
    /// Whether the computer takes the seats people leave empty, one at a time, once nobody has
    /// joined for `wait` seconds (Knock Off, Bridge Race: "fill with ai like bridgewars").
    pub computer: bool,
    /// Seconds after the last person joined before the computer comes (with `computer`), or
    /// before the countdown begins (without), once there are `fewest` people.
    pub wait: f32,
    /// Seconds counted down before play.
    pub countdown: f32,
    /// Seconds the results are up before everyone still there goes back to the town.
    pub results: f32,
    /// What it costs to join, in coins.
    pub fee: u32,
}

/// `game`'s shape: None for the two older games, which do not sit at tables.
pub fn shape(game: Minigame) -> Option<Shape> {
    match game {
        Minigame::BridgeWars | Minigame::HouseBuilder => None,
        Minigame::BlendIn => Some(blend_in::SHAPE),
        Minigame::KnockOff => Some(knock_off::SHAPE),
        Minigame::BridgeRace => Some(bridge_race::SHAPE),
        Minigame::LastStand => Some(last_stand::SHAPE),
    }
}

/// How many seats a table of `game` has with `people` people at it once the computer has
/// come: `most` with the computer, else as many as there are people (at most `most`);
/// Bridge Race's own rule (`bridge_race::seats_for`). The two older games, which are not
/// played at tables, have every seat of theirs filled, the computer taking what people
/// leave: Bridge Wars' eight ([`ARENA_SEATS`]) and House Builder's ([`GAME_SEATS`]).
pub fn seats_for(game: Minigame, people: usize) -> usize {
    match (game, shape(game)) {
        (Minigame::BridgeRace, _) => bridge_race::seats_for(people),
        (_, Some(shape)) if shape.computer => shape.most,
        (_, Some(shape)) => people.min(shape.most),
        (Minigame::BridgeWars, None) => ARENA_SEATS,
        (_, None) => GAME_SEATS,
    }
}

/// The board of a game of one of the four, on the entity its room is sent beside
/// [`Seats`](crate::Seats) (who sits where) and the game's own components: replicated,
/// re-inserted only when it changed. A few dozen bytes, sent as the stage or the seconds
/// left change: once a second while anything counts down.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Board {
    /// Which of the four this table is a game of.
    pub game: Minigame,
    /// The table's number, per game, from the server (or the device offline).
    pub id: u32,
    /// How far it has got.
    pub stage: Stage,
    /// Whole seconds left of this stage, rounded up; 0 where nothing counts down (waiting with
    /// too few people, or playing a game with no clock).
    pub left: u16,
    /// People seated, and seats the game will have (for "2/4").
    pub people: u8,
    pub seats: u8,
}

/// How far a game at a table has got.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Stage {
    /// Waiting for people (and the computer).
    Waiting,
    /// Full, counting down.
    Countdown,
    /// Being played: the game's own components say how.
    Playing,
    /// Over: how everyone came.
    Over(Results),
}

/// How a game came out, by seat.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Results {
    /// The place each came, 0 first; seats with the same place tied. A place is how many came
    /// before: after two who tied first comes the third (0, 0, 2), as [`split`] reads them.
    pub places: Vec<u8>,
    /// The score each made, as the game counts it (its words say what it is).
    pub scores: Vec<u32>,
    /// The coins each was paid (people only; the computer is paid nothing).
    pub prizes: Vec<u32>,
    /// What the whole table made together, as the game counts it, if anything (Last Stand:
    /// the waves cleared).
    pub total: Option<u32>,
}

/// Coins by place, `by_place[0]` to first: every seat with place `p`, `k` of them tied, gets
/// `(by_place[p] + ... + by_place[p + k - 1]) / k`, rounded down; a place past the end of
/// `by_place` pays 0. By seat, like `places`.
pub fn split(by_place: &[u32], places: &[u8]) -> Vec<u32> {
    places
        .iter()
        .map(|&place| {
            let tied = places.iter().filter(|&&other| other == place).count();
            let first = usize::from(place);
            let shared: u32 = (first..first + tied).map(|taken| by_place.get(taken).copied().unwrap_or(0)).sum();
            shared / tied as u32
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_pay_what_they_cover_shared_evenly_between_those_tied() {
        let pays = [130, 60, 10, 0];
        // Nobody tied: each their own place's.
        assert_eq!(split(&pays, &[0, 1, 2, 3]), vec![130, 60, 10, 0]);
        assert_eq!(split(&pays, &[3, 1, 0, 2]), vec![0, 60, 130, 10]);
        // Two tied first share first's and second's; the next is third.
        assert_eq!(split(&pays, &[0, 0, 2, 3]), vec![95, 95, 10, 0]);
        // Two tied second share second's and third's, by seat.
        assert_eq!(split(&pays, &[3, 0, 1, 1]), vec![0, 130, 35, 35]);
        // Three tied last share the last three, rounded down.
        assert_eq!(split(&pays, &[0, 1, 1, 1]), vec![130, 23, 23, 23]);
        // Everyone tied: the whole of it, shared.
        assert_eq!(split(&pays, &[0, 0, 0, 0]), vec![50, 50, 50, 50]);
        // A draw between two: the fee back each (Blend In's two people).
        assert_eq!(split(&[100, 0], &[0, 0]), vec![50, 50]);
        // Rounded down, never paying out more than there is.
        assert_eq!(split(&[100, 0, 0], &[0, 0, 0]), vec![33, 33, 33]);
        // A place past the end pays nothing, and a tie running past the
        // end shares only what there is.
        assert_eq!(split(&[100, 0], &[0, 1, 2]), vec![100, 0, 0]);
        assert_eq!(split(&[100, 40], &[0, 1, 1]), vec![100, 20, 20]);
        assert_eq!(split(&pays, &[]), Vec::<u32>::new());
        // However they tie, no more is paid than the places pay.
        for places in [[0, 0, 0, 3], [0, 1, 1, 3], [0, 0, 2, 2], [1, 0, 3, 1]] {
            let paid: u32 = split(&pays, &places).iter().sum();
            assert!(paid <= pays.iter().sum::<u32>(), "{places:?} paid {paid}");
        }
    }

    #[test]
    fn each_of_the_four_has_its_shape_and_the_older_games_none() {
        assert_eq!(shape(Minigame::BridgeWars), None);
        assert_eq!(shape(Minigame::HouseBuilder), None);
        let numbers = |game| {
            let shape = shape(game).expect("a game at a table");
            (shape.most, shape.fewest, shape.computer, shape.wait, shape.countdown, shape.results, shape.fee)
        };
        assert_eq!(numbers(Minigame::BlendIn), (4, 2, false, 15.0, 5.0, 15.0, 50));
        assert_eq!(numbers(Minigame::KnockOff), (4, 1, true, 10.0, 5.0, 15.0, 50));
        assert_eq!(numbers(Minigame::BridgeRace), (4, 1, true, 10.0, 5.0, 15.0, 50));
        assert_eq!(numbers(Minigame::LastStand), (4, 1, false, 15.0, 5.0, 20.0, 50));
        for game in Minigame::TABLES {
            let shape = shape(game).unwrap();
            assert!(shape.fewest >= 1 && shape.fewest <= shape.most, "{game:?}");
            assert!(shape.wait > 0.0 && shape.countdown > 0.0 && shape.results > 0.0, "{game:?}");
            // The computer comes as Bridge Wars' does.
            if shape.computer {
                assert_eq!(shape.wait, crate::bridge_wars::BOT_WAIT, "{game:?}");
            }
        }
    }

    #[test]
    fn how_many_seats_a_table_has_once_the_computer_has_come() {
        // Blend In and Last Stand: the people there are, and no more than four.
        for people in 1..=4 {
            assert_eq!(seats_for(Minigame::BlendIn, people), people);
            assert_eq!(seats_for(Minigame::LastStand, people), people);
        }
        assert_eq!(seats_for(Minigame::BlendIn, 6), 4);
        // Knock Off: every man for himself, always four.
        for people in 0..=4 {
            assert_eq!(seats_for(Minigame::KnockOff, people), 4);
        }
        // Bridge Race: one person is a one against one, more a two
        // against two; and the computer alone plays two against two.
        assert_eq!(seats_for(Minigame::BridgeRace, 1), 2);
        for people in [0, 2, 3, 4] {
            assert_eq!(seats_for(Minigame::BridgeRace, people), 4, "{people}");
        }
        // The older games fill every seat of theirs.
        assert_eq!(seats_for(Minigame::BridgeWars, 1), ARENA_SEATS);
        assert_eq!(seats_for(Minigame::HouseBuilder, 3), GAME_SEATS);
    }

    #[test]
    fn whoever_beats_more_than_half_of_the_others_comes_away_with_more_than_they_paid() {
        // Hajun: "Generally should be above 50% to win net positive coin,
        // and if not, net negative coin". Every way of coming out with no
        // ties, at every size of table each game is played at.
        let ways: [(Minigame, fn(&Results, &[bool]) -> Vec<u32>, usize); 6] = [
            (Minigame::BlendIn, blend_in::prizes, 2),
            (Minigame::BlendIn, blend_in::prizes, 3),
            (Minigame::BlendIn, blend_in::prizes, 4),
            (Minigame::KnockOff, knock_off::prizes, 4),
            (Minigame::BridgeRace, bridge_race::prizes, 2),
            (Minigame::BridgeRace, bridge_race::prizes, 4),
        ];
        for (game, prizes, seats) in ways {
            let fee = shape(game).unwrap().fee;
            let places: Vec<u8> = if game == Minigame::BridgeRace {
                // A team wins or loses together: the first team first, both of
                // it, and then the other.
                (0..seats).map(|seat| if bridge_race::team(seat) == 0 { 0 } else { (seats / 2) as u8 }).collect()
            } else {
                (0..seats as u8).collect()
            };
            let results = Results { places: places.clone(), ..default() };
            let paid = prizes(&results, &vec![true; seats]);
            assert_eq!(paid.len(), seats, "{game:?}");
            for seat in 0..seats {
                let beaten = places.iter().filter(|&&other| other > places[seat]).count();
                let more_than_half = beaten * 2 > seats - 1;
                assert_eq!(paid[seat] > fee, more_than_half, "{game:?} of {seats}: seat {seat}, place {}, paid {}", places[seat], paid[seat]);
                assert_ne!(paid[seat], fee, "{game:?} of {seats}: seat {seat} only has the fee back");
            }
        }
        // A draw gives everyone the fee back, in Blend In of two and Bridge Race.
        let draw = Results { places: vec![0, 0], ..default() };
        assert_eq!(blend_in::prizes(&draw, &[true, true]), vec![50, 50]);
        let draw = Results { places: vec![0; 4], ..default() };
        assert_eq!(bridge_race::prizes(&draw, &[true; 4]), vec![50; 4]);
        // Last Stand: the fee back for five waves cleared, less for fewer,
        // more for more, up to 400.
        let after = |waves: u32| last_stand::prizes(&Results { places: vec![0; 2], total: Some(waves), ..default() }, &[true, true]);
        assert_eq!(after(5), vec![50, 50]);
        assert_eq!(after(4), vec![40, 40]);
        assert_eq!(after(6), vec![60, 60]);
        assert_eq!(after(0), vec![0, 0]);
        assert_eq!(after(40), vec![400, 400]);
        assert_eq!(after(u32::MAX), vec![400, 400]);
        assert_eq!(last_stand::prizes(&Results { places: vec![0], total: None, ..default() }, &[true]), vec![0]);
    }

    #[test]
    fn the_computer_is_paid_nothing() {
        let results = Results {
            places: vec![1, 0, 3, 2],
            total: Some(12),
            ..default()
        };
        let people = [true, false, true, false];
        let none = [false; 4];
        let every: [fn(&Results, &[bool]) -> Vec<u32>; 4] =
            [blend_in::prizes, knock_off::prizes, bridge_race::prizes, last_stand::prizes];
        for prizes in every {
            let paid = prizes(&results, &people);
            assert_eq!(paid.len(), 4);
            assert_eq!((paid[1], paid[3]), (0, 0), "{paid:?}");
            assert!(prizes(&results, &none).iter().all(|&coins| coins == 0));
        }
        // Knock Off: the person second is paid second's, though the
        // computer came first.
        assert_eq!(knock_off::prizes(&results, &people), vec![60, 0, 0, 0]);
        // Bridge Race: the team that came first is paid, its people alone.
        let race = Results {
            places: vec![0, 2, 0, 2],
            ..default()
        };
        assert_eq!(bridge_race::prizes(&race, &[true, true, false, true]), vec![110, 0, 0, 0]);
        assert_eq!(last_stand::prizes(&results, &people), vec![120, 0, 120, 0]);
    }

    #[test]
    fn a_board_goes_whole_and_small() {
        // The most a board of four seats ever holds: every number as long
        // as it can be.
        let board = Board {
            game: Minigame::LastStand,
            id: u32::MAX,
            stage: Stage::Over(Results {
                places: vec![0, 1, 2, 3],
                scores: vec![u32::MAX; 4],
                prizes: vec![u32::MAX; 4],
                total: Some(u32::MAX),
            }),
            left: u16::MAX,
            people: 4,
            seats: 4,
        };
        let bytes = postcard::to_allocvec(&board).unwrap();
        assert!(bytes.len() < 100, "{} bytes", bytes.len());
        assert_eq!(postcard::from_bytes::<Board>(&bytes).unwrap(), board);
        // And while it waits, a handful.
        let waiting = Board {
            game: Minigame::BlendIn,
            id: 3,
            stage: Stage::Waiting,
            left: 12,
            people: 2,
            seats: 4,
        };
        let bytes = postcard::to_allocvec(&waiting).unwrap();
        assert!(bytes.len() <= 8, "{} bytes", bytes.len());
        assert_eq!(postcard::from_bytes::<Board>(&bytes).unwrap(), waiting);
        for stage in [Stage::Countdown, Stage::Playing, Stage::Over(Results::default())] {
            let board = Board { stage, ..waiting.clone() };
            assert_eq!(postcard::from_bytes::<Board>(&postcard::to_allocvec(&board).unwrap()).unwrap(), board);
        }
    }
}
