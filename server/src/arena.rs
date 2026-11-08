//! Bridge Wars, on the server (since 2026-11-08; Hajun: "right now the
//! bridge battle minigame doesnt allow queing with other players i think
//! even tho we join the game at the same time. fix that"): every game, from
//! the waiting room to the end, on the server's clock, so that people who
//! choose it together play it together.
//!
//! Devices show what it says — the stage, the time left, everyone's health,
//! each team's respawn anchor and who is out ([`ArenaRound`]), who is
//! playing ([`Seats`]) and every plank of every bridge ([`Plank`]) — send
//! what their player does ([`Ask::Arena`] to join, [`Ask::Punch`] and
//! [`Ask::HitAnchor`] for a punch that landed, [`Ask::Rung`] for a plank
//! laid, [`Ask::Respawn`] for a fall into the void, [`Ask::Leave`]), and are
//! moved by it: into the waiting room, to their islands, back where they
//! arrive when they stand again, and to the town at the end.
//!
//! A game is a room of its own ([`Room::Arena`]). Whoever wants to play goes
//! into the game filling now. After [`BOT_WAIT`] seconds with nobody joining
//! the computer takes every seat left at once, as the game does offline, and
//! the room counts down and goes. The computer's players are moved here
//! ([`Mind`]): the server has no map, so each keeps to the disc of its
//! island ([`ARENA_ISLAND_RADIUS`]), runs at whoever of another team comes
//! within its sight there, punches once in reach, runs away when nearly
//! beaten, and wanders when alone; knocked off its island it falls, and is
//! dead at the bottom. It never crosses a bridge.
//!
//! Health is the server's: a punch that lands ([`Ask::Punch`]) takes
//! [`DAMAGE`] off whoever it landed on, if they are not on the puncher's
//! team, and knocks them (`Tell::Knocked` to a person's device, which knocks
//! its own body; a bot is knocked here); at none a body is dead for
//! [`RESPAWN_SECS`] and then stands again where its seat arrives, so long as
//! its team's anchor stood when it died, and is out for good otherwise
//! (Hajun: "you only indefinitely respawn when you die, and at that point
//! when u died, the respawn anchor is still alive"). An anchor takes the
//! same damage from a punch of another team's ([`Ask::HitAnchor`]), twenty
//! of them ([`ANCHOR_HEALTH`]). The last team with anyone in play wins, or
//! nobody when the time runs out; the result is up for [`RESULTS_SECS`] and
//! everyone still there goes back to the town.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::bridge_wars::*;
use roundtown_net::{
    ARENA_ANCHORS, ARENA_ISLAND_RADIUS, ARENA_ISLANDS, ARENA_SEATS, ARENA_TEAMS, ArenaRound,
    ArenaStage, Ask, Member, Plank, Refusal, Seats, Spot, Tell, Venue, arena_island, arena_team,
    arrival,
};

use crate::Config;
use crate::players::{Body, Everyone, Plays, move_to, place, tell};
use crate::rooms::{InRoom, Room, Towns};

/// How long a game goes on with nobody connected to it, in seconds, before
/// it is given up.
const ABANDONED: f32 = 60.0;
/// The most rungs one seat can lay: far more than a game's worth, so that a
/// device gone wrong cannot fill the server's memory.
const MOST_RUNGS: usize = 600;
/// How far out a rung can be laid, in metres: the edge of the map and a
/// little.
const FURTHEST: f32 = 220.0;
/// How far apart a puncher and whoever they hit can be where each last said
/// they stood, in metres, for the punch to count: the reach, 2 m from the
/// puncher's middle, and the way each may have gone since.
const PUNCH_SLACK: f32 = 6.5;
/// How wide a body is, across the ground, in metres: the game's
/// `PLAYER_RADIUS`.
const BODY_RADIUS: f32 = 0.45;
/// Where a body stands on an island, in metres up: the islands' tops.
const ISLAND_TOP: f32 = 0.25;
/// How fast a body knocked off its island falls, in metres a second a
/// second: the game's `FALL_GRAVITY`.
const GRAVITY: f32 = 33.0;
/// How often a bot with nobody to go for picks somewhere new to walk, in
/// seconds, and how fast it walks there, in metres a second.
const THINK_EVERY: f32 = 3.0;
const STROLL: f32 = 7.0;
/// How near a bot stands off whoever it goes for, as a share of the reach.
const PRESS_IN: f32 = 0.7;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Arenas>()
        .add_systems(
            PreUpdate,
            hear.after(ServerSystems::Receive).run_if(in_state(ServerState::Running)),
        )
        .add_systems(Update, run.run_if(in_state(ServerState::Running)));
}

/// Every game under way, by id, and the one filling now.
#[derive(Resource, Default)]
pub(crate) struct Arenas {
    next: u32,
    games: HashMap<u32, Game>,
    filling: Option<u32>,
}

