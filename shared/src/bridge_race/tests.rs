//! Bridge Race's rules, one by one, and whole games played by the computer
//! alone.

use super::*;
use crate::bridge_wars::DYNAMITE_FURTHEST;
use crate::nav::Span;

/// One frame of a game played in a test, in seconds: the server's.
const FRAME: f32 = 1.0 / 30.0;

/// How a whole game of the computer's alone went.
#[derive(Clone, Debug)]
struct Story {
    seed: u64,
    seats: usize,
    winner: Option<u8>,
    finish: Finish,
    over: f32,
    tally: Tally,
    stalls: u32,
    pinned: u32,
    kinds: Vec<&'static str>,
    laid: Vec<f32>,
    /// Falls nobody of the other team had a hand in.
    unforced: u32,
    /// Knocks one of a team gave another of the same: never.
    teammates: u32,
    /// The longest anyone of the computer's stood in one place with
    /// somewhere to walk to, in seconds, and who, and until when: for
    /// telling that nobody is held fast.
    stuck: f32,
    stuck_by: (usize, f32),
    /// Where each that got nowhere last did, and heading where.
    stalled: Vec<(usize, Vec2, Vec2)>,
    /// How far short of the golden island each team's bridge was, in
    /// metres, as the last of them to come onto it before the team first
    /// stood on it all together came onto it, the one the team waited for
    /// (a stick blowing the end off it after is nothing to how they went
    /// over): none, gone over by its bridge; a step's worth, the last plank
    /// hopped; more, the void jumped, or a knock that carried them over.
    short: [Option<f32>; 2],
    /// How many short ways of their own onto the golden island the
    /// computer's players began, off their team's way; and how many of
    /// those from a bridge of the other team's: never.
    short_ways: u32,
    from_theirs: u32,
}

/// How far short of the golden island a bridge can be for a team that
/// stands on it to have gone over by its bridge, in metres: a step, the
/// last plank of the way, hopped (a gap of a metre or so; two planks short
/// is two and a half).
const A_STEP: f32 = 1.6;

/// Plays a whole game of `seats` seats, the computer in every one, from
/// `seed`, to its end.
fn play(seed: u64, seats: usize) -> Story {
    let mut game = Game::new(seats, &vec![false; seats], seed);
    game.begin();
    let mut unforced = 0;
    let mut teammates = 0;
    let mut still = vec![(Vec3::ZERO, 0.0f32); seats];
    let mut stuck: f32 = 0.0;
    let mut stuck_by = (0, 0.0);
    let mut short = [None; 2];
    // How far short each one's team's bridge was as they last came onto
    // the golden island, and when.
    let mut came = vec![None::<(f32, f32)>; seats];
    let ways = [game.crew().way(0).expect("red's way"), game.crew().way(1).expect("blue's way")];
    let (mut short_ways, mut from_theirs) = (0, 0);
    while game.playing() {
        for event in game.step(FRAME) {
            match event {
                Event::Fell { by: None, .. } => unforced += 1,
                Event::Knocked { victim, by, .. } if team(victim) == team(by) => teammates += 1,
                // The first rung of a bridge off its team's way: a short way
                // of its own, and from whose bridge.
                Event::Plank(plank) if plank.index == 0 => {
                    let (at, side) = (Vec2::new(plank.x, plank.z), usize::from(team(usize::from(plank.seat))));
                    let way = ways[side];
                    if !(0..=way.planks()).any(|rung| way.rung(rung).distance(at) < 0.05) {
                        short_ways += 1;
                        let theirs = game.chains.iter().filter(|(seat, _)| usize::from(team(usize::from(*seat))) != side).any(|&(seat, chain)| {
                            game.nav().rungs(seat, chain).windows(2).enumerate().any(|(plank, pair)| !game.nav().broken(seat, chain, plank as u16) && over_plank(at, pair[0], pair[1]))
                        });
                        from_theirs += u32::from(theirs && game.nav().grid().height(at).is_none());
                    }
                }
                _ => {}
            }
        }
        for seat in 0..seats {
            let side = team(seat);
            // Over its land, on it or in the air over it a moment, knocked
            // or jumping: a body comes onto it from off it.
            let over = game.standing(seat) && game.nav().grid().region(game.at(seat).0.xz()) == Some(Region::Middle);
            match (over, came[seat]) {
                (true, None) => {
                    let short = (1.0 - f32::from(game.round().across[usize::from(side)]) / 100.0) * the_way_across(side);
                    came[seat] = Some((short, game.elapsed()));
                }
                (false, Some(_)) => came[seat] = None,
                _ => {}
            }
        }
        // The team stands there all together for the first time, its ring
        // beginning to fill: how short its bridge was as the last of them
        // came.
        for side in 0..2u8 {
            if short[usize::from(side)].is_none() && game.tally().first_hold[usize::from(side)].is_some() {
                let last = (0..seats).filter(|&seat| team(seat) == side).filter_map(|seat| came[seat]).max_by(|a, b| a.1.total_cmp(&b.1));
                short[usize::from(side)] = Some(last.map_or(0.0, |(short, _)| short));
            }
        }
        // Standing in one place a long while with somewhere to walk to:
        // held fast.
        for (seat, (at, since)) in still.iter_mut().enumerate() {
            let (now, _) = game.at(seat);
            let walking = game.crew().racer(seat).is_some_and(|racer| racer.walking());
            if now.distance(*at) > 0.3 || !game.standing(seat) || !walking {
                *at = now;
                *since = game.elapsed();
                continue;
            }
            if game.elapsed() - *since > stuck {
                stuck = game.elapsed() - *since;
                stuck_by = (seat, game.elapsed());
            }
        }
    }
    let (winner, finish, over) = game.result().expect("over");
    let racers = (0..seats).filter_map(|seat| game.crew().racer(seat));
    let stalled = (0..seats)
        .filter_map(|seat| game.crew().racer(seat).and_then(|racer| racer.stalls().1).map(|(here, next)| (seat, here, next)))
        .collect();
    Story {
        seed,
        seats,
        winner,
        finish,
        over,
        tally: game.tally(),
        stalls: racers.clone().map(|racer| racer.stalls().0).sum(),
        pinned: racers.clone().map(|racer| racer.pinned()).sum(),
        kinds: racers.map(|racer| racer.kind().name).collect(),
        laid: game.laid().to_vec(),
        unforced,
        teammates,
        stuck,
        stuck_by,
        stalled,
        short,
        short_ways,
        from_theirs,
    }
}

/// The games of `seeds` at `seats` seats, played a few at a time, each told
/// in a line.
fn told(seeds: std::ops::Range<u64>, seats: usize) -> Vec<Story> {
    let seeds: Vec<u64> = seeds.collect();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 8);
    let mut stories: Vec<Story> = std::thread::scope(|scope| {
        let chunks: Vec<_> = seeds
            .chunks(seeds.len().div_ceil(threads).max(1))
            .map(|chunk| scope.spawn(move || chunk.iter().map(|&seed| play(seed, seats)).collect::<Vec<Story>>()))
            .collect();
        chunks.into_iter().flat_map(|chunk| chunk.join().expect("a game")).collect()
    });
    stories.sort_by_key(|story| story.seed);
    let time = |at: Option<f32>| at.map_or(String::from("   -"), |at| format!("{at:>4.0}"));
    println!(
        "seed seats | winner how  over | joined red blue | held red blue | knocks off unforced | sticks given thrown lost blew planks | stalls pinned stuck (who, when) | laid | kinds"
    );
    for story in &stories {
        println!(
            "{:>4} {} | {:>6} {:>5} {:>4.0} | {} {} | {} {} | {:>3} {:>3} {:>3} | {:>3} {:>3} {:>3} {:>3} {:>3} | {:>2} {:>2} {:>4.1} ({}, {:.0}) | {} | {}",
            story.seed,
            story.seats,
            story.winner.map_or("draw", |team| if team == 0 { "red" } else { "blue" }),
            format!("{:?}", story.finish),
            story.over,
            time(story.tally.joined[0]),
            time(story.tally.joined[1]),
            time(story.tally.first_hold[0]),
            time(story.tally.first_hold[1]),
            story.tally.knocks,
            story.tally.knocked_off,
            story.unforced,
            story.tally.given,
            story.tally.thrown,
            story.tally.lost,
            story.tally.blown,
            story.tally.planks_blown,
            story.stalls,
            story.pinned,
            story.stuck,
            story.stuck_by.0,
            story.stuck_by.1,
            story.laid.iter().map(|laid| format!("{laid:.0}")).collect::<Vec<_>>().join(" "),
            story.kinds.join(" "),
        );
        // Where one got nowhere, with `RT_STALLS` set (Bridge Wars' own).
        if std::env::var_os("RT_STALLS").is_some() {
            for (seat, here, next) in &story.stalled {
                println!("       seat {seat} got nowhere at {here}, heading for {next}");
            }
        }
    }
    stories
}

