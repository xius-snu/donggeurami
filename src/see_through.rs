//! Whatever stands between the camera and you is drawn see-through, and a tap
//! goes through it to what is behind.
//!
//! The camera stands where you turned and zoomed it, walls or no walls
//! (`follow_camera`): in a house, zoomed out, it is outside the house looking
//! in. Every frame the straight lines from it to points all over you, from
//! your legs to the top of your head and out to your sides ([`RINGS`]), are
//! tried against everything drawn on your island, and whatever any of them
//! goes through fades to a quarter of itself, as Roblox's Invisicam fades it,
//! and back once none does. So whatever hides any of you, however little, is
//! out of the way (Hajun, 2026-10-13); until then only what hid the middle of
//! your body, your chest or your head was (2026-10-09). Zoomed all the way
//! in, through your own eyes, there is nothing between.
//!
//! A piece of furniture, a door, a window or a town object fades whole. A
//! house, and the island itself, fade a part at a time, each part as it was
//! modelled in Blender: a wall, the roof, a floor, a tree's leaves. Whatever
//! is in a part fades with it, so that no door stands solid in a wall you see
//! through ([`inside_of`]): the doors, windows and sign of a house in the
//! town, and the doors, windows and hangings in a wall of a house built in
//! House Builder. A door or a window in one of those walls, glass and all, is
//! in the way as its wall is ([`in_a_wall`]): seen through one, the wall goes
//! see-through with everything in it. Standing in a house, any of its walls
//! in the way of you takes the house's ceiling with it ([`over_your_head`]).
//! Bodies never fade, nor water and any other glass for being in the way, as
//! they are see-through already, nor a piece being put down, nor whatever
//! the edit menu is open on, nor the ground ([`ground`]).
//!
//! Something a tap can reach (`editor::Editable`) is out of its reach for as
//! long as any of it is see-through ([`InTheWay`]): a tap goes through it, so
//! that from outside a house you can reach in and tap what is in there with
//! you. Only what is not in the way of you can be tapped.
//!
//! A part is drawn see-through with a copy of its own material, drawn with
//! alpha to coverage, so that it is one even pane however its parts overlap
//! ([`see_through`]), on both platforms alike (`RENDERING.md`).

use std::collections::{HashMap, HashSet};

use bevy::camera::primitives::Aabb;
use bevy::gltf::GltfMeshName;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

use crate::build::{Ghost, Mount, OnWall, Piece, Wall, YouAreIn};
use crate::editor::{EditMenu, EditSystems, Editable};
use crate::island::{self, Faces};
use crate::map::MapObject;
use crate::{Player, ThirdPersonCamera};

/// How much of something in the way of you is drawn: a quarter of it, as
/// much as Roblox's Invisicam leaves of what it fades. None of it was tried
/// on 2026-10-13, and Hajun found that too much; what they minded in a
/// quarter was the edges of a part showing through it, which drawing it
/// with alpha to coverage takes away ([`see_through`]).
const SEEN: f32 = 0.25;
/// How long something takes to fade, and to come back, in seconds.
const FADE_SECS: f32 = 0.15;
/// What of you the camera has to see: rings round you at these heights over
/// your feet, each as wide across you and as deep front to back as you are
/// there, the middle of each ring and points round it ([`ROUND`]). Whatever
/// hides any of them is in the way: until 2026-10-13 only the middle of you
/// counted, at 1.15, 1.4 and 1.65 m, and a wall that hid your side or your
/// legs stayed as it was.
///
/// Each is a little inside the model (`limbperson.glb`), so that a wall you
/// stand against is not in the way of a camera on your side of it: legs 0.42
/// m across and 0.16 deep, then a body 0.78 across and 0.28 deep from 0.65 m
/// up, and a head 0.41 across from 1.3 m to 1.71. The legs' rings are no
/// wider than the legs reach (`LEG_RADIUS`), and the lowest is higher than
/// any edge the legs stand over without stepping up onto it (`EDGE_RISE`),
/// so that the step you are on is never in the way; the shoes, under it, are
/// not tried. Something thinner than the gaps between the points, about a
/// quarter of a metre, like a bar across a window, can cross you unseen.
const RINGS: [(f32, f32, f32); 8] = [
    (0.2, 0.17, 0.06),
    (0.42, 0.17, 0.06),
    (0.62, 0.17, 0.06),
    (0.85, 0.36, 0.11),
    (1.05, 0.36, 0.11),
    (1.25, 0.36, 0.11),
    (1.42, 0.15, 0.15),
    (1.6, 0.15, 0.15),
];
/// How many points round each ring of [`RINGS`], besides its middle.
const ROUND: usize = 8;
/// The top of your head, as high as the camera has to see of you.
const CROWN: f32 = 1.68;
/// How far out of the box round a part of a house or of the island something
/// else of it can reach and still be in it, and go with it, in metres.
const IN_IT: f32 = 0.05;

