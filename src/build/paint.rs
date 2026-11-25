//! Paint on what stands on a plot of the town (since 2026-11-22; Hajun:
//! "the second one being a paintbrush icon (which will be a button for
//! coloring/texturing the objects/house ... u can select a color there and
//! then u can tap objects and it will paint it to that color (so if u tap
//! the blanket of the red bed, it will just change that red color to the
//! color that u chose, or if u tap your house wall, it will just change the
//! color of the walls, and if u tap the roof, it will just color the roof,
//! if u tap the floor, it will just color the floor, so its like a modular
//! colorer").
//!
//! **A part is a material.** Each of Hajun's models is drawn with a few
//! materials, named in Blender for what they are: a bed's `blanket`,
//! `bedframe` and `mattress`, a house's `plain_wall`, `plain_roof` and
//! `plain_floor`. Everything of one thing that is drawn with one material
//! is one part of it ([`Part`], given to each mesh as its model spawns,
//! `super::piece_ready`), and is painted together: a tap on a bed's blanket
//! paints all of its blanket.
//!
//! **In a house, a part is a material on one floor** (since 2026-11-25;
//! Hajun: "each floor need to be its own color/material when painting"):
//! a tap on a wall of the ground floor paints the ground floor's walls,
//! and its stairs, which are in the walls' material; one upstairs, that
//! floor's. Which floor a mesh is on goes by the tops of the house's
//! floors ([`Levels`]), and what it is called by [`parts`]: the lowest
//! floor a material is on keeps the material's own name, so that a house
//! of one floor, the ground floor of a taller one and every roof are
//! painted under the names they always were, and each floor over that is
//! the material and its number, `plain_wall@2` (an app from before sees
//! the ground floor's colour all the way up). Blender's ".001" after a
//! copy of a material is no new material in a house: the roof terrace
//! houses' upper stairs are `plain_wall.001`, and are painted with their
//! floor's walls. The Square House 2F's four walls are each modelled whole
//! from the ground to the roof: each is drawn in two bands, cut at the top
//! of the upper floor, a part to each (`super::Banded`). A floor and the
//! ceiling under it are one slab, and painted together.
//!
//! **A tap paints** ([`tap`]) while the paintbrush is the tool
//! (`customize`): the part of yours under the finger, the nearest of them
//! the ray through the finger meets, found on the meshes themselves, so
//! that it is the blanket and not the frame beside it. With the cross
//! beside the colour in hand it takes the paint off instead, and the part
//! is as its model has it (`customize::Ink::Clear`). Whatever is drawn
//! see-through for standing between the camera and you is passed through,
//! as a tap passes through it everywhere else (`see_through`): from inside
//! your house a tap reaches the wall and the floor you can see.
//!
//! **What is painted is the bag's to say** (`bag::Bag::paints`, and
//! everyone else's `bag::TheirPaints`): `stand` hangs it on each thing as
//! [`Colours`], and [`colour_parts`] draws it, each painted part with a
//! copy of its own material in that colour ([`Looks`]: as see-through as it
//! was, so that glass stays glass, and textured as it was, so that a
//! pattern is tinted). A part nobody has painted has its own material, the
//! model's; and so has every part of a thing taken back into the bag and
//! stood again.

use std::collections::HashMap;

use bevy::picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings};
use bevy::prelude::*;
use roundtown_net::Ask;

use super::stand::{Mine, Stood};
use super::{BuildSystems, Ghost, Solid, Stock};
use crate::ThirdPersonCamera;
use crate::bag::Bag;
use crate::customize::Customize;
use crate::editor::{self, EditPointer};
use crate::hud::DialogUp;
use crate::net::Online;
use crate::see_through::SeeThrough;

/// A part of a thing that can be painted: one mesh of its model, with what
/// its material is called there, which is the part's name, and that
/// material, which it is drawn with while nobody has painted it.
#[derive(Component)]
pub(super) struct Part {
    name: String,
    plain: Handle<StandardMaterial>,
}