/// Whether the team that won a game had its bridge onto the golden island,
/// or a step short of it ([`A_STEP`]), as the last of them came onto it
/// before it first stood there all together: whether it went over by its
/// bridge, rather than jumping the last of the void.
fn bridged_over(story: &Story) -> bool {
    story.winner.is_some_and(|won| story.short[usize::from(won)].is_some_and(|short| short <= A_STEP))
}

/// Whether the first team to stand on the golden island all together had
/// its bridge onto it, or a step short of it, as the last of them came
/// onto it: whether the race was to lay the way, not to jump it.
fn first_bridged_over(story: &Story) -> bool {
    let first = match story.tally.first_hold {
        [Some(red), Some(blue)] => usize::from(blue < red),
        [Some(_), None] => 0,
        [None, Some(_)] => 1,
        [None, None] => return false,
    };
    story.short[first].is_some_and(|short| short <= A_STEP)
}

/// Whether the team that won a game had its bridge all the way onto the
/// golden island by the time it first stood there all together, not even
/// the last step hopped.
fn bridged_all_the_way(story: &Story) -> bool {
    let Some(won) = story.winner.map(usize::from) else {
        return false;
    };
    match (story.tally.joined[won], story.tally.first_hold[won]) {
        (Some(joined), Some(held)) => joined <= held,
        _ => false,
    }
}

/// What a set of games came to, in a few lines.
fn summarise(stories: &[Story]) {
    let n = stories.len().max(1) as f32;
    let mut overs: Vec<f32> = stories.iter().map(|story| story.over).collect();
    overs.sort_by(f32::total_cmp);
    let held = stories.iter().filter(|story| story.finish == Finish::Held && story.winner.is_some()).count();
    let red = stories.iter().filter(|story| story.winner == Some(0)).count();
    let blue = stories.iter().filter(|story| story.winner == Some(1)).count();
    let thrown = stories.iter().filter(|story| story.tally.thrown > 0).count();
    let took = stories.iter().filter(|story| story.tally.planks_blown > 0).count();
    let mean = |pick: &dyn Fn(&Story) -> f32| stories.iter().map(pick).sum::<f32>() / n;
    println!(
        "{} games of {}: won by holding {:.0}%, red {red} blue {blue} drawn {}; over at {:.0} s at the median ({:.0} to {:.0}); a stick thrown in {:.0}% and taking planks in {:.0}%; knocks {:.1}, knocked off {:.1}, unforced falls {:.2}; stalls {:.2}, pinned {:.2}, stuck {:.1} s at the most",
        stories.len(),
        stories.first().map_or(0, |story| story.seats),
        100.0 * held as f32 / n,
        stories.len() - red - blue,
        overs[overs.len() / 2],
        overs[0],
        overs[overs.len() - 1],
        100.0 * thrown as f32 / n,
        100.0 * took as f32 / n,
        mean(&|story| story.tally.knocks as f32),
        mean(&|story| story.tally.knocked_off as f32),
        mean(&|story| story.unforced as f32),
        mean(&|story| story.stalls as f32),
        mean(&|story| story.pinned as f32),
        stories.iter().map(|story| story.stuck).fold(0.0, f32::max),
    );
    // Each team's first stand on the golden island, all of it, and who won
    // of the games one of them stood there first.
    let first = |team: usize| {
        let times: Vec<f32> = stories.iter().filter_map(|story| story.tally.first_hold[team]).collect();
        times.iter().sum::<f32>() / times.len().max(1) as f32
    };
    let ahead = stories
        .iter()
        .filter(|story| match story.tally.first_hold {
            [Some(red), Some(blue)] => (red < blue) == (story.winner == Some(0)),
            [Some(_), None] => story.winner == Some(0),
            [None, Some(_)] => story.winner == Some(1),
            [None, None] => false,
        })
        .count();
    println!(
        "first on the golden island all together at {:.0} s red, {:.0} s blue, on average; the first there won {:.0}%",
        first(0),
        first(1),
        100.0 * ahead as f32 / n
    );
    // How the first there and the winners got there, how much fighting
    // there was, and how many laid a short way of their own.
    let first = stories.iter().filter(|story| first_bridged_over(story)).count();
    let bridged = stories.iter().filter(|story| bridged_over(story)).count();
    let all_the_way = stories.iter().filter(|story| bridged_all_the_way(story)).count();
    let knocked = stories.iter().filter(|story| story.tally.knocks > 0).count();
    let off = stories.iter().filter(|story| story.tally.knocked_off > 0).count();
    // How far short the winners' bridges were, in the games they were
    // short by more than a step: by a jump's worth, or more, carried over.
    let shorts: Vec<f32> = stories
        .iter()
        .filter_map(|story| story.winner.and_then(|won| story.short[usize::from(won)]))
        .filter(|short| *short > A_STEP)
        .collect();
    let jumped = shorts.iter().filter(|short| **short <= 6.5).count();
    println!(
        "over by their bridge, or a step short: the first there all together {:.0}%, the winners {:.0}% (all the way {:.0}%), and the rest of the winners {jumped} a jump short, {} more; a knock in {:.0}% of games, someone knocked off in {:.0}%; short ways of their own {}, {} from the other team's bridge",
        100.0 * first as f32 / n,
        100.0 * bridged as f32 / n,
        100.0 * all_the_way as f32 / n,
        shorts.len() - jumped,
        100.0 * knocked as f32 / n,
        100.0 * off as f32 / n,
        stories.iter().map(|story| story.short_ways).sum::<u32>(),
        stories.iter().map(|story| story.from_theirs).sum::<u32>()
    );
    // How each kind's team did.
    let mut kinds: Vec<(&str, u32, u32)> = Vec::new();
    for story in stories {
        for (seat, &kind) in story.kinds.iter().enumerate() {
            let won = u32::from(story.winner == Some(team(seat)));
            match kinds.iter_mut().find(|(name, ..)| *name == kind) {
                Some(entry) => {
                    entry.1 += 1;
                    entry.2 += won;
                }
                None => kinds.push((kind, 1, won)),
            }
        }
    }
    let line: Vec<String> = kinds.iter().map(|(kind, played, won)| format!("{kind} {:.0}% of {played}", 100.0 * *won as f32 / *played as f32)).collect();
    println!("won, by kind: {}", line.join(", "));
}

/// Many games, for tuning by: `cargo test -p roundtown_net many_races --
/// --ignored --nocapture`, sixty games of two against two and sixty of one
/// against one, or as many as `RT_GAMES` says, from the seed 100, or from
/// the one `RT_FROM` says (one game over again: `RT_FROM=106 RT_GAMES=1`);
/// with `RT_STALLS=1`, where any of them got nowhere.
#[test]
#[ignore]
fn many_races() {
    let count: u64 = std::env::var("RT_GAMES").ok().and_then(|count| count.parse().ok()).unwrap_or(60);
    let from: u64 = std::env::var("RT_FROM").ok().and_then(|from| from.parse().ok()).unwrap_or(100);
    for seats in [4, 2] {
        let stories = told(from..from + count, seats);
        summarise(&stories);
    }
}

/// One game, told as it goes, for finding out what one of them did: `cargo
/// test -p roundtown_net watch_a_race -- --ignored --nocapture`, with
/// `RT_SEED` (100) and `RT_SEATS` (4); every `RT_EVERY` seconds (5) where
/// each of the computer's is and what it is about, and whatever happens.
#[test]
#[ignore]
fn watch_a_race() {
    let seed: u64 = std::env::var("RT_SEED").ok().and_then(|seed| seed.parse().ok()).unwrap_or(100);
    let seats: usize = std::env::var("RT_SEATS").ok().and_then(|seats| seats.parse().ok()).unwrap_or(4);
    let every: f32 = std::env::var("RT_EVERY").ok().and_then(|every| every.parse().ok()).unwrap_or(5.0);
    let mut game = Game::new(seats, &vec![false; seats], seed);
    game.begin();
    for side in 0..2 {
        let way = game.crew().way(side).expect("a way");
        println!("team {side}'s way: {} m, {} planks, from {} to {}", way.length(), way.planks(), way.from, way.to);
    }
    let mut told = 0.0;
    while game.playing() {
        for event in game.step(FRAME) {
            match event {
                Event::Plank(_) | Event::Granted { .. } => {}
                event => println!("{:>6.1} {event:?}", game.elapsed()),
            }
        }
        if game.elapsed() >= told {
            told += every;
            let round = game.round();
            println!("{:>6.1} across {:?} on gold {:?} hold {:?}", game.elapsed(), round.across, round.on_gold, round.hold);
            for side in 0..2 {
                let standing: String = game.crew().standing(side).iter().map(|stands| if *stands { '=' } else { '.' }).collect();
                println!("        team {side} {standing}");
            }
            for seat in 0..seats {
                let racer = game.crew().racer(seat).expect("the computer's");
                let (at, _) = game.at(seat);
                println!(
                    "        seat {seat} {:<8} {:?} at ({:.1}, {:.2}, {:.1}) metres {:.1} sticks {} walking {} laying {:?}",
                    racer.kind().name,
                    racer.goal(),
                    at.x,
                    at.y,
                    at.z,
                    racer.metres(),
                    game.kits()[seat].held(Utility::Dynamite),
                    racer.walking(),
                    racer.laying(),
                );
            }
        }
    }
    println!("{:?}", game.result());
}

