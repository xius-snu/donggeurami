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
//! over it, and an arrow at the edge of the screen toward the golden island
//! while it is off the screen, with how far.

mod offline;
mod show;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_race::{self as rules, Feed, RaceRound, team};
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

/// A game of Bridge Race on this device, from the table's word that you sit
/// at one to its end: its number among every fight's (`arena::Begun`), whose
/// bridges are its own; whether the server plays it; your seat; whether it
/// has begun here (the "Go!" said) and whether it is over (its end said);
/// how many of the round's grants your metres count; the round as last
/// seen, which what is drawn goes by; your sticks as last seen, to say one
/// that comes; what the server was last told of your metres, and how long
/// ago; whether each team's bridge was across, and each team holding the
/// golden island, as last said; and offline, the game itself.
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
    joined: [bool; rules::TEAMS],
    holding: [bool; rules::TEAMS],
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
            joined: [false; rules::TEAMS],
            holding: [false; rules::TEAMS],
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
/// world has yet to be told of), and the pictures it may have.
#[derive(SystemParam)]
struct Say<'w, 's> {
    log: ResMut<'w, chat::Log>,
    images: ResMut<'w, Assets<Image>>,
    pictures: ResMut<'w, hud::Pictures>,
    notices: Query<'w, 's, Entity, Or<(With<show::Notice>, With<crate::arena::Alert>)>>,
    posted: Local<'s, Option<Entity>>,
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
                // A fall into the void, once everyone has moved.
                reap.after(crate::move_bodies).before(crate::follow_camera),
                // What is drawn over the world, once the camera is placed.
                (show::plate, show::ring, show::hold_words, show::point, show::rise, show::fade_notices)
                    .after(crate::follow_camera),
            ),
        );
}

/// The buttons round the screen at a table of Bridge Race, Bridge Wars'
/// less what is not in the race (Hajun: "similar controls and cam with
/// bridge wars"): your name top left, with no health (nobody has any: the
/// void is the only way to die) and none of the four things the generators
/// make, nor the minimap; and round the jump button the fist in its place,
/// which punches, the jump button beside it, the plank, which builds, the
/// five buttons for things to use (the dynamite), and the chat's keyboard,
/// each where Bridge Wars has it.
pub(crate) fn spawn_bar(commands: &mut Commands, _assets: &AssetServer, images: &mut Assets<Image>) {
    let (left, _right) = hud::game_bar(commands);
    commands.spawn((ChildOf(left), hud::name_plate()));
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
/// in their teams' colours: who fell into the void, by whose doing; and
/// whose stick holed whose bridge. Nothing of a stick that took nothing.
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
    };
    Some(line)
}

// ------------------------------------------------------------ in and out

