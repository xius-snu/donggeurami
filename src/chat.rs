//! The chat box (since 2026-11-14; Hajun: "I think there should be a global
//! chat in the Bridge Wars game, with notifications that pop up based on
//! game events ... The chat box can go on the left side middle of the
//! screen, between the joystick and the name tag, kind of like minecraft
//! server chatbox style. There should be a text input box right below the
//! chat box with a send icon on the right. Pressing it lets you type a
//! custom message, which if you press "ENTER" on your phone OR press the
//! send icon sent to everyone and shows your name followed by the text you
//! typed. This chat box should also be in the main lobby ... The total
//! height of the chatbox shouldnt be too tall but scrollable, and should be
//! semi transparent so its not too obstrusive").
//!
//! **What it shows** ([`Log`]): every line said in the room you are in
//! since you came into it, a channel of the town or a game of Bridge Wars,
//! each as the name of whoever said it and what they said (`Tell::Said`,
//! [`hear`]); and, in a game of Bridge Wars, what the game says as it goes
//! (`arena::feed_chat`, which writes its lines here): who fell, who killed
//! whom with what, what a teammate bought for the team. A name is in its
//! team's colour in a game, and in the file's `colours.chat.name` in the
//! town. The newest line is at the bottom; the box shows four or so in the
//! town and three in a game, where there is less room for it ([`place`]),
//! and a finger dragged on it, or the mouse's wheel over it, scrolls back
//! through the last [`KEPT`]. Scrolled to the bottom it stays there as
//! lines come; scrolled up it stays where it was put. Going from the town
//! into a game, or back, or into another game, begins it afresh: what was
//! said in one room is not shown in another.
//!
//! **Saying something** ([`act`]): a press on the box under it (or on the
//! speech bubble among the town's buttons, or Enter or T on desktop) takes
//! every key and asks a phone for its keyboard (`signup::NameBox`, as a
//! name is typed), and the whole chat slides up out from under the
//! keyboard (`signup::LiftsForKeyboard`). While it is typed into you stand
//! still, as under a dialog (`hud::DialogUp`): on desktop the keys are the
//! line's, not your legs'. Enter, on either keyboard, or the dart at the
//! box's right end, says it: online it is asked of the server
//! (`Ask::Chat`), which tells everyone in the room, you among them;
//! offline it is written here, to nobody but you. Either way the box is
//! let go of. A press anywhere else on the screen lets go of it too, as it
//! goes down, and is whatever it landed on's as well, the stick's or a
//! button's ([`shut_under_a_press`]); what was typed is kept for next
//! time. A line is of what the game writes, as a name is
//! (`roundtown_net::text`: English, Korean and Vietnamese), and no longer
//! than `roundtown_net::chat::LONGEST`, a Hangul syllable being as long as
//! two letters.
//!
//! Where it is, how big, and how many lines it keeps are here; its colours
//! and words are the file's (`colours.chat`, `words.chat`).

use std::collections::VecDeque;

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use roundtown_net::{Ask, Tell, arena_team, chat as rules};

use crate::arena::Fighting;
use crate::hud::{self, ChatButton, DialogUp, Picture, Pictures, TakesPress};
use crate::island::Venue;
use crate::net::Online;
use crate::signup::{self, LiftsForKeyboard, NameBox, Signup};
use crate::ui::{UiChanged, ui};
use crate::{CloseUp, Player};

