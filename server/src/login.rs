//! The login: how a device gets in (`MULTIPLAYER.md`, phase 2).
//!
//! A device has a secret of its own, sixteen random bytes made the first time
//! it ran. It sends that here over HTTPS — Caddy does the HTTPS, on TCP 443,
//! and passes the request on to this, on 127.0.0.1:8080 — and is handed a
//! connect token for the game server: netcode's, naming its account and
//! signed with the key only the login and the game server have. Without one
//! from here, nobody gets in: the game server is in netcode's secure mode.
//!
//! * `POST /login` with `{"device": "<32 hex digits>", "to": "<address>"}`:
//!   the token, for the game server at `to`, one of its public addresses. A
//!   device never seen before gets an account: a name, a look and
//!   [`STARTING_BALANCE`] coins.
//! * `POST /delete` with `{"device": "..."}`: the account that device plays,
//!   and everything kept about it, gone for good. Apple asks for that inside
//!   the app, even for accounts nobody signed up for.
//!
//! Only a hash of each secret is kept, so a copy of the database is no way
//! in. A device that asks too often is turned away for a while.

use std::collections::HashMap;
use std::io::Read;
use std::net::SocketAddr;
use std::time::{Duration, Instant, SystemTime};

use bevy::prelude::*;
use bevy_replicon_renet::netcode::{ConnectToken, NETCODE_USER_DATA_BYTES};
use postgres::Client;
use roundtown_net::PROTOCOL_ID;
use roundtown_net::rules::STARTING_BALANCE;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tiny_http::{Header, Method, Request, Response, Server};

use crate::accounts;

/// Where the login listens, for Caddy alone.
const LISTEN: &str = "127.0.0.1:8080";
/// How long a token is good for, in seconds: long enough to connect with, as
/// the device does at once.
const TOKEN_SECS: u64 = 120;
/// How long a connection can go without a word before it is dropped, in
/// seconds.
const TIMEOUT_SECS: i32 = 15;
/// How many requests one address may make a minute.
const PER_MINUTE: u32 = 30;
/// The longest request read.
const MOST_BYTES: usize = 1024;

/// What the login was set up with.
pub(crate) struct Setup {
    pub db: String,
    pub key: [u8; 32],
    /// The game server's addresses, as devices reach it: what a token may
    /// name.
    pub public: Vec<SocketAddr>,
}

/// Starts the login on a thread of its own.
pub(crate) fn start(setup: Setup) {
    std::thread::Builder::new()
        .name("login".into())
        .spawn(move || serve(&setup))
        .expect("a thread for the login");
}

#[derive(Deserialize)]
struct Asked {
    device: String,
    to: Option<String>,
}

fn serve(setup: &Setup) {
    let server = match Server::http(LISTEN) {
        Ok(server) => server,
        Err(error) => {
            error!("the login could not listen on {LISTEN}: {error}");
            return;
        }
    };
    info!("the login is listening on {LISTEN}");
    let mut db: Option<Client> = None;
    let mut asked: HashMap<String, (Instant, u32)> = HashMap::new();
    for mut request in server.incoming_requests() {
        // Who asked: Caddy says, in front of everyone.
        let from = request
            .headers()
            .iter()
            .find(|header| header.field.equiv("X-Forwarded-For"))
            .map_or_else(
                || request.remote_addr().map(|addr| addr.ip().to_string()).unwrap_or_default(),
                |header| header.value.as_str().split(',').next().unwrap_or("").trim().to_owned(),
            );
        let now = Instant::now();
        let (since, count) = asked.entry(from.clone()).or_insert((now, 0));
        if now.duration_since(*since) > Duration::from_secs(60) {
            *since = now;
            *count = 0;
        }
        *count += 1;
        if *count > PER_MINUTE {
            reply(request, 429, Vec::new());
            continue;
        }
        if asked.len() > 10_000 {
            asked.retain(|_, (since, _)| now.duration_since(*since) < Duration::from_secs(60));
        }

        let mut body = Vec::new();
        let read = request
            .as_reader()
            .take(MOST_BYTES as u64)
            .read_to_end(&mut body)
            .is_ok();
        let path = request.url().to_owned();
        let asked: Option<Asked> = read.then(|| serde_json::from_slice(&body).ok()).flatten();
        let (Method::Post, Some(asked)) = (request.method(), asked) else {
            reply(request, 400, Vec::new());
            continue;
        };
        let Some(secret) = secret(&asked.device) else {
            reply(request, 400, Vec::new());
            continue;
        };
        if db.is_none() {
            match accounts::connect(&setup.db) {
                Ok(client) => db = Some(client),
                Err(error) => {
                    error!("the login could not reach the database: {error}");
                    reply(request, 503, Vec::new());
                    continue;
                }
            }
        }
        let Some(client) = db.as_mut() else {
            continue;
        };
        let answer = match path.as_str() {
            "/login" => login(client, setup, &secret, asked.to.as_deref()),
            "/delete" => delete(client, &secret),
            _ => Ok((404, Vec::new())),
        };
        match answer {
            Ok((status, body)) => reply(request, status, body),
            Err(error) => {
                error!("the login: {error}");
                if client.is_closed() {
                    db = None;
                }
                reply(request, 500, Vec::new());
            }
        }
    }
}

