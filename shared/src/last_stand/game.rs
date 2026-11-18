//! A game of Last Stand as a plain struct ([`Game`]): no world, no network,
//! no map file. The server steps one online and the device one offline,
//! each feeding it the same things and carrying out what it says happened
//! ([`Event`]), so that the two never differ ("one rule, two runners", as
//! Bridge Wars' rules are run). Everything a person does is checked as if it
//! came from a cheat: nothing is free, and nothing is believed that could
//! not be.

use bevy::math::{Quat, Vec2, Vec3, Vec3Swizzles};

use super::forges::Forges;
use super::map::{CRYSTAL_RADIUS, OBSTACLES, RIM, SEA_BED, SHORE, STONE_RADIUS, at_a_stand, ground};
use super::shop::{self, BEAM_DAMAGE, FORTIFY, Ware, blow, sharpened, through};
use super::waves::{BREAK, FIRST_WAVE, Kind, MOST_STANDING, drop_of, first_in, kill_score, mix, rising, strength, toughness};
use super::zombies::{Act, Seen, SPIT_RADIUS, Spittle, View, Zombie};
use super::{ANCHOR, Find, StandRound, StandStage, StandUpgrades, arrival};
use crate::bots::Dice;
use crate::bridge_wars::{
    BLAST_HEIGHT, BLAST_RADIUS, DYNAMITE_RADIUS, FIREBALL_RADIUS, Flight, HEALING, HOVER, Lob, MAGNET, Piece, REACH_OVER,
    REACH_UNDER, SPILL_FAR, SPILL_FROM, SPILL_NEAR, SPILL_WAIT, caught, toss,
};
use crate::nav::BODY;
use crate::{Item, Kit, Purse, Use, Utility};

/// How long what is dropped lies before it is gone, in seconds: what the
/// dead drop, and what a zombie does (since 2026-11-18; Hajun: "the items
/// despawns after a very long time"; mine, Minecraft's five minutes, a
/// third of a game or more). A treasure chest lies as long, unopened.
pub const LIES_FOR: f32 = 300.0;
/// The most things of one item someone who dies drops, each more than one
/// of it past this ([`Piece::worth`]) and drawn the bigger for it (the
/// game's `loot::size`), and how thickly what they drop lies, in things to
/// the square metre: a purse of a hundred lies in a ring of sand three and
/// a half metres out, of two hundred four (mine; a heap in Bridge Wars'
/// 2.4 m, at its fifty a square metre, was one lump). By Hajun's rule (since
/// 2026-11-18): "If and only if dropping like 200 silver at once will cause
/// severe lag, then make some visual merging technique for nearby drops so
/// it still looks like a lot but not Lag inducing quantity - if performance
/// is not that severly affected its ok to just drop 200 of it". Measured
/// that evening on the desktop, hidden: two hundred of each of the four,
/// 794 things bobbing and turning, kept it at 120 frames a second (8.3 ms
/// a frame before, 8.4 with them lying, twice), though an earlier run, with
/// the computer slower at everything, went from 18 ms to 22. No phone was
/// measured. Should one lag, fifty here is the merging, measured too: a
/// quarter of the things, each worth four and drawn half again as big, in
/// as wide a ring (by worth, not by things).
pub const DROP_MOST: u16 = 200;
const DROP_DENSITY: f32 = 4.0;
/// The furthest out what the dead drop lies, in metres.
const DROP_FAR: f32 = 6.0;
/// When the first treasure chest comes, in seconds into the fight, and how
/// long after one has gone, opened or lain its while, the next comes, at
/// the least and the most, thrown for each; and how near someone has to be
/// to open one, as the game believes it (the bubble comes up over it from
/// 4.5 m, the town's talking reach, and the rest is for the way they may
/// have gone since). Mine (since 2026-11-18; Hajun: "make treasurechest.glb
/// that sometimes spawn in the sand area or the ocean, with no notification
/// to the player of its location or existence ... it should a minor thing
/// not the main thing").
const CHEST_FIRST: (f32, f32) = (45.0, 120.0);
const CHEST_EVERY: (f32, f32) = (75.0, 160.0);
pub const OPEN_REACH: f32 = 6.5;

/// A sword lying where someone who died dropped it (since 2026-11-18;
/// Hajun: "they should drop all their loot (silver,gold,sapphire,
/// amethyst, sword) ... for sword, can only pick up if its greater than
/// the sword they are already holding"): its number, which sword (1 to 3),
/// where it lies across the ground and how high what it lies on is, where
/// it was thrown from, and when, in seconds into the fight. It flies out
/// and lies as what they held does ([`Piece`]), for whoever walks into it
/// holding a lesser sword, themselves once they stand again among them.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Blade {
    pub id: u32,
    pub tier: u8,
    pub at: Vec2,
    pub ground: f32,
    pub from: Vec3,
    pub born: f32,
}

impl Blade {
    /// Where its middle hangs once it has come down.
    pub fn rest(&self) -> Vec3 {
        Vec3::new(self.at.x, self.ground + HOVER, self.at.y)
    }

    /// How high its middle is `elapsed` seconds into the fight: on its way
    /// from whoever dropped it, and then where it hangs.
    pub fn height(&self, elapsed: f32) -> f32 {
        toss(self.from, self.rest(), elapsed - self.born).y
    }

    /// Whether it has come down and lain a moment, for anyone to have it.
    pub fn lying(&self, elapsed: f32) -> bool {
        elapsed - self.born >= SPILL_WAIT
    }
}

/// A treasure chest lying on the sand or in the sea, unannounced, for
/// whoever finds it to open (since 2026-11-18): its number, where it
/// stands, which way it faces, and when it came, in seconds into the fight.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Treasure {
    pub id: u32,
    pub at: Vec3,
    pub yaw: f32,
    pub born: f32,
}

/// Everyone's health, as in Bridge Wars ([`HEALTH`](crate::bridge_wars::HEALTH)):
/// how much they have, and how long after the last hit it starts to come
/// back, and how much a second.
pub const HEALTH: f32 = 100.0;
pub const REGEN_AFTER: f32 = 5.0;
pub const REGEN_PER_SEC: f32 = 4.0;
/// The anchor's health as a game begins: mine, half as much again as the
/// design's 1000, which fell to the first brutes while a person alone was
/// down for their six seconds, in half the stand-ins' games.
pub const ANCHOR_HEALTH: f32 = 1500.0;
/// How long the dead are dead before they stand beside the anchor again,
/// while it stands, in seconds; and for how long after, nothing turns on
/// them (mine: or they would come back into the heap at the anchor and go
/// down again at once).
pub const RESPAWN_SECS: f32 = 6.0;
pub const GRACE_SECS: f32 = 2.0;
/// How long after a blow by someone lands on a zombie another blow of
/// theirs counts, in seconds: Bridge Wars' (Minecraft's ten ticks), but each
/// person's own, so that four on a giant are four times one (mine).
pub const HURT_TIME: f32 = crate::bridge_wars::HURT_TIME;
/// How far apart a person and a zombie can be, where the person last said
/// they stood and where the zombie is, for a blow the person's device says
/// landed to count, in metres: the reach and the way each may have gone
/// since, as Bridge Wars' server allows.
pub const PUNCH_SLACK: f32 = 6.5;
/// How long after a zombie was last hit its fall into the void is the
/// hitter's kill, in seconds ([`chat::CREDIT_SECS`](crate::chat::CREDIT_SECS)).
pub const CREDIT_SECS: f32 = crate::chat::CREDIT_SECS;
/// How soon the next wave comes once everyone standing is ready
/// ([`Game::ready`]), in seconds.
pub const READY_SECS: f32 = 3.0;
/// How often the anchor's beam zaps, in seconds, and how far from the
/// anchor a zombie may be to be zapped, in metres (the design's).
pub const BEAM_EVERY: f32 = 2.0;
pub const BEAM_REACH: f32 = 12.0;
/// What the team's score has for each wave it cleared, beside everyone's
/// own (the design's).
pub const WAVE_SCORE: u32 = 100;
/// How long each potion lasts, in seconds: the speed potion (the design's
/// 30, where Bridge Wars' is 45), the invisibility (20, where Bridge Wars'
/// is 30) and the leap (Bridge Wars' 45).
pub const SPEED_SECS: f32 = 30.0;
pub const UNSEEN_SECS: f32 = 20.0;
pub const LEAP_SECS: f32 = crate::bridge_wars::LEAP_SECS;
/// What a fireball's blast and a stick of dynamite's take off every zombie
/// they reach, before the team's Sharpness (the design's).
pub const FIREBALL_HIT: f32 = 40.0;
pub const DYNAMITE_HIT: f32 = 120.0;
/// The longest frame the game takes, in seconds: a longer one is played as
/// this long, so that nothing jumps through anything after a stall.
const LONGEST_STEP: f32 = 0.25;
/// How many seats there are at the most: a bit to each in what is sent.
const MOST_SEATS: usize = 8;

