//! Rooms: who sees whom.
//!
//! A room is a place where everyone in it sees everyone else: one channel of
//! the town, [`TOWN_SEATS`] to a channel; your home, with only you in it for
//! now; a game of House Builder, its lobby and its plots together; a game of
//! Bridge Wars; or, since 2026-11-18, a table of one of the four games
//! (`tables`). A device is sent only what is in its own room. That is
//! replicon's visibility filters, keyed by room: [`InRoom`] on whatever is
//! sent, a player or a game, and [`ClientRoom`] on the device it may be sent
//! to.
//!
//! A newcomer to the town goes to the fullest channel that still has room, so
//! that the town feels busy rather than spread thin, or to the channel their
//! group is in. **Whoever is in a channel has a plot of it** (`land`,
//! [`Towns`]), and keeps both for as long as they are connected, wherever
//! they go: into a game and back (since 2026-11-23; Hajun: "when u go to
//! minigame, your house should remain there and u should count as someone
//! who is on that channel"), and for a while after their connection has
//! gone.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::land::{Deed, Estate, Lot, Order};
use roundtown_net::{Minigame, TOWN_SEATS};

/// The rooms, the town's channels knowing the land of every account that
/// has kept any (`lands`, read as the server starts): what its plots are
/// filled with when their owners are not there.
pub(crate) fn plugin(app: &mut App, lands: Vec<crate::accounts::Land>) {
    let mut towns = Towns {
        thrown: true,
        ..default()
    };
    for land in &lands {
        towns.record(crate::bag::lot_kept(land.id, &land.name, &land.look, &land.bag));
    }
    info!("{} of {} accounts with a bag have something standing on their land", towns.lots.len(), lands.len());
    app.insert_resource(towns)
        .add_visibility_filter::<InRoom>()
        .add_visibility_filter::<NotFor>();
}

/// One room.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Room {
    /// A channel of the town, counted from 0.
    Town(u16),
    /// The home of the player with this id.
    Home(u64),
    /// A game of House Builder, counted from 0.
    Game(u32),
    /// A game of Bridge Wars, counted from 0 (`arena`, since 2026-11-08):
    /// its waiting room and its islands together.
    Arena(u32),
    /// A table of one of the four games of 2026-11-18 (`tables`; Hajun: "add
    /// these minigames: Blend In, Knock Off, Bridge Race, Last Stand"), by
    /// its game and its number, counted from 0 for each game: from its wait
    /// to its results, everything of it in the one room.
    Table(Minigame, u32),
}

/// The room something sent to devices is in: only the devices in that room
/// are sent it.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
#[component(immutable)]
pub(crate) struct InRoom(pub Room);

/// The room a device is in.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
#[component(immutable)]
pub(crate) struct ClientRoom(pub Room);

impl VisibilityFilter for InRoom {
    type ClientComponent = ClientRoom;
    type Scope = Entity;

    fn is_visible(&self, _client: Entity, room: Option<&ClientRoom>) -> bool {
        room.is_some_and(|room| room.0 == self.0)
    }
}

/// Never sent to this device: a player's own body. Each device moves its own,
/// and has no use for the server's copy of it.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
#[component(immutable)]
pub(crate) struct NotFor(pub Entity);

impl VisibilityFilter for NotFor {
    type ClientComponent = AuthorizedClient;
    type Scope = Entity;

    fn is_visible(&self, client: Entity, _: Option<&AuthorizedClient>) -> bool {
        self.0 != client
    }
}

/// One channel of the town.
#[derive(Default)]
struct Channel {
    /// Whose each of its plots is, and what fills the rest
    /// (`roundtown_net::land`): none while nobody is in the channel, nor
    /// anyone it is kept for, and worked out afresh by whoever comes next.
    estate: Option<Estate>,
    /// The body of everyone of the channel, by who they are.
    bodies: HashMap<u64, Entity>,
}

impl Channel {
    fn count(&self) -> usize {
        self.estate.as_ref().map_or(0, Estate::count)
    }

    fn room(&self) -> usize {
        self.estate.as_ref().map_or(TOWN_SEATS, Estate::room)
    }
}

