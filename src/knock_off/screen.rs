//! What is drawn over the world in a game of Knock Off, every word and
//! colour of it the file's (`words.knock_off`, `colours.knock_off`):
//!
//! * **The buttons**: top left your name on its plate; on the right, where
//!   the jump button would be, an up arrow and a down arrow with a bar of
//!   ten steps between them, filled as hard as you aim and the number over
//!   it (Hajun: "right side is a up down arrow for controlling intensity"),
//!   and under them Ready, a round button with a tick, green once pressed;
//!   and the chat's keyboard. A press of an arrow is a step, and held it
//!   repeats. The stick on the left is the town's own, read for which way
//!   (`take_aim`).
//! * **The plate top centre**, the table's board's own look: the turn, and
//!   under it what to do now and the seconds left of it ("Aim! 7"), or how
//!   it is going ("Slide!", "The ice breaks"), or, out, where you came.
//! * **Names over heads**, each in its seat's colour, with a dark stroke so
//!   that it reads on white ice by day and by night, and a green tick over
//!   whoever is ready (the computer's players as they make up their minds),
//!   yours too, though your name is never over you (Hajun, 2026-10-31).
//!   Knock Off's camera is far enough out to see the whole island that the
//!   town's names (`tags`), sized by how far off they are, could not be
//!   read: these are as big wherever they are, as Bridge Wars' are.
//! * **The headlines** (`hud::announce`): "Go!" with how to play as the
//!   first turn begins, each turn after, and who won.
//! * **A line in the chat** for whoever goes into the sea, and how ("Mochi
//!   knocked Dubu into the sea"), names in their colours.
//! * **The panel** for whoever is out, as Bridge Wars' "You're out!" is:
//!   their place, Watch (stay, the camera as it is) and Leave (to the town,
//!   their place kept and paid).

use bevy::asset::RenderAssetUsages;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_replicon::prelude::Remote;
use roundtown_net::Member;
use roundtown_net::knock_off::{KnockRound, KnockStage};

use super::{Holding, Knock, KnockSystems, Pressed, bodies_by_seat, seat_colour};
use crate::builder::{self, DIALOG_LAYER};
use crate::chat::{self, Log};
use crate::clock::TownClock;
use crate::hud::{self, TakesPress};
use crate::table::{self, ComputerSeat, InTable, TableView};
use crate::ui::{UiChanged, fill, ui};
use crate::window::{self, Look};
use crate::{Dead, Player, ThirdPersonCamera, Venue};

/// The buttons on the right, in shares of the short side of the screen
/// from the right and the bottom of the box round the jump button
/// (`hud::beside_the_jump`), each with its size: Ready under the rest, the
/// down arrow over it, the bar of steps over that, its number over the bar,
/// and the up arrow at the top; and the chat's keyboard to Ready's left.
const READY: (f32, f32, f32) = (4.0, 0.0, 14.0);
const DOWN: (f32, f32, f32) = (5.5, 16.5, 11.0);
const BAR: (f32, f32) = (7.75, 29.0);
const BAR_SIZE: (f32, f32) = (6.5, 16.0);
const NUMBER: (f32, f32) = (5.0, 45.3);
const NUMBER_TEXT: f32 = 4.0;
const UP: (f32, f32, f32) = (5.5, 51.0, 11.0);
const CHAT: (f32, f32, f32) = (20.0, 2.5, 9.0);
/// How much of a round button its picture fills, as the town's do.
const PICTURE: f32 = 58.0;
/// The gap between two steps of the bar.
const STEP_GAP: f32 = 0.3;
/// The plate top centre, as the table's board is: its title and the line
/// under it.
const PLATE_TEXT: f32 = 4.8;
const PLATE_CAPTION: f32 = 3.4;
/// A name over a head, and the tick over it: how big, and how far over
/// the feet, in metres.
const TAG_TEXT: f32 = 2.7;
const TICK: f32 = 2.8;
const TAG_OVER: f32 = 2.05;
/// How far the dark stroke round a name stands out from its letters.
const STROKE: f32 = 0.16;
/// The panel for whoever is out: how wide, its title and its line, and
/// its buttons.
const PANEL_WIDTH: f32 = 60.0;
const PANEL_TITLE: f32 = 4.6;
const PANEL_TEXT: f32 = 3.0;
const PANEL_BUTTON: f32 = 22.0;
/// How long the headlines are up, in seconds.
const GO_SECS: f32 = 3.0;
const TURN_SECS: f32 = 1.4;
const WON_SECS: f32 = 3.0;

/// One of the power arrows on the right: up, or down.
#[derive(Component, Clone, Copy)]
struct PowerArrow {
    up: bool,
}

/// The Ready button.
#[derive(Component)]
struct ReadyButton;

/// One of the ten steps of the bar, counted from the bottom.
#[derive(Component)]
struct PowerStep(u8);

/// The number over the bar, and the dark copies under it that stroke it.
#[derive(Component, Clone, Copy)]
struct PowerNumber;

