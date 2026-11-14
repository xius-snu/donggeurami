//! The look every window over the world has (since 2026-11-14; Hajun: "I
//! think I like the ui design/color choice/ close button layout/ etc. of the
//! friends tab. I want to use this same design style throughout the project.
//! change the other ui panels to match the current friends panel design").
//! It is the friends' panel's, which was drawn from Hajun's own sketches of
//! it (`friendtabdesign1.png` to `4`): the friends' panel (`friends`), the
//! settings' window (`settings`), the game selector (`games`) and Bridge
//! Wars' two shops (`market`) are all made of what is here.
//!
//! **A window** ([`pane`]) is dark and see-through, its corners rounded,
//! and nothing of the screen round it is dimmed. Along its top
//! ([`header`]) are its tabs ([`tab`]), pictures or words, the one that is
//! shown white with a white line under it, or its title; and in its top
//! right corner a bare white cross that shuts it ([`cross`]), with no disc
//! behind it. What is in it is white on the dark and begins at the left: a
//! heading, rows of a picture on a disc ([`avatar`]) with a name over a
//! fainter line, round dark buttons at a row's end ([`round`]), pictures
//! that are pressed with nothing behind them ([`bare`]), and buttons with
//! words on them ([`choice`]), green for yes and a dark red for no. A
//! card that is pressed to choose what is on it, a game or a thing in a
//! shop, is that dark again but see-through ([`card`]; Hajun, of solid
//! ones: "too dark ... should be more transparent, not just become
//! grayer"). **A box that comes up over a window** ([`popup`]) is darker
//! still, the window darkened behind it, and a tap on that dark takes it
//! away.
//!
//! Every colour of it is `assets/ui.ron`'s `colours.window`; the sizes are
//! here, as shares of the short side of the screen, off Hajun's sketches
//! (874 by 402, so a share is four of their pixels). Each thing that is
//! pressed takes its press for itself (`hud::TakesPress`) and carries
//! nothing else: what a press on it does is whoever spawns it to say, with
//! a component of their own beside it.

use bevy::prelude::*;

use crate::builder::{self, Fill};
use crate::hud::TakesPress;
use crate::signup::LiftsForKeyboard;
use crate::ui::ui;

/// The friends' panel's size, which the settings' window has too, and the
/// room between a window's edge and what is in it; how round its corners
/// are, and a card's.
pub(crate) const WIDTH: f32 = 112.0;
pub(crate) const HEIGHT: f32 = 66.0;
pub(crate) const PAD: f32 = 3.6;
pub(crate) const ROUNDED: f32 = 3.0;
pub(crate) const CARD_ROUNDED: f32 = 1.6;
/// A tab along a window's top: how wide it is at the least and how tall,
/// the picture on it or the words, and the line under the one shown.
pub(crate) const TAB_WIDTH: f32 = 19.0;
pub(crate) const TAB_HEIGHT: f32 = 8.2;
pub(crate) const TAB_ICON: f32 = 5.2;
pub(crate) const TAB_TEXT: f32 = 3.3;
pub(crate) const LINE: f32 = 0.4;
/// The cross on a window's corner (8 since 2026-11-13, Hajun: "increase
/// size of the close button slightly"; 7 for its first evening), and how
/// far its middle is in from the window's top and right, which stays where
/// it was whatever its size.
pub(crate) const CLOSE: f32 = 8.0;
pub(crate) const CLOSE_DOWN: f32 = PAD + 3.1;
pub(crate) const CLOSE_IN: f32 = PAD + 2.7;
/// Words: a heading in a list, and a window's title or a box's; a row of
/// a list, the disc with someone's picture, their name and the line under
/// it, and the round buttons at its end, and the small ones beside a
/// heading.
pub(crate) const HEADING: f32 = 3.1;
pub(crate) const TITLE: f32 = 3.6;
pub(crate) const ROW: f32 = 10.4;
pub(crate) const AVATAR: f32 = 7.6;
pub(crate) const NAME: f32 = 3.3;
pub(crate) const STATUS: f32 = 2.3;
pub(crate) const ROW_BUTTON: f32 = 7.6;
pub(crate) const SMALL_BUTTON: f32 = 5.6;
/// A box that comes up over a window: the words of a question in it; a
/// button with words on it, and its words; the box something is typed
/// into, and what is typed; and words that are pressed.
pub(crate) const POPUP_TEXT: f32 = 3.2;
pub(crate) const CHOICE_WIDTH: f32 = 18.0;
pub(crate) const CHOICE_HEIGHT: f32 = 6.4;
pub(crate) const CHOICE_TEXT: f32 = 3.0;
pub(crate) const FIELD_HEIGHT: f32 = 7.4;
pub(crate) const FIELD_TEXT: f32 = 3.0;
pub(crate) const LINK_TEXT: f32 = 2.8;
/// The line along a window's bottom that says what came of something.
pub(crate) const NOTICE_TEXT: f32 = 2.8;

