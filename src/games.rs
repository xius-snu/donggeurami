//! The game selector, behind the gamepad button down the right edge of the
//! screen (`hud::GamesButton`; Hajun, the night of 2026-11-07: "when u press the game
//! icon ui button, it will open a popup game selector menu with 3 games
//! (the first one is called 'Bridge Wars' the second is called 'Home
//! Builders' and third is a ??? coming soon. you select one and you get a
//! quick black loading screen and then when the new map is loaded, the
//! player is teleported"): a dialog like the settings' window, with the
//! games on cards across it. Only in the town. It is drawn as the friends'
//! panel is (`window`, since 2026-11-14, when Hajun asked for that panel's
//! style everywhere): the same dark see-through window, as wide, with
//! nothing dimmed round it, "Games" along its top from the left, the bare
//! white cross in its corner, and its cards the dark of that panel's
//! buttons. Until then it was a black box in the middle of the dimmed
//! screen under a gold title, its cross a disc hung off its corner.
//!
//! **Six games since 2026-11-18** (Hajun: "add these minigames: Blend In,
//! Knock Off, Bridge Race, Last Stand ... add them to the game selector
//! panel"), on two rows of three cards: Bridge Wars, Blend In and Knock
//! Off over Bridge Race, Last Stand and Home Builders, the four new ones
//! played at tables (`table`). Each card says the game's name, a line of
//! what it is, and what it costs: Bridge Wars nothing, the rest their fees
//! (the four's by `roundtown_net::table`, Home Builders' House Builder's
//! own), each blocked out, saying so, when you cannot pay it, and Blend In
//! offline, where it is not to be had: it needs people. Until then there
//! were three cards on one row, the third a game to come ("???"), which is
//! put away ([`COMING_SOON`]). How the cards stand, the line under each
//! name and the order are mine.
//!
//! **Four of them since 2026-11-19**: Bridge Race's and Last Stand's cards
//! are turned off ([`OFF`]; Hajun, having played all six: "first turn off
//! last stand and bridge race"), which leaves Bridge Wars and Blend In over
//! Knock Off and Home Builders, two to a row ([`shown`]). Only the cards:
//! the two games, their tables and their maps are as they were, and
//! emptying [`OFF`] puts the six back as they stood.
//!
//! Choosing a game, the dialog goes and a black screen comes down
//! ([`Loading`]): [`LOADING_IN`] seconds later, under it, the game is gone
//! into — Bridge Wars' waiting room (`arena::Ask::Enter`), House Builder's
//! lobby, paying its fee (`builder::Ask::Join`, as the House Builder's Play
//! did), or a table of one of the four (`table::Ask::Join`) — and once you
//! stand on the game's island and it has loaded, and a moment more has
//! passed, it lifts. Should nothing come of the choice, it lifts after
//! [`LOADING_MOST`] seconds all the same; should the server turn down a
//! seat at one of the four's tables (`Tell::Refused`), at once, the table
//! saying why across the screen (`table`). Bridge Wars' and House
//! Builder's refusals wait out the black screen as they always have. While
//! it is down you stand still, as under a dialog (`hud::DialogUp`).

use bevy::prelude::*;
use roundtown_net::rules::ENTRY;
use roundtown_net::{Minigame, Tell, table as tables};

use crate::arena;
use crate::builder::{self, DIALOG_LAYER};
use crate::friends::Friends;
use crate::hud::{self, DialogUp, GamesButton, TakesPress};
use crate::island::{Islands, Venue};
use crate::lobby::Balance;
use crate::net::Online;
use crate::table;
use crate::ui::ui;
use crate::window;
use crate::{CloseUp, Player};

