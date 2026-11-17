//! Blend In, hide and seek in the town (since 2026-11-18; Hajun: "1 random
//! hider chosen as the person walking around pretending to be an NPC/AI. 1~3
//! other people trying to find who is the person, by punching with a
//! aura sword. This game min player count is 2, so if there 2 ppl, it will not
//! fill the 2 slot with 2 ai's but rather start game with just 2 real ppl.
//! There is a time limit, and incorrect guesses cost you time by
//! stunning/blinding etc. ... Map = roundtownmap ... You swap until every one
//! has tried hiding atleast once"): two to four people and never the
//! computer, each hiding once among a crowd of the town's people while the
//! others look for them, on the town's own map with nobody of the town's in
//! it ([`Venue::BlendIn`]). Its rules are in this module, and the server and
//! the device both play by them, as by Bridge Wars' in
//! [`crate::bridge_wars`]; how its table fills, starts and pays is
//! [`crate::table`]'s.
//!
//! **A game** is a plain [`Game`], stepped by whoever runs it: the server,
//! Blend In being played online only (with nobody to hide from, offline
//! there is no game). The crowd it is played among ([`Crowd`], `crowd`)
//! walks from the moment its table opens, and the game takes it over as the
//! play begins. Play is kept inside [`RING`], the ring road's outer edge
//! (mine): nobody walks past it. Every part of a round ends by the server's
//! clock, the one every device counts down by ([`BlendRound::until`]),
//! however the frames it was stepped by add up.
//!
//! **Rounds** ("You swap until every one has tried hiding atleast once"):
//! as many as there are people, each hiding once, in a thrown order. Each:
//!
//! 1. **Choose** ([`CHOOSE`], over as soon as everyone has said they are
//!    ready): everyone picks a look and a name to go by this round. The
//!    hider is offered one as the crowd's would be thrown (a side, a look of
//!    it, a name of it nobody else has), each seeker what they wore last
//!    round, in the first their own; whatever is not picked in time stays.
//!    A name has to be a name ([`profile::name`]); a seeker's no other
//!    seeker's, the hider's no seeker's. A seeker is never refused the
//!    hider's, which would tell them what the hider goes by: should a
//!    seeker pick it, the hider goes by another as the hide begins.
//! 2. **Hide** ([`HIDE`]): everyone is dressed as they chose, every body in
//!    the game goes by a new id, and the crowd by new names and looks. The
//!    seekers stand where they came in, on the path round the pond, and see
//!    nothing; the hider is put among the crowd at least [`HIDER_CLEAR`]
//!    from all of them, free to go where they like.
//! 3. **Seek** ([`SEEK`]): the seekers move, at a run ([`SEEKER_PACE`]),
//!    and swing, and the hider walks as the crowd does. A swing that lands
//!    on the hider finds them ([`Game::strike`]): a point to the finder,
//!    and the round is over. One that lands on one of the crowd is a wrong
//!    guess: it staggers, and the seeker can do nothing and see nothing for
//!    [`STUN_FIRST`], [`STUN_MORE`] longer for each more wrong guess of the
//!    round. One that lands on another seeker does nothing. If the time
//!    runs out the point is the hider's ("You get point as hider for
//!    surviving until timer runs out, And you get point as seeker for
//!    finding the hider before timer runs out").
//! 4. **Shown** ([`SHOWN`]): where the hider stood all along, for everyone
//!    to see; then the next round, or the end.
//!
//! **The end**: places by points, equal points sharing a place ("given a
//! draw if both found each other or both couldnt find each other", of
//! two); the table pays by them ([`prizes`]). A hider who leaves, or whose
//! device has been gone [`AWAY`], ends their round with no point (mine);
//! with fewer than two people left the game ends, places by points so far.
//!
//! **Who is who** ([`Guise`]): every body in the game, the crowd's and the
//! players', goes by an id the game gives it, thrown afresh every round
//! from one range ([`FIRST_ID`]), with a name and a look, and that is all
//! anyone is sent of it: never whose it is ([`Who`], which only whoever
//! runs the game knows), never that it is the computer's (a
//! [`Member::ai`] of false for all). The hider's seat is no secret
//! ([`BlendRound::hider`]): everyone knows who hides, not where. Every
//! throw of the game's dice and the crowd's is sealed ([`Sealed`]), so
//! that nothing anyone is sent (the ids, the crowd's names and looks and
//! walks) says anything of the throws they are not sent: where the hider
//! was put, which ids are whose. That holds only while the seeds are
//! secrets: whoever runs the game throws them from something nobody can
//! work out, and sends them nowhere.

mod crowd;
mod dice;

use std::f32::consts::{PI, TAU};
use std::sync::{Arc, LazyLock};

use bevy::math::{Quat, Vec2, Vec3, Vec3Swizzles};
use bevy::prelude::{App, Component};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

pub use crowd::{
    APART, Crowd, DAZED, Doing, LONGEST_WAIT, Leanings, Mind, PROBE, SHORE_MARGIN, SHORTEST_WAIT, STAGGER, STAGGER_SECS,
    STAND_A_TURN, STEP, TRIES, TURN_ONLY, TURN_SPEED, WALK_MAX, WALK_MIN, WALK_SPEED, Walker, walks, yaw_of,
};
pub use dice::Sealed;

use crate::nav::Grid;
use crate::{BOY_LOOKS, Member, Venue, ai_look, ai_names, profile, table};

/// How a game of Blend In fills, starts and pays: two to four people, the
/// computer never ("This game min player count is 2 ... start game with
/// just 2 real ppl"), beginning 15 s after the second has sat down, or at
/// once with four; 50 coins to join.
pub const SHAPE: table::Shape = table::Shape {
    most: 4,
    fewest: 2,
    computer: false,
    wait: 15.0,
    countdown: 5.0,
    results: 15.0,
    fee: 50,
};