/// How long a step of a game of the computer's alone takes, on average and
/// at the worst, over a few games: `cargo test -p roundtown_net
/// timing_of_a_race -- --ignored --nocapture` (offline it is the phone's
/// frame).
#[test]
#[ignore]
fn timing_of_a_race() {
    for seats in [4, 2] {
        let (mut total, mut worst, mut steps) = (std::time::Duration::ZERO, std::time::Duration::ZERO, 0u32);
        for seed in 0..20 {
            let mut game = Game::new(seats, &vec![false; seats], seed);
            game.begin();
            while game.playing() {
                let started = std::time::Instant::now();
                game.step(FRAME);
                let took = started.elapsed();
                total += took;
                worst = worst.max(took);
                steps += 1;
            }
        }
        println!("{seats} seats: {steps} steps, {:?} each on average, {:?} at the worst", total / steps, worst);
    }
}

#[test]
fn the_computer_plays_whole_races_close_varied_and_not_too_long() {
    // Two hundred of each: two against two, the game as it most often is,
    // and one against one.
    for seats in [4, 2] {
        let stories = told(0..200, seats);
        summarise(&stories);
        let n = stories.len() as f32;
        // A team wins by holding the golden island, all of it, nearly
        // always: the time limit is for when nobody gets there.
        let held = stories.iter().filter(|story| story.finish == Finish::Held && story.winner.is_some()).count();
        assert!(held as f32 >= 0.85 * n, "{seats} seats: won by holding {held} of {n}");
        // In two to four minutes, at the median.
        let mut overs: Vec<f32> = stories.iter().map(|story| story.over).collect();
        overs.sort_by(f32::total_cmp);
        let median = overs[overs.len() / 2];
        assert!((120.0..=240.0).contains(&median), "{seats} seats: over at {median} s at the median");
        // Each side wins about as often as the other.
        for side in 0..2 {
            let won = stories.iter().filter(|story| story.winner == Some(side)).count() as f32;
            assert!((0.4..=0.6).contains(&(won / n)), "{seats} seats: team {side} won {won} of {n}");
        }
        // A stick is thrown in more than half of them, and takes planks in
        // most of those.
        let thrown = stories.iter().filter(|story| story.tally.thrown > 0).count();
        let took = stories.iter().filter(|story| story.tally.planks_blown > 0).count();
        assert!(thrown * 2 > stories.len(), "{seats} seats: a stick thrown in {thrown} of {n}");
        assert!(took * 3 > thrown * 2, "{seats} seats: planks taken in {took} of the {thrown} with a stick thrown");
        // It is a race to lay the way: the first team there all together,
        // and the winners, went over by their bridge, or hopped the last
        // step of it, nearly always; the rest the void jumped, by the team
        // behind once the other stood there, or a knock that carried them
        // over. (Until 2026-11-18 the computer's players jumped the last
        // six metres in nine games of ten, a grant ahead of anyone who laid
        // the whole way.)
        let first = stories.iter().filter(|story| first_bridged_over(story)).count() as f32;
        let bridged = stories.iter().filter(|story| bridged_over(story)).count() as f32;
        assert!(first >= 0.8 * n, "{seats} seats: the first there went over by their bridge in {first} of {n}");
        assert!(bridged >= 0.75 * n, "{seats} seats: the winners went over by their bridge in {bridged} of {n}");
        // There is fighting: a knock in most games, someone knocked off in
        // many; more two against two, with twice as many to fight.
        let knocked = stories.iter().filter(|story| story.tally.knocks > 0).count() as f32;
        let off = stories.iter().filter(|story| story.tally.knocked_off > 0).count() as f32;
        let (most, many) = if seats == 4 { (0.8, 0.5) } else { (0.5, 0.2) };
        assert!(knocked >= most * n, "{seats} seats: a knock in {knocked} of {n}");
        assert!(off >= many * n, "{seats} seats: someone knocked off in {off} of {n}");
        // Nobody lays a plank of the other team's bridge for them, and a
        // short way of one's own off the team's way is now and then.
        let from_theirs: u32 = stories.iter().map(|story| story.from_theirs).sum();
        let short_ways: u32 = stories.iter().map(|story| story.short_ways).sum();
        assert_eq!(from_theirs, 0, "{seats} seats: a short way laid from the other team's bridge");
        assert!(short_ways as f32 <= 0.25 * n, "{seats} seats: {short_ways} short ways of their own in {n} games");
        // Nobody held fast: nobody gets nowhere on a way for long, nor
        // stands with a way before it and does not walk it.
        for story in &stories {
            let seed = story.seed;
            assert_eq!(story.pinned, 0, "{seats} seats, seed {seed}: one of them was held fast");
            assert!(story.stalls <= 1, "{seats} seats, seed {seed}: got nowhere {} times", story.stalls);
            assert!(story.stuck < 8.0, "{seats} seats, seed {seed}: seat {} stood {} s with a way to walk, until {}", story.stuck_by.0, story.stuck, story.stuck_by.1);
            // Never a knock on a teammate.
            assert_eq!(story.teammates, 0, "{seats} seats, seed {seed}");
            // Everyone builds: nobody's metres lie unlaid all game.
            for (seat, laid) in story.laid.iter().enumerate() {
                assert!(*laid > 5.0, "{seats} seats, seed {seed}: seat {seat} laid {laid} m");
            }
        }
        // Falls nobody of the other team had a hand in are rare: a jump
        // over a hole a blast has just widened, now and then.
        let unforced = stories.iter().map(|story| story.unforced).sum::<u32>() as f32;
        assert!(unforced <= 0.15 * n, "{seats} seats: {unforced} falls of nobody's doing in {n} games");
        // Every kind plays, and wins sometimes.
        for kind in brain::KINDS {
            let played = stories.iter().flat_map(|story| story.kinds.iter().enumerate().map(move |(seat, name)| (story, seat, *name))).filter(|(_, _, name)| *name == kind.name);
            let (games, won) = played.fold((0, 0), |(games, won), (story, seat, _)| (games + 1, won + u32::from(story.winner == Some(team(seat)))));
            assert!(games > 0 && won > 0, "{seats} seats: the {} played {games} and won {won}", kind.name);
        }
    }
    // The same dice, the same game.
    let (one, other) = (play(7, 4), play(7, 4));
    assert_eq!((one.over, one.winner, one.tally, &one.laid), (other.over, other.winner, other.tally, &other.laid));
}

// -------------------------------------------------------- the rules alone

/// A game of `seats` seats of people only, begun: the computer nowhere, so
/// that each rule can be tried alone.
fn people(seats: usize, seed: u64) -> Game {
    let mut game = Game::new(seats, &vec![true; seats], seed);
    game.begin();
    game
}

/// Steps `game` on `secs` seconds, and everything that happened.
fn run(game: &mut Game, secs: f32) -> Vec<Event> {
    let mut events = Vec::new();
    let mut gone = 0.0;
    while gone < secs - 1e-4 {
        events.extend(game.step(FRAME));
        gone += FRAME;
    }
    events
}

/// Has `count` grants come now, ahead of the clock, each what it would give
/// as things stand, to people alone (none of the computer's, and no
/// sticks): for laying a long way as a person in a test without playing
/// out the time the metres for it take, the game taking no rung past what
/// someone has been given ([`Game::most_laid`]). The next grant by the
/// clock comes once the clock has caught up.
fn grants_now(game: &mut Game, count: u16) {
    for _ in 0..count {
        game.grants += 1;
        let share = [game.share(0), game.share(1)];
        game.shares.push(share);
    }
}

/// Lays `planks` rungs' worth of `seat`'s team's way across, as the
/// person in `seat` would, walking out along it: rungs from just inside its
/// island's shore, each a plank on, as if they stood a little behind each.
fn lay_across(game: &mut Game, seat: usize, chain: u16, planks: usize) -> Span {
    let span = game.nav().grid().span(Region::Team(team(seat)), Region::Middle, 0.0).expect("a way across");
    for index in 0..=planks.min(span.planks()) {
        let at = span.rung(index);
        game.sync(seat, Vec3::new(at.x, LAND_TOP, at.y), span.heading());
        assert!(game.rung(seat, chain, index as u16, at, span.heading_at(index)), "rung {index}");
    }
    span
}

/// Stands the person in `seat` on the golden island, `out` metres from its
/// middle toward their own island.
fn on_gold(game: &mut Game, seat: usize, out: f32) {
    let toward = (ISLANDS[usize::from(team(seat))] - GOLDEN).normalize() * out;
    game.sync(seat, Vec3::new(toward.x, LAND_TOP, toward.y), 0.0);
}

