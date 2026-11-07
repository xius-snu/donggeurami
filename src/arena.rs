//! Bridge Wars: the game played on the PvP map ([`Venue::Arena`], Hajun's
//! `pvptownmap.glb`), chosen from the game selector (`games`; Hajun,
//! the night of 2026-11-07: "the player is teleported to ... the pvp map's (waitingroom
//! box above the map until 4 ppl is queued, which at that point it will
//! count down from 5 and then teleport each player to each of the 4
//! island) ... if there is no player joining for 10sec, fill it with bots.
//! ... just make it so the pvpmap game (small) timer of 10 minute (tap to
//! skip) and after it, the players go back to the main roundtownmap.
//! pvpmap doesnt need buses and doesnt need npcs ... in the pvpmap game it
//! will switch to top down cam"). For 2026-11-07 the map was the town
//! itself, and everything the town did there, it does here: the arrivals on
//! the islands, the void under the map and the punch's knockback.
//!
//! Choosing it, a black screen comes down (`games`), and you are stood in
//! the waiting room: Hajun's glass box floating 100 m over the middle of
//! the map, the object called [`WAITING_ROOM`] in `pvptownmap.glb` (their
//! export of 2026-11-07 23:21: 30 m square, its floor a metre thick with
//! its top at 100.5 m, its walls 1.5 m thick and 5 m high, open to the
//! sky, in `glass` at a tenth), found by name once the map has spawned
//! ([`find_waiting_room`], [`WaitingRoom`]) and stood on as any of the
//! map is, with the camera fixed straight down over you
//! (`OrbitCamera::top_down`) and the game's own buttons round the screen
//! (`hud::Layout::Arena`). The board top centre counts the players.
//! Nobody else can join yet, the game being this device's own: once
//! [`BOT_WAIT`] seconds have passed with nobody joining, the computer takes
//! the seats left ([`ARENA_SEATS`], four), and the board counts down
//! [`COUNTDOWN`] seconds, after which everyone is sent to the middle of the
//! island their seat says (`roundtown_net::arrival`), facing the middle of
//! the map down its bridge. Then [`PLAY_SECS`] run on the board, which a
//! tap on it cuts short, and everyone is back in the town, the camera as
//! it was.
//!
//! The fight (the night of 2026-11-07, Hajun: "make the ai, if there is
//! another ai/player within its visible radius, it will go towards them
//! and try to fight/punch them ... make punching deal damage, and show
//! healthbar for enemies. make default hp = 100 ... make damage numbers
//! appear on each hit. default damage = 12. so when health <0 u die too
//! ... 'You died. Respawning in 4...' -> 'You died. Respawning in
//! 3.5...' until it goes to 0 and then you respawn ... for falling make
//! it so u need to drop a bit more z to actually die ... make it running
//! rather than walking"): everyone runs there, at the file's
//! `arena.run_speed` (`Intent::sprint`), and everyone has [`Health`],
//! `arena.health` of it; a punch that lands and takes ([`Landed`]) takes
//! `arena.damage` off it, a number floating up off the body
//! ([`DamageNumber`]), everyone but you with a bar over their head
//! ([`BarOf`]) and you with yours top left (`hud`). At none, or
//! `arena.fall_to_die` below the map (`void_y`), a body is [`Dead`]: hidden
//! where it died, in nobody's way, for `arena.respawn_secs`, you with "You
//! died. Respawning in 4..." counting down in halves across the screen
//! ([`show_death`]), and then back in the middle of its island with its
//! health full, the camera over you there. A bot goes for whoever is
//! within `arena.hunt_radius` of it ([`hunt`], over its wandering), at a
//! run, and punches once in reach, no oftener than `arena.bot_punch_every`;
//! only while the game is on, never in the waiting room. Nothing is scored.
//!
//! Online you are in a room of your own on the map, as at home: the server
//! is asked to put you there (`Ask::Travel`) and back, and nothing of where
//! you stand goes to it meanwhile (`net`).
//!
//! Everything the game puts in the world or on the screen carries
//! [`InArena`] and goes when the game does. Your own body is the only thing
//! that comes back.

use std::f32::consts::TAU;

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use roundtown_net::bridge_wars::{BOT_WAIT, COUNTDOWN, PLAY_SECS};
use roundtown_net::{ARENA_SEATS, Ask as NetAsk, Member};

use crate::ai::{self, Steering};
use crate::builder::Fill;
use crate::drive::Driving;
use crate::hud::{self, TakesPress};
use crate::island::{self, Island, Islands, Land, Venue};
use crate::lobby::{self, Lobby, Seat, Whereabouts};
use crate::net::Online;
use crate::range::Sector;
use crate::ui::{fill, ui};
use crate::{
    Dead, Intent, Landed, OrbitCamera, PLAYER_HEIGHT, PLAYER_RADIUS, Player, PlayerJump, Punching, Shove,
    ThirdPersonCamera, View, WalkCycle, void_y,
};

/// What the waiting room is called in the map: Hajun's glass box over the
/// middle of it (2026-11-07). Everyone waits on its floor, which is read
/// off the map's shape under the middle of the box.
pub(crate) const WAITING_ROOM: &str = "waiting room";
/// How far from the middle of the room everyone stands in it, in metres,
/// facing the middle, each an eighth of the turn on from the last.
const WAIT_RING: f32 = 3.0;
/// How long the game's name is across the screen as you arrive, and
/// "Fight!" as the game begins, in seconds.
const NAME_SECS: f32 = 3.0;
const FIGHT_SECS: f32 = 2.0;
/// Sizes, as shares of the short side of the screen: the board's count,
/// countdown or time, small (Hajun: "(small) timer"), and the line under
/// it; the words across the screen while you are dead, and how far down
/// the screen they sit; someone else's health bar over their head, and
/// its rim; and a damage number.
const BOARD_TEXT: f32 = 4.8;
const CAPTION: f32 = 3.4;
const DEATH_TEXT: f32 = 5.0;
const DEATH_DOWN: f32 = 38.0;
const BAR_WIDTH: f32 = 7.0;
const BAR_HEIGHT: f32 = 1.1;
const BAR_RIM: f32 = 0.2;
const NUMBER_TEXT: f32 = 3.8;
/// How high over a body's feet its health bar stands, in metres, and a
/// damage number starts from and rises to over its while, in seconds.
const BAR_OVER: f32 = 2.2;
const NUMBER_FROM: f32 = 1.8;
const NUMBER_RISE: f32 = 1.6;
const NUMBER_SECS: f32 = 0.9;
/// How near a bot presses in on whoever it is going for before it slows
/// to a push, as a share of the punch's reach.
const PRESS_IN: f32 = 0.3;

