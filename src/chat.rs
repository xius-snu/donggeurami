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
//! behind them. **The newest line is always on the same
//! line**, the lowest ("instead of being like the first text starts at the
//! top of the rectangle and makes its way downward, no, it should be like
//! every new text starts at the same line. Even the first message stays at
//! the same area, and it goes up and fades as new messages come"): the
//! first thing said is written there, and each line after it moves what
//! was said before up one. **Three lines are in sight, and every one of
//! them is whole** (since 2026-11-15; Hajun: "actually now i think about
//! it i think no line should be transparent they should all be fully
//! visible. just keep chat height to like 2.5 line or sth", and that
//! night, having had the two and a half: "right now the chat height is
//! like 2.5lines high, i think it can be like 3"): the newest and the two
//! over it, none of them cut. **Nothing goes
//! with time**: the last thing said stays until something else is.
//!
//! That is the third way of it in a night. First a line faded as it aged
//! and was gone twelve seconds on, my reading of "partway through fading"
//! and not what was meant ("i dont want timed fades for the chat
//! messages. the last message sent should be visible always"); then the
//! three lines in sight were each fainter than the one under it, by where
//! they stood, scrolling with them ("the top one looking sooo faded, and
//! the second to top one looking somewhat faded, and the last one not
//! faded at all"; "when u scroll, the "fade" of the text should be
//! "scroll" too"). **That second way is still here, turned off**: how much
//! of itself a line is drawn with by where it stands is the file's
//! `colours.chat.strengths` ([`strength_at`], [`fade`]), every one of them
//! 1 now, and other numbers there bring the fading back.
//!
//! **It scrolls with nothing typed** ([`drag_log`]; Hajun: "also test if
//! you can scroll up on the chat", "u should be able to scroll the chat
//! without needing to open the chatbox input box/keyboard"). A finger
//! dragged on the lines, or the mouse's wheel over them, goes back through
//! the last [`KEPT`], as while a line is typed. So the lines take a press,
//! once there is more of them than is in sight: with two lines or one
//! there is nothing to scroll to ("nvm about the being able to scroll even
//! when there is 2 line of text that doesnt make sense"), and a finger put
//! down on them looks round. Left scrolled back, it goes to its newest
//! line again [`RETURN_SECS`] after it was last touched (mine: or what is
//! said next is said out of sight).
//!
//! **Each line has a thin black stroke round its words** ([`STROKE`],
//! `colours.chat.stroke`), which is all that sets them off from the world
//! behind them (Hajun, the same night, of the shadow under them that it
//! takes the place of: "i think the shadow on the text is a bit hard to
//! see maybe it should just be a very thin black stroke of the text in the
//! chatboxes"). Bevy 0.19 draws no stroke round words, so a line is
//! written five times: four in the stroke's colour, a hair up and to the
//! left, up and to the right, and so on, and its own over them
//! ([`write`]). (A line that is faded, were the fading turned on again,
//! has a stroke that is more faded still: the four copies show through
//! words that are see-through, and would make a faded line a dark one.)
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
//! Where it is, how big and how many lines it keeps are here; its
//! colours and its words are the file's (`colours.chat`, `words.chat`).
//! It keeps clear of the camera's hole (`safe`).
//!
//! **At the tables of the four games of 2026-11-18** (`table`) it is there
//! too, each table a room of its own, begun afresh: where it is in the town
//! for Blend In, which is played in the town, and where it is in Bridge
//! Wars for the other three, whose bars are as Bridge Wars' is. A name is
//! in its team's colour there once the game says its teams are out, by the
//! team its body is on (in Bridge Wars by its seat, as ever). And a game
//! may hush it ([`Hushed`]).

use std::collections::VecDeque;

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_replicon::prelude::Remote;
use roundtown_net::{Ask, Member, Tell, arena_team, chat as rules};

use crate::arena::Fighting;
use crate::hud::{self, ChatButton, DialogUp, Picture, Pictures, TakesPress};
use crate::island::Venue;
use crate::net::Online;
use crate::safe::KeepsClear;
use crate::signup::{self, LiftsForKeyboard, NameBox, Signup};
use crate::table::AtTable;
use crate::ui::{UiChanged, ui};
use crate::{CloseUp, Player, Team};

