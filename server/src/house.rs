//! Houses: who visits whose plot, what they think of it, and the board of the
//! best (since 2026-11-27; the rules, and Hajun's words for them, are
//! `roundtown_net::house`'s).
//!
//! **Visits** ([`track`], [`TRACK_EVERY`] seconds): whoever is in the town
//! and standing on the plot of someone else's land in their channel
//! (`house::on_plot`, `Towns::owner_of`), not in a car, is staying there
//! ([`Stay`]); a stay of `house::VISIT_SECS` is a visit, counted and told to
//! the owner, once for each visitor in `house::VISIT_AGAIN`: across their
//! screen if their app is open ([`Tell::House`]), and to their phone if it
//! is not ([`push`]). A stay of `house::RATE_SECS` offers the visitor the
//! stars, unless they have rated that land as it stands
//! (`house::land_print`), and the offer is taken back as they walk off.
//!
//! **Ratings** ([`hear`]): one to a rater and a house, the last given; the
//! owner is told as of a visit, "Someone just rated your house". **A house's
//! stats** and **the board** are worked out as they are asked for, from
//! what is held here, the names from `social`, what stands on each plot from
//! the bags (`bag::Holds`, `Towns::land`). An account deleted is left out
//! of everything as soon as `social` has forgotten it, and the database's
//! own rows go with it (`ON DELETE CASCADE`).
//!
//! **What is kept**: every visit counted (`house_visits`), every rating
//! (`house_ratings`) and every device's push token (`push_tokens`), each
//! written by a thread of this module's own after the game's loop has gone
//! by it ([`Keeper`]), and read once as the server starts ([`Kept`]).
//! Without a database, for as long as the server runs.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use postgres::Client;
use roundtown_net::house::{self, Ask as Asked, Board, Platform, Rateable, Ranked, Rating, Stats, Tell as Told};
use roundtown_net::{Ask, Spot, Tell, Venue, land};

use crate::bag::Holds;
use crate::players::{Account, Body, Everyone, Plays, Who, tell};
use crate::push::{Push, Pusher};
use crate::rooms::{Room, Towns};
use crate::social::Social;

/// How often whoever is on someone's plot is looked at, in seconds.
const TRACK_EVERY: f32 = 0.5;

/// The tables, made if they are not there yet, after `accounts`' own:
/// every visit counted, every rating, every device's push token, each gone
/// with either account it names.
pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS house_visits (
    owner BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    visitor BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS house_visits_owner ON house_visits (owner, at);
CREATE TABLE IF NOT EXISTS house_ratings (
    rater BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    owner BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    halves SMALLINT NOT NULL CHECK (halves BETWEEN 1 AND 10),
    words INTEGER NOT NULL DEFAULT 0,
    land BIGINT NOT NULL,
    at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (rater, owner),
    CHECK (rater <> owner)
);
CREATE INDEX IF NOT EXISTS house_ratings_owner ON house_ratings (owner);
CREATE TABLE IF NOT EXISTS push_tokens (
    player BIGINT PRIMARY KEY REFERENCES players (id) ON DELETE CASCADE,
    platform TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    at TIMESTAMPTZ NOT NULL DEFAULT now()
);
";

/// Seconds since 1970, to a part of a second.
fn unix_now() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |since| since.as_secs_f64())
}

// ----------------------------------------------------------------- what is held

/// A rating as it is held: its halves of a star, its words, the print of
/// the land it was given for, and when, in seconds since 1970.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Given {
    pub halves: u8,
    pub words: Vec<u8>,
    pub land: u64,
    pub at: f64,
}

/// Someone standing on the plot of someone else's land: whose, which plot of
/// which channel, since when by the server's clock, whether it has counted
/// as a visit yet, the print of the land they were offered to rate, if they
/// were, and whether they have rated it since.
#[derive(Clone, Debug, PartialEq)]
struct Stay {
    owner: u64,
    plot: (u16, usize),
    since: f64,
    visited: bool,
    offered: Option<u64>,
    rated: bool,
}

/// Everything about houses the game's loop goes by.
#[derive(Resource, Default)]
pub(crate) struct Houses {
    /// Every rating, by the house's owner and then its rater.
    ratings: HashMap<u64, HashMap<u64, Given>>,
    /// How many visits each house has had, ever.
    visits: HashMap<u64, u32>,
    /// The visits of the past week, each its house, its visitor and when,
    /// the oldest first.
    recent: VecDeque<(u64, u64, f64)>,
    /// When each visitor last counted on each house, by the two.
    last: HashMap<(u64, u64), f64>,
    /// When each owner was last pushed word of a visit.
    pushed: HashMap<u64, f64>,
    /// Each account's push token, and its phone's maker.
    tokens: HashMap<u64, (Platform, String)>,
    /// Whoever is standing on someone else's plot, by their body.
    stays: HashMap<Entity, Stay>,
    /// Offers to rate still standing for someone who has walked off the
    /// plot: whose house, until when by the server's clock, by the rater.
    offers: HashMap<u64, (u64, f64)>,
    /// The devices that have asked to be told about houses.
    watchers: HashSet<Entity>,
}

