//! Bridge Wars' sticks of dynamite, as they look (since 2026-11-13;
//! Hajun's `bridgewarsgameevent.md`: "it will throw an arched dynamite (2
//! minimalistic mini red cylinder shape with smaller brown cylinder
//! sticking out of one face which is the string), and after landing on the
//! ground/bridge, it will show a circular blast radius range around it and
//! it will shine bright red 3 time before it explode, and when it explode,
//! it will do 20 damage to player in that blast radius and destroy that
//! blast radius region of the bridge"): two red sticks side by side with a
//! brown fuse out of one end, tumbling end over end along an arch from the
//! hand to the ground ([`Lob`]: thrown straight at where they come down,
//! rising toward the camera over you, and drawn up the screen as they rise
//! and back down it as they fall, as an arch is on a map: never to one
//! side of the throw as such, Hajun, 2026-11-13: "its a arch projectile
//! not a boomarang"); lying where they came down with a red ring on the
//! ground round them as far as the blast will reach; shining three times,
//! the sticks themselves and a glow round them; and then the fireball's
//! own ball of fire (`fire::burst`).
//!
//! This is only how they look, and where they are from frame to frame.
//! What a blast does is the rules' (`roundtown_net::bridge_wars`): the
//! server's online, where each stick is an entity it sends ([`Dynamite`])
//! and takes away again as it blows, saying where it blew and which
//! planks went with it ([`Blasted`], [`hear_blasts`]); and this device's
//! own offline (`arena::dynamites_offline`). Either way a stick here is
//! [`throw`]n as it sets out. One that comes down where there is nothing
//! to lie on falls on into the void, and never blows.
//!
//! Since 2026-11-15 a blast takes the whole bridge down (Hajun: "from
//! that point it will destroy cascadingly the entire bridge like the truly
//! connected bridge ... but it shouldnt destroy the whole bridge instantly
//! but rather cascadingly from that initial blast point so it gives ppl
//! time to run away to safety"): the planks it reaches at once, as before,
//! and every other plank joined to them after, one by one from the hole
//! outward, each drawn in the colour of a plank about to go until it
//! falls (`bridge::Bridges::doom`, `bridge::crumble`). Which planks and
//! when is the rules' too (`roundtown_net::nav::Nav::cascade`).
//!
//! **In any fight** (since 2026-11-18, when Bridge Race came, whose
//! players are given sticks as they go): a stick is thrown on its fight's
//! venue ([`throw`], [`lobbed`]), comes down on that venue's ground
//! ([`fly`]), and is blown by whichever game it is ([`blow`], what Bridge
//! Wars' own blowing is made of, its damage and its credit its own).

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::bridge_wars::{DYNAMITE_RADIUS, Lob, caught};
use roundtown_net::nav::Nav;
use roundtown_net::{Blasted, Dynamite};

use crate::arena::{Fighting, InArena};
use crate::bridge::Bridges;
use crate::clock::TownClock;
use crate::fire;
use crate::island::{Islands, Venue};
use crate::range;
use crate::ui::{UiChanged, ui};