/// Sizes, as shares of the short side of the screen, where they are not
/// the windows' own (`window`): how tall a game's card is (three share the
/// window's width on each of the two rows; 30 tall while there was one
/// row), the gap between the cards and between the rows, the room inside a
/// card round its words, and the game's name on one and the lines under
/// it. Two rows of 24 make the window as tall as the friends' panel.
const CARD_HEIGHT: f32 = 24.0;
const CARD_GAP: f32 = 2.2;
const CARD_PAD: f32 = 1.5;
const CARD_TEXT: f32 = 4.0;
const NOTE_TEXT: f32 = 2.8;
/// The coin's picture in a card's price, as tall as the words' capitals
/// and a little more, and its gap to the number either side (mine).
const COIN_SIZE: f32 = 3.0;
const COIN_GAP: f32 = 0.6;
/// The room the line under a card's name has, in vmin: two lines of it.
const LINE_ROOM: f32 = 2.0 * NOTE_TEXT * 1.2;
/// The cards, row by row, in the order Hajun listed the games: the new
/// four after Bridge Wars, and Home Builders, stalled, last.
const CARDS: [[Minigame; 3]; 2] = [
    [Minigame::BridgeWars, Minigame::BlendIn, Minigame::KnockOff],
    [Minigame::BridgeRace, Minigame::LastStand, Minigame::HouseBuilder],
];
/// The games whose cards are turned off: Bridge Race and Last Stand since
/// 2026-11-19 (Hajun, having played all six: "laststand is a bit too long
/// and unexciting, and bridgerace not sure but its not that fun rn", and
/// then: "first turn off last stand and bridge race"). Only their cards:
/// the games, their tables on the server and their maps are as they were,
/// and an app built before this still shows and plays them. Emptied, the
/// six stand as they did.
const OFF: &[Minigame] = &[Minigame::BridgeRace, Minigame::LastStand];
/// Whether the card for a game to come ("???", "Coming soon") is on the
/// selector: not since 2026-11-18, the games having come. This one word
/// puts it back, at the end of the second row.
const COMING_SOON: bool = false;
/// The black screen: how long it is down before the game is gone into
/// under it, in seconds; how much longer at the least before it lifts, once
/// the island you are on has loaded; how long at the most, should nothing
/// come of the choice; and how long it takes to lift.
const LOADING_IN: f32 = 0.5;
const LOADING_HOLD: f32 = 0.3;
const LOADING_MOST: f32 = 3.0;
const LOADING_OUT: f32 = 0.3;

/// The root of the dialog, the dark behind it.
#[derive(Component)]
struct GamesDialog;

/// What a press on one of the dialog's buttons does: a game's card, or
/// shutting it.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Choice {
    Game(Minigame),
    Close,
}

/// Whether the dialog is up.
#[derive(Resource, Default)]
struct Selector {
    open: bool,
}

/// The black screen over everything.
#[derive(Component)]
struct LoadingScreen;

/// The black screen, from coming down to lifting: the game chosen, how
/// long it has been down, whether the game has been gone into yet, and how
/// long it has been lifting, once it is.
#[derive(Resource)]
struct Loading {
    pick: Minigame,
    age: f32,
    fired: bool,
    lifting: Option<f32>,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Selector>().add_observer(choose).add_systems(
        Update,
        (keys, load)
            .chain()
            .before(builder::GameSystems)
            .before(arena::ArenaSystems)
            .before(table::TableSystems),
    );
}

/// The cards as they stand on the selector, row by row: the games of
/// [`CARDS`] that are not [`OFF`].
fn shown() -> Vec<Vec<Minigame>> {
    laid(OFF)
}

/// The games of [`CARDS`] but those of `off`, in their order, on as many
/// rows as [`CARDS`] has and as many to each as fills them: three to a row
/// with all six, two with four (that four are laid two and two, and not
/// three and one or all in a line, is mine).
fn laid(off: &[Minigame]) -> Vec<Vec<Minigame>> {
    let on: Vec<Minigame> = CARDS.iter().flatten().copied().filter(|game| !off.contains(game)).collect();
    let to_a_row = on.len().div_ceil(CARDS.len()).max(1);
    on.chunks(to_a_row).map(<[Minigame]>::to_vec).collect()
}

