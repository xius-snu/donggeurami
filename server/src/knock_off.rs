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
//! computer's players' bodies where the game has them between slides; and
//! it says who is out (`Seat::out`), and how everyone came, once one is
//! left, or once nobody is left to play it for. What it sends the room is
//! the round, whenever any of it changes: nobody's aim is in it before the
//! reveal, and each device plays every slide out itself from the round's
//! start and launches, to the bit as this did, and places every body
//! itself, people's too, while the game is played (the device's `Frozen`),
//! so that nothing of where anyone is goes over the network while everyone
//! slides: the computer's bodies are sent where each stands as a turn
//! begins, turns to the way it will go, and comes to rest, and nobody's
//! between.
//!
//! While the table waits the computer's players come out onto the ice one
//! at a time, as Bridge Wars' come into its waiting room, and stand about
//! on it, now and then strolling somewhere else on it
//! (`knock_off::Loiter`, the device's own offline), out of everyone's way,
//! until the game begins, everyone where they stood.
//!
//! **The island is thrown as the table opens** (since 2026-11-27, Hajun:
//! "each game, a truly random shape of island forms"): from a seed of the
//! table's own, which the server throws from nothing anyone chose
//! (`tables::open_table`), sent the table's room from the first frame on
//! (`knock_off::Island`), so that everyone walks about on it while the table
//! waits and every device plays every slide on it. **Nobody is stood on a
//! spot as the game begins** (Hajun: "wherever ur last position was (or
//! nearest surface position before falling into the water if u fell into the
//! water at the start time) is your starting position"): each begins where
//! their device last said they stood, the computer's players where they
//! strolled to, on the ice and nobody on anyone (`knock_off::settle`).

use std::sync::Arc;

use bevy::prelude::*;
use roundtown_net::knock_off::{self, Event, Floe, Game, Hops, Island, KnockRound, KnockStage, Loiter, STOOD_BACK, may_hop};
use roundtown_net::{Ask, Tell};

use crate::tables::{Ctx, Rules, Table, name, yaw_of};

/// How far on by the clock the rest of a game is played at once, nobody
/// being left to play it for, in seconds: a day, which the rules pass
/// through in one step to the end of any game (`Game::step`), as a device
/// offline does when you leave once you are out.
const PLAYED_OUT: f64 = 86_400.0;

/// How long a table goes on with nobody connected to it while every person
/// at it is in the sea, their places settled, before the rest of its game
/// is played at once and everyone paid, in seconds (mine). The tables give
/// up a table nobody has been connected to for a minute, paying nobody
/// (`tables::ABANDONED`): an app put away once you are out, the computer's
/// players playing on without you, would lose your place's coins with it.
/// A few seconds first, for a connection that is only lost a moment.
const UNWATCHED_PLAY_OUT: f32 = 10.0;

/// A game of Knock Off at its table: the island it is played on, and
/// whether the room has been sent it; the game, once it has begun; the
/// computer's players strolling about the ice until then, and hopping now
/// and then all along (`knock_off::Hops`, since 2026-11-27); the round as
/// the room was last sent it, to send again only when it changes; and the
/// turn and stage the people's bodies were last stood where the game has
/// them, as the server keeps them.
pub(crate) struct KnockOff {
    seed: u64,
    island: Island,
    floe: Arc<Floe>,
    island_sent: bool,
    game: Option<Game>,
    loiter: Loiter,
    hops: Hops,
    sent: Option<KnockRound>,
    stood: Option<(u16, KnockStage)>,
}

impl KnockOff {
    /// The game at a new table, numbered as `table` is, its dice and its
    /// island thrown from `seed`.
    pub(crate) fn new(_table: &Table, seed: u64) -> Self {
        let island = Island {
            seed: seed.rotate_left(29) ^ 0x4973_6C61_6E64_2121,
        };
        Self {
            seed,
            floe: Arc::new(island.floe()),
            island,
            island_sent: false,
            game: None,
            loiter: Loiter::new(seed ^ 0x6B6E_6F63_6B5F_6F66),
            hops: Hops::new(seed ^ 0x686F_7073),
            sent: None,
            stood: None,
        }
    }

