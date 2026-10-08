//! Players: letting them in, passing on where they are, and moving them from
//! room to room.
//!
//! Each device that connects is an entity of replicon's, a client. Once it has
//! said where it is ([`Ask::Hello`]) and its protocol has been found to match,
//! it is let in ([`admit`]): the server makes a body for it, a second entity
//! holding everything everyone else in its room is sent about it — its
//! [`Member`] record, its [`Spot`] and its [`Venue`] — and tells the device who
//! it is and where it has been put.
//!
//! The body is the device's to move. Its moves come in as [`Moved`], and each
//! one is checked against the last before it is passed on ([`Pace`]): no
//! faster than a body runs, no higher than a jump, not off the edge of the
//! world. One that fails is dropped and the device is put back where it was.
//! Walking through walls is not checked: the server has no islands.
//!
//! Who a device is, [`Who`], comes with its connection. In netcode's secure
//! mode it is the account its token from the login names, loaded from the
//! database (`accounts`). Without one, for trying the server out, it is the
//! sixteen bytes the device sends of its own. A device that drops and comes
//! back is the same player, and finds its body where it left it, if that was
//! kept: in the middle of a game of House Builder, it waits for them.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::backend::connected_client::NetworkId;
use bevy_replicon_renet::netcode::NetcodeServerTransport;
use roundtown_net::{Ask, Member, Moved, Refusal, Spot, Tell, Venue, arrival};
use sha2::{Digest, Sha256};

use crate::accounts::{Bank, Loaded, Record};
use crate::builder::Games;
use crate::rooms::{ClientRoom, InRoom, NotFor, Room, Towns};

/// How fast a body can go across the ground, in metres a second: the game's
/// `MOVE_SPEED`, with some slack for the odd update arriving early.
const MOST_SPEED: f32 = 7.0 * 1.3;
/// How fast a body can rise, in metres a second: a jump, `JUMP_SPEED`, with
/// slack. Falling is not limited.
const MOST_RISE: f32 = 8.5 * 1.3;
/// How far a body can seem to go between two updates that arrive together, in
/// metres: what is kept in hand for updates that arrive bunched up, after a
/// pause on the way.
const BURST: f32 = 4.0;
/// How far out from the middle of every island a body can be, in metres: the
/// game's `OCEAN_LIMIT`, and a little.
const EDGE_OF_THE_WORLD: f32 = 581.0;

/// The names a player is given, with a number after: plain ASCII, as the
/// game's font is. The game's own AIs go by the same names without one.
const NAMES: &[&str] = &[
    "Mochi", "Dubu", "Hodu", "Bori", "Gamja", "Mandu", "Kimbap", "Yuja", "Maru", "Nabi", "Kong",
    "Podo", "Sagwa", "Gyul", "Hobak", "Bam", "Dalgi", "Haru", "Byeol", "Nuri",
];

/// A new player's name: "Mochi 482".
pub(crate) fn new_name() -> String {
    format!(
        "{} {}",
        NAMES[fastrand::usize(..NAMES.len())],
        fastrand::u16(100..1000)
    )
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Everyone>()
        .add_observer(gone)
        .add_systems(
            PreUpdate,
            (hear, admit, moves)
                .chain()
                .after(ServerSystems::Receive)
                .after(crate::accounts::answers)
                .run_if(in_state(ServerState::Running)),
        );
}

/// Who a player is: their account, in secure mode, or the hash of the
/// device's own sixteen bytes without it. The id everyone else sees them by.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Who(pub u64);

/// Who the device connected as `network` is, from what came with its
/// connection.
fn who(transport: &NetcodeServerTransport, network: &NetworkId, kept: bool) -> Option<Who> {
    let data = transport.user_data(network.get())?;
    if kept {
        // The login put the account in the token it signed.
        let id = u64::from_le_bytes(data[..8].try_into().ok()?);
        return (id != 0).then_some(Who(id));
    }
    let hash = Sha256::digest(&data[..16]);
    Some(Who(u64::from_le_bytes(hash[..8].try_into().ok()?)))
}

/// A device that has said hello, waiting to be let in once its protocol has
/// been checked and its account loaded.
#[derive(Component)]
struct Hello {
    venue: Venue,
    spot: Spot,
}

/// A device whose account is being loaded.
#[derive(Component)]
struct Loading;

/// The body a device plays.
#[derive(Component)]
pub(crate) struct Plays(pub Entity);

