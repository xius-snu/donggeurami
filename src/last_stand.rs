//! Last Stand, on this device (since 2026-11-18; Hajun: "Coop (2~4 players)
//! Last Stand (Endless Zombie survival game). You are in a circular map,
//! with a spawn anchor crystal in the center, and 2 generators and item
//! shops/upgrade shops. Zombies with increasing difficulty/skill/creativity/
//! health/attackdmg come in waves from the outer edges toward the anchor
//! crystal in the middle and/or player ... Endless upgrades, endless waves
//! of endlessly stronger and stronger enemies. You get a numerical score at
//! the end (individual scores of each person + total combined team score).
//! Map = laststandmap (similar cam and controls with bridgewars)"): one to
//! four people together, holding the anchor for as many waves as they can.
//! Its rules are `roundtown_net::last_stand`'s, which the server goes by
//! online and this device offline ([`play`]); its table is `table`'s, which
//! fixes the camera over you as in Bridge Wars.
//!
//! **The same game either way.** Offline this device runs the shared
//! [`Game`] itself and carries out what it says happened; online the server
//! runs it and sends how it stands. Either way what is drawn is drawn from
//! the same few things ([`Stand`]): the round ([`StandRound`]: the wave, the
//! anchor, everyone's health and score), the team's upgrades, everyone's
//! purse and kit, and the zombies' bodies, each carrying its kind and its
//! health ([`Shambler`]), the server's or this device's own. What is new in
//! them is said once, as it comes ([`screen`]); the rest is drawn as it
//! stands ([`looks`]).
//!
//! **The fight is Bridge Wars' kit's** (`arena`, `market`, `loot`,
//! `utility`, `gear`, `fire`, `dynamite`): while a table of it is played,
//! this game owns the fight's state (`arena::Fighting` on its venue, your
//! purse in `arena::Resources`, your kit in `arena::Outfit`, its time in
//! `arena::FightTime`, the shops' cards in `market::Stalls`), and gives it
//! back once as the table goes. Everyone is on one team ([`TEAM`]), the
//! zombies on theirs (Bridge Wars' `ZOMBIE_TEAM`), so that a punch lands on
//! a zombie and never on a teammate.

use std::collections::HashMap;

use bevy::prelude::*;
use roundtown_net::last_stand::{Game, Kind, StandRound, StandUpgrades, Ware, team_score};
use roundtown_net::{Item, Kit, Purse, UTILITY_SLOTS};

use crate::hud;
use crate::ui::{fill, ui};

mod finds;
mod looks;
mod play;
mod screen;

/// Something the server says to this device alone in its game of Last Stand
/// (`roundtown_net::last_stand::Tell`), handed on by `net`.
#[derive(Message, Clone, Debug)]
pub(crate) struct Told(pub roundtown_net::last_stand::Tell);

/// The team everyone at a table of Last Stand is on (`crate::Team`): the
/// blue, mine. One team, but a team all the same, so that a punch passes a
/// teammate by and lands on the zombie behind them, and every name over a
/// head and on the plate top left is in the one colour: blue, where red
/// would read as hurt beside the health bars and green as one of the
/// zombies.
pub(crate) const TEAM: u8 = 1;

/// A game of Last Stand on this device, from the table's word that you sit
/// at one to its end: whether the server runs it, your seat, the fight's
/// number (`arena::Begun`); offline, the game itself, once the table plays
/// it, and what of it this device has put in the world, by the game's own
/// numbers; and, either way, how it stands as last seen, which is all that
/// is drawn from, and what of that has been said.
#[derive(Resource)]
pub(crate) struct Stand {
    online: bool,
    seat: usize,
    fight: u32,
    game: Option<Game>,
    round: Option<StandRound>,
    upgrades: StandUpgrades,
    purses: Vec<Purse>,
    kits: Vec<Kit>,
    /// How far into the fight it is, in seconds (`arena::FightTime`).
    elapsed: f32,
    zombies: HashMap<u32, Entity>,
    spits: HashMap<u32, Entity>,
    balls: HashMap<u32, Entity>,
    sticks: HashMap<u32, Entity>,
    pieces: HashMap<u32, Entity>,
    /// Offline, the swords the dead dropped and the treasure chests
    /// standing, by the game's numbers ([`finds`]).
    swords: HashMap<u32, Entity>,
    chests: HashMap<u32, Entity>,
    /// Offline, sticks of dynamite the game has gone into the void, until
    /// this device's own have too (`play::sink`).
    sinking: Vec<Entity>,
    /// What has been said of how it goes ([`screen`]).
    said: Said,
    /// You have said you are ready, in this break.
    ready: bool,
    /// Its end handed to the table, offline.
    finished: bool,
    /// Where the game has just put you, offline, for you to be stood there.
    put: Option<Transform>,
    /// Whether a shop's window may come down over you as you stand in its
    /// outline, in the middle of a wave only once you have stood there a
    /// moment ([`ShopDoor`]).
    door: ShopDoor,
}

