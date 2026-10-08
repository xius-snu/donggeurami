//! What 동그라미타운 and its server say to each other.
//!
//! The game (the root crate) and the server (`server/`) both build on this
//! crate, and both register it through [`protocol`], so they register the
//! same things in the same order. replicon hashes those registrations, and a
//! client whose hash differs from the server's is told it is out of date
//! instead of being let in.
//!
//! Three rules keep it cheap (`MULTIPLAYER.md`, "The one rule"):
//!
//! * A position is whole centimetres and a 16-bit turn ([`Spot`]): about ten
//!   bytes.
//! * Only what has changed is sent. replicon works off Bevy's change
//!   detection, the same idea as `set_if_neq` everywhere in the game, and a
//!   device sends its own position only while it changes.
//! * Who someone is, their name and their look ([`Member`]), goes once, when
//!   they come into your room, not with every move.
//!
//! What goes which way:
//!
//! * **Server to everyone in a room**, replicated: each player's [`Member`],
//!   [`Spot`] and [`Venue`]; and in a game of House Builder its [`Round`], its
//!   [`Seats`] and every piece put down, [`Built`].
//! * **Device to server**: where you are, [`Moved`], unreliable, since a lost
//!   one is replaced by the next; and everything you ask for, [`Ask`], in
//!   order.
//! * **Server to one device**: what concerns you alone, [`Tell`], in order.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::replication::registry::ctx::WriteCtx;
use bevy_replicon::shared::replication::registry::rule_fns::{
    RuleFns, default_deserialize, default_serialize,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// netcode's protocol id: which game a packet is for. A packet with another
/// one is dropped without a word, so this names the game and never changes;
/// versions are told apart by replicon's protocol hash instead, which a
/// client is told about.
pub const PROTOCOL_ID: u64 = u64::from_be_bytes(*b"DGRMTOWN");

/// Bumped whenever what a message means changes without its shape changing,
/// which the protocol hash would not notice on its own.
pub const PROTOCOL_VERSION: u32 = 1;

/// The UDP port the game server listens on. 443, where QUIC goes, because a
/// network that lets any UDP through lets that through: school and office
/// networks often stop the rest.
pub const GAME_PORT: u16 = 443;

/// How many times a second the server sends what has changed, and how many
/// times a second a device sends its own position while it moves.
pub const TICK_HZ: f64 = 15.0;

/// How many players a town channel holds (`MULTIPLAYER.md`, "How many to a
/// room": 16 to start).
pub const TOWN_SEATS: usize = 16;

/// How many players a game of House Builder holds, bots included.
pub const GAME_SEATS: usize = 8;

/// The fewest bodies the town shows: below this, each device fills it out
/// with AIs of its own (Hajun's choice, 2026-10-08), one leaving for each
/// real player who arrives.
pub const TOWN_BODIES: usize = 8;

/// The look every real player wears for now: what you wear offline. Bots
/// wear the others. Outfits are for later (`MULTIPLAYER.md`, phase 4).
pub const PLAYER_LOOK: &str = "blue";

/// Registers everything both sides send, in one order for both. Call it after
/// `RepliconPlugins` and before the renet client or server is made: its
/// channels are counted from what is registered here.
pub fn protocol(app: &mut App) {
    app.replicate::<Member>()
        .replicate::<Venue>()
        .replicate_with(RuleFns::new(default_serialize::<Spot>, deserialize_spot))
        .replicate::<Round>()
        .replicate::<Seats>()
        .replicate::<Built>()
        .add_client_message::<Moved>(Channel::Unreliable)
        .add_client_message::<Ask>(Channel::Ordered)
        .add_server_message::<Tell>(Channel::Ordered);
    app.world_mut()
        .resource_mut::<ProtocolHasher>()
        .add_custom(PROTOCOL_VERSION);
}

// ---------------------------------------------------------------- islands

/// Which island something is on. The game's `island::Venue` is this.
///
/// Every island stands at the origin, in the same place: what keeps them
/// apart is this, carried by everything that moves or collides.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Venue {
    /// `circlemap1.glb`: Round Town, round a fountain, where everyone is.
    Town,
    /// Your home, where nobody else goes for now.
    Home,
    /// `lobbymap.glb`: where the players of a game of House Builder wait for
    /// it to fill.
    Lobby,
    /// One builder's own copy of `homemap.glb` in a game of House Builder, by
    /// their seat in the game.
    Plot(u8),
}

/// How far from the middle everyone stands on arriving on an island, in
/// metres: well inside the grass of the home and the lobby, which reaches 20 m
/// out, and clear of the town's fountain, which is 2.5 m across the rim.
pub const ARRIVAL_RADIUS: f32 = 6.0;

