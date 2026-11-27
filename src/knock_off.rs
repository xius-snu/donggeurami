//! Knock Off, on this device (since 2026-11-18; Hajun: "You spawn on a tiny
//! icy circular island with 2~4 other ppl (fill with ai like bridgewars and
//! its deathmatch so 1v1v1v1). Controls (left side is joystick for
//! direction, right side is a up down arrow for controlling intensity).
//! Everyone is frozen in place, and before every certain time interval like
//! 10 second, you choose the direction and intensity that you wanna slide.
//! Then after the interval, everyone goes toward that direction and
//! intensity and collide, some ppl will be knocked off. You keep going until
//! last player surviving is the winner. If you are out, you can spectate or
//! exit. Map = iceisland (not sure what camera is good for this one, you
//! choose)"): four seats on a disc of ice in the sea, everyone aiming at
//! once in secret and then sliding at once.
//!
//! Its rules are `roundtown_net::knock_off`'s ([`Game`]), which the server
//! plays online and this device offline: **the same game either way**, only
//! what it is read from differs. Online the round comes from the server
//! (the replicated [`KnockRound`] on its table's entity); offline from the
//! game this device steps itself by the town's clock, which is then its own
//! time of day. Everything else is drawn from the round alike:
//!
//! * **Every body is placed here** while the game is played, yours and
//!   everyone else's (`Frozen { placed: true }`: neither the stick nor the
//!   server's updates move one): on its spot while everyone aims, and in a
//!   slide where this device's own play of it has it ([`Slide`], from the
//!   round's start and launches, stepped to the town's clock less the
//!   moment it begins, which comes out to the bit as the server's did),
//!   then settled onto where the server says the slide left everyone. The
//!   slide is begun the moment the reveal's time is up, by the reveal's own
//!   round, which already says everything it begins from, not when the
//!   slide's round comes a little after: so that nobody jumps forward as it
//!   begins (a check of the game found everyone did, by as far as a metre
//!   at the hardest).
//! * **Your aim** is yours alone until the reveal: the stick for which way,
//!   the up and down arrows on the right for how hard, and Ready; sent to
//!   the server as it changes (or put into the game here, offline), and
//!   drawn on the ice from your feet (`ice`).
//! * **The camera** looks down over the island from your own seat's side,
//!   from the moment you sit down to the end (mine: Hajun left it to me),
//!   still while anyone aims or slides, and coming in to the ice that is
//!   left as it breaks, and to whoever won at the end.
//! * **The island** is the table's own, thrown afresh for every table
//!   (since 2026-11-27, Hajun: "each game, a truly random shape of island
//!   forms"): online the server's, sent as the table opens
//!   (`rules::Island`); offline this device's, thrown as you sit down. Its
//!   ice is drawn and stood on as it is shaped (`ice`). While the table
//!   waits everyone walks about on it, nobody through anybody
//!   ([`keep_apart`]), and as play begins everyone is where they stood
//!   (Hajun: "wherever ur last position was ... is your starting
//!   position").
//!
//! What is drawn on the ice (the arrows, the cracks, the ice breaking off,
//! the splashes, the dust where two meet) is `ice`'s; the buttons, the plate
//! top centre, the names over heads, the headlines, the lines in the chat
//! and the panel for whoever is out are `screen`'s. The table round it all
//! (the seats, the fee, the results) is `table`'s.

mod ice;
mod screen;

use std::f32::consts::{PI, TAU};
use std::sync::Arc;

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::knock_off::{self as rules, Ending, Floe, Game, Island, KnockRound, KnockStage, Loiter, ROOM, STOOD_BACK, Slide};
use roundtown_net::{Ask as NetAsk, Member, Minigame, Venue};

use crate::clock::TownClock;
use crate::lobby::{self, Whereabouts};
use crate::net::Online;
use crate::table::{self, AtTable, ComputerSeat, TableView};
use crate::ui::ui;
use crate::{Dead, Frozen, HeldCamera, Landing, OrbitCamera, Player, ThirdPersonCamera, TouchControls};

/// Something the server says to this device alone in its game of Knock Off
/// (`roundtown_net::knock_off::Tell`), handed on by `net`.
#[derive(Message, Clone, Debug)]
pub(crate) struct Told(pub roundtown_net::knock_off::Tell);

/// How far down the camera looks at the middle of the ice, in radians: 55
/// degrees (mine), steep enough that everyone on the ice is seen apart, and
/// shallow enough that the ice is seen as an island in the sea, with which
/// way is which plain from where you stand.
const CAMERA_PITCH: f32 = 55.0 * std::f32::consts::PI / 180.0;
/// How far round the ice the sea is kept in sight, in metres: enough to see
/// who went in where.
const SEA_MARGIN: f32 = 2.0;
/// What is kept clear of the island at each edge of the screen, as a share
/// of the screen: the top for the plate (its turn and the line under it),
/// the bottom for what is said across the screen and the panel for whoever
/// is out (`screen`), over the near sea, and the sides a little. The island
/// is lifted between them, the camera looking at a point a little nearer
/// than the middle (it had looked at the middle itself, the island low on
/// the screen with room to spare over it, and the headlines, which had
/// nowhere else to go, stood on the far side's heads: a check of the game,
/// 2026-11-18).
const CLEAR_TOP: f32 = 0.165;
/// What is kept clear at the top while the table waits and counts down, as
/// a share of the screen: the table's board stands there then, the game's
/// name, the seconds left and Leave under them, down to nearly a quarter of
/// the screen, and Leave stood on the far player's head (a check of the
/// merged game, 2026-11-18; mine). The camera eases to [`CLEAR_TOP`] as
/// play begins and the board goes.
const CLEAR_TOP_WAITING: f32 = 0.26;
const CLEAR_BOTTOM: f32 = 0.10;
const CLEAR_SIDES: f32 = 0.03;
/// The least the camera is fitted to as the ice breaks away, in metres from
/// the middle, before the sea round it: in the last turns, floes of 2.25 m
/// and less, room still to see whoever goes off them into the sea.
const FEWEST: f32 = 3.5;
/// How long the camera takes to come in to the ice that is left as it
/// breaks, or to whoever won, in seconds (mine: within the break's own 1.2,
/// so that it is still for every aim). With the ice it had stayed as far out
/// as the whole island all game, and by the eighth turn the last two were
/// figures of a few pixels on a phone (a check of the game, 2026-11-18).
const CAMERA_EASE: f32 = 0.9;
/// How far round whoever won the camera comes in to, at the end, in metres
/// (3.4 was a close-up their head filled, the island's facets as big as
/// they are).
const WINNER_VIEW: f32 = 5.5;
/// How many points round the circle the camera is fitted to, and how many
/// times it is tried nearer or further (and the point it looks at moved),
/// for where it fits.
const FIT_POINTS: usize = 40;
const FIT_TRIES: usize = 24;
/// How far the camera is pushed in toward the ice where two meet hard, as
/// a share of how far it shakes (mine): a kick, a moment.
const KICK: f32 = 1.8;
/// Whoever won, on the ice at the end: how high each hop goes, in metres,
/// how long one takes, in seconds, and how soon after it is over they begin
/// (mine: arms up, as the intro's landing pose has them).
const HOP: f32 = 0.32;
const HOP_SECS: f32 = 0.55;
const HOP_FROM: f32 = 0.3;

/// How a body is posed sliding (`Landing`, the intro's own pose): knees
/// half bent, arms out to the sides (mine: as someone keeping their
/// balance on ice); and how fast it has to be going for it, in metres a
/// second.
const SLIDING_SQUAT: f32 = 0.45;
const SLIDING_ARMS: f32 = 0.62;
const SLIDING_FROM: f32 = 0.6;
/// How fast a sliding body turns to face the way it slides, in radians a
/// second (mine): half round in a quarter of a second, so that one knocked
/// sideways or back is seen to swing round, not to jump.
const SLIDING_TURN: f32 = 12.0;
/// How far over its feet a body's hips are, in metres, for how far it comes
/// down crouched (`Landing::sink`).
const HIPS: f32 = 0.75;
/// How long a body takes to settle onto where the server says the slide
/// left it, if it is anywhere else, in seconds.
const SETTLE: f32 = 0.2;
/// How fast whoever stood on what broke off drops with it, in metres a
/// second each second.
const DROP: f32 = 14.0;

/// How long after a power arrow is pressed it begins to repeat, held, and
/// how often after that, in seconds.
const REPEAT_AFTER: f32 = 0.35;
const REPEAT_EVERY: f32 = 0.08;
/// How often an aim being changed is sent to the server, at the most, in
/// seconds: the last before the aim is over is the one played, and it is
/// always sent.
const SEND_EVERY: f32 = 0.1;
/// The turn an aim is for that the server has said is yours as this device
/// comes back to the game, before the round has said which turn it is: the
/// one being aimed now.
const THIS_TURN: u16 = u16::MAX;
/// How hard every turn's aim begins (since 2026-11-18; Hajun: "make the
/// default strength to be 1 rather than 0 every round"): the stick alone
/// sends you off, at the least there is. Which way is still nothing until
/// the stick is touched, so that whoever does nothing still stays where
/// they are ([`take_aim`]).
const START_POWER: u8 = 1;
/// How far under the ice's top, or out past its edge, you have gone into
/// the sea while the game waits, in metres, to be stood back on your spot.
const FELL_IN: f32 = 0.3;

/// Knock Off on this device, while you sit at one of its tables: your seat,
/// whose the table is, the round as it is drawn, and everything kept
/// between frames to draw it.
#[derive(Resource, Default)]
struct Knock {
    /// Your seat, and whether the server runs the game; none away from it.
    seat: Option<u8>,
    online: bool,
    /// The island the table plays on, and the seed it was made from, once
    /// it is known: online the server's (`rules::Island`), offline thrown
    /// here as you sit down; and whether your body is held where it stands
    /// until it is (`find_the_island`), there being nothing yet to stand on.
    floe: Option<Arc<Floe>>,
    island: Option<u64>,
    held: bool,
    /// Offline: the game this device plays, from the moment it is played,
    /// and whether its end has been told the table.
    game: Option<Game>,
    finished: bool,
    /// The computer's players standing about the ice while the table waits,
    /// offline (the server's own online).
    loiter: Option<Loiter>,
    /// The round as it is drawn, and the one before it.
    round: Option<KnockRound>,
    /// The slide being played, if one is, or the last, until the next aim.
    slide: Option<Played>,
    /// Your aim this turn.
    aim: Aim,
    /// Per seat: which way the body faces, where it was drawn last, where it
    /// settles from as the ice breaks, and whether it has been seen into the
    /// sea this turn.
    faces: Vec<f32>,
    drawn: Vec<Option<Vec3>>,
    settle_from: Vec<Option<Vec3>>,
    splashed: Vec<bool>,
    /// The stage and turn the bodies were last placed in.
    placed_in: Option<(u16, KnockStage)>,
    /// The bodies this has put its marks on (`Frozen`, `Dead`, `Landing`),
    /// to take them off again.
    marked: Vec<Entity>,
    /// Where you came, once you are out and it is known; and whether you
    /// have pressed Watch on the panel saying so, or Leave, offline, where
    /// the rest of the game is played out first, for your place to be
    /// paid, and you go as its results come.
    place: Option<u8>,
    watching: bool,
    leaving: bool,
    /// A shake of the camera, after two meet: how much longer, and how hard
    /// (as a share of how far the camera is from the ice, `ice::shake_for`).
    shake: Option<(f32, f32)>,
}