/// Sizes, as shares of the short side of the screen. Where the chat begins
/// down the screen in the town, well under your name and your balance,
/// which end 20 shares down, and how wide it is; how tall the box of lines
/// is, three lines and the room round them (a line is 3 shares from the
/// next, 1.2 times its words; since the night of 2026-11-15, Hajun: "right
/// now the chat height is like 2.5lines high, i think it can be like 3".
/// Two lines and a half for the day before it, 8.6, the half line cut
/// along the box's top, at "just keep chat height to like 2.5 line or
/// sth"; four whole lines in the town and three in a game until then),
/// the gap under it, and how tall
/// the box a line is typed into is: together they end 57 shares down, over
/// the stick, whose disc reaches up to 58 when it is held from as high as
/// it may be. The words in each, the room round them, and how round the
/// corners are. **The lowest line is where it was** in the town, 48.6
/// shares down, and in a game, 49.6 ([`ARENA_TOP`]): the box is taller
/// from its top.
const TOP: f32 = 38.5;
const LOG_HEIGHT: f32 = 10.1;
/// How wide it is, in shares of the short side: 80 since the night of
/// 2026-11-14 (Hajun: "the width of the chatbox region could be
/// wider/longer so more text fits in each line"), where it was 50: about
/// sixty letters to a line where there were forty, so that a name and a
/// line as long as lines may be are two lines and not three. And no wider
/// than [`MOST_WIDE`] parts in a hundred of the screen's width, which on
/// a phone lying on its side is the 80, and on a screen sixteen by nine,
/// where the middle of the screen is nearer, 71. How much wider is mine.
const WIDTH: f32 = 80.0;
const MOST_WIDE: f32 = 40.0;
/// How far each of the four copies of a line's words that make its
/// stroke is set from the words themselves, across and down, in shares
/// of the short side: a pixel of a screen 1,080 high, under a twenty-fifth
/// of the words' height ("a very thin black stroke"). Mine.
const STROKE: f32 = 0.09;
/// Where the four copies are set: the four corners.
const CORNERS: [Vec2; 4] = [Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0), Vec2::new(-1.0, 1.0), Vec2::new(1.0, 1.0)];
/// Where it begins in a game of Bridge Wars, where what is top left, your
/// name, your health and the four things you hold, ends 38 shares down
/// (411 of 1,080 pixels, measured): under those, with a share and a half
/// between the last of the four counts, the amethyst, and the box, and
/// two between it and the top line's words (Hajun, 2026-11-15: "you could
/// move the chatbox downward a bit to have a tiny slightly more gap
/// between chatbox and the amethyst inventory number in the bridge wars
/// minigame"; it was one share, and one and a half to the words, while
/// the box began 39 down and was three lines tall, its foot at 49.1).
/// With two lines and a half the box began 41 down, three shares under the
/// count; with three again, that night, it begins a share and a half
/// higher than that, because its foot is as low as it goes: the box a
/// line is typed into ends 57.6 down, and the stick's disc reaches up to
/// 57.75. So the gap is half a share more than it was when Hajun asked
/// for more, and not the three it was for a day. Mine.
const ARENA_TOP: f32 = 39.5;
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
/// How long after it was last scrolled, with nothing being typed, the
/// chat goes back to its newest line, in seconds.
const RETURN_SECS: f32 = 5.0;
/// How finely a line's strength is reckoned as it is scrolled: in steps
/// of this, so that it is drawn again as it comes to the next and not at
/// every hair it moves.
const STRENGTH_STEP: f32 = 0.02;

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

    /// How many lines have been said since the one numbered `id`: none
    /// since the newest.
    fn since(&self, id: u64) -> usize {
        usize::try_from(self.next.saturating_sub(id.saturating_add(1))).unwrap_or(usize::MAX)
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

/// How much of itself a line is drawn with, with nothing being typed and
/// `since` lines said after it, from 1, all of it, to 0, none: what the
/// file has for the newest, the one before it and the one before that
/// (`colours.chat.strengths`), and nothing for any older. No clock comes
/// into it. What a line is drawn with until it has been laid out, and
/// what it is drawn with after while the chat is at its newest line and
/// every line is one line tall ([`strength_at`]).
fn strength(strengths: &[f32; 3], since: usize) -> f32 {
    strengths.get(since).map_or(0.0, |share| share.clamp(0.0, 1.0))
}

/// How much of itself a line is drawn with whose foot is `up` lines over
/// the foot of what is in sight, with nothing being typed: the file's
/// three strengths on the lowest line, the one over it and the one over
/// that, and between two lines so much of the one and so much of the
/// other, in steps of [`STRENGTH_STEP`]; all of the lowest line's for a
/// line scrolled past the foot. The third holds for two lines more, which
/// is further than anything is in sight (three lines), and over
/// that there is nothing: a line on its way out of sight is not seen to
/// fade unless the file's numbers have it fade. So the fading, when there
/// is any, goes by where a line is in sight, and scrolls as the lines do.
fn strength_at(strengths: &[f32; 3], up: f32) -> f32 {
    let on = |line: usize| match line {
        line if line < strengths.len() => strength(strengths, line),
        line if line <= strengths.len() + 1 => strength(strengths, strengths.len() - 1),
        _ => 0.0,
    };
    let share = if up <= 0.0 {
        on(0)
    } else {
        let line = up.floor();
        let (under, over) = (on(line as usize), on(line as usize + 1));
        under + (over - under) * (up - line)
    };
    (share / STRENGTH_STEP).round() * STRENGTH_STEP
}

/// `colour`, as much of it as `share` leaves, from 0 to 1.
fn thinned(colour: Color, share: f32) -> Color {
    colour.with_alpha(colour.alpha() * share)
}

/// The colour of a piece of a line drawn at `share` of its strength: of
/// its words, that much of their own colour; of its stroke, that much of
/// that much of the stroke's, so that a faded line's stroke is fainter
/// than its words. The four copies that make the stroke lie under the
/// words as well as round them, and through words that are half there
/// they would show as half black: a line faded to grey, not to nothing.
fn drawn_in(ink: &Ink, share: f32) -> Color {
    thinned(ink.colour, if ink.rim { share * share } else { share })
}

/// Something to say to the room you are in, as it was typed or chosen:
/// the box's line, and what the big speech bubble offers (`quick`). Said
/// by [`say`], if it is a line at all.
#[derive(Message, Clone, Debug)]
pub(crate) struct Say(pub(crate) String);

/// The chat hushed by a game (since 2026-11-18): while it is, the box a
/// line is typed into cannot be taken up, and nothing is said, typed or
/// from the speech bubble's fan (`quick`); a line being typed as it is
/// hushed is let go of, and kept for later. Blend In hushes it from the
/// hide to the end of the seek, a line said there giving the hider away
/// (mine). A game that hushes it says so again when it is over; a table's
/// end lets it go anyway (`table`).
#[derive(Resource, Default, PartialEq)]
pub(crate) struct Hushed(pub(crate) bool);

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
/// One line in it, by its number in the [`Log`], and how much of itself
/// it is drawn with now, from 0 to 1 ([`strength`]): what holds the five
/// writings of it ([`write`]).
#[derive(Component)]
struct LineOf(u64);
#[derive(Component)]
struct Strength(f32);
/// The colour a piece of a line is in at full strength, and whether it is
/// a piece of the line's stroke rather than of its words
/// ([`drawn_in`]).
#[derive(Component)]
struct Ink {
    colour: Color,
    rim: bool,
}
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
    /// When the lines were last scrolled, in the app's own seconds
    /// ([`RETURN_SECS`]).
    scrolled_at: f32,
}

