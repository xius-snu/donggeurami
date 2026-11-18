//! What a game of Last Stand says on the screen, the same online and
//! offline, from how the game stands ([`Stand`]):
//!
//! * **The plate top centre** ([`show_plate`]), on the plate's grey as the
//!   table's board is, which is put away for the play: in a wave, "Wave 7"
//!   over "12 left"; between waves, "Next wave in 0:18" over what came
//!   before it ("Wave 7 cleared"), what the next brings that is new
//!   ("Next: A giant!"), or before the first where to buy what, or how
//!   many of you are ready; inside it, under its big line, the anchor's
//!   health, a bar with its number in it, flashing red as it is hit; and
//!   beside it, between waves, a Ready button (R on desktop): the next wave
//!   comes three seconds after everyone standing is ready (the rules'
//!   `READY_SECS`).
//! * **Across the screen**, once each as it comes ([`say`]): "Go!" as the
//!   play begins; "Wave 7" as one comes, with what is new in it under it
//!   ("Brutes!", "A giant!"); "Wave 7 cleared"; "The anchor is broken!"
//!   over "No more coming back"; and at the end "The last stand fell at
//!   wave 9". And smaller, over them, "The anchor is under attack!" while
//!   it is hit off the screen ([`warn`]). None of it while a shop's window
//!   is up, which it would show through ([`quiet_in_the_shop`]), and none
//!   under the results.
//! * **In the chat**: what someone bought the team ("Mochi bought
//!   Sharpness Lvl 3"), who else is down, who else is out and who has left,
//!   a giant brought down, the anchor broken; names in the team's colour,
//!   as what anyone types here is written.
//! * **You**: dead, "Back in 6", counting down in whole seconds, low on the
//!   screen, the kit's own words (`arena::DeathWords`), and the camera on
//!   the anchor, where you will stand again; out, with the anchor broken,
//!   a slim window along the foot of the screen saying so with a button to
//!   leave, and the camera on whoever of your team still stands, or on the
//!   anchor's place, to the end; at the end, and as the crystal shatters
//!   off the screen, the camera on it too ([`watch_view`]).
//! * **The shops' cards** ([`stalls`]): the item shop's swords, shields
//!   and things to use, by Bridge Wars' own pictures and names at Last
//!   Stand's prices; the upgrade shop's six lines, an endless one's card
//!   saying the level it takes the team to ("Next: Lvl 7") over its price.

use bevy::prelude::*;
use roundtown_net::bridge_wars::{FAST, HEALING, LEAP};
use roundtown_net::last_stand::{
    self as rules, DYNAMITE_HIT, FIREBALL_HIT, LEAP_SECS, LUCKY_MOST, PROTECTION, PROTECTION_MOST, SHIELD_SHARE, SPEED_SECS,
    SWORD_DAMAGE, StandRound, StandStage, StandUpgrades, UNSEEN_SECS, Ware, first_in, giants, price,
};
use roundtown_net::{Item, Kit, Purse, Utility, Venue};

use super::looks::{CrystalSeen, Crystal, blend};
use super::{Announced, Happened, ReadyAsked, Said, Stand, StandSystems, TEAM, TeamScore};
use crate::arena::{self, DeathWords, InArena};
use crate::chat;
use crate::clock::TownClock;
use crate::hud::{self, Picture};
use crate::market::{self, Offer, Pick, State, Stall, Stalls};
use crate::table::{self, InTable, ResultsWindow, TableView};
use crate::tags::HidesNames;
use crate::ui::{UiChanged, fill, ui};
use crate::window::{self, Look};
use crate::{Lookout, Player};

/// Sizes, as shares of the short side of the screen: the plate's big line
/// and the line under it, as the table's board's are; the anchor's bar
/// under them, thick enough to read from the corner of an eye and to hold
/// its number (mine, since 2026-11-18: at 1.1 it was a hairline a
/// play-tester never saw go down), and its number; the slim window along
/// the foot of the screen that says you are out, and how far up from the
/// foot it stands.
const PLATE_TEXT: f32 = 4.8;
const PLATE_CAPTION: f32 = 3.4;
const PLATE_BAR_WIDTH: f32 = 30.0;
const PLATE_BAR_HEIGHT: f32 = 2.8;
const PLATE_NUMBER: f32 = 2.2;
const READY_GAP: f32 = 1.8;
const OUT_UP: f32 = 3.0;
/// How far down the screen "Back in 6" stands in this game, as a share of
/// the screen's height: under the middle, where you and the crystal are
/// (the camera looks on at the anchor while you are dead, [`watch_view`]),
/// and over the buttons along the foot; the kit's own place for it, 38%
/// down, stood it on the crystal and its numbers (mine, 2026-11-18).
const BACK_DOWN: f32 = 70.0;
/// How long each thing said across the screen is up, in seconds: the
/// game's beginning, a wave's coming, a wave cleared, the anchor broken,
/// and the end.
const GO_SECS: f32 = 2.5;
const WAVE_SECS: f32 = 2.6;
const CLEARED_SECS: f32 = 2.4;
const BROKEN_SECS: f32 = 3.2;
const FELL_SECS: f32 = 4.0;
/// "The anchor is under attack!": how big, as a share of the short side of
/// the screen; how far down it stands, as a share of the screen's height
/// (clear under the plate and over what is said across the middle); how
/// long it is up, and the least while between two, in seconds (Bridge
/// Wars' own, `arena::ANCHOR_WARN_EVERY`, for its own anchor).
const WARN_TEXT: f32 = 4.0;
const WARN_DOWN: f32 = 22.0;
const WARN_SECS: f32 = 2.6;
const WARN_EVERY: f32 = 8.0;
/// Whether a wave cleared is said in the chat too, as well as across the
/// screen and on the plate (off since 2026-11-18, mine: it was said three
/// times at once, and pushed what people did out of the chat's three
/// lines; the line is kept, `words.last_stand.cleared_line`).
const CLEARED_IN_CHAT: bool = false;
/// How fast the camera goes to what it looks on at and comes back to you,
/// as so many times a second it closes the way left: half of it in about
/// a ninth of a second, nearly all of it in half a second (mine).
const EASE: f32 = 6.0;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(tap).add_systems(
        Update,
        (
            (spawn_screen, ready_key, say, warn, chat_lines, quiet_in_the_shop).chain().after(StandSystems),
            ((death_words, lower_death_words).chain(), show_plate, show_out, watch_view)
                .after(StandSystems)
                .before(crate::follow_camera),
        ),
    );
}

/// The plate top centre and what is in it.
#[derive(Component)]
struct Plate;
#[derive(Component)]
struct PlateText;
#[derive(Component)]
struct PlateCaption;
#[derive(Component)]
struct PlateFill;
#[derive(Component)]
struct PlateNumber;
/// The slim window that says you are out.
#[derive(Component)]
struct OutPane;
/// "The anchor is broken!" across the screen, which the end takes down
/// for its own.
#[derive(Component)]
struct BrokenLine;

/// What this game says high on the screen rather than across its middle:
/// "The anchor is under attack!" ([`warn`]). The arrow round you, which
/// stands where what is said across the middle does when it points up the
/// screen, is put away only for that (`looks::toward_the_anchor`).
#[derive(Component)]
pub(super) struct Aloft;

/// What a press on one of this game's own buttons does: Ready, under the
/// plate between waves; and Leave, on the window that says you are out.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum StandTap {
    Ready,
    Leave,
}

