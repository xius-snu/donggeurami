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
//! [`HeldPlank`]), walk at half the pace ([`CARRY_SLOW`]), throw no punch
//! and show no hit range (2026-11-09); and the bridge follows your feet
//! ([`lay`]): a chain of rungs, a point and a heading each, with a plank
//! between every two ([`Chain`], [`Rung`]), a quad [`HALF_WIDTH`] out
//! either side of each rung and [`THICKNESS`] deep, its top at [`TOP`].
//!
//! **The bridge is a ribbon laid under where you walk** (2026-11-09, after
//! Hajun found the first version "weird and choppy" at the edges and in a
//! sharp turn: "think of a smart way to make briding feel good and look
//! good no matter what"). The chain you are building has a tip plank that
//! is not fixed yet: it runs from the last fixed rung to a little past
//! you ([`AHEAD`]), re-aimed at you every frame, so that whichever way you
//! step over the void there is a plank under you, and grows as you walk;
//! each time you are a plank's length ([`SPACING`]) past the last fixed
//! rung, a rung is fixed on the line toward you, so that the fixed planks
//! trace the path you actually walked, with a mitred joint at each rung,
//! turned no more than [`MAX_TURN`] a rung so that no plank folds. A chain
//! begins where you leave the ground, a rung on its very edge headed
//! straight out from it with its corners pulled onto the edge, or where you
//! step off the side of a plank, a rung flush against that side (a fork);
//! and where you step onto land it ends on the land's edge, a rung headed
//! into it. Walking back onto the ground you came from with nothing built
//! yet forgets the stub. Turning building off, or dying, fixes the tip
//! where it is.
//!
//! Every plank costs its length in metres of bridge, which you buy (Hajun:
//! "players will buy the bridge by meter count ... for now make each person
//! start with 100m"): the file's `arena.bridge_metres` to start, the number
//! by the log top left (`arena::Resources`), and the tip never reaches
//! further than you have left.
//!
//! The planks are ground: their shape is laid over the island's
//! (`Islands::lay_over`), so that you, the bots and everyone stand on them
//! and fall off their sides. Online every rung you fix goes to the server
//! as it is fixed (`Ask::Rung`), and everyone's come back as [`Plank`]s,
//! from which the same bridges are built here ([`hear_planks`]), yours left
//! out; the tip is yours alone until it is fixed. Each chain is one mesh,
//! built again as it changes ([`rebuild`]), the planks' tops alternating
//! between the file's `colours.arena.plank` and a shade lighter, their
//! sides and undersides in `plank_side`, the ends open but for a tip's cap.
//! The width is Hajun's bridges' (2.726 m, their `bridge` object of
//! 2026-11-08, 0.25 m thick); a plank is half as long as it is wide, like
//! the boards of a dock, which follows a curve closely.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::{Ask as NetAsk, Plank};

use crate::arena::{Fighting, InArena, Resources};
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
/// How far one fixed rung is from the next, in metres: half as far as a
/// plank is wide, so that the planks follow a curve closely, like the
/// boards of a dock.
pub(crate) const SPACING: f32 = HALF_WIDTH;
/// How high the top of every plank lies, in metres: the islands' tops,
/// 0.25 m up, less a seam, so that where a plank's root overlaps the
/// island the island shows, and the join is the island's own edge.
pub(crate) const TOP: f32 = 0.24;
/// How much slower you walk carrying a plank, as a share of your pace.
pub(crate) const CARRY_SLOW: f32 = 0.5;
/// The most a chain turns from one rung to the next, in radians: 35
/// degrees. The joint between two planks is mitred ([`corners_of`]), and
/// a mitre pulls each inner corner back along its plank by
/// `HALF_WIDTH * tan(turn / 2)`; a plank with a mitre at both ends folds,
/// its inner edge gone, once twice that passes [`SPACING`], and the first
/// plank, square to the edge it leaves, once `HALF_WIDTH * sin(turn)` and
/// one mitre do: 35 degrees leaves it a tenth of its length. A chain turns
/// round in about 2.3 m. The rung that lands a chain is clamped by it too.
const MAX_TURN: f32 = 0.6109;
/// How far past your middle the tip plank reaches, in metres: your own
/// width, and a little, so that a plank is under your legs before they are
/// over the void. The tip is never shorter, however few metres are left.
const AHEAD: f32 = PLAYER_RADIUS + 0.6;
/// The shortest plank that lands a chain, or is left when a chain is let
/// go, in metres: shorter, the mitre at its start would reach past its
/// end. A chain landing nearer the shore than that ends on its last rung,
/// and its corners are pushed onto the shore.
const MIN_LAND: f32 = SPACING * 0.5;
/// How far a corner is pushed out to meet an edge, at the most, in metres,
/// and how far ahead of a chain's end the shore is looked for.
const ONTO_EDGE: f32 = 2.0 * SPACING;
/// How finely the shore is looked for ahead, in metres.
const SHORE_STEP: f32 = 0.1;
/// How far ahead of your middle the ground is looked at to begin a chain,
/// in metres, over where this frame's step takes you.
const LOOK: f32 = PLAYER_RADIUS + 0.4;
/// How far back behind a chain's root you have to be, in metres, for a
/// chain with nothing built yet to be forgotten: further inland than
/// anyone begins one ([`LOOK`] and a step), or the chain is begun and
/// forgotten every other frame as you come up to the edge, its tip
/// flickering and a root sent to the server each time (so it was for a
/// day: a bridge's chain was the fifth begun).
const FORGET_BEHIND: f32 = LOOK + 0.7;
/// How finely an edge is felt for, in metres, and how far round a point on
/// it the ground is felt for which way is out.
const EDGE: f32 = 0.02;
const RING: f32 = 0.6;
/// How far the tip has to move, in metres, or turn, in radians, before the
/// planks' shape is laid over the island again, and the minimap drawn
/// again: the tip reaches [`AHEAD`] past you, so the shape may lag it by
/// this. The planks are drawn again whenever the tip moves at all.
const OVERLAY_MOVE: f32 = 0.2;
const OVERLAY_TURN: f32 = 0.05;
/// How many heights the planks' tops are spread over, a millimetre apart,
/// by chain, so that two chains crossing never flicker where they overlap
/// ([`top_of`]).
const TOPS: u32 = 8;
/// The most rungs one chain holds, and one game holds all told.
const MOST_RUNGS: usize = 600;
const MOST_CHAINS: usize = 200;
/// The plank you carry: how long, how thick and how wide, in metres, and
/// where it lies in front of you, up from your feet and forward.
const HELD: Vec3 = Vec3::new(1.1, 0.08, 0.5);
const HELD_AT: Vec3 = Vec3::new(0.0, 1.08, -0.56);
/// How much lighter every other plank's top is, as a share.
const LIGHTER: f32 = 0.08;

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

