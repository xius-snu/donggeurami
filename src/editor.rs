//! Tap something you can change and four buttons appear round it: a bin
//! above, a tick below to say it is where you want it, and a 45° turn either
//! side. Drag the thing itself, with your finger on it, to move it.
//!
//! The menu is the same for everything it opens on ([`Editable`]): the objects
//! of the town save, and the pieces of a house being built in House Builder. It
//! only says what was asked of the thing ([`Edit`]); whatever looks after that
//! kind of thing does it — the town's objects here, in [`edit_town`], a house's
//! pieces in `build`.
//!
//! One rule keeps this free of platform code: everything downstream of
//! [`EditPointer`] sees a position and three edges — pressed, held, released —
//! and nothing about where they came from. Touch fills it in
//! `read_touch_controls`, desktop in [`read_desktop_pointer`].
//!
//! It only reaches what is on the island you are standing on: the town's
//! objects are there to be tapped while you are in the town, not from home.
//!
//! Hit testing is done here rather than with `bevy_picking` so that a tap tests
//! the same shapes the rest of the game already knows about: the island and the
//! sea carry meshes but nothing [`Editable`], so a tap that lands on them is a
//! tap on nothing, which is exactly what should shut the menu.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::island::{Island, Islands, Venue};
use crate::map::{MapObject, Town};
use crate::{ColliderShape, LAND_STEP_UP, Player, ThirdPersonCamera};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::hud;

/// How far a press may travel and still count as a tap rather than a look drag.
const TAP_SLOP: f32 = 16.0;
/// Where the four buttons sit around the thing, and how big they are, as
/// shares of the short side of the screen — the same units the touch HUD uses.
const MENU_RADIUS_VMIN: f32 = 15.0;
const MENU_BUTTON_VMIN: f32 = 13.0;
/// The buttons go no higher on a thing that is big on the screen than this
/// much of the top of it, where the balance, House Builder's time and theme,
/// and the button top right are: a tap on the time skips the wait.
const MENU_BELOW_VMIN: f32 = 21.0;
/// Every tap on a turn button is this much, in degrees.
const ROTATE_STEP_DEG: f32 = 45.0;
/// The buttons are round a point on the thing no higher than this over its
/// bottom, in metres.
const ANCHOR_RISE: f32 = 1.0;
/// A drag is over a level at least this far under the camera, in metres.
const DRAG_UNDER_EYE: f32 = 0.5;
/// Dragged objects stay at least this far in from the water. The sea is not
/// somewhere to leave a tree.
const DROP_INLAND: f32 = 2.0;
/// A drag carries its object over the ground in steps no longer than this, so
/// that it climbs only what the player could and stops where the land does.
const CARRY_STEP: f32 = 0.25;
/// The furthest a drag carries its object in one frame. A finger near the
/// horizon can ask for somewhere hundreds of metres off; the object gets there
/// over a few frames instead of in one long stall.
const CARRY_MAX: f32 = 6.0;
/// The selection ring, relative to the object's tap footprint.
pub(crate) const RING_SCALE: f32 = 0.82;
const RING_THICKNESS: f32 = 0.12;
const RING_LIFT: f32 = 0.03;

const ICON: Color = Color::srgba(1.0, 1.0, 1.0, 0.92);

/// The editor proper, after whatever filled [`EditPointer`] on this platform.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EditSystems;

/// One pointer for the editor, whichever device is driving it.
#[derive(Resource, Default)]
pub(crate) struct EditPointer {
    /// Where the tracked pointer is, in logical window pixels.
    pub pos: Vec2,
    /// How far it moved since last frame.
    pub delta: Vec2,
    /// A press went down somewhere this frame — including on the joystick or
    /// the jump button, which is what lets those dismiss the menu.
    pub pressed_anywhere: bool,
    /// The tracked pointer went down this frame.
    pub pressed: bool,
    pub held: bool,
    /// The tracked pointer came up this frame.
    pub released: bool,
    /// How far the running press has travelled, so that a tap can be told from
    /// a drag.
    pub travel: f32,
    /// There is no pointer at all just now — desktop took the cursor back for
    /// looking around, or it left the window. Nothing can press a button, so
    /// nothing should be waiting to be pressed.
    pub lost: bool,
}

impl EditPointer {
    /// Start of frame: last frame's edges are over.
    pub(crate) fn clear_edges(&mut self) {
        self.pressed_anywhere = false;
        self.pressed = false;
        self.released = false;
        self.delta = Vec2::ZERO;
    }

    /// Follow the pointer, counting how far this press has run.
    pub(crate) fn track(&mut self, pos: Vec2) {
        self.delta = pos - self.pos;
        self.travel += self.delta.length();
        self.pos = pos;
    }

    /// Put the pointer somewhere without counting it as travel. Touch pointers
    /// appear out of nowhere; a mouse cursor is always somewhere already.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn warp(&mut self, pos: Vec2) {
        self.delta = Vec2::ZERO;
        self.pos = pos;
    }

    pub(crate) fn press(&mut self) {
        self.pressed = true;
        self.pressed_anywhere = true;
        self.held = true;
        self.travel = 0.0;
    }

    /// A press the editor is not tracking — the joystick, or the jump button.
    /// Only touch has presses it does not follow; a mouse has the one.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn note_press(&mut self) {
        self.pressed_anywhere = true;
    }

    pub(crate) fn release(&mut self) {
        self.released = true;
        self.held = false;
    }
}

/// Something the edit menu can open on, and how.
#[derive(Component, Clone, Copy)]
pub(crate) struct Editable {
    /// What a tap on it, or a press to drag it, is tested against, in its own
    /// space.
    pub touch: Touch,
    /// How far out the ring drawn under it while it is selected reaches, or 0
    /// for none.
    pub ring: f32,
    /// Whether it has turn buttons: what hangs on a wall has none.
    pub turns: bool,
    /// Once open on it, the menu stays until it is confirmed or thrown away,
    /// whatever else is pressed: it is still being put down.
    pub sticky: bool,
}

/// Out of a tap's reach for now, though still [`Editable`]: a tap goes through
/// it to whatever is behind, or to nothing. Whatever looks after that kind of
/// thing says when; a menu already open on it stays open.
#[derive(Component)]
pub(crate) struct Untappable;

/// The shape a tap on something is tested against, in its own space.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Touch {
    /// A cylinder standing on the origin: a trunk and its canopy.
    Upright { radius: f32, height: f32 },
    /// A box between two corners.
    Block { low: Vec3, high: Vec3 },
}