/// Out of a tap's reach, while some of it is see-through for standing between
/// the camera and you: a tap goes through it to whatever is behind (`editor`).
#[derive(Component)]
pub(crate) struct InTheWay;

/// A part drawn see-through, while it is in the way of you or fading back now
/// that it is not.
#[derive(Component)]
struct SeeThrough {
    /// Its own material, which it is drawn with again once it is solid.
    solid: Handle<StandardMaterial>,
    /// The see-through copy of it that it is drawn with meanwhile.
    look: Handle<StandardMaterial>,
    /// How solid its own material is: 1, or less for glass and water.
    alpha: f32,
    /// How much of that it is drawn with, from 1, as it is, down to [`SEEN`].
    shown: f32,
    /// What it is part of that a tap could reach, if anything.
    owner: Option<Entity>,
}

/// The parts that hide you from the camera this frame, and those that fade
/// along with them, each with what it is part of that a tap could reach.
#[derive(Resource, Default)]
struct Hiding(HashMap<Entity, Option<Entity>>);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Hiding>().add_systems(
        Update,
        (find_what_hides_you, fade_what_hides_you)
            .chain()
            .after(crate::follow_camera)
            .before(EditSystems),
    );
}

/// Every part of the world that is drawn and might be in the way: everything
/// with a mesh and a material of its own, but a body.
type Parts<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Mesh3d,
        &'static MeshMaterial3d<StandardMaterial>,
        &'static Aabb,
        &'static GlobalTransform,
        &'static InheritedVisibility,
        Option<&'static SeeThrough>,
    ),
    Without<SkinnedMesh>,
>;

/// What a part is part of, looked for up the hierarchy from it.
type Lineage<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static ChildOf>,
        Has<GltfMeshName>,
        Option<&'static Piece>,
        Has<MapObject>,
        Has<Editable>,
        Has<Ghost>,
    ),
>;

/// Finds what stands between the camera and you: whatever the lines from the
/// camera to you go through, and what fades along with it.
#[allow(clippy::too_many_arguments)]
fn find_what_hides_you(
    me: Query<(&Transform, &Visibility), With<Player>>,
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Player>)>,
    parts: Parts,
    lineage: Lineage,
    children: Query<&Children>,
    on_walls: Query<(Entity, &OnWall)>,
    walls: Query<&ChildOf, With<Wall>>,
    house_you_are_in: Query<Entity, With<YouAreIn>>,
    menu: Res<EditMenu>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    mut changed: MessageReader<AssetEvent<Mesh>>,
    mut shapes: Local<HashMap<AssetId<Mesh>, Faces>>,
    mut inside: Local<HashMap<Entity, Vec<Entity>>>,
    mut hiding: ResMut<Hiding>,
) {
    // A mesh changed since it was read, like a wall with a new hole in it, is
    // read again the next time it is asked about.
    for event in changed.read() {
        if let AssetEvent::Modified { id } | AssetEvent::Removed { id } = *event {
            shapes.remove(&id);
        }
    }
    hiding.0.clear();
    // Through your eyes, or so near them that you are not drawn, there is
    // nothing between the camera and you.
    let (Ok((you, drawn)), Ok(eye)) = (me.single(), cameras.single()) else {
        return;
    };
    if *drawn == Visibility::Hidden {
        return;
    }
    let from = eye.translation;
    let points = points_on(you);
    // The box round every line, which nearly everything is nowhere near.
    let (reach_low, reach_high) = points
        .iter()
        .fold((from, from), |(low, high), &point| (low.min(point), high.max(point)));
    let mut found = Vec::new();
    let mut near = Vec::with_capacity(points.len());
    for (part, mesh, material, bounds, place, shown, see_through) in &parts {
        if !shown.get() {
            continue;
        }
        let (low, high) = world_box(bounds, place);
        if low.cmpgt(reach_high).any()
            || high.cmplt(reach_low).any()
            || ground(part, high.y, &lineage)
        {
            continue;
        }
        // Only what is drawn solid, or is see-through already for being in
        // the way: water and glass are see-through as they are. But a door
        // or a window, glass and all, is in the way as the wall it is in
        // would be (Hajun, 2026-10-16): seen through one, the wall goes
        // see-through, and what is in it with it.
        let own = see_through.map_or(&material.0, |see| &see.solid);
        let Some(look) = materials.get(own) else {
            continue;
        };
        let stands_for = match in_a_wall(part, &lineage, &on_walls) {
            Some(wall) => wall,
            None if drawn_solid(look) => part,
            None => continue,
        };
        // Then the lines that go through the box round it.
        near.clear();
        near.extend(
            points
                .iter()
                .copied()
                .filter(|&to| line_meets_box(from, to, low, high)),
        );
        if near.is_empty() {
            continue;
        }
        let Some(shape) = shape_of(mesh, &meshes, &mut shapes) else {
            continue;
        };
        let into = place.affine().inverse();
        let start = into.transform_point3(from);
        if near
            .iter()
            .any(|&to| shape.crosses(start, into.transform_point3(to)))
        {
            found.push(stands_for);
        }
    }
    // Standing in a house, any of its walls in the way of you takes its
    // ceiling with it, so that the room you are in is open to the camera
    // from above as well (Hajun, 2026-10-16).
    if let Ok(house) = house_you_are_in.single()
        && found
            .iter()
            .any(|&part| walls.get(part).is_ok_and(|wall| wall.parent() == house))
    {
        let ceiling: Vec<Entity> = children
            .iter_descendants(house)
            .filter(|&part| {
                !walls.contains(part)
                    && model_of(part, &lineage) == house
                    && parts.get(part).is_ok_and(|(.., bounds, place, _, _)| {
                        over_your_head(world_box(bounds, place).0.y, you)
                    })
            })
            .collect();
        found.extend(ceiling);
    }
    let target = menu.target();
    for part in found {
        if hiding.0.contains_key(&part) {
            continue;
        }
        let Some((fellows, owner)) = fades_with(part, &lineage, &children, &parts, target) else {
            continue;
        };
        // What goes with it: what is inside it, which is read once, as what
        // is in a part of a house or of the island stays in it; and if it is
        // a wall, whatever is in the wall now.
        let within = inside
            .entry(part)
            .or_insert_with(|| inside_of(&fellows, &lineage, &children, &parts));
        let along: Vec<Entity> = within
            .iter()
            .copied()
            .chain(
                on_walls
                    .iter()
                    .filter(|(_, on)| fellows.contains(&on.wall))
                    .map(|(piece, _)| piece),
            )
            .collect();
        hiding.0.extend(fellows.into_iter().map(|fellow| (fellow, owner)));
        for other in along {
            if hiding.0.contains_key(&other) {
                continue;
            }
            if let Some((fellows, owner)) = fades_with(other, &lineage, &children, &parts, target) {
                hiding.0.extend(fellows.into_iter().map(|fellow| (fellow, owner)));
            }
        }
    }
}

