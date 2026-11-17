//! What Bridge Race draws over the world and on the screen (`super`), all
//! of it judged from the camera fixed over you, as Bridge Wars' is:
//!
//! * **The plate top centre** ([`plate`]): the time left, "4:12", and under
//!   it each team's way across, red's and blue's side by side, a bar in
//!   its colour as long as its bridge has got toward the golden island
//!   (its plank nearest it, as a share of the 63 m of void,
//!   `RaceRound::across`), with how far it still has to go under it, or
//!   "Across!", or how many of the team stand on the golden island. On the
//!   plate's grey, as the board the table has up before the race is
//!   (Bridge Wars' own look); up from "Go!" to the end.
//! * **The ring round the golden island** ([`ring`]): over the map's own
//!   ring, the band 6 m out from its middle, an arc in the colour of a team
//!   that has every one of its players on the island, filling as its hold
//!   goes on, red's the way the clock goes from the top and blue's the
//!   other, rebuilt only as it comes to the next tenth of [`HOLD`]; unlit,
//!   so that it reads by night as by day. And over the middle of the
//!   island, "Hold! 2" in the same colour, the seconds the team still has
//!   to hold it ([`hold_words`]).
//! * **The way to the golden island** ([`point`]): while it is off the
//!   screen, an arrow at the edge of the screen pointing to it, in its
//!   gold, with how far it is from you to its shore ("63m") just inside it
//!   (the compass's idea, `compass`).
//! * **A grant**: "+3m" rising off you in the planks' colour ([`rising`],
//!   [`rise`]), as a number rises off a body hit in Bridge Wars.
//! * **What has happened**, a line across the upper half for a few
//!   seconds, Bridge Wars' own look (`arena::alert`), with a picture after
//!   it where it is about one: "Dynamite!" as a stick comes to your
//!   button, a team's bridge come across, a team holding the golden island
//!   ([`notice`], [`fade_notices`]).
//!
//! Everything here is the table's (`table::InTable`) or the fight's
//! (`arena::InArena`), and goes with it; and everything that runs every
//! frame writes only what has changed.

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use roundtown_net::bridge_race::{GOLDEN, HOLD, ISLAND_RADIUS, LIFT, the_way_across};

use super::{Racing, Say};
use crate::arena::{InArena, bar_width, clock, on_screen};
use crate::builder::DIALOG_LAYER;
use crate::hud::{self, Picture};
use crate::island::Venue;
use crate::range;
use crate::table::{InTable, TableView};
use crate::ui::{UiChanged, fill, ui};
use crate::{Dead, Player, ThirdPersonCamera};

/// Sizes on the plate, in shares of the short side of the screen: the
/// clock, as big as the board's words before the race; each team's bar,
/// long and thin, and the words under it; and the gaps between.
const CLOCK_TEXT: f32 = 4.8;
const BAR_LONG: f32 = 13.0;
const BAR_THICK: f32 = 1.3;
const PROGRESS_TEXT: f32 = 2.3;
const SIDES_APART: f32 = 3.2;

/// The ring round the golden island, in metres: from and to how far out
/// from the island's middle it is drawn, a little wider than the map's own
/// band (5.5 to 6 m out, `race_bake`) either way, so that it covers it;
/// how high its top lies over the island's, just over that band's (0.2 m
/// up), clear of it; and how finely its arc is drawn, in pieces of the
/// whole way round.
const RING_IN: f32 = 5.15;
const RING_OUT: f32 = 6.45;
const RING_OVER: f32 = 0.23;
const RING_PIECES: usize = 96;

/// "Hold! 2": how big, a little over a headline's caption; and how far
/// over the golden island's middle the point it is written at is, in
/// metres.
const HOLD_TEXT: f32 = 5.6;
const HOLD_OVER: f32 = 1.5;

/// The arrow toward the golden island: how big, as the compass's is a
/// share of the short side; how far in from the edge of the screen its
/// middle is; how big the words by it are, and how far inside the arrow
/// they begin.
const NEEDLE: f32 = 5.0;
const NEEDLE_IN: f32 = 6.0;
const NEEDLE_TEXT: f32 = 3.0;
const NEEDLE_GAP: f32 = 1.2;

