//! Rating the house of someone you visit (since 2026-11-27; Hajun: "after
//! 15 seconds of staying on the plot, if you've never rated that persons
//! plot's version before (if theyve modified the house after uve rated it,
//! u can rate it again) u will get 5 stars at the bottom middle of ur
//! screen, and when u tap it/select ur star, it will bring up a popup menu,
//! with the star count that u tapped on the top (u can tap again to modify
//! ur star betwen 0.5~5star with 0.5 increment) and also a list of rating
//! adjectives that u can choose like "cute" "cool" "interesting" "pretty"
//! "fancy" "good view" "amazing" "ugly" etc. and u can choose 0~ or up to 3
//! adjectives, and then at the bottom left u have cancel button in red and
//! submit button in greeen on right").
//!
//! Whose plot you stand on, and for how long, is the server's to say
//! (`roundtown_net::house`, `server/src/house.rs`): it offers the stars
//! ([`Rating::may`]) and takes them back as you walk off, and online alone,
//! there being nobody else's land offline.
//!
//! **The stars** ([`spawn_bar`]) stand along the bottom of the screen in
//! its middle, on a plate of the buttons' grey with a line over them ("Rate
//! {name}'s house", mine), for as long as they are offered and nothing is
//! up over the screen. Each star is two halves to press: the left half is
//! that star less a half, the right that star whole, so that a press opens
//! **the window** ([`draw_window`]) with that many already given: the five
//! stars along its top, as big as a thumb, pressed again by halves to
//! change them; under them the words to choose from
//! (`words.house.adjectives`, sixteen, the eight of Hajun's first and eight
//! of mine), up to three, a press choosing one and another letting it go;
//! and at its foot Cancel in the window's red, left, and Submit in its
//! green, right. A press outside it is Cancel. Submit sends the rating
//! (`house::Ask::Rate`); the server keeps it, and the stars go.
//!
//! What the server says to the house's owner of it, and of a visit, while
//! their app is open, is said across their screen here too ([`hear`]):
//! "{name} is visiting your house. Go say hi!", "Someone just rated your
//! house!"; and to whoever rated, "Thanks for rating!". Away from the app,
//! their phone is pushed it (`push`).

use bevy::prelude::*;
use roundtown_net::house::{self, Ask as HouseAsk, Rateable, Tell as HouseTell};
use roundtown_net::{Ask, Tell};

use crate::builder::{self, DIALOG_LAYER, Fill, StarImage};
use crate::hud::{self, DialogUp, TakesPress};
use crate::island::Venue;
use crate::net::Online;
use crate::signup::Signup;
use crate::ui::{UiChanged, fill, ui};
use crate::window::{self, Look};
use crate::{CloseUp, Player};

/// The stars along the bottom: how big each is, the room between them, how
/// big the line over them is, and how far up from the bottom of the screen
/// the plate stands, as shares of the short side.
const BAR_STAR: f32 = 7.0;
const BAR_GAP: f32 = 0.8;
const BAR_TEXT: f32 = 2.6;
const BAR_UP: f32 = 3.0;
/// The window: how wide, its stars and the room between them, the words to
/// choose from (how wide each, how tall, how big its words and the room
/// between them) and the line over them.
const WINDOW_WIDE: f32 = 84.0;
const BIG_STAR: f32 = 9.5;
const BIG_GAP: f32 = 1.2;
const WORD_WIDE: f32 = 17.6;
const WORD_HIGH: f32 = 5.6;
const WORD_TEXT: f32 = 2.7;
const WORD_GAP: f32 = 1.2;
const LINE_TEXT: f32 = 2.8;
/// A line across the screen: how big, how far down, and for how long.
const TOAST_TEXT: f32 = 4.0;
const TOAST_DOWN: f32 = 58.0;
const TOAST_SECS: f32 = 3.5;