impl Touch {
    /// What a tap on something of `shape` is tested against: the same shape,
    /// a box sitting about the origin the way the collision code reads one.
    pub(crate) fn of(shape: ColliderShape) -> Self {
        match shape {
            ColliderShape::Cylinder { radius, height } => Self::Upright { radius, height },
            ColliderShape::Aabb { half_extents } => Self::Block {
                low: -half_extents,
                high: half_extents,
            },
        }
    }

    /// The lowest and highest corners of the box it fits in.
    fn span(self) -> (Vec3, Vec3) {
        match self {
            Self::Upright { radius, height } => (
                Vec3::new(-radius, 0.0, -radius),
                Vec3::new(radius, height, radius),
            ),
            Self::Block { low, high } => (low, high),
        }
    }

    /// The corners of the box it fits in.
    fn corners(self) -> [Vec3; 8] {
        let (low, high) = self.span();
        std::array::from_fn(|i| {
            Vec3::new(
                if i & 1 == 0 { low.x } else { high.x },
                if i & 2 == 0 { low.y } else { high.y },
                if i & 4 == 0 { low.z } else { high.z },
            )
        })
    }
}

/// What the edit menu asks of the thing it is open on. Whatever looks after
/// that kind of thing does it, and ignores what is asked of anything else.
#[derive(Message, Clone, Copy, Debug)]
pub(crate) enum Edit {
    /// A tap has just opened the menu on it.
    Opened(Entity),
    /// It is being dragged, the finger now being over `ray`: it should go to
    /// `to`, or as near there as it can. `to` is on the level it was taken hold
    /// of at, and there is none while the finger is somewhere that level is
    /// not, over the horizon from it; what goes on a wall follows `ray` alone,
    /// and never waits for it.
    Drag {
        target: Entity,
        to: Option<Vec3>,
        ray: Ray3d,
    },
    /// Let go of, after a drag.
    Dropped(Entity),
    /// Turned about the upright by this many radians.
    Turn(Entity, f32),
    /// Thrown away.
    Trash(Entity),
    /// Where it is, it is to stay.
    Confirm(Entity),
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum EditButton {
    Trash,
    Confirm,
    RotateRight,
    RotateLeft,
}

impl EditButton {
    const ALL: [Self; 4] = [
        Self::Trash,
        Self::Confirm,
        Self::RotateRight,
        Self::RotateLeft,
    ];

    /// Which way out from the thing this button sits, in screen directions.
    fn offset(self) -> Vec2 {
        match self {
            Self::Trash => Vec2::new(0.0, -1.0),
            Self::Confirm => Vec2::new(0.0, 1.0),
            Self::RotateRight => Vec2::new(1.0, 0.0),
            Self::RotateLeft => Vec2::new(-1.0, 0.0),
        }
    }

    fn turns(self) -> bool {
        matches!(self, Self::RotateRight | Self::RotateLeft)
    }

    fn fill(self, held: bool) -> Color {
        let alpha = if held { 0.62 } else { 0.36 };
        match self {
            Self::Trash => Color::srgba(1.0, 0.36, 0.33, alpha),
            Self::Confirm => Color::srgba(0.36, 0.84, 0.46, alpha),
            Self::RotateRight | Self::RotateLeft => Color::srgba(0.36, 0.66, 1.0, alpha),
        }
    }

    fn edge(self) -> Color {
        match self {
            Self::Trash => Color::srgba(1.0, 0.5, 0.47, 0.62),
            Self::Confirm => Color::srgba(0.6, 1.0, 0.68, 0.62),
            Self::RotateRight | Self::RotateLeft => Color::srgba(0.55, 0.78, 1.0, 0.62),
        }
    }
}

/// What the running press on the menu is doing.
#[derive(Clone, Copy, Debug)]
enum Held {
    /// It came down on a button, which goes off if it comes up there too.
    Button(EditButton),
    /// It came down on the thing itself, standing at `base`, and is dragging
    /// it `offset` from the point under the finger at `height`: the height of
    /// the point on it that was pressed, so that that point stays under the
    /// finger however high up something as big as a house it is. With the
    /// finger not over that level when it came down, `offset` is taken the
    /// first time it is.
    Thing {
        base: f32,
        height: f32,
        offset: Option<Vec2>,
    },
}

/// The open menu, if there is one.
#[derive(Resource, Default)]
pub(crate) struct EditMenu {
    /// The thing the four buttons belong to.
    target: Option<Entity>,
    /// What the buttons are arranged around, in logical window pixels: the
    /// thing itself, wherever it is on the screen, and that includes off the
    /// edge of it: the buttons turn away with the thing rather than stay behind.
    anchor: Vec2,
    /// Whether any of the buttons can be seen: the thing is in view, or near
    /// enough to it that they reach on to the screen. Out of view they are
    /// neither drawn nor pressable, and `anchor` is out of date.
    shown: bool,
    /// Where the thing is on the screen, for a press on it to drag it by.
    grip: Option<Rect>,
    /// Whether the thing has turn buttons.
    turns: bool,
    /// Whether the menu stays until the thing is confirmed or thrown away.
    sticky: bool,
    held: Option<Held>,
    /// Set when a press has just dismissed the menu, so that the release which
    /// follows it does not open one straight back up.
    swallow_release: bool,
}

impl EditMenu {
    pub(crate) fn is_open(&self) -> bool {
        self.target.is_some()
    }

    /// Whether a press at `pos` is on one of the menu's buttons, and so the
    /// menu's alone. Only touch has to ask: a mouse that is free to press
    /// anything is not looking round.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn on_button(&self, window: &Window, pos: Vec2) -> bool {
        self.button_at(window, pos).is_some()
    }

    /// Whether a press at `pos` is on the thing the menu is open on, to drag
    /// it by: the menu's too, unless the stick or the jump button is there.
    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(crate) fn on_thing(&self, pos: Vec2) -> bool {
        self.grip.is_some_and(|grip| grip.contains(pos))
    }

    /// Which button, if any, is under `pos`. A little larger than the drawn
    /// circle, the same way the joystick and the jump button are.
    pub(crate) fn button_at(&self, window: &Window, pos: Vec2) -> Option<EditButton> {
        self.target?;
        if !self.shown {
            return None;
        }
        let metrics = Metrics::of(window);
        EditButton::ALL.into_iter().find(|button| {
            (self.turns || !button.turns())
                && pos.distance(self.anchor + button.offset() * metrics.ring)
                    <= metrics.button * 0.59
        })
    }

    /// Opens the menu on `target`, wherever it is on the screen.
    pub(crate) fn open_on(&mut self, target: Entity) {
        self.target = Some(target);
        self.shown = false;
        self.grip = None;
        self.held = None;
    }

    fn close(&mut self) {
        self.target = None;
        self.shown = false;
        self.grip = None;
        self.held = None;
    }