fn reply(request: Request, status: u16, body: Vec<u8>) {
    let response = Response::from_data(body)
        .with_status_code(status)
        .with_header(
            "Content-Type: application/octet-stream"
                .parse::<Header>()
                .expect("a header"),
        );
    let _ = request.respond(response);
}

/// The hash a device's secret, 32 hex digits, is kept under.
fn secret(hex: &str) -> Option<Vec<u8>> {
    if hex.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (n, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(n * 2..n * 2 + 2)?, 16).ok()?;
    }
    Some(Sha256::digest(bytes).to_vec())
}

/// A token for the device whose secret hashes to `secret`, for the game
/// server at `to`: its account's, made now if it has none.
fn login(
    db: &mut Client,
    setup: &Setup,
    secret: &[u8],
    to: Option<&str>,
) -> Result<(u16, Vec<u8>), postgres::Error> {
    let Some(to) = to
        .and_then(|to| to.parse::<SocketAddr>().ok())
        .filter(|to| setup.public.contains(to))
    else {
        return Ok((400, Vec::new()));
    };
    let found = db.query_opt("SELECT player FROM devices WHERE secret = $1", &[&secret])?;
    let id: i64 = match found {
        Some(row) => row.get(0),
        None => {
            let mut transaction = db.transaction()?;
            let name = crate::players::new_name();
            let row = transaction.query_one(
                "INSERT INTO players (name, look, coins) VALUES ($1, $2, $3) RETURNING id",
                &[&name, &roundtown_net::PLAYER_LOOK, &i64::from(STARTING_BALANCE)],
            )?;
            let id: i64 = row.get(0);
            transaction.execute(
                "INSERT INTO devices (secret, player) VALUES ($1, $2)",
                &[&secret, &id],
            )?;
            transaction.execute(
                "INSERT INTO coin_changes (player, amount, reason) VALUES ($1, $2, 'starting balance')",
                &[&id, &i64::from(STARTING_BALANCE)],
            )?;
            transaction.commit()?;
            info!("a new account, {id}: {name}");
            id
        }
    };
    let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
    user_data[..8].copy_from_slice(&(id as u64).to_le_bytes());
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let token = ConnectToken::generate(
        now,
        PROTOCOL_ID,
        TOKEN_SECS,
        // Fresh for every connection: one that has dropped can still be on
        // the game server's books for a while, under its old id.
        u64::from_le_bytes(bevy_replicon_renet::netcode::generate_random_bytes()),
        TIMEOUT_SECS,
        vec![to],
        Some(&user_data),
        &setup.key,
    );
    let Ok(token) = token else {
        return Ok((500, Vec::new()));
    };
    let mut bytes = Vec::new();
    if token.write(&mut bytes).is_err() {
        return Ok((500, Vec::new()));
    }
    Ok((200, bytes))
}

/// Deletes the account of the device whose secret hashes to `secret`, and
/// everything kept about it.
fn delete(db: &mut Client, secret: &[u8]) -> Result<(u16, Vec<u8>), postgres::Error> {
    let gone = db.execute(
        "DELETE FROM players WHERE id = (SELECT player FROM devices WHERE secret = $1)",
        &[&secret],
    )?;
    if gone > 0 {
        info!("an account was deleted");
    }
    Ok((204, Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_is_32_hex_digits() {
        assert!(secret("00112233445566778899aabbccddeeff").is_some());
        assert!(secret("00112233445566778899aabbccddeef").is_none());
        assert!(secret("zz112233445566778899aabbccddeeff").is_none());
        // Kept as its hash, never as itself.
        let kept = secret("00112233445566778899aabbccddeeff").unwrap();
        assert_eq!(kept.len(), 32);
        assert_ne!(&kept[..16], &[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    }
}
