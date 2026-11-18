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
            sinking: Vec::new(),
            said: Said::default(),
            ready: false,
            finished: false,
            put: None,
        }
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
}

/// The buttons round the screen at a table of Last Stand, Bridge Wars' less
/// what is not in it (Hajun: "similar cam and controls with bridgewars"):
/// top left your name, your health under it, and under that what you hold
/// of the four things the generators make, as in Bridge Wars, but no
/// minimap; and round the jump button the sword in its place, which
/// strikes, the jump button beside it, the five buttons for things to use,
/// and the chat's keyboard, each where Bridge Wars has it. No plank: there
/// is no bridge to build.
pub(crate) fn spawn_bar(commands: &mut Commands, _assets: &AssetServer, images: &mut Assets<Image>) {
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
    commands.spawn((ChildOf(round), hud::chat_button(images)));
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
            &words.ready_count,
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
            &words.fell,
            &words.back_in,
            &words.out,
            &words.out_line,
            &words.generator,
            &words.generator_level,
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
            (&words.ready_count, &["{count}", "{of}"]),
            (&words.giants, &["{count}"]),
            (&words.cleared, &["{wave}"]),
            (&words.fell, &["{wave}"]),
            (&words.back_in, &["{seconds}"]),
            (&words.generator_level, &["{level}", "{items}"]),
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
