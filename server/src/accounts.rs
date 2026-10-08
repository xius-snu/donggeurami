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

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use bevy::prelude::*;
use postgres::{Client, NoTls};
use roundtown_net::rules::STARTING_BALANCE;

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
";

/// How many times a job is tried before it is given up, with the connection
/// made again between.
const TRIES: u32 = 3;

pub(crate) fn plugin(app: &mut App, db: Option<String>) {
    let bank = match db {
        Some(url) => {
            if let Err(error) = connect(&url).and_then(|mut db| db.batch_execute(SCHEMA)) {
                // Without the tables nothing can be kept: better to stop here,
                // and have systemd say so, than to take anyone's coins.
                panic!("could not set up the database: {error}");
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
        .add_systems(PreUpdate, answers);
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
}

/// One player's account.
#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub name: String,
    pub look: String,
    pub coins: u32,
}

impl Record {
    /// Someone new, not kept anywhere.
    pub(crate) fn made_up() -> Self {
        Self {
            name: crate::players::new_name(),
            look: roundtown_net::PLAYER_LOOK.into(),
            coins: STARTING_BALANCE,
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

pub(crate) enum Job {
    Load(u64),
    Charge {
        id: u64,
        amount: u32,
        why: &'static str,
        game: Option<u32>,
    },
}

pub(crate) enum Done {
    Loaded(Loaded),
    Charged(Charged),
}

/// Passes on what the database's thread has answered.
pub(crate) fn answers(bank: Res<Bank>, mut loaded: MessageWriter<Loaded>, mut charged: MessageWriter<Charged>) {
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
                Ok(done) => {
                    answer = Some(done);
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
        // Given up: nothing loaded, and nothing taken.
        let answer = answer.unwrap_or(match job {
            Job::Load(id) => Done::Loaded(Loaded { id, record: None }),
            Job::Charge { id, .. } => Done::Charged(Charged { id, left: None }),
        });
        if done.send(answer).is_err() {
            return;
        }
    }
}

fn run(db: &mut Client, job: &Job) -> Result<Done, postgres::Error> {
    match *job {
        Job::Load(id) => {
            let row = db.query_opt(
                "UPDATE players SET seen = now() WHERE id = $1 RETURNING name, look, coins",
                &[&(id as i64)],
            )?;
            Ok(Done::Loaded(Loaded {
                id,
                record: row.map(|row| Record {
                    name: row.get(0),
                    look: row.get(1),
                    coins: row.get::<_, i64>(2).clamp(0, i64::from(u32::MAX)) as u32,
                }),
            }))
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
                return Ok(Done::Charged(Charged { id, left: None }));
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
            Ok(Done::Charged(Charged {
                id,
                left: Some(left.get::<_, i64>(0).clamp(0, i64::from(u32::MAX)) as u32),
            }))
        }
    }
}
