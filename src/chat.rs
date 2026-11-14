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
//! town. Going from the town into a game, or back, or into another game,
//! begins it afresh: what was said in one room is not shown in another.
//!
//! **With nothing being typed it is its words alone** (since later on
//! 2026-11-14; Hajun, having played it: "I don't want the permanent dark.
//! Chat box for both the global chat and the Bridgewaters game chat, but
//! rather just the text and the. Box should only appear when the when you
//! are typing a message. And when. It's idle, um, the older line should be
//! drawn fainter because it's partway through fading"). No box is drawn
//! behind them, and they take no press: a finger put down on them looks
//! round, as anywhere on the world. **The newest line is always on the same
//! line**, the lowest ("instead of being like the first text starts at the
//! top of the rectangle and makes its way downward, no, it should be like
//! every new text starts at the same line. Even the first message stays at
//! the same area, and it goes up and fades as new messages come"): the
//! first thing said is written there, and each line after it moves what
//! was said before up one. **A line fades from the moment it is said**
//! ([`faded`]), slowly at first and then faster, and is gone [`LINE_SECS`]
//! on: so of two lines the older is always the fainter. Four are in sight
//! in the town at the most and three in a game, where there is less room
//! ([`place`]).
//!
//! **Saying something** ([`act`]): a press on the small keyboard over the
//! speech bubble, bottom right (`hud::ChatButton`, in the town and in
//! Bridge Wars alike; "the input text box and the send button, they should
//! not appear until you press the the chat icon on the right bottom right
//! corner"), or Enter or T on desktop, takes every key and asks a phone
//! for its keyboard (`signup::NameBox`, as a name is typed). **Then the
//! chat is as it was to begin with**: the dark box behind the lines, every
//! line that is kept in full strength, to be scrolled back through with a
//! finger or the mouse's wheel (the last [`KEPT`]), and under it the box
//! the line is typed into, with the dart at its right end; and the whole
//! of it slides up out from under the keyboard
//! (`signup::LiftsForKeyboard`). While it is typed into you stand still,
//! as under a dialog (`hud::DialogUp`): on desktop the keys are the
//! line's, not your legs'. Enter, on either keyboard, or the dart, says it
//! ([`Say`]): online it is asked of the server (`Ask::Chat`), which tells
//! everyone in the room, you among them; offline it is written here, to
//! nobody but you. Either way the box is let go of, and the chat is its
//! words alone again. A press anywhere else on the screen lets go of it
//! too, as it goes down, and is whatever it landed on's as well, the
//! stick's or a button's ([`shut_under_a_press`]); on the keyboard's own
//! button it only lets go. What was typed is kept for next time. A line is
//! of what the game writes, as a name is (`roundtown_net::text`: English,
//! Korean and Vietnamese), and no longer than
//! `roundtown_net::chat::LONGEST`, a Hangul syllable being as long as two
//! letters.
//!
//! The big speech bubble says one of a few things at a press, with nothing
//! typed (`quick`).
//!
//! Where it is, how big, how many lines it keeps and how long one lasts
//! are here; its colours and words are the file's (`colours.chat`,
//! `words.chat`). It keeps clear of the camera's hole (`safe`).

use std::collections::VecDeque;

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use roundtown_net::{Ask, Tell, arena_team, chat as rules};

use crate::arena::Fighting;
use crate::hud::{self, ChatButton, DialogUp, Picture, Pictures, TakesPress};
use crate::island::Venue;
use crate::net::Online;
use crate::safe::KeepsClear;
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
/// How long a line is there to be read with nothing being typed, in
/// seconds, and how its fading is bent ([`faded`]): the higher, the longer
/// it stays nearly whole before it goes. Twelve seconds and this bend
/// leave a line three quarters there half way through, and half there
/// with a quarter of its time to go. (Minecraft's stay whole for ten
/// seconds and are gone in the next.) Both are mine.
const LINE_SECS: f32 = 12.0;
const FADE_BEND: f32 = 2.2;
/// How many steps a line fades in: it is drawn again only as it comes to
/// the next.
const FADE_STEPS: u8 = 40;

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