/// How far out from the middle of the town a game is played, in metres
/// (mine): to just inside the white line along the ring road's outer edge,
/// which is 70.3 to 70.7 m out, the kerb beyond it from 71. Nobody walks
/// past it: the crowd never does ([`grid`] has nothing past it), and a
/// person seen past it ([`RING_SLACK`]) is put back.
pub const RING: f32 = 70.0;
/// How many of the town's people walk among the players: the town's 32
/// bodies ([`TOWN_BODIES`](crate::TOWN_BODIES)) with four people.
pub const CROWD: usize = 28;
/// How long each part of a round is, in seconds: choosing a look and a
/// name, at the most (mine: Hajun's "maybe 30s"); the hider getting into
/// place while the seekers see nothing; the seek (mine, 2:30); and the
/// hider shown where they were at the end of it.
pub const CHOOSE: f32 = 30.0;
pub const HIDE: f32 = 15.0;
pub const SEEK: f32 = 150.0;
pub const SHOWN: f32 = 6.0;
/// How far from every seeker the hider is put as the hide begins, at the
/// least, in metres (mine): too far to be seen going anywhere by anyone
/// who could see.
pub const HIDER_CLEAR: f32 = 25.0;
/// How long a seeker is stunned for a wrong guess, in seconds: the first of
/// the round, and how much longer each after it than the one before
/// ("incorrect guesses cost you time by stunning/blinding etc."; the
/// numbers mine): 4, 7, 10 and on.
pub const STUN_FIRST: f32 = 4.0;
pub const STUN_MORE: f32 = 3.0;
/// How fast a seeker goes in the seek, in metres a second: at a run, as
/// everyone does in Bridge Wars ([`RUN_SPEED`](crate::bridge_wars::RUN_SPEED),
/// 10), where the hider walks as the crowd does ([`WALK_SPEED`], 7); who
/// goes how fast is [`BlendRound::pace`]. Mine, the design having had them
/// move "as in the town": a hider who has given themselves away and walks
/// off from a seeker who is no faster is seldom caught. In whole games of
/// stand-ins (`many_games_of_blend_in`, 40 games of each, of two, three and
/// four people, every stand-in's thresholds thrown), one who walked as the
/// crowd does until a seeker came within a thrown 5 to 11 m, and then away
/// from them, was found in 9, 18 and 26% of rounds by seekers at a walk,
/// where one who went on walking as the crowd does was found in 24, 38 and
/// 42%; by seekers at a run, in 20, 32 and 49%, as often as one who walked
/// on (21, 30 and 47%). The crowd never runs, so a seeker running gives
/// away no more than their sword does. Their device runs them at this pace
/// in the seek, as Bridge Wars' does.
pub const SEEKER_PACE: f32 = crate::bridge_wars::RUN_SPEED;
/// How far apart, across the ground, a seeker and the body they struck can
/// be, where each last was, for the strike to count, in metres: Bridge
/// Wars' punch's slack (the server's `PUNCH_SLACK`), the reach and the way
/// each may have gone since.
pub const STRIKE_SLACK: f32 = 6.5;
/// How long the hider's device may be gone before their round is given up,
/// with no point, in seconds (mine): a body that stands still that long
/// while the crowd walks is no hiding.
pub const AWAY: f32 = 20.0;
/// How far past [`RING`] a person can be seen before they are put back, in
/// metres, and how long after being put back they are let be, in seconds,
/// for where they were put to reach the game.
pub const RING_SLACK: f32 = 1.5;
pub const PUT_BACK_EVERY: f32 = 2.0;
/// How far from where they were stood still a seeker can have gone by the
/// time they may move again, in metres, before they are put back there
/// (the hide's end, a stun's): no device but one gone wrong moves them.
pub const HELD_SLACK: f32 = 2.0;
/// How far from one of the crowd the hider is put as the hide begins, in
/// metres (mine): among them, not on top of anyone.
pub const BESIDE: (f32, f32) = (1.5, 4.0);
/// The first of the ids the game gives the bodies in it, and how many
/// there are to throw from ([`Guise::id`]): three eighths of the way up the
/// range, clear of people's, which count up from 1, and of everything that
/// counts down from somewhere in it: Bridge Wars' computer players from its
/// middle and their builders from a quarter of it, House Builder's from its
/// top, and the tables' computer players from an eighth of it.
pub const FIRST_ID: u64 = u64::MAX / 8 * 3;
pub const IDS: u64 = 1 << 48;

/// Which of a seed's streams of sealed throws ([`Sealed`]) the game's own
/// dice throw (the hiding order, the hider's first guise and any name
/// thrown for them in its place, the ids, where the hider is put), and
/// which the crowd's (where they stand and walk, what they wear, how they
/// go about it): so that one seed can serve both.
const GAME_STREAM: u16 = 1;
const CROWD_STREAM: u16 = 2;

/// The town's middle as the crowd walks it, to just inside [`RING`] (69.3 m
/// out, nothing past the ring looked at): Hajun's `roundtownmap.glb`, baked
/// by the game with its own collision as its town AIs could stand and walk
/// (the game's `blend_in_bake`, a test, which writes `town.nav` and fails
/// when the map has changed since), and built into both binaries, the
/// server having no map.
static TOWN: LazyLock<Arc<Grid>> = LazyLock::new(|| {
    Arc::new(
        Grid::decode(include_bytes!("town.nav"))
            .expect("town.nav is a baked grid: bake it again (RT_BAKE_NAV=1 cargo test --lib blend_in_bake)"),
    )
});

/// The town's middle as the crowd walks it, as last baked.
pub fn grid() -> Arc<Grid> {
    TOWN.clone()
}

/// Something asked in a game of Blend In ([`Ask::BlendIn`](crate::Ask::BlendIn)).
/// A strike is a punch ([`Ask::Punch`](crate::Ask::Punch)), on whichever
/// body it landed on.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Ask {
    /// Dress me as this when the hide begins: a name to go by and one of
    /// [`BOY_LOOKS`](crate::BOY_LOOKS) and [`GIRL_LOOKS`](crate::GIRL_LOOKS)
    /// to wear. While choosing only; a name that is no name, or one a
    /// seeker other than me has asked for this round, is refused
    /// ([`Tell::Refused`]); the hider's never is, to a seeker.
    Disguise { name: String, look: String },
    /// I have chosen.
    Ready,
}