/// The town's channels, and whose each plot of each is. **A seat of the
/// town is a plot** (`roundtown_net::land::HOMES`, in Hajun's order, since
/// 2026-11-23): it says where its holder arrives and which land is theirs.
/// Whoever sits down in a channel ([`Towns::sit`]) has a plot of it until
/// their body is gone ([`Towns::leave`]), whether they are in the town or
/// away from it in a game ([`Towns::away`]), and for a while after; every
/// other plot is filled with the land of someone who is not here, which is
/// told to whoever is ([`Towns::deeds`], [`Towns::lots_in`]). A channel
/// holds as many people as the town has plots, 48 (since 2026-11-24,
/// Hajun's; 16 until then).
#[derive(Resource, Default)]
pub(crate) struct Towns {
    channels: Vec<Channel>,
    /// Everyone who has a plot: who they are, and in which channel.
    held: HashMap<Entity, (u64, u16)>,
    /// Everyone's land that has anything standing on it, as last known:
    /// what a channel's plots are filled with as it opens ([`Towns::record`]).
    lots: HashMap<u64, Lot>,
    /// The plots someone is standing on just now, by channel
    /// ([`Towns::stood_on`]): none of them is taken from under them.
    standing: HashSet<(u16, usize)>,
    /// The server's own seconds, as last said ([`Towns::tick`]).
    now: f64,
    /// Whether the order a channel's plots are taken in is thrown as it
    /// opens (`roundtown_net::land::Order`; Hajun, 2026-11-24: "every time
    /// a channel is created/launched the plot order is gona be shuffled"),
    /// as it is for the server that runs; a test's are by their numbers.
    thrown: bool,
    /// The channels something has changed in that the devices in them are
    /// told of (`players::tell_channels`, since 2026-11-21): someone has
    /// sat down or gone, got into their car or out of it, or a plot is
    /// filled with something else. Each once, until they are told.
    stirred: Vec<u16>,
    /// Who has sat down in which channel since that was last asked
    /// ([`Towns::take_sat`]): each is told of what stands on the plots of
    /// the channel they have come into, and the channel of what stands on
    /// theirs (`bag::seat_news`, since 2026-11-21).
    sat: Vec<(u16, Entity)>,
}

impl Towns {
    /// Seats `player`, who is `who`, in the town: in the channel they have
    /// a plot in already, if they have, back from a game; otherwise in the
    /// channel `rather` if it has room (their group's, or the one a group
    /// is moved to), otherwise in the one that keeps their plot for them,
    /// their connection having gone a moment ago, otherwise in the one
    /// their land fills a plot of, which is theirs (Hajun, 2026-11-24:
    /// "when u first join u first check if your plot is present in a
    /// certain channel due to another person being on a channel that
    /// summoned ur plot, and if its present, just go to that channel and
    /// take that land plot"), otherwise in the fullest that has room,
    /// otherwise in a new one. **Nothing of theirs is left in any other
    /// channel** ("what matters is that there is no duplicate plots across
    /// channels"): no land of theirs filling a plot, no plot kept for
    /// them, and a channel opens with no land that is in another already.
    /// Returns the channel and the plot (`roundtown_net::land`: their own,
    /// as a channel has it).
    pub(crate) fn sit(&mut self, player: Entity, who: u64, rather: Option<u16>) -> (u16, u8) {
        let kept = self.held.get(&player).map(|&(_, channel)| channel);
        let open = |channel: usize| self.channels.get(channel).is_some_and(|channel| channel.room() > 0);
        // The channel asked for may be the next there is none of yet.
        let asked = |channel: &usize| open(*channel) || *channel == self.channels.len();
        let waits = |channel: &usize| self.channels[*channel].estate.as_ref().is_some_and(|estate| estate.plot_of(who).is_some());
        let shown = |channel: &usize| self.channels[*channel].estate.as_ref().is_some_and(|estate| estate.shows(who));
        let channel = kept
            .map(usize::from)
            .or_else(|| rather.map(usize::from).filter(asked))
            .or_else(|| (0..self.channels.len()).find(waits))
            .or_else(|| (0..self.channels.len()).find(shown))
            .or_else(|| {
                (0..self.channels.len())
                    .filter(|&channel| open(channel))
                    .max_by_key(|&channel| (self.channels[channel].count(), std::cmp::Reverse(channel)))
            })
            .unwrap_or(self.channels.len());
        if channel == self.channels.len() {
            self.channels.push(Channel::default());
        }
        let number = channel as u16;
        // What a channel that opens is filled with: the land of whoever
        // has nothing in any channel yet, neither a plot of their own nor
        // their land filling one.
        let elsewhere = |who: u64| self.channels.iter().filter_map(|channel| channel.estate.as_ref()).any(|estate| estate.has(who));
        let lots: Vec<Lot> = match self.channels[channel].estate {
            Some(_) => Vec::new(),
            None => self.lots.values().filter(|lot| !elsewhere(lot.who)).cloned().collect(),
        };
        let standing = &self.standing;
        let thrown = self.thrown;
        let seats = &mut self.channels[channel];
        let plot = match &mut seats.estate {
            Some(estate) => estate.join(who, |plot| standing.contains(&(number, plot))),
            // Nobody in it, nor anyone it is kept for: opened, everything
            // in it somewhere else than the last time.
            closed => {
                let order = if thrown { Order::thrown(fastrand::u64(..)) } else { Order::numbered() };
                closed.insert(Estate::open(who, lots, order)).plot_of(who)
            }
        }
        .expect("only a channel with room is chosen");
        seats.bodies.insert(who, player);
        self.held.insert(player, (who, number));
        // Here, they have nothing anywhere else: the channels it was in
        // are told what is there instead.
        let mut stirred = vec![number];
        for (other, seats) in self.channels.iter_mut().enumerate() {
            if other != channel && seats.estate.as_mut().is_some_and(|estate| estate.forget(who)) {
                stirred.push(other as u16);
            }
        }
        for channel in stirred {
            self.stir(channel);
        }
        self.announce(number, player);
        (number, plot as u8)
    }

