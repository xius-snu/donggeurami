//! Last Stand at its tables, on the server (since 2026-11-18; Hajun: "Coop
//! (2~4 players) Last Stand (Endless Zombie survival game). You are in a
//! circular map, with a spawn anchor crystal in the center, and 2 generators
//! and item shops/upgrade shops"): what a table of it plays
//! (`tables::Rules`), on `laststandmap.glb`, everyone on one side.
//!
//! **The game is the shared rules' own** (`roundtown_net::last_stand::Game`,
//! a plain struct with no world in it): this is the thin wrapper round it
//! that the server runs online, as each device runs one offline. Each frame
//! of play it is told where everyone stands (as their devices last said,
//! [`Table::at`]) and the room's clock, stepped, and what it says happened
//! carried out ([`Event`]): a zombie risen is a body in the table's room, a
//! [`Shambler`] on it saying its kind and its health, moved where the game
//! has it and sent only as it changes; a zombie dead is gone; a spitter's
//! ball, a fireball and a stick of dynamite are entities sent once as they
//! set out and gone as they land, burst or blow (a blast is sent as Bridge
//! Wars' are, [`Blasted`], for every device to draw); what the generators
//! make and the zombies drop lies about as Bridge Wars' things do
//! ([`Loot`]), each gone the moment someone takes it; whoever a brute, a
//! giant or a leaper lands on is knocked by their own device
//! ([`Ctx::knock`]); whoever stands again beside the anchor is stood there
//! ([`Ctx::place`]); and the chat is told of what the team bought for
//! everyone and of a giant brought down. How the game stands goes to the
//! room on the table's entity, each part only as it changes: the round
//! ([`StandRound`]), the team's upgrades ([`StandUpgrades`]), and everyone's
//! purse and kit as Bridge Wars sends them ([`Purses`], [`Kits`]).
//!
//! What each person does comes as Bridge Wars' asks and the game's own: a
//! punch that landed on a zombie's body (`Ask::Punch`, its member's id), a
//! purchase and being ready for the next wave (`Ask::LastStand`), something
//! used off a button (`Ask::Use`) and a fall into the void
//! (`Ask::Respawn`). The game believes none of it that could not be.
//!
//! Whoever is dead with the anchor broken is out of the game
//! ([`Seat::out`](crate::tables::Seat)), free to watch the rest hold on or
//! to go; and once the anchor is broken and nobody stands, the game is over
//! and the table pays every person ten coins a wave the team cleared
//! (`last_stand::prizes`).
//!
//! **For trying it out**, never on a server real players use: a table
//! begins at a later wave with `RT_STAND_WAVE`, and everyone holding that
//! many of each of the four things with `RT_STAND_PURSE`, each said in the
//! log as a warning, as Bridge Wars' `RT_ARENA_FROM` and `RT_ARENA_PURSE`
//! are.

use std::collections::HashMap;

use bevy::prelude::*;
use roundtown_net::last_stand::{
    self, Event, Game, Knock, Shambler, Spit, StandRound, StandUpgrades, sold_at,
};
use roundtown_net::bridge_wars::Shop;
use roundtown_net::{Ask, Blasted, Dynamite, Fireball, Kit, Kits, Loot, Member, Purse, Purses, Spot, Tell, Venue, arrival};

use crate::tables::{Ctx, Rules, Table, facing, yaw_of};

/// How long the room is sent a stick of dynamite's blast for, in seconds,
/// before it is taken away again: long enough for every device to have
/// drawn it, a blast being drawn only as it comes and only if it blew a
/// moment ago (the game's `dynamite::BLEW_LATELY`). Bridge Wars keeps its
/// own to the end of the game, for the planks each took; Last Stand's take
/// none.
const BLAST_KEPT: f32 = 3.0;

/// The name every zombie's body goes by, as the room is sent it: what its
/// kind is called is each device's to say, from its own file, by the kind
/// on its body ([`Shambler`]).
const ZOMBIE_NAME: &str = "Zombie";

/// Who the zombie numbered `zombie` at the table numbered `table` is to
/// everyone (its body's [`Member`]): clear of people's ids, which count up
/// from 1, and of every other game's computer players and zombies, which
/// count down from an eighth of the way down and higher (`tables::bot_id`,
/// and Bridge Wars' and House Builder's): a sixteenth of the way down, less
/// the table's number times 2^32 and the zombie's, which reach no further
/// than 2^56 below it, a table's number being kept under 2^24.
fn zombie_id(table: u32, zombie: u32) -> u64 {
    u64::MAX / 16 - ((u64::from(table.min((1 << 24) - 1)) << 32) | u64::from(zombie))
}

