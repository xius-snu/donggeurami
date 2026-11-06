//! Settings: the gear beside your name, and the window it opens, with a tab for
//! each kind (Hajun, 2026-10-31: "a setting icon in top right corner ... a
//! semi transparent greyish rectangular popup which will have all sorts of
//! different useful settings"): Graphics and Controls, with nothing in them
//! yet, and Accounts, with your name and a pencil beside it to change it,
//! Sign in with Google and Sign in with Apple, blocked out until there is a
//! sign-in to be had, and Delete account. From 2026-10-08 until then your
//! account was a round button beside your balance, shown online only; the
//! gear was top right for an hour, until Hajun's sketch of the screen put it
//! beside your name (`hud`).
//!
//! The pencil opens a smaller dialog with your name in a box to type into
//! (`signup::NameBox`) and Save, which makes it your name everywhere the
//! sign-up's did (`signup::wear`): in the town's room, over your head, and,
//! online, with the server.
//!
//! Deleting asks once more, then asks the login to forget the account and
//! everything kept about it (`login::delete`): Apple asks for a way to
//! delete an account inside the app, even one nobody signed up for
//! (`MULTIPLAYER.md`, "Before a public online release"), and every account
//! here is one of those, made by the login the first time a device asks to
//! play. Then this device forgets its secret and who it was
//! (`signup::Profile`), and starts over from the sign-up's pages as someone
//! new.
//!
//! While the window is up it is a dialog like House Builder's others
//! ([`DialogUp`]): you stand still, and presses go to it alone. The cross on
//! its corner, a tap on the dark round it, or on desktop `Escape`, shuts it;
//! from the smaller dialogs, back to the window.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, TryRecvError};

use bevy::prelude::*;

use crate::builder::{self, BODY, BUTTON_HEIGHT, BUTTON_WIDTH, DIALOG_LAYER, Fill, SMALL, TITLE};
use crate::hud::{self, BarLeft, DialogUp, TakesPress};
use crate::lobby::Lobby;
use crate::net::{Link, Online};
use crate::signup::{self, NameBox, Profile, Save, Signup};
use crate::ui::{UiChanged, ui};
use crate::Player;

/// Sizes, as shares of the short side of the screen: the window and what is
/// in it; the pencil beside your name; and the box the name is typed into,
/// in the dialog. The colours and the words are `assets/ui.ron`'s (`ui`):
/// the window a grey glass, greyer than the dialogs' dark blue, and
/// see-through enough for the town to show behind it; a sign-in not to be
/// had yet blocked out, its words faint.
const WIDTH: f32 = 84.0;
const HEIGHT: f32 = 72.0;
const PADDING: f32 = 2.6;
const TAB_TEXT: f32 = 3.4;
const CLOSE: f32 = 9.0;
const PENCIL: f32 = 6.0;
const NAME_FIELD: f32 = 70.0;
/// The Accounts tab's buttons: wider than House Builder's, for "Sign in with
/// Google" to fit on one line, with words a little smaller than its.
const CHOICE_WIDTH: f32 = 54.0;
const CHOICE_TEXT: f32 = 3.6;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Settings>()
        .init_resource::<Deleting>()
        .add_observer(choose)
        .add_systems(Update, (spawn_button, redraw, keys, edit_name, light_save, deleted).chain());
}

/// The window, while it is up, and the tab it shows.
#[derive(Resource, Default)]
struct Settings {
    window: Option<Entity>,
    tab: Tab,
    /// The pencil, drawn once.
    pencil: Handle<Image>,
    /// The dialog that changes your name is up, in the window's place.
    renaming: bool,
    /// A press on the dialog's buttons, to be acted on ([`edit_name`]).
    pressed: Option<Choice>,
}

/// The gear beside your name.
#[derive(Component)]
struct SettingsButton;

/// The root of the window, or of one of the smaller dialogs.
#[derive(Component)]
struct SettingsDialog;

/// The window's tabs, in order.
#[derive(Component, Clone, Copy, PartialEq, Eq, Default, Debug)]
enum Tab {
    #[default]
    Graphics,
    Controls,
    Accounts,
}

impl Tab {
    const ALL: [Tab; 3] = [Tab::Graphics, Tab::Controls, Tab::Accounts];

