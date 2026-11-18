//! What Bridge Race draws over the world and on the screen (`super`), all
//! of it judged from the camera fixed over you, as Bridge Wars' is:
//!
//! * **The plate top centre** ([`plate`]): the time left, "4:12", and under
//!   it each team's way across, red's and blue's side by side, a bar in
//!   its colour as long as its bridge has got toward the golden island
//!   (its plank nearest it, as a share of the 63 m of void,
//!   `RaceRound::across`), never shorter than a sliver of its colour, with
//!   how far it still has to go under it, or "Across!", or how many of the
//!   team stand on the golden island. On the plate's grey, as the board the
//!   table has up before the race is (Bridge Wars' own look); up from "Go!"
//!   to the end.
//! * **The ring round the golden island** ([`ring`]): over the map's own
//!   ring, the band 6 m out from its middle, an arc in the colour of a team
//!   that has every one of its players on the island, filling as its hold
//!   goes on, red's the way the clock goes from the top and blue's the
//!   other, over a dark band a little wider than it, rebuilt only as it
//!   comes to the next tenth of [`HOLD`]; unlit, so that it reads by night
//!   as by day. And over the island, in each holding team's colour, "Hold!
//!   2", the seconds that team still has to hold it, each team's in a place
//!   of its own should both hold ([`hold_words`]); and "Hold broken!" in a
//!   team's colour as its hold is broken ([`rising`]).
//! * **The way to the golden island** ([`point`]): while none of it is on
//!   the screen, an arrow at the edge of the screen pointing to it, in its
//!   gold, with how far it is from you to its shore ("63m") by it, inward
//!   (the compass's idea, `compass`), kept inside the safe area and clear
//!   of the buttons bottom right and the stick bottom left.
//! * **"Jump!"** over the gap between you and the golden island, once you
//!   stand near enough to its shore to jump it and have too little bridge
//!   to lay across it ([`jump_hint`]).
//! * **The next grant**: a ring round the rim of the plank button bottom
//!   right, filling the way the clock goes toward the next grant of bridge
//!   ([`next_grant`], [`count_to_grant`]); and as it comes, "+3m" rising
//!   off you, up the screen ([`rising`], [`rise`]), as a number rises off
//!   a body hit in Bridge Wars: white, as the metres left are written in
//!   front of you while you build (mine: the planks' own colour, as first
//!   written, was lost on the planks you stand on as you build, seen in a
//!   scripted run).
//! * **What has happened**, a line across the upper half for a few
//!   seconds, Bridge Wars' own look (`arena::alert`), with a picture after
//!   it where it is about one: "Dynamite!" as a stick comes to your
//!   button, a team's bridge come across, a team holding the golden island
//!   ([`notice`], [`fade_notices`]).
//! * **The start and the end** across the middle of the screen
//!   ([`banner`], [`fade_banners`]): where and as big as `hud::announce`
//!   puts Bridge Wars' own, a team's name in its team's colour.
//!
//! **Words drawn over the world have a thin dark stroke round them**
//! ([`stroked`]): "Hold!", the arrow's metres, "Jump!", what rises off you
//! and the start and the end, as your name top left has it in its team's
//! colour (`hud::name_plate`; since 2026-11-18, mine, from a play-test that
//! day: blue's "Hold! 2" over the golden island's lilac measured 1.04 to
//! 1, red's 1.19, and gold on the void's sky 1.17, each read only by its
//! shadow; Hajun's way, his colours kept, is to stroke them).
//!
//! Everything here is the table's (`table::InTable`) or the fight's
//! (`arena::InArena`), and goes with it; and everything that runs every
//! frame writes only what has changed. The teams' colours are the file's
//! (`colours.arena.team_red`, `team_blue`), drawn in the bars, the ring and
//! the words as solid as it says (`colours.bridge_race`'s strengths).

use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use roundtown_net::bridge_race::{GOLDEN, GRANT_EVERY, HOLD, ISLAND_RADIUS, LIFT, the_way_across};

use super::{Racing, Say};
use crate::arena::{InArena, Resources, bar_width, clock, on_screen};
use crate::builder::DIALOG_LAYER;
use crate::hud::{self, Picture};
use crate::island::Venue;
use crate::range;
use crate::safe::SafeArea;
use crate::table::{InTable, ResultsWindow};
use crate::ui::{BridgeRaceColours, UiChanged, fill, ui};
use crate::{Dead, Player, ThirdPersonCamera};

/// Sizes on the plate, in shares of the short side of the screen: the
/// clock, as big as the board's words before the race; each team's bar,
/// long and thin, and the words under it (2.3 until 2026-11-18, the same
/// day: 9 points on a phone, too small to read at a glance, seen in a
/// play-test); and the gaps between.
const CLOCK_TEXT: f32 = 4.8;
const BAR_LONG: f32 = 13.0;
const BAR_THICK: f32 = 1.3;
const PROGRESS_TEXT: f32 = 2.8;
const SIDES_APART: f32 = 3.2;
/// The least of a team's bar there is in its colour, as a share of the
/// bar: with nothing laid, the two bars were the same grey, and which was
/// whose could not be told (seen in a play-test; mine).
const SLIVER: f32 = 0.06;

/// The ring round the golden island, in metres: from and to how far out
/// from the island's middle it is drawn, a little wider than the map's own
/// band (5.4 to 6 m out as seen from over it) either way, so that it covers
/// it; how much wider still, either side, the dark band under it is (mine:
/// a team's colour on the island's lilac had no edge to it); how high its
/// top lies over the island's, just over that band's (0.2 m up), clear of
/// it; and how finely its arc is drawn, in pieces of the whole way round.
const RING_IN: f32 = 5.25;
const RING_OUT: f32 = 6.25;
const RING_EDGE: f32 = 0.2;
const RING_OVER: f32 = 0.23;
const RING_PIECES: usize = 96;

/// "Hold! 2": how big, a little over a headline's caption; where it is
/// written, in metres from the golden island's middle: a little up the
/// screen from it (the map's -Z), inside the ring, clear of whoever stands
/// at the middle, where it sat on your own head at first, and of the line
/// across the upper half, which it met 3.4 m up (both seen in scripted
/// runs); and how far either side of that each team's is, should both
/// hold at once (mine: one "Hold!" went from one team's colour to the
/// other's as either got further, seen in a play-test).
const HOLD_TEXT: f32 = 5.6;
const HOLD_AT: Vec3 = Vec3::new(0.0, 1.0, -2.2);
const HOLD_APART: f32 = 2.6;

/// "Hold broken!": how big; where it begins, in metres from the golden
/// island's middle, a little down the screen from it, clear of "Hold!" up
/// the screen (begun where "Hold!" stands, red's was written over blue's
/// "Hold! 3" as blue held on, seen in a scripted run); how far it rises up
/// the screen and for how long, in seconds (mine).
const BROKEN_TEXT: f32 = 4.2;
const BROKEN_AT: Vec3 = Vec3::new(0.0, 1.0, 1.4);
const BROKEN_RISE: f32 = 0.8;
const BROKEN_SECS: f32 = 1.8;

/// The arrow toward the golden island: how big, as the compass's is a
/// share of the short side; how far in from the edge of the screen its
/// middle is; how far down the screen its middle is at the least, under
/// the plate and the minimap; how far clear of the buttons bottom right and
/// the stick bottom left it keeps (since 2026-11-18, the same day: on a
/// phone it stood right of the fist, under the right thumb, seen in a
/// play-test); how big the words by it are, and how far inside the arrow
/// they begin.
const NEEDLE: f32 = 5.0;
const NEEDLE_IN: f32 = 6.0;
const NEEDLE_TOP: f32 = 32.0;
const NEEDLE_CLEAR: f32 = 1.5;
const NEEDLE_TEXT: f32 = 3.0;
const NEEDLE_GAP: f32 = 1.2;

