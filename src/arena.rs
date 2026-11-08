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
//! ([`read_map`], [`WaitingRoom`]) and stood on as any of the map is,
//! with the camera fixed straight down over you (`OrbitCamera::top_down`)
//! and the game's own buttons round the screen (`hud::Layout::Arena`). The
//! board top centre counts the players.
//!
//! **Since 2026-11-08 the game is the server's online** (Hajun: "right now
//! the bridge battle minigame doesnt allow queing with other players i
//! think even tho we join the game at the same time. fix that"), as House
//! Builder is: whoever chooses it goes into the game filling now
//! (`Ask::Arena`, `Tell::Seated`), eight seats in four teams of two, a team
//! to an island (`ARENA_SEATS`, `arena_team`; "make it 8 player rather
//! than 4 player, so its 2v2v2v2"), and the device follows what the server
//! says of it ([`follow_round`], `ArenaRound`): the stage and the time, everyone's
//! health, each team's anchor and who is out. Everyone's punches go
//! through the server, which keeps the health and knocks the bots about
//! itself (`server/src/arena.rs`). Offline the game is this device's own,
//! as it was, with the computer in every seat but yours, and the same
//! rules played here. Both ways: nobody joined for [`BOT_WAIT`] seconds and
//! the computer takes the seats left; the board counts [`COUNTDOWN`] down;
//! everyone is sent to their island (`roundtown_net::arrival`), "Fight!";
//! then [`PLAY_SECS`] run, which a tap on the board cuts short.
//!
//! The fight (the night of 2026-11-07, Hajun: "make the ai, if there is
//! another ai/player within its visible radius, it will go towards them
//! and try to fight/punch them ... make punching deal damage, and show
//! healthbar for enemies. make default hp = 100 ... make damage numbers
//! appear on each hit. default damage = 12. so when health <0 u die too
//! ... make it running rather than walking"): everyone runs there, at the
//! file's `arena.run_speed` (`Intent::sprint`), and everyone has [`Health`],
//! `arena.health` of it; a punch that lands and takes ([`Landed`]) takes
//! `arena.damage` off it, a number floating up off the body
//! ([`DamageNumber`]), everyone but you with their name and a bar over
//! their head ([`TagOf`]: small and faint, so as not to hide the fight, in
//! a teammate's colour or an enemy's; Hajun, 2026-11-08: "make the
//! healthbar and the nametag not distracting ... some indication wether
//! they are teammate or enemy somehow - maybe with color?"), and you with
//! your bar top left (`hud`). A punch never lands on a teammate (`Team`;
//! "make it so u cannot hurt your own teammate"). At none, or
//! `arena.fall_to_die` below the map (`void_y`), a body is [`Dead`]: hidden
//! where it died, in nobody's way; **and comes back only while its team's
//! respawn anchor stands** ("make the purple rotated cube thingy on each
//! island the respawn anchor (similar to bed in bedwars) so you only
//! indefinitely respawn when you die, and at that point when u died, the
//! respawn anchor is still alive. and after u die after the respawn anchor
//! is dead, you dont respawn, you get a play again button and a exit
//! button"): the purple pillar in the material [`SPAWN_MATERIAL`] on each
//! island (`roundtown_net::ARENA_ANCHORS`), with a bar over it
//! ([`AnchorBar`]), which a punch of another team's lands on as on a body
//! ([`hit_anchors`]), `arena.anchor_health` of it, twenty punches; broken,
//! it is gone from the map ([`break_anchors`]), and its team's dead stay
//! dead, with Play again and Exit on a panel ([`Panel`]). The last team
//! with anyone in play wins, or nobody when the time runs out: the result
//! is up for [`RESULTS_SECS`], with the same buttons, and everyone is back
//! in the town, the camera as it was. A bot goes for whoever of another
//! team is within `arena.hunt_radius` ([`hunt`]), runs away when nearly
//! beaten ([`flight`]), and never crosses a bridge unless one is built for
//! it (`bridge`): the map's own bridges went on 2026-11-08, and every
//! bridge is built by walking.
//!
//! Everything the game puts in the world or on the screen carries
//! [`InArena`] and goes when the game does. Your own body is the only thing
//! that comes back.


use bevy::camera::primitives::Aabb;
use bevy::gltf::GltfMaterialName;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::{self as rules, BOT_WAIT, COUNTDOWN, PLAY_SECS, RESULTS_SECS};
use roundtown_net::{
    ANCHOR_RADIUS, ARENA_ANCHORS, ARENA_SEATS, ARENA_TEAMS, ArenaRound, ArenaStage, Ask as NetAsk, Member, Seats, Tell,
    arena_team,
};

use crate::ai::{self, Steering};
use crate::builder::{self, Fill};
use crate::drive::Driving;
use crate::hud::{self, TakesPress};
use crate::island::{self, Island, Islands, Land, Venue, is_material};
use crate::lobby::{self, Lobby, Seat, Whereabouts};
use crate::net::Online;
use crate::range::Sector;
use crate::ui::{fill, ui};
use crate::{
    Carrying, Dead, Intent, Landed, OrbitCamera, PLAYER_HEIGHT, PLAYER_RADIUS, Player, PlayerJump, Punching, Shove,
    Swung, Team, ThirdPersonCamera, View, WalkCycle, void_y,
};

/// What the waiting room is called in the map: Hajun's glass box over the
/// middle of it (2026-11-07). Everyone waits on its floor, which is read
/// off the map's shape under the middle of the box.
pub(crate) const WAITING_ROOM: &str = "waiting room";
/// What the respawn anchors' material is called in the map: the purple
/// pillar on each island (Hajun, 2026-11-07).
pub(crate) const SPAWN_MATERIAL: &str = "spawn";
/// How long the game's name is across the screen as you arrive, and
/// "Fight!" as the game begins, and an anchor's breaking, in seconds.
const NAME_SECS: f32 = 3.0;
const FIGHT_SECS: f32 = 2.0;
const ANCHOR_SECS: f32 = 3.0;
/// Sizes, as shares of the short side of the screen: the board's count,
/// countdown or time, small (Hajun: "(small) timer"), and the line under
/// it; the words across the screen while you are dead, and how far down
/// the screen they sit; someone else's name and health bar over their
/// head, small and faint; an anchor's bar; a damage number; and the panel
/// at the end, or when you are out, with its words and its buttons.
const BOARD_TEXT: f32 = 4.8;
const CAPTION: f32 = 3.4;
const DEATH_TEXT: f32 = 5.0;
const DEATH_DOWN: f32 = 38.0;
const TAG_TEXT: f32 = 2.4;
const BAR_WIDTH: f32 = 6.0;
const BAR_HEIGHT: f32 = 0.8;
const BAR_RIM: f32 = 0.15;
const ANCHOR_BAR_WIDTH: f32 = 9.0;
const ANCHOR_BAR_HEIGHT: f32 = 1.1;
const NUMBER_TEXT: f32 = 3.8;
const PANEL_WIDTH: f32 = 58.0;
const PANEL_TITLE: f32 = 5.6;
const PANEL_TEXT: f32 = 3.4;
const PANEL_BUTTON: f32 = 8.4;
const PANEL_BUTTON_WIDTH: f32 = 22.0;
const PANEL_BUTTON_TEXT: f32 = 3.4;
/// How high over a body's feet its name and bar stand, in metres, how high
/// over the ground an anchor's bar stands over its pillar, and how high a
/// damage number starts from and rises to over its while, in seconds.
const TAG_OVER: f32 = 2.2;
const ANCHOR_BAR_OVER: f32 = 4.6;
const NUMBER_FROM: f32 = 1.8;
const NUMBER_RISE: f32 = 1.6;
const NUMBER_SECS: f32 = 0.9;
/// How near a bot presses in on whoever it is going for before it slows
/// to a push, as a share of the punch's reach.
const PRESS_IN: f32 = 0.3;
/// How far ahead a bot running away looks for dry land, in metres.
const FLIGHT: f32 = 3.0;
/// Online, how long a game waits for a dropped connection to come back,
/// in seconds, before it is given up.
const RECONNECT_WAIT: f32 = 15.0;
/// Which triangles of the map are a broken anchor's, to be taken out of
/// what is stood on: those within this of its middle, across the ground,
/// in metres, and over the island's top.
const ANCHOR_REACH: f32 = 1.3;

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
    /// Another game, from the panel at the end or once you are out.
    Again,
    /// Back to the town, from the same panel.
    Exit,
}

