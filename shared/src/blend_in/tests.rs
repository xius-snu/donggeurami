//! Blend In's rules, round by round, and whole games played by stand-ins
//! for the people.

use std::collections::HashMap;

use super::*;
use crate::table::{Results, split};

/// The server's clock as a test's game begins.
const CLOCK: f64 = 1_794_000_000.0;
/// A test's frame.
const DT: f32 = 1.0 / 30.0;

fn members(people: usize) -> Vec<Member> {
    let names = ["Dubu 320", "Mochi 41", "Sol 7", "Hodu 1200"];
    let looks = ["girl", "girl_whiteblue", "boy_blackbrown", "boy"];
    (0..people)
        .map(|seat| Member {
            id: seat as u64 + 1,
            name: names[seat].into(),
            look: looks[seat].into(),
            ai: false,
        })
        .collect()
}

/// A game of `people`, begun.
fn begun(people: usize, seed: u64) -> Game {
    let mut game = Game::new(&members(people), seed, Crowd::new(seed));
    game.begin(CLOCK);
    game
}

/// Steps `game` for `secs`, and everything that happened.
fn run(game: &mut Game, secs: f32) -> Vec<Event> {
    let mut events = Vec::new();
    let mut gone = 0.0;
    while gone < secs - 1e-4 {
        events.extend(game.step(DT));
        gone += DT;
    }
    events
}

/// Steps `game` until it comes to `stage`, at most `most` seconds.
fn run_to(game: &mut Game, stage: BlendStage, most: f32) -> Vec<Event> {
    let mut events = Vec::new();
    let mut gone = 0.0;
    while game.stage() != stage {
        assert!(gone < most, "not at {stage:?} after {most} s, but at {:?}", game.stage());
        events.extend(game.step(DT));
        gone += DT;
    }
    events
}

/// Everyone says they are ready, and the game goes on to the seek.
fn to_seek(game: &mut Game) -> Vec<Event> {
    for seat in 0..game.seats.len() {
        game.ready(seat);
    }
    run_to(game, BlendStage::Seek, HIDE + 1.0)
}

/// The seat of someone other than the hider.
fn a_seeker(game: &Game) -> usize {
    let hider = game.hider().unwrap();
    (0..game.seats.len()).find(|&seat| seat != hider && !game.seats[seat].gone).unwrap()
}

/// Stands `seat` a metre from the body `who`, and the body's id.
fn beside(game: &mut Game, seat: usize, who: Who) -> u64 {
    let (at, _) = game.where_is(who);
    game.sync(seat, at + Vec3::X, 0.0);
    match who {
        Who::Crowd(index) => game.crowd.walkers()[index].guise.id,
        Who::Seat(other) => game.seats[other].worn.id,
    }
}

#[test]
fn every_person_hides_once_in_a_thrown_order() {
    let mut firsts = [0u32; 4];
    for people in 2..=4 {
        for seed in 0..12 {
            let mut game = begun(people, seed);
            let mut hiders = Vec::new();
            let mut events = Vec::new();
            for _ in 0..people {
                events.extend(to_seek(&mut game));
                events.extend(run_to(&mut game, BlendStage::Shown, SEEK + 1.0));
                events.extend(run(&mut game, SHOWN + DT));
            }
            for event in &events {
                if let Event::Choosing { round, hider } = event {
                    assert_eq!(usize::from(*round), hiders.len());
                    hiders.push(*hider);
                }
            }
            assert_eq!(game.stage(), BlendStage::Over, "{people} people, seed {seed}");
            let mut sorted = hiders.clone();
            sorted.sort();
            assert_eq!(sorted, (0..people).collect::<Vec<_>>(), "{hiders:?}");
            assert_eq!(game.round().rounds as usize, people);
            if people == 4 {
                firsts[hiders[0]] += 1;
            }
            // Nobody found anybody: each hider's point.
            assert_eq!(game.round().points, vec![1; people]);
            assert_eq!(events.iter().filter(|event| matches!(event, Event::Hidden { .. })).count(), people);
            assert_eq!(events.last(), Some(&Event::Over));
        }
    }
    // Who hides first is thrown: in twelve games of four, more than one seat.
    assert!(firsts.iter().filter(|&&first| first > 0).count() >= 2, "{firsts:?}");
}

#[test]
fn a_strike_on_the_hider_in_the_seek_finds_them_a_point_to_the_finder() {
    let mut game = begun(3, 1);
    to_seek(&mut game);
    let hider = game.hider().unwrap();
    let seeker = a_seeker(&game);
    run(&mut game, 12.3);
    let id = beside(&mut game, seeker, Who::Seat(hider));
    assert_eq!(game.strike(seeker, id), Struck::Found);
    let events = game.step(DT);
    assert!(
        matches!(events.as_slice(), [Event::Found { seeker: s, hider: h, after }] if *s == seeker && *h == hider && (*after - 12.3).abs() < 0.1),
        "{events:?}"
    );
    let round = game.round();
    assert_eq!(round.stage, BlendStage::Shown);
    assert_eq!(round.found, Some((seeker as u8, 12)));
    assert_eq!(round.unmasked, Some(id));
    let mut points = vec![0; 3];
    points[seeker] = 1;
    assert_eq!(round.points, points);
    // Shown from now: the choosing over at once, the hide, 12.3 s of seek.
    assert!((round.until - (CLOCK + f64::from(HIDE + 12.3 + SHOWN))).abs() < 0.2, "{}", round.until - CLOCK);
    // It is over: nothing more is found, and the next round comes.
    assert_eq!(game.strike(seeker, id), Struck::Nothing);
    run_to(&mut game, BlendStage::Choose, SHOWN + 1.0);
    let round = game.round();
    assert_eq!((round.round, round.found, round.unmasked), (1, None, None));
    assert_ne!(usize::from(round.hider), hider);
}