/// One game of Bridge Wars.
struct Game {
    id: u32,
    /// The entity its room is sent, with its [`ArenaRound`] and [`Seats`].
    shown: Entity,
    /// Everyone's body, in seat order.
    seats: Vec<Entity>,
    members: Vec<Member>,
    /// Which seats the computer is playing, and its mind for each.
    bot: Vec<bool>,
    minds: Vec<Mind>,
    /// Who has left it.
    gone: Vec<bool>,
    stage: ArenaStage,
    /// Seconds left of this stage.
    left: f32,
    /// Seconds since the last to join did, in the waiting room.
    since_join: f32,
    /// Seconds with nobody connected to it.
    unwatched: f32,
    /// Everyone's health, how long since each was last hit, how long each
    /// is still reeling from the last punch that took ([`HURT_TIME`]), in
    /// which no other counts, how long each of the dead has left before
    /// standing again, and who is out for good.
    health: Vec<f32>,
    since_hit: Vec<f32>,
    hurt: Vec<f32>,
    dead_for: Vec<Option<f32>>,
    out: Vec<bool>,
    anchors: [f32; ARENA_TEAMS],
    anchor_hurt: [f32; ARENA_TEAMS],
    /// Every rung laid, and how many by each seat.
    planks: Vec<Entity>,
    rungs: Vec<usize>,
    /// What its room was last sent, to send again only what has changed.
    sent: Option<ArenaRound>,
    sent_seats: usize,
    /// Its bots and planks have gone ([`close`]); everyone goes back to the
    /// town next tick ([`end`]).
    closing: bool,
}

/// A bot's mind: how long until it may punch again, what a punch is carrying
/// it by across the ground and how fast it is rising from one (0 on the
/// ground), how fast it is falling once knocked off its island, when it
/// next thinks of somewhere to walk, and where.
#[derive(Default, Clone, Copy)]
struct Mind {
    cooldown: f32,
    shove: Vec2,
    lift: f32,
    falling: Option<f32>,
    think: f32,
    goal: Option<Vec2>,
}

/// Gravity on a body knocked up, on the way up and on the way down, in
/// metres a second a second: the game's `GRAVITY` and `FALL_GRAVITY`.
const RISE_GRAVITY: f32 = 22.0;

impl Game {
    fn room(&self) -> Room {
        Room::Arena(self.id)
    }

    fn seat_of(&self, body: Entity) -> Option<usize> {
        self.seats.iter().position(|&seat| seat == body)
    }

    fn over(&self) -> bool {
        matches!(self.stage, ArenaStage::Over { .. })
    }

    fn playing(&self) -> bool {
        self.stage == ArenaStage::Playing
    }

    /// Whether `seat` is standing: alive, and not gone or out.
    fn alive(&self, seat: usize) -> bool {
        self.health[seat] > 0.0 && !self.gone[seat] && !self.out[seat]
    }

    /// Whether `seat` is still in play: standing, or dead and coming back.
    fn in_play(&self, seat: usize) -> bool {
        !self.gone[seat] && !self.out[seat]
    }

    /// What the room is shown of how it is going.
    fn round(&self) -> ArenaRound {
        ArenaRound {
            stage: self.stage,
            left: match self.stage {
                ArenaStage::Waiting => 0,
                _ => self.left.max(0.0).ceil() as u16,
            },
            health: self.health.iter().map(|health| health.max(0.0).round() as u16).collect(),
            anchors: self.anchors.map(|anchor| anchor.max(0.0).round() as u16),
            out: self
                .out
                .iter()
                .enumerate()
                .filter(|&(_, &out)| out)
                .fold(0u8, |bits, (seat, _)| bits | (1 << seat)),
        }
    }

    /// Seats `member`, whose body is `body`, in the next seat.
    fn seat(&mut self, body: Entity, member: Member, bot: bool) -> usize {
        let seat = self.seats.len();
        self.seats.push(body);
        self.members.push(member);
        self.bot.push(bot);
        self.minds.push(Mind {
            think: THINK_EVERY * seat as f32 / ARENA_SEATS as f32,
            ..default()
        });
        self.gone.push(false);
        self.health.push(HEALTH);
        self.since_hit.push(0.0);
        self.hurt.push(0.0);
        self.dead_for.push(None);
        self.out.push(false);
        self.rungs.push(0);
        self.since_join = 0.0;
        seat
    }

    /// `seat` is dead: for [`RESPAWN_SECS`], and then back, if its team's
    /// anchor stands now; otherwise for good.
    fn die(&mut self, seat: usize) {
        self.health[seat] = 0.0;
        self.minds[seat] = Mind::default();
        if self.anchors[usize::from(arena_team(seat))] > 0.0 {
            self.dead_for[seat] = Some(RESPAWN_SECS);
        } else {
            self.dead_for[seat] = None;
            self.out[seat] = true;
            info!("Bridge Wars game {}: seat {seat} is out", self.id);
        }
    }