/// A player's body, as the server keeps it: alongside what is sent of it.
#[derive(Component)]
pub(crate) struct Body {
    /// The device playing it, while one is connected.
    pub client: Option<Entity>,
    pub who: Who,
    pub room: Room,
    /// Where on the island it arrives: in the town, its seat in its channel.
    pub seat: u8,
    /// How many times the server has moved it. A device's moves say which
    /// arrival they are from, and those from before the last are dropped.
    pub arrival: u8,
    /// The town channel it was last in, to go back to.
    pub town: Option<u16>,
    /// What it has to spend: what the database last said, or without one, all
    /// there is of it.
    pub coins: u32,
    pace: Pace,
}

impl Body {
    /// A body the computer plays, in `seat` of `room`: no device, no coins.
    pub(crate) fn bot(room: Room, seat: u8, now: f64) -> Self {
        Self {
            client: None,
            who: Who(0),
            room,
            seat,
            arrival: 0,
            town: None,
            coins: 0,
            pace: Pace::new(now),
        }
    }
}

/// What a body has in hand to move with: it is earned as time passes, at the
/// fastest a body could go, and spent as it goes. A move that costs more than
/// there is in hand could not have been made.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pace {
    /// When the last move was taken, in the server's seconds.
    at: f64,
    across: f32,
    up: f32,
}

impl Pace {
    fn new(now: f64) -> Self {
        Self {
            at: now,
            across: BURST,
            up: BURST,
        }
    }

    /// Takes the move from `from` to `to` at `now`, if a body could have made
    /// it.
    fn take(&mut self, from: &Spot, to: &Spot, now: f64) -> bool {
        let since = (now - self.at).max(0.0) as f32;
        let across = (self.across + since * MOST_SPEED).min(BURST);
        let up = (self.up + since * MOST_RISE).min(BURST);
        let (a, b) = (from.translation(), to.translation());
        let went = a.xz().distance(b.xz());
        let rose = (b.y - a.y).max(0.0);
        let edge = b.x.abs().max(b.z.abs());
        if went > across || rose > up || edge > EDGE_OF_THE_WORLD {
            return false;
        }
        *self = Self {
            at: now,
            across: across - went,
            up: up - rose,
        };
        true
    }
}

/// Every player's body, by who they are, for as long as the body is kept.
#[derive(Resource, Default)]
pub(crate) struct Everyone(pub HashMap<Who, Entity>);

/// Everything sent to one device.
pub(crate) fn tell(tells: &mut MessageWriter<ToClients<Tell>>, client: Entity, tell: Tell) {
    tells.write(ToClients {
        targets: SendTargets::Single(ClientId::Client(client)),
        message: tell,
    });
}

/// What devices ask, apart from their moves and House Builder.
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<Option<&Plays>, With<ConnectedClient>>,
    mut bodies: Query<(&mut Body, &mut Spot, &mut Venue, &Member)>,
    mut towns: ResMut<Towns>,
    time: Res<Time>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    for ask in asks.read() {
        let Some(client) = ask.client_id.entity() else {
            continue;
        };
        let Ok(plays) = clients.get(client) else {
            continue;
        };
        match (&ask.message, plays) {
            (Ask::Hello { venue, spot }, None) => {
                commands.entity(client).insert(Hello {
                    venue: *venue,
                    spot: *spot,
                });
            }
            (Ask::Travel(to), Some(&Plays(body))) => {
                let Ok((mut body_of, spot, venue, member)) = bodies.get_mut(body) else {
                    continue;
                };
                let (room, seat) = match (to, body_of.room) {
                    (Venue::Home, Room::Town(_)) => {
                        towns.stand(body);
                        (Room::Home(member.id), 0)
                    }
                    (Venue::Town, Room::Home(_)) => {
                        let (channel, seat) = towns.sit(body, body_of.town);
                        body_of.town = Some(channel);
                        (Room::Town(channel), seat)
                    }
                    _ => {
                        tell(&mut tells, client, Tell::Refused(Refusal::Busy));
                        continue;
                    }
                };
                move_to(
                    &mut commands,
                    &mut tells,
                    body,
                    (body_of, spot, venue),
                    room,
                    *to,
                    seat,
                    time.elapsed_secs_f64(),
                );
            }
            _ => {}
        }
    }
}

