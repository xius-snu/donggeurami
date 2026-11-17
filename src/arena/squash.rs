//! Bridge Wars' buses run over whoever is in front of them (since
//! 2026-11-16; Hajun: "in the bridge wars minigame map, i want 2 buses to
//! go 2x normal speed around the road. and when a person is standing in
//! front of the moving bus, the bus 'squishes' that person, so the person
//! is squashed on the road for 10 second. they cannot move for 10second,
//! they are doing a 'flat on the ground with leg and arms extended outward
//! like a starfish' 'face towards the ground' emote with a 10s timer text
//! above the player, where they can be attacked by other players too
//! (being attacked doesnt reduce the timer) and they also take an initial
//! -20hp dmg. if they are already squished, they cannot be squished twice
//! even if another bus come, until they get up, and they can be squished
//! again. obviously u shouldnt be squished when u are on the side of the
//! bus trying to go in the bus from the side, only the front of the bus
//! should squish. this goes for ai too").
//!
//! The buses themselves are `car`'s (`car::Lapping`): the town's bus, one
//! to each of the four lanes of the ring of road round the pond in the
//! middle of the map since 2026-11-17 (two on its outside lane the day
//! before), going as the game has them (`roundtown_net::bridge_wars::Buses`,
//! [`steer_buses`]): online as the server sends it, offline as this device
//! drives them itself. **A bus stops short of a crossing while anyone of
//! the players is on it** (Hajun: "the bus will stop before the crosswalk
//! if there is a person on the crosswalk until they cross it ... It will
//! not stop for ppl who are not on the crosswalk but rather on the road
//! and go over them like now"), and none runs over anyone of the players
//! who is on one; a zombie is not stopped for (mine).
//!
//! * **Who says someone has been run over** ([`run_over`]). Your own
//!   device says it of you, online or off: it is where your body and the
//!   bus are at the very moment, as it is for a fall, and you are flat
//!   before the bus's floor has scooped you up; online it asks the server
//!   (`NetAsk::RunOver`), which takes the damage off you and has you flat
//!   for everyone. In a game played offline this device says it of
//!   everyone, the computer's players and the zombies with you, and takes
//!   the damage off them itself. Online, everyone else is the server's to
//!   say, or their own device's ([`follow_flat`], `Flattened`).
//! * **Only a bus's front** (`Bus::runs_over`): someone who was ahead of
//!   its front face a frame ago and whom the face has reached, within its
//!   width. Walking into its side you bump into it, or walk in at a
//!   doorway, as on the town's; riding it, in it or on its roof, you are
//!   carried.
//! * **Flat** (`crate::Squashed`): for `SQUASH_SECS`, ten seconds, lying
//!   on your face where you were with your arms and legs spread
//!   ([`lay_out`], `crate::squash_offset`), the seconds left written over
//!   you ([`count_down`]); you walk nowhere, jump, punch, build and use
//!   nothing; you are in nobody's way; a blow lands on you as on anyone,
//!   takes nothing off the time, and knocks you nowhere (mine: "they
//!   cannot move for 10second"); and no bus runs you over again until you
//!   are up. `SQUASH_DAMAGE`, 20, comes off your health as it happens,
//!   less what your shield keeps off, as a fireball's 20 does; with no
//!   more health than that, it kills you, which the chat says
//!   (`Feed::RunOver`).

use roundtown_net::Flattened;
use roundtown_net::bridge_wars::{BUS_ROAD, Buses, SQUASH_DAMAGE, SQUASH_SECS, crossing_at};

use super::*;
use crate::car::{ArenaBuses, Lapping};
use crate::clock::TownClock;
use crate::{Drinking, FLAT_BACK, FLAT_LIFT, FLAT_SQUASH, ForShow, Riding, Squashed, WalkBone};

/// How big the seconds left over someone lying flat are written, as a
/// share of the short side of the screen: bigger than a name over a head,
/// to be read at a glance from over the fight. How far from their middle
/// someone lying flat reaches, in metres, across them and along them, head
/// or feet, which what is written over them is clear of whichever way
/// they lie ([`far_end`]); how far up the screen the name and the bar
/// over anyone else reach from there, as such a share, the seconds
/// standing over those; and the gap left under them.
const CLOCK_TEXT: f32 = 4.0;
const FLAT_ACROSS: f32 = 0.55;
const FLAT_ALONG: f32 = 1.2;
const TAG_TALL: f32 = 4.0;
const CLOCK_GAP: f32 = 0.5;

