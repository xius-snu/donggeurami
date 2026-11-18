//! Bridge Race, on this device (since 2026-11-18; Hajun: "Like King of the
//! Ladder but with bridge building. You start on two islands next to each
//! other, and there is a golden island at equidistance from each island
//! like the tip of a 80deg 80deg 20deg isosceles triangle. Every interval
//! you are given more bridge meters, and sometimes u get dynamite as well,
//! randomly. Your goal is to reach the golden island first compared to the
//! other team - both of u need to make it across cuz its 2v2. You can punch
//! each other off the map, and ur respawn is the center of ur own player
//! island ... (similar controls and cam with bridge wars). Map =
//! bridgeracemap"): two teams, red and blue, each on its own island over
//! the void, bridging to the golden island, the computer making up the side
//! people do not. Its rules are `roundtown_net::bridge_race`'s, which the
//! server goes by online and this device offline ([`offline`]); its table
//! is `table`'s, which fixes the camera over you as in Bridge Wars.
//!
//! **The fight's state is this game's while its table is played** (the
//! fight kit's way, `arena::Fighting`): Bridge Wars' bridges, sticks of
//! dynamite, buttons, names over heads and punches serve it on its own
//! venue. As its table is entered the fight is said to be here
//! ([`Fighting::venue`] Bridge Race's), with a number of its own
//! (`arena::Begun`), whose bridges are this game's; everyone is on their
//! seat's team ([`rules::team`]: a punch never lands on a teammate, and
//! names are written in the team's colour once the race is on); you have
//! [`rules::START_METRES`] of bridge ([`Resources`], which the plank button
//! counts and the bridges spend); and your sticks of dynamite are on your
//! buttons as the game gives them ([`Outfit`]). Once its table is gone it
//! is given back, "no game" written once.
//!
//! **As it is played**: each grant the round says has come adds what it
//! gives your team's players to your metres, "+3m" rising off you
//! ([`RaceRound::share_of`]); a stick come to your button says so across
//! the upper half, with its picture; a stick thrown at the button goes to
//! the server, or offline to this device's own game; you fall into the
//! void, and you are back on your island's middle a moment later ("Back on
//! your island in 2"); the chat says who fell and by whose doing, and
//! whose bridge a stick holed, names in their teams' colours; the upper
//! half says as a team's bridge reaches the golden island, and as a team
//! stands on it all together; and the end is said across the screen, "Red
//! wins!", before the table's results. Over the world ([`show`]): the time
//! left and each team's way across top centre, the ring round the golden
//! island filling in the colour of a team that holds it with "Hold! 2"
//! over it, an arrow at the edge of the screen toward the golden island
//! while it is off the screen, with how far, "Jump!" once it is a jump
//! away, and a ring round the plank button filling toward the next grant;
//! and top right the whole race from above ([`map`]).
//!
//! **The camera** is Bridge Wars' (`table`), fixed over you, but that it
//! does not follow you down into the void: falling, and dead, you look on
//! from over where you went off, at the land's height; and as the race is
//! won on the golden island it goes over the island, to the winners and
//! the full ring ([`watch_over`]; mine, since a play-test of 2026-11-18,
//! the same day, whose player was knocked off at the end and saw nothing
//! but the dark void, then "Blue wins!" over their own island).

mod map;
mod offline;
mod show;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_race::{self as rules, Feed, Finish, RaceRound, team};
use roundtown_net::{Ask as NetAsk, Kits, Member, Minigame, Tell, UTILITY_SLOTS, Upgrades, Use, Utility, Venue};

use crate::arena::{ArenaSystems, Begun, DeathWords, FightTime, Fighting, InArena, NoBar, OfflineArena, Outfit, Resources, Used};
use crate::chat;
use crate::hud;
use crate::table::{self, AtTable, ComputerSeat, TableView};
use crate::ui::{fill, ui};
use crate::{Dead, Player, Team};

/// Something the server says to this device alone in its game of Bridge
/// Race (`roundtown_net::bridge_race::Tell`), handed on by `net`.
#[derive(Message, Clone, Debug)]
pub(crate) struct Told(pub roundtown_net::bridge_race::Tell);

/// How often, at the most, this device tells the server the metres of
/// bridge it has left, in seconds (Bridge Wars' own): only for giving them
/// back to it should it come back to the game afresh.
const METRES_EVERY: f32 = 1.0;

/// How far into the race this device can be as it sees it begin and still
/// say "Go!", in seconds of play: further, it has come into a race already
/// under way (put back in it, its app having been closed and opened
/// again), and takes it as it stands, saying nothing again of what has
/// happened in it (mine: until 2026-11-18, the same day, it said "Go!" and
/// your team, "Dynamite!" for the sticks you held, and whose bridge was
/// across and who held the golden island, all over again).
const LATE: f32 = 3.0;

/// How long "Go!" is across the screen, and the race's end, in seconds:
/// the end's no longer than the table waits before its results come up
/// (`table`'s two seconds; it showed through the results' window at three,
/// seen in a play-test), and taken down as they do.
const GO_SECS: f32 = 2.5;
const END_SECS: f32 = 2.0;

/// How long after "Go!" a team of two is told to build one bridge
/// together, in seconds: once "Go!" has gone (mine: two people each
/// laying their own bridge go half as fast as the computer's pair laying
/// one, and nothing said so; worked out from the rules in a play-test).
const ONE_BRIDGE_AFTER: f32 = 3.0;

/// How far from the golden island's middle you have to be, in metres, for
/// a team holding it again to be said across the upper half: further than
/// the ring is seen from over you. Nearer, the ring and "Hold!" say it
/// (mine: every hold said, a scramble on the island put up seventeen of
/// them, seen in a play-test). The first hold of each team is said
/// wherever you are.
const HOLD_SAID_PAST: f32 = 24.0;

/// How far under the land's top the camera follows you down as you fall,
/// in metres, and how long it takes to come over the golden island as the
/// race is won on it, in seconds ([`watch_over`]; mine).
const FALL_SEEN: f32 = 0.3;
const TO_THE_WINNERS: f32 = 1.2;

/// A body this game has put on its team, with no bar under its name
/// ([`dress_bodies`]): everyone else in the race, the computer's offline
/// and the people the server sends online. What it put there is taken off
/// again as the race goes ([`give_back`]), wherever the body is by then
/// (one of your group goes on into the next game with you, the same body).
#[derive(Component)]
struct Dressed;

/// A game of Bridge Race on this device, from the table's word that you sit
/// at one to its end: its number among every fight's (`arena::Begun`), whose
/// bridges are its own; whether the server plays it; your seat; whether it
/// has begun here (the "Go!" said) and whether it is over (its end said);
/// how many of the round's grants your metres count; the round as last
/// seen, which what is drawn goes by; your sticks as last seen, to say one
/// that comes; what the server was last told of your metres, and how long
/// ago; whether the server has told you your metres, back in the race
/// afresh; whether each team's bridge was across, and each team holding the
/// golden island, as last said, whether each has held it yet, and the line
/// across the upper half that says a team holds it, to take down as its
/// hold is broken; how long since the last grant came, which the ring
/// round the plank button fills by; whether a team of two is yet to be
/// told to build one bridge, and how long since "Go!"; and offline, the
/// game itself.
struct Race {
    number: u32,
    online: bool,
    seat: usize,
    begun: bool,
    over: bool,
    counted: u16,
    round: Option<RaceRound>,
    sticks: u8,
    told: Option<(f32, u16)>,
    since_told: f32,
    told_back: bool,
    joined: [bool; rules::TEAMS],
    holding: [bool; rules::TEAMS],
    held_once: [bool; rules::TEAMS],
    hold_said: Option<(usize, Entity)>,
    since_grant: f32,
    one_bridge: bool,
    since_go: f32,
    played: Option<Box<offline::Played>>,
}

impl Race {
    fn new(number: u32, online: bool, seat: u8) -> Self {
        Self {
            number,
            online,
            seat: usize::from(seat),
            begun: false,
            over: false,
            counted: 0,
            round: None,
            sticks: 0,
            told: None,
            since_told: 0.0,
            told_back: false,
            joined: [false; rules::TEAMS],
            holding: [false; rules::TEAMS],
            held_once: [false; rules::TEAMS],
            hold_said: None,
            since_grant: 0.0,
            one_bridge: false,
            since_go: 0.0,
            played: None,
        }
    }

    /// Whether the race is on: begun here, and not over.
    fn on(&self) -> bool {
        self.begun && !self.over
    }