/// One bridge: whose it is, which of theirs, its fixed rungs in order, the
/// tip not fixed yet, and what is drawn of it.
struct Chain {
    seat: u8,
    id: u16,
    rungs: Vec<Rung>,
    /// The end of the plank being laid, past the last fixed rung, aimed at
    /// its builder: yours alone, never sent, fixed once you are a plank's
    /// length on.
    tip: Option<Rung>,
    /// The tip as last drawn, and as last laid over the island, to build
    /// again once it has moved.
    built_tip: Option<Rung>,
    overlaid_tip: Option<Rung>,
    /// Rungs that came ahead of their turn, online, to go on once those
    /// before them have.
    waiting: Vec<(u16, Rung)>,
    /// The corners of each rung, left and right, as last built, the tip's
    /// last: the plain ones, but for a root on an island's edge and an end
    /// on one, pulled onto the edge ([`corners_of`]).
    corners: Vec<(Vec2, Vec2)>,
    /// Whether its last rung lands on ground, as last built.
    complete: bool,
    mesh: Entity,
    handle: Handle<Mesh>,
}

impl Chain {
    fn new(seat: u8, id: u16, rungs: Vec<Rung>) -> Self {
        Self {
            seat,
            id,
            rungs,
            tip: None,
            built_tip: None,
            overlaid_tip: None,
            waiting: Vec::new(),
            corners: Vec::new(),
            complete: false,
            mesh: Entity::PLACEHOLDER,
            handle: Handle::default(),
        }
    }

    /// Its rungs with the tip after them: what is built.
    fn all(&self) -> Vec<Rung> {
        self.rungs.iter().copied().chain(self.tip).collect()
    }

    /// Whether the plank `k`, between its `k`th and `k+1`th rungs as built,
    /// is a fixed one, rather than the tip's.
    fn fixed(&self, k: usize) -> bool {
        k + 1 < self.rungs.len()
    }

    /// How high its planks' tops lie.
    fn top(&self) -> f32 {
        top_of(self.seat, self.id)
    }
}

/// How high the tops of `seat`'s `id`th chain lie, in metres: [`TOP`], or
/// up to [`TOPS`] millimetres under it, the same on every device, so that
/// where two chains cross one lies on the other rather than flickering
/// through it.
fn top_of(seat: u8, id: u16) -> f32 {
    TOP - 0.001 * ((u32::from(seat) * 3 + u32::from(id)) % TOPS) as f32
}

/// Every bridge in the game.
#[derive(Resource, Default)]
pub(crate) struct Bridges {
    chains: Vec<Chain>,
    next_id: u16,
    /// The chain you are building now, if any.
    active: Option<usize>,
    /// Something has changed since the planks were last built.
    dirty: bool,
    /// Counts up each time the planks are built again, for the minimap.
    pub(crate) version: u32,
    /// The game they belong to (`Fighting::game`): another game, and they
    /// are cleared.
    game: u32,
}

/// What became of the chain you were building, this frame ([`grow`]).
#[derive(Default, Debug, PartialEq)]
struct Grown {
    /// Rungs fixed, each with its index, to send.
    added: Vec<(u16, Rung)>,
    /// The chain ended on land.
    landed: bool,
    /// The chain is done with, landed or left.
    done: bool,
}

impl Bridges {
    /// The chain `seat`'s `id`th, if it is here.
    fn chain(&mut self, seat: u8, id: u16) -> Option<&mut Chain> {
        self.chains.iter_mut().find(|chain| chain.seat == seat && chain.id == id)
    }

    /// The fixed plank under `at`, across the ground, if any: which chain,
    /// and which plank of it, counted from its root. Never a tip.
    fn plank_under(&self, at: Vec2) -> Option<(usize, usize)> {
        self.chains.iter().enumerate().find_map(|(c, chain)| {
            (0..chain.corners.len().saturating_sub(1))
                .filter(|&k| chain.fixed(k))
                .find(|&k| {
                    let (l0, r0) = chain.corners[k];
                    let (l1, r1) = chain.corners[k + 1];
                    inside_quad(at, [l0, r0, r1, l1])
                })
                .map(|k| (c, k))
        })
    }

