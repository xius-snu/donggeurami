//! The window each round of Blend In begins with (Hajun: "Both hider and
//! seeker can Choose (must choose within the timer of maybe 30s, autostart
//! when both have chosen): Guy/girl + skin of character, Name"), as every
//! window is, the friends' panel's (`window`, in `colours.window`): its
//! title the round ("Round 2 of 3") and the time left; under it whether you
//! hide this round or seek; your look between two arrows that step through
//! the six looks (the sign-up's pictures, `signup/{look}.png`, and its
//! arrowheads, `hud::side_arrow`); your name, in a box like the sign-up's
//! (`signup::field`, the one box everything is typed into, `NameBox`), and
//! beside it a dice that throws one of the town's own names for the look's
//! side, as the crowd goes by; the players, with a tick by each who has
//! chosen and a word by the one who hides; and Ready, green, grey while
//! the name being typed is no name.
//!
//! It opens on what the game gave you (`Tell::Wear`: the hider a guise of
//! the crowd's, a seeker what they wore last, in the first round their own
//! account), and every choice goes to the server as it is made, which
//! answers with what you wear as it stands: a name that is no name, or a
//! seeker's, is turned down ("Pick another name.") and what you had kept.
//! Nothing chosen in time keeps what it opened on. It is never your account
//! that changes: what you are in the game is the game's alone (`Profile`
//! is never written here).
//!
//! **Mine**: it stands to the right of the screen, clear of the chat down
//! the left, and is no dialog: you stand still while it is up (`hold`),
//! but the chat may be used (the design: "While it is up the chat may be
//! used"), so the box is taken for the name only while the name is being
//! typed (tapped), and nothing else is held. Its cross is Ready with what
//! is chosen, and puts it away for the round. An arrow that takes the
//! look over to the other side, a boy's to a girl's or back, throws a name
//! of that side too if the name was one of the other side's town names, so
//! that a hider is never a girl by a boy's name, which none of the crowd
//! is.

use bevy::prelude::*;
use roundtown_net::blend_in::{self as rules, BlendStage};
use roundtown_net::{Ask as NetAsk, BOY_LOOKS, GIRL_LOOKS, ai_names, profile};

use super::{Blend, ask, wrapped};
use crate::builder::Fill;
use crate::clock::TownClock;
use crate::hud::{self, WantsPointer};
use crate::signup::{self, LiftsForKeyboard, NameBox};
use crate::table::{self, InTable, TableView};
use crate::tags::HidesNames;
use crate::ui::{UiChanged, fill, ui};
use crate::window::{self, Look};

/// Sizes, as shares of the short side of the screen: the window; the card
/// with your look on it and the picture on it; an arrow beside it, its
/// head and the room round it for a finger; the box your name is in; the
/// tick by a name; the players' words; and Ready.
const WIDTH: f32 = 100.0;
const CARD: f32 = 21.0;
const PICTURE: f32 = 17.0;
const ARROW: f32 = 4.4;
const ARROW_TOUCH: f32 = 6.4;
const FIELD_WIDTH: f32 = 40.0;
const TICK: f32 = 2.8;
const PLAYER_TEXT: f32 = 3.0;
const READY_WIDTH: f32 = 38.0;
/// How far in from the right edge of the screen it stands, and how far
/// down from the top: clear of the chat's small keyboard and the speech
/// bubble round the jump button, which stay to be pressed, and of the
/// points top right; over the sword and the jump button's corner, which do
/// nothing while you choose.
const RIGHT: f32 = 29.0;
const TOP: f32 = 24.0;
/// Where it is among what is on the screen: under the screen's own, with
/// the names over heads (`tags`), which none show through it.
const LAYER: i32 = -1;

/// Every look there is to be dressed in, as the arrows step through them:
/// the boys' three, then the girls' three.
const LOOKS: [&str; 6] = [BOY_LOOKS[0], BOY_LOOKS[1], BOY_LOOKS[2], GIRL_LOOKS[0], GIRL_LOOKS[1], GIRL_LOOKS[2]];

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Chooser>()
        .add_observer(press)
        .add_systems(Startup, make_pictures)
        .add_systems(
            Update,
            (put_up, act, draw, count_down, light_ready)
                .chain()
                .after(super::follow)
                .after(signup::Typed)
                .before(super::hold),
        );
}

