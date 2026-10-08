//! The islands, and what the players stand on and bump into there.
//!
//! Each is a Blender model with the sea drawn round it: Round Town,
//! `assets/circlemap1.glb`, round a fountain, where everyone is; your home,
//! `assets/testhomemap.glb`, where only you go, for now with a room and a tall
//! wall in it to try the camera against; and House Builder's lobby,
//! `assets/lobbymap.glb`, where a game's players wait for it to fill. Those
//! three are loaded once, when the app opens. A game of House Builder also
//! has a plot for each of its builders, a copy of the plain home,
//! `assets/homemap.glb`, each, which come and go with the game
//! ([`spawn_island`]).
//!
//! Every island stands at the origin, in the same place. What keeps them
//! apart is [`Venue`]: everything that moves or collides carries one saying
//! which island it is on, bodies only meet what shares theirs, and only the
//! island you are on is drawn ([`show_where_i_am`]). Standing in one place
//! means the sky is the same over all of them, and arriving on another island
//! is a change of venue, not a journey.
//!
//! A model is the whole of its land. Once its scene has spawned, every
//! triangle in it is read back out into an [`Island`], so reshaping the land in
//! Blender and re-exporting it is all it takes to move a shore, raise the
//! bridge or put up a fence: nothing in the code knows where the land is.
//! Wherever the model has nothing underfoot is sea, at [`WATER_Y`]. Water
//! modelled into it, like the town's fountain, has the material
//! [`WATER_MATERIAL`]: it is drawn like the sea and stood in, not on. Someone
//! modelled standing on it, like the House Builder at their door, is a body,
//! not land: see [`read_island`].
//!
//! Every question a body asks is asked along a vertical line: which surfaces
//! of the model does it cross, at what height, and does each one face up or
//! down. The highest one within a step of your feet that faces up, and gently
//! enough to stand on, is the floor; anything filling the band your body takes
//! up is a wall. Which way a face points comes from its winding, so the model's
//! normals have to point out of the land (Blender: Mesh > Normals > Recalculate
//! Outside). A top turned inside out reads as an underside, and the player
//! drops straight through it.
//!
//! The camera asks along any line at all: how far a ball can travel down it
//! before it touches the model ([`Island::sweep`]). For that every face counts,
//! walls and all, from either side.
//!
//! [`WATER_Y`]: crate::WATER_Y

use bevy::gltf::GltfMaterialName;
use bevy::light::NotShadowCaster;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use crate::{LAND_TOP, Player, sky};

/// Triangles are filed under the squares of a grid this many metres across,
/// so that a query only tests the ones overhead.
const BUCKET: f32 = 2.0;
/// How finely a body is tested against the island, in metres. Anything
/// thinner can slip between two samples; the fences, at 0.5 m, cannot.
pub(crate) const LATTICE: f32 = 0.25;
/// Slack in the point-in-triangle test, as a share of the triangle, so that a
/// line down the seam between two triangles meets both rather than neither.
const EDGE: f32 = 1e-5;
/// Two crossings this close together and facing the same way are one surface,
/// met twice on the seam between two of its triangles.
const SEAM: f32 = 1e-3;
/// How far into an edge or a corner a ball has to go for it to count as
/// meeting it, in metres. One that only grazes it, passing at the length of
/// its radius, goes by.
const GRAZE: f32 = 1e-3;
/// The cosine of the steepest slope that is still ground underfoot: 50°. The
/// flank of a leaning trunk faces up too, but it is something to walk into,
/// not onto. The bridge, the steepest ground there is, climbs at 15°.
const STEEPEST_FLOOR: f32 = 0.643;
/// What a map's water is called in Blender. A part with this material is
/// drawn with the sea's material, whatever it looked like in Blender, and is
/// left out of what players stand on and bump into.
pub(crate) const WATER_MATERIAL: &str = "water";

/// Which island something is on: the town, your home, House Builder's lobby,
/// or one builder's plot in a game of it, by their seat. It is the same type
/// the server sends, so that where it says someone is, is where they are drawn.
pub(crate) use roundtown_net::Venue;

/// The islands there from the moment the app opens. A game's plots come and
/// go with it.
const LOADED: [Venue; 3] = [Venue::Town, Venue::Home, Venue::Lobby];

/// The model of `venue`, under `assets/`: the town, `testhomemap.glb` for your
/// home — the plain home with a room and a tall wall in it, to try the camera
/// against — and for every plot a copy of the plain home, without the room
/// yours has.
fn model(venue: Venue) -> &'static str {
    match venue {
        Venue::Town => "circlemap1.glb",
        Venue::Home => "testhomemap.glb",
        Venue::Plot(_) => "homemap.glb",
        Venue::Lobby => "lobbymap.glb",
    }
}

/// Where the player in `seat` stands on arriving on any island: on a ring
/// round the middle — round the fountain, in the town — facing in. The first
/// eight seats are each an eighth of the turn on from the one before, and a
/// town channel online, which holds sixteen, sits the rest between them
/// (`roundtown_net::arrival`, which the server works arrivals out with too).
pub(crate) fn arrival(seat: usize) -> Transform {
    let (spot, turn) = roundtown_net::arrival(seat);
    Transform::from_translation(spot.with_y(LAND_TOP)).with_rotation(turn)
}

