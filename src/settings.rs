//! Settings: the gear beside your name, and the window it opens, with a tab for
//! each kind (Hajun, 2026-10-31: "a setting icon in top right corner ... a
//! semi transparent greyish rectangular popup which will have all sorts of
//! different useful settings"): Graphics and Controls, with nothing in them
//! yet, and Accounts, with your name and a pencil at the end of its row to
//! change it, Sign in with Google and Sign in with Apple, blocked out until
//! there is a sign-in to be had, and Delete account. From 2026-10-08 until
//! then your account was a round button beside your balance, shown online
//! only; the gear was top right for an hour, until Hajun's sketch of the
//! screen put it beside your name (`hud`).
//!
//! **It is drawn as the friends' panel is** (`window`, since 2026-11-14;
//! Hajun: "I think I like the ui design/color choice/ close button layout/
//! etc. of the friends tab. I want to use this same design style throughout
//! the project"): the same dark see-through window of the same size, with
//! nothing dimmed round it, its tabs along the top with a white line under
//! the one shown, the bare white cross in its corner, and what is in it
//! from the left, your account on a row as a friend is on theirs. Until
//! then it was a grey glass over the dimmed screen, its tabs gold and grey
//! pills, its cross a disc hung off its corner, and what was in it down its
//! middle on big round-ended buttons.
//!
//! The pencil brings a box up over the window, "Edit name", with your name
//! in it to type into (`signup::NameBox`) and Save, which makes it your name
//! everywhere the sign-up's did (`signup::wear`): in the town's room, over
//! your head, and, online, with the server.
//!
//! Deleting asks once more, in a box over the window, then asks the login to
//! forget the account and everything kept about it (`login::delete`): Apple
//! asks for a way to delete an account inside the app, even one nobody
//! signed up for (`MULTIPLAYER.md`, "Before a public online release"), and
//! every account here is one of those, made by the login the first time a
//! device asks to play. Then this device forgets its secret and who it was
//! (`signup::Profile`), and starts over from the sign-up's pages as someone
//! new.
//!
//! While the window is up it is a dialog like House Builder's others
//! ([`DialogUp`]): you stand still, and presses go to it alone. The cross in
//! its corner, a tap anywhere off it, or on desktop `Escape`, shuts it; from
//! a box over it, Cancel, a tap on the dark round the box or `Escape` goes
//! back to the window.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, TryRecvError};

use bevy::prelude::*;

use crate::builder::{self, DIALOG_LAYER, Fill};
use crate::hud::{self, BarLeft, DialogUp, TakesPress};
use crate::lobby::Lobby;
use crate::net::{Link, Online};
use crate::signup::{self, NameBox, Profile, Save, Signup};
use crate::ui::{UiChanged, ui};
use crate::window::{self, Look};
use crate::Player;

/// Sizes, as shares of the short side of the screen, where they are not the
/// windows' own (`window`): the Accounts tab's buttons, wide enough for
/// "Sign in with Google" on one line; and the boxes that come up over the
/// window, Edit name's, for the longest name there is, and the question
/// asked before an account is deleted, for its two lines. The colours and
/// the words are `assets/ui.ron`'s (`ui`).
const CHOICE_WIDTH: f32 = 38.0;
const RENAME_WIDTH: f32 = 62.0;
const SURE_WIDTH: f32 = 62.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Settings>()
        .init_resource::<Deleting>()
        .add_observer(choose)
        .add_systems(Update, (spawn_button, redraw, keys, edit_name, light_save, deleted).chain());
}

/// The window, while it is up, the tab it shows, and what is up over it.
#[derive(Resource, Default)]
struct Settings {
    window: Option<Entity>,
    tab: Tab,
    /// The pencil, drawn once.
    pencil: Handle<Image>,
    /// The box up over the window, if one is.
    over: Option<Over>,
    /// A press on one of the buttons to do with your name, to be acted on
    /// ([`edit_name`]).
    pressed: Option<Choice>,
}

/// A box up over the window.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Over {
    /// Edit name: your name in a box to type into.
    Rename,
    /// Asked once more, before an account goes for good.
    Delete,
}

/// The gear beside your name.
#[derive(Component)]
struct SettingsButton;

/// The root of the window, over the whole screen.
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
    /// Not after all: back to the window.
    Back,
    /// Delete it, really.
    Confirm,
}

/// The login's answer, while it is being asked to delete your account.
#[derive(Resource, Default)]
struct Deleting(Option<Mutex<Receiver<Result<(), String>>>>);