#[test]
fn everyone_begins_with_ten_metres_and_more_comes_every_sixteen_seconds_shared_by_a_team() {
    assert_eq!(grant_metres(2), TEAM_GRANT);
    assert_eq!(grant_metres(4), TEAM_GRANT / 2.0);
    assert_eq!((grants_by(0.0), grants_by(GRANT_EVERY - 0.01), grants_by(GRANT_EVERY)), (0, 0, 1));
    assert_eq!(grants_by(PLAY_SECS), (PLAY_SECS / GRANT_EVERY) as u16);
    // The computer's metres, counted by the game: ten, then a share of the
    // team's grant at each.
    let mut game = Game::new(4, &[false; 4], 3);
    assert_eq!(game.metres(1), START_METRES);
    game.begin();
    let mut granted = Vec::new();
    for _ in 0..(3.5 * GRANT_EVERY / FRAME) as usize {
        for event in game.step(FRAME) {
            if let Event::Granted { grant, .. } = event {
                granted.push((grant, game.elapsed()));
            }
        }
    }
    assert_eq!(granted.iter().map(|(grant, _)| *grant).collect::<Vec<_>>(), [1, 2, 3]);
    for (grant, when) in granted {
        assert!((when - f32::from(grant) * GRANT_EVERY).abs() < 2.0 * FRAME, "grant {grant} at {when}");
    }
    assert_eq!(game.round().grants, 3);
    // Laid and given, by seat: what each has laid and has left comes to
    // what each was given.
    for seat in 0..4 {
        let given = START_METRES + 3.0 * grant_metres(4);
        assert!((game.laid()[seat] + game.metres(seat) - given).abs() < 0.01, "seat {seat}: laid {} and left {}", game.laid()[seat], game.metres(seat));
    }
    // A person's are their device's: what it says, believed only as far as
    // it could be so, and given back to it with every grant since if it
    // comes back.
    let mut game = people(2, 3);
    run(&mut game, 2.0 * GRANT_EVERY + 1.0);
    assert_eq!(game.round().grants, 2);
    assert_eq!(game.kept(0), (START_METRES + 2.0 * grant_metres(2), 2));
    game.said(0, 4.5, 1);
    assert_eq!(game.kept(0), (4.5 + grant_metres(2), 2));
    game.said(0, 999.0, 2);
    assert_eq!(game.kept(0).0, START_METRES + 2.0 * grant_metres(2), "more than was ever given");
    game.said(0, 3.0, 9);
    assert_eq!(game.kept(0).0, START_METRES + 2.0 * grant_metres(2), "a grant that has not come");
}

#[test]
fn sticks_come_with_the_grants_at_about_one_in_five_and_nobody_holds_more_than_two() {
    let (mut given, mut room) = (0, 0);
    for seed in 0..40 {
        let mut game = people(4, seed);
        for _ in 0..20 {
            let held: Vec<u8> = game.kits().iter().map(|kit| kit.held(Utility::Dynamite)).collect();
            // On to the next grant.
            let sticks: Vec<usize> = (0..)
                .take((GRANT_EVERY / FRAME) as usize + 10)
                .find_map(|_| {
                    game.step(FRAME).into_iter().find_map(|event| match event {
                        Event::Granted { sticks, .. } => Some(sticks),
                        _ => None,
                    })
                })
                .expect("a grant");
            room += held.iter().filter(|&&held| held < DYNAMITE_MOST).count();
            given += sticks.len();
            for seat in 0..4 {
                let now = game.kits()[seat].held(Utility::Dynamite);
                assert!(now <= DYNAMITE_MOST, "{now} sticks");
                assert_eq!(now, held[seat] + u8::from(sticks.contains(&seat)), "seat {seat}");
            }
        }
    }
    let share = given as f32 / room as f32;
    assert!((share - DYNAMITE_ODDS).abs() < 0.04, "{given} sticks of {room} chances: {share}");
}

#[test]
fn a_stick_takes_the_planks_its_blast_reaches_and_no_more_and_knocks_only_the_other_team() {
    let mut game = people(4, 1);
    // Red's way laid twenty planks out from its island.
    grants_now(&mut game, 4);
    let span = lay_across(&mut game, 0, 0, 20);
    let laid = game.nav().planks().count();
    assert_eq!(laid, 20);
    // Blue (seat 1) with a stick, on red's bridge a few planks out, throws
    // it eight metres along it; red's other (seat 2) stands where it comes
    // down, and blue's other (seat 3) too.
    game.kits[1] = with_sticks(game.kits[1], 1);
    let from = span.rung(4);
    let target = span.rung(10);
    game.sync(1, Vec3::new(from.x, LAND_TOP, from.y), 0.0);
    game.sync(2, Vec3::new(target.x, LAND_TOP, target.y), 0.0);
    game.sync(3, Vec3::new(target.x + 0.5, LAND_TOP, target.y), 0.0);
    // A throw needs a stick on the button; the person's, on their feet.
    assert!(!game.throw(0, 0.0, 5.0), "red has none");
    let off = target - from;
    assert!(game.throw(1, (-off.x).atan2(-off.y), off.length()));
    assert_eq!(game.kits()[1].held(Utility::Dynamite), 0);
    assert_eq!(game.kits()[1].slot_of(Utility::Dynamite), None, "the empty button is put away");
    assert!(!game.throw(1, 0.0, 5.0), "and has none left");
    let events = run(&mut game, 3.0);
    let (at, planks) = events
        .iter()
        .find_map(|event| match event {
            Event::Blew { at, planks, by: 1, .. } => Some((*at, planks.clone())),
            _ => None,
        })
        .expect("the stick blew");
    assert!(at.xz().distance(target) < 0.2, "blew at {at}");
    // Every plank whose middle it reached is gone, and no other: none
    // falls after them, however many stand beyond.
    assert!(!planks.is_empty());
    let rungs = game.nav().rungs(0, 0).to_vec();
    for plank in 0..20 {
        let middle = (rungs[plank] + rungs[plank + 1]) * 0.5;
        let reached = middle.distance(at.xz()) <= DYNAMITE_RADIUS;
        assert_eq!(planks.contains(&(0, 0, plank as u16)), reached, "plank {plank}");
        assert_eq!(game.nav().broken(0, 0, plank as u16), reached, "plank {plank}");
    }
    assert_eq!(game.nav().planks().count(), 20 - planks.len());
    // Red's other, in the blast, was knocked from it, and the thrower's
    // teammate beside them was not; nobody is hurt, there being no health.
    let knocked: Vec<usize> = events
        .iter()
        .filter_map(|event| match event {
            Event::Knocked { victim, by: 1, running: true, .. } => Some(*victim),
            _ => None,
        })
        .collect();
    assert_eq!(knocked, [2]);
    // Fallen through the hole, red's other is blue's stick's doing.
    game.fell(2);
    assert!(run(&mut game, FRAME).contains(&Event::Fell { seat: 2, by: Some(1) }));
    assert_eq!(game.tally().knocked_off, 1);
    // And nothing more of the bridge goes after it.
    let more = run(&mut game, 10.0);
    assert!(!more.iter().any(|event| matches!(event, Event::Blew { .. })));
    assert_eq!(game.nav().planks().count(), 20 - planks.len(), "nothing more falls after");
}

#[test]
fn a_stick_that_comes_down_where_there_is_nothing_is_lost_unblown() {
    let mut game = people(2, 1);
    game.kits[0] = with_sticks(game.kits[0], 1);
    // Out over the void off red's island, toward nothing.
    let edge = ISLANDS[0] + Vec2::new(-9.0, 0.0);
    game.sync(0, Vec3::new(edge.x, LAND_TOP, edge.y), 0.0);
    assert!(game.throw(0, std::f32::consts::FRAC_PI_2, DYNAMITE_FURTHEST));
    let events = run(&mut game, 3.0);
    assert!(events.iter().any(|event| matches!(event, Event::Lost { stick: 0, by: 0 })));
    assert!(!events.iter().any(|event| matches!(event, Event::Blew { .. })));
}

#[test]
fn a_punch_knocks_and_does_nothing_else_and_never_lands_on_a_teammate() {
    let mut game = people(4, 1);
    let red = ISLANDS[0];
    for (seat, x) in [(0, 0.0), (1, 1.2), (2, 2.0), (3, 9.0)] {
        game.sync(seat, Vec3::new(red.x + x, LAND_TOP, red.y), 0.0);
    }
    // Not a teammate, however near.
    assert!(!game.punch(0, 2, true));
    // Someone of the other team within reach: knocked, from the hitter, on
    // the run as they said.
    assert!(game.punch(0, 1, true));
    let events = game.step(FRAME);
    assert!(events.contains(&Event::Knocked {
        victim: 1,
        by: 0,
        from: Vec3::new(red.x, LAND_TOP, red.y),
        running: true
    }));
    // Not again while the last still smarts, and then again.
    assert!(!game.punch(2, 1, false));
    run(&mut game, HURT_TIME);
    assert!(game.punch(2, 1, false));
    // Not from across the map, and not by or on someone who has fallen.
    assert!(!game.punch(0, 3, false), "too far");
    game.fell(1);
    run(&mut game, 0.1);
    assert!(!game.punch(0, 1, false) && !game.punch(1, 0, false), "fallen");
    // A computer's player punched is knocked by the game itself, up and
    // away from the hitter.
    let mut game = Game::new(2, &[true, false], 5);
    game.begin();
    let (bot, _) = game.at(1);
    game.sync(0, bot - Vec3::X, 0.0);
    assert!(game.punch(0, 1, true));
    let body = *game.crew().body(1).expect("the computer's");
    assert!(body.lift > 0.0 && body.shove.x > 0.0, "{body:?}");
}