/// Puts up the plate, the Ready button beside it and the window that says
/// you are out (put away until they are wanted) as a game of Last Stand
/// begins here, and again in the file's words and colours as it changes
/// on desktop. All of it is the table's (`InTable`) and the fight's
/// (`InArena`), and goes with whichever goes first.
fn spawn_screen(
    stand: Option<Res<Stand>>,
    changed: Res<UiChanged>,
    plates: Query<Entity, Or<(With<Plate>, With<OutPane>)>>,
    mut commands: Commands,
) {
    let Some(stand) = stand else {
        return;
    };
    let again = changed.is_changed() && !changed.is_added() && !plates.is_empty();
    if !stand.is_added() && !again && !plates.is_empty() {
        return;
    }
    for plate in &plates {
        commands.entity(plate).try_despawn();
    }
    let ui = ui();
    let colours = ui.colours;
    commands
        .spawn((
            InTable,
            InArena,
            Plate,
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
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|plate| {
            // The plate's box: its big line, and the anchor's health under
            // it; and beside the box, not under it, Ready, clear of a
            // shop's window, which comes down to just under the line under
            // the box (`market`).
            plate
                .spawn((
                    HidesNames,
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::VMin(0.5),
                        padding: UiRect::new(Val::VMin(3.0), Val::VMin(3.0), Val::VMin(0.7), Val::VMin(1.1)),
                        border_radius: BorderRadius::all(Val::VMin(3.0)),
                        ..default()
                    },
                    BackgroundColor(colours.panel),
                    Pickable::IGNORE,
                ))
                .with_children(|plate_box| {
                    plate_box.spawn((PlateText, hud::words("", PLATE_TEXT, colours.text)));
                    // The anchor's bar, its fill under its number.
                    plate_box.spawn((
                        Node {
                            width: Val::VMin(PLATE_BAR_WIDTH),
                            height: Val::VMin(PLATE_BAR_HEIGHT),
                            border: UiRect::all(Val::VMin(0.15)),
                            border_radius: BorderRadius::all(Val::VMin(0.7)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(colours.arena.hp_back),
                        BorderColor::all(colours.arena.hp_rim),
                        Pickable::IGNORE,
                        children![
                            (
                                PlateFill,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::ZERO,
                                    top: Val::ZERO,
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    border_radius: BorderRadius::all(Val::VMin(0.5)),
                                    ..default()
                                },
                                BackgroundColor(colours.last_stand.anchor),
                                Pickable::IGNORE,
                            ),
                            (PlateNumber, hud::words("", PLATE_NUMBER, colours.text)),
                        ],
                    ));
                    plate_box.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(100.0),
                            top: Val::Percent(50.0),
                            margin: UiRect::left(Val::VMin(READY_GAP)),
                            ..default()
                        },
                        UiTransform::from_translation(Val2::new(Val::ZERO, Val::Percent(-50.0))),
                        Pickable::IGNORE,
                        children![(StandTap::Ready, HidesNames, window::choice(ui.words.last_stand.ready.clone(), Look::Yes))],
                    ));
                });
            plate.spawn((PlateCaption, hud::words("", PLATE_CAPTION, colours.text)));
        });
    let words = &ui.words.last_stand;
    commands
        .spawn((
            InTable,
            InArena,
            OutPane,
            HidesNames,
            // A slim window along the foot of the screen, between the stick
            // and the buttons round the jump, its words beside its Leave:
            // the rest of the screen is the anchor and the horde round it,
            // which it is there to let you watch (mine, since 2026-11-18;
            // half the screen's height up it, it stood on them).
            window::pane(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                bottom: Val::VMin(OUT_UP),
                padding: UiRect::axes(Val::VMin(window::PAD), Val::VMin(1.4)),
                ..default()
            }),
            UiTransform::from_translation(Val2::new(Val::Percent(-50.0), Val::ZERO)),
            Visibility::Hidden,
        ))
        .with_children(|pane| {
            // (A window is laid out from the top down, `window::pane`: its
            // one row, the words and then Leave beside them.)
            pane.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::VMin(3.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|row| {
                row.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::VMin(0.4),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|said| {
                    said.spawn(window::words(words.out.clone(), window::TITLE, colours.window.title));
                    said.spawn(window::words(words.out_line.clone(), window::NOTICE_TEXT, colours.window.status));
                });
                row.spawn((StandTap::Leave, window::choice(ui.words.table.leave.clone(), Look::No)));
            });
        });
}

/// A press on Ready, or on Leave: only what the press landed on.
fn tap(
    click: On<Pointer<Click>>,
    taps: Query<&StandTap>,
    mut ready: MessageWriter<ReadyAsked>,
    mut asks: MessageWriter<table::Ask>,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    match taps.get(click.entity) {
        Ok(StandTap::Ready) => {
            ready.write(ReadyAsked);
        }
        Ok(StandTap::Leave) => {
            asks.write(table::Ask::Leave);
        }
        Err(_) => {}
    }
}

/// R on desktop is Ready, between waves: but not while a line is typed into
/// the chat, or any dialog is up (`hud::DialogUp`), whose keys are its own
/// (until 2026-11-18, "ready" or "bro" typed in a break said you were).
fn ready_key(
    stand: Option<Res<Stand>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    dialog: Option<Res<hud::DialogUp>>,
    mut ready: MessageWriter<ReadyAsked>,
) {
    if dialog.is_some_and(|dialog| dialog.0) {
        return;
    }
    if stand.is_some_and(|stand| ready_wanted(&stand)) && keyboard.just_pressed(KeyCode::KeyR) {
        ready.write(ReadyAsked);
    }
}

/// Whether the Ready button is up: between waves, you standing and not
/// ready yet.
fn ready_wanted(stand: &Stand) -> bool {
    stand.round.as_ref().is_some_and(|round| {
        matches!(round.stage, StandStage::Break { .. }) && stand.standing(stand.seat) && !stand.ready && !round.is_ready(stand.seat)
    })
}

/// The last seconds of a break, in which the line under the plate says
/// only "Get ready" (mine).
const GET_READY_SECS: f64 = 5.0;

/// What the plate says, as the game stands at the town's clock `now`: its
/// big line and the line under it. Between waves, under the time to the
/// next: how many of you are ready, once anyone is; in the break before
/// the first, where to buy what (mine, since 2026-11-18: nothing taught a
/// first game which stand sold what); after one, what the next brings
/// that is new ("Next: A giant!", mine, so as to buy for it), or else the
/// wave cleared; and in the last seconds of it, "Get ready".
fn plate_words(stand: &Stand, now: f64) -> (String, String) {
    let file = ui();
    let words = &file.words.last_stand;
    let Some(round) = &stand.round else {
        return (String::new(), String::new());
    };
    let wave = round.wave.to_string();
    match round.stage {
        StandStage::Wave => (
            fill(&words.wave, &[("wave", &wave)]),
            fill(&words.left, &[("count", &round.left.to_string())]),
        ),
        StandStage::Break { until } => {
            let time = table::clock((until - now).max(0.0) as f32);
            let standing = (0..round.health.len()).filter(|&seat| stand.standing(seat)).count();
            let ready = (0..round.health.len()).filter(|&seat| stand.standing(seat) && round.is_ready(seat)).count();
            let under = if ready > 0 {
                fill(&words.ready_count, &[("count", &ready.to_string()), ("of", &standing.to_string())])
            } else if until - now <= GET_READY_SECS {
                words.get_ready.clone()
            } else if round.wave == 0 {
                words.tip.clone()
            } else if let Some(new) = new_in(round.wave + 1) {
                fill(&words.next_new, &[("new", &new)])
            } else {
                fill(&words.cleared_caption, &[("wave", &wave)])
            };
            (fill(&words.next_wave, &[("time", &time)]), under)
        }
        StandStage::Over => (fill(&words.wave, &[("wave", &wave)]), String::new()),
    }
}