/// What has to fade along with `fellows`, a part of a house or of the island
/// fading as a whole, so that nothing of it is left standing solid in it:
/// whatever else of the same house or island is inside the box round it,
/// like the doors, windows and sign of a house in the town, which are
/// objects of their own in the model. Nothing, for anything else, which
/// fades whole already. (What is in a wall of a house built in House Builder
/// fades with it too, by `build::OnWall`.)
fn inside_of(
    fellows: &[Entity],
    lineage: &Lineage,
    children: &Query<&Children>,
    parts: &Parts,
) -> Vec<Entity> {
    let mut inside = Vec::new();
    let Some(&first) = fellows.first() else {
        return inside;
    };
    let model = model_of(first, lineage);
    let whole = lineage
        .get(model)
        .is_ok_and(|(.., piece, town_object, _, _)| {
            town_object || piece.is_some_and(|piece| piece.def.mount != Mount::House)
        });
    if whole {
        return inside;
    }
    let Some((low, high)) = fellows
        .iter()
        .filter_map(|&fellow| parts.get(fellow).ok())
        .map(|(_, _, _, bounds, place, ..)| world_box(bounds, place))
        .reduce(|(low, high), (l, h)| (low.min(l), high.max(h)))
    else {
        return inside;
    };
    let (low, high) = (low - IN_IT, high + IN_IT);
    for inner in children.iter_descendants(model) {
        let Ok((.., bounds, place, _, _)) = parts.get(inner) else {
            continue;
        };
        let (l, h) = world_box(bounds, place);
        if !fellows.contains(&inner)
            && l.cmpge(low).all()
            && h.cmple(high).all()
            && model_of(inner, lineage) == model
        {
            inside.push(inner);
        }
    }
    inside
}

/// The wall that `part` is a part of a door, a window or a hanging in, if it
/// is: of a house built in House Builder (`build::OnWall`), its glass as well
/// as its frame. In the way, it is the wall that is.
fn in_a_wall(part: Entity, lineage: &Lineage, on_walls: &Query<(Entity, &OnWall)>) -> Option<Entity> {
    on_walls
        .get(model_of(part, lineage))
        .ok()
        .map(|(_, on)| on.wall)
}

