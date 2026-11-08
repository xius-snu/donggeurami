//! Bridges built as you walk, in a game of Bridge Wars (`arena`; since
//! 2026-11-08, Hajun: "i want to add a 'autobridge' feature ... the user
//! need to build the bridge by pressing the bridge icon (maybe the 3rd
//! button above the sword icon) and it is a toggle button that enables
//! bridging mode vs normal mode. when u toggle it on, the charater should
//! hold a small plank flat in front of him with his hands like holding a
//! baby style ... and when u are in bridging mode and u walk off the edge
//! of the island, it will place small planks beneath your ground at the
//! target direction, and the bridge can curve if you walk angularly, and
//! also the vertices of the plank is gonna match perfectly with the edge
//! of the island/edge of the bridge if ur bridging from an existing
//! bridge, and the bridge width should be same as the 4 bridges in the map
//! now, and same thickness too"). The map's own bridges went that day;
//! every bridge is built.
//!
//! The plank over the sword ([`hud::BridgeButton`], or B on desktop) turns
//! it on and off ([`Bridging`]). On, you carry a plank in front of you
//! (`Carrying`, the arms held out, with a board lying across them,
//! [`HeldPlank`]), and every frame, before you move, where you are about to
//! step is looked at ([`lay`]): over the void, a plank is laid under it.
//! Each bridge is a chain of rungs ([`Chain`], [`Rung`]), a point and a
//! heading each, with a plank between every two: a quad [`HALF_WIDTH`] out
//! either side of each rung, [`THICKNESS`] deep, its top at [`TOP`], the
//! islands' tops less a seam. A chain begins where you leave the ground
//! (Hajun's "CASE 1": a rung on the edge itself, headed straight out from
//! it, its corners pulled onto the edge so that they meet it exactly), or
//! where you step off the side of a plank (their "CASE 4", a fork: a rung
//! flush against that side), and grows a rung [`SPACING`] ahead of its tip
//! each time you come within [`AHEAD`] of the tip, headed the way you
//! walk, turned no more than [`MAX_TURN`] from the rung before, so that the
//! bridge curves as you walk round; and where the next plank would reach
//! ground it ends on the edge of it instead (their "CASE 2"), a rung on
//! that edge headed into it, with one between first if the turn is sharp.
//! Walking the far end of your own unfinished chain grows it ("CASE 3");
//! walking off the ground onto a plank is just walking ("CASE 6").
//!
//! The planks are ground: their shape is laid over the island's
//! (`Islands::lay_over`), so that you, the bots and everyone stand on them,
//! fall off their sides, and never step off the end of an unfinished chain
//! without laying another. Online every rung you lay goes to the server as
//! it is laid (`Ask::Rung`), and everyone's come back as [`Plank`]s, from
//! which the same bridges are built here ([`hear_planks`]), yours left
//! out. Nothing is taken up again: a bridge stands until the game ends.
//!
//! Each chain is one mesh, built again as it grows ([`rebuild`]): the top
//! of each plank in the file's `colours.arena.plank`, its sides and its
//! underside in `plank_side`, the two ends open (the first against the
//! edge it leaves, the last against the edge it lands on) but for the tip
//! of an unfinished chain, which is capped. The width is Hajun's bridges'
//! (2.726 m, their `bridge` object of 2026-11-08, 0.25 m thick), and a
//! plank is as long as it is wide, which lets a fork's first rung fit
//! against one plank's side, as Hajun's own rule (`s >= 2w`) asks.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::{Ask as NetAsk, Plank};

use crate::arena::{Fighting, InArena};
use crate::hud::{BridgeButton, DialogUp};
use crate::island::{Islands, Venue};
use crate::net::Online;
use crate::ui::{UiChanged, ui};
use crate::{Carrying, Dead, Intent, LAND_STEP_UP, MOVE_SPEED, PLAYER_RADIUS, Player};