/// A window: the dark, see-through pane, laid out as `node` says, from the
/// top down, with its corners rounded. A press on it is its own.
pub(crate) fn pane(node: Node) -> impl Bundle {
    (
        TakesPress,
        Node {
            flex_direction: FlexDirection::Column,
            border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
            ..node
        },
        BackgroundColor(ui().colours.window.panel),
    )
}

/// The row along a window's top that its tabs stand in, or its title, clear
/// of the cross on the corner.
pub(crate) fn header() -> impl Bundle {
    (
        Node {
            height: Val::VMin(TAB_HEIGHT),
            column_gap: Val::VMin(1.0),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            padding: UiRect::right(Val::VMin(CLOSE + 1.0)),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// One of the tabs along a window's top, with the white line under it if it
/// is the one `shown`: what is on it, a picture or words, in
/// [`tab_ink`], is spawned under it. As wide as [`TAB_WIDTH`], or as its
/// words need.
pub(crate) fn tab(shown: bool) -> impl Bundle {
    let colours = ui().colours.window;
    (
        TakesPress,
        Button,
        Node {
            min_width: Val::VMin(TAB_WIDTH),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::VMin(2.0)),
            border: UiRect::bottom(Val::VMin(LINE)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(if shown { colours.line } else { Color::NONE }),
    )
}

/// The colour of what is on a tab: the one `shown`, and one that is not.
pub(crate) fn tab_ink(shown: bool) -> Color {
    let colours = ui().colours.window;
    if shown { colours.line } else { colours.tab_off }
}

/// The cross in a window's top right corner, which carries `marker`: bare,
/// with no disc coming up behind it under the pointer or a finger (Hajun,
/// 2026-11-13: "i dont want the circle hover/click background that appears
/// when u press the close button ... it should just close it"). Spawned
/// under the window, after whatever else is along its top.
pub(crate) fn cross(image: Handle<Image>, marker: impl Bundle) -> impl Bundle {
    cross_at(image, marker, CLOSE, CLOSE_IN, CLOSE_DOWN)
}

/// A cross `size` across, its middle `left_of` the window's right edge and
/// `down` from its top: for a window whose margin is not [`PAD`], as a
/// shop's is not.
pub(crate) fn cross_at(image: Handle<Image>, marker: impl Bundle, size: f32, left_of: f32, down: f32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            top: Val::VMin(down - size * 0.5),
            right: Val::VMin(left_of - size * 0.5),
            ..default()
        },
        Pickable::IGNORE,
        children![(marker, bare(image, size, false))],
    )
}

/// Words on a window, which has a colour of its own behind them and needs
/// no shadow: on one line, from the left.
pub(crate) fn words(text: impl Into<String>, size: f32, colour: Color) -> impl Bundle {
    (
        Text::new(text),
        crate::font::sized(size),
        TextColor(colour),
        TextLayout::no_wrap(),
        Pickable::IGNORE,
    )
}

/// A picture `size` across, in `tint`.
pub(crate) fn picture(image: Handle<Image>, size: Val, tint: Color) -> impl Bundle {
    (
        ImageNode::new(image).with_color(tint),
        Node {
            width: size,
            height: size,
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// The disc with someone's picture on it, `size` across, and the dot on
/// its corner that says whether they are here.
pub(crate) fn avatar(person: Handle<Image>, size: f32, here: bool) -> impl Bundle {
    let colours = ui().colours.window;
    (
        Node {
            width: Val::VMin(size),
            height: Val::VMin(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(colours.avatar),
        Pickable::IGNORE,
        children![
            picture(person, Val::Percent(64.0), colours.avatar_mark),
            (
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::VMin(-size * 0.03),
                    bottom: Val::VMin(size * 0.02),
                    width: Val::VMin(size * 0.31),
                    height: Val::VMin(size * 0.31),
                    border: UiRect::all(Val::VMin(size * 0.045)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(if here { colours.online } else { colours.offline }),
                BorderColor::all(colours.panel.with_alpha(1.0)),
                Pickable::IGNORE,
            ),
        ],
    )
}

/// A round button with `image` on it, `size` across; or, if it is not
/// `able` to be pressed just now, one that shows so.
pub(crate) fn round(image: Handle<Image>, size: f32, able: bool) -> impl Bundle {
    let colours = ui().colours.window;
    (
        TakesPress,
        Button,
        Node {
            width: Val::VMin(size),
            height: Val::VMin(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(colours.button),
        Fill {
            idle: colours.button,
            lit: if able { colours.button_lit } else { colours.button },
        },
        children![picture(image, Val::Percent(58.0), if able { colours.button_mark } else { colours.button_off })],
    )
}

/// A button with a picture on it and nothing behind it: the cross, the
/// chevron back, the glass, the two sheets. `lit`, a disc comes up behind
/// it under the pointer or a finger; the cross has none.
pub(crate) fn bare(image: Handle<Image>, size: f32, lit: bool) -> impl Bundle {
    let colours = ui().colours.window;
    (
        TakesPress,
        Button,
        Node {
            width: Val::VMin(size),
            height: Val::VMin(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::NONE),
        Fill {
            idle: Color::NONE,
            lit: if lit { colours.button_lit } else { Color::NONE },
        },
        children![picture(image, Val::Percent(72.0), colours.button_mark)],
    )
}

/// How a button with words on it looks. As sketched: green for yes, and a
/// dark red for no. For one that only puts something away, the dark of the
/// round buttons ([`Look::Plain`]); and for one that is not to be had
/// ([`Look::Blocked`]), what a card that cannot be chosen has, all but
/// clear with faint words. Over a box that has come up over a window,
/// which is as dark as the round buttons' dark, the grey of a box that is
/// typed into instead: one that backs out ([`Look::Quiet`]), and one that
/// cannot be pressed just now ([`Look::Greyed`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Look {
    Yes,
    No,
    Plain,
    Blocked,
    Quiet,
    Greyed,
}

impl Look {
    /// The button's colour, at rest and lit under the pointer or a finger,
    /// and the colour of the words on it.
    pub(crate) fn colours(self) -> (Fill, Color) {
        let colours = ui().colours.window;
        let (idle, lit, ink) = match self {
            Look::Yes => (colours.yes, colours.yes_lit, colours.on_choice),
            Look::No => (colours.no, colours.no_lit, colours.on_choice),
            Look::Plain => (colours.button, colours.button_lit, colours.on_choice),
            // Blocked out as a card is that cannot be chosen: all but
            // clear, its words faint.
            Look::Blocked => (colours.card_off, colours.card_off, colours.name_off),
            Look::Quiet => (colours.field, colours.button_lit, colours.on_choice),
            // Faint as the picture is on a round button that cannot be
            // pressed.
            Look::Greyed => (colours.field, colours.field, colours.button_off),
        };
        (Fill { idle, lit }, ink)
    }
}

/// A button with words on it, in `look`.
pub(crate) fn choice(text: impl Into<String>, look: Look) -> impl Bundle {
    choice_wide(text, look, CHOICE_WIDTH)
}

/// A button with words on it, in `look`, `width` across: for words longer
/// than "Cancel".
pub(crate) fn choice_wide(text: impl Into<String>, look: Look, width: f32) -> impl Bundle {
    let (fill, ink) = look.colours();
    (
        TakesPress,
        Button,
        Node {
            width: Val::VMin(width),
            height: Val::VMin(CHOICE_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(0.8)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(fill.idle),
        fill,
        children![builder::label(text, CHOICE_TEXT, ink)],
    )
}

/// Words that are pressed, or, when they are not `able` to be, that only
/// say: "Add friend", "Requested".
pub(crate) fn link(text: impl Into<String>, able: bool) -> impl Bundle {
    let colours = ui().colours.window;
    (
        TakesPress,
        Button,
        Node {
            padding: UiRect::axes(Val::VMin(1.6), Val::VMin(1.2)),
            border_radius: BorderRadius::all(Val::VMin(1.2)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::NONE),
        Fill {
            idle: Color::NONE,
            lit: if able { colours.button_lit } else { Color::NONE },
        },
        children![builder::label(text, LINK_TEXT, if able { colours.link } else { colours.link_off })],
    )
}

/// How a card looks, one that is pressed to choose what is on it (a game,
/// in the selector; a thing, in one of Bridge Wars' shops), if it is
/// `able` to be chosen and if not: its colour, at rest and lit, the colour
/// of its words, and the colour of the smaller words under them.
pub(crate) fn card(able: bool) -> (Fill, Color, Color) {
    let colours = ui().colours.window;
    if able {
        (
            Fill {
                idle: colours.card,
                lit: colours.card_lit,
            },
            colours.name,
            colours.status,
        )
    } else {
        (
            Fill {
                idle: colours.card_off,
                lit: colours.card_off,
            },
            colours.name_off,
            colours.name_off,
        )
    }
}

/// A box up over the window `panel`, `width` across, the window darkened
/// behind it: what goes in it is spawned under what this gives back. The
/// dark carries `back`, which a tap on it is, to take the box away. One
/// that is `typed_into` slides up out from under a phone's keyboard.
pub(crate) fn popup(commands: &mut Commands, panel: Entity, width: f32, typed_into: bool, back: impl Bundle) -> Entity {
    let colours = ui().colours.window;
    let dark = commands
        .spawn((
            ChildOf(panel),
            back,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
                ..default()
            },
            BackgroundColor(colours.scrim),
        ))
        .id();
    // Its own presses are its own wherever it stands: slid up over a
    // phone's keyboard, its top is past the window's.
    let mut within = commands.spawn((
        ChildOf(dark),
        TakesPress,
        Node {
            width: Val::VMin(width),
            flex_direction: FlexDirection::Column,
            row_gap: Val::VMin(2.0),
            padding: UiRect::all(Val::VMin(3.0)),
            border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
            ..default()
        },
        BackgroundColor(colours.popup),
    ));
    // Its top never off the top of the screen: its title and the way back
    // out of it stay in sight, and what does not fit is under the keyboard.
    if typed_into {
        within.insert(LiftsForKeyboard::whole());
    }
    within.id()
}

/// The row the buttons at the foot of such a box stand in, side by side in
/// its middle.
pub(crate) fn choices() -> impl Bundle {
    (
        Node {
            justify_content: JustifyContent::Center,
            column_gap: Val::VMin(3.0),
            ..default()
        },
        Pickable::IGNORE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud;

    #[test]
    fn a_window_fits_a_phone_lying_on_its_side_and_a_tablet() {
        // The short side is 100 shares: a phone's long side is over 200, a
        // 4:3 tablet's 133. A window fits both with room round it.
        assert!(WIDTH < 133.0 - 2.0 * hud::EDGE);
        assert!(HEIGHT < 100.0 - 2.0 * hud::EDGE);
        // A row holds a picture, and round buttons beside the words.
        assert!(AVATAR < ROW && ROW_BUTTON < ROW);
        // The cross, about its middle, is on the window, and the row along
        // the top is clear of it.
        assert!(CLOSE * 0.5 <= CLOSE_IN.min(CLOSE_DOWN));
        assert!(CLOSE_IN + CLOSE * 0.5 <= PAD + CLOSE + 1.0);
    }

    #[test]
    fn a_button_that_cannot_be_pressed_does_not_light_up_and_one_that_can_does() {
        for look in [Look::Yes, Look::No, Look::Plain, Look::Quiet] {
            let (fill, _) = look.colours();
            assert_ne!(fill.idle, fill.lit, "{look:?}");
        }
        for look in [Look::Blocked, Look::Greyed] {
            let (fill, ink) = look.colours();
            assert_eq!(fill.idle, fill.lit, "{look:?}");
            assert!(ink.alpha() < 1.0, "{look:?}: its words are faint");
        }
        // A card that cannot be chosen is fainter than one that can, and
        // does not light up.
        let ((able, ink, _), (unable, faint, _)) = (card(true), card(false));
        assert_ne!(able.idle, able.lit);
        assert_eq!(unable.idle, unable.lit);
        assert!(faint.alpha() < ink.alpha());
    }
}