/// Puts `body` in `room`, on `venue` where `seat` arrives, and tells its
/// device so.
#[allow(clippy::too_many_arguments)]
pub(crate) fn move_to(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    entity: Entity,
    body: (Mut<Body>, Mut<Spot>, Mut<Venue>),
    room: Room,
    to: Venue,
    seat: u8,
    now: f64,
) {
    place(commands, tells, entity, body, room, to, seat, None, now);
}

/// Puts `body` in `room`, on `venue`: where `seat` arrives there, or, with an
/// `at`, standing there. Tells its device so.
#[allow(clippy::too_many_arguments)]
fn place(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    entity: Entity,
    (mut body, mut spot, mut venue): (Mut<Body>, Mut<Spot>, Mut<Venue>),
    room: Room,
    to: Venue,
    seat: u8,
    at: Option<Spot>,
    now: f64,
) {
    *spot = at.unwrap_or_else(|| {
        let (at, turn) = arrival(usize::from(seat));
        Spot::of(at, turn)
    });
    venue.set_if_neq(to);
    body.seat = seat;
    body.arrival = body.arrival.wrapping_add(1);
    body.pace = Pace::new(now);
    if body.room != room {
        body.room = room;
        commands.entity(entity).insert(InRoom(room));
        if let Some(client) = body.client {
            commands.entity(client).insert(ClientRoom(room));
        }
    }
    if let Some(client) = body.client {
        tell(
            tells,
            client,
            Tell::Arrive {
                venue: to,
                seat,
                spot: at,
                arrival: body.arrival,
            },
        );
    }
}

/// The room a device that says it is on `venue` is let into, and its seat
/// there: the town or home, where it already is. Anywhere else, the middle of
/// a game it was playing offline, it is brought to the town.
fn room_for(
    venue: Venue,
    id: u64,
    body: Entity,
    rather: Option<u16>,
    towns: &mut Towns,
) -> (Room, Venue, u8, bool) {
    match venue {
        Venue::Home => (Room::Home(id), Venue::Home, 0, true),
        Venue::Town | Venue::Lobby | Venue::Plot(_) => {
            let (channel, seat) = towns.sit(body, rather);
            (Room::Town(channel), Venue::Town, seat, venue == Venue::Town)
        }
    }
}

