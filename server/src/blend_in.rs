//! Blend In at its tables, on the server (since 2026-11-18; Hajun: "1 random
//! hider chosen as the person walking around pretending to be an NPC/AI. 1~3
//! other people trying to find who is the person, by punching with a aura
//! sword"): what a table of it plays (`tables::Rules`), on the town's own
//! map with nobody of the town's in it. Blend In is played online only, so
//! this is the only place it is ever played; its rules are
//! `roundtown_net::blend_in`'s ([`Game`]), and this carries them out.
//!
//! **The crowd** is the server's: [`CROWD`](blend_in::CROWD) bodies in the
//! table's room, each `(Replicated, Member, Spot, Venue::BlendIn, InRoom)`
//! as a person's is and as nobody's but a person's is, walking from the
//! moment the table opens (the crowd's mind, [`Crowd`]), so that people
//! waiting see the town as it will be played and every device sees the same
//! crowd in the same places. Each is sent where it stands as a person's
//! device sends where they stand ([`SEND_EVERY`], [`SEND_SLACK`]), not every
//! frame, and as standing still once it stops, so that nothing in how often
//! a body is sent tells the crowd's from the hider's. As each round's hide
//! begins every body in the game is dressed afresh, the crowd's and the
//! people's alike, in an order thrown each time: a new `Member`, a new id,
//! a new name, a look; a person's as they chose, with `players::Disguised`
//! beside it, and the account's own back on the body as they go (the
//! tables do that, however they go).
//!
//! **What devices are told** besides the room's [`BlendRound`]: what each
//! person wears (`Tell::Wear`), as the choosing begins, in answer to every
//! choice they make, and as the hide dresses them; that a choice was no
//! name (`Tell::Refused`); a seeker that they are stunned
//! (`Tell::Stunned`), and everyone that a seeker struck one of the town's
//! own (`Tell::Struck`), without saying which. Nothing anyone is sent says
//! which body is the hider's until the round is over (`BlendRound::
//! unmasked`).
//!
//! **The dice are secret.** Every throw of the game's and the crowd's is
//! sealed (`blend_in::Sealed`) by a seed nobody can work out: drawn here
//! from the system's own randomness ([`secret`]), never from the table's
//! seed, which is thrown by a generator meant for games and not for
//! secrets, and sent nowhere.

use std::hash::{BuildHasher, RandomState};

use bevy::prelude::*;
use roundtown_net::blend_in::{self, BlendRound, BlendStage, Crowd, Event, Game, Struck};
use roundtown_net::{Ask, Member, Spot, TICK_HZ, Tell};

use crate::players::tell;
use crate::tables::{Ctx, Rules, Table, yaw_of};

/// The least a table waits from someone sitting down at it after the
/// second person, in seconds (mine, in the games' design: "The game starts
/// 15 s after the second person joins (each later join resets that to at
/// least 8 s), or at once with four"): a third does not have the two
/// already there wait the whole of it again, nor come too late to look
/// about them; a fourth fills the table, which counts down at once.
const LATER_WAIT: f32 = 8.0;

/// How long each of the crowd's bodies goes between one sending of where it
/// stands and the next, at the least, in seconds, and how much longer at the
/// most, thrown each time (mine): a person's device says where they are no
/// more often than the room is sent anything ([`TICK_HZ`]), each time once a
/// frame of its own has gone past that, so the gap is that and up to a frame
/// more (a phone's frame, an eighth to a sixtieth of a second). The crowd's
/// are sent so, each body on a beat of its own, so that a body sent every
/// tick is no sign of the server's.
const SEND_EVERY: f32 = (1.0 / TICK_HZ) as f32;
const SEND_SLACK: f32 = 1.0 / 60.0;

/// A seed nobody can work out, for the sealed dice (`blend_in::Sealed`):
/// SipHash keyed from the system's own randomness (std's [`RandomState`],
/// whose keys come from the operating system), over `salt` and the time.
/// Never the table's own seed, which is wyrand's, and never sent.
fn secret(salt: u64) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    RandomState::new().hash_one((salt, now))
}

/// A game of Blend In at its table: the crowd walking while people wait,
/// the game once the play begins, and the bodies it has the crowd walk in.
pub(crate) struct BlendIn {
    /// The secret every throw of the game's and the crowd's is sealed by.
    seed: u64,
    /// The crowd, walking from the moment the table opens until the play
    /// begins, when the game takes it over.
    crowd: Option<Crowd>,
    game: Option<Game>,
    /// The crowd's bodies, one to each of the crowd, by its number.
    bodies: Vec<Entity>,
    /// How long until each of them is next sent where it stands, in
    /// seconds.
    sends: Vec<f32>,
    /// The dice for what is no secret: how long each waits to be sent, the
    /// order bodies are dressed in. Not the sealed ones' seed.
    dice: fastrand::Rng,
    /// What the room was last sent of the game, to send again only what has
    /// changed.
    sent: Option<BlendRound>,
}

impl BlendIn {
    /// The game at a new table, its dice thrown from secrets of its own
    /// ([`secret`]): the table's `seed` only salts them.
    pub(crate) fn new(table: &Table, seed: u64) -> Self {
        Self::seeded(secret(seed ^ u64::from(table.id)), secret(seed.rotate_left(17)))
    }