#[test]
fn a_wrong_guess_stuns_the_seeker_four_then_seven_then_ten_seconds_and_the_one_struck_staggers() {
    let mut game = begun(2, 3);
    to_seek(&mut game);
    let seeker = a_seeker(&game);
    for (guess, stun) in [(0, 4.0), (1, 7.0), (2, 10.0)] {
        let id = beside(&mut game, seeker, Who::Crowd(guess));
        let now = CLOCK + game.elapsed;
        assert_eq!(game.strike(seeker, id), Struck::Wrong { stun });
        assert!(matches!(game.crowd.walkers()[guess].doing(), Doing::Staggering { .. }));
        let until = game.stunned(seeker).expect("stunned");
        assert!((until - (now + f64::from(stun))).abs() < 1e-6);
        let events = game.step(DT);
        assert!(events.contains(&Event::Wrong { seeker, crowd: guess, until }), "{events:?}");
        // While stunned, nothing they strike counts, the hider least of all.
        let held = game.at(seeker);
        let other = beside(&mut game, seeker, Who::Crowd(guess + 5));
        assert_eq!(game.strike(seeker, other), Struck::Nothing);
        let hider = game.hider().unwrap();
        let them = beside(&mut game, seeker, Who::Seat(hider));
        assert_eq!(game.strike(seeker, them), Struck::Nothing);
        // Back where they were struck, where they are held, their device
        // having stood them still all along.
        game.sync(seeker, held.0, held.1);
        run(&mut game, stun - 2.0 * DT);
        assert!(game.stunned(seeker).is_some(), "{guess}");
        run(&mut game, 2.0 * DT);
        assert_eq!(game.stunned(seeker), None, "{guess}");
    }
    // Every round begins with a clean sheet.
    run_to(&mut game, BlendStage::Choose, SEEK + SHOWN + 1.0);
    to_seek(&mut game);
    let seeker = a_seeker(&game);
    let id = beside(&mut game, seeker, Who::Crowd(9));
    assert_eq!(game.strike(seeker, id), Struck::Wrong { stun: STUN_FIRST });
}

#[test]
fn a_strike_comes_to_nothing_but_from_a_free_seeker_on_a_body_in_reach_in_the_seek() {
    let mut game = begun(3, 5);
    let hider = game.hider().unwrap();
    let seeker = a_seeker(&game);
    // Choosing, and hiding: nothing.
    let id = beside(&mut game, seeker, Who::Seat(hider));
    assert_eq!(game.strike(seeker, id), Struck::Nothing);
    for seat in 0..3 {
        game.ready(seat);
    }
    run_to(&mut game, BlendStage::Hide, 1.0);
    let id = beside(&mut game, seeker, Who::Seat(hider));
    assert_eq!(game.strike(seeker, id), Struck::Nothing);
    let crowd = beside(&mut game, seeker, Who::Crowd(0));
    assert_eq!(game.strike(seeker, crowd), Struck::Nothing);
    run_to(&mut game, BlendStage::Seek, HIDE + 1.0);
    // The hider has no swing.
    let crowd = beside(&mut game, hider, Who::Crowd(1));
    assert_eq!(game.strike(hider, crowd), Struck::Nothing);
    // Another seeker is let be.
    let other = (0..3).find(|&seat| seat != hider && seat != seeker).unwrap();
    let them = beside(&mut game, seeker, Who::Seat(other));
    assert_eq!(game.strike(seeker, them), Struck::Nothing);
    // Nobody by that id; nobody in reach.
    assert_eq!(game.strike(seeker, 12345), Struck::Nothing);
    let id = game.seats[hider].worn.id;
    let (at, _) = game.at(hider);
    let away = at + Vec3::new(STRIKE_SLACK + 0.5, 0.0, 0.0);
    game.sync(seeker, away, 0.0);
    assert_eq!(game.strike(seeker, id), Struck::Nothing);
    // Nor anyone who has left.
    game.leave(other);
    let them = game.seats[other].worn.id;
    assert_eq!(game.who(them), None);
    assert_eq!(game.strike(seeker, them), Struck::Nothing);
    assert!(game.step(DT).iter().all(|event| !matches!(event, Event::Found { .. } | Event::Wrong { .. })));
    // In reach at last.
    game.sync(seeker, at + Vec3::Z * 2.0, 0.0);
    assert_eq!(game.strike(seeker, id), Struck::Found);
}

