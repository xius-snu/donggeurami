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
//! Walking through walls is not checked: the server has no islands. Nor where
//! the town's cars are, two buses, which every device works out from the
//! town's clock, set by this server's (`clock`): a body riding one moves in
//! the car's own space ([`Spot::aboard`]), checked there as anywhere else,
//! and getting off it, it can be no further from where it got on than the
//! car could have taken it.
//!
//! Who a device is, [`Who`], comes with its connection. In netcode's secure
//! mode it is the account its token from the login names, loaded from the
//! database (`accounts`). Without one, for trying the server out, it is the
//! sixteen bytes the device sends of its own. A device that drops and comes
//! back is the same player, and finds its body where it left it, if that was
//! kept: in the middle of a game of House Builder, it waits for them, and
//! so, since 2026-11-15, does their seat in a game of Bridge Wars
//! (`Arenas::keeps`, `Arenas::returns`), and since 2026-11-18 their seat at
//! a table of one of the four games (`Tables::keeps`, `Tables::returns`).
//!
//! **Who a body is, and who it passes for.** A body is sent as its
//! [`Member`], which is its account's own, as its owner chose it; but a
//! game may have a body pass for someone else for a while, under another
//! name and look (since 2026-11-18: Blend In's hider passes for one of the
//! town's people). So every person's body also keeps its account's own
//! member apart ([`Account`]), and one that passes for someone else is
//! marked so ([`Disguised`]): what the account is told of itself, its name
//! changed, and what is kept, go by the account; and every way out of a
//! game puts the account's own back on the body ([`undisguise`]).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::backend::connected_client::NetworkId;
use bevy_replicon_renet::netcode::NetcodeServerTransport;
use roundtown_net::{Ask, Driver, Member, Moved, Posed, Pose, Refusal, Seated, Spot, Tell, Venue, arrival_of_a_person, bag, cars, land, profile};
use sha2::{Digest, Sha256};

use crate::accounts::{Bank, Loaded, Record};
use crate::arena::{Arenas, Return};
use crate::builder::Games;
use crate::rooms::{ClientRoom, InRoom, NotFor, Room, Towns};
use crate::social::Social;
use crate::tables::{self, Tables};

/// How fast a body can go across the ground, in metres a second: the car
/// anyone can drive, boosted (`cars::DRIVEN_TOP_SPEED`, since 2026-11-05,
/// 40 since 2026-11-06; the game's `MOVE_SPEED`, 7, until then), with some
/// slack for the odd update arriving early. Every body is allowed as much:
/// nothing said a body was in the car until 2026-11-21 ([`Ask::Drive`],
/// [`Body::driving`]), and an app from before still does not say. The
/// number is the rules' now (`cars::MOST_SPEED`), a device going by it too.
const MOST_SPEED: f32 = cars::MOST_SPEED;
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
/// How fast the car can take a body riding it across the island, in metres a
/// second: its top speed, with the same slack as a run's, and a run on top,
/// along it.
const MOST_RIDE: f32 = cars::TOP_SPEED * 1.3 + MOST_SPEED;
/// How far apart two bodies can be, in metres, for a punch of the one to
/// land on the other ([`Ask::Punch`]): the game's reach, 3 m from the
/// puncher's middle to the other's side, and as much again for where each
/// has got to since its device last said.
const PUNCH_REACH: f32 = 6.5;

/// The names a player is given, with a number after: plain ASCII, as
/// every name was until 2026-11-14. The game's own AIs go by the same names
/// without one.
const NAMES: &[&str] = &[
    "Mochi", "Dubu", "Hodu", "Bori", "Gamja", "Mandu", "Kimbap", "Yuja", "Maru", "Nabi", "Kong",
    "Podo", "Sagwa", "Gyul", "Hobak", "Bam", "Dalgi", "Haru", "Byeol", "Nuri",
];

/// How often the town's channels are looked over, in seconds
/// ([`mind_the_town`]).
const MIND_EVERY: f32 = 0.5;

/// A new player's name: "Mochi 482".
pub(crate) fn new_name() -> String {
    format!(
        "{} {}",
        NAMES[fastrand::usize(..NAMES.len())],
        fastrand::u16(100..1000)
    )
}

/// Hearing what devices ask, letting them in and passing their moves on:
/// what comes after it in a tick finds everyone let in that tick with their
/// body.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PlayerSystems;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Everyone>()
        .add_observer(gone)
        .add_observer(body_gone)
        .add_systems(Update, (mind_the_town, gather).chain().run_if(in_state(ServerState::Running)))
        .add_systems(
            PreUpdate,
            (hear, admit, moves)
                .chain()
                .in_set(PlayerSystems)
                .after(ServerSystems::Receive)
                .after(crate::accounts::answers)
                .run_if(in_state(ServerState::Running)),
        )
        // Once everything of the tick that seats anyone or stands them up
        // has run, the games' own too.
        .add_systems(PostUpdate, tell_channels.run_if(in_state(ServerState::Running)));
}

/// A device that has asked to be told of its channel of the town
/// ([`Ask::Channel`], since 2026-11-21): one whose app knows how to read it.
#[derive(Component)]
pub(crate) struct Watching;

/// A device that has asked to be told who in its channel of the town sits
/// or lies on something ([`Ask::Resting`], since 2026-11-22): one whose app
/// knows how to draw it.
#[derive(Component)]
pub(crate) struct Posing;

/// A device that has asked to be told which car each of those in its
/// channel of the town who are in theirs is in ([`Ask::Cars`], since
/// 2026-11-25): one whose app knows the cars there are.
#[derive(Component)]
pub(crate) struct Garaging;

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
    /// In its own car, in the town, as its device last said ([`Ask::Drive`],
    /// since 2026-11-21): for the rest of its channel to see the car.
    pub driving: bool,
    /// Which of its cars that is, as its device last said ([`Ask::Car`],
    /// since 2026-11-25): with none said, the one everyone begins with.
    pub car: Option<String>,
    /// Sitting or lying on something in the town, as its device last said
    /// ([`Ask::Rest`], since 2026-11-22): for the rest of its channel to
    /// see it so.
    pub pose: Option<Pose>,
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
            driving: false,
            car: None,
            pose: None,
            pace: Pace::new(now),
        }
    }

    /// A person's body, played by the device `client`, the account `who`,
    /// in `seat` of `room`, with `coins` to spend, just arrived.
    pub(crate) fn person(client: Entity, who: Who, room: Room, seat: u8, town: Option<u16>, coins: u32, now: f64) -> Self {
        Self {
            client: Some(client),
            who,
            room,
            seat,
            arrival: 0,
            town,
            coins,
            driving: false,
            car: None,
            pose: None,
            pace: Pace::new(now),
        }
    }

    /// The server has put the body where it stands now: whatever its device
    /// sent from before is from somewhere else, and dropped ([`moves`]), and
    /// what it has in hand to move with is counted afresh. The arrival its
    /// device is to send with its moves from now on.
    pub(crate) fn arrive(&mut self, now: f64) -> u8 {
        self.arrival = self.arrival.wrapping_add(1);
        self.pace = Pace::new(now);
        self.arrival
    }
}

/// The account a person's body is, as its owner chose it, kept apart from
/// the [`Member`] the body is sent as (since 2026-11-18): the two are one
/// but while a game has the body pass for someone else ([`Disguised`]).
/// Put on every person's body as it is let in ([`admit`]), and changed with
/// the name and the look ([`Ask::Profile`]). What the account's own device
/// is told of who it is ([`Tell::Welcome`]) and what is kept of it go by
/// this.
#[derive(Component, Clone, PartialEq, Debug)]
pub(crate) struct Account(pub Member);