/// The shape of each island, once its model has loaded, kept by model: every
/// copy of one — every plot — has the same shape, read once, until something is
/// built on it (`build_on`). Until then nobody on it moves: there is nothing
/// yet to stand on.
#[derive(Resource, Default)]
pub(crate) struct Islands {
    shapes: Vec<(&'static str, Island)>,
    /// Every triangle each model was read from, to build on.
    grounds: Vec<(&'static str, Vec<[Vec3; 3]>)>,
    /// Islands with something built on them: their model and all of that.
    built: Vec<(Venue, Island)>,
}

impl Islands {
    pub(crate) fn get(&self, venue: Venue) -> Option<&Island> {
        if let Some((_, island)) = self.built.iter().find(|(on, _)| *on == venue) {
            return Some(island);
        }
        self.of_model(model(venue))
    }

    /// `venue` as its model has it, leaving out whatever is built on it.
    pub(crate) fn ground(&self, venue: Venue) -> Option<&Island> {
        self.of_model(model(venue))
    }

    fn of_model(&self, model: &str) -> Option<&Island> {
        self.shapes
            .iter()
            .find(|(read, _)| *read == model)
            .map(|(_, island)| island)
    }

    fn set(&mut self, model: &'static str, corners: Vec<[Vec3; 3]>) {
        self.shapes.retain(|(read, _)| *read != model);
        self.shapes.push((model, Island::new(&corners)));
        self.grounds.retain(|(read, _)| *read != model);
        self.grounds.push((model, corners));
    }

    /// `venue` as its model has it, with the triangles of everything built on
    /// it, `on_top`, standing there too, one list to each piece, and `screens`
    /// that only the camera meets; or as its model has it again, with nothing.
    /// Until the model has loaded there is nothing to build on.
    pub(crate) fn build_on(
        &mut self,
        venue: Venue,
        on_top: &[Vec<[Vec3; 3]>],
        screens: &[[Vec3; 3]],
    ) {
        self.built.retain(|(on, _)| *on != venue);
        if on_top.is_empty() && screens.is_empty() {
            return;
        }
        let Some((_, ground)) = self.grounds.iter().find(|(read, _)| *read == model(venue))
        else {
            return;
        };
        let parts: Vec<&[[Vec3; 3]]> = std::iter::once(ground.as_slice())
            .chain(on_top.iter().map(Vec::as_slice))
            .collect();
        self.built.push((venue, Island::of_parts(&parts, screens)));
    }

    /// Every island something is built on.
    pub(crate) fn built_on(&self) -> Vec<Venue> {
        self.built.iter().map(|(on, _)| *on).collect()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Islands>()
        .add_systems(Startup, spawn_islands)
        .add_systems(Update, show_where_i_am);
}

fn spawn_islands(mut commands: Commands, assets: Res<AssetServer>, water: Res<sky::Water>) {
    for venue in LOADED {
        spawn_island(&mut commands, &assets, &water, venue);
    }
}

/// An island on `venue`: its model, and the sea round it, both hidden until
/// `show_where_i_am` has seen who is there. Returns the two, for whoever is to
/// take them away again.
pub(crate) fn spawn_island(
    commands: &mut Commands,
    assets: &AssetServer,
    water: &sky::Water,
    venue: Venue,
) -> [Entity; 2] {
    let land = commands
        .spawn((
            venue,
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(model(venue)))),
            Visibility::Hidden,
        ))
        .observe(read_island)
        .id();
    // Each island has its own sea, which goes with it. Its own entity rather
    // than a child of the model, so that reading the island back out of the
    // model does not take the sea for a floor.
    let sea = commands
        .spawn((venue, sky::sea(water), Visibility::Hidden))
        .id();
    [land, sea]
}

/// Draws the island you are on, and everything and everyone on it, and hides
/// the others. You yourself are always where you are, and drawn unless the
/// camera is inside you (`follow_camera`).
pub(crate) fn show_where_i_am(
    me: Query<&Venue, With<Player>>,
    mut things: Query<(&Venue, &mut Visibility), Without<Player>>,
) {
    let here = me.single().ok().copied();
    for (venue, mut visibility) in &mut things {
        visibility.set_if_neq(if Some(*venue) == here {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// Reads an island's shape back out of its meshes, once the scene has put
/// them where they stand. Everyone on it waits where they are until this has
/// run, so there is never a frame with nothing to stand on. A copy of a model
/// already read, like every plot after the first, is only given its water.
///
/// Water in the model — the town's fountain — is not part of the shape: it is
/// drawn like the sea, and whoever steps into it goes through to the bottom.
///
/// Nor is anyone modelled standing there, like the House Builder at their
/// door in the town: a body is not land. A skinned mesh is drawn where its
/// skeleton puts it rather than where its own transform does, so read as land
/// it would stand somewhere else entirely — the House Builder read that way
/// was an invisible wall 23 m out from the fountain, on the way to them.
fn read_island(
    ready: On<WorldInstanceReady>,
    venues: Query<&Venue>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, Option<&GltfMaterialName>, Has<SkinnedMesh>)>,
    placement: TransformHelper,
    meshes: Res<Assets<Mesh>>,
    water: Res<sky::Water>,
    mut islands: ResMut<Islands>,
    mut commands: Commands,
) {
    let Ok(&venue) = venues.get(ready.entity) else {
        return;
    };
    let model = model(venue);
    let known = islands.of_model(model).is_some();
    let mut corners = Vec::new();
    for entity in children.iter_descendants(ready.entity) {
        let Ok((part, material, skinned)) = parts.get(entity) else {
            continue;
        };
        if material.is_some_and(|name| name.0 == WATER_MATERIAL) {
            commands
                .entity(entity)
                .insert((MeshMaterial3d(water.material.clone()), NotShadowCaster));
            continue;
        }
        if known || skinned {
            continue;
        }
        let read = meshes
            .get(&part.0)
            .zip(placement.compute_global_transform(entity).ok())
            .is_some_and(|(mesh, place)| add_mesh(&mut corners, mesh, &place));
        if !read {
            warn!("{model}: could not read one of its meshes, so the player will pass through it");
        }
    }
    if known {
        return;
    }
    if corners.is_empty() {
        warn!("{model}: no triangles to stand on, so the whole island is sea");
    }
    islands.set(model, corners);
}

/// Adds the corners of every triangle of `mesh`, placed in the world. `false`
/// if the mesh holds nothing that can be read as triangles.
pub(crate) fn add_mesh(
    corners: &mut Vec<[Vec3; 3]>,
    mesh: &Mesh,
    place: &GlobalTransform,
) -> bool {
    if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
        return false;
    }
    let Ok(VertexAttributeValues::Float32x3(positions)) =
        mesh.try_attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return false;
    };
    let points: Vec<Vec3> = positions
        .iter()
        .map(|&corner| place.transform_point(Vec3::from(corner)))
        .collect();
    let order: Vec<usize> = match mesh.try_indices_option() {
        Ok(Some(indices)) => indices.iter().collect(),
        Ok(None) => (0..points.len()).collect(),
        Err(_) => return false,
    };
    corners.extend(order.chunks_exact(3).filter_map(|tri| {
        Some([
            *points.get(tri[0])?,
            *points.get(tri[1])?,
            *points.get(tri[2])?,
        ])
    }));
    true
}

/// The shape of one island, read out of its model.
pub(crate) struct Island {
    tris: Vec<Tri>,
    /// The lowest and highest point of each part it was made of, walls and
    /// all: the model, and each piece built on it.
    heights: Vec<(f32, f32)>,
    /// For each square of the grid, the triangles whose shadow touches it.
    buckets: Vec<Vec<u32>>,
    /// The corner of the grid with the lowest x and z.
    corner: Vec2,
    columns: usize,
    rows: usize,
    /// Every triangle again, walls included, for the camera.
    faces: Faces,
}

impl Island {
    fn new(corners: &[[Vec3; 3]]) -> Self {
        Self::of_parts(&[corners], &[])
    }

    /// The island of `parts`, each one told apart from the rest in asking
    /// whether a body is buried inside something ([`Island::blocked`]), with
    /// `screens` that only the camera meets: what shuts a doorway to it, while
    /// bodies walk through.
    fn of_parts(parts: &[&[[Vec3; 3]]], screens: &[[Vec3; 3]]) -> Self {
        let tris: Vec<Tri> = parts
            .iter()
            .zip(0..)
            .flat_map(|(corners, part)| {
                corners.iter().filter_map(move |&[a, b, c]| Tri::new(a, b, c, part))
            })
            .collect();
        let heights = parts
            .iter()
            .map(|corners| {
                corners
                    .iter()
                    .flatten()
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), corner| {
                        (low.min(corner.y), high.max(corner.y))
                    })
            })
            .collect();
        let (low, high) = tris
            .iter()
            .map(Tri::shadow)
            .reduce(|(low, high), (a, b)| (low.min(a), high.max(b)))
            .unwrap_or_default();
        let squares = ((high - low) / BUCKET).floor().as_uvec2() + 1;
        let (columns, rows) = (squares.x as usize, squares.y as usize);

        let mut buckets = vec![Vec::new(); columns * rows];
        for (index, tri) in tris.iter().enumerate() {
            let (a, b) = tri.shadow();
            let first = ((a - low) / BUCKET).floor().as_uvec2();
            let last = ((b - low) / BUCKET).floor().as_uvec2().min(squares - 1);
            for row in first.y as usize..=last.y as usize {
                for column in first.x as usize..=last.x as usize {
                    buckets[row * columns + column].push(index as u32);
                }
            }
        }
        Self {
            tris,
            heights,
            buckets,
            corner: low,
            columns,
            rows,
            faces: Faces::new(
                parts
                    .iter()
                    .copied()
                    .flatten()
                    .chain(screens)
                    .filter_map(|&[a, b, c]| Face::new(a, b, c))
                    .collect(),
            ),
        }
    }