/// Whether a shop's window may open for you (since 2026-11-18; a
/// play-tester, fighting round the crystal in the eighth wave, stepped into
/// the item shop's outline 4.8 m from it and had the window come down over
/// the middle of the screen, every zombie round them under it): between
/// waves, as you step into its outline, as in Bridge Wars; in the middle of
/// a wave, only once you have stood in it [`SHOP_SETTLE`] seconds without
/// swinging or being hit (mine: whoever means to shop stops, whoever is
/// fighting there swings or is hit), and then for as long as you stay in
/// it. Its window is the kit's (`market`), which opens on Last Stand's map
/// only where the game hands it cards to show (`market::Stalls`): until it
/// may, the game hands it none (`play::publish`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ShopDoor {
    /// How long you have stood in the outline in this wave without a swing
    /// or a blow, in seconds.
    settled: f32,
    /// The window may be up: you are in an outline, and may shop there.
    open: bool,
}

/// How long you stand in a shop's outline in the middle of a wave before
/// its window comes down, in seconds (mine): 0.2 since the evening of
/// 2026-11-18 (Hajun: "right now the shops seem to only open when u rly go
/// inside deep into the shop, so make it a bit more sensitive"; it was half
/// a second, in which a body at a run is through the outline and at the
/// back of the stand), the outline a little wider too (`market`).
const SHOP_SETTLE: f32 = 0.2;

impl ShopDoor {
    /// A frame `dt` long: you `inside` a shop's outline or not, in a `wave`
    /// or not, and `stirred` (you swung, or were hit) or not. Whether its
    /// window may be up now.
    fn step(&mut self, inside: bool, wave: bool, stirred: bool, dt: f32) -> bool {
        if !inside {
            *self = Self::default();
            return false;
        }
        if !self.open {
            if !wave {
                self.open = true;
            } else if stirred {
                self.settled = 0.0;
            } else {
                self.settled += dt;
                self.open = self.settled >= SHOP_SETTLE;
            }
        }
        self.open
    }
}

/// Something this game has said across the middle of the screen
/// (`hud::announce_stroked`: "Go!", a wave come or cleared, the anchor broken, the
/// end), for as long as it is up: what this game writes over the world
/// there (the crystal's bar, the words over the pads) is put away
/// meanwhile, rather than drawn into its words ([`looks`]; a play-tester,
/// 2026-11-18: "——Go!", the bar run into it, and "Hold the anchor" into a
/// pad's "Silver"); all of it is put away while a shop's window is up, as
/// it would show through the window's see-through dark among its cards
/// ([`screen`]), and taken down as the results come up, whose window it
/// would show through as well.
#[derive(Component)]
pub(crate) struct Announced;

/// How long one body's blows are said by one number, in seconds: no number
/// comes off it oftener ([`Tally`]; mine, a play-tester having found the
/// numbers off a body clawed at by several zombies at once, or the crystal,
/// in a heap that read as one figure: "6" over "7").
const TALLY_SECS: f32 = 0.3;

/// What blows on one body have taken that no number has said yet, and when
/// the last number came off it: each blow floats a number at once, but no
/// oftener than every [`TALLY_SECS`], those that come meanwhile said
/// together by the next.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Tally {
    unsaid: f32,
    said_at: Option<f32>,
}

impl Tally {
    /// A blow that took `damage`.
    fn add(&mut self, damage: f32) {
        self.unsaid += damage.max(0.0);
    }