/// How thick each of the two sticks is, from its middle, and how long, in
/// metres: "mini", and still to be seen from the camera 15 m over you,
/// where a metre is about fifty pixels of a phone's screen.
const STICK_RADIUS: f32 = 0.16;
const STICK_LENGTH: f32 = 0.8;
/// The fuse out of one end of them: how thick, from its middle, and how
/// long. A string, and brown as one (`colours.arena.dynamite_fuse`;
/// Hajun, 2026-11-13: "the dynamite's string part should be brown, like a
/// string"): thicker and longer than it was for its first day (0.045 and
/// 0.26), when from the camera over you it was a dark hair at the sticks'
/// end, and no colour at all.
const FUSE_RADIUS: f32 = 0.07;
const FUSE_LENGTH: f32 = 0.42;
/// How many times it turns end over end on its way.
const TUMBLES: f32 = 1.5;
/// How big the glow round it is at its brightest, in metres from its
/// middle, and how see-through even then.
const GLOW: f32 = 1.1;
const GLOW_ALPHA: f32 = 0.5;
/// How many times its own red the sticks shine at their brightest. (Bevy
/// takes no exposure to emitted light: a glow is so many times the colour,
/// as the anchors' is, `lamps`.)
const SHINE: f32 = 5.0;
/// How high over the ground it lies on the ring of its blast's reach is
/// drawn, in metres: clear of a plank's top and an island's.
const RING_OVER: f32 = 0.06;
/// How fast one that found nothing to lie on falls faster, in metres a
/// second a second, as anything falls into the void; and how far it falls
/// before it is taken away.
const FALL: f32 = 33.0;
const FALLEN: f32 = 35.0;
/// How near where the server says one blew a stick here has to be aimed to
/// be that one, in metres.
const SAME: f32 = 0.5;
/// How long ago, by the server's clock, a stick can have blown and its
/// blast still be drawn here, in seconds: one from longer ago blew before
/// this device was in the game, or while its connection was gone.
const BLEW_LATELY: f64 = 3.0;
/// How long the server's word of what has blown waits for the town's
/// clock to be the server's, in seconds, as a device comes online: the
/// planks that fall after a blast fall by that clock.
const CLOCK_WAIT: f32 = 3.0;

/// A stick of dynamite thrown: how far through its throw and its fuse it
/// is ([`Lob`]); who threw it, by seat; whether the server runs it, and
/// then its entity for it, which is gone the moment the server has it
/// blow or fall; what has become of it ([`Lies`]); and which way it lies.
#[derive(Component)]
pub(crate) struct Stick {
    pub(crate) lob: Lob,
    pub(crate) seat: usize,
    pub(crate) served: bool,
    source: Option<Entity>,
    pub(crate) lies: Lies,
    yaw: f32,
}

/// What has become of a stick: still in the air; down, on ground, with
/// its fuse burning; or down where there was nothing to lie on, and
/// falling, this fast.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Lies {
    Flying,
    Lying,
    Falling(f32),
}

impl Stick {
    /// Whether it lies with its fuse burnt down: for whoever runs the
    /// game to blow.
    pub(crate) fn blows(&self) -> bool {
        self.lies == Lies::Lying && self.lob.blown()
    }

    /// Whether it is still to blow, as far as anyone can tell: in the air,
    /// or lying.
    pub(crate) fn live(&self) -> bool {
        !matches!(self.lies, Lies::Falling(_))
    }
}

/// A stick's own looks, to shine: its red, and the glow round it.
#[derive(Component)]
struct Shining {
    red: Handle<StandardMaterial>,
    glow: Handle<StandardMaterial>,
}

/// The part of a stick that tumbles; the glow round it; and the ring on
/// the ground round where it lies.
#[derive(Component)]
struct Tumbling;
#[derive(Component)]
struct Glow;
#[derive(Component)]
struct Reach;

/// The sticks' shapes and what they share of their looks, made as the app
/// opens.
#[derive(Resource)]
struct Look {
    stick: Handle<Mesh>,
    fuse: Handle<Mesh>,
    ball: Handle<Mesh>,
    brown: Handle<StandardMaterial>,
    ring: [Handle<Mesh>; 3],
    ring_fill: Handle<StandardMaterial>,
    ring_outline: Handle<StandardMaterial>,
    ring_rim: Handle<StandardMaterial>,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, load)
        .add_systems(Update, ((follow_server, hear_blasts, fit_out, fly).chain(), recolour));
}

fn load(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let colours = ui().colours.arena;
    let (fill, outline, rim) = range::disc(DYNAMITE_RADIUS);
    commands.insert_resource(Look {
        stick: meshes.add(Cylinder::new(STICK_RADIUS, STICK_LENGTH)),
        fuse: meshes.add(Cylinder::new(FUSE_RADIUS, FUSE_LENGTH)),
        ball: meshes.add(Sphere::new(1.0).mesh().ico(3).expect("a sphere")),
        brown: materials.add(StandardMaterial {
            base_color: colours.dynamite_fuse,
            perceptual_roughness: 0.9,
            ..default()
        }),
        ring: [meshes.add(fill), meshes.add(outline), meshes.add(rim)],
        ring_fill: materials.add(range::look(colours.blast_fill)),
        ring_outline: materials.add(range::look(colours.blast_outline)),
        ring_rim: materials.add(range::look(colours.aim_rim)),
    });
}