/// Everything a game has put in the world or on the screen, to go with it.
#[derive(Component)]
pub(crate) struct InArena;

/// Something asked of Bridge Wars, by a tap or a click.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ask {
    /// Into the waiting room: from the game selector (`games`).
    Enter,
    /// On to the next thing, for trying the game out: the bots now, the
    /// countdown over now, or the game over now. A tap on the board.
    Skip,
}

/// What a tap on the board asks for.
#[derive(Component, Clone, Copy)]
struct AsksFor(Ask);

/// A body's health in a game of Bridge Wars: how much is left of what it
/// started with, the file's `arena.health`, and how long since it was last
/// hit, after which it comes back ([`regen`]). Yours is the bar under your
/// name (`hud`), everyone else's the bar over their head ([`BarOf`]).
#[derive(Component, Debug)]
pub(crate) struct Health {
    left: f32,
    full: f32,
    since_hit: f32,
}

impl Health {
    /// All of it, as a body starts and stands again.
    fn full() -> Self {
        let full = ui().arena.health;
        Self {
            left: full,
            full,
            since_hit: 0.0,
        }
    }

    /// How much is left, from 0 to 1.
    pub(crate) fn share(&self) -> f32 {
        (self.left / self.full).clamp(0.0, 1.0)
    }

    /// Takes `damage` off it, to nothing at the least.
    fn take(&mut self, damage: f32) {
        self.left = (self.left - damage).max(0.0);
        self.since_hit = 0.0;
    }

    /// Gives `amount` back, up to full: whether there was any to give.
    fn mend(&mut self, amount: f32) -> bool {
        if self.left >= self.full {
            return false;
        }
        self.left = (self.left + amount).min(self.full);
        true
    }

    /// None left: dead.
    fn gone(&self) -> bool {
        self.left <= 0.0
    }
}

/// How full a health bar is drawn, in steps of half a percent, so that
/// health coming back a little every frame does not have the screen laid
/// out again every frame.
fn bar_width(health: &Health) -> Val {
    Val::Percent((health.share() * 200.0).round() * 0.5)
}

/// A bot's fighting mind in Bridge Wars, over its wandering ([`hunt`]):
/// how long until it may punch again, in seconds.
#[derive(Component, Default)]
struct Hunter {
    cooldown: f32,
}

/// The four resources of Bridge Wars, down the right edge of the screen
/// under the bag (`hud`), in the order Hajun drew them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Resource {
    Wood,
    Gold,
    Diamond,
    Emerald,
}

/// How much of each resource you hold. Nothing gives any yet.
#[derive(Resource, Default)]
pub(crate) struct Resources {
    pub wood: u32,
    pub gold: u32,
    pub diamond: u32,
    pub emerald: u32,
}

impl Resources {
    pub(crate) fn of(&self, kind: Resource) -> u32 {
        match kind {
            Resource::Wood => self.wood,
            Resource::Gold => self.gold,
            Resource::Diamond => self.diamond,
            Resource::Emerald => self.emerald,
        }
    }
}

/// A game of Bridge Wars, from the waiting room to the end.
#[derive(Resource)]
struct Game {
    /// Everyone in it, in seat order: you first, then the bots. A seat is
    /// an island.
    seats: Vec<Member>,
    /// Their bodies, in the same order.
    bodies: Vec<Entity>,
    /// Your seat in the town's room, to go back to.
    town_seat: usize,
    stage: Stage,
    /// How the camera was turned and zoomed before it was fixed over you,
    /// to go back to; none if it was over you already.
    view: Option<View>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    /// In the waiting room: how long since the last to join did.
    Waiting { since_join: f32 },
    /// Full, counting down to go.
    Countdown { left: f32 },
    /// Everyone on their island, for this much longer.
    Playing { left: f32 },
}

/// The waiting room, as the map has it: the middle of the box across the
/// ground, and the top of it, under which its floor is found
/// ([`WaitingRoom::floor`]).
#[derive(Resource, Clone, Copy, Debug)]
pub(crate) struct WaitingRoom {
    centre: Vec2,
    top: f32,
}

impl WaitingRoom {
    /// Where the floor of the room is, under its middle: the highest of the
    /// map under the top of the box there.
    fn floor(&self, map: &Island) -> Option<f32> {
        map.floor(self.centre, self.top)
    }

    /// Where the player in `seat` stands in the room, its floor `floor`
    /// up: on a ring round its middle, facing in.
    fn spot(&self, floor: f32, seat: usize) -> Transform {
        let angle = seat as f32 * TAU / 8.0;
        let middle = Vec3::new(self.centre.x, floor, self.centre.y);
        let at = middle + Vec3::new(angle.sin(), 0.0, angle.cos()) * WAIT_RING;
        Transform::from_translation(at).looking_at(middle, Vec3::Y)
    }
}

/// The board top centre, and the words on it: the count, the countdown or
/// the time, and the line under it.
#[derive(Component)]
struct Board;
#[derive(Component)]
struct BoardText;
#[derive(Component)]
struct BoardCaption;

/// The words across the screen while you are dead, and the line of them.
#[derive(Component)]
struct DeathText;
#[derive(Component)]
struct DeathLine;

/// A health bar over someone else's head: whose, and its fill.
#[derive(Component)]
struct BarOf(Entity);
#[derive(Component)]
struct BarFill;

/// A number floating up off a body a punch landed on: where the body stood,
/// and how long ago.
#[derive(Component)]
struct DamageNumber {
    at: Vec3,
    age: f32,
}

