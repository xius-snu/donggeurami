//! What stands on the town's plots and does something (since 2026-11-25;
//! Hajun: "make some of the assets "functional" for the ones u can for
//! example the treadmill could actually move the player backward when they
//! get on it like a real treadmill so that they need to keep walking forward
//! to stay on it").
//!
//! - **The treadmill's belt** ([`run_belts`]) carries whoever stands on it
//!   toward its open end, at [`BELT_SPEED`], less than a walk: walking toward
//!   its handles keeps you on it, as on a real one, and standing still puts
//!   you off its end. Every body that moves under its own power (yours, the
//!   computer's people), before it moves this frame; everyone else sees it
//!   by where your body is, as ever. **Walked up to its front, you walk on
//!   the spot there** ([`walk_on_the_spot`], since 2026-11-27; Hajun: "right
//!   now when ur walking on the running machine and u reach the front, its
//!   like glitchy and u get glitched backward constantly make it more
//!   natural"): no further along it than [`BELT_FRONT`], where your head
//!   never meets the console over the handles, which threw you back a
//!   quarter of a metre at a time. Seen by anyone else, someone on a belt
//!   walks by how they go over the belt, not over the ground
//!   ([`belt_under`], `lobby::walk_from_motion`): on the spot at its front,
//!   walking.
//! - **The ceiling fan** ([`turn_fans`]) turns, all of it about the rod it
//!   hangs from, which is over its origin: its model turned under the piece,
//!   which stays where it was put.
//!
//! **What is used** (since 2026-11-27, Hajun's asks of that day): come up
//! to something there is a use for and a bubble stands over it, as one does
//! over somewhere to sit (`rest`), a thing in the world (`bubble`): a chest
//! over what keeps things (the cabinet, the dresser, the safe, the end table
//! and the barbecue), which a tap opens (`storage`); a bulb over the lamp
//! with shade and the chandelier, which a tap switches on or off ("toggle
//! light on off rather than day/night"; `lamps`); and a drop of water over
//! the bathroom sink, which a tap runs ("turn faucet on for 5 second, water
//! comes down the faucet, and water in the sink goes up and up until its
//! full, and then the faucet turns off automatically"; [`fill_sinks`]).
//! Anyone's, on any plot of your channel, as Hajun has it of the first
//! ("anyone can open it"); and everyone in the channel sees a light
//! switched and a sink fill (`bag`, the server's word).
//!
//! The rest of what came that day is still: one mesh each, as it was
//! downloaded, with no part of it to move by itself (the desk fan's blades
//! are its stand too). What lights up is `lamps::BY_KIND`, what is sat on
//! `rest`.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use roundtown_net::Ask;
use roundtown_net::bag as rules;

use crate::bag::{Bag, Ran, TheirWorkings};
use crate::bubble::{self, Bubble, Looks, Says, Tapped};
use crate::build::stand::{Stood, TheirPiece, owner_of};
use crate::build::{Bounds, Ghost, Mount, Piece};
use crate::customize::Customize;
use crate::drive::Driving;
use crate::hud::DialogUp;
use crate::island::{Islands, Venue};
use crate::net::Online;
use crate::rest::Resting;
use crate::{CloseUp, Dead, EYE_HEIGHT, Intent, Player};

/// How fast the belt goes, in metres a second: under a walk's 7, so that
/// walking on it gets you on toward its handles, slowly, and standing still
/// takes you off its end in half a second. Mine.
const BELT_SPEED: f32 = 5.0;
/// The belt, in the treadmill's own space (`mybuilds/newones/Treadmill.glb`,
/// set on its origin): between its side rails, from its open end at its
/// front (-Z) to its handles; and how far over its origin feet on it are, it
/// being 0.32 m up at its open end and 0.49 at its handles. Measured again
/// with the treadmill three times its download's size (2026-11-25, Hajun:
/// "i meant 3x"), each 1.2 times what it was at two and a half. Its open
/// end is at -1.84: the belt reaches a body's legs past it, so that whoever
/// it has carried to the lip, and stands on it by the edge of their legs,
/// is carried on off it, down to the floor; nobody on the floor in front
/// stands at its height.
const BELT_HALF_WIDTH: f32 = 0.66;
const BELT_FROM: f32 = -1.84 - crate::LEG_RADIUS;
const BELT_TO: f32 = 1.27;
const BELT_LOW: f32 = 0.18;
const BELT_HIGH: f32 = 0.72;
/// The furthest along the belt toward its handles anyone on it goes, in its
/// own space: walking on, they walk on the spot there, the belt carrying
/// them back as far as they walk (since 2026-11-27). Past about 1.0 the top
/// of a body, 1.6 m over a belt that rises toward the handles, meets the
/// underside of the console over them, and a body is kept out of a wall a
/// quarter of a metre square at a time (`island::LATTICE`): walking into it,
/// the belt and that threw a body back and forth by a quarter of a metre
/// eight times a second (seen in a trace: from 1.07 to 0.78, and to the
/// side). At 0.75 the body's front is 0.4 m short of the console, and its
/// top 2 cm under it. Mine.
const BELT_FRONT: f32 = 0.75;
/// How fast the ceiling fan turns, in radians a second: about once in two
/// seconds. Mine.
const FAN_TURN: f32 = 3.0;