/// "Jump!": how near the golden island's shore you have to stand to be
/// told it, in metres; and how near is on it already. A body runs 10 m a
/// second in the race and is in the air for 0.77 s of a jump (the game's
/// jump and fall), so a running jump carries it 7.7 m: from where you
/// stand, the end of your bridge a metre ahead of you as you run out, that
/// clears the shore by more than a metre, and the most a grant of one
/// against one leaves you short is 6 m (mine; the computer's players
/// jumped a last gap of 2 m, and nothing told a person they could, seen in
/// a play-test; at 5.5 m a scripted run waited out a gap of 5.7 that it
/// could have jumped). How big its words are, and where over the gap they
/// stand: a metre up, and up the screen off the line of the bridge, clear
/// of the metres written in front of you at its end (`bridge`'s "0m",
/// which they were written over, seen in a scripted run).
const JUMP_FROM: f32 = 6.5;
const JUMP_NEAREST: f32 = 0.6;
const JUMP_TEXT: f32 = 4.4;
const JUMP_OVER: Vec3 = Vec3::new(0.0, 1.0, -1.8);

/// "+3m": how big, as a number rising off a body hit is; where it begins,
/// in metres from your feet, and how far it goes, up the screen (the map's
/// -Z: from the camera over you a rise straight up is no rise at all); and
/// for how long, in seconds: a little longer than a number does, there
/// being more to read. Until 2026-11-18, the same day, it began 1.2 m up
/// the screen and rose 2.2 m, into the line across the upper half that the
/// same grant's bridge across had put up (seen in a play-test).
const RISE_TEXT: f32 = 3.8;
const RISE_FROM: Vec3 = Vec3::new(0.0, 1.8, -0.4);
const RISE_BY: f32 = 1.4;
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

/// How long words across the middle of the screen take to come up and to
/// go, in seconds: `hud::announce`'s own.
const BANNER_IN: f32 = 0.4;
const BANNER_OUT: f32 = 0.8;

/// How far each of the four copies of stroked words that are their stroke
/// is set from the words, across and down, as a share of their size: as
/// thin as the stroke round your name top left is for its size (`hud`'s,
/// 0.036 of the name's).
const STROKE: f32 = 0.036;

/// The ring round the plank button that fills toward the next grant
/// ([`next_grant`]): in how many steps it fills, drawn again only as it
/// comes to the next (twice a second, a grant every 16 s); how many pixels
/// across its picture is; and from and to how far out from the button's
/// middle its band is, as shares of the button's half, on the button's own
/// rim (mine: out past the rim it ran into the button for things to use).
const NEXT_STEPS: u32 = 32;
const NEXT_PIXELS: u32 = 128;
const NEXT_IN: f32 = 0.86;
const NEXT_OUT: f32 = 0.98;

/// The plate top centre, its clock, and each team's bar and words.
#[derive(Component)]
pub(super) struct Plate;
#[derive(Component)]
pub(super) struct PlateClock;
#[derive(Component)]
pub(super) struct Progress(u8);
#[derive(Component)]
pub(super) struct ProgressWords(u8);

/// Whether the plate has each team's bar and words under the clock (how
/// far its bridge has to go, "62m to go", or how many of it stand on the
/// golden island): not since the evening of 2026-11-18 (Hajun: "remove the
/// unnecessary stats below the timer (red: 62m to go, blue: 55m to go) and
/// replace that with a Next Item: 14m (bridge icon) 1 (dynamite icon)"),
/// [`NextItem`] in their place; true puts them back, and takes it away.
const SIDES_SHOWN: bool = false;

/// Under the clock, what your next grant brings: its words ("Next Item:
/// 14m"), the plank's picture after them, and, while a stick of dynamite
/// comes with it, "1" and the stick's picture ([`NextStick`]).
#[derive(Component)]
pub(super) struct NextItem;
#[derive(Component)]
pub(super) struct NextStick;

/// How big the pictures in the next item's line are, as a share of the
/// screen's short side, and how far each reaches over and under the line
/// without making it taller (mine): the plank's is three planks side by
/// side, a third as tall as it is wide, which at the words' own height was
/// a row of dots (seen enlarged in a hidden run).
const NEXT_PICTURE: f32 = 5.6;
const NEXT_PICTURE_SPILL: f32 = 1.2;

/// The ring round the golden island, each team's arc of it, and the dark
/// band under them.
#[derive(Component)]
pub(super) struct Ring;
#[derive(Component)]
pub(super) struct Arc {
    team: u8,
    mesh: Handle<Mesh>,
}
#[derive(Component)]
pub(super) struct RingEdge;

/// "Hold! 2", over the golden island, the team's whose it is.
#[derive(Component)]
pub(super) struct HoldWords(u8);

/// The arrow toward the golden island, and how far it is, by it.
#[derive(Component)]
pub(super) struct Needle;
#[derive(Component)]
pub(super) struct NeedleWords;

/// "Jump!", over the gap between you and the golden island.
#[derive(Component)]
pub(super) struct JumpWords;

/// Words rising off somewhere in the world: where they begin, how far they
/// rise and for how long, how long they have been up, and every piece of
/// them, to fade.
#[derive(Component)]
pub(super) struct Rising {
    from: Vec3,
    by: f32,
    secs: f32,
    age: f32,
    inks: Vec<Entity>,
}

/// A line across the upper half, and how long it has been up; and the
/// picture in it.
#[derive(Component)]
pub(crate) struct Notice {
    age: f32,
}
#[derive(Component)]
pub(super) struct NoticePicture;

/// Words across the middle of the screen ([`banner`]): how long they have
/// been up, for how long they are to be, whether they go as the table's
/// results come up (the end's: they showed through the results' window,
/// seen in a play-test), and every piece of them, to fade.
#[derive(Component)]
pub(super) struct Banner {
    age: f32,
    secs: f32,
    till_results: bool,
    inks: Vec<Entity>,
}

/// One piece of stroked words ([`stroked`]): its own colour at full
/// strength, which it is faded from.
#[derive(Component, Clone, Copy)]
pub(super) struct Ink(Color);

/// The ring round the plank button that fills toward the next grant: its
/// picture, and the step it was last drawn at.
#[derive(Component)]
pub(super) struct NextGrant {
    image: Handle<Image>,
    drawn: Option<u32>,
}

/// Spawns `parts` (words, each in its colour) as one line `size` big with a
/// thin dark stroke round it, under `parent` if it has one: four copies of
/// the whole line in the stroke's colour (`colours.chat.stroke`, as your
/// name top left has it in its team's colour, `hud::name_plate`), each a
/// hair off toward a corner, under the words; the words over them, a piece
/// to each part, each in its colour. Put up as solid as `strength` says
/// (0, clear, for whatever brings it up). Every piece carries its own
/// colour ([`Ink`]) and is pushed onto `inks`. The line, for whoever puts
/// it somewhere.
fn stroked(commands: &mut Commands, parent: Option<Entity>, parts: &[(String, Color)], size: f32, strength: f32, inks: &mut Vec<Entity>) -> Entity {
    let stroke = ui().colours.chat.stroke;
    let faded = |colour: Color| colour.with_alpha(colour.alpha() * strength);
    let whole: String = parts.iter().map(|(words, _)| words.as_str()).collect();
    let mut line = commands.spawn((Node::default(), Pickable::IGNORE));
    if let Some(parent) = parent {
        line.insert(ChildOf(parent));
    }
    let line = line.id();
    let off = size * STROKE;
    for (across, down) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let rim = commands
            .spawn((
                ChildOf(line),
                Ink(stroke),
                Text::new(whole.clone()),
                crate::font::sized(size),
                TextColor(faded(stroke)),
                TextLayout::justify(Justify::Center),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::ZERO,
                    top: Val::ZERO,
                    ..default()
                },
                UiTransform::from_translation(Val2::new(Val::VMin(across * off), Val::VMin(down * off))),
                Pickable::IGNORE,
            ))
            .id();
        inks.push(rim);
    }
    let (first, colour) = parts.first().cloned().unwrap_or((String::new(), Color::WHITE));
    let text = commands.spawn((ChildOf(line), Ink(colour), hud::words(first, size, faded(colour)))).id();
    inks.push(text);
    for (words, colour) in parts.iter().skip(1) {
        let span = commands
            .spawn((
                ChildOf(text),
                Ink(*colour),
                TextSpan::new(words.clone()),
                crate::font::sized(size),
                TextColor(faded(*colour)),
            ))
            .id();
        inks.push(span);
    }
    line
}

