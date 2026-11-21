//! Bags: what each person owns, what they buy and sell, and what of it
//! stands on their plot of the town (since 2026-11-21; Hajun: "im gonna
//! change the emote button in the lobby to a inventory/shop panel ... the
//! placed inventory, after pressing the check button, gets placed in ur plot
//! and saved so each time u reopen the app it stays there and other ppl can
//! see that item in ur plot in that exact location too").
//!
//! The rules are `roundtown_net::bag`'s, which the game plays by itself
//! offline: here they are played with the account's own coins. A person's
//! bag is on their body for as long as that is here ([`Holds`]), read with
//! the account as they are let in (`accounts::Record`), and the game's loop
//! goes by its own copy:
//!
//! * **A purchase or a sale** changes the coins and the bag together. With
//!   a database that is one transaction on its thread (`Bank::deal`), and
//!   the body's own copy changes when it answers ([`dealt`]); whatever else
//!   that person asks of their bag meanwhile waits its turn
//!   ([`Holds::waiting`]), so that each thing is done on the bag the one
//!   before it left. Without one, it is done at once.
//! * **Standing a thing on the plot, moving it there and taking it back**
//!   change the bag alone: done at once, and written after the fact
//!   (`Bank::keep_bag`).
//!
//! **Who is told what** ([`bag::Tell`]), and only a device that has asked
//! ([`bag::Ask::Watch`], [`Watchers`]): its own bag, whole, as its body sits
//! down in the town ([`seat_news`]) and once in answer to each thing it
//! asks after, done or not, with how many of its asks that answers; and
//! what stands on the plot of everyone else in its channel, as it sits down
//! among them or they do, and whenever any of it changes. Which plot is
//! whose is the seat's, which the channel is told of already
//! (`players::tell_channels`); where on it a thing stands is in the plot's
//! own space, and the server, which has no map, only keeps it.

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::bag::{self, Bag, Standing};
use roundtown_net::{Ask, Member, Tell};
use serde::{Deserialize, Serialize};

use crate::accounts::{Bank, Dealt, Record};
use crate::players::{Body, Everyone, Plays, Who, tell};
use crate::rooms::{Room, Towns};

/// A bag as it is kept with its account (`players.bag`): what is owned, and
/// what of it stands on the plot. The gems are a column of their own.
#[derive(Serialize, Deserialize, Default)]
struct Kept {
    owned: Vec<(String, u16)>,
    stands: Vec<Standing>,
}

/// `bag` as it is kept with its account.
pub(crate) fn kept(bag: &Bag) -> String {
    let kept = Kept {
        owned: bag.owned.clone(),
        stands: bag.stands.clone(),
    };
    serde_json::to_string(&kept).unwrap_or_default()
}

/// The bag of an account with `gems` gems that keeps `kept` (`players.bag`):
/// with nothing kept, what everyone begins with; and whatever was kept
/// wrongly, or cannot be read, put right ([`Bag::sane`]).
fn read(gems: u32, kept: Option<&str>) -> Bag {
    let bag = match kept.map(serde_json::from_str::<Kept>) {
        Some(Ok(kept)) => Bag {
            gems,
            owned: kept.owned,
            stands: kept.stands,
        },
        Some(Err(error)) => {
            warn!("a bag that will not read ({error}): begun afresh");
            Bag { gems, ..Bag::new() }
        }
        None => Bag { gems, ..Bag::new() },
    };
    bag.sane()
}

/// A person's bag, on their body.
#[derive(Component)]
pub(crate) struct Holds {
    pub bag: Bag,
    /// A purchase or a sale on its way to the database.
    dealing: Option<Dealing>,
    /// What was asked of the bag while that was on its way, in order, each
    /// with the device that asked it.
    waiting: VecDeque<(Entity, bag::Ask)>,
}

/// A purchase or a sale on its way to the database: the bag as it will be
/// if it goes through, and the device that asked for it.
struct Dealing {
    after: Bag,
    by: Entity,
}

