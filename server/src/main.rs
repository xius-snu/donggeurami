//! 동그라미타운's game server: one program holding every room there is — the
//! town's channels, everyone's home, and every game of House Builder
//! (`MULTIPLAYER.md`, "The shape").
//!
//! It is a Bevy app with no window, no renderer and no assets: replicon over
//! renet, on one UDP port. What it holds, it holds in memory: who is where,
//! and the games under way. It knows nothing of the islands' shapes. Each
//! device moves its own body, and the server only checks that a move is one
//! a body could make ([`players`]) before passing it on to the room. What is
//! kept, accounts and their coins, is kept in Postgres ([`accounts`]), and
//! devices get in by the login ([`login`]).
//!
//! Set up from the environment, which its systemd unit reads from
//! `/etc/roundtown/env`:
//!
//! * `RT_PORT`: the UDP port, [`GAME_PORT`] unless set.
//! * `RT_KEY`: 64 hex digits, the key the login signs connect tokens with.
//!   With it, netcode's secure mode: nobody gets in without a token from the
//!   login. Without it, anyone can, saying who they are: for trying the server
//!   out only, and no build that talks to it that way goes to the stores.
//! * `RT_PUBLIC`: the addresses devices reach the game at, separated by
//!   commas, which a token names. Needed with `RT_KEY`.
//! * `RT_DB`: how to reach Postgres, e.g. `host=/var/run/postgresql
//!   user=roundtown dbname=roundtown`. Without it, accounts are made up afresh
//!   on connecting and coins kept in memory. The login needs it.
//! * `RT_ALLOW_SKIP`: `1` lets a device skip House Builder's waits, for trying
//!   the game out. Never on a server real players use.

mod accounts;
mod builder;
mod login;
mod players;
mod rooms;
mod stats;

use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::{Duration, SystemTime};

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_replicon_renet::renet::ConnectionConfig;
use bevy_replicon_renet::{RenetChannelsExt, RenetServer, RepliconRenetPlugins};
use roundtown_net::{GAME_PORT, PROTOCOL_ID, TICK_HZ};

/// How many times a second the server looks at what has come in. Moves are
/// passed on at [`TICK_HZ`]; this only keeps what arrives from waiting long
/// for the next look.
const LOOP_HZ: f64 = 60.0;

/// The most devices connected at once. netcode allows no more than 1,024.
const MAX_CLIENTS: usize = 1024;

/// How the server was set up to run.
#[derive(Resource, Clone, Debug)]
pub(crate) struct Config {
    pub port: u16,
    /// The key connect tokens are signed with: secure mode.
    pub key: Option<[u8; 32]>,
    /// Where devices reach the game.
    pub public: Vec<SocketAddr>,
    pub db: Option<String>,
    /// Devices may skip House Builder's waits.
    pub allow_skip: bool,
}

impl Config {
    fn from_env() -> Self {
        let var = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        let key = var("RT_KEY").map(|hex| {
            let mut key = [0u8; 32];
            let ok = hex.len() == 64
                && key.iter_mut().enumerate().all(|(n, byte)| {
                    hex.get(n * 2..n * 2 + 2)
                        .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                        .map(|value| *byte = value)
                        .is_some()
                });
            assert!(ok, "RT_KEY has to be 64 hex digits");
            key
        });
        Self {
            port: var("RT_PORT")
                .and_then(|port| port.parse().ok())
                .unwrap_or(GAME_PORT),
            key,
            public: var("RT_PUBLIC")
                .map(|list| {
                    list.split(',')
                        .map(|address| address.trim().parse().expect("RT_PUBLIC: an address"))
                        .collect()
                })
                .unwrap_or_default(),
            db: var("RT_DB"),
            allow_skip: var("RT_ALLOW_SKIP").is_some_and(|on| on == "1"),
        }
    }
}

fn main() {
    let config = Config::from_env();
    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / LOOP_HZ,
        ))),
    )
    .add_plugins((
        StatesPlugin,
        LogPlugin {
            filter: "info,bevy_replicon=warn,renetcode=warn,renet=warn".into(),
            ..default()
        },
    ))
    .add_plugins((RepliconPlugins, RepliconRenetPlugins))
    // What has changed goes out every time this ticks.
    .insert_resource(Time::<Fixed>::from_hz(TICK_HZ))
    .insert_resource(config.clone());
    roundtown_net::protocol(&mut app);
    accounts::plugin(&mut app, config.db.clone());
    rooms::plugin(&mut app);
    players::plugin(&mut app);
    builder::plugin(&mut app);
    stats::plugin(&mut app);
    app.add_systems(Startup, listen);
    match (config.key, config.db) {
        (Some(key), Some(db)) => login::start(login::Setup {
            db,
            key,
            public: config.public,
        }),
        (Some(_), None) => panic!("RT_KEY without RT_DB: the login needs the database"),
        (None, _) => {}
    }
    app.run();
}

/// Opens the port, once everything the two sides say to each other has been
/// registered: renet's channels are counted from that.
fn listen(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    config: Res<Config>,
) -> Result<()> {
    let server = RenetServer::new(ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..default()
    });
    let socket = socket(config.port)?;
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let (authentication, public_addresses) = match config.key {
        Some(private_key) => {
            assert!(
                !config.public.is_empty(),
                "RT_KEY without RT_PUBLIC: tokens name an address"
            );
            (
                ServerAuthentication::Secure { private_key },
                config.public.clone(),
            )
        }
        None => (ServerAuthentication::Unsecure, vec![socket.local_addr()?]),
    };
    let transport = NetcodeServerTransport::new(
        ServerConfig {
            current_time,
            max_clients: MAX_CLIENTS,
            protocol_id: PROTOCOL_ID,
            public_addresses,
            authentication,
        },
        socket,
    )?;
    commands.insert_resource(server);
    commands.insert_resource(transport);
    info!(
        "listening on UDP port {}, {}{}",
        config.port,
        if config.key.is_some() {
            "for tokens from the login"
        } else {
            "letting anyone in (no RT_KEY)"
        },
        if config.allow_skip {
            ", with House Builder's waits skippable"
        } else {
            ""
        }
    );
    Ok(())
}

/// One socket for IPv4 and IPv6 alike: phones on an IPv6-only network reach
/// the server over IPv6, everyone else over whichever they have. Where the
/// machine has no IPv6 at all, IPv4 alone.
fn socket(port: u16) -> io::Result<UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};
    let both = || -> io::Result<UdpSocket> {
        let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_only_v6(false)?;
        socket.bind(&SocketAddr::from((Ipv6Addr::UNSPECIFIED, port)).into())?;
        Ok(socket.into())
    };
    both().or_else(|error| {
        warn!("no IPv6 ({error}): listening on IPv4 alone");
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))
    })
}