    /// `player` has gone from the town into a game, or home: their plot is
    /// theirs all the while, with nobody of the computer's on it.
    pub(crate) fn away(&mut self, player: Entity) {
        let Some(&(who, channel)) = self.held.get(&player) else {
            return;
        };
        let estate = self.channels.get_mut(usize::from(channel)).and_then(|channel| channel.estate.as_mut());
        if estate.is_some_and(|estate| estate.away(who, true)) {
            self.stir(channel);
        }
    }

    /// `player`'s body is gone, their connection with it, their land as
    /// they leave it being `lot`: their plot is kept as it is for a while
    /// (`land::GRACE`), should they be back.
    pub(crate) fn leave(&mut self, player: Entity, lot: Option<Lot>) {
        let Some((who, channel)) = self.held.remove(&player) else {
            return;
        };
        let now = self.now;
        if let Some(seats) = self.channels.get_mut(usize::from(channel)) {
            if seats.bodies.get(&who) == Some(&player) {
                seats.bodies.remove(&who);
            }
            if let Some(estate) = &mut seats.estate {
                estate.leave(who, lot, now, true);
            }
        }
        self.sat.retain(|&(_, sat)| sat != player);
        self.stir(channel);
    }

    /// Moves `player`, who has a plot in one channel, to `to`: the plot
    /// they leave is filled with something else at once, their land (`lot`)
    /// being in no two channels (it filled the plot they left, for the day
    /// before 2026-11-24). Away from the town they stay away. The plot
    /// they have there; none, and nothing changed, if they have none to
    /// leave, are there already, or it has no room.
    pub(crate) fn transfer(&mut self, player: Entity, to: u16, lot: Option<Lot>) -> Option<u8> {
        let &(who, from) = self.held.get(&player)?;
        if from == to || self.room_in(to) == 0 {
            return None;
        }
        let now = self.now;
        let seats = self.channels.get_mut(usize::from(from))?;
        let away = seats.estate.as_ref().is_some_and(|estate| estate.held().any(|(_, holder, away)| holder == who && away));
        seats.bodies.remove(&who);
        if let Some(estate) = &mut seats.estate {
            estate.leave(who, lot, now, false);
            if estate.deserted() {
                seats.estate = None;
            }
        }
        self.held.remove(&player);
        self.sat.retain(|&(_, sat)| sat != player);
        self.stir(from);
        let (_, plot) = self.sit(player, who, Some(to));
        if away {
            self.away(player);
        }
        Some(plot)
    }

    /// The server's own seconds are `now`: every plot kept longer than a
    /// while for someone whose connection went is let go of, and a channel
    /// nobody is left in is shut, to be worked out afresh.
    pub(crate) fn tick(&mut self, now: f64) {
        self.now = now;
        let mut stirred = Vec::new();
        for (number, channel) in self.channels.iter_mut().enumerate() {
            let Some(estate) = &mut channel.estate else {
                continue;
            };
            if estate.tick(now) {
                stirred.push(number as u16);
            }
            if estate.deserted() {
                channel.estate = None;
                channel.bodies.clear();
            }
        }
        for channel in stirred {
            self.stir(channel);
        }
    }

