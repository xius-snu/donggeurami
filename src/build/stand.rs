//! What stands on the town's plots (since 2026-11-21; Hajun: "in the
//! inventory tab, when u click on a building item, (if the player is
//! standing on their own plot) it should close the panel and have the
//! building item in front of the character in the map similar to how the
//! building placing thing works in the house builder game with the rotate
//! buttons on the left and right and trash in the top, and check/apply in
//! the bottom. and when u place it in your plot, that item is gone from ur
//! inventory until u click that building in the map and trash it, then it
//! come back to ur inventory. also the placed inventory, after pressing the
//! check button, gets placed in ur plot and saved so each time u reopen the
//! app it stays there and other ppl can see that item in ur plot in that
//! exact location too").
//!
//! **Your own plot** ([`Home`]) is the one your seat in the town has
//! (`land::Plots`), as a square to build on (`Yard`). A building taken out
//! of your bag while you stand on it ([`Stand`], [`Home::may_stand`]) comes
//! up in front of you as a piece does in House Builder ([`bring`],
//! `super::bring_out`): a ghost with the edit menu on it, dragged and
//! turned by the same rules, kept inside the plot, the whole of it. The
//! tick puts it down, and it is out of your bag from then ([`report`],
//! `bag::Bag::put`); tapped again it is picked back up, to be moved, or,
//! by the bin, taken back into the bag ([`note_edits`], `bag::Bag::take`).
//! One house to a plot, and doors and windows go in its walls, as in House
//! Builder; a house taken back takes what was in its walls with it.
//!
//! **What stands is what the bag says stands** ([`keep_mine`]): each thing
//! by its number, where the bag has it, in the plot's own space
//! (`roundtown_net::bag::Standing`), so that it stands the same on
//! whichever plot your seat has, and is there as the app opens, out of the
//! bag kept on the device, before the server has said a word. Online the
//! server's word for the bag comes after each change and is what stands
//! from then: anything it would not have goes back into the bag.
//!
//! **Everyone else's** in your channel of the town stands on their plots
//! the same way ([`keep_theirs`]), from what the server says of each
//! (`bag::Theirs`) and the seat each has (`lobby::Channel`): drawn, stood
//! on and bumped into, and never to be tapped.
//!
//! All of it is part of the town's shape as House Builder's pieces are part
//! of their plot's (`super::reshape_plot`), laid over it beside the plots'
//! signs.
//!
//! **A thing of yours answers a tap only in edit mode, with the hand**
//! (since 2026-11-22, `customize`; Hajun: "with edit mode off, tapping
//! objects should do nothing"): [`hands_off`]. Bringing something out of
//! your bag turns edit mode on ("when u open invetory and tap an object to
//! place it, then edit mode gets toggled on until u turn it off"), and
//! leaving it, or taking up the brush, puts away whatever was in your hand
//! ([`leave`]). **What of each thing is painted** is hung on it as the bags
//! have it ([`keep_colours`], `paint::Colours`), for `paint` to draw.
//!
//! **A house picked up takes what stands in it along** ([`Rides`],
//! [`ride_along`]; Hajun, 2026-11-22: "everything inside the house obj
//! including the doors and windows should move together if u decide to move
//! the house"): its doors and windows are its own, in its walls, and the
//! furniture standing on its floors is carried where the house goes and
//! turned as it turns, in nobody's way meanwhile, and put down with it,
//! the bag told where each has come to. **Let go of by a tap somewhere
//! else** (`editor::Edit::Away`), a thing is put down where it is if it can
//! stand there (`super::edit_pieces`) and otherwise put back where it
//! stood ([`PutBack`], [`put_back`]), and a house with all that was in it.

use std::collections::HashSet;

use bevy::picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings};
use bevy::prelude::*;
use roundtown_net::{Ask, At};

use super::paint::Colours;
use super::{
    BuildSystems, Ghost, Misplaced, Mount, OnWall, PIECES, Piece, PieceDef, Reshape, Rooms, Solid, Stock, Wall, Yard,
    bring_out, piece_ready, unghost, wall_index, walls_of,
};
use crate::Player;
use crate::bag::{Bag, BagSystems, Standing, TheirPaints, Theirs};
use crate::customize::{self, Customize};
use crate::drive::Driving;
use crate::editor::{self, Edit, EditMenu, EditPointer, Editable, Touch, Untappable};
use crate::hud;
use crate::island::Venue;
use crate::land::{Deeds, Plot, Plots};
use crate::lobby::Seat;
use crate::net::Online;
use crate::ui::ui;

/// How near two places have to be to be the same place, in metres and in
/// radians: a thing put back down where it was picked up has not moved.
const SAME: f32 = 1e-3;
/// How long "It has to stand on your land." is up, in seconds.
const SAID_SECS: f32 = 2.5;

/// Your own plot of the town, while you are in the town and its plots have
/// been read: the square of it, whether you are standing on it, or within
/// a few steps of it (`customize::NEAR`), and whether a house of yours
/// stands there, and is down with its walls made.
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Home {
    pub yard: Option<Yard>,
    pub on_it: bool,
    pub near: bool,
    house: Option<bool>,
}

/// Why something cannot be taken out of your bag to stand on your plot just
/// now.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Why {
    /// You are not standing on your own plot.
    OffYourLand,
    /// A house stands there already.
    OneHouse,
    /// It goes in a wall, and there is no house down for it to go in.
    HouseFirst,
}

impl Home {
    /// A plot there is nothing known of but whether you are on it or near
    /// it.
    #[cfg(test)]
    pub(crate) fn for_tests(on_it: bool, near: bool) -> Self {
        Self {
            on_it,
            near,
            ..default()
        }
    }

    /// Whether a piece of `def` may be taken out of your bag to put down
    /// now, or why not.
    pub(crate) fn may_stand(&self, def: &PieceDef) -> Result<(), Why> {
        if self.yard.is_none() || !self.on_it {
            return Err(Why::OffYourLand);
        }
        match def.mount {
            Mount::House if self.house.is_some() => Err(Why::OneHouse),
            Mount::Door | Mount::Window | Mount::Hanging if self.house != Some(true) => Err(Why::HouseFirst),
            _ => Ok(()),
        }
    }
}

/// A building taken out of your bag, to be put down on your plot.
#[derive(Message, Clone, Copy)]
pub(crate) struct Stand(pub &'static PieceDef);

/// A piece on your own plot of the town, out of your bag.
#[derive(Component)]
pub(crate) struct Mine;

/// One of yours that stands, as far as the bag goes: its number there, and
/// where the bag has it.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(super) struct Stood {
    pub(super) id: u32,
    at: At,
}

/// One of your things that stands in your house, carried along while the
/// house is in your hand: which house, and where in the house's own space
/// it stands.
#[derive(Component, Clone, Copy)]
pub(super) struct Rides {
    house: Entity,
    at: Transform,
}

/// A thing of yours let go of where it cannot stand (`editor::Edit::Away`,
/// `super::edit_pieces`): it is to go back where it stood, or into the bag
/// if it never had.
#[derive(Message, Clone, Copy)]
pub(super) struct PutBack(pub Entity);

/// Saying what of each thing that stands is painted ([`keep_colours`]):
/// `paint` draws it after.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Painting;

/// One of yours that has been picked up, or just brought out: the bag is
/// told where it stands once it is down ([`report`]).
#[derive(Component)]
struct Lifted;

/// One of yours that is not where the bag has it, having been put back
/// down unasked: stood there again ([`keep_mine`]).
#[derive(Component)]
struct Displaced;

