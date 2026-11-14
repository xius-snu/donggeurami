//! The builder a Bridge Builder sends off (since 2026-11-12; Hajun's
//! `bridgewarsgameevent.txt`: "when u let go, it spawns a bot character
//! that walks in bridge mode toward that direction, creating a bridge in a
//! straight line for 100m or until it reaches a island/land and then the
//! bot pops/disappears"): a body in your own look, carrying a plank as you
//! do while you build, that walks off the way you aimed it at the pace
//! anyone builds at ([`BUILDER_SPEED`]) and lays a bridge of its own in
//! front of its feet from where the ground ends.
//!
//! It is only there to be seen (`ForShow`, `lobby::puppet`): in nobody's
//! way, nothing lands on it, and it walks through whatever stands on the
//! map. What it builds is a bridge like any of yours, a chain in your name
//! out of the same rungs ([`Rung`]), its root on the very edge of the
//! ground it leaves, the island's or a plank's, and cut along it as yours
//! are, and its end landed on the far shore as yours is ([`land_on`]); so
//! everyone stands on it, and online every rung goes to the server as it
//! is fixed, and the others see the bridge grow. **And the builder with
//! it** (since 2026-11-14, Hajun: "right now, the bridge bot is not
//! visible to other players, just the bridge that it creates. make your
//! bridgebot visible to other players too"): online its owner's device
//! says where it is as it walks (`Ask::Builder`), the server sends
//! everyone else in the game a body for it there
//! (`roundtown_net::BuilderOf`), and each of their devices draws that as a
//! body carrying a plank, in nobody's way ([`dress_remote`]); it is gone
//! for them as its owner's pops. Until then it was its owner's device's
//! alone, as the plank in your arms still is. Its metres are its own,
//! [`BUILDER_METRES`], not yours; its pace is its team's
//! (`roundtown_net::bridge_wars::builder_speed`: half a run, and faster as
//! the team quickens its building at the upgrade shop).
//!
//! **Where it goes** ([`advance`]): straight on over the ground until the
//! ground ends in front of it; there its bridge begins, heading the way it
//! walked, or as near that as a first plank may slant from straight out
//! of the edge ([`MAX_SLANT`]), and it walks out along it, a plank or two
//! laid ahead of it ([`LAID_AHEAD`]). Ground ahead within a plank's reach,
//! land or anyone's bridge, the bridge lands on it; out of metres, it ends
//! where it is. At the end of its bridge the builder pops. Walking
//! [`GIVES_UP`] without the ground ending, or sent off with nothing under
//! it, it pops having built nothing.

use std::sync::atomic::{AtomicU8, Ordering};

use roundtown_net::bridge_wars::{BUILDER_METRES, BUILDER_SPEED, builder_speed};
use roundtown_net::{BuilderOf, Member, Spot, TICK_HZ};

use super::*;
use crate::arena::Outfit;
use crate::lobby;

/// How far it walks over the ground without coming to the end of it
/// before it gives up, in metres: across the town's middle and a little.
const GIVES_UP: f32 = 170.0;
/// How far ahead of its feet its bridge is laid, in metres, at the least:
/// a plank is put down once the last one ends nearer than this.
const LAID_AHEAD: f32 = 1.6;
/// How long it takes to pop, in seconds: it shrinks to nothing.
const POP_SECS: f32 = 0.22;

/// A builder on its way: whose it is, by seat; which way it walks, across
/// the ground; the metres of bridge it has left; how far it has walked;
/// where it turns onto its bridge's own line, the root and the way the
/// bridge heads, until it has; which of its owner's chains it is laying,
/// once it has begun one; where that ends, once it is laid as far as it
/// goes; and, popping, for how much longer. How fast it walks, in metres a second, which is its
/// owner's team's to quicken. And, for everyone else in a game online to
/// see it: its owner's number for it, and how long since the server was
/// last told where it is.
#[derive(Component, Debug)]
pub(crate) struct Builder {
    seat: u8,
    dir: Vec2,
    metres: f32,
    walked: f32,
    turn: Option<(Vec2, Vec2)>,
    chain: Option<u16>,
    end: Option<Vec2>,
    popping: Option<f32>,
    speed: f32,
    id: u8,
    told: f32,
}

/// The number the next builder sent off from this device goes by.
static NEXT: AtomicU8 = AtomicU8::new(0);