/// A stick of dynamite setting out on `lob`, thrown by whoever is in
/// `seat`, on `venue`, the fight's, whose ground it comes down on
/// ([`fly`]); online, with the server's entity for it. What it looks like
/// is put on it a frame later ([`fit_out`]). [`InArena`], to go with the
/// game.
pub(crate) fn throw(commands: &mut Commands, lob: Lob, seat: usize, source: Option<Entity>, venue: Venue) -> Entity {
    let along = lob.to - lob.from;
    commands
        .spawn((
            Stick {
                lob,
                seat,
                served: source.is_some(),
                source,
                lies: Lies::Flying,
                yaw: (-along.x).atan2(-along.z),
            },
            InArena,
            venue,
            Name::new("Dynamite"),
            Transform::from_translation(place(&lob)),
            // Shown with the island it is on (`island::show_where_i_am`).
            Visibility::Hidden,
        ))
        .id()
}

/// A stick of dynamite thrown by someone whose feet are at `feet` on
/// `venue`, the way `heading` says, to come down `far` from them, in a
/// game this device runs: at the ground where it is to come down, as high
/// as that is now, or, with none there, as high as the ground they stand
/// on, as the server throws one online (its `Game::lob`). (Bridge Wars'
/// own until 2026-11-18, `arena::lobbed`, on its map alone.)
pub(crate) fn lobbed(islands: &Islands, venue: Venue, feet: Vec3, heading: f32, far: f32) -> Lob {
    let aimed = Lob::thrown(feet, heading, far, feet.y);
    let island = islands.get(venue);
    let ground = |at: Vec2, reach: f32| island.and_then(|island| island.floor(at, reach));
    let floor = ground(aimed.to.xz(), feet.y + 2.0)
        .or(ground(feet.xz(), feet.y + crate::LAND_STEP_UP))
        .unwrap_or(feet.y);
    Lob::thrown(feet, heading, far, floor)
}

/// What a stick of dynamite did to the bridges as it blew ([`blow`]): the
/// planks it took at once, each whose it was by seat, which of theirs and
/// which plank of that; and those of the rest of their bridges doomed to
/// fall after them, each with how many seconds after.
#[derive(Default, Debug)]
pub(crate) struct Blown {
    pub(crate) taken: Vec<(u8, u16, u16)>,
    pub(crate) later: Vec<((u8, u16, u16), f32)>,
}

/// Blows a stick of dynamite at `at`, across the ground, thrown by whoever
/// is in seat `by`, in a game this device runs, whatever the game, its
/// teams and its venue: every plank whose middle the blast reaches is
/// gone, whoever laid it, and the plank you are laying if its middle is
/// (`Bridges::blast`). With a map of the bridges to keep, `nav`, the one
/// the computer's players go by, every bridge is told to it as it stands
/// and the holes with them; and, `falls`, the rest of each bridge a hole
/// is in is doomed to fall after, plank by plank from the hole outward
/// (`Nav::cascade`, since 2026-11-15), each `now` and so many seconds on,
/// by this device's clock (`Time::elapsed_secs_f64`), drawn about to go
/// until it does (`Bridges::doom`, `bridge::crumble`); without `falls`, a
/// hole to build over (Bridge Race's rule, 2026-11-18). Without a map, the
/// blast's own planks alone. Whom the blast hurts, and whose doing a fall
/// from what follows is, is each game's own to say, by its own teams, as
/// Bridge Wars' does (`arena::dynamites_offline`, which is made of this).
pub(crate) fn blow(bridges: &mut Bridges, nav: Option<&mut Nav>, falls: bool, at: Vec2, by: u8, now: f64) -> Blown {
    let taken = bridges.blast(at, DYNAMITE_RADIUS);
    let mut later = Vec::new();
    if let Some(nav) = nav {
        for (seat, chain, index, rung) in bridges.laid() {
            nav.hear(seat, chain, index, rung);
        }
        nav.break_planks(&taken);
        if falls {
            later = nav.cascade(&taken);
            nav.doom(&later, by);
            bridges.doom(later.iter().map(|&(plank, after)| (plank, now + f64::from(after))), by);
        }
    }
    Blown { taken, later }
}

