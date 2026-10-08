//! Playing online: the server in Seoul (`MULTIPLAYER.md`), reached in the
//! background while the town is already there to play in.
//!
//! The app never waits for the network. It opens on the town as it always
//! has, with seven AIs; this connects meanwhile, and once the server has let
//! you in, whoever else is in your channel of the town walks into it, and one
//! of the AIs leaves for each of them (`lobby::fill_the_town`). With no
//! server to be had — no signal, the server down — it is the town as it was
//! offline, and this keeps trying, less and less often.
//!
//! What goes over the network is in `roundtown_net`. What this does with it:
//!
//! * **Your own body** is moved here, at once, as it always was. Where it is
//!   goes to the server 15 times a second while it moves, and not at all
//!   while you stand still ([`tell_where_i_am`]). The server only ever moves
//!   it to put you somewhere else, like going home ([`hear`]).
//! * **Everyone else** comes as an entity of replicon's, which is given a body
//!   like any other (`lobby::dress`) and drawn where the server says they
//!   are, two updates behind, smoothed between them ([`smooth`]): updates
//!   come 15 times a second, and the screen is drawn 120 times.
//! * **Going home and back** asks the server ([`Ask::Travel`]), which seats you
//!   and says where you arrive.
//!
//! When the app goes to the background the connection is closed, and when it
//! comes back it is made again: a phone can take back a sleeping app's
//! sockets, and the server would only time it out.
//!
//! On desktop `RT_SERVER` says which server instead: an address, a list of
//! them separated by commas, or `off` to stay offline.

use std::collections::VecDeque;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::SystemTime;

use bevy::prelude::*;
use bevy::window::AppLifecycle;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::netcode::{
    ClientAuthentication, ConnectToken, NetcodeClientTransport, NetcodeErrorEvent,
    generate_random_bytes,
};
use bevy_replicon_renet::renet::ConnectionConfig;
use bevy_replicon_renet::{RenetChannelsExt, RenetClient, RepliconRenetPlugins};
use roundtown_net::{Ask, Member, Moved, PROTOCOL_ID, Spot, TICK_HZ, Tell, Venue};

use crate::builder::OfflineGame;
use crate::lobby::{self, Balance, Seat, Whereabouts};
use crate::{OrbitCamera, Player, PlayerJump};

/// Where the server is: the Vultr VM in Seoul, over IPv6 first and then IPv4
/// (`MULTIPLAYER.md`, phase 1). A phone on a network with only IPv6 — Apple
/// tests every app on one — reaches it over the first; one with no IPv6, or a
/// broken one, gives up on it in [`TRY_FIRST`] seconds and uses the second.
const SERVERS: &[&str] = &[
    "[2401:c080:1c02:dd6:5400:6ff:fed3:eff6]:443",
    "141.164.62.193:443",
];

/// How long to wait for the first address to answer before trying the next,
/// and for the last, in seconds. An answer comes in a round trip, about 10 ms
/// in Korea; these only wait out an address that never will.
const TRY_FIRST: f64 = 3.0;
const TRY_LAST: f64 = 6.0;
/// How long to wait for the login to answer, in seconds: to reach it, and
/// for what it says (`login`).
const LOGIN_WAIT: f64 = 12.0;

/// How long to wait before trying again after every address failed, at
/// first, and at most, in seconds: doubling each time in between.
const REST_MIN: f64 = 4.0;
const REST_MAX: f64 = 60.0;

/// How far behind the server's latest the others are drawn, in seconds: two
/// of its updates, so that there is nearly always one on either side of the
/// moment drawn to smooth between.
const BEHIND: f64 = 2.0 / TICK_HZ;

/// How long someone is carried on the way they were going when their next
/// update is late, in seconds, before they are held where they last were.
const CARRY_ON: f64 = 0.1;

/// How far apart two updates can be, in metres, and still be a walk between
/// them rather than a jump to the second: the camera's own `CAMERA_CUT`.
const LEAP: f32 = 3.0;

/// How fast the estimate of the server's clock drifts back, in seconds a
/// second, so that a route that has got slower is noticed within seconds.
const CLOCK_DRIFT: f64 = 0.01;