impl Builder {
    fn new(seat: u8, dir: Vec2) -> Self {
        Self {
            seat,
            dir,
            metres: BUILDER_METRES,
            walked: 0.0,
            turn: None,
            chain: None,
            end: None,
            popping: None,
            speed: BUILDER_SPEED,
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            // Told of at once, as it sets off.
            told: f32::INFINITY,
        }
    }
}

/// Sends a builder off from `from`, where its owner stands, heading
/// `heading` as a plank heads: a body in `member`'s look, for the owner in
/// `seat`.
pub(crate) fn send_builder(
    commands: &mut Commands,
    assets: &AssetServer,
    member: &Member,
    seat: u8,
    from: Vec3,
    heading: f32,
) -> Entity {
    let dir = heading_of(heading);
    let place = Transform::from_translation(from).with_rotation(Quat::from_rotation_y(heading));
    let builder = lobby::puppet(commands, assets, member, Venue::Arena, place);
    commands
        .entity(builder)
        .insert((Builder::new(seat, dir), InArena, Carrying, Name::new("Bridge builder")));
    info!("a builder sets off from {:.1}, {:.1}, heading {dir}", from.x, from.z);
    builder
}

/// Puts a plank in the arms of each builder as it sets off, as the one in
/// yours is ([`carry`]).
pub(super) fn equip(
    look: Res<Look>,
    fresh: Query<Entity, Added<Builder>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for builder in &fresh {
        commands.entity(builder).with_child((Name::new("Builder's plank"), board(&mut meshes, &look)));
    }
}

/// Gives the builder someone else in the game has sent off, as the server
/// sends it ([`BuilderOf`]: a body like any other online, in its owner's
/// name and look, moved as its owner's device says), what makes it a
/// builder here: it carries a plank ([`carry_for_bots`] puts one in its
/// arms), and is only there to be seen, in nobody's way and out of every
/// punch's reach.
#[allow(clippy::type_complexity)]
pub(super) fn dress_remote(
    fresh: Query<Entity, (With<BuilderOf>, With<crate::WalkCycle>, Without<Carrying>)>,
    mut commands: Commands,
) {
    for builder in &fresh {
        commands.entity(builder).insert((crate::ForShow, Carrying));
    }
}

/// Walks every builder on, lays its bridge ahead of it ([`advance`]), and
/// pops it at the end; online, sends every rung it fixes, and says where
/// the builder is as it walks, and that it is gone, for everyone else to
/// see it. A game over, or another begun, and they are gone with the rest
/// of it.
#[allow(clippy::too_many_arguments)]
pub(super) fn walk(
    time: Res<Time>,
    fighting: Res<Fighting>,
    online: Res<Online>,
    islands: Res<Islands>,
    outfit: Res<Outfit>,
    mut bridges: ResMut<Bridges>,
    mut builders: Query<(Entity, &mut Builder, &mut Transform)>,
    mut requests: MessageWriter<NetAsk>,
    mut commands: Commands,
) {
    if builders.is_empty() {
        return;
    }
    let dt = time.delta_secs();
    let island = islands.get(Venue::Arena);
    for (entity, mut builder, mut place) in &mut builders {
        if let Some(left) = builder.popping.as_mut() {
            *left -= dt;
            if *left <= 0.0 {
                commands.entity(entity).despawn();
            } else {
                place.scale = Vec3::splat((*left / POP_SECS).clamp(0.0, 1.0));
            }
            continue;
        }
        let Some(island) = island.filter(|_| fighting.playing && fighting.game == bridges.game) else {
            commands.entity(entity).despawn();
            continue;
        };
        let reach = place.translation.y + LAND_STEP_UP;
        let land = |at: Vec2| island.floor_base(at, reach).is_some();
        let mut sent = Vec::new();
        // At the pace your team builds at, as it stands now.
        builder.speed = builder_speed(&outfit.team);
        let (at, done) = advance(&mut builder, place.translation.xz(), dt, &land, &mut bridges, &mut sent);
        // On whatever is under it now, the land or a plank; over nothing
        // for a moment, as its first plank is laid, as high as it was.
        let feet = island.floor(at, reach).unwrap_or(place.translation.y);
        place.translation = Vec3::new(at.x, feet, at.y);
        let facing = Quat::from_rotation_y(yaw_of(builder.dir));
        if place.rotation != facing {
            place.rotation = facing;
        }
        if done {
            builder.popping = Some(POP_SECS);
            info!(
                "a builder is done {:.0} m on, with {:.0} m of bridge laid",
                builder.walked,
                BUILDER_METRES - builder.metres
            );
        }
        if online.is() && fighting.online {
            // Where it is, for everyone else in the game to see it there:
            // as often as your own moves are sent; and that it is gone,
            // the moment it begins to pop.
            builder.told += dt;
            if done {
                requests.write(NetAsk::Builder {
                    id: builder.id,
                    spot: None,
                });
            } else if builder.told >= (1.0 / TICK_HZ) as f32 {
                builder.told = 0.0;
                requests.write(NetAsk::Builder {
                    id: builder.id,
                    spot: Some(Spot::of(place.translation, facing)),
                });
            }
            for (chain, index, rung) in sent {
                requests.write(NetAsk::Rung {
                    chain,
                    index,
                    x: rung.p.x,
                    z: rung.p.y,
                    heading: yaw_of(rung.d),
                });
            }
        }
    }
}

