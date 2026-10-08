//! Who is in the town: you, and enough others that it never feels empty.
//!
//! Every body holds a player — the same body, the same kind of record —
//! whether a person or the computer is playing it, and [`Member::ai`] is the
//! only thing that says which.
//!
//! Offline nobody else ever joins: you take the first seat, and the other
//! seven go to AIs ([`crate::ai`]). Online (`crate::net`), the server sends
//! whoever else is in your channel of the town, and this device keeps the
//! town at [`TOWN_BODIES`] bodies with AIs of its own ([`fill_the_town`]):
//! one leaves for each real player who arrives, and one comes back for each
//! who goes (Hajun's choice, 2026-10-08). Each device runs its own, so two
//! players see them in different places. A game of House Builder is a room
//! of its own, seated the same way, one AI at a time ([`seat_ai`]), and its
//! players are spawned by the same [`spawn_player`].
//!
//! Two rules keep the server down to being where things come from:
//!
//! * A [`Member`] is flat and primitive-only: the record the server sends. It
//!   names a [look](Member::look), never an asset path, the same way a town
//!   save names a kind.
//! * Only your own legs are driven by your stick. Everyone else's walk is read
//!   off how their body moves ([`walk_from_motion`]), because a position is all
//!   the network carries for them: no speeds, no animation state.

use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::TOWN_BODIES;

use crate::ai;
use crate::island::{self, Venue};
use crate::net::Online;
use crate::{
    Collider, ColliderShape, Intent, MOVE_SPEED, OrbitCamera, PLAYER_HEIGHT, PLAYER_RADIUS, Player,
    PlayerJump, WalkCycle, bind_walk_bones,
};

/// One player in a room, as the server describes them: an id, a name, a look,
/// and whether the computer is playing them. The very record the server
/// sends.
pub(crate) use roundtown_net::Member;

/// How many players the town holds offline, and a game of House Builder.
pub(crate) const SEATS: usize = 8;

/// The colours a player comes in, and the model for each. The first is yours,
/// and what a look this build does not know is drawn as.
const LOOKS: &[(&str, &str)] = &[
    ("blue", "limbperson.glb"),
    ("red", "AIpersonred.glb"),
    ("yellow", "AIpersonryellow.glb"),
    ("purple", "AIpersonrpurple.glb"),
];

/// What the computer's players are called. More than two rooms' worth, so
/// that the names in a room are not always the same, and the players who fill
/// a game of House Builder are not the ones you left in the town. Players
/// online go by the same names with a number after (the server's).
const AI_NAMES: &[&str] = &[
    "Mochi", "Dubu", "Hodu", "Bori", "Gamja", "Mandu", "Kimbap", "Yuja", "Maru", "Nabi", "Kong",
    "Podo", "Sagwa", "Gyul", "Hobak", "Bam", "Dalgi", "Haru", "Byeol", "Nuri",
];

/// The most seats the town's arrival ring has spots for that its AIs come in
/// at: the eight of offline, and the eight between them.
const TOWN_SPOTS: usize = 16;

/// Your look.
fn my_look() -> &'static str {
    LOOKS[0].0
}

/// You, as the first seat of a room.
pub(crate) fn me() -> Member {
    Member {
        id: 1,
        name: "You".into(),
        look: my_look().into(),
        ai: false,
    }
}

/// The town on this device: you, and the AIs it runs there.
#[derive(Resource)]
pub(crate) struct Lobby {
    /// Everyone in it. A player's seat decides where they stand on arriving
    /// on each island.
    pub seats: Vec<Member>,
    /// Which of them is playing on this device.
    pub me: u64,
}

/// What every player starts with.
pub(crate) const STARTING_BALANCE: u32 = 1000;

/// What a player has to spend. Everyone starts on `STARTING_BALANCE`, and for
/// now only House Builder changes it: its entry fee takes from it, and its
/// prizes add to it.
#[derive(Component, PartialEq)]
pub(crate) struct Balance(pub u32);

impl Default for Balance {
    fn default() -> Self {
        Self(STARTING_BALANCE)
    }
}

/// The seat a body sits in, which decides where it stands on each island: in
/// the town's room, or for a game's AIs, in the game's.
#[derive(Component)]
pub(crate) struct Seat(pub usize);

/// One of the town's own AIs, which this device runs to keep the town busy:
/// the member it plays, by id.
#[derive(Component)]
pub(crate) struct Townsfolk(u64);