/// The window, while it is up, and what was last done on it. (Which round's
/// was put away by its cross is the table's, `Blend::shut`, and goes with
/// it.)
#[derive(Resource, Default)]
struct Chooser {
    /// Up, for this round.
    up: Option<u8>,
    /// The name is being typed: the box is this window's.
    typing: bool,
    /// A press on one of its buttons, to be acted on.
    pressed: Option<ChooseTap>,
    /// What it was last drawn showing, to draw it again only as that
    /// changes.
    drawn: Option<Drawn>,
    /// The cursor was let go for it, on desktop ([`WantsPointer`]).
    pointer: bool,
}

/// The pictures it is drawn with, made once: the arrowhead, the dice and
/// the tick.
#[derive(Resource)]
struct Pictures {
    arrow: Handle<Image>,
    dice: Handle<Image>,
    tick: Handle<Image>,
}

fn make_pictures(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    commands.insert_resource(Pictures {
        arrow: images.add(hud::side_arrow()),
        dice: images.add(super::screen::dice()),
        tick: images.add(super::screen::tick()),
    });
}

/// What the window shows, as it was drawn.
#[derive(Clone, PartialEq, Debug)]
struct Drawn {
    round: u8,
    rounds: u8,
    hiding: bool,
    name: String,
    look: String,
    typing: bool,
    refused: bool,
    ready: bool,
    /// Each player: their name, whether they have chosen, whether they hide.
    players: Vec<(String, bool, bool)>,
}

/// The window's root, over the whole screen, taking no press of its own.
#[derive(Component)]
struct ChooseWindow;

/// The time left on it.
#[derive(Component)]
struct TimeLeft;

/// Its Ready.
#[derive(Component)]
struct ReadyButton;

/// What a press on one of its buttons does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum ChooseTap {
    /// The look one on (1), or one back (-1).
    Turn(i32),
    Dice,
    /// The name, to type another.
    Name,
    Ready,
    /// The cross: Ready, and the window put away.
    Shut,
}

/// A press on one of the window's buttons: only what it landed on.
fn press(click: On<Pointer<Click>>, taps: Query<&ChooseTap>, mut chooser: ResMut<Chooser>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(&tap) = taps.get(click.entity) {
        chooser.pressed = Some(tap);
    }
}

/// The round the window is up for: the one being chosen for at the table
/// you are at, unless its cross put it away (`Blend::shut`, which a table
/// begun afresh has none of; in the first build it was the window's own,
/// and a round its cross had shut at one table stayed shut at every table
/// after, its window never up and Ready never pressed, everyone waiting
/// the choosing out).
pub(super) fn choosing(blend: &Blend) -> Option<u8> {
    blend
        .playing()
        .filter(|round| round.stage == BlendStage::Choose)
        .map(|round| round.round)
        .filter(|round| blend.shut != Some(*round))
}

/// Puts the window up as a round's choosing begins, and away as it ends or
/// its cross shuts it; lets the cursor go for it on desktop while it is up.
fn put_up(
    blend: Res<Blend>,
    mut chooser: ResMut<Chooser>,
    mut name_box: ResMut<NameBox>,
    mut wants: ResMut<WantsPointer>,
    windows: Query<Entity, With<ChooseWindow>>,
    mut commands: Commands,
) {
    let choosing = choosing(&blend);
    if chooser.up != choosing {
        if chooser.up.is_some() {
            for window in &windows {
                commands.entity(window).despawn();
            }
            if chooser.typing {
                name_box.close();
                chooser.typing = false;
            }
            chooser.drawn = None;
            chooser.pressed = None;
        }
        chooser.up = choosing;
    }
    // A table begun afresh takes down everything of its own: the window
    // is put up again.
    if chooser.up.is_some() && chooser.drawn.is_some() && windows.is_empty() {
        chooser.drawn = None;
    }
    let free = chooser.up.is_some();
    if free != chooser.pointer {
        if free || wants.0 {
            wants.set_if_neq(WantsPointer(free));
        }
        chooser.pointer = free;
    }
}