/// How near something there is a use for you have to be for its bubble to
/// come up, in metres from the nearest edge of it along the ground, as near
/// as somewhere to sit (`rest`); how near to keep it, so that it does not
/// flicker at the edge of that; and for a chandelier, which hangs over you,
/// how near its edge, its bubble being under it.
const REACH: f32 = 1.5;
const KEEP_REACH: f32 = 1.9;
const OVERHEAD_REACH: f32 = 2.2;
/// How far under the foot of something there is a use for your feet may be,
/// it standing on a table or a desk, and how far over it; and how far under
/// a chandelier's foot your feet may be, a storey's height. Mine.
const UNDER: f32 = 2.2;
const OVER: f32 = 1.2;
const UNDER_THE_CEILING: f32 = 6.0;
/// How far over the top of what it stands over the point of its bubble is,
/// and how far under the foot of a chandelier: far enough for the whole
/// bubble to be under it, 0.71 m over its point.
const OVER_THE_TOP: f32 = 0.3;
const UNDER_THE_FOOT: f32 = 0.85;

/// The sink's basin (`mybuilds/newones/Bathroom Sink.glb`, the game's
/// space, measured off it): between its walls across and from its front to
/// its back, under a rim 1.028 m up. Its floor is the hollow round the
/// drain, 0.763 m up and 0.352 by 0.313 m, the flat back of the floor at
/// 0.817 m from its back to 0.083 m behind its middle, and from there
/// sloping up to 0.922 m at its front. It is full 2.8 cm under the rim.
const BASIN_X: (f32, f32) = (-1.426, -0.370);
const BASIN_Z: (f32, f32) = (-0.659, 0.396);
const HOLLOW: f32 = 0.763;
const HOLLOW_AREA: f32 = 0.352 * 0.313;
const FLAT: f32 = 0.817;
const FLAT_FROM: f32 = 0.083;
const SLOPE_TOP: f32 = 0.922;
const FULL: f32 = 1.0;
/// Where the water comes out of the tap, under the end of its spout, and
/// how thick it comes: as thick as three quarters of the spout (a third
/// was a thin line from a step away).
const SPOUT: Vec3 = Vec3::new(-0.898, 1.2786, 0.145);
const STREAM_RADIUS: f32 = 0.024;
/// How long the tap runs, filling the basin (Hajun's five seconds); how
/// long it stands full; and how long it takes to drain away, so that it may
/// be run again (mine, as is that it drains at all). The three are the
/// server's [`rules::SINK_SECS`], one run at a time being told.
const FILL_SECS: f32 = 5.0;
const HOLD_SECS: f32 = 4.0;
const DRAIN_SECS: f32 = 3.0;
/// How fast water falls, in metres a second, every second: the last of it
/// out of the tap as it is turned off, and the first as it is turned on.
const FALLS: f32 = 9.8;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, make_water).add_systems(
        Update,
        (
            run_belts.after(crate::read_player_input).before(crate::move_bodies),
            walk_on_the_spot.after(crate::move_bodies).before(crate::follow_camera),
            turn_fans,
            keep_switches.after(crate::bag::BagSystems),
            offer.after(crate::move_bodies).before(bubble::BubbleSystems),
            use_it.after(bubble::Taps),
            (hear_runs, fill_sinks).chain().after(use_it).after(crate::bag::BagSystems),
        ),
    );
}

// ------------------------------------------------------------ the treadmill

/// Whether feet at `at`, in a treadmill's own space, are on its belt.
fn on_the_belt(at: Vec3) -> bool {
    at.x.abs() <= BELT_HALF_WIDTH && (BELT_FROM..=BELT_TO).contains(&at.z) && (BELT_LOW..=BELT_HIGH).contains(&at.y)
}

/// Every treadmill there is, where it stands, on its island.
fn treadmills<'a>(pieces: impl Iterator<Item = (&'a Piece, &'a Transform, &'a Venue)>) -> Vec<(Transform, Venue)> {
    pieces.filter(|(piece, ..)| piece.def.kind == "treadmill").map(|(_, place, venue)| (*place, *venue)).collect()
}

/// How the belt of one of `belts` carries feet at `at` on `venue`, in
/// metres a second along the ground, if they are on one: toward its open
/// end. What someone's walk is read against, seen by anyone else
/// (`lobby::walk_from_motion`).
pub(crate) fn belt_under(belts: &[(Transform, Venue)], at: Vec3, venue: Venue) -> Option<Vec3> {
    belts
        .iter()
        .find(|(belt, on)| *on == venue && on_the_belt(belt.compute_affine().inverse().transform_point3(at)))
        .map(|(belt, _)| belt.rotation * Vec3::NEG_Z * BELT_SPEED)
}

/// Every treadmill there is, for whoever reads a walk against its belt.
pub(crate) fn belts(pieces: &Query<(&Piece, &Transform, &Venue), Without<Ghost>>) -> Vec<(Transform, Venue)> {
    treadmills(pieces.iter())
}