/// What the server last said you may rate, and the window while it is up.
#[derive(Resource, Default)]
pub(crate) struct Rating {
    pub(crate) may: Option<Rateable>,
    open: Option<Choice>,
}

/// A rating being chosen in the window: whose house, the name on it, its
/// halves of a star, and its words by their numbers, in the order chosen.
#[derive(Clone, Debug, PartialEq)]
struct Choice {
    owner: u64,
    halves: u8,
    words: Vec<u8>,
}

impl Choice {
    /// Chooses the word `word`, or lets it go if it was chosen: no more than
    /// `house::WORDS_MOST`, a fourth doing nothing.
    fn toggle(&mut self, word: u8) {
        if let Some(at) = self.words.iter().position(|&chosen| chosen == word) {
            self.words.remove(at);
        } else if self.words.len() < house::WORDS_MOST && usize::from(word) < house::WORDS {
            self.words.push(word);
        }
    }
}

/// What a press on something of the rating's does.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub(crate) enum RateTap {
    /// One of the stars along the bottom: opens the window with so many
    /// halves given.
    Bar(u8),
    /// One of the window's stars: so many halves.
    Star(u8),
    Word(u8),
    Cancel,
    Submit,
}

/// The presses on the rating since [`act`] last looked.
#[derive(Resource, Default)]
struct Taps(Vec<RateTap>);

/// The plate of stars along the bottom, and the window.
#[derive(Component)]
struct Bar;
#[derive(Component)]
struct Window;

/// A line across the screen, and how long it has been up.
#[derive(Component)]
struct Toast(f32);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Rating>()
        .init_resource::<Taps>()
        .add_observer(press)
        .add_systems(
            Update,
            (hear, say_so_on_coming_online, act, show_bar, draw_window, clear_toasts).chain(),
        );
}

/// What the server says about houses that is the rating's: the stars
/// offered and taken back, and what is said across the screen.
fn hear(mut tells: MessageReader<Tell>, online: Res<Online>, mut rating: ResMut<Rating>, toasts: Query<Entity, With<Toast>>, mut commands: Commands) {
    let mut said = None;
    for tell in tells.read() {
        let Tell::House(told) = tell else {
            continue;
        };
        let ui = ui();
        let words = &ui.words.house;
        match told {
            HouseTell::MayRate(may) => {
                if rating.may != *may {
                    rating.may.clone_from(may);
                }
            }
            HouseTell::Thanks => said = Some(words.thanks.clone()),
            HouseTell::Visiting { name } => said = Some(fill(&words.visiting, &[("name", name)])),
            HouseTell::Rated => said = Some(words.rated.clone()),
            HouseTell::Stats(_) | HouseTell::Board(_) => {}
        }
    }
    // Offline, nothing is offered.
    if !online.is() && (rating.may.is_some() || rating.open.is_some()) {
        rating.may = None;
    }
    if let Some(said) = said {
        for before in &toasts {
            commands.entity(before).despawn();
        }
        commands.spawn((
            Toast(0.0),
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                top: Val::Vh(TOAST_DOWN),
                justify_content: JustifyContent::Center,
                ..default()
            },
            GlobalZIndex(DIALOG_LAYER + 6),
            Pickable::IGNORE,
            children![hud::words(said, TOAST_TEXT, ui().colours.text)],
        ));
    }
}

/// Takes each line across the screen away once it has been up its while.
fn clear_toasts(time: Res<Time>, mut toasts: Query<(Entity, &mut Toast)>, mut commands: Commands) {
    for (entity, mut toast) in &mut toasts {
        toast.0 += time.delta_secs();
        if toast.0 > TOAST_SECS {
            commands.entity(entity).despawn();
        }
    }
}

/// Tells a server this device has just come online with that it shows
/// houses' stars and words (`house::Ask::Watch`).
fn say_so_on_coming_online(online: Res<Online>, mut was: Local<bool>, mut asks: MessageWriter<Ask>) {
    let now = online.is();
    if now && !*was {
        asks.write(Ask::House(HouseAsk::Watch));
    }
    *was = now;
}