/// Bridge Wars' reading of taps and the game they drive: ahead of your own
/// walking, and of the islands being shown, so that wherever it sends you
/// is drawn the same frame.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ArenaSystems;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Ask>()
        .init_resource::<Resources>()
        .add_observer(ask)
        .add_observer(find_waiting_room)
        .add_systems(
            Update,
            (play, show_board)
                .chain()
                .in_set(ArenaSystems)
                .before(crate::read_player_input)
                .before(island::show_where_i_am),
        )
        // The bots go for whoever is near, over their wandering, before
        // anyone moves; the fight's dead and its wounds are counted once
        // everyone has, and what is drawn over the world placed once the
        // camera is.
        .add_systems(
            Update,
            (
                hunt.after(ai::steer).after(crate::start_punch).before(crate::move_bodies),
                (reap, deal_damage, regen, respawn)
                    .chain()
                    .after(crate::move_bodies)
                    .before(crate::follow_camera),
                (show_death, place_bars, float_numbers).after(crate::follow_camera),
            ),
        );
}

// ------------------------------------------------------- the waiting room

/// Finds the waiting room in the map once it has spawned, by its name
/// ([`WAITING_ROOM`]): the box round every mesh of it, placed in the
/// world, for where its middle is and how high it reaches. Its floor is
/// read off the map's shape when it is needed, the box being open to the
/// sky and its walls' tops the highest of it.
fn find_waiting_room(
    ready: On<WorldInstanceReady>,
    lands: Query<&Venue, With<Land>>,
    children: Query<&Children>,
    names: Query<&Name>,
    boxes: Query<&Aabb>,
    placement: TransformHelper,
    mut commands: Commands,
) {
    // The map's own model, not a body's on it.
    if lands.get(ready.entity).ok() != Some(&Venue::Arena) {
        return;
    }
    let Some(room) = children
        .iter_descendants(ready.entity)
        .find(|&entity| names.get(entity).is_ok_and(|name| name.as_str() == WAITING_ROOM))
    else {
        warn!("{WAITING_ROOM:?} is not in the map: Bridge Wars' players will wait on their islands");
        return;
    };
    let (mut low, mut high) = (Vec3::INFINITY, Vec3::NEG_INFINITY);
    for entity in std::iter::once(room).chain(children.iter_descendants(room)) {
        let (Ok(aabb), Ok(place)) = (boxes.get(entity), placement.compute_global_transform(entity)) else {
            continue;
        };
        let (min, max) = (Vec3::from(aabb.min()), Vec3::from(aabb.max()));
        for corner in 0..8 {
            let corner = Vec3::new(
                if corner & 1 == 0 { min.x } else { max.x },
                if corner & 2 == 0 { min.y } else { max.y },
                if corner & 4 == 0 { min.z } else { max.z },
            );
            let placed = place.transform_point(corner);
            low = low.min(placed);
            high = high.max(placed);
        }
    }
    if low.x > high.x {
        warn!("{WAITING_ROOM:?} has no mesh to stand on");
        return;
    }
    let room = WaitingRoom {
        centre: ((low + high) * 0.5).xz(),
        top: high.y,
    };
    info!("Bridge Wars' waiting room: {} to {} across, {:.1} to {:.1} m up", low.xz(), high.xz(), low.y, high.y);
    commands.insert_resource(room);
}

// ---------------------------------------------------------------- the game

/// Whatever was tapped, if it asks for something: the board.
fn ask(click: On<Pointer<Click>>, buttons: Query<&AsksFor>, mut asks: MessageWriter<Ask>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(button) = buttons.get(click.entity) {
        asks.write(button.0);
    }
}

/// Where the player in `seat` waits for the game: in the waiting room, on
/// its floor, or, with no room in the map, on their island, where they
/// arrive for the game anyway.
fn waiting_spot(room: Option<&WaitingRoom>, map: &Island, seat: usize) -> Transform {
    match room.and_then(|room| Some((room, room.floor(map)?))) {
        Some((room, floor)) => room.spot(floor, seat),
        None => island::arrival(Venue::Arena, seat),
    }
}