#[test]
fn the_choosing_offers_the_hider_a_crowd_s_guise_and_each_seeker_their_own() {
    for seed in 0..10 {
        let mut game = begun(4, seed);
        let hider = game.hider().unwrap();
        // Said once, as the round began.
        assert_eq!(game.step(DT), vec![Event::Choosing { round: 0, hider }]);
        assert!(game.step(DT).is_empty());
        let accounts = members(4);
        for seat in 0..4 {
            let (name, look) = game.choice(seat).unwrap();
            if seat == hider {
                let boy = crate::BOY_LOOKS.contains(&look);
                assert!(boy || crate::GIRL_LOOKS.contains(&look), "{look}");
                assert!(ai_names(boy).contains(&name), "{name} for a {look}");
            } else {
                assert_eq!((name, look), (accounts[seat].name.as_str(), accounts[seat].look.as_str()));
            }
        }
    }
    let mut game = begun(3, 2);
    game.step(DT);
    let hider = game.hider().unwrap();
    let seeker = a_seeker(&game);
    let other = (0..3).find(|&seat| seat != hider && seat != seeker).unwrap();
    // No name, a rude one, and one somebody here goes by, whatever its
    // capitals: refused, and said so; a look nobody wears: not even that.
    let theirs = game.choice(other).unwrap().0.to_uppercase();
    for name in ["x", "  ", "Bitch", theirs.as_str()] {
        assert!(!game.disguise(seeker, name, "girl"), "{name}");
        assert_eq!(game.step(DT), vec![Event::Refused { seat: seeker }], "{name}");
    }
    assert!(!game.disguise(seeker, "Haru", "zombie"));
    assert!(game.step(DT).is_empty());
    // A name, cleaned as names are.
    assert!(game.disguise(seeker, "  Haneul ", "girl_yellowpink"));
    assert_eq!(game.choice(seeker), Some(("Haneul", "girl_yellowpink")));
    // Now nobody else can have it.
    assert!(!game.disguise(hider, "haneul", "boy"));
    // Everyone ready: the hide, in the next step, and everyone in what
    // they chose.
    for seat in 0..3 {
        assert!(game.ready(seat));
        assert!(game.round().ready[seat]);
    }
    let events = game.step(DT);
    assert_eq!(game.stage(), BlendStage::Hide);
    assert!(events.contains(&Event::Dressed));
    assert_eq!(game.seats[seeker].worn.name, "Haneul");
    assert_eq!(game.round().ready, vec![false; 3]);
    // Choosing is over: nothing is taken now.
    assert!(!game.disguise(seeker, "Nabi", "girl"));
    assert!(!game.ready(seeker));
}

#[test]
fn nobody_ready_the_hide_comes_when_the_choosing_runs_out_in_what_each_was_offered() {
    let mut game = begun(2, 8);
    let offered: Vec<(String, String)> = (0..2).map(|seat| {
        let (name, look) = game.choice(seat).unwrap();
        (name.to_owned(), look.to_owned())
    }).collect();
    game.ready(0);
    run(&mut game, CHOOSE - 0.5);
    assert_eq!(game.stage(), BlendStage::Choose);
    run(&mut game, 0.5 + DT);
    assert_eq!(game.stage(), BlendStage::Hide);
    for seat in 0..2 {
        assert_eq!((&game.seats[seat].worn.name, &game.seats[seat].worn.look), (&offered[seat].0, &offered[seat].1));
    }
    // One whose device has gone holds nobody up.
    let mut game = game_after_a_round(3, 4);
    let hider = game.hider().unwrap();
    let away = a_seeker(&game);
    game.away(away, true);
    for seat in (0..3).filter(|&seat| seat != away) {
        game.ready(seat);
    }
    game.step(DT);
    assert_eq!(game.stage(), BlendStage::Hide, "{hider}");
}

/// A game of `people` whose first round has been played out, unfound.
fn game_after_a_round(people: usize, seed: u64) -> Game {
    let mut game = begun(people, seed);
    to_seek(&mut game);
    run_to(&mut game, BlendStage::Choose, SEEK + SHOWN + 1.0);
    game
}

#[test]
fn every_body_goes_by_a_new_id_every_round_and_the_crowd_by_new_names() {
    let mut game = begun(4, 6);
    let mut before: Vec<Guise> = game.bodies().map(|figure| figure.guise.clone()).collect();
    for round in 0..4 {
        to_seek(&mut game);
        let now: Vec<Figure> = game.bodies().collect();
        assert_eq!(now.len(), CROWD + 4);
        let mut ids: Vec<u64> = now.iter().map(|figure| figure.guise.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), CROWD + 4, "round {round}: two alike");
        for figure in &now {
            assert!((FIRST_ID..FIRST_ID + IDS).contains(&figure.guise.id), "{:?}", figure.guise);
            assert!(before.iter().all(|was| was.id != figure.guise.id), "an id kept from before");
            assert_eq!(figure.guise.member().ai, false);
            assert_eq!(game.who(figure.guise.id), Some(figure.who));
        }
        // No two by one name, the players included.
        let mut names: Vec<String> = now.iter().map(|figure| figure.guise.name.to_lowercase()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), CROWD + 4, "round {round}");
        // The crowd by names mostly new, and in looks mostly new.
        let crowd_before = &before[..CROWD];
        let renamed = now[..CROWD].iter().zip(crowd_before).filter(|(now, was)| now.guise.name != was.name).count();
        assert!(renamed > CROWD / 2, "round {round}: {renamed} renamed");
        before = now.iter().map(|figure| figure.guise.clone()).collect();
        run_to(&mut game, BlendStage::Shown, SEEK + 1.0);
        // The hider's body is told, now that it is shown.
        let hider = game.hider().unwrap();
        assert_eq!(game.round().unmasked, Some(game.seats[hider].worn.id));
        run(&mut game, SHOWN + DT);
    }
    assert_eq!(game.stage(), BlendStage::Over);
}