    /// Keeps the menu on `thing`, which stands at `place`, wherever it is on
    /// the screen.
    fn follow(
        &mut self,
        window: &Window,
        camera: &Camera,
        eye: &GlobalTransform,
        thing: &Editable,
        place: &GlobalTransform,
    ) {
        self.turns = thing.turns;
        self.sticky = thing.sticky;
        let corners = thing
            .touch
            .corners()
            .map(|corner| place.transform_point(corner));
        // Behind the camera, it cannot be pressed on, and its buttons are as
        // far out of sight as it is.
        let Some(shown) = on_screen(camera, eye, corners) else {
            self.grip = None;
            self.shown = false;
            return;
        };
        // However small it is on the screen, there is a fingertip's worth of
        // it to press on.
        let metrics = Metrics::of(window);
        let grow = (Vec2::splat(metrics.button) - shown.size()).max(Vec2::ZERO) * 0.5;
        self.grip = Some(Rect::from_corners(shown.min - grow, shown.max + grow));
        // Round its middle, but never more than a little way up it: on a tree
        // or a house the buttons stay down by the trunk or the door, rather
        // than up among the time and the balance along the top of the screen.
        // Wherever that is, and no pulling it in to the screen: turned away
        // from it, the buttons leave the screen with it. With its middle
        // behind the camera, as it is from inside a house seen through your
        // own eyes, round as much of it as is on the screen.
        let (bottom, top) = thing.touch.span();
        let middle = (bottom + top) * 0.5;
        let rise = ((top.y - bottom.y) * 0.5).min(ANCHOR_RISE);
        let focus = place.transform_point(Vec3::new(middle.x, bottom.y + rise, middle.z));
        let at = camera.world_to_viewport(eye, focus).ok().or_else(|| {
            let screen = Rect::new(0.0, 0.0, window.width(), window.height());
            visible_middle(shown, screen)
        });
        // A house up the screen would have its bin over the time, so it is
        // ringed lower down, but only as far down as the house itself goes:
        // one that has gone up off the screen is not held down where it has
        // gone from, nor pulled across, and its buttons go with it.
        let clear_of_the_top = metrics.below_the_top(window).min(shown.max.y);
        let at = at.map(|at| Vec2::new(at.x, at.y.max(clear_of_the_top)));
        self.shown = at.is_some_and(|at| metrics.reaches_screen(window, at));
        if let Some(at) = at {
            self.anchor = at;
        }
    }
}

/// The menu's size for this window, in pixels.
struct Metrics {
    /// How far each button sits from the anchor.
    ring: f32,
    /// A button's width and height.
    button: f32,
}

impl Metrics {
    fn of(window: &Window) -> Self {
        let vmin = window.width().min(window.height());
        Self {
            ring: vmin * (MENU_RADIUS_VMIN / 100.0),
            button: vmin * (MENU_BUTTON_VMIN / 100.0),
        }
    }

    /// How far down the screen the middle of the buttons is to be for the
    /// bin, above it, to be below the balance, the time and the theme.
    fn below_the_top(&self, window: &Window) -> f32 {
        let vmin = window.width().min(window.height()) / 100.0;
        MENU_BELOW_VMIN * vmin + self.ring + self.button * 0.5
    }

    /// Whether buttons round `anchor` reach on to the screen at all. Past that
    /// they are out of sight, however far off `anchor` is.
    fn reaches_screen(&self, window: &Window, anchor: Vec2) -> bool {
        let reach = self.ring + self.button * 0.5;
        anchor.x > -reach
            && anchor.x < window.width() + reach
            && anchor.y > -reach
            && anchor.y < window.height() + reach
    }
}

/// The middle of the part of `shown` that is on `screen`, or `None` if none of
/// it is.
fn visible_middle(shown: Rect, screen: Rect) -> Option<Vec2> {
    let (min, max) = (shown.min.max(screen.min), shown.max.min(screen.max));
    min.cmple(max).all().then(|| (min + max) * 0.5)
}

/// The rectangle of the screen, in logical window pixels, that the box with
/// these `corners` is drawn in, or `None` if none of it is in front of the
/// camera. A box that reaches back past the camera — a house right in front of
/// you, seen through your own eyes while you look down at the floor — is cut
/// off where the camera starts drawing, and the cut runs off the edges of the
/// screen, the way the box does.
fn on_screen(camera: &Camera, eye: &GlobalTransform, corners: [Vec3; 8]) -> Option<Rect> {
    // How far in front of the camera each corner is, and how far in front
    // drawing starts: the near plane, where Bevy's depth is 1, and a whisker
    // further, so that no rounding puts a cut behind it.
    let view = eye.affine().inverse();
    let ahead = corners.map(|corner| -view.transform_point3(corner).z);
    let near = -camera.depth_ndc_to_view_z(1.0) * 1.01;
    let mut drawn = Vec::new();
    for (i, &corner) in corners.iter().enumerate() {
        if ahead[i] >= near {
            drawn.push(corner);
        }
        // Where each edge out of this corner comes into sight, if it does.
        // The corners at their other ends are one bit of `i` away
        // (`Touch::corners`), and going only up the bits takes each edge once.
        for j in [i | 1, i | 2, i | 4] {
            if j != i && (ahead[i] >= near) != (ahead[j] >= near) {
                let t = (near - ahead[i]) / (ahead[j] - ahead[i]);
                drawn.push(corner.lerp(corners[j], t));
            }
        }
    }
    let mut seen = drawn
        .into_iter()
        .filter_map(|point| camera.world_to_viewport(eye, point).ok());
    let first = seen.next()?;
    Some(seen.fold(Rect::from_corners(first, first), |shown, point| {
        shown.union_point(point)
    }))
}

/// Everything a tap might land on.
type Things<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Editable,
        &'static GlobalTransform,
        &'static InheritedVisibility,
        Has<Untappable>,
    ),
>;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<EditPointer>()
        .init_resource::<EditMenu>()
        .add_message::<Edit>()
        .add_systems(Startup, setup_menu)
        .add_systems(
            Update,
            (
                put_away,
                drive_menu,
                edit_town,
                sync_selection_ring,
                layout_menu,
            )
                .chain()
                .in_set(EditSystems)
                .after(crate::follow_camera),
        );

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    app.add_systems(Update, read_desktop_pointer.before(EditSystems));
}

// ---------------------------------------------------------------- the pointer