/// What of the chat was pressed since [`act`] last looked: the box a line
/// is typed into, the keyboard among the buttons, and the dart.
#[derive(Resource, Default)]
struct Pressed {
    input: bool,
    button: bool,
    send: bool,
}

/// Which room the lines shown were said in, and which line is the next
/// to be drawn.
#[derive(Resource, Default)]
struct Drawn {
    room: Option<(Venue, u32)>,
    next: u64,
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
        .init_resource::<Hushed>()
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
                max_width: Val::Vw(MOST_WIDE),
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
/// Wars, its waiting room too; and at the tables of the four games of
/// 2026-11-18, on each one's venue.
pub(crate) fn shown_on(here: Venue) -> bool {
    matches!(here, Venue::Town | Venue::Arena) || here.game().is_some()
}

/// Where the chat begins down the screen `here`, and how tall its box of
/// lines is, in shares of the short side: under what each place has top
/// left, and over the stick in both. In Blend In, played in the town, as
/// in the town; in the other three games of 2026-11-18, whose bars are as
/// Bridge Wars' is, or shorter, as in Bridge Wars.
fn place(here: Venue) -> (f32, f32) {
    let as_in_bridge_wars = matches!(here, Venue::Arena | Venue::KnockOff | Venue::BridgeRace | Venue::LastStand);
    (if as_in_bridge_wars { ARENA_TOP } else { TOP }, LOG_HEIGHT)
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
/// at the tables of the four games, and in the file's colour for names
/// anywhere else.
#[allow(clippy::type_complexity)]
fn hear(
    mut tells: MessageReader<Tell>,
    fighting: Res<Fighting>,
    online: Res<Online>,
    me: Query<(&Venue, Option<&Team>), With<Player>>,
    others: Query<(&Member, &Team), With<Remote>>,
    mut log: ResMut<Log>,
) {
    let (here, mine) = me.single().map_or((None, None), |(&venue, team)| (Some(venue), team.map(|team| team.0)));
    for tell in tells.read() {
        if let Tell::Said { who, name, text } = tell {
            // Who is on which team: in a fight, by their seat, as its game
            // says each seat's team (`Fighting::teams`: Bridge Wars' seat
            // modulo four as ever, Bridge Race's two, Last Stand's none, so
            // that a team of everyone is not all written in red); at another
            // game's table, by the team their body is on, yours too, where
            // its game gives one.
            let team = if fighting.venue.is_some() && fighting.venue == here {
                fighting.team_of_member(*who)
            } else if here == Some(Venue::Arena) {
                fighting.members.iter().position(|member| *member == *who && *who != 0).map(arena_team)
            } else if online.me().is_some_and(|me| me.id == *who) {
                mine
            } else {
                others.iter().find(|(member, _)| member.id == *who).map(|(_, team)| team.0)
            };
            log.push(said(name, name_colour(&fighting, team), text));
        }
    }
}

/// The colour the name of whoever is on `team` is written in: their
/// team's, in your game of Bridge Wars, by their seat, once the fight has
/// begun and the teams are out (`Fighting::teams_out`; in its waiting
/// room too until 2026-11-16), and so at the tables of the four games, as
/// each game says (since 2026-11-18); anyone else's, and anyone's until
/// then, the file's colour for a name.
fn name_colour(fighting: &Fighting, team: Option<u8>) -> Color {
    let colours = ui().colours;
    match team {
        Some(team) if fighting.teams_out => colours.arena.team(team),
        _ => colours.chat.name,
    }
}

/// Begins the chat afresh as you come into another room: from the town
/// into a game, back, or into another game. What was said in one is not
/// shown in another.
fn begin_afresh(
    me: Query<&Venue, With<Player>>,
    fighting: Res<Fighting>,
    at: Res<AtTable>,
    mut drawn: ResMut<Drawn>,
    mut log: ResMut<Log>,
) {
    let Ok(&here) = me.single() else {
        return;
    };
    // A game by its number, Bridge Wars' and each table's; the town, and
    // anywhere else, as one room each.
    let table = at.0.filter(|table| table.game.venue() == here).map(|table| table.id);
    let room = (here, if here == Venue::Arena { fighting.game } else { table.unwrap_or(0) });
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
/// file's colours change on desktop. Each as much there as its place
/// among them makes it ([`strength`]), or wholly while a line is typed,
/// until it is laid out and [`fade`] has it by where it is.
fn fill(
    log: Res<Log>,
    changed: Res<UiChanged>,
    typing: Res<Typing>,
    mut drawn: ResMut<Drawn>,
    boxes: Query<Entity, With<LogBox>>,
    lines: Query<(Entity, &LineOf)>,
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
    let colours = file.colours.chat;
    let first = log.first();
    for (entity, of) in &lines {
        if restyle || of.0 < first {
            commands.entity(entity).despawn();
        }
    }
    let from = if restyle { first } else { drawn.next.max(first) };
    for (id, line) in log.lines.iter().filter(|(id, _)| *id >= from) {
        if line.0.is_empty() {
            continue;
        }
        let share = if typing.on { 1.0 } else { strength(&colours.strengths, log.since(*id)) };
        commands
            .spawn((
                LineOf(*id),
                Strength(share),
                ChildOf(log_box),
                // Its writings one over another: the four of the stroke
                // laid out by themselves over the same room, and its own,
                // which is as tall as the line is.
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.0,
                    ..default()
                },
                // Not drawn at all, nor gone over for drawing, while none
                // of it shows.
                if share > 0.0 { Visibility::Inherited } else { Visibility::Hidden },
                Pickable::IGNORE,
            ))
            .with_children(|writings| {
                for corner in CORNERS {
                    write(writings, line, Some((colours.stroke, corner)), share);
                }
                write(writings, line, None, share);
            });
    }
    drawn.next = log.next;
}

/// Writes `line` once, under what holds its writings: its words, each
/// piece in its own colour, where the line is laid out; or, with a colour
/// and a corner, a copy of them all in that colour, laid out by itself
/// over the same room and set [`STROKE`] off toward that corner, for the
/// stroke round the words. Piece for piece as the words are written, so
/// that every letter of a copy is where the letter it lies under is. Drawn
/// at `share` of its strength ([`drawn_in`]).
fn write(writings: &mut ChildSpawnerCommands, line: &Line, rim: Option<(Color, Vec2)>, share: f32) {
    let ink = |own: Color| Ink {
        colour: rim.map_or(own, |(stroke, _)| stroke),
        rim: rim.is_some(),
    };
    let Some(((words, colour), rest)) = line.0.split_first() else {
        return;
    };
    let (node, set_off) = match rim {
        Some((_, corner)) => (
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                width: Val::Percent(100.0),
                ..default()
            },
            UiTransform::from_translation(Val2::new(Val::VMin(corner.x * STROKE), Val::VMin(corner.y * STROKE))),
        ),
        None => (Node::default(), UiTransform::default()),
    };
    let first = ink(*colour);
    writings
        .spawn((
            Text::new(words.clone()),
            crate::font::sized(TEXT),
            TextColor(drawn_in(&first, share)),
            first,
            TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
            node,
            set_off,
            Pickable::IGNORE,
        ))
        .with_children(|pieces| {
            for (words, colour) in rest {
                let piece = ink(*colour);
                pieces.spawn((
                    TextSpan::new(words.clone()),
                    crate::font::sized(TEXT),
                    TextColor(drawn_in(&piece, share)),
                    piece,
                    // A press on a piece of a line is not the piece's: it
                    // goes through to what is behind the lines, which is
                    // what a finger is dragged on to scroll them. (A piece
                    // of words is picked by itself, whatever the words it
                    // is a piece of say of themselves: for the hours that
                    // these said nothing, a finger on a line's words
                    // scrolled nothing.)
                    Pickable::IGNORE,
                ));
            }
        });
}

/// Keeps each line as much there as where it is in sight makes it
/// ([`strength_at`]), which is wholly, as the file's strengths are now,
/// and with others whatever is on the lowest line as the first has it and
/// what is over it as the next two do; nothing of a line well out of
/// sight over those, which is not gone over for drawing; and all of them
/// wholly while a line is typed. By where each was last laid out
/// against the foot of the box, so that the fading scrolls as the lines
/// do, and a line two lines tall pushes what is over it a line further
/// up; a line not laid out yet, by how many have been said since
/// ([`strength`]). A line is gone over, its five writings piece by piece,
/// only as its strength changes: as something is said after it, as it is
/// scrolled, and as the box is taken up and let go of.
#[allow(clippy::type_complexity)]
fn fade(
    log: Res<Log>,
    typing: Res<Typing>,
    windows: Query<&Window, With<PrimaryWindow>>,
    boxes: Query<(&ComputedNode, &UiGlobalTransform), With<LogBox>>,
    mut lines: Query<(Entity, &LineOf, &ComputedNode, &UiGlobalTransform, &mut Strength, &mut Visibility), Without<LogBox>>,
    kin: Query<&Children>,
    mut pieces: Query<(&Ink, &mut TextColor)>,
) {
    let strengths = ui().colours.chat.strengths;
    // The foot of what is in sight, down the screen, and how tall a line
    // is, both in the screen's own pixels.
    let sight = match (windows.single(), boxes.single()) {
        (Ok(window), Ok((node, place))) if node.size.y > 0.0 => {
            let line = window.width().min(window.height()) * TEXT * 1.2 / 100.0 / node.inverse_scale_factor();
            Some((place.translation.y + node.content_box().max.y, line.max(1.0)))
        }
        _ => None,
    };
    for (line, of, node, place, mut drawn_at, mut visibility) in &mut lines {
        let share = match sight {
            _ if typing.on => 1.0,
            Some((foot, tall)) if node.size.y > 0.0 => {
                let up = (foot - (place.translation.y + node.size.y * 0.5)) / tall;
                // (A hair of the layout's rounding is not part of a line.)
                strength_at(&strengths, if (up - up.round()).abs() < 0.04 { up.round() } else { up })
            }
            _ => strength(&strengths, log.since(of.0)),
        };
        if drawn_at.0 == share {
            continue;
        }
        drawn_at.0 = share;
        visibility.set_if_neq(if share > 0.0 { Visibility::Inherited } else { Visibility::Hidden });
        for piece in kin.iter_descendants(line) {
            if let Ok((ink, mut colour)) = pieces.get_mut(piece) {
                colour.0 = drawn_in(ink, share);
            }
        }
    }
}

/// Dresses the chat for a line being typed, and undresses it after: the
/// dark box behind the lines and the box the line is typed into are there
/// only then. Let go of, the lines are back at the newest, however far
/// back they were scrolled. With nothing typed, what is behind the lines
/// is still there to be dragged on, with nothing of it to see, once there
/// is more of them than is in sight, as they were last laid out; and
/// lines left scrolled back go to the newest again [`RETURN_SECS`] after
/// they were last scrolled.
#[allow(clippy::type_complexity)]
fn dress(
    time: Res<Time<Real>>,
    typing: Res<Typing>,
    mut backs: Query<(&mut Visibility, &mut BackgroundColor), (With<Backdrop>, Without<InputBox>)>,
    mut inputs: Query<&mut Visibility, (With<InputBox>, Without<Backdrop>)>,
    mut boxes: Query<(&ComputedNode, &mut ScrollPosition), With<LogBox>>,
    mut was: Local<bool>,
) {
    let colours = ui().colours.chat;
    // (Over by more than a pixel or two of rounding.)
    let more_than_in_sight = boxes.iter().any(|(node, _)| node.content_size().y > node.size().y + 2.0);
    for (mut visibility, mut fill) in &mut backs {
        let shown = typing.on || more_than_in_sight;
        visibility.set_if_neq(if shown { Visibility::Inherited } else { Visibility::Hidden });
        fill.set_if_neq(BackgroundColor(if typing.on { colours.back } else { Color::NONE }));
    }
    for mut visibility in &mut inputs {
        visibility.set_if_neq(if typing.on { Visibility::Inherited } else { Visibility::Hidden });
    }
    let let_go = *was && !typing.on;
    let left_alone = !typing.on && time.elapsed_secs() - typing.scrolled_at > RETURN_SECS;
    if let_go || left_alone {
        for (_, mut scroll) in &mut boxes {
            if scroll.y != STUCK {
                scroll.y = STUCK;
            }
        }
    }
    *was = typing.on;
}

/// Scrolls the box of lines `by` logical pixels down its lines, no further
/// than they go either way; back at its newest line, it keeps to it as
/// more come ([`STUCK`]). From where it was last put, not from where the
/// lines were last drawn: those are a whole pixel of the screen's each
/// time, a part of a pixel short of where they were put, and a finger
/// dragged slowly, a pixel or less at a time, took the lines half as far
/// again as it went one way and less far than it went the other (until
/// 2026-11-14; seen as soon as a drag was scripted).
fn scroll_by(node: &ComputedNode, scroll: &mut ScrollPosition, by: f32) {
    let scale = node.inverse_scale_factor();
    let most = ((node.content_size().y - node.size().y) * scale).max(0.0);
    // Where it is now, which is as far as it goes while it keeps to its
    // newest line.
    let now = scroll.y.clamp(0.0, most);
    let to = (now + by).clamp(0.0, most);
    let next = if to >= most - 0.5 { STUCK } else { to };
    if scroll.y != next {
        scroll.y = next;
    }
}

/// Scrolls the lines with a finger, or the mouse held down, dragged over
/// what is behind them: the dark box while a line is typed, and the same
/// with nothing of it to see while none is, once there are lines to
/// scroll to ([`dress`]).
fn drag_log(
    drag: On<Pointer<Drag>>,
    time: Res<Time<Real>>,
    mut typing: ResMut<Typing>,
    mut boxes: Query<(&ComputedNode, &mut ScrollPosition), With<LogBox>>,
) {
    if let Ok((node, mut scroll)) = boxes.single_mut() {
        scroll_by(node, &mut scroll, -drag.delta.y);
        typing.scrolled_at = time.elapsed_secs();
    }
}

/// Scrolls the lines with the mouse's wheel while the cursor is over what
/// is behind them, where that is there to be dragged on (the camera does
/// not zoom by it there, `read_mouse_zoom`; with nothing typed and no
/// more lines than are in sight the wheel is the camera's, over the
/// lines as anywhere).
fn wheel(
    mut wheels: MessageReader<MouseWheel>,
    time: Res<Time<Real>>,
    mut typing: ResMut<Typing>,
    windows: Query<&Window, With<PrimaryWindow>>,
    backs: Query<&InheritedVisibility, With<Backdrop>>,
    mut boxes: Query<(&ComputedNode, &UiGlobalTransform, &InheritedVisibility, &mut ScrollPosition), With<LogBox>>,
) {
    let turned: f32 = wheels
        .read()
        .map(|event| match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
        })
        .sum();
    if turned == 0.0 || !backs.iter().any(|shown| shown.get()) {
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
        typing.scrolled_at = time.elapsed_secs();
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
    (close_up, signup, hushed): (Res<CloseUp>, Res<Signup>, Res<Hushed>),
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
        // Hushed by a game: let go of, what was typed kept for later.
        if hushed.0 {
            let_go(&mut typing, &mut name_box, &mut up);
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
    // Not with anything else up, nor where the chat is not shown, nor
    // while a game has it hushed.
    let here = me.single().is_ok_and(|&venue| shown_on(venue));
    let by_key = enter || keyboard.just_pressed(KeyCode::KeyT);
    if ((button && !shut) || by_key) && here && !hushed.0 && !up.0 && !name_box.up() && !signup.up() && !close_up.busy() {
        name_box.open_for(&typing.draft, rules::LONGEST);
        up.0 = true;
        typing.on = true;
        typing.blink = 0.0;
    }
}

/// Says what there is to say ([`Say`]), each if it is a line at all
/// (`roundtown_net::chat::line`): online it is asked of the server, which
/// tells everyone in your room, you among them; offline there is nobody to
/// tell, and it is written here, under your own name. Nothing while a game
/// has the chat hushed ([`Hushed`]).
fn say(
    mut says: MessageReader<Say>,
    (online, fighting, hushed): (Res<Online>, Res<Fighting>, Res<Hushed>),
    me: Query<(&Name, &Venue, Option<&Team>), With<Player>>,
    mut log: ResMut<Log>,
    mut asks: MessageWriter<Ask>,
) {
    for Say(typed) in says.read() {
        if hushed.0 {
            continue;
        }
        let Some(text) = rules::line(typed) else {
            continue;
        };
        let Ok((name, &venue, team)) = me.single() else {
            continue;
        };
        if !shown_on(venue) {
            continue;
        }
        if online.is() {
            asks.write(Ask::Chat(text));
        } else {
            // In a game, once its fight has begun, in your team's colour,
            // as your name is on its plate (`hud::colour_name`): the team
            // your body is on, which in Bridge Wars is your seat's.
            let colour = name_colour(&fighting, team.map(|team| team.0));
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
    fn no_line_in_sight_is_faded_and_the_fading_that_is_turned_off_goes_by_where_a_line_is() {
        // As the file has them now, every line in sight is whole ("no
        // line should be transparent they should all be fully visible"),
        // wherever in sight it is, the top one of the three among them,
        // and for a line past the top of what is in sight, three lines
        // up; a line further out of sight than that is going, where
        // nobody sees it go, and two lines out is not drawn.
        let file = ui().colours.chat.strengths;
        assert_eq!(file, [1.0, 1.0, 1.0]);
        let in_sight = (LOG_HEIGHT - PAD) / (TEXT * 1.2);
        for tenth in -10..=40 {
            assert_eq!(strength_at(&file, tenth as f32 / 10.0), 1.0, "{tenth} tenths of a line up");
        }
        assert!(in_sight + 0.45 < 4.0, "{in_sight} lines in sight");
        assert_eq!(strength_at(&file, 5.0), 0.0, "out of sight, and not drawn");
        // The fading that is still here, with the numbers it had for an
        // evening: the newest whole, the one before it fainter and still
        // there to be read, the one before that fainter again, and
        // nothing of any older.
        let strengths = [1.0, 0.55, 0.25];
        let shares: Vec<f32> = (0..5).map(|since| strength(&strengths, since)).collect();
        assert!(shares[0] >= 0.95, "{shares:?}");
        assert!(shares[1] < shares[0] - 0.2 && shares[1] >= 0.4, "{shares:?}");
        assert!(shares[2] < shares[1] - 0.15 && shares[2] >= 0.1, "{shares:?}");
        assert_eq!((shares[3], shares[4]), (0.0, 0.0));
        // Whatever the file says, it is a share: none of nothing, and no
        // more than all.
        assert_eq!(strength(&[7.0, -1.0, 0.5], 0), 1.0);
        assert_eq!(strength(&[7.0, -1.0, 0.5], 1), 0.0);
        assert_eq!(strength(&[7.0, -1.0, 0.5], 2), 0.5);
        // By how many have been said since, which the log knows by their
        // numbers: the newest has had none said since, however long ago
        // it was said.
        let mut log = Log::default();
        for n in 0..4 {
            log.push(Line(vec![(format!("line {n}"), Color::WHITE)]));
        }
        assert_eq!([3, 2, 1, 0].map(|id| log.since(id)), [0, 1, 2, 3]);
        assert_eq!(log.since(10), 0, "one not said yet is no older than the newest");
        // Begun afresh, the next thing said is the newest.
        log.clear();
        log.push(Line(vec![("hello".into(), Color::WHITE)]));
        assert_eq!((log.since(4), log.since(3)), (0, 1));
        // Scrolled, the fading goes by where a line is in sight: on the
        // lowest line, the one over it and the one over that it is what
        // it is by its place among them; half way from one line to the
        // next, half way from the one strength to the other; the third
        // line's strength for two lines more, which is past what is in
        // sight; and over that, going, and gone. Never the fainter for
        // being lower.
        for line in 0..3 {
            assert!((strength_at(&strengths, line as f32) - strength(&strengths, line)).abs() <= STRENGTH_STEP * 0.5 + 1e-6, "on line {line}");
        }
        assert_eq!(strength_at(&strengths, -0.7), strength_at(&strengths, 0.0), "scrolled past the foot");
        let between = strength_at(&strengths, 0.5);
        assert!((between - (shares[0] + shares[1]) * 0.5).abs() <= STRENGTH_STEP, "{between}");
        assert!((strength_at(&strengths, 2.5) - shares[2]).abs() <= STRENGTH_STEP && (strength_at(&strengths, 4.0) - shares[2]).abs() <= STRENGTH_STEP);
        let going = strength_at(&strengths, 4.5);
        assert!(going > 0.0 && going < shares[2], "{going}");
        assert_eq!((strength_at(&strengths, 5.0), strength_at(&strengths, 7.3)), (0.0, 0.0));
        let mut last = 1.0;
        for tenth in 0..50 {
            let share = strength_at(&strengths, tenth as f32 / 10.0);
            assert!(share <= last + 1e-6, "{tenth} tenths of a line up: {share} after {last}");
            last = share;
        }
        // A colour thinned keeps what it is and loses its strength, by
        // what it had: a see-through one is never more than it was.
        let ink = Color::srgba(1.0, 0.5, 0.25, 0.8);
        assert_eq!(thinned(ink, 1.0), ink);
        assert_eq!(thinned(ink, 0.0).alpha(), 0.0);
        let half = thinned(ink, 0.5);
        assert!((half.alpha() - 0.4).abs() < 1e-6);
        assert_eq!(half.with_alpha(0.8), ink);
        // A line's words at a share of their strength, and its stroke at
        // that share of that share: whole together, gone together, and in
        // between the stroke the fainter, so that a faded line is not a
        // dark one.
        let (words, stroke) = (Ink { colour: Color::WHITE, rim: false }, Ink { colour: Color::BLACK, rim: true });
        assert_eq!((drawn_in(&words, 1.0).alpha(), drawn_in(&stroke, 1.0).alpha()), (1.0, 1.0));
        assert_eq!((drawn_in(&words, 0.0).alpha(), drawn_in(&stroke, 0.0).alpha()), (0.0, 0.0));
        assert!((drawn_in(&words, 0.5).alpha() - 0.5).abs() < 1e-6 && (drawn_in(&stroke, 0.5).alpha() - 0.25).abs() < 1e-6);
        // The stroke is thin: under a twentieth of the words' height each
        // way, and its four copies are at the four corners.
        assert!(STROKE > 0.0 && STROKE < TEXT / 20.0);
        for corner in CORNERS {
            assert_eq!(corner.abs(), Vec2::ONE);
        }
        assert_eq!(CORNERS.iter().copied().sum::<Vec2>(), Vec2::ZERO);
    }

    #[test]
    fn the_chat_is_wide_enough_for_most_of_a_line_and_stops_short_of_the_middle() {
        // Wider than it was, by half as much again and more.
        assert!(WIDTH >= 75.0);
        // On a screen lying on its side, from a tablet's shape to a
        // phone's (the short side is 100): no wider than its share of the
        // screen's width, and its right end short of the middle of the
        // screen, where you stand.
        for wide in [4.0 / 3.0, 16.0 / 9.0, 19.5 / 9.0] {
            let across = 100.0 * wide;
            let width = WIDTH.min(across * MOST_WIDE / 100.0);
            assert!(hud::EDGE + width < across * 0.45, "{wide}: it ends {} of {across} across", hud::EDGE + width);
            assert!(width >= 50.0, "{wide}: {width}");
        }
        // About sixty letters to a line: the room for words, at half
        // their height to a letter, as Inter's are on the whole.
        let letters = (WIDTH - 2.0 * PAD) / (TEXT * 0.5);
        assert!(letters >= 60.0, "{letters}");
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
            // Three lines in sight, each of them whole.
            let lines = (height - PAD) / (TEXT * 1.2);
            assert!((lines - 3.0).abs() < 0.02, "{here:?}: {lines} lines");
        }
        // The lowest line is where it was with two lines and a half, in
        // the town 48.6 shares down and in a game 49.6, and the box in a
        // game well over a share under the last of the four counts top
        // left, which is more than the one it was when Hajun asked for
        // more.
        assert!((place(Venue::Town).0 + LOG_HEIGHT - 48.6).abs() < 1e-4 && (place(Venue::Arena).0 + LOG_HEIGHT - 49.6).abs() < 1e-4);
        assert!(place(Venue::Arena).0 - 38.1 > 1.3);
        // The chat is in the town and in Bridge Wars, and at the four
        // games' tables, and nowhere else: in Blend In, played in the town
        // with the town's buttons, where it is in the town, and in the
        // others, whose top left is Bridge Wars' or less, where it is in
        // Bridge Wars.
        assert!(shown_on(Venue::Town) && shown_on(Venue::Arena));
        assert!(!shown_on(Venue::Home) && !shown_on(Venue::Lobby) && !shown_on(Venue::Plot(3)));
        for game in roundtown_net::Minigame::TABLES {
            let venue = game.venue();
            assert!(shown_on(venue), "{game:?}");
            let like = if venue == Venue::BlendIn { Venue::Town } else { Venue::Arena };
            assert_eq!(place(venue), place(like), "{game:?}");
        }
    }

    #[test]
    fn a_name_is_in_its_team_s_colour_once_the_teams_are_out() {
        let colours = ui().colours;
        let mut fighting = Fighting::default();
        assert_eq!(name_colour(&fighting, Some(1)), colours.chat.name);
        assert_eq!(name_colour(&fighting, None), colours.chat.name);
        fighting.teams_out = true;
        assert_eq!(name_colour(&fighting, Some(1)), colours.arena.team(1));
        assert_eq!(name_colour(&fighting, None), colours.chat.name);
    }
}