/// How hard a zombie's blow knocks whoever it lands on, for their device to
/// knock them by (online, [`Tell::Knocked`](crate::Tell::Knocked)'s
/// `running`; [`Kind::knocks`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Knock {
    /// Not at all: the small ones' blows, and a spitter's ball, which
    /// splashes.
    None,
    /// As a punch thrown standing knocks: a brute's.
    Light,
    /// As one thrown on the run: a giant's, and a leaper landing.
    Heavy,
}

/// Something that happened in a game of Last Stand, for whoever runs it to
/// carry out: what to send, spawn, despawn, draw and say. In the order it
/// happened, frame by frame ([`Game::step`]).
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    /// Wave `wave` has begun: `count` of it to come over the rim, and the
    /// kinds that come for the first time in it, for the headline ("Wave 7",
    /// "Brutes!").
    Wave { wave: u16, count: u16, new: Vec<Kind> },
    /// Wave `wave` is cleared: the break before the next.
    Cleared { wave: u16 },
    /// A zombie has begun to climb up over the rim: its number, its kind,
    /// where it is and which way it faces. Its body is the world's to make
    /// ([`Shambler`](super::Shambler)), moved where [`Game::zombies`] says.
    Risen { zombie: u32, kind: Kind, at: Vec3, yaw: f32 },
    /// A zombie was hurt, by the person in seat `by`, or by the anchor's
    /// beam: how much.
    Hurt { zombie: u32, damage: f32, by: Option<usize> },
    /// A zombie is dead, killed by the person in seat `by`, if anyone (the
    /// last blow; into the void, whoever knocked it there just before), at
    /// `at`: into the void, or where it fell, and what it drops comes as
    /// [`Event::Made`]s after this. Its body is to go. One that falls into
    /// the void drops nothing, nor does one killed out over it once it has
    /// fallen more than a step under the sand (what it drops comes down only
    /// on ground no higher over its feet than that,
    /// [`Pile::scatter`](crate::bridge_wars::Pile::scatter)); one killed
    /// climbing up the rim's side drops on the sand over it, from where its
    /// climb comes up.
    Killed { zombie: u32, kind: Kind, by: Option<usize>, at: Vec3, void: bool },
    /// A zombie's blow, or a spitter's ball, has landed on the person in
    /// `seat`: how much it took, after their shield and the team's
    /// protection, and how hard it knocks them, away from `from`, their own
    /// device's to do: a zombie's middle, where a leaper leapt from as it
    /// lands on them (its way on), and where a ball came down.
    Struck { seat: usize, damage: f32, from: Vec3, knock: Knock },
    /// The person in `seat` died where they stood, `at`; `out`, for good,
    /// the anchor being broken.
    Died { seat: usize, at: Vec3, out: bool },
    /// The person in `seat`, dead and waiting to stand again, is out for
    /// good: the anchor broke meanwhile.
    Out { seat: usize },
    /// The person in `seat` stands again beside the anchor, here, facing
    /// this way: whoever runs the game puts their body there.
    Back { seat: usize, at: Vec3, yaw: f32 },
    /// The anchor was hit, for so much: the device flashes it.
    AnchorHurt { damage: f32 },
    /// The anchor is broken: nobody stands again from now on.
    AnchorBroken,
    /// The anchor's beam zapped a zombie, standing here, for so much.
    Zap { zombie: u32, at: Vec3, damage: f32 },
    /// A spitter spat a ball, numbered `spit`, for the room to be sent
    /// ([`Spit`](super::Spit)).
    Spat { spit: u32, spittle: Spittle },
    /// The ball numbered `spit` came down here and splashed: the room is
    /// sent it no longer.
    Splat { spit: u32, at: Vec3 },
    /// The person in `seat` threw a fireball, numbered `ball`
    /// ([`Fireball`](crate::Fireball) for the room).
    Thrown { ball: u32, seat: usize, flight: Flight },
    /// The fireball numbered `ball` burst here.
    Burst { ball: u32, at: Vec3 },
    /// The person in `seat` threw a stick of dynamite, numbered `stick`
    /// ([`Dynamite`](crate::Dynamite) for the room).
    Lobbed { stick: u32, seat: usize, lob: Lob },
    /// The stick numbered `stick` blew up here.
    Blew { stick: u32, at: Vec3 },
    /// The stick numbered `stick` came down with no ground under it and is
    /// gone into the void, unblown.
    Sank { stick: u32 },
    /// A thing has come to lie for whoever walks into it: made by a
    /// generator, or dropped by a zombie killed ([`Piece::dropped`]); the
    /// room is sent it ([`Loot::of`](crate::Loot::of)).
    Made(Piece),
    /// The person in `seat` picked this up: it is in their purse, and no
    /// longer lies there for anyone.
    Taken { seat: usize, piece: Piece },
    /// The person in `seat` bought this: theirs ([`Tell::Bought`](super::Tell::Bought)),
    /// or the team's, which has it to `level` now, for everyone to be told
    /// ([`Tell::Upgraded`](super::Tell::Upgraded)).
    Bought { seat: usize, ware: Ware, level: u16 },
    /// The person in `seat` drank this potion (a healing one has given
    /// them their health already).
    Drank { seat: usize, potion: Utility },
    /// Someone who died dropped their sword, to lie here ([`Blade`]); what
    /// they held of the four things is [`Event::Made`], each thing thrown
    /// out from them.
    SwordDropped(Blade),
    /// The person in `seat` walked into this sword, holding a lesser one:
    /// it is theirs now, and lies there no longer.
    SwordTaken { seat: usize, blade: Blade },
    /// What was dropped has lain its while ([`LIES_FOR`]) and is gone.
    Lapsed(Piece),
    SwordLapsed(Blade),
    /// A treasure chest has come to stand here, said to nobody: the room is
    /// sent it, and whoever comes upon it may open it.
    ChestPlaced(Treasure),
    /// The person in `seat` opened this chest ([`Game::open`]): what came
    /// out of it is theirs now ([`Find`]), and the chest is gone.
    ChestOpened { seat: usize, chest: Treasure, find: Find },
    /// A chest nobody opened has stood its while and is gone.
    ChestLapsed(Treasure),
    /// The anchor is broken and nobody is standing, or everyone has gone:
    /// the game is over, `waves` cleared ([`Game::outcome`]).
    Over { waves: u16 },
}

/// One seat of a game of Last Stand.
#[derive(Clone, Debug)]
struct Player {
    /// Nobody's: a seat no person sat in.
    vacant: bool,
    /// Left for good: the seat stays theirs for the results.
    gone: bool,
    at: Vec3,
    yaw: f32,
    health: f32,
    since_hit: f32,
    /// Dead: how much longer before they stand again.
    dead_for: Option<f32>,
    /// Out for good: dead with the anchor broken.
    out: bool,
    /// When they stand again, by the room's clock: 0 standing.
    back_at: f64,
    /// How much longer nothing turns on them, just back.
    grace: f32,
    /// What their kills are worth, and how much damage they have dealt.
    kills: u32,
    dealt: f32,
    ready: bool,
    /// The last place they stood on the land, as their device said: where
    /// what they hold is dropped if they die off it, in the sea.
    footing: Vec3,
}

/// How far a game has got.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    /// Not begun: waiting at the table, or counting down.
    Before,
    /// Between waves: the next comes `ends` seconds into the fight, which is
    /// `until` by the room's clock.
    Break { ends: f32, until: f64 },
    Wave,
    Over,
}

/// Where one of a wave's places round the rim is, and when it next sends a
/// zombie over, and how many more before it pauses.
#[derive(Clone, Copy, Debug)]
struct Place {
    angle: f32,
    wait: f32,
    burst: u8,
}

/// A game of Last Stand, from the countdown's end to the last stand's fall:
/// the server's online, the device's own offline ([`crate::last_stand`]).
#[derive(Clone, Debug)]
pub struct Game {
    dice: Dice,
    elapsed: f32,
    clock: f64,
    /// When the fight began, by the room's clock ([`StandRound::began`]).
    began: f64,
    stage: Stage,
    players: Vec<Player>,
    // (These few are the module's, for its tests to set a game up far on.)
    pub(super) purses: Vec<Purse>,
    pub(super) kits: Vec<Kit>,
    pub(super) upgrades: StandUpgrades,
    pub(super) anchor: f32,
    pub(super) anchor_most: f32,
    beam_in: f32,
    pub(super) wave: u16,
    cleared: u16,
    /// The wave's zombies still to come, the next last.
    queue: Vec<Kind>,
    places: Vec<Place>,
    zombies: Vec<Zombie>,
    forges: Forges,
    spits: Vec<(u32, Spittle, f32)>,
    balls: Vec<(u32, Flight, usize)>,
    sticks: Vec<(u32, Lob, usize, bool)>,
    /// Swords the dead dropped, lying about.
    blades: Vec<Blade>,
    /// The treasure chest standing, if one is, and when the next comes, in
    /// seconds into the fight.
    chests: Vec<Treasure>,
    pub(super) chest_at: f32,
    next: [u32; 6],
    events: Vec<Event>,
}