/// Everything the window is drawn from ([`window`]).
struct View<'a> {
    tab: Tab,
    over: Option<Over>,
    online: &'a Online,
    profile: &'a Profile,
    pencil: Handle<Image>,
    /// What is typed so far, while your name is being changed.
    typed: &'a str,
    /// Whether what is typed is a name that can be saved.
    saves: bool,
}

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
/// desktop (`ui::reread`), on the tab it is on and with whatever is up over
/// it, what is typed there kept.
#[allow(clippy::too_many_arguments)]
fn redraw(
    changed: Res<UiChanged>,
    online: Res<Online>,
    profile: Res<Profile>,
    name_box: Res<NameBox>,
    assets: Res<AssetServer>,
    mut settings: ResMut<Settings>,
    dialogs: Query<Entity, With<SettingsDialog>>,
    mut commands: Commands,
) {
    if !changed.is_changed() || changed.is_added() || settings.window.is_none() {
        return;
    }
    show(&mut commands, &assets, &mut settings, &dialogs, &online, &profile, &name_box);
}

/// Opens the window, and goes on from there as its buttons are pressed.
#[allow(clippy::too_many_arguments)]
fn choose(
    click: On<Pointer<Click>>,
    choices: Query<&Choice>,
    online: Res<Online>,
    profile: Res<Profile>,
    link: Res<Link>,
    assets: Res<AssetServer>,
    mut settings: ResMut<Settings>,
    mut name_box: ResMut<NameBox>,
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
            settings.over = None;
            show(&mut commands, &assets, &mut settings, &dialogs, &online, &profile, &name_box);
            up.0 = true;
        }
        Choice::Open => {}
        Choice::Close => {
            close(&mut commands, &mut settings, &mut name_box, &mut up, &dialogs);
        }
        Choice::Show(tab) => {
            if settings.tab != tab {
                settings.tab = tab;
                show(&mut commands, &assets, &mut settings, &dialogs, &online, &profile, &name_box);
            }
        }
        Choice::Rename | Choice::SaveName | Choice::CancelRename => {
            settings.pressed = Some(choice);
        }
        Choice::Delete => {
            settings.over = Some(Over::Delete);
            show(&mut commands, &assets, &mut settings, &dialogs, &online, &profile, &name_box);
        }
        Choice::Back => {
            settings.over = None;
            show(&mut commands, &assets, &mut settings, &dialogs, &online, &profile, &name_box);
        }
        Choice::Confirm => {
            close(&mut commands, &mut settings, &mut name_box, &mut up, &dialogs);
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

/// Puts the window up as things stand, on its tab and with whatever is up
/// over it, in place of the one that was up.
fn show(
    commands: &mut Commands,
    assets: &AssetServer,
    settings: &mut Settings,
    dialogs: &Query<Entity, With<SettingsDialog>>,
    online: &Online,
    profile: &Profile,
    name_box: &NameBox,
) {
    take_down(commands, dialogs);
    let renaming = settings.over == Some(Over::Rename);
    let view = View {
        tab: settings.tab,
        over: settings.over,
        online,
        profile,
        pencil: settings.pencil.clone(),
        typed: if renaming && name_box.up() { name_box.text() } else { "" },
        saves: renaming && name_box.name().is_some(),
    };
    settings.window = Some(window(commands, assets, &view));
}

/// Takes the window down, with whatever was up over it.
fn close(
    commands: &mut Commands,
    settings: &mut Settings,
    name_box: &mut NameBox,
    up: &mut DialogUp,
    dialogs: &Query<Entity, With<SettingsDialog>>,
) {
    take_down(commands, dialogs);
    // What was being typed goes with the box it was typed in.
    if settings.over == Some(Over::Rename) {
        name_box.close();
    }
    settings.window = None;
    settings.over = None;
    if up.0 {
        up.0 = false;
    }
}

fn take_down(commands: &mut Commands, dialogs: &Query<Entity, With<SettingsDialog>>) {
    for dialog in dialogs {
        commands.entity(dialog).despawn();
    }
}

/// The window as things stand: its tabs along its top, the cross in its
/// corner, what the tab shown holds, and whatever is up over it. Returns its
/// root, which is over the whole screen, shows nothing, and shuts the
/// window at a tap.
fn window(commands: &mut Commands, assets: &AssetServer, view: &View) -> Entity {
    let root = commands
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
            BackgroundColor(Color::NONE),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let panel = commands
        .spawn((
            ChildOf(root),
            window::pane(Node {
                width: Val::VMin(window::WIDTH),
                height: Val::VMin(window::HEIGHT),
                padding: UiRect::all(Val::VMin(window::PAD)),
                ..default()
            }),
        ))
        .id();
    let tabs = commands.spawn((ChildOf(panel), window::header())).id();
    for each in Tab::ALL {
        let shown = each == view.tab;
        commands.spawn((
            ChildOf(tabs),
            Choice::Show(each),
            window::tab(shown),
            children![window::words(each.title(), window::TAB_TEXT, window::tab_ink(shown))],
        ));
    }
    commands.spawn((ChildOf(panel), window::cross(assets.load("icons/close.png"), Choice::Close)));
    // What the tab holds, from the left and the top, as a list of friends
    // is.
    let body = commands
        .spawn((
            ChildOf(panel),
            Node {
                flex_grow: 1.0,
                min_height: Val::ZERO,
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::VMin(1.2)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    if view.tab == Tab::Accounts {
        accounts(commands, assets, body, view);
    }
    match view.over {
        Some(Over::Rename) => renaming(commands, panel, view),
        Some(Over::Delete) => sure(commands, panel),
        None => {}
    }
    root
}

/// The Accounts tab: who you are, on a row as a friend is on theirs, with
/// the pencil at its end to change your name; the sign-ins to come, blocked
/// out; and, at the foot of the window, Delete account.
fn accounts(commands: &mut Commands, assets: &AssetServer, body: Entity, view: &View) {
    let ui = ui();
    let (colours, words) = (ui.colours.window, &ui.words.settings);
    // The name you chose, which the server takes on a moment after it is
    // changed; the server's only before you have chosen one.
    let name = if view.profile.set() {
        view.profile.name.clone()
    } else {
        view.online.me().map_or(words.not_signed_up.clone(), |me| me.name.clone())
    };
    let here = view.online.is();
    let standing = if here { words.guest.clone() } else { words.not_online.clone() };
    commands.spawn((
        ChildOf(body),
        Node {
            height: Val::VMin(window::ROW),
            align_items: AlignItems::Center,
            column_gap: Val::VMin(2.2),
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
        children![
            window::avatar(assets.load("icons/person.png"), window::AVATAR, here),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    min_width: Val::ZERO,
                    row_gap: Val::VMin(0.3),
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    window::words(name, window::NAME, colours.name),
                    window::words(standing, window::STATUS, colours.status),
                ],
            ),
            (Choice::Rename, window::round(view.pencil.clone(), window::ROW_BUTTON, true)),
        ],
    ));
    commands.spawn((
        ChildOf(body),
        Node {
            column_gap: Val::VMin(2.0),
            margin: UiRect::top(Val::VMin(1.6)),
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
        children![
            window::choice_wide(words.sign_in_google.clone(), Look::Blocked, CHOICE_WIDTH),
            window::choice_wide(words.sign_in_apple.clone(), Look::Blocked, CHOICE_WIDTH),
        ],
    ));
    // Whatever room is left, so that Delete account stands at the foot of
    // the window, away from the rest.
    commands.spawn((
        ChildOf(body),
        Node {
            flex_grow: 1.0,
            ..default()
        },
        Pickable::IGNORE,
    ));
    commands.spawn((
        ChildOf(body),
        Node {
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
        children![(Choice::Delete, window::choice_wide(words.delete_account.clone(), Look::No, CHOICE_WIDTH))],
    ));
}

/// The box that changes your name, up over the window: the name in a box
/// to type into, a line that says so when the name typed cannot be had,
/// Cancel, and Save. On a phone it slides up out from under the keyboard
/// (`signup::LiftsForKeyboard`), as the friends' boxes do.
fn renaming(commands: &mut Commands, panel: Entity, view: &View) {
    let ui = ui();
    let (colours, words) = (ui.colours.window, &ui.words.settings);
    let within = window::popup(commands, panel, RENAME_WIDTH, true, Choice::CancelRename);
    commands.spawn((ChildOf(within), window::words(words.edit_name.clone(), window::TITLE, colours.title)));
    commands.spawn((
        ChildOf(within),
        signup::field(
            view.typed,
            Node {
                height: Val::VMin(window::FIELD_HEIGHT),
                padding: UiRect::horizontal(Val::VMin(2.4)),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::VMin(0.8)),
                overflow: Overflow::clip_x(),
                ..default()
            },
            window::FIELD_TEXT,
            colours.field,
            colours.name,
        ),
    ));
    // "Pick another name.", while the one typed has a swear word in it;
    // nothing otherwise, and as tall either way. From the left, under the
    // box it is about.
    commands
        .spawn((ChildOf(within), signup::name_notice("", window::NOTICE_TEXT, colours.status)))
        .insert(TextLayout::justify(Justify::Left));
    commands.spawn((
        ChildOf(within),
        window::choices(),
        children![
            (Choice::CancelRename, window::choice(words.cancel.clone(), Look::No)),
            (Save, Choice::SaveName, window::choice(words.save.clone(), save_look(view.saves))),
        ],
    ));
}

/// Asked once more, in a box over the window, before an account goes for
/// good. Delete is the dark red of the window's Delete account, and
/// Cancel a quiet grey: nothing here is the green of a yes.
fn sure(commands: &mut Commands, panel: Entity) {
    let ui = ui();
    let (colours, words) = (ui.colours.window, &ui.words.settings);
    let within = window::popup(commands, panel, SURE_WIDTH, false, Choice::Back);
    commands.spawn((ChildOf(within), builder::label(words.delete_sure.clone(), window::TITLE, colours.title)));
    commands.spawn((
        ChildOf(within),
        builder::label(words.delete_warning.clone(), window::CHOICE_TEXT, colours.status),
    ));
    commands.spawn((
        ChildOf(within),
        window::choices(),
        children![
            (Choice::Back, window::choice(words.cancel.clone(), Look::Quiet)),
            (Choice::Confirm, window::choice(words.delete.clone(), Look::No)),
        ],
    ));
}

/// How Save looks: green while what is typed can be saved, and greyed
/// while it is no name.
fn save_look(able: bool) -> Look {
    if able { Look::Yes } else { Look::Greyed }
}

/// The pencil, Save and Cancel, and Enter in the box: from the Accounts tab
/// to the box that changes your name, and back to the tab, with the name
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
    let renaming = settings.over == Some(Over::Rename);
    let mut choice = settings.pressed.take();
    if choice.is_none()
        && renaming
        && (keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) || name_box.entered())
    {
        choice = Some(Choice::SaveName);
    }
    match choice {
        Some(Choice::Rename) if settings.window.is_some() && settings.over.is_none() => {
            settings.over = Some(Over::Rename);
            name_box.open(&profile.name);
        }
        Some(Choice::SaveName) if renaming => {
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
            settings.over = None;
        }
        Some(Choice::CancelRename) if renaming => {
            name_box.close();
            settings.over = None;
        }
        _ => return,
    }
    show(&mut commands, &assets, &mut settings, &dialogs, &online, &profile, &name_box);
}

/// Greys Save while what is typed is no name.
fn light_save(
    settings: Res<Settings>,
    name_box: Res<NameBox>,
    mut saves: Query<(&mut Fill, &mut BackgroundColor, &Children), With<Save>>,
    mut labels: Query<&mut TextColor>,
) {
    if settings.over != Some(Over::Rename) {
        return;
    }
    let (fill, ink) = save_look(name_box.name().is_some()).colours();
    for (mut was, mut colour, children) in &mut saves {
        if was.idle == fill.idle {
            continue;
        }
        *was = fill;
        colour.0 = fill.idle;
        let mut words = labels.iter_many_mut(children);
        while let Some(mut label) = words.fetch_next() {
            label.0 = ink;
        }
    }
}

/// On desktop, `Escape` shuts the window, or takes away what is up over
/// it.
fn keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    online: Res<Online>,
    profile: Res<Profile>,
    assets: Res<AssetServer>,
    mut settings: ResMut<Settings>,
    mut name_box: ResMut<NameBox>,
    mut up: ResMut<DialogUp>,
    dialogs: Query<Entity, With<SettingsDialog>>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::Escape) || dialogs.is_empty() {
        return;
    }
    match settings.over {
        Some(Over::Rename) => settings.pressed = Some(Choice::CancelRename),
        Some(Over::Delete) => {
            settings.over = None;
            show(&mut commands, &assets, &mut settings, &dialogs, &online, &profile, &name_box);
        }
        None => close(&mut commands, &mut settings, &mut name_box, &mut up, &dialogs),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_accounts_tab_and_the_boxes_over_the_window_fit_it() {
        // Two sign-ins side by side, within the window's margins; and
        // each box narrower than the window it comes up over.
        let within = window::WIDTH - 2.0 * window::PAD;
        assert!(2.0 * CHOICE_WIDTH + 2.0 < within);
        assert!(RENAME_WIDTH < within && SURE_WIDTH < within);
        // A row, the sign-ins and Delete account, under the tabs, within
        // its height.
        let used = window::TAB_HEIGHT + 1.2 + window::ROW + 1.6 + 2.0 * window::CHOICE_HEIGHT;
        assert!(used < window::HEIGHT - 2.0 * window::PAD);
        // Save is green only while there is a name to save.
        assert_eq!((save_look(true), save_look(false)), (Look::Yes, Look::Greyed));
    }
}
