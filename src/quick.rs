//! Things to say at a press: the quick chat (since 2026-11-14; Hajun: "the
//! current larger chat icon button should be used for a quick chat selector
//! for preset chat messages that flannel out", which I take for "fan out";
//! until that day the big speech bubble over the town's jump button took up
//! the chat's box to type into, which the small keyboard over it did after,
//! `hud::ChatButton`).
//!
//! **Since 2026-11-27 it is all there is to say** (Hajun: "get rid of the
//! keyboard icon above the chat bubble icon -> im gona disable custom text
//! chat throughout the game in both minigames and in the lobby. there is
//! only gona be quickchat with the chatbubble"; `roundtown_net::chat::TYPED`):
//! the keyboard is put away everywhere, and in the games, which had no
//! speech bubble, a small one stands where it stood ([`FanFrom`],
//! `hud::small_quick_button`). **What is offered** is the file's: in the
//! town Hajun's eleven (`words.chat.quick`: "Hi! Nice house! Follow me! Add
//! friend? Thanks! Give me money. No. Ok. Yes. Bye! Wanna play?"), and in
//! the games a list of mine (`words.chat.quick_games`), each a line anyone
//! could type, up to `roundtown_net::chat::QUICK_MOST`. **What is said is
//! its number** ([`Say::Quick`]): the server passes on the number alone,
//! and every phone writes it in its own words ([`words_of`]), in the chat
//! and in a bubble over the head of whoever said it (`speech`).
//!
//! **A press on a speech bubble** (`hud::QuickChatButton`) fans out what
//! there is to say ([`offered`]), each on a plate of its own ([`pill`]), in
//! columns of [`ROWS`] up the screen from beside the bubble, the first
//! nearest it, every plate of a column as wide as the widest words in it
//! ([`widths`]), their right ends on a curve that leans in over the town's
//! bubble ([`place`]), each coming out of the bubble a moment after the
//! one before it ([`unfurl`]); the bubble is lit while they are out (six in
//! one column until 2026-11-27, when eleven came). **A press on one says
//! it** (`chat::Say`: online everyone in your room is told, you among
//! them; offline it is written in your own chat), and the fan shuts. A
//! press on the bubble again shuts it too, and so does one anywhere else,
//! which is whatever it landed on's as well, the stick's or a button's
//! ([`shut_under_a_press`]). You can walk with it out.
//!
//! Where the chat is not shown (House Builder's islands) the bubble does
//! nothing, as it did nothing there before. The server takes a line from it
//! a few at once and then one every second and a half
//! (`roundtown_net::chat::BURST`).
//!
//! How it fans, how many to a column, how big and how fast are mine: Hajun's
//! words for all of it are the ones above. What is offered and its colours
//! are the file's (`words.chat.quick`, `quick_games`, `colours.chat.quick`,
//! `quick_text`).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use roundtown_net::chat::{self as rules, Lines, Quick};

use crate::chat::{self, Say};
use crate::hud::{self, DialogUp, QuickChatButton, TakesPress};
use crate::island::Venue;
use crate::signup::Signup;
use crate::ui::ui;
use crate::{CloseUp, Player};

/// How many plates stand one over another before the next column begins.
pub(crate) const ROWS: usize = 6;
/// Sizes, as shares of the short side of the screen: how tall a plate is,
/// how far each is over the one under it, how big its words are, the room
/// either side of them, and between two columns.
pub(crate) const HEIGHT: f32 = 5.8;
const PITCH: f32 = 6.9;
const TEXT: f32 = 2.8;
const PAD: f32 = 2.2;
const COLUMN_GAP: f32 = 1.4;
/// Where the town's plates are, in the box round the jump button, in
/// shares of the short side from its right and its bottom ([`place`]): how
/// far up the first is, just over your bag's button, whose top is 32.5 up;
/// where the right end of the last of a column is, over the place the
/// chat's keyboard stood; and how much further left than that the first
/// one's is, clear of the bubble, which reaches 15.
const FIRST_BOTTOM: f32 = 34.0;
pub(crate) const RIGHT_END: f32 = 8.5;
const SWEEP: f32 = 8.0;
/// How far left of a small bubble its plates' right ends are, in shares of
/// the short side.
const BESIDE: f32 = 1.5;
/// How long a plate takes to come out of the bubble, how long after the
/// one before it each sets off, in seconds, and how small it is as it
/// leaves, as a share of itself.
const UNFURL_SECS: f32 = 0.16;
const STAGGER_SECS: f32 = 0.03;
const FROM_SCALE: f32 = 0.3;