impl Holds {
    /// The bag the account `record` was loaded with.
    pub(crate) fn of(record: &Record) -> Self {
        Self::holding(read(record.gems, record.bag.as_deref()))
    }

    fn holding(bag: Bag) -> Self {
        Self {
            bag,
            dealing: None,
            waiting: VecDeque::new(),
        }
    }
}

/// The devices that have asked to be told of bags ([`bag::Ask::Watch`]):
/// those whose app knows how to read it, each with how many of its asks
/// about its bag have been answered ([`bag::Tell::Yours`]).
#[derive(Resource, Default)]
pub(crate) struct Watchers(HashMap<Entity, u32>);

/// With no database: everyone's bag for as long as the server runs, by
/// their account, so that someone who drops and comes back finds theirs.
#[derive(Resource, Default)]
struct Remembered(HashMap<u64, Bag>);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Watchers>()
        .init_resource::<Remembered>()
        .add_observer(forget)
        .add_systems(
            PreUpdate,
            (recall, dealt, hear)
                .chain()
                .after(crate::players::PlayerSystems)
                .after(crate::fees::credited)
                .run_if(in_state(ServerState::Running)),
        )
        // Once everyone seated this tick is, and the channel has been told
        // who sits where.
        .add_systems(
            PostUpdate,
            seat_news
                .after(crate::players::tell_channels)
                .run_if(in_state(ServerState::Running)),
        );
}

/// A device has gone: it is told nothing more.
fn forget(remove: On<Remove, ConnectedClient>, mut watchers: ResMut<Watchers>) {
    watchers.0.remove(&remove.entity);
}

/// With no database, gives a body just made the bag its account had when it
/// was last here.
fn recall(bank: Res<Bank>, remembered: Res<Remembered>, mut bodies: Query<(&Body, &mut Holds), Added<Holds>>) {
    if bank.kept() {
        return;
    }
    for (body, mut holds) in &mut bodies {
        if let Some(bag) = remembered.0.get(&body.who.0) {
            holds.bag = bag.clone();
        }
    }
}

/// What came of something asked of a bag.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Came {
    /// Nothing changed, or only what is owned.
    Nothing,
    /// What stands on the plot changed.
    Plot,
    /// A purchase or a sale has gone to the database: answered when it has.
    Waits,
}

/// Does `ask`, which the device `by` asked, to the bag `holds` of the body
/// `body`: at once, but for a purchase or a sale with a database, which is
/// sent there. Whatever is done at once is answered ([`answer`]).
#[allow(clippy::too_many_arguments)]
fn act(
    ask: &bag::Ask,
    by: Entity,
    body: &mut Body,
    holds: &mut Holds,
    bank: &Bank,
    remembered: &mut Remembered,
    watchers: &mut Watchers,
    tells: &mut MessageWriter<ToClients<Tell>>,
) -> Came {
    let who = body.who.0;
    let came = match ask {
        bag::Ask::Watch => Came::Nothing,
        bag::Ask::Buy(kind) | bag::Ask::Sell(kind) => {
            let buying = matches!(ask, bag::Ask::Buy(_));
            let mut after = holds.bag.clone();
            let deal = if buying { after.buy(kind, body.coins) } else { after.sell(kind, body.coins) };
            match deal {
                Ok(left) if bank.kept() => {
                    let amount = i64::from(left) - i64::from(body.coins);
                    let why = if buying { "bought a thing" } else { "sold a thing" };
                    bank.deal(who, amount, why, kept(&after));
                    holds.dealing = Some(Dealing { after, by });
                    return Came::Waits;
                }
                Ok(left) => {
                    holds.bag = after;
                    body.coins = left;
                    if let Some(client) = body.client {
                        tell(tells, client, Tell::Balance(left));
                    }
                }
                Err(no) => debug!("{who}: {ask:?} was not done: {no:?}"),
            }
            Came::Nothing
        }
        bag::Ask::Put { id, kind, at } => match holds.bag.put(*id, kind, *at) {
            Ok(()) => Came::Plot,
            Err(no) => {
                debug!("{who}: {ask:?} was not done: {no:?}");
                Came::Nothing
            }
        },
        bag::Ask::Take { id } => match holds.bag.take(*id) {
            Ok(()) => Came::Plot,
            Err(_) => Came::Nothing,
        },
    };
    if came == Came::Plot {
        bank.keep_bag(who, kept(&holds.bag));
    }
    if !bank.kept() {
        remembered.0.insert(who, holds.bag.clone());
    }
    answer(by, body, holds, watchers, tells);
    came
}