/// The plate top centre, its title and the line under it.
#[derive(Component)]
struct Plate;
#[derive(Component)]
struct PlateTitle;
#[derive(Component)]
struct PlateCaption;

/// What is over the head of the player in a seat: the name, in its
/// colour, with its stroke; and the tick.
#[derive(Component)]
struct OverHead(usize);
#[derive(Component, Clone, Copy)]
struct TagName;
#[derive(Component)]
struct ReadyTick;

/// The panel for whoever is out, and what its buttons do.
#[derive(Component)]
struct OutPanel;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum PanelTap {
    Watch,
    Leave,
}

pub(super) fn plugin(app: &mut App) {
    app.add_observer(press).add_observer(tap).add_systems(
        Update,
        (
            (let_go, show_power).chain().after(KnockSystems),
            (put_up, know_the_place, plate, headlines, chat_lines, out_panel).chain().after(super::place),
            over_heads.after(crate::follow_camera),
        ),
    );
}

/// The buttons round the screen at a table of Knock Off.
pub(super) fn spawn_bar(commands: &mut Commands, _assets: &AssetServer, images: &mut Assets<Image>) {
    let (left, _right) = hud::game_bar(commands);
    commands.spawn((ChildOf(left), hud::name_plate()));
    let colours = ui().colours;
    let arrow = images.add(hud::jump_arrow());
    let tick = images.add(tick());
    commands.spawn(hud::round_the_jump()).with_children(|round| {
        let (right, bottom, size) = UP;
        round.spawn((PowerArrow { up: true }, arrow_button(arrow.clone(), size, false, hud::beside_the_jump(right, bottom))));
        let (right, bottom, size) = DOWN;
        round.spawn((PowerArrow { up: false }, arrow_button(arrow, size, true, hud::beside_the_jump(right, bottom))));
        let (right, bottom, size) = READY;
        round.spawn((ReadyButton, hud::round_button(tick, size, hud::beside_the_jump(right, bottom))));
        // The bar, its ten steps from the bottom up.
        let (right, bottom) = BAR;
        round
            .spawn((
                Node {
                    width: Val::VMin(BAR_SIZE.0),
                    height: Val::VMin(BAR_SIZE.1),
                    flex_direction: FlexDirection::ColumnReverse,
                    row_gap: Val::VMin(STEP_GAP),
                    padding: UiRect::all(Val::VMin(0.5)),
                    border_radius: BorderRadius::all(Val::VMin(1.2)),
                    ..hud::beside_the_jump(right, bottom)
                },
                BackgroundColor(colours.panel),
                Pickable::IGNORE,
            ))
            .with_children(|bar| {
                for step in 0..roundtown_net::knock_off::MOST_POWER {
                    bar.spawn((
                        PowerStep(step),
                        Node {
                            flex_grow: 1.0,
                            border_radius: BorderRadius::all(Val::VMin(0.5)),
                            ..default()
                        },
                        BackgroundColor(colours.knock_off.power_off),
                        Pickable::IGNORE,
                    ));
                }
            });
        // Its number over it, stroked dark.
        let (right, bottom) = NUMBER;
        round
            .spawn((
                Node {
                    width: Val::VMin(BAR_SIZE.0 + 5.0),
                    justify_content: JustifyContent::Center,
                    ..hud::beside_the_jump(right, bottom)
                },
                Pickable::IGNORE,
            ))
            .with_children(|number| stroked(number, PowerNumber, "0", NUMBER_TEXT, colours.text));
        let (right, bottom, size) = CHAT;
        round.spawn((hud::ChatButton, hud::round_button(images.add(hud::keyboard()), size, hud::beside_the_jump(right, bottom))));
    });
}

/// A round button with the jump button's arrow on it, `size` across, where
/// `place` puts it, the arrow turned down for the down one.
fn arrow_button(arrow: Handle<Image>, size: f32, down: bool, place: Node) -> impl Bundle {
    let colours = ui().colours.hud;
    let mut picture = ImageNode::new(arrow).with_color(colours.button_mark);
    picture.flip_y = down;
    (
        TakesPress,
        Button,
        Node {
            width: Val::VMin(size),
            height: Val::VMin(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..place
        },
        BackgroundColor(colours.button),
        children![(
            picture,
            Node {
                width: Val::Percent(PICTURE),
                height: Val::Percent(PICTURE),
                ..default()
            },
            Pickable::IGNORE,
        )],
    )
}

/// Words `text`, `size` big, in `colour`, with a thin dark stroke round them
/// (four dark copies under them, as your name's plate has in Bridge Wars,
/// `hud::name_plate`): what reads on white ice and on grey sea alike. The
/// words carry `marker`; the copies are each `marker`'s too, under it,
/// marked as copies.
fn stroked(parent: &mut ChildSpawnerCommands, marker: impl Component + Copy, text: &str, size: f32, colour: Color) {
    let stroke = ui().colours.chat.stroke;
    parent
        .spawn((Node::default(), Pickable::IGNORE))
        .with_children(|words| {
            for corner in [Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0), Vec2::new(-1.0, 1.0), Vec2::new(1.0, 1.0)] {
                words.spawn((
                    marker,
                    Stroke,
                    Text::new(text),
                    crate::font::sized(size),
                    TextColor(stroke),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::ZERO,
                        top: Val::ZERO,
                        ..default()
                    },
                    UiTransform::from_translation(Val2::new(Val::VMin(corner.x * STROKE), Val::VMin(corner.y * STROKE))),
                    Pickable::IGNORE,
                ));
            }
            words.spawn((marker, Text::new(text), crate::font::sized(size), TextColor(colour), Pickable::IGNORE));
        });
}

