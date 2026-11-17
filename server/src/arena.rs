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
//! the computer takes every seat left, as the game does offline, and the
//! room counts down and goes: its players come in one at a time, a thrown
//! while apart ([`Game::bots_due`]), and go about the waiting room each as
//! who it is until the fight ([`Game::mill`], `roundtown_net::bots::mingle`;
//! since 2026-11-16, when they all came in the one frame and stood on a
//! ring). The computer's players are moved here
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
//!
//! **Since 2026-11-11 the generators, what they make, what everyone holds
//! and the shop are the server's too** (Hajun: "these generators/items in
//! map need to be server synced so other players see the same number of
//! items on map at the same times and other player taking the item from
//! the generators make it so you cant take it too"). Each game has a
//! [`Mint`], the generators' rules, which both sides share
//! (`roundtown_net::bridge_wars`); everything it makes is an entity the
//! room is sent ([`Loot`]), and whoever of the people in the game comes
//! within the magnet's reach of one, where they last said they stood, has
//! it: it is gone for everyone that tick, and theirs in the room's
//! [`Purses`]. A thing drops onto its pad from high over it as it is
//! made, and is within reach only once it has come down to whoever is
//! there, half a metre over their head ([`REACH_OVER`], since 2026-11-12;
//! for an evening before, not until it had landed and lain a moment), so
//! that someone standing on the pad sees it come, and someone who jumps
//! for it has it sooner, in the air. The computer's players pick up
//! nothing. [`Ask::Buy`] takes
//! a ware's price out of the asker's purse and tells them it is theirs
//! ([`Tell::Bought`]). And the game keeps Hajun's timetable ([`TIMETABLE`]):
//! the generators' tiers are the mint's own by the time; at fourteen
//! minutes every anchor left breaks; from fifteen, sudden death, the
//! zombies come, a few and then more, to fifty ([`zombies_at`],
//! [`Game::raise_zombies`]), seated past the players' eight seats, each
//! with the mind the device's have offline ([`Horde`]): going for whoever
//! of the players is nearest over whatever can be walked, hurt by nothing,
//! and back up a while after the void has had it; and at twenty it is a
//! draw ([`PLAY_SECS`]). (Since the night of 2026-11-16: the anchors broke
//! at ten until then, the zombies were twelve, all at once, each kept to
//! the island it rose on and killed as anyone is, and a game was twenty
//! five minutes.)
//!
//! **Since 2026-11-12 the two shops' wares are the server's as well**
//! (Hajun's `bridgewarsgameevent.txt`). What everyone has and what each
//! team has bought is the game's [`Outfits`], the shops' rules, which
//! both sides share, and its room is sent them as [`Kits`] and
//! [`Upgraded`]: a punch takes more with a sword and less through a
//! shield ([`Game::wound`]); a fireball thrown ([`Ask::Use`]) is an entity
//! the room is sent ([`Fireball`]) that the server flies and bursts
//! ([`fireballs`]); a potion drunk is the kit's until it wears off; a
//! team's generator is as fast as the team has raised it; its heal pool
//! heals it on its island, and its traps blind whoever of another team
//! steps onto it ([`Outfits::tick`]). The computer's players buy nothing,
//! and do not go for anyone unseen.
//!
//! **Since 2026-11-15 a seat waits for whoever's device has gone**
//! (Hajun, of his partner's phone in a game with him: "suddenly she
//! disconnected and went back to the main lobby for some reason while i
//! still stay in the bridge wars game ... if u exit the app while ur
//! playing and reopen, u shouldnt go to the main lobby - u should rejoin
//! the game as long as there is atleast 1 real player in that game
//! still"). A phone closes its connection whenever its app leaves the
//! foreground, an iPhone's for as little as a pull on the bar of
//! notifications, and until then that was the seat given up for good: the
//! journal has her device gone for 3.8 s, a minute into a game, and back
//! in the town. Now the body is kept ([`Arenas::keeps`]), standing where
//! it stood with nobody to move it, to be hit like anyone and to fall if
//! the ground goes from under it ([`Game::fight`]); and whoever comes back
//! as its player is put back ([`Arenas::returns`]): an app that was only
//! put away carries on where it stands, whoever else is or is not in the
//! game, and one that was closed and opened again is told to begin the
//! game afresh ([`Tell::Back`]) so long as the fight is on and someone
//! else of the people in it is connected. With nobody connected a game
//! goes on a minute and is given up ([`ABANDONED`]). Before the fight a
//! seat waits only until it begins, and is the computer's then ([`run`]).
//!
//! **And since 2026-11-15 a stick of dynamite takes the whole bridge
//! down** ([`Game::dynamites`], [`Game::crumble`], `Nav::cascade`): the
//! planks its blast reaches at once, as before, and every other plank
//! joined to them after, one by one from the hole outward.
//!
//! **Since 2026-11-16 two buses drive round the middle of the map, and
//! run over whoever is in front of one** (Hajun: "i want 2 buses to go 2x
//! normal speed around the road. and when a person is standing in front
//! of the moving bus, the bus 'squishes' that person, so the person is
//! squashed on the road for 10 second ... this goes for ai too").
//! Where each bus is is a closed form of the clock
//! (`roundtown_net::bridge_wars::Bus`), this machine's, which every device
//! has for the town's: nothing of them is sent. A person's own device says
//! a bus has run its player over ([`Ask::RunOver`]), as it says they have
//! fallen: the game looks at whoever it moves itself, the computer's
//! players, the zombies and whoever's device has gone
//! ([`Game::buses`]). Whoever is run over has [`SQUASH_DAMAGE`] off their
//! health, through their shield, and lies flat for [`SQUASH_SECS`]
//! ([`Game::run_over`]): the computer's stand where they are; anyone is
//! hit there as ever, and knocked nowhere; no bus runs them over again
//! until they are up; and the room is sent who lies flat and until when
//! ([`Flattened`]). The computer's players mind the buses, or do not, and
//! some of them go for whoever is lying there (`Squad::mind_buses`,
//! `Squad::spot_prey`).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::bots::mingle::{Mingle, Stood};
use roundtown_net::bots::{Seen, Skill, Squad, Stick, View};
use roundtown_net::bridge_wars::*;
use roundtown_net::horde::{Horde, Prey};
use roundtown_net::nav::Nav;
use roundtown_net::chat::{CREDIT_SECS, Feed, Weapon};
use roundtown_net::nav::PLANK_TOP;
use roundtown_net::{Blasted, BuilderOf, Dynamite, Flattened};
use roundtown_net::social::{Minigame, XP_PLAYED, XP_WON};
use roundtown_net::{
    ARENA_ANCHORS, ARENA_ISLAND_RADIUS, ARENA_ISLANDS, ARENA_SEATS, ARENA_TEAMS, ArenaRound,
    ArenaStage, Ask, Fireball, Kit, Kits, Loot, Member, Plank, Purse, Purses, Refusal, Seats, Spot,
    Tell, Upgraded, Upgrades, Use, Utility, Venue, Ware, arena_island, arena_team, arrival,
};

use crate::Config;
use crate::fees::Paying;
use crate::players::{Body, Everyone, Plays, Who, move_to, place, tell};
use crate::rooms::{InRoom, NotFor, Room, Towns};
use crate::social::Social;
use crate::tables::Tables;

/// How long a game goes on with nobody connected to it, in seconds, before
/// it is given up: the seats of whoever's devices have gone waiting all
/// the while ([`Arenas::keeps`]), so that someone playing alone whose app
/// was put away for less than this finds their game as they come back to
/// it.
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
/// The most builders one seat can have out for the room to be sent
/// ([`BuilderOf`]), and how long one goes on being sent with nothing heard
/// of it from its owner's device, in seconds: a device says where its
/// builder is as it walks, and that it is gone.
const MOST_BUILDERS: usize = 4;
const BUILDER_SILENT: f64 = 2.0;
/// How far under the ground where they stand someone whose device has
/// gone has to be to have fallen, in metres: under the islands and the
/// planks, on the way down into the void.
const AWAY_UNDER: f32 = 1.5;
/// The most metres of bridge a device can say it has left
/// ([`Ask::Metres`]): far more than a game's worth.
const MOST_METRES: f32 = 5000.0;
/// How far from the middle of a bus's front, where it is now or was in
/// the last moments, someone can be where they last said they stood for
/// their device's word that it has run them over to count
/// ([`Ask::RunOver`]), in metres: across half its width and a little, and
/// the way they and it may have gone since.
const RUN_OVER_SLACK: f32 = 5.5;

/// The time by this machine's clock, in seconds since 1970: what every
/// device has for the town's clock (the game's `clock`), and what the
/// buses go by ([`Leg`]); and the four games played at tables (`tables`).
pub(crate) fn wall_clock() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |since| since.as_secs_f64())
}

/// Which of Bridge Wars' buses someone whose place is `spot` is riding,
/// from 0, if they are on one ([`Spot::aboard`], [`FIRST_BUS`]).
fn riding(spot: &Spot) -> Option<usize> {
    spot.aboard.and_then(|car| car.checked_sub(FIRST_BUS)).map(usize::from).filter(|bus| *bus < BUSES)
}

/// Where someone whose place is `spot` stands in the world of a game:
/// where it says, or, riding one of the buses, where the bus has taken
/// that place on it, by the `clock`, the game's buses going as they have
/// (`buses`).
fn stands(spot: &Spot, clock: f64, buses: &BusLog) -> Vec3 {
    match riding(spot).and_then(|bus| buses.at(bus, clock)) {
        Some(bus) => bus.carries(spot.translation()),
        None => spot.translation(),
    }
}

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
    /// The order its seats are taken in as people join: [`SEAT_ORDER`],
    /// begun on a team thrown for this game ([`seat_order`]).
    order: [usize; ARENA_SEATS],
    /// The seats nobody is sitting in: between two that are taken, as the
    /// seats are taken out of order ([`Game::order`]), and whoever's they
    /// were who left before the fight began ([`Game::leave`]). Such a
    /// seat is gone and out, with no body, until someone takes it; the
    /// computer takes every one left as the wait ends, and again as the
    /// countdown does, so that the fight begins with all eight taken.
    vacant: Vec<bool>,
    stage: ArenaStage,
    /// Seconds left of this stage.
    left: f32,
    /// Seconds since the last to join did, in the waiting room.
    since_join: f32,
    /// The wait over, the computer has begun taking the seats left, one
    /// of its players at a time ([`Game::bots_due`]; since 2026-11-16):
    /// how long until the next comes, in seconds. None until then.
    coming: Option<f32>,
    /// Someone has skipped the wait ([`Ask::Skip`]): the computer takes
    /// every seat left at once, as it did for everyone until 2026-11-16.
    hurried: bool,
    /// The computer's players in the waiting room, each passing the time
    /// as who it is ([`Mingle`], [`Game::mill`]; since 2026-11-16), and
    /// what throws where anyone who comes into the room stands.
    mingle: Mingle,
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
    /// How far into the fight it is, in seconds: what the timetable goes
    /// by ([`TIMETABLE`]), and the generators' tiers.
    elapsed: f32,
    /// The time by this machine's clock, in seconds since 1970, as of
    /// this frame ([`run`]): what a blast is stamped with ([`Blasted::at`]),
    /// every device having the server's time for the town's clock. None
    /// of the game goes by it.
    clock: f64,
    /// The generators, and everything they have made that lies there yet,
    /// each by its number as the entity the room is sent ([`Loot`]).
    mint: Mint,
    loot: HashMap<u32, Entity>,
    /// What everyone holds, by seat.
    purses: Vec<Purse>,
    /// What everyone has and what each team has bought, by the shops'
    /// rules.
    outfits: Outfits,
    /// Every fireball in the air: its number, how far it has flown, and
    /// who threw it, by seat; and the entity its room is sent for each, by
    /// its number.
    balls: Vec<(u32, Flight, usize)>,
    next_ball: u32,
    shown_balls: HashMap<u32, Entity>,
    /// Every stick of dynamite thrown and not yet blown ([`Lobbed`]), and
    /// the entity its room is sent for each, by its number; and how many
    /// have blown, and how many planks they took with them, for reading a
    /// game back by.
    sticks: Vec<Lobbed>,
    next_stick: u32,
    shown_sticks: HashMap<u32, Entity>,
    blasts: u32,
    planks_blown: u32,
    /// How many deaths on land dropped anything ([`Game::drop_spoils`]),
    /// how many sapphires and amethysts they dropped between them, how
    /// many of those were picked up, and how many went with a plank they
    /// lay on, for reading a game back by.
    spilt: [u32; 4],
    /// Where everyone stands and which way they face, by seat: a
    /// person's where they last said, copied in each frame ([`sync`]); the
    /// computer's players' where the game has moved them, which is what
    /// their bodies are sent as ([`flush`]). Kept here, and not read off
    /// the bodies as it goes, so that a fight is a plain function of the
    /// game ([`Game::step`]) and whole games can be played in a test; and
    /// to the last of a metre, where a body's [`Spot`] is whole
    /// centimetres, which a slow step rounds to nothing.
    at: Vec<Vec3>,
    yaw: Vec<f32>,
    /// Whose body is in the game's room to be played with, by seat: a
    /// person's while it is there; the computer's always.
    present: Vec<bool>,
    /// Whose device has gone, by seat, their body standing in the game
    /// where it stood with nobody to move it and their seat waiting for
    /// them ([`Arenas::keeps`], [`sync`]; since 2026-11-15); and which of
    /// them has died since it went, so that the device, when it is back,
    /// is told to begin the game afresh rather than carry on with what it
    /// held ([`Arenas::returns`]).
    away: Vec<bool>,
    died_away: Vec<bool>,
    /// The metres of bridge each person has left to lay, as their device
    /// last said ([`Ask::Metres`]): the device's own to count, and kept
    /// here only to be given back to one that has forgotten
    /// ([`Tell::Back`]). None once they have died, as on the device.
    metres: Vec<f32>,
    /// Whoever a bus has run over ([`Game::run_over`], since 2026-11-16):
    /// how much longer each lies flat on the road, in seconds, by seat,
    /// nothing for whoever stands; and until when, by the clock the buses
    /// go by, which is what the room is sent ([`Flattened`],
    /// [`Game::sent_flat`]). The clock the buses stand by this frame
    /// ([`Game::step`]): this machine's, or, in a game played out with
    /// none, as a test plays one, the fight's own time. And how many have
    /// been run over, how many of them that killed, and how many blows
    /// landed on someone lying there, for reading a game back by.
    flat: Vec<f32>,
    flat_until: Vec<f64>,
    bus_clock: Option<f64>,
    squashes: [u32; 3],
    /// How the buses are going (since 2026-11-17, [`Buses`]): thrown as
    /// the game is begun, and changed as one slows for someone on a
    /// crossing, stops, and goes on ([`Game::drive_buses`]); what the room
    /// is sent whenever it changes, and how they have gone the last few
    /// seconds, for where one was a moment ago ([`BusLog`]). And whose
    /// body, by seat, a person's, is riding one of them ([`sync`]): on a
    /// crossing, a bus does not stop for whoever it is carrying over it.
    buses: Option<Buses>,
    bus_log: BusLog,
    aboard: Vec<bool>,
    /// What a step of the fight has done that the world is to be told or
    /// shown ([`Event`]), until it has been ([`flush`]).
    events: Vec<Event>,
    /// The map as the computer's players know it, with every bridge laid
    /// in this game on it ([`Nav`]): people's, as their rungs come
    /// ([`Ask::Rung`]), and the bots' own.
    nav: Nav,
    /// The zombies of sudden death, in the order they rose: the seats past
    /// the players' eight, the first of them the first zombie's
    /// (`roundtown_net::horde`).
    horde: Horde,
    /// The computer's players in the players' seats ([`Squad`]): what
    /// each is doing, and its body, which is what moves it. The zombies
    /// are not theirs: they keep their minds ([`Mind`], [`think`]).
    squad: Squad,
    /// The computer's players as they were until 2026-11-12, each kept to
    /// its island by a [`Mind`] ([`think`]): for going back to without a
    /// deploy, should the new ones go wrong (`RT_ARENA_BOTS=old`).
    old_bots: bool,
    /// The game's own dice, for the generators and the computer's
    /// players: seeded, so that a game played in a test plays the same
    /// each time.
    dice: fastrand::Rng,
    /// Who last hit each seat, how far into the fight, and with what: a
    /// fall soon after is their doing ([`CREDIT_SECS`], [`Game::fall`]).
    last_hit: Vec<Option<(usize, f32, Weapon)>>,
    /// The builders people's Bridge Builders have sent off, as the room
    /// is sent them ([`BuilderOf`], since 2026-11-14): by seat and the
    /// owner's number for it, the body the room is sent and when its
    /// owner's device last said where it was ([`Ask::Builder`]).
    builders: HashMap<(usize, u8), (Entity, f64)>,
    /// Where the builder each of the computer's players has out is now,
    /// by seat, and which way it faces ([`Squad::builder_at`]); and the
    /// body the room is sent for each, with where it was last sent as.
    bot_builders: [Option<(Vec3, f32)>; ARENA_SEATS],
    shown_bot_builders: HashMap<usize, (Entity, Spot)>,
    /// What its room was last sent, to send again only what has changed.
    sent: Option<ArenaRound>,
    sent_seats: Vec<Member>,
    sent_purses: Vec<Purse>,
    sent_kits: Vec<Kit>,
    sent_upgrades: [Upgrades; ARENA_TEAMS],
    sent_flat: Flattened,
    sent_buses: Option<Buses>,
    /// Its bots and planks have gone ([`close`]); everyone goes back to the
    /// town next tick ([`end`]).
    closing: bool,
}

/// The order a game's seats are taken in as people join (since 2026-11-12,
/// later that day; Hajun: "i want to change it so that this game is always
/// 2v2v2v2, with bots filling the remaining slots, and the 2 player become
/// same team. and if its 4 players, then those 2 players and 2 players are
/// same team too, and so on, so that real players will be teamed up with
/// each other first, and bots on remaining teams"): both seats of the
/// first island, both of the one across the map from it, and then both of
/// each of the two at its sides. So two people are a team, four are two
/// teams on opposite islands, and whoever is the odd one has the computer
/// for a teammate; pairs go by the order people join in. A seat is an
/// island and a team by its number ([`arena_team`]), so the seats between,
/// taken by nobody yet, are kept empty while the room fills
/// ([`Game::vacant`]), and the computer takes every one of them as the
/// wait ends. For half a day before, the order was an island each, 0, 2,
/// 1, 3, 4, 6, 5, 7, and two to four people began with no bots at all, a
/// 1v1 for two.
///
/// **This is the order of a game whose people begin on the first team,
/// red, and since 2026-11-15 a game begins them on any of the four**
/// ([`seat_order`], [`Game::order`]): until then whoever came first was
/// red in every game, on the same island every time.
const SEAT_ORDER: [usize; ARENA_SEATS] = [0, 4, 2, 6, 1, 5, 3, 7];

/// The order a game's seats are taken in whose people begin on the team
/// `first`: as [`SEAT_ORDER`] has it, both seats of that team's island,
/// both of the one across the map from it, and then both of each of the
/// two at its sides, the one on its left first or, `flip`, the one on its
/// right. Thrown for each game as it is opened ([`Arenas::filling`];
/// since 2026-11-15, Hajun: "right now, it seems like the real player's
/// team is fixed. but i want it so each game u are a random color, and
/// the color is fixed to which island, but the real players' team is not
/// fixed to red - one game they are blue team together one game they are
/// yellow team together etc, so they sometime play in the east island,
/// some time they play in the west island, etc."). An island is still its
/// team's, and a seat still an island and a team by its number
/// ([`arena_team`]): what is thrown is only which seats people are given.
/// Everything the order was for holds for each of them: two people are a
/// team, four are two teams on opposite islands, and a group goes in two
/// by two ([`Game::seats_for`]).
fn seat_order(first: usize, flip: bool) -> [usize; ARENA_SEATS] {
    let side = if flip { 3 } else { 1 };
    let teams = [first, first + 2, first + side, first + side + 2];
    let mut order = [0; ARENA_SEATS];
    for (pair, team) in order.chunks_mut(2).zip(teams) {
        let team = team % ARENA_TEAMS;
        pair[0] = team;
        pair[1] = team + ARENA_TEAMS;
    }
    order
}

/// The order of a game being opened, begun on any of the four teams as
/// likely as on another.
fn thrown_order() -> [usize; ARENA_SEATS] {
    seat_order(fastrand::usize(..ARENA_TEAMS), fastrand::bool())
}

/// Something a step of the fight has done ([`Game::step`], and what
/// players ask, [`hear`]) that takes the world to carry out: an entity for
/// the room to be sent, or no longer, or a word to someone's device. The
/// game only says so; [`flush`] does it, after.
#[derive(Clone, Debug)]
enum Event {
    /// A generator has made this: the room is sent it ([`Loot`]).
    Made(Piece),
    /// Someone has taken the thing numbered so: the room is sent it no
    /// longer.
    Taken(u32),
    /// A fireball, numbered so, has been thrown: the room is sent it.
    Thrown { ball: u32, shown: Fireball },
    /// The fireball numbered so has burst: the room is sent it no longer,
    /// which is how every device knows to draw the blast.
    Burst(u32),
    /// A stick of dynamite, numbered so, has been thrown: the room is
    /// sent it.
    Lobbed { stick: u32, shown: Dynamite },
    /// The stick numbered so has blown up: the room is sent it no longer,
    /// and is sent where it blew and which planks went with it, for good.
    Blew { stick: u32, shown: Blasted },
    /// The stick numbered so came down where there was nothing to lie
    /// on, and is gone into the void: the room is sent it no longer.
    Fell(u32),
    /// The person in `seat` has been knocked, by whoever has the id `by`
    /// from `from`, who was `running` if so: their device is told, and
    /// knocks its own body.
    Knocked { seat: usize, by: u64, from: Spot, running: bool },
    /// The person in this seat is sent to where they arrive on their
    /// island: as the fight begins, and each time they stand again.
    Land(usize),
    /// A zombie has risen in this seat: a body for it, for the room.
    Zombie(usize),
    /// One of the computer's players has laid this rung: the room is sent
    /// it, as it is a person's ([`Ask::Rung`]).
    Rung(Plank),
    /// Something for the chat ([`Feed`]): everyone in the game is told, or
    /// the one team it is for.
    Feed(Feed),
}

/// The turn about the upright a body facing as `turn` has, in radians:
/// what [`Game::yaw`] keeps.
fn yaw_of(turn: Quat) -> f32 {
    turn.to_euler(EulerRot::YXZ).0
}

/// Who is in a seat nobody is in: nobody, with no name and no look. A
/// device takes a member with no id for an empty seat.
fn nobody() -> Member {
    Member {
        id: 0,
        name: String::new(),
        look: String::new(),
        ai: true,
    }
}

/// A stick of dynamite thrown in a game: its number, how far through its
/// throw and its fuse it is ([`Lob`]), who threw it, by seat, and whether
/// it has come down and found ground to lie on.
#[derive(Clone, Copy, Debug)]
struct Lobbed {
    id: u32,
    lob: Lob,
    by: usize,
    lies: bool,
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
    /// A game numbered `id`, its waiting room empty, whose room is sent
    /// the entity `shown`; its dice thrown from `seed`. Its people begin
    /// on the first team until whoever opens it throws for that
    /// ([`Arenas::filling`]): not by `seed`, so that a game played out
    /// from a seed by the computer alone, as the tests play them, is the
    /// game that seed always was.
    fn new(id: u32, shown: Entity, seed: u64) -> Self {
        let mut dice = fastrand::Rng::with_seed(seed);
        Self {
            id,
            shown,
            order: SEAT_ORDER,
            seats: Vec::new(),
            members: Vec::new(),
            bot: Vec::new(),
            minds: Vec::new(),
            gone: Vec::new(),
            vacant: Vec::new(),
            stage: ArenaStage::Waiting,
            left: 0.0,
            since_join: 0.0,
            coming: None,
            hurried: false,
            mingle: Mingle::new(seed),
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
            elapsed: 0.0,
            clock: 0.0,
            mint: Mint::new(0.0, &mut || dice.f32()),
            loot: HashMap::new(),
            purses: Vec::new(),
            outfits: Outfits::default(),
            balls: Vec::new(),
            next_ball: 0,
            shown_balls: HashMap::new(),
            sticks: Vec::new(),
            next_stick: 0,
            shown_sticks: HashMap::new(),
            blasts: 0,
            planks_blown: 0,
            spilt: [0; 4],
            at: Vec::new(),
            yaw: Vec::new(),
            present: Vec::new(),
            away: Vec::new(),
            died_away: Vec::new(),
            metres: Vec::new(),
            flat: Vec::new(),
            flat_until: Vec::new(),
            bus_clock: None,
            squashes: [0; 3],
            buses: None,
            bus_log: BusLog::default(),
            aboard: Vec::new(),
            events: Vec::new(),
            nav: Nav::arena(),
            horde: Horde::new(seed ^ 0x5a5a_5a5a),
            squad: Squad::new(seed, Skill::MODERATE),
            old_bots: false,
            dice,
            last_hit: Vec::new(),
            builders: HashMap::new(),
            bot_builders: [None; ARENA_SEATS],
            shown_bot_builders: HashMap::new(),
            sent: None,
            sent_seats: Vec::new(),
            sent_purses: Vec::new(),
            sent_kits: Vec::new(),
            sent_upgrades: Default::default(),
            sent_flat: Flattened::default(),
            sent_buses: None,
            closing: false,
        }
    }

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

    /// Whether whoever is in `seat` is one of the computer's players that
    /// plays the game ([`Squad`]): in a player's seat, and not, by the
    /// switch, as they were before.
    fn brained(&self, seat: usize) -> bool {
        !self.old_bots && seat < ARENA_SEATS && self.bot[seat]
    }

    /// Which zombie `seat` is, if it is one's: the seats past the players'.
    fn zombie(&self, seat: usize) -> Option<usize> {
        seat.checked_sub(ARENA_SEATS)
    }

    /// The ground the computer's player in `seat` keeps to, as the middle
    /// of a disc and how far it reaches: its island, or a zombie's home.
    fn home(&self, seat: usize) -> (Vec2, f32) {
        match self.zombie(seat) {
            Some(zombie) => zombie_home(zombie),
            None => (ARENA_ISLANDS[arena_island(seat)], ARENA_ISLAND_RADIUS),
        }
    }

    /// Where the computer's player in `seat` stands as it comes into the
    /// fight, and again each time it stands again, and which way it faces.
    fn bot_spot(&self, seat: usize) -> Spot {
        match self.zombie(seat) {
            Some(zombie) => {
                let body = Horde::spot(&self.nav, zombie);
                Spot::of(body.at, Quat::from_rotation_y(body.yaw))
            }
            None => {
                let (at, turn) = arrival(Venue::Arena, seat);
                Spot::of(at.with_y(ISLAND_TOP), turn)
            }
        }
    }

    /// Who the computer's player in `seat` is: a boy or a girl by the toss
    /// of a coin, in any of that side's colours, under one of that side's
    /// names that nobody in the game has.
    fn bot_member(&self, seat: usize) -> Member {
        let boy = fastrand::bool();
        let names: Vec<&str> = roundtown_net::ai_names(boy)
            .iter()
            .copied()
            .filter(|name| self.members.iter().all(|member| !member.name.starts_with(name)))
            .collect();
        Member {
            id: bot_id(self.id, seat),
            name: fastrand::choice(names).unwrap_or("Bot").into(),
            look: roundtown_net::ai_look(boy, fastrand::usize(..)).into(),
            ai: true,
        }
    }

    /// Whoever in `seat` has their feet at `at` picks up everything a
    /// generator has made that is within reach of them, dropping or lying
    /// (`Mint::pick_up`), if they are a person and standing: it is
    /// theirs, and gone from the pads. What they took, for the room to
    /// stop being sent it.
    fn gather(&mut self, seat: usize, at: Vec3) -> Vec<Piece> {
        if self.bot[seat] || !self.alive(seat) {
            return Vec::new();
        }
        let taken = self.mint.pick_up(at, self.elapsed);
        for piece in &taken {
            let held = &mut self.purses[seat][piece.item.index()];
            *held = held.saturating_add(piece.worth);
            if piece.dropped() {
                self.spilt[2] += u32::from(piece.worth);
            }
        }
        taken
    }

    /// Whoever was in `victim` has been killed where they stand, by a
    /// hand, a sword or a blast: what they held of sapphire and amethyst
    /// is thrown out round them, to lie there for whoever walks into it
    /// (`Mint::spill`, since the night of 2026-11-15; Hajun: "if its a
    /// land-hp kill where they die on land, they should drop the
    /// sapphires and amethyst like small scatter explosion from their
    /// dead body location like mineraft dropping items on death, and u
    /// only get it if u pick it up. but items shouldnt scatter to location
    /// where they would fall off the map"). Onto ground a body can walk
    /// that is staying: land, or a plank of a bridge no blast has doomed.
    /// Nobody has any of it until it has come down and lain a moment
    /// (`SPILL_WAIT`; Hajun: "make sure that there is a very small delay
    /// before u can pick it up").
    /// The room is sent each thing as it is sent what a generator makes
    /// ([`Event::Made`]). Before the victim dies and loses the rest
    /// ([`Game::die`]).
    ///
    /// Whether it was: not with nowhere for any of it to lie, the victim
    /// out over the void or on a bridge that is coming down, which is no
    /// death on land, and what they held goes as it does with a fall
    /// ([`Game::hand_spoils`]). Nor of a zombie, which holds nothing.
    /// Mine: that it is so whoever killed them, a zombie or their own
    /// team's blast, and that anyone may pick it up, their own team and
    /// they themselves among them, as in Minecraft.
    fn drop_spoils(&mut self, victim: usize) -> bool {
        if victim >= ARENA_SEATS {
            return false;
        }
        let (nav, dice, from) = (&self.nav, &mut self.dice, self.at[victim]);
        let Some(held) = self.purses.get_mut(victim) else {
            return false;
        };
        let ground = |at: Vec2| nav.stays_at(at).then(|| nav.ground(at)).flatten();
        let Some(dropped) = self.mint.spill(held, from, self.elapsed, &mut || dice.f32(), &ground) else {
            return false;
        };
        if !dropped.is_empty() {
            info!("Bridge Wars game {}: seat {victim} dropped {} things at {:.1}", self.id, dropped.len(), from.xz());
            self.spilt[0] += 1;
            self.spilt[1] += dropped.iter().map(|piece| u32::from(piece.worth)).sum::<u32>();
        }
        self.events.extend(dropped.into_iter().map(Event::Made));
        true
    }

    /// What was dropped and lay on planks that have gone goes with them
    /// (`Mint::shed`): the room is no longer sent it.
    fn shed(&mut self) {
        let nav = &self.nav;
        let gone = self.mint.shed(&|at| nav.ground(at).is_some());
        self.spilt[3] += gone.iter().map(|piece| u32::from(piece.worth)).sum::<u32>();
        self.events.extend(gone.into_iter().map(|piece| Event::Taken(piece.id)));
    }

    /// Sells `ware` to whoever in `seat` stands at `at`, if the fight is
    /// on, they are standing, near the shop that sells it, it is to be had
    /// by them, and they hold its price, which comes out of their purse
    /// ([`Outfits::buy`]): whether it is theirs.
    fn buy(&mut self, seat: usize, ware: Ware, at: Vec2) -> bool {
        if !self.playing() || !self.alive(seat) || !at_a_shop(at, sold_at(ware)) {
            return false;
        }
        let bought = self.outfits.buy(seat, ware, &mut self.purses[seat]);
        // What is the whole team's, the team is told of ("Bob purchased
        // Heal Pool for your team."): how far it has it now, for what goes
        // by steps.
        if bought && sold_at(ware) == Shop::Upgrades {
            let team = self.outfits.team(seat);
            let step = match ware {
                Ware::Forge => team.forge,
                Ware::BetterShields => team.shields,
                Ware::Trap => team.traps,
                Ware::BuildSpeed => team.build_speed,
                _ => 1,
            };
            self.say(Feed::Bought {
                by: seat as u8,
                ware,
                step,
            });
        }
        bought
    }

    /// Says `feed` in the game's chat ([`Event::Feed`]): to everyone in
    /// it, or to the one team it is for. A zombie's death is nothing to
    /// say: there are twelve of them, each back a moment after it is put
    /// down (mine: Hajun's "when anyone dies" was said of players).
    fn say(&mut self, feed: Feed) {
        let died = match feed {
            Feed::Fell { seat } | Feed::Knocked { seat, .. } | Feed::Killed { seat, .. } | Feed::RunOver { seat } => {
                Some(usize::from(seat))
            }
            Feed::Bought { .. } | Feed::AnchorBroken { .. } => None,
        };
        if died.is_some_and(|seat| seat >= ARENA_SEATS) {
            return;
        }
        self.events.push(Event::Feed(feed));
    }

    /// What whoever is in `seat` hits with: the sword in their hand, by
    /// its tier, or a bare hand.
    fn weapon_of(&self, seat: usize) -> Weapon {
        match self.outfits.kit(seat).sword {
            0 => Weapon::Fists,
            tier => Weapon::Sword(tier),
        }
    }

    /// `seat` has fallen into the void: dead ([`Game::die`]), by the
    /// doing of whoever last hit them, if that was no longer ago than
    /// [`CREDIT_SECS`] ("Bob knocked Hannah off the world."), and by
    /// nobody's otherwise ("Bob fell off the world.").
    fn fall(&mut self, seat: usize) {
        info!("Bridge Wars game {}: seat {seat} fell", self.id);
        let by = self.last_hit[seat]
            .filter(|&(hitter, when, _)| hitter != seat && self.elapsed - when <= CREDIT_SECS)
            .map(|(hitter, ..)| hitter);
        self.say(match by {
            Some(by) => Feed::Knocked {
                seat: seat as u8,
                by: by as u8,
            },
            None => Feed::Fell { seat: seat as u8 },
        });
        if let Some(by) = by {
            self.hand_spoils(seat, by);
        }
        self.die(seat);
    }

