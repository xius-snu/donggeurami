//! What Blend In puts on the screen besides its window (`choose`): the
//! plate top centre, as Bridge Wars' board is, saying what is going on and
//! the time it has left by the server's clock, "Seek 2:14" or "Stay hidden
//! 2:14", with the round under it; the points top right, a row to each
//! person, the one hiding in the game's colour; top left, while you are
//! dressed as someone else, that name on your plate with "(you)" under it,
//! in place of your own; a seeker's screen black through the hide, "The
//! hider is getting into place" and the seconds on it, under the plate (the
//! blindfold's idea); and, stunned, "Wrong!" counting down over the
//! blindfold (`gear`), which leaves a seeker a small circle round them.
//!
//! Everything here is the plate's grey and the file's words and colours
//! (`words.blend_in`, `colours.blend_in`), written only as it changes.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use roundtown_net::blend_in::BlendStage;

use super::Blend;
use crate::builder::Fill;
use crate::clock::TownClock;
use crate::hud::{self, TakesPress};
use crate::table::{self, InTable, TableView};
use crate::tags::HidesNames;
use crate::ui::{UiChanged, fill, ui};

/// Sizes, as shares of the short side of the screen: the plate's words and
/// the line under them, as Bridge Wars' board's; your name on its plate, as
/// the town's, and "(you)" under it; a row of the points; the words on a
/// seeker's black screen, the line and the seconds under it; and "Wrong!".
const BOARD_TEXT: f32 = 4.8;
const BOARD_CAPTION: f32 = 3.4;
const NAME_TEXT: f32 = 4.2;
const YOU_TEXT: f32 = 2.6;
const SCORE_TEXT: f32 = 3.0;
const BLIND_LINE: f32 = 4.5;
const BLIND_COUNT: f32 = 9.0;
const WRONG_TEXT: f32 = 7.0;
/// How far down the screen "Wrong!" is, as a share of its height: over the
/// circle a stunned seeker is left to see round them.
const WRONG_DOWN: f32 = 22.0;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(skip)
        .add_systems(Update, (show_plates, show_scores, board, dark, wrong, hush_buttons).after(super::arm))
        // The camera's place for this frame is set by then.
        .add_systems(Update, blindfold.after(crate::follow_camera));
}

/// Your name on its plate top left, the town's own (`hud::name_plate`),
/// and under it, while you are dressed as someone else, the name you go by
/// with "(you)" under that, in its place: under `left`, the bar's left
/// column.
pub(super) fn plates(commands: &mut Commands, left: Entity) {
    let colours = ui().colours;
    commands.spawn((ChildOf(left), OwnPlate, Node::default(), Pickable::IGNORE, children![hud::name_plate()]));
    commands.spawn((
        ChildOf(left),
        GuisePlate,
        Node {
            display: Display::None,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            padding: UiRect::axes(Val::VMin(2.4), Val::VMin(1.0)),
            border_radius: BorderRadius::all(Val::VMin(1.6)),
            ..default()
        },
        BackgroundColor(colours.hud.name_plate),
        Pickable::IGNORE,
        children![
            (
                GuiseName,
                Text::new(""),
                crate::font::sized(NAME_TEXT),
                TextColor(colours.text),
                Pickable::IGNORE,
            ),
            (
                Text::new(ui().words.blend_in.you.clone()),
                crate::font::sized(YOU_TEXT),
                TextColor(colours.text_soft),
                Pickable::IGNORE,
            ),
        ],
    ));
}

/// Your own plate, the town's, put away while you go by another name.
#[derive(Component)]
struct OwnPlate;

/// The plate with the name you go by in the game, and its words.
#[derive(Component)]
struct GuisePlate;
#[derive(Component)]
struct GuiseName;

/// The points top right, under `right`, the bar's right column: a plate of
/// the board's grey, a row to each person, filled once the game is played
/// ([`show_scores`]).
pub(super) fn scores(commands: &mut Commands, right: Entity) {
    commands.spawn((
        ChildOf(right),
        ScoreList,
        HidesNames,
        Visibility::Hidden,
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::VMin(2.2), Val::VMin(1.2)),
            row_gap: Val::VMin(0.5),
            min_width: Val::VMin(20.0),
            border_radius: BorderRadius::all(Val::VMin(1.6)),
            ..default()
        },
        BackgroundColor(ui().colours.panel),
        Pickable::IGNORE,
    ));
}