impl Game {
    /// A new game at a table of `seats` seats, `people[seat]` false for a
    /// seat nobody sat in (Last Stand has no computer players), its dice
    /// thrown from `seed`: the same seed and the same doings, the same game.
    pub fn new(seats: usize, people: &[bool], seed: u64) -> Self {
        let seats = seats.max(people.len()).min(MOST_SEATS);
        let players = (0..seats)
            .map(|seat| {
                let (at, turn) = arrival(seat);
                let person = people.get(seat).copied().unwrap_or(false);
                Player {
                    vacant: !person,
                    gone: false,
                    at,
                    yaw: yaw_of(turn),
                    health: HEALTH,
                    since_hit: 0.0,
                    dead_for: None,
                    out: false,
                    back_at: if person { 0.0 } else { f64::INFINITY },
                    grace: 0.0,
                    kills: 0,
                    dealt: 0.0,
                    ready: false,
                    footing: at,
                }
            })
            .collect();
        let mut dice = Dice::new(seed);
        let chest_at = dice.around(CHEST_FIRST.0, CHEST_FIRST.1);
        Self {
            dice,
            elapsed: 0.0,
            clock: 0.0,
            began: 0.0,
            stage: Stage::Before,
            players,
            purses: vec![Purse::default(); seats],
            kits: vec![Kit::default(); seats],
            upgrades: StandUpgrades::default(),
            anchor: ANCHOR_HEALTH,
            anchor_most: ANCHOR_HEALTH,
            beam_in: BEAM_EVERY,
            wave: 0,
            cleared: 0,
            queue: Vec::new(),
            places: Vec::new(),
            zombies: Vec::new(),
            forges: Forges::new(),
            spits: Vec::new(),
            balls: Vec::new(),
            sticks: Vec::new(),
            blades: Vec::new(),
            chests: Vec::new(),
            chest_at,
            next: [0; 6],
            events: Vec::new(),
        }
    }

    /// The room's clock now, in seconds since 1970: the server's online, the
    /// town's offline (the game's `clock`). What the next wave and anyone
    /// standing again are told in ([`StandRound`]), so that every device
    /// counts down to them with nothing more sent; the game itself goes by
    /// its own time. Said before every [`Game::step`], and before
    /// [`Game::begin`]; never said, as in a test, the game's own time is
    /// used.
    pub fn clock(&mut self, now: f64) {
        self.clock = now;
    }

    /// The countdown is over: everyone stands where they arrive
    /// ([`arrival`]), and the first wave comes [`FIRST_WAVE`] from now.
    pub fn begin(&mut self) {
        if self.stage != Stage::Before {
            return;
        }
        self.elapsed = 0.0;
        self.began = self.on_clock(0.0);
        for (seat, player) in self.players.iter_mut().enumerate() {
            let (at, turn) = arrival(seat);
            player.at = at;
            player.yaw = yaw_of(turn);
            player.footing = at;
        }
        self.stage = Stage::Break {
            ends: FIRST_WAVE,
            until: self.on_clock(FIRST_WAVE),
        };
    }

    /// For trying the game out (the server's and the game's `RT_STAND_WAVE`,
    /// never on a server real players use): the next wave to come is
    /// `wave`, as if the team had cleared every one before it, which it is
    /// paid for and scores as if it had. Before [`Game::begin`], or in a
    /// break; nothing otherwise.
    pub fn from_wave(&mut self, wave: u16) {
        if !matches!(self.stage, Stage::Before | Stage::Break { .. }) {
            return;
        }
        let before = wave.max(1) - 1;
        self.wave = before;
        self.cleared = before;
    }

    /// For trying the game out (`RT_STAND_PURSE`, as Bridge Wars'
    /// `RT_ARENA_PURSE`): everyone holds `count` of each of the four
    /// things, whatever they held.
    pub fn fill_purses(&mut self, count: u16) {
        for purse in &mut self.purses {
            *purse = [count; 4];
        }
    }

    /// Where the person in `seat` is now, as their device last said; and,
    /// on the land, the last place they stood on it.
    pub fn sync(&mut self, seat: usize, at: Vec3, yaw: f32) {
        if let Some(player) = self.players.get_mut(seat)
            && !player.vacant
            && !player.gone
            && at.is_finite()
            && yaw.is_finite()
        {
            player.at = at;
            player.yaw = yaw;
            if let Some(top) = ground(at.xz())
                && (at.y - top).abs() <= 0.6
            {
                player.footing = Vec3::new(at.x, top, at.z);
            }
        }
    }

    /// The person in `seat` opens the treasure chest numbered `chest`: if
    /// the game is on, they stand, and it is there within their reach as
    /// their device last said ([`OPEN_REACH`]). What comes out is theirs at
    /// once ([`Find`], [`Event::ChestOpened`]), and the chest is gone.
    /// Whether it opened.
    pub fn open(&mut self, seat: usize, chest: u32) -> bool {
        if !self.playing() || !self.alive(seat) {
            return false;
        }
        let at = self.players[seat].at;
        let Some(index) = self.chests.iter().position(|it| it.id == chest && it.at.xz().distance(at.xz()) <= OPEN_REACH) else {
            return false;
        };
        let chest = self.chests.remove(index);
        let find = self.find(seat);
        self.give(seat, find);
        self.chest_at = self.elapsed + self.dice.around(CHEST_EVERY.0, CHEST_EVERY.1);
        self.events.push(Event::ChestOpened { seat, chest, find });
        true
    }

    /// The person in `seat` says their blow landed on the zombie numbered
    /// `zombie`, thrown on the run if `running`: it counts if they are
    /// standing in the fight, it is there and near enough to where they last
    /// said they stood ([`PUNCH_SLACK`]), and their last blow on it was not
    /// just now ([`HURT_TIME`]). It takes their sword's damage, sharpened
    /// ([`blow`]), knocks it away from them by its kind's share, and turns
    /// it on them; and whoever was unseen is seen again for attacking.
    /// Whether it counted.
    pub fn punch(&mut self, seat: usize, zombie: u32, running: bool) -> bool {
        if !self.playing() || !self.alive(seat) {
            return false;
        }
        let Some(index) = self.zombies.iter().position(|it| it.id == zombie && !it.dead) else {
            return false;
        };
        let from = self.players[seat].at;
        let it = &mut self.zombies[index];
        if it.at.xz().distance(from.xz()) > PUNCH_SLACK + it.radius() - BODY || it.hurt.get(seat).is_none_or(|left| *left > 0.0) {
            return false;
        }
        it.hurt[seat] = HURT_TIME;
        it.knock(from.xz(), running, 1.0);
        it.turn_on(seat);
        self.reveal(seat);
        let damage = blow(&self.kits[seat], &self.upgrades);
        self.hurt(index, damage, Some(seat));
        true
    }

    /// Sells `ware` to the person in `seat`, standing at `at` across the
    /// ground, if the game is on, they are standing, at the stand that sells
    /// it ([`at_a_stand`]), and it is to be had by them for what they hold
    /// ([`shop::buy`]); a Fortify only while the anchor stands, which it
    /// heals whole. Whether it is theirs, or the team's.
    pub fn buy(&mut self, seat: usize, ware: Ware, at: Vec2) -> bool {
        if !self.playing() || !self.alive(seat) || !at.is_finite() || !at_a_stand(at, shop::sold_at(ware)) {
            return false;
        }
        if ware == Ware::Fortify && self.anchor <= 0.0 {
            return false;
        }
        if !shop::buy(ware, &mut self.purses[seat], &mut self.kits[seat], &mut self.upgrades) {
            return false;
        }
        if ware == Ware::Fortify {
            self.anchor_most = (self.anchor_most + FORTIFY).min(f32::from(u16::MAX));
            self.anchor = self.anchor_most;
        }
        let level = shop::level(ware, &self.upgrades);
        self.events.push(Event::Bought { seat, ware, level });
        true
    }

