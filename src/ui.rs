//! The colours and the words of everything drawn over the world, in
//! `assets/ui.ron`, for Hajun to change by hand (2026-11-04: "i think ai
//! cannot rly implement my instructions exactly and precisely how i want
//! it. how can i make it so i can tweak it myself"; and, of a first file
//! that held every size and margin too, "too overfilled with constants ...
//! i just want to be able to change the colors (including transparency) of
//! UI objects like buttons and windows and text. easily ... AND Text. so i
//! want to change some of the text easily like the 'A guest account, kept
//! on this device.' in settings"). Sizes and layout stay in the code. And,
//! since 2026-11-06, every number of how the car you drive goes ([`Car`];
//! Hajun: "put all the constants related to the car in the ron file. like
//! related to movement. so the turning, the boost/reverse, acc/speed,
//! multipliers, etc. i want to modify the numbers by myself while playing
//! on pc"), which `drive` reads every frame, so that a save while driving
//! changes the car at once; since 2026-11-07 every number of the punch
//! and its knockback ([`Melee`]), Minecraft 1.8.9's, the same way; and
//! since the small hours of 2026-11-08 how Bridge Wars is fought
//! ([`Arena`]: the run, the health, the damage, the respawn, the bots'
//! sight and the fall).
//!
//! Everything that draws the screen reads [`ui`] where it spawns: a global,
//! not a resource, so that the plain functions every screen is built from
//! (`hud::words`, `builder::label`, `builder::dimmed` and the rest) can read
//! it without every caller threading it through. The file is compiled in
//! ([`BUILT_IN`]), so a phone shows whatever it said when its app was
//! built, with nothing to copy or package. On desktop it is read again
//! whenever it changes on disk ([`reread`]), within a quarter of a second
//! of a save, and [`UiChanged`] changes that frame, on which whatever stays
//! on the screen is drawn again from it (`hud::restyle`, `signup::redraw`,
//! `settings::redraw`, `tags::recolour`); a dialog, the shop or House
//! Builder's board take it when they next come up. A file that will not
//! read is logged, and the last good one kept.
//!
//! Every field is read, and a field the file has that [`Ui`] has not is an
//! error (`deny_unknown_fields`), so that a misspelt name is not silently
//! ignored; `cargo test` reads the file as built in.

use std::sync::{Arc, LazyLock, RwLock};

use bevy::prelude::*;
use serde::{Deserialize, Deserializer};

/// The file, as it was when this was built.
const BUILT_IN: &str = include_str!("../assets/ui.ron");

/// Where the file is on this PC, for reading again as it changes: only on
/// desktop, and only where the game runs beside its own tree.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/ui.ron");

/// How often the file is looked at for a change, in seconds.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const LOOK_EVERY: f32 = 0.25;

/// The file as it stands.
static CURRENT: LazyLock<RwLock<Arc<Ui>>> = LazyLock::new(|| RwLock::new(Arc::new(Ui::built_in())));