/// Sizes, as shares of the short side of the screen. Where the chat begins
/// down the screen in the town, well under your name and your balance,
/// which end 20 shares down, and how wide it is; how tall the box of lines
/// is there, four lines and the room round them (a line is 3 shares from
/// the next, 1.2 times its words: whole lines, so that none is cut along
/// the box's top), the gap under it, and how tall the box a line is typed
/// into is:
/// together they end 57 shares down, over the stick, whose disc reaches up
/// to 58 when it is held from as high as it may be. The words in each, the
/// room round them, and how round the corners are.
const TOP: f32 = 35.5;
const WIDTH: f32 = 50.0;
const LOG_HEIGHT: f32 = 13.1;
/// The same in a game of Bridge Wars, where what is top left, your name,
/// your health and the four things you hold, ends 38 shares down (411 of
/// 1,080 pixels, measured): the chat begins under those, and its box of
/// lines is a line shorter, three in sight, so as to end over the stick all
/// the same. Begun where the town's is, it lay over the last of the four
/// counts.
const ARENA_TOP: f32 = 39.0;
const ARENA_LOG_HEIGHT: f32 = 10.1;
const GAP: f32 = 0.6;
const INPUT_HEIGHT: f32 = 7.4;
const TEXT: f32 = 2.5;
const INPUT_TEXT: f32 = 2.8;
const PAD: f32 = 1.1;
const ROUNDED: f32 = 1.3;
/// How much of its button the dart that sends a line takes up, as a share.
const SEND_PICTURE: f32 = 50.0;
/// How many lines are kept to scroll back through.
pub(crate) const KEPT: usize = 60;
/// How far a notch of the mouse's wheel scrolls, in lines.
const WHEEL_LINES: f32 = 2.0;
/// How long the caret after what is typed is shown, and then not, in
/// seconds.
const BLINK_SECS: f32 = 0.5;
/// Where the box of lines is scrolled to while it keeps to its newest
/// line: further than it can ever go, which the layout brings back to as
/// far as it does go, however many lines come.
const STUCK: f32 = 1.0e6;

/// One line of the chat: its words, piece by piece, each in its colour.
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Line(pub(crate) Vec<(String, Color)>);

/// Every line there is to show, oldest first, each with a number of its
/// own, counting up, and the number the next will have.
#[derive(Resource, Default)]
pub(crate) struct Log {
    lines: VecDeque<(u64, Line)>,
    next: u64,
}

impl Log {
    /// One more line, the newest: the oldest goes once there are more than
    /// [`KEPT`].
    pub(crate) fn push(&mut self, line: Line) {
        if line.0.is_empty() {
            return;
        }
        self.lines.push_back((self.next, line));
        self.next += 1;
        while self.lines.len() > KEPT {
            self.lines.pop_front();
        }
    }

    /// Begins afresh, with nothing said.
    fn clear(&mut self) {
        self.lines.clear();
    }

    /// The number of the oldest line there is, or the next one's with
    /// none.
    fn first(&self) -> u64 {
        self.lines.front().map_or(self.next, |(id, _)| *id)
    }
}

/// `template` as a line: its own words in `ink`, and each `{key}` in it
/// filled in with that key's words, in that key's own colour (`fills`:
/// the key, its words, their colour). How a name stands in its team's
/// colour in the middle of a line.
pub(crate) fn spans(template: &str, ink: Color, fills: &[(&str, &str, Color)]) -> Line {
    fn add(parts: &mut Vec<(String, Color)>, words: &str, colour: Color) {
        if words.is_empty() {
            return;
        }
        match parts.last_mut() {
            Some((last, same)) if *same == colour => last.push_str(words),
            _ => parts.push((words.to_string(), colour)),
        }
    }
    let mut parts = Vec::new();
    let mut rest = template;
    loop {
        // The first of the keys to come in what is left.
        let next = fills
            .iter()
            .filter_map(|&(key, words, colour)| {
                let token = format!("{{{key}}}");
                rest.find(&token).map(|at| (at, token.len(), words, colour))
            })
            .min_by_key(|found| found.0);
        match next {
            Some((at, length, words, colour)) => {
                add(&mut parts, &rest[..at], ink);
                add(&mut parts, words, colour);
                rest = &rest[at + length..];
            }
            None => {
                add(&mut parts, rest, ink);
                break;
            }
        }
    }
    Line(parts)
}

