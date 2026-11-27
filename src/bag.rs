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
//!
//! **What works** (since 2026-11-27; `rules::Workings`): what is kept in
//! what keeps things on a plot, and which of its lights are on, yours
//! ([`Bag::workings`]) and everyone else's in your channel
//! ([`TheirWorkings`]). Anyone may put things in, take them out, switch the
//! lights and run a sink's tap ([`Bag::stow`], [`Bag::unstow`],
//! [`Bag::switch`], [`Bag::run`]): online each is asked of the server,
//! whose word on every plot's workings is what there is, yours too
//! (`rules::Tell::Workings`), and whatever comes into your bag is said
//! ([`Taken`]); offline there is only your own plot, and it is done here.
//! Taken back into the bag, a thing takes what was kept in it with it
//! ([`Bag::take`]), and is not taken back if that would not fit.
//!
//! **Piggy banks** (since later on 2026-11-27; `rules::Banked`): the coins
//! in each on a plot, yours ([`Bag::banked`]) and everyone else's in your
//! channel ([`TheirBanked`]), the same way: anyone may put coins in or take
//! them out ([`Bag::deposit`], [`Bag::withdraw`]), online asked of the
//! server, whose word is what is in each and what your balance is, and by
//! how much it changed ([`Coined`]); offline, on your own plot, done here
//! with the coins this device has. A piggy bank with coins in it is not
//! taken back into the bag.

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

pub(crate) use rules::{Banked, Group, Paint, Paints, Stack, Standing, THINGS, Thing, Workings};

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
    /// What works on your plot: what is kept in what keeps things there,
    /// and which lights are on. Online the server's word as it last said,
    /// whoever changed it; offline this device's own.
    pub workings: Workings,
    /// The coins in the piggy banks on your plot, the same way.
    pub banked: Banked,
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
        Self::holding(rules::Bag::new(), Paints::default(), Workings::default(), Banked::default())
    }
}

