//! Bridge Race, on this device (since 2026-11-18; Hajun: "Like King of the
//! Ladder but with bridge building. You start on two islands next to each
//! other, and there is a golden island at equidistance from each island
//! like the tip of a 80deg 80deg 20deg isosceles triangle. Every interval
//! you are given more bridge meters, and sometimes u get dynamite as well,
//! randomly. Your goal is to reach the golden island first compared to the
//! other team - both of u need to make it across cuz its 2v2. You can punch
//! each other off the map, and ur respawn is the center of ur own player
//! island ... (similar controls and cam with bridge wars). Map =
//! bridgeracemap"): two teams, each on its own island over the void,
//! bridging to the golden island, the computer making up the side people do
//! not. Its rules are `roundtown_net::bridge_race`'s, which the server goes
//! by online and this device offline; its table is `table`'s, which fixes
//! the camera over you as in Bridge Wars. This is the device's side of it:
//! for now a stand-in, Bridge Wars' buttons and, offline, the game over a
//! while after it begins, until the game itself is here.

use bevy::prelude::*;
use roundtown_net::{Minigame, UTILITY_SLOTS};

use crate::hud;
use crate::table::{self, AtTable, TableView};
use crate::ui::ui;

/// Something the server says to this device alone in its game of Bridge
/// Race (`roundtown_net::bridge_race::Tell`), handed on by `net`.
#[derive(Message, Clone, Debug)]
pub(crate) struct Told(pub roundtown_net::bridge_race::Tell);

/// How long the stand-in's game lasts, offline, in seconds, before it says
/// it is over.
const STAND_IN_SECS: f32 = 20.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Told>()
        .add_systems(Update, (hear, stand_in).after(table::TableSystems));
}

/// The buttons round the screen at a table of Bridge Race, Bridge Wars'
/// less what is not in the race (Hajun: "similar controls and cam with
/// bridge wars"): your name top left, with no health (nobody has any: the
/// void is the only way to die) and none of the four things the generators
/// make, nor the minimap; and round the jump button the fist in its place,
/// which punches, the jump button beside it, the plank, which builds, the
/// five buttons for things to use (the dynamite), and the chat's keyboard,
/// each where Bridge Wars has it.
pub(crate) fn spawn_bar(commands: &mut Commands, _assets: &AssetServer, images: &mut Assets<Image>) {
    let (left, _right) = hud::game_bar(commands);
    commands.spawn((ChildOf(left), hud::name_plate()));
    let round = commands.spawn(hud::round_the_jump()).id();
    let arrow = images.add(hud::jump_arrow());
    commands.spawn((
        ChildOf(round),
        hud::punch_button(images.add(hud::fist()), arrow.clone(), crate::JUMP_SIZE_VMIN, hud::beside_the_jump(0.0, 0.0)),
    ));
    let (right, bottom, size) = hud::ARENA_JUMP;
    commands.spawn((ChildOf(round), hud::jump_button(arrow, size, hud::beside_the_jump(right, bottom))));
    for slot in 0..UTILITY_SLOTS {
        let (right, bottom) = hud::slot_place(slot);
        commands.spawn((ChildOf(round), hud::utility_slot(slot, hud::beside_the_jump(right, bottom))));
    }
    let (right, bottom) = hud::ARENA_BRIDGE;
    commands.spawn((ChildOf(round), hud::bridge_button(images.add(hud::plank()), hud::beside_the_jump(right, bottom))));
    commands.spawn((ChildOf(round), hud::chat_button(images)));
}

/// What its score is called among the results: the file's
/// (`words.bridge_race.score`).
pub(crate) fn score_word() -> String {
    ui().words.bridge_race.score.clone()
}

/// What the line under the results says of what the whole table made:
/// nothing yet.
pub(crate) fn total_words(_total: u32) -> Option<String> {
    None
}

/// What the server says to this device of its game: nothing to do with it
/// yet.
fn hear(mut told: MessageReader<Told>) {
    for Told(said) in told.read() {
        debug!("Bridge Race: told {said:?}");
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
    let offline = at.of(Minigame::BridgeRace).is_some_and(|sitting| !sitting.online);
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
    fn every_button_round_the_jump_button_is_clear_of_every_other() {
        // The fist in the jump button's place, the jump button, the plank,
        // the five buttons for things to use and the chat's keyboard.
        let (jump_right, jump_bottom, jump_size) = hud::ARENA_JUMP;
        let (chat_right, chat_bottom, chat_size) = hud::CHAT_BUTTON;
        let mut buttons = vec![
            ((0.0, 0.0), crate::JUMP_SIZE_VMIN),
            ((jump_right, jump_bottom), jump_size),
            (hud::ARENA_BRIDGE, hud::ACTION_BUTTON),
            ((chat_right, chat_bottom), chat_size),
        ];
        buttons.extend((0..UTILITY_SLOTS).map(|slot| (hud::slot_place(slot), hud::ACTION_BUTTON)));
        hud::clear_of_each_other(&buttons).unwrap();
    }
}
