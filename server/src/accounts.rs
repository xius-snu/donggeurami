//! Accounts: who each player is from one connection to the next, and what
//! they have (`MULTIPLAYER.md`, phase 2).
//!
//! With a database (`RT_DB`), a player is a row of `players`, found by the
//! device they play on (`devices`, which keeps only a hash of each device's
//! secret), and their coins change only in one transaction with a line of
//! `coin_changes` saying how much and why. The login (`login`) makes the row
//! the first time a device asks to play, and hands it a token naming the
//! account; the game server loads the row when the device connects with it.
//!
//! Without a database, for trying the server out, everyone is made up afresh
//! on connecting, with [`STARTING_BALANCE`] coins kept in memory.
//!
//! The database is only ever reached from a thread of its own ([`work`]): the
//! game's loop sends it what to do and picks up the answers as they come, so
//! that a slow query holds up nobody's walking.
//!
//! **Friends, and what goes with them** (`social`, since 2026-11-13): every
//! account has a tag, the number after its name (`players.tag`), and what it
//! has earned toward its level (`players.xp`); who is whose friend
//! (`friends`), who has asked whom (`friend_requests`) and what has been
//! written and not yet seen off (`messages`) each have a table. The game's
//! loop holds all of that in memory, read once as the server starts
//! ([`Snapshot`]), and goes by its own copy, so that nothing about friends
//! waits on a query; every change it makes is written here after the fact
//! ([`Keep`]), in the order it was made. That is right for the few thousand
//! accounts one server holds; with many more, friends would be asked of the
//! database as accounts are.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use bevy::prelude::*;
use postgres::{Client, NoTls};
use roundtown_net::rules::STARTING_BALANCE;
use roundtown_net::social::{TAG_FIRST, TAG_LAST};

