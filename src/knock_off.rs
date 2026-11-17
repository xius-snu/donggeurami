//! Knock Off, on this device (since 2026-11-18; Hajun: "You spawn on a tiny
//! icy circular island with 2~4 other ppl (fill with ai like bridgewars and
//! its deathmatch so 1v1v1v1) ... Everyone is frozen in place, and before
//! every certain time interval like 10 second, you choose the direction and
//! intensity that you wanna slide. Then after the interval, everyone goes
//! toward that direction and intensity and collide, some ppl will be
//! knocked off. You keep going until last player surviving is the winner.
//! If you are out, you can spectate or exit. Map = iceisland"): four seats
//! on a disc of ice in the sea, the computer taking those people leave,
//! everyone aiming at once and then sliding at once. Its rules are
//! `roundtown_net::knock_off`'s, which the server goes by online and this
//! device offline; its table is `table`'s. This is the device's side of it:
//! for now a stand-in, which holds the camera over the ice and, offline,
//! ends the game a while after it begins, until the game itself is here.

use bevy::prelude::*;
use roundtown_net::Minigame;

use crate::HeldCamera;
use crate::hud;
use crate::table::{self, AtTable, TableView};
use crate::ui::ui;

/// Something the server says to this device alone in its game of Knock Off
/// (`roundtown_net::knock_off::Tell`), handed on by `net`.
#[derive(Message, Clone, Debug)]
pub(crate) struct Told(pub roundtown_net::knock_off::Tell);

/// How long the stand-in's game lasts, offline, in seconds, before it says
/// it is over.
const STAND_IN_SECS: f32 = 20.0;

/// Where the camera is held over the ice (Hajun: "not sure what camera is
/// good for this one, you choose"), for now: from the south, 20 m up and 14
/// m out, looking at the middle of the island, where the whole of it is in
/// sight (mine; the game's own will be its own).
fn over_the_ice() -> Transform {
    Transform::from_xyz(0.0, 20.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y)
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Told>()
        .add_systems(Update, (hear, hold_the_camera, stand_in).after(table::TableSystems));
}

/// The buttons round the screen at a table of Knock Off: your name on its
/// plate, and nothing else yet. Nobody jumps or punches on the ice.
pub(crate) fn spawn_bar(commands: &mut Commands, _assets: &AssetServer, _images: &mut Assets<Image>) {
    let (left, _right) = hud::game_bar(commands);
    commands.spawn((ChildOf(left), hud::name_plate()));
}

/// What its score is called among the results: the file's
/// (`words.knock_off.score`).
pub(crate) fn score_word() -> String {
    ui().words.knock_off.score.clone()
}

/// What the line under the results says of what the whole table made: Knock
/// Off counts nothing together.
pub(crate) fn total_words(_total: u32) -> Option<String> {
    None
}

/// What the server says to this device of its game: nothing to do with it
/// yet.
fn hear(mut told: MessageReader<Told>) {
    for Told(said) in told.read() {
        debug!("Knock Off: told {said:?}");
    }
}

/// Holds the camera over the ice while you are at a table of Knock Off
/// ([`over_the_ice`]); the table lets it go as it ends.
fn hold_the_camera(at: Res<AtTable>, mut held: ResMut<HeldCamera>) {
    if at.of(Minigame::KnockOff).is_some() {
        held.set_if_neq(HeldCamera(Some(over_the_ice())));
    }
}

/// The stand-in's game, offline: once the table is played, [`STAND_IN_SECS`]
/// later it is over, the first seat first and the rest in seat order, every
/// score nothing.
fn stand_in(
    time: Res<Time>,
    at: Res<AtTable>,
    view: Res<TableView>,
    mut played: Local<f32>,
    mut finish: MessageWriter<table::Finish>,
) {
    let offline = at.of(Minigame::KnockOff).is_some_and(|sitting| !sitting.online);
    if !offline || !view.playing() {
        *played = 0.0;
        return;
    }
    let was = *played;
    *played += time.delta_secs();
    if was < STAND_IN_SECS && *played >= STAND_IN_SECS {
        let seats = view.members.len().max(1);
        finish.write(table::Finish {
            places: (0..seats as u8).collect(),
            scores: vec![0; seats],
            total: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_camera_looks_at_the_middle_of_the_ice_from_the_south() {
        let view = over_the_ice();
        assert_eq!(view.translation, Vec3::new(0.0, 20.0, 14.0));
        let looking = view.forward();
        let at_the_middle = (Vec3::ZERO - view.translation).normalize();
        assert!(looking.dot(at_the_middle) > 0.9999);
        // Toward the north, the top of the screen as it is over Bridge
        // Wars' map.
        assert!(looking.z < 0.0);
    }
}