/// "+3m": how big, as a number rising off a body hit is; from how high
/// over your feet it rises and how far, in metres; and for how long, in
/// seconds: a little longer than a number does, there being more to read.
const RISE_TEXT: f32 = 3.8;
const RISE_FROM: f32 = 1.8;
const RISE_BY: f32 = 1.8;
const RISE_SECS: f32 = 1.3;

/// A line across the upper half (Bridge Wars' own, `arena::alert`): how far
/// down the screen it is, in shares of its height; how big its words and
/// its picture are; how long it is up, and how long it takes to come up
/// and to go.
const NOTICE_DOWN: f32 = 24.0;
const NOTICE_TEXT: f32 = 3.6;
const NOTICE_PICTURE: f32 = 3.8;
const NOTICE_SECS: f32 = 5.0;
const NOTICE_IN: f32 = 0.3;
const NOTICE_OUT: f32 = 0.8;

/// The plate top centre, its clock, and each team's bar and words.
#[derive(Component)]
pub(super) struct Plate;
#[derive(Component)]
pub(super) struct PlateClock;
#[derive(Component)]
pub(super) struct Progress(u8);
#[derive(Component)]
pub(super) struct ProgressWords(u8);

/// The ring round the golden island, and each team's arc of it.
#[derive(Component)]
pub(super) struct Ring;
#[derive(Component)]
pub(super) struct Arc {
    team: u8,
    mesh: Handle<Mesh>,
}

/// "Hold! 2", over the golden island.
#[derive(Component)]
pub(super) struct HoldWords;

/// The arrow toward the golden island, and how far it is, by it.
#[derive(Component)]
pub(super) struct Needle;
#[derive(Component)]
pub(super) struct NeedleWords;

/// Words rising off you: where you stood, and for how long they have.
#[derive(Component)]
pub(super) struct Rising {
    at: Vec3,
    age: f32,
}

/// A line across the upper half, and how long it has been up; and the
/// picture in it.
#[derive(Component)]
pub(crate) struct Notice {
    age: f32,
}
#[derive(Component)]
pub(super) struct NoticePicture;

/// Puts up what the race draws on the screen, each piece put away until it
/// is wanted: the plate, "Hold!", the arrow and its words. The ring is put
/// up by [`ring`], which has the meshes to make it with.
pub(super) fn spawn(commands: &mut Commands, images: &mut Assets<Image>, pictures: &mut hud::Pictures) {
    let file = ui();
    let colours = file.colours;
    let accent = colours.bridge_race.accent;
    commands
        .spawn((
            InTable,
            Plate,
            Name::new("Bridge Race's plate"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::VMin(hud::EDGE),
                left: Val::ZERO,
                right: Val::ZERO,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|plate| {
            plate
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::VMin(0.7),
                        padding: UiRect::new(Val::VMin(3.0), Val::VMin(3.0), Val::VMin(0.6), Val::VMin(1.1)),
                        border_radius: BorderRadius::all(Val::VMin(3.0)),
                        ..default()
                    },
                    BackgroundColor(colours.panel),
                    Pickable::IGNORE,
                ))
                .with_children(|inside| {
                    inside.spawn((PlateClock, hud::words("", CLOCK_TEXT, colours.text)));
                    inside
                        .spawn((
                            Node {
                                column_gap: Val::VMin(SIDES_APART),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|sides| {
                            for side in 0..2u8 {
                                let ink = colours.arena.team(side);
                                sides
                                    .spawn((
                                        Node {
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Center,
                                            row_gap: Val::VMin(0.5),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ))
                                    .with_children(|column| {
                                        column
                                            .spawn((
                                                Node {
                                                    width: Val::VMin(BAR_LONG),
                                                    height: Val::VMin(BAR_THICK),
                                                    border_radius: BorderRadius::all(Val::VMin(BAR_THICK * 0.5)),
                                                    overflow: Overflow::clip(),
                                                    ..default()
                                                },
                                                BackgroundColor(colours.bridge_race.bar_back),
                                                Pickable::IGNORE,
                                            ))
                                            .with_children(|bar| {
                                                bar.spawn((
                                                    Progress(side),
                                                    Node {
                                                        width: Val::Percent(0.0),
                                                        height: Val::Percent(100.0),
                                                        ..default()
                                                    },
                                                    BackgroundColor(ink.with_alpha(1.0)),
                                                    Pickable::IGNORE,
                                                ));
                                            });
                                        column.spawn((ProgressWords(side), hud::words("", PROGRESS_TEXT, ink.with_alpha(1.0))));
                                    });
                            }
                        });
                });
        });
    commands.spawn((
        InTable,
        HoldWords,
        Name::new("Hold!"),
        hud::words("", HOLD_TEXT, accent),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            ..default()
        },
        UiTransform::default(),
        Visibility::Hidden,
        GlobalZIndex(-1),
    ));
    commands.spawn((
        InTable,
        Needle,
        Name::new("The way to the golden island"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            width: Val::VMin(NEEDLE),
            height: Val::VMin(NEEDLE),
            ..default()
        },
        ImageNode::new(pictures.of(images, Picture::Compass)).with_color(accent),
        UiTransform::default(),
        Visibility::Hidden,
        GlobalZIndex(-1),
        Pickable::IGNORE,
    ));
    commands.spawn((
        InTable,
        NeedleWords,
        hud::words("", NEEDLE_TEXT, accent),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            ..default()
        },
        UiTransform::default(),
        Visibility::Hidden,
        GlobalZIndex(-1),
    ));
}