/// Moves `builder`, standing at `at` across the ground, on by `dt`
/// seconds: where it now stands, and whether it is done, to pop. `land` is
/// the island itself; the `bridges` are everyone's, its own among them,
/// begun and laid here, every rung fixed going into `sent` with its
/// chain's number and its own, to send.
fn advance(
    builder: &mut Builder,
    at: Vec2,
    dt: f32,
    land: &dyn Fn(Vec2) -> bool,
    bridges: &mut Bridges,
    sent: &mut Vec<(u16, u16, Rung)>,
) -> (Vec2, bool) {
    let step = builder.speed * dt;
    // Still on the ground it set off from: where does that end?
    if builder.chain.is_none() && builder.end.is_none() {
        let begun = {
            let ground = |p: Vec2| land(p) || bridges.plank_under(p).is_some();
            if !ground(at) {
                // Sent off with nothing under it.
                return (at, true);
            }
            let probe = at + builder.dir * (step + LOOK);
            if ground(probe) {
                None
            } else {
                let e = bisect(at, probe, &ground);
                let square = match bridges.plank_under(at) {
                    Some(c) => bridges.chains[c].edge_normal(e),
                    None => outward(e, &ground),
                };
                Some((Rung { p: e, d: square.unwrap_or(builder.dir) }, !land(at)))
            }
        };
        match begun {
            Some((root, forked)) => {
                if bridges.chains.len() >= MOST_CHAINS {
                    return (at, true);
                }
                let id = bridges.next_id;
                bridges.next_id = bridges.next_id.wrapping_add(1);
                let mut chain = Chain::new(builder.seat, id, vec![root]);
                chain.forked = forked;
                bridges.chains.push(chain);
                bridges.dirty = true;
                sent.push((id, 0, root));
                builder.chain = Some(id);
                builder.turn = Some((root.p, clamp_turn(builder.dir, root.d, MAX_SLANT)));
            }
            None if builder.walked > GIVES_UP => return (at, true),
            None => {}
        }
    }
    // Its bridge, laid on ahead of its feet.
    if let Some(id) = builder.chain
        && builder.end.is_none()
    {
        let Some(c) = bridges.chains.iter().position(|chain| chain.seat == builder.seat && chain.id == id) else {
            // Cleared away with the game.
            return (at, true);
        };
        let d = builder.turn.map_or(builder.dir, |(_, d)| d);
        loop {
            let last = *bridges.chains[c].rungs.last().expect("a chain has a root");
            if (last.p - at).dot(d) >= LAID_AHEAD {
                break;
            }
            let others = bridges.ground_but(c);
            let ground = |p: Vec2| land(p) || others.iter().any(|&quad| inside_quad(p, quad));
            let chain = &mut bridges.chains[c];
            let first = chain.rungs.len() == 1;
            let whole = if first && chain.forked { first_length(d.angle_to(chain.rungs[0].d)) } else { SPACING };
            let mut grown = Grown::default();
            let ended = match shore_ahead(last.p, d, &ground, whole + STRETCH) {
                // Ground within a plank's reach, and the metres to reach
                // it: the bridge lands on it, as one of yours does.
                Some(gap) if gap.max(MIN_LAND) <= builder.metres => {
                    land_on(chain, last.p + d * (gap + STAND_IN), &ground, &mut builder.metres, &mut grown);
                    true
                }
                _ => {
                    let length = whole.min(builder.metres);
                    if length < MIN_LAND || chain.rungs.len() >= MOST_RUNGS {
                        true
                    } else {
                        builder.metres -= length;
                        fix(chain, Rung { p: last.p + d * length, d }, &mut grown);
                        builder.metres < MIN_LAND
                    }
                }
            };
            sent.extend(grown.added.iter().map(|&(index, rung)| (id, index, rung)));
            if !grown.added.is_empty() {
                bridges.dirty = true;
            }
            if ended {
                let chain = &bridges.chains[c];
                builder.end = Some(chain.rungs.last().expect("a chain has a root").p);
                // Nothing but its root: nothing built.
                if chain.rungs.len() < 2 {
                    bridges.forget(c);
                }
                break;
            }
        }
    }
    // Its step: to where its bridge begins, and on along the bridge.
    let (mut at, mut left) = (at, step);
    if let Some((corner, d)) = builder.turn {
        let to = (corner - at).dot(builder.dir).max(0.0);
        if to <= left {
            at = corner;
            left -= to;
            builder.dir = d;
            builder.turn = None;
        }
    }
    at += builder.dir * left;
    builder.walked += step;
    let done = builder.end.is_some_and(|end| (end - at).dot(builder.dir) <= 0.0);
    (at, done)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two round islands 10 m across the ground from their middles, 30 m
    /// apart along x: 10 m of void between them.
    fn two_islands(at: Vec2) -> bool {
        at.length() <= 10.0 || at.distance(Vec2::new(30.0, 0.0)) <= 10.0
    }

    /// Walks a builder sent from `from` along `dir` over `land` until it
    /// is done, or `most` seconds are up: the builder as it ended, where
    /// it stood, the bridges, and every rung it sent. At every step off
    /// the land there is a plank under its feet.
    fn run(from: Vec2, dir: Vec2, land: &dyn Fn(Vec2) -> bool, most: f32) -> (Builder, Vec2, Bridges, Vec<(u16, u16, Rung)>, bool) {
        let mut bridges = Bridges::default();
        let mut builder = Builder::new(2, dir);
        let (mut at, mut sent, mut done) = (from, Vec::new(), false);
        let (dt, mut time) = (1.0 / 60.0, 0.0);
        while !done && time < most {
            let (next, ended) = advance(&mut builder, at, dt, land, &mut bridges, &mut sent);
            at = next;
            done = ended;
            time += dt;
            bridges.recut(land);
            assert!(
                done || land(at) || bridges.plank_under(at).is_some(),
                "nothing under the builder at {at}, {time:.2} s in"
            );
        }
        (builder, at, bridges, sent, done)
    }

    #[test]
    fn a_builder_lays_a_straight_bridge_from_the_edge_to_the_far_shore_and_is_done_there() {
        let (builder, at, bridges, sent, done) = run(Vec2::new(2.0, 0.0), Vec2::X, &two_islands, 30.0);
        assert!(done, "still walking at {at}");
        // One bridge, in its owner's name, from the near edge to the far.
        assert_eq!(bridges.chains.len(), 1);
        let chain = &bridges.chains[0];
        assert_eq!((chain.seat, chain.id), (2, 0));
        let (root, last) = (chain.rungs[0], *chain.rungs.last().unwrap());
        assert!(root.p.distance(Vec2::new(10.0, 0.0)) < 0.05 && root.d.distance(Vec2::X) < 0.02, "{root:?}");
        assert!(two_islands(last.p + last.d * 0.05) && last.p.x >= 19.9 && last.p.x < 21.5, "{last:?}");
        // Straight all the way, a plank's length a rung until the last.
        for pair in chain.rungs.windows(2) {
            assert!(pair[1].d.distance(Vec2::X) < 0.02 && pair[1].p.y.abs() < 0.05, "{pair:?}");
            assert!(pair[1].p.x - pair[0].p.x <= SPACING + STRETCH + 1e-3);
        }
        assert!(chain.complete, "it lands");
        // Every rung was sent once, in order, from the root.
        let indexes: Vec<u16> = sent.iter().map(|&(id, index, _)| (id == 0).then_some(index).unwrap()).collect();
        assert_eq!(indexes, (0..chain.rungs.len() as u16).collect::<Vec<_>>());
        assert!(sent.iter().zip(&chain.rungs).all(|((_, _, sent), fixed)| sent == fixed));
        // About ten metres of its hundred, and it stands at the end of
        // its bridge, on the far shore.
        let used = BUILDER_METRES - builder.metres;
        assert!((9.5..=11.5).contains(&used), "{used} m");
        assert!(at.distance(last.p) < 0.2 && builder.walked > 17.5, "{at} after {} m", builder.walked);
    }

    #[test]
    fn with_no_shore_ahead_it_lays_its_hundred_metres_and_stops() {
        let island = |at: Vec2| at.length() <= 10.0;
        let (builder, at, bridges, sent, done) = run(Vec2::ZERO, Vec2::NEG_Y, &island, 60.0);
        assert!(done);
        let chain = &bridges.chains[0];
        let (root, last) = (chain.rungs[0], *chain.rungs.last().unwrap());
        // A hundred metres of it in whole planks, and a last one of what
        // is left, if that is as much as the least a plank may be: what is
        // left after is less than that.
        let laid = root.p.distance(last.p);
        assert!(laid <= BUILDER_METRES && BUILDER_METRES - laid < MIN_LAND, "{laid} m");
        assert!((laid - (BUILDER_METRES - builder.metres)).abs() < 0.01 && !chain.complete);
        let whole = (BUILDER_METRES / SPACING).floor();
        let planks = whole as usize + usize::from(BUILDER_METRES - whole * SPACING >= MIN_LAND);
        assert_eq!((sent.len(), chain.rungs.len()), (planks + 1, planks + 1));
        assert!((55..=62).contains(&planks), "{planks} planks");
        // It stands at the end of its bridge, ten metres of island and its
        // bridge from where it set out, at the pace anyone builds at.
        assert!(at.distance(last.p) < 0.2 && at.distance(Vec2::new(0.0, -10.0 - laid)) < 0.2, "{at}");
        assert!((builder.walked - (10.0 + laid)).abs() < 0.3);
    }

    #[test]
    fn heading_off_an_edge_at_a_slant_its_bridge_is_a_straight_one_it_turns_onto() {
        // Off the east side of the island walking north-east, 60 degrees
        // off straight out of the edge there: the bridge heads as near
        // that as a first plank may, 45 degrees, and the builder walks
        // along it.
        let island = |at: Vec2| at.length() <= 10.0;
        let dir = Vec2::new(0.5, -0.866);
        let (builder, at, bridges, _, done) = run(Vec2::new(9.0, 2.0), dir, &island, 4.0);
        assert!(!done);
        let chain = &bridges.chains[0];
        let (root, way) = (chain.rungs[0], chain.rungs[1].d);
        assert!((way.angle_to(root.d).abs() - MAX_SLANT).abs() < 0.02, "{way} off {}", root.d);
        assert!(chain.rungs.windows(2).skip(1).all(|pair| pair[1].d.distance(way) < 1e-4));
        assert!(builder.dir.distance(way) < 1e-4 && builder.turn.is_none());
        // On the bridge's own middle line.
        assert!((at - root.p).perp_dot(way).abs() < 0.01, "{at}");
    }

    #[test]
    fn it_forks_off_a_bridge_and_gives_up_on_ground_that_never_ends() {
        // Along someone's bridge and off its side: a fork, rooted on the
        // plank's edge, square to it.
        let mut bridges = Bridges::default();
        let rungs: Vec<Rung> = (0..12).map(|k| Rung { p: Vec2::new(10.0 + k as f32 * SPACING, 0.0), d: Vec2::X }).collect();
        bridges.chains.push(Chain::new(0, 0, rungs));
        bridges.next_id = 1;
        let island = |at: Vec2| at.length() <= 10.0;
        bridges.recut(&island);
        let mut builder = Builder::new(0, Vec2::NEG_Y);
        let (mut at, mut sent) = (Vec2::new(16.0, 0.5), Vec::new());
        for _ in 0..120 {
            let (next, done) = advance(&mut builder, at, 1.0 / 60.0, &island, &mut bridges, &mut sent);
            assert!(!done);
            at = next;
            bridges.recut(&island);
            assert!(bridges.plank_under(at).is_some(), "nothing under it at {at}");
        }
        let fork = &bridges.chains[1];
        assert!(fork.forked && fork.id == 1);
        assert!((fork.rungs[0].p.y + HALF_WIDTH).abs() < 0.02 && fork.rungs[0].d.distance(Vec2::NEG_Y) < 0.02);
        assert!(at.y < -HALF_WIDTH - 5.0);
        // Over nothing to begin with, it is done at once with nothing
        // built; and on ground with no end it gives up.
        let (_, _, bridges, sent, done) = run(Vec2::new(50.0, 50.0), Vec2::X, &island, 1.0);
        assert!(done && bridges.chains.is_empty() && sent.is_empty());
        let everywhere = |_: Vec2| true;
        let (builder, _, bridges, _, done) = run(Vec2::ZERO, Vec2::X, &everywhere, 60.0);
        assert!(done && bridges.chains.is_empty() && builder.walked > GIVES_UP);
    }
}
