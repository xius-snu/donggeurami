//! Whatever stands between the camera and you is drawn see-through, and a tap
//! goes through it to what is behind.
//!
//! The camera stands where you turned and zoomed it, walls or no walls
//! (`follow_camera`): in a house, zoomed out, it is outside the house looking
//! in. Every frame the straight lines from it to your body, your chest and
//! your head are tried against everything drawn on your island, and whatever
//! any of them goes through fades to a quarter of itself, as Roblox's
//! Invisicam fades it, and back once none does (Hajun, 2026-10-09). Zoomed
//! all the way in, through your own eyes, there is nothing between.
//!
//! A piece of furniture, a door, a window or a town object fades whole. A
//! house, and the island itself, fade a part at a time, each part as it was
//! modelled in Blender: a wall, the roof, a floor, a tree's leaves. Bodies
//! never fade, nor water and glass, which are see-through already, nor a
//! piece being put down, nor whatever the edit menu is open on.
//!
//! Something a tap can reach (`editor::Editable`) is out of its reach for as
//! long as any of it is see-through ([`InTheWay`]): a tap goes through it, so
//! that from outside a house you can reach in and tap what is in there with
//! you. Only what is not in the way of you can be tapped.
//!
//! A part is drawn see-through with a copy of its own material, blended, as
//! everything see-through in this world is, so that both platforms draw it
//! alike (`RENDERING.md`).

use std::collections::{HashMap, HashSet};

use bevy::camera::primitives::Aabb;
use bevy::gltf::GltfMeshName;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

use crate::build::{Ghost, Mount, Piece};
use crate::editor::{EditMenu, EditSystems, Editable};
use crate::island::{self, Faces};
use crate::map::MapObject;
use crate::{Player, ThirdPersonCamera};

/// How much of something in the way of you is drawn: a quarter of it, as
/// much as Roblox's Invisicam leaves of what it fades.
const SEEN: f32 = 0.25;
/// How long something takes to fade, and to come back, in seconds.
const FADE_SECS: f32 = 0.15;
/// What of you the camera has to see, as heights over your feet: the bottom
/// of your body, where your legs end, your chest and your head. Whatever
/// hides any of them is in the way; what hides only your legs, like a bed you
/// stand behind, is not.
const SEEN_AT: [f32; 3] = [1.15, 1.4, 1.65];

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
    /// How much of it is drawn, from 1, solid, down to [`SEEN`].
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
    menu: Res<EditMenu>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    mut changed: MessageReader<AssetEvent<Mesh>>,
    mut shapes: Local<HashMap<AssetId<Mesh>, Faces>>,
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
    let lines = SEEN_AT.map(|up| (eye.translation, you.translation + Vec3::Y * up));
    let mut found = Vec::new();
    for (part, mesh, material, bounds, place, shown, see_through) in &parts {
        // The box round it first, which nearly everything is nowhere near.
        if !shown.get() || !lines.iter().any(|&(from, to)| in_box(from, to, bounds, place)) {
            continue;
        }
        // Only what is drawn solid, or is see-through already for being in
        // the way: water and glass are see-through as they are.
        let own = see_through.map_or(&material.0, |see| &see.solid);
        if !materials.get(own).is_some_and(drawn_solid) {
            continue;
        }
        let Some(shape) = shape_of(mesh, &meshes, &mut shapes) else {
            continue;
        };
        let into = place.affine().inverse();
        let through = |&(from, to): &(Vec3, Vec3)| {
            shape.crosses(into.transform_point3(from), into.transform_point3(to))
        };
        if lines.iter().any(through) {
            found.push(part);
        }
    }
    for part in found {
        if hiding.0.contains_key(&part) {
            continue;
        }
        if let Some((fellows, owner)) = fades_with(part, &lineage, &children, &parts, menu.target())
        {
            hiding.0.extend(fellows.into_iter().map(|fellow| (fellow, owner)));
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
        let Some(solid) = materials.get(&own.0).filter(|solid| drawn_solid(solid)).cloned() else {
            continue;
        };
        let look = materials.add(see_through(solid));
        commands.entity(part).try_insert((
            SeeThrough {
                solid: own.0.clone(),
                look: look.clone(),
                shown: 1.0,
                owner,
            },
            MeshMaterial3d(look),
        ));
    }
    // Each a step nearer as much of it as should be drawn: a quarter while it
    // is in the way, all of it once it is not, and then its own material back.
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
            look.base_color.set_alpha(shown);
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

/// Whether `material` is drawn solid, and so can be made see-through: not
/// water, glass, a ghost or the sky, which are blended already.
fn drawn_solid(material: &StandardMaterial) -> bool {
    matches!(material.alpha_mode, AlphaMode::Opaque)
}

/// `solid` as a part in the way is drawn: blended, in its own colours, and
/// only the side of it facing the camera, so that a wall is one pane rather
/// than its two faces one over the other. It starts as solid as it was.
fn see_through(mut look: StandardMaterial) -> StandardMaterial {
    look.alpha_mode = AlphaMode::Blend;
    look.base_color.set_alpha(1.0);
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

/// Whether the straight line from `from` to `to` goes through the box round a
/// part standing at `place`, which fits in `bounds` in its own space.
fn in_box(from: Vec3, to: Vec3, bounds: &Aabb, place: &GlobalTransform) -> bool {
    let affine = place.affine();
    let middle = Vec3::from(affine.transform_point3a(bounds.center));
    let half = Vec3::from(affine.matrix3.abs() * bounds.half_extents);
    line_meets_box(from, to, middle - half, middle + half)
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
/// made of one object in Blender, one for each of its materials, or a wall the
/// game made again (`build`).
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
            .get(object)
            .into_iter()
            .flat_map(|meshes| meshes.iter())
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

    /// Whether the box from `low` to `high` hides you, standing at the
    /// origin, from a camera at `eye`: tried as a part is, against the box
    /// round it and then its triangles.
    fn hides(eye: Vec3, (low, high): (Vec3, Vec3)) -> bool {
        let shape = Faces::of(&crate::camera_tests::boxes(&[(low, high)]));
        let bounds = Aabb::from_min_max(low, high);
        let place = GlobalTransform::IDENTITY;
        SEEN_AT.iter().any(|&up| {
            let to = Vec3::Y * up;
            in_box(eye, to, &bounds, &place) && shape.crosses(eye, to)
        })
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
            // Never what is behind you, beside you or under you, nor a bed,
            // which hides only your legs.
            for (name, part) in [("back", BACK), ("left", LEFT), ("floor", FLOOR), ("bed", BED)] {
                assert!(!hides(eye, part), "{name} from {eye}");
            }
        }
        // From in the room with you, nothing.
        let inside = Vec3::new(0.0, 2.5, 4.5);
        for part in [FLOOR, FRONT, BACK, LEFT, ROOF, BED] {
            assert!(!hides(inside, part));
        }
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
        let tried = |eye: Vec3| {
            let to = Vec3::Y * SEEN_AT[1];
            in_box(eye, to, &bounds, &place)
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
    fn see_through_is_blended_and_starts_solid() {
        let solid = StandardMaterial {
            base_color: Color::srgb(0.8, 0.6, 0.4),
            double_sided: true,
            cull_mode: None,
            ..default()
        };
        assert!(drawn_solid(&solid));
        let look = see_through(solid.clone());
        assert!(!drawn_solid(&look));
        assert_eq!(look.alpha_mode, AlphaMode::Blend);
        assert_eq!(look.base_color.alpha(), 1.0);
        assert_eq!(look.cull_mode, Some(Face::Back));
        assert_eq!(look.base_color.with_alpha(1.0), solid.base_color);
    }
}
