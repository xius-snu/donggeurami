//! House Builder, on the server: every game, from the queue to the results,
//! on the server's clock.
//!
//! Devices show what it says — the stage, the time left and the theme
//! ([`Round`]), who is playing ([`Seats`]) and every piece put down
//! ([`Built`]) — send what their player does ([`Ask::Play`], [`Ask::Rate`],
//! [`Ask::Put`], [`Ask::Take`], [`Ask::Leave`]), and are moved by it: to the
//! lobby, to their plots, round the houses and to the winner's.
//!
//! A game is a room of its own ([`Room::Game`]): its lobby, its plots,
//! everyone in it and every piece down in it. Whoever wants to play goes into
//! the game filling now. After [`PEOPLE_WAIT`] seconds of waiting for people,
//! the computer takes the seats still empty, one at a time, as it does
//! offline. Its players stand where they arrive (the server has no islands
//! for them to walk on), build nothing, and rate every house at random.
//!
//! Every game is kept in [`Games`]; the entity its room is sent, with its
//! [`Round`] and [`Seats`], only mirrors it.
//!
//! The fee is taken here when you join. Coins are kept in memory for now, so
//! every connection starts with [`STARTING_BALANCE`], as every launch of the
//! app does.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::rules::*;
use roundtown_net::{
    Ask, At, Built, GAME_SEATS, Member, Refusal, Round, Seats, Spot, Stage, Tell, Venue, arrival,
};

use crate::Config;
use crate::accounts::{Bank, Charged};
use crate::players::{Body, Everyone, Plays, Who, move_to, tell};
use crate::rooms::{InRoom, Room, Towns};

/// What the computer's players wear: anything but what people do.
const BOT_LOOKS: &[&str] = &["red", "yellow", "purple"];
/// What the computer's players are called: the names people get, without
/// the number after.
const BOT_NAMES: &[&str] = &[
    "Mochi", "Dubu", "Hodu", "Bori", "Gamja", "Mandu", "Kimbap", "Yuja", "Maru", "Nabi", "Kong",
    "Podo", "Sagwa", "Gyul", "Hobak", "Bam", "Dalgi", "Haru", "Byeol", "Nuri",
];
/// How long after building ends a piece put down still counts, in seconds: a
/// device puts down whatever it still had out once it hears the time is up.
const GRACE: f32 = 3.0;
/// How long a game is kept once it is over, for anyone who has not left it,
/// in seconds: then they are sent back to the town.
const OVER_KEEP: f32 = 600.0;
/// How long a game goes on with nobody connected to it, in seconds, before it
/// is given up.
const ABANDONED: f32 = 120.0;
/// The most pieces one plot can hold, and the longest name a kind can have:
/// far more than anyone builds, so that a device gone wrong cannot fill the
/// server's memory.
const MOST_PIECES: usize = 400;
const LONGEST_KIND: usize = 48;
/// How far out a piece can be put, in metres: the edge of the world.
const FURTHEST: f32 = 600.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Games>()
        .add_systems(
            PreUpdate,
            (
                hear.after(ServerSystems::Receive),
                paid.after(crate::accounts::answers),
            )
                .run_if(in_state(ServerState::Running)),
        )
        .add_systems(Update, run_games.run_if(in_state(ServerState::Running)));
}

/// A body whose fee is being taken in the database.
#[derive(Component)]
struct Paying;

/// Every game under way, by id, and the one filling now.
#[derive(Resource, Default)]
pub(crate) struct Games {
    next: u32,
    games: HashMap<u32, Game>,
    filling: Option<u32>,
}