/// Where a speech bubble fans its plates out from, in the box round the
/// jump button, in shares of the short side: how far up the first stands,
/// where the right end of a column's top one is, how much further left the
/// first's is, and the bubble itself, its right, its bottom and how big.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) struct FanFrom {
    pub bottom: f32,
    pub right: f32,
    pub sweep: f32,
    pub bubble: (f32, f32, f32),
}

impl FanFrom {
    /// The town's big bubble (`hud::QUICK_BUTTON`), and Blend In's: from
    /// just over your bag's button, leaning in over the bubble.
    pub(crate) fn town() -> Self {
        Self {
            bottom: FIRST_BOTTOM,
            right: RIGHT_END,
            sweep: SWEEP,
            bubble: hud::QUICK_BUTTON,
        }
    }

    /// A small bubble at `bubble` (its right, bottom and size), where the
    /// chat's keyboard stood in the games: from its foot up, just left of
    /// it.
    pub(crate) fn beside(bubble: (f32, f32, f32)) -> Self {
        let (right, bottom, size) = bubble;
        Self {
            bottom,
            right: right + size + BESIDE,
            sweep: 0.0,
            bubble,
        }
    }
}

/// The fan: what is offered while it is out, nothing while it is shut,
/// where it fans from and how wide each of its columns is; and what has been
/// pressed since [`act`] last looked, a bubble, one of the plates, or
/// anywhere else.
#[derive(Resource, Default)]
struct Fan {
    offered: Vec<(Quick, String)>,
    from: Option<FanFrom>,
    widths: Vec<f32>,
    toggled: Option<Entity>,
    picked: Option<usize>,
    shut: bool,
    /// Its plates were asked for this frame, and are not there to count
    /// yet.
    fresh: bool,
}

impl Fan {
    fn out(&self) -> bool {
        !self.offered.is_empty()
    }
}

/// One of the plates, by which of what is offered it says, from the one
/// nearest the bubble.
#[derive(Component)]
struct Pill(usize);

/// A plate still coming out of the bubble, and for how many seconds it has
/// been.
#[derive(Component)]
struct Unfurl(f32);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Fan>().add_observer(press).add_systems(
        Update,
        (
            // Before you are moved by the press that shut it.
            shut_under_a_press.before(crate::read_player_input),
            // After the chat's box has been let go of by the same press, so
            // that a press on the bubble while a line is typed fans out.
            act.after(shut_under_a_press).after(chat::LetGo),
            unfurl,
        ),
    );
}

/// Which list of things to say is offered `here`: the town's in the town,
/// the games' anywhere else.
pub(crate) fn lines_for(here: Venue) -> Lines {
    if here == Venue::Town { Lines::Town } else { Lines::Games }
}

/// The file's list of things to say for `lines`, as written.
fn list(lines: Lines) -> Vec<String> {
    let file = ui();
    match lines {
        Lines::Town => file.words.chat.quick.clone(),
        Lines::Games => file.words.chat.quick_games.clone(),
    }
}

/// What there is to say from `lines`: the file's, each by its number in
/// the list and as the line it is said as (`roundtown_net::chat::line`:
/// cleaned, and not too long), those that are lines at all, and no more
/// than `roundtown_net::chat::QUICK_MOST`.
fn offered(lines: Lines) -> Vec<(Quick, String)> {
    list(lines)
        .iter()
        .take(usize::from(rules::QUICK_MOST))
        .enumerate()
        .filter_map(|(line, words)| Some((Quick { lines, line: line as u8 }, rules::line(words)?)))
        .collect()
}

