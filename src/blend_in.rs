//! Blend In, on this device (since 2026-11-18; Hajun: "1 random hider
//! chosen as the person walking around pretending to be an NPC/AI. 1~3
//! other people trying to find who is the person, by punching with a aura
//! sword. This game min player count is 2, so if there 2 ppl, it will not
//! fill the 2 slot with 2 ai's but rather start game with just 2 real ppl.
//! There is a time limit, and incorrect guesses cost you time by
//! stunning/blinding etc. ... Map = roundtownmap ... You swap until every one
//! has tried hiding atleast once ... (i think freecam is ok for blend in
//! mode, similar to main lobby camera)"): two to four people and never the
//! computer, each hiding once among a crowd of the town's people while the
//! others look for them, in the town as it stands with nobody of its own in
//! it (`island::Scenery`). Its rules are `roundtown_net::blend_in`'s, which
//! the server plays by; it is online only, its card on the selector blocked
//! out offline (`games`), and its table is `table`'s. This is the device's
//! side of it: for now a stand-in, the town's camera and a game's buttons,
//! until the game itself is here.

use bevy::prelude::*;
use roundtown_net::Minigame;

use crate::hud;
use crate::table::{self, AtTable, TableView};
use crate::ui::ui;

/// Something the server says to this device alone in its game of Blend In
/// (`roundtown_net::blend_in::Tell`), handed on by `net`.
#[derive(Message, Clone, Debug)]
pub(crate) struct Told(pub roundtown_net::blend_in::Tell);

/// How long the stand-in's game lasts, offline, in seconds, before it says
/// it is over: never played offline, which Blend In is not, but there for
/// a table that is.
const STAND_IN_SECS: f32 = 20.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Told>()
        .add_systems(Update, (hear, stand_in).after(table::TableSystems));
}

/// The buttons round the screen at a table of Blend In, a game and not the
/// town: your name on its plate, and the chat's keyboard where the town has
/// it; no balance, gear, friends, games, car or things to say, and the
/// town's own jump button (`hud::Layout::BlendIn`).
pub(crate) fn spawn_bar(commands: &mut Commands, _assets: &AssetServer, images: &mut Assets<Image>) {
    let (left, _right) = hud::game_bar(commands);
    commands.spawn((ChildOf(left), hud::name_plate()));
    let round = commands.spawn(hud::round_the_jump()).id();
    commands.spawn((ChildOf(round), hud::chat_button(images)));
}

/// What its score is called among the results: the file's
/// (`words.blend_in.score`).
pub(crate) fn score_word() -> String {
    ui().words.blend_in.score.clone()
}

/// What the line under the results says of what the whole table made:
/// Blend In counts nothing together.
pub(crate) fn total_words(_total: u32) -> Option<String> {
    None
}

/// What the server says to this device of its game: nothing to do with it
/// yet.
fn hear(mut told: MessageReader<Told>) {
    for Told(said) in told.read() {
        debug!("Blend In: told {said:?}");
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
    let offline = at.of(Minigame::BlendIn).is_some_and(|sitting| !sitting.online);
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
    fn the_chat_s_keyboard_is_clear_of_the_town_s_jump_button() {
        let (right, bottom, size) = hud::CHAT_BUTTON;
        hud::clear_of_each_other(&[((0.0, 0.0), crate::JUMP_SIZE_VMIN), ((right, bottom), size)]).unwrap();
    }
}