/// A whole number of a test knob from the environment, if it is set: for
/// trying the game out, and said in the log as a warning.
fn knob(name: &str) -> Option<u16> {
    let value = std::env::var(name).ok()?.trim().parse::<u16>().ok()?;
    warn!("{name}={value}: Last Stand's tables begin so, for trying the game out; never on a server real players use");
    Some(value)
}

/// A game of Last Stand at its table: the game itself once it has begun,
/// and what of it is in the world, each thing by the game's own number for
/// it, with what was last sent of it.
pub(crate) struct LastStand {
    seed: u64,
    /// The test knobs, read as the table opens ([`knob`]).
    from_wave: Option<u16>,
    purse: Option<u16>,
    game: Option<Game>,
    /// Each zombie's body, and the kind and health last sent on it.
    zombies: HashMap<u32, (Entity, Shambler)>,
    /// Each zombie's number, by its body's member's id: what a punch on it
    /// is asked by.
    by_member: HashMap<u64, u32>,
    spits: HashMap<u32, Entity>,
    balls: HashMap<u32, Entity>,
    sticks: HashMap<u32, Entity>,
    /// Everything lying about, by its number in the game's pile.
    pieces: HashMap<u32, Entity>,
    /// Each blast sent, and how long it has been.
    blasts: Vec<(Entity, f32)>,
    /// What the room was last sent of how the game stands.
    sent_round: Option<StandRound>,
    sent_upgrades: Option<StandUpgrades>,
    sent_purses: Option<Vec<Purse>>,
    sent_kits: Option<Vec<Kit>>,
}

impl LastStand {
    /// The game at a new table, numbered as `table` is, its dice thrown from
    /// `seed`.
    pub(crate) fn new(_table: &Table, seed: u64) -> Self {
        Self {
            seed,
            from_wave: knob("RT_STAND_WAVE").filter(|&wave| wave > 1),
            purse: knob("RT_STAND_PURSE"),
            game: None,
            zombies: HashMap::new(),
            by_member: HashMap::new(),
            spits: HashMap::new(),
            balls: HashMap::new(),
            sticks: HashMap::new(),
            pieces: HashMap::new(),
            blasts: Vec::new(),
            sent_round: None,
            sent_upgrades: None,
            sent_purses: None,
            sent_kits: None,
        }
    }