/// One game of House Builder.
struct Game {
    id: u32,
    /// The entity its room is sent, with its [`Round`] and [`Seats`].
    shown: Entity,
    /// Everyone's body, in seat order: a builder's plot is their seat.
    seats: Vec<Entity>,
    members: Vec<Member>,
    /// Who has left it, back to the town or for another game.
    gone: Vec<bool>,
    /// Which seats the computer is playing.
    bot: Vec<bool>,
    stage: Stage,
    /// Seconds left of this stage.
    left: f32,
    /// Seconds spent in the queue.
    waited: f32,
    /// Seconds since the building ended.
    since_build: f32,
    /// Seconds with nobody connected to it.
    unwatched: f32,
    theme: u8,
    stars: [u32; GAME_SEATS],
    /// The stars each seat has given the house being visited.
    given: HashMap<u8, u8>,
    /// Every piece down, by plot and the builder's own id for it.
    pieces: HashMap<(u8, u32), Entity>,
    /// What its room was last sent, to send again only what has changed.
    sent: Option<Round>,
    sent_seats: usize,
}

impl Game {
    fn room(&self) -> Room {
        Room::Game(self.id)
    }

    fn seat_of(&self, body: Entity) -> Option<u8> {
        self.seats
            .iter()
            .position(|&seat| seat == body)
            .map(|seat| seat as u8)
    }

    fn bots(&self) -> usize {
        self.bot.iter().filter(|&&bot| bot).count()
    }

    /// Pieces can still be put down and picked up: while the building lasts,
    /// and for a moment after.
    fn building(&self) -> bool {
        match self.stage {
            Stage::Build => true,
            Stage::Visit(_) => self.since_build < GRACE,
            _ => false,
        }
    }

    /// What the room is shown of how it is going.
    fn round(&self) -> Round {
        Round {
            stage: self.stage,
            left: match self.stage {
                Stage::Queue | Stage::Over { .. } => 0,
                _ => self.left.max(0.0).ceil() as u16,
            },
            theme: self.theme,
        }
    }
}

impl Games {
    /// The game filling now, if it still has room; otherwise a new one.
    fn filling(&mut self, commands: &mut Commands) -> &mut Game {
        let open = self.filling.filter(|id| {
            self.games
                .get(id)
                .is_some_and(|game| game.stage == Stage::Queue && game.seats.len() < GAME_SEATS)
        });
        let id = open.unwrap_or_else(|| {
            let id = self.next;
            self.next += 1;
            let theme = fastrand::u8(..);
            let shown = commands
                .spawn((Replicated, InRoom(Room::Game(id)), Seats::default()))
                .id();
            self.games.insert(
                id,
                Game {
                    id,
                    shown,
                    seats: Vec::new(),
                    members: Vec::new(),
                    gone: Vec::new(),
                    bot: Vec::new(),
                    stage: Stage::Queue,
                    left: 0.0,
                    waited: 0.0,
                    since_build: 0.0,
                    unwatched: 0.0,
                    theme,
                    stars: [0; GAME_SEATS],
                    given: HashMap::new(),
                    pieces: HashMap::new(),
                    sent: None,
                    sent_seats: 0,
                },
            );
            info!("House Builder game {id} opened");
            self.filling = Some(id);
            id
        });
        self.games.get_mut(&id).expect("just found or made")
    }

    /// The game in `room`, and `body`'s seat in it.
    fn of(&mut self, room: Room, body: Entity) -> Option<(&mut Game, u8)> {
        let Room::Game(id) = room else {
            return None;
        };
        let game = self.games.get_mut(&id)?;
        let seat = game.seat_of(body)?;
        Some((game, seat))
    }

    /// The id of the game filling now, for the line in `coin_changes` saying
    /// what a fee was for: most likely the one it seats them in.
    fn filling_id(&self) -> Option<u32> {
        self.filling
    }

    /// `body` has left the game in `room` while it was going on: the
    /// connection went, and it gave the game up.
    pub(crate) fn walk_out(&mut self, room: Room, body: Entity) {
        if let Some((game, seat)) = self.of(room, body) {
            game.gone[usize::from(seat)] = true;
        }
    }
}