/// The seconds left written over someone lying flat: whose.
#[derive(Component)]
struct FlatClock(Entity);

/// A body lying flat as it is drawn: which of its parts is its model, and
/// how that stood in it before it was laid down, to stand it up again by.
#[derive(Component)]
struct Laid {
    model: Entity,
    rest: Transform,
}

pub(super) fn plugin(app: &mut App) {
    // Before the buses are put where they are going.
    app.add_systems(PreUpdate, steer_buses.before(crate::car::drive));
    app.add_systems(
        Update,
        (
            // Once everyone has moved this frame, and before the fight's
            // dead and wounds are counted.
            (run_over, follow_flat, lie).chain().after(crate::move_bodies).before(reap),
            lay_out.after(lie),
            count_down.after(crate::follow_camera),
        ),
    );
}

/// Whether whoever is in `seat`, standing at `at` in a game played
/// offline, was killed by the bus that has just run them over, the damage
/// taken off `health`: and if so, dead as anyone is ([`Game::die`]), what
/// they held of sapphire and amethyst dropped round them on the road, and
/// the chat told.
fn hurt_offline(game: &mut Game, seat: usize, at: Vec3, health: &mut Health, fed: &mut MessageWriter<Fed>) -> Option<Dead> {
    let damage = game.outfits.through(seat, SQUASH_DAMAGE);
    game.outfits.reveal(seat);
    health.take(damage);
    if !health.gone() {
        return None;
    }
    if seat < ARENA_SEATS {
        fed.write(Fed(Feed::RunOver { seat: seat as u8 }));
    }
    game.run_down(seat, at);
    Some(game.die(seat))
}

/// How the buses are going in the game you are in ([`ArenaBuses`], since
/// 2026-11-17): online, as the server last sent it; offline, as this
/// device drives them, by the town's clock, each thrown round its lane as
/// the game is first looked at and stopping short of the next crossing
/// ahead of it that any of the players is on, for as long as they are
/// (`Buses::drive`): standing, not riding a bus, nor dead. Not a zombie.
#[allow(clippy::type_complexity)]
fn steer_buses(
    game: Option<ResMut<Game>>,
    town: Res<TownClock>,
    sent: Query<&Buses, (With<Remote>, Changed<Buses>)>,
    bodies: Query<(&Transform, &Venue, &Seat, Option<&Riding>), (With<WalkCycle>, Without<Dead>, Without<ForShow>)>,
    mut log: ResMut<ArenaBuses>,
) {
    let Some(mut game) = game else {
        return;
    };
    if game.online {
        for buses in &sent {
            log.0.take(buses);
        }
        return;
    }
    let clock = town.now();
    let mut crowded = [false; 4];
    for (place, &venue, seat, riding) in &bodies {
        if venue != Venue::Arena || seat.0 >= ARENA_SEATS || riding.is_some_and(|riding| riding.0.is_some()) {
            continue;
        }
        if let Some(crossing) = crossing_at(place.translation) {
            crowded[crossing] = true;
        }
    }
    let buses = game.buses.get_or_insert_with(|| Buses::new(clock, &mut || fastrand::f32()));
    buses.drive(clock, crowded);
    let buses = *buses;
    log.0.take(&buses);
}