/// What `part` is part of: the piece or town object it is in, or else the
/// island it is on.
fn model_of(part: Entity, lineage: &Lineage) -> Entity {
    let mut at = part;
    loop {
        let Ok((parent, _, piece, town_object, ..)) = lineage.get(at) else {
            return at;
        };
        match parent {
            _ if piece.is_some() || town_object => return at,
            Some(parent) => at = parent.parent(),
            None => return at,
        }
    }
}

/// Fades out what has come between the camera and you, and back in what has
/// gone from between, and puts out of a tap's reach whatever any of that is
/// part of, for as long as any of it is see-through.
fn fade_what_hides_you(
    time: Res<Time>,
    hiding: Res<Hiding>,
    looks: Query<&MeshMaterial3d<StandardMaterial>>,
    mut fading: Query<(Entity, &mut SeeThrough)>,
    marked: Query<Entity, With<InTheWay>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mut out_of_reach: HashSet<Entity> = HashSet::new();
    // Just come in the way: from now on drawn with a see-through copy of its
    // own material, as solid as it was, to fade from there.
    for (&part, &owner) in &hiding.0 {
        out_of_reach.extend(owner);
        if fading.contains(part) {
            continue;
        }
        let Ok(own) = looks.get(part) else {
            continue;
        };
        let Some((solid, alpha)) = materials
            .get(&own.0)
            .and_then(|solid| Some((solid.clone(), own_alpha(solid)?)))
        else {
            continue;
        };
        let look = materials.add(see_through(solid, alpha));
        commands.entity(part).try_insert((
            SeeThrough {
                solid: own.0.clone(),
                look: look.clone(),
                alpha,
                shown: 1.0,
                owner,
            },
            MeshMaterial3d(look),
        ));
    }
    // Each a step nearer as much of it as should be drawn: none of it while
    // it is in the way, all of it once it is not, and then its own material
    // back.
    let step = time.delta_secs() / FADE_SECS * (1.0 - SEEN);
    for (part, mut see) in &mut fading {
        out_of_reach.extend(see.owner);
        let goal = if hiding.0.contains_key(&part) { SEEN } else { 1.0 };
        let shown = if see.shown < goal {
            (see.shown + step).min(goal)
        } else {
            (see.shown - step).max(goal)
        };
        if shown == see.shown {
            continue;
        }
        see.shown = shown;
        if shown >= 1.0 {
            let mut solid = commands.entity(part);
            // Unless something else has given it a material of its own since.
            if looks.get(part).is_ok_and(|drawn| drawn.0 == see.look) {
                solid.try_insert(MeshMaterial3d(see.solid.clone()));
            }
            solid.try_remove::<SeeThrough>();
        } else if let Some(mut look) = materials.get_mut(&see.look) {
            look.base_color.set_alpha(see.alpha * shown);
        }
    }
    for owner in &marked {
        if !out_of_reach.contains(&owner) {
            commands.entity(owner).try_remove::<InTheWay>();
        }
    }
    for owner in out_of_reach {
        if !marked.contains(owner) {
            commands.entity(owner).try_insert(InTheWay);
        }
    }
}

/// Whether `material` is drawn solid, and so can be in the way: not water,
/// glass, a ghost or the sky, which are blended already. (The glass of a
/// door or a window is in the way all the same, as its wall: [`in_a_wall`].)
fn drawn_solid(material: &StandardMaterial) -> bool {
    matches!(material.alpha_mode, AlphaMode::Opaque)
}

/// How solid something drawn with `material` is, if it can fade: 1 for
/// anything solid, and its own alpha for glass and water, which fade along
/// with what they are in. `None` for anything else, like the sun's glow,
/// which is added to what is behind it.
fn own_alpha(material: &StandardMaterial) -> Option<f32> {
    match material.alpha_mode {
        AlphaMode::Opaque => Some(1.0),
        AlphaMode::Blend => Some(material.base_color.alpha()),
        _ => None,
    }
}

