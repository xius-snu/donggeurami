//! The part of the screen that nothing of the phone's own is over: its safe
//! area (since 2026-11-14; Hajun: "in my mobile Samsung phone, the chat box
//! is covered slightly, uh, and the input field too, by the, uh, small
//! camera hole. Uh, so I guess I need the UI elements to use the OS safe
//! area, so that the left margin is just for the UI elements is shifted to
//! the safe area so it's not covered").
//!
//! The game is drawn edge to edge, under the camera's hole too
//! (`MainActivity.applyImmersive`: the window lies in the display's cutout
//! along its short edges). Held on its side, the phone has the hole half
//! way down its left edge, or its right one, turned the other way up. How
//! far in from each of those edges the hole reaches is Android's to say
//! (the display's cutout, in the window's insets), and `MainActivity` hands
//! it over ([`INSETS`]), as it does the keyboard's height.
//!
//! **What moves** ([`KeepsClear`]): what stands [`hud::EDGE`] in from the
//! side of the screen, which is where the hole is: the bar across the top
//! (your name and your balance, or in Bridge Wars your health and what you
//! hold, on the left; the buttons down the right edge, or the minimap) and
//! the chat. Each is moved in until it is [`CLEAR`] of the hole's side of
//! the safe area, by a margin, and no further: on a screen with no hole, or
//! one whose hole ends short of them, they are where they were. **What does
//! not**: the stick and the buttons round the jump button, which stand 7
//! and 6 parts in a hundred of the screen's width in, where a hole in the
//! edge of a phone ends 3 or 4 in (reckoned from how tall a status bar is,
//! not measured on one), and a thumb knows where they are; and whatever
//! comes up in the middle of the screen.
//!
//! **An iPhone says too** (since 2026-11-15; Hajun: "for android the OS
//! safe area thing is working fine, but for iphone 16 pro, the camera hole
//! is not just a circle but a bar so it also covers the amethyst icon and
//! the chat text as well in the brdidge wars game. check for ios"). Until
//! then it was never asked, and nothing moved on one. Its island is a bar,
//! as long as a third of the screen's short side, half way down the left
//! edge or the right one, and iOS's safe area begins the same way in from
//! both of those edges whichever the island is on (62 points on a 16 Pro,
//! whose island ends 51 in), so on an iPhone both sides are moved in. The
//! window says what it is ([`read_the_window`]): winit's "inner" position
//! and size of a window are, on iOS, its safe area's.
//!
//! **Two things follow from a safe area as deep as that.** The right end
//! of the bar across the top is moved in no further than there is room for
//! ([`RightRoom`]): moved the whole way it ran into the things to say,
//! fanned out of the speech bubble, and what is at that end of it (the
//! buttons down the right edge, or the minimap) is above where an island
//! lies and, moved as far as it is, clear of the notch older iPhones have.
//! And the chat's small button moves ([`StepsIn`]), which hangs out of the
//! right of the box round the jump button, nearer the edge than the rest
//! of it, and half way down, where an island on the right lies: it steps
//! in, and no further than keeps it clear of the buttons about it.
//!
//! On desktop there is no hole, and `RT_SAFE_AREA=<left>,<right>`, in
//! logical pixels, makes one up to look at.

use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::hud;

/// How far inside the safe area's edge what is moved keeps, as a share of
/// the short side of the screen: the hole is round, and the safe area ends
/// at the hole's very edge.
const CLEAR: f32 = 1.5;
/// The most of the screen's width a hole is believed to take up on one
/// side, as a share of it: whatever says more is taken to say this.
const MOST: f32 = 0.2;
/// How often an iPhone's window is asked what its safe area is, in seconds:
/// as often as Android's activity says ([`INSETS`]). It is the same held
/// either way up, so there is nothing to catch quickly; it is asked again
/// at all because the window may not know it yet when it is first asked.
const ASKED_EVERY: f32 = 0.3;

/// How far in from the left and the right of the screen the safe area
/// begins, in the window's logical pixels: nothing where there is no hole.
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub(crate) struct SafeArea {
    pub left: f32,
    pub right: f32,
}

