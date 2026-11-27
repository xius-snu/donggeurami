//! Sitting and lying down on what stands on the town's plots (Hajun,
//! 2026-11-22): "i want players to be able to sit, and lie down on chairs,
//! sofas, and beds. i think the proximitybubble thing can work here, but
//! reduce the activation distance so u need to go closer to the
//! chairs/sofas and make the icon instead of ... to be a chair_sit icon,
//! and lie_down icon ... for the sofa there should be 2 places u can sit so
//! 2 proximity bubbles, chair just 1, and single bed just 1, and double bed
//! 2. and it will do a very quick animation to get to that pose and stay
//! that pose and both animation + the pose should be visible to other
//! players. and jump to leave that pose and go back to normal."
//!
//! **Where** ([`spots`]): each chair, sofa and bed there is to own has its
//! places, in its own space, by its name: the cafe chair one to sit, the
//! soft couch two to sit back on, the single bed one to lie on and the
//! double bed two. Yours and everyone's in your channel alike, and only in
//! the town: a piece in House Builder is being built with. **And since
//! 2026-11-25 what came for the plots that day** (Hajun: "also the bed,
//! chair, sofas, couch need the interactability with the sitting/lying
//! down. also the bathtub could be lyind down animation, and the l shaped
//! couch can have maybe 3 seating places"): the chair, the rocking chair
//! and the stool one to sit on, the armchair one to sit back in, the L
//! shaped couch three, two along it and one on the end that comes forward,
//! the bed two to lie on, and the bathtub one, lying along it
//! ([`Spot::turn`]: each body faces as its piece does, but there), until
//! its tap came to be run instead (later on 2026-11-27, `working`;
//! [`LIE_IN_THE_BATH`]). **And the toilet one to sit on** (since
//! 2026-11-27; Hajun: "toilet = sit on toilet like a chair"), on its lid,
//! which is down; nobody of the computer's sits there
//! ([`residents_sit_on`]).
//!
//! **The bubbles** ([`offer`]): come within [`REACH`] of such a thing, on
//! its floor, and a bubble stands over each of its places that is free and
//! in sight of your eyes (`bubble`: a thing in the world, someone sat or
//! someone lying in it). Much nearer than someone to talk to is found
//! (`talk::REACH`, 4.5 m). Not in edit mode, where a tap is for the thing
//! itself, nor in your car, nor while anything else is up.
//!
//! **Resting** ([`Resting`], [`sit`]): a tap on a bubble, or E on desktop
//! for the nearest, and you are there: your body at the place, facing as
//! the thing does, and in a fifth of a second in its pose
//! ([`roundtown_net::Pose`]). The pose is your model's, moved and turned
//! under your body ([`settle`], as `arena::squash` lays one flat), and
//! your limbs' ([`limb`], from `animate_walk`): sat, the hips on the seat
//! and the knees bent over its front; sat back, the legs out along the
//! cushion; lying, on the back with the head on the pillow. Nothing moves
//! you meanwhile (`move_bodies`, `read_player_input`), you are in nobody's
//! way, and you throw no punch.
//!
//! **Up again** ([`leave`]): the jump button, or Space, and you are on your
//! feet where you stood before, jumping. And by itself, with no jump, if
//! what you rest on is picked up, moved or taken away, or you get into
//! your car, or leave the town.
//!
//! **Everyone sees it.** Your body is where it rests, which your moves say
//! as ever; and you say how you rest (`Ask::Rest`), which the server tells
//! everyone in your channel whose app asked (`Ask::Resting`,
//! `Tell::Resting`: at the end of what is said, with no new protocol). Who
//! rests is drawn so ([`their_poses`]), getting into the pose as you do.

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::{Ask, Member, Pose, Tell};

use crate::build::{Bounds, Ghost, Piece};
use crate::bubble::{self, Bubble, Looks, Says, Tapped};
use crate::customize::Customize;
use crate::drive::Driving;
use crate::hud::DialogUp;
use crate::island::{Islands, Venue};
use crate::net::Online;
use crate::{CloseUp, Dead, EYE_HEIGHT, Intent, JUMP_SPEED, Player, PlayerJump, WalkBone, WalkBoneKind, WalkCycle};

/// How near a chair, a sofa or a bed you have to be for its bubbles to
/// come up, in metres from the nearest edge of it along the ground; how
/// near to keep them, so that one does not flicker at the edge of that;
/// and how far over or under its foot your feet may be, which keeps the
/// floor above and the floor below out of it.
const REACH: f32 = 1.5;
const KEEP_REACH: f32 = 1.9;
const HIGHER: f32 = 1.2;
/// How far from you a place itself may be, along the ground: no further
/// than a body may be put in one go (the server's `BURST`, 4 m).
const FAR: f32 = 3.7;
/// How near a place someone resting has to be for it to be theirs.
const TAKEN: f32 = 0.45;
/// How long getting into a pose takes, in seconds.
const EASE_SECS: f32 = 0.2;

/// Sat: how far the model is let down for its hips to be on the seat (the
/// hip is 0.68 m up, and a thigh 0.07 thick under it), and how far each
/// limb is turned forward from how it stands, in radians: the thighs out
/// in front, the shins down from the knee, the arms a little forward.
const SIT_DROP: f32 = 0.61;
const SIT_THIGH: f32 = 1.5;
const SIT_SHIN: f32 = 1.45;
const SIT_ARM: f32 = 0.3;
/// Sat back: how far the body leans back, and its limbs, the legs out
/// level along the cushion: forward of the body by a quarter turn less
/// what it leans.
const LOUNGE_LEAN: f32 = 0.16;
const LOUNGE_THIGH: f32 = FRAC_PI_2 - LOUNGE_LEAN;
const LOUNGE_SHIN: f32 = 0.06;
const LOUNGE_ARM: f32 = 0.42;
/// Lying: how far up from level the body lies, head end up, for its head
/// to be on a pillow 0.22 m over where its heels are; how high its middle
/// is over where its heels lie; and how far up a body its middle is.
const LIE_TILT: f32 = 0.32;
const LIE_LIFT: f32 = 0.3;
const LIE_MID: f32 = 0.82;

