//! The game selector, behind the gamepad button down the right edge of the
//! screen (`hud::GamesButton`; Hajun, the night of 2026-11-07: "when u press the game
//! icon ui button, it will open a popup game selector menu with 3 games
//! (the first one is called 'Bridge Wars' the second is called 'Home
//! Builders' and third is a ??? coming soon. you select one and you get a
//! quick black loading screen and then when the new map is loaded, the
//! player is teleported"): a dialog like the settings' window, with the
//! three games on cards across it, the third blocked out. Only in the town.
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
use crate::builder::{self, DIALOG_LAYER, Fill, TITLE};
use crate::friends::Friends;
use crate::hud::{self, DialogUp, GamesButton, TakesPress};
use crate::island::{Islands, Venue};
use crate::lobby::Balance;
use crate::net::Online;
use crate::ui::{fill, ui};
use crate::{CloseUp, Player};

/// Sizes, as shares of the short side of the screen: a game's card, the
/// gap between the cards, the game's name on it and the line under it, and
/// the cross on the dialog's corner.
const CARD_WIDTH: f32 = 24.0;
const CARD_HEIGHT: f32 = 26.0;
const CARD_GAP: f32 = 3.0;
const CARD_TEXT: f32 = 4.0;
const NOTE_TEXT: f32 = 3.0;
const CLOSE: f32 = 9.0;
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
/// through on its way out, so that a tap on a card is not also a tap on the
/// dark round it.
#[allow(clippy::too_many_arguments)]
fn choose(
    click: On<Pointer<Click>>,
    buttons: Query<(), With<GamesButton>>,
    choices: Query<&Choice>,
    me: Query<(&Venue, &Balance), With<Player>>,
    close_up: Res<CloseUp>,
    loading: Option<Res<Loading>>,
    (online, friends): (Res<Online>, Res<Friends>),
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
        open(&mut commands, balance.0 >= ENTRY, follows);
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

/// The dialog: the screen dimmed behind it, its title, the three games on
/// cards across it, and the cross on its corner. Home Builders is blocked
/// out when you cannot pay for it; and both games when you `follows`
/// someone else's group, whose leader starts its games (since 2026-11-13,
/// Hajun: "when youre in the group with ur friend, only the leader can
/// queue for a minigame"), with a line under the cards that says so.
fn open(commands: &mut Commands, affordable: bool, follows: bool) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.games);
    let coins = hud::money(ENTRY);
    let mut panel = builder::dimmed(commands, (GamesDialog, Choice::Close));
    panel.with_children(|panel| {
        panel.spawn(builder::label(words.title.clone(), TITLE, colours.gold));
        panel
            .spawn((
                Node {
                    column_gap: Val::VMin(CARD_GAP),
                    margin: UiRect::vertical(Val::VMin(1.0)),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|cards| {
                let mut wars = cards.spawn(card(words.bridge_wars.clone(), words.free.clone(), colours.text_soft, !follows));
                if !follows {
                    wars.insert((Choice::BridgeWars, TakesPress, Button));
                }
                let (note, tint) = if affordable {
                    (fill(&ui.words.house_builder.price, &[("coins", &coins)]), colours.text_soft)
                } else {
                    (fill(&ui.words.house_builder.need_coins, &[("coins", &coins)]), colours.warn)
                };
                let able = affordable && !follows;
                let mut home = cards.spawn(card(words.home_builders.clone(), note, tint, able));
                if able {
                    home.insert((Choice::HomeBuilders, TakesPress, Button));
                }
                cards.spawn(card(words.coming_soon.clone(), words.coming_soon_note.clone(), colours.off_text, false));
            });
        if follows {
            panel.spawn(builder::label(ui.words.friends.only_leader.clone(), NOTE_TEXT, colours.warn));
        }
        panel.spawn(hud::close_cross(CLOSE, Choice::Close));
    });
}

/// One game's card: its name, and a line under it saying what it costs, or
/// that it is not to be had; lit under the pointer if it can be chosen,
/// and blocked out if not.
fn card(name: String, note: String, note_colour: Color, choosable: bool) -> impl Bundle {
    let colours = ui().colours;
    let (fill, ink) = if choosable {
        (
            Fill {
                idle: colours.quiet,
                lit: colours.quiet_lit,
            },
            colours.text,
        )
    } else {
        (
            Fill {
                idle: colours.off,
                lit: colours.off,
            },
            colours.off_text,
        )
    };
    (
        Node {
            width: Val::VMin(CARD_WIDTH),
            height: Val::VMin(CARD_HEIGHT),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::VMin(1.6),
            padding: UiRect::all(Val::VMin(1.5)),
            border_radius: BorderRadius::all(Val::VMin(3.0)),
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![
            builder::label(name, CARD_TEXT, ink),
            builder::label(note, NOTE_TEXT, note_colour),
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