    /// What the tab is called, as the file has it.
    fn title(self) -> String {
        let ui = ui();
        let words = &ui.words.settings;
        match self {
            Tab::Graphics => words.graphics.clone(),
            Tab::Controls => words.controls.clone(),
            Tab::Accounts => words.accounts.clone(),
        }
    }
}

/// What a press on one of the settings' buttons does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Choice {
    /// Open the window.
    Open,
    /// Put it away again.
    Close,
    /// Show this tab.
    Show(Tab),
    /// Change your name: the pencil.
    Rename,
    /// Make the name typed your name.
    SaveName,
    /// Keep the name you had: back to the window.
    CancelRename,
    /// Delete the account: asked once more first.
    Delete,
    /// Delete it, really.
    Confirm,
}

/// The login's answer, while it is being asked to delete your account.
#[derive(Resource, Default)]
struct Deleting(Option<Mutex<Receiver<Result<(), String>>>>);

/// The gear, on the bar top left as it comes up: as the app opens, and again
/// whenever the bar is put up afresh in the file's colours (`hud::restyle`).
fn spawn_button(
    bars: Query<Entity, Added<BarLeft>>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut settings: ResMut<Settings>,
    mut commands: Commands,
) {
    for bar in &bars {
        if settings.pencil == Handle::default() {
            settings.pencil = images.add(hud::pencil());
        }
        // After your name and the person beside it, in the buttons' own
        // style, with Material's gear on it (`hud::ICONS`; one drawn in code
        // until 2026-11-01, when Hajun asked for the one in their
        // screenshot).
        commands.spawn((
            ChildOf(bar),
            SettingsButton,
            Choice::Open,
            hud::round_button(assets.load("icons/settings.png"), hud::SMALL_BUTTON, Node::default()),
        ));
    }
}

/// Draws the window again in the file's colours and words as it changes on
/// desktop (`ui::reread`), on the tab it is on; not while one of the
/// smaller dialogs is up in its place.
fn redraw(
    changed: Res<UiChanged>,
    online: Res<Online>,
    profile: Res<Profile>,
    mut settings: ResMut<Settings>,
    dialogs: Query<Entity, With<SettingsDialog>>,
    mut commands: Commands,
) {
    if !changed.is_changed() || changed.is_added() || settings.window.is_none() {
        return;
    }
    show(&mut commands, &mut settings, &dialogs, &online, &profile);
}

/// Opens the window, and goes on from there as its buttons are pressed.
#[allow(clippy::too_many_arguments)]
fn choose(
    click: On<Pointer<Click>>,
    choices: Query<&Choice>,
    online: Res<Online>,
    profile: Res<Profile>,
    link: Res<Link>,
    mut settings: ResMut<Settings>,
    mut up: ResMut<DialogUp>,
    dialogs: Query<Entity, With<SettingsDialog>>,
    mut deleting: ResMut<Deleting>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    let Ok(&choice) = choices.get(click.entity) else {
        return;
    };
    match choice {
        Choice::Open if !up.0 => {
            open(&mut commands, &mut settings, &mut up, &dialogs, &online, &profile);
        }
        Choice::Open => {}
        Choice::Close => {
            close(&mut commands, &mut settings, &mut up, &dialogs);
        }
        Choice::Show(tab) => {
            if settings.tab != tab {
                settings.tab = tab;
                show(&mut commands, &mut settings, &dialogs, &online, &profile);
            }
        }
        Choice::Rename | Choice::SaveName | Choice::CancelRename => {
            settings.pressed = Some(choice);
        }
        Choice::Delete => {
            take_down(&mut commands, &dialogs);
            settings.window = None;
            sure(&mut commands);
        }
        Choice::Confirm => {
            close(&mut commands, &mut settings, &mut up, &dialogs);
            let ui = ui();
            let words = &ui.words.settings;
            match link.delete_account() {
                Some(answer) => {
                    deleting.0 = Some(Mutex::new(answer));
                    hud::announce(&mut commands, None, &words.deleting, hud::CAPTION, 2.0);
                }
                None => {
                    hud::announce(&mut commands, None, &words.not_online_try, hud::CAPTION, 3.0);
                }
            }
        }
    }
}

