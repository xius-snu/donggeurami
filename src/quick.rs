//! Things to say at a press: the quick chat (since 2026-11-14; Hajun: "the
//! current larger chat icon button should be used for a quick chat selector
//! for preset chat messages that flannel out", which I take for "fan out";
//! until that day the big speech bubble over the town's jump button took up
//! the chat's box to type into, which the small keyboard over it does now,
//! `hud::ChatButton`).
//!
//! **A press on the big speech bubble** (`hud::QuickChatButton`) fans out
//! what there is to say ([`offered`]: the file's `words.chat.quick`, up to
//! [`MOST`]), each on a plate of its own ([`pill`]), one over another up the
//! screen from beside the bubble, their right ends on a curve that leans
//! in over it ([`place`]), each coming out of the bubble a moment after
//! the one under it ([`unfurl`]); the bubble is lit while they are out. **A
//! press on one says it**, as if it had been typed (`chat::Say`: online
//! everyone in your room is told, you among them; offline it is written in
//! your own chat), and the fan shuts. A press on the bubble again shuts it
//! too, and so does one anywhere else, which is whatever it landed on's as
//! well, the stick's or a button's ([`shut_under_a_press`]), as the chat's
//! box is let go of. You can walk with it out.
//!
//! It is the town's alone: Bridge Wars has no big speech bubble (the
//! bridge button is about there), and nothing was asked of it. Where the
//! chat is not shown (House Builder's islands) the bubble does nothing, as
//! it did nothing there before. The server takes a line from it as any
//! other, a few at once and then one every second and a half
//! (`roundtown_net::chat::BURST`): leaning on it says no more than typing
//! would.
//!
//! How it fans, how many, how big and how fast are mine: Hajun's words for
//! all of it are the ones above. What is offered and its colours are the
//! file's (`words.chat.quick`, `colours.chat.quick`, `quick_text`).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use roundtown_net::chat as rules;

use crate::chat::{self, Say};
use crate::hud::{self, DialogUp, QuickChatButton, RoundTheJump, TakesPress};
use crate::island::Venue;
use crate::signup::Signup;
use crate::ui::ui;
use crate::{CloseUp, Player};

/// How many things are offered at the most.
pub(crate) const MOST: usize = 6;
/// Sizes, as shares of the short side of the screen: how tall a plate is,
/// how far each is over the one under it, how big its words are, and the
/// room either side of them.
const HEIGHT: f32 = 5.8;
const PITCH: f32 = 6.9;
const TEXT: f32 = 2.8;
const PAD: f32 = 2.2;
/// Where the plates are, in the box round the jump button, in shares of
/// the short side from its right and its bottom ([`place`]): how far up
/// the first is, just over the face, whose top is 32.5 up; where the right
/// end of the last is, just left of the chat's small button, which reaches
/// 7 in; and how much further left than that the first one's is, clear of
/// the bubble, which reaches 15.
const FIRST_BOTTOM: f32 = 34.0;
const RIGHT_END: f32 = 8.5;
const SWEEP: f32 = 8.0;
/// How long a plate takes to come out of the bubble, how long after the
/// one under it each sets off, in seconds, and how small it is as it
/// leaves, as a share of itself.
const UNFURL_SECS: f32 = 0.16;
const STAGGER_SECS: f32 = 0.035;
const FROM_SCALE: f32 = 0.3;