#[test]
fn whoever_falls_is_back_on_their_own_island_s_middle_a_moment_later_with_what_they_had() {
    let mut game = people(4, 2);
    game.kits[3] = with_sticks(game.kits[3], 2);
    game.sync(3, Vec3::new(-20.0, -40.0, 0.0), 0.0);
    game.fell(3);
    assert!(!game.standing(3));
    let events = run(&mut game, RESPAWN - 0.1);
    assert!(events.contains(&Event::Fell { seat: 3, by: None }));
    assert!(!events.iter().any(|event| matches!(event, Event::Back { .. })));
    let events = run(&mut game, 0.2);
    let (home, yaw) = home(3);
    assert!(events.contains(&Event::Back { seat: 3, at: home, yaw }));
    assert_eq!(home.xz(), ISLANDS[1], "blue's island's middle");
    assert!(game.standing(3));
    assert_eq!(game.kits()[3].held(Utility::Dynamite), 2, "the sticks are kept");
    // A fall someone of the other team knocked them into is theirs.
    game.sync(0, home + Vec3::X, 0.0);
    assert!(game.punch(0, 3, false));
    game.fell(3);
    assert!(run(&mut game, FRAME).contains(&Event::Fell { seat: 3, by: Some(0) }));
    // The computer's fall by themselves, out of the map.
    let mut game = Game::new(2, &[true, false], 2);
    game.begin();
    game.sync(0, Vec3::new(ISLANDS[1].x - 1.0, LAND_TOP, ISLANDS[1].y), 0.0);
    // Knocked off the far edge of blue's island, again and again, it falls.
    let mut fell = false;
    for _ in 0..400 {
        let (bot, _) = game.at(1);
        let off = bot.xz() - ISLANDS[1];
        game.sync(0, bot - Vec3::new(off.x, 0.0, off.y).normalize_or(Vec3::X) * 1.2, 0.0);
        game.punch(0, 1, true);
        if run(&mut game, 0.1).iter().any(|event| matches!(event, Event::Fell { seat: 1, by: Some(0) })) {
            fell = true;
            break;
        }
    }
    assert!(fell, "the computer's never fell");
}

#[test]
fn a_team_wins_with_every_one_of_it_on_the_golden_island_together_for_three_seconds() {
    let mut game = people(4, 4);
    // One of red on it: nothing yet.
    on_gold(&mut game, 0, 3.0);
    run(&mut game, HOLD + 1.0);
    assert_eq!(game.round().on_gold, [1, 0]);
    assert_eq!(game.round().hold, [0, 0]);
    // Both: the ring fills.
    on_gold(&mut game, 2, 1.0);
    let events = run(&mut game, HOLD * 0.5);
    assert!(events.contains(&Event::Holding { team: 0, holding: true }));
    assert_eq!(game.round().on_gold, [2, 0]);
    assert!((4..=6).contains(&game.round().hold[0]), "{:?}", game.round().hold);
    // Knocked off it, or in the air over it, one of them: empty.
    game.sync(2, Vec3::new(0.0, LAND_TOP + 1.5, 0.0), 0.0);
    let events = run(&mut game, FRAME);
    assert!(events.contains(&Event::Holding { team: 0, holding: false }));
    assert_eq!(game.round().hold, [0, 0]);
    // On it again, both of them, for the whole of it: won.
    on_gold(&mut game, 2, 1.0);
    let events = run(&mut game, HOLD + 0.2);
    assert!(events.contains(&Event::Over { winner: Some(0), finish: Finish::Held }));
    assert_eq!(game.round().winner, Some(0));
    assert!(game.round().over() && !game.playing());
    assert_eq!(game.round().finish, Some(Finish::Held));
    let (places, scores, total) = game.outcome().expect("over");
    assert_eq!(places, [0, 1, 0, 1]);
    assert_eq!(scores, [0; 4]);
    assert_eq!(total, None);
    // Nothing more happens once it is over.
    assert!(run(&mut game, 5.0).is_empty());
    assert!(!game.rung(0, 0, 0, ISLANDS[0], 0.0));
    // A plank under someone over the golden island's land is the island:
    // standing on the end of a bridge over its rim counts; over the void
    // beside it does not.
    let mut game = people(2, 4);
    game.sync(0, Vec3::new(-ISLAND_RADIUS + 0.3, LAND_TOP, 0.0), 0.0);
    assert!(game.on_gold(0));
    game.sync(0, Vec3::new(-ISLAND_RADIUS - 1.0, LAND_TOP, 0.0), 0.0);
    assert!(!game.on_gold(0));
    // Both teams all there at once: both rings fill, and filled together,
    // a draw.
    let mut game = people(2, 4);
    on_gold(&mut game, 0, 2.0);
    on_gold(&mut game, 1, 2.0);
    let events = run(&mut game, HOLD + 0.2);
    assert!(events.contains(&Event::Over { winner: None, finish: Finish::Held }));
    assert_eq!(game.outcome().expect("over").0, [0, 0]);
    // Which the round says, for every device to say it so.
    assert_eq!((game.round().winner, game.round().finish), (None, Some(Finish::Held)));
}

/// Stands the computer's player in `seat` `at`, there and then.
fn put(game: &mut Game, seat: usize, at: Vec3) {
    let now = game.elapsed();
    game.crew.stand(seat, at, 0.0, now);
    game.at[seat] = at;
}

#[test]
fn the_ring_empties_when_one_of_a_team_is_knocked_by_a_punch_or_by_a_blast() {
    // Blue the computer's, both stood on the golden island, holding it;
    // red people, one of them there too, the other at home, so that red
    // does not.
    let mut game = Game::new(4, &[true, false, true, false], 12);
    game.begin();
    put(&mut game, 1, Vec3::new(-2.0, LAND_TOP, 1.0));
    put(&mut game, 3, Vec3::new(2.0, LAND_TOP, -1.0));
    let events = run(&mut game, 1.2);
    assert!(events.contains(&Event::Holding { team: 1, holding: true }), "{:?}", game.round());
    assert!(game.round().hold[1] > 0);
    // A punch on one of them knocks it up off its feet: empty.
    let (bot, _) = game.at(1);
    game.sync(0, bot - Vec3::X, 0.0);
    assert!(game.punch(0, 1, true));
    let events = run(&mut game, FRAME);
    assert!(events.contains(&Event::Holding { team: 1, holding: false }));
    assert_eq!(game.round().hold[1], 0);
    // Down on the island again, and holding again.
    let events = run(&mut game, 1.5);
    assert!(events.contains(&Event::Holding { team: 1, holding: true }), "{:?}", game.round());
    // A stick of red's blowing at the other's feet: empty again, both of
    // them it reaches knocked, neither hurt.
    let (feet, _) = game.at(3);
    let mut lob = Lob::thrown(feet + Vec3::X * 3.0, std::f32::consts::FRAC_PI_2, 3.0, feet.y);
    lob.to = feet;
    lob.age = lob.secs() + crate::bridge_wars::DYNAMITE_FUSE - 0.01;
    game.sticks.push(Lobbed { id: 7, lob, by: 0, lies: true });
    let events = run(&mut game, FRAME);
    assert!(events.iter().any(|event| matches!(event, Event::Blew { stick: 7, .. })));
    assert!(events.contains(&Event::Knocked { victim: 3, by: 0, from: feet, running: true }));
    assert!(events.contains(&Event::Holding { team: 1, holding: false }));
    assert_eq!(game.round().hold[1], 0);
    assert!(game.playing());
}