/// What the window's buttons do: the look on or back, a name thrown, the
/// name typed, Ready; each choice sent to the server as it is made
/// ([`super::ask`]), and shown at once, the server's answer to follow
/// (`Tell::Wear`).
#[allow(clippy::too_many_arguments)]
fn act(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut blend: ResMut<Blend>,
    mut chooser: ResMut<Chooser>,
    mut name_box: ResMut<NameBox>,
    mut asks: MessageWriter<NetAsk>,
) {
    let Some(round) = chooser.up else {
        chooser.pressed = None;
        return;
    };
    let ready = blend
        .seat
        .zip(blend.playing())
        .is_some_and(|(seat, now)| now.ready.get(seat).copied().unwrap_or(false));
    let mut pressed = chooser.pressed.take();
    if chooser.typing {
        // Whatever else took the box has it: the name as it was.
        if !name_box.up() {
            chooser.typing = false;
        } else if pressed.is_none() && (name_box.entered() || keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter])) {
            pressed = Some(ChooseTap::Name);
        } else if keyboard.just_pressed(KeyCode::Escape) {
            name_box.close();
            chooser.typing = false;
            return;
        }
    }
    let Some(tap) = pressed else {
        return;
    };
    // Ready is grey while the name being typed is no name: it does nothing.
    if chooser.typing && tap == ChooseTap::Ready && name_box.name().is_none() {
        return;
    }
    let (name, look) = blend.offered.clone().unwrap_or_default();
    // Done typing, by Enter, a press on the box again, or any other of
    // the window's buttons: the name typed sent, if it is one.
    let mut chosen = (name.clone(), look.clone());
    if chooser.typing {
        if let Some(typed) = name_box.name() {
            chosen.0 = typed;
        }
        name_box.close();
        chooser.typing = false;
    } else if tap == ChooseTap::Name && !ready && !name_box.up() {
        name_box.open(&name);
        chooser.typing = true;
        return;
    }
    if !ready {
        match tap {
            ChooseTap::Turn(by) => {
                let at = LOOKS.iter().position(|&each| each == chosen.1).unwrap_or(0);
                let next = LOOKS[(at as i32 + by).rem_euclid(LOOKS.len() as i32) as usize];
                let boy = BOY_LOOKS.contains(&next);
                // Over to the other side, by a name of the other side's:
                // one of this side's instead.
                if ai_names(!boy).contains(&chosen.0.as_str()) {
                    chosen.0 = thrown_name(boy, &chosen.0);
                }
                chosen.1 = next.to_owned();
            }
            ChooseTap::Dice => {
                chosen.0 = thrown_name(BOY_LOOKS.contains(&chosen.1.as_str()), &chosen.0);
            }
            ChooseTap::Name | ChooseTap::Ready | ChooseTap::Shut => {}
        }
        if chosen != (name, look) && profile::name(&chosen.0).is_some() && profile::look(&chosen.1) {
            ask(
                &mut asks,
                rules::Ask::Disguise {
                    name: chosen.0.clone(),
                    look: chosen.1.clone(),
                },
            );
            // Shown at once, and the next tap steps on from it: the
            // answers to this and to any before it are counted back in
            // (`super::hear`), the last of them what the server has.
            blend.offered = Some(chosen);
            blend.asked += 1;
            blend.refused = false;
        }
    }
    if matches!(tap, ChooseTap::Ready | ChooseTap::Shut) && !ready {
        ask(&mut asks, rules::Ask::Ready);
    }
    if tap == ChooseTap::Shut {
        blend.shut = Some(round);
    }
}

/// One of the town's own names for a boy (`boy`) or a girl, as the crowd
/// goes by, thrown, and never `but` (the one the box has).
fn thrown_name(boy: bool, but: &str) -> String {
    let names: Vec<&str> = ai_names(boy).iter().copied().filter(|&name| name != but).collect();
    fastrand::choice(names).unwrap_or(ai_names(boy)[0]).to_owned()
}