/// The fan: what is offered while it is out, nothing while it is shut; and
/// what has been pressed since [`act`] last looked, the bubble, one of the
/// plates, or anywhere else.
#[derive(Resource, Default)]
struct Fan {
    offered: Vec<String>,
    toggled: bool,
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

/// What there is to say: the file's, each as the line it would be said as
/// (`roundtown_net::chat::line`: cleaned, and not too long), those that are
/// lines at all, and no more than [`MOST`] of them.
fn offered() -> Vec<String> {
    let file = ui();
    file.words.chat.quick.iter().filter_map(|words| rules::line(words)).take(MOST).collect()
}

/// Where the plate numbered `index` is, as its right and bottom offsets in
/// the box round the jump button, in shares of the short side: one over
/// another from just over the face, the first's right end clear of the
/// bubble beside it and each one's after a little further right, over the
/// bubble, by less each time, like the sticks of a fan.
fn place(index: usize) -> (f32, f32) {
    let through = (index as f32 / (MOST - 1) as f32).min(1.0);
    (RIGHT_END + SWEEP * (1.0 - through).powi(2), FIRST_BOTTOM + index as f32 * PITCH)
}

/// A plate `through` its way out of the bubble, from 0 to 1: small, and
/// with its right end at the bubble's middle, to begin with; in its own
/// place and its own size at the end, quickly at first and then slower.
fn furled(index: usize, through: f32) -> UiTransform {
    let (right, bottom) = place(index);
    let (quick_right, quick_bottom, size) = hud::QUICK_BUTTON;
    let eased = 1.0 - (1.0 - through.clamp(0.0, 1.0)).powi(3);
    let left = 1.0 - eased;
    // From the plate to the bubble's middle: to the right, and down the
    // screen.
    let to_bubble = Vec2::new(
        right - (quick_right + size * 0.5),
        bottom + HEIGHT * 0.5 - (quick_bottom + size * 0.5),
    );
    UiTransform {
        translation: Val2::new(Val::VMin(to_bubble.x * left), Val::VMin(to_bubble.y * left)),
        scale: Vec2::splat(FROM_SCALE + (1.0 - FROM_SCALE) * eased),
        rotation: Rot2::IDENTITY,
    }
}

/// One of the plates: `words` on the buttons' grey, a little more solid,
/// with rounded ends, as wide as its words, where [`place`] puts it. It
/// takes a press for itself.
fn pill(index: usize, words: &str) -> impl Bundle {
    let colours = ui().colours;
    let (right, bottom) = place(index);
    (
        Pill(index),
        Unfurl(0.0),
        TakesPress,
        Button,
        Node {
            position_type: PositionType::Absolute,
            right: Val::VMin(right),
            bottom: Val::VMin(bottom),
            height: Val::VMin(HEIGHT),
            padding: UiRect::horizontal(Val::VMin(PAD)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(Val::VMin(HEIGHT * 0.5)),
            ..default()
        },
        BackgroundColor(colours.chat.quick),
        furled(index, 0.0),
        children![(
            Text::new(words.to_string()),
            crate::font::sized(TEXT),
            TextColor(colours.chat.quick_text),
            TextShadow {
                offset: Vec2::new(0.0, 2.0),
                color: colours.text_shadow,
            },
            // On one line, however long: the plate is as wide as its words.
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            Pickable::IGNORE,
        )],
    )
}

/// A press, as it goes down, on the bubble or on one of the plates: for
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
        fan.toggled = true;
    }
}

/// A finger put down anywhere but on the fan, while it is out, shuts it in
/// the frame it lands, and is whatever it landed on's as well, as the
/// chat's box is let go of (`chat::shut_under_a_press`).
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
/// by a press on the bubble, on a plate, or anywhere else; and shuts it
/// when anything comes up over the screen, when you leave where the chat
/// is, and when the buttons it hangs among are put up afresh.
#[allow(clippy::too_many_arguments)]
fn act(
    mut fan: ResMut<Fan>,
    (dialog, close_up, signup): (Res<DialogUp>, Res<CloseUp>, Res<Signup>),
    me: Query<&Venue, With<Player>>,
    boxes: Query<Entity, With<RoundTheJump>>,
    pills: Query<Entity, With<Pill>>,
    mut bubbles: Query<&mut BackgroundColor, With<QuickChatButton>>,
    mut says: MessageWriter<Say>,
    mut commands: Commands,
) {
    // Most frames: shut, and nothing pressed.
    if !fan.out() && !fan.toggled && fan.picked.is_none() && !fan.shut {
        return;
    }
    let (toggled, picked, shut) = (
        std::mem::take(&mut fan.toggled),
        fan.picked.take(),
        std::mem::take(&mut fan.shut),
    );
    let free = me.single().is_ok_and(|&venue| chat::shown_on(venue))
        && !dialog.0
        && !signup.up()
        && close_up.on.is_none();
    let colours = ui().colours.hud;
    if fan.out() {
        let gone = !std::mem::take(&mut fan.fresh) && pills.is_empty();
        if let Some(words) = picked.and_then(|index| fan.offered.get(index)) {
            says.write(Say(words.clone()));
        } else if !(toggled || shut || !free || gone) {
            return;
        }
        fan.offered.clear();
        for pill in &pills {
            commands.entity(pill).despawn();
        }
        for mut fill in &mut bubbles {
            fill.set_if_neq(BackgroundColor(colours.button));
        }
        return;
    }
    if !(toggled && free) {
        return;
    }
    let Ok(round) = boxes.single() else {
        return;
    };
    let offered = offered();
    for (index, words) in offered.iter().enumerate() {
        commands.spawn((ChildOf(round), pill(index, words)));
    }
    if offered.is_empty() {
        return;
    }
    fan.offered = offered;
    fan.fresh = true;
    for mut fill in &mut bubbles {
        fill.set_if_neq(BackgroundColor(colours.button_held));
    }
}