    /// The highest ground at `at` that is no higher than `reach`, or `None`
    /// over open water.
    pub(crate) fn floor(&self, at: Vec2, reach: f32) -> Option<f32> {
        self.crossings(at)
            .filter(|&(y, tri)| tri.floor && y <= reach)
            .map(|(y, _)| y)
            .reduce(f32::max)
    }

    /// The lowest underside over `at` higher than `above`: a ceiling, the top
    /// of a doorway, a roof overhead. `None` under open sky.
    pub(crate) fn ceiling(&self, at: Vec2, above: f32) -> Option<f32> {
        self.crossings(at)
            .filter(|&(y, tri)| !tri.faces_up() && y > above)
            .map(|(y, _)| y)
            .reduce(f32::min)
    }

    /// The squares of a fine lattice under a circle at `centre` that something
    /// on the island fills somewhere between the heights `low` and `high`, as
    /// their lowest and highest corners. A body kept out of all of them is
    /// clear of the island.
    pub(crate) fn walls(
        &self,
        centre: Vec2,
        radius: f32,
        low: f32,
        high: f32,
    ) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        let first = ((centre - radius) / LATTICE).floor().as_ivec2();
        let last = ((centre + radius) / LATTICE).floor().as_ivec2();
        (first.y..=last.y)
            .flat_map(move |row| (first.x..=last.x).map(move |column| IVec2::new(column, row)))
            .map(|square| square.as_vec2() * LATTICE)
            .filter(move |&corner| self.blocked(corner + LATTICE * 0.5, low, high))
            .map(|corner| (corner, corner + LATTICE))
    }

    /// The squares of the lattice round a body at `centre` that something on
    /// the island fills where the body is, as their lowest and highest
    /// corners and how far out from its middle the body reaches there.
    /// `shape` is the body, up from its feet: a radius over each band of
    /// heights, in the world. A square filled in more than one band comes
    /// once, with the widest of them; one no band reaches is not looked at.
    /// A body kept that far out of each square is clear of the island.
    ///
    /// Each square is looked down through once, whatever the bands: beside
    /// the fountain's tower that is a couple of thousand triangles a look.
    pub(crate) fn walls_round(
        &self,
        centre: Vec2,
        shape: &[(f32, f32, f32)],
    ) -> Vec<(Vec2, Vec2, f32)> {
        let widest = shape.iter().map(|&(radius, ..)| radius).fold(0.0, f32::max);
        let first = ((centre - widest) / LATTICE).floor().as_ivec2();
        let last = ((centre + widest) / LATTICE).floor().as_ivec2();
        let mut walls = Vec::new();
        let mut column = Vec::new();
        for row in first.y..=last.y {
            for across in first.x..=last.x {
                let low = IVec2::new(across, row).as_vec2() * LATTICE;
                let high = low + LATTICE;
                // How near the body comes to the square: a band no wider than
                // that never meets it.
                let near = centre.distance(centre.clamp(low, high));
                if near >= widest {
                    continue;
                }
                column.clear();
                column.extend(
                    self.crossings(low + LATTICE * 0.5)
                        .map(|(y, tri)| (tri.part, y, tri.faces_up())),
                );
                tidy(&mut column);
                let reach = shape
                    .iter()
                    .filter(|&&(radius, bottom, top)| {
                        radius > near
                            && (column.iter().any(|&(_, y, _)| y > bottom && y < top)
                                || self.buried(&column, bottom, top))
                    })
                    .map(|&(radius, ..)| radius)
                    .reduce(f32::max);
                if let Some(radius) = reach {
                    walls.push((low, high, radius));
                }
            }
        }
        walls
    }

    /// Whether something at `at` fills any of the heights between `low` and
    /// `high`.
    pub(crate) fn blocked(&self, at: Vec2, low: f32, high: f32) -> bool {
        let mut crossings = Vec::new();
        for (y, tri) in self.crossings(at) {
            if y > low && y < high {
                return true;
            }
            crossings.push((tri.part, y, tri.faces_up()));
        }
        tidy(&mut crossings);
        self.buried(&crossings, low, high)
    }

    /// Whether the band between `low` and `high`, which none of the surfaces
    /// a vertical line crosses (`crossings`, tidied) passes through, is buried
    /// inside something all the same, like the middle of a trunk.
    fn buried(&self, crossings: &[(u32, f32, bool)], low: f32, high: f32) -> bool {
        // Count the surfaces on either side of it: from above a top is a way
        // in and a bottom a way out, and from below the other way round. For a
        // closed shape the two agree; a shape left open at one end, like a
        // trunk with no lid where it meets its branches, still shows up from
        // the other.
        //
        // Each part is counted on its own, and only if it reaches the band's
        // heights. A part that is not closed, like a blanket that is only a
        // sheet, would otherwise throw out the count for all that stands over
        // or under it: a bed upstairs filled the room under it, and a chair
        // downstairs the floor over it, with invisible walls.
        let middle = (low + high) * 0.5;
        crossings
            .chunk_by(|a, b| a.0 == b.0)
            .filter(|part| {
                let (bottom, top) = self.heights[part[0].0 as usize];
                bottom < high && top > low
            })
            .any(|part| {
                let (mut from_above, mut from_below) = (0, 0);
                for &(_, y, up) in part {
                    let way_in = if up { 1 } else { -1 };
                    if y > middle {
                        from_above += way_in;
                    } else {
                        from_below -= way_in;
                    }
                }
                from_above > 0 || from_below > 0
            })
    }

    /// Every surface a vertical line through `at` crosses, and the height it
    /// crosses it at.
    fn crossings(&self, at: Vec2) -> impl Iterator<Item = (f32, &Tri)> + '_ {
        self.overhead(at).iter().filter_map(move |&index| {
            let tri = &self.tris[index as usize];
            tri.crossing(at).map(|y| (y, tri))
        })
    }

    /// The triangles that might be over `at`.
    fn overhead(&self, at: Vec2) -> &[u32] {
        let square = ((at - self.corner) / BUCKET).floor();
        let outside = square.x < 0.0
            || square.y < 0.0
            || square.x >= self.columns as f32
            || square.y >= self.rows as f32;
        if outside {
            return &[];
        }
        &self.buckets[square.y as usize * self.columns + square.x as usize]
    }

    /// How far a ball of `radius` gets from `from` toward `to` before it
    /// touches the island, or `None` if it gets all the way. A face it starts
    /// off touching only stops it going further in.
    pub(crate) fn sweep(&self, from: Vec3, to: Vec3, radius: f32) -> Option<f32> {
        self.faces.sweep(from, to, radius)
    }

    /// Whether a ball of `radius` at `at` is clear of the island: touching
    /// none of it, and not buried inside any of it.
    pub(crate) fn room_for(&self, at: Vec3, radius: f32) -> bool {
        !self.faces.touch(at, radius) && !self.blocked(at.xz(), at.y, at.y)
    }
}