/// Tells the device `by` its bag as it stands, in answer to something it
/// asked: one more of its asks answered. Not if it has gone, or is no
/// longer the device playing this body.
fn answer(by: Entity, body: &Body, holds: &Holds, watchers: &mut Watchers, tells: &mut MessageWriter<ToClients<Tell>>) {
    let Some(asked) = watchers.0.get_mut(&by) else {
        return;
    };
    *asked += 1;
    if body.client == Some(by) {
        tell(
            tells,
            by,
            Tell::Bag(bag::Tell::Yours {
                asked: *asked,
                bag: holds.bag.clone(),
            }),
        );
    }
}

/// The devices of everyone seated in `channel` but `but`, which asked to be
/// told of bags.
fn others_watching(
    towns: &Towns,
    channel: u16,
    but: Entity,
    watchers: &Watchers,
    client_of: impl Fn(Entity) -> Option<Entity>,
) -> Vec<Entity> {
    towns
        .seated(channel)
        .into_iter()
        .filter(|&(_, body)| body != but)
        .filter_map(|(_, body)| client_of(body))
        .filter(|client| watchers.0.contains_key(client))
        .collect()
}

/// What devices ask of their bags.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<Option<&Plays>, With<ConnectedClient>>,
    mut bodies: Query<(&mut Body, &mut Holds, &Member)>,
    mut towns: ResMut<Towns>,
    bank: Res<Bank>,
    mut remembered: ResMut<Remembered>,
    mut watchers: ResMut<Watchers>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    for ask in asks.read() {
        let Ask::Bag(asked) = &ask.message else {
            continue;
        };
        let Some(client) = ask.client_id.entity() else {
            continue;
        };
        let Ok(plays) = clients.get(client) else {
            continue;
        };
        if *asked == bag::Ask::Watch {
            // Told from here on: its own bag and its channel's plots as it
            // sits down, which is now if it sits there already.
            *watchers.0.entry(client).or_insert(0) += 1;
            if let Some(&Plays(body)) = plays
                && let Ok((body_of, ..)) = bodies.get(body)
                && let Room::Town(channel) = body_of.room
            {
                towns.announce(channel, body);
            }
            continue;
        }
        let Some(&Plays(body)) = plays else {
            // Not let in yet: there is no bag to ask anything of. Counted
            // all the same, so that the device's count and this one agree.
            if let Some(count) = watchers.0.get_mut(&client) {
                *count += 1;
            }
            continue;
        };
        let Ok((mut body_of, mut holds, member)) = bodies.get_mut(body) else {
            continue;
        };
        // One thing at a time: behind whatever of theirs is at the
        // database, and whatever waits behind that.
        if holds.dealing.is_some() {
            holds.waiting.push_back((client, asked.clone()));
            continue;
        }
        let (who, room) = (member.id, body_of.room);
        let came = act(asked, client, &mut body_of, &mut holds, &bank, &mut remembered, &mut watchers, &mut tells);
        if came == Came::Plot {
            let stands = holds.bag.stands.clone();
            tell_the_channel(&towns, room, body, who, stands, &watchers, &bodies, &mut tells);
        }
    }
}

