//! What Blend In puts in the world: the ring past which nobody walks, a
//! dashed line on the ground at the ring road's outer edge ([`RING`],
//! unlit: "Play is kept inside a ring 70 m out ... a faint ring is drawn on
//! the ground there"), and a soft light rising off it as you come near it
//! ([`wall`]); a faint glow on the ground round each seeker from the hide
//! to the end of the seek (mine: so that the hider sees seekers coming, as
//! in any hide-and-seek, and everyone knows who they are); and once a
//! round is over, a tall soft column of light over the hider where they
//! stood ("the hider is lit up where they stand"), for as long as the round
//! is shown, with a pool of its light on the ground at their feet.
//!
//! The glow and the column are light, not things: unlit, added to what is
//! behind them (`AlphaMode::Add`, which both the phone and the desktop
//! draw), and soft at their edges, each faded by how far into it a point
//! is (its corners' colours), so that nothing of them is a hard edge. The
//! glow and the column hang from the bodies they are on, so that each goes
//! where its body goes and is drawn only where its body is; the column is
//! one sheet turned square to the camera ([`face_the_camera`]), so that
//! the hider stands in front of it whichever way they are seen from. The
//! ring and its light are on Blend In's venue alone. Not yet seen on a
//! phone.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_replicon::prelude::Remote;
use roundtown_net::blend_in::{self, BlendStage, RING};
use roundtown_net::{Member, Venue};

use super::Blend;
use crate::table::InTable;
use crate::ui::{UiChanged, ui};
use crate::{Player, WalkCycle};

/// The line round the ring: how long each dash is, and each gap, along it,
/// in metres; how wide (0.3, from 0.22, since a play-tester could not find
/// it by day); and how far over the ground it lies (as high as a kerb's
/// paint, so that the road under it never hides it).
const DASH: f32 = 1.6;
const GAP: f32 = 1.6;
const DASH_WIDTH: f32 = 0.3;
const DASH_UP: f32 = 0.04;
/// The glow round a seeker: how far out from their middle it reaches, in
/// metres, how far over their feet it lies, and how many pieces its round
/// edge is made of.
const GLOW: f32 = 1.5;
const GLOW_UP: f32 = 0.05;
const GLOW_SIDES: usize = 40;
/// The column over the hider: how high, and how wide, in metres.
const COLUMN_HIGH: f32 = 24.0;
const COLUMN_WIDE: f32 = 2.4;
/// How strong the column is up it: metres over the ground, and a share of
/// its colour. Nothing at its very foot, so that where it meets the ground
/// is no edge; faint up the hider's body and round the name over their
/// head (`tags::OVER_HEAD`, 1.85 m), so that they and their name stand out
/// against it; strongest a body's height over that; nothing at its top.
/// Mine, since a play-tester found its foot cut in hard straight edges, a
/// sheet of it seen edge-on a bright line down the hider's own body, and
/// the name over them white on its white by day: until then three sheets
/// crossed at the hider, strongest at the ground.
const COLUMN_UP: [(f32, f32); 6] = [(0.0, 0.0), (0.45, 0.32), (2.7, 0.3), (5.0, 0.85), (12.0, 0.45), (COLUMN_HIGH, 0.0)];
/// How strong it is across, from its middle out to its edge (a share of
/// half its width) and a share of its colour: nothing at its edges.
const COLUMN_ACROSS: [(f32, f32); 4] = [(0.0, 1.0), (0.3, 0.85), (0.65, 0.4), (1.0, 0.0)];
/// How much brighter than their colours the glow and the column are drawn,
/// as the punch's range and the fountain's water are, being unlit and
/// tonemapped: white at full strength comes out white.
const GAIN: f32 = 2.5;
/// How much brighter than its colour the ring's line and the light off it
/// are drawn: less than the glow, so that the game's violet comes out
/// violet, by day too, and not near white (mine).
const RING_GAIN: f32 = 1.5;
/// The light off the ring as you come near it ([`wall`]): how high it
/// rises, in metres; how far along the ring it reaches either way of the
/// point nearest you, fading to nothing; how far in from the ring you
/// begin to see it, and from how near it is whole; how far you go along
/// the ring before it is laid again round where you are; and how strong it
/// is up it, metres over the ground and a share. Mine.
const WALL_HIGH: f32 = 1.6;
const WALL_REACH: f32 = 18.0;
const WALL_FROM: f32 = 12.0;
const WALL_WHOLE: f32 = 2.0;
const WALL_RELAY: f32 = 1.5;
const WALL_UP: [(f32, f32); 4] = [(-0.1, 1.0), (0.4, 0.65), (1.0, 0.25), (WALL_HIGH, 0.0)];
/// How many pieces the light off the ring is laid in along it.
const WALL_PIECES: usize = 36;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, make)
        .add_systems(Update, (ring, wall, glow, light_up, restyle).after(super::arm))
        // The camera's place for this frame is set by then.
        .add_systems(Update, face_the_camera.after(crate::follow_camera));
}