/// Puts what a vertical line crosses in order, part by part and up each part,
/// with a surface met twice, on the seam between two of its triangles, kept
/// once.
fn tidy(crossings: &mut Vec<(u32, f32, bool)>) {
    crossings.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    crossings.dedup_by(|next, kept| next.0 == kept.0 && next.2 == kept.2 && next.1 - kept.1 < SEAM);
}

/// How many faces are left in a box before it is split no further.
const FEW_FACES: usize = 4;

/// Every triangle of an island, walls included, as the camera sees them: in
/// boxes inside boxes, so that a ball is only tried against the few triangles
/// near its path. The fountain alone is thousands.
struct Faces {
    faces: Vec<Face>,
    /// The first holds every face, and each of the rest half of the faces in
    /// the box it sits in.
    boxes: Vec<Bounds>,
}

/// A box round some of the faces.
struct Bounds {
    low: Vec3,
    high: Vec3,
    /// With `count` 0, the second of the two boxes inside it; the first comes
    /// straight after this one. Otherwise the first of its `count` faces.
    first: u32,
    count: u32,
}

impl Faces {
    fn new(mut faces: Vec<Face>) -> Self {
        let mut boxes = Vec::with_capacity(2 * faces.len() / FEW_FACES + 1);
        if !faces.is_empty() {
            fill(&mut boxes, &mut faces, 0);
        }
        Self { faces, boxes }
    }