impl Knock {
    /// Begun afresh at a table: in `seat`, the server's or this device's;
    /// offline, on an island thrown here from a seed nobody chose.
    fn sit(seat: u8, online: bool) -> Self {
        let island = (!online).then(|| fastrand::u64(..));
        Self {
            seat: Some(seat),
            online,
            floe: island.map(|seed| Arc::new(Floe::thrown(seed))),
            island,
            ..default()
        }
    }

    /// How big the ice is now, as a share of the island: all of it before
    /// the game.
    fn size(&self) -> f32 {
        self.playing().map_or(1.0, |round| round.size)
    }

    /// The round, once the game is played (turn 1 on).
    fn playing(&self) -> Option<&KnockRound> {
        self.round.as_ref().filter(|round| round.turn > 0)
    }

    /// Whether you are still on the ice.
    fn in_play(&self) -> bool {
        match (self.seat, self.playing()) {
            (Some(seat), Some(round)) => round.out.get(usize::from(seat)).is_some_and(Option::is_none),
            (Some(_), None) => true,
            _ => false,
        }
    }

    /// Whether you are aiming now: everyone aims, you are on the ice, and
    /// you have not said you are ready.
    fn aiming(&self) -> bool {
        self.playing().is_some_and(|round| round.stage == KnockStage::Aim) && self.in_play() && !self.aim.ready
    }

    /// Whether this turn's slide is being played here: its round says so,
    /// or the reveal's time is up and it has begun here as the reveal's
    /// round said it would ([`place`]), its own round not here yet.
    fn sliding(&self) -> bool {
        self.playing().is_some_and(|round| match round.stage {
            KnockStage::Slide => true,
            KnockStage::Reveal => self.slide.as_ref().is_some_and(|played| played.turn == round.turn),
            _ => false,
        })
    }
}

/// A slide as this device plays it: in which turn, when it began by the
/// town's clock, what it began from as the round said it (where everyone
/// stood, how each set off, how far the ice reached), the slide itself,
/// how it will come out (played out at once as it begins: for the chat to
/// say two who knock each other in in one line), and how many of its hits
/// have been shown.
struct Played {
    turn: u16,
    since: f64,
    start: Vec<Option<(f32, f32)>>,
    launches: Vec<(f32, f32)>,
    size: f32,
    slide: Slide,
    ending: Ending,
    hits_shown: usize,
}

impl Played {
    /// The slide `round` says (the reveal's, or the slide's own), begun at
    /// `since` by the town's clock, on `floe`.
    fn of(round: &KnockRound, since: f64, floe: &Arc<Floe>) -> Self {
        let start: Vec<Option<Vec2>> = round.start.iter().map(|at| at.map(|(x, z)| Vec2::new(x, z))).collect();
        let launches: Vec<Vec2> = round.launches.iter().map(|&(x, z)| Vec2::new(x, z)).collect();
        Self {
            turn: round.turn,
            since,
            start: round.start.clone(),
            launches: round.launches.clone(),
            size: round.size,
            slide: Slide::new(&start, &launches, floe, round.size),
            ending: Slide::play_out(&start, &launches, floe, round.size),
            hits_shown: 0,
        }
    }

    /// Whether it is the very slide `round` says, begun at `since`: the one
    /// the reveal's round began here is, once the slide's own comes, unless
    /// something changed in the reveal's last moment (someone left in it,
    /// standing nowhere and setting off nowhere), which the slide's round
    /// is then begun again from.
    fn is(&self, round: &KnockRound, since: f64) -> bool {
        self.turn == round.turn && self.since == since && self.size == round.size && self.start == round.start && self.launches == round.launches
    }
}

/// `from` turned toward `to` by no more than `most`, the short way round,
/// in radians about the upright.
fn turned(from: f32, to: f32, most: f32) -> f32 {
    let off = (to - from + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    from + off.clamp(-most, most)
}

/// Your aim this turn: the turn it is for, which way (across the ground, of
/// length one, or nothing before the stick has been touched) and how hard,
/// whether you have said you are ready, and what the server was last sent
/// of it and how long ago.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct Aim {
    turn: u16,
    way: Vec2,
    power: u8,
    ready: bool,
    sent: Option<(Vec2, u8)>,
    since_sent: f32,
}

/// What the buttons on the right have been pressed for since last read
/// (`screen`): steps of power up or down, and Ready; and the arrow held
/// down, by which pointer, how long, and how many steps it has repeated.
#[derive(Resource, Default)]
struct Pressed {
    power: i32,
    ready: bool,
    held: Option<Holding>,
}

/// A power arrow held down.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Holding {
    pointer: bevy::picking::pointer::PointerId,
    up: bool,
    held: f32,
    repeated: u32,
}

/// Knock Off's systems that read the round and move the game on, before
/// anything is drawn from it.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct KnockSystems;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Told>()
        .init_resource::<Knock>()
        .init_resource::<Pressed>()
        .configure_sets(Update, KnockSystems.after(table::TableSystems).before(crate::move_bodies))
        .add_systems(
            Update,
            (enter, hear, find_the_island, play_offline, follow, loiter_offline, fell_in, take_aim, send_aim, hold_the_camera)
                .chain()
                .in_set(KnockSystems)
                .after(crate::read_player_input),
        )
        .add_systems(Update, (keep_apart, place).chain().after(crate::move_bodies).before(crate::animate_walk));
    ice::plugin(app);
    screen::plugin(app);
}

/// The buttons round the screen at a table of Knock Off (`screen`).
pub(crate) fn spawn_bar(commands: &mut Commands, assets: &AssetServer, images: &mut Assets<Image>) {
    screen::spawn_bar(commands, assets, images);
}

/// Puts the table's headline, the game's name as you come to it, where
/// this game's own go: low over the near sea (`screen`).
pub(crate) fn lower_headline(headline: EntityCommands) {
    screen::lower(headline);
}

/// What its score is called among the results: the file's
/// (`words.knock_off.score`), the turns each was still on the ice at the
/// end of (mine: the place has a column of its own there).
pub(crate) fn score_word() -> String {
    ui().words.knock_off.score.clone()
}

/// What the line under the results says of what the whole table made: Knock
/// Off counts nothing together.
pub(crate) fn total_words(_total: u32) -> Option<String> {
    None
}

/// Each seat's colour (mine: Bridge Wars' red, blue, green and yellow, as
/// bright as they read on white ice, the file's `colours.knock_off`): on
/// its arrow, the ring at its feet, its name over its head and in the chat.
fn seat_colour(seat: usize) -> Color {
    let colours = ui().colours.knock_off;
    [colours.seat_red, colours.seat_blue, colours.seat_green, colours.seat_yellow][seat % 4]
}

/// Which way something faces that looks along `way` across the ground (x,
/// z): its turn about the upright from -Z, as a body's. None for no way.
fn facing(way: Vec2) -> Option<f32> {
    (way.length_squared() > 1e-8).then(|| (-way.x).atan2(-way.y))
}

/// The camera over the whole of `floe` for whoever sits in `seat`, as it
/// is while the table waits and as the game begins ([`view_of`]): for the
/// tests ([`camera_target`] says it in play).
#[cfg(test)]
fn over_the_ice(seat: u8, aspect: f32, floe: &Floe) -> Transform {
    view_of(seat, aspect, &fit_points(Some(floe), Fit::Ice(1.0)), CLEAR_TOP)
}

/// What the camera is fitted to ([`camera_target`]): the ice at a share of
/// the island ([`rules::size_in`]) and the sea round it; or everything
/// within so far of a spot (whoever won, at the end).
#[derive(Clone, Copy, PartialEq, Debug)]
enum Fit {
    Ice(f32),
    Round(Vec2, f32),
}

/// The points on the ground the camera keeps on the screen for `fit`, on
/// `floe` (none yet: the round island it was): round the ice, every other
/// point of its outline taken [`SEA_MARGIN`] further out from its middle,
/// and never round less than [`FEWEST`] of it about the ice's middle; or
/// round the spot.
fn fit_points(floe: Option<&Floe>, fit: Fit) -> Vec<Vec2> {
    let circle = |centre: Vec2, reach: f32| (0..FIT_POINTS).map(move |k| centre + Vec2::from_angle(k as f32 * TAU / FIT_POINTS as f32) * reach);
    match (fit, floe) {
        (Fit::Round(centre, reach), _) => circle(centre, reach).collect(),
        (Fit::Ice(size), Some(floe)) => {
            let size = size.max(0.0);
            let mut points: Vec<Vec2> = floe
                .rim()
                .iter()
                .step_by(2)
                .map(|&at| at * size + at.normalize_or_zero() * SEA_MARGIN)
                .collect();
            points.extend(circle(floe.middle(size), FEWEST + SEA_MARGIN));
            points
        }
        (Fit::Ice(size), None) => circle(Vec2::ZERO, (rules::RADIUS * size).max(FEWEST) + SEA_MARGIN).collect(),
    }
}