/// The points top right, and what they were last written as: each
/// person's name, their points, and whether they hide this round.
#[derive(Component)]
struct ScoreList;
#[derive(Component, PartialEq)]
struct Scored(Vec<(String, u8, bool)>);

/// Puts your plate as the game has you: the name you go by, with "(you)"
/// under it, from the first hide to the end, in your own plate's place;
/// your own otherwise. The one not shown takes no room (its node is laid
/// out again only as that changes).
fn show_plates(
    blend: Res<Blend>,
    mut own: Query<&mut Node, (With<OwnPlate>, Without<GuisePlate>)>,
    mut guise: Query<&mut Node, (With<GuisePlate>, Without<OwnPlate>)>,
    mut names: Query<&mut Text, With<GuiseName>>,
) {
    let going_by = blend.worn.as_ref().filter(|_| blend.playing().is_some()).map(|(name, _)| name);
    let (mine, theirs) = if going_by.is_some() {
        (Display::None, Display::Flex)
    } else {
        (Display::Flex, Display::None)
    };
    for mut plate in &mut own {
        if plate.display != mine {
            plate.display = mine;
        }
    }
    for mut plate in &mut guise {
        if plate.display != theirs {
            plate.display = theirs;
        }
    }
    if let Some(name) = going_by {
        for mut text in &mut names {
            if text.0 != *name {
                text.0.clone_from(name);
            }
        }
    }
}

/// Writes the points top right as they change, and on a list put up
/// afresh (the bar is, as the file changes on desktop): each person by
/// the name they sit at the table under, the one hiding this round in the
/// game's colour, from the first round to the end.
fn show_scores(
    blend: Res<Blend>,
    view: Res<TableView>,
    mut lists: Query<(Entity, &mut Visibility, Option<&Scored>), With<ScoreList>>,
    mut commands: Commands,
) {
    let Some(round) = blend.round.as_ref().filter(|_| blend.seat.is_some()) else {
        for (_, mut shown, _) in &mut lists {
            shown.set_if_neq(Visibility::Hidden);
        }
        return;
    };
    let playing = round.stage != BlendStage::Over;
    let scored: Vec<(String, u8, bool)> = round
        .points
        .iter()
        .enumerate()
        .map(|(seat, &points)| {
            let name = view.members.get(seat).map(|member| member.name.clone()).unwrap_or_default();
            (name, points, playing && usize::from(round.hider) == seat)
        })
        .collect();
    let file = ui();
    for (list, mut shown, was) in &mut lists {
        shown.set_if_neq(Visibility::Inherited);
        if was.is_some_and(|was| was.0 == scored) {
            continue;
        }
        commands.entity(list).despawn_children().insert(Scored(scored.clone()));
        for (name, points, hides) in &scored {
            let colour = if *hides { file.colours.blend_in.accent } else { file.colours.text };
            commands.spawn((
                ChildOf(list),
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: Val::VMin(3.0),
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    hud::words(name.clone(), SCORE_TEXT, colour),
                    hud::words(points.to_string(), SCORE_TEXT, colour),
                ],
            ));
        }
    }
}

/// Puts the speech bubble and the chat's keyboard away while the game has
/// the chat hushed, from the hide to the end of the seek: there is nothing
/// to say then, and a button that says nothing is no button. Only Blend
/// In's own ([`super::ForTheChat`]), and written only as it changes.
fn hush_buttons(hushed: Res<crate::chat::Hushed>, mut buttons: Query<&mut Visibility, With<super::ForTheChat>>) {
    let shown = if hushed.0 { Visibility::Hidden } else { Visibility::Inherited };
    for mut button in &mut buttons {
        button.set_if_neq(shown);
    }
}

/// The plate top centre while the game is played, the column it stands in
/// with the line under it, and what it says.
#[derive(Component)]
struct Board;
#[derive(Component)]
struct BoardColumn;
#[derive(Component)]
struct BoardText;
#[derive(Component)]
struct BoardCaption;

/// A tap on the plate: the round on, for trying the game out, as a tap on
/// a table's board is ([`table::Ask::Skip`]; the server allows it only
/// with `RT_ALLOW_SKIP`).
fn skip(click: On<Pointer<Click>>, boards: Query<(), With<Board>>, mut asks: MessageWriter<table::Ask>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if boards.contains(click.entity) {
        asks.write(table::Ask::Skip);
    }
}