/// The tables, made if they are not there yet. Run every time the server
/// starts: each statement leaves what is already there alone.
pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS players (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    look TEXT NOT NULL,
    coins BIGINT NOT NULL CHECK (coins >= 0),
    created TIMESTAMPTZ NOT NULL DEFAULT now(),
    seen TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS devices (
    secret BYTEA PRIMARY KEY,
    player BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    created TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS devices_player ON devices (player);
CREATE TABLE IF NOT EXISTS coin_changes (
    id BIGSERIAL PRIMARY KEY,
    player BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    amount BIGINT NOT NULL,
    reason TEXT NOT NULL,
    game BIGINT,
    at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS coin_changes_player ON coin_changes (player);
ALTER TABLE players ADD COLUMN IF NOT EXISTS tag SMALLINT CHECK (tag BETWEEN 1000 AND 9999);
ALTER TABLE players ADD COLUMN IF NOT EXISTS xp BIGINT NOT NULL DEFAULT 0 CHECK (xp >= 0);
CREATE TABLE IF NOT EXISTS friends (
    a BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    b BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    since TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (a, b),
    CHECK (a < b)
);
CREATE INDEX IF NOT EXISTS friends_b ON friends (b);
CREATE TABLE IF NOT EXISTS friend_requests (
    sender BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    receiver BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (sender, receiver),
    CHECK (sender <> receiver)
);
CREATE INDEX IF NOT EXISTS friend_requests_receiver ON friend_requests (receiver);
CREATE TABLE IF NOT EXISTS messages (
    id BIGINT PRIMARY KEY,
    sender BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    receiver BIGINT NOT NULL REFERENCES players (id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS messages_receiver ON messages (receiver);
";

/// What is made once every account has its tag ([`give_tags`]): no two
/// accounts with both the same name, whatever its capitals, and the same
/// tag. The server's own copy of who is called what keeps to that
/// (`social`); this is what holds the login to it as well, and anything
/// that ever writes the table by hand.
pub(crate) const SCHEMA_AFTER: &str = "
CREATE UNIQUE INDEX IF NOT EXISTS players_name_tag ON players (lower(name), tag);
";

/// How many times a job is tried before it is given up, with the connection
/// made again between.
const TRIES: u32 = 3;

/// Sets accounts up, and gives back everything kept about who is whose
/// friend ([`Snapshot`]): nothing, without a database.
pub(crate) fn plugin(app: &mut App, db: Option<String>) -> Snapshot {
    let mut snapshot = Snapshot::default();
    let bank = match db {
        Some(url) => {
            match connect(&url).and_then(|mut db| set_up(&mut db)) {
                Ok(read) => snapshot = read,
                // Without the tables nothing can be kept: better to stop here,
                // and have systemd say so, than to take anyone's coins.
                Err(error) => panic!("could not set up the database: {error}"),
            }
            let (jobs, to_worker) = channel();
            let (from_worker, done) = channel();
            std::thread::Builder::new()
                .name("accounts".into())
                .spawn(move || work(&url, &to_worker, &from_worker))
                .expect("a thread for the database");
            info!("accounts are kept in the database");
            Bank::Database {
                jobs,
                done: std::sync::Mutex::new(done),
            }
        }
        None => {
            warn!("no RT_DB: everyone is made up afresh, and coins are kept in memory");
            Bank::Memory
        }
    };
    app.insert_resource(bank)
        .add_message::<Loaded>()
        .add_message::<Charged>()
        .add_message::<Credited>()
        .add_systems(PreUpdate, answers);
    snapshot
}

/// Makes the tables, gives every account that has none its tag, and reads
/// what is kept about friends: as the server starts, before anyone is let
/// in.
pub(crate) fn set_up(db: &mut Client) -> Result<Snapshot, postgres::Error> {
    db.batch_execute(SCHEMA)?;
    let given = give_tags(db)?;
    if given > 0 {
        info!("{given} accounts were given their tags");
    }
    db.batch_execute(SCHEMA_AFTER)?;
    let snapshot = Snapshot::read(db)?;
    info!(
        "{} accounts, {} friendships, {} requests and {} messages waiting",
        snapshot.people.len(),
        snapshot.friends.len(),
        snapshot.requests.len(),
        snapshot.letters.len()
    );
    Ok(snapshot)
}

/// Gives every account that has no tag one, at random, that nobody of its
/// name has: every account from before there were tags (2026-11-13), the
/// first time the server starts after. How many were given one.
fn give_tags(db: &mut Client) -> Result<usize, postgres::Error> {
    let without = db.query("SELECT id, lower(name) FROM players WHERE tag IS NULL ORDER BY id", &[])?;
    if without.is_empty() {
        return Ok(0);
    }
    let mut taken: HashMap<String, HashSet<u16>> = HashMap::new();
    for row in db.query("SELECT lower(name), tag FROM players WHERE tag IS NOT NULL", &[])? {
        let tag: i16 = row.get(1);
        taken.entry(row.get(0)).or_default().insert(tag as u16);
    }
    let mut transaction = db.transaction()?;
    for row in &without {
        let (id, name): (i64, String) = (row.get(0), row.get(1));
        let of_name = taken.entry(name).or_default();
        let tag = free_tag(of_name, &mut || fastrand::u16(TAG_FIRST..=TAG_LAST));
        of_name.insert(tag);
        transaction.execute("UPDATE players SET tag = $2 WHERE id = $1", &[&id, &(tag as i16)])?;
    }
    transaction.commit()?;
    Ok(without.len())
}

/// A tag that is not among `taken`, as `throw` throws them: a few throws,
/// and then the first free one from where the last landed. With all of
/// them taken, [`TAG_FIRST`], which nine thousand accounts of one name
/// have come to.
pub(crate) fn free_tag(taken: &HashSet<u16>, throw: &mut impl FnMut() -> u16) -> u16 {
    let mut tag = TAG_FIRST;
    for _ in 0..32 {
        tag = throw().clamp(TAG_FIRST, TAG_LAST);
        if !taken.contains(&tag) {
            return tag;
        }
    }
    let all = TAG_LAST - TAG_FIRST + 1;
    (0..all)
        .map(|step| TAG_FIRST + (tag - TAG_FIRST + step) % all)
        .find(|tag| !taken.contains(tag))
        .unwrap_or(TAG_FIRST)
}

/// Everything kept about who is whose friend, as it is read when the server
/// starts: every account's name, tag, what it has earned and when it was
/// last here; every pair of friends; every request not yet answered, oldest
/// first; and every message not yet seen off, oldest first.
#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct Snapshot {
    pub people: Vec<Kept>,
    pub friends: Vec<(u64, u64)>,
    /// Who asked, and whom.
    pub requests: Vec<(u64, u64)>,
    pub letters: Vec<Letter>,
}

/// One account, as far as its friends are concerned.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Kept {
    pub id: u64,
    pub name: String,
    pub tag: u16,
    pub xp: u32,
    /// When it was last here, in seconds since 1970.
    pub seen: u64,
}

/// One message kept for whoever it was written to.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Letter {
    pub id: u64,
    pub from: u64,
    pub to: u64,
    pub text: String,
}

impl Snapshot {
    fn read(db: &mut Client) -> Result<Self, postgres::Error> {
        // An account is the table's own number for it, read back just as
        // it is written wherever one is asked for (`id as i64`), whatever
        // its sign. The table counts accounts up from 1; a server run
        // without the login takes a device's own bytes for its account
        // (`players::who`), which is any number at all, and read as nothing
        // under nought, every such account past the middle was one account.
        let account = |n: i64| n as u64;
        let whole = |n: i64| n.max(0) as u64;
        let people = db
            .query(
                "SELECT id, name, tag, xp, extract(epoch FROM seen)::bigint FROM players ORDER BY id",
                &[],
            )?
            .iter()
            .map(|row| Kept {
                id: account(row.get(0)),
                name: row.get(1),
                tag: row.get::<_, Option<i16>>(2).map_or(0, |tag| tag as u16),
                xp: row.get::<_, i64>(3).clamp(0, i64::from(u32::MAX)) as u32,
                seen: whole(row.get(4)),
            })
            .collect();
        let pairs = |db: &mut Client, sql: &str| -> Result<Vec<(u64, u64)>, postgres::Error> {
            Ok(db
                .query(sql, &[])?
                .iter()
                .map(|row| (account(row.get(0)), account(row.get(1))))
                .collect())
        };
        let friends = pairs(db, "SELECT a, b FROM friends ORDER BY a, b")?;
        let requests = pairs(db, "SELECT sender, receiver FROM friend_requests ORDER BY at, sender, receiver")?;
        let letters = db
            .query("SELECT id, sender, receiver, body FROM messages ORDER BY id", &[])?
            .iter()
            .map(|row| Letter {
                id: whole(row.get(0)),
                from: account(row.get(1)),
                to: account(row.get(2)),
                text: row.get(3),
            })
            .collect();
        Ok(Self {
            people,
            friends,
            requests,
            letters,
        })
    }
}

/// Something about friends to be written down after the fact, the game's
/// loop having gone by its own copy already (`social`). Nothing comes back.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Keep {
    /// This account's tag, given it here rather than by the login.
    Tag { id: u64, tag: u16 },
    /// This account was here until now.
    Seen(u64),
    /// This account has earned this much more toward its level.
    Xp { id: u64, amount: u32 },
    /// These two are friends now, or no longer.
    Friends { a: u64, b: u64 },
    Unfriend { a: u64, b: u64 },
    /// `from` has asked `to` to be friends, or that is answered or taken
    /// back.
    Request { from: u64, to: u64 },
    Unrequest { from: u64, to: u64 },
    /// A message written, or seen off.
    Letter(Letter),
    Unletter(u64),
}

/// Where accounts are kept.
#[derive(Resource)]
pub(crate) enum Bank {
    /// Nowhere: made up on connecting, coins in memory.
    Memory,
    /// In the database, by way of its thread.
    Database {
        jobs: Sender<Job>,
        done: std::sync::Mutex<Receiver<Done>>,
    },
}

impl Bank {
    pub(crate) fn kept(&self) -> bool {
        matches!(self, Self::Database { .. })
    }

    /// Asks for the account `id`; the answer comes as [`Loaded`].
    pub(crate) fn load(&self, id: u64) {
        if let Self::Database { jobs, .. } = self {
            let _ = jobs.send(Job::Load(id));
        }
    }

    /// Takes `amount` from the account `id` if it has that much, saying why;
    /// the answer comes as [`Charged`].
    pub(crate) fn charge(&self, id: u64, amount: u32, why: &'static str, game: Option<u32>) {
        if let Self::Database { jobs, .. } = self {
            let _ = jobs.send(Job::Charge {
                id,
                amount,
                why,
                game,
            });
        }
    }

    /// Gives the account `id` `amount` more, saying why; the answer comes as
    /// [`Credited`].
    pub(crate) fn credit(&self, id: u64, amount: u32, why: &'static str, game: Option<u32>) {
        if let Self::Database { jobs, .. } = self {
            let _ = jobs.send(Job::Credit {
                id,
                amount,
                why,
                game,
            });
        }
    }

    /// Keeps the name and the look the account `id`'s owner chose
    /// (`Ask::Profile`), both checked already, and the tag that goes with
    /// the name, which `social` has seen is nobody else's of that name.
    /// Nothing comes back: the account carries them from its next load.
    pub(crate) fn profile(&self, id: u64, name: &str, look: &str, tag: u16) {
        if let Self::Database { jobs, .. } = self {
            let _ = jobs.send(Job::Profile {
                id,
                name: name.to_string(),
                look: look.to_string(),
                tag,
            });
        }
    }

    /// Writes down something about friends that the game's loop has
    /// already gone by (`social`). Nothing comes back.
    pub(crate) fn keep(&self, keep: Keep) {
        if let Self::Database { jobs, .. } = self {
            let _ = jobs.send(Job::Keep(keep));
        }
    }
}

/// One player's account.
#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub name: String,
    pub look: String,
    pub coins: u32,
    /// The number after its name, if it has been given one.
    pub tag: Option<u16>,
    /// What it has earned toward its level.
    pub xp: u32,
}

