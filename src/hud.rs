//! What is drawn over the world, laid out as Hajun sketched it
//! (`assets/UIbuttonlayout.jpg`, 2026-10-31): top left your name on a plate,
//! with a person button and the settings' gear beside it (`settings`) and
//! your balance under; down the right edge a gift, a leaderboard, a group
//! of people and a gamepad; round a phone's jump button a face, a speech
//! bubble and a fist; and headlines across the middle — the town's name, as
//! a welcome, whenever you arrive in the town, and whatever else is
//! [`announce`]d, like House Builder's theme. And how a phone's stick and
//! jump button look (`setup_hud`). Of the buttons only the gear, the stick
//! and the jump button do anything yet (Hajun: "i dont need you to make
//! them functional yet. just make them in the screen").
//!
//! Every button is in one style, the stick's and the jump button's: a
//! see-through grey disc with a whitish picture on it and no rim
//! ([`round_button`], [`TOUCH_DISC`], [`TOUCH_MARK`]). The pictures are drawn
//! in code where they are simple (the jump button's arrow, the gear, the
//! pencil) and Google's Material icons otherwise ([`ICONS`], in
//! `assets/icons/`, Apache 2.0, turned white there for the buttons to tint;
//! Hajun: "search online for that icon + png/transparent, download and
//! modify it").
//!
//! Every button over the world is a `bevy_ui` node pressed through
//! `bevy_picking`'s click, which a mouse and a finger both send, so there is
//! no platform code in them. Each carries [`TakesPress`]; the code that reads
//! fingers and the mouse for everything else asks [`Presses`] first, so that
//! a press on one is not also taken for a look round or a tap on the town.
//!
//! Until 2026-10-31 a button top right took you home and back again; Hajun
//! had it taken out. Your home (`testhomemap.glb`) is still loaded, with
//! nothing that takes you there.

use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::Player;
use crate::island::{Islands, Venue};
use crate::lobby::{Balance, STARTING_BALANCE};
use crate::signup::Signup;

pub(crate) const COIN_ICON: &str = "coin.png";

/// Sizes, as shares of the short side of the screen, so that a phone and a
/// monitor show the same thing: your name on its plate and your balance
/// under it; the round buttons beside your name, the bigger ones down the
/// right edge, and the bigger still round a phone's jump button (Hajun,
/// 2026-10-31: "keep the size of the user icon and the settings icon but
/// increase the size of every button the right hand side both top and
/// bottom"; 8 and 12 until then).
const NAME_TEXT: f32 = 4.2;
const BALANCE_TEXT: f32 = 5.0;
const BALANCE_ICON: f32 = 5.0;
pub(crate) const SMALL_BUTTON: f32 = 8.0;
/// The buttons down the right edge: 9.5 until Hajun asked for "very
/// slightly" bigger (2026-11-01), and that much further in from the top
/// and the right than the name is ([`EDGE`]), "very slightly" more margin.
const SIDE_BUTTON: f32 = 10.5;
const SIDE_INSET: f32 = 1.5;
const ACTION_BUTTON: f32 = 14.0;
/// How much of a round button its picture takes up, as a share of it.
const PICTURE: f32 = 58.0;
pub(crate) const HEADLINE: f32 = 8.0;
pub(crate) const CAPTION: f32 = 4.5;
/// How far in from the top and the sides of the screen the balance sits:
/// further than a phone's rounded corner reaches.
pub(crate) const EDGE: f32 = 4.0;
/// How much of the jump button its arrow takes up, as a share of it.
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
pub(crate) const JUMP_ARROW: f32 = 56.0;
/// Room left under a headline, as a share of the screen's height, which lifts
/// it clear of your own character in the middle.
const ABOVE_YOU: f32 = 36.0;

/// How long a headline takes to come up and to go, in seconds, and how long
/// the welcome is up for altogether.
const HEADLINE_IN: f32 = 0.4;
const HEADLINE_OUT: f32 = 0.8;
const WELCOME_SECS: f32 = 3.5;