    /// Your team.
    fn team(&self) -> u8 {
        team(self.seat)
    }
}

/// The game of Bridge Race you are at, if you are at one.
#[derive(Resource, Default)]
struct Racing(Option<Race>);

/// The fight's state, which this game owns while its table is played: where
/// the fight is and who is in it, whether a game is played offline, your
/// metres of bridge, your kit, the fight's clock, and the count every
/// game's number comes from.
#[derive(SystemParam)]
struct Fight<'w> {
    fighting: ResMut<'w, Fighting>,
    offline: ResMut<'w, OfflineArena>,
    stock: ResMut<'w, Resources>,
    outfit: ResMut<'w, Outfit>,
    clock: ResMut<'w, FightTime>,
    begun: ResMut<'w, Begun>,
}

/// What a line said in the game goes on: the chat, the line across the
/// upper half (every one up, and the one put up this very frame, which the
/// world has yet to be told of), and the pictures it may have; and the
/// words rising over the world, taken down for the end with the lines.
#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
struct Say<'w, 's> {
    log: ResMut<'w, chat::Log>,
    images: ResMut<'w, Assets<Image>>,
    pictures: ResMut<'w, hud::Pictures>,
    notices: Query<'w, 's, Entity, Or<(With<show::Notice>, With<crate::arena::Alert>)>>,
    posted: Local<'s, Option<Entity>>,
    risings: Query<'w, 's, Entity, With<show::Rising>>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Told>()
        .init_resource::<Racing>()
        .add_systems(
            Update,
            // What the table and the server say, the game followed and what
            // you do with it, before you are moved and before the bridges
            // and the buttons go by the fight's state; in Bridge Wars' own
            // set, after the buttons that say what was used and before the
            // gear that dresses bodies.
            (enter, hear, follow, use_sticks, tell_metres, dress_bodies)
                .chain()
                .after(table::TableSystems)
                .in_set(ArenaSystems)
                .before(crate::read_player_input)
                .before(crate::island::show_where_i_am),
        )
        .add_systems(
            Update,
            (
                // The game itself, offline: the punch thrown this frame
                // landed, the computer's players moved before anyone else
                // is.
                offline::play.after(crate::start_punch).before(crate::move_bodies),
                // A fall into the void, once everyone has moved; and where
                // the camera looks on from, before it is placed.
                (reap, watch_over).chain().after(crate::move_bodies).before(crate::follow_camera),
                // What is drawn over the world, once the camera is placed.
                (
                    show::plate,
                    show::ring,
                    show::hold_words,
                    show::point,
                    show::jump_hint,
                    show::rise,
                    show::fade_notices,
                    show::fade_banners,
                    show::count_to_grant,
                    map::draw,
                    map::dots,
                    map::recolour,
                )
                    .after(crate::follow_camera),
            ),
        );
}

/// The buttons round the screen at a table of Bridge Race, Bridge Wars'
/// less what is not in the race (Hajun: "similar controls and cam with
/// bridge wars"): your name top left, with no health (nobody has any: the
/// void is the only way to die) and none of the four things the generators
/// make; top right the race's own minimap ([`map`], where Bridge Wars has
/// its own; since 2026-11-18, the same day: none until then); and round
/// the jump button the fist in its place, which punches, the jump button
/// beside it, the plank, which builds, with the ring round its rim that
/// fills toward the next grant of bridge ([`show::next_grant`]), the five
/// buttons for things to use (the dynamite), and the chat's keyboard, each
/// where Bridge Wars has it.
pub(crate) fn spawn_bar(commands: &mut Commands, _assets: &AssetServer, images: &mut Assets<Image>) {
    let (left, right) = hud::game_bar(commands);
    commands.spawn((ChildOf(left), hud::name_plate()));
    map::spawn(commands, right, images);
    let round = commands.spawn(hud::round_the_jump()).id();
    let arrow = images.add(hud::jump_arrow());
    commands.spawn((
        ChildOf(round),
        hud::punch_button(images.add(hud::fist()), arrow.clone(), crate::JUMP_SIZE_VMIN, hud::beside_the_jump(0.0, 0.0)),
    ));
    let (right, bottom, size) = hud::ARENA_JUMP;
    commands.spawn((ChildOf(round), hud::jump_button(arrow, size, hud::beside_the_jump(right, bottom))));
    for slot in 0..UTILITY_SLOTS {
        let (right, bottom) = hud::slot_place(slot);
        commands.spawn((ChildOf(round), hud::utility_slot(slot, hud::beside_the_jump(right, bottom))));
    }
    let (right, bottom) = hud::ARENA_BRIDGE;
    commands.spawn((ChildOf(round), hud::bridge_button(images.add(hud::plank()), hud::beside_the_jump(right, bottom))));
    // Over the plank button, on its rim, taking no press of its own.
    commands.spawn((ChildOf(round), show::next_grant(images, right, bottom)));
    commands.spawn((ChildOf(round), hud::chat_button(images)));
}

/// What its score is called among the results: the file's
/// (`words.bridge_race.score`).
pub(crate) fn score_word() -> String {
    ui().words.bridge_race.score.clone()
}

/// What the line under the results says of what the whole table made:
/// nothing, the race being a race between its two teams.
pub(crate) fn total_words(_total: u32) -> Option<String> {
    None
}

/// A team's name, in the file's words: red's island and blue's.
fn team_name(side: u8) -> String {
    let file = ui();
    let words = &file.words.bridge_race;
    if side == 0 { words.red.clone() } else { words.blue.clone() }
}

/// The name of whoever sits in `seat`, as the table has them.
fn name_of(view: &TableView, seat: usize) -> String {
    view.members.get(seat).map_or_else(String::new, |member| member.name.clone())
}

/// Says in the chat what the game says has happened ([`Feed`]), names in
/// their teams' colours ([`feed_words`]).
fn feed_line(log: &mut chat::Log, view: &TableView, said: Feed) {
    if let Some(line) = feed_words(view, said) {
        log.push(line);
    }
}

/// What the chat says of what the game says has happened ([`Feed`]), names
/// in their teams' colours: who fell into the void, by whose doing; whose
/// stick holed whose bridge; and whose went into the void with nothing
/// under it. Nothing of a stick that blew and took nothing.
fn feed_words(view: &TableView, said: Feed) -> Option<chat::Line> {
    let file = ui();
    let (words, colours) = (&file.words.bridge_race, file.colours);
    let ink = colours.chat.event;
    let of = |seat: u8| (name_of(view, usize::from(seat)), colours.arena.team(team(usize::from(seat))));
    let line = match said {
        Feed::Fell { seat, by: None } => {
            let (name, colour) = of(seat);
            chat::spans(&words.fell, ink, &[("name", &name, colour)])
        }
        Feed::Fell { seat, by: Some(by) } => {
            let ((name, colour), (killer, theirs)) = (of(seat), of(by));
            chat::spans(&words.knocked, ink, &[("killer", &killer, theirs), ("name", &name, colour)])
        }
        Feed::Blew { by, team: Some(side), .. } => {
            let (name, colour) = of(by);
            let holed = team_name(side);
            chat::spans(&words.blew, ink, &[("name", &name, colour), ("team", &holed, colours.arena.team(side))])
        }
        // A stick that took nothing is nothing to say.
        Feed::Blew { team: None, .. } => return None,
        Feed::Lost { by } => {
            let (name, colour) = of(by);
            chat::spans(&words.missed, ink, &[("name", &name, colour)])
        }
    };
    Some(line)
}

// ------------------------------------------------------------ in and out