/// Somewhere to rest on a piece, in the piece's own space (its front is
/// its -Z): how, where the body is put (the hips' place on the seat, or
/// under the middle of someone lying), where the point of its bubble is,
/// and how far the body there is turned about the upright from the way the
/// piece faces, in radians (to its left, as the engine turns): nothing, but
/// in the bathtub, which is lain along.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Spot {
    pub(crate) pose: Pose,
    pub(crate) at: Vec3,
    pub(crate) tip: Vec3,
    pub(crate) turn: f32,
}

const fn spot(pose: Pose, at: Vec3, tip: Vec3) -> Spot {
    Spot { pose, at, tip, turn: 0.0 }
}

/// The cafe chair, a round stool 0.77 m high with a back round the far
/// half of it: sat on the front of it, the knees over its edge.
const CHAIR: [Spot; 1] = [spot(Pose::Sit, Vec3::new(0.0, 0.77, -0.07), Vec3::new(0.0, 1.55, -0.07))];
/// The soft couch: a cushion 0.62 m high between two arms, 2.9 m across
/// and 1.2 deep, its back rising from 0.13 m behind its middle.
const SOFA: [Spot; 2] = [
    spot(Pose::Lounge, Vec3::new(-0.72, 0.62, -0.02), Vec3::new(-0.72, 1.68, -0.02)),
    spot(Pose::Lounge, Vec3::new(0.72, 0.62, -0.02), Vec3::new(0.72, 1.68, -0.02)),
];
/// The beds: 0.95 m high where the blanket is, a pillow 0.22 over that at
/// the head, 1.5 m behind the middle. The middle of someone lying is 0.52
/// short of their head.
const SINGLE_BED: [Spot; 1] = [spot(Pose::Lie, Vec3::new(0.0, 0.95, 0.98), Vec3::new(0.0, 1.5, 1.1))];
const DOUBLE_BED: [Spot; 2] = [
    spot(Pose::Lie, Vec3::new(-0.62, 0.95, 0.98), Vec3::new(-0.62, 1.5, 1.1)),
    spot(Pose::Lie, Vec3::new(0.62, 0.95, 0.98), Vec3::new(0.62, 1.5, 1.1)),
];

// What came for the plots on 2026-11-25 (`mybuilds/newones`), each as its
// model is in the game, measured off it (a test holds each place to its
// surfaces); all but the bathtub face their own -Z as the game has them.
// Measured again that afternoon, the 56 downloads made three times their
// size where they had been two and a half (Hajun: "i meant 3x"): a body
// being the size it was, each sits as far in from the front of a seat,
// and as far out from the back of a sofa, and lies with the head at the
// pillow, as before; the heights and the bubbles are the models'.

/// The wooden chair: a seat 0.8 m high and 0.82 deep, its back behind it;
/// sat on 0.35 m in from its front, as the others.
const WOODEN_CHAIR: [Spot; 1] = [spot(Pose::Sit, Vec3::new(0.0, 0.8, 0.02), Vec3::new(0.0, 2.1, 0.02))];
/// The rocking chair: a seat that slopes back from 0.93 m to 0.84, between
/// its arms; sat on 0.35 m in from its front.
const ROCKING_CHAIR: [Spot; 1] = [spot(Pose::Sit, Vec3::new(0.0, 0.9, -0.55), Vec3::new(0.0, 2.28, -0.55))];
/// The bar stool: a round seat 0.64 m across and 1.09 high.
const STOOL: [Spot; 1] = [spot(Pose::Sit, Vec3::new(0.0, 1.09, 0.0), Vec3::new(0.0, 1.98, 0.0))];
/// The armchair: a cushion 0.93 m high between its arms, a deep one, its
/// back rising 0.45 m behind its origin: sat back in, as on a sofa, 0.2 m
/// out from it. The middle of its seat is 0.22 m left of its origin.
const ARMCHAIR: [Spot; 1] = [spot(Pose::Lounge, Vec3::new(-0.22, 0.93, 0.25), Vec3::new(-0.22, 2.34, 0.25))];
/// The L shaped couch: cushions 0.87 m high, its back rising 0.4 m behind
/// its origin all along it, and its left end coming forward 2.4 m: two
/// along it and one on that end, the legs out along it.
const L_COUCH: [Spot; 3] = [
    spot(Pose::Lounge, Vec3::new(-1.38, 0.87, 0.22), Vec3::new(-1.38, 2.1, 0.22)),
    spot(Pose::Lounge, Vec3::new(-0.04, 0.87, 0.22), Vec3::new(-0.04, 2.1, 0.22)),
    spot(Pose::Lounge, Vec3::new(1.56, 0.87, 0.22), Vec3::new(1.56, 2.1, 0.22)),
];
/// The big bed: a blanket 1.51 m high and 1.56 under the pillows' end of
/// it, its two pillows rising from 1.82 m behind its origin, which is 0.8 m
/// right of its middle: one lying on each side, the head against the
/// pillow.
const BIG_BED: [Spot; 2] = [
    spot(Pose::Lie, Vec3::new(-0.14, 1.52, 1.3), Vec3::new(-0.14, 2.76, 1.3)),
    spot(Pose::Lie, Vec3::new(1.76, 1.52, 1.3), Vec3::new(1.76, 2.76, 1.3)),
];
/// The toilet, its lid down: a seat 0.87 m high and 0.97 deep in front of
/// its tank, which rises from 0.03 m behind its origin; sat on 0.35 m in
/// from its front, as a chair is.
const TOILET: [Spot; 1] = [spot(Pose::Sit, Vec3::new(0.0, 0.87, -0.585), Vec3::new(0.0, 2.1, -0.585))];
/// Whether the bathtub is lain in: so it was from 2026-11-25 until later on
/// 2026-11-27, when Hajun had its tap run instead ("for the bathtub, instead
/// of being a lie down proximity button, make it similar to the sink";
/// `working`): off, and nobody lies in it, you or anyone of the computer's.
pub(crate) const LIE_IN_THE_BATH: bool = false;

