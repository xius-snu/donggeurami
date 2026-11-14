//! The game selector, behind the gamepad button down the right edge of the
//! screen (`hud::GamesButton`; Hajun, the night of 2026-11-07: "when u press the game
//! icon ui button, it will open a popup game selector menu with 3 games
//! (the first one is called 'Bridge Wars' the second is called 'Home
//! Builders' and third is a ??? coming soon. you select one and you get a
//! quick black loading screen and then when the new map is loaded, the
//! player is teleported"): a dialog like the settings' window, with the
//! three games on cards across it, the third blocked out. Only in the town.
//! It is drawn as the friends' panel is (`window`, since 2026-11-14, when
//! Hajun asked for that panel's style everywhere): the same dark
//! see-through window, as wide, with nothing dimmed round it, "Games" along
//! its top from the left, the bare white cross in its corner, and its cards
//! the dark of that panel's buttons. Until then it was a black box in the
//! middle of the dimmed screen under a gold title, its cross a disc hung
//! off its corner.
//!
//! Choosing Bridge Wars or Home Builders, the dialog goes and a black
//! screen comes down ([`Loading`]): [`LOADING_IN`] seconds later, under it,
//! the game is gone into — Bridge Wars' waiting room (`arena::Ask::Enter`),
//! or House Builder's lobby, paying its fee (`builder::Ask::Join`, as the
//! House Builder's Play did; the card is blocked out without the coins) —
//! and once the island you are on has loaded and a moment more has passed,
//! it lifts. Should nothing come of the choice, as when the server turns
//! it down, it lifts after [`LOADING_MOST`] seconds all the same. While it
//! is down you stand still, as under a dialog (`hud::DialogUp`).

use bevy::prelude::*;
use roundtown_net::rules::ENTRY;

use crate::arena;
use crate::builder::{self, DIALOG_LAYER};
use crate::friends::Friends;
use crate::hud::{self, DialogUp, GamesButton, TakesPress};
use crate::island::{Islands, Venue};
use crate::lobby::Balance;
use crate::net::Online;
use crate::ui::{fill, ui};
use crate::window;
use crate::{CloseUp, Player};

/// Sizes, as shares of the short side of the screen, where they are not
/// the windows' own (`window`): how tall a game's card is (the three share
/// the window's width between them), the gap between the cards, and the
/// game's name on one and the line under it.
const CARD_HEIGHT: f32 = 30.0;
const CARD_GAP: f32 = 2.2;
const CARD_TEXT: f32 = 4.0;
const NOTE_TEXT: f32 = 2.8;
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

/// What a press on one of the dialog's buttons does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Choice {
    BridgeWars,
    HomeBuilders,
    Close,
}

/// A game chosen, to be gone into under the black screen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pick {
    BridgeWars,
    HomeBuilders,
}

/// Whether the dialog is up.
#[derive(Resource, Default)]
struct Selector {
    open: bool,
}

/// The black screen over everything.
#[derive(Component)]
struct LoadingScreen;