    /// Every plank's top, as a quad of its corners, tips included: for the
    /// minimap.
    pub(crate) fn quads(&self) -> impl Iterator<Item = [Vec2; 4]> + '_ {
        self.chains.iter().flat_map(|chain| {
            chain.corners.windows(2).map(|pair| {
                let ((l0, r0), (l1, r1)) = (pair[0], pair[1]);
                [l0, r0, r1, l1]
            })
        })
    }

    /// Begins a chain of `seat`'s at `root`, and says which of theirs it is.
    fn begin(&mut self, seat: u8, root: Rung) -> u16 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.chains.push(Chain::new(seat, id, vec![root]));
        self.active = Some(self.chains.len() - 1);
        id
    }

    /// Lets go of the chain you are building: its tip fixed where it is, if
    /// it has one worth keeping, and a chain of nothing but its root
    /// forgotten. The rung fixed, if any, to send.
    fn let_go(&mut self, metres: &mut f32) -> Option<(u16, u16, Rung)> {
        let c = self.active.take()?;
        let chain = &mut self.chains[c];
        let id = chain.id;
        let mut grown = Grown::default();
        fix_tip(chain, metres, &mut grown);
        if chain.rungs.len() < 2 {
            self.chains.remove(c);
        }
        self.dirty = true;
        grown.added.pop().map(|(index, rung)| (id, index, rung))
    }
}