/// The colours and the words as the file has them now. Read wherever
/// something is spawned on the screen; a handle, so that it costs nothing
/// to read and nothing to hold.
pub(crate) fn ui() -> Arc<Ui> {
    CURRENT
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

/// `template` with each `{name}` of `fills` filled in: "You came {place}".
pub(crate) fn fill(template: &str, fills: &[(&str, &str)]) -> String {
    let mut text = template.to_string();
    for (name, value) in fills {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

/// Changed in the frame the file has been read again (`DetectChanges`), for
/// whatever stays on the screen to be drawn again from it.
#[derive(Resource, Default)]
pub(crate) struct UiChanged;

/// A colour as the file writes it, as a web page would: `"#rrggbb"`, or
/// `"#rrggbbaa"` with its alpha (`00` invisible, `ff` solid; `"#rgb"` and
/// `"#rgba"` too); or `"rgb(255, 214, 77)"`, `"rgba(255, 214, 77, 0.5)"`,
/// `"hsl(43, 100%, 65%)"` or `"hsla(43, 100%, 65%, 0.5)"`, whichever of them
/// VS Code's colour picker writes back (`editor.defaultColorDecorators`).
fn colour<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Color, D::Error> {
    let text = String::deserialize(deserializer)?;
    parse_colour(&text).map_err(serde::de::Error::custom)
}

/// What colour `text` names, or what is wrong with it.
fn parse_colour(text: &str) -> Result<Color, String> {
    let text = text.trim();
    let wrong =
        || format!("{text:?} is not a colour: #rrggbb, #rrggbbaa, rgb(), rgba(), hsl() or hsla()");
    if text.starts_with('#') {
        return Srgba::hex(text).map(Color::from).map_err(|_| wrong());
    }
    let (name, inside) = text
        .split_once('(')
        .and_then(|(name, rest)| {
            rest.strip_suffix(')')
                .map(|inside| (name.trim().to_ascii_lowercase(), inside))
        })
        .ok_or_else(wrong)?;
    // `rgb(255, 214, 77)`, `rgba(255 214 77 / 50%)`: commas, spaces or a
    // slash between the numbers, each of which may be a percentage of
    // `full`, the most that number can be.
    let parts: Vec<&str> = inside
        .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect();
    let number = |part: &str, full: f32| -> Result<f32, String> {
        let value = match part.strip_suffix('%') {
            Some(percent) => percent
                .trim()
                .parse::<f32>()
                .map(|percent| percent / 100.0 * full),
            None => part.parse::<f32>(),
        };
        value.map_err(|_| format!("{part:?} in {text:?} is not a number"))
    };
    let alpha = parts.get(3).map_or(Ok(1.0), |part| number(part, 1.0))?;
    match (name.as_str(), parts.len()) {
        ("rgb" | "rgba", 3 | 4) => Ok(Color::srgba(
            number(parts[0], 255.0)? / 255.0,
            number(parts[1], 255.0)? / 255.0,
            number(parts[2], 255.0)? / 255.0,
            alpha,
        )),
        ("hsl" | "hsla", 3 | 4) => {
            let hsla = Hsla::new(
                number(parts[0], 360.0)?,
                number(parts[1], 1.0)?,
                number(parts[2], 1.0)?,
                alpha,
            );
            Ok(Color::Srgba(Srgba::from(hsla)))
        }
        _ => Err(wrong()),
    }
}

/// Everything the file says.
#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Ui {
    pub colours: Colours,
    pub words: Words,
    pub car: Car,
    pub melee: Melee,
    pub arena: Arena,
}

/// How Bridge Wars is fought (`arena`, since the night of 2026-11-07): how
/// fast everyone runs there, in metres a second; the health everyone
/// starts with and what a punch takes off it; how long the dead stay dead,
/// in seconds; how long after the last hit health starts to come back,
/// in seconds, and how much a second; how far a bot sees whoever it goes
/// for, in metres, the least time between its punches, in seconds, and
/// under what share of its health it runs away instead; and how far
/// below the map a body has fallen before it is dead, in metres. The file
/// says what each is; [`Arena::check`] keeps out a number the game would
/// break on.
#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Arena {
    pub run_speed: f32,
    pub health: f32,
    pub damage: f32,
    pub respawn_secs: f32,
    pub regen_after: f32,
    pub regen_per_sec: f32,
    pub hunt_radius: f32,
    pub bot_punch_every: f32,
    pub flee_below: f32,
    pub fall_to_die: f32,
    /// What a team's respawn anchor starts with: ten punches' worth
    /// (since 2026-11-09; twenty on 2026-11-08).
    pub anchor_health: f32,
    /// How many metres of bridge everyone starts a game with (`bridge`,
    /// since 2026-11-09; Hajun: "players will buy the bridge by meter
    /// count ... for now make each person start with 100m"; 500 since
    /// 2026-11-10: "increase the default bridge meter to 500m instead of
    /// 100"; and none since 2026-11-11, when the shop began to sell it,
    /// 25 m for 4 silver (50 for its first evening), as a game of Bedwars
    /// begins with no blocks: my
    /// call, Hajun not having said).
    pub bridge_metres: f32,
    /// With how many metres of bridge left, and fewer, the number written
    /// in front of you as you build turns from `colours.arena.bridge_left`
    /// to `bridge_low`, and with how many, and fewer, it is `bridge_out`,
    /// shading from the one to the other between (`bridge::show_left`,
    /// since 2026-11-10; Hajun: "white font but orange -> red when its
    /// 10m->5m left").
    pub bridge_low_at: f32,
    pub bridge_red_at: f32,
    /// How far up the camera is fixed over you in a game, in metres, as
    /// `car.overhead` is over the car (since late on 2026-11-11; Hajun: "make the
    /// camera in that mode's default zoom to be zoom in a bit more so the
    /// borders of each players island just fits around perfectly on the
    /// camera on my mobile phone, and lock that zoom so u cannot zoom in
    /// or out"): nothing zooms it (`OrbitCamera::top_down`).
    pub camera_height: f32,
    /// How a sword is held and swung ([`SwordSwing`]).
    pub sword_swing: SwordSwing,
    /// How far a sword held on its edge (`SwordSwing::Chop`) is tilted at
    /// the ready, in degrees from straight up: forward, toward where you
    /// face, and over to the right (`crate::Guard`; since later on
    /// 2026-11-12, Hajun, of 61 and 36: "the sword holding is too tilted
    /// now (both forward tilt and side tilt). make it more straight").
    /// Straight up altogether, the camera over you sees it end on, a dot.
    pub sword_tilt_forward: f32,
    pub sword_tilt_side: f32,
}

/// How a sword is held and swung in Bridge Wars (`sword_offset` and
/// `chop_offset` in `lib.rs`, `gear`): the file's `arena.sword_swing`.
/// Both are kept, to go back and forth between by the one word (Hajun,
/// 2026-11-12: "instead of removing the code for the sword animation now
/// (cuz i might wanna change it back after trying this new way), make a
/// new sword animation").
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SwordSwing {
    /// The first way (2026-11-12): held flat, its flat side up, and swept
    /// level across in front from the right to the left.
    Sweep,
    /// Held on its edge, "narrow", and cut downward from over the right
    /// shoulder to low on the left ("this time instead of holding the
    /// sword flat, hold it narrow and slash downward(?)").
    Chop,
}

impl Arena {
    /// What is wrong with these numbers, if anything: one that is not a
    /// number or less than nothing, a speed, a health or a wait between
    /// punches of nothing, which the game divides by or would spin on, or
    /// a share over 1.
    fn check(&self) -> Result<(), String> {
        let all = [
            ("run_speed", self.run_speed),
            ("health", self.health),
            ("damage", self.damage),
            ("respawn_secs", self.respawn_secs),
            ("regen_after", self.regen_after),
            ("regen_per_sec", self.regen_per_sec),
            ("hunt_radius", self.hunt_radius),
            ("bot_punch_every", self.bot_punch_every),
            ("flee_below", self.flee_below),
            ("fall_to_die", self.fall_to_die),
            ("anchor_health", self.anchor_health),
            ("bridge_metres", self.bridge_metres),
            ("bridge_low_at", self.bridge_low_at),
            ("bridge_red_at", self.bridge_red_at),
            ("camera_height", self.camera_height),
            ("sword_tilt_forward", self.sword_tilt_forward),
            ("sword_tilt_side", self.sword_tilt_side),
        ];
        for (name, value) in all {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("arena.{name} is {value}: it has to be a number, 0 or more"));
            }
        }
        for (name, value) in [
            ("run_speed", self.run_speed),
            ("health", self.health),
            ("bot_punch_every", self.bot_punch_every),
            ("anchor_health", self.anchor_health),
            ("camera_height", self.camera_height),
        ] {
            if value <= 0.0 {
                return Err(format!("arena.{name} is {value}: it has to be more than 0"));
            }
        }
        if self.flee_below > 1.0 {
            return Err(format!("arena.flee_below is {}: it is a share, 0 to 1", self.flee_below));
        }
        for (name, value) in [("sword_tilt_forward", self.sword_tilt_forward), ("sword_tilt_side", self.sword_tilt_side)] {
            if value > 90.0 {
                return Err(format!("arena.{name} is {value}: it is degrees from straight up, 0 to 90"));
            }
        }
        if self.bridge_red_at > self.bridge_low_at {
            return Err(format!(
                "arena.bridge_red_at is {} and arena.bridge_low_at {}: the number turns red with fewer metres left than it turns orange with",
                self.bridge_red_at, self.bridge_low_at
            ));
        }
        Ok(())
    }
}