    /// The plots someone is standing on just now, each its channel and its
    /// number: whoever comes into a channel takes none of them.
    pub(crate) fn stood_on(&mut self, standing: HashSet<(u16, usize)>) {
        self.standing = standing;
    }

    /// The land of the account `lot` is of is this now: what fills a plot
    /// of a channel that opens from here on, if anything stands on it.
    pub(crate) fn record(&mut self, lot: Lot) {
        if lot.stands.is_empty() {
            self.lots.remove(&lot.who);
        } else if self.lots.get(&lot.who) != Some(&lot) {
            self.lots.insert(lot.who, lot);
        }
    }

    /// The account `who` is no more: its land fills nothing from here on.
    pub(crate) fn forget(&mut self, who: u64) {
        self.lots.remove(&who);
        let mut stirred = Vec::new();
        for (number, channel) in self.channels.iter_mut().enumerate() {
            if channel.estate.as_mut().is_some_and(|estate| estate.forget(who)) {
                stirred.push(number as u16);
            }
        }
        for channel in stirred {
            self.stir(channel);
        }
    }

    /// The channel `player` has a plot in, in the town or away from it.
    pub(crate) fn channel_of(&self, player: Entity) -> Option<u16> {
        self.held.get(&player).map(|&(_, channel)| channel)
    }

    /// How many more `channel` has room for: all of it, for one there is not
    /// yet.
    pub(crate) fn room_in(&self, channel: u16) -> usize {
        self.channels.get(usize::from(channel)).map_or(TOWN_SEATS, Channel::room)
    }

    /// The channel with room for `people` more that has the most in it
    /// already, `but` aside; none, if there is none.
    pub(crate) fn roomy(&self, people: usize, but: u16) -> Option<u16> {
        (0..self.channels.len())
            .filter(|&channel| channel != usize::from(but) && self.channels[channel].room() >= people)
            .max_by_key(|&channel| (self.channels[channel].count(), std::cmp::Reverse(channel)))
            .map(|channel| channel as u16)
    }

    /// How many channels there are, shut ones among them: the number a new
    /// one would have.
    pub(crate) fn channels(&self) -> u16 {
        self.channels.len() as u16
    }

    /// `player`, seated in `channel`, is to be told of the channel's plots,
    /// and the channel of theirs, as if they had just sat down: they have,
    /// or their device has only now asked to be told.
    pub(crate) fn announce(&mut self, channel: u16, player: Entity) {
        if !self.sat.contains(&(channel, player)) {
            self.sat.push((channel, player));
        }
    }

    /// Who has sat down in which channel since this was last asked, each
    /// once, those who have gone from it again since left out.
    pub(crate) fn take_sat(&mut self) -> Vec<(u16, Entity)> {
        let sat = std::mem::take(&mut self.sat);
        sat.into_iter()
            .filter(|&(channel, player)| self.held.get(&player).is_some_and(|&(_, held)| held == channel))
            .collect()
    }

    /// Whether anyone has sat down since that was last asked.
    pub(crate) fn has_sat(&self) -> bool {
        !self.sat.is_empty()
    }

    /// Something in `channel` has changed that the devices in it are told
    /// of: whose a plot is, or who is in their car.
    pub(crate) fn stir(&mut self, channel: u16) {
        if !self.stirred.contains(&channel) {
            self.stirred.push(channel);
        }
    }

    /// Whether anything has changed in any channel since they were last
    /// taken.
    pub(crate) fn is_stirred(&self) -> bool {
        !self.stirred.is_empty()
    }

    /// The channels something has changed in since this was last asked,
    /// each once.
    pub(crate) fn take_stirred(&mut self) -> Vec<u16> {
        std::mem::take(&mut self.stirred)
    }

    /// Everyone of `channel` and their body: each one's plot, and whether
    /// they are away from the town.
    pub(crate) fn held_in(&self, channel: u16) -> Vec<(u8, Entity, bool)> {
        let Some(seats) = self.channels.get(usize::from(channel)) else {
            return Vec::new();
        };
        let Some(estate) = &seats.estate else {
            return Vec::new();
        };
        estate
            .held()
            .filter_map(|(plot, who, away)| Some((plot as u8, *seats.bodies.get(&who)?, away)))
            .collect()
    }

    /// Who is in the town in `channel`, plot by plot: each plot and the
    /// body of whoever has it and is there now.
    pub(crate) fn seated(&self, channel: u16) -> Vec<(u8, Entity)> {
        self.held_in(channel)
            .into_iter()
            .filter(|&(.., away)| !away)
            .map(|(plot, body, _)| (plot, body))
            .collect()
    }