/// `solid` as a part in the way is drawn: in its own colours, only the side
/// of it facing the camera, and starting as solid as it was, `alpha`.
///
/// What is solid is drawn with alpha to coverage: with the camera's 4x MSAA,
/// a quarter of it covers one of the four samples in each pixel, writes its
/// depth there, and leaves the other three to what is behind. Whatever of it
/// is nearest the camera in a pixel hides the rest of it there, so a house
/// in the way is one even pane, however its parts overlap. Blended, as it
/// was until 2026-10-13, every face of it over another added another layer,
/// and the edges of its parts showed through each other, darker, which
/// Hajun found dirty. Glass and water stay blended, as they are drawn, and
/// so does anything with a texture, whose alpha Bevy sharpens to all or
/// nothing under alpha to coverage (`pbr_fragment.wgsl`): of the shop's
/// pieces, the Soft Couch, which is one mesh.
fn see_through(mut look: StandardMaterial, alpha: f32) -> StandardMaterial {
    look.alpha_mode = if look.alpha_mode == AlphaMode::Blend || look.base_color_texture.is_some() {
        AlphaMode::Blend
    } else {
        AlphaMode::AlphaToCoverage
    };
    look.base_color.set_alpha(alpha);
    look.cull_mode = Some(Face::Back);
    look.double_sided = false;
    look
}

/// The triangles of `mesh`, in its own space, read the first time they are
/// asked for. `None` until it has loaded.
fn shape_of<'a>(
    mesh: &Mesh3d,
    meshes: &Assets<Mesh>,
    shapes: &'a mut HashMap<AssetId<Mesh>, Faces>,
) -> Option<&'a Faces> {
    let id = mesh.id();
    if !shapes.contains_key(&id) {
        let mut corners = Vec::new();
        island::add_mesh(&mut corners, meshes.get(id)?, &GlobalTransform::IDENTITY);
        shapes.insert(id, Faces::of(&corners));
    }
    shapes.get(&id)
}

/// The points on `you` that the camera has to see ([`RINGS`]), where you
/// stand and the way you face.
fn points_on(you: &Transform) -> Vec<Vec3> {
    let (across, ahead) = (you.right().as_vec3(), you.forward().as_vec3());
    let round: [(f32, f32); ROUND] = std::array::from_fn(|step| {
        (step as f32 * std::f32::consts::TAU / ROUND as f32).sin_cos()
    });
    let mut points = Vec::with_capacity(RINGS.len() * (ROUND + 1) + 1);
    for (up, wide, deep) in RINGS {
        let middle = you.translation + Vec3::Y * up;
        points.push(middle);
        for (sin, cos) in round {
            points.push(middle + across * (wide * cos) + ahead * (deep * sin));
        }
    }
    points.push(you.translation + Vec3::Y * CROWN);
    points
}

/// Whether a part of the house you are in, whose box goes no lower than
/// `bottom`, is its ceiling over `you`: wholly over your head, like its roof
/// and gables, and downstairs the floor upstairs, but not the floor you stand
/// on, its walls or its stairs. It fades along with any of the house's walls
/// in the way of you.
fn over_your_head(bottom: f32, you: &Transform) -> bool {
    bottom > you.translation.y + CROWN
}

/// Whether `part`, whose box goes no higher than `top`, is the ground: the
/// island's own, not anything built or put on it, and no higher than a step
/// over the land, as the town's paving, roads and grass are, and a plot's.
/// The ground never fades. What it hides of you is your feet, below a kerb
/// or a step up from you, or your legs while you are in the sea, and made
/// see-through it would take the whole island with it: the town's paving is
/// one piece 140 m across.
fn ground(part: Entity, top: f32, lineage: &Lineage) -> bool {
    if top > crate::LAND_TOP + crate::LAND_STEP_UP {
        return false;
    }
    let mut at = Some(part);
    while let Some(entity) = at {
        let Ok((parent, _, piece, town_object, ..)) = lineage.get(entity) else {
            break;
        };
        if piece.is_some() || town_object {
            return false;
        }
        at = parent.map(ChildOf::parent);
    }
    true
}

/// The box round a part standing at `place`, which fits in `bounds` in its
/// own space, from its lowest corner to its highest.
fn world_box(bounds: &Aabb, place: &GlobalTransform) -> (Vec3, Vec3) {
    let affine = place.affine();
    let middle = Vec3::from(affine.transform_point3a(bounds.center));
    let half = Vec3::from(affine.matrix3.abs() * bounds.half_extents);
    (middle - half, middle + half)
}

/// Whether the straight line from `from` to `to` goes through the box from
/// `low` to `high`, or ends in it.
fn line_meets_box(from: Vec3, to: Vec3, low: Vec3, high: Vec3) -> bool {
    let way = to - from;
    let (mut near, mut far) = (0.0f32, 1.0f32);
    for axis in 0..3 {
        if way[axis].abs() < 1e-9 {
            if from[axis] < low[axis] || from[axis] > high[axis] {
                return false;
            }
            continue;
        }
        let (a, b) = (
            (low[axis] - from[axis]) / way[axis],
            (high[axis] - from[axis]) / way[axis],
        );
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return false;
        }
    }
    true
}