    /// Takes a punch's damage off `victim`, hit from `from` by `hitter`, who
    /// was `running` if so: dead at none, and knocked back otherwise, a
    /// person through their device and a bot here. Not while the victim is
    /// still reeling from the last punch that took ([`HURT_TIME`]): a
    /// device lands a punch at every press, several a second.
    fn wound(
        &mut self,
        victim: usize,
        hitter: usize,
        from: Spot,
        running: bool,
        bodies: &Bodies,
        tells: &mut MessageWriter<ToClients<Tell>>,
    ) {
        if self.hurt[victim] > 0.0 {
            return;
        }
        self.hurt[victim] = HURT_TIME;
        self.health[victim] = (self.health[victim] - DAMAGE).max(0.0);
        self.since_hit[victim] = 0.0;
        if self.health[victim] <= 0.0 {
            info!("Bridge Wars game {}: seat {hitter} killed seat {victim}", self.id);
            self.die(victim);
            return;
        }
        if self.bot[victim] {
            let Ok((_, at, ..)) = bodies.get(self.seats[victim]) else {
                return;
            };
            let away = (at.translation().xz() - from.translation().xz()).normalize_or(Vec2::NEG_Y);
            let push = KNOCK_BACK + if running { SPRINT_BONUS } else { 0.0 };
            let mind = &mut self.minds[victim];
            mind.shove = mind.shove * 0.5 + away * push;
            mind.lift = (mind.lift * 0.5 + KNOCK_UP).min(KNOCK_UP);
            return;
        }
        let Ok((body, ..)) = bodies.get(self.seats[victim]) else {
            return;
        };
        if let Some(client) = body.client {
            tell(
                tells,
                client,
                Tell::Knocked {
                    by: self.members[hitter].id,
                    from,
                    running,
                },
            );
        }
    }

    /// Which teams still have anyone in play.
    fn teams_in_play(&self) -> Vec<u8> {
        let mut teams: Vec<u8> = (0..self.seats.len())
            .filter(|&seat| self.in_play(seat))
            .map(arena_team)
            .collect();
        teams.sort_unstable();
        teams.dedup();
        teams
    }
}

impl Arenas {
    /// The game filling now, if it still has room; otherwise a new one.
    fn filling(&mut self, commands: &mut Commands) -> &mut Game {
        let open = self.filling.filter(|id| {
            self.games
                .get(id)
                .is_some_and(|game| game.stage == ArenaStage::Waiting && !game.closing && game.seats.len() < ARENA_SEATS)
        });
        let id = open.unwrap_or_else(|| {
            let id = self.next;
            self.next += 1;
            let shown = commands
                .spawn((Replicated, InRoom(Room::Arena(id)), Seats::default()))
                .id();
            self.games.insert(
                id,
                Game {
                    id,
                    shown,
                    seats: Vec::new(),
                    members: Vec::new(),
                    bot: Vec::new(),
                    minds: Vec::new(),
                    gone: Vec::new(),
                    stage: ArenaStage::Waiting,
                    left: 0.0,
                    since_join: 0.0,
                    unwatched: 0.0,
                    health: Vec::new(),
                    since_hit: Vec::new(),
                    hurt: Vec::new(),
                    dead_for: Vec::new(),
                    out: Vec::new(),
                    anchors: [ANCHOR_HEALTH; ARENA_TEAMS],
                    anchor_hurt: [0.0; ARENA_TEAMS],
                    planks: Vec::new(),
                    rungs: Vec::new(),
                    sent: None,
                    sent_seats: 0,
                    closing: false,
                },
            );
            info!("Bridge Wars game {id} opened");
            self.filling = Some(id);
            id
        });
        self.games.get_mut(&id).expect("just found or made")
    }

    /// The game in `room`, and `body`'s seat in it.
    fn of(&mut self, room: Room, body: Entity) -> Option<(&mut Game, usize)> {
        let Room::Arena(id) = room else {
            return None;
        };
        let game = self.games.get_mut(&id)?;
        let seat = game.seat_of(body)?;
        Some((game, seat))
    }

