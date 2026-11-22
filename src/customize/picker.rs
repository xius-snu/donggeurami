//! The window a colour to paint with is chosen in, behind the round button
//! beside the paintbrush (Hajun, 2026-11-22: "u tap that color button and
//! it opens a rgb selector panel ... the rgb selector should be a circular
//! color selector with white in the middle, and also on the right it should
//! have a brightness (whiter/darker) bar selector, no transparency
//! selector").
//!
//! A window as every window is (`window`): dark and see-through, its title
//! along its top, a bare white cross in its corner. In it a wheel, every
//! colour round its rim and white in its middle, and to its right a bar
//! from the colour chosen on the wheel, at its top, down to black. A
//! finger put on either, or dragged over it, chooses: where on the wheel is
//! which colour and how much of it ([`on_the_wheel`]), and how far down the
//! bar is how dark ([`down_the_bar`]). A ring on each marks what is chosen,
//! the bar's filled with the colour itself; and the round button the
//! window was opened from, beside the brush, is that colour as it changes.
//! The cross, a tap outside the window, or `Escape`, puts it away, and the
//! brush paints that colour from then.
//!
//! Along its foot, the colours painted with before, the last seven, newest
//! first (`Customize::used`; Hajun, later that day: "on the bottom of hat
//! panel, it should be like one row of the previous used color history (in
//! the beginning it can be empty)"): a tap on one chooses it again.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::{Customize, Hsv};
use crate::builder::DIALOG_LAYER;
use crate::hud::{DialogUp, TakesPress};
use crate::ui::{UiChanged, ui};
use crate::window;

/// The wheel: how big across; the bar: how wide, as tall as the wheel; the
/// room between them, and round them in the window; and the rings that
/// mark what is chosen: how big, and how thick. In shares of the short
/// side of the screen.
const WHEEL: f32 = 46.0;
const BAR: f32 = 6.5;
const BETWEEN: f32 = 6.0;
const SIDE: f32 = 6.0;
const OVER_TITLE: f32 = 1.5;
const UNDER_TITLE: f32 = 1.5;
const UNDER: f32 = 4.5;
const RING: f32 = 4.6;
const RING_LINE: f32 = 0.5;
/// The row of colours painted with before, along the window's foot: how
/// big each is, how far the row is under the wheel, and the gap between
/// two of them, which is what spreads a full row of them from under the
/// wheel's left side to under the bar's right.
const PAST: f32 = 6.4;
const OVER_PAST: f32 = 3.2;
const PAST_GAP: f32 = (WHEEL + BETWEEN + BAR - super::USED as f32 * PAST) / (super::USED as f32 - 1.0);
/// How many dots the wheel's picture is across, and the bar's is tall.
const WHEEL_DOTS: u32 = 256;
const BAR_DOTS: u32 = 64;

/// The window: whether it is up, and whether the round button was pressed
/// since that was last looked at.
#[derive(Resource, Default)]
pub(crate) struct Picker {
    open: bool,
    asked: bool,
    closed: bool,
}

impl Picker {
    /// The round button beside the brush was pressed.
    pub(super) fn ask(&mut self) {
        self.asked = true;
    }

    pub(super) fn is_open(&self) -> bool {
        self.open
    }
}

/// One of the colours painted with before, along the window's foot.
#[derive(Component, Clone, Copy)]
struct Past(Hsv);

/// The root of the window, over the whole screen.
#[derive(Component)]
struct Root;

/// What shuts the window: its cross, and the screen round it.
#[derive(Component)]
struct Shuts;

#[derive(Component)]
struct Wheel;

#[derive(Component)]
struct Bar;

/// The rings that mark what is chosen, on the wheel and on the bar.
#[derive(Component)]
struct WheelRing;

#[derive(Component)]
struct BarRing;