/// Has each bus run over whoever its front has come to, of those this
/// device says it of: you, always; and in a game played offline everyone,
/// the computer's players and the zombies. Nobody riding a bus, nobody
/// dead, and nobody lying flat already.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn run_over(
    game: Option<ResMut<Game>>,
    buses: Query<&Lapping>,
    islands: Res<Islands>,
    mut bodies: Query<
        (
            Entity,
            &mut Transform,
            &Venue,
            &Seat,
            Option<&mut Health>,
            &mut PlayerJump,
            &mut Shove,
            Option<&mut Riding>,
            Has<Player>,
            Has<Remote>,
        ),
        (With<WalkCycle>, Without<Dead>, Without<Squashed>, Without<ForShow>),
    >,
    // Where each stood a frame ago, and whether it was riding a bus then.
    mut stood: Local<HashMap<Entity, (Vec3, bool)>>,
    (mut requests, mut fed): (MessageWriter<NetAsk>, MessageWriter<Fed>),
    mut bridging: ResMut<bridge::Bridging>,
    mut commands: Commands,
) {
    let Some(mut game) = game.filter(|game| game.playing()) else {
        stood.clear();
        return;
    };
    let buses: Vec<Lapping> = buses.iter().copied().collect();
    let mut here: HashMap<Entity, (Vec3, bool)> = HashMap::with_capacity(stood.len());
    for (body, mut transform, &venue, seat, health, mut jump, mut shove, riding, mine, remote) in &mut bodies {
        if venue != Venue::Arena {
            continue;
        }
        let now = transform.translation;
        let on = riding.as_ref().is_some_and(|riding| riding.0.is_some());
        let (was, aboard) = stood.get(&body).copied().unwrap_or((now, on));
        here.insert(body, (now, on));
        // Whose it is to say: yours is this device's, always; and in a
        // game it runs itself, everyone's.
        let ours = mine || (!game.online && !remote);
        // Riding one, in it or on its roof, is being carried by it: as
        // they were a frame ago, since in a long frame a bus's floor has
        // whoever its front came to scooped up already.
        if !ours || aboard {
            continue;
        }
        // On a crossing, one of the players is stopped for, not run over.
        if seat.0 < ARENA_SEATS && crossing_at(now).is_some() {
            continue;
        }
        let Some(bus) = buses.iter().find(|bus| bus.now.runs_over(&bus.then, was, now, PLAYER_RADIUS)) else {
            continue;
        };
        // Down on the road where they are, whatever they were at: out of
        // the air, and off whatever a knock was carrying them by.
        let ground = islands
            .get(Venue::Arena)
            .and_then(|map| map.floor(now.xz(), now.y + 1.0))
            .unwrap_or(BUS_ROAD);
        if transform.translation.y != ground {
            transform.translation.y = ground;
        }
        *jump = PlayerJump::default();
        *shove = Shove {
            pinned: true,
            ..default()
        };
        if let Some(mut riding) = riding {
            riding.set_if_neq(Riding(None));
        }
        info!("a bus ran seat {} over at {:.1}", seat.0, now.xz());
        // The plank you were building with is out of your arms.
        if mine {
            bridging.set_if_neq(bridge::Bridging(false));
        }
        if game.online {
            // The server's to take the damage off, and to tell everyone.
            requests.write(NetAsk::RunOver { bus: bus.bus as u8 });
            commands.entity(body).remove::<Drinking>().insert(Squashed { left: SQUASH_SECS });
            continue;
        }
        // (Nothing hurts a zombie: one a bus runs over is laid flat, and
        // no more.)
        let dead = health.filter(|_| seat.0 < ARENA_SEATS).and_then(|mut health| {
            let before = health.left;
            let dead = hurt_offline(&mut game, seat.0, now, &mut health, &mut fed);
            spawn_number(&mut commands, now, before - health.left);
            dead
        });
        match dead {
            Some(dead) => {
                shove.pinned = false;
                commands.entity(body).insert(dead);
            }
            None => {
                commands.entity(body).remove::<Drinking>().insert(Squashed { left: SQUASH_SECS });
            }
        }
    }
    *stood = here;
}

/// Online, lays flat and stands up everyone else as the server has them
/// (`Flattened`): whoever of the people and the computer's players in
/// your game a bus has run over, each for as long as the server's clock
/// says is left. You are your own device's to lay flat ([`run_over`]);
/// but if the server has you lying there for more than a moment yet and
/// this device does not, having been away when it happened, you lie down
/// for what is left of it.
#[allow(clippy::type_complexity)]
fn follow_flat(
    game: Option<Res<Game>>,
    clock: Res<TownClock>,
    flattened: Query<&Flattened, With<Remote>>,
    mut others: Query<(Entity, &Member, Option<&mut Squashed>, Has<Dead>), (With<Remote>, With<WalkCycle>, Without<Player>)>,
    me: Query<(Entity, Has<Squashed>, Has<Dead>), With<Player>>,
    mut commands: Commands,
) {
    let Some(game) = game.filter(|game| game.online && game.playing()) else {
        return;
    };
    let now = clock.now();
    let nobody = Flattened::default();
    let flat = flattened.single().unwrap_or(&nobody);
    let left_of = |seat: usize| flat.until(seat).map(|until| (until - now) as f32).filter(|left| *left > 0.0);
    for (body, member, squashed, dead) in &mut others {
        let seat = game.seats.iter().position(|seated| seated.id == member.id && member.id != 0);
        let left = seat.and_then(left_of).filter(|_| !dead);
        match (left, squashed) {
            (Some(left), Some(mut squashed)) => {
                if squashed.left != left {
                    squashed.left = left;
                }
            }
            (Some(left), None) => {
                commands.entity(body).insert(Squashed { left });
            }
            (None, Some(_)) => {
                commands.entity(body).remove::<Squashed>();
            }
            (None, None) => {}
        }
    }
    if let Ok((you, squashed, dead)) = me.single()
        && !squashed
        && !dead
        && let Some(left) = left_of(game.me).filter(|left| *left > 1.0)
    {
        commands.entity(you).insert(Squashed { left });
    }
}

