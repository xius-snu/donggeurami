//! Whole games of Last Stand played by stand-ins for people, by numbers, to
//! hold the game to what it is for and to tune it by: one alone should
//! clear five to nine waves, six or seven most often, so that a decent player
//! has the fee back at five and a good one does well (Hajun: "Generally
//! should be above 50% to win net positive coin"); four, nine to fifteen; and
//! a game should take eight to twenty minutes. `cargo test -p roundtown_net
//! stand_many -- --ignored --nocapture` prints a table of many
//! (`RT_GAMES` games for each number of people, from the seed `RT_FROM`).

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::*;
use crate::bots::Dice;
use crate::bridge_wars::{AIR_DRAG, FAST, GROUND_DRAG, KNOCK_BACK, KNOCK_UP, RUN_SPEED, SPRINT_BONUS, Shop};
use crate::nav::BODY;
use crate::{Item, Kit, Use, Utility};

/// How long a frame of a whole game is, in seconds.
const DT: f32 = 1.0 / 20.0;
/// The longest a whole game is played before the test gives it up.
const LONGEST: f32 = 60.0 * 60.0;

/// What a stand-in is doing between waves.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Errand {
    /// To this shop's stand, to buy what it wants there.
    Shop(Shop),
    /// To this pad, for what lies on it.
    Pad(usize),
    /// Back by the anchor.
    Home,
}

/// A stand-in for a person, played by numbers: it keeps by the anchor and
/// goes for the zombie nearest it of those at the anchor or at itself, a
/// spitter spitting at the anchor, and anything once the anchor is broken or
/// a wave's last few are left; presses a few times a second, as a person
/// does, each press landing, most of the time, on whichever zombie in reach
/// is nearest and not just sent flying; steps out of a spitter's ball; picks
/// up what lies near it; drinks a healing potion when it is low; throws a
/// fireball or a stick into a crowd now and then; goes round what holds it
/// up, the way it took; and between waves takes what the pads have made,
/// buys by a greedy list, saving for what it wants most, and presses Ready
/// near a break's end once it has no sword, shield or generator step to save
/// for. Knocked, it is carried as a body is; over the rim, it falls. Not a
/// good player: a plain one, which a person who minds their health, kites
/// and picks what to hit does better than.
struct StandIn {
    seat: usize,
    at: Vec3,
    shove: Vec2,
    lift: f32,
    dead: bool,
    /// Until it may swing again, and what share of its swings land.
    swing: f32,
    knack: f32,
    errand: Errand,
    /// Which way round what holds it up it is going: none while nothing
    /// does.
    side: f32,
    dice: Dice,
}

impl StandIn {
    fn new(seat: usize, seed: u64, at: Vec3) -> Self {
        let mut dice = Dice::new(seed.wrapping_mul(31).wrapping_add(seat as u64 + 7));
        let knack = dice.around(0.75, 0.95);
        Self {
            seat,
            at,
            shove: Vec2::ZERO,
            lift: 0.0,
            dead: false,
            swing: 0.0,
            knack,
            errand: Errand::Home,
            side: 0.0,
            dice,
        }
    }

    /// What happened to it last frame.
    fn hear(&mut self, events: &[Event]) {
        for event in events {
            match *event {
                Event::Struck { seat, from, knock, .. } if seat == self.seat => {
                    let away = (self.at.xz() - from.xz()).normalize_or(Vec2::X);
                    let push = match knock {
                        Knock::None => continue,
                        Knock::Light => KNOCK_BACK,
                        Knock::Heavy => KNOCK_BACK + SPRINT_BONUS,
                    };
                    self.shove = self.shove * 0.5 + away * push;
                    self.lift = (self.lift * 0.5 + KNOCK_UP).min(KNOCK_UP);
                }
                Event::Died { seat, .. } | Event::Out { seat } if seat == self.seat => {
                    self.dead = true;
                    self.shove = Vec2::ZERO;
                    self.lift = 0.0;
                }
                Event::Back { seat, at, .. } if seat == self.seat => {
                    self.dead = false;
                    self.at = at;
                }
                _ => {}
            }
        }
    }

    /// Carried by a knock and its weight, as a body is: whether it is in
    /// the air. Fallen far into the void, it says so.
    fn carry(&mut self, game: &mut Game, dt: f32) -> bool {
        let floor = ground(self.at.xz());
        let base = floor.unwrap_or(f32::NEG_INFINITY);
        let airborne = self.lift > 0.0 || (self.at.y - base).abs() > 1e-3;
        if airborne {
            let before = self.at.y;
            self.at.y += self.lift * dt;
            self.lift -= if self.lift > 0.0 { 22.0 } else { 33.0 } * dt;
            if self.at.y <= base && before >= base - 0.3 {
                self.at.y = base;
                self.lift = 0.0;
            }
        } else if self.lift != 0.0 {
            self.at.y = base;
            self.lift = 0.0;
        }
        if self.shove != Vec2::ZERO {
            let to = keep_out(self.at.xz() + self.shove * dt, BODY);
            self.at = Vec3::new(to.x, self.at.y, to.y);
            self.shove *= if airborne { AIR_DRAG } else { GROUND_DRAG }.powf(dt * 20.0);
            if self.shove.length_squared() < 0.01 {
                self.shove = Vec2::ZERO;
            }
        }
        if self.at.y < SAND_TOP - 8.0 {
            game.fell(self.seat);
            self.dead = true;
        }
        self.lift != 0.0 || ground(self.at.xz()).is_none_or(|floor| (self.at.y - floor).abs() > 1e-3)
    }