/// What the table says of Bridge Race: that you sit at one of its tables
/// (`table::Entered`), which begins the game here afresh, the fight's state
/// this game's from now on, and its pieces over the world put up again (the
/// table has taken down everything of its own, `table::InTable`); the same
/// table told again (your seat moved before the race, or this device put
/// back into it) keeps what it has and takes the seat. And, sat at no table
/// of Bridge Race any more, the race given back ([`give_back`]): away from
/// the tables (`table::Gone`), or at another game's, taken on into it
/// straight from this one's results by your group's leader, which the table
/// says with that game's `Entered` and no `Gone` between (its way, S2's).
/// Until 2026-11-18, the same day, only a `Gone` gave it back, and taken
/// on into another game the race went on here under it, writing the fight
/// as its own every frame, its team on you and your death its own.
#[allow(clippy::too_many_arguments)]
fn enter(
    mut entered: MessageReader<table::Entered>,
    at_table: Res<AtTable>,
    mut racing: ResMut<Racing>,
    mut fight: Fight,
    dressed: Query<Entity, With<Dressed>>,
    fought: Query<Entity, With<InArena>>,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<hud::Pictures>,
    mut commands: Commands,
) {
    for at in entered.read() {
        if at.game != Minigame::BridgeRace {
            continue;
        }
        let same = at.same_table && racing.0.as_ref().is_some_and(|race| race.online == at.online);
        match racing.0.as_mut().filter(|_| same) {
            Some(race) => race.seat = usize::from(at.seat),
            None => {
                // Whatever another game left up goes with it.
                for thing in &fought {
                    commands.entity(thing).try_despawn();
                }
                racing.0 = Some(Race::new(fight.begun.next(), at.online, at.seat));
                fight.stock.set_if_neq(Resources {
                    bridge_metres: rules::START_METRES,
                    ..default()
                });
                fight.outfit.set_if_neq(Outfit::default());
                if fight.clock.0 != 0.0 {
                    fight.clock.0 = 0.0;
                }
            }
        }
        fight.offline.set_if_neq(OfflineArena(!at.online));
        show::spawn(&mut commands, &mut images, &mut pictures);
        info!("Bridge Race: in seat {}{}", at.seat, if same { ", the same table" } else { "" });
    }
    if at_table.of(Minigame::BridgeRace).is_none()
        && let Some(race) = racing.0.take()
    {
        give_back(&race, &mut fight, &fought, &dressed, &mut commands);
        info!("Bridge Race: the fight given back{}", if at_table.0.is_some() { ", on into another game" } else { "" });
    }
}

/// The fight given back as the race goes from this device, once: if it is
/// still the race's (another game taken on into may have taken it already,
/// in this very frame, the order of the two being nobody's: then it is
/// that game's, and left as it is), "no game" written (its number kept),
/// nothing played offline, no metres, no kit and no clock, and whatever the
/// fight put up taken down; and off every body the race put on its team
/// ([`Dressed`]), wherever it is now, the team and the bar's absence. Your
/// own body the table lets go of (`table::Gone`), or the game you are taken
/// into dresses as it will.
fn give_back(race: &Race, fight: &mut Fight, fought: &Query<Entity, With<InArena>>, dressed: &Query<Entity, With<Dressed>>, commands: &mut Commands) {
    let ours = fight.fighting.game == race.number || fight.fighting.venue.is_none_or(|venue| venue == Venue::BridgeRace);
    if ours {
        let none = Fighting::none(fight.fighting.game);
        if *fight.fighting != none {
            *fight.fighting = none;
        }
        fight.offline.set_if_neq(OfflineArena(false));
        fight.outfit.set_if_neq(Outfit::default());
        if fight.clock.0 != 0.0 {
            fight.clock.0 = 0.0;
        }
        fight.stock.set_if_neq(Resources::default());
        for thing in fought {
            commands.entity(thing).try_despawn();
        }
    }
    for body in dressed {
        commands.entity(body).try_remove::<(Team, NoBar, crate::Carrying, Dressed)>();
    }
}

/// What the server says to this device of its game: the metres of bridge
/// you had, as you come back into it afresh, and every grant since; the
/// chat's lines; and, as you are stood on your island again after a fall,
/// that you are up.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut told: MessageReader<Told>,
    mut tells: MessageReader<Tell>,
    mut racing: ResMut<Racing>,
    mut stock: ResMut<Resources>,
    view: Res<TableView>,
    me: Query<(Entity, Has<Dead>), With<Player>>,
    mut log: ResMut<chat::Log>,
    mut commands: Commands,
) {
    let Some(race) = racing.0.as_mut().filter(|race| race.online) else {
        told.clear();
        tells.clear();
        return;
    };
    for Told(said) in told.read() {
        match said {
            rules::Tell::Metres { metres, grants } => {
                info!("Bridge Race: back with {metres:.1} m of bridge, {grants} grants counted");
                stock.bridge_metres = *metres;
                race.counted = *grants;
                race.told_back = true;
            }
            rules::Tell::Feed(said) => feed_line(&mut log, &view, *said),
            rules::Tell::Nothing => {}
        }
    }
    for tell in tells.read() {
        if let Tell::Arrive {
            venue: Venue::BridgeRace,
            spot: Some(_),
            ..
        } = tell
            && let Ok((you, true)) = me.single()
        {
            commands.entity(you).remove::<Dead>();
        }
    }
}

// ------------------------------------------------------------ the race

/// Follows the game, every frame its table is up: the fight's state as it
/// stands, written as it changes; your metres before the race, those every
/// race begins with; the race begun as its table is played ("Go!", and
/// offline the game itself), or, come into it under way, taken as it
/// stands with nothing of it said again; and what the round says, online
/// as the server sends it to the table's entity and offline as this
/// device's own game has it ([`take_round`]), with your sticks of
/// dynamite.
#[allow(clippy::too_many_arguments)]
fn follow(
    time: Res<Time>,
    mut racing: ResMut<Racing>,
    at: Res<AtTable>,
    view: Res<TableView>,
    rounds: Query<(&RaceRound, Option<&Kits>), With<Remote>>,
    mut fight: Fight,
    me: Query<(&Transform, Has<Dead>), With<Player>>,
    mut finish: MessageWriter<table::Finish>,
    mut say: Say,
    mut commands: Commands,
) {
    let Some(race) = racing.0.as_mut() else {
        return;
    };
    if let Some(sitting) = at.of(Minigame::BridgeRace)
        && usize::from(sitting.seat) != race.seat
    {
        race.seat = usize::from(sitting.seat);
    }
    let seats = view.members.len();
    // Before the race your metres are those every race begins with (but
    // for what the server tells you, back in it afresh): written again
    // should anything have undone them since the table was entered, as
    // another game's way of giving back its fight does, Last Stand's, in
    // the very frame this table is entered from its results, the order
    // of the two being nobody's (mine).
    if !race.begun && !race.told_back {
        fight.stock.set_if_neq(Resources {
            bridge_metres: rules::START_METRES,
            ..default()
        });
    }
    // The race begins as its table is played: "Go!" over your team, and
    // offline the game itself. Online, come into it under way, put back
    // in it as your app was opened again, it is taken as it stands, and
    // nothing of what has happened in it is said again ([`LATE`]): which
    // bridges are across and who holds the golden island, your sticks,
    // the grants, whose metres the server tells you.
    if view.playing() && !race.begun {
        race.begun = true;
        let under_way = view
            .shown
            .and_then(|shown| rounds.get(shown).ok())
            .filter(|(round, _)| race.online && (round.over() || f32::from(round.left) < rules::PLAY_SECS - LATE));
        match under_way {
            Some((round, kits)) => {
                race.joined = round.across.map(|across| across >= 100);
                race.holding = round.hold.map(|hold| hold > 0);
                race.held_once = race.holding;
                race.over = round.over();
                if !race.told_back {
                    race.counted = round.grants;
                }
                let elapsed = (rules::PLAY_SECS - f32::from(round.left)).max(0.0);
                race.since_grant = elapsed % rules::GRANT_EVERY;
                race.sticks = kits.and_then(|kits| kits.0.get(race.seat)).map_or(0, |kit| kit.held(Utility::Dynamite));
                info!("Bridge Race: back in the race {} s in, {} seats, your team {}", rules::PLAY_SECS as u16 - round.left.min(rules::PLAY_SECS as u16), seats, race.team());
            }
            None => {
                let file = ui();
                let (words, colours) = (&file.words.bridge_race, file.colours);
                let mine = race.team();
                let yours = chat::spans(&words.your_team, colours.text, &[("team", &team_name(mine), show::team_ink(mine))]).0;
                let go = show::banner(&mut commands, Some(yours.as_slice()), &[(words.go.clone(), colours.gold)], GO_SECS, false);
                commands.entity(go).insert(table::InTable);
                // A team of two, people or one with the computer, is told
                // to lay one bridge between them, once "Go!" has gone.
                race.one_bridge = (0..seats).filter(|&seat| team(seat) == mine && view.members[seat].id != 0).count() >= 2;
                if !race.online {
                    let people: Vec<bool> = view.members.iter().map(|member| member.id != 0 && !member.ai).collect();
                    race.played = Some(Box::new(offline::Played::new(&people, fastrand::u64(..))));
                }
                info!("Bridge Race: the race is on, {} seats, your team {}", seats, race.team());
            }
        }
    }
    // The round, and your own kit, as they stand: offline this device's
    // game's, online as the server sends them to the table's entity.
    let seat = race.seat;
    let (round, kit) = match (&race.played, view.shown.and_then(|shown| rounds.get(shown).ok())) {
        (Some(played), _) => (Some(played.game.round()), played.game.kits().get(seat).copied()),
        (None, Some((round, kits))) => (Some(round.clone()), kits.and_then(|kits| kits.0.get(seat).copied())),
        (None, None) => (None, None),
    };
    if let Some(round) = round
        && race.begun
    {
        let (feet, dead) = me.single().map_or((Vec3::ZERO, true), |(body, dead)| (body.translation, dead));
        take_round(race, round, feet, dead, &mut fight, &mut finish, &mut say, &mut commands);
    }
    if let Some(kit) = kit {
        let sticks = kit.held(Utility::Dynamite);
        if sticks > race.sticks && race.on() {
            let file = ui();
            show::notice(&mut commands, &mut say, &file.words.bridge_race.dynamite, Some(hud::Picture::Dynamite));
        }
        race.sticks = sticks;
        fight.outfit.set_if_neq(Outfit {
            kit,
            team: Upgrades::default(),
        });
    }
    // Online, how long since your metres were told; how long since the
    // last grant came, offline as the game has it and online by this
    // device's own count from the round that brought it; and a moment after
    // "Go!", to a team of two, that one bridge is laid twice as fast.
    let dt = time.delta_secs();
    race.since_told += dt;
    if race.on() {
        race.since_go += dt;
        race.since_grant = match &race.played {
            Some(played) => played.game.elapsed() % rules::GRANT_EVERY,
            None => (race.since_grant + dt).min(rules::GRANT_EVERY),
        };
        if race.one_bridge && race.since_go >= ONE_BRIDGE_AFTER {
            race.one_bridge = false;
            show::notice(&mut commands, &mut say, &ui().words.bridge_race.one_bridge, None);
        }
    }
    // The fight's state as it stands now, the race begun or over this very
    // frame included.
    let wanted = Fighting {
        playing: race.on(),
        teams_out: race.begun,
        seat: Some(race.seat as u8),
        online: race.online,
        game: race.number,
        members: view.members.iter().map(|member| member.id).collect(),
        watched: false,
        venue: Some(Venue::BridgeRace),
        teams: (0..seats).map(team).collect(),
    };
    if *fight.fighting != wanted {
        *fight.fighting = wanted;
    }
}