/// What the table says of Bridge Race: that you sit at one of its tables
/// (`table::Entered`), which begins the game here afresh, the fight's state
/// this game's from now on, and its pieces over the world put up again (the
/// table has taken down everything of its own, `table::InTable`); the same
/// table told again (your seat moved before the race, or this device put
/// back into it) keeps what it has and takes the seat. And that you have
/// left the tables (`table::Gone`): the fight's state given back, "no
/// game" written once, and whatever the fight put up taken down.
#[allow(clippy::too_many_arguments)]
fn enter(
    mut entered: MessageReader<table::Entered>,
    mut gone: MessageReader<table::Gone>,
    mut racing: ResMut<Racing>,
    mut fight: Fight,
    worn: Query<(Entity, &Venue), (With<NoBar>, Without<Player>)>,
    fought: Query<Entity, With<InArena>>,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<hud::Pictures>,
    mut commands: Commands,
) {
    for left in gone.read() {
        if left.game != Minigame::BridgeRace || racing.0.take().is_none() {
            continue;
        }
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
        for thing in &fought {
            commands.entity(thing).try_despawn();
        }
        for (body, &venue) in &worn {
            if venue == Venue::BridgeRace {
                commands.entity(body).try_remove::<(Team, NoBar, crate::Carrying)>();
            }
        }
        info!("Bridge Race: the fight given back");
    }
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
/// stands, written as it changes; the race begun as its table is played
/// ("Go!", and offline the game itself); and what the round says, online
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
    // The race begins as its table is played.
    if view.playing() && !race.begun {
        race.begun = true;
        let file = ui();
        let words = &file.words.bridge_race;
        let yours = fill(&words.your_team, &[("team", &team_name(race.team()))]);
        hud::announce(&mut commands, Some(&yours), &words.go, hud::HEADLINE, 2.5).insert(table::InTable);
        if !race.online {
            let people: Vec<bool> = view.members.iter().map(|member| member.id != 0 && !member.ai).collect();
            race.played = Some(Box::new(offline::Played::new(&people, fastrand::u64(..))));
        }
        info!("Bridge Race: the race is on, {} seats, your team {}", seats, race.team());
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
    // Online, how long since your metres were told.
    race.since_told += time.delta_secs();
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
/// each team's bridge come across to the golden island, and each team
/// standing on it all together, said across the upper half and in the
/// chat; the fight's clock; and the end, across the screen, and offline
/// how everyone came, for the table to put up and pay.
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
        let share = round.share_of(mine);
        fight.stock.bridge_metres += share;
        if !dead {
            show::rising(commands, feet, &fill(&words.granted, &[("metres", &format!("{share:.0}"))]));
        }
    }
    let ink = file.colours.chat.event;
    for side in 0..rules::TEAMS {
        let colour = file.colours.arena.team(side as u8);
        let name = team_name(side as u8);
        let across = round.across[side] >= 100;
        if across && !race.joined[side] && !round.over {
            say.log.push(chat::spans(&words.reached, ink, &[("team", &name, colour)]));
            show::notice(commands, say, &fill(&words.reached, &[("team", &name)]), None);
        }
        race.joined[side] = across;
        let holding = round.hold[side] > 0;
        if holding && !race.holding[side] && !round.over {
            say.log.push(chat::spans(&words.holding, ink, &[("team", &name, colour)]));
            show::notice(commands, say, &fill(&words.holding, &[("team", &name)]), None);
        }
        race.holding[side] = holding;
    }
    let elapsed = (rules::PLAY_SECS - f32::from(round.left)).max(0.0);
    if fight.clock.0 != elapsed {
        fight.clock.0 = elapsed;
    }
    if round.over && !race.over {
        race.over = true;
        // The end said on its own: whatever line is up goes.
        show::clear_notices(commands, say);
        let title = match round.winner {
            Some(side) => fill(&words.wins, &[("team", &team_name(side))]),
            None => words.draw.clone(),
        };
        let caption = match round.winner {
            _ if round.left == 0 => words.times_up.clone(),
            Some(side) if side == mine => words.won.clone(),
            Some(_) => words.lost.clone(),
            None => words.times_up.clone(),
        };
        hud::announce(commands, Some(&caption), &title, hud::HEADLINE, 3.0).insert(table::InTable);
        info!("Bridge Race: over, {:?} won, {} m and {} m across", round.winner, round.across[0], round.across[1]);
        if let Some(played) = &race.played
            && let Some((places, scores, total)) = played.game.outcome()
        {
            finish.write(table::Finish { places, scores, total });
        }
    }
    race.round = Some(round);
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
/// they are among the table's seats. Each written only as it changes.
#[allow(clippy::type_complexity)]
fn dress_bodies(
    racing: Res<Racing>,
    view: Res<TableView>,
    me: Query<(Entity, Option<&Team>), With<Player>>,
    remotes: Query<(Entity, &Member, &Venue, Option<&Team>, Has<NoBar>), (With<Remote>, Without<Player>)>,
    computers: Query<(Entity, &ComputerSeat, Option<&Team>, Has<NoBar>), Without<Player>>,
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
            commands.entity(body).insert(NoBar);
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
        world.write_message(table::Entered {
            game: Minigame::BridgeRace,
            seat: 0,
            online: false,
            same_table: false,
        });
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
}