/// Desktop points with the mouse cursor, so editing is live exactly while the
/// cursor is free — which is the state `Escape` already toggles into. While it
/// is grabbed the cursor is hidden and pinned to the middle of the screen,
/// where nothing but the player's own back is ever under it, so there is
/// nothing to aim at and nothing to click.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn read_desktop_pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cursors: Query<&CursorOptions, With<PrimaryWindow>>,
    presses: hud::Presses,
    mut pointer: ResMut<EditPointer>,
) {
    pointer.clear_edges();
    let free = cursors
        .single()
        .is_ok_and(|cursor| cursor.grab_mode == CursorGrabMode::None);
    let window = windows.single().ok().filter(|_| free);
    let position = window.and_then(Window::cursor_position);
    let (Some(window), Some(position)) = (window, position) else {
        // Cursor grabbed, or gone off the window mid-drag. Let go of whatever
        // was being held rather than leaving it stuck down.
        if pointer.held {
            pointer.release();
        }
        pointer.lost = true;
        return;
    };

    pointer.lost = false;
    pointer.track(position);
    // A click on a button over the world — the one top right, House
    // Builder's — is the button's, not a tap on the town.
    if mouse.just_pressed(MouseButton::Left) && !presses.claimed(window, position) {
        pointer.press();
    }
    // So only a press that was taken here comes back up here.
    if mouse.just_released(MouseButton::Left) && pointer.held {
        pointer.release();
    }
}

// ----------------------------------------------------------------- the editor

fn drive_menu(
    pointer: Res<EditPointer>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    things: Things,
    mut menu: ResMut<EditMenu>,
    mut edits: MessageWriter<Edit>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    // The camera has no parent, so this *is* its global transform — and it is
    // this frame's, not the one transform propagation published last frame.
    let eye = GlobalTransform::from(*camera_transform);

    // Keep up with the thing the menu is open on: gone, and the menu goes too.
    if let Some(target) = menu.target {
        match things.get(target) {
            Ok((_, thing, place, ..)) => menu.follow(window, camera, &eye, thing, place),
            Err(_) => menu.close(),
        }
    }
    // With nothing left to press its buttons with, a menu that need not wait
    // for an answer goes.
    if pointer.lost && menu.is_open() && !menu.sticky {
        shut(&mut menu, &mut edits);
    }

    let ray = camera.viewport_to_world(&eye, pointer.pos).ok();
    if pointer.pressed {
        let on_thing = menu.grip.is_some_and(|grip| grip.contains(pointer.pos));
        if let Some(button) = menu.button_at(window, pointer.pos) {
            menu.held = Some(Held::Button(button));
        } else if on_thing
            && let Some(target) = menu.target
            && let Ok((_, thing, place, ..)) = things.get(target)
            && let Some(ray) = ray
        {
            // Where the press went into it, or, just beside it, its foot; but
            // never so high that the finger would be dragging it over a level
            // the camera is not looking down on.
            let at = place.translation();
            let height = ray_hit(ray, place, thing.touch)
                .map_or(at.y, |far| ray.get_point(far).y)
                .min(ray.origin.y - DRAG_UNDER_EYE)
                .max(at.y);
            menu.held = Some(Held::Thing {
                base: at.y,
                height,
                offset: level_point(ray, height).map(|under| at.xz() - under.xz()),
            });
        } else if menu.is_open() && !menu.sticky {
            // A press anywhere else puts the menu away, unless it is waiting
            // for the thing to be put down.
            shut(&mut menu, &mut edits);
            menu.swallow_release = true;
        }
    } else if pointer.pressed_anywhere && menu.is_open() && !menu.sticky {
        // A press the menu is not following: the joystick, the jump button.
        shut(&mut menu, &mut edits);
    }

    // Dragged, the thing goes where the finger does, the point pressed on it
    // staying under the finger; whatever looks after it keeps it on the
    // ground, or on its wall. Every move of the finger is passed on, over the
    // level it was taken hold of at or not, so that what goes on a wall, which
    // follows the finger up and down it, never stops short.
    if pointer.held
        && pointer.delta != Vec2::ZERO
        && let Some(Held::Thing {
            base,
            height,
            offset,
        }) = menu.held
        && let Some(target) = menu.target
        && let Some(ray) = ray
    {
        let under = level_point(ray, height);
        let offset = offset.or_else(|| {
            let (_, _, place, ..) = things.get(target).ok()?;
            Some(place.translation().xz() - under?.xz())
        });
        menu.held = Some(Held::Thing {
            base,
            height,
            offset,
        });
        let to = under
            .zip(offset)
            .map(|(under, offset)| Vec3::new(under.x + offset.x, base, under.z + offset.y));
        edits.write(Edit::Drag { target, to, ray });
    }

    if !pointer.released {
        return;
    }
    let swallowed = std::mem::take(&mut menu.swallow_release);
    match menu.held.take() {
        Some(Held::Thing { .. }) => {
            if let Some(target) = menu.target {
                edits.write(Edit::Dropped(target));
            }
        }
        // A button goes off on a release inside the circle it came down on,
        // so that sliding off one cancels it.
        Some(Held::Button(button)) if menu.button_at(window, pointer.pos) == Some(button) => {
            press(button, &mut menu, &mut edits);
        }
        Some(Held::Button(_)) => {}
        None if !swallowed && !pointer.lost && pointer.travel <= TAP_SLOP && !menu.is_open() => {
            if let Some(ray) = ray
                && let Some(target) = pick(&things, ray)
            {
                menu.open_on(target);
                edits.write(Edit::Opened(target));
            }
        }
        None => {}
    }
}

/// Puts the menu away, letting go of whatever was being dragged.
fn shut(menu: &mut EditMenu, edits: &mut MessageWriter<Edit>) {
    if let (Some(Held::Thing { .. }), Some(target)) = (menu.held, menu.target) {
        edits.write(Edit::Dropped(target));
    }
    menu.close();
}

/// Does what `button` is for, to whatever the menu is open on.
fn press(button: EditButton, menu: &mut EditMenu, edits: &mut MessageWriter<Edit>) {
    let Some(target) = menu.target else {
        return;
    };
    match button {
        EditButton::Trash => {
            edits.write(Edit::Trash(target));
            menu.close();
        }
        EditButton::Confirm => {
            edits.write(Edit::Confirm(target));
            menu.close();
        }
        // A positive turn about Y always swings the face nearest the camera to
        // the camera's right, whichever way the camera is orbited, so "right"
        // means the same thing from every angle.
        EditButton::RotateRight => {
            edits.write(Edit::Turn(target, ROTATE_STEP_DEG.to_radians()));
        }
        EditButton::RotateLeft => {
            edits.write(Edit::Turn(target, -ROTATE_STEP_DEG.to_radians()));
        }
    }
}

/// You have gone to another island, and whatever was being edited has stayed
/// behind.
fn put_away(
    moved: Query<(), (With<Player>, Changed<Venue>)>,
    mut menu: ResMut<EditMenu>,
    mut edits: MessageWriter<Edit>,
) {
    if moved.is_empty() || !menu.is_open() {
        return;
    }
    shut(&mut menu, &mut edits);
}