/// What the round says, as it changes: each grant come, your team's share
/// added to your metres with "+3m" rising off you (`feet`, unless `dead`);
/// each team's bridge come across to the golden island, said across the
/// upper half and in the chat; each team standing on it all together, its
/// first hold said in the chat and across the upper half, and a hold again
/// across it only while you are far from the island, the line taken down
/// as that hold is broken, with "Hold broken!" over the island in its
/// colour (mine: every hold was said in both, seventeen in one scramble,
/// one left up after its hold had gone, and a hold broken said nothing,
/// seen in a play-test); the fight's clock; and the end, across the screen,
/// and offline how everyone came, for the table to put up and pay.
#[allow(clippy::too_many_arguments)]
fn take_round(
    race: &mut Race,
    round: RaceRound,
    feet: Vec3,
    dead: bool,
    fight: &mut Fight,
    finish: &mut MessageWriter<table::Finish>,
    say: &mut Say,
    commands: &mut Commands,
) {
    let file = ui();
    let words = &file.words.bridge_race;
    let mine = race.team();
    while race.counted < round.grants {
        race.counted += 1;
        race.since_grant = 0.0;
        let share = round.share_of(mine);
        fight.stock.bridge_metres += share;
        if !dead {
            show::granted(commands, feet, &fill(&words.granted, &[("metres", &format!("{share:.0}"))]));
        }
    }
    let ink = file.colours.chat.event;
    for side in 0..rules::TEAMS {
        let colour = file.colours.arena.team(side as u8);
        let name = team_name(side as u8);
        let across = round.across[side] >= 100;
        if across && !race.joined[side] && !round.over() {
            say.log.push(chat::spans(&words.reached, ink, &[("team", &name, colour)]));
            show::notice(commands, say, &fill(&words.reached, &[("team", &name)]), None);
        }
        race.joined[side] = across;
        let holding = round.hold[side] > 0;
        if holding && !race.holding[side] && !round.over() {
            let first = !std::mem::replace(&mut race.held_once[side], true);
            if first {
                say.log.push(chat::spans(&words.holding, ink, &[("team", &name, colour)]));
            }
            if first || feet.xz().distance(rules::GOLDEN) > HOLD_SAID_PAST {
                let posted = show::notice(commands, say, &fill(&words.holding, &[("team", &name)]), None);
                race.hold_said = Some((side, posted));
            }
        }
        if !holding && race.holding[side] && !round.over() {
            if let Some((of, posted)) = race.hold_said
                && of == side
            {
                commands.entity(posted).try_despawn();
                race.hold_said = None;
            }
            show::broken(commands, side as u8);
        }
        race.holding[side] = holding;
    }
    let elapsed = (rules::PLAY_SECS - f32::from(round.left)).max(0.0);
    if fight.clock.0 != elapsed {
        fight.clock.0 = elapsed;
    }
    if round.over() && !race.over {
        race.over = true;
        // The end said on its own: whatever line is up goes, and whatever
        // is rising ("Hold broken!" a moment before a win ran into "Blue
        // wins!", seen in a scripted run).
        show::clear_notices(commands, say);
        for rising in &say.risings {
            commands.entity(rising).try_despawn();
        }
        let (title, caption) = end_words(&round, mine);
        let end = show::banner(commands, caption.as_deref(), &title, END_SECS, true);
        commands.entity(end).insert(table::InTable);
        info!(
            "Bridge Race: over, {:?} won by {:?} (\"{}\" over {:?}), {} m and {} m across",
            round.winner,
            round.finish,
            written(&title),
            caption.as_deref().map(written),
            round.across[0],
            round.across[1]
        );
        if let Some(played) = &race.played
            && let Some((places, scores, total)) = played.game.outcome()
        {
            finish.write(table::Finish { places, scores, total });
        }
    }
    race.round = Some(round);
}

/// Words in pieces, each in its colour: a line across the screen.
type Pieces = Vec<(String, Color)>;

/// What `pieces` say, all together.
fn written(pieces: &[(String, Color)]) -> String {
    pieces.iter().map(|(words, _)| words.as_str()).collect()
}

/// What is said across the screen as the race ends, as the round has it,
/// a team's name in its colour: who won, or a draw ("Red wins!", "Draw"),
/// in gold, over how it came to an end, for you on team `mine`
/// ([`RaceRound::finish`]), in white: held by your team, or by the other
/// (named: until 2026-11-18, the same day, "your team" or "the other team
/// got there first", wrong when the first there lost the hold, seen in a
/// play-test); held by both at once; the time up, however it stood; or
/// every one of a team gone, whoever is left winning. Until earlier that
/// day the time's being up was told by the clock alone and every other end
/// by who won, so that a team all gone was "Your team got there first!",
/// both holding at once "Time's up!", and a skip "got there first" (mine).
fn end_words(round: &RaceRound, mine: u8) -> (Pieces, Option<Pieces>) {
    let file = ui();
    let (words, colours) = (&file.words.bridge_race, file.colours);
    let (gold, white) = (colours.gold, colours.text);
    let named = |template: &str, ink: Color, side: u8| chat::spans(template, ink, &[("team", &team_name(side), show::team_ink(side))]).0;
    let title = match round.winner {
        Some(side) => named(&words.wins, gold, side),
        None => vec![(words.draw.clone(), gold)],
    };
    let caption = match (round.finish, round.winner) {
        (Some(Finish::Held), Some(side)) if side == mine => Some(vec![(words.won.clone(), white)]),
        (Some(Finish::Held), Some(side)) => Some(named(&words.lost, white, side)),
        (Some(Finish::Held), None) => Some(vec![(words.together.clone(), white)]),
        (Some(Finish::Left), Some(side)) => Some(named(&words.team_left, white, 1 - side.min(1))),
        // Everyone gone: nobody is here to be told how.
        (Some(Finish::Left), None) => None,
        (Some(Finish::Time) | None, _) => Some(vec![(words.times_up.clone(), white)]),
    };
    (title, caption)
}

