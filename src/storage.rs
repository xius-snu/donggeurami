//! What keeps things, opened (since 2026-11-27; Hajun: "cabinets can have
//! storage inventory that is like a chest - u can put items in there, any
//! anyone can open it and take it out or put sth there").
//!
//! **What keeps things, and how many** (`roundtown_net::bag::keeps`, laid
//! out here, [`layout`]): "cabinet = 8 square slots (4 on left, 4 on
//! right) barbeque = 3 square slots but food only (1 row of 3) end table =
//! 3 square slots (1 row of 3) safe = 9 square slots (3 rows of 3, like a
//! 3x3 square) dresser = 12 square slots (6 on left, 6 on right)": the
//! cabinet two rows of two on each side of a gap, as its two doors are, and
//! the dresser three rows of two, as its six drawers are.
//!
//! **The window** ("so when u click the proximity bubble, it just opens a
//! closable small popup that just shows text like "Cabinet" at top left,
//! close button on top right, and the square slots with small white plus
//! icon in the middle if theyre empty or the item + quantity on bottom
//! right corner of that square if there is an item"): a tap on the chest
//! in the bubble over one (`working`) opens it, a window as every window is
//! (`window`), its name along its top from the left, the bare cross in its
//! corner, and its slots under it, each a square card: an empty one with a
//! plus in it, a full one with the thing's picture in it and how many in
//! its lower right corner. The cross, a tap anywhere off it, or Escape,
//! shuts it; so does the thing going (its owner taking it back into their
//! bag), and leaving the town.
//!
//! **Putting something in** ("u press the plus and it brings up another
//! small popup on top where u can select which item from ur inventory to
//! add it, and then after u select it it shows another popup for quantity
//! and the when u press ok ur back to the first popup with the Cabinet"):
//! the plus opens a box over the window, "Choose an item", what is in your
//! bag four to a row as the bag's window has it (food alone, for the
//! barbecue), each with how many of it you have; a thing chosen, a box over
//! that asks how many, from one to all you have of it, with a minus and a
//! plus either side of the number and a bar to drag along under it, and
//! OK, which puts that many in and goes back to the window. A tap on the
//! dark round a box, or Escape, goes back one box.
//!
//! **Taking it out** ("when u click on an item, u get that item added to
//! ur inventory, and shows quick message like "_____ x1 has been added to
//! your inventory""): a tap on a full slot takes all of it into your bag
//! (mine: all, not one at a time), or as much as your bag has room for, and
//! says so across the screen ([`say_taken`]).
//!
//! What is done is `bag`'s (`Bag::stow`, `Bag::unstow`): online the
//! server's to do and to say, and the window shows what it says is in
//! there, whoever put it there; offline on your own plot alone, at once.

use bevy::asset::RenderAssetUsages;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui_widgets::ScrollArea;
use roundtown_net::Ask;
use roundtown_net::bag as rules;

use crate::bag::{Bag, BagSystems, Stack, Taken, TheirWorkings, Thing, Workings};
use crate::build::stand::{Stood, TheirPiece, owner_of};
use crate::build::{PIECES, Piece, Stock};
use crate::builder::{DIALOG_LAYER, Fill};
use crate::customize::Customize;
use crate::drive::{Driving, car_def};
use crate::flick::{self, Coasting, Throw, Track, Way};
use crate::hud::{self, DialogUp, TakesPress};
use crate::island::Venue;
use crate::net::Online;
use crate::ui::{UiChanged, fill, ui};
use crate::window::{self, Look};
use crate::Player;

/// A slot, in shares of the short side of the screen: how big, how far
/// apart, and how far apart a cabinet's or a dresser's two sides; how much
/// of it a thing's picture takes, and the plus; and how many there are in
/// it, in its corner, and how far in from its edges.
const SLOT: f32 = 13.0;
const SLOT_GAP: f32 = 1.6;
const SIDE_GAP: f32 = 4.4;
const SLOT_PICTURE: f32 = 82.0;
const PLUS: f32 = 36.0;
const COUNT_TEXT: f32 = 2.9;
const COUNT_IN: f32 = 0.7;
/// The window's title, and the room under it before the slots.
const UNDER_TITLE: f32 = 1.4;
/// The box things to put in are chosen in: how wide, how tall at the most,
/// four cards to a row, each this tall, the name under it and its words.
const CHOOSE_WIDE: f32 = 80.0;
const CHOOSE_HIGH: f32 = 76.0;
const CARD_HIGH: f32 = 14.0;
const CARD_GAP: f32 = 2.0;
const NAME_ROW: f32 = 4.6;
const NAME_TEXT: f32 = 2.4;
/// The box that asks how many: how wide; the thing's picture; the number
/// and the round buttons either side of it; and the bar under them, how
/// thick, and its knob.
const COUNT_WIDE: f32 = 58.0;
const COUNT_PICTURE: f32 = 15.0;
const NUMBER_TEXT: f32 = 6.5;
const STEP_BUTTON: f32 = 8.0;
const BAR: f32 = 1.6;
const KNOB: f32 = 4.4;
/// How far a press may be dragged and still be a tap, in logical pixels,
/// as in the bag's window.
const TAP_SLOP: f32 = 16.0;
/// How long a line across the screen saying what has come into your bag is
/// up, in seconds, and how far down the screen it is, and how big: under the
/// tallest window, the dresser's, which reaches 79 down, and between the
/// stick and the buttons round the jump button.
const SAID_SECS: f32 = 2.6;
const SAID_DOWN: f32 = 84.0;
const SAID_TEXT: f32 = 4.0;
/// How long something asked of the server is waited for before the slots
/// answer a tap again, in seconds, if it has said nothing.
const WAIT_SECS: f32 = 1.5;