#[test]
fn whoever_is_left_of_a_team_is_given_the_whole_of_its_grant() {
    // Two against two of people: blue's second leaves for good after the
    // first grant.
    let mut game = people(4, 13);
    run(&mut game, GRANT_EVERY + 0.5);
    assert_eq!((game.round().grants, game.round().share), (1, [30, 30]));
    game.leave(3);
    assert_eq!(game.round().share, [30, 30], "what the last grant gave");
    run(&mut game, GRANT_EVERY);
    let round = game.round();
    assert_eq!((round.grants, round.share), (2, [30, 60]));
    assert_eq!((round.share_of(0), round.share_of(1)), (grant_metres(4), TEAM_GRANT));
    // Back on a device that has forgotten, each is given every grant as it
    // came, and is believed no further than that.
    assert_eq!(game.kept(1), (START_METRES + 3.0 + 6.0, 2));
    assert_eq!(game.kept(0), (START_METRES + 3.0 + 3.0, 2));
    game.said(1, 999.0, 2);
    assert_eq!(game.kept(1).0, START_METRES + 9.0);
    game.said(1, 2.0, 1);
    assert_eq!(game.kept(1), (2.0 + 6.0, 2));
    // The computer's too: a person and the computer's against two of the
    // computer's, the person gone before the first grant.
    let mut game = Game::new(4, &[true, false, false, false], 13);
    game.begin();
    game.leave(0);
    run(&mut game, 3.0 * GRANT_EVERY + 0.5);
    let given = |game: &Game, seat: usize| game.laid()[seat] + game.metres(seat);
    assert!((given(&game, 2) - (START_METRES + 3.0 * TEAM_GRANT)).abs() < 0.01, "red's: {}", given(&game, 2));
    for seat in [1, 3] {
        assert!((given(&game, seat) - (START_METRES + 3.0 * grant_metres(4))).abs() < 0.01, "blue's {seat}: {}", given(&game, seat));
    }
}

#[test]
fn a_bridge_laid_out_wide_of_the_islands_is_ground_the_game_knows() {
    // Red's laid out west from its island, away from everything, fifteen
    // metres over the void: a stick of blue's thrown onto its end lies on
    // it, and blows, and takes planks.
    let mut game = people(2, 14);
    let heading = std::f32::consts::FRAC_PI_2;
    let shore = ISLANDS[0] - Vec2::X * (ISLAND_RADIUS - 0.5);
    let out = ISLANDS[0] - Vec2::X * (ISLAND_RADIUS + 15.0);
    let planks = 11;
    for index in 0..=planks {
        let at = shore.lerp(out, index as f32 / planks as f32);
        game.sync(0, Vec3::new(at.x, LAND_TOP, at.y), heading);
        assert!(game.rung(0, 0, index as u16, at, heading), "rung {index}");
    }
    let end = out + Vec2::X * 1.0;
    assert!(game.nav().ground(end).is_some() && game.nav().walkable_at(end), "{end} is no ground");
    game.kits[1] = with_sticks(game.kits[1], 1);
    let from = out + Vec2::X * 7.0;
    game.sync(1, Vec3::new(from.x, LAND_TOP, from.y), heading);
    let off = end - from;
    assert!(game.throw(1, (-off.x).atan2(-off.y), off.length()));
    let events = run(&mut game, 3.0);
    assert!(!events.iter().any(|event| matches!(event, Event::Lost { .. })), "fell through it");
    assert!(events.iter().any(|event| matches!(event, Event::Blew { planks, .. } if !planks.is_empty())));
}

#[test]
fn one_of_the_computer_s_left_on_the_other_team_s_bridge_lays_none_of_it_and_jumps_from_it() {
    // Red a person, whose bridge is laid straight toward the golden island
    // to some nine metres short of it; blue's player the computer's, stood
    // on red's bridge near its end with metres to lay, and cut off from
    // its own team's way, which nobody has laid.
    let mut game = Game::new(2, &[true, false], 15);
    game.begin();
    grants_now(&mut game, 9);
    let span = game.nav().grid().span(Region::Team(0), Region::Middle, 0.0).expect("red's way");
    let short_of = |rung: usize| span.rung(rung).distance(GOLDEN) - ISLAND_RADIUS;
    let last = (0..=span.planks()).take_while(|&rung| short_of(rung) >= 9.0).last().expect("rungs");
    for index in 0..=last {
        let at = span.rung(index);
        game.sync(0, Vec3::new(at.x, LAND_TOP, at.y), span.heading());
        assert!(game.rung(0, 0, index as u16, at, span.heading_at(index)));
    }
    game.sync(0, Vec3::new(ISLANDS[0].x, LAND_TOP, ISLANDS[0].y), 0.0);
    let stood = (span.rung(last - 3) + span.rung(last - 2)) * 0.5;
    put(&mut game, 1, Vec3::new(stood.x, crate::nav::PLANK_TOP, stood.y));
    let now = game.elapsed();
    game.crew.grant(1, 20.0, now);
    // It lays nothing: not on from the end of red's bridge for them, and
    // its own way it cannot get to. It stands on red's bridge, in their
    // way.
    let laid = |events: &[Event]| events.iter().any(|event| matches!(event, Event::Plank(plank) if plank.seat == 1));
    let events = run(&mut game, 20.0);
    assert!(!laid(&events), "it laid from red's bridge");
    assert!(game.standing(1) && game.nav().on_plank(game.at(1).0.xz()), "at {}", game.at(1).0);
    // Red laid on to a jump short of the golden island: it jumps for it
    // from there, and stands on it.
    for index in last + 1..=span.planks() - 3 {
        let at = span.rung(index);
        game.sync(0, Vec3::new(at.x, LAND_TOP, at.y), span.heading());
        assert!(game.rung(0, 0, index as u16, at, span.heading_at(index)));
    }
    game.sync(0, Vec3::new(ISLANDS[0].x, LAND_TOP, ISLANDS[0].y), 0.0);
    let events = run(&mut game, 15.0);
    assert!(!laid(&events), "it laid from red's bridge");
    assert!(game.nav().grid().region(game.at(1).0.xz()) == Some(Region::Middle), "at {}", game.at(1).0);
}

#[test]
fn at_the_time_limit_more_on_the_golden_island_wins_or_the_bridge_further_across_or_it_is_a_draw() {
    // More of one team on the island: one of blue, and the other at home,
    // so that blue does not hold it.
    let mut game = people(4, 5);
    on_gold(&mut game, 1, 2.0);
    let events = run(&mut game, PLAY_SECS + 0.5);
    assert!(events.contains(&Event::Over { winner: Some(1), finish: Finish::Time }), "{:?}", game.result());
    assert_eq!(game.round().left, 0);
    // As many: the bridge further across.
    let mut game = people(2, 5);
    grants_now(&mut game, 1);
    lay_across(&mut game, 1, 0, 12);
    lay_across(&mut game, 0, 0, 8);
    let round = game.step(FRAME);
    drop(round);
    let across = game.round().across;
    assert!(across[1] > across[0] && across[0] > 0, "{across:?}");
    for seat in 0..2 {
        game.sync(seat, Vec3::new(ISLANDS[seat].x, LAND_TOP, ISLANDS[seat].y), 0.0);
    }
    let events = run(&mut game, PLAY_SECS);
    assert!(events.contains(&Event::Over { winner: Some(1), finish: Finish::Time }), "{:?}", game.result());
    // Neither: a draw, everyone first.
    let mut game = people(2, 5);
    for seat in 0..2 {
        game.sync(seat, Vec3::new(ISLANDS[seat].x, LAND_TOP, ISLANDS[seat].y), 0.0);
    }
    run(&mut game, PLAY_SECS + 0.5);
    assert_eq!(game.result().map(|(winner, finish, _)| (winner, finish)), Some((None, Finish::Time)));
    assert_eq!(game.outcome().expect("over").0, [0, 0]);
    // A bridge joined to the golden island is the whole way across.
    let mut game = people(2, 5);
    grants_now(&mut game, 8);
    let span = lay_across(&mut game, 0, 0, usize::MAX);
    game.step(FRAME);
    assert_eq!(game.round().across[0], 100, "{} planks", span.planks());
    assert!(game.tally().joined[0].is_some());
}

#[test]
fn a_rung_is_taken_from_someone_it_could_be_from_and_what_they_lay_is_their_score() {
    let mut game = people(2, 6);
    let span = lay_across(&mut game, 0, 3, 6);
    let events = game.step(FRAME);
    drop(events);
    // Each told, the computer's as a person's.
    let length: f32 = (0..6).map(|plank| span.rung(plank).distance(span.rung(plank + 1))).sum();
    assert!((game.laid()[0] - length).abs() < 0.01, "{} laid of {length}", game.laid()[0]);
    // Not from across the map, not from someone not playing, not a rung
    // that is no number.
    let far = span.rung(30);
    assert!(!game.rung(0, 3, 7, far, 0.0), "laid {} m from where they stand", far.distance(span.rung(6)));
    assert!(!game.rung(0, 3, 7, Vec2::new(f32::NAN, 0.0), 0.0));
    assert!(!game.rung(5, 0, 0, span.rung(6), 0.0), "nobody's seat");
    // One it has had already is nothing new, and not told again.
    game.step(FRAME);
    assert!(game.rung(0, 3, 6, span.rung(6), 0.0));
    assert!(game.step(FRAME).is_empty());
    assert!((game.laid()[0] - length).abs() < 0.01);
    // A rung out of order waits for the one before it, and is counted
    // once that comes.
    assert!(game.rung(0, 3, 8, span.rung(8), 0.0));
    assert!((game.laid()[0] - length).abs() < 0.01);
    assert!(game.rung(0, 3, 7, span.rung(7), 0.0));
    let both = span.rung(6).distance(span.rung(7)) + span.rung(7).distance(span.rung(8));
    assert!((game.laid()[0] - length - both).abs() < 0.01);
    let events = run(&mut game, 1.0);
    let _ = events;
    // Over, each one's score is what they laid, to the whole metre.
    game.sync(0, Vec3::new(0.0, LAND_TOP, 0.0), 0.0);
    run(&mut game, HOLD + 0.5);
    let (places, scores, _) = game.outcome().expect("red held it");
    assert_eq!(places, [0, 1]);
    assert_eq!(scores, [(length + both).round() as u32, 0]);
}