    /// What a number is to say now, at `now` seconds, if one is to come.
    fn due(&mut self, now: f32) -> Option<f32> {
        if self.unsaid <= 0.0 || self.said_at.is_some_and(|at| now - at < TALLY_SECS) {
            return None;
        }
        self.said_at = Some(now);
        Some(std::mem::take(&mut self.unsaid))
    }

    /// Whatever is unsaid, to be said with a last blow.
    fn rest(&mut self) -> f32 {
        std::mem::take(&mut self.unsaid)
    }
}

/// How far a number floating off a body stands from it, either way across
/// the screen and up and down it, in metres, thrown afresh for each: from
/// straight overhead a number rises toward the camera and hardly moves on
/// the screen, so that numbers off one body stood one over another (mine).
const SPREAD_ACROSS: f32 = 0.75;
const SPREAD_ALONG: f32 = 0.45;

/// Where a number off the body at `at` floats from: thrown about it
/// ([`SPREAD_ACROSS`]), the camera over it looking down the way it does,
/// -Z up the screen.
fn spread(at: Vec3) -> Vec3 {
    at + Vec3::new((fastrand::f32() * 2.0 - 1.0) * SPREAD_ACROSS, 0.0, (fastrand::f32() * 2.0 - 1.0) * SPREAD_ALONG)
}

impl Stand {
    /// A game of Last Stand at a table, you in `seat`, the server's if
    /// `online`, numbered `fight`: nothing of it seen yet.
    fn new(online: bool, seat: usize, fight: u32) -> Self {
        Self {
            online,
            seat,
            fight,
            game: None,
            round: None,
            upgrades: StandUpgrades::default(),
            purses: Vec::new(),
            kits: Vec::new(),
            elapsed: 0.0,
            zombies: HashMap::new(),
            spits: HashMap::new(),
            balls: HashMap::new(),
            sticks: HashMap::new(),
            pieces: HashMap::new(),
            swords: HashMap::new(),
            chests: HashMap::new(),
            sinking: Vec::new(),
            said: Said::default(),
            ready: false,
            finished: false,
            put: None,
            door: ShopDoor::default(),
        }
    }

    /// Whether a shop's window is up over you: you in its outline, it
    /// allowed to be ([`ShopDoor`]), the game on.
    fn shopping(&self) -> bool {
        self.door.open && self.playing()
    }

    /// Your purse, as last seen.
    fn purse(&self) -> Purse {
        self.purses.get(self.seat).copied().unwrap_or_default()
    }

    /// Your kit, as last seen.
    fn kit(&self) -> Kit {
        self.kits.get(self.seat).copied().unwrap_or_default()
    }

    /// Whether the game is being played: begun, and not over.
    fn playing(&self) -> bool {
        self.round
            .as_ref()
            .is_some_and(|round| !matches!(round.stage, roundtown_net::last_stand::StandStage::Over))
    }

    /// Whether whoever is in `seat` is standing: in the game, alive.
    fn standing(&self, seat: usize) -> bool {
        self.round
            .as_ref()
            .is_some_and(|round| round.health.get(seat).is_some_and(|health| *health > 0))
    }
}

/// What has been said of how a game goes, across the screen and in the
/// chat, for each thing to be said once as it comes: that it has begun;
/// the wave last said, and whether it was on; the anchor's health as last
/// seen, and whether it was said to be broken; everyone's health, whether
/// they were out, and whether they had left the game, by seat; and the
/// end.
#[derive(Default)]
struct Said {
    begun: bool,
    wave: u16,
    on: bool,
    anchor: Option<u16>,
    broken: bool,
    health: Vec<u16>,
    out: Vec<bool>,
    gone: Vec<bool>,
    over: bool,
}

/// Something that happened in a game of Last Stand that its round does not
/// say, for what is drawn and said of it: the anchor's beam zapped what
/// stands here; someone, by seat, bought this for the team, which has it
/// to this level now; someone killed a zombie of this kind. Offline from
/// the game's own events, online from what the server tells
/// ([`Told`]).
#[derive(Message, Clone, Copy, PartialEq, Debug)]
pub(crate) enum Happened {
    Zap(Vec3),
    Upgraded { by: usize, ware: Ware, level: u16 },
    Slain { by: usize, kind: Kind },
}