    fn sweep(&self, from: Vec3, to: Vec3, radius: f32) -> Option<f32> {
        let (way, length) = (to - from).normalize_and_length();
        // Along the line, one over how far it goes each way per metre: huge,
        // rather than infinite, where it does not go that way at all.
        let per = Vec3::select(way.abs().cmplt(Vec3::splat(1e-9)), Vec3::splat(1e9), way.recip());
        // How far along the line it enters a box grown by the ball, if it
        // does before `reach`.
        let enters = |low: Vec3, high: Vec3, reach: f32| {
            let a = (low - radius - from) * per;
            let b = (high + radius - from) * per;
            let (near, far) = (a.min(b).max_element(), a.max(b).min_element());
            (near <= far && far >= 0.0 && near <= reach).then_some(near)
        };
        let mut first: Option<f32> = None;
        let mut next = Vec::with_capacity(32);
        if !self.boxes.is_empty() {
            next.push(0);
        }
        while let Some(at) = next.pop() {
            let bounds: &Bounds = &self.boxes[at];
            let reach = first.unwrap_or(length);
            if enters(bounds.low, bounds.high, reach).is_none() {
                continue;
            }
            if bounds.count == 0 {
                // The nearer box last, to be looked in first: whatever it
                // meets there can save looking in the other at all.
                let (one, other) = (at + 1, bounds.first as usize);
                let far = |index: usize| {
                    let inner = &self.boxes[index];
                    enters(inner.low, inner.high, reach).unwrap_or(f32::INFINITY)
                };
                if far(one) < far(other) {
                    next.extend([other, one]);
                } else {
                    next.extend([one, other]);
                }
                continue;
            }
            let start = bounds.first as usize;
            for face in &self.faces[start..start + bounds.count as usize] {
                let reach = first.unwrap_or(length);
                if enters(face.low, face.high, reach).is_none() {
                    continue;
                }
                if let Some(met) = face.meets(from, way, reach, radius) {
                    first = Some(met);
                }
            }
        }
        first
    }

    /// Whether any face comes within `radius` of `at`.
    fn touch(&self, at: Vec3, radius: f32) -> bool {
        let mut next = Vec::with_capacity(32);
        if !self.boxes.is_empty() {
            next.push(0);
        }
        while let Some(index) = next.pop() {
            let bounds: &Bounds = &self.boxes[index];
            if at.distance_squared(at.clamp(bounds.low, bounds.high)) >= radius * radius {
                continue;
            }
            if bounds.count == 0 {
                next.extend([index + 1, bounds.first as usize]);
                continue;
            }
            let start = bounds.first as usize;
            let faces = &self.faces[start..start + bounds.count as usize];
            if faces.iter().any(|face| face.touches(at, radius)) {
                return true;
            }
        }
        false
    }
}

/// Puts a box round `faces`, which begin `offset` into all of them, and two
/// boxes inside it round each half of them, and so on down to a few faces a
/// box. The faces are put in the order the boxes hold them.
fn fill(boxes: &mut Vec<Bounds>, faces: &mut [Face], offset: usize) {
    let (low, high) = faces
        .iter()
        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(low, high), face| {
            (low.min(face.low), high.max(face.high))
        });
    let at = boxes.len();
    boxes.push(Bounds {
        low,
        high,
        first: offset as u32,
        count: faces.len() as u32,
    });
    if faces.len() <= FEW_FACES {
        return;
    }
    // Halved across the way their middles are most spread out.
    let (least, most) = faces
        .iter()
        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(least, most), face| {
            (least.min(face.middle()), most.max(face.middle()))
        });
    let across = (most - least).max_position();
    let half = faces.len() / 2;
    faces.select_nth_unstable_by(half, |a, b| a.middle()[across].total_cmp(&b.middle()[across]));
    let (first_half, second_half) = faces.split_at_mut(half);
    fill(boxes, first_half, offset);
    let second = boxes.len() as u32;
    fill(boxes, second_half, offset + half);
    boxes[at].first = second;
    boxes[at].count = 0;
}

/// One triangle of an island, walls included, as the camera sees it.
struct Face {
    a: Vec3,
    b: Vec3,
    c: Vec3,
    /// Which way it faces, a metre long. Either side stops a ball.
    normal: Vec3,
    /// The lowest and highest corners of the box it fits in.
    low: Vec3,
    high: Vec3,
}

impl Face {
    fn new(a: Vec3, b: Vec3, c: Vec3) -> Option<Self> {
        let normal = (b - a).cross(c - a).try_normalize()?;
        Some(Self {
            a,
            b,
            c,
            normal,
            low: a.min(b).min(c),
            high: a.max(b).max(c),
        })
    }