#[test]
fn the_seekers_wait_where_they_came_in_and_the_hider_far_from_them_among_the_crowd() {
    for seed in 0..15 {
        let people = 2 + seed as usize % 3;
        let mut game = begun(people, seed);
        for seat in 0..people {
            game.ready(seat);
        }
        let events = game.step(DT);
        let hider = game.hider().unwrap();
        let places: HashMap<usize, Vec3> = events
            .iter()
            .filter_map(|event| match event {
                Event::Place { seat, at, .. } => Some((*seat, *at)),
                _ => None,
            })
            .collect();
        assert_eq!(places.len(), people, "{events:?}");
        let hidden = places[&hider];
        for seat in (0..people).filter(|&seat| seat != hider) {
            let (came, _) = arrival(seat);
            assert!(places[&seat].xz().distance(came.xz()) < 1e-4);
            assert!(hidden.xz().distance(places[&seat].xz()) >= HIDER_CLEAR, "seed {seed}");
        }
        let grid = grid();
        assert!(grid.clear(hidden.xz()) && hidden.xz().length() < RING, "seed {seed}: {hidden}");
        assert_eq!(Some(hidden.y), grid.height(hidden.xz()));
        let nearest = game.crowd.walkers().iter().map(|walker| walker.at.xz().distance(hidden.xz())).fold(f32::INFINITY, f32::min);
        assert!(nearest >= APART - 0.3 && nearest <= BESIDE.1 + 1.5, "seed {seed}: the nearest of the crowd {nearest} m off");
        // A seeker gone anywhere while they could not is put back as the
        // seek begins; one where they were is let be.
        let seeker = a_seeker(&game);
        let (at, yaw) = game.at(seeker);
        game.sync(seeker, at + Vec3::new(6.0, 0.0, 0.0), yaw);
        let events = run_to(&mut game, BlendStage::Seek, HIDE + 1.0);
        let back: Vec<&Event> = events.iter().filter(|event| matches!(event, Event::Place { .. })).collect();
        assert_eq!(back, [&Event::Place { seat: seeker, at, yaw }], "seed {seed}");
    }
}

#[test]
fn nobody_stays_past_the_ring() {
    let mut game = begun(2, 9);
    to_seek(&mut game);
    let hider = game.hider().unwrap();
    game.sync(hider, Vec3::new(0.0, 0.25, RING + 5.0), 1.0);
    let events = game.step(DT);
    let [Event::Place { seat, at, yaw }] = events.as_slice() else {
        panic!("{events:?}");
    };
    assert_eq!((*seat, *yaw), (hider, 1.0));
    assert!(at.xz().length() < RING && grid().clear(at.xz()), "{at}");
    // Not again until it has had time to reach them.
    game.sync(hider, Vec3::new(0.0, 0.25, RING + 5.0), 1.0);
    assert!(run(&mut game, PUT_BACK_EVERY - 0.1).is_empty());
    assert_eq!(run(&mut game, 0.2).len(), 1);
    // A metre past it is no matter.
    game.sync(hider, Vec3::new(0.0, 0.25, RING + 1.0), 1.0);
    assert!(run(&mut game, PUT_BACK_EVERY + 0.1).is_empty());
}

#[test]
fn points_place_everyone_and_two_who_both_find_or_both_hide_draw() {
    // Two: each finds the other.
    let mut game = begun(2, 10);
    for _ in 0..2 {
        to_seek(&mut game);
        let seeker = a_seeker(&game);
        let hider = game.hider().unwrap();
        let id = beside(&mut game, seeker, Who::Seat(hider));
        assert_eq!(game.strike(seeker, id), Struck::Found);
        run(&mut game, SHOWN + DT);
    }
    assert_eq!(game.stage(), BlendStage::Over);
    let (places, points, total) = game.outcome().expect("over");
    assert_eq!((places.as_slice(), points.as_slice(), total), ([0, 0].as_slice(), [1, 1].as_slice(), None));
    let results = Results { places, scores: points, ..Default::default() };
    assert_eq!(prizes(&results, &[true, true]), vec![50, 50]);
    // Two: neither finds the other.
    let mut game = begun(2, 11);
    for _ in 0..2 {
        to_seek(&mut game);
        run_to(&mut game, BlendStage::Shown, SEEK + 1.0);
        run(&mut game, SHOWN + DT);
    }
    assert_eq!(game.outcome().unwrap().0, vec![0, 0]);
    // Two: one hides out the time and finds the other.
    let mut game = begun(2, 12);
    let first = game.hider().unwrap();
    to_seek(&mut game);
    run_to(&mut game, BlendStage::Choose, SEEK + SHOWN + 1.0);
    to_seek(&mut game);
    let id = beside(&mut game, first, Who::Seat(1 - first));
    assert_eq!(game.strike(first, id), Struck::Found);
    run_to(&mut game, BlendStage::Over, SHOWN + 1.0);
    let (places, points, _) = game.outcome().unwrap();
    let mut expected = (vec![1, 1], vec![0, 0]);
    expected.0[first] = 0;
    expected.1[first] = 2;
    assert_eq!((places, points.iter().map(|&p| p as u8).collect::<Vec<_>>()), expected);
    // Four, by points: 2, 1, 1, 0 are first, second twice, and fourth, and
    // the two second share second's and third's coins.
    let mut game = begun(4, 13);
    for (seat, points) in [2, 1, 1, 0].into_iter().enumerate() {
        game.seats[seat].points = points;
    }
    game.over();
    let (places, ..) = game.outcome().unwrap();
    assert_eq!(places, vec![0, 1, 1, 3]);
    assert_eq!(split(&[120, 70, 10, 0], &places), vec![120, 40, 40, 0]);
    // Not over, no outcome.
    assert_eq!(begun(3, 1).outcome(), None);
}