/// What a tap on the board or a panel's button asks for.
#[derive(Component, Clone, Copy)]
struct AsksFor(Ask);

/// A body's health in a game of Bridge Wars: how much is left of what it
/// started with, and how long since it was last hit, after which it comes
/// back ([`regen`]). Yours is the bar under your name (`hud`), everyone
/// else's the bar over their head ([`TagOf`]). Online it is the server's
/// ([`follow_round`]).
#[derive(Component, Debug)]
pub(crate) struct Health {
    left: f32,
    full: f32,
    since_hit: f32,
}

impl Health {
    /// All of it, as a body starts and stands again: the file's
    /// `arena.health` offline, the server's online.
    fn of(full: f32) -> Self {
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
fn bar_width(share: f32) -> Val {
    Val::Percent((share.clamp(0.0, 1.0) * 200.0).round() * 0.5)
}

/// A bot's fighting mind in Bridge Wars, over its wandering ([`hunt`]):
/// how long until it may punch again, in seconds. Offline only: online the
/// server moves its bots.
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

/// How much of each resource you hold: the metres of bridge you have left
/// to lay (`bridge`; the file's `arena.bridge_metres` as a game begins,
/// shown by the log as "100m"; since 2026-11-09, Hajun: "players will buy
/// the bridge by meter count"), and the rest, which nothing gives yet.
#[derive(bevy::prelude::Resource, Default)]
pub(crate) struct Resources {
    pub bridge_metres: f32,
    pub gold: u32,
    pub diamond: u32,
    pub emerald: u32,
}

impl Resources {
    /// What is shown beside `kind`'s picture.
    pub(crate) fn of(&self, kind: Resource) -> String {
        match kind {
            Resource::Wood => format!("{}m", self.bridge_metres.max(0.0).round()),
            Resource::Gold => self.gold.to_string(),
            Resource::Diamond => self.diamond.to_string(),
            Resource::Emerald => self.emerald.to_string(),
        }
    }
}

/// A game of Bridge Wars, from the waiting room to the end.
#[derive(bevy::prelude::Resource)]
struct Game {
    /// Which game this is on this device, counting up: another number is
    /// another game (`bridge` clears its bridges on it).
    id: u32,
    /// Everyone in it, in seat order. A seat is an island and a team
    /// (`arena_team`). Online the server's ([`follow_round`]).
    seats: Vec<Member>,
    /// Their bodies, in the same order, offline; online only yours is
    /// here, everyone else being the server's.
    bodies: Vec<Option<Entity>>,
    /// Your seat in the town's room, to go back to, offline.
    town_seat: usize,
    /// Your seat in the game.
    me: usize,
    /// The server is running it, rather than this device.
    online: bool,
    /// Online, how long the connection has been gone, if it has.
    cut_off: Option<f32>,
    stage: Stage,
    /// How the camera was turned and zoomed before it was fixed over you,
    /// to go back to; none if it was over you already.
    view: Option<View>,
    /// Everyone's health as last seen, online, to notice a hit by.
    health: Vec<f32>,
    /// Each team's respawn anchor: what is left of it, 0 broken; and how
    /// long each is still reeling from the last punch that took, in which
    /// no other counts (`melee.hurt_time`, as a body is).
    anchors: [f32; ARENA_TEAMS],
    anchor_hurt: [f32; ARENA_TEAMS],
    /// The seats out for good: dead with their anchor broken.
    out: [bool; ARENA_SEATS],
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    /// In the waiting room: how long since the last to join did.
    Waiting { since_join: f32 },
    /// Full, counting down to go.
    Countdown { left: f32 },
    /// Everyone on their island, for this much longer.
    Playing { left: f32 },
    /// Done: `winner` is the team left standing, or none; the result is up
    /// for `left` more seconds.
    Over { winner: Option<u8>, left: f32 },
}

impl Game {
    fn playing(&self) -> bool {
        matches!(self.stage, Stage::Playing { .. })
    }

    fn over(&self) -> bool {
        matches!(self.stage, Stage::Over { .. })
    }

    /// Your team.
    fn my_team(&self) -> u8 {
        arena_team(self.me)
    }

    /// Which teams still have anyone in play: anyone not out.
    fn teams_in_play(&self) -> Vec<u8> {
        let mut teams: Vec<u8> = (0..self.seats.len().min(ARENA_SEATS))
            .filter(|&seat| !self.out[seat])
            .map(arena_team)
            .collect();
        teams.sort_unstable();
        teams.dedup();
        teams
    }

    /// `seat` has died: it comes back after the file's `arena.respawn_secs`
    /// if its team's anchor stands now, and is out for good otherwise.
    /// What to mark its body dead with.
    fn die(&mut self, seat: usize) -> Dead {
        let anchor = self.anchors[usize::from(arena_team(seat))];
        if anchor > 0.0 {
            Dead {
                left: ui().arena.respawn_secs,
            }
        } else {
            if seat < ARENA_SEATS {
                self.out[seat] = true;
            }
            Dead { left: f32::INFINITY }
        }
    }
}

/// How the fight stands, for the bridges (`bridge`): whether a game is on,
/// your seat in it, whether it is the server's, and which game it is, so
/// that another game's bridges are not this one's.
#[derive(bevy::prelude::Resource, Default, PartialEq)]
pub(crate) struct Fighting {
    pub(crate) playing: bool,
    pub(crate) seat: Option<u8>,
    pub(crate) online: bool,
    pub(crate) game: u32,
}

/// A game of Bridge Wars is being played offline: nothing goes online in
/// the middle of it (`net`).
#[derive(bevy::prelude::Resource, Default, PartialEq)]
pub(crate) struct OfflineArena(pub bool);

/// How many games have been begun on this device.
#[derive(bevy::prelude::Resource, Default)]
struct Begun(u32);

/// The waiting room, as the map has it: the middle of the box across the
/// ground, and the top of it, under which its floor is found
/// ([`WaitingRoom::floor`]).
#[derive(bevy::prelude::Resource, Clone, Copy, Debug)]
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
    /// up: on a ring round its middle, facing in, as the server has it
    /// (`roundtown_net::bridge_wars::waiting_spot`).
    fn spot(&self, floor: f32, seat: usize) -> Transform {
        let (at, turn) = rules::waiting_spot(seat);
        Transform::from_translation(Vec3::new(self.centre.x + at.x, floor, self.centre.y + at.z)).with_rotation(turn)
    }
}

/// The respawn anchors as the map has them: the meshes of each team's
/// pillar, to hide when it breaks; the map's triangles as they were, to
/// stand them back up at the end; and which are broken.
#[derive(bevy::prelude::Resource, Default)]
struct Anchors {
    pillars: [Vec<Entity>; ARENA_TEAMS],
    original: Option<Vec<[Vec3; 3]>>,
    broken: [bool; ARENA_TEAMS],
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

/// Someone else's name and health bar over their head: whose, and the
/// bar's fill.
#[derive(Component)]
struct TagOf(Entity);
#[derive(Component)]
struct TagFill;

/// A team's anchor's bar, over its pillar: whose, and its fill.
#[derive(Component)]
struct AnchorBar(u8);
#[derive(Component)]
struct AnchorFill;

/// A number floating up off a body a punch landed on: where the body stood,
/// and how long ago.
#[derive(Component)]
struct DamageNumber {
    at: Vec3,
    age: f32,
}

/// The panel in the middle of the screen with Play again and Exit on it,
/// while you are out. (The end had one too for a day; Hajun found a
/// "times up popup" in the way and had it taken out, 2026-11-09: the end
/// is a headline now.)
#[derive(Component)]
struct Panel;

/// Bridge Wars' reading of taps and the game they drive: ahead of your own
/// walking, and of the islands being shown, so that wherever it sends you
/// is drawn the same frame.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ArenaSystems;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Ask>()
        .init_resource::<Resources>()
        .init_resource::<Fighting>()
        .init_resource::<OfflineArena>()
        .init_resource::<Begun>()
        .add_observer(ask)
        .add_observer(read_map)
        .add_systems(
            Update,
            (hear, follow_round, cut_off, play, publish, show_board)
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
                (team_up, hunt).chain().after(ai::steer).after(crate::start_punch).before(crate::move_bodies),
                (reap, deal_damage, hit_anchors, regen, respawn, break_anchors)
                    .chain()
                    .after(crate::move_bodies)
                    .before(crate::follow_camera),
                (show_death, show_panel, place_tags, place_anchor_bars, float_numbers).after(crate::follow_camera),
            ),
        );
}