    /// A frame of it.
    fn play(&mut self, game: &mut Game, dt: f32) {
        if self.dead || !game.alive(self.seat) {
            return;
        }
        self.swing = (self.swing - dt).max(0.0);
        if self.carry(game, dt) || self.dead {
            game.sync(self.seat, self.at, 0.0);
            return;
        }
        let kit = game.kits()[self.seat];
        let elapsed = game.elapsed();
        let pace = RUN_SPEED * if kit.fast(elapsed) { FAST } else { 1.0 };
        // Low, it drinks.
        if game.health(self.seat) < self.dice.around(25.0, 45.0) && kit.held(Utility::Healing) > 0 {
            game.use_item(self.seat, Use::Potion(Utility::Healing));
        }
        let round = game.round();
        let to = match round.stage {
            StandStage::Wave => self.fight(game, &kit),
            _ => self.errands(game, &kit),
        };
        // Out of the way of a ball about to come down on it.
        let mut to = to;
        for (_, spit) in game.spits() {
            let left = spit.secs() - spit.age;
            if left < 0.7 && spit.to.xz().distance(self.at.xz()) < SPIT_RADIUS + BODY + 0.3 {
                let away = (self.at.xz() - spit.to.xz()).normalize_or(Vec2::X);
                to = self.at.xz() + away * 3.0;
            }
        }
        self.walk(to, pace, dt);
        game.sync(self.seat, self.at, 0.0);
    }

    /// Where it goes in a wave, and what it hits: it guards the anchor,
    /// going for the zombie nearest it of those at the anchor, or of those
    /// at itself, and never further from the anchor than a few metres; and
    /// swings at whatever is in its reach, a press every so often, most of
    /// them landing; a fireball or a stick into a crowd now and then.
    fn fight(&mut self, game: &mut Game, kit: &Kit) -> Vec2 {
        let me = self.at.xz();
        let home = arrival(self.seat).0.xz();
        let zombies: Vec<(u32, Kind, Vec3)> = game.zombies().map(|(id, kind, at, ..)| (id, kind, at)).collect();
        // With the anchor broken there is nothing to guard, and with the
        // last few of a wave left there is nothing else to do: anything
        // goes.
        let few = usize::from(game.round().left) == zombies.len() && zombies.len() <= 5;
        let broken = game.anchor().0 <= 0.0 || few;
        let guards = |at: Vec2| broken || at.distance(ANCHOR) < 7.0;
        let target = zombies
            .iter()
            .filter(|(id, kind, at)| {
                let at = at.xz();
                !game.climbing(*id)
                    && (broken
                        || at.distance(ANCHOR) < 6.0
                        || (*kind == Kind::Spitter && at.distance(ANCHOR) < 14.0)
                        || (at.distance(me) < 4.0 && guards(at)))
            })
            .min_by(|a, b| a.2.xz().distance(me).total_cmp(&b.2.xz().distance(me)));
        // A crowd within a throw: a fireball, or a stick if it is a big one.
        let crowd = |around: Vec2| zombies.iter().filter(|(_, _, at)| at.xz().distance(around) < 3.0).count();
        if let Some((_, _, at)) = zombies.iter().find(|(_, _, at)| {
            let far = at.xz().distance(me);
            (4.0..9.0).contains(&far) && crowd(at.xz()) >= 3
        }) {
            let way = at.xz() - me;
            let heading = (-way.x).atan2(-way.y);
            if kit.held(Utility::Dynamite) > 0 && crowd(at.xz()) >= 4 && self.dice.roll() < 0.05 {
                game.use_item(self.seat, Use::Dynamite { heading, far: way.length() });
            } else if kit.held(Utility::Fireball) > 0 && self.dice.roll() < 0.05 {
                game.use_item(self.seat, Use::Fireball { heading });
            }
        }
        // A press every so often, as a person presses: it lands on the
        // nearest in reach that the last press did not just send flying
        // (the game says so, by not counting it), most of the time.
        if self.swing <= 0.0 {
            self.swing = self.dice.around(0.25, 0.4);
            if self.dice.roll() < self.knack {
                let mut near: Vec<&(u32, Kind, Vec3)> =
                    zombies.iter().filter(|(_, kind, at)| at.xz().distance(me) <= 2.0 + BODY * kind.size()).collect();
                near.sort_by(|a, b| a.2.xz().distance(me).total_cmp(&b.2.xz().distance(me)));
                for (id, ..) in near {
                    if game.punch(self.seat, *id, false) {
                        break;
                    }
                }
            }
        }
        let Some(&(_, kind, at)) = target else {
            return self.pick_up_near(game, home, 8.0);
        };
        let far = at.xz().distance(me);
        let reach = 2.0 + BODY * kind.size();
        if far > reach * 0.8 { at.xz() } else { me }
    }