    /// The person in `seat` has used what is on one of their buttons: a
    /// fireball thrown that way, a stick of dynamite lobbed that way and
    /// that far, or a potion drunk (their device having taken its time over
    /// it, as in Bridge Wars). Whether they had one and it was used.
    pub fn use_item(&mut self, seat: usize, used: Use) -> bool {
        if !self.playing() || !self.alive(seat) {
            return false;
        }
        let at = self.players[seat].at;
        match used {
            Use::Fireball { heading } => {
                if !heading.is_finite() || !spend(&mut self.kits[seat], Utility::Fireball) {
                    return false;
                }
                let flight = Flight::thrown(at, heading);
                let ball = self.number(1);
                self.balls.push((ball, flight, seat));
                self.events.push(Event::Thrown { ball, seat, flight });
            }
            Use::Dynamite { heading, far } => {
                if !heading.is_finite() || !far.is_finite() || !spend(&mut self.kits[seat], Utility::Dynamite) {
                    return false;
                }
                // At the ground where it is to come down, or, with none
                // there, as high as the thrower stands.
                let aimed = Lob::thrown(at, heading, far, at.y);
                let floor = ground(aimed.to.xz()).or(ground(at.xz())).unwrap_or(at.y);
                let lob = Lob::thrown(at, heading, far, floor);
                let stick = self.number(2);
                self.sticks.push((stick, lob, seat, false));
                self.events.push(Event::Lobbed { stick, seat, lob });
            }
            Use::Potion(potion) => {
                if potion.aimed() || !spend(&mut self.kits[seat], potion) {
                    return false;
                }
                let until = |secs: f32| (self.elapsed + secs).ceil().min(f32::from(u16::MAX)) as u16;
                let (fast, unseen, leap) = (until(SPEED_SECS), until(UNSEEN_SECS), until(LEAP_SECS));
                let kit = &mut self.kits[seat];
                match potion {
                    Utility::Speed => kit.fast_until = fast,
                    Utility::Invisibility => kit.unseen_until = unseen,
                    Utility::Leap => kit.leap_until = leap,
                    Utility::Healing => {
                        let player = &mut self.players[seat];
                        player.health = (player.health + HEALTH * HEALING).min(HEALTH);
                    }
                    Utility::Fireball | Utility::Builder | Utility::Dynamite => {}
                }
                self.events.push(Event::Drank { seat, potion });
            }
            Use::Builder => return false,
        }
        true
    }

    /// The person in `seat` is ready for the next wave: once everyone
    /// standing is, it comes in [`READY_SECS`], if not sooner. Looked at
    /// again at every step too, so that whoever was not ready leaving, or
    /// dying, leaves the rest no longer waiting for them.
    pub fn ready(&mut self, seat: usize) {
        if !matches!(self.stage, Stage::Break { .. }) || !self.alive(seat) {
            return;
        }
        self.players[seat].ready = true;
        self.hurry();
    }

    /// The person in `seat` has fallen into the void, as their device says:
    /// dead, by nobody's doing.
    pub fn fell(&mut self, seat: usize) {
        if self.playing() && self.alive(seat) {
            let at = self.players[seat].at;
            self.die(seat, at);
        }
    }

    /// The person in `seat` has left for good: nothing goes for them, and
    /// their score stays theirs.
    pub fn leave(&mut self, seat: usize) {
        if let Some(player) = self.players.get_mut(seat) {
            player.gone = true;
            player.ready = false;
            player.dead_for = None;
            player.back_at = f64::INFINITY;
        }
    }

    /// A step of play, `dt` seconds of it: everything that happened since the
    /// last, for whoever runs the game to carry out, in order.
    pub fn step(&mut self, dt: f32) -> Vec<Event> {
        if !matches!(self.stage, Stage::Break { .. } | Stage::Wave) {
            return std::mem::take(&mut self.events);
        }
        let dt = dt.clamp(0.0, LONGEST_STEP);
        self.elapsed += dt;
        self.stand_again(dt);
        self.hurry();
        self.come(dt);
        self.shamble(dt);
        self.splash(dt);
        self.fly(dt);
        self.dynamite(dt);
        self.beam(dt);
        self.forge();
        self.gather();
        self.lapse();
        self.place_chest();
        self.potions();
        self.bury();
        self.clear();
        self.end();
        std::mem::take(&mut self.events)
    }

    /// Whether the game is being played: begun and not over.
    pub fn playing(&self) -> bool {
        matches!(self.stage, Stage::Break { .. } | Stage::Wave)
    }

    /// Whether it is over.
    pub fn over(&self) -> bool {
        self.stage == Stage::Over
    }

    /// How long it has been played, in seconds: the fight's time, which
    /// everyone's kit goes by ([`Kit::fast`]), as Bridge Wars' does.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// The wave on now, or the last.
    pub fn wave(&self) -> u16 {
        self.wave
    }

    /// How many waves the team has cleared.
    pub fn cleared(&self) -> u16 {
        self.cleared
    }

    /// The anchor's health now, and the most it has.
    pub fn anchor(&self) -> (f32, f32) {
        (self.anchor, self.anchor_most)
    }

    /// Whether the person in `seat` is standing in the fight: in the game,
    /// alive, and not out.
    pub fn alive(&self, seat: usize) -> bool {
        self.players
            .get(seat)
            .is_some_and(|player| !player.vacant && !player.gone && !player.out && player.dead_for.is_none())
    }

    /// The health of the person in `seat`: none, dead.
    pub fn health(&self, seat: usize) -> f32 {
        if self.alive(seat) { self.players[seat].health } else { 0.0 }
    }

    /// Where the body in each seat is and which way it faces: a person's as
    /// their device last said, or where the game has put them (as they
    /// begin, [`Event::Back`]).
    pub fn at(&self, seat: usize) -> (Vec3, f32) {
        self.players.get(seat).map_or((Vec3::ZERO, 0.0), |player| (player.at, player.yaw))
    }

    /// What everyone holds, by seat (the room's [`Purses`](crate::Purses)).
    pub fn purses(&self) -> &[Purse] {
        &self.purses
    }

    /// What everyone has, by seat (the room's [`Kits`](crate::Kits)).
    pub fn kits(&self) -> &[Kit] {
        &self.kits
    }

    /// What the team has bought at the upgrade shop.
    pub fn upgrades(&self) -> StandUpgrades {
        self.upgrades
    }

    /// Everything lying about, for whoever walks into it.
    pub fn pieces(&self) -> &[Piece] {
        self.forges.pile().pieces()
    }

    /// Every sword the dead dropped, lying about.
    pub fn swords(&self) -> &[Blade] {
        &self.blades
    }

    /// The treasure chests standing, unopened.
    pub fn chests(&self) -> &[Treasure] {
        &self.chests
    }