/// A piece on the plot of someone else in your channel: whose, its number
/// in their bag, where that has it, and which plot of the town it was
/// stood on.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
struct TheirPiece {
    who: u64,
    id: u32,
    at: At,
    plot: usize,
}

/// The line that says a piece has to stand on your land, while it is up.
#[derive(Component)]
struct Said;

/// How soon after a tap on a thing of yours another has to come, with edit
/// mode off, for it to be someone trying to move it, in seconds; and how
/// long they are told how.
const TRIES_WITHIN: f32 = 4.0;
const HINT_SECS: f32 = 3.5;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Home>()
        .add_message::<Stand>()
        .add_message::<PutBack>()
        .add_systems(
            Update,
            (
                (find_home, note_edits, bring, leave)
                    .chain()
                    .after(editor::EditSystems)
                    .after(BagSystems)
                    .after(crate::land::DeedSystems)
                    .before(BuildSystems),
                hands_off.before(editor::EditSystems),
                (put_back, ride_along, report, keep_mine, keep_theirs, keep_colours.in_set(Painting), say_misplaced)
                    .chain()
                    .after(BuildSystems),
                hint_at_edit_mode.after(crate::bubble::Taps),
            ),
        );
}

/// The plot `plot` as a square to build on.
pub(crate) fn yard_of(plot: &Plot) -> Yard {
    Yard {
        middle: plot.middle,
        ground: plot.ground,
        front: plot.front,
        half: plot.corners[0].distance(plot.middle) / std::f32::consts::SQRT_2,
    }
}

/// Where a thing that stands at this place and turn in `yard`'s own space
/// stands in the world.
pub(super) fn placed(yard: &Yard, x: f32, y: f32, z: f32, yaw: f32) -> Transform {
    yard.place() * Transform::from_xyz(x, y, z).with_rotation(Quat::from_rotation_y(yaw))
}

/// Where a piece standing at `place` in the world stands in `yard`'s own
/// space.
fn stood_at(yard: &Yard, place: &Transform) -> At {
    let plot = yard.place();
    let at = plot.compute_affine().inverse().transform_point3(place.translation);
    let turn = plot.rotation.inverse() * place.rotation;
    At::Ground {
        x: at.x,
        y: at.y,
        z: at.z,
        yaw: turn.to_euler(EulerRot::YXZ).0,
    }
}

/// Whether `a` and `b` are the same place, to a millimetre.
fn same_place(a: &At, b: &At) -> bool {
    let near = |a: f32, b: f32| (a - b).abs() < SAME;
    match (*a, *b) {
        (At::Ground { x, y, z, yaw }, At::Ground { x: x2, y: y2, z: z2, yaw: yaw2 }) => {
            let turned = (yaw - yaw2).rem_euclid(std::f32::consts::TAU);
            near(x, x2) && near(y, y2) && near(z, z2) && turned.min(std::f32::consts::TAU - turned) < SAME
        }
        (
            At::Wall { wall, along, up, face },
            At::Wall {
                wall: wall2,
                along: along2,
                up: up2,
                face: face2,
            },
        ) => wall == wall2 && face == face2 && near(along, along2) && near(up, up2),
        _ => false,
    }
}

/// Nothing of yours is picked up by a tap but in edit mode, with the hand
/// (`customize`; Hajun, 2026-11-22: "with edit mode off, tapping objects
/// should do nothing"): not with the brush, which paints what it taps
/// (`paint`), and never in your car, where a stray tap driving across your
/// plot would have a bed in your hand. Everything of yours there but a
/// house, which `super::untappable_from_inside` puts out of reach the same
/// way with the houses you stand in. Written only as it changes.
#[allow(clippy::type_complexity)]
fn hands_off(
    driving: Res<Driving>,
    customize: Res<Customize>,
    pieces: Query<(Entity, Has<Untappable>), (With<Mine>, Without<Rooms>)>,
    mut commands: Commands,
) {
    let out_of_reach = driving.on() || !customize.moving();
    for (piece, untappable) in &pieces {
        if out_of_reach && !untappable {
            commands.entity(piece).insert(Untappable);
        } else if !out_of_reach && untappable {
            commands.entity(piece).remove::<Untappable>();
        }
    }
}

/// Says which plot is yours, whether you stand on it, and whether your
/// house is on it ([`Home`]). Written only as it changes.
fn find_home(
    plots: Res<Plots>,
    me: Query<(&Transform, &Seat, &Venue), With<Player>>,
    houses: Query<(&Piece, Has<Ghost>, Has<Rooms>), (With<Mine>, With<Stood>)>,
    mut home: ResMut<Home>,
) {
    let mine = me.single().ok().filter(|(.., venue)| **venue == Venue::Town).and_then(|(at, seat, _)| {
        let yard = yard_of(plots.all.get(*plots.of_seat.get(seat.0)?)?);
        let at = at.translation.xz();
        Some((yard, yard.holds(at, 0.0), yard.holds(at, customize::NEAR)))
    });
    let house = houses
        .iter()
        .find(|(piece, ..)| piece.def.mount == Mount::House)
        .map(|(_, ghost, walled)| !ghost && walled);
    home.set_if_neq(Home {
        yard: mine.map(|(yard, ..)| yard),
        on_it: mine.is_some_and(|(_, on_it, _)| on_it),
        near: mine.is_some_and(|(.., near)| near),
        house,
    });
}

