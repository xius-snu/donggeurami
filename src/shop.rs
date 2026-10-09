//! House Builder's shop: the hammer button, while you build, and the window it
//! opens of everything there is to build with (`build::PIECES`).
//!
//! The window has a tab for each kind of piece along its top, and a grid of
//! them under it, three across and two down, which scrolls to show more: drag
//! it with a finger, or turn a mouse wheel over it. Tap a piece and the window
//! shuts, and the piece comes up in front of you to be put down (`build`).
//! What cannot be chosen just now is greyed out, with the reason over the
//! grid: a second house, or a door with no house to put it in.
//!
//! While the window is up it is a dialog like House Builder's others
//! ([`DialogUp`]): you stand still, and presses go to it alone. The cross on
//! its corner, a tap on the dark round it, or on desktop `Escape`, shuts it.
//! The hammer is put away while something is still being put down: that comes
//! first.

use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;

use crate::build::{Ghost, Mount, PIECES, Piece, PieceDef, Shadow, Stock, Tab};
use crate::builder::{self, Building, DIALOG_LAYER, DIM, InGame, QUIET};
use crate::hud::{self, DOOR, DialogUp, GOLD, TakesPress};

/// Where the hammer sits: below and in from where the button that goes home
/// sits in the town, which is put away while you build.
const HAMMER_TOP: f32 = hud::EDGE + HAMMER + 3.0;
const HAMMER_RIGHT: f32 = hud::EDGE + 6.0;
/// How big the hammer is: a little bigger than the button that goes home.
const HAMMER: f32 = 13.5;
/// The window's share of the screen, and how far down from the top of it the
/// window starts, as a share of its short side: under the time and the theme,
/// which stay in sight while you shop.
const WIDTH: f32 = 65.0;
const HEIGHT: f32 = 68.0;
const TOP: f32 = 22.0;
/// Sizes, as shares of the short side of the screen.
const PADDING: f32 = 2.6;
const TAB_TEXT: f32 = 3.4;
const NOTE_TEXT: f32 = 3.0;
const NAME_TEXT: f32 = 3.0;
const CELL_GAP: f32 = 0.8;
const CLOSE: f32 = 9.0;
/// How far a press may be dragged and still be a tap on what it began on,
/// rather than a drag to scroll the grid.
const TAP_SLOP: f32 = 16.0;

const WINDOW: Color = Color::srgba(0.04, 0.10, 0.18, 0.86);
const CARD: Color = Color::srgba(1.0, 1.0, 1.0, 0.1);
const CARD_LIT: Color = Color::srgba(1.0, 1.0, 1.0, 0.22);
const CARD_OFF: Color = Color::srgba(1.0, 1.0, 1.0, 0.04);
const PICTURE_OFF: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);
const NAME_OFF: Color = Color::srgba(1.0, 1.0, 1.0, 0.35);
const NOTE: Color = Color::srgb(1.0, 0.82, 0.6);
const HANDLE: Color = Color::srgb(0.96, 0.78, 0.52);

/// A piece chosen in the shop, to be put down.
#[derive(Message, Clone, Copy)]
pub(crate) struct Chosen(pub &'static PieceDef);

/// The shop window, while it is up, and the tab it shows.
#[derive(Resource, Default)]
struct Shop {
    window: Option<Entity>,
    tab: Tab,
    /// How far the press now down has been dragged, so that a drag to scroll
    /// is not also a tap on the piece it began on.
    dragged: f32,
}

/// The hammer button.
#[derive(Component)]
struct Hammer;

/// What pressing something in the window does.
#[derive(Component, Clone, Copy)]
enum ShopButton {
    Close,
    Tab(Tab),
    /// A piece, and whether it can be chosen just now.
    Piece(&'static PieceDef, bool),
}

/// The grid of pieces.
#[derive(Component)]
struct Grid;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Chosen>()
        .init_resource::<Shop>()
        .add_observer(press)
        .add_observer(start_press)
        .add_systems(Startup, spawn_hammer)
        .add_systems(
            Update,
            (show_hammer, light_hammer, light_cards, shut_when_done).after(builder::GameSystems),
        );
}

// ---------------------------------------------------------------- the hammer

fn spawn_hammer(mut commands: Commands) {
    commands
        .spawn((
            Hammer,
            TakesPress,
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(HAMMER_TOP),
                right: Val::VMin(HAMMER_RIGHT),
                width: Val::VMin(HAMMER),
                height: Val::VMin(HAMMER),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(hud::button_fill(Interaction::None)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.5)),
            Visibility::Hidden,
            children![(
                hud::picture(),
                // Upright, then leaning right, the way a hammer is drawn.
                UiTransform::from_rotation(Rot2::degrees(45.0)),
                hammer(),
            )],
        ))
        .observe(open);
}