    /// Whatever lies within `within` of it, the nearest first; or `home`.
    fn pick_up_near(&self, game: &Game, home: Vec2, within: f32) -> Vec2 {
        let me = self.at.xz();
        game.pieces()
            .iter()
            .filter(|piece| piece.lying(game.elapsed()) && piece.at.distance(me) < within && piece.at.distance(ANCHOR) < 20.0)
            .min_by(|a, b| a.at.distance(me).total_cmp(&b.at.distance(me)))
            .map_or(home, |piece| piece.at)
    }

    /// What it would buy now, in the order it wants it, out of what it holds:
    /// a better sword, a better shield, the generators to amethyst, the
    /// anchor fortified when it is low, two healing potions, then the
    /// cheapest of the upgrades, and fireballs with silver to spare.
    fn wants(game: &Game, seat: usize) -> Option<Ware> {
        let (kit, purse, team) = (game.kits()[seat], game.purses()[seat], game.upgrades());
        let (anchor, most) = game.anchor();
        let mut list: Vec<Ware> = Vec::new();
        list.extend((1..=3).map(Ware::Sword).filter(|ware| matches!(ware, Ware::Sword(tier) if *tier == kit.sword + 1)));
        list.extend((1..=3).map(Ware::Shield).filter(|ware| matches!(ware, Ware::Shield(tier) if *tier == kit.shield + 1)));
        if team.generator < 3 {
            list.push(Ware::Generator);
        }
        if anchor < most * 0.7 {
            list.push(Ware::Fortify);
        }
        if kit.held(Utility::Healing) < 2 {
            list.push(Ware::Utility(Utility::Healing));
        }
        // The cheapest of the rest, by what it costs in silver's worth.
        let worth = [1.0, 4.0, 12.0, 30.0];
        let mut upgrades: Vec<(f32, Ware)> = Ware::UPGRADES
            .into_iter()
            .filter_map(|ware| price(ware, &kit, &team).map(|(item, cost)| (f32::from(cost) * worth[item.index()], ware)))
            .collect();
        upgrades.sort_by(|a, b| a.0.total_cmp(&b.0));
        list.extend(upgrades.into_iter().map(|(_, ware)| ware));
        if purse[Item::Silver.index()] > 60 {
            list.push(Ware::Utility(Utility::Fireball));
        }
        if purse[Item::Amethyst.index()] > 2 {
            list.push(Ware::Utility(Utility::Dynamite));
        }
        // The first it can have; what it cannot have yet it saves for, and
        // spends none of that on what it wants less.
        let mut spare = purse;
        for ware in list {
            let Some((item, cost)) = price(ware, &kit, &team) else {
                continue;
            };
            if spare[item.index()] >= cost {
                return Some(ware);
            }
            spare[item.index()] = 0;
        }
        None
    }

    /// Where it goes between waves: what lies near the anchor, the pads,
    /// the stand that sells what it wants, and back.
    fn errands(&mut self, game: &mut Game, _kit: &Kit) -> Vec2 {
        let me = self.at.xz();
        let home = arrival(self.seat).0.xz();
        // Buy what it can where it stands.
        if let Some(ware) = Self::wants(game, self.seat)
            && at_a_stand(me, sold_at(ware))
            && me.distance(match sold_at(ware) {
                Shop::Items => ITEM_SHOP,
                Shop::Upgrades => UPGRADE_SHOP,
            }) < 1.2
        {
            game.buy(self.seat, ware, me);
        }
        let near = self.pick_up_near(game, home, 6.0);
        if near != home {
            return near;
        }
        // Each errand until it is done.
        let lying = |pad: usize| game.pieces().iter().filter(|piece| !piece.dropped() && piece.at.distance(PADS[pad]) < 1.5).count();
        self.errand = match self.errand {
            Errand::Shop(shop) if Self::wants(game, self.seat).is_some_and(|ware| sold_at(ware) == shop) => Errand::Shop(shop),
            Errand::Pad(pad) if lying(pad) > 0 => Errand::Pad(pad),
            _ => match Self::wants(game, self.seat) {
                Some(ware) => Errand::Shop(sold_at(ware)),
                None => match (0..PADS.len()).max_by_key(|pad| lying(*pad)) {
                    Some(pad) if lying(pad) > 2 => Errand::Pad(pad),
                    _ => Errand::Home,
                },
            },
        };
        match self.errand {
            Errand::Shop(Shop::Items) => ITEM_SHOP,
            Errand::Shop(Shop::Upgrades) => UPGRADE_SHOP,
            Errand::Pad(pad) => PADS[pad],
            Errand::Home => {
                // Anything lying further about, before the next wave; and
                // with nothing, and the break nearly over, it is ready for
                // it, as someone done with their shopping is.
                let to = self.pick_up_near(game, home, 20.0);
                let left = match game.round().stage {
                    StandStage::Break { until } => until as f32 - game.elapsed(),
                    _ => 0.0,
                };
                // Not while it saves for its next sword, shield or
                // generator, which the break's generators may yet bring.
                let (kit, team) = (game.kits()[self.seat], game.upgrades());
                let saving = [Ware::Sword(kit.sword + 1), Ware::Shield(kit.shield + 1), Ware::Generator]
                    .into_iter()
                    .filter(|ware| *ware != Ware::Generator || team.generator < 3)
                    .any(|ware| price(ware, &kit, &team).is_some());
                if to == home && left < 10.0 && !saving {
                    game.ready(self.seat);
                }
                to
            }
        }
    }