/// What the plate says of `side`'s way across, as the round has it, at a
/// table of `seats`: how many of the team stand on the golden island, if
/// any do; across, if its bridge is; or how many metres of void it still
/// has to bridge.
fn progress_words(side: usize, across: u8, on_gold: u8, seats: usize) -> String {
    let file = ui();
    let words = &file.words.bridge_race;
    if on_gold > 0 {
        let of = (seats / 2).max(1).to_string();
        fill(&words.on_island, &[("count", &on_gold.to_string()), ("of", &of)])
    } else if across >= 100 {
        words.across.clone()
    } else {
        let left = the_way_across(side as u8) * (1.0 - f32::from(across) / 100.0);
        fill(&words.to_go, &[("metres", &format!("{:.0}", left.max(1.0).ceil()))])
    }
}

/// Keeps the plate up from "Go!" to the end, with the time left and each
/// team's way across as the round has them, each written only as it
/// changes; and puts up afresh everything this module drew, in the file's
/// colours, as the file changes on desktop.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn plate(
    racing: Res<Racing>,
    view: Res<TableView>,
    changed: Res<UiChanged>,
    mut plates: Query<&mut Visibility, With<Plate>>,
    mut clocks: Query<&mut Text, (With<PlateClock>, Without<ProgressWords>)>,
    mut bars: Query<(&Progress, &mut Node)>,
    mut said: Query<(&ProgressWords, &mut Text), Without<PlateClock>>,
    drawn: Query<Entity, Or<(With<Plate>, With<HoldWords>, With<Needle>, With<NeedleWords>, With<Ring>)>>,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<hud::Pictures>,
    mut commands: Commands,
) {
    let Some(race) = racing.0.as_ref() else {
        return;
    };
    if changed.is_changed() && !changed.is_added() {
        for piece in &drawn {
            commands.entity(piece).despawn();
        }
        spawn(&mut commands, &mut images, &mut pictures);
        return;
    }
    let round = race.round.as_ref().filter(|_| race.begun);
    for mut shown in &mut plates {
        shown.set_if_neq(if round.is_some() { Visibility::Inherited } else { Visibility::Hidden });
    }
    let Some(round) = round else {
        return;
    };
    let time = clock(f32::from(round.left));
    for mut text in &mut clocks {
        if text.0 != time {
            text.0.clone_from(&time);
        }
    }
    for (side, mut bar) in &mut bars {
        let width = bar_width(f32::from(round.across[usize::from(side.0)]) / 100.0);
        if bar.width != width {
            bar.width = width;
        }
    }
    let seats = view.members.len();
    for (side, mut text) in &mut said {
        let side = usize::from(side.0);
        let words = progress_words(side, round.across[side], round.on_gold[side], seats);
        if text.0 != words {
            text.0 = words;
        }
    }
}

