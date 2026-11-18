//! Bridge Race at its tables, on the server (since 2026-11-18; Hajun: "Like
//! King of the Ladder but with bridge building ... Your goal is to reach the
//! golden island first compared to the other team - both of u need to make
//! it across cuz its 2v2"): what a table of it plays (`tables::Rules`), on
//! `bridgeracemap.glb`, two teams by seat (`roundtown_net::bridge_race::team`).
//!
//! Its seats are taken two by two onto a team ([`BridgeRace::seats_for`]),
//! so that people are teamed with each other first and a group goes in
//! together. Everyone waits in the glass room over the map, and as the
//! countdown ends the game itself begins: the rules' own [`Game`]
//! (`roundtown_net::bridge_race`), the very one a device plays offline,
//! which this module feeds and carries out, as `arena` does Bridge Wars':
//!
//! * **In**: where each person stands, as their device says (the table
//!   copies it in each frame), and what each asks: a rung laid
//!   (`Ask::Rung`), a punch landed (`Ask::Punch`), a stick of dynamite
//!   thrown (`Ask::Use`), a fall into the void (`Ask::Respawn`), the metres
//!   a device has left (`Ask::BridgeRace`). Someone whose device has gone
//!   stands where they stood, counts for nothing on the golden island, and
//!   falls if what was under them goes (`Game::away`).
//! * **Out**, to the table's room: every rung as a [`Plank`] and every stick
//!   as a [`Dynamite`], each a replicated entity of the room (`InRoom`); a
//!   stick that blew as a [`Blasted`], with the rest of the bridge to fall
//!   after it, which and when (since the evening of 2026-11-18, as Bridge
//!   Wars'; nothing after it until then); the computer's players where the
//!   game moves them (`Table::at`, which the table sends); the round
//!   ([`RaceRound`]: the clock, the grants, the golden island's ring, who
//!   won) and everyone's sticks ([`Kits`]) on the table's own entity, each
//!   sent again only as it changes; a knock to whoever it landed on
//!   (`Tell::Knocked`), and whoever stands again stood on their island
//!   (`Tell::Arrive`); the chat's lines to everyone connected
//!   (`bridge_race::Tell::Feed`); and once it is over, how everyone came
//!   (`Table::finish`, which pays): the winning team first, every score the
//!   metres of bridge each laid.

use std::collections::HashMap;

use bevy::prelude::*;
use roundtown_net::bridge_race::{self as rules, Event, Game, RaceRound};
use roundtown_net::{Ask, Blasted, Kit, Kits, Spot, Tell, Use, Venue, arrival};

use crate::players::tell;
use crate::tables::{Ctx, Rules, Table, facing, yaw_of};

/// A game of Bridge Race at its table: before it begins, its dice; and from
/// then on the game itself, with what the room has been sent of it.
pub(crate) struct BridgeRace {
    /// What the game's dice are thrown from (the table's own seed).
    seed: u64,
    /// The game, from the countdown's end.
    game: Option<Game>,
    /// Each stick of dynamite in the air or lying, by the game's number for
    /// it, as the entity the room is sent: gone as it blows or falls.
    sticks: HashMap<u32, Entity>,
    /// Every rung and every blast the room has been sent, kept to the end
    /// (which planks there are and which have gone is part of every bridge
    /// from then on), and taken away with the table.
    kept: Vec<Entity>,
    /// The round and the kits as the room was last sent them.
    sent_round: Option<RaceRound>,
    sent_kits: Option<Vec<Kit>>,
}

impl BridgeRace {
    /// The game at a new table, numbered as `table` is, its dice thrown from
    /// `seed`.
    pub(crate) fn new(_table: &Table, seed: u64) -> Self {
        Self {
            seed,
            game: None,
            sticks: HashMap::new(),
            kept: Vec::new(),
            sent_round: None,
            sent_kits: None,
        }
    }