/// Whatever of the rating's was pressed, to be acted on ([`act`]): only
/// what the press landed on.
fn press(click: On<Pointer<Click>>, taps: Query<&RateTap>, mut pressed: ResMut<Taps>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(&tap) = taps.get(click.entity) {
        pressed.0.push(tap);
    }
}

/// Does what was pressed: a star along the bottom opens the window, the
/// window's own change what is chosen, Cancel shuts it, Submit sends the
/// rating and shuts it. Escape on desktop is Cancel.
fn act(
    mut taps: ResMut<Taps>,
    mut rating: ResMut<Rating>,
    mut up: ResMut<DialogUp>,
    keyboard: Res<ButtonInput<KeyCode>>,
    online: Res<Online>,
    mut asks: MessageWriter<Ask>,
) {
    let mut taps = std::mem::take(&mut taps.0);
    // Escape on desktop, and the connection gone, are Cancel.
    if rating.open.is_some() && (keyboard.just_pressed(KeyCode::Escape) || !online.is()) {
        taps.push(RateTap::Cancel);
    }
    for tap in taps {
        match tap {
            RateTap::Bar(halves) => {
                let Some(may) = rating.may.clone() else {
                    continue;
                };
                if rating.open.is_none() && !up.0 {
                    rating.open = Some(Choice {
                        owner: may.owner,
                        halves,
                        words: Vec::new(),
                    });
                    up.0 = true;
                }
            }
            RateTap::Star(halves) => {
                if let Some(open) = rating.open.as_mut() {
                    open.halves = halves;
                }
            }
            RateTap::Word(word) => {
                if let Some(open) = rating.open.as_mut() {
                    open.toggle(word);
                }
            }
            RateTap::Cancel => {
                if rating.open.take().is_some() {
                    up.0 = false;
                }
            }
            RateTap::Submit => {
                if let Some(open) = rating.open.take() {
                    asks.write(Ask::House(HouseAsk::Rate {
                        owner: open.owner,
                        halves: open.halves,
                        words: open.words,
                    }));
                    up.0 = false;
                }
            }
        }
    }
}

/// How much of the star numbered `star` (from 0) `halves` fills: none, half
/// or all of it.
pub(crate) fn filled(star: u8, halves: u8) -> f32 {
    (f32::from(halves) - 2.0 * f32::from(star)).clamp(0.0, 2.0) / 2.0
}