/// The soft shadow under every word, so that it reads over sky and grass alike.
const SHADOW: Color = Color::srgba(0.02, 0.12, 0.22, 0.4);
/// The dark glass behind whatever has to stay legible over anything.
pub(crate) const PANEL: Color = Color::srgba(0.04, 0.10, 0.18, 0.42);
pub(crate) const GOLD: Color = Color::srgb(1.0, 0.84, 0.3);
/// The pictures on buttons, and the dark blue of words on gold.
pub(crate) const ICON: Color = Color::srgba(1.0, 1.0, 1.0, 0.94);
pub(crate) const DOOR: Color = Color::srgb(0.16, 0.30, 0.42);
/// Every button, and a phone's stick: a see-through grey, with no rim, a
/// little more solid while the jump button is held; and the picture on each,
/// the stick's knob and the jump button's arrow among them, a see-through
/// white (Hajun, 2026-10-31; "a little greyed", 0.92 at 0.38, until later
/// that day, when Hajun asked for "slightly more whiter"). Your name's plate
/// is the same grey, a little more solid.
pub(crate) const TOUCH_DISC: Color = Color::srgba(0.4, 0.4, 0.4, 0.32);
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
pub(crate) const TOUCH_DISC_HELD: Color = Color::srgba(0.4, 0.4, 0.4, 0.5);
pub(crate) const TOUCH_MARK: Color = Color::srgba(1.0, 1.0, 1.0, 0.52);
const PLATE: Color = Color::srgba(0.4, 0.4, 0.4, 0.45);

/// The pictures on the buttons that are not drawn in code, in
/// `assets/icons/`, white on clear: Google's Material icons (its
/// `LICENSE-material-icons.txt` says which), a person and the settings'
/// gear beside your name, a gamepad down the right edge, and a face and a
/// speech bubble round the jump button; and Font Awesome's gift and
/// comment-dots (`LICENSE-font-awesome.txt`; the ones Hajun's FlutterFlow
/// screenshots showed, 2026-11-01, rendered from their SVGs), the gift and
/// the social chat's bubble down the right edge. The fist with the face
/// and the bubble is drawn in code ([`fist`]). A test keeps every one in
/// the APK. The leaderboard's (`leaderboard.png`), the group of people
/// (`groups.png`) and the boxing glove (`punch.png`) are still beside them,
/// unused since 2026-11-01.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) const ICONS: [&str; 7] = ["person", "settings", "gift", "social", "gamepad", "mood", "chat"];

/// Your balance, top left.
#[derive(Component)]
struct BalanceText;

/// Your name, top left, on its plate.
#[derive(Component)]
struct NamePlate;

/// Words across the middle of the screen, up for a moment, and how long they
/// have been up.
#[derive(Component)]
struct Headline {
    age: f32,
    secs: f32,
}

/// The headline that is the town's name, as a welcome.
#[derive(Component)]
struct Welcome;

/// You have arrived in the town, and are to be welcomed as soon as it has
/// loaded.
#[derive(Resource, Default)]
struct WelcomeDue(bool);

/// Something on the screen that takes a press for itself: every button over
/// the world, the dark behind a dialog, the bubble over whoever you can talk
/// to, and the whole screen while you talk (`talk`). Whatever reads a finger
/// or the mouse for the world asks [`Presses::claimed`] before it takes a
/// press.
#[derive(Component, Default)]
pub(crate) struct TakesPress;

/// Where the things that take a press for themselves are on the screen, as
/// they were last laid out.
#[derive(SystemParam)]
pub(crate) struct Presses<'w, 's> {
    takers: Query<
        'w,
        's,
        (
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static InheritedVisibility,
        ),
        With<TakesPress>,
    >,
}

impl Presses<'_, '_> {
    /// Whether `pos`, in logical window pixels, is on something showing that
    /// takes a press for itself. Tested the way picking tests it, against the
    /// shape each one was actually laid out in, so that the two always agree.
    pub(crate) fn claimed(&self, window: &Window, pos: Vec2) -> bool {
        let pos = pos * window.scale_factor();
        self.takers
            .iter()
            .any(|(node, place, shown)| shown.get() && node.contains_point(*place, pos))
    }
}

/// A dialog is up, over everything else: nothing but its own buttons can be
/// pressed, you stand still, and on desktop the cursor is let go for clicking
/// them.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct DialogUp(pub bool);

/// Something out in the world is waiting on the pointer — a piece being put
/// down in House Builder, to be dragged and confirmed — so on desktop the
/// cursor is let go for as long as it is, the same as for a dialog. Unlike a
/// dialog, you can still walk about.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct WantsPointer(pub bool);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<WelcomeDue>()
        .init_resource::<DialogUp>()
        .init_resource::<WantsPointer>()
        .add_observer(press_punch)
        .add_systems(Startup, show_top_bar)
        .add_systems(
            Update,
            (
                show_name,
                count_balance,
                hide_round_the_jump,
                (arrive, welcome, fade_headlines).chain(),
            ),
        );
}