    /// Carries out what has happened in the game ([`Event`]), for the room
    /// to see and for whoever it happened to.
    fn carry_out(&mut self, ctx: &mut Ctx, table: &mut Table, event: Event) {
        if let Some(said) = rules::feed(&event) {
            say(ctx, table, said);
        }
        match event {
            Event::Plank(plank) => {
                let entity = ctx.spawn_in_room(table, plank);
                self.kept.push(entity);
            }
            // A person's own device knocks their body; the computer's the
            // game has knocked already.
            Event::Knocked { victim, by, from, running } => {
                if table.seats.get(victim).is_some_and(|it| it.here()) {
                    let by = table.seats.get(by).map_or(0, |it| it.member.id);
                    ctx.knock(table, victim, by, Spot::of(from, Quat::IDENTITY), running);
                }
            }
            Event::Thrown { stick, shown } => {
                let entity = ctx.spawn_in_room(table, shown);
                self.sticks.insert(stick, entity);
            }
            Event::Lost { stick, .. } => {
                if let Some(entity) = self.sticks.remove(&stick) {
                    ctx.commands.entity(entity).try_despawn();
                }
            }
            Event::Blew { stick, at, planks, later, .. } => {
                if let Some(entity) = self.sticks.remove(&stick) {
                    ctx.commands.entity(entity).try_despawn();
                }
                // With the rest of the bridge to fall after it, which and
                // when, as Bridge Wars' (since the evening of 2026-11-18):
                // every device takes each at its time.
                let blasted = Blasted {
                    x: at.x,
                    y: at.y,
                    z: at.z,
                    planks,
                    at: ctx.clock,
                    later: Blasted::later_of(&later),
                };
                let entity = ctx.spawn_in_room(table, blasted);
                self.kept.push(entity);
            }
            // A person stands again where the game has them, as a fresh
            // arrival there; the computer's is sent there with the rest of
            // its moves.
            Event::Back { seat, at, yaw } => {
                if table.seats.get(seat).is_some_and(|it| it.here()) {
                    ctx.place(table, seat, at, yaw);
                }
            }
            Event::Over { winner, finish } => {
                info!("Bridge Race table {}: over, {winner:?} winning by {finish:?}", table.id);
                if let Some((places, scores, total)) = self.game.as_ref().and_then(Game::outcome) {
                    table.finish(places, scores, total);
                }
            }
            Event::Granted { .. } | Event::Fell { .. } | Event::Holding { .. } => {}
        }
    }
}

/// Says `said` in the chat of everyone at `table` whose device is connected
/// and in its room: told straight to each device, never through
/// `Ctx::tell`, whose word to a device that has gone has it put back in the
/// game afresh when it is back, which a line of the chat is no reason for.
fn say(ctx: &mut Ctx, table: &Table, said: rules::Feed) {
    for it in &table.seats {
        if !it.here() || it.away {
            continue;
        }
        if let Ok((body, ..)) = ctx.bodies.get(it.body)
            && body.room == table.room
            && let Some(client) = body.client
        {
            tell(ctx.tells, client, Tell::BridgeRace(rules::Tell::Feed(said)));
        }
    }
}