/// Brings each plate out of the bubble ([`furled`]), each a moment after
/// the one under it, and leaves it be once it is out: nothing is written
/// to a fan that has opened.
fn unfurl(
    time: Res<Time<Real>>,
    mut pills: Query<(Entity, &Pill, &mut Unfurl, &mut UiTransform)>,
    mut commands: Commands,
) {
    for (entity, pill, mut unfurl, mut transform) in &mut pills {
        unfurl.0 += time.delta_secs();
        let through = ((unfurl.0 - pill.0 as f32 * STAGGER_SECS) / UNFURL_SECS).clamp(0.0, 1.0);
        *transform = furled(pill.0, through);
        if through >= 1.0 {
            commands.entity(entity).remove::<Unfurl>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_the_file_offers_to_say_is_lines_anyone_could_type() {
        let file = ui();
        let quick = &file.words.chat.quick;
        // Something is offered, and every one of the file's is a line as
        // it is written: of what the game writes, not too long, with
        // nothing the word filter would star out.
        assert!(!quick.is_empty());
        for words in quick {
            assert_eq!(rules::line(words).as_deref(), Some(words.as_str()), "{words:?}");
        }
        let offered = offered();
        assert_eq!(offered.len(), quick.len().min(MOST));
        assert_eq!(offered[0], quick[0]);
        // No two the same.
        for (index, words) in offered.iter().enumerate() {
            assert!(!offered[..index].contains(words), "{words:?} twice");
        }
    }

    #[test]
    fn the_plates_fan_up_from_beside_the_bubble_clear_of_the_buttons() {
        let (quick_right, quick_bottom, quick_size) = hud::QUICK_BUTTON;
        let (chat_right, chat_bottom, chat_size) = hud::CHAT_BUTTON;
        // A disc's middle, from the box's right and bottom, and its radius.
        let disc = |right: f32, bottom: f32, size: f32| (Vec2::new(right + size * 0.5, bottom + size * 0.5), size * 0.5);
        let discs = [
            disc(quick_right, quick_bottom, quick_size),
            disc(chat_right, chat_bottom, chat_size),
            // The face, up and to the left of the jump button.
            disc(18.5, 18.5, 14.0),
        ];
        let mut last = (f32::INFINITY, f32::NEG_INFINITY);
        for index in 0..MOST {
            let (right, bottom) = place(index);
            // Each over the last, with room between, and its right end no
            // further left than the last one's: they lean in over the
            // bubble.
            assert!(bottom >= last.1 + HEIGHT + 0.9, "{index}: {bottom} over {}", last.1);
            assert!(right <= last.0, "{index}: {right} after {}", last.0);
            last = (right, bottom);
            // Clear of every button there: the nearest point of the plate
            // to each disc's middle is further from it than the disc
            // reaches. A plate goes from `right` leftward, as far as its
            // words take it: say 40 shares.
            for (middle, radius) in discs {
                let nearest = Vec2::new(middle.x.clamp(right, right + 40.0), middle.y.clamp(bottom, bottom + HEIGHT));
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
        for index in 0..MOST {
            let (right, bottom) = place(index);
            let from = furled(index, 0.0);
            assert_eq!(from.scale, Vec2::splat(FROM_SCALE));
            let (Val::VMin(across), Val::VMin(down)) = (from.translation.x, from.translation.y) else {
                panic!("{:?}", from.translation);
            };
            assert!((right - across - (quick_right + quick_size * 0.5)).abs() < 1e-4);
            assert!((bottom + HEIGHT * 0.5 - down - (quick_bottom + quick_size * 0.5)).abs() < 1e-4);
            let out = furled(index, 1.0);
            assert_eq!(out.scale, Vec2::ONE);
            assert_eq!(out.translation, Val2::new(Val::VMin(0.0), Val::VMin(0.0)));
            let mut scale = 0.0;
            for step in 0..=10 {
                let now = furled(index, step as f32 / 10.0).scale.x;
                assert!(now >= scale, "{index} at {step}");
                scale = now;
            }
        }
    }
}