/// The file who this device is lives in, beside the town save.
const DEVICE_FILE: &str = "device.id";

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((RepliconPlugins, RepliconRenetPlugins));
    roundtown_net::protocol(app);
    app.insert_resource(Link::new())
        .init_resource::<Online>()
        .init_resource::<Clock>()
        .add_observer(transport_failed)
        .add_observer(outdated)
        .add_systems(
            PreUpdate,
            (dress_newcomers, hear_spots, hear)
                .chain()
                .after(ClientSystems::Receive),
        )
        .add_systems(
            Update,
            (
                (lifecycle, link).chain(),
                smooth.before(crate::move_bodies),
                tell_where_i_am.after(crate::move_bodies),
            ),
        );
}

/// Whether this device is playing online: let in by the server, which has said
/// who you are.
#[derive(Resource, Default)]
pub(crate) struct Online {
    me: Option<Member>,
    /// How many times the server has moved you, as of now: sent with every
    /// move, so that the server can tell a move from before it moved you.
    arrival: u8,
    /// The island the server last put you on. Your moves are sent only while
    /// you are where it thinks you are.
    venue: Option<Venue>,
}

impl Online {
    pub(crate) fn is(&self) -> bool {
        self.me.is_some()
    }

    /// Who you are online, while you are.
    #[allow(dead_code)]
    pub(crate) fn me(&self) -> Option<&Member> {
        self.me.as_ref()
    }
}

/// The connection to the server, and what it is doing.
#[derive(Resource)]
pub(crate) struct Link {
    servers: Vec<SocketAddr>,
    /// Who this device is, to the server: the same from one run to the next.
    device: [u8; 16],
    state: LinkState,
    /// How long to rest after the next failure.
    rest: f64,
    /// Getting in by the login, with a token, rather than saying who you are.
    secure: bool,
    /// The login's answer, while it is being asked.
    answer: Option<Mutex<Receiver<Answer>>>,
    /// The address the connection came up on, last.
    linked_to: Option<SocketAddr>,
    /// To be connected again from the start, as someone new.
    again: bool,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum LinkState {
    /// Not trying: there is no server to try.
    Off,
    /// The app is in the background.
    Asleep,
    /// The server is newer than this app, which has to be updated to play
    /// online.
    Outdated,
    /// Waiting to try again until this time, in real seconds.
    Resting { until: f64 },
    /// Asking the login for a token for the server's `nth` address, since
    /// then.
    Asking { nth: usize, since: f64 },
    /// Trying the server's `nth` address, since then.
    Trying { nth: usize, since: f64 },
    /// Connected.
    Linked,
}

/// What the login answered.
type Answer = Result<ConnectToken, String>;

impl Link {
    fn new() -> Self {
        let desktop = cfg!(not(any(target_os = "android", target_os = "ios")));
        let configured = std::env::var("RT_SERVER").ok().filter(|_| desktop);
        let servers: Vec<SocketAddr> = match configured.as_deref() {
            Some("off") => Vec::new(),
            Some(list) => list
                .split(',')
                .filter_map(|address| address.trim().parse().ok())
                .collect(),
            None => SERVERS.iter().filter_map(|address| address.parse().ok()).collect(),
        };
        // Saying who you are without the login, for trying a server out on
        // this machine: only a debug build on desktop, never a store's.
        let secure = !(desktop
            && cfg!(debug_assertions)
            && std::env::var_os("RT_UNSECURE").is_some());
        Self {
            state: if servers.is_empty() {
                LinkState::Off
            } else {
                LinkState::Resting { until: 0.0 }
            },
            servers,
            device: device(),
            rest: REST_MIN,
            secure,
            answer: None,
            linked_to: None,
            again: false,
        }
    }

    /// Asks the login to delete this device's account and everything kept
    /// about it, on a thread of its own: the answer comes on what this
    /// returns. Only while connected through the login.
    pub(crate) fn delete_account(&self) -> Option<Receiver<Result<(), String>>> {
        let server = self.linked_to.filter(|_| self.secure && self.state == LinkState::Linked)?;
        let device = self.device;
        let (answer, answered) = channel();
        std::thread::Builder::new()
            .name("delete".into())
            .spawn(move || {
                let _ = answer.send(crate::login::delete(server, device));
            })
            .ok()?;
        Some(answered)
    }