/// You are ready for the next wave: the button beside the plate between
/// waves, or R on desktop ([`screen`]).
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct ReadyAsked;

/// The systems of a game of Last Stand: after the tables', which say when
/// one begins and ends.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StandSystems;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Told>().add_message::<Happened>().add_message::<ReadyAsked>();
    play::plugin(app);
    looks::plugin(app);
    screen::plugin(app);
    finds::plugin(app);
}

/// The buttons round the screen at a table of Last Stand, Bridge Wars' less
/// what is not in it (Hajun: "similar cam and controls with bridgewars"):
/// top left your name, your health under it, and under that what you hold
/// of the four things the generators make, as in Bridge Wars, but no
/// minimap; and round the jump button the sword in its place, which
/// strikes, the jump button beside it, the five buttons for things to use,
/// and the chat's keyboard, each where Bridge Wars has it. No plank: there
/// is no bridge to build.
pub(crate) fn spawn_bar(commands: &mut Commands, assets: &AssetServer, images: &mut Assets<Image>) {
    let colours = ui().colours;
    let (left, _right) = hud::game_bar(commands);
    commands.spawn((ChildOf(left), hud::name_plate()));
    commands.spawn((ChildOf(left), hud::health_bar()));
    for item in Item::ALL {
        commands.spawn((
            ChildOf(left),
            hud::resource_row(item, images.add(hud::item_picture(item)), colours.arena.item(item)),
        ));
    }
    let round = commands.spawn(hud::round_the_jump()).id();
    let arrow = images.add(hud::jump_arrow());
    commands.spawn((
        ChildOf(round),
        hud::punch_button(images.add(hud::sword()), arrow.clone(), crate::JUMP_SIZE_VMIN, hud::beside_the_jump(0.0, 0.0)),
    ));
    let (right, bottom, size) = hud::ARENA_JUMP;
    commands.spawn((ChildOf(round), hud::jump_button(arrow, size, hud::beside_the_jump(right, bottom))));
    for slot in 0..UTILITY_SLOTS {
        let (right, bottom) = hud::slot_place(slot);
        commands.spawn((ChildOf(round), hud::utility_slot(slot, hud::beside_the_jump(right, bottom))));
    }
    // The chat's keyboard while lines may be typed, and since 2026-11-27
    // a small speech bubble in its place (`quick`).
    if roundtown_net::chat::TYPED {
        commands.spawn((ChildOf(round), hud::chat_button(images)));
    } else {
        commands.spawn((ChildOf(round), hud::chat_spot(assets)));
    }
}

/// What its score is called among the results: the file's
/// (`words.last_stand.score`).
pub(crate) fn score_word() -> String {
    ui().words.last_stand.score.clone()
}

/// What the line under the results says of what the whole table made:
/// the waves it cleared, `total`, which is what each person is paid by,
/// and the team's score, everyone's added up and 100 a wave cleared
/// (Hajun: "individual scores of each person + total combined team
/// score"): "Waves cleared: 8 · Team score: 4,210". The team's score is
/// worked out from everyone's as the game ended ([`TeamScore`]), which the
/// results' window has but this is not handed.
pub(crate) fn total_words(total: u32) -> Option<String> {
    let score = TeamScore::get().filter(|(waves, _)| *waves == total).map_or(0, |(_, score)| score);
    Some(fill(
        &ui().words.last_stand.total,
        &[("waves", &total.to_string()), ("score", &hud::money(score))],
    ))
}

/// The team's score as the last game here ended, with the waves it
/// cleared: for the line under the results ([`total_words`]), which the
/// table writes from the waves alone. Kept in one place for the whole app,
/// there being one game at a time on a device; read only for the waves it
/// was kept with, so that another game's never stands in for this one's.
struct TeamScore;

