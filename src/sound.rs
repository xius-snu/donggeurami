//! What the town sounds like (Hajun, 2026-10-30): music all the time.
//!
//! The music, "Pelican Breeze - An Evening at the Bar", plays from the start
//! and round again whenever it ends, wherever you are. It goes quiet while
//! the app is in the background: a phone keeps playing what an app has going
//! after it has been put away, and the game is not drawn then.
//!
//! The town's buses had an engine for an evening, heard from where each bus
//! was, until Hajun had it taken out ("remove engine sound just keep music").
//! Its loop, `sounds/bus_engine_loop.wav`, is still in `assets/`, and left
//! out of the APK.

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::window::AppLifecycle;

/// The music, under `assets/`.
pub(crate) const MUSIC: &str = "sounds/Pelican Breeze - An Evening at the Bar.mp3";

/// How loud the music plays, of how loud it was made: -30 dBFS on average
/// (its own is -17.9). It was 0.5 until Hajun found it too loud (2026-10-30).
const MUSIC_VOLUME: f32 = 0.25;

/// The music playing.
#[derive(Component)]
struct Music;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, play_music)
        .add_systems(Update, (play_again, hush));
}

/// Played once, not looped: Bevy loops a sound by keeping every sample of it
/// as it goes, 56 MB for this one, so the music is started over instead
/// (`play_again`), from the file.
fn play_music(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((
        Name::new("Music"),
        Music,
        AudioPlayer::new(assets.load(MUSIC)),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(MUSIC_VOLUME)),
    ));
}

/// Starts the music over once it has ended: without its sink, Bevy plays it
/// again from the start.
fn play_again(music: Query<(Entity, &AudioSink), With<Music>>, mut commands: Commands) {
    for (entity, sink) in &music {
        if sink.empty() {
            commands.entity(entity).remove::<AudioSink>();
        }
    }
}

/// Pauses the music while the app is in the background, and plays it again
/// once it is back.
fn hush(
    mut lifecycle: MessageReader<AppLifecycle>,
    mut away: Local<bool>,
    music: Query<&AudioSink, With<Music>>,
) {
    for event in lifecycle.read() {
        match event {
            AppLifecycle::WillSuspend | AppLifecycle::Suspended => *away = true,
            AppLifecycle::WillResume | AppLifecycle::Running => *away = false,
            AppLifecycle::Idle => {}
        }
    }
    for sink in &music {
        if sink.is_paused() != *away {
            if *away {
                sink.pause();
            } else {
                sink.play();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::audio::{AudioSource, Decodable, Source};

    #[test]
    fn the_music_decodes() {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join(MUSIC);
        let bytes = std::fs::read(&file).expect("the music's file");
        let decoder = AudioSource { bytes: bytes.into() }.decoder();
        let (channels, rate) = (decoder.channels().get(), decoder.sample_rate().get());
        let samples = decoder.count();
        let seconds = samples as f32 / f32::from(channels) / rate as f32;
        assert!(seconds > 150.0, "the music decoded to {seconds} s");
    }
}
