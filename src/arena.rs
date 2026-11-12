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
//! the waiting room: Hajun's glass box floating 150 m over the middle of
//! the map, the object called [`WAITING_ROOM`] in `pvptownmap.glb` (30 m
//! square, its floor a metre thick with its top at 150.5 m, its walls
//! 1.5 m thick and 5 m high, open to the sky, in `glass` at a twentieth,
//! since their map of 2026-11-11; 100 m up, at a tenth, from their export
//! of 2026-11-07 23:21 until then), found by name once the map has spawned
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
//! in the town, the camera as it was. Until 2026-11-12 a bot went for
//! whoever of another team was within `arena.hunt_radius` ([`hunt`]), ran
//! away when nearly beaten ([`flight`]), and never left its island; the
//! map's own bridges went on 2026-11-08, and every bridge is built by
//! walking.
//!
//! **Since 2026-11-12 the computer's players play the game** (Hajun: "i
//! need the bots to be intelligent enough ... the difficulty need to
//! moderate - not too hard, not too easy"): they collect, shop, lay
//! bridges to the middle and the smaller islands, buy their team's
//! upgrades, raid, defend, back off and go for anchors, by one brain
//! (`roundtown_net::bots`, on a map baked from this one,
//! `roundtown_net::nav`): the server's for its own bots online, and this
//! device's for a game played offline ([`drive_bots`], [`Brains`]), where
//! each is a body put where the brain has it, in nobody's way
//! ([`Brained`]), as the server's are. Your anchor being hit is said in
//! the timetable's line ([`warn_of_the_anchor`]).
//!
//! **Since 2026-11-11 the game has generators, a shop and a timetable**
//! (Hajun: "i want to now make the generators functional in the pvpmap ...
//! read the bridgewarsgameevent.txt for events and generator rates"). The
//! rules are `roundtown_net::bridge_wars`'s, which the server plays by
//! online and this device by itself offline: a `Mint` of sixteen
//! generators on the map's pads makes silver and gold on each team's
//! island, sapphire on the four smaller islands and amethyst in the
//! middle, each as fast and as many as Hajun's file says; whoever walks
//! within the magnet's reach of a thing has it (offline [`gather`]; online
//! the server, whose [`Purses`] say what everyone holds), and what you
//! hold is counted top left ([`Resources`], `hud`); the things themselves
//! are drawn by `loot`, and the shop that sells bridge for silver is
//! `market`. Over each sapphire and amethyst generator its level is
//! written, and under it the seconds to its next thing, counting down
//! ([`write_over_generators`]). The game lasts 25 minutes, and on the way
//! its timetable happens ([`timetable`], `rules::TIMETABLE`), each thing
//! said in a line across the upper half of the screen ([`alert`];
//! "Sapphire generators have been upgraded to Lvl II.", the item's picture
//! after its name): the sapphire and amethyst generators
//! quicken at 2, 4, 6 and 8 minutes; at 10 every anchor left breaks; at
//! 15, sudden death, twelve zombies rise all over the map ([`Zombie`]:
//! the body everyone has, gone green, on nobody's team, going for whoever
//! is nearest and back up a while after it is put down; offline this
//! device's, online the server's, seated past the players' eight seats);
//! and at 25 it is a draw.
//!
//! Everything the game puts in the world or on the screen carries
//! [`InArena`] and goes when the game does. Your own body is the only thing
//! that comes back.


#[cfg(test)]
mod nav_bake;

use bevy::camera::primitives::Aabb;
use bevy::gltf::GltfMaterialName;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use bevy_replicon::prelude::Remote;
use roundtown_net::bots::{self, Seen, Squad};
use roundtown_net::bridge_wars::{
    self as rules, BOT_WAIT, COUNTDOWN, Flight, Happening, Mint, Outfits, PLAY_SECS, RESULTS_SECS,
};
use roundtown_net::nav::Nav;
use roundtown_net::{
    ANCHOR_RADIUS, ARENA_ANCHORS, ARENA_SEATS, ARENA_TEAMS, ArenaRound, ArenaStage, Ask as NetAsk, Item, Kit, Kits,
    Member, PAD_HALF, PAD_TOP, Purse, Purses, Seats, Tell, Upgraded, Upgrades, Use, Utility, Ware, arena_team,
};

use crate::ai::{self, Steering};
use crate::bridge;
use crate::builder::{self, Fill};
use crate::drive::Driving;
use crate::fire;
use crate::hud::{self, TakesPress};
use crate::island::{self, Island, Islands, Land, Venue, is_material};
use crate::lobby::{self, Lobby, Seat, Whereabouts};
use crate::loot;
use crate::net::Online;
use crate::range::Sector;
use crate::ui::{UiChanged, fill, ui};
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
/// The line that says what the timetable has just done ([`alert`]): how
/// big, small as Hajun asked ("a small alert"); how far down the screen it
/// is, as a share of the screen's height, "around middle of the upper half
/// of screen", under the board; and how long it is up, coming and going,
/// in seconds.
const ALERT_TEXT: f32 = 3.6;
const ALERT_DOWN: f32 = 24.0;
const ALERT_SECS: f32 = 5.0;
const ALERT_IN: f32 = 0.3;
const ALERT_OUT: f32 = 0.8;
/// The picture of the item such a line is about, after its name: how big,
/// a little over the words' height; and how far it and the words either
/// side of it stand apart, about a space.
const ALERT_PICTURE: f32 = 3.8;
const ALERT_GAP: f32 = 1.0;
/// What is written over and under a sapphire or an amethyst generator
/// ([`write_over_generators`]): how big, between the names over heads
/// and the alert's line (Hajun: "small enough so it doesnt cover the
/// amethysts or teh generator but large enough to be readable"; the
/// number is mine: 2 mm letters on a phone, a line about four pads wide,
/// which covers nothing of the pad, being over and under it); and
/// how far past the pad's edge the words begin, in metres across the
/// ground, clear of what lies on the pad and of what is dropping onto it.
const GENERATOR_TEXT: f32 = 3.0;
const GENERATOR_GAP: f32 = 0.55;
const PANEL_WIDTH: f32 = 58.0;
const PANEL_TITLE: f32 = 5.6;
const PANEL_TEXT: f32 = 3.4;
const PANEL_BUTTON: f32 = 8.4;
const PANEL_BUTTON_WIDTH: f32 = 22.0;
const PANEL_BUTTON_TEXT: f32 = 3.4;
/// How high over a body's feet its name and bar stand, in metres, how high
/// over the ground an anchor's bar stands over its pillar (0.65 m over its
/// top, 3.75 m up since the map of 2026-11-11; 4.6 over a top at 3.96
/// until then), and how high a damage number starts from and rises to
/// over its while, in seconds.
const TAG_OVER: f32 = 2.2;
const ANCHOR_BAR_OVER: f32 = 4.4;
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

/// A zombie's fighting mind in a game played offline, over its wandering
/// ([`hunt`]): how long until it may punch again, in seconds. Online the
/// server moves its own. Until 2026-11-12 every bot of a game played
/// offline had one, and kept to its island; the computer's players have
/// a brain of their own since ([`Brained`]).
#[derive(Component, Default)]
struct Hunter {
    cooldown: f32,
}

/// One of the computer's players in a game played offline (since
/// 2026-11-12): a body put where the game's brain has it each frame
/// ([`drive_bots`], [`Brains`]), as the server's bots are online, rather
/// than walked by `move_bodies` (`lobby::spawn_bot_at`).
#[derive(Component)]
struct Brained;

/// The computer's players in a game played offline: the map as they know
/// it, with every bridge laid on it as it is built (`nav`); the brain of
/// each (`roundtown_net::bots`, the very one the server plays its own by
/// online, with the same numbers, `Skill::MODERATE`); what each holds, by
/// seat, as yours is in [`Resources`]; which of them were dead a frame
/// ago, to tell a death and a standing again by; and which of the bridges
/// they have been told of (`bridge::Bridges::version`).
struct Brains {
    nav: Nav,
    squad: Squad,
    purses: [Purse; ARENA_SEATS],
    dead: [bool; ARENA_SEATS],
    heard: Option<u32>,
    /// How many rungs each has laid ([`MOST_BOT_RUNGS`]).
    rungs: [usize; ARENA_SEATS],
}

impl Brains {
    fn new() -> Self {
        Self {
            nav: Nav::arena(),
            squad: Squad::new(fastrand::u64(..), bots::Skill::MODERATE),
            purses: [Purse::default(); ARENA_SEATS],
            dead: [false; ARENA_SEATS],
            heard: None,
            rungs: [0; ARENA_SEATS],
        }
    }
}

/// The most rungs one of the computer's players lays in a game played
/// offline: the server's own limit for anyone (`MOST_RUNGS`).
const MOST_BOT_RUNGS: usize = 600;

/// What you have in a game of Bridge Wars: the metres of bridge left to
/// lay (`bridge`; the file's `arena.bridge_metres` as a game begins, none
/// since 2026-11-11, and 25 more for every 4 silver at the shop, `market`;
/// written in front of you as you build, and in the shop), and how many
/// you hold of each thing the generators make, counted top left (`hud`):
/// offline what you have picked up ([`gather`]), online what the server
/// says you hold ([`Purses`]). Dying loses all of it ([`lose_on_death`]).
#[derive(bevy::prelude::Resource, Default, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Resources {
    pub bridge_metres: f32,
    pub held: Purse,
}

impl Resources {
    /// How many of `item` you hold, as written beside its picture.
    pub(crate) fn count(&self, item: Item) -> String {
        self.held[item.index()].to_string()
    }

    /// The metres of bridge you have left, as written in front of you
    /// while you build: "50m".
    pub(crate) fn metres(&self) -> String {
        format!("{}m", self.bridge_metres.max(0.0).round())
    }
}

/// A zombie of sudden death (`rules::ZOMBIES`): a body on nobody's team
/// (`rules::ZOMBIE_TEAM`), in a seat past the players', drawn green
/// ([`paint_zombies`]). Offline this device moves it, as it does the
/// bots; online the server does.
#[derive(Component)]
pub(crate) struct Zombie;

/// What a body in a game of Bridge Wars has on it that is to be seen
/// (since 2026-11-12, `gear`): the sword in its hand and the shield round
/// it, by tier, 0 for none, and whether it has drunk the invisibility
/// potion and is not to be drawn for anyone else. On every body in a game
/// that is on ([`dress`]), from the game's kits: this device's offline, the
/// server's online.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Wears {
    pub(crate) sword: u8,
    pub(crate) shield: u8,
    pub(crate) unseen: bool,
}

/// What you have yourself, and what your team has bought, for the shops
/// (`market`) and the second button (`utility`): nothing out of a game.
#[derive(bevy::prelude::Resource, Clone, Copy, PartialEq, Debug, Default)]
pub(crate) struct Outfit {
    pub(crate) kit: Kit,
    pub(crate) team: Upgrades,
}

/// How far into the fight it is, in seconds: what a potion's time left is
/// counted from. Online it is the server's, to the second.
#[derive(bevy::prelude::Resource, Default)]
pub(crate) struct FightTime(pub(crate) f32);

