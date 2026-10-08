//! Rooms: who sees whom.
//!
//! A room is a place where everyone in it sees everyone else: one channel of
//! the town, [`TOWN_SEATS`] to a channel; your home, with only you in it for
//! now; or a game of House Builder, its lobby and its plots together. A
//! device is sent only what is in its own room. That is replicon's visibility
//! filters, keyed by room: [`InRoom`] on whatever is sent, a player or a game,
//! and [`ClientRoom`] on the device it may be sent to.
//!
//! A newcomer to the town goes to the fullest channel that still has room, so
//! that the town feels busy rather than spread thin; coming back from home,
//! to the channel they left if it still has room for them.

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::TOWN_SEATS;

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
/// where its player arrives.
#[derive(Resource, Default)]
pub(crate) struct Towns {
    channels: Vec<[Option<Entity>; TOWN_SEATS]>,
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
        (channel as u16, seat as u8)
    }

    /// Frees whatever seat in the town `player` had.
    pub(crate) fn stand(&mut self, player: Entity) {
        for seats in &mut self.channels {
            for seat in seats.iter_mut() {
                if *seat == Some(player) {
                    *seat = None;
                }
            }
        }
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
}
