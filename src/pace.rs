//! How often the desktop build draws a frame: about 120 times a second.
//!
//! Bevy draws one every time the screen refreshes. On the laptop this was
//! written on that is 240 times a second, and every one of them is the whole
//! scene — four shadow cascades, the sea, the clouds, every player — drawn in
//! full, for motion that at 120 looks as smooth. Measured there on
//! 2026-09-27: at 240 frames a second the game took 168% of a CPU core and
//! the GPU drew 45 W; at 60 it took 50% and 23–28 W, next to the 22–24 W that
//! GPU draws with the game shut. 120 costs about twice what 60 does.
//!
//! So on a screen faster than 120 Hz, a frame is drawn every so many of its
//! refreshes rather than every one: every 2nd at 240 Hz. A whole number of
//! refreshes keeps every frame on screen for as long as every other, so motion
//! stays even. A screen at 144 Hz or slower is left as it was.
//!
//! Phones are not paced here. On Android `MainActivity` asks for the screen's
//! 120 Hz mode, which paces the frames at the screen itself; iOS holds an app
//! to 60 unless its `Info.plist` sets `CADisableMinimumFrameDurationOnPhone`.

use std::time::Duration;

use bevy::prelude::*;
use bevy::window::{Monitor, PrimaryMonitor};
use bevy::winit::{UpdateMode, WinitSettings};

/// About how many frames a second to draw on a fast screen.
const TARGET_FPS: f64 = 120.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, pace_to_screen);
}

/// Sets the frame rate from the screen's, once it is known and whenever it
/// changes. Until then Bevy's own settings stand.
fn pace_to_screen(
    monitors: Query<&Monitor, (With<PrimaryMonitor>, Changed<Monitor>)>,
    mut settings: ResMut<WinitSettings>,
) {
    let Ok(monitor) = monitors.single() else {
        return;
    };
    let wanted = match monitor.refresh_rate_millihertz.and_then(frame_every) {
        Some(wait) => {
            // Only the clock starts a frame. Input that comes in between is
            // kept for the next one; if it started frames of its own, moving
            // the mouse would draw one for every movement.
            let paced = UpdateMode::Reactive {
                wait,
                react_to_device_events: false,
                react_to_user_events: false,
                react_to_window_events: false,
            };
            WinitSettings {
                focused_mode: paced,
                unfocused_mode: paced,
            }
        }
        None => WinitSettings::game(),
    };
    if settings.focused_mode != wanted.focused_mode
        || settings.unfocused_mode != wanted.unfocused_mode
    {
        *settings = wanted;
    }
}

/// How long to leave between frames on a screen that refreshes `millihertz`
/// thousandths of a time a second: as many of its refreshes as bring it down
/// to about [`TARGET_FPS`]. `None` if that is just one, on a screen that
/// already refreshes at about that rate or slower.
fn frame_every(millihertz: u32) -> Option<Duration> {
    let hz = f64::from(millihertz) / 1000.0;
    // With some slack, so that a 239.76 Hz screen counts as a 240 Hz one.
    let refreshes = (hz / TARGET_FPS + 0.05).floor();
    (refreshes >= 2.0).then(|| Duration::from_secs_f64(refreshes / hz))
}