/// What `game` is called: on its card, on its table's board and results
/// (`table`), and where a friend is, playing it (`friends`).
pub(crate) fn name_of(game: Minigame) -> String {
    let ui = ui();
    let words = &ui.words.games;
    match game {
        Minigame::BridgeWars => words.bridge_wars.clone(),
        Minigame::HouseBuilder => words.home_builders.clone(),
        Minigame::BlendIn => words.blend_in.clone(),
        Minigame::KnockOff => words.knock_off.clone(),
        Minigame::BridgeRace => words.bridge_race.clone(),
        Minigame::LastStand => words.last_stand.clone(),
    }
}

/// The line of what `game` is, under its name on its card.
fn line_of(game: Minigame) -> String {
    let ui = ui();
    let words = &ui.words.games;
    match game {
        Minigame::BridgeWars => words.bridge_wars_line.clone(),
        Minigame::HouseBuilder => words.home_builders_line.clone(),
        Minigame::BlendIn => words.blend_in_line.clone(),
        Minigame::KnockOff => words.knock_off_line.clone(),
        Minigame::BridgeRace => words.bridge_race_line.clone(),
        Minigame::LastStand => words.last_stand_line.clone(),
    }
}

/// What `game` costs to play, in coins: Bridge Wars' since 2026-11-18
/// (Hajun: "make the cost of bridgewars to be 50 coins as well"; free until
/// then), the four's by their tables, Home Builders' House Builder's own.
fn fee(game: Minigame) -> u32 {
    match game {
        Minigame::BridgeWars => roundtown_net::bridge_wars::FEE,
        Minigame::HouseBuilder => ENTRY,
        game => tables::shape(game).map_or(0, |shape| shape.fee),
    }
}

/// What a card says under its line: words; or words with a price in them,
/// which stands where they say `{coins}` as the coin's picture and how many
/// (since 2026-11-18; Hajun: "instead of writing the text "50 coins" with
/// the word "coins" use the actual coin icon instead so its "(coin_icon)
/// 50""): "(coin) 50", "Needs (coin) 50".
#[derive(Clone, PartialEq, Debug)]
enum Note {
    Words(String),
    Price { words: String, coins: u32 },
}

impl Note {
    /// The words before the price, how many it is, and the words after it;
    /// words alone are all before.
    fn parts(&self) -> (String, Option<String>, String) {
        match self {
            Note::Words(words) => (words.clone(), None, String::new()),
            Note::Price { words, coins } => match words.split_once("{coins}") {
                Some((before, after)) => (before.trim_end().to_string(), Some(hud::money(*coins)), after.trim_start().to_string()),
                None => (words.clone(), Some(hud::money(*coins)), String::new()),
            },
        }
    }

    /// As it reads, the coin's picture as a letter: for measuring it.
    #[cfg(test)]
    fn read(&self) -> String {
        let (before, coins, after) = self.parts();
        let price = coins.map(|coins| format!("O {coins}"));
        [Some(before), price, Some(after)].into_iter().flatten().filter(|part| !part.is_empty()).collect::<Vec<_>>().join(" ")
    }
}

/// What `game`'s card says under its line, for someone with `balance`
/// coins, `online` or not: what it costs, or why it is not to be had, in
/// the colour of a line that says why something cannot be had; and
/// whether it is to be had. Every game is blocked out without the coins
/// (Bridge Wars too, since it costs); and Blend In offline, where there is
/// nobody to play it with. A game that cost nothing would say so.
fn price(game: Minigame, balance: u32, online: bool) -> (Note, Option<Color>, bool) {
    let ui = ui();
    let words = &ui.words.games;
    let notice = ui.colours.window.notice;
    let coins = fee(game);
    let priced = |words: &String| Note::Price {
        words: words.clone(),
        coins,
    };
    match game {
        Minigame::BlendIn if !online => (Note::Words(words.online_only.clone()), Some(notice), false),
        _ if coins == 0 => (Note::Words(words.free.clone()), None, true),
        _ if balance >= coins => (priced(&words.price), None, true),
        _ => (priced(&words.needs_coins), Some(notice), false),
    }
}

