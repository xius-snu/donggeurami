//! Knock Off at its tables, on the server (since 2026-11-18; Hajun: "You
//! spawn on a tiny icy circular island with 2~4 other ppl (fill with ai
//! like bridgewars and its deathmatch so 1v1v1v1) ... Everyone is frozen in
//! place, and before every certain time interval like 10 second, you choose
//! the direction and intensity that you wanna slide"): what a table of it
//! plays (`tables::Rules`), on `iceisland.glb`.
//!
//! The game itself is `roundtown_net::knock_off::Game`, the very one a
//! device plays offline: this only carries it out at a table. It is
//! stepped by the clock every device has for the town's (`Ctx::clock`), so
//! that whatever it stamps a stage with ([`KnockRound`]: when an aim ends,
//! when a slide begins) is the moment every device shows it at; it hears
//! each person's aim and their word that they are ready; it moves the
//! computer's players' bodies where the game has them; and it says who is
//! out (`Seat::out`), and how everyone came, once one is left. What it
//! sends the room is the round alone, whenever any of it changes: nobody's
//! aim is in it before the reveal, and each device plays every slide out
//! itself from the round's start and launches, to the bit as this did, and
//! places every body itself, people's too, while the game is played (the
//! device's `Frozen`), so that nothing of where anyone is goes over the
//! network between the reveal and the next turn.
//!
//! While the table waits the computer's players come out onto the ice one
//! at a time, as Bridge Wars' come into its waiting room, and stand about
//! on it, now and then strolling somewhere else on it
//! (`knock_off::Loiter`, the device's own offline), until the game stands
//! everyone on their spot.

use bevy::prelude::*;
use roundtown_net::knock_off::{self, Event, Game, KnockRound, Loiter};
use roundtown_net::{Ask, Tell};

use crate::tables::{Ctx, Rules, Table, name, yaw_of};

/// A game of Knock Off at its table: the game, once it has begun; the
/// computer's players strolling about the ice until then; and the round as
/// the room was last sent it, to send again only when it changes.
pub(crate) struct KnockOff {
    seed: u64,
    game: Option<Game>,
    loiter: Loiter,
    sent: Option<KnockRound>,
}

impl KnockOff {
    /// The game at a new table, numbered as `table` is, its dice thrown from
    /// `seed`.
    pub(crate) fn new(_table: &Table, seed: u64) -> Self {
        Self {
            seed,
            game: None,
            loiter: Loiter::new(seed ^ 0x6B6E_6F63_6B5F_6F66),
            sent: None,
        }
    }

    /// Carries out what a step of the game came to: whoever went in is out
    /// of it, free to watch or to go (`Seat::out`: "If you are out, you can
    /// spectate or exit"); and the log says how it goes. The headlines and
    /// the lines in the chat are every device's own, from the round it is
    /// sent and the slide it plays itself.
    fn carry_out(table: &mut Table, events: Vec<Event>) {
        let game = name(table.game);
        let id = table.id;
        for event in events {
            match event {
                Event::Aim { turn } => info!("{game} table {id}: turn {turn}, everyone aims"),
                Event::Fell { seat, by, broke } => {
                    if let Some(it) = table.seats.get_mut(seat) {
                        it.out = true;
                    }
                    let who = table.seats.get(seat).map_or("?", |it| it.member.name.as_str());
                    match (by.and_then(|by| table.seats.get(by)), broke) {
                        (_, true) => info!("{game} table {id}: the ice broke under {who}"),
                        (Some(by), false) => info!("{game} table {id}: {} knocked {who} in", by.member.name),
                        (None, false) => info!("{game} table {id}: {who} slid in"),
                    }
                }
                Event::Over { first } => {
                    let first: Vec<&str> = first
                        .iter()
                        .filter_map(|&seat| table.seats.get(seat))
                        .map(|it| it.member.name.as_str())
                        .collect();
                    info!("{game} table {id}: over, first {}", first.join(" and "));
                }
                Event::Reveal | Event::Slide | Event::Breaks { .. } => {}
            }
        }
    }

    /// Where everyone stands on the ice now, by seat, as the table has them
    /// (none for a seat nobody is in, or whoever has left): for the
    /// computer's players to keep clear of as they stroll about.
    fn everyone(table: &Table) -> Vec<Option<Vec2>> {
        (0..table.seats.len())
            .map(|seat| {
                let it = &table.seats[seat];
                (!it.vacant && !it.gone).then(|| table.at[seat].xz())
            })
            .collect()
    }
}