/// One of the dark copies under words that stroke them ([`stroked`]).
#[derive(Component, Clone, Copy)]
struct Stroke;

/// A press on one of the buttons on the right, as it goes down: a step of
/// power up or down, the arrow held from then until its pointer is up
/// (`let_go`), or Ready.
fn press(
    press: On<Pointer<Press>>,
    arrows: Query<&PowerArrow>,
    readies: Query<(), With<ReadyButton>>,
    mut pressed: ResMut<Pressed>,
) {
    if press.button != PointerButton::Primary || press.original_event_target() != press.entity {
        return;
    }
    if let Ok(arrow) = arrows.get(press.entity) {
        pressed.power += if arrow.up { 1 } else { -1 };
        pressed.held = Some(Holding {
            pointer: press.pointer_id,
            up: arrow.up,
            held: 0.0,
            repeated: 0,
        });
    }
    if readies.contains(press.entity) {
        pressed.ready = true;
    }
}

/// A power arrow held down is let go of once its pointer is up, a finger
/// or the mouse, wherever it went.
fn let_go(touches: Res<Touches>, mouse: Res<ButtonInput<MouseButton>>, mut pressed: ResMut<Pressed>) {
    let Some(holding) = pressed.held else {
        return;
    };
    let down = match holding.pointer {
        PointerId::Touch(id) => touches.get_pressed(id).is_some(),
        PointerId::Mouse => mouse.pressed(MouseButton::Left),
        PointerId::Custom(_) => false,
    };
    if !down {
        pressed.held = None;
    }
}

/// The bar filled as hard as you aim, its number over it, and Ready green
/// once pressed; the arrows and Ready faint while there is no aiming to do.
/// Each written only as it changes.
#[allow(clippy::type_complexity)]
fn show_power(
    knock: Res<Knock>,
    mut steps: Query<(&PowerStep, &mut BackgroundColor), (Without<ReadyButton>, Without<PowerArrow>)>,
    mut numbers: Query<(&mut Text, Has<Stroke>), With<PowerNumber>>,
    mut readies: Query<&mut BackgroundColor, (With<ReadyButton>, Without<PowerStep>, Without<PowerArrow>)>,
    mut arrows: Query<&mut BackgroundColor, (With<PowerArrow>, Without<PowerStep>, Without<ReadyButton>)>,
) {
    let colours = ui().colours;
    let aim = knock.aim;
    let current = knock.playing().is_some_and(|round| round.stage == KnockStage::Aim && aim.turn == round.turn);
    let power = if current { aim.power } else { 0 };
    for (step, mut back) in &mut steps {
        back.set_if_neq(BackgroundColor(if step.0 < power { colours.knock_off.power } else { colours.knock_off.power_off }));
    }
    let said = power.to_string();
    for (mut text, _) in &mut numbers {
        if text.0 != said {
            text.0.clone_from(&said);
        }
    }
    let aiming = knock.aiming();
    let ready = current && aim.ready;
    let button = colours.hud.button;
    let faint = button.with_alpha(button.alpha() * 0.45);
    for mut back in &mut readies {
        back.set_if_neq(BackgroundColor(if ready {
            colours.knock_off.ready
        } else if aiming {
            button
        } else {
            faint
        }));
    }
    for mut back in &mut arrows {
        back.set_if_neq(BackgroundColor(if aiming { button } else { faint }));
    }
}

