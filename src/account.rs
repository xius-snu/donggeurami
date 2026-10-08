//! Your account, while you are online: who you are to everyone else, and the
//! way to delete it.
//!
//! A small round button beside your balance, shown only online, opens it: the
//! name everyone sees you by, and Delete account, which asks once more before
//! it goes ahead. Apple asks for a way to delete an account inside the app,
//! even one nobody signed up for (`MULTIPLAYER.md`, "Before a public online
//! release"), and every account here is one of those: made by the login the
//! first time a device asks to play, and found again by the device's secret.
//!
//! Deleting asks the login to forget the account and everything kept about it
//! (`login::delete`). Then this device forgets its secret, and connects again
//! as someone new, with a new name and the coins everyone starts with.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, TryRecvError};

use bevy::prelude::*;

use crate::builder::{
    self, BODY, BUTTON_HEIGHT, BUTTON_WIDTH, Fill, QUIET, QUIET_LIT, SMALL, SOFT,
    TITLE, WARN,
};
use crate::hud::{self, BarLeft, DOOR, DialogUp, GOLD, PANEL, TakesPress};
use crate::net::{Link, Online};

/// How big the button is, as a share of the short side of the screen: as tall
/// as the balance beside it.
const BUTTON: f32 = 8.0;
/// The button lit up under the pointer, and Delete.
const PANEL_LIT: Color = Color::srgba(0.04, 0.10, 0.18, 0.6);
const WARN_LIT: Color = Color::srgb(1.0, 0.74, 0.69);
/// How big a button's words are: a little smaller than House Builder's, for
/// "Delete account" to fit one of its buttons on one line.
const CHOICE_TEXT: f32 = 4.0;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Deleting>()
        .add_observer(choose)
        .add_systems(Startup, spawn_button.after(hud::show_top_bar))
        .add_systems(Update, (show_button, deleted));
}

/// The button beside your balance.
#[derive(Component)]
struct AccountButton;

/// The root of one of the account's dialogs.
#[derive(Component)]
struct AccountDialog;

/// What a press on one of the account's buttons does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Choice {
    /// Show your account.
    Open,
    /// Put it away again.
    Close,
    /// Delete it: asked once more first.
    Delete,
    /// Delete it, really.
    Confirm,
}

/// The login's answer, while it is being asked to delete your account.
#[derive(Resource, Default)]
struct Deleting(Option<Mutex<Receiver<Result<(), String>>>>);

fn spawn_button(bars: Query<Entity, With<BarLeft>>, mut commands: Commands) {
    let Ok(bar) = bars.single() else {
        return;
    };
    commands.spawn((
        ChildOf(bar),
        AccountButton,
        Choice::Open,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(BUTTON),
            height: Val::VMin(BUTTON),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(PANEL),
        Fill {
            idle: PANEL,
            lit: PANEL_LIT,
        },
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        Visibility::Hidden,
        children![(hud::picture(), builder::person())],
    ));
}

/// The button is there only while you are online: offline there is no
/// account to see.
fn show_button(online: Res<Online>, mut buttons: Query<&mut Visibility, With<AccountButton>>) {
    for mut shown in &mut buttons {
        shown.set_if_neq(if online.is() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

/// Opens the account, and goes on from there as its buttons are pressed.
#[allow(clippy::too_many_arguments)]
fn choose(
    click: On<Pointer<Click>>,
    choices: Query<&Choice>,
    online: Res<Online>,
    link: Res<Link>,
    mut up: ResMut<DialogUp>,
    dialogs: Query<Entity, With<AccountDialog>>,
    mut deleting: ResMut<Deleting>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    let Ok(&choice) = choices.get(click.entity) else {
        return;
    };
    let take_down = |commands: &mut Commands| {
        for dialog in &dialogs {
            commands.entity(dialog).despawn();
        }
    };
    match choice {
        Choice::Open if !up.0 => {
            let Some(me) = online.me() else {
                return;
            };
            yours(&mut commands, &me.name);
            up.set_if_neq(DialogUp(true));
        }
        Choice::Open => {}
        Choice::Close => {
            take_down(&mut commands);
            up.set_if_neq(DialogUp(false));
        }
        Choice::Delete => {
            take_down(&mut commands);
            sure(&mut commands);
        }
        Choice::Confirm => {
            take_down(&mut commands);
            up.set_if_neq(DialogUp(false));
            match link.delete_account() {
                Some(answer) => {
                    deleting.0 = Some(Mutex::new(answer));
                    hud::announce(&mut commands, None, "Deleting your account...", hud::CAPTION, 2.0);
                }
                None => {
                    hud::announce(&mut commands, None, "Not online: try again", hud::CAPTION, 3.0);
                }
            }
        }
    }
}

/// Your account: the name everyone sees you by, and the choice of deleting it.
fn yours(commands: &mut Commands, name: &str) {
    builder::dimmed(commands, (AccountDialog, Choice::Close)).with_children(|panel| {
        panel.spawn(hud::words("Your account", TITLE, GOLD));
        panel.spawn(builder::label(name, BODY, Color::WHITE));
        panel.spawn(builder::label(
            "A guest account, kept on this device.",
            SMALL,
            SOFT,
        ));
        panel.spawn(builder::choices()).with_children(|choices| {
            choices.spawn(button("Close", Choice::Close, false));
            choices.spawn(button("Delete account", Choice::Delete, true));
        });
    });
}

/// Asked once more, before an account goes for good.
fn sure(commands: &mut Commands) {
    builder::dimmed(commands, (AccountDialog, Choice::Close)).with_children(|panel| {
        panel.spawn(hud::words("Delete your account?", TITLE, GOLD));
        panel.spawn(builder::label(
            "Your name and coins go for good.\nThis cannot be undone.",
            BODY,
            Color::WHITE,
        ));
        panel.spawn(builder::choices()).with_children(|choices| {
            choices.spawn(button("Cancel", Choice::Close, false));
            choices.spawn(button("Delete", Choice::Confirm, true));
        });
    });
}

/// One of a dialog's two buttons: the quiet one that backs out, or the one
/// that deletes, in the colour of a warning.
fn button(text: &str, choice: Choice, warn: bool) -> impl Bundle {
    let (fill, ink) = if warn {
        (
            Fill {
                idle: WARN,
                lit: WARN_LIT,
            },
            DOOR,
        )
    } else {
        (
            Fill {
                idle: QUIET,
                lit: QUIET_LIT,
            },
            Color::WHITE,
        )
    };
    (
        choice,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(BUTTON_WIDTH),
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

/// The login has answered: the account is gone, and this device starts over
/// as someone new; or it could not be done, and nothing has changed.
fn deleted(mut deleting: ResMut<Deleting>, mut link: ResMut<Link>, mut commands: Commands) {
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
            hud::announce(&mut commands, None, "Your account is deleted", hud::CAPTION, 3.0);
        }
        Err(error) => {
            warn!("could not delete the account: {error}");
            hud::announce(
                &mut commands,
                None,
                "Could not delete it: try again",
                hud::CAPTION,
                3.0,
            );
        }
    }
}