#[test]
fn every_event_a_rung_and_a_grant_and_a_throw_say_is_for_the_room_as_bridge_wars_sends_it() {
    let mut game = people(2, 1);
    game.kits[0] = with_sticks(game.kits[0], 1);
    let span = lay_across(&mut game, 0, 2, 3);
    let events = game.step(FRAME);
    let planks: Vec<&Plank> = events
        .iter()
        .filter_map(|event| match event {
            Event::Plank(plank) => Some(plank),
            _ => None,
        })
        .collect();
    assert_eq!(planks.len(), 4);
    for (index, plank) in planks.iter().enumerate() {
        assert_eq!((plank.seat, plank.chain, plank.index), (0, 2, index as u16));
        assert_eq!(Vec2::new(plank.x, plank.z), span.rung(index));
    }
    let feet = span.rung(1);
    game.sync(0, Vec3::new(feet.x, LAND_TOP, feet.y), 0.0);
    assert!(game.throw(0, span.heading(), 4.0));
    let events = game.step(FRAME);
    let shown = events
        .iter()
        .find_map(|event| match event {
            Event::Thrown { stick: 0, shown } => Some(*shown),
            _ => None,
        })
        .expect("thrown");
    assert_eq!(shown.seat, 0);
    let lob = Lob::thrown(Vec3::new(feet.x, LAND_TOP, feet.y), span.heading(), 4.0, PLANK_TOP_AT_LIFT);
    assert!(Vec3::new(shown.to_x, shown.to_y, shown.to_z).distance(lob.to) < 0.05, "{shown:?} {lob:?}");
}

/// A plank's top on Bridge Race's map, lifted: where a stick thrown onto a
/// bridge lies.
const PLANK_TOP_AT_LIFT: f32 = crate::nav::PLANK_TOP;

#[test]
fn a_team_with_nobody_left_in_the_game_has_lost_it() {
    let mut game = people(4, 8);
    assert_eq!(game.round().players, [2, 2]);
    game.leave(1);
    run(&mut game, 1.0);
    assert!(game.playing(), "blue has another");
    // The round says how many of each are still in it: the plate's "1/1
    // on the island" for blue now.
    assert_eq!(game.round().players, [2, 1]);
    // Blue's other left too: red wins it, which the round says.
    game.leave(3);
    let events = run(&mut game, FRAME);
    assert!(events.contains(&Event::Over { winner: Some(0), finish: Finish::Left }));
    assert_eq!((game.round().winner, game.round().finish, game.round().players), (Some(0), Some(Finish::Left), [2, 0]));
    // Gone, they count for nothing on the golden island: the one of red
    // left there holds it for red.
    let mut game = people(4, 8);
    game.leave(2);
    on_gold(&mut game, 0, 2.0);
    let events = run(&mut game, HOLD + 0.2);
    assert!(events.contains(&Event::Over { winner: Some(0), finish: Finish::Held }));
}

#[test]
fn the_round_goes_small_and_whole() {
    let mut game = Game::new(4, &[false; 4], 9);
    game.begin();
    let round = game.round();
    assert_eq!(round.left, PLAY_SECS as u16);
    assert_eq!((round.grants, round.hold, round.on_gold, round.winner, round.finish, round.across), (0, [0, 0], [0, 0], None, None, [0, 0]));
    assert!(!round.over());
    assert_eq!(round.players, [2, 2]);
    assert_eq!(round.share, [30, 30], "3 m each, two against two");
    assert_eq!(round.share_of(1), grant_metres(4));
    // As long as it ever is.
    let longest = RaceRound {
        left: u16::MAX,
        grants: u16::MAX,
        share: [255, 255],
        hold: [10, 10],
        on_gold: [2, 2],
        winner: Some(1),
        finish: Some(Finish::Left),
        across: [100, 100],
        players: [u8::MAX, u8::MAX],
    };
    let bytes = postcard::to_allocvec(&longest).unwrap();
    assert!(bytes.len() <= 20, "{} bytes", bytes.len());
    assert_eq!(postcard::from_bytes::<RaceRound>(&bytes).unwrap(), longest);
    // As long as it is in a game, won, the clock at its longest.
    let won = RaceRound {
        left: PLAY_SECS as u16,
        grants: grants_by(PLAY_SECS),
        share: [60, 60],
        hold: [10, 10],
        on_gold: [2, 2],
        winner: Some(1),
        finish: Some(Finish::Held),
        across: [100, 100],
        players: [2, 2],
    };
    let most = postcard::to_allocvec(&won).unwrap().len();
    assert!(most <= 17, "{most} bytes");
    // And as it most often is.
    let bytes = postcard::to_allocvec(&round).unwrap();
    assert!(bytes.len() <= 15, "{} bytes", bytes.len());
    println!("the round: {} bytes as it begins, {most} at the most in a game, {} at the most it could be", bytes.len(), postcard::to_allocvec(&longest).unwrap().len());
    // What a device says and is told, too.
    for ask in [Ask::Metres { metres: 12.5, grants: 3 }] {
        let bytes = postcard::to_allocvec(&ask).unwrap();
        assert!(bytes.len() <= 8);
        assert_eq!(postcard::from_bytes::<Ask>(&bytes).unwrap(), ask);
    }
    for tell in [
        Tell::Metres { metres: 3.0, grants: 9 },
        Tell::Feed(Feed::Fell { seat: 3, by: Some(0) }),
        Tell::Feed(Feed::Blew { by: 1, team: Some(0), planks: 4 }),
        Tell::Feed(Feed::Lost { by: 2 }),
    ] {
        let bytes = postcard::to_allocvec(&tell).unwrap();
        assert!(bytes.len() <= 8, "{tell:?}: {} bytes", bytes.len());
        assert_eq!(postcard::from_bytes::<Tell>(&bytes).unwrap(), tell);
    }
    // Everyone's kit as the room is sent it, every one with two sticks.
    let kits = crate::Kits(vec![with_sticks(Kit::default(), DYNAMITE_MOST); 4]);
    assert!(postcard::to_allocvec(&kits).unwrap().len() <= 64);
    // And the game is one a server keeps among its tables, on any thread.
    fn kept<T: Send + Sync + 'static>() {}
    kept::<Game>();
}

#[test]
fn the_computer_s_players_lay_their_team_s_way_and_stand_on_the_golden_island() {
    // A game of one against one of the computer alone, watched: each
    // lays its own team's way, from its island toward the golden island,
    // and the one that wins stands there.
    let mut game = Game::new(2, &[false, false], 11);
    game.begin();
    let ways = [game.crew().way(0).expect("red's way"), game.crew().way(1).expect("blue's way")];
    for (side, way) in ways.iter().enumerate() {
        assert_eq!(game.nav().grid().region(way.from), Some(Region::Team(side as u8)));
        assert_eq!(game.nav().grid().region(way.to), Some(Region::Middle));
        assert!(way.length() < 70.0, "{} m", way.length());
    }
    let mut planks = [0usize; 2];
    while game.playing() {
        for event in game.step(FRAME) {
            if let Event::Plank(plank) = event {
                let side = usize::from(team(usize::from(plank.seat)));
                // Every rung on its own team's way, or a short way of its
                // own onto the golden island.
                let at = Vec2::new(plank.x, plank.z);
                let on_way = (0..=ways[side].planks()).any(|rung| ways[side].rung(rung).distance(at) < 0.05);
                let toward_gold = at.distance(GOLDEN) < ISLAND_RADIUS + 12.0;
                assert!(on_way || toward_gold, "seat {} laid at {at}", plank.seat);
                planks[side] += 1;
            }
        }
    }
    assert!(planks.iter().all(|&laid| laid > 20), "{planks:?}");
    let (winner, finish, _) = game.result().expect("over");
    assert_eq!(finish, Finish::Held);
    let winner = winner.expect("won");
    assert!(game.on_gold(usize::from(winner)), "the winner stands on the golden island");
}