impl Ask {
    /// The placeholder the four games' shared base was laid with
    /// (2026-11-18), which a test of that base still names: kept until that
    /// line names a real one, and to go with it.
    #[allow(non_upper_case_globals)]
    #[doc(hidden)]
    pub const Nothing: Ask = Ask::Ready;
}

/// Something told one device in a game of Blend In
/// ([`Tell::BlendIn`](crate::Tell::BlendIn)).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Tell {
    /// You struck one of the town's own: you can do nothing, and see
    /// nothing, until `until`, by the server's clock, in seconds since 1970
    /// (every device has it: the game's `clock`), or until the seek is
    /// over, whichever comes first.
    Stunned { until: f64 },
    /// The name you asked for is no name, or a seeker's this round: pick
    /// another. A seeker is never refused the hider's name: that would say
    /// what the hider goes by.
    Refused,
    /// What you wear this round: sent as each round's choosing begins (what
    /// you wear if you choose nothing else), to the hider as the hide
    /// begins if a seeker took the name they chose (one thrown in its
    /// place, [`Event::Renamed`]), and again to a device that comes back.
    /// The hider's is for them alone. Since G2 also in answer to every
    /// [`Ask::Disguise`], what the choice stands at once it has been heard
    /// (the asked, or what it was if that was refused), and to everyone as
    /// the hide begins ([`Event::Dressed`]): what they are dressed as.
    Wear { name: String, look: String },
    /// The seeker in seat `seeker` struck one of the town's own, a wrong
    /// guess ([`Event::Wrong`]): told everyone in the game, for its chat to
    /// say so (G2; Hajun: "incorrect guesses cost you time by
    /// stunning/blinding etc."). Never which of the crowd it was: the one
    /// struck staggers where it stands, for anyone looking to see.
    Struck { seeker: u8 },
}

impl Tell {
    /// As [`Ask::Nothing`]: the base's placeholder, until its test names a
    /// real one.
    #[allow(non_upper_case_globals)]
    #[doc(hidden)]
    pub const Nothing: Tell = Tell::Refused;
}

/// A game of Blend In as everyone in it sees it: which round of how many,
/// who hides, how far the round has got and until when, everyone's points,
/// who has chosen, and, once a round is over, who found the hider and when,
/// and which body was theirs. On the table's entity beside its
/// [`table::Board`], sent whenever any of it changes: a few dozen bytes, a
/// few times a round.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BlendRound {
    /// Which round it is, from 0, of how many there will be: as many as
    /// people hide, one fewer for everyone who leaves before their turn.
    pub round: u8,
    pub rounds: u8,
    /// The seat hiding this round.
    pub hider: u8,
    pub stage: BlendStage,
    /// When this part of the round ends, by the server's clock, in seconds
    /// since 1970; once the game is over, when it ended.
    pub until: f64,
    /// Everyone's points, by seat.
    pub points: Vec<u8>,
    /// Who has said they are ready, by seat, while choosing; nobody else.
    pub ready: Vec<bool>,
    /// Who found the hider this round, by seat, and how many whole seconds
    /// into the seek; none while the seek goes on, or if it ran out.
    pub found: Option<(u8, u16)>,
    /// The id the hider's body went by this round ([`Guise::id`]), once the
    /// round is over and where they were is shown: a secret until then.
    pub unmasked: Option<u64>,
    /// The id each seeker's body goes by this round, by seat, from the hide
    /// to the end of the seek: the bodies that hold the aura sword, for
    /// every device to draw it in their hands, and that a seeker's swing
    /// passes by (G2). Nobody's (0) for the hider's seat and for anyone
    /// gone; empty the rest of the time. Seekers are no secret, sword in
    /// hand: what is kept from everyone is which of the rest is the hider.
    pub swords: Vec<u64>,
}

impl BlendRound {
    /// How fast the person in `seat` goes, in metres a second: at a run
    /// while they seek ([`SEEKER_PACE`]), at the crowd's walk
    /// ([`WALK_SPEED`]) otherwise, the hider always. Their own device keeps
    /// to it, as it keeps every pace in the game; the server checks a move
    /// only against the fastest anything goes (the car's boost), so that
    /// nothing checks this one.
    pub fn pace(&self, seat: usize) -> f32 {
        if self.stage == BlendStage::Seek && usize::from(self.hider) != seat {
            SEEKER_PACE
        } else {
            WALK_SPEED
        }
    }
}

/// How far a round of Blend In has got.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlendStage {
    /// Everyone choosing a look and a name.
    Choose,
    /// The hider getting into place; the seekers see nothing.
    Hide,
    /// The seekers looking.
    Seek,
    /// The round over, the hider shown where they stood.
    Shown,
    /// Everyone has hidden: the game is over.
    Over,
}

/// Registers this game's own replicated components: the round
/// ([`BlendRound`]).
pub fn register(app: &mut App) {
    app.replicate::<BlendRound>();
}

/// Where the player in `seat` stands as they arrive on the game's venue, and which way they
/// face: where they would in the town, on the path round its pond
/// ([`TOWN_ARRIVAL_RADIUS`](crate::TOWN_ARRIVAL_RADIUS)), facing the fountain. Where a
/// seeker stands through each hide too.
pub fn arrival(seat: usize) -> (Vec3, Quat) {
    crate::arrival(Venue::Town, seat)
}

/// What each seat is paid, from how the game came out (`results.places`, `scores`, `total`);
/// `people[seat]` is false for the computer's seats, which are paid nothing. By place, as
/// many places as sat down at the table (a seat each, every one a person's): of two 100 and
/// nothing, of three 110, 40 and nothing, of four 120, 70, 10 and nothing; those who tie
/// share ([`table::split`]), so that two who draw have their fee back.
pub fn prizes(results: &table::Results, people: &[bool]) -> Vec<u32> {
    let by_place: &[u32] = match results.places.len() {
        0..=2 => &[100, 0],
        3 => &[110, 40, 0],
        _ => &[120, 70, 10, 0],
    };
    table::split(by_place, &results.places)
        .into_iter()
        .enumerate()
        .map(|(seat, coins)| if people.get(seat).copied().unwrap_or(false) { coins } else { 0 })
        .collect()
}