/// Something laid out [`hud::EDGE`] in from the left of the screen, the
/// right, or both, which is kept clear of the camera's hole
/// ([`keep_clear`]): its node is given a margin on either side, and has to
/// have none of its own there.
#[derive(Component, Default)]
pub(crate) struct KeepsClear;

/// How far at the most something [`KeepsClear`] is moved in from the right,
/// where there is something further in for it to run into: `vw` parts in a
/// hundred of the screen's width and `vmin` of its short side, and not at
/// all where that comes to less than nothing. Without one it is moved as
/// far as the safe area says.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) struct RightRoom {
    pub vw: f32,
    pub vmin: f32,
}

impl RightRoom {
    /// The room there is on a screen `width` across, whose short side is
    /// `vmin`, in logical pixels.
    fn most(&self, width: f32, vmin: f32) -> f32 {
        (width * self.vw / 100.0 + vmin * self.vmin / 100.0).max(0.0)
    }
}

/// Something that stands by the right of the screen where it is put, not
/// [`hud::EDGE`] in, and steps in from there when the safe area reaches it
/// ([`keep_clear`]): `vw` parts in a hundred of the screen's width and
/// `vmin` of its short side in from the right (less than nothing: out past
/// what it is measured from), and moved, by a margin on its right, until it
/// is [`CLEAR`] of the safe area's edge or has gone `most` shares of the
/// short side, whichever comes first. Its node has to have no margin of its
/// own there.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) struct StepsIn {
    pub vw: f32,
    pub vmin: f32,
    pub most: f32,
}

impl StepsIn {
    /// How far it steps in on a screen `width` across, whose short side is
    /// `vmin`, for a safe area that begins `inset` in from the right, all in
    /// logical pixels: nothing with no hole, or with one that ends short of
    /// it.
    fn in_by(&self, inset: f32, width: f32, vmin: f32) -> f32 {
        let inset = inset.clamp(0.0, width * MOST);
        if inset <= 0.0 {
            return 0.0;
        }
        let stands = width * self.vw / 100.0 + vmin * self.vmin / 100.0;
        (inset + vmin * CLEAR / 100.0 - stands).clamp(0.0, (vmin * self.most / 100.0).max(0.0))
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(asked_for()).add_systems(Update, keep_clear);
    #[cfg(target_os = "android")]
    app.add_systems(PreUpdate, read);
    #[cfg(target_os = "ios")]
    app.add_systems(PreUpdate, read_the_window);
}

/// The safe area a desktop is told to make up, for looking at what a hole
/// does: `RT_SAFE_AREA=<left>,<right>`, in logical pixels. None anywhere
/// else, until the phone says ([`read`]).
fn asked_for() -> SafeArea {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if let Ok(asked) = std::env::var("RT_SAFE_AREA") {
        let mut sides = asked.split(',').map(|side| side.trim().parse::<f32>().unwrap_or(0.0).max(0.0));
        return SafeArea {
            left: sides.next().unwrap_or(0.0),
            right: sides.next().unwrap_or(0.0),
        };
    }
    SafeArea::default()
}

/// How far in from the left and the right of the screen the display's
/// cutout reaches, on Android, in the screen's own pixels: what
/// `MainActivity` reads off the window's insets and hands over through
/// [`safe_insets`], every third of a second while the game is up, so that
/// a phone turned the other way up moves the margin to the other side.
/// Numbers only, as the keyboard's height is handed over
/// (`signup::IME_INSET`).
#[cfg(target_os = "android")]
static INSETS: [std::sync::atomic::AtomicI32; 2] =
    [std::sync::atomic::AtomicI32::new(0), std::sync::atomic::AtomicI32::new(0)];

/// `MainActivity.safeInsets(int, int)`: the activity, on its own thread,
/// saying how far in the camera's hole reaches from the left and from the
/// right. JNI finds it by this name.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_safeInsets(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    left: i32,
    right: i32,
) {
    INSETS[0].store(left, std::sync::atomic::Ordering::Relaxed);
    INSETS[1].store(right, std::sync::atomic::Ordering::Relaxed);
}