/// What the plate says, and the line under it, at `now` by the server's
/// clock: what is going on for you, and the time it has left; the round.
fn said(blend: &Blend, now: f64) -> Option<(String, String)> {
    let round = blend.playing()?;
    let file = ui();
    let words = &file.words.blend_in;
    let time = table::clock((round.until - now).max(0.0) as f32);
    let hiding = blend.hiding();
    let last = round.round + 1 >= round.rounds;
    let template = match round.stage {
        BlendStage::Choose => &words.choosing,
        BlendStage::Hide if hiding => &words.hide,
        BlendStage::Hide => &words.hiding,
        BlendStage::Seek if hiding => &words.stay_hidden,
        BlendStage::Seek => &words.seek,
        BlendStage::Shown if last => &words.last_round,
        BlendStage::Shown => &words.next_round,
        BlendStage::Over => return None,
    };
    let caption = fill(&words.round, &[("round", &(round.round + 1).to_string()), ("rounds", &round.rounds.to_string())]);
    Some((fill(template, &[("time", &time)]), caption))
}

/// Puts the plate up top centre while the game is played, as Bridge Wars'
/// board is, and writes on it what it says as that changes; takes it away
/// as the game ends, for the table's results; and puts it up again in the
/// file's colours as the file changes on desktop.
#[allow(clippy::too_many_arguments)]
fn board(
    blend: Res<Blend>,
    town: Res<TownClock>,
    changed: Res<UiChanged>,
    boards: Query<Entity, With<BoardColumn>>,
    mut texts: Query<&mut Text, (With<BoardText>, Without<BoardCaption>)>,
    mut captions: Query<&mut Text, (With<BoardCaption>, Without<BoardText>)>,
    mut commands: Commands,
) {
    let said = said(&blend, town.now());
    let restyled = changed.is_changed() && !changed.is_added();
    if said.is_none() || restyled {
        for board in &boards {
            commands.entity(board).despawn();
        }
    }
    let Some((line, caption)) = said else {
        return;
    };
    if boards.is_empty() || restyled {
        let colours = ui().colours;
        commands
            .spawn((
                BoardColumn,
                InTable,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::VMin(hud::EDGE),
                    left: Val::ZERO,
                    right: Val::ZERO,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::VMin(0.8),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|column| {
                column.spawn((
                    Board,
                    TakesPress,
                    Button,
                    HidesNames,
                    Node {
                        padding: UiRect::axes(Val::VMin(3.0), Val::VMin(0.7)),
                        border_radius: BorderRadius::all(Val::VMin(3.0)),
                        ..default()
                    },
                    BackgroundColor(colours.panel),
                    Fill {
                        idle: colours.panel,
                        lit: colours.panel_lit,
                    },
                    children![(BoardText, hud::words(line, BOARD_TEXT, colours.text))],
                ));
                column.spawn((BoardCaption, hud::words(caption, BOARD_CAPTION, colours.text)));
            });
        return;
    }
    for mut text in &mut texts {
        if text.0 != line {
            text.0.clone_from(&line);
        }
    }
    for mut text in &mut captions {
        if text.0 != caption {
            text.0.clone_from(&caption);
        }
    }
}

/// A seeker's screen through the hide, and the seconds on it.
#[derive(Component)]
struct Dark;
#[derive(Component)]
struct DarkCount;

/// Blacks a seeker's screen out through the hide, under the plate and the
/// buttons and over the world and its names, as the blindfold is: "The
/// hider is getting into place", and the seconds left.
fn dark(
    blend: Res<Blend>,
    town: Res<TownClock>,
    darks: Query<Entity, With<Dark>>,
    mut counts: Query<&mut Text, With<DarkCount>>,
    mut commands: Commands,
) {
    let hidden = blend
        .playing()
        .filter(|round| round.stage == BlendStage::Hide && !blend.hiding())
        .map(|round| ((round.until - town.now()).max(0.0).ceil() as u32).to_string());
    let Some(seconds) = hidden else {
        for dark in &darks {
            commands.entity(dark).despawn();
        }
        return;
    };
    if darks.is_empty() {
        let file = ui();
        commands.spawn((
            Dark,
            InTable,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(2.0),
                ..default()
            },
            BackgroundColor(file.colours.blend_in.blind),
            // As the blindfold is: over the names in the world, under the
            // screen's own.
            GlobalZIndex(-1),
            ZIndex(1),
            Pickable::IGNORE,
            children![
                hud::words(file.words.blend_in.blind.clone(), BLIND_LINE, file.colours.text),
                (DarkCount, hud::words(seconds, BLIND_COUNT, file.colours.gold)),
            ],
        ));
        return;
    }
    for mut text in &mut counts {
        if text.0 != seconds {
            text.0.clone_from(&seconds);
        }
    }
}