    /// Whose every plot of `channel` is, as its devices are told
    /// (`land::Deed`), given the name and the look of the body of someone
    /// of the channel.
    pub(crate) fn deeds(&self, channel: u16, known: impl Fn(Entity) -> Option<(String, String)>) -> Vec<Deed> {
        let Some(seats) = self.channels.get(usize::from(channel)) else {
            return Vec::new();
        };
        let Some(estate) = &seats.estate else {
            return Vec::new();
        };
        estate.deeds(|who| known(*seats.bodies.get(&who)?))
    }

    /// The land that fills each plot of `channel` that nobody of it has,
    /// or that is kept for someone who went a moment ago.
    pub(crate) fn lots_in(&self, channel: u16) -> Vec<&Lot> {
        let estate = self.channels.get(usize::from(channel)).and_then(|channel| channel.estate.as_ref());
        estate
            .into_iter()
            .flat_map(|estate| (0..roundtown_net::land::PLOTS).filter_map(|plot| estate.lot(plot)))
            .collect()
    }

    /// The plots that have been filled with something else since this was
    /// last asked, each its channel and the land that fills it now.
    pub(crate) fn take_news(&mut self) -> Vec<(u16, Lot)> {
        let mut news = Vec::new();
        for (number, channel) in self.channels.iter_mut().enumerate() {
            let Some(estate) = &mut channel.estate else {
                continue;
            };
            for plot in estate.take_news() {
                if let Some(lot) = estate.lot(plot) {
                    news.push((number as u16, lot.clone()));
                }
            }
        }
        news
    }

    /// Whether any plot has been filled with something else since that was
    /// last asked.
    pub(crate) fn has_news(&self) -> bool {
        self.channels.iter().any(|channel| channel.estate.as_ref().is_some_and(Estate::has_news))
    }

    /// How many are in each channel, those away in a game among them.
    pub(crate) fn counts(&self) -> Vec<usize> {
        self.channels.iter().map(Channel::count).collect()
    }
}

#[cfg(test)]
mod tests {
    use roundtown_net::At;
    use roundtown_net::bag::{LAMP, Standing};
    use roundtown_net::land::{GRACE, Holder, PLOTS};

    use super::*;

    fn player(n: u32) -> Entity {
        Entity::from_raw_u32(n + 1).unwrap()
    }

    /// Sits the `n`th player down, their account numbered as they are.
    fn sit(towns: &mut Towns, n: u32, rather: Option<u16>) -> (u16, u8) {
        towns.sit(player(n), u64::from(n) + 1, rather)
    }

    /// The land of the account `who`, a street lamp on it.
    fn land(who: u64) -> Lot {
        Lot {
            who,
            name: format!("Person {who}"),
            look: "girl".into(),
            stands: vec![Standing {
                id: 1,
                kind: LAMP.into(),
                at: At::Ground {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    yaw: 0.0,
                },
            }],
            paints: Vec::new(),
        }
    }

    fn known(body: Entity) -> Option<(String, String)> {
        Some((format!("{body:?}"), "boy".into()))
    }

    #[test]
    fn the_fullest_channel_with_room_first() {
        let mut towns = Towns::default();
        for n in 0..TOWN_SEATS as u32 {
            assert_eq!(sit(&mut towns, n, None), (0, n as u8));
        }
        // Full: a second channel.
        assert_eq!(sit(&mut towns, 100, None).0, 1);
        // A place comes free in the first, its holder gone to the second:
        // the next newcomer goes to the first, the fuller of the two.
        assert_eq!(towns.transfer(player(3), 1, None), Some(1));
        assert_eq!(sit(&mut towns, 101, None), (0, 3));
        // To the channel asked for, if it has room.
        assert_eq!(sit(&mut towns, 102, Some(1)).0, 1);
        assert_eq!(sit(&mut towns, 103, Some(0)).0, 1);
        assert_eq!(towns.counts(), vec![TOWN_SEATS, 4]);
        assert_eq!((towns.room_in(0), towns.room_in(1), towns.room_in(7)), (0, TOWN_SEATS - 4, TOWN_SEATS));
        assert_eq!((towns.roomy(2, 0), towns.roomy(2, 1), towns.roomy(TOWN_SEATS - 3, 0)), (Some(1), None, None));
    }