impl Part {
    pub(super) fn new(name: &str, plain: Handle<StandardMaterial>) -> Self {
        Self {
            name: name.to_string(),
            plain,
        }
    }
}

/// The colour a part is drawn in, while it is painted.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
struct Coloured([u8; 3]);

/// What of a thing is painted: each part by its name, and its colour, as a
/// screen has it. On everything that stands on a plot of the town, yours
/// and everyone else's, where `stand` keeps it as the bags have it.
#[derive(Component, Default, Clone, PartialEq, Debug)]
pub(crate) struct Colours(pub Vec<(String, [u8; 3])>);

impl Colours {
    /// What is painted of a thing: `painted`, each part and its colour.
    pub(crate) fn of<'a>(painted: impl Iterator<Item = (&'a str, [u8; 3])>) -> Self {
        Self(painted.map(|(part, colour)| (part.to_string(), colour)).collect())
    }

    fn colour(&self, part: &str) -> Option<[u8; 3]> {
        self.0.iter().find(|(name, _)| name == part).map(|&(_, colour)| colour)
    }
}

/// The painted copies of materials, each made the first time something is
/// painted that colour: by the material it is a copy of, and the colour.
#[derive(Resource, Default)]
struct Looks(HashMap<(AssetId<StandardMaterial>, [u8; 3]), Handle<StandardMaterial>>);

impl Looks {
    /// `plain`, painted `colour`.
    fn of(
        &mut self,
        plain: &Handle<StandardMaterial>,
        colour: [u8; 3],
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.0
            .entry((plain.id(), colour))
            .or_insert_with(|| {
                let mut look = materials.get(plain).cloned().unwrap_or_default();
                look.base_color = painted(look.base_color, colour);
                materials.add(look)
            })
            .clone()
    }
}

/// `colour` on something that was `plain`: as see-through as it was.
fn painted(plain: Color, colour: [u8; 3]) -> Color {
    Color::srgb_u8(colour[0], colour[1], colour[2]).with_alpha(plain.alpha())
}

/// How far under the top of a floor the foot of something may be and it
/// still be on that floor, and how near a floor's top a wall may end and
/// not run up past it, in metres: Hajun's walls stand on a floor's top,
/// give or take the last bits of a number.
const ON_A_FLOOR: f32 = 0.05;

/// What a mesh of a thing is, for naming it ([`parts`]): a floor of a
/// house, a wall of one, or anything else.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Shape {
    Floor,
    Wall,
    Other,
}

/// One mesh of a thing that can be painted, as its model has it: what its
/// material is called, how high its foot and its top are in the thing's
/// own space, and what it is.
pub(super) type Drawn<'a> = (&'a str, f32, f32, Shape);

/// The floors of a house, to tell what is on one of them from what is on
/// another: the top of each, bottom to top, each height once. Of anything
/// that is no house, none: all of it is on the one floor.
#[derive(Default, Debug, PartialEq)]
pub(super) struct Levels(Vec<f32>);

impl Levels {
    /// Of a house whose floors, its `Floor` objects, fill the boxes
    /// `floors`, in its own space.
    pub(super) fn of(floors: &[(Vec3, Vec3)]) -> Self {
        let mut tops: Vec<f32> = floors.iter().map(|(_, high)| high.y).collect();
        tops.sort_by(f32::total_cmp);
        tops.dedup_by(|above, below| *above - *below < ON_A_FLOOR);
        Self(tops)
    }

    /// Which floor something standing from `foot` up is on, the ground
    /// floor 1: the floor whose top it stands on, the highest under it. A
    /// floor is the floor it is the top of (its top as `foot`).
    pub(super) fn storey(&self, foot: f32) -> u32 {
        let under = self.0.iter().filter(|&&top| top <= foot + ON_A_FLOOR).count();
        u32::try_from(under).unwrap_or(u32::MAX).max(1)
    }