/// Puts up the plate top centre and a name over every seat's head as you
/// sit down at a table of Knock Off (the table took down those of the one
/// before), and again in the file's colours as it changes on desktop.
fn put_up(
    knock: Res<Knock>,
    changed: Res<UiChanged>,
    plates: Query<Entity, With<Plate>>,
    heads: Query<Entity, With<OverHead>>,
    mut images: ResMut<Assets<Image>>,
    mut tick_image: Local<Option<Handle<Image>>>,
    mut commands: Commands,
) {
    if knock.seat.is_none() {
        return;
    }
    let redraw = changed.is_changed() && !changed.is_added();
    if redraw {
        for thing in plates.iter().chain(heads.iter()) {
            commands.entity(thing).despawn();
        }
    } else if !plates.is_empty() {
        return;
    }
    let colours = ui().colours;
    commands
        .spawn((
            InTable,
            Plate,
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(hud::EDGE),
                left: Val::ZERO,
                right: Val::ZERO,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(0.8),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|plate| {
            plate.spawn((
                crate::tags::HidesNames,
                Node {
                    padding: UiRect::axes(Val::VMin(3.0), Val::VMin(0.7)),
                    border_radius: BorderRadius::all(Val::VMin(3.0)),
                    ..default()
                },
                BackgroundColor(colours.panel),
                Pickable::IGNORE,
                children![(PlateTitle, hud::words("", PLATE_TEXT, colours.text))],
            ));
            plate.spawn((PlateCaption, hud::words("", PLATE_CAPTION, colours.text)));
        });
    let tick = tick_image.get_or_insert_with(|| images.add(tick())).clone();
    for seat in 0..4 {
        commands
            .spawn((
                InTable,
                OverHead(seat),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    top: Val::ZERO,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::VMin(0.2),
                    ..default()
                },
                UiTransform::default(),
                Visibility::Hidden,
                GlobalZIndex(-1),
                Pickable::IGNORE,
            ))
            .with_children(|head| {
                head.spawn((
                    ReadyTick,
                    ImageNode::new(tick.clone()).with_color(colours.knock_off.ready),
                    Node {
                        width: Val::VMin(TICK),
                        height: Val::VMin(TICK),
                        ..default()
                    },
                    Visibility::Hidden,
                    Pickable::IGNORE,
                ));
                stroked(head, TagName, "", TAG_TEXT, seat_colour(seat));
            });
    }
}

/// The plate top centre while the game is played: the turn, and under it
/// what to do now and the seconds left of it, or how it is going; out,
/// where you came. Up from the first turn's round to the end of play, its
/// words written only as they change. Before it, the table's board says
/// the countdown in the same place ("Starting in 3"), and the first turn's
/// round comes with the play's first moment: the plate has nothing before
/// it to say (it had a "Get ready" there, which a check of the game found
/// never shown).
#[allow(clippy::type_complexity)]
fn plate(
    clock: Res<TownClock>,
    knock: Res<Knock>,
    view: Res<TableView>,
    mut plates: Query<&mut Visibility, With<Plate>>,
    mut titles: Query<&mut Text, (With<PlateTitle>, Without<PlateCaption>)>,
    mut captions: Query<&mut Text, (With<PlateCaption>, Without<PlateTitle>)>,
) {
    let round = knock.playing().filter(|round| knock.seat.is_some() && view.playing() && round.stage != KnockStage::Over);
    for mut plate in &mut plates {
        plate.set_if_neq(if round.is_some() { Visibility::Inherited } else { Visibility::Hidden });
    }
    let Some(round) = round else {
        return;
    };
    let ui = ui();
    let words = &ui.words.knock_off;
    let title = if round.next_radius <= 0.0 {
        words.last_turn.clone()
    } else {
        fill(&words.turn, &[("turn", &round.turn.to_string())])
    };
    let seconds = ((round.until - clock.now()).max(0.0).ceil() as u32).to_string();
    let caption = if !knock.in_play() {
        match knock.place {
            Some(place) => fill(&words.watching, &[("place", &place_word(place))]),
            None => words.went_in.clone(),
        }
    } else {
        match round.stage {
            KnockStage::Aim if knock.aim.ready && knock.aim.turn == round.turn => fill(&words.ready, &[("seconds", &seconds)]),
            KnockStage::Aim => fill(&words.aim, &[("seconds", &seconds)]),
            KnockStage::Reveal | KnockStage::Slide => words.sliding.clone(),
            KnockStage::Break => words.breaks.clone(),
            KnockStage::Over => String::new(),
        }
    };
    for mut text in &mut titles {
        if text.0 != title {
            text.0.clone_from(&title);
        }
    }
    for mut text in &mut captions {
        if text.0 != caption {
            text.0.clone_from(&caption);
        }
    }
}

/// Where the player in `seat`, gone into the sea, came, 0 first: the game's
/// own word for it, as the server sends it (`KnockRound::places`, the rule
/// the results go by, and nothing worked out here), once the slide of the
/// turn they went in is over: until then, whoever else goes in at the very
/// same moment of it would share it, and the panel saying it waits for the
/// slide anyway, for them to be seen into the sea first. None while they
/// are on the ice.
fn place_of(round: &KnockRound, seat: usize) -> Option<u8> {
    let went = round.out.get(seat).copied().flatten()?;
    if went == round.turn && matches!(round.stage, KnockStage::Aim | KnockStage::Reveal | KnockStage::Slide) {
        return None;
    }
    round.places.get(seat).copied().flatten()
}

/// Where you came, once you are out and it is known ([`place_of`]).
fn know_the_place(mut knock: ResMut<Knock>) {
    let (Some(seat), Some(round)) = (knock.seat, knock.playing()) else {
        return;
    };
    let place = place_of(round, usize::from(seat));
    if place.is_some() && place != knock.place {
        knock.place = place;
    }
}