impl Houses {
    /// What the database kept, as the server starts.
    fn from_kept(kept: Kept, unix: f64) -> Self {
        let mut houses = Self::default();
        for (rater, owner, given) in kept.ratings {
            houses.ratings.entry(owner).or_default().insert(rater, given);
        }
        houses.visits = kept.visits.into_iter().collect();
        for (owner, visitor, at) in kept.recent {
            if unix - at <= house::WEEK {
                houses.recent.push_back((owner, visitor, at));
                let last = houses.last.entry((visitor, owner)).or_insert(at);
                *last = last.max(at);
            }
        }
        houses.tokens = kept.tokens.into_iter().map(|(player, platform, token)| (player, (platform, token))).collect();
        houses
    }

    /// `visitor` has stayed long enough on `owner`'s house at `unix`: a
    /// visit, counted, if they have not counted there in
    /// `house::VISIT_AGAIN`. Whether it counted.
    pub(crate) fn visit(&mut self, owner: u64, visitor: u64, unix: f64) -> bool {
        if self.last.get(&(visitor, owner)).is_some_and(|&last| unix - last < house::VISIT_AGAIN) {
            return false;
        }
        self.last.insert((visitor, owner), unix);
        *self.visits.entry(owner).or_default() += 1;
        self.recent.push_back((owner, visitor, unix));
        // What is more than a week old is of no use any more.
        while self.recent.front().is_some_and(|&(.., at)| unix - at > house::WEEK) {
            self.recent.pop_front();
        }
        if self.last.len() > 4_096 {
            self.last.retain(|_, &mut last| unix - last < house::VISIT_AGAIN);
        }
        true
    }

    /// How many visits `owner`'s house has had, ever and in the week to
    /// `unix`.
    pub(crate) fn visits_of(&self, owner: u64, unix: f64) -> (u32, u32) {
        let week = self.recent.iter().filter(|&&(house, _, at)| house == owner && unix - at <= house::WEEK).count();
        (self.visits.get(&owner).copied().unwrap_or(0), week as u32)
    }

    /// The print of `owner`'s land that `rater` rated last, if they have.
    pub(crate) fn rated(&self, rater: u64, owner: u64) -> Option<u64> {
        self.ratings.get(&owner)?.get(&rater).map(|given| given.land)
    }

    /// `rater`'s rating of `owner`'s house, in place of any before it.
    pub(crate) fn rate(&mut self, rater: u64, owner: u64, given: Given) {
        self.ratings.entry(owner).or_default().insert(rater, given);
    }

    /// The ratings of `owner`'s house by those of `known` (an account not
    /// known is deleted), the newest first.
    fn ratings_of(&self, owner: u64, known: impl Fn(u64) -> bool) -> Vec<(u64, &Given)> {
        let mut ratings: Vec<(u64, &Given)> = self
            .ratings
            .get(&owner)
            .into_iter()
            .flatten()
            .filter(|&(&rater, _)| known(rater))
            .map(|(&rater, given)| (rater, given))
            .collect();
        ratings.sort_by(|a, b| b.1.at.total_cmp(&a.1.at).then(a.0.cmp(&b.0)));
        ratings
    }

    /// Every house that has been rated, of the accounts `known`, in the
    /// board's order: each its owner and the count and the sum of its
    /// ratings.
    pub(crate) fn ranked(&self, known: impl Fn(u64) -> bool) -> Vec<(u64, u32, u32)> {
        let houses = self
            .ratings
            .iter()
            .filter(|&(&owner, _)| known(owner))
            .map(|(&owner, raters)| {
                let (count, sum) = raters
                    .iter()
                    .filter(|&(&rater, _)| known(rater))
                    .fold((0u32, 0u32), |(count, sum), (_, given)| (count + 1, sum + u32::from(given.halves)));
                (owner, count, sum)
            })
            .collect();
        house::ranked(houses)
    }

    /// A push token is `who`'s now, nobody else's.
    fn hold_token(&mut self, who: u64, platform: Platform, token: String) {
        self.tokens.retain(|&holder, (_, held)| holder == who || *held != token);
        self.tokens.insert(who, (platform, token));
    }
}

// ------------------------------------------------------------- what is kept