    #[test]
    fn a_plot_is_its_holder_s_into_a_game_and_back_and_for_a_while_after_they_are_gone() {
        let mut towns = Towns::default();
        assert_eq!(sit(&mut towns, 0, None), (0, 0));
        assert_eq!(sit(&mut towns, 1, None), (0, 1));
        towns.take_stirred();
        // Into a game: still of the channel, with their plot, and nobody
        // who is in the town.
        towns.away(player(1));
        assert_eq!(towns.take_stirred(), [0]);
        assert_eq!(towns.seated(0), [(0, player(0))]);
        assert_eq!(towns.held_in(0), [(0, player(0), false), (1, player(1), true)]);
        assert_eq!((towns.counts(), towns.channel_of(player(1))), (vec![2], Some(0)));
        assert_eq!(towns.deeds(0, known)[1].holder, Holder::Away);
        // Back: the plot they had, whatever channel is asked for.
        assert_eq!(sit(&mut towns, 1, Some(5)), (0, 1));
        assert_eq!(towns.seated(0), [(0, player(0)), (1, player(1))]);
        // Their body gone: kept as they left it a while, and theirs again
        // if they are back within it, in a body made anew.
        towns.tick(100.0);
        towns.leave(player(1), Some(land(2)));
        assert_eq!((towns.counts(), towns.channel_of(player(1))), (vec![2], None));
        let deeds = towns.deeds(0, known);
        assert_eq!((deeds[1].who, deeds[1].holder, deeds[1].name.as_str()), (2, Holder::Away, "Person 2"));
        // The one lot of the channel: no made-up land fills the rest
        // (`premade::FILLS`).
        assert_eq!(towns.lots_in(0).len(), 1);
        assert_eq!(towns.sit(player(9), 2, None), (0, 1));
        assert_eq!(towns.deeds(0, known)[1].holder, Holder::Here);
        // Gone for longer: their land fills the plot, someone of the
        // computer's on it, and the channel has room for one more.
        towns.leave(player(9), Some(land(2)));
        towns.take_stirred();
        towns.tick(101.0 + GRACE);
        assert_eq!(towns.take_stirred(), [0]);
        let deeds = towns.deeds(0, known);
        assert_eq!((deeds[1].who, deeds[1].holder, towns.counts()), (2, Holder::Minded, vec![1]));
        // The last of them gone and the while over: the channel is shut,
        // and whoever comes next has it worked out afresh.
        towns.leave(player(0), None);
        towns.tick(102.0 + 2.0 * GRACE);
        assert_eq!(towns.counts(), vec![0]);
        assert!(towns.deeds(0, known).is_empty() && towns.lots_in(0).is_empty());
        assert_eq!(sit(&mut towns, 4, None), (0, 0));
    }

    #[test]
    fn a_channel_opens_filled_with_the_land_there_is_and_news_of_a_plot_is_told_once() {
        let mut towns = Towns::default();
        towns.record(land(40));
        towns.record(land(41));
        // Land with nothing on it is no longer land that fills anything.
        towns.record(Lot { stands: Vec::new(), ..land(41) });
        assert_eq!(sit(&mut towns, 0, None), (0, 0));
        let deeds = towns.deeds(0, known);
        // Theirs, and the land, someone walking it; every other plot bare,
        // nobody's, no made-up land filling it (`premade::FILLS`).
        assert_eq!(deeds.len(), 2);
        assert_eq!((deeds[0].holder, deeds[1].who, deeds[1].holder), (Holder::Here, 40, Holder::Minded));
        assert_eq!(towns.lots_in(0).len(), 1);
        assert!(!towns.has_news());
        // Someone comes: the first bare plot. The land of whoever is not
        // here stays where it is, and nobody is told of any plot.
        assert_eq!(sit(&mut towns, 1, None), (0, 2));
        assert!(!towns.has_news() && towns.take_news().is_empty());
        assert_eq!(towns.deeds(0, known)[1].who, 40);
        // With someone standing on the next plot, the one after it.
        towns.stood_on(HashSet::from([(0, 3)]));
        assert_eq!(sit(&mut towns, 2, None), (0, 4));
        // The owner of that land comes: theirs is the plot it fills.
        assert_eq!(towns.sit(player(7), 40, None), (0, 1));
        // An account that is no more fills nothing: the plot it filled is
        // bare, which the channel is told as it is told whose each plot is,
        // and nothing is told of what stands there, nothing standing.
        towns.leave(player(7), Some(land(40)));
        towns.take_stirred();
        towns.forget(40);
        assert!(towns.deeds(0, known).iter().all(|deed| deed.who != 40 && deed.plot != 1));
        assert_eq!(towns.take_stirred(), [0]);
        assert!(towns.has_news());
        assert!(towns.take_news().is_empty());
        assert!(!towns.has_news());
    }