/// What the edit menu asks of one of your pieces that the bag has to hear
/// of: picked back up, it is marked, to be told of once it is down again
/// ([`report`]), and a house with it whatever of yours stands on its
/// floors, which rides along from here ([`Rides`]); thrown away, it is back
/// in the bag, and with a house, whatever was in its walls, which goes with
/// it. Before the menu's asks are done (`super::edit_pieces`), which takes
/// what is thrown away out of the world.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn note_edits(
    mut edits: MessageReader<Edit>,
    mine: Query<(&Piece, Option<&Stood>), With<Mine>>,
    houses: Query<(&Transform, &Rooms, &super::Bounds), With<Mine>>,
    furniture: Query<(Entity, &Piece, &Transform, Has<Ghost>), (With<Mine>, With<Stood>, Without<Rooms>)>,
    online: Res<Online>,
    mut bag: ResMut<Bag>,
    mut asks: MessageWriter<Ask>,
    mut commands: Commands,
) {
    for &edit in edits.read() {
        match edit {
            Edit::Opened(target) if mine.contains(target) => {
                commands.entity(target).insert(Lifted);
                let Ok((house, rooms, bounds)) = houses.get(target) else {
                    continue;
                };
                // A house: what stands on its floors goes where it goes.
                let into = house.compute_affine().inverse();
                for (thing, piece, place, ghost) in &furniture {
                    if ghost || piece.def.on_wall() || !rooms.hold(into.transform_point3(place.translation), bounds.high.y) {
                        continue;
                    }
                    let at = Transform::from_matrix(Mat4::from(into * place.compute_affine()));
                    commands.entity(thing).insert((Ghost, Lifted, Rides { house: target, at }));
                }
            }
            Edit::Trash(target) => {
                let Ok((piece, stood)) = mine.get(target) else {
                    continue;
                };
                if let Some(stood) = stood {
                    bag.take(online.is(), &mut asks, stood.id);
                }
                if piece.def.mount == Mount::House {
                    for (other, stood) in &mine {
                        if let Some(stood) = stood.filter(|_| other.def.on_wall()) {
                            bag.take(online.is(), &mut asks, stood.id);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Puts a ghost of yours away unasked: one just out of the bag goes back
/// into it, never having stood; one picked up to be moved is put down
/// again, and stood back where the bag has it ([`Displaced`]).
fn settle(
    ghost: Entity,
    stood: bool,
    children: &Query<&Children>,
    solids: &Query<&Solid>,
    reshape: &mut Reshape,
    commands: &mut Commands,
) {
    if !stood {
        commands.entity(ghost).despawn();
        return;
    }
    unghost(ghost, children, solids, commands);
    commands
        .entity(ghost)
        .remove::<(Ghost, super::Fresh, Lifted, Rides, editor::CannotStand)>()
        .insert(Displaced);
    reshape.0 = true;
}

/// Puts back a thing let go of where it cannot stand ([`PutBack`]): its
/// menu away, and it back where the bag has it, or into the bag if it never
/// stood ([`settle`]); and with a house, whatever was riding in it.
#[allow(clippy::type_complexity)]
fn put_back(
    mut asked: MessageReader<PutBack>,
    ghosts: Query<(Entity, Has<Stood>, Option<&Rides>), (With<Mine>, With<Ghost>)>,
    (children, solids): (Query<&Children>, Query<&Solid>),
    (mut reshape, mut menu): (ResMut<Reshape>, ResMut<EditMenu>),
    mut commands: Commands,
) {
    for &PutBack(target) in asked.read() {
        for (ghost, stood, rides) in &ghosts {
            if ghost == target || rides.is_some_and(|rides| rides.house == target) {
                menu.let_go_of(ghost);
                settle(ghost, stood, &children, &solids, &mut reshape, &mut commands);
            }
        }
    }
}

/// Carries what stands in a house along with it while the house is in your
/// hand ([`Rides`]): where it stood in the house, wherever the house now is
/// and whichever way it is turned. Once the house is down, so is what was
/// in it, where it has come to, and the bag is told of each ([`report`]);
/// and a house that has gone from your hand any other way, binned, leaves
/// what was in it where it stood. Only what has moved is written.
#[allow(clippy::type_complexity)]
fn ride_along(
    houses: Query<(&Transform, Has<Ghost>), (With<Mine>, With<Rooms>)>,
    mut riders: Query<(Entity, &Rides, &mut Transform), Without<Rooms>>,
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    for (rider, rides, mut place) in &mut riders {
        let Ok((house, in_hand)) = houses.get(rides.house) else {
            commands.entity(rider).remove::<(Ghost, Lifted, Rides)>().insert(Displaced);
            reshape.0 = true;
            continue;
        };
        let wanted = Transform::from_matrix(Mat4::from(house.compute_affine() * rides.at.compute_affine()));
        if *place != wanted {
            *place = wanted;
        }
        if !in_hand {
            commands.entity(rider).remove::<(Ghost, Rides)>();
            reshape.0 = true;
        }
    }
}

/// Brings a building out of your bag onto your plot, in front of you, as a
/// ghost with the edit menu open on it. Whatever was a ghost already is
/// put away first ([`settle`]): one thing is put down at a time. **And
/// edit mode is on from here**, with the hand, until it is turned off
/// (`customize`; Hajun: "when u open invetory and tap an object to place
/// it, then edit mode gets toggled on until u turn it off").
#[allow(clippy::too_many_arguments)]
fn bring(
    mut stands: MessageReader<Stand>,
    home: Res<Home>,
    stock: Res<Stock>,
    me: Query<&Transform, With<Player>>,
    mine: Query<(Entity, &Piece, Has<Stood>, Has<Ghost>), With<Mine>>,
    (children, solids): (Query<&Children>, Query<&Solid>),
    (mut menu, mut customize): (ResMut<EditMenu>, ResMut<Customize>),
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    for &Stand(def) in stands.read() {
        if home.may_stand(def).is_err() {
            continue;
        }
        let Ok(me) = me.single() else {
            continue;
        };
        for (ghost, _, stood, _) in mine.iter().filter(|(.., ghost)| *ghost) {
            settle(ghost, stood, &children, &solids, &mut reshape, &mut commands);
        }
        let house = mine
            .iter()
            .find(|(_, piece, stood, _)| *stood && piece.def.mount == Mount::House)
            .map(|(house, ..)| house);
        let tag = (Mine, Lifted, Colours::default());
        if bring_out(&mut commands, &stock, &mut menu, def, house, Venue::Town, me.translation, tag).is_some() && !customize.moving() {
            customize.begin();
        }
    }
}

/// Leaving the town, whatever of yours was a ghost is put away
/// ([`settle`]): the menu on it has gone (`editor::put_away`). And so it is
/// as the hand is put down with something still in it, edit mode left for
/// being no longer by your plot, or in your car (`customize`): its menu
/// goes with it. (The tick and the brush put what is in your hand down
/// where it is first, and are not done if that cannot be:
/// `customize::settle`.)
fn leave(
    me: Query<&Venue, (With<Player>, Changed<Venue>)>,
    customize: Res<Customize>,
    ghosts: Query<(Entity, Has<Stood>), (With<Mine>, With<Ghost>)>,
    (children, solids): (Query<&Children>, Query<&Solid>),
    (mut reshape, mut menu): (ResMut<Reshape>, ResMut<EditMenu>),
    mut commands: Commands,
) {
    let gone = me.single().is_ok_and(|&venue| venue != Venue::Town);
    let hand_down = customize.is_changed() && !customize.is_added() && !customize.moving();
    if !(gone || hand_down) {
        return;
    }
    for (ghost, stood) in &ghosts {
        menu.let_go_of(ghost);
        settle(ghost, stood, &children, &solids, &mut reshape, &mut commands);
    }
}

/// Tells the bag of each of your pieces that has just been put down: where
/// it stands, in the plot's own space, under its number, or under a new one
/// if it is just out of the bag. One the bag will not have stood goes back
/// into it; one put down where it was picked up is not spoken of.
#[allow(clippy::type_complexity)]
fn report(
    home: Res<Home>,
    online: Res<Online>,
    mut bag: ResMut<Bag>,
    mut lifted: Query<
        (Entity, &Piece, &Transform, Option<&OnWall>, Option<&mut Stood>),
        (With<Mine>, With<Lifted>, Without<Ghost>),
    >,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    mut asks: MessageWriter<Ask>,
    mut commands: Commands,
) {
    let Some(yard) = home.yard else {
        return;
    };
    for (entity, piece, place, on_wall, stood) in &mut lifted {
        let at = match (piece.def.on_wall(), on_wall) {
            (false, _) => stood_at(&yard, place),
            (true, Some(on)) => {
                let Some(wall) = wall_index(on.wall, &walls) else {
                    continue;
                };
                At::Wall {
                    wall,
                    along: on.along,
                    up: on.up,
                    face: on.face as i8,
                }
            }
            // Not on its wall yet.
            (true, None) => continue,
        };
        commands.entity(entity).remove::<Lifted>();
        match stood {
            Some(stood) if same_place(&stood.at, &at) => {}
            Some(mut stood) => {
                if bag.put(online.is(), &mut asks, stood.id, piece.def.kind, at) {
                    stood.at = at;
                } else {
                    commands.entity(entity).insert(Displaced);
                }
            }
            None => {
                let id = bag.has.next_id();
                if bag.put(online.is(), &mut asks, id, piece.def.kind, at) {
                    commands.entity(entity).insert(Stood { id, at });
                } else {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Stands a piece of `def` where `at` has it on the plot `yard`, with `tag`
/// on it: on the ground there, or in the wall of `house` that `at` names.
/// Nothing, yet, for one whose wall is not there to stand in; and nothing
/// for one kept as what it is not, a door on the ground.
fn raise(
    commands: &mut Commands,
    stock: &Stock,
    def: &'static PieceDef,
    at: &At,
    yard: &Yard,
    house: Option<Entity>,
    walls: &Query<(Entity, &Wall, &ChildOf)>,
    tag: impl Bundle,
) -> Option<Entity> {
    let mut piece = match (*at, def.on_wall()) {
        // Drawn only in the town, once it is known you are there
        // (`island::show_where_i_am`).
        (At::Ground { x, y, z, yaw }, false) => commands.spawn((Venue::Town, placed(yard, x, y, z, yaw), Visibility::Hidden)),
        (At::Wall { wall, along, up, face }, true) => {
            let house = house?;
            let wall = *walls_of(house, walls).get(usize::from(wall))?;
            let (_, of, _) = walls.get(wall).ok()?;
            let on = OnWall {
                wall,
                along,
                up,
                face: f32::from(face),
            };
            commands.spawn((ChildOf(house), of.place(def.holed(), on), on, Visibility::Inherited))
        }
        _ => return None,
    };
    piece.insert((Piece { def }, tag, Colours::default(), WorldAssetRoot(stock.scene(def))));
    piece.observe(piece_ready);
    Some(piece.id())
}

/// Stands a piece that is in the world already where `at` has it: on the
/// ground of `yard`, or in the wall of `house` that `at` names. Whether it
/// could be: not while its wall is not there.
#[allow(clippy::too_many_arguments)]
fn set_down(
    commands: &mut Commands,
    entity: Entity,
    def: &PieceDef,
    at: &At,
    yard: &Yard,
    house: Option<Entity>,
    walls: &Query<(Entity, &Wall, &ChildOf)>,
    place: &mut Mut<Transform>,
) -> bool {
    match *at {
        At::Ground { x, y, z, yaw } => {
            place.set_if_neq(placed(yard, x, y, z, yaw));
            true
        }
        At::Wall { wall, along, up, face } => {
            let found = house.and_then(|house| {
                let wall = *walls_of(house, walls).get(usize::from(wall))?;
                let (_, of, _) = walls.get(wall).ok()?;
                Some((wall, of))
            });
            let Some((wall, of)) = found else {
                return false;
            };
            let on = OnWall {
                wall,
                along,
                up,
                face: f32::from(face),
            };
            place.set_if_neq(of.place(def.holed(), on));
            commands.entity(entity).insert(on);
            true
        }
    }
}

/// The piece that goes by `kind`, if this app has one.
fn def_of(kind: &str) -> Option<&'static PieceDef> {
    PIECES.iter().find(|def| def.kind == kind)
}

/// What a tap on one of your own pieces is tested against until its model
/// has loaded and says how big it is, and how its menu goes.
fn tappable(def: &PieceDef) -> Editable {
    Editable {
        touch: Touch::Block {
            low: Vec3::splat(-0.5),
            high: Vec3::splat(0.5),
        },
        ring: 0.0,
        turns: !def.on_wall(),
        sticky: true,
    }
}

/// Keeps what stands on your plot as the bag has it: each thing the bag
/// says stands there stands, where it says, and nothing else of yours does,
/// but what is a ghost just now, which the menu has. As the bag changes,
/// as your plot does, and for as long as something waits for its wall (a
/// door's house has to have loaded before the door can be stood in it).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn keep_mine(
    bag: Res<Bag>,
    home: Res<Home>,
    stock: Res<Stock>,
    mut mine: Query<(Entity, &Piece, &mut Stood, &mut Transform, Has<Ghost>, Has<Displaced>), With<Mine>>,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    (mut waiting, mut on): (Local<bool>, Local<Option<Yard>>),
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    let Some(yard) = home.yard else {
        return;
    };
    let moved = *on != Some(yard);
    let displaced = mine.iter().any(|(.., displaced)| displaced);
    if !(bag.is_changed() || moved || displaced || *waiting) {
        return;
    }
    *on = Some(yard);
    *waiting = false;
    let stands = &bag.has.stands;
    let wanted = |id: u32, kind: &str| stands.iter().find(|stood| stood.id == id && stood.kind == kind);
    let house = mine
        .iter()
        .find(|(_, piece, stood, ..)| piece.def.mount == Mount::House && wanted(stood.id, piece.def.kind).is_some())
        .map(|(house, ..)| house);
    // What stands and should not goes; what stands in the wrong place is
    // stood in the right one.
    let mut standing: Vec<u32> = Vec::new();
    for (entity, piece, mut stood, mut place, ghost, displaced) in &mut mine {
        let Some(wanted) = wanted(stood.id, piece.def.kind) else {
            commands.entity(entity).despawn();
            continue;
        };
        standing.push(stood.id);
        if ghost {
            continue;
        }
        let elsewhere = stood.at != wanted.at;
        if !(elsewhere || displaced || (moved && !piece.def.on_wall())) {
            continue;
        }
        if set_down(&mut commands, entity, piece.def, &wanted.at, &yard, house, &walls, &mut place) {
            if elsewhere {
                stood.at = wanted.at;
            }
            commands.entity(entity).remove::<Displaced>();
            reshape.0 = true;
        } else {
            *waiting = true;
        }
    }
    // What should stand and does not is stood.
    for stood in stands.iter().filter(|stood| !standing.contains(&stood.id)) {
        let Some(def) = def_of(&stood.kind) else {
            continue;
        };
        let tag = (Mine, Stood { id: stood.id, at: stood.at }, tappable(def));
        if raise(&mut commands, &stock, def, &stood.at, &yard, house, &walls, tag).is_none() && def.on_wall() {
            *waiting = true;
        }
    }
}

/// Keeps what stands on every plot of the town but your own as it is
/// said to: for each plot that is someone's (`land::Deeds`: whoever else
/// is in your channel, and the land that fills the rest), what stands on
/// their land ([`Theirs`], the server's word online and each made-up
/// owner's own offline), on that plot. As any of that changes, and for as
/// long as something waits for its wall.
#[allow(clippy::too_many_arguments)]
fn keep_theirs(
    theirs: Res<Theirs>,
    deeds: Res<Deeds>,
    plots: Res<Plots>,
    stock: Res<Stock>,
    mut pieces: Query<(Entity, &Piece, &mut TheirPiece, &mut Transform)>,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    mut waiting: Local<bool>,
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    if !(theirs.is_changed() || deeds.is_changed() || plots.is_changed() || *waiting) {
        return;
    }
    *waiting = false;
    // Whose plot is which: every one but yours, which no deed is for.
    let plot_of = |who: u64| -> Option<usize> {
        plots.of_seat.get(deeds.plot_of(who)?).copied().filter(|&plot| plot < plots.all.len())
    };
    let wanted = |who: u64, id: u32, kind: &str| -> Option<(&Standing, usize)> {
        let plot = plot_of(who)?;
        let stood = theirs.0.get(&who)?.iter().find(|stood| stood.id == id && stood.kind == kind)?;
        Some((stood, plot))
    };
    let houses: Vec<(u64, Entity)> = pieces
        .iter()
        .filter(|(_, piece, ..)| piece.def.mount == Mount::House)
        .map(|(house, _, theirs, _)| (theirs.who, house))
        .collect();
    let house_of = |who: u64| houses.iter().find(|(whose, _)| *whose == who).map(|&(_, house)| house);
    let mut standing: Vec<(u64, u32)> = Vec::new();
    for (entity, piece, mut stood, mut place) in &mut pieces {
        let Some((wanted, plot)) = wanted(stood.who, stood.id, piece.def.kind) else {
            // A house takes what is in its walls with it: those are gone
            // already by the time their own turn comes.
            commands.entity(entity).try_despawn();
            continue;
        };
        standing.push((stood.who, stood.id));
        if stood.at == wanted.at && stood.plot == plot {
            continue;
        }
        let yard = yard_of(&plots.all[plot]);
        if set_down(&mut commands, entity, piece.def, &wanted.at, &yard, house_of(stood.who), &walls, &mut place) {
            (stood.at, stood.plot) = (wanted.at, plot);
            reshape.0 = true;
        } else {
            *waiting = true;
        }
    }
    for (&who, stands) in &theirs.0 {
        let Some(plot) = plot_of(who) else {
            continue;
        };
        let yard = yard_of(&plots.all[plot]);
        for stood in stands.iter().filter(|stood| !standing.contains(&(who, stood.id))) {
            let Some(def) = def_of(&stood.kind) else {
                continue;
            };
            let tag = TheirPiece {
                who,
                id: stood.id,
                at: stood.at,
                plot,
            };
            if raise(&mut commands, &stock, def, &stood.at, &yard, house_of(who), &walls, tag).is_none() && def.on_wall() {
                *waiting = true;
            }
        }
    }
}

/// Hangs on each thing that stands what of it is painted, as the bags have
/// it (`paint::Colours`): yours out of your own bag, by its number there;
/// everyone else's as the server said of their plot. As either changes,
/// and as a thing is stood. Only what differs is written, and `paint`
/// draws whatever is.
#[allow(clippy::type_complexity)]
fn keep_colours(
    bag: Res<Bag>,
    their_paints: Res<TheirPaints>,
    stood: Query<(), Or<(Added<Stood>, Added<TheirPiece>)>>,
    mut mine: Query<(&Stood, &mut Colours), (With<Mine>, Without<TheirPiece>)>,
    mut theirs: Query<(&TheirPiece, &mut Colours), Without<Mine>>,
) {
    if !(bag.is_changed() || their_paints.is_changed() || !stood.is_empty()) {
        return;
    }
    for (stood, mut colours) in &mut mine {
        let wanted = Colours::of(bag.paints.of(stood.id));
        if *colours != wanted {
            *colours = wanted;
        }
    }
    for (piece, mut colours) in &mut theirs {
        let painted = their_paints.0.get(&piece.who).map(Vec::as_slice).unwrap_or_default();
        let wanted = Colours::of(painted.iter().filter(|paint| paint.id == piece.id).map(|paint| (paint.part.as_str(), paint.colour)));
        if *colours != wanted {
            *colours = wanted;
        }
    }
}

/// Whether a tap on a thing of yours with edit mode off, `now`, is someone
/// trying to move it: the second within [`TRIES_WITHIN`] of the one before
/// (`last`), which is then forgotten, so that it takes two more to be told
/// again.
fn tried_again(last: &mut Option<f32>, now: f32) -> bool {
    match last.take() {
        Some(then) if now - then <= TRIES_WITHIN => true,
        _ => {
            *last = Some(now);
            false
        }
    }
}

/// Whether the pointer has come up this frame from a tap on the world that
/// is nobody else's: not a drag, which looks round; not a tap a bubble
/// took; and not with a window up, which every press is then for.
fn tapped(pointer: &EditPointer, took: bool, dialog: bool) -> bool {
    pointer.released && !pointer.lost && pointer.travel <= editor::TAP_SLOP && !took && !dialog
}

/// Whether a tap at a thing of yours is someone's who may not know that
/// edit mode is off: it is off, and they stand on their plot, where the
/// button that turns it on is, and are not in their car, where nothing of
/// theirs answers a tap whatever the mode ([`hands_off`]).
fn at_a_loss(customize: &Customize, home: &Home, driving: bool) -> bool {
    !customize.on() && home.on_it && !driving
}

/// Whether a tap at `at` on the screen is on a thing of yours: the ray
/// through it meets a mesh of one that is down and showing. One ray, cast
/// against yours alone.
fn meets_mine(
    at: Vec2,
    cameras: &Query<(&Camera, &Transform), With<crate::ThirdPersonCamera>>,
    mine: &Query<(Entity, &InheritedVisibility), (With<Mine>, Without<Ghost>)>,
    children: &Query<&Children>,
    ray_cast: &mut MeshRayCast,
) -> bool {
    let Ok((camera, place)) = cameras.single() else {
        return false;
    };
    // The camera has no parent: this is where it is this frame.
    let Ok(ray) = camera.viewport_to_world(&GlobalTransform::from(*place), at) else {
        return false;
    };
    let parts: HashSet<Entity> = mine
        .iter()
        .filter(|(_, shown)| shown.get())
        .flat_map(|(thing, _)| children.iter_descendants(thing))
        .collect();
    let within = |entity: Entity| parts.contains(&entity);
    let settings = MeshRayCastSettings::default().with_filter(&within);
    !ray_cast.cast_ray(ray, &settings).is_empty()
}

/// Whether the tap `now` at a thing of yours with edit mode off is answered
/// in words as well as by the button: while the words are said at all
/// (`on`, which is `customize::HINTS`), the first few times ever
/// (`Customize::may_hint`), and at the second tap of two
/// ([`tried_again`]).
fn says_how(on: bool, customize: &Customize, last: &mut Option<f32>, now: f32) -> bool {
    on && customize.may_hint() && tried_again(last, now)
}

/// Shows someone who taps at a thing of theirs with edit mode off where
/// edit mode is: the button that turns it on, by the left edge, beats
/// (`customize::Pointed`), every time, uncounted and without a word
/// (since 2026-11-23; Hajun: "she was also confused why she couldnt move
/// the object by tapping, but its because she was not in house edit mode").
/// A tap is on a thing of yours if the ray through the finger meets one of
/// its meshes ([`meets_mine`]); a tap a bubble took is not one, nor a tap
/// in your car ([`tapped`], [`at_a_loss`]).
///
/// **It told them in words until then**, the first few times ever
/// (`Customize::may_hint`), at the second tap of two ([`says_how`]; Hajun,
/// 2026-11-22: "if they repeatedly tap on an object without being on house
/// edit mode, give small text like "Press the edit house mode to move
/// objects" sth like that as a hint. cuz i think some ppl after placing an
/// object and they try to press the object to move again, but dont know
/// they have to be in house edit mode to do it"). The words are turned off
/// (`customize::HINTS`; "i think the tip notification to go into house edit
/// mode is not the real fix - should delete that and use another way"), and
/// still here: the same tap would say them, the one ray cast for both.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn hint_at_edit_mode(
    time: Res<Time>,
    (pointer, took, dialog): (Res<EditPointer>, Res<crate::bubble::TookTap>, Res<hud::DialogUp>),
    (home, driving): (Res<Home>, Res<Driving>),
    (mut customize, mut pointed): (ResMut<Customize>, ResMut<customize::Pointed>),
    cameras: Query<(&Camera, &Transform), With<crate::ThirdPersonCamera>>,
    mine: Query<(Entity, &InheritedVisibility), (With<Mine>, Without<Ghost>)>,
    children: Query<&Children>,
    mut ray_cast: MeshRayCast,
    said: Query<Entity, With<Said>>,
    mut last: Local<Option<f32>>,
    mut commands: Commands,
) {
    if !tapped(&pointer, took.0, dialog.0) || !at_a_loss(&customize, &home, driving.on()) {
        return;
    }
    if !meets_mine(pointer.pos, &cameras, &mine, &children, &mut ray_cast) {
        return;
    }
    pointed.begin();
    if !says_how(customize::HINTS, &customize, &mut last, time.elapsed_secs()) {
        return;
    }
    for line in &said {
        commands.entity(line).despawn();
    }
    hud::announce(&mut commands, None, &ui().words.bag.edit_hint, hud::CAPTION, HINT_SECS).insert(Said);
    customize.note_hint();
}

/// Says why a piece asked to be put down has not been
/// (`super::Misplaced`): across the screen, one line at a time.
fn say_misplaced(mut misplaced: MessageReader<Misplaced>, said: Query<Entity, With<Said>>, mut commands: Commands) {
    let Some(&why) = misplaced.read().last() else {
        return;
    };
    for line in &said {
        commands.entity(line).despawn();
    }
    let ui = ui();
    let words = if why.in_the_way { &ui.words.bag.in_the_way } else { &ui.words.bag.off_your_land };
    hud::announce(&mut commands, None, words, hud::CAPTION, SAID_SECS).insert(Said);
}

#[cfg(test)]
mod tests {
    use super::super::Bounds;
    use super::*;

    fn yard(front: Vec2) -> Yard {
        Yard {
            middle: Vec2::new(25.0, 90.0),
            ground: 0.27,
            front,
            half: 12.0,
        }
    }

    fn def(kind: &str) -> &'static PieceDef {
        def_of(kind).unwrap()
    }

    #[test]
    fn a_plot_is_a_yard_as_wide_as_the_plot() {
        let middle = Vec2::new(25.0, 90.0);
        // A plot 24 m square, turned a little.
        let turn = Vec2::from_angle(0.3);
        let corners = [(-12.0, -12.0), (12.0, -12.0), (12.0, 12.0), (-12.0, 12.0)].map(|(x, z)| middle + turn.rotate(Vec2::new(x, z)));
        let plot = Plot {
            middle,
            ground: 0.27,
            corners,
            ring: 0,
            front: turn.rotate(Vec2::NEG_Y),
            street: false,
            sign: middle,
        };
        let yard = yard_of(&plot);
        assert!((yard.half - 12.0).abs() < 1e-4, "{}", yard.half);
        // Its corners are on it, and just past them is not.
        for corner in corners {
            assert!(yard.holds(corner, 0.01), "{corner}");
            assert!(!yard.holds(middle + (corner - middle) * 1.05, 0.0), "{corner}");
        }
        assert!(yard.holds(middle, 0.0));
    }

    #[test]
    fn what_stands_in_the_plot_s_own_space_stands_the_same_on_any_plot() {
        // A thing three metres to the right of someone looking out over
        // the front, and five in front of the middle, turned a quarter.
        let (x, y, z, yaw) = (3.0, 0.0, -5.0, std::f32::consts::FRAC_PI_2);
        for front in [Vec2::NEG_Y, Vec2::X, Vec2::new(0.6, 0.8), Vec2::new(-0.8, -0.6)] {
            let yard = yard(front);
            let place = placed(&yard, x, y, z, yaw);
            let right = Vec2::new(-front.y, front.x);
            let wanted = yard.middle + front * 5.0 + right * 3.0;
            assert!(place.translation.xz().distance(wanted) < 1e-4, "{front}: {} for {wanted}", place.translation);
            assert!((place.translation.y - yard.ground).abs() < 1e-6);
            // And back again.
            let back = stood_at(&yard, &place);
            assert!(same_place(&back, &At::Ground { x, y, z, yaw }), "{front}: {back:?}");
        }
        // Turned by nothing, its front (the loader's -Z) looks out over the
        // plot's front.
        let yard = yard(Vec2::new(0.6, 0.8));
        let faces = (placed(&yard, 0.0, 0.0, 0.0, 0.0).rotation * Vec3::NEG_Z).xz();
        assert!(faces.distance(yard.front) < 1e-5, "{faces}");
    }

    #[test]
    fn a_piece_is_on_the_plot_only_with_the_whole_of_it_there() {
        let yard = yard(Vec2::NEG_Y);
        // A desk two metres by one about its own middle.
        let bounds = Bounds {
            low: Vec3::new(-1.0, 0.0, -0.5),
            high: Vec3::new(1.0, 1.5, 0.5),
        };
        let at = |x: f32, z: f32| Vec3::new(yard.middle.x + x, yard.ground, yard.middle.y + z);
        let square = Quat::IDENTITY;
        assert!(yard.holds_piece(at(0.0, 0.0), bounds, square));
        // Flush against the edge is on it; a centimetre over is not.
        assert!(yard.holds_piece(at(11.0, 11.5), bounds, square));
        assert!(!yard.holds_piece(at(11.01, 0.0), bounds, square));
        assert!(!yard.holds_piece(at(0.0, -11.51), bounds, square));
        // Turned a quarter, it is the other way about.
        let quarter = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        assert!(yard.holds_piece(at(11.5, 11.0), bounds, quarter));
        assert!(!yard.holds_piece(at(11.0, 11.5), bounds, quarter));
        // Pulled in, it is as little further in as it takes.
        let pulled = yard.pull_in(at(30.0, 3.0), bounds, square);
        assert!(pulled.distance(at(11.0, 3.0)) < 1e-4, "{pulled}");
        assert!(yard.holds_piece(pulled, bounds, square));
        assert_eq!(yard.pull_in(at(2.0, 3.0), bounds, square), at(2.0, 3.0));
        // Turned an eighth, its corners reach further.
        let eighth = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
        let pulled = yard.pull_in(at(30.0, -30.0), bounds, eighth);
        assert!(yard.holds_piece(pulled, bounds, eighth));
        assert!(!yard.holds_piece(pulled + Vec3::new(0.05, 0.0, 0.0), bounds, eighth));
    }

    #[test]
    fn a_thing_comes_out_of_the_bag_only_onto_your_own_plot_one_house_to_it_and_doors_into_a_house() {
        let on_it = Home {
            yard: Some(yard(Vec2::NEG_Y)),
            on_it: true,
            near: true,
            house: None,
        };
        let (lamp, house, door, window) = (def("streetlamp"), def("square_house"), def("classic_door"), def("wide_window"));
        // Off your plot, and anywhere but the town, nothing.
        for home in [Home::default(), Home { on_it: false, ..on_it }] {
            for def in [lamp, house, door] {
                assert_eq!(home.may_stand(def), Err(Why::OffYourLand));
            }
        }
        // On it with no house: furniture and a house, and no door.
        assert_eq!(on_it.may_stand(lamp), Ok(()));
        assert_eq!(on_it.may_stand(house), Ok(()));
        assert_eq!(on_it.may_stand(door), Err(Why::HouseFirst));
        assert_eq!(on_it.may_stand(window), Err(Why::HouseFirst));
        // With the house picked up, or its walls not made yet: no second
        // house, and no door yet.
        let lifted = Home { house: Some(false), ..on_it };
        assert_eq!(lifted.may_stand(house), Err(Why::OneHouse));
        assert_eq!(lifted.may_stand(door), Err(Why::HouseFirst));
        assert_eq!(lifted.may_stand(lamp), Ok(()));
        // With it down: doors and windows, and still no second house.
        let housed = Home { house: Some(true), ..on_it };
        assert_eq!(housed.may_stand(door), Ok(()));
        assert_eq!(housed.may_stand(window), Ok(()));
        assert_eq!(housed.may_stand(house), Err(Why::OneHouse));
    }

    #[test]
    fn a_house_picked_up_takes_what_stands_in_it_along_and_puts_it_down_where_it_comes_to() {
        use std::f32::consts::FRAC_PI_2;

        use bevy::ecs::system::RunSystemOnce;

        use super::super::Rooms;
        let mut world = World::new();
        world.init_resource::<Messages<Edit>>();
        world.init_resource::<Messages<Ask>>();
        world.init_resource::<Bag>();
        world.init_resource::<Reshape>();
        world.insert_resource(Online::default());
        let here = At::Ground { x: 0.0, y: 0.0, z: 0.0, yaw: 0.0 };
        let stood = |id: u32| Stood { id, at: here };
        // A house 8 by 6 with one floor, a bed standing on that floor, and a
        // lamp out on the plot.
        let bounds = Bounds {
            low: Vec3::new(-4.0, 0.0, -3.0),
            high: Vec3::new(4.0, 5.0, 3.0),
        };
        let floor = Rooms(vec![(Vec3::new(-4.0, 0.0, -3.0), Vec3::new(4.0, 0.1, 3.0))]);
        let house = world
            .spawn((Mine, Piece { def: def("square_house") }, stood(1), Transform::from_xyz(10.0, 0.0, 10.0), floor, bounds))
            .id();
        let bed = world.spawn((Mine, Piece { def: def("singlebed") }, stood(2), Transform::from_xyz(12.0, 0.1, 11.0))).id();
        let lamp = world.spawn((Mine, Piece { def: def("streetlamp") }, stood(3), Transform::from_xyz(20.0, 0.0, 10.0))).id();
        let pick_up = |world: &mut World| {
            world.write_message(Edit::Opened(house));
            world.run_system_once(note_edits).unwrap();
            world.resource_mut::<Messages<Edit>>().clear();
            // (`super::edit_pieces` makes the house itself a ghost.)
            world.entity_mut(house).insert(Ghost);
        };
        // Picked up: the bed rides in it from here, in nobody's way and to
        // be told of to the bag once it is down; the lamp stays where it is.
        pick_up(&mut world);
        assert!(world.get::<Lifted>(house).is_some());
        assert!(world.get::<Rides>(bed).is_some() && world.get::<Ghost>(bed).is_some() && world.get::<Lifted>(bed).is_some());
        assert!(world.get::<Rides>(lamp).is_none() && world.get::<Ghost>(lamp).is_none());
        // The house five metres on and turned a quarter: the bed where it
        // stood in the house, two metres to its right and one behind its
        // middle, turned with it (Hajun: "everything inside the house obj
        // ... should move together if u decide to move the house").
        *world.get_mut::<Transform>(house).unwrap() = Transform::from_xyz(15.0, 0.0, 10.0).with_rotation(Quat::from_rotation_y(FRAC_PI_2));
        world.run_system_once(ride_along).unwrap();
        let rode = *world.get::<Transform>(bed).unwrap();
        assert!(rode.translation.distance(Vec3::new(16.0, 0.1, 8.0)) < 1e-4, "{}", rode.translation);
        assert!(rode.rotation.angle_between(Quat::from_rotation_y(FRAC_PI_2)) < 1e-2);
        assert!(world.get::<Ghost>(bed).is_some());
        assert_eq!(world.get::<Transform>(lamp).unwrap().translation, Vec3::new(20.0, 0.0, 10.0));
        // Put down: so is the bed, there, and the bag is to hear of it.
        world.entity_mut(house).remove::<Ghost>();
        world.resource_mut::<Reshape>().0 = false;
        world.run_system_once(ride_along).unwrap();
        assert!(world.get::<Rides>(bed).is_none() && world.get::<Ghost>(bed).is_none());
        assert!(world.get::<Lifted>(bed).is_some());
        assert!(world.resource::<Reshape>().0);
        assert!(world.get::<Transform>(bed).unwrap().translation.distance(Vec3::new(16.0, 0.1, 8.0)) < 1e-4);
        // Picked up again and binned: the bed is left where the bag has it.
        world.entity_mut(bed).remove::<Lifted>();
        pick_up(&mut world);
        assert!(world.get::<Rides>(bed).is_some());
        world.entity_mut(house).despawn();
        world.run_system_once(ride_along).unwrap();
        assert!(world.get::<Rides>(bed).is_none() && world.get::<Ghost>(bed).is_none() && world.get::<Lifted>(bed).is_none());
        assert!(world.get::<Displaced>(bed).is_some());
    }

    #[test]
    fn a_thing_let_go_of_where_it_cannot_stand_goes_back_and_a_house_with_all_that_rode_in_it() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.init_resource::<Messages<PutBack>>();
        world.init_resource::<Reshape>();
        world.init_resource::<EditMenu>();
        let here = At::Ground { x: 0.0, y: 0.0, z: 0.0, yaw: 0.0 };
        // A house in your hand with a bed riding in it, and a lamp that is
        // down.
        let house = world.spawn((Mine, Ghost, Lifted, Stood { id: 1, at: here })).id();
        let rides = Rides {
            house,
            at: Transform::IDENTITY,
        };
        let bed = world.spawn((Mine, Ghost, Lifted, Stood { id: 2, at: here }, rides)).id();
        let lamp = world.spawn((Mine, Stood { id: 3, at: here })).id();
        world.resource_mut::<EditMenu>().open_on(house);
        world.write_message(PutBack(house));
        world.run_system_once(put_back).unwrap();
        world.resource_mut::<Messages<PutBack>>().clear();
        // Both back where the bag has them, to be stood there again, and
        // the menu away; the lamp as it was.
        for thing in [house, bed] {
            assert!(world.get::<Ghost>(thing).is_none() && world.get::<Lifted>(thing).is_none(), "{thing}");
            assert!(world.get::<Displaced>(thing).is_some(), "{thing}");
        }
        assert!(world.get::<Rides>(bed).is_none());
        assert!(!world.resource::<EditMenu>().is_open());
        assert!(world.get::<Displaced>(lamp).is_none());
        // One just out of the bag, which never stood: back into the bag.
        let fresh = world.spawn((Mine, Ghost, Lifted)).id();
        world.write_message(PutBack(fresh));
        world.run_system_once(put_back).unwrap();
        assert!(world.get_entity(fresh).is_err());
    }

    #[test]
    fn two_taps_close_together_are_someone_trying_to_move_a_thing() {
        let mut last = None;
        // One tap is a tap; a second soon after is a try, and told of; and
        // it takes two more to be told again.
        assert!(!tried_again(&mut last, 10.0));
        assert!(tried_again(&mut last, 11.5));
        assert!(!tried_again(&mut last, 12.0));
        assert!(tried_again(&mut last, 13.0));
        // Long after the one before, a tap is the first again.
        assert!(!tried_again(&mut last, 20.0));
        assert!(!tried_again(&mut last, 20.0 + TRIES_WITHIN + 0.1));
        assert!(tried_again(&mut last, 25.0));
    }

    #[test]
    fn a_tap_at_a_thing_of_yours_points_at_the_button_only_with_edit_mode_off_on_your_plot_and_out_of_your_car() {
        // A finger that came up this frame, having hardly moved: a tap.
        let up = EditPointer {
            released: true,
            travel: 3.0,
            ..default()
        };
        assert!(tapped(&up, false, false));
        // A bubble took it; a window is up; it was dragged, to look round;
        // it has not come up; or there is no pointer at all: none of them.
        assert!(!tapped(&up, true, false));
        assert!(!tapped(&up, false, true));
        let dragged = EditPointer {
            released: true,
            travel: editor::TAP_SLOP + 1.0,
            ..default()
        };
        assert!(!tapped(&dragged, false, false));
        assert!(!tapped(&EditPointer { held: true, ..default() }, false, false));
        let lost = EditPointer {
            released: true,
            lost: true,
            ..default()
        };
        assert!(!tapped(&lost, false, false));
        // With edit mode off, on your plot, out of your car: someone who
        // may not know it is off (Hajun: "she couldnt move the object by
        // tapping, but its because she was not in house edit mode").
        let off = Customize::default();
        let (on_it, beside_it) = (Home::for_tests(true, true), Home::for_tests(false, true));
        assert!(at_a_loss(&off, &on_it, false));
        // In your car nothing answers a tap whatever the mode; and a step
        // off your plot the button is not there to point at.
        assert!(!at_a_loss(&off, &on_it, true));
        assert!(!at_a_loss(&off, &beside_it, false));
        // In edit mode a tap picks the thing up, or paints it.
        let mut hand = Customize::default();
        hand.begin();
        assert!(!at_a_loss(&hand, &on_it, false));
        assert!(!at_a_loss(&Customize::with_the_brush(), &on_it, false));
    }

    #[test]
    fn how_a_thing_is_moved_is_said_in_words_only_while_they_are_turned_on() {
        // Turned on, as they were for a day: nothing at the first tap, the
        // words at the second, and after a few times never again.
        let mut customize = Customize::default();
        let mut last = None;
        let mut said = 0;
        for tap in 0..40 {
            if says_how(true, &customize, &mut last, tap as f32) {
                assert!(tap % 2 == 1, "at tap {tap}");
                customize.note_hint();
                said += 1;
            }
        }
        assert!(said > 0 && !customize.may_hint(), "said {said} times");
        // Turned off, never, however often a thing is tapped: nor is a tap
        // counted toward the words meanwhile.
        let (customize, mut last) = (Customize::default(), None);
        for tap in 0..40 {
            assert!(!says_how(false, &customize, &mut last, tap as f32));
        }
        assert_eq!(last, None);
        // And off is how they stand (Hajun, 2026-11-23: "the tip
        // notification to go into house edit mode is not the real fix -
        // should delete that and use another way"): the button beats
        // instead, every time (`customize::Pointed`).
        assert!(!customize::HINTS);
    }

    #[test]
    fn nothing_of_yours_answers_a_tap_but_in_edit_mode_with_the_hand_and_never_in_your_car() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.insert_resource(Driving::for_tests(false));
        world.insert_resource(Customize::default());
        let bed = world.spawn(Mine).id();
        let out_of_reach = |world: &mut World| {
            world.run_system_once(hands_off).unwrap();
            world.get::<Untappable>(bed).is_some()
        };
        // Edit mode off (Hajun: "with edit mode off, tapping objects should
        // do nothing").
        assert!(out_of_reach(&mut world));
        // On, with the hand: a tap picks it up.
        world.resource_mut::<Customize>().begin();
        assert!(!out_of_reach(&mut world));
        // In your car, not even then.
        world.insert_resource(Driving::for_tests(true));
        assert!(out_of_reach(&mut world));
        world.insert_resource(Driving::for_tests(false));
        assert!(!out_of_reach(&mut world));
        // With the brush a tap paints it, and does not pick it up.
        world.insert_resource(Customize::with_the_brush());
        assert!(out_of_reach(&mut world));
        // Something brought out of the bag: the hand again.
        world.resource_mut::<Customize>().begin();
        assert!(!out_of_reach(&mut world));
    }

    #[test]
    fn what_is_painted_of_each_thing_is_hung_on_it_as_the_bags_have_it() {
        use bevy::ecs::system::RunSystemOnce;
        use crate::bag::Paint;
        let here = At::Ground { x: 1.0, y: 0.0, z: 2.0, yaw: 0.0 };
        let mut world = World::new();
        world.init_resource::<Bag>();
        world.init_resource::<TheirPaints>();
        {
            let mut bag = world.resource_mut::<Bag>();
            bag.has.buy("singlebed", 1_000).unwrap();
            bag.has.buy("singlebed", 1_000).unwrap();
            bag.has.put(2, "singlebed", here).unwrap();
            bag.has.put(3, "singlebed", here).unwrap();
            let stands = bag.has.stands.clone();
            bag.paints.set(&stands, 2, "blanket", [0, 0, 255]).unwrap();
        }
        // Two beds of yours, one painted; and someone else's bed, which
        // goes by the same number in their bag, painted red.
        let painted = world.spawn((Mine, Stood { id: 2, at: here }, Colours::default())).id();
        let plain = world.spawn((Mine, Stood { id: 3, at: here }, Colours::default())).id();
        let whose = TheirPiece {
            who: 9,
            id: 2,
            at: here,
            plot: 0,
        };
        let theirs = world.spawn((whose, Colours::default())).id();
        let red = Paint {
            id: 2,
            part: "blanket".into(),
            colour: [255, 0, 0],
        };
        world.resource_mut::<TheirPaints>().0.insert(9, vec![red]);
        let colours = |world: &World, thing: Entity| world.get::<Colours>(thing).unwrap().0.clone();
        world.run_system_once(keep_colours).unwrap();
        assert_eq!(colours(&world, painted), [("blanket".to_string(), [0, 0, 255])]);
        assert!(colours(&world, plain).is_empty());
        assert_eq!(colours(&world, theirs), [("blanket".to_string(), [255, 0, 0])]);
        // Yours taken back and stood again under its number is as it was
        // made; and theirs is whatever the server says next, nothing here.
        {
            let mut bag = world.resource_mut::<Bag>();
            bag.has.take(2).unwrap();
            let stands = bag.has.stands.clone();
            bag.paints.fit(&stands);
            bag.has.put(2, "singlebed", here).unwrap();
        }
        world.resource_mut::<TheirPaints>().0.insert(9, Vec::new());
        world.run_system_once(keep_colours).unwrap();
        assert!(colours(&world, painted).is_empty());
        assert!(colours(&world, theirs).is_empty());
    }

    #[test]
    fn a_place_is_the_same_to_a_millimetre_and_a_turn_the_same_all_the_way_round() {
        let here = At::Ground { x: 1.0, y: 0.0, z: 2.0, yaw: 0.0 };
        assert!(same_place(&here, &At::Ground { x: 1.0004, y: 0.0, z: 2.0, yaw: std::f32::consts::TAU }));
        assert!(!same_place(&here, &At::Ground { x: 1.01, y: 0.0, z: 2.0, yaw: 0.0 }));
        assert!(!same_place(&here, &At::Ground { x: 1.0, y: 0.0, z: 2.0, yaw: 0.1 }));
        let in_a_wall = At::Wall { wall: 1, along: 2.0, up: 0.0, face: 1 };
        assert!(same_place(&in_a_wall, &At::Wall { wall: 1, along: 2.0005, up: 0.0, face: 1 }));
        assert!(!same_place(&in_a_wall, &At::Wall { wall: 2, along: 2.0, up: 0.0, face: 1 }));
        assert!(!same_place(&in_a_wall, &here));
    }

    #[test]
    fn every_building_there_is_to_own_is_a_piece_the_game_can_put_down() {
        for thing in crate::bag::THINGS.iter().filter(|thing| thing.stands()) {
            assert!(def_of(thing.kind).is_some(), "{} is no piece of `build::PIECES`", thing.kind);
        }
        // And every piece there is can be owned.
        for def in PIECES {
            assert!(roundtown_net::bag::thing(def.kind).is_some_and(|thing| thing.stands()), "{} cannot be owned", def.kind);
        }
    }
}