#[test]
fn one_who_leaves_gives_up_their_turn_and_a_hider_who_leaves_their_round() {
    // A seeker yet to hide leaves: a round fewer, the round going on.
    let mut game = begun(4, 14);
    to_seek(&mut game);
    let hider = game.hider().unwrap();
    let later = game.order[2];
    game.leave(later);
    assert_eq!(game.round().rounds, 3);
    assert_eq!(game.stage(), BlendStage::Seek);
    assert!(game.bodies().all(|figure| figure.who != Who::Seat(later)));
    // The hider leaves: no point, and the next round.
    game.leave(hider);
    let events = game.step(DT);
    assert_eq!(events.first(), Some(&Event::Void { hider }));
    assert!(matches!(events.get(1), Some(Event::Choosing { round: 1, .. })), "{events:?}");
    assert_eq!(game.round().points, vec![0; 4]);
    assert_eq!(game.round().rounds, 3);
    // Down to one: over at once, places by points so far.
    let next = game.hider().unwrap();
    game.leave(next);
    assert_eq!(game.stage(), BlendStage::Over);
    assert_eq!(game.step(DT), vec![Event::Over]);
    assert_eq!(game.outcome().unwrap().0, vec![0; 4]);
    // A hider whose device has gone a while gives up their round; one
    // back in time does not.
    let mut game = begun(3, 15);
    to_seek(&mut game);
    let hider = game.hider().unwrap();
    game.away(hider, true);
    run(&mut game, AWAY - 1.0);
    game.away(hider, false);
    game.away(hider, true);
    let events = run(&mut game, AWAY - 1.0);
    assert!(!events.iter().any(|event| matches!(event, Event::Void { .. })));
    let events = run(&mut game, 1.5);
    assert!(events.contains(&Event::Void { hider }), "{events:?}");
    assert_eq!(game.round().round, 1);
    // Gone before the play begins: never seated in the order.
    let mut game = Game::new(&members(3), 1, Crowd::new(1));
    game.leave(1);
    game.begin(CLOCK);
    assert_eq!(game.round().rounds, 2);
    let mut game = Game::new(&members(2), 1, Crowd::new(1));
    game.leave(0);
    game.begin(CLOCK);
    assert_eq!(game.stage(), BlendStage::Over);
}

#[test]
fn what_the_room_is_sent_is_small_and_comes_back_as_it_went() {
    let round = BlendRound {
        round: 3,
        rounds: 4,
        hider: 3,
        stage: BlendStage::Shown,
        until: CLOCK + 1234.5,
        points: vec![u8::MAX; 4],
        ready: vec![true; 4],
        found: Some((2, u16::MAX)),
        unmasked: Some(FIRST_ID + IDS - 1),
    };
    let bytes = postcard::to_allocvec(&round).unwrap();
    assert!(bytes.len() <= 80, "{} bytes", bytes.len());
    assert_eq!(postcard::from_bytes::<BlendRound>(&bytes).unwrap(), round);
    // As a game sends it, round by round.
    let mut game = begun(4, 16);
    let sent = postcard::to_allocvec(&game.round()).unwrap();
    assert!(sent.len() <= 40, "{} bytes", sent.len());
    println!("a round of four: {} bytes at the most, {} as the first is chosen", bytes.len(), sent.len());
    // The same until something has changed.
    assert_eq!(game.round(), game.round());
    let before = game.round();
    game.step(DT);
    assert_eq!(game.round(), before, "nothing has changed in a step of choosing");
    for ask in [Ask::Ready, Ask::Disguise { name: "Mochi".into(), look: "girl".into() }] {
        let wrapped = crate::Ask::BlendIn(ask.clone());
        let back: crate::Ask = postcard::from_bytes(&postcard::to_allocvec(&wrapped).unwrap()).unwrap();
        assert!(matches!(back, crate::Ask::BlendIn(back) if back == ask));
    }
    for tell in [Tell::Stunned { until: CLOCK + 4.0 }, Tell::Refused, Tell::Wear { name: "Dubu".into(), look: "girl".into() }] {
        let wrapped = crate::Tell::BlendIn(tell.clone());
        let back: crate::Tell = postcard::from_bytes(&postcard::to_allocvec(&wrapped).unwrap()).unwrap();
        assert!(matches!(back, crate::Tell::BlendIn(back) if back == tell));
    }
}

// ------------------------------------------------- whole games, stand-ins