/// What the anchor's bar on the plate says of it: how much it has left of
/// the most it has.
fn anchor_number(left: u16, most: u16) -> String {
    fill(&ui().words.last_stand.anchor_health, &[("left", &left.to_string()), ("most", &most.to_string())])
}

/// Shows the plate while the game is played and over (the table's board is
/// up before), and writes on it what it says, only as that changes: its
/// lines, the anchor's bar, its number and its flash as the anchor is hit
/// (red, with the crystal's own flash), and the Ready button between
/// waves.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn show_plate(
    stand: Option<Res<Stand>>,
    view: Res<TableView>,
    town: Res<TownClock>,
    crystal: Res<Crystal>,
    mut plates: Query<&mut Visibility, (With<Plate>, Without<StandTap>)>,
    mut texts: Query<&mut Text, (With<PlateText>, Without<PlateCaption>, Without<PlateNumber>)>,
    mut captions: Query<&mut Text, (With<PlateCaption>, Without<PlateText>, Without<PlateNumber>)>,
    mut numbers: Query<&mut Text, (With<PlateNumber>, Without<PlateText>, Without<PlateCaption>)>,
    mut fills: Query<(&mut Node, &mut BackgroundColor), With<PlateFill>>,
    mut buttons: Query<(&StandTap, &mut Visibility), Without<Plate>>,
    mut caption_shown: Query<&mut Visibility, (With<PlateCaption>, Without<Plate>, Without<StandTap>)>,
) {
    let Some(stand) = stand else {
        return;
    };
    let up = (view.playing() || view.over()) && stand.round.is_some();
    for mut plate in &mut plates {
        plate.set_if_neq(if up { Visibility::Inherited } else { Visibility::Hidden });
    }
    if !up {
        return;
    }
    let (big, small) = plate_words(&stand, town.now());
    for mut text in &mut texts {
        if text.0 != big {
            text.0.clone_from(&big);
        }
    }
    for mut text in &mut captions {
        if text.0 != small {
            text.0.clone_from(&small);
        }
    }
    if let Some(round) = &stand.round {
        let width = arena::bar_width(f32::from(round.anchor) / f32::from(round.anchor_most.max(1)));
        let colours = ui().colours.last_stand;
        let tint = blend(colours.anchor, colours.anchor_hit, crystal.flash());
        for (mut fill, mut colour) in &mut fills {
            if fill.width != width {
                fill.width = width;
            }
            if colour.0 != tint {
                colour.0 = tint;
            }
        }
        let said = anchor_number(round.anchor, round.anchor_most);
        for mut number in &mut numbers {
            if number.0 != said {
                number.0.clone_from(&said);
            }
        }
    }
    // A shop's window up comes down over the line under the plate
    // (`market`): the line is put away meanwhile.
    let shopping = stand.shopping();
    for mut shown in &mut caption_shown {
        shown.set_if_neq(if shopping { Visibility::Hidden } else { Visibility::Inherited });
    }
    let ready = if ready_wanted(&stand) { Visibility::Inherited } else { Visibility::Hidden };
    for (tap, mut shown) in &mut buttons {
        if *tap == StandTap::Ready {
            shown.set_if_neq(ready);
        }
    }
}

/// Whether you are out of the game: dead with the anchor broken, the game
/// not over.
fn out(stand: &Stand) -> bool {
    stand.round.as_ref().is_some_and(|round| {
        round.stage != StandStage::Over
            && round.health.get(stand.seat) == Some(&0)
            && round.back_at.get(stand.seat).is_some_and(|back| back.is_infinite())
    })
}

/// The window that says you are out, up while you are.
fn show_out(stand: Option<Res<Stand>>, mut panes: Query<&mut Visibility, With<OutPane>>) {
    let shown = stand.is_some_and(|stand| out(&stand));
    for mut pane in &mut panes {
        pane.set_if_neq(if shown { Visibility::Inherited } else { Visibility::Hidden });
    }
}

/// While you are dead and will stand again: "Back in 6", counting down by
/// the town's clock in whole seconds (mine, since 2026-11-18: "Back in
/// 5.5" read fussy), the kit's own words across the screen
/// (`arena::DeathWords`). Only in a game of Last Stand, the words taken
/// down once as it goes: with none here they are Bridge Wars' to put up
/// and take down (`arena::show_death`), and until 2026-11-18 this took
/// them down every frame they were up, its "You died. Respawning in 4..."
/// showing every other frame.
fn death_words(stand: Option<Res<Stand>>, town: Res<TownClock>, mut had: Local<bool>, mut words: DeathWords, mut commands: Commands) {
    let file = ui();
    let back_in = &file.words.last_stand.back_in;
    let Some(stand) = stand.as_deref() else {
        if std::mem::take(&mut *had) {
            words.say(&mut commands, back_in, None);
        }
        return;
    };
    *had = true;
    let left = stand.round.as_ref().and_then(|round| {
        (round.health.get(stand.seat) == Some(&0) && round.stage != StandStage::Over)
            .then(|| round.back_at.get(stand.seat).copied())
            .flatten()
            .filter(|back| back.is_finite())
            // (Never "Back in 0": the game stands you again a moment after
            // the clock says, a frame behind it, or more on a slow one.)
            .map(|back| whole_seconds(back - town.now()))
    });
    if left.is_none() && !words.up() {
        return;
    }
    words.say(&mut commands, back_in, left);
}

/// What "Back in" says of `left` seconds: the whole seconds it is in, never
/// none.
fn whole_seconds(left: f64) -> f32 {
    (left.max(0.25) as f32).ceil()
}

/// "Back in 6" low on the screen, in this game alone ([`BACK_DOWN`]): the
/// kit's own words, put up where it puts them, moved there in the frame
/// they are put up.
fn lower_death_words(stand: Option<Res<Stand>>, mut words: Query<&mut Node, With<arena::DeathText>>) {
    if stand.is_none() {
        return;
    }
    for mut node in &mut words {
        if node.top != Val::Vh(BACK_DOWN) {
            node.top = Val::Vh(BACK_DOWN);
        }
    }
}

/// What the camera looks on at in a game of Last Stand, if not at you
/// (`crate::Lookout`), as it stands at `now` by the app's own time: once
/// the game is over, the anchor's place, where the last stand fell (mine,
/// 2026-11-18: the end was said over empty stone at the rim); out of the
/// game, whoever of your team still stands, the nearest of them to the
/// anchor (`standing`), or with none, the anchor's place, to the end; dead
/// and to stand again, the anchor, where you will (mine, that day: the
/// camera stayed six seconds on the stone where you fell and then jumped);
/// and a moment as the crystal shatters off the screen, it ([`SHATTER_LOOK`](super::looks::SHATTER_LOOK)),
/// unless a shop's window is up over you.
fn looked_at(stand: &Stand, crystal: &Crystal, now: f32, standing: Option<Vec3>) -> Option<Vec3> {
    let round = stand.round.as_ref()?;
    let anchor = Vec3::new(rules::ANCHOR.x, rules::STONE_TOP, rules::ANCHOR.y);
    if round.stage == StandStage::Over {
        return Some(anchor);
    }
    if out(stand) {
        return Some(standing.unwrap_or(anchor));
    }
    let dead = round.health.get(stand.seat) == Some(&0);
    if dead || (crystal.shattering_unseen(now) && !stand.shopping()) {
        return Some(anchor);
    }
    None
}