    #[test]
    fn the_server_that_runs_throws_where_everything_is_as_a_channel_opens() {
        // Opened many times over, whoever opens a channel has been on most
        // of the town's plots, and is on the one they are told.
        let mut plots = HashSet::new();
        for _ in 0..200 {
            let mut towns = Towns {
                thrown: true,
                ..default()
            };
            towns.record(land(40));
            let (channel, plot) = sit(&mut towns, 0, None);
            let deeds = towns.deeds(0, known);
            // Theirs and the land; the rest bare (`premade::FILLS`).
            assert_eq!((channel, deeds.len()), (0, 2));
            let theirs = deeds.iter().find(|deed| deed.plot == plot).map(|deed| (deed.who, deed.holder));
            assert_eq!(theirs, Some((1, Holder::Here)));
            // Come again, and back from a game, it is theirs still.
            assert_eq!(sit(&mut towns, 0, None), (0, plot));
            // Whoever comes after has another, and the land of someone
            // who is not here is where it was.
            let land_on = |deeds: &[Deed]| deeds.iter().find(|deed| deed.who == 40).map(|deed| deed.plot);
            let (_, other) = sit(&mut towns, 1, None);
            assert_ne!(other, plot);
            assert_eq!(land_on(&towns.deeds(0, known)), land_on(&deeds));
            assert!(land_on(&deeds).is_some());
            plots.insert(plot);
        }
        assert!(plots.len() > PLOTS / 2, "{} plots", plots.len());
    }

    #[test]
    fn sitting_again_keeps_the_plot() {
        let mut towns = Towns::default();
        sit(&mut towns, 0, None);
        sit(&mut towns, 0, None);
        assert_eq!(towns.counts(), vec![1]);
    }

    #[test]
    fn whoever_sits_down_is_said_to_have_once_and_not_if_they_have_gone_again() {
        let mut towns = Towns::default();
        assert!(!towns.has_sat());
        let (channel, _) = sit(&mut towns, 1, None);
        sit(&mut towns, 2, None);
        assert!(towns.has_sat());
        // Said again of someone seated, it is said once.
        towns.announce(channel, player(1));
        assert_eq!(towns.take_sat(), vec![(channel, player(1)), (channel, player(2))]);
        assert!(!towns.has_sat() && towns.take_sat().is_empty());
        // Sat down and gone again before anyone was told: nobody is.
        sit(&mut towns, 3, None);
        towns.leave(player(3), None);
        assert!(towns.take_sat().is_empty());
        // Said of someone who sits there already: told again.
        towns.announce(channel, player(2));
        assert_eq!(towns.take_sat(), vec![(channel, player(2))]);
    }

    #[test]
    fn a_channel_is_stirred_as_anyone_sits_in_it_or_goes() {
        let mut towns = Towns::default();
        assert!(!towns.is_stirred());
        assert!(towns.take_stirred().is_empty());
        assert_eq!(sit(&mut towns, 0, None), (0, 0));
        assert_eq!(sit(&mut towns, 1, None), (0, 1));
        // Once, however much of it changed, and not again until it does.
        assert!(towns.is_stirred());
        assert_eq!(towns.take_stirred(), [0]);
        assert!(!towns.is_stirred());
        assert!(towns.take_stirred().is_empty());
        assert_eq!(towns.seated(0), [(0, player(0)), (1, player(1))]);
        towns.leave(player(0), None);
        assert_eq!(towns.take_stirred(), [0]);
        assert_eq!(towns.seated(0), [(1, player(1))]);
        // Going from no plot stirs nothing; nor is there anyone in a
        // channel there is not.
        towns.leave(player(7), None);
        towns.away(player(7));
        assert!(towns.take_stirred().is_empty());
        assert!(towns.seated(9).is_empty());
        // Someone into their car, say: stirred by name.
        towns.stir(0);
        towns.stir(0);
        assert_eq!(towns.take_stirred(), [0]);
        // A full channel and a newcomer: the second alone is stirred.
        for n in 2..=TOWN_SEATS as u32 {
            sit(&mut towns, n, None);
        }
        towns.take_stirred();
        assert_eq!(sit(&mut towns, 100, None).0, 1);
        assert_eq!(towns.take_stirred(), [1]);
    }