/// A stick of dynamite thrown at its button (`arena::Used`, `utility`), in
/// the race: asked of the server, which throws it for everyone; offline,
/// thrown in this device's own game, which says so as it goes
/// ([`offline::play`] draws it).
fn use_sticks(mut used: MessageReader<Used>, mut racing: ResMut<Racing>, mut requests: MessageWriter<NetAsk>) {
    let Some(race) = racing.0.as_mut().filter(|race| race.on()) else {
        used.clear();
        return;
    };
    for &used in used.read() {
        let Used::Dynamite { heading, far } = used else {
            continue;
        };
        if race.online {
            requests.write(NetAsk::Use(Use::Dynamite { heading, far }));
        } else if let Some(played) = race.played.as_mut() {
            played.game.throw(race.seat, heading, far);
        }
    }
}

/// Online, the metres of bridge you have left, and how many grants they
/// count, told the server as they change, no oftener than once a second
/// ([`METRES_EVERY`]): kept there to give back to you should you come back
/// to the game afresh.
fn tell_metres(mut racing: ResMut<Racing>, stock: Res<Resources>, mut requests: MessageWriter<NetAsk>) {
    let Some(race) = racing.0.as_mut().filter(|race| race.online && race.on()) else {
        return;
    };
    let now = (stock.bridge_metres, race.counted);
    if race.told == Some(now) || race.since_told < METRES_EVERY {
        return;
    }
    requests.write(NetAsk::BridgeRace(rules::Ask::Metres { metres: now.0, grants: now.1 }));
    race.told = Some(now);
    race.since_told = 0.0;
}

/// Everyone in the race on their seat's team ([`Team`]: a punch never lands
/// on a teammate, and the names over heads are in the team's colour once
/// the race is on), and each a name over its head with no bar under it
/// ([`NoBar`]: nobody has health): you, the computer's players offline
/// (`table::ComputerSeat`), and online everyone the server sends, by who
/// they are among the table's seats, each of them marked as the race's to
/// undo ([`Dressed`]). Each written only as it changes.
#[allow(clippy::type_complexity)]
fn dress_bodies(
    racing: Res<Racing>,
    view: Res<TableView>,
    me: Query<(Entity, Option<&Team>), With<Player>>,
    remotes: Query<(Entity, &Member, &Venue, Option<&Team>, Has<Dressed>), (With<Remote>, Without<Player>)>,
    computers: Query<(Entity, &ComputerSeat, Option<&Team>, Has<Dressed>), Without<Player>>,
    mut commands: Commands,
) {
    let Some(race) = racing.0.as_ref() else {
        return;
    };
    if let Ok((you, worn)) = me.single() {
        let yours = Team(race.team());
        if worn != Some(&yours) {
            commands.entity(you).insert(yours);
        }
    }
    let mut dress = |body: Entity, seat: usize, worn: Option<&Team>, marked: bool| {
        let theirs = Team(team(seat));
        if worn != Some(&theirs) {
            commands.entity(body).insert(theirs);
        }
        if !marked {
            commands.entity(body).insert((NoBar, Dressed));
        }
    };
    if race.online {
        for (body, member, &venue, worn, marked) in &remotes {
            if venue != Venue::BridgeRace || member.id == 0 {
                continue;
            }
            if let Some(seat) = view.members.iter().position(|seated| seated.id == member.id) {
                dress(body, seat, worn, marked);
            }
        }
    } else {
        for (body, seat, worn, marked) in &computers {
            dress(body, usize::from(seat.0), worn, marked);
        }
    }
}

/// You have fallen far enough into the void, the race on: dead, hidden
/// where you fell, until you are stood on your island's middle again
/// ([`rules::RESPAWN`]), the server told (`Ask::Respawn`) or offline this
/// device's own game; and while you wait, "Back on your island in 2"
/// across the screen, counting down. And once the race is over, anyone
/// still waiting is stood up at once, the game standing nobody up after
/// its end.
#[allow(clippy::type_complexity)]
fn reap(
    time: Res<Time>,
    mut racing: ResMut<Racing>,
    mut me: Query<(Entity, &Transform, &Venue, Option<&mut Dead>), With<Player>>,
    mut requests: MessageWriter<NetAsk>,
    mut words: DeathWords,
    (mut was_over, mut had): (Local<bool>, Local<bool>),
    mut commands: Commands,
) {
    let file = ui();
    let back_in = &file.words.bridge_race.back_in;
    // Away from the race, the words are taken away once, as it goes: never
    // another game's (Bridge Wars says its own the same way).
    let Some(race) = racing.0.as_mut() else {
        if std::mem::take(&mut *had) {
            words.say(&mut commands, back_in, None);
        }
        *was_over = false;
        return;
    };
    *had = true;
    let Ok((you, body, &venue, dead)) = me.single_mut() else {
        return;
    };
    let ended = race.over && !std::mem::replace(&mut *was_over, race.over);
    match dead {
        Some(mut dead) => {
            dead.left = (dead.left - time.delta_secs()).max(0.0);
            words.say(&mut commands, back_in, Some(dead.left));
            // Over with you still waiting: the server stands you up on your
            // island at a word now (`Ask::Respawn`); offline the game here
            // does (`offline::play`).
            if ended && race.online {
                requests.write(NetAsk::Respawn);
            }
        }
        None => {
            words.say(&mut commands, back_in, None);
            if race.on() && venue == Venue::BridgeRace && body.translation.y < crate::void_y() {
                info!("Bridge Race: fell into the void");
                commands.entity(you).insert(Dead { left: rules::RESPAWN });
                if race.online {
                    requests.write(NetAsk::Respawn);
                } else if let Some(played) = race.played.as_mut() {
                    played.game.fell(race.seat);
                }
            }
        }
    }
}

/// Where the camera looks on from, if not from over you (`crate::Lookout`,
/// the camera standing as it would were you there; mine): falling into
/// the void, and dead, from over where you went off, at the land's height,
/// following you no further down than [`FALL_SEEN`] under it (it followed
/// you down into the void, where there was nothing but the dark to see,
/// seen in a play-test whose player was knocked off at the end and missed
/// the other team's hold that won it); and as the race is won on the
/// golden island, over the island, the winners on it and its ring full,
/// coming there smoothly in [`TO_THE_WINNERS`]. Written only as it changes;
/// with the race gone, given back once (the table puts you where it will).
fn watch_over(
    time: Res<Time>,
    racing: Res<Racing>,
    me: Query<(&Transform, &Venue, Has<Dead>), With<Player>>,
    mut lookout: ResMut<crate::Lookout>,
    (mut toward, mut wrote): (Local<Option<(Vec3, f32)>>, Local<bool>),
) {
    let Some(race) = racing.0.as_ref().filter(|race| race.begun) else {
        *toward = None;
        if std::mem::take(&mut *wrote) {
            lookout.set_if_neq(crate::Lookout(None));
        }
        return;
    };
    let Ok((body, &venue, dead)) = me.single() else {
        return;
    };
    *wrote = true;
    let feet = body.translation;
    let held = race.over && race.round.as_ref().is_some_and(|round| round.finish == Some(Finish::Held));
    lookout.set_if_neq(crate::Lookout(looked_on_from(feet, venue == Venue::BridgeRace, dead, held, lookout.0, &mut toward, time.delta_secs())));
}