/// Takes you into a game and moves it on: into the waiting room, the bots
/// in once nobody else has come, the countdown, everyone to their islands,
/// the time running, and at the end of it everyone back in the town. Asked
/// in before the map has loaded, it waits for it, under the black screen
/// (`games`).
#[allow(clippy::too_many_arguments)]
fn play(
    time: Res<Time>,
    mut asks: MessageReader<Ask>,
    game: Option<ResMut<Game>>,
    online: Res<Online>,
    lobby: Res<Lobby>,
    islands: Res<Islands>,
    room: Option<Res<WaitingRoom>>,
    me: Query<(Entity, &Seat), With<Player>>,
    in_arena: Query<Entity, With<InArena>>,
    mut bodies: Whereabouts,
    (mut orbit, mut driving): (ResMut<OrbitCamera>, ResMut<Driving>),
    assets: Res<AssetServer>,
    mut requests: MessageWriter<NetAsk>,
    mut asked_in: Local<bool>,
    mut commands: Commands,
) {
    let Ok((you, seat)) = me.single() else {
        asks.clear();
        return;
    };
    let mut skip = false;
    for &ask in asks.read() {
        match ask {
            Ask::Enter => *asked_in = true,
            Ask::Skip => skip = true,
        }
    }
    let room = room.as_deref().copied();
    let Some(mut game) = game else {
        if *asked_in && let Some(map) = islands.get(Venue::Arena) {
            *asked_in = false;
            let spot = waiting_spot(room.as_ref(), map, 0);
            let game = start(&mut commands, you, seat.0, spot, &lobby, &online, &mut bodies, &mut orbit, &mut driving, &mut requests);
            commands.insert_resource(game);
        }
        return;
    };
    *asked_in = false;
    let game = &mut *game;
    let dt = time.delta_secs();
    match game.stage {
        Stage::Waiting { since_join } => {
            // Nobody else is coming. Once nobody has for a while, the
            // computer takes every seat left, all at once.
            let since_join = if skip { BOT_WAIT } else { since_join + dt };
            if since_join >= BOT_WAIT && game.seats.len() < ARENA_SEATS {
                let Some(map) = islands.get(Venue::Arena) else {
                    return;
                };
                while game.seats.len() < ARENA_SEATS {
                    lobby::seat_ai(&mut game.seats, &lobby.seats);
                    let seat = game.bodies.len();
                    let body = lobby::spawn_player_at(
                        &mut commands,
                        &assets,
                        &game.seats[seat],
                        seat,
                        Venue::Arena,
                        false,
                        waiting_spot(room.as_ref(), map, seat),
                    );
                    commands.entity(body).insert((InArena, Health::full(), Hunter::default()));
                    game.bodies.push(body);
                }
                info!("Bridge Wars: nobody joined in {BOT_WAIT} s, so the computer takes the seats left");
            }
            game.stage = if game.seats.len() < ARENA_SEATS {
                Stage::Waiting { since_join }
            } else {
                Stage::Countdown { left: COUNTDOWN }
            };
        }
        Stage::Countdown { left } if left > dt && !skip => game.stage = Stage::Countdown { left: left - dt },
        Stage::Countdown { .. } => {
            for (seat, &body) in game.bodies.iter().enumerate() {
                lobby::send(&mut bodies, &mut orbit, body, Venue::Arena, seat);
            }
            hud::announce(&mut commands, None, &ui().words.arena.fight, hud::HEADLINE, FIGHT_SECS).insert(InArena);
            info!("Bridge Wars: everyone to their islands, {PLAY_SECS} s to play");
            game.stage = Stage::Playing { left: PLAY_SECS };
        }
        Stage::Playing { left } if left > dt && !skip => game.stage = Stage::Playing { left: left - dt },
        // Time's up: back to the town, alive, and the camera as it was.
        Stage::Playing { .. } => {
            let (town_seat, view) = (game.town_seat, game.view);
            commands.remove_resource::<Game>();
            for thing in &in_arena {
                commands.entity(thing).try_despawn();
            }
            commands.entity(you).insert(Seat(town_seat)).remove::<(Health, Dead)>();
            lobby::send(&mut bodies, &mut orbit, you, Venue::Town, town_seat);
            match view {
                Some(view) => orbit.back_to(view),
                None => *orbit = OrbitCamera::behind(&island::arrival(Venue::Town, town_seat)),
            }
            if online.is() {
                requests.write(NetAsk::Travel(Venue::Town));
            }
            info!("Bridge Wars: over, back to the town");
        }
    }
}

/// A new game, with you at `spot`, in the waiting room, your health full:
/// the camera fixed over you, the board up, the game's name across the
/// screen, and online the server asked to put you on the map, in a room
/// of your own. Getting there gets you out of the car (`drive::toggle`,
/// on the change of island), and the camera goes back to how it was
/// before the car had it, not to the car's.
#[allow(clippy::too_many_arguments)]
fn start(
    commands: &mut Commands,
    you: Entity,
    town_seat: usize,
    spot: Transform,
    lobby: &Lobby,
    online: &Online,
    bodies: &mut Whereabouts,
    orbit: &mut OrbitCamera,
    driving: &mut Driving,
    requests: &mut MessageWriter<NetAsk>,
) -> Game {
    let before_the_car = driving.take_view();
    let view = orbit.top_down().or(before_the_car);
    commands.entity(you).insert((Seat(0), Health::full())).remove::<Dead>();
    lobby::put(bodies, orbit, you, Venue::Arena, spot);
    spawn_board(commands);
    hud::announce(commands, None, &ui().words.arena.name, hud::HEADLINE, NAME_SECS).insert(InArena);
    if online.is() {
        requests.write(NetAsk::Travel(Venue::Arena));
    }
    info!("Bridge Wars: into the waiting room");
    Game {
        seats: vec![lobby.mine()],
        bodies: vec![you],
        town_seat,
        stage: Stage::Waiting { since_join: 0.0 },
        view,
    }
}

// --------------------------------------------------------------- the fight