    /// How far a ball of `radius` gets from `from` along `way`, one metre
    /// long, before it touches this face, if it does within `reach`. Touching
    /// at the start counts only if the ball is heading further in.
    fn meets(&self, from: Vec3, way: Vec3, reach: f32, radius: f32) -> Option<f32> {
        // Measured from whichever side the ball starts on.
        let mut normal = self.normal;
        let mut height = normal.dot(from - self.a);
        if height < 0.0 {
            normal = -normal;
            height = -height;
        }
        let closing = -normal.dot(way);
        if height >= radius {
            // Nothing touches before the flat of the face does, and a ball
            // not heading for the flat never touches any of it.
            if closing <= 1e-6 {
                return None;
            }
            let met = (height - radius) / closing;
            if met > reach {
                return None;
            }
            if self.holds(from + way * met - normal * radius) {
                return Some(met);
            }
        } else if self.holds(from - normal * height) {
            return (closing > 0.0).then_some(0.0);
        }
        // It passes the flat beside the face, so if it touches, it touches an
        // edge or a corner first.
        [(self.a, self.b), (self.b, self.c), (self.c, self.a)]
            .into_iter()
            .filter_map(|(start, end)| meets_edge(from, way, reach, radius, start, end))
            .chain(
                [self.a, self.b, self.c]
                    .into_iter()
                    .filter_map(|corner| meets_corner(from, way, reach, radius, corner)),
            )
            .reduce(f32::min)
    }

    /// Whether some of it is within `radius` of `at`.
    fn touches(&self, at: Vec3, radius: f32) -> bool {
        let height = self.normal.dot(at - self.a);
        if height.abs() >= radius {
            return false;
        }
        if self.holds(at - self.normal * height) {
            return true;
        }
        // Beside the flat of it, the nearest of it is on an edge.
        [(self.a, self.b), (self.b, self.c), (self.c, self.a)]
            .into_iter()
            .any(|(start, end)| {
                let edge = end - start;
                let along = ((at - start).dot(edge) / edge.length_squared()).clamp(0.0, 1.0);
                at.distance_squared(start + edge * along) < radius * radius
            })
    }

    /// The middle of it.
    fn middle(&self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }

    /// Whether `at`, a point in the face's flat, is inside it.
    fn holds(&self, at: Vec3) -> bool {
        let inside =
            |start: Vec3, end: Vec3| (end - start).cross(at - start).dot(self.normal) >= 0.0;
        inside(self.a, self.b) && inside(self.b, self.c) && inside(self.c, self.a)
    }
}

/// How far a ball gets from `from` along `way` before it touches the edge from
/// `start` to `end`, if it does within `reach`: where its middle first comes
/// within `radius` of the line through them, between them.
fn meets_edge(
    from: Vec3,
    way: Vec3,
    reach: f32,
    radius: f32,
    start: Vec3,
    end: Vec3,
) -> Option<f32> {
    let edge = end - start;
    let long = edge.length_squared();
    if long <= 1e-12 {
        return None;
    }
    // Everything seen end on, down the edge.
    let offset = from - start;
    let apart = offset - edge * (offset.dot(edge) / long);
    let across = way - edge * (way.dot(edge) / long);
    let a = across.length_squared();
    let b = apart.dot(across);
    let c = apart.length_squared() - radius * radius;
    // Heading straight down the edge, a corner is met first.
    if a <= 1e-12 || b >= 0.0 {
        return None;
    }
    // Nearest the line it passes, it has to be inside the ball.
    if apart.length_squared() - b * b / a > (radius - GRAZE).powi(2) {
        return None;
    }
    let met = if c <= 0.0 {
        0.0
    } else {
        let d = b * b - a * c;
        if d < 0.0 {
            return None;
        }
        (-b - d.sqrt()) / a
    };
    let along = (offset + way * met).dot(edge) / long;
    (met <= reach && (0.0..=1.0).contains(&along)).then_some(met)
}

/// How far a ball gets from `from` along `way` before it touches `corner`, if
/// it does within `reach`.
fn meets_corner(from: Vec3, way: Vec3, reach: f32, radius: f32, corner: Vec3) -> Option<f32> {
    let offset = from - corner;
    let b = offset.dot(way);
    let c = offset.length_squared() - radius * radius;
    if b >= 0.0 || offset.length_squared() - b * b > (radius - GRAZE).powi(2) {
        return None;
    }
    let met = if c <= 0.0 {
        0.0
    } else {
        let d = b * b - c;
        if d < 0.0 {
            return None;
        }
        -b - d.sqrt()
    };
    (met <= reach).then_some(met)
}

/// One triangle of the island that is not a wall, laid out for a line dropped
/// straight down through it.
struct Tri {
    a: Vec3,
    ab: Vec3,
    ac: Vec3,
    /// One over twice the signed area of its shadow on the ground, which comes
    /// out negative exactly when the triangle faces up.
    inv: f32,
    /// Faces up, and gently enough to stand on.
    floor: bool,
    /// Which part of the island it belongs to (`Island::heights`).
    part: u32,
}

impl Tri {
    fn new(a: Vec3, b: Vec3, c: Vec3, part: u32) -> Option<Self> {
        let (ab, ac) = (b - a, c - a);
        let normal = ab.cross(ac);
        // A wall is crossed by no vertical line, or by one along its whole
        // height at once. Either way it says nothing that the faces above and
        // below it do not.
        if normal.y.abs() <= normal.length() * 1e-3 {
            return None;
        }
        Some(Self {
            a,
            ab,
            ac,
            inv: 1.0 / ab.xz().perp_dot(ac.xz()),
            floor: normal.y >= normal.length() * STEEPEST_FLOOR,
            part,
        })
    }