/// The camera goes to what it looks on at ([`looked_at`]), and back to you
/// once there is nothing, easing all the way rather than jumping (until
/// 2026-11-18, out of the game, it jumped): the point it looks on at
/// brought [`EASE`] of the way there in a second, near enough, from where
/// it looked as it began, and let go once back at you.
#[allow(clippy::type_complexity)]
fn watch_view(
    time: Res<Time>,
    stand: Option<Res<Stand>>,
    view: Res<TableView>,
    crystal: Res<Crystal>,
    bodies: Query<(&Transform, &roundtown_net::Member), (With<bevy_replicon::prelude::Remote>, Without<Player>)>,
    me: Query<&Transform, With<Player>>,
    mut lookout: ResMut<Lookout>,
    mut eased: Local<Option<Vec3>>,
) {
    let Some(stand) = stand else {
        // (The game's end lets the camera go, `play::Fight::give_back`.)
        *eased = None;
        return;
    };
    let anchor = Vec3::new(rules::ANCHOR.x, rules::STONE_TOP, rules::ANCHOR.y);
    let standing = bodies
        .iter()
        .filter_map(|(place, member)| {
            let seat = view.members.iter().position(|seated| seated.id == member.id && seated.id != 0)?;
            stand.standing(seat).then_some(place.translation)
        })
        .min_by(|a, b| a.distance(anchor).total_cmp(&b.distance(anchor)));
    let target = looked_at(&stand, &crystal, time.elapsed_secs(), standing);
    let Ok(you) = me.single() else {
        return;
    };
    // Where you are, as far as the camera goes: never under the sand's top,
    // fallen into the void.
    let here = you.translation.with_y(you.translation.y.max(rules::SAND_TOP));
    let Some(from) = eased.or(target.map(|_| here)) else {
        return;
    };
    let goal = target.unwrap_or(here);
    let near = from + (goal - from) * ease_share(time.delta_secs());
    if target.is_none() && near.distance(goal) < 0.1 {
        *eased = None;
        lookout.set_if_neq(Lookout(None));
        return;
    }
    *eased = Some(near);
    lookout.set_if_neq(Lookout(Some(near)));
}

/// How much of the way to where the camera goes it comes in a frame `dt`
/// long ([`EASE`]).
fn ease_share(dt: f32) -> f32 {
    1.0 - (-EASE * dt.max(0.0)).exp()
}

/// What is new in how the game stands, said once as it comes: across the
/// screen, and in the chat. Coming into a game under way, what stands is
/// taken as said, and only what changes from then on is. What is said
/// across the screen is taken down as the results come up, whose
/// see-through window it would show through (a play-tester, 2026-11-18:
/// "The last stand fell at wave 4" across the results' title).
#[allow(clippy::too_many_arguments)]
fn say(
    stand: Option<ResMut<Stand>>,
    view: Res<TableView>,
    mut log: ResMut<chat::Log>,
    said_across: Query<Entity, With<Announced>>,
    broken_lines: Query<Entity, With<BrokenLine>>,
    results_up: Query<(), With<ResultsWindow>>,
    mut commands: Commands,
) {
    let Some(mut stand) = stand else {
        return;
    };
    // The results come: the team's score kept for the line under them,
    // from everyone's as the table has them.
    if let Some(results) = view.results() {
        let waves = results.total.unwrap_or(0);
        let team = rules::team_score(&results.scores, waves);
        if TeamScore::get() != Some((waves, team)) {
            TeamScore::keep(waves, &results.scores);
        }
    }
    if !results_up.is_empty() {
        for said in &said_across {
            commands.entity(said).try_despawn();
        }
    }
    let Some(round) = stand.round.clone() else {
        return;
    };
    let file = ui();
    let words = &file.words.last_stand;
    let outs: Vec<bool> = round.back_at.iter().map(|back| back.is_infinite()).collect();
    let gone: Vec<bool> = (0..round.health.len()).map(|seat| round.is_gone(seat)).collect();
    let elapsed = stand.elapsed;
    let me = stand.seat;
    let said = &mut stand.said;
    if !said.begun {
        said.begun = true;
        said.wave = round.wave;
        said.on = round.stage == StandStage::Wave;
        said.anchor = Some(round.anchor);
        said.broken = round.anchor == 0;
        said.health = round.health.clone();
        said.out = outs;
        said.gone = gone;
        said.over = round.stage == StandStage::Over;
        // The play just begun, not a game come back into.
        if round.stage != StandStage::Over && elapsed < 5.0 {
            hud::announce_stroked(&mut commands, &words.go, hud::HEADLINE, GO_SECS)
                .insert((InArena, Announced))
                .with_children(|lines| {
                    lines.spawn(hud::words(words.go_caption.clone(), hud::CAPTION, file.colours.text.with_alpha(0.0)));
                });
        }
        return;
    }
    let name = |seat: usize| view.members.get(seat).map(|member| member.name.clone()).unwrap_or_default();
    let ink = file.colours.chat.event;
    // Names in the team's colour, as what anyone types here is written
    // (`Fighting::teams`, `play::claim`).
    let theirs = file.colours.arena.team(TEAM);
    // A wave come: its number, and what is new in it under it.
    if round.stage == StandStage::Wave && (round.wave != said.wave || !said.on) {
        said.wave = round.wave;
        said.on = true;
        let new = new_in(round.wave);
        hud::announce_stroked(&mut commands, &fill(&words.wave, &[("wave", &round.wave.to_string())]), hud::HEADLINE, WAVE_SECS)
            .insert((InArena, Announced))
            .with_children(|lines| {
                if let Some(new) = new {
                    lines.spawn(hud::words(new, hud::CAPTION, file.colours.text.with_alpha(0.0)));
                }
            });
    }
    // A wave cleared.
    if matches!(round.stage, StandStage::Break { .. }) && said.on {
        said.on = false;
        let wave = round.wave.to_string();
        hud::announce_stroked(&mut commands, &fill(&words.cleared, &[("wave", &wave)]), hud::HEADLINE, CLEARED_SECS)
            .insert((InArena, Announced));
        if CLEARED_IN_CHAT {
            log.push(chat::spans(&fill(&words.cleared_line, &[("wave", &wave)]), ink, &[]));
        }
    }
    // The anchor broken: across the screen, in the gold of everything else
    // said there, with what it means under it (mine, since 2026-11-18: said
    // in Bridge Wars' line for its timetable, `arena::alert`, the game's
    // most important line was white on the pale stone by day and hardly
    // read); but not when the game is over with it, the last of you down as
    // it broke, when the end says it all (the line made in this frame would
    // not be there yet for the end to take down, and the two would stand
    // over each other).
    if round.anchor == 0 && !said.broken {
        said.broken = true;
        if round.stage != StandStage::Over {
            hud::announce_stroked(&mut commands, &words.anchor_broken, hud::HEADLINE, BROKEN_SECS)
                .insert((InArena, Announced, BrokenLine))
                .with_children(|lines| {
                    lines.spawn(hud::words(words.anchor_broken_caption.clone(), hud::CAPTION, file.colours.text.with_alpha(0.0)));
                });
        }
        log.push(chat::spans(&words.anchor_line, ink, &[]));
    }
    said.anchor = Some(round.anchor);
    // Whoever else is down, whoever else is out, and whoever has left: not
    // you, whom the screen tells ("Back in 6", and the window that says you
    // are out), and whose every death pushed what the others did out of
    // the chat's three lines (a play-tester, 2026-11-18; mine).
    let person = |seat: usize| seat != me && view.members.get(seat).is_some_and(|member| member.id != 0);
    for (seat, news) in news_of(said, &round, person) {
        let line = match news {
            News::Down => &words.down_line,
            News::Out => &words.out_chat,
            News::Left => &words.left_line,
        };
        log.push(chat::spans(line, ink, &[("name", &name(seat), theirs)]));
    }
    said.health.clone_from(&round.health);
    said.out = outs;
    said.gone = gone;
    // The end: over the anchor's line, if that is still up (the last of
    // you falling a moment after it broke), which goes for it.
    if round.stage == StandStage::Over && !said.over {
        said.over = true;
        for line in &broken_lines {
            commands.entity(line).try_despawn();
        }
        hud::announce_stroked(&mut commands, &fill(&words.fell, &[("wave", &round.wave.to_string())]), hud::HEADLINE, FELL_SECS)
            .insert((InArena, Announced));
    }
}