/// How the slots of a thing of `kind` are laid out: so many rows, so many
/// slots across each side of it, and one side or two. Numbered along the
/// rows of its left side, top to bottom, then of its right.
pub(crate) fn layout(kind: &str) -> Option<(u8, u8, u8)> {
    match kind {
        // Two doors, four behind each.
        "cabinet" => Some((2, 2, 2)),
        // Six drawers, three down each side, two to a drawer.
        "dresser" => Some((3, 2, 2)),
        "safe" => Some((3, 3, 1)),
        "end_table" | "barbecue" => Some((1, 3, 1)),
        _ => None,
    }
}

/// The window of what keeps things that is this piece is to open (a tap on
/// the chest in the bubble over it, `working`).
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Open(pub(crate) Entity);

/// Which box is up over the window.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Step {
    /// None: the slots.
    #[default]
    Slots,
    /// What to put into this slot is being chosen.
    Choosing { slot: usize },
    /// How many of this to put into this slot.
    Counting { slot: usize, kind: &'static str, count: u16 },
}

/// The window: the piece it is of, if it is up; which box is up over it;
/// and, online, something asked of the server and not yet said, with what
/// was in there as it was asked.
#[derive(Resource, Default)]
struct Chest {
    open: Option<Entity>,
    step: Step,
    asked: Option<(f32, Workings)>,
}

/// What a press on something of the window's does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Tap {
    Close,
    Slot(usize),
    Choose(&'static str),
    Less,
    More,
    Ok,
    /// The dark round a box: back out of it.
    Back,
    Nothing,
}

/// The presses since they were last acted on; where along the bar a finger
/// put it, from 0 at its left to 1 at its right; and the press now down, as
/// the bag's window keeps it, for a list to be scrolled and flicked.
#[derive(Resource, Default)]
struct Pressed {
    taps: Vec<Tap>,
    bar: Option<f32>,
    dragged: f32,
    pointer: Option<PointerId>,
    way: Way,
    slack: f32,
    followed: f32,
    caught: bool,
    track: Track,
}

impl Pressed {
    fn feel(&mut self, gone: Vec2) -> Way {
        if self.way == Way::Unsure {
            self.way = flick::way(gone);
            if self.way == Way::Along {
                self.slack = flick::SLOP.copysign(gone.y);
                self.followed = 0.0;
            }
        }
        self.way
    }

    fn tapped(&self) -> bool {
        self.way == Way::Unsure && self.dragged <= TAP_SLOP && !self.caught
    }
}

/// The root of the window, over the whole screen; the list of things to
/// choose from, which scrolls; and the bar that says how many.
#[derive(Component)]
struct Root;
#[derive(Component)]
struct List;
#[derive(Component)]
pub(crate) struct Bar;

/// The pictures drawn for the window: the plus in an empty slot, and the
/// minus and the plus either side of how many.
#[derive(Resource)]
struct Pictures {
    plus: Handle<Image>,
    minus: Handle<Image>,
}

/// A line across the screen saying what has come into your bag, and how
/// long it has been up.
#[derive(Component)]
struct Said(f32);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Chest>()
        .init_resource::<Pressed>()
        .add_message::<Open>()
        .add_observer(start_press)
        .add_observer(press)
        .add_systems(Startup, draw_pictures)
        // Before your walk is read, as the bag's window is: the window up
        // stands you still the frame it comes up. (A tap on the bubble that
        // opens it is read after the camera, which is after your walk: the
        // window comes up the frame after.)
        .add_systems(
            Update,
            (open, act, shut_when_gone, redraw, say_taken, clear_said)
                .chain()
                .after(BagSystems)
                .after(flick::coast)
                .before(crate::read_player_input),
        );
}

fn draw_pictures(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    commands.insert_resource(Pictures {
        plus: images.add(sign(true)),
        minus: images.add(sign(false)),
    });
}