/// Hajun's bathtub, lain in along its length, the head toward its left end
/// (its -X), which slopes up from 1.4 m out: on its floor, 0.17 m up.
const BATHTUB: [Spot; 1] = [Spot {
    pose: Pose::Lie,
    at: Vec3::new(-0.73, 0.17, 0.0),
    tip: Vec3::new(-0.73, 1.95, 0.0),
    turn: -FRAC_PI_2,
}];

/// Where one can rest on a piece of this kind (`build::PieceDef::kind`):
/// nowhere, on most.
pub(crate) fn spots(kind: &str) -> &'static [Spot] {
    match kind {
        "cafechair" => &CHAIR,
        "softcouch" => &SOFA,
        "singlebed" => &SINGLE_BED,
        "doublebed" => &DOUBLE_BED,
        "chair" => &WOODEN_CHAIR,
        "rocking_chair" => &ROCKING_CHAIR,
        "stool" => &STOOL,
        "armchair" => &ARMCHAIR,
        "couch" => &L_COUCH,
        "bed" => &BIG_BED,
        "bathtub" if LIE_IN_THE_BATH => &BATHTUB,
        "toilet" => &TOILET,
        _ => &[],
    }
}

/// Whether the computer's people sit and lie on a piece of this kind, as
/// you may (`resident`): everywhere but on the toilet (mine, 2026-11-27).
pub(crate) fn residents_sit_on(kind: &str) -> bool {
    kind != "toilet"
}

/// How high over a place's surface a body resting like this is put: one
/// lying is put by its middle.
fn lift(pose: Pose) -> f32 {
    match pose {
        Pose::Sit | Pose::Lounge => 0.0,
        Pose::Lie => LIE_LIFT,
    }
}

/// Where a body rests on `place` of a piece that stands as `piece` does:
/// there, facing as the piece does, turned as the place is.
pub(crate) fn rests_at(piece: &Transform, place: &Spot) -> Transform {
    Transform {
        translation: piece.transform_point(place.at) + Vec3::Y * lift(place.pose),
        rotation: piece.rotation * Quat::from_rotation_y(place.turn),
        scale: Vec3::ONE,
    }
}

/// Someone sitting or lying on something: how, how far into the pose they
/// are, from 0 standing to 1, and, of you alone, what you rest on ([`On`]).
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Resting {
    pub(crate) pose: Pose,
    eased: f32,
    on: Option<On>,
}

/// What you rest on: the piece, where it stood as you came to it, where
/// your body was put, and where you stood before, which is where you get
/// up to.
#[derive(Clone, Copy, PartialEq, Debug)]
struct On {
    piece: Entity,
    stood: Transform,
    at: Vec3,
    from: Vec3,
}

impl Resting {
    /// Just come to rest like this, standing still.
    pub(crate) fn begun(pose: Pose) -> Self {
        Self {
            pose,
            eased: 0.0,
            on: None,
        }
    }

    /// What you rest on, if it is you: nothing of it is in the way of the
    /// camera's sight of you (`see_through`), sat in it as you are.
    pub(crate) fn on(&self) -> Option<Entity> {
        self.on.map(|on| on.piece)
    }

    /// How much of the pose the body is in, from 0 to 1: quickly at first,
    /// and settling into it.
    fn share(&self) -> f32 {
        let left = 1.0 - self.eased.clamp(0.0, 1.0);
        1.0 - left * left
    }
}

/// A body whose model has been moved into a pose: the model, and how it
/// stands in its body at rest, to be put back.
#[derive(Component)]
struct Settled {
    model: Entity,
    rest: Transform,
}

/// A bubble over somewhere to rest: on this piece, this one of its places.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Place {
    piece: Entity,
    spot: usize,
}

/// Who in your channel of the town rests, and how, as the server last said
/// (`Tell::Resting`): by their `Member::id`. Nobody, offline.
#[derive(Resource, Default, PartialEq, Debug)]
struct Theirs(HashMap<u64, Pose>);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Theirs>().add_systems(
        Update,
        (
            // Up before anything moves a body this frame, so that the jump
            // that got you up is this frame's.
            leave.after(crate::read_player_input).before(crate::move_bodies),
            (ease, settle).chain().after(crate::move_bodies).before(crate::animate_walk),
            offer.after(crate::move_bodies).before(bubble::BubbleSystems),
            sit.after(bubble::Taps),
            (hear, their_poses).chain(),
            say_so_on_coming_online,
        ),
    );
}

// ---------------------------------------------------------------- the pose

/// Where a model that stands in its body as `rest` does is in `pose`: let
/// down onto its hips, leant back, or laid on its back about its middle.
fn posed(pose: Pose, rest: &Transform) -> Transform {
    let (turn, pivot) = match pose {
        Pose::Sit => (Quat::IDENTITY, SIT_DROP),
        // In the body's own space it faces -Z with +Y up: back is a turn
        // about its side to side axis that takes its head behind it.
        Pose::Lounge => (Quat::from_rotation_x(LOUNGE_LEAN), SIT_DROP),
        Pose::Lie => (Quat::from_rotation_x(FRAC_PI_2 - LIE_TILT), LIE_MID),
    };
    Transform {
        translation: turn * (rest.translation - Vec3::Y * pivot),
        rotation: turn * rest.rotation,
        scale: rest.scale,
    }
}