    /// The account is gone: this device forgets its secret, makes up a new
    /// one, and connects again as someone new.
    pub(crate) fn start_over(&mut self) {
        if let Some(path) = crate::map::data_file(DEVICE_FILE) {
            let _ = std::fs::remove_file(path);
        }
        self.device = device();
        self.again = true;
    }
}

/// Who this device is: sixteen random bytes made up the first time it ran,
/// kept with the town save. They are the secret its account is found by, so
/// they come from the system's own randomness, and they are never shown to
/// anyone. Without anywhere to keep them, made up afresh each run.
fn device() -> [u8; 16] {
    let path = crate::map::data_file(DEVICE_FILE);
    let parse = |text: &str| -> Option<[u8; 16]> {
        let text = text.trim();
        (text.len() == 32).then_some(())?;
        let mut device = [0; 16];
        for (n, byte) in device.iter_mut().enumerate() {
            *byte = u8::from_str_radix(text.get(n * 2..n * 2 + 2)?, 16).ok()?;
        }
        Some(device)
    };
    if let Some(device) = path
        .as_deref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| parse(&text))
    {
        return device;
    }
    let device: [u8; 16] = generate_random_bytes();
    if let Some(path) = path {
        let text: String = device.iter().map(|byte| format!("{byte:02x}")).collect();
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&path, text));
        if let Err(error) = written {
            warn!("could not keep who this device is in {}: {error}", path.display());
        }
    }
    device
}

/// Opens a connection to `server`, getting in as `authentication` says,
/// which renet and replicon take on from here: they say when it is up.
fn open(
    commands: &mut Commands,
    channels: &RepliconChannels,
    server: SocketAddr,
    authentication: ClientAuthentication,
) -> std::io::Result<()> {
    let client = RenetClient::new(ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..default()
    });
    let any = if server.is_ipv6() {
        SocketAddr::from((Ipv6Addr::UNSPECIFIED, 0))
    } else {
        SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))
    };
    let socket = UdpSocket::bind(any)?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let transport = NetcodeClientTransport::new(now, authentication, socket)
        .map_err(std::io::Error::other)?;
    commands.insert_resource(client);
    commands.insert_resource(transport);
    Ok(())
}

/// Getting in without the login: saying who you are, for a server on this
/// machine run without its key.
fn unsecure(server: SocketAddr, device: [u8; 16]) -> ClientAuthentication {
    let mut user_data = [0u8; 256];
    user_data[..16].copy_from_slice(&device);
    ClientAuthentication::Unsecure {
        protocol_id: PROTOCOL_ID,
        // Fresh for every connection: one that has dropped can still be on
        // the server's books for a while, under its old id.
        client_id: u64::from_le_bytes(generate_random_bytes()),
        server_addr: server,
        user_data: Some(user_data),
    }
}

/// Closes the connection, if there is one, telling the server so.
fn close(commands: &mut Commands, transport: Option<&mut NetcodeClientTransport>) {
    if let Some(transport) = transport {
        transport.disconnect();
    }
    commands.remove_resource::<RenetClient>();
    commands.remove_resource::<NetcodeClientTransport>();
}

/// The app going to the background closes the connection; coming back makes
/// it again, at once.
fn lifecycle(
    mut lifecycle: MessageReader<AppLifecycle>,
    mut link: ResMut<Link>,
    mut transport: Option<ResMut<NetcodeClientTransport>>,
    mut commands: Commands,
) {
    for event in lifecycle.read() {
        match event {
            AppLifecycle::WillSuspend | AppLifecycle::Suspended => {
                if !matches!(link.state, LinkState::Off | LinkState::Outdated | LinkState::Asleep) {
                    close(&mut commands, transport.as_deref_mut());
                    link.state = LinkState::Asleep;
                    info!("in the background: offline until it comes back");
                }
            }
            AppLifecycle::WillResume | AppLifecycle::Running => {
                if link.state == LinkState::Asleep {
                    link.state = LinkState::Resting { until: 0.0 };
                    link.rest = REST_MIN;
                }
            }
            AppLifecycle::Idle => {}
        }
    }
}