/// Tells everyone else seated with `body` in a channel of the town, whose
/// device asked to be told, what stands on its plot now.
#[allow(clippy::too_many_arguments)]
fn tell_the_channel(
    towns: &Towns,
    room: Room,
    body: Entity,
    who: u64,
    stands: Vec<Standing>,
    watchers: &Watchers,
    bodies: &Query<(&mut Body, &mut Holds, &Member)>,
    tells: &mut MessageWriter<ToClients<Tell>>,
) {
    let Room::Town(channel) = room else {
        return;
    };
    let client_of = |other: Entity| bodies.get(other).ok().and_then(|(other, ..)| other.client);
    for device in others_watching(towns, channel, body, watchers, client_of) {
        tell(
            tells,
            device,
            Tell::Bag(bag::Tell::Theirs {
                who,
                stands: stands.clone(),
            }),
        );
    }
}

/// A purchase or a sale has been through the database: if it went through,
/// the body's bag and coins are what it left them, and its device is told
/// what it has; either way the device that asked is answered, and whatever
/// was asked of the bag meanwhile is done now, in order, up to the next
/// purchase or sale, which goes to the database in its turn.
#[allow(clippy::too_many_arguments)]
fn dealt(
    mut dealt: MessageReader<Dealt>,
    everyone: Res<Everyone>,
    mut bodies: Query<(&mut Body, &mut Holds, &Member)>,
    towns: Res<Towns>,
    bank: Res<Bank>,
    mut remembered: ResMut<Remembered>,
    mut watchers: ResMut<Watchers>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    for answered in dealt.read() {
        let Some(&body) = everyone.0.get(&Who(answered.id)) else {
            continue;
        };
        let Ok((mut body_of, mut holds, member)) = bodies.get_mut(body) else {
            continue;
        };
        let Some(dealing) = holds.dealing.take() else {
            continue;
        };
        if let Some(left) = answered.left {
            holds.bag = dealing.after;
            body_of.coins = left;
            if let Some(client) = body_of.client {
                tell(&mut tells, client, Tell::Balance(left));
            }
        }
        answer(dealing.by, &body_of, &holds, &mut watchers, &mut tells);
        let (who, room) = (member.id, body_of.room);
        let mut moved = false;
        while holds.dealing.is_none() {
            let Some((by, asked)) = holds.waiting.pop_front() else {
                break;
            };
            let came = act(&asked, by, &mut body_of, &mut holds, &bank, &mut remembered, &mut watchers, &mut tells);
            moved |= came == Came::Plot;
        }
        if moved {
            let stands = holds.bag.stands.clone();
            tell_the_channel(&towns, room, body, who, stands, &watchers, &bodies, &mut tells);
        }
    }
}