/// What someone called `name` said, as a line, their name in `colour`.
fn said(name: &str, colour: Color, text: &str) -> Line {
    let file = ui();
    let ink = file.colours.chat.text;
    spans(&file.words.chat.said, ink, &[("name", name, colour), ("text", text, ink)])
}

/// The chat, the box of lines and the box a line is typed into.
#[derive(Component)]
struct ChatRoot;
/// The box of lines.
#[derive(Component)]
struct LogBox;
/// One line in it, by its number in the [`Log`].
#[derive(Component)]
struct LineOf(u64);
/// The box a line is typed into, what is written in it, the caret after
/// that, and the dart at its right end with its picture.
#[derive(Component)]
struct InputBox;
#[derive(Component)]
struct InputText;
#[derive(Component)]
struct Caret;
#[derive(Component)]
struct SendButton;
#[derive(Component)]
struct SendPicture;

/// Whether a line is being typed, and what was typed and not said the
/// last time the box was let go of, to go on from.
#[derive(Resource, Default)]
struct Typing {
    on: bool,
    draft: String,
    /// Seconds the caret has blinked for.
    blink: f32,
}

/// What of the chat was pressed since [`act`] last looked: the box a line
/// is typed into (or the town's speech bubble), and the dart.
#[derive(Resource, Default)]
struct Pressed {
    input: bool,
    send: bool,
}

/// Which room the lines shown were said in, and which line is the next to
/// be drawn.
#[derive(Resource, Default)]
struct Drawn {
    room: Option<(Venue, u32)>,
    next: u64,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Log>()
        .init_resource::<Typing>()
        .init_resource::<Pressed>()
        .init_resource::<Drawn>()
        .add_observer(press)
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            (
                // Before you are moved by the press that let go of it.
                shut_under_a_press.before(crate::read_player_input),
                // After what is typed has been taken into the box, so that
                // the key that opens it is not its first letter.
                act.after(signup::Typed).after(shut_under_a_press),
                (hear, begin_afresh, fill).chain(),
                show,
                write_input.after(act),
                wheel,
            ),
        );
}