/// Fixes `chain`'s tip where it is, if it is a plank's worth ([`MIN_LAND`])
/// and the chain has room, charging `metres`; a shorter tip is dropped.
fn fix_tip(chain: &mut Chain, metres: &mut f32, grown: &mut Grown) {
    chain.built_tip = None;
    chain.overlaid_tip = None;
    let Some(tip) = chain.tip.take() else {
        return;
    };
    let Some(last) = chain.rungs.last() else {
        return;
    };
    let length = last.p.distance(tip.p);
    if length >= MIN_LAND && chain.rungs.len() < MOST_RUNGS {
        *metres = (*metres - length).max(0.0);
        grown.added.push((chain.rungs.len() as u16, tip));
        chain.rungs.push(tip);
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

/// Lays the bridge under you as you build: before you move, so that it is
/// there when your feet are.
#[allow(clippy::too_many_arguments)]
pub(crate) fn lay(
    time: Res<Time>,
    bridging: Res<Bridging>,
    fighting: Res<Fighting>,
    online: Res<Online>,
    islands: Res<Islands>,
    me: Query<(&Transform, &Venue, &Intent, Has<Dead>), With<Player>>,
    mut bridges: ResMut<Bridges>,
    mut stock: ResMut<Resources>,
    mut requests: MessageWriter<NetAsk>,
) {
    let Ok((body, &venue, intent, dead)) = me.single() else {
        return;
    };
    let Some(seat) = fighting.seat else {
        return;
    };
    let mut sent: Vec<(u16, u16, Rung)> = Vec::new();
    let building = bridging.0 && fighting.playing && !dead && venue == Venue::Arena;
    if !building {
        // Hands empty, or dead: whatever was being built is left as it is.
        if bridges.active.is_some()
            && let Some(fixed) = bridges.let_go(&mut stock.bridge_metres)
        {
            sent.push(fixed);
        }
    } else if let Some(island) = islands.get(Venue::Arena) {
        let feet = body.translation.y;
        let here = body.translation.xz();
        let reach = feet + LAND_STEP_UP;
        let ground = |at: Vec2| island.floor(at, reach).is_some();
        let land = |at: Vec2| island.floor_base(at, reach).is_some();
        match bridges.active {
            Some(c) => {
                let under = bridges.plank_under(here);
                let grown = grow(&mut bridges.chains[c], here, under.map(|(other, _)| other == c), &mut stock.bridge_metres, &land, &ground);
                let id = bridges.chains[c].id;
                sent.extend(grown.added.iter().map(|&(index, rung)| (id, index, rung)));
                if grown.done {
                    bridges.active = None;
                    if bridges.chains[c].rungs.len() < 2 {
                        bridges.chains.remove(c);
                    }
                    bridges.dirty = true;
                } else if !grown.added.is_empty() {
                    bridges.dirty = true;
                }
            }
            None => {
                // Nothing being built: about to step over the void?
                if intent.walk.length() < 0.1 || stock.bridge_metres < SPACING || bridges.chains.len() >= MOST_CHAINS {
                    return;
                }
                let dir = intent.walk.xz().normalize();
                let speed = MOVE_SPEED * intent.sprint * intent.walk.length().min(1.0);
                let probe = here + dir * (speed * time.delta_secs() + LOOK);
                if ground(probe) || !ground(here) {
                    return;
                }
                let root = match bridges.plank_under(here) {
                    // Off the side of a plank, anyone's: a fork, flush
                    // against that side.
                    Some((c, k)) => {
                        let chain = &bridges.chains[c];
                        let (l0, r0) = chain.corners[k];
                        let (l1, r1) = chain.corners[k + 1];
                        let along = (chain.rungs[k + 1].p - chain.rungs[k].p).normalize_or(dir);
                        side_root(here, along, (l0, l1), (r0, r1))
                    }
                    // Off the ground: a rung on its very edge, headed
                    // straight out.
                    None => {
                        let e = bisect(here, probe, &ground);
                        Rung {
                            p: e,
                            d: outward(e, &ground).unwrap_or(dir),
                        }
                    }
                };
                let id = bridges.begin(seat, root);
                sent.push((id, 0, root));
                let c = bridges.chains.len() - 1;
                // The tip, reaching past you from the start.
                let d = clamp_turn(dir, root.d, MAX_TURN);
                let length = (AHEAD + (here - root.p).dot(d).max(0.0)).min(stock.bridge_metres.max(AHEAD));
                bridges.chains[c].tip = Some(Rung { p: root.p + d * length, d });
                bridges.dirty = true;
            }
        }
    }
    if online.is() && fighting.online {
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

/// Grows `chain`, which its builder standing at `here` is building, by this
/// frame: lands it if they are on `land`, or on another bridge (`ground`
/// is land and planks together), past where it began; forgets it if they
/// have gone back onto what it began on with nothing built; fixes a rung on
/// the line toward them each plank's length they are past the last, as far
/// as `metres` of bridge allow; and aims the tip at them. `on_own` says
/// whether they stand on one of its fixed planks, on another chain's, or
/// on no plank.
fn grow(
    chain: &mut Chain,
    here: Vec2,
    on_own: Option<bool>,
    metres: &mut f32,
    land: &dyn Fn(Vec2) -> bool,
    ground: &dyn Fn(Vec2) -> bool,
) -> Grown {
    let mut grown = Grown::default();
    let root = chain.rungs[0];
    let mut last = *chain.rungs.last().expect("a chain has a root");
    let push = |chain: &mut Chain, rung: Rung, grown: &mut Grown| {
        grown.added.push((chain.rungs.len() as u16, rung));
        chain.rungs.push(rung);
    };
    // Past where it began, with a plank fixed; or still at its root, on
    // the shore or the plank it began on, or gone back behind the root.
    let gone_out = chain.rungs.len() >= 2 && here.distance(root.p) > 2.0 * SPACING;
    let back_inland = (here - root.p).dot(root.d) < -FORGET_BEHIND;
    let on_land = land(here);
    if on_land || on_own == Some(false) {
        if gone_out {
            // The far shore, or another bridge: the chain ends on its
            // edge, headed at where you stepped onto it, with a plank's
            // worth of rung if the edge is that far from the last; nearer,
            // it ends on the last rung, whose corners reach the shore. Over
            // ground already, it is just left where it got to.
            let edge: &dyn Fn(Vec2) -> bool = if on_land { land } else { ground };
            if edge(last.p) {
                fix_tip(chain, metres, &mut grown);
            } else {
                let h = clamp_turn((here - last.p).normalize_or(last.d), last.d, MAX_TURN);
                if let Some(gap) = shore_ahead(last.p, h, edge) {
                    if gap >= MIN_LAND {
                        *metres = (*metres - gap).max(0.0);
                        push(chain, Rung { p: last.p + h * gap, d: h }, &mut grown);
                    }
                    grown.landed = true;
                }
                chain.tip = None;
            }
            grown.done = true;
            return grown;
        }
        if back_inland {
            chain.tip = None;
            grown.done = true;
            return grown;
        }
        // At the root still, coming up to the edge: the tip waits past it.
    } else if on_own == Some(true) {
        // Back over the chain's own planks: the tip stays as it is.
        return grown;
    }
    // Over the void, or coming up to the edge: a rung fixed on the line
    // toward you each plank's length you are past the last, and the tip
    // aimed at you past that; behind the last rung still, the tip reaches
    // straight on from it.
    loop {
        let to = here - last.p;
        let far = to.length();
        if far < SPACING || to.dot(last.d) <= 0.0 || *metres < SPACING || chain.rungs.len() >= MOST_RUNGS {
            break;
        }
        let d = clamp_turn(to / far, last.d, MAX_TURN);
        let rung = Rung { p: last.p + d * SPACING, d };
        *metres -= SPACING;
        push(chain, rung, &mut grown);
        last = rung;
    }
    let to = here - last.p;
    let (d, length) = if to.dot(last.d) > 0.1 {
        let far = to.length();
        (clamp_turn(to / far, last.d, MAX_TURN), far + AHEAD)
    } else {
        (last.d, AHEAD)
    };
    let length = length.min(metres.max(AHEAD));
    chain.tip = Some(Rung { p: last.p + d * length, d });
    grown
}

/// How far ahead of `from`, along `d`, `ground` begins, to within
/// [`EDGE`], if within [`ONTO_EDGE`]; `from` is over nothing.
fn shore_ahead(from: Vec2, d: Vec2, ground: &dyn Fn(Vec2) -> bool) -> Option<f32> {
    let steps = (ONTO_EDGE / SHORE_STEP).ceil() as usize;
    let on = (1..=steps).map(|k| from + d * (k as f32 * SHORE_STEP)).find(|&at| ground(at))?;
    let e = bisect(on, on - d * SHORE_STEP, ground);
    Some((e - from).dot(d).max(0.0))
}

/// The root of a fork off the side of a plank, for someone standing at
/// `here` on it: on whichever side edge they are nearer, `left` or `right`
/// (each from its start to its end, the plank running `along`), flush
/// against it and headed straight out from it, clamped along the edge so
/// that the new plank's width fits against it where it can.
fn side_root(here: Vec2, along: Vec2, left: (Vec2, Vec2), right: (Vec2, Vec2)) -> Rung {
    let middle = (left.0 + right.0) * 0.5;
    let on_right = perp(along).dot(here - middle) > 0.0;
    let (start, end) = if on_right { right } else { left };
    let edge = end - start;
    let length = edge.length();
    let t = if length > 2.0 * HALF_WIDTH {
        ((here - start).dot(edge) / length).clamp(HALF_WIDTH, length - HALF_WIDTH) / length
    } else {
        0.5
    };
    let out = perp(edge.normalize_or(along));
    Rung {
        p: start + edge * t,
        d: if on_right { out } else { -out },
    }
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

/// The left and right corners of each rung of `rungs`, the ribbon's two
/// edges: [`HALF_WIDTH`] out either side, square to the plank at the
/// root and at the end, and mitred at every rung between, across the
/// bisector of the two planks that meet there and as much wider as the
/// mitre needs (`1 / cos(turn / 2)`), so that the planks share their
/// edges and neither gap nor overlap however the chain curves; a mitre
/// that would reach past a short plank's end is drawn in instead. A corner
/// over the void at the root or at the end is pushed out, back along the
/// first plank or on along the last, onto the edge of the `land` there, so
/// that the bridge meets the shore exactly however it curves; a corner
/// over land stays, hidden under the land, whose top is higher. And
/// whether the chain ends on land: land within [`ONTO_EDGE`] ahead of its
/// last rung. A chain still `building`, its tip the last of the rungs, has
/// no end yet, however near the shore its tip reaches.
fn corners_of(rungs: &[Rung], building: bool, land: &dyn Fn(Vec2) -> bool) -> (Vec<(Vec2, Vec2)>, bool) {
    let n = rungs.len();
    let Some(last) = rungs.last() else {
        return (Vec::new(), false);
    };
    // Each plank's direction and length, from its rung to the next.
    let planks: Vec<(Vec2, f32)> = rungs
        .windows(2)
        .map(|pair| {
            let along = pair[1].p - pair[0].p;
            let length = along.length();
            (if length > 1e-4 { along / length } else { pair[0].d }, length)
        })
        .collect();
    let mut corners: Vec<(Vec2, Vec2)> = (0..n)
        .map(|k| {
            let across = match (k.checked_sub(1).map(|j| planks[j]), planks.get(k)) {
                (None, None) => perp(rungs[k].d) * HALF_WIDTH,
                (None, Some(&(out, _))) => perp(out) * HALF_WIDTH,
                (Some((into, _)), None) => perp(into) * HALF_WIDTH,
                (Some((into, before)), Some(&(out, after))) => {
                    let bisector = (perp(into) + perp(out)).normalize_or(perp(out));
                    let half_turn = into.angle_to(out).abs() * 0.5;
                    let mut length = HALF_WIDTH / half_turn.cos().max(0.1);
                    // Along the planks the mitre reaches `length * sin`;
                    // no further than most of the shorter plank.
                    let most = 0.45 * before.min(after);
                    if half_turn.sin() > 1e-4 && length * half_turn.sin() > most {
                        length = most / half_turn.sin();
                    }
                    bisector * length
                }
            };
            (rungs[k].p - across, rungs[k].p + across)
        })
        .collect();
    // Pushes a corner over the void out along `out` onto the land's edge,
    // if land is within reach that way.
    let onto_edge = |corner: Vec2, out: Vec2| -> Vec2 {
        if land(corner) {
            return corner;
        }
        match shore_ahead(corner, out, land) {
            Some(gap) => corner + out * gap,
            None => corner,
        }
    };
    if n > 1 {
        let back = -planks[0].0;
        let (l, r) = corners[0];
        corners[0] = (onto_edge(l, back), onto_edge(r, back));
    }
    let complete = !building && n > 1 && !land(last.p) && shore_ahead(last.p, planks[n - 2].0, land).is_some();
    if complete {
        let on = planks[n - 2].0;
        let (l, r) = corners[n - 1];
        corners[n - 1] = (onto_edge(l, on), onto_edge(r, on));
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
        if bridges.chain(plank.seat, plank.chain).is_none() {
            if bridges.chains.len() >= MOST_CHAINS {
                continue;
            }
            bridges.chains.push(Chain::new(plank.seat, plank.chain, Vec::new()));
        }
        let chain = bridges.chain(plank.seat, plank.chain).expect("just found or made");
        take(chain, plank.index, rung);
        bridges.dirty = true;
    }
}

/// Puts `rung`, the `index`th of `chain`, in its place: on the end if it is
/// next, or waiting until those before it have come.
fn take(chain: &mut Chain, index: u16, rung: Rung) {
    if usize::from(index) < chain.rungs.len() || chain.rungs.len() >= MOST_RUNGS {
        return;
    }
    chain.waiting.push((index, rung));
    chain.waiting.sort_by_key(|(index, _)| *index);
    while chain.waiting.first().is_some_and(|(index, _)| usize::from(*index) == chain.rungs.len()) {
        let (_, rung) = chain.waiting.remove(0);
        chain.rungs.push(rung);
    }
}

/// Builds every chain's mesh again as it changes, as its tip moves enough,
/// or as the file's colours change, and lays the planks' shape over the
/// island; and clears everything out of the arena, or as another game
/// begins.
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
            bridges.version += 1;
            info!("the bridges are cleared away");
        }
        bridges.active = None;
        bridges.next_id = 0;
        bridges.dirty = false;
        bridges.game = fighting.game;
        return;
    }
    // Drawn again whenever a tip has moved at all; laid over the island
    // again, and the minimap drawn again, once one has moved enough.
    let tip_moved = bridges.chains.iter().any(|chain| chain.tip != chain.built_tip);
    let overlay_due = bridges.chains.iter().any(|chain| match (chain.tip, chain.overlaid_tip) {
        (Some(tip), Some(laid)) => tip.p.distance(laid.p) > OVERLAY_MOVE || tip.d.angle_to(laid.d).abs() > OVERLAY_TURN,
        (None, None) => false,
        _ => true,
    });
    let restyled = changed.is_changed() && !changed.is_added();
    if !bridges.dirty && !tip_moved && !restyled {
        return;
    }
    let Some(island) = islands.get(Venue::Arena) else {
        return;
    };
    let relay = bridges.dirty || overlay_due || restyled;
    let land = |at: Vec2| island.floor_base(at, TOP + LAND_STEP_UP).is_some();
    let colours = ui().colours.arena;
    let mut parts = Vec::new();
    for chain in &mut bridges.chains {
        let rungs = chain.all();
        let redraw = relay || chain.tip != chain.built_tip || chain.mesh == Entity::PLACEHOLDER;
        if redraw {
            let (corners, complete) = corners_of(&rungs, chain.tip.is_some(), &land);
            chain.corners = corners;
            chain.complete = complete;
            chain.built_tip = chain.tip;
        }
        if relay {
            chain.overlaid_tip = chain.tip;
        }
        if rungs.len() < 2 {
            if redraw
                && chain.mesh != Entity::PLACEHOLDER
                && meshes.insert(&chain.handle, Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())).is_err()
            {
                warn!("a bridge has lost its mesh");
            }
            continue;
        }
        if !redraw {
            continue;
        }
        let (mesh, triangles) = planks(&chain.corners, chain.complete, chain.top(), colours.plank, colours.plank_side);
        if relay {
            parts.push(triangles);
        }
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
    if relay {
        bridges.version += 1;
        islands.lay_over(Venue::Arena, (!parts.is_empty()).then_some(parts));
    }
}

/// The planks between the rungs with these `corners`, their tops `height`
/// up, as a mesh coloured by vertex, `top` on top (every other plank a
/// shade lighter) and `side` everywhere else, and as the triangles bodies
/// stand on: each plank's top, underside, left side and right side, and a
/// cap across the tip if the chain is not `complete`.
fn planks(corners: &[(Vec2, Vec2)], complete: bool, height: f32, top: Color, side: Color) -> (Mesh, Vec<[Vec3; 3]>) {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colours: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut triangles = Vec::new();
    let tint = |colour: Color, lighter: f32| {
        let linear = colour.to_linear();
        [
            (linear.red * (1.0 + lighter)).min(1.0),
            (linear.green * (1.0 + lighter)).min(1.0),
            (linear.blue * (1.0 + lighter)).min(1.0),
            1.0,
        ]
    };
    let side_tint = tint(side, 0.0);
    let up = |at: Vec2| Vec3::new(at.x, height, at.y);
    let down = |at: Vec2| Vec3::new(at.x, height - THICKNESS, at.y);
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
        let top_tint = tint(top, if k % 2 == 1 { LIGHTER } else { 0.0 });
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

    /// Where the planks of `rungs` reach, as the quads of their corners,
    /// the last of them a tip still being laid if `building`.
    fn quads(rungs: &[Rung], building: bool, land: &dyn Fn(Vec2) -> bool) -> Vec<[Vec2; 4]> {
        let (corners, _) = corners_of(rungs, building, land);
        corners
            .windows(2)
            .map(|pair| {
                let ((l0, r0), (l1, r1)) = (pair[0], pair[1]);
                [l0, r0, r1, l1]
            })
            .collect()
    }

    #[test]
    fn a_turn_is_clamped_and_a_heading_goes_round_trip() {
        let east = Vec2::X;
        assert!(clamp_turn(Vec2::new(1.0, 0.2).normalize(), east, MAX_TURN).distance(Vec2::new(1.0, 0.2).normalize()) < 1e-6);
        let turned = clamp_turn(-east, east, MAX_TURN);
        assert!((east.angle_to(turned).abs() - MAX_TURN).abs() < 1e-5, "{turned}");
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
        let (corners, complete) = corners_of(&[root, Rung { p: e + out * SPACING, d: out }], false, &land);
        assert!(!complete);
        for corner in [corners[0].0, corners[0].1] {
            assert!((corner.length() - 10.0).abs() < 0.03, "{corner} is off the edge");
            assert!((corner.y.abs() - HALF_WIDTH).abs() < 0.02, "{corner} is not a plank's width out");
        }
        // The next rung's corners are the plain ones.
        assert!((corners[1].0 - (root.p + out * SPACING - perp(out) * HALF_WIDTH)).length() < 1e-5);
    }

    #[test]
    fn the_bridge_follows_your_feet_round_a_curve_and_lands_on_the_far_shore() {
        let mut corners = disc(Vec2::ZERO, 10.0);
        corners.extend(disc(Vec2::new(30.0, 8.0), 10.0));
        let island = Island::new(&corners);
        let land = is_land(&island);
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        let mut chain = Chain::new(0, 0, vec![root]);
        let mut metres = 100.0;
        // You walk from the edge in an arc, 0.1 m a frame, bending north
        // and then east toward the far island: every frame the tip reaches
        // past you, the fixed rungs trace where you walked, a plank's
        // length apart, and the plank under you is never missing.
        let mut here = Vec2::new(10.05, 0.0);
        let mut heading = 0.0f32;
        let mut frames = 0;
        let grown = loop {
            frames += 1;
            assert!(frames < 2000, "never landed");
            heading = if frames < 90 { heading + 0.012 } else { (heading - 0.012).max(-0.25) };
            here += Vec2::from_angle(heading) * 0.1;
            let grown = grow(&mut chain, here, None, &mut metres, &land, &land);
            if grown.done {
                break grown;
            }
            let rungs = chain.all();
            let under = quads(&rungs, true, &land).into_iter().any(|quad| inside_quad(here, quad));
            assert!(under, "frame {frames}: nothing under {here}, rungs {:?}", rungs.len());
            let tip = chain.tip.expect("a tip while building");
            assert!((tip.p - here).dot(tip.d) >= AHEAD - 0.5, "the tip stops short at {here}");
        };
        assert!(grown.landed && chain.complete || grown.landed, "did not land");
        assert!(chain.rungs.len() > 8, "{} rungs", chain.rungs.len());
        for pair in chain.rungs.windows(2) {
            let apart = pair[0].p.distance(pair[1].p);
            assert!(apart <= SPACING * 1.5 + 1e-4 || pair[1] == *chain.rungs.last().unwrap(), "{apart} m between rungs");
            assert!(pair[0].d.angle_to(pair[1].d).abs() <= MAX_TURN + 1e-4);
        }
        let turned = chain.rungs.iter().map(|rung| Vec2::X.angle_to(rung.d).abs()).fold(0.0f32, f32::max);
        assert!(turned > 0.5, "never bent round: {turned}");
        // The last rung lies on the far island's edge, headed into it, or,
        // the shore nearer than a plank's worth, short of it by that; every
        // rung is over the void; the metres laid are the bridge's length.
        let last = chain.rungs.last().unwrap();
        let off = last.p.distance(Vec2::new(30.0, 8.0)) - 10.0;
        assert!((-EDGE..MIN_LAND).contains(&off), "{} is {off} m off the shore", last.p);
        assert!(shore_ahead(last.p, last.d, &land).is_some_and(|gap| gap < MIN_LAND + 0.5), "the last rung does not head into the shore");
        for rung in &chain.rungs[1..] {
            assert!(!land(rung.p), "{} is over land", rung.p);
        }
        let length: f32 = chain.rungs.windows(2).map(|pair| pair[0].p.distance(pair[1].p)).sum();
        assert!((100.0 - metres - length).abs() < 1e-3, "laid {length} m for {} m of bridge", 100.0 - metres);
        // Built, the chain reaches the shore, the planks face out and stand
        // at the right height, and none is folded.
        let (built, complete) = corners_of(&chain.rungs, false, &land);
        assert!(complete);
        let end = built.len() - 1;
        for corner in [built[end].0, built[end].1] {
            assert!(land(corner) || (corner.distance(Vec2::new(30.0, 8.0)) - 10.0).abs() < 0.05, "{corner} is off the far shore");
        }
        for corner in [built[0].0, built[0].1] {
            assert!(land(corner) || (corner.length() - 10.0).abs() < 0.05, "{corner} is off the near shore");
        }
        assert_unfolded(&built, TOP, complete);
        // An unfinished chain is capped at its tip.
        let (open, _) = planks(&built[..3], false, TOP, Color::WHITE, Color::BLACK);
        assert_eq!(open.count_vertices(), 2 * 16 + 4);
    }

    /// Every plank of `corners` faces up on top, down underneath and level
    /// at its sides, and none is folded: each plank's two edges run on the
    /// way the plank goes.
    fn assert_unfolded(corners: &[(Vec2, Vec2)], height: f32, complete: bool) {
        let (mesh, triangles) = planks(corners, complete, height, Color::WHITE, Color::BLACK);
        let caps = if complete { 0 } else { 2 };
        assert_eq!(triangles.len(), (corners.len() - 1) * 8 + caps);
        assert_eq!(mesh.count_vertices(), (corners.len() - 1) * 16 + caps * 2);
        for [a, b, c] in &triangles {
            let normal = (*b - *a).cross(*c - *a);
            let middle = (*a + *b + *c) / 3.0;
            if (middle.y - height).abs() < 1e-4 {
                assert!(normal.y > 1e-3, "a top faces down at {middle}");
            } else if (middle.y - (height - THICKNESS)).abs() < 1e-4 {
                assert!(normal.y < -1e-3, "an underside faces up at {middle}");
            } else {
                assert!(normal.y.abs() < 1e-4);
            }
        }
        for pair in corners.windows(2) {
            let ((l0, r0), (l1, r1)) = (pair[0], pair[1]);
            let along = ((l1 + r1) - (l0 + r0)).normalize();
            assert!((l1 - l0).dot(along) > 0.02, "the left edge folds between {l0} and {l1}");
            assert!((r1 - r0).dot(along) > 0.02, "the right edge folds between {r0} and {r1}");
        }
    }

    #[test]
    fn a_chain_turning_as_hard_as_it_may_never_folds() {
        // Off the shore square, and then every rung turned the most it may,
        // one way, then the other, then back and forth.
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        let mut rungs = vec![root];
        let turns = [1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 0.0];
        for sign in turns {
            let last = *rungs.last().unwrap();
            let d = Vec2::from_angle(sign * MAX_TURN).rotate(last.d);
            rungs.push(Rung { p: last.p + d * SPACING, d });
        }
        // Built over the void, with a tip and without.
        let (corners, complete) = corners_of(&rungs, true, &land);
        assert!(!complete);
        assert_unfolded(&corners, TOP, complete);
        let (corners, _) = corners_of(&rungs, false, &land);
        assert_unfolded(&corners, top_of(3, 5), false);
        // The mitres make the ribbon wider at a turn, by 1 / cos(17.5
        // degrees), never more; the root's corners are on the shore.
        for (k, (l, r)) in corners.iter().enumerate().skip(1) {
            let width = l.distance(*r) / (2.0 * HALF_WIDTH);
            assert!(width > 0.999 && width < 1.0 / (MAX_TURN * 0.5).cos() + 1e-3, "rung {k} is {width} wide");
        }
        for corner in [corners[0].0, corners[0].1] {
            assert!(land(corner) || (corner.length() - 10.0).abs() < 0.05, "{corner} is off the shore");
        }
        // A short plank after a turn, as a landing or a tip left over, has
        // its mitre drawn in rather than reaching past its end.
        let last = *rungs.last().unwrap();
        let d = Vec2::from_angle(MAX_TURN).rotate(last.d);
        rungs.push(Rung { p: last.p + d * MIN_LAND, d });
        let (corners, _) = corners_of(&rungs, false, &land);
        assert_unfolded(&corners, TOP, false);
        // Two chains' tops are a millimetre or more apart, or the same.
        assert!(top_of(0, 0) <= TOP && top_of(0, 0) > TOP - 0.001 * TOPS as f32);
        assert!((top_of(0, 0) - top_of(0, 1)).abs() > 0.0009);
    }

    #[test]
    fn the_bridge_runs_out_with_the_metres_and_a_stub_is_forgotten() {
        let island = Island::new(&disc(Vec2::ZERO, 10.0));
        let land = is_land(&island);
        let root = Rung { p: Vec2::new(10.0, 0.0), d: Vec2::X };
        let mut chain = Chain::new(0, 0, vec![root]);
        // Three metres of bridge: two rungs fixed and a tip to the end of
        // them, and no more however far you walk.
        let mut metres = 3.0;
        for step in 1..60 {
            let here = Vec2::new(10.0 + step as f32 * 0.1, 0.0);
            grow(&mut chain, here, None, &mut metres, &land, &land);
        }
        assert_eq!(chain.rungs.len(), 3);
        assert!(metres < SPACING, "{metres} m left");
        let tip = chain.tip.unwrap();
        assert!((tip.p.x - (10.0 + 2.0 * SPACING + AHEAD)).abs() < 1e-3, "the tip reaches {}", tip.p.x);
        // Stepping back onto the island with nothing fixed yet forgets the
        // chain; letting go with a tip out fixes it.
        let mut bridges = Bridges::default();
        bridges.begin(0, root);
        bridges.chains[0].tip = Some(Rung { p: Vec2::new(11.5, 0.0), d: Vec2::X });
        let mut metres = 10.0;
        assert_eq!(bridges.let_go(&mut metres).map(|(_, index, _)| index), Some(1));
        assert!((metres - 8.5).abs() < 1e-4);
        assert_eq!(bridges.chains[0].rungs.len(), 2);
        assert_eq!(bridges.active, None);
        let mut stub = Bridges::default();
        stub.begin(0, root);
        assert_eq!(stub.let_go(&mut metres), None);
        assert!(stub.chains.is_empty());
        // Coming up to the edge from inside the island, from as far in as
        // a chain is begun, the chain just begun keeps its tip waiting over
        // the void; back inland with only a root, it is done, and nothing
        // kept.
        for x in [10.0 - LOOK - 0.2, 9.7] {
            let mut fresh = Chain::new(0, 0, vec![root]);
            let grown = grow(&mut fresh, Vec2::new(x, 0.0), None, &mut metres, &land, &land);
            assert!(!grown.done && fresh.tip.is_some_and(|tip| tip.p.x > 10.5), "from {x}: {grown:?} {:?}", fresh.tip);
        }
        let mut back = Chain::new(0, 0, vec![root]);
        let grown = grow(&mut back, Vec2::new(10.0 - FORGET_BEHIND - 0.1, 0.0), None, &mut metres, &land, &land);
        assert!(grown.done && !grown.landed && grown.added.is_empty());
    }

    #[test]
    fn a_fork_begins_flush_against_the_plank_s_side() {
        // A plank running east from (0, 0) to (SPACING, 0); standing on its
        // right side (+z in the ground's (x, z)), the fork's root lies on
        // that edge, headed straight out from it.
        let along = Vec2::X;
        let left = (Vec2::new(0.0, -HALF_WIDTH), Vec2::new(SPACING, -HALF_WIDTH));
        let right = (Vec2::new(0.0, HALF_WIDTH), Vec2::new(SPACING, HALF_WIDTH));
        let root = side_root(Vec2::new(0.4, 0.5), along, left, right);
        assert!((root.p.y - HALF_WIDTH).abs() < 1e-5 && (root.p.x - SPACING * 0.5).abs() < 1e-5, "{root:?}");
        assert!(root.d.distance(Vec2::Y) < 1e-5, "{root:?}");
        let other = side_root(Vec2::new(0.4, -0.5), along, left, right);
        assert!(other.d.distance(Vec2::NEG_Y) < 1e-5, "{other:?}");
    }

    #[test]
    fn rungs_arriving_out_of_turn_wait_for_those_before_them() {
        let mut chain = Chain::new(3, 7, Vec::new());
        let rung = |index: u16| Rung {
            p: Vec2::new(index as f32 * SPACING, 0.0),
            d: Vec2::X,
        };
        take(&mut chain, 2, rung(2));
        take(&mut chain, 0, rung(0));
        assert_eq!(chain.rungs.len(), 1);
        take(&mut chain, 1, rung(1));
        assert_eq!(chain.rungs.len(), 3);
        assert!(chain.waiting.is_empty());
        assert!((chain.rungs[2].p.x - 2.0 * SPACING).abs() < 1e-4);
    }
}
