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
//!
//! **Colours** (since 2026-11-22; Hajun: "u can tap objects and it will
//! paint it to that color ... color gets reset if u trash that item and
//! place it back, but stays if u leave it and also the new color should be
//! visible to other players"): what a person has painted of what stands on
//! their plot ([`bag::Paints`]) is on their body beside the bag, kept in the
//! same document, changed by [`bag::Ask::Paint`], and let go of for whatever
//! is taken back into the bag. It is told ([`bag::Tell::Painted`]) only to a
//! device that has asked for colours too ([`bag::Ask::Colours`],
//! [`Colourers`]): after its own bag, each time that is said, and after
//! what stands on anyone else's plot, and whenever they paint. An app from
//! before there were colours never asks, and is told what it always was.

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::bag::{self, Bag, Paint, Paints, Standing};
use roundtown_net::land::Lot;
use roundtown_net::{Ask, Member, Tell};
use serde::{Deserialize, Serialize};

use crate::accounts::{Bank, Dealt, Record};
use crate::players::{Account, Body, Everyone, Plays, Who, tell};
use crate::rooms::{Room, Towns};

/// A bag as it is kept with its account (`players.bag`): what is owned,
/// what of it stands on the plot, and what of that is painted (nothing, in
/// a bag kept before 2026-11-22). The gems are a column of their own.
#[derive(Serialize, Deserialize, Default)]
struct Kept {
    owned: Vec<(String, u16)>,
    stands: Vec<Standing>,
    #[serde(default)]
    paints: Paints,
}

/// `bag`, and the colours on its plot, as they are kept with their account.
pub(crate) fn kept(bag: &Bag, paints: &Paints) -> String {
    let kept = Kept {
        owned: bag.owned.clone(),
        stands: bag.stands.clone(),
        paints: paints.clone(),
    };
    serde_json::to_string(&kept).unwrap_or_default()
}

/// The bag of an account with `gems` gems that keeps `kept` (`players.bag`),
/// and the colours on its plot: with nothing kept, what everyone begins
/// with; and whatever was kept wrongly, or cannot be read, put right
/// ([`Bag::sane`], [`Paints::sane`]).
pub(crate) fn read(gems: u32, kept: Option<&str>) -> (Bag, Paints) {
    let (bag, paints) = match kept.map(serde_json::from_str::<Kept>) {
        Some(Ok(kept)) => (
            Bag {
                gems,
                owned: kept.owned,
                stands: kept.stands,
            },
            kept.paints,
        ),
        Some(Err(error)) => {
            warn!("a bag that will not read ({error}): begun afresh");
            (Bag { gems, ..Bag::new() }, Paints::default())
        }
        None => (Bag { gems, ..Bag::new() }, Paints::default()),
    };
    let bag = bag.sane();
    let paints = paints.sane(&bag.stands);
    (bag, paints)
}

/// A person's bag, on their body, and the colours on their plot.
#[derive(Component)]
pub(crate) struct Holds {
    pub bag: Bag,
    pub paints: Paints,
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
        let (bag, paints) = read(record.gems, record.bag.as_deref());
        Self { paints, ..Self::holding(bag) }
    }

    fn holding(bag: Bag) -> Self {
        Self {
            bag,
            paints: Paints::default(),
            dealing: None,
            waiting: VecDeque::new(),
        }
    }

    /// Their land as it stands now, `member` being who their account is:
    /// what fills a plot of the town with it when they are not there
    /// (`land::Lot`, `Towns`).
    pub(crate) fn lot(&self, member: &Member) -> Lot {
        Lot {
            who: member.id,
            name: member.name.clone(),
            look: member.look.clone(),
            stands: self.bag.stands.clone(),
            paints: self.paints.0.clone(),
        }
    }
}

/// The land of an account as it is kept, read as the server starts
/// (`accounts::Snapshot`): what fills a plot with it before its owner has
/// ever been here this run.
pub(crate) fn lot_kept(who: u64, name: &str, look: &str, kept: &str) -> Lot {
    let (bag, paints) = read(0, Some(kept));
    Lot {
        who,
        name: name.to_string(),
        look: look.to_string(),
        stands: bag.stands,
        paints: paints.0,
    }
}

/// The devices that have asked to be told of bags ([`bag::Ask::Watch`]):
/// those whose app knows how to read it, each with how many of its asks
/// about its bag have been answered ([`bag::Tell::Yours`]).
#[derive(Resource, Default)]
pub(crate) struct Watchers(HashMap<Entity, u32>);