/// What the menu asks of the town's own objects: moved over the town's ground,
/// turned, thrown away. Every change is written to the save.
fn edit_town(
    mut edits: MessageReader<Edit>,
    islands: Res<Islands>,
    mut objects: Query<&mut Transform, With<MapObject>>,
    mut town: ResMut<Town>,
    mut commands: Commands,
) {
    for &edit in edits.read() {
        match edit {
            Edit::Drag { target, to, .. } => {
                if let Some(to) = to
                    && let Some(island) = islands.get(Venue::Town)
                    && let Ok(mut transform) = objects.get_mut(target)
                {
                    let at = carry(island, transform.translation, to.xz(), Quat::IDENTITY, |_| true);
                    if transform.translation != at {
                        transform.translation = at;
                    }
                }
            }
            Edit::Dropped(target) if objects.contains(target) => town.touch(),
            Edit::Turn(target, angle) => {
                if let Ok(mut transform) = objects.get_mut(target) {
                    transform.rotate_y(angle);
                    town.touch();
                }
            }
            Edit::Trash(target) if objects.contains(target) => {
                commands.entity(target).despawn();
                town.touch();
            }
            _ => {}
        }
    }
}

/// The nearest thing that can be edited under `ray`. Distance from the player
/// does not come into it: if you can see it well enough to tap it, you can
/// change it. And if you cannot see it — it is on another island — you cannot,
/// nor anything [`Untappable`] just now.
fn pick(things: &Things, ray: Ray3d) -> Option<Entity> {
    let mut best: Option<(f32, Entity)> = None;
    for (entity, thing, place, shown, untappable) in things.iter() {
        if !shown.get() || untappable {
            continue;
        }
        let Some(distance) = ray_hit(ray, place, thing.touch) else {
            continue;
        };
        if best.is_none_or(|(nearest, _)| distance < nearest) {
            best = Some((distance, entity));
        }
    }
    best.map(|(_, entity)| entity)
}

/// Where `ray` meets the level at `height`.
fn level_point(ray: Ray3d, height: f32) -> Option<Vec3> {
    let point =
        ray.plane_intersection_point(Vec3::new(0.0, height, 0.0), InfinitePlane3d::new(Vec3::Y))?;
    point.is_finite().then_some(point)
}

/// Where a dragged object ends up when the finger asks for `goal`. It is
/// carried there over the ground in short steps, the way the player would walk
/// it: up and over the bridge, but never off the land into the sea, nor
/// anywhere `fits` says it cannot stand. Stopped by something, it goes right
/// up to it and then slides along it instead of stopping dead: along a wall,
/// or along the shore where the land runs out. It slides across or along the
/// way `square` turns it, so that something turned to stand square to a wall,
/// or to another thing, slides along it.
pub(crate) fn carry(
    island: &Island,
    from: Vec3,
    goal: Vec2,
    square: Quat,
    fits: impl Fn(Vec3) -> bool,
) -> Vec3 {
    // Somewhere a drag could never have left it: out at sea, most likely,
    // where a town saved against an older map can have put it. It follows the
    // finger freely until it is back where it can be.
    if drop_height(island, from.xz(), from.y).is_none() || !fits(from) {
        let y = drop_height(island, goal, from.y).unwrap_or(from.y);
        return Vec3::new(goal.x, y, goal.y);
    }
    let path = (goal - from.xz()).clamp_length_max(CARRY_MAX);
    let steps = (path.length() / CARRY_STEP).ceil().max(1.0);
    let step = path / steps;
    // Where it would stand `slide` on from `at`, if it may.
    let stand = |at: Vec3, slide: Vec2| {
        let spot = at.xz() + slide;
        drop_height(island, spot, at.y)
            .map(|y| Vec3::new(spot.x, y, spot.y))
            .filter(|&next| fits(next))
    };
    let (across, along) = ((square * Vec3::X).xz(), (square * Vec3::Z).xz());
    let sides = [across * step.dot(across), along * step.dot(along)];
    let mut at = from;
    let mut stopped = false;
    for _ in 0..steps as u32 {
        if let Some(next) = stand(at, step) {
            at = next;
            stopped = false;
            continue;
        }
        if !stopped {
            at = up_to(at, step, stand);
            stopped = true;
        }
        match sides.into_iter().find_map(|slide| stand(at, slide)) {
            Some(next) => at = next,
            // In a corner: up to both sides of it.
            None => {
                for slide in sides {
                    at = up_to(at, slide, stand);
                }
                break;
            }
        }
    }
    at
}

/// As far from `at` toward `way` as `stand` lets it go, short of all the way,
/// found to within a few millimetres by halving.
fn up_to(at: Vec3, way: Vec2, stand: impl Fn(Vec3, Vec2) -> Option<Vec3>) -> Vec3 {
    const HALVINGS: u32 = 6;
    let (mut short, mut far) = (0.0, 1.0);
    let mut reached = at;
    for _ in 0..HALVINGS {
        let middle = (short + far) * 0.5;
        match stand(at, way * middle) {
            Some(next) => {
                reached = next;
                short = middle;
            }
            None => far = middle,
        }
    }
    reached
}

/// The height an object carried from the height `from` would stand at on
/// `spot`, or `None` if it may not be left there: the ground has to be within
/// a step of it, and there has to be land [`DROP_INLAND`] out on every side.
pub(crate) fn drop_height(island: &Island, spot: Vec2, from: f32) -> Option<f32> {
    let reach = from + LAND_STEP_UP;
    let inland = [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
        .into_iter()
        .all(|side| island.floor(spot + side * DROP_INLAND, reach).is_some());
    if !inland {
        return None;
    }
    island.floor(spot, reach)
}

/// How far along `ray` the shape standing at `place` is first hit, or `None`
/// if the ray misses it.
fn ray_hit(ray: Ray3d, place: &GlobalTransform, shape: Touch) -> Option<f32> {
    // Into the thing's own space, so that its turn is accounted for. Nothing
    // in this world is scaled, so distances come back unchanged and stay
    // comparable between things.
    let inverse = place.affine().inverse();
    let origin = inverse.transform_point3(ray.origin);
    let direction = inverse.transform_vector3(*ray.direction);
    match shape {
        // Cylinders stand on the entity's origin, the way the collision code
        // reads them.
        Touch::Upright { radius, height } => entry(overlap(
            tube(origin, direction, radius)?,
            slab(origin.y, direction.y, 0.0, height)?,
        )?),
        Touch::Block { low, high } => {
            let x = slab(origin.x, direction.x, low.x, high.x)?;
            let y = slab(origin.y, direction.y, low.y, high.y)?;
            let z = slab(origin.z, direction.z, low.z, high.z)?;
            entry(overlap(overlap(x, y)?, z)?)
        }
    }
}

/// The stretch of a ray inside one volume, as distances along it.
type Span = (f32, f32);

/// Where the ray is between two parallel planes on one axis.
fn slab(origin: f32, direction: f32, min: f32, max: f32) -> Option<Span> {
    if direction.abs() <= 1e-6 {
        return (origin >= min && origin <= max).then_some((f32::NEG_INFINITY, f32::INFINITY));
    }
    let near = (min - origin) / direction;
    let far = (max - origin) / direction;
    Some((near.min(far), near.max(far)))
}

/// Where the ray is inside a cylinder of unbounded height about the Y axis.
fn tube(origin: Vec3, direction: Vec3, radius: f32) -> Option<Span> {
    let a = direction.x * direction.x + direction.z * direction.z;
    if a <= 1e-6 {
        let inside = origin.x * origin.x + origin.z * origin.z <= radius * radius;
        return inside.then_some((f32::NEG_INFINITY, f32::INFINITY));
    }
    let b = 2.0 * (origin.x * direction.x + origin.z * direction.z);
    let c = origin.x * origin.x + origin.z * origin.z - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }
    let root = discriminant.sqrt();
    Some(((-b - root) / (2.0 * a), (-b + root) / (2.0 * a)))
}