/// Sends each bot for whoever is nearest within its sight, while the game
/// is on: at a run up to them, over whatever walk it was on, pressing in
/// on them once there, and a punch whenever they are in its reach and it
/// may throw one, which knocks them back as yours does ([`Shove::knock`])
/// and lands ([`Landed`]) for the damage to be dealt. It sees no further
/// than the file's `arena.hunt_radius`, and the dead not at all; walking
/// off its island it is kept from, as ever (`StaysAshore`), so a target
/// across the void is stood at the edge for. With less than the file's
/// `arena.flee_below` of its health it runs away from them instead
/// ([`flight`]), until its health has come back ([`regen`]).
#[allow(clippy::too_many_arguments)]
fn hunt(
    time: Res<Time>,
    game: Option<Res<Game>>,
    islands: Res<Islands>,
    mut bodies: Query<(
        Entity,
        &Transform,
        &Venue,
        &mut Intent,
        &mut Shove,
        &mut PlayerJump,
        Option<&mut Hunter>,
        Option<&mut Steering>,
        Option<&Health>,
        Has<Dead>,
    )>,
    mut landed: MessageWriter<Landed>,
    mut commands: Commands,
) {
    if !game.is_some_and(|game| matches!(game.stage, Stage::Playing { .. })) {
        return;
    }
    let file = ui();
    let (tuning, melee) = (file.arena, file.melee);
    let sector = Sector::of(&melee);
    let dt = time.delta_secs();
    let alive: Vec<(Entity, Vec3)> = bodies
        .iter()
        .filter(|(_, _, venue, .., dead)| **venue == Venue::Arena && !dead)
        .map(|(body, at, ..)| (body, at.translation))
        .collect();
    let hunters: Vec<Entity> = bodies
        .iter()
        .filter(|(_, _, venue, _, _, _, hunter, _, _, dead)| **venue == Venue::Arena && hunter.is_some() && !dead)
        .map(|(body, ..)| body)
        .collect();
    for bot in hunters {
        // What the bot does this frame, with its own body in hand; the
        // punch is thrown once that is let go of.
        let punch = {
            let Ok((_, body, _, mut intent, _, _, Some(mut hunter), steering, health, _)) = bodies.get_mut(bot) else {
                continue;
            };
            hunter.cooldown = (hunter.cooldown - dt).max(0.0);
            let here = body.translation;
            let Some((target, there)) = alive
                .iter()
                .filter(|&&(other, at)| {
                    other != bot
                        && at.xz().distance(here.xz()) <= tuning.hunt_radius
                        && (at.y - here.y).abs() <= PLAYER_HEIGHT * 2.0
                })
                .min_by(|a, b| a.1.distance_squared(here).total_cmp(&b.1.distance_squared(here)))
                .copied()
            else {
                continue;
            };
            if let Some(mut steering) = steering {
                steering.stop();
            }
            let off = (there - here).xz();
            let toward = Vec3::new(off.x, 0.0, off.y).normalize_or_zero();
            // Nearly beaten, it runs for it rather than fights.
            if health.is_some_and(|health| health.share() < tuning.flee_below) {
                intent.walk = islands.get(Venue::Arena).map_or(-toward, |map| flight(map, here, toward));
                continue;
            }
            intent.walk = toward * if off.length() > melee.reach { 1.0 } else { PRESS_IN };
            let facing = body.forward().xz().normalize_or(Vec2::NEG_Y);
            let in_reach = (there.y - here.y).abs() <= PLAYER_HEIGHT && sector.holds(facing, off, PLAYER_RADIUS);
            if !in_reach || hunter.cooldown > 0.0 {
                continue;
            }
            hunter.cooldown = tuning.bot_punch_every;
            let running = intent.walk.length() >= 0.5;
            (target, here.xz(), there.xz(), running)
        };
        let (target, from, at, running) = punch;
        commands.entity(bot).insert(Punching::next());
        if let Ok((_, _, _, _, mut shove, mut jump, ..)) = bodies.get_mut(target)
            && shove.knock(&mut jump, from, at, running)
        {
            landed.write(Landed { by: bot, victim: target });
        }
    }
}

/// Which way a bot at `here` on `map` runs from someone `toward` it: straight
/// away, or failing dry land that way for [`FLIGHT`] metres, the nearest
/// way round to it that has it, to either side and then across; with no
/// way at all, it stands its ground. A run off the island is kept from
/// anyway (`StaysAshore`); this keeps the bot from standing at the edge
/// being hit.
fn flight(map: &Island, here: Vec3, toward: Vec3) -> Vec3 {
    let away = -toward;
    [0.0, 0.6, -0.6, 1.2, -1.2, 1.57, -1.57]
        .into_iter()
        .map(|turn: f32| Quat::from_rotation_y(turn) * away)
        .find(|&heading| ai::dry_all_the_way(map, here, heading, FLIGHT))
        .unwrap_or(Vec3::ZERO)
}

/// How far ahead a bot running away looks for dry land, in metres.
const FLIGHT: f32 = 3.0;

/// Whoever has fallen far enough into the void is dead ([`void_y`]): you,
/// or a bot knocked off its island.
fn reap(fallen: Query<(Entity, &Transform, &Venue), (With<WalkCycle>, Without<Dead>)>, mut commands: Commands) {
    let void = void_y();
    for (body, transform, &venue) in &fallen {
        if venue == Venue::Arena && transform.translation.y < void {
            die(&mut commands, body);
            info!("{body} fell into the void");
        }
    }
}

/// Deals every punch that landed ([`Landed`]) its damage, while the game is
/// on: the file's `arena.damage` off the victim's health, a number floating
/// up off them, and death at none.
fn deal_damage(
    mut hits: MessageReader<Landed>,
    game: Option<Res<Game>>,
    mut victims: Query<(&Transform, &mut Health), Without<Dead>>,
    mut commands: Commands,
) {
    let fighting = game.is_some_and(|game| matches!(game.stage, Stage::Playing { .. }));
    for hit in hits.read() {
        if !fighting {
            continue;
        }
        let Ok((at, mut health)) = victims.get_mut(hit.victim) else {
            continue;
        };
        let damage = ui().arena.damage;
        health.take(damage);
        spawn_number(&mut commands, at.translation, damage);
        if health.gone() {
            die(&mut commands, hit.victim);
            info!("{} killed {}", hit.by, hit.victim);
        }
    }
}

/// Gives everyone alive their health back, a little a second, once nobody
/// has hit them for a while: the file's `arena.regen_per_sec` after
/// `arena.regen_after`. The while is counted without marking the health
/// changed, so that a bar at rest is left alone.
fn regen(time: Res<Time>, mut bodies: Query<&mut Health, Without<Dead>>) {
    let tuning = ui().arena;
    let dt = time.delta_secs();
    for mut health in &mut bodies {
        let timer = health.bypass_change_detection();
        timer.since_hit += dt;
        if timer.since_hit >= tuning.regen_after && timer.left < timer.full {
            health.mend(tuning.regen_per_sec * dt);
        }
    }
}

/// `body` is dead: hidden where it is, out of the way and out of play, for
/// the file's `arena.respawn_secs`.
fn die(commands: &mut Commands, body: Entity) {
    commands.entity(body).insert(Dead {
        left: ui().arena.respawn_secs,
    });
}

/// Puts the dead back where their seat arrives, the middle of their
/// island, with their health full, once their while is up.
fn respawn(
    time: Res<Time>,
    mut dead: Query<(Entity, &mut Dead, &Seat, &mut Health)>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let mut back = Vec::new();
    for (body, mut dead, seat, mut health) in &mut dead {
        dead.left -= dt;
        if dead.left <= 0.0 {
            *health = Health::full();
            back.push((body, seat.0));
        }
    }
    for (body, seat) in back {
        commands.entity(body).remove::<Dead>();
        lobby::send(&mut bodies, &mut orbit, body, Venue::Arena, seat);
        info!("{body} stands again on island {}", roundtown_net::arena_island(seat));
    }
}

// ---------------------------------------------------- what is drawn over it