/// A plus, or a minus, white on clear, its strokes rounded at their ends.
fn sign(plus: bool) -> Image {
    const SIZE: u32 = 96;
    const REACH: f32 = 0.72;
    const THICK: f32 = 0.11;
    let pixel = 2.0 / SIZE as f32;
    let stroke = |at: Vec2, along: Vec2| {
        let share = (at.dot(along) / REACH).clamp(-1.0, 1.0) * REACH;
        at.distance(along * share) - THICK
    };
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new((column as f32 + 0.5) * pixel - 1.0, 1.0 - (row as f32 + 0.5) * pixel);
            let across = stroke(at, Vec2::X);
            let edge = if plus { across.min(stroke(at, Vec2::Y)) } else { across };
            let cover = (0.5 - edge / pixel).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (cover * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

// ------------------------------------------------------------- the presses

/// A new press, wherever it lands: nothing dragged yet, going no way yet;
/// one that goes down on the list while it is going on by itself stops it,
/// and is no tap.
fn start_press(
    press: On<Pointer<Press>>,
    time: Res<Time<Real>>,
    parents: Query<&ChildOf>,
    coasting: Query<(), With<Coasting>>,
    mut pressed: ResMut<Pressed>,
    mut commands: Commands,
) {
    if press.button != PointerButton::Primary || press.original_event_target() != press.entity {
        return;
    }
    let held = std::iter::successors(Some(press.entity), |&entity| parents.get(entity).ok().map(ChildOf::parent)).find(|&entity| coasting.contains(entity));
    if let Some(list) = held {
        commands.entity(list).try_remove::<Coasting>();
    }
    pressed.dragged = 0.0;
    pressed.pointer = Some(press.pointer_id);
    pressed.way = Way::Unsure;
    pressed.caught = held.is_some();
    pressed.track.begin(time.elapsed_secs_f64());
}

/// Whatever of the window's was tapped, to be acted on ([`act`]): only what
/// the press landed on, and not a press dragged to scroll the list.
fn press(click: On<Pointer<Click>>, taps: Query<&Tap>, mut pressed: ResMut<Pressed>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(&tap) = taps.get(click.entity)
        && pressed.tapped()
    {
        pressed.taps.push(tap);
    }
}

/// A press dragged anywhere on a box: how far it has gone, which way, and
/// where it has been.
fn drag_box(drag: On<Pointer<Drag>>, time: Res<Time<Real>>, mut pressed: ResMut<Pressed>) {
    if Some(drag.pointer_id) != pressed.pointer {
        return;
    }
    pressed.dragged = pressed.dragged.max(drag.distance.length());
    pressed.feel(drag.distance);
    pressed.track.moved(time.elapsed_secs_f64(), time.delta_secs_f64(), drag.distance);
}

/// The list scrolled by a finger, or the mouse held down, dragged along it.
fn drag_list(drag: On<Pointer<Drag>>, mut lists: Query<(&ComputedNode, &mut ScrollPosition), With<List>>, mut pressed: ResMut<Pressed>) {
    if Some(drag.pointer_id) != pressed.pointer || pressed.feel(drag.distance) != Way::Along {
        return;
    }
    let Ok((node, mut scroll)) = lists.get_mut(drag.entity) else {
        return;
    };
    let wanted = drag.distance.y - pressed.slack;
    let by = wanted - pressed.followed;
    pressed.followed = wanted;
    let to = (scroll.y - by).clamp(0.0, flick::room(node));
    if scroll.y != to {
        scroll.y = to;
    }
}

/// The list let go of: flicked, it goes on by itself and slows to a stop.
fn let_go_of_list(end: On<Pointer<DragEnd>>, time: Res<Time<Real>>, lists: Query<&ComputedNode, With<List>>, pressed: Res<Pressed>, mut commands: Commands) {
    if Some(end.pointer_id) != pressed.pointer || pressed.way != Way::Along {
        return;
    }
    let Ok(node) = lists.get(end.entity) else {
        return;
    };
    if flick::room(node) <= 0.0 {
        return;
    }
    let now = time.elapsed_secs_f64();
    let speed = -pressed.track.speed(now, time.delta_secs_f64()).y;
    if let Some(throw) = Throw::at(speed) {
        commands.entity(end.entity).insert(Coasting::new(throw, now));
    }
}

/// How far along the bar a pointer at `pointer` is, from 0 at its left to
/// 1 at its right.
fn along_the_bar(node: &ComputedNode, place: &UiGlobalTransform, pointer: Vec2) -> f32 {
    let half = (node.size().x * 0.5).max(1.0);
    let x = pointer.x / node.inverse_scale_factor().max(f32::EPSILON) - place.translation.x;
    (0.5 + x / half * 0.5).clamp(0.0, 1.0)
}

/// A finger put on the bar, or dragged along it: that many.
fn press_bar(press: On<Pointer<Press>>, bars: Query<(&ComputedNode, &UiGlobalTransform), With<Bar>>, mut pressed: ResMut<Pressed>) {
    if let Ok((node, place)) = bars.get(press.entity) {
        pressed.bar = Some(along_the_bar(node, place, press.pointer_location.position));
    }
}

fn drag_bar(drag: On<Pointer<Drag>>, bars: Query<(&ComputedNode, &UiGlobalTransform), With<Bar>>, mut pressed: ResMut<Pressed>) {
    if let Ok((node, place)) = bars.get(drag.entity) {
        pressed.bar = Some(along_the_bar(node, place, drag.pointer_location.position));
    }
}

// ------------------------------------------------------------- the window

/// What can be put into a thing that only takes food or not, `food_only`,
/// of what is in the bag `has`: each with how many of it there are, in the
/// order the bag's window shows what you own.
fn choosable(has: &rules::Bag, food_only: bool) -> Vec<(&'static Thing, u16)> {
    crate::bag::panel::in_your_bag(has).into_iter().filter(|(thing, _)| !food_only || rules::is_food(thing.kind)).collect()
}

/// The most of `kind` that can go into an empty slot out of `has`: all of
/// it there is in the bag, up to as many as one may own.
fn most_of(has: &rules::Bag, kind: &str) -> u16 {
    has.free(kind).min(rules::MOST_OF_A_KIND)
}

/// How many the bar says, at `share` along it, of from one to `most`.
fn count_along(share: f32, most: u16) -> u16 {
    if most <= 1 {
        return 1;
    }
    (1.0 + share.clamp(0.0, 1.0) * f32::from(most - 1)).round() as u16
}

/// How far along the bar `count` of from one to `most` is.
fn share_of(count: u16, most: u16) -> f32 {
    if most <= 1 { 1.0 } else { f32::from(count.saturating_sub(1)) / f32::from(most - 1) }
}

/// Opens the window of what keeps things that was asked for, with nothing
/// else up.
fn open(mut opens: MessageReader<Open>, mut chest: ResMut<Chest>, mut up: ResMut<DialogUp>, pieces: Query<&Piece>) {
    let Some(&Open(piece)) = opens.read().last() else {
        return;
    };
    if chest.open.is_some() || up.0 || !pieces.get(piece).is_ok_and(|of| rules::keeps(of.def.kind).is_some()) {
        return;
    }
    *chest = Chest {
        open: Some(piece),
        ..default()
    };
    up.0 = true;
}

/// Shuts the window and whatever is up over it.
fn shut(chest: &mut Chest, up: &mut DialogUp) {
    *chest = Chest::default();
    if up.0 {
        up.0 = false;
    }
}

/// Does what was pressed, and Escape on desktop: back out of a box, or shut
/// the window.
#[allow(clippy::too_many_arguments)]
fn act(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pressed: ResMut<Pressed>,
    mut chest: ResMut<Chest>,
    mut up: ResMut<DialogUp>,
    online: Res<Online>,
    (mut bag, theirs): (ResMut<Bag>, Res<TheirWorkings>),
    pieces: Query<(&Piece, Option<&Stood>, Option<&TheirPiece>)>,
    (mut asks, mut taken): (MessageWriter<Ask>, MessageWriter<Taken>),
    mut commands: Commands,
) {
    let Some(piece) = chest.open else {
        pressed.taps.clear();
        pressed.bar = None;
        return;
    };
    let Ok((of, mine, their)) = pieces.get(piece) else {
        return;
    };
    let (Some((who, id)), Some(keeps)) = (owner_of(mine, their), rules::keeps(of.def.kind)) else {
        return;
    };
    let now = time.elapsed_secs();
    let held = theirs.of(&bag, who).cloned().unwrap_or_default();
    // Something asked of the server is answered once what is in there has
    // changed, or has been waited for long enough.
    if chest.asked.as_ref().is_some_and(|(since, before)| *before != held || now - since > WAIT_SECS) {
        chest.asked = None;
    }
    let mut taps = std::mem::take(&mut pressed.taps);
    if keyboard.just_pressed(KeyCode::Escape) {
        taps.push(match chest.step {
            Step::Slots => Tap::Close,
            _ => Tap::Back,
        });
    }
    // The bar, dragged: that many.
    if let (Some(share), Step::Counting { slot, kind, count }) = (pressed.bar.take(), chest.step) {
        let now = count_along(share, most_of(&bag.has, kind));
        if now != count {
            chest.step = Step::Counting { slot, kind, count: now };
        }
    }
    let me = online.me().map(|me| me.id);
    for tap in taps {
        match (tap, chest.step) {
            (Tap::Close, _) => {
                shut(&mut chest, &mut up);
                return;
            }
            (Tap::Back, Step::Counting { slot, .. }) => chest.step = Step::Choosing { slot },
            (Tap::Back, Step::Choosing { .. }) => chest.step = Step::Slots,
            (Tap::Slot(slot), Step::Slots) if chest.asked.is_none() => match held.slots(id).get(slot).cloned().flatten() {
                // A full one: all of it into your bag, as much as fits.
                Some(stack) => {
                    if bag.has.room_for(&stack.kind) == 0 {
                        say(&mut commands, &fill(&ui().words.storage.bag_full, &[("name", ui().words.bag.name(&stack.kind))]));
                        continue;
                    }
                    match bag.unstow(me, &mut asks, who, id, slot) {
                        Ok(Some(came)) => {
                            taken.write(Taken(came));
                        }
                        Ok(None) => chest.asked = Some((now, held.clone())),
                        Err(_) => {}
                    }
                }
                // An empty one: what to put in it.
                None if slot < usize::from(keeps.slots) => chest.step = Step::Choosing { slot },
                None => {}
            },
            (Tap::Choose(kind), Step::Choosing { slot }) if most_of(&bag.has, kind) > 0 => {
                chest.step = Step::Counting { slot, kind, count: 1 };
            }
            (Tap::Less, Step::Counting { slot, kind, count }) if count > 1 => {
                chest.step = Step::Counting { slot, kind, count: count - 1 };
            }
            (Tap::More, Step::Counting { slot, kind, count }) if count < most_of(&bag.has, kind) => {
                chest.step = Step::Counting { slot, kind, count: count + 1 };
            }
            (Tap::Ok, Step::Counting { slot, kind, count }) => {
                let count = count.min(most_of(&bag.has, kind));
                if count > 0 && bag.stow(me, &mut asks, who, id, slot, kind, count) && me.is_some() {
                    chest.asked = Some((now, held.clone()));
                }
                chest.step = Step::Slots;
            }
            _ => {}
        }
    }
}

/// Shuts the window as the thing is gone (its owner has taken it back into
/// their bag, or it has gone with their plot), as you leave the town, or
/// get into your car, or edit mode comes on.
fn shut_when_gone(
    mut chest: ResMut<Chest>,
    mut up: ResMut<DialogUp>,
    pieces: Query<(), With<Piece>>,
    me: Query<&Venue, With<Player>>,
    (driving, customize): (Res<Driving>, Res<Customize>),
) {
    let Some(piece) = chest.open else {
        return;
    };
    let away = me.single().is_ok_and(|&venue| venue != Venue::Town);
    if !pieces.contains(piece) || away || driving.on() || customize.on() {
        shut(&mut chest, &mut up);
    }
}

/// What the window shows: what keeps things, what is in each of its slots,
/// the box up over it, and what there is to choose from in that box, or how
/// many there are of what is being counted. It is drawn again only when this
/// differs: a button drawn again between a finger going down on it and
/// coming up is never pressed (seen online, someone else's doll coming into
/// the cabinet as its "+" was pressed).
#[derive(Clone, PartialEq, Debug)]
struct Shown {
    kind: &'static str,
    slots: Vec<Option<Stack>>,
    step: Step,
    offered: Vec<(&'static str, u16)>,
}

/// Puts the window up while it is open, and draws it again whenever what
/// it shows has changed ([`Shown`]), or the file's colours and words have.
/// Takes it down once it is shut.
#[allow(clippy::too_many_arguments)]
fn redraw(
    chest: Res<Chest>,
    (bag, theirs): (Res<Bag>, Res<TheirWorkings>),
    changed: Res<UiChanged>,
    roots: Query<Entity, With<Root>>,
    pieces: Query<(&Piece, Option<&Stood>, Option<&TheirPiece>)>,
    (assets, stock, pictures): (Res<AssetServer>, Res<Stock>, Res<Pictures>),
    mut drawn: Local<Option<Shown>>,
    mut commands: Commands,
) {
    let open = chest.open.and_then(|piece| {
        let (of, mine, their) = pieces.get(piece).ok()?;
        let (who, id) = owner_of(mine, their)?;
        Some((of.def.kind, theirs.of(&bag, who).cloned().unwrap_or_default(), id))
    });
    let Some((kind, held, id)) = open else {
        for root in &roots {
            commands.entity(root).despawn();
        }
        *drawn = None;
        return;
    };
    let offered = match chest.step {
        Step::Slots => Vec::new(),
        Step::Choosing { .. } => {
            let food_only = rules::keeps(kind).is_some_and(|keeps| keeps.food_only);
            choosable(&bag.has, food_only).into_iter().map(|(thing, count)| (thing.kind, count)).collect()
        }
        Step::Counting { kind, .. } => vec![(kind, most_of(&bag.has, kind))],
    };
    let shown = Shown {
        kind,
        slots: held.slots(id).to_vec(),
        step: chest.step,
        offered,
    };
    let restyled = changed.is_changed() && !changed.is_added();
    if !restyled && !roots.is_empty() && drawn.as_ref() == Some(&shown) {
        return;
    }
    *drawn = Some(shown);
    for root in &roots {
        commands.entity(root).despawn();
    }
    let view = View {
        kind,
        slots: held.slots(id),
        step: chest.step,
        has: &bag.has,
        assets: &assets,
        stock: &stock,
        pictures: &pictures,
    };
    draw(&mut commands, &view);
}

/// What the window is drawn from.
struct View<'a> {
    kind: &'static str,
    slots: &'a [Option<Stack>],
    step: Step,
    has: &'a rules::Bag,
    assets: &'a AssetServer,
    stock: &'a Stock,
    pictures: &'a Pictures,
}

/// A thing's picture, as tall as `share` of what it is in and as wide as
/// it is tall, as the bag's window has it.
fn square_picture(image: Handle<Image>, share: f32) -> impl Bundle {
    (
        ImageNode::new(image),
        Node {
            height: Val::Percent(share),
            aspect_ratio: Some(1.0),
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// The picture of the thing that goes by `kind`: a building's is its
/// piece's, a car's its own.
fn picture_of(view: &View, kind: &str) -> Handle<Image> {
    match PIECES.iter().find(|def| def.kind == kind) {
        Some(def) => view.stock.picture(def),
        None => view.assets.load(car_def(kind).picture),
    }
}

/// The window as things stand: its root over the whole screen, which a tap
/// on shuts it; the pane, its title and its cross; its slots; and the box
/// up over it, if there is one.
fn draw(commands: &mut Commands, view: &View) {
    let ui = ui();
    let colours = ui.colours.window;
    let root = commands
        .spawn((
            Root,
            Tap::Close,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let pane = commands
        .spawn((
            ChildOf(root),
            Tap::Nothing,
            window::pane(Node {
                padding: UiRect::all(Val::VMin(window::PAD)),
                ..default()
            }),
        ))
        .id();
    // Its name along its top, from the left, clear of the cross.
    commands.spawn((
        ChildOf(pane),
        window::header(),
        children![window::words(ui.words.bag.name(view.kind).to_string(), window::TITLE, colours.title)],
    ));
    commands.spawn((ChildOf(pane), window::cross(view.assets.load("icons/close.png"), Tap::Close)));
    let (rows, across, sides) = layout(view.kind).unwrap_or((1, 3, 1));
    let grid = commands
        .spawn((
            ChildOf(pane),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::VMin(SLOT_GAP),
                margin: UiRect::top(Val::VMin(UNDER_TITLE)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for row in 0..rows {
        let line = commands
            .spawn((
                ChildOf(grid),
                Node {
                    column_gap: Val::VMin(SLOT_GAP),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        for side in 0..sides {
            if side > 0 {
                commands.spawn((
                    ChildOf(line),
                    Node {
                        width: Val::VMin(SIDE_GAP - SLOT_GAP),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
            for column in 0..across {
                let slot = usize::from(side) * usize::from(rows * across) + usize::from(row * across + column);
                slot_card(commands, line, view, slot);
            }
        }
    }
    match view.step {
        Step::Slots => {}
        Step::Choosing { .. } => choosing(commands, root, view),
        Step::Counting { kind, count, .. } => counting(commands, root, view, kind, count),
    }
}

/// One slot: a square card, with a plus in it if it is empty, and the
/// thing's picture in it if not, how many in its lower right corner.
fn slot_card(commands: &mut Commands, line: Entity, view: &View, slot: usize) {
    let ui = ui();
    let colours = ui.colours.storage;
    let card = commands
        .spawn((
            ChildOf(line),
            Tap::Slot(slot),
            TakesPress,
            Button,
            Node {
                width: Val::VMin(SLOT),
                height: Val::VMin(SLOT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::VMin(window::CARD_ROUNDED)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(colours.slot),
            Fill {
                idle: colours.slot,
                lit: colours.slot_lit,
            },
        ))
        .id();
    match view.slots.get(slot).cloned().flatten() {
        None => {
            commands.spawn((ChildOf(card), window::picture(view.pictures.plus.clone(), Val::Percent(PLUS), colours.plus)));
        }
        Some(stack) => {
            commands.spawn((ChildOf(card), square_picture(picture_of(view, &stack.kind), SLOT_PICTURE)));
            commands.spawn((
                ChildOf(card),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::VMin(COUNT_IN),
                    bottom: Val::VMin(COUNT_IN * 0.5),
                    ..default()
                },
                Pickable::IGNORE,
                children![(
                    window::words(fill(&ui.words.storage.count, &[("count", &stack.count.to_string())]), COUNT_TEXT, colours.count),
                    TextShadow {
                        offset: Vec2::new(1.5, 1.5),
                        color: colours.count_stroke,
                    },
                )],
            ));
        }
    }
}

/// The dark over the whole screen behind a box, which a tap on goes back
/// out of the box, `over` the window by `layer`; and the box itself,
/// `wide` across, its title `title` in `ink`, for what goes in it.
fn a_box(commands: &mut Commands, root: Entity, layer: i32, wide: f32, title: String, ink: Color) -> Entity {
    let colours = ui().colours.window;
    let dark = commands
        .spawn((
            ChildOf(root),
            Tap::Back,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(colours.scrim),
            GlobalZIndex(DIALOG_LAYER + layer),
        ))
        .id();
    commands
        .spawn((
            ChildOf(dark),
            Tap::Nothing,
            TakesPress,
            Node {
                width: Val::VMin(wide),
                max_height: Val::VMin(CHOOSE_HIGH),
                flex_direction: FlexDirection::Column,
                row_gap: Val::VMin(2.0),
                padding: UiRect::all(Val::VMin(3.0)),
                border_radius: BorderRadius::all(Val::VMin(window::ROUNDED)),
                ..default()
            },
            BackgroundColor(colours.popup),
            children![window::words(title, window::TITLE, ink)],
        ))
        .observe(drag_box)
        .id()
}

/// The box things to put in are chosen in ("Choose an item"): what is in
/// your bag, four to a row, each a card with its picture, how many of it
/// you have, and its name under it in its rarity's colour; a list that
/// scrolls. With nothing that can go in, a line that says so.
fn choosing(commands: &mut Commands, root: Entity, view: &View) {
    let ui = ui();
    let words = &ui.words.storage;
    let food_only = rules::keeps(view.kind).is_some_and(|keeps| keeps.food_only);
    let inside = a_box(commands, root, 1, CHOOSE_WIDE, words.choose.clone(), ui.colours.window.title);
    let things = choosable(view.has, food_only);
    if things.is_empty() {
        let said = if food_only { &words.no_food } else { &words.nothing };
        commands.spawn((ChildOf(inside), window::words(said.clone(), window::HEADING, ui.colours.window.status)));
        return;
    }
    let list = commands
        .spawn((
            ChildOf(inside),
            List,
            ScrollArea,
            Node {
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                overflow: Overflow::scroll_y(),
                min_height: Val::ZERO,
                flex_shrink: 1.0,
                ..default()
            },
            ScrollPosition::default(),
        ))
        .observe(drag_list)
        .observe(let_go_of_list)
        .id();
    for (thing, free) in things {
        let cell = commands
            .spawn((
                ChildOf(list),
                Node {
                    width: Val::Percent(25.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::VMin(CARD_GAP * 0.5)),
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        let (card_fill, ..) = window::card(true);
        let card = commands
            .spawn((
                ChildOf(cell),
                Tap::Choose(thing.kind),
                TakesPress,
                Button,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::VMin(CARD_HIGH),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::VMin(window::CARD_ROUNDED)),
                    ..default()
                },
                BackgroundColor(card_fill.idle),
                card_fill,
            ))
            .id();
        commands.spawn((ChildOf(card), square_picture(picture_of(view, thing.kind), SLOT_PICTURE)));
        commands.spawn((
            ChildOf(card),
            Node {
                position_type: PositionType::Absolute,
                right: Val::VMin(COUNT_IN),
                bottom: Val::VMin(COUNT_IN * 0.5),
                ..default()
            },
            Pickable::IGNORE,
            children![(
                window::words(fill(&ui.words.bag.count, &[("count", &free.to_string())]), COUNT_TEXT, ui.colours.storage.count),
                TextShadow {
                    offset: Vec2::new(1.5, 1.5),
                    color: ui.colours.storage.count_stroke,
                },
            )],
        ));
        commands.spawn((
            ChildOf(cell),
            Node {
                height: Val::VMin(NAME_ROW),
                padding: UiRect::top(Val::VMin(0.7)),
                justify_content: JustifyContent::Center,
                overflow: Overflow::clip(),
                max_width: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
            children![window::words(ui.words.bag.name(thing.kind).to_string(), NAME_TEXT, ui.colours.bag.rarity(thing.rarity))],
        ));
    }
}

/// The box that asks how many ("How many?"): the thing's picture and its
/// name in its rarity's colour; the number, a minus to its left and a plus
/// to its right; the bar under them, as far along as the number is of
/// all you have, to drag; and OK.
fn counting(commands: &mut Commands, root: Entity, view: &View, kind: &'static str, count: u16) {
    let ui = ui();
    let colours = ui.colours.window;
    let inside = a_box(commands, root, 2, COUNT_WIDE, ui.words.storage.how_many.clone(), colours.title);
    let most = most_of(view.has, kind);
    let rarity = rules::thing(kind).map_or(colours.name, |thing| ui.colours.bag.rarity(thing.rarity));
    commands.spawn((
        ChildOf(inside),
        Node {
            column_gap: Val::VMin(2.4),
            align_items: AlignItems::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![
            window::picture(picture_of(view, kind), Val::VMin(COUNT_PICTURE), Color::WHITE),
            window::words(ui.words.bag.name(kind).to_string(), window::NAME, rarity),
        ],
    ));
    let step = |commands: &mut Commands, row: Entity, tap: Tap, image: Handle<Image>, able: bool| {
        commands.spawn((ChildOf(row), tap, window::round(image, STEP_BUTTON, able)));
    };
    let row = commands
        .spawn((
            ChildOf(inside),
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::VMin(5.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    step(commands, row, Tap::Less, view.pictures.minus.clone(), count > 1);
    commands.spawn((
        ChildOf(row),
        Node {
            min_width: Val::VMin(14.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![window::words(count.to_string(), NUMBER_TEXT, colours.name)],
    ));
    step(commands, row, Tap::More, view.pictures.plus.clone(), count < most);
    // The bar: its whole length, how far along the number is, and a knob
    // there. A press anywhere on it, or a drag along it, is that many.
    let storage = ui.colours.storage;
    let share = share_of(count, most);
    commands
        .spawn((
            ChildOf(inside),
            Bar,
            TakesPress,
            Node {
                height: Val::VMin(KNOB),
                margin: UiRect::horizontal(Val::VMin(KNOB * 0.5)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            children![
                (
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::VMin(BAR),
                        border_radius: BorderRadius::all(Val::VMin(BAR)),
                        ..default()
                    },
                    BackgroundColor(storage.bar),
                    Pickable::IGNORE,
                    children![(
                        Node {
                            width: Val::Percent(share * 100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::VMin(BAR)),
                            ..default()
                        },
                        BackgroundColor(storage.bar_fill),
                        Pickable::IGNORE,
                    )],
                ),
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(share * 100.0),
                        width: Val::VMin(KNOB),
                        height: Val::VMin(KNOB),
                        margin: UiRect::left(Val::VMin(-KNOB * 0.5)),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(colours.line),
                    Pickable::IGNORE,
                ),
            ],
        ))
        .observe(press_bar)
        .observe(drag_bar);
    commands.spawn((
        ChildOf(inside),
        window::choices(),
        children![(Tap::Ok, window::choice(ui.words.storage.ok.clone(), Look::Yes))],
    ));
}

// ---------------------------------------------------------------- said

/// Says across the screen what has come into your bag, taken out of
/// something that keeps things ("_____ x1 has been added to your
/// inventory").
fn say_taken(mut taken: MessageReader<Taken>, mut commands: Commands) {
    for Taken(stack) in taken.read() {
        let ui = ui();
        let words = fill(&ui.words.storage.added, &[("name", ui.words.bag.name(&stack.kind)), ("count", &stack.count.to_string())]);
        say(&mut commands, &words);
    }
}

/// A line across the screen, under its middle and over everything, for a
/// few seconds: the only one up.
fn say(commands: &mut Commands, words: &str) {
    commands.queue(|world: &mut World| {
        let before: Vec<Entity> = world.query_filtered::<Entity, With<Said>>().iter(world).collect();
        for line in before {
            world.despawn(line);
        }
    });
    commands.spawn((
        Said(0.0),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            right: Val::ZERO,
            top: Val::Vh(SAID_DOWN),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(DIALOG_LAYER + 6),
        Pickable::IGNORE,
        children![hud::words(words.to_string(), SAID_TEXT, ui().colours.text)],
    ));
}

/// Takes the line away once it has been up its while.
fn clear_said(time: Res<Time>, mut said: Query<(Entity, &mut Said)>, mut commands: Commands) {
    for (line, mut said) in &mut said {
        said.0 += time.delta_secs();
        if said.0 > SAID_SECS {
            commands.entity(line).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_thing_that_keeps_things_is_laid_out_as_many_slots_as_it_keeps() {
        for kind in ["cabinet", "dresser", "safe", "end_table", "barbecue"] {
            let (rows, across, sides) = layout(kind).unwrap();
            let keeps = rules::keeps(kind).unwrap();
            assert_eq!(usize::from(rows * across * sides), usize::from(keeps.slots), "{kind}");
        }
        // As Hajun has them: four on each side of the cabinet, six on each
        // of the dresser, three rows of three in the safe, one row of three
        // in the end table and the barbecue.
        assert_eq!(layout("cabinet"), Some((2, 2, 2)));
        assert_eq!(layout("dresser"), Some((3, 2, 2)));
        assert_eq!(layout("safe"), Some((3, 3, 1)));
        assert_eq!(layout("end_table"), Some((1, 3, 1)));
        assert_eq!(layout("barbecue"), Some((1, 3, 1)));
        assert_eq!(layout("desk"), None);
    }

    #[test]
    fn the_window_fits_a_phone_lying_on_its_side() {
        // The dresser's, the widest and tallest: four slots and the gap
        // across, three down, the title over them and the margins round it,
        // inside the short side's hundred shares; its boxes too.
        let wide = 4.0 * SLOT + 2.0 * SLOT_GAP + SIDE_GAP + 2.0 * window::PAD;
        let high = window::TAB_HEIGHT + UNDER_TITLE + 3.0 * SLOT + 2.0 * SLOT_GAP + 2.0 * window::PAD;
        assert!(wide < 200.0 - 2.0 * hud::EDGE && high < 100.0 - 2.0 * hud::EDGE, "{wide} by {high}");
        assert!(CHOOSE_HIGH < 100.0 - 2.0 * hud::EDGE && CHOOSE_WIDE < 133.0 - 2.0 * hud::EDGE);
        // A slot is pressed by a finger: more than a centimetre on a phone
        // whose short side is 6.5 cm.
        assert!(SLOT * 6.5 / 100.0 > 0.8);
    }

    #[test]
    fn how_many_goes_from_one_to_all_of_it_along_the_bar() {
        assert_eq!(count_along(0.0, 20), 1);
        assert_eq!(count_along(1.0, 20), 20);
        assert_eq!(count_along(0.5, 21), 11);
        assert_eq!(count_along(0.7, 1), 1);
        for count in [1, 7, 20] {
            assert_eq!(count_along(share_of(count, 20), 20), count);
        }
        // All of what is in the bag, up to as many as may be owned.
        let mut has = rules::Bag::new();
        has.take_in("hamburger", 5).unwrap();
        assert_eq!(most_of(&has, "hamburger"), 5);
        assert_eq!(most_of(&has, "doll"), 0);
    }

    #[test]
    fn the_barbecue_is_offered_food_alone_and_anything_else_all_that_is_in_the_bag() {
        let mut has = rules::Bag::new();
        has.take_in("hamburger", 2).unwrap();
        has.take_in("doll", 1).unwrap();
        let kinds = |has: &rules::Bag, food_only: bool| choosable(has, food_only).into_iter().map(|(thing, count)| (thing.kind, count)).collect::<Vec<_>>();
        assert_eq!(kinds(&has, true), [("hamburger", 2)]);
        let all = kinds(&has, false);
        for kind in ["hamburger", "doll", rules::LAMP, rules::CAR] {
            assert!(all.iter().any(|&(of, _)| of == kind), "{kind}: {all:?}");
        }
        // Nothing standing is offered: it is not in the bag.
        has.put(1, rules::LAMP, roundtown_net::At::Ground { x: 0.0, y: 0.0, z: 0.0, yaw: 0.0 }).unwrap();
        assert!(!kinds(&has, false).iter().any(|&(of, _)| of == rules::LAMP));
    }

    #[test]
    fn a_plus_and_a_minus_are_white_strokes_on_clear() {
        let at = |image: &Image, x: u32, y: u32| image.data.as_ref().unwrap()[((y * 96 + x) * 4 + 3) as usize];
        let (plus, minus) = (sign(true), sign(false));
        // Both have the stroke across through the middle; the plus has the
        // stroke down too; neither reaches the corners.
        assert_eq!((at(&plus, 48, 48), at(&minus, 48, 48)), (255, 255));
        assert_eq!((at(&plus, 48, 20), at(&minus, 48, 20)), (255, 0));
        assert_eq!((at(&plus, 2, 2), at(&minus, 2, 2)), (0, 0));
    }
}