/// Carries everyone standing on a treadmill's belt toward its open end, as
/// far as the belt goes this frame: before they move under their own power
/// (`move_bodies`), which keeps them out of whatever they are carried into.
/// Only whoever is on a belt is written.
#[allow(clippy::type_complexity)]
fn run_belts(
    time: Res<Time>,
    pieces: Query<(&Piece, &Transform, &Venue), (Without<Ghost>, Without<Intent>)>,
    mut bodies: Query<(&mut Transform, &Venue), (With<Intent>, Without<Piece>, Without<Resting>, Without<Dead>)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let belts = treadmills(pieces.iter());
    if belts.is_empty() {
        return;
    }
    for (mut body, &venue) in &mut bodies {
        if let Some(carries) = belt_under(&belts, body.translation, venue) {
            body.translation += carries * dt;
        }
    }
}

/// Where along the belt a body at `own`, in its treadmill's own space, is
/// to be: no further toward its handles than [`BELT_FRONT`].
fn short_of_the_front(own: Vec3) -> Option<Vec3> {
    (own.z > BELT_FRONT).then(|| own.with_z(BELT_FRONT))
}

/// Keeps everyone on a belt from walking further along it than its front
/// ([`BELT_FRONT`]): walking on, they walk on the spot there, the belt
/// carrying them back as far as they go (Hajun: "make it more natural").
/// Once they have moved this frame, and before the camera follows them;
/// only whoever is past it is written.
#[allow(clippy::type_complexity)]
fn walk_on_the_spot(
    pieces: Query<(&Piece, &Transform, &Venue), (Without<Ghost>, Without<Intent>)>,
    mut bodies: Query<(&mut Transform, &Venue), (With<Intent>, Without<Piece>, Without<Resting>, Without<Dead>)>,
) {
    let belts = treadmills(pieces.iter());
    if belts.is_empty() {
        return;
    }
    for (mut body, &venue) in &mut bodies {
        let Some((belt, _)) = belts
            .iter()
            .find(|(belt, on)| *on == venue && on_the_belt(belt.compute_affine().inverse().transform_point3(body.translation)))
        else {
            continue;
        };
        let own = belt.compute_affine().inverse().transform_point3(body.translation);
        if let Some(kept) = short_of_the_front(own) {
            body.translation = belt.transform_point(kept);
        }
    }
}

/// The turn of a ceiling fan's model under its piece, as its scene made it.
#[derive(Component)]
struct Fan(Quat);

/// Turns every ceiling fan about the rod it hangs from: each part of its
/// scene directly under the piece, about the piece's upright.
#[allow(clippy::type_complexity)]
fn turn_fans(
    time: Res<Time>,
    fans: Query<(&Piece, &Children)>,
    mut parts: Query<(Entity, &mut Transform, Option<&Fan>), Without<Piece>>,
    mut commands: Commands,
) {
    let spin = Quat::from_rotation_y((time.elapsed_secs() * FAN_TURN) % std::f32::consts::TAU);
    for (piece, children) in &fans {
        if piece.def.kind != "ceiling_fan" {
            continue;
        }
        for &child in children {
            let Ok((part, mut place, fan)) = parts.get_mut(child) else {
                continue;
            };
            let rest = match fan {
                Some(&Fan(rest)) => rest,
                None => {
                    commands.entity(part).insert(Fan(place.rotation));
                    place.rotation
                }
            };
            place.rotation = spin * rest;
        }
    }
}

// --------------------------------------------------------------- the uses

/// What something standing on a plot is used for, by whoever comes up to
/// it: nothing, for most.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Use {
    /// Opened, to put things in and take them out (`storage`).
    Open,
    /// Switched on or off: a light.
    Switch,
    /// Its tap run: the sink.
    Run,
}

impl Use {
    /// What a piece of `kind` is used for, if anything.
    pub(crate) fn of(kind: &str) -> Option<Self> {
        if rules::keeps(kind).is_some() {
            Some(Use::Open)
        } else if rules::switched(kind) {
            Some(Use::Switch)
        } else if kind == rules::SINK {
            Some(Use::Run)
        } else {
            None
        }
    }

    /// What is drawn in its bubble.
    fn says(self) -> Says {
        match self {
            Use::Open => Says::Store,
            Use::Switch => Says::Light,
            Use::Run => Says::Water,
        }
    }
}

/// A bubble over something to use: this piece.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
struct UseBubble(Entity);

/// A light switched by hand ([`rules::SWITCHED`]), and whether it is on,
/// as its owner's plot's workings have it (`bag`): what `lamps` lights it
/// by.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Lit(pub(crate) bool);

/// Says of every light switched by hand that stands on a plot whether it is
/// on ([`Lit`]), as what works there says: yours out of your own bag, and
/// everyone else's as the server said of their plot. As either changes, and
/// as such a light is stood. Only what differs is written.
#[allow(clippy::type_complexity)]
fn keep_switches(
    bag: Res<Bag>,
    theirs: Res<TheirWorkings>,
    stood: Query<(), Or<(Added<Stood>, Added<TheirPiece>)>>,
    mut lights: Query<(Entity, &Piece, Option<&Stood>, Option<&TheirPiece>, Option<&mut Lit>)>,
    mut commands: Commands,
) {
    if !(bag.is_changed() || theirs.is_changed() || !stood.is_empty()) {
        return;
    }
    for (light, piece, mine, their, lit) in &mut lights {
        if !rules::switched(piece.def.kind) {
            continue;
        }
        let on = owner_of(mine, their).is_some_and(|(who, id)| theirs.of(&bag, who).is_some_and(|works| works.is_lit(id)));
        match lit {
            Some(mut lit) => {
                lit.set_if_neq(Lit(on));
            }
            None => {
                commands.entity(light).insert(Lit(on));
            }
        }
    }
}