    /// The game with its secret `seed` and the plain dice's `plain` given:
    /// so that a test plays the same each time.
    fn seeded(seed: u64, plain: u64) -> Self {
        Self {
            seed,
            crowd: None,
            game: None,
            bodies: Vec::new(),
            sends: Vec::new(),
            dice: fastrand::Rng::with_seed(plain),
            sent: None,
        }
    }

    /// The crowd's walkers as they stand now: the game's, once it is played,
    /// and the crowd's own while people wait.
    fn walkers(&self) -> &[blend_in::Walker] {
        match (&self.game, &self.crowd) {
            (Some(game), _) => game.crowd().walkers(),
            (None, Some(crowd)) => crowd.walkers(),
            (None, None) => &[],
        }
    }

    /// The crowd's bodies, put up in the table's room where each of the
    /// crowd stands, as who each goes by, if they are not up yet: each first
    /// sent a thrown moment from now, so that they are not all sent in one
    /// frame.
    fn put_up_the_crowd(&mut self, ctx: &mut Ctx, table: &Table) {
        if !self.bodies.is_empty() {
            return;
        }
        let crowd = self.crowd.get_or_insert_with(|| Crowd::new(self.seed));
        let walkers: Vec<(Member, Vec3, f32)> =
            crowd.walkers().iter().map(|walker| (walker.guise.member(), walker.at, walker.yaw)).collect();
        for (member, at, yaw) in walkers {
            self.bodies.push(ctx.spawn_body(table, member, at, yaw));
            self.sends.push(self.dice.f32() * SEND_EVERY);
        }
        info!("Blend In table {}: the crowd is up, {} of the town's people", table.id, self.bodies.len());
    }

    /// Sends each of the crowd's bodies where it stands, when its time to
    /// be sent has come round, `dt` seconds on: as standing still once it
    /// has stopped (`Ctx::move_body`).
    fn send_the_crowd(&mut self, ctx: &mut Ctx, dt: f32) {
        let walkers: Vec<(Vec3, f32)> = self.walkers().iter().map(|walker| (walker.at, walker.yaw)).collect();
        for (index, (at, yaw)) in walkers.into_iter().enumerate() {
            let (Some(&body), Some(send)) = (self.bodies.get(index), self.sends.get_mut(index)) else {
                continue;
            };
            *send -= dt;
            if *send > 0.0 {
                continue;
            }
            // From when it was due, not from now: the server's frames are a
            // sixtieth of a second, so that one sending comes four of them
            // after the last and the next five, as a device's arrive.
            *send = (*send + SEND_EVERY + self.dice.f32() * SEND_SLACK).max(0.0);
            ctx.move_body(body, at, yaw);
        }
    }

    /// Sends each of the crowd's bodies as standing still where it stands:
    /// the game is over, and nothing will move it again.
    fn still_the_crowd(&mut self, ctx: &mut Ctx) {
        let walkers: Vec<(Vec3, f32)> = self.walkers().iter().map(|walker| (walker.at, walker.yaw)).collect();
        for (&body, (at, yaw)) in self.bodies.iter().zip(walkers) {
            let Ok((_, mut spot, ..)) = ctx.bodies.get_mut(body) else {
                continue;
            };
            let mut still = Spot::of(at, Quat::from_rotation_y(yaw));
            still.still = true;
            if !still.same(&spot) {
                *spot = still;
            }
        }
    }