/// "Wrong!" and the seconds left of a stun, while you are stunned.
#[derive(Component)]
struct Wrong;

/// Says over the blindfold that you struck one of the town's own, and how
/// long you are stunned for, counting down (`Tell::Stunned`, by the
/// server's clock); gone as the stun is.
fn wrong(
    blend: Res<Blend>,
    town: Res<TownClock>,
    mut words: Query<(Entity, &mut Text), With<Wrong>>,
    headlines: Query<Entity, With<super::Headline>>,
    mut commands: Commands,
) {
    let now = town.now();
    let left = blend
        .stunned
        .filter(|_| blend.stunned_at(now))
        .map(|until| fill(&ui().words.blend_in.wrong, &[("seconds", &((until - now).max(0.0).ceil() as u32).to_string())]));
    let Some(said) = left else {
        for (shown, _) in &words {
            commands.entity(shown).despawn();
        }
        return;
    };
    let mut any = false;
    for (_, mut text) in &mut words {
        any = true;
        if text.0 != said {
            text.0.clone_from(&said);
        }
    }
    if !any {
        // "Seek!", if it is still up, would be under it: gone.
        for headline in &headlines {
            commands.entity(headline).try_despawn();
        }
        let colours = ui().colours;
        commands.spawn((
            Wrong,
            InTable,
            hud::words(said, WRONG_TEXT, colours.warn),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(WRONG_DOWN),
                left: Val::ZERO,
                right: Val::ZERO,
                ..default()
            },
        ));
    }
}

/// The dark over the screen while you are stunned, all but a circle round
/// you.
#[derive(Component)]
struct Blindfold;

