//! What is drawn over the world: your balance top left, the button top right
//! that takes you home and back again, and headlines across the middle — the
//! town's name, as a welcome, whenever you arrive in the town, and whatever
//! else is [`announce`]d, like House Builder's theme.
//!
//! The button is a `bevy_ui` node pressed through `bevy_picking`'s click,
//! which a mouse and a finger both send, so there is no platform code in it.
//! It, and every button House Builder puts up, carries [`TakesPress`]; the code
//! that reads fingers and the mouse for everything else asks [`Presses`]
//! first, so that a press on one is not also taken for a look round or a tap
//! on the town.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use roundtown_net::Ask;

use crate::island::{Islands, Venue};
use crate::lobby::{self, Balance, STARTING_BALANCE, Seat, Whereabouts};
use crate::net::Online;
use crate::{OrbitCamera, Player};

pub(crate) const COIN_ICON: &str = "coin.png";

/// Sizes, as shares of the short side of the screen, so that a phone and a
/// monitor show the same thing.
const BALANCE_TEXT: f32 = 6.0;
const BALANCE_ICON: f32 = 6.0;
pub(crate) const BUTTON: f32 = 12.0;
pub(crate) const HEADLINE: f32 = 8.0;
pub(crate) const CAPTION: f32 = 4.5;
/// How far in from the top and the sides of the screen the balance and the
/// button sit: further than a phone's rounded corner reaches.
pub(crate) const EDGE: f32 = 4.0;
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
/// The pictures on the button, and the door of the house.
pub(crate) const ICON: Color = Color::srgba(1.0, 1.0, 1.0, 0.94);
pub(crate) const DOOR: Color = Color::srgb(0.16, 0.30, 0.42);

/// Your balance, top left.
#[derive(Component)]
struct BalanceText;

/// The button top right.
#[derive(Component)]
struct TravelButton;

/// One of the pictures on the button: the island it takes you to. Only the one
/// for the island you are not on is shown.
#[derive(Component)]
struct Destination(Venue);

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

/// Something on the screen that takes a press for itself: the button top
/// right, and every button House Builder puts up, the dark behind its dialogs
/// included. Whatever reads a finger or the mouse for the world asks
/// [`Presses::claimed`] before it takes a press.
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
        .add_systems(Startup, show_top_bar)
        .add_systems(
            Update,
            (
                count_balance,
                light_button,
                (arrive, welcome, fade_headlines).chain(),
            ),
        );
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

/// Your balance top left and the button top right, for as long as the app is
/// open.
pub(crate) fn show_top_bar(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(EDGE),
                left: Val::VMin(EDGE),
                right: Val::VMin(EDGE),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            // The balance, and beside it what goes with it: online, the
            // button for your account (`account`).
            bar.spawn((
                BarLeft,
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::VMin(1.4),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_child(balance(&assets));
            bar.spawn(travel_button()).observe(travel);
        });
}

/// The left of the bar along the top: your balance, and whatever goes with
/// it.
#[derive(Component)]
pub(crate) struct BarLeft;

// ---------------------------------------------------------------- balance

fn balance(assets: &AssetServer) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::VMin(1.4),
            padding: UiRect::new(
                Val::VMin(1.2),
                Val::VMin(3.0),
                Val::VMin(0.8),
                Val::VMin(0.8),
            ),
            border_radius: BorderRadius::all(Val::VMin(3.0)),
            ..default()
        },
        BackgroundColor(PANEL),
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

// ----------------------------------------------------------------- button