    /// Where a wall from `bottom` up to `top` goes from one floor to the
    /// next: the tops of the floors it runs up past, which it is cut at to
    /// be painted a floor at a time. None for a wall of one floor.
    pub(super) fn cuts(&self, bottom: f32, top: f32) -> Vec<f32> {
        self.0
            .iter()
            .copied()
            .filter(|&level| level > bottom + ON_A_FLOOR && level < top - ON_A_FLOOR)
            .collect()
    }
}

/// The material a part of a house is drawn with: its name in Blender, less
/// the ".001" Blender puts after the name of a copy of one.
pub(super) fn material_of(name: &str) -> &str {
    match name.rsplit_once('.') {
        Some((material, copy)) if !material.is_empty() && copy.len() == 3 && copy.bytes().all(|b| b.is_ascii_digit()) => material,
        _ => name,
    }
}

/// The names the parts of a thing are painted by ([`Part`]), and where a
/// house's walls are cut to be painted a floor at a time: of each of its
/// meshes, `drawn`, the bands it is drawn in, bottom first, each with how
/// high up it begins and its name. A mesh is one band, the whole of it,
/// but for a wall of a house that runs up past a floor, which is a band to
/// each floor it is on. A part is a material on a floor: the lowest floor
/// of the thing that material is on keeps the material's own name, and
/// each over it is called by the material and its floor (`plain_wall@2`).
/// Of anything but a house, every part is on the one floor, and called by
/// its material's name as its model has it.
pub(super) fn parts(house: bool, floors: &[(Vec3, Vec3)], drawn: &[Drawn]) -> Vec<Vec<(f32, String)>> {
    let levels = if house { Levels::of(floors) } else { Levels::default() };
    // Each mesh's bands, each with its material and the floor it is on.
    let bands: Vec<Vec<(f32, &str, u32)>> = drawn
        .iter()
        .map(|&(named, foot, top, shape)| {
            let material = if house { material_of(named) } else { named };
            match shape {
                Shape::Wall => std::iter::once(foot)
                    .chain(levels.cuts(foot, top))
                    .map(|bottom| (bottom, material, levels.storey(bottom)))
                    .collect(),
                Shape::Floor => vec![(foot, material, levels.storey(top))],
                Shape::Other => vec![(foot, material, levels.storey(foot))],
            }
        })
        .collect();
    // The lowest floor each material is on.
    let mut lowest: HashMap<&str, u32> = HashMap::new();
    for &(_, material, storey) in bands.iter().flatten() {
        lowest.entry(material).and_modify(|low| *low = (*low).min(storey)).or_insert(storey);
    }
    let name = |material: &str, storey: u32| match lowest.get(material) {
        Some(&low) if storey > low => format!("{material}@{storey}"),
        _ => material.to_string(),
    };
    bands
        .into_iter()
        .map(|bands| bands.into_iter().map(|(bottom, material, storey)| (bottom, name(material, storey))).collect())
        .collect()
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Looks>().add_systems(
        Update,
        (
            tap.after(editor::EditSystems).before(BuildSystems),
            // Once `stand` has said what is painted of each thing.
            colour_parts.after(BuildSystems).after(super::stand::Painting),
        ),
    );
}

/// The parts of `thing`, each mesh under it; but not those of another
/// thing that hangs from it, a door in a house's wall, which are that
/// thing's own.
fn parts_of(thing: Entity, children: &Query<&Children>, things: &Query<(Entity, &Colours)>) -> Vec<Entity> {
    let mut parts = Vec::new();
    let mut stack = vec![thing];
    while let Some(entity) = stack.pop() {
        for child in children.get(entity).into_iter().flat_map(|kids| kids.iter()) {
            if !things.contains(child) {
                parts.push(child);
                stack.push(child);
            }
        }
    }
    parts
}