/// How a limb is turned, in its body's space, of someone resting: as it
/// stands (`stance`, `crate::stood`), and turned forward about the body's
/// side to side axis by as much of the pose's turn as they are into it. A
/// knee bends back, as it does walking.
pub(crate) fn limb(kind: WalkBoneKind, stance: f32, resting: &Resting) -> Quat {
    let (thigh, shin, arm) = match resting.pose {
        Pose::Sit => (SIT_THIGH, SIT_SHIN, SIT_ARM),
        Pose::Lounge => (LOUNGE_THIGH, LOUNGE_SHIN, LOUNGE_ARM),
        Pose::Lie => (0.0, 0.0, 0.0),
    };
    let forward = match kind {
        WalkBoneKind::ThighL | WalkBoneKind::ThighR => thigh,
        WalkBoneKind::ShinL | WalkBoneKind::ShinR => -shin,
        WalkBoneKind::ArmL | WalkBoneKind::ArmR => arm,
    };
    Quat::from_axis_angle(Vec3::X, forward * resting.share()) * crate::stood(stance)
}

/// How much lower over where their body is the head of someone resting is
/// than of someone on their feet, in metres, for what is written over it
/// (`tags`): sat, by as much as their hips came down; lying, their name is
/// over the middle of them.
pub(crate) fn lower(resting: Option<&Resting>) -> f32 {
    match resting.map(|rests| rests.pose) {
        None => 0.0,
        Some(Pose::Sit | Pose::Lounge) => SIT_DROP,
        Some(Pose::Lie) => 1.1,
    }
}

/// Brings everyone resting further into their pose, until they are in it.
fn ease(time: Res<Time>, mut resting: Query<&mut Resting>) {
    let step = time.delta_secs() / EASE_SECS;
    for mut rests in &mut resting {
        if rests.eased < 1.0 {
            rests.eased = (rests.eased + step).min(1.0);
        }
    }
}

/// Moves the model of everyone resting into their pose, as far as they are
/// into it, and puts it back as they get up. Its model is the part of a
/// body its limbs are under. Written only while a pose is being got into,
/// and once as it is left.
#[allow(clippy::type_complexity)]
fn settle(
    resting: Query<(Entity, Ref<Resting>, Option<&Settled>, &Children)>,
    up: Query<(Entity, &Settled), Without<Resting>>,
    children: Query<&Children>,
    bones: Query<(), With<WalkBone>>,
    mut models: Query<&mut Transform>,
    mut commands: Commands,
) {
    for (body, rests, settled, parts) in &resting {
        let (model, rest) = match settled {
            Some(_) if !rests.is_changed() => continue,
            Some(settled) => (settled.model, settled.rest),
            None => {
                let Some(model) = parts
                    .iter()
                    .find(|&part| children.iter_descendants(part).any(|under| bones.contains(under)))
                else {
                    continue;
                };
                let Ok(rest) = models.get(model).map(|place| *place) else {
                    continue;
                };
                commands.entity(body).insert(Settled { model, rest });
                (model, rest)
            }
        };
        let Ok(mut place) = models.get_mut(model) else {
            continue;
        };
        let (to, share) = (posed(rests.pose, &rest), rests.share());
        let now = Transform {
            translation: rest.translation.lerp(to.translation, share),
            rotation: rest.rotation.slerp(to.rotation, share),
            scale: rest.scale,
        };
        if *place != now {
            *place = now;
        }
    }
    for (body, settled) in &up {
        if let Ok(mut place) = models.get_mut(settled.model)
            && *place != settled.rest
        {
            *place = settled.rest;
        }
        commands.entity(body).remove::<Settled>();
    }
}

// ------------------------------------------------------------- the bubbles

/// How far `at` is from the edge of a piece that stands as `piece` does and
/// is as big as `bounds`, along the ground: nothing, over it.
fn off_the_edge(piece: &Transform, bounds: &Bounds, at: Vec3) -> f32 {
    let own = piece.compute_affine().inverse().transform_point3(at);
    let nearest = own.clamp(bounds.low, bounds.high);
    (own.xz() - nearest.xz()).length()
}

/// Which of a piece's places a bubble is to stand over, for someone
/// standing at `you`: those of a piece within reach, on your floor, each no
/// further than a body is put in one go, free of anyone resting (`taken`,
/// where each of them is) and in sight (`clear`, from your eyes to where
/// its bubble is). `held`, its bubbles are up already, and stay to a
/// little further off.
pub(crate) fn offered<'a>(
    you: Vec3,
    piece: &'a Transform,
    bounds: &Bounds,
    places: &'a [Spot],
    held: bool,
    taken: &'a [Vec3],
    clear: impl Fn(Vec3, Vec3) -> bool + 'a,
) -> impl Iterator<Item = usize> + 'a {
    let reach = if held { KEEP_REACH } else { REACH };
    // Nowhere near it, which is nearly every piece there is, nothing more
    // is worked out; and whether a place is in sight, which takes longest
    // to say, is asked last, of a place that is otherwise to be offered.
    let near = (you.y - piece.translation.y).abs() <= HIGHER
        && you.xz().distance(piece.translation.xz()) <= reach + bounds.low.xz().length().max(bounds.high.xz().length())
        && off_the_edge(piece, bounds, you) <= reach;
    places.iter().enumerate().filter(move |_| near).filter_map(move |(index, place)| {
        let there = rests_at(piece, place).translation;
        let free = taken.iter().all(|other| other.distance(there) > TAKEN);
        let tip = piece.transform_point(place.tip);
        (free && there.xz().distance(you.xz()) <= FAR && clear(you + Vec3::Y * EYE_HEIGHT, tip + Vec3::Y * bubble::RADIUS)).then_some(index)
    })
}