    /// Walks toward `to` at `pace` for `dt`: round what is in its way (the
    /// straight step, or, held up by something, the step turned aside that
    /// gets it nearest), and never over the rim of itself.
    fn walk(&mut self, to: Vec2, pace: f32, dt: f32) {
        let me = self.at.xz();
        let off = to - me;
        if off.length() < 0.05 {
            return;
        }
        let step = off.clamp_length_max(pace * dt);
        let taken = |turn: f32| {
            let mut next = keep_out(me + Vec2::from_angle(turn).rotate(step), BODY);
            if next.length() > WALK_EDGE {
                next = next.normalize() * WALK_EDGE;
            }
            next
        };
        let mut next = taken(0.0);
        if next.distance(me) < step.length() * 0.5 {
            // Held up: round it, the side it took the first time it was
            // held up here (the nearer way then), until it is past.
            let side = match self.side {
                0.0 => {
                    let (left, right) = (taken(1.2), taken(-1.2));
                    if left.distance(to) <= right.distance(to) { 1.0 } else { -1.0 }
                }
                side => side,
            };
            self.side = side;
            next = [0.6, 1.2, 1.6]
                .into_iter()
                .map(|turn| taken(turn * side))
                .find(|next| next.distance(me) > step.length() * 0.5)
                .unwrap_or(next);
        } else {
            self.side = 0.0;
        }
        let floor = ground(next).unwrap_or(self.at.y);
        self.at = Vec3::new(next.x, floor, next.y);
    }
}

/// How a whole game went.
#[derive(Clone, Debug, Default)]
struct Told {
    seed: u64,
    people: usize,
    waves: u16,
    minutes: f32,
    /// Kills by kind, as [`Kind::ALL`] has them, and how many went into the
    /// void; and how many walked off the rim by themselves (none, ever).
    kills: [u32; 6],
    void: u32,
    walked_off: u32,
    deaths: u32,
    /// When the anchor broke, in which wave.
    broke: Option<u16>,
    /// What was bought, by kind: swords, shields, things to use, and each
    /// upgrade, as [`Ware::UPGRADES`] has them.
    swords: u32,
    shields: u32,
    things: u32,
    upgrades: [u32; 6],
    /// How many balls were spat, leaps landed, things dropped.
    spat: u32,
    zaps: u32,
    dropped: u32,
    /// What the team scored, and what was left in the purses at the end, in
    /// silver's worth.
    score: u32,
    left: f32,
    /// Whether the endless lines still had something to buy at the end.
    endless: bool,
    /// Whether it ended, within [`LONGEST`].
    over: bool,
}

/// A whole game of `people` stand-ins from `seed`.
fn played(people: usize, seed: u64) -> Told {
    let mut game = Game::new(people, &vec![true; people], seed);
    game.begin();
    let mut stand_ins: Vec<StandIn> = (0..people).map(|seat| StandIn::new(seat, seed, game.at(seat).0)).collect();
    let mut told = Told {
        seed,
        people,
        ..Told::default()
    };
    let mut events: Vec<Event> = Vec::new();
    while !game.over() && game.elapsed() < LONGEST {
        for stand_in in &mut stand_ins {
            stand_in.hear(&events);
            stand_in.play(&mut game, DT);
        }
        events = game.step(DT);
        for event in &events {
            match *event {
                Event::Killed { kind, by, void, .. } => {
                    let index = Kind::ALL.iter().position(|other| *other == kind).expect("a kind");
                    told.kills[index] += 1;
                    if void {
                        told.void += 1;
                        if by.is_none() {
                            told.walked_off += 1;
                        }
                    }
                }
                Event::Died { .. } => told.deaths += 1,
                Event::AnchorBroken => told.broke = Some(game.wave()),
                Event::Bought { ware, .. } => match ware {
                    Ware::Sword(_) => told.swords += 1,
                    Ware::Shield(_) => told.shields += 1,
                    Ware::Utility(_) => told.things += 1,
                    upgrade => {
                        let index = Ware::UPGRADES.iter().position(|other| *other == upgrade).expect("an upgrade");
                        told.upgrades[index] += 1;
                    }
                },
                Event::Spat { .. } => told.spat += 1,
                Event::Zap { .. } => told.zaps += 1,
                Event::Made(piece) if piece.dropped() => told.dropped += 1,
                _ => {}
            }
        }
    }
    told.waves = game.cleared();
    told.minutes = game.elapsed() / 60.0;
    told.score = game.team_score();
    let worth = [1.0, 4.0, 12.0, 30.0];
    told.left = game.purses().iter().flat_map(|purse| purse.iter().zip(worth).map(|(count, worth)| f32::from(*count) * worth)).sum();
    told.endless = [Ware::Generator, Ware::Sharpness, Ware::Fortify, Ware::Beam]
        .into_iter()
        .all(|ware| price(ware, &Kit::default(), &game.upgrades()).is_some());
    told.over = game.over();
    told
}