/// Keeps trying to connect while not, and notices when the connection goes:
/// everyone else goes with it, and the town is as it was offline.
#[allow(clippy::too_many_arguments)]
fn link(
    time: Res<Time<Real>>,
    mut link: ResMut<Link>,
    client: Res<State<ClientState>>,
    mut transport: Option<ResMut<NetcodeClientTransport>>,
    channels: Res<RepliconChannels>,
    me: Query<(&Transform, &Venue), With<Player>>,
    remote: Query<Entity, With<Remote>>,
    offline_game: Res<OfflineGame>,
    mut online: ResMut<Online>,
    mut clock: ResMut<Clock>,
    mut asks: MessageWriter<Ask>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    let connected = *client.get() == ClientState::Connected;
    let forget = |commands: &mut Commands, online: &mut Online, clock: &mut Clock| {
        for entity in &remote {
            commands.entity(entity).despawn();
        }
        *online = Online::default();
        *clock = Clock::default();
    };
    match link.state {
        LinkState::Off | LinkState::Asleep | LinkState::Outdated => {
            link.answer = None;
            if transport.is_some() {
                close(&mut commands, transport.as_deref_mut());
            }
            if online.is() || !remote.is_empty() {
                forget(&mut commands, &mut online, &mut clock);
            }
        }
        LinkState::Resting { until } => {
            // Not in the middle of a game of House Builder played offline:
            // online, a game is the server's to run. In the middle of one
            // played online, the connection is wanted back at once.
            if now >= until && !offline_game.0 {
                try_address(&mut link, &mut commands, &channels, 0, now);
            }
        }
        LinkState::Asking { nth, since } => {
            let answer = link.answer.as_ref().and_then(|answer| {
                match answer.lock().ok()?.try_recv() {
                    Ok(answer) => Some(answer),
                    Err(TryRecvError::Empty) => None,
                    Err(TryRecvError::Disconnected) => Some(Err("the login went quiet".into())),
                }
            });
            match answer {
                Some(Ok(connect_token)) => {
                    link.answer = None;
                    let authentication = ClientAuthentication::Secure { connect_token };
                    connect(&mut link, &mut commands, &channels, nth, authentication, now);
                }
                Some(Err(error)) => {
                    debug!("{}: {error}", link.servers[nth]);
                    link.answer = None;
                    give_up_on(&mut link, &mut commands, &channels, nth, now);
                }
                None if now - since > LOGIN_WAIT => {
                    link.answer = None;
                    give_up_on(&mut link, &mut commands, &channels, nth, now);
                }
                None => {}
            }
        }
        LinkState::Trying { nth, since } => {
            if connected {
                let Ok((at, &venue)) = me.single() else {
                    return;
                };
                asks.write(Ask::Hello {
                    venue,
                    spot: Spot::of(at.translation, at.rotation),
                });
                link.state = LinkState::Linked;
                link.rest = REST_MIN;
                link.linked_to = Some(link.servers[nth]);
                info!("connected to {}", link.servers[nth]);
                return;
            }
            let wait = if nth + 1 < link.servers.len() {
                TRY_FIRST
            } else {
                TRY_LAST
            };
            if now - since > wait {
                // Never connected: there is nobody to say goodbye to.
                close(&mut commands, None);
                give_up_on(&mut link, &mut commands, &channels, nth, now);
            }
        }
        LinkState::Linked => {
            if link.again {
                // Someone new: the account has just been deleted.
                link.again = false;
                close(&mut commands, transport.as_deref_mut());
                forget(&mut commands, &mut online, &mut clock);
                link.state = LinkState::Resting { until: now };
            } else if !connected {
                info!("disconnected: offline until the server answers again");
                close(&mut commands, transport.as_deref_mut());
                forget(&mut commands, &mut online, &mut clock);
                link.state = LinkState::Resting {
                    until: now + REST_MIN * 0.5,
                };
            }
        }
    }
}

/// Starts trying the server's `nth` address: asking the login there for a
/// token first, on a thread of its own, or, without the login, connecting
/// at once.
fn try_address(
    link: &mut Link,
    commands: &mut Commands,
    channels: &RepliconChannels,
    nth: usize,
    now: f64,
) {
    let server = link.servers[nth];
    if link.secure {
        let (answer, answered) = channel();
        let device = link.device;
        let asked = std::thread::Builder::new()
            .name("login".into())
            .spawn(move || {
                let _ = answer.send(crate::login::token(server, device));
            });
        match asked {
            Ok(_) => {
                link.answer = Some(Mutex::new(answered));
                link.state = LinkState::Asking { nth, since: now };
            }
            Err(error) => {
                debug!("could not ask the login: {error}");
                give_up_on(link, commands, channels, nth, now);
            }
        }
        return;
    }
    connect(link, commands, channels, nth, unsecure(server, link.device), now);
}