/// The safe area, on Android: what the activity last said ([`INSETS`]), in
/// the screen's pixels, which the game draws fewer of (`RENDER_SCALE`),
/// brought to the window's logical pixels as the keyboard's height is
/// (`signup::read_keyboard`). The log is told what it is to begin with and
/// each time it changes (`adb logcat`, "safe area"): the first thing to
/// read if something is still under the hole on a phone.
#[cfg(target_os = "android")]
fn read(mut area: ResMut<SafeArea>, windows: Query<&Window, With<PrimaryWindow>>, mut said: Local<bool>) {
    let Ok(window) = windows.single() else {
        return;
    };
    let pixels = |side: usize| INSETS[side].load(std::sync::atomic::Ordering::Relaxed).max(0);
    let logical = |side: usize| pixels(side) as f32 * crate::RENDER_SCALE / window.scale_factor();
    let now = SafeArea {
        left: logical(0),
        right: logical(1),
    };
    if !*said || *area != now {
        *said = true;
        info!(
            "the safe area begins {} px in from the left and {} from the right ({} and {} of the screen's own)",
            now.left,
            now.right,
            pixels(0),
            pixels(1)
        );
    }
    area.set_if_neq(now);
}

/// The safe area, on an iPhone: what its window says of itself, asked
/// [`ASKED_EVERY`] so often. winit 0.30 gives a window an "outer" position
/// and size, which on iOS are the whole window's, and an "inner" one, which
/// there are its safe area's (`UIWindow.safeAreaInsets` taken off its
/// bounds): how far right of the one the other begins is the left inset,
/// and what is left over of the width is the right one. Bevy has the window
/// the whole screen (winit's `Resized` is the whole of it), so the insets
/// are brought to its logical pixels as shares of its width ([`within`]).
///
/// The window is the main thread's, where winit keeps it and UIKit wants it
/// asked: hence the marker, though every schedule runs there as it is
/// (`one_thread_per_world`). The log is told what was read to begin with
/// and each time it changes (the iPhone's `log.txt`, "safe area"): the first
/// thing to read if something is still under the island.
///
/// Nothing here is the iPhone's alone to compile: winit has these four on
/// every platform, and only where this is added is (`plugin`). **Not yet
/// run on an iPhone**: what winit does there is read off its source.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
fn read_the_window(
    _on_the_main_thread: NonSendMarker,
    time: Res<Time<Real>>,
    mut wait: Local<f32>,
    mut said: Local<bool>,
    mut area: ResMut<SafeArea>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
) {
    *wait -= time.delta_secs();
    if *wait > 0.0 {
        return;
    }
    *wait = ASKED_EVERY;
    let Ok((entity, window)) = windows.single() else {
        return;
    };
    let told = bevy::winit::WINIT_WINDOWS.with(|all| {
        let all = all.try_borrow().ok()?;
        let its = all.get_window(entity)?;
        let (inner, outer) = (its.inner_position().ok()?, its.outer_position().ok()?);
        Some(((inner.x - outer.x) as f32, its.inner_size().width as f32, its.outer_size().width as f32))
    });
    let Some((left, safe, whole)) = told else {
        return;
    };
    let now = within(left, safe, whole, window.width());
    if !*said || *area != now {
        *said = true;
        info!(
            "the safe area begins {} px in from the left and {} from the right (the window says {left} in, {safe} across, of its {whole})",
            now.left, now.right
        );
    }
    area.set_if_neq(now);
}

/// The safe area of a window `width` logical pixels across, from what winit
/// says of it in pixels of its own: its safe area begins `left` in from its
/// left edge and is `safe` across, of the `whole` the window is.
fn within(left: f32, safe: f32, whole: f32, width: f32) -> SafeArea {
    if whole.is_nan() || whole <= 0.0 {
        return SafeArea::default();
    }
    let logical = width / whole;
    let left = left.clamp(0.0, whole);
    let right = (whole - safe - left).clamp(0.0, whole);
    SafeArea {
        left: left * logical,
        right: right * logical,
    }
}

/// How much further in than [`hud::EDGE`] something by the side of a
/// screen `width` across, whose short side is `vmin`, has to stand to be
/// [`CLEAR`] of a safe area that begins `inset` in from that side, all in
/// logical pixels: nothing with no hole, or with one that ends short of
/// it.
fn further_in(inset: f32, width: f32, vmin: f32) -> f32 {
    let inset = inset.clamp(0.0, width * MOST);
    if inset <= 0.0 {
        return 0.0;
    }
    (inset + vmin * (CLEAR - hud::EDGE) / 100.0).max(0.0)
}