    /// Carries out what the game says happened in a frame, in the order it
    /// happened.
    fn carry_out(&mut self, ctx: &mut Ctx, table: &mut Table, events: Vec<Event>) {
        for event in events {
            match event {
                Event::Risen { zombie, kind, at, yaw } => {
                    let Some((health, most)) = self.game.as_ref().and_then(|game| {
                        game.zombies().find(|(id, ..)| *id == zombie).map(|(.., health, most)| (health, most))
                    }) else {
                        continue;
                    };
                    let id = zombie_id(table.id, zombie);
                    let member = Member {
                        id,
                        name: ZOMBIE_NAME.into(),
                        look: roundtown_net::bridge_wars::ZOMBIE_LOOK.into(),
                        ai: true,
                    };
                    let shambler = Shambler { kind, health, most };
                    let body = ctx.spawn_body(table, member, at, yaw);
                    ctx.commands.entity(body).insert(shambler);
                    self.zombies.insert(zombie, (body, shambler));
                    self.by_member.insert(id, zombie);
                }
                Event::Killed { zombie, kind, by, .. } => {
                    if let Some((body, _)) = self.zombies.remove(&zombie) {
                        ctx.commands.entity(body).try_despawn();
                    }
                    self.by_member.remove(&zombie_id(table.id, zombie));
                    // Said in the chat by the one rule both sides go by.
                    if kind.announced()
                        && let Some(by) = by
                    {
                        ctx.tell_room(table, Tell::LastStand(last_stand::Tell::Slain { by: by as u8, kind }));
                    }
                }
                Event::Struck { seat, from, knock, .. } => {
                    if knock != Knock::None {
                        ctx.knock(table, seat, 0, Spot::of(from, Quat::IDENTITY), knock == Knock::Heavy);
                    }
                }
                Event::Back { seat, at, yaw } => ctx.place(table, seat, at, yaw),
                Event::Zap { at, .. } => {
                    ctx.tell_room(table, Tell::LastStand(last_stand::Tell::Zap { x: at.x, y: at.y, z: at.z }));
                }
                Event::Spat { spit, spittle } => {
                    let ball = ctx.spawn_in_room(table, Spit::of(&spittle));
                    self.spits.insert(spit, ball);
                }
                Event::Splat { spit, .. } => {
                    if let Some(ball) = self.spits.remove(&spit) {
                        ctx.commands.entity(ball).try_despawn();
                    }
                }
                Event::Thrown { ball, seat, flight } => {
                    let thrown = ctx.spawn_in_room(
                        table,
                        Fireball {
                            seat: seat as u8,
                            x: flight.from.x,
                            y: flight.from.y,
                            z: flight.from.z,
                            heading: flight.heading,
                        },
                    );
                    self.balls.insert(ball, thrown);
                }
                Event::Burst { ball, .. } => {
                    if let Some(thrown) = self.balls.remove(&ball) {
                        ctx.commands.entity(thrown).try_despawn();
                    }
                }
                Event::Lobbed { stick, seat, lob } => {
                    let thrown = ctx.spawn_in_room(
                        table,
                        Dynamite {
                            seat: seat as u8,
                            x: lob.from.x,
                            y: lob.from.y,
                            z: lob.from.z,
                            to_x: lob.to.x,
                            to_y: lob.to.y,
                            to_z: lob.to.z,
                        },
                    );
                    self.sticks.insert(stick, thrown);
                }
                Event::Blew { stick, at } => {
                    if let Some(thrown) = self.sticks.remove(&stick) {
                        ctx.commands.entity(thrown).try_despawn();
                    }
                    let blast = ctx.spawn_in_room(
                        table,
                        Blasted {
                            x: at.x,
                            y: at.y,
                            z: at.z,
                            planks: Vec::new(),
                            at: ctx.clock,
                            later: Vec::new(),
                        },
                    );
                    self.blasts.push((blast, 0.0));
                }
                // Gone into the void: taken away with no blast, which every
                // device has fall on from where it is.
                Event::Sank { stick } => {
                    if let Some(thrown) = self.sticks.remove(&stick) {
                        ctx.commands.entity(thrown).try_despawn();
                    }
                }
                Event::Made(piece) => {
                    let thing = ctx.spawn_in_room(table, Loot::of(&piece));
                    self.pieces.insert(piece.id, thing);
                }
                Event::Taken { piece, .. } => {
                    if let Some(thing) = self.pieces.remove(&piece.id) {
                        ctx.commands.entity(thing).try_despawn();
                    }
                }
                Event::Bought { seat, ware, level } => {
                    ctx.tell(table, seat, Tell::LastStand(last_stand::Tell::Bought(ware)));
                    if sold_at(ware) == Shop::Upgrades {
                        ctx.tell_room(
                            table,
                            Tell::LastStand(last_stand::Tell::Upgraded {
                                by: seat as u8,
                                ware,
                                level,
                            }),
                        );
                    }
                }
                // What the round and everyone's purse and kit say as they
                // are sent: the waves, the anchor, the dead and the out,
                // what was drunk, and the end.
                Event::Wave { .. }
                | Event::Cleared { .. }
                | Event::Hurt { .. }
                | Event::Died { .. }
                | Event::Out { .. }
                | Event::AnchorHurt { .. }
                | Event::AnchorBroken
                | Event::Drank { .. }
                | Event::Over { .. } => {}
            }
        }
    }

    /// Sends each zombie's body where the game has it, and its health on it
    /// as that changes: each only if anything has.
    fn move_zombies(&mut self, ctx: &mut Ctx) {
        let Some(game) = &self.game else {
            return;
        };
        for (zombie, kind, at, yaw, health, most) in game.zombies() {
            let Some((body, sent)) = self.zombies.get_mut(&zombie) else {
                continue;
            };
            ctx.move_body(*body, at, yaw);
            let now = Shambler { kind, health, most };
            if *sent != now {
                *sent = now;
                ctx.commands.entity(*body).insert(now);
            }
        }
    }

    /// Takes away each blast sent a while ago ([`BLAST_KEPT`]).
    fn clear_blasts(&mut self, ctx: &mut Ctx, dt: f32) {
        self.blasts.retain_mut(|(blast, age)| {
            *age += dt;
            let kept = *age < BLAST_KEPT;
            if !kept {
                ctx.commands.entity(*blast).try_despawn();
            }
            kept
        });
    }
}