/// Where the player in `seat` stands on arriving on any island, and which way
/// they face: on a ring round the middle — round the fountain, in the town —
/// facing in. The first eight seats are each an eighth of the turn on from
/// the one before, the next eight between them, and past sixteen there is a
/// ring further out. The server works out arrivals with this too, so that
/// everyone sees you arrive where you do.
pub fn arrival(seat: usize) -> (Vec3, Quat) {
    let eighth = TAU / 8.0;
    let ring = seat / 16;
    let within = seat % 16;
    let angle = (within % 8) as f32 * eighth
        + if within >= 8 { eighth * 0.5 } else { 0.0 }
        + ring as f32 * eighth * 0.25;
    let radius = ARRIVAL_RADIUS + ring as f32 * 2.0;
    let turn = Quat::from_rotation_y(angle);
    // Facing in: a turn of `angle` about the upright points -Z at the middle
    // from where it puts +Z.
    (turn * Vec3::new(0.0, 0.0, radius), turn)
}

// ---------------------------------------------------------------- players

/// One player in a room, as the server describes them: sent once, when they
/// come into your room. Flat and primitive-only: a look is a name, never an
/// asset path.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[component(immutable)]
pub struct Member {
    /// Unique and stable for as long as they are connected, so that whatever
    /// is said about them can say who it is about.
    pub id: u64,
    /// What everyone else sees them called. Plain ASCII: the game's font has
    /// no more.
    pub name: String,
    /// Which of the game's looks they wear.
    pub look: String,
    /// The computer is playing this seat, not a person.
    pub ai: bool,
}

/// Where a body stands and which way it faces: whole centimetres, and a turn
/// in 65,536ths of a circle. About eleven bytes once postcard has packed it.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub struct Spot {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: u16,
    /// The body has come to a stop here. Someone watching draws it standing
    /// here from then on, rather than carrying it on the way it was going
    /// while they wait for the next update, which is not coming.
    pub still: bool,
    /// On a device, the server tick of the update that brought it, so that
    /// updates can be laid out in the time they were sent rather than the
    /// time they happened to arrive. Never sent.
    #[serde(skip)]
    pub tick: u32,
}

impl Spot {
    /// Where a body at `translation`, turned by `rotation`, stands.
    pub fn of(translation: Vec3, rotation: Quat) -> Self {
        let forward = rotation * Vec3::NEG_Z;
        let yaw = (-forward.x).atan2(-forward.z);
        Self {
            x: centimetres(translation.x),
            y: centimetres(translation.y),
            z: centimetres(translation.z),
            yaw: ((yaw / TAU).rem_euclid(1.0) * 65_536.0).round() as u32 as u16,
            still: false,
            tick: 0,
        }
    }

    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32) / 100.0
    }

    /// The turn about the upright, in radians.
    pub fn yaw(&self) -> f32 {
        f32::from(self.yaw) / 65_536.0 * TAU
    }

    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw())
    }

    /// The same place, the same way and as still, whenever it was sent.
    pub fn same(&self, other: &Self) -> bool {
        (self.x, self.y, self.z, self.yaw, self.still)
            == (other.x, other.y, other.z, other.yaw, other.still)
    }
}

fn centimetres(metres: f32) -> i32 {
    (metres * 100.0).round() as i32
}

/// A [`Spot`] as it arrives, stamped with the tick of the update it came in.
fn deserialize_spot(ctx: &mut WriteCtx, message: &mut Bytes) -> Result<Spot> {
    let mut spot: Spot = default_deserialize(ctx, message)?;
    spot.tick = ctx.message_tick.get();
    Ok(spot)
}

// ------------------------------------------------------------ House Builder

/// House Builder's rules, which the server plays by online and the game by
/// itself offline: in one place, so that the two never differ.
pub mod rules {
    /// What every player starts with, in coins.
    pub const STARTING_BALANCE: u32 = 1000;
    /// What a game costs to join, in coins.
    pub const ENTRY: u32 = 100;
    /// How long the lobby waits for people before the computer starts taking
    /// the empty seats, in seconds: online only (Hajun's choice, 2026-10-08).
    /// Offline there is nobody to wait for.
    pub const PEOPLE_WAIT: f32 = 10.0;
    /// How long the computer takes to fill the seats left, in seconds: one at
    /// a time, evenly over it, the last as it runs out.
    pub const QUEUE_FILL: f32 = 5.0;
    /// How long a full room shows 8/8 before everyone leaves for their plot.
    pub const FULL_HOLD: f32 = 2.0;
    /// How long everyone has to build, in seconds.
    pub const BUILD_SECS: f32 = 300.0;
    /// How long everyone spends at each house rating it, in seconds.
    pub const VISIT_SECS: f32 = 15.0;
    /// The most stars a house can be given.
    pub const MOST_STARS: u8 = 5;
}

/// A game of House Builder as everyone in it sees it: how far it has got, how
/// long is left of that, and the theme.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Round {
    pub stage: Stage,
    /// Whole seconds left of this stage, rounded up: what the board shows.
    pub left: u16,
    /// Which of the game's themes, by its place in `builder::THEMES`.
    pub theme: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Stage {
    /// In the lobby, waiting for the room to fill.
    Queue,
    /// Full, and about to go.
    Full,
    /// Everyone on their own plot, building.
    Build,
    /// Everyone at the house on this plot, rating it.
    Visit(u8),
    /// Done: the house on `winner` had the most stars, and these are every
    /// house's, by plot.
    Over { winner: u8, stars: [u32; GAME_SEATS] },
}