/// The camera for whoever sits in `seat` (mine: Hajun, "not sure what
/// camera is good for this one, you choose"): from their own seat's side
/// ([`rules::side`]), looking down at [`CAMERA_PITCH`] along it, as near as
/// keeps every one of `points` on the ground (round the ice and the sea, or
/// whoever won, [`fit_points`]) on a screen `aspect` times as wide as it is
/// high, clear of what stands at its edges (`clear_top` of it at the top,
/// [`CLEAR_TOP`] in play, and the rest), and midway between the top and the
/// bottom kept clear, and between the sides; their side toward the bottom
/// of the screen, and the stick's up away from the camera, across the ice
/// toward its middle.
fn view_of(seat: u8, aspect: f32, points: &[Vec2], clear_top: f32) -> Transform {
    let side = rules::side(usize::from(seat)).normalize_or(Vec2::Y);
    let across = side.perp();
    let half = (crate::CAMERA_FOV * 0.5).tan();
    let aspect = aspect.clamp(0.5, 4.0);
    let (down, along) = (CAMERA_PITCH.sin(), CAMERA_PITCH.cos());
    // The middle of what is fitted, along the side and across it.
    let (mut near_side, mut far_side) = (f32::MAX, f32::MIN);
    let (mut left, mut right) = (f32::MAX, f32::MIN);
    for point in points {
        near_side = near_side.min(point.dot(side));
        far_side = far_side.max(point.dot(side));
        left = left.min(point.dot(across));
        right = right.max(point.dot(across));
    }
    if points.is_empty() {
        (near_side, far_side, left, right) = (0.0, 0.0, 0.0, 0.0);
    }
    let centre = side * (near_side + far_side) * 0.5 + across * (left + right) * 0.5;
    let reach = ((far_side - near_side).max(right - left) * 0.5).max(1.0);
    // From `distance` off the point `toward` metres from `centre` toward
    // you, looking at it.
    let view = |distance: f32, toward: f32| {
        let look = centre + side * toward;
        let eye = Vec3::new(look.x + side.x * distance * along, distance * down, look.y + side.y * distance * along);
        Transform::from_translation(eye).looking_at(Vec3::new(look.x, 0.0, look.y), Vec3::Y)
    };
    // Where the points come on the screen from `eye`: the lowest and the
    // highest, and how far to either side, as shares of half the screen;
    // none if any of them is behind the camera.
    let lies = |eye: &Transform| {
        let into = eye.to_matrix().inverse();
        let (mut low, mut high, mut wide) = (f32::MAX, f32::MIN, 0.0f32);
        for point in points {
            let seen = into.transform_point3(Vec3::new(point.x, 0.0, point.y));
            let deep = -seen.z;
            if deep <= 0.1 {
                return None;
            }
            let up = seen.y / deep / half;
            low = low.min(up);
            high = high.max(up);
            wide = wide.max((seen.x / deep / (half * aspect)).abs());
        }
        Some((low, high, wide))
    };
    let (bottom, top, sides) = (-1.0 + 2.0 * CLEAR_BOTTOM, 1.0 - 2.0 * clear_top, 1.0 - 2.0 * CLEAR_SIDES);
    let middle = (bottom + top) * 0.5;
    // From `distance`, the point looked at moved nearer you or further until
    // the points are midway between the top and the bottom kept clear (a
    // point nearer you lifts them up the screen); and whether they fit.
    let placed = |distance: f32| {
        let (mut away, mut near) = (-reach, reach);
        let mut eye = view(distance, 0.0);
        let mut fits = false;
        for _ in 0..FIT_TRIES {
            let toward = (away + near) * 0.5;
            eye = view(distance, toward);
            match lies(&eye) {
                Some((low, high, wide)) => {
                    fits = high - low <= top - bottom && wide <= sides;
                    if (low + high) * 0.5 > middle {
                        near = toward;
                    } else {
                        away = toward;
                    }
                }
                None => {
                    fits = false;
                    near = toward;
                }
            }
        }
        (eye, fits)
    };
    let (mut near, mut far) = (2.0f32, 200.0f32);
    for _ in 0..FIT_TRIES {
        let distance = (near + far) * 0.5;
        if placed(distance).1 {
            far = distance;
        } else {
            near = distance;
        }
    }
    placed(far).0
}

/// Whoever won, if it is over and they are on the ice, and where they
/// stand: the one left (by its place, first, and alone).
pub(super) fn winner_on_the_ice(round: &KnockRound) -> Option<(usize, Vec2)> {
    if round.stage != KnockStage::Over {
        return None;
    }
    let mut first = (0..round.places.len()).filter(|&seat| round.places[seat] == Some(0));
    let (seat, None) = (first.next()?, first.next()) else {
        return None;
    };
    let (x, z) = round.start.get(seat).copied().flatten()?;
    round.out.get(seat).is_some_and(Option::is_none).then_some((seat, Vec2::new(x, z)))
}

/// What the camera is fitted to now ([`view_of`], [`fit_points`]): the ice
/// and the sea round it. The whole island while the table waits; while
/// everyone aims, looks and slides, the ice there is this turn; as it
/// breaks, what will be left (so that the camera comes in as the edge
/// goes); and once it is over, whoever won, if they are on the ice.
fn camera_target(knock: &Knock) -> Fit {
    let Some(round) = knock.playing() else {
        return Fit::Ice(1.0);
    };
    match round.stage {
        KnockStage::Aim | KnockStage::Reveal | KnockStage::Slide => Fit::Ice(round.size),
        KnockStage::Break => Fit::Ice(round.next_size),
        KnockStage::Over => match winner_on_the_ice(round) {
            Some((_, at)) => Fit::Round(at, WINNER_VIEW),
            None => Fit::Ice(round.size),
        },
    }
}

/// The body in each seat at the table, as this device has them: yours, and
/// online everyone else's by who they are, offline the computer's by their
/// seat. None for a seat with no body here.
fn bodies_by_seat(
    knock: &Knock,
    view: &TableView,
    me: Option<Entity>,
    remotes: &Query<(Entity, &Member), With<Remote>>,
    computers: &Query<(Entity, &ComputerSeat)>,
) -> Vec<Option<Entity>> {
    let seats = view.members.len().max(4);
    (0..seats)
        .map(|seat| {
            if knock.seat == Some(seat as u8) {
                return me;
            }
            if knock.online {
                let id = view.members.get(seat).map_or(0, |member| member.id);
                (id != 0).then(|| remotes.iter().find(|(_, member)| member.id == id).map(|(body, _)| body)).flatten()
            } else {
                computers.iter().find(|(_, of)| usize::from(of.0) == seat).map(|(body, _)| body)
            }
        })
        .collect()
}

/// A table of Knock Off begun here (`table::Entered`): everything of one
/// before is dropped and begun afresh, in the seat it says; and away from
/// the tables (`table::Gone`), everything this put on any body is taken off
/// again. The same table begun afresh (a device back in the game it was in,
/// or a seat moved before play) keeps nothing either, what it needs coming
/// with the round, but that you have pressed Watch, out: the panel does not
/// come back to ask again.
#[allow(clippy::too_many_arguments)]
fn enter(
    mut entered: MessageReader<table::Entered>,
    mut gone: MessageReader<table::Gone>,
    mut knock: ResMut<Knock>,
    mut pressed: ResMut<Pressed>,
    me: Query<Entity, With<Player>>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    mut commands: Commands,
) {
    let mut begun = None;
    let mut left = false;
    for entered in entered.read() {
        if entered.game == Minigame::KnockOff {
            begun = Some(*entered);
        } else {
            left = true;
        }
    }
    for gone in gone.read() {
        if gone.game == Minigame::KnockOff {
            left = true;
            begun = None;
        }
    }
    if !left && begun.is_none() {
        return;
    }
    for &body in &knock.marked {
        if let Ok(mut body) = commands.get_entity(body) {
            body.try_remove::<(Frozen, Dead, Landing)>();
        }
    }
    *pressed = Pressed::default();
    let watching = knock.watching && begun.is_some_and(|entered| entered.same_table && knock.seat == Some(entered.seat));
    *knock = match begun {
        Some(entered) => {
            info!("Knock Off: in seat {}{}", entered.seat, if entered.online { ", online" } else { "" });
            Knock {
                watching,
                ..Knock::sit(entered.seat, entered.online)
            }
        }
        None => Knock::default(),
    };
    // Offline, on the island thrown here: out on your seat's side of it.
    // (Online the server stands you on its own.)
    if let (Some(seat), Some(floe), false) = (knock.seat, &knock.floe, knock.online)
        && let Ok(you) = me.single()
    {
        let (at, turn) = rules::waiting_spot(usize::from(seat), floe);
        lobby::put(&mut bodies, &mut orbit, you, Venue::KnockOff, Transform::from_translation(at).with_rotation(turn));
        info!("Knock Off: on an island of its own, {} m at its furthest", floe.furthest().round());
    }
}

/// The island the table plays on: online, as the server sends it on the
/// table's entity (`rules::Island`), from the moment the table opens; offline
/// the one this device threw as you sat down ([`Knock::sit`]). Until it is
/// here, your body is held where the server stood you (`Frozen`, `placed`):
/// the ice under it is the island's, which is not yet shaped.
fn find_the_island(
    view: Res<TableView>,
    sent: Query<&Island, With<Remote>>,
    mut knock: ResMut<Knock>,
    me: Query<(Entity, Has<Frozen>), With<Player>>,
    mut commands: Commands,
) {
    if knock.seat.is_none() {
        return;
    }
    if knock.online
        && let Some(island) = view.shown.and_then(|shown| sent.get(shown).ok())
        && knock.island != Some(island.seed)
    {
        info!("Knock Off: the table's island, {:#x}", island.seed);
        knock.island = Some(island.seed);
        knock.floe = Some(Arc::new(island.floe()));
    }
    let Ok((you, frozen)) = me.single() else {
        return;
    };
    let waiting = knock.floe.is_none() && knock.playing().is_none();
    if waiting && !frozen {
        commands.entity(you).insert(Frozen { placed: true });
        knock.held = true;
        if !knock.marked.contains(&you) {
            knock.marked.push(you);
        }
    } else if !waiting && knock.held {
        knock.held = false;
        if knock.playing().is_none() {
            commands.entity(you).remove::<Frozen>();
        }
    }
}

/// What the server says to this device alone: its own aim, as the game
/// holds it, coming back to the game while everyone aims (`Tell::Aimed`);
/// and whether you had said you were ready by then, the round says
/// (`take_aim`).
fn hear(mut told: MessageReader<Told>, mut knock: ResMut<Knock>) {
    for Told(said) in told.read() {
        match said {
            rules::Tell::Aimed { x, z, power } => {
                // Told as the device comes back, before the round it is
                // for has come: the turn being aimed now, whichever.
                let turn = knock.playing().filter(|round| round.stage == KnockStage::Aim).map_or(THIS_TURN, |round| round.turn);
                let way = Vec2::new(*x, *z).normalize_or_zero();
                knock.aim = Aim {
                    turn,
                    way,
                    power: (*power).min(rules::MOST_POWER),
                    ready: false,
                    sent: Some((way, *power)),
                    since_sent: 0.0,
                };
            }
        }
    }
}