/// Seats `body`, paid up, in the game filling now: out of the town, or out of
/// the game it has just finished, for another.
#[allow(clippy::too_many_arguments)]
fn seat(
    games: &mut Games,
    towns: &mut Towns,
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    body: Entity,
    body_of: (Mut<Body>, Mut<Spot>, Mut<Venue>),
    member: Member,
    now: f64,
) {
    let room = body_of.0.room;
    if let Some((old, seat)) = games.of(room, body) {
        old.gone[usize::from(seat)] = true;
    }
    towns.stand(body);
    let game = games.filling(commands);
    let seat = game.seats.len() as u8;
    game.seats.push(body);
    game.members.push(member);
    game.gone.push(false);
    game.bot.push(false);
    let room = game.room();
    if let Some(client) = body_of.0.client {
        tell(tells, client, Tell::Joined { seat });
    }
    move_to(commands, tells, body, body_of, room, Venue::Lobby, seat, now);
}

/// The fee taken in the database: seated, or turned away without the coins.
#[allow(clippy::too_many_arguments)]
fn paid(
    mut charged: MessageReader<Charged>,
    everyone: Res<Everyone>,
    mut bodies: Bodies,
    mut games: ResMut<Games>,
    mut towns: ResMut<Towns>,
    time: Res<Time>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    for answer in charged.read() {
        let Some(&body) = everyone.0.get(&Who(answer.id)) else {
            continue;
        };
        commands.entity(body).try_remove::<Paying>();
        let Ok((mut body_of, spot, venue, member)) = bodies.get_mut(body) else {
            continue;
        };
        let Some(left) = answer.left else {
            if let Some(client) = body_of.client {
                tell(&mut tells, client, Tell::Refused(Refusal::Coins));
            }
            continue;
        };
        body_of.coins = left;
        if let Some(client) = body_of.client {
            tell(&mut tells, client, Tell::Balance(left));
        }
        let member = member.clone();
        seat(
            &mut games,
            &mut towns,
            &mut commands,
            &mut tells,
            body,
            (body_of, spot, venue),
            member,
            now,
        );
    }
}

type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Body,
        &'static mut Spot,
        &'static mut Venue,
        &'static Member,
    ),
>;