/// Everything kept about houses, as it is read as the server starts: every
/// rating, its rater and house; how many visits each house has had; the
/// visits of the past week; and every push token.
#[derive(Default, Debug, PartialEq)]
pub(crate) struct Kept {
    pub ratings: Vec<(u64, u64, Given)>,
    pub visits: Vec<(u64, u32)>,
    pub recent: Vec<(u64, u64, f64)>,
    pub tokens: Vec<(u64, Platform, String)>,
}

/// How a platform is written in the database.
fn platform_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Android => "android",
        Platform::Ios => "ios",
    }
}

fn platform_named(name: &str) -> Option<Platform> {
    match name {
        "android" => Some(Platform::Android),
        "ios" => Some(Platform::Ios),
        _ => None,
    }
}

/// Makes the tables and reads what they keep.
fn set_up(db: &mut Client) -> Result<Kept, postgres::Error> {
    db.batch_execute(SCHEMA)?;
    let account = |n: i64| n as u64;
    let ratings = db
        .query(
            "SELECT rater, owner, halves, words, land, extract(epoch FROM at)::float8 FROM house_ratings",
            &[],
        )?
        .iter()
        .map(|row| {
            let given = Given {
                halves: row.get::<_, i16>(2).clamp(1, 10) as u8,
                words: house::bits_to_words(row.get::<_, i32>(3) as u32),
                land: row.get::<_, i64>(4) as u64,
                at: row.get(5),
            };
            (account(row.get(0)), account(row.get(1)), given)
        })
        .collect();
    let visits = db
        .query("SELECT owner, count(*) FROM house_visits GROUP BY owner", &[])?
        .iter()
        .map(|row| (account(row.get(0)), row.get::<_, i64>(1).clamp(0, i64::from(u32::MAX)) as u32))
        .collect();
    let recent = db
        .query(
            "SELECT owner, visitor, extract(epoch FROM at)::float8 FROM house_visits WHERE at > now() - interval '7 days' ORDER BY at",
            &[],
        )?
        .iter()
        .map(|row| (account(row.get(0)), account(row.get(1)), row.get(2)))
        .collect();
    let tokens = db
        .query("SELECT player, platform, token FROM push_tokens", &[])?
        .iter()
        .filter_map(|row| Some((account(row.get(0)), platform_named(row.get(1))?, row.get(2))))
        .collect();
    Ok(Kept {
        ratings,
        visits,
        recent,
        tokens,
    })
}

/// Something to be written down, the game's loop having gone by it.
#[derive(Clone, Debug, PartialEq)]
enum Job {
    Visit { owner: u64, visitor: u64 },
    Rate { rater: u64, owner: u64, halves: u8, words: Vec<u8>, land: u64 },
    Token { player: u64, platform: Platform, token: String },
    Forget(String),
}

/// Where what is to be kept goes: to the database's thread, or nowhere.
#[derive(Resource)]
struct Keeper(Option<Sender<Job>>);

impl Keeper {
    fn keep(&self, job: Job) {
        if let Some(jobs) = &self.0 {
            let _ = jobs.send(job);
        }
    }
}

/// Writes `job` down. What names an account deleted meanwhile is left
/// unwritten, rather than refused by the table.
fn write(db: &mut Client, job: &Job) -> Result<(), postgres::Error> {
    let row = |id: u64| id as i64;
    const BOTH: &str = "EXISTS (SELECT 1 FROM players WHERE id = $1) AND EXISTS (SELECT 1 FROM players WHERE id = $2)";
    match job {
        Job::Visit { owner, visitor } => {
            db.execute(
                &format!("INSERT INTO house_visits (owner, visitor) SELECT $1::bigint, $2::bigint WHERE {BOTH}"),
                &[&row(*owner), &row(*visitor)],
            )?;
        }
        Job::Rate {
            rater,
            owner,
            halves,
            words,
            land,
        } => {
            db.execute(
                &format!(
                    "INSERT INTO house_ratings (rater, owner, halves, words, land) \
                     SELECT $1::bigint, $2::bigint, $3::smallint, $4::integer, $5::bigint WHERE {BOTH} \
                     ON CONFLICT (rater, owner) DO UPDATE SET halves = $3, words = $4, land = $5, at = now()"
                ),
                &[&row(*rater), &row(*owner), &i16::from(*halves), &(house::words_to_bits(words) as i32), &(*land as i64)],
            )?;
        }
        Job::Token { player, platform, token } => {
            let mut transaction = db.transaction()?;
            transaction.execute("DELETE FROM push_tokens WHERE token = $1 AND player <> $2", &[token, &row(*player)])?;
            transaction.execute(
                "INSERT INTO push_tokens (player, platform, token) SELECT $1::bigint, $2::text, $3::text \
                 WHERE EXISTS (SELECT 1 FROM players WHERE id = $1) \
                 ON CONFLICT (player) DO UPDATE SET platform = $2, token = $3, at = now()",
                &[&row(*player), &platform_name(*platform), token],
            )?;
            transaction.commit()?;
        }
        Job::Forget(token) => {
            db.execute("DELETE FROM push_tokens WHERE token = $1", &[token])?;
        }
    }
    Ok(())
}