/// Keeps everyone lying flat where they lie, and stands them up once
/// their time is: counted down here for you and for everyone this device
/// moves; someone else online is up when the server says ([`follow_flat`]).
/// Nothing a blow would have carried them by is kept for when they are up,
/// and how long they reel from one is counted here, nobody else counting
/// it while they lie. And nobody lies flat out of a fight, or dead.
#[allow(clippy::type_complexity)]
fn lie(
    time: Res<Time>,
    game: Option<Res<Game>>,
    mut bodies: Query<(
        Entity,
        &mut Squashed,
        &Venue,
        Option<&mut Shove>,
        Option<&mut PlayerJump>,
        Has<Dead>,
        Has<Remote>,
        Has<Intent>,
    )>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let playing = game.is_some_and(|game| game.playing());
    for (body, mut squashed, &venue, shove, jump, dead, remote, walked) in &mut bodies {
        let mut up = !playing || venue != Venue::Arena || dead;
        if !remote {
            squashed.left -= dt;
            up |= squashed.left <= 0.0;
        }
        if let Some(mut shove) = shove {
            if shove.across != Vec2::ZERO {
                shove.across = Vec2::ZERO;
            }
            // (One of the computer's players' is counted with its brain,
            // `drive_bots`.)
            if walked && shove.hurt > 0.0 {
                shove.hurt = (shove.hurt - dt).max(0.0);
            }
            if shove.pinned == up {
                shove.pinned = !up;
            }
        }
        if let Some(mut jump) = jump
            && jump.velocity_y != 0.0
        {
            jump.velocity_y = 0.0;
        }
        if up {
            commands.entity(body).remove::<Squashed>();
        }
    }
}

/// Lays the model of everyone lying flat down on its face where they are,
/// squashed, and stands it up again as they get up: turned forward onto
/// its front about their feet, put back by half its height so that its
/// middle lies where they stood, and up by what keeps the front of it out
/// of the road (`crate::FLAT_SQUASH` and the rest). Their limbs are
/// spread by `animate_walk` (`crate::squash_offset`). Written once as
/// they go down and once as they get up.
#[allow(clippy::type_complexity)]
fn lay_out(
    flat: Query<(Entity, &Children), (With<Squashed>, Without<Laid>)>,
    up: Query<(Entity, &Laid), Without<Squashed>>,
    children: Query<&Children>,
    bones: Query<(), With<WalkBone>>,
    mut models: Query<&mut Transform>,
    mut commands: Commands,
) {
    for (body, parts) in &flat {
        // Its model is the part of it its limbs are under.
        let Some(model) = parts
            .iter()
            .find(|&part| children.iter_descendants(part).any(|under| bones.contains(under)))
        else {
            continue;
        };
        let Ok(mut place) = models.get_mut(model) else {
            continue;
        };
        let rest = *place;
        // In the body's own space it faces -Z with +Y up: onto its face
        // is a quarter turn forward about its side to side axis.
        let down = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        *place = Transform {
            translation: Vec3::new(0.0, FLAT_LIFT, FLAT_BACK) + down * (rest.translation * FLAT_SQUASH),
            rotation: down * rest.rotation,
            scale: rest.scale * FLAT_SQUASH,
        };
        commands.entity(body).insert(Laid { model, rest });
    }
    for (body, laid) in &up {
        if let Ok(mut place) = models.get_mut(laid.model) {
            *place = laid.rest;
        }
        commands.entity(body).remove::<Laid>();
    }
}