/// Everyone in a game of House Builder, in seat order: a builder's plot is
/// their seat.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Seats(pub Vec<Member>);

/// A piece put down on a plot in a game of House Builder: a house, a door, a
/// window, a bed. Sent once it is down, so that everyone visiting sees it;
/// never while it is being dragged.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[component(immutable)]
pub struct Built {
    /// Whose plot it is on.
    pub plot: u8,
    /// What it is: `build::PIECES`' name for it, never an asset path.
    pub kind: String,
    pub at: At,
}

/// Where a piece is on its plot.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum At {
    /// Standing on the plot: where its origin is, and its turn about the
    /// upright, in radians.
    Ground { x: f32, y: f32, z: f32, yaw: f32 },
    /// In or on a wall of the plot's house: which of its walls, counted in
    /// order of where they stand in the house, how far along it and up it,
    /// and which face, 1 or -1, for what hangs on one.
    Wall {
        wall: u8,
        along: f32,
        up: f32,
        face: i8,
    },
}

// --------------------------------------------------------------- messages

/// Where you are now: sent while it changes, 15 times a second, and nothing
/// while you stand still. Unreliable: a lost one is replaced by the next.
#[derive(Message, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Moved {
    /// How many times the server has moved you, as of this position. One
    /// sent before the server last moved you is from where you were before,
    /// and is dropped rather than taken for a leap.
    pub arrival: u8,
    pub spot: Spot,
}

/// Something you ask of the server. In order, and never lost.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub enum Ask {
    /// The first thing a device says: where it already is, so that it is let
    /// in there rather than moved.
    Hello { venue: Venue, spot: Spot },
    /// Take me home, or back to the town.
    Travel(Venue),
    /// Pay, and join a game of House Builder: from the House Builder's offer,
    /// or after one game for another.
    Play,
    /// Back to the town after a game.
    Leave,
    /// Give the house on `plot` this many stars.
    Rate { plot: u8, stars: u8 },
    /// End the wait early: for trying the game out, and only allowed by a
    /// server that says so.
    Skip,
    /// This piece is down on my plot, here. `id` is mine to choose, and says
    /// the same piece again if it is picked up and put down again.
    Put { id: u32, kind: String, at: At },
    /// This piece of mine is no longer down: picked back up, or thrown away.
    Take { id: u32 },
}

/// What the server tells you alone. In order, and never lost.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub enum Tell {
    /// You are in, and this is who you are to everyone else.
    Welcome { me: Member },
    /// The server has put you on `venue`, where `seat` arrives there, or, with
    /// a `spot`, where you already stood. `arrival` is what to send with your
    /// moves from now on.
    Arrive {
        venue: Venue,
        seat: u8,
        spot: Option<Spot>,
        arrival: u8,
    },
    /// The last move was not one a body could make: back to here.
    PutBack { arrival: u8, spot: Spot },
    /// What you have, now that it has changed.
    Balance(u32),
    /// You are in a game of House Builder, in this seat: your plot.
    Joined { seat: u8 },
    /// What you asked for cannot be done.
    Refused(Refusal),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// You have not got the coins.
    Coins,
    /// You are in the middle of something else.
    Busy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spot_keeps_where_and_which_way() {
        for (at, yaw) in [
            (Vec3::new(5.55, 0.0, -3.21), 0.0),
            (Vec3::new(-580.0, 12.34, 580.0), 1.0),
            (Vec3::new(0.004, -0.7, 0.0), -2.5),
            (Vec3::ZERO, std::f32::consts::PI),
        ] {
            let rotation = Quat::from_rotation_y(yaw);
            let spot = Spot::of(at, rotation);
            assert!(spot.translation().distance(at) < 0.01, "{at} came back {}", spot.translation());
            let back = spot.rotation() * Vec3::NEG_Z;
            let was = rotation * Vec3::NEG_Z;
            assert!(back.distance(was) < 1e-3, "{yaw}: {back} for {was}");
        }
    }

    #[test]
    fn the_first_eight_arrive_where_they_did_offline() {
        for seat in 0..8 {
            let (at, turn) = arrival(seat);
            let was = Quat::from_rotation_y(seat as f32 * TAU / 8.0)
                * Vec3::new(0.0, 0.0, ARRIVAL_RADIUS);
            assert!(at.distance(was) < 1e-4, "seat {seat}: {at} for {was}");
            let facing = turn * Vec3::NEG_Z;
            assert!(facing.distance(-was.normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
    }

    #[test]
    fn a_full_town_arrives_apart() {
        let spots: Vec<Vec3> = (0..TOWN_SEATS * 2).map(|seat| arrival(seat).0).collect();
        for (i, a) in spots.iter().enumerate() {
            for b in &spots[i + 1..] {
                assert!(a.distance(*b) > 2.0, "{a} and {b}");
            }
        }
    }

    #[test]
    fn a_spot_is_about_ten_bytes() {
        let spot = Spot::of(Vec3::new(-234.56, 1.2, 345.67), Quat::from_rotation_y(2.0));
        let bytes = postcard::to_allocvec(&spot).unwrap();
        assert!(bytes.len() <= 12, "{} bytes", bytes.len());
    }
}