/// How a stand-in for the hider moves.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Hiding {
    /// As one of the crowd does: by the crowd's own mind.
    Crowdlike,
    /// Never at all.
    Still,
    /// Never stopping: from one place to the next, on and on.
    Restless,
    /// As the crowd does, until a seeker comes near; then straight away
    /// from them, as long as they come on.
    Fleeing,
}

/// How near a seeker comes before a fleeing stand-in goes, in metres.
const FLEE_FROM: f32 = 8.0;

/// How a stand-in for a seeker goes about it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Seeking {
    /// Strikes whoever is nearest, now and then.
    Nearest,
    /// Watches for whoever walks longer than any of the crowd ever does,
    /// and goes for them first.
    Watchful,
}

/// A stand-in for a person.
struct StandIn {
    at: Vec3,
    yaw: f32,
    mind: Mind,
    /// Where a restless hider is going.
    going: Option<Vec2>,
    /// The body a seeker is going for, and how long it waits before it
    /// strikes once there.
    target: Option<u64>,
    wait: f32,
    struck: Vec<u64>,
}

/// How a whole game went.
#[derive(Clone, PartialEq, Debug, Default)]
struct Chronicle {
    hiders: Vec<usize>,
    points: Vec<u8>,
    places: Vec<u8>,
    found: u32,
    hidden: u32,
    voids: u32,
    found_after: Vec<f32>,
    wrong: u32,
    secs: f32,
    events: usize,
}

/// A whole game of `people` stand-ins, the hider moving as `hiding` has
/// it and the seekers looking as `seeking` does, at a seeker's pace.
fn play(people: usize, seed: u64, hiding: Hiding, seeking: Seeking) -> Chronicle {
    play_at(people, seed, hiding, seeking, SEEKER_PACE)
}