/// Offline, the game this device plays: begun as the table's play is (the
/// countdown over, everyone stood on their spot), the computer in the seats
/// it took; stepped by the town's clock, which is this device's own time of
/// day offline, as the server steps it by its own online; its round drawn as
/// the server's would be; and once one is left, the table told how
/// everyone came (`table::Finish`).
#[allow(clippy::too_many_arguments)]
fn play_offline(
    clock: Res<TownClock>,
    view: Res<TableView>,
    mut knock: ResMut<Knock>,
    me: Query<&Transform, With<Player>>,
    computers: Query<(&ComputerSeat, &Transform), Without<Player>>,
    mut finish: MessageWriter<table::Finish>,
    mut asks: MessageWriter<table::Ask>,
) {
    if knock.online || knock.seat.is_none() {
        return;
    }
    // Out, and going: once the results the rest of the game came to are
    // up, and your place paid, to the town.
    if knock.leaving && view.over() {
        knock.leaving = false;
        asks.write(table::Ask::Leave);
        return;
    }
    if knock.game.is_none() {
        if !view.playing() {
            return;
        }
        let Some(floe) = knock.floe.clone() else {
            return;
        };
        // Everyone begins where they stand: you, and the computer's players
        // where they strolled to.
        let seats = view.members.len().max(1);
        let mut standing: Vec<Option<Vec2>> = vec![None; seats];
        if let (Some(seat), Ok(you)) = (knock.seat, me.single())
            && let Some(mine) = standing.get_mut(usize::from(seat))
        {
            *mine = Some(you.translation.xz());
        }
        for (seat, at) in &computers {
            if let Some(theirs) = standing.get_mut(usize::from(seat.0)) {
                *theirs = Some(at.translation.xz());
            }
        }
        let people: Vec<bool> = view.members.iter().map(|member| member.id != 0 && !member.ai).collect();
        let mut game = Game::new(seats, &people, fastrand::u64(..), floe);
        game.begin(clock.now(), &standing);
        knock.loiter = None;
        knock.game = Some(game);
    }
    let Knock {
        game: Some(game),
        seat: Some(seat),
        aim,
        leaving,
        ..
    } = &mut *knock
    else {
        return;
    };
    // Your aim, as it stands, and your word that you are ready: heard only
    // while everyone aims, as the server hears them.
    if game.stage() == KnockStage::Aim && game.turn() == aim.turn {
        game.aim(usize::from(*seat), aim.way, aim.power);
        if aim.ready {
            game.ready(usize::from(*seat));
        }
    }
    // Out, and going (offline): the rest of the game played out at once,
    // a day on by its clock, which it passes through in one step, for how
    // it came out.
    let now = if *leaving && !game.in_play_at(usize::from(*seat)) { clock.now() + 86_400.0 } else { clock.now() };
    game.step(now);
    let round = game.round();
    let outcome = game.outcome();
    if knock.round.as_ref() != Some(&round) {
        knock.round = Some(round);
    }
    if let Some((places, scores, total)) = outcome
        && !knock.finished
    {
        knock.finished = true;
        finish.write(table::Finish { places, scores, total });
    }
}

/// Online, the round as the server sends it, on its table's entity.
fn follow(view: Res<TableView>, rounds: Query<&KnockRound, With<Remote>>, mut knock: ResMut<Knock>) {
    if !knock.online || knock.seat.is_none() {
        return;
    }
    let sent = view.shown.and_then(|shown| rounds.get(shown).ok());
    if let Some(sent) = sent
        && knock.round.as_ref() != Some(sent)
    {
        knock.round = Some(sent.clone());
    }
}

/// Offline, the computer's players standing about the ice while the table
/// waits for its countdown to be over: each, as it comes out onto the ice,
/// strolling now and then somewhere else on it, as the server's do online
/// (`knock_off::Loiter`, one piece for both).
#[allow(clippy::type_complexity)]
fn loiter_offline(
    time: Res<Time>,
    view: Res<TableView>,
    mut knock: ResMut<Knock>,
    me: Query<&Transform, (With<Player>, Without<ComputerSeat>)>,
    mut computers: Query<(&ComputerSeat, &mut Transform, &Venue), Without<Player>>,
) {
    if knock.online || knock.seat.is_none() || knock.game.is_some() || view.playing() || view.over() {
        return;
    }
    let Some(floe) = knock.floe.clone() else {
        return;
    };
    let loiter = knock.loiter.get_or_insert_with(|| Loiter::new(fastrand::u64(..)));
    let mut everyone: Vec<Option<Vec2>> = vec![None; 4];
    if let Ok(me) = me.single() {
        everyone[0] = Some(me.translation.xz());
    }
    for (seat, at, venue) in &computers {
        if *venue == Venue::KnockOff && usize::from(seat.0) < everyone.len() {
            everyone[usize::from(seat.0)] = Some(at.translation.xz());
        }
    }
    for (seat, mut at, venue) in &mut computers {
        let seat = usize::from(seat.0);
        if *venue != Venue::KnockOff {
            continue;
        }
        if !loiter.has(seat) {
            // Out onto the island on its seat's side of it, clear of
            // everyone already there.
            let (spot, turn) = rules::waiting_spot(seat, &floe);
            let others: Vec<Vec2> = everyone.iter().enumerate().filter(|&(other, _)| other != seat).filter_map(|(_, at)| *at).collect();
            let spot = floe.free_spot(spot.xz(), &others, 1.0, STOOD_BACK, ROOM);
            at.set_if_neq(Transform::from_xyz(spot.x, 0.0, spot.y).with_rotation(turn));
            everyone[seat] = Some(spot);
            loiter.enter(seat, spot, turn.to_euler(EulerRot::YXZ).0);
        }
        if let Some((spot, yaw)) = loiter.step(seat, &everyone, &floe, time.delta_secs()) {
            let to = Transform::from_xyz(spot.x, 0.0, spot.y).with_rotation(Quat::from_rotation_y(yaw));
            at.set_if_neq(to);
        }
    }
}

/// Before the game, someone who has walked off the ice into the sea is
/// stood back on it, the nearest spot to where they went in, clear of
/// everyone (since 2026-11-27; on their spot until then): nobody jumps in
/// Knock Off, and the ice's rim is higher than a step out of the water.
/// Offline here; online the server is asked to, as Bridge Wars' fall is
/// (`Ask::Respawn`), once a second at the most, and stands you where it has
/// you last. In the sea as the game begins, you begin on the ice there too
/// (`rules::settle`).
#[allow(clippy::too_many_arguments)]
fn fell_in(
    time: Res<Time>,
    knock: Res<Knock>,
    view: Res<TableView>,
    online: Res<Online>,
    me: Query<(Entity, Has<Frozen>), With<Player>>,
    remotes: Query<(Entity, &Member), With<Remote>>,
    computers: Query<(Entity, &ComputerSeat)>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    mut asks: MessageWriter<NetAsk>,
    mut since: Local<f32>,
) {
    *since += time.delta_secs();
    let (Some(_), Some(floe)) = (knock.seat, &knock.floe) else {
        return;
    };
    let Ok((you, frozen)) = me.single() else {
        return;
    };
    let Ok((_, at, venue, ..)) = bodies.get(you) else {
        return;
    };
    if *venue != Venue::KnockOff || frozen || knock.playing().is_some() || view.playing() || view.over() {
        return;
    }
    let (here, turn) = (at.translation.xz(), at.rotation);
    let off = at.translation.y < -FELL_IN || floe.depth(here, 1.0) < -FELL_IN;
    if !off || *since < 1.0 {
        return;
    }
    *since = 0.0;
    if knock.online && online.is() {
        asks.write(NetAsk::Respawn);
    } else if !knock.online {
        let others: Vec<Vec2> = bodies_by_seat(&knock, &view, Some(you), &remotes, &computers)
            .into_iter()
            .flatten()
            .filter(|&body| body != you)
            .filter_map(|body| bodies.get(body).ok().map(|(_, at, ..)| at.translation.xz()))
            .collect();
        let back = floe.free_spot(here, &others, 1.0, STOOD_BACK, ROOM);
        lobby::put(&mut bodies, &mut orbit, you, Venue::KnockOff, Transform::from_xyz(back.x, 0.0, back.y).with_rotation(turn));
    }
}

/// While the table waits, nobody walks through anybody (Hajun, 2026-11-27:
/// "collision matters during waiting period too so u cant start on top of
/// each other"): your body, once it has moved, is put back out of everyone
/// else's at the table, as far from each as two bodies reach in a slide
/// ([`ROOM`]), so that wherever you stop is somewhere you can begin. Theirs
/// are where this device draws them; their own devices, and the computer's
/// players, keep out of yours (`rules::Loiter`). Not once the game is
/// played, when the game places everyone.
fn keep_apart(
    knock: Res<Knock>,
    view: Res<TableView>,
    me: Query<Entity, With<Player>>,
    remotes: Query<(Entity, &Member), With<Remote>>,
    computers: Query<(Entity, &ComputerSeat)>,
    mut bodies: Query<(&mut Transform, &Venue, Has<Frozen>, Has<Dead>)>,
) {
    if knock.seat.is_none() || knock.playing().is_some() || view.playing() || view.over() {
        return;
    }
    let Ok(you) = me.single() else {
        return;
    };
    let others: Vec<Vec2> = bodies_by_seat(&knock, &view, Some(you), &remotes, &computers)
        .into_iter()
        .flatten()
        .filter(|&body| body != you)
        .filter_map(|body| bodies.get(body).ok())
        .filter(|(_, venue, _, dead)| **venue == Venue::KnockOff && !dead)
        .map(|(at, ..)| at.translation.xz())
        .collect();
    let Ok((mut at, venue, frozen, _)) = bodies.get_mut(you) else {
        return;
    };
    if *venue != Venue::KnockOff || frozen {
        return;
    }
    let mut here = at.translation.xz();
    for _ in 0..3 {
        for &other in &others {
            let apart = here - other;
            if apart.length_squared() < ROOM * ROOM {
                here = other + apart.normalize_or(Vec2::X) * ROOM;
            }
        }
    }
    if here != at.translation.xz() {
        at.translation.x = here.x;
        at.translation.z = here.y;
    }
}

/// How many steps a power arrow, or its key, held `held` seconds has
/// repeated by: none for the first [`REPEAT_AFTER`], then one every
/// [`REPEAT_EVERY`], however few frames that has been in.
fn repeats(held: f32) -> u32 {
    if held < REPEAT_AFTER {
        0
    } else {
        ((held - REPEAT_AFTER) / REPEAT_EVERY).floor() as u32 + 1
    }
}