/// The black screen, from coming down to lifting: what was chosen, how long
/// it has been down, whether the game has been gone into yet, and how long
/// it has been lifting, once it is.
#[derive(Resource)]
struct Loading {
    pick: Pick,
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
            .before(arena::ArenaSystems),
    );
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
        open(&mut commands, &assets, balance.0 >= ENTRY, follows);
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
        Choice::BridgeWars => begin(&mut commands, Pick::BridgeWars),
        Choice::HomeBuilders => begin(&mut commands, Pick::HomeBuilders),
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
/// along its top, the cross in its corner, and the three games on cards
/// across it. Its root is over the whole screen, shows nothing, and shuts
/// the dialog at a tap. Home Builders is blocked out when you cannot pay
/// for it, and says what it needs; and both games when you `follows`
/// someone else's group, whose leader starts its games (since 2026-11-13,
/// Hajun: "when youre in the group with ur friend, only the leader can
/// queue for a minigame"), with a line along the window's bottom that says
/// so.
fn open(commands: &mut Commands, assets: &AssetServer, affordable: bool, follows: bool) {
    let ui = ui();
    let (colours, words) = (ui.colours.window, &ui.words.games);
    let coins = hud::money(ENTRY);
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
                    bottom: Val::VMin(if follows { window::PAD + window::NOTICE_TEXT + 1.4 } else { window::PAD }),
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
    let cards = commands
        .spawn((
            ChildOf(panel),
            Node {
                column_gap: Val::VMin(CARD_GAP),
                margin: UiRect::top(Val::VMin(1.2)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let mut wars = commands.spawn((ChildOf(cards), card(words.bridge_wars.clone(), words.free.clone(), None, !follows)));
    if !follows {
        wars.insert((Choice::BridgeWars, TakesPress, Button));
    }
    // What it costs; or, without the coins, what it needs, in the colour
    // of a line that says why something cannot be had.
    let (note, tint) = if affordable {
        (fill(&ui.words.house_builder.price, &[("coins", &coins)]), None)
    } else {
        (fill(&ui.words.house_builder.need_coins, &[("coins", &coins)]), Some(colours.notice))
    };
    let able = affordable && !follows;
    let mut home = commands.spawn((ChildOf(cards), card(words.home_builders.clone(), note, tint, able)));
    if able {
        home.insert((Choice::HomeBuilders, TakesPress, Button));
    }
    commands.spawn((ChildOf(cards), card(words.coming_soon.clone(), words.coming_soon_note.clone(), None, false)));
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

/// One game's card, a share of the window's width: its name, and a line
/// under it saying what it costs, or that it is not to be had, in
/// `note_colour` if that is not the card's own; lit under the pointer if it
/// can be chosen, and blocked out if not.
fn card(name: String, note: String, note_colour: Option<Color>, choosable: bool) -> impl Bundle {
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
            padding: UiRect::all(Val::VMin(1.5)),
            border_radius: BorderRadius::all(Val::VMin(window::CARD_ROUNDED)),
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![
            builder::label(name, CARD_TEXT, ink),
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
fn begin(commands: &mut Commands, pick: Pick) {
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

/// Goes into the game chosen once the black screen has been down a moment,
/// and lifts the screen once the island you are on has loaded and a moment
/// more has passed, or once it has been down long enough that nothing is
/// coming of the choice.
#[allow(clippy::too_many_arguments)]
fn load(
    time: Res<Time>,
    loading: Option<ResMut<Loading>>,
    me: Query<&Venue, With<Player>>,
    islands: Res<Islands>,
    mut screens: Query<(Entity, &mut BackgroundColor), With<LoadingScreen>>,
    mut up: ResMut<DialogUp>,
    mut arena_asks: MessageWriter<arena::Ask>,
    mut builder_asks: MessageWriter<builder::Ask>,
    mut commands: Commands,
) {
    let Some(mut loading) = loading else {
        return;
    };
    let dt = time.delta_secs();
    loading.age += dt;
    if !loading.fired && loading.age >= LOADING_IN {
        loading.fired = true;
        match loading.pick {
            Pick::BridgeWars => {
                arena_asks.write(arena::Ask::Enter);
            }
            Pick::HomeBuilders => {
                builder_asks.write(builder::Ask::Join);
            }
        }
    }
    let lifted = match loading.lifting {
        Some(lifted) => lifted + dt,
        None => {
            // Landed: on an island that has loaded, and for Bridge Wars on
            // its map, which online the server has to seat you on first.
            let landed = me.single().is_ok_and(|&here| {
                islands.get(here).is_some() && (loading.pick != Pick::BridgeWars || here == Venue::Arena)
            });
            let settled = loading.fired && loading.age >= LOADING_IN + LOADING_HOLD && landed;
            if !settled && loading.age < LOADING_MOST {
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