/// The games of `seeds` with `people` stand-ins each, a line each.
fn told(people: usize, seeds: std::ops::Range<u64>) -> Vec<Told> {
    let games: Vec<Told> = seeds.map(|seed| played(people, seed)).collect();
    println!("{people} | seed | waves | min  | walk run brute giant spit leap | void | died | broke | sw sh it | gen shp pro for bm lk | spat zaps drops | score | left");
    for game in &games {
        let [walk, run, brute, giant, spit, leap] = game.kills;
        let [generator, sharp, protect, fortify, beam, lucky] = game.upgrades;
        println!(
            "{} | {:>4} | {:>5} | {:>4.1} | {walk:>4} {run:>3} {brute:>5} {giant:>5} {spit:>4} {leap:>4} | {:>4} | {:>4} | {:>5} | {:>2} {:>2} {:>2} | {generator:>3} {sharp:>3} {protect:>3} {fortify:>3} {beam:>2} {lucky:>2} | {:>4} {:>4} {:>5} | {:>5} | {:>4.0}",
            game.people,
            game.seed,
            game.waves,
            game.minutes,
            game.void,
            game.deaths,
            game.broke.map_or(String::from("-"), |wave| wave.to_string()),
            game.swords,
            game.shields,
            game.things,
            game.spat,
            game.zaps,
            game.dropped,
            game.score,
            game.left,
        );
        if !game.over {
            println!("  ^ not over after {:.0} minutes", game.minutes);
        }
    }
    games
}

/// The middle one of `values`.
fn median(mut values: Vec<f32>) -> f32 {
    values.sort_by(f32::total_cmp);
    values.get(values.len() / 2).copied().unwrap_or(0.0)
}

#[test]
fn whole_games_end_at_a_wave_that_fits_the_team_and_nothing_absurd_happens() {
    let alone = told(1, 1..7);
    let four = told(4, 1..4);
    for game in alone.iter().chain(&four) {
        let seed = game.seed;
        assert!(game.over, "seed {seed}, {} people: not over after {} minutes", game.people, game.minutes);
        // Never a zombie off the rim of its own accord.
        assert_eq!(game.walked_off, 0, "seed {seed}");
        // The anchor broke before the end, which is how a game ends.
        assert!(game.broke.is_some(), "seed {seed}");
        // Something was bought, of each shop.
        assert!(game.swords >= 1 && game.upgrades.iter().sum::<u32>() >= 1, "seed {seed}");
        // The endless upgrades never run out.
        assert!(game.endless, "seed {seed}");
        // Over in a sensible time.
        assert!(game.minutes > 2.0 && game.minutes < 45.0, "seed {seed}: {} minutes", game.minutes);
    }
    // Alone, a few waves and not many; four together, more.
    let waves = |games: &[Told]| median(games.iter().map(|game| f32::from(game.waves)).collect());
    let (one, many) = (waves(&alone), waves(&four));
    assert!((3.0..=11.0).contains(&one), "alone: a median of {one} waves");
    assert!(many > one, "four together {many}, alone {one}");
    // The same dice, the same game.
    let (a, b) = (played(2, 11), played(2, 11));
    assert_eq!((a.waves, a.kills, a.deaths, a.score), (b.waves, b.kills, b.deaths, b.score));
}