    /// Carries out what happened in the game.
    fn carry_out(&mut self, ctx: &mut Ctx, table: &mut Table, event: Event) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let id = table.id;
        match event {
            Event::Choosing { round, hider } => {
                info!("Blend In table {id}: round {} of {}, seat {hider} hides", round + 1, game.round().rounds);
                for seat in 0..table.seats.len() {
                    wear(ctx, table, game, seat);
                }
            }
            Event::Refused { seat } => {
                ctx.tell(table, seat, Tell::BlendIn(blend_in::Tell::Refused));
            }
            // Told what they wear, with everyone else, as they are dressed.
            Event::Renamed { seat } => {
                info!("Blend In table {id}: seat {seat} hides under another name, a seeker having taken theirs");
            }
            Event::Dressed => self.dress(ctx, table),
            Event::Place { seat, at, yaw } => ctx.place(table, seat, at, yaw),
            Event::Seeking => info!("Blend In table {id}: the seek"),
            Event::Wrong { seeker, crowd: _, until } => {
                let stun = until - ctx.clock;
                info!("Blend In table {id}: seat {seeker} struck one of the town's own, stunned {stun:.1} s");
                ctx.tell(table, seeker, Tell::BlendIn(blend_in::Tell::Stunned { until }));
                say_to_everyone(ctx, table, Tell::BlendIn(blend_in::Tell::Struck { seeker: seeker as u8 }));
            }
            Event::Found { seeker, hider, after } => {
                info!("Blend In table {id}: seat {seeker} found seat {hider} {after:.1} s into the seek");
            }
            Event::Hidden { hider } => info!("Blend In table {id}: seat {hider} stayed hidden"),
            Event::Void { hider } => info!("Blend In table {id}: seat {hider}'s round is given up"),
            Event::Over => {
                if let Some((places, scores, total)) = game.outcome() {
                    info!("Blend In table {id}: over, points {scores:?}");
                    table.finish(places, scores, total);
                }
                self.still_the_crowd(ctx);
            }
        }
    }

    /// The hide has begun: every body in the game dressed as the game has it
    /// now, the crowd's and the people's, in an order thrown afresh, so that
    /// nothing in the order they change in says whose is whose; and each
    /// person told what they wear.
    fn dress(&mut self, ctx: &mut Ctx, table: &mut Table) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let mut dressing: Vec<(Option<usize>, Entity, Member)> = Vec::new();
        for (index, walker) in game.crowd().walkers().iter().enumerate() {
            if let Some(&body) = self.bodies.get(index) {
                dressing.push((None, body, walker.guise.member()));
            }
        }
        for figure in game.bodies() {
            if let blend_in::Who::Seat(seat) = figure.who {
                dressing.push((Some(seat), Entity::PLACEHOLDER, figure.guise.member()));
            }
        }
        self.dice.shuffle(&mut dressing);
        for (seat, body, member) in dressing {
            match seat {
                Some(seat) => ctx.disguise(table, seat, member),
                None => {
                    ctx.commands.entity(body).insert(member);
                }
            }
        }
        for seat in 0..table.seats.len() {
            wear(ctx, table, game, seat);
        }
        info!("Blend In table {}: everyone dressed for the hide", table.id);
    }
}

/// Tells the person in `seat`, if they are here, what they wear, or will
/// from the coming hide (`Tell::Wear`).
fn wear(ctx: &mut Ctx, table: &mut Table, game: &Game, seat: usize) {
    if !table.seats.get(seat).is_some_and(|it| it.here()) {
        return;
    }
    if let Some((name, look)) = game.choice(seat) {
        let told = Tell::BlendIn(blend_in::Tell::Wear {
            name: name.to_owned(),
            look: look.to_owned(),
        });
        ctx.tell(table, seat, told);
    }
}

/// Tells every person at the table whose device is here `told`, as the
/// chat is told what is said: nothing for a device that has gone, which
/// would otherwise have it begin the game afresh on coming back (`Ctx::
/// tell`) for a line of the chat.
fn say_to_everyone(ctx: &mut Ctx, table: &Table, told: Tell) {
    for it in &table.seats {
        if !it.here() {
            continue;
        }
        let Ok((body, ..)) = ctx.bodies.get(it.body) else {
            continue;
        };
        if body.room != table.room {
            continue;
        }
        if let Some(client) = body.client {
            tell(ctx.tells, client, told.clone());
        }
    }
}

/// Where the person in `seat` would arrive in the town, on the path round
/// its pond, on its ground, and which way they face: where they wait for
/// the game, and where the seekers stand through each hide.
fn came_in(seat: usize) -> (Vec3, f32) {
    let (at, turn) = blend_in::arrival(seat);
    let ground = blend_in::grid().height(at.xz()).unwrap_or(at.y);
    (at.with_y(ground), yaw_of(turn))
}