/// Draws the window afresh whenever what it shows changes: the round, your
/// role, your look and name, the box taken or let go, a name turned down,
/// who has chosen; and in the file's colours and words as the file changes
/// on desktop.
#[allow(clippy::too_many_arguments)]
fn draw(
    blend: Res<Blend>,
    view: Res<TableView>,
    changed: Res<UiChanged>,
    name_box: Res<NameBox>,
    pictures: Res<Pictures>,
    assets: Res<AssetServer>,
    mut chooser: ResMut<Chooser>,
    windows: Query<Entity, With<ChooseWindow>>,
    mut commands: Commands,
) {
    let (Some(_), Some(seat), Some(round)) = (chooser.up, blend.seat, blend.playing()) else {
        return;
    };
    let (name, look) = blend.offered.clone().unwrap_or_else(|| {
        view.members
            .get(seat)
            .map(|member| (member.name.clone(), member.look.clone()))
            .unwrap_or_default()
    });
    let now = Drawn {
        round: round.round,
        rounds: round.rounds,
        hiding: usize::from(round.hider) == seat,
        name,
        look,
        typing: chooser.typing,
        refused: blend.refused,
        ready: round.ready.get(seat).copied().unwrap_or(false),
        players: (0..round.points.len())
            .map(|other| {
                let name = view.members.get(other).map(|member| member.name.clone()).unwrap_or_default();
                (name, round.ready.get(other).copied().unwrap_or(false), usize::from(round.hider) == other)
            })
            .collect(),
    };
    let restyled = changed.is_changed() && !changed.is_added();
    if chooser.drawn.as_ref() == Some(&now) && !restyled {
        return;
    }
    for window in &windows {
        commands.entity(window).despawn();
    }
    let typed = if now.typing { name_box.text().to_owned() } else { now.name.clone() };
    spawn(&mut commands, &assets, &pictures, &now, &typed);
    chooser.drawn = Some(now);
}