impl Bag {
    fn holding(has: rules::Bag, paints: Paints, workings: Workings, banked: Banked) -> Self {
        Self {
            has,
            paints,
            workings,
            banked,
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
    /// way: as it was made, whatever colour any of it was painted, and with
    /// whatever was kept in it (`rules::take_back`). Whether it was: not if
    /// what was kept in it would not fit in the bag.
    pub(crate) fn take(&mut self, online: bool, asks: &mut MessageWriter<Ask>, id: u32) -> bool {
        if rules::take_back(&mut self.has, &mut self.workings, &self.banked, id).is_err() {
            return false;
        }
        self.paints.fit(&self.has.stands);
        if online {
            self.ask(asks, rules::Ask::Take { id });
        }
        true
    }

    /// Whether what is kept in the things standing under these numbers
    /// would all fit in the bag, were they taken back into it together.
    pub(crate) fn has_room_for_what_is_in(&self, ids: &[u32]) -> bool {
        let mut kept: Vec<(&str, u32)> = Vec::new();
        for stack in ids.iter().flat_map(|&id| self.workings.slots(id)).flatten() {
            match kept.iter_mut().find(|(kind, _)| *kind == stack.kind) {
                Some((_, count)) => *count += u32::from(stack.count),
                None => kept.push((stack.kind.as_str(), u32::from(stack.count))),
            }
        }
        kept.iter().all(|&(kind, count)| rules::thing(kind).is_none() || u32::from(self.has.room_for(kind)) >= count)
    }

    /// Whether a piggy bank among the things standing under these numbers
    /// has coins in it: none of them is taken back into the bag until they
    /// are taken out ([`rules::No::Banked`]).
    pub(crate) fn has_coins_in(&self, ids: &[u32]) -> bool {
        ids.iter().any(|&id| self.banked.of(id) > 0)
    }

    /// Puts `count` of `kind` out of the bag into slot `slot` of the thing
    /// standing under `id` on `who`'s plot (`None`, yours): asked of the
    /// server online, as `me`, whose word is what comes of it; offline, on
    /// your own plot, done here by the rules. Whether it was done or asked.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stow(&mut self, me: Option<u64>, asks: &mut MessageWriter<Ask>, who: Option<u64>, id: u32, slot: usize, kind: &str, count: u16) -> bool {
        match (me, who) {
            (Some(me), _) => {
                let Ok(slot) = u8::try_from(slot) else {
                    return false;
                };
                self.ask(asks, rules::Ask::Stow { who: who.unwrap_or(me), id, slot, kind: kind.to_string(), count });
                true
            }
            (None, None) => {
                let stands = self.has.stands.clone();
                rules::stow(&mut self.has, &mut self.workings, &stands, id, slot, kind, count).is_ok()
            }
            (None, Some(_)) => false,
        }
    }

    /// Takes what is in slot `slot` of the thing standing under `id` on
    /// `who`'s plot into the bag, the same way: what came of it, offline,
    /// where it is done here and now. Online what comes is said by the
    /// server after ([`Taken`]).
    pub(crate) fn unstow(&mut self, me: Option<u64>, asks: &mut MessageWriter<Ask>, who: Option<u64>, id: u32, slot: usize) -> Result<Option<Stack>, rules::No> {
        match (me, who) {
            (Some(me), _) => {
                let slot = u8::try_from(slot).map_err(|_| rules::No::Place)?;
                self.ask(asks, rules::Ask::Unstow { who: who.unwrap_or(me), id, slot });
                Ok(None)
            }
            (None, None) => {
                let stands = self.has.stands.clone();
                rules::unstow(&mut self.has, &mut self.workings, &stands, id, slot).map(Some)
            }
            (None, Some(_)) => Err(rules::No::None),
        }
    }

    /// Switches the light standing under `id` on `who`'s plot on or off, the
    /// same way. Whether it was done or asked.
    pub(crate) fn switch(&mut self, me: Option<u64>, asks: &mut MessageWriter<Ask>, who: Option<u64>, id: u32, on: bool) -> bool {
        match (me, who) {
            (Some(me), _) => {
                self.ask(asks, rules::Ask::Switch { who: who.unwrap_or(me), id, on });
                true
            }
            (None, None) => {
                let stands = self.has.stands.clone();
                self.workings.switch(&stands, id, on).is_ok()
            }
            (None, Some(_)) => false,
        }
    }

    /// Runs the tap of the sink standing under `id` on `who`'s plot: online
    /// asked of the server, which tells everyone in the channel, you too
    /// (`rules::Tell::Ran`); offline nothing is asked, and whoever ran it
    /// runs it here.
    pub(crate) fn run(&mut self, me: Option<u64>, asks: &mut MessageWriter<Ask>, who: Option<u64>, id: u32) {
        if let Some(me) = me {
            self.ask(asks, rules::Ask::Run { who: who.unwrap_or(me), id });
        }
    }

    /// Puts `coins` of your balance into the piggy bank standing under `id`
    /// on `who`'s plot (`None`, yours): asked of the server online, as `me`,
    /// whose word is what comes of it ([`Coined`], after your balance);
    /// offline, on your own plot, done here by the rules, and said at once.
    /// Whether it was done or asked.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn deposit(
        &mut self,
        me: Option<u64>,
        asks: &mut MessageWriter<Ask>,
        coined: &mut MessageWriter<Coined>,
        balance: &mut Balance,
        who: Option<u64>,
        id: u32,
        coins: u32,
    ) -> bool {
        match (me, who) {
            (Some(me), _) => {
                self.ask(asks, rules::Ask::Deposit { who: who.unwrap_or(me), id, coins });
                true
            }
            (None, None) => {
                let stands = self.has.stands.clone();
                let Ok(left) = rules::deposit(balance.0, &mut self.banked, &stands, id, coins) else {
                    return false;
                };
                balance.0 = left;
                coined.write(Coined(-i64::from(coins)));
                true
            }
            (None, Some(_)) => false,
        }
    }