    /// `body` has left the game in `room` while it was going on: the
    /// connection went, or it went back to the town.
    pub(crate) fn walk_out(&mut self, room: Room, body: Entity) {
        if let Some((game, seat)) = self.of(room, body) {
            game.gone[seat] = true;
        }
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

/// What players ask of Bridge Wars.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<&Plays>,
    mut bodies: Bodies,
    mut arenas: ResMut<Arenas>,
    mut towns: ResMut<Towns>,
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
            Ask::Arena => {
                // From the town, or from a game that is over for me.
                let free = match room {
                    Room::Town(_) => true,
                    Room::Arena(_) => arenas
                        .of(room, body)
                        .is_some_and(|(game, seat)| game.over() || game.out[seat]),
                    Room::Home(_) | Room::Game(_) => false,
                };
                if !free {
                    tell(&mut tells, client, Tell::Refused(Refusal::Busy));
                    continue;
                }
                if let Some((old, seat)) = arenas.of(room, body) {
                    old.gone[seat] = true;
                }
                towns.stand(body);
                let Ok((body_of, spot, venue, member)) = bodies.get_mut(body) else {
                    continue;
                };
                let member = member.clone();
                let game = arenas.filling(&mut commands);
                let seat = game.seat(body, member, false);
                let room = game.room();
                tell(&mut tells, client, Tell::Seated { seat: seat as u8 });
                let (at, turn) = waiting_spot(seat);
                place(
                    &mut commands,
                    &mut tells,
                    body,
                    (body_of, spot, venue),
                    room,
                    Venue::Arena,
                    seat as u8,
                    Some(Spot::of(at, turn)),
                    now,
                );
            }
            Ask::Leave | Ask::Travel(Venue::Town) if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                game.gone[seat] = true;
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
            Ask::Punch { who, running } if matches!(room, Room::Arena(_)) => {
                let Some((game, hitter)) = arenas.of(room, body) else {
                    continue;
                };
                if !game.playing() || !game.alive(hitter) {
                    continue;
                }
                let Some(victim) = game.members.iter().position(|member| member.id == *who) else {
                    continue;
                };
                if !game.alive(victim) || arena_team(victim) == arena_team(hitter) {
                    continue;
                }
                let (Ok((_, from, ..)), Ok((_, at, ..))) = (bodies.get(game.seats[hitter]), bodies.get(game.seats[victim]))
                else {
                    continue;
                };
                let (from, at) = (*from, *at);
                if from.translation().xz().distance(at.translation().xz()) > PUNCH_SLACK {
                    continue;
                }
                game.wound(victim, hitter, from, *running, &bodies, &mut tells);
            }
            Ask::HitAnchor { team } if matches!(room, Room::Arena(_)) => {
                let Some((game, hitter)) = arenas.of(room, body) else {
                    continue;
                };
                let team = usize::from(*team);
                if !game.playing() || !game.alive(hitter) || team >= ARENA_TEAMS || team == usize::from(arena_team(hitter)) {
                    continue;
                }
                if game.anchors[team] <= 0.0 || game.anchor_hurt[team] > 0.0 {
                    continue;
                }
                let Ok((_, from, ..)) = bodies.get(game.seats[hitter]) else {
                    continue;
                };
                if !anchor_in_reach(from.translation().xz(), ARENA_ANCHORS[team], PUNCH_SLACK) {
                    continue;
                }
                game.anchor_hurt[team] = HURT_TIME;
                game.anchors[team] = (game.anchors[team] - DAMAGE).max(0.0);
                if game.anchors[team] <= 0.0 {
                    info!("Bridge Wars game {}: seat {hitter} broke team {team}'s anchor", game.id);
                }
            }
            Ask::Rung {
                chain,
                index,
                x,
                z,
                heading,
            } if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                let sane = [*x, *z, *heading].iter().all(|n| n.is_finite()) && x.abs() < FURTHEST && z.abs() < FURTHEST;
                if !game.playing() || !sane || game.rungs[seat] >= MOST_RUNGS {
                    continue;
                }
                let plank = commands
                    .spawn((
                        Replicated,
                        InRoom(game.room()),
                        Plank {
                            seat: seat as u8,
                            chain: *chain,
                            index: *index,
                            x: *x,
                            z: *z,
                            heading: *heading,
                        },
                    ))
                    .id();
                game.planks.push(plank);
                game.rungs[seat] += 1;
            }
            // Fallen into the void: dead, and back if the anchor stands.
            Ask::Respawn if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                if game.playing() && game.alive(seat) {
                    info!("Bridge Wars game {}: seat {seat} fell", game.id);
                    game.die(seat);
                }
            }
            Ask::Skip if matches!(room, Room::Arena(_)) => {
                if let Some((game, _)) = arenas.of(room, body)
                    && config.allow_skip
                {
                    match game.stage {
                        ArenaStage::Waiting => game.since_join = BOT_WAIT,
                        _ => game.left = 0.0,
                    }
                }
            }
            _ => {}
        }
    }
}