/// Who a body in a game goes by and what it wears: an id, a name and a
/// look, and nothing else, the same for the crowd's and the players'.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Guise {
    /// Thrown afresh every round from one range ([`FIRST_ID`]), so that
    /// nothing in it says whose the body is; before the first hide, a
    /// player's is their account's.
    pub id: u64,
    pub name: String,
    pub look: String,
}

impl Guise {
    /// As the room is sent it: as a person, whoever it is
    /// ([`Member::ai`] false), the crowd no less than the players.
    pub fn member(&self) -> Member {
        Member {
            id: self.id,
            name: self.name.clone(),
            look: self.look.clone(),
            ai: false,
        }
    }
}

/// Whose a body in a game is: one of the crowd, by its number, or the
/// player in a seat. Only whoever runs the game knows it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Who {
    Crowd(usize),
    Seat(usize),
}

/// A body in a game ([`Game::bodies`]): whose, who it goes by and what it
/// wears, where it stands and which way it faces.
#[derive(Clone, Copy, Debug)]
pub struct Figure<'a> {
    pub who: Who,
    pub guise: &'a Guise,
    pub at: Vec3,
    pub yaw: f32,
}

/// What a strike came to ([`Game::strike`]).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Struck {
    /// Nothing: no seek, no seeker, stunned, too far, another seeker, or no
    /// body by that id.
    Nothing,
    /// The hider, found.
    Found,
    /// One of the crowd: the seeker is stunned this long, in seconds.
    Wrong { stun: f32 },
}

/// Something that happened in a game, for whoever runs it to carry out.
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    /// Round `round` (from 0) has begun, `hider` hiding: everyone chooses.
    /// Tell each person what they wear unless they choose otherwise
    /// ([`Game::choice`], [`Tell::Wear`]).
    Choosing { round: u8, hider: usize },
    /// The disguise `seat` asked for was refused: tell them
    /// ([`Tell::Refused`]).
    Refused { seat: usize },
    /// The name the hider (`seat`) chose is a seeker's too, which no two
    /// bodies may share and which a seeker is never refused: as the hide
    /// begins they go by another of their look's side instead, thrown as
    /// the crowd's are ([`Game::choice`]). Tell them ([`Tell::Wear`]).
    /// Comes just before [`Event::Dressed`].
    Renamed { seat: usize },
    /// The hide has begun: every body in the game goes by a new id, the
    /// crowd's by new names and looks, and each player wears what they
    /// chose ([`Game::bodies`]). Dress every body again.
    Dressed,
    /// Stand the person in `seat` here, facing `yaw`, wherever their device
    /// has them: the hider among the crowd and the seekers where they came
    /// in as the hide begins, a seeker who moved while they could not, or
    /// anyone past the ring.
    Place { seat: usize, at: Vec3, yaw: f32 },
    /// The seek has begun.
    Seeking,
    /// `seeker` struck one of the crowd, numbered `crowd`, which staggers
    /// back: they are stunned until `until`, by the server's clock (tell
    /// them, [`Tell::Stunned`]).
    Wrong { seeker: usize, crowd: usize, until: f64 },
    /// `seeker` found `hider`, `after` seconds into the seek: their point.
    Found { seeker: usize, hider: usize, after: f32 },
    /// The seek ran out with `hider` unfound: their point.
    Hidden { hider: usize },
    /// `hider`'s round is given up, with no point: they have left, or their
    /// device has been gone too long ([`AWAY`]).
    Void { hider: usize },
    /// The game is over ([`Game::outcome`]).
    Over,
}

/// One player in a game.
#[derive(Clone, Debug)]
struct Seat {
    /// What their body wears now: their account's until the first hide.
    worn: Guise,
    /// The name and look they will wear from the coming hide.
    chosen: (String, String),
    /// Where their body stands and faces: as their device last said, or
    /// where the game last put it.
    at: Vec3,
    yaw: f32,
    points: u8,
    ready: bool,
    /// Left for good: the seat is theirs for the results, with no body.
    gone: bool,
    /// Since when their device has been gone, by the server's clock, while
    /// it is.
    away: Option<f64>,
    /// Until when they are stunned, by the server's clock, while they are.
    stun: Option<f64>,
    /// Wrong guesses this round.
    wrong: u8,
    /// Where they were stood still, while they are (the hide, a stun).
    held: Option<Vec3>,
    /// Until when, by the server's clock, they are let be after being put
    /// back inside the ring.
    put_back: f64,
}

/// A game of Blend In, from its first round's choosing to its end: the
/// rules alone, no world, no network. Its methods are what the people in
/// it do, each checked as if it came from a cheat; [`Game::step`] moves it
/// on and says what happened ([`Event`]), for the runner to carry out.
pub struct Game {
    crowd: Crowd,
    dice: Sealed,
    seats: Vec<Seat>,
    /// Who hides, round by round: thrown, less whoever leaves before their
    /// turn.
    order: Vec<usize>,
    round: usize,
    stage: BlendStage,
    begun: bool,
    /// The server's clock as it was last told ([`Game::begin`],
    /// [`Game::step`]), in seconds since 1970: what every part of a round
    /// ends by, and every `until` is told by.
    clock: f64,
    /// When this part of the round ends, by that clock.
    until: f64,
    /// When the seek began, by it.
    seek_from: f64,
    found: Option<(u8, u16)>,
    unmasked: Option<u64>,
    events: Vec<Event>,
}