impl Rules for LastStand {
    /// On a ring round the middle of the glass room, facing in.
    fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
        let at = last_stand::waiting_spot(seat);
        (at, facing(at.xz(), last_stand::WAITING_ROOM))
    }

    /// The game begins: everyone round the anchor, facing out
    /// (`last_stand::arrival`), and the first wave twenty seconds off.
    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        let people: Vec<bool> = table.seats.iter().map(|seat| seat.person()).collect();
        let mut game = Game::new(table.seats.len(), &people, self.seed);
        if let Some(wave) = self.from_wave {
            game.from_wave(wave);
        }
        if let Some(count) = self.purse {
            game.fill_purses(count);
        }
        game.clock(ctx.clock);
        game.begin();
        for seat in 0..table.seats.len() {
            let (at, yaw) = game.at(seat);
            ctx.place(table, seat, at, yaw);
        }
        self.game = Some(game);
    }

    fn step(&mut self, ctx: &mut Ctx, table: &mut Table, dt: f32) {
        let Some(game) = &mut self.game else {
            return;
        };
        game.clock(ctx.clock);
        for seat in 0..table.seats.len() {
            if table.seats[seat].here() {
                game.sync(seat, table.at[seat], table.yaw[seat]);
            }
        }
        let events = game.step(dt);
        self.carry_out(ctx, table, events);
        self.move_zombies(ctx);
        self.clear_blasts(ctx, dt);
        let Some(game) = &self.game else {
            return;
        };
        // Dead with the anchor broken: out of it, free to watch or to go.
        let round = game.round();
        for (seat, it) in table.seats.iter_mut().enumerate() {
            if it.person() && !it.out && round.back_at.get(seat).is_some_and(|back| back.is_infinite()) && game.anchor().0 <= 0.0 {
                it.out = true;
            }
        }
        if let Some((places, scores, total)) = game.outcome() {
            info!(
                "Last Stand table {}: the last stand fell at wave {}, {} cleared, the team's score {}",
                table.id,
                game.wave(),
                game.cleared(),
                game.team_score()
            );
            table.finish(places, scores, total);
        }
    }

    fn hear(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize, ask: &Ask) {
        let Some(game) = &mut self.game else {
            // Before the game: a fall out of the glass room (which there
            // should be no way out of) is back where the seat waits.
            if let Ask::Respawn = ask {
                let (at, yaw) = self.waiting_spot(table, seat);
                ctx.place(table, seat, at, yaw);
            }
            return;
        };
        match ask {
            Ask::Punch { who, running } => {
                if let Some(&zombie) = self.by_member.get(who) {
                    game.punch(seat, zombie, *running);
                }
            }
            Ask::LastStand(last_stand::Ask::Buy(ware)) => {
                game.buy(seat, *ware, table.at[seat].xz());
            }
            Ask::LastStand(last_stand::Ask::Ready) => game.ready(seat),
            Ask::Use(used) => {
                game.use_item(seat, *used);
            }
            Ask::Respawn => {
                if game.alive(seat) {
                    game.fell(seat);
                } else if !game.playing() {
                    // Fallen with the game over: where the seat began.
                    let (at, turn) = arrival(Venue::LastStand, seat);
                    ctx.place(table, seat, at, yaw_of(turn));
                }
            }
            // For trying the game out (`RT_ALLOW_SKIP`): a break is over in
            // a moment, as if everyone standing were ready.
            Ask::Skip => {
                for one in 0..table.seats.len() {
                    game.ready(one);
                }
            }
            _ => {}
        }
    }

    /// Whoever has left is no one the zombies go for, and stands again
    /// nowhere; their score stays theirs.
    fn left(&mut self, _ctx: &mut Ctx, _table: &mut Table, seat: usize, _body: Option<Entity>) {
        if let Some(game) = &mut self.game {
            game.leave(seat);
        }
    }

    /// How the game stands, on the table's entity, each part only as it
    /// changes.
    fn show(&mut self, commands: &mut Commands, table: &Table) {
        let Some(game) = &self.game else {
            return;
        };
        let round = game.round();
        if self.sent_round.as_ref() != Some(&round) {
            commands.entity(table.shown).insert(round.clone());
            self.sent_round = Some(round);
        }
        let upgrades = game.upgrades();
        if self.sent_upgrades != Some(upgrades) {
            commands.entity(table.shown).insert(upgrades);
            self.sent_upgrades = Some(upgrades);
        }
        if self.sent_purses.as_deref() != Some(game.purses()) {
            commands.entity(table.shown).insert(Purses(game.purses().to_vec()));
            self.sent_purses = Some(game.purses().to_vec());
        }
        if self.sent_kits.as_deref() != Some(game.kits()) {
            commands.entity(table.shown).insert(Kits(game.kits().to_vec()));
            self.sent_kits = Some(game.kits().to_vec());
        }
    }

    /// Everything the game put in the room goes with the table: the
    /// zombies, the balls and sticks in the air, the blasts, and whatever
    /// lies about. (It put nothing on anyone's body.)
    fn close(&mut self, commands: &mut Commands, _table: &Table) {
        let things = self
            .zombies
            .drain()
            .map(|(_, (body, _))| body)
            .chain(self.spits.drain().map(|(_, ball)| ball))
            .chain(self.balls.drain().map(|(_, ball)| ball))
            .chain(self.sticks.drain().map(|(_, stick)| stick))
            .chain(self.pieces.drain().map(|(_, thing)| thing))
            .chain(self.blasts.drain(..).map(|(blast, _)| blast));
        for thing in things {
            commands.entity(thing).try_despawn();
        }
        self.by_member.clear();
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use bevy::ecs::system::{RunSystemOnce, SystemParam};
    use bevy_replicon::prelude::*;
    use roundtown_net::Minigame;
    use roundtown_net::last_stand::{ITEM_SHOP, StandStage, UPGRADE_SHOP, Ware};
    use roundtown_net::{Item, Utility};

    use super::*;
    use crate::Config;
    use crate::players::{Account, Accounts, Body, Who};
    use crate::rooms::{InRoom, Room};
    use crate::tables::{Bodies, Stage};

    /// A table of Last Stand being played, its game beside it, on a world
    /// of the server's own with no network.
    #[derive(Resource)]
    struct Played {
        table: Table,
        game: LastStand,
    }

    /// The server with `people` people at a table of Last Stand, the game
    /// begun, each person's body in the table's room on its venue.
    fn played(people: usize, seed: u64) -> World {
        let mut world = World::new();
        world.insert_resource(Config {
            port: 0,
            key: None,
            public: Vec::new(),
            db: None,
            allow_skip: true,
            arena_from: 0.0,
            arena_purse: 0,
            arena_old_bots: false,
        });
        world.init_resource::<Messages<ToClients<Tell>>>();
        let room = Room::Table(Minigame::LastStand, 0);
        let shown = world.spawn((Replicated, InRoom(room))).id();
        let mut table = Table::new(Minigame::LastStand, 0, shown, seed).expect("a table of Last Stand");
        table.seats.truncate(people);
        table.at.truncate(people);
        table.yaw.truncate(people);
        table.size = people;
        for seat in 0..people {
            let member = Member {
                id: seat as u64 + 1,
                name: format!("Person {}", seat + 1),
                look: "boy".into(),
                ai: false,
            };
            let client = world.spawn_empty().id();
            let (at, turn) = arrival(Venue::LastStand, seat);
            let body = world
                .spawn((
                    Replicated,
                    member.clone(),
                    Account(member.clone()),
                    Spot::of(at, turn),
                    Venue::LastStand,
                    InRoom(room),
                    Body::person(client, Who(member.id), room, seat as u8, None, 1000, 0.0),
                ))
                .id();
            let it = &mut table.seats[seat];
            it.body = body;
            it.member = member;
            it.vacant = false;
            it.gone = false;
        }
        table.stage = Stage::Playing { elapsed: 0.0 };
        let game = LastStand::new(&table, seed);
        world.insert_resource(Played { table, game });
        act(&mut world, |ctx, table, game| game.begin(ctx, table));
        world
    }

    /// What a game is handed to act on the world with, as the tables hand
    /// it ([`Ctx`]), in one system parameter.
    #[derive(SystemParam)]
    struct Hands<'w, 's> {
        commands: Commands<'w, 's>,
        tells: MessageWriter<'w, ToClients<Tell>>,
        bodies: Bodies<'w, 's>,
        accounts: Accounts<'w, 's>,
    }

    /// Has the game at the table do something with what a game is handed
    /// ([`Ctx`]), the room's clock `clock`.
    fn act_at(world: &mut World, clock: f64, deed: impl FnOnce(&mut Ctx, &mut Table, &mut LastStand) + Send + Sync + 'static) {
        let mut deed = Some(deed);
        world
            .run_system_once(
                move |mut hands: Hands, config: Res<Config>, mut played: ResMut<Played>| {
                    let Hands {
                        commands,
                        tells,
                        bodies,
                        accounts,
                    } = &mut hands;
                    let mut ctx = Ctx {
                        commands,
                        tells,
                        bodies,
                        accounts,
                        now: clock,
                        clock,
                        config: &config,
                    };
                    let Played { table, game } = &mut *played;
                    if let Some(deed) = deed.take() {
                        deed(&mut ctx, table, game);
                    }
                },
            )
            .expect("the deed");
    }

    fn act(world: &mut World, deed: impl FnOnce(&mut Ctx, &mut Table, &mut LastStand) + Send + Sync + 'static) {
        act_at(world, 1.8e9, deed);
    }

    /// A frame of play, `dt` seconds on, at the room's clock `clock`: where
    /// everyone stands copied in, as the tables do, the game stepped and
    /// shown, and everything told to devices kept.
    fn frame(world: &mut World, dt: f32, clock: f64, told: &mut Vec<Tell>) {
        let at: Vec<(Vec3, f32)> = {
            let played = world.resource::<Played>();
            played
                .table
                .seats
                .iter()
                .map(|it| world.get::<Spot>(it.body).map_or((Vec3::ZERO, 0.0), |spot| (spot.translation(), spot.yaw())))
                .collect()
        };
        {
            let mut played = world.resource_mut::<Played>();
            for (seat, (spot, yaw)) in at.into_iter().enumerate() {
                played.table.at[seat] = spot;
                played.table.yaw[seat] = yaw;
            }
            if let Stage::Playing { elapsed } = played.table.stage {
                played.table.stage = Stage::Playing { elapsed: elapsed + dt };
            }
        }
        act_at(world, clock, move |ctx, table, game| {
            if table.playing() {
                game.step(ctx, table, dt);
            }
            game.show(ctx.commands, table);
        });
        told.extend(world.resource_mut::<Messages<ToClients<Tell>>>().drain().map(|told| told.message));
    }

    /// Stands the body in `seat` at `at`, as its device would say.
    fn stand(world: &mut World, seat: usize, at: Vec3) {
        let body = world.resource::<Played>().table.seats[seat].body;
        world.entity_mut(body).insert(Spot::of(at, Quat::IDENTITY));
    }

    /// The zombies' bodies the room is sent: each with its member's id, its
    /// kind and health, and where it is sent standing.
    fn zombies(world: &mut World) -> Vec<(u64, Shambler, Vec3)> {
        let mut query = world.query::<(&Member, &Shambler, &Spot, &InRoom, &Venue)>();
        query
            .iter(world)
            .inspect(|(.., room, venue)| {
                assert_eq!(room.0, Room::Table(Minigame::LastStand, 0));
                assert_eq!(**venue, Venue::LastStand);
            })
            .map(|(member, shambler, spot, ..)| (member.id, *shambler, spot.translation()))
            .collect()
    }

    #[test]
    fn a_table_of_last_stand_is_played_through_from_the_first_wave_to_the_last_stand_s_fall() {
        let mut world = played(2, 7);
        let shown = world.resource::<Played>().table.shown;
        // Begun: both stood round the anchor where the game has them,
        // and the round sent, the first wave twenty seconds off.
        let mut told = Vec::new();
        frame(&mut world, 0.05, 1.8e9, &mut told);
        let round = world.get::<StandRound>(shown).expect("the round").clone();
        assert_eq!((round.wave, round.began), (0, 1.8e9));
        assert_eq!(round.stage, StandStage::Break { until: 1.8e9 + 20.0 });
        assert!(world.get::<StandUpgrades>(shown).is_some() && world.get::<Purses>(shown).is_some() && world.get::<Kits>(shown).is_some());
        // Person 1 stands by the anchor and swings at whatever zombie is
        // in reach a couple of times a second, with a sword bought at the
        // first chance; person 2 does nothing.
        let mut clock = 1.8e9;
        let mut seen_zombies = 0;
        let mut ever_dropped = false;
        let mut bought = false;
        let mut last_swing = 0.0;
        let mut kills_seen = 0;
        let mut max_wave = 0;
        let dt = 0.05;
        let mut elapsed = 0.0;
        while world.resource::<Played>().table.playing() {
            assert!(elapsed < 3600.0, "a game that never ends");
            elapsed += dt;
            clock += f64::from(dt);
            // On a generator's pad until there is silver enough for a
            // sword; by the anchor after.
            let here = if bought {
                last_stand::arrival(0).0
            } else {
                Vec3::new(last_stand::PADS[0].x, last_stand::PAD_TOP, last_stand::PADS[0].y)
            };
            stand(&mut world, 0, here);
            frame(&mut world, dt, clock, &mut told);
            let now = zombies(&mut world);
            seen_zombies = seen_zombies.max(now.len());
            let round = world.get::<StandRound>(shown).expect("the round").clone();
            max_wave = max_wave.max(round.wave);
            // Swing at the nearest in reach.
            if elapsed - last_swing >= 0.5
                && let Some((who, ..)) = now
                    .iter()
                    .filter(|(_, _, at)| at.xz().distance(here.xz()) < 2.5)
                    .min_by(|a, b| a.2.distance(here).total_cmp(&b.2.distance(here)))
            {
                last_swing = elapsed;
                let who = *who;
                act_at(&mut world, clock, move |ctx, table, game| {
                    game.hear(ctx, table, 0, &Ask::Punch { who, running: false });
                });
            }
            // A sword, as soon as it can be had, from the stand that sells it.
            if !bought && world.get::<Purses>(shown).is_some_and(|purses| purses.0[0][Item::Silver.index()] >= 12) {
                let at = Vec3::new(ITEM_SHOP.x, last_stand::STONE_TOP, ITEM_SHOP.y);
                stand(&mut world, 0, at);
                frame(&mut world, dt, clock, &mut told);
                act_at(&mut world, clock, |ctx, table, game| {
                    game.hear(ctx, table, 0, &Ask::LastStand(last_stand::Ask::Buy(Ware::Sword(1))));
                });
                frame(&mut world, dt, clock, &mut told);
                bought = world.get::<Kits>(shown).is_some_and(|kits| kits.0[0].sword == 1);
                assert!(bought, "the sword bought at its stand");
            }
            ever_dropped |= world.query::<&Loot>().iter(&world).any(|loot| loot.spilt.is_some());
            kills_seen = kills_seen.max(world.get::<StandRound>(shown).map_or(0, |round| round.scores[0]));
        }
        // Over: the anchor broken and nobody standing, the results the
        // game's, the table paid by the waves cleared.
        let table = &world.resource::<Played>().table;
        let results = table.results.clone().expect("how it came out");
        let game = world.resource::<Played>().game.game.as_ref().expect("the game");
        assert!(game.over());
        assert_eq!(game.anchor().0, 0.0);
        assert_eq!(results.total, Some(u32::from(game.cleared())));
        assert_eq!(results.scores, game.scores());
        assert!(results.scores[0] > results.scores[1], "{:?}: only one swung", results.scores);
        assert_eq!(results.places, vec![0, 1]);
        assert_eq!(results.prizes, last_stand::prizes(&results, &[true, true]));
        assert!(table.seats.iter().all(|seat| seat.out), "both out, dead with the anchor broken");
        assert!(seen_zombies > 5 && max_wave >= 1 && kills_seen > 0 && bought, "{seen_zombies} {max_wave} {kills_seen}");
        assert!(ever_dropped, "a zombie killed drops something now and then");
        // Things lay on the pads, and the room was told of the knocks the
        // big ones dealt, if any came, and of the anchor's beam, if bought.
        assert!(world.query::<&Loot>().iter(&world).any(|loot| loot.spilt.is_none()), "what the generators made");
        // Every zombie's id is the table's, for a punch on it to find it.
        // Closed: everything it put in the room goes.
        act(&mut world, |ctx, table, game| game.close(ctx.commands, table));
        assert_eq!(world.query::<&Shambler>().iter(&world).count(), 0);
        assert_eq!(world.query::<&Loot>().iter(&world).count(), 0);
        assert_eq!(world.query::<&Spit>().iter(&world).count(), 0);
        assert_eq!(world.query::<&Fireball>().iter(&world).count(), 0);
        assert_eq!(world.query::<&Dynamite>().iter(&world).count(), 0);
        assert_eq!(world.query::<&Blasted>().iter(&world).count(), 0);
    }

    #[test]
    fn what_a_person_asks_is_carried_out_and_told() {
        let mut world = played(1, 3);
        let shown = world.resource::<Played>().table.shown;
        let mut told = Vec::new();
        let mut clock = 1.8e9;
        // Everything held to buy with, at the upgrade shop's stand.
        world.resource_mut::<Played>().game.game.as_mut().expect("begun").fill_purses(200);
        stand(&mut world, 0, Vec3::new(UPGRADE_SHOP.x, last_stand::STONE_TOP, UPGRADE_SHOP.y));
        clock += 0.05;
        frame(&mut world, 0.05, clock, &mut told);
        act_at(&mut world, clock, |ctx, table, game| {
            game.hear(ctx, table, 0, &Ask::LastStand(last_stand::Ask::Buy(Ware::Sharpness)));
            // Not here: the item shop is across the anchor.
            game.hear(ctx, table, 0, &Ask::LastStand(last_stand::Ask::Buy(Ware::Sword(3))));
        });
        clock += 0.05;
        frame(&mut world, 0.05, clock, &mut told);
        assert_eq!(world.get::<StandUpgrades>(shown).map(|upgrades| upgrades.sharpness), Some(1));
        assert_eq!(world.get::<Kits>(shown).map(|kits| kits.0[0].sword), Some(0));
        assert!(told.iter().any(|tell| matches!(tell, Tell::LastStand(last_stand::Tell::Bought(Ware::Sharpness)))));
        assert!(told.iter().any(|tell| matches!(
            tell,
            Tell::LastStand(last_stand::Tell::Upgraded { by: 0, ware: Ware::Sharpness, level: 1 })
        )));
        // A fireball bought at the item shop and thrown: an entity for the
        // room as it sets out, gone as it bursts.
        stand(&mut world, 0, Vec3::new(ITEM_SHOP.x, last_stand::STONE_TOP, ITEM_SHOP.y));
        clock += 0.05;
        frame(&mut world, 0.05, clock, &mut told);
        act_at(&mut world, clock, |ctx, table, game| {
            game.hear(ctx, table, 0, &Ask::LastStand(last_stand::Ask::Buy(Ware::Utility(Utility::Fireball))));
        });
        clock += 0.05;
        frame(&mut world, 0.05, clock, &mut told);
        act_at(&mut world, clock, |ctx, table, game| {
            game.hear(ctx, table, 0, &Ask::Use(roundtown_net::Use::Fireball { heading: 0.0 }));
        });
        clock += 0.05;
        frame(&mut world, 0.05, clock, &mut told);
        assert_eq!(world.query::<&Fireball>().iter(&world).count(), 1);
        for _ in 0..80 {
            clock += 0.05;
            frame(&mut world, 0.05, clock, &mut told);
        }
        assert_eq!(world.query::<&Fireball>().iter(&world).count(), 0, "burst at the end of its range");
        // Ready for the first wave: it comes in three seconds, not twenty.
        act_at(&mut world, clock, |ctx, table, game| game.hear(ctx, table, 0, &Ask::LastStand(last_stand::Ask::Ready)));
        let mut waited = 0.0;
        while world.get::<StandRound>(shown).is_some_and(|round| round.wave == 0) {
            clock += 0.05;
            waited += 0.05;
            frame(&mut world, 0.05, clock, &mut told);
            assert!(waited < 5.0, "the wave did not come");
        }
        // The zombies' ids are clear of everyone else's.
        assert!(zombie_id(0, 0) < crate::tables::bot_id(Minigame::LastStand, u32::MAX >> 8, 63));
        assert!(zombie_id(u32::MAX, u32::MAX) > 1 << 59);
        assert_ne!(zombie_id(0, 1), zombie_id(1, 0));
        // A fall into the void is a death, and the dead stand again by the
        // anchor six seconds later, stood there by the server.
        stand(&mut world, 0, Vec3::new(0.0, -40.0, 0.0));
        act_at(&mut world, clock, |ctx, table, game| game.hear(ctx, table, 0, &Ask::Respawn));
        told.clear();
        for _ in 0..(7.0 / 0.05) as usize {
            clock += 0.05;
            frame(&mut world, 0.05, clock, &mut told);
        }
        assert!(told.iter().any(|tell| matches!(tell, Tell::Arrive { venue: Venue::LastStand, seat: 0, .. })), "{told:?}");
        let spot = world.get::<Spot>(world.resource::<Played>().table.seats[0].body).expect("a body").translation();
        assert!((spot - last_stand::arrival(0).0).length() < 0.05, "{spot}");
    }
}