/// Your aim, while you aim: which way from the stick (or, on desktop, W, A,
/// S and D or the arrow keys, eight ways), turned as the camera looks, its
/// up away from the camera, and kept as it was once the stick is let go;
/// how hard from the arrows on the right (`screen`), or Q and E, Page Up and
/// Down and the mouse's wheel; and Ready from its button, or Space. A new
/// turn begins at [`START_POWER`] with no way yet: standing still is what
/// not aiming is, and pushing the stick is all it takes to aim (until
/// 2026-11-18 a turn began at no power, and the stick alone did nothing).
#[allow(clippy::too_many_arguments)]
fn take_aim(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<TouchControls>,
    mut wheel: MessageReader<MouseWheel>,
    typing: Res<crate::signup::NameBox>,
    cameras: Query<&Transform, With<ThirdPersonCamera>>,
    mut pressed: ResMut<Pressed>,
    mut knock: ResMut<Knock>,
    mut keys_held: Local<Option<(bool, f32, u32)>>,
) {
    // A line being typed into the chat has every key: none of them aims.
    let keys = if typing.up() { &ButtonInput::<KeyCode>::default() } else { &*keys };
    let turn = knock.playing().map(|round| (round.turn, round.stage));
    // A new turn: no aim yet; but an aim the server has said is yours,
    // coming back to the game, is this turn's (`hear`).
    if let Some((turn, KnockStage::Aim)) = turn
        && knock.aim.turn != turn
    {
        if knock.aim.turn == THIS_TURN {
            knock.aim.turn = turn;
        } else {
            knock.aim = Aim {
                turn,
                power: START_POWER,
                ..default()
            };
        }
    }
    ready_as_held(&mut knock);
    let scrolled: f32 = wheel
        .read()
        .map(|scroll| match scroll.unit {
            MouseScrollUnit::Line => scroll.y,
            MouseScrollUnit::Pixel => scroll.y / 40.0,
        })
        .sum();
    let mut steps = std::mem::take(&mut pressed.power);
    let ready = std::mem::take(&mut pressed.ready) || keys.just_pressed(KeyCode::Space);
    // The power arrows held down: one step as they go down (`screen`), and
    // held, a step every so often after a moment.
    if let Some(holding) = &mut pressed.held {
        holding.held += time.delta_secs();
        let due = repeats(holding.held);
        if due > holding.repeated {
            let more = (due - holding.repeated) as i32;
            holding.repeated = due;
            steps += if holding.up { more } else { -more };
        }
    }
    // The keys, likewise.
    let up = keys.pressed(KeyCode::KeyE) || keys.pressed(KeyCode::PageUp);
    let down = keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::PageDown);
    if keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::PageUp) {
        steps += 1;
        *keys_held = Some((true, 0.0, 0));
    } else if keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::PageDown) {
        steps -= 1;
        *keys_held = Some((false, 0.0, 0));
    }
    match keys_held.as_mut() {
        Some((going_up, held, repeated)) if (*going_up && up) || (!*going_up && down) => {
            *held += time.delta_secs();
            let due = repeats(*held);
            if due > *repeated {
                let more = (due - *repeated) as i32;
                *repeated = due;
                steps += if *going_up { more } else { -more };
            }
        }
        _ => *keys_held = None,
    }
    if scrolled.abs() >= 0.5 {
        steps += scrolled.round() as i32;
    }
    if !knock.aiming() {
        return;
    }
    // Which way: the stick, or the keys, as the camera looks.
    let ahead = cameras.single().map_or(Vec3::NEG_Z, crate::camera_ahead);
    let (forward, right) = (ahead.xz(), ahead.cross(Vec3::Y).xz());
    let stick = controls.stick_value;
    let mut way = if stick.length() >= crate::STICK_DEADZONE {
        right * stick.x + forward * stick.y
    } else {
        Vec2::ZERO
    };
    if way == Vec2::ZERO {
        for (codes, toward) in [
            ([KeyCode::KeyW, KeyCode::ArrowUp], forward),
            ([KeyCode::KeyS, KeyCode::ArrowDown], -forward),
            ([KeyCode::KeyA, KeyCode::ArrowLeft], -right),
            ([KeyCode::KeyD, KeyCode::ArrowRight], right),
        ] {
            if codes.iter().any(|&code| keys.pressed(code)) {
                way += toward;
            }
        }
    }
    let mut aim = knock.aim;
    if let Some(way) = way.try_normalize() {
        aim.way = way;
    }
    let power = (i32::from(aim.power) + steps).clamp(0, i32::from(rules::MOST_POWER));
    aim.power = power as u8;
    // Some power set by hand and no way yet: across the ice, toward its
    // middle. Not the power a turn begins at, which would send whoever does
    // nothing off across the ice.
    if steps != 0 && aim.power > 0 && aim.way == Vec2::ZERO {
        let seat = knock.seat.map_or(0, usize::from);
        let from = knock
            .playing()
            .and_then(|round| round.start.get(seat).copied().flatten())
            .map_or(rules::side(seat) * 5.0, |(x, z)| Vec2::new(x, z));
        let middle = knock.floe.as_ref().map_or(Vec2::ZERO, |floe| floe.middle(knock.size()));
        aim.way = (middle - from).normalize_or(forward);
    }
    if ready {
        aim.ready = true;
    }
    if aim != knock.aim {
        knock.aim = aim;
    }
}

/// Ready as the game holds you to it: said on this device before it came
/// back to the game, or before the game was begun afresh here, the round
/// says you are, and the game hears no more of your aim this turn (it
/// keeps what it had, which it has told this device, `Tell::Aimed`): so
/// nothing more is sent, and the arrow, the arrows on the right and Ready
/// are as they are once you have pressed it, not live under a tick that
/// says otherwise.
fn ready_as_held(knock: &mut Knock) {
    let held = knock.seat.zip(knock.playing()).is_some_and(|(seat, round)| {
        round.stage == KnockStage::Aim && round.turn == knock.aim.turn && round.ready.get(usize::from(seat)).copied().unwrap_or(false)
    });
    if held && !knock.aim.ready {
        let aim = &mut knock.aim;
        aim.ready = true;
        aim.sent = Some((aim.way, aim.power));
        aim.since_sent = f32::NEG_INFINITY;
    }
}

/// Online, your aim to the server as it changes, no oftener than
/// [`SEND_EVERY`] but always the last of it, and your word that you are
/// ready after it. (Offline the game here takes it as it is, `play_offline`.)
fn send_aim(time: Res<Time>, online: Res<Online>, mut knock: ResMut<Knock>, mut asks: MessageWriter<NetAsk>) {
    if !knock.online || !online.is() {
        return;
    }
    let Some(round) = knock.playing() else {
        return;
    };
    if round.stage != KnockStage::Aim || knock.aim.turn != round.turn || !knock.in_play() {
        return;
    }
    let aim = &mut knock.aim;
    aim.since_sent += time.delta_secs();
    let now = (aim.way, aim.power);
    let changed = aim.sent != Some(now);
    if changed && (aim.since_sent >= SEND_EVERY || aim.ready) {
        asks.write(NetAsk::KnockOff(rules::Ask::Aim {
            x: aim.way.x,
            z: aim.way.y,
            power: aim.power,
        }));
        aim.sent = Some(now);
        aim.since_sent = 0.0;
    }
    if aim.ready && aim.sent == Some(now) && aim.since_sent >= 0.0 {
        asks.write(NetAsk::KnockOff(rules::Ask::Ready));
        // Said once: nothing more is heard from you this turn.
        aim.since_sent = f32::NEG_INFINITY;
    }
}

/// The camera being brought from one view to another ([`hold_the_camera`]):
/// for whom and what shape of screen, what it is fitted to
/// ([`camera_target`]) and how much is kept clear over it, from where to
/// where, and how far it has come, 0 to 1.
#[derive(Default)]
struct Eased {
    seat: Option<u8>,
    shape: u32,
    island: Option<u64>,
    target: Option<Fit>,
    top: u32,
    from: Transform,
    to: Transform,
    done: f32,
}

impl Eased {
    /// Where it is now: eased in and out from one to the other. The two
    /// look the same way (from the same side, as steeply), so that it comes
    /// straight in.
    fn now(&self) -> Transform {
        let share = self.done.clamp(0.0, 1.0);
        let share = share * share * (3.0 - 2.0 * share);
        Transform {
            translation: self.from.translation.lerp(self.to.translation, share),
            rotation: self.from.rotation.slerp(self.to.rotation, share),
            scale: Vec3::ONE,
        }
    }
}

/// The camera over the ice from your seat's side ([`view_of`]), held there
/// by the game from the moment you sit down to the end: still while anyone
/// aims or slides, brought in over [`CAMERA_EASE`] to what is left of the
/// ice as it breaks, and to whoever won at the end ([`camera_target`]); and
/// shaken where two meet in a slide, and pushed in a little where they meet
/// hard (mine: a check of the game found the few centimetres it had shaken
/// by, from nearly 30 m off, were not a pixel on a phone). By the time that
/// passes (`Time<Real>`), not the engine's time of the frames, which it
/// holds to a quarter of a second each, so that a device that slows still
/// has the camera in before the next aim.
fn hold_the_camera(
    time: Res<Time<Real>>,
    at: Res<AtTable>,
    windows: Query<&Window>,
    mut knock: ResMut<Knock>,
    mut held: ResMut<HeldCamera>,
    mut eased: Local<Eased>,
) {
    let Some(sitting) = at.of(Minigame::KnockOff) else {
        if eased.seat.is_some() {
            *eased = Eased::default();
        }
        return;
    };
    let aspect = windows.single().map_or(16.0 / 9.0, |window| window.width() / window.height().max(1.0));
    let target = camera_target(&knock);
    // More kept clear at the top while the table's board is up, before play.
    let top = if knock.playing().is_some() { CLEAR_TOP } else { CLEAR_TOP_WAITING };
    // Fitted again only for another seat, another shape of screen, another
    // island or something else to fit: at once for the first three, eased
    // for the last.
    let fresh = eased.seat != Some(sitting.seat) || eased.shape != aspect.to_bits() || eased.island != knock.island;
    if fresh || eased.target != Some(target) || eased.top != top.to_bits() {
        let to = view_of(sitting.seat, aspect, &fit_points(knock.floe.as_deref(), target), top);
        let from = if fresh { to } else { eased.now() };
        *eased = Eased {
            seat: Some(sitting.seat),
            shape: aspect.to_bits(),
            island: knock.island,
            target: Some(target),
            top: top.to_bits(),
            from,
            to,
            done: if fresh { 1.0 } else { 0.0 },
        };
    }
    if eased.done < 1.0 {
        eased.done = (eased.done + time.delta_secs() / CAMERA_EASE).min(1.0);
    }
    let mut view = eased.now();
    if let Some((left, share)) = &mut knock.shake {
        *left -= time.delta_secs();
        if *left <= 0.0 {
            knock.shake = None;
        } else {
            // As a share of how far off the ice the camera is, so that it is
            // as much on the screen wherever the camera has come in to.
            let fade = *left / ice::SHAKE_SECS;
            let off = view.translation.y / CAMERA_PITCH.sin();
            let clock = time.elapsed_secs();
            let wobble = *share * off * fade * fade;
            let (right, up, ahead) = (view.right(), view.up(), view.forward());
            view.translation += right * (clock * 73.0).sin() * wobble
                + up * (clock * 57.0 + 1.1).sin() * wobble * 0.6
                + ahead * *share * KICK * off * fade * fade * fade;
        }
    }
    held.set_if_neq(HeldCamera(Some(view)));
}

