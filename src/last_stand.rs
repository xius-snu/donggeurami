//! Last Stand, on this device (since 2026-11-18; Hajun: "Coop (2~4 players)
//! Last Stand (Endless Zombie survival game). You are in a circular map,
//! with a spawn anchor crystal in the center, and 2 generators and item
//! shops/upgrade shops. Zombies with increasing difficulty/skill/creativity/
//! health/attackdmg come in waves from the outer edges toward the anchor
//! crystal in the middle and/or player ... Endless upgrades, endless waves
//! of endlessly stronger and stronger enemies. You get a numerical score at
//! the end (individual scores of each person + total combined team score).
//! Map = laststandmap (similar cam and controls with bridgewars)"): one to
//! four people together, holding the anchor for as many waves as they can.
//! Its rules are `roundtown_net::last_stand`'s, which the server goes by
//! online and this device offline; its table is `table`'s, which fixes the
//! camera over you as in Bridge Wars. This is the device's side of it: for
//! now a stand-in, Bridge Wars' buttons and, offline, the game over a while
//! after it begins, until the game itself is here.

use bevy::prelude::*;
use roundtown_net::{Item, Minigame, UTILITY_SLOTS};

use crate::hud;
use crate::table::{self, AtTable, TableView};
use crate::ui::ui;

/// Something the server says to this device alone in its game of Last Stand
/// (`roundtown_net::last_stand::Tell`), handed on by `net`.
#[derive(Message, Clone, Debug)]
pub(crate) struct Told(pub roundtown_net::last_stand::Tell);

/// How long the stand-in's game lasts, offline, in seconds, before it says
/// it is over, and how many waves it says the table cleared.
const STAND_IN_SECS: f32 = 20.0;
const STAND_IN_WAVES: u32 = 2;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Told>()
        .add_systems(Update, (hear, stand_in).after(table::TableSystems));
}

/// The buttons round the screen at a table of Last Stand, Bridge Wars' less
/// what is not in it (Hajun: "similar cam and controls with bridgewars"):
/// top left your name, your health under it, and under that what you hold
/// of the four things the generators make, as in Bridge Wars, but no
/// minimap; and round the jump button the sword in its place, which
/// strikes, the jump button beside it, the five buttons for things to use,
/// and the chat's keyboard, each where Bridge Wars has it. No plank: there
/// is no bridge to build.
pub(crate) fn spawn_bar(commands: &mut Commands, _assets: &AssetServer, images: &mut Assets<Image>) {
    let colours = ui().colours;
    let (left, _right) = hud::game_bar(commands);
    commands.spawn((ChildOf(left), hud::name_plate()));
    commands.spawn((ChildOf(left), hud::health_bar()));
    for item in Item::ALL {
        commands.spawn((
            ChildOf(left),
            hud::resource_row(item, images.add(hud::item_picture(item)), colours.arena.item(item)),
        ));
    }
    let round = commands.spawn(hud::round_the_jump()).id();
    let arrow = images.add(hud::jump_arrow());
    commands.spawn((
        ChildOf(round),
        hud::punch_button(images.add(hud::sword()), arrow.clone(), crate::JUMP_SIZE_VMIN, hud::beside_the_jump(0.0, 0.0)),
    ));
    let (right, bottom, size) = hud::ARENA_JUMP;
    commands.spawn((ChildOf(round), hud::jump_button(arrow, size, hud::beside_the_jump(right, bottom))));
    for slot in 0..UTILITY_SLOTS {
        let (right, bottom) = hud::slot_place(slot);
        commands.spawn((ChildOf(round), hud::utility_slot(slot, hud::beside_the_jump(right, bottom))));
    }
    commands.spawn((ChildOf(round), hud::chat_button(images)));
}

/// What its score is called among the results: the file's
/// (`words.last_stand.score`).
pub(crate) fn score_word() -> String {
    ui().words.last_stand.score.clone()
}

/// What the line under the results says of what the whole table made, the
/// waves it cleared: nothing yet (the game's own words will say it).
pub(crate) fn total_words(_total: u32) -> Option<String> {
    None
}

/// What the server says to this device of its game: nothing to do with it
/// yet.
fn hear(mut told: MessageReader<Told>) {
    for Told(said) in told.read() {
        debug!("Last Stand: told {said:?}");
    }
}

/// The stand-in's game, offline: once the table is played, [`STAND_IN_SECS`]
/// later it is over, the first seat first and the rest in seat order, every
/// score nothing, and [`STAND_IN_WAVES`] waves cleared together.
fn stand_in(
    time: Res<Time>,
    at: Res<AtTable>,
    view: Res<TableView>,
    mut played: Local<f32>,
    mut finish: MessageWriter<table::Finish>,
) {
    let offline = at.of(Minigame::LastStand).is_some_and(|sitting| !sitting.online);
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
            total: Some(STAND_IN_WAVES),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_button_round_the_jump_button_is_clear_of_every_other() {
        // The sword in the jump button's place, the jump button, the five
        // buttons for things to use and the chat's keyboard.
        let (jump_right, jump_bottom, jump_size) = hud::ARENA_JUMP;
        let (chat_right, chat_bottom, chat_size) = hud::CHAT_BUTTON;
        let mut buttons = vec![
            ((0.0, 0.0), crate::JUMP_SIZE_VMIN),
            ((jump_right, jump_bottom), jump_size),
            ((chat_right, chat_bottom), chat_size),
        ];
        buttons.extend((0..UTILITY_SLOTS).map(|slot| (hud::slot_place(slot), hud::ACTION_BUTTON)));
        hud::clear_of_each_other(&buttons).unwrap();
    }
}
