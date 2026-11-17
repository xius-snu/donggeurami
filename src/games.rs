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
//! Choosing a game, the dialog goes and a black screen comes down
//! ([`Loading`]): [`LOADING_IN`] seconds later, under it, the game is gone
//! into — Bridge Wars' waiting room (`arena::Ask::Enter`), House Builder's
//! lobby, paying its fee (`builder::Ask::Join`, as the House Builder's Play
//! did), or a table of one of the four (`table::Ask::Join`) — and once you
//! stand on the game's island and it has loaded, and a moment more has
//! passed, it lifts. Should nothing come of the choice, it lifts after
//! [`LOADING_MOST`] seconds all the same; should the server turn it down
//! (`Tell::Refused`), at once, with the reason across the screen. While it
//! is down you stand still, as under a dialog (`hud::DialogUp`).

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
use crate::ui::{fill, ui};
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
/// The cards, row by row, in the order Hajun listed the games: the new
/// four after Bridge Wars, and Home Builders, stalled, last.
const CARDS: [[Minigame; 3]; 2] = [
    [Minigame::BridgeWars, Minigame::BlendIn, Minigame::KnockOff],
    [Minigame::BridgeRace, Minigame::LastStand, Minigame::HouseBuilder],
];
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

/// What `game` costs to play, in coins.
fn fee(game: Minigame) -> u32 {
    match game {
        Minigame::BridgeWars => 0,
        Minigame::HouseBuilder => ENTRY,
        game => tables::shape(game).map_or(0, |shape| shape.fee),
    }
}