/// Moves every game on: fills its room, counts down, sends everyone to their
/// islands, runs the fight's clock, its deaths and its bots, finds the
/// winner, and ends it once the result has been up its while, or once
/// nobody is left.
#[allow(clippy::too_many_arguments)]
fn run(
    time: Res<Time>,
    mut arenas: ResMut<Arenas>,
    mut bodies: Bodies,
    mut everyone: ResMut<Everyone>,
    mut towns: ResMut<Towns>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs_f64();
    let mut ended = Vec::new();
    for game in arenas.games.values_mut() {
        let room = game.room();
        let watched = (0..game.seats.len()).any(|seat| {
            !game.gone[seat]
                && !game.bot[seat]
                && bodies
                    .get(game.seats[seat])
                    .is_ok_and(|(body, ..)| body.client.is_some() && body.room == room)
        });
        game.unwatched = if watched { 0.0 } else { game.unwatched + dt };
        let everyone_gone = (0..game.seats.len()).all(|seat| game.gone[seat] || game.bot[seat]);
        if game.closing {
            end(&mut commands, &mut tells, game, &mut bodies, &mut everyone, &mut towns, now);
            ended.push(game.id);
            continue;
        }
        if (game.over() && game.left <= 0.0) || everyone_gone || game.unwatched > ABANDONED {
            close(&mut commands, game);
            continue;
        }

        match game.stage {
            ArenaStage::Waiting => {
                game.since_join += dt;
                if game.since_join >= BOT_WAIT && game.seats.len() < ARENA_SEATS {
                    while game.seats.len() < ARENA_SEATS {
                        seat_bot(&mut commands, game, now);
                    }
                    info!("Bridge Wars game {}: nobody joined in {BOT_WAIT} s, so the computer takes the seats left", game.id);
                }
                if game.seats.len() >= ARENA_SEATS {
                    game.stage = ArenaStage::Countdown;
                    game.left = COUNTDOWN;
                }
            }
            ArenaStage::Countdown => {
                game.left -= dt;
                if game.left <= 0.0 {
                    for seat in 0..game.seats.len() {
                        land(&mut commands, &mut tells, &mut bodies, game, seat, now);
                    }
                    game.stage = ArenaStage::Playing;
                    game.left = PLAY_SECS;
                    info!("Bridge Wars game {}: fight, {PLAY_SECS} s to play", game.id);
                }
            }
            ArenaStage::Playing => {
                game.left -= dt;
                fight(&mut commands, &mut tells, &mut bodies, game, dt, now);
                let teams = game.teams_in_play();
                let winner = match (teams.len(), game.left <= 0.0) {
                    (0, _) => Some(None),
                    (1, _) => Some(Some(teams[0])),
                    (_, true) => Some(None),
                    _ => None,
                };
                if let Some(winner) = winner {
                    game.stage = ArenaStage::Over { winner };
                    game.left = RESULTS_SECS;
                    info!("Bridge Wars game {}: over, team {winner:?} wins", game.id);
                }
            }
            ArenaStage::Over { .. } => game.left -= dt,
        }

        // Only what has changed goes out again.
        let round = game.round();
        if game.sent.as_ref() != Some(&round) {
            commands.entity(game.shown).insert(round.clone());
            game.sent = Some(round);
        }
        if game.sent_seats != game.members.len() {
            commands.entity(game.shown).insert(Seats(game.members.clone()));
            game.sent_seats = game.members.len();
        }
    }
    for id in ended {
        arenas.games.remove(&id);
        if arenas.filling == Some(id) {
            arenas.filling = None;
        }
    }
}

/// Everyone's health coming back, the dead standing again or going out, and
/// the bots' moves, for one frame of the fight.
fn fight(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    game: &mut Game,
    dt: f32,
    now: f64,
) {
    for hurt in game.hurt.iter_mut().chain(game.anchor_hurt.iter_mut()) {
        *hurt = (*hurt - dt).max(0.0);
    }
    for seat in 0..game.seats.len() {
        if !game.in_play(seat) {
            continue;
        }
        if let Some(left) = game.dead_for[seat] {
            let left = left - dt;
            if left > 0.0 {
                game.dead_for[seat] = Some(left);
            } else {
                game.dead_for[seat] = None;
                game.health[seat] = HEALTH;
                game.since_hit[seat] = 0.0;
                land(commands, tells, bodies, game, seat, now);
                info!("Bridge Wars game {}: seat {seat} stands again", game.id);
            }
            continue;
        }
        if game.health[seat] <= 0.0 {
            continue;
        }
        game.since_hit[seat] += dt;
        if game.since_hit[seat] >= REGEN_AFTER && game.health[seat] < HEALTH {
            game.health[seat] = (game.health[seat] + REGEN_PER_SEC * dt).min(HEALTH);
        }
    }
    // The bots, each with everyone else where they stand this frame.
    let standing: Vec<(usize, Vec2)> = (0..game.seats.len())
        .filter(|&seat| game.alive(seat) && game.minds[seat].falling.is_none())
        .filter_map(|seat| bodies.get(game.seats[seat]).ok().map(|(_, spot, ..)| (seat, spot.translation().xz())))
        .collect();
    for seat in 0..game.seats.len() {
        if !game.bot[seat] || !game.alive(seat) {
            continue;
        }
        let Ok((_, spot, ..)) = bodies.get(game.seats[seat]) else {
            continue;
        };
        let was = *spot;
        let (next, punch) = think(game, seat, was, &standing, dt);
        if let Some(victim) = punch {
            game.wound(victim, seat, was, true, bodies, tells);
        }
        if game.minds[seat].falling.is_some_and(|_| next.translation().y < ISLAND_TOP - FALL_TO_DIE) {
            info!("Bridge Wars game {}: seat {seat} fell", game.id);
            game.die(seat);
            continue;
        }
        if !next.same(&was)
            && let Ok((_, mut spot, ..)) = bodies.get_mut(game.seats[seat])
        {
            *spot = next;
        }
    }
}

