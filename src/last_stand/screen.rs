//! What a game of Last Stand says on the screen, the same online and
//! offline, from how the game stands ([`Stand`]):
//!
//! * **The plate top centre** ([`show_plate`]), on the plate's grey as the
//!   table's board is, which is put away for the play: in a wave, "Wave 7"
//!   over "12 left"; between waves, "Next wave in 0:18" over what came
//!   before it ("Wave 7 cleared", or before the first "Get ready"), or how
//!   many of you are ready; inside it, under its big line, the anchor's
//!   health, a bar; and beside it, between waves, a Ready button (R on
//!   desktop): the next wave comes three seconds after everyone standing
//!   is ready (the rules' `READY_SECS`).
//! * **Across the screen**, once each as it comes ([`say`]): "Go!" as the
//!   play begins; "Wave 7" as one comes, with what is new in it under it
//!   ("Brutes!", "A giant!"); "Wave 7 cleared"; "The anchor is broken: no
//!   more coming back" (Bridge Wars' line for what its timetable does,
//!   `arena::alert`); and at the end "The last stand fell at wave 9".
//! * **In the chat**: what someone bought the team ("Mochi bought
//!   Sharpness Lvl 3"), who is down and who is out, a giant brought down,
//!   the anchor broken, a wave cleared; names in the team's colour, as
//!   what anyone types here is written.
//! * **You**: dead, "Back in 6", counting down, the kit's own words
//!   (`arena::DeathWords`); out, with the anchor broken, a small window
//!   saying so with a button to leave, and the camera on whoever of your
//!   team still stands, or on the anchor's place, to the end
//!   ([`out_view`]).
//! * **The shops' cards** ([`stalls`]): the item shop's swords, shields
//!   and things to use, by Bridge Wars' own pictures and names at Last
//!   Stand's prices; the upgrade shop's six lines, an endless one's card
//!   saying the level it takes the team to ("Lvl 7") over its price.

use bevy::prelude::*;
use roundtown_net::bridge_wars::{FAST, HEALING, LEAP};
use roundtown_net::last_stand::{
    self as rules, DYNAMITE_HIT, FIREBALL_HIT, LEAP_SECS, LUCKY_MOST, PROTECTION, PROTECTION_MOST, SHIELD_SHARE, SPEED_SECS,
    SWORD_DAMAGE, StandStage, StandUpgrades, UNSEEN_SECS, Ware, first_in, giants, price,
};
use roundtown_net::{Item, Kit, Purse, Utility, Venue};

use super::{Happened, ReadyAsked, Stand, StandSystems, TEAM, TeamScore};
use crate::arena::{self, DeathWords, InArena, Line};
use crate::chat;
use crate::clock::TownClock;
use crate::hud::{self, Picture};
use crate::market::{self, Offer, Pick, State, Stall, Stalls};
use crate::table::{self, InTable, TableView};
use crate::tags::HidesNames;
use crate::ui::{UiChanged, fill, ui};
use crate::window::{self, Look};
use crate::{Lookout, Player};

/// Sizes, as shares of the short side of the screen: the plate's big line
/// and the line under it, as the table's board's are; the anchor's bar
/// under them; the window that says you are out, and the words on it.
const PLATE_TEXT: f32 = 4.8;
const PLATE_CAPTION: f32 = 3.4;
const PLATE_BAR_WIDTH: f32 = 24.0;
const PLATE_BAR_HEIGHT: f32 = 1.1;
const READY_GAP: f32 = 1.8;
const OUT_WIDTH: f32 = 50.0;
/// How far down the screen the window that says you are out begins, as a
/// share of the screen's height.
const OUT_DOWN: f32 = 62.0;
/// How long each thing said across the screen is up, in seconds: the
/// game's beginning, a wave's coming, a wave cleared and the end.
const GO_SECS: f32 = 2.5;
const WAVE_SECS: f32 = 2.6;
const CLEARED_SECS: f32 = 2.4;
const FELL_SECS: f32 = 4.0;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(tap).add_systems(
        Update,
        (
            (spawn_screen, ready_key, say, chat_lines).chain().after(StandSystems),
            (show_plate, show_out, death_words, out_view).after(StandSystems).after(crate::follow_camera),
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
/// The small window that says you are out.
#[derive(Component)]
struct OutPane;

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
                    plate_box.spawn((
                        Node {
                            width: Val::VMin(PLATE_BAR_WIDTH),
                            height: Val::VMin(PLATE_BAR_HEIGHT),
                            border: UiRect::all(Val::VMin(0.15)),
                            border_radius: BorderRadius::all(Val::VMin(0.5)),
                            ..default()
                        },
                        BackgroundColor(colours.arena.hp_back),
                        BorderColor::all(colours.arena.hp_rim),
                        Pickable::IGNORE,
                        children![(
                            PlateFill,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                border_radius: BorderRadius::all(Val::VMin(0.3)),
                                ..default()
                            },
                            BackgroundColor(colours.last_stand.anchor),
                            Pickable::IGNORE,
                        )],
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
            // Low on the screen, clear of the middle, where whoever you
            // watch is.
            window::pane(Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Vh(OUT_DOWN),
                width: Val::VMin(OUT_WIDTH),
                padding: UiRect::all(Val::VMin(window::PAD)),
                align_items: AlignItems::Center,
                row_gap: Val::VMin(1.6),
                ..default()
            }),
            UiTransform::from_translation(Val2::new(Val::Percent(-50.0), Val::ZERO)),
            Visibility::Hidden,
        ))
        .with_children(|pane| {
            pane.spawn(window::words(words.out.clone(), window::TITLE, colours.window.title));
            pane.spawn(window::words(words.out_line.clone(), window::NAME, colours.window.status));
            pane.spawn((StandTap::Leave, window::choice(ui.words.table.leave.clone(), Look::No)));
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

/// R on desktop is Ready, between waves.
fn ready_key(stand: Option<Res<Stand>>, keyboard: Res<ButtonInput<KeyCode>>, mut ready: MessageWriter<ReadyAsked>) {
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

/// What the plate says, as the game stands at the town's clock `now`: its
/// big line and the line under it.
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
            } else if round.wave == 0 {
                words.get_ready.clone()
            } else {
                fill(&words.cleared_caption, &[("wave", &wave)])
            };
            (fill(&words.next_wave, &[("time", &time)]), under)
        }
        StandStage::Over => (fill(&words.wave, &[("wave", &wave)]), String::new()),
    }
}