/// Puts the chat up, hidden until you are somewhere it is shown ([`show`]):
/// the box of lines, see-through, and under it the box a line is typed
/// into, with the dart at its right end.
fn spawn(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut pictures: ResMut<Pictures>) {
    let file = ui();
    let colours = file.colours.chat;
    let dart = pictures.of(&mut images, Picture::Send);
    commands
        .spawn((
            ChatRoot,
            Name::new("Chat"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::VMin(hud::EDGE),
                top: Val::VMin(TOP),
                width: Val::VMin(WIDTH),
                flex_direction: FlexDirection::Column,
                row_gap: Val::VMin(GAP),
                ..default()
            },
            // Slid up out from under a phone's keyboard while a line is
            // typed.
            UiTransform::default(),
            LiftsForKeyboard::default(),
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                LogBox,
                TakesPress,
                Node {
                    height: Val::VMin(LOG_HEIGHT),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    // Cut where the lines' own room ends, not at the box's
                    // edge: the feet of the line scrolled off the top
                    // showed in the room round the lines.
                    overflow_clip_margin: OverflowClipMargin::content_box(),
                    padding: UiRect::axes(Val::VMin(PAD), Val::VMin(PAD * 0.5)),
                    border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
                    ..default()
                },
                BackgroundColor(colours.back),
                ScrollPosition(Vec2::new(0.0, STUCK)),
            ))
            .observe(drag_log);
            root.spawn((
                InputBox,
                TakesPress,
                Button,
                Node {
                    height: Val::VMin(INPUT_HEIGHT),
                    align_items: AlignItems::Center,
                    padding: UiRect::left(Val::VMin(PAD * 1.4)),
                    border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
                    ..default()
                },
                BackgroundColor(colours.field),
            ))
            .with_children(|input| {
                // What is typed, on one line: short, it begins at the
                // left; too long for the box, its end is what shows, the
                // beginning gone off the left under the box's edge.
                input
                    .spawn((
                        Node {
                            flex_grow: 1.0,
                            min_width: Val::ZERO,
                            height: Val::Percent(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexEnd,
                            overflow: Overflow::clip_x(),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|line| {
                        line.spawn((
                            InputText,
                            Text::new(file.words.chat.hint.clone()),
                            crate::font::sized(INPUT_TEXT),
                            TextColor(colours.hint),
                            TextLayout::new(Justify::Left, LineBreak::NoWrap),
                            Node {
                                min_width: Val::Percent(100.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_child((
                            Caret,
                            TextSpan::new("|"),
                            crate::font::sized(INPUT_TEXT),
                            TextColor(colours.typed.with_alpha(0.0)),
                        ));
                    });
                input.spawn((
                    SendButton,
                    TakesPress,
                    Button,
                    Node {
                        width: Val::VMin(INPUT_HEIGHT),
                        height: Val::Percent(100.0),
                        flex_shrink: 0.0,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![(
                        SendPicture,
                        ImageNode::new(dart).with_color(colours.send_off),
                        Node {
                            width: Val::Percent(SEND_PICTURE),
                            height: Val::Percent(SEND_PICTURE),
                            ..default()
                        },
                        Pickable::IGNORE,
                    )],
                ));
            });
        });
}

/// Whether the chat is shown `here`: in the town and in a game of Bridge
/// Wars, its waiting room too.
fn shown_on(here: Venue) -> bool {
    matches!(here, Venue::Town | Venue::Arena)
}

/// Where the chat begins down the screen `here`, and how tall its box of
/// lines is there, in shares of the short side: under what each place has
/// top left, and over the stick in both.
fn place(here: Venue) -> (f32, f32) {
    if here == Venue::Arena { (ARENA_TOP, ARENA_LOG_HEIGHT) } else { (TOP, LOG_HEIGHT) }
}

/// Shows the chat in the town and in Bridge Wars, where each has room for
/// it ([`place`]), and puts it away anywhere else, under the sign-up's
/// pages, and while the camera is on someone you talk to, as the stick is.
#[allow(clippy::type_complexity)]
fn show(
    me: Query<&Venue, With<Player>>,
    signup: Res<Signup>,
    close_up: Res<CloseUp>,
    mut roots: Query<(&mut Visibility, &mut Node), (With<ChatRoot>, Without<LogBox>)>,
    mut boxes: Query<&mut Node, (With<LogBox>, Without<ChatRoot>)>,
) {
    let venue = me.single().ok().copied();
    let here = venue.is_some_and(shown_on);
    let wanted = if here && !signup.up() && close_up.on.is_none() { Visibility::Inherited } else { Visibility::Hidden };
    let (top, height) = place(venue.unwrap_or(Venue::Town));
    for (mut visibility, mut node) in &mut roots {
        visibility.set_if_neq(wanted);
        // (Written only as it changes: a node written is the screen laid
        // out again.)
        if here && node.top != Val::VMin(top) {
            node.top = Val::VMin(top);
        }
    }
    for mut node in &mut boxes {
        if here && node.height != Val::VMin(height) {
            node.height = Val::VMin(height);
        }
    }
}

/// What everyone in your room says ([`Tell::Said`]), you among them: a
/// line each, the name in its team's colour in a game of Bridge Wars, and
/// in the file's colour for names anywhere else.
fn hear(mut tells: MessageReader<Tell>, fighting: Res<Fighting>, mut log: ResMut<Log>) {
    for tell in tells.read() {
        if let Tell::Said { who, name, text } = tell {
            log.push(said(name, name_colour(&fighting, *who), text));
        }
    }
}

/// The colour the name of whoever has the id `who` is written in: their
/// team's, in your game of Bridge Wars, by their seat; anyone else's, the
/// file's colour for a name.
fn name_colour(fighting: &Fighting, who: u64) -> Color {
    let colours = ui().colours;
    match fighting.members.iter().position(|member| *member == who && who != 0) {
        Some(seat) => colours.arena.team(arena_team(seat)),
        None => colours.chat.name,
    }
}

/// Begins the chat afresh as you come into another room: from the town
/// into a game, back, or into another game. What was said in one is not
/// shown in another.
fn begin_afresh(
    me: Query<&Venue, With<Player>>,
    fighting: Res<Fighting>,
    mut drawn: ResMut<Drawn>,
    mut log: ResMut<Log>,
) {
    let Ok(&here) = me.single() else {
        return;
    };
    // A game by its number; the town, and anywhere else, as one room each.
    let room = (here, if here == Venue::Arena { fighting.game } else { 0 });
    if drawn.room == Some(room) {
        return;
    }
    if drawn.room.is_some() {
        log.clear();
    }
    drawn.room = Some(room);
}

/// Draws the lines there are in the box: each new one under the last, and
/// those the log has let go of taken down. All of them again as the
/// file's colours change on desktop.
fn fill(
    log: Res<Log>,
    changed: Res<UiChanged>,
    mut drawn: ResMut<Drawn>,
    boxes: Query<Entity, With<LogBox>>,
    lines: Query<(Entity, &LineOf)>,
    mut backs: Query<&mut BackgroundColor, With<LogBox>>,
    mut commands: Commands,
) {
    let restyle = changed.is_changed() && !changed.is_added();
    if !log.is_changed() && !restyle {
        return;
    }
    let Ok(log_box) = boxes.single() else {
        return;
    };
    let file = ui();
    let shadow = file.colours.text_shadow;
    if restyle {
        for mut back in &mut backs {
            back.set_if_neq(BackgroundColor(file.colours.chat.back));
        }
    }
    let first = log.first();
    for (entity, of) in &lines {
        if restyle || of.0 < first {
            commands.entity(entity).despawn();
        }
    }
    let from = if restyle { first } else { drawn.next.max(first) };
    for (id, line) in log.lines.iter().filter(|(id, _)| *id >= from) {
        let Some(((words, colour), rest)) = line.0.split_first() else {
            continue;
        };
        commands
            .spawn((
                LineOf(*id),
                ChildOf(log_box),
                Text::new(words.clone()),
                crate::font::sized(TEXT),
                TextColor(*colour),
                TextShadow {
                    offset: Vec2::new(0.0, 2.0),
                    color: shadow,
                },
                TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|line| {
                for (words, colour) in rest {
                    line.spawn((TextSpan::new(words.clone()), crate::font::sized(TEXT), TextColor(*colour)));
                }
            });
    }
    drawn.next = log.next;
}

/// Scrolls the box of lines `by` logical pixels down its lines, no further
/// than they go either way; back at its newest line, it keeps to it as
/// more come ([`STUCK`]).
fn scroll_by(node: &ComputedNode, scroll: &mut ScrollPosition, by: f32) {
    let scale = node.inverse_scale_factor();
    let most = ((node.content_size().y - node.size().y) * scale).max(0.0);
    // Where it is drawn now, which is as far as it goes while it keeps to
    // its newest line.
    let now = (node.scroll_position.y * scale).min(most);
    let to = (now + by).clamp(0.0, most);
    let next = if to >= most - 0.5 { STUCK } else { to };
    if scroll.y != next {
        scroll.y = next;
    }
}

/// Scrolls the lines with a finger, or the mouse held down, dragged over
/// them.
fn drag_log(drag: On<Pointer<Drag>>, mut boxes: Query<(&ComputedNode, &mut ScrollPosition), With<LogBox>>) {
    if let Ok((node, mut scroll)) = boxes.get_mut(drag.entity) {
        scroll_by(node, &mut scroll, -drag.delta.y);
    }
}

/// Scrolls the lines with the mouse's wheel while the cursor is over them
/// (the camera does not zoom by it there, `read_mouse_zoom`).
fn wheel(
    mut wheels: MessageReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut boxes: Query<(&ComputedNode, &UiGlobalTransform, &InheritedVisibility, &mut ScrollPosition), With<LogBox>>,
) {
    let turned: f32 = wheels
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
        })
        .sum();
    if turned == 0.0 {
        return;
    }
    let (Ok(window), Ok((node, place, shown, mut scroll))) = (windows.single(), boxes.single_mut()) else {
        return;
    };
    let over = window
        .cursor_position()
        .is_some_and(|cursor| shown.get() && node.contains_point(*place, cursor * window.scale_factor()));
    if over {
        let line = window.width().min(window.height()) * TEXT * 1.2 / 100.0;
        scroll_by(node, &mut scroll, -turned * WHEEL_LINES * line);
    }
}

/// A press, as it goes down, on the box a line is typed into, on the dart
/// that sends it, or on the speech bubble among the town's buttons: for
/// [`act`] to act on.
fn press(
    press: On<Pointer<Press>>,
    inputs: Query<(), Or<(With<InputBox>, With<ChatButton>)>>,
    sends: Query<(), With<SendButton>>,
    mut pressed: ResMut<Pressed>,
) {
    if press.button != PointerButton::Primary || press.original_event_target() != press.entity {
        return;
    }
    if sends.contains(press.entity) {
        pressed.send = true;
    } else if inputs.contains(press.entity) {
        pressed.input = true;
    }
}

/// Lets go of the box, keeping what is typed in it for next time.
fn let_go(typing: &mut Typing, name_box: &mut NameBox, up: &mut DialogUp) {
    typing.draft = name_box.shown();
    typing.on = false;
    name_box.close();
    if up.0 {
        up.0 = false;
    }
}

/// A finger put down anywhere but on the chat, while a line is being
/// typed, lets go of the box in the frame it lands, before you are moved
/// by it: so that a thumb put on the stick walks you, and one on the jump
/// button jumps, with that same press, as the friends' panel is put away
/// (`friends::shut_under_a_press`). On something that takes a press of its
/// own, a button, that has it as well.
fn shut_under_a_press(
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    roots: Query<(&ComputedNode, &UiGlobalTransform), With<ChatRoot>>,
    mut typing: ResMut<Typing>,
    mut name_box: ResMut<NameBox>,
    mut up: ResMut<DialogUp>,
) {
    if !typing.on {
        return;
    }
    let (Ok(window), Ok((node, place))) = (windows.single(), roots.single()) else {
        return;
    };
    let fingers = touches.iter_just_pressed().map(|touch| touch.position());
    let cursor = if mouse.just_pressed(MouseButton::Left) { window.cursor_position() } else { None };
    if fingers.chain(cursor).any(|at| !node.contains_point(*place, at * window.scale_factor())) {
        let_go(&mut typing, &mut name_box, &mut up);
    }
}

/// Takes the box up, says what is typed, or lets go of it: by a press on
/// it or on the dart, by Enter on either keyboard, by T or Enter on
/// desktop to begin, and Escape to leave off.
#[allow(clippy::too_many_arguments)]
fn act(
    mut pressed: ResMut<Pressed>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut typing: ResMut<Typing>,
    mut name_box: ResMut<NameBox>,
    mut up: ResMut<DialogUp>,
    (online, fighting, close_up, signup): (Res<Online>, Res<Fighting>, Res<CloseUp>, Res<Signup>),
    me: Query<(&Name, &Venue), With<Player>>,
    mut log: ResMut<Log>,
    mut asks: MessageWriter<Ask>,
) {
    let (input, send) = (std::mem::take(&mut pressed.input), std::mem::take(&mut pressed.send));
    let enter = keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    if typing.on {
        // Whatever else took the box, or the screen, has it.
        if !name_box.up() {
            typing.on = false;
            return;
        }
        if name_box.entered() || enter || send {
            if let Some(text) = rules::line(&name_box.shown()) {
                if online.is() {
                    asks.write(Ask::Chat(text));
                } else if let Ok((name, _)) = me.single() {
                    // Offline there is nobody to tell: it is written here.
                    let mine = online.me().map_or(0, |me| me.id);
                    let colour = match fighting.seat.filter(|_| fighting.members.len() > 1 || mine != 0) {
                        Some(seat) => ui().colours.arena.team(arena_team(usize::from(seat))),
                        None => ui().colours.chat.name,
                    };
                    log.push(said(name.as_str(), colour, &text));
                }
            }
            let_go(&mut typing, &mut name_box, &mut up);
            typing.draft.clear();
        } else if keyboard.just_pressed(KeyCode::Escape) {
            let_go(&mut typing, &mut name_box, &mut up);
        } else if input {
            name_box.ask_for_the_keyboard_again();
        }
        return;
    }
    // Not with anything else up, nor where the chat is not shown.
    let here = me.single().is_ok_and(|(_, &venue)| shown_on(venue));
    let by_key = enter || keyboard.just_pressed(KeyCode::KeyT);
    if (input || send || by_key) && here && !up.0 && !name_box.up() && !signup.up() && !close_up.busy() {
        name_box.open_for(&typing.draft, rules::LONGEST);
        up.0 = true;
        typing.on = true;
        typing.blink = 0.0;
    }
}

/// Keeps the box showing what is typed, with the caret blinking after it;
/// or, let go of, what was left in it, or the words that say what it is
/// for. Lit while it is typed into, and the dart with it once there is
/// something to say.
#[allow(clippy::type_complexity)]
fn write_input(
    time: Res<Time<Real>>,
    mut typing: ResMut<Typing>,
    name_box: Res<NameBox>,
    mut texts: Query<(&mut Text, &mut TextColor), (With<InputText>, Without<Caret>)>,
    mut carets: Query<&mut TextColor, (With<Caret>, Without<InputText>)>,
    mut boxes: Query<&mut BackgroundColor, With<InputBox>>,
    mut darts: Query<&mut ImageNode, With<SendPicture>>,
) {
    let file = ui();
    let colours = file.colours.chat;
    // (What is typed as it reads, a letter put together from its pieces,
    // with whatever a desktop's input method is composing after it.)
    let typed = name_box.shown();
    let (words, ink, caret) = if typing.on {
        typing.blink += time.delta_secs();
        let lit = (typing.blink / BLINK_SECS) as u32 % 2 == 0;
        (typed.as_str(), colours.typed, lit)
    } else if typing.draft.is_empty() {
        (file.words.chat.hint.as_str(), colours.hint, false)
    } else {
        (typing.draft.as_str(), colours.hint, false)
    };
    for (mut text, mut colour) in &mut texts {
        if text.0 != words {
            text.0 = words.to_string();
        }
        colour.set_if_neq(TextColor(ink));
    }
    // The caret keeps its place whether it shows or not, so that what is
    // typed does not shift as it blinks.
    let caret = colours.typed.with_alpha(if caret { colours.typed.alpha() } else { 0.0 });
    for mut colour in &mut carets {
        colour.set_if_neq(TextColor(caret));
    }
    for mut fill in &mut boxes {
        fill.set_if_neq(BackgroundColor(if typing.on { colours.field_lit } else { colours.field }));
    }
    let ready = typing.on && rules::line(&typed).is_some();
    for mut dart in &mut darts {
        let tint = if ready { colours.send } else { colours.send_off };
        if dart.color != tint {
            dart.color = tint;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_is_its_words_with_each_name_in_its_own_colour() {
        let (ink, red, blue) = (Color::WHITE, Color::srgb(1.0, 0.0, 0.0), Color::srgb(0.0, 0.0, 1.0));
        // A name, then words.
        let line = spans("{name} fell off the world.", ink, &[("name", "Bob", red)]);
        assert_eq!(line.0, vec![("Bob".into(), red), (" fell off the world.".into(), ink)]);
        // Two names, in whichever order the words have them, and the
        // words between and after.
        let fills = [("name", "Hannah", blue), ("killer", "Bob", red), ("weapon", "a Basic Sword", ink)];
        let line = spans("{killer} killed {name} with {weapon}.", ink, &fills);
        assert_eq!(
            line.0,
            vec![
                ("Bob".into(), red),
                (" killed ".into(), ink),
                ("Hannah".into(), blue),
                (" with a Basic Sword.".into(), ink),
            ]
        );
        // A key the words have not got is left out; a key they have twice
        // is filled in twice; words with no key at all are themselves.
        assert_eq!(spans("{name} and {name}", ink, &fills).0.len(), 3);
        assert_eq!(spans("Nothing.", ink, &fills).0, vec![("Nothing.".into(), ink)]);
        assert!(spans("", ink, &fills).0.is_empty());
        // What someone said: their name, a colon, and their words, which
        // are theirs as typed, braces and all.
        let line = said("Bob", red, "gg {name}");
        assert_eq!(line.0, vec![("Bob".into(), red), (": gg {name}".into(), ui().colours.chat.text)]);
    }

    #[test]
    fn the_log_keeps_the_newest_lines_and_numbers_them_as_they_come() {
        let mut log = Log::default();
        assert_eq!(log.first(), 0);
        for n in 0..KEPT + 5 {
            log.push(Line(vec![(format!("line {n}"), Color::WHITE)]));
        }
        assert_eq!(log.lines.len(), KEPT);
        assert_eq!((log.first(), log.next), (5, KEPT as u64 + 5));
        assert_eq!(log.lines.back().unwrap().1.0[0].0, format!("line {}", KEPT + 4));
        // Nothing said is no line.
        log.push(Line(Vec::new()));
        assert_eq!(log.next, KEPT as u64 + 5);
        // Begun afresh, the numbers go on from where they were, so that a
        // line of the new room is never taken for one of the old.
        log.clear();
        assert_eq!((log.lines.len(), log.first()), (0, KEPT as u64 + 5));
        log.push(Line(vec![("hello".into(), Color::WHITE)]));
        assert_eq!(log.first(), KEPT as u64 + 5);
    }

    #[test]
    fn the_chat_lies_between_what_is_top_left_and_the_stick() {
        // Over the stick's disc, which reaches up to 76.5 less its radius,
        // 12.5, less as far as its middle may be set from its own, half
        // its radius, on a screen lying on its side (where the height is
        // the short side).
        let stick_top = 100.0 - crate::STICK_BOTTOM_VH - crate::STICK_SIZE_VMIN * (1.0 + crate::STICK_ORIGIN_REACH * 0.5);
        // And under what is top left: your name and your balance in the
        // town, which end 20 shares down; your name, your health and the
        // four things you hold in Bridge Wars, which end 38 down (411 of
        // 1,080 pixels in the desktop build).
        for (here, top_left_ends) in [(Venue::Town, 20.0), (Venue::Arena, 38.1)] {
            let (top, height) = place(here);
            assert!(top > top_left_ends + 0.5, "{here:?}: {top}");
            let ends = top + height + GAP + INPUT_HEIGHT;
            assert!(ends < stick_top, "{here:?}: {ends} against {stick_top}");
            // Not too tall, and its words small beside the buttons': three
            // lines in sight at the least, and five at the most; and whole
            // lines, none cut along the top of the box.
            let lines = (height - PAD) / (TEXT * 1.2);
            assert!((2.95..=5.05).contains(&lines), "{here:?}: {lines} lines");
            assert!((lines - lines.round()).abs() < 0.02, "{here:?}: {lines} lines");
        }
        // The chat is in the town and in Bridge Wars, and nowhere else.
        assert!(shown_on(Venue::Town) && shown_on(Venue::Arena));
        assert!(!shown_on(Venue::Home) && !shown_on(Venue::Lobby) && !shown_on(Venue::Plot(3)));
    }
}