/// Blinds you while a wrong guess has you stunned, as a blindness trap
/// blinds you in Bridge Wars (`gear`'s blindfold, whose way of drawing it
/// this is: Hajun, "they can only see a very small circle around their
/// character and the rest of their screen is black except their ui
/// menu/buttons/minimap"): the screen covered in the file's
/// `colours.blend_in.blind` but for a circle round you as far as a blinded
/// player sees in Bridge Wars (`BLIND_SIGHT`), under the screen's own and
/// over the world and the names in it. Its own here rather than `gear`'s,
/// which goes by Bridge Wars' kit and says Bridge Wars' words as it comes
/// ("A trap has blinded you!"). The node itself written only as the window
/// or the camera's distance changes, its place as you move.
#[allow(clippy::type_complexity)]
fn blindfold(
    blend: Res<Blend>,
    town: Res<TownClock>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &Transform), With<crate::ThirdPersonCamera>>,
    me: Query<&Transform, With<crate::Player>>,
    mut folds: Query<(Entity, &mut Node, &mut UiTransform), With<Blindfold>>,
    mut commands: Commands,
) {
    let seen = blend.stunned_at(town.now()).then_some(()).and_then(|()| {
        let body = me.single().ok()?;
        let (camera, eye) = cameras.single().ok()?;
        let window = windows.single().ok()?;
        let eye_place = GlobalTransform::from(*eye);
        let middle = body.translation + Vec3::Y * BLIND_CENTRE;
        let centre = camera.world_to_viewport(&eye_place, middle).ok()?;
        let edge = camera.world_to_viewport(&eye_place, middle + eye.right() * BLIND_SIGHT).ok()?;
        Some((centre, centre.distance(edge), Vec2::new(window.width(), window.height())))
    });
    let Some((centre, radius, screen)) = seen else {
        for (fold, ..) in &folds {
            commands.entity(fold).despawn();
        }
        return;
    };
    // As thick as reaches every corner of the screen from anywhere on it.
    let (radius, thick) = (radius.round(), screen.length().ceil());
    let side = 2.0 * (radius + thick);
    let place = UiTransform {
        translation: Val2::px(centre.x - side * 0.5, centre.y - side * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    };
    let Ok((_, mut node, mut placed)) = folds.single_mut() else {
        commands.spawn((
            Blindfold,
            InTable,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                width: Val::Px(side),
                height: Val::Px(side),
                border: UiRect::all(Val::Px(thick)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(ui().colours.blend_in.blind),
            place,
            GlobalZIndex(-1),
            ZIndex(1),
            Pickable::IGNORE,
        ));
        return;
    };
    if node.width != Val::Px(side) || node.border.left != Val::Px(thick) {
        node.width = Val::Px(side);
        node.height = Val::Px(side);
        node.border = UiRect::all(Val::Px(thick));
    }
    placed.set_if_neq(place);
}

/// How far round you a stunned seeker sees, in metres, and how high over
/// your feet the middle of it is: a blinded player's in Bridge Wars, round
/// your chest, where the camera looks.
const BLIND_SIGHT: f32 = roundtown_net::bridge_wars::BLIND_SIGHT;
const BLIND_CENTRE: f32 = 0.9;

/// A dice, seen from the front: a block with its corners rounded and five
/// spots cut out of it, white on clear for a button to tint (mine: the dice
/// that throws a name).
pub(super) fn dice() -> Image {
    drawn(|at| {
        const ROUND: f32 = 0.18;
        const SPOT: f32 = 0.13;
        let block = box_distance(at, Vec2::splat(0.8 - ROUND)) - ROUND;
        let spots = [Vec2::ZERO, Vec2::new(-0.4, 0.4), Vec2::new(0.4, 0.4), Vec2::new(-0.4, -0.4), Vec2::new(0.4, -0.4)]
            .into_iter()
            .map(|spot| at.distance(spot) - SPOT)
            .fold(f32::INFINITY, f32::min);
        block.max(-spots)
    })
}

/// A tick, white on clear: by each who has chosen, in the window.
pub(super) fn tick() -> Image {
    drawn(|at| {
        const THICK: f32 = 0.13;
        let (start, bend, end) = (Vec2::new(-0.68, 0.02), Vec2::new(-0.2, -0.5), Vec2::new(0.7, 0.55));
        segment_distance(at, start, bend).min(segment_distance(at, bend, end)) - THICK
    })
}

/// How far `at` is from the edge of a box `reach` out from its middle each
/// way: less than nothing inside it.
fn box_distance(at: Vec2, reach: Vec2) -> f32 {
    let off = at.abs() - reach;
    off.max(Vec2::ZERO).length() + off.x.max(off.y).min(0.0)
}

/// How far `at` is from the line from `a` to `b`.
fn segment_distance(at: Vec2, a: Vec2, b: Vec2) -> f32 {
    let along = b - a;
    let share = ((at - a).dot(along) / along.length_squared()).clamp(0.0, 1.0);
    at.distance(a + along * share)
}

/// A picture of a shape, white on clear, edged by how far each pixel is
/// from it, as `hud`'s are: `outside` says how far a point is from its
/// edge, in shares of half the picture from its middle, y up, less than
/// nothing inside.
fn drawn(outside: impl Fn(Vec2) -> f32) -> Image {
    const SIZE: u32 = 128;
    let pixel = 2.0 / SIZE as f32;
    let mut data = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for row in 0..SIZE {
        for column in 0..SIZE {
            let at = Vec2::new((column as f32 + 0.5) * pixel - 1.0, 1.0 - (row as f32 + 0.5) * pixel);
            let cover = (0.5 - outside(at) / pixel).clamp(0.0, 1.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// How solid `image` is at (`x`, `y`), each from 0 to 1 across it,
    /// from its top left.
    fn alpha_at(image: &Image, x: f32, y: f32) -> u8 {
        let size = image.width();
        let (column, row) = ((x * size as f32) as u32, (y * size as f32) as u32);
        image.data.as_ref().unwrap()[((row * size + column) * 4 + 3) as usize]
    }

    #[test]
    fn the_dice_is_a_block_with_its_spots_cut_out_and_the_tick_a_bent_stroke() {
        let dice = dice();
        assert_eq!(alpha_at(&dice, 0.5, 0.5), 0, "the middle spot");
        assert_eq!(alpha_at(&dice, 0.3, 0.3), 0, "a corner spot");
        assert_eq!(alpha_at(&dice, 0.5, 0.3), 255, "between spots");
        assert_eq!(alpha_at(&dice, 0.02, 0.02), 0, "outside its rounded corner");
        let tick = tick();
        // Its bend low and to the left, its long arm up to the right.
        assert_eq!(alpha_at(&tick, 0.4, 0.75), 255);
        assert_eq!(alpha_at(&tick, 0.82, 0.25), 255);
        assert_eq!(alpha_at(&tick, 0.75, 0.75), 0);
        assert_eq!(alpha_at(&tick, 0.2, 0.2), 0);
    }
}