/// What `part`, found in the way of you, fades along with, and what it is
/// part of that a tap could reach, if anything; or `None` if it is to stay
/// as it is, as part of a piece being put down or of whatever the edit menu
/// is open on (`menu`).
///
/// A piece of furniture, a door, a window or a town object fades whole. A
/// house, and the island, fade a part at a time: the meshes the glTF loader
/// made of one object in Blender, one for each of its materials, and of the
/// objects parented to it, like a door's handles; or a wall the game made
/// again (`build`).
fn fades_with(
    part: Entity,
    lineage: &Lineage,
    children: &Query<&Children>,
    parts: &Parts,
    menu: Option<Entity>,
) -> Option<(Vec<Entity>, Option<Entity>)> {
    let (parent, from_blender, ..) = lineage.get(part).ok()?;
    // The object in Blender it is a mesh of.
    let object = parent.filter(|_| from_blender).map(ChildOf::parent);
    let (mut whole, mut owner) = (None, None);
    let mut at = Some(part);
    while let Some(entity) = at {
        let Ok((parent, _, piece, town_object, editable, ghost)) = lineage.get(entity) else {
            break;
        };
        if ghost || (editable && menu == Some(entity)) {
            return None;
        }
        if editable && owner.is_none() {
            owner = Some(entity);
        }
        // A piece is the furthest up a part goes: a door is in its house, but
        // fades on its own.
        if piece.is_some() || town_object {
            if piece.is_none_or(|piece| piece.def.mount != Mount::House) {
                whole = Some(entity);
            }
            break;
        }
        at = parent.map(ChildOf::parent);
    }
    let fellows = match (whole, object) {
        (Some(whole), _) => children
            .iter_descendants(whole)
            .filter(|&fellow| parts.contains(fellow))
            .collect(),
        (None, Some(object)) => children
            .iter_descendants(object)
            .filter(|&fellow| parts.contains(fellow))
            .collect(),
        (None, None) => vec![part],
    };
    Some((fellows, owner))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A house's walls, floor and roof, 12 m square and 5 m to the top of its
    /// walls, as boxes, with you in the middle of it.
    const FLOOR: (Vec3, Vec3) = (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.1, 6.0));
    const FRONT: (Vec3, Vec3) = (Vec3::new(-6.0, 0.1, 5.5), Vec3::new(6.0, 5.1, 6.0));
    const BACK: (Vec3, Vec3) = (Vec3::new(-6.0, 0.1, -6.0), Vec3::new(6.0, 5.1, -5.5));
    const LEFT: (Vec3, Vec3) = (Vec3::new(-6.0, 0.1, -5.5), Vec3::new(-5.5, 5.1, 5.5));
    const ROOF: (Vec3, Vec3) = (Vec3::new(-6.2, 5.1, -6.2), Vec3::new(6.2, 5.6, 6.2));
    /// A bed in front of you, between you and the front wall: Hajun's double
    /// bed is 0.95 m high, and on a floor 0.1 m up.
    const BED: (Vec3, Vec3) = (Vec3::new(-1.5, 0.1, 1.0), Vec3::new(1.5, 1.05, 3.0));

    /// Whether the box from `low` to `high` hides any of you, standing at the
    /// origin and facing -z, from a camera at `eye`.
    fn hides(eye: Vec3, part: (Vec3, Vec3)) -> bool {
        hides_you(eye, &Transform::IDENTITY, part)
    }

    /// Whether the box from `low` to `high` hides any of `you` from a camera
    /// at `eye`: tried as a part is, against the box round it and then its
    /// triangles.
    fn hides_you(eye: Vec3, you: &Transform, (low, high): (Vec3, Vec3)) -> bool {
        let shape = Faces::of(&crate::camera_tests::boxes(&[(low, high)]));
        points_on(you)
            .into_iter()
            .any(|to| line_meets_box(eye, to, low, high) && shape.crosses(eye, to))
    }

    #[test]
    fn from_outside_a_house_the_near_wall_and_the_roof_are_in_the_way() {
        // Zoomed out behind you, level and from high up: out through the
        // front wall, and up through the roof.
        let level = Vec3::new(0.0, 2.0, 10.0);
        let high = Vec3::new(0.0, 12.0, 6.0);
        assert!(hides(level, FRONT));
        assert!(!hides(level, ROOF));
        assert!(hides(high, ROOF));
        for eye in [level, high] {
            // Never what is behind you, beside you or under you.
            for (name, part) in [("back", BACK), ("left", LEFT), ("floor", FLOOR)] {
                assert!(!hides(eye, part), "{name} from {eye}");
            }
        }
        // A bed in front of you hides your legs from a camera nearly level
        // with you, and is in the way; from high up it hides nothing.
        assert!(hides(level, BED));
        assert!(!hides(high, BED));
        // From in the room with you, only the bed, which you stand behind:
        // looking down past it, the camera cannot see your feet.
        let inside = Vec3::new(0.0, 2.5, 4.5);
        for part in [FLOOR, FRONT, BACK, LEFT, ROOF] {
            assert!(!hides(inside, part));
        }
        assert!(hides(inside, BED));
    }

    #[test]
    fn a_wall_that_hides_only_your_side_is_in_the_way() {
        // Between the camera and you, ending a little to the right of the
        // straight line from it to your middle: it hides your right arm and
        // shoulder, and nothing else of you.
        let eye = Vec3::new(0.0, 2.0, 10.0);
        let wall = (Vec3::new(0.15, 0.0, 3.0), Vec3::new(5.0, 3.0, 3.3));
        let shape = Faces::of(&crate::camera_tests::boxes(&[wall]));
        for up in [0.2, 1.05, 1.4, 1.68] {
            assert!(!shape.crosses(eye, Vec3::Y * up), "your middle, {up} m up");
        }
        assert!(hides(eye, wall));
        // Turned round, facing the camera, you are as wide, and it still is.
        let turned = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI));
        assert!(hides_you(eye, &turned, wall));
        // A little further out, it hides none of you.
        let clear = (Vec3::new(0.4, 0.0, 3.0), Vec3::new(5.0, 3.0, 3.3));
        assert!(!hides(eye, clear));
    }

    #[test]
    fn a_wall_you_stand_against_is_in_the_way_only_from_its_far_side() {
        // On your right, as near as your body comes to it.
        let wall = (
            Vec3::new(crate::PLAYER_RADIUS, 0.0, -5.0),
            Vec3::new(crate::PLAYER_RADIUS + 0.5, 3.0, 5.0),
        );
        for eye in [Vec3::new(0.0, 2.0, 10.0), Vec3::new(-3.0, 5.0, 8.0), Vec3::new(0.0, 13.2, 0.0)] {
            assert!(!hides(eye, wall), "from {eye}");
        }
        assert!(hides(Vec3::new(4.0, 2.0, 9.0), wall));
        // And something under your hips beside your legs, as near as they
        // come to it.
        let low = (
            Vec3::new(crate::LEG_RADIUS, 0.0, -1.0),
            Vec3::new(1.0, crate::HIP_HEIGHT - 0.05, 1.0),
        );
        assert!(!hides(Vec3::new(0.0, 2.0, 10.0), low));
        assert!(!hides(Vec3::new(-3.0, 1.0, 8.0), low));
    }

    #[test]
    fn looking_straight_down_the_roof_is_in_the_way_and_the_floor_is_not() {
        let eye = Vec3::new(0.0, 13.2, 0.0);
        assert!(hides(eye, ROOF));
        assert!(!hides(eye, FLOOR));
        // Nor the floor you stand on from as low as the camera goes behind
        // you, looking up past you.
        assert!(!hides(Vec3::new(0.0, 0.58, 2.72), FLOOR));
    }

    #[test]
    fn standing_in_a_house_its_ceiling_is_what_is_over_your_head() {
        // On the floor of the house: its roof is, its floor and walls are not.
        let downstairs = Transform::from_xyz(0.0, FLOOR.1.y, 0.0);
        assert!(over_your_head(ROOF.0.y, &downstairs));
        for (name, part) in [("floor", FLOOR), ("front", FRONT), ("left", LEFT)] {
            assert!(!over_your_head(part.0.y, &downstairs), "{name}");
        }
        // With a floor upstairs 5 m up, as in Hajun's Two Floor House, and
        // its roof 5 m higher: downstairs both are, upstairs only the roof.
        let upper = (Vec3::new(-6.0, 4.9, -6.0), Vec3::new(4.0, 5.1, 6.0));
        let roof = ROOF.0.y + 5.0;
        assert!(over_your_head(upper.0.y, &downstairs));
        let upstairs = Transform::from_xyz(0.0, upper.1.y, 0.0);
        assert!(!over_your_head(upper.0.y, &upstairs));
        assert!(over_your_head(roof, &upstairs));
    }

    #[test]
    fn every_point_on_you_is_inside_you_and_turns_with_you() {
        let feet = Vec3::new(3.0, 0.5, -2.0);
        let facing = Transform::from_translation(feet);
        let turned = facing.with_rotation(Quat::from_rotation_y(1.1));
        for you in [facing, turned] {
            let points = points_on(&you);
            assert_eq!(points.len(), RINGS.len() * (ROUND + 1) + 1);
            for point in points {
                let (up, out) = (point.y - feet.y, point.xz().distance(feet.xz()));
                // Over any edge your legs stand over, and under the top of
                // your head.
                assert!(up > crate::EDGE_RISE && up < 1.71, "{up} up");
                // Inside whatever you bump into things with at that height.
                let reach = if up < crate::HIP_HEIGHT {
                    crate::LEG_RADIUS
                } else {
                    assert!(up > crate::WAIST_HEIGHT, "{up} up, in your hips");
                    crate::PLAYER_RADIUS
                };
                assert!(out < reach, "{out} out, {up} up");
            }
        }
        // Across you, the way you face: your body is wide across and thin
        // front to back.
        let across = |you: &Transform| {
            points_on(you)
                .into_iter()
                .map(|point| (point - you.translation).dot(you.right().as_vec3()))
                .fold(0.0f32, f32::max)
        };
        assert!((across(&facing) - 0.36).abs() < 1e-5);
        assert!((across(&turned) - 0.36).abs() < 1e-5);
    }

    #[test]
    fn a_turned_part_is_tried_where_it_stands() {
        // The front wall, turned a quarter round to stand off to one side.
        let shape = Faces::of(&crate::camera_tests::boxes(&[FRONT]));
        let bounds = Aabb::from_min_max(FRONT.0, FRONT.1);
        let place = GlobalTransform::from(Transform::from_rotation(Quat::from_rotation_y(
            std::f32::consts::FRAC_PI_2,
        )));
        let into = place.affine().inverse();
        let (low, high) = world_box(&bounds, &place);
        let tried = |eye: Vec3| {
            let to = Vec3::Y * 1.4;
            line_meets_box(eye, to, low, high)
                && shape.crosses(into.transform_point3(eye), into.transform_point3(to))
        };
        assert!(tried(Vec3::new(10.0, 2.0, 0.0)));
        assert!(!tried(Vec3::new(0.0, 2.0, 10.0)));
    }

    #[test]
    fn a_line_into_a_box_meets_it_and_one_short_of_it_does_not() {
        let (low, high) = (Vec3::splat(-1.0), Vec3::splat(1.0));
        assert!(line_meets_box(Vec3::new(5.0, 0.0, 0.0), Vec3::ZERO, low, high));
        assert!(line_meets_box(Vec3::new(5.0, 0.5, 0.5), Vec3::new(-5.0, 0.5, 0.5), low, high));
        assert!(!line_meets_box(Vec3::new(5.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0), low, high));
        assert!(!line_meets_box(Vec3::new(5.0, 2.0, 0.0), Vec3::new(-5.0, 2.0, 0.0), low, high));
        // Along an axis, in it and beside it.
        assert!(line_meets_box(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -5.0), low, high));
        assert!(!line_meets_box(Vec3::new(3.0, 0.0, 5.0), Vec3::new(3.0, 0.0, -5.0), low, high));
    }

    #[test]
    fn see_through_is_alpha_to_coverage_and_starts_solid() {
        let solid = StandardMaterial {
            base_color: Color::srgb(0.8, 0.6, 0.4),
            double_sided: true,
            cull_mode: None,
            ..default()
        };
        assert!(drawn_solid(&solid));
        assert_eq!(own_alpha(&solid), Some(1.0));
        let look = see_through(solid.clone(), 1.0);
        assert!(!drawn_solid(&look));
        assert_eq!(look.alpha_mode, AlphaMode::AlphaToCoverage);
        assert_eq!(look.base_color.alpha(), 1.0);
        assert_eq!(look.cull_mode, Some(Face::Back));
        assert_eq!(look.base_color.with_alpha(1.0), solid.base_color);
        // Glass is never in the way, but fades with the window it is in,
        // from as see-through as it is, and stays blended.
        let glass = StandardMaterial {
            base_color: Color::srgba(0.8, 0.9, 1.0, 0.2),
            alpha_mode: AlphaMode::Blend,
            ..default()
        };
        assert!(!drawn_solid(&glass));
        assert_eq!(own_alpha(&glass), Some(0.2));
        let pane = see_through(glass, 0.2);
        assert_eq!(pane.alpha_mode, AlphaMode::Blend);
        assert_eq!(pane.base_color.alpha(), 0.2);
        // Something with a texture would go all or nothing under alpha to
        // coverage, and is blended.
        let textured = StandardMaterial {
            base_color_texture: Some(Handle::default()),
            ..default()
        };
        assert_eq!(see_through(textured, 1.0).alpha_mode, AlphaMode::Blend);
        // And what is added to what is behind it does not fade at all.
        let glow = StandardMaterial {
            alpha_mode: AlphaMode::Add,
            ..default()
        };
        assert_eq!(own_alpha(&glow), None);
    }
}