/// The box round a phone's jump button, with the three buttons about it.
#[derive(Component)]
struct RoundTheJump;

/// The fist among them, which throws a punch (`start_punch` in `src/lib.rs`).
#[derive(Component)]
struct PunchButton;

/// A press on the fist, as it goes down rather than as it comes up, so that
/// quick presses land as quickly as they are made: a punch is asked for.
fn press_punch(
    press: On<Pointer<Press>>,
    fists: Query<(), With<PunchButton>>,
    mut punches: MessageWriter<crate::PunchPressed>,
) {
    if press.button != PointerButton::Primary || press.original_event_target() != press.entity {
        return;
    }
    if fists.contains(press.entity) {
        punches.write(crate::PunchPressed);
    }
}

/// Puts the three buttons round the jump button away while the camera is on
/// someone you talk to (`talk`), as the stick and the jump button are
/// (Hajun, 2026-11-01: "the punch, quickchat, and emote button is not
/// hidden. hide those too").
fn hide_round_the_jump(
    close_up: Res<crate::CloseUp>,
    mut boxes: Query<&mut Visibility, With<RoundTheJump>>,
) {
    let shown = if close_up.on.is_some() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut visibility in &mut boxes {
        visibility.set_if_neq(shown);
    }
}

/// Text in the house style: the default font, sized as a share of the short
/// side of the screen, with a soft shadow so that it reads over sky and grass
/// alike. The shadow is as see-through as the words.
pub(crate) fn words(text: impl Into<String>, size_vmin: f32, colour: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::VMin(size_vmin),
            ..default()
        },
        TextColor(colour),
        TextShadow {
            offset: Vec2::new(0.0, 3.0),
            color: SHADOW.with_alpha(SHADOW.alpha() * colour.alpha()),
        },
        TextLayout::justify(Justify::Center),
        Pickable::IGNORE,
    )
}

/// Everything round the edges of the screen, for as long as the app is open:
/// top left your name, the buttons beside it and your balance; down the
/// right edge four buttons; and round a phone's jump button three more,
/// placed about where it is on every platform, so that the layout can be
/// seen on desktop too.
pub(crate) fn show_top_bar(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let icon = |name: &str| assets.load(format!("icons/{name}.png"));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(EDGE),
                left: Val::VMin(EDGE),
                right: Val::VMin(EDGE),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::VMin(1.4),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|left| {
                // Your name, and beside it the buttons that are about you:
                // a person, and the settings' gear, which `settings` puts
                // after it.
                left.spawn((
                    BarLeft,
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::VMin(1.4),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    row.spawn(name_plate());
                    row.spawn(round_button(icon("person"), SMALL_BUTTON, Node::default()));
                });
                left.spawn(balance(&assets));
            });
            bar.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::VMin(1.6),
                    // A little further in from the corner than the name is.
                    margin: UiRect::new(Val::ZERO, Val::VMin(SIDE_INSET), Val::VMin(SIDE_INSET), Val::ZERO),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|right| {
                // A leaderboard stood between the gift and the group until
                // Hajun had it taken out (2026-11-01), and the group became
                // a speech bubble the same day.
                for name in ["gift", "social", "gamepad"] {
                    right.spawn(round_button(icon(name), SIDE_BUTTON, Node::default()));
                }
            });
        });
    // Round the jump button: a box where it is, and the three about it, each
    // placed from the box's corner, in shares of the short side: the face up
    // and to the left of it, the speech bubble over it and the fist beside
    // it, each about 2.5 shares clear of it, as Hajun sketched them.
    commands
        .spawn((
            RoundTheJump,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Vw(crate::JUMP_RIGHT_VW),
                bottom: Val::Vh(crate::JUMP_BOTTOM_VH),
                width: Val::VMin(crate::JUMP_SIZE_VMIN),
                height: Val::VMin(crate::JUMP_SIZE_VMIN),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|round| {
            let fist = images.add(fist());
            for (name, right, bottom) in [("mood", 18.5, 18.5), ("chat", 1.0, 24.3), ("punch", 24.3, 1.2)] {
                let picture = if name == "punch" { fist.clone() } else { icon(name) };
                let mut button = round.spawn(round_button(
                    picture,
                    ACTION_BUTTON,
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::VMin(right),
                        bottom: Val::VMin(bottom),
                        ..default()
                    },
                ));
                // The fist is the one of the three that does something: a
                // punch, each time it is pressed.
                if name == "punch" {
                    button.insert(PunchButton);
                }
            }
        });
}