/// What players ask of House Builder.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<&Plays>,
    mut bodies: Bodies,
    paying: Query<(), With<Paying>>,
    mut games: ResMut<Games>,
    mut towns: ResMut<Towns>,
    bank: Res<Bank>,
    config: Res<Config>,
    time: Res<Time>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    for ask in asks.read() {
        let Some(client) = ask.client_id.entity() else {
            continue;
        };
        let Ok(&Plays(body)) = clients.get(client) else {
            continue;
        };
        let Ok((body_of, ..)) = bodies.get(body) else {
            continue;
        };
        let room = body_of.room;
        match &ask.message {
            Ask::Play => {
                // From the town, or from a game that is over, for another.
                let free = match room {
                    Room::Town(_) => true,
                    Room::Game(_) => games
                        .of(room, body)
                        .is_some_and(|(game, _)| matches!(game.stage, Stage::Over { .. })),
                    Room::Home(_) => false,
                };
                if !free {
                    tell(&mut tells, client, Tell::Refused(Refusal::Busy));
                    continue;
                }
                let Ok((mut body_of, spot, venue, member)) = bodies.get_mut(body) else {
                    continue;
                };
                if body_of.coins < ENTRY || paying.contains(body) {
                    tell(&mut tells, client, Tell::Refused(Refusal::Coins));
                    continue;
                }
                // Kept in the database: taken there, in one transaction with
                // the line saying why, and seated once it has been ([`paid`]).
                if bank.kept() {
                    bank.charge(body_of.who.0, ENTRY, "House Builder entry", games.filling_id());
                    commands.entity(body).insert(Paying);
                    continue;
                }
                body_of.coins -= ENTRY;
                tell(&mut tells, client, Tell::Balance(body_of.coins));
                let member = member.clone();
                seat(
                    &mut games,
                    &mut towns,
                    &mut commands,
                    &mut tells,
                    body,
                    (body_of, spot, venue),
                    member,
                    now,
                );
            }
            Ask::Leave => {
                let Some((game, seat)) = games.of(room, body) else {
                    continue;
                };
                if !matches!(game.stage, Stage::Over { .. }) {
                    continue;
                }
                game.gone[usize::from(seat)] = true;
                let Ok((mut body_of, spot, venue, _)) = bodies.get_mut(body) else {
                    continue;
                };
                let (channel, town_seat) = towns.sit(body, body_of.town);
                body_of.town = Some(channel);
                move_to(
                    &mut commands,
                    &mut tells,
                    body,
                    (body_of, spot, venue),
                    Room::Town(channel),
                    Venue::Town,
                    town_seat,
                    now,
                );
            }
            Ask::Rate { plot, stars } => {
                if let Some((game, seat)) = games.of(room, body)
                    && game.stage == Stage::Visit(*plot)
                    && *plot != seat
                {
                    game.given.insert(seat, (*stars).clamp(1, MOST_STARS));
                }
            }
            Ask::Skip => {
                if let Some((game, _)) = games.of(room, body)
                    && config.allow_skip
                    && matches!(game.stage, Stage::Build | Stage::Visit(_))
                {
                    game.left = 0.0;
                }
            }
            Ask::Put { id, kind, at } => {
                let Some((game, seat)) = games.of(room, body) else {
                    continue;
                };
                if !game.building() || kind.len() > LONGEST_KIND || !sane(at) {
                    continue;
                }
                let built = Built {
                    plot: seat,
                    kind: kind.clone(),
                    at: *at,
                };
                if let Some(&piece) = game.pieces.get(&(seat, *id)) {
                    commands.entity(piece).insert(built);
                } else if game.pieces.keys().filter(|(plot, _)| *plot == seat).count() < MOST_PIECES {
                    let piece = commands
                        .spawn((Replicated, InRoom(game.room()), built))
                        .id();
                    game.pieces.insert((seat, *id), piece);
                }
            }
            Ask::Take { id } => {
                if let Some((game, seat)) = games.of(room, body)
                    && game.building()
                    && let Some(piece) = game.pieces.remove(&(seat, *id))
                {
                    commands.entity(piece).despawn();
                }
            }
            Ask::Hello { .. } | Ask::Travel(_) => {}
        }
    }
}

/// Whether `at` is somewhere a piece could be.
fn sane(at: &At) -> bool {
    match *at {
        At::Ground { x, y, z, yaw } => {
            [x, y, z, yaw].iter().all(|n| n.is_finite())
                && x.abs() < FURTHEST
                && z.abs() < FURTHEST
                && y.abs() < 100.0
        }
        At::Wall { along, up, .. } => {
            along.is_finite() && up.is_finite() && along.abs() < 100.0 && up.abs() < 100.0
        }
    }
}