/// The database's thread for houses: writes each job as it comes,
/// connecting again whenever the connection has gone.
fn work(url: &str, jobs: &Receiver<Job>) {
    let mut db: Option<Client> = None;
    for job in jobs {
        for _ in 0..3 {
            if db.is_none() {
                match crate::accounts::connect(url) {
                    Ok(client) => db = Some(client),
                    Err(error) => {
                        error!("houses could not reach the database: {error}");
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                }
            }
            let Some(client) = db.as_mut() else {
                continue;
            };
            match write(client, &job) {
                Ok(()) => break,
                Err(error) => {
                    error!("houses, the database: {error}");
                    if client.is_closed() {
                        db = None;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

pub(crate) fn plugin(app: &mut App, db: Option<String>) {
    let unix = unix_now();
    let (houses, keeper) = match db {
        Some(url) => {
            let kept = match crate::accounts::connect(&url).and_then(|mut db| set_up(&mut db)) {
                Ok(kept) => kept,
                // As with the accounts: better to stop than to lose what is
                // kept.
                Err(error) => panic!("could not set up the houses' tables: {error}"),
            };
            info!(
                "{} ratings, {} houses visited and {} push tokens kept",
                kept.ratings.len(),
                kept.visits.len(),
                kept.tokens.len()
            );
            let (jobs, to_worker) = channel();
            std::thread::Builder::new()
                .name("houses".into())
                .spawn(move || work(&url, &to_worker))
                .expect("a thread for the houses' database");
            (Houses::from_kept(kept, unix), Keeper(Some(jobs)))
        }
        None => (Houses::default(), Keeper(None)),
    };
    app.insert_resource(houses)
        .insert_resource(keeper)
        .add_observer(forget)
        .add_systems(
            PreUpdate,
            hear.after(crate::players::PlayerSystems).run_if(in_state(ServerState::Running)),
        )
        .add_systems(Update, (track, forget_tokens).run_if(in_state(ServerState::Running)));
}

/// A device has gone: it is told nothing more.
fn forget(remove: On<Remove, ConnectedClient>, mut houses: ResMut<Houses>) {
    houses.watchers.remove(&remove.entity);
}

/// Tells the device `client` about houses, if it asked to be told.
fn tell_house(tells: &mut MessageWriter<ToClients<Tell>>, houses: &Houses, client: Entity, told: Told) {
    if houses.watchers.contains(&client) {
        tell(tells, client, Tell::House(told));
    }
}

/// What stands on `owner`'s plot, and its paint: their bag's, while they
/// are here, and as last known otherwise.
fn land_of<'a>(owner: u64, everyone: &Everyone, holds: &'a Query<&Holds>, towns: &'a Towns) -> (&'a [roundtown_net::bag::Standing], &'a [roundtown_net::bag::Paint]) {
    if let Some(held) = everyone.0.get(&Who(owner)).and_then(|&body| holds.get(body).ok()) {
        return (&held.bag.stands, &held.paints.0);
    }
    towns.land(owner).map_or((&[], &[]), |lot| (lot.stands.as_slice(), lot.paints.as_slice()))
}

/// Tells `owner` of something about their house: across their screen if
/// their app is open, and connected; to their phone if not, if it has said
/// where to push.
#[allow(clippy::too_many_arguments)]
fn tell_owner(
    owner: u64,
    told: Told,
    push: (&str, &'static str),
    everyone: &Everyone,
    bodies: &Query<(Entity, &Body, &Spot, &Venue, &Account)>,
    houses: &Houses,
    pusher: &Pusher,
    tells: &mut MessageWriter<ToClients<Tell>>,
) {
    let device = everyone.0.get(&Who(owner)).and_then(|&body| bodies.get(body).ok()).and_then(|(_, body, ..)| body.client);
    match device {
        Some(client) => tell_house(tells, houses, client, told),
        None => {
            if let Some((platform, token)) = houses.tokens.get(&owner) {
                pusher.send(Push {
                    platform: *platform,
                    token: token.clone(),
                    body: push.0.to_string(),
                    kind: push.1,
                });
            }
        }
    }
}

/// What devices ask about houses.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<&Plays>,
    bodies: Query<(Entity, &Body, &Spot, &Venue, &Account)>,
    holds: Query<&Holds>,
    everyone: Res<Everyone>,
    towns: Res<Towns>,
    social: Res<Social>,
    time: Res<Time>,
    keeper: Res<Keeper>,
    pusher: Res<Pusher>,
    mut houses: ResMut<Houses>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    let now = time.elapsed_secs_f64();
    for ask in asks.read() {
        let Ask::House(asked) = &ask.message else {
            continue;
        };
        let Some(client) = ask.client_id.entity() else {
            continue;
        };
        let Ok(&Plays(body)) = clients.get(client) else {
            continue;
        };
        let Ok((_, me, ..)) = bodies.get(body) else {
            continue;
        };
        let who = me.who.0;
        let known = |id: u64| social.person(id).is_some();
        match asked {
            Asked::Watch => {
                houses.watchers.insert(client);
                // Already on a plot they were offered: offered again.
                if let Some(stay) = houses.stays.get(&body)
                    && stay.offered.is_some()
                    && !stay.rated
                {
                    let rateable = rateable(stay.owner, &social, &towns);
                    tell_house(&mut tells, &houses, client, Told::MayRate(Some(rateable)));
                }
            }
            Asked::Push { platform, token } => {
                if !house::token_ok(token) {
                    continue;
                }
                if houses.tokens.get(&who).is_some_and(|(was, held)| was == platform && held == token) {
                    continue;
                }
                houses.hold_token(who, *platform, token.clone());
                keeper.keep(Job::Token {
                    player: who,
                    platform: *platform,
                    token: token.clone(),
                });
                debug!("{who} pushes to {:?}", platform);
            }
            Asked::Rate { owner, halves, words } => {
                let (owner, halves) = (*owner, *halves);
                let Some(words) = house::chosen(words) else {
                    continue;
                };
                if owner == who || !house::halves_ok(halves) || !known(owner) {
                    continue;
                }
                // Only from someone offered it: on the plot now, or a moment
                // ago.
                let on_it = houses.stays.get(&body).is_some_and(|stay| stay.owner == owner && stay.offered.is_some() && !stay.rated);
                let lately = houses.offers.get(&who).is_some_and(|&(house, until)| house == owner && now <= until);
                if !(on_it || lately) {
                    debug!("{who} rated {owner}'s house, not offered to");
                    continue;
                }
                let (stands, paints) = land_of(owner, &everyone, &holds, &towns);
                let print = house::land_print(stands, paints);
                if houses.rated(who, owner) == Some(print) {
                    continue;
                }
                let given = Given {
                    halves,
                    words: words.clone(),
                    land: print,
                    at: unix_now(),
                };
                houses.rate(who, owner, given);
                keeper.keep(Job::Rate {
                    rater: who,
                    owner,
                    halves,
                    words,
                    land: print,
                });
                if let Some(stay) = houses.stays.get_mut(&body) {
                    stay.rated = true;
                }
                houses.offers.remove(&who);
                tell_house(&mut tells, &houses, client, Told::MayRate(None));
                tell_house(&mut tells, &houses, client, Told::Thanks);
                tell_owner(owner, Told::Rated, (house::RATED, "rated"), &everyone, &bodies, &houses, &pusher, &mut tells);
                info!("{who} rated {owner}'s house {halves} halves");
            }
            Asked::Stats(whose) => {
                let owner = whose.unwrap_or(who);
                if let Some(stats) = stats_of(owner, &houses, &social, land_of(owner, &everyone, &holds, &towns)) {
                    tell_house(&mut tells, &houses, client, Told::Stats(stats));
                }
            }
            Asked::Board => {
                let board = board_for(who, &houses, &social);
                tell_house(&mut tells, &houses, client, Told::Board(board));
            }
        }
    }
}

/// The house of `owner` as someone may rate it: its owner, and the name
/// on its sign.
fn rateable(owner: u64, social: &Social, towns: &Towns) -> Rateable {
    let name = social
        .person(owner)
        .map(|person| person.name.clone())
        .or_else(|| towns.land(owner).map(|lot| lot.name.clone()))
        .unwrap_or_default();
    Rateable { owner, name }
}

/// The stats of `owner`'s house, its land being `land`: none for an
/// account not known.
fn stats_of(owner: u64, houses: &Houses, social: &Social, land: (&[roundtown_net::bag::Standing], &[roundtown_net::bag::Paint])) -> Option<Stats> {
    let person = social.person(owner)?;
    let known = |id: u64| social.person(id).is_some();
    let (visits, visits_week) = houses.visits_of(owner, unix_now());
    let rated = houses.ratings_of(owner, known);
    let sum = rated.iter().map(|(_, given)| u32::from(given.halves)).sum();
    let ratings = rated
        .iter()
        .take(house::LISTED_MOST)
        .filter_map(|&(rater, given)| {
            let by = social.person(rater)?;
            Some(Rating {
                who: rater,
                name: by.name.clone(),
                tag: by.tag,
                halves: given.halves,
                words: given.words.clone(),
            })
        })
        .collect();
    Some(Stats {
        who: owner,
        name: person.name.clone(),
        tag: person.tag,
        worth: land::worth(land.0),
        things: land.0.len() as u32,
        visits,
        visits_week,
        count: rated.len() as u32,
        sum,
        ratings,
    })
}

/// The board as `asker` is shown it: the top `house::BOARD`, and their own
/// place, if their house is rated and further down.
fn board_for(asker: u64, houses: &Houses, social: &Social) -> Board {
    let ranked = houses.ranked(|id| social.person(id).is_some());
    let row = |place: usize, &(who, count, sum): &(u64, u32, u32)| -> Option<Ranked> {
        let person = social.person(who)?;
        Some(Ranked {
            place: place as u32 + 1,
            who,
            name: person.name.clone(),
            tag: person.tag,
            count,
            sum,
        })
    };
    let top = ranked.iter().take(house::BOARD).enumerate().filter_map(|(place, house)| row(place, house)).collect();
    let mine = ranked
        .iter()
        .enumerate()
        .skip(house::BOARD)
        .find(|(_, house)| house.0 == asker)
        .and_then(|(place, house)| row(place, house));
    Board { top, mine }
}

/// Looks at who is on whose plot, [`TRACK_EVERY`] seconds: a stay long
/// enough is a visit, told to the owner, and longer, an offer of the stars
/// to the visitor, which is taken back as they walk off.
#[allow(clippy::too_many_arguments)]
fn track(
    time: Res<Time>,
    mut since: Local<f32>,
    bodies: Query<(Entity, &Body, &Spot, &Venue, &Account)>,
    holds: Query<&Holds>,
    everyone: Res<Everyone>,
    towns: Res<Towns>,
    social: Res<Social>,
    keeper: Res<Keeper>,
    pusher: Res<Pusher>,
    mut houses: ResMut<Houses>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    *since += time.delta_secs();
    if *since < TRACK_EVERY {
        return;
    }
    *since = 0.0;
    let now = time.elapsed_secs_f64();
    let unix = unix_now();
    // Its fields borrowed one by one, the stays apart from the ratings.
    let houses = &mut *houses;
    let mut staying = HashSet::new();
    // What came of it, to be done once every stay has been looked at.
    let mut visited: Vec<(u64, u64, String)> = Vec::new();
    let mut offered: Vec<(Entity, Entity, u64)> = Vec::new();
    let mut ended: Vec<(Entity, Stay)> = Vec::new();
    for (entity, body, spot, venue, account) in &bodies {
        // People, here, on foot in the town.
        let Some(client) = body.client else {
            continue;
        };
        let Room::Town(channel) = body.room else {
            continue;
        };
        if *venue != Venue::Town || spot.aboard.is_some() || body.driving {
            continue;
        }
        let at = spot.translation();
        let Some(plot) = house::plot_under(Vec2::new(at.x, at.z)) else {
            continue;
        };
        let visitor = body.who.0;
        let Some(owner) = towns.owner_of(channel, plot).filter(|&owner| owner != visitor) else {
            continue;
        };
        staying.insert(entity);
        let fresh = Stay {
            owner,
            plot: (channel, plot),
            since: now,
            visited: false,
            offered: None,
            rated: false,
        };
        let stay = houses.stays.entry(entity).or_insert_with(|| fresh.clone());
        if stay.owner != owner || stay.plot != (channel, plot) {
            let was = std::mem::replace(stay, fresh);
            ended.push((entity, was));
            continue;
        }
        let stood = (now - stay.since) as f32;
        if stood >= house::VISIT_SECS && !stay.visited {
            stay.visited = true;
            visited.push((owner, visitor, account.0.name.clone()));
        }
        if stood >= house::RATE_SECS && stay.offered.is_none() {
            let (stands, paints) = land_of(owner, &everyone, &holds, &towns);
            let print = house::land_print(stands, paints);
            stay.offered = Some(print);
            let rated = houses.ratings.get(&owner).and_then(|raters| raters.get(&visitor)).map(|given| given.land);
            if rated == Some(print) {
                // Rated as it stands: nothing to offer this stay.
                stay.rated = true;
            } else {
                offered.push((entity, client, owner));
            }
        }
    }
    // Stays that are over: walked off, gone into a game, gone.
    let over: Vec<Entity> = houses.stays.keys().copied().filter(|body| !staying.contains(body)).collect();
    for body in over {
        if let Some(stay) = houses.stays.remove(&body) {
            ended.push((body, stay));
        }
    }
    for (body, stay) in ended {
        if stay.offered.is_none() || stay.rated {
            continue;
        }
        let Ok((_, held, ..)) = bodies.get(body) else {
            continue;
        };
        houses.offers.insert(held.who.0, (stay.owner, now + house::OFFER_STANDS));
        if let Some(client) = held.client {
            tell_house(&mut tells, houses, client, Told::MayRate(None));
        }
    }
    for (_, client, owner) in offered {
        let rateable = rateable(owner, &social, &towns);
        tell_house(&mut tells, houses, client, Told::MayRate(Some(rateable)));
    }
    for (owner, visitor, name) in visited {
        if !houses.visit(owner, visitor, unix) {
            continue;
        }
        keeper.keep(Job::Visit { owner, visitor });
        info!("{visitor} is visiting {owner}'s house");
        // Their phone buzzes no oftener than so often for visits.
        let pushed = houses.pushed.get(&owner).is_some_and(|&last| unix - last < house::VISIT_PUSH_GAP);
        let words = house::VISITING.replace("{name}", &name);
        let here = everyone.0.get(&Who(owner)).and_then(|&body| bodies.get(body).ok()).is_some_and(|(_, body, ..)| body.client.is_some());
        if here || !pushed {
            if !here {
                houses.pushed.insert(owner, unix);
            }
            tell_owner(owner, Told::Visiting { name }, (&words, "visit"), &everyone, &bodies, houses, &pusher, &mut tells);
        }
    }
    houses.offers.retain(|_, &mut (_, until)| now <= until);
}

/// Forgets every push token a service has said is nobody's.
fn forget_tokens(pusher: Res<Pusher>, keeper: Res<Keeper>, mut houses: ResMut<Houses>) {
    for token in pusher.gone() {
        houses.tokens.retain(|_, (_, held)| *held != token);
        keeper.keep(Job::Forget(token));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn given(halves: u8, at: f64) -> Given {
        Given {
            halves,
            words: vec![1, 4],
            land: 77,
            at,
        }
    }

    #[test]
    fn a_visitor_counts_once_in_five_minutes_and_the_week_goes_back_seven_days() {
        let mut houses = Houses::default();
        let day = 86_400.0;
        assert!(houses.visit(1, 2, 1_000.0));
        assert!(!houses.visit(1, 2, 1_000.0 + house::VISIT_AGAIN - 1.0), "the same five minutes");
        assert!(houses.visit(1, 2, 1_000.0 + house::VISIT_AGAIN), "five minutes on");
        assert!(houses.visit(1, 3, 2_000.0), "someone else");
        assert!(houses.visit(4, 2, 2_000.0), "another house");
        assert_eq!(houses.visits_of(1, 3_000.0), (3, 3));
        // Eight days on, the week has none of them; ever has them all.
        assert_eq!(houses.visits_of(1, 8.0 * day), (3, 0));
        assert!(houses.visit(1, 2, 8.0 * day));
        assert_eq!(houses.visits_of(1, 8.0 * day), (4, 1));
        assert!(houses.recent.iter().all(|&(.., at)| 8.0 * day - at <= house::WEEK), "the old let go of");
    }

    #[test]
    fn a_rating_given_again_takes_the_place_of_the_one_before_and_the_board_leaves_out_the_deleted() {
        let mut houses = Houses::default();
        houses.rate(10, 1, given(10, 1.0));
        houses.rate(11, 1, given(8, 2.0));
        houses.rate(10, 2, given(8, 3.0));
        assert_eq!(houses.rated(10, 1), Some(77));
        assert_eq!(houses.rated(12, 1), None);
        // Again, for land changed since.
        houses.rate(10, 1, Given { land: 78, ..given(4, 5.0) });
        assert_eq!(houses.rated(10, 1), Some(78));
        let all = |_: u64| true;
        // Four stars once, weighted toward three (3.25), beats two and three
        // (3.0).
        assert_eq!(houses.ranked(all), vec![(2, 1, 8), (1, 2, 12)]);
        // The newest first.
        let listed: Vec<u64> = houses.ratings_of(1, all).iter().map(|(rater, _)| *rater).collect();
        assert_eq!(listed, vec![10, 11]);
        // An account deleted is nobody's rater, and has no house.
        let without = |id: u64| id != 11 && id != 2;
        assert_eq!(houses.ranked(without), vec![(1, 1, 4)]);
    }

    #[test]
    fn a_token_is_one_account_s_at_a_time() {
        let mut houses = Houses::default();
        houses.hold_token(1, Platform::Android, "abc".into());
        houses.hold_token(2, Platform::Android, "abc".into());
        assert!(!houses.tokens.contains_key(&1), "the phone is the other account's now");
        houses.hold_token(2, Platform::Ios, "def".into());
        assert_eq!(houses.tokens.get(&2), Some(&(Platform::Ios, "def".into())));
    }

    /// Everything this module writes and reads, against a copy of the
    /// server's own database (`RT_TEST_DB`, a name ending in `_scratch`, as
    /// `accounts`' tests take): its tables made beside the accounts' as the
    /// server starts, two of its accounts visiting, rating (twice) and
    /// pushing to one phone, and all of it read back as the next start
    /// would. Run on the VM before the deploy that brings the tables
    /// (`SERVER.md`, "The database").
    #[test]
    #[ignore = "needs a copy of the database: RT_TEST_DB"]
    fn the_database_keeps_visits_ratings_and_tokens() {
        let url = std::env::var("RT_TEST_DB").expect("RT_TEST_DB: a copy of the database");
        let mut db = crate::accounts::connect(&url).expect("the scratch database");
        let name: String = db.query_one("SELECT current_database()", &[]).expect("its name").get(0);
        assert!(name.ends_with("_scratch"), "{name} is not a scratch database");
        // As the server starts: the accounts' tables first, then these.
        crate::accounts::set_up(&mut db).expect("the accounts' tables");
        let before = set_up(&mut db).expect("the houses' tables");
        let ids: Vec<u64> = db
            .query("SELECT id FROM players ORDER BY id LIMIT 2", &[])
            .expect("accounts")
            .iter()
            .map(|row| row.get::<_, i64>(0) as u64)
            .collect();
        let [owner, visitor] = ids[..] else {
            panic!("the copy has fewer than two accounts");
        };
        let jobs = [
            Job::Visit { owner, visitor },
            Job::Rate {
                rater: visitor,
                owner,
                halves: 6,
                words: vec![1, 2],
                land: u64::MAX - 3,
            },
            // Rated again, for the land changed: in the first's place.
            Job::Rate {
                rater: visitor,
                owner,
                halves: 9,
                words: vec![0, 8, 15],
                land: 42,
            },
            Job::Token {
                player: owner,
                platform: Platform::Android,
                token: "scratch-token-1".into(),
            },
            // The same phone, the other account's now.
            Job::Token {
                player: visitor,
                platform: Platform::Android,
                token: "scratch-token-1".into(),
            },
            Job::Token {
                player: owner,
                platform: Platform::Ios,
                token: "scratch-token-2".into(),
            },
            Job::Forget("scratch-token-2".into()),
            // An account that is not there is written nowhere, and refused
            // by nothing.
            Job::Visit { owner, visitor: u64::MAX / 2 },
        ];
        for job in &jobs {
            write(&mut db, job).unwrap_or_else(|error| panic!("{job:?}: {error}"));
        }
        let after = set_up(&mut db).expect("read back");
        let visits = |kept: &Kept| kept.visits.iter().find(|(house, _)| *house == owner).map_or(0, |(_, count)| *count);
        assert_eq!(visits(&after), visits(&before) + 1);
        assert!(after.recent.iter().any(|&(house, who, _)| house == owner && who == visitor));
        let rated: Vec<&(u64, u64, Given)> = after.ratings.iter().filter(|(rater, house, _)| *rater == visitor && *house == owner).collect();
        assert_eq!(rated.len(), 1, "one rating to a rater and a house");
        let given = &rated[0].2;
        assert_eq!((given.halves, given.words.clone(), given.land), (9, vec![0, 8, 15], 42));
        let tokens: Vec<&(u64, Platform, String)> = after.tokens.iter().filter(|(_, _, token)| token.starts_with("scratch-token")).collect();
        assert_eq!(tokens, [&(visitor, Platform::Android, "scratch-token-1".to_string())]);
        // And the server's own copy of it all, as it starts.
        let houses = Houses::from_kept(after, unix_now());
        assert_eq!(houses.rated(visitor, owner), Some(42));
        assert!(houses.ranked(|_| true).iter().any(|&(house, ..)| house == owner));
        // Cleaned up: the copy is as it was but for its new tables.
        for sql in [
            "DELETE FROM house_ratings WHERE land = 42",
            "DELETE FROM push_tokens WHERE token LIKE 'scratch-token%'",
        ] {
            db.execute(sql, &[]).expect(sql);
        }
    }

    #[test]
    fn what_was_kept_is_held_again_as_the_server_starts() {
        let unix = 100.0 * 86_400.0;
        let kept = Kept {
            ratings: vec![(10, 1, given(9, unix - 50.0))],
            visits: vec![(1, 40)],
            recent: vec![(1, 2, unix - 2.0 * 86_400.0), (1, 3, unix - 60.0)],
            tokens: vec![(1, Platform::Ios, "aa".into())],
        };
        let mut houses = Houses::from_kept(kept, unix);
        assert_eq!(houses.visits_of(1, unix), (40, 2));
        assert_eq!(houses.rated(10, 1), Some(77));
        assert!(houses.tokens.contains_key(&1));
        // Someone who counted a minute ago does not count again yet.
        assert!(!houses.visit(1, 3, unix));
        assert!(houses.visit(1, 2, unix));
    }
}