/// The flat band of a ring between [`RING_IN`] and [`RING_OUT`] round its
/// middle, `share` of the way round from the top of the screen (the map's
/// -Z), the way the clock goes, or `backward` the other way: in pieces of
/// [`RING_PIECES`] to the whole, at least one, facing up.
fn arc_mesh(share: f32, backward: bool) -> Mesh {
    let pieces = ((share.clamp(0.0, 1.0) * RING_PIECES as f32).round() as usize).max(1);
    let sweep = share.clamp(0.0, 1.0) * std::f32::consts::TAU;
    let way = if backward { -1.0 } else { 1.0 };
    let mut positions = Vec::with_capacity((pieces + 1) * 2);
    for k in 0..=pieces {
        let angle = way * sweep * k as f32 / pieces as f32;
        let (sin, cos) = angle.sin_cos();
        for radius in [RING_IN, RING_OUT] {
            positions.push([sin * radius, 0.0, -cos * radius]);
        }
    }
    let mut indices = Vec::with_capacity(pieces * 6);
    for k in 0..pieces as u32 {
        let (a, b, c, d) = (2 * k, 2 * k + 1, 2 * k + 2, 2 * k + 3);
        indices.extend_from_slice(&[a, c, b, b, c, d]);
    }
    let count = positions.len();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; count])
        .with_inserted_indices(Indices::U32(indices))
}