/// When the slide `round` is for begins by the town's clock, once the clock
/// `now` has come to it: as its own round says; or, its own not here yet,
/// as the reveal's says, whose end it begins at (`until`), from what the
/// reveal already shows everyone (where each stands, how each sets off).
/// The slide's round comes a server's tick and the network after that, and
/// a slide begun only then had everyone jump forward as it began.
fn slide_begins(round: &KnockRound, now: f64) -> Option<f64> {
    match round.stage {
        KnockStage::Reveal if now >= round.until && !round.launches.is_empty() => Some(round.until),
        KnockStage::Slide => Some(round.since),
        _ => None,
    }
}

/// Where the body in `seat` is drawn, and how: on the ice, facing `yaw`,
/// posed for sliding or not, or cheering, having won; or nowhere, gone
/// into the sea.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Pose {
    On { at: Vec3, yaw: f32, sliding: bool, cheer: bool },
    Gone,
}

/// Where the body in `seat` is drawn by `round`, `clock` by the town's
/// clock, with the slide this device plays (`played`); facing `face` unless
/// the stage turns it, and settling from `settle_from` onto where the
/// server has it as the ice breaks. Everyone faces the middle while they
/// aim (never toward their aim, which would give it away), and the way
/// they will set off once it is shown; sliding, the way they slide (the
/// spec: "turned to face the way it slides"), which a hit may turn
/// (`place` swings them round to it), and once they slow, as they last
/// faced. The reveal's round with its slide begun here is drawn as the
/// slide.
#[allow(clippy::too_many_arguments)]
fn pose(round: &KnockRound, played: Option<&Played>, seat: usize, clock: f64, face: f32, settle_from: Option<Vec3>, middle: Vec2) -> Pose {
    let start = round.start.get(seat).copied().flatten().map(|(x, z)| Vec2::new(x, z));
    let out = round.out.get(seat).copied().flatten().is_some();
    let since = (clock - round.since).max(0.0) as f32;
    let on = |at: Vec2, yaw: f32| Pose::On {
        at: Vec3::new(at.x, 0.0, at.y),
        yaw,
        sliding: false,
        cheer: false,
    };
    let played = played.filter(|played| played.turn == round.turn);
    let stage = match round.stage {
        KnockStage::Reveal if played.is_some() => KnockStage::Slide,
        stage => stage,
    };
    match stage {
        KnockStage::Aim => match start {
            Some(at) if !out => on(at, facing(middle - at).unwrap_or(face)),
            _ => Pose::Gone,
        },
        KnockStage::Reveal => match start {
            Some(at) if !out => {
                let launch = round.launches.get(seat).map_or(Vec2::ZERO, |&(x, z)| Vec2::new(x, z));
                on(at, facing(launch).or(facing(middle - at)).unwrap_or(face))
            }
            _ => Pose::Gone,
        },
        KnockStage::Slide => {
            let slid = played.and_then(|played| Some((played.slide.at(seat)?, played.slide.velocity(seat).unwrap_or(Vec2::ZERO))));
            match slid {
                Some((at, _)) if at.y <= -rules::SUNK + 1e-3 => Pose::Gone,
                Some((at, going)) => {
                    let sliding = going.length() > SLIDING_FROM && at.y > -0.05;
                    let yaw = if sliding { facing(going).unwrap_or(face) } else { face };
                    Pose::On { at, yaw, sliding, cheer: false }
                }
                None => match start {
                    Some(at) if !out => on(at, face),
                    _ => Pose::Gone,
                },
            }
        }
        KnockStage::Break => match start {
            // Taken by what broke off: down with it into the sea.
            Some(at) if out => {
                let drop = (0.5 * DROP * since * since).min(rules::SUNK);
                if drop >= rules::SUNK - 1e-3 {
                    Pose::Gone
                } else {
                    Pose::On {
                        at: Vec3::new(at.x, -drop, at.y),
                        yaw: face,
                        sliding: false,
                        cheer: false,
                    }
                }
            }
            // Settled where the server has the slide leave them.
            Some(at) => {
                let there = Vec3::new(at.x, 0.0, at.y);
                let at = match settle_from {
                    Some(from) if from.distance(there) > 0.01 => from.lerp(there, (since / SETTLE).clamp(0.0, 1.0)),
                    _ => there,
                };
                Pose::On {
                    at,
                    yaw: face,
                    sliding: false,
                    cheer: false,
                }
            }
            None => Pose::Gone,
        },
        // Whoever won, alone on the ice, cheers (`place` has them hop).
        KnockStage::Over => match start {
            Some(at) if !out => Pose::On {
                at: Vec3::new(at.x, 0.0, at.y),
                yaw: face,
                sliding: false,
                cheer: winner_on_the_ice(round).is_some_and(|(won, _)| won == seat),
            },
            _ => Pose::Gone,
        },
    }
}