/// A place as the results say it: the file's first to fourth.
fn place_word(place: u8) -> String {
    let ui = ui();
    ui.words
        .table
        .places
        .get(usize::from(place))
        .cloned()
        .unwrap_or_else(|| format!("{}th", u32::from(place) + 1))
}

/// The headlines: "Go!" as the first turn begins, with how to play; each
/// turn after, and the last turn as it is; and once it is over, who won.
/// A turn's only while it is news, within its own seconds of the turn
/// beginning by the town's clock: not to a device that has come back to the
/// game in the middle of an aim, for which it is old.
#[allow(clippy::too_many_arguments)]
fn headlines(
    clock: Res<TownClock>,
    knock: Res<Knock>,
    view: Res<TableView>,
    mut said: Local<Option<(u16, KnockStage)>>,
    mut won: Local<bool>,
    mut commands: Commands,
) {
    if knock.seat.is_none() {
        *said = None;
        *won = false;
        return;
    }
    let ui = ui();
    let words = &ui.words.knock_off;
    if let Some(round) = knock.playing() {
        let now = (round.turn, round.stage);
        let news = |secs: f32| clock.now() - round.since < f64::from(secs);
        if *said != Some(now) && round.stage == KnockStage::Aim {
            if round.turn == 1 && news(GO_SECS) {
                hud::announce(&mut commands, Some(&words.how), &words.go, hud::HEADLINE, GO_SECS).insert(InTable);
            } else if round.turn > 1 && knock.in_play() && news(TURN_SECS) {
                let title = if round.next_radius <= 0.0 {
                    words.last_turn_headline.clone()
                } else {
                    fill(&words.turn, &[("turn", &round.turn.to_string())])
                };
                hud::announce(&mut commands, None, &title, hud::HEADLINE * 0.8, TURN_SECS).insert(InTable);
            }
        }
        *said = Some(now);
    }
    // Who won: the first place of the results, as the table has them.
    if let Some(results) = view.results()
        && !*won
    {
        *won = true;
        let first: Vec<usize> = (0..results.places.len()).filter(|&seat| results.places[seat] == 0).collect();
        let title = if knock.seat.is_some_and(|seat| first == [usize::from(seat)]) {
            words.you_win.clone()
        } else {
            let names: Vec<String> = first
                .iter()
                .filter_map(|&seat| view.members.get(seat))
                .map(|member| member.name.clone())
                .collect();
            winners(&names)
        };
        hud::announce(&mut commands, None, &title, hud::HEADLINE, WON_SECS).insert(InTable);
    }
    if view.results().is_none() {
        *won = false;
    }
}

/// Who won, in the file's words: "Mochi wins!", or for a first shared,
/// "Mochi & Dubu win!", the names joined as the file joins them
/// (`words.knock_off.wins`, `win_together`, `and`).
fn winners(names: &[String]) -> String {
    let ui = ui();
    let words = &ui.words.knock_off;
    match names {
        [one] => fill(&words.wins, &[("name", one)]),
        _ => fill(&words.win_together, &[("names", &names.join(&words.and))]),
    }
}

/// A line in the chat for each who goes into the sea, as they do, and how:
/// slid in on their own, knocked in by whoever last hit them (this device's
/// own play of the slide says who), or with the ice that broke under them;
/// or that they left; and at the end, who won. Names in their seats'
/// colours.
#[allow(clippy::too_many_arguments)]
fn chat_lines(knock: Res<Knock>, view: Res<TableView>, mut log: ResMut<Log>, mut said: Local<(Vec<bool>, bool)>) {
    let Some(round) = knock.playing() else {
        *said = (Vec::new(), false);
        return;
    };
    let ui = ui();
    let words = &ui.words.knock_off;
    let ink = ui.colours.chat.event;
    let name = |seat: usize| view.members.get(seat).map_or_else(String::new, |member| member.name.clone());
    if said.0.len() != round.out.len() {
        // Begun afresh: whoever is out already was said, or is not news.
        said.0 = round.out.iter().map(Option::is_some).collect();
    }
    let slide = knock.slide.as_ref().filter(|played| played.turn == round.turn).map(|played| &played.slide);
    for seat in 0..round.out.len() {
        if said.0[seat] || round.out[seat].is_none() {
            continue;
        }
        let in_slide = slide.and_then(|slide| slide.over_edge(seat));
        let line = match round.stage {
            KnockStage::Slide if in_slide.is_none() => continue,
            KnockStage::Slide => {
                let by = slide.and_then(|slide| slide.hit_by(seat));
                match by {
                    Some(by) => chat::spans(&words.knocked_in, ink, &[("by", &name(by), seat_colour(by)), ("name", &name(seat), seat_colour(seat))]),
                    None => chat::spans(&words.slid_in, ink, &[("name", &name(seat), seat_colour(seat))]),
                }
            }
            KnockStage::Break if round.start.get(seat).copied().flatten().is_some() => {
                chat::spans(&words.broke_under, ink, &[("name", &name(seat), seat_colour(seat))])
            }
            _ if in_slide.is_some() => chat::spans(&words.slid_in, ink, &[("name", &name(seat), seat_colour(seat))]),
            _ => chat::spans(&words.left, ink, &[("name", &name(seat), seat_colour(seat))]),
        };
        said.0[seat] = true;
        log.push(line);
    }
    if let Some(results) = view.results()
        && !said.1
    {
        said.1 = true;
        let first: Vec<usize> = (0..results.places.len()).filter(|&seat| results.places[seat] == 0).collect();
        if let [one] = first[..] {
            log.push(chat::spans(&words.won, ink, &[("name", &name(one), seat_colour(one))]));
        } else if !first.is_empty() {
            // A first shared: one line, every name in its own colour.
            let keys: Vec<String> = (0..first.len()).map(|k| format!("name{k}")).collect();
            let names: Vec<String> = first.iter().map(|&seat| name(seat)).collect();
            let joined = keys.iter().map(|key| format!("{{{key}}}")).collect::<Vec<_>>().join(&words.and);
            let template = fill(&words.won_together, &[("names", &joined)]);
            let fills: Vec<(&str, &str, Color)> = first
                .iter()
                .enumerate()
                .map(|(k, &seat)| (keys[k].as_str(), names[k].as_str(), seat_colour(seat)))
                .collect();
            log.push(chat::spans(&template, ink, &fills));
        }
    }
}