/// The row top left: your name, and whatever goes with it.
#[derive(Component)]
pub(crate) struct BarLeft;

/// A round button in the style of the stick and the jump button: a
/// see-through grey disc, `size_vmin` across, with `icon` on it in a
/// see-through white, and no rim; where `place` puts it. It takes a press
/// for itself; what the press does, if anything, is whoever spawns it to
/// say.
pub(crate) fn round_button(icon: Handle<Image>, size_vmin: f32, place: Node) -> impl Bundle {
    (
        TakesPress,
        Button,
        Node {
            width: Val::VMin(size_vmin),
            height: Val::VMin(size_vmin),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..place
        },
        BackgroundColor(TOUCH_DISC),
        children![(
            ImageNode::new(icon).with_color(TOUCH_MARK),
            Node {
                width: Val::Percent(PICTURE),
                height: Val::Percent(PICTURE),
                ..default()
            },
            Pickable::IGNORE,
        )],
    )
}

/// Your name on a plate: the buttons' grey, a little more solid, with its
/// corners rounded a little.
fn name_plate() -> impl Bundle {
    (
        Node {
            padding: UiRect::axes(Val::VMin(2.4), Val::VMin(1.0)),
            border_radius: BorderRadius::all(Val::VMin(1.6)),
            ..default()
        },
        BackgroundColor(PLATE),
        Pickable::IGNORE,
        children![(
            NamePlate,
            Text::new(""),
            TextFont {
                font_size: FontSize::VMin(NAME_TEXT),
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        )],
    )
}

/// Shows your name on its plate, as you are called now: as the app opens,
/// and again when you sign up or change it (`signup`).
fn show_name(
    me: Query<&Name, (With<Player>, Changed<Name>)>,
    mut plates: Query<&mut Text, With<NamePlate>>,
) {
    let Ok(name) = me.single() else {
        return;
    };
    for mut text in &mut plates {
        if text.0 != name.as_str() {
            text.0 = name.to_string();
        }
    }
}

// ---------------------------------------------------------------- balance

/// Your balance: the coin and the number, under your name, on nothing (as
/// Hajun sketched it; on a dark glass until 2026-10-31).
fn balance(assets: &AssetServer) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::VMin(1.4),
            padding: UiRect::left(Val::VMin(1.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            (
                ImageNode::new(assets.load(COIN_ICON)),
                Node {
                    width: Val::VMin(BALANCE_ICON),
                    height: Val::VMin(BALANCE_ICON),
                    ..default()
                },
                Pickable::IGNORE,
            ),
            (BalanceText, words(money(STARTING_BALANCE), BALANCE_TEXT, Color::WHITE)),
        ],
    )
}

/// Shows your balance whenever it changes.
fn count_balance(
    me: Query<&Balance, (With<Player>, Changed<Balance>)>,
    mut texts: Query<&mut Text, With<BalanceText>>,
) {
    let Ok(balance) = me.single() else {
        return;
    };
    let label = money(balance.0);
    for mut text in &mut texts {
        if text.0 != label {
            text.0.clone_from(&label);
        }
    }
}