    /// Whoever is in `killer` has killed whoever is in `victim`: what the
    /// victim held of sapphire and amethyst is the killer's, at once, if
    /// they are players of two teams ([`takes_spoils`], [`spoils`]; since
    /// the night of 2026-11-14, Hajun: "if you kill an enemy in
    /// bridgewars, you should automatically get their sapphire and
    /// amethyst (not gold or iron) if they had any - not drop but just
    /// auto get it in ur inven"). Before the victim dies and loses the
    /// rest ([`Game::die`]). The room is sent both purses as it is sent
    /// any, so a device from before this shows it too.
    ///
    /// Since the night of 2026-11-15 only where the victim was knocked
    /// off the world, or died with no land under them: dead on land, they
    /// drop it ([`Game::drop_spoils`]).
    fn hand_spoils(&mut self, victim: usize, killer: usize) {
        if !takes_spoils(killer, victim) {
            return;
        }
        let Some(held) = self.purses.get_mut(victim) else {
            return;
        };
        let won = spoils(held);
        if won == Purse::default() {
            return;
        }
        if let Some(purse) = self.purses.get_mut(killer) {
            gain(purse, won);
        }
        info!("Bridge Wars game {}: seat {killer} took {won:?} off seat {victim}", self.id);
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
            // The players' seats alone: a zombie is never out.
            out: self
                .out
                .iter()
                .enumerate()
                .take(ARENA_SEATS)
                .filter(|&(_, &out)| out)
                .fold(0u8, |bits, (seat, _)| bits | (1 << seat)),
        }
    }

    /// The next of the players' seats to be taken: the first, in the order
    /// this game's are taken in ([`Game::order`]), that nobody has sat in.
    fn free_seat(&self) -> Option<usize> {
        self.order
            .into_iter()
            .find(|&seat| seat >= self.seats.len() || self.vacant[seat])
    }

    /// The seat the computer's next player takes, coming into the waiting
    /// room one at a time ([`Game::bots_due`]): the last, in the order this
    /// game's seats are taken in, that nobody has sat in. So the seats
    /// people are given stay free the longest, and whoever comes while the
    /// computer's players are still coming is teamed up with the people
    /// there, as they would have been a moment sooner.
    fn last_free_seat(&self) -> Option<usize> {
        self.order
            .into_iter()
            .rev()
            .find(|&seat| seat >= self.seats.len() || self.vacant[seat])
    }

    /// How many of the computer's players come into the waiting room this
    /// frame, `dt` seconds on: none until nobody has joined for
    /// [`BOT_WAIT`] seconds; then one, and one more each time the gap
    /// thrown after the last has passed ([`BOT_GAP`]; since 2026-11-16,
    /// Hajun: "make them join slightly delay one by one rather than all at
    /// once"), for as long as a seat is free, whoever joins or leaves
    /// meanwhile; or, the wait skipped, every one of them at once. Until
    /// then the computer took every seat left in the one frame.
    fn bots_due(&mut self, dt: f32) -> usize {
        let free = (0..ARENA_SEATS)
            .filter(|&seat| seat >= self.seats.len() || self.vacant[seat])
            .count();
        if free == 0 || self.stage != ArenaStage::Waiting {
            return 0;
        }
        if self.hurried {
            return free;
        }
        if self.coming.is_none() && self.since_join >= BOT_WAIT {
            self.coming = Some(0.0);
        }
        let Some(left) = self.coming.as_mut() else {
            return 0;
        };
        *left -= dt;
        if *left > 0.0 {
            return 0;
        }
        *left = BOT_GAP.0 + (BOT_GAP.1 - BOT_GAP.0) * fastrand::f32();
        1
    }

    /// Where everyone in the waiting room stands, across its floor from
    /// its middle, by seat, with nobody in `but` nor in a seat nobody is
    /// in: what the computer's players there go by ([`Mingle`]).
    fn in_the_room(&self, but: Option<usize>) -> Vec<Option<Vec2>> {
        (0..self.seats.len().min(ARENA_SEATS))
            .map(|seat| (Some(seat) != but && !self.vacant[seat] && !self.gone[seat] && self.present[seat]).then(|| self.at[seat].xz()))
            .collect()
    }

    /// A frame of the computer's players in the waiting room, `dt` seconds
    /// on, before the fight: each goes about as who it is ([`Mingle`]), and
    /// is where that has it, for its body to be sent as ([`flush`]).
    /// `soon`, the room is counting down.
    fn mill(&mut self, dt: f32, soon: bool) {
        let mut everyone = self.in_the_room(None);
        for seat in 0..self.seats.len().min(ARENA_SEATS) {
            if !self.bot[seat] {
                continue;
            }
            let Some(stood) = self.mingle.step(seat, &everyone, soon, dt) else {
                continue;
            };
            self.at[seat] = stood.in_room(Vec2::ZERO, WAITING_FLOOR);
            self.yaw[seat] = stood.yaw;
            everyone[seat] = Some(stood.at);
        }
    }

    /// The seats `count` people take who go in together, in the order they
    /// take them (since 2026-11-13, a group of friends going in with its
    /// leader, `social`; Hajun: "everyone get queued at the same time in
    /// the same team (unless group is >2 and ur playing 2v2v2v2 bridge
    /// wars, then it will need to be 2 ppl same team 1 ppl other team i
    /// guess)"): two by two onto teams with both seats free, in the order
    /// the teams fill in ([`Game::order`], which has each team's two seats
    /// together), and an odd one last, wherever the next to come alone
    /// would sit. One alone takes [`Game::free_seat`]. None if the room is
    /// no longer waiting, or has not the teams free.
    fn seats_for(&self, count: usize) -> Option<Vec<usize>> {
        if self.stage != ArenaStage::Waiting {
            return None;
        }
        let free = |seat: usize| seat >= self.seats.len() || self.vacant[seat];
        let mut seats = Vec::with_capacity(count);
        let mut teams = self.order.chunks(2);
        for _ in 0..count / 2 {
            let pair = teams.by_ref().find(|pair| pair.iter().all(|&seat| free(seat)))?;
            seats.extend_from_slice(pair);
        }
        if count % 2 == 1 {
            seats.push(self.order.into_iter().find(|&seat| free(seat) && !seats.contains(&seat))?);
        }
        Some(seats)
    }

    /// Whether anyone of the people in it but whoever is in `seat` is
    /// connected to it: in its room, with a device ([`sync`]).
    fn watched_but(&self, seat: usize) -> bool {
        (0..self.seats.len().min(ARENA_SEATS)).any(|other| {
            other != seat
                && !self.bot[other]
                && !self.vacant[other]
                && !self.gone[other]
                && self.present[other]
                && !self.away[other]
        })
    }

    /// How many people are in it: not the computer's players, nor whoever
    /// has left.
    fn people(&self) -> usize {
        (0..self.seats.len())
            .filter(|&seat| !self.bot[seat] && !self.vacant[seat] && !self.gone[seat])
            .count()
    }

    /// Seats `member`, whose body is `body`, in the next of the players'
    /// seats ([`Game::free_seat`]); or, with those all taken or the fight
    /// begun, in the next seat past them, as a zombie is. Only the tests
    /// seat anyone so since 2026-11-13: people are seated together, each
    /// where [`Game::seats_for`] says ([`seat_together`]).
    #[cfg(test)]
    fn seat(&mut self, body: Entity, member: Member, bot: bool) -> usize {
        let seat = match self.free_seat() {
            Some(seat) if self.stage == ArenaStage::Waiting => seat,
            _ => self.seats.len().max(ARENA_SEATS),
        };
        self.put(seat, body, member, bot)
    }

    /// Seats `member`, whose body is `body`, in `seat`, with every seat
    /// before it that nobody is in yet kept empty.
    fn put(&mut self, seat: usize, body: Entity, member: Member, bot: bool) -> usize {
        while self.seats.len() <= seat {
            self.push_vacant();
        }
        self.seats[seat] = body;
        self.members[seat] = member;
        self.bot[seat] = bot;
        self.minds[seat] = Mind {
            think: THINK_EVERY * seat as f32 / ARENA_SEATS as f32,
            ..default()
        };
        self.gone[seat] = false;
        self.vacant[seat] = false;
        self.health[seat] = HEALTH;
        self.out[seat] = false;
        self.last_hit[seat] = None;
        self.since_join = 0.0;
        // In the waiting room, until the game says otherwise: where a
        // person is put as they are seated, and where the computer's
        // comes in. Somewhere about its middle, thrown, clear of whoever
        // is there, and by nothing of the seat (since 2026-11-16; Hajun:
        // "your team selection should not be shown/determine until the
        // game start"): until then each stood at their seat's own place
        // on a ring ([`waiting_spot`]), and a seat is a team. Whoever is
        // seated with the fight on, a zombie, is stood by whoever seats
        // it.
        let waits = seat < ARENA_SEATS && matches!(self.stage, ArenaStage::Waiting | ArenaStage::Countdown);
        let stood = if waits {
            let everyone = self.in_the_room(Some(seat));
            self.mingle.arrive(&everyone)
        } else {
            let (at, turn) = waiting_spot(seat);
            Stood {
                at: at.xz(),
                up: 0.0,
                yaw: yaw_of(turn),
            }
        };
        let at = stood.in_room(Vec2::ZERO, WAITING_FLOOR);
        self.at[seat] = at;
        self.yaw[seat] = stood.yaw;
        self.present[seat] = true;
        self.away[seat] = false;
        self.died_away[seat] = false;
        self.metres[seat] = 0.0;
        self.flat[seat] = 0.0;
        self.mingle.leave(seat);
        if bot && seat < ARENA_SEATS {
            self.squad.seat(seat, at, stood.yaw);
            // And there it passes the time, as who it is.
            if waits && let Some(persona) = self.squad.bot(seat).map(|bot| bot.persona()) {
                self.mingle.enter(seat, persona, stood);
            }
        } else {
            self.squad.unseat(seat);
        }
        seat
    }

    /// Empties `seat`: nobody in it, gone and out, with no body, as a seat
    /// nobody has taken yet is ([`Game::push_vacant`]), for the next to
    /// come, a person or the computer, to take.
    fn vacate(&mut self, seat: usize) {
        self.seats[seat] = Entity::PLACEHOLDER;
        self.members[seat] = nobody();
        self.bot[seat] = false;
        self.minds[seat] = Mind::default();
        self.gone[seat] = true;
        self.vacant[seat] = true;
        self.health[seat] = 0.0;
        self.since_hit[seat] = 0.0;
        self.hurt[seat] = 0.0;
        self.dead_for[seat] = None;
        self.out[seat] = true;
        self.last_hit[seat] = None;
        self.rungs[seat] = 0;
        self.purses[seat] = Purse::default();
        self.present[seat] = false;
        self.away[seat] = false;
        self.died_away[seat] = false;
        self.metres[seat] = 0.0;
        self.flat[seat] = 0.0;
        self.squad.unseat(seat);
        self.mingle.leave(seat);
        if let Some(kit) = self.outfits.kits.get_mut(seat) {
            *kit = Kit::default();
        }
    }

    /// Whoever was in `seat` has left the game for good: gone back to the
    /// town; or come back to the server from outside the game with nobody
    /// else of its people connected to it ([`Arenas::returns`]); or, until
    /// 2026-11-15, their connection had gone, which a seat now waits out
    /// ([`Arenas::keeps`]). Before the fight begins, in the waiting room
    /// or through the countdown, their seat is anyone's again
    /// ([`Game::vacate`]), a person's who comes or the computer's, so
    /// that no team begins short (until 2026-11-12 it stayed theirs,
    /// empty, a hole in a team for the whole game); once it has begun, it
    /// stays theirs and empty.
    fn leave(&mut self, seat: usize) {
        if seat < ARENA_SEATS && matches!(self.stage, ArenaStage::Waiting | ArenaStage::Countdown) {
            info!("Bridge Wars game {}: seat {seat} left before the fight, and is free again", self.id);
            self.vacate(seat);
        } else {
            self.gone[seat] = true;
        }
    }

    /// One more seat with nobody in it: gone and out, with no body.
    fn push_vacant(&mut self) {
        self.seats.push(Entity::PLACEHOLDER);
        self.members.push(nobody());
        self.bot.push(false);
        self.minds.push(Mind::default());
        self.gone.push(true);
        self.vacant.push(true);
        self.health.push(0.0);
        self.since_hit.push(0.0);
        self.hurt.push(0.0);
        self.dead_for.push(None);
        self.out.push(true);
        self.last_hit.push(None);
        self.rungs.push(0);
        self.purses.push(Purse::default());
        self.at.push(Vec3::ZERO);
        self.yaw.push(0.0);
        self.present.push(false);
        self.away.push(false);
        self.aboard.push(false);
        self.died_away.push(false);
        self.metres.push(0.0);
        self.flat.push(0.0);
        self.flat_until.push(0.0);
        self.outfits.seat(self.seats.len());
    }

    /// Makes the players' seats eight, with nobody in whichever nobody
    /// has taken: a zombie's seat is past the players' eight, whoever is
    /// in them ([`arena_team`]). The computer has taken every seat left
    /// by the time the fight begins, so this only keeps the count.
    fn pad(&mut self) {
        while self.seats.len() < ARENA_SEATS {
            self.push_vacant();
        }
    }

    /// `seat` is dead: for [`RESPAWN_SECS`], and then back, if its team's
    /// anchor stands now; otherwise for good. A zombie is always back, a
    /// while later ([`ZOMBIE_RESPAWN_SECS`]).
    fn die(&mut self, seat: usize) {
        self.health[seat] = 0.0;
        self.last_hit[seat] = None;
        self.minds[seat] = Mind::default();
        // Dead where a bus had flattened them, they lie there no longer.
        self.flat[seat] = 0.0;
        // The metres of bridge one of the computer's players had go with
        // the rest, and the bridge it was laying is anyone's to finish.
        self.squad.died(seat, self.elapsed);
        // Their sword and what was on their second button go with them,
        // and everything they held of what the generators make (since
        // 2026-11-12; Hajun: "death make u lose all ur silver, gold,
        // sapphire, amethyst, bridge meters"; the metres of bridge are
        // their own device's to take). The room is sent the empty purse
        // with the rest (`Purses`).
        self.outfits.die(seat);
        if let Some(purse) = self.purses.get_mut(seat) {
            *purse = Purse::default();
        }
        // The metres of bridge they had left go too, as their device has
        // it; and one whose device is not there to see them die is told
        // of it when it is back ([`Arenas::returns`]).
        self.metres[seat] = 0.0;
        if self.away[seat] {
            self.died_away[seat] = true;
        }
        if self.zombie(seat).is_some() {
            self.dead_for[seat] = Some(ZOMBIE_RESPAWN_SECS);
            return;
        }
        if self.anchors[usize::from(arena_team(seat))] > 0.0 {
            self.dead_for[seat] = Some(RESPAWN_SECS);
        } else {
            self.dead_for[seat] = None;
            self.out[seat] = true;
            info!("Bridge Wars game {}: seat {seat} is out", self.id);
        }
    }

    /// Takes a punch's damage off `victim`, hit from `from` by `hitter`, who
    /// was `running` if so: more with a sword in the hitter's hand, less
    /// through the victim's shield ([`Outfits::punch`]). Not while the
    /// victim is still reeling from the last punch that took
    /// ([`HURT_TIME`]): a device lands a punch at every press, several a
    /// second. A hitter who was unseen is seen again, whether or not the
    /// punch took ("being hit or attacking reveals u").
    fn wound(&mut self, victim: usize, hitter: usize, from: Spot, running: bool) {
        self.outfits.reveal(hitter);
        if self.hurt[victim] > 0.0 {
            return;
        }
        self.hurt[victim] = HURT_TIME;
        // A zombie's blow is a heavier one than a bare fist's.
        let bare = if self.zombie(hitter).is_some() { ZOMBIE_DAMAGE } else { DAMAGE };
        let damage = self.outfits.punch(bare, hitter, victim);
        let with = self.weapon_of(hitter);
        self.harm(victim, hitter, from, running, damage, with);
    }

    /// Takes `damage` off `victim`, by `hitter`'s doing, `with` that, and
    /// no more than that: dead at none, which the game's chat says
    /// ([`Feed::Killed`]). Whoever was unseen is seen again for being
    /// hit. Whether they are still standing. What a stick of dynamite's
    /// blast does to whoever it reaches: it knocks nobody.
    fn hurt_by(&mut self, victim: usize, hitter: usize, damage: f32, with: Weapon) -> bool {
        // Nothing hurts a zombie (since the night of 2026-11-16; Hajun:
        // "zombie should be invincible"): it stands, whatever hit it.
        if self.zombie(victim).is_some() {
            return true;
        }
        if self.flat[victim] > 0.0 {
            self.squashes[2] += 1;
        }
        self.outfits.reveal(victim);
        self.health[victim] = (self.health[victim] - damage).max(0.0);
        self.since_hit[victim] = 0.0;
        self.last_hit[victim] = Some((hitter, self.elapsed, with));
        if self.health[victim] <= 0.0 {
            info!("Bridge Wars game {}: seat {hitter} killed seat {victim}", self.id);
            self.say(Feed::Killed {
                seat: victim as u8,
                by: hitter as u8,
                with,
            });
            // Dead on land, what they held of sapphire and amethyst lies
            // round them; with no land for it to lie on, it is whoever
            // killed them's, as when they are knocked off the world.
            if !self.drop_spoils(victim) {
                self.hand_spoils(victim, hitter);
            }
            self.die(victim);
            return false;
        }
        debug!("Bridge Wars game {}: seat {hitter} hit seat {victim}, left with {}", self.id, self.health[victim]);
        true
    }

    /// Takes `damage` off `victim`, struck from `from` by `hitter`, `with`
    /// that: dead at none, and knocked back from there otherwise, further
    /// if `running`, a person through their device ([`Event::Knocked`])
    /// and a bot here. Whoever was unseen is seen again for being hit.
    fn harm(&mut self, victim: usize, hitter: usize, from: Spot, running: bool, damage: f32, with: Weapon) {
        if !self.hurt_by(victim, hitter, damage, with) {
            return;
        }
        // Whoever a bus has flattened is hurt where they lie, and knocked
        // nowhere (mine: "they cannot move for 10second").
        if self.flat[victim] > 0.0 {
            return;
        }
        if self.brained(victim) {
            self.squad.knock(victim, from.translation().xz(), running);
            return;
        }
        // A zombie takes a share of the knock: it keeps coming.
        if let Some(zombie) = self.zombie(victim) {
            self.horde.knock(zombie, from.translation().xz(), running);
            return;
        }
        if self.bot[victim] {
            let away = (self.at[victim].xz() - from.translation().xz()).normalize_or(Vec2::NEG_Y);
            // A zombie takes a share of the knock: it keeps coming.
            let taken = if self.zombie(victim).is_some() { ZOMBIE_KNOCK } else { 1.0 };
            let push = (KNOCK_BACK + if running { SPRINT_BONUS } else { 0.0 }) * taken;
            let up = KNOCK_UP * taken;
            let mind = &mut self.minds[victim];
            mind.shove = mind.shove * 0.5 + away * push;
            mind.lift = (mind.lift * 0.5 + up).min(up);
            return;
        }
        self.events.push(Event::Knocked {
            seat: victim,
            by: self.members[hitter].id,
            from,
            running,
        });
    }

    /// Which teams still have anyone in play: the players' teams, the
    /// zombies being nobody's.
    fn teams_in_play(&self) -> Vec<u8> {
        let mut teams: Vec<u8> = (0..self.seats.len().min(ARENA_SEATS))
            .filter(|&seat| self.in_play(seat))
            .map(arena_team)
            .collect();
        teams.sort_unstable();
        teams.dedup();
        teams
    }

    /// Puts `seat` where it arrives on its island, as the fight begins and
    /// each time it stands again: the computer's player there at once, a
    /// person through their device ([`Event::Land`]).
    fn stand(&mut self, seat: usize) {
        if self.vacant[seat] {
            return;
        }
        // A zombie, where it first came up: as it rises, and again out of
        // the void.
        if let Some(zombie) = self.zombie(seat) {
            self.horde.stand(&self.nav, zombie);
            if let Some(body) = self.horde.body(zombie) {
                self.at[seat] = body.at;
                self.yaw[seat] = body.yaw;
            }
            self.minds[seat] = Mind::default();
            return;
        }
        if self.bot[seat] {
            let spot = self.bot_spot(seat);
            self.at[seat] = spot.translation();
            self.yaw[seat] = yaw_of(spot.rotation());
            self.minds[seat] = Mind {
                think: THINK_EVERY * seat as f32 / ARENA_SEATS as f32,
                ..default()
            };
            self.squad.stand(seat, self.at[seat], self.yaw[seat], self.elapsed);
            return;
        }
        // Where the game has them from now, though their body is put
        // there only as the frame is carried out ([`flush`]): until
        // 2026-11-15 they were where they had died for the rest of the
        // frame, and someone whose device had gone, fallen into the void,
        // was found over the void again the moment they stood, and fell
        // a second time ([`Game::fight`]).
        let (at, turn) = arrival(Venue::Arena, seat);
        self.at[seat] = at;
        self.yaw[seat] = yaw_of(turn);
        self.events.push(Event::Land(seat));
    }

    /// Something on the timetable happens ([`TIMETABLE`]): a tier is the
    /// mint's own, by the time; every anchor left breaks; or the zombies
    /// come.
    fn happen(&mut self, event: Happening) {
        info!("Bridge Wars game {}: {event:?}, {:.0} s in", self.id, self.elapsed);
        match event {
            Happening::Tier { .. } => {}
            Happening::AnchorsBreak => self.anchors = [0.0; ARENA_TEAMS],
            // From now on they come, a few and then more
            // ([`Game::raise_zombies`]); and the end is near, which every
            // device says of itself.
            Happening::SuddenDeath | Happening::DrawSoon => {}
        }
    }

    /// As many zombies as there are to be by now ([`zombies_at`]): each
    /// that is due rises, one after another.
    fn raise_zombies(&mut self) {
        while self.horde.len() < zombies_at(self.elapsed) {
            self.seat_zombie();
        }
    }

    /// The next zombie, in the next seat, past the players': standing
    /// where it comes up ([`zombie_spot`]), with a mind of its own
    /// ([`Horde`]). Its body is the world's to make ([`Event::Zombie`]).
    fn seat_zombie(&mut self) {
        let zombie = self.horde.rise(&self.nav);
        let seat = self.seats.len().max(ARENA_SEATS);
        debug_assert_eq!(seat, ARENA_SEATS + zombie, "the zombies come once the players' seats are full");
        let member = Member {
            id: bot_id(self.id, seat),
            name: ZOMBIE_NAME.into(),
            look: ZOMBIE_LOOK.into(),
            ai: true,
        };
        self.put(seat, Entity::PLACEHOLDER, member, true);
        self.stand(seat);
        self.events.push(Event::Zombie(seat));
    }

    /// Whoever in `seat`, standing at `at`, throws a fireball the way
    /// `heading` says, if there is one on their second button: it is in
    /// the air, the room is sent it ([`Event::Thrown`]), and the game
    /// flies it ([`Game::fireballs`]). Whether it was thrown.
    fn throw(&mut self, seat: usize, at: Vec3, heading: f32) -> bool {
        if !self.playing() || !self.alive(seat) || !heading.is_finite() || !self.outfits.spend(seat, Utility::Fireball) {
            return false;
        }
        let flight = Flight::thrown(at, heading);
        let ball = self.next_ball;
        self.next_ball = self.next_ball.wrapping_add(1);
        self.events.push(Event::Thrown {
            ball,
            shown: Fireball {
                seat: seat as u8,
                x: flight.from.x,
                y: flight.from.y,
                z: flight.from.z,
                heading,
            },
        });
        self.balls.push((ball, flight, seat));
        debug!("Bridge Wars game {}: seat {seat} threw a fireball", self.id);
        true
    }

    /// Whoever in `seat`, standing at `at`, throws a stick of dynamite the
    /// way `heading` says, to come down `far` from them, if there is one
    /// on their buttons: it is in the air, the room is sent it
    /// ([`Event::Lobbed`]), and the game lobs it on and blows it
    /// ([`Game::dynamites`]). It is thrown at the ground where it is to
    /// come down, as high as that is now, or, with none there, as high as
    /// the ground the thrower stands on. Whether it was thrown.
    fn lob(&mut self, seat: usize, at: Vec3, heading: f32, far: f32) -> bool {
        if !self.playing() || !self.alive(seat) || !heading.is_finite() || !far.is_finite() {
            return false;
        }
        if !self.outfits.spend(seat, Utility::Dynamite) {
            return false;
        }
        let aimed = Lob::thrown(at, heading, far, at.y);
        let floor = self.nav.ground(aimed.to.xz()).or(self.nav.ground(at.xz())).unwrap_or(at.y);
        let lob = Lob::thrown(at, heading, far, floor);
        let stick = self.next_stick;
        self.next_stick = self.next_stick.wrapping_add(1);
        self.events.push(Event::Lobbed {
            stick,
            shown: Dynamite {
                seat: seat as u8,
                x: lob.from.x,
                y: lob.from.y,
                z: lob.from.z,
                to_x: lob.to.x,
                to_y: lob.to.y,
                to_z: lob.to.z,
            },
        });
        self.sticks.push(Lobbed {
            id: stick,
            lob,
            by: seat,
            lies: false,
        });
        debug!("Bridge Wars game {}: seat {seat} threw a stick of dynamite {:.1} m", self.id, lob.chord());
        true
    }

    /// A punch by whoever is in `hitter`, standing at `from` across the
    /// ground, on the respawn anchor of `team`, if it is another team's,
    /// still stands and is within `reach` of them ([`anchor_in_reach`]):
    /// the punch's damage off it, more with a sword as off a body, and no
    /// oftener than a body takes one ([`HURT_TIME`]). Hitting an anchor is
    /// attacking: whoever was unseen is seen again, whether or not the hit
    /// takes. Whether it took.
    fn strike_anchor(&mut self, hitter: usize, team: usize, from: Vec2, reach: f32) -> bool {
        if !self.playing() || !self.alive(hitter) || team >= ARENA_TEAMS || team == usize::from(arena_team(hitter)) {
            return false;
        }
        if self.anchors[team] <= 0.0 || !anchor_in_reach(from, ARENA_ANCHORS[team], reach) {
            return false;
        }
        self.outfits.reveal(hitter);
        if self.anchor_hurt[team] > 0.0 {
            return false;
        }
        self.anchor_hurt[team] = HURT_TIME;
        // A sword breaks an anchor sooner, as it does a body.
        let damage = punch_damage(DAMAGE, &self.outfits.kit(hitter), &self.outfits.team(hitter));
        self.anchors[team] = (self.anchors[team] - damage).max(0.0);
        if self.anchors[team] <= 0.0 {
            info!("Bridge Wars game {}: seat {hitter} broke team {team}'s anchor", self.id);
            self.say(Feed::AnchorBroken {
                team: team as u8,
                by: hitter as u8,
            });
        }
        true
    }

    /// One frame of the fight, `dt` seconds of it: the clock and the
    /// timetable, the generators, everyone's health and the dead standing
    /// again, the computer's players, the fireballs, the potions, traps
    /// and pools, and whether anyone has won. A plain function of the
    /// game: it reads where people are from [`Game::at`], and whatever it
    /// does that the world is to be shown or told it only says
    /// ([`Game::events`]), so that a whole game can be played in a test.
    fn step(&mut self, dt: f32) {
        self.left -= dt;
        let before = self.elapsed;
        self.elapsed += dt;
        let due: Vec<Happening> = happening_between(before, self.elapsed).collect();
        for event in due {
            self.happen(event);
        }
        self.raise_zombies();
        // The buses go by the clock: this machine's, which every device
        // has for the town's; and in a game played out with none, as the
        // tests play them, by the fight's own time.
        let now = if self.clock > 0.0 { self.clock } else { f64::from(self.elapsed) };
        let before = self.bus_clock.unwrap_or(now - f64::from(dt));
        self.drive_buses(now);
        let stood = self.at.clone();
        self.mint_step();
        self.crumble(dt);
        self.fight(dt);
        self.buses(before, now, &stood);
        self.fireballs(dt);
        self.dynamites(dt);
        self.outfit(dt);
        let teams = self.teams_in_play();
        let winner = match (teams.len(), self.left <= 0.0) {
            (0, _) => Some(None),
            (1, _) => Some(Some(teams[0])),
            (_, true) => Some(None),
            _ => None,
        };
        if let Some(winner) = winner {
            self.stage = ArenaStage::Over { winner };
            self.left = RESULTS_SECS;
            info!("Bridge Wars game {}: over, team {winner:?} wins", self.id);
        }
    }

    /// The generators, for one frame of the fight: what they make comes up
    /// on its pad for the whole room ([`Event::Made`]), and what anyone
    /// has come near enough to is theirs, and gone from the room
    /// ([`Event::Taken`]).
    fn mint_step(&mut self) {
        // Each team's own generator as fast as the team has raised it.
        let forge = self.outfits.teams.map(|team| team.forge);
        let dice = &mut self.dice;
        let made = self.mint.run(self.elapsed, &forge, &mut || dice.f32());
        self.events.extend(made.into_iter().map(Event::Made));
        for seat in 0..self.seats.len() {
            if self.bot[seat] || self.gone[seat] || !self.present[seat] {
                continue;
            }
            let taken = self.gather(seat, self.at[seat]);
            if !taken.is_empty() {
                debug!("Bridge Wars game {}: seat {seat} picked up {}, and holds {:?}", self.id, taken.len(), self.purses[seat]);
            }
            self.events.extend(taken.into_iter().map(|piece| Event::Taken(piece.id)));
        }
        // And then the computer's players, each no more than its next
        // purchases need ([`Squad::wants`]): whoever of them it matters
        // most to first, so that a team's bridge comes before its
        // keeper's shield. People are served before any of them, whatever
        // their seats.
        let mut takers: Vec<(u8, usize)> = (0..self.seats.len().min(ARENA_SEATS))
            .filter(|&seat| self.brained(seat) && self.alive(seat))
            .map(|seat| (self.squad.wants(seat).1, seat))
            .collect();
        takers.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        // (What someone who died dropped is anyone's who walks into it,
        // whatever they were saving for: nobody leaves that lying.)
        let dropped = self.mint.pieces().iter().any(Piece::dropped);
        for (_, seat) in takers {
            let (mut wants, _) = self.squad.wants(seat);
            if wants == [0; 4] && !dropped {
                continue;
            }
            let taken = self.mint.pick_up_where(self.at[seat], self.elapsed, &mut |piece| {
                if piece.dropped() {
                    return true;
                }
                let wanted = &mut wants[piece.item.index()];
                let take = *wanted > 0;
                *wanted = wanted.saturating_sub(1);
                take
            });
            for piece in &taken {
                let held = &mut self.purses[seat][piece.item.index()];
                *held = held.saturating_add(piece.worth);
                if piece.dropped() {
                    self.spilt[2] += u32::from(piece.worth);
                }
            }
            if !taken.is_empty() {
                debug!("Bridge Wars game {}: seat {seat} (bot) picked up {}, and holds {:?}", self.id, taken.len(), self.purses[seat]);
            }
            self.events.extend(taken.into_iter().map(|piece| Event::Taken(piece.id)));
        }
    }

    /// Everyone's health coming back, the dead standing again or going
    /// out, and the bots' moves, for one frame of the fight.
    fn fight(&mut self, dt: f32) {
        for hurt in self.hurt.iter_mut().chain(self.anchor_hurt.iter_mut()) {
            *hurt = (*hurt - dt).max(0.0);
        }
        for seat in 0..self.seats.len() {
            if !self.in_play(seat) {
                continue;
            }
            if let Some(left) = self.dead_for[seat] {
                let left = left - dt;
                if left > 0.0 {
                    self.dead_for[seat] = Some(left);
                } else {
                    self.dead_for[seat] = None;
                    self.health[seat] = HEALTH;
                    self.since_hit[seat] = 0.0;
                    self.stand(seat);
                    info!("Bridge Wars game {}: seat {seat} stands again", self.id);
                }
                continue;
            }
            if self.health[seat] <= 0.0 {
                continue;
            }
            // Whoever a bus flattened is up again once their while is.
            if self.flat[seat] > 0.0 {
                self.flat[seat] = (self.flat[seat] - dt).max(0.0);
                if self.flat[seat] <= 0.0 {
                    debug!("Bridge Wars game {}: seat {seat} is up again", self.id);
                }
            }
            self.since_hit[seat] += dt;
            if self.since_hit[seat] >= REGEN_AFTER && self.health[seat] < HEALTH {
                self.health[seat] = (self.health[seat] + REGEN_PER_SEC * dt).min(HEALTH);
            }
        }
        // Someone whose device has gone stands where they stood, with
        // nobody to move them, and can be hit there like anyone. With
        // nothing under them, the plank they stood on taken by a blast,
        // or gone in mid-air over the void, they fall as anyone would:
        // their own device would have said so ([`Ask::Respawn`]).
        for seat in 0..self.seats.len().min(ARENA_SEATS) {
            if !self.away[seat] || self.bot[seat] || !self.present[seat] || !self.alive(seat) {
                continue;
            }
            let at = self.at[seat];
            if self.nav.ground(at.xz()).is_none_or(|ground| at.y < ground - AWAY_UNDER) {
                self.fall(seat);
            }
        }
        // The bots, each with everyone else where they stand this frame:
        // but for whoever is unseen, whom they do not go for.
        let standing: Vec<(usize, Vec2)> = (0..self.seats.len())
            .filter(|&seat| self.alive(seat) && self.present[seat] && self.minds[seat].falling.is_none())
            .filter(|&seat| !self.outfits.kit(seat).unseen(self.elapsed))
            .map(|seat| (seat, self.at[seat].xz()))
            .collect();
        // What the computer's players see of everyone, by seat.
        let mut seen: Vec<Seen> = (0..self.seats.len())
            .map(|seat| Seen {
                at: self.at[seat],
                alive: self.alive(seat) && self.present[seat],
                in_play: self.in_play(seat),
                unseen: self.outfits.kit(seat).unseen(self.elapsed),
                person: !self.bot[seat] && !self.vacant[seat],
                health: self.health[seat] / HEALTH,
                flat: self.flat[seat] > 0.0,
            })
            .collect();
        for seat in 0..self.seats.len() {
            if !self.bot[seat] || !self.alive(seat) {
                continue;
            }
            if self.brained(seat) {
                self.play_bot(seat, &mut seen, dt);
                continue;
            }
            // A zombie: at whoever of the players is nearest that it can
            // see, over whatever can be walked, and a blow at whoever is
            // in its reach, which knocks as one thrown standing does; one
            // a bus has flattened lies where it is; and one knocked into
            // the void, or with its bridge gone from under it, is gone
            // for a while ([`Horde`], [`Game::fall`]).
            if let Some(zombie) = self.zombie(seat) {
                let prey: Vec<Prey> = standing
                    .iter()
                    .filter(|(other, _)| *other < ARENA_SEATS)
                    .map(|&(other, _)| Prey {
                        seat: other,
                        at: self.at[other],
                    })
                    .collect();
                let was = Spot::of(self.at[seat], Quat::from_rotation_y(self.yaw[seat]));
                let hit = self.horde.step(zombie, &self.nav, &prey, self.flat[seat] > 0.0, dt);
                if let Some(body) = self.horde.body(zombie) {
                    self.at[seat] = body.at;
                    self.yaw[seat] = body.yaw;
                }
                if let Some(victim) = hit {
                    self.wound(victim, seat, was, false);
                }
                if self.at[seat].y < ISLAND_TOP - FALL_TO_DIE {
                    self.fall(seat);
                }
                continue;
            }
            if self.flat[seat] > 0.0 {
                continue;
            }
            let was = Spot::of(self.at[seat], Quat::from_rotation_y(self.yaw[seat]));
            if let Some(victim) = think(self, seat, &standing, dt) {
                // A bot's punch is thrown on the run.
                self.wound(victim, seat, was, true);
            }
            if self.minds[seat].falling.is_some() && self.at[seat].y < ISLAND_TOP - FALL_TO_DIE {
                self.fall(seat);
            }
        }
        // Where the builder each of the computer's players has out is
        // now, for the room to be shown it (`flush`): on the ground or on
        // its own planks, facing the way it walks.
        for seat in 0..ARENA_SEATS {
            let builder = if seat < self.seats.len() && self.brained(seat) { self.squad.builder_at(seat) } else { None };
            self.bot_builders[seat] = builder.map(|(at, way)| {
                let height = self.nav.ground(at).unwrap_or(PLANK_TOP);
                (Vec3::new(at.x, height, at.y), (-way.x).atan2(-way.y))
            });
        }
    }

    /// One frame of the computer's player in `seat` ([`Squad::step`]): it
    /// thinks and moves, and what comes of it is carried out here, as it
    /// would be for a person who asked: a punch landed, a purchase made,
    /// a rung laid for the room to be sent, a potion drunk. Fallen far
    /// enough, it is dead.
    fn play_bot(&mut self, seat: usize, seen: &mut [Seen], dt: f32) {
        let was = Spot::of(self.at[seat], Quat::from_rotation_y(self.yaw[seat]));
        // Every stick of dynamite there is, as a bot sees one: where it
        // lies or will, how long until it blows, and whose it is.
        let sticks: Vec<Stick> = self
            .sticks
            .iter()
            .map(|stick| Stick {
                at: stick.lob.to.xz(),
                left: (stick.lob.secs() + DYNAMITE_FUSE - stick.lob.age).max(0.0),
                team: arena_team(stick.by),
            })
            .collect();
        let view = View {
            elapsed: self.elapsed,
            nav: &self.nav,
            seats: seen,
            purses: &self.purses,
            outfits: &self.outfits,
            mint: &self.mint,
            anchors: self.anchors,
            sticks: &sticks,
            traffic: self.buses.zip(self.bus_clock).map(|(buses, clock)| Traffic { buses, clock }),
        };
        let order = self.squad.step(seat, &view, dt);
        if let Some(bot) = self.squad.bot(seat) {
            self.at[seat] = bot.body.at;
            self.yaw[seat] = bot.body.yaw;
            seen[seat].at = bot.body.at;
        }
        if let Some(victim) = order.punch
            && victim < self.seats.len()
            && self.alive(victim)
            && arena_team(victim) != arena_team(seat)
            && self.at[victim].xz().distance(self.at[seat].xz()) <= REACH + BODY_RADIUS + 0.5
        {
            self.wound(victim, seat, was, order.running);
        }
        if let Some(team) = order.strike {
            self.strike_anchor(seat, usize::from(team), self.at[seat].xz(), REACH);
        }
        if let Some(ware) = order.buy
            && self.buy(seat, ware, self.at[seat].xz())
        {
            debug!("Bridge Wars game {}: seat {seat} (bot) bought {ware:?}", self.id);
            self.squad.bought(seat, ware, self.elapsed);
        }
        for (chain, index, at, heading) in order.rungs {
            if self.rungs[seat] >= MOST_RUNGS {
                break;
            }
            self.rungs[seat] += 1;
            self.nav.hear(seat as u8, chain, index, at);
            self.events.push(Event::Rung(Plank {
                seat: seat as u8,
                chain,
                index,
                x: at.x,
                z: at.y,
                heading,
            }));
        }
        // A Bridge Builder sent off: one comes off its buttons, as off a
        // person's who sends one (`Use::Builder`), and its bridge comes
        // as rungs in the frames after, as a person's does.
        if order.build {
            if self.outfits.spend(seat, Utility::Builder) {
                debug!("Bridge Wars game {}: seat {seat} (bot) sent a Bridge Builder", self.id);
            } else {
                self.squad.unsend(seat);
            }
        }
        if let Some(heading) = order.throw {
            self.throw(seat, self.at[seat], heading);
        }
        if let Some((heading, far)) = order.blast {
            self.lob(seat, self.at[seat], heading, far);
        }
        if let Some(potion) = order.drink
            && self.outfits.drink(seat, potion, self.elapsed) == Some(Utility::Healing)
        {
            self.health[seat] = (self.health[seat] + HEALTH * HEALING).min(HEALTH);
        }
        if self.at[seat].y < ISLAND_TOP - FALL_TO_DIE {
            self.fall(seat);
        }
    }

    /// The buses, for one frame of the fight (since 2026-11-16): gone from
    /// where they were by the clock a frame ago, `before`, to where they
    /// are by it `now`, each runs over whoever its front has come to
    /// ([`Bus::runs_over`]) of those the game moves itself, each where it
    /// stood as the frame began, `stood`, and where it stands now: the
    /// computer's players, the zombies, and whoever's device has gone,
    /// who stands where they stood with nobody to move them out of its
    /// way. A person's own device says a bus has run their player over
    /// ([`Ask::RunOver`]). Nobody lying flat already.
    fn buses(&mut self, before: f64, now: f64, stood: &[Vec3]) {
        let (Some(was), Some(is)): (Option<Vec<Bus>>, Option<Vec<Bus>>) = (
            (0..BUSES).map(|bus| self.bus_log.at(bus, before)).collect(),
            (0..BUSES).map(|bus| self.bus_log.at(bus, now)).collect(),
        ) else {
            return;
        };
        for seat in 0..self.seats.len() {
            let ours = self.bot[seat] || self.away[seat];
            if !ours || !self.alive(seat) || !self.present[seat] || self.flat[seat] > 0.0 {
                continue;
            }
            let (then, here) = (stood.get(seat).copied().unwrap_or(self.at[seat]), self.at[seat]);
            // On a crossing, one of the players is stopped for, not run
            // over (since 2026-11-17); a zombie is not.
            if seat < ARENA_SEATS && crossing_at(here).is_some() {
                continue;
            }
            if (0..BUSES).any(|bus| is[bus].runs_over(&was[bus], then, here, BODY_RADIUS)) {
                self.run_over(seat);
            }
        }
    }

    /// The buses, `now` by the clock they go by (since 2026-11-17; Hajun:
    /// "the bus will stop before the crosswalk if there is a person on the
    /// crosswalk until they cross it ... if no player on that crosswalk,
    /// just same speed pass through crosswalk"): thrown round their lanes
    /// as the game is begun ([`Buses::new`]), and each slowing for the
    /// next crossing ahead of it that any of the players stands on,
    /// stopping short of it while they are there, and going on once they
    /// are not ([`Buses::drive`]). Any of the players: a person, one of
    /// the computer's, or whoever's device has gone and stands there;
    /// standing in the fight, and not riding a bus. Not a zombie (mine:
    /// "person", "player"). In every stage of the game, so that they are
    /// going round as the room looks down on them.
    fn drive_buses(&mut self, now: f64) {
        let dice = &mut self.dice;
        let buses = self.buses.get_or_insert_with(|| Buses::new(now, &mut || dice.f32()));
        let mut crowded = [false; 4];
        for seat in 0..self.seats.len().min(ARENA_SEATS) {
            if !(self.health[seat] > 0.0 && !self.gone[seat] && !self.out[seat]) || !self.present[seat] || self.aboard[seat] {
                continue;
            }
            if let Some(crossing) = crossing_at(self.at[seat]) {
                crowded[crossing] = true;
            }
        }
        buses.drive(now, crowded);
        self.bus_log.take(buses);
        self.bus_clock = Some(now);
    }

    /// A bus has run over whoever is in `seat` (since 2026-11-16; Hajun:
    /// "the bus 'squishes' that person, so the person is squashed on the
    /// road for 10 second. they cannot move for 10second ... where they
    /// can be attacked by other players too (being attacked doesnt reduce
    /// the timer) and they also take an initial -20hp dmg"): its damage
    /// off their health, less what their shield keeps off, as a
    /// fireball's is; and they lie flat where they are for
    /// [`SQUASH_SECS`], moved by nobody and nothing, hit as anyone is
    /// ([`Game::harm`] knocks them nowhere), and run over by no bus again
    /// until they are up ("if they are already squished, they cannot be
    /// squished twice even if another bus come, until they get up").
    /// Whoever was unseen is seen again, for being hit. With no health
    /// left they are dead of it, which the chat says
    /// ([`Feed::RunOver`]), what they held of sapphire and amethyst lying
    /// round them on the road ([`Game::drop_spoils`]): it is nobody's
    /// doing but the bus's.
    fn run_over(&mut self, seat: usize) {
        // (Nothing hurts a zombie: it is laid flat, and no more.)
        let damage = if self.zombie(seat).is_some() { 0.0 } else { self.outfits.through(seat, SQUASH_DAMAGE) };
        self.outfits.reveal(seat);
        self.health[seat] = (self.health[seat] - damage).max(0.0);
        self.since_hit[seat] = 0.0;
        self.squashes[0] += 1;
        if self.health[seat] <= 0.0 {
            info!("Bridge Wars game {}: a bus ran seat {seat} over, and killed them", self.id);
            self.squashes[1] += 1;
            self.say(Feed::RunOver { seat: seat as u8 });
            self.drop_spoils(seat);
            self.die(seat);
            return;
        }
        info!("Bridge Wars game {}: a bus ran seat {seat} over, left with {}", self.id, self.health[seat]);
        self.flat[seat] = SQUASH_SECS;
        self.flat_until[seat] = self.bus_clock.unwrap_or(self.clock) + f64::from(SQUASH_SECS);
        // One the game moves is on the road where it was, whatever was
        // carrying it.
        if self.bot[seat] {
            let mind = &mut self.minds[seat];
            mind.shove = Vec2::ZERO;
            mind.lift = 0.0;
            if !self.brained(seat) {
                self.at[seat].y = self.nav.ground(self.at[seat].xz()).unwrap_or(BUS_ROAD);
            }
        }
    }

    /// Who lies flat, as the room is sent it ([`Flattened`]): by seat,
    /// with until when.
    fn flattened(&self) -> Flattened {
        Flattened(
            (0..self.seats.len())
                .filter(|&seat| self.flat[seat] > 0.0 && self.alive(seat))
                .map(|seat| (seat as u8, self.flat_until[seat]))
                .collect(),
        )
    }

    /// The bridges a blast has doomed, for one frame of the fight
    /// ([`Nav::crumble`], since 2026-11-15): every plank whose time has
    /// come is gone from under whoever stands on it. Nobody is told: each
    /// device was told as the stick blew which planks would go and when
    /// ([`Blasted::later`]), and takes them itself. Whoever of another
    /// team than the thrower's stood on one is theirs to have knocked off
    /// the world, should they fall ([`Game::fall`], [`CREDIT_SECS`]): a
    /// person's own device says whether they do, the computer's players
    /// fall here, and whoever's device has gone falls with the rest
    /// ([`Game::fight`]).
    fn crumble(&mut self, dt: f32) {
        let fallen = self.nav.crumble(dt);
        if fallen.is_empty() {
            return;
        }
        self.planks_blown += fallen.len() as u32;
        self.shed();
        for seat in 0..self.seats.len().min(ARENA_SEATS) {
            if !self.alive(seat) || !self.present[seat] {
                continue;
            }
            let at = self.at[seat].xz();
            let Some(by) = fallen.iter().find(|plank| plank.under(at)).map(|plank| usize::from(plank.by)) else {
                continue;
            };
            if by < ARENA_SEATS && arena_team(by) != arena_team(seat) {
                self.last_hit[seat] = Some((by, self.elapsed, Weapon::Dynamite));
            }
        }
    }

    /// Every fireball in the air, for one frame of the fight: flown on,
    /// and burst where it meets someone of another team than its
    /// thrower's, or at the end of its range. Its blast takes its damage
    /// off everyone of another team it reaches, less what their shields
    /// keep off, and knocks them from where it burst; the room is no
    /// longer sent it ([`Event::Burst`]), which is how every device knows
    /// to draw the blast.
    fn fireballs(&mut self, dt: f32) {
        let mut flying = std::mem::take(&mut self.balls);
        flying.retain_mut(|(ball, flight, thrower)| {
            let spent = flight.fly(dt);
            let team = arena_team(*thrower);
            let others: Vec<(usize, Vec3)> = (0..self.seats.len())
                .filter(|&seat| arena_team(seat) != team && self.alive(seat) && self.present[seat])
                .map(|seat| (seat, self.at[seat]))
                .collect();
            if !spent && !others.iter().any(|(_, feet)| flight.meets(*feet)) {
                return true;
            }
            let burst = flight.at();
            let from = Spot::of(burst - Vec3::Y * FIREBALL_HEIGHT, Quat::IDENTITY);
            debug!("Bridge Wars game {}: seat {thrower}'s fireball burst at {burst:.1}, {:.1} m on", self.id, flight.flown);
            let mut hurt = false;
            for (seat, feet) in others {
                if in_blast(burst, feet) {
                    let damage = self.outfits.through(seat, FIREBALL_DAMAGE);
                    self.harm(seat, *thrower, from, false, damage, Weapon::Fireball);
                    hurt = true;
                }
            }
            // To have hurt someone is to have attacked them: whoever
            // threw it unseen is seen again ("attacking another person or
            // anchor reveals u").
            if hurt {
                self.outfits.reveal(*thrower);
            }
            self.events.push(Event::Burst(*ball));
            false
        });
        self.balls = flying;
    }

    /// Every stick of dynamite thrown, for one frame of the fight: lobbed
    /// on; as it comes down, lying there if there is ground for it to lie
    /// on ([`caught`]), a bridge's plank or the land, and gone into the
    /// void if there is not ([`Event::Fell`]); and once its fuse has burnt
    /// down, blown up. Its blast takes its damage off everyone of another
    /// team than its thrower's that it reaches, less what their shields
    /// keep off, knocking nobody (mine: Hajun's file gives the fireball a
    /// knock and this none); and every plank whose middle it reaches is
    /// gone, whoever laid it, from under whoever stands on it
    /// ([`Nav::blast`]): the computer's players fall with them, and a
    /// person's own device has them fall, told which planks went
    /// ([`Event::Blew`], [`Blasted`]). An anchor it does not touch.
    ///
    /// And since 2026-11-15 the rest of the bridge falls after them
    /// ([`Nav::cascade`], [`Game::crumble`]; Hajun: "from that point it
    /// will destroy cascadingly the entire bridge like the truly connected
    /// bridge ... but it shouldnt destroy the whole bridge instantly"):
    /// which planks and when is worked out as the stick blows, and said
    /// with it.
    fn dynamites(&mut self, dt: f32) {
        let mut lying = std::mem::take(&mut self.sticks);
        lying.retain_mut(|stick| {
            stick.lob.tick(dt);
            if !stick.lob.down() {
                return true;
            }
            let at = stick.lob.to;
            if !stick.lies {
                if !caught(at.xz(), |spot| self.nav.ground(spot).is_some()) {
                    debug!("Bridge Wars game {}: seat {}'s dynamite fell into the void at {:.1}", self.id, stick.by, at.xz());
                    self.events.push(Event::Fell(stick.id));
                    return false;
                }
                stick.lies = true;
            }
            if !stick.lob.blown() {
                return true;
            }
            let team = arena_team(stick.by);
            let reached: Vec<usize> = (0..self.seats.len())
                .filter(|&seat| arena_team(seat) != team && self.alive(seat) && self.present[seat])
                .filter(|&seat| in_dynamite(at, self.at[seat]))
                .collect();
            if !reached.is_empty() {
                self.outfits.reveal(stick.by);
            }
            let planks = self.nav.blast(at.xz(), DYNAMITE_RADIUS);
            // And the rest of the bridge they were part of falls after
            // them, plank by plank from the hole outward.
            let later = self.nav.cascade(&planks);
            self.nav.doom(&later, stick.by as u8);
            // What had been dropped on the planks it took goes with them.
            self.shed();
            // (The planks first, since the night of 2026-11-15: whoever
            // it kills on the bridge it takes has died with nothing under
            // them that is staying, and what they held is the thrower's,
            // not dropped onto planks that are going,
            // [`Game::drop_spoils`].)
            for seat in reached {
                let damage = self.outfits.through(seat, DYNAMITE_DAMAGE);
                self.hurt_by(seat, stick.by, damage, Weapon::Dynamite);
            }
            info!(
                "Bridge Wars game {}: seat {}'s dynamite blew at {:.1}, taking {} planks, and {} more to fall over {:.1} s",
                self.id,
                stick.by,
                at.xz(),
                planks.len(),
                later.len(),
                later.last().map_or(0.0, |(_, secs)| *secs)
            );
            self.blasts += 1;
            self.planks_blown += planks.len() as u32;
            self.events.push(Event::Blew {
                stick: stick.id,
                shown: Blasted {
                    x: at.x,
                    y: at.y,
                    z: at.z,
                    planks,
                    at: self.clock,
                    later: Blasted::later_of(&later),
                },
            });
            false
        });
        self.sticks = lying;
    }

    /// The potions, the traps and the heal pools, for one frame of the
    /// fight ([`Outfits::tick`]), with every living player where they
    /// stand: whoever a pool heals has that much health back.
    fn outfit(&mut self, dt: f32) {
        let standing: Vec<(usize, Vec2)> = (0..self.seats.len().min(ARENA_SEATS))
            .filter(|&seat| self.alive(seat) && self.present[seat])
            .map(|seat| (seat, self.at[seat].xz()))
            .collect();
        let ticked = self.outfits.tick(dt, self.elapsed, &standing);
        for seat in ticked.healed {
            self.health[seat] = (self.health[seat] + HEAL_POOL).min(HEALTH);
        }
        for (seat, island) in ticked.sprung {
            info!("Bridge Wars game {}: seat {seat} walked into team {island}'s trap", self.id);
        }
    }
}