/// A hammer, upright: a handle, and a head across the top of it.
fn hammer() -> impl Bundle {
    children![
        (
            Node {
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..hud::frame(45.5, 34.0, 9.0, 50.0)
            },
            BackgroundColor(HANDLE),
            Pickable::IGNORE,
        ),
        (
            Node {
                border_radius: BorderRadius::all(Val::Percent(22.0)),
                ..hud::frame(23.0, 17.0, 54.0, 19.0)
            },
            BackgroundColor(hud::ICON),
            Pickable::IGNORE,
        ),
    ]
}

/// The hammer is there while you build, unless the window is up or something
/// is still being put down.
fn show_hammer(
    building: Res<Building>,
    shop: Res<Shop>,
    ghosts: Query<(), With<Ghost>>,
    mut hammers: Query<&mut Visibility, With<Hammer>>,
) {
    let shown = building.0.is_some() && shop.window.is_none() && ghosts.is_empty();
    for mut visibility in &mut hammers {
        visibility.set_if_neq(if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

fn light_hammer(
    mut hammers: Query<(&Interaction, &mut BackgroundColor), (With<Hammer>, Changed<Interaction>)>,
) {
    for (interaction, mut fill) in &mut hammers {
        fill.set_if_neq(BackgroundColor(hud::button_fill(*interaction)));
    }
}

fn open(
    click: On<Pointer<Click>>,
    building: Res<Building>,
    stock: Res<Stock>,
    pieces: Query<&Piece, Without<Shadow>>,
    mut shop: ResMut<Shop>,
    mut up: ResMut<DialogUp>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary || shop.window.is_some() || building.0.is_none() {
        return;
    }
    let house = pieces.iter().any(|piece| piece.def.mount == Mount::House);
    shop.window = Some(spawn_window(&mut commands, &stock, shop.tab, house));
    up.set_if_neq(DialogUp(true));
}

// ---------------------------------------------------------------- the window

/// The window, showing `tab`, over the whole screen dimmed. `house` says
/// whether there is a house on the plot already.
fn spawn_window(commands: &mut Commands, stock: &Stock, tab: Tab, house: bool) -> Entity {
    let dark = commands
        .spawn((
            InGame,
            ShopButton::Close,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            BackgroundColor(DIM),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let panel = commands
        .spawn((
            ChildOf(dark),
            Node {
                width: Val::Percent(WIDTH),
                height: Val::Percent(HEIGHT),
                margin: UiRect::top(Val::VMin(TOP)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::VMin(1.6),
                padding: UiRect::all(Val::VMin(PADDING)),
                border_radius: BorderRadius::all(Val::VMin(3.5)),
                ..default()
            },
            BackgroundColor(WINDOW),
        ))
        .id();

    commands
        .spawn((
            ChildOf(panel),
            Node {
                column_gap: Val::VMin(1.2),
                // Clear of the cross on the corner.
                margin: UiRect::right(Val::VMin(CLOSE * 0.5)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|tabs| {
            for each in Tab::ALL.into_iter().filter(|each| each.sold()) {
                tabs.spawn(tab_button(each, each == tab));
            }
        });

    let able = |def: &PieceDef| match def.mount {
        Mount::House => !house,
        Mount::Door | Mount::Window | Mount::Hanging => house,
        Mount::Floor => true,
    };
    let offered: Vec<&'static PieceDef> = PIECES.iter().filter(|def| def.tab == tab).collect();
    if let Some(why) = note(tab, house, offered.iter().any(|def| !able(def))) {
        commands.spawn((ChildOf(panel), hud::words(why, NOTE_TEXT, NOTE)));
    }

    commands
        .spawn((
            ChildOf(panel),
            Grid,
            ScrollArea,
            Node {
                flex_grow: 1.0,
                min_height: Val::ZERO,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ))
        .observe(drag_grid)
        .with_children(|grid| {
            for def in offered {
                grid.spawn(cell(stock, def, able(def)));
            }
        });

    commands.spawn((ChildOf(panel), close_button()));
    dark
}

/// Why some of what `tab` shows cannot be chosen, if any of it cannot.
fn note(tab: Tab, house: bool, any_off: bool) -> Option<&'static str> {
    if !any_off {
        return None;
    }
    Some(match tab {
        Tab::Foundation if house => "One house to a plot: bin yours to choose another.",
        Tab::Openings => "Put a house down first: these go in its walls.",
        _ => "Put a house down first to hang things on its walls.",
    })
}

fn tab_button(tab: Tab, chosen: bool) -> impl Bundle {
    let (fill, ink) = if chosen {
        (GOLD, DOOR)
    } else {
        (QUIET, Color::WHITE)
    };
    (
        ShopButton::Tab(tab),
        TakesPress,
        Button,
        Node {
            flex_grow: 1.0,
            flex_basis: Val::ZERO,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::VMin(1.0), Val::VMin(1.4)),
            border_radius: BorderRadius::all(Val::VMin(2.2)),
            ..default()
        },
        BackgroundColor(fill),
        children![label(tab.title(), TAB_TEXT, ink)],
    )
}

/// One piece in the grid: its picture and its name, on a card that takes up a
/// third of the grid's width and half its height.
fn cell(stock: &Stock, def: &'static PieceDef, able: bool) -> impl Bundle {
    let (card, picture, name) = if able {
        (CARD, Color::WHITE, Color::WHITE)
    } else {
        (CARD_OFF, PICTURE_OFF, NAME_OFF)
    };
    (
        Node {
            width: Val::Percent(100.0 / 3.0),
            height: Val::Percent(50.0),
            padding: UiRect::all(Val::VMin(CELL_GAP)),
            ..default()
        },
        Pickable::IGNORE,
        children![(
            ShopButton::Piece(def, able),
            TakesPress,
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(0.6),
                padding: UiRect::all(Val::VMin(1.0)),
                border_radius: BorderRadius::all(Val::VMin(2.4)),
                ..default()
            },
            BackgroundColor(card),
            children![
                (
                    ImageNode::new(stock.picture(def)).with_color(picture),
                    Node {
                        height: Val::Percent(74.0),
                        aspect_ratio: Some(1.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ),
                label(def.name, NAME_TEXT, name),
            ],
        )],
    )
}

/// The round cross on the window's top right corner.
fn close_button() -> impl Bundle {
    (
        ShopButton::Close,
        TakesPress,
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::VMin(-CLOSE * 0.35),
            right: Val::VMin(-CLOSE * 0.35),
            width: Val::VMin(CLOSE),
            height: Val::VMin(CLOSE),
            border: UiRect::all(Val::Px(2.0)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.72, 0.2, 0.2, 0.92)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        children![
            (
                hud::shape(22.0, 44.0, 56.0, 12.0),
                UiTransform::from_rotation(Rot2::degrees(45.0)),
            ),
            (
                hud::shape(22.0, 44.0, 56.0, 12.0),
                UiTransform::from_rotation(Rot2::degrees(-45.0)),
            ),
        ],
    )
}

/// Words in the window, which has a colour of its own behind them and needs
/// no shadow to read.
fn label(text: &str, size_vmin: f32, colour: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::VMin(size_vmin),
            ..default()
        },
        TextColor(colour),
        TextLayout::justify(Justify::Center),
        Pickable::IGNORE,
    )
}

// ---------------------------------------------------------------- pressing

/// A new press: nothing dragged yet.
fn start_press(_press: On<Pointer<Press>>, mut shop: ResMut<Shop>) {
    shop.dragged = 0.0;
}

/// Scrolls the grid with a finger, or a mouse held down, dragged over it.
fn drag_grid(
    drag: On<Pointer<Drag>>,
    mut grids: Query<(&ComputedNode, &mut ScrollPosition), With<Grid>>,
    mut shop: ResMut<Shop>,
) {
    shop.dragged = shop.dragged.max(drag.distance.length());
    let Ok((node, mut scroll)) = grids.get_mut(drag.entity) else {
        return;
    };
    let shown = node.size().y * node.inverse_scale_factor();
    let whole = node.content_size().y * node.inverse_scale_factor();
    let most = (whole - shown).max(0.0);
    let to = (scroll.y - drag.delta.y).clamp(0.0, most);
    if scroll.y != to {
        scroll.y = to;
    }
}

/// Whatever in the window was tapped or clicked. Only what the press landed
/// on, not what it passed up through, so that a tap on a card is not also a
/// tap on the dark round the window.
fn press(
    click: On<Pointer<Click>>,
    buttons: Query<&ShopButton>,
    building: Res<Building>,
    stock: Res<Stock>,
    pieces: Query<&Piece, Without<Shadow>>,
    mut shop: ResMut<Shop>,
    mut up: ResMut<DialogUp>,
    mut chosen: MessageWriter<Chosen>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    let Ok(&button) = buttons.get(click.entity) else {
        return;
    };
    match button {
        ShopButton::Close => shut(&mut shop, &mut up, &mut commands),
        ShopButton::Tab(tab) if tab != shop.tab => {
            // The window again, on the new tab and scrolled to its top.
            shop.tab = tab;
            if let Some(window) = shop.window.take() {
                commands.entity(window).try_despawn();
            }
            let house = pieces.iter().any(|piece| piece.def.mount == Mount::House);
            shop.window = Some(spawn_window(&mut commands, &stock, tab, house));
        }
        ShopButton::Piece(def, true) if shop.dragged <= TAP_SLOP && building.0.is_some() => {
            shut(&mut shop, &mut up, &mut commands);
            chosen.write(Chosen(def));
        }
        _ => {}
    }
}

/// Lights a card up while it is under the pointer or held.
fn light_cards(
    mut cards: Query<(&Interaction, &ShopButton, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, button, mut fill) in &mut cards {
        let lit = *interaction != Interaction::None;
        let colour = match *button {
            ShopButton::Piece(_, true) if lit => CARD_LIT,
            ShopButton::Piece(_, true) => CARD,
            ShopButton::Piece(_, false) => CARD_OFF,
            ShopButton::Tab(_) | ShopButton::Close => continue,
        };
        fill.set_if_neq(BackgroundColor(colour));
    }
}

/// Shuts the window when the time to build runs out, or on desktop, at
/// `Escape`.
fn shut_when_done(
    building: Res<Building>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut shop: ResMut<Shop>,
    mut up: ResMut<DialogUp>,
    mut commands: Commands,
) {
    let done = building.0.is_none() || keyboard.just_pressed(KeyCode::Escape);
    if shop.window.is_some() && done {
        shut(&mut shop, &mut up, &mut commands);
    }
}

fn shut(shop: &mut Shop, up: &mut DialogUp, commands: &mut Commands) {
    if let Some(window) = shop.window.take() {
        commands.entity(window).try_despawn();
    }
    if up.0 {
        up.0 = false;
    }
}