/// The words of one of the things to say, as this phone has them, if it
/// has any by that number: what is written in the chat and over the head of
/// whoever said it, whoever said it.
pub(crate) fn words_of(quick: Quick) -> Option<String> {
    list(quick.lines).get(usize::from(quick.line)).and_then(|words| rules::line(words))
}

/// How wide `words` are written at a size of one, in the game's fonts,
/// each letter from the first that has it (`font::files`), as far apart as
/// the font says: a plate's words, measured before they are laid out, for
/// every plate of a column to be as wide as its widest.
fn em_width(words: &str) -> f32 {
    let fonts: Vec<swash::FontRef> = crate::font::files()
        .iter()
        .filter_map(|file| swash::FontRef::from_index(file, 0))
        .collect();
    words
        .chars()
        .map(|letter| {
            fonts
                .iter()
                .find_map(|font| {
                    let glyph = font.charmap().map(letter);
                    (glyph != 0).then(|| font.glyph_metrics(&[]).scale(1.0).advance_width(glyph))
                })
                // A letter no font has is drawn as nothing much: half an em.
                .unwrap_or(0.5)
        })
        .sum()
}

/// How wide each column of plates is, `offered` being what there is to
/// say: as wide as the widest words in it, with room either side.
fn widths(offered: &[(Quick, String)]) -> Vec<f32> {
    offered
        .chunks(ROWS)
        .map(|column| column.iter().map(|(_, words)| em_width(words) * TEXT).fold(0.0, f32::max) + 2.0 * PAD + 0.6)
        .collect()
}

/// Where the plate numbered `index` is, as its right and bottom offsets in
/// the box round the jump button, in shares of the short side, the fan
/// coming `from` where it does and its columns `widths` wide: one over
/// another from beside the bubble, a column at a time, the next column to
/// the left of the last; each one's right end a little further right than
/// the one under it, over the town's bubble, by less each time, like the
/// sticks of a fan.
pub(crate) fn place(index: usize, from: &FanFrom, widths: &[f32]) -> (f32, f32) {
    let (column, row) = (index / ROWS, index % ROWS);
    let through = row as f32 / (ROWS - 1) as f32;
    let before: f32 = widths.iter().take(column).map(|wide| wide + COLUMN_GAP).sum();
    (from.right + from.sweep * (1.0 - through).powi(2) + before, from.bottom + row as f32 * PITCH)
}

/// A plate `through` its way out of the bubble, from 0 to 1: small, and
/// with its right end at the bubble's middle, to begin with; in its own
/// place and its own size at the end, quickly at first and then slower.
fn furled(index: usize, through: f32, from: &FanFrom, widths: &[f32]) -> UiTransform {
    let (right, bottom) = place(index, from, widths);
    let (bubble_right, bubble_bottom, size) = from.bubble;
    let eased = 1.0 - (1.0 - through.clamp(0.0, 1.0)).powi(3);
    let left = 1.0 - eased;
    // From the plate to the bubble's middle: to the right, and down the
    // screen.
    let to_bubble = Vec2::new(
        right - (bubble_right + size * 0.5),
        bottom + HEIGHT * 0.5 - (bubble_bottom + size * 0.5),
    );
    UiTransform {
        translation: Val2::new(Val::VMin(to_bubble.x * left), Val::VMin(to_bubble.y * left)),
        scale: Vec2::splat(FROM_SCALE + (1.0 - FROM_SCALE) * eased),
        rotation: Rot2::IDENTITY,
    }
}