impl Arenas {
    /// The game filling now, if it still has room for `count` who go in
    /// together ([`Game::seats_for`]); otherwise a new one, the old one
    /// left to fill with whoever is in it and the computer.
    fn filling(&mut self, commands: &mut Commands, old_bots: bool, count: usize) -> &mut Game {
        let open = self.filling.filter(|id| {
            self.games
                .get(id)
                .is_some_and(|game| !game.closing && game.seats_for(count).is_some())
        });
        let id = open.unwrap_or_else(|| {
            let id = self.next;
            self.next += 1;
            let shown = commands
                .spawn((Replicated, InRoom(Room::Arena(id)), Seats::default()))
                .id();
            let mut game = Game::new(id, shown, fastrand::u64(..));
            game.old_bots = old_bots;
            // Which team its people begin on is this game's own.
            game.order = thrown_order();
            info!("Bridge Wars game {id} opened, its people on team {} first", arena_team(game.order[0]));
            self.games.insert(id, game);
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
            game.leave(seat);
        }
    }

    /// Whether `body`'s seat in the game in `room` waits for it, its
    /// device having gone (since 2026-11-15; until then a seat was given
    /// up the moment a connection closed, and a phone closes its own
    /// whenever its app leaves the foreground, an iPhone's for as little
    /// as a pull on the bar of notifications: on 2026-11-13 Hajun's
    /// partner in a game was out of it for good, and back in the town,
    /// for under four seconds away). The body is kept, standing where it
    /// stood, for as long as the game goes on with anyone connected to
    /// it, and a minute more ([`ABANDONED`]); whoever comes back as its
    /// player is put back in it ([`Arenas::returns`]). A seat that has
    /// been left for good, back to the town, does not wait.
    pub(crate) fn keeps(&self, room: Room, body: Entity) -> bool {
        let Room::Arena(id) = room else {
            return false;
        };
        self.games.get(&id).is_some_and(|game| {
            !game.closing && game.seat_of(body).is_some_and(|seat| !game.bot[seat] && !game.gone[seat])
        })
    }

    /// What becomes of whoever plays `body`, whose device is back, the
    /// body having been kept in the game in `room` ([`Arenas::keeps`]).
    /// `still_there`, the device says it is in a game of Bridge Wars yet:
    /// its app was only put away, and it has the game as it left it.
    ///
    /// - Still there, it carries on where it stands ([`Return::Resume`]):
    ///   unless its player has died meanwhile, which it was not there to
    ///   see, and then it is told to begin the game afresh, as one that
    ///   has forgotten is.
    /// - Not there, the app closed and opened again, it is put back in
    ///   the game ([`Return::Back`]) so long as the fight is on and
    ///   someone else of the people in it is connected to it (Hajun: "u
    ///   should rejoin the game as long as there is atleast 1 real player
    ///   in that game still").
    /// - Otherwise the seat is given up, and they are in the town
    ///   ([`Return::GiveUp`]): the caller walks them out.
    pub(crate) fn returns(&mut self, room: Room, body: Entity, still_there: bool) -> Return {
        let Some((game, seat)) = self.of(room, body) else {
            return Return::GiveUp;
        };
        if game.gone[seat] || game.closing {
            return Return::GiveUp;
        }
        let died = std::mem::take(&mut game.died_away[seat]);
        if still_there && (!died || !game.playing()) {
            game.away[seat] = false;
            return Return::Resume;
        }
        if !game.playing() || !(still_there || game.watched_but(seat)) {
            return Return::GiveUp;
        }
        game.away[seat] = false;
        Return::Back {
            seat: seat as u8,
            metres: game.metres[seat],
            compass: game.outfits.kit(seat).compass,
        }
    }

    /// Whether the game in `room` has yet to begin its fight: in its
    /// waiting room still, or counting down. What someone's friends are
    /// told of where they are (`social`).
    pub(crate) fn queued(&self, room: Room) -> bool {
        let Room::Arena(id) = room else {
            return false;
        };
        self.games
            .get(&id)
            .is_some_and(|game| matches!(game.stage, ArenaStage::Waiting | ArenaStage::Countdown))
    }

    /// Whether `body` is done with the game in `room`, over or out of it,
    /// and free to go into another game, as it is for another of Bridge
    /// Wars ([`Ask::Arena`]): what the four games played at tables go by
    /// (`tables`, since 2026-11-18).
    pub(crate) fn done_with(&self, room: Room, body: Entity) -> bool {
        let Room::Arena(id) = room else {
            return false;
        };
        self.games
            .get(&id)
            .is_some_and(|game| game.seat_of(body).is_some_and(|seat| game.over() || game.out[seat]))
    }
}

/// What becomes of someone whose device is back while their body was kept
/// in a game ([`Arenas::returns`]).
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Return {
    /// The device never left the game: it carries on where it stands.
    Resume,
    /// It is put back in the game, in this seat, which it begins afresh
    /// ([`Tell::Back`]), with what only the device knew of it.
    Back { seat: u8, metres: f32, compass: bool },
    /// The seat is given up: they are in the town.
    GiveUp,
}

/// Seats `going`, who go in together, the first of them having asked, in
/// the game filling now, or in a new one if that has not the room for them:
/// two by two onto one team ([`Game::seats_for`]), each out of the town, or
/// out of the game that was over for them (of Bridge Wars, or since
/// 2026-11-18 at a table, `tables`), and into the waiting room.
#[allow(clippy::too_many_arguments)]
fn seat_together(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    arenas: &mut Arenas,
    tables: &mut Tables,
    towns: &mut Towns,
    old_bots: bool,
    going: &[Entity],
    now: f64,
) {
    for &body in going {
        let Ok((body_of, ..)) = bodies.get(body) else {
            continue;
        };
        if let Some((old, seat)) = arenas.of(body_of.room, body) {
            old.leave(seat);
        }
        tables.walk_out(body_of.room, body);
        towns.stand(body);
    }
    let game = arenas.filling(commands, old_bots, going.len());
    let Some(seats) = game.seats_for(going.len()) else {
        return;
    };
    let room = game.room();
    for (&body, seat) in going.iter().zip(seats) {
        let Ok((body_of, spot, venue, member)) = bodies.get_mut(body) else {
            continue;
        };
        game.put(seat, body, member.clone(), false);
        if let Some(client) = body_of.client {
            tell(tells, client, Tell::Seated { seat: seat as u8 });
        }
        // Where the game has just stood them in the waiting room.
        let (at, turn) = (game.at[seat], Quat::from_rotation_y(game.yaw[seat]));
        place(
            commands,
            tells,
            body,
            (body_of, spot, venue),
            room,
            Venue::Arena,
            seat as u8,
            Some(Spot::of(at, turn)),
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

/// What players ask of Bridge Wars.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<&Plays>,
    mut bodies: Bodies,
    paying: Query<(), With<Paying>>,
    (mut arenas, mut tables): (ResMut<Arenas>, ResMut<Tables>),
    mut towns: ResMut<Towns>,
    (everyone, mut social): (Res<Everyone>, ResMut<Social>),
    config: Res<Config>,
    time: Res<Time>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs_f64();
    let clock = wall_clock();
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
                // Whoever asks goes in, and with them, if they lead a
                // group, everyone of it who is here and free to (`social`,
                // since 2026-11-13); in a group someone else leads, nobody
                // does: only its leader starts a game.
                let Some(party) = social.party(body_of.who.0) else {
                    tell(&mut tells, client, Tell::Refused(Refusal::NotLeader));
                    continue;
                };
                // Free to: in the town, or in a game that is over for them,
                // of Bridge Wars or at a table (`tables`); and not paying a
                // fee for another (`fees::Paying`, since 2026-11-18: until
                // then someone could be seated here with House Builder's
                // fee on the way, and then moved there as it was taken).
                let (mut going, mut staying) = (Vec::new(), Vec::new());
                for id in party {
                    let entity = everyone.0.get(&Who(id)).copied();
                    let free = entity.is_some_and(|entity| {
                        !paying.contains(entity)
                            && bodies.get(entity).is_ok_and(|(body_of, ..)| {
                                body_of.client.is_some()
                                    && match body_of.room {
                                        Room::Town(_) => true,
                                        Room::Arena(_) => arenas
                                            .of(body_of.room, entity)
                                            .is_some_and(|(game, seat)| game.over() || game.out[seat]),
                                        Room::Table(..) => tables.done_with(body_of.room, entity),
                                        Room::Home(_) | Room::Game(_) => false,
                                    }
                            })
                    });
                    match entity {
                        Some(entity) if free => going.push(entity),
                        _ => staying.push(id),
                    }
                }
                // The one who asked is first of them, or it is nobody's to
                // go.
                if going.first() != Some(&body) {
                    tell(&mut tells, client, Tell::Refused(Refusal::Busy));
                    continue;
                }
                for id in staying {
                    social.left_behind(id);
                }
                seat_together(
                    &mut commands,
                    &mut tells,
                    &mut bodies,
                    &mut arenas,
                    &mut tables,
                    &mut towns,
                    config.arena_old_bots,
                    &going,
                    now,
                );
            }
            Ask::Leave | Ask::Travel(Venue::Town) if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                game.leave(seat);
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
                // (Where each is in the world, whichever of them is
                // riding a bus.)
                let (from_at, at_at) = (stands(&from, clock, &game.bus_log), stands(&at, clock, &game.bus_log));
                if from_at.xz().distance(at_at.xz()) > PUNCH_SLACK {
                    continue;
                }
                // Whoever a bus has flattened throws no punch.
                if game.flat[hitter] > 0.0 {
                    continue;
                }
                // Knocked from where they stand now, as their body says,
                // and told so once the frame's fight is done ([`flush`]).
                if !game.bot[victim] {
                    game.at[victim] = at_at;
                }
                // One of the computer's is knocked from where the puncher
                // is in the world; a person's device is told the place as
                // it was said, on a bus or off it.
                let from = if game.bot[victim] { Spot::of(from_at, from.rotation()) } else { from };
                game.wound(victim, hitter, from, *running);
            }
            Ask::HitAnchor { team } if matches!(room, Room::Arena(_)) => {
                let Some((game, hitter)) = arenas.of(room, body) else {
                    continue;
                };
                let Ok((_, from, ..)) = bodies.get(game.seats[hitter]) else {
                    continue;
                };
                if game.flat[hitter] > 0.0 {
                    continue;
                }
                game.strike_anchor(hitter, usize::from(*team), stands(from, clock, &game.bus_log).xz(), PUNCH_SLACK);
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
                if !game.playing() || !sane || game.rungs[seat] >= MOST_RUNGS || game.flat[seat] > 0.0 {
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
                game.nav.hear(seat as u8, *chain, *index, Vec2::new(*x, *z));
            }
            // Fallen into the void: dead, and back if the anchor stands.
            Ask::Respawn if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                if game.playing() && game.alive(seat) {
                    game.fall(seat);
                }
            }
            Ask::Buy(ware) if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                let Ok((_, at, ..)) = bodies.get(game.seats[seat]) else {
                    continue;
                };
                if game.flat[seat] <= 0.0 && game.buy(seat, *ware, stands(at, clock, &game.bus_log).xz()) {
                    info!("Bridge Wars game {}: seat {seat} bought {ware:?}", game.id);
                    tell(&mut tells, client, Tell::Bought(*ware));
                }
            }
            Ask::Use(used) if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                // (Whoever a bus has flattened uses nothing.)
                if !game.playing() || !game.alive(seat) || game.flat[seat] > 0.0 {
                    continue;
                }
                match used {
                    Use::Fireball { heading } => {
                        let Ok((_, at, ..)) = bodies.get(game.seats[seat]) else {
                            continue;
                        };
                        game.throw(seat, stands(at, clock, &game.bus_log), *heading);
                    }
                    Use::Builder => {
                        game.outfits.spend(seat, Utility::Builder);
                    }
                    Use::Dynamite { heading, far } => {
                        let Ok((_, at, ..)) = bodies.get(game.seats[seat]) else {
                            continue;
                        };
                        game.lob(seat, stands(at, clock, &game.bus_log), *heading, *far);
                    }
                    Use::Potion(potion) => {
                        let drunk = game.outfits.drink(seat, *potion, game.elapsed);
                        if drunk == Some(Utility::Healing) {
                            game.health[seat] = (game.health[seat] + HEALTH * HEALING).min(HEALTH);
                        }
                        debug!("Bridge Wars game {}: seat {seat} drank {drunk:?}", game.id);
                    }
                }
            }
            // Where the builder they sent off is now, for everyone else
            // in the game to see it; or that it is gone.
            Ask::Builder { id, spot } if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                let key = (seat, *id);
                let Some(spot) = spot else {
                    if let Some((shown, _)) = game.builders.remove(&key) {
                        commands.entity(shown).try_despawn();
                    }
                    continue;
                };
                let at = spot.translation();
                let sane = at.x.abs() < FURTHEST && at.z.abs() < FURTHEST && at.y.abs() < FURTHEST && spot.aboard.is_none();
                if !game.playing() || !sane {
                    continue;
                }
                match game.builders.get_mut(&key) {
                    Some((shown, heard)) => {
                        *heard = now;
                        commands.entity(*shown).insert(*spot);
                    }
                    None => {
                        let out = game.builders.keys().filter(|(owner, _)| *owner == seat).count();
                        if out >= MOST_BUILDERS {
                            continue;
                        }
                        // In its owner's name and look, and never sent to
                        // its owner's own device, which draws its own.
                        let member = Member {
                            id: builder_id(game.id, seat, *id),
                            ai: true,
                            ..game.members[seat].clone()
                        };
                        let shown = commands
                            .spawn((
                                Replicated,
                                member,
                                *spot,
                                Venue::Arena,
                                InRoom(game.room()),
                                NotFor(client),
                                BuilderOf(seat as u8),
                            ))
                            .id();
                        game.builders.insert(key, (shown, now));
                    }
                }
            }
            // The metres of bridge they have left, which their device
            // counts: kept to give back to one that has forgotten.
            Ask::Metres(metres) if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                if game.playing() && game.alive(seat) && metres.is_finite() {
                    game.metres[seat] = metres.clamp(0.0, MOST_METRES);
                }
            }
            // A bus has run them over, their own device says: flat, with
            // its damage off them, if they are standing in the fight, not
            // flat already, not riding, and where they last said they
            // stood is anywhere near that bus's front, as it is or was in
            // the moments their word took to come.
            Ask::RunOver { bus } if matches!(room, Room::Arena(_)) => {
                let Some((game, seat)) = arenas.of(room, body) else {
                    continue;
                };
                let Ok((_, spot, ..)) = bodies.get(game.seats[seat]) else {
                    continue;
                };
                if !game.playing() || !game.alive(seat) || game.flat[seat] > 0.0 || spot.aboard.is_some() {
                    continue;
                }
                let (at, bus) = (spot.translation().xz(), usize::from(*bus));
                let near = bus < BUSES
                    && (0..=4).any(|ago| {
                        game.bus_log.at(bus, clock - f64::from(ago) * 0.2).is_some_and(|then| {
                            then.world(Vec2::new(0.0, BUS_HALF_LENGTH)).distance(at) < RUN_OVER_SLACK
                        })
                    });
                if !near {
                    debug!("Bridge Wars game {}: seat {seat} says bus {bus} ran them over, {at:.1} being nowhere near it", game.id);
                    continue;
                }
                game.bus_clock = Some(clock);
                game.run_over(seat);
            }
            Ask::Skip if matches!(room, Room::Arena(_)) => {
                if let Some((game, _)) = arenas.of(room, body)
                    && config.allow_skip
                {
                    match game.stage {
                        ArenaStage::Waiting => {
                            game.since_join = BOT_WAIT;
                            game.hurried = true;
                        }
                        _ => game.left = 0.0,
                    }
                }
            }
            _ => {}
        }
    }
}