/// Puts the window up, on the tab it was last on, as a dialog.
fn open(
    commands: &mut Commands,
    settings: &mut Settings,
    up: &mut DialogUp,
    dialogs: &Query<Entity, With<SettingsDialog>>,
    online: &Online,
    profile: &Profile,
) {
    show(commands, settings, dialogs, online, profile);
    if !up.0 {
        up.0 = true;
    }
}

/// Puts the window up on its tab, in place of whatever of the settings was
/// up.
fn show(
    commands: &mut Commands,
    settings: &mut Settings,
    dialogs: &Query<Entity, With<SettingsDialog>>,
    online: &Online,
    profile: &Profile,
) {
    take_down(commands, dialogs);
    settings.renaming = false;
    let pencil = settings.pencil.clone();
    settings.window = Some(window(commands, settings.tab, online, profile, pencil));
}

/// Takes the window, or the dialog, down.
fn close(
    commands: &mut Commands,
    settings: &mut Settings,
    up: &mut DialogUp,
    dialogs: &Query<Entity, With<SettingsDialog>>,
) {
    take_down(commands, dialogs);
    settings.window = None;
    settings.renaming = false;
    if up.0 {
        up.0 = false;
    }
}

fn take_down(commands: &mut Commands, dialogs: &Query<Entity, With<SettingsDialog>>) {
    for dialog in dialogs {
        commands.entity(dialog).despawn();
    }
}