/// How far `at` is from the edge of a piece that stands as `piece` does and
/// is as big as `bounds`, along the ground: nothing, over it.
fn off_the_edge(piece: &Transform, bounds: &Bounds, at: Vec3) -> f32 {
    let own = piece.compute_affine().inverse().transform_point3(at);
    let nearest = own.clamp(bounds.low, bounds.high);
    (own.xz() - nearest.xz()).length()
}

/// Where the point of the bubble over a piece is, in its own space: over
/// the middle of its top; and under a chandelier, which hangs over you,
/// under the middle of its foot.
fn tip_of(bounds: &Bounds, overhead: bool) -> Vec3 {
    let middle = (bounds.low + bounds.high) * 0.5;
    if overhead {
        Vec3::new(middle.x, bounds.low.y - UNDER_THE_FOOT, middle.z)
    } else {
        Vec3::new(middle.x, bounds.high.y + OVER_THE_TOP, middle.z)
    }
}

/// Whether someone with their feet at `you` is near enough to use a piece
/// that stands as `piece` does and is as big as `bounds`: on its floor, or
/// under it if it hangs `overhead`, and within reach of its edge along the
/// ground, further `held`, its bubble being up already.
fn near_enough(you: Vec3, piece: &Transform, bounds: &Bounds, overhead: bool, held: bool) -> bool {
    let foot = piece.translation.y;
    let (reach, level) = if overhead {
        (OVERHEAD_REACH, you.y < foot && foot - you.y <= UNDER_THE_CEILING)
    } else {
        (if held { KEEP_REACH } else { REACH }, you.y >= foot - UNDER && you.y <= foot + OVER)
    };
    level
        && you.xz().distance(piece.translation.xz()) <= reach + bounds.low.xz().length().max(bounds.high.xz().length())
        && off_the_edge(piece, bounds, you) <= reach
}

/// Keeps a bubble over everything near you there is a use for, and takes
/// away the rest: in the town, on your feet, out of your car and out of edit
/// mode, with nothing else up; and not over a sink whose tap is running.
/// Each in sight of your eyes.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn offer(
    me: Query<(&Transform, &Venue, Has<Resting>), (With<Player>, Without<Dead>)>,
    pieces: Query<(Entity, &Piece, &Transform, &Venue, &Bounds, &InheritedVisibility, Option<&Stood>, Option<&TheirPiece>, Has<Running>), Without<Ghost>>,
    bubbles: Query<(Entity, &UseBubble)>,
    (islands, looks): (Res<Islands>, Option<Res<Looks>>),
    (dialog, close_up, driving, customize): (Res<DialogUp>, Res<CloseUp>, Res<Driving>, Res<Customize>),
    mut commands: Commands,
) {
    let mut wanted: Vec<(Entity, Use, Vec3)> = Vec::new();
    let you = me
        .single()
        .ok()
        .filter(|&(_, &venue, rests)| venue == Venue::Town && !rests && !dialog.0 && !close_up.busy() && !driving.on() && !customize.on());
    if let (Some((you, ..)), Some(island)) = (you, islands.get(Venue::Town)) {
        for (piece, of, place, &venue, bounds, shown, mine, theirs, running) in &pieces {
            let Some(using) = Use::of(of.def.kind) else {
                continue;
            };
            if venue != Venue::Town || !shown.get() || running || owner_of(mine, theirs).is_none() {
                continue;
            }
            let overhead = of.def.mount == Mount::Ceiling;
            let held = bubbles.iter().any(|(_, over)| over.0 == piece);
            if !near_enough(you.translation, place, bounds, overhead, held) {
                continue;
            }
            let tip = place.transform_point(tip_of(bounds, overhead));
            let eyes = you.translation + Vec3::Y * EYE_HEIGHT;
            if island.sweep(eyes, tip + Vec3::Y * bubble::RADIUS, 0.05).is_none() {
                wanted.push((piece, using, tip));
            }
        }
    }
    for (bubble, over) in &bubbles {
        if !wanted.iter().any(|&(piece, ..)| piece == over.0) {
            commands.entity(bubble).despawn();
        }
    }
    let Some(looks) = looks else {
        return;
    };
    for (piece, using, tip) in wanted {
        if bubbles.iter().any(|(_, over)| over.0 == piece) {
            continue;
        }
        commands.spawn((UseBubble(piece), looks.bubble(using.says(), tip, Venue::Town)));
    }
}

