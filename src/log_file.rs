//! What the game logs, in a file the phone itself can show: on an iPhone,
//! `log.txt` in the app's Documents, which the Files app lists under On My
//! iPhone > Donggeurami Town (`UIFileSharingEnabled` in `Info.plist`).
//!
//! iOS keeps an app's own log where only a Mac can read it, and there is no
//! Mac here (`IOS.md`). So the phone writes its own: everything Bevy logs, from
//! which GPU it found to every warning, and any panic with the thread it was
//! on and where. A panic on the render thread leaves the screen black rather
//! than closing the app, since iOS does not let an app quit, which is what
//! 1.0.1 did (2026-10-08). Each launch starts the file afresh.

use std::backtrace::Backtrace;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use bevy::log::BoxedLayer;
use bevy::log::tracing_subscriber::{Layer, fmt};
use bevy::prelude::*;

/// What the file is called.
const NAME: &str = "log.txt";

/// For `LogPlugin::custom_layer` on an iPhone: everything logged, and any
/// panic, written to the app's Documents.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
pub(crate) fn in_documents(_app: &mut App) -> Option<BoxedLayer> {
    let path = Path::new(&std::env::var_os("HOME")?).join("Documents").join(NAME);
    let layer = writing_to(&path)?;
    keep_panics_in(&path);
    Some(layer)
}

/// A layer that writes everything logged to the file at `path`, emptied
/// first.
fn writing_to(path: &Path) -> Option<BoxedLayer> {
    std::fs::create_dir_all(path.parent()?).ok()?;
    File::create(path).ok()?;
    // Appended to, like the panics, so that neither writes over the other.
    let file = OpenOptions::new().append(true).open(path).ok()?;
    Some(
        fmt::layer()
            .with_writer(Mutex::new(file))
            .with_ansi(false)
            .boxed(),
    )
}

/// Adds every panic to the file at `path`, with the thread it was on and the
/// way it got there, before whatever was done with a panic before.
fn keep_panics_in(path: &Path) {
    let path = path.to_path_buf();
    let earlier = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        if let Ok(mut file) = OpenOptions::new().append(true).open(&path) {
            let thread = std::thread::current();
            let _ = writeln!(
                file,
                "PANIC on the {} thread: {panic}\n{}",
                thread.name().unwrap_or("unnamed"),
                Backtrace::force_capture()
            );
        }
        earlier(panic);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::log::LogPlugin;
    use bevy::log::tracing_subscriber::{Registry, layer::SubscriberExt};

    /// What `main` hands Bevy on an iPhone, made here too: no iOS build can be
    /// checked on Windows (`ring` and `tracing-oslog` want Apple's SDK).
    #[test]
    fn it_is_what_the_log_plugin_takes() {
        let plugin = LogPlugin {
            custom_layer: in_documents,
            ..default()
        };
        assert_eq!(plugin.custom_layer as usize, in_documents as usize);
    }

    #[test]
    fn what_is_logged_is_in_the_file() {
        let path = std::env::temp_dir()
            .join(format!("roundtown-log-{}", std::process::id()))
            .join(NAME);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "from the launch before\n").unwrap();
        let layer = writing_to(&path).expect("a layer");
        bevy::log::tracing::subscriber::with_default(Registry::default().with(layer), || {
            info!("the town is open");
            warn!("the fountain is dry");
        });
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("launch before"), "{text}");
        assert!(text.contains("the town is open") && text.contains("the fountain is dry"), "{text}");
        assert!(text.contains("WARN"), "{text}");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
