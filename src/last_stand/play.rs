//! A game of Last Stand being played: begun and ended with its table
//! ([`enter`]), the fight's state owned meanwhile ([`claim`], [`publish`]),
//! and the game itself, which this device runs offline ([`run_offline`]) and
//! follows online ([`follow_online`], [`ask_online`]).
//!
//! **Offline** the game is the shared [`Game`], made as the table begins
//! its play, everyone where it has them, and stepped every frame: told
//! where you stand and the town's clock, and what you did (a punch that
//! landed on a zombie's body, `crate::Landed`; a card pressed in a shop,
//! `market::Picked`; something used off a button, `arena::Used`; Ready
//! pressed; a fall into the void). What it says happened is carried out
//! here, as the server carries it out online: a zombie risen is a body
//! (`lobby::spawn_bot_at`, its kind and health on it as a [`Shambler`], as
//! the server's are), moved where the game has it, only as that changes;
//! a fireball, a stick of dynamite, a spitter's ball and what lies about
//! are the kit's own (`fire`, `dynamite`, `loot`) or this game's
//! ([`looks`](super::looks)); you are knocked by a big one's blow, and
//! stood again by the anchor; and at its end the table is told how it came
//! out (`table::Finish`).
//!
//! **Online** the server runs it: how it stands comes on the table's
//! entity ([`StandRound`], [`StandUpgrades`], `Purses`, `Kits`), the
//! zombies as bodies with a [`Shambler`] on each, and what you do is asked
//! of it (`Ask::Punch` by the zombie's member, which `net` sends for any
//! punch on someone this device does not move; `Ask::LastStand`;
//! `Ask::Use`; `Ask::Respawn`).

use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::ZOMBIE_LOOK;
use roundtown_net::last_stand::{
    self as rules, Event, Game, Knock, RESPAWN_SECS, Shambler, StandRound, StandStage, StandUpgrades,
};
use roundtown_net::{Ask as NetAsk, Kits, Loot, Member, Minigame, Purses, Upgrades, Use, Venue};

use super::looks::SpitBall;
use super::{Happened, ReadyAsked, Stand, StandSystems, TEAM};
use crate::arena::{self, Begun, FightTime, Fighting, Health, InArena, OfflineArena, Outfit, Resources, Used, Wears};
use crate::clock::TownClock;
use crate::lobby::{self, Whereabouts};
use crate::loot::{self, Key, PadTop};
use crate::market::{Picked, Stalls};
use crate::net::Online;
use crate::table::{self, TableView};
use crate::{Dead, Intent, Landed, Lookout, OrbitCamera, Player, PlayerJump, Shove, Team, WalkCycle, dynamite, fire};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (enter, claim, begin_offline, follow_online)
            .chain()
            .in_set(StandSystems)
            .after(table::TableSystems)
            .before(crate::read_player_input)
            .before(crate::island::show_where_i_am),
    )
    .add_systems(
        Update,
        (run_offline, put_back, ask_online, publish)
            .chain()
            .in_set(StandSystems)
            .after(crate::move_bodies)
            .after(arena::ArenaSystems)
            .before(crate::animate_walk),
    );
}

/// A zombie's body this device moves itself, offline: the game's number
/// for it.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct OwnZombie(pub(super) u32);

/// What a game of Last Stand takes over of the fight's state as it begins
/// and gives back as it ends: the fight itself, whether it is played
/// offline, its number, what you hold and have, its time, the shops'
/// cards, where things from the generators hang, the camera's lookout, and
/// your body.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Fight<'w, 's> {
    commands: Commands<'w, 's>,
    fighting: ResMut<'w, Fighting>,
    offline: ResMut<'w, OfflineArena>,
    begun: ResMut<'w, Begun>,
    stock: ResMut<'w, Resources>,
    outfit: ResMut<'w, Outfit>,
    clock: ResMut<'w, FightTime>,
    stalls: ResMut<'w, Stalls>,
    pad_top: ResMut<'w, PadTop>,
    lookout: ResMut<'w, Lookout>,
    me: Query<'w, 's, Entity, With<Player>>,
    worn: Query<'w, 's, Entity, With<Wears>>,
    things: Query<'w, 's, Entity, With<InArena>>,
}

