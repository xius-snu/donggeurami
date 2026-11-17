//! What Blend In puts in the world: the ring past which nobody walks, a
//! faint dashed line on the ground at the ring road's outer edge ([`RING`],
//! unlit, the plate's grey: "Play is kept inside a ring 70 m out ... a faint
//! ring is drawn on the ground there"); a faint glow on the ground round
//! each seeker from the hide to the end of the seek (mine: so that the
//! hider sees seekers coming, as in any hide-and-seek, and everyone knows
//! who they are); and once a round is over, a tall soft column of light
//! over the hider where they stood ("the hider is lit up where they
//! stand"), for as long as the round is shown.
//!
//! The glow and the column are light, not things: unlit, added to what is
//! behind them (`AlphaMode::Add`, which both the phone and the desktop
//! draw), and soft at their edges, each faded by how far into it a point
//! is (its corners' colours), so that nothing of them is a hard edge. The
//! glow and the column hang from the bodies they are on, so that each goes
//! where its body goes and is drawn only where its body is. The ring is on
//! Blend In's venue alone. Not yet seen on a phone.

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
/// in metres; how wide; and how far over the ground it lies (as high as a
/// kerb's paint, so that the road under it never hides it).
const DASH: f32 = 1.6;
const GAP: f32 = 1.6;
const DASH_WIDTH: f32 = 0.22;
const DASH_UP: f32 = 0.04;
/// The glow round a seeker: how far out from their middle it reaches, in
/// metres, how far over their feet it lies, and how many pieces its round
/// edge is made of.
const GLOW: f32 = 1.5;
const GLOW_UP: f32 = 0.05;
const GLOW_SIDES: usize = 40;
/// The column over the hider: how high, and how wide, in metres; and how
/// many sheets of it cross at its middle, each turned on from the last.
const COLUMN_HIGH: f32 = 24.0;
const COLUMN_WIDE: f32 = 2.4;
const COLUMN_SHEETS: usize = 3;
/// How much brighter than their colours the glow and the column are drawn,
/// as the punch's range and the fountain's water are, being unlit and
/// tonemapped: white at full strength comes out white.
const GAIN: f32 = 2.5;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, make)
        .add_systems(Update, (ring, glow, light_up, restyle).after(super::arm));
}

/// What the ring, the glow and the column are made of, made once.
#[derive(Resource)]
struct Looks {
    ring: Handle<StandardMaterial>,
    glow: (Handle<Mesh>, Handle<StandardMaterial>),
    column: (Handle<Mesh>, Handle<StandardMaterial>),
}

fn make(mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    let colours = ui().colours.blend_in;
    commands.insert_resource(Looks {
        ring: materials.add(flat_look(colours.ring)),
        glow: (meshes.add(glow_mesh()), materials.add(light_look(colours.glow))),
        column: (meshes.add(column_mesh()), materials.add(light_look(colours.column))),
    });
}

/// Takes the file's colours again as it changes on desktop.
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
/// alpha says, a little brighter than it, as the punch's range is.
fn flat_look(colour: Color) -> StandardMaterial {
    let mut look = crate::range::look(colour);
    look.depth_bias = 1.0;
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
/// [`COLUMN_WIDE`] across: sheets crossing at its middle, each strongest
/// along its middle and at its foot, and nothing at its sides nor at its
/// top, so that from any side it is a soft beam with no edge.
fn column_mesh() -> Mesh {
    // Across a sheet, from one side to the other; up it, from the foot.
    const ACROSS: [(f32, f32); 3] = [(-1.0, 0.0), (0.0, 1.0), (1.0, 0.0)];
    const UP: [(f32, f32); 4] = [(0.0, 0.8), (0.12, 0.7), (0.5, 0.3), (1.0, 0.0)];
    let (mut positions, mut colours, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    for sheet in 0..COLUMN_SHEETS {
        let way = Vec2::from_angle(sheet as f32 * TAU * 0.5 / COLUMN_SHEETS as f32);
        let base = positions.len() as u32;
        for &(up, strength) in &UP {
            for &(across, middle) in &ACROSS {
                let at = way * across * COLUMN_WIDE * 0.5;
                positions.push([at.x, up * COLUMN_HIGH, at.y]);
                colours.push([1.0, 1.0, 1.0, strength * middle]);
            }
        }
        let corner = |row: u32, column: u32| base + row * ACROSS.len() as u32 + column;
        for row in 0..UP.len() as u32 - 1 {
            for column in 0..ACROSS.len() as u32 - 1 {
                indices.extend([corner(row, column), corner(row + 1, column), corner(row, column + 1)]);
                indices.extend([corner(row, column + 1), corner(row + 1, column), corner(row + 1, column + 1)]);
            }
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
}