/// What the ring, its light, the glow and the column are made of, made
/// once.
#[derive(Resource)]
struct Looks {
    ring: Handle<StandardMaterial>,
    wall: Handle<StandardMaterial>,
    glow: (Handle<Mesh>, Handle<StandardMaterial>),
    column: (Handle<Mesh>, Handle<StandardMaterial>),
}

fn make(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let colours = ui().colours.blend_in;
    commands.insert_resource(Looks {
        ring: materials.add(flat_look(colours.ring)),
        wall: materials.add(wall_look(colours.wall, 0.0)),
        glow: (meshes.add(glow_mesh()), materials.add(light_look(colours.glow))),
        column: (meshes.add(column_mesh()), materials.add(light_look(colours.column))),
    });
}

/// Takes the file's colours again as it changes on desktop. (The light off
/// the ring takes its own as it is next drawn, [`wall`].)
fn restyle(changed: Res<UiChanged>, looks: Option<Res<Looks>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let Some(looks) = looks else {
        return;
    };
    let colours = ui().colours.blend_in;
    for (handle, look) in [
        (&looks.ring, flat_look(colours.ring)),
        (&looks.glow.1, light_look(colours.glow)),
        (&looks.column.1, light_look(colours.column)),
    ] {
        if let Some(mut material) = materials.get_mut(handle) {
            *material = look;
        }
    }
}

/// What the ring's line is made of: unlit, `colour` as see-through as its
/// alpha says, a little brighter than it ([`RING_GAIN`]).
fn flat_look(colour: Color) -> StandardMaterial {
    let linear = colour.to_linear();
    StandardMaterial {
        base_color: Color::linear_rgba(linear.red * RING_GAIN, linear.green * RING_GAIN, linear.blue * RING_GAIN, linear.alpha),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        fog_enabled: false,
        depth_bias: 1.0,
        ..default()
    }
}

/// What the light off the ring is made of: as the line is, `colour` a
/// little brighter, as see-through as its alpha and `strength` say, and
/// each corner's colour (it is laid over what is behind it rather than
/// added, so that it shows by day on the pale road and sky).
fn wall_look(colour: Color, strength: f32) -> StandardMaterial {
    let mut look = flat_look(colour.with_alpha(colour.alpha() * strength));
    look.depth_bias = 0.0;
    look
}

/// What a light is made of: `colour`, brighter by [`GAIN`], added to what
/// is behind it as much as its alpha and each corner's colour say.
fn light_look(colour: Color) -> StandardMaterial {
    let linear = colour.to_linear();
    StandardMaterial {
        base_color: Color::linear_rgba(linear.red * GAIN, linear.green * GAIN, linear.blue * GAIN, linear.alpha),
        unlit: true,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        fog_enabled: false,
        ..default()
    }
}

/// The ring's line, once a game of Blend In is played here.
#[derive(Component)]
struct RingLine;