/// A tap on a bubble over something to use, or E with it the nearest bubble
/// (`bubble::tap`): what keeps things is opened (`storage`); a light is
/// switched the other way; a sink's tap is run, here at once and, online,
/// for everyone in the channel (`bag::Bag::run`).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn use_it(
    mut tapped: MessageReader<Tapped>,
    time: Res<Time>,
    bubbles: Query<&UseBubble, With<Bubble>>,
    pieces: Query<(&Piece, Option<&Stood>, Option<&TheirPiece>, Option<&Lit>, Has<Running>)>,
    online: Res<Online>,
    mut bag: ResMut<Bag>,
    (mut asks, mut opens): (MessageWriter<Ask>, MessageWriter<crate::storage::Open>),
    mut commands: Commands,
) {
    let me = online.me().map(|me| me.id);
    for &Tapped(bubble) in tapped.read() {
        let Ok(&UseBubble(piece)) = bubbles.get(bubble) else {
            continue;
        };
        let Ok((of, mine, theirs, lit, running)) = pieces.get(piece) else {
            continue;
        };
        let Some((who, id)) = owner_of(mine, theirs) else {
            continue;
        };
        match Use::of(of.def.kind) {
            Some(Use::Open) => {
                opens.write(crate::storage::Open(piece));
            }
            Some(Use::Switch) => {
                let on = !lit.is_some_and(|lit| lit.0);
                bag.switch(me, &mut asks, who, id, on);
            }
            Some(Use::Run) if !running => {
                bag.run(me, &mut asks, who, id);
                commands.entity(piece).insert(Running { since: time.elapsed_secs() });
            }
            Some(Use::Run) | None => {}
        }
    }
}

// --------------------------------------------------------------- the sink

/// A sink whose tap has been run, and when, by this device's own seconds:
/// it fills, stands full and drains ([`fill_sinks`]).
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Running {
    since: f32,
}

/// What is drawn of the water in a sink while it runs: one thing in the
/// world for each sink, on the town's island (`island::show_where_i_am`)
/// and on no piece, so that nothing of it is part of the plot's shape,
/// where the sink stands; the water in its basin and the water coming out of
/// its tap under it, each shown while there is any.
#[derive(Component)]
struct Water(Entity);
#[derive(Component)]
struct Basin;
#[derive(Component)]
struct Stream;

/// What the water is drawn with: a square, laid flat and made as big as the
/// basin, and a round stick, made as long as the stream and as thick, each
/// see-through; made once.
#[derive(Resource)]
struct WaterLooks {
    flat: Handle<Mesh>,
    stick: Handle<Mesh>,
    still: Handle<StandardMaterial>,
    falling: Handle<StandardMaterial>,
}

fn make_water(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    // A pale blue, see-through over the basin's floor, and a little paler
    // and less see-through where it falls. Mine.
    let water = |red: f32, green: f32, blue: f32, alpha: f32| StandardMaterial {
        base_color: Color::srgba(red, green, blue, alpha),
        perceptual_roughness: 0.08,
        reflectance: 0.6,
        alpha_mode: AlphaMode::Blend,
        ..default()
    };
    commands.insert_resource(WaterLooks {
        flat: meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.5))),
        stick: meshes.add(Cylinder::new(1.0, 1.0)),
        still: materials.add(water(0.55, 0.78, 0.95, 0.62)),
        falling: materials.add(water(0.85, 0.94, 1.0, 0.85)),
    });
}

/// How much water there is in the sink's basin with it up to `level`, in
/// cubic metres: in the hollow round the drain, then over the flat back of
/// its floor and up the slope to its front, then over all of it.
fn volume(level: f32) -> f32 {
    let wide = BASIN_X.1 - BASIN_X.0;
    let hollow = HOLLOW_AREA * (level.min(FLAT) - HOLLOW).max(0.0);
    if level <= FLAT {
        return hollow;
    }
    let (back, slope) = (BASIN_Z.1 - FLAT_FROM, FLAT_FROM - BASIN_Z.0);
    let high = level.min(SLOPE_TOP) - FLAT;
    let share = high / (SLOPE_TOP - FLAT);
    let up_the_slope = wide * (back * high + slope * (SLOPE_TOP - FLAT) * share * share * 0.5);
    let over_all = wide * (BASIN_Z.1 - BASIN_Z.0) * (level - SLOPE_TOP).max(0.0);
    hollow + up_the_slope + over_all
}

/// How high the water stands with `share` of a full basin in it: as the
/// tap runs at the one rate, it rises quickly in the hollow and slowly once
/// it is over the whole floor.
fn level_at(share: f32) -> f32 {
    let wanted = volume(FULL) * share.clamp(0.0, 1.0);
    let (mut low, mut high) = (HOLLOW, FULL);
    for _ in 0..24 {
        let middle = (low + high) * 0.5;
        if volume(middle) < wanted {
            low = middle;
        } else {
            high = middle;
        }
    }
    (low + high) * 0.5
}

/// How a sink is, `secs` after its tap was run: how high the water stands,
/// and where the water coming out of the tap reaches between, top and
/// bottom, if any is: running five seconds and filling the basin; the last
/// of it falling as it is turned off; full; draining away.
fn sink_at(secs: f32) -> (f32, Option<(f32, f32)>) {
    if secs < FILL_SECS {
        let level = level_at(secs / FILL_SECS);
        // The first of it falls from the tap.
        let head = SPOUT.y - 0.5 * FALLS * secs * secs;
        return (level, Some((SPOUT.y, head.max(level.max(HOLLOW)))));
    }
    let after = secs - FILL_SECS;
    if after < HOLD_SECS {
        let tail = SPOUT.y - 0.5 * FALLS * after * after;
        return (FULL, (tail > FULL).then_some((tail, FULL)));
    }
    let draining = ((after - HOLD_SECS) / DRAIN_SECS).clamp(0.0, 1.0);
    (level_at(1.0 - draining), None)
}