impl TeamScore {
    fn kept() -> &'static std::sync::Mutex<Option<(u32, u32)>> {
        static KEPT: std::sync::Mutex<Option<(u32, u32)>> = std::sync::Mutex::new(None);
        &KEPT
    }

    /// The game has ended, `waves` cleared, everyone's scores `scores`.
    fn keep(waves: u32, scores: &[u32]) {
        if let Ok(mut kept) = Self::kept().lock() {
            *kept = Some((waves, team_score(scores, waves)));
        }
    }

    fn get() -> Option<(u32, u32)> {
        Self::kept().lock().ok().and_then(|kept| *kept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_button_round_the_jump_button_is_clear_of_every_other() {
        // The sword in the jump button's place, the jump button, the five
        // buttons for things to use and the chat's keyboard.
        let (jump_right, jump_bottom, jump_size) = hud::ARENA_JUMP;
        let (chat_right, chat_bottom, chat_size) = hud::CHAT_BUTTON;
        let mut buttons = vec![
            ((0.0, 0.0), crate::JUMP_SIZE_VMIN),
            ((jump_right, jump_bottom), jump_size),
            ((chat_right, chat_bottom), chat_size),
        ];
        buttons.extend((0..UTILITY_SLOTS).map(|slot| (hud::slot_place(slot), hud::ACTION_BUTTON)));
        hud::clear_of_each_other(&buttons).unwrap();
    }

    #[test]
    fn the_rules_sea_is_where_a_body_wades_in_the_town_s_sea() {
        // Where the feet of a body in the sea come to (`water_rest_y`, but
        // for its bob), as the rules have them for zombies and the drops.
        assert!((roundtown_net::last_stand::SEA_BED - (crate::WATER_Y - crate::WATER_WADE)).abs() < 1e-4);
        // And Last Stand's map has the sea round it.
        assert!(!crate::island::bottomless(roundtown_net::Venue::LastStand));
    }

    #[test]
    fn the_rules_numbers_for_the_map_are_hajun_s_map_s() {
        // The ground, the plinth, the pads, the stands, the palms and the
        // crates where the server's zombies go round them, held to
        // `laststandmap.glb` as the app loads it, lifted.
        let parts = crate::island::glb::read_venue(roundtown_net::Venue::LastStand);
        let view: Vec<(&str, Option<&str>, &[[Vec3; 3]])> = parts
            .iter()
            .map(|part| (part.node.as_str(), part.material.as_deref(), part.triangles.as_slice()))
            .collect();
        let wrong = roundtown_net::last_stand::disagreements(&view);
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    #[test]
    fn a_shop_s_window_comes_down_at_once_between_waves_and_in_a_wave_only_once_you_stand_still() {
        // (Frames a sixteenth of a second long, which add up exactly.)
        let frame = 0.0625;
        let mut door = ShopDoor::default();
        // Between waves, as you step in; out, shut.
        assert!(door.step(true, false, false, frame));
        assert!(!door.step(false, false, false, frame));
        // In a wave: not as you step in, nor while you swing or are hit;
        // once you have stood there a moment.
        let mut opened_at = None;
        for step in 0..40 {
            let stirred = step < 10;
            if door.step(true, true, stirred, frame) && opened_at.is_none() {
                opened_at = Some(step);
            }
        }
        // Stirred last in frame 9: open the while after, in frame 13.
        assert_eq!(opened_at, Some(9 + (SHOP_SETTLE / frame).ceil() as i32));
        // A moment, no longer (since the evening of 2026-11-18: half a
        // second, a run through the outline and more, until then).
        assert!(SHOP_SETTLE <= 0.25);
        // Open, it stays so while you are in it, whatever you do.
        assert!(door.step(true, true, true, frame));
        // Walking through in a wave, in less than the while: never.
        let mut door = ShopDoor::default();
        assert!((0..3).all(|_| !door.step(true, true, false, frame)));
        assert!(!door.step(false, true, false, frame));
        // Open as the wave comes, it stays open.
        let mut door = ShopDoor::default();
        assert!(door.step(true, false, false, frame) && door.step(true, true, true, frame));
    }

    #[test]
    fn blows_on_one_body_are_said_at_once_and_then_together_no_oftener_than_a_while() {
        let mut tally = Tally::default();
        assert_eq!(tally.due(0.0), None, "nothing taken");
        tally.add(6.0);
        assert_eq!(tally.due(1.0), Some(6.0), "the first at once");
        tally.add(7.0);
        tally.add(5.0);
        assert_eq!(tally.due(1.1), None);
        assert_eq!(tally.due(1.01 + TALLY_SECS), Some(12.0), "the next two as one");
        // With a last blow, whatever is unsaid.
        tally.add(3.0);
        assert_eq!(tally.rest(), 3.0);
        assert_eq!(tally.due(9.0), None);
        // Thrown about the body, a little way off it at most.
        for _ in 0..50 {
            let off = spread(Vec3::new(4.0, 1.0, -2.0)) - Vec3::new(4.0, 1.0, -2.0);
            assert!(off.x.abs() <= SPREAD_ACROSS && off.z.abs() <= SPREAD_ALONG && off.y == 0.0);
        }
    }

    #[test]
    fn the_line_under_the_results_says_the_waves_and_the_team_s_score() {
        TeamScore::keep(8, &[1800, 1610]);
        let line = total_words(8).expect("a line");
        assert_eq!(line, "Waves cleared: 8 · Team score: 4,210");
        // Another game's score is never this one's.
        assert_eq!(total_words(3).expect("a line"), "Waves cleared: 3 · Team score: 0");
    }

    #[test]
    fn every_word_of_last_stand_s_is_one_the_fonts_draw_and_fills_what_it_should() {
        let ui = crate::ui::Ui::built_in();
        let words = &ui.words.last_stand;
        let mut all: Vec<&str> = vec![
            &words.score,
            &words.total,
            &words.wave,
            &words.left,
            &words.next_wave,
            &words.get_ready,
            &words.cleared_caption,
            &words.next_new,
            &words.tip,
            &words.ready_count,
            &words.anchor_health,
            &words.ready,
            &words.go,
            &words.go_caption,
            &words.runners,
            &words.brutes,
            &words.spitters,
            &words.leapers,
            &words.a_giant,
            &words.giants,
            &words.cleared,
            &words.anchor_broken,
            &words.anchor_broken_caption,
            &words.under_attack,
            &words.fell,
            &words.back_in,
            &words.out,
            &words.out_line,
            &words.generator,
            &words.generator_level,
            &words.generator_all,
            &words.generator_fast,
            &words.bought_line,
            &words.down_line,
            &words.out_chat,
            &words.left_line,
            &words.slain_line,
            &words.anchor_line,
            &words.cleared_line,
            &words.items_title,
            &words.upgrades_title,
            &words.sword_stat,
            &words.dynamite_stat,
            &words.sharpness,
            &words.protection,
            &words.protection_stat,
            &words.fortify,
            &words.beam,
            &words.lucky,
            &words.lucky_stat,
            &words.level,
        ];
        all.extend(words.forge.iter().map(String::as_str));
        all.extend(words.forge_stats.iter().map(String::as_str));
        for said in all {
            assert!(!said.trim().is_empty(), "an empty word");
            assert!(said.chars().all(roundtown_net::text::allowed), "{said:?} has a letter the fonts do not draw");
        }
        // Over a zombie, its kind's name, or nothing but its bar.
        for kind in &words.kinds {
            assert!(kind.chars().all(roundtown_net::text::allowed), "{kind:?}");
        }
        for (word, keys) in [
            (&words.total, &["{waves}", "{score}"][..]),
            (&words.wave, &["{wave}"]),
            (&words.left, &["{count}"]),
            (&words.next_wave, &["{time}"]),
            (&words.next_new, &["{new}"]),
            (&words.ready_count, &["{count}", "{of}"]),
            (&words.anchor_health, &["{left}", "{most}"]),
            (&words.giants, &["{count}"]),
            (&words.cleared, &["{wave}"]),
            (&words.fell, &["{wave}"]),
            (&words.back_in, &["{seconds}"]),
            (&words.generator_level, &["{level}", "{items}"]),
            (&words.generator_all, &["{level}"]),
            (&words.generator_fast, &["{level}", "{fast}"]),
            (&words.bought_line, &["{name}", "{ware}", "{level}"]),
            (&words.down_line, &["{name}"]),
            (&words.out_chat, &["{name}"]),
            (&words.left_line, &["{name}"]),
            (&words.slain_line, &["{name}"]),
            (&words.level, &["{level}"]),
        ] {
            for key in keys {
                assert!(word.contains(key), "{word:?} has no {key}");
            }
        }
    }
}
