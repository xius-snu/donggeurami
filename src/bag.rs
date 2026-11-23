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
//! **What you have painted** of what stands on your plot ([`Bag::paints`],
//! since 2026-11-22; Hajun: "u can tap objects and it will paint it to that
//! color ... color gets reset if u trash that item and place it back, but
//! stays if u leave it and also the new color should be visible to other
//! players") is beside the bag, by the same rules (`rules::Paints`): done
//! here at once and told to the server ([`Bag::paint`]), whose word comes
//! after each of its answers about the bag ([`Bag::heard_paints`]) to an
//! app that has asked for colours (`rules::Ask::Colours`), and kept in the
//! same file.
//!
//! What stands on the plots of the others in your channel of the town is
//! here too ([`Theirs`]), and what of it they have painted
//! ([`TheirPaints`]), as the server says it, for `build::stand` to draw.
//! The window itself is [`panel`]; your gems are on the bar across the top
//! (`hud`).

use std::collections::HashMap;

use bevy::prelude::*;
use roundtown_net::bag as rules;
use roundtown_net::{Ask, At, Tell};
use serde::{Deserialize, Serialize};

use crate::Player;
use crate::land::Deeds;
use crate::lobby::Balance;
use crate::net::Online;

pub(crate) mod panel;

pub(crate) use rules::{Group, Paint, Paints, Standing, THINGS, Thing};

/// The file your bag is kept in, beside the town's save, and its version.
const FILE: &str = "bag.json";
const VERSION: u32 = 1;

/// Your bag.
#[derive(Resource, Debug)]
pub(crate) struct Bag {
    /// What you own, as last known: the server's word online, this
    /// device's own offline.
    pub has: rules::Bag,
    /// What you have painted of what stands on your plot, as last known
    /// the same way.
    pub paints: Paints,
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
        Self::holding(rules::Bag::new(), Paints::default())
    }
}