/// The same, the seekers going `pace` metres a second.
fn play_at(people: usize, seed: u64, hiding: Hiding, seeking: Seeking, pace: f32) -> Chronicle {
    let mut game = begun(people, seed);
    let grid = grid();
    let mut dice = Dice::new(seed.wrapping_mul(31).wrapping_add(7));
    let mut stand_ins: Vec<StandIn> = (0..people)
        .map(|seat| {
            let (at, yaw) = game.at(seat);
            StandIn {
                at,
                yaw,
                mind: Mind::new(&mut dice),
                going: None,
                target: None,
                wait: 0.0,
                struck: Vec::new(),
            }
        })
        .collect();
    // How long each body has been on the move without stopping, as anyone
    // watching sees it.
    let mut moving: HashMap<u64, (Vec2, f32, f32)> = HashMap::new();
    let mut chronicle = Chronicle::default();
    let most = people as f32 * (CHOOSE + HIDE + SEEK + SHOWN) + 1.0;
    let mut ready_at: Vec<f32> = vec![0.0; people];
    // (Not the stage it is at, so that the first choosing is seen to begin.)
    let mut stage = BlendStage::Over;
    while game.stage() != BlendStage::Over {
        assert!(chronicle.secs < most, "seed {seed}: no end after {} s", chronicle.secs);
        let hider = game.hider();
        if game.stage() != stage {
            stage = game.stage();
            if stage == BlendStage::Choose {
                // Ready after a thrown while, now and then never.
                for ready in &mut ready_at {
                    *ready = if dice.roll() < 0.1 { f32::INFINITY } else { dice.between(1.0, 20.0) };
                }
            }
            if stage == BlendStage::Seek {
                moving.clear();
                for stand_in in &mut stand_ins {
                    stand_in.struck.clear();
                    stand_in.target = None;
                    stand_in.going = None;
                }
            }
        }
        let seen: Vec<(u64, Who, Vec2)> = game.bodies().map(|figure| (figure.guise.id, figure.who, figure.at.xz())).collect();
        for seat in 0..people {
            let stand_in = &mut stand_ins[seat];
            match game.stage() {
                BlendStage::Choose => {
                    ready_at[seat] -= DT;
                    if ready_at[seat] <= 0.0 {
                        game.ready(seat);
                    }
                }
                BlendStage::Hide | BlendStage::Seek if hider == Some(seat) => match hiding {
                    Hiding::Crowdlike => {
                        stand_in.mind.step(&mut stand_in.at, &mut stand_in.yaw, &grid, &mut dice, DT, &|_| true);
                    }
                    Hiding::Still => {}
                    Hiding::Restless => {
                        let here = stand_in.at.xz();
                        let to = *stand_in.going.get_or_insert_with(|| {
                            let open = game.crowd.open();
                            open[pick(&mut dice, open.len())]
                        });
                        let stride = WALK_SPEED * DT;
                        let there = if here.distance(to) <= stride {
                            stand_in.going = None;
                            to
                        } else {
                            here + (to - here).normalize() * stride
                        };
                        stand_in.at = Vec3::new(there.x, grid.height(there).unwrap_or(stand_in.at.y), there.y);
                    }
                    Hiding::Fleeing => {
                        let here = stand_in.at.xz();
                        let chaser = (0..people)
                            .filter(|&other| other != seat && game.seeking(other))
                            .map(|other| game.at(other).0.xz())
                            .min_by(|a, b| a.distance(here).total_cmp(&b.distance(here)));
                        match chaser {
                            Some(chaser) if game.stage() == BlendStage::Seek && chaser.distance(here) < FLEE_FROM => {
                                // Away, or as near away as the ground goes.
                                let away = (here - chaser).normalize_or(Vec2::X);
                                let way = [0.0f32, 0.4, -0.4, 0.8, -0.8, 1.2, -1.2, 1.6, -1.6]
                                    .into_iter()
                                    .map(|turn| Vec2::from_angle(turn).rotate(away))
                                    .find(|&way| walks(&grid, stand_in.at, way, 1.0));
                                if let Some(way) = way {
                                    let there = here + way * WALK_SPEED * DT;
                                    stand_in.at = Vec3::new(there.x, grid.height(there).unwrap_or(stand_in.at.y), there.y);
                                    stand_in.yaw = yaw_of(way);
                                }
                            }
                            _ => {
                                stand_in.mind.step(&mut stand_in.at, &mut stand_in.yaw, &grid, &mut dice, DT, &|_| true);
                            }
                        }
                    }
                },
                BlendStage::Seek if game.stunned(seat).is_none() => {
                    let suspects = |id: &u64| moving.get(id).is_some_and(|&(_, _, longest)| longest > WALK_MAX / WALK_SPEED + 0.3);
                    let candidates: Vec<&(u64, Who, Vec2)> = seen
                        .iter()
                        .filter(|(id, who, _)| {
                            let a_seeker = matches!(who, Who::Seat(other) if Some(*other) != hider);
                            !a_seeker && !stand_in.struck.contains(id)
                        })
                        .collect();
                    let here = stand_in.at.xz();
                    let wanted = stand_in.target.and_then(|id| candidates.iter().find(|(seen, ..)| *seen == id));
                    let target = match (wanted, seeking) {
                        (Some(target), _) => Some(*target),
                        (None, Seeking::Watchful) if candidates.iter().any(|(id, ..)| suspects(id)) => candidates
                            .iter()
                            .filter(|(id, ..)| suspects(id))
                            .min_by(|a, b| a.2.distance(here).total_cmp(&b.2.distance(here)))
                            .copied(),
                        (None, _) => candidates.iter().min_by(|a, b| a.2.distance(here).total_cmp(&b.2.distance(here))).copied(),
                    };
                    let Some(&(id, _, there)) = target else {
                        continue;
                    };
                    if stand_in.target != Some(id) {
                        stand_in.target = Some(id);
                        // A suspect is struck as soon as they are reached;
                        // anyone else after a look at them.
                        stand_in.wait = if suspects(&id) { 0.0 } else { dice.between(0.3, 3.0) };
                    }
                    if here.distance(there) > 1.8 {
                        let way = (there - here).normalize();
                        let next = here + way * pace * DT;
                        stand_in.at = Vec3::new(next.x, stand_in.at.y, next.y);
                        stand_in.yaw = yaw_of(way);
                    } else {
                        stand_in.wait -= DT;
                        if stand_in.wait <= 0.0 {
                            match game.strike(seat, id) {
                                Struck::Nothing => {}
                                Struck::Found => chronicle.found_after.push((game.elapsed - game.seek_from) as f32),
                                Struck::Wrong { .. } => chronicle.wrong += 1,
                            }
                            stand_in.struck.push(id);
                            stand_in.target = None;
                        }
                    }
                }
                _ => {}
            }
            game.sync(seat, stand_in.at, stand_in.yaw);
        }
        for event in game.step(DT) {
            chronicle.events += 1;
            match event {
                Event::Place { seat, at, yaw } => {
                    stand_ins[seat].at = at;
                    stand_ins[seat].yaw = yaw;
                }
                Event::Choosing { hider, .. } => chronicle.hiders.push(hider),
                Event::Found { .. } => chronicle.found += 1,
                Event::Hidden { .. } => chronicle.hidden += 1,
                Event::Void { .. } => chronicle.voids += 1,
                _ => {}
            }
        }
        for figure in game.bodies() {
            let at = figure.at.xz();
            let entry = moving.entry(figure.guise.id).or_insert((at, 0.0, 0.0));
            entry.1 = if at.distance(entry.0) > 1e-3 { entry.1 + DT } else { 0.0 };
            entry.2 = entry.2.max(entry.1);
            entry.0 = at;
        }
        chronicle.secs += DT;
    }
    let (places, points, _) = game.outcome().expect("over");
    chronicle.places = places;
    chronicle.points = points.into_iter().map(|points| points as u8).collect();
    chronicle
}