impl Record {
    /// Someone new, not kept anywhere.
    pub(crate) fn made_up() -> Self {
        Self {
            name: crate::players::new_name(),
            look: roundtown_net::PLAYER_LOOK.into(),
            coins: STARTING_BALANCE,
            tag: None,
            xp: 0,
        }
    }
}

/// The account asked for: `None` if there is none, deleted, say.
#[derive(Message)]
pub(crate) struct Loaded {
    pub id: u64,
    pub record: Option<Record>,
}

/// What the account was left with, or `None` if it had not got the coins.
#[derive(Message)]
pub(crate) struct Charged {
    pub id: u64,
    pub left: Option<u32>,
}

/// What the account has now, or `None` if nothing could be given it: there is
/// no such account any more, say.
#[derive(Message)]
pub(crate) struct Credited {
    pub id: u64,
    pub left: Option<u32>,
}

pub(crate) enum Job {
    Load(u64),
    Charge {
        id: u64,
        amount: u32,
        why: &'static str,
        game: Option<u32>,
    },
    Credit {
        id: u64,
        amount: u32,
        why: &'static str,
        game: Option<u32>,
    },
    Profile {
        id: u64,
        name: String,
        look: String,
        tag: u16,
    },
    Keep(Keep),
}

pub(crate) enum Done {
    Loaded(Loaded),
    Charged(Charged),
    Credited(Credited),
}