/// How the car you drive goes (`drive`): every number of it, in metres,
/// seconds and degrees, for tuning while playing. The file says what each
/// is; [`Car::check`] keeps out a number the car would break on.
#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Car {
    /// Speeds, in metres a second: with the stick pushed all the way, with
    /// the jump button held, and with the fist held.
    pub cruise: f32,
    pub boost: f32,
    pub reverse: f32,
    /// In metres a second a second: gaining speed cruising, gaining it
    /// boosting or reversing, and losing it.
    pub accel: f32,
    pub boost_accel: f32,
    pub brake: f32,
    /// The tightest circle, as the radius its back wheels go round, in
    /// metres, up to `easy_speed` metres a second; wider in proportion
    /// faster.
    pub tightest: f32,
    pub easy_speed: f32,
    /// How fast the front wheels turn, in degrees a second, and how far off
    /// the stick, in degrees, they are turned all the way.
    pub steer_rate: f32,
    pub full_lock_at: f32,
    /// The least speed, in metres a second, its steering turns it at with
    /// the stick pushed: stopped against a wall, it comes round.
    pub creep: f32,
    /// The camera over the car, in metres: where it starts, and the nearest
    /// and the furthest it zooms.
    pub overhead: f32,
    pub overhead_nearest: f32,
    pub overhead_furthest: f32,
    /// How its nose and its tail are felt for walls: each a circle of this
    /// radius, in metres, its middle this far back from the car's very end,
    /// between these heights over your feet; and how much of the move it
    /// asked for the car has to make, as a share, before it counts as
    /// stopped by what it met.
    pub end_radius: f32,
    pub end_back: f32,
    pub felt_low: f32,
    pub felt_high: f32,
    pub stopped_short: f32,
    /// The gun (the gift button held, in the car): how fast a bullet goes,
    /// in metres a second; how long between bullets while the button is
    /// held, in seconds; how far a bullet goes from where it was fired, in
    /// metres, before it is gone; and its radius, in metres.
    pub bullet_speed: f32,
    pub bullet_every: f32,
    pub bullet_range: f32,
    pub bullet_radius: f32,
}

impl Car {
    /// What is wrong with these numbers, if anything: one that is not a
    /// number, or less than nothing (a rate under 0 would panic a clamp),
    /// one the car divides by at 0, or a range inside out.
    fn check(&self) -> Result<(), String> {
        let all = [
            ("cruise", self.cruise),
            ("boost", self.boost),
            ("reverse", self.reverse),
            ("accel", self.accel),
            ("boost_accel", self.boost_accel),
            ("brake", self.brake),
            ("tightest", self.tightest),
            ("easy_speed", self.easy_speed),
            ("steer_rate", self.steer_rate),
            ("full_lock_at", self.full_lock_at),
            ("creep", self.creep),
            ("overhead", self.overhead),
            ("overhead_nearest", self.overhead_nearest),
            ("overhead_furthest", self.overhead_furthest),
            ("end_radius", self.end_radius),
            ("end_back", self.end_back),
            ("felt_low", self.felt_low),
            ("felt_high", self.felt_high),
            ("stopped_short", self.stopped_short),
            ("bullet_speed", self.bullet_speed),
            ("bullet_every", self.bullet_every),
            ("bullet_range", self.bullet_range),
            ("bullet_radius", self.bullet_radius),
        ];
        for (name, value) in all {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("car.{name} is {value}: it has to be a number, 0 or more"));
            }
        }
        for (name, value) in [
            ("tightest", self.tightest),
            ("easy_speed", self.easy_speed),
            ("full_lock_at", self.full_lock_at),
            ("overhead_nearest", self.overhead_nearest),
            ("bullet_every", self.bullet_every),
            ("bullet_radius", self.bullet_radius),
        ] {
            if value <= 0.0 {
                return Err(format!("car.{name} is {value}: it has to be more than 0"));
            }
        }
        if !(self.overhead_nearest <= self.overhead && self.overhead <= self.overhead_furthest) {
            return Err(format!(
                "car.overhead is {}: it has to be between overhead_nearest ({}) and overhead_furthest ({})",
                self.overhead, self.overhead_nearest, self.overhead_furthest
            ));
        }
        if self.felt_low >= self.felt_high {
            return Err(format!(
                "car.felt_low ({}) has to be under car.felt_high ({})",
                self.felt_low, self.felt_high
            ));
        }
        if self.stopped_short > 1.0 {
            return Err(format!("car.stopped_short is {}: it is a share, 0 to 1", self.stopped_short));
        }
        Ok(())
    }
}

/// How a punch lands and knocks back whoever it hits (`start_punch`,
/// `Shove::knock`): Minecraft 1.8.9's sword, as Hajun asked (2026-11-07:
/// "pressing melee (the punch) actually deals 'knockback' (minecraft 1.8.9
/// bedwars style melee knockback) ... make the melee time/cooldown numbers
/// similar to minecraft 1.8.9 sword melee"), in metres, seconds and
/// degrees, Minecraft's blocks a tick made metres a second (times 20). The
/// file says what each is; [`Melee::check`] keeps out a number the punch
/// would break on.
#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Melee {
    /// The swing: the arm out, and back, in seconds.
    pub swing_out: f32,
    pub swing_back: f32,
    /// The range a punch lands in, the band drawn round you (`range`):
    /// how far it reaches and how near it begins, in metres in front of
    /// your middle, how wide it is where it begins, in metres, and how far
    /// it opens out from there, in degrees, the whole cone.
    pub reach: f32,
    pub near: f32,
    pub width: f32,
    pub cone: f32,
    /// How long after a punch lands on a body no other moves it, in
    /// seconds.
    pub hurt_time: f32,
    /// How fast a body is knocked, in metres a second: away from the
    /// puncher, and that much more if the puncher was on the move; and up.
    pub knock_back: f32,
    pub sprint_bonus: f32,
    pub knock_up: f32,
    /// What is left of the knock each twentieth of a second, from 0 to 1,
    /// in the air and on the ground.
    pub air_drag: f32,
    pub ground_drag: f32,
}