/// As anyone sits down in a channel of the town: their device, if it asked
/// to be told, is told its own bag and what stands on the plot of everyone
/// else there who has anything standing; and everyone else there whose
/// device asked is told what stands on the newcomer's, if anything does.
/// Whoever has nothing standing is not spoken of: a device knows nothing of
/// a plot until it is told, and lets go of what it was told of anyone who
/// is no longer in its channel.
fn seat_news(
    mut towns: ResMut<Towns>,
    bodies: Query<(&Body, &Holds, &Member)>,
    watchers: Res<Watchers>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    if !towns.has_sat() {
        return;
    }
    for (channel, newcomer) in towns.take_sat() {
        let Ok((body, holds, member)) = bodies.get(newcomer) else {
            continue;
        };
        let seated = towns.seated(channel);
        let others = || seated.iter().filter(|&&(_, other)| other != newcomer).filter_map(|&(_, other)| bodies.get(other).ok());
        if let Some(client) = body.client
            && let Some(&asked) = watchers.0.get(&client)
        {
            tell(
                &mut tells,
                client,
                Tell::Bag(bag::Tell::Yours {
                    asked,
                    bag: holds.bag.clone(),
                }),
            );
            for (_, theirs, other) in others() {
                if !theirs.bag.stands.is_empty() {
                    tell(
                        &mut tells,
                        client,
                        Tell::Bag(bag::Tell::Theirs {
                            who: other.id,
                            stands: theirs.bag.stands.clone(),
                        }),
                    );
                }
            }
        }
        if holds.bag.stands.is_empty() {
            continue;
        }
        for (other, ..) in others() {
            if let Some(device) = other.client
                && watchers.0.contains_key(&device)
            {
                tell(
                    &mut tells,
                    device,
                    Tell::Bag(bag::Tell::Theirs {
                        who: member.id,
                        stands: holds.bag.stands.clone(),
                    }),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use roundtown_net::At;
    use roundtown_net::bag::{CAR, LAMP};

    use super::*;

    fn here() -> At {
        At::Ground {
            x: 3.25,
            y: 0.0,
            z: -4.5,
            yaw: 0.785,
        }
    }

    #[test]
    fn a_bag_is_kept_as_a_document_and_read_back_as_it_was() {
        let mut bag = Bag::new();
        bag.buy("singlebed", 1_000).unwrap();
        bag.put(4, LAMP, here()).unwrap();
        bag.put(5, "singlebed", At::Wall { wall: 1, along: -2.5, up: 1.1, face: -1 }).unwrap();
        let document = kept(&bag);
        assert!(document.contains("streetlamp") && !document.contains("gems"), "{document}");
        assert_eq!(read(10, Some(&document)), bag);
        // The gems are the account's own column, whatever the document was
        // written with.
        assert_eq!(read(3, Some(&document)).gems, 3);
        // An account that has kept nothing has what everyone begins with.
        assert_eq!(read(10, None), Bag::new());
        assert_eq!(read(7, None), Bag { gems: 7, ..Bag::new() });
        // One that will not read is begun afresh, and one kept wrongly put
        // right.
        assert_eq!(read(10, Some("{not json")), Bag::new());
        let wrong = r#"{"owned":[["a_castle",2],["streetlamp",1]],"stands":[{"id":1,"kind":"a_castle","at":{"Ground":{"x":0.0,"y":0.0,"z":0.0,"yaw":0.0}}}]}"#;
        let put_right = read(10, Some(wrong));
        assert_eq!(put_right.owned, vec![(LAMP.to_string(), 1)]);
        assert!(put_right.stands.is_empty());
    }

    /// A server with no database, one channel of the town, and `people`
    /// seated in it, each on a device that has asked to be told of bags:
    /// their bodies and their devices, in order.
    fn town(people: u64) -> (App, Vec<Entity>, Vec<Entity>) {
        let mut app = App::new();
        app.add_message::<ToClients<Tell>>()
            .add_message::<FromClient<Ask>>()
            .add_message::<Dealt>()
            .insert_resource(Bank::Memory)
            .init_resource::<Towns>()
            .init_resource::<Everyone>()
            .init_resource::<Watchers>()
            .init_resource::<Remembered>();
        let (mut bodies, mut devices) = (Vec::new(), Vec::new());
        for n in 0..people {
            let body = app.world_mut().spawn_empty().id();
            let device = app.world_mut().spawn((ConnectedClient { max_size: 1_200 }, Plays(body))).id();
            let (channel, seat) = app.world_mut().resource_mut::<Towns>().sit(body, None);
            let who = Who(100 + n);
            app.world_mut().entity_mut(body).insert((
                Body::person(device, who, Room::Town(channel), seat, Some(channel), 1_000, 0.0),
                Holds::holding(Bag::new()),
                Member {
                    id: who.0,
                    name: format!("p{n}"),
                    look: "boy".into(),
                    ai: false,
                },
            ));
            app.world_mut().resource_mut::<Everyone>().0.insert(who, body);
            app.world_mut().resource_mut::<Watchers>().0.insert(device, 1);
            bodies.push(body);
            devices.push(device);
        }
        (app, bodies, devices)
    }

    /// What `device` has been told since this was last asked.
    fn told(app: &mut App, device: Entity) -> Vec<Tell> {
        let mut all = app.world_mut().resource_mut::<Messages<ToClients<Tell>>>();
        let (mut theirs, mut rest) = (Vec::new(), Vec::new());
        for told in all.drain() {
            if matches!(told.targets, SendTargets::Single(ClientId::Client(to)) if to == device) {
                theirs.push(told.message);
            } else {
                rest.push(told);
            }
        }
        for told in rest {
            all.write(told);
        }
        theirs
    }

    fn ask(app: &mut App, device: Entity, ask: bag::Ask) {
        app.world_mut().write_message(FromClient {
            client_id: ClientId::Client(device),
            message: Ask::Bag(ask),
        });
        app.world_mut().run_system_once(hear).unwrap();
        // A system run once reads from the first of what is there.
        app.world_mut().resource_mut::<Messages<FromClient<Ask>>>().clear();
    }

    /// The database's answer to a purchase or a sale, as its thread would
    /// give it.
    fn answer_the_deal(app: &mut App, left: Option<u32>) {
        app.world_mut().write_message(Dealt { id: 100, left });
        app.world_mut().run_system_once(dealt).unwrap();
        app.world_mut().resource_mut::<Messages<Dealt>>().clear();
    }

    fn bag_of(app: &App, body: Entity) -> Bag {
        app.world().get::<Holds>(body).unwrap().bag.clone()
    }

    #[test]
    fn whoever_sits_down_is_told_their_bag_and_the_plots_about_them_and_they_of_theirs() {
        let (mut app, bodies, devices) = town(3);
        // The second has a lamp standing; nobody else anything.
        app.world_mut().get_mut::<Holds>(bodies[1]).unwrap().bag.put(1, LAMP, here()).unwrap();
        app.world_mut().run_system_once(seat_news).unwrap();
        // The first: its own bag, and the second's plot; nothing of the
        // third's, which is empty.
        let first = told(&mut app, devices[0]);
        assert!(matches!(&first[0], Tell::Bag(bag::Tell::Yours { asked: 1, bag }) if *bag == Bag::new()), "{first:?}");
        let plots: Vec<u64> = first
            .iter()
            .filter_map(|tell| match tell {
                Tell::Bag(bag::Tell::Theirs { who, stands }) if stands.len() == 1 => Some(*who),
                _ => None,
            })
            .collect();
        // Once as the first sat among them, and once as the second sat
        // down: all three sat in the one tick here.
        assert!(!plots.is_empty() && plots.iter().all(|&who| who == 101), "{first:?}");
        // The second: its own bag, the lamp standing, and no plot.
        let second = told(&mut app, devices[1]);
        assert!(matches!(&second[0], Tell::Bag(bag::Tell::Yours { bag, .. }) if bag.stands.len() == 1), "{second:?}");
        assert_eq!(second.len(), 1, "{second:?}");
        // The third: its own bag, and the second's plot.
        let third = told(&mut app, devices[2]);
        assert!(third.iter().any(|tell| matches!(tell, Tell::Bag(bag::Tell::Yours { bag, .. }) if *bag == Bag::new())), "{third:?}");
        assert!(third.iter().any(|tell| matches!(tell, Tell::Bag(bag::Tell::Theirs { who: 101, .. }))), "{third:?}");
        // Nothing more is said until someone sits down again.
        app.world_mut().run_system_once(seat_news).unwrap();
        assert!(told(&mut app, devices[0]).is_empty() && told(&mut app, devices[2]).is_empty());
        // A device that never asked is told nothing of any of it.
        app.world_mut().resource_mut::<Watchers>().0.remove(&devices[2]);
        app.world_mut().resource_mut::<Towns>().announce(0, bodies[1]);
        app.world_mut().resource_mut::<Towns>().announce(0, bodies[2]);
        app.world_mut().run_system_once(seat_news).unwrap();
        assert!(told(&mut app, devices[2]).is_empty());
        assert_eq!(told(&mut app, devices[0]).len(), 1);
    }

    #[test]
    fn buying_and_selling_change_the_coins_and_the_bag_and_are_answered_once_each() {
        let (mut app, bodies, devices) = town(2);
        ask(&mut app, devices[0], bag::Ask::Buy("singlebed".into()));
        assert_eq!(app.world().get::<Body>(bodies[0]).unwrap().coins, 850);
        assert_eq!(bag_of(&app, bodies[0]).count("singlebed"), 1);
        let heard = told(&mut app, devices[0]);
        assert!(matches!(heard[0], Tell::Balance(850)), "{heard:?}");
        assert!(matches!(&heard[1], Tell::Bag(bag::Tell::Yours { asked: 2, bag }) if bag.count("singlebed") == 1), "{heard:?}");
        // Nobody else is told of what someone owns.
        assert!(told(&mut app, devices[1]).is_empty());
        // What cannot be bought is answered all the same, the bag as it was.
        ask(&mut app, devices[0], bag::Ask::Buy(LAMP.into()));
        let heard = told(&mut app, devices[0]);
        assert_eq!(heard.len(), 1, "{heard:?}");
        assert!(matches!(&heard[0], Tell::Bag(bag::Tell::Yours { asked: 3, bag }) if bag.count(LAMP) == 1), "{heard:?}");
        assert_eq!(app.world().get::<Body>(bodies[0]).unwrap().coins, 850);
        // Sold: the car for half of fifteen thousand.
        ask(&mut app, devices[0], bag::Ask::Sell(CAR.into()));
        assert_eq!(app.world().get::<Body>(bodies[0]).unwrap().coins, 8_350);
        assert!(!bag_of(&app, bodies[0]).has_car());
        let heard = told(&mut app, devices[0]);
        assert!(matches!(heard[0], Tell::Balance(8_350)), "{heard:?}");
        assert!(matches!(&heard[1], Tell::Bag(bag::Tell::Yours { asked: 4, .. })), "{heard:?}");
        // With no database the bag is remembered for when they are back.
        assert!(!app.world().resource::<Remembered>().0[&100].has_car());
    }

    #[test]
    fn what_is_stood_on_a_plot_is_told_to_the_channel_and_taken_back_too() {
        let (mut app, bodies, devices) = town(3);
        // The third's device never asked.
        app.world_mut().resource_mut::<Watchers>().0.remove(&devices[2]);
        ask(&mut app, devices[0], bag::Ask::Put { id: 1, kind: LAMP.into(), at: here() });
        assert_eq!(bag_of(&app, bodies[0]).stands.len(), 1);
        let mine = told(&mut app, devices[0]);
        assert!(matches!(&mine[0], Tell::Bag(bag::Tell::Yours { asked: 2, bag }) if bag.free(LAMP) == 0), "{mine:?}");
        assert_eq!(mine.len(), 1);
        let theirs = told(&mut app, devices[1]);
        assert!(
            matches!(&theirs[0], Tell::Bag(bag::Tell::Theirs { who: 100, stands }) if stands.len() == 1 && stands[0].at == here()),
            "{theirs:?}"
        );
        assert!(told(&mut app, devices[2]).is_empty());
        // What cannot be stood, a second lamp, is answered and told to
        // nobody else.
        ask(&mut app, devices[0], bag::Ask::Put { id: 2, kind: LAMP.into(), at: here() });
        assert_eq!(told(&mut app, devices[0]).len(), 1);
        assert!(told(&mut app, devices[1]).is_empty());
        // Taken back: the channel is told the plot is bare.
        ask(&mut app, devices[0], bag::Ask::Take { id: 1 });
        assert!(bag_of(&app, bodies[0]).stands.is_empty());
        let theirs = told(&mut app, devices[1]);
        assert!(matches!(&theirs[0], Tell::Bag(bag::Tell::Theirs { who: 100, stands }) if stands.is_empty()), "{theirs:?}");
    }

    #[test]
    fn with_a_database_a_purchase_waits_for_it_and_what_is_asked_meanwhile_waits_its_turn() {
        let (mut app, bodies, devices) = town(2);
        // A bank that keeps: nothing listens at the other end of it here,
        // and the answers are written by hand.
        let (jobs, _sent) = std::sync::mpsc::channel();
        let (_answers, done) = std::sync::mpsc::channel();
        app.insert_resource(Bank::Database {
            jobs,
            done: std::sync::Mutex::new(done),
        });
        ask(&mut app, devices[0], bag::Ask::Buy("singlebed".into()));
        // Nothing has changed yet, and nothing is said.
        assert_eq!(app.world().get::<Body>(bodies[0]).unwrap().coins, 1_000);
        assert_eq!(bag_of(&app, bodies[0]).count("singlebed"), 0);
        assert!(told(&mut app, devices[0]).is_empty());
        // Standing the lamp and the bed, asked meanwhile, wait.
        ask(&mut app, devices[0], bag::Ask::Put { id: 1, kind: LAMP.into(), at: here() });
        ask(&mut app, devices[0], bag::Ask::Put { id: 2, kind: "singlebed".into(), at: here() });
        assert!(bag_of(&app, bodies[0]).stands.is_empty());
        assert!(told(&mut app, devices[0]).is_empty() && told(&mut app, devices[1]).is_empty());
        // The database answers: the purchase went through.
        answer_the_deal(&mut app, Some(850));
        assert_eq!(app.world().get::<Body>(bodies[0]).unwrap().coins, 850);
        // The bed that was bought stands, with the lamp: each thing was
        // done on the bag the one before it left.
        let bag = bag_of(&app, bodies[0]);
        assert_eq!((bag.count("singlebed"), bag.stands.len()), (1, 2));
        let heard = told(&mut app, devices[0]);
        assert!(matches!(heard[0], Tell::Balance(850)), "{heard:?}");
        let answers: Vec<u32> = heard
            .iter()
            .filter_map(|tell| match tell {
                Tell::Bag(bag::Tell::Yours { asked, .. }) => Some(*asked),
                _ => None,
            })
            .collect();
        assert_eq!(answers, [2, 3, 4], "one answer to each thing asked, in order: {heard:?}");
        // The channel is told of the plot once, as it stands at the end.
        let theirs = told(&mut app, devices[1]);
        assert_eq!(theirs.len(), 1, "{theirs:?}");
        assert!(matches!(&theirs[0], Tell::Bag(bag::Tell::Theirs { who: 100, stands }) if stands.len() == 2), "{theirs:?}");
        // One the database turned down changes nothing, and is answered.
        ask(&mut app, devices[0], bag::Ask::Buy("cafechair".into()));
        answer_the_deal(&mut app, None);
        assert_eq!(app.world().get::<Body>(bodies[0]).unwrap().coins, 850);
        assert_eq!(bag_of(&app, bodies[0]).count("cafechair"), 0);
        let heard = told(&mut app, devices[0]);
        assert_eq!(heard.len(), 1, "{heard:?}");
        assert!(matches!(&heard[0], Tell::Bag(bag::Tell::Yours { asked: 5, .. })), "{heard:?}");
    }

    #[test]
    fn a_device_that_asks_to_be_told_while_seated_is_told_as_if_it_had_just_sat_down() {
        let (mut app, bodies, devices) = town(2);
        app.world_mut().resource_mut::<Towns>().take_sat();
        app.world_mut().resource_mut::<Watchers>().0.clear();
        ask(&mut app, devices[0], bag::Ask::Watch);
        assert_eq!(app.world().resource::<Watchers>().0.get(&devices[0]), Some(&1));
        assert_eq!(app.world_mut().resource_mut::<Towns>().take_sat(), vec![(0, bodies[0])]);
    }
}