/// Writes `said` into a line of stroked words of one part ([`stroked`]):
/// the words and the four copies that are their stroke, each only if it
/// changes.
fn rewrite(line: Entity, said: &str, children: &Query<&Children>, texts: &mut Query<&mut Text, With<Ink>>) {
    for child in children.get(line).into_iter().flat_map(|kids| kids.iter()) {
        if let Ok(mut text) = texts.get_mut(child)
            && text.0 != said
        {
            text.0 = said.to_string();
        }
    }
}

/// Draws every piece of stroked words in `pieces` as solid as `solid`
/// says of its own colour ([`Ink`]), its shadow with it: only those it
/// changes.
fn fade(pieces: &[Entity], solid: f32, inks: &mut Query<(&Ink, &mut TextColor, Option<&mut TextShadow>)>) {
    let tint = ui().colours.text_shadow;
    for &piece in pieces {
        if let Ok((ink, mut colour, shadow)) = inks.get_mut(piece) {
            let faded = ink.0.with_alpha(ink.0.alpha() * solid);
            if colour.0 != faded {
                colour.0 = faded;
                if let Some(mut shadow) = shadow {
                    shadow.color = tint.with_alpha(tint.alpha() * faded.alpha());
                }
            }
        }
    }
}

/// A team's colour as the race writes it in words over the world: as
/// solid as the file says "Hold!" is (`colours.bridge_race.hold_strength`).
pub(super) fn team_ink(side: u8) -> Color {
    let colours = ui().colours;
    BridgeRaceColours::solid(colours.arena.team(side), colours.bridge_race.hold_strength)
}

/// Words placed over the world each frame (`UiTransform`), put away until
/// they are wanted: absolute, at the screen's corner until placed, under
/// the buttons.
fn placed() -> (Node, UiTransform, Visibility, GlobalZIndex) {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            ..default()
        },
        UiTransform::default(),
        Visibility::Hidden,
        GlobalZIndex(-1),
    )
}

/// Puts up what the race draws on the screen, each piece put away until it
/// is wanted: the plate, each team's "Hold!", the arrow and its words, and
/// "Jump!". The ring is put up by [`ring`], which has the meshes to make it
/// with; the minimap and the ring round the plank button go with the
/// buttons (`super::spawn_bar`).
pub(super) fn spawn(commands: &mut Commands, images: &mut Assets<Image>, pictures: &mut hud::Pictures) {
    let file = ui();
    let colours = file.colours;
    let accent = colours.bridge_race.accent;
    let words = &file.words.bridge_race;
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
                    if !SIDES_SHOWN {
                        let picture = |image: Handle<Image>| {
                            (
                                ImageNode::new(image),
                                Node {
                                    width: Val::VMin(NEXT_PICTURE),
                                    height: Val::VMin(NEXT_PICTURE),
                                    margin: UiRect::vertical(Val::VMin(-NEXT_PICTURE_SPILL)),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            )
                        };
                        let plank = pictures.of(images, Picture::Plank);
                        let stick = pictures.of(images, Picture::Dynamite);
                        inside
                            .spawn((
                                Node {
                                    align_items: AlignItems::Center,
                                    column_gap: Val::VMin(0.8),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|line| {
                                line.spawn((NextItem, hud::words("", PROGRESS_TEXT, colours.text)));
                                line.spawn(picture(plank));
                                line.spawn((
                                    NextStick,
                                    Node {
                                        align_items: AlignItems::Center,
                                        column_gap: Val::VMin(0.5),
                                        margin: UiRect::left(Val::VMin(0.8)),
                                        ..default()
                                    },
                                    Visibility::Hidden,
                                    Pickable::IGNORE,
                                ))
                                .with_children(|stick_line| {
                                    stick_line.spawn(hud::words("1", PROGRESS_TEXT, colours.text));
                                    stick_line.spawn(picture(stick));
                                });
                            });
                        return;
                    }
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
                                let ink = BridgeRaceColours::solid(colours.arena.team(side), colours.bridge_race.bar_strength);
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
                                                // Rounded at its end as the bar
                                                // it fills is (square, until a
                                                // play-test of 2026-11-18).
                                                bar.spawn((
                                                    Progress(side),
                                                    Node {
                                                        width: bar_width(SLIVER),
                                                        height: Val::Percent(100.0),
                                                        border_radius: BorderRadius::all(Val::VMin(BAR_THICK * 0.5)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(ink),
                                                    Pickable::IGNORE,
                                                ));
                                            });
                                        // In white, over the team's bar: a team's colour on
                                        // the plate's see-through grey went all but unread
                                        // over the islands' grey by day (seen in a scripted
                                        // run), and the bar says whose it is.
                                        column.spawn((ProgressWords(side), hud::words("", PROGRESS_TEXT, colours.text)));
                                    });
                            }
                        });
                });
        });
    let mut inks = Vec::new();
    for side in 0..2u8 {
        let said = fill(&words.hold, &[("seconds", "3")]);
        let line = stroked(commands, None, &[(said, team_ink(side))], HOLD_TEXT, 1.0, &mut inks);
        commands.entity(line).insert((InTable, HoldWords(side), Name::new("Hold!"), placed()));
    }
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
    let line = stroked(commands, None, &[(String::new(), accent)], NEEDLE_TEXT, 1.0, &mut inks);
    commands.entity(line).insert((InTable, NeedleWords, placed()));
    let line = stroked(commands, None, &[(words.jump.clone(), accent)], JUMP_TEXT, 1.0, &mut inks);
    commands.entity(line).insert((InTable, JumpWords, Name::new("Jump!"), placed()));
}

/// What the plate says of `side`'s way across, as the round has it: how
/// many of the team stand on the golden island, if any do, of its
/// `players` still in the game (of its seats, until 2026-11-18, the same
/// day, so that "1/2" stayed up under a ring filling for the one left of
/// two); across, if its bridge is; or how many metres of void it still
/// has to bridge.
fn progress_words(side: usize, across: u8, on_gold: u8, players: u8) -> String {
    let file = ui();
    let words = &file.words.bridge_race;
    if on_gold > 0 {
        let of = players.max(on_gold).max(1).to_string();
        fill(&words.on_island, &[("count", &on_gold.to_string()), ("of", &of)])
    } else if across >= 100 {
        words.across.clone()
    } else {
        let left = the_way_across(side as u8) * (1.0 - f32::from(across) / 100.0);
        fill(&words.to_go, &[("metres", &format!("{:.0}", left.max(1.0).ceil()))])
    }
}

/// How long a team's bar is, as a share of the whole, its bridge having
/// got `across` hundredths of the way: never less than a sliver of its
/// colour ([`SLIVER`]), so that whose bar is whose shows from the start.
fn bar_share(across: u8) -> f32 {
    (f32::from(across) / 100.0).clamp(SLIVER, 1.0)
}