impl Rules for BlendIn {
    /// Where they would arrive in the town, on the path round its pond
    /// ([`came_in`]).
    fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
        came_in(seat)
    }

    /// The whole wait (15 s) from the second person, with whom the game can
    /// begin; from each after them, what was left of it, but never under
    /// [`LATER_WAIT`].
    fn wait_from_join(&self, table: &Table, left: Option<f32>) -> f32 {
        left.map_or(table.shape.wait, |left| left.max(LATER_WAIT))
    }

    /// The crowd walks while people wait for the game, and through its
    /// countdown: up as the table opens.
    fn idle(&mut self, ctx: &mut Ctx, table: &mut Table, dt: f32) {
        self.put_up_the_crowd(ctx, table);
        let people: Vec<Vec2> = (0..table.seats.len())
            .filter(|&seat| table.seats[seat].here())
            .map(|seat| table.at[seat].xz())
            .collect();
        if let Some(crowd) = self.crowd.as_mut() {
            crowd.step(dt, &people);
        }
        self.send_the_crowd(ctx, dt);
    }

    /// The play begins: the game of the people seated, as their accounts
    /// have them, among the crowd that has been walking; its first round's
    /// choosing; and everyone stood where they came in, on the path round
    /// the pond.
    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        self.put_up_the_crowd(ctx, table);
        let members: Vec<Member> = table.seats.iter().map(|it| it.member.clone()).collect();
        let crowd = self.crowd.take().unwrap_or_else(|| Crowd::new(self.seed));
        let mut game = Game::new(&members, self.seed, crowd);
        for (seat, it) in table.seats.iter().enumerate() {
            if !it.here() {
                game.leave(seat);
            } else if it.away {
                game.away(seat, true);
            }
        }
        game.begin(ctx.clock);
        for seat in 0..table.seats.len() {
            let (at, yaw) = game.at(seat);
            ctx.place(table, seat, at, yaw);
        }
        self.game = Some(game);
    }

    /// A frame of the game: where everyone is, and whose device has gone,
    /// into it; it moved on by the frame and the server's clock; what
    /// happened carried out; the crowd sent where it walks.
    fn step(&mut self, ctx: &mut Ctx, table: &mut Table, dt: f32) {
        let Some(game) = self.game.as_mut() else {
            return;
        };
        for seat in 0..table.seats.len() {
            let it = &table.seats[seat];
            if it.here() {
                game.sync(seat, table.at[seat], table.yaw[seat]);
                game.away(seat, it.away);
            }
        }
        let events = game.step(dt, ctx.clock);
        for event in events {
            self.carry_out(ctx, table, event);
        }
        if table.playing() {
            self.send_the_crowd(ctx, dt);
        }
    }

    /// What the people seated ask of the game: a look and a name, answered
    /// with what they wear as it stands; that they have chosen; a swing
    /// that landed on a body, a strike ([`Game::strike`]); for trying the
    /// game out, its time up now (`RT_ALLOW_SKIP`, [`Game::hurry`]); and,
    /// fallen out of the world, to be stood back where they were.
    fn hear(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize, ask: &Ask) {
        if self.game.is_none() {
            if let Ask::Respawn = ask {
                let (at, yaw) = came_in(seat);
                ctx.place(table, seat, at, yaw);
            }
            return;
        }
        let Some(game) = self.game.as_mut() else {
            return;
        };
        match ask {
            Ask::BlendIn(blend_in::Ask::Disguise { name, look }) => {
                game.disguise(seat, name, look);
                wear(ctx, table, game, seat);
            }
            Ask::BlendIn(blend_in::Ask::Ready) => {
                game.ready(seat);
            }
            Ask::Punch { who, .. } => {
                // Where the device has the seeker now, as its last move
                // said, rather than as the table last copied it in.
                if let Some(spot) = ctx.spot(table, seat) {
                    game.sync(seat, spot.translation(), spot.yaw());
                }
                match game.strike(seat, *who) {
                    Struck::Nothing => debug!("Blend In table {}: seat {seat}'s swing came to nothing", table.id),
                    Struck::Found => debug!("Blend In table {}: seat {seat}'s swing found the hider", table.id),
                    Struck::Wrong { stun } => {
                        debug!("Blend In table {}: seat {seat}'s swing was a wrong guess, {stun} s", table.id);
                    }
                }
            }
            Ask::Skip => game.hurry(),
            Ask::Respawn => {
                let (at, yaw) = game.at(seat);
                let ground = game.crowd().grid().height(at.xz()).unwrap_or(at.y);
                ctx.place(table, seat, at.with_y(ground), yaw);
            }
            _ => {}
        }
    }

    /// Whoever has left for good gives up their turn to hide, or their
    /// round if it is theirs; with fewer than two people left the game is
    /// over. What the game had them pass for is taken off their body by the
    /// table, however they left.
    fn left(&mut self, _ctx: &mut Ctx, table: &mut Table, seat: usize, _body: Option<Entity>) {
        if let Some(game) = self.game.as_mut() {
            info!("Blend In table {}: seat {seat} has left the game", table.id);
            game.leave(seat);
        }
    }

    /// A device back in the game, begun afresh on it: what it wears, or
    /// will, and that it is stunned, if it is.
    fn back(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        wear(ctx, table, game, seat);
        if let Some(until) = game.stunned(seat) {
            ctx.tell(table, seat, Tell::BlendIn(blend_in::Tell::Stunned { until }));
        }
    }

    /// The round as everyone sees it ([`BlendRound`]), on the table's own
    /// entity, sent again only when it has changed.
    fn show(&mut self, commands: &mut Commands, table: &Table) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let round = game.round();
        if self.sent.as_ref() != Some(&round) {
            if round.stage != self.sent.as_ref().map_or(BlendStage::Over, |sent| sent.stage) {
                debug!("Blend In table {}: {:?} until {:.1}", table.id, round.stage, round.until);
            }
            commands.entity(table.shown).insert(round.clone());
            self.sent = Some(round);
        }
    }

    /// The crowd's bodies go with the table. Whatever the game had the
    /// people pass for, the table takes off them as they go.
    fn close(&mut self, commands: &mut Commands, _table: &Table) {
        for body in self.bodies.drain(..) {
            commands.entity(body).try_despawn();
        }
        self.sends.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use bevy::ecs::message::Messages;
    use bevy::ecs::system::{RunSystemOnce, SystemId, SystemParam};
    use bevy_replicon::prelude::*;
    use roundtown_net::blend_in::{CROWD, FIRST_ID, HIDER_CLEAR, IDS, SEEK, SHOWN};
    use roundtown_net::{Minigame, Venue};

    use super::*;
    use crate::Config;
    use crate::players::{Account, Accounts, Body, Disguised, Who};
    use crate::rooms::{InRoom, Room};
    use crate::tables::{Bodies, Stage};

    const ROOM: Room = Room::Table(Minigame::BlendIn, 0);
    /// The server's clock as a test's table opens.
    const CLOCK: f64 = 1_794_000_000.0;
    /// A server's frame.
    const DT: f32 = 1.0 / 60.0;

    /// What a game is handed to act on the world with, as the tables'
    /// own systems hand it.
    #[derive(SystemParam)]
    struct Hands<'w, 's> {
        commands: Commands<'w, 's>,
        tells: MessageWriter<'w, ToClients<Tell>>,
        bodies: Bodies<'w, 's>,
        accounts: Accounts<'w, 's>,
    }

    /// A table of Blend In with no network: its table, its game, the
    /// clocks, and what its people ask this frame.
    #[derive(Resource)]
    struct Harness {
        table: Table,
        game: BlendIn,
        now: f64,
        clock: f64,
        asks: Vec<(usize, Ask)>,
        begin: bool,
    }

    /// A frame of the table, as the tables run one: what was asked heard,
    /// where everyone is copied in, the game idle, begun or stepped, and
    /// what the room is sent.
    fn frame(mut hands: Hands, config: Res<Config>, mut harness: ResMut<Harness>) {
        let harness = &mut *harness;
        let mut ctx = Ctx {
            commands: &mut hands.commands,
            tells: &mut hands.tells,
            bodies: &mut hands.bodies,
            accounts: &hands.accounts,
            now: harness.now,
            clock: harness.clock,
            config: &config,
        };
        let (table, game) = (&mut harness.table, &mut harness.game);
        for (seat, ask) in std::mem::take(&mut harness.asks) {
            game.hear(&mut ctx, table, seat, &ask);
        }
        for seat in 0..table.seats.len() {
            if let Some(spot) = ctx.spot(table, seat) {
                table.at[seat] = spot.translation();
                table.yaw[seat] = spot.yaw();
            }
        }
        if std::mem::take(&mut harness.begin) {
            table.stage = Stage::Playing { elapsed: 0.0 };
            game.begin(&mut ctx, table);
        } else if table.playing() {
            game.step(&mut ctx, table, DT);
        } else if !table.over() {
            game.idle(&mut ctx, table, DT);
        }
        game.show(ctx.commands, table);
    }

    struct Server {
        world: World,
        frame: SystemId,
        /// Each person's device and body, by seat.
        people: Vec<(Entity, Entity)>,
        /// Everything every device was told, in order.
        told: Vec<(Entity, Tell)>,
    }

    impl Server {
        /// A table of Blend In whose crowd is up, `people` seated at it
        /// waiting, each of them an account of their own.
        fn new(people: usize, seed: u64) -> Self {
            let mut world = World::new();
            world.insert_resource(Time::<()>::default());
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
            let shown = world.spawn((Replicated, InRoom(ROOM))).id();
            let mut table = Table::new(Minigame::BlendIn, 0, shown, seed).expect("Blend In is played at tables");
            let mut seated = Vec::new();
            for seat in 0..people {
                let client = world.spawn_empty().id();
                let member = Member {
                    id: seat as u64 + 1,
                    name: ["Dubu", "Mochi", "Hodu", "Kong"][seat].into(),
                    look: "boy".into(),
                    ai: false,
                };
                let (at, turn) = blend_in::arrival(seat);
                let body = world
                    .spawn((
                        Replicated,
                        member.clone(),
                        Account(member.clone()),
                        Spot::of(at, turn),
                        Venue::BlendIn,
                        InRoom(ROOM),
                        Body::person(client, Who(member.id), ROOM, seat as u8, None, 1000, 0.0),
                    ))
                    .id();
                let it = &mut table.seats[seat];
                it.body = body;
                it.member = member;
                it.vacant = false;
                it.gone = false;
                it.paid = 50;
                seated.push((client, body));
            }
            // As the tables have it once a game of people alone begins:
            // as many seats as people, numbered from 0.
            table.seats.truncate(people);
            table.at.truncate(people);
            table.yaw.truncate(people);
            table.size = people;
            let mut game = BlendIn::seeded(seed, seed.wrapping_add(1));
            for seat in 0..people {
                let (at, yaw) = game.waiting_spot(&table, seat);
                table.at[seat] = at;
                table.yaw[seat] = yaw;
            }
            world.insert_resource(Harness {
                table,
                game,
                now: 0.0,
                clock: CLOCK,
                asks: Vec::new(),
                begin: false,
            });
            let frame = world.register_system(frame);
            let mut server = Self {
                world,
                frame,
                people: seated,
                told: Vec::new(),
            };
            server.frame();
            server
        }

        fn harness(&self) -> &Harness {
            self.world.resource::<Harness>()
        }

        fn harness_mut(&mut self) -> Mut<'_, Harness> {
            self.world.resource_mut::<Harness>()
        }

        /// One frame, a sixtieth of a second on by both clocks.
        fn frame(&mut self) {
            {
                let mut harness = self.harness_mut();
                harness.now += f64::from(DT);
                harness.clock += f64::from(DT);
            }
            self.world.resource_mut::<Time>().advance_by(Duration::from_secs_f32(DT));
            self.world.run_system(self.frame).expect("the table's frame");
            let told: Vec<ToClients<Tell>> = self.world.resource_mut::<Messages<ToClients<Tell>>>().drain().collect();
            for told in told {
                if let SendTargets::Single(ClientId::Client(client)) = told.targets {
                    self.told.push((client, told.message));
                }
            }
        }

        fn frames(&mut self, secs: f32) {
            for _ in 0..(secs / DT).round() as usize {
                self.frame();
            }
        }

        /// Frames until `until` holds, for at most `most` seconds.
        fn until(&mut self, most: f32, until: impl Fn(&Self) -> bool) {
            let mut took = 0.0;
            while !until(self) {
                assert!(took < most, "not within {most} s");
                self.frame();
                took += DT;
            }
        }

        fn ask(&mut self, seat: usize, ask: Ask) {
            self.harness_mut().asks.push((seat, ask));
        }

        fn round(&self) -> BlendRound {
            let shown = self.harness().table.shown;
            self.world.get::<BlendRound>(shown).expect("the round, on the table's entity").clone()
        }

        fn stage(&self) -> BlendStage {
            self.round().stage
        }

        /// Every body of the crowd's.
        fn crowd(&self) -> Vec<Entity> {
            self.harness().game.bodies.clone()
        }

        /// Everything `seat`'s device was told of the game, in order.
        fn heard(&self, seat: usize) -> Vec<blend_in::Tell> {
            let client = self.people[seat].0;
            self.told
                .iter()
                .filter(|(to, _)| *to == client)
                .filter_map(|(_, told)| match told {
                    Tell::BlendIn(told) => Some(told.clone()),
                    _ => None,
                })
                .collect()
        }

        /// Where `seat`'s device was last stood afresh.
        fn placed(&self, seat: usize) -> Option<Spot> {
            let client = self.people[seat].0;
            self.told.iter().rev().find_map(|(to, told)| match told {
                Tell::Arrive { spot, .. } if *to == client => *spot,
                _ => None,
            })
        }

        /// Moves `seat`'s body to `at`, as its device's move would.
        fn walk(&mut self, seat: usize, at: Vec3) {
            let body = self.people[seat].1;
            *self.world.get_mut::<Spot>(body).expect("a body") = Spot::of(at, Quat::IDENTITY);
        }

        fn member(&self, body: Entity) -> Member {
            self.world.get::<Member>(body).expect("a member").clone()
        }

        /// Everyone at the table ready: the hide.
        fn all_ready(&mut self) {
            for seat in 0..self.people.len() {
                self.ask(seat, Ask::BlendIn(blend_in::Ask::Ready));
            }
            self.until(1.0, |server| server.stage() == BlendStage::Hide);
        }
    }

    #[test]
    fn the_crowd_walks_from_the_moment_the_table_opens_sent_as_people_are() {
        let mut server = Server::new(2, 3);
        let crowd = server.crowd();
        assert_eq!(crowd.len(), CROWD);
        let mut ids = Vec::new();
        for &body in &crowd {
            let member = server.member(body);
            // As a person is sent, and as nobody's but a person's is.
            assert!(!member.ai, "{member:?}");
            assert!((FIRST_ID..FIRST_ID + IDS).contains(&member.id));
            let named = [roundtown_net::BOY_NAMES, roundtown_net::GIRL_NAMES]
                .iter()
                .any(|names| names.contains(&member.name.as_str()));
            assert!(named, "{member:?}");
            assert_eq!(server.world.get::<Venue>(body), Some(&Venue::BlendIn));
            assert_eq!(server.world.get::<InRoom>(body), Some(&InRoom(ROOM)));
            assert!(server.world.get::<Spot>(body).is_some());
            ids.push(member.id);
        }
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), CROWD);
        // A minute of waiting: they walk, and each is sent where it is no
        // more often than a person's device sends where they are, a fifteenth
        // of a second apart at the least.
        // Each body's sendings from its first on: until then, whenever it
        // was put up.
        let mut was: HashMap<Entity, (Spot, f32)> = crowd
            .iter()
            .map(|&body| (body, (*server.world.get::<Spot>(body).unwrap(), f32::NEG_INFINITY)))
            .collect();
        let (mut sent, mut moved) = (0, 0);
        let mut gaps = Vec::new();
        let mut clock = 0.0;
        for _ in 0..(60.0 / DT) as usize {
            server.frame();
            clock += DT;
            for &body in &crowd {
                let now = *server.world.get::<Spot>(body).unwrap();
                let (then, when) = was[&body];
                if !now.same(&then) {
                    sent += 1;
                    if now.translation() != then.translation() {
                        moved += 1;
                    }
                    gaps.push(clock - when);
                    was.insert(body, (now, clock));
                }
            }
        }
        let shortest = gaps.iter().copied().fold(f32::INFINITY, f32::min);
        let finite: Vec<f32> = gaps.iter().copied().filter(|gap| gap.is_finite() && *gap < 0.1).collect();
        let mean = finite.iter().sum::<f32>() / finite.len().max(1) as f32;
        println!(
            "{sent} sendings of the crowd in a minute, {moved} of them moving; shortest gap {shortest:.3} s, \
             {mean:.3} s on average between sendings a step apart"
        );
        assert!(shortest >= SEND_EVERY - DT * 0.5, "{shortest}");
        // Not every tick of the room's, as a body written every frame would
        // be: four of the server's frames apart, and five.
        assert!(mean > SEND_EVERY + DT * 0.2 && mean < SEND_EVERY + DT * 0.8, "{mean}");
        assert!(moved > CROWD * 20, "they walk: {moved}");
        // Standing still once stopped: the last sending of each standing
        // body says so.
        let standing = crowd
            .iter()
            .filter(|&&body| server.world.get::<Spot>(body).unwrap().still)
            .count();
        assert!(standing > CROWD / 3, "{standing} of them sent as standing");
    }

    #[test]
    fn a_table_of_two_is_played_whole_through_its_rules() {
        let mut server = Server::new(2, 5);
        let crowd = server.crowd();
        server.harness_mut().begin = true;
        server.frame();
        assert_eq!(server.stage(), BlendStage::Choose);
        server.frame();
        // Each told what they wear unless they choose otherwise: the hider a
        // crowd's guise, the seeker their own account.
        let hider = usize::from(server.round().hider);
        let seeker = 1 - hider;
        let offered = |server: &Server, seat: usize| -> (String, String) {
            server
                .heard(seat)
                .iter()
                .rev()
                .find_map(|told| match told {
                    blend_in::Tell::Wear { name, look } => Some((name.clone(), look.clone())),
                    _ => None,
                })
                .expect("told what they wear")
        };
        assert_eq!(offered(&server, seeker).0, ["Dubu", "Mochi"][seeker]);
        // A choice is answered with what is worn as it stands: a name, and
        // no name, refused, and what it was kept.
        server.ask(seeker, Ask::BlendIn(blend_in::Ask::Disguise { name: "Sol".into(), look: "girl".into() }));
        server.frame();
        assert_eq!(offered(&server, seeker), ("Sol".into(), "girl".into()));
        server.ask(seeker, Ask::BlendIn(blend_in::Ask::Disguise { name: "x".into(), look: "boy".into() }));
        server.frames(0.05);
        assert_eq!(offered(&server, seeker), ("Sol".into(), "girl".into()));
        assert!(server.heard(seeker).contains(&blend_in::Tell::Refused));
        // Ready: the hide. Everyone dressed afresh, crowd and people alike.
        let before: Vec<Member> = crowd.iter().map(|&body| server.member(body)).collect();
        server.all_ready();
        server.frame();
        let mut ids = Vec::new();
        for (index, &body) in crowd.iter().enumerate() {
            let member = server.member(body);
            assert_ne!(member.id, before[index].id, "a new id");
            assert!(!member.ai);
            ids.push(member.id);
        }
        for seat in 0..2 {
            let body = server.people[seat].1;
            let member = server.member(body);
            assert!(server.world.get::<Disguised>(body).is_some());
            assert!(!member.ai);
            assert!((FIRST_ID..FIRST_ID + IDS).contains(&member.id), "{member:?}");
            assert_eq!((member.name.clone(), member.look.clone()), offered(&server, seat));
            ids.push(member.id);
        }
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), CROWD + 2, "no two bodies alike");
        // The seeker's sword is sent, never the hider's body.
        let round = server.round();
        let (hider_body, seeker_body) = (server.people[hider].1, server.people[seeker].1);
        let hider_id = server.member(hider_body).id;
        assert_eq!(round.swords[seeker], server.member(seeker_body).id);
        assert_eq!(round.swords[hider], 0);
        assert_eq!(round.unmasked, None);
        // The hider is stood among the crowd far from the seeker, who waits
        // where they came in.
        let hidden = server.placed(hider).expect("the hider stood somewhere").translation();
        let waits = server.placed(seeker).expect("the seeker stood where they came in").translation();
        assert!(hidden.xz().distance(waits.xz()) >= HIDER_CLEAR, "{hidden} from {waits}");
        server.walk(hider, hidden);
        server.walk(seeker, waits);
        // The seek: a swing on one of the crowd is a wrong guess, stunning
        // the seeker, said to everyone, and the one struck staggers.
        server.until(16.0, |server| server.stage() == BlendStage::Seek);
        let crowd_spots: Vec<Vec3> = crowd.iter().map(|&body| server.world.get::<Spot>(body).unwrap().translation()).collect();
        let (struck, near) = crowd
            .iter()
            .zip(&crowd_spots)
            .min_by(|a, b| a.1.distance(waits).total_cmp(&b.1.distance(waits)))
            .map(|(&body, &at)| (body, at))
            .unwrap();
        server.walk(seeker, near + Vec3::X);
        let struck_id = server.member(struck).id;
        server.ask(seeker, Ask::Punch { who: struck_id, running: false });
        server.frames(0.6);
        assert!(server.heard(seeker).iter().any(|told| matches!(told, blend_in::Tell::Stunned { .. })));
        for seat in 0..2 {
            assert!(server.heard(seat).contains(&blend_in::Tell::Struck { seeker: seeker as u8 }), "seat {seat}");
        }
        let staggered = server.world.get::<Spot>(struck).unwrap().translation();
        assert!(staggered.xz().distance(near.xz()) > 0.4, "{near} to {staggered}");
        // Stunned, a swing comes to nothing; the stun over, one on the
        // hider's body finds them.
        server.walk(seeker, hidden + Vec3::X);
        server.ask(seeker, Ask::Punch { who: hider_id, running: false });
        server.frames(0.1);
        assert_eq!(server.stage(), BlendStage::Seek, "stunned");
        server.frames(4.0);
        server.walk(seeker, hidden + Vec3::X);
        server.ask(seeker, Ask::Punch { who: hider_id, running: false });
        server.frames(0.1);
        let round = server.round();
        assert_eq!(round.stage, BlendStage::Shown);
        assert_eq!(round.found.map(|(by, _)| usize::from(by)), Some(seeker));
        assert_eq!(round.unmasked, Some(hider_id));
        assert!(round.swords.is_empty());
        // The second round: the finder hides, and nobody finds them before
        // the time is up: their point, and two to none, the first's 100
        // coins theirs.
        server.frames(SHOWN + 0.1);
        assert_eq!(server.round().round, 1);
        assert_eq!(usize::from(server.round().hider), seeker);
        server.all_ready();
        server.until(16.0, |server| server.stage() == BlendStage::Seek);
        server.until(SEEK + 1.0, |server| server.stage() == BlendStage::Shown);
        assert_eq!(server.round().found, None);
        server.until(SHOWN + 1.0, |server| server.harness().table.over());
        let results = server.harness().table.results.clone().expect("how it came out");
        let mut scores = vec![0; 2];
        scores[seeker] = 2;
        assert_eq!(results.scores, scores);
        assert_eq!((results.places[seeker], results.places[hider]), (0, 1));
        assert_eq!((results.prizes[seeker], results.prizes[hider]), (100, 0));
        assert_eq!(server.stage(), BlendStage::Over);
        // Every crowd body sent as standing still, nothing to move it.
        assert!(crowd.iter().all(|&body| server.world.get::<Spot>(body).unwrap().still));
        // The table closes: the crowd goes with it.
        {
            let world = &mut server.world;
            world
                .run_system_once(|mut commands: Commands, mut harness: ResMut<Harness>| {
                    let harness = &mut *harness;
                    harness.game.close(&mut commands, &harness.table);
                })
                .expect("the close");
        }
        assert!(crowd.iter().all(|&body| server.world.get_entity(body).is_err()));
    }

    #[test]
    fn a_hider_who_leaves_gives_up_their_round_and_a_device_back_is_told_what_it_wears() {
        let mut server = Server::new(3, 9);
        server.harness_mut().begin = true;
        server.frame();
        server.frame();
        let hider = usize::from(server.round().hider);
        server.all_ready();
        // A device back in the game is told what it wears again.
        let other = (hider + 1) % 3;
        let told = server.heard(other).len();
        {
            let world = &mut server.world;
            world
                .run_system_once(move |mut hands: Hands, config: Res<Config>, mut harness: ResMut<Harness>| {
                    let harness = &mut *harness;
                    let mut ctx = Ctx {
                        commands: &mut hands.commands,
                        tells: &mut hands.tells,
                        bodies: &mut hands.bodies,
                        accounts: &hands.accounts,
                        now: harness.now,
                        clock: harness.clock,
                        config: &config,
                    };
                    harness.game.back(&mut ctx, &mut harness.table, other);
                })
                .expect("the device back");
        }
        server.frame();
        assert!(matches!(server.heard(other)[told..], [blend_in::Tell::Wear { .. }]), "{:?}", &server.heard(other)[told..]);
        // The hider leaves: their round is given up, and the next begins.
        {
            let world = &mut server.world;
            world
                .run_system_once(move |mut hands: Hands, config: Res<Config>, mut harness: ResMut<Harness>| {
                    let harness = &mut *harness;
                    let mut ctx = Ctx {
                        commands: &mut hands.commands,
                        tells: &mut hands.tells,
                        bodies: &mut hands.bodies,
                        accounts: &hands.accounts,
                        now: harness.now,
                        clock: harness.clock,
                        config: &config,
                    };
                    harness.table.seats[hider].gone = true;
                    harness.game.left(&mut ctx, &mut harness.table, hider, None);
                })
                .expect("the hider gone");
        }
        server.frame();
        let round = server.round();
        assert_eq!(round.stage, BlendStage::Choose);
        assert_eq!(round.round, 1);
        assert_eq!(round.points, vec![0, 0, 0], "no point for a round given up");
        assert_ne!(usize::from(round.hider), hider);
    }

    #[test]
    fn a_tap_on_the_board_hurries_the_round_on_for_trying_it_out() {
        let mut server = Server::new(2, 11);
        server.harness_mut().begin = true;
        server.frame();
        let hider = usize::from(server.round().hider);
        for stage in [BlendStage::Hide, BlendStage::Seek, BlendStage::Shown] {
            server.ask(0, Ask::Skip);
            server.until(0.5, |server| server.stage() == stage);
        }
        assert_eq!(server.round().points[hider], 1, "the seek run out: the hider's point");
    }
}