/// "The anchor is under attack!", a line over the middle of the screen,
/// as the anchor loses health while its crystal is off the screen (mine,
/// since 2026-11-18, as Bridge Wars says its own anchor is, with its words
/// and as often: a play-tester fought at the rim while the anchor went
/// from 1,500 to nothing with nothing on the screen saying so). No oftener
/// than every [`WARN_EVERY`] seconds of the fight, never while something
/// else is said across the screen, and only in a wave, with you on the
/// map and standing. The arrow round you points the way (`looks`).
#[allow(clippy::too_many_arguments)]
fn warn(
    stand: Option<Res<Stand>>,
    seen: Res<CrystalSeen>,
    me: Query<(&crate::island::Venue, Has<crate::Dead>), With<Player>>,
    said_across: Query<(), With<Announced>>,
    mut last: Local<(u32, Option<u16>, Option<f32>)>,
    mut commands: Commands,
) {
    let Some(stand) = stand else {
        *last = (0, None, None);
        return;
    };
    let Some(round) = stand.round.as_ref() else {
        return;
    };
    if last.0 != stand.fight {
        *last = (stand.fight, None, None);
    }
    let (_, was, said_at) = &mut *last;
    let hit = was.is_some_and(|was| round.anchor < was);
    *was = Some(round.anchor);
    let here = me.single().is_ok_and(|(venue, dead)| *venue == crate::island::Venue::LastStand && !dead);
    let due = said_at.is_none_or(|at| stand.elapsed - at >= WARN_EVERY);
    if !(hit && due && here && !seen.0 && round.anchor > 0 && round.stage == StandStage::Wave && said_across.is_empty()) {
        return;
    }
    *said_at = Some(stand.elapsed);
    let file = ui();
    let mut line = hud::announce_stroked(&mut commands, &file.words.last_stand.under_attack, WARN_TEXT, WARN_SECS);
    line.insert((
        InArena,
        Announced,
        Aloft,
        // High on the screen, over what is said across its middle.
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Vh(WARN_DOWN),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
    ));
}

/// While a shop's window is up over you, nothing this game says across the
/// screen is seen: it would show through the window's see-through dark
/// among its cards (a play-tester, 2026-11-18: "Go! Hold the anchor"
/// between the sword cards). It goes on fading meanwhile, and is seen
/// again, if it is still up, as the window goes.
fn quiet_in_the_shop(stand: Option<Res<Stand>>, mut said: Query<&mut Visibility, With<Announced>>) {
    let shown = if stand.is_some_and(|stand| stand.shopping()) {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut visibility in &mut said {
        visibility.set_if_neq(shown);
    }
}

/// What is said under "Wave 7" of what is new in it: the kinds that come
/// in it for the first time, and a giant, or giants, whenever they come.
fn new_in(wave: u16) -> Option<String> {
    let file = ui();
    let words = &file.words.last_stand;
    let giants = giants(wave);
    if giants > 1 {
        return Some(fill(&words.giants, &[("count", &giants.to_string())]));
    }
    if giants == 1 {
        return Some(words.a_giant.clone());
    }
    first_in(wave).into_iter().find_map(|kind| match kind {
        rules::Kind::Runner => Some(words.runners.clone()),
        rules::Kind::Brute => Some(words.brutes.clone()),
        rules::Kind::Spitter => Some(words.spitters.clone()),
        rules::Kind::Leaper => Some(words.leapers.clone()),
        rules::Kind::Walker | rules::Kind::Giant => None,
    })
}

/// What the chat says of someone at the table as it comes ([`news_of`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum News {
    /// Down, to stand again in a moment ("Mochi is down").
    Down,
    /// Down with the anchor broken: out for good ("Mochi is out").
    Out,
    /// Gone from the game for good ("Mochi has left").
    Left,
}

/// What is new of the people at the table (each seat `person` says is
/// one), between what was last said of them (`said`) and how the round
/// stands now: whoever is down, whoever is out, and whoever has left the
/// game, who is neither, though their health is none and they stand again
/// never, as someone out does (the round's `gone` tells them apart; until
/// 2026-11-18 someone who left was said to be down, or out). Nothing of
/// anyone out or leaving once the game is over, as everyone goes then.
fn news_of(said: &Said, round: &StandRound, person: impl Fn(usize) -> bool) -> Vec<(usize, News)> {
    let over = round.stage == StandStage::Over;
    let mut news = Vec::new();
    for seat in (0..round.health.len()).filter(|&seat| person(seat)) {
        if round.is_gone(seat) {
            if !over && !said.gone.get(seat).copied().unwrap_or(false) {
                news.push((seat, News::Left));
            }
            continue;
        }
        let was = said.health.get(seat).copied().unwrap_or(0);
        let now = round.health[seat];
        let out_now = round.back_at.get(seat).is_some_and(|back| back.is_infinite()) && round.anchor == 0;
        let out_was = said.out.get(seat).copied().unwrap_or(false);
        if was > 0 && now == 0 && !out_now {
            news.push((seat, News::Down));
        }
        if out_now && !out_was && !over {
            news.push((seat, News::Out));
        }
    }
    news
}

/// What the chat says of what happened that the round does not say: what
/// someone bought the team, and a giant brought down.
fn chat_lines(mut happened: MessageReader<Happened>, view: Res<TableView>, stand: Option<Res<Stand>>, mut log: ResMut<chat::Log>) {
    if stand.is_none() {
        happened.clear();
        return;
    }
    let file = ui();
    let words = &file.words.last_stand;
    let (ink, theirs) = (file.colours.chat.event, file.colours.arena.team(TEAM));
    let name = |seat: usize| view.members.get(seat).map(|member| member.name.clone()).filter(|name| !name.is_empty());
    for happening in happened.read() {
        match *happening {
            Happened::Upgraded { by, ware, level } => {
                let Some(who) = name(by) else {
                    continue;
                };
                let line = fill(&words.bought_line, &[("ware", &upgrade_name(ware, level)), ("level", &level.to_string())]);
                log.push(chat::spans(&line, ink, &[("name", &who, theirs)]));
            }
            Happened::Slain { by, .. } => {
                let Some(who) = name(by) else {
                    continue;
                };
                log.push(chat::spans(&words.slain_line, ink, &[("name", &who, theirs)]));
            }
            Happened::Zap(_) => {}
        }
    }
}

/// What an upgrade is called, the team having it to `level` now: the
/// generator's by its step (its fourth, overclocking, for every one past
/// the third).
fn upgrade_name(ware: Ware, level: u16) -> String {
    let words = &ui().words.last_stand;
    match ware {
        Ware::Generator => words.forge[usize::from(level.clamp(1, 4)) - 1].clone(),
        Ware::Sharpness => words.sharpness.clone(),
        Ware::Protection => words.protection.clone(),
        Ware::Fortify => words.fortify.clone(),
        Ware::Beam => words.beam.clone(),
        Ware::Lucky => words.lucky.clone(),
        Ware::Sword(_) | Ware::Shield(_) | Ware::Utility(_) => String::new(),
    }
}