/// A body whose walk is read off how far it goes each frame, rather than set
/// from how fast it is trying to go.
#[derive(Component)]
pub(crate) struct WalkFromMotion {
    /// Where it stood last frame.
    last: Vec3,
}

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(offline_room())
        .add_systems(Startup, spawn_players)
        .add_systems(
            Update,
            (
                walk_from_motion
                    .after(crate::move_bodies)
                    .before(crate::animate_walk),
                fill_the_town,
            ),
        );
}

/// A room nobody else joined: you in the first seat, and the computer in every
/// other.
fn offline_room() -> Lobby {
    let mut seats = vec![me()];
    fill_with_ai(&mut seats, &[]);
    Lobby { seats, me: 1 }
}

/// Seats the computer in every seat of `seats` still empty, under names nobody
/// there or in `elsewhere` has.
fn fill_with_ai(seats: &mut Vec<Member>, elsewhere: &[Member]) {
    while seats.len() < SEATS {
        seat_ai(seats, elsewhere);
    }
}

/// Seats the computer in the next seat of `seats`, under a name nobody there
/// or in `elsewhere` has. The room waiting for a game of House Builder fills
/// this way, one at a time, with nobody from the town.
pub(crate) fn seat_ai(seats: &mut Vec<Member>, elsewhere: &[Member]) {
    let seat = seats.len();
    let member = ai_member(seat, seat as u64 + 1, seats, elsewhere);
    seats.push(member);
}

/// The computer, in `seat`, as `id`, under a name nobody in `seats` or
/// `elsewhere` has.
fn ai_member(seat: usize, id: u64, seats: &[Member], elsewhere: &[Member]) -> Member {
    let names: Vec<&str> = AI_NAMES
        .iter()
        .copied()
        .filter(|name| {
            seats
                .iter()
                .chain(elsewhere)
                .all(|member| member.name != *name)
        })
        .collect();
    let ai_looks = &LOOKS[1..];
    Member {
        id,
        name: fastrand::choice(names).unwrap_or("Bot").into(),
        // Seat 0 is always a person's, so the first AI is the first look.
        look: ai_looks[seat.saturating_sub(1) % ai_looks.len()].0.into(),
        ai: true,
    }
}

/// Everyone in the room, standing round the fountain in the middle of the town.
fn spawn_players(
    mut commands: Commands,
    assets: Res<AssetServer>,
    lobby: Res<Lobby>,
    mut orbit: ResMut<OrbitCamera>,
) {
    for (seat, member) in lobby.seats.iter().enumerate() {
        let mine = member.id == lobby.me;
        let body = spawn_player(&mut commands, &assets, member, seat, Venue::Town, mine);
        if mine {
            *orbit = OrbitCamera::behind(&island::arrival(seat));
        } else {
            commands.entity(body).insert(Townsfolk(member.id));
        }
    }
}

/// Keeps the town at [`TOWN_BODIES`] bodies: you, whoever else is in your
/// channel of it online, and AIs of this device's own for the rest. Those
/// furthest from you are the first to go, so that one is seldom seen
/// vanishing. While you are away from the town online, who is there cannot be
/// seen, and it is left as it was.
fn fill_the_town(
    online: Res<Online>,
    me: Query<(&Transform, &Venue), With<Player>>,
    others: Query<&Venue, (With<Member>, With<Remote>)>,
    folk: Query<(Entity, &Townsfolk, &Transform, &Seat)>,
    mut lobby: ResMut<Lobby>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let Ok((you, &here)) = me.single() else {
        return;
    };
    let people = if online.is() {
        if here != Venue::Town {
            return;
        }
        1 + others.iter().filter(|&&venue| venue == Venue::Town).count()
    } else {
        1
    };
    let wanted = TOWN_BODIES.saturating_sub(people);
    let have = folk.iter().count();
    if have > wanted {
        let mut leaving: Vec<(f32, Entity, u64)> = folk
            .iter()
            .map(|(body, folk, at, _)| (at.translation.distance(you.translation), body, folk.0))
            .collect();
        leaving.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, body, id) in leaving.into_iter().take(have - wanted) {
            commands.entity(body).despawn();
            lobby.seats.retain(|member| member.id != id);
        }
    } else if have < wanted {
        let mut taken: Vec<usize> = folk.iter().map(|(.., seat)| seat.0).collect();
        for _ in have..wanted {
            let Some(seat) = (1..TOWN_SPOTS).find(|seat| !taken.contains(seat)) else {
                break;
            };
            taken.push(seat);
            let id = lobby.seats.iter().map(|member| member.id).max().unwrap_or(0) + 1;
            let member = ai_member(seat, id, &lobby.seats, &[]);
            lobby.seats.push(member.clone());
            let body = spawn_player(&mut commands, &assets, &member, seat, Venue::Town, false);
            commands.entity(body).insert(Townsfolk(id));
        }
    }
}