/// Lets in every device that has said hello and whose protocol matches: the
/// body it had, if it dropped and came back while that was kept, or a new one
/// where it says it is, once its account has been loaded.
#[allow(clippy::too_many_arguments)]
fn admit(
    hellos: Query<
        (Entity, &Hello, &NetworkId, Has<Loading>),
        (With<AuthorizedClient>, Without<Plays>),
    >,
    transport: Res<NetcodeServerTransport>,
    bank: Res<Bank>,
    mut loaded: MessageReader<Loaded>,
    mut everyone: ResMut<Everyone>,
    mut bodies: Query<(&mut Body, &mut Spot, &mut Venue, &Member)>,
    mut towns: ResMut<Towns>,
    mut games: ResMut<Games>,
    time: Res<Time>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    let mut records: HashMap<u64, Option<Record>> = loaded
        .read()
        .map(|loaded| (loaded.id, loaded.record.clone()))
        .collect();
    for (client, hello, network, loading) in &hellos {
        let Some(who) = who(&transport, network, bank.kept()) else {
            commands.entity(client).despawn();
            continue;
        };

        // Back again: the body is still here.
        if let Some(&body) = everyone.0.get(&who)
            && let Ok((mut kept, spot, venue, member)) = bodies.get_mut(body)
        {
            // Still connected from before, as far as the server knows: that
            // connection is the one that has gone.
            if let Some(old) = kept.client.replace(client)
                && old != client
            {
                commands.entity(old).try_despawn();
            }
            commands
                .entity(client)
                .insert((Plays(body), who, ClientRoom(kept.room)))
                .remove::<(Hello, Loading)>();
            commands.entity(body).insert(NotFor(client));
            let member = member.clone();
            tell(&mut tells, client, Tell::Welcome { me: member.clone() });
            tell(&mut tells, client, Tell::Balance(kept.coins));
            let in_game = matches!(kept.room, Room::Game(_));
            let still_playing = matches!(hello.venue, Venue::Lobby | Venue::Plot(_));
            if in_game && still_playing {
                // Back into the game it was in.
                tell(&mut tells, client, Tell::Joined { seat: kept.seat });
                kept.arrival = kept.arrival.wrapping_add(1);
                kept.pace = Pace::new(now);
                tell(
                    &mut tells,
                    client,
                    Tell::Arrive {
                        venue: *venue,
                        seat: kept.seat,
                        spot: Some(*spot),
                        arrival: kept.arrival,
                    },
                );
                info!("{} is back, in {:?}", member.name, kept.room);
                continue;
            }
            if in_game {
                // It gave the game up while the connection was gone.
                games.walk_out(kept.room, body);
            }
            // Let in where it says it is, as it is.
            let town = kept.town;
            let (room, at, seat, keeps) = room_for(hello.venue, member.id, body, town, &mut towns);
            if let Room::Town(channel) = room {
                kept.town = Some(channel);
            }
            place(
                &mut commands,
                &mut tells,
                body,
                (kept, spot, venue),
                room,
                at,
                seat,
                keeps.then_some(hello.spot),
                now,
            );
            info!("{} is back, in {room:?}", member.name);
            continue;
        }

        // New this time: as the account says, once it has been loaded.
        let record = if bank.kept() {
            match records.remove(&who.0) {
                Some(Some(record)) => record,
                Some(None) => {
                    // No such account: deleted, it may be, since the token.
                    warn!("no account {} to let in", who.0);
                    commands.entity(client).despawn();
                    continue;
                }
                None => {
                    if !loading {
                        bank.load(who.0);
                        commands.entity(client).insert(Loading);
                    }
                    continue;
                }
            }
        } else {
            Record::made_up()
        };
        let member = Member {
            id: who.0,
            name: record.name,
            look: record.look,
            ai: false,
        };
        let body = commands.spawn_empty().id();
        let (room, venue, seat, keeps) = room_for(hello.venue, who.0, body, None, &mut towns);
        let town = match room {
            Room::Town(channel) => Some(channel),
            _ => None,
        };
        let spot = keeps.then_some(hello.spot);
        let placed = spot.unwrap_or_else(|| {
            let (at, turn) = arrival(usize::from(seat));
            Spot::of(at, turn)
        });
        commands.entity(body).insert((
            Replicated,
            member.clone(),
            placed,
            venue,
            InRoom(room),
            NotFor(client),
            Body {
                client: Some(client),
                who,
                room,
                seat,
                arrival: 0,
                town,
                coins: record.coins,
                pace: Pace::new(now),
            },
        ));
        commands
            .entity(client)
            .insert((Plays(body), who, ClientRoom(room)))
            .remove::<(Hello, Loading)>();
        everyone.0.insert(who, body);
        tell(&mut tells, client, Tell::Welcome { me: member.clone() });
        tell(&mut tells, client, Tell::Balance(record.coins));
        tell(
            &mut tells,
            client,
            Tell::Arrive {
                venue,
                seat,
                spot,
                arrival: 0,
            },
        );
        info!("{} joined, in {room:?}", member.name);
    }
}

/// Passes on where each body has moved to, once it is sure the body could
/// have; otherwise puts it back where it was.
fn moves(
    mut moves: MessageReader<FromClient<Moved>>,
    clients: Query<&Plays>,
    mut bodies: Query<(&mut Body, &mut Spot)>,
    time: Res<Time>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    let now = time.elapsed_secs_f64();
    for moved in moves.read() {
        let Some(client) = moved.client_id.entity() else {
            continue;
        };
        let Ok(&Plays(body)) = clients.get(client) else {
            continue;
        };
        let Ok((mut body, mut spot)) = bodies.get_mut(body) else {
            continue;
        };
        // From before the server last moved it: from somewhere else.
        if moved.arrival != body.arrival {
            continue;
        }
        if spot.same(&moved.spot) {
            continue;
        }
        if body.pace.take(&spot, &moved.spot, now) {
            *spot = moved.spot;
        } else {
            body.arrival = body.arrival.wrapping_add(1);
            body.pace = Pace::new(now);
            tell(
                &mut tells,
                client,
                Tell::PutBack {
                    arrival: body.arrival,
                    spot: *spot,
                },
            );
        }
    }
}