/// Keeps the plate up from "Go!" to the end, with the time left and each
/// team's way across as the round has them, each written only as it
/// changes; and puts up afresh everything this module drew, in the file's
/// colours, as the file changes on desktop.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn plate(
    racing: Res<Racing>,
    changed: Res<UiChanged>,
    mut plates: Query<&mut Visibility, With<Plate>>,
    mut clocks: Query<&mut Text, (With<PlateClock>, Without<ProgressWords>)>,
    mut bars: Query<(&Progress, &mut Node)>,
    mut said: Query<(&ProgressWords, &mut Text), (Without<PlateClock>, Without<NextItem>)>,
    mut next: Query<&mut Text, (With<NextItem>, Without<PlateClock>, Without<ProgressWords>)>,
    mut stick_next: Query<&mut Visibility, (With<NextStick>, Without<Plate>)>,
    drawn: Query<Entity, Or<(With<Plate>, With<HoldWords>, With<Needle>, With<NeedleWords>, With<JumpWords>, With<Ring>)>>,
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
        let width = bar_width(bar_share(round.across[usize::from(side.0)]));
        if bar.width != width {
            bar.width = width;
        }
    }
    for (side, mut text) in &mut said {
        let side = usize::from(side.0);
        let words = progress_words(side, round.across[side], round.on_gold[side], round.players[side]);
        if text.0 != words {
            text.0 = words;
        }
    }
    // What your next grant brings: your share of your team's next metres,
    // and a stick of dynamite if one comes with it and there is room for it
    // on your button.
    let words = next_words(round.next_of(race.team()));
    for mut text in &mut next {
        if text.0 != words {
            text.0.clone_from(&words);
        }
    }
    let stick = round.stick_next(race.seat) && race.sticks < roundtown_net::bridge_race::DYNAMITE_MOST;
    for mut shown in &mut stick_next {
        shown.set_if_neq(if stick { Visibility::Inherited } else { Visibility::Hidden });
    }
}

/// What the plate says of your next grant of `metres` ("Next Item: 14m"):
/// in whole metres, or to a half where a team of two shares an odd number.
fn next_words(metres: f32) -> String {
    let file = ui();
    let shown = if (metres - metres.round()).abs() < 0.05 {
        format!("{metres:.0}")
    } else {
        format!("{metres:.1}")
    };
    fill(&file.words.bridge_race.next_item, &[("metres", &shown)])
}