/// Half the width of a plank, in metres: Hajun's bridges in
/// `pvptownmap.glb` were 2.726 m wide.
pub(crate) const HALF_WIDTH: f32 = 1.363;
/// How thick a plank is, in metres: Hajun's bridges' 0.25.
pub(crate) const THICKNESS: f32 = 0.25;
/// How far one rung is from the next, in metres: as far as a plank is
/// wide, so that a fork's first rung fits against one plank's side.
pub(crate) const SPACING: f32 = 2.0 * HALF_WIDTH;
/// How high the top of every plank lies, in metres: the islands' tops,
/// 0.25 m up, less a seam, so that where a plank's root overlaps the
/// island the island shows, and the join is the island's own edge.
pub(crate) const TOP: f32 = 0.24;
/// The most a chain turns from one rung to the next, in radians: 30
/// degrees, Hajun's number.
const MAX_TURN: f32 = 0.5236;
/// A chain that would land on an edge turned more than this from its last
/// rung, in radians (80 degrees, Hajun's), gets a rung between first.
const SHARP_TURN: f32 = 1.3963;
/// How far ahead of your middle the ground is looked at, in metres, over
/// where this frame's step takes you: your own width, and a little, so
/// that a plank is under your legs before they are over the void.
const AHEAD: f32 = PLAYER_RADIUS + 0.4;
/// How finely an edge is felt for, in metres, and how far round a point on
/// it the ground is felt for which way is out.
const EDGE: f32 = 0.02;
const RING: f32 = 0.6;
/// How finely the way ahead of a tip is felt for ground to land on, in
/// metres.
const PROBE: f32 = 0.25;
/// The most rungs one chain holds, and one game holds all told.
const MOST_RUNGS: usize = 400;
const MOST_CHAINS: usize = 200;
/// The plank you carry: how long, how thick and how wide, in metres, and
/// where it lies in front of you, up from your feet and forward.
const HELD: Vec3 = Vec3::new(1.1, 0.08, 0.5);
const HELD_AT: Vec3 = Vec3::new(0.0, 1.08, -0.56);

/// Whether you are building a bridge as you walk: the plank button's
/// toggle.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct Bridging(pub bool);

/// The plank button pressed, or B on desktop.
#[derive(Message, Default)]
pub(crate) struct BridgePressed;

/// One rung of a chain: where it lies across the ground, and which way the
/// chain heads there.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Rung {
    pub(crate) p: Vec2,
    pub(crate) d: Vec2,
}

/// One bridge: whose it is, which of theirs, its rungs in order, and what
/// is drawn of it.
struct Chain {
    seat: u8,
    id: u16,
    rungs: Vec<Rung>,
    /// Rungs that came ahead of their turn, online, to go on once those
    /// before them have.
    waiting: Vec<(u16, Rung)>,
    /// The corners of each rung, left and right, as last built: the plain
    /// ones, but for a root on an island's edge and an end on one, pulled
    /// onto the edge ([`corners_of`]).
    corners: Vec<(Vec2, Vec2)>,
    /// Whether its last rung lands on ground, as last built.
    complete: bool,
    mesh: Entity,
    handle: Handle<Mesh>,
}

/// Every bridge in the game.
#[derive(Resource, Default)]
pub(crate) struct Bridges {
    chains: Vec<Chain>,
    next_id: u16,
    /// Something has changed since the planks were last built.
    dirty: bool,
    /// The game they belong to (`Fighting::game`): another game, and they
    /// are cleared.
    game: u32,
}

impl Bridges {
    /// The chain `seat`'s `id`th, if it is here.
    fn chain(&mut self, seat: u8, id: u16) -> Option<&mut Chain> {
        self.chains.iter_mut().find(|chain| chain.seat == seat && chain.id == id)
    }

    /// The plank under `at`, across the ground, if any: which chain, and
    /// which plank of it, counted from its root.
    fn plank_under(&self, at: Vec2) -> Option<(usize, usize)> {
        self.chains.iter().enumerate().find_map(|(c, chain)| {
            (0..chain.corners.len().saturating_sub(1))
                .find(|&k| {
                    let (l0, r0) = chain.corners[k];
                    let (l1, r1) = chain.corners[k + 1];
                    inside_quad(at, [l0, r0, r1, l1])
                })
                .map(|k| (c, k))
        })
    }
}

/// The mesh of one chain.
#[derive(Component)]
struct PlankMesh;

/// The plank you carry, hung in front of you.
#[derive(Component)]
struct HeldPlank;

/// What the planks are drawn with: one material, coloured by vertex.
#[derive(Resource)]
struct Look(Handle<StandardMaterial>);

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<BridgePressed>()
        .init_resource::<Bridges>()
        .init_resource::<Bridging>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (toggle, hear_planks).before(crate::read_player_input),
                (rebuild, carry, light_button).after(lay),
            ),
        );
}

fn setup(mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    commands.insert_resource(Look(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        ..default()
    })));
}

/// Turns building on and off at the plank button, or B, in a game of Bridge
/// Wars while it is on; off anywhere else, and as a game ends.
fn toggle(
    mut presses: MessageReader<BridgePressed>,
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Res<DialogUp>,
    fighting: Res<Fighting>,
    me: Query<&Venue, With<Player>>,
    mut bridging: ResMut<Bridging>,
) {
    let pressed = presses.read().count() > 0 || keyboard.just_pressed(KeyCode::KeyB);
    let in_a_game = fighting.playing && me.single().is_ok_and(|&here| here == Venue::Arena);
    if !in_a_game {
        bridging.set_if_neq(Bridging(false));
        return;
    }
    if pressed && !dialog.0 {
        bridging.0 = !bridging.0;
        info!("building a bridge: {}", if bridging.0 { "on" } else { "off" });
    }
}