// --------------------------------------------------------------- the shops

/// Everything the two shops sell, in the order of their cards, each card's
/// number its place here: the item shop's shelves, then the upgrade
/// shop's.
fn wares() -> impl Iterator<Item = Ware> {
    Ware::ITEMS.into_iter().flatten().copied().chain(Ware::UPGRADES)
}

/// The ware a card numbered `pick` sells ([`stalls`]), if there is one.
pub(super) fn ware_of(pick: u16) -> Option<Ware> {
    wares().nth(usize::from(pick))
}

/// The shops' windows, for someone holding `purse` with `kit` on a team
/// with `team`: the item shop's three shelves and the upgrade shop's one,
/// each card as it stands for them.
pub(super) fn stalls(purse: &Purse, kit: &Kit, team: &StandUpgrades) -> Stalls {
    let words = &ui().words.last_stand;
    let mut number = 0u16;
    let mut card = |ware: Ware| {
        let offer = offer(ware, number, purse, kit, team);
        number += 1;
        offer
    };
    let items = Ware::ITEMS.iter().map(|shelf| shelf.iter().map(|&ware| card(ware)).collect()).collect();
    let upgrades = vec![Ware::UPGRADES.iter().map(|&ware| card(ware)).collect()];
    Stalls {
        venue: Some(Venue::LastStand),
        items: Stall {
            title: words.items_title.clone(),
            shelves: items,
        },
        upgrades: Stall {
            title: words.upgrades_title.clone(),
            shelves: upgrades,
        },
    }
}

/// The card for `ware`, numbered `pick`, for someone holding `purse` with
/// `kit` on a team with `team`: its picture, name and line, out of the
/// file (Bridge Wars' own for its swords, shields and things to use), its
/// numbers out of the rules.
fn offer(ware: Ware, pick: u16, purse: &Purse, kit: &Kit, team: &StandUpgrades) -> Offer {
    let file = ui();
    let (market, words, colours) = (&file.words.market, &file.words.last_stand, file.colours);
    let whole = |number: f32| format!("{number:.0}");
    let percent = |share: f32| format!("{:.0}", share * 100.0);
    let tier = |tier: u8| usize::from(tier.clamp(1, 3));
    let level = |level: u16| fill(&words.level, &[("level", &(u32::from(level) + 1).to_string())]);
    let (picture, tint, name, stat, pips) = match ware {
        Ware::Sword(sword) => (
            Picture::Sword,
            colours.arena.sword(sword),
            market.swords[tier(sword) - 1].clone(),
            fill(&words.sword_stat, &[("damage", &whole(SWORD_DAMAGE[tier(sword)]))]),
            None,
        ),
        Ware::Shield(shield) => (
            Picture::Shield,
            colours.arena.shield(shield).with_alpha(1.0),
            market.shields[tier(shield) - 1].clone(),
            fill(&market.shield_stat, &[("share", &percent(SHIELD_SHARE[tier(shield)]))]),
            None,
        ),
        Ware::Utility(utility) => {
            let (picture, tint) = market::look_of(utility);
            let (name, stat) = match utility {
                Utility::Healing => (&market.healing, fill(&market.healing_stat, &[("share", &percent(HEALING))])),
                Utility::Speed => (
                    &market.speed,
                    fill(&market.speed_stat, &[("more", &percent(FAST - 1.0)), ("seconds", &whole(SPEED_SECS))]),
                ),
                Utility::Fireball => (&market.fireball, fill(&market.fireball_stat, &[("damage", &whole(FIREBALL_HIT))])),
                Utility::Dynamite => (&market.dynamite, fill(&words.dynamite_stat, &[("damage", &whole(DYNAMITE_HIT))])),
                Utility::Invisibility => (&market.invisibility, fill(&market.invisibility_stat, &[("seconds", &whole(UNSEEN_SECS))])),
                Utility::Leap => (
                    &market.leap,
                    fill(&market.leap_stat, &[("times", &whole(LEAP)), ("seconds", &whole(LEAP_SECS))]),
                ),
                Utility::Builder => (&market.builder, String::new()),
            };
            (picture, tint, name.clone(), stat, None)
        }
        Ware::Generator => {
            // The next step, and what it brings: each mineral in turn, and
            // past them, faster (the level it takes the overclocking to).
            let step = team.generator;
            let named = usize::from(step.min(3));
            let (tint, stat) = match step {
                0 => (colours.arena.item(Item::Gold), words.forge_stats[0].clone()),
                1 => (colours.arena.item(Item::Sapphire), words.forge_stats[1].clone()),
                2 => (colours.arena.item(Item::Amethyst), words.forge_stats[2].clone()),
                // The line's level it takes the team to, as the pads say it
                // and the chat says it bought.
                _ => (colours.arena.fireball, fill(&words.forge_stats[3], &[("level", &(step + 1).to_string())])),
            };
            (Picture::Forge, tint, words.forge[named].clone(), stat, None)
        }
        Ware::Sharpness => (Picture::SwordPlus, colours.hud.icon, words.sharpness.clone(), level(team.sharpness), None),
        Ware::Protection => (
            Picture::ShieldPlus,
            colours.hud.icon,
            words.protection.clone(),
            fill(&words.protection_stat, &[("share", &percent(PROTECTION))]),
            Some((team.protection, PROTECTION_MOST)),
        ),
        Ware::Fortify => (Picture::Crystal, colours.last_stand.anchor.with_alpha(1.0), words.fortify.clone(), level(team.fortify), None),
        Ware::Beam => (Picture::Beam, colours.last_stand.beam, words.beam.clone(), level(team.beam), None),
        Ware::Lucky => (
            Picture::Clover,
            colours.last_stand.spit,
            words.lucky.clone(),
            words.lucky_stat.clone(),
            Some((team.lucky, LUCKY_MOST)),
        ),
    };
    let state = match price(ware, kit, team) {
        Some((item, cost)) => State::Buy {
            item,
            cost,
            able: purse[item.index()] >= cost,
        },
        None => match ware {
            Ware::Sword(_) | Ware::Shield(_) => State::Owned,
            Ware::Utility(utility) if kit.slot_of(utility).is_none() => State::Taken,
            _ => State::Max,
        },
    };
    let held = match ware {
        Ware::Utility(utility) => kit.held(utility),
        _ => 0,
    };
    Offer {
        pick: Pick::Own(pick),
        picture,
        tint,
        name,
        stat,
        pips,
        held,
        state,
    }
}

#[cfg(test)]
mod tests {
    use roundtown_net::bridge_wars::Shop;

    use super::*;