#[test]
fn every_kind_of_zombie_comes_and_does_its_thing_in_a_long_game() {
    // Four stand-ins, with a team that has bought nothing yet but is far
    // along: from wave twelve, every kind comes, and each does what it does.
    let mut game = Game::new(4, &[true; 4], 77);
    game.begin();
    let mut stand_ins: Vec<StandIn> = (0..4).map(|seat| StandIn::new(seat, 77, game.at(seat).0)).collect();
    let (mut kinds, mut spat, mut struck, mut knocked) = ([false; 6], 0, 0, 0);
    let mut events: Vec<Event> = Vec::new();
    // Skip to wave 12, as a team that has done well: the stand-ins armed.
    game.wave = 11;
    game.upgrades = StandUpgrades {
        sharpness: 10,
        protection: 5,
        fortify: 4,
        ..StandUpgrades::default()
    };
    game.anchor_most = ANCHOR_HEALTH + 4.0 * FORTIFY;
    game.anchor = game.anchor_most;
    for kit in &mut game.kits {
        kit.sword = 3;
        kit.shield = 3;
    }
    while !game.over() && game.wave() <= 12 && game.elapsed() < 600.0 {
        for stand_in in &mut stand_ins {
            stand_in.hear(&events);
            stand_in.play(&mut game, DT);
        }
        events = game.step(DT);
        for event in &events {
            match *event {
                Event::Risen { kind, .. } => kinds[Kind::ALL.iter().position(|other| *other == kind).unwrap()] = true,
                Event::Spat { .. } => spat += 1,
                Event::Struck { knock, .. } => {
                    struck += 1;
                    knocked += u32::from(knock != Knock::None);
                }
                _ => {}
            }
        }
    }
    // Wave 12 has no giant; every other kind came, and the spitters spat
    // and somebody was hit hard (a brute or a leaper).
    assert_eq!(kinds, [true, true, true, false, true, true], "{kinds:?}");
    assert!(spat > 0 && struck > 0 && knocked > 0, "spat {spat}, struck {struck}, knocked {knocked}");
}

#[test]
fn the_anchor_breaks_and_the_game_ends_exactly_when_nobody_is_standing() {
    let mut game = Game::new(2, &[true, true], 5);
    game.begin();
    // Nobody fights: the first wave goes for the anchor and breaks it in
    // the end; the two standing by are hit by the ones that come for people.
    let mut broke_at = None;
    let mut events = Vec::new();
    while !game.over() && game.elapsed() < 900.0 {
        events = game.step(DT);
        if events.contains(&Event::AnchorBroken) {
            broke_at = Some(game.elapsed());
        }
        // Standing, never over.
        if (0..2).any(|seat| game.alive(seat)) {
            assert!(!game.over());
        }
    }
    assert!(broke_at.is_some(), "the anchor stood");
    assert!(game.over() && events.iter().any(|event| matches!(event, Event::Over { waves: 0 })));
    // Nobody stands again with it broken: both are out for good.
    let round = game.round();
    assert_eq!(round.anchor, 0);
    assert!(round.back_at.iter().all(|at| at.is_infinite()), "{:?}", round.back_at);
    let (places, scores, total) = game.outcome().expect("over");
    assert_eq!(total, Some(0));
    assert_eq!(places.len(), 2);
    assert_eq!(scores, vec![0, 0]);
    // Steps after the end do nothing.
    assert!(game.step(DT).is_empty());
}

#[test]
fn a_broken_anchor_stops_anyone_standing_again() {
    let mut game = Game::new(1, &[true], 6);
    game.begin();
    game.step(DT);
    // Dead while the anchor stands: back beside it in six seconds.
    game.fell(0);
    assert!(!game.alive(0));
    let round = game.round();
    assert!(round.back_at[0] > 0.0 && round.back_at[0].is_finite());
    let mut back = false;
    for _ in 0..(7.0 / DT) as usize {
        back |= game.step(DT).iter().any(|event| matches!(event, Event::Back { seat: 0, .. }));
    }
    assert!(back && game.alive(0));
    // Dead again, and the anchor breaks while they wait: out for good, and
    // the game is over.
    game.fell(0);
    game.hit_anchor(ANCHOR_HEALTH * 2.0);
    let events = game.step(DT);
    assert!(events.contains(&Event::Out { seat: 0 }) || game.round().back_at[0].is_infinite(), "{events:?}");
    assert!(game.over());
}

#[test]
fn what_is_sent_is_small() {
    // A round for four, every number as long as it gets.
    let round = StandRound {
        wave: u16::MAX,
        stage: StandStage::Break { until: 1.8e9 },
        anchor: u16::MAX,
        anchor_most: u16::MAX,
        left: u16::MAX,
        health: vec![100; 4],
        back_at: vec![1.8e9, 0.0, f64::INFINITY, 0.0],
        scores: vec![u32::MAX; 4],
        ready: 0b1111,
    };
    let bytes = postcard::to_allocvec(&round).unwrap();
    assert!(bytes.len() < 120, "{} bytes", bytes.len());
    assert_eq!(postcard::from_bytes::<StandRound>(&bytes).unwrap(), round);
    // As it is in a game.
    let mut game = Game::new(4, &[true; 4], 1);
    game.begin();
    for _ in 0..(40.0 / DT) as usize {
        game.step(DT);
    }
    let bytes = postcard::to_allocvec(&game.round()).unwrap();
    assert!(bytes.len() < 60, "{} bytes", bytes.len());
    let shambler = Shambler {
        kind: Kind::Giant,
        health: u16::MAX,
        most: u16::MAX,
    };
    let bytes = postcard::to_allocvec(&shambler).unwrap();
    assert!(bytes.len() <= 7, "{} bytes", bytes.len());
    assert_eq!(postcard::from_bytes::<Shambler>(&bytes).unwrap(), shambler);
    let upgrades = StandUpgrades {
        generator: 300,
        sharpness: 300,
        protection: 5,
        fortify: 300,
        beam: 300,
        lucky: 5,
    };
    let bytes = postcard::to_allocvec(&upgrades).unwrap();
    assert!(bytes.len() <= 14, "{} bytes", bytes.len());
    assert_eq!(postcard::from_bytes::<StandUpgrades>(&bytes).unwrap(), upgrades);
    let spit = Spit::of(&Spittle::spat(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 0.25, 6.0)));
    let bytes = postcard::to_allocvec(&spit).unwrap();
    assert_eq!(bytes.len(), 24);
    assert_eq!(postcard::from_bytes::<Spit>(&bytes).unwrap().spittle(), Spittle::spat(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 0.25, 6.0)));
    // The asks and tells come back as they went.
    for ask in [Ask::Buy(Ware::Sword(2)), Ask::Buy(Ware::Utility(Utility::Leap)), Ask::Buy(Ware::Lucky), Ask::Ready] {
        let bytes = postcard::to_allocvec(&ask).unwrap();
        assert_eq!(postcard::from_bytes::<Ask>(&bytes).unwrap(), ask);
    }
    for tell in [
        Tell::Bought(Ware::Shield(3)),
        Tell::Upgraded {
            by: 3,
            ware: Ware::Sharpness,
            level: 12,
        },
        Tell::Zap { x: 1.0, y: 0.25, z: -3.0 },
    ] {
        let bytes = postcard::to_allocvec(&tell).unwrap();
        assert_eq!(postcard::from_bytes::<Tell>(&bytes).unwrap(), tell);
    }
}