/// `12,500`.
pub(crate) fn money(amount: u32) -> String {
    let digits = amount.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (n, digit) in digits.chars().enumerate() {
        if n > 0 && (digits.len() - n).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

// ---------------------------------------------------------------- buttons

/// A round button's fill, the dark glass, darker under the pointer and held.
pub(crate) fn button_fill(interaction: Interaction) -> Color {
    let alpha = match interaction {
        Interaction::None => PANEL.alpha(),
        Interaction::Hovered => 0.56,
        Interaction::Pressed => 0.7,
    };
    PANEL.with_alpha(alpha)
}

/// Whenever you land on an island — in the town when the app opens, and back
/// in it after a game of House Builder — and it is the town, has you
/// welcomed.
fn arrive(
    me: Query<&Venue, (With<Player>, Changed<Venue>)>,
    welcomes: Query<Entity, With<Welcome>>,
    mut due: ResMut<WelcomeDue>,
    mut commands: Commands,
) {
    let Ok(&here) = me.single() else {
        return;
    };
    // One still up from the last time is out of date, whichever way you went.
    for welcome in &welcomes {
        commands.entity(welcome).despawn();
    }
    due.0 = here == Venue::Town;
}

// -------------------------------------------------------------- headlines

/// The welcome, once there is a town to see behind it. On the phone the first
/// one can come a second or two before the island has loaded; and the first
/// time the app is run, it waits for the sign-up's pages to go (`signup`),
/// which stand over the town.
fn welcome(
    islands: Res<Islands>,
    signup: Res<Signup>,
    mut due: ResMut<WelcomeDue>,
    mut commands: Commands,
) {
    if !due.0 || signup.up() || islands.get(Venue::Town).is_none() {
        return;
    }
    due.0 = false;
    announce(&mut commands, None, "Donggeurami Town", HEADLINE, WELCOME_SECS).insert(Welcome);
}

/// Puts `title` across the middle of the screen for `secs` seconds, in gold,
/// `size` big ([`HEADLINE`] unless it is long), with `caption` over it in
/// white, fading in and out.
pub(crate) fn announce<'a>(
    commands: &'a mut Commands,
    caption: Option<&str>,
    title: &str,
    size: f32,
    secs: f32,
) -> EntityCommands<'a> {
    let mut headline = commands.spawn((
        Headline { age: 0.0, secs },
        // Over the whole screen, with the words stacked in the middle and high
        // enough to leave your character, just below the middle, in sight.
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::bottom(Val::Vh(ABOVE_YOU)),
            ..default()
        },
        Pickable::IGNORE,
    ));
    // Put up clear, for `fade_headlines` to bring up: whenever in the frame
    // this is spawned, it is never drawn once at full strength first.
    headline.with_children(|lines| {
        if let Some(caption) = caption {
            lines.spawn(words(caption, CAPTION, Color::WHITE.with_alpha(0.0)));
        }
        lines.spawn(words(title, size, GOLD.with_alpha(0.0)));
    });
    headline
}

/// Brings each headline up, holds it, and takes it away again.
fn fade_headlines(
    time: Res<Time>,
    mut headlines: Query<(Entity, &mut Headline, &Children)>,
    mut lines: Query<(&mut TextColor, &mut TextShadow)>,
    mut commands: Commands,
) {
    for (entity, mut headline, children) in &mut headlines {
        headline.age += time.delta_secs();
        if headline.age >= headline.secs {
            commands.entity(entity).despawn();
            continue;
        }
        let solid = (headline.age / HEADLINE_IN)
            .min((headline.secs - headline.age) / HEADLINE_OUT)
            .clamp(0.0, 1.0);
        let mut words = lines.iter_many_mut(children);
        while let Some((mut colour, mut shadow)) = words.fetch_next() {
            // Only while it is fading: held, the words are left alone.
            let faded = colour.0.with_alpha(solid);
            if colour.0 != faded {
                colour.0 = faded;
                shadow.color = SHADOW.with_alpha(SHADOW.alpha() * solid);
            }
        }
    }
}

// --------------------------------------------------------------- pictures

/// A picture filling a button, drawn from flat shapes the way the edit
/// buttons are, all in percentages of the button's own box.
pub(crate) fn picture() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// Where a shape sits in a picture.
pub(crate) fn frame(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(left),
        top: Val::Percent(top),
        width: Val::Percent(width),
        height: Val::Percent(height),
        ..default()
    }
}

/// A plain white shape.
pub(crate) fn shape(left: f32, top: f32, width: f32, height: f32) -> impl Bundle {
    (
        frame(left, top, width, height),
        BackgroundColor(ICON),
        Pickable::IGNORE,
    )
}