/// Where, in the world, the end of someone lying flat that is furthest up
/// the screen is, just over the road: their head, their feet or a hand,
/// as they lie, the camera being turned as `eye` is. What is written over
/// them stands over this, their name and bar among it ([`place_tags`]):
/// over where their head was when they stood, it would be off to one
/// side of them as they lie.
pub(super) fn far_end(body: &Transform, eye: Quat) -> Vec3 {
    // Which way across the ground is up the screen.
    let up = (eye * Vec3::Y).with_y(0.0).normalize_or(Vec3::NEG_Z);
    let facing = (body.rotation * Vec3::NEG_Z).with_y(0.0).normalize_or(Vec3::NEG_Z);
    let reach = FLAT_ACROSS + (FLAT_ALONG - FLAT_ACROSS) * facing.dot(up).abs();
    body.translation + up * reach + Vec3::Y * 0.2
}

/// What is written over someone with `left` seconds still to lie: the
/// whole seconds, counted down from ten, "10s" to "1s".
fn seconds_left(left: f32) -> String {
    format!("{}s", left.max(0.0).ceil().max(1.0) as u32)
}

/// Writes the seconds left over everyone lying flat, you among them, each
/// laid out once and moved over its body without being laid out again, as
/// a name over a head is; over the name and the bar of anyone else, and
/// just over you, who have neither.
#[allow(clippy::type_complexity)]
fn count_down(
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    flat: Query<(Entity, &Transform, &Squashed, &Venue, Has<Player>, Has<Dead>), Without<ThirdPersonCamera>>,
    here: Query<&Venue, With<Player>>,
    mut clocks: Query<(Entity, &FlatClock, &mut Text, &mut Visibility, &mut UiTransform, &ComputedNode)>,
    mut commands: Commands,
) {
    for (body, ..) in &flat {
        if !clocks.iter().any(|(_, of, ..)| of.0 == body) {
            commands.spawn((
                InArena,
                FlatClock(body),
                Name::new("Seconds left lying flat"),
                hud::words(String::new(), CLOCK_TEXT, ui().colours.arena.squashed),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    top: Val::ZERO,
                    ..default()
                },
                UiTransform::default(),
                Visibility::Hidden,
                GlobalZIndex(-1),
            ));
        }
    }
    let (Ok((camera, eye)), Ok(window)) = (cameras.single(), windows.single()) else {
        return;
    };
    let turned = eye.rotation;
    let eye = GlobalTransform::from(*eye);
    let in_arena = here.single().is_ok_and(|venue| *venue == Venue::Arena);
    let vmin = window.width().min(window.height()) / 100.0;
    for (clock, of, mut text, mut shown, mut place, node) in &mut clocks {
        let Ok((_, body, squashed, &venue, mine, dead)) = flat.get(of.0) else {
            commands.entity(clock).despawn();
            continue;
        };
        // Over the end of them that is furthest up the screen, as they
        // lie; and over the name and bar of anyone who has those, which
        // stand there too.
        let point = (in_arena && venue == Venue::Arena && !dead)
            .then(|| camera.world_to_viewport(&eye, far_end(body, turned)).ok())
            .flatten()
            .filter(|point| on_screen(window, *point));
        let Some(end) = point else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let said = seconds_left(squashed.left);
        if text.0 != said {
            text.0 = said;
        }
        let foot = end.y - (CLOCK_GAP + if mine { 0.0 } else { TAG_TALL }) * vmin;
        let size = node.size() * node.inverse_scale_factor();
        place.set_if_neq(UiTransform {
            translation: Val2::px(end.x - size.x * 0.5, foot - size.y),
            scale: Vec2::ONE,
            rotation: Rot2::IDENTITY,
        });
        shown.set_if_neq(Visibility::Inherited);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seconds_left_are_whole_and_count_down_from_ten_to_one() {
        assert_eq!(seconds_left(SQUASH_SECS), "10s");
        assert_eq!(seconds_left(9.99), "10s");
        assert_eq!(seconds_left(9.0), "9s");
        assert_eq!(seconds_left(0.4), "1s");
        // Never nothing, in the frame before they are up.
        assert_eq!(seconds_left(0.0), "1s");
        assert_eq!(seconds_left(-0.1), "1s");
    }
}