/// Connects to the server's `nth` address, getting in as `authentication`
/// says.
fn connect(
    link: &mut Link,
    commands: &mut Commands,
    channels: &RepliconChannels,
    nth: usize,
    authentication: ClientAuthentication,
    now: f64,
) {
    let server = link.servers[nth];
    match open(commands, channels, server, authentication) {
        Ok(()) => link.state = LinkState::Trying { nth, since: now },
        Err(error) => {
            debug!("could not try {server}: {error}");
            give_up_on(link, commands, channels, nth, now);
        }
    }
}

/// The server's `nth` address did not answer: on to the next, or, with none
/// left, a rest before starting again.
fn give_up_on(
    link: &mut Link,
    commands: &mut Commands,
    channels: &RepliconChannels,
    nth: usize,
    now: f64,
) {
    if nth + 1 < link.servers.len() {
        try_address(link, commands, channels, nth + 1, now);
    } else {
        link.state = LinkState::Resting {
            until: now + link.rest,
        };
        link.rest = (link.rest * 2.0).min(REST_MAX);
    }
}

/// The connection broke underneath, or never got going: the network is not
/// there for this address, say, as with IPv6 on a network without it. Given
/// up on at once rather than waited out.
fn transport_failed(
    error: On<NetcodeErrorEvent>,
    mut link: ResMut<Link>,
    time: Res<Time<Real>>,
) {
    if let LinkState::Trying { nth, .. } = link.state {
        debug!("{}: {}", link.servers[nth], *error);
        link.state = LinkState::Trying {
            nth,
            since: time.elapsed_secs_f64() - TRY_LAST - 1.0,
        };
    }
}

/// The server is newer than this app. Online is for the newer app: this one
/// stops trying, and says so.
fn outdated(_: On<ProtocolMismatch>, mut link: ResMut<Link>, mut commands: Commands) {
    if link.state == LinkState::Outdated {
        return;
    }
    link.state = LinkState::Outdated;
    warn!("the server is newer than this app: offline until it is updated");
    crate::hud::announce(
        &mut commands,
        Some("Update the app to play online"),
        "A new version is out",
        crate::hud::CAPTION,
        5.0,
    );
}

// -------------------------------------------------------------- your body

/// What the server tells this device alone: who you are, and where it has put
/// you.
fn hear(
    mut tells: MessageReader<Tell>,
    mut online: ResMut<Online>,
    mut me: Query<(Entity, &mut Seat, &mut Balance), With<Player>>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
) {
    for tell in tells.read() {
        let Ok((you, mut seat, mut balance)) = me.single_mut() else {
            continue;
        };
        match tell {
            Tell::Welcome { me } => {
                info!("online as {}", me.name);
                online.me = Some(me.clone());
            }
            Tell::Arrive {
                venue,
                seat: arrives,
                spot,
                arrival,
            } => {
                online.arrival = *arrival;
                online.venue = Some(*venue);
                seat.0 = usize::from(*arrives);
                match spot {
                    // Let in where you already were: nothing moves.
                    Some(spot) => {
                        let here = bodies.get(you).map(|(_, &on, ..)| on).ok();
                        if here != Some(*venue) {
                            let at = Transform::from_translation(spot.translation())
                                .with_rotation(spot.rotation());
                            lobby::put(&mut bodies, &mut orbit, you, *venue, at);
                        }
                    }
                    None => lobby::send(&mut bodies, &mut orbit, you, *venue, seat.0),
                }
            }
            Tell::PutBack { arrival, spot } => {
                online.arrival = *arrival;
                if let Ok((mut transform, _, mut jump, ..)) = bodies.get_mut(you) {
                    transform.translation = spot.translation();
                    transform.rotation = spot.rotation();
                    *jump = PlayerJump::default();
                }
            }
            Tell::Balance(coins) => {
                balance.set_if_neq(Balance(*coins));
            }
            Tell::Joined { .. } | Tell::Refused(_) => {}
        }
    }
}

/// What the last move sent said, and when it went.
#[derive(Default)]
struct Sent {
    /// Seconds since the last one went.
    since: f32,
    last: Option<Spot>,
    /// Where you were last frame, to tell whether you have stopped.
    was: Option<Spot>,
    /// How many more times to send the place you stopped at, in case the one
    /// that said so was lost.
    again: u8,
}