// ---------------------------------------------------------------- the map

/// Reads what the game needs off the map once it has spawned: the waiting
/// room, by its name ([`WAITING_ROOM`]), as the box round every mesh of
/// it, placed in the world, for where its middle is and how high it
/// reaches (its floor is read off the map's shape when it is needed, the
/// box being open to the sky and its walls' tops the highest of it); and
/// the respawn anchors' pillars, every mesh in [`SPAWN_MATERIAL`], each
/// the team's whose anchor it stands at.
fn read_map(
    ready: On<WorldInstanceReady>,
    lands: Query<&Venue, With<Land>>,
    children: Query<&Children>,
    names: Query<&Name>,
    materials: Query<&GltfMaterialName>,
    boxes: Query<&Aabb>,
    placement: TransformHelper,
    mut commands: Commands,
) {
    // The map's own model, not a body's on it.
    if lands.get(ready.entity).ok() != Some(&Venue::Arena) {
        return;
    }
    let mut anchors = Anchors::default();
    for entity in children.iter_descendants(ready.entity) {
        let Ok(material) = materials.get(entity) else {
            continue;
        };
        if !is_material(&material.0, SPAWN_MATERIAL) {
            continue;
        }
        let Ok(place) = placement.compute_global_transform(entity) else {
            continue;
        };
        let middle = boxes.get(entity).map_or(Vec3::ZERO, |aabb| Vec3::from(aabb.center));
        let at = place.transform_point(middle).xz();
        let team = (0..ARENA_TEAMS)
            .min_by(|&a, &b| ARENA_ANCHORS[a].distance_squared(at).total_cmp(&ARENA_ANCHORS[b].distance_squared(at)))
            .expect("four teams");
        anchors.pillars[team].push(entity);
    }
    info!(
        "Bridge Wars' anchors: {:?} meshes a team",
        anchors.pillars.iter().map(Vec::len).collect::<Vec<_>>()
    );
    commands.insert_resource(anchors);

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

/// Where the player in `seat` waits for the game: in the waiting room, on
/// its floor, or, with no room in the map, on their island, where they
/// arrive for the game anyway.
fn waiting_spot(room: Option<&WaitingRoom>, map: &Island, seat: usize) -> Transform {
    match room.and_then(|room| Some((room, room.floor(map)?))) {
        Some((room, floor)) => room.spot(floor, seat),
        None => island::arrival(Venue::Arena, seat),
    }
}

// ---------------------------------------------------------------- the game

/// Whatever was tapped, if it asks for something: the board, or a panel's
/// button.
fn ask(click: On<Pointer<Click>>, buttons: Query<&AsksFor>, mut asks: MessageWriter<Ask>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(button) = buttons.get(click.entity) {
        asks.write(button.0);
    }
}

/// What the server tells this device alone about Bridge Wars, online: that
/// it has seated you in a game ([`Tell::Seated`]), which begins it here;
/// that you stand again on your island; and that you are back in the town,
/// which ends it here.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut tells: MessageReader<Tell>,
    game: Option<ResMut<Game>>,
    lobby: Res<Lobby>,
    me: Query<(Entity, &Seat), With<Player>>,
    in_arena: Query<Entity, With<InArena>>,
    mut begun: ResMut<Begun>,
    anchors: Option<ResMut<Anchors>>,
    mut islands: ResMut<Islands>,
    mut visibilities: Query<&mut Visibility>,
    mut bodies: Whereabouts,
    (mut orbit, mut driving): (ResMut<OrbitCamera>, ResMut<Driving>),
    mut commands: Commands,
) {
    let mut game = game;
    let mut anchors = anchors;
    for tell in tells.read() {
        let Ok((you, seat)) = me.single() else {
            continue;
        };
        match tell {
            Tell::Seated { seat: mine } => {
                // Another game straight after the last keeps the view the
                // first fixed the camera from.
                let kept = game.as_ref().and_then(|game| game.view);
                for thing in &in_arena {
                    commands.entity(thing).try_despawn();
                }
                if let Some(anchors) = anchors.as_deref_mut() {
                    stand_anchors_up(anchors, &mut islands, &mut visibilities);
                }
                let mut opened = open(
                    &mut commands,
                    you,
                    usize::from(*mine),
                    seat.0,
                    true,
                    &mut begun,
                    &lobby,
                    &mut orbit,
                    &mut driving,
                );
                opened.view = kept.or(opened.view);
                info!("Bridge Wars: seated online, seat {mine}");
                commands.insert_resource(opened);
                game = None;
            }
            Tell::Arrive {
                venue: Venue::Arena, ..
            } => {
                if let Some(game) = game.as_deref() {
                    if game.online {
                        commands.entity(you).remove::<Dead>().insert(Health::of(rules::HEALTH));
                    }
                }
            }
            Tell::Arrive {
                venue: Venue::Town | Venue::Home,
                ..
            } => {
                if let Some(game) = game.as_deref()
                    && game.online
                {
                    finish(&mut commands, game, &in_arena, you, &mut bodies, &mut orbit, anchors.as_deref_mut(), &mut islands, &mut visibilities, false);
                    info!("Bridge Wars: over, back to the town");
                }
                game = None;
            }
            Tell::Refused(refusal) => {
                info!("the server turned that down: {refusal:?}");
            }
            _ => {}
        }
    }
}

/// Online, the game is the server's: whatever it says the round is, this
/// shows: the stage and the time left, who is in it, everyone's health,
/// each anchor and who is out; and as it moves on, what goes with that
/// here: "Fight!" as the fight begins, a number off whoever was hit, the
/// dead hidden, your own death, and the panel at the end.
#[allow(clippy::too_many_arguments)]
fn follow_round(
    game: Option<ResMut<Game>>,
    rounds: Query<(&ArenaRound, &Seats), With<Remote>>,
    mut me: Query<(Entity, &Transform, &mut Health, Option<&Dead>), With<Player>>,
    mut others: Query<
        (Entity, &Member, &Transform, Option<&mut Health>, Has<Dead>),
        (With<Remote>, With<WalkCycle>, Without<Player>),
    >,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        return;
    };
    if !game.online {
        return;
    }
    let Ok((round, seats)) = rounds.single() else {
        return;
    };
    if game.seats != seats.0 {
        game.seats.clone_from(&seats.0);
    }
    let left = f32::from(round.left);
    let stage = match round.stage {
        ArenaStage::Waiting => Stage::Waiting { since_join: 0.0 },
        ArenaStage::Countdown => Stage::Countdown { left },
        ArenaStage::Playing => Stage::Playing { left },
        ArenaStage::Over { winner } => Stage::Over { winner, left },
    };
    let was = game.stage;
    game.stage = stage;
    if matches!(stage, Stage::Playing { .. }) && !matches!(was, Stage::Playing { .. }) {
        hud::announce(&mut commands, None, &ui().words.arena.fight, hud::HEADLINE, FIGHT_SECS).insert(InArena);
    }
    if let Stage::Over { winner, .. } = stage
        && !matches!(was, Stage::Over { .. })
    {
        announce_result(&mut commands, game.my_team(), winner);
    }
    for (team, anchor) in round.anchors.iter().enumerate() {
        game.anchors[team] = f32::from(*anchor);
    }
    for seat in 0..ARENA_SEATS {
        game.out[seat] = round.is_out(seat);
    }
    // Everyone's health: yours on your body, everyone else's on theirs,
    // the dead hidden and a number off whoever was hit since last time.
    let mut health: Vec<f32> = round.health.iter().map(|&left| f32::from(left)).collect();
    health.resize(health.len().max(game.health.len()), rules::HEALTH);
    for (seat, &left) in health.iter().enumerate() {
        let before = game.health.get(seat).copied().unwrap_or(rules::HEALTH);
        if seat == game.me {
            if let Ok((you, at, mut mine, dead)) = me.single_mut() {
                if (mine.left - left).abs() > 1e-3 {
                    mine.left = left;
                    mine.full = rules::HEALTH;
                }
                if left < before {
                    spawn_number(&mut commands, at.translation, before - left);
                }
                // Dead, as the server says: counting down to standing
                // again, or out for good. A fall marks you dead here first
                // ([`reap`]); the server's word on which it is wins.
                let out = game.out[game.me];
                let waiting = dead.is_some_and(|dead| dead.left.is_finite());
                if left <= 0.0 && (dead.is_none() || (out && waiting)) {
                    let dead = if out {
                        Dead { left: f32::INFINITY }
                    } else {
                        Dead { left: rules::RESPAWN_SECS }
                    };
                    commands.entity(you).insert(dead);
                }
            }
            continue;
        }
        let Some(member) = game.seats.get(seat) else {
            continue;
        };
        let Some((body, _, at, theirs, dead)) = others.iter_mut().find(|(_, other, ..)| other.id == member.id) else {
            continue;
        };
        match theirs {
            Some(mut theirs) => {
                if (theirs.left - left).abs() > 1e-3 {
                    theirs.left = left;
                    theirs.full = rules::HEALTH;
                }
            }
            None => {
                let mut theirs = Health::of(rules::HEALTH);
                theirs.left = left;
                commands.entity(body).insert(theirs);
            }
        }
        if left < before {
            spawn_number(&mut commands, at.translation, before - left);
        }
        if left <= 0.0 && !dead {
            commands.entity(body).insert(Dead { left: f32::INFINITY });
        } else if left > 0.0 && dead {
            commands.entity(body).remove::<Dead>();
        }
    }
    game.health = health;
}