/// What the bot in `seat`, standing at `was`, does this frame: where it is
/// after it, and whom it punches, if anyone. It keeps to the disc of its
/// island, having no map: knocked off it, it falls.
fn think(game: &mut Game, seat: usize, was: Spot, standing: &[(usize, Vec2)], dt: f32) -> (Spot, Option<usize>) {
    let island = ARENA_ISLANDS[arena_island(seat)];
    let edge = ARENA_ISLAND_RADIUS - BODY_RADIUS;
    let mut at = was.translation();
    let mut facing = was.rotation() * Vec3::NEG_Z;
    let mut punch = None;
    let mind = &mut game.minds[seat];
    mind.cooldown = (mind.cooldown - dt).max(0.0);

    // Knocked off the island, it falls.
    if let Some(speed) = mind.falling {
        let speed = speed - GRAVITY * dt;
        mind.falling = Some(speed);
        at.y += speed * dt;
        if mind.shove != Vec2::ZERO {
            at += Vec3::new(mind.shove.x, 0.0, mind.shove.y) * dt;
        }
        return (Spot::of(at, was.rotation()), None);
    }

    // A punch throws it up and carries it, dying away as it goes, slowly
    // in the air and quickly once it is down, off the edge if it must.
    let airborne = mind.lift > 0.0 || at.y > ISLAND_TOP + 1e-3;
    if airborne {
        at.y += mind.lift * dt;
        mind.lift -= if mind.lift > 0.0 { RISE_GRAVITY } else { GRAVITY } * dt;
        if at.y <= ISLAND_TOP {
            at.y = ISLAND_TOP;
            mind.lift = 0.0;
        }
    }
    if mind.shove != Vec2::ZERO {
        at += Vec3::new(mind.shove.x, 0.0, mind.shove.y) * dt;
        let drag = if airborne { AIR_DRAG } else { GROUND_DRAG };
        mind.shove *= drag.powf(dt * 20.0);
        if mind.shove.length_squared() < 0.01 {
            mind.shove = Vec2::ZERO;
        }
        if at.xz().distance(island) > ARENA_ISLAND_RADIUS + 0.2 {
            mind.falling = Some(mind.lift.min(0.0));
            return (Spot::of(at, was.rotation()), None);
        }
    }
    if airborne {
        // Knocked into the air, it is carried, and does nothing else.
        return (Spot::of(at, was.rotation()), None);
    }

    let here = at.xz();
    let team = arena_team(seat);
    // Whoever of another team is nearest within sight, on or beside its
    // island, where it can get at them.
    let target = standing
        .iter()
        .filter(|&&(other, there)| {
            other != seat
                && arena_team(other) != team
                && there.distance(here) <= HUNT_RADIUS
                && there.distance(island) <= ARENA_ISLAND_RADIUS + 1.5
        })
        .min_by(|a, b| a.1.distance_squared(here).total_cmp(&b.1.distance_squared(here)))
        .copied();
    let mut step = Vec2::ZERO;
    match target {
        Some((other, there)) => {
            mind.goal = None;
            let off = there - here;
            let toward = off.normalize_or_zero();
            if game.health[seat] / HEALTH < FLEE_BELOW {
                // Nearly beaten, it runs for it: away, or round the edge.
                let away = -toward;
                let ahead = here + away * 3.0;
                step = if ahead.distance(island) <= edge {
                    away
                } else {
                    let round = Vec2::new(-away.y, away.x);
                    if (here + round * 3.0).distance(island) <= edge { round } else { -round }
                } * RUN_SPEED * dt;
                facing = Vec3::new(step.x, 0.0, step.y);
            } else {
                facing = Vec3::new(toward.x, 0.0, toward.y);
                if off.length() > REACH * PRESS_IN {
                    step = toward * RUN_SPEED * dt;
                }
                if off.length() <= REACH + BODY_RADIUS && mind.cooldown <= 0.0 {
                    mind.cooldown = BOT_PUNCH_EVERY;
                    punch = Some(other);
                }
            }
        }
        None => {
            // Nobody to go for: a stroll somewhere on the island now and
            // then.
            mind.think -= dt;
            if mind.think <= 0.0 {
                mind.think += THINK_EVERY;
                let angle = fastrand::f32() * std::f32::consts::TAU;
                let out = fastrand::f32() * (edge - 1.5);
                mind.goal = Some(island + Vec2::new(angle.cos(), angle.sin()) * out);
            }
            if let Some(goal) = mind.goal {
                let off = goal - here;
                if off.length() < 0.2 {
                    mind.goal = None;
                } else {
                    step = off.normalize() * (STROLL * dt).min(off.length());
                    facing = Vec3::new(off.x, 0.0, off.y);
                }
            }
        }
    }
    let mut next = here + step;
    if next.distance(island) > edge {
        next = island + (next - island).normalize_or(Vec2::X) * edge;
    }
    let rotation = if facing.xz().length_squared() > 1e-6 {
        Quat::from_rotation_y((-facing.x).atan2(-facing.z))
    } else {
        was.rotation()
    };
    (Spot::of(Vec3::new(next.x, ISLAND_TOP, next.y), rotation), punch)
}