impl Game {
    /// A game of the people in `members`, a seat each by their place there,
    /// each as their account has them, standing where they came in; among
    /// `crowd`, which has been walking while they waited. Made as the play
    /// begins, once nobody can join or leave the table but by leaving the
    /// game; the hiding order thrown here, by `seed`, which is a secret: it
    /// says where every hider will be put, and the ids every body will go
    /// by ([`Sealed`]).
    pub fn new(members: &[Member], seed: u64, crowd: Crowd) -> Self {
        let mut dice = Sealed::new(seed, GAME_STREAM);
        let seats: Vec<Seat> = members
            .iter()
            .enumerate()
            .map(|(seat, member)| {
                let (at, yaw) = came_in(seat, crowd.grid());
                let worn = Guise {
                    id: member.id,
                    name: member.name.clone(),
                    look: member.look.clone(),
                };
                Seat {
                    chosen: (worn.name.clone(), worn.look.clone()),
                    worn,
                    at,
                    yaw,
                    points: 0,
                    ready: false,
                    gone: false,
                    away: None,
                    stun: None,
                    wrong: 0,
                    held: None,
                    put_back: 0.0,
                }
            })
            .collect();
        let mut order: Vec<usize> = (0..seats.len()).collect();
        for last in (1..order.len()).rev() {
            order.swap(last, dice.pick(last + 1));
        }
        Self {
            crowd,
            dice,
            seats,
            order,
            round: 0,
            stage: BlendStage::Choose,
            begun: false,
            clock: 0.0,
            until: 0.0,
            seek_from: 0.0,
            found: None,
            unmasked: None,
            events: Vec::new(),
        }
    }

    /// The play begins, `clock` being the server's clock now, in seconds
    /// since 1970: the first round's choosing. With fewer than two people
    /// left, the game is over at once.
    pub fn begin(&mut self, clock: f64) {
        if self.begun {
            return;
        }
        self.begun = true;
        self.clock = clock;
        for them in &mut self.seats {
            if them.away.is_some() {
                them.away = Some(clock);
            }
        }
        let seats = &self.seats;
        self.order.retain(|&seat| !seats[seat].gone);
        if self.in_play() < 2 {
            self.over();
        } else {
            self.choose();
        }
    }

    /// Where the person in `seat` is now, as their device last said.
    pub fn sync(&mut self, seat: usize, at: Vec3, yaw: f32) {
        if let Some(seat) = self.seats.get_mut(seat)
            && !seat.gone
            && at.is_finite()
            && yaw.is_finite()
        {
            seat.at = at;
            seat.yaw = yaw;
        }
    }

    /// `seat` would be dressed as `name` and `look` from the hide: while
    /// choosing, a look either side wears, and a name that is one
    /// ([`profile::name`]) and no seeker's but their own this round,
    /// whatever its capitals. Whether it is to be; a name refused is said
    /// ([`Event::Refused`]).
    ///
    /// A seeker's name is held against the other seekers' alone, never the
    /// hider's: a seeker refused a name would know it for the hider's, and
    /// the hider's name hangs over them all the seek; and a modified device
    /// could ask for every one of the town's 48 in one choosing. Should a
    /// seeker pick the hider's, the hider goes by another as the hide begins
    /// ([`Event::Renamed`]). The hider's is held against every seeker's,
    /// which are no secret: they carry swords.
    pub fn disguise(&mut self, seat: usize, name: &str, look: &str) -> bool {
        if !self.begun || self.stage != BlendStage::Choose || !self.playing(seat) || !profile::look(look) {
            return false;
        }
        let hider = self.order[self.round];
        let free = profile::name(name).filter(|name| {
            let lower = name.to_lowercase();
            self.seats
                .iter()
                .enumerate()
                .all(|(other, them)| other == seat || other == hider || them.gone || them.chosen.0.to_lowercase() != lower)
        });
        let Some(name) = free else {
            self.events.push(Event::Refused { seat });
            return false;
        };
        self.seats[seat].chosen = (name, look.to_owned());
        true
    }

    /// `seat` has chosen. Once everyone here has, the hide begins.
    pub fn ready(&mut self, seat: usize) -> bool {
        if !self.begun || self.stage != BlendStage::Choose || !self.playing(seat) {
            return false;
        }
        self.seats[seat].ready = true;
        true
    }

    /// `seat`'s punch landed on the body that goes by `who`
    /// ([`Ask::Punch`](crate::Ask::Punch)): if they are seeking, not
    /// stunned, and that body is within reach of where they are
    /// ([`STRIKE_SLACK`]), the hider is found, or one of the crowd is
    /// struck in a wrong guess and they are stunned. Anything else comes to
    /// nothing.
    pub fn strike(&mut self, seat: usize, who: u64) -> Struck {
        let Some(hider) = self.hider() else {
            return Struck::Nothing;
        };
        if self.stage != BlendStage::Seek || seat == hider || !self.playing(seat) || self.seats[seat].stun.is_some() {
            return Struck::Nothing;
        }
        let from = self.seats[seat].at;
        let Some(target) = self.who(who) else {
            return Struck::Nothing;
        };
        if self.where_is(target).0.xz().distance(from.xz()) > STRIKE_SLACK {
            return Struck::Nothing;
        }
        match target {
            Who::Seat(struck) if struck == hider => {
                let after = (self.clock - self.seek_from).max(0.0) as f32;
                self.seats[seat].points = self.seats[seat].points.saturating_add(1);
                self.found = Some((seat as u8, after as u16));
                self.events.push(Event::Found { seeker: seat, hider, after });
                self.shown();
                Struck::Found
            }
            Who::Seat(_) => Struck::Nothing,
            Who::Crowd(index) => {
                let clock = self.clock;
                let them = &mut self.seats[seat];
                them.wrong = them.wrong.saturating_add(1);
                let stun = STUN_FIRST + STUN_MORE * f32::from(them.wrong - 1);
                let until = clock + f64::from(stun);
                them.stun = Some(until);
                them.held = Some(from);
                self.crowd.stagger(index, from.xz());
                self.events.push(Event::Wrong { seeker: seat, crowd: index, until });
                Struck::Wrong { stun }
            }
        }
    }

    /// `seat`'s device has gone (`away`), or is back. The hider's round is
    /// given up once theirs has been gone [`AWAY`] by the server's clock;
    /// anyone else's waits.
    pub fn away(&mut self, seat: usize, away: bool) {
        let clock = self.clock;
        if let Some(them) = self.seats.get_mut(seat)
            && !them.gone
        {
            them.away = match (away, them.away) {
                (true, Some(since)) => Some(since),
                (true, None) => Some(clock),
                (false, _) => None,
            };
        }
    }