/// Keeps a bubble over every place you could rest on just now, and takes
/// away the rest: in the town, on your feet, out of your car and out of
/// edit mode, with nothing else up.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn offer(
    me: Query<(&Transform, &Venue, Has<Resting>), (With<Player>, Without<Dead>)>,
    pieces: Query<(Entity, &Piece, &Transform, &Venue, &Bounds, &InheritedVisibility), Without<Ghost>>,
    resting: Query<&Transform, With<Resting>>,
    bubbles: Query<(Entity, &Place)>,
    (islands, looks): (Res<Islands>, Option<Res<Looks>>),
    (dialog, close_up, driving, customize): (Res<DialogUp>, Res<CloseUp>, Res<Driving>, Res<Customize>),
    mut commands: Commands,
) {
    let mut wanted: Vec<(Place, Vec3)> = Vec::new();
    let you = me
        .single()
        .ok()
        .filter(|&(_, &venue, rests)| venue == Venue::Town && !rests && !dialog.0 && !close_up.busy() && !driving.on() && !customize.on());
    if let (Some((you, ..)), Some(island)) = (you, islands.get(Venue::Town)) {
        let taken: Vec<Vec3> = resting.iter().map(|body| body.translation).collect();
        let clear = |from: Vec3, to: Vec3| island.sweep(from, to, 0.05).is_none();
        for (piece, of, place, &venue, bounds, shown) in &pieces {
            let places = spots(of.def.kind);
            if places.is_empty() || venue != Venue::Town || !shown.get() {
                continue;
            }
            let held = bubbles.iter().any(|(_, over)| over.piece == piece);
            for spot in offered(you.translation, place, bounds, places, held, &taken, clear) {
                wanted.push((Place { piece, spot }, place.transform_point(places[spot].tip)));
            }
        }
    }
    for (bubble, over) in &bubbles {
        if !wanted.iter().any(|(place, _)| place == over) {
            commands.entity(bubble).despawn();
        }
    }
    let Some(looks) = looks else {
        return;
    };
    for (place, tip) in wanted {
        if bubbles.iter().any(|(_, over)| *over == place) {
            continue;
        }
        let Ok((_, of, ..)) = pieces.get(place.piece) else {
            continue;
        };
        let says = match spots(of.def.kind)[place.spot].pose {
            Pose::Sit | Pose::Lounge => Says::Sit,
            Pose::Lie => Says::Lie,
        };
        commands.spawn((place, looks.bubble(says, tip, Venue::Town)));
    }
}

// ------------------------------------------------------ resting, and up

/// A tap on a bubble over somewhere to rest, or E on desktop with it the
/// nearest bubble (`bubble::tap`): you are there, in its pose from
/// standing, and the server is told for everyone else in your channel.
#[allow(clippy::type_complexity)]
fn sit(
    mut tapped: MessageReader<Tapped>,
    bubbles: Query<&Place, With<Bubble>>,
    pieces: Query<(&Piece, &Transform), (Without<Ghost>, Without<Player>)>,
    mut me: Query<(Entity, &mut Transform, &mut PlayerJump, &mut WalkCycle, Has<Resting>), With<Player>>,
    online: Res<Online>,
    mut asks: MessageWriter<Ask>,
    mut commands: Commands,
) {
    let Ok((you, mut body, mut jump, mut walk, rests)) = me.single_mut() else {
        tapped.clear();
        return;
    };
    let place = tapped.read().find_map(|&Tapped(bubble)| bubbles.get(bubble).ok().copied());
    let Some(place) = place.filter(|_| !rests) else {
        return;
    };
    let Ok((of, piece)) = pieces.get(place.piece) else {
        return;
    };
    let Some(spot) = spots(of.def.kind).get(place.spot) else {
        return;
    };
    let from = body.translation;
    *body = rests_at(piece, spot);
    jump.velocity_y = 0.0;
    walk.speed = 0.0;
    commands.entity(you).insert(Resting {
        on: Some(On {
            piece: place.piece,
            stood: *piece,
            at: body.translation,
            from,
        }),
        ..Resting::begun(spot.pose)
    });
    if online.is() {
        asks.write(Ask::Rest(Some(spot.pose)));
    }
}

/// Whether what you rest on is still under you, your body being at `body`:
/// there, down, and standing where it stood as you came to it, and you
/// where you were put on it, which nothing else has moved you from (the
/// server, were it ever to put you back where you stood).
fn still_there(on: Option<On>, body: Vec3, pieces: &Query<&Transform, (With<Piece>, Without<Ghost>, Without<Player>)>) -> bool {
    on.is_none_or(|on| pieces.get(on.piece).is_ok_and(|now| *now == on.stood) && body.distance(on.at) < 0.05)
}

/// Gets you up: at the jump button, or Space, on your feet where you stood
/// before you sat or lay down, and jumping there; and with no jump, where
/// you are, as what you rest on is picked up, moved or taken away, as you
/// get into your car, and as you leave the town or die. The server is
/// told.
///
/// Back where you stood, and not up where you sat: that is ground you were
/// standing on, clear of everything. Got up where it sat and gone on
/// forward off the seat, a body sat at a table landed on the table.
#[allow(clippy::type_complexity)]
fn leave(
    mut me: Query<(Entity, &mut Transform, &Venue, &mut Intent, &mut PlayerJump, &Resting, Has<Dead>), With<Player>>,
    pieces: Query<&Transform, (With<Piece>, Without<Ghost>, Without<Player>)>,
    (online, driving): (Res<Online>, Res<Driving>),
    mut asks: MessageWriter<Ask>,
    mut commands: Commands,
) {
    let Ok((you, mut body, &venue, mut intent, mut jump, rests, dead)) = me.single_mut() else {
        return;
    };
    let jumped = intent.jump;
    let gone = venue != Venue::Town || dead || driving.on() || !still_there(rests.on, body.translation, &pieces);
    if !jumped && !gone {
        return;
    }
    commands.entity(you).remove::<Resting>();
    if online.is() {
        asks.write(Ask::Rest(None));
    }
    if jumped && !gone {
        if let Some(on) = rests.on {
            body.translation = on.from;
        }
        jump.velocity_y = JUMP_SPEED;
        intent.jump = false;
    }
}