/// Moves every game on: fills its room, sends everyone to their plots, round
/// the houses, and to the winner's; and gives it up once nobody is left.
#[allow(clippy::too_many_arguments)]
fn run_games(
    time: Res<Time>,
    mut games: ResMut<Games>,
    mut bodies: Bodies,
    mut everyone: ResMut<Everyone>,
    mut towns: ResMut<Towns>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs_f64();
    let mut ended = Vec::new();
    for game in games.games.values_mut() {
        let room = game.room();
        // Whether anyone is still playing: connected, in it, and not gone.
        let watched = (0..game.seats.len()).any(|seat| {
            !game.gone[seat]
                && !game.bot[seat]
                && bodies
                    .get(game.seats[seat])
                    .is_ok_and(|(body, ..)| body.client.is_some() && body.room == room)
        });
        game.unwatched = if watched { 0.0 } else { game.unwatched + dt };
        let everyone_gone = (0..game.seats.len()).all(|seat| game.gone[seat] || game.bot[seat]);
        let over = matches!(game.stage, Stage::Over { .. });
        if (over && (everyone_gone || game.left <= 0.0)) || game.unwatched > ABANDONED {
            end(&mut commands, &mut tells, game, &mut bodies, &mut everyone, &mut towns, now);
            ended.push(game.id);
            continue;
        }

        match game.stage {
            Stage::Queue => {
                game.waited += dt;
                let every = QUEUE_FILL / (GAME_SEATS - 1) as f32;
                while game.seats.len() < GAME_SEATS
                    && game.waited >= PEOPLE_WAIT + game.bots() as f32 * every
                {
                    seat_bot(&mut commands, game, now);
                }
                if game.seats.len() >= GAME_SEATS {
                    game.stage = Stage::Full;
                    game.left = FULL_HOLD;
                }
            }
            Stage::Full => {
                game.left -= dt;
                if game.left <= 0.0 {
                    for (seat, &body) in game.seats.iter().enumerate() {
                        let plot = Venue::Plot(seat as u8);
                        send(&mut commands, &mut tells, &mut bodies, body, room, plot, seat as u8, now);
                    }
                    game.stage = Stage::Build;
                    game.left = BUILD_SECS;
                    info!("House Builder game {}: building", game.id);
                }
            }
            Stage::Build => {
                game.left -= dt;
                if game.left <= 0.0 {
                    game.since_build = 0.0;
                    visit(&mut commands, &mut tells, &mut bodies, game, 0, now);
                }
            }
            Stage::Visit(plot) => {
                game.left -= dt;
                game.since_build += dt;
                if game.left <= 0.0 {
                    tally(game, plot);
                    let next = plot + 1;
                    if usize::from(next) < game.seats.len() {
                        visit(&mut commands, &mut tells, &mut bodies, game, next, now);
                    } else {
                        let winner = winner(&game.stars);
                        for (seat, &body) in game.seats.iter().enumerate() {
                            let plot = Venue::Plot(winner);
                            send(&mut commands, &mut tells, &mut bodies, body, room, plot, seat as u8, now);
                        }
                        game.stage = Stage::Over {
                            winner,
                            stars: game.stars,
                        };
                        game.left = OVER_KEEP;
                        info!(
                            "House Builder game {}: plot {winner} won, with {:?}",
                            game.id, game.stars
                        );
                    }
                }
            }
            Stage::Over { .. } => game.left -= dt,
        }

        // Only what has changed goes out again.
        let round = game.round();
        if game.sent.as_ref() != Some(&round) {
            commands.entity(game.shown).insert(round.clone());
            game.sent = Some(round);
        }
        if game.sent_seats != game.members.len() {
            commands
                .entity(game.shown)
                .insert(Seats(game.members.clone()));
            game.sent_seats = game.members.len();
        }
    }
    for id in ended {
        games.games.remove(&id);
        if games.filling == Some(id) {
            games.filling = None;
        }
    }
}

/// Sends `body`, if it is still in the game's `room`, to `venue` where `seat`
/// arrives there.
#[allow(clippy::too_many_arguments)]
fn send(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    body: Entity,
    room: Room,
    venue: Venue,
    seat: u8,
    now: f64,
) {
    let Ok((body_of, spot, at, _)) = bodies.get_mut(body) else {
        return;
    };
    if body_of.room != room {
        return;
    }
    move_to(commands, tells, body, (body_of, spot, at), room, venue, seat, now);
}

/// Everyone to the house on `plot`, to rate it.
fn visit(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    game: &mut Game,
    plot: u8,
    now: f64,
) {
    let room = game.room();
    for (seat, &body) in game.seats.iter().enumerate() {
        send(commands, tells, bodies, body, room, Venue::Plot(plot), seat as u8, now);
    }
    game.given.clear();
    game.stage = Stage::Visit(plot);
    game.left = VISIT_SECS;
}

/// Adds up the stars the house on `plot` was given: what each person gave
/// it, if anything, and the computer's players', at random. Nobody rates
/// their own house.
fn tally(game: &mut Game, plot: u8) {
    let mut sum = 0;
    for seat in 0..game.seats.len() {
        let seat = seat as u8;
        if seat == plot {
            continue;
        }
        sum += u32::from(if game.bot[usize::from(seat)] {
            fastrand::u8(1..=MOST_STARS)
        } else {
            game.given.get(&seat).copied().unwrap_or(0)
        });
    }
    game.stars[usize::from(plot)] += sum;
}