impl Melee {
    /// How long a punch takes, out and back, in seconds.
    pub(crate) fn swing(&self) -> f32 {
        self.swing_out + self.swing_back
    }

    /// What is wrong with these numbers, if anything: one that is not a
    /// number or less than nothing, a swing of no time (which the arm's
    /// reach divides by), a cone of nothing or wider than all round, or
    /// not under 180 with a width (whose sides would never meet), a near
    /// edge past the reach, or a drag that would make a knock grow.
    fn check(&self) -> Result<(), String> {
        let all = [
            ("swing_out", self.swing_out),
            ("swing_back", self.swing_back),
            ("reach", self.reach),
            ("near", self.near),
            ("width", self.width),
            ("cone", self.cone),
            ("hurt_time", self.hurt_time),
            ("knock_back", self.knock_back),
            ("sprint_bonus", self.sprint_bonus),
            ("knock_up", self.knock_up),
            ("air_drag", self.air_drag),
            ("ground_drag", self.ground_drag),
        ];
        for (name, value) in all {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("melee.{name} is {value}: it has to be a number, 0 or more"));
            }
        }
        for (name, value) in [("swing_out", self.swing_out), ("swing_back", self.swing_back), ("cone", self.cone)] {
            if value <= 0.0 {
                return Err(format!("melee.{name} is {value}: it has to be more than 0"));
            }
        }
        if self.cone > 360.0 {
            return Err(format!("melee.cone is {}: it is in degrees, all round at most", self.cone));
        }
        if self.width > 0.0 && self.cone >= 180.0 {
            return Err(format!(
                "melee.cone is {}: with a width the sides have to open out from a point, under 180",
                self.cone
            ));
        }
        if self.near >= self.reach {
            return Err(format!("melee.near ({}) has to be under melee.reach ({})", self.near, self.reach));
        }
        for (name, value) in [("air_drag", self.air_drag), ("ground_drag", self.ground_drag)] {
            if value > 1.0 {
                return Err(format!("melee.{name} is {value}: it is a share, 0 to 1"));
            }
        }
        Ok(())
    }
}