    /// Takes `coins` out of the piggy bank standing under `id` on `who`'s
    /// plot into your balance, the same way.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn withdraw(
        &mut self,
        me: Option<u64>,
        asks: &mut MessageWriter<Ask>,
        coined: &mut MessageWriter<Coined>,
        balance: &mut Balance,
        who: Option<u64>,
        id: u32,
        coins: u32,
    ) -> bool {
        match (me, who) {
            (Some(me), _) => {
                self.ask(asks, rules::Ask::Withdraw { who: who.unwrap_or(me), id, coins });
                true
            }
            (None, None) => {
                let stands = self.has.stands.clone();
                let Ok(after) = rules::withdraw(balance.0, &mut self.banked, &stands, id, coins) else {
                    return false;
                };
                balance.0 = after;
                coined.write(Coined(i64::from(coins)));
                true
            }
            (None, Some(_)) => false,
        }
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
        self.workings = Workings::default();
        self.banked = Banked::default();
        self.begin();
    }
}

/// The coins in the piggy banks on the plot of each of the others in your
/// channel of the town, by who they are, as the server has said
/// (`rules::Tell::Banked`). Nobody's offline.
#[derive(Resource, Default, Debug)]
pub(crate) struct TheirBanked(pub HashMap<u64, Banked>);

impl TheirBanked {
    /// The coins in the piggy banks on `who`'s plot (`None`, yours, out of
    /// `bag`): none, for a plot the server has said nothing of.
    pub(crate) fn of(&self, bag: &Bag, who: Option<u64>) -> Banked {
        match who {
            None => bag.banked.clone(),
            Some(who) => self.0.get(&who).cloned().unwrap_or_default(),
        }
    }
}

/// So many coins have just come into your balance out of a piggy bank, or,
/// less than none, gone into one: said by the server online
/// (`rules::Tell::Coins`), and offline by whoever did it.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Coined(pub i64);

/// What works on the plot of each of the others in your channel of the
/// town, by who they are, as the server has said (`rules::Tell::Workings`).
/// Nobody's offline.
#[derive(Resource, Default, Debug)]
pub(crate) struct TheirWorkings(pub HashMap<u64, Workings>);

impl TheirWorkings {
    /// What works on `who`'s plot (`None`, yours, out of `bag`).
    pub(crate) fn of<'a>(&'a self, bag: &'a Bag, who: Option<u64>) -> Option<&'a Workings> {
        match who {
            None => Some(&bag.workings),
            Some(who) => self.0.get(&who),
        }
    }
}

/// So many of a kind of thing have just come into your bag, taken out of
/// something that keeps things: said by the server online
/// (`rules::Tell::Got`), and offline by whoever took them.
#[derive(Message, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Taken(pub Stack);

/// The tap of the sink standing under `id` on `who`'s plot (`None`, yours)
/// has been run, as the server says (`rules::Tell::Ran`): `working` fills
/// it.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Ran {
    pub who: Option<u64>,
    pub id: u32,
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

/// The file as written: the bag, what is painted of what stands (nothing,
/// in a file written before 2026-11-22), what works there (nothing, in one
/// written before 2026-11-27) and the coins in its piggy banks (none, in
/// one written before they were).
#[derive(Serialize, Deserialize)]
struct Saved {
    version: u32,
    bag: rules::Bag,
    #[serde(default)]
    paints: Paints,
    #[serde(default)]
    workings: Workings,
    #[serde(default)]
    banked: Banked,
}

/// Hearing what the server says of bags and doing what the panel asked:
/// whatever draws what is in the bag comes after it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BagSystems;

pub(crate) fn plugin(app: &mut App) {
    let (has, paints, workings, banked) = load();
    app.insert_resource(Bag::holding(has, paints, workings, banked))
        .init_resource::<Theirs>()
        .init_resource::<TheirPaints>()
        .init_resource::<TheirWorkings>()
        .init_resource::<TheirBanked>()
        .add_message::<Deal>()
        .add_message::<Taken>()
        .add_message::<Coined>()
        .add_message::<Ran>()
        .add_systems(Update, (come_and_go, hear, deal, save).chain().in_set(BagSystems).after(crate::land::DeedSystems));
    panel::plugin(app);
}