// ----------------------------------------------------------- everyone else

/// Tells a server this device has just come online with that it draws who
/// rests, and, if you are resting, that you are: you sat down offline, or
/// before the connection went.
fn say_so_on_coming_online(
    online: Res<Online>,
    me: Query<&Resting, With<Player>>,
    mut was: Local<bool>,
    mut asks: MessageWriter<Ask>,
) {
    let now = online.is();
    if now && !*was {
        asks.write(Ask::Resting);
        if let Ok(rests) = me.single() {
            asks.write(Ask::Rest(Some(rests.pose)));
        }
    }
    *was = now;
}

/// Keeps who rests in your channel as the server says it, and forgets it
/// offline.
fn hear(mut tells: MessageReader<Tell>, online: Res<Online>, mut theirs: ResMut<Theirs>) {
    let mut said = None;
    for tell in tells.read() {
        if let Tell::Resting(everyone) = tell {
            said = Some(everyone);
        }
    }
    match said {
        Some(everyone) => {
            let now: HashMap<u64, Pose> = everyone.iter().map(|posed| (posed.who, posed.pose)).collect();
            if theirs.0 != now {
                theirs.0 = now;
            }
        }
        None if !online.is() && !theirs.0.is_empty() => theirs.0.clear(),
        None => {}
    }
}

/// How someone else in the town is to be drawn, of those the server says
/// rest: in their pose, if they are one of them, and on their feet if not.
fn pose_of(theirs: &HashMap<u64, Pose>, who: u64, venue: Venue) -> Option<Pose> {
    (venue == Venue::Town).then(|| theirs.get(&who).copied()).flatten()
}