/// The line a card ends on ([`Note`]), under `card`, in `colour`: its
/// words, and where it has a price, the coin's picture and the number,
/// on one line in the card's middle, the picture as tall as the words.
fn spawn_note(commands: &mut Commands, assets: &AssetServer, card: Entity, note: &Note, colour: Color) {
    let (before, coins, after) = note.parts();
    let row = commands
        .spawn((
            ChildOf(card),
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::VMin(COIN_GAP),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    if !before.is_empty() {
        commands.spawn((ChildOf(row), builder::label(before, NOTE_TEXT, colour)));
    }
    if let Some(coins) = coins {
        commands.spawn((
            ChildOf(row),
            // Dimmed with the words on a card that cannot be had.
            ImageNode::new(assets.load(hud::COIN_ICON)).with_color(Color::WHITE.with_alpha(colour.alpha())),
            Node {
                width: Val::VMin(COIN_SIZE),
                height: Val::VMin(COIN_SIZE),
                ..default()
            },
            Pickable::IGNORE,
        ));
        commands.spawn((ChildOf(row), builder::label(coins, NOTE_TEXT, colour)));
    }
    if !after.is_empty() {
        commands.spawn((ChildOf(row), builder::label(after, NOTE_TEXT, colour)));
    }
}

/// Opens the dialog from the gamepad, and goes on from there as its buttons
/// are pressed. Only what the press landed on, not what the click passed up
/// through on its way out, so that a tap on a card is not also a tap off
/// the window, which shuts it.
#[allow(clippy::too_many_arguments)]
fn choose(
    click: On<Pointer<Click>>,
    buttons: Query<(), With<GamesButton>>,
    choices: Query<&Choice>,
    me: Query<(&Venue, &Balance), With<Player>>,
    close_up: Res<CloseUp>,
    loading: Option<Res<Loading>>,
    (online, friends, assets): (Res<Online>, Res<Friends>, Res<AssetServer>),
    dialogs: Query<Entity, With<GamesDialog>>,
    mut up: ResMut<DialogUp>,
    mut selector: ResMut<Selector>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if buttons.contains(click.entity) {
        let Ok((&here, balance)) = me.single() else {
            return;
        };
        // Only in the town, with nothing else up.
        if selector.open || up.0 || close_up.busy() || loading.is_some() || here != Venue::Town {
            return;
        }
        // In a group someone else leads, its games are theirs to start
        // (`friends`): the cards are blocked out, and say so.
        let follows = online.me().is_some_and(|me| friends.follows(me.id));
        open(&mut commands, &assets, balance.0, online.is(), follows);
        selector.open = true;
        up.0 = true;
        return;
    }
    let Ok(&choice) = choices.get(click.entity) else {
        return;
    };
    if !selector.open {
        return;
    }
    close(&mut commands, &dialogs, &mut selector);
    match choice {
        Choice::Close => up.0 = false,
        Choice::Game(game) => begin(&mut commands, game),
    }
}

/// On desktop, `Escape` shuts the dialog.
fn keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    dialogs: Query<Entity, With<GamesDialog>>,
    mut selector: ResMut<Selector>,
    mut up: ResMut<DialogUp>,
    mut commands: Commands,
) {
    if !selector.open || !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    close(&mut commands, &dialogs, &mut selector);
    up.0 = false;
}

/// The dialog, a window as the friends' panel is one (`window`): its title
/// along its top, the cross in its corner, and the games on cards in two
/// rows across it ([`shown`]). Its root is over the whole screen, shows
/// nothing, and shuts the dialog at a tap. A card is blocked out when its
/// game is not to be had, saying why ([`price`]); and every one when you
/// `follows` someone else's group, whose leader starts its games (since
/// 2026-11-13, Hajun: "when youre in the group with ur friend, only the
/// leader can queue for a minigame"), with a line along the window's bottom
/// that says so.
fn open(commands: &mut Commands, assets: &AssetServer, balance: u32, online: bool, follows: bool) {
    let ui = ui();
    let (colours, words) = (ui.colours.window, &ui.words.games);
    let root = commands
        .spawn((
            GamesDialog,
            Choice::Close,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let panel = commands
        .spawn((
            ChildOf(root),
            window::pane(Node {
                width: Val::VMin(window::WIDTH),
                // More at the bottom while there is a line to go there.
                padding: UiRect {
                    bottom: Val::VMin(bottom_room(follows)),
                    ..UiRect::all(Val::VMin(window::PAD))
                },
                ..default()
            }),
        ))
        .id();
    commands.spawn((
        ChildOf(panel),
        window::header(),
        children![window::words(words.title.clone(), window::TITLE, colours.title)],
    ));
    commands.spawn((ChildOf(panel), window::cross(assets.load("icons/close.png"), Choice::Close)));
    let rows = commands
        .spawn((
            ChildOf(panel),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::VMin(CARD_GAP),
                margin: UiRect::top(Val::VMin(1.2)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let shown = shown();
    let to_a_row = shown.first().map_or(0, Vec::len);
    for (index, games) in shown.iter().enumerate() {
        let row = commands
            .spawn((
                ChildOf(rows),
                Node {
                    column_gap: Val::VMin(CARD_GAP),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        for &game in games {
            let (note, tint, able) = price(game, balance, online);
            let able = able && !follows;
            let mut card = commands.spawn((ChildOf(row), card(name_of(game), line_of(game), able)));
            if able {
                card.insert((Choice::Game(game), TakesPress, Button));
            }
            let card = card.id();
            let (.., soft) = window::card(able);
            spawn_note(commands, assets, card, &note, tint.unwrap_or(soft));
        }
        // The game to come, were it back ([`COMING_SOON`]): at the end of
        // the last row, blocked out.
        let mut stand = games.len();
        if COMING_SOON && index + 1 == shown.len() {
            let card = commands.spawn((ChildOf(row), card(words.coming_soon.clone(), String::new(), false))).id();
            let (.., soft) = window::card(false);
            spawn_note(commands, assets, card, &Note::Words(words.coming_soon_note.clone()), soft);
            stand += 1;
        }
        // A row short of the first keeps the room its missing cards would
        // have, so that its own are no wider than the rest.
        for _ in stand..to_a_row {
            commands.spawn((
                ChildOf(row),
                Node {
                    flex_grow: 1.0,
                    flex_basis: Val::ZERO,
                    ..default()
                },
                Pickable::IGNORE,
            ));
        }
    }
    if follows {
        commands.spawn((
            ChildOf(panel),
            window::words(ui.words.friends.only_leader.clone(), window::NOTICE_TEXT, colours.notice),
            Node {
                position_type: PositionType::Absolute,
                left: Val::VMin(window::PAD),
                bottom: Val::VMin(1.8),
                ..default()
            },
        ));
    }
}

/// The room at the bottom of the window inside it: as round its sides, and
/// with the line that says only the leader starts a game, room for it.
fn bottom_room(follows: bool) -> f32 {
    if follows { window::PAD + window::NOTICE_TEXT + 1.4 } else { window::PAD }
}

/// One game's card, a third of a row: its name, the line under it of what
/// it is, and under that (spawned under it, [`spawn_note`]) what it costs,
/// or why it is not to be had; lit under the pointer if it can be chosen,
/// and blocked out if not.
fn card(name: String, line: String, choosable: bool) -> impl Bundle {
    let (fill, ink, soft) = window::card(choosable);
    (
        Node {
            flex_grow: 1.0,
            flex_basis: Val::ZERO,
            height: Val::VMin(CARD_HEIGHT),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::VMin(1.2),
            padding: UiRect::all(Val::VMin(CARD_PAD)),
            border_radius: BorderRadius::all(Val::VMin(window::CARD_ROUNDED)),
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![
            builder::label(name, CARD_TEXT, ink),
            (
                builder::label(line, NOTE_TEXT, soft),
                // Two lines' room, whether the line takes one or two, so
                // that every card's name and price stand level with its
                // neighbours' (a card with a short line sat lower; mine,
                // 2026-11-18).
                Node {
                    min_height: Val::VMin(LINE_ROOM),
                    ..default()
                },
            ),
        ],
    )
}

/// Takes the dialog down.
fn close(commands: &mut Commands, dialogs: &Query<Entity, With<GamesDialog>>, selector: &mut Selector) {
    for dialog in dialogs {
        commands.entity(dialog).despawn();
    }
    selector.open = false;
}

/// Brings the black screen down, with `pick` to go into under it.
fn begin(commands: &mut Commands, pick: Minigame) {
    commands.spawn((
        LoadingScreen,
        TakesPress,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(DIALOG_LAYER + 5),
    ));
    commands.insert_resource(Loading {
        pick,
        age: 0.0,
        fired: false,
        lifting: None,
    });
}

/// Whether you have got where `pick` takes you, standing `here`: on an
/// island that has loaded, and for Bridge Wars on its map, for one of the
/// four on its venue, which online the server has to seat you on first
/// (Blend In's being the town's map, loaded with it); for Home Builders, as
/// ever, anywhere loaded.
fn landed(pick: Minigame, here: Venue, islands: &Islands) -> bool {
    islands.get(here).is_some() && (pick == Minigame::HouseBuilder || here == pick.venue())
}

/// Goes into the game chosen once the black screen has been down a moment,
/// and lifts the screen once you have got there and a moment more has
/// passed, or once it has been down long enough that nothing is coming of
/// the choice, or, for a seat at one of the four's tables, at once if the
/// server has turned it down, the table saying why (`table`). Bridge Wars
/// and House Builder say nothing of a refusal across the screen, and their
/// black screen waits as it did before the four came: left as they were.
#[allow(clippy::too_many_arguments)]
fn load(
    time: Res<Time>,
    loading: Option<ResMut<Loading>>,
    me: Query<&Venue, With<Player>>,
    islands: Res<Islands>,
    mut tells: MessageReader<Tell>,
    mut screens: Query<(Entity, &mut BackgroundColor), With<LoadingScreen>>,
    mut up: ResMut<DialogUp>,
    (mut arena_asks, mut builder_asks, mut table_asks): (
        MessageWriter<arena::Ask>,
        MessageWriter<builder::Ask>,
        MessageWriter<table::Ask>,
    ),
    mut commands: Commands,
) {
    let Some(mut loading) = loading else {
        tells.clear();
        return;
    };
    let dt = time.delta_secs();
    loading.age += dt;
    if !loading.fired && loading.age >= LOADING_IN {
        loading.fired = true;
        match loading.pick {
            Minigame::BridgeWars => {
                arena_asks.write(arena::Ask::Enter);
            }
            Minigame::HouseBuilder => {
                builder_asks.write(builder::Ask::Join);
            }
            game => {
                table_asks.write(table::Ask::Join(game));
            }
        }
    }
    // Every word read, whatever it is, so that none is read a frame late.
    let refused = tells.read().filter(|tell| matches!(tell, Tell::Refused(_))).count() > 0;
    let turned_down = refused && loading.fired && loading.pick.at_a_table();
    let lifted = match loading.lifting {
        Some(lifted) => lifted + dt,
        None => {
            let settled = loading.fired
                && loading.age >= LOADING_IN + LOADING_HOLD
                && me.single().is_ok_and(|&here| landed(loading.pick, here, &islands));
            if !settled && loading.age < LOADING_MOST && !turned_down {
                return;
            }
            up.0 = false;
            0.0
        }
    };
    loading.lifting = Some(lifted);
    let left = 1.0 - lifted / LOADING_OUT;
    if left <= 0.0 {
        for (screen, _) in &screens {
            commands.entity(screen).despawn();
        }
        commands.remove_resource::<Loading>();
        return;
    }
    for (_, mut colour) in &mut screens {
        colour.0 = Color::BLACK.with_alpha(left);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How tall the selector is, in shares of the short side: its room
    /// round what is in it, its title's row, the gap under it, its rows of
    /// cards and the gaps between them, and the room at its bottom.
    fn height(follows: bool) -> f32 {
        let rows = shown().len() as f32;
        window::PAD + window::TAB_HEIGHT + 1.2 + rows * CARD_HEIGHT + (rows - 1.0) * CARD_GAP + bottom_room(follows)
    }

    #[test]
    fn the_cards_in_two_rows_fit_any_screen_lying_on_its_side() {
        // Every game once: the six.
        let mut every: Vec<Minigame> = CARDS.iter().flatten().copied().collect();
        assert_eq!(every.len(), 6);
        every.dedup();
        assert_eq!(every.len(), 6);
        for game in Minigame::TABLES {
            assert!(every.contains(&game), "{game:?}");
        }
        // Bridge Race's and Last Stand's are turned off: four stand, two to
        // a row, in the order they were in. With none off, the six as they
        // stood; with one, three over two; with all, no row at all.
        assert_eq!(shown(), [
            vec![Minigame::BridgeWars, Minigame::BlendIn],
            vec![Minigame::KnockOff, Minigame::HouseBuilder],
        ]);
        assert_eq!(laid(&[]), CARDS.map(|row| row.to_vec()));
        assert_eq!(laid(&[Minigame::LastStand]).iter().map(Vec::len).collect::<Vec<_>>(), [3, 2]);
        assert!(laid(&CARDS.concat()).is_empty());
        // As tall as the friends' panel, and with the leader's line no
        // taller than the screen leaves between its edges, on a tablet, a
        // monitor and a phone, the short side being the height.
        for wide in [4.0 / 3.0, 16.0 / 9.0, 19.5 / 9.0] {
            let (width, tall) = (100.0 * wide, 100.0);
            assert!(height(true) <= 92.0, "{}", height(true));
            assert!(height(false) <= window::HEIGHT + 1.0, "{}", height(false));
            assert!(height(true) <= tall - 2.0 * hud::EDGE);
            assert!(window::WIDTH <= width - 2.0 * hud::EDGE, "{wide}");
        }
        // And on each card its words: the name on one line, the line under
        // it on two at the most, and the price, with the room round them.
        // A letter of Inter is about half as wide as it is tall, a little
        // more for a name's capitals. Every game's, those turned off too,
        // on a card at its narrowest, three to a row.
        let inside = (window::WIDTH - 2.0 * window::PAD - 2.0 * CARD_GAP) / 3.0 - 2.0 * CARD_PAD;
        for game in every {
            let name = name_of(game).chars().count() as f32 * CARD_TEXT * 0.55;
            assert!(name <= inside, "{game:?}: its name is {name} wide in {inside}");
            let line = line_of(game).chars().count() as f32 * NOTE_TEXT * 0.5;
            assert!(line <= 2.0 * inside, "{game:?}: its line is {line} wide in two of {inside}");
            let (price, ..) = price(game, 0, true);
            let read = price.read();
            assert!(read.chars().count() as f32 * NOTE_TEXT * 0.5 + COIN_SIZE <= inside, "{game:?}: {read}");
        }
        let stacked = 2.0 * CARD_PAD + CARD_TEXT * 1.2 + 1.2 + LINE_ROOM + 1.2 + NOTE_TEXT * 1.2;
        assert!(stacked <= CARD_HEIGHT, "{stacked}");
    }

    #[test]
    fn each_card_says_what_its_game_costs_or_why_it_is_not_to_be_had() {
        let ui = ui();
        let words = &ui.words.games;
        // Every game at its fee, the four's by their tables, Bridge Wars' 50
        // by the rules, Home Builders' its own; blocked without the coins.
        assert_eq!(fee(Minigame::BridgeWars), 50);
        assert_eq!(fee(Minigame::HouseBuilder), ENTRY);
        for game in CARDS.iter().flatten().copied() {
            let cost = fee(game);
            if let Some(shape) = tables::shape(game) {
                assert_eq!(cost, shape.fee);
            }
            let (said, tint, able) = price(game, cost, true);
            assert!(able && tint.is_none(), "{game:?}");
            assert_eq!(said, Note::Price { words: words.price.clone(), coins: cost });
            let (said, tint, able) = price(game, cost - 1, true);
            assert!(!able && tint.is_some(), "{game:?}");
            assert_eq!(said, Note::Price { words: words.needs_coins.clone(), coins: cost });
        }
        // The coin's picture where the words say how many, and no "coins"
        // (Hajun: "(coin_icon) 50").
        let (before, coins, after) = price(Minigame::KnockOff, 50, true).0.parts();
        assert_eq!((before.as_str(), coins.as_deref(), after.as_str()), ("", Some("50"), ""));
        assert_eq!(price(Minigame::KnockOff, 0, true).0.read(), "Needs O 50");
        assert_eq!(price(Minigame::HouseBuilder, ENTRY, true).0.read(), "O 100");
        // Blend In needs people: offline it is not to be had, whatever you
        // have; the rest are.
        assert_eq!(price(Minigame::BlendIn, 10_000, false).0, Note::Words(words.online_only.clone()));
        assert!(!price(Minigame::BlendIn, 10_000, false).2);
        for game in [Minigame::BridgeWars, Minigame::KnockOff, Minigame::BridgeRace, Minigame::LastStand, Minigame::HouseBuilder] {
            assert!(price(game, 10_000, false).2, "{game:?}");
        }
        // Words with no price in them are words.
        assert_eq!(Note::Words("Free".into()).parts(), ("Free".to_string(), None, String::new()));
    }

    #[test]
    fn the_black_screen_lifts_where_each_game_is_played() {
        let mut islands = Islands::default();
        // Nothing loaded: nowhere to have landed.
        assert!(!landed(Minigame::KnockOff, Venue::KnockOff, &islands));
        islands.set_for_tests(Venue::Town);
        islands.set_for_tests(Venue::KnockOff);
        assert!(landed(Minigame::KnockOff, Venue::KnockOff, &islands));
        assert!(!landed(Minigame::KnockOff, Venue::Town, &islands), "not yet seated");
        // Blend In on the town's map, which is loaded with the town.
        assert!(landed(Minigame::BlendIn, Venue::BlendIn, &islands));
        assert!(!landed(Minigame::BlendIn, Venue::Town, &islands));
        // Home Builders anywhere loaded, as ever.
        assert!(landed(Minigame::HouseBuilder, Venue::Town, &islands));
    }

    #[test]
    fn a_refusal_lifts_the_black_screen_at_once_for_a_table_alone() {
        use bevy::ecs::system::RunSystemOnce;
        use roundtown_net::Refusal;
        for (pick, at_once) in [
            (Minigame::KnockOff, true),
            (Minigame::BlendIn, true),
            (Minigame::BridgeWars, false),
            (Minigame::HouseBuilder, false),
        ] {
            let mut world = World::new();
            world.init_resource::<Time>();
            world.init_resource::<Islands>();
            world.init_resource::<DialogUp>();
            world.init_resource::<Messages<Tell>>();
            world.init_resource::<Messages<arena::Ask>>();
            world.init_resource::<Messages<builder::Ask>>();
            world.init_resource::<Messages<table::Ask>>();
            world.spawn((Player, Venue::Town));
            // Down a moment, the game asked for, and nowhere yet.
            world.insert_resource(Loading {
                pick,
                age: LOADING_IN + 0.1,
                fired: true,
                lifting: None,
            });
            world.resource_mut::<Messages<Tell>>().write(Tell::Refused(Refusal::Busy));
            world.run_system_once(load).expect("loaded");
            assert_eq!(world.resource::<Loading>().lifting.is_some(), at_once, "{pick:?}");
            // Nothing across the screen from here: a table says why itself.
            assert_eq!(world.query::<&Text>().iter(&world).count(), 0, "{pick:?}");
        }
    }
}