/// The body passes for someone else, a game having put another [`Member`] on
/// it (Blend In's hider among the town's people, and every seeker too):
/// until the game takes it off, the account's own name and look are kept
/// off it, chosen anew or not ([`Ask::Profile`]); and every way out of a
/// game takes it off ([`undisguise`]).
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Disguised;

/// Every person's account, apart from who their body passes for.
pub(crate) type Accounts<'w, 's> = Query<'w, 's, (&'static Account, Has<Disguised>)>;

/// Puts the account's own member back on `body` ([`Account`]) if a game has
/// it passing for someone else ([`Disguised`]), as it leaves the game,
/// however it leaves; and nothing if not, so that nobody in its room is
/// sent it again for nothing.
pub(crate) fn undisguise(commands: &mut Commands, body: Entity, accounts: &Accounts) {
    if let Ok((account, true)) = accounts.get(body) {
        commands.entity(body).insert(account.0.clone()).remove::<Disguised>();
    }
}

/// Who `body`, which wears `worn`, is by its account, going into another
/// game: the account's own member, put back on the body if a game had it
/// passing for someone else ([`undisguise`]), for the game it goes into to
/// seat it as. Someone out of a game of the four before it is over may go
/// straight from it into Bridge Wars or House Builder (`tables::Seat::out`,
/// since 2026-11-18), in whatever it had them wear.
pub(crate) fn themselves(commands: &mut Commands, body: Entity, accounts: &Accounts, worn: &Member) -> Member {
    undisguise(commands, body, accounts);
    accounts.get(body).map_or_else(|_| worn.clone(), |(account, _)| account.0.clone())
}

/// Tells `body`'s device to stand where the server has it, `spot` on
/// `venue`, as a fresh arrival: where it is to begin a game afresh from
/// ([`Tell::Arrive`]), and back there if it is anywhere else
/// ([`Tell::PutBack`]).
pub(crate) fn stand_where_it_is(tells: &mut MessageWriter<ToClients<Tell>>, body: &mut Body, spot: Spot, venue: Venue, now: f64) {
    let arrival = body.arrive(now);
    let Some(client) = body.client else {
        return;
    };
    tell(
        tells,
        client,
        Tell::Arrive {
            venue,
            seat: body.seat,
            spot: Some(spot),
            arrival,
        },
    );
    tell(tells, client, Tell::PutBack { arrival, spot });
}

/// What a body has in hand to move with: it is earned as time passes, at the
/// fastest a body could go, and spent as it goes. A move that costs more than
/// there is in hand could not have been made.
///
/// On the car, the same goes in the car's own space: walking and jumping on
/// it are walks and jumps. Getting on, the body can be anywhere on it, as the
/// server cannot tell where the car is; getting off, it can be no further
/// from where it got on than the car could have taken it in the time
/// between, and it run.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pace {
    /// When the last move was taken, in the server's seconds.
    at: f64,
    across: f32,
    up: f32,
    /// Where on the island the body got onto the car it is riding, and when:
    /// `None` off it, or on it since before the server knew it.
    boarded: Option<(Vec3, f64)>,
}

impl Pace {
    fn new(now: f64) -> Self {
        Self {
            at: now,
            across: BURST,
            up: BURST,
            boarded: None,
        }
    }