/// The window, showing `tab`: the screen dimmed behind it, its tabs along
/// its top, the cross on its corner, and what the tab holds. Returns its
/// root, the dark.
fn window(
    commands: &mut Commands,
    tab: Tab,
    online: &Online,
    profile: &Profile,
    pencil: Handle<Image>,
) -> Entity {
    let colours = ui().colours;
    let dark = commands
        .spawn((
            SettingsDialog,
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
            BackgroundColor(colours.dim),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let panel = commands
        .spawn((
            ChildOf(dark),
            Node {
                width: Val::VMin(WIDTH),
                height: Val::VMin(HEIGHT),
                flex_direction: FlexDirection::Column,
                row_gap: Val::VMin(2.0),
                padding: UiRect::all(Val::VMin(PADDING)),
                border_radius: BorderRadius::all(Val::VMin(3.0)),
                ..default()
            },
            BackgroundColor(colours.settings.window),
        ))
        .id();
    commands.spawn((ChildOf(panel), close_button()));
    commands
        .spawn((
            ChildOf(panel),
            Node {
                column_gap: Val::VMin(1.2),
                // Clear of the cross on the corner.
                margin: UiRect::right(Val::VMin(CLOSE * 0.5)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|tabs| {
            for each in Tab::ALL {
                tabs.spawn(tab_button(each, each == tab));
            }
        });
    let body = commands
        .spawn((
            ChildOf(panel),
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(2.4),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    if tab == Tab::Accounts {
        accounts(commands, body, online, profile, pencil);
    }
    dark
}

/// The Accounts tab: who you are, with the pencil to change it, the
/// sign-ins to come, blocked out, and Delete account.
fn accounts(
    commands: &mut Commands,
    body: Entity,
    online: &Online,
    profile: &Profile,
    pencil: Handle<Image>,
) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.settings);
    // The name you chose, which the server takes on a moment after it is
    // changed; the server's only before you have chosen one.
    let name = if profile.set() {
        profile.name.clone()
    } else {
        online.me().map_or(words.not_signed_up.clone(), |me| me.name.clone())
    };
    commands
        .spawn((
            ChildOf(body),
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::VMin(2.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn(builder::label(name, BODY, colours.text));
            row.spawn((
                Choice::Rename,
                TakesPress,
                Button,
                Node {
                    width: Val::VMin(PENCIL),
                    height: Val::VMin(PENCIL),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(colours.quiet),
                Fill {
                    idle: colours.quiet,
                    lit: colours.quiet_lit,
                },
                children![(
                    ImageNode {
                        image: pencil,
                        color: colours.hud.icon,
                        ..default()
                    },
                    Node {
                        width: Val::Percent(66.0),
                        height: Val::Percent(66.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                )],
            ));
        });
    commands.spawn((
        ChildOf(body),
        builder::label(
            if online.is() { words.guest.as_str() } else { words.not_online.as_str() },
            SMALL,
            colours.text_soft,
        ),
    ));
    commands.spawn((ChildOf(body), blocked(&words.sign_in_google)));
    commands.spawn((ChildOf(body), blocked(&words.sign_in_apple)));
    commands.spawn((
        ChildOf(body),
        button(&words.delete_account, Choice::Delete, warning(), CHOICE_WIDTH),
    ));
}

/// One of the window's tabs: the one shown in gold, the rest quiet.
fn tab_button(tab: Tab, chosen: bool) -> impl Bundle {
    let (fill, ink) = if chosen {
        let colours = ui().colours;
        (
            Fill {
                idle: colours.gold,
                lit: colours.gold,
            },
            colours.on_gold,
        )
    } else {
        quiet()
    };
    (
        Choice::Show(tab),
        TakesPress,
        Button,
        Node {
            flex_grow: 1.0,
            flex_basis: Val::ZERO,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::VMin(1.0), Val::VMin(1.4)),
            border_radius: BorderRadius::all(Val::VMin(2.2)),
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![builder::label(tab.title(), TAB_TEXT, ink)],
    )
}

/// The cross on the window's corner, as the shop's.
fn close_button() -> impl Bundle {
    let colours = ui().colours;
    (
        Choice::Close,
        TakesPress,
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::VMin(-CLOSE * 0.35),
            right: Val::VMin(-CLOSE * 0.35),
            width: Val::VMin(CLOSE),
            height: Val::VMin(CLOSE),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(colours.close),
        BorderColor::all(colours.close_rim),
        children![
            (
                hud::shape(22.0, 44.0, 56.0, 12.0),
                UiTransform::from_rotation(Rot2::degrees(45.0)),
            ),
            (
                hud::shape(22.0, 44.0, 56.0, 12.0),
                UiTransform::from_rotation(Rot2::degrees(-45.0)),
            ),
        ],
    )
}

/// A sign-in that is not to be had yet: a button blocked out, which does
/// nothing.
fn blocked(text: &str) -> impl Bundle {
    let colours = ui().colours.settings;
    (
        Node {
            width: Val::VMin(CHOICE_WIDTH),
            height: Val::VMin(BUTTON_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(BUTTON_HEIGHT * 0.5)),
            ..default()
        },
        BackgroundColor(colours.blocked),
        Pickable::IGNORE,
        children![builder::label(text, CHOICE_TEXT, colours.blocked_text)],
    )
}

/// The smaller dialog that changes your name: the name in a box to type
/// into, Cancel, and Save. On a phone it slides up out from under the
/// keyboard (`signup::LiftsForKeyboard`).
fn renaming(commands: &mut Commands, name: &str) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.settings);
    builder::dimmed(commands, (SettingsDialog, Choice::CancelRename))
        .insert(signup::LiftsForKeyboard::default())
        .with_children(|panel| {
            panel.spawn(hud::words(words.edit_name.as_str(), TITLE, colours.gold));
            panel.spawn(signup::name_box(name, NAME_FIELD));
            panel.spawn(builder::choices()).with_children(|choices| {
                choices.spawn(button(&words.cancel, Choice::CancelRename, quiet(), BUTTON_WIDTH));
                choices.spawn((
                    Save,
                    button(
                        &words.save,
                        Choice::SaveName,
                        signup::save_look(true, signup::gold_button()),
                        BUTTON_WIDTH,
                    ),
                ));
            });
        });
}

/// Asked once more, before an account goes for good.
fn sure(commands: &mut Commands) {
    let ui = ui();
    let (colours, words) = (ui.colours, &ui.words.settings);
    builder::dimmed(commands, (SettingsDialog, Choice::Close)).with_children(|panel| {
        panel.spawn(hud::words(words.delete_sure.as_str(), TITLE, colours.gold));
        panel.spawn(builder::label(words.delete_warning.as_str(), BODY, colours.text));
        panel.spawn(builder::choices()).with_children(|choices| {
            choices.spawn(button(&words.cancel, Choice::Close, quiet(), BUTTON_WIDTH));
            choices.spawn(button(&words.delete, Choice::Confirm, warning(), BUTTON_WIDTH));
        });
    });
}

/// The look of a quiet button, one that backs out; and of one that deletes,
/// in the colour of a warning.
fn quiet() -> (Fill, Color) {
    let colours = ui().colours;
    (
        Fill {
            idle: colours.quiet,
            lit: colours.quiet_lit,
        },
        colours.text,
    )
}

fn warning() -> (Fill, Color) {
    let colours = ui().colours;
    (
        Fill {
            idle: colours.warn,
            lit: colours.warn_lit,
        },
        colours.on_gold,
    )
}

/// A button, `width` wide, in `look`: its fill and the colour of its words.
fn button(text: &str, choice: Choice, look: (Fill, Color), width: f32) -> impl Bundle {
    let (fill, ink) = look;
    (
        choice,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(width),
            height: Val::VMin(BUTTON_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(BUTTON_HEIGHT * 0.5)),
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![builder::label(text, CHOICE_TEXT, ink)],
    )
}

/// The pencil, Save and Cancel, and Enter in the box: from the Accounts tab
/// to the dialog that changes your name, and back to the tab, with the name
/// changed or not.
#[allow(clippy::too_many_arguments)]
fn edit_name(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<Settings>,
    mut name_box: ResMut<NameBox>,
    mut profile: ResMut<Profile>,
    mut lobby: ResMut<Lobby>,
    online: Res<Online>,
    dialogs: Query<Entity, With<SettingsDialog>>,
    mut me: Query<(&mut Name, &mut WorldAssetRoot), With<Player>>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let mut choice = settings.pressed.take();
    if choice.is_none()
        && settings.renaming
        && (keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) || name_box.entered())
    {
        choice = Some(Choice::SaveName);
    }
    match choice {
        Some(Choice::Rename) if settings.window.is_some() => {
            take_down(&mut commands, &dialogs);
            settings.window = None;
            settings.renaming = true;
            name_box.open(&profile.name);
            renaming(&mut commands, &profile.name);
        }
        Some(Choice::SaveName) if settings.renaming => {
            let Some(name) = name_box.name() else {
                return;
            };
            if name != profile.name {
                profile.name = name;
                profile.save();
                info!("now {}", profile.name);
                signup::wear(&profile, &mut lobby, &mut me, &assets);
            }
            name_box.close();
            show(&mut commands, &mut settings, &dialogs, &online, &profile);
        }
        Some(Choice::CancelRename) if settings.renaming => {
            name_box.close();
            show(&mut commands, &mut settings, &dialogs, &online, &profile);
        }
        _ => {}
    }
}

/// Greys the dialog's Save while what is typed is no name.
fn light_save(
    settings: Res<Settings>,
    name_box: Res<NameBox>,
    mut saves: Query<(&mut Fill, &mut BackgroundColor, &Children), With<Save>>,
    mut labels: Query<&mut TextColor>,
) {
    if settings.renaming {
        signup::light(name_box.name().is_some(), signup::gold_button(), &mut saves, &mut labels);
    }
}

/// On desktop, `Escape` shuts the window, or takes a smaller dialog back to
/// it.
fn keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<Settings>,
    mut up: ResMut<DialogUp>,
    dialogs: Query<Entity, With<SettingsDialog>>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::Escape) || dialogs.is_empty() {
        return;
    }
    if settings.renaming {
        settings.pressed = Some(Choice::CancelRename);
    } else {
        close(&mut commands, &mut settings, &mut up, &dialogs);
    }
}

/// The login has answered: the account is gone, and this device starts over
/// as someone new, from the sign-up's pages, as it did the first time it was
/// run; or it could not be done, and nothing has changed.
#[allow(clippy::too_many_arguments)]
fn deleted(
    mut deleting: ResMut<Deleting>,
    mut link: ResMut<Link>,
    mut profile: ResMut<Profile>,
    mut signup: ResMut<Signup>,
    mut up: ResMut<DialogUp>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let answer = deleting.0.as_ref().and_then(|answer| {
        match answer.lock().ok()?.try_recv() {
            Ok(answer) => Some(answer),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err("the login went quiet".into())),
        }
    });
    let Some(answer) = answer else {
        return;
    };
    deleting.0 = None;
    match answer {
        Ok(()) => {
            info!("the account was deleted: starting over");
            link.start_over();
            profile.forget();
            signup::open(&mut commands, &assets, &mut signup, &mut up);
        }
        Err(error) => {
            warn!("could not delete the account: {error}");
            hud::announce(
                &mut commands,
                None,
                &ui().words.settings.could_not_delete,
                hud::CAPTION,
                3.0,
            );
        }
    }
}
