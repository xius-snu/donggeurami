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
#[cfg(test)]
use roundtown_net::{BOY_LOOKS, BOY_NAMES, GIRL_LOOKS, GIRL_NAMES, PEOPLE_LOOKS};

use crate::ai;
use crate::car::Driven;
use crate::island::{self, Venue};
use crate::net::Online;
use crate::signup::Profile;
use crate::talk::Speaker;
use crate::{
    Collider, ColliderShape, Intent, MOVE_SPEED, OrbitCamera, PLAYER_HEIGHT, PLAYER_RADIUS, Player,
    PlayerJump, Riding, Shove, WalkCycle, bind_walk_bones,
};

/// One player in a room, as the server describes them: an id, a name, a look,
/// and whether the computer is playing them. The very record the server
/// sends.
pub(crate) use roundtown_net::Member;

/// How many players the town holds offline, and a game of House Builder.
pub(crate) const SEATS: usize = 8;

/// The model the boy is drawn with, and the computer's players, and any
/// look this build has not heard of: Hajun's
/// `playercharacter.glb` (2026-11-01: "try using playercharacter in asset
/// folder instead of both boy/girl model for the main player and all the
/// AI's"), a Rigify metarig, scaled to 0.616 in the file, under a
/// skin-coloured body with shoes, a blue top and dark trousers, skinned,
/// and a mouth and two eyes hung rigid on the head bone (`spine.006`):
/// about 4,150 triangles in seven materials, hair among them, 1.64 m to the top of its round
/// head, 0.53 m across, resting with its arms a little out (an A-pose),
/// which the walk hangs at its sides. That is its fourth version of the
/// day (23:17, with hair; 22:53 dressed): the first, a bare 1.70 m figure
/// of 1,468 triangles, and the second, the same smooth-shaded. Until then
/// the boy was `limbperson.glb`, the girl `limbpersongirl.glb` (the same
/// in a pink top), and the computer's players `AIpersonred.glb`,
/// `AIpersonryellow.glb` and `AIpersonrpurple.glb`, all still in `assets/`
/// and out of the APK.
pub(crate) const MODEL: &str = "playercharacter.glb";

/// The model the girl is drawn with: Hajun's `playercharactergirl.glb`
/// (the night of 2026-11-01: "now there is playercharacter girl as well.
/// use those as the player choice in the beginning of the game character
/// selection"), the boy's rig, body and rest with hair to the waist and a
/// skirt, 3,440 triangles in eight materials. The sign-up's pictures are
/// rendered from both (`blender/render_person.py`).
pub(crate) const GIRL_MODEL: &str = "playercharactergirl.glb";

/// The looks a player comes in, and the model for each: first the two a
/// person chooses between on signing up (`signup`, [`PEOPLE_LOOKS`], in
/// their order), the boy first, which is what a look this build does not
/// know is drawn as, among them `blue`, the look of every account from
/// before there was a choice; then Hajun's recolourings of the two, which
/// a person may choose as well and the computer's players wear along with
/// them ([`BOY_LOOKS`], [`GIRL_LOOKS`], `roundtown_net::ai_look`: a boy or
/// a girl by the toss of a coin, then any of that side; 2026-11-02, "random
/// girl/guy 50/50" and "use variations of the colors"), each a `.glb`
/// exported from the boy's or the girl's `.blend` with its clothes and hair
/// recoloured; and the three the
/// computer's players took turns at until that day, kept for the server's
/// bots of before and older apps, drawn as the boy. The boy is [`MODEL`]
/// and the girl [`GIRL_MODEL`] (since the night of 2026-11-01; for the
/// day's first hours every look was [`MODEL`]).
const LOOKS: &[(&str, &str)] = &[
    ("boy", MODEL),
    ("girl", GIRL_MODEL),
    ("boy_blackbrown", "playercharacterblackbrown.glb"),
    ("boy_whitejeans", "playercharacterwhitejeans.glb"),
    ("girl_whiteblue", "playercharactergirlwhiteblue.glb"),
    ("girl_yellowpink", "playercharactergirlyellowpink.glb"),
    ("red", MODEL),
    ("yellow", MODEL),
    ("purple", MODEL),
    // A zombie of Bridge Wars' sudden death: the boy, whose skin `arena`
    // turns green (`roundtown_net::bridge_wars::ZOMBIE_LOOK`).
    ("zombie", MODEL),
];