    /// It faces the sky rather than the sea bed.
    fn faces_up(&self) -> bool {
        self.inv < 0.0
    }

    /// The height at which a vertical line through `at` crosses it, if it does.
    fn crossing(&self, at: Vec2) -> Option<f32> {
        let p = at - self.a.xz();
        let u = p.perp_dot(self.ac.xz()) * self.inv;
        let v = self.ab.xz().perp_dot(p) * self.inv;
        (u >= -EDGE && v >= -EDGE && u + v <= 1.0 + EDGE)
            .then(|| self.a.y + u * self.ab.y + v * self.ac.y)
    }

    /// The corners of the box its shadow on the ground fits in.
    fn shadow(&self) -> (Vec2, Vec2) {
        let (a, b, c) = (self.a.xz(), (self.a + self.ab).xz(), (self.a + self.ac).xz());
        (a.min(b).min(c), a.max(b).max(c))
    }
}

#[cfg(test)]
impl Island {
    pub(crate) fn new_for_tests(corners: &[[Vec3; 3]]) -> Self {
        Self::new(corners)
    }

    pub(crate) fn screened_for_tests(corners: &[[Vec3; 3]], screens: &[[Vec3; 3]]) -> Self {
        Self::of_parts(&[corners], screens)
    }

    pub(crate) fn of_parts_for_tests(parts: &[&[[Vec3; 3]]]) -> Self {
        Self::of_parts(parts, &[])
    }
}

#[cfg(test)]
mod sweep_tests {
    use super::*;

    fn close(a: Option<f32>, b: f32) -> bool {
        a.is_some_and(|a| (a - b).abs() < 1e-4)
    }

    fn floor() -> Face {
        let v = Vec3::new;
        Face::new(v(-10.0, 0.0, -10.0), v(10.0, 0.0, -10.0), v(0.0, 0.0, 10.0)).unwrap()
    }

    /// The plane x = 1, from y 0 to 3 and z -2 to 2, as one triangle's worth.
    fn wall() -> Face {
        let v = Vec3::new;
        Face::new(v(1.0, 0.0, -2.0), v(1.0, 0.0, 2.0), v(1.0, 3.0, 2.0)).unwrap()
    }

    #[test]
    fn falls_onto_a_floor() {
        assert!(close(floor().meets(Vec3::new(0.0, 2.0, 0.0), Vec3::NEG_Y, 5.0, 0.3), 1.7));
        assert!(floor().meets(Vec3::new(0.0, 1.0, 0.0), Vec3::X, 5.0, 0.3).is_none());
        assert!(floor().meets(Vec3::new(0.0, 2.0, 0.0), Vec3::NEG_Y, 1.0, 0.3).is_none());
    }

    #[test]
    fn walls_stop_it_from_either_side() {
        assert!(close(wall().meets(Vec3::new(0.0, 1.0, 1.0), Vec3::X, 5.0, 0.3), 0.7));
        assert!(close(wall().meets(Vec3::new(2.0, 1.0, 1.0), Vec3::NEG_X, 5.0, 0.3), 0.7));
    }

    #[test]
    fn grazes_an_edge() {
        // Beside the wall's vertical edge at z = 2: its middle comes 0.3 from
        // the edge at x = 1 - sqrt(0.05).
        let met = wall().meets(Vec3::new(0.0, 1.0, 2.2), Vec3::X, 5.0, 0.3);
        assert!(close(met, 1.0 - 0.05f32.sqrt()), "{met:?}");
        // Wide of it altogether.
        assert!(wall().meets(Vec3::new(0.0, 1.0, 2.4), Vec3::X, 5.0, 0.3).is_none());
    }

    #[test]
    fn meets_a_corner() {
        // Straight at the top corner (1, 3, 2), from beyond it.
        let met = wall().meets(Vec3::new(1.0, 3.0, 4.0), Vec3::NEG_Z, 5.0, 0.3);
        assert!(close(met, 1.7), "{met:?}");
    }

    #[test]
    fn starting_against_it() {
        let from = Vec3::new(0.9, 1.0, 0.0);
        assert!(wall().meets(from, Vec3::NEG_X, 5.0, 0.3).is_none());
        assert!(close(wall().meets(from, Vec3::X, 5.0, 0.3), 0.0));
    }

    /// A closed box from (-1, 0, -1) to (1, 2, 1), wound facing out.
    fn cube() -> Vec<[Vec3; 3]> {
        let p = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
        let quads = [
            [p(-1., 0., -1.), p(1., 0., -1.), p(1., 0., 1.), p(-1., 0., 1.)],
            [p(-1., 2., -1.), p(-1., 2., 1.), p(1., 2., 1.), p(1., 2., -1.)],
            [p(-1., 0., -1.), p(-1., 0., 1.), p(-1., 2., 1.), p(-1., 2., -1.)],
            [p(1., 0., -1.), p(1., 2., -1.), p(1., 2., 1.), p(1., 0., 1.)],
            [p(-1., 0., -1.), p(-1., 2., -1.), p(1., 2., -1.), p(1., 0., -1.)],
            [p(-1., 0., 1.), p(1., 0., 1.), p(1., 2., 1.), p(-1., 2., 1.)],
        ];
        quads
            .iter()
            .flat_map(|q| [[q[0], q[1], q[2]], [q[0], q[2], q[3]]])
            .collect()
    }