/// Draws every painted part in its colour, and every part no longer
/// painted as its model has it: whenever what is painted of anything
/// changes, and as a model's parts are made. A part of a thing that is a
/// ghost just now (`super::ghost`) is given the colour to be put down in,
/// and drawn meanwhile as the ghost of that.
#[allow(clippy::too_many_arguments)]
fn colour_parts(
    changed: Query<(), Changed<Colours>>,
    made: Query<(), Added<Part>>,
    things: Query<(Entity, &Colours)>,
    children: Query<&Children>,
    mut parts: Query<(&Part, Option<&Coloured>, Option<&mut Solid>)>,
    mut looks: ResMut<Looks>,
    mut stock: ResMut<Stock>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if changed.is_empty() && made.is_empty() {
        return;
    }
    for (thing, colours) in &things {
        for part in parts_of(thing, &children, &things) {
            let Ok((of, coloured, solid)) = parts.get_mut(part) else {
                continue;
            };
            let wanted = colours.colour(&of.name);
            if wanted == coloured.map(|coloured| coloured.0) {
                continue;
            }
            let look = match wanted {
                Some(colour) => looks.of(&of.plain, colour, &mut materials),
                None => of.plain.clone(),
            };
            let mut drawn = commands.entity(part);
            match solid {
                Some(mut solid) => {
                    drawn.try_insert(MeshMaterial3d(stock.ghost_of(&look, &mut materials)));
                    solid.0 = look;
                }
                None => {
                    drawn.try_insert(MeshMaterial3d(look));
                }
            }
            match wanted {
                Some(colour) => drawn.try_insert(Coloured(colour)),
                None => drawn.try_remove::<Coloured>(),
            };
        }
    }
}