/// Passes on what the database's thread has answered.
pub(crate) fn answers(
    bank: Res<Bank>,
    mut loaded: MessageWriter<Loaded>,
    mut charged: MessageWriter<Charged>,
    mut credited: MessageWriter<Credited>,
) {
    let Bank::Database { done, .. } = &*bank else {
        return;
    };
    let Ok(done) = done.lock() else {
        return;
    };
    while let Ok(answer) = done.try_recv() {
        match answer {
            Done::Loaded(answer) => {
                loaded.write(answer);
            }
            Done::Charged(answer) => {
                charged.write(answer);
            }
            Done::Credited(answer) => {
                credited.write(answer);
            }
        }
    }
}

pub(crate) fn connect(url: &str) -> Result<Client, postgres::Error> {
    Client::connect(url, NoTls)
}

/// The database's thread: does each job as it comes, connecting again
/// whenever the connection has gone.
fn work(url: &str, jobs: &Receiver<Job>, done: &Sender<Done>) {
    let mut db: Option<Client> = None;
    for job in jobs {
        let mut answer = None;
        let mut finished = false;
        for _ in 0..TRIES {
            if db.is_none() {
                match connect(url) {
                    Ok(client) => db = Some(client),
                    Err(error) => {
                        error!("could not reach the database: {error}");
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                }
            }
            let Some(client) = db.as_mut() else {
                continue;
            };
            match run(client, &job) {
                Ok(result) => {
                    answer = result;
                    finished = true;
                    break;
                }
                Err(error) => {
                    error!("the database: {error}");
                    if client.is_closed() {
                        db = None;
                    } else {
                        break;
                    }
                }
            }
        }
        // Given up: nothing loaded, nothing taken, nothing given and nothing
        // kept.
        if !finished {
            answer = match job {
                Job::Load(id) => Some(Done::Loaded(Loaded { id, record: None })),
                Job::Charge { id, .. } => Some(Done::Charged(Charged { id, left: None })),
                Job::Credit { id, .. } => Some(Done::Credited(Credited { id, left: None })),
                Job::Profile { id, name, .. } => {
                    error!("could not keep {id}'s name, {name}");
                    None
                }
                Job::Keep(keep) => {
                    // The game's loop has gone by it already: it holds until
                    // the server starts again, and reads what was kept.
                    error!("could not keep {keep:?}");
                    None
                }
            };
        }
        if let Some(answer) = answer
            && done.send(answer).is_err()
        {
            return;
        }
    }
}

/// Does `job`: what it answers, if it answers at all.
fn run(db: &mut Client, job: &Job) -> Result<Option<Done>, postgres::Error> {
    match *job {
        Job::Load(id) => {
            let row = db.query_opt(
                "UPDATE players SET seen = now() WHERE id = $1 RETURNING name, look, coins, tag, xp",
                &[&(id as i64)],
            )?;
            Ok(Some(Done::Loaded(Loaded {
                id,
                record: row.map(|row| Record {
                    name: row.get(0),
                    look: row.get(1),
                    coins: row.get::<_, i64>(2).clamp(0, i64::from(u32::MAX)) as u32,
                    tag: row.get::<_, Option<i16>>(3).map(|tag| tag as u16),
                    xp: row.get::<_, i64>(4).clamp(0, i64::from(u32::MAX)) as u32,
                }),
            })))
        }
        Job::Profile {
            id,
            ref name,
            ref look,
            tag,
        } => {
            db.execute(
                "UPDATE players SET name = $2, look = $3, tag = $4 WHERE id = $1",
                &[&(id as i64), name, look, &(tag as i16)],
            )?;
            Ok(None)
        }
        Job::Keep(ref keep) => {
            keep_it(db, keep)?;
            Ok(None)
        }
        Job::Charge {
            id,
            amount,
            why,
            game,
        } => {
            // The coins and the line saying why change together, or not at
            // all; and never below nothing.
            let mut transaction = db.transaction()?;
            let left = transaction.query_opt(
                "UPDATE players SET coins = coins - $2 WHERE id = $1 AND coins >= $2 RETURNING coins",
                &[&(id as i64), &i64::from(amount)],
            )?;
            let Some(left) = left else {
                return Ok(Some(Done::Charged(Charged { id, left: None })));
            };
            transaction.execute(
                "INSERT INTO coin_changes (player, amount, reason, game) VALUES ($1, $2, $3, $4)",
                &[
                    &(id as i64),
                    &-i64::from(amount),
                    &why,
                    &game.map(i64::from),
                ],
            )?;
            transaction.commit()?;
            Ok(Some(Done::Charged(Charged {
                id,
                left: Some(left.get::<_, i64>(0).clamp(0, i64::from(u32::MAX)) as u32),
            })))
        }
        Job::Credit {
            id,
            amount,
            why,
            game,
        } => {
            // The coins and the line saying why change together, or not at
            // all.
            let mut transaction = db.transaction()?;
            let now = transaction.query_opt(
                "UPDATE players SET coins = coins + $2 WHERE id = $1 RETURNING coins",
                &[&(id as i64), &i64::from(amount)],
            )?;
            let Some(now) = now else {
                return Ok(Some(Done::Credited(Credited { id, left: None })));
            };
            transaction.execute(
                "INSERT INTO coin_changes (player, amount, reason, game) VALUES ($1, $2, $3, $4)",
                &[&(id as i64), &i64::from(amount), &why, &game.map(i64::from)],
            )?;
            transaction.commit()?;
            Ok(Some(Done::Credited(Credited {
                id,
                left: Some(now.get::<_, i64>(0).clamp(0, i64::from(u32::MAX)) as u32),
            })))
        }
    }
}

/// Writes `keep` down. Each can be written twice and leave what once would;
/// and what names an account that has been deleted meanwhile is left
/// unwritten, rather than refused by the table it would go in.
pub(crate) fn keep_it(db: &mut Client, keep: &Keep) -> Result<(), postgres::Error> {
    let row = |id: u64| id as i64;
    // Two friends as the table has them, the lesser first as the table
    // counts (`a < b`), which is by the numbers it holds.
    let pair = |a: u64, b: u64| (row(a).min(row(b)), row(a).max(row(b)));
    const BOTH_THERE: &str = "EXISTS (SELECT 1 FROM players WHERE id = $1) AND EXISTS (SELECT 1 FROM players WHERE id = $2)";
    match *keep {
        Keep::Tag { id, tag } => {
            db.execute("UPDATE players SET tag = $2 WHERE id = $1", &[&row(id), &(tag as i16)])?;
        }
        Keep::Seen(id) => {
            db.execute("UPDATE players SET seen = now() WHERE id = $1", &[&row(id)])?;
        }
        Keep::Xp { id, amount } => {
            db.execute("UPDATE players SET xp = xp + $2 WHERE id = $1", &[&row(id), &i64::from(amount)])?;
        }
        Keep::Friends { a, b } => {
            let (a, b) = pair(a, b);
            db.execute(
                &format!("INSERT INTO friends (a, b) SELECT $1::bigint, $2::bigint WHERE {BOTH_THERE} ON CONFLICT DO NOTHING"),
                &[&a, &b],
            )?;
        }
        Keep::Unfriend { a, b } => {
            let (a, b) = pair(a, b);
            db.execute("DELETE FROM friends WHERE a = $1 AND b = $2", &[&a, &b])?;
        }
        Keep::Request { from, to } => {
            db.execute(
                &format!("INSERT INTO friend_requests (sender, receiver) SELECT $1::bigint, $2::bigint WHERE {BOTH_THERE} ON CONFLICT DO NOTHING"),
                &[&row(from), &row(to)],
            )?;
        }
        Keep::Unrequest { from, to } => {
            db.execute("DELETE FROM friend_requests WHERE sender = $1 AND receiver = $2", &[&row(from), &row(to)])?;
        }
        Keep::Letter(ref letter) => {
            db.execute(
                &format!("INSERT INTO messages (sender, receiver, id, body) SELECT $1::bigint, $2::bigint, $3::bigint, $4::text WHERE {BOTH_THERE} ON CONFLICT DO NOTHING"),
                &[&row(letter.from), &row(letter.to), &row(letter.id), &letter.text],
            )?;
        }
        Keep::Unletter(id) => {
            db.execute("DELETE FROM messages WHERE id = $1", &[&row(id)])?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_free_tag_is_one_nobody_of_the_name_has() {
        // Nobody of the name: whatever is thrown.
        let none = HashSet::new();
        assert_eq!(free_tag(&none, &mut || 4321), 4321);
        // Thrown one that is taken, again and again: the next free one on
        // from it, round past the last to the first.
        let taken: HashSet<u16> = [4321, 4322, TAG_LAST].into();
        assert_eq!(free_tag(&taken, &mut || 4321), 4323);
        assert_eq!(free_tag(&taken, &mut || TAG_LAST), TAG_FIRST);
        // A throw outside the tags there are is brought inside them.
        assert_eq!(free_tag(&none, &mut || 12), TAG_FIRST);
        // All but one taken: that one, wherever the throws land.
        let mut nearly: HashSet<u16> = (TAG_FIRST..=TAG_LAST).collect();
        nearly.remove(&7777);
        assert_eq!(free_tag(&nearly, &mut || 1000), 7777);
        // Thrown fairly, every tag given is free and in range.
        let mut given = HashSet::new();
        for _ in 0..500 {
            let tag = free_tag(&given, &mut || fastrand::u16(TAG_FIRST..=TAG_LAST));
            assert!((TAG_FIRST..=TAG_LAST).contains(&tag));
            assert!(given.insert(tag), "{tag} was given twice");
        }
    }

    /// The database a test is to be run on, which has to be called something
    /// ending in `_scratch`: these tests drop tables and change accounts, and
    /// must never be pointed at the server's own.
    fn scratch(url: &str) -> Client {
        let mut db = connect(url).expect("the scratch database");
        let name: String = db.query_one("SELECT current_database()", &[]).expect("its name").get(0);
        assert!(name.ends_with("_scratch"), "{name} is not a scratch database: its name has to end in _scratch");
        db
    }

    /// Everything this writes and reads, against a real Postgres: an empty
    /// database nobody else is using, its name ending in `_scratch`, named
    /// by `RT_TEST_DB` (as `RT_DB` is written), which this empties, fills
    /// and leaves filled. Run on the server's own machine before a deploy
    /// that changes the tables:
    /// `RT_TEST_DB="host=/var/run/postgresql user=... dbname=..._scratch" <test binary> --ignored the_database`.
    #[test]
    #[ignore = "needs a scratch Postgres: RT_TEST_DB"]
    fn the_database_keeps_tags_friends_requests_and_messages() {
        let url = std::env::var("RT_TEST_DB").expect("RT_TEST_DB: a scratch database");
        let mut db = scratch(&url);
        // As it was before tags: the tables of 2026-11-12, with accounts in
        // them, two of one name.
        db.batch_execute(
            "DROP TABLE IF EXISTS messages, friend_requests, friends, coin_changes, devices, players CASCADE;
             CREATE TABLE players (
                 id BIGSERIAL PRIMARY KEY, name TEXT NOT NULL, look TEXT NOT NULL,
                 coins BIGINT NOT NULL CHECK (coins >= 0),
                 created TIMESTAMPTZ NOT NULL DEFAULT now(), seen TIMESTAMPTZ NOT NULL DEFAULT now());
             INSERT INTO players (name, look, coins) VALUES
                 ('Mochi 482', 'blue', 1000), ('mochi 482', 'girl', 900), ('Hajooon', 'boy', 1325), ('Bobby', 'boy', 5);",
        )
        .expect("the old tables");
        // Starting up gives every account its tag, no two of one name the
        // same, and reads nobody's friends.
        let first = set_up(&mut db).expect("set up");
        assert_eq!(first.people.len(), 4);
        assert!(first.people.iter().all(|person| (TAG_FIRST..=TAG_LAST).contains(&person.tag)), "{first:?}");
        assert_ne!(first.people[0].tag, first.people[1].tag, "two Mochi 482s");
        assert!(first.friends.is_empty() && first.requests.is_empty() && first.letters.is_empty());
        let (a, b, c, d) = (first.people[0].id, first.people[1].id, first.people[2].id, first.people[3].id);
        // Starting again changes nothing.
        assert_eq!(set_up(&mut db).expect("set up again"), first);
        // The table itself will not have two of one name and tag, whatever
        // their capitals.
        let same = db.execute(
            "UPDATE players SET name = 'MOCHI 482', tag = $2 WHERE id = $1",
            &[&(b as i64), &(first.people[0].tag as i16)],
        );
        assert!(same.is_err(), "two accounts with one name and tag");
        // Everything `social` asks to be kept.
        let done = |db: &mut Client, job: Job| run(db, &job).expect("a job");
        done(&mut db, Job::Profile { id: c, name: "Hajun".into(), look: "girl".into(), tag: 4242 });
        for keep in [
            Keep::Tag { id: d, tag: 1234 },
            Keep::Xp { id: c, amount: 70 },
            Keep::Xp { id: c, amount: 50 },
            Keep::Friends { a: c, b: a },
            Keep::Friends { a, b: c },
            Keep::Friends { a: b, b: d },
            Keep::Unfriend { a: d, b },
            Keep::Request { from: d, to: c },
            Keep::Request { from: d, to: c },
            Keep::Request { from: b, to: c },
            Keep::Unrequest { from: b, to: c },
            Keep::Letter(Letter { id: 11, from: a, to: c, text: "play at 9?".into() }),
            Keep::Letter(Letter { id: 12, from: a, to: c, text: "ok".into() }),
            Keep::Unletter(11),
            Keep::Seen(a),
            // About an account that is not there: left unwritten, no error.
            Keep::Friends { a, b: 999_999 },
            Keep::Request { from: 999_999, to: a },
            Keep::Letter(Letter { id: 13, from: a, to: 999_999, text: "hello?".into() }),
        ] {
            keep_it(&mut db, &keep).unwrap_or_else(|error| panic!("{keep:?}: {error}"));
        }
        let kept = Snapshot::read(&mut db).expect("read back");
        let person = |id: u64| kept.people.iter().find(|person| person.id == id).expect("an account");
        assert_eq!((person(c).name.as_str(), person(c).tag, person(c).xp), ("Hajun", 4242, 120));
        assert_eq!(person(d).tag, 1234);
        assert!(person(a).seen > 1_700_000_000, "seen is seconds since 1970: {}", person(a).seen);
        assert_eq!(kept.friends, vec![(a.min(c), a.max(c))]);
        assert_eq!(kept.requests, vec![(d, c)]);
        assert_eq!(kept.letters, vec![Letter { id: 12, from: a, to: c, text: "ok".into() }]);
        // An account loads with its tag and what it has earned.
        let Some(Done::Loaded(loaded)) = done(&mut db, Job::Load(c)) else {
            panic!("no answer to a load");
        };
        let record = loaded.record.expect("the account");
        assert_eq!((record.name.as_str(), record.look.as_str(), record.coins, record.tag, record.xp), ("Hajun", "girl", 1325, Some(4242), 120));
        // An account deleted takes its friends, what it asked and what was
        // written to and by it with it.
        db.execute("DELETE FROM players WHERE id = $1", &[&(c as i64)]).expect("delete");
        let after = Snapshot::read(&mut db).expect("read back");
        assert!(after.friends.is_empty() && after.requests.is_empty() && after.letters.is_empty(), "{after:?}");
        assert_eq!(after.people.len(), 3);
        // The login makes a new device an account with a tag nobody of its
        // name has, a device and its starting coins; and finds it again.
        let mut made = Vec::new();
        for device in 0..40u8 {
            let secret = [device; 32];
            let id = crate::login::account_of(&mut db, &secret).expect("the login").expect("an account");
            assert_eq!(crate::login::account_of(&mut db, &secret).expect("the login"), Some(id), "the same device again");
            made.push(id as u64);
        }
        let with_them = Snapshot::read(&mut db).expect("read back");
        assert_eq!(with_them.people.len(), 43);
        let mut handles = HashSet::new();
        for person in &with_them.people {
            assert!((TAG_FIRST..=TAG_LAST).contains(&person.tag), "{person:?}");
            assert!(handles.insert((person.name.to_lowercase(), person.tag)), "{person:?} twice");
        }
        let coins: i64 = db
            .query_one("SELECT coins FROM players WHERE id = $1", &[&(made[0] as i64)])
            .expect("the account")
            .get(0);
        assert_eq!(coins, i64::from(STARTING_BALANCE));
    }

    /// The tables as they stand, with the accounts in them, taken through
    /// starting up: for a copy of the server's own database, its name ending
    /// in `_scratch`, before the server is first started on the real one
    /// with tags (`RT_TEST_DB`, as above; this drops nothing, and changes
    /// only what starting up changes). It says how many accounts there are
    /// and nothing of whose.
    #[test]
    #[ignore = "needs a copy of the database: RT_TEST_DB"]
    fn the_accounts_there_are_take_their_tags() {
        let url = std::env::var("RT_TEST_DB").expect("RT_TEST_DB: a copy of the database");
        let mut db = scratch(&url);
        // Everything about every account but its tag, as it stands.
        let rows = |db: &mut Client| -> Vec<(i64, String, String, i64)> {
            db.query("SELECT id, name, look, coins FROM players ORDER BY id", &[])
                .expect("players")
                .iter()
                .map(|row| (row.get(0), row.get(1), row.get(2), row.get(3)))
                .collect()
        };
        let count = |db: &mut Client, table: &str| -> i64 {
            db.query_one(&format!("SELECT count(*) FROM {table}"), &[]).expect("a count").get(0)
        };
        let before = rows(&mut db);
        let (devices, changes) = (count(&mut db, "devices"), count(&mut db, "coin_changes"));
        let first = set_up(&mut db).expect("set up");
        assert_eq!(rows(&mut db), before, "an account's name, look or coins changed, or an account came or went");
        assert_eq!(count(&mut db, "devices"), devices, "a device came or went");
        assert_eq!(count(&mut db, "coin_changes"), changes, "a coin change came or went");
        assert_eq!(first.people.len(), before.len());
        let mut handles = HashSet::new();
        for person in &first.people {
            assert!((TAG_FIRST..=TAG_LAST).contains(&person.tag), "account {}: tag {}", person.id, person.tag);
            assert!(handles.insert((person.name.to_lowercase(), person.tag)), "account {}: its name and tag twice", person.id);
            assert_eq!(person.xp, 0, "account {}", person.id);
        }
        assert!(first.friends.is_empty() && first.requests.is_empty() && first.letters.is_empty());
        // Starting again gives nobody another tag.
        assert_eq!(set_up(&mut db).expect("set up again"), first);
        // And the login still makes an account in it, and finds it again:
        // taken out after, so that the copy is left as starting up left it.
        let secret = [0xA5u8; 32];
        let made = crate::login::account_of(&mut db, &secret).expect("the login").expect("an account");
        assert_eq!(crate::login::account_of(&mut db, &secret).expect("the login"), Some(made));
        assert_eq!(rows(&mut db).len(), before.len() + 1);
        db.execute("DELETE FROM players WHERE id = $1", &[&made]).expect("delete");
        assert_eq!(rows(&mut db), before);
        assert_eq!(count(&mut db, "devices"), devices, "the device made went with its account");
        let names: HashSet<String> = before.iter().map(|row| row.1.to_lowercase()).collect();
        println!(
            "{} accounts ({} names among them), {} devices and {} coin changes: each account has its tag, and nothing else of it changed",
            before.len(),
            names.len(),
            devices,
            changes
        );
    }
}
