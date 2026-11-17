//! A game of Bridge Wars the computer plays alone, for looking on at, on
//! desktop, offline (since 2026-11-16; Hajun: "run a test game on this
//! computer. i want to not be a real player, i want to just spectate a
//! game of 8 ai bots fighting against each other, and make it so it is
//! time 14:00 so that 1min later the zombie spawn i wana see from the view
//! of an ai. like simulation"). `RT_ARENA_WATCH=1` turns it on; with it
//! go `RT_SERVER=off`, and `RT_ARENA_FROM=840` for a game begun at 14:00
//! and `RT_ARENA_PURSE=<count>` for what each of its players begins with.
//!
//! **Your seat is one of the computer's too.** The game is the game played
//! offline, with its eight seats and yours among them (`play`); once the
//! fight begins your own body is the brain's to place, as the seven
//! others are ([`possess`], `drive_bots`): it has neither your stick nor
//! anything of its own to walk by, takes what it takes and buys what it
//! buys as they do, and what the game keeps of yours alone it keeps of
//! nobody ([`Game::mine`]). So the view is one of the computer's players',
//! with its health, what it holds and what is on its buttons where yours
//! would be ([`purse`]).
//!
//! **It begins by itself** ([`enter`]): into a game as the town is there,
//! the wait for others skipped, the countdown as ever.
//!
//! **Tab is the next player's view** ([`look`]): the camera over them as
//! it would be over you (`crate::Lookout`), and whose view it is written
//! along the bottom of the screen ([`Label`], `words.arena.watching`).
//! Whoever is watched and is out for good, the view goes to the next still
//! in it by itself. What is top left stays your own seat's player's. Nobody is asked to play again; the game ends as any
//! does, and you are in the town again, yourself.

use std::sync::OnceLock;

use super::*;
use crate::Collider;
use crate::lobby::WalkFromMotion;

/// How long after the town is there the game is asked for, and how often
/// the wait for others is skipped until it is over, in seconds.
const ENTER_AFTER: f32 = 1.0;
const SKIP_EVERY: f32 = 0.5;
/// Whose view it is, written along the bottom of the screen, between the
/// stick and the buttons.
#[derive(Component)]
struct Label;

/// Whether the computer is to play alone, to be looked on at: on desktop,
/// with `RT_ARENA_WATCH` set.
pub(super) fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        cfg!(not(any(target_os = "android", target_os = "ios")))
            && std::env::var("RT_ARENA_WATCH").is_ok_and(|said| !matches!(said.trim(), "" | "0" | "off"))
    })
}

/// The looking on: how long the app has been up, whether a game has been
/// asked for and when the wait was last skipped; whose view it is, by
/// seat, none being your own seat's; whose name was said last; and what
/// was taken off your body for the brain to have it, to be put back.
#[derive(Resource, Default)]
struct Watch {
    since: f32,
    asked: bool,
    skipped: f32,
    seat: Option<usize>,
    said: Option<usize>,
    collider: Option<Collider>,
}

pub(super) fn plugin(app: &mut App) {
    if !on() {
        return;
    }
    app.init_resource::<Watch>().add_systems(
        Update,
        (
            enter.before(ArenaSystems),
            (possess, purse).after(drive_bots).before(crate::move_bodies),
            look.after(crate::move_bodies).before(crate::follow_camera),
        ),
    );
}

/// Asks for a game once the town is there and its opening over, and skips
/// the wait for anyone else: the computer takes the seats left at once.
/// The countdown is left as it is, and nothing is skipped after it: a
/// skip in the fight ends the game.
fn enter(
    time: Res<Time>,
    intro: Res<crate::intro::Intro>,
    online: Res<Online>,
    game: Option<Res<Game>>,
    mut watch: ResMut<Watch>,
    mut asks: MessageWriter<Ask>,
) {
    if intro.welcomes() || online.is() {
        return;
    }
    watch.since += time.delta_secs();
    if !watch.asked {
        if watch.since >= ENTER_AFTER {
            watch.asked = true;
            asks.write(Ask::Enter);
            info!("Bridge Wars, watched: asking for a game");
        }
        return;
    }
    let waiting = game.is_some_and(|game| matches!(game.stage, Stage::Waiting { .. }));
    if waiting && watch.since - watch.skipped >= SKIP_EVERY {
        watch.skipped = watch.since;
        asks.write(Ask::Skip);
    }
}