/// A sink run by anyone in your channel, as the server says (`bag::Ran`):
/// filling from now, unless it is already, run here a moment ago.
#[allow(clippy::type_complexity)]
fn hear_runs(
    mut ran: MessageReader<Ran>,
    time: Res<Time>,
    sinks: Query<(Entity, &Piece, Option<&Stood>, Option<&TheirPiece>, Has<Running>)>,
    mut commands: Commands,
) {
    for &Ran { who, id } in ran.read() {
        let found = sinks
            .iter()
            .find(|(_, piece, mine, theirs, _)| piece.def.kind == rules::SINK && owner_of(*mine, *theirs) == Some((who, id)));
        if let Some((sink, .., false)) = found {
            commands.entity(sink).insert(Running { since: time.elapsed_secs() });
        }
    }
}

/// Fills every sink whose tap has been run, as far as it has got
/// ([`sink_at`]): the water in its basin a flat sheet over the whole of it,
/// as high as it stands, the basin's floor and walls hiding what of it is
/// under them; and the water from its tap a stick from the spout down to
/// it. Gone once the sink has drained, or is gone itself. Only what has
/// changed is written.
#[allow(clippy::type_complexity)]
fn fill_sinks(
    time: Res<Time>,
    looks: Option<Res<WaterLooks>>,
    sinks: Query<(Entity, &Transform, &Running), Without<Water>>,
    mut waters: Query<(Entity, &Water, &mut Transform, &Children), Without<Running>>,
    mut basins: Query<(&mut Transform, &mut Visibility), (With<Basin>, Without<Water>, Without<Stream>, Without<Running>)>,
    mut streams: Query<(&mut Transform, &mut Visibility), (With<Stream>, Without<Water>, Without<Basin>, Without<Running>)>,
    mut commands: Commands,
) {
    let Some(looks) = looks else {
        return;
    };
    let now = time.elapsed_secs();
    // What is no longer of a sink that runs goes, with all of it.
    for (water, &Water(sink), ..) in &waters {
        if !sinks.contains(sink) {
            commands.entity(water).despawn();
        }
    }
    for (sink, place, running) in &sinks {
        let secs = now - running.since;
        if secs >= rules::SINK_SECS as f32 {
            commands.entity(sink).remove::<Running>();
            continue;
        }
        let (level, falling) = sink_at(secs);
        let middle = Vec3::new((BASIN_X.0 + BASIN_X.1) * 0.5, level, (BASIN_Z.0 + BASIN_Z.1) * 0.5);
        let size = Vec3::new(BASIN_X.1 - BASIN_X.0, 1.0, BASIN_Z.1 - BASIN_Z.0);
        let sheet = Transform::from_translation(middle).with_scale(size);
        let stick = falling.map(|(top, bottom)| {
            let long = (top - bottom).max(0.001);
            Transform::from_translation(SPOUT.with_y((top + bottom) * 0.5)).with_scale(Vec3::new(STREAM_RADIUS, long, STREAM_RADIUS))
        });
        let shown = |seen: bool| if seen { Visibility::Inherited } else { Visibility::Hidden };
        let Some((_, _, mut at, children)) = waters.iter_mut().find(|(_, water, ..)| water.0 == sink) else {
            commands.spawn((
                Water(sink),
                Name::new("Sink water"),
                Venue::Town,
                *place,
                Visibility::Hidden,
                children![
                    (Basin, Mesh3d(looks.flat.clone()), MeshMaterial3d(looks.still.clone()), sheet, Visibility::Hidden, NotShadowCaster),
                    (Stream, Mesh3d(looks.stick.clone()), MeshMaterial3d(looks.falling.clone()), stick.unwrap_or_default(), Visibility::Hidden, NotShadowCaster),
                ],
            ));
            continue;
        };
        at.set_if_neq(*place);
        for &part in children {
            if let Ok((mut at, mut seen)) = basins.get_mut(part) {
                at.set_if_neq(sheet);
                seen.set_if_neq(shown(level > HOLLOW + 0.002));
            }
            if let Ok((mut at, mut seen)) = streams.get_mut(part) {
                if let Some(stick) = stick {
                    at.set_if_neq(stick);
                }
                seen.set_if_neq(shown(stick.is_some()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_belt_is_between_its_rails_from_its_open_end_to_its_handles() {
        // On it, anywhere along it, at its height.
        for at in [Vec3::new(0.0, 0.36, -1.68), Vec3::new(0.6, 0.48, 1.2), Vec3::new(-0.6, 0.42, 0.0)] {
            assert!(on_the_belt(at), "{at}");
        }
        // Beside it, past its ends, on the floor by it, or jumping over it.
        for at in [Vec3::new(0.84, 0.42, 0.0), Vec3::new(0.0, 0.36, -2.2), Vec3::new(0.0, 0.48, 1.44), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.44, 0.0)] {
            assert!(!on_the_belt(at), "{at}");
        }
        // Stood on its open end's lip by the edge of the legs, just past
        // it: still carried, off it; on the floor there, not.
        assert!(on_the_belt(Vec3::new(0.0, 0.31, -1.95)));
        assert!(!on_the_belt(Vec3::new(0.0, 0.0, -1.95)));
        // Slower than a walk, so that walking on it gets one on.
        assert!(BELT_SPEED < 7.0 && BELT_SPEED > 3.0);
    }

    #[test]
    fn the_belt_carries_whoever_stands_on_it_toward_its_open_end_and_nobody_else() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        {
            let mut time = world.resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(100));
        }
        let treadmill = crate::build::PIECES.iter().find(|def| def.kind == "treadmill").unwrap();
        // A treadmill turned a quarter, its front toward the world's -X.
        let turned = Transform::from_xyz(10.0, 0.0, 5.0).with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
        world.spawn((Piece { def: treadmill }, turned, Venue::Town));
        let on = world.spawn((Transform::from_xyz(10.0, 0.35, 5.0), Venue::Town, Intent::default())).id();
        let off = world.spawn((Transform::from_xyz(12.0, 0.0, 5.0), Venue::Town, Intent::default())).id();
        world.run_system_once(run_belts).unwrap();
        // Half a metre in a tenth of a second, toward its open end.
        let moved = world.get::<Transform>(on).unwrap().translation;
        assert!(moved.distance(Vec3::new(9.5, 0.35, 5.0)) < 1e-4, "{moved}");
        assert_eq!(world.get::<Transform>(off).unwrap().translation, Vec3::new(12.0, 0.0, 5.0));
        // Seen by someone else, the belt carries them as fast and that way.
        let belts = vec![(turned, Venue::Town)];
        let carries = belt_under(&belts, Vec3::new(10.0, 0.35, 5.0), Venue::Town).unwrap();
        assert!(carries.distance(Vec3::new(-5.0, 0.0, 0.0)) < 1e-4, "{carries}");
        assert!(belt_under(&belts, Vec3::new(12.0, 0.0, 5.0), Venue::Town).is_none());
        assert!(belt_under(&belts, Vec3::new(10.0, 0.35, 5.0), Venue::Arena).is_none());
    }

    #[test]
    fn walked_up_to_its_front_whoever_is_on_the_belt_walks_on_the_spot_there() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        let treadmill = crate::build::PIECES.iter().find(|def| def.kind == "treadmill").unwrap();
        let turned = Transform::from_xyz(10.0, 0.0, 5.0).with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
        world.spawn((Piece { def: treadmill }, turned, Venue::Town));
        // Walked to 0.9 along it, as far as a frame's walk and the belt
        // leave someone: back to 0.75, at the height they were and as far
        // across; one short of it is left where they are.
        let past = world.spawn((Transform::from_translation(turned.transform_point(Vec3::new(0.3, 0.47, 0.9))), Venue::Town, Intent::default())).id();
        let short = turned.transform_point(Vec3::new(-0.2, 0.45, 0.5));
        let before = world.spawn((Transform::from_translation(short), Venue::Town, Intent::default())).id();
        world.run_system_once(walk_on_the_spot).unwrap();
        let own = turned.compute_affine().inverse().transform_point3(world.get::<Transform>(past).unwrap().translation);
        assert!(own.distance(Vec3::new(0.3, 0.47, BELT_FRONT)) < 1e-4, "{own}");
        assert_eq!(world.get::<Transform>(before).unwrap().translation, short);
        // Short of where the console over its handles meets the top of a
        // body (1.0 along it), with room to spare.
        assert!(BELT_FRONT + 0.2 < 1.0 && BELT_FRONT > 0.0);
        assert_eq!(short_of_the_front(Vec3::new(0.0, 0.47, 0.7)), None);
    }

    #[test]
    fn what_is_used_is_what_keeps_things_the_two_lights_and_the_sink() {
        for kind in ["cabinet", "dresser", "safe", "end_table", "barbecue"] {
            assert_eq!(Use::of(kind), Some(Use::Open), "{kind}");
        }
        assert_eq!(Use::of("lamp_with_shade"), Some(Use::Switch));
        assert_eq!(Use::of("chandelier"), Some(Use::Switch));
        assert_eq!(Use::of("bathroom_sink"), Some(Use::Run));
        assert_eq!(Use::of("treadmill"), None);
        assert_eq!(Use::of("streetlamp"), None);
        // Each a piece there is, and the chandelier the one that hangs.
        for kind in ["cabinet", "dresser", "safe", "end_table", "barbecue", "lamp_with_shade", "chandelier", "bathroom_sink"] {
            assert!(crate::build::PIECES.iter().any(|def| def.kind == kind), "{kind}");
        }
        let hung = crate::build::PIECES.iter().find(|def| def.kind == "chandelier").unwrap();
        assert_eq!(hung.mount, Mount::Ceiling);
    }

    #[test]
    fn a_bubble_comes_up_only_near_what_is_used_on_its_floor_or_under_what_hangs() {
        // A cabinet off at (10, 0.27, 20), 1.5 m across and 0.9 deep.
        let cabinet = Transform::from_xyz(10.0, 0.27, 20.0);
        let size = Bounds {
            low: Vec3::new(-0.74, 0.0, -0.04),
            high: Vec3::new(0.74, 1.48, 0.86),
        };
        let at = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
        assert!(near_enough(at(10.0, 0.27, 18.9), &cabinet, &size, false, false));
        assert!(!near_enough(at(10.0, 0.27, 18.0), &cabinet, &size, false, false));
        // Up already, it stays a little further off.
        assert!(near_enough(at(10.0, 0.27, 18.3), &cabinet, &size, false, true));
        // Not from the floor over it, nor one far under it; a lamp on a desk
        // is reached from the floor.
        assert!(!near_enough(at(10.0, 3.6, 18.9), &cabinet, &size, false, false));
        let on_a_desk = Transform::from_xyz(10.0, 1.6, 20.0);
        assert!(near_enough(at(10.0, 0.27, 18.9), &on_a_desk, &size, false, false));
        // A chandelier hung 4 m up: from under it, not from beside it on
        // the floor over it.
        let chandelier = Transform::from_xyz(0.0, 4.0, 0.0);
        let hung = Bounds {
            low: Vec3::new(-1.14, 0.0, -1.15),
            high: Vec3::new(1.14, 2.0, 1.13),
        };
        assert!(near_enough(at(0.5, 0.27, 2.5), &chandelier, &hung, true, false));
        assert!(!near_enough(at(0.5, 4.27, 2.5), &chandelier, &hung, true, false));
        assert!(!near_enough(at(0.5, 0.27, 5.0), &chandelier, &hung, true, false));
        // Its bubble is under it, the whole of the bubble; anything else's
        // over it.
        assert!(tip_of(&hung, true).y + 0.71 < hung.low.y);
        assert!(tip_of(&size, false).y > size.high.y);
    }

    #[test]
    fn the_sink_fills_in_five_seconds_stands_full_and_drains_and_its_tap_runs_while_it_fills() {
        // The three together are what the server waits before telling of
        // another run.
        assert!((FILL_SECS + HOLD_SECS + DRAIN_SECS - rules::SINK_SECS as f32).abs() < 1e-6);
        // Empty to begin with, and full at the end of the five seconds,
        // rising all the while.
        assert!((level_at(0.0) - HOLLOW).abs() < 1e-3 && (level_at(1.0) - FULL).abs() < 1e-3);
        let mut last = 0.0;
        for step in 0..=50 {
            let (level, falling) = sink_at(step as f32 * 0.1);
            assert!(level >= last - 1e-4, "{step}");
            last = level;
            // The tap runs from the spout down to the water, or the floor,
            // the first of it falling from the spout as it is turned on.
            let (top, bottom) = falling.unwrap();
            assert_eq!(top, SPOUT.y);
            assert!(bottom >= level.max(HOLLOW) - 1e-4 && bottom <= top, "{step}: {bottom} {top}");
            // It falls the half metre to the basin in three tenths of a
            // second.
            if step > 3 {
                assert!((bottom - level.max(HOLLOW)).abs() < 1e-4, "{step}: reaches the water");
            }
        }
        // It rises fastest in the hollow round the drain, and slowest over
        // the whole floor.
        let rise = |at: f32| sink_at(at + 0.1).0 - sink_at(at).0;
        assert!(rise(0.0) > rise(4.5));
        // As it is turned off the last of the water falls, and it is gone
        // in a moment; full meanwhile.
        let (level, tail) = sink_at(FILL_SECS + 0.05);
        assert_eq!(level, FULL);
        assert!(tail.is_some_and(|(top, bottom)| top < SPOUT.y && bottom == FULL));
        assert_eq!(sink_at(FILL_SECS + 1.0), (FULL, None));
        // Draining, lower and lower, until there is none.
        let (half, none) = (sink_at(FILL_SECS + HOLD_SECS + DRAIN_SECS * 0.5), sink_at(rules::SINK_SECS as f32 - 0.001));
        assert!(half.0 < FULL && half.0 > HOLLOW && half.1.is_none());
        assert!(none.0 < HOLLOW + 0.01);
        // Full is under the basin's rim, which is 1.028 m up.
        assert!(FULL < 1.028);
    }

    #[test]
    fn the_basin_and_the_tap_are_where_the_sink_s_model_has_them() {
        // Read off the model as the game has it: the basin's floor under its
        // middle, its rim round it, and the spout's end over the hollow.
        let def = crate::build::PIECES.iter().find(|def| def.kind == rules::SINK).unwrap();
        let triangles: Vec<[Vec3; 3]> = crate::island::glb::read(&format!("{}/{}.glb", def.dir, def.file))
            .into_iter()
            .flat_map(|part| part.triangles)
            .collect();
        let top = |x: f32, z: f32| {
            triangles
                .iter()
                .filter_map(|[a, b, c]| {
                    let (a2, b2, c2, at) = (a.xz(), b.xz(), c.xz(), Vec2::new(x, z));
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
        };
        // The rim, just outside the basin, and its floor inside it.
        assert!(top(BASIN_X.0 - 0.05, 0.0).is_some_and(|rim| (rim - 1.028).abs() < 0.01));
        let floor = top((BASIN_X.0 + BASIN_X.1) * 0.5, BASIN_Z.0 + 0.02).unwrap();
        assert!((floor - SLOPE_TOP).abs() < 0.02, "{floor}");
        assert!(top(-0.8, 0.3).is_some_and(|back| (back - FLAT).abs() < 0.01 || (back - HOLLOW).abs() < 0.01));
        // The spout's end, over the hollow round the drain.
        let spout = top(SPOUT.x, SPOUT.z).unwrap();
        assert!(spout > SPOUT.y && spout < SPOUT.y + 0.1, "{spout}");
    }
}