    /// `seat` has left for good: their turn to hide, if it has not come, is
    /// given up; if they were hiding, their round is, with no point; with
    /// fewer than two people left, the game is over.
    pub fn leave(&mut self, seat: usize) {
        if !self.playing(seat) {
            return;
        }
        let hider = self.hider();
        let them = &mut self.seats[seat];
        them.gone = true;
        them.ready = false;
        them.held = None;
        them.stun = None;
        them.away = None;
        if let Some(turn) = self.order.iter().position(|&other| other == seat)
            && (turn > self.round || !self.begun)
        {
            self.order.remove(turn);
        }
        if !self.begun || self.stage == BlendStage::Over {
            return;
        }
        if self.in_play() < 2 {
            self.over();
        } else if hider == Some(seat) && matches!(self.stage, BlendStage::Choose | BlendStage::Hide | BlendStage::Seek) {
            self.void();
        }
    }

    /// Moves the game on: the crowd by `dt` seconds, the frame's time;
    /// the round, the stuns and the time anyone's device has been gone to
    /// `clock`, the server's clock now in seconds since 1970, the one every
    /// device counts down by. So that a part of a round ends when the
    /// devices' countdowns say it does, however the frames add up (a frame's
    /// time is cut short after a stall). Everything that happened since the
    /// last step, what the people in it did included, for the runner to
    /// carry out.
    pub fn step(&mut self, dt: f32, clock: f64) -> Vec<Event> {
        let people: Vec<Vec2> = self.seats.iter().filter(|them| !them.gone).map(|them| them.at.xz()).collect();
        self.crowd.step(dt, &people);
        if self.begun && self.stage != BlendStage::Over {
            if clock.is_finite() {
                self.clock = clock;
            }
            for seat in 0..self.seats.len() {
                if self.seats[seat].stun.is_some_and(|until| self.clock >= until) {
                    self.seats[seat].stun = None;
                    self.hold_to(seat);
                }
            }
            let hider = self.order[self.round];
            let lost = self.seats[hider].away.is_some_and(|since| self.clock - since >= f64::from(AWAY));
            let due = self.clock >= self.until;
            match self.stage {
                BlendStage::Choose if due || self.everyone_ready() => self.hide(),
                BlendStage::Hide | BlendStage::Seek if lost => self.void(),
                BlendStage::Hide if due => self.seek(),
                BlendStage::Seek if due => {
                    self.seats[hider].points = self.seats[hider].points.saturating_add(1);
                    self.events.push(Event::Hidden { hider });
                    self.shown();
                }
                BlendStage::Shown if due => self.next(),
                _ => {}
            }
            self.keep_inside();
        }
        std::mem::take(&mut self.events)
    }

    /// Where the player in `seat` stands and faces.
    pub fn at(&self, seat: usize) -> (Vec3, f32) {
        self.seats.get(seat).map_or((Vec3::ZERO, 0.0), |them| (them.at, them.yaw))
    }

    /// Every body in the game: the crowd's, by their numbers, then the
    /// players' still here, by seat.
    pub fn bodies(&self) -> impl Iterator<Item = Figure<'_>> + '_ {
        let crowd = self.crowd.walkers().iter().enumerate().map(|(index, walker)| Figure {
            who: Who::Crowd(index),
            guise: &walker.guise,
            at: walker.at,
            yaw: walker.yaw,
        });
        let seats = self.seats.iter().enumerate().filter(|(_, them)| !them.gone).map(|(seat, them)| Figure {
            who: Who::Seat(seat),
            guise: &them.worn,
            at: them.at,
            yaw: them.yaw,
        });
        crowd.chain(seats)
    }

    /// Whose the body that goes by `id` is, if it is in the game.
    pub fn who(&self, id: u64) -> Option<Who> {
        if let Some(index) = self.crowd.walkers().iter().position(|walker| walker.guise.id == id) {
            return Some(Who::Crowd(index));
        }
        self.seats
            .iter()
            .position(|them| !them.gone && them.worn.id == id)
            .map(Who::Seat)
    }

    /// The name and look `seat` wears from the coming hide, as things stand:
    /// what to tell them as the choosing begins ([`Tell::Wear`]).
    pub fn choice(&self, seat: usize) -> Option<(&str, &str)> {
        self.seats.get(seat).map(|them| (them.chosen.0.as_str(), them.chosen.1.as_str()))
    }

    /// Until when `seat` is stunned, by the server's clock, if they are.
    pub fn stunned(&self, seat: usize) -> Option<f64> {
        self.seats.get(seat).and_then(|them| them.stun)
    }

    /// Who hides this round: none before the play begins or once it is over.
    pub fn hider(&self) -> Option<usize> {
        (self.begun && self.stage != BlendStage::Over).then(|| self.order[self.round])
    }

    /// Whether `seat` is seeking: hiding or seeking, and not the hider.
    pub fn seeking(&self, seat: usize) -> bool {
        matches!(self.stage, BlendStage::Hide | BlendStage::Seek) && self.playing(seat) && self.hider() != Some(seat)
    }

    pub fn stage(&self) -> BlendStage {
        self.stage
    }

    /// The crowd, as it is now.
    pub fn crowd(&self) -> &Crowd {
        &self.crowd
    }

    /// What the room is sent ([`BlendRound`]): the same until something in
    /// it has changed, so that it is sent only then.
    pub fn round(&self) -> BlendRound {
        let round = self.round.min(self.order.len().saturating_sub(1));
        let choosing = self.begun && self.stage == BlendStage::Choose;
        let armed = self.begun && matches!(self.stage, BlendStage::Hide | BlendStage::Seek);
        let hider = self.order.get(round).copied();
        BlendRound {
            round: round as u8,
            rounds: self.order.len() as u8,
            hider: hider.unwrap_or(0) as u8,
            stage: self.stage,
            until: self.until,
            points: self.seats.iter().map(|them| them.points).collect(),
            ready: self.seats.iter().map(|them| choosing && them.ready).collect(),
            found: self.found,
            unmasked: self.unmasked,
            swords: if armed {
                self.seats
                    .iter()
                    .enumerate()
                    .map(|(seat, them)| if Some(seat) == hider || them.gone { 0 } else { them.worn.id })
                    .collect()
            } else {
                Vec::new()
            },
        }
    }