    #[test]
    fn the_boxes_miss_nothing() {
        // A scatter of blocks, and lines through them every which way: the
        // boxes have to find the same first face as trying every face does.
        let mut seed = 7u32;
        let mut next = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / 16_777_216.0
        };
        let mut corners = Vec::new();
        for _ in 0..150 {
            let low = Vec3::new(next() * 40.0 - 20.0, next() * 6.0, next() * 40.0 - 20.0);
            let size = Vec3::new(next(), next(), next()) * 3.0 + 0.05;
            for tri in cube() {
                corners.push(tri.map(|corner| low + (corner + 1.0) * 0.5 * size));
            }
        }
        let island = Island::new(&corners);
        let every: Vec<Face> = corners.iter().filter_map(|&[a, b, c]| Face::new(a, b, c)).collect();
        for _ in 0..600 {
            let from = Vec3::new(next() * 50.0 - 25.0, next() * 8.0, next() * 50.0 - 25.0);
            let to = Vec3::new(next() * 50.0 - 25.0, next() * 8.0, next() * 50.0 - 25.0);
            let (way, length) = (to - from).normalize_and_length();
            let slow = every
                .iter()
                .filter_map(|face| face.meets(from, way, length, 0.3))
                .reduce(f32::min);
            let fast = island.sweep(from, to, 0.3);
            match (slow, fast) {
                (Some(slow), Some(fast)) => assert!((slow - fast).abs() < 1e-4, "{slow} {fast}"),
                (None, None) => {}
                _ => panic!("from {from} to {to}: every face says {slow:?}, the boxes {fast:?}"),
            }
        }
    }

    #[test]
    fn sweep_finds_the_box() {
        let island = Island::new(&cube());
        let hit = island.sweep(Vec3::new(5.0, 1.0, 0.0), Vec3::new(-5.0, 1.0, 0.0), 0.3);
        assert!(close(hit, 3.7), "{hit:?}");
        // Over the top of it, with room to spare.
        assert!(island.sweep(Vec3::new(5.0, 2.5, 0.0), Vec3::new(-5.0, 2.5, 0.0), 0.3).is_none());
        // Low enough to catch its top edge.
        let hit = island.sweep(Vec3::new(5.0, 2.2, 0.0), Vec3::new(-5.0, 2.2, 0.0), 0.3);
        assert!(close(hit, 4.0 - 0.05f32.sqrt()), "{hit:?}");
        // Passing the length of its radius over the edge, it only grazes it.
        assert!(island.sweep(Vec3::new(5.0, 2.3, 0.0), Vec3::new(-5.0, 2.3, 0.0), 0.3).is_none());
        // A line well off to one side meets nothing.
        assert!(island.sweep(Vec3::new(5.0, 1.0, 9.0), Vec3::new(-5.0, 1.0, 9.0), 0.3).is_none());
        // And the vertical questions still see the top and the walls.
        assert_eq!(island.floor(Vec2::ZERO, 3.0), Some(2.0));
        assert!(island.walls(Vec2::new(1.2, 0.0), 0.45, 0.75, 1.6).next().is_some());
    }

    #[test]
    fn walls_round_a_body_are_the_walls_of_each_of_its_bands() {
        // A scatter of blocks, and a body of three widths at three heights
        // stood about among them: each square comes with the widest band it
        // fills that reaches it, exactly the squares each band's own `walls`
        // has there.
        let mut seed = 11u32;
        let mut next = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / 16_777_216.0
        };
        let mut corners = Vec::new();
        for _ in 0..60 {
            let low = Vec3::new(next() * 20.0 - 10.0, next() * 2.0, next() * 20.0 - 10.0);
            let size = Vec3::new(next(), next(), next()) * 2.0 + 0.1;
            for tri in cube() {
                corners.push(tri.map(|corner| low + (corner + 1.0) * 0.5 * size));
            }
        }
        let island = Island::new(&corners);
        for _ in 0..300 {
            let centre = Vec2::new(next() * 20.0 - 10.0, next() * 20.0 - 10.0);
            let feet = next() * 2.0;
            let shape = [
                (0.21, feet + 0.495, feet + 0.65),
                (0.33, feet + 0.65, feet + 0.8),
                (0.45, feet + 0.8, feet + 1.6),
            ];
            let round = island.walls_round(centre, &shape);
            for &(radius, low, high) in &shape {
                let reaching = |square: (Vec2, Vec2)| {
                    centre.distance(centre.clamp(square.0, square.1)) < radius
                };
                let each: Vec<(Vec2, Vec2)> =
                    island.walls(centre, radius, low, high).filter(|&s| reaching(s)).collect();
                for square in &each {
                    assert!(
                        round.iter().any(|&(a, b, r)| (a, b) == *square && r >= radius),
                        "{square:?} of the band {low}..{high} is not in {round:?}"
                    );
                }
            }
            for &(a, b, radius) in &round {
                let (_, low, high) = *shape.iter().find(|band| band.0 == radius).unwrap();
                assert!(island.walls(centre, radius, low, high).any(|s| s == (a, b)), "{a} {b}");
            }
        }
    }

    #[test]
    fn room_for_a_ball() {
        let island = Island::new(&cube());
        // Clear of it, beside it and over it.
        assert!(island.room_for(Vec3::new(1.4, 1.0, 0.0), 0.3));
        assert!(island.room_for(Vec3::new(0.0, 2.4, 0.0), 0.3));
        // Touching a side, an edge and a corner from outside.
        assert!(!island.room_for(Vec3::new(1.2, 1.0, 0.0), 0.3));
        assert!(!island.room_for(Vec3::new(1.2, 2.2, 0.0), 0.3));
        assert!(!island.room_for(Vec3::new(1.15, 2.15, 1.15), 0.3));
        // Off the corner by more than the ball.
        assert!(island.room_for(Vec3::new(1.2, 2.2, 1.2), 0.3));
        // Deep inside, nowhere near a face.
        assert!(!island.room_for(Vec3::new(0.0, 1.0, 0.0), 0.3));
    }
}