/// Puts "You died. Respawning in 4..." across the screen while you are
/// dead, counting down in halves, and takes it away as you stand again.
fn show_death(
    me: Query<&Dead, With<Player>>,
    texts: Query<Entity, With<DeathText>>,
    mut lines: Query<&mut Text, With<DeathLine>>,
    mut commands: Commands,
) {
    let dead = me.single().ok();
    let text = texts.single().ok();
    match (dead, text) {
        (Some(dead), Some(_)) => {
            let line = died_in(dead.left);
            for mut text in &mut lines {
                if text.0 != line {
                    text.0.clone_from(&line);
                }
            }
        }
        (Some(dead), None) => {
            commands.spawn((
                InArena,
                DeathText,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Vh(DEATH_DOWN),
                    left: Val::ZERO,
                    right: Val::ZERO,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
                children![(DeathLine, hud::words(died_in(dead.left), DEATH_TEXT, ui().colours.text))],
            ));
        }
        (None, Some(text)) => commands.entity(text).despawn(),
        (None, None) => {}
    }
}

/// "You died. Respawning in 3.5...": `left` seconds, rounded up to the
/// half: "4", "3.5", down to "0.5".
fn died_in(left: f32) -> String {
    let halves = (left.max(0.0) * 2.0).ceil();
    let seconds = if halves % 2.0 == 0.0 {
        format!("{}", halves / 2.0)
    } else {
        format!("{:.1}", halves / 2.0)
    };
    fill(&ui().words.arena.died, &[("seconds", &seconds)])
}

/// A health bar over the head of everyone else in the game with health:
/// made as they come, kept over their heads as the camera moves, filled
/// as far as their health goes, and put away while they are dead or off
/// the screen, or you are not in the game.
#[allow(clippy::too_many_arguments)]
fn place_bars(
    me: Query<&Venue, With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    bodies: Query<(Entity, &Transform, &Health, Has<Dead>), Without<Player>>,
    mut bars: Query<(Entity, &BarOf, &mut Visibility, &mut UiTransform, &ComputedNode, &Children)>,
    mut fills: Query<&mut Node, With<BarFill>>,
    mut commands: Commands,
) {
    let in_arena = me.single().ok() == Some(&Venue::Arena);
    if in_arena {
        for (body, ..) in &bodies {
            if !bars.iter().any(|(_, of, ..)| of.0 == body) {
                spawn_bar(&mut commands, body);
            }
        }
    }
    let (Ok((camera, eye)), Ok(window)) = (cameras.single(), windows.single()) else {
        return;
    };
    let eye = GlobalTransform::from(*eye);
    for (bar, of, mut shown, mut place, node, children) in &mut bars {
        let Ok((_, body, health, dead)) = bodies.get(of.0) else {
            commands.entity(bar).despawn();
            continue;
        };
        let point = (in_arena && !dead)
            .then(|| camera.world_to_viewport(&eye, body.translation + Vec3::Y * BAR_OVER).ok())
            .flatten()
            .filter(|point| on_screen(window, *point));
        let Some(point) = point else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let size = node.size() * node.inverse_scale_factor();
        place.set_if_neq(UiTransform {
            translation: Val2::px(point.x - size.x * 0.5, point.y - size.y),
            scale: Vec2::ONE,
            rotation: Rot2::IDENTITY,
        });
        shown.set_if_neq(Visibility::Inherited);
        let width = bar_width(health);
        for child in children.iter() {
            if let Ok(mut fill) = fills.get_mut(child)
                && fill.width != width
            {
                fill.width = width;
            }
        }
    }
}

/// Whether `point`, in the window's logical pixels, is on the screen.
fn on_screen(window: &Window, point: Vec2) -> bool {
    point.x >= 0.0 && point.y >= 0.0 && point.x <= window.width() && point.y <= window.height()
}

/// The bar over `body`'s head, as yours is drawn top left (`hud`), smaller.
fn spawn_bar(commands: &mut Commands, body: Entity) {
    let colours = ui().colours.arena;
    commands.spawn((
        InArena,
        BarOf(body),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            width: Val::VMin(BAR_WIDTH),
            height: Val::VMin(BAR_HEIGHT),
            border: UiRect::all(Val::VMin(BAR_RIM)),
            border_radius: BorderRadius::all(Val::VMin(0.4)),
            ..default()
        },
        BackgroundColor(colours.hp_back),
        BorderColor::all(colours.hp_rim),
        UiTransform::default(),
        Visibility::Hidden,
        Pickable::IGNORE,
        children![(
            BarFill,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                border_radius: BorderRadius::all(Val::VMin(0.25)),
                ..default()
            },
            BackgroundColor(colours.hp),
            Pickable::IGNORE,
        )],
    ));
}

/// A number floating up off the body at `at`, for `damage` taken.
fn spawn_number(commands: &mut Commands, at: Vec3, damage: f32) {
    commands.spawn((
        InArena,
        DamageNumber { at, age: 0.0 },
        hud::words(format!("{damage:.0}"), NUMBER_TEXT, ui().colours.arena.damage),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            ..default()
        },
        UiTransform::default(),
        Visibility::Hidden,
    ));
}

/// Floats every damage number up off where its body stood, fading as it
/// goes, and takes it away once it has risen.
fn float_numbers(
    time: Res<Time>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut numbers: Query<(
        Entity,
        &mut DamageNumber,
        &mut Visibility,
        &mut UiTransform,
        &ComputedNode,
        &mut TextColor,
        &mut TextShadow,
    )>,
    mut commands: Commands,
) {
    let (Ok((camera, eye)), Ok(window)) = (cameras.single(), windows.single()) else {
        return;
    };
    let eye = GlobalTransform::from(*eye);
    let (ink, shadow) = {
        let colours = ui().colours;
        (colours.arena.damage, colours.text_shadow)
    };
    let dt = time.delta_secs();
    for (number, mut floating, mut shown, mut place, node, mut colour, mut under) in &mut numbers {
        floating.age += dt;
        let done = (floating.age / NUMBER_SECS).min(1.0);
        if done >= 1.0 {
            commands.entity(number).despawn();
            continue;
        }
        let at = floating.at + Vec3::Y * (NUMBER_FROM + NUMBER_RISE * done);
        let Some(point) = camera.world_to_viewport(&eye, at).ok().filter(|point| on_screen(window, *point)) else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let size = node.size() * node.inverse_scale_factor();
        place.set_if_neq(UiTransform {
            translation: Val2::px(point.x - size.x * 0.5, point.y - size.y * 0.5),
            scale: Vec2::ONE,
            rotation: Rot2::IDENTITY,
        });
        let solid = 1.0 - done * done;
        colour.0 = ink.with_alpha(ink.alpha() * solid);
        under.color = shadow.with_alpha(shadow.alpha() * solid);
        shown.set_if_neq(Visibility::Inherited);
    }
}