/// Every body at the table placed by the game while it is played: held
/// where the game puts it (`Frozen`, `placed`), on its spot, in the slide
/// this device plays from the round, settled as the ice breaks, and hidden
/// once in the sea (`Dead`, which `island` draws nowhere); posed for
/// sliding while it slides, swung round to the way it slides. Each written
/// only when it changes. The slide is begun here the moment it begins by
/// the town's clock ([`slide_begins`]), and stepped to it.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn place(
    time: Res<Time>,
    clock: Res<TownClock>,
    view: Res<TableView>,
    mut knock: ResMut<Knock>,
    me: Query<Entity, With<Player>>,
    remotes: Query<(Entity, &Member), With<Remote>>,
    computers: Query<(Entity, &ComputerSeat)>,
    mut bodies: Query<(&mut Transform, Has<Frozen>, Option<&Landing>, Has<Dead>)>,
    mut splashes: MessageWriter<ice::Splashed>,
    mut met: MessageWriter<ice::Met>,
    mut commands: Commands,
) {
    let Some(round) = knock.playing().cloned() else {
        return;
    };
    let Some(floe) = knock.floe.clone() else {
        return;
    };
    let middle = floe.middle(round.size);
    let now = clock.now();
    let seats = round.start.len();
    if let Some(since) = slide_begins(&round, now)
        && knock.slide.as_ref().is_none_or(|played| !played.is(&round, since))
    {
        knock.slide = Some(Played::of(&round, since, &floe));
    }
    if let Some(played) = &mut knock.slide
        && played.turn == round.turn
    {
        played.slide.step_to((now - played.since) as f32);
    }
    if knock.faces.len() != seats {
        knock.faces = (0..seats).map(|seat| facing(-rules::side(seat)).unwrap_or(0.0)).collect();
        knock.drawn = vec![None; seats];
        knock.settle_from = vec![None; seats];
        knock.splashed = vec![false; seats];
    }
    // A new stage: who faces which way now, and where each settles from.
    let stage = (round.turn, round.stage);
    if knock.placed_in != Some(stage) {
        if round.stage == KnockStage::Aim {
            knock.splashed = vec![false; seats];
        }
        for seat in 0..seats {
            let start = round.start[seat].map(|(x, z)| Vec2::new(x, z));
            match round.stage {
                KnockStage::Aim => {
                    if let Some(yaw) = start.and_then(|at| facing(middle - at)) {
                        knock.faces[seat] = yaw;
                    }
                }
                KnockStage::Reveal => {
                    let launch = round.launches.get(seat).map_or(Vec2::ZERO, |&(x, z)| Vec2::new(x, z));
                    if let Some(yaw) = facing(launch) {
                        knock.faces[seat] = yaw;
                    }
                }
                _ => {}
            }
            knock.settle_from[seat] = if round.stage == KnockStage::Break { knock.drawn[seat] } else { None };
        }
        knock.placed_in = Some(stage);
    }
    let by_seat = bodies_by_seat(&knock, &view, me.single().ok(), &remotes, &computers);
    for seat in 0..seats {
        let Some(body) = by_seat.get(seat).copied().flatten() else {
            continue;
        };
        let Ok((mut transform, frozen, landing, dead)) = bodies.get_mut(body) else {
            continue;
        };
        if !knock.marked.contains(&body) {
            knock.marked.push(body);
        }
        if !frozen {
            commands.entity(body).insert(Frozen { placed: true });
        }
        let pose = pose(&round, knock.slide.as_ref(), seat, now, knock.faces[seat], knock.settle_from[seat], middle);
        match pose {
            Pose::On { at, yaw, sliding, cheer } => {
                // Sliding, swung round to the way it goes, not snapped, and
                // facing so as it slows; having won, swung round to face the
                // camera.
                let yaw = if sliding {
                    let swung = turned(knock.faces[seat], yaw, SLIDING_TURN * time.delta_secs());
                    knock.faces[seat] = swung;
                    swung
                } else if cheer {
                    let camera = rules::side(usize::from(knock.seat.unwrap_or(0))) * 60.0;
                    let toward = facing(camera - at.xz()).unwrap_or(yaw);
                    let swung = turned(knock.faces[seat], toward, SLIDING_TURN * 0.5 * time.delta_secs());
                    knock.faces[seat] = swung;
                    swung
                } else {
                    yaw
                };
                let pose_wanted = if sliding {
                    Some(Landing {
                        squat: SLIDING_SQUAT,
                        arms: SLIDING_ARMS,
                    })
                } else if cheer {
                    Some(Landing { squat: 0.0, arms: 1.0 })
                } else {
                    None
                };
                if landing != pose_wanted.as_ref() {
                    match pose_wanted {
                        Some(wanted) => commands.entity(body).insert(wanted),
                        None => commands.entity(body).remove::<Landing>(),
                    };
                }
                let sink = pose_wanted.map_or(0.0, |pose| pose.sink(HIPS));
                // Having won, hopping, arms up.
                let won_for = (now - round.since) as f32 - HOP_FROM;
                let hop = if cheer && won_for > 0.0 { HOP * (won_for * PI / HOP_SECS).sin().abs() } else { 0.0 };
                let wanted = Transform::from_translation(at - Vec3::Y * sink + Vec3::Y * hop).with_rotation(Quat::from_rotation_y(yaw));
                transform.set_if_neq(wanted);
                if dead {
                    commands.entity(body).remove::<Dead>();
                }
                // Into the sea, the moment it reaches the water: a splash.
                let was = knock.drawn[seat];
                if at.y < crate::WATER_Y && was.is_some_and(|was| was.y >= crate::WATER_Y) && !knock.splashed[seat] {
                    knock.splashed[seat] = true;
                    splashes.write(ice::Splashed { at: Vec3::new(at.x, crate::WATER_Y, at.z), big: true });
                }
                knock.drawn[seat] = Some(at);
            }
            Pose::Gone => {
                if !dead {
                    commands.entity(body).insert(Dead { left: f32::INFINITY });
                }
                if landing.is_some() {
                    commands.entity(body).remove::<Landing>();
                }
                knock.drawn[seat] = None;
            }
        }
    }
    // Two meeting in the slide: the dust where they met, and a shake of the
    // camera with it, by how hard they met.
    let mut shaken = None;
    if let Some(played) = &mut knock.slide
        && played.turn == round.turn
    {
        let hits = played.slide.hits();
        for hit in &hits[played.hits_shown.min(hits.len())..] {
            // (Coming back in the middle of one, only those just now.)
            if played.slide.time() - hit.time < 0.4 {
                met.write(ice::Met {
                    at: Vec3::new(hit.at.x, 0.0, hit.at.y),
                    speed: hit.speed,
                    seats: hit.seats,
                });
                shaken = Some(shaken.unwrap_or(0.0f32).max(hit.speed));
            }
        }
        played.hits_shown = hits.len();
    }
    if let Some(speed) = shaken {
        knock.shake = Some((ice::SHAKE_SECS, ice::shake_for(speed)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The round island 12 m out the game was played on until 2026-11-27,
    /// each seat's spot 7 m out on its side (`rules::arrival`).
    fn round_island() -> Arc<Floe> {
        Arc::new(Floe::round(rules::RADIUS))
    }

    /// A round of four seats on the spots they began on, on the round
    /// island.
    fn round(stage: KnockStage) -> KnockRound {
        KnockRound {
            turn: 1,
            stage,
            since: 100.0,
            until: 110.0,
            size: 1.0,
            next_size: rules::size_after(1),
            start: (0..4).map(|seat| Some((rules::arrival(seat).0.x, rules::arrival(seat).0.z))).collect(),
            launches: Vec::new(),
            ready: vec![false; 4],
            out: vec![None; 4],
            places: vec![None; 4],
        }
    }

    #[test]
    fn the_camera_sees_the_whole_island_from_your_own_side_your_spot_low_on_the_screen() {
        let floe = Floe::round(rules::RADIUS);
        for aspect in [16.0 / 9.0, 19.5 / 9.0, 4.0 / 3.0] {
            for seat in 0..4u8 {
                let view = over_the_ice(seat, aspect, &floe);
                let side = rules::side(usize::from(seat));
                // From your own side, looking down at 55 degrees along the
                // line from you through the middle.
                assert!(view.translation.xz().normalize().dot(side) > 0.999, "{seat}: {view:?}");
                let looking = view.forward();
                assert!((looking.y.asin().abs() - CAMERA_PITCH).abs() < 1e-3, "{looking}");
                assert!(looking.xz().normalize().dot(-side) > 0.9999);
                // The stick's up, away from the camera, is toward the middle
                // from your spot.
                let ahead = crate::camera_ahead(&view).xz();
                assert!(ahead.dot(-side) > 0.999);
                // Your spot below the middle of the screen, and the ice's
                // far edge under the top.
                let into = view.to_matrix().inverse();
                let spot = into.transform_point3(rules::arrival(usize::from(seat)).0);
                assert!(spot.y / -spot.z < 0.0, "your spot toward the bottom");
                let half = (crate::CAMERA_FOV * 0.5).tan();
                let far = into.transform_point3((-side * rules::RADIUS).extend(0.0).xzy());
                assert!(far.y / -far.z / half < 1.0 - 2.0 * CLEAR_TOP + 1e-3, "{aspect} {seat}");
                // Not far off: the island fills the screen as it fits.
                assert!(view.translation.length() < 45.0, "{}", view.translation.length());
                // And the near edge over the band kept at the bottom for
                // what is said across the screen.
                let near = into.transform_point3((side * rules::RADIUS).extend(0.0).xzy());
                assert!(near.y / -near.z / half > -1.0 + 2.0 * CLEAR_BOTTOM - 1e-3, "{aspect} {seat}");
                // While the table waits, the far edge, and the far player
                // on it, under the board and its Leave as well.
                let waiting = view_of(seat, aspect, &fit_points(Some(&floe), Fit::Ice(1.0)), CLEAR_TOP_WAITING);
                let into = waiting.to_matrix().inverse();
                let far = into.transform_point3((-side * rules::RADIUS).extend(0.0).xzy());
                assert!(far.y / -far.z / half < 1.0 - 2.0 * CLEAR_TOP_WAITING + 1e-3, "{aspect} {seat}");
                let across = (seat + 2) % 4;
                let them = into.transform_point3(rules::arrival(usize::from(across)).0);
                assert!(them.y / -them.z / half < 1.0 - 2.0 * CLEAR_TOP_WAITING - 0.1, "{aspect} {seat}: the far player");
            }
        }
    }

    #[test]
    fn the_camera_sees_the_whole_of_any_island_between_what_stands_at_the_edges() {
        let half = (crate::CAMERA_FOV * 0.5).tan();
        for seed in 0..30 {
            let floe = Floe::thrown(seed);
            for (aspect, seat) in [(16.0 / 9.0, 0u8), (19.5 / 9.0, 1), (4.0 / 3.0, 2), (19.5 / 9.0, 3)] {
                for (size, top) in [(1.0, CLEAR_TOP_WAITING), (1.0, CLEAR_TOP), (rules::size_in(5), CLEAR_TOP)] {
                    let view = view_of(seat, aspect, &fit_points(Some(&floe), Fit::Ice(size)), top);
                    let into = view.to_matrix().inverse();
                    // From your own side, as steep as ever.
                    assert!(view.forward().xz().normalize().dot(-rules::side(usize::from(seat))) > 0.9999, "seed {seed}");
                    // Every point of the ice's edge on the screen, clear of
                    // the top, the bottom and the sides.
                    let (mut low, mut high, mut wide) = (f32::MAX, f32::MIN, 0.0f32);
                    for at in floe.rim_at(size) {
                        let seen = into.transform_point3(Vec3::new(at.x, 0.0, at.y));
                        let up = seen.y / -seen.z / half;
                        low = low.min(up);
                        high = high.max(up);
                        wide = wide.max((seen.x / -seen.z / (half * aspect)).abs());
                        assert!(up < 1.0 - 2.0 * top + 1e-3 && up > -1.0 + 2.0 * CLEAR_BOTTOM - 1e-3, "seed {seed} seat {seat}: {at} at {up}");
                        assert!(wide <= 1.0 - 2.0 * CLEAR_SIDES + 1e-3, "seed {seed} seat {seat}: {at} {wide} across");
                    }
                    // And filling the screen as it fits: either all but the
                    // whole height kept, or all but the whole width.
                    let (tall, room) = (high - low, (1.0 - 2.0 * top) - (-1.0 + 2.0 * CLEAR_BOTTOM));
                    assert!(tall > 0.6 * room || wide > 0.6 * (1.0 - 2.0 * CLEAR_SIDES), "seed {seed} seat {seat}: {tall} of {room} tall, {wide} wide");
                }
            }
        }
    }

    #[test]
    fn the_camera_comes_in_as_the_ice_breaks_and_to_whoever_won() {
        let aspect = 19.5 / 9.0;
        let off = |view: Transform| view.translation.y / CAMERA_PITCH.sin();
        // The ice that is left, nearer each turn, to the least it fits.
        // On the round island the last turns at least twice as near as the
        // first; on one of blobmaker's, lying across the screen, which is
        // fitted from nearer to begin with, a good deal nearer still.
        for (floe, nearer) in [(Floe::round(rules::RADIUS), 2.0), (Floe::thrown(8), 1.6), (Floe::thrown(9), 1.6)] {
            let mut last = f32::MAX;
            for turn in 1..=rules::LAST_TURN {
                let view = view_of(2, aspect, &fit_points(Some(&floe), Fit::Ice(rules::size_in(turn))), CLEAR_TOP);
                assert!(off(view) <= last + 0.3, "turn {turn}");
                last = last.min(off(view));
            }
            assert!(off(over_the_ice(2, aspect, &floe)) > nearer * last, "the last turns nearer than the first");
        }
        // As the ice breaks, fitted to what will be left; at the end, to
        // whoever won, alone on the ice.
        let mut knock = Knock::sit(2, false);
        let mut round = round(KnockStage::Aim);
        round.turn = 4;
        round.size = rules::size_in(4);
        round.next_size = rules::size_after(4);
        knock.round = Some(round.clone());
        assert_eq!(camera_target(&knock), Fit::Ice(round.size));
        round.stage = KnockStage::Break;
        knock.round = Some(round.clone());
        assert_eq!(camera_target(&knock), Fit::Ice(round.next_size));
        round.stage = KnockStage::Over;
        round.out = vec![Some(2), None, Some(3), Some(3)];
        round.places = vec![Some(3), Some(0), Some(1), Some(1)];
        round.start = vec![None, Some((1.0, -2.0)), None, None];
        knock.round = Some(round.clone());
        assert_eq!(camera_target(&knock), Fit::Round(Vec2::new(1.0, -2.0), WINNER_VIEW));
        let Pose::On { cheer, .. } = pose(&round, None, 1, round.since + 1.0, 0.0, None, Vec2::ZERO) else {
            panic!("on the ice");
        };
        assert!(cheer);
        // With nobody left on the ice, nobody to come in to.
        round.out[1] = Some(3);
        round.start[1] = None;
        round.places = vec![Some(3), Some(0), Some(0), Some(0)];
        knock.round = Some(round.clone());
        assert_eq!(camera_target(&knock), Fit::Ice(round.size));
    }

    #[test]
    fn a_power_arrow_held_repeats_after_a_moment_however_slow_the_frames() {
        assert_eq!(repeats(0.0), 0);
        assert_eq!(repeats(REPEAT_AFTER - 0.01), 0);
        assert_eq!(repeats(REPEAT_AFTER), 1);
        // Held a second: the press and eight or nine steps more, enough to
        // go from nothing to all but the most.
        assert_eq!(repeats(1.0), 9);
        // However it is counted, a frame at a time or in one long one.
        let mut held = 0.0;
        let mut counted = 0;
        for _ in 0..10 {
            held += 0.1;
            counted = counted.max(repeats(held));
        }
        assert_eq!(counted, repeats(1.0));
    }

    #[test]
    fn everyone_faces_the_middle_while_they_aim_and_the_way_they_go_once_shown() {
        let aiming = round(KnockStage::Aim);
        for seat in 0..4 {
            let Pose::On { at, yaw, sliding, .. } = pose(&aiming, None, seat, 101.0, 0.0, None, Vec2::ZERO) else {
                panic!("on the ice");
            };
            assert_eq!(at, rules::arrival(seat).0);
            assert!(!sliding);
            let ahead = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
            assert!(ahead.xz().dot(-at.xz().normalize()) > 0.999, "seat {seat}");
        }
        let mut shown = round(KnockStage::Reveal);
        shown.launches = vec![(0.0, -3.0), (2.0, 0.0), (0.0, 0.0), (0.0, 0.0)];
        let Pose::On { yaw, .. } = pose(&shown, None, 1, 110.5, 0.0, None, Vec2::ZERO) else {
            panic!("on the ice");
        };
        let ahead = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
        assert!(ahead.x > 0.999, "{ahead}");
        // Whoever is out is nowhere.
        shown.out[3] = Some(1);
        assert_eq!(pose(&shown, None, 3, 110.5, 0.0, None, Vec2::ZERO), Pose::Gone);
    }

    #[test]
    fn a_slide_drawn_here_is_the_game_s_and_settles_onto_the_server_s_ending() {
        // The game plays a turn out, everyone sliding at the middle.
        let floe = round_island();
        let mut game = Game::new(4, &[true, true, true, true], 1, Arc::clone(&floe));
        game.begin(1000.0, &[]);
        for seat in 0..4 {
            game.aim(seat, -rules::arrival(seat).0.xz(), 6);
            game.ready(seat);
        }
        game.step(1000.01);
        game.step(1000.01 + f64::from(rules::REVEAL) + 0.001);
        let round = game.round();
        assert_eq!(round.stage, KnockStage::Slide);
        let mut played = Played::of(&round, round.since, &floe);
        // A second in, drawn where the game has them.
        let at = round.since + 1.0;
        game.step(at);
        played.slide.step_to((at - played.since) as f32);
        for seat in 0..4 {
            match pose(&game.round(), Some(&played), seat, at, 0.0, None, Vec2::ZERO) {
                Pose::On { at: drawn, .. } => assert!(drawn.distance(game.at(seat).0) < 1e-5, "seat {seat}"),
                Pose::Gone => assert!(game.at(seat).0.y <= -rules::SUNK + 1e-3),
            }
        }
        // As the ice breaks, from where the slide left them onto where the
        // server says, over a fifth of a second.
        let mut breaking = round.clone();
        breaking.stage = KnockStage::Break;
        breaking.since = 2000.0;
        breaking.start[0] = Some((1.0, 2.0));
        let from = Vec3::new(1.5, 0.0, 2.0);
        let Pose::On { at: first, .. } = pose(&breaking, None, 0, 2000.0, 0.0, Some(from), Vec2::ZERO) else {
            panic!("on the ice");
        };
        assert!(first.distance(from) < 1e-5);
        let Pose::On { at: later, .. } = pose(&breaking, None, 0, 2000.0 + f64::from(SETTLE), 0.0, Some(from), Vec2::ZERO) else {
            panic!("on the ice");
        };
        assert!(later.distance(Vec3::new(1.0, 0.0, 2.0)) < 1e-5);
        // Whoever the ice broke under drops with it, and is gone.
        breaking.out[2] = Some(1);
        let Pose::On { at: dropping, .. } = pose(&breaking, None, 2, 2000.2, 0.0, None, Vec2::ZERO) else {
            panic!("dropping");
        };
        assert!(dropping.y < 0.0);
        assert_eq!(pose(&breaking, None, 2, 2001.5, 0.0, None, Vec2::ZERO), Pose::Gone);
    }

    #[test]
    fn the_slide_begins_as_the_reveal_s_time_is_up_and_its_own_round_goes_on_with_it() {
        // Four people sliding at the middle, as the game reveals it.
        let floe = round_island();
        let mut game = Game::new(4, &[true; 4], 1, Arc::clone(&floe));
        game.begin(1000.0, &[]);
        for seat in 0..4 {
            game.aim(seat, -rules::arrival(seat).0.xz(), 6);
            game.ready(seat);
        }
        game.step(1000.01);
        let reveal = game.round();
        assert_eq!(reveal.stage, KnockStage::Reveal);
        // Not before its time is up; then, by the reveal's own round, from
        // the moment it was up.
        assert_eq!(slide_begins(&reveal, reveal.until - 0.01), None);
        assert_eq!(slide_begins(&reveal, reveal.until + 0.05), Some(reveal.until));
        let mut early = Played::of(&reveal, reveal.until, &floe);
        // Drawn as the slide at once, where the game has everyone then.
        let at = reveal.until + 0.15;
        game.step(at);
        early.slide.step_to((at - early.since) as f32);
        for seat in 0..4 {
            let Pose::On { at: drawn, sliding, .. } = pose(&reveal, Some(&early), seat, at, 0.0, None, Vec2::ZERO) else {
                panic!("on the ice");
            };
            assert!(drawn.distance(game.at(seat).0) < 1e-5, "seat {seat}: {drawn} and {}", game.at(seat).0);
            assert!(sliding, "seat {seat}");
        }
        // The slide's own round, a little after, is that very slide: it goes
        // on, not begun again.
        let slide = game.round();
        assert_eq!(slide.stage, KnockStage::Slide);
        assert_eq!(slide_begins(&slide, at), Some(slide.since));
        assert!(early.is(&slide, slide.since));
        // Unless someone left in the reveal's last moment: then it says
        // other than the reveal did, and is begun from.
        let mut changed = slide.clone();
        changed.start[2] = None;
        changed.launches[2] = (0.0, 0.0);
        assert!(!early.is(&changed, changed.since));
    }

    #[test]
    fn a_body_slides_facing_the_way_it_goes_and_swings_round_to_it() {
        // Seat 0 set off sideways, across the way it faced; seat 2 stands.
        let mut sliding = round(KnockStage::Slide);
        sliding.launches = vec![(9.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];
        let mut played = Played::of(&sliding, sliding.since, &round_island());
        played.slide.step_to(0.2);
        let Pose::On { yaw, sliding: posed, .. } = pose(&sliding, Some(&played), 0, sliding.since + 0.2, 0.0, None, Vec2::ZERO) else {
            panic!("on the ice");
        };
        assert!(posed);
        let ahead = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
        assert!(ahead.x > 0.999, "{ahead}");
        // Standing, as it was.
        let Pose::On { yaw, sliding: posed, .. } = pose(&sliding, Some(&played), 2, sliding.since + 0.2, 0.7, None, Vec2::ZERO) else {
            panic!("on the ice");
        };
        assert!(!posed);
        assert_eq!(yaw, 0.7);
        // Swung round the short way (from 3 radians to -3 is on past a half
        // turn, not back through none), no faster than it turns.
        assert_eq!(turned(0.0, 2.5, 1.0), 1.0);
        assert!((turned(3.0, -3.0, 10.0) - (std::f32::consts::TAU - 3.0)).abs() < 1e-5);
        assert!((turned(1.0, 1.2, 1.0) - 1.2).abs() < 1e-6);
    }

    #[test]
    fn every_turn_s_aim_begins_at_one_with_no_way_so_that_doing_nothing_stays_put() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.init_resource::<Time>();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.init_resource::<TouchControls>();
        world.init_resource::<Messages<MouseWheel>>();
        world.init_resource::<crate::signup::NameBox>();
        world.init_resource::<Pressed>();
        let mut knock = Knock::sit(1, false);
        knock.floe = Some(round_island());
        knock.round = Some(round(KnockStage::Aim));
        world.insert_resource(knock);
        world.run_system_once(take_aim).expect("aimed");
        let aim = world.resource::<Knock>().aim;
        assert_eq!((aim.turn, aim.power, aim.way), (1, START_POWER, Vec2::ZERO));
        // Doing nothing it stays so: power, but no way, goes nowhere.
        world.run_system_once(take_aim).expect("aimed");
        assert_eq!(world.resource::<Knock>().aim.way, Vec2::ZERO);
        // A step up by hand with no way yet: toward the middle, as ever.
        world.resource_mut::<Pressed>().power = 1;
        world.run_system_once(take_aim).expect("aimed");
        let aim = world.resource::<Knock>().aim;
        assert_eq!(aim.power, START_POWER + 1);
        let from = rules::arrival(1).0.xz();
        assert!(aim.way.dot(-from.normalize()) > 0.999, "{:?}", aim.way);
        // The next turn begins at one again, whatever the last was.
        let mut next = round(KnockStage::Aim);
        next.turn = 2;
        world.resource_mut::<Knock>().round = Some(next);
        world.run_system_once(take_aim).expect("aimed");
        let aim = world.resource::<Knock>().aim;
        assert_eq!((aim.turn, aim.power, aim.way), (2, START_POWER, Vec2::ZERO));
    }

    #[test]
    fn ready_said_before_coming_back_is_ready_still() {
        let mut knock = Knock::sit(1, true);
        let mut aiming = round(KnockStage::Aim);
        knock.round = Some(aiming.clone());
        knock.aim = Aim {
            turn: 1,
            way: Vec2::X,
            power: 7,
            ..default()
        };
        ready_as_held(&mut knock);
        assert!(!knock.aim.ready && knock.aiming());
        // The round says you were: you are, with nothing more to send.
        aiming.ready[1] = true;
        knock.round = Some(aiming);
        ready_as_held(&mut knock);
        assert!(knock.aim.ready && !knock.aiming());
        assert_eq!(knock.aim.sent, Some((Vec2::X, 7)));
        assert_eq!(knock.aim.since_sent, f32::NEG_INFINITY);
    }

    #[test]
    fn begun_afresh_at_the_same_table_out_and_watching_the_panel_stays_away() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.init_resource::<Messages<table::Entered>>();
        world.init_resource::<Messages<table::Gone>>();
        world.init_resource::<Pressed>();
        world.init_resource::<OrbitCamera>();
        world.insert_resource(Knock {
            watching: true,
            ..Knock::sit(2, true)
        });
        let entered = |same_table: bool| table::Entered {
            game: Minigame::KnockOff,
            seat: 2,
            online: true,
            same_table,
        };
        world.write_message(entered(true));
        world.run_system_once(enter).expect("entered");
        assert!(world.resource::<Knock>().watching);
        // Another table: begun afresh, its panel its own.
        world.resource_mut::<Messages<table::Entered>>().clear();
        world.write_message(entered(false));
        world.run_system_once(enter).expect("entered");
        assert!(!world.resource::<Knock>().watching);
    }
}