/// Moves every game on: fills its room, counts down, sends everyone to their
/// islands, runs the fight ([`Game::step`]) and does in the world what the
/// game says is to be done there ([`flush`]), and ends it once the result
/// has been up its while, or once nobody is left.
#[allow(clippy::too_many_arguments)]
fn run(
    time: Res<Time>,
    config: Res<Config>,
    mut arenas: ResMut<Arenas>,
    mut bodies: Bodies,
    mut everyone: ResMut<Everyone>,
    mut social: ResMut<Social>,
    mut towns: ResMut<Towns>,
    mut tells: MessageWriter<ToClients<Tell>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs_f64();
    // The time by this machine's clock, which every device has for the
    // town's (`clock`): what a blast is stamped with, and what the buses
    // go by.
    let clock = wall_clock();
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

        sync(game, &bodies, clock);
        game.clock = clock;
        match game.stage {
            ArenaStage::Waiting => {
                game.since_join += dt;
                // Nobody has joined for a while: the computer takes the
                // seats left, however many people there are (every game
                // is two to a team on four islands, the people teamed up
                // with each other first, [`Game::order`]), its players
                // coming in one at a time ([`Game::bots_due`]).
                let begun = game.coming.is_some();
                for _ in 0..game.bots_due(dt) {
                    seat_bot(&mut commands, game, now, false);
                }
                if game.coming.is_some() && !begun {
                    info!(
                        "Bridge Wars game {}: nobody joined in {BOT_WAIT} s: {} people, and the computer comes for the seats left",
                        game.id,
                        game.people()
                    );
                }
                game.mill(dt, false);
                if game.free_seat().is_none() {
                    game.pad();
                    game.stage = ArenaStage::Countdown;
                    game.left = COUNTDOWN;
                }
            }
            ArenaStage::Countdown => {
                game.left -= dt;
                game.mill(dt, true);
                if game.left <= 0.0 {
                    // Whoever's device went while the room waited, and
                    // is not back as the fight begins, has given their
                    // seat up: a seat waits through a fight, not for one
                    // that has yet to begin with nobody in it.
                    for seat in 0..game.seats.len().min(ARENA_SEATS) {
                        if game.bot[seat] || game.vacant[seat] {
                            continue;
                        }
                        let body = game.seats[seat];
                        let Ok((body_of, ..)) = bodies.get(body) else {
                            continue;
                        };
                        if body_of.client.is_some() {
                            continue;
                        }
                        info!(
                            "Bridge Wars game {}: seat {seat}'s device is not back as the fight begins, and the seat is free again",
                            game.id
                        );
                        everyone.0.remove(&body_of.who);
                        commands.entity(body).try_despawn();
                        game.vacate(seat);
                    }
                    // Whoever left during the countdown has the computer
                    // in their seat, on its island from the start.
                    while game.free_seat().is_some() {
                        seat_bot(&mut commands, game, now, true);
                    }
                    game.mingle.clear();
                    for seat in 0..game.seats.len() {
                        game.stand(seat);
                    }
                    game.stage = ArenaStage::Playing;
                    // From the start, or, for trying the timetable out,
                    // from as far into it as the server was told to.
                    game.elapsed = config.arena_from.clamp(0.0, PLAY_SECS - 1.0);
                    game.left = PLAY_SECS - game.elapsed;
                    let dice = &mut game.dice;
                    game.mint = Mint::new(game.elapsed, &mut || dice.f32());
                    // Nothing in anyone's purse, but for trying the shops
                    // out (`RT_ARENA_PURSE`).
                    if config.arena_purse > 0 {
                        for seat in 0..game.purses.len().min(ARENA_SEATS) {
                            if !game.bot[seat] {
                                game.purses[seat] = [config.arena_purse; 4];
                            }
                        }
                    }
                    info!("Bridge Wars game {}: fight, {} s to play", game.id, game.left);
                    // Who the computer's players are this time, and how
                    // their teams mean to open: for reading a game back by.
                    let cast: Vec<String> = (0..game.seats.len().min(ARENA_SEATS))
                        .filter_map(|seat| {
                            game.squad
                                .bot(seat)
                                .map(|bot| format!("{} {} is {}", seat, game.members[seat].name, bot.persona().name))
                        })
                        .collect();
                    let openings: Vec<String> = (0..ARENA_TEAMS).map(|team| format!("{:?}", game.squad.plan(team).opening)).collect();
                    info!("Bridge Wars game {}: {}; the teams open {}", game.id, cast.join(", "), openings.join(", "));
                    // Begun late, everything before then has happened.
                    let before: Vec<Happening> = happening_between(-1.0, game.elapsed).collect();
                    for event in before {
                        game.happen(event);
                    }
                }
            }
            ArenaStage::Playing => game.step(dt),
            ArenaStage::Over { .. } => game.left -= dt,
        }
        // The buses go round in every stage, the fight's own being driven
        // in its step.
        if !game.playing() {
            game.drive_buses(clock);
        }
        flush(&mut commands, &mut tells, &mut bodies, game, now);

        // Whoever of the people in it played it to its end has earned
        // toward their level (`social`, since 2026-11-13): more, on the
        // team that won. Once a game.
        if let ArenaStage::Over { winner } = game.stage
            && social.first_time(Minigame::BridgeWars, game.id)
        {
            for seat in 0..game.seats.len().min(ARENA_SEATS) {
                // (Nor whoever's device has gone, their seat waiting.)
                if game.bot[seat] || game.vacant[seat] || game.gone[seat] || game.away[seat] {
                    continue;
                }
                let won = winner == Some(arena_team(seat));
                social.award(game.members[seat].id, XP_PLAYED + if won { XP_WON } else { 0 });
            }
        }

        // Only what has changed goes out again.
        let round = game.round();
        if game.sent.as_ref() != Some(&round) {
            commands.entity(game.shown).insert(round.clone());
            game.sent = Some(round);
        }
        if game.sent_seats != game.members {
            commands.entity(game.shown).insert(Seats(game.members.clone()));
            game.sent_seats.clone_from(&game.members);
        }
        if game.sent_purses != game.purses {
            commands.entity(game.shown).insert(Purses(game.purses.clone()));
            game.sent_purses.clone_from(&game.purses);
        }
        if game.sent_kits != game.outfits.kits {
            commands.entity(game.shown).insert(Kits(game.outfits.kits.clone()));
            game.sent_kits.clone_from(&game.outfits.kits);
        }
        if game.sent_upgrades != game.outfits.teams {
            commands.entity(game.shown).insert(Upgraded(game.outfits.teams));
            game.sent_upgrades = game.outfits.teams;
        }
        let flat = game.flattened();
        if game.sent_flat != flat {
            commands.entity(game.shown).insert(flat.clone());
            game.sent_flat = flat;
        }
        if game.buses != game.sent_buses
            && let Some(buses) = game.buses
        {
            commands.entity(game.shown).insert(buses);
            game.sent_buses = Some(buses);
        }
    }
    for id in ended {
        arenas.games.remove(&id);
        if arenas.filling == Some(id) {
            arenas.filling = None;
        }
    }
}

/// Copies into `game` where each person in it last said they stood, and
/// whether their body is in its room to be played with ([`Game::at`],
/// [`Game::present`]): what a step of the fight goes by. The computer's
/// players are where the game has them.
fn sync(game: &mut Game, bodies: &Bodies, clock: f64) {
    let room = game.room();
    for seat in 0..game.seats.len() {
        if game.bot[seat] || game.vacant[seat] {
            continue;
        }
        match bodies.get(game.seats[seat]) {
            Ok((body, spot, ..)) => {
                // On one of the buses, where they are is a place on it
                // (`Spot::aboard`): in the world, where the bus has taken
                // it, by the clock.
                let bus = riding(spot);
                game.present[seat] = body.room == room && (spot.aboard.is_none() || bus.is_some());
                game.away[seat] = body.client.is_none();
                game.aboard[seat] = bus.is_some();
                game.at[seat] = stands(spot, clock, &game.bus_log);
                game.yaw[seat] = yaw_of(spot.rotation())
                    + bus.and_then(|bus| game.bus_log.at(bus, clock)).map_or(0.0, |bus| bus.yaw());
            }
            Err(_) => {
                game.present[seat] = false;
                game.away[seat] = false;
            }
        }
    }
}

/// Does in the world what `game` has said is to be done there
/// ([`Game::events`]): the room is sent what the generators have made and
/// the fireballs thrown, and no longer what has been taken or has burst;
/// whoever was knocked is told; whoever stands again is sent to their
/// island; a zombie risen has its body. And each of the computer's players
/// is sent where the game has moved it, if it has.
fn flush(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    game: &mut Game,
    now: f64,
) {
    let room = game.room();
    for event in std::mem::take(&mut game.events) {
        match event {
            Event::Made(piece) => {
                let entity = commands
                    .spawn((
                        Replicated,
                        InRoom(room),
                        Loot::of(&piece),
                    ))
                    .id();
                game.loot.insert(piece.id, entity);
            }
            Event::Taken(piece) => {
                if let Some(entity) = game.loot.remove(&piece) {
                    commands.entity(entity).try_despawn();
                }
            }
            Event::Thrown { ball, shown } => {
                let entity = commands.spawn((Replicated, InRoom(room), shown)).id();
                game.shown_balls.insert(ball, entity);
            }
            Event::Burst(ball) => {
                if let Some(entity) = game.shown_balls.remove(&ball) {
                    commands.entity(entity).try_despawn();
                }
            }
            Event::Lobbed { stick, shown } => {
                let entity = commands.spawn((Replicated, InRoom(room), shown)).id();
                game.shown_sticks.insert(stick, entity);
            }
            Event::Blew { stick, shown } => {
                if let Some(entity) = game.shown_sticks.remove(&stick) {
                    commands.entity(entity).try_despawn();
                }
                // Kept to the end of the game with the rungs: which planks
                // are gone is part of every bridge from now on.
                let entity = commands.spawn((Replicated, InRoom(room), shown)).id();
                game.planks.push(entity);
            }
            Event::Fell(stick) => {
                if let Some(entity) = game.shown_sticks.remove(&stick) {
                    commands.entity(entity).try_despawn();
                }
            }
            Event::Knocked { seat, by, from, running } => {
                if let Ok((body, ..)) = bodies.get(game.seats[seat])
                    && let Some(client) = body.client
                {
                    tell(tells, client, Tell::Knocked { by, from, running });
                }
            }
            Event::Land(seat) => land(commands, tells, bodies, game, seat, now),
            Event::Rung(plank) => {
                let entity = commands.spawn((Replicated, InRoom(room), plank)).id();
                game.planks.push(entity);
            }
            Event::Feed(feed) => {
                let only = feed.only_for();
                for seat in 0..game.seats.len().min(ARENA_SEATS) {
                    if game.bot[seat] || game.vacant[seat] || game.gone[seat] || only.is_some_and(|team| team != arena_team(seat)) {
                        continue;
                    }
                    if let Ok((body, ..)) = bodies.get(game.seats[seat])
                        && body.room == room
                        && let Some(client) = body.client
                    {
                        tell(tells, client, Tell::Feed(feed));
                    }
                }
            }
            Event::Zombie(seat) => {
                let body = commands
                    .spawn((
                        Replicated,
                        game.members[seat].clone(),
                        Spot::of(game.at[seat], Quat::from_rotation_y(game.yaw[seat])),
                        Venue::Arena,
                        InRoom(room),
                        Body::bot(room, seat as u8, now),
                    ))
                    .id();
                game.seats[seat] = body;
            }
        }
    }
    for seat in 0..game.seats.len() {
        if !game.bot[seat] {
            continue;
        }
        let Ok((_, mut spot, ..)) = bodies.get_mut(game.seats[seat]) else {
            continue;
        };
        let mut next = Spot::of(game.at[seat], Quat::from_rotation_y(game.yaw[seat]));
        // Come to a stop, it is sent once more as standing still, so that
        // nobody watching carries it on the way it was going while they
        // wait for an update that is not coming ([`Spot::still`]).
        next.still = (next.x, next.y, next.z) == (spot.x, spot.y, spot.z);
        if !next.same(&spot) {
            *spot = next;
        }
    }
    // A builder a person sent off, with nothing heard of it from their
    // device for a while, is gone: the device has, or its connection.
    game.builders.retain(|_, (shown, heard)| {
        let kept = now - *heard <= BUILDER_SILENT;
        if !kept {
            commands.entity(*shown).try_despawn();
        }
        kept
    });
    // The builders the computer's players have out: a body for each, in
    // its owner's name and look, moved as it walks, and gone as it is.
    for seat in 0..ARENA_SEATS {
        match (game.bot_builders[seat], game.shown_bot_builders.get(&seat).copied()) {
            (Some((at, yaw)), shown) => {
                let mut next = Spot::of(at, Quat::from_rotation_y(yaw));
                match shown {
                    Some((entity, last)) => {
                        next.still = (next.x, next.y, next.z) == (last.x, last.y, last.z);
                        if !next.same(&last) {
                            commands.entity(entity).insert(next);
                            game.shown_bot_builders.insert(seat, (entity, next));
                        }
                    }
                    None => {
                        let member = Member {
                            id: builder_id(game.id, seat, u8::MAX),
                            ai: true,
                            ..game.members[seat].clone()
                        };
                        let entity = commands
                            .spawn((Replicated, member, next, Venue::Arena, InRoom(room), BuilderOf(seat as u8)))
                            .id();
                        game.shown_bot_builders.insert(seat, (entity, next));
                    }
                }
            }
            (None, Some((entity, _))) => {
                commands.entity(entity).try_despawn();
                game.shown_bot_builders.remove(&seat);
            }
            (None, None) => {}
        }
    }
}

/// What the bot in `seat` does this frame: it is moved ([`Game::at`]), and
/// whom it punches, if anyone, is given back. It keeps to the disc of its
/// island, having no map: knocked off it, it falls.
fn think(game: &mut Game, seat: usize, standing: &[(usize, Vec2)], dt: f32) -> Option<usize> {
    let (island, reach) = game.home(seat);
    let edge = reach - BODY_RADIUS;
    // A zombie comes on slower than anyone runs, and never runs away.
    let zombie = game.zombie(seat).is_some();
    let pace = if zombie { ZOMBIE_SPEED } else { RUN_SPEED };
    let beaten = !zombie && game.health[seat] / HEALTH < FLEE_BELOW;
    let mut at = game.at[seat];
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
        game.at[seat] = at;
        return None;
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
        if at.xz().distance(island) > reach + 0.2 {
            mind.falling = Some(mind.lift.min(0.0));
            game.at[seat] = at;
            return None;
        }
    }
    if airborne {
        // Knocked into the air, it is carried, and does nothing else.
        game.at[seat] = at;
        return None;
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
                && there.distance(island) <= reach + 1.5
        })
        .min_by(|a, b| a.1.distance_squared(here).total_cmp(&b.1.distance_squared(here)))
        .copied();
    let mut step = Vec2::ZERO;
    let mut facing = Vec2::ZERO;
    let mut punch = None;
    match target {
        Some((other, there)) => {
            mind.goal = None;
            let off = there - here;
            let toward = off.normalize_or_zero();
            if beaten {
                // Nearly beaten, it runs for it: away, or round the edge.
                let away = -toward;
                let ahead = here + away * 3.0;
                step = if ahead.distance(island) <= edge {
                    away
                } else {
                    let round = Vec2::new(-away.y, away.x);
                    if (here + round * 3.0).distance(island) <= edge { round } else { -round }
                } * RUN_SPEED * dt;
                facing = step;
            } else {
                facing = toward;
                if off.length() > REACH * PRESS_IN {
                    step = toward * pace * dt;
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
                let angle = game.dice.f32() * std::f32::consts::TAU;
                let out = game.dice.f32() * (edge - 1.5);
                mind.goal = Some(island + Vec2::new(angle.cos(), angle.sin()) * out);
            }
            if let Some(goal) = mind.goal {
                let off = goal - here;
                if off.length() < 0.2 {
                    mind.goal = None;
                } else {
                    step = off.normalize() * (STROLL * dt).min(off.length());
                    facing = off;
                }
            }
        }
    }
    let mut next = here + step;
    if next.distance(island) > edge {
        next = island + (next - island).normalize_or(Vec2::X) * edge;
    }
    if facing.length_squared() > 1e-6 {
        game.yaw[seat] = (-facing.x).atan2(-facing.y);
    }
    game.at[seat] = Vec3::new(next.x, ISLAND_TOP, next.y);
    punch
}

/// Sends the person in `seat` to where they arrive on their island,
/// through their device.
fn land(
    commands: &mut Commands,
    tells: &mut MessageWriter<ToClients<Tell>>,
    bodies: &mut Bodies,
    game: &Game,
    seat: usize,
    now: f64,
) {
    let room = game.room();
    let Ok((body_of, spot, venue, _)) = bodies.get_mut(game.seats[seat]) else {
        return;
    };
    if body_of.room != room {
        return;
    }
    move_to(commands, tells, game.seats[seat], (body_of, spot, venue), room, Venue::Arena, seat as u8, now);
}

/// The computer, in the next free seat of `game` ([`Game::free_seat`]),
/// standing in the waiting room, or, `landed`, where its seat arrives on
/// its island, the fight being about to begin: a boy or a girl by the
/// toss of a coin, in any of that side's colours, under one of that
/// side's names that nobody there has, as House Builder's are
/// (`builder::seat_bot`).
fn seat_bot(commands: &mut Commands, game: &mut Game, now: f64, landed: bool) {
    // Into the waiting room, the last of the seats left, so that the
    // people's stay free the longest ([`Game::last_free_seat`]); as the
    // fight begins, whichever is left.
    let free = if landed { game.free_seat() } else { game.last_free_seat() };
    let Some(seat) = free else {
        return;
    };
    let member = game.bot_member(seat);
    let body = commands.spawn_empty().id();
    game.put(seat, body, member.clone(), true);
    // Where the game has just stood it in the waiting room.
    let spot = if landed {
        game.bot_spot(seat)
    } else {
        Spot::of(game.at[seat], Quat::from_rotation_y(game.yaw[seat]))
    };
    commands.entity(body).insert((
        Replicated,
        member,
        spot,
        Venue::Arena,
        InRoom(game.room()),
        Body::bot(game.room(), seat as u8, now),
    ));
}

/// Who the computer's player in `seat` of the game numbered `game` is to
/// everyone: clear of people's ids, which count up from 1, of House
/// Builder's bots, which count down from the top, and of every other
/// game's, a game having no more than [`MOST_SEATS`] seats, its zombies
/// counted.
fn bot_id(game: u32, seat: usize) -> u64 {
    u64::MAX / 2 - u64::from(game) * MOST_SEATS - seat as u64
}

/// The most seats a game has: its players' and its zombies', and room.
const MOST_SEATS: u64 = 64;