#[test]
fn the_chat_says_who_fell_by_whose_doing_and_whose_bridge_a_stick_holed() {
    // A fall, by nobody's doing and by the other team's.
    assert_eq!(feed(&Event::Fell { seat: 2, by: None }), Some(Feed::Fell { seat: 2, by: None }));
    assert_eq!(feed(&Event::Fell { seat: 3, by: Some(0) }), Some(Feed::Fell { seat: 3, by: Some(0) }));
    // A stick of blue's (seat 1): mostly red's planks, red's bridge holed;
    // only its own team's, its own; none, nobody's.
    let blew = |planks: Vec<(u8, u16, u16)>| Event::Blew { stick: 0, by: 1, at: Vec3::ZERO, planks };
    assert_eq!(feed(&blew(vec![(0, 0, 3), (0, 0, 4), (2, 1, 0)])), Some(Feed::Blew { by: 1, team: Some(0), planks: 3 }));
    assert_eq!(feed(&blew(vec![(0, 0, 3), (1, 0, 9), (3, 0, 9)])), Some(Feed::Blew { by: 1, team: Some(0), planks: 3 }));
    assert_eq!(feed(&blew(vec![(3, 0, 9)])), Some(Feed::Blew { by: 1, team: Some(1), planks: 1 }));
    assert_eq!(feed(&blew(Vec::new())), Some(Feed::Blew { by: 1, team: None, planks: 0 }));
    // A stick that came down on nothing, whose it was.
    assert_eq!(feed(&Event::Lost { stick: 4, by: 3 }), Some(Feed::Lost { by: 3 }));
    // Nothing else is the chat's.
    assert_eq!(feed(&Event::Holding { team: 0, holding: true }), None);
    assert_eq!(feed(&Event::Granted { grant: 1, sticks: vec![0] }), None);
}

#[test]
fn time_called_ends_a_game_being_played_by_the_time_limit_s_own_rule() {
    // Nothing before it begins.
    let mut game = Game::new(2, &[true, true], 4);
    game.call_time();
    assert_eq!(game.result(), None);
    // Begun, red's bridge further across: red wins it, by the time.
    game.begin();
    grants_now(&mut game, 1);
    lay_across(&mut game, 0, 0, 12);
    run(&mut game, 1.0);
    game.call_time();
    let events = run(&mut game, FRAME);
    assert!(events.contains(&Event::Over { winner: Some(0), finish: Finish::Time }), "{events:?}");
    assert!(!game.playing());
    // The round says so: time, with the clock not yet run out.
    assert_eq!((game.round().winner, game.round().finish), (Some(0), Some(Finish::Time)));
    assert!(game.round().left > 0);
    // And once over, again is nothing.
    game.call_time();
    assert!(run(&mut game, FRAME).is_empty());
}

#[test]
fn a_person_whose_device_has_gone_counts_for_nothing_on_the_golden_island_and_falls_as_they_would_have() {
    // One against one of people, red's on the golden island: holding it.
    let mut game = people(2, 16);
    on_gold(&mut game, 0, 2.0);
    let events = run(&mut game, 1.0);
    assert!(events.contains(&Event::Holding { team: 0, holding: true }));
    // Red's device gone, its body standing there with nothing to knock it
    // off: no longer on it, the ring empty, and nobody wins by it.
    game.away(0, true);
    let events = run(&mut game, FRAME);
    assert!(events.contains(&Event::Holding { team: 0, holding: false }));
    assert_eq!((game.round().on_gold, game.round().hold), ([0, 0], [0, 0]));
    assert!(game.standing(0), "standing there all the same");
    run(&mut game, HOLD + 1.0);
    assert!(game.playing(), "{:?}", game.result());
    // Back, it is on it again, and holds it, and wins.
    game.away(0, false);
    let events = run(&mut game, HOLD + 0.2);
    assert!(events.contains(&Event::Over { winner: Some(0), finish: Finish::Held }), "{:?}", game.result());
    // Nor does it count at the time limit: red's there and gone, blue's at
    // home, neither bridge out, a draw (with red's device there, red's).
    let mut game = people(2, 16);
    on_gold(&mut game, 0, 2.0);
    game.sync(1, Vec3::new(ISLANDS[1].x, LAND_TOP, ISLANDS[1].y), 0.0);
    game.away(0, true);
    run(&mut game, PLAY_SECS + 0.5);
    assert_eq!(game.result().map(|(winner, finish, _)| (winner, finish)), Some((None, Finish::Time)));
    // Gone standing on a bridge, it stands; last said under it, on its way
    // down into the void, or over the void beside it, it falls.
    let mut game = people(2, 16);
    grants_now(&mut game, 1);
    let span = lay_across(&mut game, 0, 0, 6);
    let on = span.rung(3);
    game.sync(0, Vec3::new(on.x, LAND_TOP, on.y), 0.0);
    game.away(0, true);
    run(&mut game, 1.0);
    assert!(game.standing(0), "fell from the bridge it stood on");
    game.sync(0, Vec3::new(on.x, LAND_TOP - 3.0, on.y), 0.0);
    assert!(game.nav().ground(on).is_some(), "the bridge is ground there");
    assert!(run(&mut game, FRAME).contains(&Event::Fell { seat: 0, by: None }));
    // Stood up again on its island, gone or not, it stays.
    run(&mut game, RESPAWN + 0.2);
    assert!(game.standing(0));
    let beside = on + (span.rung(4) - span.rung(3)).normalize().perp() * 12.0;
    assert!(game.nav().ground(beside).is_none(), "{beside} is ground");
    game.sync(0, Vec3::new(beside.x, LAND_TOP, beside.y), 0.0);
    assert!(run(&mut game, FRAME).contains(&Event::Fell { seat: 0, by: None }));
}

#[test]
fn a_rung_past_the_metres_a_person_has_been_given_is_not_taken_nor_one_laid_a_while_after_they_fell() {
    // A device that says it has more than it was given, laying straight
    // across at the start: taken as far as the ten metres it began with,
    // and what its own bridge lays past them, and no further.
    let mut game = people(2, 17);
    let span = game.nav().grid().span(Region::Team(0), Region::Middle, 0.0).expect("red's way");
    let lay = |game: &mut Game, index: usize| {
        let at = span.rung(index);
        game.sync(0, Vec3::new(at.x, LAND_TOP, at.y), span.heading());
        game.rung(0, 0, index as u16, at, span.heading_at(index))
    };
    let taken = (0..=span.planks()).take_while(|&index| lay(&mut game, index)).last().expect("rungs");
    let laid = game.laid()[0];
    assert!(taken < span.planks() && laid > START_METRES, "{laid} m laid in {taken} rungs");
    assert!(laid <= START_METRES + SPARE_EACH + SPARE_ONCE, "{laid} m laid in {taken} rungs");
    assert!(!lay(&mut game, taken + 1), "and again");
    // A grant comes: on as far as it gives.
    run(&mut game, GRANT_EVERY + 0.1);
    assert!(lay(&mut game, taken + 1));
    // Fallen: the plank in hand as they fell, which their device sends
    // just after, is taken; one laid a while after, until they stand
    // again, is not.
    game.fell(0);
    assert!(lay(&mut game, taken + 2));
    run(&mut game, FALLEN_RUNG + 0.1);
    assert!(!lay(&mut game, taken + 3));
    run(&mut game, RESPAWN);
    assert!(game.standing(0));
    assert!(lay(&mut game, taken + 3));
}

#[test]
fn every_rung_a_device_lays_on_what_it_is_given_is_taken_however_often_it_runs_out() {
    // A device laying as the game's own bridge lays, on what it is given
    // and no more: what it has laid a plank at a time, and as it runs out
    // the plank in hand fixed a step ahead of you whatever is left (1.05 m
    // past what it has), at the start and at every grant, to the end of a
    // game, two against two and one against one: every rung of it taken.
    for (seats, grants) in [(4, grants_by(PLAY_SECS - 1.0)), (2, 14)] {
        let mut game = people(seats, 18);
        let way = (GOLDEN - ISLANDS[0]).normalize();
        let shore = ISLANDS[0] + way * (ISLAND_RADIUS - 0.5);
        let mut index = 0u16;
        let mut along = 0.0;
        let mut lay = |game: &mut Game, along: f32| {
            let at = shore + way * along;
            game.sync(0, Vec3::new(at.x, LAND_TOP, at.y), 0.0);
            let taken = game.rung(0, 0, index, at, 0.0);
            index += 1;
            taken
        };
        assert!(lay(&mut game, along));
        let mut metres = START_METRES;
        for grant in 0..=grants {
            if grant > 0 {
                while game.round().grants < grant {
                    game.step(FRAME);
                }
                metres += game.round().share_of(0);
            }
            while metres >= crate::nav::PLANK_SPACING {
                along += crate::nav::PLANK_SPACING;
                metres -= crate::nav::PLANK_SPACING;
                assert!(lay(&mut game, along), "{seats} seats, grant {grant}: a plank at {along} m");
            }
            along += 1.05;
            metres = 0.0;
            assert!(lay(&mut game, along), "{seats} seats, grant {grant}: the plank in hand at {along} m");
            // Back on the island to wait for the next (out on the golden
            // island, one alone of a team would hold it and win).
            game.sync(0, Vec3::new(ISLANDS[0].x, LAND_TOP, ISLANDS[0].y), 0.0);
        }
        assert!(game.playing());
        println!("{seats} seats: {:.1} m laid on {grants} grants", game.laid()[0]);
    }
}