    /// For trying the game out (the server's `RT_ALLOW_SKIP`, a tap on the
    /// board): the part of the round under way ends now, by the server's
    /// clock, as if its time were up, on the next step. The choosing as if
    /// everyone had chosen, the hide, the seek as run out (the hider's
    /// point), the round shown. Nothing before the play nor once it is over.
    pub fn hurry(&mut self) {
        if self.begun && self.stage != BlendStage::Over {
            self.until = self.until.min(self.clock);
        }
    }

    /// Once the game is over: the place each seat came, by its points, 0
    /// first and those with the same points tied ([`table::Results`]); the
    /// points; and no total.
    pub fn outcome(&self) -> Option<(Vec<u8>, Vec<u32>, Option<u32>)> {
        (self.begun && self.stage == BlendStage::Over).then(|| {
            let points: Vec<u8> = self.seats.iter().map(|them| them.points).collect();
            let places = points
                .iter()
                .map(|&mine| points.iter().filter(|&&theirs| theirs > mine).count() as u8)
                .collect();
            (places, points.iter().map(|&mine| u32::from(mine)).collect(), None)
        })
    }

    fn playing(&self, seat: usize) -> bool {
        self.seats.get(seat).is_some_and(|them| !them.gone)
    }

    fn in_play(&self) -> usize {
        self.seats.iter().filter(|them| !them.gone).count()
    }

    /// Whether everyone still here whose device is has chosen.
    fn everyone_ready(&self) -> bool {
        self.seats.iter().all(|them| them.gone || them.away.is_some() || them.ready)
    }

    /// Where the body `who` stands and faces.
    fn where_is(&self, who: Who) -> (Vec3, f32) {
        match who {
            Who::Crowd(index) => self.crowd.walkers().get(index).map_or((Vec3::ZERO, 0.0), |walker| (walker.at, walker.yaw)),
            Who::Seat(seat) => self.at(seat),
        }
    }

    /// The stage `stage`, `secs` long from now by the server's clock.
    fn enter(&mut self, stage: BlendStage, secs: f32) {
        self.stage = stage;
        self.until = self.clock + f64::from(secs);
    }

    /// The round's choosing: the hider offered a guise as one of the crowd
    /// would be thrown, nobody else's name; everyone else what they wore.
    fn choose(&mut self) {
        self.enter(BlendStage::Choose, CHOOSE);
        self.found = None;
        self.unmasked = None;
        let hider = self.order[self.round];
        for them in &mut self.seats {
            them.ready = false;
            them.wrong = 0;
            them.stun = None;
            them.held = None;
            them.chosen = (them.worn.name.clone(), them.worn.look.clone());
        }
        let taken: Vec<String> = self
            .seats
            .iter()
            .enumerate()
            .filter(|&(seat, them)| seat != hider && !them.gone)
            .map(|(_, them)| them.chosen.0.to_lowercase())
            .collect();
        self.seats[hider].chosen = thrown(&mut self.dice, &taken);
        self.events.push(Event::Choosing {
            round: self.round as u8,
            hider,
        });
    }

    /// The hide: everyone dressed, the hider by another name if a seeker
    /// took theirs, every body with a new id, the crowd dressed afresh; the
    /// seekers where they came in, the hider among the crowd, far from them.
    fn hide(&mut self) {
        self.enter(BlendStage::Hide, HIDE);
        let hider = self.order[self.round];
        let seekers: Vec<String> = self
            .seats
            .iter()
            .enumerate()
            .filter(|&(seat, them)| seat != hider && !them.gone)
            .map(|(_, them)| them.chosen.0.to_lowercase())
            .collect();
        if seekers.contains(&self.seats[hider].chosen.0.to_lowercase()) {
            let boy = BOY_LOOKS.contains(&self.seats[hider].chosen.1.as_str());
            self.seats[hider].chosen.0 = named(&mut self.dice, boy, &seekers).to_owned();
            self.events.push(Event::Renamed { seat: hider });
        }
        let worn: Vec<u64> = self
            .seats
            .iter()
            .map(|them| them.worn.id)
            .chain(self.crowd.walkers().iter().map(|walker| walker.guise.id))
            .collect();
        let ids = fresh_ids(&mut self.dice, self.crowd.len() + self.seats.len(), &worn);
        let (for_crowd, for_seats) = ids.split_at(self.crowd.len());
        for (them, &id) in self.seats.iter_mut().zip(for_seats) {
            them.ready = false;
            if !them.gone {
                them.worn = Guise {
                    id,
                    name: them.chosen.0.clone(),
                    look: them.chosen.1.clone(),
                };
            }
        }
        let taken: Vec<String> = self.seats.iter().filter(|them| !them.gone).map(|them| them.worn.name.to_lowercase()).collect();
        self.crowd.redress(&taken, for_crowd);
        self.events.push(Event::Dressed);
        let mut seekers = Vec::new();
        for seat in 0..self.seats.len() {
            if seat == hider || self.seats[seat].gone {
                continue;
            }
            let (at, yaw) = came_in(seat, self.crowd.grid());
            self.put(seat, at, yaw);
            self.seats[seat].held = Some(at);
            seekers.push(at.xz());
        }
        let spot = hiding_place(&self.crowd, &mut self.dice, &seekers);
        let (at, yaw) = match spot {
            Some(spot) => (
                Vec3::new(spot.x, self.crowd.grid().height(spot).unwrap_or(0.0), spot.y),
                self.dice.between(-PI, PI),
            ),
            None => came_in(hider, self.crowd.grid()),
        };
        self.put(hider, at, yaw);
        self.seats[hider].held = None;
    }

    /// The seek: anyone stood still for the hide who has gone anywhere is
    /// put back first.
    fn seek(&mut self) {
        self.enter(BlendStage::Seek, SEEK);
        self.seek_from = self.clock;
        for seat in 0..self.seats.len() {
            self.hold_to(seat);
        }
        self.events.push(Event::Seeking);
    }