    /// Every zombie, where it is: its number, its kind, its feet, which way
    /// it faces, and its health and the most it had, in whole points
    /// ([`Shambler`](super::Shambler)). In the order they rose.
    pub fn zombies(&self) -> impl Iterator<Item = (u32, Kind, Vec3, f32, u16, u16)> + '_ {
        self.zombies
            .iter()
            .filter(|zombie| !zombie.dead)
            .map(|zombie| (zombie.id, zombie.kind, zombie.at, zombie.yaw, whole(zombie.health), whole(zombie.most)))
    }

    /// Whether the zombie numbered `zombie` is climbing over the rim still.
    pub fn climbing(&self, zombie: u32) -> bool {
        self.zombies.iter().any(|it| it.id == zombie && it.climbing())
    }

    /// Every spitter's ball on its way, by number.
    pub fn spits(&self) -> impl Iterator<Item = (u32, Spittle)> + '_ {
        self.spits.iter().map(|(spit, spittle, _)| (*spit, *spittle))
    }

    /// Every fireball in the air, by number, and whose.
    pub fn fireballs(&self) -> impl Iterator<Item = (u32, Flight, usize)> + '_ {
        self.balls.iter().copied()
    }

    /// Every stick of dynamite on its way or lying with its fuse burning, by
    /// number, and whose.
    pub fn sticks(&self) -> impl Iterator<Item = (u32, Lob, usize)> + '_ {
        self.sticks.iter().map(|(stick, lob, seat, _)| (*stick, *lob, *seat))
    }

    /// Everyone's score by seat: what their kills are worth ([`kill_score`])
    /// and one for every ten damage they dealt.
    pub fn scores(&self) -> Vec<u32> {
        self.players
            .iter()
            .map(|player| player.kills.saturating_add((player.dealt / 10.0).floor().min(u32::MAX as f32) as u32))
            .collect()
    }

    /// What the room is sent of how the game stands ([`StandRound`]): cheap
    /// to compare, and to be sent only when it changed.
    pub fn round(&self) -> StandRound {
        StandRound {
            wave: self.wave,
            stage: match self.stage {
                Stage::Before => StandStage::Break { until: 0.0 },
                Stage::Break { until, .. } => StandStage::Break { until },
                Stage::Wave => StandStage::Wave,
                Stage::Over => StandStage::Over,
            },
            anchor: whole(self.anchor),
            anchor_most: whole(self.anchor_most),
            left: if self.stage == Stage::Wave {
                u16::try_from(self.queue.len() + self.zombies.iter().filter(|zombie| !zombie.dead).count()).unwrap_or(u16::MAX)
            } else {
                0
            },
            health: (0..self.players.len()).map(|seat| if self.alive(seat) { whole(self.players[seat].health).max(1) } else { 0 }).collect(),
            back_at: self.players.iter().map(|player| player.back_at).collect(),
            scores: self.scores(),
            ready: self
                .players
                .iter()
                .enumerate()
                .filter(|(_, player)| player.ready)
                .fold(0u8, |bits, (seat, _)| bits | (1 << seat)),
            gone: self
                .players
                .iter()
                .enumerate()
                .filter(|(_, player)| player.gone)
                .fold(0u8, |bits, (seat, _)| bits | (1 << seat)),
            began: self.began,
        }
    }

    /// How it came out, once it is over: places by score (equal scores
    /// share a place, a place being how many scored more), scores, and the
    /// waves the table cleared, which is what it is paid by
    /// ([`prizes`](super::prizes)). None while it is played.
    pub fn outcome(&self) -> Option<(Vec<u8>, Vec<u32>, Option<u32>)> {
        if !self.over() {
            return None;
        }
        let scores = self.scores();
        let places = scores
            .iter()
            .map(|score| scores.iter().filter(|other| *other > score).count() as u8)
            .collect();
        Some((places, scores, Some(u32::from(self.cleared))))
    }

    /// The team's score: everyone's, and [`WAVE_SCORE`] for each wave
    /// cleared ([`team_score`](super::team_score)).
    pub fn team_score(&self) -> u32 {
        super::team_score(&self.scores(), u32::from(self.cleared))
    }

    // -------------------------------------------------------------- inside

    /// The next number of `kind` of thing (0 zombies, 1 fireballs, 2 sticks,
    /// 3 balls of spit, 4 swords dropped, 5 treasure chests).
    fn number(&mut self, kind: usize) -> u32 {
        let number = self.next[kind];
        self.next[kind] = number.wrapping_add(1);
        number
    }

    /// `after` seconds from now by the room's clock, or by the game's own
    /// time with no clock said.
    fn on_clock(&self, after: f32) -> f64 {
        if self.clock > 0.0 { self.clock + f64::from(after) } else { f64::from(self.elapsed + after) }
    }

    /// Between waves, with someone standing and everyone standing ready
    /// ([`Game::ready`]): the next wave comes in [`READY_SECS`], if it was
    /// not coming sooner.
    fn hurry(&mut self) {
        let Stage::Break { ends, .. } = self.stage else {
            return;
        };
        let mut standing = (0..self.players.len()).filter(|&seat| self.alive(seat)).peekable();
        if standing.peek().is_none() || !standing.all(|seat| self.players[seat].ready) {
            return;
        }
        let soon = self.elapsed + READY_SECS;
        if soon < ends {
            self.stage = Stage::Break {
                ends: soon,
                until: self.on_clock(READY_SECS),
            };
        }
    }

    /// The person in `seat` has attacked: seen again if they were unseen,
    /// and no longer just back.
    fn reveal(&mut self, seat: usize) {
        if let Some(kit) = self.kits.get_mut(seat) {
            kit.unseen_until = 0;
        }
        if let Some(player) = self.players.get_mut(seat) {
            player.grace = 0.0;
        }
    }

    /// The dead stand again, beside the anchor while it stands, and out for
    /// good otherwise; and everyone's health comes back after a while.
    fn stand_again(&mut self, dt: f32) {
        for seat in 0..self.players.len() {
            let player = &mut self.players[seat];
            if player.vacant || player.gone || player.out {
                continue;
            }
            if let Some(left) = player.dead_for {
                if left > dt {
                    player.dead_for = Some(left - dt);
                    continue;
                }
                player.dead_for = None;
                if self.anchor <= 0.0 {
                    player.out = true;
                    player.back_at = f64::INFINITY;
                    self.events.push(Event::Out { seat });
                    continue;
                }
                let (at, turn) = arrival(seat);
                *player = Player {
                    at,
                    yaw: yaw_of(turn),
                    health: HEALTH,
                    since_hit: 0.0,
                    back_at: 0.0,
                    grace: GRACE_SECS,
                    ..player.clone()
                };
                self.events.push(Event::Back { seat, at, yaw: yaw_of(turn) });
                continue;
            }
            player.grace = (player.grace - dt).max(0.0);
            player.since_hit += dt;
            if player.since_hit >= REGEN_AFTER && player.health < HEALTH {
                player.health = (player.health + REGEN_PER_SEC * dt).min(HEALTH);
            }
        }
    }

    /// The waves: the next begins as the break ends, and its zombies climb
    /// over the rim from its places round it, a few at a time from each
    /// and then a pause, as many a second all told as [`rising`] has, while
    /// fewer than [`MOST_STANDING`] stand.
    fn come(&mut self, dt: f32) {
        match self.stage {
            Stage::Break { ends, .. } if self.elapsed >= ends => self.next_wave(),
            Stage::Wave => {}
            _ => return,
        }
        let mut standing = self.zombies.iter().filter(|zombie| !zombie.dead).count();
        let people = self.players.iter().filter(|player| !player.vacant && !player.gone).count();
        let every = self.places.len() as f32 / rising(self.wave, people);
        for place in 0..self.places.len() {
            self.places[place].wait -= dt;
            if self.places[place].wait > 0.0 || self.queue.is_empty() {
                continue;
            }
            if standing >= MOST_STANDING {
                self.places[place].wait = self.dice.around(0.2, 0.6);
                continue;
            }
            let kind = self.queue.pop().expect("not empty");
            let angle = self.places[place].angle + self.dice.around(-0.07, 0.07);
            self.raise(kind, angle);
            standing += 1;
            let place = &mut self.places[place];
            place.burst = place.burst.saturating_sub(1);
            place.wait = if place.burst == 0 {
                // The pause after a few, as long as the few take at the
                // wave's pace.
                place.burst = self.dice.between(2.0, 4.99) as u8;
                (every * f32::from(place.burst) - 0.5 * f32::from(place.burst - 1)).max(0.5) * self.dice.around(0.75, 1.25)
            } else {
                self.dice.around(0.3, 0.7)
            };
        }
    }

    /// The next wave begins: its zombies in the order they come ([`mix`]),
    /// from two to four places round the rim, more places as the waves go
    /// on, each well apart from the others.
    fn next_wave(&mut self) {
        self.wave = self.wave.saturating_add(1);
        let people = self.players.iter().filter(|player| !player.vacant && !player.gone).count();
        let mut queue = mix(self.wave, people, &mut self.dice);
        queue.reverse();
        self.queue = queue;
        let most = (2 + usize::from(self.wave) / 4).min(4);
        let count = (self.dice.between(2.0, most as f32 + 0.99) as usize).clamp(2, most);
        let mut angles: Vec<f32> = Vec::new();
        for _ in 0..count {
            let mut best = (f32::NEG_INFINITY, 0.0);
            for _ in 0..8 {
                let angle = self.dice.roll() * std::f32::consts::TAU;
                let room = angles.iter().map(|other| turn_between(angle, *other)).fold(f32::INFINITY, f32::min);
                if room > best.0 {
                    best = (room, angle);
                }
            }
            angles.push(best.1);
        }
        self.places = angles
            .into_iter()
            .map(|angle| Place {
                angle,
                wait: self.dice.around(0.0, 1.2),
                burst: self.dice.between(2.0, 4.99) as u8,
            })
            .collect();
        for player in &mut self.players {
            player.ready = false;
        }
        self.stage = Stage::Wave;
        self.events.push(Event::Wave {
            wave: self.wave,
            count: u16::try_from(self.queue.len()).unwrap_or(u16::MAX),
            new: first_in(self.wave),
        });
    }

    /// One more zombie of `kind` climbs over the rim at `angle`: going for
    /// the anchor, or for whoever is nearest, by a throw that goes more and
    /// more for people as the waves go on (mine: a giant always for the
    /// anchor, a runner mostly for people).
    fn raise(&mut self, kind: Kind, angle: f32) {
        let for_anchor = match kind {
            Kind::Giant => 1.0,
            Kind::Runner => 0.35,
            Kind::Brute => 0.85,
            _ => (0.8 - 0.035 * f32::from(self.wave.saturating_sub(1))).clamp(0.35, 0.8),
        };
        let hunter = self.dice.roll() >= for_anchor;
        let id = self.number(0);
        let people = self.players.iter().filter(|player| !player.vacant && !player.gone).count();
        let (health, damage, pace) = strength(kind, self.wave);
        let numbers = (health * toughness(kind, people), damage, pace);
        let zombie = Zombie::rise(id, kind, self.wave, numbers, angle, self.players.len(), hunter, &mut self.dice);
        self.events.push(Event::Risen {
            zombie: id,
            kind,
            at: zombie.at,
            yaw: zombie.yaw,
        });
        self.zombies.push(zombie);
    }

    /// Every zombie, for a frame: moved, and what each did carried out.
    fn shamble(&mut self, dt: f32) {
        let people: Vec<Seen> = (0..self.players.len())
            .map(|seat| Seen {
                at: self.players[seat].at,
                standing: self.alive(seat),
                open: !self.kits[seat].unseen(self.elapsed) && self.players[seat].grace <= 0.0,
            })
            .collect();
        let crowd: Vec<(Vec3, f32)> = self.zombies.iter().map(|zombie| (zombie.at, if zombie.dead { 0.0 } else { zombie.kind.size() })).collect();
        let anchor = self.anchor > 0.0;
        // How many of the wave are left, once all of it has come.
        let last = self.queue.is_empty().then(|| self.zombies.iter().filter(|zombie| !zombie.dead).count());
        let mut acts = Vec::new();
        for index in 0..self.zombies.len() {
            if self.zombies[index].dead {
                continue;
            }
            let view = View {
                people: &people,
                anchor,
                crowd: &crowd,
                index,
                last,
            };
            if let Some(act) = self.zombies[index].step(&view, &mut self.dice, dt) {
                acts.push((index, act));
            }
        }
        for (index, act) in acts {
            let (damage, kind) = (self.zombies[index].damage, self.zombies[index].kind);
            match act {
                Act::Hit { seat, knock, from } => {
                    if self.alive(seat) {
                        let damage = through(damage, &self.kits[seat], &self.upgrades);
                        self.strike(seat, damage, from, knock);
                    }
                }
                Act::HitAnchor => self.hit_anchor(damage * kind.on_the_anchor()),
                Act::Spat(spittle) => {
                    let spit = self.number(3);
                    self.spits.push((spit, spittle, damage));
                    self.events.push(Event::Spat { spit, spittle });
                }
                Act::Fell => {
                    let zombie = &mut self.zombies[index];
                    zombie.dead = true;
                    zombie.void = true;
                    zombie.killer = zombie.last_hit.filter(|(_, when)| self.elapsed - when <= CREDIT_SECS).map(|(seat, _)| seat);
                }
            }
        }
    }

    /// `damage` lands on the person in `seat`, from `from`, knocking them so
    /// hard: dead at none.
    fn strike(&mut self, seat: usize, damage: f32, from: Vec3, knock: Knock) {
        let player = &mut self.players[seat];
        player.health = (player.health - damage).max(0.0);
        player.since_hit = 0.0;
        self.events.push(Event::Struck { seat, damage, from, knock });
        if self.players[seat].health <= 0.0 {
            let at = self.players[seat].at;
            self.die(seat, at);
        }
    }

    /// The person in `seat` is dead, at `at`: for [`RESPAWN_SECS`] while the
    /// anchor stands, and for good otherwise. What they had drunk is over;
    /// and everything they held of the four things, and their sword, is
    /// thrown out round where they fell, to lie there ([`Game::drop_all`]),
    /// for them to fetch again once they stand, or a teammate to take (since
    /// 2026-11-18; Hajun: "when a player dies they should drop all their
    /// loot (silver,gold,sapphire, amethyst, sword) onto the ground, so
    /// after they respawn, they need to go collect it again, or if ur
    /// playing with a teammate, ur teammate can pick ur stuff up too": until
    /// then they kept it all). Their shield and what is on their buttons
    /// they keep (mine: Hajun's five are the things dropped).
    fn die(&mut self, seat: usize, at: Vec3) {
        let out = self.anchor <= 0.0;
        let back = self.on_clock(RESPAWN_SECS);
        let player = &mut self.players[seat];
        player.health = 0.0;
        player.ready = false;
        player.grace = 0.0;
        if out {
            player.out = true;
            player.back_at = f64::INFINITY;
        } else {
            player.dead_for = Some(RESPAWN_SECS);
            player.back_at = back;
        }
        let kit = &mut self.kits[seat];
        kit.fast_until = 0;
        kit.unseen_until = 0;
        kit.leap_until = 0;
        kit.blind = false;
        self.events.push(Event::Died { seat, at, out });
        self.drop_all(seat, at);
    }

    /// The person in `seat`, dead at `at`, drops all they hold of the four
    /// things and their sword: thrown out round them, onto the land only,
    /// from where they fell, or, dead in the sea, from where they last stood
    /// on the land (nothing lies in the sea); a thing for each one of an
    /// item, up to [`DROP_MOST`] of it, past that each more than one; the
    /// more there is, the wider it lies, by what it is worth and not by how
    /// many things ([`DROP_DENSITY`], out to [`DROP_FAR`]); each for whoever walks into it once it has lain a
    /// moment, them or a teammate, and gone after [`LIES_FOR`]. The sword
    /// lies among them for whoever holds a lesser one ([`Game::gather`]).
    fn drop_all(&mut self, seat: usize, at: Vec3) {
        let on_land = ground(at.xz()).is_some_and(|top| (at.y - top).abs() <= 1.0);
        let from = if on_land { at } else { self.players[seat].footing };
        let purse = self.purses[seat];
        let mut things: Vec<(Item, u16)> = Vec::new();
        for item in Item::ALL {
            let count = purse[item.index()];
            let pieces = count.min(DROP_MOST);
            for piece in 0..pieces {
                things.push((item, count / pieces + u16::from(piece < count % pieces)));
            }
        }
        let sword = self.kits[seat].sword;
        if things.is_empty() && sword == 0 {
            return;
        }
        // As wide as a thing for each one would lie, however few things
        // they are: fewer and bigger past the most, not a smaller heap.
        let worth = things.iter().map(|&(_, worth)| u32::from(worth)).sum::<u32>() + u32::from(sword > 0);
        let far = (worth as f32 / (std::f32::consts::PI * DROP_DENSITY) + SPILL_NEAR * SPILL_NEAR)
            .sqrt()
            .clamp(SPILL_FAR, DROP_FAR);
        let elapsed = self.elapsed;
        if !things.is_empty() {
            let dice = &mut self.dice;
            // Nowhere at all for it to lie (never, from the land), it is
            // kept.
            if let Some(dropped) = self.forges.pile_mut().scatter_within(&things, from, far, elapsed, &mut || dice.roll(), &lies_on) {
                self.purses[seat] = Purse::default();
                self.events.extend(dropped.into_iter().map(Event::Made));
            }
        }
        if sword > 0
            && let Some((spot, top)) = self.spot_near(from, far)
        {
            self.kits[seat].sword = 0;
            let blade = Blade {
                id: self.number(4),
                tier: sword,
                at: spot,
                ground: top,
                from: from + Vec3::Y * SPILL_FROM,
                born: elapsed,
            };
            self.blades.push(blade);
            self.events.push(Event::SwordDropped(blade));
        }
    }

    /// Somewhere for a thing thrown from `from` to lie, from [`SPILL_NEAR`]
    /// to `far` off it: the land, a step over its feet at the most, as
    /// [`Pile::scatter`](crate::bridge_wars::Pile::scatter) has it; or under
    /// the feet; or nowhere.
    fn spot_near(&mut self, from: Vec3, far: f32) -> Option<(Vec2, f32)> {
        let lies = |at: Vec2| lies_on(at).filter(|top| (-12.0..=0.6).contains(&(top - from.y)));
        for _ in 0..12 {
            let at = from.xz() + Vec2::from_angle(self.dice.roll() * std::f32::consts::TAU) * self.dice.around(SPILL_NEAR, far);
            if let Some(top) = lies(at) {
                return Some((at, top));
            }
        }
        lies(from.xz()).map(|top| (from.xz(), top))
    }

    /// What a treasure chest has in it for the person in `seat` (mine): a
    /// handful of silver most often, then gold, sapphire, amethyst, a
    /// sword one better than theirs, or something for a button, a little
    /// more of each as the waves go on. A sword or a thing for a button
    /// they could not have is gold instead.
    fn find(&mut self, seat: usize) -> Find {
        let more = 1.0 + 0.08 * f32::from(self.wave.max(1) - 1);
        let roll = self.dice.roll();
        let dice = &mut self.dice;
        let mut count = |low: f32, high: f32| (dice.around(low, high + 0.99).floor() * more).floor().clamp(1.0, 999.0) as u16;
        let kit = self.kits[seat];
        let gold = |count: u16| Find::Things { item: Item::Gold, count };
        match roll {
            roll if roll < 0.32 => Find::Things { item: Item::Silver, count: count(10.0, 25.0) },
            roll if roll < 0.60 => gold(count(2.0, 6.0)),
            roll if roll < 0.78 => Find::Things { item: Item::Sapphire, count: count(1.0, 3.0) },
            roll if roll < 0.87 => Find::Things { item: Item::Amethyst, count: count(1.0, 2.0) },
            roll if roll < 0.94 => {
                if kit.sword < 3 { Find::Sword(kit.sword + 1) } else { gold(count(4.0, 8.0)) }
            }
            roll => {
                let utility = if roll < 0.97 { Utility::Fireball } else { Utility::Healing };
                let room = match kit.slot_of(utility) {
                    Some(slot) => kit.slots[slot].is_some_and(|stack| stack.count < 9),
                    None => kit.free_slot().is_some(),
                };
                if room { Find::Utility(utility) } else { gold(count(2.0, 6.0)) }
            }
        }
    }

    /// What came out of a chest, the person in `seat`'s at once.
    fn give(&mut self, seat: usize, find: Find) {
        match find {
            Find::Things { item, count } => {
                let held = &mut self.purses[seat][item.index()];
                *held = held.saturating_add(count);
            }
            Find::Sword(tier) => {
                let kit = &mut self.kits[seat];
                kit.sword = kit.sword.max(tier);
            }
            Find::Utility(utility) => {
                let kit = &mut self.kits[seat];
                if let Some(slot) = kit.slot_of(utility).or(kit.free_slot()) {
                    kit.slots[slot].get_or_insert(crate::Stack { utility, count: 0 }).count += 1;
                }
            }
        }
    }

    /// The anchor takes `damage`: broken at none, and whoever is dead and
    /// waiting to stand again is out.
    pub(super) fn hit_anchor(&mut self, damage: f32) {
        if self.anchor <= 0.0 {
            return;
        }
        self.anchor = (self.anchor - damage).max(0.0);
        self.events.push(Event::AnchorHurt { damage });
        if self.anchor > 0.0 {
            return;
        }
        self.events.push(Event::AnchorBroken);
        for seat in 0..self.players.len() {
            let player = &mut self.players[seat];
            if player.dead_for.take().is_some() {
                player.out = true;
                player.back_at = f64::INFINITY;
                self.events.push(Event::Out { seat });
            }
        }
    }

    /// The zombie at `index` takes `damage`, dealt by the person in `by`, or
    /// the beam: whoever dealt it is credited with as much as it had left;
    /// dead at none, theirs.
    fn hurt(&mut self, index: usize, damage: f32, by: Option<usize>) {
        let elapsed = self.elapsed;
        let zombie = &mut self.zombies[index];
        if zombie.dead || damage <= 0.0 {
            return;
        }
        let dealt = damage.min(zombie.health);
        zombie.health -= damage;
        if let Some(seat) = by {
            zombie.last_hit = Some((seat, elapsed));
        }
        self.events.push(Event::Hurt {
            zombie: zombie.id,
            damage,
            by,
        });
        if zombie.health <= 0.0 {
            zombie.dead = true;
            zombie.killer = by;
        }
        if let Some(player) = by.and_then(|seat| self.players.get_mut(seat)) {
            player.dealt += dealt;
        }
    }

    /// Every spitter's ball, for a frame: on its way, and splashing whoever
    /// is within [`SPIT_RADIUS`] of where it comes down, through their
    /// shield, and the anchor if it comes down by it.
    fn splash(&mut self, dt: f32) {
        let mut landed = Vec::new();
        self.spits.retain_mut(|(spit, spittle, damage)| {
            spittle.tick(dt);
            if spittle.landed() {
                landed.push((*spit, spittle.to, *damage));
                return false;
            }
            true
        });
        for (spit, at, damage) in landed {
            self.events.push(Event::Splat { spit, at });
            for seat in 0..self.players.len() {
                let feet = self.players[seat].at;
                if self.alive(seat) && feet.xz().distance(at.xz()) <= SPIT_RADIUS + BODY && (feet.y - at.y).abs() <= 2.0 {
                    let damage = through(damage, &self.kits[seat], &self.upgrades);
                    self.strike(seat, damage, at, Knock::None);
                }
            }
            if at.xz().distance(ANCHOR) <= CRYSTAL_RADIUS + SPIT_RADIUS {
                self.hit_anchor(damage * Kind::Spitter.on_the_anchor());
            }
        }
    }

    /// Every fireball in the air, for a frame: flown on, and burst on the
    /// first zombie it meets or at the end of its range, its blast taking
    /// its damage off every zombie it reaches, sharpened, and knocking them
    /// from where it burst. Whoever threw it unseen is seen again, for the
    /// harm it did.
    fn fly(&mut self, dt: f32) {
        let mut flying = std::mem::take(&mut self.balls);
        flying.retain_mut(|(ball, flight, seat)| {
            let spent = flight.fly(dt);
            let at = flight.at();
            let met = self
                .zombies
                .iter()
                .any(|zombie| !zombie.dead && at.xz().distance(zombie.at.xz()) <= FIREBALL_RADIUS + zombie.radius() && at.y >= zombie.at.y - 0.2 && at.y <= zombie.at.y + 2.0 * zombie.kind.size());
            if !spent && !met {
                return true;
            }
            let damage = sharpened(FIREBALL_HIT, &self.upgrades);
            if self.blast(at, BLAST_RADIUS, damage, false, *seat) {
                self.reveal(*seat);
            }
            self.events.push(Event::Burst { ball: *ball, at });
            false
        });
        self.balls = flying;
    }

    /// Every stick of dynamite, for a frame: lobbed on, lying where it comes
    /// down if there is ground for it, or gone into the void, and once its
    /// fuse has burnt down, blown up: its damage, sharpened, off every
    /// zombie in its blast, and a knock on the run from its middle (the
    /// design's "a big knock").
    fn dynamite(&mut self, dt: f32) {
        let mut lying = std::mem::take(&mut self.sticks);
        lying.retain_mut(|(stick, lob, seat, lies)| {
            lob.tick(dt);
            if !lob.down() {
                return true;
            }
            if !*lies {
                if !caught(lob.to.xz(), |at| ground(at).is_some()) {
                    self.events.push(Event::Sank { stick: *stick });
                    return false;
                }
                *lies = true;
            }
            if !lob.blown() {
                return true;
            }
            let damage = sharpened(DYNAMITE_HIT, &self.upgrades);
            if self.blast(lob.to, DYNAMITE_RADIUS, damage, true, *seat) {
                self.reveal(*seat);
            }
            self.events.push(Event::Blew { stick: *stick, at: lob.to });
            false
        });
        self.sticks = lying;
    }

    /// A blast at `at`, `radius` across the ground and Bridge Wars' height up
    /// and down, by the person in `seat`: `damage` off every zombie whose
    /// body it reaches, and a knock from its middle, on the run if
    /// `running`. Whether it hurt any.
    fn blast(&mut self, at: Vec3, radius: f32, damage: f32, running: bool, seat: usize) -> bool {
        let caught: Vec<usize> = (0..self.zombies.len())
            .filter(|&index| {
                let zombie = &self.zombies[index];
                !zombie.dead
                    && zombie.at.xz().distance(at.xz()) <= radius + zombie.radius()
                    && (at.y - (zombie.at.y + zombie.kind.size())).abs() <= BLAST_HEIGHT * zombie.kind.size().max(1.0)
            })
            .collect();
        for &index in &caught {
            self.zombies[index].knock(at.xz(), running, 1.0);
            self.zombies[index].turn_on(seat);
            self.hurt(index, damage, Some(seat));
        }
        !caught.is_empty()
    }

    /// The anchor's beam, for a frame, while the anchor stands and the team
    /// has it: every [`BEAM_EVERY`], the zombie nearest the anchor within
    /// [`BEAM_REACH`] of it is zapped, for [`BEAM_DAMAGE`] a step.
    fn beam(&mut self, dt: f32) {
        if self.upgrades.beam == 0 || self.anchor <= 0.0 {
            return;
        }
        self.beam_in -= dt;
        if self.beam_in > 0.0 {
            return;
        }
        let nearest = (0..self.zombies.len())
            .filter(|&index| !self.zombies[index].dead && !self.zombies[index].climbing())
            .map(|index| (index, self.zombies[index].at.xz().distance(ANCHOR)))
            .filter(|(_, far)| *far <= BEAM_REACH)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((index, _)) = nearest else {
            // Nobody near: it looks again in a moment.
            self.beam_in = 0.25;
            return;
        };
        self.beam_in = BEAM_EVERY;
        let damage = BEAM_DAMAGE * f32::from(self.upgrades.beam);
        let (zombie, at) = (self.zombies[index].id, self.zombies[index].at);
        self.events.push(Event::Zap { zombie, at, damage });
        self.hurt(index, damage, None);
    }

    /// The generators, for a frame: what they make comes to lie on their
    /// pads, as much more as there are more people still in the game.
    fn forge(&mut self) {
        let people = self.players.iter().filter(|player| !player.vacant && !player.gone).count();
        let dice = &mut self.dice;
        let made = self.forges.run(self.elapsed, self.upgrades.generator, people, &mut || dice.roll());
        self.events.extend(made.into_iter().map(Event::Made));
    }

    /// Everyone standing, the lowest seat first, picks up whatever is within
    /// reach of their feet ([`Pile::pick_up`](crate::bridge_wars::Pile::pick_up)):
    /// into their own purse.
    fn gather(&mut self) {
        let elapsed = self.elapsed;
        for seat in 0..self.players.len() {
            if !self.alive(seat) {
                continue;
            }
            let feet = self.players[seat].at;
            for piece in self.forges.pile_mut().pick_up(feet, elapsed) {
                let held = &mut self.purses[seat][piece.item.index()];
                *held = held.saturating_add(piece.worth);
                self.events.push(Event::Taken { seat, piece });
            }
            // A sword the dead dropped, within reach as anything lying is
            // ([`MAGNET`], [`REACH_OVER`], [`REACH_UNDER`]) and lying a
            // moment: the best of them, if it is better than the one they
            // hold (Hajun: "for sword, can only pick up if its greater than
            // the sword they are already holding"); the one they held goes
            // as a sword bought over it does.
            let held = self.kits[seat].sword;
            let best = self
                .blades
                .iter()
                .enumerate()
                .filter(|(_, blade)| {
                    let over = blade.height(elapsed) - feet.y;
                    blade.tier > held
                        && blade.lying(elapsed)
                        && blade.at.distance(feet.xz()) <= MAGNET
                        && (-REACH_UNDER..=REACH_OVER).contains(&over)
                })
                .max_by_key(|(_, blade)| blade.tier)
                .map(|(index, _)| index);
            if let Some(index) = best {
                let blade = self.blades.remove(index);
                self.kits[seat].sword = blade.tier;
                self.events.push(Event::SwordTaken { seat, blade });
            }
        }
    }

    /// What was dropped and has lain [`LIES_FOR`], and a chest nobody
    /// opened that long, are gone; and the next chest comes a while after.
    fn lapse(&mut self) {
        let elapsed = self.elapsed;
        let gone = self.forges.pile_mut().lapse(elapsed, LIES_FOR);
        self.events.extend(gone.into_iter().map(Event::Lapsed));
        if self.blades.iter().any(|blade| elapsed - blade.born >= LIES_FOR) {
            let (gone, kept): (Vec<Blade>, Vec<Blade>) =
                std::mem::take(&mut self.blades).into_iter().partition(|blade| elapsed - blade.born >= LIES_FOR);
            self.blades = kept;
            self.events.extend(gone.into_iter().map(Event::SwordLapsed));
        }
        if let Some(index) = self.chests.iter().position(|chest| elapsed - chest.born >= LIES_FOR) {
            let chest = self.chests.remove(index);
            self.chest_at = elapsed + self.dice.around(CHEST_EVERY.0, CHEST_EVERY.1);
            self.events.push(Event::ChestLapsed(chest));
        }
    }

    /// A treasure chest, once its time has come and none stands: somewhere
    /// on the sand or out in the sea, thrown, clear of the stands, the
    /// palms, the barrels and the sand's very edge, and said to nobody
    /// (Hajun: "sometimes spawn in the sand area or the ocean, with no
    /// notification to the player of its location or existence, so the
    /// player can sometime wander around and find a treasure chest"). One at
    /// a time (mine).
    fn place_chest(&mut self) {
        if !self.chests.is_empty() || self.elapsed < self.chest_at {
            return;
        }
        for _ in 0..16 {
            let out = self.dice.around(STONE_RADIUS + 1.5, SHORE - 1.0);
            if (out - RIM).abs() < 1.0 {
                continue;
            }
            let at = Vec2::from_angle(self.dice.roll() * std::f32::consts::TAU) * out;
            if OBSTACLES.iter().any(|obstacle| obstacle.outside(at).0 < 1.0) {
                continue;
            }
            let y = ground(at).unwrap_or(SEA_BED);
            let chest = Treasure {
                id: self.number(5),
                at: Vec3::new(at.x, y, at.y),
                yaw: self.dice.roll() * std::f32::consts::TAU,
                born: self.elapsed,
            };
            self.chests.push(chest);
            self.events.push(Event::ChestPlaced(chest));
            return;
        }
        // Nowhere this time: again in a moment.
        self.chest_at = self.elapsed + 2.0;
    }

    /// What was drunk runs out, and a button with none left on it and
    /// nothing lasting from it is empty (Bridge Wars' rule).
    fn potions(&mut self) {
        let elapsed = self.elapsed;
        for kit in &mut self.kits {
            if kit.fast_until != 0 && !kit.fast(elapsed) {
                kit.fast_until = 0;
            }
            if kit.unseen_until != 0 && !kit.unseen(elapsed) {
                kit.unseen_until = 0;
            }
            if kit.leap_until != 0 && !kit.leaping(elapsed) {
                kit.leap_until = 0;
            }
            let lasting = [(Utility::Speed, kit.fast_until), (Utility::Invisibility, kit.unseen_until), (Utility::Leap, kit.leap_until)];
            for slot in &mut kit.slots {
                if slot.is_some_and(|stack| stack.count == 0 && !lasting.iter().any(|(potion, until)| *potion == stack.utility && *until != 0)) {
                    *slot = None;
                }
            }
        }
    }

    /// The zombies that died this frame: what each was worth to whoever
    /// killed it, what it drops where it fell (on the sand over the rim's
    /// side, for one killed climbing; not into the void, which had it), and
    /// gone.
    fn bury(&mut self) {
        if !self.zombies.iter().any(|zombie| zombie.dead) {
            return;
        }
        let (dead, standing): (Vec<Zombie>, Vec<Zombie>) = std::mem::take(&mut self.zombies).into_iter().partition(|zombie| zombie.dead);
        self.zombies = standing;
        for zombie in dead {
            if let Some(player) = zombie.killer.and_then(|seat| self.players.get_mut(seat)) {
                player.kills = player.kills.saturating_add(kill_score(zombie.kind, zombie.wave));
            }
            self.events.push(Event::Killed {
                zombie: zombie.id,
                kind: zombie.kind,
                by: zombie.killer,
                at: zombie.at,
                void: zombie.void,
            });
            if zombie.void {
                continue;
            }
            let things = drop_of(zombie.kind, zombie.wave, self.upgrades.lucky, &mut self.dice);
            if things.is_empty() {
                continue;
            }
            let dice = &mut self.dice;
            if let Some(dropped) = self.forges.pile_mut().scatter(&things, zombie.drops_from(), self.elapsed, &mut || dice.roll(), &lies_on) {
                self.events.extend(dropped.into_iter().map(Event::Made));
            }
        }
    }

    /// The wave is cleared once all of it has come and none of it stands:
    /// the break begins.
    fn clear(&mut self) {
        if self.stage != Stage::Wave || !self.queue.is_empty() || !self.zombies.is_empty() {
            return;
        }
        self.cleared = self.wave;
        self.events.push(Event::Cleared { wave: self.wave });
        self.stage = Stage::Break {
            ends: self.elapsed + BREAK,
            until: self.on_clock(BREAK),
        };
    }

    /// Over once the anchor is broken and nobody is standing, or everyone
    /// has gone.
    fn end(&mut self) {
        let standing = (0..self.players.len()).any(|seat| self.alive(seat));
        let anyone = self.players.iter().any(|player| !player.vacant && !player.gone);
        if (self.anchor <= 0.0 && !standing) || !anyone {
            self.stage = Stage::Over;
            for player in &mut self.players {
                player.ready = false;
            }
            self.events.push(Event::Over { waves: self.cleared });
        }
    }
}