/// Shows the plate while the game is played and over (the table's board is
/// up before), and writes on it what it says, only as that changes: its
/// lines, the anchor's bar, and the Ready button between waves.
#[allow(clippy::type_complexity)]
fn show_plate(
    stand: Option<Res<Stand>>,
    view: Res<TableView>,
    town: Res<TownClock>,
    mut plates: Query<&mut Visibility, (With<Plate>, Without<StandTap>)>,
    mut texts: Query<&mut Text, (With<PlateText>, Without<PlateCaption>)>,
    mut captions: Query<&mut Text, (With<PlateCaption>, Without<PlateText>)>,
    mut fills: Query<&mut Node, With<PlateFill>>,
    mut buttons: Query<(&StandTap, &mut Visibility), Without<Plate>>,
    mut caption_shown: Query<&mut Visibility, (With<PlateCaption>, Without<Plate>, Without<StandTap>)>,
    (shops, me): (Res<market::Shops>, Query<&Transform, With<Player>>),
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
        for mut fill in &mut fills {
            if fill.width != width {
                fill.width = width;
            }
        }
    }
    // Standing in a shop's outline, its window comes down over the line
    // under the plate (`market`): the line is put away meanwhile.
    let shopping = me.single().is_ok_and(|you| shops.at(Venue::LastStand, you.translation).is_some());
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
/// the town's clock, the kit's own words across the screen
/// (`arena::DeathWords`).
fn death_words(stand: Option<Res<Stand>>, town: Res<TownClock>, mut words: DeathWords, mut commands: Commands) {
    let left = stand.as_deref().and_then(|stand| {
        let round = stand.round.as_ref()?;
        (round.health.get(stand.seat) == Some(&0) && round.stage != StandStage::Over)
            .then(|| round.back_at.get(stand.seat).copied())
            .flatten()
            .filter(|back| back.is_finite())
            // (Never "Back in 0": the game stands you again a moment after
            // the clock says, a frame behind it, or more on a slow one.)
            .map(|back| (back - town.now()).max(0.25) as f32)
    });
    if left.is_none() && !words.up() {
        return;
    }
    words.say(&mut commands, &ui().words.last_stand.back_in, left);
}

/// Out of the game, the camera stays on whoever of your team still stands,
/// the nearest of them to the anchor, or with none, on the anchor's place
/// (`crate::Lookout`), to the end; and goes back to you once you are not.
fn out_view(
    stand: Option<Res<Stand>>,
    view: Res<TableView>,
    bodies: Query<(&Transform, &roundtown_net::Member), (With<bevy_replicon::prelude::Remote>, Without<Player>)>,
    mut lookout: ResMut<Lookout>,
    mut looking: Local<bool>,
) {
    let watching = stand.as_deref().filter(|stand| out(stand));
    let Some(stand) = watching else {
        if std::mem::take(&mut *looking) {
            lookout.set_if_neq(Lookout(None));
        }
        return;
    };
    *looking = true;
    let anchor = Vec3::new(rules::ANCHOR.x, rules::STONE_TOP, rules::ANCHOR.y);
    let standing = bodies
        .iter()
        .filter_map(|(place, member)| {
            let seat = view.members.iter().position(|seated| seated.id == member.id && seated.id != 0)?;
            stand.standing(seat).then_some(place.translation)
        })
        .min_by(|a, b| a.distance(anchor).total_cmp(&b.distance(anchor)));
    lookout.set_if_neq(Lookout(Some(standing.unwrap_or(anchor))));
}