/// A fist, for the punch button: a closed hand seen from the front, four
/// fingers curled over the palm with the thumb lying across them and the
/// wrist under, white on clear for the button to tint, drawn as the gift is
/// (Hajun, 2026-11-01: "a fist punch icon that is more a hand fist rather
/// than boxing glove"; Material has no closed hand, only its boxing glove,
/// `sports_mma`, which the button had until then).
pub(crate) fn fist() -> Image {
    const SIZE: u32 = 128;
    const ROUND: f32 = 0.1;
    /// The gap round the thumb, so that it reads as one.
    const GAP: f32 = 0.05;
    // In shares of half the image, from its middle, y up: each finger by
    // its middle and the top of it, all one width from one bottom; the
    // palm and the wrist by their middles and half sizes; and the thumb,
    // a box lying across the palm, its middle, half size and tilt.
    const FINGERS: [(f32, f32); 4] = [(-0.57, 0.68), (-0.19, 0.8), (0.19, 0.8), (0.57, 0.68)];
    const FINGER_HALF_WIDTH: f32 = 0.17;
    const FINGER_BOTTOM: f32 = 0.0;
    let palm = (Vec2::new(0.0, -0.2), Vec2::new(0.76, 0.3));
    let wrist = (Vec2::new(0.0, -0.7), Vec2::new(0.48, 0.22));
    let thumb = (Vec2::new(-0.22, -0.2), Vec2::new(0.5, 0.15));
    let (sin, cos) = 0.2f32.sin_cos();
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new(
                (column as f32 + 0.5) * pixel - 1.0,
                1.0 - (row as f32 + 0.5) * pixel,
            );
            let fingers = FINGERS
                .into_iter()
                .map(|(x, top)| {
                    let middle = Vec2::new(x, (top + FINGER_BOTTOM) * 0.5);
                    let reach = Vec2::new(FINGER_HALF_WIDTH - ROUND, (top - FINGER_BOTTOM) * 0.5 - ROUND);
                    box_distance(at - middle, reach) - ROUND
                })
                .fold(f32::INFINITY, f32::min);
            let palm_edge = box_distance(at - palm.0, palm.1 - ROUND) - ROUND;
            let wrist_edge = box_distance(at - wrist.0, wrist.1 - ROUND) - ROUND;
            // `at` as the thumb sees it, tilted up to the right.
            let from_thumb = at - thumb.0;
            let local = Vec2::new(from_thumb.x * cos + from_thumb.y * sin, from_thumb.y * cos - from_thumb.x * sin);
            let thumb_edge = box_distance(local, thumb.1 - ROUND) - ROUND;
            // The hand, with a gap cut round the thumb, and the thumb laid on.
            let hand = fingers.min(palm_edge).min(wrist_edge);
            let outside = hand.max(-(thumb_edge - GAP)).min(thumb_edge);
            let cover = (0.5 - outside / pixel).clamp(0.0, 1.0);
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

/// The jump button's arrow, pointing up: a broad head on a short shaft, one
/// filled shape with its corners rounded off, white on clear for the button
/// to tint (Hajun, 2026-10-31: "a semi transparent whiter filled round
/// arrow"). Drawn rather than built from nodes, as the arrow in strokes before
/// it was, because nodes that overlap would show where they do through the
/// see-through white; and edged by how far each pixel is from it, so that it
/// is smooth at whatever size a screen shows it.
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
pub(crate) fn jump_arrow() -> Image {
    const SIZE: u32 = 128;
    /// How much its corners are rounded off, as a share of half the image.
    const ROUND: f32 = 0.1;
    // In shares of half the image, from its middle, y up: the head's tip and
    // the ends of its base, and the shaft's half width and its two ends, each
    // drawn in by `ROUND`, which rounding puts back out.
    let tip = Vec2::new(0.0, 0.92 - ROUND);
    let base = Vec2::new(0.88 - ROUND, 0.06 + ROUND * 0.5);
    let (half_width, top, bottom) = (0.34 - ROUND, 0.2, -0.92 + ROUND);
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new(
                (column as f32 + 0.5) * pixel - 1.0,
                1.0 - (row as f32 + 0.5) * pixel,
            );
            let head = triangle_distance(at, tip, Vec2::new(-base.x, base.y), base);
            let middle = Vec2::new(0.0, (top + bottom) * 0.5);
            let reach = Vec2::new(half_width, (top - bottom) * 0.5);
            let shaft = box_distance(at - middle, reach);
            let outside = head.min(shaft) - ROUND;
            let cover = (0.5 - outside / pixel).clamp(0.0, 1.0);
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

/// An arrowhead pointing left, for turning through a card's colours on the
/// sign-up's character page (`signup`; flipped across for the right): a
/// rounded triangle, white on clear, drawn as the jump button's arrow is.
pub(crate) fn side_arrow() -> Image {
    const SIZE: u32 = 128;
    /// How much its corners are rounded off, as a share of half the image.
    const ROUND: f32 = 0.12;
    // In shares of half the image, from its middle, y up: the tip and the
    // two ends of the base, each drawn in by `ROUND`, which rounding puts
    // back out.
    let tip = Vec2::new(-0.85 + ROUND, 0.0);
    let (top, bottom) = (Vec2::new(0.6 - ROUND, 0.9 - ROUND), Vec2::new(0.6 - ROUND, -0.9 + ROUND));
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new(
                (column as f32 + 0.5) * pixel - 1.0,
                1.0 - (row as f32 + 0.5) * pixel,
            );
            let outside = triangle_distance(at, tip, bottom, top) - ROUND;
            let cover = (0.5 - outside / pixel).clamp(0.0, 1.0);
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

/// A pencil, for the settings' Edit name (`settings`): a rounded shaft with
/// a point at its lower left, leaning to the upper right, white on clear
/// for the button to tint, drawn as the gear is.
pub(crate) fn pencil() -> Image {
    const SIZE: u32 = 128;
    const ROUND: f32 = 0.04;
    // In shares of half the image, along the pencil and across it: the
    // shaft, a box from the point's base up to the top, and the point, a
    // triangle on the base, its corners drawn in by `ROUND`, which rounding
    // puts back out.
    const HALF_WIDTH: f32 = 0.17;
    const BASE: f32 = -0.32;
    const TOP: f32 = 0.78;
    const TIP: f32 = -0.78;
    let shaft_reach = Vec2::new(HALF_WIDTH - ROUND, (TOP - BASE) * 0.5 - ROUND);
    let shaft_middle = Vec2::new(0.0, (TOP + BASE) * 0.5);
    let tip = Vec2::new(0.0, TIP + ROUND);
    let base_left = Vec2::new(-HALF_WIDTH + ROUND, BASE);
    let base_right = Vec2::new(HALF_WIDTH - ROUND, BASE);
    let (sin, cos) = std::f32::consts::FRAC_PI_4.sin_cos();
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new(
                (column as f32 + 0.5) * pixel - 1.0,
                1.0 - (row as f32 + 0.5) * pixel,
            );
            // `at` as the pencil sees it, leaning 45 degrees to the right.
            let local = Vec2::new(at.x * cos - at.y * sin, at.x * sin + at.y * cos);
            let shaft = box_distance(local - shaft_middle, shaft_reach);
            let point = triangle_distance(local, tip, base_left, base_right);
            let outside = shaft.min(point) - ROUND;
            let cover = (0.5 - outside / pixel).clamp(0.0, 1.0);
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

/// How far `at` is from the edge of the triangle `a`, `b`, `c`: less than
/// nothing inside it. Inigo Quilez's `sdTriangle`.
pub(crate) fn triangle_distance(at: Vec2, a: Vec2, b: Vec2, c: Vec2) -> f32 {
    let (e0, e1, e2) = (b - a, c - b, a - c);
    let (v0, v1, v2) = (at - a, at - b, at - c);
    let along = |v: Vec2, e: Vec2| v - e * (v.dot(e) / e.dot(e)).clamp(0.0, 1.0);
    let (p0, p1, p2) = (along(v0, e0), along(v1, e1), along(v2, e2));
    let s = (e0.x * e2.y - e0.y * e2.x).signum();
    let d = Vec2::new(p0.dot(p0), s * (v0.x * e0.y - v0.y * e0.x))
        .min(Vec2::new(p1.dot(p1), s * (v1.x * e1.y - v1.y * e1.x)))
        .min(Vec2::new(p2.dot(p2), s * (v2.x * e2.y - v2.y * e2.x)));
    -d.x.sqrt() * d.y.signum()
}

/// How far `at` is from the edge of a box round the origin reaching `reach`
/// out each way: less than nothing inside it.
fn box_distance(at: Vec2, reach: Vec2) -> f32 {
    let d = at.abs() - reach;
    d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_jump_arrow_points_up_with_a_broad_head_on_a_narrow_shaft() {
        let image = jump_arrow();
        let size = image.width();
        let data = image.data.as_ref().unwrap();
        // How solid it is at `x`, `y`, in shares of half the image from its
        // middle, y up.
        let alpha = |x: f32, y: f32| {
            let column = (((x + 1.0) * 0.5 * size as f32) as u32).min(size - 1);
            let row = (((1.0 - y) * 0.5 * size as f32) as u32).min(size - 1);
            data[((row * size + column) * 4 + 3) as usize]
        };
        // Solid up its middle, from the foot of the shaft to the tip.
        for y in [-0.85, -0.4, 0.0, 0.4, 0.8] {
            assert_eq!(alpha(0.0, y), 255, "at {y}");
        }
        // The head is wide, and the shaft beside it is not there.
        assert_eq!(alpha(-0.7, 0.1), 255);
        assert_eq!(alpha(0.7, 0.1), 255);
        assert_eq!(alpha(-0.6, -0.5), 0);
        assert_eq!(alpha(0.6, -0.5), 0);
        // Nothing in the corners, nor over the tip.
        for (x, y) in [(-0.97, 0.97), (0.97, 0.97), (-0.97, -0.97), (0.97, -0.97), (0.0, 0.99)] {
            assert_eq!(alpha(x, y), 0, "at {x} {y}");
        }
        // White throughout, for the button to tint.
        assert!(data.chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]));
    }

    #[test]
    fn the_fist_is_four_fingers_over_a_palm_with_the_thumb_across_it() {
        let image = fist();
        let size = image.width();
        let data = image.data.as_ref().unwrap();
        let alpha = |x: f32, y: f32| {
            let column = (((x + 1.0) * 0.5 * size as f32) as u32).min(size - 1);
            let row = (((1.0 - y) * 0.5 * size as f32) as u32).min(size - 1);
            data[((row * size + column) * 4 + 3) as usize]
        };
        // Solid on each finger and nothing between the middle two; solid
        // on the palm, the thumb and the wrist; nothing over the fingers,
        // beside the wrist, or in the corners.
        for x in [-0.57, -0.19, 0.19, 0.57] {
            assert_eq!(alpha(x, 0.5), 255, "finger at {x}");
        }
        assert_eq!(alpha(0.0, 0.5), 0);
        assert_eq!(alpha(0.6, -0.3), 255);
        assert_eq!(alpha(-0.2, -0.24), 255);
        assert_eq!(alpha(0.0, -0.75), 255);
        assert_eq!(alpha(0.0, 0.95), 0);
        assert_eq!(alpha(0.75, -0.75), 0);
        assert_eq!(alpha(0.95, 0.95), 0);
        assert!(data.chunks(4).all(|pixel| pixel[..3] == [255, 255, 255]));
    }

    #[test]
    fn the_pencil_leans_to_the_upper_right_with_its_point_at_the_lower_left() {
        let image = pencil();
        let size = image.width();
        let data = image.data.as_ref().unwrap();
        let alpha = |x: f32, y: f32| {
            let column = (((x + 1.0) * 0.5 * size as f32) as u32).min(size - 1);
            let row = (((1.0 - y) * 0.5 * size as f32) as u32).min(size - 1);
            data[((row * size + column) * 4 + 3) as usize]
        };
        // Solid along the diagonal from the point to the top, and nothing
        // along the other diagonal or beyond either end.
        for t in [-0.45, -0.2, 0.0, 0.3, 0.5] {
            assert_eq!(alpha(t, t), 255, "at {t}");
        }
        assert_eq!(alpha(-0.5, 0.5), 0);
        assert_eq!(alpha(0.5, -0.5), 0);
        assert_eq!(alpha(0.7, 0.7), 0);
        assert_eq!(alpha(-0.7, -0.7), 0);
        // The point is narrower than the shaft: beside the diagonal near the
        // point there is nothing, beside it up the shaft there is pencil.
        assert_eq!(alpha(-0.42, -0.62), 0);
        assert_eq!(alpha(0.1, 0.3), 255);
    }

    #[test]
    fn every_icon_is_there_white_on_clear_and_in_the_apk() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("mobile/android/app/assets-left-out.txt"))
            .expect("the APK's list of what it leaves out");
        let patterns: Vec<&str> = list
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.trim_start_matches("<dir>").trim_start_matches("<file>"))
            .collect();
        for icon in ICONS {
            let picture = format!("icons/{icon}.png");
            let bytes = std::fs::read(root.join("assets").join(&picture)).expect(&picture);
            assert_eq!(&bytes[1..4], b"PNG", "{picture}");
            for name in picture.split('/') {
                let left_out = patterns.iter().find(|pattern| {
                    let pattern: &str = pattern;
                    if let Some(end) = pattern.strip_prefix('*') {
                        name.ends_with(end)
                    } else if let Some(start) = pattern.strip_suffix('*') {
                        name.starts_with(start)
                    } else {
                        pattern == name
                    }
                });
                assert!(left_out.is_none(), "{picture} is left out of the APK by `{}`", left_out.unwrap());
            }
        }
    }

    #[test]
    fn a_triangle_is_inside_out_neither_way_round() {
        let (a, b, c) = (Vec2::new(0.0, 1.0), Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0));
        for (a, b, c) in [(a, b, c), (a, c, b)] {
            assert!(triangle_distance(Vec2::ZERO, a, b, c) < 0.0);
            assert!((triangle_distance(Vec2::new(0.0, -2.0), a, b, c) - 1.0).abs() < 1e-5);
        }
    }
}