/// The model of every look, each once: what a body may be drawn with, for
/// the APK's test that each goes into the APK, and the walk's that each
/// has the limbs it swings.
pub(crate) fn models() -> Vec<&'static str> {
    let mut models: Vec<&str> = Vec::new();
    for (_, model) in LOOKS {
        if !models.contains(model) {
            models.push(model);
        }
    }
    models
}

/// The most seats the town's arrivals have spots for that its AIs come in
/// at: sixteen on the path round the pond and sixteen more on a ring 2 m
/// further out (`roundtown_net::arrival`; for 2026-11-07, eight to each of
/// the PvP map's four islands), for a town of [`TOWN_BODIES`] (16 until
/// 2026-11-01, when the town went from 8 bodies to 32).
const TOWN_SPOTS: usize = 32;

/// The look a body wears when none has been chosen: the boy's. The House
/// Builder's, and yours until you have signed up.
pub(crate) fn default_look() -> &'static str {
    LOOKS[0].0
}

/// You, as the first seat of a room: by the name and in the look you chose on
/// signing up, or, before that, "You" in the default look.
pub(crate) fn me(profile: &Profile) -> Member {
    Member {
        id: 1,
        name: if profile.name.is_empty() { "You".into() } else { profile.name.clone() },
        look: if profile.look.is_empty() { default_look().into() } else { profile.look.clone() },
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

impl Lobby {
    /// You, as this device has you: in the town's room, by the name and in
    /// the look you signed up with.
    pub(crate) fn mine(&self) -> Member {
        self.seats
            .iter()
            .find(|member| member.id == self.me)
            .cloned()
            .unwrap_or_else(|| me(&Profile::default()))
    }

    /// Takes on the name and the look you signed up with, for the rest of
    /// the room to know you by.
    pub(crate) fn sign_up(&mut self, profile: &Profile) {
        let id = self.me;
        if let Some(member) = self.seats.iter_mut().find(|member| member.id == id) {
            *member = Member { id, ..me(profile) };
        }
    }
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

/// One of the town's own AIs, which this device runs to keep the town busy,
/// and which you can talk to (`talk`): the member it plays, by id.
#[derive(Component)]
pub(crate) struct Townsfolk(u64);

/// A body whose walk is read off how far it goes each frame, rather than set
/// from how fast it is trying to go.
#[derive(Component)]
pub(crate) struct WalkFromMotion {
    /// Where it stood last frame: on the island, or, riding the car, on the
    /// car, in its own space.
    last: Vec3,
    /// The car it was riding then.
    on: Option<Entity>,
}

pub(crate) fn plugin(app: &mut App) {
    // Who you are, as you signed up (`signup`, which comes before this), if
    // you have.
    let profile = app.world().get_resource::<Profile>().cloned().unwrap_or_default();
    app.insert_resource(offline_room(&profile))
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
fn offline_room(profile: &Profile) -> Lobby {
    let mut seats = vec![me(profile)];
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

/// The computer, in `seat`, as `id`: a boy or a girl by the toss of a coin,
/// in any of that side's colours, under one of that side's names that
/// nobody in `seats` or `elsewhere` has (`roundtown_net::ai_look`,
/// `ai_names`; Hajun, 2026-11-02: "make the ai's also be random girl/guy
/// 50/50", "use variations of the colors", "use the more manly ai names
/// for the boy ais and girly cute ai names for the girl ais"; by seat,
/// red, yellow or purple in turn, under any of 48 names, until then). The
/// names a town holds ([`TOWN_BODIES`], 32) are more than a side has, so
/// some of a full town are "Bot".
fn ai_member(seat: usize, id: u64, seats: &[Member], elsewhere: &[Member]) -> Member {
    let _ = seat;
    let boy = fastrand::bool();
    let names: Vec<&str> = roundtown_net::ai_names(boy)
        .iter()
        .copied()
        .filter(|name| {
            seats
                .iter()
                .chain(elsewhere)
                .all(|member| member.name != *name)
        })
        .collect();
    Member {
        id,
        name: fastrand::choice(names).unwrap_or("Bot").into(),
        look: roundtown_net::ai_look(boy, fastrand::usize(..)).into(),
        ai: true,
    }
}

/// Everyone in the room, standing on the path round the pond in the middle of
/// the town.
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
            // Fixed over you, the camera is as it is wherever you arrive.
            if !orbit.overhead {
                *orbit = OrbitCamera::behind(&island::arrival(Venue::Town, seat));
            }
        } else {
            commands.entity(body).insert((Townsfolk(member.id), Speaker::Townsfolk));
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
            commands.entity(body).insert((Townsfolk(id), Speaker::Townsfolk));
        }
    }
}

/// What every body is, whoever plays it: its name, its look, where it stands
/// and its walk.
fn body(assets: &AssetServer, member: &Member, at: Transform) -> impl Bundle {
    (
        Name::new(member.name.clone()),
        dressed_as(assets, &member.look),
        at,
        WalkCycle::default(),
    )
}

/// The model a body in `look` is drawn with. Given to a body that has one
/// already, the old is taken down and the new put up in its place, and its
/// walk bones bound again (`bind_walk_bones`).
pub(crate) fn dressed_as(assets: &AssetServer, look: &str) -> WorldAssetRoot {
    WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(model(look))))
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
    spawn_player_at(commands, assets, member, seat, venue, mine, island::arrival(venue, seat))
}