/// Online, the connection gone in the middle of a game: it waits a moment
/// for it to come back, and then gives the game up, back to the town as it
/// is offline.
#[allow(clippy::too_many_arguments)]
fn cut_off(
    time: Res<Time>,
    online: Res<Online>,
    game: Option<ResMut<Game>>,
    me: Query<Entity, With<Player>>,
    in_arena: Query<Entity, With<InArena>>,
    anchors: Option<ResMut<Anchors>>,
    mut islands: ResMut<Islands>,
    mut visibilities: Query<&mut Visibility>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        return;
    };
    if !game.online || online.is() {
        return;
    }
    let gone = match game.cut_off {
        Some(gone) => gone + time.delta_secs(),
        None => {
            hud::announce(&mut commands, None, &ui().words.house_builder.reconnecting, hud::CAPTION, 3.0).insert(InArena);
            0.0
        }
    };
    game.cut_off = Some(gone);
    if gone < RECONNECT_WAIT {
        return;
    }
    let Ok(you) = me.single() else {
        return;
    };
    let mut anchors = anchors;
    finish(&mut commands, &game, &in_arena, you, &mut bodies, &mut orbit, anchors.as_deref_mut(), &mut islands, &mut visibilities, true);
    hud::announce(&mut commands, None, &ui().words.house_builder.lost_connection, hud::CAPTION, 3.0);
}

/// Takes you into a game and moves it on, offline: into the waiting room,
/// the bots in once nobody else has come, the countdown, everyone to their
/// islands, the time running, the fight's end, and at the end of it
/// everyone back in the town. Online, asks the server for each instead.
/// Asked in before the map has loaded, it waits for it, under the black
/// screen (`games`).
#[allow(clippy::too_many_arguments)]
fn play(
    time: Res<Time>,
    mut asks: MessageReader<Ask>,
    game: Option<ResMut<Game>>,
    online: Res<Online>,
    lobby: Res<Lobby>,
    room: Option<Res<WaitingRoom>>,
    me: Query<(Entity, &Seat), With<Player>>,
    in_arena: Query<Entity, With<InArena>>,
    (anchors, mut islands, mut visibilities): (Option<ResMut<Anchors>>, ResMut<Islands>, Query<&mut Visibility>),
    mut bodies: Whereabouts,
    (mut orbit, mut driving): (ResMut<OrbitCamera>, ResMut<Driving>),
    (mut begun, assets, mut requests): (ResMut<Begun>, Res<AssetServer>, MessageWriter<NetAsk>),
    mut asked_in: Local<bool>,
    mut commands: Commands,
) {
    let mut anchors = anchors;
    let Ok((you, seat)) = me.single() else {
        asks.clear();
        return;
    };
    let mut skip = false;
    let (mut again, mut exit) = (false, false);
    for &ask in asks.read() {
        match ask {
            Ask::Enter => {
                if online.is() {
                    requests.write(NetAsk::Arena);
                    info!("Bridge Wars: asking the server for a game");
                } else {
                    *asked_in = true;
                }
            }
            Ask::Skip => skip = true,
            Ask::Again => again = true,
            Ask::Exit => exit = true,
        }
    }
    let room = room.as_deref().copied();
    let Some(mut game) = game else {
        if *asked_in && let Some(map) = islands.get(Venue::Arena) {
            *asked_in = false;
            let spot = waiting_spot(room.as_ref(), map, 0);
            let game = open(&mut commands, you, 0, seat.0, false, &mut begun, &lobby, &mut orbit, &mut driving);
            lobby::put(&mut bodies, &mut orbit, you, Venue::Arena, spot);
            commands.insert_resource(game);
        }
        return;
    };
    *asked_in = false;
    let game = &mut *game;
    let free_to_leave = game.over() || game.out.get(game.me).copied().unwrap_or(false);
    if game.online {
        if skip {
            requests.write(NetAsk::Skip);
        }
        if again && free_to_leave {
            requests.write(NetAsk::Arena);
        }
        if exit && free_to_leave {
            requests.write(NetAsk::Leave);
        }
        return;
    }
    if (again || exit) && free_to_leave {
        finish(&mut commands, game, &in_arena, you, &mut bodies, &mut orbit, anchors.as_deref_mut(), &mut islands, &mut visibilities, true);
        *asked_in = again;
        return;
    }
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
                    commands.entity(body).insert((
                        InArena,
                        Health::of(ui().arena.health),
                        Hunter::default(),
                        Team(arena_team(seat)),
                    ));
                    game.bodies.push(Some(body));
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
            for (seat, body) in game.bodies.iter().enumerate() {
                if let Some(body) = body {
                    lobby::send(&mut bodies, &mut orbit, *body, Venue::Arena, seat);
                }
            }
            hud::announce(&mut commands, None, &ui().words.arena.fight, hud::HEADLINE, FIGHT_SECS).insert(InArena);
            info!("Bridge Wars: everyone to their islands, {PLAY_SECS} s to play");
            game.stage = Stage::Playing { left: PLAY_SECS };
        }
        Stage::Playing { left } => {
            let teams = game.teams_in_play();
            let winner = match (teams.len(), left <= dt || skip) {
                (0, _) => Some(None),
                (1, _) => Some(Some(teams[0])),
                (_, true) => Some(None),
                _ => None,
            };
            game.stage = match winner {
                Some(winner) => {
                    info!("Bridge Wars: over, team {winner:?} wins");
                    announce_result(&mut commands, game.my_team(), winner);
                    Stage::Over {
                        winner,
                        left: RESULTS_SECS,
                    }
                }
                None => Stage::Playing { left: left - dt },
            };
        }
        Stage::Over { winner, left } if left > dt && !skip => {
            game.stage = Stage::Over {
                winner,
                left: left - dt,
            }
        }
        // Time's up: back to the town, alive, and the camera as it was.
        Stage::Over { .. } => {
            finish(&mut commands, game, &in_arena, you, &mut bodies, &mut orbit, anchors.as_deref_mut(), &mut islands, &mut visibilities, true);
            info!("Bridge Wars: over, back to the town");
        }
    }
}

