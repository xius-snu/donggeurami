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
//! Only Android says anything: on desktop there is no hole, and
//! `RT_SAFE_AREA=<left>,<right>`, in logical pixels, makes one up to look
//! at. **An iPhone is not asked** (winit 0.30 has its safe area, but
//! nothing behind `target_os = "ios"` can be checked on this PC), so an
//! iPhone's island still lies over whatever is beside it.

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

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(asked_for()).add_systems(Update, keep_clear);
    #[cfg(target_os = "android")]
    app.add_systems(PreUpdate, read);
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
/// each side: written as the safe area changes, as the window's size does,
/// and on whatever has just been put up, and not otherwise.
fn keep_clear(
    area: Res<SafeArea>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<&mut Node, With<KeepsClear>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let (width, vmin) = (window.width(), window.width().min(window.height()));
    let left = Val::Px(further_in(area.left, width, vmin).round());
    let right = Val::Px(further_in(area.right, width, vmin).round());
    for mut node in &mut nodes {
        if node.margin.left != left || node.margin.right != right {
            node.margin.left = left;
            node.margin.right = right;
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
}
