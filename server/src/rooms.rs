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
//! that the town feels busy rather than spread thin; coming back from home,
//! to the channel they left if it still has room for them.

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::{Minigame, TOWN_SEATS};

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Towns>()
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

/// The town's channels, and who sits in each seat of each: the seat decides
/// where its player arrives, and which plot of the town is theirs (the
/// game's `land`).
#[derive(Resource, Default)]
pub(crate) struct Towns {
    channels: Vec<[Option<Entity>; TOWN_SEATS]>,
    /// The channels something has changed in that the devices in them are
    /// told of (`players::tell_channels`, since 2026-11-21): someone has
    /// sat down or stood up, or got into their car or out of it. Each once,
    /// until they are told.
    stirred: Vec<u16>,
    /// Who has sat down in which channel since that was last asked
    /// ([`Towns::take_sat`]): each is told of what stands on the plots of
    /// the channel they have come into, and the channel of what stands on
    /// theirs (`bag::seat_news`, since 2026-11-21).
    sat: Vec<(u16, Entity)>,
}

impl Towns {
    /// Seats `player` in the town: in the channel `rather` if it has room,
    /// otherwise in the fullest that does, otherwise in a new one. Returns the
    /// channel and the seat.
    pub(crate) fn sit(&mut self, player: Entity, rather: Option<u16>) -> (u16, u8) {
        self.stand(player);
        let count = |seats: &[Option<Entity>; TOWN_SEATS]| seats.iter().flatten().count();
        let open = |channel: usize| self.channels.get(channel).is_some_and(|seats| count(seats) < TOWN_SEATS);
        let channel = rather
            .map(usize::from)
            .filter(|&channel| open(channel))
            .or_else(|| {
                (0..self.channels.len())
                    .filter(|&channel| open(channel))
                    .max_by_key(|&channel| (count(&self.channels[channel]), std::cmp::Reverse(channel)))
            })
            .unwrap_or_else(|| {
                self.channels.push([None; TOWN_SEATS]);
                self.channels.len() - 1
            });
        let seats = &mut self.channels[channel];
        let seat = seats
            .iter()
            .position(Option::is_none)
            .expect("only a channel with room is chosen");
        seats[seat] = Some(player);
        self.stir(channel as u16);
        self.announce(channel as u16, player);
        (channel as u16, seat as u8)
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
    /// once, those who have stood up again since left out.
    pub(crate) fn take_sat(&mut self) -> Vec<(u16, Entity)> {
        let sat = std::mem::take(&mut self.sat);
        sat.into_iter()
            .filter(|&(channel, player)| {
                self.channels.get(usize::from(channel)).is_some_and(|seats| seats.contains(&Some(player)))
            })
            .collect()
    }

    /// Whether anyone has sat down since that was last asked.
    pub(crate) fn has_sat(&self) -> bool {
        !self.sat.is_empty()
    }

    /// Frees whatever seat in the town `player` had.
    pub(crate) fn stand(&mut self, player: Entity) {
        let mut left = Vec::new();
        for (channel, seats) in self.channels.iter_mut().enumerate() {
            for seat in seats.iter_mut() {
                if *seat == Some(player) {
                    *seat = None;
                    left.push(channel as u16);
                }
            }
        }
        for channel in left {
            self.stir(channel);
        }
    }

    /// Something in `channel` has changed that the devices in it are told
    /// of: who sits where, or who is in their car.
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

    /// Who sits in `channel`, seat by seat: each taken seat and its player.
    pub(crate) fn seated(&self, channel: u16) -> Vec<(u8, Entity)> {
        self.channels
            .get(usize::from(channel))
            .into_iter()
            .flat_map(|seats| seats.iter().enumerate())
            .filter_map(|(seat, player)| Some((seat as u8, (*player)?)))
            .collect()
    }

    /// How many are in each channel.
    pub(crate) fn counts(&self) -> Vec<usize> {
        self.channels
            .iter()
            .map(|seats| seats.iter().flatten().count())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(n: u32) -> Entity {
        Entity::from_raw_u32(n + 1).unwrap()
    }

    #[test]
    fn the_fullest_channel_with_room_first() {
        let mut towns = Towns::default();
        for n in 0..TOWN_SEATS as u32 {
            assert_eq!(towns.sit(player(n), None), (0, n as u8));
        }
        // Full: a second channel.
        assert_eq!(towns.sit(player(100), None).0, 1);
        // A seat comes free in the first: the next newcomer goes there, the
        // fuller of the two.
        towns.stand(player(3));
        assert_eq!(towns.sit(player(101), None), (0, 3));
        // Back from home, to the channel left, if it has room.
        assert_eq!(towns.sit(player(102), Some(1)).0, 1);
        assert_eq!(towns.sit(player(103), Some(0)).0, 1);
        assert_eq!(towns.counts(), vec![TOWN_SEATS, 3]);
    }

    #[test]
    fn sitting_again_frees_the_old_seat() {
        let mut towns = Towns::default();
        towns.sit(player(0), None);
        towns.sit(player(0), None);
        assert_eq!(towns.counts(), vec![1]);
    }

    #[test]
    fn whoever_sits_down_is_said_to_have_once_and_not_if_they_have_stood_up_again() {
        let mut towns = Towns::default();
        assert!(!towns.has_sat());
        let (channel, _) = towns.sit(player(1), None);
        towns.sit(player(2), None);
        assert!(towns.has_sat());
        // Said again of someone seated, it is said once.
        towns.announce(channel, player(1));
        assert_eq!(towns.take_sat(), vec![(channel, player(1)), (channel, player(2))]);
        assert!(!towns.has_sat() && towns.take_sat().is_empty());
        // Sat down and gone again before anyone was told: nobody is.
        towns.sit(player(3), None);
        towns.stand(player(3));
        assert!(towns.take_sat().is_empty());
        // Said of someone who sits there already: told again.
        towns.announce(channel, player(2));
        assert_eq!(towns.take_sat(), vec![(channel, player(2))]);
    }

    #[test]
    fn a_channel_is_stirred_as_anyone_sits_in_it_or_stands_up() {
        let mut towns = Towns::default();
        assert!(!towns.is_stirred());
        assert!(towns.take_stirred().is_empty());
        assert_eq!(towns.sit(player(0), None), (0, 0));
        assert_eq!(towns.sit(player(1), None), (0, 1));
        // Once, however much of it changed, and not again until it does.
        assert!(towns.is_stirred());
        assert_eq!(towns.take_stirred(), [0]);
        assert!(!towns.is_stirred());
        assert!(towns.take_stirred().is_empty());
        assert_eq!(towns.seated(0), [(0, player(0)), (1, player(1))]);
        towns.stand(player(0));
        assert_eq!(towns.take_stirred(), [0]);
        assert_eq!(towns.seated(0), [(1, player(1))]);
        // Standing up from no seat stirs nothing; nor is there anyone in a
        // channel there is not.
        towns.stand(player(7));
        assert!(towns.take_stirred().is_empty());
        assert!(towns.seated(9).is_empty());
        // Someone into their car, say: stirred by name.
        towns.stir(0);
        towns.stir(0);
        assert_eq!(towns.take_stirred(), [0]);
        // A full channel and a newcomer: the second alone is stirred.
        for n in 2..=TOWN_SEATS as u32 {
            towns.sit(player(n), None);
        }
        towns.take_stirred();
        assert_eq!(towns.sit(player(100), None).0, 1);
        assert_eq!(towns.take_stirred(), [1]);
    }
}