/// The flat band of a ring between `inner` and `outer` round its middle,
/// `share` of the way round from the top of the screen (the map's -Z), the
/// way the clock goes, or `backward` the other way: in pieces of
/// [`RING_PIECES`] to the whole, at least one, facing up.
fn band_mesh(inner: f32, outer: f32, share: f32, backward: bool) -> Mesh {
    let pieces = ((share.clamp(0.0, 1.0) * RING_PIECES as f32).round() as usize).max(1);
    let sweep = share.clamp(0.0, 1.0) * std::f32::consts::TAU;
    let way = if backward { -1.0 } else { 1.0 };
    let mut positions = Vec::with_capacity((pieces + 1) * 2);
    for k in 0..=pieces {
        let angle = way * sweep * k as f32 / pieces as f32;
        let (sin, cos) = angle.sin_cos();
        for radius in [inner, outer] {
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

/// A team's arc of the ring, between [`RING_IN`] and [`RING_OUT`],
/// `share` of the way round ([`band_mesh`]).
fn arc_mesh(share: f32, backward: bool) -> Mesh {
    band_mesh(RING_IN, RING_OUT, share, backward)
}

/// The ring round the golden island: put up as the race is, on its venue
/// (shown with it, `island::show_where_i_am`), each team's arc in its own
/// mesh, made full and put away, over a dark band a little wider than they
/// are, the whole way round ([`RING_EDGE`], `colours.bridge_race.ring_edge`),
/// shown while either is; and each arc as long as its team's hold has gone
/// ([`roundtown_net::bridge_race::RaceRound::hold`], in tenths of
/// [`HOLD`]), made again only as that comes to another tenth, and put away
/// with none; full, once a team has won by holding, to the end. A mesh is
/// never made with nothing in it: an arc is put away rather than made
/// empty.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(super) fn ring(
    racing: Res<Racing>,
    rings: Query<(), With<Ring>>,
    mut arcs: Query<(&Arc, &mut Visibility), Without<RingEdge>>,
    mut edges: Query<&mut Visibility, (With<RingEdge>, Without<Arc>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drawn: Local<[u8; 2]>,
    mut commands: Commands,
) {
    let Some(race) = racing.0.as_ref() else {
        return;
    };
    if rings.is_empty() {
        let colours = ui().colours;
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
                ring.spawn((
                    RingEdge,
                    Mesh3d(meshes.add(band_mesh(RING_IN - RING_EDGE, RING_OUT + RING_EDGE, 1.0, false))),
                    MeshMaterial3d(materials.add(range::look(colours.bridge_race.ring_edge))),
                    // Under both arcs.
                    Transform::from_xyz(0.0, -0.004, 0.0),
                    Visibility::Hidden,
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                for side in 0..2u8 {
                    let mesh = meshes.add(arc_mesh(1.0, side == 1));
                    ring.spawn((
                        Arc {
                            team: side,
                            mesh: mesh.clone(),
                        },
                        Mesh3d(mesh),
                        MeshMaterial3d(materials.add(range::look(BridgeRaceColours::solid(colours.arena.team(side), colours.bridge_race.ring_strength)))),
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
    // Over, each arc stays as its hold ended: full round the island in the
    // colour of a team that won by holding it.
    let hold = race.round.as_ref().filter(|_| race.begun).map_or([0; 2], |round| round.hold);
    for mut shown in &mut edges {
        shown.set_if_neq(if hold.iter().any(|&tenths| tenths > 0) { Visibility::Inherited } else { Visibility::Hidden });
    }
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

/// Puts a line of words over the world (a node placed by its
/// `UiTransform`) with its middle on `point`, on the screen: written only
/// if it moves.
fn put(place: &mut Mut<UiTransform>, node: &ComputedNode, point: Vec2) {
    let size = node.size() * node.inverse_scale_factor();
    place.set_if_neq(UiTransform {
        translation: Val2::px(point.x - size.x * 0.5, point.y - size.y * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
}

/// "Hold! 2" over the golden island for each team that holds it, in its
/// colour, the seconds it still has to hold it for, each team's in a place
/// of its own should both hold at once (one "Hold!" for the team further
/// on with it, until 2026-11-18, the same day, which went from one colour
/// to the other in a scramble, seen in a play-test). Put away otherwise,
/// and off the screen.
#[allow(clippy::type_complexity)]
pub(super) fn hold_words(
    racing: Res<Racing>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    children: Query<&Children>,
    mut lines: Query<(Entity, &HoldWords, &mut Visibility, &mut UiTransform, &ComputedNode)>,
    mut texts: Query<&mut Text, With<Ink>>,
) {
    let round = racing.0.as_ref().and_then(|race| race.round.as_ref().filter(|round| race.begun && !round.over()));
    let hold = round.map_or([0; 2], |round| round.hold);
    let both = hold.iter().all(|&tenths| tenths > 0);
    let window = windows.single().ok();
    for (line, side, mut shown, mut place, node) in &mut lines {
        let tenths = hold[usize::from(side.0 % 2)];
        let aside = if both { Vec3::X * if side.0 == 0 { -HOLD_APART } else { HOLD_APART } } else { Vec3::ZERO };
        let point = (tenths > 0)
            .then(|| on_the_screen(&cameras, Vec3::new(GOLDEN.x, LIFT, GOLDEN.y) + HOLD_AT + aside))
            .flatten()
            .filter(|point| window.is_some_and(|window| on_screen(window, *point)));
        let Some(point) = point else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let seconds = (HOLD * f32::from(10 - tenths.min(10)) / 10.0).ceil().max(1.0);
        let said = fill(&ui().words.bridge_race.hold, &[("seconds", &format!("{seconds:.0}"))]);
        rewrite(line, &said, &children, &mut texts);
        put(&mut place, node, point);
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// Something on the screen the arrow toward the golden island keeps clear
/// of ([`at_the_edge`]): a box, its least and most corners, in logical
/// pixels; and where the arrow stands instead, should the way to the
/// golden island leave the screen inside it: just over the buttons round
/// the jump button, on the right edge, out from under the right thumb;
/// or, with none said, up or down the edge it is on, or along it, to
/// whichever end of the box is nearer and still on the screen.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Clear {
    least: Vec2,
    most: Vec2,
    instead: Option<Vec2>,
}

impl Clear {
    fn holds(&self, at: Vec2) -> bool {
        at.x > self.least.x && at.x < self.most.x && at.y > self.least.y && at.y < self.most.y
    }
}

/// Where the arrow toward a point off the screen, `toward`, stands on a
/// screen `size` across (logical pixels): where the way from the middle of
/// the screen to it leaves the part the arrow may be in (`within`, its
/// least and most corners: in from the edges, the safe area's too, and
/// under the plate), and out of each of `kept_clear` as that says.
fn at_the_edge(size: Vec2, toward: Vec2, within: (Vec2, Vec2), kept_clear: &[Clear]) -> Vec2 {
    let (low, high) = within;
    let middle = (size * 0.5).clamp(low, high);
    let way = (toward - middle).normalize_or(Vec2::X);
    let reach = |along: f32, from: f32, least: f32, most: f32| {
        if along > 1e-6 {
            (most - from) / along
        } else if along < -1e-6 {
            (least - from) / along
        } else {
            f32::INFINITY
        }
    };
    let far = reach(way.x, middle.x, low.x, high.x).min(reach(way.y, middle.y, low.y, high.y)).max(0.0);
    let mut at = middle + way * far;
    for clear in kept_clear {
        if !clear.holds(at) {
            continue;
        }
        if let Some(instead) = clear.instead {
            at = instead.clamp(low, high);
            continue;
        }
        // Up or down a side's edge; along the top's or the bottom's.
        let on_side = (at.x - low.x).abs() < 0.5 || (at.x - high.x).abs() < 0.5;
        let (now, ends, (lo, hi)) = if on_side {
            (at.y, [clear.least.y, clear.most.y], (low.y, high.y))
        } else {
            (at.x, [clear.least.x, clear.most.x], (low.x, high.x))
        };
        let end = ends
            .into_iter()
            .filter(|end| (lo..=hi).contains(end))
            .min_by(|a, b| (a - now).abs().total_cmp(&(b - now).abs()));
        if let Some(end) = end {
            if on_side {
                at.y = end;
            } else {
                at.x = end;
            }
        }
    }
    at
}

/// The part of a screen `size` across (logical pixels) the arrow may stand
/// in, and what it keeps clear of ([`at_the_edge`]): [`NEEDLE_IN`] in from
/// each edge and the safe area's sides (an iPhone's island, a camera's
/// hole), under the plate and the minimap ([`NEEDLE_TOP`]); clear of the
/// buttons round the jump button bottom right, as far up as the chat's
/// small one and as far left as the jump button, standing over them on the
/// right edge instead, and of the stick bottom left, by [`NEEDLE_CLEAR`]
/// and half the arrow.
fn needle_room(size: Vec2, safe: SafeArea) -> ((Vec2, Vec2), [Clear; 2]) {
    let vmin = size.min_element() / 100.0;
    let (vw, vh) = (size.x / 100.0, size.y / 100.0);
    let within = (
        Vec2::new(safe.left + NEEDLE_IN * vmin, NEEDLE_TOP * vmin),
        Vec2::new(size.x - safe.right - NEEDLE_IN * vmin, size.y - NEEDLE_IN * vmin),
    );
    let clear = (NEEDLE * 0.5 + NEEDLE_CLEAR) * vmin;
    let (jump_right, _, jump_size) = hud::ARENA_JUMP;
    let (_, chat_bottom, chat_size) = hud::CHAT_BUTTON;
    let over = size.y - crate::JUMP_BOTTOM_VH * vh - (chat_bottom + chat_size) * vmin - clear;
    let buttons = Clear {
        least: Vec2::new(size.x - crate::JUMP_RIGHT_VW * vw - (jump_right + jump_size) * vmin - clear, over),
        most: size + Vec2::splat(1.0),
        instead: Some(Vec2::new(within.1.x, over)),
    };
    let stick = Clear {
        least: Vec2::new(-1.0, size.y - crate::STICK_BOTTOM_VH * vh - crate::STICK_SIZE_VMIN * vmin - clear),
        most: Vec2::new(crate::STICK_LEFT_VW * vw + crate::STICK_SIZE_VMIN * vmin + clear, size.y + 1.0),
        instead: None,
    };
    (within, [buttons, stick])
}

/// Whether any of the golden island is on a screen `size` across, its
/// middle being at `middle` on it and its shore `shore` pixels out from
/// that.
fn island_seen(size: Vec2, middle: Vec2, shore: f32) -> bool {
    middle.clamp(Vec2::ZERO, size).distance(middle) < shore
}

/// While the race is on and none of the golden island is on the screen,
/// the arrow at the edge of the screen pointing the way to it, with how
/// far it is from you to its shore just inside it ("63m"), kept to the
/// screen's room for it ([`needle_room`]); put away otherwise, and while
/// you are dead. Until 2026-11-18, the same day, it stood until the
/// island's middle came on the screen, its gold words over the island's
/// own gold band by then, and anywhere on the edge, under a thumb or a
/// phone's island (seen in a play-test).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(super) fn point(
    racing: Res<Racing>,
    safe: Res<SafeArea>,
    me: Query<(&Transform, &Venue, Has<Dead>), With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    children: Query<&Children>,
    mut needles: Query<(&mut Visibility, &mut UiTransform, &ComputedNode), (With<Needle>, Without<NeedleWords>)>,
    mut words: Query<(Entity, &mut Visibility, &mut UiTransform, &ComputedNode), (With<NeedleWords>, Without<Needle>)>,
    mut texts: Query<&mut Text, With<Ink>>,
) {
    let (Ok((mut needle_shown, mut needle_at, needle_node)), Ok((line, mut words_shown, mut words_at, words_node))) =
        (needles.single_mut(), words.single_mut())
    else {
        return;
    };
    let on = racing.0.as_ref().is_some_and(|race| race.on());
    let you = me.single().ok().filter(|(_, venue, dead)| on && **venue == Venue::BridgeRace && !dead);
    let gold = Vec3::new(GOLDEN.x, LIFT, GOLDEN.y);
    let island = on_the_screen(&cameras, gold).zip(on_the_screen(&cameras, gold + Vec3::X * ISLAND_RADIUS));
    let wanted = you.zip(island).zip(windows.single().ok());
    let size = |window: &Window| Vec2::new(window.width(), window.height());
    let Some(((you, (point, _)), window)) =
        wanted.filter(|((_, (point, shore)), window)| !island_seen(size(window), *point, point.distance(*shore)))
    else {
        needle_shown.set_if_neq(Visibility::Hidden);
        words_shown.set_if_neq(Visibility::Hidden);
        return;
    };
    let size = size(window);
    let vmin = size.min_element() / 100.0;
    let (within, kept_clear) = needle_room(size, *safe);
    let edge = at_the_edge(size, point, within, &kept_clear);
    let way = (point - edge).normalize_or(Vec2::X);
    let needle = needle_node.size() * needle_node.inverse_scale_factor();
    needle_at.set_if_neq(UiTransform {
        translation: Val2::px(edge.x - needle.x * 0.5, edge.y - needle.y * 0.5),
        scale: Vec2::ONE,
        // The arrow's picture points up the screen: turned to point that way.
        rotation: Rot2::radians(way.x.atan2(-way.y)),
    });
    needle_shown.set_if_neq(Visibility::Inherited);
    let away = (you.0.translation.xz().distance(GOLDEN) - ISLAND_RADIUS).max(0.0);
    let said = fill(&ui().words.bridge_race.away, &[("metres", &format!("{away:.0}"))]);
    rewrite(line, &said, &children, &mut texts);
    // Inward from the arrow, toward the middle of the screen: clear of the
    // buttons, now that the arrow is.
    let inward = (size * 0.5 - edge).normalize_or(Vec2::NEG_X);
    let words_size = words_node.size() * words_node.inverse_scale_factor();
    let past = 0.5 * (inward.x.abs() * words_size.x + inward.y.abs() * words_size.y);
    let middle = edge + inward * ((NEEDLE * 0.5 + NEEDLE_GAP) * vmin + past);
    words_at.set_if_neq(UiTransform {
        translation: Val2::px(middle.x - words_size.x * 0.5, middle.y - words_size.y * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
    words_shown.set_if_neq(Visibility::Inherited);
}

/// How far `feet` are from the golden island's shore, if that is near
/// enough to jump ([`JUMP_FROM`]) and you have too little bridge left,
/// `metres`, to lay across it, standing on something (not in the air, as
/// you jump, nor falling): where "Jump!" is said.
fn jump_gap(feet: Vec3, metres: f32) -> Option<f32> {
    let gap = feet.xz().distance(GOLDEN) - ISLAND_RADIUS;
    let standing = (feet.y - LIFT).abs() <= 0.3;
    (standing && gap > JUMP_NEAREST && gap <= JUMP_FROM && metres < gap).then_some(gap)
}

/// "Jump!" in the golden island's gold, over the gap between you and its
/// shore, while you stand near enough to jump it with too little bridge to
/// lay across it ([`jump_gap`]), the race on and you on your feet; put
/// away otherwise.
#[allow(clippy::type_complexity)]
pub(super) fn jump_hint(
    racing: Res<Racing>,
    stock: Res<Resources>,
    me: Query<(&Transform, &Venue, Has<Dead>), With<Player>>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut words: Query<(&mut Visibility, &mut UiTransform, &ComputedNode), With<JumpWords>>,
) {
    let Ok((mut shown, mut place, node)) = words.single_mut() else {
        return;
    };
    let on = racing.0.as_ref().is_some_and(|race| race.on());
    let feet = me.single().ok().filter(|(_, venue, dead)| on && **venue == Venue::BridgeRace && !dead).map(|(body, ..)| body.translation);
    let over = feet.and_then(|feet| {
        let gap = jump_gap(feet, stock.bridge_metres)?;
        let out = (feet.xz() - GOLDEN).normalize_or(Vec2::X) * (ISLAND_RADIUS + gap * 0.5) + GOLDEN;
        on_the_screen(&cameras, Vec3::new(out.x, LIFT, out.y) + JUMP_OVER)
    });
    let Some(point) = over.filter(|point| windows.single().is_ok_and(|window| on_screen(window, *point))) else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    put(&mut place, node, point);
    shown.set_if_neq(Visibility::Inherited);
}

/// `words` rising `by` metres up the screen from `from`, in the world, in
/// `colour`, `size` big, with a thin stroke, fading as they go, for `secs`
/// seconds: "+3m" off you as a grant of bridge comes, and "Hold broken!"
/// over the golden island.
pub(super) fn rising(commands: &mut Commands, from: Vec3, words: &str, colour: Color, size: f32, by: f32, secs: f32) {
    let mut inks = Vec::new();
    let line = stroked(commands, None, &[(words.to_string(), colour)], size, 1.0, &mut inks);
    let (node, place, shown, _) = placed();
    commands.entity(line).insert((
        InArena,
        Rising {
            from,
            by,
            secs,
            age: 0.0,
            inks,
        },
        node,
        place,
        shown,
    ));
}

/// "+3m" rising off you, at `feet`, as a grant of bridge comes: white, as
/// the metres left are written in front of you while you build.
pub(super) fn granted(commands: &mut Commands, feet: Vec3, words: &str) {
    rising(commands, feet + RISE_FROM, words, ui().colours.bridge_race.granted, RISE_TEXT, RISE_BY, RISE_SECS);
}

/// "Hold broken!" over the golden island in `side`'s colour, as its hold
/// on it is broken (mine: the ring emptied and nothing said why, seen in a
/// play-test; a jump breaks it as a knock does, being off its ground).
pub(super) fn broken(commands: &mut Commands, side: u8) {
    let words = ui().words.bridge_race.hold_broken.clone();
    let from = Vec3::new(GOLDEN.x, LIFT, GOLDEN.y) + BROKEN_AT;
    rising(commands, from, &words, team_ink(side), BROKEN_TEXT, BROKEN_RISE, BROKEN_SECS);
}

/// Floats whatever is rising up off where it began, fading as it goes, and
/// takes it away once it has risen.
#[allow(clippy::type_complexity)]
pub(super) fn rise(
    time: Res<Time>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut risings: Query<(Entity, &mut Rising, &mut Visibility, &mut UiTransform, &ComputedNode)>,
    mut inks: Query<(&Ink, &mut TextColor, Option<&mut TextShadow>)>,
    mut commands: Commands,
) {
    if risings.is_empty() {
        return;
    }
    let window = windows.single().ok();
    for (thing, mut rising, mut shown, mut place, node) in &mut risings {
        rising.age += time.delta_secs();
        let done = rising.age / rising.secs.max(1e-3);
        if done >= 1.0 {
            commands.entity(thing).despawn();
            continue;
        }
        let at = rising.from + Vec3::NEG_Z * (rising.by * done);
        let point = on_the_screen(&cameras, at).filter(|point| window.is_some_and(|window| on_screen(window, *point)));
        let Some(point) = point else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        put(&mut place, node, point);
        fade(&rising.inks, 1.0 - done * done, &mut inks);
        shown.set_if_neq(Visibility::Inherited);
    }
}

/// A line across the upper half saying what has just happened, `words`,
/// with `picture` after it if it has one (the dynamite's, as a stick comes
/// to your button): up for a few seconds, one at a time, any line up
/// before it, this game's or Bridge Wars' own, taken down first. The line,
/// to take down early.
pub(super) fn notice(commands: &mut Commands, say: &mut Say, words: &str, picture: Option<Picture>) -> Entity {
    clear_notices(commands, say);
    let image = picture.map(|picture| say.pictures.of(&mut say.images, picture));
    // Put up clear, to be brought up.
    let ink = ui().colours.arena.alert.with_alpha(0.0);
    let posted = commands
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
        })
        .id();
    *say.posted = Some(posted);
    posted
}

/// Takes down every line across the upper half: those up, and one put up
/// this very frame, which the world has yet to be told of (two said in one
/// frame were drawn over each other, seen in a scripted run).
pub(super) fn clear_notices(commands: &mut Commands, say: &mut Say) {
    for old in say.notices.iter().chain(say.posted.take()) {
        commands.entity(old).try_despawn();
    }
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

/// Words across the middle of the screen for `secs` seconds, the race's
/// own headline: where and as big as `hud::announce` puts its own (the
/// title, `title`, under its caption, `caption`, high enough to leave you
/// in sight), each a line of parts in their colours, a team's name in its
/// own (mine: "You are on Red" and "Red wins!" never were, seen in a
/// play-test), with a thin stroke round them (white over the stands of
/// the islands and gold over the golden island's lilac read only by their
/// shadow); fading in and out as `hud::announce`'s do. The end's go as the
/// table's results come up, `till_results`. The words, to keep with the
/// table.
pub(super) fn banner(commands: &mut Commands, caption: Option<&[(String, Color)]>, title: &[(String, Color)], secs: f32, till_results: bool) -> Entity {
    let held = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(Val::Vh(hud::ABOVE_YOU)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    // Put up clear, for `fade_banners` to bring up.
    let mut inks = Vec::new();
    if let Some(caption) = caption {
        stroked(commands, Some(held), caption, hud::CAPTION, 0.0, &mut inks);
    }
    stroked(commands, Some(held), title, hud::HEADLINE, 0.0, &mut inks);
    commands.entity(held).insert(Banner {
        age: 0.0,
        secs,
        till_results,
        inks,
    });
    held
}

/// Brings words across the middle of the screen up, holds them, and takes
/// them away again: written only while they fade. The end's go as soon as
/// the table's results are up.
pub(super) fn fade_banners(
    time: Res<Time>,
    results: Query<(), With<ResultsWindow>>,
    mut banners: Query<(Entity, &mut Banner)>,
    mut inks: Query<(&Ink, &mut TextColor, Option<&mut TextShadow>)>,
    mut commands: Commands,
) {
    for (entity, mut banner) in &mut banners {
        banner.age += time.delta_secs();
        if banner.age >= banner.secs || (banner.till_results && !results.is_empty()) {
            commands.entity(entity).despawn();
            continue;
        }
        let solid = (banner.age / BANNER_IN).min((banner.secs - banner.age) / BANNER_OUT).clamp(0.0, 1.0);
        fade(&banner.inks, solid, &mut inks);
    }
}

/// The ring round the rim of the plank button that fills toward the next
/// grant of bridge, where the button is (`right`, `bottom`, as the box
/// round the jump button has it) and as big; put away until the race is
/// on (mine: "+6m" came every 16 s, a sixth of a second's walk, and
/// nothing said when, seen in a play-test that stood at the end of its
/// bridge 85 seconds in every hundred).
pub(super) fn next_grant(images: &mut Assets<Image>, right: f32, bottom: f32) -> impl Bundle {
    let image = images.add(Image::new_fill(
        Extent3d {
            width: NEXT_PIXELS,
            height: NEXT_PIXELS,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    (
        NextGrant {
            image: image.clone(),
            drawn: None,
        },
        Name::new("The next grant"),
        ImageNode::new(image),
        Node {
            width: Val::VMin(hud::ACTION_BUTTON),
            height: Val::VMin(hud::ACTION_BUTTON),
            ..hud::beside_the_jump(right, bottom)
        },
        Visibility::Hidden,
        Pickable::IGNORE,
    )
}

/// The ring's picture, `share` of the way round filled from the top the
/// way the clock goes, in `fill`, over the rest of it in `track`: a band
/// between [`NEXT_IN`] and [`NEXT_OUT`] of the picture's half, its edges
/// soft by a pixel, clear inside and out.
fn next_picture(share: f32, fill: Color, track: Color) -> Vec<u8> {
    let half = NEXT_PIXELS as f32 * 0.5;
    let (inner, outer) = (NEXT_IN * half, NEXT_OUT * half);
    let (fill, track) = (fill.to_srgba(), track.to_srgba());
    let mut data = vec![0u8; (NEXT_PIXELS * NEXT_PIXELS * 4) as usize];
    for y in 0..NEXT_PIXELS {
        for x in 0..NEXT_PIXELS {
            let at = Vec2::new(x as f32 + 0.5 - half, y as f32 + 0.5 - half);
            let out = at.length();
            let cover = (out - inner + 0.5).clamp(0.0, 1.0).min((outer - out + 0.5).clamp(0.0, 1.0));
            if cover <= 0.0 {
                continue;
            }
            // From the top, the way the clock goes (the picture's y down).
            let mut angle = at.x.atan2(-at.y);
            if angle < 0.0 {
                angle += std::f32::consts::TAU;
            }
            let colour = if angle / std::f32::consts::TAU < share { fill } else { track };
            let k = ((y * NEXT_PIXELS + x) * 4) as usize;
            data[k..k + 4].copy_from_slice(&colour.with_alpha(colour.alpha * cover).to_u8_array());
        }
    }
    data
}

/// Fills the ring round the plank button toward the next grant, as far as
/// the race has come since the last ([`super::Race::since_grant`]), drawn
/// again only as it comes to another of its [`NEXT_STEPS`]; shown while
/// the race is on.
pub(super) fn count_to_grant(racing: Res<Racing>, mut rings: Query<(&mut NextGrant, &mut Visibility)>, mut images: ResMut<Assets<Image>>) {
    let share = racing.0.as_ref().filter(|race| race.on()).map(|race| (race.since_grant / GRANT_EVERY).clamp(0.0, 1.0));
    for (mut ring, mut shown) in &mut rings {
        let Some(share) = share else {
            shown.set_if_neq(Visibility::Hidden);
            continue;
        };
        let step = ((share * NEXT_STEPS as f32).floor() as u32).min(NEXT_STEPS);
        if ring.drawn != Some(step) {
            ring.drawn = Some(step);
            let colours = ui().colours.bridge_race;
            let data = next_picture(step as f32 / NEXT_STEPS as f32, colours.next_grant, colours.next_grant_track);
            if let Some(mut image) = images.get_mut(&ring.image) {
                image.data = Some(data);
            }
        }
        shown.set_if_neq(Visibility::Inherited);
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
        // Over the map's own band, wider than it either way; and the dark
        // band under it wider still, the whole way round.
        const { assert!(RING_IN < 5.5 && RING_OUT > 6.0 && RING_EDGE > 0.0) };
        let edge = band_mesh(RING_IN - RING_EDGE, RING_OUT + RING_EDGE, 1.0, false);
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(corners)) = edge.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("no corners");
        };
        assert!(corners.iter().all(|corner| {
            let out = Vec2::new(corner[0], corner[2]).length();
            out < RING_IN - RING_EDGE * 0.5 || out > RING_OUT + RING_EDGE * 0.5
        }));
    }

    /// A phone held on its side, in points, and a desktop window.
    const PHONE: Vec2 = Vec2::new(852.0, 393.0);
    const DESK: Vec2 = Vec2::new(1920.0, 1080.0);

    #[test]
    fn the_arrow_stands_in_from_the_edge_the_way_the_golden_island_is_clear_of_the_thumbs_and_a_phone_s_island() {
        for size in [PHONE, DESK] {
            let vmin = size.min_element() / 100.0;
            let (within, kept_clear) = needle_room(size, SafeArea::default());
            let middle = size * 0.5;
            // Off to the right, a little up: on the right edge, in by the
            // inset, a little over the middle.
            let at = at_the_edge(size, middle + Vec2::new(3000.0, -200.0), within, &kept_clear);
            assert!((at.x - (size.x - NEEDLE_IN * vmin)).abs() < 1e-3 && at.y < middle.y, "{size}: {at}");
            // Off to the right and down, toward the buttons round the jump
            // button: on the right edge over them, clear of every one of
            // them, and never under the plate and the minimap.
            let (buttons, stick) = (kept_clear[0], kept_clear[1]);
            for down in [400.0, 2000.0] {
                let at = at_the_edge(size, middle + Vec2::new(3000.0, down), within, &kept_clear);
                assert!(!buttons.holds(at) && !stick.holds(at), "{size}: {at} in {buttons:?}");
                assert!((at.x - within.1.x).abs() < 1e-3 && at.y <= buttons.least.y + 1e-3, "{size}: {at}");
                assert!(at.y >= NEEDLE_TOP * vmin - 1e-3, "{size}: {at}");
            }
            // Down more than to the right: on the bottom edge, between the
            // stick and the buttons.
            let at = at_the_edge(size, middle + Vec2::new(3000.0, 9000.0), within, &kept_clear);
            assert!(!buttons.holds(at) && !stick.holds(at) && (at.y - within.1.y).abs() < 1e-3, "{size}: {at}");
            // Straight down: along the bottom, out of the stick's way and
            // the buttons'.
            for across in [-200.0, -3000.0] {
                let at = at_the_edge(size, middle + Vec2::new(across, 5000.0), within, &kept_clear);
                assert!(!buttons.holds(at) && !stick.holds(at), "{size}: {at}");
            }
            // Straight up: the top of where it may be, under the plate.
            let at = at_the_edge(size, middle + Vec2::new(0.0, -5000.0), within, &kept_clear);
            assert!((at - Vec2::new(middle.x, NEEDLE_TOP * vmin)).length() < 1e-3, "{size}: {at}");
        }
        // An iPhone held on its side, its island 59 points in on either
        // side: the arrow stands inside its safe area.
        let safe = SafeArea { left: 59.0, right: 59.0 };
        let (within, kept_clear) = needle_room(PHONE, safe);
        let at = at_the_edge(PHONE, Vec2::new(3000.0, 150.0), within, &kept_clear);
        assert!(at.x <= PHONE.x - 59.0 - NEEDLE_IN * PHONE.min_element() / 100.0 + 1e-3, "{at}");
    }

    #[test]
    fn the_arrow_goes_once_any_of_the_golden_island_is_on_the_screen() {
        // Its middle off to the right, its shore further than that from
        // the screen's edge: none of it seen.
        assert!(!island_seen(PHONE, Vec2::new(1100.0, 200.0), 200.0));
        // Its shore on the screen, its middle still off it: seen.
        assert!(island_seen(PHONE, Vec2::new(1000.0, 200.0), 200.0));
        assert!(island_seen(PHONE, Vec2::new(400.0, 200.0), 100.0));
    }

    #[test]
    fn jump_is_said_only_a_jump_from_the_golden_island_with_too_little_bridge_to_lay_across() {
        let from = |gap: f32| Vec3::new(-(ISLAND_RADIUS + gap), LIFT, 0.0);
        // Three metres short with nothing left: jump.
        assert_eq!(jump_gap(from(3.0), 0.0).map(|gap| (gap * 10.0).round()), Some(30.0));
        // With bridge enough to lay across, build; too far to jump, wait;
        // on its shore already, nothing; in the air or falling, nothing.
        assert_eq!(jump_gap(from(3.0), 4.0), None);
        assert_eq!(jump_gap(from(9.0), 0.0), None);
        assert_eq!(jump_gap(from(0.2), 0.0), None);
        assert_eq!(jump_gap(from(3.0) - Vec3::Y * 2.0, 0.0), None);
        assert_eq!(jump_gap(from(3.0) + Vec3::Y * 1.0, 0.0), None, "in the air, jumping");
        // A running jump carries a body further than it is said from, with
        // a metre in hand: the game's own jump and fall, at the race's run.
        // (Until the evening of 2026-11-18 it was said for every gap a
        // grant of one against one could leave, 6 m; since then a grant is
        // 8 to 16 m, and a gap it leaves wider than a jump is a grant's
        // wait, as the next grant's size on the plate says.)
        let airborne = 2.0 * crate::JUMP_SPEED / crate::GRAVITY;
        assert!(ui().arena.run_speed * airborne > JUMP_FROM + 1.0, "{} m", ui().arena.run_speed * airborne);
    }

    #[test]
    fn the_plate_says_what_your_next_grant_brings_in_whole_metres_or_to_a_half() {
        let said = |metres: &str| fill(&ui().words.bridge_race.next_item, &[("metres", metres)]);
        assert_eq!(next_words(14.0), said("14"));
        assert_eq!(next_words(6.5), said("6.5"));
        assert_eq!(next_words(8.0), said("8"));
        // The file's own words, as written.
        assert_eq!(said("14"), "Next Item: 14m");
        // Each team's rows of how far it has to go are put away for it.
        assert!(!SIDES_SHOWN);
    }

    #[test]
    fn the_ring_round_the_plank_button_fills_the_way_the_clock_goes_on_its_rim() {
        let (fill, track) = (Color::srgba(1.0, 1.0, 1.0, 1.0), Color::srgba(0.0, 0.0, 1.0, 0.5));
        let pixel = |data: &[u8], at: Vec2| {
            let (x, y) = (at.x as u32, at.y as u32);
            let k = ((y * NEXT_PIXELS + x) * 4) as usize;
            [data[k], data[k + 1], data[k + 2], data[k + 3]]
        };
        let half = NEXT_PIXELS as f32 * 0.5;
        let on_rim = |turn: f32| {
            let way = Vec2::new(turn.sin(), -turn.cos());
            Vec2::splat(half) + way * (NEXT_IN + NEXT_OUT) * 0.5 * half
        };
        // A quarter filled: the top right of the rim filled, the rest the
        // track; the middle and the corners clear.
        let data = next_picture(0.25, fill, track);
        assert_eq!(pixel(&data, on_rim(0.1 * std::f32::consts::TAU)), [255, 255, 255, 255]);
        assert_eq!(pixel(&data, on_rim(0.6 * std::f32::consts::TAU))[2], 255);
        assert!(pixel(&data, on_rim(0.6 * std::f32::consts::TAU))[3] < 140);
        assert_eq!(pixel(&data, Vec2::splat(half))[3], 0);
        assert_eq!(pixel(&data, Vec2::ZERO)[3], 0);
        // Nothing filled, and all of it.
        assert!(pixel(&next_picture(0.0, fill, track), on_rim(0.01))[3] < 140);
        assert_eq!(pixel(&next_picture(1.0, fill, track), on_rim(0.99 * std::f32::consts::TAU)), [255, 255, 255, 255]);
        // On the button's own rim, inside its disc.
        const { assert!(NEXT_IN > 0.6 && NEXT_OUT < 1.0 && NEXT_IN < NEXT_OUT) };
    }

    #[test]
    fn the_plate_says_how_far_a_bridge_has_to_go_or_that_it_is_across_or_who_stands_there() {
        // Nobody across and nothing laid: the whole way.
        let whole = the_way_across(0);
        assert_eq!(progress_words(0, 0, 0, 2), fill(&ui().words.bridge_race.to_go, &[("metres", &format!("{:.0}", whole.ceil()))]));
        assert_eq!(progress_words(1, 100, 0, 2), ui().words.bridge_race.across);
        assert_eq!(progress_words(1, 100, 1, 2), fill(&ui().words.bridge_race.on_island, &[("count", "1"), ("of", "2")]));
        assert_eq!(progress_words(0, 40, 1, 1), fill(&ui().words.bridge_race.on_island, &[("count", "1"), ("of", "1")]));
        // One of two gone for good, the one left there is all of the team.
        assert_eq!(progress_words(1, 100, 1, 1), fill(&ui().words.bridge_race.on_island, &[("count", "1"), ("of", "1")]));
        // Each bar a sliver of its colour with nothing laid, and as long as
        // its bridge has got after that.
        assert_eq!(bar_share(0), SLIVER);
        assert_eq!(bar_share(50), 0.5);
        assert_eq!(bar_share(100), 1.0);
    }

    #[test]
    fn the_teams_colours_are_drawn_as_solid_as_the_file_says_and_no_more() {
        // The file's strengths, and the teams' own colour under them.
        let colours = ui().colours;
        let red = colours.arena.team(0);
        let bar = BridgeRaceColours::solid(red, colours.bridge_race.bar_strength);
        assert_eq!(bar.alpha(), colours.bridge_race.bar_strength.clamp(0.0, 1.0));
        assert_eq!(bar.with_alpha(1.0), red.with_alpha(1.0), "the team's own colour");
        assert_eq!(team_ink(1).alpha(), colours.bridge_race.hold_strength.clamp(0.0, 1.0));
        // Out of 0 to 1, held to it; no number, all of it.
        assert_eq!(BridgeRaceColours::solid(red, 3.0).alpha(), 1.0);
        assert_eq!(BridgeRaceColours::solid(red, -1.0).alpha(), 0.0);
        assert_eq!(BridgeRaceColours::solid(red, f32::NAN).alpha(), 1.0);
    }

    #[test]
    fn stroked_words_are_the_words_over_four_copies_of_them_in_the_stroke_s_colour() {
        let mut world = World::new();
        let parts = vec![("Red".to_string(), Color::srgb(1.0, 0.0, 0.0)), (" wins!".to_string(), Color::srgb(1.0, 0.8, 0.0))];
        let mut inks = Vec::new();
        let line = {
            let mut commands = world.commands();
            stroked(&mut commands, None, &parts, 5.0, 0.0, &mut inks)
        };
        world.flush();
        // Four copies of the whole of it, the words, and a span for each
        // part after the first: every one of them clear, to be brought up.
        assert_eq!(inks.len(), 6);
        let kids: Vec<Entity> = world.get::<Children>(line).expect("its pieces").iter().collect();
        assert_eq!(kids.len(), 5);
        let rims = kids.iter().filter(|kid| world.get::<Text>(**kid).is_some_and(|text| text.0 == "Red wins!")).count();
        assert_eq!(rims, 4);
        assert!(kids.iter().any(|kid| world.get::<Text>(*kid).is_some_and(|text| text.0 == "Red")));
        for ink in &inks {
            assert_eq!(world.get::<TextColor>(*ink).expect("a colour").0.alpha(), 0.0);
        }
        let stroke = ui().colours.chat.stroke;
        assert_eq!(inks.iter().filter(|ink| world.get::<Ink>(**ink).is_some_and(|own| own.0 == stroke)).count(), 4);
    }
}