/// A new game with you in `me`, your health full: the camera fixed over
/// you, the board, the anchors' bars, the game's name across the screen.
/// Offline you are put in the waiting room by whoever calls this; online
/// the server puts you there. Getting there gets you out of the car
/// (`drive::toggle`, on the change of island), and the camera goes back to
/// how it was before the car had it, not to the car's.
#[allow(clippy::too_many_arguments)]
fn open(
    commands: &mut Commands,
    you: Entity,
    me: usize,
    town_seat: usize,
    online: bool,
    begun: &mut Begun,
    lobby: &Lobby,
    orbit: &mut OrbitCamera,
    driving: &mut Driving,
) -> Game {
    let before_the_car = driving.take_view();
    let view = orbit.top_down().or(before_the_car);
    let full = if online { rules::HEALTH } else { ui().arena.health };
    commands
        .entity(you)
        .insert((Seat(me), Team(arena_team(me)), Health::of(full)))
        .remove::<Dead>();
    // What you have to build with: the file's metres of bridge.
    commands.insert_resource(Resources {
        bridge_metres: ui().arena.bridge_metres,
        ..default()
    });
    spawn_board(commands);
    spawn_anchor_bars(commands, arena_team(me));
    hud::announce(commands, None, &ui().words.arena.name, hud::HEADLINE, NAME_SECS).insert(InArena);
    begun.0 += 1;
    info!("Bridge Wars: into the waiting room{}", if online { " (online)" } else { "" });
    let mut bodies = vec![None; ARENA_SEATS.max(me + 1)];
    bodies[me] = Some(you);
    Game {
        id: begun.0,
        seats: if online { Vec::new() } else { vec![lobby.mine()] },
        bodies: if online { bodies } else { vec![Some(you)] },
        town_seat,
        me,
        online,
        cut_off: None,
        stage: Stage::Waiting { since_join: 0.0 },
        view,
        health: Vec::new(),
        anchors: [if online { rules::ANCHOR_HEALTH } else { ui().arena.anchor_health }; ARENA_TEAMS],
        anchor_hurt: [0.0; ARENA_TEAMS],
        out: [false; ARENA_SEATS],
    }
}

/// Says how the game went across the screen as it ends: for you, over who
/// won, or that the time ran out.
fn announce_result(commands: &mut Commands, my_team: u8, winner: Option<u8>) {
    let words = &ui().words.arena;
    let title = match winner {
        Some(team) if team == my_team => words.victory.clone(),
        Some(_) => words.defeat.clone(),
        None => words.times_up.clone(),
    };
    let caption = winner.map(|team| fill(&words.team_wins, &[("team", &team_name(team))]));
    hud::announce(commands, caption.as_deref(), &title, hud::HEADLINE, RESULT_SECS).insert(InArena);
}

/// How long the result is across the screen, in seconds.
const RESULT_SECS: f32 = 4.0;

/// The end of a game here: everything it put up goes, your health and your
/// team with it, the anchors stand again, the camera goes back to how it
/// was; and `move_you`, you are sent back to the town where your seat
/// there arrives (offline; online the server has moved you already).
#[allow(clippy::too_many_arguments)]
fn finish(
    commands: &mut Commands,
    game: &Game,
    in_arena: &Query<Entity, With<InArena>>,
    you: Entity,
    bodies: &mut Whereabouts,
    orbit: &mut OrbitCamera,
    anchors: Option<&mut Anchors>,
    islands: &mut Islands,
    visibilities: &mut Query<&mut Visibility>,
    move_you: bool,
) {
    commands.remove_resource::<Game>();
    for thing in in_arena {
        commands.entity(thing).try_despawn();
    }
    commands.entity(you).remove::<(Health, Dead, Team, Carrying)>();
    if move_you {
        commands.entity(you).insert(Seat(game.town_seat));
        lobby::send(bodies, orbit, you, Venue::Town, game.town_seat);
    }
    match game.view {
        Some(view) => orbit.back_to(view),
        None => *orbit = OrbitCamera::behind(&island::arrival(Venue::Town, game.town_seat)),
    }
    if let Some(anchors) = anchors {
        stand_anchors_up(anchors, islands, visibilities);
    }
}

/// Says how the fight stands, for the bridges (`bridge`) and the network
/// (`net`).
fn publish(game: Option<Res<Game>>, mut fighting: ResMut<Fighting>, mut offline: ResMut<OfflineArena>) {
    let now = match game.as_deref() {
        Some(game) => Fighting {
            playing: game.playing(),
            seat: Some(game.me as u8),
            online: game.online,
            game: game.id,
        },
        None => Fighting {
            game: fighting.game,
            ..default()
        },
    };
    fighting.set_if_neq(now);
    offline.set_if_neq(OfflineArena(game.is_some_and(|game| !game.online)));
}

// --------------------------------------------------------------- the fight

/// Whether the fight is this device's own to run, and on.
fn offline_fight(game: &Option<ResMut<Game>>) -> bool {
    game.as_deref().is_some_and(|game| !game.online && game.playing())
}

/// Gives everyone in the game their team: yours and the bots' as they
/// come, and online everyone the server seats, by their seat, once it
/// has said who is in which.
fn team_up(
    game: Option<Res<Game>>,
    others: Query<(Entity, &Member, Option<&Team>), (With<Remote>, With<WalkCycle>)>,
    mut commands: Commands,
) {
    let Some(game) = game else {
        return;
    };
    if !game.online {
        return;
    }
    for (body, member, team) in &others {
        let Some(seat) = game.seats.iter().position(|seated| seated.id == member.id) else {
            continue;
        };
        let theirs = Team(arena_team(seat));
        if team != Some(&theirs) {
            commands.entity(body).insert(theirs);
        }
    }
}