/// Where a stick on `lob` is drawn: where it is, and as high over that as
/// it is thick, so that lying it lies on the ground and not in it.
fn place(lob: &Lob) -> Vec3 {
    lob.at() + Vec3::Y * STICK_RADIUS
}

/// How bright a stick's red is, shining `shine`, from 0 to 1: what it
/// gives off, on top of what lights it.
fn shining(red: Color, shine: f32) -> LinearRgba {
    let red = red.to_linear();
    let times = SHINE * shine.clamp(0.0, 1.0);
    LinearRgba::rgb(red.red * times, red.green * times, red.blue * times)
}

/// Online, the sticks are the server's: each it sends is thrown here as
/// it comes, on the fight's venue (`Fighting::served_on`; one sent before
/// this device knew where the fight was moved there as soon as it does,
/// `arena::settle`).
fn follow_server(
    fresh: Query<(Entity, &Dynamite), (With<Remote>, Added<Dynamite>)>,
    fighting: Res<Fighting>,
    me: Query<&Venue, With<crate::Player>>,
    mut commands: Commands,
) {
    if fresh.is_empty() {
        return;
    }
    let venue = fighting.served_on(me.single().ok().copied());
    for (source, stick) in &fresh {
        let lob = Lob {
            from: Vec3::new(stick.x, stick.y, stick.z),
            to: Vec3::new(stick.to_x, stick.to_y, stick.to_z),
            age: 0.0,
        };
        let thrown = throw(&mut commands, lob, usize::from(stick.seat), Some(source), venue);
        fighting.settle_later(&mut commands, thrown);
    }
}

/// Online, a stick the server has blown ([`Blasted`]): the planks it says
/// went with it are gone (`bridge::Bridges::break_planks`), and so is the
/// plank you are laying if the blast reached its middle, which only this
/// device knows of; the blast is drawn where it blew; and the stick drawn
/// here for it is taken away.
///
/// And the rest of the bridge is doomed, each plank to fall at the time
/// the server says, by its clock, which is the town's
/// (`bridge::Bridges::doom`, since 2026-11-15): one whose time has passed
/// is gone at once. A stick that blew before this device was there, as it
/// comes into a game under way or back into one, is no blast to draw
/// ([`BLEW_LATELY`]). And all of them are gone over again once the
/// bridges have been cleared for another game (`Bridges::game`), as the
/// rungs are (`bridge::hear_planks`).
#[allow(clippy::too_many_arguments)]
fn hear_blasts(
    time: Res<Time>,
    town: Res<TownClock>,
    fresh: Query<&Blasted, (With<Remote>, Added<Blasted>)>,
    every: Query<&Blasted, With<Remote>>,
    sticks: Query<(Entity, &Stick)>,
    (fighting, me): (Res<Fighting>, Query<&Venue, With<crate::Player>>),
    mut heard: Local<(u32, f32)>,
    mut bridges: ResMut<Bridges>,
    mut commands: Commands,
) {
    let again = heard.0 != bridges.game();
    if !again && fresh.is_empty() {
        return;
    }
    // Not until the town's clock is the server's, which it is a moment
    // after a device comes online: the times are by it.
    if !town.is_the_servers() && heard.1 < CLOCK_WAIT {
        *heard = (u32::MAX, heard.1 + time.delta_secs());
        return;
    }
    *heard = (bridges.game(), 0.0);
    let (now, clock) = (time.elapsed_secs_f64(), town.now());
    let venue = fighting.served_on(me.single().ok().copied());
    let mut take = |blast: &Blasted| {
        let at = Vec3::new(blast.x, blast.y, blast.z);
        bridges.break_planks(&blast.planks);
        // (Whose stick it was is the server's to know: it says whose doing
        // a fall was.)
        bridges.doom(blast.later().map(|(plank, after)| (plank, now + (blast.at + f64::from(after) - clock))), u8::MAX);
        if clock - blast.at > BLEW_LATELY {
            return;
        }
        bridges.blast_tip(at.xz(), DYNAMITE_RADIUS);
        let burst = fire::burst(&mut commands, at + Vec3::Y * STICK_RADIUS, venue);
        fighting.settle_later(&mut commands, burst);
        info!(
            "a stick of dynamite blew at {:.1}, taking {} planks, and {} more to fall",
            at.xz(),
            blast.planks.len(),
            blast.later.len()
        );
        if let Some((stick, _)) = sticks
            .iter()
            .filter(|(_, stick)| stick.served && stick.lob.to.distance(at) < SAME)
            .min_by(|a, b| b.1.lob.age.total_cmp(&a.1.lob.age))
        {
            commands.entity(stick).try_despawn();
        }
    };
    if again {
        every.iter().for_each(&mut take);
    } else {
        fresh.iter().for_each(&mut take);
    }
}