/// The colours: what every screen shares, then each screen's own.
#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Colours {
    #[serde(deserialize_with = "colour")]
    pub text: Color,
    #[serde(deserialize_with = "colour")]
    pub text_soft: Color,
    #[serde(deserialize_with = "colour")]
    pub text_shadow: Color,
    #[serde(deserialize_with = "colour")]
    pub gold: Color,
    #[serde(deserialize_with = "colour")]
    pub gold_lit: Color,
    #[serde(deserialize_with = "colour")]
    pub on_gold: Color,
    #[serde(deserialize_with = "colour")]
    pub warn: Color,
    #[serde(deserialize_with = "colour")]
    pub warn_lit: Color,
    #[serde(deserialize_with = "colour")]
    pub dim: Color,
    #[serde(deserialize_with = "colour")]
    pub dialog: Color,
    #[serde(deserialize_with = "colour")]
    pub panel: Color,
    #[serde(deserialize_with = "colour")]
    pub panel_lit: Color,
    #[serde(deserialize_with = "colour")]
    pub quiet: Color,
    #[serde(deserialize_with = "colour")]
    pub quiet_lit: Color,
    #[serde(deserialize_with = "colour")]
    pub off: Color,
    #[serde(deserialize_with = "colour")]
    pub off_text: Color,
    #[serde(deserialize_with = "colour")]
    pub close: Color,
    #[serde(deserialize_with = "colour")]
    pub close_rim: Color,
    pub hud: HudColours,
    pub signup: SignupColours,
    pub settings: SettingsColours,
    pub house_builder: HouseBuilderColours,
    pub shop: ShopColours,
    pub talk: TalkColours,
    pub edit_menu: EditMenuColours,
    pub melee: MeleeColours,
    pub arena: ArenaColours,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct HudColours {
    #[serde(deserialize_with = "colour")]
    pub button: Color,
    #[serde(deserialize_with = "colour")]
    pub button_held: Color,
    #[serde(deserialize_with = "colour")]
    pub button_mark: Color,
    #[serde(deserialize_with = "colour")]
    pub name_plate: Color,
    #[serde(deserialize_with = "colour")]
    pub icon: Color,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignupColours {
    #[serde(deserialize_with = "colour")]
    pub page: Color,
    #[serde(deserialize_with = "colour")]
    pub field: Color,
    #[serde(deserialize_with = "colour")]
    pub field_lit: Color,
    #[serde(deserialize_with = "colour")]
    pub chosen: Color,
    #[serde(deserialize_with = "colour")]
    pub paper: Color,
    #[serde(deserialize_with = "colour")]
    pub paper_lit: Color,
    #[serde(deserialize_with = "colour")]
    pub ink: Color,
    #[serde(deserialize_with = "colour")]
    pub arrow: Color,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct SettingsColours {
    #[serde(deserialize_with = "colour")]
    pub window: Color,
    #[serde(deserialize_with = "colour")]
    pub blocked: Color,
    #[serde(deserialize_with = "colour")]
    pub blocked_text: Color,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct HouseBuilderColours {
    #[serde(deserialize_with = "colour")]
    pub star_off: Color,
    #[serde(deserialize_with = "colour")]
    pub results: Color,
    #[serde(deserialize_with = "colour")]
    pub row: Color,
    #[serde(deserialize_with = "colour")]
    pub your_row: Color,
    #[serde(deserialize_with = "colour")]
    pub silver: Color,
    #[serde(deserialize_with = "colour")]
    pub bronze: Color,
    #[serde(deserialize_with = "colour")]
    pub no_prize: Color,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShopColours {
    #[serde(deserialize_with = "colour")]
    pub window: Color,
    #[serde(deserialize_with = "colour")]
    pub hammer_rim: Color,
    #[serde(deserialize_with = "colour")]
    pub handle: Color,
    #[serde(deserialize_with = "colour")]
    pub card: Color,
    #[serde(deserialize_with = "colour")]
    pub card_lit: Color,
    #[serde(deserialize_with = "colour")]
    pub card_off: Color,
    #[serde(deserialize_with = "colour")]
    pub picture_off: Color,
    #[serde(deserialize_with = "colour")]
    pub name_off: Color,
    #[serde(deserialize_with = "colour")]
    pub note: Color,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct TalkColours {
    #[serde(deserialize_with = "colour")]
    pub ink: Color,
    #[serde(deserialize_with = "colour")]
    pub paper: Color,
    #[serde(deserialize_with = "colour")]
    pub paper_lit: Color,
    #[serde(deserialize_with = "colour")]
    pub greyed: Color,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct EditMenuColours {
    #[serde(deserialize_with = "colour")]
    pub icon: Color,
    #[serde(deserialize_with = "colour")]
    pub trash: Color,
    #[serde(deserialize_with = "colour")]
    pub trash_held: Color,
    #[serde(deserialize_with = "colour")]
    pub trash_rim: Color,
    #[serde(deserialize_with = "colour")]
    pub confirm: Color,
    #[serde(deserialize_with = "colour")]
    pub confirm_held: Color,
    #[serde(deserialize_with = "colour")]
    pub confirm_rim: Color,
    #[serde(deserialize_with = "colour")]
    pub turn: Color,
    #[serde(deserialize_with = "colour")]
    pub turn_held: Color,
    #[serde(deserialize_with = "colour")]
    pub turn_rim: Color,
}

/// The punch's hit range, the sector drawn round you on the ground
/// (`range`): its outline, and the fill inside it, each as see-through as
/// its alpha says.
#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct MeleeColours {
    #[serde(deserialize_with = "colour")]
    pub outline: Color,
    #[serde(deserialize_with = "colour")]
    pub fill: Color,
}

/// Bridge Wars' own (`arena`, and `hud`'s layout there): your health bar,
/// its fill, what shows behind it and its rim; the rim of an empty slot
/// among the buttons bottom right; a damage number; the names and the
/// health bars over everyone else's head, a teammate's and an enemy's; the
/// respawn anchors' bars, your team's and another's; the planks of the
/// bridges (`bridge`), their tops and their sides; the minimap; the
/// metres of bridge left as they are written in front of you while you
/// build, with plenty, with few and with next to none; the four things the
/// generators make, as their pictures are tinted beside their counts and
/// in the shop (`hud`, `market`); and the line that says what the
/// timetable has just done (`arena::alert`).
#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArenaColours {
    #[serde(deserialize_with = "colour")]
    pub hp: Color,
    #[serde(deserialize_with = "colour")]
    pub hp_back: Color,
    #[serde(deserialize_with = "colour")]
    pub hp_rim: Color,
    #[serde(deserialize_with = "colour")]
    pub slot_rim: Color,
    #[serde(deserialize_with = "colour")]
    pub damage: Color,
    #[serde(deserialize_with = "colour")]
    pub ally: Color,
    #[serde(deserialize_with = "colour")]
    pub enemy: Color,
    #[serde(deserialize_with = "colour")]
    pub hp_ally: Color,
    #[serde(deserialize_with = "colour")]
    pub hp_enemy: Color,
    #[serde(deserialize_with = "colour")]
    pub anchor_mine: Color,
    #[serde(deserialize_with = "colour")]
    pub anchor_enemy: Color,
    #[serde(deserialize_with = "colour")]
    pub plank: Color,
    #[serde(deserialize_with = "colour")]
    pub plank_side: Color,
    #[serde(deserialize_with = "colour")]
    pub map_land: Color,
    #[serde(deserialize_with = "colour")]
    pub map_back: Color,
    #[serde(deserialize_with = "colour")]
    pub map_me: Color,
    #[serde(deserialize_with = "colour")]
    pub map_mate: Color,
    #[serde(deserialize_with = "colour")]
    pub bridge_left: Color,
    #[serde(deserialize_with = "colour")]
    pub bridge_low: Color,
    #[serde(deserialize_with = "colour")]
    pub bridge_out: Color,
    /// The metres of bridge you have, written small inside the bridge
    /// button, under its plank (`hud::BridgeCount`).
    #[serde(deserialize_with = "colour")]
    pub bridge_count: Color,
    /// What the shops sell, as each one's picture is tinted on its card
    /// and on the second button (`market`, `utility`), and, for the
    /// shields, the ball of force round whoever has one (`gear`): the
    /// three swords, the three shields, the fireball, the three potions,
    /// the generator, the heal pool and the trap.
    #[serde(deserialize_with = "colour")]
    pub sword_basic: Color,
    #[serde(deserialize_with = "colour")]
    pub sword_silver: Color,
    #[serde(deserialize_with = "colour")]
    pub sword_aura: Color,
    #[serde(deserialize_with = "colour")]
    pub shield_basic: Color,
    #[serde(deserialize_with = "colour")]
    pub shield_advanced: Color,
    #[serde(deserialize_with = "colour")]
    pub shield_aura: Color,
    #[serde(deserialize_with = "colour")]
    pub fireball: Color,
    #[serde(deserialize_with = "colour")]
    pub potion_speed: Color,
    #[serde(deserialize_with = "colour")]
    pub potion_invisibility: Color,
    #[serde(deserialize_with = "colour")]
    pub potion_healing: Color,
    #[serde(deserialize_with = "colour")]
    pub heal_pool: Color,
    #[serde(deserialize_with = "colour")]
    pub trap: Color,
    /// The pips on an upgrade's card that say how far it has been taken:
    /// one taken, and one still to take.
    #[serde(deserialize_with = "colour")]
    pub pip: Color,
    #[serde(deserialize_with = "colour")]
    pub pip_off: Color,
    /// The band on the ground that shows where a fireball or a builder
    /// will go while you aim it (`utility`): its outline and its fill, as
    /// the punch's range has them (`colours.melee`), and the dark rim
    /// round it, which is what shows it over the sky (since 2026-11-12).
    #[serde(deserialize_with = "colour")]
    pub aim_outline: Color,
    #[serde(deserialize_with = "colour")]
    pub aim_fill: Color,
    #[serde(deserialize_with = "colour")]
    pub aim_rim: Color,
    /// The time left of a drink, written in front of you while a potion
    /// is at your mouth (`utility`).
    #[serde(deserialize_with = "colour")]
    pub drink: Color,
    /// What covers the screen, all but a small circle round you, while a
    /// trap has you blinded (`gear`).
    #[serde(deserialize_with = "colour")]
    pub blind: Color,
    #[serde(deserialize_with = "colour")]
    pub item_silver: Color,
    #[serde(deserialize_with = "colour")]
    pub item_gold: Color,
    #[serde(deserialize_with = "colour")]
    pub item_sapphire: Color,
    #[serde(deserialize_with = "colour")]
    pub item_amethyst: Color,
    #[serde(deserialize_with = "colour")]
    pub alert: Color,
    /// What is written over and under a sapphire or an amethyst generator
    /// (`arena::write_over_generators`).
    #[serde(deserialize_with = "colour")]
    pub generator: Color,
}

impl ArenaColours {
    /// The colour a sword's picture is tinted, by its tier, 1 to 3.
    pub(crate) fn sword(&self, tier: u8) -> Color {
        [self.sword_basic, self.sword_silver, self.sword_aura][usize::from(tier.clamp(1, 3)) - 1]
    }

    /// The colour a shield's picture is tinted, and the ball of force
    /// round whoever has it, by its tier, 1 to 3.
    pub(crate) fn shield(&self, tier: u8) -> Color {
        [self.shield_basic, self.shield_advanced, self.shield_aura][usize::from(tier.clamp(1, 3)) - 1]
    }

    /// The colour `item`'s picture is tinted, beside its count and in the
    /// shop.
    pub(crate) fn item(&self, item: roundtown_net::Item) -> Color {
        match item {
            roundtown_net::Item::Silver => self.item_silver,
            roundtown_net::Item::Gold => self.item_gold,
            roundtown_net::Item::Sapphire => self.item_sapphire,
            roundtown_net::Item::Amethyst => self.item_amethyst,
        }
    }
}

/// The words, screen by screen. One with `{something}` in it is filled in
/// with [`fill`].
#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Words {
    pub welcome: String,
    pub signup: SignupWords,
    pub settings: SettingsWords,
    pub house_builder: HouseBuilderWords,
    pub shop: ShopWords,
    pub talk: TalkWords,
    pub online: OnlineWords,
    pub games: GamesWords,
    pub arena: ArenaWords,
    pub market: MarketWords,
}

/// The game selector (`games`): its title, the three games, and what each
/// costs.
#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct GamesWords {
    pub title: String,
    pub bridge_wars: String,
    pub home_builders: String,
    pub coming_soon: String,
    pub coming_soon_note: String,
    pub free: String,
}

/// Bridge Wars' board, headlines and panels (`arena`).
#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArenaWords {
    pub name: String,
    pub players: String,
    pub waiting: String,
    pub match_found: String,
    pub fight: String,
    pub died: String,
    pub team: String,
    pub out: String,
    pub anchor_gone: String,
    pub anchor_destroyed: String,
    pub your_anchor_destroyed: String,
    pub victory: String,
    pub defeat: String,
    pub times_up: String,
    pub team_wins: String,
    pub play_again: String,
    pub exit: String,
    /// What the generators make, as each is called in a line about it.
    pub silver: String,
    pub gold: String,
    pub sapphire: String,
    pub amethyst: String,
    /// The lines that say what the timetable has just done, in the upper
    /// half of the screen: a generator's tier raised (`{item}`, and
    /// `{level}` as I, II or III), every anchor broken, and sudden death;
    /// and the line under "Time's up!" when nobody has won.
    pub tier_up: String,
    pub anchors_broken: String,
    pub sudden_death: String,
    pub draw: String,
    /// Over a sapphire or an amethyst generator, its level (`{item}`,
    /// `{level}` as 1, 2 or 3), and under it the count to its next thing
    /// (`{item}`, `{seconds}`).
    pub generator: String,
    pub generator_next: String,
    /// The lines said as a trap goes off: to you, blinded by one, and to
    /// the team whose trap it was.
    pub blinded: String,
    pub trap_sprung: String,
}

impl ArenaWords {
    /// What `item` is called.
    pub(crate) fn item(&self, item: roundtown_net::Item) -> &str {
        match item {
            roundtown_net::Item::Silver => &self.silver,
            roundtown_net::Item::Gold => &self.gold,
            roundtown_net::Item::Sapphire => &self.sapphire,
            roundtown_net::Item::Amethyst => &self.amethyst,
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignupWords {
    pub title: String,
    pub start: String,
    pub have_account: String,
    pub log_in: String,
    pub not_available: String,
    pub choose_name: String,
    pub name_hint: String,
    pub save: String,
    pub choose_character: String,
    pub change_later: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct SettingsWords {
    pub graphics: String,
    pub controls: String,
    pub accounts: String,
    pub not_signed_up: String,
    pub guest: String,
    pub not_online: String,
    pub sign_in_google: String,
    pub sign_in_apple: String,
    pub delete_account: String,
    pub edit_name: String,
    pub cancel: String,
    pub save: String,
    pub delete_sure: String,
    pub delete_warning: String,
    pub delete: String,
    pub deleting: String,
    pub not_online_try: String,
    pub could_not_delete: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct HouseBuilderWords {
    pub name: String,
    pub rules: String,
    pub pitch: String,
    pub cannot_pay: String,
    pub play: String,
    pub later: String,
    pub players: String,
    pub waiting: String,
    pub match_found: String,
    pub build_a_house: String,
    pub theme_is: String,
    pub who_lives_there: String,
    pub theme: String,
    pub your_house: String,
    pub a_house: String,
    pub whose_house: String,
    pub rate: String,
    pub being_rated: String,
    pub you_won: String,
    pub you_came: String,
    pub you: String,
    pub back_in: String,
    pub exit: String,
    pub play_again: String,
    pub price: String,
    pub need_coins: String,
    pub reconnecting: String,
    pub lost_connection: String,
}

/// Bridge Wars' two shops (`market`, since 2026-11-12): their titles; what
/// a card says in place of a price when the thing is yours already, when
/// there is no more of it to buy, and when the second button holds
/// something else; and each thing's name and the one line under it, in
/// which `{...}` is a number out of the rules (`roundtown_net::bridge_wars`).
/// A sword's and a shield's are by tier, a generator's by its step.
#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct MarketWords {
    pub items: String,
    pub upgrades: String,
    pub owned: String,
    pub max: String,
    pub taken: String,
    pub bridge: String,
    pub bridge_stat: String,
    pub swords: [String; 3],
    pub sword_stat: String,
    pub shields: [String; 3],
    pub shield_stat: String,
    pub fireball: String,
    pub fireball_stat: String,
    pub builder: String,
    pub builder_stat: String,
    pub speed: String,
    pub speed_stat: String,
    pub invisibility: String,
    pub invisibility_stat: String,
    pub healing: String,
    pub healing_stat: String,
    pub forge: [String; 4],
    pub forge_stats: [String; 4],
    pub stronger_swords: String,
    pub stronger_swords_stat: String,
    pub better_shields: String,
    pub better_shields_stat: String,
    pub heal_pool: String,
    pub heal_pool_stat: String,
    pub trap: String,
    pub trap_stat: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShopWords {
    pub one_house: String,
    pub house_first_openings: String,
    pub house_first_hangings: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct TalkWords {
    pub house_builder: String,
    pub someone: String,
    /// What the town's AIs say, and what you say back.
    pub small_talk: Vec<(String, String)>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct OnlineWords {
    pub new_version: String,
    pub update_to_play: String,
}

impl Ui {
    /// The file as compiled in. `cargo test` reads it, so a build whose tests
    /// pass cannot fail here.
    pub(crate) fn built_in() -> Self {
        read(BUILT_IN)
            .unwrap_or_else(|error| panic!("assets/ui.ron, as built in, will not read: {error}"))
    }
}

/// What `text` says, or what is wrong with it, with the line and column.
fn read(text: &str) -> Result<Ui, String> {
    let ui: Ui = ron::from_str(text).map_err(|error| error.to_string())?;
    if ui.words.talk.small_talk.is_empty() {
        return Err("talk.small_talk has nothing in it".into());
    }
    ui.car.check()?;
    ui.melee.check()?;
    ui.arena.check()?;
    Ok(ui)
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<UiChanged>();
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    app.init_resource::<Watch>().add_systems(PreUpdate, reread);
}

/// The file on disk, as last looked at.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Resource, Default)]
struct Watch {
    /// When it was last written, as of the last look: nothing until the
    /// first look, so that the file on disk is read as the app opens, in
    /// case it has changed since the build.
    written: Option<std::time::SystemTime>,
    /// What was wrong with it at the last look, if anything, so that the
    /// same thing is not logged four times a second.
    wrong: Option<String>,
    /// Seconds since it was last looked at.
    since: f32,
}

/// Reads the file again whenever it has been written since the last look,
/// and if what it says differs, makes that what [`ui`] gives and changes
/// [`UiChanged`]. A file that will not read is logged once, and looked at
/// again every look until it does, as an editor may have been caught halfway
/// through writing it.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn reread(time: Res<Time<Real>>, mut watch: ResMut<Watch>, mut changed: ResMut<UiChanged>) {
    watch.since += time.delta_secs();
    if watch.since < LOOK_EVERY && watch.written.is_some() {
        return;
    }
    watch.since = 0.0;
    let Ok(written) = std::fs::metadata(PATH).and_then(|file| file.modified()) else {
        return;
    };
    if watch.written == Some(written) && watch.wrong.is_none() {
        return;
    }
    watch.written = Some(written);
    let said = std::fs::read_to_string(PATH)
        .map_err(|error| error.to_string())
        .and_then(|text| read(&text));
    match said {
        Ok(new) => {
            if watch.wrong.take().is_some() {
                info!("assets/ui.ron reads again");
            }
            if *ui() != new {
                *CURRENT
                    .write()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Arc::new(new);
                changed.set_changed();
                info!("assets/ui.ron has changed: the screen is drawn from it again, and the car goes by it");
            }
        }
        Err(error) => {
            if watch.wrong.as_ref() != Some(&error) {
                warn!("assets/ui.ron will not read ({error}): the last good one is kept");
                watch.wrong = Some(error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_built_in_reads() {
        let ui = Ui::built_in();
        assert_eq!(ui.words.signup.title, "Donggeurami Town");
        assert!(ui.words.talk.small_talk.len() > 10);
        // Every word is plain ASCII, which is all the built-in font draws.
        for (says, back) in &ui.words.talk.small_talk {
            assert!(says.is_ascii() && back.is_ascii(), "{says}");
        }
        assert!(ui.words.settings.guest.is_ascii());
        // A colour's alpha is read: the dark behind a dialog is see-through.
        assert!(ui.colours.dim.alpha() < 1.0);
        assert_eq!(ui.colours.text.alpha(), 1.0);
        // The car's numbers are read, and hang together.
        assert!(ui.car.boost > ui.car.cruise && ui.car.cruise > ui.car.creep);
        assert!(ui.car.overhead_nearest < ui.car.overhead && ui.car.overhead < ui.car.overhead_furthest);
        // Bridge Wars' too: a run faster than a walk, and a punch that takes
        // several to kill.
        assert!(ui.arena.run_speed > crate::MOVE_SPEED);
        assert!(ui.arena.damage > 0.0 && ui.arena.damage < ui.arena.health);
        assert!(read(&BUILT_IN.replacen("camera_height: 15.5", "camera_height: 0", 1)).unwrap_err().contains("camera_height"));
        assert!(read(&BUILT_IN.replacen("run_speed: 10.0", "run_speed: 0", 1)).unwrap_err().contains("run_speed"));
        // A sword's tilt at the ready is degrees from straight up: none is
        // straight up, and past lying flat, or less than none, is refused,
        // whatever the file has just now.
        assert!(ui.arena.check().is_ok());
        for (forward, side, wrong) in [(0.0, 0.0, None), (90.0, 90.0, None), (120.0, 10.0, Some("sword_tilt_forward")), (30.0, 91.0, Some("sword_tilt_side")), (-5.0, 10.0, Some("sword_tilt_forward")), (30.0, f32::NAN, Some("sword_tilt_side"))] {
            let arena = Arena {
                sword_tilt_forward: forward,
                sword_tilt_side: side,
                ..ui.arena
            };
            match wrong {
                None => assert!(arena.check().is_ok(), "{forward} and {side}"),
                Some(field) => assert!(arena.check().unwrap_err().contains(field), "{forward} and {side}"),
            }
        }
    }

    #[test]
    fn a_car_number_the_car_would_break_on_will_not_read() {
        let with = |field: &str, value: &str| -> String {
            let line = BUILT_IN
                .lines()
                .find(|line| line.trim_start().starts_with(&format!("{field}:")))
                .unwrap_or_else(|| panic!("no car.{field} in the file"));
            let number = line.trim_start().strip_prefix(&format!("{field}:")).unwrap().trim().trim_end_matches(',');
            BUILT_IN.replacen(line, &line.replace(number, value), 1)
        };
        // Divided by.
        assert!(read(&with("easy_speed", "0.0")).unwrap_err().contains("easy_speed"));
        assert!(read(&with("full_lock_at", "0")).unwrap_err().contains("full_lock_at"));
        // A rate under 0 would panic a clamp; a gun firing every 0 s would
        // spawn without end.
        assert!(read(&with("brake", "-1.0")).unwrap_err().contains("brake"));
        assert!(read(&with("bullet_every", "0")).unwrap_err().contains("bullet_every"));
        assert!(read(&with("steer_rate", "NaN")).is_err());
        // Ranges inside out.
        assert!(read(&with("overhead", "100.0")).unwrap_err().contains("overhead"));
        assert!(read(&with("felt_high", "0.1")).unwrap_err().contains("felt_high"));
        assert!(read(&with("stopped_short", "2.0")).unwrap_err().contains("stopped_short"));
        // Any sensible number reads, written with a point or without.
        assert_eq!(read(&with("cruise", "15")).unwrap().car.cruise, 15.0);
        assert_eq!(read(&with("boost", "25.5")).unwrap().car.boost, 25.5);
        assert_eq!(read(&with("end_back", "0")).unwrap().car.end_back, 0.0);
    }

    #[test]
    fn a_file_that_will_not_read_says_what_is_wrong() {
        assert!(read("(colours: (").is_err());
        let misspelt = BUILT_IN.replacen("gold:", "glod:", 1);
        let error = read(&misspelt).unwrap_err();
        assert!(error.contains("glod") || error.contains("gold"), "{error}");
        let missing = BUILT_IN.replacen("        text: \"#ffffff\",\n", "", 1);
        assert!(read(&missing).unwrap_err().contains("text"));
        let not_a_colour = BUILT_IN.replacen("\"#ffffff\"", "\"white\"", 1);
        assert!(read(&not_a_colour).unwrap_err().contains("white"));
    }

    #[test]
    fn a_colour_reads_in_any_web_form() {
        let close = |a: Color, b: Color| {
            let (a, b) = (a.to_srgba(), b.to_srgba());
            [
                a.red - b.red,
                a.green - b.green,
                a.blue - b.blue,
                a.alpha - b.alpha,
            ]
            .iter()
            .all(|off| off.abs() < 0.01)
        };
        let gold = Color::srgba(1.0, 0.84, 0.3, 1.0);
        for text in [
            "#ffd64c",
            "#FFD64CFF",
            " #ffd64c ",
            "rgb(255, 214, 76)",
            "rgba(255, 214, 76, 1)",
            "rgb(255 214 76 / 100%)",
            "hsl(46, 100%, 65%)",
            "hsla(46, 100%, 65%, 1.0)",
        ] {
            let colour = parse_colour(text).unwrap();
            assert!(close(colour, gold), "{text}: {colour:?}");
        }
        // Alpha, as hex, as a number and as a percentage.
        assert!(close(
            parse_colour("#fff8").unwrap(),
            Color::srgba(1.0, 1.0, 1.0, 0.533)
        ));
        assert!(close(
            parse_colour("#00000073").unwrap(),
            Color::srgba(0.0, 0.0, 0.0, 0.45)
        ));
        assert!(close(
            parse_colour("rgba(0, 0, 0, 45%)").unwrap(),
            Color::srgba(0.0, 0.0, 0.0, 0.45)
        ));
        for bad in [
            "#ffd64",
            "gold",
            "rgb(255, 214)",
            "rgba(1, 2, 3, 4, 5)",
            "(1.0, 0.84, 0.3, 1.0)",
            "",
        ] {
            assert!(parse_colour(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_word_is_filled_in() {
        assert_eq!(
            fill("You came {place}", &[("place", "3rd")]),
            "You came 3rd"
        );
        assert_eq!(
            fill("{count}/{seats} Players", &[("count", "3"), ("seats", "8")]),
            "3/8 Players"
        );
        assert_eq!(fill("Play", &[("place", "3rd")]), "Play");
    }
}