/// A tap, with the paintbrush: the part of yours under it is painted the
/// colour chosen (`customize`), which the bag is told ([`Bag::paint`]) and
/// what is drawn follows; or, with the cross in hand, has its paint taken
/// off ([`Bag::unpaint`]). The nearest of your parts the ray through the
/// finger meets: of things that are down and showing, and not a part drawn
/// see-through just now, which a tap goes through. A colour painted with
/// is noted, for the window to offer again.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn tap(
    mut customize: ResMut<Customize>,
    (pointer, took): (Res<EditPointer>, Res<crate::bubble::TookTap>),
    dialog: Res<DialogUp>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    mine: Query<(Entity, &Stood, &InheritedVisibility), (With<Mine>, Without<Ghost>)>,
    (things, children): (Query<(Entity, &Colours)>, Query<&Children>),
    parts: Query<&Part, Without<SeeThrough>>,
    mut ray_cast: MeshRayCast,
    online: Res<Online>,
    mut bag: ResMut<Bag>,
    mut asks: MessageWriter<Ask>,
) {
    // (A tap a bubble took, someone to talk to beside what you paint, is
    // the bubble's.)
    if !customize.painting() || dialog.0 || !pointer.released || pointer.lost || pointer.travel > editor::TAP_SLOP || took.0 {
        return;
    }
    let Ok((camera, place)) = cameras.single() else {
        return;
    };
    // The camera has no parent: this is where it is this frame.
    let Ok(ray) = camera.viewport_to_world(&GlobalTransform::from(*place), pointer.pos) else {
        return;
    };
    let mut reach: HashMap<Entity, u32> = HashMap::new();
    for (thing, stood, shown) in &mine {
        if !shown.get() {
            continue;
        }
        for part in parts_of(thing, &children, &things) {
            if parts.contains(part) {
                reach.insert(part, stood.id);
            }
        }
    }
    let within = |entity: Entity| reach.contains_key(&entity);
    let settings = MeshRayCastSettings::default().with_filter(&within);
    let Some(&(part, _)) = ray_cast.cast_ray(ray, &settings).first() else {
        return;
    };
    let (Some(&id), Ok(of)) = (reach.get(&part), parts.get(part)) else {
        return;
    };
    // The bag is watched for changes, and so is the edit mode: each left
    // alone where nothing would change.
    let painted = bag.paints.colour(id, &of.name);
    if customize.clearing() {
        if painted.is_some() {
            bag.unpaint(online.is(), &mut asks, id, &of.name);
        }
        return;
    }
    let colour = customize.colour();
    if painted != Some(colour) {
        bag.paint(online.is(), &mut asks, id, &of.name, colour);
    }
    if !customize.last_used() {
        customize.note_used();
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    #[test]
    fn paint_keeps_how_see_through_a_thing_was() {
        let glass = Color::srgba(0.9, 0.95, 1.0, 0.3);
        let blue = painted(glass, [0, 0, 255]).to_srgba();
        assert_eq!((blue.red, blue.green, blue.blue), (0.0, 0.0, 1.0));
        assert!((blue.alpha - 0.3).abs() < 1e-6);
        assert_eq!(painted(Color::WHITE, [255, 128, 0]).alpha(), 1.0);
    }

    /// A world with one thing in it, a bed of three meshes, two of them its
    /// blanket, and a door hung from it with a part of the same name: the
    /// bed, its parts, and the door's part; and the materials each began
    /// with.
    fn bed(world: &mut World) -> (Entity, [Entity; 3], Entity, [Handle<StandardMaterial>; 2]) {
        world.init_resource::<Looks>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.insert_resource(Stock::for_tests());
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let red = materials.add(StandardMaterial::from(Color::srgb(0.7, 0.1, 0.1)));
        let grey = materials.add(StandardMaterial::from(Color::srgb(0.6, 0.6, 0.6)));
        let bed = world.spawn(Colours::default()).id();
        let part = |world: &mut World, under: Entity, name: &str, plain: &Handle<StandardMaterial>| {
            world.spawn((ChildOf(under), Part::new(name, plain.clone()), MeshMaterial3d(plain.clone()))).id()
        };
        let blanket = part(world, bed, "blanket", &red);
        let pillow = part(world, bed, "blanket", &red);
        let frame = part(world, bed, "bedframe", &grey);
        let door = world.spawn((ChildOf(bed), Colours::default())).id();
        let leaf = part(world, door, "blanket", &red);
        (bed, [blanket, pillow, frame], leaf, [red, grey])
    }

    fn drawn_with(world: &World, part: Entity) -> Handle<StandardMaterial> {
        world.get::<MeshMaterial3d<StandardMaterial>>(part).expect("a material").0.clone()
    }

    fn colour_of(world: &World, part: Entity) -> Srgba {
        let look = drawn_with(world, part);
        world.resource::<Assets<StandardMaterial>>().get(&look).expect("the material").base_color.to_srgba()
    }

    #[test]
    fn a_part_painted_is_every_mesh_of_that_material_and_nothing_else() {
        let mut world = World::new();
        let (bed, [blanket, pillow, frame], leaf, [red, grey]) = bed(&mut world);
        world.run_system_once(colour_parts).expect("drawn");
        // Nothing painted: each as its model has it.
        assert_eq!(drawn_with(&world, blanket), red);
        // The blanket, blue: both its meshes, with one material between
        // them; the frame as it was; and the door, a thing of its own
        // though it hangs from the bed, as it was too.
        world.get_mut::<Colours>(bed).expect("colours").0 = vec![("blanket".into(), [0, 0, 255])];
        world.run_system_once(colour_parts).expect("drawn");
        let blue = colour_of(&world, blanket);
        assert_eq!((blue.red, blue.green, blue.blue), (0.0, 0.0, 1.0));
        assert_eq!(drawn_with(&world, blanket), drawn_with(&world, pillow));
        assert_ne!(drawn_with(&world, blanket), red);
        assert_eq!(drawn_with(&world, frame), grey);
        assert_eq!(drawn_with(&world, leaf), red);
        assert_eq!(world.get::<Coloured>(blanket), Some(&Coloured([0, 0, 255])));
        // The model's own material is as it was: another bed is still red.
        let own = world.resource::<Assets<StandardMaterial>>().get(&red).expect("red").base_color.to_srgba();
        assert!((own.red - 0.7).abs() < 1e-6);
        // Painted another colour, and then the frame too.
        world.get_mut::<Colours>(bed).expect("colours").0 = vec![("blanket".into(), [0, 255, 0]), ("bedframe".into(), [255, 255, 255])];
        world.run_system_once(colour_parts).expect("drawn");
        assert_eq!(colour_of(&world, pillow).green, 1.0);
        assert_eq!(colour_of(&world, frame).to_u8_array_no_alpha(), [255, 255, 255]);
        // The same colour again is the same material again, made once.
        let green = drawn_with(&world, blanket);
        world.get_mut::<Colours>(bed).expect("colours").0 = vec![("blanket".into(), [0, 0, 255])];
        world.run_system_once(colour_parts).expect("drawn");
        world.get_mut::<Colours>(bed).expect("colours").0 = vec![("blanket".into(), [0, 255, 0])];
        world.run_system_once(colour_parts).expect("drawn");
        assert_eq!(drawn_with(&world, blanket), green);
        // No longer painted (taken back and stood again): as its model has
        // it.
        world.get_mut::<Colours>(bed).expect("colours").0.clear();
        world.run_system_once(colour_parts).expect("drawn");
        assert_eq!(drawn_with(&world, blanket), red);
        assert_eq!(drawn_with(&world, frame), grey);
        assert!(world.get::<Coloured>(blanket).is_none());
    }

    /// The names of each mesh's bands, in order.
    fn names(named: &[Vec<(f32, String)>]) -> Vec<Vec<&str>> {
        named.iter().map(|bands| bands.iter().map(|(_, name)| name.as_str()).collect()).collect()
    }

    #[test]
    fn a_house_is_painted_a_floor_at_a_time() {
        // Hajun, 2026-11-25: "each floor need to be its own color/material
        // when painting". The Square House 2F as he modelled it: its walls
        // whole from the ground to the roof, the stairs in the walls'
        // material, the upper floor 5 m up, the roof and its gables.
        let v = Vec3::new;
        let floors = [(v(-6.0, 0.0, -6.0), v(6.0, 0.1, 6.0)), (v(-5.5, 5.0, -5.5), v(3.5, 5.1, 5.5))];
        let drawn: Vec<Drawn> = vec![
            ("plain_floor", 0.0, 0.1, Shape::Floor),
            ("plain_wall", 0.1, 10.1, Shape::Wall),
            ("plain_wall", 0.1, 5.1, Shape::Other),
            ("plain_floor", 5.0, 5.1, Shape::Floor),
            ("plain_roof", 9.61, 14.79, Shape::Other),
            ("plain_wall", 10.1, 14.1, Shape::Other),
        ];
        let named = parts(true, &floors, &drawn);
        assert_eq!(
            names(&named),
            [
                vec!["plain_floor"],
                vec!["plain_wall", "plain_wall@2"],
                vec!["plain_wall"],
                vec!["plain_floor@2"],
                vec!["plain_roof"],
                vec!["plain_wall@2"],
            ]
        );
        // The wall is cut at the top of the upper floor.
        assert_eq!(named[1].iter().map(|&(bottom, _)| bottom).collect::<Vec<_>>(), [0.1, 5.1]);
        // The Roof Terrace 3F: a wall to each floor, the upper stairs in a
        // copy of the walls' material, and the terrace's parapet. Of its
        // floors only the terrace is drawn here: the lowest floor its
        // material is on, it keeps the material's own name.
        let floors = [(v(-6.0, 0.0, -6.0), v(6.0, 0.1, 6.0)), (v(-5.5, 5.0, -5.5), v(5.5, 5.1, 5.5)), (v(-5.5, 10.0, -5.5), v(5.5, 10.1, 5.5))];
        let drawn: Vec<Drawn> = vec![
            ("plain_wall", 0.1, 5.1, Shape::Wall),
            ("plain_wall", 0.1, 5.1, Shape::Other),
            ("plain_wall", 5.1, 10.1, Shape::Wall),
            ("plain_wall.001", 5.1, 10.1, Shape::Other),
            ("plain_floor", 10.0, 10.1, Shape::Floor),
            ("plain_wall", 10.1, 11.1, Shape::Other),
        ];
        assert_eq!(
            names(&parts(true, &floors, &drawn)),
            [vec!["plain_wall"], vec!["plain_wall"], vec!["plain_wall@2"], vec!["plain_wall@2"], vec!["plain_floor"], vec!["plain_wall@3"]]
        );
        // A house of one floor is painted as it always was.
        let floors = [(v(-6.0, 0.0, -6.0), v(6.0, 0.1, 6.0))];
        let drawn: Vec<Drawn> = vec![
            ("plain_floor", 0.0, 0.1, Shape::Floor),
            ("plain_wall", 0.1, 5.1, Shape::Wall),
            ("plain_roof", 4.61, 9.79, Shape::Other),
            ("plain_wall", 5.1, 9.1, Shape::Other),
        ];
        assert_eq!(names(&parts(true, &floors, &drawn)), [vec!["plain_floor"], vec!["plain_wall"], vec!["plain_roof"], vec!["plain_wall"]]);
        // Anything but a house: every part its material, as its model names
        // it, a copy's ".001" and all.
        let drawn: Vec<Drawn> = vec![("blanket", 0.3, 0.7, Shape::Other), ("bedframe.001", 0.0, 0.5, Shape::Other), ("blanket", 0.9, 1.2, Shape::Other)];
        assert_eq!(names(&parts(false, &[], &drawn)), [vec!["blanket"], vec!["bedframe.001"], vec!["blanket"]]);
        assert_eq!((material_of("plain_wall.001"), material_of("plain_wall"), material_of("v1.5"), material_of(".001")), ("plain_wall", "plain_wall", "v1.5", ".001"));
    }

    #[test]
    fn every_floor_of_every_house_hajun_modelled_is_painted_by_itself() {
        use crate::build::{Mount, PIECES};
        use crate::island::glb;
        let words = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];
        let mut seen = 0;
        for def in PIECES.iter().filter(|def| def.mount == Mount::House) {
            // Each mesh as the game reads it (`super::piece_ready`): a wall,
            // a floor or anything else by the object it is a mesh of.
            let model = glb::read(&def.model());
            let mut floors = Vec::new();
            let mut meshes: Vec<(&str, &str, f32, f32, Shape)> = Vec::new();
            for part in &model {
                let Some(material) = &part.material else {
                    continue;
                };
                let corners = part.triangles.iter().flatten();
                let low = corners.clone().fold(Vec3::INFINITY, |low, &corner| low.min(corner));
                let high = corners.fold(Vec3::NEG_INFINITY, |high, &corner| high.max(corner));
                let shape = if part.node.starts_with("Wall") {
                    Shape::Wall
                } else if part.node.starts_with("Floor") {
                    floors.push((low, high));
                    Shape::Floor
                } else {
                    Shape::Other
                };
                meshes.push((part.node.as_str(), material.as_str(), low.y, high.y, shape));
            }
            let drawn: Vec<Drawn> = meshes.iter().map(|&(_, material, foot, top, shape)| (material, foot, top, shape)).collect();
            let named = parts(true, &floors, &drawn);
            let levels = Levels::of(&floors);
            // Every band of every mesh: what it is a band of, its material,
            // the floor it is on, and its name.
            let mut bands: Vec<(&str, &str, u32, &str)> = Vec::new();
            for (&(node, material, _, top, shape), mesh) in meshes.iter().zip(&named) {
                for (bottom, name) in mesh {
                    // A floor is on the floor it is the top of.
                    let storey = levels.storey(if shape == Shape::Floor { top } else { *bottom });
                    bands.push((node, material_of(material), storey, name.as_str()));
                }
            }
            // Two parts have one name if, and only if, they are of one
            // material on one floor.
            for &(node, material, storey, name) in &bands {
                assert!(roundtown_net::bag::part_named(name), "{}: {name:?}", def.kind);
                for &(other, other_material, other_storey, other_name) in &bands {
                    let alike = material == other_material && storey == other_storey;
                    assert_eq!(alike, name == other_name, "{}: {node} ({name}) and {other} ({other_name})", def.kind);
                }
            }
            // The lowest floor of each material keeps the material's own
            // name: the roof, and all of a house of one floor, as before.
            for &(node, material, storey, name) in &bands {
                let lowest = bands.iter().filter(|each| each.1 == material).map(|each| each.2).min().unwrap();
                assert_eq!(name == material, storey == lowest, "{}: {node} ({name}) on floor {storey}", def.kind);
                if node.starts_with("Roof") {
                    assert_eq!(name, "plain_roof", "{}", def.kind);
                }
            }
            // As many floors to paint as the house has floors.
            let floors_named: Vec<&str> = bands.iter().filter(|each| each.0.starts_with("Floor")).map(|each| each.3).collect();
            let mut distinct = floors_named.clone();
            distinct.sort_unstable();
            distinct.dedup();
            if let Some(count) = words.iter().position(|word| def.kind.ends_with(&format!("_{word}_floor"))) {
                assert_eq!(distinct.len(), count + 1, "{}: {floors_named:?}", def.kind);
                assert_eq!(levels.0.len(), count + 1, "{}", def.kind);
            }
            // Of every house of more than one floor, the walls of each floor
            // that has walls of its own are a part of their own: the Square
            // House 2F's whole walls each cut in two.
            if def.kind == "square_house_two_floor" {
                for (&(node, ..), mesh) in meshes.iter().zip(&named) {
                    if node.starts_with("Wall") {
                        assert_eq!(mesh.iter().map(|(_, name)| name.as_str()).collect::<Vec<_>>(), ["plain_wall", "plain_wall@2"], "{node}");
                    }
                }
            } else {
                for mesh in &named {
                    assert_eq!(mesh.len(), 1, "{}: a wall cut that runs up one floor", def.kind);
                }
            }
            seen += 1;
        }
        assert_eq!(seen, 21, "every house there is");
    }

    #[test]
    fn a_ghost_is_painted_as_what_it_is_put_down_as() {
        let mut world = World::new();
        let (bed, [blanket, ..], _, [red, _]) = bed(&mut world);
        // Picked up: drawn as a ghost of itself, its own material kept.
        world.entity_mut(blanket).insert(Solid(red.clone()));
        world.get_mut::<Colours>(bed).expect("colours").0 = vec![("blanket".into(), [0, 0, 255])];
        world.run_system_once(colour_parts).expect("drawn");
        // What it is put down as is the blue; what it is drawn with now is
        // neither that nor the red, but the ghost of the blue.
        let solid = world.get::<Solid>(blanket).expect("solid").0.clone();
        let materials = world.resource::<Assets<StandardMaterial>>();
        let blue = materials.get(&solid).expect("blue").base_color.to_srgba();
        assert_eq!((blue.red, blue.blue, blue.alpha), (0.0, 1.0, 1.0));
        let ghost = drawn_with(&world, blanket);
        assert!(ghost != solid && ghost != red);
        let ghost = materials.get(&ghost).expect("the ghost").base_color.to_srgba();
        assert!(ghost.alpha < 1.0 && ghost.blue == 1.0, "{ghost:?}");
    }
}