/// How much of itself a line said `age` seconds ago is drawn with, with
/// nothing being typed, in steps of [`FADE_STEPS`], which is all of it:
/// less from the moment it is said, slowly at first and then faster, and
/// nothing [`LINE_SECS`] on. So of two lines the older is the fainter,
/// "because it's partway through fading".
fn faded(age: f32) -> u8 {
    let through = (age / LINE_SECS).clamp(0.0, 1.0);
    ((1.0 - through.powf(FADE_BEND)) * f32::from(FADE_STEPS)).round() as u8
}

/// `colour`, as much of it as `step` of [`FADE_STEPS`] leaves.
fn thinned(colour: Color, step: u8) -> Color {
    colour.with_alpha(colour.alpha() * f32::from(step) / f32::from(FADE_STEPS))
}

/// Something to say to the room you are in, as it was typed or chosen:
/// the box's line, and what the big speech bubble offers (`quick`). Said
/// by [`say`], if it is a line at all.
#[derive(Message, Clone, Debug)]
pub(crate) struct Say(pub(crate) String);

/// What someone called `name` said, as a line, their name in `colour`.
fn said(name: &str, colour: Color, text: &str) -> Line {
    let file = ui();
    let ink = file.colours.chat.text;
    spans(&file.words.chat.said, ink, &[("name", name, colour), ("text", text, ink)])
}

/// The chat, the box of lines and the box a line is typed into.
#[derive(Component)]
struct ChatRoot;
/// The box of lines: only its lines are ever seen of it.
#[derive(Component)]
struct LogBox;
/// The dark box behind the lines, there while a line is typed: what a
/// finger is dragged on to scroll them.
#[derive(Component)]
struct Backdrop;
/// One line in it, by its number in the [`Log`]; when it was said, in the
/// app's own seconds; and how much of itself it is drawn with now, in
/// steps of [`FADE_STEPS`].
#[derive(Component)]
struct LineOf(u64);
#[derive(Component)]
struct SaidAt(f32);
#[derive(Component)]
struct Step(u8);
/// The colour a piece of a line is in at full strength: what it fades
/// from.
#[derive(Component)]
struct Ink(Color);
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
    /// A press somewhere else has just let go of the box
    /// ([`shut_under_a_press`]): if it was on the chat's own button, it
    /// does not take the box up again ([`act`]).
    shut: bool,
}

/// What of the chat was pressed since [`act`] last looked: the box a line
/// is typed into, the keyboard among the buttons, and the dart.
#[derive(Resource, Default)]
struct Pressed {
    input: bool,
    button: bool,
    send: bool,
}

/// Which room the lines shown were said in, which line is the next to be
/// drawn, and when each line there is was first drawn, oldest first, by
/// its number: what it fades from, kept here so that a line drawn again
/// (as the file's colours change on desktop) is as old as it was.
#[derive(Resource, Default)]
struct Drawn {
    room: Option<(Venue, u32)>,
    next: u64,
    said: VecDeque<(u64, f32)>,
}

impl Drawn {
    /// When the line numbered `id` was said, if it has been drawn.
    fn said_at(&self, id: u64) -> Option<f32> {
        let first = self.said.front()?.0;
        let index = usize::try_from(id.checked_sub(first)?).ok()?;
        self.said.get(index).filter(|(numbered, _)| *numbered == id).map(|(_, at)| *at)
    }
}

/// The box a line is typed into being let go of by a press somewhere else
/// ([`shut_under_a_press`]): whatever that press is also for, and cannot
/// be done under a dialog, comes after this (`quick`).
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LetGo;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Log>()
        .init_resource::<Typing>()
        .init_resource::<Pressed>()
        .init_resource::<Drawn>()
        .add_message::<Say>()
        .add_observer(press)
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            (
                // Before you are moved by the press that let go of it.
                shut_under_a_press.before(crate::read_player_input).in_set(LetGo),
                // After what is typed has been taken into the box, so that
                // the key that opens it is not its first letter.
                act.after(signup::Typed).after(shut_under_a_press),
                (say.after(act), hear, begin_afresh, fill, fade).chain(),
                show,
                (write_input, dress).after(act),
                wheel,
            ),
        );
}