/// How high what a thing a zombie drops may lie on is at `at`, if it may lie
/// there: the ground, but not in a stand, a barrel, a palm or the crystal,
/// where nobody reaches it, nor at their very edges.
fn lies_on(at: Vec2) -> Option<f32> {
    if OBSTACLES.iter().any(|obstacle| obstacle.outside(at).0 < 0.25) {
        return None;
    }
    ground(at)
}

/// One of `utility` off the button it is on: whether there was one.
fn spend(kit: &mut Kit, utility: Utility) -> bool {
    match kit.slots.iter_mut().flatten().find(|stack| stack.utility == utility && stack.count > 0) {
        Some(stack) => {
            stack.count -= 1;
            true
        }
        None => false,
    }
}

/// `value` in whole points, rounded up, as a count holds it.
fn whole(value: f32) -> u16 {
    value.ceil().clamp(0.0, f32::from(u16::MAX)) as u16
}

/// The turn about the upright of `turn`: -Z turned by it is ahead.
fn yaw_of(turn: Quat) -> f32 {
    let ahead = turn * Vec3::NEG_Z;
    (-ahead.x).atan2(-ahead.z)
}

/// How far apart two turns are, the short way round, in radians.
fn turn_between(a: f32, b: f32) -> f32 {
    let off = (a - b).rem_euclid(std::f32::consts::TAU);
    off.min(std::f32::consts::TAU - off)
}