/// What a file holds: the bag, what is painted, what works and the coins in
/// piggy banks.
type Held = (rules::Bag, Paints, Workings, Banked);

/// The bag kept from the last run, what was painted of what stood, what
/// worked there and the coins in its piggy banks, put right if they were
/// kept wrongly; with none, or one that will not read, what everyone begins
/// with.
fn load() -> Held {
    let fresh = || (rules::Bag::new(), Paints::default(), Workings::default(), Banked::default());
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
fn kept_right(saved: Saved) -> Held {
    let bag = saved.bag.sane();
    let paints = saved.paints.sane(&bag.stands);
    let workings = saved.workings.sane(&bag.stands);
    let banked = saved.banked.sane(&bag.stands);
    (bag, paints, workings, banked)
}

/// Keeps the bag for the next run whenever what is in it changes, what is
/// painted, what works or the coins in its piggy banks: written beside the
/// file and renamed over it, as the profile is.
fn save(bag: Res<Bag>, mut kept: Local<Option<Held>>) {
    let same = |(has, paints, workings, banked): &Held| *has == bag.has && *paints == bag.paints && *workings == bag.workings && *banked == bag.banked;
    if !bag.is_changed() || kept.as_ref().is_some_and(same) {
        return;
    }
    // What it was loaded as is what the file has.
    let loaded = kept.is_none() && bag.is_added();
    *kept = Some((bag.has.clone(), bag.paints.clone(), bag.workings.clone(), bag.banked.clone()));
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
        workings: bag.workings.clone(),
        banked: bag.banked.clone(),
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
/// them ([`rules::Ask::Colours`]), what works ([`rules::Ask::Workings`])
/// and the coins in piggy banks ([`rules::Ask::Banked`]).
/// Lets go of the land of whoever has no
/// plot here any more, as whose each plot is changes (`land::Deeds`): what
/// the server said of it is not said again when they are gone. And
/// offline, where nobody says anything, stands on every plot but yours its
/// own made-up land (`roundtown_net::premade`: the very land the server
/// fills a plot with when no real person's does, so that a plot filled
/// here is the same once it has spoken).
#[allow(clippy::type_complexity)]
fn come_and_go(
    online: Res<Online>,
    deeds: Res<Deeds>,
    mut was: Local<bool>,
    mut bag: ResMut<Bag>,
    (mut theirs, mut their_paints, mut their_workings, mut their_banked): (ResMut<Theirs>, ResMut<TheirPaints>, ResMut<TheirWorkings>, ResMut<TheirBanked>),
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
            bag.ask(&mut asks, rules::Ask::Workings);
            bag.ask(&mut asks, rules::Ask::Banked);
        } else {
            if !their_workings.0.is_empty() {
                their_workings.0.clear();
            }
            if !their_banked.0.is_empty() {
                their_banked.0.clear();
            }
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
        if their_workings.0.keys().any(|who| !deeds.has(*who)) {
            their_workings.0.retain(|who, _| deeds.has(*who));
        }
        if their_banked.0.keys().any(|who| !deeds.has(*who)) {
            their_banked.0.retain(|who, _| deeds.has(*who));
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
/// is painted, by that owner: their own made-up land, whichever plot it is
/// on this time.
fn made_up(deeds: &Deeds) -> (HashMap<u64, Vec<Standing>>, HashMap<u64, Vec<Paint>>) {
    let lots = deeds.0.iter().filter_map(|deed| roundtown_net::premade::of(deed.who));
    let (mut stands, mut paints) = (HashMap::new(), HashMap::new());
    for lot in lots {
        paints.insert(lot.who, lot.paints);
        stands.insert(lot.who, lot.stands);
    }
    (stands, paints)
}

/// What the server says of bags: your own, what stands on the plots of the
/// others in your channel, what is painted of either, what works there and
/// the coins in their piggy banks; what has come into your bag out of
/// something that keeps things, a sink run, and coins into your balance out
/// of a piggy bank or out of it into one.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn hear(
    mut tells: MessageReader<Tell>,
    online: Res<Online>,
    mut bag: ResMut<Bag>,
    (mut theirs, mut their_paints, mut their_workings, mut their_banked): (ResMut<Theirs>, ResMut<TheirPaints>, ResMut<TheirWorkings>, ResMut<TheirBanked>),
    (mut taken, mut ran, mut coined): (MessageWriter<Taken>, MessageWriter<Ran>, MessageWriter<Coined>),
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
            // The server's word on what works anywhere, yours too: anyone
            // in the channel may have changed it.
            rules::Tell::Workings { who, workings } if Some(*who) == me => {
                if bag.workings != *workings {
                    bag.workings = workings.clone();
                }
            }
            rules::Tell::Workings { who, workings } => {
                if their_workings.0.get(who) != Some(workings) {
                    their_workings.0.insert(*who, workings.clone());
                }
            }
            rules::Tell::Got(stack) => {
                taken.write(Taken(stack.clone()));
            }
            rules::Tell::Ran { who, id } => {
                ran.write(Ran {
                    who: (Some(*who) != me).then_some(*who),
                    id: *id,
                });
            }
            rules::Tell::Banked { who, banked } if Some(*who) == me => {
                if bag.banked != *banked {
                    bag.banked = banked.clone();
                }
            }
            rules::Tell::Banked { who, banked } => {
                if their_banked.0.get(who) != Some(banked) {
                    their_banked.0.insert(*who, banked.clone());
                }
            }
            rules::Tell::Coins(by) => {
                coined.write(Coined(*by));
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
        world.init_resource::<TheirWorkings>();
        world.init_resource::<TheirBanked>();
        world.init_resource::<Messages<Taken>>();
        world.init_resource::<Messages<Ran>>();
        world.init_resource::<Messages<Coined>>();
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
        assert_eq!(asked(&mut world), [rules::Ask::Watch, rules::Ask::Colours, rules::Ask::Workings, rules::Ask::Banked]);
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
        // asks for colours, for what works and for piggy banks, is not the
        // purchase's: still waiting, and its bag passed over.
        let mut bought = rules::Bag::new();
        bought.buy("singlebed", 1_000).unwrap();
        assert!(!world.resource_mut::<Bag>().heard(4, &rules::Bag::new()));
        assert!(world.resource::<Bag>().waits());
        // Its answer to the purchase is.
        assert!(world.resource_mut::<Bag>().heard(5, &bought));
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
        assert!(world.resource_mut::<Bag>().heard(5, &rules::Bag::new()));
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
        use roundtown_net::land::{Deed, Holder, Order, PLOTS, PREMADE, offline_deeds};
        // Every plot but `mine` with its made-up owner, as the land's own
        // offline deeds are while the made-up land fills plots
        // (`roundtown_net::premade::FILLS`, off since 2026-11-25, when
        // offline every plot but yours is bare).
        assert!(offline_deeds(0, &Order::numbered()).is_empty());
        let made_up = |mine: usize, order: &Order| -> Vec<Deed> {
            let mut deeds: Vec<Deed> = (0..PLOTS)
                .filter(|&place| order.plot(place) != mine)
                .map(|place| {
                    let lot = roundtown_net::premade::lot(place);
                    Deed {
                        plot: order.plot(place) as u8,
                        who: lot.who,
                        name: lot.name,
                        look: lot.look,
                        holder: Holder::Minded,
                    }
                })
                .collect();
            deeds.sort_by_key(|deed| deed.plot);
            deeds
        };
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
        world.resource_mut::<Deeds>().0 = made_up(0, &Order::numbered());
        world.run_system(come_and_go).unwrap();
        let theirs = world.resource::<Theirs>();
        assert_eq!(theirs.0.len(), PLOTS - 1);
        for plot in 1..PLOTS {
            let lot = roundtown_net::premade::lot(plot);
            assert_eq!(theirs.0.get(&lot.who), Some(&lot.stands), "plot {plot}");
            assert_eq!(world.resource::<TheirPaints>().0.get(&lot.who), Some(&lot.paints), "plot {plot}");
        }
        assert!(!theirs.0.contains_key(&PREMADE));
        // In an order the device has thrown, each owner's land is their
        // own all the same, on whichever plot they are.
        let order = Order::thrown(11);
        world.resource_mut::<Deeds>().0 = made_up(order.plot(0), &order);
        world.run_system(come_and_go).unwrap();
        let theirs = world.resource::<Theirs>();
        assert_eq!(theirs.0.len(), PLOTS - 1);
        for deed in &world.resource::<Deeds>().0 {
            let lot = roundtown_net::premade::of(deed.who).unwrap();
            assert_eq!(theirs.0.get(&deed.who), Some(&lot.stands), "plot {}", deed.plot);
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
        bag.take_in("lamp_with_shade", 1).unwrap();
        bag.put(4, "lamp_with_shade", here()).unwrap();
        let workings = Workings { held: Vec::new(), lit: vec![4] };
        bag.take_in(rules::PIGGY, 1).unwrap();
        bag.put(5, rules::PIGGY, here()).unwrap();
        let banked = Banked(vec![(5, 77)]);
        let saved = Saved {
            version: VERSION,
            bag: bag.clone(),
            paints: paints.clone(),
            workings: workings.clone(),
            banked: banked.clone(),
        };
        let text = serde_json::to_string(&saved).unwrap();
        let back: Saved = serde_json::from_str(&text).unwrap();
        assert_eq!(back.version, VERSION);
        assert_eq!(kept_right(back), (bag.clone(), paints.clone(), workings.clone(), banked));
        // A file from before there were colours reads as it did, with
        // nothing painted, one from before anything worked with nothing at
        // work, and one from before piggy banks with no coins in any.
        let before = format!(r#"{{"version":{VERSION},"bag":{}}}"#, serde_json::to_string(&bag).unwrap());
        let back: Saved = serde_json::from_str(&before).unwrap();
        assert_eq!(kept_right(back), (bag.clone(), Paints::default(), Workings::default(), Banked::default()));
        let before = format!(r#"{{"version":{VERSION},"bag":{},"paints":{}}}"#, serde_json::to_string(&bag).unwrap(), serde_json::to_string(&paints).unwrap());
        let back: Saved = serde_json::from_str(&before).unwrap();
        assert_eq!(kept_right(back), (bag.clone(), paints.clone(), Workings::default(), Banked::default()));
        let before = format!(
            r#"{{"version":{VERSION},"bag":{},"paints":{},"workings":{}}}"#,
            serde_json::to_string(&bag).unwrap(),
            serde_json::to_string(&paints).unwrap(),
            serde_json::to_string(&workings).unwrap()
        );
        let back: Saved = serde_json::from_str(&before).unwrap();
        assert_eq!(kept_right(back), (bag, paints, workings, Banked::default()));
    }

    #[test]
    fn coins_go_into_a_piggy_bank_here_offline_on_your_own_plot_and_are_asked_of_the_server_online() {
        // Offline: forty of your thousand into your piggy bank and ten out,
        // here and now, each said; none into anyone else's; and the piggy
        // bank not taken back into the bag with coins in it.
        let mut alone = world(false);
        alone
            .run_system_once(
                |mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>, mut coined: MessageWriter<Coined>, mut me: Query<&mut Balance>| {
                    let mut balance = me.single_mut().unwrap();
                    bag.has.take_in(rules::PIGGY, 1).unwrap();
                    assert!(bag.put(false, &mut asks, 1, rules::PIGGY, here()));
                    assert!(bag.deposit(None, &mut asks, &mut coined, &mut balance, None, 1, 40));
                    assert!(bag.withdraw(None, &mut asks, &mut coined, &mut balance, None, 1, 10));
                    assert_eq!((balance.0, bag.banked.of(1)), (970, 30));
                    assert!(!bag.deposit(None, &mut asks, &mut coined, &mut balance, Some(9), 1, 5));
                    assert!(!bag.deposit(None, &mut asks, &mut coined, &mut balance, None, 1, 971));
                    assert!(!bag.withdraw(None, &mut asks, &mut coined, &mut balance, None, 1, 31));
                    assert!(bag.has_coins_in(&[2, 1]) && !bag.has_coins_in(&[2]));
                    assert!(!bag.take(false, &mut asks, 1));
                    assert!(bag.withdraw(None, &mut asks, &mut coined, &mut balance, None, 1, 30));
                    assert!(bag.take(false, &mut asks, 1));
                    assert_eq!(balance.0, 1_000);
                },
            )
            .unwrap();
        assert!(asked(&mut alone).is_empty());
        let coined: Vec<Coined> = alone.resource_mut::<Messages<Coined>>().drain().collect();
        assert_eq!(coined, [Coined(-40), Coined(10), Coined(30)]);
        // Online: asked, of your own plot as you, of anyone else's as whose
        // it is; nothing done here until the server says, and then as it
        // says.
        let mut world = world(true);
        world.run_system_once(come_and_go).unwrap();
        asked(&mut world);
        world
            .run_system_once(
                |mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>, mut coined: MessageWriter<Coined>, mut me: Query<&mut Balance>| {
                    let mut balance = me.single_mut().unwrap();
                    assert!(bag.deposit(Some(7), &mut asks, &mut coined, &mut balance, Some(9), 4, 25));
                    assert!(bag.withdraw(Some(7), &mut asks, &mut coined, &mut balance, None, 5, 3));
                    assert_eq!(balance.0, 1_000);
                },
            )
            .unwrap();
        assert_eq!(
            asked(&mut world),
            [rules::Ask::Deposit { who: 9, id: 4, coins: 25 }, rules::Ask::Withdraw { who: 7, id: 5, coins: 3 }]
        );
        world.write_message(Tell::Bag(rules::Tell::Banked { who: 9, banked: Banked(vec![(4, 25)]) }));
        world.write_message(Tell::Bag(rules::Tell::Banked { who: 7, banked: Banked(vec![(5, 2)]) }));
        world.write_message(Tell::Bag(rules::Tell::Coins(-25)));
        world.run_system_once(hear).unwrap();
        assert_eq!(world.resource::<Bag>().banked, Banked(vec![(5, 2)]));
        assert_eq!(world.resource::<TheirBanked>().of(world.resource::<Bag>(), Some(9)), Banked(vec![(4, 25)]));
        let coined: Vec<Coined> = world.resource_mut::<Messages<Coined>>().drain().collect();
        assert_eq!(coined, [Coined(-25)]);
    }

    #[test]
    fn what_works_is_done_here_offline_on_your_own_plot_and_asked_of_the_server_online() {
        // Offline: three hamburgers into a cabinet of yours and out again,
        // here and now, and none into anyone else's.
        let mut alone = world(false);
        alone
            .run_system_once(|mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>| {
                bag.has.take_in("cabinet", 1).unwrap();
                bag.has.take_in("hamburger", 3).unwrap();
                assert!(bag.put(false, &mut asks, 1, "cabinet", here()));
                assert!(bag.stow(None, &mut asks, None, 1, 0, "hamburger", 3));
                assert_eq!(bag.has.count("hamburger"), 0);
                assert!(!bag.stow(None, &mut asks, Some(8), 1, 0, "hamburger", 1));
                let came = bag.unstow(None, &mut asks, None, 1, 0);
                assert_eq!(came, Ok(Some(Stack { kind: "hamburger".into(), count: 3 })));
                // Taken back into the bag with one in it, the one comes too.
                assert!(bag.stow(None, &mut asks, None, 1, 2, "hamburger", 1));
                assert!(bag.has_room_for_what_is_in(&[1]));
                assert!(bag.take(false, &mut asks, 1));
                assert_eq!((bag.has.count("hamburger"), bag.has.free("cabinet")), (3, 1));
                assert!(bag.workings.is_empty());
            })
            .unwrap();
        assert!(asked(&mut alone).is_empty());
        // Online: each is asked, of your own plot as you, of anyone else's
        // as whose it is; nothing is done here until the server says.
        let mut world = world(true);
        world.run_system_once(come_and_go).unwrap();
        asked(&mut world);
        world
            .run_system_once(|mut bag: ResMut<Bag>, mut asks: MessageWriter<Ask>| {
                bag.has.take_in("doll", 2).unwrap();
                assert!(bag.stow(Some(7), &mut asks, Some(9), 4, 1, "doll", 2));
                assert_eq!(bag.unstow(Some(7), &mut asks, None, 5, 0), Ok(None));
                assert!(bag.switch(Some(7), &mut asks, Some(9), 6, true));
                bag.run(Some(7), &mut asks, None, 3);
            })
            .unwrap();
        assert_eq!(
            asked(&mut world),
            [
                rules::Ask::Stow { who: 9, id: 4, slot: 1, kind: "doll".into(), count: 2 },
                rules::Ask::Unstow { who: 7, id: 5, slot: 0 },
                rules::Ask::Switch { who: 9, id: 6, on: true },
                rules::Ask::Run { who: 7, id: 3 },
            ]
        );
        assert_eq!(world.resource::<Bag>().has.count("doll"), 2);
        // The server's word: what works on your plot and on theirs, what
        // came into your bag, and two sinks run, theirs and yours.
        let works = Workings { held: Vec::new(), lit: vec![6] };
        world.write_message(Tell::Bag(rules::Tell::Workings { who: 7, workings: works.clone() }));
        world.write_message(Tell::Bag(rules::Tell::Workings { who: 9, workings: works.clone() }));
        world.write_message(Tell::Bag(rules::Tell::Got(Stack { kind: "doll".into(), count: 2 })));
        world.write_message(Tell::Bag(rules::Tell::Ran { who: 9, id: 3 }));
        world.write_message(Tell::Bag(rules::Tell::Ran { who: 7, id: 3 }));
        world.run_system_once(hear).unwrap();
        assert_eq!(world.resource::<Bag>().workings, works);
        assert_eq!(world.resource::<TheirWorkings>().0[&9], works);
        let taken: Vec<Taken> = world.resource_mut::<Messages<Taken>>().drain().collect();
        assert_eq!(taken, [Taken(Stack { kind: "doll".into(), count: 2 })]);
        let ran: Vec<Ran> = world.resource_mut::<Messages<Ran>>().drain().collect();
        assert_eq!(ran, [Ran { who: Some(9), id: 3 }, Ran { who: None, id: 3 }]);
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
        // The server's answer to the lamp, the fifth thing asked, has no
        // colour yet: passed over, a newer one being on its way.
        let mut stood = rules::Bag::new();
        stood.put(1, LAMP, here()).unwrap();
        let said = |asked, paints: Vec<Paint>| Tell::Bag(rules::Tell::Painted { who: 7, asked, paints });
        world.write_message(Tell::Bag(rules::Tell::Yours { asked: 5, bag: stood.clone() }));
        world.write_message(said(5, Vec::new()));
        world.run_system_once(hear).unwrap();
        world.resource_mut::<Messages<Tell>>().clear();
        assert_eq!(world.resource::<Bag>().paints.colour(1, "lampostbody"), Some(blue));
        // Its answer to the paint is its word: here, another colour.
        let green = Paint { id: 1, part: "lampostbody".into(), colour: [0, 255, 0] };
        world.write_message(Tell::Bag(rules::Tell::Yours { asked: 6, bag: stood }));
        world.write_message(said(6, vec![green.clone()]));
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