impl Rules for BridgeRace {
    /// On a ring round the middle of the glass room, facing in.
    fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
        let at = rules::waiting_spot(seat);
        (at, facing(at.xz(), rules::WAITING_ROOM))
    }

    /// Two by two onto a team, people with people first (Hajun: "if its 2
    /// players then its a 2v2 against bots"): each pair of a group onto a
    /// team with both its seats free, red first, and an odd one into the
    /// first seat free, in the order people take seats
    /// (`bridge_race::PEOPLE_FIRST`). One alone takes red's first seat,
    /// two red's both, three red's both and blue's first; and a group of
    /// two after someone alone is blue together. As Bridge Wars seats a
    /// group (`arena::Game::seats_for`).
    fn seats_for(&self, table: &Table, count: usize) -> Option<Vec<usize>> {
        let order = table.order();
        let mut seats = Vec::with_capacity(count);
        let mut teams = order.chunks(2);
        for _ in 0..count / 2 {
            let pair = teams.by_ref().find(|pair| pair.len() == 2 && pair.iter().all(|&seat| table.open(seat)))?;
            seats.extend_from_slice(pair);
        }
        if count % 2 == 1 {
            seats.push(order.iter().copied().find(|&seat| table.open(seat) && !seats.contains(&seat))?);
        }
        Some(seats)
    }

    /// The game begins: everyone on their team's island, where their seat
    /// arrives, facing the golden island, a person as a fresh arrival
    /// there; and the clock on the board.
    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        let people: Vec<bool> = table.seats.iter().map(|it| it.person()).collect();
        let mut game = Game::new(table.seats.len(), &people, self.seed);
        game.begin();
        for seat in 0..table.seats.len() {
            let (at, turn) = arrival(Venue::BridgeRace, seat);
            ctx.place(table, seat, at, yaw_of(turn));
        }
        table.play_left = Some(rules::PLAY_SECS);
        self.game = Some(game);
    }

    /// A frame of the game: everyone's feet into it, a step of it, and
    /// everything that came of it carried out; the computer's players where
    /// it has them, and the clock on the board.
    fn step(&mut self, ctx: &mut Ctx, table: &mut Table, dt: f32) {
        let Some(game) = self.game.as_mut() else {
            return;
        };
        for seat in 0..table.seats.len() {
            let it = &table.seats[seat];
            if !it.here() {
                continue;
            }
            game.sync(seat, table.at[seat], table.yaw[seat]);
            // Someone whose device has gone stands where they stood, nothing
            // to knock them (a knock is their own device's to carry out):
            // they count for nothing on the golden island, and fall as their
            // device would have said, the game's own rule (`Game::away`).
            game.away(seat, it.away);
        }
        let events = game.step(dt);
        let left = (rules::PLAY_SECS - game.elapsed()).max(0.0);
        for seat in 0..table.seats.len() {
            if table.seats[seat].computer {
                let (at, yaw) = game.at(seat);
                table.at[seat] = at;
                table.yaw[seat] = yaw;
            }
        }
        if !table.over() {
            table.play_left = Some(left);
        }
        for event in events {
            self.carry_out(ctx, table, event);
        }
    }

    /// What a person at the table asks of the game, each to the rules' own
    /// way of hearing it, which turns away whatever could not be so.
    fn hear(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize, ask: &Ask) {
        // Fallen into the void before the game or after it: back where the
        // seat waits, or where it begins, as a fresh arrival there.
        if let Ask::Respawn = ask
            && !self.game.as_ref().is_some_and(Game::playing)
        {
            let (at, yaw) = if table.playing() || table.over() {
                rules::home(seat)
            } else {
                self.waiting_spot(table, seat)
            };
            ctx.place(table, seat, at, yaw);
            return;
        }
        let Some(game) = self.game.as_mut() else {
            return;
        };
        match ask {
            Ask::Respawn => game.fell(seat),
            Ask::Punch { who, running } => {
                if let Some(victim) = table.seats.iter().position(|it| !it.vacant && it.member.id == *who) {
                    game.punch(seat, victim, *running);
                }
            }
            Ask::Rung { chain, index, x, z, heading } => {
                game.rung(seat, *chain, *index, Vec2::new(*x, *z), *heading);
            }
            Ask::Use(Use::Dynamite { heading, far }) => {
                game.throw(seat, *heading, *far);
            }
            Ask::BridgeRace(rules::Ask::Metres { metres, grants }) => game.said(seat, *metres, *grants),
            // For trying the game out (`RT_ALLOW_SKIP`): the time is up now.
            Ask::Skip => game.call_time(),
            _ => {}
        }
    }

    /// Someone has left for good: no longer anyone the golden island waits
    /// for, and their team's grants go to whoever of it is left. Nothing was
    /// put on their body.
    fn left(&mut self, _ctx: &mut Ctx, _table: &mut Table, seat: usize, _body: Option<Entity>) {
        if let Some(game) = self.game.as_mut() {
            game.leave(seat);
        }
    }

    /// A device back in the game afresh has forgotten its metres of bridge:
    /// it is told them as it last said, with every grant since.
    fn back(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let (metres, grants) = game.kept(seat);
        ctx.tell(table, seat, Tell::BridgeRace(rules::Tell::Metres { metres, grants }));
    }

    /// The round and everyone's sticks on the table's entity, each only as
    /// it changes.
    fn show(&mut self, commands: &mut Commands, table: &Table) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let round = game.round();
        if self.sent_round.as_ref() != Some(&round) {
            commands.entity(table.shown).insert(round.clone());
            self.sent_round = Some(round);
        }
        if self.sent_kits.as_deref() != Some(game.kits()) {
            commands.entity(table.shown).insert(Kits(game.kits().to_vec()));
            self.sent_kits = Some(game.kits().to_vec());
        }
    }

    /// Every rung, blast and stick the room was sent goes with the table (its
    /// entity, with the round and the kits on it, is the table's to take).
    fn close(&mut self, commands: &mut Commands, _table: &Table) {
        for entity in self.kept.drain(..).chain(self.sticks.drain().map(|(_, entity)| entity)) {
            commands.entity(entity).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use bevy::ecs::system::{RunSystemOnce, SystemParam};
    use bevy_replicon::prelude::*;
    use roundtown_net::{Dynamite, Member, Minigame, Plank};

    use super::*;
    use crate::Config;
    use crate::players::{Account, Accounts, Body, Who};
    use crate::rooms::{InRoom, Room};
    use crate::tables::{Bodies, Stage, bot_id};

    /// One frame of a game here, in seconds: the server's.
    const FRAME: f32 = 1.0 / 30.0;

    /// What the game is handed to act on the world with, as the tables'
    /// own systems have it (`tables::Hands`), their lifetimes one.
    #[derive(SystemParam)]
    struct Hands<'w, 's> {
        commands: Commands<'w, 's>,
        tells: MessageWriter<'w, ToClients<Tell>>,
        bodies: Bodies<'w, 's>,
        accounts: Accounts<'w, 's>,
    }

    /// A table of Bridge Race being played, with nothing else of the server
    /// round it: its game, and its seats' bodies.
    #[derive(Resource)]
    struct Played {
        table: Table,
        race: BridgeRace,
    }

    /// The server with no network: a world holding the bodies of a table of
    /// `seats` seats, people in those of `people` and the computer in the
    /// rest, its game begun, and everything every device was told.
    struct Server {
        world: World,
        told: Vec<(Entity, Tell)>,
    }

    impl Server {
        fn new(seats: usize, people: &[usize], seed: u64) -> Self {
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
            let room = Room::Table(Minigame::BridgeRace, 0);
            let shown = world.spawn((Replicated, InRoom(room))).id();
            let mut table = Table::new(Minigame::BridgeRace, 0, shown, seed).expect("a table");
            for seat in 0..seats {
                let (at, turn) = arrival(Venue::BridgeRace, seat);
                let spot = Spot::of(rules::waiting_spot(seat), turn);
                let it = &mut table.seats[seat];
                if people.contains(&seat) {
                    let id = seat as u64 + 1;
                    let member = Member {
                        id,
                        name: format!("Person {id}"),
                        look: "boy".into(),
                        ai: false,
                    };
                    let client = world.spawn_empty().id();
                    let body = world
                        .spawn((
                            member.clone(),
                            Account(member.clone()),
                            spot,
                            Venue::BridgeRace,
                            InRoom(room),
                            Body::person(client, Who(id), room, seat as u8, None, 1000, 0.0),
                        ))
                        .id();
                    it.body = body;
                    it.member = member;
                    it.computer = false;
                } else {
                    let member = Member {
                        id: bot_id(Minigame::BridgeRace, 0, seat),
                        name: format!("Bot {seat}"),
                        look: "girl".into(),
                        ai: true,
                    };
                    let body = world
                        .spawn((member.clone(), spot, Venue::BridgeRace, InRoom(room), Body::bot(room, seat as u8, 0.0)))
                        .id();
                    it.body = body;
                    it.member = member;
                    it.computer = true;
                }
                it.gone = false;
                it.vacant = false;
                table.at[seat] = at;
                table.yaw[seat] = yaw_of(turn);
            }
            table.seats.truncate(seats);
            table.at.truncate(seats);
            table.yaw.truncate(seats);
            table.size = seats;
            table.stage = Stage::Playing { elapsed: 0.0 };
            let race = BridgeRace::new(&table, seed);
            world.insert_resource(Played { table, race });
            let mut server = Self { world, told: Vec::new() };
            server.act(|ctx, table, race| race.begin(ctx, table));
            server
        }

        /// Has the game do something with what a game is handed, as the
        /// table would, and keeps what devices were told.
        fn act(&mut self, deed: impl FnOnce(&mut Ctx, &mut Table, &mut BridgeRace) + Send + Sync + 'static) {
            let mut deed = Some(deed);
            self.world
                .run_system_once(
                    move |mut hands: Hands, config: Res<Config>, mut played: ResMut<Played>| {
                        let Some(deed) = deed.take() else {
                            return;
                        };
                        let Played { table, race } = &mut *played;
                        let mut ctx = Ctx {
                            commands: &mut hands.commands,
                            tells: &mut hands.tells,
                            bodies: &mut hands.bodies,
                            accounts: &hands.accounts,
                            now: 0.0,
                            clock: 1.0e9,
                            config: &config,
                        };
                        deed(&mut ctx, table, race);
                    },
                )
                .expect("the deed");
            let told: Vec<ToClients<Tell>> = self.world.resource_mut::<Messages<ToClients<Tell>>>().drain().collect();
            for told in told {
                if let SendTargets::Single(ClientId::Client(client)) = told.targets {
                    self.told.push((client, told.message));
                }
            }
        }

        /// A frame of the table, `dt` seconds on, as `tables::run` has it:
        /// where each person stands copied in, the game stepped, the
        /// computer's bodies sent where it has them, and the round shown.
        fn frame(&mut self, dt: f32) {
            self.act(move |ctx, table, race| {
                if let Stage::Playing { elapsed } = table.stage {
                    table.stage = Stage::Playing { elapsed: elapsed + dt };
                }
                for seat in 0..table.seats.len() {
                    if table.seats[seat].here()
                        && let Some(spot) = ctx.spot(table, seat)
                    {
                        table.at[seat] = spot.translation();
                        table.yaw[seat] = spot.yaw();
                    }
                    // Whether their device has gone, as `tables::sync` has it.
                    if table.seats[seat].here()
                        && let Ok((body, ..)) = ctx.bodies.get(table.seats[seat].body)
                    {
                        table.seats[seat].away = body.client.is_none();
                    }
                }
                if table.playing() {
                    race.step(ctx, table, dt);
                }
                for seat in 0..table.seats.len() {
                    if table.seats[seat].computer {
                        ctx.move_body(table.seats[seat].body, table.at[seat], table.yaw[seat]);
                    }
                }
                race.show(ctx.commands, table);
            });
        }

        fn table(&self) -> &Table {
            &self.world.resource::<Played>().table
        }

        fn game(&self) -> &Game {
            self.world.resource::<Played>().race.game.as_ref().expect("begun")
        }

        /// The person in `seat` stands at `at`, as their device says.
        fn stand(&mut self, seat: usize, at: Vec3) {
            let body = self.table().seats[seat].body;
            *self.world.get_mut::<Spot>(body).expect("a body") = Spot::of(at, Quat::IDENTITY);
        }

        /// The person in `seat` asks `ask`.
        fn ask(&mut self, seat: usize, ask: Ask) {
            self.act(move |ctx, table, race| race.hear(ctx, table, seat, &ask));
        }

        /// Everything the device of the person in `seat` was told.
        fn heard(&self, seat: usize) -> Vec<&Tell> {
            let body = self.table().seats[seat].body;
            let client = self.world.get::<Body>(body).and_then(|body| body.client).expect("a person");
            self.told.iter().filter(|(to, _)| *to == client).map(|(_, tell)| tell).collect()
        }

        /// How many entities of the room there are with `T` on them.
        fn count<T: Component>(&mut self) -> usize {
            let mut query = self.world.query_filtered::<&InRoom, With<T>>();
            query.iter(&self.world).filter(|room| room.0 == Room::Table(Minigame::BridgeRace, 0)).count()
        }
    }

    #[test]
    fn a_table_against_the_computer_is_played_to_its_end_and_everything_of_it_goes_with_it() {
        // One person, who stands where they arrive and does nothing, against
        // the computer: everyone stood on their island, the person by being
        // told so.
        let mut server = Server::new(2, &[0], 7);
        let arrived = server.heard(0).iter().any(|tell| matches!(tell, Tell::Arrive { venue: Venue::BridgeRace, seat: 0, spot: Some(_), .. }));
        assert!(arrived, "{:?}", server.heard(0));
        assert_eq!(server.table().play_left, Some(rules::PLAY_SECS));
        let bot = server.table().seats[1].body;
        let started = server.world.get::<Spot>(bot).expect("the computer's body").translation();
        // Played out: the computer lays its way across and holds the golden
        // island, and the table is over, the computer's team first, with
        // nothing to pay the person, who lost.
        let mut played = 0.0;
        while server.table().playing() {
            assert!(played < rules::PLAY_SECS + 5.0, "never over");
            server.frame(FRAME);
            played += FRAME;
        }
        let (winner, finish, when) = server.game().result().expect("over");
        println!("over at {when:.1} s of play, {played:.1} s of frames, {:?} won by {finish:?}", winner);
        assert_eq!((winner, finish), (Some(1), rules::Finish::Held));
        let results = server.table().results.clone().expect("how it came out");
        assert_eq!((results.places, results.prizes.clone()), (vec![1, 0], vec![0, 0]));
        assert!(results.scores[1] > 40 && results.scores[0] == 0, "{:?}", results.scores);
        // Its bridge, a rung at a time, each in the room; its body sent
        // where it went; the round and the kits on the table's entity.
        // (Thirty rungs and more, the way across being some forty: more
        // than forty until the evening of 2026-11-18, when with grants of 8
        // to 16 m it came to jump the last few metres now and then, 39
        // rungs here.)
        assert!(server.count::<Plank>() > 30, "{} rungs", server.count::<Plank>());
        let moved = server.world.get::<Spot>(bot).expect("the computer's body").translation();
        assert!(moved.distance(started) > 50.0, "{started} to {moved}");
        let shown = server.table().shown;
        let round = server.world.get::<RaceRound>(shown).expect("the round").clone();
        assert!(round.over() && round.winner == Some(1) && round.across[1] >= 90, "{round:?}");
        assert_eq!(round.finish, Some(rules::Finish::Held));
        assert_eq!(server.world.get::<Kits>(shown).map(|kits| kits.0.len()), Some(2));
        // The table closes: every rung, blast and stick it sent goes.
        server.act(|ctx, table, race| race.close(ctx.commands, table));
        server.world.flush();
        assert_eq!(server.count::<Plank>() + server.count::<Blasted>() + server.count::<Dynamite>(), 0);
    }

    #[test]
    fn what_a_person_asks_goes_to_the_game_and_what_comes_of_it_to_their_device() {
        let mut server = Server::new(2, &[0], 3);
        server.frame(FRAME);
        // A rung of their own bridge, out from their island where they
        // stand: taken, and sent to the room; one from across the map is
        // not.
        use roundtown_net::nav::Region;
        let way = server.game().nav().grid().span(Region::Team(0), Region::Middle, 0.0).expect("red's way across");
        let rung = way.rung(1);
        server.stand(0, Vec3::new(rung.x, rules::LAND_TOP, rung.y));
        server.frame(FRAME);
        for index in 0..3u16 {
            let at = way.rung(usize::from(index));
            server.ask(0, Ask::Rung { chain: 0, index, x: at.x, z: at.y, heading: way.heading() });
        }
        server.ask(0, Ask::Rung { chain: 1, index: 0, x: 90.0, z: 0.0, heading: 0.0 });
        server.frame(FRAME);
        assert_eq!(server.count::<Plank>(), 3);
        assert_eq!(server.game().nav().rungs(0, 0).len(), 3);
        // A punch on the computer's player, standing beside it: knocked.
        let (bot_at, _) = server.game().at(1);
        server.stand(0, bot_at + Vec3::X);
        server.frame(FRAME);
        let knocks = server.game().tally().knocks;
        let who = server.table().seats[1].member.id;
        server.ask(0, Ask::Punch { who, running: true });
        assert_eq!(server.game().tally().knocks, knocks + 1);
        // Their metres as their device says, given back to it should it
        // come back to the game afresh, with the grants since.
        server.ask(0, Ask::BridgeRace(rules::Ask::Metres { metres: 4.5, grants: 0 }));
        // (What the grant brings, the round's word for it beforehand: 8 to
        // 16 m to a team of one, since the evening of 2026-11-18.)
        let next = server.game().round().next_of(0);
        for _ in 0..(rules::GRANT_EVERY / FRAME) as usize + 2 {
            server.frame(FRAME);
        }
        server.act(|ctx, table, race| race.back(ctx, table, 0));
        let told = server.heard(0).into_iter().rev().find_map(|tell| match tell {
            Tell::BridgeRace(rules::Tell::Metres { metres, grants }) => Some((*metres, *grants)),
            _ => None,
        });
        assert_eq!(told, Some((4.5 + next, 1)));
        assert!((rules::grant_metres(2).0..=rules::grant_metres(2).1).contains(&next), "{next}");
        // A fall into the void, as their device says: the chat's line to
        // them, and a moment later stood on their island's middle again.
        server.stand(0, Vec3::new(-60.0, -40.0, -10.0));
        server.ask(0, Ask::Respawn);
        for _ in 0..(rules::RESPAWN / FRAME) as usize + 4 {
            server.frame(FRAME);
        }
        let heard = server.heard(0);
        assert!(heard.iter().any(|tell| matches!(tell, Tell::BridgeRace(rules::Tell::Feed(rules::Feed::Fell { seat: 0, .. })))));
        let back = heard.iter().rev().find_map(|tell| match tell {
            Tell::Arrive { venue: Venue::BridgeRace, spot: Some(spot), .. } => Some(spot.translation()),
            _ => None,
        });
        assert!(back.is_some_and(|at| at.distance(rules::home(0).0) < 0.05), "{back:?}");
        // Skipped, for trying it out: over now, by the time limit's rule.
        server.ask(0, Ask::Skip);
        server.frame(FRAME);
        assert_eq!(server.game().result().map(|(_, finish, _)| finish), Some(rules::Finish::Time));
        assert!(server.table().over());
    }

    #[test]
    fn a_person_whose_device_has_gone_holds_nothing_and_a_rung_past_their_metres_reaches_nobody() {
        // One person against the computer, on the golden island, holding
        // it; then their device gone, the body standing there with nothing
        // to knock it off: not on it, however long it stands there.
        let mut server = Server::new(2, &[0], 5);
        server.stand(0, Vec3::new(-2.0, rules::LAND_TOP, -0.5));
        for _ in 0..(1.0 / FRAME) as usize {
            server.frame(FRAME);
        }
        assert_eq!(server.game().round().on_gold[0], 1);
        assert!(server.game().round().hold[0] > 0);
        let body = server.table().seats[0].body;
        server.world.get_mut::<Body>(body).expect("a body").client = None;
        for _ in 0..((rules::HOLD + 1.0) / FRAME) as usize {
            server.frame(FRAME);
        }
        assert!(server.table().seats[0].away);
        assert_eq!((server.game().round().on_gold[0], server.game().round().hold[0]), (0, 0));
        assert!(server.table().playing(), "{:?}", server.game().result());
        // A device laying straight across at the start, as if it had all
        // the metres it says: taken as far as the ten it was given and what
        // its own bridge lays past them, and the rest sent to nobody.
        let mut server = Server::new(2, &[0], 6);
        use roundtown_net::nav::Region;
        let way = server.game().nav().grid().span(Region::Team(0), Region::Middle, 0.0).expect("red's way across");
        for index in 0..=way.planks() {
            let at = way.rung(index);
            server.stand(0, Vec3::new(at.x, rules::LAND_TOP, at.y));
            server.frame(FRAME);
            server.ask(0, Ask::Rung { chain: 0, index: index as u16, x: at.x, z: at.y, heading: way.heading() });
        }
        server.frame(FRAME);
        let (rungs, laid) = (server.count::<Plank>(), server.game().laid()[0]);
        assert!(rungs > 6 && rungs < way.planks() / 2, "{rungs} of {} rungs", way.planks() + 1);
        assert!(laid > rules::START_METRES && laid < 2.0 * rules::START_METRES, "{laid} m laid");
    }
}