impl Fight<'_, '_> {
    /// A game of Last Stand begins on this device, you in `seat`, the
    /// server's if `online`: the fight's state its own and afresh.
    fn take(&mut self, online: bool, seat: usize) -> Stand {
        let fight = self.begun.next();
        let stand = Stand::new(online, seat, fight);
        self.stock.set_if_neq(Resources::default());
        self.outfit.set_if_neq(Outfit::default());
        if self.clock.0 != 0.0 {
            self.clock.0 = 0.0;
        }
        self.stalls.set_if_neq(Stalls::default());
        self.pad_top.set_if_neq(PadTop(rules::PAD_TOP));
        if let Ok(you) = self.me.single() {
            self.commands.entity(you).insert(Team(TEAM)).remove::<(Health, Dead)>();
        }
        stand
    }

    /// The game, numbered `fight`, is over here: the fight's state given
    /// back, once, everything of it taken down, and nothing on your body
    /// or anyone's of it.
    fn give_back(&mut self, fight: u32) {
        self.fighting.set_if_neq(Fighting::none(fight));
        self.offline.set_if_neq(OfflineArena(false));
        self.stock.set_if_neq(Resources::default());
        self.outfit.set_if_neq(Outfit::default());
        if self.clock.0 != 0.0 {
            self.clock.0 = 0.0;
        }
        self.stalls.set_if_neq(Stalls::default());
        self.pad_top.set_if_neq(PadTop::default());
        self.lookout.set_if_neq(Lookout(None));
        for body in &self.worn {
            self.commands.entity(body).remove::<Wears>();
        }
        for thing in &self.things {
            self.commands.entity(thing).try_despawn();
        }
        if let Ok(you) = self.me.single() {
            self.commands.entity(you).remove::<(Health, Dead, Team)>();
        }
        self.commands.remove_resource::<Stand>();
    }
}

/// A table of Last Stand has begun here, or another of anything has, or you
/// have left the tables: a game of it begun afresh on every table of it
/// (`table::Entered`, the same table begun afresh too: a device put back in
/// its game, or a seat moved before play), and given back as it goes.
fn enter(mut entered: MessageReader<table::Entered>, mut gone: MessageReader<table::Gone>, stand: Option<Res<Stand>>, mut fight: Fight) {
    let mut fighting = stand.map(|stand| stand.fight);
    for gone in gone.read() {
        if gone.game == Minigame::LastStand
            && let Some(number) = fighting.take()
        {
            fight.give_back(number);
            info!("Last Stand: away from its table");
        }
    }
    for entered in entered.read() {
        if let Some(number) = fighting.take() {
            fight.give_back(number);
        }
        if entered.game != Minigame::LastStand {
            continue;
        }
        let stand = fight.take(entered.online, usize::from(entered.seat));
        fighting = Some(stand.fight);
        info!(
            "Last Stand: at its table in seat {}{}, the fight numbered {}",
            entered.seat,
            if entered.online { " (online)" } else { "" },
            stand.fight
        );
        fight.commands.insert_resource(stand);
    }
}

/// Says how the fight stands, for the kit (`arena::Fighting`), as it
/// changes: on Last Stand's venue from its waiting room to its end, played
/// from the moment the table's play begins until the game is over, your
/// seat, its number, everyone's id by seat, and everyone on the one team
/// ([`TEAM`]), which is out from the first (there is nothing to hide of
/// it): the chat writes everyone's name in its blue, as the game's own
/// lines, the names over heads and yours top left are. Played offline,
/// said so (`arena::OfflineArena`).
fn claim(stand: Option<Res<Stand>>, view: Res<TableView>, mut fighting: ResMut<Fighting>, mut offline: ResMut<OfflineArena>) {
    let Some(stand) = stand else {
        return;
    };
    let playing = view.playing() && stand.playing();
    let members = view.members.iter().map(|member| member.id);
    let same = fighting.members.len() == view.members.len() && fighting.members.iter().copied().eq(members);
    let one_team = fighting.teams.len() == view.members.len() && fighting.teams.iter().all(|&team| team == TEAM);
    if fighting.playing != playing
        || !fighting.teams_out
        || fighting.seat != Some(stand.seat as u8)
        || fighting.online != stand.online
        || fighting.game != stand.fight
        || fighting.venue != Some(Venue::LastStand)
        || !same
        || !one_team
        || fighting.watched
    {
        *fighting = Fighting {
            playing,
            teams_out: true,
            seat: Some(stand.seat as u8),
            online: stand.online,
            game: stand.fight,
            members: view.members.iter().map(|member| member.id).collect(),
            watched: false,
            venue: Some(Venue::LastStand),
            teams: vec![TEAM; view.members.len()],
        };
    }
    offline.set_if_neq(OfflineArena(!stand.online));
}