/// You want to buy this, standing at a shop (`market`): out of what you
/// hold here, offline, or asked of the server.
#[derive(Message, Clone, Copy)]
pub(crate) struct Want(pub(crate) Ware);

/// You have used what is on one of your buttons (`utility`): a fireball
/// thrown that way, as a plank heads; a builder sent off that way; or
/// this potion drunk.
#[derive(Message, Clone, Copy, Debug)]
pub(crate) enum Used {
    Fireball { heading: f32 },
    Builder { heading: f32 },
    Potion(Utility),
}

/// A zombie whose model has been turned green.
#[derive(Component)]
struct Painted;

/// What of a zombie is painted, by what each is called in its model
/// (`playercharacter.glb`), and the colour: its skin green, and its hair a
/// darker green, the top of its head being most of what the camera over
/// you sees of it.
const ZOMBIE_PAINT: [(&str, Color); 2] = [
    ("skin", Color::srgb(0.38, 0.6, 0.32)),
    ("hair", Color::srgb(0.12, 0.3, 0.1)),
];

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
    /// How far into the fight it is, in seconds: what the timetable goes
    /// by (`rules::TIMETABLE`). Offline counted here; online what the
    /// server's time left makes it, to the second.
    elapsed: f32,
    /// How far into the fight it was when the timetable was last looked
    /// at ([`timetable`]).
    said: f32,
    /// The generators, offline: online they are the server's.
    mint: Mint,
    /// What everyone has and what each team has bought (since
    /// 2026-11-12): this device's own to run offline, by the shops' rules,
    /// and online what the server says ([`follow_round`]).
    outfits: Outfits,
    /// The computer's players, offline ([`drive_bots`]); online they are
    /// the server's, and there are none here.
    brains: Option<Box<Brains>>,
    /// What was left of your team's anchor when it was last looked at,
    /// and when it was last said to be under attack, in seconds into the
    /// fight ([`warn_of_the_anchor`]).
    anchor_was: f32,
    anchor_said: f32,
    /// Online, whether you have the compass (`compass`): this device's
    /// own word of it, the server having sold it to you ([`hear`]) and
    /// not sending it with the room's kits (`roundtown_net::Kit::compass`),
    /// until you die. Offline it is in your kit as the rest is.
    compass: bool,
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
    /// if its team's anchor stands now, and is out for good otherwise; a
    /// zombie, in a seat past the players', is always back, a while later.
    /// What to mark its body dead with.
    fn die(&mut self, seat: usize) -> Dead {
        // Their sword and what was on their second button go with them.
        self.outfits.die(seat);
        if seat >= ARENA_SEATS {
            return Dead {
                left: rules::ZOMBIE_RESPAWN_SECS,
            };
        }
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

/// A line saying what the timetable has just done ([`alert`]), and how
/// long it has been up.
#[derive(Component)]
struct Alert {
    age: f32,
}
/// The picture in such a line, of the item it is about.
#[derive(Component)]
struct AlertPicture(Item);

/// What such a line says: words, and, where it is about an item, that
/// item's picture standing right after its name, with the rest of the
/// words after it (Hajun, late on 2026-11-11: "add the item icon after that item
/// name in that game event text", the picture its count has top left).
#[derive(Clone, PartialEq, Debug)]
struct Line {
    before: String,
    picture: Option<Item>,
    after: String,
}

impl Line {
    fn plain(words: &str) -> Self {
        Self {
            before: words.to_string(),
            picture: None,
            after: String::new(),
        }
    }

    /// The file's line for a generator's tier raised, `template`, with the
    /// item's `name` and its `level` filled in, cut after the name for the
    /// picture of `item` to stand there. A line that does not name the
    /// item has no picture.
    fn of_tier(template: &str, item: Item, name: &str, level: &str) -> Self {
        let fills = [("item", name), ("level", level)];
        match template.split_once("{item}") {
            Some((before, after)) => Self {
                before: format!("{}{name}", fill(before, &fills)),
                picture: Some(item),
                after: fill(after, &fills).trim_start().to_string(),
            },
            None => Self::plain(&fill(template, &fills)),
        }
    }
}

/// What is written by a sapphire or an amethyst generator
/// ([`write_over_generators`]): which generator's (`rules::generator`);
/// whether these are the words under its pad, the count to its next
/// thing, rather than those over it, its level; and the number they last
/// said, the seconds or the level, if they have said one.
#[derive(Component)]
struct GeneratorWords {
    generator: usize,
    under: bool,
    said: Option<u32>,
}

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
/// and how long ago; or off one that has been healed, by a potion or its
/// team's heal pool, in another colour.
#[derive(Component)]
struct DamageNumber {
    at: Vec3,
    age: f32,
    healed: bool,
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
        .add_message::<Want>()
        .add_message::<Used>()
        .init_resource::<Resources>()
        .init_resource::<Outfit>()
        .init_resource::<FightTime>()
        .init_resource::<Fighting>()
        .init_resource::<OfflineArena>()
        .init_resource::<Begun>()
        .add_observer(ask)
        .add_observer(read_map)
        .add_systems(
            Update,
            (hear, follow_round, cut_off, play, timetable, warn_of_the_anchor, gather, sell, use_things, publish, dress, show_board)
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
                (team_up, hunt, drive_bots)
                    .chain()
                    .after(ai::steer)
                    .after(crate::start_punch)
                    .before(crate::move_bodies),
                (reap, deal_damage, fireballs_offline, hit_anchors, regen, outfit_offline, respawn, break_anchors)
                    .chain()
                    .after(crate::move_bodies)
                    .after(fire::fly)
                    .before(crate::follow_camera),
                (show_death, show_panel, place_tags, place_anchor_bars, float_numbers, write_over_generators)
                    .after(crate::follow_camera),
                (paint_zombies, fade_alerts, lose_on_death, trap_alerts),
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
    (mut orbit, mut driving, mut stock): (ResMut<OrbitCamera>, ResMut<Driving>, ResMut<Resources>),
    mut commands: Commands,
) {
    let mut game = game;
    let mut anchors = anchors;
    for tell in tells.read() {
        let Ok((you, seat)) = me.single() else {
            continue;
        };
        match tell {
            // The shop has sold you a bridge: its price is out of what
            // you hold, as the room's purses say; the metres are yours.
            Tell::Bought(Ware::Bridge) => {
                if game.as_deref().is_some_and(|game| game.online) {
                    stock.bridge_metres += rules::BRIDGE_A_BUY;
                    info!("bought {} m of bridge", rules::BRIDGE_A_BUY);
                }
            }
            // The shop has sold you the compass: nobody is sent that but
            // you, so this device keeps its own word of it.
            Tell::Bought(Ware::Compass) => {
                if let Some(game) = game.as_deref_mut()
                    && game.online
                {
                    game.compass = true;
                    let me = game.me;
                    game.outfits.seat(me + 1);
                    game.outfits.kits[me].compass = true;
                    info!("bought the compass");
                }
            }
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
    rounds: Query<(&ArenaRound, &Seats, Option<&Purses>, Option<&Kits>, Option<&Upgraded>), With<Remote>>,
    mut stock: ResMut<Resources>,
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
    let Ok((round, seats, purses, kits, upgraded)) = rounds.single() else {
        return;
    };
    if game.seats != seats.0 {
        game.seats.clone_from(&seats.0);
    }
    // What everyone has, and what each team has bought, is the server's:
    // but for your compass, which is not sent with it, and is put back on
    // your own kit as this device has it.
    if let Some(kits) = kits {
        let (me, mine) = (game.me, game.compass);
        let same = game.outfits.kits.len() == kits.0.len()
            && game.outfits.kits.iter().zip(&kits.0).enumerate().all(|(seat, (here, theirs))| {
                *here
                    == Kit {
                        compass: seat == me && mine,
                        ..*theirs
                    }
            });
        if !same {
            game.outfits.kits.clone_from(&kits.0);
            if let Some(kit) = game.outfits.kits.get_mut(me) {
                kit.compass = mine;
            }
        }
    }
    if let Some(upgraded) = upgraded
        && game.outfits.teams != upgraded.0
    {
        game.outfits.teams = upgraded.0;
    }
    // What you hold is what the server says you hold.
    if let Some(mine) = purses.and_then(|purses| purses.0.get(game.me))
        && stock.held != *mine
    {
        stock.held = *mine;
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
    if let Stage::Playing { left } = stage {
        // The timetable's time is the server's, to the second.
        game.elapsed = PLAY_SECS - left;
        if !matches!(was, Stage::Playing { .. }) {
            game.said = game.elapsed;
            let mine = your_team(game.my_team());
            hud::announce(&mut commands, Some(&mine), &ui().words.arena.fight, hud::HEADLINE, FIGHT_SECS).insert(InArena);
        }
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
            // Dead, your compass is gone with the rest.
            if left <= 0.0 && game.compass {
                game.compass = false;
                let me = game.me;
                if let Some(kit) = game.outfits.kits.get_mut(me) {
                    kit.compass = false;
                }
            }
            if let Ok((you, at, mut mine, dead)) = me.single_mut() {
                if (mine.left - left).abs() > 1e-3 {
                    mine.left = left;
                    mine.full = rules::HEALTH;
                }
                if left < before {
                    spawn_number(&mut commands, at.translation, before - left);
                } else if healed_by(before, left) {
                    spawn_healed(&mut commands, at.translation, left - before);
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
        } else if healed_by(before, left) {
            spawn_healed(&mut commands, at.translation, left - before);
        }
        if left <= 0.0 && !dead {
            commands.entity(body).insert(Dead { left: f32::INFINITY });
        } else if left > 0.0 && dead {
            commands.entity(body).remove::<Dead>();
        }
    }
    game.health = health;
}

/// Whether health gone from `before` to `left` between two of the
/// server's words on it is a heal, a potion's or a pool's, rather than
/// what comes back by itself, a point or two at a time, or someone
/// standing again with all of it.
fn healed_by(before: f32, left: f32) -> bool {
    before > 0.0 && left - before >= rules::HEAL_POOL - 1.0
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
                    // A body the game's brain places, as the server's
                    // bots are placed online ([`drive_bots`]).
                    let body = lobby::spawn_bot_at(
                        &mut commands,
                        &assets,
                        &game.seats[seat],
                        seat,
                        Venue::Arena,
                        waiting_spot(room.as_ref(), map, seat),
                    );
                    commands.entity(body).insert((
                        InArena,
                        Health::of(ui().arena.health),
                        Brained,
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
            // The computer's players, each where its seat arrives.
            if let Some(brains) = game.brains.as_deref_mut() {
                for seat in (0..game.seats.len().min(ARENA_SEATS)).filter(|&seat| seat != game.me) {
                    let arrival = island::arrival(Venue::Arena, seat);
                    brains.squad.seat(seat, arrival.translation, yaw_of(arrival.rotation));
                }
            }
            let mine = your_team(game.my_team());
            hud::announce(&mut commands, Some(&mine), &ui().words.arena.fight, hud::HEADLINE, FIGHT_SECS).insert(InArena);
            // From the start, or, for trying the timetable out, from as
            // far into it as `RT_ARENA_FROM` says.
            // Begun late, everything before then happens at once, as
            // the fight begins ([`timetable`], which looks from 0).
            game.elapsed = fight_from();
            game.said = 0.0;
            game.mint = Mint::new(game.elapsed, &mut || fastrand::f32());
            let left = PLAY_SECS - game.elapsed;
            info!("Bridge Wars: everyone to their islands, {left} s to play");
            game.stage = Stage::Playing { left };
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
                None => {
                    game.elapsed += dt;
                    Stage::Playing { left: left - dt }
                }
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
    // What you have to build with: the file's metres of bridge, and
    // nothing in your purse (but for trying the shops out,
    // `RT_ARENA_PURSE`).
    commands.insert_resource(Resources {
        bridge_metres: ui().arena.bridge_metres,
        held: [purse_from(); 4],
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
        elapsed: 0.0,
        said: 0.0,
        mint: Mint::new(0.0, &mut || fastrand::f32()),
        outfits: {
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            outfits
        },
        brains: (!online).then(|| Box::new(Brains::new())),
        anchor_was: if online { rules::ANCHOR_HEALTH } else { ui().arena.anchor_health },
        anchor_said: f32::NEG_INFINITY,
        compass: false,
    }
}

/// How many of each thing you begin a game played offline with: none, but
/// for trying the shops out on desktop, where `RT_ARENA_PURSE=60` begins
/// it with sixty of each (the server has the same, for its own games).
fn purse_from() -> u16 {
    std::env::var("RT_ARENA_PURSE").ok().and_then(|count| count.parse().ok()).unwrap_or(0)
}

/// How far into its timetable a game played offline begins its fight, in
/// seconds: none, but for trying the timetable out on desktop, where
/// `RT_ARENA_FROM=595` begins it five seconds before every anchor breaks
/// (the server has the same, for its own games).
fn fight_from() -> f32 {
    std::env::var("RT_ARENA_FROM")
        .ok()
        .and_then(|seconds| seconds.parse::<f32>().ok())
        .filter(|seconds| seconds.is_finite() && *seconds >= 0.0)
        .map_or(0.0, |seconds| seconds.min(PLAY_SECS - 1.0))
}

/// Says each thing on the game's timetable as its time comes
/// (`rules::TIMETABLE`, Hajun's `bridgewarsgameevent.txt`), in a line
/// across the upper half of the screen ([`alert`]), online and offline
/// alike; and offline does it, the server doing it online: the tiers are
/// the generators' own by the time, every anchor left breaks, and the
/// zombies rise.
#[allow(clippy::too_many_arguments)]
fn timetable(
    game: Option<ResMut<Game>>,
    islands: Res<Islands>,
    assets: Res<AssetServer>,
    alerts: Query<Entity, With<Alert>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        return;
    };
    if !game.playing() || game.elapsed <= game.said {
        return;
    }
    let (before, now) = (game.said, game.elapsed);
    game.said = now;
    // One line at a time, the last: a game begun far into its timetable
    // (`RT_ARENA_FROM`) has everything before then happen at once.
    let mut said = None;
    for happening in rules::happening_between(before, now) {
        info!("Bridge Wars: {happening:?}, {now:.0} s in");
        let file = ui();
        let words = &file.words.arena;
        said = Some(match happening {
            Happening::Tier { item, tier } => Line::of_tier(&words.tier_up, item, words.item(item), roman(tier)),
            Happening::AnchorsBreak => Line::plain(&words.anchors_broken),
            Happening::SuddenDeath => Line::plain(&words.sudden_death),
        });
        if game.online {
            continue;
        }
        match happening {
            Happening::Tier { .. } => {}
            Happening::AnchorsBreak => game.anchors = [0.0; ARENA_TEAMS],
            Happening::SuddenDeath => {
                let map = islands.get(Venue::Arena);
                for zombie in 0..rules::ZOMBIES {
                    raise_zombie(&mut commands, &assets, &mut game, map, zombie);
                }
            }
        }
    }
    if let Some(line) = said {
        for old in &alerts {
            commands.entity(old).try_despawn();
        }
        alert(&mut commands, &mut images, &line);
    }
}

/// "I", "II", "III": a generator's tier as it is written.
fn roman(tier: u8) -> &'static str {
    match tier {
        0 | 1 => "I",
        2 => "II",
        3 => "III",
        _ => "IV",
    }
}

/// Where the zombie numbered `zombie` rises, and rises again, on `map`:
/// its spot (`rules::zombie_spot`) on whatever ground is there, facing the
/// middle of the ground it keeps to.
fn zombie_place(map: Option<&Island>, zombie: usize) -> Transform {
    let at = rules::zombie_spot(zombie);
    let ground = map
        .and_then(|map| map.floor(at, 3.0))
        .unwrap_or(island::LAND_TOP_OF_THE_ISLANDS);
    let (home, _) = rules::zombie_home(zombie);
    let toward = (home - at).normalize_or(Vec2::NEG_Y);
    Transform::from_xyz(at.x, ground, at.y).with_rotation(Quat::from_rotation_y((-toward.x).atan2(-toward.y)))
}

/// A zombie, offline, in the next seat past the players': a body like the
/// bots', with everyone's health, on nobody's team, that goes for whoever
/// is nearest ([`hunt`]).
fn raise_zombie(commands: &mut Commands, assets: &AssetServer, game: &mut Game, map: Option<&Island>, zombie: usize) {
    let seat = game.seats.len().max(ARENA_SEATS);
    let member = Member {
        id: seat as u64 + 1,
        name: rules::ZOMBIE_NAME.into(),
        look: rules::ZOMBIE_LOOK.into(),
        ai: true,
    };
    let body = lobby::spawn_player_at(commands, assets, &member, seat, Venue::Arena, false, zombie_place(map, zombie));
    commands.entity(body).insert((
        InArena,
        Zombie,
        Health::of(ui().arena.health),
        Hunter::default(),
        Team(rules::ZOMBIE_TEAM),
    ));
    game.seats.resize(seat, member.clone());
    game.seats.push(member);
    game.bodies.resize(seat, None);
    game.bodies.push(Some(body));
}

/// The generators, offline: what they make drops onto its pad (`loot`),
/// and whatever you walk within the magnet's reach of is yours, counted
/// top left. Online both are the server's (`loot::follow_server`,
/// [`follow_round`]). The computer's players take what their purchases
/// need of what is left, later in the frame ([`drive_bots`]): you are
/// served first.
fn gather(
    time: Res<Time>,
    game: Option<ResMut<Game>>,
    look: Res<loot::Look>,
    me: Query<(Entity, &Transform, Has<Dead>), With<Player>>,
    things: Query<(Entity, &loot::Thing)>,
    mut stock: ResMut<Resources>,
    mut commands: Commands,
) {
    if !offline_fight(&game) {
        return;
    }
    let mut game = game.expect("checked");
    let elapsed = game.elapsed;
    // Whatever is within your reach, dropping or lying
    // (`Mint::pick_up`): standing on the pad, you see each thing come
    // down, and it is yours as it comes to half a metre over your head,
    // sooner if you jump for it.
    if let Ok((you, at, dead)) = me.single()
        && !dead
    {
        for piece in game.mint.pick_up(at.translation, elapsed) {
            let held = &mut stock.held[piece.item.index()];
            *held = held.saturating_add(1);
            if let Some((thing, _)) = things.iter().find(|(_, thing)| thing.key == loot::Key::Own(piece.id)) {
                loot::take(&mut commands, thing, Some(you));
            }
        }
    }
    // Each team's own generator as fast as the team has raised it at the
    // upgrade shop: yours, offline, the computer buying nothing.
    let forge = game.outfits.teams.map(|team| team.forge);
    for piece in game.mint.run(elapsed, &forge, &mut || fastrand::f32()) {
        loot::lay(&mut commands, &look, time.elapsed_secs(), loot::Key::Own(piece.id), piece.item, piece.at);
    }
}

/// Sells you what you ask a shop for ([`Want`], `market`): offline out of
/// what you hold, by the shops' rules (`Outfits::buy`), a bridge's metres
/// with it; online asked of the server, which says so if it is yours
/// (`Tell::Bought`, [`hear`]) and sends what everyone has.
fn sell(
    mut wants: MessageReader<Want>,
    game: Option<ResMut<Game>>,
    mut stock: ResMut<Resources>,
    mut requests: MessageWriter<NetAsk>,
) {
    let Some(mut game) = game else {
        wants.clear();
        return;
    };
    for Want(ware) in wants.read().copied() {
        if !game.playing() {
            continue;
        }
        if game.online {
            requests.write(NetAsk::Buy(ware));
            continue;
        }
        let me = game.me;
        let mut held = stock.held;
        if game.outfits.buy(me, ware, &mut held) {
            stock.held = held;
            if ware == Ware::Bridge {
                stock.bridge_metres += rules::BRIDGE_A_BUY;
            }
            info!("bought {ware:?}");
        }
    }
}

/// Does what using the thing on your second button does ([`Used`],
/// `utility`): offline here, by the shops' rules, a fireball thrown from
/// where you stand (`fire`), a potion's healing given as it is drunk;
/// online asked of the server, which does the same for everyone to see.
/// A builder is this device's either way (`bridge::send_builder`): sent
/// off from where you stand, its bridge going to the server a rung at a
/// time, as yours do.
fn use_things(
    mut used: MessageReader<Used>,
    game: Option<ResMut<Game>>,
    look: Res<fire::Look>,
    assets: Res<AssetServer>,
    mut me: Query<(&Transform, &mut Health), With<Player>>,
    mut requests: MessageWriter<NetAsk>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        used.clear();
        return;
    };
    for used in used.read().copied() {
        if !game.playing() {
            continue;
        }
        let (seat, elapsed) = (game.me, game.elapsed);
        if game.online {
            requests.write(NetAsk::Use(match used {
                Used::Fireball { heading } => Use::Fireball { heading },
                Used::Builder { .. } => Use::Builder,
                Used::Potion(potion) => Use::Potion(potion),
            }));
            // The server takes the builder off your button; the builder
            // itself is yours to send (`utility` asks only with one there).
            if let Used::Builder { heading } = used
                && let (Ok((at, _)), Some(member)) = (me.single(), game.seats.get(seat))
            {
                bridge::send_builder(&mut commands, &assets, member, seat as u8, at.translation, heading);
            }
            continue;
        }
        match used {
            Used::Fireball { heading } => {
                if game.outfits.spend(seat, Utility::Fireball)
                    && let Ok((at, _)) = me.single()
                {
                    fire::throw(&mut commands, &look, Flight::thrown(at.translation, heading), seat, None);
                }
            }
            Used::Builder { heading } => {
                if game.outfits.spend(seat, Utility::Builder)
                    && let (Ok((at, _)), Some(member)) = (me.single(), game.seats.get(seat))
                {
                    bridge::send_builder(&mut commands, &assets, member, seat as u8, at.translation, heading);
                }
            }
            Used::Potion(potion) => {
                if game.outfits.drink(seat, potion, elapsed) == Some(Utility::Healing)
                    && let Ok((at, mut health)) = me.single_mut()
                {
                    let back = health.full * rules::HEALING;
                    if health.mend(back) {
                        spawn_healed(&mut commands, at.translation, back);
                    }
                }
            }
        }
    }
}

/// The potions, the traps and the heal pools, offline (`Outfits::tick`),
/// with every living player where they stand: whoever a pool heals has
/// that much health back.
fn outfit_offline(
    time: Res<Time>,
    game: Option<ResMut<Game>>,
    mut bodies: Query<(&Transform, &Venue, &Seat, &mut Health), (With<WalkCycle>, Without<Dead>)>,
    mut commands: Commands,
) {
    if !offline_fight(&game) {
        return;
    }
    let mut game = game.expect("checked");
    let standing: Vec<(usize, Vec2)> = bodies
        .iter()
        .filter(|(_, venue, ..)| **venue == Venue::Arena)
        .map(|(at, _, seat, _)| (seat.0, at.translation.xz()))
        .collect();
    let elapsed = game.elapsed;
    let ticked = game.outfits.tick(time.delta_secs(), elapsed, &standing);
    for (at, _, seat, mut health) in &mut bodies {
        if ticked.healed.contains(&seat.0) && health.mend(rules::HEAL_POOL) {
            spawn_healed(&mut commands, at.translation, rules::HEAL_POOL);
        }
    }
}

/// Flies this device's own fireballs, offline, to whoever of another team
/// than the thrower's they meet, or the end of their range, and bursts
/// them there (`fire` draws them, and flies them on each frame): the
/// blast's damage off everyone of another team it reaches, less what
/// their shields keep off, a number off each, and a knock away from where
/// it burst. Online the server does all of it, and its fireballs are only
/// drawn here.
fn fireballs_offline(
    game: Option<ResMut<Game>>,
    balls: Query<(Entity, &fire::Ball)>,
    mut bodies: Query<
        (Entity, &Transform, &Venue, &mut Health, &Seat, &mut Shove, &mut PlayerJump, Has<Zombie>),
        (With<WalkCycle>, Without<Dead>),
    >,
    mut commands: Commands,
) {
    if !offline_fight(&game) {
        return;
    }
    let mut game = game.expect("checked");
    for (ball, flying) in &balls {
        if flying.source.is_some() {
            continue;
        }
        let flight = flying.flight;
        let team = arena_team(flying.seat);
        let met = bodies
            .iter()
            .any(|(_, at, venue, _, seat, ..)| *venue == Venue::Arena && arena_team(seat.0) != team && flight.meets(at.translation));
        if !flying.spent() && !met {
            continue;
        }
        let burst = flight.at();
        for (body, at, venue, mut health, seat, mut shove, mut jump, zombie) in &mut bodies {
            if *venue != Venue::Arena || arena_team(seat.0) == team || !rules::in_blast(burst, at.translation) {
                continue;
            }
            let damage = game.outfits.through(seat.0, rules::FIREBALL_DAMAGE);
            game.outfits.reveal(seat.0);
            health.take(damage);
            spawn_number(&mut commands, at.translation, damage);
            // Knocked from where it burst, whatever they were reeling from.
            shove.hurt = 0.0;
            shove.knock(&mut jump, burst.xz(), at.translation.xz(), false);
            if zombie {
                shove.across *= rules::ZOMBIE_KNOCK;
                jump.velocity_y *= rules::ZOMBIE_KNOCK;
            }
            if health.gone() {
                let dead = game.die(seat.0);
                commands.entity(body).insert(dead);
            }
        }
        fire::burst(&mut commands, burst);
        commands.entity(ball).despawn();
    }
}

/// Puts on every body in a game that is on what it has that is to be seen
/// ([`Wears`]), by its seat, and says what you have yourself ([`Outfit`])
/// and how far into the fight it is ([`FightTime`]); out of a game, nobody
/// wears anything.
#[allow(clippy::type_complexity)]
fn dress(
    game: Option<Res<Game>>,
    mut outfit: ResMut<Outfit>,
    mut clock: ResMut<FightTime>,
    me: Query<(Entity, Option<&Wears>), With<Player>>,
    locals: Query<(Entity, &Seat, Option<&Wears>), (With<WalkCycle>, Without<Player>, Without<Remote>)>,
    remotes: Query<(Entity, &Member, Option<&Wears>), (With<Remote>, With<WalkCycle>)>,
    worn: Query<Entity, With<Wears>>,
    mut commands: Commands,
) {
    let Some(game) = game.filter(|game| game.playing()) else {
        for body in &worn {
            commands.entity(body).remove::<Wears>();
        }
        outfit.set_if_neq(Outfit::default());
        if clock.0 != 0.0 {
            clock.0 = 0.0;
        }
        return;
    };
    let elapsed = game.elapsed;
    if clock.0 != elapsed {
        clock.0 = elapsed;
    }
    let wears = |seat: usize| {
        let kit = game.outfits.kit(seat);
        Wears {
            sword: kit.sword,
            shield: kit.shield,
            unseen: kit.unseen(elapsed),
        }
    };
    let mut put = |body: Entity, had: Option<&Wears>, seat: usize| {
        let now = wears(seat);
        if had != Some(&now) {
            commands.entity(body).insert(now);
        }
    };
    if let Ok((you, had)) = me.single() {
        put(you, had, game.me);
    }
    if game.online {
        for (body, member, had) in &remotes {
            if let Some(seat) = game.seats.iter().position(|seated| seated.id == member.id) {
                put(body, had, seat);
            }
        }
    } else {
        for (body, seat, had) in &locals {
            put(body, had, seat.0);
        }
    }
    outfit.set_if_neq(Outfit {
        kit: game.outfits.kit(game.me),
        team: game.outfits.team(game.me),
    });
}

/// Dying loses what you bought at the item shop ("all items are lost upon
/// death"), and, since 2026-11-12, everything you hold of what the
/// generators make (Hajun: "death make u lose all ur silver, gold,
/// sapphire, amethyst, bridge meters"; until then those were kept). The
/// server, or this device's own game, takes your sword and what was on
/// your second button; the metres of bridge are this device's to take,
/// here, online too; and so, offline, is what you hold, which online the
/// server empties and says so ([`Purses`], [`follow_round`]).
fn lose_on_death(died: Query<(), (With<Player>, Added<Dead>)>, game: Option<Res<Game>>, mut stock: ResMut<Resources>) {
    if died.is_empty() {
        return;
    }
    let offline = game.is_some_and(|game| !game.online);
    let lost = lost_on_death(&stock, offline);
    if *stock != lost {
        *stock = lost;
    }
}

/// What is left of `stock` to someone who has died: no metres of bridge,
/// and, in a game played `offline`, nothing held of what the generators
/// make (online that is the server's to take).
fn lost_on_death(stock: &Resources, offline: bool) -> Resources {
    Resources {
        bridge_metres: 0.0,
        held: if offline { Purse::default() } else { stock.held },
    }
}

/// Says, in the line the timetable's are said in, that one of your team's
/// traps has blinded someone, and that one has blinded you.
fn trap_alerts(
    outfit: Res<Outfit>,
    mut was: Local<(bool, u8)>,
    alerts: Query<Entity, With<Alert>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let now = (outfit.kit.blind, outfit.team.traps);
    if now == *was {
        return;
    }
    let file = ui();
    let words = &file.words.arena;
    let said = if now.0 && !was.0 {
        Some(&words.blinded)
    } else if now.1 < was.1 {
        Some(&words.trap_sprung)
    } else {
        None
    };
    *was = now;
    if let Some(said) = said {
        for old in &alerts {
            commands.entity(old).try_despawn();
        }
        alert(&mut commands, &mut images, &Line::plain(said));
    }
}

/// A line across the upper half of the screen, small, saying what has just
/// happened in the game, up for a few seconds ([`fade_alerts`]; Hajun,
/// 2026-11-11: "it should show a small alert in around middle of the
/// upper half of screen that tells the event, for example: 'Sapphire
/// generators have been upgraded to Lvl II.'").
/// A line about an item has that item's picture after its name, the one
/// its count has top left, in the item's colour (since late on 2026-11-11).
fn alert(commands: &mut Commands, images: &mut Assets<Image>, line: &Line) {
    // Put up clear, to be brought up.
    let colours = ui().colours.arena;
    let ink = colours.alert.with_alpha(0.0);
    commands
        .spawn((
            InArena,
            Alert { age: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Vh(ALERT_DOWN),
                left: Val::ZERO,
                right: Val::ZERO,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::VMin(ALERT_GAP),
                ..default()
            },
            // Over the shop's window, should it be up.
            GlobalZIndex(builder::DIALOG_LAYER - 1),
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            if !line.before.is_empty() {
                row.spawn(hud::words(line.before.as_str(), ALERT_TEXT, ink));
            }
            if let Some(item) = line.picture {
                row.spawn((
                    AlertPicture(item),
                    ImageNode::new(images.add(hud::item_picture(item))).with_color(colours.item(item).with_alpha(0.0)),
                    Node {
                        width: Val::VMin(ALERT_PICTURE),
                        height: Val::VMin(ALERT_PICTURE),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
            if !line.after.is_empty() {
                row.spawn(hud::words(line.after.as_str(), ALERT_TEXT, ink));
            }
        });
}

/// Brings each alert up, holds it, and takes it away again, its picture
/// with its words.
fn fade_alerts(
    time: Res<Time>,
    mut alerts: Query<(Entity, &mut Alert, &Children)>,
    mut lines: Query<(&mut TextColor, &mut TextShadow)>,
    mut pictures: Query<(&AlertPicture, &mut ImageNode)>,
    mut commands: Commands,
) {
    let colours = ui().colours;
    let (ink, shadow) = (colours.arena.alert, colours.text_shadow);
    for (entity, mut alert, children) in &mut alerts {
        alert.age += time.delta_secs();
        if alert.age >= ALERT_SECS {
            commands.entity(entity).despawn();
            continue;
        }
        let solid = (alert.age / ALERT_IN).min((ALERT_SECS - alert.age) / ALERT_OUT).clamp(0.0, 1.0);
        for child in children.iter() {
            // Only while it is fading: held, the line is left alone.
            if let Ok((mut colour, mut under)) = lines.get_mut(child) {
                let faded = ink.with_alpha(ink.alpha() * solid);
                if colour.0 != faded {
                    colour.0 = faded;
                    under.color = shadow.with_alpha(shadow.alpha() * solid);
                }
            } else if let Ok((picture, mut image)) = pictures.get_mut(child) {
                let tint = colours.arena.item(picture.0);
                let faded = tint.with_alpha(tint.alpha() * solid);
                if image.color != faded {
                    image.color = faded;
                }
            }
        }
    }
}

/// Writes over each sapphire and amethyst generator what level it is at,
/// and under it how long until its next thing, counting down (since
/// late on 2026-11-11; Hajun: "for the sapphire and the amethyst generators, put a
/// white text above it that shows the generator level like Amethyst
/// Generator Lvl 2, and below it, put 'Next Amethyst in 24sec...' that
/// counts down"): two lines of small white words to each of the eight,
/// one ending a little north of its pad and one beginning a little south
/// of it, which under the camera fixed over you are over it and under it
/// on the screen, put up as the fight begins and gone with the game.
///
/// Nothing is sent for them. Both are the time's alone: the level by the
/// timetable (`rules::tier`), and the count to when the generator next
/// comes round (`rules::next_at`), which it does whether or not it is
/// full; full, it makes nothing then, and the count starts over. Online
/// the time is the server's, to the second (`follow_round`), so every
/// device writes the same count.
fn write_over_generators(
    game: Option<Res<Game>>,
    changed: Res<UiChanged>,
    me: Query<&Venue, With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut words: Query<(&mut GeneratorWords, &mut Text, &mut TextColor, &mut Visibility, &mut UiTransform, &ComputedNode)>,
    mut commands: Commands,
) {
    let Some(game) = game else {
        return;
    };
    let fighting = game.playing() && me.single().ok() == Some(&Venue::Arena);
    if words.is_empty() {
        if fighting {
            let ink = ui().colours.arena.generator;
            for generator in (0..rules::GENERATORS).filter(|index| rules::steady(rules::generator(*index).0)) {
                for under in [false, true] {
                    commands.spawn((
                        InArena,
                        GeneratorWords {
                            generator,
                            under,
                            said: None,
                        },
                        hud::words("", GENERATOR_TEXT, ink),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::ZERO,
                            top: Val::ZERO,
                            ..default()
                        },
                        UiTransform::default(),
                        Visibility::Hidden,
                        // Under the buttons and the windows, as the names
                        // over heads are.
                        GlobalZIndex(-1),
                    ));
                }
            }
        }
        return;
    }
    let (Ok((camera, eye)), Ok(window)) = (cameras.single(), windows.single()) else {
        return;
    };
    let eye = GlobalTransform::from(*eye);
    let file = ui();
    let names = &file.words.arena;
    // The file's words and colours, taken again as it is saved.
    let again = changed.is_changed() && !changed.is_added();
    for (mut label, mut text, mut colour, mut shown, mut place, node) in &mut words {
        let (item, pad) = rules::generator(label.generator);
        let off = if label.under { PAD_HALF + GENERATOR_GAP } else { -(PAD_HALF + GENERATOR_GAP) };
        let point = fighting
            .then(|| camera.world_to_viewport(&eye, Vec3::new(pad.x, PAD_TOP, pad.y + off)).ok())
            .flatten()
            .filter(|point| on_screen(window, *point));
        let Some(point) = point else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let number = if label.under {
            seconds_to_next(item, game.elapsed)
        } else {
            u32::from(rules::tier(item, game.elapsed))
        };
        if label.said != Some(number) || again {
            label.said = Some(number);
            let line = generator_line(names, item, label.under, number);
            if text.0 != line {
                text.0 = line;
            }
            colour.set_if_neq(TextColor(file.colours.arena.generator));
        }
        // Centred on the pad across the screen: the words over it end
        // where the point is, and those under it begin there.
        let size = node.size() * node.inverse_scale_factor();
        place.set_if_neq(UiTransform {
            translation: Val2::px(point.x - size.x * 0.5, if label.under { point.y } else { point.y - size.y }),
            scale: Vec2::ONE,
            rotation: Rot2::IDENTITY,
        });
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// How many seconds until a generator of `item` next comes round, as
/// written under it, `elapsed` seconds into the fight: whole seconds,
/// rounded up, so that the last second reads 1, and the moment it comes
/// the whole wait to the one after.
fn seconds_to_next(item: Item, elapsed: f32) -> u32 {
    (rules::next_at(item, elapsed) - elapsed).ceil().max(1.0) as u32
}

/// "Amethyst Generator Lvl 2", or, `under` the pad, "Next Amethyst in
/// 24sec...": the file's words with the item's name and `number` in them.
fn generator_line(words: &crate::ui::ArenaWords, item: Item, under: bool, number: u32) -> String {
    let (name, number) = (words.item(item), number.to_string());
    if under {
        fill(&words.generator_next, &[("item", name), ("seconds", &number)])
    } else {
        fill(&words.generator, &[("item", name), ("level", &number)])
    }
}

/// Turns each zombie green as its model spawns: its skin and its hair,
/// whatever it wears ([`ZOMBIE_PAINT`]). Each a copy of the model's own
/// material in the new colour, made once and shared by every zombie.
fn paint_zombies(
    zombies: Query<Entity, (With<Zombie>, Without<Painted>)>,
    children: Query<&Children>,
    parts: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut paints: Local<[Option<Handle<StandardMaterial>>; 2]>,
    mut commands: Commands,
) {
    for zombie in &zombies {
        let mut painted = false;
        for part in children.iter_descendants(zombie) {
            let Ok((name, worn)) = parts.get(part) else {
                continue;
            };
            let Some(which) = ZOMBIE_PAINT.iter().position(|(called, _)| is_material(&name.0, called)) else {
                continue;
            };
            let green = paints[which]
                .get_or_insert_with(|| {
                    let mut look = materials.get(&worn.0).cloned().unwrap_or_default();
                    look.base_color = ZOMBIE_PAINT[which].1;
                    materials.add(look)
                })
                .clone();
            commands.entity(part).insert(MeshMaterial3d(green));
            painted = true;
        }
        // Not until its model is there: it comes a frame or two after.
        if painted {
            commands.entity(zombie).insert(Painted);
        }
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
    let caption = match winner {
        Some(team) => fill(&words.team_wins, &[("team", &team_name(team))]),
        None => words.draw.clone(),
    };
    let caption = Some(caption);
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
    zombies: Query<(), With<Zombie>>,
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
        // The server's zombies, to be turned green.
        if theirs.0 == rules::ZOMBIE_TEAM && !zombies.contains(body) {
            commands.entity(body).insert(Zombie);
        }
    }
}

/// Sends each zombie ([`Hunter`]) for whoever of another team is nearest
/// within its sight, while the game is on, offline: up to them, over
/// whatever walk it was on, pressing in on them once there, and a punch
/// whenever they are in its reach and it may throw one, which knocks them
/// back as yours does ([`Shove::knock`]) and lands ([`Landed`]) for the
/// damage to be dealt. It sees no further than the file's
/// `arena.hunt_radius`, the dead not at all, and never a teammate; walking
/// off its ground it is kept from, as ever (`StaysAshore`), but onto a
/// bridge it will go. Until 2026-11-12 this was every bot's mind, which
/// with less than the file's `arena.flee_below` of its health ran away
/// instead ([`flight`]); the computer's players have a brain of their own
/// since ([`drive_bots`]), and are among those a zombie goes for.
#[allow(clippy::too_many_arguments)]
fn hunt(
    time: Res<Time>,
    game: Option<ResMut<Game>>,
    islands: Res<Islands>,
    mut bodies: Query<(
        Entity,
        &Transform,
        &Venue,
        // The computer's players have none, being placed, not walked.
        Option<&mut Intent>,
        &mut Shove,
        &mut PlayerJump,
        Option<&mut Hunter>,
        Option<&mut Steering>,
        Option<&Health>,
        Option<&Team>,
        Has<Dead>,
    )>,
    (wears, seats): (Query<&Wears>, Query<&Seat>),
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
    // Everyone a bot might go for: the living, but for whoever is unseen.
    let alive: Vec<(Entity, Vec3, Option<Team>)> = bodies
        .iter()
        .filter(|(_, _, venue, .., dead)| **venue == Venue::Arena && !dead)
        .filter(|(body, ..)| !wears.get(*body).is_ok_and(|wears| wears.unseen))
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
            let Ok((_, body, _, Some(mut intent), _, _, Some(mut hunter), steering, health, team, _)) =
                bodies.get_mut(bot)
            else {
                continue;
            };
            hunter.cooldown = (hunter.cooldown - dt).max(0.0);
            let here = body.translation;
            let mine = team.copied();
            // Blinded by a trap, it sees no further than anyone blinded.
            let blind = seats
                .get(bot)
                .is_ok_and(|seat| game.as_deref().is_some_and(|game| game.outfits.kit(seat.0).blind));
            let sight = if blind { rules::BLIND_SIGHT } else { tuning.hunt_radius };
            let Some((target, there, _)) = alive
                .iter()
                .filter(|&&(other, at, theirs)| {
                    other != bot
                        && !(mine.is_some() && theirs == mine)
                        && at.xz().distance(here.xz()) <= sight
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
            // A zombie comes on slower than anyone runs, and never runs
            // away.
            let zombie = mine == Some(Team(rules::ZOMBIE_TEAM));
            // Nearly beaten, it runs for it rather than fights.
            if !zombie && health.is_some_and(|health| health.share() < tuning.flee_below) {
                intent.walk = islands.get(Venue::Arena).map_or(-toward, |map| flight(map, here, toward));
                continue;
            }
            let pace = if zombie { (rules::ZOMBIE_SPEED / tuning.run_speed).min(1.0) } else { 1.0 };
            intent.walk = toward * if off.length() > melee.reach { pace } else { PRESS_IN };
            let facing = body.forward().xz().normalize_or(Vec2::NEG_Y);
            let in_reach = (there.y - here.y).abs() <= PLAYER_HEIGHT && sector.holds(facing, off, PLAYER_RADIUS);
            if !in_reach || hunter.cooldown > 0.0 {
                continue;
            }
            hunter.cooldown = tuning.bot_punch_every;
            // A zombie's punch knocks as one thrown standing does: it
            // comes on at a walk.
            let running = !zombie && intent.walk.length() >= 0.5;
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

/// How a body turned by `rotation` faces, as a turn about the upright in
/// radians: -Z turned by it is ahead, as a plank heads.
fn yaw_of(rotation: Quat) -> f32 {
    let forward = rotation * Vec3::NEG_Z;
    (-forward.x).atan2(-forward.z)
}

/// The computer's players in a game played offline, a frame of each
/// (since 2026-11-12; `roundtown_net::bots`, the brain the server plays
/// its own by online, with the same numbers): each thinks and moves, its
/// body is put where the brain has it, and what comes of it is carried
/// out here as the server carries it out there (`Game::play_bot`): a
/// punch knocks whoever it lands on and lands ([`Landed`]) for the damage
/// to be dealt, an anchor is struck, a purchase made out of what the bot
/// holds, the rungs of its bridge laid (`bridge::Bridges::lay_remote`), a
/// fireball thrown, a potion drunk. A knock that has landed on its body,
/// from a punch or a blast (`Shove::knock`), is handed on to the brain,
/// which carries it by it, off an edge if it must. Every bridge built is
/// told to the brains' map as it is fixed, yours among them. And each
/// takes what its next purchases need of what lies within its reach, you
/// having had yours already this frame ([`gather`]), and loses it all as
/// it dies.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn drive_bots(
    time: Res<Time>,
    game: Option<ResMut<Game>>,
    mut bodies: Query<
        (
            Entity,
            &mut Transform,
            &Venue,
            &Seat,
            Option<&mut Health>,
            &mut Shove,
            &mut PlayerJump,
            Has<Dead>,
            Has<Brained>,
            Has<Carrying>,
        ),
        With<WalkCycle>,
    >,
    things: Query<(Entity, &loot::Thing)>,
    mut bridges: ResMut<bridge::Bridges>,
    look: Res<fire::Look>,
    mut landed: MessageWriter<Landed>,
    mut commands: Commands,
) {
    if !offline_fight(&game) {
        return;
    }
    let mut game = game.expect("checked");
    let game = &mut *game;
    let Some(brains) = game.brains.as_deref_mut() else {
        return;
    };
    let Brains {
        nav,
        squad,
        purses,
        dead,
        heard,
        rungs,
    } = brains;
    let (dt, elapsed, me) = (time.delta_secs(), game.elapsed, game.me);
    let file = ui();
    // Every bridge as it stands, whoever built it: a rung heard already
    // is nothing new.
    if *heard != Some(bridges.version) {
        *heard = Some(bridges.version);
        for (seat, chain, index, at) in bridges.laid() {
            nav.hear(seat, chain, index, at);
        }
    }
    // Everyone, by seat, as the brains see them.
    let seats = game.seats.len();
    let mut seen = vec![Seen::default(); seats];
    let mut body_of: Vec<Option<Entity>> = vec![None; seats];
    for (body, at, venue, seat, health, _, _, fallen, ..) in &bodies {
        if *venue != Venue::Arena || seat.0 >= seats {
            continue;
        }
        body_of[seat.0] = Some(body);
        seen[seat.0] = Seen {
            at: at.translation,
            alive: !fallen,
            in_play: seat.0 >= ARENA_SEATS || !game.out[seat.0],
            unseen: game.outfits.kit(seat.0).unseen(elapsed),
            person: seat.0 == me,
            health: health.map_or(1.0, Health::share),
        };
    }
    let computers = move |seat: &usize| *seat != me;
    // Whoever of them has died since the last frame has lost what it
    // held, and whoever stands again begins again from where it stands.
    for seat in (0..seats.min(ARENA_SEATS)).filter(computers) {
        let fallen = !seen[seat].alive;
        if fallen && !dead[seat] {
            squad.died(seat, elapsed);
            purses[seat] = Purse::default();
        } else if !fallen && dead[seat] {
            let turn = body_of[seat]
                .and_then(|body| bodies.get(body).ok())
                .map_or(0.0, |(_, at, ..)| yaw_of(at.rotation));
            squad.stand(seat, seen[seat].at, turn, elapsed);
        }
        dead[seat] = fallen;
    }
    for seat in (0..seats.min(ARENA_SEATS)).filter(computers) {
        let Some(entity) = body_of[seat].filter(|_| seen[seat].alive) else {
            continue;
        };
        // What the bot does this frame, with its own body in hand; its
        // punch is thrown once that is let go of.
        let punch = {
            let Ok((_, mut transform, _, _, health, mut shove, mut jump, _, brained, carrying)) = bodies.get_mut(entity)
            else {
                continue;
            };
            if !brained {
                continue;
            }
            // A knock that has landed on the body is the brain's to carry
            // it by; and it reels from it no longer than anyone.
            if shove.across != Vec2::ZERO || jump.velocity_y != 0.0 {
                squad.shoved(seat, shove.across, jump.velocity_y.max(0.0));
                shove.across = Vec2::ZERO;
                jump.velocity_y = 0.0;
            }
            if shove.hurt > 0.0 {
                shove.hurt = (shove.hurt - dt).max(0.0);
            }
            let order = {
                let view = bots::View {
                    elapsed,
                    nav,
                    seats: &seen,
                    purses: purses.as_slice(),
                    outfits: &game.outfits,
                    mint: &game.mint,
                    anchors: game.anchors,
                };
                squad.step(seat, &view, dt)
            };
            let Some(bot) = squad.bot(seat) else {
                continue;
            };
            let (at, turn) = (bot.body.at, Quat::from_rotation_y(bot.body.yaw));
            if transform.translation != at {
                transform.translation = at;
            }
            if transform.rotation != turn {
                transform.rotation = turn;
            }
            seen[seat].at = at;
            let here = at.xz();
            if let Some(team) = order.strike.map(usize::from)
                && team < ARENA_TEAMS
                && team != usize::from(arena_team(seat))
                && game.anchors[team] > 0.0
                && game.anchor_hurt[team] <= 0.0
                && rules::anchor_in_reach(here, ARENA_ANCHORS[team], rules::REACH)
            {
                // A sword breaks an anchor sooner, as it does a body.
                let damage = rules::punch_damage(file.arena.damage, &game.outfits.kit(seat), &game.outfits.team(seat));
                game.anchor_hurt[team] = file.melee.hurt_time;
                game.outfits.reveal(seat);
                game.anchors[team] = (game.anchors[team] - damage).max(0.0);
                let anchor = ARENA_ANCHORS[team];
                spawn_number(&mut commands, Vec3::new(anchor.x, 1.2, anchor.y), damage);
                commands.entity(entity).insert(Punching::next());
                if game.anchors[team] <= 0.0 {
                    info!("team {team}'s anchor is broken, by seat {seat}");
                }
            }
            if let Some(ware) = order.buy
                && rules::at_a_shop(here, rules::sold_at(ware))
                && game.outfits.buy(seat, ware, &mut purses[seat])
            {
                squad.bought(seat, ware, elapsed);
                debug!("seat {seat} bought {ware:?}");
            }
            for (chain, index, rung, heading) in order.rungs {
                if rungs[seat] >= MOST_BOT_RUNGS {
                    break;
                }
                rungs[seat] += 1;
                nav.hear(seat as u8, chain, index, rung);
                bridges.lay_remote(
                    seat as u8,
                    chain,
                    index,
                    bridge::Rung {
                        p: rung,
                        d: bridge::heading_of(heading),
                    },
                );
            }
            if let Some(heading) = order.throw
                && game.outfits.spend(seat, Utility::Fireball)
            {
                fire::throw(&mut commands, &look, Flight::thrown(at, heading), seat, None);
            }
            if let Some(potion) = order.drink
                && game.outfits.drink(seat, potion, elapsed) == Some(Utility::Healing)
                && let Some(mut health) = health
            {
                let back = health.full * rules::HEALING;
                if health.mend(back) {
                    spawn_healed(&mut commands, at, back);
                }
            }
            // Seen laying its bridge: a plank in its arms (`bridge`).
            if order.carrying != carrying {
                if order.carrying {
                    commands.entity(entity).insert(Carrying);
                } else {
                    commands.entity(entity).remove::<Carrying>();
                }
            }
            order.punch.map(|victim| (victim, here, order.running))
        };
        let Some((victim, from, running)) = punch else {
            continue;
        };
        let Some(target) = body_of.get(victim).copied().flatten().filter(|_| seen[victim].alive) else {
            continue;
        };
        if arena_team(victim) == arena_team(seat) {
            continue;
        }
        commands.entity(entity).insert(Punching::next());
        if let Ok((_, there, _, _, _, mut shove, mut jump, ..)) = bodies.get_mut(target) {
            let at = there.translation.xz();
            if at.distance(from) <= rules::REACH + PLAYER_RADIUS + 0.5 && shove.knock(&mut jump, from, at, running) {
                landed.write(Landed {
                    by: entity,
                    victim: target,
                });
            }
        }
    }
    // What lies within their reach: only what each one's next purchases
    // need, whoever it matters most to first.
    let mut takers: Vec<(u8, usize)> = (0..seats.min(ARENA_SEATS))
        .filter(computers)
        .filter(|&seat| seen[seat].alive)
        .map(|seat| (squad.wants(seat).1, seat))
        .collect();
    takers.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for (_, seat) in takers {
        let (mut wants, _) = squad.wants(seat);
        if wants == [0; 4] {
            continue;
        }
        let taken = game.mint.pick_up_where(seen[seat].at, elapsed, &mut |piece| {
            let wanted = &mut wants[piece.item.index()];
            let take = *wanted > 0;
            *wanted = wanted.saturating_sub(1);
            take
        });
        for piece in taken {
            let held = &mut purses[seat][piece.item.index()];
            *held = held.saturating_add(1);
            if let Some((thing, _)) = things.iter().find(|(_, thing)| thing.key == loot::Key::Own(piece.id)) {
                loot::take(&mut commands, thing, body_of[seat]);
            }
        }
    }
}

/// How long between two sayings that your anchor is under attack, in
/// seconds.
const ANCHOR_WARN_EVERY: f32 = 8.0;

/// Says that your team's anchor is under attack, in the line the
/// timetable's are said in (since 2026-11-12, the computer's players
/// going for anchors now): whenever it has lost health and still stands,
/// no oftener than every [`ANCHOR_WARN_EVERY`] seconds. Online the health
/// is the server's ([`follow_round`]), offline this device's. Broken, it
/// has a headline of its own ([`break_anchors`]), and so has the
/// timetable's breaking of every anchor.
fn warn_of_the_anchor(
    game: Option<ResMut<Game>>,
    alerts: Query<Entity, With<Alert>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let Some(mut game) = game else {
        return;
    };
    let left = game.anchors[usize::from(game.my_team())];
    let was = game.anchor_was;
    if left == was {
        return;
    }
    game.anchor_was = left;
    if !anchor_warned(game.playing(), was, left, game.elapsed - game.anchor_said) {
        return;
    }
    game.anchor_said = game.elapsed;
    for old in &alerts {
        commands.entity(old).try_despawn();
    }
    alert(&mut commands, &mut images, &Line::plain(&ui().words.arena.anchor_attacked));
}

/// Whether to say that your anchor is under attack: the game `playing`,
/// the anchor gone from `was` down to `left` and still standing, and
/// `since` seconds gone since it was last said.
fn anchor_warned(playing: bool, was: f32, left: f32, since: f32) -> bool {
    playing && left < was && left > 0.0 && since >= ANCHOR_WARN_EVERY
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
    mut victims: Query<(&Transform, &mut Health, &Seat, &mut Shove, &mut PlayerJump, Has<Zombie>), Without<Dead>>,
    seats: Query<&Seat>,
    mut commands: Commands,
) {
    if !offline_fight(&game) {
        hits.clear();
        return;
    }
    let mut game = game.expect("checked");
    for hit in hits.read() {
        let Ok((at, mut health, seat, mut shove, mut jump, zombie)) = victims.get_mut(hit.victim) else {
            continue;
        };
        // A zombie takes a share of the knock that has just landed on it
        // (`Shove::knock`): it keeps coming (`rules::ZOMBIE_KNOCK`).
        if zombie {
            shove.across *= rules::ZOMBIE_KNOCK;
            jump.velocity_y *= rules::ZOMBIE_KNOCK;
        }
        // More with a sword in the hitter's hand, less through the
        // victim's shield (`Outfits::punch`); a zombie, in a seat past the
        // players', has neither.
        let hitter = seats.get(hit.by).map_or(usize::MAX, |seat| seat.0);
        let damage = game.outfits.punch(ui().arena.damage, hitter, seat.0);
        // Whoever of the two was unseen is seen again ("being hit or
        // attacking reveals u").
        game.outfits.reveal(hitter);
        game.outfits.reveal(seat.0);
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
    // A sword breaks an anchor sooner, as it does a body.
    let damage = rules::punch_damage(file.arena.damage, &game.outfits.kit(game.me), &game.outfits.team(game.me));
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
                // Hitting an anchor is attacking: unseen no longer.
                let me = game.me;
                game.outfits.reveal(me);
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
    islands: Res<Islands>,
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
        // A zombie rises where it rose; anyone else where their seat
        // arrives.
        match seat.checked_sub(ARENA_SEATS) {
            Some(zombie) => {
                let place = zombie_place(islands.get(Venue::Arena), zombie);
                lobby::put(&mut bodies, &mut orbit, body, Venue::Arena, place);
            }
            None => {
                lobby::send(&mut bodies, &mut orbit, body, Venue::Arena, seat);
                info!("{body} stands again on island {}", roundtown_net::arena_island(seat));
            }
        }
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
    // More than one at once is the timetable's doing, every anchor left
    // broken at ten minutes, which its own line says ([`timetable`]): a
    // headline for each would be four on top of each other.
    let breaking = (0..ARENA_TEAMS)
        .filter(|&team| game.anchors[team] <= 0.0 && !anchors.broken[team])
        .count();
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
        if breaking > 1 {
            continue;
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
/// the pillar stands on its point on the slab's top, 0.75 m up, is widest
/// 2.25 m up, a metre out at its corners, and ends in a point at 3.75 (a
/// cube on its corner from 0.54 to 3.96 until the map of 2026-11-11), and
/// every one of its triangles has a corner at 2.25 or over.
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

/// "Team Green": `team` as it is written on the screen, by its colour
/// (since 2026-11-12, Hajun: "team should be red, blue, green, yellow";
/// "Team 3", counted from 1, until then). Red is the island at the bottom
/// of the map, blue the one on its left, green the one at its top and
/// yellow the one on its right, by seat (`arena_team`).
fn team_name(team: u8) -> String {
    let file = ui();
    let words = &file.words.arena;
    let colour = [&words.team_red, &words.team_blue, &words.team_green, &words.team_yellow]
        .get(usize::from(team))
        .map_or("?", |word| word.as_str());
    fill(&words.team, &[("team", colour)])
}

/// "You are on Team Red": said under "Fight!" as the game begins, the
/// names over heads being in their teams' colours now.
fn your_team(team: u8) -> String {
    fill(&ui().words.arena.your_team, &[("team", &team_name(team))])
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
/// health and a team: made as they come, the name in their team's colour
/// and the bar a teammate's or an enemy's, kept over their heads as the
/// camera moves, the bar filled as far as their health goes, and put away
/// while they are dead or off the screen, or you are not in the game.
/// Small and faint, so as not to hide the fight.
///
/// **The bar's fill is the bar's child, not the tag's**: from 2026-11-08,
/// when the name went over the bar and the two were put in a column,
/// until 2026-11-12, this looked for the fill among the tag's own
/// children, found none, and every bar stayed full whatever its body's
/// health (Hajun: "i think i noticed the healthbar doesnt decrease
/// visually"). The numbers floating off a body and the deaths were right
/// all along.
#[allow(clippy::too_many_arguments)]
fn place_tags(
    me: Query<(&Venue, Option<&Team>), With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    bodies: Query<(Entity, &Transform, &Health, &Team, &Name, Has<Dead>), Without<Player>>,
    wears: Query<&Wears>,
    mut tags: Query<(Entity, &TagOf, &mut Visibility, &mut UiTransform, &ComputedNode, &Children)>,
    parts: Query<&Children>,
    mut fills: Query<&mut Node, With<TagFill>>,
    mut commands: Commands,
) {
    let (here, my_team) = me.single().map(|(venue, team)| (Some(*venue), team.copied())).unwrap_or((None, None));
    let in_arena = here == Some(Venue::Arena);
    if in_arena {
        for (body, _, _, team, name, _) in &bodies {
            if !tags.iter().any(|(_, of, ..)| of.0 == body) {
                spawn_tag(&mut commands, body, name.as_str(), team.0, Some(*team) == my_team);
            }
        }
    }
    let (Ok((camera, eye)), Ok(window)) = (cameras.single(), windows.single()) else {
        return;
    };
    let eye = GlobalTransform::from(*eye);
    for (tag, of, mut shown, mut place, node, children) in &mut tags {
        let Ok((_, body, health, team, _, dead)) = bodies.get(of.0) else {
            commands.entity(tag).despawn();
            continue;
        };
        // Nor over anyone unseen, who is not drawn either: but one of your
        // own team, whom you see, see-through (`gear`).
        let unseen = wears.get(of.0).is_ok_and(|wears| wears.unseen) && Some(*team) != my_team;
        let point = (in_arena && !dead && !unseen)
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
        for fill in fills_of(children, &parts) {
            if let Ok(mut fill) = fills.get_mut(fill)
                && fill.width != width
            {
                fill.width = width;
            }
        }
    }
}

/// Everything a tag is made of that might be its bar's fill: its own
/// parts, and theirs, the fill being inside the bar, which is inside the
/// tag ([`spawn_tag`]).
fn fills_of(tag: &Children, parts: &Query<&Children>) -> Vec<Entity> {
    tag.iter()
        .flat_map(|part| std::iter::once(part).chain(parts.get(part).into_iter().flat_map(|inner| inner.iter())))
        .collect()
}

/// Whether `point`, in the window's logical pixels, is on the screen.
fn on_screen(window: &Window, point: Vec2) -> bool {
    point.x >= 0.0 && point.y >= 0.0 && point.x <= window.width() && point.y <= window.height()
}

/// The name and the bar over `body`'s head: the name in its `team`'s
/// colour (red, blue, green or yellow, since 2026-11-12; a teammate's
/// green or an enemy's red until then), and the bar in a teammate's
/// colour if `ally`, or an enemy's.
fn spawn_tag(commands: &mut Commands, body: Entity, name: &str, team: u8, ally: bool) {
    let colours = ui().colours;
    let arena = colours.arena;
    let ink = arena.team(team);
    let hp = if ally { arena.hp_ally } else { arena.hp_enemy };
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

/// A number floating up off the body at `at`, for health given back to
/// it: "+5", in the heal pool's colour.
fn spawn_healed(commands: &mut Commands, at: Vec3, health: f32) {
    commands.spawn((
        InArena,
        DamageNumber {
            at,
            age: 0.0,
            healed: true,
        },
        hud::words(format!("+{health:.0}"), NUMBER_TEXT, ui().colours.arena.heal_pool),
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

/// A number floating up off the body at `at`, for `damage` taken.
fn spawn_number(commands: &mut Commands, at: Vec3, damage: f32) {
    commands.spawn((
        InArena,
        DamageNumber {
            at,
            age: 0.0,
            healed: false,
        },
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
    let (hurt, healed, shadow) = {
        let colours = ui().colours;
        (colours.arena.damage, colours.arena.heal_pool, colours.text_shadow)
    };
    let dt = time.delta_secs();
    for (number, mut floating, mut shown, mut place, node, mut colour, mut under) in &mut numbers {
        let ink = if floating.healed { healed } else { hurt };
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
        // Not the seats the server keeps empty between two that are
        // taken, which are nobody's.
        &[
            ("count", &game.seats.iter().filter(|member| member.id != 0).count().to_string()),
            ("seats", &ARENA_SEATS.to_string()),
        ],
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
        // "make the spawnpoint in the center of the island"): under each of
        // the eight is the island's own top, 0.25 m up, and nothing that
        // stands on the island reaches into a body standing there, not the
        // market stands either side of the middle, the anchor's slab nor
        // the pad at the back (Hajun's map of 2026-11-11; until then a desk
        // lay across the middle, and whoever arrived was shoved off it);
        // and the void is beside the islands, where the bridges were until
        // 2026-11-08.
        let map = hajuns_map();
        for seat in 0..ARENA_SEATS {
            let at = island::arrival(Venue::Arena, seat).translation;
            let ground = map.floor(at.xz(), 3.0).unwrap_or_else(|| panic!("seat {seat}: nothing under {at}"));
            assert!((ground - island::LAND_TOP_OF_THE_ISLANDS).abs() < 0.03, "seat {seat}: ground at {ground}");
            let body = [(PLAYER_RADIUS, ground + crate::LAND_STEP_UP, ground + PLAYER_HEIGHT)];
            let walls = map.walls_round(at.xz(), &body);
            assert!(walls.is_empty(), "seat {seat}: {} squares of something in the way at {at}", walls.len());
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
            elapsed: 0.0,
            said: 0.0,
            mint: Mint::new(0.0, &mut || 0.5),
            outfits: Outfits::default(),
            brains: None,
            anchor_was: 120.0,
            anchor_said: f32::NEG_INFINITY,
            compass: false,
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
        assert_eq!(team_name(2), "Team Green");
        assert_eq!(team_name(0), "Team Red");
        assert_eq!(your_team(3), "You are on Team Yellow");
        // A zombie, in a seat past the players', is always back, whatever
        // the anchors, and is nobody's team to be left in play.
        game.anchors = [0.0; ARENA_TEAMS];
        for zombie in 0..rules::ZOMBIES {
            let seat = ARENA_SEATS + zombie;
            assert_eq!(arena_team(seat), rules::ZOMBIE_TEAM);
            assert_eq!(game.die(seat).left, rules::ZOMBIE_RESPAWN_SECS);
        }
        assert_eq!(game.teams_in_play(), vec![0]);
        assert_eq!(game.out, [false, true, true, true, false, true, true, true]);
    }

    #[test]
    fn the_generators_pads_are_where_the_map_has_them() {
        use roundtown_net::{AMETHYST_PADS, ARENA_PADS, PAD_HALF, SAPPHIRE_PADS};
        // The pads by what Hajun calls them in Blender: `gen1` to `gen4`
        // on the teams' islands, `sapphiregen` to `sapphiregen4` on the
        // smaller ones, `amethystgen` to `.003` in the middle; each the
        // box round its mesh, across the ground and up.
        let parts = crate::island::glb::read("pvptownmap.glb");
        let pads = |called: &str| -> Vec<(Vec3, Vec3)> {
            parts
                .iter()
                .filter(|part| part.node.starts_with(called))
                .map(|part| {
                    part.triangles
                        .iter()
                        .flatten()
                        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), corner| (low.min(*corner), high.max(*corner)))
                })
                .collect()
        };
        let map = hajuns_map();
        for (called, wanted) in [("gen", &ARENA_PADS), ("sapphiregen", &SAPPHIRE_PADS), ("amethystgen", &AMETHYST_PADS)] {
            let found = pads(called);
            assert_eq!(found.len(), wanted.len(), "{called}: {found:?}");
            for pad in wanted {
                // Where the server takes each to be is the middle of one,
                // a slab 2 m square with its top where things are laid.
                let (low, high) = found
                    .iter()
                    .find(|(low, high)| ((*low + *high) * 0.5).xz().distance(*pad) < 0.05)
                    .unwrap_or_else(|| panic!("{called}: no pad at {pad}, of {found:?}"));
                assert!((high.y - PAD_TOP).abs() < 1e-3, "{called} at {pad}: its top is {} up", high.y);
                let half = (*high - *low).xz() * 0.5;
                assert!((half.x - PAD_HALF).abs() < 0.01 && (half.y - PAD_HALF).abs() < 0.01, "{called}: {half}");
                // Wherever on it a thing may come up, the pad's top is
                // what is under it, and nothing stands over it to hide it
                // or keep anyone off it.
                for (dx, dz) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (0.0, 0.0)] {
                    let at = *pad + Vec2::new(dx, dz) * rules::PAD_SPREAD;
                    let ground = map.floor(at, 3.0).unwrap_or_else(|| panic!("{called}: nothing under {at}"));
                    assert!((ground - PAD_TOP).abs() < 1e-3, "{called}: {ground} up at {at}");
                }
                let body = [(PLAYER_RADIUS, PAD_TOP + crate::LAND_STEP_UP, PAD_TOP + PLAYER_HEIGHT)];
                assert!(map.walls_round(*pad, &body).is_empty(), "{called} at {pad}: something stands on it");
            }
        }
        // And the pad is no higher over the island than a step: walked
        // onto, not jumped onto.
        assert!(PAD_TOP - island::LAND_TOP_OF_THE_ISLANDS < crate::LAND_STEP_UP);
        // Every generator is on one of them.
        for index in 0..rules::GENERATORS {
            let (_, pad) = rules::generator(index);
            assert!(ARENA_PADS.contains(&pad) || SAPPHIRE_PADS.contains(&pad) || AMETHYST_PADS.contains(&pad));
        }
    }

    #[test]
    fn the_zombies_rise_on_open_ground_in_the_map() {
        // Each on ground, about the islands' top or the town's paving,
        // with nothing reaching into a body standing there, and inside
        // the ground the server keeps it to.
        let map = hajuns_map();
        for zombie in 0..rules::ZOMBIES {
            let place = zombie_place(Some(&map), zombie).translation;
            let at = rules::zombie_spot(zombie);
            assert_eq!(place.xz(), at);
            let ground = map.floor(at, 3.0).unwrap_or_else(|| panic!("zombie {zombie}: nothing under {at}"));
            assert_eq!(place.y, ground);
            assert!((0.0..=0.3).contains(&ground), "zombie {zombie}: ground at {ground}");
            let body = [(PLAYER_RADIUS, ground + crate::LAND_STEP_UP, ground + PLAYER_HEIGHT)];
            assert!(map.walls_round(at, &body).is_empty(), "zombie {zombie} rises inside something at {at}");
            // Dry land all round it, a few steps each way.
            for step in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
                assert!(map.floor(at + step * 2.0, 3.0).is_some(), "zombie {zombie}: no ground beside {at}");
            }
        }
    }

    #[test]
    fn a_line_about_an_item_has_its_picture_after_its_name() {
        assert_eq!(roman(1), "I");
        assert_eq!(roman(2), "II");
        assert_eq!(roman(3), "III");
        // The file's line, as it is said at 2:00: the name, the picture,
        // and the rest.
        let file = ui();
        let words = &file.words.arena;
        let line = Line::of_tier(&words.tier_up, Item::Sapphire, words.item(Item::Sapphire), roman(2));
        assert_eq!(
            line,
            Line {
                before: "Sapphire".into(),
                picture: Some(Item::Sapphire),
                after: "generators have been upgraded to Lvl II.".into(),
            }
        );
        // Wherever the name is in the line, the picture is right after it;
        // and a line that does not name the item has none.
        let line = Line::of_tier("Lvl {level} for the {item} generators!", Item::Amethyst, "Amethyst", "III");
        assert_eq!(line.before, "Lvl III for the Amethyst");
        assert_eq!(line.picture, Some(Item::Amethyst));
        assert_eq!(line.after, "generators!");
        let line = Line::of_tier("Faster generators: Lvl {level}.", Item::Amethyst, "Amethyst", "II");
        assert_eq!(line, Line::plain("Faster generators: Lvl II."));
        assert_eq!(Line::plain(&words.anchors_broken).picture, None);
    }

    #[test]
    fn a_generator_s_level_and_the_count_to_its_next_thing_are_written_by_the_time() {
        let file = ui();
        let words = &file.words.arena;
        // As the fight begins: the first sapphire is 30 s off and the
        // first amethyst 60.
        assert_eq!(generator_line(words, Item::Amethyst, false, 2), "Amethyst Generator Lvl 2");
        assert_eq!(generator_line(words, Item::Amethyst, true, 24), "Next Amethyst in 24sec...");
        assert_eq!(seconds_to_next(Item::Sapphire, 0.0), 30);
        assert_eq!(seconds_to_next(Item::Amethyst, 0.0), 60);
        // The last second reads 1, and the moment one comes the count is
        // the whole wait to the next: never 0.
        assert_eq!(seconds_to_next(Item::Sapphire, 29.2), 1);
        assert_eq!(seconds_to_next(Item::Sapphire, 30.0), 30);
        assert_eq!(seconds_to_next(Item::Amethyst, 36.0), 24);
        // Online the time comes in whole seconds, and at every one of
        // them the count is a second less than the second before, or a
        // whole wait again; the waits are the tiers', 23 s for a sapphire
        // from two minutes and 12 from six, 40 for an amethyst from four
        // and 25 from eight.
        for item in [Item::Sapphire, Item::Amethyst] {
            let mut last = seconds_to_next(item, 0.0);
            for second in 1..(PLAY_SECS as u32) {
                let now = seconds_to_next(item, second as f32);
                assert!(now >= 1 && (now == last - 1 || last == 1), "{item:?}: {now} after {last}, {second} s in");
                last = now;
            }
        }
        assert_eq!(seconds_to_next(Item::Sapphire, 120.0), 23);
        assert_eq!(seconds_to_next(Item::Sapphire, 372.0), 12);
        assert_eq!(seconds_to_next(Item::Amethyst, 240.0), 40);
        assert_eq!(seconds_to_next(Item::Amethyst, 480.0), 25);
        assert_eq!(seconds_to_next(Item::Amethyst, 505.0), 25);
        // Only the sapphire and the amethyst generators are written over:
        // eight of the sixteen.
        let written = (0..rules::GENERATORS).filter(|index| rules::steady(rules::generator(*index).0)).count();
        assert_eq!(written, 8);
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
    fn dying_loses_the_metres_of_bridge_and_offline_everything_held() {
        let stock = Resources {
            bridge_metres: 62.5,
            held: [31, 4, 2, 1],
        };
        // Offline all of it is this device's to take.
        let lost = lost_on_death(&stock, true);
        assert_eq!((lost.bridge_metres, lost.held), (0.0, [0; 4]));
        assert_eq!((lost.metres(), lost.count(Item::Silver), lost.count(Item::Amethyst)), ("0m".into(), "0".into(), "0".into()));
        // Online the metres are, and what is held is the server's, which
        // empties it as it has you dead.
        let lost = lost_on_death(&stock, false);
        assert_eq!((lost.bridge_metres, lost.held), (0.0, [31, 4, 2, 1]));
        // With nothing, nothing changes.
        assert_eq!(lost_on_death(&Resources::default(), true), Resources::default());
    }

    #[test]
    fn the_clock_rounds_up() {
        assert_eq!(clock(600.0), "10:00");
        assert_eq!(clock(599.2), "10:00");
        assert_eq!(clock(59.0), "0:59");
        assert_eq!(clock(0.4), "0:01");
        assert_eq!(clock(0.0), "0:00");
    }

    #[test]
    fn your_anchor_under_attack_is_said_when_it_is_hit_and_stands_and_not_too_often() {
        // Hit, and still standing, with nothing said lately.
        assert!(anchor_warned(true, 120.0, 108.0, f32::INFINITY));
        assert!(anchor_warned(true, 108.0, 86.0, ANCHOR_WARN_EVERY));
        // Not again within a few seconds of the last.
        assert!(!anchor_warned(true, 108.0, 96.0, ANCHOR_WARN_EVERY - 0.1));
        // Not as it breaks, which has a headline of its own, as the
        // timetable's breaking of every anchor has its line.
        assert!(!anchor_warned(true, 12.0, 0.0, f32::INFINITY));
        assert!(!anchor_warned(true, 120.0, 0.0, f32::INFINITY));
        // Nor when it has lost nothing, nor out of the fight.
        assert!(!anchor_warned(true, 96.0, 96.0, f32::INFINITY));
        assert!(!anchor_warned(true, 96.0, 120.0, f32::INFINITY));
        assert!(!anchor_warned(false, 120.0, 108.0, f32::INFINITY));
    }

    #[test]
    fn the_computer_s_players_of_a_game_played_offline_stand_where_their_map_can_be_walked() {
        // A turn about the upright is read back as a plank heads.
        for yaw in [0.0f32, 0.7, -1.2, 3.0] {
            assert!((yaw_of(Quat::from_rotation_y(yaw)) - yaw).abs() < 1e-4, "{yaw}");
        }
        // Whoever arrives on the island at +Z faces the middle of the map.
        assert!(yaw_of(island::arrival(Venue::Arena, 0).rotation).abs() < 1e-3);
        // Every arrival is somewhere a body walks on the map the brains
        // have (`roundtown_net::nav`, baked from the map itself), and the
        // brain stands a body put there, a quarter of a metre under the
        // island's top as the game's arrivals are, onto it.
        let mut brains = Brains::new();
        for seat in 0..ARENA_SEATS {
            let at = island::arrival(Venue::Arena, seat).translation;
            assert!(brains.nav.walkable_at(at.xz()), "seat {seat} at {at}");
        }
        let seen: Vec<Seen> = (0..ARENA_SEATS)
            .map(|seat| Seen {
                at: island::arrival(Venue::Arena, seat).translation,
                alive: true,
                in_play: true,
                unseen: false,
                person: false,
                health: 1.0,
            })
            .collect();
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        let mint = Mint::new(0.0, &mut || 0.5);
        // Whoever each of them is and whichever way it sets off, by dice
        // seeded here (a game's own are not, and for a night this test
        // went by those). With nothing to do a bot wanders its island
        // (since 2026-11-12), and the white outline in front of a stand is
        // 2 cm over the island's top: the second of a team, who arrives at
        // the edge of one, stood on it at 0.27 m now and then, where this
        // asked for the island's top to the last of a metre, and failed two
        // times in six (found by the session adding the item slots,
        // 2026-11-13).
        for seed in 0..16 {
            brains.squad = Squad::new(seed, bots::Skill::MODERATE);
            for seat in 0..ARENA_SEATS {
                let arrival = island::arrival(Venue::Arena, seat);
                brains.squad.seat(seat, arrival.translation, yaw_of(arrival.rotation));
            }
            for seat in 0..ARENA_SEATS {
                for _ in 0..3 {
                    let view = bots::View {
                        elapsed: 0.0,
                        nav: &brains.nav,
                        seats: &seen,
                        purses: brains.purses.as_slice(),
                        outfits: &outfits,
                        mint: &mint,
                        anchors: [120.0; ARENA_TEAMS],
                    };
                    brains.squad.step(seat, &view, 1.0 / 60.0);
                }
                let body = brains.squad.bot(seat).expect("seated").body;
                // On the ground, which is the island's top or the outline
                // on it: no longer a quarter of a metre under it.
                assert!(body.free(&brains.nav), "seed {seed}, seat {seat}: {body:?}");
                let over = body.at.y - island::LAND_TOP_OF_THE_ISLANDS;
                assert!((-1e-4..=0.03).contains(&over), "seed {seed}, seat {seat}: {body:?}");
            }
        }
    }
}
