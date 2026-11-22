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
//! `super::piece_ready`), and is painted together: a tap on any wall of
//! your house paints its walls.
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