    #[test]
    fn someone_moved_to_another_channel_leaves_nothing_behind_and_stays_away_if_they_were() {
        let mut towns = Towns::default();
        sit(&mut towns, 0, None);
        sit(&mut towns, 1, None);
        sit(&mut towns, 2, Some(1));
        assert_eq!(towns.counts(), vec![2, 1]);
        // Nowhere to move within one's own channel, nor for someone with no
        // plot.
        assert_eq!(towns.transfer(player(1), 0, None), None);
        assert_eq!(towns.transfer(player(9), 1, None), None);
        // In a game, moved: of the other channel now, and still away.
        towns.away(player(1));
        assert_eq!(towns.transfer(player(1), 1, Some(land(2))), Some(1));
        assert_eq!((towns.counts(), towns.channel_of(player(1))), (vec![1, 2], Some(1)));
        assert_eq!(towns.held_in(1), [(0, player(2), false), (1, player(1), true)]);
        // Nothing of theirs is on the plot they left, their land being in
        // no two channels: bare, nobody's (`premade::FILLS`).
        let deeds = towns.deeds(0, known);
        assert!(deeds.iter().all(|deed| deed.plot != 1));
        // The last one out of a channel shuts it at once.
        assert_eq!(towns.transfer(player(0), 1, None), Some(2));
        assert_eq!(towns.counts(), vec![0, 3]);
    }

    #[test]
    fn whoever_comes_goes_to_the_channel_their_land_fills_a_plot_of_and_nobody_s_land_is_in_two() {
        let shown = |towns: &Towns, channel: u16, who: u64| towns.deeds(channel, known).iter().position(|deed| deed.who == who);
        let mut towns = Towns::default();
        towns.record(land(40));
        towns.record(land(41));
        // The first channel opens with both, someone walking each.
        assert_eq!(sit(&mut towns, 0, None), (0, 0));
        assert_eq!((shown(&towns, 0, 40), shown(&towns, 0, 41)), (Some(1), Some(2)));
        // Another channel opens, and has more people in it: neither is
        // there too.
        assert_eq!(sit(&mut towns, 1, Some(1)), (1, 0));
        assert_eq!(sit(&mut towns, 2, Some(1)), (1, 1));
        assert_eq!((shown(&towns, 1, 40), shown(&towns, 1, 41)), (None, None));
        assert_eq!(towns.deeds(1, known).len(), 2);
        towns.take_news();
        // The owner of one comes: to the channel it is in, fuller or not,
        // and to the plot it fills.
        assert_eq!(towns.sit(player(7), 40, None), (0, 1));
        assert_eq!(towns.deeds(0, known)[1].holder, Holder::Here);
        assert!(!towns.has_news());
        // The owner of the other comes to where their group is: nothing of
        // theirs is left in the first, which is told what is there now.
        assert_eq!(towns.sit(player(8), 41, Some(1)), (1, 2));
        // Bare there now (`premade::FILLS`): nothing stands on it to be told
        // of, and whose each plot is says nobody's.
        assert_eq!(shown(&towns, 0, 41), None);
        assert!(towns.deeds(0, known).iter().all(|deed| deed.plot != 2));
        assert!(towns.has_news() && towns.take_news().is_empty());
        // Gone, and the while over: their land fills that plot of the
        // channel they were in, and of no other.
        towns.leave(player(8), Some(land(41)));
        towns.tick(1.0 + GRACE);
        assert_eq!((shown(&towns, 1, 41), shown(&towns, 0, 41)), (Some(2), None));
        assert_eq!(towns.deeds(1, known)[2].holder, Holder::Minded);
        // Nor of one that opens after: not theirs, nor that of someone who
        // has a plot in another.
        assert_eq!(sit(&mut towns, 3, Some(2)), (2, 0));
        assert_eq!((shown(&towns, 2, 41), shown(&towns, 2, 40)), (None, None));
        // Back, with no group to go to: to the channel it is in, the same
        // plot.
        assert_eq!(towns.sit(player(9), 41, None), (1, 2));
        // One whose connection went a moment ago and who comes back into
        // another channel, their group's: no plot is kept for them in the
        // one they were in.
        towns.leave(player(7), Some(land(40)));
        assert_eq!(towns.counts(), vec![2, 3, 1]);
        assert_eq!(towns.sit(player(10), 40, Some(2)), (2, 1));
        assert_eq!((towns.counts(), shown(&towns, 0, 40)), (vec![1, 3, 2], None));
    }
}