#[test]
fn whole_games_of_stand_ins_end_with_everyone_having_hidden_once() {
    let mut found = HashMap::<(Hiding, Seeking), (u32, u32)>::new();
    let kinds = [
        (Hiding::Crowdlike, Seeking::Nearest),
        (Hiding::Crowdlike, Seeking::Watchful),
        (Hiding::Restless, Seeking::Watchful),
        (Hiding::Still, Seeking::Nearest),
    ];
    for seed in 0..8u64 {
        let people = 2 + seed as usize % 3;
        for (hiding, seeking) in kinds {
            let chronicle = play(people, seed, hiding, seeking);
            let mut hiders = chronicle.hiders.clone();
            hiders.sort();
            assert_eq!(hiders, (0..people).collect::<Vec<_>>(), "{chronicle:?}");
            assert_eq!(chronicle.found + chronicle.hidden, people as u32, "{chronicle:?}");
            assert_eq!(chronicle.voids, 0);
            assert_eq!(chronicle.points.iter().map(|&points| u32::from(points)).sum::<u32>(), people as u32);
            assert!(chronicle.found_after.iter().all(|&after| (0.0..=SEEK).contains(&after)));
            let tally = found.entry((hiding, seeking)).or_default();
            tally.0 += chronicle.found;
            tally.1 += people as u32;
        }
    }
    let rate = |kind| {
        let (found, rounds) = found[&kind];
        found as f32 / rounds as f32
    };
    println!("found: {:?}", kinds.map(|kind| (kind, rate(kind))));
    // Whoever never stops walking stands out from the crowd to anyone
    // watching for it; whoever walks as the crowd does, far less.
    assert!(rate((Hiding::Restless, Seeking::Watchful)) > 0.8);
    assert!(rate((Hiding::Crowdlike, Seeking::Watchful)) < rate((Hiding::Restless, Seeking::Watchful)) - 0.2);
    // And striking whoever is nearest finds a hider now and then.
    assert!(rate((Hiding::Crowdlike, Seeking::Nearest)) > 0.0);
    // The same seed, the same game.
    assert_eq!(play(3, 4, Hiding::Crowdlike, Seeking::Watchful), play(3, 4, Hiding::Crowdlike, Seeking::Watchful));
}

#[test]
fn a_hider_who_walks_off_from_a_seeker_is_caught_as_often_as_one_who_walks_on_as_the_crowd_does() {
    // Games of three and of four, where seekers can cut one off.
    let rate = |hiding: Hiding, pace: f32| {
        let (mut found, mut rounds) = (0, 0);
        for seed in 0..24u64 {
            let people = 3 + seed as usize % 2;
            found += play_at(people, seed, hiding, Seeking::Watchful, pace).found;
            rounds += people as u32;
        }
        found as f32 / rounds as f32
    };
    let (walks_on, walks_off) = (rate(Hiding::Crowdlike, SEEKER_PACE), rate(Hiding::Fleeing, SEEKER_PACE));
    let (walks_on_walked, walks_off_walked) = (rate(Hiding::Crowdlike, WALK_SPEED), rate(Hiding::Fleeing, WALK_SPEED));
    println!(
        "found, seekers running: {:.0}% walking on, {:.0}% walking off; seekers walking: {:.0}%, {:.0}%",
        walks_on * 100.0,
        walks_off * 100.0,
        walks_on_walked * 100.0,
        walks_off_walked * 100.0
    );
    assert!(walks_off > walks_on * 0.6, "{walks_off} found walking off, {walks_on} walking on");
    assert!(walks_off > walks_off_walked, "{walks_off} found by seekers running, {walks_off_walked} walking");
}

/// Plays many whole games of stand-ins and prints how they went, for
/// tuning: `cargo test -p roundtown_net many_games_of_blend_in --
/// --ignored --nocapture`, `RT_GAMES` games of each kind (20), from the
/// seed `RT_FROM` (100).
#[test]
#[ignore]
fn many_games_of_blend_in() {
    let games: u64 = std::env::var("RT_GAMES").ok().and_then(|n| n.parse().ok()).unwrap_or(20);
    let from: u64 = std::env::var("RT_FROM").ok().and_then(|n| n.parse().ok()).unwrap_or(100);
    println!("people  hider      seekers   pace  games  found  after(s)  wrong/seeker-round  draws  minutes");
    for people in 2..=4 {
        for (hiding, seeking, pace) in [
            (Hiding::Crowdlike, Seeking::Nearest, WALK_SPEED),
            (Hiding::Crowdlike, Seeking::Watchful, WALK_SPEED),
            (Hiding::Still, Seeking::Watchful, WALK_SPEED),
            (Hiding::Restless, Seeking::Watchful, WALK_SPEED),
            (Hiding::Fleeing, Seeking::Watchful, WALK_SPEED),
            (Hiding::Crowdlike, Seeking::Nearest, crate::bridge_wars::RUN_SPEED),
            (Hiding::Fleeing, Seeking::Watchful, crate::bridge_wars::RUN_SPEED),
        ] {
            let (mut found, mut after, mut wrong, mut draws, mut secs) = (0, Vec::new(), 0, 0, 0.0);
            for seed in from..from + games {
                let chronicle = play_at(people, seed, hiding, seeking, pace);
                found += chronicle.found;
                after.extend(chronicle.found_after);
                wrong += chronicle.wrong;
                draws += u32::from(chronicle.places.iter().all(|&place| place == 0));
                secs += chronicle.secs;
            }
            let rounds = (games as usize * people) as f32;
            println!(
                "{people:>6}  {:<9}  {:<8}  {pace:>4.0}  {games:>5}  {:>4.0}%  {:>8.0}  {:>18.2}  {:>4.0}%  {:>7.1}",
                format!("{hiding:?}"),
                format!("{seeking:?}"),
                found as f32 / rounds * 100.0,
                after.iter().sum::<f32>() / after.len().max(1) as f32,
                wrong as f32 / (rounds * (people - 1) as f32),
                draws as f32 / games as f32 * 100.0,
                secs / games as f32 / 60.0,
            );
        }
    }
}