    /// The round over: the hider shown; stuns over with it.
    fn shown(&mut self) {
        self.enter(BlendStage::Shown, SHOWN);
        let hider = self.order[self.round];
        self.unmasked = Some(self.seats[hider].worn.id);
        for them in &mut self.seats {
            them.stun = None;
            them.held = None;
        }
    }

    /// The hider's round given up, with no point.
    fn void(&mut self) {
        let hider = self.order[self.round];
        self.events.push(Event::Void { hider });
        self.next();
    }

    /// The next round, or the end.
    fn next(&mut self) {
        if self.round + 1 < self.order.len() && self.in_play() >= 2 {
            self.round += 1;
            self.choose();
        } else {
            self.over();
        }
    }

    fn over(&mut self) {
        self.enter(BlendStage::Over, 0.0);
        for them in &mut self.seats {
            them.stun = None;
            them.held = None;
            them.ready = false;
        }
        self.events.push(Event::Over);
    }

    /// Stands `seat` at `at`, facing `yaw`, and says so.
    fn put(&mut self, seat: usize, at: Vec3, yaw: f32) {
        let them = &mut self.seats[seat];
        them.at = at;
        them.yaw = yaw;
        self.events.push(Event::Place { seat, at, yaw });
    }

    /// `seat` may move again: if they were stood still and have gone
    /// anywhere since, they are put back where they were stood.
    fn hold_to(&mut self, seat: usize) {
        let Some(held) = self.seats[seat].held.take() else {
            return;
        };
        let them = &self.seats[seat];
        if !them.gone && them.at.xz().distance(held.xz()) > HELD_SLACK {
            let yaw = them.yaw;
            self.put(seat, held, yaw);
        }
    }

    /// Puts back inside the ring anyone seen past it.
    fn keep_inside(&mut self) {
        for seat in 0..self.seats.len() {
            let them = &self.seats[seat];
            if them.gone || self.clock < them.put_back || them.at.xz().length() <= RING + RING_SLACK {
                continue;
            }
            let inside = them.at.xz().clamp_length_max(RING - 2.0);
            let (at, yaw) = match self.crowd.grid().nearest_clear(inside, 4.0) {
                Some(spot) => (Vec3::new(spot.x, self.crowd.grid().height(spot).unwrap_or(0.0), spot.y), them.yaw),
                None => came_in(seat, self.crowd.grid()),
            };
            self.seats[seat].put_back = self.clock + f64::from(PUT_BACK_EVERY);
            self.put(seat, at, yaw);
        }
    }
}

/// Where the player in `seat` came in ([`arrival`]), on the ground there,
/// and the turn about the upright they face.
fn came_in(seat: usize, grid: &Grid) -> (Vec3, f32) {
    let (at, turn) = arrival(seat);
    let ahead = turn * Vec3::NEG_Z;
    (at.with_y(grid.height(at.xz()).unwrap_or(at.y)), (-ahead.x).atan2(-ahead.z))
}

/// Where the hider is put as the hide begins: beside one of the crowd far
/// enough from every seeker ([`HIDER_CLEAR`], [`BESIDE`]), or, with none,
/// anywhere far enough; open ground with room round it, nobody standing
/// nearer it than a walk of the crowd's may end ([`APART`], thrown).
fn hiding_place(crowd: &Crowd, dice: &mut Sealed, seekers: &[Vec2]) -> Option<Vec2> {
    let far = |at: Vec2, by: f32| seekers.iter().all(|seeker| seeker.distance(at) >= HIDER_CLEAR + by);
    let walkers: Vec<Vec2> = crowd.walkers().iter().map(|walker| walker.at.xz()).collect();
    let apart = dice.between(APART.0, APART.1);
    let room = |at: Vec2| crowd.roomy(at) && walkers.iter().all(|walker| walker.distance(at) >= apart);
    let beside: Vec<Vec2> = walkers.iter().copied().filter(|&walker| far(walker, BESIDE.1)).collect();
    for _ in 0..40 {
        let Some(&by) = beside.get(dice.pick(beside.len())) else {
            break;
        };
        let spot = by + Vec2::from_angle(dice.between(0.0, TAU)) * dice.between(BESIDE.0, BESIDE.1);
        if far(spot, 0.0) && room(spot) {
            return Some(spot);
        }
    }
    let open = crowd.open();
    for _ in 0..400 {
        let Some(&spot) = open.get(dice.pick(open.len())) else {
            break;
        };
        if far(spot, 0.0) && room(spot) {
            return Some(spot);
        }
    }
    None
}

/// A name and a look as one of the crowd's would be thrown: a boy or a girl
/// by the coin, one of that side's looks, and one of that side's names that
/// is none of `taken` (small letters), or the other side's if that side has
/// none left.
fn thrown(dice: &mut Sealed, taken: &[String]) -> (String, String) {
    let boy = dice.roll() < 0.5;
    let name = named(dice, boy, taken);
    let look = ai_look(crate::BOY_NAMES.contains(&name), dice.pick(3));
    (name.to_owned(), look.to_owned())
}

/// One of the names of a boy (`boy`) or a girl of the crowd's that is none
/// of `taken` (small letters), thrown; or, with none of that side's left,
/// one of the other side's.
fn named(dice: &mut Sealed, boy: bool, taken: &[String]) -> &'static str {
    let free = |boy: bool| -> Vec<&'static str> {
        ai_names(boy)
            .iter()
            .copied()
            .filter(|name| !taken.contains(&name.to_lowercase()))
            .collect()
    };
    let names = Some(free(boy)).filter(|names| !names.is_empty()).unwrap_or_else(|| free(!boy));
    names.get(dice.pick(names.len())).copied().unwrap_or(ai_names(boy)[0])
}

/// `count` ids thrown from [`FIRST_ID`]'s range, none alike, nor any of
/// `besides`: each the low bits of a sealed throw, so that no id says
/// anything of any other, nor of which body it was thrown for.
fn fresh_ids(dice: &mut Sealed, count: usize, besides: &[u64]) -> Vec<u64> {
    let mut ids: Vec<u64> = Vec::with_capacity(count);
    while ids.len() < count {
        let id = FIRST_ID + dice.bits() % IDS;
        if !ids.contains(&id) && !besides.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

#[cfg(test)]
mod tests;