/// A round button with a house on it, for going home, and a town hall, for
/// going back to the town. [`arrive`] decides which one shows, and whether the
/// button does at all.
fn travel_button() -> impl Bundle {
    (
        TravelButton,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(BUTTON),
            height: Val::VMin(BUTTON),
            // At the top of the bar however tall the balance beside it is.
            align_self: AlignSelf::FlexStart,
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(button_fill(Interaction::None)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        children![
            (Destination(Venue::Home), picture(), house()),
            (
                Destination(Venue::Town),
                picture(),
                town_hall(),
                Visibility::Hidden
            ),
        ],
    )
}

pub(crate) fn button_fill(interaction: Interaction) -> Color {
    let alpha = match interaction {
        Interaction::None => PANEL.alpha(),
        Interaction::Hovered => 0.56,
        Interaction::Pressed => 0.7,
    };
    PANEL.with_alpha(alpha)
}

fn light_button(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (With<TravelButton>, Changed<Interaction>),
    >,
) {
    for (interaction, mut fill) in &mut buttons {
        fill.0 = button_fill(*interaction);
    }
}

/// Takes you home from the town, and back to the town from home: to where your
/// seat arrives on that island, facing in, with whatever jump you were in the
/// middle of dropped and the camera swung round behind you. Online it is the
/// server that seats you, and says where you arrive (`net`).
fn travel(
    click: On<Pointer<Click>>,
    me: Query<(Entity, &Seat), With<Player>>,
    mut bodies: Whereabouts,
    mut orbit: ResMut<OrbitCamera>,
    online: Res<Online>,
    mut asks: MessageWriter<Ask>,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    let Ok((body, seat)) = me.single() else {
        return;
    };
    let Some(there) = bodies
        .get(body)
        .ok()
        .and_then(|(_, &here, ..)| destination(here))
    else {
        return;
    };
    if online.is() {
        asks.write(Ask::Travel(there));
        return;
    }
    lobby::send(&mut bodies, &mut orbit, body, there, seat.0);
}

/// Where the button takes you from `here`, if anywhere. It goes between the
/// town and your home, and is put away while you are playing House Builder,
/// which has its own way back.
fn destination(here: Venue) -> Option<Venue> {
    match here {
        Venue::Town => Some(Venue::Home),
        Venue::Home => Some(Venue::Town),
        Venue::Lobby | Venue::Plot(_) => None,
    }
}

/// Whenever you land on an island — in the town when the app opens, then
/// wherever the button takes you — puts the way back on the button, and in
/// the town, has you welcomed.
fn arrive(
    me: Query<&Venue, (With<Player>, Changed<Venue>)>,
    mut buttons: Query<&mut Visibility, (With<TravelButton>, Without<Destination>)>,
    mut pictures: Query<(&Destination, &mut Visibility), Without<TravelButton>>,
    welcomes: Query<Entity, With<Welcome>>,
    mut due: ResMut<WelcomeDue>,
    mut commands: Commands,
) {
    let Ok(&here) = me.single() else {
        return;
    };
    let way = destination(here);
    for mut visibility in &mut buttons {
        visibility.set_if_neq(if way.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (picture, mut visibility) in &mut pictures {
        visibility.set_if_neq(if Some(picture.0) == way {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    // One still up from the last time is out of date, whichever way you went.
    for welcome in &welcomes {
        commands.entity(welcome).despawn();
    }
    due.0 = here == Venue::Town;
}

// -------------------------------------------------------------- headlines

/// The welcome, once there is a town to see behind it. On the phone the first
/// one can come a second or two before the island has loaded.
fn welcome(islands: Res<Islands>, mut due: ResMut<WelcomeDue>, mut commands: Commands) {
    if !due.0 || islands.get(Venue::Town).is_none() {
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

/// A picture filling the button, drawn from flat shapes the way the jump arrow
/// and the edit buttons are, all in percentages of the button's own box.
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

/// A white circle.
pub(crate) fn disc(left: f32, top: f32, size: f32) -> impl Bundle {
    (
        Node {
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..frame(left, top, size, size)
        },
        BackgroundColor(ICON),
        Pickable::IGNORE,
    )
}

/// A person, head and shoulders: you.
pub(crate) fn person() -> impl Bundle {
    children![
        disc(37.0, 16.0, 26.0),
        (
            Node {
                border_radius: BorderRadius::top(Val::Percent(50.0)),
                ..frame(23.0, 47.0, 54.0, 33.0)
            },
            BackgroundColor(ICON),
            Pickable::IGNORE,
        ),
    ]
}

/// A house with a pitched roof, a chimney and a door. The roof is a square
/// turned on its corner, its lower half hidden in the walls.
fn house() -> impl Bundle {
    children![
        shape(62.0, 25.0, 6.5, 15.0),
        (
            frame(32.3, 28.3, 35.4, 35.4),
            BackgroundColor(ICON),
            UiTransform::from_rotation(Rot2::degrees(45.0)),
            Pickable::IGNORE,
        ),
        shape(25.0, 46.0, 50.0, 31.0),
        (
            Node {
                border_radius: BorderRadius::top(Val::Percent(50.0)),
                ..frame(44.0, 60.0, 12.0, 17.0)
            },
            BackgroundColor(DOOR),
            Pickable::IGNORE,
        ),
    ]
}

/// A town hall: a flag on a dome, over a row of columns on a stepped base. The
/// dome is a circle, its lower half hidden in the drum it stands on.
fn town_hall() -> impl Bundle {
    children![
        shape(49.25, 21.0, 1.5, 10.0),
        shape(50.75, 21.0, 6.75, 5.0),
        disc(41.0, 29.0, 18.0),
        shape(41.0, 38.0, 18.0, 8.0),
        shape(24.0, 46.0, 52.0, 5.5),
        shape(27.0, 51.5, 6.5, 18.5),
        shape(40.17, 51.5, 6.5, 18.5),
        shape(53.33, 51.5, 6.5, 18.5),
        shape(66.5, 51.5, 6.5, 18.5),
        shape(24.0, 70.0, 52.0, 4.5),
        shape(20.0, 74.5, 60.0, 4.5),
    ]
}