fn overlap(a: Span, b: Span) -> Option<Span> {
    let span = (a.0.max(b.0), a.1.min(b.1));
    (span.0 <= span.1).then_some(span)
}

/// Where the ray first meets a shape: where it goes in, or, starting inside
/// it, where it comes back out. Something you are inside of — a house, from
/// one of its rooms — is then behind whatever is in there with you, not in
/// front of everything.
fn entry(span: Span) -> Option<f32> {
    if span.1 < 0.0 {
        return None;
    }
    Some(if span.0 >= 0.0 { span.0 } else { span.1 })
}

// ------------------------------------------------------------- what you see

/// The ring drawn under whatever is being edited.
#[derive(Component)]
struct SelectionRing {
    target: Entity,
}

fn sync_selection_ring(
    menu: Res<EditMenu>,
    rings: Query<(Entity, &SelectionRing)>,
    things: Query<&Editable>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let wanted = menu
        .target
        .filter(|&target| things.get(target).is_ok_and(|thing| thing.ring > 0.0));
    if rings.iter().next().map(|(_, ring)| ring.target) == wanted {
        return;
    }
    for (entity, _) in &rings {
        commands.entity(entity).try_despawn();
    }
    let Some(target) = wanted else {
        return;
    };
    let Ok(thing) = things.get(target) else {
        return;
    };

    let radius = thing.ring;
    let half = radius * RING_THICKNESS * 0.5;
    // Unlit and blended, the same as everything else translucent in this world:
    // `base_color` is the whole story, and both platforms draw it identically.
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.95, 0.68, 0.85),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });

    // A child of the object, so that it follows every drag and every turn.
    commands.entity(target).with_children(|parent| {
        parent
            .spawn((
                SelectionRing { target },
                Mesh3d(meshes.add(Annulus::new(radius - half, radius + half).mesh())),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, RING_LIFT, 0.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                NotShadowCaster,
            ))
            // Which way the object faces. Without it a 45° turn on something as
            // round as a tree is nearly impossible to see.
            .with_child((
                Mesh3d(meshes.add(Circle::new(half * 2.4).mesh())),
                MeshMaterial3d(material),
                Transform::from_xyz(0.0, radius, 0.002),
                NotShadowCaster,
            ));
    });
}

#[derive(Component)]
struct EditMenuRoot;

fn setup_menu(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let turn = images.add(turn_image());
    commands
        .spawn((
            EditMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|menu| {
            for button in EditButton::ALL {
                menu.spawn((
                    button,
                    Node {
                        position_type: PositionType::Absolute,
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(button.fill(false)),
                    BorderColor::all(button.edge()),
                ))
                .with_children(|icon| match button {
                    EditButton::Trash => {
                        icon.spawn(icon_part(40.0, 16.0, 20.0, 7.0, 50.0, 0.0));
                        icon.spawn(icon_part(21.0, 25.0, 58.0, 9.0, 50.0, 0.0));
                        icon.spawn(icon_part(29.0, 38.0, 42.0, 42.0, 18.0, 0.0));
                    }
                    // A tick: a short stroke down and a long one up.
                    EditButton::Confirm => {
                        icon.spawn(icon_part(24.0, 53.0, 24.0, 11.0, 50.0, 45.0));
                        icon.spawn(icon_part(33.0, 44.0, 46.0, 11.0, 50.0, -50.0));
                    }
                    // The usual circular arrow, clockwise for the right button
                    // and its mirror image for the left.
                    EditButton::RotateRight => {
                        icon.spawn(turn_arrow(&turn, false));
                    }
                    EditButton::RotateLeft => {
                        icon.spawn(turn_arrow(&turn, true));
                    }
                });
            }
        });
}

/// One white shape inside a button, in percentages of the button's own box —
/// the same way the jump arrow on the touch HUD is drawn. `rotation` is
/// clockwise.
fn icon_part(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    radius: f32,
    rotation: f32,
) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(left),
            top: Val::Percent(top),
            width: Val::Percent(width),
            height: Val::Percent(height),
            border_radius: BorderRadius::all(Val::Percent(radius)),
            ..default()
        },
        BackgroundColor(ICON),
        UiTransform::from_rotation(Rot2::degrees(rotation)),
    )
}

/// The circular arrow in the middle of a turn button, `flipped` for the one
/// that turns the other way.
fn turn_arrow(image: &Handle<Image>, flipped: bool) -> impl Bundle {
    let mut arrow = ImageNode::new(image.clone()).with_color(ICON);
    arrow.flip_x = flipped;
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(14.0),
            top: Val::Percent(14.0),
            width: Val::Percent(72.0),
            height: Val::Percent(72.0),
            ..default()
        },
        arrow,
    )
}