/// For trying the game out offline (`RT_STAND_WAVE`, `RT_STAND_PURSE`, as
/// Bridge Wars' `RT_ARENA_FROM` and `RT_ARENA_PURSE`): a whole number from
/// the environment, said in the log as a warning. Never on a phone, which
/// has none.
fn knob(name: &str) -> Option<u16> {
    let value = std::env::var(name).ok()?.trim().parse::<u16>().ok()?;
    warn!("{name}={value}: Last Stand begins so, for trying the game out");
    Some(value)
}

/// Offline, the table's play has begun: the game is made, its dice thrown
/// afresh, everyone at the table in it as the table seats them (you alone,
/// Last Stand having no computer players), and begun by the town's clock.
fn begin_offline(mut stand: Option<ResMut<Stand>>, view: Res<TableView>, town: Res<TownClock>) {
    let Some(stand) = stand.as_deref_mut() else {
        return;
    };
    if stand.online || stand.game.is_some() || !view.playing() {
        return;
    }
    let people: Vec<bool> = view.members.iter().map(|member| member.id != 0 && !member.ai).collect();
    let mut game = Game::new(people.len().max(1), &people, fastrand::u64(..));
    if let Some(wave) = knob("RT_STAND_WAVE").filter(|&wave| wave > 1) {
        game.from_wave(wave);
    }
    if let Some(count) = knob("RT_STAND_PURSE") {
        game.fill_purses(count);
    }
    game.clock(town.now());
    game.begin();
    info!("Last Stand: the game begins offline, {} at the table", people.len());
    stand.round = Some(game.round());
    stand.game = Some(game);
}

/// What you did this frame that the game is to hear of, read alike offline
/// and online.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Doings<'w, 's> {
    landed: MessageReader<'w, 's, Landed>,
    picked: MessageReader<'w, 's, Picked>,
    used: MessageReader<'w, 's, Used>,
    ready: MessageReader<'w, 's, ReadyAsked>,
}

/// What was used off a button, as the rules have it.
fn rules_use(used: Used) -> Option<Use> {
    match used {
        Used::Fireball { heading } => Some(Use::Fireball { heading }),
        Used::Dynamite { heading, far } => Some(Use::Dynamite { heading, far }),
        Used::Potion(potion) => Some(Use::Potion(potion)),
        // No bridges here, nor a builder sold.
        Used::Builder { .. } => None,
    }
}

/// What the kit's looks need for what this device makes of the game's own
/// things offline: the fireballs' and the things' that lie about.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Makers<'w> {
    assets: Res<'w, AssetServer>,
    fire: Res<'w, fire::Look>,
    loot: Res<'w, loot::Look>,
}