/// The panel for whoever is out, once their turn's slide is over and where
/// they came is known: "You're out! 3rd", Watch and Leave. Not once the
/// game is over (its results say it all), nor once Watch is pressed.
fn out_panel(
    knock: Res<Knock>,
    view: Res<TableView>,
    panels: Query<Entity, With<OutPanel>>,
    mut commands: Commands,
) {
    let place = match knock.playing() {
        Some(round) if !knock.watching && !knock.leaving && !view.over() && round.stage != KnockStage::Over => knock.place,
        _ => None,
    };
    let up = panels.single().ok();
    match (place, up) {
        (Some(place), None) => spawn_panel(&mut commands, place),
        (None, Some(panel)) => commands.entity(panel).despawn(),
        _ => {}
    }
}

/// The panel saying you are out, and where you came, with Watch and
/// Leave: drawn as the friends' panel is, and laid out as Bridge Wars'
/// "You're out!" is (`arena`), not a dialog, the game going on round it.
fn spawn_panel(commands: &mut Commands, place: u8) {
    let ui = ui();
    let (colours, words) = (ui.colours.window, &ui.words.knock_off);
    commands
        .spawn((
            InTable,
            OutPanel,
            crate::tags::HidesNames,
            window::pane(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Vh(26.0),
                width: Val::VMin(PANEL_WIDTH),
                align_items: AlignItems::Center,
                row_gap: Val::VMin(1.6),
                padding: UiRect::all(Val::VMin(window::PAD)),
                ..default()
            }),
            UiTransform::from_translation(Val2::new(Val::Percent(-50.0), Val::ZERO)),
            GlobalZIndex(DIALOG_LAYER - 1),
        ))
        .with_children(|panel| {
            panel.spawn(builder::label(fill(&words.out, &[("place", &place_word(place))]), PANEL_TITLE, colours.title));
            panel.spawn(builder::label(words.out_line.clone(), PANEL_TEXT, colours.status));
            panel
                .spawn((
                    Node {
                        justify_content: JustifyContent::Center,
                        column_gap: Val::VMin(3.0),
                        margin: UiRect::top(Val::VMin(1.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|choices| {
                    choices.spawn((PanelTap::Leave, window::choice_wide(words.leave.clone(), Look::No, PANEL_BUTTON)));
                    choices.spawn((PanelTap::Watch, window::choice_wide(words.watch.clone(), Look::Yes, PANEL_BUTTON)));
                });
        });
}

/// A press on the panel's buttons: Watch puts it away, the camera as it
/// is; Leave goes to the town, your place kept and paid: online the server
/// pays it at the end; offline the rest of the game is played out first
/// (`play_offline`), and you go as its results come.
fn tap(click: On<Pointer<Click>>, taps: Query<&PanelTap>, mut knock: ResMut<Knock>, mut asks: MessageWriter<table::Ask>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    match taps.get(click.entity) {
        Ok(PanelTap::Watch) => knock.watching = true,
        Ok(PanelTap::Leave) if !knock.online => knock.leaving = true,
        Ok(PanelTap::Leave) => {
            asks.write(table::Ask::Leave);
        }
        Err(_) => {}
    }
}

/// Over every head on the ice its name, in its seat's colour (never over
/// yours), and while everyone aims, a tick over whoever is ready: as big
/// wherever they are, kept over their heads as the camera is, and put away
/// while they are in the sea, off the screen, or anywhere but on the ice.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn over_heads(
    knock: Res<Knock>,
    view: Res<TableView>,
    me: Query<Entity, With<Player>>,
    remotes: Query<(Entity, &Member), With<Remote>>,
    computers: Query<(Entity, &ComputerSeat)>,
    here: Query<&Venue, With<Player>>,
    bodies: Query<(&Transform, &Venue, Has<Dead>), Without<ThirdPersonCamera>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    hiding: Query<(&ComputedNode, &UiGlobalTransform, &InheritedVisibility), (With<crate::tags::HidesNames>, Without<OverHead>)>,
    mut heads: Query<(&OverHead, &Children, &mut Visibility, &mut UiTransform, &ComputedNode), (Without<ReadyTick>, Without<TagName>)>,
    mut ticks: Query<&mut Visibility, (With<ReadyTick>, Without<OverHead>, Without<TagName>)>,
    mut names: Query<(&mut Text, &mut Visibility), (With<TagName>, Without<OverHead>, Without<ReadyTick>)>,
    parts: Query<&Children>,
) {
    let (Ok((camera, eye)), Ok(window)) = (cameras.single(), windows.single()) else {
        return;
    };
    let on_the_ice = here.single().is_ok_and(|venue| *venue == Venue::KnockOff);
    let eye = GlobalTransform::from(*eye);
    let by_seat = bodies_by_seat(&knock, &view, me.single().ok(), &remotes, &computers);
    let round = knock.playing();
    let aiming = round.is_some_and(|round| round.stage == KnockStage::Aim);
    // What no name is shown under: the results, the panel for whoever is
    // out, the plate top centre (`tags::HidesNames`), where they are in the
    // window's logical pixels.
    let hidden: Vec<Rect> = hiding
        .iter()
        .filter(|(.., shown)| shown.get())
        .map(|(node, place, _)| {
            let scale = node.inverse_scale_factor();
            Rect::from_center_size(place.translation * scale, node.size() * scale)
        })
        .collect();
    for (head, children, mut shown, mut place, node) in &mut heads {
        let seat = head.0;
        let member = view.members.get(seat).filter(|member| member.id != 0);
        let mine = knock.seat == Some(seat as u8);
        let body = by_seat
            .get(seat)
            .copied()
            .flatten()
            .filter(|_| on_the_ice && (member.is_some() || mine))
            .and_then(|body| bodies.get(body).ok())
            .filter(|(_, venue, dead)| **venue == Venue::KnockOff && !dead);
        let point = body
            .and_then(|(at, ..)| camera.world_to_viewport(&eye, at.translation + Vec3::Y * TAG_OVER).ok())
            .filter(|point| point.x >= 0.0 && point.y >= 0.0 && point.x <= window.width() && point.y <= window.height());
        let ready = aiming && round.is_some_and(|round| round.ready.get(seat).copied().unwrap_or(false))
            || (mine && aiming && knock.aim.ready && round.is_some_and(|round| knock.aim.turn == round.turn));
        let named = member.is_some() && !mine;
        let Some(point) = point.filter(|_| named || ready) else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let size = node.size() * node.inverse_scale_factor();
        let rect = Rect::from_center_size(Vec2::new(point.x, point.y - size.y * 0.5), size);
        if hidden.iter().any(|hider| !hider.intersect(rect).is_empty()) {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        }
        place.set_if_neq(UiTransform {
            translation: Val2::px(point.x - size.x * 0.5, point.y - size.y),
            scale: Vec2::ONE,
            rotation: Rot2::IDENTITY,
        });
        shown.set_if_neq(Visibility::Inherited);
        let wanted_name = if named { member.map_or("", |member| member.name.as_str()) } else { "" };
        for part in children.iter() {
            if let Ok(mut tick) = ticks.get_mut(part) {
                tick.set_if_neq(if ready { Visibility::Inherited } else { Visibility::Hidden });
            }
            for word in parts.get(part).into_iter().flat_map(|words| words.iter()) {
                if let Ok((mut text, mut seen)) = names.get_mut(word) {
                    if text.0 != wanted_name {
                        text.0 = wanted_name.to_string();
                    }
                    seen.set_if_neq(if named { Visibility::Inherited } else { Visibility::Hidden });
                }
            }
        }
    }
}

/// A tick, drawn white for a tint: a short stroke down to the right and a
/// long one up, its corners rounded.
fn tick() -> Image {
    const SIZE: u32 = 96;
    const THICK: f32 = 0.17;
    let corners = [Vec2::new(-0.62, 0.02), Vec2::new(-0.18, -0.44), Vec2::new(0.66, 0.5)];
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new((column as f32 + 0.5) * pixel - 1.0, 1.0 - (row as f32 + 0.5) * pixel);
            let near = segment_distance(at, corners[0], corners[1]).min(segment_distance(at, corners[1], corners[2]));
            let cover = (0.5 - (near - THICK) / pixel).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (cover * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// How far `at` is from the line from `a` to `b`.
fn segment_distance(at: Vec2, a: Vec2, b: Vec2) -> f32 {
    let along = b - a;
    let share = ((at - a).dot(along) / along.length_squared().max(1e-9)).clamp(0.0, 1.0);
    at.distance(a + along * share)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_buttons_on_the_right_are_clear_of_each_other() {
        let round = |(right, bottom, size): (f32, f32, f32)| ((right, bottom), size);
        hud::clear_of_each_other(&[round(READY), round(DOWN), round(UP), round(CHAT)]).expect("clear of each other");
        // The bar between the arrows, touching neither.
        assert!(BAR.1 > DOWN.1 + DOWN.2 + 0.5 && BAR.1 + BAR_SIZE.1 < NUMBER.1 && NUMBER.1 + NUMBER_TEXT < UP.1 + 0.5);
    }

    #[test]
    fn where_you_came_is_the_game_s_word_for_it_once_your_slide_is_over() {
        // Seat 0 went in a turn before, seat 2 in this turn's slide and seat
        // 3 with the ice after it, as the server places them; seat 1 is on
        // the ice.
        let mut round = KnockRound {
            turn: 2,
            stage: KnockStage::Break,
            since: 0.0,
            until: 1.0,
            radius: 11.0,
            next_radius: 10.0,
            start: vec![None, Some((1.0, 1.0)), None, Some((10.8, 0.0))],
            launches: Vec::new(),
            ready: vec![false; 4],
            out: vec![Some(1), None, Some(2), Some(2)],
            places: vec![Some(3), None, Some(2), Some(1)],
        };
        assert_eq!(place_of(&round, 1), None, "still on the ice");
        assert_eq!(place_of(&round, 3), Some(1));
        assert_eq!(place_of(&round, 2), Some(2));
        assert_eq!(place_of(&round, 0), Some(3));
        // Until this turn's slide is over, not said of whoever went in in it
        // (whoever goes in at the very same moment would share it); of a
        // turn before, as it is.
        round.stage = KnockStage::Slide;
        round.out[3] = None;
        round.places[3] = None;
        assert_eq!(place_of(&round, 2), None);
        assert_eq!(place_of(&round, 0), Some(3));
        // And whatever this device would have made of it, the game's: a
        // device back in the middle of the game, with none of the slide that
        // put anyone in, has it all the same.
        round.stage = KnockStage::Aim;
        round.turn = 3;
        assert_eq!(place_of(&round, 2), Some(2));
    }

    #[test]
    fn who_won_is_said_of_one_or_of_several_together() {
        let ui = ui();
        let words = &ui.words.knock_off;
        let one = winners(&["Mochi".to_string()]);
        assert_eq!(one, fill(&words.wins, &[("name", "Mochi")]));
        let two = winners(&["Mochi".to_string(), "Dubu".to_string()]);
        assert_eq!(two, fill(&words.win_together, &[("names", &format!("Mochi{}Dubu", words.and))]));
        assert!(!two.contains('{') && two.contains("Mochi") && two.contains("Dubu"), "{two}");
        // Every word the game says is one the fonts draw, none empty, and
        // has in it what is filled in.
        let all: &[(&String, &[&str])] = &[
            (&words.score, &[]),
            (&words.turn, &["{turn}"]),
            (&words.last_turn, &[]),
            (&words.aim, &["{seconds}"]),
            (&words.ready, &["{seconds}"]),
            (&words.sliding, &[]),
            (&words.breaks, &[]),
            (&words.watching, &["{place}"]),
            (&words.went_in, &[]),
            (&words.go, &[]),
            (&words.how, &[]),
            (&words.last_turn_headline, &[]),
            (&words.wins, &["{name}"]),
            (&words.win_together, &["{names}"]),
            (&words.you_win, &[]),
            (&words.slid_in, &["{name}"]),
            (&words.knocked_in, &["{by}", "{name}"]),
            (&words.broke_under, &["{name}"]),
            (&words.left, &["{name}"]),
            (&words.won, &["{name}"]),
            (&words.won_together, &["{names}"]),
            (&words.out, &["{place}"]),
            (&words.out_line, &[]),
            (&words.watch, &[]),
            (&words.leave, &[]),
        ];
        for &(said, fills) in all {
            assert!(!said.trim().is_empty(), "an empty word");
            assert!(said.chars().all(roundtown_net::text::allowed), "{said:?} has a letter the fonts do not draw");
            for key in fills {
                assert!(said.contains(key), "{said:?} has no {key}");
            }
        }
        assert!(!words.and.is_empty() && words.and.chars().all(roundtown_net::text::allowed));
    }

    #[test]
    fn a_tick_is_drawn_and_its_corners_are_clear() {
        let image = tick();
        let data = image.data.as_ref().expect("pixels");
        let covered = data.chunks(4).filter(|pixel| pixel[3] > 128).count();
        assert!(covered > 300 && covered < 96 * 96 / 2, "{covered}");
        assert_eq!(data[3], 0, "the top left corner is clear");
    }
}