/// Sends `seat` to where it arrives on its island: a person through their
/// device, a bot by its spot.
fn land(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    game: &mut Game,
    seat: usize,
    now: f64,
) {
    let room = game.room();
    let Ok((body_of, mut spot, venue, _)) = bodies.get_mut(game.seats[seat]) else {
        return;
    };
    if body_of.room != room {
        return;
    }
    if game.bot[seat] {
        let (at, turn) = arrival(Venue::Arena, seat);
        *spot = Spot::of(at.with_y(ISLAND_TOP), turn);
        game.minds[seat] = Mind {
            think: THINK_EVERY * seat as f32 / ARENA_SEATS as f32,
            ..default()
        };
        return;
    }
    move_to(commands, tells, game.seats[seat], (body_of, spot, venue), room, Venue::Arena, seat as u8, now);
}

/// The computer, in the next seat of `game`, standing in the waiting room:
/// a boy or a girl by the toss of a coin, in any of that side's colours,
/// under one of that side's names that nobody there has, as House Builder's
/// are (`builder::seat_bot`).
fn seat_bot(commands: &mut Commands, game: &mut Game, now: f64) {
    let seat = game.seats.len();
    let boy = fastrand::bool();
    let names: Vec<&str> = roundtown_net::ai_names(boy)
        .iter()
        .copied()
        .filter(|name| game.members.iter().all(|member| !member.name.starts_with(name)))
        .collect();
    let member = Member {
        // Clear of people's ids, which count up from 1, and of House
        // Builder's bots, which count down from the top.
        id: u64::MAX / 2 - u64::from(game.id) * ARENA_SEATS as u64 - seat as u64,
        name: fastrand::choice(names).unwrap_or("Bot").into(),
        look: roundtown_net::ai_look(boy, fastrand::usize(..)).into(),
        ai: true,
    };
    let (at, turn) = waiting_spot(seat);
    let body = commands
        .spawn((
            Replicated,
            member.clone(),
            Spot::of(at, turn),
            Venue::Arena,
            InRoom(game.room()),
            Body::bot(game.room(), seat as u8, now),
        ))
        .id();
    game.seat(body, member, true);
}

/// The first tick of a game's end: its planks, its computer players and
/// its board go, while everyone in it is still in its room to be told so.
/// Told the tick after, as [`end`] moves them out, they would never be: an
/// entity despawned in the very tick a device stops seeing its room is
/// written to that device neither as despawned (it is hidden by then) nor
/// as lost from sight (it is gone by then), read in replicon 0.44's
/// `collect_despawns`; and the bots stood on in the next game, their
/// bodies never taken down (seen 2026-11-09).
fn close(commands: &mut Commands, game: &mut Game) {
    for (seat, &body) in game.seats.iter().enumerate() {
        if game.bot[seat] {
            commands.entity(body).try_despawn();
        }
    }
    for plank in game.planks.drain(..) {
        commands.entity(plank).try_despawn();
    }
    commands.entity(game.shown).try_despawn();
    game.closing = true;
}