/// Lights the plank button while building is on.
fn light_button(bridging: Res<Bridging>, mut buttons: Query<&mut BackgroundColor, With<BridgeButton>>) {
    let colours = ui().colours.hud;
    for mut fill in &mut buttons {
        fill.set_if_neq(BackgroundColor(if bridging.0 { colours.button_held } else { colours.button }));
    }
}

/// Has you carry a plank while building is on and you are alive, and not
/// otherwise: the arms held out (`Carrying`), and the board across them.
fn carry(
    bridging: Res<Bridging>,
    look: Res<Look>,
    me: Query<(Entity, Has<Dead>, Has<Carrying>), With<Player>>,
    mut held: Query<&mut Visibility, With<HeldPlank>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let Ok((you, dead, carrying)) = me.single() else {
        return;
    };
    let wanted = bridging.0 && !dead;
    if wanted != carrying {
        if wanted {
            commands.entity(you).insert(Carrying);
        } else {
            commands.entity(you).remove::<Carrying>();
        }
    }
    let shown = if wanted { Visibility::Inherited } else { Visibility::Hidden };
    match held.single_mut() {
        Ok(mut visibility) => {
            visibility.set_if_neq(shown);
        }
        Err(_) => {
            let colour = ui().colours.arena.plank.to_linear();
            let mut mesh = Mesh::from(Cuboid::new(HELD.x, HELD.y, HELD.z));
            let count = mesh.count_vertices();
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[colour.red, colour.green, colour.blue, 1.0]; count]);
            commands.entity(you).with_child((
                HeldPlank,
                Name::new("Held plank"),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(look.0.clone()),
                Transform::from_translation(HELD_AT),
                shown,
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
    }
}

/// Lays a plank under where you are about to step, if that is over the
/// void and you are building: before you move, so that it is there when
/// your feet are.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lay(
    time: Res<Time>,
    bridging: Res<Bridging>,
    fighting: Res<Fighting>,
    online: Res<Online>,
    islands: Res<Islands>,
    me: Query<(&Transform, &Venue, &Intent, Has<Dead>), With<Player>>,
    mut bridges: ResMut<Bridges>,
    mut requests: MessageWriter<NetAsk>,
) {
    if !bridging.0 || !fighting.playing {
        return;
    }
    let Ok((body, &venue, intent, dead)) = me.single() else {
        return;
    };
    if dead || venue != Venue::Arena || intent.walk.length() < 0.1 {
        return;
    }
    let Some(island) = islands.get(Venue::Arena) else {
        return;
    };
    let dir = intent.walk.xz().normalize();
    let speed = MOVE_SPEED * intent.sprint * intent.walk.length().min(1.0);
    let feet = body.translation.y;
    let here = body.translation.xz();
    let reach = feet + LAND_STEP_UP;
    let ground = |at: Vec2| island.floor(at, reach).is_some();
    let land = |at: Vec2| island.floor_base(at, reach).is_some();
    let probe = here + dir * (speed * time.delta_secs() + AHEAD);
    if ground(probe) || !ground(here) {
        // Ground ahead, planks or no planks; or already over the void,
        // with nothing to build from.
        return;
    }
    let Some(seat) = fighting.seat else {
        return;
    };
    let new_rungs: Vec<(u16, u16, Rung)>;
    match bridges.plank_under(here) {
        Some((c, k)) => {
            let chain = &bridges.chains[c];
            let tip = *chain.rungs.last().expect("a chain with a plank has rungs");
            let own_tip = chain.seat == seat && !chain.complete && k + 2 == chain.rungs.len();
            if own_tip && (probe - tip.p).dot(tip.d) > 0.0 {
                // Off the far end of your own unfinished bridge: on it goes.
                let c_id = chain.id;
                let added = extend(&mut bridges.chains[c], dir, &land);
                new_rungs = added.into_iter().map(|(index, rung)| (c_id, index, rung)).collect();
            } else {
                // Off the side of a plank, anyone's: a fork, flush against
                // that side.
                let (l0, r0) = chain.corners[k];
                let (l1, r1) = chain.corners[k + 1];
                let along = (chain.rungs[k + 1].p - chain.rungs[k].p).normalize_or(tip.d);
                let right = perp(along).dot(here - chain.rungs[k].p) > 0.0;
                let (start, end) = if right { (r0, r1) } else { (l0, l1) };
                let edge = end - start;
                let length = edge.length();
                let t = if length > 2.0 * HALF_WIDTH {
                    ((here - start).dot(edge) / length).clamp(HALF_WIDTH, length - HALF_WIDTH) / length
                } else {
                    0.5
                };
                let e = start + edge * t;
                let out = if right { perp(edge.normalize_or(along)) } else { -perp(edge.normalize_or(along)) };
                if bridges.chains.len() >= MOST_CHAINS {
                    return;
                }
                let id = begin(&mut bridges, seat, Rung { p: e, d: out });
                let c = bridges.chains.len() - 1;
                let mut added = vec![(id, 0u16, Rung { p: e, d: out })];
                added.extend(extend(&mut bridges.chains[c], dir, &land).into_iter().map(|(index, rung)| (id, index, rung)));
                new_rungs = added;
            }
        }
        None => {
            // Off the ground: a rung on its very edge, headed straight out.
            let e = bisect(here, probe, &ground);
            let out = outward(e, &ground).unwrap_or(dir);
            if bridges.chains.len() >= MOST_CHAINS {
                return;
            }
            let id = begin(&mut bridges, seat, Rung { p: e, d: out });
            let c = bridges.chains.len() - 1;
            let mut added = vec![(id, 0u16, Rung { p: e, d: out })];
            added.extend(extend(&mut bridges.chains[c], dir, &land).into_iter().map(|(index, rung)| (id, index, rung)));
            new_rungs = added;
        }
    }
    if new_rungs.is_empty() {
        return;
    }
    bridges.dirty = true;
    if online.is() && fighting.online {
        for (chain, index, rung) in new_rungs {
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

/// Begins a chain of `seat`'s at `root`, and says which of theirs it is.
fn begin(bridges: &mut Bridges, seat: u8, root: Rung) -> u16 {
    let id = bridges.next_id;
    bridges.next_id = bridges.next_id.wrapping_add(1);
    bridges.chains.push(Chain {
        seat,
        id,
        rungs: vec![root],
        waiting: Vec::new(),
        corners: Vec::new(),
        complete: false,
        mesh: Entity::PLACEHOLDER,
        handle: Handle::default(),
    });
    id
}

/// Grows `chain` by a rung [`SPACING`] past its tip, headed `heading` as
/// far as the turn allows, or, where that plank would reach land, by a
/// rung on the edge of the land headed into it (with one between first,
/// at a sharp turn): the rungs added, each with its index.
fn extend(chain: &mut Chain, heading: Vec2, land: &dyn Fn(Vec2) -> bool) -> Vec<(u16, Rung)> {
    if chain.complete || chain.rungs.len() >= MOST_RUNGS {
        return Vec::new();
    }
    let tip = *chain.rungs.last().expect("a chain has a root");
    let d = clamp_turn(heading, tip.d, MAX_TURN);
    let mut added = Vec::new();
    let mut push = |chain: &mut Chain, rung: Rung| {
        added.push((chain.rungs.len() as u16, rung));
        chain.rungs.push(rung);
    };
    // Land ahead within the plank's length: the chain ends on its edge.
    let steps = (SPACING / PROBE).ceil() as u32;
    let landing = (1..=steps)
        .map(|step| tip.p + d * (step as f32 * PROBE).min(SPACING))
        .find(|&at| land(at));
    if let Some(hit) = landing {
        let e = bisect(hit, tip.p, land);
        let into = outward(e, land).map_or(d, |out| -out);
        if tip.d.angle_to(into).abs() > SHARP_TURN {
            push(chain, Rung {
                p: (tip.p + e) * 0.5,
                d: (tip.d + into).normalize_or(d),
            });
        }
        push(chain, Rung { p: e, d: into });
        chain.complete = true;
    } else {
        push(chain, Rung { p: tip.p + d * SPACING, d });
    }
    added
}

/// `heading` turned no more than `most` radians from `from`, both unit.
fn clamp_turn(heading: Vec2, from: Vec2, most: f32) -> Vec2 {
    let turn = from.angle_to(heading);
    if turn.abs() <= most {
        heading
    } else {
        Vec2::from_angle(turn.signum() * most).rotate(from)
    }
}

/// The point between `on` and `off` where `ground` ends, to within
/// [`EDGE`]: `on` is over ground and `off` is not.
fn bisect(on: Vec2, off: Vec2, ground: &dyn Fn(Vec2) -> bool) -> Vec2 {
    let (mut a, mut b) = (on, off);
    while a.distance(b) > EDGE {
        let middle = (a + b) * 0.5;
        if ground(middle) {
            a = middle;
        } else {
            b = middle;
        }
    }
    (a + b) * 0.5
}

/// Which way is off the ground at `e`, a point on its edge: toward where
/// the ground is not, round a ring [`RING`] out; or none, with ground, or
/// none, all round.
fn outward(e: Vec2, ground: &dyn Fn(Vec2) -> bool) -> Option<Vec2> {
    let sum: Vec2 = (0..16)
        .map(|k| Vec2::from_angle(k as f32 * TAU / 16.0) * RING)
        .filter(|&round| !ground(e + round))
        .sum();
    sum.try_normalize()
}

/// Perpendicular to `d`, to its right as the chain heads.
fn perp(d: Vec2) -> Vec2 {
    Vec2::new(-d.y, d.x)
}

/// The turn about the upright that heads `d`, as `Spot` has a turn: the
/// game's -Z turned by it is `d`.
fn yaw_of(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}

/// The heading a turn of `yaw` about the upright gives.
fn heading_of(yaw: f32) -> Vec2 {
    Vec2::new(-yaw.sin(), -yaw.cos())
}

/// Whether `at` is inside the convex quad `q`, its corners in order round
/// it either way.
fn inside_quad(at: Vec2, q: [Vec2; 4]) -> bool {
    let mut sign = 0.0f32;
    for k in 0..4 {
        let edge = q[(k + 1) % 4] - q[k];
        let side = edge.perp_dot(at - q[k]);
        if side.abs() < 1e-6 {
            continue;
        }
        if sign == 0.0 {
            sign = side.signum();
        } else if side.signum() != sign {
            return false;
        }
    }
    true
}

/// The left and right corners of each rung of `rungs`: [`HALF_WIDTH`] out
/// either side, but for a root on the edge of the land and an end on one,
/// whose corners are pulled onto the edge, in or out along the rung's
/// heading, so that the plank meets the edge exactly however it curves.
/// And whether the chain ends on land.
fn corners_of(rungs: &[Rung], land: &dyn Fn(Vec2) -> bool) -> (Vec<(Vec2, Vec2)>, bool) {
    let plain = |rung: &Rung| (rung.p - perp(rung.d) * HALF_WIDTH, rung.p + perp(rung.d) * HALF_WIDTH);
    let mut corners: Vec<(Vec2, Vec2)> = rungs.iter().map(plain).collect();
    let Some((first, last)) = rungs.first().zip(rungs.last()) else {
        return (corners, false);
    };
    // Onto the edge of the land on the `inland` side of the rung: a corner
    // over the void is drawn in, one over the land pushed out.
    let onto_edge = |corner: Vec2, inland: Vec2| -> Vec2 {
        let far = 2.0 * HALF_WIDTH;
        if land(corner) {
            if land(corner - inland * far) {
                return corner;
            }
            bisect(corner, corner - inland * far, land)
        } else {
            if !land(corner + inland * far) {
                return corner;
            }
            bisect(corner + inland * far, corner, land)
        }
    };
    if land(first.p - first.d * 0.3) {
        let (l, r) = corners[0];
        corners[0] = (onto_edge(l, -first.d), onto_edge(r, -first.d));
    }
    let complete = rungs.len() > 1 && land(last.p + last.d * 0.3);
    if complete {
        let end = corners.len() - 1;
        let (l, r) = corners[end];
        corners[end] = (onto_edge(l, last.d), onto_edge(r, last.d));
    }
    (corners, complete)
}

/// Takes in every rung the server says someone has laid ([`Plank`]), but
/// your own, which are laid here as you lay them, and builds their bridges.
fn hear_planks(
    fighting: Res<Fighting>,
    planks: Query<&Plank, (With<Remote>, Added<Plank>)>,
    mut bridges: ResMut<Bridges>,
) {
    for plank in &planks {
        if fighting.seat == Some(plank.seat) {
            continue;
        }
        let rung = Rung {
            p: Vec2::new(plank.x, plank.z),
            d: heading_of(plank.heading),
        };
        let found = bridges.chain(plank.seat, plank.chain).is_some();
        if !found {
            if bridges.chains.len() >= MOST_CHAINS {
                continue;
            }
            bridges.chains.push(Chain {
                seat: plank.seat,
                id: plank.chain,
                rungs: Vec::new(),
                waiting: Vec::new(),
                corners: Vec::new(),
                complete: false,
                mesh: Entity::PLACEHOLDER,
                handle: Handle::default(),
            });
        }
        let chain = bridges.chain(plank.seat, plank.chain).expect("just found or made");
        if usize::from(plank.index) < chain.rungs.len() || chain.rungs.len() >= MOST_RUNGS {
            continue;
        }
        chain.waiting.push((plank.index, rung));
        chain.waiting.sort_by_key(|(index, _)| *index);
        while chain.waiting.first().is_some_and(|(index, _)| usize::from(*index) == chain.rungs.len()) {
            let (_, rung) = chain.waiting.remove(0);
            chain.rungs.push(rung);
        }
        bridges.dirty = true;
    }
}

/// Builds every chain's mesh again as it changes, or as the file's colours
/// do, and lays the planks' shape over the island; and clears everything
/// out of the arena, or as another game begins.
#[allow(clippy::too_many_arguments)]
fn rebuild(
    fighting: Res<Fighting>,
    changed: Res<UiChanged>,
    look: Res<Look>,
    me: Query<&Venue, With<Player>>,
    mut bridges: ResMut<Bridges>,
    mut islands: ResMut<Islands>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let here = me.single().ok().copied();
    if here != Some(Venue::Arena) || fighting.game != bridges.game {
        if !bridges.chains.is_empty() {
            for chain in bridges.chains.drain(..) {
                if chain.mesh != Entity::PLACEHOLDER {
                    commands.entity(chain.mesh).try_despawn();
                }
            }
            islands.lay_over(Venue::Arena, None);
            info!("the bridges are cleared away");
        }
        bridges.next_id = 0;
        bridges.dirty = false;
        bridges.game = fighting.game;
        return;
    }
    let restyled = changed.is_changed() && !changed.is_added();
    if !bridges.dirty && !restyled {
        return;
    }
    let Some(island) = islands.get(Venue::Arena) else {
        return;
    };
    let land = |at: Vec2| island.floor_base(at, TOP + LAND_STEP_UP).is_some();
    let colours = ui().colours.arena;
    let mut parts = Vec::new();
    for chain in &mut bridges.chains {
        let (corners, complete) = corners_of(&chain.rungs, &land);
        chain.corners = corners;
        chain.complete = complete;
        if chain.rungs.len() < 2 {
            continue;
        }
        let (mesh, triangles) = planks(&chain.corners, chain.complete, colours.plank, colours.plank_side);
        parts.push(triangles);
        if chain.mesh == Entity::PLACEHOLDER {
            chain.handle = meshes.add(mesh);
            chain.mesh = commands
                .spawn((
                    InArena,
                    PlankMesh,
                    Name::new(format!("Bridge {} of seat {}", chain.id, chain.seat)),
                    Venue::Arena,
                    Mesh3d(chain.handle.clone()),
                    MeshMaterial3d(look.0.clone()),
                    Transform::default(),
                    NotShadowCaster,
                ))
                .id();
        } else if meshes.insert(&chain.handle, mesh).is_err() {
            warn!("a bridge has lost its mesh");
        }
    }
    bridges.dirty = false;
    islands.lay_over(Venue::Arena, (!parts.is_empty()).then_some(parts));
}

/// The planks between the rungs with these `corners`, as a mesh coloured by
/// vertex, `top` on top and `side` everywhere else, and as the triangles
/// bodies stand on: each plank's top, underside, left side and right side,
/// and a cap across the tip if the chain is not `complete`.
fn planks(corners: &[(Vec2, Vec2)], complete: bool, top: Color, side: Color) -> (Mesh, Vec<[Vec3; 3]>) {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colours: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut triangles = Vec::new();
    let tint = |colour: Color| {
        let linear = colour.to_linear();
        [linear.red, linear.green, linear.blue, 1.0]
    };
    let (top_tint, side_tint) = (tint(top), tint(side));
    let up = |at: Vec2| Vec3::new(at.x, TOP, at.y);
    let down = |at: Vec2| Vec3::new(at.x, TOP - THICKNESS, at.y);
    // One face: four corners round it, facing `out`, wound to face that way.
    let mut face = |a: Vec3, b: Vec3, c: Vec3, d: Vec3, out: Vec3, colour: [f32; 4]| {
        let (b, d) = if (b - a).cross(c - a).dot(out) < 0.0 { (d, b) } else { (b, d) };
        let first = positions.len() as u32;
        for corner in [a, b, c, d] {
            positions.push(corner.to_array());
            normals.push(out.to_array());
            colours.push(colour);
            uvs.push([0.0, 0.0]);
        }
        indices.extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        triangles.push([a, b, c]);
        triangles.push([a, c, d]);
    };
    for k in 0..corners.len().saturating_sub(1) {
        let (l0, r0) = corners[k];
        let (l1, r1) = corners[k + 1];
        let along = ((l1 + r1) - (l0 + r0)).normalize_or(Vec2::X);
        let left_out = -perp(along);
        face(up(l0), up(r0), up(r1), up(l1), Vec3::Y, top_tint);
        face(down(l0), down(r0), down(r1), down(l1), Vec3::NEG_Y, side_tint);
        face(up(l0), up(l1), down(l1), down(l0), Vec3::new(left_out.x, 0.0, left_out.y), side_tint);
        face(up(r0), up(r1), down(r1), down(r0), Vec3::new(-left_out.x, 0.0, -left_out.y), side_tint);
        if k + 2 == corners.len() && !complete {
            face(up(l1), up(r1), down(r1), down(l1), Vec3::new(along.x, 0.0, along.y), side_tint);
        }
    }
    let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices));
    (mesh, triangles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::island::Island;

    /// A disc of land `radius` across from its middle at `centre`, its
    /// top at 0.25 m, as the arena's islands are.
    fn disc(centre: Vec2, radius: f32) -> Vec<[Vec3; 3]> {
        let at = |angle: f32| Vec3::new(centre.x + angle.cos() * radius, 0.25, centre.y + angle.sin() * radius);
        let middle = Vec3::new(centre.x, 0.25, centre.y);
        (0..64)
            .map(|k| {
                let (a, b) = (k as f32 * TAU / 64.0, (k + 1) as f32 * TAU / 64.0);
                // Wound to face up: in (x, z) the other way round than on a
                // page.
                [middle, at(b), at(a)]
            })
            .collect()
    }

    fn is_land(island: &Island) -> impl Fn(Vec2) -> bool + '_ {
        move |at| island.floor_base(at, 1.0).is_some()
    }

    #[test]
    fn a_turn_is_clamped_and_a_heading_goes_round_trip() {
        let east = Vec2::X;
        assert!(clamp_turn(Vec2::new(1.0, 0.2).normalize(), east, MAX_TURN).distance(Vec2::new(1.0, 0.2).normalize()) < 1e-6);
        let turned = clamp_turn(Vec2::Y, east, MAX_TURN);
        assert!((east.angle_to(turned) - MAX_TURN).abs() < 1e-5, "{turned}");
        let back = clamp_turn(-east, east, MAX_TURN);
        assert!((east.angle_to(back).abs() - MAX_TURN).abs() < 1e-5);
        for d in [Vec2::X, Vec2::NEG_Y, Vec2::new(0.6, 0.8), Vec2::new(-0.3, -0.954).normalize()] {
            assert!(heading_of(yaw_of(d)).distance(d) < 1e-5, "{d}");
        }
        assert!(inside_quad(Vec2::new(0.5, 0.5), [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]));
        assert!(!inside_quad(Vec2::new(1.5, 0.5), [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]));
        assert!(inside_quad(Vec2::new(0.5, 0.5), [Vec2::Y, Vec2::ONE, Vec2::X, Vec2::ZERO]));
    }

    #[test]
    fn leaving_an_island_the_first_rung_lies_on_its_edge_and_meets_it_at_the_corners() {
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        // Walking east from 9 m out: the edge is at 10.
        let e = bisect(Vec2::new(9.0, 0.0), Vec2::new(11.0, 0.0), &land);
        assert!((e.x - 10.0).abs() < EDGE && e.y.abs() < 1e-6, "{e}");
        let out = outward(e, &land).unwrap();
        assert!(out.distance(Vec2::X) < 0.05, "{out}");
        // Its corners, plain, would stand off the circle; pulled onto it,
        // each is on the edge.
        let root = Rung { p: e, d: out };
        let (corners, complete) = corners_of(&[root, Rung { p: e + out * SPACING, d: out }], &land);
        assert!(!complete);
        for corner in [corners[0].0, corners[0].1] {
            assert!((corner.length() - 10.0).abs() < 0.03, "{corner} is off the edge");
            assert!((corner.y.abs() - HALF_WIDTH).abs() < 0.02, "{corner} is not a plank's width out");
        }
        // The next rung's corners are the plain ones.
        assert!((corners[1].0 - (root.p + out * SPACING - perp(out) * HALF_WIDTH)).length() < 1e-5);
    }

    #[test]
    fn a_chain_curves_as_you_walk_round_and_lands_on_the_far_shore() {
        let mut corners = disc(Vec2::ZERO, 10.0);
        corners.extend(disc(Vec2::new(30.0, 8.0), 10.0));
        let island = Island::new(&corners);
        let land = is_land(&island);
        let mut chain = Chain {
            seat: 0,
            id: 0,
            rungs: vec![Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X }],
            waiting: Vec::new(),
            corners: Vec::new(),
            complete: false,
            mesh: Entity::PLACEHOLDER,
            handle: Handle::default(),
        };
        // Walking north-east, the chain turns toward it, never more than
        // the clamp a rung, and each rung is a plank's length on.
        let heading = Vec2::new(1.0, 1.0).normalize();
        let mut laid = 0;
        while !chain.complete && laid < 40 {
            let added = extend(&mut chain, heading, &land);
            assert!(!added.is_empty());
            laid += 1;
        }
        assert!(chain.complete, "never landed");
        // Every rung a plank's length on from the last, turned no more
        // than the clamp, but for the landing, which heads into the shore
        // however that lies (with a rung between at a sharp turn).
        let n = chain.rungs.len();
        for (k, pair) in chain.rungs.windows(2).enumerate() {
            let turn = pair[0].d.angle_to(pair[1].d).abs();
            if k + 3 < n {
                assert!(turn <= MAX_TURN + 1e-4, "rung {k}: turned {turn}");
            } else {
                assert!(turn <= SHARP_TURN + 1e-4, "rung {k}: turned {turn} landing");
            }
            let apart = pair[0].p.distance(pair[1].p);
            assert!(apart <= SPACING + 1e-4, "{apart} m between rungs");
        }
        let turned = chain.rungs.iter().skip(1).take(3).map(|rung| rung.d.angle_to(Vec2::X).abs()).collect::<Vec<_>>();
        assert!(turned[0] > 0.4 && turned.iter().any(|&turn| turn > 0.7), "{turned:?}");
        // The last rung lies on the far island's edge, headed into it, and
        // every rung between is over the void.
        let last = chain.rungs.last().unwrap();
        assert!((last.p.distance(Vec2::new(30.0, 8.0)) - 10.0).abs() < 0.05, "{} off the shore", last.p);
        assert!(land(last.p + last.d * 0.5) && !land(last.p - last.d * 0.5));
        for rung in &chain.rungs[1..chain.rungs.len() - 1] {
            assert!(!land(rung.p), "{} is over land", rung.p);
        }
        // Built, the planks face out, stand at the right height, and are
        // found under a point on them; the chain has no cap.
        let (built, complete) = corners_of(&chain.rungs, &land);
        assert!(complete);
        let (mesh, triangles) = planks(&built, complete, Color::WHITE, Color::BLACK);
        assert_eq!(triangles.len(), (built.len() - 1) * 8);
        assert_eq!(mesh.count_vertices(), (built.len() - 1) * 16);
        for [a, b, c] in &triangles {
            let normal = (*b - *a).cross(*c - *a);
            let middle = (*a + *b + *c) / 3.0;
            // Up on top, down under, and outward on the sides: away from
            // the chain's spine.
            if (middle.y - TOP).abs() < 1e-4 {
                assert!(normal.y > 0.0);
            } else if (middle.y - (TOP - THICKNESS)).abs() < 1e-4 {
                assert!(normal.y < 0.0);
            } else {
                assert!(normal.y.abs() < 1e-4);
            }
        }
        let mut bridges = Bridges::default();
        chain.corners = built.clone();
        bridges.chains.push(chain);
        let on = bridges.chains[0].rungs[2].p + bridges.chains[0].rungs[2].d * 0.5;
        assert_eq!(bridges.plank_under(on), Some((0, 2)));
        assert_eq!(bridges.plank_under(Vec2::new(0.0, -30.0)), None);
        // And an unfinished chain is capped at its tip.
        let (open, _) = planks(&built[..3], false, Color::WHITE, Color::BLACK);
        assert_eq!(open.count_vertices(), 2 * 16 + 4);
    }

    /// Takes in `plank` as [`hear_planks`] does, for someone else's chain.
    fn take(bridges: &mut Bridges, plank: Plank) {
        let rung = Rung {
            p: Vec2::new(plank.x, plank.z),
            d: heading_of(plank.heading),
        };
        if bridges.chain(plank.seat, plank.chain).is_none() {
            bridges.chains.push(Chain {
                seat: plank.seat,
                id: plank.chain,
                rungs: Vec::new(),
                waiting: Vec::new(),
                corners: Vec::new(),
                complete: false,
                mesh: Entity::PLACEHOLDER,
                handle: Handle::default(),
            });
        }
        let chain = bridges.chain(plank.seat, plank.chain).unwrap();
        chain.waiting.push((plank.index, rung));
        chain.waiting.sort_by_key(|(index, _)| *index);
        while chain.waiting.first().is_some_and(|(index, _)| usize::from(*index) == chain.rungs.len()) {
            let (_, rung) = chain.waiting.remove(0);
            chain.rungs.push(rung);
        }
    }

    #[test]
    fn rungs_arriving_out_of_turn_wait_for_those_before_them() {
        let mut bridges = Bridges::default();
        let plank = |index: u16| Plank {
            seat: 3,
            chain: 7,
            index,
            x: index as f32 * SPACING,
            z: 0.0,
            heading: yaw_of(Vec2::X),
        };
        take(&mut bridges, plank(2));
        take(&mut bridges, plank(0));
        assert_eq!(bridges.chains[0].rungs.len(), 1);
        take(&mut bridges, plank(1));
        assert_eq!(bridges.chains[0].rungs.len(), 3);
        assert!(bridges.chains[0].waiting.is_empty());
        assert!((bridges.chains[0].rungs[2].p.x - 2.0 * SPACING).abs() < 1e-4);
    }
}