/// Lays the dashed line round the ring on the ground as a game of Blend In
/// begins here, on its venue alone: it goes with the table.
fn ring(
    blend: Res<Blend>,
    looks: Option<Res<Looks>>,
    lines: Query<(), With<RingLine>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    if blend.seat.is_none() || !lines.is_empty() {
        return;
    }
    let Some(looks) = looks else {
        return;
    };
    commands.spawn((
        RingLine,
        InTable,
        Name::new("Blend In's ring"),
        Venue::BlendIn,
        Mesh3d(meshes.add(ring_mesh())),
        MeshMaterial3d(looks.ring.clone()),
        Transform::default(),
        Visibility::Hidden,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

/// The dashes round the ring, each a flat strip [`DASH_WIDTH`] wide along
/// the ring, lying [`DASH_UP`] over the town's ground there (as baked for
/// the crowd, out to the ring), every [`DASH`] and [`GAP`] round.
fn ring_mesh() -> Mesh {
    let grid = blend_in::grid();
    let round = TAU * RING;
    let count = (round / (DASH + GAP)).round().max(8.0) as usize;
    let step = TAU / count as f32;
    let dash = step * DASH / (DASH + GAP);
    let (mut positions, mut indices) = (Vec::new(), Vec::new());
    for n in 0..count {
        let from = n as f32 * step;
        let base = positions.len() as u32;
        for angle in [from, from + dash] {
            let way = Vec2::from_angle(angle);
            // Just inside the ring, where the ground is baked.
            let ground = grid.height(way * (RING - 0.3)).unwrap_or(0.0) + DASH_UP;
            for radius in [RING - DASH_WIDTH * 0.5, RING + DASH_WIDTH * 0.5] {
                let at = way * radius;
                positions.push([at.x, ground, at.y]);
            }
        }
        indices.extend([base, base + 2, base + 1, base + 1, base + 2, base + 3]);
    }
    let count = positions.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_indices(Indices::U32(indices))
}

/// A disc of light lying flat, [`GLOW`] out, strongest round its middle and
/// nothing at its edge.
fn glow_mesh() -> Mesh {
    let mut positions = vec![[0.0, 0.0, 0.0]];
    let mut colours = vec![[1.0, 1.0, 1.0, 0.85]];
    let mut indices = Vec::new();
    // A ring halfway out as strong as the middle, then nothing at the edge.
    for (radius, alpha) in [(GLOW * 0.45, 0.7), (GLOW, 0.0)] {
        for side in 0..GLOW_SIDES {
            let at = Vec2::from_angle(side as f32 * TAU / GLOW_SIDES as f32) * radius;
            positions.push([at.x, 0.0, at.y]);
            colours.push([1.0, 1.0, 1.0, alpha]);
        }
    }
    let ring = |band: usize, side: usize| (1 + band * GLOW_SIDES + side % GLOW_SIDES) as u32;
    for side in 0..GLOW_SIDES {
        indices.extend([0, ring(0, side + 1), ring(0, side)]);
        indices.extend([ring(0, side), ring(0, side + 1), ring(1, side)]);
        indices.extend([ring(1, side), ring(0, side + 1), ring(1, side + 1)]);
    }
    let count = positions.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
}

/// A column of light standing on the ground, [`COLUMN_HIGH`] up and
/// [`COLUMN_WIDE`] across: one upright sheet facing along +Z, which is
/// turned square to the camera as it is drawn ([`face_the_camera`]), as
/// strong as [`COLUMN_UP`] and [`COLUMN_ACROSS`] say at each of its
/// corners, so that from any side it is a soft beam with no edge.
fn column_mesh() -> Mesh {
    let across: Vec<(f32, f32)> = COLUMN_ACROSS
        .iter()
        .rev()
        .map(|&(out, strength)| (-out, strength))
        .chain(COLUMN_ACROSS.iter().skip(1).copied())
        .collect();
    let (mut positions, mut colours, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    for &(up, strength) in &COLUMN_UP {
        for &(out, middle) in &across {
            positions.push([out * COLUMN_WIDE * 0.5, up, 0.0]);
            colours.push([1.0, 1.0, 1.0, strength * middle]);
        }
    }
    let corner = |row: usize, column: usize| (row * across.len() + column) as u32;
    for row in 0..COLUMN_UP.len() - 1 {
        for column in 0..across.len() - 1 {
            indices.extend([corner(row, column), corner(row, column + 1), corner(row + 1, column)]);
            indices.extend([corner(row, column + 1), corner(row + 1, column + 1), corner(row + 1, column)]);
        }
    }
    let count = positions.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
}

/// How strong the light off the ring is for someone standing `at`, from
/// nothing [`WALL_FROM`] in from the ring to whole [`WALL_WHOLE`] in from
/// it and nearer, eased in; in steps of a twentieth, so that what it is
/// made of is written again only as it comes to the next.
fn wall_strength(at: Vec2) -> f32 {
    let inside = RING - at.length();
    let share = ((WALL_FROM - inside) / (WALL_FROM - WALL_WHOLE)).clamp(0.0, 1.0);
    let eased = share * share * (3.0 - 2.0 * share);
    (eased * 20.0).round() / 20.0
}

/// The light off the ring round the point of it at `bearing` (radians round
/// from +X toward -Z, as `at.x = cos`, `at.z = -sin`): [`WALL_REACH`] along it
/// either way, standing on the town's ground there (as baked for the crowd,
/// just inside the ring), rising [`WALL_HIGH`], as strong as [`WALL_UP`]
/// says up it and fading to nothing toward either end.
fn wall_mesh(bearing: f32) -> Mesh {
    let grid = blend_in::grid();
    let reach = WALL_REACH / RING;
    let (mut positions, mut colours, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    for piece in 0..=WALL_PIECES {
        let share = piece as f32 / WALL_PIECES as f32 * 2.0 - 1.0;
        let angle = bearing + share * reach;
        let way = Vec2::new(angle.cos(), -angle.sin());
        let ground = grid.height(way * (RING - 0.3)).unwrap_or(0.0);
        let along = 1.0 - share * share;
        for &(up, strength) in &WALL_UP {
            let at = way * RING;
            positions.push([at.x, ground + up, at.y]);
            colours.push([1.0, 1.0, 1.0, strength * along * along]);
        }
    }
    let rows = WALL_UP.len();
    let corner = |piece: usize, row: usize| (piece * rows + row) as u32;
    for piece in 0..WALL_PIECES {
        for row in 0..rows - 1 {
            indices.extend([corner(piece, row), corner(piece + 1, row), corner(piece, row + 1)]);
            indices.extend([corner(piece + 1, row), corner(piece + 1, row + 1), corner(piece, row + 1)]);
        }
    }
    let count = positions.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(indices))
}

/// The light off the ring, while it is up: the bearing it was last laid
/// round, and how strong it was last drawn.
#[derive(Component)]
struct Wall {
    bearing: f32,
    strength: f32,
}

/// Raises a soft light off the ring as you come near it, in the game's
/// colour (`colours.blend_in.wall`), round the point of it nearest you, so
/// that where nobody walks past is plain before you walk into it (mine:
/// a play-tester found the line on the ground lost on the road by day,
/// and a seeker running out stopped at a wall nobody could see). Laid again
/// round where you are as you go along it, and drawn as strong as how near
/// you are says ([`wall_strength`]), not at all far from it; written only
/// as either changes. Up from the first time you come near it at a table,
/// and shown on Blend In's venue alone, as the line is (`island`).
fn wall(
    blend: Res<Blend>,
    looks: Option<Res<Looks>>,
    me: Query<(&Transform, &Venue), With<Player>>,
    mut walls: Query<(&mut Wall, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let Some(looks) = looks else {
        return;
    };
    let near = me
        .single()
        .ok()
        .filter(|(_, venue)| blend.seat.is_some() && **venue == Venue::BlendIn)
        .map(|(body, _)| body.translation.xz())
        .map(|at| (wall_strength(at), (-at.y).atan2(at.x)));
    let (strength, bearing) = near.unwrap_or((0.0, 0.0));
    let Some((mut wall, mesh)) = walls.iter_mut().next() else {
        if strength > 0.0 {
            if let Some(mut material) = materials.get_mut(&looks.wall) {
                *material = wall_look(ui().colours.blend_in.wall, strength);
            }
            commands.spawn((
                Wall { bearing, strength },
                InTable,
                Name::new("Blend In's ring, lit"),
                Venue::BlendIn,
                Mesh3d(meshes.add(wall_mesh(bearing))),
                MeshMaterial3d(looks.wall.clone()),
                Transform::default(),
                Visibility::Hidden,
                NotShadowCaster,
                NotShadowReceiver,
            ));
        }
        return;
    };
    if wall.strength != strength {
        wall.strength = strength;
        if let Some(mut material) = materials.get_mut(&looks.wall) {
            *material = wall_look(ui().colours.blend_in.wall, strength);
        }
    }
    let moved = Vec2::from_angle(bearing).angle_to(Vec2::from_angle(wall.bearing)).abs() * RING;
    if strength > 0.0 && moved > WALL_RELAY {
        wall.bearing = bearing;
        if let Some(mut laid) = meshes.get_mut(&mesh.0) {
            *laid = wall_mesh(bearing);
        }
    }
}

/// A body with a glow round it, and the glow.
#[derive(Component)]
struct Glowing(Entity);

/// A body with the column of light over it, and the column.
#[derive(Component)]
struct LitUp(Entity);

/// A glow, and a column: what [`Glowing`] and [`LitUp`] point at, while
/// they are there (a table begun afresh takes down everything of its own).
#[derive(Component)]
struct Glow;
#[derive(Component)]
struct Column;

/// Glows round every seeker from the hide to the end of the seek, you too
/// as you seek, and round nobody else: each hung from its body, put up and
/// taken away only as that changes.
#[allow(clippy::type_complexity)]
fn glow(
    blend: Res<Blend>,
    looks: Option<Res<Looks>>,
    me: Query<(Entity, Option<&Glowing>), With<Player>>,
    others: Query<(Entity, &Member, Option<&Glowing>), (With<Remote>, With<WalkCycle>)>,
    glows: Query<(), With<Glow>>,
    mut commands: Commands,
) {
    let Some(looks) = looks else {
        return;
    };
    if blend.seat.is_none() {
        return;
    }
    let seekers: Vec<u64> = blend
        .playing()
        .filter(|round| matches!(round.stage, BlendStage::Hide | BlendStage::Seek))
        .map(|round| round.swords.iter().copied().filter(|&id| id != 0).collect())
        .unwrap_or_default();
    let mut light = |body: Entity, lit: bool, glowing: Option<&Glowing>| match (lit, glowing.filter(|on| glows.contains(on.0))) {
        (true, None) => {
            let glow = commands
                .spawn((
                    Glow,
                    ChildOf(body),
                    InTable,
                    Name::new("Seeker's glow"),
                    Mesh3d(looks.glow.0.clone()),
                    MeshMaterial3d(looks.glow.1.clone()),
                    Transform::from_xyz(0.0, GLOW_UP, 0.0),
                    NotShadowCaster,
                    NotShadowReceiver,
                ))
                .id();
            commands.entity(body).insert(Glowing(glow));
        }
        (false, Some(glowing)) => {
            commands.entity(glowing.0).try_despawn();
            commands.entity(body).try_remove::<Glowing>();
        }
        (false, None) if glowing.is_some() => {
            commands.entity(body).try_remove::<Glowing>();
        }
        _ => {}
    };
    if let Ok((you, glowing)) = me.single() {
        light(you, blend.armed(), glowing);
    }
    for (body, member, glowing) in &others {
        light(body, seekers.contains(&member.id), glowing);
    }
}

/// Lights the hider up where they stand once the round is over, a column
/// of light over them for as long as the round is shown: over the body the
/// room is told was theirs (`BlendRound::unmasked`), or over you if it was
/// you. Taken away as the next round begins.
#[allow(clippy::type_complexity)]
fn light_up(
    blend: Res<Blend>,
    looks: Option<Res<Looks>>,
    me: Query<(Entity, Option<&LitUp>), With<Player>>,
    others: Query<(Entity, &Member, Option<&LitUp>), With<Remote>>,
    columns: Query<(), With<Column>>,
    mut commands: Commands,
) {
    let Some(looks) = looks else {
        return;
    };
    if blend.seat.is_none() {
        return;
    }
    let shown = blend
        .playing()
        .filter(|round| round.stage == BlendStage::Shown)
        .and_then(|round| round.unmasked);
    let hiding = blend.hiding();
    let mut light = |body: Entity, lit: bool, up: Option<&LitUp>| match (lit, up.filter(|on| columns.contains(on.0))) {
        (true, None) => {
            // The column, and a pool of its light on the ground at the
            // hider's feet, the glow's shape (mine: where it meets the
            // ground, soft).
            let column = commands
                .spawn((
                    Column,
                    ChildOf(body),
                    InTable,
                    Name::new("The hider lit up"),
                    Mesh3d(looks.column.0.clone()),
                    MeshMaterial3d(looks.column.1.clone()),
                    Transform::default(),
                    NotShadowCaster,
                    NotShadowReceiver,
                    children![(
                        Name::new("The hider's pool of light"),
                        Mesh3d(looks.glow.0.clone()),
                        MeshMaterial3d(looks.column.1.clone()),
                        Transform::from_xyz(0.0, GLOW_UP, 0.0),
                        NotShadowCaster,
                        NotShadowReceiver,
                    )],
                ))
                .id();
            commands.entity(body).insert(LitUp(column));
        }
        (false, Some(up)) => {
            commands.entity(up.0).try_despawn();
            commands.entity(body).try_remove::<LitUp>();
        }
        (false, None) if up.is_some() => {
            commands.entity(body).try_remove::<LitUp>();
        }
        _ => {}
    };
    if let Ok((you, up)) = me.single() {
        light(you, shown.is_some() && hiding, up);
    }
    for (body, member, up) in &others {
        light(body, shown == Some(member.id), up);
    }
}

/// Turns every column of light square to the camera about the upright
/// ([`column_mesh`]), so that it is the same soft beam from wherever it is
/// seen, and the hider in it stands in front of it (one sheet through
/// their middle, which their own body hides behind them). Written only as
/// the way it is to face changes by more than half a degree.
#[allow(clippy::type_complexity)]
fn face_the_camera(
    cameras: Query<&Transform, (With<crate::ThirdPersonCamera>, Without<Column>)>,
    bodies: Query<&Transform, Without<Column>>,
    mut columns: Query<(&mut Transform, &ChildOf), With<Column>>,
) {
    let Ok(eye) = cameras.single() else {
        return;
    };
    for (mut column, on) in &mut columns {
        let Ok(body) = bodies.get(on.parent()) else {
            continue;
        };
        let toward = (eye.translation - body.translation).xz();
        if toward.length_squared() < 1e-4 {
            continue;
        }
        // Its face (+Z) toward the camera, as the body it hangs from is
        // turned.
        let facing = body.rotation.inverse() * Quat::from_rotation_y(toward.x.atan2(toward.y));
        if column.rotation.angle_between(facing) > 0.01 {
            column.rotation = facing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corners(mesh: &Mesh) -> Vec<Vec3> {
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|positions| positions.as_float3())
            .expect("positions")
            .iter()
            .map(|&at| Vec3::from(at))
            .collect()
    }

    #[test]
    fn the_ring_s_line_lies_round_the_ring_on_its_ground_in_dashes() {
        let mesh = ring_mesh();
        let points = corners(&mesh);
        assert!(!points.is_empty());
        for at in &points {
            let out = at.xz().length();
            assert!((out - RING).abs() <= DASH_WIDTH * 0.5 + 1e-3, "{at}");
            assert!(at.y >= DASH_UP - 1e-3 && at.y < 1.0, "on the ground: {at}");
        }
        // About as many dashes as fit round it, a dash and a gap each.
        let dashes = points.len() / 4;
        assert!((dashes as f32 - TAU * RING / (DASH + GAP)).abs() <= 1.0, "{dashes}");
    }

    #[test]
    fn the_glow_and_the_column_fade_to_nothing_at_their_edges() {
        for (mesh, edge) in [(glow_mesh(), GLOW), (column_mesh(), COLUMN_WIDE * 0.5)] {
            let points = corners(&mesh);
            let Some(bevy::mesh::VertexAttributeValues::Float32x4(colours)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
                panic!("each corner has its strength");
            };
            assert_eq!(points.len(), colours.len());
            for (at, colour) in points.iter().zip(colours) {
                if at.xz().length() >= edge - 1e-3 {
                    assert_eq!(colour[3], 0.0, "{at}: nothing at the edge");
                }
            }
            assert!(colours.iter().any(|colour| colour[3] > 0.5), "something to see");
        }
        // The column stands on the ground, as high as it is.
        let top = corners(&column_mesh()).iter().map(|at| at.y).fold(0.0, f32::max);
        assert_eq!(top, COLUMN_HIGH);
    }

    /// How strong a mesh's corners are, each with where it is.
    fn strengths(mesh: &Mesh) -> Vec<(Vec3, f32)> {
        let Some(bevy::mesh::VertexAttributeValues::Float32x4(colours)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("each corner has its strength");
        };
        corners(mesh).into_iter().zip(colours.iter().map(|colour| colour[3])).collect()
    }

    /// How strong the column is `up` metres over the ground, along its
    /// middle, between the corners there.
    fn column_at(up: f32) -> f32 {
        let rows = COLUMN_UP;
        let at = rows.windows(2).find(|pair| up >= pair[0].0 && up <= pair[1].0).expect("on the column");
        let share = (up - at[0].0) / (at[1].0 - at[0].0);
        at[0].1 + (at[1].1 - at[0].1) * share
    }

    #[test]
    fn the_column_meets_the_ground_softly_and_is_faint_where_the_hider_and_their_name_are() {
        let column = strengths(&column_mesh());
        // One upright sheet, across x, which is turned to the camera.
        assert!(column.iter().all(|(at, _)| at.z == 0.0));
        for (at, strength) in &column {
            if at.y == 0.0 {
                assert_eq!(*strength, 0.0, "{at}: no edge where it meets the ground");
            }
        }
        // Up the body and round the name over its head, faint; strongest
        // over that.
        for up in [1.0, crate::tags::OVER_HEAD, crate::tags::OVER_HEAD + 0.5] {
            assert!(column_at(up) <= 0.35, "{up} m up: {}", column_at(up));
        }
        let strongest = COLUMN_UP.iter().map(|&(_, strength)| strength).fold(0.0, f32::max);
        assert!((column_at(5.0) - strongest).abs() < 1e-4 && strongest > 0.6);
    }

    #[test]
    fn the_light_off_the_ring_rises_from_it_near_you_and_fades_along_it() {
        // Far in, nothing; at the ring, whole; between, some, in steps.
        assert_eq!(wall_strength(Vec2::ZERO), 0.0);
        assert_eq!(wall_strength(Vec2::new(RING - WALL_FROM - 1.0, 0.0)), 0.0);
        assert_eq!(wall_strength(Vec2::new(0.0, RING - 0.45)), 1.0);
        let between = wall_strength(Vec2::new(RING - 6.0, 0.0));
        assert!(between > 0.0 && between < 1.0 && ((between * 20.0).round() - between * 20.0).abs() < 1e-4, "{between}");
        // Laid round the point nearest you, on the ring, rising off the
        // ground, and nothing at either end of it.
        let bearing = 2.0;
        let wall = strengths(&wall_mesh(bearing));
        let grid = blend_in::grid();
        let nearest = Vec2::new(bearing.cos(), -bearing.sin()) * RING;
        for (at, strength) in &wall {
            assert!((at.xz().length() - RING).abs() < 1e-3, "{at}");
            assert!(at.xz().distance(nearest) <= WALL_REACH + 0.5, "{at}: as far along it as it reaches");
            let ground = grid.height(at.xz() * ((RING - 0.3) / RING)).unwrap_or(0.0);
            assert!(at.y >= ground - 0.2 && at.y <= ground + WALL_HIGH + 1e-3, "{at}");
            if (at.xz().distance(nearest) - WALL_REACH).abs() < 0.2 || at.y >= ground + WALL_HIGH - 1e-3 {
                assert!(*strength < 1e-3, "{at}: nothing at its ends and its top");
            }
        }
        assert!(wall.iter().any(|(_, strength)| *strength > 0.9), "something to see");
    }
}