/// Where the camera looks on from ([`watch_over`]), you standing at `feet`,
/// `on` the race's map or not, `dead` or not, the race won on the golden
/// island (`held`) or not; where it looked on from last (`was`); and the
/// way it is coming over the golden island, if it is, from where and for
/// how long (`toward`), `dt` seconds on. None: from over you.
fn looked_on_from(feet: Vec3, on: bool, dead: bool, held: bool, was: Option<Vec3>, toward: &mut Option<(Vec3, f32)>, dt: f32) -> Option<Vec3> {
    if held {
        let (from, age) = toward.get_or_insert((was.unwrap_or(feet), 0.0));
        *age += dt;
        let t = (*age / TO_THE_WINNERS).clamp(0.0, 1.0);
        let gold = Vec3::new(rules::GOLDEN.x, rules::LIFT, rules::GOLDEN.y);
        return Some(from.lerp(gold, t * t * (3.0 - 2.0 * t)));
    }
    *toward = None;
    (on && (dead || feet.y < rules::LIFT - FALL_SEEN)).then(|| Vec3::new(feet.x, feet.y.max(rules::LIFT - FALL_SEEN), feet.z))
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use roundtown_net::table::{Board, Stage};

    use super::*;

    #[test]
    fn every_button_round_the_jump_button_is_clear_of_every_other() {
        // The fist in the jump button's place, the jump button, the plank,
        // the five buttons for things to use and the chat's keyboard.
        let (jump_right, jump_bottom, jump_size) = hud::ARENA_JUMP;
        let (chat_right, chat_bottom, chat_size) = hud::CHAT_BUTTON;
        let mut buttons = vec![
            ((0.0, 0.0), crate::JUMP_SIZE_VMIN),
            ((jump_right, jump_bottom), jump_size),
            (hud::ARENA_BRIDGE, hud::ACTION_BUTTON),
            ((chat_right, chat_bottom), chat_size),
        ];
        buttons.extend((0..UTILITY_SLOTS).map(|slot| (hud::slot_place(slot), hud::ACTION_BUTTON)));
        hud::clear_of_each_other(&buttons).unwrap();
    }

    /// A world with what this game's own systems go by, and nothing else
    /// of the game round it.
    fn world() -> World {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.init_resource::<Racing>();
        world.init_resource::<Fighting>();
        world.init_resource::<OfflineArena>();
        world.init_resource::<Resources>();
        world.init_resource::<Outfit>();
        world.init_resource::<FightTime>();
        world.init_resource::<Begun>();
        world.init_resource::<chat::Log>();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<hud::Pictures>();
        world.init_resource::<TableView>();
        world.init_resource::<AtTable>();
        world.init_resource::<Messages<table::Entered>>();
        world.init_resource::<Messages<table::Gone>>();
        world.init_resource::<Messages<table::Finish>>();
        world
    }

    /// You sit at a table of Bridge Race in seat 0, the server's if
    /// `online`, as the table has it (`table::Seating::open`): at it, and
    /// told so, at another table than any you were at.
    fn sit(world: &mut World, online: bool) {
        world.resource_mut::<AtTable>().0 = Some(table::Sitting {
            game: Minigame::BridgeRace,
            seat: 0,
            online,
            id: 1,
        });
        world.write_message(table::Entered {
            game: Minigame::BridgeRace,
            seat: 0,
            online,
            same_table: false,
        });
    }

    /// A table this device hosts, offline, you in seat 0 and the computer
    /// in seat 1, at `stage`.
    fn hosted(world: &mut World, stage: Stage) {
        let me = Member {
            id: 7,
            name: "Mochi".into(),
            look: "boy".into(),
            ai: false,
        };
        let bot = Member {
            id: 2,
            name: "Dori".into(),
            look: "girl".into(),
            ai: true,
        };
        *world.resource_mut::<TableView>() = TableView {
            shown: None,
            board: Some(Board {
                game: Minigame::BridgeRace,
                id: 0,
                stage,
                left: 0,
                people: 1,
                seats: 2,
            }),
            members: vec![me, bot],
        };
    }

    #[test]
    fn at_a_table_the_fight_is_the_race_s_from_the_waiting_room_to_the_end_and_given_back_once_after() {
        let mut world = world();
        // The game's own systems, kept from frame to frame as the app keeps
        // them, each reading what is said once.
        let (enter, follow) = (world.register_system(enter), world.register_system(follow));
        // Sat at a table offline: ten metres of bridge, the fight here,
        // offline, numbered afresh, nothing being played yet.
        sit(&mut world, false);
        hosted(&mut world, Stage::Waiting);
        world.run_system(enter).expect("in");
        world.run_system(follow).expect("followed");
        assert_eq!(world.resource::<Resources>().bridge_metres, rules::START_METRES);
        assert!(world.resource::<OfflineArena>().0, "offline");
        let fighting = world.resource::<Fighting>();
        assert_eq!((fighting.venue, fighting.seat, fighting.playing, fighting.teams_out), (Some(Venue::BridgeRace), Some(0), false, false));
        assert_eq!((fighting.members.clone(), fighting.teams.clone()), (vec![7, 2], vec![0, 1]));
        let number = fighting.game;
        assert!(number > 0);
        // The table is played: the race is on, the teams out, and this
        // device's own game begun, a seat each.
        hosted(&mut world, Stage::Playing);
        world.run_system(follow).expect("followed");
        let fighting = world.resource::<Fighting>();
        assert!(fighting.playing && fighting.teams_out && fighting.game == number);
        let game = &world.resource::<Racing>().0.as_ref().expect("a race").played.as_ref().expect("played here").game;
        assert!(game.playing() && game.person(0) && !game.person(1));
        // A grant comes in the game: your team's share of it on your
        // metres, once, however often the round is looked at.
        {
            let mut racing = world.resource_mut::<Racing>();
            let played = racing.0.as_mut().and_then(|race| race.played.as_mut()).expect("played here");
            while played.game.round().grants < 1 {
                played.game.step(0.1);
            }
        }
        world.run_system(follow).expect("followed");
        world.run_system(follow).expect("followed");
        assert_eq!(world.resource::<Resources>().bridge_metres, rules::START_METRES + rules::grant_metres(2));
        // Gone from the tables: the fight given back, "no game" with the
        // game's number kept, and nothing of the race left.
        world.resource_mut::<AtTable>().0 = None;
        world.write_message(table::Gone { game: Minigame::BridgeRace });
        world.run_system(enter).expect("out");
        assert!(*world.resource::<Fighting>() == Fighting::none(number));
        assert!(!world.resource::<OfflineArena>().0, "no game offline");
        assert_eq!(*world.resource::<Resources>(), Resources::default());
        assert!(world.resource::<Racing>().0.is_none());
    }

    #[test]
    fn the_chat_writes_who_fell_by_whose_doing_and_whose_bridge_was_holed() {
        let mut world = world();
        hosted(&mut world, Stage::Playing);
        let view = world.resource::<TableView>().clone();
        let written = |said: Feed| -> Option<(String, Vec<Color>)> {
            let line = feed_words(&view, said)?;
            Some((line.0.iter().map(|(words, _)| words.as_str()).collect(), line.0.iter().map(|(_, colour)| *colour).collect()))
        };
        let file = ui();
        let (words, colours) = (&file.words.bridge_race, file.colours);
        // Knocked off by the other team: both names, each in its team's
        // colour.
        let (line, inks) = written(Feed::Fell { seat: 1, by: Some(0) }).expect("a line");
        assert_eq!(line, fill(&words.knocked, &[("killer", "Mochi"), ("name", "Dori")]));
        assert!(inks.contains(&colours.arena.team(0)) && inks.contains(&colours.arena.team(1)));
        let (line, _) = written(Feed::Fell { seat: 0, by: None }).expect("a line");
        assert_eq!(line, fill(&words.fell, &[("name", "Mochi")]));
        let (line, _) = written(Feed::Blew { by: 1, team: Some(0), planks: 3 }).expect("a line");
        assert_eq!(line, fill(&words.blew, &[("name", "Dori"), ("team", &words.red)]));
        // A stick that took nothing says nothing.
        assert_eq!(written(Feed::Blew { by: 1, team: None, planks: 0 }), None);
    }

    /// Whether anything on the screen says `words`.
    fn says(world: &mut World, words: &str) -> bool {
        world.query::<&Text>().iter(world).any(|text| text.0 == words)
    }

    #[test]
    fn taken_on_into_another_game_s_table_the_race_gives_the_fight_back_as_on_leaving_the_tables() {
        let mut world = world();
        let (enter, follow, dress) = (world.register_system(enter), world.register_system(follow), world.register_system(dress_bodies));
        // Online, you and one of your group in the race, its body as the
        // server sends it.
        let you = world.spawn((Player, Transform::default(), Venue::BridgeRace)).id();
        let mate = Member {
            id: 8,
            name: "Dubu".into(),
            look: "girl".into(),
            ai: false,
        };
        let theirs = world.spawn((Remote, mate.clone(), Venue::BridgeRace, Transform::default())).id();
        sit(&mut world, true);
        hosted(&mut world, Stage::Playing);
        world.resource_mut::<TableView>().members[1] = mate;
        for system in [enter, follow, dress] {
            world.run_system(system).expect("run");
        }
        let number = world.resource::<Fighting>().game;
        assert_eq!(world.resource::<Fighting>().venue, Some(Venue::BridgeRace));
        assert!(world.get::<Team>(theirs) == Some(&Team(1)) && world.get::<NoBar>(theirs).is_some());
        assert_eq!(world.get::<Team>(you), Some(&Team(0)));
        // Taken on into a table of Last Stand from the race's results by
        // your group's leader, the mate with you, its body on Last Stand's
        // venue by now: the table says so with Last Stand's `Entered`, and
        // no `Gone`.
        world.resource_mut::<AtTable>().0 = Some(table::Sitting {
            game: Minigame::LastStand,
            seat: 1,
            online: true,
            id: 3,
        });
        world.write_message(table::Entered {
            game: Minigame::LastStand,
            seat: 1,
            online: true,
            same_table: false,
        });
        *world.get_mut::<Venue>(theirs).expect("a body") = Venue::LastStand;
        world.run_system(enter).expect("on");
        // Given back as on leaving the tables: "no game" with the number
        // kept, nothing offline, no metres, no kit, no clock; and off the
        // mate's body, wherever it is, the race's team and the bar's
        // absence.
        assert!(world.resource::<Racing>().0.is_none());
        assert!(*world.resource::<Fighting>() == Fighting::none(number));
        assert!(!world.resource::<OfflineArena>().0);
        assert_eq!(*world.resource::<Resources>(), Resources::default());
        assert_eq!(*world.resource::<Outfit>(), Outfit::default());
        world.flush();
        assert!(world.get::<Team>(theirs).is_none() && world.get::<NoBar>(theirs).is_none() && world.get::<Dressed>(theirs).is_none());
        // And nothing of the race comes back, frame after frame, over what
        // the next game writes: its fight, and your team as it has it.
        *world.resource_mut::<Fighting>() = Fighting {
            venue: Some(Venue::LastStand),
            ..Fighting::none(number + 1)
        };
        world.entity_mut(you).insert(Team(5));
        for _ in 0..3 {
            for system in [enter, follow, dress] {
                world.run_system(system).expect("run");
            }
        }
        assert_eq!(world.resource::<Fighting>().venue, Some(Venue::LastStand));
        assert_eq!(world.get::<Team>(you), Some(&Team(5)));
        assert!(world.get::<Team>(theirs).is_none());
        // The next game having taken the fight already in the very frame
        // the race is given back (whichever goes first is nobody's say):
        // its fight, metres and all, left as it has them.
        sit(&mut world, true);
        world.run_system(enter).expect("in");
        world.run_system(follow).expect("followed");
        assert_eq!(world.resource::<Fighting>().venue, Some(Venue::BridgeRace));
        world.resource_mut::<AtTable>().0 = None;
        *world.resource_mut::<Fighting>() = Fighting {
            venue: Some(Venue::Arena),
            ..Fighting::none(number + 9)
        };
        world.resource_mut::<Resources>().bridge_metres = 25.0;
        world.run_system(enter).expect("out");
        assert!(world.resource::<Racing>().0.is_none());
        assert_eq!(world.resource::<Fighting>().venue, Some(Venue::Arena));
        assert_eq!(world.resource::<Resources>().bridge_metres, 25.0);
    }

    #[test]
    fn a_race_come_back_into_under_way_is_taken_as_it_stands_and_nothing_of_it_said_again() {
        let mut world = world();
        world.init_resource::<Messages<Told>>();
        world.init_resource::<Messages<Tell>>();
        let (enter, hear, follow) = (world.register_system(enter), world.register_system(hear), world.register_system(follow));
        world.spawn((Player, Transform::default(), Venue::BridgeRace));
        let file = ui();
        let words = &file.words.bridge_race;
        // The table as the server sends it, a hundred seconds in: red's
        // bridge across and red holding the golden island, six grants come,
        // a stick on your button.
        let board = Board {
            game: Minigame::BridgeRace,
            id: 4,
            stage: Stage::Playing,
            left: 0,
            people: 1,
            seats: 2,
        };
        let round = RaceRound {
            left: 260,
            grants: 6,
            share: [60, 60],
            hold: [4, 0],
            on_gold: [1, 0],
            winner: None,
            finish: None,
            across: [100, 60],
            players: [1, 1],
        };
        let kits = Kits(vec![roundtown_net::Kit::default().holding(Utility::Dynamite, 1), roundtown_net::Kit::default()]);
        let shown = world.spawn((Remote, board.clone(), round.clone(), kits)).id();
        // This device put back in it, its app opened again: the table, and
        // the race, afresh.
        sit(&mut world, true);
        hosted(&mut world, Stage::Playing);
        world.resource_mut::<TableView>().shown = Some(shown);
        world.run_system(enter).expect("in");
        world.run_system(follow).expect("followed");
        // Nothing said: no "Go!", no line across the upper half (each of
        // which goes with a line in the chat), nothing rising off you.
        assert!(!says(&mut world, &words.go) && !says(&mut world, &fill(&words.your_team, &[("team", &words.red)])));
        assert!(!says(&mut world, &words.dynamite) && !says(&mut world, &fill(&words.granted, &[("metres", "6")])));
        assert_eq!(world.query_filtered::<(), With<show::Notice>>().iter(&world).count(), 0);
        {
            let race = world.resource::<Racing>().0.as_ref().expect("a race");
            assert!(race.begun && !race.over);
            assert_eq!((race.joined, race.holding, race.sticks, race.counted), ([true, false], [true, false], 1, 6));
        }
        assert!(world.resource::<Fighting>().playing && world.resource::<Fighting>().teams_out);
        // Your metres as the server tells them; and a grant after, said as
        // ever.
        world.write_message(Told(rules::Tell::Metres { metres: 23.5, grants: 6 }));
        world.run_system(hear).expect("heard");
        world.run_system(follow).expect("followed");
        assert_eq!(world.resource::<Resources>().bridge_metres, 23.5);
        *world.get_mut::<RaceRound>(shown).expect("the round") = RaceRound { grants: 7, left: 244, ..round };
        world.run_system(follow).expect("followed");
        assert_eq!(world.resource::<Resources>().bridge_metres, 29.5);
        assert!(says(&mut world, &fill(&words.granted, &[("metres", "6")])));
        // Seen from its start, as ever: "Go!".
        let mut begun = self::world();
        let (enter, follow) = (begun.register_system(super::enter), begun.register_system(super::follow));
        begun.spawn((Player, Transform::default(), Venue::BridgeRace));
        let shown = begun.spawn((Remote, board, fresh_round())).id();
        sit(&mut begun, true);
        hosted(&mut begun, Stage::Playing);
        begun.resource_mut::<TableView>().shown = Some(shown);
        begun.run_system(enter).expect("in");
        begun.run_system(follow).expect("followed");
        assert!(says(&mut begun, &words.go));
    }

    /// A round as a race begins.
    fn fresh_round() -> RaceRound {
        RaceRound {
            left: rules::PLAY_SECS as u16,
            grants: 0,
            share: [60, 60],
            hold: [0, 0],
            on_gold: [0, 0],
            winner: None,
            finish: None,
            across: [0, 0],
            players: [1, 1],
        }
    }

    #[test]
    fn the_end_is_said_as_it_came() {
        let file = ui();
        let words = &file.words.bridge_race;
        let over = |winner: Option<u8>, finish: Finish| RaceRound {
            winner,
            finish: Some(finish),
            left: 100,
            ..fresh_round()
        };
        let said = |(title, caption): (Pieces, Option<Pieces>)| (written(&title), caption.as_deref().map(written));
        let red_wins = fill(&words.wins, &[("team", &words.red)]);
        // Held: by your team, or the other, named; both at once, a draw.
        assert_eq!(said(end_words(&over(Some(0), Finish::Held), 0)), (red_wins.clone(), Some(words.won.clone())));
        assert_eq!(said(end_words(&over(Some(0), Finish::Held), 1)), (red_wins.clone(), Some(fill(&words.lost, &[("team", &words.red)]))));
        assert_eq!(said(end_words(&over(None, Finish::Held), 0)), (words.draw.clone(), Some(words.together.clone())));
        // The time up, or skipped: the time, whoever won.
        assert_eq!(said(end_words(&over(Some(0), Finish::Time), 0)), (red_wins.clone(), Some(words.times_up.clone())));
        assert_eq!(said(end_words(&over(None, Finish::Time), 1)), (words.draw.clone(), Some(words.times_up.clone())));
        // Blue all gone: red wins, and blue is said to have left.
        assert_eq!(said(end_words(&over(Some(0), Finish::Left), 0)), (red_wins, Some(fill(&words.team_left, &[("team", &words.blue)]))));
        assert_eq!(said(end_words(&over(None, Finish::Left), 0)), (words.draw.clone(), None));
        // A team's name in its own colour, the rest of the title in gold
        // and of the caption in white.
        let (title, caption) = end_words(&over(Some(1), Finish::Held), 0);
        assert!(title.contains(&(words.blue.clone(), show::team_ink(1))), "{title:?}");
        assert!(title.iter().any(|(_, colour)| *colour == file.colours.gold));
        let caption = caption.expect("how");
        assert!(caption.contains(&(words.blue.clone(), show::team_ink(1))), "{caption:?}");
        assert!(caption.iter().any(|(_, colour)| *colour == file.colours.text));
        // Every one of them words the fonts draw, the team filled in.
        assert!(words.team_left.contains("{team}") && words.lost.contains("{team}"));
        for said in [&words.together, &words.team_left, &words.times_up, &words.won, &words.lost, &words.hold_broken, &words.jump, &words.one_bridge, &words.missed] {
            assert!(!said.trim().is_empty() && said.chars().all(roundtown_net::text::allowed), "{said:?}");
        }
    }

    #[test]
    fn a_stick_lost_in_the_void_is_said_in_the_chat_whose_it_was() {
        let mut world = world();
        hosted(&mut world, Stage::Playing);
        let view = world.resource::<TableView>().clone();
        let line = feed_words(&view, Feed::Lost { by: 1 }).expect("a line");
        let file = ui();
        assert_eq!(written(&line.0), fill(&file.words.bridge_race.missed, &[("name", "Dori")]));
        assert!(line.0.contains(&("Dori".to_string(), file.colours.arena.team(1))));
    }

    #[test]
    fn a_hold_is_said_across_the_upper_half_its_line_goes_as_it_is_broken_and_the_break_is_said() {
        let mut world = world();
        world.init_resource::<Messages<Told>>();
        let (enter, follow) = (world.register_system(enter), world.register_system(follow));
        // Online, you on red by the golden island, the race from its start.
        let you = world.spawn((Player, Transform::from_xyz(-2.0, rules::LIFT, 0.0), Venue::BridgeRace)).id();
        let board = Board {
            game: Minigame::BridgeRace,
            id: 4,
            stage: Stage::Playing,
            left: 0,
            people: 1,
            seats: 2,
        };
        let shown = world.spawn((Remote, board, fresh_round())).id();
        sit(&mut world, true);
        hosted(&mut world, Stage::Playing);
        world.resource_mut::<TableView>().shown = Some(shown);
        world.run_system(enter).expect("in");
        world.run_system(follow).expect("followed");
        let file = ui();
        let words = &file.words.bridge_race;
        let holding = fill(&words.holding, &[("team", &words.blue)]);
        let notices = |world: &mut World| world.query_filtered::<Entity, With<show::Notice>>().iter(world).count();
        let hold = |world: &mut World, tenths: [u8; 2]| {
            world.get_mut::<RaceRound>(shown).expect("the round").hold = tenths;
        };
        let held_once = |world: &World| world.resource::<Racing>().0.as_ref().expect("a race").held_once;
        // Blue holds: across the upper half, the first time (and in the
        // chat, which is said with it).
        hold(&mut world, [0, 2]);
        world.run_system(follow).expect("followed");
        assert_eq!(notices(&mut world), 1);
        assert!(says(&mut world, &holding));
        assert_eq!(held_once(&world), [false, true]);
        // Knocked off it: the line taken down, and "Hold broken!" said.
        hold(&mut world, [0, 0]);
        world.run_system(follow).expect("followed");
        world.flush();
        assert_eq!(notices(&mut world), 0);
        assert!(says(&mut world, &words.hold_broken));
        // Again, by the island: nothing across the upper half, the ring and
        // "Hold!" saying it.
        hold(&mut world, [0, 3]);
        world.run_system(follow).expect("followed");
        assert_eq!(notices(&mut world), 0);
        // And again, with you far off on your own island: across the upper
        // half.
        hold(&mut world, [0, 0]);
        world.run_system(follow).expect("followed");
        world.get_mut::<Transform>(you).expect("you").translation = Vec3::new(rules::ISLANDS[0].x, rules::LIFT, rules::ISLANDS[0].y);
        hold(&mut world, [0, 1]);
        world.run_system(follow).expect("followed");
        assert_eq!(notices(&mut world), 1);
    }

    #[test]
    fn a_team_of_two_is_told_once_go_has_gone_to_build_one_bridge_and_a_team_of_one_is_not() {
        for seats in [2, 4] {
            let mut world = world();
            let (enter, follow) = (world.register_system(enter), world.register_system(follow));
            world.spawn((Player, Transform::default(), Venue::BridgeRace));
            sit(&mut world, false);
            hosted(&mut world, Stage::Playing);
            // Four seats: you and the computer on red, two of it on blue.
            if seats == 4 {
                let mut view = world.resource_mut::<TableView>();
                let bot = |id: u64, name: &str| Member {
                    id,
                    name: name.into(),
                    look: "girl".into(),
                    ai: true,
                };
                view.members.extend([bot(3, "Bori"), bot(4, "Latte")]);
                view.board.as_mut().expect("a board").seats = 4;
            }
            world.run_system(enter).expect("in");
            world.run_system(follow).expect("followed");
            let words = ui().words.bridge_race.one_bridge.clone();
            assert!(!says(&mut world, &words), "not with \"Go!\"");
            let mut told = 0;
            for _ in 0..50 {
                world.resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
                world.run_system(follow).expect("followed");
                told += usize::from(says(&mut world, &words) && told == 0);
            }
            assert_eq!(told, usize::from(seats == 4), "{seats} seats");
        }
    }

    #[test]
    fn falling_and_dead_the_camera_stays_at_the_land_over_where_you_went_and_goes_to_the_winners() {
        let mut toward = None;
        let feet = Vec3::new(-50.0, rules::LIFT, -10.0);
        // On your feet, jumping, or off the race's map: over you.
        assert_eq!(looked_on_from(feet, true, false, false, None, &mut toward, 0.1), None);
        assert_eq!(looked_on_from(feet + Vec3::Y, true, false, false, None, &mut toward, 0.1), None);
        assert_eq!(looked_on_from(feet - Vec3::Y * 9.0, false, false, false, None, &mut toward, 0.1), None);
        // Falling, and dead far down: over where you are, at the land.
        let falling = looked_on_from(feet - Vec3::Y * 9.0, true, false, false, None, &mut toward, 0.1).expect("held up");
        assert_eq!(falling, Vec3::new(feet.x, rules::LIFT - FALL_SEEN, feet.z));
        let dead = looked_on_from(feet - Vec3::Y * 40.0, true, true, false, Some(falling), &mut toward, 0.1).expect("held up");
        assert_eq!(dead, falling);
        // Won on the golden island: on to it, smoothly, from where it was,
        // and there once it has come.
        let gold = Vec3::new(rules::GOLDEN.x, rules::LIFT, rules::GOLDEN.y);
        let first = looked_on_from(feet, true, false, true, Some(dead), &mut toward, 0.1).expect("going");
        assert!(first.distance(dead) < 2.0 && first.distance(gold) < dead.distance(gold), "{first}");
        let mut at = first;
        for _ in 0..30 {
            at = looked_on_from(feet, true, false, true, Some(at), &mut toward, 0.1).expect("going");
        }
        assert!(at.distance(gold) < 1e-3, "{at}");
        // Over another way: over you again.
        assert_eq!(looked_on_from(feet, true, false, false, Some(at), &mut toward, 0.1), None);
        assert!(toward.is_none());
    }

    #[test]
    fn the_camera_is_given_back_with_the_race() {
        let mut world = world();
        world.init_resource::<crate::Lookout>();
        let watch = world.register_system(watch_over);
        world.spawn((Player, Transform::from_xyz(-50.0, -40.0, -10.0), Venue::BridgeRace, Dead { left: 1.0 }));
        let mut race = Race::new(1, true, 0);
        race.begun = true;
        world.resource_mut::<Racing>().0 = Some(race);
        world.run_system(watch).expect("watched");
        assert_eq!(world.resource::<crate::Lookout>().0, Some(Vec3::new(-50.0, rules::LIFT - FALL_SEEN, -10.0)));
        world.resource_mut::<Racing>().0 = None;
        world.run_system(watch).expect("watched");
        assert_eq!(world.resource::<crate::Lookout>().0, None);
    }
}