/// A device has gone. Its body goes with it, unless it is in the middle of a
/// game of House Builder, which keeps its seat for it.
fn gone(
    remove: On<Remove, ConnectedClient>,
    clients: Query<&Plays>,
    mut bodies: Query<(&mut Body, &Member)>,
    mut everyone: ResMut<Everyone>,
    mut towns: ResMut<Towns>,
    mut commands: Commands,
) {
    let Ok(&Plays(entity)) = clients.get(remove.entity) else {
        return;
    };
    let Ok((mut body, member)) = bodies.get_mut(entity) else {
        return;
    };
    // Taken over by a new connection from the same player.
    if body.client != Some(remove.entity) {
        return;
    }
    body.client = None;
    match body.room {
        Room::Town(_) | Room::Home(_) => {
            towns.stand(entity);
            everyone.0.remove(&body.who);
            commands.entity(entity).despawn();
            info!("{} left", member.name);
        }
        Room::Game(_) => {
            info!("{} dropped out of {:?}", member.name, body.room);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{Quat, Vec3};

    use super::*;

    fn spot(x: f32, y: f32, z: f32) -> Spot {
        Spot::of(Vec3::new(x, y, z), Quat::IDENTITY)
    }

    #[test]
    fn running_is_a_move_a_body_can_make() {
        let mut pace = Pace::new(0.0);
        let mut at = spot(0.0, 0.0, 0.0);
        // Full speed for ten seconds, an update every fifteenth of a second.
        for step in 1..=150 {
            let next = spot(step as f32 * 7.0 / 15.0, 0.0, 0.0);
            assert!(pace.take(&at, &next, f64::from(step) / 15.0), "step {step}");
            at = next;
        }
    }

    #[test]
    fn updates_bunched_up_still_pass() {
        let mut pace = Pace::new(0.0);
        let mut at = spot(0.0, 0.0, 0.0);
        // Running, then three updates held up and arriving at once.
        for step in 1..=15 {
            let next = spot(step as f32 * 7.0 / 15.0, 0.0, 0.0);
            let when = if (8..=10).contains(&step) {
                10.0 / 15.0
            } else {
                f64::from(step) / 15.0
            };
            assert!(pace.take(&at, &next, when), "step {step}");
            at = next;
        }
    }

    #[test]
    fn a_leap_is_not() {
        let mut pace = Pace::new(0.0);
        assert!(!pace.take(&spot(0.0, 0.0, 0.0), &spot(30.0, 0.0, 0.0), 1.0 / 15.0));
        // Twice the speed for a few seconds runs out what is in hand.
        let mut pace = Pace::new(0.0);
        let mut at = spot(0.0, 0.0, 0.0);
        let caught = (1..=60).any(|step| {
            let next = spot(step as f32 * 14.0 / 15.0, 0.0, 0.0);
            let taken = pace.take(&at, &next, f64::from(step) / 15.0);
            at = next;
            !taken
        });
        assert!(caught);
    }

    #[test]
    fn a_jump_is_a_rise_a_body_can_make_and_a_flight_is_not() {
        let mut pace = Pace::new(0.0);
        // Up 1.64 m over the 0.39 s a jump takes to its top.
        assert!(pace.take(&spot(0.0, 0.0, 0.0), &spot(0.0, 1.64, 0.0), 0.39));
        let mut pace = Pace::new(0.0);
        assert!(!pace.take(&spot(0.0, 0.0, 0.0), &spot(0.0, 20.0, 0.0), 0.5));
        // Off the edge of the world.
        let mut pace = Pace::new(0.0);
        assert!(!pace.take(&spot(580.0, 0.0, 0.0), &spot(582.0, 0.0, 0.0), 0.5));
    }

    #[test]
    fn the_fountain_throwing_you_up_is_a_rise_a_body_can_make() {
        // The town's fountain throws whoever stands in its top bowl, 1.4 m
        // up, as high as its jet goes, 4.5 m (the game's `fountain`): up at
        // the speed that takes them that high under the game's gravity, 22
        // m/s², a little faster than a jump, and down at one and a half times
        // that. Moves are sent fifteen times a second; the game moves 120.
        let floor = 1.4f32;
        let (mut y, mut rising) = (floor, (2.0f32 * 22.0 * 4.5).sqrt());
        let mut pace = Pace::new(0.0);
        let mut at = spot(0.0, y, 0.0);
        for step in 1..=30 {
            for _ in 0..8 {
                let dt = 1.0 / 120.0;
                rising -= if rising > 0.0 { 22.0 } else { 33.0 } * dt;
                y = (y + rising * dt).max(floor);
            }
            let next = spot(0.0, y, 0.0);
            assert!(pace.take(&at, &next, f64::from(step) / 15.0), "step {step}, {y} m up");
            at = next;
        }
        assert_eq!(y, floor, "back down in the bowl");
    }
}