    #[test]
    fn every_card_asks_for_what_it_shows_and_is_priced_as_the_rules_have_it() {
        let purse: Purse = [30, 10, 6, 1];
        let kit = Kit::default();
        let team = StandUpgrades::default();
        let shown = stalls(&purse, &kit, &team);
        assert_eq!(shown.venue, Some(Venue::LastStand));
        // Swords, shields and the six things to use; the six upgrades.
        let shelves: Vec<usize> = shown.items.shelves.iter().map(Vec::len).collect();
        assert_eq!(shelves, [3, 3, 6]);
        assert_eq!(shown.upgrades.shelves.iter().map(Vec::len).collect::<Vec<_>>(), [6]);
        let cards: Vec<&Offer> = shown.items.shelves.iter().chain(&shown.upgrades.shelves).flatten().collect();
        for card in &cards {
            let Pick::Own(pick) = card.pick else {
                panic!("a card of Bridge Wars' own");
            };
            let ware = ware_of(pick).expect("a ware");
            match (card.state, price(ware, &kit, &team)) {
                (State::Buy { item, cost, able }, Some((priced, costs))) => {
                    assert_eq!((item, cost), (priced, costs), "{ware:?}");
                    assert_eq!(able, purse[item.index()] >= cost, "{ware:?}");
                }
                (state, priced) => panic!("{ware:?}: {state:?} for {priced:?}"),
            }
            assert!(!card.name.is_empty(), "{ware:?}");
            assert_eq!(rules::sold_at(ware) == Shop::Upgrades, shown.upgrades.shelves[0].iter().any(|up| up.pick == card.pick));
        }
        assert_eq!(ware_of(18), None);
        // An endless line's card says the level it takes the team to; a
        // capped one's, its pips.
        let team = StandUpgrades {
            sharpness: 6,
            protection: 2,
            generator: 5,
            ..StandUpgrades::default()
        };
        let shown = stalls(&purse, &kit, &team);
        let upgrade = |ware: Ware| shown.upgrades.shelves[0].iter().find(|card| ware_of(match card.pick {
            Pick::Own(pick) => pick,
            Pick::Ware(_) => u16::MAX,
        }) == Some(ware)).expect("its card");
        assert_eq!(upgrade(Ware::Sharpness).stat, "Next: Lvl 7");
        assert_eq!(upgrade(Ware::Protection).pips, Some((2, 5)));
        assert_eq!(
            (upgrade(Ware::Generator).name.as_str(), upgrade(Ware::Generator).stat.as_str()),
            ("Overclock", "Next: Lvl 6")
        );
        // A sword no better than yours is yours already; a thing to use
        // with every button full of others is not to be had.
        let mut kit = Kit::default();
        kit.sword = 2;
        let shown = stalls_for(&kit);
        assert_eq!(shown.items.shelves[0][0].state, State::Owned);
        assert!(matches!(shown.items.shelves[0][2].state, State::Buy { .. }));
    }

    fn stalls_for(kit: &Kit) -> Stalls {
        stalls(&[99; 4], kit, &StandUpgrades::default())
    }

    /// The anchor's line across the screen stands until the game is over,
    /// which takes it down for its own; and when the anchor breaks in the
    /// very step that ends the game, the last of you down as it broke, it
    /// is never put up, the end alone saying so. As the results come up,
    /// the end goes too.
    #[test]
    fn the_anchor_s_line_never_stands_under_the_end_nor_the_end_under_the_results() {
        let round = |stage: StandStage, anchor: u16, health: u16| StandRound {
            wave: 9,
            stage,
            anchor,
            anchor_most: 1500,
            left: 12,
            health: vec![health],
            back_at: vec![if health == 0 { f64::INFINITY } else { 0.0 }],
            scores: vec![40],
            ready: 0,
            gone: 0,
            began: 0.0,
        };
        let alerts = |app: &mut App| app.world_mut().query_filtered::<(), With<BrokenLine>>().iter(app.world()).count();
        let said = |app: &mut App| app.world_mut().query_filtered::<(), With<Announced>>().iter(app.world()).count();
        for together in [false, true] {
            let mut app = App::new();
            app.init_resource::<chat::Log>().init_resource::<TableView>().add_systems(Update, say);
            let mut stand = Stand::new(false, 0, 1);
            stand.elapsed = 300.0;
            stand.round = Some(round(StandStage::Wave, 200, 60));
            app.insert_resource(stand);
            app.update();
            assert_eq!(alerts(&mut app), 0);
            if !together {
                // Broken, and you still standing: said across the screen.
                app.world_mut().resource_mut::<Stand>().round = Some(round(StandStage::Wave, 0, 60));
                app.update();
                assert_eq!(alerts(&mut app), 1);
            }
            app.world_mut().resource_mut::<Stand>().round = Some(round(StandStage::Over, 0, 0));
            app.update();
            assert_eq!(alerts(&mut app), 0, "the anchor broken {}", if together { "as it ended" } else { "before" });
            // The end said; and gone as the results' window comes up.
            assert_eq!(said(&mut app), 1);
            app.world_mut().spawn(ResultsWindow);
            app.update();
            assert_eq!(said(&mut app), 0, "the end under the results");
        }
    }

    /// A round of two in a wave: health and when each stands again by seat,
    /// the anchor's health, and who has gone.
    fn of_two(health: [u16; 2], back_at: [f64; 2], anchor: u16, gone: u8) -> StandRound {
        StandRound {
            wave: 3,
            stage: StandStage::Wave,
            anchor,
            anchor_most: 1500,
            left: 5,
            health: health.to_vec(),
            back_at: back_at.to_vec(),
            scores: vec![0, 0],
            ready: 0,
            gone,
            began: 0.0,
        }
    }

    /// What was said of everyone, as of `round`.
    fn said_of(round: &StandRound) -> Said {
        Said {
            begun: true,
            health: round.health.clone(),
            out: round.back_at.iter().map(|back| back.is_infinite()).collect(),
            gone: (0..round.health.len()).map(|seat| round.is_gone(seat)).collect(),
            ..Said::default()
        }
    }

    #[test]
    fn someone_who_leaves_is_said_to_have_left_and_never_to_be_down_or_out() {
        let everyone = |_: usize| true;
        let never = f64::INFINITY;
        let before = of_two([80, 60], [0.0, 0.0], 900, 0);
        let said = said_of(&before);
        // Down, to stand again in a moment; and out, down with the anchor
        // broken.
        assert_eq!(news_of(&said, &of_two([80, 0], [0.0, 1.8e9], 900, 0), everyone), [(1, News::Down)]);
        assert_eq!(news_of(&said, &of_two([80, 0], [0.0, never], 0, 0), everyone), [(1, News::Out)]);
        // Gone, the anchor standing or broken: neither, as their health
        // none and their back never alone would have it.
        assert_eq!(news_of(&said, &of_two([80, 0], [0.0, never], 900, 0b10), everyone), [(1, News::Left)]);
        assert_eq!(news_of(&said, &of_two([80, 0], [0.0, never], 0, 0b10), everyone), [(1, News::Left)]);
        // Down first, and then gone: each once.
        let down = of_two([80, 0], [0.0, 1.8e9], 900, 0);
        let left = of_two([80, 0], [0.0, never], 900, 0b10);
        assert_eq!(news_of(&said_of(&down), &left, everyone), [(1, News::Left)]);
        assert!(news_of(&said_of(&left), &left, everyone).is_empty());
        // Nothing of anyone leaving once it is over, nor of a seat no
        // person sat in.
        let over = StandRound {
            stage: StandStage::Over,
            ..of_two([0, 0], [never, never], 0, 0b10)
        };
        assert!(news_of(&said_of(&of_two([0, 0], [never, never], 0, 0)), &over, everyone).is_empty());
        assert!(news_of(&said, &of_two([80, 0], [0.0, 1.8e9], 900, 0), |seat| seat == 0).is_empty());
    }