#[test]
fn nothing_is_believed_that_could_not_be() {
    let mut game = Game::new(2, &[true, false], 3);
    // Nothing before the game begins.
    assert!(!game.buy(0, Ware::Sword(1), ITEM_SHOP));
    game.begin();
    game.purses[0] = [50, 50, 50, 50];
    // Not at the wrong stand, nor from the anchor between them, nor by
    // the seat nobody sat in.
    assert!(!game.buy(0, Ware::Sword(1), UPGRADE_SHOP));
    assert!(!game.buy(0, Ware::Sword(1), ANCHOR));
    assert!(!game.buy(1, Ware::Sword(1), ITEM_SHOP));
    assert!(!game.buy(0, Ware::Sword(1), Vec2::NAN));
    assert!(game.buy(0, Ware::Sword(1), ITEM_SHOP + Vec2::new(1.0, 0.5)));
    assert_eq!(game.purses()[0][Item::Silver.index()], 38);
    // An upgrade is the team's, and said with its level.
    assert!(game.buy(0, Ware::Fortify, UPGRADE_SHOP));
    assert_eq!(game.anchor(), (ANCHOR_HEALTH + FORTIFY, ANCHOR_HEALTH + FORTIFY));
    let events = game.step(DT);
    assert!(events.contains(&Event::Bought { seat: 0, ware: Ware::Fortify, level: 1 }), "{events:?}");
    // Nothing used that is not on a button, nor a builder at all.
    assert!(!game.use_item(0, Use::Fireball { heading: 0.0 }));
    assert!(!game.use_item(0, Use::Builder));
    // Punches: only on a zombie that is there, near enough, and not twice
    // in a hurt's while.
    for _ in 0..(25.0 / DT) as usize {
        game.step(DT);
    }
    let (id, _, at, ..) = game.zombies().next().expect("the first wave has come");
    assert!(!game.punch(0, id, false), "from the anchor to the rim");
    game.sync(0, at + Vec3::new(1.0, 0.0, 0.0), 0.0);
    assert!(game.punch(0, id, false));
    assert!(!game.punch(0, id, false), "twice in a hurt's while");
    assert!(!game.punch(0, 9999, false));
    assert!(!game.punch(1, id, false), "nobody sits there");
    // A position that is no number is not taken.
    game.sync(0, Vec3::NAN, 0.0);
    assert!(game.at(0).0.is_finite());
}