/// The ring round the golden island: put up as the race is, on its venue
/// (shown with it, `island::show_where_i_am`), each team's arc in its own
/// mesh, made full and put away; and each arc as long as its team's hold
/// has gone ([`roundtown_net::bridge_race::RaceRound::hold`], in tenths of
/// [`HOLD`]), made again only as that comes to another tenth, and put
/// away with none. A mesh is never made with nothing in it: an arc is put
/// away rather than made empty.
#[allow(clippy::type_complexity)]
pub(super) fn ring(
    racing: Res<Racing>,
    rings: Query<(), With<Ring>>,
    mut arcs: Query<(&Arc, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drawn: Local<[u8; 2]>,
    mut commands: Commands,
) {
    let Some(race) = racing.0.as_ref() else {
        return;
    };
    if rings.is_empty() {
        let colours = ui().colours.arena;
        *drawn = [0; 2];
        commands
            .spawn((
                InTable,
                Ring,
                Name::new("The golden island's ring"),
                Venue::BridgeRace,
                Transform::from_xyz(GOLDEN.x, LIFT + RING_OVER, GOLDEN.y),
                Visibility::Hidden,
            ))
            .with_children(|ring| {
                for side in 0..2u8 {
                    let mesh = meshes.add(arc_mesh(1.0, side == 1));
                    ring.spawn((
                        Arc {
                            team: side,
                            mesh: mesh.clone(),
                        },
                        Mesh3d(mesh),
                        MeshMaterial3d(materials.add(range::look(colours.team(side).with_alpha(0.92)))),
                        // Red's over blue's, should both hold at once.
                        Transform::from_xyz(0.0, 0.004 * f32::from(1 - side), 0.0),
                        Visibility::Hidden,
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                }
            });
        return;
    }
    let hold = race.round.as_ref().filter(|round| race.begun && !round.over).map_or([0; 2], |round| round.hold);
    for (arc, mut shown) in &mut arcs {
        let side = usize::from(arc.team);
        let tenths = hold[side].min(10);
        if tenths == drawn[side] {
            continue;
        }
        drawn[side] = tenths;
        if tenths == 0 {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        }
        if meshes.insert(&arc.mesh, arc_mesh(f32::from(tenths) / 10.0, arc.team == 1)).is_err() {
            warn!("the golden island's ring has lost its mesh");
        }
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// Where a point of the world is on the screen, if it is in front of the
/// camera at all: in the window's logical pixels, on it or off it.
fn on_the_screen(cameras: &Query<(&Camera, &Transform), With<ThirdPersonCamera>>, at: Vec3) -> Option<Vec2> {
    let (camera, eye) = cameras.single().ok()?;
    camera.world_to_viewport(&GlobalTransform::from(*eye), at).ok()
}

/// "Hold! 2" over the golden island while a team holds it, in that team's
/// colour, the seconds it still has to hold it for; the team further on
/// with it, should both. Put away otherwise, and off the screen.
#[allow(clippy::type_complexity)]
pub(super) fn hold_words(
    racing: Res<Racing>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut words: Query<(&mut Text, &mut TextColor, &mut Visibility, &mut UiTransform, &ComputedNode), With<HoldWords>>,
) {
    let Ok((mut text, mut ink, mut shown, mut place, node)) = words.single_mut() else {
        return;
    };
    let holding = racing.0.as_ref().and_then(|race| race.round.as_ref().filter(|round| race.begun && !round.over));
    let held = holding.and_then(|round| (0..2u8).filter(|&side| round.hold[usize::from(side)] > 0).max_by_key(|&side| round.hold[usize::from(side)]).map(|side| (side, round.hold[usize::from(side)])));
    let point = held.and_then(|_| on_the_screen(&cameras, Vec3::new(GOLDEN.x, LIFT + HOLD_OVER, GOLDEN.y)));
    let (Some((side, tenths)), Some(point), Ok(window)) = (held, point, windows.single()) else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    if !on_screen(window, point) {
        shown.set_if_neq(Visibility::Hidden);
        return;
    }
    let file = ui();
    let seconds = (HOLD * f32::from(10 - tenths.min(10)) / 10.0).ceil().max(1.0);
    let said = fill(&file.words.bridge_race.hold, &[("seconds", &format!("{seconds:.0}"))]);
    if text.0 != said {
        text.0 = said;
    }
    let colour = file.colours.arena.team(side).with_alpha(1.0);
    if ink.0 != colour {
        ink.0 = colour;
    }
    let size = node.size() * node.inverse_scale_factor();
    place.set_if_neq(UiTransform {
        translation: Val2::px(point.x - size.x * 0.5, point.y - size.y * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
    shown.set_if_neq(Visibility::Inherited);
}

/// Where along the edge of a screen `size` across (logical pixels) the way
/// from its middle toward `toward`, a point off it, leaves it, `inset` in
/// from the edge: and which way that is, as a unit across the screen.
fn at_the_edge(size: Vec2, toward: Vec2, inset: f32) -> (Vec2, Vec2) {
    let middle = size * 0.5;
    let way = (toward - middle).normalize_or(Vec2::X);
    let reach = |along: f32, half: f32| if along.abs() < 1e-6 { f32::INFINITY } else { (half - inset).max(0.0) / along.abs() };
    let far = reach(way.x, middle.x).min(reach(way.y, middle.y));
    (middle + way * far, way)
}

/// While the race is on and the golden island's middle is off the screen,
/// the arrow at the edge of the screen pointing the way to it, with how
/// far it is from you to its shore just inside it ("63m"); put away
/// otherwise, and while you are dead.
#[allow(clippy::type_complexity)]
pub(super) fn point(
    racing: Res<Racing>,
    me: Query<(&Transform, &Venue, Has<Dead>), With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut needles: Query<(&mut Visibility, &mut UiTransform, &ComputedNode), (With<Needle>, Without<NeedleWords>)>,
    mut words: Query<(&mut Text, &mut Visibility, &mut UiTransform, &ComputedNode), (With<NeedleWords>, Without<Needle>)>,
) {
    let (Ok((mut needle_shown, mut needle_at, needle_node)), Ok((mut text, mut words_shown, mut words_at, words_node))) =
        (needles.single_mut(), words.single_mut())
    else {
        return;
    };
    let on = racing.0.as_ref().is_some_and(|race| race.on());
    let you = me.single().ok().filter(|(_, venue, dead)| on && **venue == Venue::BridgeRace && !dead);
    let gold = Vec3::new(GOLDEN.x, LIFT, GOLDEN.y);
    let wanted = you.zip(on_the_screen(&cameras, gold)).zip(windows.single().ok());
    let Some(((you, point), window)) = wanted.filter(|((_, point), window)| !on_screen(window, *point)) else {
        needle_shown.set_if_neq(Visibility::Hidden);
        words_shown.set_if_neq(Visibility::Hidden);
        return;
    };
    let vmin = window.width().min(window.height()) / 100.0;
    let (edge, way) = at_the_edge(Vec2::new(window.width(), window.height()), point, NEEDLE_IN * vmin);
    let size = needle_node.size() * needle_node.inverse_scale_factor();
    needle_at.set_if_neq(UiTransform {
        translation: Val2::px(edge.x - size.x * 0.5, edge.y - size.y * 0.5),
        scale: Vec2::ONE,
        // The arrow's picture points up the screen: turned to point that way.
        rotation: Rot2::radians(way.x.atan2(-way.y)),
    });
    needle_shown.set_if_neq(Visibility::Inherited);
    let away = (you.0.translation.xz().distance(GOLDEN) - ISLAND_RADIUS).max(0.0);
    let said = fill(&ui().words.bridge_race.away, &[("metres", &format!("{away:.0}"))]);
    if text.0 != said {
        text.0 = said;
    }
    // Inside the arrow, toward the middle of the screen, clear of it.
    let words_size = words_node.size() * words_node.inverse_scale_factor();
    let past = 0.5 * (way.x.abs() * words_size.x + way.y.abs() * words_size.y);
    let middle = edge - way * ((NEEDLE * 0.5 + NEEDLE_GAP) * vmin + past);
    words_at.set_if_neq(UiTransform {
        translation: Val2::px(middle.x - words_size.x * 0.5, middle.y - words_size.y * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
    words_shown.set_if_neq(Visibility::Inherited);
}

/// `words` rising off whoever stands with their feet `at`, in the planks'
/// colour: "+3m" as a grant of bridge comes.
pub(super) fn rising(commands: &mut Commands, at: Vec3, words: &str) {
    commands.spawn((
        InArena,
        Rising { at, age: 0.0 },
        hud::words(words, RISE_TEXT, ui().colours.arena.plank),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            ..default()
        },
        UiTransform::default(),
        Visibility::Hidden,
    ));
}

/// Floats whatever is rising up off where it began, fading as it goes, and
/// takes it away once it has risen.
#[allow(clippy::type_complexity)]
pub(super) fn rise(
    time: Res<Time>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut risings: Query<(Entity, &mut Rising, &mut Visibility, &mut UiTransform, &ComputedNode, &mut TextColor, &mut TextShadow)>,
    mut commands: Commands,
) {
    if risings.is_empty() {
        return;
    }
    let colours = ui().colours;
    let (ink, shadow) = (colours.arena.plank, colours.text_shadow);
    let window = windows.single().ok();
    for (thing, mut rising, mut shown, mut place, node, mut colour, mut under) in &mut risings {
        rising.age += time.delta_secs();
        let done = rising.age / RISE_SECS;
        if done >= 1.0 {
            commands.entity(thing).despawn();
            continue;
        }
        let at = rising.at + Vec3::Y * (RISE_FROM + RISE_BY * done);
        let point = on_the_screen(&cameras, at).filter(|point| window.is_some_and(|window| on_screen(window, *point)));
        let Some(point) = point else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let size = node.size() * node.inverse_scale_factor();
        place.set_if_neq(UiTransform {
            translation: Val2::px(point.x - size.x * 0.5, point.y - size.y * 0.5),
            scale: Vec2::ONE,
            rotation: Rot2::IDENTITY,
        });
        let solid = 1.0 - done * done;
        colour.0 = ink.with_alpha(ink.alpha() * solid);
        under.color = shadow.with_alpha(shadow.alpha() * solid);
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// A line across the upper half saying what has just happened, `words`,
/// with `picture` after it if it has one (the dynamite's, as a stick comes
/// to your button): up for a few seconds, one at a time, any line up
/// before it, this game's or Bridge Wars' own, taken down first.
pub(super) fn notice(commands: &mut Commands, say: &mut Say, words: &str, picture: Option<Picture>) {
    for old in &say.notices {
        commands.entity(old).try_despawn();
    }
    let image = picture.map(|picture| say.pictures.of(&mut say.images, picture));
    // Put up clear, to be brought up.
    let ink = ui().colours.arena.alert.with_alpha(0.0);
    commands
        .spawn((
            InArena,
            Notice { age: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Vh(NOTICE_DOWN),
                left: Val::ZERO,
                right: Val::ZERO,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::VMin(1.0),
                ..default()
            },
            GlobalZIndex(DIALOG_LAYER - 1),
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn(hud::words(words, NOTICE_TEXT, ink));
            if let Some(image) = image {
                row.spawn((
                    NoticePicture,
                    ImageNode::new(image).with_color(Color::WHITE.with_alpha(0.0)),
                    Node {
                        width: Val::VMin(NOTICE_PICTURE),
                        height: Val::VMin(NOTICE_PICTURE),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
        });
}

/// Brings each line up, holds it, and takes it away again, its picture
/// with its words: written only while it is fading.
pub(super) fn fade_notices(
    time: Res<Time>,
    mut notices: Query<(Entity, &mut Notice, &Children)>,
    mut lines: Query<(&mut TextColor, &mut TextShadow)>,
    mut pictures: Query<&mut ImageNode, With<NoticePicture>>,
    mut commands: Commands,
) {
    if notices.is_empty() {
        return;
    }
    let colours = ui().colours;
    let (ink, shadow) = (colours.arena.alert, colours.text_shadow);
    for (entity, mut notice, children) in &mut notices {
        notice.age += time.delta_secs();
        if notice.age >= NOTICE_SECS {
            commands.entity(entity).despawn();
            continue;
        }
        let solid = (notice.age / NOTICE_IN).min((NOTICE_SECS - notice.age) / NOTICE_OUT).clamp(0.0, 1.0);
        for child in children.iter() {
            if let Ok((mut colour, mut under)) = lines.get_mut(child) {
                let faded = ink.with_alpha(ink.alpha() * solid);
                if colour.0 != faded {
                    colour.0 = faded;
                    under.color = shadow.with_alpha(shadow.alpha() * solid);
                }
            } else if let Ok(mut image) = pictures.get_mut(child) {
                let faded = Color::WHITE.with_alpha(solid);
                if image.color != faded {
                    image.color = faded;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_s_arc_is_as_long_as_the_hold_and_never_empty() {
        // A tenth of the way round, the way the clock goes from the top:
        // its pieces' corners all between the two edges, starting at the
        // top, and ending a tenth round toward the right of the screen.
        let mesh = arc_mesh(0.1, false);
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(corners)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("no corners");
        };
        assert!(!corners.is_empty());
        for corner in corners {
            let out = Vec2::new(corner[0], corner[2]).length();
            assert!((RING_IN - 1e-4..=RING_OUT + 1e-4).contains(&out), "{out}");
        }
        assert!(corners[0][0].abs() < 1e-5 && corners[0][2] < 0.0, "begins at the top: {:?}", corners[0]);
        let last = corners[corners.len() - 1];
        assert!(last[0] > 0.0, "red's goes the way the clock does: {last:?}");
        let blue = arc_mesh(0.1, true);
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(corners)) = blue.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("no corners");
        };
        assert!(corners[corners.len() - 1][0] < 0.0, "blue's the other way");
        // Nothing of a hold is still something drawn, never an empty mesh.
        assert!(arc_mesh(0.0, false).count_vertices() >= 4);
        // Over the map's own band, wider than it either way.
        const { assert!(RING_IN < 5.5 && RING_OUT > 6.0) };
    }

    #[test]
    fn the_arrow_stands_in_from_the_edge_the_way_the_golden_island_is() {
        let size = Vec2::new(1280.0, 720.0);
        // Off to the right, a little down: on the right edge, in by the
        // inset, a little under the middle.
        let (at, way) = at_the_edge(size, Vec2::new(3000.0, 500.0), 40.0);
        assert!((at.x - 1240.0).abs() < 1e-3 && at.y > 360.0 && at.y < 400.0, "{at}");
        assert!(way.x > 0.9);
        // Straight up: the top edge.
        let (at, _) = at_the_edge(size, Vec2::new(640.0, -900.0), 40.0);
        assert!((at - Vec2::new(640.0, 40.0)).length() < 1e-3, "{at}");
        // Off a corner: inside the screen whichever edge it meets first.
        let (at, _) = at_the_edge(size, Vec2::new(-5000.0, 6000.0), 40.0);
        assert!(at.x >= 40.0 - 1e-3 && at.y <= 680.0 + 1e-3, "{at}");
    }

    #[test]
    fn the_plate_says_how_far_a_bridge_has_to_go_or_that_it_is_across_or_who_stands_there() {
        // Nobody across and nothing laid: the whole way.
        let whole = the_way_across(0);
        assert_eq!(progress_words(0, 0, 0, 4), fill(&ui().words.bridge_race.to_go, &[("metres", &format!("{:.0}", whole.ceil()))]));
        assert_eq!(progress_words(1, 100, 0, 4), ui().words.bridge_race.across);
        assert_eq!(progress_words(1, 100, 1, 4), fill(&ui().words.bridge_race.on_island, &[("count", "1"), ("of", "2")]));
        assert_eq!(progress_words(0, 40, 1, 2), fill(&ui().words.bridge_race.on_island, &[("count", "1"), ("of", "1")]));
    }
}
