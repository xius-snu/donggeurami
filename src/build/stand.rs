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

use bevy::prelude::*;
use roundtown_net::{Ask, At};

use super::{
    BuildSystems, Ghost, Misplaced, Mount, OnWall, PIECES, Piece, PieceDef, Reshape, Rooms, Solid, Stock, Wall, Yard,
    bring_out, piece_ready, unghost, wall_index, walls_of,
};
use crate::Player;
use crate::bag::{Bag, BagSystems, Standing, Theirs};
use crate::drive::Driving;
use crate::editor::{self, Edit, EditMenu, Editable, Touch, Untappable};
use crate::hud;
use crate::island::Venue;
use crate::land::{Plot, Plots};
use crate::lobby::{Channel, Seat};
use crate::net::Online;
use crate::ui::ui;

/// How near two places have to be to be the same place, in metres and in
/// radians: a thing put back down where it was picked up has not moved.
const SAME: f32 = 1e-3;
/// How long "It has to stand on your land." is up, in seconds.
const SAID_SECS: f32 = 2.5;

/// Your own plot of the town, while you are in the town and its plots have
/// been read: the square of it, whether you are standing on it, and whether
/// a house of yours stands there, and is down with its walls made.
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub(crate) struct Home {
    pub yard: Option<Yard>,
    pub on_it: bool,
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
struct Stood {
    id: u32,
    at: At,
}

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

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Home>()
        .add_message::<Stand>()
        .add_systems(
            Update,
            (
                (find_home, note_edits, bring, leave)
                    .chain()
                    .after(editor::EditSystems)
                    .after(BagSystems)
                    .before(BuildSystems),
                hands_off_in_the_car.before(editor::EditSystems),
                (report, keep_mine, keep_theirs, say_misplaced).chain().after(BuildSystems),
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
fn placed(yard: &Yard, x: f32, y: f32, z: f32, yaw: f32) -> Transform {
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

/// In your car, nothing of yours is picked up by a tap: driving across your
/// plot, a stray one would have a bed in your hand. Everything of yours
/// there but a house, which `super::untappable_from_inside` puts out of
/// reach the same way with the houses you stand in. Written only as you
/// get in and out.
#[allow(clippy::type_complexity)]
fn hands_off_in_the_car(
    driving: Res<Driving>,
    pieces: Query<(Entity, Has<Untappable>), (With<Mine>, Without<Rooms>)>,
    mut commands: Commands,
) {
    let driving = driving.on();
    for (piece, untappable) in &pieces {
        if driving && !untappable {
            commands.entity(piece).insert(Untappable);
        } else if !driving && untappable {
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
        Some((yard, yard.holds(at.translation.xz(), 0.0)))
    });
    let house = houses
        .iter()
        .find(|(piece, ..)| piece.def.mount == Mount::House)
        .map(|(_, ghost, walled)| !ghost && walled);
    home.set_if_neq(Home {
        yard: mine.map(|(yard, _)| yard),
        on_it: mine.is_some_and(|(_, on_it)| on_it),
        house,
    });
}

/// What the edit menu asks of one of your pieces that the bag has to hear
/// of: picked back up, it is marked, to be told of once it is down again
/// ([`report`]); thrown away, it is back in the bag, and with a house,
/// whatever was in its walls, which goes with it. Before the menu's asks
/// are done (`super::edit_pieces`), which takes what is thrown away out of
/// the world.
fn note_edits(
    mut edits: MessageReader<Edit>,
    mine: Query<(&Piece, Option<&Stood>), With<Mine>>,
    online: Res<Online>,
    mut bag: ResMut<Bag>,
    mut asks: MessageWriter<Ask>,
    mut commands: Commands,
) {
    for &edit in edits.read() {
        match edit {
            Edit::Opened(target) if mine.contains(target) => {
                commands.entity(target).insert(Lifted);
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
    commands.entity(ghost).remove::<(Ghost, super::Fresh, Lifted)>().insert(Displaced);
    reshape.0 = true;
}

/// Brings a building out of your bag onto your plot, in front of you, as a
/// ghost with the edit menu open on it. Whatever was a ghost already is
/// put away first ([`settle`]): one thing is put down at a time.
#[allow(clippy::too_many_arguments)]
fn bring(
    mut stands: MessageReader<Stand>,
    home: Res<Home>,
    stock: Res<Stock>,
    me: Query<&Transform, With<Player>>,
    mine: Query<(Entity, &Piece, Has<Stood>, Has<Ghost>), With<Mine>>,
    (children, solids): (Query<&Children>, Query<&Solid>),
    mut menu: ResMut<EditMenu>,
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
        bring_out(&mut commands, &stock, &mut menu, def, house, Venue::Town, me.translation, (Mine, Lifted));
    }
}

/// Leaving the town, whatever of yours was a ghost is put away
/// ([`settle`]): the menu on it has gone (`editor::put_away`).
fn leave(
    me: Query<&Venue, (With<Player>, Changed<Venue>)>,
    ghosts: Query<(Entity, Has<Stood>), (With<Mine>, With<Ghost>)>,
    (children, solids): (Query<&Children>, Query<&Solid>),
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    if me.single().is_ok_and(|&venue| venue != Venue::Town) {
        for (ghost, stood) in &ghosts {
            settle(ghost, stood, &children, &solids, &mut reshape, &mut commands);
        }
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
    piece.insert((Piece { def }, tag, WorldAssetRoot(stock.scene(def))));
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

/// Keeps what stands on the plots of the others in your channel of the
/// town as the server has said it: for each of them who sits in a seat
/// ([`Channel`]), what it said stands on their plot ([`Theirs`]), on the
/// plot their seat has. As any of that changes, and for as long as
/// something waits for its wall.
#[allow(clippy::too_many_arguments)]
fn keep_theirs(
    theirs: Res<Theirs>,
    channel: Res<Channel>,
    plots: Res<Plots>,
    online: Res<Online>,
    stock: Res<Stock>,
    mut pieces: Query<(Entity, &Piece, &mut TheirPiece, &mut Transform)>,
    walls: Query<(Entity, &Wall, &ChildOf)>,
    mut waiting: Local<bool>,
    mut reshape: ResMut<Reshape>,
    mut commands: Commands,
) {
    if !(theirs.is_changed() || channel.is_changed() || plots.is_changed() || *waiting) {
        return;
    }
    *waiting = false;
    let me = online.me().map(|me| me.id);
    // Whose plot is which: everyone seated but you.
    let plot_of = |who: u64| -> Option<usize> {
        let seated = channel.0.iter().find(|seated| seated.who == who && Some(who) != me)?;
        plots.of_seat.get(usize::from(seated.seat)).copied().filter(|&plot| plot < plots.all.len())
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
            commands.entity(entity).despawn();
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

/// Says why a piece asked to be put down has not been
/// (`super::Misplaced`): across the screen, one line at a time.
fn say_misplaced(mut misplaced: MessageReader<Misplaced>, said: Query<Entity, With<Said>>, mut commands: Commands) {
    if misplaced.read().count() == 0 {
        return;
    }
    for line in &said {
        commands.entity(line).despawn();
    }
    let ui = ui();
    hud::announce(&mut commands, None, &ui.words.bag.off_your_land, hud::CAPTION, SAID_SECS).insert(Said);
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