/// Offline, a frame of the game: told the town's clock and where you
/// stand, and what you did; stepped; what it says happened carried out;
/// and every zombie's body where the game has it, its health on it, each
/// written only as it changes. At its end, the table is told how it came
/// out, once.
#[allow(clippy::too_many_arguments)]
fn run_offline(
    time: Res<Time>,
    town: Res<TownClock>,
    stand: Option<ResMut<Stand>>,
    mut doings: Doings,
    kit: Makers,
    mut me: Query<(Entity, &Transform, &Intent, Has<Dead>, &mut Shove, &mut PlayerJump), With<Player>>,
    mut zombies: Query<(&mut Transform, &mut Shove, &mut PlayerJump, &mut Shambler, &OwnZombie), Without<Player>>,
    mut happened: MessageWriter<Happened>,
    mut finish: MessageWriter<table::Finish>,
    mut commands: Commands,
) {
    let Some(mut stand) = stand else {
        return;
    };
    if stand.online {
        return;
    }
    let stand = &mut *stand;
    let seat = stand.seat;
    let Some(game) = stand.game.as_mut() else {
        // Not begun: nothing yet to hear of.
        doings.landed.clear();
        doings.picked.clear();
        doings.used.clear();
        doings.ready.clear();
        return;
    };
    let Ok((you, body, intent, dead, mut your_shove, mut your_jump)) = me.single_mut() else {
        return;
    };
    let here = body.translation;
    game.clock(town.now());
    if game.alive(seat) {
        game.sync(seat, here, yaw_of(body.rotation));
    }
    let running = intent.walk.length() >= 0.5;
    for landed in doings.landed.read() {
        if landed.by != you {
            continue;
        }
        let Ok((_, mut shove, mut jump, _, own)) = zombies.get_mut(landed.victim) else {
            continue;
        };
        game.punch(seat, own.0, running);
        // The game knocks it, and says whether the blow counted: the
        // body's own knock and its while reeling are no part of it, which
        // would keep the next blow from landing here at all.
        *shove = Shove::default();
        *jump = PlayerJump::default();
    }
    for picked in doings.picked.read() {
        if picked.venue == Venue::LastStand
            && let Some(ware) = super::screen::ware_of(picked.pick)
        {
            game.buy(seat, ware, here.xz());
        }
    }
    for used in doings.used.read() {
        if let Some(used) = rules_use(*used) {
            game.use_item(seat, used);
        }
    }
    for _ in doings.ready.read() {
        game.ready(seat);
        stand.ready = true;
    }
    // Fallen into the void: dead, the anchor standing or not; once the
    // game is over, back where you began, as in the town.
    if here.y < crate::void_y() && !dead {
        if game.playing() {
            game.fell(seat);
        } else {
            let (at, turn) = rules::arrival(seat);
            stand.put = Some(Transform::from_translation(at).with_rotation(turn));
        }
    }
    let events = game.step(time.delta_secs());
    let now = time.elapsed_secs();
    for event in events {
        match event {
            Event::Risen { zombie, kind, at, yaw } => {
                let Some((health, most)) = game.zombies().find(|(id, ..)| *id == zombie).map(|(.., health, most)| (health, most))
                else {
                    continue;
                };
                let member = Member {
                    id: u64::from(zombie) + 1,
                    name: String::new(),
                    look: ZOMBIE_LOOK.into(),
                    ai: true,
                };
                let place = Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw));
                let body = lobby::spawn_bot_at(&mut commands, &kit.assets, &member, 0, Venue::LastStand, place);
                commands.entity(body).insert((table::InTable, OwnZombie(zombie), Shambler { kind, health, most }));
                stand.zombies.insert(zombie, body);
            }
            Event::Killed { zombie, .. } => {
                if let Some(body) = stand.zombies.remove(&zombie) {
                    commands.entity(body).try_despawn();
                }
            }
            Event::Struck { seat: struck, from, knock, .. } => {
                if struck == seat && knock != Knock::None {
                    your_shove.knock(&mut your_jump, from.xz(), here.xz(), knock == Knock::Heavy);
                }
            }
            Event::Back { seat: back, at, yaw } => {
                if back == seat {
                    stand.put = Some(Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw)));
                }
            }
            Event::Zap { at, .. } => {
                happened.write(Happened::Zap(at));
            }
            Event::Spat { spit, spittle } => {
                let ball = commands.spawn(SpitBall::of(spittle, None)).id();
                stand.spits.insert(spit, ball);
            }
            // Splashed: the ball here splashes as it comes down, which it
            // does in the same frame, flown by the same rules.
            Event::Splat { spit, .. } => {
                stand.spits.remove(&spit);
            }
            Event::Thrown { ball, seat: by, flight } => {
                let thrown = fire::throw(&mut commands, &kit.fire, flight, by, None, Venue::LastStand);
                stand.balls.insert(ball, thrown);
            }
            Event::Burst { ball, at } => {
                if let Some(thrown) = stand.balls.remove(&ball) {
                    commands.entity(thrown).try_despawn();
                }
                fire::burst(&mut commands, at, Venue::LastStand);
            }
            Event::Lobbed { stick, seat: by, lob } => {
                let thrown = dynamite::throw(&mut commands, lob, by, None, Venue::LastStand);
                stand.sticks.insert(stick, thrown);
            }
            Event::Blew { stick, at } => {
                if let Some(thrown) = stand.sticks.remove(&stick) {
                    commands.entity(thrown).try_despawn();
                }
                fire::burst(&mut commands, at + Vec3::Y * 0.1, Venue::LastStand);
            }
            // Gone into the void: the stick falls on there by itself.
            Event::Sank { stick } => {
                stand.sticks.remove(&stick);
            }
            Event::Made(piece) => {
                let thing = loot::lay(&mut commands, &kit.loot, now, Key::Own(piece.id), Loot::of(&piece), Venue::LastStand, rules::PAD_TOP);
                stand.pieces.insert(piece.id, thing);
            }
            Event::Taken { seat: by, piece } => {
                if let Some(thing) = stand.pieces.remove(&piece.id) {
                    loot::take(&mut commands, thing, (by == seat).then_some(you));
                }
            }
            Event::Bought { seat: by, ware, level } => {
                if rules::sold_at(ware) == roundtown_net::bridge_wars::Shop::Upgrades {
                    happened.write(Happened::Upgraded { by, ware, level });
                }
            }
            Event::Hurt { .. } => {}
            Event::Wave { .. } => stand.ready = false,
            Event::Over { waves } => {
                if !stand.finished {
                    stand.finished = true;
                    if let Some((places, scores, total)) = game.outcome() {
                        info!("Last Stand: the last stand fell at wave {}, {waves} cleared, scores {scores:?}", game.wave());
                        finish.write(table::Finish { places, scores, total });
                    }
                }
            }
            Event::Cleared { .. }
            | Event::Died { .. }
            | Event::Out { .. }
            | Event::AnchorHurt { .. }
            | Event::AnchorBroken
            | Event::Drank { .. } => {}
        }
    }
    for (zombie, kind, at, yaw, health, most) in game.zombies() {
        let Some(&body) = stand.zombies.get(&zombie) else {
            continue;
        };
        let Ok((mut transform, _, _, mut shambler, _)) = zombies.get_mut(body) else {
            continue;
        };
        let turn = Quat::from_rotation_y(yaw);
        if transform.translation != at {
            transform.translation = at;
        }
        if transform.rotation != turn {
            transform.rotation = turn;
        }
        shambler.set_if_neq(Shambler { kind, health, most });
    }
    let round = game.round();
    if stand.round.as_ref() != Some(&round) {
        stand.round = Some(round);
    }
    if stand.upgrades != game.upgrades() {
        stand.upgrades = game.upgrades();
    }
    if stand.purses.as_slice() != game.purses() {
        stand.purses = game.purses().to_vec();
    }
    if stand.kits.as_slice() != game.kits() {
        stand.kits = game.kits().to_vec();
    }
    stand.elapsed = game.elapsed();
}