impl Rules for KnockOff {
    fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
        let (at, turn) = knock_off::arrival(seat);
        (at, yaw_of(turn))
    }

    /// The computer's players, as each comes out onto the ice, stand about
    /// on it and now and then stroll somewhere else on it, until the game
    /// begins.
    fn idle(&mut self, _ctx: &mut Ctx, table: &mut Table, dt: f32) {
        let everyone = Self::everyone(table);
        for seat in 0..table.seats.len() {
            let it = &table.seats[seat];
            if !it.computer || it.vacant {
                if self.loiter.has(seat) {
                    self.loiter.leave(seat);
                }
                continue;
            }
            if !self.loiter.has(seat) {
                self.loiter.enter(seat, table.at[seat].xz(), table.yaw[seat]);
            }
            if let Some((at, yaw)) = self.loiter.step(seat, &everyone, dt) {
                table.at[seat] = Vec3::new(at.x, 0.0, at.y);
                table.yaw[seat] = yaw;
            }
        }
    }

    /// Everyone on their spot, facing the middle, and the first turn's aim
    /// begins: by the clock every device has for the town's, which the whole
    /// game goes by from here.
    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        self.loiter.clear();
        let people: Vec<bool> = table.seats.iter().map(|it| it.person()).collect();
        let mut game = Game::new(table.seats.len(), &people, self.seed);
        let events = game.begin(ctx.clock);
        for seat in 0..table.seats.len() {
            let (at, yaw) = game.at(seat);
            ctx.place(table, seat, at, yaw);
        }
        let kinds: Vec<String> = (0..table.seats.len())
            .filter_map(|seat| game.persona(seat).map(|kind| format!("{seat} {kind}")))
            .collect();
        info!("{} table {}: the computer plays {}", name(table.game), table.id, kinds.join(", "));
        Self::carry_out(table, events);
        self.game = Some(game);
    }

    /// The game on, to the clock: the computer's players' bodies where it
    /// has them, what it came to carried out, and once one is left, or
    /// nobody, the table told how everyone came.
    fn step(&mut self, ctx: &mut Ctx, table: &mut Table, _dt: f32) {
        let Some(game) = &mut self.game else {
            return;
        };
        let events = game.step(ctx.clock);
        for seat in 0..table.seats.len() {
            if table.seats[seat].computer {
                let (at, yaw) = game.at(seat);
                table.at[seat] = at;
                table.yaw[seat] = yaw;
            }
        }
        Self::carry_out(table, events);
        if let Some((places, scores, total)) = game.outcome() {
            table.finish(places, scores, total);
        }
    }

    /// Each person's aim as it changes, and their word that they are ready;
    /// for trying the game out (`RT_ALLOW_SKIP`), a skip, which has everyone
    /// still on the ice ready, so that the aim is over at once; and before
    /// the game, someone who has gone into the sea stood back on their spot
    /// (they wade there, with no jump to climb out by: Knock Off has none).
    fn hear(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize, ask: &Ask) {
        match (ask, &mut self.game) {
            (Ask::KnockOff(knock_off::Ask::Aim { x, z, power }), Some(game)) => {
                game.aim(seat, Vec2::new(*x, *z), *power);
            }
            (Ask::KnockOff(knock_off::Ask::Ready), Some(game)) => game.ready(seat),
            (Ask::Skip, Some(game)) => {
                for anyone in 0..table.seats.len() {
                    game.ready(anyone);
                }
            }
            (Ask::Respawn, None) => {
                let (at, yaw) = self.waiting_spot(table, seat);
                ctx.place(table, seat, at, yaw);
            }
            _ => {}
        }
    }

    /// Whoever has left for good is out of the game as they go, in the turn
    /// they go in (`Game::leave`): their place is kept and paid.
    fn left(&mut self, _ctx: &mut Ctx, table: &mut Table, seat: usize, _body: Option<Entity>) {
        if let Some(game) = &mut self.game {
            game.leave(seat);
            info!("{} table {}: seat {seat} left the game", name(table.game), table.id);
        }
    }

    /// A device back in the game while everyone aims has its own aim drawn
    /// where it left it ([`knock_off::Tell::Aimed`]); everything else it
    /// needs is in the round it is sent.
    fn back(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize) {
        if let Some((way, power)) = self.game.as_ref().and_then(|game| game.aim_of(seat)) {
            ctx.tell(table, seat, Tell::KnockOff(knock_off::Tell::Aimed { x: way.x, z: way.y, power }));
        }
    }

    fn show(&mut self, commands: &mut Commands, table: &Table) {
        let Some(game) = &self.game else {
            return;
        };
        let round = game.round();
        if self.sent.as_ref() != Some(&round) {
            commands.entity(table.shown).insert(round.clone());
            self.sent = Some(round);
        }
    }

    /// Nothing of its own to take away: the round is on the table's own
    /// entity, which goes with the table, and nothing is put on anyone's
    /// body.
    fn close(&mut self, _commands: &mut Commands, _table: &Table) {}
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::{RunSystemOnce, SystemParam};
    use bevy_replicon::prelude::*;
    use roundtown_net::knock_off::KnockStage;
    use roundtown_net::{Member, Minigame, Spot, Venue};

    use super::*;
    use crate::Config;
    use crate::players::{Accounts, Body, Who};
    use crate::rooms::{InRoom, Room};
    use crate::tables::{Bodies, Stage};

    /// The clock a test begins by: a day in November 2026, in seconds since
    /// 1970, as the server's would be.
    const CLOCK: f64 = 1_795_000_000.0;

    /// A table of Knock Off being played, outside any server: its game, the
    /// table, the clock, and the bodies of the people at it.
    #[derive(Resource)]
    struct Played {
        rules: KnockOff,
        table: Table,
        clock: f64,
    }

    /// A table of four seats, `people` of them people's (each a body in the
    /// table's room with a device), the rest the computer's, its game made
    /// from `seed`: as it is the moment its countdown is over.
    fn table(people: usize, seed: u64) -> (World, Vec<Entity>) {
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
        let shown = world.spawn_empty().id();
        let mut table = Table::new(Minigame::KnockOff, 0, shown, seed).expect("Knock Off is played at tables");
        let room = Room::Table(Minigame::KnockOff, 0);
        let mut clients = Vec::new();
        for seat in 0..4 {
            let member = Member {
                id: if seat < people { seat as u64 + 1 } else { crate::tables::bot_id(Minigame::KnockOff, 0, seat) },
                name: format!("Seat {seat}"),
                look: "boy".into(),
                ai: seat >= people,
            };
            let it = &mut table.seats[seat];
            it.vacant = false;
            it.gone = false;
            it.computer = seat >= people;
            it.member = member.clone();
            if seat < people {
                let client = world.spawn_empty().id();
                let (at, turn) = knock_off::arrival(seat);
                it.body = world
                    .spawn((
                        member.clone(),
                        Spot::of(at, turn),
                        Venue::KnockOff,
                        InRoom(room),
                        Body::person(client, Who(member.id), room, seat as u8, None, 950, 0.0),
                    ))
                    .id();
                clients.push(client);
            }
        }
        table.size = 4;
        table.stage = Stage::Playing { elapsed: 0.0 };
        let rules = KnockOff::new(&table, seed);
        world.insert_resource(Played { rules, table, clock: CLOCK });
        (world, clients)
    }

    /// Something for the game at the table to do ([`act`]).
    #[derive(Resource)]
    struct Deed(fn(&mut KnockOff, &mut Ctx, &mut Table));

    /// What the test acts on the world with, as the tables' own systems do
    /// (theirs is `tables::Hands`).
    #[derive(SystemParam)]
    struct Hands<'w, 's> {
        commands: Commands<'w, 's>,
        tells: MessageWriter<'w, ToClients<Tell>>,
        bodies: Bodies<'w, 's>,
        accounts: Accounts<'w, 's>,
    }

    /// Does the deed asked for, with what a game is handed, the clock where
    /// the test has it.
    fn do_the_deed(mut hands: Hands, config: Res<Config>, deed: Res<Deed>, mut played: ResMut<Played>) {
        let Played { rules, table, clock } = &mut *played;
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
            now: 0.0,
            clock: *clock,
            config: &config,
        };
        (deed.0)(rules, &mut ctx, table);
    }

    /// Has the game at the table do something with what a game is handed,
    /// the clock where the test has it.
    fn act(world: &mut World, deed: fn(&mut KnockOff, &mut Ctx, &mut Table)) {
        world.insert_resource(Deed(deed));
        world.run_system_once(do_the_deed).expect("the game's deed");
    }

    /// One frame of play `dt` seconds on by the clock: the game stepped, and
    /// the room shown what changed.
    fn frame(world: &mut World, dt: f64) {
        world.resource_mut::<Played>().clock += dt;
        act(world, |rules, ctx, table| {
            if table.playing() {
                rules.step(ctx, table, 1.0 / 60.0);
            }
            rules.show(ctx.commands, table);
        });
    }

    fn sent_round(world: &World) -> Option<KnockRound> {
        let shown = world.resource::<Played>().table.shown;
        world.get::<KnockRound>(shown).cloned()
    }

    /// Everything the devices were told, by device.
    fn told(world: &mut World) -> Vec<(Entity, Tell)> {
        world
            .resource_mut::<Messages<ToClients<Tell>>>()
            .drain()
            .filter_map(|told| match told.targets {
                SendTargets::Single(ClientId::Client(client)) => Some((client, told.message)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_table_of_one_person_and_the_computer_is_played_from_go_to_its_end() {
        let (mut world, clients) = table(1, 7);
        act(&mut world, |rules, ctx, table| rules.begin(ctx, table));
        // Everyone on their spot as the game begins: the person told so,
        // the computer's players where the table sends their bodies.
        let said = told(&mut world);
        assert!(
            said.iter().any(|(client, tell)| *client == clients[0] && matches!(tell, Tell::Arrive { venue: Venue::KnockOff, seat: 0, .. })),
            "{said:?}"
        );
        {
            let table = &world.resource::<Played>().table;
            for seat in 1..4 {
                assert!((table.at[seat] - knock_off::arrival(seat).0).length() < 1e-4, "seat {seat}");
            }
        }
        frame(&mut world, 1.0 / 60.0);
        let first = sent_round(&world).expect("the round is sent");
        assert_eq!((first.turn, first.stage), (1, KnockStage::Aim));
        assert!(first.launches.is_empty(), "nobody's aim before the reveal");
        // The person aims across the middle, hard, and says they are ready.
        act(&mut world, |rules, ctx, table| {
            rules.hear(ctx, table, 0, &Ask::KnockOff(knock_off::Ask::Aim { x: 0.3, z: -1.0, power: 7 }));
            rules.hear(ctx, table, 0, &Ask::KnockOff(knock_off::Ask::Ready));
        });
        // Played on by the clock: the stages in order, the room sent each
        // round as it changes, whoever goes in out, and at the end how
        // everyone came.
        let mut stages = vec![(first.turn, first.stage)];
        let mut seconds = 0.0;
        while !world.resource::<Played>().table.over() {
            assert!(seconds < 400.0, "never over: {stages:?}");
            frame(&mut world, 1.0 / 30.0);
            seconds += 1.0 / 30.0;
            let round = sent_round(&world).expect("the round");
            if stages.last() != Some(&(round.turn, round.stage)) {
                stages.push((round.turn, round.stage));
            }
            let played = world.resource::<Played>();
            for seat in 0..4 {
                if round.out[seat].is_some() {
                    assert!(played.table.seats[seat].out, "seat {seat} went in and is out: {round:?}");
                }
                // The computer's bodies where the game has them.
                if played.table.seats[seat].computer {
                    assert_eq!(played.table.at[seat], played.rules.game.as_ref().expect("a game").at(seat).0);
                }
            }
        }
        assert_eq!(stages[1].1, KnockStage::Reveal, "{stages:?}");
        assert_eq!(stages.last().map(|stage| stage.1), Some(KnockStage::Over));
        let played = world.resource::<Played>();
        let results = played.table.results.clone().expect("how it came out");
        let (places, scores, _) = played.rules.game.as_ref().expect("the game").outcome().expect("over");
        assert_eq!((results.places.clone(), results.scores.clone()), (places, scores));
        assert_eq!(results.prizes, knock_off::prizes(&results, &[true, false, false, false]));
        assert!(results.places.contains(&0), "someone came first: {results:?}");
    }

    #[test]
    fn nobody_s_aim_is_sent_before_the_reveal_and_a_device_back_is_told_its_own() {
        let (mut world, clients) = table(2, 11);
        act(&mut world, |rules, ctx, table| rules.begin(ctx, table));
        told(&mut world);
        act(&mut world, |rules, ctx, table| {
            rules.hear(ctx, table, 0, &Ask::KnockOff(knock_off::Ask::Aim { x: 1.0, z: 0.0, power: 4 }));
            rules.hear(ctx, table, 1, &Ask::KnockOff(knock_off::Ask::Aim { x: -1.0, z: 0.0, power: 9 }));
        });
        frame(&mut world, 0.5);
        let round = sent_round(&world).expect("the round");
        assert_eq!(round.stage, KnockStage::Aim);
        assert!(round.launches.is_empty());
        // Seat 1's device comes back: told its own aim, and nobody else's.
        act(&mut world, |rules, ctx, table| rules.back(ctx, table, 1));
        let said = told(&mut world);
        assert_eq!(said.len(), 1, "{said:?}");
        assert_eq!(said[0].0, clients[1]);
        assert!(matches!(said[0].1, Tell::KnockOff(knock_off::Tell::Aimed { power: 9, .. })), "{said:?}");
        // Both ready: the aim is over at once, and the reveal shows both.
        act(&mut world, |rules, ctx, table| {
            rules.hear(ctx, table, 0, &Ask::KnockOff(knock_off::Ask::Ready));
            rules.hear(ctx, table, 1, &Ask::KnockOff(knock_off::Ask::Ready));
        });
        frame(&mut world, 1.0 / 60.0);
        let round = sent_round(&world).expect("the round");
        assert_eq!(round.stage, KnockStage::Reveal);
        assert_eq!(round.launches.len(), 4);
        assert!(round.launches[0].0 > 0.0 && round.launches[1].0 < 0.0, "{round:?}");
    }

    #[test]
    fn whoever_leaves_is_out_as_they_go_and_their_place_is_kept() {
        let (mut world, _) = table(2, 5);
        act(&mut world, |rules, ctx, table| rules.begin(ctx, table));
        frame(&mut world, 1.0 / 60.0);
        act(&mut world, |rules, ctx, table| rules.left(ctx, table, 1, None));
        frame(&mut world, 1.0 / 60.0);
        let round = sent_round(&world).expect("the round");
        assert_eq!(round.out[1], Some(1), "{round:?}");
        assert_eq!(round.start[1], None);
        // Played to its end with the other person never ready: whoever left
        // in the first turn is last, or shares last.
        let mut seconds = 0.0;
        while !world.resource::<Played>().table.over() {
            assert!(seconds < 400.0, "never over");
            frame(&mut world, 1.0 / 20.0);
            seconds += 1.0 / 20.0;
        }
        let results = world.resource::<Played>().table.results.clone().expect("how it came out");
        assert_eq!(results.places[1], *results.places.iter().max().expect("places"), "{results:?}");
    }

    #[test]
    fn a_skip_ends_the_aim_at_once_and_a_fall_before_the_game_is_put_back() {
        let (mut world, clients) = table(1, 3);
        // Before the game: into the sea, and back on the spot.
        act(&mut world, |rules, ctx, table| {
            table.stage = Stage::Waiting { since_join: 0.0, coming: None };
            rules.hear(ctx, table, 0, &Ask::Respawn);
        });
        let said = told(&mut world);
        assert!(said.iter().any(|(client, tell)| *client == clients[0] && matches!(tell, Tell::Arrive { venue: Venue::KnockOff, .. })), "{said:?}");
        act(&mut world, |rules, ctx, table| {
            table.stage = Stage::Playing { elapsed: 0.0 };
            rules.begin(ctx, table);
        });
        frame(&mut world, 1.0);
        assert_eq!(sent_round(&world).expect("the round").stage, KnockStage::Aim);
        act(&mut world, |rules, ctx, table| rules.hear(ctx, table, 0, &Ask::Skip));
        frame(&mut world, 1.0 / 60.0);
        assert_eq!(sent_round(&world).expect("the round").stage, KnockStage::Reveal);
    }

    #[test]
    fn the_computer_s_players_stroll_about_the_ice_while_it_waits_and_stay_on_it() {
        let (mut world, _) = table(1, 9);
        act(&mut world, |_, _, table| table.stage = Stage::Waiting { since_join: 0.0, coming: Some(0.0) });
        let start: Vec<Vec3> = world.resource::<Played>().table.at.clone();
        let mut moved = [false; 4];
        for _ in 0..(60 * 40) {
            act(&mut world, |rules, ctx, table| rules.idle(ctx, table, 1.0 / 60.0));
            let table = &world.resource::<Played>().table;
            for seat in 1..4 {
                let at = table.at[seat];
                assert!(at.xz().length() < knock_off::RADIUS - 1.0, "seat {seat} off toward the edge: {at}");
                moved[seat] |= at.distance(start[seat]) > 0.5;
            }
        }
        assert_eq!(moved, [false, true, true, true], "{moved:?}");
    }
}