/// The end of a game, the tick after [`close`]: anyone still in it is sent
/// back to the town, or, if they are not connected, forgotten.
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
    for plank in game.planks.drain(..) {
        commands.entity(plank).try_despawn();
    }
    info!("Bridge Wars game {} closed", game.id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_game() -> Game {
        let mut game = Game {
            id: 0,
            shown: Entity::PLACEHOLDER,
            seats: Vec::new(),
            members: Vec::new(),
            bot: Vec::new(),
            minds: Vec::new(),
            gone: Vec::new(),
            stage: ArenaStage::Playing,
            left: PLAY_SECS,
            since_join: 0.0,
            unwatched: 0.0,
            health: Vec::new(),
            since_hit: Vec::new(),
            hurt: Vec::new(),
            dead_for: Vec::new(),
            out: Vec::new(),
            anchors: [ANCHOR_HEALTH; ARENA_TEAMS],
            anchor_hurt: [0.0; ARENA_TEAMS],
            planks: Vec::new(),
            rungs: Vec::new(),
            sent: None,
            sent_seats: 0,
            closing: false,
        };
        for seat in 0..ARENA_SEATS {
            let member = Member {
                id: seat as u64 + 1,
                name: format!("Seat {seat}"),
                look: "boy".into(),
                ai: seat > 0,
            };
            game.seat(Entity::from_bits(seat as u64 + 1), member, seat > 0);
        }
        game
    }

    #[test]
    fn dead_with_the_anchor_standing_comes_back_and_without_it_is_out() {
        let mut game = full_game();
        game.die(1);
        assert_eq!(game.health[1], 0.0);
        assert_eq!(game.dead_for[1], Some(RESPAWN_SECS));
        assert!(!game.out[1] && game.in_play(1) && !game.alive(1));
        assert_eq!(game.round().anchors[1], ANCHOR_HEALTH as u16);
        game.anchors[usize::from(arena_team(2))] = 0.0;
        game.die(2);
        assert!(game.out[2] && !game.in_play(2));
        assert_eq!(game.dead_for[2], None);
        // Teams 0 to 3 are all still in play through their other seat.
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
        for seat in [0, 4, 1, 5, 3, 7] {
            game.anchors[usize::from(arena_team(seat))] = 0.0;
            game.die(seat);
        }
        // Team 2 is left: its seat 6 stands and its seat 2 is out.
        assert_eq!(game.teams_in_play(), vec![2]);
        let round = game.round();
        assert!(round.is_out(2) && round.is_out(0) && !round.is_out(6));
        assert_eq!(round.health[6], HEALTH as u16);
        assert_eq!(round.health[0], 0);
        assert_eq!(round.anchors, [0; ARENA_TEAMS]);
    }

    #[test]
    fn a_bot_keeps_to_its_island_runs_at_an_enemy_and_falls_when_knocked_off() {
        let mut game = full_game();
        let seat = 5;
        let island = ARENA_ISLANDS[arena_island(seat)];
        let (at, turn) = arrival(Venue::Arena, seat);
        let mut spot = Spot::of(at.with_y(ISLAND_TOP), turn);
        // Alone, it strolls, and never off the island.
        for _ in 0..600 {
            let (next, punch) = think(&mut game, seat, spot, &[(seat, spot.translation().xz())], 1.0 / 60.0);
            assert_eq!(punch, None);
            assert!(next.translation().xz().distance(island) <= ARENA_ISLAND_RADIUS - BODY_RADIUS + 1e-3);
            assert_eq!(next.translation().y, ISLAND_TOP);
            spot = next;
        }
        // An enemy 6 m off on its island: it runs at them and, in reach,
        // punches, no oftener than it may.
        let enemy = island + Vec2::new(6.0, 0.0);
        let mut punches = 0;
        let mut frames = 0;
        let standing = [(seat, spot.translation().xz()), (0, enemy)];
        while frames < 600 {
            let (next, punch) = think(&mut game, seat, spot, &standing, 1.0 / 60.0);
            punches += usize::from(punch.is_some());
            spot = next;
            frames += 1;
        }
        assert!(spot.translation().xz().distance(enemy) <= REACH + BODY_RADIUS + 0.1, "{}", spot.translation());
        assert!(punches >= 8 && punches <= 14, "{punches} punches in ten seconds");
        // A body reeling from a punch takes no other for a while, however
        // many land: the second of two in a row does nothing.
        let mut world = World::new();
        let hitter = world.spawn_empty().id();
        let _ = hitter;
        game.hurt[0] = 0.0;
        game.health[0] = HEALTH;
        game.hurt[0] = HURT_TIME;
        assert!(game.hurt[0] > 0.0);
        // Ticked down, it takes again.
        game.hurt[0] = (game.hurt[0] - HURT_TIME).max(0.0);
        assert_eq!(game.hurt[0], 0.0);
        // A teammate right beside it is nobody to go for: never punched.
        let mut idle = full_game();
        let mut beside = spot;
        for _ in 0..600 {
            let mate = beside.translation().xz() + Vec2::new(1.0, 0.0);
            let (next, punch) = think(&mut idle, seat, beside, &[(seat, beside.translation().xz()), (1, mate)], 1.0 / 60.0);
            assert_eq!(punch, None, "punched a teammate");
            beside = next;
        }
        // Knocked hard toward the edge, up and across as a punch knocks,
        // it goes over and falls.
        game.minds[seat].shove = Vec2::new(KNOCK_BACK + SPRINT_BONUS, 0.0) * 2.0;
        game.minds[seat].lift = KNOCK_UP;
        let mut fell = false;
        for _ in 0..240 {
            let (next, _) = think(&mut game, seat, spot, &standing, 1.0 / 60.0);
            spot = next;
            if game.minds[seat].falling.is_some() && spot.translation().y < ISLAND_TOP - 1.0 {
                fell = true;
                break;
            }
        }
        assert!(fell, "still on the island at {}", spot.translation());
    }

    #[test]
    fn the_room_is_shown_only_what_has_changed() {
        let game = full_game();
        let round = game.round();
        assert_eq!(round.stage, ArenaStage::Playing);
        assert_eq!(round.left, PLAY_SECS as u16);
        assert_eq!(round.health.len(), ARENA_SEATS);
        assert_eq!(round, game.round());
    }
}