/// What is new in how the game stands, said once as it comes: across the
/// screen, and in the chat. Coming into a game under way, what stands is
/// taken as said, and only what changes from then on is.
#[allow(clippy::too_many_arguments)]
fn say(
    stand: Option<ResMut<Stand>>,
    view: Res<TableView>,
    mut images: ResMut<Assets<Image>>,
    mut log: ResMut<chat::Log>,
    alerts: Query<Entity, With<arena::Alert>>,
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
    let Some(round) = stand.round.clone() else {
        return;
    };
    let file = ui();
    let words = &file.words.last_stand;
    let outs: Vec<bool> = round.back_at.iter().map(|back| back.is_infinite()).collect();
    let elapsed = stand.elapsed;
    let said = &mut stand.said;
    if !said.begun {
        said.begun = true;
        said.wave = round.wave;
        said.on = round.stage == StandStage::Wave;
        said.anchor = Some(round.anchor);
        said.broken = round.anchor == 0;
        said.health = round.health.clone();
        said.out = outs;
        said.over = round.stage == StandStage::Over;
        // The play just begun, not a game come back into.
        if round.stage != StandStage::Over && elapsed < 5.0 {
            hud::announce(&mut commands, None, &words.go, hud::HEADLINE, GO_SECS)
                .insert(InArena)
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
        hud::announce(&mut commands, None, &fill(&words.wave, &[("wave", &round.wave.to_string())]), hud::HEADLINE, WAVE_SECS)
            .insert(InArena)
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
        hud::announce(&mut commands, None, &fill(&words.cleared, &[("wave", &wave)]), hud::HEADLINE, CLEARED_SECS).insert(InArena);
        log.push(chat::spans(&fill(&words.cleared_line, &[("wave", &wave)]), ink, &[]));
    }
    // The anchor broken: across the screen, but for when the game is over
    // with it, the last of you down as it broke, when the end says it all
    // (the line made in this frame would not be there yet for the end to
    // take down, and the two would stand over each other).
    if round.anchor == 0 && !said.broken {
        said.broken = true;
        if round.stage != StandStage::Over {
            arena::alert(&mut commands, &mut images, &Line::plain(&words.anchor_broken));
        }
        log.push(chat::spans(&words.anchor_line, ink, &[]));
    }
    said.anchor = Some(round.anchor);
    // Whoever is down, and whoever is out.
    for seat in 0..round.health.len() {
        let was = said.health.get(seat).copied().unwrap_or(0);
        let now = round.health[seat];
        let out_now = outs.get(seat).copied().unwrap_or(false) && round.anchor == 0;
        let out_was = said.out.get(seat).copied().unwrap_or(false);
        let person = view.members.get(seat).is_some_and(|member| member.id != 0);
        if person && was > 0 && now == 0 && !out_now {
            log.push(chat::spans(&words.down_line, ink, &[("name", &name(seat), theirs)]));
        }
        if person && out_now && !out_was && round.stage != StandStage::Over {
            log.push(chat::spans(&words.out_chat, ink, &[("name", &name(seat), theirs)]));
        }
    }
    said.health.clone_from(&round.health);
    said.out = outs;
    // The end: over the anchor's line, if that is still up (the last of
    // you falling a moment after it broke), which goes for it.
    if round.stage == StandStage::Over && !said.over {
        said.over = true;
        for alert in &alerts {
            commands.entity(alert).try_despawn();
        }
        hud::announce(&mut commands, None, &fill(&words.fell, &[("wave", &round.wave.to_string())]), hud::HEADLINE, FELL_SECS)
            .insert(InArena);
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
    use roundtown_net::last_stand::StandRound;

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
        assert_eq!(upgrade(Ware::Sharpness).stat, "Lvl 7");
        assert_eq!(upgrade(Ware::Protection).pips, Some((2, 5)));
        assert_eq!((upgrade(Ware::Generator).name.as_str(), upgrade(Ware::Generator).stat.as_str()), ("Overclock", "Lvl 6"));
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
    /// is never put up, the end alone saying so.
    #[test]
    fn the_anchor_s_line_never_stands_under_the_end() {
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
            began: 0.0,
        };
        let alerts = |app: &mut App| app.world_mut().query_filtered::<(), With<arena::Alert>>().iter(app.world()).count();
        for together in [false, true] {
            let mut app = App::new();
            app.init_resource::<Assets<Image>>()
                .init_resource::<chat::Log>()
                .init_resource::<TableView>()
                .add_systems(Update, say);
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
        }
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