/// Gives your body to the brain as the fight begins, and back as the game
/// goes: placed, it is what the computer's others are (`lobby::spawn_bot_at`),
/// with nothing to walk by and in nobody's way, its walk read off how it
/// is moved.
fn possess(
    game: Option<Res<Game>>,
    me: Query<(Entity, &Transform, Option<&Collider>, Has<Brained>), With<Player>>,
    mut watch: ResMut<Watch>,
    mut commands: Commands,
) {
    let Ok((you, at, collider, placed)) = me.single() else {
        return;
    };
    let fought = game.is_some_and(|game| game.watched && matches!(game.stage, Stage::Playing { .. } | Stage::Over { .. }));
    match (fought, placed) {
        (true, false) => {
            watch.collider = collider.copied();
            commands
                .entity(you)
                .insert((Brained, WalkFromMotion::standing(at.translation)))
                .remove::<(Intent, Collider)>();
            info!("Bridge Wars, watched: your seat is the computer's to play");
        }
        (false, true) => {
            let mut body = commands.entity(you);
            body.remove::<(Brained, WalkFromMotion)>().insert(Intent::default());
            if let Some(collider) = watch.collider.take() {
                body.insert(collider);
            }
            info!("Bridge Wars, watched: your body is your own again");
        }
        _ => {}
    }
}

/// Shows what your seat's player holds where what you hold is shown, top
/// left: it is the brain's, not yours (`Brains::purses`).
fn purse(game: Option<Res<Game>>, mut stock: ResMut<Resources>) {
    let Some(game) = game.filter(|game| game.watched && game.playing()) else {
        return;
    };
    let Some(held) = game.brains.as_deref().and_then(|brains| brains.purses.get(game.me)) else {
        return;
    };
    if stock.held != *held {
        stock.held = *held;
    }
}

/// Whose view it is: your seat's player's to begin with, the next one's
/// still in the game at Tab, and by itself once whoever is watched is out
/// for good. The camera is put over them (`crate::Lookout`), and whose
/// view it is written along the bottom of the screen.
fn look(
    keys: Res<ButtonInput<KeyCode>>,
    game: Option<Res<Game>>,
    bodies: Query<&Transform>,
    labels: Query<Entity, With<Label>>,
    mut watch: ResMut<Watch>,
    mut lookout: ResMut<crate::Lookout>,
    mut commands: Commands,
) {
    let Some(game) = game.filter(|game| game.watched && game.playing()) else {
        lookout.set_if_neq(crate::Lookout(None));
        if watch.seat.is_some() || watch.said.is_some() {
            (watch.seat, watch.said) = (None, None);
        }
        return;
    };
    let me = game.me;
    let in_it = |seat: usize| game.out.get(seat).is_some_and(|out| !out);
    let was = watch.seat.unwrap_or(me);
    let seat = if keys.just_pressed(KeyCode::Tab) || !in_it(was) {
        (1..=ARENA_SEATS).map(|on| (was + on) % ARENA_SEATS).find(|&next| in_it(next)).unwrap_or(was)
    } else {
        was
    };
    if watch.seat != Some(seat) {
        watch.seat = Some(seat);
    }
    let at = game.bodies.get(seat).copied().flatten().and_then(|body| bodies.get(body).ok());
    let over = at.filter(|_| seat != me).map(|at| at.translation);
    lookout.set_if_neq(crate::Lookout(over));
    // Whose it is, written again as it goes to someone else; gone with
    // the game, as everything the game put up is (`InArena`).
    if watch.said != Some(seat) {
        watch.said = Some(seat);
        for label in &labels {
            commands.entity(label).despawn();
        }
        let file = ui();
        let name = game.seats.get(seat).map_or("", |member| member.name.as_str());
        commands.spawn((
            InArena,
            Label,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::VMin(hud::EDGE),
                left: Val::ZERO,
                right: Val::ZERO,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
            children![hud::words(fill(&file.words.arena.watching, &[("name", name)]), CAPTION, file.colours.text)],
        ));
        info!("Bridge Wars, watched: the view is seat {seat}'s, {name}");
    }
}