/// One of the plates: `words` on the buttons' grey, a little more solid,
/// with rounded ends, as wide as its column, where [`place`] puts it. It
/// takes a press for itself.
fn pill(index: usize, words: &str, from: &FanFrom, widths: &[f32]) -> impl Bundle {
    let colours = ui().colours;
    let (right, bottom) = place(index, from, widths);
    let wide = widths.get(index / ROWS).copied().unwrap_or(20.0);
    (
        Pill(index),
        Unfurl(0.0),
        TakesPress,
        Button,
        Node {
            position_type: PositionType::Absolute,
            right: Val::VMin(right),
            bottom: Val::VMin(bottom),
            width: Val::VMin(wide),
            height: Val::VMin(HEIGHT),
            padding: UiRect::horizontal(Val::VMin(PAD)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(Val::VMin(HEIGHT * 0.5)),
            ..default()
        },
        BackgroundColor(colours.chat.quick),
        furled(index, 0.0, from, widths),
        // Over whatever else is round the jump button while it is out.
        ZIndex(2),
        children![(
            Text::new(words.to_string()),
            crate::font::sized(TEXT),
            TextColor(colours.chat.quick_text),
            TextShadow {
                offset: Vec2::new(0.0, 2.0),
                color: colours.text_shadow,
            },
            // On one line, however long: the plate is as wide as its
            // column's widest words.
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            Pickable::IGNORE,
        )],
    )
}

/// A press, as it goes down, on a bubble or on one of the plates: for
/// [`act`] to act on.
fn press(
    press: On<Pointer<Press>>,
    bubbles: Query<(), With<QuickChatButton>>,
    pills: Query<&Pill>,
    mut fan: ResMut<Fan>,
) {
    if press.button != PointerButton::Primary || press.original_event_target() != press.entity {
        return;
    }
    if let Ok(pill) = pills.get(press.entity) {
        fan.picked = Some(pill.0);
    } else if bubbles.contains(press.entity) {
        fan.toggled = Some(press.entity);
    }
}

/// A finger put down anywhere but on the fan, while it is out, shuts it in
/// the frame it lands, and is whatever it landed on's as well, as the
/// chat's box is let go of (`chat::shut_under_a_press`).
#[allow(clippy::type_complexity)]
fn shut_under_a_press(
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    own: Query<(&ComputedNode, &UiGlobalTransform), Or<(With<Pill>, With<QuickChatButton>)>>,
    mut fan: ResMut<Fan>,
) {
    if !fan.out() {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let fingers = touches.iter_just_pressed().map(|touch| touch.position());
    let cursor = if mouse.just_pressed(MouseButton::Left) { window.cursor_position() } else { None };
    let on_the_fan = |at: Vec2| own.iter().any(|(node, place)| node.contains_point(*place, at * window.scale_factor()));
    if fingers.chain(cursor).any(|at| !on_the_fan(at)) {
        fan.shut = true;
    }
}

/// Fans out what there is to say, says what is picked, and shuts the fan:
/// by a press on a bubble, on a plate, or anywhere else; and shuts it when
/// anything comes up over the screen, when you leave where the chat is,
/// and when the buttons it hangs among are put up afresh.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn act(
    mut fan: ResMut<Fan>,
    (dialog, close_up, signup): (Res<DialogUp>, Res<CloseUp>, Res<Signup>),
    me: Query<&Venue, With<Player>>,
    bubbles: Query<(Option<&FanFrom>, &ChildOf), With<QuickChatButton>>,
    pills: Query<Entity, With<Pill>>,
    mut fills: Query<&mut BackgroundColor, With<QuickChatButton>>,
    mut says: MessageWriter<Say>,
    mut commands: Commands,
) {
    // Most frames: shut, and nothing pressed.
    if !fan.out() && fan.toggled.is_none() && fan.picked.is_none() && !fan.shut {
        return;
    }
    let (toggled, picked, shut) = (fan.toggled.take(), fan.picked.take(), std::mem::take(&mut fan.shut));
    let here = me.single().ok().copied();
    let free = here.is_some_and(chat::shown_on) && !dialog.0 && !signup.up() && close_up.on.is_none();
    let colours = ui().colours.hud;
    if fan.out() {
        let gone = !std::mem::take(&mut fan.fresh) && pills.is_empty();
        if let Some(&(quick, _)) = picked.and_then(|index| fan.offered.get(index)) {
            says.write(Say::Quick(quick));
        } else if !(toggled.is_some() || shut || !free || gone) {
            return;
        }
        fan.offered.clear();
        for pill in &pills {
            commands.entity(pill).despawn();
        }
        for mut fill in &mut fills {
            fill.set_if_neq(BackgroundColor(colours.button));
        }
        return;
    }
    let (Some(bubble), Some(here)) = (toggled, here) else {
        return;
    };
    if !free {
        return;
    }
    let Ok((from, parent)) = bubbles.get(bubble) else {
        return;
    };
    let from = from.copied().unwrap_or_else(FanFrom::town);
    let offered = offered(lines_for(here));
    if offered.is_empty() {
        return;
    }
    let widths = widths(&offered);
    for (index, (_, words)) in offered.iter().enumerate() {
        commands.spawn((ChildOf(parent.parent()), pill(index, words, &from, &widths)));
    }
    fan.offered = offered;
    fan.from = Some(from);
    fan.widths = widths;
    fan.fresh = true;
    if let Ok(mut fill) = fills.get_mut(bubble) {
        fill.set_if_neq(BackgroundColor(colours.button_held));
    }
}

/// Brings each plate out of the bubble ([`furled`]), each a moment after
/// the one before it, and leaves it be once it is out: nothing is written
/// to a fan that has opened.
fn unfurl(
    time: Res<Time<Real>>,
    fan: Res<Fan>,
    mut pills: Query<(Entity, &Pill, &mut Unfurl, &mut UiTransform)>,
    mut commands: Commands,
) {
    let Some(from) = fan.from else {
        return;
    };
    for (entity, pill, mut unfurl, mut transform) in &mut pills {
        unfurl.0 += time.delta_secs();
        let through = ((unfurl.0 - pill.0 as f32 * STAGGER_SECS) / UNFURL_SECS).clamp(0.0, 1.0);
        *transform = furled(pill.0, through, &from, &fan.widths);
        if through >= 1.0 {
            commands.entity(entity).remove::<Unfurl>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_the_file_offers_to_say_is_lines_anyone_could_type_hajun_s_in_the_town() {
        let file = ui();
        // Hajun's own, in his order (2026-11-27).
        assert_eq!(
            file.words.chat.quick,
            [
                "Hi!", "Nice house!", "Follow me!", "Add friend?", "Thanks!", "Give me money.", "No.", "Ok.", "Yes.", "Bye!",
                "Wanna play?"
            ]
        );
        for lines in [Lines::Town, Lines::Games] {
            let quick = list(lines);
            // Something is offered, and every one of the file's is a line
            // as it is written: of what the game writes, not too long, with
            // nothing the word filter would star out; and no more of them
            // than the server takes by number.
            assert!(!quick.is_empty() && quick.len() <= usize::from(rules::QUICK_MOST), "{lines:?}");
            for words in &quick {
                assert_eq!(rules::line(words).as_deref(), Some(words.as_str()), "{words:?}");
            }
            let offered = offered(lines);
            assert_eq!(offered.len(), quick.len());
            for (index, (number, words)) in offered.iter().enumerate() {
                assert_eq!(*number, Quick { lines, line: index as u8 });
                assert_eq!(words_of(*number).as_deref(), Some(words.as_str()));
                // No two the same.
                assert!(!offered[..index].iter().any(|(_, before)| before == words), "{words:?} twice");
            }
            // A number this phone has no words for says nothing.
            assert_eq!(words_of(Quick { lines, line: rules::QUICK_MOST - 1 }), None);
        }
        assert_eq!(lines_for(Venue::Town), Lines::Town);
        assert_eq!(lines_for(Venue::Arena), Lines::Games);
    }

    #[test]
    fn a_plate_is_as_wide_as_the_widest_words_of_its_column() {
        // Wider words are wider; nothing is nothing.
        assert!(em_width("Give me money.") > em_width("No."));
        assert_eq!(em_width(""), 0.0);
        assert!(em_width("안녕") > 0.0, "Hangul from the second font");
        let offered = offered(Lines::Town);
        let widths = widths(&offered);
        assert_eq!(widths.len(), offered.len().div_ceil(ROWS));
        for (index, (_, words)) in offered.iter().enumerate() {
            assert!(widths[index / ROWS] >= em_width(words) * TEXT + 2.0 * PAD, "{words:?}");
        }
    }

    #[test]
    fn the_town_s_plates_fan_up_from_beside_the_bubble_clear_of_the_buttons() {
        let (quick_right, quick_bottom, quick_size) = hud::QUICK_BUTTON;
        // A disc's middle, from the box's right and bottom, and its radius.
        let disc = |right: f32, bottom: f32, size: f32| (Vec2::new(right + size * 0.5, bottom + size * 0.5), size * 0.5);
        let mut discs = vec![
            disc(quick_right, quick_bottom, quick_size),
            // Your bag's button, up and to the left of the jump button.
            disc(18.5, 18.5, 14.0),
        ];
        // The chat's keyboard, were lines typed again.
        if rules::TYPED {
            let (chat_right, chat_bottom, chat_size) = hud::CHAT_BUTTON;
            discs.push(disc(chat_right, chat_bottom, chat_size));
            discs.push(disc(chat_right + hud::CHAT_STEPS_IN, chat_bottom, chat_size));
        }
        let from = FanFrom::town();
        let offered = offered(Lines::Town);
        let widths = widths(&offered);
        for (index, _) in offered.iter().enumerate() {
            let (right, bottom) = place(index, &from, &widths);
            let wide = widths[index / ROWS];
            // Over the one under it in its column, with room between, and
            // its right end no further left than that one's: they lean in
            // over the bubble.
            if index % ROWS > 0 {
                let (under_right, under_bottom) = place(index - 1, &from, &widths);
                assert!(bottom >= under_bottom + HEIGHT + 0.9, "{index}");
                assert!(right <= under_right, "{index}");
            }
            // A column clear of the one to its right.
            if index >= ROWS {
                let (beside, _) = place(index - ROWS, &from, &widths);
                assert!(right >= beside + widths[index / ROWS - 1] + 1.0, "{index}");
            }
            // Clear of every button there: the nearest point of the plate
            // to each disc's middle is further from it than the disc
            // reaches.
            for &(middle, radius) in &discs {
                let nearest = Vec2::new(middle.x.clamp(right, right + wide), middle.y.clamp(bottom, bottom + HEIGHT));
                let gap = nearest.distance(middle) - radius;
                assert!(gap > 1.0, "{index}: {gap} from the button at {middle}");
            }
            // On the screen, under its top, lying on its side.
            let top = crate::JUMP_BOTTOM_VH + bottom + HEIGHT;
            assert!(top < 100.0 - hud::EDGE, "{index}: its top is {top} up");
        }
        // Out of the bubble: to begin with a plate is small and its right
        // end is at the bubble's middle; at the end it is itself, where it
        // belongs, and it only ever grows on the way.
        for index in 0..offered.len() {
            let (right, bottom) = place(index, &from, &widths);
            let start = furled(index, 0.0, &from, &widths);
            assert_eq!(start.scale, Vec2::splat(FROM_SCALE));
            let (Val::VMin(across), Val::VMin(down)) = (start.translation.x, start.translation.y) else {
                panic!("{:?}", start.translation);
            };
            assert!((right - across - (quick_right + quick_size * 0.5)).abs() < 1e-4);
            assert!((bottom + HEIGHT * 0.5 - down - (quick_bottom + quick_size * 0.5)).abs() < 1e-4);
            let out = furled(index, 1.0, &from, &widths);
            assert_eq!(out.scale, Vec2::ONE);
            assert_eq!(out.translation, Val2::new(Val::VMin(0.0), Val::VMin(0.0)));
            let mut scale = 0.0;
            for step in 0..=10 {
                let now = furled(index, step as f32 / 10.0, &from, &widths).scale.x;
                assert!(now >= scale, "{index} at {step}");
                scale = now;
            }
        }
    }

    #[test]
    fn a_small_bubble_s_plates_stand_up_from_its_foot_to_its_left_on_the_screen() {
        for bubble in [hud::CHAT_BUTTON, (20.0, 2.5, 9.0)] {
            let from = FanFrom::beside(bubble);
            let offered = offered(Lines::Games);
            let widths = widths(&offered);
            for index in 0..offered.len() {
                let (right, bottom) = place(index, &from, &widths);
                // Left of the bubble, never over it.
                assert!(right >= bubble.0 + bubble.2 + 1.0, "{index}");
                let top = crate::JUMP_BOTTOM_VH + bottom + HEIGHT;
                assert!(top < 100.0 - hud::EDGE, "{index}: its top is {top} up");
            }
        }
    }
}