/// The devices among them that have asked to be told of colours too
/// ([`bag::Ask::Colours`]): those whose app knows how to read them.
#[derive(Resource, Default)]
pub(crate) struct Colourers(HashSet<Entity>);

/// With no database: everyone's bag and colours for as long as the server
/// runs, by their account, so that someone who drops and comes back finds
/// theirs.
#[derive(Resource, Default)]
struct Remembered(HashMap<u64, (Bag, Paints)>);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Watchers>()
        .init_resource::<Colourers>()
        .init_resource::<Remembered>()
        .add_observer(forget)
        .add_systems(
            PreUpdate,
            (recall, dealt, hear, register)
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
fn forget(remove: On<Remove, ConnectedClient>, mut watchers: ResMut<Watchers>, mut colourers: ResMut<Colourers>) {
    watchers.0.remove(&remove.entity);
    colourers.0.remove(&remove.entity);
}

/// With no database, gives a body just made the bag its account had when it
/// was last here.
fn recall(bank: Res<Bank>, remembered: Res<Remembered>, mut bodies: Query<(&Body, &mut Holds), Added<Holds>>) {
    if bank.kept() {
        return;
    }
    for (body, mut holds) in &mut bodies {
        if let Some((bag, paints)) = remembered.0.get(&body.who.0) {
            holds.bag = bag.clone();
            holds.paints = paints.clone();
        }
    }
}

/// Keeps what the town knows of everyone's land as their bags have it
/// (`Towns::record`): as an account is let in, as what stands on its plot
/// changes or is painted, and as it takes another name or look. It is what
/// a channel's plots are filled with as it opens (`land::Estate`), whether
/// or not its owner is here then.
#[allow(clippy::type_complexity)]
fn register(lands: Query<(&Account, &Holds), Or<(Changed<Holds>, Changed<Account>)>>, mut towns: ResMut<Towns>) {
    for (account, holds) in &lands {
        towns.record(holds.lot(&account.0));
    }
}

/// What came of something asked of a bag.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Came {
    /// Nothing changed, or only what is owned.
    Nothing,
    /// What stands on the plot changed.
    Plot,
    /// What stands there is as it was, and painted another colour.
    Paint,
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
    (watchers, colourers): (&mut Watchers, &Colourers),
    tells: &mut MessageWriter<ToClients<Tell>>,
) -> Came {
    let who = body.who.0;
    let came = match ask {
        bag::Ask::Watch | bag::Ask::Colours => Came::Nothing,
        bag::Ask::Buy(kind) | bag::Ask::Sell(kind) => {
            let buying = matches!(ask, bag::Ask::Buy(_));
            let mut after = holds.bag.clone();
            let deal = if buying { after.buy(kind, body.coins) } else { after.sell(kind, body.coins) };
            match deal {
                Ok(left) if bank.kept() => {
                    // In coins, or for a thing sold for gems in gems, which
                    // are a column of their own (`Bank::deal`).
                    let amount = i64::from(left) - i64::from(body.coins);
                    let gems = i64::from(after.gems) - i64::from(holds.bag.gems);
                    let why = if buying { "bought a thing" } else { "sold a thing" };
                    bank.deal(who, amount, gems, why, kept(&after, &holds.paints));
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
            // Back in the bag, it is as it was made: its colours go.
            Ok(()) => {
                holds.paints.fit(&holds.bag.stands);
                Came::Plot
            }
            Err(_) => Came::Nothing,
        },
        bag::Ask::Paint { id, part, colour } => match holds.paints.set(&holds.bag.stands, *id, part, *colour) {
            Ok(()) => Came::Paint,
            Err(no) => {
                debug!("{who}: {ask:?} was not done: {no:?}");
                Came::Nothing
            }
        },
        bag::Ask::Unpaint { id, part } => {
            if holds.paints.clear(*id, part) {
                Came::Paint
            } else {
                Came::Nothing
            }
        }
    };
    if matches!(came, Came::Plot | Came::Paint) {
        bank.keep_bag(who, kept(&holds.bag, &holds.paints));
    }
    if !bank.kept() {
        remembered.0.insert(who, (holds.bag.clone(), holds.paints.clone()));
    }
    answer(by, body, holds, watchers, colourers, tells);
    came
}

/// Tells the device `by` its bag as it stands, in answer to something it
/// asked: one more of its asks answered; and, if it asked for colours, the
/// colours on its plot after it. Not if it has gone, or is no longer the
/// device playing this body.
fn answer(
    by: Entity,
    body: &Body,
    holds: &Holds,
    watchers: &mut Watchers,
    colourers: &Colourers,
    tells: &mut MessageWriter<ToClients<Tell>>,
) {
    let Some(asked) = watchers.0.get_mut(&by) else {
        return;
    };
    *asked += 1;
    if body.client == Some(by) {
        tell_its_own(by, *asked, body.who.0, holds, colourers, tells);
    }
}

/// Tells `device` its own bag as it stands, with `asked` of its asks
/// answered, and after it, if it asked for colours, the colours on its
/// plot, which is `who`'s.
fn tell_its_own(device: Entity, asked: u32, who: u64, holds: &Holds, colourers: &Colourers, tells: &mut MessageWriter<ToClients<Tell>>) {
    tell(
        tells,
        device,
        Tell::Bag(bag::Tell::Yours {
            asked,
            bag: holds.bag.clone(),
        }),
    );
    if colourers.0.contains(&device) {
        tell(
            tells,
            device,
            Tell::Bag(bag::Tell::Painted {
                who,
                asked,
                paints: holds.paints.0.clone(),
            }),
        );
    }
}

/// Tells `device` what stands on `who`'s plot, if `stands` says, and, if it
/// asked for colours, what of it is painted.
fn tell_of_a_plot(
    device: Entity,
    who: u64,
    stands: Option<&[Standing]>,
    paints: &[Paint],
    colourers: &Colourers,
    tells: &mut MessageWriter<ToClients<Tell>>,
) {
    if let Some(stands) = stands {
        tell(
            tells,
            device,
            Tell::Bag(bag::Tell::Theirs {
                who,
                stands: stands.to_vec(),
            }),
        );
    }
    if colourers.0.contains(&device) {
        tell(
            tells,
            device,
            Tell::Bag(bag::Tell::Painted {
                who,
                asked: 0,
                paints: paints.to_vec(),
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
    (mut watchers, mut colourers): (ResMut<Watchers>, ResMut<Colourers>),
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
        if matches!(asked, bag::Ask::Watch | bag::Ask::Colours) {
            // Told from here on: its own bag and its channel's plots as it
            // sits down, which is now if it sits there already; and, having
            // asked for them, their colours.
            *watchers.0.entry(client).or_insert(0) += 1;
            if *asked == bag::Ask::Colours {
                colourers.0.insert(client);
            }
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
        let came = act(asked, client, &mut body_of, &mut holds, &bank, &mut remembered, (&mut watchers, &colourers), &mut tells);
        if matches!(came, Came::Plot | Came::Paint) {
            let stands = (came == Came::Plot).then(|| holds.bag.stands.clone());
            let paints = holds.paints.0.clone();
            tell_the_channel(&towns, room, body, who, stands, paints, (&watchers, &colourers), &bodies, &mut tells);
        }
    }
}

/// Tells everyone else seated with `body` in a channel of the town, whose
/// device asked to be told, what stands on its plot now, if `stands` says
/// (it has changed), and those that asked for colours what of it is
/// painted.
#[allow(clippy::too_many_arguments)]
fn tell_the_channel(
    towns: &Towns,
    room: Room,
    body: Entity,
    who: u64,
    stands: Option<Vec<Standing>>,
    paints: Vec<Paint>,
    (watchers, colourers): (&Watchers, &Colourers),
    bodies: &Query<(&mut Body, &mut Holds, &Member)>,
    tells: &mut MessageWriter<ToClients<Tell>>,
) {
    let Room::Town(channel) = room else {
        return;
    };
    let client_of = |other: Entity| bodies.get(other).ok().and_then(|(other, ..)| other.client);
    for device in others_watching(towns, channel, body, watchers, client_of) {
        tell_of_a_plot(device, who, stands.as_deref(), &paints, colourers, tells);
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
    (mut watchers, colourers): (ResMut<Watchers>, Res<Colourers>),
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
        answer(dealing.by, &body_of, &holds, &mut watchers, &colourers, &mut tells);
        let (who, room) = (member.id, body_of.room);
        let (mut moved, mut painted) = (false, false);
        while holds.dealing.is_none() {
            let Some((by, asked)) = holds.waiting.pop_front() else {
                break;
            };
            let came = act(&asked, by, &mut body_of, &mut holds, &bank, &mut remembered, (&mut watchers, &colourers), &mut tells);
            moved |= came == Came::Plot;
            painted |= came == Came::Paint;
        }
        if moved || painted {
            let stands = moved.then(|| holds.bag.stands.clone());
            let paints = holds.paints.0.clone();
            tell_the_channel(&towns, room, body, who, stands, paints, (&watchers, &colourers), &bodies, &mut tells);
        }
    }
}

/// As anyone sits down in a channel of the town, come new or back from a
/// game: their device, if it asked to be told, is told its own bag, and
/// what stands on every other plot of the channel: on those of everyone
/// else of it, in the town or away, out of their own bags, and on those
/// nobody of it has, as the land that fills each has it
/// (`Towns::lots_in`). And everyone else who is in the town there, whose
/// device asked, is told what stands on the newcomer's. Each plot is told
/// of whether or not anything stands on it: a device may have been told
/// otherwise of the same owner before, their land having filled a plot
/// while they were not here. And as a plot is filled with something else
/// (`Towns::take_news`), everyone in the town there is told what stands
/// on it now. A device that asked for colours is told those of each plot
/// it is told of, its own too, after what stands there. Whose each plot
/// is has been said already this tick (`players::tell_channels`).
fn seat_news(
    mut towns: ResMut<Towns>,
    bodies: Query<(&Body, &Holds, &Member)>,
    (watchers, colourers): (Res<Watchers>, Res<Colourers>),
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    if !towns.has_sat() && !towns.has_news() {
        return;
    }
    for (channel, newcomer) in towns.take_sat() {
        let Ok((body, holds, member)) = bodies.get(newcomer) else {
            continue;
        };
        let held = towns.held_in(channel);
        let others = || {
            held.iter()
                .filter(|&&(_, other, _)| other != newcomer)
                .filter_map(|&(_, other, away)| Some((bodies.get(other).ok()?, away)))
        };
        if let Some(client) = body.client
            && let Some(&asked) = watchers.0.get(&client)
        {
            tell_its_own(client, asked, member.id, holds, &colourers, &mut tells);
            for ((_, theirs, other), _) in others() {
                tell_of_a_plot(client, other.id, Some(&theirs.bag.stands), &theirs.paints.0, &colourers, &mut tells);
            }
            for lot in towns.lots_in(channel) {
                tell_of_a_plot(client, lot.who, Some(&lot.stands), &lot.paints, &colourers, &mut tells);
            }
        }
        for ((other, ..), away) in others() {
            if !away
                && let Some(device) = other.client
                && watchers.0.contains_key(&device)
            {
                tell_of_a_plot(device, member.id, Some(&holds.bag.stands), &holds.paints.0, &colourers, &mut tells);
            }
        }
    }
    for (channel, lot) in towns.take_news() {
        for (_, body) in towns.seated(channel) {
            if let Ok((body, ..)) = bodies.get(body)
                && let Some(device) = body.client
                && watchers.0.contains_key(&device)
            {
                tell_of_a_plot(device, lot.who, Some(&lot.stands), &lot.paints, &colourers, &mut tells);
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
        let mut paints = Paints::default();
        paints.set(&bag.stands, 4, "lampostbody", [20, 40, 250]).unwrap();
        paints.set(&bag.stands, 5, "blanket", [255, 255, 0]).unwrap();
        let document = kept(&bag, &paints);
        assert!(document.contains("streetlamp") && !document.contains("gems"), "{document}");
        assert!(document.contains("lampostbody"), "{document}");
        assert_eq!(read(10, Some(&document)), (bag.clone(), paints));
        // The gems are the account's own column, whatever the document was
        // written with.
        assert_eq!(read(3, Some(&document)).0.gems, 3);
        // An account that has kept nothing has what everyone begins with.
        assert_eq!(read(10, None), (Bag::new(), Paints::default()));
        assert_eq!(read(7, None).0, Bag { gems: 7, ..Bag::new() });
        // One that will not read is begun afresh, and one kept wrongly put
        // right.
        assert_eq!(read(10, Some("{not json")).0, Bag::new());
        let wrong = r#"{"owned":[["a_castle",2],["streetlamp",1]],"stands":[{"id":1,"kind":"a_castle","at":{"Ground":{"x":0.0,"y":0.0,"z":0.0,"yaw":0.0}}}]}"#;
        let (put_right, painted) = read(10, Some(wrong));
        assert_eq!(put_right.owned, vec![(LAMP.to_string(), 1)]);
        assert!(put_right.stands.is_empty());
        // A bag kept before there were colours has none, and reads as it
        // did.
        assert!(painted.0.is_empty());
        let before = r#"{"owned":[["streetlamp",1]],"stands":[{"id":4,"kind":"streetlamp","at":{"Ground":{"x":3.25,"y":0.0,"z":-4.5,"yaw":0.785}}}]}"#;
        let (old, none) = read(10, Some(before));
        assert_eq!((old.stands.len(), none), (1, Paints::default()));
        // And colours kept of what does not stand are let go of.
        let stray = r#"{"owned":[["streetlamp",1]],"stands":[],"paints":[{"id":4,"part":"lampostbody","colour":[1,2,3]}]}"#;
        assert_eq!(read(10, Some(stray)).1, Paints::default());
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
            .init_resource::<Colourers>()
            .init_resource::<Remembered>();
        let (mut bodies, mut devices) = (Vec::new(), Vec::new());
        for n in 0..people {
            let body = app.world_mut().spawn_empty().id();
            let device = app.world_mut().spawn((ConnectedClient { max_size: 1_200 }, Plays(body))).id();
            let who = Who(100 + n);
            let (channel, seat) = app.world_mut().resource_mut::<Towns>().sit(body, who.0, None);
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

    /// What a device has been told stands on plots, each owner and how
    /// many things, in order: those of the channel's own people, and how
    /// many plots filled with made-up land beside.
    fn plots_told(told: &[Tell]) -> (Vec<(u64, usize)>, usize) {
        let all: Vec<(u64, usize)> = told
            .iter()
            .filter_map(|tell| match tell {
                Tell::Bag(bag::Tell::Theirs { who, stands }) => Some((*who, stands.len())),
                _ => None,
            })
            .collect();
        let made_up = all.iter().filter(|(who, _)| roundtown_net::land::is_premade(*who)).count();
        (all.into_iter().filter(|(who, _)| !roundtown_net::land::is_premade(*who)).collect(), made_up)
    }

    #[test]
    fn whoever_sits_down_is_told_their_bag_and_every_plot_about_them_and_they_of_theirs() {
        let (mut app, bodies, devices) = town(3);
        // The second has a lamp standing; nobody else anything.
        app.world_mut().get_mut::<Holds>(bodies[1]).unwrap().bag.put(1, LAMP, here()).unwrap();
        app.world_mut().run_system_once(seat_news).unwrap();
        // The first: its own bag; then, as it sat among them, the plots of
        // the other two, the bare one too, and of none of the 45 others,
        // which no made-up land fills (`premade::FILLS`); and once more of
        // each of the two, as each sat down in turn: all three sat in the
        // one tick here.
        let first = told(&mut app, devices[0]);
        assert!(matches!(&first[0], Tell::Bag(bag::Tell::Yours { asked: 1, bag }) if *bag == Bag::new()), "{:?}", first[0]);
        assert_eq!(plots_told(&first), (vec![(101, 1), (102, 0), (101, 1), (102, 0)], 0));
        // The second: its own bag, the lamp standing, and everyone else's
        // plot, never its own.
        let second = told(&mut app, devices[1]);
        assert!(second.iter().any(|tell| matches!(tell, Tell::Bag(bag::Tell::Yours { bag, .. }) if bag.stands.len() == 1)));
        assert_eq!(plots_told(&second), (vec![(100, 0), (100, 0), (102, 0), (102, 0)], 0));
        // The third the same.
        let third = told(&mut app, devices[2]);
        assert!(third.iter().any(|tell| matches!(tell, Tell::Bag(bag::Tell::Yours { bag, .. }) if *bag == Bag::new())));
        assert_eq!(plots_told(&third), (vec![(100, 0), (101, 1), (100, 0), (101, 1)], 0));
        // Nothing more is said until someone sits down again.
        app.world_mut().run_system_once(seat_news).unwrap();
        assert!(told(&mut app, devices[0]).is_empty() && told(&mut app, devices[2]).is_empty());
        // A device that never asked is told nothing of any of it; one that
        // did is told of whoever sits down, whether or not they asked.
        app.world_mut().resource_mut::<Watchers>().0.remove(&devices[2]);
        app.world_mut().resource_mut::<Towns>().announce(0, bodies[1]);
        app.world_mut().resource_mut::<Towns>().announce(0, bodies[2]);
        app.world_mut().run_system_once(seat_news).unwrap();
        assert!(told(&mut app, devices[2]).is_empty());
        assert_eq!(plots_told(&told(&mut app, devices[0])), (vec![(101, 1), (102, 0)], 0));
    }

    #[test]
    fn someone_away_in_a_game_keeps_their_plot_and_a_plot_filled_anew_is_told_to_whoever_is_in_the_town() {
        use roundtown_net::land::Lot;
        let (mut app, bodies, devices) = town(3);
        app.world_mut().get_mut::<Holds>(bodies[1]).unwrap().bag.put(1, LAMP, here()).unwrap();
        app.world_mut().run_system_once(seat_news).unwrap();
        for device in &devices {
            told(&mut app, *device);
        }
        // The second goes into a game, and the third comes back from one:
        // told of every plot again, the second's among them, out of the
        // bag of whoever is away; and whoever is away is told nothing.
        app.world_mut().resource_mut::<Towns>().away(bodies[1]);
        app.world_mut().resource_mut::<Towns>().announce(0, bodies[2]);
        app.world_mut().run_system_once(seat_news).unwrap();
        assert_eq!(plots_told(&told(&mut app, devices[2])), (vec![(100, 0), (101, 1)], 0));
        assert!(told(&mut app, devices[1]).is_empty());
        assert_eq!(plots_told(&told(&mut app, devices[0])), (vec![(102, 0)], 0));
        // The third's body is gone, their land with a lamp on it: kept on
        // their plot as it was, which whoever sits down is told of with the
        // rest.
        let land = Lot {
            who: 102,
            name: "p2".into(),
            look: "boy".into(),
            stands: app.world().get::<Holds>(bodies[1]).unwrap().bag.stands.clone(),
            paints: Vec::new(),
        };
        app.world_mut().resource_mut::<Towns>().leave(bodies[2], Some(land));
        app.world_mut().resource_mut::<Towns>().announce(0, bodies[0]);
        app.world_mut().run_system_once(seat_news).unwrap();
        assert_eq!(plots_told(&told(&mut app, devices[0])), (vec![(101, 1), (102, 1)], 0));
        // Someone new takes a bare plot: no real land had to move, so
        // nobody is told of a plot filled anew.
        let newcomer = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<Towns>().sit(newcomer, 500, None);
        assert!(!app.world().resource::<Towns>().has_news());
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
        assert!(!app.world().resource::<Remembered>().0[&100].0.has_car());
        // A house sold for gems (since 2026-11-25): not with ten gems; with
        // sixty, for fifty of them, the coins as they were.
        let house = "square_house_five_floor";
        ask(&mut app, devices[0], bag::Ask::Buy(house.into()));
        assert_eq!((bag_of(&app, bodies[0]).count(house), bag_of(&app, bodies[0]).gems), (0, 10));
        app.world_mut().get_mut::<Holds>(bodies[0]).unwrap().bag.gems = 60;
        let _ = told(&mut app, devices[0]);
        ask(&mut app, devices[0], bag::Ask::Buy(house.into()));
        assert_eq!((bag_of(&app, bodies[0]).count(house), bag_of(&app, bodies[0]).gems), (1, 10));
        assert_eq!(app.world().get::<Body>(bodies[0]).unwrap().coins, 8_350);
        let heard = told(&mut app, devices[0]);
        assert!(matches!(&heard[..], [Tell::Balance(8_350), Tell::Bag(bag::Tell::Yours { bag, .. })] if bag.gems == 10), "{heard:?}");
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

    /// The colours a device has been told are on `who`'s plot, and with how
    /// many of its asks answered, each time it was told, in order.
    fn painted(told: &[Tell], who: u64) -> Vec<(u32, Vec<Paint>)> {
        told.iter()
            .filter_map(|tell| match tell {
                Tell::Bag(bag::Tell::Painted { who: whose, asked, paints }) if *whose == who => Some((*asked, paints.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_part_painted_is_kept_and_told_to_whoever_asked_for_colours_and_to_nobody_else() {
        let (mut app, bodies, devices) = town(3);
        told(&mut app, devices[0]);
        // The first and the second have apps that know of colours; the
        // third's is from before them, and never asks.
        ask(&mut app, devices[0], bag::Ask::Colours);
        ask(&mut app, devices[1], bag::Ask::Colours);
        assert!(app.world().resource::<Colourers>().0.contains(&devices[0]));
        assert!(!app.world().resource::<Colourers>().0.contains(&devices[2]));
        // Asking is one more thing asked, as a watch is, and whoever asks
        // while seated is told as if it had just sat down: its bag, and
        // the colours on its plot, none yet.
        app.world_mut().run_system_once(seat_news).unwrap();
        let first = told(&mut app, devices[0]);
        assert!(matches!(&first[0], Tell::Bag(bag::Tell::Yours { asked: 2, .. })), "{first:?}");
        assert_eq!(painted(&first, 100), [(2, Vec::new())]);
        told(&mut app, devices[1]);
        told(&mut app, devices[2]);
        // The first stands its lamp and paints its post blue.
        ask(&mut app, devices[0], bag::Ask::Put { id: 1, kind: LAMP.into(), at: here() });
        told(&mut app, devices[0]);
        told(&mut app, devices[1]);
        told(&mut app, devices[2]);
        ask(&mut app, devices[0], bag::Ask::Paint { id: 1, part: "lampostbody".into(), colour: [0, 0, 255] });
        let blue = vec![Paint { id: 1, part: "lampostbody".into(), colour: [0, 0, 255] }];
        assert_eq!(app.world().get::<Holds>(bodies[0]).unwrap().paints.0, blue);
        // It is answered, its bag and then its colours, the fourth of its
        // asks; the second is told the colours and not the plot again,
        // which has not changed; the third nothing at all.
        let first = told(&mut app, devices[0]);
        assert!(matches!(&first[0], Tell::Bag(bag::Tell::Yours { asked: 4, bag }) if bag.stands.len() == 1), "{first:?}");
        assert_eq!(painted(&first, 100), [(4, blue.clone())]);
        let second = told(&mut app, devices[1]);
        assert_eq!(painted(&second, 100), [(0, blue.clone())]);
        assert_eq!(second.len(), 1, "{second:?}");
        assert!(told(&mut app, devices[2]).is_empty());
        // Painted again, it is the new colour, one to a part; what does not
        // stand, or has no name, is not painted, and is answered all the
        // same.
        ask(&mut app, devices[0], bag::Ask::Paint { id: 1, part: "lampostbody".into(), colour: [9, 9, 9] });
        ask(&mut app, devices[0], bag::Ask::Paint { id: 7, part: "lampostbody".into(), colour: [1, 1, 1] });
        ask(&mut app, devices[0], bag::Ask::Paint { id: 1, part: String::new(), colour: [1, 1, 1] });
        let grey = vec![Paint { id: 1, part: "lampostbody".into(), colour: [9, 9, 9] }];
        let first = told(&mut app, devices[0]);
        assert_eq!(painted(&first, 100), [(5, grey.clone()), (6, grey.clone()), (7, grey.clone())]);
        assert_eq!(painted(&told(&mut app, devices[1]), 100), [(0, grey.clone())]);
        // With no database it is remembered with the bag.
        assert_eq!(app.world().resource::<Remembered>().0[&100].1.0, grey);
        // The paint taken off, the part is as its model has it: the channel
        // is told, and the device answered; and taken off again, there is
        // nothing to tell.
        ask(&mut app, devices[0], bag::Ask::Unpaint { id: 1, part: "lampostbody".into() });
        assert!(app.world().get::<Holds>(bodies[0]).unwrap().paints.0.is_empty());
        assert_eq!(painted(&told(&mut app, devices[0]), 100), [(8, Vec::new())]);
        assert_eq!(painted(&told(&mut app, devices[1]), 100), [(0, Vec::new())]);
        ask(&mut app, devices[0], bag::Ask::Unpaint { id: 1, part: "lampostbody".into() });
        assert_eq!(painted(&told(&mut app, devices[0]), 100), [(9, Vec::new())]);
        assert!(told(&mut app, devices[1]).is_empty());
        ask(&mut app, devices[0], bag::Ask::Paint { id: 1, part: "lampostbody".into(), colour: [9, 9, 9] });
        told(&mut app, devices[0]);
        told(&mut app, devices[1]);
        // Someone who sits down among them is told the plot and then its
        // colours, if their app asked for colours; the third the plot
        // alone.
        app.world_mut().resource_mut::<Towns>().announce(0, bodies[1]);
        app.world_mut().resource_mut::<Towns>().announce(0, bodies[2]);
        app.world_mut().run_system_once(seat_news).unwrap();
        let second = told(&mut app, devices[1]);
        let plot = second.iter().position(|tell| matches!(tell, Tell::Bag(bag::Tell::Theirs { who: 100, .. }))).unwrap();
        assert!(matches!(&second[plot + 1], Tell::Bag(bag::Tell::Painted { who: 100, asked: 0, paints }) if *paints == grey), "{second:?}");
        let third = told(&mut app, devices[2]);
        assert!(third.iter().any(|tell| matches!(tell, Tell::Bag(bag::Tell::Theirs { who: 100, .. }))), "{third:?}");
        assert!(third.iter().all(|tell| !matches!(tell, Tell::Bag(bag::Tell::Painted { .. }))), "{third:?}");
        told(&mut app, devices[0]);
        // Taken back into the bag, the lamp is as it was made: its colours
        // go, and the channel is told the plot and its colours, bare both.
        ask(&mut app, devices[0], bag::Ask::Take { id: 1 });
        assert!(app.world().get::<Holds>(bodies[0]).unwrap().paints.0.is_empty());
        let second = told(&mut app, devices[1]);
        assert!(matches!(&second[0], Tell::Bag(bag::Tell::Theirs { who: 100, stands }) if stands.is_empty()), "{second:?}");
        assert_eq!(painted(&second, 100), [(0, Vec::new())]);
        let third = told(&mut app, devices[2]);
        assert_eq!(third.len(), 1, "{third:?}");
        // Stood again under the same number, it is not the colour it was.
        ask(&mut app, devices[0], bag::Ask::Put { id: 1, kind: LAMP.into(), at: here() });
        assert!(app.world().get::<Holds>(bodies[0]).unwrap().paints.0.is_empty());
        // A device that has gone is told nothing more, of colours either.
        app.add_observer(forget);
        app.world_mut().entity_mut(devices[1]).despawn();
        app.world_mut().flush();
        assert!(!app.world().resource::<Colourers>().0.contains(&devices[1]));
    }

    #[test]
    fn a_purchase_keeps_the_colours_and_a_part_painted_meanwhile_waits_its_turn() {
        let (mut app, bodies, devices) = town(2);
        // A bank that keeps, its answers written by hand.
        let (jobs, _sent) = std::sync::mpsc::channel();
        let (_answers, done) = std::sync::mpsc::channel();
        app.insert_resource(Bank::Database {
            jobs,
            done: std::sync::Mutex::new(done),
        });
        ask(&mut app, devices[0], bag::Ask::Colours);
        ask(&mut app, devices[1], bag::Ask::Colours);
        ask(&mut app, devices[0], bag::Ask::Put { id: 1, kind: LAMP.into(), at: here() });
        ask(&mut app, devices[0], bag::Ask::Paint { id: 1, part: "lightbulb".into(), colour: [255, 0, 0] });
        told(&mut app, devices[0]);
        told(&mut app, devices[1]);
        // A purchase is at the database: a part painted meanwhile waits
        // behind it, and is done, kept and told once it has answered.
        ask(&mut app, devices[0], bag::Ask::Buy("cafechair".into()));
        ask(&mut app, devices[0], bag::Ask::Paint { id: 1, part: "lampostbody".into(), colour: [0, 255, 0] });
        assert_eq!(app.world().get::<Holds>(bodies[0]).unwrap().paints.0.len(), 1);
        assert!(told(&mut app, devices[1]).is_empty());
        answer_the_deal(&mut app, Some(950));
        let holds = app.world().get::<Holds>(bodies[0]).unwrap();
        assert_eq!((holds.bag.count("cafechair"), holds.paints.0.len()), (1, 2));
        let second = told(&mut app, devices[1]);
        assert_eq!(painted(&second, 100).len(), 1, "{second:?}");
        assert_eq!(painted(&second, 100)[0].1.len(), 2);
        // The purchase's own answer and the paint's: the bag each time,
        // and the colours after it.
        let first = told(&mut app, devices[0]);
        let answers: Vec<u32> = painted(&first, 100).iter().map(|(asked, _)| *asked).collect();
        assert_eq!(answers, [5, 6], "{first:?}");
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