/// Stands you where the game has put you, offline, as it says: beside the
/// anchor again, or, fallen once it is over, where you began.
fn put_back(
    stand: Option<ResMut<Stand>>,
    me: Query<Entity, With<Player>>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
) {
    let Some(at) = stand.and_then(|mut stand| stand.put.take()) else {
        return;
    };
    if let Ok(you) = me.single() {
        lobby::put(&mut bodies, &mut orbit, you, Venue::LastStand, at);
    }
}

/// The turn about the upright of a body turned by `rotation`, in radians:
/// -Z turned by it is ahead, as the game has a yaw.
fn yaw_of(rotation: Quat) -> f32 {
    let ahead = rotation * Vec3::NEG_Z;
    (-ahead.x).atan2(-ahead.z)
}

/// Online, how the game stands, as the server sends it on the table's
/// entity, into what is drawn from, each part only as it changes; the
/// fight's time from when it began by the town's clock, which is the
/// server's; and what the server tells this device of it, for what is
/// drawn and said of that.
fn follow_online(
    stand: Option<ResMut<Stand>>,
    view: Res<TableView>,
    town: Res<TownClock>,
    tables: Query<(Option<&StandRound>, Option<&StandUpgrades>, Option<&Purses>, Option<&Kits>), With<Remote>>,
    mut told: MessageReader<super::Told>,
    mut happened: MessageWriter<Happened>,
) {
    let Some(mut stand) = stand else {
        told.clear();
        return;
    };
    if !stand.online {
        told.clear();
        return;
    }
    if let Some(Ok((round, upgrades, purses, kits))) = view.shown.map(|shown| tables.get(shown)) {
        if let Some(round) = round
            && stand.round.as_ref() != Some(round)
        {
            // A wave come: nobody is ready for the next any more.
            if round.stage == StandStage::Wave && stand.round.as_ref().is_some_and(|was| was.stage != StandStage::Wave) {
                stand.ready = false;
            }
            stand.round = Some(round.clone());
        }
        if let Some(upgrades) = upgrades
            && stand.upgrades != *upgrades
        {
            stand.upgrades = *upgrades;
        }
        if let Some(purses) = purses
            && stand.purses != purses.0
        {
            stand.purses.clone_from(&purses.0);
        }
        if let Some(kits) = kits
            && stand.kits != kits.0
        {
            stand.kits.clone_from(&kits.0);
        }
    }
    let began = stand.round.as_ref().map_or(0.0, |round| round.began);
    if began > 0.0 {
        let elapsed = (town.now() - began).max(0.0) as f32;
        if stand.elapsed != elapsed {
            stand.elapsed = elapsed;
        }
    }
    for super::Told(said) in told.read() {
        match *said {
            rules::Tell::Zap { x, y, z } => {
                happened.write(Happened::Zap(Vec3::new(x, y, z)));
            }
            rules::Tell::Upgraded { by, ware, level } => {
                happened.write(Happened::Upgraded {
                    by: usize::from(by),
                    ware,
                    level,
                });
            }
            rules::Tell::Slain { by, kind } => {
                happened.write(Happened::Slain { by: usize::from(by), kind });
            }
            rules::Tell::Bought(ware) => debug!("Last Stand: bought {ware:?}"),
        }
    }
}