/// Who the builder numbered `builder` that the one in `seat` of the game
/// numbered `game` has sent off is to everyone ([`BuilderOf`]): clear of
/// people's ids, of every bot's ([`bot_id`]), and of every other
/// builder's.
fn builder_id(game: u32, seat: usize, builder: u8) -> u64 {
    u64::MAX / 4 - (u64::from(game) * MOST_SEATS + seat as u64) * 256 - u64::from(builder)
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
        if game.bot[seat] && body != Entity::PLACEHOLDER {
            commands.entity(body).try_despawn();
        }
    }
    for plank in game.planks.drain(..) {
        commands.entity(plank).try_despawn();
    }
    for (_, thing) in game.loot.drain() {
        commands.entity(thing).try_despawn();
    }
    for (_, ball) in game.shown_balls.drain() {
        commands.entity(ball).try_despawn();
    }
    for (_, stick) in game.shown_sticks.drain() {
        commands.entity(stick).try_despawn();
    }
    for (_, (builder, _)) in game.builders.drain() {
        commands.entity(builder).try_despawn();
    }
    for (_, (builder, _)) in game.shown_bot_builders.drain() {
        commands.entity(builder).try_despawn();
    }
    game.bot_builders = [None; ARENA_SEATS];
    // Whatever the game had still to say goes unsaid: there is no room
    // left to be shown it.
    game.balls.clear();
    game.sticks.clear();
    game.events.clear();
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
        let mut game = Game::new(0, Entity::PLACEHOLDER, 7);
        game.stage = ArenaStage::Playing;
        game.left = PLAY_SECS;
        game.mint = Mint::new(0.0, &mut || 0.5);
        for seat in 0..ARENA_SEATS {
            let member = Member {
                id: seat as u64 + 1,
                name: format!("Seat {seat}"),
                look: "boy".into(),
                ai: seat > 0,
            };
            game.put(seat, Entity::from_bits(seat as u64 + 1), member, seat > 0);
        }
        game
    }

    /// Sudden death, with `count` zombies risen, one after another as
    /// they do ([`Game::raise_zombies`]).
    fn risen(game: &mut Game, count: usize) {
        game.happen(Happening::SuddenDeath);
        for _ in 0..count {
            game.seat_zombie();
        }
    }

    /// A waiting room with nobody in it yet.
    fn empty_room() -> Game {
        let mut game = full_game();
        game.stage = ArenaStage::Waiting;
        for list in [&mut game.bot, &mut game.gone, &mut game.vacant, &mut game.out] {
            list.clear();
        }
        game.seats.clear();
        game.members.clear();
        game.last_hit.clear();
        game.minds.clear();
        game.health.clear();
        game.since_hit.clear();
        game.hurt.clear();
        game.dead_for.clear();
        game.rungs.clear();
        game.purses.clear();
        game.at.clear();
        game.yaw.clear();
        game.present.clear();
        game.away.clear();
        game.died_away.clear();
        game.metres.clear();
        game.flat.clear();
        game.flat_until.clear();
        game.outfits = Outfits::default();
        game
    }

    fn person(n: u64) -> Member {
        Member {
            id: n,
            name: format!("Person {n}"),
            look: "boy".into(),
            ai: false,
        }
    }

    /// The computer in every seat of `game` nobody is in, as the wait's end
    /// puts it there, without the entities.
    fn fill_with_bots(game: &mut Game) {
        while let Some(seat) = game.free_seat() {
            let member = game.bot_member(seat);
            game.put(seat, Entity::from_bits(100 + seat as u64), member, true);
        }
        game.pad();
    }

    #[test]
    fn people_are_teamed_up_first_and_the_computer_takes_every_seat_left() {
        // Two people are a team, and four are two teams on opposite
        // islands (Hajun, 2026-11-12: "the 2 player become same team. and
        // if its 4 players, then those 2 players and 2 players are same
        // team too, and so on").
        assert_eq!(arena_team(SEAT_ORDER[0]), arena_team(SEAT_ORDER[1]));
        assert_eq!(arena_team(SEAT_ORDER[2]), arena_team(SEAT_ORDER[3]));
        assert_eq!(ARENA_ISLANDS[arena_island(SEAT_ORDER[0])], -ARENA_ISLANDS[arena_island(SEAT_ORDER[2])]);
        let mut order = SEAT_ORDER;
        order.sort_unstable();
        assert_eq!(order, [0, 1, 2, 3, 4, 5, 6, 7], "every seat is taken once");
        // However many come, each team is filled before the next is
        // begun, and the computer has every seat left once the wait is
        // over: all eight are taken, every team is two, and nobody is out.
        for people in 1..=ARENA_SEATS {
            let mut game = empty_room();
            for n in 0..people {
                let seat = game.seat(Entity::from_bits(n as u64 + 1), person(n as u64 + 1), false);
                assert_eq!(seat, SEAT_ORDER[n], "{people} people: the {n}th to come");
            }
            assert_eq!(game.people(), people);
            let teams: Vec<u8> = (0..people).map(|n| arena_team(SEAT_ORDER[n])).collect();
            for pair in teams.chunks_exact(2) {
                assert_eq!(pair[0], pair[1], "{people} people: teams {teams:?}");
            }
            fill_with_bots(&mut game);
            assert_eq!(game.seats.len(), ARENA_SEATS);
            assert!(game.vacant.iter().all(|vacant| !vacant), "{people} people: a seat left empty");
            assert_eq!(game.free_seat(), None);
            assert_eq!(game.people(), people);
            assert_eq!(game.bot.iter().filter(|bot| **bot).count(), ARENA_SEATS - people);
            assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
            assert_eq!(game.round().out, 0);
            assert!((0..ARENA_SEATS).all(|seat| game.health[seat] == HEALTH && !game.gone[seat]));
            // Where a person and the computer are a team, the person has
            // the lower seat, and so is the first to be given what lies
            // on their pad.
            for team in 0..ARENA_TEAMS {
                let (first, second) = (team, team + ARENA_TEAMS);
                assert!(!game.bot[first] || game.bot[second], "{people} people: team {team}");
            }
            // And no two of the computer's players go by one name.
            let mut names: Vec<&str> = game.members.iter().map(|member| member.name.as_str()).collect();
            names.sort_unstable();
            names.dedup();
            assert_eq!(names.len(), ARENA_SEATS, "{people} people: {names:?}");
        }
        // The fight on, whoever is seated is past the players, as a
        // zombie is.
        let mut game = empty_room();
        game.seat(Entity::from_bits(1), person(1), false);
        fill_with_bots(&mut game);
        game.stage = ArenaStage::Playing;
        let zombie = game.seat(Entity::from_bits(99), person(99), true);
        assert_eq!(zombie, ARENA_SEATS);
        assert_eq!(game.zombie(zombie), Some(0));
    }

    #[test]
    fn which_team_people_begin_on_is_thrown_for_each_game_and_an_island_is_still_its_team_s() {
        // The order as it was is that of a game begun on the first team,
        // and a game made is that until whoever opens it throws.
        assert_eq!(seat_order(0, false), SEAT_ORDER);
        assert_eq!(empty_room().order, SEAT_ORDER);
        for first in 0..ARENA_TEAMS {
            for flip in [false, true] {
                let order = seat_order(first, flip);
                let what = format!("begun on team {first}, {flip}: {order:?}");
                // Every seat is taken once; each team's two seats are
                // together, the lower first; and the team begun on is the
                // one asked for, with the one across the map from it next
                // and the two at its sides after, one way round or the
                // other.
                let mut sorted = order;
                sorted.sort_unstable();
                assert_eq!(sorted, [0, 1, 2, 3, 4, 5, 6, 7], "{what}");
                for pair in order.chunks(2) {
                    assert_eq!(arena_team(pair[0]), arena_team(pair[1]), "{what}");
                    assert_eq!(pair[1], pair[0] + ARENA_TEAMS, "{what}");
                }
                assert_eq!(usize::from(arena_team(order[0])), first, "{what}");
                assert_eq!(ARENA_ISLANDS[arena_island(order[0])], -ARENA_ISLANDS[arena_island(order[2])], "{what}");
                assert_eq!(ARENA_ISLANDS[arena_island(order[4])], -ARENA_ISLANDS[arena_island(order[6])], "{what}");
                assert_ne!(seat_order(first, !flip)[4], order[4], "{what}");
                // A game seated by it: whoever comes is given its seats in
                // that order, so the first two are a team, on that team's
                // island and no other, and the next two are a team across
                // the map; the computer takes every seat left, the lower
                // of a team's two being a person's where there is one.
                for people in 1..=ARENA_SEATS {
                    let mut game = empty_room();
                    game.order = order;
                    // (Bodies numbered from 20: the one numbered 1 is the
                    // one that stands for nobody, `Entity::PLACEHOLDER`,
                    // which every empty seat has.)
                    for n in 0..people {
                        let seat = game.seat(Entity::from_bits(n as u64 + 20), person(n as u64 + 1), false);
                        assert_eq!(seat, order[n], "{what}: the {n}th of {people} to come");
                    }
                    assert_eq!(usize::from(arena_team(game.seat_of(Entity::from_bits(20)).unwrap())), first, "{what}");
                    if people >= 2 {
                        let second = game.seat_of(Entity::from_bits(21)).unwrap();
                        assert_eq!(usize::from(arena_team(second)), first, "{what}");
                        assert_eq!(ARENA_ISLANDS[arena_island(second)], ARENA_ISLANDS[first], "{what}");
                    }
                    fill_with_bots(&mut game);
                    assert_eq!((game.seats.len(), game.people(), game.free_seat()), (ARENA_SEATS, people, None), "{what}");
                    assert!(game.vacant.iter().all(|vacant| !vacant), "{what}: a seat left empty with {people} people");
                    assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3], "{what}");
                    for team in 0..ARENA_TEAMS {
                        assert!(!game.bot[team] || game.bot[team + ARENA_TEAMS], "{what}: team {team} with {people} people");
                    }
                    let mut names: Vec<&str> = game.members.iter().map(|member| member.name.as_str()).collect();
                    names.sort_unstable();
                    names.dedup();
                    assert_eq!(names.len(), ARENA_SEATS, "{what}: {names:?}");
                }
                // A group goes onto it two by two as onto any: two onto
                // the team begun on, four onto it and the one across the
                // map, and with a stranger there already, two onto the
                // team across the map whole.
                let mut room = empty_room();
                room.order = order;
                assert_eq!(room.seats_for(1), Some(vec![first]), "{what}");
                assert_eq!(room.seats_for(2), Some(vec![first, first + ARENA_TEAMS]), "{what}");
                assert_eq!(room.seats_for(4), Some(order[..4].to_vec()), "{what}");
                assert_eq!(room.seats_for(3), Some(order[..3].to_vec()), "{what}");
                room.seat(Entity::from_bits(20), person(1), false);
                assert_eq!(room.seats_for(2), Some(order[2..4].to_vec()), "{what}");
                assert_eq!(room.seats_for(3), Some(vec![order[2], order[3], order[1]]), "{what}");
                // A seat left before the fight is the next to come's
                // again, wherever in the order it was.
                room.leave(first);
                assert_eq!(room.free_seat(), Some(first), "{what}");
            }
        }
        // Thrown, a game begins on any of the four about as often as on
        // another, and on its side islands either way round: of 800
        // thrown, each team has about 200 (and never as few as 120, which
        // is one chance in many millions), and each way round about half.
        let (mut firsts, mut flipped) = ([0usize; ARENA_TEAMS], 0usize);
        for _ in 0..800 {
            let order = thrown_order();
            let first = usize::from(arena_team(order[0]));
            firsts[first] += 1;
            assert!(order == seat_order(first, false) || order == seat_order(first, true), "{order:?}");
            flipped += usize::from(order == seat_order(first, true));
        }
        assert!(firsts.iter().all(|&count| (120..=280).contains(&count)), "{firsts:?}");
        assert!((300..=500).contains(&flipped), "{flipped}");
    }

    #[test]
    fn a_group_goes_in_two_by_two_onto_one_team_and_an_odd_one_onto_another() {
        // Each team's two seats are together in the order seats are taken,
        // the lower first: what going in two by two goes by.
        for pair in SEAT_ORDER.chunks(2) {
            assert_eq!(arena_team(pair[0]), arena_team(pair[1]), "{pair:?}");
            assert!(pair[0] < pair[1], "{pair:?}");
        }
        let teams = |seats: &[usize]| -> Vec<u8> { seats.iter().map(|&seat| arena_team(seat)).collect() };
        // Into an empty room: one where one alone sits; two on one team;
        // three, two on one team and the third on another; four, two
        // teams, on opposite islands.
        let room = empty_room();
        assert_eq!(room.seats_for(1), Some(vec![SEAT_ORDER[0]]));
        assert_eq!(room.seats_for(1).map(|seats| seats[0]), room.free_seat());
        assert_eq!(room.seats_for(2), Some(vec![0, 4]));
        assert_eq!(room.seats_for(3), Some(vec![0, 4, 2]));
        assert_eq!(room.seats_for(4), Some(vec![0, 4, 2, 6]));
        assert_eq!(teams(&room.seats_for(4).unwrap()), vec![0, 0, 2, 2]);
        assert_eq!(ARENA_ISLANDS[arena_island(0)], -ARENA_ISLANDS[arena_island(2)]);
        // With one person in it already, alone on their team: two go onto
        // the next team whole, and never beside the stranger; three, two
        // onto a team of their own and the third beside the stranger.
        let mut room = empty_room();
        room.seat(Entity::from_bits(1), person(1), false);
        assert_eq!(room.seats_for(2), Some(vec![2, 6]));
        assert_eq!(room.seats_for(3), Some(vec![2, 6, 4]));
        assert_eq!(room.seats_for(4), Some(vec![2, 6, 1, 5]));
        // Seated so, each pair is one team, and the next to come alone
        // sits where there is room.
        for (n, seat) in room.seats_for(4).unwrap().into_iter().enumerate() {
            room.put(seat, Entity::from_bits(10 + n as u64), person(10 + n as u64), false);
        }
        assert_eq!(room.people(), 5);
        assert_eq!(arena_team(room.seat_of(Entity::from_bits(10)).unwrap()), arena_team(room.seat_of(Entity::from_bits(11)).unwrap()));
        assert_eq!(arena_team(room.seat_of(Entity::from_bits(12)).unwrap()), arena_team(room.seat_of(Entity::from_bits(13)).unwrap()));
        assert_eq!(room.seats_for(1), Some(vec![4]));
        // One team is left whole: two more fit, four do not, nor three
        // once the stranger's teammate has come.
        assert_eq!(room.seats_for(2), Some(vec![3, 7]));
        assert_eq!(room.seats_for(3), Some(vec![3, 7, 4]));
        assert_eq!(room.seats_for(4), None);
        room.seat(Entity::from_bits(2), person(2), false);
        assert_eq!(room.seats_for(3), None);
        assert_eq!(room.seats_for(2), Some(vec![3, 7]));
        // Full, or no longer waiting: nobody.
        fill_with_bots(&mut room);
        assert_eq!(room.seats_for(1), None);
        let mut begun = empty_room();
        begun.stage = ArenaStage::Countdown;
        assert_eq!(begun.seats_for(2), None);
        // The computer still takes every seat a group left.
        let mut room = empty_room();
        for (n, seat) in room.seats_for(3).unwrap().into_iter().enumerate() {
            room.put(seat, Entity::from_bits(1 + n as u64), person(1 + n as u64), false);
        }
        fill_with_bots(&mut room);
        assert_eq!(room.people(), 3);
        assert_eq!(room.bot.iter().filter(|bot| **bot).count(), ARENA_SEATS - 3);
        assert!(room.vacant.iter().all(|vacant| !vacant));
        assert_eq!(room.teams_in_play(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn a_seat_left_before_the_fight_is_anyone_s_again_and_one_left_after_stays_empty() {
        let mut game = empty_room();
        for n in 1..=3 {
            game.seat(Entity::from_bits(n), person(n), false);
        }
        // Three people, in seats 0, 4 and 2. The second leaves the
        // waiting room: their seat is empty, and the next to come has it.
        assert_eq!(game.people(), 3);
        game.leave(4);
        assert!(game.vacant[4] && game.gone[4] && game.out[4] && game.members[4].id == 0);
        assert_eq!(game.seats[4], Entity::PLACEHOLDER);
        assert_eq!(game.people(), 2);
        assert_eq!(game.free_seat(), Some(4));
        assert_eq!(game.seat(Entity::from_bits(9), person(9), false), 4);
        assert_eq!((game.people(), game.members[4].id), (3, 9));
        assert!(!game.vacant[4] && !game.gone[4] && !game.out[4] && game.health[4] == HEALTH);
        // Through the countdown the same, and the computer takes it as
        // the fight begins.
        game.stage = ArenaStage::Countdown;
        game.leave(2);
        assert_eq!(game.free_seat(), Some(2));
        fill_with_bots(&mut game);
        assert!(game.bot[2] && !game.vacant[2]);
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
        // The fight on, a seat left stays its own, empty: nobody takes it,
        // and its team is one short.
        game.stage = ArenaStage::Playing;
        game.leave(0);
        assert!(game.gone[0] && !game.vacant[0] && !game.in_play(0));
        assert_eq!(game.free_seat(), None);
        assert_eq!(game.people(), 1);
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3], "seat 4 still plays for team 0");
        // With everyone gone from an empty waiting room, nobody is left.
        let mut game = empty_room();
        game.seat(Entity::from_bits(1), person(1), false);
        game.leave(0);
        assert!((0..game.seats.len()).all(|seat| game.gone[seat] || game.bot[seat]));
        assert_eq!(game.free_seat(), Some(0));
    }

    /// A frame of a waiting room's wait, as [`run`] counts it, without the
    /// entities: the computer's players that are due come in. Which seats
    /// they took.
    fn wait_a_frame(game: &mut Game, dt: f32) -> Vec<usize> {
        game.since_join += dt;
        let mut came = Vec::new();
        for _ in 0..game.bots_due(dt) {
            let seat = game.last_free_seat().expect("one is due only while a seat is free");
            let member = game.bot_member(seat);
            game.put(seat, Entity::from_bits(100 + seat as u64), member, true);
            came.push(seat);
        }
        game.mill(dt, false);
        came
    }

    #[test]
    fn the_computer_s_players_come_in_one_at_a_time_and_the_people_s_seats_are_the_last_taken() {
        // Hajun, 2026-11-16: "make them join slightly delay one by one
        // rather than all at once". One person, and nobody else comes.
        let dt = 1.0 / 60.0;
        let mut game = empty_room();
        game.seat(Entity::from_bits(1), person(1), false);
        let mut clock = 0.0;
        while clock < BOT_WAIT - 0.1 {
            assert!(wait_a_frame(&mut game, dt).is_empty(), "one came {clock} s in");
            clock += dt;
        }
        // The wait over, the first comes at once, and the rest one at a
        // time, each a thrown while after the last, whatever seating one
        // does to the time since the last to join.
        let mut came: Vec<(f32, usize)> = Vec::new();
        while game.free_seat().is_some() {
            clock += dt;
            let now = wait_a_frame(&mut game, dt);
            assert!(now.len() <= 1, "{} came in one frame", now.len());
            came.extend(now.into_iter().map(|seat| (clock, seat)));
            assert!(clock < BOT_WAIT + 12.0, "only {} have come {clock} s in", came.len());
        }
        assert_eq!(came.len(), ARENA_SEATS - 1);
        assert!(came[0].0 - BOT_WAIT < 0.2, "the first came {} s in", came[0].0);
        for pair in came.windows(2) {
            let gap = pair[1].0 - pair[0].0;
            assert!(gap >= BOT_GAP.0 - 2.0 * dt && gap <= BOT_GAP.1 + 2.0 * dt, "{gap} s between two of them");
        }
        let all_in = came[came.len() - 1].0 - came[0].0;
        assert!((1.5..=6.6).contains(&all_in), "all in after {all_in} s");
        // The seat beside the person is the last the computer takes, and
        // each of them is in the room passing the time.
        assert_eq!(came[came.len() - 1].1, SEAT_ORDER[1]);
        assert_eq!(game.people(), 1);
        assert!((0..ARENA_SEATS).all(|seat| game.bot[seat] == game.mingle.has(seat)));

        // Someone who comes while they are still coming is teamed up with
        // the person there, and two who come together have a team to
        // themselves; and the computer goes on coming without waiting
        // again.
        let mut game = empty_room();
        game.seat(Entity::from_bits(1), person(1), false);
        let mut in_now = 0;
        while in_now < 3 {
            in_now += wait_a_frame(&mut game, dt).len();
        }
        assert_eq!(game.seats_for(2), Some(vec![SEAT_ORDER[2], SEAT_ORDER[3]]));
        assert_eq!(game.seat(Entity::from_bits(2), person(2), false), SEAT_ORDER[1]);
        assert_eq!(arena_team(SEAT_ORDER[0]), arena_team(SEAT_ORDER[1]));
        let mut since = 0.0;
        while wait_a_frame(&mut game, dt).is_empty() {
            since += dt;
            assert!(since <= BOT_GAP.1 + 2.0 * dt, "none came for {since} s after someone joined");
        }
        // The wait skipped, they all come at once.
        let mut game = empty_room();
        game.seat(Entity::from_bits(1), person(1), false);
        game.hurried = true;
        assert_eq!(wait_a_frame(&mut game, dt).len(), ARENA_SEATS - 1);
        assert_eq!(game.free_seat(), None);
    }

    #[test]
    fn in_the_waiting_room_nobody_stands_by_their_seat_and_the_computer_s_players_go_about() {
        use roundtown_net::bots::mingle::{ARRIVES, HALF};
        // Where someone is stood as they come in is thrown, and says
        // nothing of their seat, which is their team (Hajun, 2026-11-16:
        // "your team selection should not be shown/determine until the
        // game start"): the same seat of two games is two places, about
        // the middle, facing it.
        let room = |seed: u64| {
            let mut game = empty_room();
            game.mingle = Mingle::new(seed);
            game
        };
        let mut places = Vec::new();
        for seed in 0..12 {
            let mut game = room(seed);
            let seat = game.seat(Entity::from_bits(1), person(1), false);
            assert_eq!(seat, SEAT_ORDER[0]);
            let at = game.at[seat];
            let out = at.xz().length();
            assert_eq!(at.y, WAITING_FLOOR);
            assert!(out >= ARRIVES.0 - 1e-3 && out <= ARRIVES.1 + 1e-3, "{out} m from the middle");
            let ahead = Quat::from_rotation_y(game.yaw[seat]) * Vec3::NEG_Z;
            assert!(ahead.xz().dot(-at.xz().normalize()) > 0.999, "facing {ahead}");
            places.push(at.xz());
        }
        let ring = waiting_spot(SEAT_ORDER[0]).0.xz();
        assert!(places.iter().filter(|place| place.distance(ring) > 1.0).count() >= 10, "{places:?}");
        places.dedup_by(|a, b| a.distance(*b) < 0.5);
        assert!(places.len() >= 10, "{places:?}");

        // The computer's players, once in, go about the room: over ten
        // seconds every one of them has done something to be seen (gone
        // somewhere, left the floor, or turned round), most of them have
        // gone a good way, none has left the room, and the person in it
        // is where they were.
        let dt = 1.0 / 60.0;
        let mut game = room(5);
        let me = game.seat(Entity::from_bits(1), person(1), false);
        let mine = game.at[me];
        game.hurried = true;
        wait_a_frame(&mut game, dt);
        let from: Vec<Vec3> = game.at.clone();
        let (mut furthest, mut highest, mut turned) = (vec![0.0f32; ARENA_SEATS], vec![0.0f32; ARENA_SEATS], vec![0.0f32; ARENA_SEATS]);
        let mut facing = game.yaw.clone();
        for _ in 0..600 {
            game.mill(dt, false);
            for seat in (0..ARENA_SEATS).filter(|&seat| game.bot[seat]) {
                let at = game.at[seat];
                assert!(at.x.abs() <= HALF + 1e-3 && at.z.abs() <= HALF + 1e-3, "seat {seat} at {at}");
                assert!(at.y >= WAITING_FLOOR && at.y <= WAITING_FLOOR + 1.9, "seat {seat} at {at}");
                furthest[seat] = furthest[seat].max(at.xz().distance(from[seat].xz()));
                highest[seat] = highest[seat].max(at.y - WAITING_FLOOR);
                let turn = (game.yaw[seat] - facing[seat] + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                turned[seat] += turn.abs();
                facing[seat] = game.yaw[seat];
            }
        }
        let bots: Vec<usize> = (0..ARENA_SEATS).filter(|&seat| game.bot[seat]).collect();
        assert_eq!(bots.len(), ARENA_SEATS - 1);
        assert!(
            bots.iter().all(|&seat| furthest[seat] > 1.0 || highest[seat] > 1.0 || turned[seat] > 3.0),
            "{furthest:?} {highest:?} {turned:?}"
        );
        assert!(bots.iter().filter(|&&seat| furthest[seat] > 3.0).count() >= 4, "{furthest:?}");
        assert_eq!(game.at[me], mine);
    }

    #[test]
    fn a_seat_waits_for_whoever_s_device_has_gone_and_they_are_put_back_in_it() {
        // Two people on one team, in seats 0 and 4, the computer in the
        // rest, and the fight on (Hajun, 2026-11-15, of his partner's
        // phone: "suddenly she disconnected and went back to the main
        // lobby for some reason while i still stay in the bridge wars
        // game").
        let home = |seat: usize| arrival(Venue::Arena, seat).0.with_y(ISLAND_TOP);
        let room = Room::Arena(4);
        let (leader, partner) = (Entity::from_bits(1), Entity::from_bits(5));
        let mut arenas = Arenas::default();
        arenas.games.insert(4, with_people(7, &[0, 4], home));
        fn game(arenas: &mut Arenas) -> &mut Game {
            arenas.games.get_mut(&4).expect("there")
        }
        assert_eq!(game(&mut arenas).seat_of(partner), Some(4));
        // A person's seat waits, the computer's does not, and one left
        // for the town is left.
        assert!(arenas.keeps(room, partner) && arenas.keeps(room, leader));
        assert!(!arenas.keeps(room, Entity::from_bits(101)) && !arenas.keeps(Room::Arena(9), partner));
        // Her device goes: she stands where she stood, in play, her team
        // with both its seats, and whatever she held still hers.
        game(&mut arenas).away[4] = true;
        game(&mut arenas).purses[4] = [9, 2, 1, 1];
        game(&mut arenas).metres[4] = 37.5;
        game(&mut arenas).outfits.kits[4].compass = true;
        for _ in 0..90 {
            step_with_people(game(&mut arenas), &home);
        }
        assert!(game(&mut arenas).alive(4) && game(&mut arenas).in_play(4) && !game(&mut arenas).gone[4]);
        assert_eq!(game(&mut arenas).purses[4], [9, 2, 1, 1]);
        // Back with the app only put away, it carries on.
        assert_eq!(arenas.returns(room, partner, true), Return::Resume);
        assert!(!game(&mut arenas).away[4]);
        // Back with the app closed and opened again, it is put back in
        // her seat, with the metres of bridge and the compass only her
        // own device knew of: so long as someone else is there.
        game(&mut arenas).away[4] = true;
        assert_eq!(
            arenas.returns(room, partner, false),
            Return::Back {
                seat: 4,
                metres: 37.5,
                compass: true
            }
        );
        // With nobody else of the people in it connected, a game is not
        // come back to from outside it; but an app that never left it
        // carries on all the same.
        game(&mut arenas).away[4] = true;
        game(&mut arenas).away[0] = true;
        assert_eq!(arenas.returns(room, partner, true), Return::Resume);
        game(&mut arenas).away[4] = true;
        assert_eq!(arenas.returns(room, partner, false), Return::GiveUp);
        // Dead while her device was away, which it was not there to see,
        // it begins the game afresh even if its app never left, with none
        // of what dying loses: once, and it carries on from there.
        game(&mut arenas).away[0] = false;
        game(&mut arenas).away[4] = true;
        game(&mut arenas).die(4);
        assert_eq!(game(&mut arenas).dead_for[4], Some(RESPAWN_SECS));
        assert_eq!(
            arenas.returns(room, partner, true),
            Return::Back {
                seat: 4,
                metres: 0.0,
                compass: false
            }
        );
        assert_eq!(arenas.returns(room, partner, true), Return::Resume);
        // A seat left for the town is not come back to, nor a game over.
        game(&mut arenas).leave(4);
        assert!(!arenas.keeps(room, partner));
        assert_eq!(arenas.returns(room, partner, true), Return::GiveUp);
        game(&mut arenas).stage = ArenaStage::Over { winner: None };
        game(&mut arenas).away[0] = true;
        assert_eq!(arenas.returns(room, leader, false), Return::GiveUp);
        assert_eq!(arenas.returns(room, leader, true), Return::Resume);
    }

    #[test]
    fn whoever_s_device_has_gone_falls_with_the_ground_gone_from_under_them_and_stands_otherwise() {
        let home = |seat: usize| arrival(Venue::Arena, seat).0.with_y(ISLAND_TOP);
        // Gone in mid-air over the void, between her island and the
        // middle: with nobody to say she fell, the game says so.
        let mut game = with_people(7, &[0, 4], home);
        let over_the_void = ARENA_ISLANDS[arena_island(4)] * 0.75;
        assert!(game.nav.ground(over_the_void).is_none(), "{over_the_void} is over the void");
        game.away[4] = true;
        game.at[4] = Vec3::new(over_the_void.x, ISLAND_TOP, over_the_void.y);
        game.metres[4] = 50.0;
        game.step(1.0 / 30.0);
        assert!(!game.alive(4) && game.dead_for[4] == Some(RESPAWN_SECS) && game.died_away[4]);
        assert_eq!(game.metres[4], 0.0);
        // The same of someone who is there is their own device's to say.
        let mut game = with_people(7, &[0, 4], home);
        game.at[4] = Vec3::new(over_the_void.x, ISLAND_TOP, over_the_void.y);
        game.step(1.0 / 30.0);
        assert!(game.alive(4));
        // And under her island, on the way down.
        let mut game = with_people(7, &[0, 4], home);
        game.away[4] = true;
        game.at[4] = home(4) - Vec3::Y * 6.0;
        game.step(1.0 / 30.0);
        assert!(!game.alive(4));
        // She stands again on her island once, with nobody to say where
        // she is, and stays standing: the fall was one fall.
        said(&mut game);
        let mut falls = 1;
        for _ in 0..(3.0 * RESPAWN_SECS * 30.0) as usize {
            game.step(1.0 / 30.0);
            falls += said(&mut game).iter().filter(|feed| matches!(feed, Feed::Fell { seat: 4 } | Feed::Knocked { seat: 4, .. })).count();
            game.events.clear();
        }
        assert!(game.alive(4) && game.at[4].xz().distance(home(4).xz()) < 0.1, "at {}", game.at[4]);
        assert_eq!(falls, 1);
    }

    #[test]
    fn whoever_falls_with_a_bridge_a_stick_has_doomed_was_knocked_off_the_world_by_its_thrower() {
        use roundtown_net::nav::{PLANK_TOP, Region};
        // Two people of team 0, in seats 0 and 4, far out on a bridge of
        // seat 1's from its island to the middle, by the middle's end: one
        // there, one whose device has gone. Seat 2, of another team, blows
        // the island's end of it.
        let mut game = with_people(7, &[0, 4], |_| Vec3::ZERO);
        let span = game.nav.grid().span(Region::Team(1), Region::Middle, 0.0).expect("a span");
        for index in 0..=span.planks() {
            game.nav.hear(1, 0, index as u16, span.rung(index));
        }
        let on = |share: f32| {
            let at = span.point(share);
            Vec3::new(at.x, PLANK_TOP, at.y)
        };
        (game.at[0], game.at[4]) = (on(0.9), on(0.8));
        game.away[4] = true;
        game.purses[4] = [5, 1, 3, 2];
        let stands = |seat: usize| if seat == 0 { on(0.9) } else { on(0.8) };
        game.outfits.kits[2] = Kit::default().holding(Utility::Dynamite, 1);
        let from = on(0.15);
        game.at[2] = from;
        game.squad.stand(2, from, 0.0, 0.0);
        let aim = span.point(0.3) - from.xz();
        assert!(game.lob(2, from, (-aim.x).atan2(-aim.y), aim.length()));
        // Until it blows, and until the falling has reached them, they
        // stand, and nobody has hit them.
        let mut blown = None;
        let mut fell_away = None;
        for _ in 0..(40.0 * 30.0) as usize {
            step_with_people(&mut game, &stands);
            if game.blasts == 1 && blown.is_none() {
                blown = Some(game.elapsed);
                assert!(game.alive(0) && game.alive(4) && game.last_hit[0].is_none() && game.last_hit[4].is_none());
                assert!(game.nav.falls_in(on(0.8).xz()).is_some_and(|left| left > 2.0), "her plank is doomed, and stands");
            }
            // Hers goes from under her: she falls, by seat 2's doing,
            // which has her sapphire and her amethyst and nothing else.
            if !game.alive(4) && fell_away.is_none() {
                fell_away = Some(game.elapsed);
                assert!(game.died_away[4]);
                assert_eq!((game.purses[4], game.purses[2][2], game.purses[2][3]), (Purse::default(), 3, 2));
            }
            if game.nav.planks().count() == 0 {
                break;
            }
        }
        let (blown, fell_away) = (blown.expect("it blew"), fell_away.expect("she fell"));
        let due = cascade_after((0.8 - 0.3) * span.length());
        assert!((fell_away - blown - due).abs() < 1.0, "she fell {} s after the blast, her plank due at {due}", fell_away - blown);
        // His went from under him too: his own device is to say he fell,
        // and when it does, it was seat 2 that knocked him off the world.
        assert!(game.alive(0));
        assert!(matches!(game.last_hit[0], Some((2, _, Weapon::Dynamite))), "{:?}", game.last_hit[0]);
        said(&mut game);
        game.fall(0);
        assert_eq!(said(&mut game), vec![Feed::Knocked { seat: 0, by: 2 }]);
        // Seat 2's own teammate, standing on it, is nobody's doing.
        assert!(game.last_hit[6].is_none());
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
        // The computer's players as they were until 2026-11-12, which the
        // server can be switched back to (`RT_ARENA_BOTS=old`), and as
        // the zombies still are.
        let mut game = full_game();
        game.old_bots = true;
        let seat = 5;
        let island = ARENA_ISLANDS[arena_island(seat)];
        game.stand(seat);
        assert_eq!(game.at[seat].y, ISLAND_TOP);
        // Alone, it strolls, and never off the island.
        for _ in 0..600 {
            let here = game.at[seat].xz();
            let punch = think(&mut game, seat, &[(seat, here)], 1.0 / 60.0);
            assert_eq!(punch, None);
            assert!(game.at[seat].xz().distance(island) <= ARENA_ISLAND_RADIUS - BODY_RADIUS + 1e-3);
            assert_eq!(game.at[seat].y, ISLAND_TOP);
        }
        // An enemy 6 m off on its island: it runs at them and, in reach,
        // punches, no oftener than it may.
        let enemy = island + Vec2::new(6.0, 0.0);
        let mut punches = 0;
        for _ in 0..600 {
            let standing = [(seat, game.at[seat].xz()), (0, enemy)];
            punches += usize::from(think(&mut game, seat, &standing, 1.0 / 60.0).is_some());
        }
        assert!(game.at[seat].xz().distance(enemy) <= REACH + BODY_RADIUS + 0.1, "{}", game.at[seat]);
        assert!(punches >= 8 && punches <= 14, "{punches} punches in ten seconds");
        // A punch that lands takes its damage off a bot and knocks it,
        // away from whoever threw it and up; a body reeling from one
        // takes no other for a while, however many land.
        let mut hit = full_game();
        hit.old_bots = true;
        hit.stand(1);
        let from = Spot::of(hit.at[1] + Vec3::X * 2.0, Quat::IDENTITY);
        hit.wound(1, 0, from, true);
        assert_eq!(hit.health[1], HEALTH - DAMAGE);
        assert!(hit.minds[1].shove.x < -(KNOCK_BACK + SPRINT_BONUS) + 0.01 && hit.minds[1].lift == KNOCK_UP);
        hit.wound(1, 0, from, true);
        assert_eq!(hit.health[1], HEALTH - DAMAGE, "the second, while it reels, takes nothing");
        hit.hurt[1] = 0.0;
        hit.wound(1, 0, from, false);
        assert_eq!(hit.health[1], HEALTH - 2.0 * DAMAGE);
        assert!(hit.events.is_empty(), "a bot is knocked here, with nobody to tell");
        // A person hit is knocked by their own device, which is told who
        // by and from where.
        hit.wound(0, 1, from, false);
        assert_eq!(hit.health[0], HEALTH - DAMAGE);
        match hit.events.as_slice() {
            [Event::Knocked { seat: 0, by, running: false, .. }] => assert_eq!(*by, hit.members[1].id),
            other => panic!("{other:?}"),
        }
        // A teammate right beside it is nobody to go for: never punched.
        let mut idle = full_game();
        idle.old_bots = true;
        idle.stand(seat);
        for _ in 0..600 {
            let here = idle.at[seat].xz();
            let punch = think(&mut idle, seat, &[(seat, here), (1, here + Vec2::new(1.0, 0.0))], 1.0 / 60.0);
            assert_eq!(punch, None, "punched a teammate");
        }
        // Knocked hard toward the edge, up and across as a punch knocks,
        // it goes over and falls.
        game.minds[seat].shove = Vec2::new(KNOCK_BACK + SPRINT_BONUS, 0.0) * 2.0;
        game.minds[seat].lift = KNOCK_UP;
        let mut fell = false;
        for _ in 0..240 {
            let standing = [(seat, game.at[seat].xz()), (0, enemy)];
            think(&mut game, seat, &standing, 1.0 / 60.0);
            if game.minds[seat].falling.is_some() && game.at[seat].y < ISLAND_TOP - 1.0 {
                fell = true;
                break;
            }
        }
        assert!(fell, "still on the island at {}", game.at[seat]);
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

    #[test]
    fn what_a_generator_makes_is_whoever_comes_near_its_and_nobody_else_s() {
        use roundtown_net::{ARENA_PADS, ARENA_SHOPS, Item, PAD_TOP};
        let mut game = full_game();
        // Twenty seconds and a bit of every generator: ten silver and two
        // gold on each team's pad.
        let (dt, mut dice) = (0.05, || 0.5);
        for _ in 0..410 {
            game.elapsed += dt;
            game.mint.run(game.elapsed, &[0; ARENA_TEAMS], &mut dice);
        }
        // The tenth silver of each pad has only just been made. A fifth
        // of a second after, still dropping, it is out of reach over the
        // head of someone standing on the pad, who has the other eleven;
        // someone at the top of a jump a moment later has it too, in the
        // air, with what lies under their feet.
        let pad = ARENA_PADS[0];
        let on_pad = Vec3::new(pad.x, PAD_TOP, pad.y);
        let last = game.mint.pieces().iter().map(|piece| piece.born).fold(0.0, f32::max);
        assert!(game.elapsed - last < DROP_SECS + 0.1, "the tenth silver was made {last} s in");
        assert_eq!(game.mint.clone().pick_up(on_pad, last + 0.2).len(), 11, "the one still dropping is left");
        assert_eq!(game.mint.clone().pick_up(on_pad + Vec3::Y * 1.6, last + 0.36).len(), 12);
        // A second on, with nothing more made, all of them lie there.
        game.elapsed += 1.0;
        let lying = game.mint.pieces().len();
        assert_eq!(lying, 4 * (10 + 2));
        // The computer's player in seat 4, on the pad, picks up nothing;
        // nor does the person in seat 0 while dead, or high over it.
        assert!(game.gather(4, on_pad).is_empty());
        game.health[0] = 0.0;
        assert!(game.gather(0, on_pad).is_empty());
        game.health[0] = HEALTH;
        assert!(game.gather(0, on_pad + Vec3::Y * (WAITING_FLOOR - PAD_TOP)).is_empty());
        assert_eq!(game.mint.pieces().len(), lying);
        // Standing on it, they have all of it, and it is gone: whoever
        // comes after finds nothing there.
        let taken = game.gather(0, on_pad);
        assert_eq!(taken.len(), 12);
        assert_eq!(game.purses[0], [10, 2, 0, 0]);
        assert_eq!(game.mint.pieces().len(), lying - 12);
        game.bot[4] = false;
        assert!(game.gather(4, on_pad).is_empty());
        assert_eq!(game.purses[4], [0; 4]);
        // The shop sells a bridge for 4 silver to someone standing at it,
        // as often as they can pay, and not from the pad.
        let shop = ARENA_SHOPS[0];
        assert!(!game.buy(0, Ware::Bridge, pad));
        assert!(game.buy(0, Ware::Bridge, shop));
        assert!(game.buy(0, Ware::Bridge, shop));
        assert_eq!(game.purses[0][Item::Silver.index()], 2);
        assert!(!game.buy(0, Ware::Bridge, shop), "with 2 silver");
        assert_eq!(game.purses[0], [2, 2, 0, 0]);
        // Nor to the dead, nor to someone with nothing.
        assert!(!game.buy(4, Ware::Bridge, ARENA_SHOPS[0]));
        game.purses[0][0] = 4;
        game.health[0] = 0.0;
        assert!(!game.buy(0, Ware::Bridge, shop));
    }

    #[test]
    fn the_two_shops_sell_their_own_wares_and_dying_loses_the_sword() {
        use roundtown_net::{ARENA_SHOPS, ARENA_UPGRADE_SHOPS};
        let mut game = full_game();
        game.purses[0] = [60, 20, 20, 10];
        let (items, upgrades) = (ARENA_SHOPS[0], ARENA_UPGRADE_SHOPS[0]);
        // A sword at the item shop, not at the upgrade shop; an upgrade
        // the other way round.
        assert!(!game.buy(0, Ware::Sword(1), upgrades));
        assert!(game.buy(0, Ware::Sword(1), items));
        assert!(!game.buy(0, Ware::Forge, items));
        assert!(game.buy(0, Ware::Forge, upgrades));
        assert_eq!(game.purses[0], [50, 20, 18, 10]);
        assert_eq!((game.outfits.kits[0].sword, game.outfits.teams[0].forge), (1, 1));
        // Never the sword in hand again, nor a lesser one; a better one,
        // a shield, a fireball for the second button, and a potion beside
        // it, on the next (since 2026-11-13: the one button there was
        // took no potion with a fireball on it).
        assert!(!game.buy(0, Ware::Sword(1), items));
        assert!(game.buy(0, Ware::Sword(3), items) && game.buy(0, Ware::Shield(2), items));
        assert!(game.buy(0, Ware::Utility(Utility::Fireball), items));
        assert!(game.buy(0, Ware::Utility(Utility::Speed), items));
        assert_eq!(game.purses[0], [10, 8, 18, 5]);
        let kit = game.outfits.kit(0);
        assert_eq!((kit.slot_of(Utility::Fireball), kit.slot_of(Utility::Speed)), (Some(0), Some(1)));
        // The team's upgrade is its other seat's too: seat 4 swings a
        // stronger sword once it has one, and seat 0's punch takes what a
        // sword's does, less what a shield keeps off.
        assert!(game.buy(0, Ware::StrongerSwords, upgrades));
        assert_eq!(game.outfits.team(4), game.outfits.team(0));
        assert_eq!(game.outfits.punch(DAMAGE, 0, 1), DAMAGE + 20.0 + 5.0);
        assert_eq!(game.outfits.punch(DAMAGE, 4, 1), DAMAGE);
        assert!((game.outfits.punch(DAMAGE, 1, 0) - DAMAGE * 0.86).abs() < 1e-4);
        // Dead, the sword, the fireball and the potion are gone, and everything held
        // of what the generators make; the shield and the team's upgrades
        // are not, nor anyone else's purse.
        game.purses[4] = [3, 2, 1, 0];
        assert_ne!(game.purses[0], [0; 4]);
        game.die(0);
        assert_eq!(game.outfits.kits[0], Kit { shield: 2, ..Kit::default() });
        assert!(game.outfits.teams[0].swords && game.outfits.teams[0].forge == 1);
        assert_eq!((game.purses[0], game.purses[4]), ([0; 4], [3, 2, 1, 0]));
        // A potion drunk heals at once, or lasts its while.
        game.health[0] = 40.0;
        game.dead_for[0] = None;
        game.purses[0] = [0, 6, 0, 1];
        assert!(game.buy(0, Ware::Utility(Utility::Healing), items));
        assert_eq!(game.outfits.drink(0, Utility::Healing, game.elapsed), Some(Utility::Healing));
        assert!(game.buy(0, Ware::Utility(Utility::Healing), items), "the button is the healing's until it is ticked empty");
        // Unseen, seat 0 is seen again the moment it is hit.
        game.outfits.kits[0] = Kit { unseen_until: 600, ..Kit::default() };
        assert!(game.outfits.kit(0).unseen(game.elapsed));
        assert!(game.outfits.reveal(0) && !game.outfits.kit(0).unseen(game.elapsed));
    }

    /// What `game` has said for the chat since this was last asked.
    fn said(game: &mut Game) -> Vec<Feed> {
        let (feeds, rest): (Vec<Event>, Vec<Event>) =
            std::mem::take(&mut game.events).into_iter().partition(|event| matches!(event, Event::Feed(_)));
        game.events = rest;
        feeds
            .into_iter()
            .filter_map(|event| match event {
                Event::Feed(feed) => Some(feed),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_chat_is_told_who_killed_whom_with_what_who_fell_and_what_a_team_bought() {
        use roundtown_net::{ARENA_SHOPS, ARENA_UPGRADE_SHOPS};
        let mut game = full_game();
        let from = Spot::of(game.at[0], Quat::IDENTITY);
        // Killed by a punch: with a bare hand, and with the sword in hand.
        game.health[1] = 5.0;
        game.wound(1, 0, from, false);
        assert_eq!(said(&mut game), vec![Feed::Killed { seat: 1, by: 0, with: Weapon::Fists }]);
        game.outfits.kits[0].sword = 2;
        game.health[2] = 5.0;
        game.wound(2, 0, from, false);
        assert_eq!(said(&mut game), vec![Feed::Killed { seat: 2, by: 0, with: Weapon::Sword(2) }]);
        // By a fireball's blast, and by a stick of dynamite's.
        game.health[6] = 5.0;
        game.harm(6, 3, from, false, 20.0, Weapon::Fireball);
        game.health[7] = 5.0;
        assert!(!game.hurt_by(7, 3, 20.0, Weapon::Dynamite));
        assert_eq!(
            said(&mut game),
            vec![
                Feed::Killed { seat: 6, by: 3, with: Weapon::Fireball },
                Feed::Killed { seat: 7, by: 3, with: Weapon::Dynamite },
            ]
        );
        // Hit, and fallen within a few seconds: knocked off the world, by
        // whoever hit them. Fallen later than that, or never hit: fell.
        game.wound(3, 0, from, true);
        assert_eq!(said(&mut game), vec![]);
        game.elapsed += CREDIT_SECS - 0.5;
        game.fall(3);
        assert_eq!(said(&mut game), vec![Feed::Knocked { seat: 3, by: 0 }]);
        game.wound(5, 0, from, true);
        game.elapsed += CREDIT_SECS + 0.5;
        game.fall(5);
        game.fall(4);
        assert_eq!(said(&mut game), vec![Feed::Fell { seat: 5 }, Feed::Fell { seat: 4 }]);
        // Whoever hit them in the life before has nothing to do with a
        // fall in this one.
        game.dead_for[3] = None;
        game.health[3] = HEALTH;
        game.fall(3);
        assert_eq!(said(&mut game), vec![Feed::Fell { seat: 3 }]);
        // What is the whole team's, the team is told of, with how far it
        // has it now; what is one's own is nobody's to be told.
        let (items, upgrades) = (ARENA_SHOPS[0], ARENA_UPGRADE_SHOPS[0]);
        game.purses[0] = [60, 20, 40, 10];
        assert!(game.buy(0, Ware::Sword(3), items) && game.buy(0, Ware::Utility(Utility::Leap), items));
        assert_eq!(said(&mut game), vec![]);
        assert!(game.buy(0, Ware::HealPool, upgrades));
        assert!(game.buy(0, Ware::BuildSpeed, upgrades) && game.buy(0, Ware::BuildSpeed, upgrades));
        let told = said(&mut game);
        assert_eq!(
            told,
            vec![
                Feed::Bought { by: 0, ware: Ware::HealPool, step: 1 },
                Feed::Bought { by: 0, ware: Ware::BuildSpeed, step: 1 },
                Feed::Bought { by: 0, ware: Ware::BuildSpeed, step: 2 },
            ]
        );
        assert!(told.iter().all(|feed| feed.only_for() == Some(arena_team(0))));
        assert_eq!(game.purses[0][2], 40 - 1 - 4 - 8);
        assert_eq!(Feed::Fell { seat: 1 }.only_for(), None);
        // An anchor broken by someone's punches is said, to everyone.
        for _ in 0..20 {
            game.anchor_hurt = [0.0; ARENA_TEAMS];
            game.strike_anchor(0, 1, ARENA_ANCHORS[1], REACH);
        }
        assert_eq!(game.anchors[1], 0.0);
        assert_eq!(said(&mut game), vec![Feed::AnchorBroken { team: 1, by: 0 }]);
        // A blow at a zombie is nothing to say, nothing hurting one; a
        // zombie that kills is.
        risen(&mut game, 2);
        let zombie = ARENA_SEATS;
        game.health[zombie] = 5.0;
        game.wound(zombie, 0, from, false);
        assert_eq!((said(&mut game), game.health[zombie], game.alive(zombie)), (vec![], 5.0, true));
        game.health[0] = 5.0;
        game.wound(0, zombie + 1, from, false);
        assert_eq!(said(&mut game), vec![Feed::Killed { seat: 0, by: (zombie + 1) as u8, with: Weapon::Fists }]);
    }

    #[test]
    fn whoever_kills_someone_has_their_sapphire_and_amethyst_and_nothing_else_of_theirs() {
        let mut game = full_game();
        // Everyone out over the void, between an island and the middle,
        // on nothing: no death here is one on land, where what the dead
        // held is dropped (the test after this one).
        let void = Vec3::new(0.0, PLANK_TOP, 100.0);
        assert!(game.nav.ground(void.xz()).is_none());
        for at in game.at.iter_mut() {
            *at = void;
        }
        let from = Spot::of(game.at[0], Quat::IDENTITY);
        // Seats 0 and 4 are one team, and 1 another.
        assert!(arena_team(0) == arena_team(4) && arena_team(0) != arena_team(1));
        // Killed with a hand: every sapphire and every amethyst the dead
        // held is the killer's, on top of the killer's own; the silver
        // and the gold go with the dead, as everything did.
        game.purses[0] = [1, 2, 3, 4];
        game.purses[1] = [30, 20, 7, 5];
        game.health[1] = 5.0;
        game.wound(1, 0, from, false);
        assert!(!game.alive(1));
        assert_eq!((game.purses[0], game.purses[1]), ([1, 2, 10, 9], [0, 0, 0, 0]));
        // Knocked off the world: whoever hit them last has it, as the
        // chat has it their doing.
        game.purses[2] = [9, 9, 2, 1];
        game.wound(2, 0, from, true);
        game.elapsed += CREDIT_SECS - 0.5;
        game.fall(2);
        assert_eq!((game.purses[0], game.purses[2]), ([1, 2, 12, 10], [0, 0, 0, 0]));
        // By a blast, a fireball's or a stick of dynamite's, as by a hand.
        game.purses[3] = [0, 0, 1, 1];
        game.health[3] = 5.0;
        assert!(!game.hurt_by(3, 0, 20.0, Weapon::Dynamite));
        assert_eq!(game.purses[0], [1, 2, 13, 11]);
        // Fallen with nobody's hand in it: nobody has it. It is gone.
        game.purses[5] = [4, 4, 4, 4];
        game.fall(5);
        assert_eq!((game.purses[0], game.purses[5]), ([1, 2, 13, 11], [0, 0, 0, 0]));
        // A teammate caught in their own team's blast leaves nothing to
        // whoever threw it.
        game.purses[4] = [0, 0, 6, 6];
        game.health[4] = 5.0;
        assert!(!game.hurt_by(4, 0, 20.0, Weapon::Dynamite));
        assert_eq!((game.purses[0], game.purses[4]), ([1, 2, 13, 11], [0, 0, 0, 0]));
        // The killer dies in their turn, and whoever kills them has all
        // of it: what they came with and what they took.
        game.health[0] = 5.0;
        game.wound(0, 6, from, false);
        assert_eq!((game.purses[6], game.purses[0]), ([0, 0, 13, 11], [0, 0, 0, 0]));
        // A zombie takes nothing off whoever it kills, and has nothing to
        // be taken.
        risen(&mut game, 2);
        let zombie = ARENA_SEATS;
        game.purses[7] = [0, 0, 3, 3];
        game.health[7] = 5.0;
        game.wound(7, zombie, from, false);
        assert_eq!(game.purses[7], [0, 0, 0, 0]);
        game.health[zombie + 1] = 5.0;
        game.wound(zombie + 1, 6, from, false);
        assert_eq!(game.purses[6], [0, 0, 13, 11]);
        // Never more than a purse holds.
        game.dead_for[7] = None;
        game.health[7] = 5.0;
        game.purses[7] = [0, 0, u16::MAX - 5, 2];
        assert!(!game.hurt_by(7, 6, 20.0, Weapon::Fists));
        assert_eq!(game.purses[6], [0, 0, u16::MAX, 13]);
    }

    #[test]
    fn whoever_is_killed_on_land_drops_their_sapphire_and_amethyst_for_whoever_walks_into_them() {
        use roundtown_net::nav::Region;
        use roundtown_net::{Item, Loot};
        let mut game = full_game();
        let home = |seat: usize| {
            let (at, _) = arrival(Venue::Arena, seat);
            at.with_y(ISLAND_TOP)
        };
        for seat in 0..ARENA_SEATS {
            game.at[seat] = home(seat);
        }
        let made = |game: &mut Game| -> Vec<Piece> {
            let (made, rest): (Vec<Event>, Vec<Event>) = std::mem::take(&mut game.events).into_iter().partition(|event| matches!(event, Event::Made(_)));
            game.events = rest;
            made.into_iter()
                .filter_map(|event| match event {
                    Event::Made(piece) => Some(piece),
                    _ => None,
                })
                .collect()
        };
        let on = |piece: &Piece| Vec3::new(piece.at.x, piece.ground, piece.at.y);
        // Seat 0, a person, kills seat 1 with a hand where it stands on
        // its island: nothing of what it held is the killer's. Its
        // sapphire and its amethyst lie round it, a thing for each, on
        // the island, and the room is sent each as one that was dropped,
        // from the middle of the dead; its silver and its gold go with
        // it.
        let stood = home(1);
        game.at[0] = stood + Vec3::new(1.2, 0.0, 0.0);
        let from = Spot::of(game.at[0], Quat::IDENTITY);
        game.purses[0] = [1, 2, 3, 4];
        game.purses[1] = [30, 20, 7, 5];
        game.health[1] = 5.0;
        game.wound(1, 0, from, false);
        assert!(!game.alive(1));
        assert_eq!((game.purses[0], game.purses[1]), ([1, 2, 3, 4], [0, 0, 0, 0]));
        assert_eq!(said(&mut game), vec![Feed::Killed { seat: 1, by: 0, with: Weapon::Fists }]);
        let dropped = made(&mut game);
        assert_eq!(dropped.len(), 12);
        for piece in &dropped {
            assert!(piece.dropped() && piece.worth == 1 && game.nav.stays_at(piece.at), "{piece:?}");
            assert!(game.nav.grid().region(piece.at) == Some(Region::Team(arena_team(1))), "{piece:?}");
            let far = piece.at.distance(stood.xz());
            assert!((SPILL_NEAR - 1e-3..=SPILL_FAR + 1e-3).contains(&far), "{far} m from the dead");
            let sent = Loot::of(piece).spilt.expect("sent as one that was dropped");
            assert_eq!((sent.ground, sent.worth), (piece.ground, 1));
            assert!(Vec3::new(sent.from_x, sent.from_y, sent.from_z).distance(stood + Vec3::Y * SPILL_FROM) < 1e-4);
        }
        // Nobody has any of it while it is in the air, nor for a moment
        // after it has come down, so that whoever killed them, standing
        // over it, sees it lie; after that it is whoever's walks into it,
        // a thing at a time, and gone from the room.
        assert!(game.gather(0, on(&dropped[0])).is_empty());
        game.elapsed += TOSS_SECS + 0.05;
        assert!(game.gather(0, on(&dropped[0])).is_empty(), "down, and not to be had yet");
        game.elapsed -= TOSS_SECS + 0.05;
        game.elapsed += SPILL_WAIT + 0.01;
        let mut taken = 0;
        for piece in &dropped {
            taken += game.gather(0, on(piece)).len();
        }
        assert_eq!((taken, game.purses[0]), (12, [1, 2, 10, 9]));
        assert!(game.mint.pieces().iter().all(|piece| !piece.dropped()));
        // One of the computer's players takes what was dropped as it
        // comes by, whatever it was saving for, and so may the dead's own
        // teammate: seat 6 is seat 2's, killed by the person in seat 0.
        assert_eq!(arena_team(2), arena_team(6));
        game.at[0] = home(2) + Vec3::new(1.2, 0.0, 0.0);
        game.purses[2] = [0, 0, 2, 0];
        game.purses[6] = [0, 0, 0, 0];
        game.health[2] = 5.0;
        game.wound(2, 0, Spot::of(game.at[0], Quat::IDENTITY), false);
        let dropped = made(&mut game);
        assert_eq!((dropped.len(), game.purses[0]), (2, [1, 2, 10, 9]));
        game.elapsed += SPILL_WAIT + 0.01;
        for piece in &dropped {
            game.at[6] = on(piece);
            game.squad.stand(6, game.at[6], 0.0, 0.0);
            game.mint_step();
        }
        assert_eq!((game.purses[6][Item::Sapphire.index()], game.purses[0]), (2, [1, 2, 10, 9]));
        assert!(game.mint.pieces().iter().all(|piece| !piece.dropped()));
        game.events.clear();
        // More than ten of one: ten things, worth all of it between them.
        game.dead_for[1] = None;
        (game.health[1], game.hurt[1]) = (5.0, 0.0);
        game.purses[1] = [0, 0, 25, 0];
        game.wound(1, 0, from, false);
        let dropped = made(&mut game);
        assert_eq!((dropped.len(), dropped.iter().map(|piece| piece.worth).sum::<u16>()), (10, 25));
        game.elapsed += SPILL_WAIT + 0.01;
        for piece in &dropped {
            game.gather(0, on(piece));
        }
        assert_eq!(game.purses[0], [1, 2, 35, 9]);
        // Killed by a zombie, or by their own team's blast, they drop it
        // all the same: it is a death on land.
        risen(&mut game, 1);
        game.events.clear();
        game.purses[3] = [0, 0, 1, 2];
        game.health[3] = 5.0;
        game.wound(3, ARENA_SEATS, Spot::of(game.at[3], Quat::IDENTITY), false);
        assert_eq!((made(&mut game).len(), game.purses[3]), (3, [0, 0, 0, 0]));
        game.purses[4] = [0, 0, 6, 0];
        game.health[4] = 5.0;
        assert!(!game.hurt_by(4, 0, 20.0, Weapon::Dynamite));
        assert_eq!((made(&mut game).len(), game.purses[4], game.purses[0]), (6, [0, 0, 0, 0], [1, 2, 35, 9]));
        // A zombie drops nothing, holding nothing.
        game.health[ARENA_SEATS] = 5.0;
        game.wound(ARENA_SEATS, 0, from, false);
        assert!(made(&mut game).is_empty());

        // On a bridge: onto its planks, and nothing off its sides.
        let span = game.nav.grid().span(Region::Team(0), Region::Middle, 0.0).expect("a span");
        for index in 0..=span.planks() {
            game.nav.hear(4, 0, index as u16, span.rung(index));
        }
        let along = |share: f32| {
            let at = span.point(share);
            Vec3::new(at.x, PLANK_TOP, at.y)
        };
        game.at[5] = along(0.5);
        game.purses[5] = [0, 0, 9, 9];
        game.health[5] = 5.0;
        game.wound(5, 0, from, false);
        let on_bridge = made(&mut game);
        assert_eq!((on_bridge.len(), game.purses[5]), (18, [0, 0, 0, 0]));
        assert!(on_bridge.iter().all(|piece| game.nav.on_plank(piece.at) && piece.ground == PLANK_TOP), "{on_bridge:?}");
        // (And another further along it, toward the middle.)
        game.at[7] = along(0.8);
        game.purses[7] = [0, 0, 2, 0];
        game.health[7] = 5.0;
        game.wound(7, 0, from, false);
        assert_eq!(made(&mut game).len(), 2);
        // A stick of dynamite on that bridge takes the planks under it,
        // and what lay on them goes with them at once: the room is no
        // longer sent those. Whoever it kills on the bridge has died with
        // nothing under them that is staying, and what they held is the
        // thrower's, as with a fall.
        game.dead_for[1] = None;
        game.health[1] = 5.0;
        game.at[1] = along(0.5) + Vec3::new(0.4, 0.0, 0.0);
        game.present[1] = true;
        game.purses[1] = [0, 0, 4, 1];
        game.purses[2] = [0, 0, 0, 0];
        game.dead_for[2] = None;
        game.health[2] = HEALTH;
        game.outfits.kits[2] = Kit::default().holding(Utility::Dynamite, 1);
        game.at[2] = along(0.5) + Vec3::new(0.0, 0.0, 8.0);
        let lying = game.mint.pieces().iter().filter(|piece| piece.dropped()).count();
        assert!(game.lob(2, game.at[2], 0.0, 8.0));
        game.events.clear();
        while !game.sticks.is_empty() {
            game.dynamites(1.0 / 60.0);
        }
        assert!(!game.alive(1) && game.blasts == 1);
        assert_eq!((game.purses[1], game.purses[2]), ([0, 0, 0, 0], [0, 0, 4, 1]));
        assert!(made(&mut game).is_empty(), "nothing is dropped onto a bridge that is coming down");
        let went = game.events.iter().filter(|event| matches!(event, Event::Taken(_))).count();
        let left = game.mint.pieces().iter().filter(|piece| piece.dropped()).count();
        assert!(went > 0 && left == lying - went, "{went} of {lying} went with the planks, {left} lie");
        assert!(game.mint.pieces().iter().filter(|piece| piece.dropped()).all(|piece| game.nav.ground(piece.at).is_some()));
        // Killed by a hand on what is left of it, which is going: the
        // same.
        game.dead_for[5] = None;
        (game.health[5], game.hurt[5]) = (5.0, 0.0);
        game.at[5] = along(0.85);
        assert!(game.nav.walkable_at(game.at[5].xz()) && !game.nav.stays_at(game.at[5].xz()));
        game.purses[5] = [0, 0, 1, 1];
        game.wound(5, 0, from, false);
        assert_eq!((made(&mut game).len(), game.purses[0]), (0, [1, 2, 36, 10]));
        // And the rest of what was dropped on it goes as the planks it
        // lies on fall: by the last of them none of it is left, and
        // whatever lies on the islands is.
        game.events.clear();
        while game.nav.crumbling() {
            game.crumble(1.0 / 30.0);
        }
        let fell = game.events.iter().filter(|event| matches!(event, Event::Taken(_))).count();
        assert!(fell >= 2 && fell == left - 9, "{fell} fell with the planks, of {left}");
        assert_eq!(game.mint.pieces().iter().filter(|piece| piece.dropped()).count(), 9);
    }

    #[test]
    fn the_zombies_come_a_few_and_then_more_sit_past_the_players_and_nothing_hurts_one() {
        let mut game = full_game();
        // None as sudden death is said; as many as the rules have by each
        // moment after, each in the next seat past the players', standing
        // where it rises, with a body asked of the world for it.
        game.happen(Happening::SuddenDeath);
        game.raise_zombies();
        assert_eq!((game.horde.len(), game.seats.len()), (0, ARENA_SEATS));
        for (elapsed, risen) in [(SUDDEN_DEATH, 2), (SUDDEN_DEATH + 36.0, 4), (SUDDEN_DEATH + 72.0, 8), (PLAY_SECS, ZOMBIES)] {
            game.elapsed = elapsed;
            game.raise_zombies();
            assert_eq!((game.horde.len(), game.seats.len()), (risen, ARENA_SEATS + risen), "{elapsed} s in");
        }
        game.elapsed = 0.0;
        for zombie in 0..ZOMBIES {
            let seat = ARENA_SEATS + zombie;
            assert_eq!(game.zombie(seat), Some(zombie));
            assert_eq!(arena_team(seat), ZOMBIE_TEAM);
            assert!(game.at[seat].xz().distance(zombie_spot(zombie)) < 0.01);
            assert_eq!(Some(game.at[seat].y), game.nav.ground(zombie_spot(zombie)));
            assert!(game.bot[seat] && !game.brained(seat) && game.alive(seat) && game.members[seat].name == ZOMBIE_NAME);
            assert_eq!(game.members[seat].id, bot_id(game.id, seat));
        }
        let bodies: Vec<usize> = game
            .events
            .iter()
            .filter_map(|event| match event {
                Event::Zombie(seat) => Some(*seat),
                _ => None,
            })
            .collect();
        assert_eq!(bodies, (ARENA_SEATS..ARENA_SEATS + ZOMBIES).collect::<Vec<_>>());
        assert_eq!(game.purses.len(), game.seats.len());
        // Every id in the game its own, and none another game's.
        assert!(game.seats.len() as u64 <= MOST_SEATS);
        let mut ids: Vec<u64> = (0..game.seats.len()).map(|seat| bot_id(game.id, seat)).collect();
        ids.extend((0..game.seats.len()).map(|seat| bot_id(game.id + 1, seat)));
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 2 * game.seats.len());
        // They are nobody's team: the four teams are still what is in
        // play.
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
        // Nothing hurts one: not a blow, not a blast, not a bus, which
        // lays it flat and no more.
        let zombie = ARENA_SEATS + 5;
        let from = Spot::of(game.at[0], Quat::IDENTITY);
        game.wound(zombie, 0, from, true);
        assert!(game.hurt_by(zombie, 0, 1000.0, Weapon::Dynamite));
        game.run_over(zombie);
        assert_eq!((game.health[zombie], game.alive(zombie)), (HEALTH, true));
        assert!(game.flat[zombie] > 0.0);
        assert_eq!(said(&mut game), vec![]);
        // Gone into the void, it is back after its while, whatever the
        // anchors, where it first came up.
        game.anchors = [0.0; ARENA_TEAMS];
        game.fall(zombie);
        assert_eq!(game.dead_for[zombie], Some(ZOMBIE_RESPAWN_SECS));
        assert!(!game.out[zombie] && game.in_play(zombie) && !game.alive(zombie));
        for seat in 0..ARENA_SEATS {
            game.die(seat);
        }
        assert!(game.teams_in_play().is_empty());
        let round = game.round();
        assert_eq!(round.out, 0xff);
        assert_eq!(round.health.len(), ARENA_SEATS + ZOMBIES);

        // The ninth rises on team 0's island, and goes for whoever is
        // there: at a zombie's pace, slower than anyone runs, never off
        // the ground, a blow of twenty at a time once they are in its
        // reach, until they are dead of it, which the chat says.
        let mut game = full_game();
        risen(&mut game, 9);
        let seat = ARENA_SEATS + 8;
        assert_eq!(zombie_home(8).0, ARENA_ISLANDS[0]);
        let from = game.at[seat].xz();
        let prey = ARENA_ISLANDS[0] + (ARENA_ISLANDS[0] - from).normalize() * 3.5;
        assert!(game.nav.walkable_at(prey));
        for other in 0..ARENA_SEATS {
            game.at[other] = Vec3::new(500.0, ISLAND_TOP, 500.0 + other as f32 * 50.0);
        }
        game.at[0] = Vec3::new(prey.x, ISLAND_TOP, prey.y);
        game.events.clear();
        let (dt, mut last, mut hits) = (1.0 / 60.0, from, Vec::new());
        for frame in 0..900 {
            let before = game.health[0];
            game.fight(dt);
            game.hurt.iter_mut().for_each(|hurt| *hurt = (*hurt - dt).max(0.0));
            let at = game.at[seat].xz();
            assert!(at.distance(last) <= ZOMBIE_SPEED * dt + 1e-3, "frame {frame}: a step of {} m", at.distance(last));
            assert!(game.nav.ground(at).is_some(), "frame {frame}: off the ground at {at}");
            last = at;
            if game.health[0] < before {
                hits.push(before - game.health[0]);
            }
            if !game.alive(0) {
                break;
            }
        }
        assert_eq!(hits, vec![ZOMBIE_DAMAGE; 5], "what its blows took");
        assert!(!game.alive(0) && game.health[seat] == HEALTH);
        assert!(said(&mut game).contains(&Feed::Killed { seat: 0, by: seat as u8, with: Weapon::Fists }));
    }

    #[test]
    fn at_fourteen_minutes_every_anchor_breaks_and_whoever_dies_after_is_out() {
        let mut game = full_game();
        game.anchors[2] = 0.0;
        game.happen(Happening::Tier {
            item: roundtown_net::Item::Sapphire,
            tier: 2,
        });
        assert_eq!(game.anchors, [ANCHOR_HEALTH, ANCHOR_HEALTH, 0.0, ANCHOR_HEALTH]);
        game.happen(Happening::AnchorsBreak);
        assert_eq!(game.anchors, [0.0; ARENA_TEAMS]);
        game.die(1);
        assert!(game.out[1]);
    }

    #[test]
    fn an_anchor_takes_a_punch_from_another_team_within_reach_and_no_oftener_than_a_body() {
        use roundtown_net::ANCHOR_RADIUS;
        let mut game = full_game();
        let anchor = ARENA_ANCHORS[1];
        let beside = anchor + Vec2::new(1.5, 0.0);
        // Not one's own, nor from out of reach, nor a team there is none
        // of, nor by the dead.
        assert!(!game.strike_anchor(1, 1, beside, PUNCH_SLACK));
        assert!(!game.strike_anchor(5, 1, beside, PUNCH_SLACK));
        assert!(!game.strike_anchor(0, 1, anchor + Vec2::new(PUNCH_SLACK + ANCHOR_RADIUS + 0.5, 0.0), PUNCH_SLACK));
        assert!(!game.strike_anchor(0, 9, beside, PUNCH_SLACK));
        game.health[2] = 0.0;
        assert!(!game.strike_anchor(2, 1, beside, PUNCH_SLACK));
        assert_eq!(game.anchors[1], ANCHOR_HEALTH);
        // A punch's damage off it, and nothing while it reels from one.
        assert!(game.strike_anchor(0, 1, beside, PUNCH_SLACK));
        assert_eq!(game.anchors[1], ANCHOR_HEALTH - DAMAGE);
        assert!(!game.strike_anchor(0, 1, beside, PUNCH_SLACK));
        assert_eq!(game.anchors[1], ANCHOR_HEALTH - DAMAGE);
        game.anchor_hurt[1] = 0.0;
        // A sword breaks it sooner, as it does a body; and whoever was
        // unseen is seen again for hitting it.
        game.outfits.kits[0] = Kit {
            sword: 2,
            unseen_until: 900,
            ..Kit::default()
        };
        assert!(game.strike_anchor(0, 1, beside, PUNCH_SLACK));
        assert_eq!(game.anchors[1], ANCHOR_HEALTH - 2.0 * DAMAGE - SWORD_DAMAGE[2]);
        assert!(!game.outfits.kit(0).unseen(game.elapsed));
        // Broken, it takes no more.
        game.anchors[1] = 0.0;
        game.anchor_hurt[1] = 0.0;
        assert!(!game.strike_anchor(0, 1, beside, PUNCH_SLACK));
    }

    #[test]
    fn a_fireball_thrown_flies_and_bursts_on_whoever_it_meets_and_the_room_is_shown_both() {
        let mut game = full_game();
        // Seat 0 on its island with one fireball; seat 1, a bot of
        // another team, 6 m ahead of it the way it throws.
        game.outfits.kits[0] = Kit::default().holding(Utility::Fireball, 1);
        let from = Vec3::new(0.0, ISLAND_TOP, 130.0);
        game.at[0] = from;
        game.at[1] = from + Vec3::new(0.0, 0.0, -6.0);
        assert!(game.throw(0, from, 0.0));
        assert!(!game.throw(0, from, 0.0), "with none left");
        assert!(!game.throw(1, game.at[1], 0.0), "with none at all");
        assert!(matches!(game.events.as_slice(), [Event::Thrown { ball: 0, shown }] if shown.seat == 0));
        game.events.clear();
        let mut frames = 0;
        while !game.balls.is_empty() {
            game.fireballs(1.0 / 60.0);
            frames += 1;
            assert!(frames < 120, "still flying");
        }
        // It met them well short of its range, took its damage and
        // knocked them on the way it flew; a teammate of the thrower's
        // beside them would have taken nothing.
        assert!(frames < 20, "{frames} frames");
        assert_eq!(game.health[1], HEALTH - FIREBALL_DAMAGE);
        let knocked = game.squad.bot(1).expect("the computer's").body;
        assert!(knocked.shove.y < -1.0 && knocked.lift > 0.0, "{knocked:?}");
        assert!(matches!(game.events.as_slice(), [Event::Burst(0)]));
        assert_eq!(game.health[4], HEALTH);
    }

    #[test]
    fn a_stick_of_dynamite_comes_down_blows_hurts_other_teams_and_takes_the_planks_under_it() {
        use roundtown_net::nav::{PLANK_TOP, Region};
        let mut game = full_game();
        // A bridge from seat 0's island to the middle, its teammate's.
        let span = game.nav.grid().span(Region::Team(0), Region::Middle, 0.0).expect("a span");
        for index in 0..=span.planks() {
            game.nav.hear(4, 0, index as u16, span.rung(index));
        }
        assert!(game.nav.joined(Region::Team(0), Region::Middle));
        // Seat 0 on the island at the bridge's end, with two sticks; the
        // bridge runs north from there. Seat 1, a bot of another team
        // with the best shield there is, stands on it 8 m out, and seat
        // 4, of the thrower's own team, beside it; seat 2 well off.
        game.outfits.kits[0] = Kit::default().holding(Utility::Dynamite, 2);
        game.outfits.kits[1].shield = 3;
        let from = Vec3::new(span.from.x, ISLAND_TOP, span.from.y + 1.0);
        let down = Vec3::new(from.x, PLANK_TOP, from.z - 8.0);
        game.at[0] = from;
        game.at[2] = down + Vec3::new(0.0, 0.0, -12.0);
        for seat in [1, 4] {
            game.at[seat] = down + Vec3::new(0.3, 0.0, -1.0);
            game.squad.stand(seat, game.at[seat], 0.0, 0.0);
        }
        assert!(!game.lob(1, game.at[1], 0.0, 8.0), "with none at all");
        game.clock = 1_794_000_000.5;
        assert!(game.lob(0, from, 0.0, 8.0));
        let [Event::Lobbed { stick: 0, shown }] = game.events.as_slice() else {
            panic!("the room is not shown it: {:?}", game.events);
        };
        assert_eq!(shown.seat, 0);
        assert!(Vec3::new(shown.to_x, shown.to_y, shown.to_z).distance(down) < 1e-3, "{shown:?}");
        assert!(Vec3::new(shown.x, shown.y, shown.z).distance(from + Vec3::new(0.0, DYNAMITE_HAND, -DYNAMITE_AHEAD)) < 1e-3);
        game.events.clear();
        assert_eq!(game.outfits.kit(0).held(Utility::Dynamite), 1);
        // It is in the air for most of a second, lies for its fuse, and
        // blows: nothing happens to anyone until then.
        let mut seconds = 0.0;
        while !game.sticks.is_empty() {
            assert_eq!((game.health[1], game.blasts), (HEALTH, 0));
            assert!(game.events.is_empty() && game.nav.joined(Region::Team(0), Region::Middle));
            game.dynamites(1.0 / 60.0);
            seconds += 1.0 / 60.0;
            assert!(seconds < 4.0, "it never blew");
        }
        assert!((seconds - (0.45 + 0.04 * 8.0 + DYNAMITE_FUSE)).abs() < 0.05, "{seconds} s");
        // Its damage off the one of another team within its reach, less
        // what their shield keeps off, and nobody knocked; nothing off the
        // thrower's teammate beside them, nor off anyone further away.
        assert!((game.health[1] - (HEALTH - DYNAMITE_DAMAGE * (1.0 - SHIELD_SHARE[3]))).abs() < 1e-3, "{}", game.health[1]);
        assert_eq!((game.health[4], game.health[2], game.health[0]), (HEALTH, HEALTH, HEALTH));
        let body = game.squad.bot(1).expect("the computer's").body;
        assert_eq!((body.shove, body.lift), (Vec2::ZERO, 0.0), "a stick of dynamite knocks nobody");
        // The planks whose middles it reached are gone, one after another
        // of that one bridge, and the room is told which, with where it
        // blew; the island is parted from the middle again.
        let [Event::Blew { stick: 0, shown }] = game.events.as_slice() else {
            panic!("the room is not shown the blast: {:?}", game.events);
        };
        assert!(Vec3::new(shown.x, shown.y, shown.z).distance(down) < 1e-3);
        assert!((4..=5).contains(&shown.planks.len()), "{:?}", shown.planks);
        assert!(shown.planks.iter().all(|&(seat, chain, _)| (seat, chain) == (4, 0)));
        assert!(shown.planks.windows(2).all(|pair| pair[1].2 == pair[0].2 + 1));
        assert_eq!((game.blasts, game.planks_blown), (1, shown.planks.len() as u32));
        assert!(!game.nav.joined(Region::Team(0), Region::Middle));
        assert_eq!(game.nav.ground(down.xz()), None);
        // And the room is told, with when it blew by the server's clock,
        // which of the rest of the bridge is to fall after them and how
        // long after, the nearest the hole first: every other plank of
        // it, none of which has gone yet.
        let (took, later) = (shown.planks.len(), shown.later.clone());
        assert_eq!(shown.at, 1_794_000_000.5);
        assert_eq!(took + later.len(), span.planks(), "{later:?}");
        assert!(later.iter().all(|&(seat, chain, plank, _)| (seat, chain) == (4, 0) && !shown.planks.contains(&(seat, chain, plank))));
        assert!(later.windows(2).all(|pair| pair[0].3 <= pair[1].3));
        assert!((f32::from(later[0].3) / 1000.0 - cascade_after(span.length() / span.planks() as f32)).abs() < 0.03);
        assert_eq!(game.nav.planks().count(), later.len());
        game.events.clear();
        // Whoever of the computer's stood on them falls with them, and is
        // dead of it soon enough.
        for _ in 0..150 {
            game.step(1.0 / 60.0);
        }
        assert!(!game.alive(1) && !game.alive(4), "they stand on nothing still: {} {}", game.at[1], game.at[4]);
        // Two and a half seconds on, the planks whose time has come are
        // gone, and no others; nobody is told again.
        let due = later.iter().filter(|plank| f32::from(plank.3) / 1000.0 <= 2.5).count();
        assert!(due >= 6 && due < later.len(), "{due} of {}", later.len());
        assert!(game.nav.planks().count().abs_diff(later.len() - due) <= 1, "{} stand", game.nav.planks().count());
        assert_eq!(game.planks_blown as usize, took + later.len() - game.nav.planks().count());
        assert!(game.events.iter().all(|event| !matches!(event, Event::Blew { .. })), "{:?}", game.events);
        // And by the last of their times nothing of the bridge is left.
        let longest = f32::from(later[later.len() - 1].3) / 1000.0;
        for _ in 0..((longest - 2.5 + 0.2) * 60.0) as usize {
            game.step(1.0 / 60.0);
        }
        assert_eq!(game.nav.planks().count(), 0);
        assert_eq!(game.planks_blown as usize, span.planks());
        game.events.clear();
        // One thrown where there is nothing to lie on is gone into the
        // void as it comes down, and nothing blows.
        let west = std::f32::consts::FRAC_PI_2;
        assert!(game.lob(0, from, west, 9.0));
        game.events.clear();
        while !game.sticks.is_empty() {
            game.dynamites(1.0 / 60.0);
        }
        assert!(matches!(game.events.as_slice(), [Event::Fell(1)]), "{:?}", game.events);
        assert_eq!((game.blasts, game.outfits.kit(0).held(Utility::Dynamite)), (1, 0));
        assert!(!game.lob(0, from, 0.0, 5.0), "with none left");
        // And one thrown onto the land blows there and takes no plank.
        game.outfits.kits[0] = Kit::default().holding(Utility::Dynamite, 1);
        assert!(game.lob(0, from, std::f32::consts::PI, 5.0));
        game.events.clear();
        while !game.sticks.is_empty() {
            game.dynamites(1.0 / 60.0);
        }
        assert!(matches!(game.events.as_slice(), [Event::Blew { stick: 2, shown }] if shown.planks.is_empty()), "{:?}", game.events);
    }

    /// Plays `game` on by itself for `secs` seconds, or until it is over,
    /// a thirtieth of a second at a time, with nobody to show or tell
    /// what it says is to be.
    fn play(game: &mut Game, secs: f32) {
        let dt = 1.0 / 30.0;
        let until = game.elapsed + secs;
        while game.playing() && game.elapsed < until {
            game.step(dt);
            game.events.clear();
        }
    }

    /// A game of eight of the computer's players, on their islands, the
    /// fight just begun: those that play the game, or, `old`, those kept
    /// to their islands.
    fn bots_alone(seed: u64, old: bool) -> Game {
        let mut game = Game::new(3, Entity::PLACEHOLDER, seed);
        game.old_bots = old;
        fill_with_bots(&mut game);
        for seat in 0..ARENA_SEATS {
            game.stand(seat);
        }
        game.stage = ArenaStage::Playing;
        game.left = PLAY_SECS;
        game
    }

    #[test]
    fn a_whole_game_of_the_bots_as_they_were_plays_itself_out() {
        let mut game = bots_alone(11, true);
        assert_eq!((game.people(), game.seats.len()), (0, ARENA_SEATS));
        // Just short of fourteen minutes: every anchor stands, and
        // everyone is where they can be.
        play(&mut game, ANCHORS_BREAK - 1.0);
        assert!(game.playing());
        assert_eq!(game.anchors, [ANCHOR_HEALTH; ARENA_TEAMS]);
        assert!(game.at.iter().all(|at| at.is_finite()));
        assert!(!game.mint.pieces().is_empty());
        assert_eq!(game.seats.len(), ARENA_SEATS);
        // Past fourteen the anchors have broken, and past fifteen the
        // zombies have begun to come: as many as the rules have by then.
        play(&mut game, 70.0);
        assert_eq!(game.anchors, [0.0; ARENA_TEAMS]);
        assert!(game.elapsed > SUDDEN_DEATH && game.horde.len() >= 2);
        assert_eq!(game.seats.len(), ARENA_SEATS + zombies_at(game.elapsed));
        // And it is over by its time at the latest, with a winner or a
        // draw.
        play(&mut game, PLAY_SECS);
        assert!(game.over(), "still {:?} at {} s", game.stage, game.elapsed);
        assert!(game.elapsed <= PLAY_SECS + 0.1);
        assert!(game.at.iter().all(|at| at.is_finite()));
        // The same dice, the same game.
        let (mut one, mut other) = (bots_alone(5, true), bots_alone(5, true));
        play(&mut one, 60.0);
        play(&mut other, 60.0);
        assert_eq!(one.at, other.at);
        assert_eq!(one.mint.pieces(), other.mint.pieces());
    }

    /// What everyone in `game` is up to, for looking at one played in a
    /// test.
    fn report(game: &Game) -> String {
        use roundtown_net::nav::Region;
        let joined: Vec<String> = Region::LANDS
            .iter()
            .filter(|&&land| land != Region::Middle && game.nav.joined(land, Region::Middle))
            .map(|land| format!("{land:?}"))
            .collect();
        let mut lines = vec![format!(
            "t={:>4.0} anchors {:?} joined to the middle: {}",
            game.elapsed,
            game.anchors.map(|anchor| anchor as i32),
            joined.join(" ")
        )];
        for seat in 0..game.seats.len().min(ARENA_SEATS) {
            let Some(bot) = game.squad.bot(seat) else {
                continue;
            };
            let (at, kit) = (game.at[seat], game.outfits.kit(seat));
            let state = if game.out[seat] {
                "OUT "
            } else if game.alive(seat) {
                "    "
            } else {
                "dead"
            };
            lines.push(format!(
                "  seat {seat} team {} {:<6} hp {:>3.0} {state} ({:>6.1},{:>5.1},{:>6.1}) {:<14} purse {:?} sword {} shield {} buttons {} {:>3.0}m  {:?}",
                arena_team(seat),
                format!("{:?}", bot.role()),
                game.health[seat],
                at.x,
                at.y,
                at.z,
                game.nav.grid().region(at.xz()).map_or(String::from("-"), |land| format!("{land:?}")),
                game.purses[seat],
                kit.sword,
                kit.shield,
                kit.slots
                    .iter()
                    .flatten()
                    .map(|stack| format!("{:?}x{}", stack.utility, stack.count))
                    .collect::<Vec<_>>()
                    .join(" "),
                bot.metres(),
                bot.goal(),
            ));
        }
        let teams: Vec<String> = game
            .outfits
            .teams
            .iter()
            .map(|team| format!("forge {} swords {} shields {} pool {} traps {}", team.forge, team.swords, team.shields, team.heal_pool, team.traps))
            .collect();
        lines.push(format!("  upgrades: {}", teams.join(" | ")));
        lines.join("\n")
    }

    /// Not a test of anything: a game of the computer's players alone,
    /// told as it goes, for looking at what they do. `cargo test -p
    /// roundtown_server watch -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn watch_a_game_of_bots() {
        let seed = std::env::var("RT_SEED").ok().and_then(|seed| seed.parse().ok()).unwrap_or(1);
        let brief = std::env::var_os("RT_BRIEF").is_some();
        let mut game = bots_alone(seed, false);
        let mut next = 0.0;
        let mut alive = [true; ARENA_SEATS];
        let mut anchors = game.anchors;
        let mut goals = [roundtown_net::bots::Goal::Idle; ARENA_SEATS];
        while game.playing() {
            if game.elapsed >= next && !brief {
                println!("{}", report(&game));
                next += if game.elapsed < 300.0 { 20.0 } else { 60.0 };
            }
            let was = game.at.clone();
            game.step(1.0 / 30.0);
            game.events.clear();
            for seat in 0..ARENA_SEATS {
                let bot = game.squad.bot(seat).expect("the computer's");
                if alive[seat] && !game.alive(seat) {
                    let how = if game.at[seat].y < -5.0 { "FELL" } else { "KILLED" };
                    println!(
                        "    {:>4.0} s: seat {seat} (team {}) {how} from ({:.0},{:.0}) {:?} on a plank {} doing {:?}{}",
                        game.elapsed,
                        arena_team(seat),
                        was[seat].x,
                        was[seat].z,
                        game.nav.grid().region(was[seat].xz()),
                        game.nav.on_plank(was[seat].xz()),
                        goals[seat],
                        if game.out[seat] { ", and is OUT" } else { "" },
                    );
                }
                alive[seat] = game.alive(seat);
                if game.alive(seat) {
                    if goals[seat] != bot.goal() && brief && matches!(bot.goal(), roundtown_net::bots::Goal::Raid(_) | roundtown_net::bots::Goal::Hunt | roundtown_net::bots::Goal::Defend) {
                        println!("    {:>4.0} s: seat {seat} (team {}) -> {:?}", game.elapsed, arena_team(seat), bot.goal());
                    }
                    goals[seat] = bot.goal();
                }
            }
            for team in 0..ARENA_TEAMS {
                if anchors[team] > 0.0 && game.anchors[team] <= 0.0 {
                    println!("    {:>4.0} s: team {team}'s anchor is gone", game.elapsed);
                }
            }
            anchors = game.anchors;
        }
        println!("{}", report(&game));
        println!("over: {:?} at {:.0} s", game.stage, game.elapsed);
    }

    /// How many kinds of thing there are for one of their buttons
    /// (`bots::KINDS`).
    const BUTTON_KINDS: usize = roundtown_net::bots::KINDS.len();

    /// What happened in a game of the computer's players alone, for
    /// telling whether they play it sensibly.
    #[derive(Debug)]
    struct Chronicle {
        seed: u64,
        /// When each team's island was joined to the middle, and when the
        /// first of the smaller islands was.
        joined: [Option<f32>; ARENA_TEAMS],
        sapphire: Option<f32>,
        /// When the first team upgrade was bought, and the first sword.
        upgrade: Option<f32>,
        sword: Option<f32>,
        /// When each anchor went, and when each team was out.
        anchor: [Option<f32>; ARENA_TEAMS],
        out: [Option<f32>; ARENA_TEAMS],
        /// Deaths before the anchors break, and after: killed, and fallen.
        killed: (u32, u32),
        fell: (u32, u32),
        /// Whoever fell with nobody having hit them in the three seconds
        /// before: walked off, which must never be. But for whoever fell
        /// with a bridge that came down under them, or from one that was
        /// coming down, before the anchors break and after
        /// (`with_bridge`), since 2026-11-15, when a stick of dynamite
        /// began to take the whole bridge; and whoever, left on planks
        /// with no land to get to, stepped off them (`stepped_off`).
        /// And how many times a bridge was brought down (`cascades`).
        walked_off: u32,
        with_bridge: (u32, u32),
        stepped_off: u32,
        cascades: u32,
        /// How many times one of them set out to cut another team's way
        /// with a stick of dynamite (`Squad::way_to_cut`).
        cuts: u32,
        /// The most seconds any one of them stood with nothing to do
        /// while standing, before the anchors broke.
        idlest: f32,
        /// Sudden death: how many of the players a zombie killed; how
        /// many of them were still in the game at 15:00 and at each
        /// minute after, to 20:00; and for how many seconds, all told,
        /// they were running from a zombie.
        by_zombies: u32,
        left_at: [Option<u8>; 6],
        dreading: f32,
        /// How it ended.
        over: f32,
        winner: Option<u8>,
        /// Who each of the eight was, how each team opened, and how often
        /// each of them died.
        who: [&'static str; ARENA_SEATS],
        openings: [char; ARENA_TEAMS],
        deaths: [u32; ARENA_SEATS],
        /// How many of each kind of thing for a button they came by, all
        /// of them together, and how many they used (`bots::KINDS`: a
        /// fireball, a speed potion, an invisibility potion, a healing
        /// potion, a Bridge Builder, a stick of dynamite, a leap potion);
        /// and the most
        /// kinds any one of them had on its buttons at once.
        carried: [u32; BUTTON_KINDS],
        used: [u32; BUTTON_KINDS],
        fullest: usize,
        /// How many sticks of dynamite blew, and how many planks they
        /// took with them.
        blasts: u32,
        planks_blown: u32,
        /// How many deaths on land dropped anything, how many sapphires
        /// and amethysts between them, how many of those were picked up,
        /// and how many went with a plank (`Game::spilt`).
        spilt: [u32; 4],
        /// How many a bus ran over, how many of them it killed, and how
        /// many blows landed on someone lying flat (`Game::squashes`); and
        /// how many of those run over were zombies, how many of the
        /// players were in the middle of a fight, and how many times one
        /// of the computer's set off to finish someone lying there.
        squashes: [u32; 3],
        squashed_zombies: u32,
        squashed_fighting: u32,
        preyed: u32,
        /// How many times one of the players stepped into where a bus
        /// sweeps, how many of those with a bus due there within a second
        /// and a half, and how many seconds they spent in there between
        /// them.
        lane: (u32, u32, f32),
        /// How often one of them got nowhere on a way it had, three
        /// seconds together (`Bot::stalls`), standing its ground or
        /// held; and how many of those times something held it
        /// (`Bot::pinned`), which must never be.
        stalls: u32,
        pinned: u32,
        /// How they spent the game ([`Habits`]).
        habits: Habits,
    }

    /// How the computer's players of a game spent it, all eight together
    /// (since 2026-11-15, Hajun: "the ai seem to be just in their island
    /// for too long of the game like idle like both of them on the
    /// generator"): what is counted to tell whether they are out and about.
    /// Every time is in seconds of one of them standing, before the anchors
    /// break at ten minutes, the keeper's first and the one that goes out's
    /// second ([`roundtown_net::bots::Role`], as it was that frame).
    #[derive(Clone, Copy, Debug, Default)]
    struct Habits {
        /// Standing at all.
        alive: [f32; 2],
        /// On the land of its own island; within two and a half metres of
        /// its own pad; with nothing to do (`Goal::Idle`); there for what
        /// the pad makes (`Goal::Collect`); and on its own island, going
        /// nowhere.
        home: [f32; 2],
        pad: [f32; 2],
        idle: [f32; 2],
        collect: [f32; 2],
        still: [f32; 2],
        /// On the land of another team's island; in the middle; on one of
        /// the smaller islands; and out on a bridge.
        enemy: [f32; 2],
        middle: [f32; 2],
        sapphire: [f32; 2],
        bridge: [f32; 2],
        /// Once its island was joined to anything: standing, and of that
        /// on its own island.
        bridged: [f32; 2],
        bridged_home: [f32; 2],
        /// At a stand's outline, going nowhere; and out looking about
        /// (`Goal::Scout`), once there is such a thing.
        browsing: [f32; 2],
        scouting: [f32; 2],
        /// A team's two, in seconds of both of them standing: so long in
        /// all, with both by their pad, and with both on their island.
        pair: f32,
        pair_pad: f32,
        pair_home: f32,
        /// How many times one of them set off on a raid, and how many
        /// times one stepped onto the island it was raiding; and what it
        /// did there first: hit someone, hit the anchor, stood on the pad,
        /// or none of them before it left or died.
        raids: u32,
        landings: u32,
        first: [u32; 4],
        /// What the anchors had lost to punches by ten minutes, all four
        /// together, in points of health; and how many punches that was.
        anchor_lost: f32,
        anchor_hits: u32,
        /// How many bridges were begun, by where from and where to: its
        /// own island to the middle, its own island to a smaller island,
        /// the middle to a smaller island, the middle to another team's
        /// island, a smaller island to another team's island, its own
        /// island straight to another team's, and anything else.
        bridges: [u32; 7],
        /// How far the teams had quickened their building by the end, all
        /// four together; and how many leap potions were bought and drunk.
        build_speed: u32,
        leaps: (u32, u32),
    }

    impl Default for Chronicle {
        fn default() -> Self {
            Self {
                seed: 0,
                joined: [None; ARENA_TEAMS],
                sapphire: None,
                upgrade: None,
                sword: None,
                anchor: [None; ARENA_TEAMS],
                out: [None; ARENA_TEAMS],
                killed: (0, 0),
                fell: (0, 0),
                walked_off: 0,
                with_bridge: (0, 0),
                stepped_off: 0,
                cascades: 0,
                cuts: 0,
                idlest: 0.0,
                by_zombies: 0,
                left_at: [None; 6],
                dreading: 0.0,
                over: 0.0,
                winner: None,
                who: ["-"; ARENA_SEATS],
                openings: ['M'; ARENA_TEAMS],
                deaths: [0; ARENA_SEATS],
                carried: [0; BUTTON_KINDS],
                used: [0; BUTTON_KINDS],
                fullest: 0,
                blasts: 0,
                planks_blown: 0,
                spilt: [0; 4],
                squashes: [0; 3],
                squashed_zombies: 0,
                squashed_fighting: 0,
                preyed: 0,
                lane: (0, 0, 0.0),
                stalls: 0,
                pinned: 0,
                habits: Habits::default(),
            }
        }
    }

    /// The squares round `here`, a line to a row, for seeing what holds a
    /// body there: `@` where it is and `X` where it is going, if that is
    /// near; `.` land to walk, `o` land to walk under a plank, `=` a
    /// bridge's deck, `-` a plank's edge, which holds a body and is not
    /// walked, `#` land that holds a body and is not walked, and nothing
    /// where there is nothing.
    fn squares_round(nav: &roundtown_net::nav::Nav, here: Vec2, next: Vec2) -> String {
        let cell = roundtown_net::nav::CELL;
        let middle = (here / cell).floor() * cell + Vec2::splat(cell * 0.5);
        let mut drawn = String::new();
        for row in -7i32..=7 {
            drawn.push_str("  |");
            for column in -10i32..=10 {
                let at = middle + Vec2::new(column as f32, row as f32) * cell;
                drawn.push(if (row, column) == (0, 0) {
                    '@'
                } else if (next - at).abs().max_element() < cell * 0.5 {
                    'X'
                } else if nav.grid().clear(at) {
                    if nav.on_plank(at) { 'o' } else { '.' }
                } else if nav.walkable_at(at) {
                    '='
                } else if nav.on_plank(at) {
                    '-'
                } else if nav.grid().height(at).is_some() {
                    '#'
                } else {
                    ' '
                });
            }
            drawn.push_str(&format!("| z {:.2}
", middle.y + row as f32 * cell));
        }
        let way = (next - here).normalize_or_zero();
        drawn.push_str(&format!("  from x {:.2}, heading ({:.2}, {:.2})", middle.x - 10.0 * cell, way.x, way.y));
        drawn
    }

    /// Plays a game of eight of the computer's players to its end, and
    /// says what happened in it.
    fn chronicle(seed: u64) -> Chronicle {
        use roundtown_net::bots::{Goal, Role};
        use roundtown_net::nav::Region;
        let mut game = bots_alone(seed, false);
        // (`RT_BEGIN=<seconds>`: the fight begun that far into the
        // timetable, everything before then having happened, as the
        // server begins one late with `RT_ARENA_FROM`: for looking at
        // sudden death, which the computer's players alone are mostly
        // done before.)
        if let Some(from) = std::env::var("RT_BEGIN").ok().and_then(|from| from.parse::<f32>().ok()) {
            game.elapsed = from.clamp(0.0, PLAY_SECS - 1.0);
            game.left = PLAY_SECS - game.elapsed;
            let dice = &mut game.dice;
            game.mint = Mint::new(game.elapsed, &mut || dice.f32());
            let before: Vec<Happening> = happening_between(-1.0, game.elapsed).collect();
            for event in before {
                game.happen(event);
            }
        }
        let mut told = Chronicle {
            seed,
            who: std::array::from_fn(|seat| game.squad.bot(seat).map_or("-", |bot| bot.persona().name)),
            openings: std::array::from_fn(|team| match game.squad.plan(team).opening {
                roundtown_net::bots::Opening::Middle => 'M',
                roundtown_net::bots::Opening::Sapphire => 'S',
                roundtown_net::bots::Opening::Late => 'L',
            }),
            ..Chronicle::default()
        };
        let dt = 1.0 / 30.0;
        let mut alive = [true; ARENA_SEATS];
        let mut idle = [0.0f32; ARENA_SEATS];
        let mut held = [[0u8; BUTTON_KINDS]; ARENA_SEATS];
        let mut stalls = [0u32; ARENA_SEATS];
        let mut pinned = [0u32; ARENA_SEATS];
        // For how they spent it ([`Habits`]): where each was a frame ago,
        // whether it was on a raid, the island it has stepped onto on one
        // and done nothing on yet, whether it stood on that island a frame
        // ago, how many leap potions it held, and what the anchors had.
        let mut was_at: [Vec2; ARENA_SEATS] = std::array::from_fn(|seat| game.at[seat].xz());
        let mut raiding = [false; ARENA_SEATS];
        let mut landed: [Option<usize>; ARENA_SEATS] = [None; ARENA_SEATS];
        let mut ashore = [false; ARENA_SEATS];
        let mut leaps_held = [0u8; ARENA_SEATS];
        let mut anchors_were = game.anchors;
        // For telling a fall with a bridge from walking off an edge: the
        // plank each stood on a frame ago, out over the void, if it did;
        // when the ground last went from under where each stood, and
        // when each last stood on planks that were coming down; and
        // whether any were, a frame ago.
        let mut stood: [Option<Vec2>; ARENA_SEATS] = [None; ARENA_SEATS];
        let mut dropped = [f32::NEG_INFINITY; ARENA_SEATS];
        let mut crumbling = false;
        let mut cutting = [false; ARENA_SEATS];
        // How each anchor went, for saying so (`RT_ANCHORS`): when it was
        // first hit, how many punches it has taken, who threw them, and
        // how many of its own team stood on their island at each.
        let say_anchors = std::env::var_os("RT_ANCHORS").is_some();
        let mut first_hit = [None::<f32>; ARENA_TEAMS];
        let mut punches = [0u32; ARENA_TEAMS];
        let mut hitters: [Vec<usize>; ARENA_TEAMS] = Default::default();
        let mut defended = [0u32; ARENA_TEAMS];
        // One of them followed, for finding what it was about where it
        // got nowhere: `RT_TRACE=<seat>,<from>,<until>`, in seconds.
        let trace: Option<(usize, f32, f32)> = std::env::var("RT_TRACE").ok().and_then(|trace| {
            let mut parts = trace.split(',');
            Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
        });
        let mut traced = 0.0;
        let mut flat_was = [false; 32];
        let mut prey_was = [None::<usize>; ARENA_SEATS];
        let mut in_lane = [false; ARENA_SEATS];
        while game.playing() {
            game.step(dt);
            for seat in 0..ARENA_SEATS {
                let at = game.at[seat];
                let lane = Bus::sweeps(at.xz(), BODY_RADIUS).filter(|_| game.alive(seat) && at.y.abs() < 1.5 && crossing_at(at).is_none());
                let inside = lane.is_some();
                if inside {
                    told.lane.2 += dt;
                }
                if let Some(lane) = lane
                    && !in_lane[seat]
                {
                    told.lane.0 += 1;
                    let clock = game.bus_clock.unwrap_or(0.0);
                    if game.buses.is_some_and(|buses| buses.reaches_in(lane, clock, at.xz()) < 1.5) {
                        told.lane.1 += 1;
                    }
                }
                in_lane[seat] = inside;
            }
            // Whoever a bus has just run over: a zombie, or one of the
            // players, in a fight or about its business; and whoever has
            // just set off to finish someone lying flat.
            for seat in 0..game.seats.len().min(32) {
                let flat = game.flat[seat] > 0.0;
                if flat && !flat_was[seat] {
                    if seat >= ARENA_SEATS {
                        told.squashed_zombies += 1;
                    } else if game.squad.bot(seat).is_some_and(|bot| bot.stance().is_some()) {
                        told.squashed_fighting += 1;
                    }
                    if std::env::var_os("RT_SAY_BUSES").is_some() {
                        println!(
                            "  {:>6.1} s: a bus ran seat {seat} over at ({:.1}, {:.1}), {}",
                            game.elapsed,
                            game.at[seat].x,
                            game.at[seat].z,
                            game.squad.bot(seat).map_or(String::from("a zombie"), |bot| format!("{} {:?} {:?}", bot.persona().name, bot.goal(), bot.stance()))
                        );
                    }
                }
                flat_was[seat] = flat;
            }
            for seat in 0..ARENA_SEATS {
                let prey = game.squad.bot(seat).and_then(|bot| bot.prey());
                if prey.is_some() && prey != prey_was[seat] {
                    told.preyed += 1;
                }
                prey_was[seat] = prey;
            }
            if let Some((seat, from, until)) = trace
                && (from..until).contains(&game.elapsed)
                && game.elapsed >= traced
            {
                traced = game.elapsed + 0.25;
                let bot = game.squad.bot(seat).expect("the computer's");
                let here = game.at[seat];
                let others: Vec<String> = (0..ARENA_SEATS)
                    .filter(|&other| other != seat && game.alive(other) && game.at[other].xz().distance(here.xz()) < 25.0)
                    .map(|other| format!("{other}{}@({:.1},{:.1})", if arena_team(other) == arena_team(seat) { "m" } else { "" }, game.at[other].x, game.at[other].z))
                    .collect();
                println!(
                    "  {:>6.2} s seat {seat}: ({:.2}, {:.2}, {:.2}) {:?} {:?} health {:.0} walked {} sticks {} falling {:?} getting off {} | {}",
                    game.elapsed,
                    here.x,
                    here.y,
                    here.z,
                    bot.goal(),
                    bot.stance(),
                    game.health[seat],
                    game.nav.walkable_at(here.xz()),
                    game.sticks.len(),
                    game.nav.falls_in(here.xz()),
                    bot.fleeing(),
                    others.join(" "),
                );
            }
            // Whoever a zombie has just killed.
            told.by_zombies += game
                .events
                .iter()
                .filter(|event| matches!(event, Event::Feed(Feed::Killed { seat, by, .. }) if usize::from(*seat) < ARENA_SEATS && usize::from(*by) >= ARENA_SEATS))
                .count() as u32;
            game.events.clear();
            let now = game.elapsed;
            // Who is left as sudden death begins, and each minute of it.
            for (minute, left) in told.left_at.iter_mut().enumerate() {
                if left.is_none() && now >= SUDDEN_DEATH + 60.0 * minute as f32 - 0.02 {
                    *left = Some((0..ARENA_SEATS).filter(|&seat| game.in_play(seat)).count() as u8);
                }
            }
            told.dreading += (0..ARENA_SEATS).filter(|&seat| game.alive(seat) && game.squad.bot(seat).is_some_and(|bot| bot.dreads())).count() as f32 * (1.0 / 30.0);
            let early = now < ANCHORS_BREAK;
            // What the anchors have lost to punches, before they all break.
            let struck: Vec<usize> = (0..ARENA_TEAMS).filter(|&team| game.anchors[team] < anchors_were[team]).collect();
            if early {
                for &team in &struck {
                    told.habits.anchor_lost += anchors_were[team] - game.anchors[team];
                    told.habits.anchor_hits += 1;
                }
            }
            if early && say_anchors {
                for &team in &struck {
                    let grid = game.nav.grid();
                    let own = Some(Region::Team(team as u8));
                    let home = (0..ARENA_SEATS)
                        .filter(|&seat| usize::from(arena_team(seat)) == team && game.alive(seat))
                        .filter(|&seat| grid.height(game.at[seat].xz()).and_then(|_| grid.region(game.at[seat].xz())) == own)
                        .count() as u32;
                    let hitter = (0..ARENA_SEATS)
                        .filter(|&seat| usize::from(arena_team(seat)) != team && game.alive(seat))
                        .min_by(|&a, &b| {
                            let far = |seat: usize| game.at[seat].xz().distance(ARENA_ANCHORS[team]);
                            far(a).total_cmp(&far(b))
                        });
                    first_hit[team].get_or_insert(now);
                    punches[team] += 1;
                    defended[team] += home;
                    if let Some(hitter) = hitter
                        && !hitters[team].contains(&hitter)
                    {
                        hitters[team].push(hitter);
                    }
                    if game.anchors[team] <= 0.0 {
                        let who: Vec<String> = hitters[team]
                            .iter()
                            .map(|&seat| {
                                let bot = game.squad.bot(seat).expect("the computer's");
                                format!("{} {:?} zeal {:.2}", bot.persona().name, bot.role(), bot.temper().zeal)
                            })
                            .collect();
                        println!(
                            "ANCHOR seed {seed}: team {team}'s gone at {now:.0} s, first hit at {:.0} s, {} punches by {}; {:.1} of its own at home at a punch",
                            first_hit[team].unwrap_or(now),
                            punches[team],
                            who.join(", "),
                            defended[team] as f32 / punches[team].max(1) as f32,
                        );
                    }
                }
            }
            anchors_were = game.anchors;
            for team in 0..ARENA_TEAMS {
                if told.joined[team].is_none() && game.nav.joined(Region::Team(team as u8), Region::Middle) {
                    told.joined[team] = Some(now);
                }
                if told.anchor[team].is_none() && game.anchors[team] <= 0.0 {
                    told.anchor[team] = Some(now);
                }
                if told.out[team].is_none() && !game.teams_in_play().contains(&(team as u8)) {
                    told.out[team] = Some(now);
                }
                let bought = game.outfits.teams[team];
                if told.upgrade.is_none() && (bought.forge > 0 || bought.heal_pool || bought.swords || bought.shields > 0 || bought.traps > 0) {
                    told.upgrade = Some(now);
                }
            }
            if told.sapphire.is_none() && (0..4).any(|island| game.nav.joined(Region::Sapphire(island), Region::Middle)) {
                told.sapphire = Some(now);
            }
            if told.sword.is_none() && game.outfits.kits.iter().any(|kit| kit.sword > 0) {
                told.sword = Some(now);
            }
            for seat in 0..ARENA_SEATS {
                let standing = game.alive(seat);
                // What it has on its buttons: more of a kind is bought,
                // and less of one, while it stands, is used.
                let kit = game.outfits.kit(seat);
                for (kind, utility) in roundtown_net::bots::KINDS.into_iter().enumerate() {
                    let (now_held, was) = (kit.held(utility), held[seat][kind]);
                    if now_held > was {
                        told.carried[kind] += u32::from(now_held - was);
                    } else if now_held < was && standing && alive[seat] {
                        told.used[kind] += u32::from(was - now_held);
                    }
                    held[seat][kind] = now_held;
                }
                told.fullest = told.fullest.max(kit.slots.iter().flatten().filter(|stack| stack.count > 0).count());
                // The planks it stood on a frame ago are gone, or are
                // going: whatever becomes of it in the next few seconds is
                // the bridge's doing.
                let across = game.at[seat].xz();
                if stood[seat].is_some_and(|plank| game.nav.ground(plank).is_none()) || game.nav.falls_in(across).is_some() {
                    dropped[seat] = now;
                }
                stood[seat] = (standing && game.nav.grid().height(across).is_none() && game.nav.ground(across).is_some()).then_some(across);
                if alive[seat] && !standing {
                    told.deaths[seat] += 1;
                    let fallen = game.at[seat].y < -5.0;
                    let count = if fallen { &mut told.fell } else { &mut told.killed };
                    if early {
                        count.0 += 1;
                    } else {
                        count.1 += 1;
                    }
                    if fallen && now - dropped[seat] < 4.0 {
                        let count = &mut told.with_bridge;
                        if early {
                            count.0 += 1;
                        } else {
                            count.1 += 1;
                        }
                    } else if fallen && game.squad.bot(seat).is_some_and(|bot| bot.stepped_off()) {
                        told.stepped_off += 1;
                    } else if fallen && game.since_hit[seat] > 3.0 {
                        told.walked_off += 1;
                    }
                }
                alive[seat] = standing;
                let bot = game.squad.bot(seat).expect("the computer's");
                if bot.cutting().is_some() && !cutting[seat] {
                    told.cuts += 1;
                }
                cutting[seat] = bot.cutting().is_some();
                // Got nowhere on its way somewhere, and whether something
                // held it: said, with `RT_STALLS` set, for finding what
                // did, and with `RT_STALLS=2` the squares round it too.
                let (count, last) = bot.stalls();
                if count > stalls[seat] {
                    stalls[seat] = count;
                    told.stalls += 1;
                    let held = bot.pinned() > pinned[seat];
                    if held {
                        pinned[seat] = bot.pinned();
                        told.pinned += 1;
                    }
                    if let (Ok(how), Some((here, next))) = (std::env::var("RT_STALLS"), last) {
                        // (And what else was going on there: how near
                        // the nearest of another team stood, how it
                        // stood to them, and whether the planks under
                        // it were coming down.)
                        let nearest = (0..ARENA_SEATS)
                            .filter(|&other| game.alive(other) && arena_team(other) != arena_team(seat))
                            .map(|other| game.at[other].xz().distance(here))
                            .fold(f32::INFINITY, f32::min);
                        println!(
                            "{} seed {seed} at {now:.0} s: {} (seat {seat}) at ({:.2}, {:.2}) for ({:.2}, {:.2}), {:?}, {:?}, on a plank {}, walked there {}; the nearest of another team {nearest:.1} m off, {:?}; falling in {:?}, getting off {}",
                            if held { "HELD" } else { "nowhere" },
                            bot.persona().name,
                            here.x,
                            here.y,
                            next.x,
                            next.y,
                            bot.goal(),
                            game.nav.grid().region(here),
                            game.nav.on_plank(here),
                            game.nav.walkable_at(here),
                            bot.stance(),
                            game.nav.falls_in(here),
                            bot.fleeing(),
                        );
                        if how == "2" {
                            println!("{}", squares_round(&game.nav, here, next));
                        }
                    }
                }
                // The one that goes out with nothing to do, while it
                // stands, before the anchors break. The keeper's part is
                // to stand by its anchor.
                if standing && early && bot.role() == Role::Runner && bot.goal() == Goal::Idle {
                    idle[seat] += dt;
                    told.idlest = told.idlest.max(idle[seat]);
                } else {
                    idle[seat] = 0.0;
                }
                // How it spends the game ([`Habits`]): where it is and
                // what it is about, before the anchors break.
                let team = usize::from(arena_team(seat));
                let own = Region::Team(team as u8);
                let at = game.at[seat].xz();
                let grid = game.nav.grid();
                let land = grid.height(at).and_then(|_| grid.region(at));
                let went = at.distance(was_at[seat]);
                was_at[seat] = at;
                let goal = bot.goal();
                let role = usize::from(bot.role() == Role::Runner);
                let habits = &mut told.habits;
                if standing && early {
                    habits.alive[role] += dt;
                    match land {
                        Some(region) if region == own => habits.home[role] += dt,
                        Some(Region::Team(_)) => habits.enemy[role] += dt,
                        Some(Region::Middle | Region::Pocket) => habits.middle[role] += dt,
                        Some(Region::Sapphire(_)) => habits.sapphire[role] += dt,
                        None => habits.bridge[role] += dt,
                    }
                    let still = went < 0.3 * dt;
                    if at.distance(roundtown_net::ARENA_PADS[team]) < 2.5 {
                        habits.pad[role] += dt;
                    }
                    if goal == Goal::Idle {
                        habits.idle[role] += dt;
                    }
                    if goal == Goal::Collect {
                        habits.collect[role] += dt;
                    }
                    if matches!(goal, Goal::Scout(_)) {
                        habits.scouting[role] += dt;
                    }
                    if still && land == Some(own) {
                        habits.still[role] += dt;
                    }
                    // (In a stand's outline, which reaches a metre and a
                    // half from its middle.)
                    let stands = [roundtown_net::ARENA_SHOPS[team], roundtown_net::ARENA_UPGRADE_SHOPS[team]];
                    if still && stands.iter().any(|stand| stand.distance(at) < 1.6) {
                        habits.browsing[role] += dt;
                    }
                    let bridged = game.nav.joined(own, Region::Middle) || (0..4).any(|island| game.nav.joined(own, Region::Sapphire(island)));
                    if bridged {
                        habits.bridged[role] += dt;
                        if land == Some(own) {
                            habits.bridged_home[role] += dt;
                        }
                    }
                }
                // A raid set off on, an island stepped onto on one, and
                // what it does there first.
                let on_raid = match goal {
                    Goal::Raid(victim) if standing => Some(usize::from(victim)),
                    _ => None,
                };
                if early && on_raid.is_some() && !raiding[seat] {
                    habits.raids += 1;
                }
                raiding[seat] = on_raid.is_some();
                let there = on_raid.filter(|&victim| land == Some(Region::Team(victim as u8)));
                if early && there.is_some() && !ashore[seat] {
                    habits.landings += 1;
                    landed[seat] = there;
                }
                ashore[seat] = there.is_some();
                if let Some(victim) = landed[seat] {
                    let hit_someone = (0..ARENA_SEATS).any(|other| {
                        usize::from(arena_team(other)) == victim
                            && game.last_hit[other].is_some_and(|(hitter, when, _)| hitter == seat && (now - when).abs() < 1.5 * dt)
                    });
                    let hit_anchor = struck.contains(&victim)
                        && at.distance(ARENA_ANCHORS[victim]) <= REACH + roundtown_net::ANCHOR_RADIUS + 0.3;
                    let on_pad = at.distance(roundtown_net::ARENA_PADS[victim]) < 1.6;
                    let did = if hit_someone {
                        Some(0)
                    } else if hit_anchor {
                        Some(1)
                    } else if on_pad {
                        Some(2)
                    } else if there.is_none() {
                        Some(3)
                    } else {
                        None
                    };
                    if let Some(did) = did {
                        habits.first[did] += 1;
                        landed[seat] = None;
                    }
                }
                // Leap potions: more of them is one bought, and fewer,
                // while it stands, one drunk.
                let leaps = kit.held(Utility::Leap);
                if leaps > leaps_held[seat] {
                    habits.leaps.0 += u32::from(leaps - leaps_held[seat]);
                } else if leaps < leaps_held[seat] && standing {
                    habits.leaps.1 += u32::from(leaps_held[seat] - leaps);
                }
                leaps_held[seat] = leaps;
            }
            // A team's two together: both by their pad, both at home.
            if early {
                for team in 0..ARENA_TEAMS {
                    let (one, other) = (team, team + ARENA_TEAMS);
                    if !game.alive(one) || !game.alive(other) {
                        continue;
                    }
                    let grid = game.nav.grid();
                    let own = Some(Region::Team(team as u8));
                    let at_home = |seat: usize| {
                        let at = game.at[seat].xz();
                        grid.height(at).and_then(|_| grid.region(at)) == own
                    };
                    let by_pad = |seat: usize| game.at[seat].xz().distance(roundtown_net::ARENA_PADS[team]) < 2.5;
                    told.habits.pair += dt;
                    if by_pad(one) && by_pad(other) {
                        told.habits.pair_pad += dt;
                    }
                    if at_home(one) && at_home(other) {
                        told.habits.pair_home += dt;
                    }
                }
            }
            if game.nav.crumbling() && !crumbling {
                told.cascades += 1;
            }
            crumbling = game.nav.crumbling();
            assert!(game.at.iter().all(|at| at.is_finite()), "seed {seed}: somebody is nowhere at {now} s");
        }
        told.over = game.elapsed;
        told.winner = match game.stage {
            ArenaStage::Over { winner } => winner,
            _ => None,
        };
        told.blasts = game.blasts;
        told.planks_blown = game.planks_blown;
        told.spilt = game.spilt;
        told.squashes = game.squashes;
        // What they built, by where from and where to; and how far they
        // quickened their building.
        for &(from, to, times) in game.squad.begun() {
            let kind = match (from, to) {
                (Region::Team(_), Region::Middle) => 0,
                (Region::Team(_), Region::Sapphire(_)) => 1,
                (Region::Middle, Region::Sapphire(_)) => 2,
                (Region::Middle, Region::Team(_)) => 3,
                (Region::Sapphire(_), Region::Team(_)) => 4,
                (Region::Team(_), Region::Team(_)) => 5,
                _ => 6,
            };
            told.habits.bridges[kind] += u32::from(times);
        }
        told.habits.build_speed = game.outfits.teams.iter().map(|team| u32::from(team.build_speed)).sum();
        told
    }

    /// The games of `seeds`, each told in a line.
    fn told(seeds: std::ops::Range<u64>) -> Vec<Chronicle> {
        let games: Vec<Chronicle> = seeds.map(chronicle).collect();
        let time = |at: Option<f32>| at.map_or(String::from("   -"), |at| format!("{at:>4.0}"));
        println!("seed | joined to the middle | sapph | sword | upgr | anchors gone        | out                 | killed fell (early) (late) | off | idle | over winner");
        for game in &games {
            println!(
                "{:>4} | {} {} {} {}  | {}  | {}  | {} | {} {} {} {} | {} {} {} {} | {:>2} {:>2}   {:>2} {:>2}            | {:>3} | {:>4.0} | {:>4.0} {:?}",
                game.seed,
                time(game.joined[0]),
                time(game.joined[1]),
                time(game.joined[2]),
                time(game.joined[3]),
                time(game.sapphire),
                time(game.sword),
                time(game.upgrade),
                time(game.anchor[0]),
                time(game.anchor[1]),
                time(game.anchor[2]),
                time(game.anchor[3]),
                time(game.out[0]),
                time(game.out[1]),
                time(game.out[2]),
                time(game.out[3]),
                game.killed.0,
                game.fell.0,
                game.killed.1,
                game.fell.1,
                game.walked_off,
                game.idlest,
                game.over,
                game.winner,
            );
            if std::env::var("RT_WHO").is_ok() {
                println!("       {} | opened {}", game.who.join(" "), game.openings.iter().collect::<String>());
            }
        }
        games
    }

    #[test]
    fn the_computer_s_players_play_whole_games_sensibly() {
        let games = told(1..9);
        tell_habits(&games);
        let mut winners = [0u32; ARENA_TEAMS];
        for game in &games {
            let seed = game.seed;
            // Every team's island is joined to the middle before long, by
            // its own bridge or, a team that keeps to itself, by someone
            // else's; a smaller island is reached, arms are bought and
            // the teams' upgrades begun. Not all in the first half
            // minute, as it was when they all opened alike: some build
            // to a smaller island first, and some arm themselves first.
            for team in 0..ARENA_TEAMS {
                assert!(game.joined[team].is_some_and(|at| at < 330.0), "seed {seed}: team {team} joined at {:?}", game.joined[team]);
            }
            assert!(game.joined.iter().flatten().any(|at| *at < 60.0), "seed {seed}: nobody across in the first minute");
            assert!(game.sapphire.is_some_and(|at| at < 240.0), "seed {seed}: a smaller island at {:?}", game.sapphire);
            assert!(game.sword.is_some_and(|at| at < 120.0), "seed {seed}: a sword at {:?}", game.sword);
            assert!(game.upgrade.is_some_and(|at| at < 300.0), "seed {seed}: an upgrade at {:?}", game.upgrade);
            // Nobody walks off an edge, ever.
            assert_eq!(game.walked_off, 0, "seed {seed}");
            // And nothing holds anybody fast on their way anywhere: no
            // corner of a stand, nor the end of a bridge against the
            // land (`RT_STALLS=2` shows where).
            assert_eq!(game.pinned, 0, "seed {seed}: one of them was held fast");
            assert!(game.stalls <= 1, "seed {seed}: they got nowhere {} times", game.stalls);
            // No anchor goes before anchors are fair game, and nobody is
            // out before then.
            for team in 0..ARENA_TEAMS {
                assert!(game.anchor[team].is_none_or(|at| at >= 240.0), "seed {seed}: team {team}'s anchor at {:?}", game.anchor[team]);
                assert!(game.out[team].is_none_or(|at| at >= 240.0), "seed {seed}: team {team} out at {:?}", game.out[team]);
            }
            // It ends, somebody having won it, well before its time is up.
            assert!(game.over < 1200.0, "seed {seed}: over at {}", game.over);
            if let Some(winner) = game.winner {
                winners[usize::from(winner)] += 1;
            }
        }
        // And no island wins them all.
        println!("wins by team: {winners:?}");
        assert!(winners.iter().filter(|wins| **wins > 0).count() >= 3, "{winners:?}");
        // **They are out and about** (since 2026-11-15; Hajun: "the ai
        // seem to be just in their island for too long of the game like
        // idle like both of them on the generator"), over the eight games
        // together, before the anchors break. By their pad a small part
        // of the time, a team's two all but never both there (55% of it
        // and 36% until then); off their island more than on it once they
        // have a way off it (on it 77%); at a stand some of the time they
        // are home, and out looking about some of the rest.
        let sum = |pick: &dyn Fn(&Habits) -> [f32; 2]| games.iter().map(|game| pick(&game.habits)).fold(0.0, |sum, spent| sum + spent[0] + spent[1]);
        let alive = sum(&|habits| habits.alive);
        let pair = games.iter().map(|game| game.habits.pair).sum::<f32>();
        let pair_pad = games.iter().map(|game| game.habits.pair_pad).sum::<f32>();
        assert!(sum(&|habits| habits.pad) < 0.12 * alive, "by their pads {:.0}% of the time", 100.0 * sum(&|habits| habits.pad) / alive);
        assert!(pair_pad < 0.03 * pair, "a team's two both by their pad {:.0}% of the time", 100.0 * pair_pad / pair);
        let (home, bridged) = (sum(&|habits| habits.bridged_home), sum(&|habits| habits.bridged));
        assert!(home < 0.6 * bridged, "on their islands {:.0}% of the time, with a way off them", 100.0 * home / bridged);
        assert!(sum(&|habits| habits.browsing) > 0.06 * alive, "at their stands {:.0}% of the time", 100.0 * sum(&|habits| habits.browsing) / alive);
        assert!(sum(&|habits| habits.scouting) > 0.02 * alive, "out looking about {:.0}% of the time", 100.0 * sum(&|habits| habits.scouting) / alive);
        // What one is on an island it raids for is not the same every
        // time: whoever is home, their anchor, what lies on their pad.
        // (The anchor first one time in sixteen until then, and the pad
        // one in twenty.)
        let count = |pick: &dyn Fn(&Habits) -> u32| games.iter().map(|game| pick(&game.habits)).sum::<u32>();
        let landings = count(&|habits| habits.landings);
        let [hit, anchor, pad, _] = [0, 1, 2, 3].map(|which| count(&|habits| habits.first[which]));
        assert!(landings >= 80, "{landings} islands stepped onto");
        assert!(5 * hit >= landings && 20 * anchor >= landings && 30 * pad >= landings, "of {landings}: someone hit {hit}, the anchor {anchor}, on the pad {pad}");
        // Anchors go to punches before they all break, a couple a game
        // (one game in two saw one go, until then), and not all of them
        // in every game: [`Skill::anchor_keen`] is what to turn.
        let broken = games.iter().map(|game| game.anchor.iter().filter(|at| at.is_some_and(|at| at < ANCHORS_BREAK - 1.0)).count()).sum::<usize>();
        assert!((6..=26).contains(&broken), "{broken} anchors broken by punches in eight games");
        // And they build ways nobody has to have, quicken their building,
        // and drink the leap potions they buy.
        let schemes = count(&|habits| habits.bridges[3] + habits.bridges[4] + habits.bridges[5]);
        assert!(schemes >= 4, "{schemes} bridges onto other teams' islands begun");
        assert!(count(&|habits| habits.build_speed) >= 2, "their building quickened {} steps", count(&|habits| habits.build_speed));
        assert!(count(&|habits| habits.leaps.1) >= 4, "{} leap potions drunk", count(&|habits| habits.leaps.1));
        // The same dice, the same game.
        let (one, other) = (chronicle(5), chronicle(5));
        assert_eq!((one.over, one.winner, one.killed, one.fell), (other.over, other.winner, other.killed, other.fell));
    }

    /// More of the same, for tuning by: `cargo test -p roundtown_server
    /// many -- --ignored --nocapture`, sixty games, or as many as
    /// `RT_GAMES` says, from the seed 100, or from the one `RT_FROM`
    /// says (one game over again: `RT_FROM=106 RT_GAMES=1`).
    #[test]
    #[ignore]
    fn many_games_of_bots() {
        let count: u64 = std::env::var("RT_GAMES").ok().and_then(|count| count.parse().ok()).unwrap_or(60);
        let from: u64 = std::env::var("RT_FROM").ok().and_then(|from| from.parse().ok()).unwrap_or(100);
        let games = told(from..from + count);
        let mean = |pick: &dyn Fn(&Chronicle) -> f32| games.iter().map(pick).sum::<f32>() / games.len() as f32;
        println!(
            "over at {:.0} s on average; got nowhere {:.2} times a game, held fast {:.2}; killed early {:.1}, fell early {:.1}; killed late {:.1}, fell late {:.1}; anchors broken before they all break {:.1}",
            mean(&|game| game.over),
            mean(&|game| game.stalls as f32),
            mean(&|game| game.pinned as f32),
            mean(&|game| game.killed.0 as f32),
            mean(&|game| game.fell.0 as f32),
            mean(&|game| game.killed.1 as f32),
            mean(&|game| game.fell.1 as f32),
            mean(&|game| game.anchor.iter().filter(|at| at.is_some_and(|at| at < ANCHORS_BREAK - 1.0)).count() as f32),
        );
        // Sudden death: who is left as it begins and at each minute of
        // it, in the games still going then; how many a zombie killed;
        // how long they ran from them; and how the games ended.
        let left: Vec<String> = (0..6)
            .map(|minute| {
                let going: Vec<f32> = games.iter().filter_map(|game| game.left_at[minute]).map(f32::from).collect();
                format!("{}:00 {:.1} in {} games", 15 + minute, going.iter().sum::<f32>() / going.len().max(1) as f32, going.len())
            })
            .collect();
        let over_by = |secs: f32| games.iter().filter(|game| game.over < secs).count();
        println!(
            "sudden death: players left at {}; {:.2} a game killed by a zombie; {:.0} s a game run from them, all told; over before 14:00 in {} games, before 15:00 in {}, before 16:00 in {}, before 18:00 in {}, drawn at 20:00 in {} of {}",
            left.join(", "),
            mean(&|game| game.by_zombies as f32),
            mean(&|game| game.dreading),
            over_by(ANCHORS_BREAK),
            over_by(SUDDEN_DEATH),
            over_by(SUDDEN_DEATH + 60.0),
            over_by(SUDDEN_DEATH + 180.0),
            games.iter().filter(|game| game.winner.is_none()).count(),
            games.len(),
        );
        // Who won, by island, and how many were drawn; and whose anchor
        // went first, and who was out first.
        let mut winners = [0u32; ARENA_TEAMS];
        let mut first_anchor = [0u32; ARENA_TEAMS];
        let mut first_out = [0u32; ARENA_TEAMS];
        let first = |times: &[Option<f32>; ARENA_TEAMS]| {
            (0..ARENA_TEAMS)
                .filter(|&team| times[team].is_some_and(|at| at < 599.0))
                .min_by(|&a, &b| times[a].unwrap_or(f32::MAX).total_cmp(&times[b].unwrap_or(f32::MAX)))
        };
        for game in &games {
            if let Some(team) = game.winner {
                winners[usize::from(team)] += 1;
            }
            if let Some(team) = first(&game.anchor) {
                first_anchor[team] += 1;
            }
            if let Some(team) = first(&game.out) {
                first_out[team] += 1;
            }
        }
        let drawn = games.iter().filter(|game| game.winner.is_none()).count();
        println!("of {} games: won by island {winners:?}, drawn {drawn}; first anchor broken before ten minutes {first_anchor:?}; first out before ten minutes {first_out:?}", games.len());
        // And by who they were: how many of each there were, how often
        // their team won, and how often each died in a game.
        let mut kinds: Vec<(&str, u32, u32, u32)> = Vec::new();
        let mut opened: [(u32, u32); 3] = [(0, 0); 3];
        for game in &games {
            for seat in 0..ARENA_SEATS {
                let name = game.who[seat];
                let won = game.winner == Some(arena_team(seat));
                match kinds.iter_mut().find(|kind| kind.0 == name) {
                    Some(kind) => {
                        kind.1 += 1;
                        kind.2 += u32::from(won);
                        kind.3 += game.deaths[seat];
                    }
                    None => kinds.push((name, 1, u32::from(won), game.deaths[seat])),
                }
            }
            for team in 0..ARENA_TEAMS {
                let which = "MSL".find(game.openings[team]).unwrap_or(0);
                opened[which].0 += 1;
                opened[which].1 += u32::from(game.winner == Some(team as u8));
            }
        }
        kinds.sort_by(|a, b| a.0.cmp(b.0));
        for (name, played, won, died) in kinds {
            println!("{name}: {played} played, their team won {:.0}%, died {:.1} times a game", 100.0 * won as f32 / played as f32, died as f32 / played as f32);
        }
        for (which, (teams, won)) in ["to the middle", "to a smaller island", "later"].into_iter().zip(opened) {
            println!("opened {which}: {teams} teams, won {:.0}%", 100.0 * won as f32 / teams.max(1) as f32);
        }
        // What they had on their buttons, all eight together, a game.
        let each = |pick: &dyn Fn(&Chronicle) -> [u32; BUTTON_KINDS]| {
            let mut sum = [0.0f32; BUTTON_KINDS];
            for game in &games {
                for (kind, count) in pick(game).into_iter().enumerate() {
                    sum[kind] += count as f32 / games.len() as f32;
                }
            }
            sum.map(|mean| format!("{mean:.1}")).join(", ")
        };
        println!(
            "a game, of fireballs, speed, invisibility and healing potions, Bridge Builders, sticks of dynamite and leap potions: bought {}; used {}",
            each(&|game| game.carried),
            each(&|game| game.used)
        );
        println!(
            "a game: {:.1} sticks of dynamite blew, taking {:.1} planks; the most in one game {} sticks and {} planks",
            mean(&|game| game.blasts as f32),
            mean(&|game| game.planks_blown as f32),
            games.iter().map(|game| game.blasts).max().unwrap_or(0),
            games.iter().map(|game| game.planks_blown).max().unwrap_or(0),
        );
        println!(
            "a game: {:.1} deaths on land dropped {:.1} sapphires and amethysts between them, of which {:.1} were picked up and {:.1} went with a bridge",
            mean(&|game| game.spilt[0] as f32),
            mean(&|game| game.spilt[1] as f32),
            mean(&|game| game.spilt[2] as f32),
            mean(&|game| game.spilt[3] as f32),
        );
        println!(
            "a game: a bus ran {:.2} over ({:.2} of them zombies, {:.2} of the players with a fight on), killing {:.2}; {:.2} blows landed on someone lying flat, and one of them set off to finish someone lying there {:.2} times; nobody run over in {} games of {}, the most in one game {}",
            mean(&|game| game.squashes[0] as f32),
            mean(&|game| game.squashed_zombies as f32),
            mean(&|game| game.squashed_fighting as f32),
            mean(&|game| game.squashes[1] as f32),
            mean(&|game| game.squashes[2] as f32),
            mean(&|game| game.preyed as f32),
            games.iter().filter(|game| game.squashes[0] == 0).count(),
            games.len(),
            games.iter().map(|game| game.squashes[0]).max().unwrap_or(0),
        );
        println!(
            "a game: {:.1} times one of them stepped into where a bus sweeps, {:.1} of them with a bus due within a second and a half; {:.1} s in there between them",
            mean(&|game| game.lane.0 as f32),
            mean(&|game| game.lane.1 as f32),
            mean(&|game| game.lane.2),
        );
        println!(
            "a game: {:.2} times one set out to cut another team's way; {:.2} bridges brought down, at least one in {} games of {}; {:.2} of them fell with one before ten minutes and {:.2} after; {:.2} stepped off planks left in the middle of nowhere; {:.2} walked off an edge",
            mean(&|game| game.cuts as f32),
            mean(&|game| game.cascades as f32),
            games.iter().filter(|game| game.cascades > 0).count(),
            games.len(),
            mean(&|game| game.with_bridge.0 as f32),
            mean(&|game| game.with_bridge.1 as f32),
            mean(&|game| game.stepped_off as f32),
            mean(&|game| game.walked_off as f32),
        );
        let mut fullest = [0u32; 6];
        for game in &games {
            fullest[game.fullest.min(5)] += 1;
        }
        println!("the most kinds on one of them at once, by how many games (none to five): {fullest:?}");
        tell_habits(&games);
    }

    /// Says how the computer's players of `games` spent them
    /// ([`Habits`]): where they were and what they were about before the
    /// anchors broke, as shares of the time each stood, all of them
    /// together and the keepers and those that go out apart; and what
    /// came of it, a game.
    fn tell_habits(games: &[Chronicle]) {
        let sum = |pick: &dyn Fn(&Habits) -> [f32; 2]| {
            games.iter().fold([0.0f32; 2], |sum, game| {
                let add = pick(&game.habits);
                [sum[0] + add[0], sum[1] + add[1]]
            })
        };
        let of = |spent: [f32; 2], whole: [f32; 2]| {
            format!(
                "{:>3.0}% (keepers {:>3.0}%, those that go out {:>3.0}%)",
                100.0 * (spent[0] + spent[1]) / (whole[0] + whole[1]).max(1.0),
                100.0 * spent[0] / whole[0].max(1.0),
                100.0 * spent[1] / whole[1].max(1.0)
            )
        };
        let alive = sum(&|habits| habits.alive);
        println!("how they spent the first ten minutes, as shares of the time each stood:");
        for (what, spent) in [
            ("on its own island", sum(&|habits| habits.home)),
            ("  by its own pad", sum(&|habits| habits.pad)),
            ("  there for the pad", sum(&|habits| habits.collect)),
            ("  with nothing to do", sum(&|habits| habits.idle)),
            ("  on it, standing still", sum(&|habits| habits.still)),
            ("  still at a stand", sum(&|habits| habits.browsing)),
            ("out looking about", sum(&|habits| habits.scouting)),
            ("out on a bridge", sum(&|habits| habits.bridge)),
            ("in the middle", sum(&|habits| habits.middle)),
            ("on a smaller island", sum(&|habits| habits.sapphire)),
            ("on another team's", sum(&|habits| habits.enemy)),
        ] {
            println!("  {what:<24} {}", of(spent, alive));
        }
        println!(
            "  {:<24} {}",
            "on its own island, once that was joined to anything",
            of(sum(&|habits| habits.bridged_home), sum(&|habits| habits.bridged))
        );
        let pair: f32 = games.iter().map(|game| game.habits.pair).sum::<f32>().max(1.0);
        println!(
            "  a team's two, of the time both stood: both by their pad {:.0}%, both on their island {:.0}%",
            100.0 * games.iter().map(|game| game.habits.pair_pad).sum::<f32>() / pair,
            100.0 * games.iter().map(|game| game.habits.pair_home).sum::<f32>() / pair,
        );
        let many = games.len().max(1) as f32;
        let mean = |pick: &dyn Fn(&Habits) -> f32| games.iter().map(|game| pick(&game.habits)).sum::<f32>() / many;
        println!(
            "a game, before ten minutes: {:.1} raids set off on, {:.1} islands stepped onto; first there, someone hit {:.1}, the anchor hit {:.1}, on the pad {:.1}, none of them {:.1}",
            mean(&|habits| habits.raids as f32),
            mean(&|habits| habits.landings as f32),
            mean(&|habits| habits.first[0] as f32),
            mean(&|habits| habits.first[1] as f32),
            mean(&|habits| habits.first[2] as f32),
            mean(&|habits| habits.first[3] as f32),
        );
        // When the anchors that went to punches went, and how many games
        // saw none, one, two, three and all four go so.
        let broken: Vec<f32> = games.iter().flat_map(|game| game.anchor.iter().flatten().copied().filter(|at| *at < 599.0)).collect();
        let mut by_count = [0u32; ARENA_TEAMS + 1];
        for game in games {
            by_count[game.anchor.iter().filter(|at| at.is_some_and(|at| at < 599.0)).count()] += 1;
        }
        let mut sorted = broken.clone();
        sorted.sort_by(f32::total_cmp);
        let middle = sorted.get(sorted.len() / 2).copied().unwrap_or(0.0);
        println!(
            "a game: {:.2} anchors broken by punches before ten minutes (games with none to four: {by_count:?}), the first of them at {:.0} s on average and half of them all by {middle:.0} s; the anchors had lost {:.0} of {} to {:.1} punches",
            broken.len() as f32 / many,
            games
                .iter()
                .filter_map(|game| game.anchor.iter().flatten().copied().filter(|at| *at < 599.0).min_by(f32::total_cmp))
                .sum::<f32>()
                / by_count[1..].iter().sum::<u32>().max(1) as f32,
            mean(&|habits| habits.anchor_lost),
            ANCHOR_HEALTH * ARENA_TEAMS as f32,
            mean(&|habits| habits.anchor_hits as f32),
        );
        let bridges: Vec<String> = (0..7).map(|kind| format!("{:.2}", mean(&|habits| habits.bridges[kind] as f32))).collect();
        println!(
            "a game, bridges begun: home to the middle, home to a smaller island, the middle to a smaller island, the middle to another team's, a smaller island to another team's, home to another team's, anything else: {}",
            bridges.join(", ")
        );
        println!(
            "a game: building quickened {:.2} steps, all four teams together; {:.2} leap potions bought, {:.2} drunk",
            mean(&|habits| habits.build_speed as f32),
            mean(&|habits| habits.leaps.0 as f32),
            mean(&|habits| habits.leaps.1 as f32),
        );
    }

    /// A game with people in `people` seats who do nothing but stand where
    /// `stands` puts each, and the computer in the rest: the fight just
    /// begun.
    fn with_people(seed: u64, people: &[usize], stands: impl Fn(usize) -> Vec3) -> Game {
        let mut game = Game::new(4, Entity::PLACEHOLDER, seed);
        for &seat in people {
            game.put(seat, Entity::from_bits(seat as u64 + 1), person(seat as u64 + 1), false);
        }
        fill_with_bots(&mut game);
        for seat in 0..ARENA_SEATS {
            game.stand(seat);
        }
        for &seat in people {
            game.at[seat] = stands(seat);
        }
        game.events.clear();
        game.stage = ArenaStage::Playing;
        game.left = PLAY_SECS;
        game
    }

    /// Plays `game` on a thirtieth of a second, with the people in it put
    /// back where `stands` has them each time they stand again.
    fn step_with_people(game: &mut Game, stands: &impl Fn(usize) -> Vec3) {
        game.step(1.0 / 30.0);
        for event in std::mem::take(&mut game.events) {
            if let Event::Land(seat) = event {
                game.at[seat] = stands(seat);
            }
        }
    }

    #[test]
    fn a_team_of_people_is_raided_no_more_than_its_share_and_its_anchor_left_alone_at_first() {
        use roundtown_net::bots::Goal;
        // Two people on team 0 who only stand in the middle of their
        // island: the worst defenders there could be.
        let home = |seat: usize| {
            let (at, _) = arrival(Venue::Arena, seat);
            at.with_y(ISLAND_TOP)
        };
        let (mut on_people, mut on_bots) = (0u32, 0u32);
        for seed in [3, 8, 21] {
            let mut game = with_people(seed, &[0, 4], home);
            assert_eq!((game.people(), game.squad.bot(0).is_none(), game.squad.bot(1).is_some()), (2, true, true));
            let mut goals = [Goal::Idle; ARENA_SEATS];
            let mut first_raid_on_people = None;
            while game.playing() && game.elapsed < 600.0 {
                step_with_people(&mut game, &home);
                let now = game.elapsed;
                for seat in 0..ARENA_SEATS {
                    let Some(bot) = game.squad.bot(seat) else {
                        continue;
                    };
                    if let Goal::Raid(victim) = bot.goal()
                        && goals[seat] != bot.goal()
                    {
                        if victim == 0 {
                            on_people += 1;
                            first_raid_on_people.get_or_insert(now);
                        } else {
                            on_bots += 1;
                        }
                    }
                    goals[seat] = bot.goal();
                }
                // Nobody comes for them before raids begin, and their
                // anchor is not touched before anchors are fair game:
                // each of the computer's has its own time for both since
                // 2026-11-15, the earliest of them a little over a minute
                // and a half in for a raid (the keenest kind, half a
                // minute sooner than most, and thrown early at that) and
                // two minutes and a half for an anchor (one as set on
                // anchors as they come). 150 and 240 s for all of them
                // until then.
                if now < 100.0 {
                    assert!(first_raid_on_people.is_none(), "seed {seed}: raided at {now} s");
                }
                if now < 145.0 {
                    assert_eq!(game.anchors[0], ANCHOR_HEALTH, "seed {seed}: their anchor hit at {now} s");
                }
            }
            println!(
                "seed {seed}: the people first raided at {first_raid_on_people:?}; their anchor {} at the end, in play {:?}",
                game.anchors[0],
                game.teams_in_play()
            );
        }
        // Over the three games, the raids on them are no more than about
        // a team's share of all of them.
        println!("raids on the people {on_people}, on the computer's teams {on_bots}");
        assert!(on_people > 0, "nobody ever came for them");
        assert!(on_people as f32 <= 0.45 * (on_people + on_bots) as f32, "{on_people} of {} raids were on the people", on_people + on_bots);
    }

    #[test]
    fn a_person_s_bot_teammate_comes_to_the_pad_for_what_lies_there_and_is_off_again() {
        use roundtown_net::bots::{Goal, Role};
        use roundtown_net::nav::Region;
        use roundtown_net::{ARENA_PADS, PAD_TOP};
        // A person in seat 0 who stands on their pad for the first twenty
        // seconds and in the middle of their island after, never going
        // near the pad again; the computer in seat 4 beside them. Seven
        // minutes of it, three times over.
        let (pad, island) = (ARENA_PADS[0], ARENA_ISLANDS[0]);
        let dt = 1.0 / 30.0;
        let (mut went_out, mut went_raiding, mut fetched) = (0, 0, 0);
        for seed in [9, 2, 5] {
            let mut game = with_people(seed, &[0], |_| Vec3::new(pad.x, PAD_TOP, pad.y));
            let (mut armed, mut raided, mut fetching) = (false, false, false);
            let mut given = [0; 4];
            // How long it has been by the pad this time, the longest it
            // ever was, how long in all, and how many times it came; and
            // how long it was off its own island.
            let (mut stay, mut longest, mut by_the_pad, mut visits, mut away) = (0.0f32, 0.0f32, 0.0f32, 0u32, 0.0f32);
            let mut alive_for = 0.0f32;
            while game.playing() && game.elapsed < 420.0 {
                let now = game.elapsed;
                let stands = |_: usize| {
                    if now < 20.0 {
                        Vec3::new(pad.x, PAD_TOP, pad.y)
                    } else {
                        Vec3::new(0.0, ISLAND_TOP, 130.0)
                    }
                };
                game.at[0] = stands(0);
                step_with_people(&mut game, &stands);
                let bot = game.squad.bot(4).expect("the computer's");
                // Once it has thought at all, it is the one that keeps, as
                // far as who goes for whom is concerned.
                if now > 2.0 {
                    assert_eq!(bot.role(), Role::Keeper, "at {now} s");
                }
                raided |= matches!(bot.goal(), Goal::Raid(_));
                fetching |= matches!(bot.goal(), Goal::Fetch(..));
                // A sword is lost with a death: had at any time, it was
                // bought.
                armed |= game.outfits.kit(4).sword > 0;
                // While the person is on the pad, the bot is not there for
                // what it makes: that is theirs.
                if now < 19.0 && now > 2.0 {
                    given = game.purses[0];
                    assert_ne!(bot.goal(), Goal::Collect, "seed {seed}: at {now} s it is at the pad with them");
                    assert_eq!(game.purses[4], [0; 4], "seed {seed}: at {now} s the bot holds {:?}", game.purses[4]);
                }
                if !game.alive(4) || now < 30.0 {
                    stay = 0.0;
                    continue;
                }
                alive_for += dt;
                let at = game.at[4].xz();
                if at.distance(pad) < 2.0 {
                    if stay == 0.0 {
                        visits += 1;
                    }
                    stay += dt;
                    by_the_pad += dt;
                    longest = longest.max(stay);
                } else {
                    stay = 0.0;
                }
                if at.distance(island) > ARENA_ISLAND_RADIUS + 1.0 {
                    away += dt;
                }
            }
            println!(
                "seed {seed}: the bot beside a person is {}: by the pad {visits} times, {by_the_pad:.0} s of {alive_for:.0} in all and {longest:.1} s at the longest; off its island {away:.0} s; raided {raided}, fetched {fetching}; kit {:?}, upgrades {:?}",
                game.squad.bot(4).unwrap().persona().name,
                game.outfits.kit(4),
                game.outfits.teams[0],
            );
            assert!(given[0] >= 7 && given[1] >= 1, "seed {seed}: the person on the pad was given what it made: {given:?}");
            // It comes for what lies there, again and again, and is never
            // there long: a few seconds at the most, and a small part of
            // its time in all (until 2026-11-13 it stood there whenever it
            // was saving for anything, most of the game).
            assert!(visits >= 6, "seed {seed}: it came to the pad {visits} times in seven minutes");
            assert!(longest < 5.0, "seed {seed}: it stood by the pad for {longest} s");
            assert!(by_the_pad < 0.15 * alive_for, "seed {seed}: by the pad {by_the_pad} s of {alive_for}");
            // With no bridge built by the person, it has collected for
            // one, bought it and laid it; and it has a sword.
            assert!(game.nav.joined(Region::Team(0), Region::Middle), "seed {seed}: no bridge to the middle by seven minutes");
            assert!(armed, "seed {seed}: it never had a sword in seven minutes");
            went_out += u32::from(away > 0.2 * alive_for);
            went_raiding += u32::from(raided);
            fetched += u32::from(fetching);
        }
        // And it is out and about: off its island for a good part of the
        // time, out to the pads in the middle and on the smaller islands,
        // and off on a raid of its own, where until 2026-11-13 it never
        // went on one by itself.
        assert!(went_out >= 2, "off its island for a fifth of the time in {went_out} games of three");
        assert!(fetched >= 2, "it went out for amethyst or sapphire in {fetched} games of three");
        assert!(went_raiding >= 1, "it went on a raid in {went_raiding} games of three");
    }
    /// The buses going steadily round their lanes from the fight's first
    /// moment, each from somewhere well clear of the crossings, for a test
    /// to put people in front of ([`in_front_of`]).
    fn steady_buses() -> Buses {
        Buses(std::array::from_fn(|bus| Leg {
            from: 0.0,
            round: 100.0 + 7.0 * bus as f64,
            speed: BUS_SPEED,
            gathers: 0.0,
        }))
    }

    /// Where someone stands to have bus `bus`'s front come to them `secs`
    /// seconds after `clock`, the buses going steadily ([`steady_buses`]),
    /// `across` metres to the right of the line down its middle: on the
    /// road.
    fn in_front_of(bus: usize, clock: f64, secs: f64, across: f32) -> Vec3 {
        let at = steady_buses().at(bus, clock + secs).world(Vec2::new(across, BUS_HALF_LENGTH + BODY_RADIUS + 0.05));
        Vec3::new(at.x, BUS_ROAD, at.y)
    }

    #[test]
    fn a_bus_runs_over_whoever_the_game_moves_and_they_lie_flat_hurt_and_unknocked_until_they_are_up() {
        // Two people of one team who only stand where they are put: one
        // whose device has gone, and one whose device is there to say what
        // happens to them. Both in the lane, where a bus is on them in a
        // fifth of a second; and one of the computer's beside them.
        let stands = |seat: usize| in_front_of(0, 0.0, 0.2, if seat == 0 { 0.9 } else { -0.9 });
        let mut game = with_people(31, &[0, 4], stands);
        game.buses = Some(steady_buses());
        game.away[0] = true;
        let bot_at = in_front_of(0, 0.0, 0.2, 0.0);
        game.squad.stand(1, bot_at, 0.0, 0.0);
        game.at[1] = bot_at;
        assert_eq!(game.flattened(), Flattened::default());
        for _ in 0..12 {
            step_with_people(&mut game, &stands);
        }
        // The one the game moves and the one nobody moves are flat, with
        // the bus's 20 off them; the one whose device is there is not the
        // game's to run over.
        assert!(game.flat[0] > 0.0 && game.flat[1] > 0.0, "{:?}", &game.flat[..ARENA_SEATS]);
        assert_eq!(game.flat[4], 0.0);
        assert_eq!((game.health[0], game.health[1], game.health[4]), (HEALTH - SQUASH_DAMAGE, HEALTH - SQUASH_DAMAGE, HEALTH));
        assert_eq!(game.squashes, [2, 0, 0]);
        let sent = game.flattened();
        assert_eq!(sent.0.len(), 2);
        let until = sent.until(1).expect("seat 1 lies flat");
        assert!((until - (f64::from(game.elapsed) + f64::from(game.flat[1]))).abs() < 0.1, "{until} at {}", game.elapsed);
        assert_eq!(sent.until(4), None);
        // Flat, the computer's player lies where it was, on the road.
        let lay = game.at[1];
        assert!(lay.xz().distance(bot_at.xz()) < 2.0 && lay.y.abs() < 0.3, "{lay}");
        // Hit as it lies there, on the run: the punch's damage, no knock,
        // and no less time to lie.
        let (health, left) = (game.health[1], game.flat[1]);
        let from = Spot::of(lay + Vec3::X, Quat::IDENTITY);
        game.wound(1, 4, from, true);
        assert_eq!(game.health[1], health - DAMAGE);
        assert_eq!(game.flat[1], left);
        // And the person whose device has gone: hurt, and their device,
        // were it there, told of no knock.
        game.wound(0, 2, from, true);
        assert_eq!(game.health[0], HEALTH - SQUASH_DAMAGE - DAMAGE);
        assert!(!game.events.iter().any(|event| matches!(event, Event::Knocked { .. })), "{:?}", game.events);
        assert_eq!(game.squashes[2], 2);
        for _ in 0..60 {
            step_with_people(&mut game, &stands);
        }
        assert!(game.at[1].distance(lay) < 1e-3, "moved {} m", game.at[1].distance(lay));
        assert!((game.flat[1] - (left - 2.0)).abs() < 0.05);
        // No bus runs over someone who is flat already: with another
        // bus's front on them, they lie as they lay.
        let clock = game.bus_clock.expect("the buses have been moved");
        let again = in_front_of(1, clock, 0.2, 0.3);
        game.at[0] = again;
        let stands_again = |_: usize| again;
        let health = game.health[0];
        for _ in 0..12 {
            step_with_people(&mut game, &stands_again);
        }
        assert_eq!(game.squashes[0], 2);
        assert!(game.flat[0] > 0.0 && game.health[0] >= health);
        // Up again after ten seconds from when it ran them over, both.
        while game.elapsed < 10.0 {
            step_with_people(&mut game, &stands_again);
        }
        assert!(game.flat[0] > 0.0 && game.flat[1] > 0.0, "{} s in: {:?}", game.elapsed, &game.flat[..2]);
        while game.elapsed < 10.7 {
            step_with_people(&mut game, &stands_again);
        }
        assert_eq!((game.flat[0], game.flat[1]), (0.0, 0.0));
        assert_eq!(game.flattened(), Flattened::default());
        // And then they can be run over again: by whichever bus is next
        // on them.
        let clock = game.bus_clock.expect("the buses have been moved");
        let once_more = in_front_of(0, clock, 0.2, -0.4);
        game.at[0] = once_more;
        let stands_once_more = |_: usize| once_more;
        for _ in 0..12 {
            step_with_people(&mut game, &stands_once_more);
        }
        assert_eq!(game.squashes[0], 3);
        assert!(game.flat[0] > 9.0);
        // With no more health than the bus takes, it kills them: said in
        // the chat, and they are dead, not flat.
        game.flat[0] = 0.0;
        game.health[0] = 15.0;
        game.events.clear();
        game.run_over(0);
        assert_eq!(said(&mut game), vec![Feed::RunOver { seat: 0 }]);
        assert!(!game.alive(0) && game.flat[0] == 0.0 && game.dead_for[0].is_some());
        assert_eq!(game.squashes[1], 1);
        // A shield keeps its share of the bus off, as of anything.
        game.outfits.kits[4].shield = 3;
        game.health[4] = HEALTH;
        game.run_over(4);
        assert!((game.health[4] - (HEALTH - SQUASH_DAMAGE * (1.0 - SHIELD_SHARE[3]))).abs() < 1e-3, "{}", game.health[4]);
    }

    #[test]
    fn someone_riding_a_bus_is_where_the_bus_has_taken_them() {
        // A place on the first of Bridge Wars' buses, 2 m behind its
        // windscreen on its floor: in the world, on the lane, by how the
        // game's buses go.
        let clock = 7.5;
        let mut log = BusLog::default();
        log.take(&steady_buses());
        let aboard = Spot::of(Vec3::new(0.5, 0.47, -4.0), Quat::IDENTITY).aboard(Some(FIRST_BUS));
        assert_eq!(riding(&aboard), Some(0));
        let at = stands(&aboard, clock, &log);
        let bus = steady_buses().at(0, clock);
        assert!(bus.local(at.xz()).distance(Vec2::new(0.5, 4.0)) < 0.02, "{at}");
        assert!((at.xz().length() - LANES[0].out).abs() < 4.5 && (at.y - 0.47).abs() < 0.01);
        // The others are the next numbers, the fourth the last; one of
        // the town's buses is none of the game's, nor is anyone on foot.
        assert_eq!(riding(&aboard.aboard(Some(FIRST_BUS + 1))), Some(1));
        assert_eq!(riding(&aboard.aboard(Some(FIRST_BUS + 3))), Some(3));
        assert_eq!(riding(&aboard.aboard(Some(0))), None);
        assert_eq!(riding(&aboard.aboard(Some(FIRST_BUS + 4))), None);
        let on_the_fourth = stands(&aboard.aboard(Some(FIRST_BUS + 3)), clock, &log);
        assert!((on_the_fourth.xz().length() - LANES[3].out).abs() < 4.5);
        let afoot = Spot::of(Vec3::new(10.0, 0.25, -3.0), Quat::IDENTITY);
        assert_eq!(riding(&afoot), None);
        assert!(stands(&afoot, clock, &log).distance(Vec3::new(10.0, 0.25, -3.0)) < 0.01);
        // Before the game has any buses, where they said.
        assert!(stands(&aboard, clock, &BusLog::default()).distance(aboard.translation()) < 0.01);
    }

    #[test]
    fn the_buses_stop_short_of_a_crossing_someone_stands_on_and_go_over_it_once_they_are_off() {
        // A person standing on the first crossing, from the start of a
        // fight: every bus comes to stand short of it, and none on it, in
        // well under a lap of the longest lane; the room is sent each bus
        // as it changes. Off it, they go on.
        let crossing = Vec2::new(CROSSINGS[0].cos(), -CROSSINGS[0].sin()) * CROSSING_OUT;
        let on_it = Vec3::new(crossing.x, BUS_ROAD, crossing.y);
        let mut game = with_people(5, &[0], |_| on_it);
        assert_eq!(crossing_at(on_it), Some(0));
        let stands = |_: usize| on_it;
        while game.elapsed < 40.0 {
            step_with_people(&mut game, &stands);
        }
        let (buses, clock) = (game.buses.expect("the buses go round"), game.bus_clock.expect("by the clock"));
        for bus in 0..BUSES {
            assert_eq!(buses.going(bus, clock).1, 0.0, "bus {bus} is still going");
            let near = buses.at(bus, clock).at.distance(crossing);
            assert!(near < 25.0, "bus {bus} stands {near} m from the crossing");
        }
        assert_eq!(game.flattened(), Flattened::default());
        // Off it, over to the far side of the road: they go on.
        let off_it = Vec3::new(crossing.x * 1.4, ISLAND_TOP, crossing.y * 1.4);
        let stands_off = |_: usize| off_it;
        game.at[0] = off_it;
        while game.elapsed < 50.0 {
            step_with_people(&mut game, &stands_off);
        }
        let (buses, clock) = (game.buses.expect("the buses go round"), game.bus_clock.expect("by the clock"));
        for bus in 0..BUSES {
            assert_eq!(buses.going(bus, clock).1, BUS_SPEED, "bus {bus}");
        }
    }
}