impl Bag {
    fn holding(has: rules::Bag, paints: Paints) -> Self {
        Self {
            has,
            paints,
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
    /// way: as it was made, whatever colour any of it was painted.
    pub(crate) fn take(&mut self, online: bool, asks: &mut MessageWriter<Ask>, id: u32) -> bool {
        if self.has.take(id).is_err() {
            return false;
        }
        self.paints.fit(&self.has.stands);
        if online {
            self.ask(asks, rules::Ask::Take { id });
        }
        true
    }

    /// Paints `part` of what stands under the number `id` this colour, the
    /// same way: here at once, and, `online`, told to the server. Whether
    /// it was done; and nothing is said of a part that is that colour
    /// already.
    pub(crate) fn paint(&mut self, online: bool, asks: &mut MessageWriter<Ask>, id: u32, part: &str, colour: [u8; 3]) -> bool {
        if self.paints.colour(id, part) == Some(colour) {
            return true;
        }
        if self.paints.set(&self.has.stands, id, part, colour).is_err() {
            return false;
        }
        if online {
            self.ask(asks, rules::Ask::Paint {
                id,
                part: part.to_string(),
                colour,
            });
        }
        true
    }

    /// Takes the paint off `part` of what stands under the number `id`,
    /// the same way: as its model has it again. Whether it was painted.
    pub(crate) fn unpaint(&mut self, online: bool, asks: &mut MessageWriter<Ask>, id: u32, part: &str) -> bool {
        if !self.paints.clear(id, part) {
            return false;
        }
        if online {
            self.ask(asks, rules::Ask::Unpaint {
                id,
                part: part.to_string(),
            });
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
            // Until the server says what is painted, or if it never does:
            // nothing of what no longer stands.
            self.paints.fit(&self.has.stands);
        }
        true
    }

    /// The server has said what is painted on your plot, having answered
    /// `asked` of this connection's asks: it is that now, unless more have
    /// been sent since. Whether it was taken.
    fn heard_paints(&mut self, asked: u32, paints: &[Paint]) -> bool {
        if asked < self.sent {
            return false;
        }
        if self.paints.0 != paints {
            self.paints = Paints(paints.to_vec()).sane(&self.has.stands);
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
        self.paints = Paints::default();
        self.begin();
    }
}

/// What stands on the plot of each of the others in your channel of the
/// town, by who they are (`Member::id`), as the server has said
/// (`bag::Tell::Theirs`). Nobody's offline.
#[derive(Resource, Default, Debug)]
pub(crate) struct Theirs(pub HashMap<u64, Vec<Standing>>);

/// What each of them has painted of what stands on their plot, the same
/// way (`bag::Tell::Painted`).
#[derive(Resource, Default, Debug)]
pub(crate) struct TheirPaints(pub HashMap<u64, Vec<Paint>>);

/// A purchase or a sale, asked for on the panel.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Deal {
    Buy(&'static str),
    Sell(&'static str),
}

/// The file as written: the bag, and what is painted of what stands
/// (nothing, in a file written before 2026-11-22).
#[derive(Serialize, Deserialize)]
struct Saved {
    version: u32,
    bag: rules::Bag,
    #[serde(default)]
    paints: Paints,
}

/// Hearing what the server says of bags and doing what the panel asked:
/// whatever draws what is in the bag comes after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BagSystems;

pub(crate) fn plugin(app: &mut App) {
    let (has, paints) = load();
    app.insert_resource(Bag::holding(has, paints))
        .init_resource::<Theirs>()
        .init_resource::<TheirPaints>()
        .add_message::<Deal>()
        .add_systems(Update, (come_and_go, hear, deal, save).chain().in_set(BagSystems).after(crate::land::DeedSystems));
    panel::plugin(app);
}

/// The bag kept from the last run, and what was painted of what stood, put
/// right if they were kept wrongly; with none, or one that will not read,
/// what everyone begins with.
fn load() -> (rules::Bag, Paints) {
    let fresh = || (rules::Bag::new(), Paints::default());
    let Some(path) = crate::map::data_file(FILE) else {
        return fresh();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return fresh();
    };
    match serde_json::from_str::<Saved>(&text) {
        Ok(saved) if saved.version == VERSION => kept_right(saved),
        Ok(_) => fresh(),
        Err(error) => {
            warn!("{} is not a bag ({error}): begun afresh", path.display());
            fresh()
        }
    }
}

/// What a file held, as it may be.
fn kept_right(saved: Saved) -> (rules::Bag, Paints) {
    let bag = saved.bag.sane();
    let paints = saved.paints.sane(&bag.stands);
    (bag, paints)
}

/// Keeps the bag for the next run whenever what is in it changes, or what
/// is painted: written beside the file and renamed over it, as the profile
/// is.
fn save(bag: Res<Bag>, mut kept: Local<Option<(rules::Bag, Paints)>>) {
    if !bag.is_changed() || kept.as_ref().is_some_and(|(has, paints)| *has == bag.has && *paints == bag.paints) {
        return;
    }
    // What it was loaded as is what the file has.
    let loaded = kept.is_none() && bag.is_added();
    *kept = Some((bag.has.clone(), bag.paints.clone()));
    if loaded {
        return;
    }
    let Some(path) = crate::map::data_file(FILE) else {
        return;
    };
    let saved = Saved {
        version: VERSION,
        bag: bag.has.clone(),
        paints: bag.paints.clone(),
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
/// here on, its own first ([`rules::Ask::Watch`]), and of colours with
/// them ([`rules::Ask::Colours`]). Lets go of the land of whoever has no
/// plot here any more, as whose each plot is changes (`land::Deeds`): what
/// the server said of it is not said again when they are gone. And
/// offline, where nobody says anything, stands on every plot but yours its
/// own made-up land (`roundtown_net::premade`: the very land the server
/// fills a plot with when no real person's does, so that a plot filled
/// here is the same once it has spoken).
fn come_and_go(
    online: Res<Online>,
    deeds: Res<Deeds>,
    mut was: Local<bool>,
    mut bag: ResMut<Bag>,
    (mut theirs, mut their_paints): (ResMut<Theirs>, ResMut<TheirPaints>),
    mut asks: MessageWriter<Ask>,
) {
    let now = online.is();
    let turned = now != *was;
    if turned {
        *was = now;
        bag.begin();
        if now {
            bag.ask(&mut asks, rules::Ask::Watch);
            bag.ask(&mut asks, rules::Ask::Colours);
        }
    }
    if !(turned || deeds.is_changed()) {
        return;
    }
    if now {
        if deeds.0.is_empty() {
            return;
        }
        if theirs.0.keys().any(|who| !deeds.has(*who)) {
            theirs.0.retain(|who, _| deeds.has(*who));
        }
        if their_paints.0.keys().any(|who| !deeds.has(*who)) {
            their_paints.0.retain(|who, _| deeds.has(*who));
        }
        return;
    }
    let (stands, paints) = made_up(&deeds);
    if theirs.0 != stands {
        theirs.0 = stands;
    }
    if their_paints.0 != paints {
        their_paints.0 = paints;
    }
}

/// What stands on each plot `deeds` gives a made-up owner, and what of it
/// is painted, by that owner: the plot's own made-up land.
fn made_up(deeds: &Deeds) -> (HashMap<u64, Vec<Standing>>, HashMap<u64, Vec<Paint>>) {
    let lots = deeds
        .0
        .iter()
        .filter(|deed| roundtown_net::land::is_premade(deed.who))
        .map(|deed| roundtown_net::premade::lot(usize::from(deed.plot)));
    let (mut stands, mut paints) = (HashMap::new(), HashMap::new());
    for lot in lots {
        paints.insert(lot.who, lot.paints);
        stands.insert(lot.who, lot.stands);
    }
    (stands, paints)
}

/// What the server says of bags: your own, what stands on the plots of the
/// others in your channel, and what is painted of either.
fn hear(
    mut tells: MessageReader<Tell>,
    online: Res<Online>,
    mut bag: ResMut<Bag>,
    (mut theirs, mut their_paints): (ResMut<Theirs>, ResMut<TheirPaints>),
) {
    let me = online.me().map(|me| me.id);
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
            rules::Tell::Painted { who, asked, paints } if Some(*who) == me => {
                // Only where it differs: the bag is watched for changes.
                if *asked >= bag.sent && bag.paints.0 != *paints {
                    bag.heard_paints(*asked, paints);
                }
            }
            rules::Tell::Painted { who, paints, .. } => {
                if their_paints.0.get(who) != Some(paints) {
                    their_paints.0.insert(*who, paints.clone());
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
        world.init_resource::<TheirPaints>();
        world.init_resource::<Deeds>();
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
        assert_eq!(asked(&mut world), [rules::Ask::Watch, rules::Ask::Colours]);
        world.write_message(Deal::Buy("singlebed"));
        world.write_message(Deal::Buy("cafechair"));
        world.run_system_once(deal).unwrap();
        world.resource_mut::<Messages<Deal>>().clear();
        // The first is asked; the second, come while it was on its way, is
        // dropped. Nothing is changed here yet.
        assert_eq!(asked(&mut world), [rules::Ask::Buy("singlebed".into())]);
        assert!(world.resource::<Bag>().waits());
        assert_eq!((world.resource::<Bag>().has.count("singlebed"), balance(&mut world)), (0, 1_000));
        // The server's answer to the first things sent, the watch and the
        // ask for colours, is not the purchase's: still waiting, and its
        // bag passed over.
        let mut bought = rules::Bag::new();
        bought.buy("singlebed", 1_000).unwrap();
        assert!(!world.resource_mut::<Bag>().heard(2, &rules::Bag::new()));
        assert!(world.resource::<Bag>().waits());
        // Its answer to the purchase is.
        assert!(world.resource_mut::<Bag>().heard(3, &bought));
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
        assert!(!world.resource_mut::<Bag>().heard(2, &rules::Bag::new()));
        assert_eq!(world.resource::<Bag>().has.stands.len(), 1);
        // The answer to the lamp itself is the server's word: here, that it
        // would not have it.
        assert!(world.resource_mut::<Bag>().heard(3, &rules::Bag::new()));
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
    fn what_stands_on_someone_s_plot_is_kept_while_they_have_a_plot_here_and_offline_every_plot_has_its_own() {
        use roundtown_net::land::{Deed, Holder, PLOTS, PREMADE, offline_deeds};
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
        // The server says whose each plot is: 9 has none any more.
        let deed = |who, plot| Deed {
            plot,
            who,
            name: "someone".into(),
            look: "boy".into(),
            holder: Holder::Here,
        };
        world.resource_mut::<Deeds>().0 = vec![deed(8, 1), deed(PREMADE + 2, 2)];
        world.run_system(come_and_go).unwrap();
        assert_eq!(world.resource::<Theirs>().0.keys().copied().collect::<Vec<_>>(), [8]);
        // Offline every plot but yours has its made-up owner, and what
        // stands on each is that owner's own land, with what of it is
        // painted.
        world.insert_resource(Online::default());
        world.resource_mut::<Deeds>().0 = offline_deeds(0);
        world.run_system(come_and_go).unwrap();
        let theirs = world.resource::<Theirs>();
        assert_eq!(theirs.0.len(), PLOTS - 1);
        for plot in 1..PLOTS {
            let lot = roundtown_net::premade::lot(plot);
            assert_eq!(theirs.0.get(&lot.who), Some(&lot.stands), "plot {plot}");
            assert_eq!(world.resource::<TheirPaints>().0.get(&lot.who), Some(&lot.paints), "plot {plot}");
        }
        assert!(!theirs.0.contains_key(&PREMADE));
        // Nothing is written again while nothing changes.
        world.run_system(come_and_go).unwrap();
        // Online again, what the server has not spoken of yet stays as it
        // was until it says whose each plot is.
        world.insert_resource(Online::for_tests(Member {
            id: 7,
            name: "me".into(),
            look: "boy".into(),
            ai: false,
        }));
        world.run_system(come_and_go).unwrap();
        assert_eq!(world.resource::<Theirs>().0.len(), PLOTS - 1);
    }

    #[test]
    fn a_bag_is_kept_as_it_is_and_read_back_the_same() {
        let mut bag = rules::Bag::new();
        bag.buy("doublebed", 1_000).unwrap();
        bag.put(3, LAMP, here()).unwrap();
        let mut paints = Paints::default();
        paints.set(&bag.stands, 3, "lampostbody", [200, 30, 30]).unwrap();
        let saved = Saved {
            version: VERSION,
            bag: bag.clone(),
            paints: paints.clone(),
        };
        let text = serde_json::to_string(&saved).unwrap();
        let back: Saved = serde_json::from_str(&text).unwrap();
        assert_eq!(back.version, VERSION);
        assert_eq!(kept_right(back), (bag.clone(), paints));
        // A file from before there were colours reads as it did, with
        // nothing painted.
        let before = format!(r#"{{"version":{VERSION},"bag":{}}}"#, serde_json::to_string(&bag).unwrap());
        let back: Saved = serde_json::from_str(&before).unwrap();
        assert_eq!(kept_right(back), (bag, Paints::default()));
    }

    #[test]
    fn a_part_is_painted_at_once_and_told_and_the_server_s_word_is_what_it_is() {
        let mut world = world(true);
        world.run_system_once(come_and_go).unwrap();
        asked(&mut world);
        let blue = [0, 0, 255];
        world
            .run_system_once(move |mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>| {
                assert!(bag.put(true, &mut asks, 1, LAMP, here()));
                assert!(bag.paint(true, &mut asks, 1, "lampostbody", blue));
                // The same colour again is nothing to say; a thing that
                // does not stand is not painted.
                assert!(bag.paint(true, &mut asks, 1, "lampostbody", blue));
                assert!(!bag.paint(true, &mut asks, 8, "lampostbody", blue));
            })
            .unwrap();
        let paint = rules::Ask::Paint { id: 1, part: "lampostbody".into(), colour: blue };
        assert_eq!(asked(&mut world), [rules::Ask::Put { id: 1, kind: LAMP.into(), at: here() }, paint]);
        assert_eq!(world.resource::<Bag>().paints.colour(1, "lampostbody"), Some(blue));
        // The server's answer to the lamp, the third thing asked, has no
        // colour yet: passed over, a newer one being on its way.
        let mut stood = rules::Bag::new();
        stood.put(1, LAMP, here()).unwrap();
        let said = |asked, paints: Vec<Paint>| Tell::Bag(rules::Tell::Painted { who: 7, asked, paints });
        world.write_message(Tell::Bag(rules::Tell::Yours { asked: 3, bag: stood.clone() }));
        world.write_message(said(3, Vec::new()));
        world.run_system_once(hear).unwrap();
        world.resource_mut::<Messages<Tell>>().clear();
        assert_eq!(world.resource::<Bag>().paints.colour(1, "lampostbody"), Some(blue));
        // Its answer to the paint is its word: here, another colour.
        let green = Paint { id: 1, part: "lampostbody".into(), colour: [0, 255, 0] };
        world.write_message(Tell::Bag(rules::Tell::Yours { asked: 4, bag: stood }));
        world.write_message(said(4, vec![green.clone()]));
        // And someone else's plot is theirs.
        world.write_message(Tell::Bag(rules::Tell::Painted { who: 8, asked: 0, paints: vec![green.clone()] }));
        world.run_system_once(hear).unwrap();
        assert_eq!(world.resource::<Bag>().paints.0, [green.clone()]);
        assert_eq!(world.resource::<TheirPaints>().0[&8], [green]);
        // The paint taken off: at once, and asked; and nothing to ask
        // where there is none.
        asked(&mut world);
        world
            .run_system_once(|mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>| {
                assert!(bag.unpaint(true, &mut asks, 1, "lampostbody"));
                assert!(!bag.unpaint(true, &mut asks, 1, "lampostbody"));
                assert!(bag.paint(true, &mut asks, 1, "lightbulb", [1, 2, 3]));
            })
            .unwrap();
        let unpaint = rules::Ask::Unpaint { id: 1, part: "lampostbody".into() };
        let bulb = rules::Ask::Paint { id: 1, part: "lightbulb".into(), colour: [1, 2, 3] };
        assert_eq!(asked(&mut world), [unpaint, bulb]);
        assert_eq!(world.resource::<Bag>().paints.colour(1, "lampostbody"), None);
        // Taken back into the bag, it is as it was made.
        world
            .run_system_once(|mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>| {
                assert!(bag.take(true, &mut asks, 1));
            })
            .unwrap();
        assert!(world.resource::<Bag>().paints.0.is_empty());
        // Offline it is painted here alone, and nothing is asked.
        asked(&mut world);
        world.insert_resource(Online::default());
        world
            .run_system_once(move |mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>| {
                assert!(bag.put(false, &mut asks, 2, LAMP, here()));
                assert!(bag.paint(false, &mut asks, 2, "lightbulb", blue));
            })
            .unwrap();
        assert!(asked(&mut world).is_empty());
        assert_eq!(world.resource::<Bag>().paints.colour(2, "lightbulb"), Some(blue));
    }
}