/// A circle open at the right with an arrowhead on its top end, going
/// clockwise, white on clear, for the turn buttons to tint. Drawn here for the
/// same reason as the rating star (`builder::star_image`): the built-in font
/// has no such arrow, and edged by how far each pixel is from it, it is smooth
/// at whatever size a button is.
fn turn_image() -> Image {
    const SIZE: u32 = 192;
    // Shares of half the image.
    const RADIUS: f32 = 0.6;
    /// Half the width of the stroke.
    const STROKE: f32 = 0.12;
    const HEAD_WIDTH: f32 = 0.3;
    const HEAD_LENGTH: f32 = 0.36;
    // Where the stroke starts and ends, in degrees clockwise from twelve
    // o'clock: the arrowhead is on the end, up by one, and the start is round
    // past three, leaving the gap between them.
    const START_DEG: f32 = 115.0;
    const END_DEG: f32 = 30.0;
    let half = SIZE as f32 * 0.5;
    let (radius, stroke) = (half * RADIUS, half * STROKE);
    let on_ring = |angle: f32| radius * Vec2::new(angle.sin(), angle.cos());
    let (start, end) = (START_DEG.to_radians(), END_DEG.to_radians());
    let sweep = (end - start).rem_euclid(TAU);
    // The head stands across the ring where the stroke ends and points on
    // round it, to a tip on the ring.
    let out = Vec2::new(end.sin(), end.cos());
    let base = on_ring(end);
    let (wing_out, wing_in) = (
        base + out * half * HEAD_WIDTH,
        base - out * half * HEAD_WIDTH,
    );
    let tip = on_ring(end + half * HEAD_LENGTH / radius);
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            // From the middle, in pixels, with y up.
            let at = Vec2::new(column as f32 + 0.5 - half, half - (row as f32 + 0.5));
            let along = (at.x.atan2(at.y) - start).rem_euclid(TAU);
            let to_stroke = if along <= sweep {
                (at.length() - radius).abs()
            } else {
                at.distance(on_ring(start)).min(at.distance(base))
            } - stroke;
            let outside = to_stroke.min(triangle_distance(at, wing_out, wing_in, tip));
            let cover = (0.5 - outside).clamp(0.0, 1.0);
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

/// How far `at` is from the edge of the triangle `a`, `b`, `c`: less than
/// nothing inside it. Inigo Quilez's `sdTriangle`.
fn triangle_distance(at: Vec2, a: Vec2, b: Vec2, c: Vec2) -> f32 {
    let (e0, e1, e2) = (b - a, c - b, a - c);
    let (v0, v1, v2) = (at - a, at - b, at - c);
    let q0 = v0 - e0 * (v0.dot(e0) / e0.dot(e0)).clamp(0.0, 1.0);
    let q1 = v1 - e1 * (v1.dot(e1) / e1.dot(e1)).clamp(0.0, 1.0);
    let q2 = v2 - e2 * (v2.dot(e2) / e2.dot(e2)).clamp(0.0, 1.0);
    let side = (e0.x * e2.y - e0.y * e2.x).signum();
    let nearest = Vec2::new(q0.dot(q0), side * (v0.x * e0.y - v0.y * e0.x))
        .min(Vec2::new(q1.dot(q1), side * (v1.x * e1.y - v1.y * e1.x)))
        .min(Vec2::new(q2.dot(q2), side * (v2.x * e2.y - v2.y * e2.x)));
    -nearest.x.sqrt() * nearest.y.signum()
}

fn layout_menu(
    menu: Res<EditMenu>,
    windows: Query<&Window>,
    mut root: Query<&mut Node, (With<EditMenuRoot>, Without<EditButton>)>,
    mut buttons: Query<(&EditButton, &mut Node, &mut BackgroundColor), Without<EditMenuRoot>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut root) = root.single_mut() else {
        return;
    };
    if !menu.is_open() || !menu.shown {
        edit_node(&mut root, |root| root.display = Display::None);
        return;
    }

    let metrics = Metrics::of(window);
    edit_node(&mut root, |root| {
        root.display = Display::Flex;
        root.left = Val::Px(menu.anchor.x);
        root.top = Val::Px(menu.anchor.y);
    });
    let held = match menu.held {
        Some(Held::Button(button)) => Some(button),
        _ => None,
    };
    for (button, mut node, mut color) in &mut buttons {
        let centre = button.offset() * metrics.ring;
        edit_node(&mut node, |node| {
            node.display = if menu.turns || !button.turns() {
                Display::Flex
            } else {
                Display::None
            };
            node.left = Val::Px(centre.x - metrics.button * 0.5);
            node.top = Val::Px(centre.y - metrics.button * 0.5);
            node.width = Val::Px(metrics.button);
            node.height = Val::Px(metrics.button);
        });
        color.set_if_neq(BackgroundColor(button.fill(held == Some(*button))));
    }
}