/// A body for `member`, in `seat`, standing at `spawn` on `venue`, as
/// [`spawn_player`] makes one: for somewhere other than where its seat
/// arrives, like Bridge Wars' waiting room (`arena`).
pub(crate) fn spawn_player_at(
    commands: &mut Commands,
    assets: &AssetServer,
    member: &Member,
    seat: usize,
    venue: Venue,
    mine: bool,
    spawn: Transform,
) -> Entity {
    let mut avatar = commands.spawn((
        body(assets, member, spawn),
        venue,
        Seat(seat),
        Balance::default(),
        PlayerJump::default(),
        Riding::default(),
        Shove::default(),
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
            on: None,
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
            Riding::default(),
            WalkFromMotion {
                last: at.translation,
                on: None,
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
        Entity,
        &'static mut Transform,
        &'static mut Venue,
        &'static mut PlayerJump,
        Option<&'static mut WalkFromMotion>,
        Option<&'static mut ai::Steering>,
        Has<Player>,
        Option<&'static mut Riding>,
        Option<&'static mut Shove>,
    ),
>;

/// Puts `body` on `venue`, where `seat` arrives there, facing in: whatever
/// jump it was in dropped, whatever car it was riding left behind, whatever
/// walk the computer had it on forgotten, and the leap not taken for a
/// stride. If it is yours, the camera swings round behind you.
pub(crate) fn send(
    bodies: &mut Whereabouts,
    orbit: &mut OrbitCamera,
    body: Entity,
    venue: Venue,
    seat: usize,
) {
    put(bodies, orbit, body, venue, island::arrival(venue, seat));
}

/// Puts `body` on `venue`, standing `at`, as [`send`] does.
pub(crate) fn put(
    bodies: &mut Whereabouts,
    orbit: &mut OrbitCamera,
    body: Entity,
    venue: Venue,
    at: Transform,
) {
    let Ok((_, mut transform, mut on, mut jump, seen, steering, mine, riding, shove)) = bodies.get_mut(body)
    else {
        return;
    };
    transform.set_if_neq(at);
    on.set_if_neq(venue);
    *jump = PlayerJump::default();
    if let Some(mut shove) = shove {
        *shove = Shove::default();
    }
    if let Some(mut riding) = riding {
        riding.set_if_neq(Riding(None));
    }
    if let Some(mut seen) = seen {
        seen.last = at.translation;
        seen.on = None;
    }
    if let Some(mut steering) = steering {
        steering.stop();
    }
    if mine && !orbit.overhead {
        orbit.yaw = OrbitCamera::behind(&at).yaw;
    }
}

/// The model for a look. One this build has never heard of — a newer client's,
/// say — is drawn as the first look rather than left out: whoever wears it is
/// still in the room.
pub(crate) fn model(look: &str) -> &'static str {
    LOOKS
        .iter()
        .find(|(name, _)| *name == look)
        .map_or(LOOKS[0].1, |(_, model)| model)
}

/// Reads each body's walk off how far it went since last frame. Only ground
/// covered counts: rising and falling in a jump is not walking, and nor is
/// being carried along by the car, so that a rider stands still on it. A
/// body going faster than a walk, running in Bridge Wars (`arena`), reads
/// as more than a walk, and its legs swing the faster for it
/// (`animate_walk`).
fn walk_from_motion(
    time: Res<Time>,
    cars: Query<&Driven>,
    mut bodies: Query<(&Transform, Option<&Riding>, &mut WalkFromMotion, &mut WalkCycle)>,
) {
    let dt = time.delta_secs();
    for (transform, riding, mut seen, mut walk) in &mut bodies {
        let on = riding.and_then(|riding| riding.0);
        let here = match on.and_then(|car| cars.get(car).ok()) {
            Some(car) => car.now.inward(transform.translation),
            None => transform.translation,
        };
        let moved = if seen.on == on {
            here.xz().distance(seen.last.xz())
        } else {
            0.0
        };
        seen.last = here;
        seen.on = on;
        walk.speed = if dt > 0.0 {
            (moved / dt / MOVE_SPEED).min(2.0)
        } else {
            0.0
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_looks_a_person_chooses_between_come_first() {
        let people: Vec<&str> = LOOKS.iter().map(|(look, _)| *look).take(PEOPLE_LOOKS.len()).collect();
        assert_eq!(people, PEOPLE_LOOKS);
        // An account from before the choice, and a look this build has not
        // heard of, are drawn as the boy.
        assert_eq!(model(roundtown_net::PLAYER_LOOK), model("boy"));
        assert_eq!(model("hat"), model("boy"));
        assert_eq!(model("boy"), MODEL);
        assert_eq!(model("girl"), GIRL_MODEL);
        assert_ne!(model("girl"), model("boy"));
        assert_eq!(models().len(), 6);
        assert_eq!(&models()[..2], &[MODEL, GIRL_MODEL]);
        // The computer's players are boys or girls at random, in any of
        // the colours of their side, each of which has a model, under a name
        // of their side: over a few towns' worth, about half boys, and every
        // colour seen.
        for look in BOY_LOOKS.iter().chain(&GIRL_LOOKS) {
            assert!(LOOKS.iter().any(|(name, _)| name == look), "{look} has no model");
        }
        let seats: Vec<Member> = (0..300).map(|seat| ai_member(seat, seat as u64 + 1, &[], &[])).collect();
        let boys = seats.iter().filter(|member| BOY_LOOKS.contains(&member.look.as_str())).count();
        let girls = seats.iter().filter(|member| GIRL_LOOKS.contains(&member.look.as_str())).count();
        assert_eq!(boys + girls, 300);
        assert!((100..=200).contains(&boys), "{boys} of 300 are boys");
        for look in BOY_LOOKS.iter().chain(&GIRL_LOOKS) {
            assert!(seats.iter().any(|member| member.look == *look), "nobody wears {look}");
        }
        for member in &seats {
            let names: &[&str] = if BOY_LOOKS.contains(&member.look.as_str()) { &BOY_NAMES } else { &GIRL_NAMES };
            assert!(names.contains(&member.name.as_str()), "{} is a {}", member.name, member.look);
        }
        // A side's names run out before a town is full, and the rest are Bot.
        let full = {
            let mut seats: Vec<Member> = Vec::new();
            for seat in 0..60 {
                seats.push(ai_member(seat, seat as u64 + 1, &seats, &[]));
            }
            seats
        };
        assert!(full.iter().any(|member| member.name == "Bot"));
    }

    #[test]
    fn you_are_who_you_signed_up_as() {
        let nobody = Profile::default();
        assert_eq!((me(&nobody).name.as_str(), me(&nobody).look.as_str()), ("You", "boy"));
        let hajun = Profile {
            name: "Hajun".into(),
            look: "girl".into(),
        };
        let mut lobby = offline_room(&nobody);
        lobby.sign_up(&hajun);
        let mine = lobby.mine();
        assert_eq!((mine.id, mine.name.as_str(), mine.look.as_str(), mine.ai), (1, "Hajun", "girl", false));
        assert_eq!(lobby.seats.len(), SEATS);
    }
}
