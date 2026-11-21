//! Your bag: what you own, and what of it stands on your plot of the town
//! (since 2026-11-21; Hajun: "im gonna change the emote button in the lobby
//! to a inventory/shop panel ... the first tab is a inventory tab and the
//! second tab is the shop tab").
//!
//! **What there is to own** is `roundtown_net::bag`'s list of things: every
//! piece House Builder's shop sells, and the car you drive. Each has a
//! rarity, the colour its name is written in (`colours.bag`), and a price,
//! which for now is its rarity's; sold back it fetches half. Everyone
//! begins with ten gems, one car and one street lamp. The rules are that
//! module's, the same for the server and for a device by itself.
//!
//! **Whose word it is.** Online your bag is the account's, the server's to
//! say ([`hear`], `Tell::Bag`): a purchase or a sale is asked of it
//! ([`Deal`], `Ask::Bag`) and is done when it answers, with your coins as
//! it has them; what you stand on your plot or take back is done here at
//! once and told to it ([`Bag::put`], [`Bag::take`]), and its answer, the
//! whole bag, puts right whatever it would not have. Offline it is this
//! device's own, bought with the coins it has. Either way the bag as last
//! known is kept in `bag.json` beside the town's save ([`save`]), so that
//! what stands on your plot stands there as the app opens, before any
//! server has been heard from; and whatever the device had is the server's
//! word again as soon as that comes.
//!
//! **Every answer is the whole bag**, numbered: the server says how many of
//! this connection's asks it has answered, and one that answers fewer than
//! have been sent is passed over for the one behind it ([`Bag::heard`]).
//!
//! What stands on the plots of the others in your channel of the town is
//! here too ([`Theirs`]), as the server says it, for `build::stand` to draw.
//! The window itself is [`panel`]; your gems are on the bar across the top
//! (`hud`).

use std::collections::HashMap;

use bevy::prelude::*;
use roundtown_net::bag as rules;
use roundtown_net::{Ask, At, Tell};
use serde::{Deserialize, Serialize};

use crate::Player;
use crate::lobby::{Balance, Channel};
use crate::net::Online;

pub(crate) mod panel;

pub(crate) use rules::{Group, Standing, THINGS, Thing};

/// The file your bag is kept in, beside the town's save, and its version.
const FILE: &str = "bag.json";
const VERSION: u32 = 1;

/// Your bag.
#[derive(Resource, Debug)]
pub(crate) struct Bag {
    /// What you own, as last known: the server's word online, this
    /// device's own offline.
    pub has: rules::Bag,
    /// How many things this connection has asked of the server about the
    /// bag, the highest number of them it has said it has answered, and
    /// which of them was the last purchase or sale, if one is still on its
    /// way.
    sent: u32,
    answered: u32,
    dealing: Option<u32>,
}

impl Default for Bag {
    fn default() -> Self {
        Self::holding(rules::Bag::new())
    }
}

impl Bag {
    fn holding(has: rules::Bag) -> Self {
        Self {
            has,
            sent: 0,
            answered: 0,
            dealing: None,
        }
    }

    /// Whether a purchase or a sale asked of the server has not been
    /// answered yet: nothing more is bought or sold until it has.
    pub(crate) fn waits(&self) -> bool {
        self.dealing.is_some_and(|asked| self.answered < asked)
    }

    /// Asks the server `ask` of the bag: one more thing sent.
    fn ask(&mut self, asks: &mut MessageWriter<Ask>, ask: rules::Ask) {
        self.sent += 1;
        asks.write(Ask::Bag(ask));
    }

    /// Stands one of `kind` on your plot at `at` under the number `id`, or
    /// moves what stands under that number: done here at once, by the
    /// rules, and, `online`, told to the server. Whether it was done.
    pub(crate) fn put(&mut self, online: bool, asks: &mut MessageWriter<Ask>, id: u32, kind: &str, at: At) -> bool {
        if self.has.put(id, kind, at).is_err() {
            return false;
        }
        if online {
            self.ask(asks, rules::Ask::Put {
                id,
                kind: kind.to_string(),
                at,
            });
        }
        true
    }

    /// Takes what stands under the number `id` back into the bag, the same
    /// way.
    pub(crate) fn take(&mut self, online: bool, asks: &mut MessageWriter<Ask>, id: u32) -> bool {
        if self.has.take(id).is_err() {
            return false;
        }
        if online {
            self.ask(asks, rules::Ask::Take { id });
        }
        true
    }