/// Applies `edit` to `node` only if it changes something. A changed node has
/// the whole screen's layout worked out again, and this runs every frame,
/// with the menu shut as much as open.
fn edit_node(node: &mut Mut<Node>, edit: impl FnOnce(&mut Node)) {
    let mut edited = node.clone();
    edit(&mut edited);
    node.set_if_neq(edited);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OrbitCamera;
    use bevy::camera::{CameraProjection, RenderTargetInfo};
    use bevy::window::WindowResolution;

    fn window() -> Window {
        Window {
            resolution: WindowResolution::new(1280, 720),
            ..default()
        }
    }

    /// The camera through your own eyes, over you standing at the origin,
    /// turned the way it starts, looking down at the floor.
    fn first_person(window: &Window) -> (Camera, GlobalTransform) {
        let orbit = OrbitCamera::default();
        let mut camera = Camera::default();
        camera.computed.clip_from_view = PerspectiveProjection {
            fov: orbit.fov,
            aspect_ratio: window.width() / window.height(),
            ..default()
        }
        .get_clip_from_view();
        camera.computed.target_info = Some(RenderTargetInfo {
            physical_size: window.physical_size(),
            scale_factor: window.scale_factor(),
        });
        let eye = crate::eye_view(&orbit, &Transform::default(), None);
        (camera, GlobalTransform::from(eye))
    }

    /// The cottage as a ghost: 6.7 m square and 4.9 m to the top of its roof.
    fn cottage() -> Editable {
        Editable {
            touch: Touch::Block {
                low: Vec3::new(-3.35, 0.0, -3.35),
                high: Vec3::new(3.35, 4.9, 3.35),
            },
            ring: 0.0,
            turns: true,
            sticky: true,
        }
    }

    /// The menu open on the cottage standing at `at`.
    fn menu_on(window: &Window, camera: &Camera, eye: &GlobalTransform, at: Vec3) -> EditMenu {
        let mut menu = EditMenu::default();
        menu.open_on(Entity::PLACEHOLDER);
        let place = GlobalTransform::from_translation(at);
        menu.follow(window, camera, eye, &cottage(), &place);
        menu
    }

    /// Whether all four of the menu's buttons are on the screen, whole.
    fn all_on_screen(menu: &EditMenu, window: &Window) -> bool {
        let metrics = Metrics::of(window);
        let screen = Rect::new(0.0, 0.0, window.width(), window.height());
        let reach = Vec2::splat(metrics.button * 0.5);
        EditButton::ALL.into_iter().all(|button| {
            let centre = menu.anchor + button.offset() * metrics.ring;
            screen.contains(centre - reach) && screen.contains(centre + reach)
        })
    }

    #[test]
    fn a_house_right_in_front_of_you_has_its_menu_round_it() {
        // Straight ahead, its near side 1.05 m from your middle, which is as
        // near as one can be dragged. The near corners of its roof are behind
        // your eyes.
        let window = window();
        let (camera, eye) = first_person(&window);
        let at = Vec3::new(0.0, 0.0, -4.4);
        let corners = cottage().touch.corners().map(|corner| corner + at);
        let behind = |&corner: &Vec3| camera.world_to_viewport(&eye, corner).is_err();
        assert!(corners.iter().any(behind));

        let menu = menu_on(&window, &camera, &eye, at);
        assert!(all_on_screen(&menu, &window), "{}", menu.anchor);
        // Round the house, which is straight ahead.
        let off_middle = (menu.anchor.x - window.width() * 0.5).abs();
        assert!(off_middle < 1.0, "{}", menu.anchor);
        // It fills the view, so a press in the middle of the screen is on it.
        let middle = Vec2::new(window.width(), window.height()) * 0.5;
        assert!(menu.grip.is_some_and(|grip| grip.contains(middle)));
    }

    #[test]
    fn from_inside_a_house_the_menu_is_round_what_you_can_see() {
        // Facing its front wall from just inside it: its middle is behind you.
        let window = window();
        let (camera, eye) = first_person(&window);
        let menu = menu_on(&window, &camera, &eye, Vec3::new(0.0, 0.0, 2.0));
        assert!(all_on_screen(&menu, &window), "{}", menu.anchor);
        assert!(menu.grip.is_some());
    }

    /// Where the buttons go round the cottage standing at `at`: the point on
    /// it they are anchored to, on the screen, however far off the screen.
    fn focus_on_screen(camera: &Camera, eye: &GlobalTransform, at: Vec3) -> Vec2 {
        camera
            .world_to_viewport(eye, at + Vec3::Y * ANCHOR_RISE)
            .unwrap()
    }

    #[test]
    fn behind_you_the_buttons_are_gone() {
        let window = window();
        let (camera, eye) = first_person(&window);
        let mut menu = menu_on(&window, &camera, &eye, Vec3::new(0.0, 0.0, -20.0));
        assert!(menu.shown);
        let anchor = menu.anchor;
        let behind = GlobalTransform::from_xyz(0.0, 0.0, 20.0);
        menu.follow(&window, &camera, &eye, &cottage(), &behind);
        // Not drawn, and not somewhere to press where they were.
        assert!(!menu.shown);
        assert_eq!(menu.button_at(&window, anchor), None);
        assert!(menu.grip.is_none());
    }

    #[test]
    fn turned_away_the_buttons_go_off_the_screen_with_it() {
        let window = window();
        let (camera, eye) = first_person(&window);
        let width = window.width();
        // Slide the cottage out through the right side of the view.
        let mut last = 0.0;
        let mut out_of_sight = false;
        for step in 0..200 {
            let at = Vec3::new(step as f32 * 0.25, 0.0, -20.0);
            let menu = menu_on(&window, &camera, &eye, at);
            let focus = focus_on_screen(&camera, &eye, at);
            if menu.shown {
                // Always on the cottage itself, never pulled in to the screen.
                assert!((menu.anchor.x - focus.x).abs() < 0.01, "{at}");
                assert!(menu.grip.is_some_and(|grip| grip.contains(menu.anchor)));
                last = menu.anchor.x;
            } else {
                out_of_sight = true;
                assert_eq!(menu.button_at(&window, focus), None);
            }
        }
        // It went off the edge on its way, buttons and all...
        assert!(last > width, "{last}");
        // ...and once no button could reach the screen it was put away.
        assert!(out_of_sight);
    }

    #[test]
    fn a_small_far_thing_keeps_its_buttons_up_on_it() {
        // Far off, the cottage is a small thing up near the horizon, in the
        // top third of the screen where the time and the balance are. The
        // buttons stay on it, and are not pulled down to clear them.
        let window = window();
        let (camera, eye) = first_person(&window);
        let at = Vec3::new(0.0, 0.0, -60.0);
        let menu = menu_on(&window, &camera, &eye, at);
        let focus = focus_on_screen(&camera, &eye, at);
        assert!(focus.y < window.height() * 0.3, "{focus}");
        assert!(menu.shown);
        assert!(menu.grip.is_some_and(|grip| grip.contains(menu.anchor)));
        let clear = Metrics::of(&window).below_the_top(&window);
        assert!(menu.anchor.y < clear - 50.0, "{}", menu.anchor);
    }

    /// The bin's top edge is below the balance, the time and the theme.
    fn bin_clear_of_the_top(menu: &EditMenu, window: &Window) -> bool {
        let metrics = Metrics::of(window);
        let vmin = window.width().min(window.height()) / 100.0;
        menu.anchor.y - metrics.ring - metrics.button * 0.5 >= MENU_BELOW_VMIN * vmin - 0.5
    }

    #[test]
    fn a_house_as_it_comes_up_has_its_bin_clear_of_the_time() {
        // Where the shop puts a house: straight ahead of you on the ring
        // round the middle of the plot, in third person and through your
        // own eyes. Its middle is high up the screen, and a tap on the time
        // skips the wait, so the bin must not be over it.
        let window = window();
        let you = crate::island::arrival(0);
        let cottage_ahead = you.translation
            + Vec3::NEG_Z * (crate::PLAYER_RADIUS + crate::build::HOUSE_IN_FRONT + 3.35);
        let orbit = OrbitCamera::default();

        let (first, _) = first_person(&window);
        let first_eye = GlobalTransform::from(crate::eye_view(&orbit, &you, None));

        // With nothing in the way, as on the open grass of a plot.
        let (third_view, _) =
            crate::place_camera(&orbit, &you, 0.0, None, crate::CameraFit::default(), 0.0);
        let mut third = first.clone();
        third.computed.clip_from_view = PerspectiveProjection {
            fov: crate::CAMERA_FOV,
            aspect_ratio: window.width() / window.height(),
            ..default()
        }
        .get_clip_from_view();
        let third_eye = GlobalTransform::from(third_view);

        for (name, camera, eye) in [("first", &first, &first_eye), ("third", &third, &third_eye)] {
            let menu = menu_on(&window, camera, eye, cottage_ahead);
            assert!(menu.shown, "{name}");
            assert!(
                bin_clear_of_the_top(&menu, &window),
                "{name}: {}",
                menu.anchor
            );
            assert!(all_on_screen(&menu, &window), "{name}: {}", menu.anchor);
        }
    }
}