/// Online, what you did, asked of the server: a card pressed in a shop, a
/// thing used off a button, Ready, and a fall into the void, which has you
/// dead here at once (and the server stands you again where it has you). A
/// punch that landed on a zombie is asked by `net` as any punch on someone
/// else's body is (`Ask::Punch`).
fn ask_online(
    stand: Option<Res<Stand>>,
    mut doings: Doings,
    online: Res<Online>,
    me: Query<(Entity, &Transform, Has<Dead>), With<Player>>,
    mut asks: MessageWriter<NetAsk>,
    mut commands: Commands,
) {
    let Some(stand) = stand.filter(|stand| stand.online) else {
        return;
    };
    // (Punches go as `Ask::Punch`, from `net`; these are this game's.)
    doings.landed.clear();
    if !online.is() {
        doings.picked.clear();
        doings.used.clear();
        doings.ready.clear();
        return;
    }
    for picked in doings.picked.read() {
        if picked.venue == Venue::LastStand
            && let Some(ware) = super::screen::ware_of(picked.pick)
        {
            asks.write(NetAsk::LastStand(rules::Ask::Buy(ware)));
        }
    }
    for used in doings.used.read() {
        if let Some(used) = rules_use(*used) {
            asks.write(NetAsk::Use(used));
        }
    }
    for _ in doings.ready.read() {
        asks.write(NetAsk::LastStand(rules::Ask::Ready));
    }
    let Ok((you, body, dead)) = me.single() else {
        return;
    };
    if body.translation.y < crate::void_y() && !dead {
        asks.write(NetAsk::Respawn);
        commands.entity(you).insert(Dead { left: RESPAWN_SECS });
        info!("Last Stand: fallen into the void, the server told");
    }
    let _ = stand;
}