/// A star `size` across, under `parent`: `fill` of it in the given star's
/// colour from the left, and the rest in the colour of one not given. With
/// `halves`, its left half and its right are pressed, each for what it
/// says. Shared with the House tab of the friends' panel, whose stars are
/// only shown.
pub(crate) fn spawn_star(commands: &mut Commands, parent: Entity, image: &Handle<Image>, size: f32, fill: f32, halves: Option<(RateTap, RateTap)>) {
    let colours = ui().colours.house;
    let star = commands
        .spawn((
            ChildOf(parent),
            Node {
                width: Val::VMin(size),
                height: Val::VMin(size),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
            children![
                (
                    ImageNode::new(image.clone()).with_color(colours.star_off),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::VMin(size),
                        height: Val::VMin(size),
                        ..default()
                    },
                    Pickable::IGNORE,
                ),
                // The given part, cut off where it ends.
                (
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(fill * 100.0),
                        height: Val::Percent(100.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![(
                        ImageNode::new(image.clone()).with_color(colours.star),
                        Node {
                            width: Val::VMin(size),
                            height: Val::VMin(size),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        Pickable::IGNORE,
                    )],
                ),
            ],
        ))
        .id();
    // The two halves, over the rest, each taking its press for itself.
    let Some((left, right)) = halves else {
        return;
    };
    let row = commands
        .spawn((
            ChildOf(star),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for tap in [left, right] {
        commands.spawn((
            ChildOf(row),
            tap,
            TakesPress,
            Button,
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ));
    }
}

/// Puts the stars up along the bottom while you may rate a house and
/// nothing is up over the screen, in the town, and takes them away
/// otherwise; draws them again as the house or the file changes.
#[allow(clippy::too_many_arguments)]
fn show_bar(
    rating: Res<Rating>,
    changed: Res<UiChanged>,
    (dialog, signup, close_up): (Res<DialogUp>, Res<Signup>, Res<CloseUp>),
    me: Query<&Venue, With<Player>>,
    bars: Query<Entity, With<Bar>>,
    star_image: Option<Res<StarImage>>,
    mut drawn_for: Local<Option<Rateable>>,
    mut commands: Commands,
) {
    let wanted = rating
        .may
        .clone()
        .filter(|_| !dialog.0 && !signup.up() && close_up.on.is_none() && me.single().is_ok_and(|&venue| venue == Venue::Town));
    let restyle = changed.is_changed() && !changed.is_added();
    if *drawn_for == wanted && !restyle && (wanted.is_none() || !bars.is_empty()) {
        return;
    }
    for bar in &bars {
        commands.entity(bar).despawn();
    }
    drawn_for.clone_from(&wanted);
    let (Some(may), Some(star_image)) = (wanted, star_image) else {
        return;
    };
    let ui = ui();
    let colours = ui.colours;
    let plate = commands
        .spawn((
            Bar,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                bottom: Val::VMin(BAR_UP),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let inside = commands
        .spawn((
            ChildOf(plate),
            TakesPress,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(0.6),
                padding: UiRect::axes(Val::VMin(2.2), Val::VMin(1.2)),
                border_radius: BorderRadius::all(Val::VMin(2.0)),
                ..default()
            },
            BackgroundColor(colours.house.plate),
            children![hud::words(fill(&ui.words.house.rate, &[("name", &may.name)]), BAR_TEXT, colours.text)],
        ))
        .id();
    let row = commands
        .spawn((
            ChildOf(inside),
            Node {
                column_gap: Val::VMin(BAR_GAP),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for number in 0..5u8 {
        let halves = (RateTap::Bar(2 * number + 1), RateTap::Bar(2 * number + 2));
        spawn_star(&mut commands, row, &star_image.0, BAR_STAR, 0.0, Some(halves));
    }
}

/// Puts the window up while a rating is being chosen, drawn again whole as
/// what is chosen changes, and takes it down once it is shut.
fn draw_window(
    rating: Res<Rating>,
    changed: Res<UiChanged>,
    windows: Query<Entity, With<Window>>,
    star_image: Option<Res<StarImage>>,
    mut commands: Commands,
) {
    let restyle = changed.is_changed() && !changed.is_added();
    if !rating.is_changed() && !restyle {
        return;
    }
    for window in &windows {
        commands.entity(window).despawn();
    }
    let (Some(open), Some(star_image)) = (&rating.open, star_image) else {
        return;
    };
    let ui = ui();
    let (colours, said) = (&ui.colours, &ui.words.house);
    // Over the whole screen, a press on it outside the window being Cancel.
    let root = commands
        .spawn((
            Window,
            RateTap::Cancel,
            TakesPress,
            Button,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let pane = commands
        .spawn((
            ChildOf(root),
            window::pane(Node {
                width: Val::VMin(WINDOW_WIDE),
                padding: UiRect::all(Val::VMin(window::PAD)),
                row_gap: Val::VMin(2.0),
                align_items: AlignItems::Center,
                ..default()
            }),
        ))
        .id();
    // The stars, along its top.
    let stars = commands
        .spawn((
            ChildOf(pane),
            Node {
                column_gap: Val::VMin(BIG_GAP),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for number in 0..5u8 {
        let halves = (RateTap::Star(2 * number + 1), RateTap::Star(2 * number + 2));
        spawn_star(&mut commands, stars, &star_image.0, BIG_STAR, filled(number, open.halves), Some(halves));
    }
    // How many, in numbers, beside them.
    commands.spawn((
        ChildOf(stars),
        Node {
            margin: UiRect::left(Val::VMin(1.6)),
            ..default()
        },
        Pickable::IGNORE,
        children![window::words(format!("{:.1}", f32::from(open.halves) / 2.0), window::TITLE, colours.window.title)],
    ));
    commands.spawn((ChildOf(pane), window::words(said.pick_words.clone(), LINE_TEXT, colours.window.status)));
    // The words to choose from, four to a row.
    let grid = commands
        .spawn((
            ChildOf(pane),
            Node {
                width: Val::Percent(100.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                column_gap: Val::VMin(WORD_GAP),
                row_gap: Val::VMin(WORD_GAP),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for (number, word) in said.adjectives.iter().take(house::WORDS).enumerate() {
        let chosen = open.words.contains(&(number as u8));
        let house_colours = colours.house;
        let (idle, lit, ink) = if chosen {
            (house_colours.word_on, house_colours.word_on, house_colours.word_on_text)
        } else {
            (house_colours.word, house_colours.word_lit, house_colours.word_text)
        };
        commands.spawn((
            ChildOf(grid),
            RateTap::Word(number as u8),
            TakesPress,
            Button,
            Node {
                width: Val::VMin(WORD_WIDE),
                height: Val::VMin(WORD_HIGH),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::VMin(WORD_HIGH * 0.5)),
                ..default()
            },
            BackgroundColor(idle),
            Fill { idle, lit },
            children![builder::label(word.clone(), WORD_TEXT, ink)],
        ));
    }
    // Cancel at the left of its foot, Submit at the right.
    commands.spawn((
        ChildOf(pane),
        Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            margin: UiRect::top(Val::VMin(0.6)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (RateTap::Cancel, window::choice(said.cancel.clone(), Look::No)),
            (RateTap::Submit, window::choice(said.submit.clone(), Look::Yes)),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_star_is_filled_by_halves() {
        // Three and a half: three whole, a half, and one empty.
        let fills: Vec<f32> = (0..5).map(|star| filled(star, 7)).collect();
        assert_eq!(fills, [1.0, 1.0, 1.0, 0.5, 0.0]);
        assert_eq!((0..5).map(|star| filled(star, 1)).collect::<Vec<_>>(), [0.5, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!((0..5).map(|star| filled(star, 10)).collect::<Vec<_>>(), [1.0; 5]);
        // Each half of each star is a rating there is.
        for number in 0..5u8 {
            assert!(house::halves_ok(2 * number + 1) && house::halves_ok(2 * number + 2));
        }
    }

    #[test]
    fn up_to_three_words_a_press_choosing_one_and_another_letting_it_go() {
        let mut choice = Choice {
            owner: 1,
            halves: 6,
            words: Vec::new(),
        };
        for word in [0, 5, 9, 12] {
            choice.toggle(word);
        }
        assert_eq!(choice.words, [0, 5, 9], "a fourth does nothing");
        choice.toggle(5);
        assert_eq!(choice.words, [0, 9]);
        choice.toggle(12);
        assert_eq!(choice.words, [0, 9, 12]);
        choice.toggle(house::WORDS as u8);
        assert_eq!(choice.words, [0, 9, 12], "no such word");
        assert!(house::chosen(&choice.words).is_some(), "what the server takes");
    }

    #[test]
    fn the_file_has_a_word_for_every_one_the_server_keeps() {
        let file = ui();
        assert_eq!(file.words.house.adjectives.len(), house::WORDS);
        // Hajun's eight first, in his order.
        assert_eq!(
            file.words.house.adjectives[..8],
            ["Cute", "Cool", "Interesting", "Pretty", "Fancy", "Good view", "Amazing", "Ugly"]
        );
    }
}