    #[test]
    fn the_words_while_you_are_dead_are_bridge_wars_own_with_no_game_of_last_stand_here() {
        let mut app = App::new();
        app.init_resource::<TownClock>().add_systems(Update, death_words);
        let up = |app: &mut App| {
            app.world_mut()
                .query_filtered::<(), With<arena::DeathText>>()
                .iter(app.world())
                .count()
        };
        // Bridge Wars' "You died. Respawning in 4...", put up by its own
        // system: left up, every frame.
        let theirs = app.world_mut().spawn((arena::DeathText, InArena)).id();
        for _ in 0..4 {
            app.update();
            assert_eq!(up(&mut app), 1, "Bridge Wars' words taken down");
        }
        app.world_mut().despawn(theirs);
        // Dead in a game of Last Stand: "Back in", up; the game gone, down.
        let now = app.world().resource::<TownClock>().now();
        let mut stand = Stand::new(false, 0, 1);
        stand.round = Some(StandRound {
            stage: StandStage::Wave,
            ..of_two([0, 70], [now + 6.0, 0.0], 900, 0)
        });
        app.insert_resource(stand);
        app.update();
        app.update();
        assert_eq!(up(&mut app), 1, "Back in 6");
        app.world_mut().remove_resource::<Stand>();
        app.update();
        app.update();
        assert_eq!(up(&mut app), 0, "taken down with the game");
        // And Bridge Wars' left alone again.
        app.world_mut().spawn((arena::DeathText, InArena));
        for _ in 0..3 {
            app.update();
            assert_eq!(up(&mut app), 1);
        }
    }

    #[test]
    fn r_is_ready_but_not_while_a_line_is_typed() {
        #[derive(Resource, Default)]
        struct Asked(usize);
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<hud::DialogUp>()
            .init_resource::<Asked>()
            .add_message::<ReadyAsked>()
            .add_systems(
                Update,
                (ready_key, |mut asked: MessageReader<ReadyAsked>, mut count: ResMut<Asked>| count.0 += asked.read().count()).chain(),
            );
        let mut stand = Stand::new(false, 0, 1);
        stand.round = Some(StandRound {
            stage: StandStage::Break { until: 2e9 },
            ..of_two([100, 100], [0.0, 0.0], 900, 0)
        });
        app.insert_resource(stand);
        let press = |app: &mut App| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyR);
            keys.clear();
            keys.press(KeyCode::KeyR);
            app.update();
            app.world().resource::<Asked>().0
        };
        // Typing in the chat: "r" is a letter of the line.
        app.world_mut().resource_mut::<hud::DialogUp>().0 = true;
        assert_eq!(press(&mut app), 0);
        app.world_mut().resource_mut::<hud::DialogUp>().0 = false;
        assert_eq!(press(&mut app), 1);
    }

    #[test]
    fn under_the_plate_between_waves_where_to_buy_what_comes_next_and_get_ready() {
        let now = 1000.0;
        let break_of = |wave: u16, left: f64| {
            let mut stand = Stand::new(false, 0, 1);
            stand.round = Some(StandRound {
                wave,
                stage: StandStage::Break { until: now + left },
                ..of_two([100, 100], [0.0, 0.0], 900, 0)
            });
            plate_words(&stand, now)
        };
        // Before the first wave, which stand sells what; at the last, get
        // ready.
        assert_eq!(break_of(0, 18.0), ("Next wave in 0:18".to_string(), ui().words.last_stand.tip.clone()));
        assert_eq!(break_of(0, 4.0).1, "Get ready");
        // Before a wave with something new in it, what; else, the wave
        // cleared.
        assert_eq!(break_of(4, 20.0).1, "Next: A giant!");
        assert_eq!(break_of(7, 20.0).1, "Next: Spitters!");
        assert_eq!(break_of(5, 20.0).1, "Wave 5 cleared");
        // Someone ready says so first.
        let mut stand = Stand::new(false, 0, 1);
        stand.round = Some(StandRound {
            wave: 4,
            stage: StandStage::Break { until: now + 20.0 },
            ready: 0b1,
            ..of_two([100, 100], [0.0, 0.0], 900, 0)
        });
        assert_eq!(plate_words(&stand, now).1, "1/2 ready");
        // And the anchor's number in its bar.
        assert_eq!(anchor_number(517, 1500), "517/1500");
    }

    #[test]
    fn the_chat_says_who_else_is_down_and_out_and_never_you() {
        let before = of_two([80, 60], [0.0, 0.0], 900, 0);
        let both_down = of_two([0, 0], [1.8e9, 1.8e9], 900, 0);
        let both_out = of_two([0, 0], [f64::INFINITY, f64::INFINITY], 0, 0);
        // You in seat 0: the chat, as the game asks it (`say`), of the
        // other alone.
        let others = |seat: usize| seat != 0;
        assert_eq!(news_of(&said_of(&before), &both_down, others), [(1, News::Down)]);
        assert_eq!(news_of(&said_of(&both_down), &both_out, others), [(1, News::Out)]);
    }

    #[test]
    fn back_in_whole_seconds_and_never_none() {
        assert_eq!(whole_seconds(5.3), 6.0);
        assert_eq!(whole_seconds(6.0), 6.0);
        assert_eq!(whole_seconds(0.1), 1.0);
        assert_eq!(whole_seconds(-0.4), 1.0);
    }

    #[test]
    fn the_camera_looks_on_at_the_anchor_dead_out_alone_at_the_end_and_as_it_shatters_unseen() {
        let anchor = Vec3::new(rules::ANCHOR.x, rules::STONE_TOP, rules::ANCHOR.y);
        let crystal = Crystal::default();
        let with = |health: [u16; 2], back_at: [f64; 2], anchor_left: u16, stage: StandStage| {
            let mut stand = Stand::new(false, 0, 1);
            stand.round = Some(StandRound {
                stage,
                ..of_two(health, back_at, anchor_left, 0)
            });
            stand
        };
        let mate = Vec3::new(4.0, rules::STONE_TOP, 9.0);
        // Standing: at you.
        assert_eq!(looked_at(&with([90, 90], [0.0, 0.0], 900, StandStage::Wave), &crystal, 10.0, Some(mate)), None);
        // Dead, to stand again: the anchor.
        assert_eq!(looked_at(&with([0, 90], [1.8e9, 0.0], 900, StandStage::Wave), &crystal, 10.0, Some(mate)), Some(anchor));
        // Out: whoever still stands, else the anchor's place.
        let out = with([0, 90], [f64::INFINITY, 0.0], 0, StandStage::Wave);
        assert_eq!(looked_at(&out, &crystal, 10.0, Some(mate)), Some(mate));
        assert_eq!(looked_at(&out, &crystal, 10.0, None), Some(anchor));
        // Over: the anchor's place, where the last stand fell.
        assert_eq!(
            looked_at(&with([0, 0], [f64::INFINITY, f64::INFINITY], 0, StandStage::Over), &crystal, 10.0, Some(mate)),
            Some(anchor)
        );
        // Easing: some of the way in a frame, all of it in no time.
        assert!(ease_share(1.0 / 60.0) > 0.05 && ease_share(1.0 / 60.0) < 0.2);
        assert!(ease_share(0.5) > 0.9 && ease_share(0.0) == 0.0);
    }

    #[test]
    fn what_is_new_in_a_wave_is_said_under_it() {
        assert_eq!(new_in(1), None);
        assert_eq!(new_in(3).as_deref(), Some("Runners!"));
        assert_eq!(new_in(4).as_deref(), Some("Brutes!"));
        assert_eq!(new_in(5).as_deref(), Some("A giant!"));
        assert_eq!(new_in(8).as_deref(), Some("Spitters!"));
        assert_eq!(new_in(12).as_deref(), Some("Leapers!"));
        assert_eq!(new_in(15).as_deref(), Some("2 giants!"));
    }
}