/// Keeps whatever [`KeepsClear`] clear of the camera's hole, by a margin on
/// each side, and whatever [`StepsIn`] by one on its right: written as the
/// safe area changes, as the window's size does, and on whatever has just
/// been put up, and not otherwise.
fn keep_clear(
    area: Res<SafeArea>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<(&mut Node, Option<&RightRoom>), With<KeepsClear>>,
    mut stepping: Query<(&StepsIn, &mut Node), Without<KeepsClear>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (width, vmin) = (window.width(), window.width().min(window.height()));
    let left = Val::Px(further_in(area.left, width, vmin).round());
    let whole = further_in(area.right, width, vmin);
    for (mut node, room) in &mut nodes {
        let right = Val::Px(room.map_or(whole, |room| whole.min(room.most(width, vmin))).round());
        if node.margin.left != left || node.margin.right != right {
            node.margin.left = left;
            node.margin.right = right;
        }
    }
    for (steps, mut node) in &mut stepping {
        let in_by = Val::Px(steps.in_by(area.right, width, vmin).round());
        if node.margin.right != in_by {
            node.margin.right = in_by;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_by_the_edge_is_moved_just_clear_of_the_hole_and_no_further() {
        // A phone lying on its side, 2,340 by 1,080 logical pixels at one to
        // one: the short side is 1,080, and the edge is 43.2 in.
        let (width, vmin) = (2340.0, 1080.0);
        let edge = vmin * hud::EDGE / 100.0;
        let clear = vmin * CLEAR / 100.0;
        // No hole: nothing moves, and what a margin of nothing is has to be
        // what a node has to begin with, so that it is not written.
        assert_eq!(further_in(0.0, width, vmin), 0.0);
        assert_eq!(Val::Px(0.0), UiRect::default().left);
        // A hole that ends 75 in: what was 43.2 in is now clear of it.
        let moved = edge + further_in(75.0, width, vmin);
        assert!((moved - (75.0 + clear)).abs() < 1e-3, "{moved}");
        // One that ends well short of the edge's own margin moves nothing.
        assert_eq!(further_in(edge - clear - 1.0, width, vmin), 0.0);
        // And the deeper the hole, the further in, never the other way.
        let mut last = 0.0;
        for inset in 0..300 {
            let extra = further_in(inset as f32, width, vmin);
            assert!(extra >= last, "{inset}: {extra} after {last}");
            last = extra;
        }
        // Something absurd is not believed past a fifth of the screen.
        assert!(further_in(1.0e6, width, vmin) <= width * MOST);
        // The stick and the buttons round the jump button are left where
        // they are, being further in than a hole reaches: one as deep as
        // this, 75 of 2,340, ends 3.2 parts in a hundred of the width in.
        assert!(crate::STICK_LEFT_VW > 5.0 && crate::JUMP_RIGHT_VW > 5.0);
    }

    #[test]
    fn an_iphone_s_window_says_where_its_safe_area_is() {
        // A 16 Pro lying on its side: 874 by 402 points, three pixels to a
        // point, and a safe area 62 points in from both sides whichever the
        // island is on. winit says the window is 2,622 pixels across, its
        // safe area 2,250 of them, beginning 186 in.
        assert_eq!(within(186.0, 2250.0, 2622.0, 874.0), SafeArea { left: 62.0, right: 62.0 });
        // One with a notch, 47 points in: 844 points across.
        let notch = within(141.0, 2250.0, 2532.0, 844.0);
        assert!((notch.left - 47.0).abs() < 1e-3 && (notch.right - 47.0).abs() < 1e-3, "{notch:?}");
        // An iPad, whose window is all safe from side to side.
        assert_eq!(within(0.0, 2732.0, 2732.0, 1366.0), SafeArea::default());
        // A hole on one side only is on that side only.
        assert_eq!(within(100.0, 1900.0, 2000.0, 1000.0), SafeArea { left: 50.0, right: 0.0 });
        assert_eq!(within(0.0, 1900.0, 2000.0, 1000.0), SafeArea { left: 0.0, right: 50.0 });
        // The window's own pixels need not be the screen's: shares of its
        // width are what is kept.
        assert_eq!(within(93.0, 1125.0, 1311.0, 874.0), SafeArea { left: 62.0, right: 62.0 });
        // A window that is not there yet, or says nonsense, has none; and
        // an inset is never less than nothing.
        assert_eq!(within(0.0, 0.0, 0.0, 874.0), SafeArea::default());
        assert_eq!(within(10.0, 100.0, f32::NAN, 874.0), SafeArea::default());
        assert_eq!(within(-30.0, 2700.0, 2622.0, 874.0), SafeArea::default());
    }

    #[test]
    fn on_an_iphone_the_bar_and_the_chat_stand_clear_of_the_island_on_both_sides() {
        // The 16 Pro again: its island ends 51 points in from the side it
        // is on, and iOS's safe area 62 in from both.
        let (width, vmin, island) = (874.0, 402.0, 51.0);
        let area = within(186.0, 2250.0, 2622.0, width);
        let edge = vmin * hud::EDGE / 100.0;
        for inset in [area.left, area.right] {
            let stands = edge + further_in(inset, width, vmin);
            // Inside the safe area, and so well clear of the island.
            assert!(stands > 62.0 && stands > island + 10.0, "{stands}");
            // And not a fifth of the way across the screen for it.
            assert!(stands < width * 0.1, "{stands}");
        }
    }

    #[test]
    fn the_right_of_the_bar_is_moved_in_no_further_than_its_room() {
        // The room the bar has on its right, as `hud` gives it: on a 16 Pro,
        // 874 by 402 points, some 18 points, where the safe area asks 52 of
        // what stands at the edge.
        let room = hud::bar_room();
        let (width, vmin) = (874.0, 402.0);
        let (asked, most) = (further_in(62.0, width, vmin), room.most(width, vmin));
        assert!(asked > 50.0 && most > 15.0 && most < 22.0, "{asked} asked, {most} the most");
        // (That what stands at that end is clear of a notch by then is
        // `hud`'s to say, which knows where it stands.)
        // A hole no deeper than a phone's round one is given all it asks.
        assert!(further_in(75.0, 2340.0, 1080.0) < room.most(2340.0, 1080.0));
        // Where there is no room there is none, never less.
        assert_eq!(room.most(1024.0, 768.0), 0.0);
    }

    #[test]
    fn the_chat_s_button_steps_in_from_an_island_and_no_further_than_it_may() {
        let button = StepsIn {
            vw: crate::JUMP_RIGHT_VW,
            vmin: hud::CHAT_BUTTON.0,
            most: hud::CHAT_STEPS_IN,
        };
        // On a 16 Pro it stands 44.4 points in from the right, under an
        // island on that side, which ends 51 in.
        let (width, vmin, island) = (874.0, 402.0, 51.0);
        let stands = width * button.vw / 100.0 + vmin * button.vmin / 100.0;
        assert!(stands < island, "{stands}");
        // The safe area, 62 in, asks for more than it may go: it goes as
        // far as it may, and that is clear of the island.
        let stepped = button.in_by(62.0, width, vmin);
        assert!((stepped - vmin * button.most / 100.0).abs() < 1e-3, "{stepped}");
        assert!(stands + stepped > island + 3.0, "{}", stands + stepped);
        // No hole, no step; and a hole that ends short of it, as a phone's
        // round one does (75 of 2,340: the button stands 119 in), none.
        assert_eq!(button.in_by(0.0, width, vmin), 0.0);
        assert_eq!(button.in_by(75.0, 2340.0, 1080.0), 0.0);
        // A hole a little deeper than the button stands moves it just clear
        // of it, as the bar is moved.
        let clear = 1080.0 * CLEAR / 100.0;
        let just = button.in_by(110.0, 2340.0, 1080.0);
        assert!((2340.0 * button.vw / 100.0 + 1080.0 * button.vmin / 100.0 + just - (110.0 + clear)).abs() < 1e-2, "{just}");
        // Deeper is further in, never the other way, and never past the
        // most.
        let mut last = 0.0;
        for inset in 0..400 {
            let step = button.in_by(inset as f32, 2340.0, 1080.0);
            assert!(step >= last && step <= 1080.0 * button.most / 100.0 + 1e-3, "{inset}: {step} after {last}");
            last = step;
        }
    }
}