/// Sends each bot for whoever of another team is nearest within its sight,
/// while the game is on, offline: at a run up to them, over whatever walk
/// it was on, pressing in on them once there, and a punch whenever they
/// are in its reach and it may throw one, which knocks them back as yours
/// does ([`Shove::knock`]) and lands ([`Landed`]) for the damage to be
/// dealt. It sees no further than the file's `arena.hunt_radius`, the dead
/// not at all, and never a teammate; walking off its island it is kept
/// from, as ever (`StaysAshore`), but onto a bridge it will go. With less
/// than the file's `arena.flee_below` of its health it runs away from them
/// instead ([`flight`]), until its health has come back ([`regen`]).
#[allow(clippy::too_many_arguments)]
fn hunt(
    time: Res<Time>,
    game: Option<ResMut<Game>>,
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
        Option<&Team>,
        Has<Dead>,
    )>,
    mut landed: MessageWriter<Landed>,
    mut commands: Commands,
) {
    if !offline_fight(&game) {
        return;
    }
    let file = ui();
    let (tuning, melee) = (file.arena, file.melee);
    let sector = Sector::of(&melee);
    let dt = time.delta_secs();
    let alive: Vec<(Entity, Vec3, Option<Team>)> = bodies
        .iter()
        .filter(|(_, _, venue, .., dead)| **venue == Venue::Arena && !dead)
        .map(|(body, at, _, _, _, _, _, _, _, team, _)| (body, at.translation, team.copied()))
        .collect();
    let hunters: Vec<Entity> = bodies
        .iter()
        .filter(|(_, _, venue, _, _, _, hunter, _, _, _, dead)| **venue == Venue::Arena && hunter.is_some() && !dead)
        .map(|(body, ..)| body)
        .collect();
    for bot in hunters {
        // What the bot does this frame, with its own body in hand; the
        // punch is thrown once that is let go of.
        let punch = {
            let Ok((_, body, _, mut intent, _, _, Some(mut hunter), steering, health, team, _)) = bodies.get_mut(bot)
            else {
                continue;
            };
            hunter.cooldown = (hunter.cooldown - dt).max(0.0);
            let here = body.translation;
            let mine = team.copied();
            let Some((target, there, _)) = alive
                .iter()
                .filter(|&&(other, at, theirs)| {
                    other != bot
                        && !(mine.is_some() && theirs == mine)
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

/// Whoever has fallen far enough into the void is dead ([`void_y`]): you,
/// or a bot knocked off its island. Online your fall is told to the server,
/// which decides whether you come back; everyone else's is theirs.
fn reap(
    game: Option<ResMut<Game>>,
    fallen: Query<(Entity, &Transform, &Venue, &Seat, Has<Player>, Has<Remote>), (With<WalkCycle>, Without<Dead>)>,
    mut requests: MessageWriter<NetAsk>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        return;
    };
    if !game.playing() {
        return;
    }
    let void = void_y();
    for (body, transform, &venue, seat, mine, remote) in &fallen {
        if venue != Venue::Arena || transform.translation.y >= void {
            continue;
        }
        if game.online {
            if mine {
                // Counting down to standing again if the anchor stands,
                // for the eye; the server has the last word.
                let anchor = game.anchors[usize::from(arena_team(seat.0))];
                let left = if anchor > 0.0 { rules::RESPAWN_SECS } else { f32::INFINITY };
                commands.entity(body).insert(Dead { left });
                requests.write(NetAsk::Respawn);
                info!("fell into the void");
            } else if !remote {
                commands.entity(body).insert(Dead { left: f32::INFINITY });
            }
            continue;
        }
        let dead = game.die(seat.0);
        commands.entity(body).insert(dead);
        info!("{body} fell into the void");
    }
}

/// Deals every punch that landed ([`Landed`]) its damage, while the game is
/// on, offline: the file's `arena.damage` off the victim's health, a number
/// floating up off them, and death at none.
fn deal_damage(
    mut hits: MessageReader<Landed>,
    game: Option<ResMut<Game>>,
    mut victims: Query<(&Transform, &mut Health, &Seat), Without<Dead>>,
    mut commands: Commands,
) {
    if !offline_fight(&game) {
        hits.clear();
        return;
    }
    let mut game = game.expect("checked");
    for hit in hits.read() {
        let Ok((at, mut health, seat)) = victims.get_mut(hit.victim) else {
            continue;
        };
        let damage = ui().arena.damage;
        health.take(damage);
        spawn_number(&mut commands, at.translation, damage);
        if health.gone() {
            let dead = game.die(seat.0);
            commands.entity(hit.victim).insert(dead);
            info!("{} killed {}", hit.by, hit.victim);
        }
    }
}

/// Lands every punch you throw ([`Swung`]) on an enemy team's anchor in its
/// range, as it would land on a body that wide, while the game is on: the
/// damage off the anchor offline, or asked of the server online, and a
/// number off the pillar either way. No oftener than a body takes a punch
/// (`melee.hurt_time`): a press lands one every frame otherwise.
fn hit_anchors(
    time: Res<Time>,
    mut swings: MessageReader<Swung>,
    game: Option<ResMut<Game>>,
    mut requests: MessageWriter<NetAsk>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        swings.clear();
        return;
    };
    if !game.playing() {
        swings.clear();
        return;
    }
    let dt = time.delta_secs();
    for hurt in &mut game.anchor_hurt {
        *hurt = (*hurt - dt).max(0.0);
    }
    let file = ui();
    let sector = Sector::of(&file.melee);
    let damage = file.arena.damage;
    let my_team = game.my_team();
    for swing in swings.read() {
        if (swing.feet - island::LAND_TOP_OF_THE_ISLANDS).abs() > PLAYER_HEIGHT {
            continue;
        }
        for team in 0..ARENA_TEAMS {
            if team as u8 == my_team || game.anchors[team] <= 0.0 || game.anchor_hurt[team] > 0.0 {
                continue;
            }
            let anchor = ARENA_ANCHORS[team];
            if !sector.holds(swing.facing, anchor - swing.at, ANCHOR_RADIUS) {
                continue;
            }
            game.anchor_hurt[team] = file.melee.hurt_time;
            spawn_number(&mut commands, Vec3::new(anchor.x, 1.2, anchor.y), damage);
            if game.online {
                requests.write(NetAsk::HitAnchor { team: team as u8 });
            } else {
                game.anchors[team] = (game.anchors[team] - damage).max(0.0);
                if game.anchors[team] <= 0.0 {
                    info!("team {team}'s anchor is broken");
                }
            }
        }
    }
}

/// Gives everyone alive their health back, a little a second, once nobody
/// has hit them for a while, offline: the file's `arena.regen_per_sec`
/// after `arena.regen_after`. The while is counted without marking the
/// health changed, so that a bar at rest is left alone.
fn regen(time: Res<Time>, game: Option<ResMut<Game>>, mut bodies: Query<&mut Health, Without<Dead>>) {
    if !offline_fight(&game) {
        return;
    }
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

/// Puts the dead back where their seat arrives with their health full,
/// once their while is up, offline: never those out for good, whose while
/// never ends.
fn respawn(
    time: Res<Time>,
    game: Option<ResMut<Game>>,
    mut dead: Query<(Entity, &mut Dead, &Seat, &mut Health)>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    mut commands: Commands,
) {
    if !offline_fight(&game) {
        return;
    }
    let dt = time.delta_secs();
    let mut back = Vec::new();
    for (body, mut dead, seat, mut health) in &mut dead {
        if dead.left.is_infinite() {
            continue;
        }
        dead.left -= dt;
        if dead.left <= 0.0 {
            *health = Health::of(ui().arena.health);
            back.push((body, seat.0));
        }
    }
    for (body, seat) in back {
        commands.entity(body).remove::<Dead>();
        lobby::send(&mut bodies, &mut orbit, body, Venue::Arena, seat);
        info!("{body} stands again on island {}", roundtown_net::arena_island(seat));
    }
}

/// Takes a broken anchor out of the map: its pillar hidden and taken out of
/// what is stood on, and a headline saying whose it was.
fn break_anchors(
    game: Option<Res<Game>>,
    anchors: Option<ResMut<Anchors>>,
    mut islands: ResMut<Islands>,
    mut visibilities: Query<&mut Visibility>,
    mut commands: Commands,
) {
    let (Some(game), Some(mut anchors)) = (game, anchors) else {
        return;
    };
    let mut broke = false;
    for team in 0..ARENA_TEAMS {
        if game.anchors[team] > 0.0 || anchors.broken[team] {
            continue;
        }
        anchors.broken[team] = true;
        broke = true;
        for &pillar in &anchors.pillars[team] {
            if let Ok(mut visibility) = visibilities.get_mut(pillar) {
                visibility.set_if_neq(Visibility::Hidden);
            }
        }
        let words = &ui().words.arena;
        let line = if team as u8 == game.my_team() {
            words.your_anchor_destroyed.clone()
        } else {
            fill(&words.anchor_destroyed, &[("team", &team_name(team as u8))])
        };
        hud::announce(&mut commands, None, &line, hud::CAPTION, ANCHOR_SECS).insert(InArena);
    }
    if !broke {
        return;
    }
    // The map without the broken pillars, for bodies to stand on.
    let original = match &anchors.original {
        Some(original) => original.clone(),
        None => {
            let Some(corners) = islands.corners(Venue::Arena) else {
                return;
            };
            let corners = corners.to_vec();
            anchors.original = Some(corners.clone());
            corners
        }
    };
    let broken: Vec<Vec2> = (0..ARENA_TEAMS)
        .filter(|&team| anchors.broken[team])
        .map(|team| ARENA_ANCHORS[team])
        .collect();
    let remaining: Vec<[Vec3; 3]> = original
        .into_iter()
        .filter(|triangle| !is_pillar(triangle, &broken))
        .collect();
    islands.reshape(Venue::Arena, remaining, &[]);
}

/// Whether `triangle` is of one of the pillars at `anchors`: every corner of
/// it within [`ANCHOR_REACH`] of the anchor across the ground, and some of
/// it higher than the slab the pillar stands on reaches ([`PILLAR_OVER`]):
/// the pillar's lowest point is 0.54 m up and its highest 3.96, the slab's
/// top 0.75.
fn is_pillar(triangle: &[Vec3; 3], anchors: &[Vec2]) -> bool {
    anchors.iter().any(|anchor| {
        triangle.iter().all(|corner| corner.xz().distance(*anchor) <= ANCHOR_REACH)
            && triangle.iter().any(|corner| corner.y > PILLAR_OVER)
    })
}

/// How high a triangle of a pillar reaches at the least, in metres, which
/// nothing else within an anchor's reach does: the slab under it tops at
/// 0.75.
const PILLAR_OVER: f32 = 1.0;

/// Stands every anchor back up, as the game ends: the pillars shown, and
/// the map as it was to stand on.
fn stand_anchors_up(anchors: &mut Anchors, islands: &mut Islands, visibilities: &mut Query<&mut Visibility>) {
    if !anchors.broken.iter().any(|&broken| broken) {
        return;
    }
    anchors.broken = [false; ARENA_TEAMS];
    for pillar in anchors.pillars.iter().flatten() {
        if let Ok(mut visibility) = visibilities.get_mut(*pillar) {
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
    if let Some(original) = anchors.original.clone() {
        islands.reshape(Venue::Arena, original, &[]);
    }
}

/// "Team 3": `team` as it is written on the screen, counted from 1.
fn team_name(team: u8) -> String {
    fill(&ui().words.arena.team, &[("team", &(team + 1).to_string())])
}

// ---------------------------------------------------- what is drawn over it

/// Puts "You died. Respawning in 4..." across the screen while you are
/// dead and coming back, counting down in halves, and takes it away as you
/// stand again. Out for good, the panel says so instead ([`show_panel`]).
fn show_death(
    time: Res<Time>,
    game: Option<Res<Game>>,
    mut me: Query<&mut Dead, With<Player>>,
    texts: Query<Entity, With<DeathText>>,
    mut lines: Query<&mut Text, With<DeathLine>>,
    mut commands: Commands,
) {
    let dead = me.single_mut().ok();
    let text = texts.single().ok();
    let waiting = dead.filter(|dead| dead.left.is_finite());
    match (waiting, text) {
        (Some(mut dead), Some(_)) => {
            // Online the server stands you up; the count is for the eye.
            if game.is_some_and(|game| game.online) {
                dead.left = (dead.left - time.delta_secs()).max(0.0);
            }
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

/// Keeps the panel in the middle of the screen while you are out for good,
/// with Play again and Exit on it, and none otherwise.
fn show_panel(game: Option<Res<Game>>, panels: Query<Entity, With<Panel>>, mut commands: Commands) {
    let wanted = game.as_deref().is_some_and(|game| game.out.get(game.me).copied().unwrap_or(false));
    let up = panels.single().ok();
    match (wanted, up) {
        (true, None) => spawn_panel(&mut commands),
        (false, Some(panel)) => commands.entity(panel).despawn(),
        _ => {}
    }
}

/// The panel saying you are out: its title, its line, and Play again and
/// Exit, in the middle of the screen. Not a dialog: the world goes on
/// round it, and only a press on it is its own.
fn spawn_panel(commands: &mut Commands) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.arena);
    commands
        .spawn((
            InArena,
            Panel,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Vh(26.0),
                width: Val::VMin(PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(1.6),
                padding: UiRect::all(Val::VMin(3.0)),
                border_radius: BorderRadius::all(Val::VMin(3.5)),
                ..default()
            },
            // Half its own width back, so that it is centred.
            UiTransform::from_translation(Val2::new(Val::Percent(-50.0), Val::ZERO)),
            BackgroundColor(colours.house_builder.results),
            GlobalZIndex(builder::DIALOG_LAYER - 1),
        ))
        .with_children(|panel| {
            panel.spawn(hud::words(words.out.clone(), PANEL_TITLE, colours.gold));
            panel.spawn(builder::label(words.anchor_gone.clone(), PANEL_TEXT, colours.text));
            panel
                .spawn((
                    Node {
                        justify_content: JustifyContent::Center,
                        column_gap: Val::VMin(3.0),
                        margin: UiRect::top(Val::VMin(1.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|choices| {
                    choices.spawn(panel_button(
                        Ask::Exit,
                        words.exit.clone(),
                        Fill {
                            idle: colours.quiet,
                            lit: colours.quiet_lit,
                        },
                        colours.text,
                    ));
                    choices.spawn(panel_button(
                        Ask::Again,
                        words.play_again.clone(),
                        Fill {
                            idle: colours.gold,
                            lit: colours.gold_lit,
                        },
                        colours.on_gold,
                    ));
                });
        });
}

/// One of the panel's buttons.
fn panel_button(ask: Ask, text: String, fill: Fill, ink: Color) -> impl Bundle {
    (
        AsksFor(ask),
        TakesPress,
        Button,
        Node {
            width: Val::VMin(PANEL_BUTTON_WIDTH),
            height: Val::VMin(PANEL_BUTTON),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(PANEL_BUTTON * 0.5)),
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![builder::label(text, PANEL_BUTTON_TEXT, ink)],
    )
}

/// A name and a health bar over the head of everyone else in the game with
/// health and a team: made as they come, in a teammate's colours or an
/// enemy's, kept over their heads as the camera moves, the bar filled as
/// far as their health goes, and put away while they are dead or off the
/// screen, or you are not in the game. Small and faint, so as not to hide
/// the fight.
#[allow(clippy::too_many_arguments)]
fn place_tags(
    me: Query<(&Venue, Option<&Team>), With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    bodies: Query<(Entity, &Transform, &Health, &Team, &Name, Has<Dead>), Without<Player>>,
    mut tags: Query<(Entity, &TagOf, &mut Visibility, &mut UiTransform, &ComputedNode, &Children)>,
    mut fills: Query<&mut Node, With<TagFill>>,
    mut commands: Commands,
) {
    let (here, my_team) = me.single().map(|(venue, team)| (Some(*venue), team.copied())).unwrap_or((None, None));
    let in_arena = here == Some(Venue::Arena);
    if in_arena {
        for (body, _, _, team, name, _) in &bodies {
            if !tags.iter().any(|(_, of, ..)| of.0 == body) {
                spawn_tag(&mut commands, body, name.as_str(), Some(*team) == my_team);
            }
        }
    }
    let (Ok((camera, eye)), Ok(window)) = (cameras.single(), windows.single()) else {
        return;
    };
    let eye = GlobalTransform::from(*eye);
    for (tag, of, mut shown, mut place, node, children) in &mut tags {
        let Ok((_, body, health, .., dead)) = bodies.get(of.0) else {
            commands.entity(tag).despawn();
            continue;
        };
        let point = (in_arena && !dead)
            .then(|| camera.world_to_viewport(&eye, body.translation + Vec3::Y * TAG_OVER).ok())
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
        let width = bar_width(health.share());
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

/// The name and the bar over `body`'s head, in a teammate's colours if
/// `ally`, or an enemy's.
fn spawn_tag(commands: &mut Commands, body: Entity, name: &str, ally: bool) {
    let colours = ui().colours;
    let arena = colours.arena;
    let (ink, hp) = if ally { (arena.ally, arena.hp_ally) } else { (arena.enemy, arena.hp_enemy) };
    commands
        .spawn((
            InArena,
            TagOf(body),
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(0.25),
                ..default()
            },
            UiTransform::default(),
            Visibility::Hidden,
            GlobalZIndex(-1),
            Pickable::IGNORE,
        ))
        .with_children(|tag| {
            tag.spawn(hud::words(name, TAG_TEXT, ink));
            tag.spawn((
                Node {
                    width: Val::VMin(BAR_WIDTH),
                    height: Val::VMin(BAR_HEIGHT),
                    border: UiRect::all(Val::VMin(BAR_RIM)),
                    border_radius: BorderRadius::all(Val::VMin(0.4)),
                    ..default()
                },
                BackgroundColor(arena.hp_back),
                BorderColor::all(arena.hp_rim),
                Pickable::IGNORE,
                children![(
                    TagFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        border_radius: BorderRadius::all(Val::VMin(0.25)),
                        ..default()
                    },
                    BackgroundColor(hp),
                    Pickable::IGNORE,
                )],
            ));
        });
}

/// A bar over each team's anchor, in your team's colour or an enemy's.
fn spawn_anchor_bars(commands: &mut Commands, my_team: u8) {
    let colours = ui().colours.arena;
    for team in 0..ARENA_TEAMS as u8 {
        let tint = if team == my_team { colours.anchor_mine } else { colours.anchor_enemy };
        commands.spawn((
            InArena,
            AnchorBar(team),
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                width: Val::VMin(ANCHOR_BAR_WIDTH),
                height: Val::VMin(ANCHOR_BAR_HEIGHT),
                border: UiRect::all(Val::VMin(BAR_RIM)),
                border_radius: BorderRadius::all(Val::VMin(0.5)),
                ..default()
            },
            BackgroundColor(colours.hp_back),
            BorderColor::all(colours.hp_rim),
            UiTransform::default(),
            Visibility::Hidden,
            GlobalZIndex(-1),
            Pickable::IGNORE,
            children![(
                AnchorFill,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::VMin(0.3)),
                    ..default()
                },
                BackgroundColor(tint),
                Pickable::IGNORE,
            )],
        ));
    }
}

/// Keeps each anchor's bar over its pillar, filled as far as the anchor
/// goes, and put away once it is broken, or off the screen, or you are not
/// in the game.
fn place_anchor_bars(
    game: Option<Res<Game>>,
    me: Query<&Venue, With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut bars: Query<(&AnchorBar, &mut Visibility, &mut UiTransform, &ComputedNode, &Children)>,
    mut fills: Query<&mut Node, With<AnchorFill>>,
) {
    let (Some(game), Ok((camera, eye)), Ok(window)) = (game, cameras.single(), windows.single()) else {
        return;
    };
    let in_arena = me.single().ok() == Some(&Venue::Arena) && game.playing();
    let full = if game.online { rules::ANCHOR_HEALTH } else { ui().arena.anchor_health };
    let eye = GlobalTransform::from(*eye);
    for (bar, mut shown, mut place, node, children) in &mut bars {
        let team = usize::from(bar.0);
        let anchor = ARENA_ANCHORS[team];
        let point = (in_arena && game.anchors[team] > 0.0)
            .then(|| camera.world_to_viewport(&eye, Vec3::new(anchor.x, ANCHOR_BAR_OVER, anchor.y)).ok())
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
        let width = bar_width(game.anchors[team] / full.max(1e-3));
        for child in children.iter() {
            if let Ok(mut fill) = fills.get_mut(child)
                && fill.width != width
            {
                fill.width = width;
            }
        }
    }
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
        Stage::Playing { left } | Stage::Over { left, .. } => (clock(left), words.name.clone()),
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

    /// The whole map, every mesh of it but the bodies'.
    fn hajuns_map() -> Island {
        let parts = crate::island::glb::read("pvptownmap.glb");
        let corners: Vec<[Vec3; 3]> = parts
            .iter()
            .filter(|part| !part.skinned)
            .flat_map(|part| part.triangles.iter().copied())
            .collect();
        Island::new(&corners)
    }

    #[test]
    fn the_map_s_waiting_room_has_a_floor_and_walls_too_thick_to_slip_through_and_too_high_to_jump() {
        let (room, shape) = hajuns_room();
        let floor = room.floor(&shape).expect("a floor under the middle of the room");
        // Over the map, well clear of the islands, and the walls over the
        // floor by more than a jump reaches (1.64 m); and where the server
        // puts everyone, having no map to read it off.
        assert!(floor > 20.0, "floor at {floor}");
        assert!((floor - rules::WAITING_FLOOR).abs() < 0.01, "floor at {floor}, the server says {}", rules::WAITING_FLOOR);
        assert!(room.centre.length() < 0.01, "the room is at {}", room.centre);
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
    fn every_arrival_is_dry_land_in_the_map_with_the_void_beyond_the_islands() {
        // Everyone arrives on their island (Hajun, the night of 2026-11-07:
        // "make the spawnpoint in the center of the island"): there is
        // ground under each of the eight, about the islands' top, 0.25 m
        // up, or the desk that lies across the middle, and the void beside
        // the islands, where the bridges were until 2026-11-08.
        let map = hajuns_map();
        for seat in 0..ARENA_SEATS {
            let at = island::arrival(Venue::Arena, seat).translation;
            let ground = map.floor(at.xz(), 3.0).unwrap_or_else(|| panic!("seat {seat}: nothing under {at}"));
            assert!((0.0..=1.3).contains(&ground), "seat {seat}: ground at {ground}");
        }
        for team in 0..ARENA_TEAMS {
            let island = roundtown_net::ARENA_ISLANDS[team];
            let out = island.normalize() * 12.0;
            assert!(map.floor(island + out, 3.0).is_none(), "team {team}: land 12 m past the island's middle");
        }
        // No bridge: between an island and the town there is nothing to
        // stand on.
        for team in 0..ARENA_TEAMS {
            let island = roundtown_net::ARENA_ISLANDS[team];
            let between = island * (100.0 / 130.0);
            assert!(map.floor(between, 3.0).is_none(), "team {team}: ground between its island and the town at {between}");
        }
    }

    #[test]
    fn the_anchors_stand_where_the_map_s_pillars_do() {
        // Each team's anchor (`roundtown_net::ARENA_ANCHORS`) is the
        // middle of the purple pillar on its island, read off the map.
        let parts = crate::island::glb::read("pvptownmap.glb");
        let pillars: Vec<Vec2> = parts
            .iter()
            .filter(|part| part.material.as_deref().is_some_and(|name| is_material(name, SPAWN_MATERIAL)))
            .map(|part| {
                let corners: Vec<Vec3> = part.triangles.iter().flatten().copied().collect();
                let (low, high) = corners.iter().fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), c| (low.min(*c), high.max(*c)));
                assert!(high.y > 3.5 && low.y > 0.3, "{}: {low} to {high}", part.node);
                ((low + high) * 0.5).xz()
            })
            .collect();
        assert_eq!(pillars.len(), ARENA_TEAMS, "{pillars:?}");
        for team in 0..ARENA_TEAMS {
            let anchor = ARENA_ANCHORS[team];
            let nearest = pillars.iter().map(|pillar| pillar.distance(anchor)).fold(f32::MAX, f32::min);
            assert!(nearest < 0.1, "team {team}: anchor {anchor} is {nearest} m from the nearest pillar");
            // On its own team's island.
            assert!(anchor.distance(roundtown_net::ARENA_ISLANDS[team]) < roundtown_net::ARENA_ISLAND_RADIUS);
            // And its pillar's triangles, and no others, are taken for it.
            let map = hajuns_map();
            let _ = map;
        }
        let all: Vec<[Vec3; 3]> = parts.iter().filter(|part| !part.skinned).flat_map(|part| part.triangles.iter().copied()).collect();
        let taken = all.iter().filter(|triangle| is_pillar(triangle, &ARENA_ANCHORS)).count();
        let pillar_triangles: usize = parts
            .iter()
            .filter(|part| part.material.as_deref().is_some_and(|name| is_material(name, SPAWN_MATERIAL)))
            .map(|part| part.triangles.len())
            .sum();
        assert_eq!(taken, pillar_triangles, "the pillars' triangles alone are taken out when they break");
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
        assert_eq!(bar_width(health.share()), Val::Percent(33.5));
    }

    #[test]
    fn dead_with_the_anchor_standing_comes_back_and_without_it_is_out() {
        let mut game = Game {
            id: 1,
            seats: (0..ARENA_SEATS)
                .map(|seat| Member {
                    id: seat as u64 + 1,
                    name: format!("Seat {seat}"),
                    look: "boy".into(),
                    ai: seat > 0,
                })
                .collect(),
            bodies: Vec::new(),
            town_seat: 0,
            me: 0,
            online: false,
            cut_off: None,
            stage: Stage::Playing { left: 100.0 },
            view: None,
            health: Vec::new(),
            anchors: [120.0; ARENA_TEAMS],
            anchor_hurt: [0.0; ARENA_TEAMS],
            out: [false; ARENA_SEATS],
        };
        assert_eq!(game.my_team(), 0);
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
        // Seat 1 dies with its anchor standing: back after a while.
        let dead = game.die(1);
        assert_eq!(dead.left, ui().arena.respawn_secs);
        assert!(!game.out[1]);
        // Team 2's anchor is broken: seat 2 dies for good, but seat 6
        // keeps the team in play.
        game.anchors[2] = 0.0;
        let dead = game.die(2);
        assert!(dead.left.is_infinite());
        assert!(game.out[2]);
        assert_eq!(game.teams_in_play(), vec![0, 1, 2, 3]);
        game.die(6);
        assert_eq!(game.teams_in_play(), vec![0, 1, 3]);
        // Every other team's anchor broken and everyone on them dead: team
        // 0 is left.
        for team in [1, 3] {
            game.anchors[team] = 0.0;
        }
        for seat in [1, 5, 3, 7] {
            game.die(seat);
        }
        assert_eq!(game.teams_in_play(), vec![0]);
        assert_eq!(team_name(2), "Team 3");
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
        let map = hajuns_map();
        let here = Vec3::new(6.0, 0.25, 130.0);
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
