//! What is drawn over the world: your balance top left, and headlines across
//! the middle — the town's name, as a welcome, whenever you arrive in the
//! town, and whatever else is [`announce`]d, like House Builder's theme. And
//! how a phone's stick and jump button look (`setup_hud`).
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

pub(crate) const COIN_ICON: &str = "coin.png";

/// Sizes, as shares of the short side of the screen, so that a phone and a
/// monitor show the same thing.
const BALANCE_TEXT: f32 = 6.0;
const BALANCE_ICON: f32 = 6.0;
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
/// A phone's stick and jump button: a see-through grey, with no rim, a little
/// more solid while the jump button is held; and the stick's knob and the
/// jump button's arrow, a see-through white a little greyed (Hajun,
/// 2026-10-31).
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
pub(crate) const TOUCH_DISC: Color = Color::srgba(0.4, 0.4, 0.4, 0.32);
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
pub(crate) const TOUCH_DISC_HELD: Color = Color::srgba(0.4, 0.4, 0.4, 0.5);
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
pub(crate) const TOUCH_MARK: Color = Color::srgba(0.92, 0.92, 0.92, 0.38);

/// Your balance, top left.
#[derive(Component)]
struct BalanceText;

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
        .add_systems(Startup, show_top_bar)
        .add_systems(
            Update,
            (count_balance, (arrive, welcome, fade_headlines).chain()),
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

/// Your balance top left, for as long as the app is open.
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
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
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
    fn a_triangle_is_inside_out_neither_way_round() {
        let (a, b, c) = (Vec2::new(0.0, 1.0), Vec2::new(-1.0, -1.0), Vec2::new(1.0, -1.0));
        for (a, b, c) in [(a, b, c), (a, c, b)] {
            assert!(triangle_distance(Vec2::ZERO, a, b, c) < 0.0);
            assert!((triangle_distance(Vec2::new(0.0, -2.0), a, b, c) - 1.0).abs() < 1e-5);
        }
    }
}