/// The house with the most stars, a tie settled by lot.
fn winner(stars: &[u32; GAME_SEATS]) -> u8 {
    let most = stars.iter().copied().max().unwrap_or(0);
    let best: Vec<usize> = (0..GAME_SEATS).filter(|&plot| stars[plot] == most).collect();
    best[fastrand::usize(..best.len())] as u8
}

/// The computer, in the next seat of `game`, standing in the lobby under a
/// name nobody there has.
fn seat_bot(commands: &mut Commands, game: &mut Game, now: f64) {
    let seat = game.seats.len();
    let names: Vec<&str> = BOT_NAMES
        .iter()
        .copied()
        .filter(|name| game.members.iter().all(|member| !member.name.starts_with(name)))
        .collect();
    let member = Member {
        // Clear of people's ids, which count up from 1.
        id: u64::MAX - u64::from(game.id) * GAME_SEATS as u64 - seat as u64,
        name: fastrand::choice(names).unwrap_or("Bot").into(),
        look: BOT_LOOKS[seat.saturating_sub(1) % BOT_LOOKS.len()].into(),
        ai: true,
    };
    let (at, turn) = arrival(seat);
    let body = commands
        .spawn((
            Replicated,
            member.clone(),
            Spot::of(at, turn),
            Venue::Lobby,
            InRoom(game.room()),
            Body::bot(game.room(), seat as u8, now),
        ))
        .id();
    game.seats.push(body);
    game.members.push(member);
    game.gone.push(false);
    game.bot.push(true);
}

/// The end of a game: its pieces and its computer players go, and anyone still
/// in it is sent back to the town, or, if they are not connected, forgotten.
fn end(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    game: &mut Game,
    bodies: &mut Bodies,
    everyone: &mut Everyone,
    towns: &mut Towns,
    now: f64,
) {
    let room = game.room();
    for (seat, &body) in game.seats.iter().enumerate() {
        if game.bot[seat] {
            commands.entity(body).despawn();
            continue;
        }
        let Ok((mut body_of, spot, venue, _)) = bodies.get_mut(body) else {
            continue;
        };
        if body_of.room != room {
            continue;
        }
        if body_of.client.is_none() {
            everyone.0.remove(&body_of.who);
            commands.entity(body).despawn();
            continue;
        }
        let (channel, town_seat) = towns.sit(body, body_of.town);
        body_of.town = Some(channel);
        move_to(
            commands,
            tells,
            body,
            (body_of, spot, venue),
            Room::Town(channel),
            Venue::Town,
            town_seat,
            now,
        );
    }
    for (_, piece) in game.pieces.drain() {
        commands.entity(piece).despawn();
    }
    commands.entity(game.shown).despawn();
    info!("House Builder game {} closed", game.id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_most_stars_win() {
        let mut stars = [0; GAME_SEATS];
        stars[3] = 30;
        stars[5] = 12;
        assert_eq!(winner(&stars), 3);
    }

    #[test]
    fn a_tie_goes_to_one_of_those_tied() {
        let mut stars = [0; GAME_SEATS];
        stars[1] = 20;
        stars[6] = 20;
        for _ in 0..50 {
            assert!(matches!(winner(&stars), 1 | 6));
        }
    }

    #[test]
    fn a_piece_has_to_be_somewhere() {
        assert!(sane(&At::Ground {
            x: 3.0,
            y: 0.0,
            z: -4.0,
            yaw: 1.0
        }));
        assert!(!sane(&At::Ground {
            x: f32::NAN,
            y: 0.0,
            z: 0.0,
            yaw: 0.0
        }));
        assert!(!sane(&At::Ground {
            x: 1e9,
            y: 0.0,
            z: 0.0,
            yaw: 0.0
        }));
        assert!(sane(&At::Wall {
            wall: 2,
            along: 1.5,
            up: 5.0,
            face: 1
        }));
    }
}