/// Puts on each stick what it looks like, the frame after it is thrown:
/// its two sticks and its fuse, which tumble together; the glow round it;
/// and the ring on the ground, put away until it lies there.
fn fit_out(
    look: Res<Look>,
    fresh: Query<Entity, Added<Stick>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let colours = ui().colours.arena;
    for stick in &fresh {
        let red = materials.add(StandardMaterial {
            base_color: colours.dynamite,
            perceptual_roughness: 0.7,
            ..default()
        });
        let glow = materials.add(StandardMaterial {
            base_color: colours.dynamite.with_alpha(0.0),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        // A cylinder stands up its own y: laid along the stick's z.
        let lying = Quat::from_rotation_x(FRAC_PI_2);
        commands
            .entity(stick)
            .insert(Shining {
                red: red.clone(),
                glow: glow.clone(),
            })
            .with_children(|parts| {
                parts.spawn((Tumbling, Transform::default(), Visibility::Inherited)).with_children(|body| {
                    for side in [-1.0, 1.0] {
                        body.spawn((
                            Mesh3d(look.stick.clone()),
                            MeshMaterial3d(red.clone()),
                            Transform::from_xyz(side * STICK_RADIUS, 0.0, 0.0).with_rotation(lying),
                            NotShadowCaster,
                        ));
                    }
                    // The fuse, out of the end that leads.
                    body.spawn((
                        Mesh3d(look.fuse.clone()),
                        MeshMaterial3d(look.brown.clone()),
                        Transform::from_xyz(0.0, 0.0, -(STICK_LENGTH + FUSE_LENGTH) * 0.5).with_rotation(lying),
                        NotShadowCaster,
                    ));
                });
                parts.spawn((
                    Glow,
                    Mesh3d(look.ball.clone()),
                    MeshMaterial3d(glow),
                    Transform::from_scale(Vec3::splat(0.01)),
                    Visibility::Hidden,
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                parts
                    .spawn((Reach, Transform::from_xyz(0.0, RING_OVER - STICK_RADIUS, 0.0), Visibility::Hidden))
                    .with_children(|ring| {
                        let sheets = [(&look.ring[2], &look.ring_rim, 0.0), (&look.ring[0], &look.ring_fill, 0.0), (&look.ring[1], &look.ring_outline, 0.002)];
                        for (mesh, material, over) in sheets {
                            ring.spawn((
                                Mesh3d(mesh.clone()),
                                MeshMaterial3d(material.clone()),
                                Transform::from_xyz(0.0, over, 0.0),
                                NotShadowCaster,
                                NotShadowReceiver,
                            ));
                        }
                    });
            });
    }
}

/// Moves every stick on: along its arch, tumbling; as it comes down, onto
/// the ground of its own venue if there is any for it to lie on
/// ([`caught`]), with the ring of its blast's reach round it, and on down
/// into the void if there is not; shining as its fuse burns
/// ([`Lob::shine`]). One of the server's that the server has taken away
/// without a blast ([`hear_blasts`] takes those) has fallen into the void
/// there: here it falls on from where it is, or, if this device had it
/// lying on something, is gone.
#[allow(clippy::type_complexity)]
fn fly(
    time: Res<Time>,
    islands: Res<Islands>,
    sources: Query<(), With<Dynamite>>,
    mut sticks: Query<(Entity, &mut Stick, &Venue, &mut Transform, Option<&Shining>, Option<&Children>)>,
    mut tumbling: Query<&mut Transform, (With<Tumbling>, Without<Stick>, Without<Glow>)>,
    mut glows: Query<(&mut Transform, &mut Visibility), (With<Glow>, Without<Stick>, Without<Tumbling>, Without<Reach>)>,
    mut rings: Query<&mut Visibility, (With<Reach>, Without<Glow>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let colours = ui().colours.arena;
    for (entity, mut stick, &venue, mut transform, looks, children) in &mut sticks {
        let island = islands.get(venue);
        // Taken away by the server with no blast: it found nothing to lie
        // on there.
        if stick.served && stick.source.is_some_and(|source| !sources.contains(source)) {
            if stick.lies == Lies::Lying {
                commands.entity(entity).despawn();
                continue;
            }
            stick.source = None;
        }
        match stick.lies {
            Lies::Flying => {
                stick.lob.tick(dt);
                transform.translation = place(&stick.lob);
                if stick.lob.down() {
                    let to = stick.lob.to;
                    let ground = |spot: Vec2| island.and_then(|island| island.floor(spot, to.y + 1.0)).is_some_and(|floor| (floor - to.y).abs() < 1.0);
                    // One of the server's it has taken away already fell
                    // there, whatever this device makes of the ground.
                    let gone = stick.served && stick.source.is_none();
                    stick.lies = if !gone && caught(to.xz(), ground) {
                        Lies::Lying
                    } else {
                        // On down as fast as it was coming.
                        Lies::Falling(4.0 * stick.lob.apex() / stick.lob.secs())
                    };
                }
            }
            Lies::Lying => {
                // This device's own are blown by its game, once their
                // fuse has burnt down (`arena::dynamites_offline`); the
                // server's by the server.
                stick.lob.tick(dt);
                transform.translation = place(&stick.lob);
            }
            Lies::Falling(speed) => {
                transform.translation.y -= speed * dt;
                stick.lies = Lies::Falling(speed + FALL * dt);
                if transform.translation.y < stick.lob.to.y - FALLEN {
                    commands.entity(entity).despawn();
                    continue;
                }
            }
        }
        let lying = stick.lies == Lies::Lying;
        let shine = if lying { stick.lob.shine() } else { 0.0 };
        let turn = match stick.lies {
            // End over end on its way, and on as it falls.
            Lies::Flying => (stick.lob.age / stick.lob.secs()).clamp(0.0, 1.0) * TUMBLES * TAU,
            Lies::Lying => 0.0,
            Lies::Falling(speed) => speed * 0.6,
        };
        let turned = Quat::from_rotation_y(stick.yaw) * Quat::from_rotation_x(turn);
        for &child in children.into_iter().flatten() {
            if let Ok(mut body) = tumbling.get_mut(child) {
                if body.rotation != turned {
                    body.rotation = turned;
                }
            } else if let Ok((mut glow, mut shown)) = glows.get_mut(child) {
                shown.set_if_neq(if shine > 0.01 { Visibility::Inherited } else { Visibility::Hidden });
                let size = Vec3::splat((GLOW * (0.5 + 0.5 * shine)).max(0.01));
                if glow.scale != size {
                    glow.scale = size;
                }
            } else if let Ok(mut shown) = rings.get_mut(child) {
                shown.set_if_neq(if lying { Visibility::Inherited } else { Visibility::Hidden });
            }
        }
        if let Some(looks) = looks {
            let glowing = shining(colours.dynamite, shine);
            if let Some(mut red) = materials.get_mut(&looks.red)
                && red.emissive != glowing
            {
                red.emissive = glowing;
            }
            let alpha = GLOW_ALPHA * shine;
            if let Some(mut glow) = materials.get_mut(&looks.glow)
                && (glow.base_color.alpha() - alpha).abs() > 1e-3
            {
                glow.base_color = range::bright(colours.dynamite).with_alpha(alpha);
            }
        }
    }
}

/// Takes the ring's colours and the fuse's again as the file changes on
/// desktop; a stick's own red is taken as it next shines ([`fly`]).
fn recolour(changed: Res<UiChanged>, look: Res<Look>, mut materials: ResMut<Assets<StandardMaterial>>) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let colours = ui().colours.arena;
    for (handle, colour) in [
        (&look.ring_fill, colours.blast_fill),
        (&look.ring_outline, colours.blast_outline),
        (&look.ring_rim, colours.aim_rim),
    ] {
        if let Some(mut material) = materials.get_mut(handle) {
            material.base_color = range::bright(colour);
        }
    }
    if let Some(mut brown) = materials.get_mut(&look.brown) {
        brown.base_color = colours.dynamite_fuse;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stick_is_drawn_on_the_ground_it_lies_on_and_shines_only_as_its_fuse_burns() {
        let feet = Vec3::new(0.0, 0.25, 130.0);
        let mut lob = Lob::thrown(feet, 0.0, 6.0, 0.24);
        // In the air it gives off nothing; lying, at the height of each
        // flash, several times its own red.
        let red = Color::srgb(0.9, 0.2, 0.15);
        assert_eq!(shining(red, lob.shine()), LinearRgba::rgb(0.0, 0.0, 0.0));
        let mut brightest = 0.0f32;
        while !lob.blown() {
            lob.tick(1.0 / 60.0);
            let glowing = shining(red, lob.shine());
            assert!(lob.down() || glowing.red == 0.0);
            brightest = brightest.max(glowing.red);
        }
        assert!(brightest > 3.0 * red.to_linear().red, "{brightest}");
        // Lying, its sticks rest on the ground: their middles as high over
        // it as they are thick.
        assert!((place(&lob).y - (0.24 + STICK_RADIUS)).abs() < 1e-5);
        assert!(place(&lob).xz().distance(Vec2::new(0.0, 124.0)) < 1e-4);
        // The sticks are small, and still wider across than the fuse,
        // which is long enough to be seen for the string it is.
        assert!(STICK_LENGTH < 1.0 && 4.0 * STICK_RADIUS < 1.0 && FUSE_RADIUS < 0.5 * STICK_RADIUS);
        assert!(FUSE_LENGTH > 0.4 * STICK_LENGTH && FUSE_LENGTH < STICK_LENGTH);
        // One that lies with its fuse burnt down is for the game to blow;
        // one falling never is.
        let mut stick = Stick {
            lob,
            seat: 0,
            served: false,
            source: None,
            lies: Lies::Lying,
            yaw: 0.0,
        };
        assert!(stick.blows() && stick.live());
        stick.lies = Lies::Falling(3.0);
        assert!(!stick.blows() && !stick.live());
        stick.lies = Lies::Flying;
        assert!(!stick.blows() && stick.live());
    }

    /// A disc of land `radius` round `centre`, its top `top` up: a map
    /// read from no model.
    fn disc(centre: Vec2, radius: f32, top: f32) -> Vec<[Vec3; 3]> {
        let at = |angle: f32| Vec3::new(centre.x + angle.cos() * radius, top, centre.y + angle.sin() * radius);
        let middle = Vec3::new(centre.x, top, centre.y);
        (0..64)
            .map(|k| {
                let (a, b) = (k as f32 * TAU / 64.0, (k + 1) as f32 * TAU / 64.0);
                [middle, at(b), at(a)]
            })
            .collect()
    }

    #[test]
    fn a_stick_thrown_on_another_venue_comes_down_on_that_venue_s_ground() {
        use bevy::ecs::system::RunSystemOnce;
        use std::time::Duration;
        // Last Stand's stone as a disc 25 m round its middle, its top a
        // quarter of a metre up as it stands lifted; and Bridge Wars' map
        // far off, nothing of it here.
        let mut islands = Islands::default();
        islands.reshape(Venue::LastStand, disc(Vec2::ZERO, 25.0, 0.25), &[]);
        islands.reshape(Venue::Arena, disc(Vec2::splat(300.0), 10.0, 0.25), &[]);
        // Thrown 8 m on from 2 m off the middle, the way a plank heads at
        // no turn: lobbed onto Last Stand's ground there.
        let feet = Vec3::new(0.0, 0.25, 2.0);
        let lob = lobbed(&islands, Venue::LastStand, feet, 0.0, 8.0);
        assert!((lob.to.y - 0.25).abs() < 1e-4 && lob.to.xz().distance(Vec2::new(0.0, -6.0)) < 1e-3, "{}", lob.to);
        let mut world = World::new();
        world.insert_resource(islands);
        world.insert_resource(Time::<()>::default());
        world.init_resource::<Assets<StandardMaterial>>();
        let (mine, theirs) = {
            let mut commands = world.commands();
            (throw(&mut commands, lob, 0, None, Venue::LastStand), throw(&mut commands, lob, 0, None, Venue::Arena))
        };
        world.flush();
        for _ in 0..240 {
            world.resource_mut::<Time>().advance_by(Duration::from_secs_f32(1.0 / 60.0));
            world.run_system_once(fly).expect("it flies");
        }
        // On its own venue's ground it lies, its fuse burning, for its game
        // to blow; the very same throw on Bridge Wars' map has nothing to
        // lie on there, and falls on into the void.
        assert_eq!(world.get::<Venue>(mine), Some(&Venue::LastStand));
        assert_eq!(world.get::<Stick>(mine).map(|stick| stick.lies), Some(Lies::Lying));
        assert!(world.get::<Stick>(mine).is_some_and(Stick::blows));
        assert!(world.get::<Stick>(theirs).is_none_or(|stick| matches!(stick.lies, Lies::Falling(_))));
    }

    #[test]
    fn a_blast_takes_the_planks_it_reaches_and_the_rest_of_the_bridge_only_where_bridges_fall() {
        use roundtown_net::nav::Region;
        use crate::bridge::Rung;
        // A bridge from the south island to the middle of Bridge Wars' map,
        // a span of the map the computer's players keep, laid as one of
        // theirs is, rung by rung.
        let build = || {
            let mut nav = Nav::arena();
            let span = nav.grid().span(Region::Team(0), Region::Middle, 0.0).expect("a span");
            let mut bridges = Bridges::default();
            for index in 0..=span.planks() {
                let here = span.rung(index);
                let along = if index < span.planks() { span.rung(index + 1) - here } else { here - span.rung(index - 1) };
                let d = along.normalize_or(Vec2::NEG_Y);
                bridges.lay_remote(4, 0, index as u16, Rung { p: here, d });
                nav.hear(4, 0, index as u16, here);
            }
            (nav, bridges, span.point(0.3))
        };
        // Blown with the map, its bridge falling (Bridge Wars' rule since
        // 2026-11-15): a hole at once, and the rest doomed after it.
        let (mut nav, mut bridges, at) = build();
        let blown = blow(&mut bridges, Some(&mut nav), true, at, 2, 10.0);
        assert!(!blown.taken.is_empty() && blown.later.len() > blown.taken.len(), "{blown:?}");
        assert_eq!(bridges.broken().count(), blown.taken.len());
        let ((seat, chain, plank), _) = blown.later[0];
        assert!(bridges.doomed(seat, chain, plank));
        // With no falling (Bridge Race's rule, 2026-11-18), or no map: the
        // hole alone, to be built over.
        for keep in [true, false] {
            let (mut nav, mut bridges, at) = build();
            let blown = blow(&mut bridges, keep.then_some(&mut nav), false, at, 2, 10.0);
            assert!(!blown.taken.is_empty() && blown.later.is_empty(), "{blown:?}");
            assert_eq!(bridges.broken().count(), blown.taken.len());
            assert!(bridges.laid().all(|(seat, chain, index, _)| !bridges.doomed(seat, chain, index)));
        }
    }
}