/// What every body is, whoever plays it: its name, its look, where it stands
/// and its walk.
fn body(assets: &AssetServer, member: &Member, at: Transform) -> impl Bundle {
    (
        Name::new(member.name.clone()),
        WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(model(&member.look)))),
        at,
        WalkCycle::default(),
    )
}

/// A body for `member`, in `seat`, standing where that seat arrives on
/// `venue`. The one that is `mine` takes this device's stick and the camera;
/// every other walks the way it is seen to move, and the computer's has a mind
/// of its own.
pub(crate) fn spawn_player(
    commands: &mut Commands,
    assets: &AssetServer,
    member: &Member,
    seat: usize,
    venue: Venue,
    mine: bool,
) -> Entity {
    let spawn = island::arrival(seat);
    let mut avatar = commands.spawn((
        body(assets, member, spawn),
        venue,
        Seat(seat),
        Balance::default(),
        PlayerJump::default(),
        Collider {
            shape: ColliderShape::Cylinder {
                radius: PLAYER_RADIUS,
                height: PLAYER_HEIGHT,
            },
        },
    ));
    avatar.observe(bind_walk_bones);
    if mine {
        avatar.insert((Player, Intent::default()));
    } else {
        avatar.insert(WalkFromMotion {
            last: spawn.translation,
        });
    }
    if member.ai {
        avatar.insert(ai::brain(seat));
    }
    avatar.id()
}

/// Gives `entity`, someone the server says is in your room, a body like any
/// other, standing `at`: drawn where the server says they are (`net`), their
/// walk read off how they move, and nobody to bump into. Players walk through
/// each other online (Hajun's choice, 2026-10-08), so that nobody can stand in
/// a doorway or in front of the House Builder and block it, and lag never
/// shoves anyone.
pub(crate) fn dress(entity: &mut EntityCommands, assets: &AssetServer, member: &Member, at: Transform) {
    entity
        .insert((
            body(assets, member, at),
            Seat(0),
            WalkFromMotion {
                last: at.translation,
            },
        ))
        .observe(bind_walk_bones);
}

/// What has to change about a body for it to be somewhere else from one frame
/// to the next, rather than walk there.
pub(crate) type Whereabouts<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Venue,
        &'static mut PlayerJump,
        Option<&'static mut WalkFromMotion>,
        Option<&'static mut ai::Steering>,
        Has<Player>,
    ),
>;

/// Puts `body` on `venue`, where `seat` arrives there, facing in: whatever
/// jump it was in dropped, whatever walk the computer had it on forgotten, and
/// the leap not taken for a stride. If it is yours, the camera swings round
/// behind you.
pub(crate) fn send(
    bodies: &mut Whereabouts,
    orbit: &mut OrbitCamera,
    body: Entity,
    venue: Venue,
    seat: usize,
) {
    put(bodies, orbit, body, venue, island::arrival(seat));
}

/// Puts `body` on `venue`, standing `at`, as [`send`] does.
pub(crate) fn put(
    bodies: &mut Whereabouts,
    orbit: &mut OrbitCamera,
    body: Entity,
    venue: Venue,
    at: Transform,
) {
    let Ok((mut transform, mut on, mut jump, seen, steering, mine)) = bodies.get_mut(body) else {
        return;
    };
    transform.set_if_neq(at);
    on.set_if_neq(venue);
    *jump = PlayerJump::default();
    if let Some(mut seen) = seen {
        seen.last = at.translation;
    }
    if let Some(mut steering) = steering {
        steering.stop();
    }
    if mine {
        orbit.yaw = OrbitCamera::behind(&at).yaw;
    }
}

/// The model for a look. One this build has never heard of — a newer client's,
/// say — is drawn as the first look rather than left out: whoever wears it is
/// still in the room.
fn model(look: &str) -> &'static str {
    LOOKS
        .iter()
        .find(|(name, _)| *name == look)
        .map_or(LOOKS[0].1, |(_, model)| model)
}

/// Reads each body's walk off how far it went since last frame. Only ground
/// covered counts: rising and falling in a jump is not walking.
fn walk_from_motion(
    time: Res<Time>,
    mut bodies: Query<(&Transform, &mut WalkFromMotion, &mut WalkCycle)>,
) {
    let dt = time.delta_secs();
    for (transform, mut seen, mut walk) in &mut bodies {
        let moved = transform.translation.xz().distance(seen.last.xz());
        seen.last = transform.translation;
        walk.speed = if dt > 0.0 {
            (moved / dt / MOVE_SPEED).min(1.0)
        } else {
            0.0
        };
    }
}