// --------------------------------------------------------------- the board

/// The board top centre: how many are in the room, then the countdown, then
/// how long is left, with a line under it saying what for. A tap on it
/// moves the game on, for trying it out.
fn spawn_board(commands: &mut Commands) {
    commands
        .spawn((
            InArena,
            Board,
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(hud::EDGE),
                left: Val::ZERO,
                right: Val::ZERO,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(0.8),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|board| {
            let colours = ui().colours;
            board.spawn((
                AsksFor(Ask::Skip),
                TakesPress,
                Button,
                Node {
                    padding: UiRect::axes(Val::VMin(3.0), Val::VMin(0.7)),
                    border_radius: BorderRadius::all(Val::VMin(3.0)),
                    ..default()
                },
                BackgroundColor(colours.panel),
                Fill {
                    idle: colours.panel,
                    lit: colours.panel_lit,
                },
                children![(BoardText, hud::words("", BOARD_TEXT, colours.text))],
            ));
            board.spawn((BoardCaption, hud::words("", CAPTION, colours.text)));
        });
}

/// Writes the count, the countdown or the time on the board, and what it is
/// for.
fn show_board(
    game: Option<Res<Game>>,
    mut texts: Query<&mut Text, With<BoardText>>,
    mut captions: Query<&mut Text, (With<BoardCaption>, Without<BoardText>)>,
) {
    let Some(game) = game else {
        return;
    };
    let ui = ui();
    let words = &ui.words.arena;
    let players = fill(
        &words.players,
        &[("count", &game.seats.len().to_string()), ("seats", &ARENA_SEATS.to_string())],
    );
    let (big, small) = match game.stage {
        Stage::Waiting { .. } => (players, words.waiting.clone()),
        Stage::Countdown { left } => (left.max(0.0).ceil().to_string(), words.match_found.clone()),
        Stage::Playing { left } => (clock(left), words.name.clone()),
    };
    for mut text in &mut texts {
        if text.0 != big {
            text.0.clone_from(&big);
        }
    }
    for mut text in &mut captions {
        if text.0 != small {
            text.0.clone_from(&small);
        }
    }
}

/// `9:59`: seconds left, rounded up, so that the last second reads `0:01`.
fn clock(left: f32) -> String {
    let seconds = left.max(0.0).ceil() as u32;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The waiting room as Hajun's map has it: the box round its object,
    /// and the map's shape made of it alone.
    fn hajuns_room() -> (WaitingRoom, Island) {
        let parts = crate::island::glb::read("pvptownmap.glb");
        let corners: Vec<[Vec3; 3]> = parts
            .iter()
            .filter(|part| part.node == WAITING_ROOM)
            .flat_map(|part| part.triangles.iter().copied())
            .collect();
        assert!(!corners.is_empty(), "no {WAITING_ROOM:?} in the map");
        let (low, high) = corners.iter().flatten().fold(
            (Vec3::INFINITY, Vec3::NEG_INFINITY),
            |(low, high), corner| (low.min(*corner), high.max(*corner)),
        );
        let room = WaitingRoom {
            centre: ((low + high) * 0.5).xz(),
            top: high.y,
        };
        (room, Island::new(&corners))
    }

    #[test]
    fn the_map_s_waiting_room_has_a_floor_and_walls_too_thick_to_slip_through_and_too_high_to_jump() {
        let (room, shape) = hajuns_room();
        let floor = room.floor(&shape).expect("a floor under the middle of the room");
        // Over the map, well clear of the islands, and the walls over the
        // floor by more than a jump reaches (1.64 m).
        assert!(floor > 20.0, "floor at {floor}");
        assert!(room.top - floor > 1.7, "walls {} m over the floor", room.top - floor);
        // Thicker than the lattice bodies are kept out of walls on, all
        // the way round, and the floor open inside them.
        let band = (floor + crate::LAND_STEP_UP, floor + crate::PLAYER_HEIGHT);
        let half = 14.0;
        let mut wall = 0;
        for k in 0..40 {
            let along = -half + half * 2.0 * k as f32 / 39.0;
            for side in [-1.0, 1.0] {
                for at in [Vec2::new(side * half, along), Vec2::new(along, side * half)] {
                    let at = room.centre + at;
                    if shape.blocked(at, band.0, band.1) {
                        wall += 1;
                    }
                }
            }
        }
        assert!(wall >= 150, "only {wall} of 160 points 14 m out are in the walls");
        assert!(!shape.blocked(room.centre, band.0, band.1));
        assert!(!shape.blocked(room.centre + Vec2::new(10.0, 10.0), band.0, band.1));
    }

    #[test]
    fn everyone_waits_inside_the_room_facing_its_middle() {
        let (room, shape) = hajuns_room();
        let floor = room.floor(&shape).unwrap();
        for seat in 0..ARENA_SEATS * 2 {
            let spot = waiting_spot(Some(&room), &shape, seat);
            let at = spot.translation;
            let off = at.xz() - room.centre;
            assert!(off.x.abs() < 12.0 && off.y.abs() < 12.0, "seat {seat} at {at}");
            assert_eq!(at.y, floor);
            let middle = Vec3::new(room.centre.x, floor, room.centre.y);
            let toward = (middle - at).normalize();
            assert!(spot.forward().dot(toward) > 0.999, "seat {seat} faces {:?}", spot.forward());
        }
        // Apart from each other.
        for a in 0..ARENA_SEATS {
            for b in a + 1..ARENA_SEATS {
                let apart = waiting_spot(Some(&room), &shape, a)
                    .translation
                    .distance(waiting_spot(Some(&room), &shape, b).translation);
                assert!(apart > 2.0 * crate::PLAYER_RADIUS, "{a} and {b} are {apart} m apart");
            }
        }
        // With no room in the map, everyone waits on their island.
        let on_island = waiting_spot(None, &shape, 1);
        assert_eq!(on_island.translation, island::arrival(Venue::Arena, 1).translation);
    }

    #[test]
    fn every_island_s_middle_is_dry_land_in_the_map() {
        // Everyone arrives in the middle of their island (Hajun, the night
        // of 2026-11-07: "make the spawnpoint in the center of the
        // island"): there is ground under each, about the islands' top,
        // 0.25 m up, or the desk that lies across the middle, and the void
        // beside the islands.
        let parts = crate::island::glb::read("pvptownmap.glb");
        let corners: Vec<[Vec3; 3]> = parts
            .iter()
            .filter(|part| !part.skinned)
            .flat_map(|part| part.triangles.iter().copied())
            .collect();
        let map = Island::new(&corners);
        for seat in 0..ARENA_SEATS {
            let at = island::arrival(Venue::Arena, seat).translation;
            let ground = map.floor(at.xz(), 3.0).unwrap_or_else(|| panic!("seat {seat}: nothing under {at}"));
            assert!((0.0..=1.3).contains(&ground), "seat {seat}: ground at {ground}");
            let out = at.xz().normalize() * 12.0;
            assert!(map.floor(at.xz() + out, 3.0).is_none(), "seat {seat}: land 12 m past the island's middle");
        }
    }

    #[test]
    fn health_goes_to_nothing_and_no_further_and_comes_back_to_full() {
        let mut health = Health {
            left: 100.0,
            full: 100.0,
            since_hit: 9.0,
        };
        assert_eq!(health.share(), 1.0);
        assert!(!health.mend(5.0), "nothing to give back when full");
        health.take(12.0);
        assert!((health.share() - 0.88).abs() < 1e-6);
        assert!(!health.gone());
        assert_eq!(health.since_hit, 0.0, "a hit starts the wait for it to come back over");
        assert!(health.mend(5.0));
        assert!((health.left - 93.0).abs() < 1e-6);
        assert!(health.mend(50.0));
        assert_eq!(health.left, 100.0, "never over full");
        for _ in 0..9 {
            health.take(12.0);
        }
        assert_eq!(health.left, 0.0);
        assert!(health.gone());
        assert_eq!(health.share(), 0.0);
        // A bar is drawn in steps of half a percent.
        health.left = 33.3;
        assert_eq!(bar_width(&health), Val::Percent(33.5));
    }

    #[test]
    fn a_bot_runs_away_along_dry_land() {
        // A strip of land 20 m long and 12 m wide, someone coming at the
        // bot from the east: in the middle of the strip it runs straight
        // away, west; near the west end, where straight away is off the
        // land, it runs round, along the strip; and off the land altogether
        // it stands.
        let v = Vec3::new;
        let strip = Island::new(&[
            [v(-10.0, 0.0, -6.0), v(-10.0, 0.0, 6.0), v(10.0, 0.0, 6.0)],
            [v(-10.0, 0.0, -6.0), v(10.0, 0.0, 6.0), v(10.0, 0.0, -6.0)],
        ]);
        let toward = Vec3::X;
        assert_eq!(flight(&strip, v(0.0, 0.0, 0.0), toward), -Vec3::X);
        let near_the_end = flight(&strip, v(-8.5, 0.0, 0.0), toward);
        assert!(near_the_end.x > -0.9 && near_the_end.z.abs() > 0.4, "{near_the_end}");
        assert!(ai::dry_all_the_way(&strip, v(-8.5, 0.0, 0.0), near_the_end, FLIGHT));
        assert_eq!(flight(&strip, v(30.0, 0.0, 0.0), toward), Vec3::ZERO);
    }

    #[test]
    fn on_the_map_a_bot_beside_the_island_s_middle_runs_straight_away() {
        // On the first island, 6 m east of its middle with you in the
        // middle: east is 4 m of island, enough to run straight away.
        let parts = crate::island::glb::read("pvptownmap.glb");
        let corners: Vec<[Vec3; 3]> = parts
            .iter()
            .filter(|part| !part.skinned)
            .flat_map(|part| part.triangles.iter().copied())
            .collect();
        let map = Island::new(&corners);
        let here = Vec3::new(6.0, 0.25, 130.0);
        for step in 1..=12 {
            let at = here + Vec3::X * step as f32 * 0.25;
            println!("x {}: floor {:?}", at.x, map.floor(at.xz(), at.y + crate::LAND_STEP_UP));
        }
        assert!(ai::dry_all_the_way(&map, here, Vec3::X, FLIGHT), "not dry east of the middle");
        assert_eq!(flight(&map, here, Vec3::NEG_X), Vec3::X);
    }

    #[test]
    fn the_death_count_goes_down_in_halves() {
        let seconds = |left: f32| {
            let line = died_in(left);
            line.trim_start_matches("You died. Respawning in ").trim_end_matches("...").to_string()
        };
        assert_eq!(seconds(4.0), "4");
        assert_eq!(seconds(3.9), "4");
        assert_eq!(seconds(3.5), "3.5");
        assert_eq!(seconds(3.2), "3.5");
        assert_eq!(seconds(0.3), "0.5");
        assert_eq!(seconds(0.0), "0");
    }

    #[test]
    fn the_clock_rounds_up() {
        assert_eq!(clock(600.0), "10:00");
        assert_eq!(clock(599.2), "10:00");
        assert_eq!(clock(59.0), "0:59");
        assert_eq!(clock(0.4), "0:01");
        assert_eq!(clock(0.0), "0:00");
    }
}