/// The window, as `drawn` says, with `typed` in the box.
fn spawn(commands: &mut Commands, assets: &AssetServer, pictures: &Pictures, drawn: &Drawn, typed: &str) {
    let file = ui();
    let (colours, words) = (file.colours.window, &file.words.blend_in);
    let able = !drawn.ready;
    // Under the screen's own (the chat's lines and its box over it where
    // a narrower screen than a phone's has them meet, the buttons round
    // the jump button), as the world's names are.
    let root = commands
        .spawn((
            ChooseWindow,
            InTable,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexStart,
                padding: UiRect {
                    right: Val::VMin(RIGHT),
                    top: Val::VMin(TOP),
                    ..default()
                },
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(LAYER),
        ))
        .id();
    let pane = commands
        .spawn((
            ChildOf(root),
            HidesNames,
            LiftsForKeyboard::whole(),
            window::pane(Node {
                width: Val::VMin(WIDTH),
                padding: UiRect::all(Val::VMin(window::PAD)),
                row_gap: Val::VMin(1.6),
                ..default()
            }),
        ))
        .id();
    // Along its top the round, and at its far end the time left.
    let title = fill(&words.round_of, &[("round", &(drawn.round + 1).to_string()), ("rounds", &drawn.rounds.to_string())]);
    commands.spawn((
        ChildOf(pane),
        window::header(),
        children![
            window::words(title, window::TITLE, colours.title),
            (Node { flex_grow: 1.0, ..default() }, Pickable::IGNORE),
            (TimeLeft, window::words("", window::TITLE, colours.status)),
        ],
    ));
    commands.spawn((ChildOf(pane), window::cross(assets.load("icons/close.png"), ChooseTap::Shut)));
    let line = if drawn.hiding { &words.hiding_line } else { &words.seeking_line };
    commands.spawn((ChildOf(pane), wrapped(line.clone(), window::NAME, colours.name)));
    let body = commands
        .spawn((
            ChildOf(pane),
            Node {
                column_gap: Val::VMin(3.0),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    // Your look, between its arrows.
    let (card, _, _) = window::card(true);
    let ink = if able { colours.button_mark } else { colours.button_off };
    commands.spawn((
        ChildOf(body),
        Node {
            column_gap: Val::VMin(0.6),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
        children![
            arrow(pictures.arrow.clone(), -1, ink),
            (
                Node {
                    width: Val::VMin(CARD),
                    height: Val::VMin(CARD * 1.42),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::VMin(window::CARD_ROUNDED)),
                    ..default()
                },
                BackgroundColor(card.idle),
                Pickable::IGNORE,
                children![(
                    ImageNode::new(assets.load(format!("signup/{}.png", drawn.look))),
                    Node {
                        width: Val::VMin(PICTURE),
                        height: Val::VMin(PICTURE * 1.5),
                        ..default()
                    },
                    Pickable::IGNORE,
                )],
            ),
            arrow(pictures.arrow.clone(), 1, ink),
        ],
    ));
    // Your name, the dice beside it, what says a name was turned down, and
    // everyone with a tick by each who has chosen.
    let side = commands
        .spawn((
            ChildOf(body),
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                min_width: Val::ZERO,
                row_gap: Val::VMin(1.2),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands.spawn((ChildOf(side), window::words(words.name.clone(), window::STATUS, colours.status)));
    let row = commands
        .spawn((
            ChildOf(side),
            Node {
                column_gap: Val::VMin(1.6),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let field = Node {
        width: Val::VMin(FIELD_WIDTH),
        height: Val::VMin(window::FIELD_HEIGHT),
        padding: UiRect::horizontal(Val::VMin(2.4)),
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(Val::VMin(0.8)),
        overflow: Overflow::clip_x(),
        flex_shrink: 0.0,
        ..default()
    };
    if drawn.typing {
        commands.spawn((ChildOf(row), signup::field(typed, field, window::FIELD_TEXT, colours.field, colours.name)));
    } else {
        commands.spawn((
            ChildOf(row),
            ChooseTap::Name,
            hud::TakesPress,
            Button,
            field,
            BackgroundColor(colours.field),
            Fill {
                idle: colours.field,
                lit: if able { colours.button_lit } else { colours.field },
            },
            children![window::words(typed.to_owned(), window::FIELD_TEXT, if able { colours.name } else { colours.name_off })],
        ));
    }
    commands.spawn((ChildOf(row), ChooseTap::Dice, window::round(pictures.dice.clone(), window::ROW_BUTTON, able)));
    if drawn.typing {
        // "Pick another name." while the one typed has a swear in it.
        commands
            .spawn((ChildOf(side), signup::name_notice("", window::NOTICE_TEXT, colours.status)))
            .insert(TextLayout::justify(Justify::Left));
    } else {
        let said = if drawn.refused { words.refused.clone() } else { String::new() };
        commands.spawn((
            ChildOf(side),
            Node {
                min_height: Val::VMin(window::NOTICE_TEXT * 1.2),
                ..default()
            },
            Pickable::IGNORE,
            children![window::words(said, window::NOTICE_TEXT, colours.notice)],
        ));
    }
    let players = commands
        .spawn((
            ChildOf(side),
            Node {
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::VMin(2.6),
                row_gap: Val::VMin(0.8),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let tick = ui().colours.blend_in.tick;
    for (name, chose, hides) in &drawn.players {
        let one = commands
            .spawn((
                ChildOf(players),
                Node {
                    column_gap: Val::VMin(0.8),
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        commands.spawn((
            ChildOf(one),
            window::picture(pictures.tick.clone(), Val::VMin(TICK), if *chose { tick } else { Color::NONE }),
        ));
        commands.spawn((ChildOf(one), window::words(name.clone(), PLAYER_TEXT, colours.name)));
        if *hides {
            commands.spawn((ChildOf(one), window::words(words.hides.clone(), window::STATUS, file.colours.blend_in.accent)));
        }
    }
    // Ready, under them: clear of the jump button's corner, which the
    // window's own corner may be over.
    let (label, look) = if drawn.ready { (&words.waiting, Look::Greyed) } else { (&words.ready, Look::Yes) };
    commands.spawn((
        ChildOf(side),
        Node {
            margin: UiRect::top(Val::VMin(0.8)),
            ..default()
        },
        Pickable::IGNORE,
        children![(ReadyButton, ChooseTap::Ready, window::choice_wide(label.clone(), look, READY_WIDTH))],
    ));
}

/// An arrow beside your look, stepping `by` one on or one back: an
/// arrowhead in `ink` on nothing, pointing left, or turned to point right.
fn arrow(image: Handle<Image>, by: i32, ink: Color) -> impl Bundle {
    (
        ChooseTap::Turn(by),
        hud::TakesPress,
        Button,
        Node {
            width: Val::VMin(ARROW_TOUCH),
            height: Val::VMin(ARROW_TOUCH * 1.6),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        children![(
            ImageNode {
                flip_x: by > 0,
                color: ink,
                ..ImageNode::new(image)
            },
            Node {
                width: Val::VMin(ARROW),
                height: Val::VMin(ARROW * 1.4),
                ..default()
            },
            Pickable::IGNORE,
        )],
    )
}

/// Writes the time the choosing has left, by the server's clock, as it
/// changes.
fn count_down(blend: Res<Blend>, town: Res<TownClock>, mut times: Query<&mut Text, With<TimeLeft>>) {
    let Some(round) = blend.playing() else {
        return;
    };
    let left = table::clock((round.until - town.now()).max(0.0) as f32);
    for mut text in &mut times {
        if text.0 != left {
            text.0.clone_from(&left);
        }
    }
}

/// Greys Ready while the name being typed is no name, and gives it its
/// green back once it is; only as that changes.
fn light_ready(
    chooser: Res<Chooser>,
    name_box: Res<NameBox>,
    mut buttons: Query<(&mut Fill, &mut BackgroundColor, &Children), With<ReadyButton>>,
    mut labels: Query<&mut TextColor>,
) {
    let Some(drawn) = chooser.drawn.as_ref().filter(|drawn| !drawn.ready) else {
        return;
    };
    let able = !drawn.typing || name_box.name().is_some();
    let (fill, ink) = if able { Look::Yes } else { Look::Greyed }.colours();
    for (mut was, mut colour, children) in &mut buttons {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arrows_step_through_every_look_there_is_round_and_round() {
        for look in roundtown_net::BOY_LOOKS.iter().chain(&roundtown_net::GIRL_LOOKS) {
            assert!(LOOKS.contains(look), "{look}");
            assert!(profile::look(look));
        }
        assert_eq!(LOOKS.len(), 6);
        // Every look has its picture, as the sign-up has them.
        for look in LOOKS {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("assets/signup/{look}.png"));
            assert!(path.exists(), "{}", path.display());
        }
    }

    #[test]
    fn a_name_thrown_is_one_of_the_town_s_own_for_the_side_and_never_the_one_it_had() {
        for _ in 0..200 {
            let boy = thrown_name(true, "Kong");
            assert!(ai_names(true).contains(&boy.as_str()) && boy != "Kong");
            let girl = thrown_name(false, "Mochi");
            assert!(ai_names(false).contains(&girl.as_str()) && girl != "Mochi");
            assert!(profile::name(&boy).is_some() && profile::name(&girl).is_some());
        }
    }

    #[test]
    fn the_window_stands_clear_of_the_chat_on_a_phone_and_of_the_chat_s_button_everywhere() {
        // On a phone lying on its side (19.5 by 9: 216.7 shares across) the
        // chat down the left is 80 shares wide; the window stands to the
        // right of it.
        let phone = 216.7;
        assert!(phone - RIGHT - WIDTH >= hud::EDGE + 80.0);
        // The chat's small keyboard and the speech bubble round the jump
        // button are right of it, which is RIGHT in from the screen's edge,
        // the jump button's box `JUMP_RIGHT_VW` in from it (6 in a hundred of
        // the width, 10.7 shares on a screen 16 by 9 and 13 on a phone).
        for width in [133.3_f32, 177.8, 216.7] {
            let box_right = width * crate::JUMP_RIGHT_VW / 100.0;
            let (right, _, size) = hud::CHAT_BUTTON;
            let (quick_right, _, quick_size) = hud::QUICK_BUTTON;
            let leftmost = (box_right + right + size).max(box_right + quick_right + quick_size);
            assert!(RIGHT >= leftmost, "{width}: the buttons reach {leftmost} in");
        }
        // Its body: the look's card and its arrows beside the name's box
        // and the dice, inside the window's padding.
        let card = 2.0 * ARROW_TOUCH + CARD + 2.0 * 0.6;
        let side = FIELD_WIDTH + 1.6 + window::ROW_BUTTON;
        assert!(card + 3.0 + side <= WIDTH - 2.0 * window::PAD, "{}", card + 3.0 + side);
        // Under the points top right, and on the screen at its foot: its
        // padding, the title, the line (two lines), the card, and the gaps.
        let tall = 2.0 * window::PAD + window::TAB_HEIGHT + 2.0 * window::NAME * 1.2 + CARD * 1.42 + 3.0 * 1.6;
        assert!(TOP + tall <= 100.0 - hud::EDGE, "{}", TOP + tall);
    }
}