/// One game, second by second, to see what goes on in it: `cargo test -p
/// roundtown_net stand_watch -- --ignored --nocapture`, with `RT_SEED` (1)
/// and `RT_PEOPLE` (1). Each line: the time, the wave and how many of it
/// are left, the anchor's health, each stand-in's health, where it stands
/// and what it is doing, how many zombies are within 4 m of it and of the
/// anchor, and the blows that landed, on zombies and on people, that second.
#[test]
#[ignore]
fn stand_watch() {
    let seed: u64 = std::env::var("RT_SEED").ok().and_then(|seed| seed.parse().ok()).unwrap_or(1);
    let people: usize = std::env::var("RT_PEOPLE").ok().and_then(|people| people.parse().ok()).unwrap_or(1);
    let mut game = Game::new(people, &vec![true; people], seed);
    game.begin();
    let mut stand_ins: Vec<StandIn> = (0..people).map(|seat| StandIn::new(seat, seed, game.at(seat).0)).collect();
    let mut events: Vec<Event> = Vec::new();
    let (mut landed, mut struck, mut anchor_hits) = (0, 0, 0);
    let mut second = 0;
    while !game.over() && game.elapsed() < LONGEST {
        for stand_in in &mut stand_ins {
            stand_in.hear(&events);
            stand_in.play(&mut game, DT);
        }
        events = game.step(DT);
        for event in &events {
            match event {
                Event::Hurt { by: Some(_), .. } => landed += 1,
                Event::Struck { .. } => struck += 1,
                Event::AnchorHurt { .. } => anchor_hits += 1,
                Event::Wave { .. } | Event::Cleared { .. } | Event::AnchorBroken | Event::Died { .. } | Event::Back { .. } | Event::Bought { .. } => {
                    println!("      {:>6.1} {event:?}", game.elapsed());
                }
                _ => {}
            }
        }
        if game.elapsed() >= second as f32 {
            second += 1;
            let round = game.round();
            let zombies: Vec<Vec3> = game.zombies().map(|(_, _, at, ..)| at).collect();
            let near = |at: Vec2| zombies.iter().filter(|zombie| zombie.xz().distance(at) < 4.0).count();
            let them: Vec<String> = stand_ins
                .iter()
                .map(|stand_in| {
                    format!(
                        "{:>3} at ({:>5.1},{:>5.1}) {:?} near {}",
                        round.health[stand_in.seat],
                        stand_in.at.x,
                        stand_in.at.z,
                        stand_in.errand,
                        near(stand_in.at.xz())
                    )
                })
                .collect();
            // The last few of a wave, where they are and how they are.
            if round.left > 0 && round.left <= 2 && second % 10 == 0 {
                for (id, kind, at, yaw, health, _) in game.zombies() {
                    println!("      zombie {id} {kind:?} at {at:.2} facing {yaw:.2}, {health} health, climbing {}", game.climbing(id));
                }
            }
            println!(
                "{:>4}s wave {:>2} left {:>3} anchor {:>4} by it {:>2} | {} | landed {landed} struck {struck} anchor hit {anchor_hits}",
                second - 1,
                round.wave,
                round.left,
                round.anchor,
                near(ANCHOR),
                them.join(" | "),
            );
            (landed, struck, anchor_hits) = (0, 0, 0);
        }
    }
    println!("over: {} waves cleared, team score {}", game.cleared(), game.team_score());
}

/// The table to tune by: `RT_GAMES` games (100) for each number of people,
/// from the seed `RT_FROM` (100): how far each came, how long it took,
/// what it killed and bought; and for each number of people, the waves
/// cleared at the least, the most and in the middle, how many cleared
/// each, the minutes, and what a game killed and bought on the whole.
#[test]
#[ignore]
fn stand_many() {
    let count: u64 = std::env::var("RT_GAMES").ok().and_then(|count| count.parse().ok()).unwrap_or(100);
    let from: u64 = std::env::var("RT_FROM").ok().and_then(|from| from.parse().ok()).unwrap_or(100);
    let mut summary = Vec::new();
    for people in 1..=4 {
        let games = told(people, from..from + count);
        let waves: Vec<f32> = games.iter().map(|game| f32::from(game.waves)).collect();
        let minutes: Vec<f32> = games.iter().map(|game| game.minutes).collect();
        let low = waves.iter().copied().fold(f32::INFINITY, f32::min);
        let high = waves.iter().copied().fold(0.0, f32::max);
        let paid_back = waves.iter().filter(|waves| **waves >= 5.0).count();
        let spread: Vec<String> = (low as u16..=high as u16)
            .map(|wave| format!("{wave}: {}", games.iter().filter(|game| game.waves == wave).count()))
            .collect();
        let mean = |pick: &dyn Fn(&Told) -> f32| games.iter().map(pick).sum::<f32>() / games.len().max(1) as f32;
        let kills: Vec<String> = Kind::ALL
            .iter()
            .enumerate()
            .map(|(index, kind)| format!("{kind:?} {:.0}", mean(&|game| game.kills[index] as f32)))
            .collect();
        let bought: Vec<String> = Ware::UPGRADES
            .iter()
            .enumerate()
            .map(|(index, ware)| format!("{ware:?} {:.1}", mean(&|game| game.upgrades[index] as f32)))
            .collect();
        summary.push(format!(
            "{people} people: waves cleared {low:.0} to {high:.0}, {:.1} in the middle, {paid_back} of {count} with the fee back; {:.1} minutes in the middle\n   cleared ({})\n   a game kills {} (into the void {:.1}), and the people die {:.1} times\n   a game buys {:.1} swords, {:.1} shields, {:.1} things to use, and of the upgrades {}",
            median(waves.clone()),
            median(minutes),
            spread.join(", "),
            kills.join(", "),
            mean(&|game| game.void as f32),
            mean(&|game| game.deaths as f32),
            mean(&|game| game.swords as f32),
            mean(&|game| game.shields as f32),
            mean(&|game| game.things as f32),
            bought.join(", "),
        ));
    }
    for line in summary {
        println!("{line}");
    }
}