/// What `game`'s card says under its line, for someone with `balance`
/// coins, `online` or not: what it costs, or why it is not to be had, in
/// the colour of a line that says why something cannot be had; and
/// whether it is to be had. Bridge Wars is free; the rest are blocked out
/// without the coins; and Blend In offline, where there is nobody to play
/// it with.
fn price(game: Minigame, balance: u32, online: bool) -> (String, Option<Color>, bool) {
    let ui = ui();
    let words = &ui.words.games;
    let notice = ui.colours.window.notice;
    let coins = hud::money(fee(game));
    match game {
        Minigame::BridgeWars => (words.free.clone(), None, true),
        Minigame::BlendIn if !online => (words.online_only.clone(), Some(notice), false),
        // Home Builders' in House Builder's own words, as ever.
        Minigame::HouseBuilder if balance >= ENTRY => (fill(&ui.words.house_builder.price, &[("coins", &coins)]), None, true),
        Minigame::HouseBuilder => (fill(&ui.words.house_builder.need_coins, &[("coins", &coins)]), Some(notice), false),
        game if balance >= fee(game) => (fill(&words.price, &[("coins", &coins)]), None, true),
        _ => (fill(&words.needs_coins, &[("coins", &coins)]), Some(notice), false),
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
/// rows across it ([`CARDS`]). Its root is over the whole screen, shows
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
    for (index, games) in CARDS.iter().enumerate() {
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
            let mut card = commands.spawn((ChildOf(row), card(name_of(game), line_of(game), note, tint, able)));
            if able {
                card.insert((Choice::Game(game), TakesPress, Button));
            }
        }
        // The game to come, were it back ([`COMING_SOON`]): at the end of
        // the last row, blocked out.
        if COMING_SOON && index + 1 == CARDS.len() {
            commands.spawn((
                ChildOf(row),
                card(words.coming_soon.clone(), String::new(), words.coming_soon_note.clone(), None, false),
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
/// it is, and under that what it costs, or why it is not to be had, in
/// `note_colour` if that is not the card's own; lit under the pointer if it
/// can be chosen, and blocked out if not.
fn card(name: String, line: String, note: String, note_colour: Option<Color>, choosable: bool) -> impl Bundle {
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
            builder::label(line, NOTE_TEXT, soft),
            builder::label(note, NOTE_TEXT, note_colour.unwrap_or(soft)),
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
/// the choice, or at once if the server has turned it down, saying why: a
/// table says why for itself (`table`), and for Bridge Wars and House
/// Builder it is said here.
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
    let refused = tells
        .read()
        .filter_map(|tell| match tell {
            Tell::Refused(refusal) => Some(*refusal),
            _ => None,
        })
        .last()
        .filter(|_| loading.fired);
    let lifted = match loading.lifting {
        Some(lifted) => lifted + dt,
        None => {
            let settled = loading.fired
                && loading.age >= LOADING_IN + LOADING_HOLD
                && me.single().is_ok_and(|&here| landed(loading.pick, here, &islands));
            if !settled && loading.age < LOADING_MOST && refused.is_none() {
                return;
            }
            if let Some(refusal) = refused
                && !loading.pick.at_a_table()
            {
                table::say_refused(&mut commands, refusal);
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
    /// round what is in it, its title's row, the gap under it, the two rows
    /// of cards and the gap between them, and the room at its bottom.
    fn height(follows: bool) -> f32 {
        window::PAD + window::TAB_HEIGHT + 1.2 + CARDS.len() as f32 * CARD_HEIGHT + CARD_GAP + bottom_room(follows)
    }

    #[test]
    fn six_cards_in_two_rows_fit_any_screen_lying_on_its_side() {
        // Every game once: the six.
        let mut every: Vec<Minigame> = CARDS.iter().flatten().copied().collect();
        assert_eq!(every.len(), 6);
        every.dedup();
        assert_eq!(every.len(), 6);
        for game in Minigame::TABLES {
            assert!(every.contains(&game), "{game:?}");
        }
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
        // more for a name's capitals.
        let inside = (window::WIDTH - 2.0 * window::PAD - 2.0 * CARD_GAP) / 3.0 - 2.0 * CARD_PAD;
        for game in every {
            let name = name_of(game).chars().count() as f32 * CARD_TEXT * 0.55;
            assert!(name <= inside, "{game:?}: its name is {name} wide in {inside}");
            let line = line_of(game).chars().count() as f32 * NOTE_TEXT * 0.5;
            assert!(line <= 2.0 * inside, "{game:?}: its line is {line} wide in two of {inside}");
            let (price, ..) = price(game, 0, true);
            assert!(price.chars().count() as f32 * NOTE_TEXT * 0.5 <= inside, "{game:?}: {price}");
        }
        let stacked = 2.0 * CARD_PAD + CARD_TEXT * 1.2 + 1.2 + 2.0 * NOTE_TEXT * 1.2 + 1.2 + NOTE_TEXT * 1.2;
        assert!(stacked <= CARD_HEIGHT, "{stacked}");
    }

    #[test]
    fn each_card_says_what_its_game_costs_or_why_it_is_not_to_be_had() {
        let ui = ui();
        let words = &ui.words.games;
        // Bridge Wars is free, to anyone.
        assert_eq!(price(Minigame::BridgeWars, 0, false), (words.free.clone(), None, true));
        // The four at their fees, by the rules; blocked without the coins.
        for game in Minigame::TABLES {
            let cost = tables::shape(game).unwrap().fee;
            assert_eq!(fee(game), cost);
            let (said, tint, able) = price(game, cost, true);
            assert!(able && tint.is_none(), "{game:?}");
            assert_eq!(said, fill(&words.price, &[("coins", &hud::money(cost))]));
            let (said, tint, able) = price(game, cost - 1, true);
            assert!(!able && tint.is_some(), "{game:?}");
            assert_eq!(said, fill(&words.needs_coins, &[("coins", &hud::money(cost))]));
        }
        // Blend In needs people: offline it is not to be had, whatever you
        // have; the other three are.
        assert_eq!(price(Minigame::BlendIn, 10_000, false).0, words.online_only);
        assert!(!price(Minigame::BlendIn, 10_000, false).2);
        for game in [Minigame::KnockOff, Minigame::BridgeRace, Minigame::LastStand] {
            assert!(price(game, 10_000, false).2, "{game:?}");
        }
        // Home Builders as ever, in House Builder's own words.
        let coins = hud::money(ENTRY);
        assert_eq!(price(Minigame::HouseBuilder, ENTRY, false).0, fill(&ui.words.house_builder.price, &[("coins", &coins)]));
        assert!(!price(Minigame::HouseBuilder, ENTRY - 1, true).2);
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
}