/// Everything about you and the others that the fight's kit draws, from how
/// the game stands, each written only as it changes: what you hold
/// (`arena::Resources`, counted top left) and have (`arena::Outfit`: your
/// buttons, the shops' cards, your pace on a potion), the fight's time; the
/// shops' cards (`market::Stalls`); your health (the bar top left), and
/// whether you are dead (`crate::Dead`, how long for); and on every
/// person's body what they wear (`arena::Wears`: the sword in their hand,
/// the shield round them, whether they are unseen), and on your
/// teammates', online, their health for the bars over their heads, and the
/// team.
#[allow(clippy::too_many_arguments)]
fn publish(
    stand: Option<Res<Stand>>,
    view: Res<TableView>,
    town: Res<TownClock>,
    mut stock: ResMut<Resources>,
    mut outfit: ResMut<Outfit>,
    mut clock: ResMut<FightTime>,
    mut stalls: ResMut<Stalls>,
    mut shelved: Local<Option<(u32, roundtown_net::Purse, roundtown_net::Kit, StandUpgrades)>>,
    mut me: Query<(Entity, &Transform, Option<&mut Health>, Option<&mut Dead>, Option<&Wears>), With<Player>>,
    mut others: Query<(Entity, &Member, Option<&mut Health>, Option<&Team>, Option<&Wears>), (With<Remote>, With<WalkCycle>, Without<Shambler>, Without<Player>)>,
    mut commands: Commands,
) {
    let Some(stand) = stand else {
        *shelved = None;
        return;
    };
    let (purse, kit) = (stand.purse(), stand.kit());
    if stock.held != purse {
        stock.held = purse;
    }
    outfit.set_if_neq(Outfit {
        kit,
        team: Upgrades::default(),
    });
    if clock.0 != stand.elapsed {
        clock.0 = stand.elapsed;
    }
    // The shops' cards, made again only as anything on them changes.
    let shelf = (stand.fight, purse, kit, stand.upgrades);
    if shelved.as_ref() != Some(&shelf) {
        *shelved = Some(shelf);
        stalls.set_if_neq(super::screen::stalls(&purse, &kit, &stand.upgrades));
    }
    let Some(round) = &stand.round else {
        return;
    };
    let wears = |kit: &roundtown_net::Kit| Wears {
        sword: kit.sword,
        shield: kit.shield,
        unseen: kit.unseen(stand.elapsed),
        fast: kit.fast(stand.elapsed),
        leaping: kit.leaping(stand.elapsed),
    };
    let health = |seat: usize| round.health.get(seat).copied().map(f32::from);
    if let Ok((you, body, mine, dead, worn)) = me.single_mut() {
        let now = health(stand.seat).unwrap_or(rules::HEALTH);
        match mine {
            Some(mut mine) if mine.left() != now || mine.full() != rules::HEALTH => *mine = Health::at(now, rules::HEALTH),
            Some(_) => {}
            None => {
                commands.entity(you).insert(Health::at(now, rules::HEALTH));
            }
        }
        // Dead, until the game has you standing again: how long for, by
        // the town's clock; for good with the anchor broken.
        let back = round.back_at.get(stand.seat).copied().unwrap_or(0.0);
        let left = if back.is_finite() { (back - town.now()).max(0.0) as f32 } else { f32::INFINITY };
        let standing = now > 0.0;
        // Fallen into the void online, you are dead here until the server
        // has you standing again where it puts you: not up again the frame
        // before it has heard.
        let fallen = body.translation.y < crate::void_y();
        // (Dead as the game ends too: the last of you to fall, which is
        // what ends it, lies where they fell.)
        match dead {
            Some(_) if standing && !fallen => {
                commands.entity(you).remove::<Dead>();
            }
            Some(mut dead) => {
                if !standing && dead.left != left {
                    dead.left = left;
                }
            }
            None if !standing => {
                commands.entity(you).insert(Dead { left });
            }
            None => {}
        }
        let wearing = wears(&stand.kit());
        if worn != Some(&wearing) {
            commands.entity(you).insert(wearing);
        }
    }
    if !stand.online {
        return;
    }
    // Your teammates, online: their health, team and what they wear, by
    // their seat at the table.
    for (body, member, health_now, team, worn) in &mut others {
        let Some(seat) = view.members.iter().position(|seated| seated.id == member.id && seated.id != 0) else {
            continue;
        };
        if team != Some(&Team(TEAM)) {
            commands.entity(body).insert(Team(TEAM));
        }
        let now = health(seat).unwrap_or(rules::HEALTH);
        match health_now {
            Some(mut health_now) if health_now.left() != now => *health_now = Health::at(now, rules::HEALTH),
            Some(_) => {}
            None => {
                commands.entity(body).insert(Health::at(now, rules::HEALTH));
            }
        }
        if let Some(kit) = stand.kits.get(seat) {
            let wearing = wears(kit);
            if worn != Some(&wearing) {
                commands.entity(body).insert(wearing);
            }
        }
    }
}