    /// Where everyone but `seat` stands at the table now (whoever is seated
    /// there and has not gone): for a body put somewhere to keep clear of.
    fn others(table: &Table, seat: usize) -> Vec<Vec2> {
        (0..table.seats.len())
            .filter(|&other| other != seat && !table.seats[other].vacant && !table.seats[other].gone)
            .map(|other| table.at[other].xz())
            .collect()
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

    /// The rest of the game played at once, nobody being left to play it
    /// for, and the table told how everyone came (`Table::finish`), so that
    /// whoever is in the sea is paid for where they came: whoever left in
    /// this frame too out as they went, and the computer's players, alone
    /// on the ice, playing on to the end in a step, as the rules do a stall
    /// of any length (the same as a device offline does when you leave once
    /// you are out).
    fn play_out(game: &mut Game, ctx: &Ctx, table: &mut Table) {
        for seat in 0..table.seats.len() {
            if table.seats[seat].person() && table.seats[seat].gone {
                game.leave(seat);
            }
        }
        let events = game.step(ctx.clock + PLAYED_OUT);
        Self::carry_out(table, events);
        if let Some((places, scores, total)) = game.outcome() {
            info!("{} table {}: nobody is left to play it for, and it is played out at once", name(table.game), table.id);
            table.finish(places, scores, total);
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
    /// Out on the seat's own side of the island ([`knock_off::waiting_spot`]),
    /// or the nearest spot to it clear of everyone already there.
    fn waiting_spot(&mut self, table: &Table, seat: usize) -> (Vec3, f32) {
        let (at, turn) = knock_off::waiting_spot(seat, &self.floe);
        let at = self.floe.free_spot(at.xz(), &Self::others(table, seat), 1.0, STOOD_BACK, knock_off::ROOM);
        (Vec3::new(at.x, 0.0, at.y), yaw_of(turn))
    }

    /// The computer's players, as each comes out onto the ice, stand about
    /// on it and now and then stroll somewhere else on it, until the game
    /// begins; and some of them hop now and then (`Hops`), how high sent as
    /// their bodies are.
    fn idle(&mut self, _ctx: &mut Ctx, table: &mut Table, dt: f32) {
        let everyone = Self::everyone(table);
        for seat in 0..table.seats.len() {
            let it = &table.seats[seat];
            if !it.computer || it.vacant {
                if self.loiter.has(seat) {
                    self.loiter.leave(seat);
                    self.hops.leave(seat);
                }
                continue;
            }
            if !self.loiter.has(seat) {
                self.loiter.enter(seat, table.at[seat].xz(), table.yaw[seat]);
            }
            if let Some((at, yaw)) = self.loiter.step(seat, &everyone, &self.floe, dt) {
                let lift = self.hops.step(seat, dt, true);
                table.at[seat] = Vec3::new(at.x, lift, at.y);
                table.yaw[seat] = yaw;
            }
        }
    }

    /// Everyone where they stood as the countdown ended (a person where
    /// their device last said, the computer's players where they strolled
    /// to), on the ice and nobody on anyone, facing the middle, and the first
    /// turn's aim begins: by the clock every device has for the town's, which
    /// the whole game goes by from here.
    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        self.loiter.clear();
        let people: Vec<bool> = table.seats.iter().map(|it| it.person()).collect();
        let standing: Vec<Option<Vec2>> = (0..table.seats.len())
            .map(|seat| (!table.seats[seat].vacant && !table.seats[seat].gone).then(|| table.at[seat].xz()))
            .collect();
        let mut game = Game::new(table.seats.len(), &people, self.seed, Arc::clone(&self.floe));
        let events = game.begin(ctx.clock, &standing);
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
    /// nobody, the table told how everyone came. Every person in the sea
    /// and none of them connected a while ([`UNWATCHED_PLAY_OUT`]), there
    /// is nobody to show the rest to, and it is played at once.
    fn step(&mut self, ctx: &mut Ctx, table: &mut Table, dt: f32) {
        let Some(game) = &mut self.game else {
            return;
        };
        let events = game.step(ctx.clock);
        // The computer's bodies where the game has them, but while everyone
        // slides: each device plays the slide out itself and places every
        // body (`Frozen`), so where they are as they go would be sent to
        // nobody who looks; they are sent where it left them as the ice
        // breaks. Hopping now and then, where hops are (`may_hop`): how high
        // is sent as they are, and every device draws the hop.
        let stage = game.stage();
        for seat in 0..table.seats.len() {
            if !table.seats[seat].computer {
                continue;
            }
            let lift = self.hops.step(seat, dt, may_hop(stage) && game.in_play_at(seat));
            if stage != KnockStage::Slide {
                let (at, yaw) = game.at(seat);
                table.at[seat] = at + Vec3::Y * if game.in_play_at(seat) { lift } else { 0.0 };
                table.yaw[seat] = yaw;
            }
        }
        // Each person's body as the server keeps it, where the game has
        // them, as every stage but a slide begins: their device places it
        // itself and says nothing of it while the game is played, but for a
        // hop on the spot (since 2026-11-27), which has to be a move from
        // where it stands; after a slide the server's had it where the last
        // turn began, and took the hop for a leap across the ice.
        if stage != KnockStage::Slide && self.stood != Some((game.turn(), stage)) {
            self.stood = Some((game.turn(), stage));
            for seat in 0..table.seats.len() {
                let it = &table.seats[seat];
                if it.person() && it.here() && game.in_play_at(seat) {
                    let (at, yaw) = game.at(seat);
                    ctx.move_body(it.body, at, yaw);
                }
            }
        }
        Self::carry_out(table, events);
        if let Some((places, scores, total)) = game.outcome() {
            table.finish(places, scores, total);
            return;
        }
        let in_the_sea = (0..table.seats.len()).all(|seat| !table.seats[seat].person() || !game.in_play_at(seat));
        if in_the_sea && table.unwatched >= UNWATCHED_PLAY_OUT {
            Self::play_out(game, ctx, table);
        }
    }

    /// Each person's aim as it changes, and their word that they are ready;
    /// and before the game, someone who has gone into the sea stood back on
    /// the ice, the nearest spot to where their device last said they were,
    /// clear of everyone (they wade there, with no jump to climb out by:
    /// Knock Off has none). A skip (`RT_ALLOW_SKIP`) is the table's alone, for its
    /// wait and its countdown (mine): in play Ready is the game's own skip,
    /// and a tap on the board meant for the countdown that lands as play
    /// begins is to end nobody's aim, as it ended everyone's when a skip in
    /// play had everyone ready.
    fn hear(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize, ask: &Ask) {
        match (ask, &mut self.game) {
            (Ask::KnockOff(knock_off::Ask::Aim { x, z, power }), Some(game)) => {
                game.aim(seat, Vec2::new(*x, *z), *power);
            }
            (Ask::KnockOff(knock_off::Ask::Ready), Some(game)) => game.ready(seat),
            (Ask::Respawn, None) => {
                let Some(last) = table.at.get(seat).copied() else {
                    return;
                };
                let back = self.floe.free_spot(last.xz(), &Self::others(table, seat), 1.0, STOOD_BACK, knock_off::ROOM);
                let yaw = table.yaw.get(seat).copied().unwrap_or(0.0);
                ctx.place(table, seat, Vec3::new(back.x, 0.0, back.y), yaw);
            }
            _ => {}
        }
    }

    /// Whoever has left for good is out of the game as they go, in the turn
    /// they go in (`Game::leave`): their place is kept and paid. Once the
    /// last person has gone, the computer's players play the rest of it at
    /// once and everyone is paid as they came ([`KnockOff::play_out`]):
    /// otherwise the table, with nobody at it, would close as it is, paying
    /// nobody (Hajun: "If you are out, you can spectate or exit"; the panel
    /// says "Your place is kept").
    fn left(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize, _body: Option<Entity>) {
        if let Some(game) = &mut self.game {
            game.leave(seat);
            info!("{} table {}: seat {seat} left the game", name(table.game), table.id);
            if table.playing() && table.people() == 0 {
                Self::play_out(game, ctx, table);
            }
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

    /// The island, once, from the first frame the table is open; and the
    /// round whenever it changes, once the game has begun.
    fn show(&mut self, commands: &mut Commands, table: &Table) {
        if !self.island_sent {
            commands.entity(table.shown).insert(self.island);
            self.island_sent = true;
        }
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
    /// from `seed`, everyone standing where they arrived on its island: as
    /// it is the moment its countdown is over.
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
        let mut rules = KnockOff::new(&table, seed);
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
            let (at, yaw) = rules.waiting_spot(&table, seat);
            table.at[seat] = at;
            table.yaw[seat] = yaw;
            let it = &mut table.seats[seat];
            if seat < people {
                let client = world.spawn_empty().id();
                it.body = world
                    .spawn((
                        member.clone(),
                        Spot::of(at, Quat::from_rotation_y(yaw)),
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
        // Everyone where they stood as the game begins: the person told so,
        // the computer's players where the table sends their bodies.
        let said = told(&mut world);
        assert!(
            said.iter().any(|(client, tell)| *client == clients[0] && matches!(tell, Tell::Arrive { venue: Venue::KnockOff, seat: 0, .. })),
            "{said:?}"
        );
        {
            let played = world.resource::<Played>();
            for seat in 1..4 {
                let (spot, _) = knock_off::waiting_spot(seat, &played.rules.floe);
                assert!((played.table.at[seat] - spot).length() < 1e-4, "seat {seat}");
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
        let mut before_the_slide = Vec::new();
        let mut slid = 0;
        while !world.resource::<Played>().table.over() {
            assert!(seconds < 400.0, "never over: {stages:?}");
            frame(&mut world, 1.0 / 30.0);
            seconds += 1.0 / 30.0;
            let round = sent_round(&world).expect("the round");
            if stages.last() != Some(&(round.turn, round.stage)) {
                stages.push((round.turn, round.stage));
            }
            let played = world.resource::<Played>();
            let game = played.rules.game.as_ref().expect("a game");
            for seat in 0..4 {
                if round.out[seat].is_some() {
                    assert!(played.table.seats[seat].out, "seat {seat} went in and is out: {round:?}");
                }
            }
            // The computer's bodies where the game has them, but while
            // everyone slides: then where they stood as it began, sent
            // nobody as they go, every device placing them itself.
            if game.stage() == KnockStage::Slide {
                if before_the_slide.is_empty() {
                    before_the_slide = played.table.at.clone();
                    slid += 1;
                }
                assert_eq!(played.table.at, before_the_slide, "{round:?}");
            } else {
                before_the_slide.clear();
                // Where the game has them, a hop now and then over it.
                for seat in (0..4).filter(|&seat| played.table.seats[seat].computer) {
                    let (sent, there) = (played.table.at[seat], game.at(seat).0);
                    assert_eq!(sent.xz(), there.xz(), "seat {seat}");
                    assert!(sent.y >= there.y && sent.y <= there.y + 1.7, "seat {seat}: {sent} over {there}");
                }
                // And the person's body as the server keeps it where the
                // game has them, for a hop on the spot to be a move from
                // there.
                if game.in_play_at(0) && game.stage() != KnockStage::Slide {
                    let body = played.table.seats[0].body;
                    let kept = world.get::<roundtown_net::Spot>(body).expect("the person's body").translation();
                    assert!(kept.xz().distance(game.at(0).0.xz()) < 0.02, "{kept} and {}", game.at(0).0);
                }
            }
        }
        assert!(slid > 0, "{stages:?}");
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
    fn a_skip_in_play_ends_nobody_s_aim_and_a_fall_before_the_game_is_put_back() {
        let (mut world, clients) = table(2, 3);
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
        // A skip as play begins (a tap on the board meant for the
        // countdown): nobody's aim is over, and nobody is ready by it.
        act(&mut world, |rules, ctx, table| rules.hear(ctx, table, 0, &Ask::Skip));
        frame(&mut world, 1.0 / 60.0);
        let round = sent_round(&world).expect("the round");
        assert_eq!(round.stage, KnockStage::Aim);
        assert!(!round.ready[0] && !round.ready[1], "{round:?}");
        // Ready, the game's own, from both: over at once.
        act(&mut world, |rules, ctx, table| {
            rules.hear(ctx, table, 0, &Ask::KnockOff(knock_off::Ask::Ready));
            rules.hear(ctx, table, 1, &Ask::KnockOff(knock_off::Ask::Ready));
        });
        frame(&mut world, 1.0 / 60.0);
        assert_eq!(sent_round(&world).expect("the round").stage, KnockStage::Reveal);
    }

    /// Plays the table on, a tenth of a second a frame, until the person in
    /// seat 0 is aiming again, or in the sea, or it is over: the round then.
    fn to_the_next_aim(world: &mut World) -> KnockRound {
        let mut seconds = 0.0;
        loop {
            frame(world, 0.1);
            seconds += 0.1;
            let round = sent_round(world).expect("the round");
            let over = world.resource::<Played>().table.over();
            if over || round.out[0].is_some() || (round.stage == KnockStage::Aim && round.ready.first() == Some(&false)) {
                return round;
            }
            assert!(seconds < 60.0, "{round:?}");
        }
    }

    /// A table of one person and the computer, played until the person, who
    /// stands still while the computer's players knock each other about, has
    /// slid into the sea on purpose with one of them gone in before them and
    /// two still on the ice: third. From the first of the seeds from `from`
    /// to play out so; and the round as the person went in.
    fn third_with_two_on_the_ice(from: u64) -> (World, KnockRound) {
        for seed in from..from + 200 {
            let (mut world, _) = table(1, seed);
            act(&mut world, |rules, ctx, table| rules.begin(ctx, table));
            loop {
                let round = to_the_next_aim(&mut world);
                if world.resource::<Played>().table.over() || round.out[0].is_some() {
                    break;
                }
                match round.out[1..].iter().filter(|out| out.is_some()).count() {
                    // Standing still, until one of them has gone in.
                    0 => act(&mut world, |rules, ctx, table| rules.hear(ctx, table, 0, &Ask::KnockOff(knock_off::Ask::Ready))),
                    // Then off the ice, straight out from the middle, as
                    // hard as there is.
                    1 => {
                        act(&mut world, |rules, ctx, table| {
                            let out = rules.game.as_ref().expect("a game").at(0).0;
                            rules.hear(ctx, table, 0, &Ask::KnockOff(knock_off::Ask::Aim { x: out.x, z: out.z, power: knock_off::MOST_POWER }));
                            rules.hear(ctx, table, 0, &Ask::KnockOff(knock_off::Ask::Ready));
                        });
                        // Until the slide is over: the ice about to break,
                        // or the game over.
                        let mut round = sent_round(&world).expect("the round");
                        while round.stage != KnockStage::Break && !world.resource::<Played>().table.over() {
                            frame(&mut world, 0.1);
                            round = sent_round(&world).expect("the round");
                        }
                        let third = round.places[0] == Some(2) && round.places.iter().filter(|&&place| place == Some(2)).count() == 1;
                        let on_the_ice = round.out[1..].iter().filter(|out| out.is_none()).count();
                        if third && on_the_ice == 2 && !world.resource::<Played>().table.over() {
                            return (world, round);
                        }
                        break;
                    }
                    _ => break,
                }
            }
        }
        panic!("no seed from {from} put the person in third, two on the ice");
    }

    #[test]
    fn the_last_person_to_go_has_the_rest_played_at_once_and_is_paid_for_their_place() {
        // Third, two of the computer's still on the ice, and they leave (the
        // panel's Leave, another game, or their device given up): the table
        // has nobody at it, and the computer's players play the rest at
        // once, so that it is over, the person paid as they came, before
        // the table closes as it does with nobody at it ("your place kept
        // and paid", which the tables pay, `tables::run`).
        let (mut world, round) = third_with_two_on_the_ice(0);
        assert_eq!(round.stage, KnockStage::Break, "{round:?}");
        act(&mut world, |rules, ctx, table| {
            let it = &mut table.seats[0];
            it.gone = true;
            it.away = false;
            it.body = Entity::PLACEHOLDER;
            assert_eq!(table.people(), 0);
            rules.left(ctx, table, 0, None);
        });
        let played = world.resource::<Played>();
        assert!(played.table.over(), "played out and over");
        let results = played.table.results.clone().expect("how it came out");
        let (places, scores, _) = played.rules.game.as_ref().expect("the game").outcome().expect("over");
        assert_eq!((results.places.clone(), results.scores.clone()), (places, scores));
        assert_eq!(results.places[0], 2, "{results:?}");
        assert_eq!(results.prizes, vec![10, 0, 0, 0], "{results:?}");
        assert!(played.table.seats[1..].iter().filter(|it| it.out).count() >= 2, "the computer played on to the end");
    }

    #[test]
    fn every_person_in_the_sea_and_nobody_connected_a_while_has_the_rest_played_at_once() {
        // Third, two of the computer's still on the ice, the person's app
        // put away: while it is a moment, the computer's players play on at
        // their own pace (whoever comes back sees the rest); once nobody has
        // been connected a while, the rest is played at once, and the
        // person paid as they came, not lost with the table the tables give
        // up after a minute with nobody connected.
        let (mut world, round) = third_with_two_on_the_ice(0);
        let turn = round.turn;
        act(&mut world, |_, _, table| {
            table.seats[0].away = true;
            table.unwatched = UNWATCHED_PLAY_OUT * 0.5;
        });
        frame(&mut world, 0.1);
        assert!(!world.resource::<Played>().table.over());
        // Someone back: not, however long it has been.
        act(&mut world, |_, _, table| {
            table.seats[0].away = false;
            table.unwatched = 0.0;
        });
        frame(&mut world, 0.1);
        assert!(!world.resource::<Played>().table.over());
        assert_eq!(world.resource::<Played>().rules.game.as_ref().expect("a game").turn(), turn);
        act(&mut world, |_, _, table| {
            table.seats[0].away = true;
            table.unwatched = UNWATCHED_PLAY_OUT;
        });
        frame(&mut world, 0.1);
        let played = world.resource::<Played>();
        assert!(played.table.over(), "played out and over");
        let results = played.table.results.clone().expect("how it came out");
        assert_eq!((results.places[0], results.prizes[0]), (2, 10), "{results:?}");
        // With a person still on the ice, their device gone a while, it
        // plays on: they may be back, and the table's minute is theirs.
        let (mut world, _) = table(1, 4);
        act(&mut world, |rules, ctx, table| rules.begin(ctx, table));
        act(&mut world, |_, _, table| {
            table.seats[0].away = true;
            table.unwatched = UNWATCHED_PLAY_OUT * 3.0;
        });
        frame(&mut world, 0.1);
        assert!(!world.resource::<Played>().table.over());
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
            let floe = &world.resource::<Played>().rules.floe;
            for seat in 1..4 {
                let at = table.at[seat];
                assert!(floe.depth(at.xz(), 1.0) > 0.0, "seat {seat} off the ice: {at}");
                moved[seat] |= at.distance(start[seat]) > 0.5;
                for other in 0..seat {
                    assert!(at.distance(table.at[other]) > knock_off::ROOM - 0.1, "seat {seat} on seat {other}");
                }
            }
        }
        assert_eq!(moved, [false, true, true, true], "{moved:?}");
    }

    #[test]
    fn the_island_is_sent_as_the_table_opens_and_everyone_begins_where_they_stood() {
        let (mut world, clients) = table(2, 21);
        // The island, from the first frame the table is shown, before any
        // round: what every device makes its ice from.
        act(&mut world, |rules, ctx, table| {
            table.stage = Stage::Waiting { since_join: 0.0, coming: None };
            rules.show(ctx.commands, table);
        });
        world.flush();
        let (shown, floe) = {
            let played = world.resource::<Played>();
            (played.table.shown, Arc::clone(&played.rules.floe))
        };
        let island = *world.get::<Island>(shown).expect("the island is sent");
        assert_eq!(island.floe(), *floe);
        assert!(world.get::<KnockRound>(shown).is_none());
        // Seat 0 walked well in; seat 1 walked off into the sea; the
        // computer's two strolled onto the very same spot.
        let middle = floe.middle(1.0);
        let off = Vec2::X * (floe.reach_toward(Vec2::X) + 1.5);
        // Along the way out from the island's own middle to the point of
        // its outline furthest from its deepest spot: ice all the way.
        let far = floe.rim().iter().copied().max_by(|a, b| a.distance_squared(middle).total_cmp(&b.distance_squared(middle))).expect("an outline");
        let together = far * 0.6;
        {
            let table = &mut world.resource_mut::<Played>().into_inner().table;
            table.at[0] = Vec3::new(middle.x, 0.0, middle.y);
            table.at[1] = Vec3::new(off.x, -1.0, off.y);
            table.at[2] = Vec3::new(together.x, 0.0, together.y);
            table.at[3] = Vec3::new(together.x, 0.0, together.y);
            table.stage = Stage::Playing { elapsed: 0.0 };
        }
        told(&mut world);
        act(&mut world, |rules, ctx, table| rules.begin(ctx, table));
        frame(&mut world, 1.0 / 60.0);
        let round = sent_round(&world).expect("the round");
        let start: Vec<Vec2> = round.start.iter().map(|at| at.map(|(x, z)| Vec2::new(x, z)).expect("on the ice")).collect();
        if together.distance(middle) > 2.0 * knock_off::ROOM {
            assert_eq!(start[0], middle);
        }
        assert!(floe.depth(start[1], 1.0) >= STOOD_BACK * 0.97 && start[1].distance(off) < 1.5 + STOOD_BACK + 1.0, "{}", start[1]);
        assert!(start[2].distance(start[3]) >= knock_off::ROOM, "{} {}", start[2], start[3]);
        // Each person told where they begin, there.
        let said = told(&mut world);
        for seat in 0..2 {
            let placed = said.iter().find_map(|(client, tell)| match tell {
                Tell::Arrive { venue: Venue::KnockOff, spot: Some(spot), .. } if *client == clients[seat] => Some(spot.translation().xz()),
                _ => None,
            });
            // (A spot is sent to the centimetre.)
            assert!(placed.is_some_and(|placed| placed.distance(start[seat]) < 0.01), "seat {seat}: {placed:?}, {said:?}");
        }
    }

    #[test]
    fn whoever_walks_into_the_sea_before_the_game_is_stood_back_where_they_went_in() {
        let (mut world, clients) = table(2, 8);
        let floe = Arc::clone(&world.resource::<Played>().rules.floe);
        let way = Vec2::new(-0.6, 0.8);
        let off = way * (floe.reach_toward(way) + 0.8);
        {
            let table = &mut world.resource_mut::<Played>().into_inner().table;
            table.stage = Stage::Waiting { since_join: 0.0, coming: None };
            table.at[1] = Vec3::new(off.x, -0.6, off.y);
        }
        act(&mut world, |rules, ctx, table| rules.hear(ctx, table, 1, &Ask::Respawn));
        let said = told(&mut world);
        let back = said
            .iter()
            .find_map(|(client, tell)| match tell {
                Tell::Arrive { spot: Some(spot), .. } if *client == clients[1] => Some(spot.translation().xz()),
                _ => None,
            })
            .expect("stood back");
        assert!(floe.depth(back, 1.0) >= STOOD_BACK * 0.97, "{back}");
        assert!(back.distance(off) < 0.8 + STOOD_BACK + 1.0, "{off} to {back}");
    }
}