    /// Takes the move from `from` to `to` at `now`, if a body could have made
    /// it.
    fn take(&mut self, from: &Spot, to: &Spot, now: f64) -> bool {
        let since = (now - self.at).max(0.0) as f32;
        let across = (self.across + since * MOST_SPEED).min(BURST);
        let up = (self.up + since * MOST_RISE).min(BURST);
        let (a, b) = (from.translation(), to.translation());
        let on_the_car = b.length() <= cars::REACH;
        let in_the_world = b.x.abs().max(b.z.abs()) <= EDGE_OF_THE_WORLD;
        let mut spent = (0.0, 0.0);
        match (from.aboard, to.aboard) {
            // Getting on: anywhere on the car.
            (None, Some(_)) => {
                if !on_the_car {
                    return false;
                }
                self.boarded = Some((a, now));
            }
            // From one car onto another, the same.
            (Some(was), Some(is)) if was != is => {
                if !on_the_car {
                    return false;
                }
            }
            // Getting off: as far from where it got on as the car could have
            // gone since.
            (Some(_), None) => {
                let ridden = self.boarded.is_none_or(|(on, when)| {
                    let far = MOST_RIDE * (now - when).max(0.0) as f32 + BURST;
                    b.xz().distance(on.xz()) <= far
                });
                if !ridden || !in_the_world {
                    return false;
                }
                self.boarded = None;
            }
            // Within one space, the island's or the car's: a walk, a jump or
            // a fall, and on the car, never far from it.
            (_, is) => {
                let went = a.xz().distance(b.xz());
                let rose = (b.y - a.y).max(0.0);
                let kept = if is.is_some() { on_the_car } else { in_the_world };
                if went > across || rose > up || !kept {
                    return false;
                }
                spent = (went, rose);
            }
        }
        self.at = now;
        self.across = across - spent.0;
        self.up = up - spent.1;
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
#[allow(clippy::too_many_arguments)]
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<Option<&Plays>, With<ConnectedClient>>,
    mut bodies: Query<(&mut Body, &mut Spot, &mut Venue, &Member)>,
    accounts: Accounts,
    bags: Query<&crate::bag::Holds>,
    mut towns: ResMut<Towns>,
    bank: Res<Bank>,
    mut social: ResMut<Social>,
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
                    // A game of Bridge Wars is its own to leave (`arena`),
                    // and so is a table (`tables`).
                    (_, Room::Arena(_) | Room::Table(..)) => continue,
                    // Home: a room of your own.
                    (Venue::Home, Room::Town(_)) => {
                        towns.away(body);
                        (Room::Home(member.id), 0)
                    }
                    (Venue::Home, Room::Home(_)) => (Room::Home(member.id), 0),
                    (Venue::Town, Room::Home(_)) => {
                        let (channel, seat) = towns.sit(body, body_of.who.0, body_of.town);
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
            // Who they are, as they chose: kept with the account, and told
            // to everyone in the room by their member record being
            // replaced, and to them again as a welcome. Not while a game
            // has them passing for someone else ([`Disguised`]): then the
            // account has it, and the body is sent it as the game ends.
            (Ask::Profile { name, look }, Some(&Plays(body))) => {
                let (Some(name), true) = (profile::name(name), profile::look(look)) else {
                    warn!("{client:?} sent a name or a look that is no good: {name:?}, {look:?}");
                    continue;
                };
                let Ok((_, _, _, member)) = bodies.get(body) else {
                    continue;
                };
                let (account, disguised) = match accounts.get(body) {
                    Ok((account, disguised)) => (account.0.clone(), disguised),
                    Err(_) => (member.clone(), false),
                };
                // The tag that goes with the name: their own, unless
                // someone of that name has it, and then another (`social`),
                // so that no two accounts have both the same.
                let Some(tag) = social.rename(account.id, &name) else {
                    warn!("{} cannot be called {name}: every tag of that name is taken", account.id);
                    continue;
                };
                let member = Member {
                    name,
                    look: look.clone(),
                    ..account
                };
                info!("{} is now {}#{tag}, a {}", member.id, member.name, member.look);
                bank.profile(member.id, &member.name, &member.look, tag);
                commands.entity(body).insert(Account(member.clone()));
                if !disguised {
                    commands.entity(body).insert(member.clone());
                }
                // The name on their plot's sign is to be the new one
                // ([`Tell::Plots`], once the body has it).
                if let Some(channel) = towns.channel_of(body) {
                    towns.stir(channel);
                }
                tell(&mut tells, client, Tell::Welcome { me: member });
            }
            // Fallen off the town into the void: back where its seat
            // arrives, as a fresh arrival, so that the leap back is not a
            // move to be checked and undone.
            (Ask::Respawn, Some(&Plays(body))) => {
                let Ok((body_of, spot, venue, _)) = bodies.get_mut(body) else {
                    continue;
                };
                let (room, seat) = (body_of.room, body_of.seat);
                // In a game of Bridge Wars a fall is a death, which the
                // game answers (`arena`); at a table, its game answers it
                // as it will (`tables`).
                if matches!(room, Room::Arena(_) | Room::Table(..)) {
                    continue;
                }
                if !matches!(room, Room::Town(_)) {
                    tell(&mut tells, client, Tell::Refused(Refusal::Busy));
                    continue;
                }
                move_to(
                    &mut commands,
                    &mut tells,
                    body,
                    (body_of, spot, venue),
                    room,
                    Venue::Town,
                    seat,
                    time.elapsed_secs_f64(),
                );
            }
            // A punch landed on someone in the room: their device is told
            // to knock them back from the puncher, if the two are near
            // enough for it where each last said it was. On a bus, each is
            // a place on the bus, and off it on the island: in different
            // spaces there is no telling, and the punch is taken as it is.
            (Ask::Punch { who, running }, Some(&Plays(body))) => {
                let Ok((hitter, from, on, member)) = bodies.get(body) else {
                    continue;
                };
                let (room, from, on, by) = (hitter.room, *from, *on, member.id);
                // In a game of Bridge Wars the punch wounds as well, and
                // the game deals it (`arena`); at a table, its game judges
                // every punch, on people and on the computer's alike
                // (`tables`).
                if matches!(room, Room::Arena(_) | Room::Table(..)) {
                    continue;
                }
                let victim = bodies.iter().find_map(|(other, spot, venue, member)| {
                    (member.id == *who && other.room == room && *venue == on)
                        .then_some((other.client, *spot))
                });
                let Some((Some(their_device), spot)) = victim else {
                    continue;
                };
                let near = match (from.aboard, spot.aboard) {
                    (a, b) if a == b => from.translation().xz().distance(spot.translation().xz()) <= PUNCH_REACH,
                    _ => true,
                };
                if !near {
                    continue;
                }
                tell(
                    &mut tells,
                    their_device,
                    Tell::Knocked {
                        by,
                        from,
                        running: *running,
                    },
                );
            }
            // An app that reads what is said of its channel of the town:
            // told from here on ([`tell_channels`]), and at once if it is
            // in one already. Said before it is let in, as a rule, and its
            // being seated is what tells it first.
            (Ask::Channel, plays) => {
                commands.entity(client).insert(Watching);
                if let Some(&Plays(body)) = plays
                    && let Ok((body_of, ..)) = bodies.get(body)
                    && let Room::Town(channel) = body_of.room
                {
                    towns.stir(channel);
                }
            }
            // Into its own car, or out of it: in the town alone, where the
            // car is to be had, and the rest of its channel is told. Into
            // it only if it owns that car (`bag`, since 2026-11-21): the one
            // it said ([`Ask::Car`]), or the one everyone begins with, which
            // everyone has who has not sold it.
            (Ask::Drive(on), Some(&Plays(body))) => {
                let Ok((mut body_of, ..)) = bodies.get_mut(body) else {
                    continue;
                };
                let car = body_of.car.clone().unwrap_or_else(|| bag::CAR.to_string());
                if *on && bags.get(body).is_ok_and(|holds| holds.bag.count(&car) == 0) {
                    continue;
                }
                if let Room::Town(channel) = body_of.room
                    && body_of.driving != *on
                {
                    body_of.driving = *on;
                    towns.stir(channel);
                }
            }
            // An app that draws who sits and who lies: told from here on
            // ([`tell_channels`]), and at once if it is in a channel
            // already, as of the channel itself.
            (Ask::Resting, plays) => {
                commands.entity(client).insert(Posing);
                if let Some(&Plays(body)) = plays
                    && let Ok((body_of, ..)) = bodies.get(body)
                    && let Room::Town(channel) = body_of.room
                {
                    towns.stir(channel);
                }
            }
            // Sat or lain down on something, or up again: in the town
            // alone, where there is anything to rest on, and the rest of
            // its channel is told. Where it rests is where its body is,
            // which its moves say as they always did.
            (Ask::Rest(pose), Some(&Plays(body))) => {
                let Ok((mut body_of, ..)) = bodies.get_mut(body) else {
                    continue;
                };
                if let Room::Town(channel) = body_of.room
                    && body_of.pose != *pose
                {
                    body_of.pose = *pose;
                    towns.stir(channel);
                }
            }
            // An app that knows the cars there are: told from here on which
            // car each driver in its channel is in ([`tell_channels`]), and
            // at once if it is in a channel already.
            (Ask::Cars, plays) => {
                commands.entity(client).insert(Garaging);
                if let Some(&Plays(body)) = plays
                    && let Ok((body_of, ..)) = bodies.get(body)
                    && let Room::Town(channel) = body_of.room
                {
                    towns.stir(channel);
                }
            }
            // Which car it is in, or is getting into: a car there is, and
            // the rest of its channel is told if it is in it now. Whether
            // it owns it is asked as it gets in ([`Ask::Drive`]).
            (Ask::Car(car), Some(&Plays(body))) => {
                let Ok((mut body_of, ..)) = bodies.get_mut(body) else {
                    continue;
                };
                if bag::thing(car).is_none_or(|thing| thing.group != bag::Group::Vehicles) {
                    continue;
                }
                if body_of.car.as_deref() != Some(car.as_str()) {
                    body_of.car = Some(car.clone());
                    if body_of.driving
                        && let Room::Town(channel) = body_of.room
                    {
                        towns.stir(channel);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Tells every device in a channel of the town that something has changed
/// in, and that asked to be told ([`Watching`]), of everyone in it: who
/// is in the town on which plot, and who is in their own car
/// ([`Tell::Channel`]), the whole channel each time, 48 at the most;
/// and whose every plot of the channel is ([`Tell::Plots`], since
/// 2026-11-23: all 48, those away in a game and those nobody here has
/// among them, by the name each account goes by). And each that asked that
/// too ([`Posing`]), who in it sits or lies on something
/// ([`Tell::Resting`], [`resting_in`]); and each that asked that
/// ([`Garaging`], since 2026-11-25), which car each driver is in
/// ([`Tell::Cars`], [`cars_in`]).
#[allow(clippy::too_many_arguments)]
pub(crate) fn tell_channels(
    mut towns: ResMut<Towns>,
    bodies: Query<(&Body, &Member)>,
    accounts: Query<&Account>,
    watching: Query<(), With<Watching>>,
    posing: Query<(), With<Posing>>,
    garaging: Query<(), With<Garaging>>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    // Looked at before it is taken: nothing is written on a tick nothing
    // has changed in.
    if !towns.is_stirred() {
        return;
    }
    for channel in towns.take_stirred() {
        let seated = towns.seated(channel);
        let (everyone, devices) = channel_of(&seated, |body| bodies.get(body).ok());
        let resting = resting_in(&seated, |body| bodies.get(body).ok());
        let cars = cars_in(&seated, |body| bodies.get(body).ok());
        let named = |body: Entity| accounts.get(body).ok().map(|account| (account.0.name.clone(), account.0.look.clone()));
        let deeds = towns.deeds(channel, named);
        for device in devices {
            if watching.contains(device) {
                tell(&mut tells, device, Tell::Channel(everyone.clone()));
                tell(&mut tells, device, Tell::Plots(deeds.clone()));
            }
            if posing.contains(device) {
                tell(&mut tells, device, Tell::Resting(resting.clone()));
            }
            if garaging.contains(device) {
                tell(&mut tells, device, Tell::Cars(cars.clone()));
            }
        }
    }
}

/// Everyone seated in a channel who is in their own car, and which, as
/// they are told of ([`Driver`]), given each taken seat and the body in it
/// and what is known of a body: the car each said, or the one everyone
/// begins with.
fn cars_in<'a>(seated: &[(u8, Entity)], known: impl Fn(Entity) -> Option<(&'a Body, &'a Member)>) -> Vec<Driver> {
    seated
        .iter()
        .filter_map(|&(_, body)| known(body))
        .filter(|(body, _)| body.driving)
        .map(|(body, member)| Driver {
            who: member.id,
            car: body.car.clone().unwrap_or_else(|| bag::CAR.to_string()),
        })
        .collect()
}

/// Everyone seated in a channel who sits or lies on something, as they are
/// told of ([`Posed`]), given each taken seat and the body in it and what
/// is known of a body. Nobody, most of the time.
fn resting_in<'a>(seated: &[(u8, Entity)], known: impl Fn(Entity) -> Option<(&'a Body, &'a Member)>) -> Vec<Posed> {
    seated
        .iter()
        .filter_map(|&(_, body)| known(body))
        .filter_map(|(body, member)| Some(Posed { who: member.id, pose: body.pose? }))
        .collect()
}

/// Everyone seated in a channel, as they are told of ([`Seated`]), and the
/// devices playing them, given each taken seat and the body in it and what
/// is known of a body. A body that is gone is left out.
fn channel_of<'a>(
    seated: &[(u8, Entity)],
    known: impl Fn(Entity) -> Option<(&'a Body, &'a Member)>,
) -> (Vec<Seated>, Vec<Entity>) {
    let mut everyone = Vec::new();
    let mut devices = Vec::new();
    for &(seat, body) in seated {
        let Some((body, member)) = known(body) else {
            continue;
        };
        everyone.push(Seated {
            who: member.id,
            seat,
            driving: body.driving,
        });
        devices.extend(body.client);
    }
    (everyone, devices)
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
pub(crate) fn place(
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
    // Everyone the server puts anywhere is a person: in the town, in the
    // middle of their own plot (`land`, since 2026-11-20).
    *spot = at.unwrap_or_else(|| {
        let (at, turn) = arrival_of_a_person(to, usize::from(seat));
        Spot::of(at, turn)
    });
    venue.set_if_neq(to);
    body.seat = seat;
    body.arrival = body.arrival.wrapping_add(1);
    body.pace = Pace::new(now);
    if body.room != room {
        body.room = room;
        // Out of its own car, which is the town's alone and which its
        // device gets out of as it is sent to any other island; from one
        // channel of the town to another it stays in it. And up from
        // whatever it sat or lay on, which stayed where it was.
        if !matches!(room, Room::Town(_)) {
            body.driving = false;
        }
        body.pose = None;
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
/// there: the town or home, where it already is. Anywhere else, the middle
/// of a game of House Builder or Bridge Wars, or of one of the four played
/// at tables (2026-11-18), that it was playing offline, it is brought to
/// the town: online, a game is the server's, and whoever is back for a game
/// of the server's finds it by their body, which the game kept for them
/// ([`admit`]).
fn room_for(
    venue: Venue,
    id: u64,
    body: Entity,
    rather: Option<u16>,
    towns: &mut Towns,
) -> (Room, Venue, u8, bool) {
    match venue {
        Venue::Home => (Room::Home(id), venue, 0, true),
        Venue::Town
        | Venue::Lobby
        | Venue::Plot(_)
        | Venue::Arena
        | Venue::BlendIn
        | Venue::KnockOff
        | Venue::BridgeRace
        | Venue::LastStand => {
            let (channel, seat) = towns.sit(body, id, rather);
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
    accounts: Accounts,
    mut towns: ResMut<Towns>,
    mut games: ResMut<Games>,
    mut arenas: ResMut<Arenas>,
    mut tables: ResMut<Tables>,
    mut social: ResMut<Social>,
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
            && let Ok((mut kept, mut spot, venue, member)) = bodies.get_mut(body)
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
            // This device has heard nothing of its friends yet, though the
            // account may never have been seen to leave.
            social.connected(who.0);
            let member = member.clone();
            // Who the account is, whoever a game has its body passing for
            // ([`Account`]): told the disguise, the device would take it
            // for a name it had not chosen, and change the account's to
            // its own again ([`Ask::Profile`]), over the disguise.
            let me = accounts.get(body).map_or_else(|_| member.clone(), |(account, _)| account.0.clone());
            tell(&mut tells, client, Tell::Welcome { me });
            tell(&mut tells, client, Tell::Balance(kept.coins));
            let in_game = matches!(kept.room, Room::Game(_) | Room::Arena(_) | Room::Table(..));
            // A table of one of the four games kept the body in its seat,
            // and says what becomes of whoever is back for it, as Bridge
            // Wars does (`Tables::returns`): carrying on where it stands,
            // put back in the game, which its table begins afresh on it
            // (`tables`, as it next runs), or given up, below.
            if let Room::Table(game, _) = kept.room {
                match tables.returns(kept.room, body, hello.venue == game.venue()) {
                    // The app was only put away: it carries on, where it
                    // says it stands if that is where the server had it or
                    // a step from there, and otherwise where the server has
                    // it, as in Bridge Wars.
                    tables::Return::Resume => {
                        let near = hello.spot.aboard.is_none()
                            && hello.spot.translation().distance(spot.translation()) <= BURST;
                        if near {
                            *spot = hello.spot;
                            let arrival = kept.arrive(now);
                            tell(
                                &mut tells,
                                client,
                                Tell::Arrive {
                                    venue: *venue,
                                    seat: kept.seat,
                                    spot: Some(*spot),
                                    arrival,
                                },
                            );
                        } else {
                            stand_where_it_is(&mut tells, &mut kept, *spot, *venue, now);
                        }
                        info!("{} is back, in {:?}, and carries on", member.name, kept.room);
                        continue;
                    }
                    tables::Return::Back { seat } => {
                        info!("{} is back, put back in seat {seat} of {:?}", member.name, kept.room);
                        continue;
                    }
                    tables::Return::GiveUp => {}
                }
            }
            // A game of Bridge Wars kept the body in its seat, and says
            // what becomes of whoever is back for it (`Arenas::returns`).
            if let Room::Arena(_) = kept.room {
                match arenas.returns(kept.room, body, hello.venue == Venue::Arena) {
                    // The app was only put away: it carries on, where it
                    // says it stands if that is where the server had it or
                    // a step from there, and otherwise where the server
                    // has it, the body having stood again on its island
                    // since.
                    Return::Resume => {
                        let near = hello.spot.aboard.is_none()
                            && hello.spot.translation().distance(spot.translation()) <= BURST;
                        if near {
                            *spot = hello.spot;
                        }
                        kept.arrival = kept.arrival.wrapping_add(1);
                        kept.pace = Pace::new(now);
                        tell(
                            &mut tells,
                            client,
                            Tell::Arrive {
                                venue: Venue::Arena,
                                seat: kept.seat,
                                spot: Some(*spot),
                                arrival: kept.arrival,
                            },
                        );
                        if !near {
                            tell(
                                &mut tells,
                                client,
                                Tell::PutBack {
                                    arrival: kept.arrival,
                                    spot: *spot,
                                },
                            );
                        }
                        info!("{} is back, in {:?}, and carries on", member.name, kept.room);
                        continue;
                    }
                    // The app was closed and opened again, or its player
                    // died while it was away: the game begun afresh on
                    // it, standing where the body stands.
                    Return::Back { seat, metres, compass } => {
                        tell(&mut tells, client, Tell::Back { seat, metres, compass });
                        kept.arrival = kept.arrival.wrapping_add(1);
                        kept.pace = Pace::new(now);
                        tell(
                            &mut tells,
                            client,
                            Tell::Arrive {
                                venue: Venue::Arena,
                                seat: kept.seat,
                                spot: Some(*spot),
                                arrival: kept.arrival,
                            },
                        );
                        tell(
                            &mut tells,
                            client,
                            Tell::PutBack {
                                arrival: kept.arrival,
                                spot: *spot,
                            },
                        );
                        info!("{} is back, put back in seat {seat} of {:?}", member.name, kept.room);
                        continue;
                    }
                    // Nothing to come back to: the seat is given up, below.
                    Return::GiveUp => {}
                }
            }
            let still_playing = matches!(kept.room, Room::Game(_)) && matches!(hello.venue, Venue::Lobby | Venue::Plot(_));
            if in_game && still_playing {
                // Back into the game of House Builder it was in.
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
                // It gave the game up while the connection was gone, and is
                // whoever its account is again.
                games.walk_out(kept.room, body);
                arenas.walk_out(kept.room, body);
                tables.walk_out(kept.room, body);
                undisguise(&mut commands, body, &accounts);
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
            // Nothing is kept: whoever has been here since the server
            // started is who they were then, by their name, their tag and
            // what they have earned (`social`); anyone else is made up.
            match social.person(who.0) {
                Some(person) => Record {
                    name: person.name.clone(),
                    tag: Some(person.tag),
                    xp: person.xp,
                    ..Record::made_up()
                },
                None => Record::made_up(),
            }
        };
        // Known to their friends by this name and tag from here on: the
        // account's own, or, with none yet, one given now.
        let tag = social.enroll(who.0, &record.name, record.tag, record.xp);
        // What they own, and what of it stands on their plot (`bag`).
        let holds = crate::bag::Holds::of(&record);
        let member = Member {
            id: who.0,
            name: record.name,
            look: record.look,
            ai: false,
        };
        let body = commands.spawn_empty().id();
        // To the channel their group is in, if they are in one and it has
        // room (Hajun, 2026-11-23: "they should move to the same channel").
        let with = social
            .mates(who.0)
            .into_iter()
            .find_map(|mate| everyone.0.get(&Who(mate)).and_then(|&mate| towns.channel_of(mate)));
        let (room, venue, seat, keeps) = room_for(hello.venue, who.0, body, with, &mut towns);
        let town = match room {
            Room::Town(channel) => Some(channel),
            _ => None,
        };
        // Let in where it stands; but a device in the town that has only
        // just opened stands in the middle of the plot of the seat it
        // guessed at, the first, and is stood on the plot of the seat it
        // has (`land::just_arrived`): where it stood is someone else's.
        let stands = hello.spot.translation();
        let guessed = venue == Venue::Town && hello.spot.aboard.is_none() && land::just_arrived(stands.xz());
        let spot = (keeps && !guessed).then_some(hello.spot);
        let placed = spot.unwrap_or_else(|| {
            let (at, turn) = arrival_of_a_person(venue, usize::from(seat));
            Spot::of(at, turn)
        });
        commands.entity(body).insert((
            Replicated,
            member.clone(),
            Account(member.clone()),
            placed,
            venue,
            InRoom(room),
            NotFor(client),
            Body::person(client, who, room, seat, town, record.coins, now),
            holds,
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
        info!("{}#{tag} joined, in {room:?}", member.name);
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
/// game of House Builder, which keeps its seat for it, or in a game of
/// Bridge Wars, which does too (`Arenas::keeps`, since 2026-11-15): the
/// body stands in the game where it stood, to be hit like anyone, until its
/// device is back ([`admit`]) or the game is over. Until then a game of
/// Bridge Wars went on without it and its seat was given up for good,
/// however short a while the device was gone; and a phone's is gone
/// whenever its app leaves the foreground (the game's `net::lifecycle`).
/// A seat at a table of one of the four games waits as Bridge Wars' does,
/// once its game is being played (`Tables::keeps`, since 2026-11-18);
/// before then it is given up, and its fee given back.
#[allow(clippy::too_many_arguments)]
fn gone(
    remove: On<Remove, ConnectedClient>,
    clients: Query<&Plays>,
    mut bodies: Query<(&mut Body, &Member)>,
    mut everyone: ResMut<Everyone>,
    mut arenas: ResMut<Arenas>,
    mut tables: ResMut<Tables>,
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
        Room::Arena(_) if arenas.keeps(body.room, entity) => {
            info!("{} dropped out of {:?}: their seat waits for them", member.name, body.room);
        }
        Room::Table(..) if tables.keeps(body.room, entity) => {
            info!("{} dropped out of {:?}: their seat waits for them", member.name, body.room);
        }
        Room::Town(_) | Room::Home(_) | Room::Arena(_) | Room::Table(..) => {
            if let Room::Arena(_) = body.room {
                arenas.walk_out(body.room, entity);
            }
            if let Room::Table(..) = body.room {
                tables.walk_out(body.room, entity);
            }
            // Its plot of the town is kept a while ([`body_gone`]).
            everyone.0.remove(&body.who);
            commands.entity(entity).despawn();
            info!("{} left", member.name);
        }
        Room::Game(_) => {
            info!("{} dropped out of {:?}", member.name, body.room);
        }
    }
}

/// A body is gone, however it went: whatever plot of the town it had is
/// kept as it leaves it for a while (`Towns::leave`, `land::GRACE`), with
/// what stands on it as its bag has it now, so that someone whose phone
/// only put the app away finds their land where it was, and so does
/// everyone else in the channel meanwhile.
fn body_gone(
    remove: On<Remove, Body>,
    bodies: Query<(Option<&Account>, Option<&crate::bag::Holds>)>,
    mut towns: ResMut<Towns>,
) {
    let lot = bodies
        .get(remove.entity)
        .ok()
        .and_then(|(account, holds)| Some(holds?.lot(&account?.0)));
    towns.leave(remove.entity, lot);
}

/// Looks the town's channels over, [`MIND_EVERY`] seconds: lets go of the
/// plots kept long enough for someone who went (`Towns::tick`), and says
/// which plots someone is standing on, none of which whoever comes next
/// takes from under them (Hajun, 2026-11-23: "unless there is a real
/// player standing on that plot, then replace the next number plot on the
/// priority").
fn mind_the_town(time: Res<Time>, mut since: Local<f32>, mut towns: ResMut<Towns>, bodies: Query<(&Body, &Spot, &Venue)>) {
    *since += time.delta_secs();
    if *since < MIND_EVERY {
        return;
    }
    *since = 0.0;
    towns.tick(time.elapsed_secs_f64());
    let standing = bodies
        .iter()
        .filter_map(|(body, spot, venue)| {
            let Room::Town(channel) = body.room else {
                return None;
            };
            if *venue != Venue::Town || spot.aboard.is_some() {
                return None;
            }
            Some((channel, land::plot_at(spot.translation().xz())?))
        })
        .collect();
    towns.stood_on(standing);
}

/// Brings a group together in one channel of the town as someone comes
/// into it (`Social::take_gathered`; Hajun, 2026-11-23: "if u invite
/// someone to ur group, they should move to the same channel (if they are
/// already on the same channel it doesnt matter), but they should first
/// try to move to the channel of the leader, but if its full with 16
/// players, then maybe it should go to another channel with 2+ empty spots
/// so that they are on same channel"): whoever of it has a plot in another
/// channel than its leader is moved to the leader's, their land with them;
/// or, that one having no room for them, all of them, the leader too, to
/// the fullest channel that has room for the whole group, or a new one.
/// Whoever is in the town is stood on their new plot and told; whoever is
/// away in a game finds theirs there when they are back.
#[allow(clippy::too_many_arguments)]
fn gather(
    mut social: ResMut<Social>,
    everyone: Res<Everyone>,
    mut towns: ResMut<Towns>,
    mut bodies: Query<(&mut Body, &mut Spot, &mut Venue)>,
    lots: Query<(&Account, &crate::bag::Holds)>,
    time: Res<Time>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    for (leader, members) in social.take_gathered() {
        let body_of = |who: u64| everyone.0.get(&Who(who)).copied();
        let Some(target) = body_of(leader).and_then(|body| towns.channel_of(body)) else {
            continue;
        };
        // Everyone of the group who has a plot anywhere, and those of them
        // who have it somewhere else than the leader.
        let with_plots: Vec<(u64, Entity)> = members
            .iter()
            .filter_map(|&who| Some((who, body_of(who)?)))
            .filter(|&(_, body)| towns.channel_of(body).is_some())
            .collect();
        let apart = with_plots.iter().filter(|&&(_, body)| towns.channel_of(body) != Some(target)).count();
        if apart == 0 {
            continue;
        }
        let to = if towns.room_in(target) >= apart {
            target
        } else {
            towns.roomy(with_plots.len(), target).unwrap_or(towns.channels())
        };
        for (who, body) in with_plots {
            let lot = lots.get(body).ok().map(|(account, holds)| holds.lot(&account.0));
            let Some(plot) = towns.transfer(body, to, lot) else {
                continue;
            };
            let Ok((mut body_of, spot, venue)) = bodies.get_mut(body) else {
                continue;
            };
            body_of.town = Some(to);
            info!("{who} is moved to channel {to} of the town, with their group");
            if matches!(body_of.room, Room::Town(_)) {
                place(&mut commands, &mut tells, body, (body_of, spot, venue), Room::Town(to), Venue::Town, plot, None, now);
                social.gathered_to(who);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{Quat, Vec3};

    use super::*;

    #[test]
    fn a_channel_is_told_as_who_sits_where_and_who_is_in_their_car() {
        let entity = |n: u32| Entity::from_raw_u32(n).unwrap();
        let member = |id: u64| Member {
            id,
            name: format!("Person {id}"),
            look: "boy".into(),
            ai: false,
        };
        let mut first = Body::person(entity(11), Who(7), Room::Town(0), 0, Some(0), 0, 0.0);
        first.driving = true;
        let second = Body::person(entity(12), Who(8), Room::Town(0), 3, Some(0), 0, 0.0);
        let people = [(entity(1), first, member(7)), (entity(2), second, member(8))];
        let known = |body: Entity| {
            people
                .iter()
                .find(|(entity, ..)| *entity == body)
                .map(|(_, body, member)| (body, member))
        };
        // Everyone as they sit and as they go, and the devices to tell; a
        // body that has gone, its seat not yet given up, is left out.
        let (everyone, devices) = channel_of(&[(0, entity(1)), (2, entity(9)), (3, entity(2))], known);
        assert_eq!(
            everyone,
            [
                Seated {
                    who: 7,
                    seat: 0,
                    driving: true
                },
                Seated {
                    who: 8,
                    seat: 3,
                    driving: false
                }
            ]
        );
        assert_eq!(devices, [entity(11), entity(12)]);
        assert_eq!(channel_of(&[], known), (Vec::new(), Vec::new()));
        // Nobody rests, and nobody is told of.
        assert_eq!(resting_in(&[(0, entity(1)), (2, entity(9)), (3, entity(2))], known), Vec::new());
    }

    #[test]
    fn a_channel_is_told_who_rests_and_how() {
        let entity = |n: u32| Entity::from_raw_u32(n).unwrap();
        let member = |id: u64| Member {
            id,
            name: format!("Person {id}"),
            look: "girl".into(),
            ai: false,
        };
        let body = |client: u32, who: u64, seat: u8, pose: Option<Pose>| {
            let mut body = Body::person(entity(client), Who(who), Room::Town(0), seat, Some(0), 0, 0.0);
            body.pose = pose;
            body
        };
        let people = [
            (entity(1), body(11, 7, 0, Some(Pose::Lie)), member(7)),
            (entity(2), body(12, 8, 3, None), member(8)),
            (entity(3), body(13, 9, 5, Some(Pose::Sit)), member(9)),
        ];
        let known = |body: Entity| {
            people
                .iter()
                .find(|(entity, ..)| *entity == body)
                .map(|(_, body, member)| (body, member))
        };
        // Those who rest, each as they do; one on their feet is left out,
        // and so is a body that has gone.
        let resting = resting_in(&[(0, entity(1)), (2, entity(9)), (3, entity(2)), (5, entity(3))], known);
        assert_eq!(resting, [Posed { who: 7, pose: Pose::Lie }, Posed { who: 9, pose: Pose::Sit }]);
        // Someone the server puts anywhere is on their feet.
        assert_eq!(Body::person(entity(11), Who(7), Room::Town(0), 0, Some(0), 0, 0.0).pose, None);
    }

    #[test]
    fn a_channel_is_told_which_car_each_driver_is_in() {
        let entity = |n: u32| Entity::from_raw_u32(n).unwrap();
        let member = |id: u64| Member {
            id,
            name: format!("Person {id}"),
            look: "boy".into(),
            ai: false,
        };
        let body = |client: u32, who: u64, driving: bool, car: Option<&str>| {
            let mut body = Body::person(entity(client), Who(who), Room::Town(0), 0, Some(0), 0, 0.0);
            body.driving = driving;
            body.car = car.map(str::to_string);
            body
        };
        let people = [
            // In the car it said; in a car, having said none (an app from
            // before); out of the one it said; and on foot.
            (entity(1), body(11, 7, true, Some("yang")), member(7)),
            (entity(2), body(12, 8, true, None), member(8)),
            (entity(3), body(13, 9, false, Some("van")), member(9)),
            (entity(4), body(14, 10, false, None), member(10)),
        ];
        let known = |body: Entity| {
            people
                .iter()
                .find(|(entity, ..)| *entity == body)
                .map(|(_, body, member)| (body, member))
        };
        let seated = [(0, entity(1)), (1, entity(2)), (2, entity(3)), (3, entity(4)), (4, entity(99))];
        assert_eq!(
            cars_in(&seated, known),
            [Driver { who: 7, car: "yang".into() }, Driver { who: 8, car: bag::CAR.into() }]
        );
        // Someone the server makes says no car until their device does.
        assert_eq!(Body::person(entity(11), Who(7), Room::Town(0), 0, Some(0), 0, 0.0).car, None);
    }

    /// A bare world with the town in it, to seat people in and gather
    /// their groups; and someone seated in it, in the channel asked for:
    /// their body and their device.
    fn town() -> World {
        let mut world = World::new();
        world.init_resource::<Messages<ToClients<Tell>>>();
        world.init_resource::<Everyone>();
        world.init_resource::<Towns>();
        world.init_resource::<Time>();
        world.insert_resource(Social::new(crate::accounts::Snapshot::default()));
        world
    }

    fn seated(world: &mut World, id: u64, rather: Option<u16>) -> (Entity, Entity) {
        use roundtown_net::social::Presence;
        let (client, body) = (world.spawn_empty().id(), world.spawn_empty().id());
        let (channel, plot) = world.resource_mut::<Towns>().sit(body, id, rather);
        let member = Member {
            id,
            name: format!("Person {id}"),
            look: "boy".into(),
            ai: false,
        };
        world.entity_mut(body).insert((
            Body::person(client, Who(id), Room::Town(channel), plot, Some(channel), 0, 0.0),
            Spot::default(),
            Venue::Town,
            member.clone(),
            Account(member),
            crate::bag::Holds::of(&crate::accounts::Record::made_up()),
        ));
        world.resource_mut::<Everyone>().0.insert(Who(id), body);
        let mut social = world.resource_mut::<Social>();
        social.enroll(id, &format!("Person {id}"), None, 0);
        social.saw(id, Presence::Town { channel });
        (body, client)
    }

    /// Has `them` come into `leader`'s group: friends first, then asked,
    /// and their yes.
    fn grouped(world: &mut World, leader: u64, them: u64) {
        use roundtown_net::social::{Act, News, Post};
        let mut social = world.resource_mut::<Social>();
        social.act(leader, Act::Befriend(them), 0.0);
        social.act(them, Act::Befriend(leader), 0.0);
        social.told();
        social.act(leader, Act::Invite(them), 0.0);
        let mail = social
            .told()
            .into_iter()
            .find_map(|(to, news)| match news {
                News::Inbox(mail) if to == them => mail.into_iter().find(|mail| mail.what == Post::Invite).map(|mail| mail.id),
                _ => None,
            })
            .expect("asked into the group");
        social.act(them, Act::Answer { mail, yes: true }, 0.0);
        social.told();
    }

    /// Where the server has told `device` to stand since this was last
    /// asked: each plot of the town it was put on.
    fn put_on(world: &mut World, device: Entity) -> Vec<u8> {
        let mut all = world.resource_mut::<Messages<ToClients<Tell>>>();
        all.drain()
            .filter(|told| matches!(told.targets, SendTargets::Single(ClientId::Client(to)) if to == device))
            .filter_map(|told| match told.message {
                Tell::Arrive {
                    venue: Venue::Town,
                    seat,
                    spot: None,
                    ..
                } => Some(seat),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn someone_who_comes_into_a_group_is_moved_to_its_leader_s_channel_their_land_with_them() {
        use bevy::ecs::system::RunSystemOnce;
        use roundtown_net::social::{News, Outcome};
        let mut world = town();
        let (leader, leaders) = seated(&mut world, 1, None);
        let (them, theirs) = seated(&mut world, 2, Some(1));
        let (other, _) = seated(&mut world, 3, Some(1));
        assert_eq!(world.resource::<Towns>().counts(), vec![1, 2]);
        // In one channel already: nobody is moved.
        grouped(&mut world, 2, 3);
        world.run_system_once(gather).unwrap();
        assert_eq!(world.resource::<Towns>().counts(), vec![1, 2]);
        // Into the leader's group from another channel: moved to the
        // leader's, stood on the plot they have there and told; the leader
        // stays where they are, and whoever is left where they were.
        grouped(&mut world, 1, 2);
        world.run_system_once(gather).unwrap();
        world.flush();
        let towns = world.resource::<Towns>();
        assert_eq!((towns.channel_of(leader), towns.channel_of(them), towns.channel_of(other)), (Some(0), Some(0), Some(1)));
        assert_eq!(towns.counts(), vec![2, 1]);
        let body = world.get::<Body>(them).unwrap();
        assert_eq!((body.room, body.seat, body.town), (Room::Town(0), 1, Some(0)));
        assert_eq!(world.get::<crate::rooms::InRoom>(them).map(|room| room.0), Some(Room::Town(0)));
        assert_eq!(put_on(&mut world, theirs), [1]);
        assert!(put_on(&mut world, leaders).is_empty());
        let said: Vec<(u64, News)> = world.resource_mut::<Social>().told();
        assert!(said.contains(&(2, News::Said(Outcome::Gathered))), "{said:?}");
        assert!(!said.contains(&(1, News::Said(Outcome::Gathered))));
    }

    #[test]
    fn with_no_room_in_its_leader_s_channel_the_whole_group_goes_where_there_is_room_for_it() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = town();
        // The leader in a full channel, and whoever comes into their group
        // in another with a few in it.
        let (leader, leaders) = seated(&mut world, 1, None);
        for id in 10..10 + roundtown_net::TOWN_SEATS as u64 - 1 {
            seated(&mut world, id, None);
        }
        let (them, theirs) = seated(&mut world, 2, None);
        seated(&mut world, 3, None);
        assert_eq!(world.resource::<Towns>().counts(), vec![roundtown_net::TOWN_SEATS, 2]);
        grouped(&mut world, 1, 2);
        world.run_system_once(gather).unwrap();
        world.flush();
        // Both in the other channel now: the leader moved to it, and stood
        // on their plot there; the one already there is as they were.
        let towns = world.resource::<Towns>();
        assert_eq!((towns.channel_of(leader), towns.channel_of(them)), (Some(1), Some(1)));
        assert_eq!(towns.counts(), vec![roundtown_net::TOWN_SEATS - 1, 3]);
        assert_eq!(put_on(&mut world, leaders), [2]);
        assert!(put_on(&mut world, theirs).is_empty());
        // Away in a game, someone is moved without being stood anywhere:
        // their plot is in the channel they come back to.
        let mut world = town();
        let (leader, _) = seated(&mut world, 1, None);
        let (them, theirs) = seated(&mut world, 2, Some(1));
        world.resource_mut::<Towns>().away(them);
        world.get_mut::<Body>(them).unwrap().room = Room::Table(roundtown_net::Minigame::KnockOff, 0);
        grouped(&mut world, 1, 2);
        world.run_system_once(gather).unwrap();
        world.flush();
        let towns = world.resource::<Towns>();
        assert_eq!((towns.channel_of(leader), towns.channel_of(them)), (Some(0), Some(0)));
        assert_eq!(towns.held_in(0), [(0, leader, false), (1, them, true)]);
        assert!(put_on(&mut world, theirs).is_empty());
        assert_eq!(world.get::<Body>(them).unwrap().town, Some(0));
        // Back from the game, they sit down where they are kept.
        assert_eq!(world.resource_mut::<Towns>().sit(them, 2, Some(1)), (0, 1));
    }

    #[test]
    fn a_body_that_is_gone_leaves_its_plot_kept_as_it_was_and_everyone_is_told_whose_each_is() {
        use bevy::ecs::system::RunSystemOnce;
        use roundtown_net::land::Holder;
        let mut world = town();
        world.add_observer(body_gone);
        let (_, first) = seated(&mut world, 1, None);
        let (them, _) = seated(&mut world, 2, None);
        world.entity_mut(first).insert(Watching);
        // Everyone in the town who asked is told whose every plot is, by
        // the names their accounts go by.
        world.run_system_once(tell_channels).unwrap();
        let plots = |world: &mut World| {
            let mut all = world.resource_mut::<Messages<ToClients<Tell>>>();
            all.drain()
                .filter_map(|told| match told.message {
                    Tell::Plots(deeds) => Some(deeds),
                    _ => None,
                })
                .last()
        };
        let deeds = plots(&mut world).expect("told of the plots");
        // The two of them; every other plot bare, nobody's, no made-up
        // land filling it (`premade::FILLS`).
        assert_eq!(deeds.len(), 2);
        assert_eq!((deeds[1].who, deeds[1].name.as_str(), deeds[1].holder), (2, "Person 2", Holder::Here));
        // Their body despawned, however: the plot is theirs still, with
        // nobody on it, and the channel is told so.
        world.entity_mut(them).despawn();
        world.run_system_once(tell_channels).unwrap();
        let deeds = plots(&mut world).expect("told again");
        assert_eq!((deeds[1].who, deeds[1].name.as_str(), deeds[1].holder), (2, "Person 2", Holder::Away));
        assert_eq!(world.resource::<Towns>().counts(), vec![2]);
        // Nothing is told while nothing changes.
        world.run_system_once(tell_channels).unwrap();
        assert!(plots(&mut world).is_none());
    }

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
        // The car boosted for ten seconds is a move a body can make.
        let mut pace = Pace::new(0.0);
        let mut at = spot(0.0, 0.0, 0.0);
        for step in 1..=150 {
            let next = spot(step as f32 * cars::DRIVEN_TOP_SPEED / 15.0, 0.0, 0.0);
            assert!(pace.take(&at, &next, f64::from(step) / 15.0), "step {step}");
            at = next;
        }
        // Twice that for a few seconds runs out what is in hand.
        let mut pace = Pace::new(0.0);
        let mut at = spot(0.0, 0.0, 0.0);
        let caught = (1..=60).any(|step| {
            let next = spot(step as f32 * 2.0 * cars::DRIVEN_TOP_SPEED / 15.0, 0.0, 0.0);
            let taken = pace.take(&at, &next, f64::from(step) / 15.0);
            at = next;
            !taken
        });
        assert!(caught);
    }

    #[test]
    fn every_name_a_player_is_given_is_a_name() {
        // With any number after it: nothing the word filter takes for a
        // swear word, which a name may not have in it (`profile::name`).
        for name in NAMES {
            for number in [100, 482, 999] {
                let given = format!("{name} {number}");
                assert_eq!(profile::name(&given).as_deref(), Some(given.as_str()));
            }
        }
        assert!(profile::name(&new_name()).is_some());
        // And a name that has one is not taken from a device.
        assert_eq!(profile::name("asshole"), None);
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
    fn riding_the_car_is_a_walk_on_it_and_getting_off_is_where_it_went() {
        let on = |x: f32, y: f32, z: f32| spot(x, y, z).aboard(Some(0));
        let mut pace = Pace::new(0.0);
        // Running up to the car, 25 m out on the inner road, and jumping on.
        let road = spot(25.0, -0.2, 3.0);
        assert!(pace.take(&spot(24.6, -0.2, 3.0), &road, 1.0 / 15.0));
        assert!(pace.take(&road, &on(-2.5, 1.0, 0.4), 2.0 / 15.0));
        // Standing on it while it drives, and walking about on it: still,
        // then a walk, in its own space.
        let mut at = on(-2.5, 1.0, 0.4);
        for step in 3..=40 {
            let next = on(-2.5 + (step - 3) as f32 * 0.2, 1.0, 0.4);
            assert!(pace.take(&at, &next, f64::from(step) / 15.0), "step {step}");
            at = next;
        }
        // Thirty seconds on, off it where it has taken you: up to its top
        // speed all that time, and a run, from where you got on.
        let off = spot(25.0 + 30.0 * cars::TOP_SPEED, -0.2, 3.0);
        assert!(pace.take(&at, &off, 30.0 + 40.0 / 15.0));
        // Further than it could have taken you is a leap.
        let mut pace = Pace::new(0.0);
        assert!(pace.take(&road, &on(0.0, 1.0, 0.0), 1.0));
        assert!(!pace.take(&on(0.0, 1.0, 0.0), &spot(-400.0, 0.0, 300.0), 3.0));
        // Nor is anywhere far off the car a place on it.
        let mut pace = Pace::new(0.0);
        assert!(!pace.take(&road, &on(40.0, 1.0, 0.0), 1.0));
        let mut pace = Pace::new(0.0);
        assert!(pace.take(&road, &on(0.0, 1.0, 0.0), 1.0));
        assert!(!pace.take(&on(0.0, 1.0, 0.0), &on(0.0, 1.0, 30.0), 1.1));
    }

    #[test]
    fn the_fountain_throwing_you_up_is_a_rise_a_body_can_make() {
        // The town's fountain throws whoever stands in its top bowl, 1.4 m
        // up, 9 m higher (the game's `fountain::THROW_HEIGHT`): up at the
        // speed that takes them that high under the game's gravity, 22 m/s²,
        // over twice as fast as a jump, and down at one and a half times
        // that. Moves are sent fifteen times a second; the game moves 120.
        let floor = 1.4f32;
        let (mut y, mut rising) = (floor, (2.0f32 * 22.0 * 9.0).sqrt());
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

    #[test]
    fn a_jump_on_a_leap_potion_is_a_rise_a_body_can_make() {
        // Bridge Wars' leap potion has a jump go five times as high
        // (`bridge_wars::LEAP`): up at the square root of five times a
        // jump's 8.5 m/s, 19 m/s, which is 8.2 m under the game's gravity,
        // and slower than the fountain throws. Three of them one after
        // another, each begun as the last comes down, as someone on the
        // potion jumps; and a run of 14 m/s across the ground all the
        // while, as someone on a speed potion too goes.
        let floor = 0.25f32;
        let speed = 8.5 * roundtown_net::bridge_wars::leap_speed();
        let (mut x, mut y, mut rising) = (0.0f32, floor, speed);
        let mut highest = floor;
        let mut pace = Pace::new(0.0);
        let mut at = spot(x, y, 0.0);
        let mut jumps = 1;
        for step in 1..=75 {
            for _ in 0..8 {
                let dt = 1.0 / 120.0;
                rising -= if rising > 0.0 { 22.0 } else { 33.0 } * dt;
                y += rising * dt;
                x += 14.0 * dt;
                if y <= floor {
                    // Down, and up again at once.
                    y = floor;
                    rising = speed;
                    jumps += 1;
                }
                highest = highest.max(y);
            }
            let next = spot(x, y, 0.0);
            assert!(pace.take(&at, &next, f64::from(step) / 15.0), "step {step}, {y} m up");
            at = next;
        }
        assert!(jumps >= 3, "{jumps} jumps");
        assert!((highest - floor - 8.2).abs() < 0.2, "{highest}");
    }
}