/// The wheel's picture and the bar's, each drawn once.
#[derive(Resource)]
struct Pictures {
    wheel: Handle<Image>,
    bar: Handle<Image>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Picker>()
        .add_observer(shut_on_a_tap)
        .add_observer(choose_again)
        .add_systems(Startup, draw_pictures)
        // Before you are moved by the press that opened it (and so before
        // the building code, which `super::settle` comes after: the brush
        // put down is heard of here a frame later).
        .add_systems(Update, (act, redraw, show_choice).chain().before(crate::read_player_input));
}

fn draw_pictures(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    commands.insert_resource(Pictures {
        wheel: images.add(wheel_picture()),
        bar: images.add(bar_picture()),
    });
}

/// The wheel: every colour as bright as it goes, by where it is from the
/// middle ([`on_the_wheel`]), and clear outside its rim.
fn wheel_picture() -> Image {
    let size = WHEEL_DOTS;
    let dot = 2.0 / size as f32;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for row in 0..size {
        for column in 0..size {
            let at = Vec2::new((column as f32 + 0.5) * dot - 1.0, (row as f32 + 0.5) * dot - 1.0);
            let (hue, out) = on_the_wheel(at);
            let [red, green, blue] = Hsv { hue, out, bright: 1.0 }.rgb();
            let cover = (0.5 + (1.0 - at.length()) / dot).clamp(0.0, 1.0);
            data.extend_from_slice(&[red, green, blue, (cover * 255.0).round() as u8]);
        }
    }
    picture(size, size, data)
}

/// The bar: white at its top down to black, for whoever shows it to tint
/// with the colour chosen on the wheel.
fn bar_picture() -> Image {
    let mut data = Vec::with_capacity((BAR_DOTS * 4) as usize);
    for row in 0..BAR_DOTS {
        let bright = (255.0 * (1.0 - row as f32 / (BAR_DOTS - 1) as f32)).round() as u8;
        data.extend_from_slice(&[bright, bright, bright, 255]);
    }
    picture(1, BAR_DOTS, data)
}