/// Sends where you are 15 times a second while you move: and once you stop, a
/// couple more times, marked still, in case the first was lost. Nothing while
/// you stand still, and nothing while you are anywhere the server has not
/// put you.
fn tell_where_i_am(
    time: Res<Time>,
    online: Res<Online>,
    me: Query<(&Transform, &Venue), With<Player>>,
    mut sent: Local<Sent>,
    mut moves: MessageWriter<Moved>,
) {
    let Ok((at, &venue)) = me.single() else {
        return;
    };
    if !online.is() || online.venue != Some(venue) {
        *sent = Sent::default();
        return;
    }
    let mut spot = Spot::of(at.translation, at.rotation);
    spot.still = sent.was.is_some_and(|was| was.same(&spot));
    sent.was = Some(Spot { still: spot.still, ..spot });
    sent.since += time.delta_secs();
    if sent.since < (1.0 / TICK_HZ) as f32 {
        return;
    }
    sent.since = 0.0;
    let changed = sent.last.is_none_or(|last| !last.same(&spot));
    if changed {
        sent.again = if spot.still { 2 } else { 0 };
    } else if sent.again > 0 {
        sent.again -= 1;
    } else {
        return;
    }
    sent.last = Some(spot);
    moves.write(Moved {
        arrival: online.arrival,
        spot,
    });
}

// --------------------------------------------------------- everyone else

/// Gives everyone the server has said is in your room a body, standing where
/// it says they are, once both who they are and where have come: replicon
/// puts the parts of an entity in one at a time, and drawn before its place
/// had come, someone stood in the middle of the fountain.
fn dress_newcomers(
    time: Res<Time<Real>>,
    mut clock: ResMut<Clock>,
    newcomers: Query<(Entity, &Member, &Spot), (With<Remote>, Without<Track>)>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    for (newcomer, member, spot) in &newcomers {
        let sent = f64::from(spot.tick) / TICK_HZ;
        clock.heard(sent, time.elapsed_secs_f64());
        let at = Transform::from_translation(spot.translation()).with_rotation(spot.rotation());
        let mut entity = commands.entity(newcomer);
        lobby::dress(&mut entity, &assets, member, at);
        entity.insert(Track {
            updates: VecDeque::from([(sent, *spot)]),
        });
    }
}

/// The server's clock, as near as this device can tell: how far ahead of its
/// own it runs, taken from the update that came quickest, the least held up
/// on the way.
#[derive(Resource, Default)]
struct Clock {
    ahead: Option<f64>,
    /// The real time it was last looked at.
    at: f64,
}

impl Clock {
    /// An update sent at the server's `sent` came in at `now`.
    fn heard(&mut self, sent: f64, now: f64) {
        let ahead = sent - now;
        if self.ahead.is_none() {
            self.at = now;
        }
        self.ahead = Some(self.ahead.map_or(ahead, |was| was.max(ahead)));
    }

    /// The server's time now, about: a little behind it, by the quickest any
    /// update has come.
    fn now(&mut self, now: f64) -> Option<f64> {
        let ahead = self.ahead.as_mut()?;
        *ahead -= (now - self.at).max(0.0) * CLOCK_DRIFT;
        self.at = now;
        Some(now + *ahead)
    }
}

/// Where someone has been, update by update, oldest first: when the server
/// sent it, in its seconds, and where they were.
#[derive(Component, Default)]
struct Track {
    updates: VecDeque<(f64, Spot)>,
}

/// Lays each update about someone out in the time the server sent it. One
/// that takes them somewhere else entirely — another island, or further than
/// a walk — starts their track again, so that they go there at once rather
/// than slide.
fn hear_spots(
    time: Res<Time<Real>>,
    mut clock: ResMut<Clock>,
    mut tracks: Query<(&Spot, Ref<Venue>, &mut Track), (With<Remote>, Changed<Spot>)>,
) {
    let now = time.elapsed_secs_f64();
    for (spot, venue, mut track) in &mut tracks {
        let sent = f64::from(spot.tick) / TICK_HZ;
        clock.heard(sent, now);
        let leap = track.updates.back().is_some_and(|(_, last)| {
            last.translation().distance(spot.translation()) > LEAP
        });
        if venue.is_changed() || leap {
            track.updates.clear();
        }
        match track.updates.back() {
            Some((last, _)) if *last >= sent => {}
            _ => track.updates.push_back((sent, *spot)),
        }
        while track.updates.len() > 2
            && track.updates.get(1).is_some_and(|(second, _)| *second < sent - 1.0)
        {
            track.updates.pop_front();
        }
    }
}