/// Draws everyone else in your channel who rests in their pose, getting
/// into it as they come to it, and on their feet again as they get up.
/// Only what has changed is written.
fn their_poses(
    theirs: Res<Theirs>,
    bodies: Query<(Entity, &Member, &Venue, Option<&Resting>), With<Remote>>,
    mut commands: Commands,
) {
    for (body, member, &venue, rests) in &bodies {
        match (pose_of(&theirs.0, member.id, venue), rests) {
            (Some(pose), Some(rests)) if rests.pose == pose => {}
            (Some(pose), _) => {
                commands.entity(body).try_insert(Resting::begun(pose));
            }
            (None, Some(_)) => {
                commands.entity(body).try_remove::<Resting>();
            }
            (None, None) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::PIECES;

    fn bounds(kind: &str) -> Bounds {
        // As the models are (`assets/mybuilds`), to the centimetre.
        let (low, high) = match kind {
            "cafechair" => (Vec3::new(-0.42, 0.0, -0.42), Vec3::new(0.42, 1.38, 0.42)),
            "softcouch" => (Vec3::new(-1.88, 0.0, -1.06), Vec3::new(1.88, 1.53, 0.94)),
            "singlebed" => (Vec3::new(-1.15, 0.0, -2.25), Vec3::new(1.14, 1.17, 2.25)),
            _ => (Vec3::new(-1.5, 0.0, -2.25), Vec3::new(1.5, 1.17, 2.25)),
        };
        Bounds { low, high }
    }

    /// Every piece with somewhere to rest, how many places each, and how
    /// one rests there. (The bathtub, while it is lain in:
    /// [`LIE_IN_THE_BATH`].)
    const RESTED_ON: [(&str, usize, Pose); 11] = [
        ("cafechair", 1, Pose::Sit),
        ("softcouch", 2, Pose::Lounge),
        ("singlebed", 1, Pose::Lie),
        ("doublebed", 2, Pose::Lie),
        ("chair", 1, Pose::Sit),
        ("rocking_chair", 1, Pose::Sit),
        ("stool", 1, Pose::Sit),
        ("armchair", 1, Pose::Lounge),
        ("couch", 3, Pose::Lounge),
        ("bed", 2, Pose::Lie),
        ("toilet", 1, Pose::Sit),
    ];

    #[test]
    fn a_chair_has_one_place_a_sofa_two_a_single_bed_one_and_a_double_bed_two() {
        let count = |kind: &str, pose: Pose| spots(kind).iter().filter(|spot| spot.pose == pose).count();
        for (kind, places, pose) in RESTED_ON {
            assert_eq!((spots(kind).len(), count(kind, pose)), (places, places), "{kind}");
            // Each of them a piece there is.
            assert!(PIECES.iter().any(|def| def.kind == kind), "{kind}");
        }
        // And nothing else has any: a table, a house, a door.
        let with_places = PIECES.iter().filter(|def| !spots(def.kind).is_empty()).count();
        assert_eq!(with_places, RESTED_ON.len());
        // Two places on one piece far enough apart for two to rest side by
        // side; the L shaped couch's three each a metre from the next.
        for (kind, ..) in RESTED_ON {
            let places = spots(kind);
            for (nth, one) in places.iter().enumerate() {
                for other in &places[nth + 1..] {
                    assert!(one.at.distance(other.at) > 1.0, "{kind}");
                }
            }
        }
        // The bathtub's tap is run, and nobody lies in it.
        assert!(!LIE_IN_THE_BATH && spots("bathtub").is_empty());
        // All but the bathtub's face as their piece does; in it, lying along
        // it, the head to its left.
        for (kind, ..) in RESTED_ON {
            for place in spots(kind) {
                let head = Quat::from_rotation_y(place.turn) * Vec3::Z;
                if kind == "bathtub" {
                    assert!(head.distance(Vec3::NEG_X) < 1e-5, "{head}");
                } else {
                    assert_eq!(place.turn, 0.0, "{kind}");
                }
            }
        }
    }

    /// The highest surface of `triangles` over the point (`x`, `z`), if any
    /// is: what a vertical line down through it meets first.
    fn top_of(triangles: &[[Vec3; 3]], x: f32, z: f32) -> Option<f32> {
        let at = Vec2::new(x, z);
        triangles
            .iter()
            .filter_map(|[a, b, c]| {
                let (a2, b2, c2) = (a.xz(), b.xz(), c.xz());
                let area = (b2 - a2).perp_dot(c2 - a2);
                if area.abs() < 1e-9 {
                    return None;
                }
                let u = (b2 - at).perp_dot(c2 - at) / area;
                let v = (c2 - at).perp_dot(a2 - at) / area;
                let w = 1.0 - u - v;
                (u >= -1e-6 && v >= -1e-6 && w >= -1e-6).then(|| u * a.y + v * b.y + w * c.y)
            })
            .reduce(f32::max)
    }

    #[test]
    fn every_place_to_rest_is_on_its_piece_as_its_model_is() {
        // Each read off its model as the game has it: where the body goes is
        // on the top of what is under it there (a seat, a blanket, the
        // bathtub's floor) to a few centimetres; its bubble over the piece,
        // clear of what is under it and well over the place; and someone
        // lying has their heels on what they lie on, and their head within
        // the piece.
        for (kind, ..) in RESTED_ON {
            let def = PIECES.iter().find(|def| def.kind == kind).unwrap();
            let triangles: Vec<[Vec3; 3]> = crate::island::glb::read(&format!("{}/{}.glb", def.dir, def.file))
                .into_iter()
                .flat_map(|part| part.triangles)
                .collect();
            let (low, high) = triangles
                .iter()
                .flatten()
                .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), &corner| (low.min(corner), high.max(corner)));
            let inside = |at: Vec3| at.xz().cmpgt(low.xz()).all() && at.xz().cmplt(high.xz()).all();
            for place in spots(kind) {
                let (at, tip) = (place.at, place.tip);
                assert!(inside(at) && inside(tip), "{kind}: {at} {tip}, its model {low} to {high}");
                let under = top_of(&triangles, at.x, at.z).unwrap_or_else(|| panic!("{kind}: nothing under {at}"));
                assert!((under - at.y).abs() < 0.05, "{kind}: {at} over a top {under} m up");
                let below_tip = top_of(&triangles, tip.x, tip.z).unwrap_or(0.0);
                assert!(tip.y > at.y + 0.4 && tip.y > below_tip + 0.3, "{kind}: bubble {tip} over {below_tip}");
                if place.pose == Pose::Lie {
                    // Heels 0.78 m and head 0.52 m from the middle, along the
                    // way the head points.
                    let toward = Quat::from_rotation_y(place.turn) * Vec3::Z;
                    let (heels, head) = (at - toward * 0.78, at + toward * 0.52);
                    let feet = top_of(&triangles, heels.x, heels.z).unwrap_or_else(|| panic!("{kind}: nothing under the heels"));
                    assert!((feet - at.y).abs() < 0.1, "{kind}: heels at {heels} on a top {feet} m up");
                    assert!(inside(head), "{kind}: head at {head}");
                }
            }
        }
    }

    #[test]
    fn bubbles_come_up_only_close_to_a_thing_and_over_its_free_places_in_sight() {
        // A sofa turned a quarter, off at (10, 0.27, 20).
        let sofa = Transform::from_xyz(10.0, 0.27, 20.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2));
        let size = bounds("softcouch");
        let open = |_: Vec3, _: Vec3| true;
        let shown = |you: Vec3, held: bool, taken: &[Vec3]| offered(you, &sofa, &size, &SOFA, held, taken, open).collect::<Vec<_>>();
        // Its front is its own -Z, which turned a quarter is the world's
        // -X: stood a metre in front of its edge, both come up.
        let front = |off: f32| Vec3::new(10.0 - 1.06 - off, 0.27, 20.0);
        assert_eq!(shown(front(1.0), false, &[]), [0, 1]);
        // Not from two and a half metres off, where someone to talk to
        // would be found long since (4.5 m), nor from 1.7; but up already,
        // they stay that far.
        assert!(shown(front(2.5), false, &[]).is_empty());
        assert!(shown(front(1.7), false, &[]).is_empty());
        assert_eq!(shown(front(1.7), true, &[]), [0, 1]);
        assert!(shown(front(2.0), true, &[]).is_empty());
        // Nor from the floor above.
        assert!(shown(front(1.0) + Vec3::Y * 3.0, false, &[]).is_empty());
        // Someone on one of its places: only the other.
        let first = rests_at(&sofa, &SOFA[0]).translation;
        assert_eq!(shown(front(1.0), false, &[first]), [1]);
        // And nothing through a wall.
        let wall = |_: Vec3, _: Vec3| false;
        assert_eq!(offered(front(1.0), &sofa, &size, &SOFA, false, &[], wall).count(), 0);
        // A bed from its foot: within reach of its edge, but its place is
        // further than a body is put in one go; along its side, there.
        let bed = Transform::from_xyz(0.0, 0.0, 0.0);
        let bed_size = bounds("singlebed");
        let at_the_foot = offered(Vec3::new(0.0, 0.0, -3.2), &bed, &bed_size, &SINGLE_BED, false, &[], open).count();
        let by_the_side = offered(Vec3::new(2.0, 0.0, 0.0), &bed, &bed_size, &SINGLE_BED, false, &[], open).count();
        assert_eq!((at_the_foot, by_the_side), (0, 1));
    }

    #[test]
    fn someone_in_the_bathtub_lies_along_it_with_the_head_toward_its_left_end() {
        // The bathtub turned a quarter in the world: the body turned with it
        // and a quarter more, so that its head (behind it, lying) is toward
        // the tub's own -X, wherever that is.
        let tub = Transform::from_xyz(3.0, 0.27, -1.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2));
        let lain = rests_at(&tub, &BATHTUB[0]);
        let head = lain.rotation * Vec3::Z;
        let tubs_left = tub.rotation * Vec3::NEG_X;
        assert!(head.distance(tubs_left) < 1e-5, "{head} {tubs_left}");
        assert!((lain.translation.y - (0.27 + 0.17 + LIE_LIFT)).abs() < 1e-5);
    }

    #[test]
    fn someone_resting_is_on_the_place_facing_as_the_thing_does() {
        let chair = Transform::from_xyz(4.0, 0.37, -2.0).with_rotation(Quat::from_rotation_y(1.0));
        let sat = rests_at(&chair, &CHAIR[0]);
        // On the seat, 0.77 m up it, a little in front of its middle.
        assert!((sat.translation.y - (0.37 + 0.77)).abs() < 1e-5);
        let ahead = chair.rotation * Vec3::NEG_Z;
        assert!((sat.translation - chair.translation).with_y(0.0).dot(ahead) > 0.05);
        assert_eq!(sat.rotation, chair.rotation);
        // Lying, the body is put by its middle, over the blanket, and its
        // head is toward the pillow, behind the bed's middle.
        let bed = Transform::from_xyz(0.0, 0.27, 0.0);
        let lain = rests_at(&bed, &SINGLE_BED[0]);
        assert!((lain.translation.y - (0.27 + 0.95 + LIE_LIFT)).abs() < 1e-5);
        assert!(lain.translation.z > 0.5);
    }

    #[test]
    fn a_pose_lets_the_model_down_onto_its_hips_or_lays_it_on_its_back_with_its_head_up() {
        // A model as the loader has it: turned half round in its body.
        let rest = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI));
        let place = |pose: Pose, point: Vec3| posed(pose, &rest).transform_point(rest.rotation.inverse() * point);
        // Sat: every point of it as far down as its hips were up, no more.
        let hip = Vec3::new(0.06, 0.684, 0.0);
        assert!((place(Pose::Sit, hip) - Vec3::new(0.06, 0.684 - SIT_DROP, 0.0)).length() < 1e-5);
        // Sat back: the hips where they would be sat, and the head behind
        // them.
        let head = Vec3::new(0.0, 1.375, -0.015);
        assert!((place(Pose::Lounge, Vec3::Y * SIT_DROP)).length() < 1e-5);
        assert!(place(Pose::Lounge, head).z > 0.05 && place(Pose::Lounge, head).y > 0.7);
        // Lying: the middle of it where its body is, the head behind it
        // and higher than the heels by a pillow's height, face up.
        assert!(place(Pose::Lie, Vec3::Y * LIE_MID).length() < 1e-5);
        let (head, heels) = (place(Pose::Lie, head), place(Pose::Lie, Vec3::new(0.0, 0.02, 0.05)));
        assert!(head.z > 0.4 && heels.z < -0.6, "{head} {heels}");
        // The back of the head, a head's radius under its middle, is a
        // pillow's 0.22 m over the heels.
        let rise = (head.y - 0.265) - heels.y;
        assert!((rise - 0.22).abs() < 0.03, "{rise}");
        // And the heels are where the body was lifted from.
        assert!((heels.y + LIE_LIFT).abs() < 0.03, "{}", heels.y);
        let face = posed(Pose::Lie, &rest).rotation * rest.rotation.inverse() * Vec3::NEG_Z;
        assert!(face.y > 0.9, "{face}");
    }

    #[test]
    fn limbs_get_into_a_pose_from_standing_and_stay_in_it() {
        let mut rests = Resting::begun(Pose::Sit);
        // Standing still as it begins: every limb as it stands.
        for kind in [WalkBoneKind::ThighL, WalkBoneKind::ShinR, WalkBoneKind::ArmL] {
            assert!(limb(kind, 0.1, &rests).angle_between(crate::stood(0.1)) < 1e-5);
        }
        rests.eased = 1.0;
        assert_eq!(rests.share(), 1.0);
        // Sat: the thighs out in front, nearly level; the shins down from
        // them, bent back at the knee; the two legs alike.
        let down = Vec3::NEG_Y;
        let thigh = limb(WalkBoneKind::ThighL, 0.0, &rests) * down;
        assert!(thigh.z < -0.95 && thigh.y.abs() < 0.2, "{thigh}");
        let shin = limb(WalkBoneKind::ThighL, 0.0, &rests) * limb(WalkBoneKind::ShinL, 0.0, &rests) * down;
        assert!(shin.y < -0.95, "{shin}");
        assert_eq!(limb(WalkBoneKind::ThighL, 0.0, &rests), limb(WalkBoneKind::ThighR, 0.0, &rests));
        // Sat back, the legs out along the cushion though the body leans.
        let back = Resting { eased: 1.0, ..Resting::begun(Pose::Lounge) };
        let leg = Quat::from_rotation_x(LOUNGE_LEAN) * limb(WalkBoneKind::ThighR, 0.0, &back) * down;
        assert!(leg.z < -0.99 && leg.y.abs() < 0.05, "{leg}");
        // Lying, as they stand.
        let lain = Resting { eased: 1.0, ..Resting::begun(Pose::Lie) };
        assert!(limb(WalkBoneKind::ArmR, -0.2, &lain).angle_between(crate::stood(-0.2)) < 1e-5);
        // Half way there in time is most of the way in the pose: quick.
        let half = Resting { eased: 0.5, ..Resting::begun(Pose::Sit) };
        assert!(half.share() > 0.7 && half.share() < 0.8);
    }

    #[test]
    fn only_those_the_server_says_rest_are_drawn_so_and_only_in_the_town() {
        let theirs: HashMap<u64, Pose> = [(7, Pose::Lie), (9, Pose::Sit)].into();
        assert_eq!(pose_of(&theirs, 7, Venue::Town), Some(Pose::Lie));
        assert_eq!(pose_of(&theirs, 9, Venue::Town), Some(Pose::Sit));
        assert_eq!(pose_of(&theirs, 8, Venue::Town), None);
        assert_eq!(pose_of(&theirs, 7, Venue::Arena), None);
    }
}