    /// The server has said what the bag is, having answered `asked` of this
    /// connection's asks about it: it is that now, unless more have been
    /// sent since, whose answer is coming with what they did. Whether it
    /// was taken.
    fn heard(&mut self, asked: u32, bag: &rules::Bag) -> bool {
        self.answered = self.answered.max(asked);
        if self.dealing.is_some_and(|dealt| self.answered >= dealt) {
            self.dealing = None;
        }
        if asked < self.sent {
            return false;
        }
        if self.has != *bag {
            self.has = bag.clone().sane();
        }
        true
    }

    /// A connection has been made, or has gone: nothing has been asked on
    /// it.
    fn begin(&mut self) {
        (self.sent, self.answered, self.dealing) = (0, 0, None);
    }

    /// The account is gone (`settings`): the bag is what a new one begins
    /// with, until the server says what the new account has.
    pub(crate) fn forget(&mut self) {
        self.has = rules::Bag::new();
        self.begin();
    }
}

/// What stands on the plot of each of the others in your channel of the
/// town, by who they are (`Member::id`), as the server has said
/// (`bag::Tell::Theirs`). Nobody's offline.
#[derive(Resource, Default, Debug)]
pub(crate) struct Theirs(pub HashMap<u64, Vec<Standing>>);

/// A purchase or a sale, asked for on the panel.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Deal {
    Buy(&'static str),
    Sell(&'static str),
}

/// The file as written.
#[derive(Serialize, Deserialize)]
struct Saved {
    version: u32,
    bag: rules::Bag,
}

/// Hearing what the server says of bags and doing what the panel asked:
/// whatever draws what is in the bag comes after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BagSystems;

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(Bag::holding(load()))
        .init_resource::<Theirs>()
        .add_message::<Deal>()
        .add_systems(Update, (come_and_go, hear, deal, save).chain().in_set(BagSystems));
    panel::plugin(app);
}

/// The bag kept from the last run, put right if it was kept wrongly; with
/// none, or one that will not read, what everyone begins with.
fn load() -> rules::Bag {
    let Some(path) = crate::map::data_file(FILE) else {
        return rules::Bag::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return rules::Bag::new();
    };
    match serde_json::from_str::<Saved>(&text) {
        Ok(saved) if saved.version == VERSION => saved.bag.sane(),
        Ok(_) => rules::Bag::new(),
        Err(error) => {
            warn!("{} is not a bag ({error}): begun afresh", path.display());
            rules::Bag::new()
        }
    }
}

/// Keeps the bag for the next run whenever what is in it changes: written
/// beside the file and renamed over it, as the profile is.
fn save(bag: Res<Bag>, mut kept: Local<Option<rules::Bag>>) {
    if !bag.is_changed() || kept.as_ref() == Some(&bag.has) {
        return;
    }
    // What it was loaded as is what the file has.
    if kept.is_none() && bag.is_added() {
        *kept = Some(bag.has.clone());
        return;
    }
    *kept = Some(bag.has.clone());
    let Some(path) = crate::map::data_file(FILE) else {
        return;
    };
    let saved = Saved {
        version: VERSION,
        bag: bag.has.clone(),
    };
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| serde_json::to_string(&saved).map_err(std::io::Error::other))
        .and_then(|text| {
            let partial = path.with_extension("json.new");
            std::fs::write(&partial, text)?;
            std::fs::rename(&partial, &path)
        });
    if let Err(error) = written {
        warn!("could not keep the bag in {}: {error}", path.display());
    }
}

/// As this device comes online, asks the server to tell it of bags from
/// here on, its own first ([`rules::Ask::Watch`]); and as it goes offline,
/// forgets what stood on everyone else's plot, there being nobody else.
/// And lets go of whoever is no longer in your channel, as the server says
/// who is: what it said of their plot is not said again when they are
/// gone.
fn come_and_go(
    online: Res<Online>,
    channel: Res<Channel>,
    mut was: Local<bool>,
    mut bag: ResMut<Bag>,
    mut theirs: ResMut<Theirs>,
    mut asks: MessageWriter<Ask>,
) {
    let now = online.is();
    if now != *was {
        *was = now;
        bag.begin();
        if now {
            bag.ask(&mut asks, rules::Ask::Watch);
        } else if !theirs.0.is_empty() {
            theirs.0.clear();
        }
    }
    if now && channel.is_changed() && !channel.0.is_empty() {
        let here = |who: &u64| channel.0.iter().any(|seated| seated.who == *who);
        if theirs.0.keys().any(|who| !here(who)) {
            theirs.0.retain(|who, _| here(who));
        }
    }
}