/// Puts the chat up, hidden until you are somewhere it is shown ([`show`]):
/// the box of lines, of which only the lines are seen; behind it the dark
/// box, and under it the box a line is typed into, with the dart at its
/// right end, both put away until a line is typed ([`dress`]).
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
            // Moved in from the left of the screen where the camera's hole
            // is there.
            KeepsClear,
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            // The dark box, behind the lines and as big as their box: it
            // is laid out by itself, over the same room.
            root.spawn((
                Backdrop,
                TakesPress,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    right: Val::ZERO,
                    top: Val::ZERO,
                    height: Val::VMin(LOG_HEIGHT),
                    border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
                    ..default()
                },
                BackgroundColor(colours.back),
                Visibility::Hidden,
            ))
            .observe(drag_log);
            root.spawn((
                LogBox,
                Node {
                    height: Val::VMin(LOG_HEIGHT),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    // Cut where the lines' own room ends, not at the box's
                    // edge: the feet of the line scrolled off the top
                    // showed in the room round the lines.
                    overflow_clip_margin: OverflowClipMargin::content_box(),
                    padding: UiRect::axes(Val::VMin(PAD), Val::VMin(PAD * 0.5)),
                    ..default()
                },
                ScrollPosition(Vec2::new(0.0, STUCK)),
                Pickable::IGNORE,
            ))
            // Whatever room the lines leave is over them, so that the
            // newest is always on the lowest line, the first thing said
            // too; with more lines than there is room for it is nothing,
            // and they scroll.
            .with_child((
                Node {
                    flex_grow: 1.0,
                    min_height: Val::ZERO,
                    ..default()
                },
                Pickable::IGNORE,
            ));
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
                Visibility::Hidden,
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
pub(crate) fn shown_on(here: Venue) -> bool {
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
    mut roots: Query<(&mut Visibility, &mut Node), (With<ChatRoot>, Without<LogBox>, Without<Backdrop>)>,
    mut boxes: Query<&mut Node, (Or<(With<LogBox>, With<Backdrop>)>, Without<ChatRoot>)>,
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
/// file's colours change on desktop. Each as much there as it is by now
/// ([`faded`]), or wholly while a line is typed.
#[allow(clippy::too_many_arguments)]
fn fill(
    log: Res<Log>,
    changed: Res<UiChanged>,
    time: Res<Time<Real>>,
    typing: Res<Typing>,
    mut drawn: ResMut<Drawn>,
    boxes: Query<Entity, With<LogBox>>,
    lines: Query<(Entity, &LineOf)>,
    mut backs: Query<&mut BackgroundColor, With<Backdrop>>,
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
    let shadow = file.colours.chat.shadow;
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
    while drawn.said.front().is_some_and(|(id, _)| *id < first) {
        drawn.said.pop_front();
    }
    let now = time.elapsed_secs();
    let from = if restyle { first } else { drawn.next.max(first) };
    for (id, line) in log.lines.iter().filter(|(id, _)| *id >= from) {
        let Some(((words, colour), rest)) = line.0.split_first() else {
            continue;
        };
        let said = match drawn.said_at(*id) {
            Some(said) => said,
            None => {
                drawn.said.push_back((*id, now));
                now
            }
        };
        let step = if typing.on { FADE_STEPS } else { faded(now - said) };
        commands
            .spawn((
                (LineOf(*id), SaidAt(said), Step(step), Ink(*colour)),
                ChildOf(log_box),
                Text::new(words.clone()),
                crate::font::sized(TEXT),
                TextColor(thinned(*colour, step)),
                TextShadow {
                    offset: Vec2::new(0.0, 2.0),
                    color: thinned(shadow, step),
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
                    line.spawn((
                        TextSpan::new(words.clone()),
                        crate::font::sized(TEXT),
                        Ink(*colour),
                        TextColor(thinned(*colour, step)),
                    ));
                }
            });
    }
    drawn.next = log.next;
}

/// Fades each line as it grows old ([`faded`]), and draws them all in
/// full strength while a line is typed: its words, piece by piece, and
/// the shadow under them, written only as a line comes to its next step.
#[allow(clippy::type_complexity)]
fn fade(
    time: Res<Time<Real>>,
    typing: Res<Typing>,
    mut lines: Query<(&SaidAt, &mut Step, &Ink, &mut TextColor, &mut TextShadow, Option<&Children>), Without<TextSpan>>,
    mut pieces: Query<(&Ink, &mut TextColor), With<TextSpan>>,
) {
    let now = time.elapsed_secs();
    let shadow = ui().colours.chat.shadow;
    for (said, mut step, ink, mut colour, mut shade, children) in &mut lines {
        let wanted = if typing.on { FADE_STEPS } else { faded(now - said.0) };
        if step.0 == wanted {
            continue;
        }
        step.0 = wanted;
        colour.0 = thinned(ink.0, wanted);
        shade.color = thinned(shadow, wanted);
        for child in children.into_iter().flatten() {
            if let Ok((ink, mut colour)) = pieces.get_mut(*child) {
                colour.0 = thinned(ink.0, wanted);
            }
        }
    }
}

/// Dresses the chat for a line being typed, and undresses it after: the
/// dark box behind the lines and the box the line is typed into are there
/// only then. Let go of, the lines are back at the newest, however far
/// back they were scrolled.
fn dress(
    typing: Res<Typing>,
    mut parts: Query<&mut Visibility, Or<(With<Backdrop>, With<InputBox>)>>,
    mut boxes: Query<&mut ScrollPosition, With<LogBox>>,
    mut was: Local<bool>,
) {
    let wanted = if typing.on { Visibility::Inherited } else { Visibility::Hidden };
    for mut visibility in &mut parts {
        visibility.set_if_neq(wanted);
    }
    if *was && !typing.on {
        for mut scroll in &mut boxes {
            if scroll.y != STUCK {
                scroll.y = STUCK;
            }
        }
    }
    *was = typing.on;
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
/// the dark box behind them, which is there while a line is typed.
fn drag_log(drag: On<Pointer<Drag>>, mut boxes: Query<(&ComputedNode, &mut ScrollPosition), With<LogBox>>) {
    if let Ok((node, mut scroll)) = boxes.single_mut() {
        scroll_by(node, &mut scroll, -drag.delta.y);
    }
}

/// Scrolls the lines with the mouse's wheel while the cursor is over them
/// and a line is typed (the camera does not zoom by it there,
/// `read_mouse_zoom`; with nothing typed the wheel is the camera's, over
/// the lines as anywhere).
fn wheel(
    mut wheels: MessageReader<MouseWheel>,
    typing: Res<Typing>,
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
    if turned == 0.0 || !typing.on {
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
/// that sends it, or on the keyboard among the buttons: for [`act`] to act
/// on.
fn press(
    press: On<Pointer<Press>>,
    inputs: Query<(), With<InputBox>>,
    buttons: Query<(), With<ChatButton>>,
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
    } else if buttons.contains(press.entity) {
        pressed.button = true;
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
        typing.shut = true;
    }
}

/// Takes the box up, says what is typed, or lets go of it: by a press on
/// the keyboard among the buttons, or by T or Enter on desktop, to begin;
/// by Enter on either keyboard, or a press on the dart, to say it; and by
/// Escape, or a press on the keyboard's button again, to leave off.
#[allow(clippy::too_many_arguments)]
fn act(
    mut pressed: ResMut<Pressed>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut typing: ResMut<Typing>,
    mut name_box: ResMut<NameBox>,
    mut up: ResMut<DialogUp>,
    (close_up, signup): (Res<CloseUp>, Res<Signup>),
    me: Query<&Venue, With<Player>>,
    mut says: MessageWriter<Say>,
) {
    let (input, button, send) = (
        std::mem::take(&mut pressed.input),
        std::mem::take(&mut pressed.button),
        std::mem::take(&mut pressed.send),
    );
    // The press that let go of the box, if it was on the keyboard's own
    // button, does not take it up again.
    let shut = std::mem::take(&mut typing.shut);
    let enter = keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    if typing.on {
        // Whatever else took the box, or the screen, has it.
        if !name_box.up() {
            typing.on = false;
            return;
        }
        if name_box.entered() || enter || send {
            says.write(Say(name_box.shown()));
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
    let here = me.single().is_ok_and(|&venue| shown_on(venue));
    let by_key = enter || keyboard.just_pressed(KeyCode::KeyT);
    if ((button && !shut) || by_key) && here && !up.0 && !name_box.up() && !signup.up() && !close_up.busy() {
        name_box.open_for(&typing.draft, rules::LONGEST);
        up.0 = true;
        typing.on = true;
        typing.blink = 0.0;
    }
}

/// Says what there is to say ([`Say`]), each if it is a line at all
/// (`roundtown_net::chat::line`): online it is asked of the server, which
/// tells everyone in your room, you among them; offline there is nobody to
/// tell, and it is written here, under your own name.
fn say(
    mut says: MessageReader<Say>,
    (online, fighting): (Res<Online>, Res<Fighting>),
    me: Query<(&Name, &Venue), With<Player>>,
    mut log: ResMut<Log>,
    mut asks: MessageWriter<Ask>,
) {
    for Say(typed) in says.read() {
        let Some(text) = rules::line(typed) else {
            continue;
        };
        let Ok((name, &venue)) = me.single() else {
            continue;
        };
        if !shown_on(venue) {
            continue;
        }
        if online.is() {
            asks.write(Ask::Chat(text));
        } else {
            let mine = online.me().map_or(0, |me| me.id);
            let colour = match fighting.seat.filter(|_| fighting.members.len() > 1 || mine != 0) {
                Some(seat) => ui().colours.arena.team(arena_team(usize::from(seat))),
                None => ui().colours.chat.name,
            };
            log.push(said(name.as_str(), colour, &text));
        }
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
    fn a_line_fades_from_the_moment_it_is_said_and_the_older_is_the_fainter() {
        // Whole as it is said, and gone when its time is up, and after.
        assert_eq!(faded(0.0), FADE_STEPS);
        assert_eq!(faded(LINE_SECS), 0);
        assert_eq!(faded(LINE_SECS * 3.0), 0);
        assert_eq!(faded(-1.0), FADE_STEPS);
        // Never the stronger for being older, and a line said a few
        // seconds before another is the fainter of the two: what Hajun
        // asked for, "the older line should be drawn fainter because it's
        // partway through fading".
        let mut last = FADE_STEPS;
        for tenth in 0..=(LINE_SECS * 10.0) as u32 {
            let step = faded(tenth as f32 / 10.0);
            assert!(step <= last, "{tenth}: {step} after {last}");
            last = step;
        }
        for age in [2.0, 4.0, 6.0, 8.0] {
            assert!(faded(age + 3.0) < faded(age), "{age}");
        }
        // It is slow at first: still nine parts in ten there a third of
        // the way through, to be read, and more than half there with a
        // third of its time to go.
        assert!(faded(LINE_SECS / 3.0) >= FADE_STEPS / 10 * 9);
        assert!(faded(LINE_SECS * 2.0 / 3.0) > FADE_STEPS / 2);
        // A colour thinned keeps what it is and loses its strength, by
        // what it had: a see-through one is never more than it was.
        let ink = Color::srgba(1.0, 0.5, 0.25, 0.8);
        assert_eq!(thinned(ink, FADE_STEPS), ink);
        assert_eq!(thinned(ink, 0).alpha(), 0.0);
        let half = thinned(ink, FADE_STEPS / 2);
        assert!((half.alpha() - 0.4).abs() < 1e-6);
        assert_eq!(half.with_alpha(0.8), ink);
    }

    #[test]
    fn when_each_line_was_said_is_kept_by_its_number() {
        let mut drawn = Drawn::default();
        assert_eq!(drawn.said_at(0), None);
        for (id, at) in [(7, 1.0), (8, 2.5), (9, 2.5), (10, 9.0)] {
            drawn.said.push_back((id, at));
        }
        assert_eq!(drawn.said_at(7), Some(1.0));
        assert_eq!(drawn.said_at(9), Some(2.5));
        assert_eq!(drawn.said_at(10), Some(9.0));
        // One from before these, and one not drawn yet.
        assert_eq!(drawn.said_at(6), None);
        assert_eq!(drawn.said_at(11), None);
        // The oldest let go of, the rest are still found.
        drawn.said.pop_front();
        assert_eq!(drawn.said_at(7), None);
        assert_eq!(drawn.said_at(8), Some(2.5));
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