/// Draws everyone else where they were [`BEHIND`] ago in the server's time,
/// between the two updates either side of that moment. Late updates: carried
/// on the way they were going for a moment, then held. Only what has changed
/// is written, so that someone standing still is not taken for someone who
/// moved.
fn smooth(
    time: Res<Time<Real>>,
    mut clock: ResMut<Clock>,
    mut bodies: Query<(&mut Transform, &Track), With<Remote>>,
) {
    let Some(server) = clock.now(time.elapsed_secs_f64()) else {
        return;
    };
    let then = server - BEHIND;
    for (mut transform, track) in &mut bodies {
        if let Some(pose) = pose_at(&track.updates, then) {
            transform.set_if_neq(pose);
        }
    }
}

/// Where the track `updates` has its body at the server's time `then`.
fn pose_at(updates: &VecDeque<(f64, Spot)>, then: f64) -> Option<Transform> {
    let pose = |spot: &Spot| {
        Transform::from_translation(spot.translation()).with_rotation(spot.rotation())
    };
    let (first, last) = (updates.front()?, updates.back()?);
    if then <= first.0 {
        return Some(pose(&first.1));
    }
    if then >= last.0 {
        // Late: on the way it was going, for a moment, unless it had stopped.
        let before = updates.len().checked_sub(2).and_then(|n| updates.get(n));
        return Some(match before {
            Some(before) if !last.1.still && last.0 > before.0 => {
                let over = (then - last.0).min(CARRY_ON) as f32;
                let velocity =
                    (last.1.translation() - before.1.translation()) / (last.0 - before.0) as f32;
                pose(&last.1).with_translation(last.1.translation() + velocity * over)
            }
            _ => pose(&last.1),
        });
    }
    let after = updates.iter().position(|(sent, _)| *sent > then)?;
    let (a, b) = (&updates[after - 1], &updates[after]);
    let share = ((then - a.0) / (b.0 - a.0)) as f32;
    Some(
        Transform::from_translation(a.1.translation().lerp(b.1.translation(), share))
            .with_rotation(a.1.rotation().slerp(b.1.rotation(), share)),
    )
}

#[cfg(test)]
mod tests {
    use bevy::math::Quat;

    use super::*;

    fn spot(x: f32, still: bool) -> Spot {
        Spot {
            still,
            ..Spot::of(Vec3::new(x, 0.0, 0.0), Quat::IDENTITY)
        }
    }

    fn track(updates: &[(f64, Spot)]) -> VecDeque<(f64, Spot)> {
        updates.iter().copied().collect()
    }

    #[test]
    fn between_two_updates_it_is_between_them() {
        let updates = track(&[(1.0, spot(0.0, false)), (1.1, spot(1.0, false))]);
        let x = pose_at(&updates, 1.05).unwrap().translation.x;
        assert!((x - 0.5).abs() < 1e-4, "{x}");
    }

    #[test]
    fn late_it_carries_on_for_a_moment_and_then_holds() {
        let updates = track(&[(1.0, spot(0.0, false)), (1.1, spot(1.0, false))]);
        let x = pose_at(&updates, 1.15).unwrap().translation.x;
        assert!((x - 1.5).abs() < 1e-3, "{x}");
        let x = pose_at(&updates, 5.0).unwrap().translation.x;
        assert!((x - 2.0).abs() < 1e-3, "{x}");
    }

    #[test]
    fn stopped_it_stays_stopped() {
        let updates = track(&[(1.0, spot(0.0, false)), (1.1, spot(1.0, true))]);
        let x = pose_at(&updates, 2.0).unwrap().translation.x;
        assert!((x - 1.0).abs() < 1e-4, "{x}");
    }

    #[test]
    fn the_clock_goes_by_the_quickest_update() {
        let mut clock = Clock::default();
        clock.heard(10.0, 100.05);
        clock.heard(10.1, 100.12);
        clock.heard(10.2, 100.40);
        // The second came quickest: 0.02 s after it was sent, against 0.05
        // and 0.2.
        let server = clock.now(100.12).unwrap();
        assert!((server - 10.1).abs() < 1e-3, "{server}");
    }
}