/// What the server says of bags: your own, and what stands on the plots of
/// the others in your channel.
fn hear(mut tells: MessageReader<Tell>, mut bag: ResMut<Bag>, mut theirs: ResMut<Theirs>) {
    for tell in tells.read() {
        let Tell::Bag(told) = tell else {
            continue;
        };
        match told {
            rules::Tell::Yours { asked, bag: yours } => {
                bag.heard(*asked, yours);
            }
            rules::Tell::Theirs { who, stands } => {
                if theirs.0.get(who) != Some(stands) {
                    theirs.0.insert(*who, stands.clone());
                }
            }
        }
    }
}

/// Does what the panel asked: online, asks the server, one purchase or sale
/// at a time; offline, does it here by the rules, with the coins this
/// device has.
fn deal(
    mut deals: MessageReader<Deal>,
    online: Res<Online>,
    mut bag: ResMut<Bag>,
    mut me: Query<&mut Balance, With<Player>>,
    mut asks: MessageWriter<Ask>,
) {
    for &deal in deals.read() {
        if online.is() {
            if bag.waits() {
                continue;
            }
            let ask = match deal {
                Deal::Buy(kind) => rules::Ask::Buy(kind.to_string()),
                Deal::Sell(kind) => rules::Ask::Sell(kind.to_string()),
            };
            bag.ask(&mut asks, ask);
            bag.dealing = Some(bag.sent);
            continue;
        }
        let Ok(mut balance) = me.single_mut() else {
            continue;
        };
        let done = match deal {
            Deal::Buy(kind) => bag.has.buy(kind, balance.0),
            Deal::Sell(kind) => bag.has.sell(kind, balance.0),
        };
        if let Ok(left) = done {
            balance.set_if_neq(Balance(left));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use roundtown_net::Member;
    use rules::{CAR, LAMP};

    use super::*;

    fn here() -> At {
        At::Ground {
            x: 1.0,
            y: 0.0,
            z: -2.0,
            yaw: 0.0,
        }
    }

    fn world(online: bool) -> World {
        let mut world = World::new();
        world.init_resource::<Messages<Ask>>();
        world.init_resource::<Messages<Tell>>();
        world.init_resource::<Messages<Deal>>();
        world.init_resource::<Bag>();
        world.init_resource::<Theirs>();
        world.init_resource::<Channel>();
        let me = Member {
            id: 7,
            name: "me".into(),
            look: "boy".into(),
            ai: false,
        };
        world.insert_resource(if online { Online::for_tests(me) } else { Online::default() });
        world.spawn((Player, Balance(1_000)));
        world
    }

    fn asked(world: &mut World) -> Vec<rules::Ask> {
        world
            .resource_mut::<Messages<Ask>>()
            .drain()
            .filter_map(|ask| match ask {
                Ask::Bag(ask) => Some(ask),
                _ => None,
            })
            .collect()
    }

    fn balance(world: &mut World) -> u32 {
        world.query::<&Balance>().single(world).unwrap().0
    }

    #[test]
    fn offline_a_purchase_and_a_sale_are_this_device_s_own() {
        let mut world = world(false);
        world.write_message(Deal::Buy("singlebed"));
        world.run_system_once(deal).unwrap();
        assert_eq!((world.resource::<Bag>().has.count("singlebed"), balance(&mut world)), (1, 850));
        world.resource_mut::<Messages<Deal>>().clear();
        world.write_message(Deal::Sell(LAMP));
        world.run_system_once(deal).unwrap();
        assert_eq!((world.resource::<Bag>().has.count(LAMP), balance(&mut world)), (0, 2_100));
        // What cannot be afforded is not bought, and nothing is asked of
        // anyone.
        world.resource_mut::<Messages<Deal>>().clear();
        world.write_message(Deal::Buy(CAR));
        world.run_system_once(deal).unwrap();
        assert_eq!((world.resource::<Bag>().has.count(CAR), balance(&mut world)), (1, 2_100));
        assert!(asked(&mut world).is_empty());
    }

    #[test]
    fn online_a_purchase_is_asked_of_the_server_one_at_a_time_and_done_when_it_answers() {
        let mut world = world(true);
        world.run_system_once(come_and_go).unwrap();
        assert_eq!(asked(&mut world), [rules::Ask::Watch]);
        world.write_message(Deal::Buy("singlebed"));
        world.write_message(Deal::Buy("cafechair"));
        world.run_system_once(deal).unwrap();
        world.resource_mut::<Messages<Deal>>().clear();
        // The first is asked; the second, come while it was on its way, is
        // dropped. Nothing is changed here yet.
        assert_eq!(asked(&mut world), [rules::Ask::Buy("singlebed".into())]);
        assert!(world.resource::<Bag>().waits());
        assert_eq!((world.resource::<Bag>().has.count("singlebed"), balance(&mut world)), (0, 1_000));
        // The server's answer to the first thing sent, the watch, is not
        // the purchase's: still waiting, and its bag passed over.
        let mut bought = rules::Bag::new();
        bought.buy("singlebed", 1_000).unwrap();
        assert!(!world.resource_mut::<Bag>().heard(1, &rules::Bag::new()));
        assert!(world.resource::<Bag>().waits());
        // Its answer to the purchase is.
        assert!(world.resource_mut::<Bag>().heard(2, &bought));
        assert!(!world.resource::<Bag>().waits());
        assert_eq!(world.resource::<Bag>().has.count("singlebed"), 1);
    }

    #[test]
    fn what_is_stood_on_the_plot_is_done_at_once_and_told_and_an_older_answer_does_not_undo_it() {
        let mut world = world(true);
        world.run_system_once(come_and_go).unwrap();
        asked(&mut world);
        world
            .run_system_once(|mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>| {
                assert!(bag.put(true, &mut asks, 1, LAMP, here()));
                // The one lamp stands: a second cannot, and nothing is
                // asked for it.
                assert!(!bag.put(true, &mut asks, 2, LAMP, here()));
            })
            .unwrap();
        assert_eq!(asked(&mut world), [rules::Ask::Put { id: 1, kind: LAMP.into(), at: here() }]);
        assert_eq!(world.resource::<Bag>().has.free(LAMP), 0);
        // The answer to the watch, sent before the lamp was stood, has the
        // plot bare: passed over.
        assert!(!world.resource_mut::<Bag>().heard(1, &rules::Bag::new()));
        assert_eq!(world.resource::<Bag>().has.stands.len(), 1);
        // The answer to the lamp itself is the server's word: here, that it
        // would not have it.
        assert!(world.resource_mut::<Bag>().heard(2, &rules::Bag::new()));
        assert!(world.resource::<Bag>().has.stands.is_empty());
        // Taken back offline, nothing is asked.
        world.insert_resource(Online::default());
        world
            .run_system_once(|mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>| {
                assert!(bag.put(false, &mut asks, 3, LAMP, here()));
                assert!(bag.take(false, &mut asks, 3));
                assert!(!bag.take(false, &mut asks, 3));
            })
            .unwrap();
        assert!(asked(&mut world).is_empty());
    }

    #[test]
    fn what_stands_on_someone_s_plot_is_kept_while_they_are_in_your_channel() {
        use roundtown_net::Seated;
        let mut world = world(true);
        // The one system throughout: it remembers whether it was online.
        let come_and_go = world.register_system(come_and_go);
        world.run_system(come_and_go).unwrap();
        let stood = vec![Standing {
            id: 1,
            kind: LAMP.into(),
            at: here(),
        }];
        for who in [8, 9] {
            world.write_message(Tell::Bag(rules::Tell::Theirs {
                who,
                stands: stood.clone(),
            }));
        }
        world.run_system_once(hear).unwrap();
        assert_eq!(world.resource::<Theirs>().0.len(), 2);
        // The server says who is in the channel: 9 has gone.
        let seated = |who, seat| Seated { who, seat, driving: false };
        world.resource_mut::<Channel>().0 = vec![seated(7, 0), seated(8, 1)];
        world.run_system(come_and_go).unwrap();
        assert_eq!(world.resource::<Theirs>().0.keys().copied().collect::<Vec<_>>(), [8]);
        // Offline there is nobody else.
        world.insert_resource(Online::default());
        world.run_system(come_and_go).unwrap();
        assert!(world.resource::<Theirs>().0.is_empty());
    }

    #[test]
    fn a_bag_is_kept_as_it_is_and_read_back_the_same() {
        let mut bag = rules::Bag::new();
        bag.buy("doublebed", 1_000).unwrap();
        bag.put(3, LAMP, here()).unwrap();
        let text = serde_json::to_string(&Saved { version: VERSION, bag: bag.clone() }).unwrap();
        let back: Saved = serde_json::from_str(&text).unwrap();
        assert_eq!((back.version, back.bag.sane()), (VERSION, bag));
    }
}