fn picture(width: u32, height: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Which colour is `at` on the wheel, and how much of it: `at` is from the
/// wheel's middle, its rim one away, across and down the screen. Red is at
/// the right, and they go round the way the hours do not, yellow up and to
/// the right of it; the middle is white, and past the rim is the rim.
fn on_the_wheel(at: Vec2) -> (f32, f32) {
    let hue = (-at.y).atan2(at.x) / std::f32::consts::TAU;
    (hue.rem_euclid(1.0), at.length().min(1.0))
}

/// Where a colour is on the wheel, as [`on_the_wheel`] has it.
fn wheel_place(colour: Hsv) -> Vec2 {
    let angle = colour.hue * std::f32::consts::TAU;
    Vec2::new(angle.cos(), -angle.sin()) * colour.out.clamp(0.0, 1.0)
}

/// How bright `down` the bar is: `down` is from its middle, its top and
/// its foot one away. As bright as it goes at its top, black at its foot.
fn down_the_bar(down: f32) -> f32 {
    (0.5 - down * 0.5).clamp(0.0, 1.0)
}

/// Where a pointer at `pointer`, in logical pixels of the window, is on
/// something laid out as `node` at `place`: from its middle, each of its
/// edges one away.
fn within(node: &ComputedNode, place: &UiGlobalTransform, pointer: Vec2) -> Vec2 {
    let half = (node.size() * 0.5).max(Vec2::splat(1.0));
    (pointer / node.inverse_scale_factor().max(f32::EPSILON) - place.translation) / half
}

/// A finger put on the wheel: that colour.
fn press_wheel(
    press: On<Pointer<Press>>,
    wheels: Query<(&ComputedNode, &UiGlobalTransform), With<Wheel>>,
    customize: ResMut<Customize>,
) {
    if let Ok((node, place)) = wheels.get(press.entity) {
        choose_on_the_wheel(within(node, place, press.pointer_location.position), customize);
    }
}

/// And dragged over it, or off it: the colour it is over, or the nearest
/// on the rim.
fn drag_wheel(
    drag: On<Pointer<Drag>>,
    wheels: Query<(&ComputedNode, &UiGlobalTransform), With<Wheel>>,
    customize: ResMut<Customize>,
) {
    if let Ok((node, place)) = wheels.get(drag.entity) {
        choose_on_the_wheel(within(node, place, drag.pointer_location.position), customize);
    }
}

fn choose_on_the_wheel(at: Vec2, mut customize: ResMut<Customize>) {
    let was = customize.chosen();
    let (hue, out) = on_the_wheel(at);
    // In the very middle there is no way round to be: it keeps the one it
    // had, so that the bar does not jump to red.
    let now = Hsv {
        hue: if out > 0.0 { hue } else { was.hue },
        out,
        ..was
    };
    if now != was {
        customize.choose(now);
    }
}

/// A finger put on the bar: that bright.
fn press_bar(press: On<Pointer<Press>>, bars: Query<(&ComputedNode, &UiGlobalTransform), With<Bar>>, customize: ResMut<Customize>) {
    if let Ok((node, place)) = bars.get(press.entity) {
        choose_down_the_bar(within(node, place, press.pointer_location.position).y, customize);
    }
}

fn drag_bar(drag: On<Pointer<Drag>>, bars: Query<(&ComputedNode, &UiGlobalTransform), With<Bar>>, customize: ResMut<Customize>) {
    if let Ok((node, place)) = bars.get(drag.entity) {
        choose_down_the_bar(within(node, place, drag.pointer_location.position).y, customize);
    }
}

fn choose_down_the_bar(down: f32, mut customize: ResMut<Customize>) {
    let was = customize.chosen();
    let now = Hsv {
        bright: down_the_bar(down),
        ..was
    };
    if now != was {
        customize.choose(now);
    }
}

/// A tap on one of the colours painted with before: that colour again.
fn choose_again(click: On<Pointer<Click>>, pasts: Query<&Past>, mut customize: ResMut<Customize>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(&Past(colour)) = pasts.get(click.entity)
        && customize.chosen() != colour
    {
        customize.choose(colour);
    }
}

/// A tap on the cross, or on the screen round the window: it is to be
/// shut ([`act`]). Only what the press landed on, not what the click
/// passed up through.
fn shut_on_a_tap(click: On<Pointer<Click>>, shuts: Query<(), With<Shuts>>, mut picker: ResMut<Picker>) {
    if click.button == PointerButton::Primary && click.original_event_target() == click.entity && shuts.contains(click.entity) {
        picker.closed = true;
    }
}

/// Opens the window when the round button is pressed, with the brush in
/// hand and nothing else up; and shuts it at its cross, a tap round it,
/// `Escape`, or the brush put down.
fn act(mut picker: ResMut<Picker>, customize: Res<Customize>, mut up: ResMut<DialogUp>, keyboard: Res<ButtonInput<KeyCode>>) {
    if !(picker.open || picker.asked || picker.closed) {
        return;
    }
    let asked = std::mem::take(&mut picker.asked);
    let closed = std::mem::take(&mut picker.closed);
    if !picker.open {
        if asked && customize.painting() && !up.0 {
            picker.open = true;
            up.0 = true;
        }
    } else if closed || keyboard.just_pressed(KeyCode::Escape) || !customize.painting() {
        picker.open = false;
        if up.0 {
            up.0 = false;
        }
    }
}

/// Puts the window up while it is open, again in the file's colours and
/// words as those change on desktop, and takes it down once it is shut.
fn redraw(
    picker: Res<Picker>,
    changed: Res<UiChanged>,
    customize: Res<Customize>,
    roots: Query<Entity, With<Root>>,
    pictures: Res<Pictures>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let stale = changed.is_changed() && !changed.is_added();
    if !picker.open || stale {
        for root in &roots {
            commands.entity(root).despawn();
        }
    }
    if !picker.open || (!stale && !roots.is_empty()) {
        return;
    }
    let ui = ui();
    let colours = ui.colours.window;
    let root = commands
        .spawn((
            Root,
            Shuts,
            TakesPress,
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
                padding: UiRect {
                    left: Val::VMin(SIDE),
                    right: Val::VMin(SIDE),
                    top: Val::VMin(OVER_TITLE),
                    bottom: Val::VMin(UNDER),
                },
                ..default()
            }),
        ))
        .id();
    commands.spawn((
        ChildOf(pane),
        window::header(),
        children![window::words(ui.words.bag.paint_title.clone(), window::TITLE, colours.title)],
    ));
    commands.spawn((ChildOf(pane), window::cross(assets.load("icons/close.png"), Shuts)));
    let body = commands
        .spawn((
            ChildOf(pane),
            Node {
                margin: UiRect::top(Val::VMin(UNDER_TITLE)),
                column_gap: Val::VMin(BETWEEN),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands
        .spawn((
            ChildOf(body),
            Wheel,
            TakesPress,
            ImageNode::new(pictures.wheel.clone()),
            Node {
                width: Val::VMin(WHEEL),
                height: Val::VMin(WHEEL),
                flex_shrink: 0.0,
                ..default()
            },
            children![(WheelRing, ring(Color::NONE))],
        ))
        .observe(press_wheel)
        .observe(drag_wheel);
    commands
        .spawn((
            ChildOf(body),
            Bar,
            TakesPress,
            // One dot wide and stretched: as wide as the bar is.
            ImageNode::new(pictures.bar.clone()).with_mode(NodeImageMode::Stretch),
            Node {
                width: Val::VMin(BAR),
                height: Val::VMin(WHEEL),
                flex_shrink: 0.0,
                border_radius: BorderRadius::all(Val::VMin(BAR * 0.5)),
                ..default()
            },
            children![(BarRing, ring(Color::WHITE))],
        ))
        .observe(press_bar)
        .observe(drag_bar);
    // The colours painted with before, newest first: a row as tall as one
    // of them whether or not there is one yet.
    let past = commands
        .spawn((
            ChildOf(pane),
            Node {
                height: Val::VMin(PAST),
                margin: UiRect::top(Val::VMin(OVER_PAST)),
                column_gap: Val::VMin(PAST_GAP),
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for colour in customize.used() {
        commands.spawn((
            ChildOf(past),
            Past(colour),
            TakesPress,
            Button,
            Node {
                width: Val::VMin(PAST),
                height: Val::VMin(PAST),
                border: UiRect::all(Val::VMin(RING_LINE * 0.7)),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(colour.color()),
            BorderColor::all(colours.line),
        ));
    }
}

/// A ring that marks what is chosen: white, with a thin dark line round it
/// so that it shows on white too, and `fill` inside it. Put where the
/// choice is by [`show_choice`].
fn ring(fill: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::VMin(RING),
            height: Val::VMin(RING),
            margin: UiRect::all(Val::VMin(-RING * 0.5)),
            border: UiRect::all(Val::VMin(RING_LINE)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(fill),
        BorderColor::all(Color::WHITE),
        Outline {
            width: Val::VMin(0.2),
            offset: Val::ZERO,
            color: Color::BLACK.with_alpha(0.55),
        },
        Pickable::IGNORE,
    )
}

/// Shows what is chosen on the window: the ring on the wheel where the
/// colour is, the bar from that colour down to black, and the ring on the
/// bar as far down as it is dark, filled with the colour itself. As the
/// choice changes, and on a window just put up. Only what differs is
/// written.
#[allow(clippy::type_complexity)]
fn show_choice(
    customize: Res<Customize>,
    fresh: Query<(), Added<Wheel>>,
    mut wheel_rings: Query<&mut Node, (With<WheelRing>, Without<BarRing>)>,
    mut bar_rings: Query<(&mut Node, &mut BackgroundColor), (With<BarRing>, Without<WheelRing>)>,
    mut bars: Query<&mut ImageNode, With<Bar>>,
) {
    if !customize.is_changed() && fresh.is_empty() {
        return;
    }
    let colour = customize.chosen();
    let at = wheel_place(colour);
    let place = |share: f32| Val::Percent(50.0 + 50.0 * share);
    for mut node in &mut wheel_rings {
        let (left, top) = (place(at.x), place(at.y));
        if node.left != left || node.top != top {
            (node.left, node.top) = (left, top);
        }
    }
    let down = Val::Percent(100.0 * (1.0 - colour.bright.clamp(0.0, 1.0)));
    for (mut node, mut fill) in &mut bar_rings {
        if node.top != down {
            node.top = down;
        }
        if fill.0 != colour.color() {
            fill.0 = colour.color();
        }
    }
    let full = colour.full().color();
    for mut bar in &mut bars {
        if bar.color != full {
            bar.color = full;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_is_white_in_its_middle_and_every_colour_round_its_rim() {
        // The middle: no colour at all, whichever way.
        assert_eq!(on_the_wheel(Vec2::ZERO).1, 0.0);
        // Right is red; up and to the right, toward yellow; up, between
        // yellow and green; left, cyan; down, between blue and magenta.
        let colour = |at: Vec2| {
            let (hue, out) = on_the_wheel(at);
            Hsv { hue, out, bright: 1.0 }.rgb()
        };
        assert_eq!(colour(Vec2::new(1.0, 0.0)), [255, 0, 0]);
        assert_eq!(colour(Vec2::new(0.5, -0.866_025_4)), [255, 255, 0]);
        assert_eq!(colour(Vec2::new(-1.0, 0.0)), [0, 255, 255]);
        assert_eq!(colour(Vec2::new(-0.5, 0.866_025_4)), [0, 0, 255]);
        // Half way out, half way to white; and past the rim is the rim.
        assert_eq!(colour(Vec2::new(0.5, 0.0)), [255, 128, 128]);
        assert_eq!(colour(Vec2::new(7.0, 0.0)), [255, 0, 0]);
        // Where a colour is on the wheel is where the wheel has it.
        for (hue, out) in [(0.0, 1.0), (0.2, 0.6), (0.55, 0.3), (0.9, 1.0)] {
            let (back, far) = on_the_wheel(wheel_place(Hsv { hue, out, bright: 0.4 }));
            assert!((back - hue).abs() < 1e-5 && (far - out).abs() < 1e-5, "{hue} {out}: {back} {far}");
        }
        // The picture: white in its middle, red at its right, clear at
        // its corners, and every dot of it as the wheel has it.
        let image = wheel_picture();
        let data = image.data.as_ref().expect("the wheel's dots");
        let dot = |column: u32, row: u32| {
            let at = ((row * WHEEL_DOTS + column) * 4) as usize;
            [data[at], data[at + 1], data[at + 2], data[at + 3]]
        };
        let (middle, edge) = (WHEEL_DOTS / 2, WHEEL_DOTS - 2);
        assert!(dot(middle, middle)[..3].iter().all(|&part| part > 250), "{:?}", dot(middle, middle));
        let right = dot(edge, middle);
        assert!(right[0] == 255 && right[1] < 12 && right[2] < 12 && right[3] == 255, "{right:?}");
        assert_eq!(dot(0, 0)[3], 0);
        assert_eq!(dot(WHEEL_DOTS - 1, WHEEL_DOTS - 1)[3], 0);
    }

    #[test]
    fn a_row_of_the_colours_painted_with_fits_under_the_wheel_and_the_bar() {
        let across = super::super::USED as f32 * PAST + (super::super::USED as f32 - 1.0) * PAST_GAP;
        assert!((across - (WHEEL + BETWEEN + BAR)).abs() < 1e-4, "{across}");
        assert!(PAST_GAP > 1.0, "{PAST_GAP}");
        // And the window, a title, the wheel and the row, fits a phone
        // lying on its side with room over and under.
        let tall = OVER_TITLE + window::TAB_HEIGHT + UNDER_TITLE + WHEEL + OVER_PAST + PAST + UNDER;
        assert!(tall < 100.0 - 2.0 * crate::hud::EDGE, "{tall}");
    }

    #[test]
    fn the_bar_is_as_bright_as_it_goes_at_its_top_and_black_at_its_foot() {
        assert_eq!(down_the_bar(-1.0), 1.0);
        assert_eq!(down_the_bar(0.0), 0.5);
        assert_eq!(down_the_bar(1.0), 0.0);
        // Past either end is the end.
        assert_eq!(down_the_bar(-3.0), 1.0);
        assert_eq!(down_the_bar(3.0), 0.0);
        let image = bar_picture();
        let data = image.data.as_ref().expect("the bar's dots");
        assert_eq!(&data[..4], &[255, 255, 255, 255]);
        assert_eq!(&data[data.len() - 4..], &[0, 0, 0, 255]);
    }

    #[test]
    fn a_pointer_is_found_on_what_it_is_over_from_its_middle() {
        // Something 200 dots across and 100 tall, its middle at (300, 400)
        // on a screen of two dots to the logical pixel.
        let node = ComputedNode {
            size: Vec2::new(200.0, 100.0),
            inverse_scale_factor: 0.5,
            ..default()
        };
        let place = UiGlobalTransform::from_translation(Vec2::new(300.0, 400.0));
        assert_eq!(within(&node, &place, Vec2::new(150.0, 200.0)), Vec2::ZERO);
        assert_eq!(within(&node, &place, Vec2::new(200.0, 200.0)), Vec2::new(1.0, 0.0));
        assert_eq!(within(&node, &place, Vec2::new(150.0, 175.0)), Vec2::new(0.0, -1.0));
        assert_eq!(within(&node, &place, Vec2::new(100.0, 225.0)), Vec2::new(-1.0, 1.0));
    }

    #[test]
    fn the_window_opens_only_with_the_brush_in_hand_and_shuts_when_it_is_put_down() {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        world.init_resource::<Picker>();
        world.init_resource::<DialogUp>();
        world.init_resource::<ButtonInput<KeyCode>>();
        world.insert_resource(Customize::default());
        let open = |world: &mut World| {
            world.run_system_once(act).expect("acted");
            (world.resource::<Picker>().open, world.resource::<DialogUp>().0)
        };
        // Asked for with edit mode off, or with the hand: nothing.
        world.resource_mut::<Picker>().ask();
        assert_eq!(open(&mut world), (false, false));
        world.resource_mut::<Customize>().begin();
        world.resource_mut::<Picker>().ask();
        assert_eq!(open(&mut world), (false, false));
        // With the brush: up, and nothing else can be pressed.
        world.resource_mut::<Customize>().take(super::super::Wants::Brush);
        world.resource_mut::<Picker>().ask();
        assert_eq!(open(&mut world), (true, true));
        assert_eq!(open(&mut world), (true, true));
        // Its cross: away.
        world.resource_mut::<Picker>().closed = true;
        assert_eq!(open(&mut world), (false, false));
        // Up again, and the brush put down (you walked off your plot):
        // away.
        world.resource_mut::<Picker>().ask();
        assert_eq!(open(&mut world), (true, true));
        world.resource_mut::<Customize>().take(super::super::Wants::Off);
        assert_eq!(open(&mut world), (false, false));
        // Not while something else is up.
        world.resource_mut::<Customize>().begin();
        world.resource_mut::<Customize>().take(super::super::Wants::Brush);
        world.resource_mut::<DialogUp>().0 = true;
        world.resource_mut::<Picker>().ask();
        assert_eq!(open(&mut world), (false, true));
    }
}
