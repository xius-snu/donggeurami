//! The computer's players in Bridge Wars (since 2026-11-12; Hajun: "right
//! now the other bots dont do anything other than stay in their base. i
//! need the bots to be intelligent enough. think of a way to make this game
//! enjoyable. the difficulty need to moderate - not too hard, not too easy
//! ... maybe one way is every 1~2 seconds the bot scores a short list of
//! goals and does the highest one ... and a bonus for the current goal so
//! the bot doesn't flip between two every tick").
//!
//! One brain for the server's bots online and the device's own in a game
//! played offline, as the rules are one ([`crate::bridge_wars`]). It plays
//! by them: nothing is free, and everything is lost on death.
//!
//! * [`Body`]: a bot's body, moved over the map ([`Nav`]) as a body moves:
//!   walked where a body can walk, carried by a knock, landed on whatever
//!   ground is under it, land or plank, and fallen where there is none.
//! * [`Squad`]: every bot of a game. Each has a slow layer, which every
//!   second or so scores what it might be doing and takes the best, the
//!   one it is on getting a bonus ([`Goal`]); and a fast one, every frame,
//!   which does it ([`Squad::step`]): follows its way there, fights whoever
//!   comes into reach, lays its planks, and keeps to the ground.
//! * [`Order`]: what a frame of that comes to that the game has to carry
//!   out, whoever runs it: a punch, a purchase, a rung laid.
//! * [`Skill`]: every number of how hard they play, in one place.
//! * [`mingle`]: what they do in the waiting room before any of that, each
//!   as who it is.

pub mod mingle;

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use crate::bridge_wars::{
    AIR_DRAG, BLIND_SIGHT, BOT_PUNCH_EVERY, BRIDGE_A_BUY, BUILDER_METRES, DRINK_SECS, DRINK_SLOW,
    DYNAMITE_FURTHEST, DYNAMITE_FUSE, DYNAMITE_NEAREST, DYNAMITE_RADIUS, FAST,
    FIREBALL_RANGE, FORGE_SPEED,
    GROUND_DRAG, HUNT_RADIUS, KNOCK_BACK, KNOCK_UP, Mint, Outfits, REACH, RUN_SPEED, SPRINT_BONUS, builder_speed, interval,
    leap_speed, price,
};
use crate::nav::{BODY, Gap, Nav, Region, Span};
use crate::{
    AMETHYST_PADS, ANCHOR_RADIUS, ARENA_ANCHORS, ARENA_ISLANDS, ARENA_PADS, ARENA_SEATS, ARENA_SHOPS, ARENA_TEAMS,
    ARENA_UPGRADE_SHOPS, Item, Kit, Purse, SAPPHIRE_PADS, Upgrades, Utility, Ware, arena_team,
};

/// Gravity on a body knocked up, on the way up and on the way down, in
/// metres a second a second: the game's `GRAVITY` and `FALL_GRAVITY`.
const RISE_GRAVITY: f32 = 22.0;
const FALL_GRAVITY: f32 = 33.0;
/// How high a rise a body walks up, in metres: the game's step, and the
/// centimetre the map's floors are kept to.
const STEP: f32 = 0.51;
/// This near a point of its way a bot has come to it, in metres.
const ARRIVED: f32 = 0.35;
/// How far ahead of its feet a bot's bridge is laid, in metres, at the
/// least, as the Bridge Builder's is.
const LAID_AHEAD: f32 = 1.6;
/// How near a bot stands off whoever it fights, as a share of its reach.
const PRESS_IN: f32 = 0.7;
/// How fast a bot hops out of the pond, upward and across, in metres a
/// second: over its rim, 0.6 m over its floor.
const HOP_UP: f32 = 6.5;
const HOP_ACROSS: f32 = 5.0;
/// How fast a body leaves the ground in a jump of its own, in metres a
/// second: the game's `JUMP_SPEED`.
const JUMP_UP: f32 = 8.5;
/// How far a body knocked off an edge looks for ground to steer back to,
/// in metres.
const REACH_BACK: f32 = 8.0;
/// How far whoever a punch lands on is carried by it, in metres: what a
/// bot goes by in choosing where to stand ([`flung`]). A knock standing
/// carries about 3 and a running one more.
const CARRY: [f32; 2] = [3.0, 4.5];
/// How far out from whoever it fights a bot stands to punch them, how far
/// back it steps between punches when it pokes at them, and how far off
/// it stands when it waits for them to come, in metres.
const RING: f32 = 1.45;
const POKE_BACK: f32 = 3.6;
const HOLD_OFF: f32 = 4.2;
/// How much a bridge may slant where it leaves the land and where it
/// meets it, as so much to the side for each metre on (about 24
/// degrees); and how many times as long as the straight line between its
/// ends it may be, bends and all ([`Squad::bends`]).
const MOST_SLANT: f32 = 0.45;
const MOST_BENT: f32 = 1.05;
/// How long a body is in the air in a jump of its own, in seconds, at the
/// least (the game's `JUMP_SPEED` against its two gravities; a frame or
/// two more as it is stepped); and how much of the ground it would cross
/// in that while a bot keeps in hand when it jumps a gap, in metres: it
/// comes down that far past where the ground begins again, or does not
/// jump ([`Squad::reach_of_a_jump`]). At a run that is a gap of 6.1 m:
/// the four planks a stick of dynamite all but always takes out of a
/// bridge (6.0 m of planks a tenth bigger than Hajun's, since the night of
/// 2026-11-14), and not the five it takes blown on the very middle of a
/// plank (7.5), which nobody clears. It kept 1.2 m in hand, a gap of 5.8,
/// while a plank was Hajun's own size and four of them 5.45 m, and while
/// a plank was a quarter bigger and three of them 5.1: with planks a
/// tenth bigger that left no hole a stick makes for a bot to jump, so it
/// is 0.9, the metre a run carries it past four planks less a little.
const JUMP_SECS: f32 = 0.7;
const JUMP_SPARE: f32 = 0.9;
/// And with a leap potion on it (since 2026-11-15): a jump that leaves the
/// ground the square root of five times as fast, 19.0 m a second, is 0.86
/// s going up 8.2 m and 0.70 s coming down, so this long in the air; and
/// it keeps more in hand over a gap, coming down from that high. At a run
/// that is a gap of 14 m: the hole a stick of dynamite blows, for as long
/// as the planks either side of it stand (the rest of the bridge falls
/// after it, since the same day); from one bridge to another laid beside
/// it; and no water between two of the map's lands, the narrowest of
/// which is 37 m.
const LEAP_SECS_ALOFT: f32 = 1.55;
const LEAP_SPARE: f32 = 1.5;
/// How long a bot stands at a stand over each thing it buys there, in
/// seconds, the least and the most, thrown again for each ([`Dice::around`]:
/// mostly about a second), as someone with the shop's window open reads
/// it, finds the card and presses it (the night of 2026-11-14, Hajun:
/// "right now the ai's use the shops in like instantly. but real humans
/// take like 0.5~1.5s ish to buy sth. make the ais take some time to use
/// the shop and buy sth too"). Until then it bought as its foot touched
/// the outline, and three things in three frames.
const SHOP_SECS: (f32, f32) = (0.5, 1.5);
/// How long a bot with a stick of dynamite on it goes between thinking
/// of going out to cut another team's way, in seconds, about
/// ([`Squad::way_to_cut`]): whether it means to is thrown each time, by
/// who it is. And how long it then means to, in seconds, about: it goes
/// when there is nothing it would sooner do, and having meant to that
/// long without going, has thought better of it.
const CUT_AGAIN: (f32, f32) = (20.0, 60.0);
const CUT_MEANT: (f32, f32) = (40.0, 100.0);
/// How likely it is to mean to, each time it thinks of it: so much for
/// anyone, so much more for one given to schemes ([`Persona::schemes`]),
/// and so much for one that likes to see people fall
/// ([`Temper::shoves`]). What to turn for how many bridges the
/// computer's players burn in a game.
const CUT_KEEN: [f32; 3] = [0.2, 0.7, 0.3];
/// Someone of another team this near a bot is on top of it, in metres:
/// it turns on them whatever lies between ([`Squad::foe`]). Nearer than
/// one bridge laid beside another leaves its edge from the other's.
const ON_TOP_OF_IT: f32 = 3.5;
/// How often a bot on its way somewhere looks along the straight walk to
/// the next point of its way, in seconds, to find the way again if it
/// cannot be walked from where it has got to ([`Squad::walk_to`]).
const LOOK_EVERY: f32 = 0.25;
/// How far ahead a bot reads where whoever it is after will be, at the
/// most, in seconds ([`Squad::meet`]).
const FORESEE: f32 = 1.6;
/// How far past where a stick of dynamite's blast reaches a bot gets out
/// of its way, and how far short of it one waits that was walking into
/// it, in metres ([`Squad::mind_sticks`]).
const CLEAR_OF_BLAST: f32 = 1.0;
const SHORT_OF_BLAST: f32 = 3.0;
/// And how much further off than that one walking at a stick takes note
/// of it, in metres: what it runs in the while it takes to wake up to it.
const NOTICED_FROM: f32 = 12.0;
/// How long a bot takes to wake up to a stick of dynamite thrown at it, in
/// seconds from the throw, at the soonest and at the latest: the worst of
/// them at a fight ([`Persona::pvp`] nothing), and how much sooner the
/// best. A stick is in the air about 0.6 s and lies 1.2 s, and getting out
/// of its blast takes a third of a second: the best are moving about as it
/// comes down, most by its second flash, and the worst are caught by one
/// thrown from close by that comes down on them about two times in five
/// (a test counts them, kind by kind). In a fight everyone is that much
/// later ([`STICK_BUSY`]), which is when a stick dropped at your own feet
/// catches them. One lying ahead on its way a bot has seen coming, and
/// takes this share of the time over ([`STICK_AHEAD`]).
const STICK_WAKES: (f32, f32) = (0.75, 2.0);
const STICK_WAKES_SOONER: (f32, f32) = (0.4, 1.0);
const STICK_BUSY: f32 = 0.25;
const STICK_AHEAD: f32 = 0.5;
/// A bot and someone who keeps coming at its base ([`Vigil`]), every
/// number of it a spread that each time is thrown from: how long it looks
/// after them when they step back, before it has had enough of them, in
/// seconds; how long it stands its ground once it has, before it either
/// goes at them or lets them be; how far from it they may be for it to go
/// at them from where it stands, in metres, for a bot as bold as most; how
/// far from the middle of its island they have to be to be gone, and for
/// how long; how far from them it stands, and how far in from the edge of
/// its island; and how long after they were last about it has forgotten
/// them. All mine: Hajun asked for the patience, and for it to differ from
/// one to the next ([`Persona::patience`]).
const VIGIL_LOOK: (f32, f32) = (0.8, 4.0);
const VIGIL_STAND: (f32, f32) = (12.0, 36.0);
const VIGIL_WATCH: (f32, f32) = (5.0, 12.0);
const VIGIL_GONE: (f32, f32) = (24.0, 38.0);
const VIGIL_CALM: (f32, f32) = (2.5, 8.0);
const VIGIL_OFF: (f32, f32) = (3.0, 5.0);
const VIGIL_INSIDE: (f32, f32) = (0.9, 2.2);
const VIGIL_FORGET: (f32, f32) = (50.0, 110.0);
/// What a bot's offer to stand its ground scores, of a score of 1
/// ([`Goal::Guard`]): over anything it does about its island, and, with
/// what the goal it is on has over the rest, still under backing off hurt.
const GUARD_SCORE: f32 = 0.79;
/// How much further off than it would go for someone a bot keeps after
/// one it is on already, as so many times as far; how far either side of
/// how near it lets people come it wanders from one while to the next
/// ([`Bot::keen`]), and for how long each while is, in seconds; how long a
/// blow smarts, in which it goes for whoever is near; and how far either
/// side of where it backs off hurt its nerve is from one fright to the
/// next, as a share of its health. Nothing a bot does begins or ends on a
/// line that is the same twice (Hajun, 2026-11-13: "if some thing becomes
/// a definite if-then, then its not a real human, but a program").
const KEEPS_ON: f32 = 1.3;
const KEEN: (f32, f32) = (0.72, 1.35);
const KEEN_LASTS: (f32, f32) = (4.0, 11.0);
const STING: (f32, f32) = (2.2, 6.5);
const NERVE: (f32, f32) = (-0.08, 0.08);
/// A bridge this short is a hole mended, and is laid straight, with no
/// bends of its own ([`Squad::lay`]).
const MEND: f32 = 12.0;
/// How far inside what stands, either side of a hole, the planks that
/// mend it begin and end, in metres: as a bridge's first and last rungs
/// lie inside the shores it joins.
const MEND_INSET: f32 = 0.5;
/// A bot and its team's pad (since 2026-11-15; Hajun: "the ai seem to be
/// just in their island for too long of the game like idle like both of
/// them on the generator which is not only inefficient but not
/// humanlike"). What a pad makes lies on it until someone comes for it,
/// so standing on it gains nothing: a bot comes once what lies there is
/// worth the walk, takes it, stays a moment longer if the rest of what it
/// is short of is all but made, and is off, and not back for a while.
/// Each of these a spread thrown from again each visit: how many of the
/// things it wants have to lie there for it to come; how long it will
/// stand there for the rest, in seconds; and how long before it comes
/// again, sooner for one that keeps to its island ([`Persona::roams`]).
/// Over three hundred games they were by their pad 55% of the first ten
/// minutes until then, a team's two both there a third of it.
const PAD_WORTH: (f32, f32) = (2.0, 9.0);
const PAD_WAIT: (f32, f32) = (2.0, 9.0);
const PAD_AGAIN: (f32, f32) = (6.0, 20.0);
/// With nothing it has to do, a bot is about its island from one thing to
/// another ([`Goal::Idle`]): at one of its two stands for the most part,
/// in the outline, as someone with the shop's window open stands, this
/// many seconds at a time; or at the shore its bridges leave from, looking
/// out; or it goes out to look about ([`Goal::Scout`]): at each place it
/// comes to it looks about for this long, and once it is back it stays in
/// for this long, or half of it for one that is always out ([`Persona::
/// roams`]). And whether it is one for going out at all just now is thrown
/// again every so many seconds ([`Bot::out`]): nobody is the same all
/// game.
/// A bot on a bridge that is coming down ([`Squad::mind_falling`];
/// since 2026-11-15, when a stick of dynamite began to take the whole
/// bridge with it, plank by plank from the hole outward: Hajun, "it
/// shouldnt destroy the whole bridge instantly but rather cascadingly
/// from that initial blast point so it gives ppl time to run away to
/// safety"). How long it takes to take in that the planks under it are
/// going, in seconds, the least and the most, for the worst of them at a
/// fight, the best being about twice as quick ([`Persona::pvp`]); and how
/// much time it keeps in hand over each plank on its way off, for one that
/// minds the edge as most do, more for one that minds it more and less for
/// one that does not ([`Temper::minds`]). Each thrown again every time:
/// the falling begins 0.6 s after the blast and comes on at 6 m a second,
/// against a run of 10, and faster with every second until it is going at
/// 12 (`bridge_wars::cascade_after`, since the night of 2026-11-15; at 6
/// all the way until then), so most make it off a bridge to the middle
/// from anywhere but beside the hole, and the slow and the careless now
/// and then do not; and off a much longer one, from far along it, none.
const FALL_WAKES: (f32, f32) = (0.15, 0.9);
const FALL_SPARE: (f32, f32) = (0.1, 0.6);
/// How far off what someone who died has dropped may lie for a bot to go
/// and take it, in metres, the least and the most (since the night of
/// 2026-11-15, when the dead began to drop their sapphire and amethyst
/// where they die on land, `bridge_wars::Mint::spill`; Hajun: "u only get
/// it if u pick it up"): thrown again whenever nothing lies that near, so
/// that one a little way off takes a moment to see it, and one far off
/// may not before someone else has it.
const SPOILS_WITHIN: (f32, f32) = (8.0, 30.0);
/// A bot that is by as someone dies and drops what they held, as whoever
/// killed them with a hand is, steps over and takes it before it goes on
/// with whatever it was about ([`Squad::mind_spoils`]; Hajun, that night:
/// "does the ai collect the loot after they kill someone? if not they
/// should." Whoever killed someone had some of what they dropped within
/// six seconds after half of their kills, over 300 games, a raider going
/// on for the anchor past it, and after two in three at all). How often
/// one does, of the times it is by; how near what was dropped has to be
/// for it to, in metres, the least and the most; how near someone of
/// another team may be before it sees to them first; and how long it
/// gives it, in seconds. Each thrown again for every death.
const GRAB_ODDS: f32 = 0.93;
const GRAB_WITHIN: (f32, f32) = (5.0, 10.0);
const GRAB_CLEAR: (f32, f32) = (3.5, 7.0);
const GRAB_LASTS: (f32, f32) = (4.0, 9.0);
const BROWSE: (f32, f32) = (3.0, 9.0);
const LOOK_ABOUT: (f32, f32) = (2.0, 7.0);
const SCOUT_AGAIN: (f32, f32) = (8.0, 34.0);
const MOOD_LASTS: (f32, f32) = (25.0, 70.0);

/// How hard the computer's players play: every number of their pacing, in
/// one place, to tune by.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Skill {
    /// How often a bot thinks again of what to do, in seconds, between
    /// these two.
    pub think_every: (f32, f32),
    /// The bonus the goal it is on has over the rest, of a score of 1.
    pub sticky: f32,
    /// How long after coming into reach of someone a bot's first punch
    /// comes, in seconds, and how long between its punches after.
    pub windup: f32,
    pub punch_every: f32,
    /// How long between a bot's punches on an anchor, in seconds, between
    /// these two, thrown again for each; and one that is there for the
    /// anchor and nothing else is quicker by its zeal ([`Temper::zeal`]).
    /// (Two seconds flat until 2026-11-15, when Hajun found "the rate at
    /// which they destroy the anchor is too slow".)
    pub anchor_every: (f32, f32),
    /// How near someone of another team has to come for a bot minding its
    /// own business to go for them, in metres: this at first, growing to
    /// its full sight between the two times, in seconds into the fight.
    pub aggro: f32,
    pub aggro_grows: (f32, f32),
    /// Under this share of its health, with someone after it, a bot backs
    /// off; and it comes back once it has this much again.
    pub retreat_below: f32,
    pub recovered: f32,
    /// No bot goes to another team's island to fight before about this,
    /// in seconds into the fight, each a little sooner or later than the
    /// next ([`Bot::raids_from`]); none touches an anchor before a time
    /// of its own thrown between the two of `anchors_from`, sooner by its
    /// zeal ([`Bot::anchors_from`]); and the one that keeps its team's
    /// base does not go out on a raid before one thrown between the two of
    /// `keeper_raids_from`. (Each one time for all of them until
    /// 2026-11-15: 150, 240 and 330 s.)
    pub raid_from: f32,
    pub anchors_from: (f32, f32),
    pub keeper_raids_from: (f32, f32),
    /// The more anchors are broken, whoever broke them, the less a bot
    /// goes for another: as one in one and this much for each that is
    /// gone, until a time of its own thrown between the two of
    /// `anchors_spared_until`, in seconds into the fight. The first
    /// minutes are not for everyone to be out in. (Until 2026-11-15 none
    /// at all went for a third before 480 s.)
    pub anchors_spared: f32,
    pub anchors_spared_until: (f32, f32),
    /// How much they go for anchors at all, all of them alike, as so many
    /// times what each one's own leanings come to ([`Squad::take_aim`]):
    /// what to turn to have more anchors broken in a game, or fewer.
    pub anchor_keen: f32,
    /// How long a raid stays on an island at the most, in seconds; how
    /// long a bot that died on one, or came back from one, stays home
    /// after; and how long a team that was raided is left alone after:
    /// each between these two, thrown again each time. (45, 40 and 90 s
    /// flat until 2026-11-15, which with a team's two kept home between
    /// times had them on their own island four fifths of a game.)
    pub raid_secs: (f32, f32),
    pub raid_rest: (f32, f32),
    pub victim_rest: (f32, f32),
    /// The most of the raids that go to a team with a person on it, as a
    /// share of all of them, while there are other teams to go to: no
    /// more than a team's share of four.
    pub people_share: f32,
    /// From when everyone goes for everyone, wherever they are, in
    /// seconds: a while after the anchors break, so that the end of a
    /// game is minutes and not moments. Until then the raids go on as
    /// they were, each death the last.
    pub hunt_from: f32,
    /// Under this share of its health a bot backs off once its anchor is
    /// gone, when dying is for good.
    pub careful_below: f32,
    /// How long between a bot's fireballs, in seconds.
    pub throw_every: f32,
    /// How long after a knock lands a bot is carried by it and no more,
    /// before it answers, in seconds, between these two; and how much of
    /// its pace it then steers by in the air, between these two. Thrown
    /// again at every knock: now it is back on its feet at once, now it
    /// goes over the edge.
    pub reel: (f32, f32),
    pub grip: (f32, f32),
}

impl Skill {
    /// Not too hard, not too easy.
    pub const MODERATE: Skill = Skill {
        think_every: (1.0, 1.5),
        sticky: 0.15,
        windup: 0.3,
        punch_every: BOT_PUNCH_EVERY,
        anchor_every: (1.6, 2.4),
        aggro: 6.0,
        aggro_grows: (150.0, 600.0),
        retreat_below: 0.35,
        recovered: 0.6,
        raid_from: 150.0,
        anchors_from: (190.0, 320.0),
        keeper_raids_from: (210.0, 360.0),
        anchors_spared: 2.0,
        anchors_spared_until: (400.0, 540.0),
        anchor_keen: 0.55,
        raid_secs: (32.0, 60.0),
        raid_rest: (12.0, 38.0),
        victim_rest: (30.0, 70.0),
        people_share: 1.0 / 3.0,
        hunt_from: 780.0,
        careful_below: 0.5,
        throw_every: 6.0,
        reel: (0.06, 0.36),
        grip: (0.5, 1.0),
    };

    /// How near someone has to come for a bot to go for them unprovoked,
    /// `elapsed` seconds into the fight.
    pub fn aggro_at(&self, elapsed: f32) -> f32 {
        let (from, to) = self.aggro_grows;
        let share = ((elapsed - from) / (to - from).max(1.0)).clamp(0.0, 1.0);
        self.aggro + (HUNT_RADIUS - self.aggro).max(0.0) * share
    }
}

/// Dice of a bot's own: the same seed, the same game.
#[derive(Clone, Debug)]
pub struct Dice(u64);

impl Dice {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// A number from 0 up to 1.
    pub fn roll(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn between(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.roll()
    }

    /// A number about 0 as a bell rings them: two in three within 1 of
    /// it, and none further than 3.5.
    pub fn bell(&mut self) -> f32 {
        (self.roll() + self.roll() + self.roll() + self.roll() - 2.0) * 1.732
    }

    /// A number from `low` to `high`, mostly about the middle.
    pub fn around(&mut self, low: f32, high: f32) -> f32 {
        ((low + high) * 0.5 + (high - low) / 6.0 * self.bell()).clamp(low.min(high), high.max(low))
    }
}

/// A bot's body: where it is, which way it faces, what a knock is
/// carrying it by, across the ground and upward, and whether it is
/// jumping a gap ([`Body::leap`]).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Body {
    pub at: Vec3,
    /// Its turn about the upright, in radians: -Z turned by this is ahead.
    pub yaw: f32,
    pub shove: Vec2,
    pub lift: f32,
    /// It means to leave the ground it is on: walking, it goes on over any
    /// ground there is, to its very edge, and off that with a jump, as
    /// anyone jumps a gap, where otherwise it never walks off an edge
    /// (since 2026-11-13). Its brain's to say, each frame.
    pub leap: bool,
    /// A leap potion is on it: a jump of its own leaves the ground as much
    /// faster as takes it five times as high ([`leap_speed`]), and is in
    /// the air more than twice as long ([`LEAP_SECS_ALOFT`]; since
    /// 2026-11-15). Its brain's to say, each frame, by its kit.
    pub high: bool,
}

impl Body {
    pub fn standing(at: Vec3, yaw: f32) -> Self {
        Self {
            at,
            yaw,
            ..Self::default()
        }
    }

    /// How fast a jump of its own leaves the ground, in metres a second.
    fn spring(&self) -> f32 {
        JUMP_UP * if self.high { leap_speed() } else { 1.0 }
    }

    /// Whether it is on the ground and its own to move: not in the air,
    /// nor being carried along by a knock.
    pub fn free(&self, nav: &Nav) -> bool {
        self.lift == 0.0
            && self.shove == Vec2::ZERO
            && nav.ground(self.at.xz()).is_some_and(|ground| (self.at.y - ground).abs() < 0.02)
    }

    /// Knocks it, by a punch or a blast from `from`, thrown on the run if
    /// `running`: up, and away from there, on top of half of whatever
    /// already carried it, as a punch knocks anyone. `share` is how much
    /// of the knock it takes.
    pub fn knock(&mut self, from: Vec2, running: bool, share: f32) {
        let away = (self.at.xz() - from).normalize_or(Vec2::NEG_Y);
        let push = (KNOCK_BACK + if running { SPRINT_BONUS } else { 0.0 }) * share;
        let up = KNOCK_UP * share;
        self.shove = self.shove * 0.5 + away * push;
        self.lift = (self.lift * 0.5 + up).min(up);
    }

    /// Jumps it `way`: up and across, for getting out of the pond.
    pub fn hop(&mut self, way: Vec2) {
        self.lift = HOP_UP;
        self.shove = way.normalize_or_zero() * HOP_ACROSS;
    }

    /// Jumps it, as anyone jumps: up, from the ground it stands on.
    /// Whether it did.
    pub fn jump(&mut self, nav: &Nav) -> bool {
        let grounded = self.lift == 0.0 && nav.ground(self.at.xz()).is_some_and(|ground| (self.at.y - ground).abs() < 0.02);
        if grounded {
            self.lift = self.spring();
        }
        grounded
    }

    /// Moves it on by `dt` seconds: up and down as a knock or a jump threw
    /// it, landing on whatever ground it comes down onto and falling
    /// where there is none; carried across by a knock, dying away as it
    /// goes; and walked by `walk`, metres a second across the ground, all
    /// the while, as anyone's legs are their own in the air and with a
    /// knock on them (until 2026-11-12 a bot knocked was carried and no
    /// more until it had come to rest; Hajun: "the bot is like stunned
    /// ... so its so easy to knock them off the map"). On the ground it
    /// walks only where a body can walk; in the air only over ground, or,
    /// over the void already, anywhere, which is how one knocked off an
    /// edge steers back. It never walks off an edge: a knock is what
    /// takes it off one.
    pub fn step(&mut self, nav: &Nav, walk: Vec2, dt: f32) {
        let mut here = self.at.xz();
        let ground = nav.ground(here);
        // Stood a little under the ground there, as a game that stands it
        // where its seat arrives may stand it (the device's arrivals are
        // at the land's own height, under an island's top): onto it.
        if self.lift == 0.0
            && let Some(ground) = ground
            && self.at.y < ground
            && ground - self.at.y <= STEP
        {
            self.at.y = ground;
        }
        let floor = ground.unwrap_or(f32::NEG_INFINITY);
        // In the air: going up, over the ground, or under it, having
        // fallen past an edge and been carried in under the land or a
        // plank, where it goes on falling (until 2026-11-12 it hung there,
        // neither on the ground nor in the air, for the rest of the game).
        let airborne = self.lift > 0.0 || self.at.y > floor + 1e-3 || self.at.y < floor - 1e-3;
        if airborne {
            let before = self.at.y;
            self.at.y += self.lift * dt;
            self.lift -= if self.lift > 0.0 { RISE_GRAVITY } else { FALL_GRAVITY } * dt;
            // Down onto ground it was over: not up through an island from
            // under it.
            if self.at.y <= floor && before >= floor - 0.3 {
                self.at.y = floor;
                self.lift = 0.0;
            }
        } else if self.lift != 0.0 {
            // Come down to within a hair of the ground without quite
            // touching it: it is on it. (Until 2026-11-13 it stood there
            // still falling, as far as it knew, for as long as it stayed on
            // ground of that height: neither in the air nor its own to
            // move, so it never jumped nor drank.)
            self.at.y = floor;
            self.lift = 0.0;
        }
        if self.shove != Vec2::ZERO {
            here += self.shove * dt;
            self.at.x = here.x;
            self.at.z = here.y;
            let drag = if airborne { AIR_DRAG } else { GROUND_DRAG };
            self.shove *= drag.powf(dt * 20.0);
            if self.shove.length_squared() < 0.01 {
                self.shove = Vec2::ZERO;
            }
        }
        if walk == Vec2::ZERO {
            return;
        }
        // Where a body walks; or, off that already, knocked against a
        // stand or onto an island's rim, any ground, to get back to it.
        let astray = !nav.walkable_at(here);
        let whole = here + walk * dt;
        let tries = [whole, Vec2::new(whole.x, here.y), Vec2::new(here.x, whole.y)];
        if airborne {
            // In the air: over the void, wherever it steers, as it does
            // jumping a gap; over ground, only on over ground, and not out
            // of where a body walks.
            let over_void = nav.ground(here).is_none();
            let open = |to: Vec2| over_void || self.leap || ((nav.walkable_at(to) || astray) && nav.ground(to).is_some());
            if let Some(to) = tries.into_iter().find(|&to| to != here && open(to)) {
                self.at.x = to.x;
                self.at.z = to.y;
            }
        } else if self.leap {
            // Jumping a gap: on over any ground there is, to its very
            // edge, and off that with a jump.
            match nav.ground(whole) {
                Some(there) if there - self.at.y <= STEP => {
                    self.at.x = whole.x;
                    self.at.z = whole.y;
                    if self.at.y - there <= STEP {
                        self.at.y = there;
                    }
                }
                // Something too high to step up.
                Some(_) => {}
                None => {
                    self.lift = self.spring();
                    self.at.x = whole.x;
                    self.at.z = whole.y;
                }
            }
        } else {
            let open = |to: Vec2| {
                (nav.walkable_at(to) || astray) && nav.ground(to).is_some_and(|there| there - self.at.y <= STEP)
            };
            if let Some(to) = tries.into_iter().find(|&to| to != here && open(to)) {
                self.at.x = to.x;
                self.at.z = to.y;
                if let Some(ground) = nav.ground(to) {
                    // Down a step at once, as up one; down a drop, it
                    // falls.
                    if self.at.y - ground <= STEP {
                        self.at.y = ground;
                    }
                }
            }
        }
        if walk.length_squared() > 1e-4 {
            self.yaw = (-walk.x).atan2(-walk.y);
        }
    }
}

/// What a bot is doing, as its slow layer last chose.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Goal {
    /// Nothing in particular: at home, by its anchor or its pad.
    Idle,
    /// On its team's pad, for what its generators make.
    Collect,
    /// To its island's shop for this, and buying it.
    Shop(Ware),
    /// Laying the bridge from the one land to the other.
    Lay(Region, Region),
    /// To a pad somewhere for what lies on it.
    Fetch(Vec2, Item),
    /// Home, where someone of another team is, to see them off.
    Defend,
    /// Between its base and someone who was at it and has only stepped
    /// back: looking after them a moment, or, having had enough of their
    /// coming and going, standing its ground until they are gone for good
    /// ([`Vigil`]).
    Guard,
    /// Away from whoever is after it, to get its health back.
    Retreat,
    /// To this team's island: to fight whoever is home, take what lies on
    /// their pad, and, once anchors are fair game, break theirs.
    Raid(u8),
    /// After whoever of another team is nearest, wherever they are: once
    /// the anchors are gone.
    Hunt,
    /// Out to this place to look about, with nothing it has to do: over
    /// its team's bridges to the middle, a smaller island, the end of
    /// someone else's bridge ([`Squad::lookout`]; since 2026-11-15).
    Scout(Vec2),
}

/// What a bot that has stepped onto the island it is raiding is there
/// for, thrown as it steps ashore by who it is and how things stand
/// ([`Squad::take_aim`]; since 2026-11-15, Hajun: "right now sometime the
/// ai when they invade other island they just go for the generator first,
/// but sometime they should just go for the anchor first or the players
/// first"). Until then every one of them went for whoever was home, and
/// with nobody home stood on their pad, or, a few, hit their anchor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Aim {
    /// Their anchor, and whoever is home only as far as it is not set on
    /// it ([`Temper::zeal`]).
    Anchor,
    /// Whoever is home, wherever on the island they are.
    Players,
    /// What lies on their pad, and whoever comes near it.
    Loot,
}

impl Goal {
    /// Whether `other` is the same goal, whatever it is for.
    fn alike(&self, other: &Goal) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Which of a team's two a bot is: the one that keeps the base, or the
/// one that goes out from it. The lower seat keeps; and a bot whose
/// teammate is a person keeps, whichever its seat.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Keeper,
    Runner,
}

/// How a team of the computer's opens (since 2026-11-12; Hajun: "add
/// variation to strategy. some ai might choose to build bridge directly
/// to the center, some ai might choose to bridge from their base to the
/// diamond generators in the side, some ai might choose to bridge
/// later").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Opening {
    /// A bridge to the middle at once, and from the middle to a smaller
    /// island after: the only way they opened until then.
    Middle,
    /// A bridge from its own island straight to a smaller island beside
    /// it, 75 m and four buys, for its sapphire; the middle after, when
    /// it comes to that.
    Sapphire,
    /// Arms first: no bridge until a while in.
    Late,
}

/// What a team of the computer's means to do, as the one of its two that
/// goes out would have it ([`Persona::plan`]): how it opens; from when it
/// builds its first bridge, in seconds into the fight; which of the two
/// smaller islands beside its own it goes for; and from when it builds to
/// the middle, having opened to a smaller island.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Plan {
    pub opening: Opening,
    pub bridge_from: f32,
    pub side: u8,
    pub middle_from: f32,
}

impl Plan {
    /// As every team played until 2026-11-12: to the middle at once.
    const PLAIN: Plan = Plan {
        opening: Opening::Middle,
        bridge_from: 0.0,
        side: 0,
        middle_from: 0.0,
    };
}

/// A way of playing: who a bot is. Each of the computer's players takes
/// one by chance as it is seated, two of a game the same one as likely as
/// not, and plays the whole game by it (since 2026-11-12; Hajun: "at game
/// start the bots each take a certain personality/playstyle that is a
/// predefined set of probability distribution for the if ___ then do ___
/// thing for the various situations. for example like a estp personality
/// might u know, buy the action items, want to engage in combat in a
/// active way and yolo. or like a isfj personality might wanna play safe,
/// just bridging to the diamond islands and not the main land, and
/// upgrading his generator/defenses, or like intj might wanna optimize
/// their stack first and then go battle, or esfp might wanna just troll
/// and buy invisible potion and go to the enemy base and camp and knock
/// them off ... there should be more personality than 8, so that each
/// game feels new and each character shows some distinctive
/// characteristic"). Sixteen of them, by the four letters people sort
/// each other by ([`PERSONAS`]). Every number here is odds or a leaning,
/// not a rule: what it does at each turn is still thrown for.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Persona {
    pub name: &'static str,
    /// How much it wants to be the one of its team that goes out, from 0
    /// to 1: of a team's two, the one that wants it more goes, and the
    /// other keeps the base.
    pub drive: f32,
    /// Its odds of opening with a bridge to the middle, with one straight
    /// to a smaller island, and with none until later ([`Opening`]).
    pub opening: [f32; 3],
    /// Having opened to a smaller island, from when it builds to the
    /// middle of its own accord, in seconds into the fight: the careful
    /// ones leave it a long while.
    pub middle_from: f32,
    /// What it spends on first, each a leaning about 1: its sword, its
    /// shield, something for its second button, its team's generator,
    /// its team's defences (better shields, the heal pool, traps), and
    /// its team's swords.
    pub spends: [f32; 6],
    /// What it carries on its buttons for things to use (five of them
    /// since 2026-11-13; Hajun: "there is 4 more item slot, so the ai
    /// should be adjusted so it can buy up to 5 even. match it with the
    /// personality so more diverse interaction"): its leaning to each
    /// kind, as odds ([`KINDS`]: a fireball, a speed potion, an
    /// invisibility potion, a healing potion, a Bridge Builder, since
    /// 2026-11-13 a stick of dynamite and, since 2026-11-15, a leap
    /// potion), and
    /// how many kinds it carries, at the least and at the most. Which of
    /// them, and in what order it buys them, is thrown by these each life
    /// ([`Persona::loadout`]): the likeliest mostly first. (One kind a
    /// life until then, there being one button.)
    pub carries: [f32; 7],
    pub pockets: [u8; 2],
    /// How much it is one for being out and about, from 0 to 1, when
    /// there is nothing it has to do: out over its team's bridges to look
    /// about, rather than about its own island from one stand to the
    /// other; and how soon it is off its pad again (since 2026-11-15;
    /// Hajun: "atleast they should be standing in the shop so they look
    /// like theyre doing sth or atleast just going around the map like
    /// just a scout").
    pub roams: f32,
    /// How set it is on an anchor it has gone for, from 0 to 1: at 0 it
    /// turns on whoever comes near, as anyone would; at 1 it hits the
    /// anchor and nothing else, whoever is hitting it, until the anchor
    /// is gone or it has to run (since 2026-11-15; Hajun: "there should be
    /// some ai personality that just wanna go to other ppls island and
    /// only attack their anchor too").
    pub zeal: f32,
    /// How much it is one for bridges its team could do without, from 0
    /// to 1: a side door onto another team's island from a smaller one, a
    /// second way onto it from the middle, its own island straight to
    /// theirs ([`Squad::scheme`]); and how much it leans to quickening its
    /// team's building for them (since 2026-11-15; Hajun: "they rly decide
    /// to only make necessary bridges rather than some creative attack
    /// strategy bridges - some ai personality can be like that").
    pub schemes: f32,
    /// How much better than the rest it is in a fight, hand to hand, from
    /// 0 to 1 (since 2026-11-13; Hajun, having played them: "pvp seems
    /// moderate right now for all the bots equally ... i can always win
    /// even in 1v2 ... the bots, depending on personality, need to vary
    /// in pvp skill based on personality so some bots are like pro-er, and
    /// some bots are moderate like now"). At 0 it fights as they all did
    /// until then; at 1 its punches come as fast as they count, each
    /// thrown on the run, from out of your reach and back out of it; it
    /// is on its feet again sooner after a knock; it reads where you are
    /// going rather than where you are; and it is quicker to jump a gap
    /// after you and to get out from under a stick of dynamite
    /// ([`Temper::pvp`]).
    pub pvp: f32,
    /// How many times someone has to come at its base and step back
    /// before it stands its ground against them rather than go back to
    /// what it was doing, about: from 1, at once, to 3 (since 2026-11-13;
    /// Hajun, who had a team's anchor down by stepping onto their island
    /// and back onto his bridge over and over, the two of them coming at
    /// him and going back to their generator each time: "Real humans have
    /// like a patience(?) threshold that depends on the person - some
    /// people, after 1 agitation they choose to just stand between me and
    /// their generator ... some people who have stronger patience, maybe
    /// they will go back to their generator 1 time but after i come back
    /// again to attack they will be like oh this guy keep coming back, so
    /// ill just stay here until he rly rly leaves, and some ppl have more
    /// patience so maybe they will wait until the 3rd agitation"). Each
    /// bot's own is thrown about its kind's, and again each time it has
    /// forgotten whoever it was ([`Vigil`]).
    pub patience: f32,
    /// How readily it goes in, about 1 (how near it lets someone come
    /// before it goes for them, and how hurt it has to be to back off);
    /// how much it jumps about, from 0 to 1; and how widely it weaves as
    /// it walks, about 1.
    pub bold: f32,
    pub bouncy: f32,
    pub weave: f32,
    /// Its leaning to each way of fighting, about 1, on top of what the
    /// place calls for: pressing, circling, poking, waiting, lying in
    /// wait, and rushing ([`Stance`]).
    pub fights: [f32; 6],
    /// How much it goes for throwing people off the edge, and how much it
    /// minds being thrown off itself, each about 1.
    pub shoves: f32,
    pub minds: f32,
    /// How much sooner (under 0) or later than the rest it first goes
    /// raiding, in seconds; how much it likes a raid, added to how it
    /// scores one; how long it stays on one, as so many times the usual;
    /// and how likely it is to go for the anchor there rather than for
    /// whoever is home and what lies on their pad, from 0 to 1.
    pub eager: f32,
    pub raids: f32,
    pub lingers: f32,
    pub anchors: f32,
}

impl Persona {
    /// What it means to carry this life, in the order it buys them, and
    /// how many of each it keeps: as many kinds as its pockets have it,
    /// each thrown by its leanings from the kinds not yet taken. Fireballs
    /// by the handful for one that leans to them, two healing potions for
    /// one that leans on those, two sticks of dynamite for one a fifth of
    /// whose leanings are to it, and one of anything else.
    fn loadout(&self, dice: &mut Dice) -> Vec<(Utility, u8)> {
        // No more kinds than there are buttons for them.
        let all = (KINDS.len().min(crate::UTILITY_SLOTS)) as u8;
        let least = self.pockets[0].clamp(1, all);
        let most = self.pockets[1].clamp(least, all);
        let many = (least + (dice.roll() * f32::from(most - least + 1)) as u8).min(most);
        // (How many of a kind it keeps goes by its leaning to that kind
        // as a share of its leanings to the six there were before the
        // leap potion, which came after and takes nothing from the rest.)
        let all = self.carries.iter().sum::<f32>().max(1e-3);
        let before = (all - self.carries[KINDS.len() - 1]).max(1e-3);
        let mut odds = self.carries;
        let mut carried = Vec::new();
        for _ in 0..many {
            if odds.iter().all(|odd| *odd <= 0.0) {
                break;
            }
            let which = pick(dice, &odds);
            let share = self.carries[which] / if KINDS[which] == Utility::Leap { all } else { before };
            let count = match KINDS[which] {
                Utility::Fireball if share >= 0.35 => 3,
                Utility::Fireball if share >= 0.15 => 2,
                Utility::Healing if share >= 0.45 => 2,
                Utility::Dynamite if share >= 0.2 => 2,
                Utility::Leap if share >= 0.25 => 2,
                _ => 1,
            };
            carried.push((KINDS[which], count));
            odds[which] = 0.0;
        }
        carried
    }

    /// How its team opens, by its odds, if it is the one that goes out.
    fn plan(&self, dice: &mut Dice) -> Plan {
        let opening = [Opening::Middle, Opening::Sapphire, Opening::Late][pick(dice, &self.opening)];
        Plan {
            opening,
            bridge_from: if opening == Opening::Late { dice.around(45.0, 120.0) } else { 0.0 },
            side: (dice.roll() * 2.0) as u8 % 2,
            middle_from: if opening == Opening::Sapphire { self.middle_from } else { 0.0 },
        }
    }
}

/// The kinds of thing there are to carry on a button, in the order a
/// kind's leanings to them are written ([`Persona::carries`]).
pub const KINDS: [Utility; 7] = [
    Utility::Fireball,
    Utility::Speed,
    Utility::Invisibility,
    Utility::Healing,
    Utility::Builder,
    Utility::Dynamite,
    Utility::Leap,
];

/// Which of `odds` comes up: each as likely as it is big.
fn pick(dice: &mut Dice, odds: &[f32]) -> usize {
    let mut left = dice.roll() * odds.iter().sum::<f32>();
    for (which, weight) in odds.iter().enumerate() {
        if *weight > 0.0 && left <= *weight {
            return which;
        }
        left -= weight;
    }
    odds.iter().rposition(|weight| *weight > 0.0).unwrap_or(0)
}

/// Every way of playing there is. My reading of each, from Hajun's four
/// examples on: the numbers are leanings to tune.
pub const PERSONAS: [Persona; 16] = [
    // The daredevil: the action items, into every fight, and never mind
    // the edge.
    Persona {
        name: "ESTP",
        drive: 0.9,
        opening: [0.8, 0.05, 0.15],
        middle_from: 0.0,
        spends: [1.3, 0.6, 1.6, 0.5, 0.5, 0.9],
        carries: [0.5, 0.35, 0.1, 0.05, 0.05, 0.3, 0.3],
        roams: 0.8,
        zeal: 0.2,
        schemes: 0.3,
        pvp: 0.9,
        patience: 1.4,
        pockets: [3, 5],
        bold: 1.4,
        bouncy: 0.8,
        weave: 1.3,
        fights: [1.4, 1.0, 1.2, 0.3, 0.4, 1.8],
        shoves: 1.0,
        minds: 0.5,
        eager: -30.0,
        raids: 0.08,
        lingers: 1.0,
        anchors: 0.3,
    },
    // The troll: an invisibility potion, over to someone's island, and
    // there to lurk by the edge and knock them off it.
    Persona {
        name: "ESFP",
        drive: 0.85,
        opening: [0.7, 0.1, 0.2],
        middle_from: 60.0,
        spends: [0.9, 0.6, 1.8, 0.5, 0.5, 0.6],
        carries: [0.15, 0.2, 0.6, 0.05, 0.1, 0.35, 0.25],
        roams: 0.8,
        zeal: 0.0,
        schemes: 0.45,
        pvp: 0.55,
        patience: 2.4,
        pockets: [2, 4],
        bold: 1.15,
        bouncy: 1.0,
        weave: 1.6,
        fights: [0.7, 1.3, 1.0, 1.0, 2.2, 1.0],
        shoves: 1.8,
        minds: 0.8,
        eager: -15.0,
        raids: 0.07,
        lingers: 1.8,
        anchors: 0.05,
    },
    // The commander: straight for the anchors, armed for it.
    Persona {
        name: "ENTJ",
        drive: 0.8,
        opening: [0.75, 0.05, 0.2],
        middle_from: 0.0,
        spends: [1.5, 1.0, 0.8, 1.1, 0.8, 1.5],
        carries: [0.4, 0.4, 0.05, 0.15, 0.15, 0.15, 0.2],
        roams: 0.5,
        zeal: 0.95,
        schemes: 0.5,
        pvp: 0.8,
        patience: 1.5,
        pockets: [2, 4],
        bold: 1.25,
        bouncy: 0.4,
        weave: 0.8,
        fights: [1.4, 1.0, 1.0, 0.6, 0.8, 1.3],
        shoves: 1.1,
        minds: 1.0,
        eager: -10.0,
        raids: 0.08,
        lingers: 1.1,
        anchors: 0.9,
    },
    // The trickster: odd openings, a fireball, in and out, and a shove.
    Persona {
        name: "ENTP",
        drive: 0.7,
        opening: [0.35, 0.4, 0.25],
        middle_from: 90.0,
        spends: [1.0, 0.7, 1.5, 0.9, 0.7, 0.8],
        carries: [0.45, 0.2, 0.3, 0.05, 0.3, 0.4, 0.35],
        roams: 0.7,
        zeal: 0.3,
        schemes: 0.9,
        pvp: 0.65,
        patience: 2.2,
        pockets: [3, 5],
        bold: 1.1,
        bouncy: 0.7,
        weave: 1.5,
        fights: [0.7, 1.4, 1.5, 0.9, 1.6, 1.1],
        shoves: 1.6,
        minds: 0.9,
        eager: 0.0,
        raids: 0.04,
        lingers: 1.3,
        anchors: 0.3,
    },
    // The captain: a bit of everything, and its team's swords and
    // shields.
    Persona {
        name: "ENFJ",
        drive: 0.65,
        opening: [0.7, 0.1, 0.2],
        middle_from: 60.0,
        spends: [1.1, 1.1, 0.8, 1.2, 1.3, 1.3],
        carries: [0.2, 0.3, 0.05, 0.45, 0.1, 0.05, 0.1],
        roams: 0.5,
        zeal: 0.4,
        schemes: 0.3,
        pvp: 0.4,
        patience: 1.6,
        pockets: [2, 3],
        bold: 1.05,
        bouncy: 0.45,
        weave: 0.9,
        fights: [1.1, 1.1, 1.0, 0.9, 0.9, 1.0],
        shoves: 1.0,
        minds: 1.0,
        eager: 0.0,
        raids: 0.03,
        lingers: 1.0,
        anchors: 0.6,
    },
    // The free spirit: a speed potion, all over the map, jumping.
    Persona {
        name: "ENFP",
        drive: 0.8,
        opening: [0.55, 0.25, 0.2],
        middle_from: 45.0,
        spends: [0.9, 0.7, 1.5, 0.7, 0.7, 0.7],
        carries: [0.2, 0.55, 0.2, 0.05, 0.1, 0.15, 0.6],
        roams: 1.0,
        zeal: 0.1,
        schemes: 0.5,
        pvp: 0.5,
        patience: 2.7,
        pockets: [2, 4],
        bold: 1.2,
        bouncy: 1.0,
        weave: 1.8,
        fights: [1.0, 1.5, 1.2, 0.6, 0.8, 1.5],
        shoves: 1.0,
        minds: 0.6,
        eager: -25.0,
        raids: 0.06,
        lingers: 0.8,
        anchors: 0.3,
    },
    // The enforcer: head on, and on to the anchor.
    Persona {
        name: "ESTJ",
        drive: 0.7,
        opening: [0.8, 0.05, 0.15],
        middle_from: 0.0,
        spends: [1.3, 1.2, 0.7, 1.0, 1.0, 1.2],
        carries: [0.35, 0.25, 0.0, 0.4, 0.05, 0.1, 0.1],
        roams: 0.5,
        zeal: 0.7,
        schemes: 0.1,
        pvp: 0.7,
        patience: 1.3,
        pockets: [2, 3],
        bold: 1.2,
        bouncy: 0.3,
        weave: 0.6,
        fights: [1.6, 0.8, 0.8, 0.7, 0.7, 1.2],
        shoves: 1.0,
        minds: 1.0,
        eager: -5.0,
        raids: 0.05,
        lingers: 1.0,
        anchors: 0.8,
    },
    // The host: its own island first, with traps and a pool for whoever
    // comes.
    Persona {
        name: "ESFJ",
        drive: 0.45,
        opening: [0.65, 0.15, 0.2],
        middle_from: 120.0,
        spends: [1.0, 1.3, 0.9, 1.1, 1.6, 1.0],
        carries: [0.1, 0.15, 0.05, 0.7, 0.1, 0.2, 0.05],
        roams: 0.15,
        zeal: 0.2,
        schemes: 0.1,
        pvp: 0.2,
        patience: 1.5,
        pockets: [1, 3],
        bold: 1.0,
        bouncy: 0.5,
        weave: 1.0,
        fights: [1.2, 1.0, 0.9, 1.1, 1.3, 0.8],
        shoves: 1.1,
        minds: 1.2,
        eager: 25.0,
        raids: -0.02,
        lingers: 0.9,
        anchors: 0.5,
    },
    // The mastermind: everything bought first, and then to battle.
    Persona {
        name: "INTJ",
        drive: 0.5,
        opening: [0.35, 0.25, 0.4],
        middle_from: 150.0,
        spends: [1.4, 1.3, 0.7, 1.6, 1.0, 1.4],
        carries: [0.3, 0.3, 0.1, 0.3, 0.3, 0.25, 0.2],
        roams: 0.35,
        zeal: 0.6,
        schemes: 0.7,
        pvp: 0.6,
        patience: 1.8,
        pockets: [3, 5],
        bold: 1.0,
        bouncy: 0.25,
        weave: 0.6,
        fights: [1.0, 1.0, 1.3, 1.2, 1.3, 0.7],
        shoves: 1.3,
        minds: 1.3,
        eager: 55.0,
        raids: 0.05,
        lingers: 1.2,
        anchors: 0.8,
    },
    // The tinkerer: its generator, something clever for its button, and
    // no hurry to fight.
    Persona {
        name: "INTP",
        drive: 0.3,
        opening: [0.3, 0.3, 0.4],
        middle_from: 180.0,
        spends: [0.9, 0.9, 1.3, 1.7, 1.1, 0.9],
        carries: [0.4, 0.2, 0.25, 0.15, 0.45, 0.4, 0.3],
        roams: 0.3,
        zeal: 0.3,
        schemes: 0.8,
        pvp: 0.2,
        patience: 2.5,
        pockets: [3, 5],
        bold: 0.8,
        bouncy: 0.3,
        weave: 1.1,
        fights: [0.7, 1.0, 1.4, 1.4, 1.2, 0.6],
        shoves: 1.2,
        minds: 1.2,
        eager: 40.0,
        raids: -0.02,
        lingers: 0.9,
        anchors: 0.5,
    },
    // The watchman: careful, and waiting for you beside your own bridge.
    Persona {
        name: "INFJ",
        drive: 0.25,
        opening: [0.4, 0.4, 0.2],
        middle_from: 240.0,
        spends: [0.9, 1.3, 0.9, 1.1, 1.6, 0.8],
        carries: [0.1, 0.1, 0.1, 0.7, 0.1, 0.3, 0.05],
        roams: 0.2,
        zeal: 0.2,
        schemes: 0.2,
        pvp: 0.35,
        patience: 1.1,
        pockets: [1, 3],
        bold: 0.8,
        bouncy: 0.2,
        weave: 0.8,
        fights: [0.8, 0.9, 1.0, 1.4, 1.8, 0.5],
        shoves: 1.3,
        minds: 1.4,
        eager: 45.0,
        raids: -0.03,
        lingers: 0.8,
        anchors: 0.4,
    },
    // The dreamer: out of everyone's way for as long as it can be.
    Persona {
        name: "INFP",
        drive: 0.2,
        opening: [0.3, 0.45, 0.25],
        middle_from: 300.0,
        spends: [0.7, 1.2, 1.0, 1.2, 1.3, 0.6],
        carries: [0.05, 0.25, 0.3, 0.4, 0.1, 0.1, 0.2],
        roams: 0.45,
        zeal: 0.0,
        schemes: 0.3,
        pvp: 0.0,
        patience: 2.8,
        pockets: [1, 2],
        bold: 0.65,
        bouncy: 0.5,
        weave: 1.5,
        fights: [0.6, 1.1, 1.1, 1.6, 1.3, 0.6],
        shoves: 0.8,
        minds: 1.5,
        eager: 70.0,
        raids: -0.08,
        lingers: 0.6,
        anchors: 0.2,
    },
    // The sentinel: by the book. The middle, a sword, a shield.
    Persona {
        name: "ISTJ",
        drive: 0.4,
        opening: [0.8, 0.05, 0.15],
        middle_from: 0.0,
        spends: [1.2, 1.3, 0.7, 1.2, 1.2, 1.0],
        carries: [0.15, 0.1, 0.0, 0.75, 0.05, 0.05, 0.0],
        roams: 0.25,
        zeal: 0.5,
        schemes: 0.0,
        pvp: 0.4,
        patience: 1.2,
        pockets: [1, 2],
        bold: 0.95,
        bouncy: 0.15,
        weave: 0.5,
        fights: [1.3, 0.8, 0.9, 1.2, 1.0, 0.7],
        shoves: 0.9,
        minds: 1.3,
        eager: 20.0,
        raids: 0.0,
        lingers: 1.0,
        anchors: 0.7,
    },
    // The defender: safe. The smaller islands and not the middle, its
    // generator, and its team's defences.
    Persona {
        name: "ISFJ",
        drive: 0.15,
        opening: [0.1, 0.8, 0.1],
        middle_from: 420.0,
        spends: [0.8, 1.5, 0.9, 1.5, 1.7, 0.6],
        carries: [0.05, 0.05, 0.0, 0.9, 0.3, 0.25, 0.05],
        roams: 0.1,
        zeal: 0.1,
        schemes: 0.2,
        pvp: 0.1,
        patience: 1.0,
        pockets: [1, 3],
        bold: 0.7,
        bouncy: 0.2,
        weave: 0.7,
        fights: [0.8, 0.8, 0.8, 1.6, 1.6, 0.5],
        shoves: 0.9,
        minds: 1.5,
        eager: 60.0,
        raids: -0.06,
        lingers: 0.7,
        anchors: 0.3,
    },
    // The lone wolf: a sword and its own two feet. In and out, round
    // you, and off the edge with you.
    Persona {
        name: "ISTP",
        drive: 0.6,
        opening: [0.6, 0.2, 0.2],
        middle_from: 60.0,
        spends: [1.5, 0.8, 1.0, 0.7, 0.6, 0.9],
        carries: [0.35, 0.35, 0.2, 0.1, 0.05, 0.15, 0.25],
        roams: 0.85,
        zeal: 0.3,
        schemes: 0.4,
        pvp: 1.0,
        patience: 1.7,
        pockets: [1, 3],
        bold: 1.15,
        bouncy: 0.5,
        weave: 1.0,
        fights: [0.9, 1.4, 1.6, 0.9, 1.2, 1.0],
        shoves: 1.5,
        minds: 1.1,
        eager: -15.0,
        raids: 0.05,
        lingers: 1.0,
        anchors: 0.4,
    },
    // The drifter: here and there, a potion in hand, and never mind the
    // anchors.
    Persona {
        name: "ISFP",
        drive: 0.35,
        opening: [0.35, 0.45, 0.2],
        middle_from: 150.0,
        spends: [0.9, 1.0, 1.3, 0.9, 1.0, 0.7],
        carries: [0.1, 0.4, 0.3, 0.2, 0.1, 0.1, 0.4],
        roams: 0.95,
        zeal: 0.0,
        schemes: 0.3,
        pvp: 0.25,
        patience: 2.6,
        pockets: [2, 4],
        bold: 0.85,
        bouncy: 0.7,
        weave: 1.6,
        fights: [0.8, 1.3, 1.2, 1.2, 1.0, 0.8],
        shoves: 0.9,
        minds: 1.2,
        eager: 30.0,
        raids: -0.03,
        lingers: 0.8,
        anchors: 0.2,
    },
];

/// How one bot is, thrown as it takes its seat about what its way of
/// playing has ([`Persona`]), so that no two of a kind are quite alike:
/// how readily it goes in, how much it jumps about, how widely it weaves,
/// how much it goes for throwing people off the edge and how much it
/// minds being thrown off.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Temper {
    pub bold: f32,
    pub bouncy: f32,
    pub weave: f32,
    pub shoves: f32,
    pub minds: f32,
    /// How much better than the rest it is in a fight, from 0 to 1: its
    /// kind's ([`Persona::pvp`]), a little more or less.
    pub pvp: f32,
    /// How much it is one for being out and about, and how set it is on
    /// an anchor it has gone for, each from 0 to 1: its kind's
    /// ([`Persona::roams`], [`Persona::zeal`]), a little more or less.
    pub roams: f32,
    pub zeal: f32,
}

/// A jump a bot is making over a gap ([`Body::leap`]): where it means to
/// come down, how fast it runs at it, until when it keeps to it whatever
/// comes of it, and whether it has left the ground yet.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Leap {
    to: Vec2,
    pace: f32,
    until: f32,
    aloft: bool,
}

/// Where someone was last seen to be and when they last moved, how fast
/// and which way they were going, and how fast that was turning, in
/// radians a second: what a bot reads where they will be from
/// ([`Squad::foresee`]). People's places come a few times a second
/// online, and every frame offline: both are taken as they come.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct Track {
    known: bool,
    at: Vec2,
    since: f32,
    velocity: Vec2,
    /// How they went between the last two places they were seen at, not
    /// smoothed: what the turning is read off.
    last: Vec2,
    turning: f32,
}

/// What a bot has made of the things someone dropped beside it as they
/// died ([`Squad::mind_spoils`]): when they were dropped, which says whose
/// death it was; whether it is going for them; from how near; with nobody
/// of another team within how far; and until when.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Grab {
    born: f32,
    goes: bool,
    within: f32,
    clear: f32,
    until: f32,
}

/// A bot getting off a bridge that is coming down
/// ([`Squad::mind_falling`]): when it wakes up to it; how much time it
/// keeps in hand over each plank on its way; the way off it has found, if
/// there is one, as the places to run through ([`Nav::escape`]); and how
/// many blasts the map had seen, and when it was, as it last looked for
/// one.
#[derive(Clone, PartialEq, Debug)]
struct Flight {
    wakes: f32,
    spare: f32,
    route: Option<Vec<Vec2>>,
    cuts: u32,
    asked: f32,
}

/// A stick of dynamite a bot can see: where it lies, or will come down;
/// how long until it blows, in seconds, its flight and its fuse together;
/// and whose it is, by team.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Stick {
    pub at: Vec2,
    pub left: f32,
    pub team: u8,
}

/// How a bot is fighting whoever it fights, for a few seconds at a time
/// (since 2026-11-12; Hajun: "think of some way to randomize this
/// interaction ... so that there is no 'optimal' way to kill bots, and
/// there is constantly surprise/different random fighting interactions").
/// Until then every bot ran straight at whoever it fought and stood in
/// front of them, wherever that was: on the last plank of a bridge, with
/// them to one side of its end, which is where one punch put it in the
/// void.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stance {
    /// Up to them, wherever it is best to stand ([`Squad::spot`]), and at
    /// them.
    Press,
    /// Round them as it punches, one way or the other.
    Circle,
    /// In for a punch and back out of their reach, to come in again on
    /// the run.
    Poke,
    /// Standing off out of their reach: let them come.
    Wait,
    /// On the land beside where their bridge comes ashore, to one side:
    /// whoever steps off it is thrown back over its side.
    Ambush,
    /// Off the bridge it is on and past them onto the land, at a run.
    Rush,
}

/// A fight with one foe: how the bot is going about it and until when,
/// which way round it goes, where it means to stand and until when it
/// keeps to that, until when it is stepping back from a punch, and
/// whether each of them was out on a bridge when it chose.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Duel {
    foe: usize,
    stance: Stance,
    until: f32,
    side: f32,
    spot: Option<Vec2>,
    spot_until: f32,
    back_until: f32,
    me_out: bool,
    foe_out: bool,
    /// When it makes up its mind about jumping a gap to get at them;
    /// until when, having thought better of it, it goes round by the land
    /// instead; and whether the way round was far, when it last looked,
    /// with where they stood then.
    leap_at: f32,
    round_until: f32,
    far: Option<(bool, Vec2)>,
    /// It stepped back from its last punch to come in again as the next
    /// falls due: that one is thrown the moment they are in its reach,
    /// on the run, with no waiting to take it in that they are.
    lunge: bool,
}

/// A bridge a bot is laying: the span, which of its bridges it is, which
/// rung comes next, and how many blasts the map had seen when it was last
/// looked over for holes ([`Nav::cuts`]).
#[derive(Clone, Copy, PartialEq, Debug)]
struct Laying {
    span: Span,
    from: Region,
    to: Region,
    chain: u16,
    next: usize,
    cuts: u32,
}

/// A Bridge Builder a bot has sent off: the bridge it is laying, straight
/// from the one land to the other, which of the bot's bridges that is,
/// which rung comes next, and how far along it the builder has walked.
/// Its bridge goes up a plank or two ahead of where it is, and whoever
/// runs the game shows the builder there ([`Squad::builder_at`]; until
/// 2026-11-14 nobody saw it, and the bridge went up by itself).
#[derive(Clone, Copy, PartialEq, Debug)]
struct Sent {
    span: Span,
    from: Region,
    to: Region,
    chain: u16,
    next: usize,
    along: f32,
}

/// One of the computer's players.
#[derive(Clone, Debug)]
pub struct Bot {
    pub body: Body,
    seat: usize,
    role: Role,
    goal: Goal,
    goal_since: f32,
    think_at: f32,
    /// Its way to where its goal takes it: the points still ahead, where
    /// they lead, and the map as it was when the way was found.
    path: Vec<Vec2>,
    path_to: Option<Vec2>,
    path_version: u32,
    path_at: f32,
    /// Whom it is fighting, since when they have been in its reach, and
    /// when it may next punch.
    foe: Option<usize>,
    in_reach_since: Option<f32>,
    punch_at: f32,
    /// How many metres of bridge it has, how many bridges it has begun,
    /// and the one it is laying.
    metres: f32,
    chains: u16,
    laying: Option<Laying>,
    /// Until when it is drinking a potion, and which; when it last
    /// finished one; and how near the island it is raiding it drinks a
    /// leap potion, to go in over their heads, in metres, thrown again
    /// each time it has.
    drinking: Option<(f32, Utility)>,
    drank_at: f32,
    leap_near: f32,
    /// How far it would go just now for what someone who died has
    /// dropped ([`SPOILS_WITHIN`]): thrown as something first lies
    /// anywhere, so that a game in which nothing has been dropped yet
    /// rolls no dice for it.
    spoils_within: Option<f32>,
    /// What it has made of the things someone dropped beside it
    /// ([`Squad::mind_spoils`]).
    grab: Option<Grab>,
    /// What its health was a frame ago, and when it last lost any.
    health_was: f32,
    hurt_at: f32,
    /// Where it was, and when, for telling that it is getting nowhere;
    /// and what it has given up on, until when.
    progress: (Vec2, f32),
    barred: Option<(Goal, f32)>,
    tries: u8,
    /// What it stands at a stand to buy, and when it will have made up
    /// its mind and bought it ([`SHOP_SECS`]).
    browsing: Option<(Ware, f32)>,
    /// How often it has got nowhere on a way it had and thought again
    /// ([`Squad::walk_to`]), and where it last did, heading where: for
    /// telling, over whole games, that nothing holds a body fast.
    stalls: u32,
    stalled: Option<(Vec2, Vec2)>,
    /// How many of those times it could not take the next step of its
    /// way at all: held by a wall, or by the lack of ground.
    pinned: u32,
    /// When it last looked along the straight walk to the next point of
    /// its way ([`LOOK_EVERY`]).
    looked_at: f32,
    /// When it was last on its way anywhere ([`Squad::walk_to`]): after a
    /// while of anything else, standing its ground or fighting where it
    /// stood, it is not one that has been getting nowhere.
    walked_at: f32,
    /// How many of each thing it would take from a pad now, and how much
    /// it matters.
    wants: Purse,
    urgency: u8,
    /// How fast it went, the frame before.
    speed: f32,
    /// Until when it stays home from raids; since when it has been on the
    /// island it is raiding; and when it may next hit an anchor, and
    /// throw a fireball.
    rest_until: f32,
    raiding_since: Option<f32>,
    strike_at: f32,
    throw_at: f32,
    /// Who it is ([`Persona`]), and how it is itself ([`Temper`]).
    persona: &'static Persona,
    temper: Temper,
    /// What it means to carry on its buttons this life, in the order it
    /// buys them, and how many of each ([`Persona::loadout`]); and, on a
    /// raid, what it is there for ([`Aim`]), how long it stays this time,
    /// and when it next thinks again of what it is there for.
    loadout: Vec<(Utility, u8)>,
    aim: Aim,
    raid_lasts: f32,
    aim_again: f32,
    /// From when it goes raiding, from when an anchor is fair game to it,
    /// until when it mostly leaves the anchors be once a couple are
    /// broken, and from when, keeping its team's base, it goes out on a
    /// raid all the same: each its own, thrown as it takes its seat about
    /// what all of them go by ([`Skill::raid_from`] and the rest).
    raids_from: f32,
    anchors_from: f32,
    spares_until: f32,
    keeper_from: f32,
    /// Whether it is one for going out just now, with nothing it has to
    /// do, and until when that holds ([`MOOD_LASTS`]); until when it stays
    /// in, having been out to look about; how many more places it means to
    /// go on to this time, and until when it looks about at the one it
    /// has come to ([`Goal::Scout`]).
    out: bool,
    out_until: f32,
    scout_again: f32,
    stops: u8,
    looking: Option<f32>,
    /// At its pad: how many of the things it wants have to lie there for
    /// it to come this time, and how long it will stand there for the
    /// rest ([`PAD_WORTH`], [`PAD_WAIT`]).
    pad_worth: u16,
    pad_wait: f32,
    /// The bridge its last thought settled on laying next, if any
    /// ([`Squad::next_span`]); and a Bridge Builder it has sent off, and
    /// how far that has got.
    planned: Option<(Region, Region, Span)>,
    sent: Option<Sent>,
    /// A bridge its team could do without that it has settled on laying
    /// ([`Squad::scheme`]); when it next thinks of one; how many it has
    /// begun; and the team whose island it has just built its way onto,
    /// and until when it goes in after it whoever's turn it is.
    scheme: Option<(Region, Region, Span)>,
    scheme_at: f32,
    schemed: u8,
    door: Option<(u8, f32)>,
    /// How it is fighting whoever it fights; and whom it has given up on
    /// for now, until when.
    duel: Option<Duel>,
    shun: Option<(usize, f32)>,
    /// How long the knock that has just landed carries it before it
    /// answers, to be counted from its next frame; until when that is;
    /// and how much of its pace it steers by in the air this time.
    reel: Option<f32>,
    reel_until: f32,
    grip: f32,
    /// How long after having someone in reach its next punch comes; when
    /// it may next jump; and how far round its weaving is as it walks.
    windup: f32,
    hop_at: f32,
    weave: f32,
    /// Which way it is to face this frame, if not the way it walks.
    face: Option<f32>,
    /// A person of its team is at their pad: what the pad makes is theirs
    /// ([`Squad::wants`]).
    yields: bool,
    /// With nothing to do, where on its island it is wandering to, and
    /// until when.
    loiter: Option<(Vec2, f32)>,
    /// A gap it is jumping ([`Leap`]); a hole in a bridge it is on its
    /// way to, to jump it ([`Nav::crossing`]); and how many blasts the map
    /// had seen when it last found its way ([`Nav::cuts`]).
    leaping: Option<Leap>,
    cross: Option<Gap>,
    path_cuts: u32,
    /// Whoever it is fighting is on their way over a gap to it, this
    /// frame: it jumps none itself until they are down ([`Squad::fight`]).
    awaits: bool,
    /// A stick of dynamite it is in the way of, and when it wakes up to
    /// it; and when it may next throw one of its own.
    alarm: Option<(Vec2, f32)>,
    blast_at: f32,
    /// On a bridge that is coming down: how it is getting off it
    /// ([`Flight`]). And, on a raid, whether it means to burn the bridge
    /// the people of that island would come home over
    /// ([`Squad::burn_behind`]), thrown as it steps ashore.
    flight: Option<Flight>,
    burns: bool,
    /// Out looking about, whether it means to cut a bridge of another
    /// team's that it has come to the end of ([`Squad::cut_off`]), thrown
    /// each time it stops to look.
    cuts: bool,
    /// Whose way it has gone out to cut with the stick of dynamite on it,
    /// and where it means the stick to come down
    /// ([`Squad::way_to_cut`]); until when it means to cut one, having
    /// thought of it ([`CUT_MEANT`]); and when it next thinks of it
    /// ([`CUT_AGAIN`]).
    cutting: Option<(u8, Vec2)>,
    means_to_cut: Option<f32>,
    cut_again: f32,
    /// Left on planks with no land it can get to ([`Squad::think`]):
    /// from when it has had enough of that; and, having had enough, which
    /// way it steps off them.
    marooned: Option<f32>,
    steps_off: Option<Vec2>,
    /// Beside a person: since when it has been at its team's pad on this
    /// visit, and when it may next come for what lies there
    /// ([`Goal::Collect`]).
    pad_since: Option<f32>,
    pad_again: f32,
    /// Whoever has been coming at its base, and how it is taking it
    /// ([`Vigil`]); how many of their comings it takes before it stands
    /// its ground ([`Persona::patience`]); and what its team's anchor had
    /// left when it last thought.
    vigil: Option<Vigil>,
    patience: u8,
    anchor_was: f32,
    /// Until when it is on its way home for its anchor, which was being
    /// hit when it last looked.
    called: Option<f32>,
    /// How near it lets people come before it goes for them, as so many
    /// times what it usually does, for the while until `keen_until`; how
    /// long the blow it last took smarts; and how far off its usual it
    /// backs off hurt this time ([`KEEN`], [`STING`], [`NERVE`]).
    keen: f32,
    keen_until: f32,
    sting: f32,
    nerve: f32,
}

impl Bot {
    fn new(seat: usize, body: Body, dice: &mut Dice, persona: &'static Persona, skill: &Skill) -> Self {
        let temper = Temper {
            bold: (persona.bold * (1.0 + 0.1 * dice.bell())).clamp(0.5, 1.6),
            bouncy: (persona.bouncy + 0.1 * dice.bell()).clamp(0.0, 1.0),
            weave: (persona.weave * (1.0 + 0.15 * dice.bell())).clamp(0.3, 2.0),
            shoves: persona.shoves,
            minds: persona.minds,
            pvp: (persona.pvp + 0.07 * dice.bell()).clamp(0.0, 1.0),
            roams: (persona.roams + 0.1 * dice.bell()).clamp(0.0, 1.0),
            zeal: (persona.zeal + 0.08 * dice.bell()).clamp(0.0, 1.0),
        };
        let loadout = persona.loadout(dice);
        // When its raids begin, when an anchor is fair game to it, and the
        // rest: its own, about what they all go by. One set on anchors
        // comes to them sooner.
        let raids_from = skill.raid_from + persona.eager + dice.around(-18.0, 18.0);
        let anchors_from = dice.around(skill.anchors_from.0, skill.anchors_from.1) - 45.0 * temper.zeal;
        let spares_until = dice.around(skill.anchors_spared_until.0, skill.anchors_spared_until.1);
        let keeper_from = dice.around(skill.keeper_raids_from.0, skill.keeper_raids_from.1) - 60.0 * (persona.drive - 0.5);
        Self {
            body,
            seat,
            role: if seat < ARENA_TEAMS { Role::Keeper } else { Role::Runner },
            goal: Goal::Idle,
            goal_since: 0.0,
            // Not all at once.
            think_at: dice.between(0.0, 1.0),
            path: Vec::new(),
            path_to: None,
            path_version: 0,
            path_at: 0.0,
            foe: None,
            in_reach_since: None,
            punch_at: 0.0,
            metres: 0.0,
            chains: 0,
            laying: None,
            drinking: None,
            drank_at: f32::NEG_INFINITY,
            leap_near: dice.around(16.0, 40.0),
            spoils_within: None,
            grab: None,
            health_was: 1.0,
            hurt_at: f32::NEG_INFINITY,
            progress: (body.at.xz(), 0.0),
            stalls: 0,
            stalled: None,
            pinned: 0,
            looked_at: f32::MIN,
            walked_at: f32::MIN,
            barred: None,
            tries: 0,
            browsing: None,
            wants: [0; 4],
            urgency: 0,
            speed: 0.0,
            rest_until: 0.0,
            raiding_since: None,
            strike_at: 0.0,
            throw_at: 0.0,
            persona,
            temper,
            loadout,
            aim: Aim::Players,
            raid_lasts: 45.0,
            aim_again: 0.0,
            raids_from,
            anchors_from,
            spares_until,
            keeper_from,
            out: false,
            out_until: 0.0,
            scout_again: dice.between(0.0, SCOUT_AGAIN.1),
            stops: 0,
            looking: None,
            pad_worth: dice.around(PAD_WORTH.0, PAD_WORTH.1).round() as u16,
            pad_wait: dice.around(PAD_WAIT.0, PAD_WAIT.1),
            planned: None,
            sent: None,
            scheme: None,
            scheme_at: dice.between(0.0, 40.0),
            schemed: 0,
            door: None,
            duel: None,
            shun: None,
            reel: None,
            reel_until: f32::NEG_INFINITY,
            grip: 1.0,
            windup: 0.3,
            hop_at: dice.between(1.0, 4.0),
            weave: dice.between(0.0, std::f32::consts::TAU),
            face: None,
            yields: false,
            loiter: None,
            leaping: None,
            awaits: false,
            cross: None,
            path_cuts: 0,
            alarm: None,
            flight: None,
            burns: false,
            cuts: false,
            cutting: None,
            means_to_cut: None,
            cut_again: 0.0,
            marooned: None,
            steps_off: None,
            blast_at: 0.0,
            pad_since: None,
            pad_again: 0.0,
            vigil: None,
            patience: patience_of(persona, dice),
            anchor_was: f32::INFINITY,
            called: None,
            keen: dice.around(KEEN.0, KEEN.1),
            keen_until: dice.between(KEEN_LASTS.0, KEEN_LASTS.1),
            sting: 3.0,
            nerve: dice.around(NERVE.0, NERVE.1),
        }
    }

    /// Whoever it has in mind for coming at its base, if anyone.
    pub fn vigil(&self) -> Option<Vigil> {
        self.vigil
    }

    /// How many comings at its base it takes before it stands its ground.
    pub fn patience(&self) -> u8 {
        self.patience
    }

    /// Whether it is standing its ground against whoever keeps coming at
    /// its base, rather than only looking after them a moment.
    pub fn stands_its_ground(&self) -> bool {
        self.vigil.is_some_and(|vigil| vigil.stand.is_some_and(|stand| stand.resolved))
    }

    /// Whether it is in the middle of jumping a gap.
    pub fn leaping(&self) -> bool {
        self.leaping.is_some()
    }

    pub fn goal(&self) -> Goal {
        self.goal
    }

    /// What it is on the island it is raiding for, as it last threw it
    /// ([`Aim`]).
    pub fn aim(&self) -> Aim {
        self.aim
    }

    /// Whether it is one for going out just now, with nothing it has to
    /// do ([`MOOD_LASTS`]).
    pub fn out(&self) -> bool {
        self.out
    }

    /// Whether it is getting off a bridge that is coming down
    /// ([`Squad::mind_falling`]).
    pub fn fleeing(&self) -> bool {
        self.flight.is_some()
    }

    /// Whose way it has gone out to cut with the stick of dynamite on
    /// it, if it has ([`Squad::way_to_cut`]): for telling, over whole
    /// games, how often they go and what comes of it.
    pub fn cutting(&self) -> Option<u8> {
        self.cutting.map(|(victim, _)| victim)
    }

    /// Whether, left on planks with no land it could get to, it has
    /// stepped off them into the void ([`Squad::think`]): for telling that
    /// apart, over whole games, from one that walked off an edge.
    pub fn stepped_off(&self) -> bool {
        self.steps_off.is_some()
    }

    /// How often it has got nowhere on a way it had, three seconds
    /// together, and thought again; and where it last did, and the point
    /// it was walking to. A bot that stands its ground with somewhere to
    /// go counts among these.
    pub fn stalls(&self) -> (u32, Option<(Vec2, Vec2)>) {
        (self.stalls, self.stalled)
    }

    /// How many of those times something held it: the next step of its
    /// way could not be taken. What must never be.
    pub fn pinned(&self) -> u32 {
        self.pinned
    }

    /// Who it is.
    pub fn persona(&self) -> &'static Persona {
        self.persona
    }

    /// How it is itself.
    pub fn temper(&self) -> Temper {
        self.temper
    }

    /// How it is fighting whoever it fights, if it is fighting.
    /// What it means to carry on its buttons this life, in the order it
    /// buys them, and how many of each.
    /// Where the builder it has sent off is now, if it has one out, across
    /// the ground, and which way it walks ([`Squad::builder_at`]).
    pub fn builder_at(&self) -> Option<(Vec2, Vec2)> {
        let sent = self.sent?;
        let way = sent.span.way();
        Some((sent.span.from + way * sent.along.min(sent.span.chord()), way))
    }

    pub fn loadout(&self) -> &[(Utility, u8)] {
        &self.loadout
    }

    /// Whether a Bridge Builder it sent off is still laying its bridge.
    pub fn building(&self) -> bool {
        self.sent.is_some()
    }

    pub fn stance(&self) -> Option<Stance> {
        self.duel.map(|duel| duel.stance)
    }

    pub fn role(&self) -> Role {
        self.role
    }

    /// How many metres of bridge it has to lay.
    pub fn metres(&self) -> f32 {
        self.metres
    }

    fn set_goal(&mut self, goal: Goal, now: f32) {
        if self.goal != goal {
            self.goal = goal;
            self.goal_since = now;
            self.path.clear();
            self.path_to = None;
            self.tries = 0;
            self.progress = (self.body.at.xz(), now);
        }
    }
}

/// A bot's mind of someone who has come at its base: on the land of its
/// island, or at its anchor as that lost health. Who, and how many times
/// they have come, counted each time they come again after having stepped
/// back; whether they are at it now; when they were last about; how long
/// after that it will have forgotten them; and how it stands toward them
/// while they are only stepped back ([`Stand`]).
///
/// Until 2026-11-13 a bot had no memory of anyone: it went home for
/// whoever stood on its island, and back to what it was doing the moment
/// they stepped off it, as often as they liked. Now it looks after them a
/// moment each time; and once they have come as many times as it has the
/// patience for, which is 1 to 3 by who it is and by chance, it stands
/// its ground: between them and its anchor, at them if they come near
/// enough, there until they are gone a good way off for a while. Standing
/// there long enough with them still about, it either goes out at them or
/// lets them be, as it is like, and it has not forgotten them either way:
/// the next time they come, it stands at once.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vigil {
    pub foe: usize,
    pub comings: u8,
    at_it: bool,
    about: f32,
    forget: f32,
    stand: Option<Stand>,
}

/// How a bot stands toward someone who was at its base and has stepped
/// back: where, between them and what it keeps; until when, at the
/// longest; whether it has had enough of them and stands its ground, or
/// only looks after them a moment; how far from it they may be for it to
/// go at them; how far from the middle of its island they have to be to
/// be gone, for how long, and since when they have been.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Stand {
    spot: Vec2,
    until: f32,
    resolved: bool,
    watch: f32,
    gone: f32,
    calm: f32,
    gone_since: Option<f32>,
    off: f32,
    inside: f32,
}

/// Where a bot stands between what its team keeps, at `keep`, and someone
/// `there` who has stepped back from it: on the land of its island
/// `home`, on the line from them to it, the nearest place to them that a
/// body can stand which is `off` or more from them and `inside` or more
/// in from where that line comes ashore; or, with too little land between
/// the shore and what it keeps for that, the last place on that land
/// before what it keeps, right in front of it. `None` with no land
/// between them at all: they are on the island still, or beside what it
/// keeps.
fn guard_spot(nav: &Nav, home: Region, keep: Vec2, there: Vec2, off: f32, inside: f32) -> Option<Vec2> {
    let line = keep - there;
    let far = line.length();
    if far < 1.0 {
        return None;
    }
    let way = line / far;
    let ashore = |at: Vec2| nav.grid().region(at) == Some(home) && nav.grid().height(at).is_some() && nav.walkable_at(at);
    let mut landed: Option<f32> = None;
    let mut last = None;
    let mut along = 0.0;
    // (No nearer what it keeps than a body and a half: that is in the
    // way of nothing.)
    while along <= far - 1.3 {
        let at = there + way * along;
        if ashore(at) {
            let from = *landed.get_or_insert(along);
            if along >= off && along - from >= inside {
                return Some(at);
            }
            // (Not on the very edge, at the least.)
            if along - from >= 0.5 {
                last = Some(at);
            }
        } else {
            landed = None;
        }
        along += 0.25;
    }
    last
}

/// How many times someone has to come at a bot's base before it stands
/// its ground, thrown about its kind's: 1, 2 or 3.
fn patience_of(persona: &Persona, dice: &mut Dice) -> u8 {
    (persona.patience + 0.6 * dice.bell()).round().clamp(1.0, 3.0) as u8
}

/// Someone in the game, as a bot sees them: where they are, whether they
/// are standing, in play, unseen, a person, and what share of their
/// health they have.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Seen {
    pub at: Vec3,
    pub alive: bool,
    pub in_play: bool,
    pub unseen: bool,
    pub person: bool,
    pub health: f32,
}

/// The game as the bots see it, for one frame.
pub struct View<'a> {
    /// Seconds into the fight.
    pub elapsed: f32,
    pub nav: &'a Nav,
    /// Everyone, by seat, the zombies past the players'.
    pub seats: &'a [Seen],
    pub purses: &'a [Purse],
    pub outfits: &'a Outfits,
    pub mint: &'a Mint,
    pub anchors: [f32; ARENA_TEAMS],
    /// Every stick of dynamite thrown and not yet blown.
    pub sticks: &'a [Stick],
}

/// What a frame of a bot comes to that the game is to carry out.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Order {
    /// A punch at whoever is in this seat, thrown on the run if `running`.
    pub punch: Option<usize>,
    pub running: bool,
    /// A punch at this team's anchor.
    pub strike: Option<u8>,
    /// Buy this, standing where it is.
    pub buy: Option<Ware>,
    /// Throw a fireball, as a plank heads.
    pub throw: Option<f32>,
    /// Throw a stick of dynamite, as a plank heads, to come down this far
    /// off, in metres.
    pub blast: Option<(f32, f32)>,
    /// This potion, off its button, is drunk.
    pub drink: Option<Utility>,
    /// A Bridge Builder is sent off: one comes off its button. Its bridge
    /// comes as `rungs`, a frame at a time, as one the bot walks does.
    pub build: bool,
    /// Rungs it lays: which of its bridges, which rung, where, and heading
    /// how.
    pub rungs: Vec<(u16, u16, Vec2, f32)>,
    /// How it is to be seen: carrying a plank, or with a potion at its
    /// mouth.
    pub carrying: bool,
    pub drinking: bool,
}

/// Where a team's bots stand for what is on their island.
#[derive(Clone, Copy, Debug, Default)]
struct Spots {
    heart: Vec2,
    pad: Vec2,
    items: Vec2,
    upgrades: Vec2,
    guard: Vec2,
    /// Where another team's bot stands to hit this island's anchor: on
    /// the side of it the bridges come from.
    strike: Vec2,
    /// What the team keeps: its anchor, where the map has it on the
    /// island, and the middle of the island otherwise.
    keep: Vec2,
    /// Where one of them stands to look out from its island: by the
    /// shore its bridges to the middle leave from. And where, in the
    /// middle, the way from this island comes ashore, about: where one out
    /// looking about stands to see who comes and goes.
    lookout: Vec2,
    rim: Vec2,
}

/// Every one of the computer's players in a game.
#[derive(Clone, Debug)]
pub struct Squad {
    pub skill: Skill,
    bots: Vec<Option<Bot>>,
    /// Which spans are being laid, and by whom: one bot to a span.
    claims: Vec<(Region, Region, usize)>,
    /// How many bridges have been begun across each span, so that one
    /// begun after another was left unfinished lies beside it.
    begun: Vec<(Region, Region, u8)>,
    spots: Option<[Spots; ARENA_TEAMS]>,
    /// Who is raiding each team, by seat; until when each team is left
    /// alone, having been raided; how many raids there have been, and how
    /// many of them on a team with a person on it; and who is at each
    /// team's anchor. The raids are steered a little, so that nobody is
    /// mobbed and nobody is out in the first minutes.
    raids: [Vec<usize>; ARENA_TEAMS],
    spared_until: [f32; ARENA_TEAMS],
    raided: (u32, u32),
    strikers: [Option<usize>; ARENA_TEAMS],
    /// What this game's choices of whom to raid are thrown with, so that
    /// who goes for whom first is not the same in every game.
    salt: usize,
    /// How each team means to open ([`Plan`]).
    plans: [Plan; ARENA_TEAMS],
    /// How everyone in the game has been moving, by seat, and when that
    /// was last looked at ([`Squad::watch`]).
    tracks: Vec<Track>,
    watched_at: f32,
    dice: Dice,
}

/// The other seat of `seat`'s team.
fn mate_of(seat: usize) -> usize {
    (seat + ARENA_TEAMS) % ARENA_SEATS
}

impl Squad {
    pub fn new(seed: u64, skill: Skill) -> Self {
        let mut dice = Dice::new(seed);
        Self {
            skill,
            bots: vec![None; ARENA_SEATS],
            claims: Vec::new(),
            begun: Vec::new(),
            spots: None,
            raids: Default::default(),
            spared_until: [0.0; ARENA_TEAMS],
            raided: (0, 0),
            strikers: [None; ARENA_TEAMS],
            salt: (dice.roll() * 11.0) as usize,
            plans: [Plan::PLAIN; ARENA_TEAMS],
            tracks: Vec::new(),
            watched_at: f32::NEG_INFINITY,
            dice,
        }
    }

    /// How `team` means to open.
    pub fn plan(&self, team: usize) -> Plan {
        self.plans[team % ARENA_TEAMS]
    }

    /// Every water a bridge has been begun across by one of them, from
    /// which land to which, and how many times: for telling, over whole
    /// games, what they build.
    pub fn begun(&self) -> &[(Region, Region, u8)] {
        &self.begun
    }

    /// Has `team` open as `plan` has it: for trying one out.
    pub fn set_plan(&mut self, team: usize, plan: Plan) {
        self.plans[team % ARENA_TEAMS] = plan;
    }

    /// Has the bot in `seat` be who is called `name` ([`PERSONAS`]), and
    /// its team open as the one of its two that goes out would: for
    /// trying one out. Whether there is such a bot and such a one.
    pub fn cast(&mut self, seat: usize, name: &str) -> bool {
        let Some(persona) = PERSONAS.iter().find(|persona| persona.name == name) else {
            return false;
        };
        let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) else {
            return false;
        };
        let body = bot.body;
        *bot = Bot::new(seat, body, &mut self.dice, persona, &self.skill);
        self.settle(usize::from(arena_team(seat)));
        true
    }

    /// Works out again how `team` means to open: as the one of its bots
    /// that goes out would have it, the one of the two that wants to
    /// more.
    fn settle(&mut self, team: usize) {
        let lead = [team, team + ARENA_TEAMS]
            .into_iter()
            .filter_map(|seat| self.bots.get(seat).and_then(Option::as_ref).map(|bot| (bot.persona, seat)))
            .max_by(|a, b| a.0.drive.total_cmp(&b.0.drive).then(a.1.cmp(&b.1)));
        if let Some((persona, _)) = lead {
            self.plans[team % ARENA_TEAMS] = persona.plan(&mut self.dice);
        }
    }

    /// The bot in `seat` is raiding nobody any more, `now`: a team nobody
    /// is raiding any longer is left alone for a while.
    fn call_off(&mut self, seat: usize, now: f32) {
        for team in 0..ARENA_TEAMS {
            let before = self.raids[team].len();
            self.raids[team].retain(|&raider| raider != seat);
            if before > 0 && self.raids[team].is_empty() {
                let rest = self.skill.victim_rest;
                self.spared_until[team] = now + self.dice.around(rest.0, rest.1);
            }
            if self.strikers[team] == Some(seat) {
                self.strikers[team] = None;
            }
        }
    }

    /// The computer takes `seat`, standing `at`: as someone of its own,
    /// by chance ([`PERSONAS`]).
    pub fn seat(&mut self, seat: usize, at: Vec3, yaw: f32) {
        if seat < ARENA_SEATS {
            let who = ((self.dice.roll() * PERSONAS.len() as f32) as usize).min(PERSONAS.len() - 1);
            self.bots[seat] = Some(Bot::new(seat, Body::standing(at, yaw), &mut self.dice, &PERSONAS[who], &self.skill));
            self.settle(usize::from(arena_team(seat)));
        }
    }

    /// Nobody of the computer's is in `seat` any more.
    pub fn unseat(&mut self, seat: usize) {
        if let Some(bot) = self.bots.get_mut(seat) {
            *bot = None;
        }
        self.claims.retain(|claim| claim.2 != seat);
        self.call_off(seat, 0.0);
    }

    /// The bot in `seat`, if the computer has it.
    pub fn bot(&self, seat: usize) -> Option<&Bot> {
        self.bots.get(seat).and_then(Option::as_ref)
    }

    /// Where the builder the bot in `seat` has sent off is now, if it has
    /// one out, across the ground, and which way it walks: along the
    /// bridge it is laying, as far as it has walked. For whoever runs the
    /// game to show it there (since 2026-11-14; Hajun: "make your
    /// bridgebot visible to other players too").
    pub fn builder_at(&self, seat: usize) -> Option<(Vec2, Vec2)> {
        self.bot(seat)?.builder_at()
    }

    /// The bot in `seat` stands `at`: as the fight begins, and each time
    /// it stands again.
    pub fn stand(&mut self, seat: usize, at: Vec3, yaw: f32, now: f32) {
        let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) else {
            return;
        };
        bot.body = Body::standing(at, yaw);
        bot.health_was = 1.0;
        bot.foe = None;
        bot.in_reach_since = None;
        bot.drinking = None;
        bot.barred = None;
        bot.duel = None;
        bot.shun = None;
        bot.reel = None;
        bot.reel_until = f32::NEG_INFINITY;
        bot.face = None;
        bot.leaping = None;
        bot.cross = None;
        bot.alarm = None;
        bot.flight = None;
        bot.marooned = None;
        bot.steps_off = None;
        bot.cutting = None;
        bot.means_to_cut = None;
        bot.looking = None;
        bot.stops = 0;
        bot.pad_since = None;
        if let Some(vigil) = &mut bot.vigil {
            vigil.stand = None;
        }
        // Another life, and perhaps other things for its buttons.
        bot.loadout = bot.persona.loadout(&mut self.dice);
        bot.planned = None;
        bot.goal = Goal::Idle;
        bot.goal_since = now;
        bot.path.clear();
        bot.path_to = None;
        bot.progress = (at.xz(), now);
        bot.think_at = now + self.dice.between(0.1, 0.6);
    }

    /// The bot in `seat` has died: the metres of bridge it had are gone
    /// with the rest of what it held, and the bridge it was laying is
    /// anyone's to finish.
    pub fn died(&mut self, seat: usize, now: f32) {
        let rest = self.dice.around(self.skill.raid_rest.0, self.skill.raid_rest.1);
        let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) else {
            return;
        };
        bot.metres = 0.0;
        bot.laying = None;
        // A builder it had sent off goes with it, its bridge left as far
        // as it got.
        bot.sent = None;
        bot.planned = None;
        bot.scheme = None;
        bot.door = None;
        bot.drinking = None;
        bot.foe = None;
        bot.duel = None;
        bot.leaping = None;
        bot.cross = None;
        bot.alarm = None;
        bot.flight = None;
        bot.marooned = None;
        bot.steps_off = None;
        bot.cutting = None;
        bot.means_to_cut = None;
        // Whoever it stood against it has not forgotten; where it stood
        // it has.
        if let Some(vigil) = &mut bot.vigil {
            vigil.stand = None;
        }
        bot.wants = [0; 4];
        // Dead on a raid, it stays home a while once it stands again.
        if matches!(bot.goal, Goal::Raid(_)) {
            bot.rest_until = now + rest;
        }
        bot.raiding_since = None;
        self.claims.retain(|claim| claim.2 != seat);
        self.call_off(seat, now);
    }

    /// The bot in `seat` has been knocked, from `from`. How long it is
    /// carried before it answers, and how well it steers when it does,
    /// are thrown again each time ([`Skill::reel`], [`Skill::grip`]).
    pub fn knock(&mut self, seat: usize, from: Vec2, running: bool) {
        let (reel, grip) = self.reeling(seat);
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.body.knock(from, running, 1.0);
            bot.reel = Some(reel);
            bot.grip = grip;
            // A jump it was making is whatever the knock makes of it.
            bot.leaping = None;
        }
    }

    /// How long a knock landing now carries the bot in `seat` before it
    /// answers, and how much of its pace it then steers by in the air:
    /// the better it is in a fight, the sooner and the surer
    /// ([`Temper::pvp`]): one that is as good as they come is never
    /// carried longer than the quickest third of what the rest may be,
    /// and never steers by less than most of its pace.
    fn reeling(&mut self, seat: usize) -> (f32, f32) {
        let skill = self.skill;
        let pvp = self.bot(seat).map_or(0.0, |bot| bot.temper.pvp);
        let slowest = skill.reel.0 + (skill.reel.1 - skill.reel.0) * (1.0 - 0.7 * pvp);
        let weakest = skill.grip.0 + (skill.grip.1 - skill.grip.0) * 0.7 * pvp;
        (self.dice.around(skill.reel.0, slowest), self.dice.around(weakest, skill.grip.1))
    }

    /// The bot in `seat` has been knocked by this much, across the ground
    /// and upward, as whoever runs the game has worked it out for itself:
    /// a game played offline, where a knock is the file's numbers and
    /// lands on the body the device draws. On top of half of whatever
    /// already carried it, as [`Body::knock`] has it.
    pub fn shoved(&mut self, seat: usize, across: Vec2, up: f32) {
        let (reel, grip) = self.reeling(seat);
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.body.shove = bot.body.shove * 0.5 + across;
            bot.body.lift = (bot.body.lift * 0.5 + up).min(up.max(0.0));
            bot.reel = Some(reel);
            bot.grip = grip;
            bot.leaping = None;
        }
    }

    /// The Bridge Builder the bot in `seat` meant to send off
    /// ([`Order::build`]) was not on its buttons after all, as whoever
    /// runs the game found: no bridge is laid for it. Nothing is free.
    pub fn unsend(&mut self, seat: usize) {
        if let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) {
            bot.sent = None;
        }
    }

    /// The bot in `seat` has what it asked the shop for.
    pub fn bought(&mut self, seat: usize, ware: Ware, now: f32) {
        let Some(bot) = self.bots.get_mut(seat).and_then(Option::as_mut) else {
            return;
        };
        if ware == Ware::Bridge {
            bot.metres += BRIDGE_A_BUY;
        }
        bot.tries = 0;
        bot.think_at = now;
    }

    /// How many of each thing the bot in `seat` would take from a pad it
    /// stands on, and how much it matters to its team that it has them:
    /// whoever of a team wants them most is given them first. Nothing
    /// while a person of its team is at their pad, though it stands on it
    /// with them: what the pad makes is theirs (Hajun, 2026-11-12: "just
    /// let it stay on the generator even while ur collecting but the item
    /// collection goes to the player"; until then it stepped off as they
    /// came and back on as they left).
    pub fn wants(&self, seat: usize) -> (Purse, u8) {
        self.bot(seat)
            .filter(|bot| !bot.yields)
            .map_or(([0; 4], 0), |bot| (bot.wants, bot.urgency))
    }

    /// Where each team's bots stand on their island, worked out once.
    fn spots(&mut self, nav: &Nav) -> [Spots; ARENA_TEAMS] {
        if let Some(spots) = self.spots {
            return spots;
        }
        let grid = nav.grid();
        let mut spots = [Spots::default(); ARENA_TEAMS];
        for (team, spot) in spots.iter_mut().enumerate() {
            let island = ARENA_ISLANDS[team];
            let near = |at: Vec2| grid.nearest_clear(at, 4.0).unwrap_or(at);
            spot.heart = nav.heart(Region::Team(team as u8));
            spot.pad = near(ARENA_PADS[team]);
            spot.items = near(ARENA_SHOPS[team]);
            spot.upgrades = near(ARENA_UPGRADE_SHOPS[team]);
            // Between the anchor and the island's middle, a little to one
            // side: by the anchor, and clear of where its team arrives.
            let anchor = ARENA_ANCHORS[team];
            let inward = (island - anchor).normalize_or_zero();
            spot.guard = near(anchor + inward * 2.4 + Vec2::new(-inward.y, inward.x) * 1.6);
            spot.strike = near(anchor - inward * 1.9);
            let on_it = grid.nearest_clear(anchor, 4.0).is_some_and(|by| grid.region(by) == Some(Region::Team(team as u8)));
            spot.keep = if on_it { anchor } else { spot.heart };
            // Toward the middle of the map from the island's own: a couple
            // of metres in from its shore, and a few in from the middle's.
            let toward = (-island).normalize_or_zero();
            let own = |at: Vec2| grid.region(at) == Some(Region::Team(team as u8));
            spot.lookout = (0..24)
                .map(|step| island + toward * (crate::ARENA_ISLAND_RADIUS - 1.5 - step as f32 * 0.5))
                .find_map(|at| grid.nearest_clear(at, 1.5).filter(|at| own(*at)))
                .unwrap_or(spot.heart);
            let middle = nav.heart(Region::Middle);
            spot.rim = (0..80)
                .map(|step| island + toward * (crate::ARENA_ISLAND_RADIUS + 20.0 + step as f32))
                .find(|at| grid.region(*at) == Some(Region::Middle))
                .and_then(|shore| grid.nearest_clear(shore + toward * 4.0, 4.0))
                .unwrap_or(middle);
        }
        self.spots = Some(spots);
        spots
    }

    /// One frame of the bot in `seat`: it thinks, if it is time; it acts
    /// on what it thought; its body is moved; and what the game is to
    /// carry out of it is given back.
    pub fn step(&mut self, seat: usize, view: &View, dt: f32) -> Order {
        let mut order = Order::default();
        self.watch(view);
        let Some(mut bot) = self.bots.get_mut(seat).and_then(Option::take) else {
            return order;
        };
        let me = view.seats.get(seat).copied().unwrap_or_default();
        if me.health < bot.health_was - 1e-3 {
            bot.hurt_at = view.elapsed;
            // How long this one smarts: not the same as the last.
            bot.sting = self.dice.around(STING.0, STING.1);
            // A drink is spilt by a blow.
            bot.drinking = None;
        }
        bot.health_was = me.health;
        let before = bot.body.at.xz();
        if !me.alive {
            self.bots[seat] = Some(bot);
            return order;
        }
        // What its pad makes is a person's while they are at it.
        let mate = view.seats.get(mate_of(seat)).copied().unwrap_or_default();
        let pad = ARENA_PADS[usize::from(arena_team(seat)) % ARENA_TEAMS];
        bot.yields = mate.person && mate.alive && mate.at.xz().distance(pad) < 3.5;
        // A knock that has landed since its last frame carries it for a
        // moment before it answers.
        if let Some(reel) = bot.reel.take() {
            bot.reel_until = view.elapsed + reel;
        }
        // A builder it sent off lays on, wherever the bot itself is.
        build_on(&mut bot, builder_speed(&view.outfits.team(seat)), dt, &mut order);
        if view.elapsed >= bot.think_at {
            self.think(&mut bot, view);
        }
        // Its legs are its own in the air and with a knock on it, as
        // anyone's are.
        let walk = self.act(&mut bot, view, dt, &mut order);
        bot.body.step(view.nav, walk, dt);
        if let Some(face) = bot.face.take() {
            bot.body.yaw = face;
        }
        bot.speed = bot.body.at.xz().distance(before) / dt.max(1e-4);
        self.bots[seat] = Some(bot);
        order
    }

    /// Looks at where everyone is, once a frame, whichever bot's turn it
    /// is first: how fast and which way each is going, and how fast that
    /// is turning, each smoothed a little, from where they were when they
    /// last moved ([`Track`]). What a bot reads where someone will be
    /// from ([`Squad::foresee`]).
    fn watch(&mut self, view: &View) {
        let now = view.elapsed;
        if now <= self.watched_at {
            return;
        }
        self.watched_at = now;
        if self.tracks.len() < view.seats.len() {
            self.tracks.resize(view.seats.len(), Track::default());
        }
        for (track, seen) in self.tracks.iter_mut().zip(view.seats) {
            let at = seen.at.xz();
            let begun = Track {
                known: seen.alive,
                at,
                since: now,
                ..Track::default()
            };
            if !seen.alive || !track.known {
                *track = begun;
                continue;
            }
            let moved = at - track.at;
            let dt = now - track.since;
            if moved.length_squared() < 1e-6 {
                // Nothing new for a while is standing still: online a
                // person's place comes only while it changes.
                if dt > 0.25 {
                    *track = begun;
                }
                continue;
            }
            // Sent somewhere, not walked there: begun again from it.
            if dt < 1e-4 || moved.length() > 8.0 {
                *track = begun;
                continue;
            }
            let fresh = moved / dt;
            if track.last.length() > 2.0 && fresh.length() > 2.0 {
                let rate = (track.last.angle_to(fresh) / dt).clamp(-6.0, 6.0);
                track.turning += (rate - track.turning) * (1.0 - (-dt / 0.25).exp());
            } else {
                track.turning *= 0.5;
            }
            track.velocity += (fresh - track.velocity) * (1.0 - (-dt / 0.1).exp());
            track.last = fresh;
            track.at = at;
            track.since = now;
        }
    }

    /// Where whoever is in `seat`, now `at`, will be in `secs` seconds if
    /// they go on as they are going: on round the bend they are on, if
    /// they are turning, and no further than there is ground under them.
    /// `at` itself for someone standing, or not seen moving.
    fn foresee(&self, nav: &Nav, seat: usize, at: Vec2, secs: f32) -> Vec2 {
        let Some(track) = self.tracks.get(seat).filter(|track| track.known && track.velocity.length() >= 1.0) else {
            return at;
        };
        let mut velocity = track.velocity;
        let mut place = at;
        let mut left = secs;
        while left > 1e-3 {
            let step = left.min(0.05);
            velocity = Vec2::from_angle(track.turning * step).rotate(velocity);
            let next = place + velocity * step;
            if nav.ground(next).is_none() {
                break;
            }
            place = next;
            left -= step;
        }
        place
    }

    /// How fast whoever is in `seat` is going, in metres a second.
    fn speed_of(&self, seat: usize) -> f32 {
        self.tracks.get(seat).filter(|track| track.known).map_or(0.0, |track| track.velocity.length())
    }

    /// Where the bot heads to get at whoever is in `foe`, now `there`,
    /// going at `pace`: not where they are but where it can be on them,
    /// the soonest it could be within a punch of where they will be then
    /// ([`Squad::foresee`], no further ahead than [`FORESEE`]). Someone
    /// running round and round is cut off across the round, where running
    /// at where they are only ever follows them (Hajun, 2026-11-13:
    /// "instead of attacking i just run around in a circle, the AI always
    /// is one step behind so he just keeps following in a circle behind
    /// me in an endless loop"). How much of that it goes by is how good
    /// it is in a fight: all of it for the best, a little over half for
    /// the rest.
    fn meet(&self, nav: &Nav, bot: &Bot, foe: usize, there: Vec2, pace: f32) -> Vec2 {
        let here = bot.body.at.xz();
        if self.speed_of(foe) < 1.0 {
            return there;
        }
        let mut meet = there;
        let mut secs = 0.1;
        while secs <= FORESEE {
            meet = self.foresee(nav, foe, there, secs);
            if here.distance(meet) - RING <= pace.max(1.0) * secs {
                break;
            }
            secs += 0.1;
        }
        there.lerp(meet, 0.55 + 0.45 * bot.temper.pvp)
    }

    /// How far across a gap a bot going at `pace` jumps: what it crosses
    /// in the air, less what it keeps in hand ([`JUMP_SPARE`]); `high`,
    /// with a leap potion on it ([`LEAP_SECS_ALOFT`], [`LEAP_SPARE`]).
    fn reach_of_a_jump(pace: f32, high: bool) -> f32 {
        if high { pace * LEAP_SECS_ALOFT - LEAP_SPARE } else { pace * JUMP_SECS - JUMP_SPARE }
    }

    /// Whether a bot at a run can get from `a` to `b`: walking, or walking
    /// and jumping one hole in a bridge on the way ([`Nav::crossing`]).
    fn reachable(nav: &Nav, a: Vec2, b: Vec2) -> bool {
        nav.connected(a, b) || nav.crossing(a, b, Self::reach_of_a_jump(RUN_SPEED, false)).is_some()
    }

    /// What a bridge that is coming down makes of the way the bot was
    /// going to walk (since 2026-11-15: a stick of dynamite takes the
    /// planks in its blast at once, and dooms every other plank of the
    /// bridge, which fall one after another from the hole outward,
    /// [`Nav::cascade`]). Nothing, while no bridge is.
    ///
    /// **On ground that is staying** it does not step onto planks that
    /// are going: it stops at their edge. No way of its is found over
    /// them ([`Nav::path`] leaves them out from the moment they are
    /// doomed), so this is only for a straight walk at someone, or out of
    /// a blast.
    ///
    /// **On planks that are going**, once it has taken that in
    /// ([`FALL_WAKES`]), it drops whatever it was about, the plank in its
    /// arms, the potion at its mouth, the bridge it was laying, and runs
    /// the way off: the shortest over planks that will still be there as
    /// it comes to each, with a little in hand, to ground that is staying
    /// ([`Nav::escape`], asked again as planks fall). With no such way,
    /// the falling being between it and everything that stays, it jumps
    /// for whatever ground is a jump away ([`Nav::leap`]: five times as
    /// far with a leap potion on it); and with nothing to jump to, it
    /// runs from the falling for as far as there are planks, as anyone
    /// would. One laying a bridge that it has the metres to finish goes
    /// on laying it: the planks it lays from now on are staying.
    fn mind_falling(&mut self, bot: &mut Bot, view: &View, walk: Vec2, order: &mut Order) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        if !nav.crumbling() {
            bot.flight = None;
            return walk;
        }
        let here = bot.body.at.xz();
        let going = |at: Vec2| nav.grid().height(at).is_none() && !nav.stays_at(at) && nav.falls_in(at).is_some();
        // Where it stands is going: planks out over the void; or the rim
        // of the land under their end, where a body's middle is walked
        // only for the planks' being there. No way is found from there
        // while they stand ([`Nav::path`] leaves such squares out with the
        // planks), so one come ashore just there, or stood there looking
        // out as the bridge was blown, goes on to ground that is staying:
        // until this was so it stood on the rim, on land, with no way to
        // anywhere, for the three seconds the last planks took to fall.
        let on_them = going(here) || (nav.walkable_at(here) && !nav.stays_at(here));
        if !on_them {
            bot.flight = None;
            let step = walk.normalize_or_zero();
            if step != Vec2::ZERO && [0.5, 1.0].into_iter().any(|far| going(here + step * far)) {
                // (Waiting there, not getting nowhere.)
                bot.progress = (here, now);
                return Vec2::ZERO;
            }
            return walk;
        }
        let kit = view.outfits.kit(bot.seat);
        let pace = RUN_SPEED * if kit.fast(now) { FAST } else { 1.0 };
        let mut flight = match bot.flight.take() {
            Some(flight) => flight,
            None => Flight {
                wakes: now + self.dice.around(FALL_WAKES.0, FALL_WAKES.1) * (1.15 - 0.6 * bot.temper.pvp),
                spare: self.dice.around(FALL_SPARE.0, FALL_SPARE.1) * bot.temper.minds,
                route: None,
                cuts: u32::MAX,
                asked: f32::NEG_INFINITY,
            },
        };
        if now < flight.wakes {
            bot.flight = Some(flight);
            return walk;
        }
        // Laying a bridge it has the metres to finish: on with it.
        if let Some(laying) = bot.laying {
            let left = laying.span.length() * (1.0 - laying.next.saturating_sub(1) as f32 / laying.span.planks() as f32);
            if bot.metres + 0.5 >= left && order.carrying {
                bot.flight = Some(flight);
                return walk;
            }
            bot.laying = None;
            bot.planned = None;
            bot.scheme = None;
            self.claims.retain(|claim| claim.2 != bot.seat);
        }
        order.carrying = false;
        order.drinking = false;
        bot.drinking = None;
        bot.path.clear();
        bot.path_to = None;
        bot.cross = None;
        bot.progress = (here, now);
        // The way off, looked for again as planks fall, and every so
        // often besides: it may have been knocked off the one it had.
        if flight.cuts != nav.cuts || now - flight.asked > 0.3 {
            flight.route = nav.escape(here, pace, flight.spare);
            flight.cuts = nav.cuts;
            flight.asked = now;
        }
        if let Some(route) = &mut flight.route {
            while route.len() > 1 && route[0].distance(here) <= ARRIVED + 0.15 {
                route.remove(0);
            }
            if let Some(&next) = route.first() {
                bot.flight = Some(flight);
                return (next - here).normalize_or_zero() * pace;
            }
        }
        // No way off in time. Ground a jump away, that is staying and
        // that it can be at the edge for before the edge goes: the
        // nearest.
        let reach = Self::reach_of_a_jump(pace, bot.body.high);
        let jump = (0..16)
            .filter_map(|spoke| {
                let way = Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / 16.0);
                let (edge, down) = nav.leap(here, here + way * (reach + 4.0), reach)?;
                let stays = nav.grid().height(down).is_some() || nav.stays_at(down);
                let in_time = nav.falls_in(edge).is_none_or(|left| left > here.distance(edge) / pace + 0.1);
                (stays && in_time).then_some(down)
            })
            .min_by(|a, b| a.distance_squared(here).total_cmp(&b.distance_squared(here)));
        if let Some(down) = jump
            && bot.body.free(nav)
        {
            bot.leaping = Some(Leap {
                to: down,
                pace,
                until: now + if bot.body.high { 3.5 } else { 2.5 },
                aloft: false,
            });
            bot.flight = Some(flight);
            return (down - here).normalize_or_zero() * pace;
        }
        // Nothing for it but to run from the falling: to whichever of the
        // planks round it has the longest left, if that is longer than
        // where it stands has.
        let left = nav.falls_in(here).unwrap_or(0.0);
        let away = (0..12)
            .filter_map(|spoke| {
                let to = here + Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / 12.0) * 1.4;
                (nav.walkable_at(to) && nav.line(here, to)).then(|| (nav.falls_in(to).unwrap_or(f32::INFINITY), to))
            })
            .filter(|(later, _)| *later > left + 0.05)
            .max_by(|a, b| a.0.total_cmp(&b.0));
        bot.flight = Some(flight);
        away.map_or(Vec2::ZERO, |(_, to)| (to - here).normalize_or_zero() * pace)
    }

    /// The stand the bot buys `ware` at: its own island's, the red-roofed
    /// one or the blue by what it is; or, with no way to that, the
    /// nearest stand of the kind on another island that it can walk to.
    /// Every island's stands sell to anyone who stands at them, and one
    /// cut off from its own, the bridge it came over having come down
    /// behind it, buys the planks to get back by where it is (since
    /// 2026-11-15: until then it stood there with silver in its purse
    /// until the game was over, out of everyone's reach, and the game a
    /// draw). None with no stand it can get to.
    fn stand_for(&mut self, bot: &Bot, nav: &Nav, ware: Ware) -> Option<Vec2> {
        let here = bot.body.at.xz();
        let team = usize::from(arena_team(bot.seat)) % ARENA_TEAMS;
        let items = matches!(crate::bridge_wars::sold_at(ware), crate::bridge_wars::Shop::Items);
        let spots = self.spots(nav);
        let stand = |island: usize| if items { spots[island].items } else { spots[island].upgrades };
        if Self::reachable(nav, here, stand(team)) {
            return Some(stand(team));
        }
        (0..ARENA_TEAMS)
            .filter(|&island| island != team)
            .map(stand)
            .filter(|at| nav.connected(here, *at))
            .min_by(|a, b| a.distance_squared(here).total_cmp(&b.distance_squared(here)))
    }

    /// Whether a stick of dynamite thrown to come down `at` is worth
    /// throwing, now that the whole bridge goes with it (since
    /// 2026-11-15; [`Nav::would_blast`], [`Nav::cascade`]). Not one that
    /// takes no plank. **Not the bridge it stands on itself**, out over
    /// the void, nor one any of its own team stands on: the thrower's
    /// side of it goes down too. Mostly not a bridge off the land it
    /// stands on, away from home: it came over one of them. And **its
    /// own island's bridge**, with someone of another team coming over
    /// it, by who it is: the careful cut their way out to keep their
    /// island, the bold would sooner have the fight; either the likelier
    /// with its back to the wall, seeing someone off, standing its ground
    /// or running from them, and the less while its teammate is away
    /// across it; and never with a person of its team away across it,
    /// who would have no way back.
    ///
    /// `unmanned`, there is nobody on it that it is thrown at, and it is
    /// thrown to leave a team no way over: then only a bridge that comes
    /// to the island of another team that is still in the game, never
    /// its own island's, and never that of a team with a person on it,
    /// who would be left across the water from their own stands.
    fn worth_blowing(&mut self, bot: &Bot, view: &View, at: Vec2, unmanned: bool) -> bool {
        let nav = view.nav;
        let grid = nav.grid();
        let seat = bot.seat;
        let team = arena_team(seat);
        let home = Region::Team(team);
        let taken = nav.would_blast(at, DYNAMITE_RADIUS);
        if taken.is_empty() {
            return false;
        }
        let falls = nav.cascade(&taken);
        let going: Vec<(Vec2, Vec2)> = taken
            .iter()
            .copied()
            .chain(falls.iter().map(|fall| fall.0))
            .filter_map(|(seat, chain, plank)| {
                let rungs = nav.rungs(seat, chain);
                Some((*rungs.get(usize::from(plank))?, *rungs.get(usize::from(plank) + 1)?))
            })
            .collect();
        let on = |at: Vec2| grid.height(at).is_none() && going.iter().any(|&(a, b)| over_plank(at, a, b));
        if on(bot.body.at.xz()) {
            return false;
        }
        let mates = || view.seats.iter().enumerate().filter(move |&(other, seen)| other != seat && seen.alive && arena_team(other) == team);
        if mates().any(|(_, seen)| on(seen.at.xz())) {
            return false;
        }
        if unmanned {
            let lands = |land: Region| going.iter().any(|&(a, b)| [a, b].into_iter().any(|end| grid.height(end).is_some() && grid.region(end) == Some(land)));
            let theirs = (0..ARENA_TEAMS as u8).filter(|&other| other != team && lands(Region::Team(other))).collect::<Vec<u8>>();
            let of = |other: u8| view.seats.iter().enumerate().filter(move |&(seat, _)| seat < ARENA_SEATS && arena_team(seat) == other);
            let fair = !theirs.is_empty() && theirs.iter().all(|&other| of(other).any(|(_, seen)| seen.in_play) && !of(other).any(|(_, seen)| seen.person && seen.in_play));
            return fair && !lands(home);
        }
        // Nor, for most, a bridge that leaves the land it stands on, not
        // being its own island: it came over one of them, and may have no
        // other way back. The bold and the careless do it all the same.
        let here = bot.body.at.xz();
        let stood = grid.region(here).filter(|land| grid.height(here).is_some() && *land != home && *land != Region::Middle);
        if let Some(land) = stood
            && going.iter().any(|&(a, b)| [a, b].into_iter().any(|end| grid.height(end).is_some() && grid.region(end) == Some(land)))
            && self.dice.roll() >= (0.3 + 0.4 * (bot.temper.bold - 1.0) - 0.25 * (bot.temper.minds - 1.0)).clamp(0.05, 0.8)
        {
            return false;
        }
        let ashore = |end: Vec2| grid.height(end).is_some() && grid.region(end) == Some(home);
        if going.iter().any(|&(a, b)| ashore(a) || ashore(b)) {
            let cornered = matches!(bot.goal, Goal::Defend | Goal::Guard | Goal::Retreat);
            let away = |seen: &Seen| grid.region(seen.at.xz()) != Some(home);
            if mates().any(|(_, seen)| seen.person && away(seen)) {
                return false;
            }
            // (More than one of them on it, sooner still: two for the
            // one stick.)
            let on_it = view.seats.iter().enumerate().filter(|&(other, seen)| seen.alive && arena_team(other) != team && on(seen.at.xz())).count();
            let odds = (0.4 + 0.35 * bot.temper.minds - 0.25 * (bot.temper.bold - 1.0) + if cornered { 0.2 } else { 0.0 } + if on_it > 1 { 0.2 } else { 0.0 })
                * if mates().any(|(_, seen)| away(seen)) { 0.4 } else { 1.0 };
            return self.dice.roll() < odds;
        }
        true
    }

    /// Whose way a bot with a stick of dynamite on it goes to cut, where
    /// it stands to do it, and where it means the stick to come down
    /// (since 2026-11-15, a stick taking the whole bridge with it): a
    /// bridge that leaves the middle where the way from another team's
    /// island comes ashore there ([`Spots::rim`]); the bot on the land
    /// beside its end, the stick a few planks out over the void. From
    /// there the bridge falls plank by plank toward their island, slower
    /// than anyone on it runs. Only the way of a team still in the game
    /// with **no person on it**: they would be left on their island with
    /// a bridge to lay again, which is a fair thing to do to the
    /// computer's and not to somebody playing ([`Squad::worth_blowing`]
    /// says the same again as it is thrown, of whatever would go by
    /// then). One of them by chance; none with no such bridge it can get
    /// to.
    fn way_to_cut(&mut self, bot: &Bot, view: &View) -> Option<(u8, Vec2, Vec2)> {
        let nav = view.nav;
        let grid = nav.grid();
        let here = bot.body.at.xz();
        let team = usize::from(arena_team(bot.seat)) % ARENA_TEAMS;
        let spots = self.spots(nav);
        let of = |other: usize| view.seats.iter().enumerate().filter(move |&(seat, _)| seat < ARENA_SEATS && usize::from(arena_team(seat)) == other);
        let fair = |other: usize| other != team && of(other).any(|(_, seen)| seen.in_play) && !of(other).any(|(_, seen)| seen.person && seen.in_play);
        let ashore = |end: Vec2| grid.height(end).is_some();
        let mut ways: Vec<(u8, Vec2, Vec2)> = Vec::new();
        for other in (0..ARENA_TEAMS).filter(|&other| fair(other)) {
            let rim = spots[other].rim;
            // The plank that leaves the middle nearest where their way
            // comes ashore: its end on the land, and its end out.
            let found = nav
                .planks()
                .filter_map(|(a, b)| {
                    let (root, out) = if ashore(a) && !ashore(b) {
                        (a, b)
                    } else if ashore(b) && !ashore(a) {
                        (b, a)
                    } else {
                        return None;
                    };
                    (grid.region(root) == Some(Region::Middle) && root.distance(rim) <= 9.0).then_some((root.distance(rim), root, out))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let Some((_, root, out)) = found else {
                continue;
            };
            let way = (out - root).normalize_or_zero();
            let at = out + way * 3.0;
            let Some(stand) = nav.nearest_walkable(root - way * 2.0) else {
                continue;
            };
            if out_over_the_void(nav, at)
                && grid.height(stand).is_some()
                && (DYNAMITE_NEAREST + 0.5..=DYNAMITE_FURTHEST - 0.5).contains(&stand.distance(at))
                && Self::reachable(nav, here, stand)
            {
                ways.push((other as u8, stand, at));
            }
        }
        if ways.is_empty() {
            return None;
        }
        let which = ((self.dice.roll() * ways.len() as f32) as usize).min(ways.len() - 1);
        Some(ways[which])
    }

    /// Where a bot out looking about throws a stick of dynamite to cut a
    /// bridge it has come to the end of: out on the nearest plank that is
    /// over the void a throw from where it stands, a few metres out from
    /// the land. Whose bridge it is, and whether that is worth a stick,
    /// is [`Squad::worth_blowing`]'s to say.
    fn cut_off(&self, bot: &Bot, view: &View) -> Option<Vec2> {
        let nav = view.nav;
        let here = bot.body.at.xz();
        nav.planks()
            .map(|(a, b)| (a + b) * 0.5)
            .filter(|at| out_over_the_void(nav, *at))
            .map(|at| (at.distance(here), at))
            .filter(|(far, _)| (DYNAMITE_NEAREST + 1.5..=DYNAMITE_FURTHEST - 1.0).contains(far))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, at)| at)
    }

    /// Where a bot on the island of the team `victim`, there for their
    /// anchor with none of them home, throws a stick of dynamite to burn
    /// the bridge they would come home over: out on a plank of a bridge
    /// that leaves their island, a few metres from the shore and a throw
    /// from where it stands. Only one set on anchors does, now and then
    /// ([`Bot::burns`], thrown as it steps ashore), and never to a team
    /// with a person on it: they would be left across the water from
    /// their own shop, with no way to buy the planks to come back by.
    fn burn_behind(&self, bot: &Bot, view: &View, victim: usize) -> Option<Vec2> {
        let nav = view.nav;
        let grid = nav.grid();
        let theirs = Region::Team(victim as u8);
        let here = bot.body.at.xz();
        let of_theirs = |other: usize| other < ARENA_SEATS && usize::from(arena_team(other)) == victim;
        let them = || view.seats.iter().enumerate().filter(|&(other, seen)| of_theirs(other) && seen.in_play);
        let home = them().any(|(_, seen)| seen.alive && grid.region(seen.at.xz()) == Some(theirs));
        if home || them().any(|(_, seen)| seen.person) || !them().any(|(_, seen)| seen.alive) {
            return None;
        }
        let ashore = |end: Vec2| grid.height(end).is_some() && grid.region(end) == Some(theirs);
        nav.planks()
            .filter_map(|(a, b)| {
                let (root, out) = if ashore(a) && !ashore(b) {
                    (a, b)
                } else if ashore(b) && !ashore(a) {
                    (b, a)
                } else {
                    return None;
                };
                let at = out + (out - root).normalize_or_zero() * 2.5;
                let far = at.distance(here);
                (out_over_the_void(nav, at) && (DYNAMITE_NEAREST + 0.3..=DYNAMITE_FURTHEST).contains(&far)).then_some((far, at))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, at)| at)
    }

    /// What the things someone has just dropped beside it make of the way
    /// the bot was going to walk (since the night of 2026-11-15, when the
    /// dead began to drop their sapphire and amethyst where they die on
    /// land, and Hajun asked whether the computer's players collect it
    /// after they kill someone: "if not they should"). With something
    /// dropped lying, or on its way down, no further off than it goes for
    /// such things this time ([`GRAB_WITHIN`]), on ground that is staying
    /// and a straight walk from where it is, it walks there, and from
    /// there to the next of them, until none is left or it has given it
    /// long enough ([`GRAB_LASTS`]); at one that cannot be had yet, it
    /// stands by it for the moment that takes. Not every time
    /// ([`GRAB_ODDS`]), and not with someone of another team upon it
    /// ([`GRAB_CLEAR`]), nor in a fight, with a plank in its arms or a
    /// bridge to lay: those come first. What it was about is as it was:
    /// it goes on with it after. What lies further off it goes for, or
    /// does not, as it goes for anything ([`SPOILS_WITHIN`]).
    fn mind_spoils(&mut self, bot: &mut Bot, view: &View, walk: Vec2, order: &Order) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        if bot.duel.is_some() || order.carrying || bot.laying.is_some() || bot.flight.is_some() || bot.cross.is_some() {
            return walk;
        }
        let here = bot.body.at.xz();
        let nearest = view
            .mint
            .pieces()
            .iter()
            .filter(|piece| piece.dropped() && nav.stays_at(piece.at))
            .map(|piece| (here.distance(piece.at), piece))
            .filter(|(far, _)| *far <= GRAB_WITHIN.1)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let Some((far, piece)) = nearest else {
            return walk;
        };
        // Thrown again for each death's things.
        let grab = match bot.grab {
            Some(grab) if grab.born == piece.born => grab,
            _ => {
                let grab = Grab {
                    born: piece.born,
                    goes: self.dice.roll() < GRAB_ODDS,
                    within: self.dice.around(GRAB_WITHIN.0, GRAB_WITHIN.1),
                    clear: self.dice.around(GRAB_CLEAR.0, GRAB_CLEAR.1),
                    until: now + self.dice.around(GRAB_LASTS.0, GRAB_LASTS.1),
                };
                bot.grab = Some(grab);
                grab
            }
        };
        if !grab.goes || far > grab.within || now > grab.until {
            return walk;
        }
        let team = arena_team(bot.seat);
        let upon_it = view
            .seats
            .iter()
            .enumerate()
            .any(|(other, seen)| seen.alive && seen.in_play && !seen.unseen && arena_team(other) != team && seen.at.xz().distance(here) < grab.clear);
        if upon_it || !nav.line(here, piece.at) {
            return walk;
        }
        // (Getting somewhere, whatever its way to where it was going
        // makes of it.)
        bot.progress = (here, now);
        if far <= ARRIVED {
            return Vec2::ZERO;
        }
        let pace = RUN_SPEED * if view.outfits.kit(bot.seat).fast(now) { FAST } else { 1.0 };
        (piece.at - here).normalize_or_zero() * pace
    }

    /// What a stick of dynamite makes of the way the bot was going to
    /// walk ([`Stick`]). In the way of one, a plank of a bridge under it
    /// and the stick near enough to take it, or the stick another team's
    /// and near enough to hurt, it runs for the nearest ground clear of
    /// the blast, once it has woken up to it, which the better ones do
    /// sooner and anyone in a fight later ([`STICK_WAKES`]): the worst of
    /// them are still in it now and then as it blows. Out of it, it
    /// keeps out until it has blown, whatever it was doing there: it does
    /// not walk back in after whoever it was fighting. Walking toward one
    /// that lies on a bridge, it waits short of it. The three times a
    /// stick shines are there for this. A stick of its own team's on the
    /// land is nothing to it.
    fn mind_sticks(&mut self, bot: &mut Bot, view: &View, walk: Vec2) -> Vec2 {
        if view.sticks.is_empty() {
            bot.alarm = None;
            return walk;
        }
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let team = arena_team(bot.seat);
        let out = out_over_the_void(nav, here);
        let clear = DYNAMITE_RADIUS + BODY + CLEAR_OF_BLAST;
        // The nearest it has to mind: one that would take the floor from
        // under it or hurt it where it stands, or one lying on a bridge
        // that it is walking at; and one it has in mind already it keeps
        // in mind until it has blown, from as far off as it waits for one.
        // (One it is walking at it takes note of from further off than it
        // waits at, so that it has woken up to it by the time it is there.)
        let heading = walk.normalize_or_zero();
        let in_mind = bot.alarm.map(|(at, _)| at);
        let minded = view
            .sticks
            .iter()
            .filter(|stick| {
                let away = stick.at.distance(here);
                let harms = stick.team != team || out;
                let on_a_bridge = nav.on_plank(stick.at) && nav.grid().height(stick.at).is_none();
                let walking_at = heading.dot((stick.at - here).normalize_or_zero()) > 0.3;
                let known = in_mind.is_some_and(|at| at.distance(stick.at) < 0.2);
                (away < clear && harms)
                    || (away < clear + SHORT_OF_BLAST + NOTICED_FROM && on_a_bridge && walking_at)
                    || (away < clear + SHORT_OF_BLAST && harms && known)
            })
            .min_by(|a, b| a.at.distance_squared(here).total_cmp(&b.at.distance_squared(here)))
            .copied();
        let Some(stick) = minded else {
            bot.alarm = None;
            return walk;
        };
        // It wakes up to it a while after it is thrown.
        let woken = match bot.alarm {
            Some((at, when)) if at.distance(stick.at) < 0.2 => when,
            _ => {
                let pvp = bot.temper.pvp;
                let busy = if bot.duel.is_some() { STICK_BUSY } else { 0.0 };
                let soonest = STICK_WAKES.0 - STICK_WAKES_SOONER.0 * pvp + busy;
                let latest = STICK_WAKES.1 - STICK_WAKES_SOONER.1 * pvp + busy;
                let share = if stick.at.distance(here) >= clear { STICK_AHEAD } else { 1.0 };
                let when = now + self.dice.around(soonest, latest) * share;
                bot.alarm = Some((stick.at, when));
                when
            }
        };
        if now < woken {
            return walk;
        }
        let pace = RUN_SPEED * if bot.drinking.is_some() { DRINK_SLOW } else { 1.0 };
        let away = stick.at.distance(here);
        if away >= clear {
            // Clear of it: no nearer it than a little short of the blast
            // until it has blown, whether it lies on the bridge ahead or
            // where the fight was; any other way it goes as it would.
            let nearer = heading.dot((stick.at - here).normalize_or_zero()) > 0.0;
            return if away < clear + SHORT_OF_BLAST && nearer { Vec2::ZERO } else { walk };
        }
        // Out of it: to the nearest place clear of it that can be walked
        // straight to, or failing that straight away from it for as far
        // as there is ground.
        let ring = clear + 0.3;
        let best = (0..16)
            .map(|k| stick.at + Vec2::from_angle(k as f32 * std::f32::consts::TAU / 16.0) * ring)
            .filter(|&to| nav.walkable_at(to) && nav.line(here, to))
            .min_by(|a, b| a.distance_squared(here).total_cmp(&b.distance_squared(here)));
        bot.path.clear();
        bot.path_to = None;
        match best {
            Some(to) => (to - here).normalize_or_zero() * pace,
            None => (here - stick.at).normalize_or(-heading) * pace,
        }
    }

    /// The slow layer: what the bot might be doing, each with a score,
    /// the best of them taken, the one it is on getting a bonus.
    fn think(&mut self, bot: &mut Bot, view: &View) {
        let skill = self.skill;
        let now = view.elapsed;
        bot.think_at = now + self.dice.between(skill.think_every.0, skill.think_every.1);
        let (seat, nav) = (bot.seat, view.nav);
        let team = usize::from(arena_team(seat));
        let home = Region::Team(team as u8);
        let spots = self.spots(nav)[team];
        let me = view.seats[seat];
        let here = bot.body.at.xz();
        let mate = mate_of(seat);
        let mate_seen = view.seats.get(mate).copied().unwrap_or_default();
        let with_person = mate_seen.person && mate_seen.in_play;
        let mate_bot = self.bots[mate].as_ref().filter(|_| mate_seen.in_play);
        // Whom its teammate is raiding, if it is one of the computer's and
        // on a raid.
        let mate_raids = mate_bot.and_then(|mate| match mate.goal {
            Goal::Raid(victim) => Some(usize::from(victim)),
            _ => None,
        });
        // Of a team's two, the one that wants to go out more goes, and the
        // other keeps the base: beside a person, the bot keeps.
        let mate_drive = mate_bot.map(|mate| mate.persona.drive);
        let mate_bot = mate_bot.is_some();
        let mine = bot.persona.drive;
        bot.role = match mate_drive {
            Some(theirs) if !with_person && (mine > theirs || (mine == theirs && seat > mate)) => Role::Runner,
            _ => Role::Keeper,
        };
        // A bot with nobody beside it does both parts.
        let alone = !with_person && !mate_bot;
        let (keeper, runner) = (bot.role == Role::Keeper, bot.role == Role::Runner || alone);
        let purse = view.purses.get(seat).copied().unwrap_or_default();
        let kit = view.outfits.kit(seat);
        let upgrades = view.outfits.team(seat);
        // Walking, or over a hole in a bridge that it can jump: a wider
        // one for one with a leap potion on it, or on its buttons to
        // drink at the hole's edge.
        let jump = Self::reach_of_a_jump(RUN_SPEED, bot.body.high || kit.held(Utility::Leap) > 0);
        let reach = |a: Vec2, b: Vec2| nav.connected(a, b) || nav.crossing(a, b, jump).is_some();
        let can_go_home = reach(here, spots.heart);
        let on_home = nav.grid().region(here) == Some(home) && !nav.on_plank(here);
        let foes: Vec<Seen> = view
            .seats
            .iter()
            .enumerate()
            .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
            .map(|(_, seen)| *seen)
            .collect();
        let nearest_foe = foes.iter().map(|foe| foe.at.xz().distance(here)).fold(f32::INFINITY, f32::min);

        let mut offers: Vec<(f32, Goal)> = vec![(0.05, Goal::Idle)];

        // How near it lets people come, for the next while: nobody keeps
        // to the same line all day ([`KEEN`]).
        if now >= bot.keen_until {
            bot.keen = self.dice.around(KEEN.0, KEEN.1);
            bot.keen_until = now + self.dice.around(KEEN_LASTS.0, KEEN_LASTS.1);
        }
        // Whether it is one for going out just now, when there is
        // nothing it has to do: by how much it roams, the one that keeps
        // the base less than the one that goes out, and less again while
        // its teammate is away from it; thrown again every so often
        // ([`MOOD_LASTS`]), so that the same one is home now and out
        // later.
        if now >= bot.out_until {
            let mate_away = mate_bot && nav.grid().region(mate_seen.at.xz()) != Some(home);
            let odds = match (bot.role, with_person) {
                (_, true) => 0.3 + 0.5 * bot.temper.roams,
                (Role::Runner, _) => 0.4 + 0.55 * bot.temper.roams,
                (Role::Keeper, _) => (0.05 + 0.45 * bot.temper.roams) * if mate_away { 0.5 } else { 1.0 },
            };
            bot.out = self.dice.roll() < odds;
            bot.out_until = now + self.dice.around(MOOD_LASTS.0, MOOD_LASTS.1);
        }

        // Hurt, with someone after it: away, until it has its health back.
        let pressed = nearest_foe < 10.0 || now - bot.hurt_at < bot.sting;
        let mortal = view.anchors[team] <= 0.0;
        // The bolder, the longer it stays in; and a little sooner or later
        // each time than the last ([`NERVE`]).
        let give_way = (if mortal { skill.careful_below } else { skill.retreat_below }) + (1.0 - bot.temper.bold) * 0.15 + bot.nerve;
        if me.health < give_way && pressed {
            offers.push((0.95, Goal::Retreat));
        } else if bot.goal == Goal::Retreat && me.health < skill.recovered + bot.nerve && nearest_foe < 16.0 {
            offers.push((0.9, Goal::Retreat));
        }

        // Someone of another team on its island, standing on the land of
        // it: home, to see them off.
        let on_its_land = |at: Vec3| {
            let across = at.xz();
            nav.grid().region(across) == Some(home) && !nav.on_plank(across) && at.y > -1.0
        };
        let intruder = foes.iter().any(|foe| on_its_land(foe.at));
        if intruder && can_go_home {
            offers.push((if on_home || keeper { 0.88 } else { 0.6 }, Goal::Defend));
        }

        // Whoever is at its base: on the land of its island, or by its
        // anchor as that loses health. It has them in mind from then on,
        // and once they have only stepped back it looks after them, or
        // stands its ground ([`Vigil`]).
        let anchor_left = view.anchors[team];
        let struck = anchor_left < bot.anchor_was - 1e-3;
        let hit = struck && bot.anchor_was.is_finite();
        bot.anchor_was = anchor_left;
        // Its anchor is being hit, as its chat says: home for it, from
        // wherever it is and whatever it was about, if that is not too far
        // to be worth it this time; the nearer, the surer (since
        // 2026-11-15, when they began to be away from their islands more
        // than on them; until then only whoever stood on its land brought
        // one home, and the one that goes out not from a raid).
        if hit && anchor_left > 0.0 && bot.called.is_none_or(|until| now >= until) {
            let odds = (1.7 - here.distance(spots.keep) / 100.0).clamp(0.2, 1.0);
            bot.called = (self.dice.roll() < odds).then(|| now + self.dice.around(5.0, 11.0));
        }
        if bot.called.is_some_and(|until| now < until) && anchor_left > 0.0 && can_go_home {
            offers.push((0.9, Goal::Defend));
        }
        let caller = view
            .seats
            .iter()
            .enumerate()
            .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
            .filter(|(_, seen)| on_its_land(seen.at) || (struck && seen.at.xz().distance(spots.keep) < 5.0))
            .min_by(|a, b| a.1.at.xz().distance_squared(spots.keep).total_cmp(&b.1.at.xz().distance_squared(spots.keep)))
            .map(|(other, _)| other);
        self.keep_vigil(bot, view, caller, on_home, &spots);
        if bot.vigil.is_some_and(|vigil| vigil.stand.is_some()) && can_go_home {
            offers.push((GUARD_SCORE, Goal::Guard));
        }

        // The bridge it means to lay next, and the metres it is short of
        // for it.
        let span = self.next_span(bot, view, runner, with_person);
        // With none it has to build, one its team could do without, for
        // one that is given to them ([`Squad::scheme`]): not beside a
        // person, whose base it keeps, and not before it is armed.
        let span = match span {
            Some(span) => Some(span),
            None => self.scheme(bot, view, runner && !with_person && kit.sword > 0 && me.health >= 0.6),
        };
        let scheming = span.is_some_and(|(from, to, _)| bot.scheme.is_some_and(|scheme| (scheme.0, scheme.1) == (from, to)));
        // With none it has to build, one worth a Bridge Builder, for one
        // that carries them: bought for it when it has the amethyst, and
        // laid by the builder alone. Not beside a person, whose base it
        // keeps.
        let holding = kit.held(Utility::Builder) > 0;
        let carries_builders = bot.loadout.iter().any(|(kind, _)| *kind == Utility::Builder);
        let spare = (span.is_none()
            && runner
            && !with_person
            && bot.laying.is_none()
            && bot.sent.is_none()
            && (holding || carries_builders))
            .then(|| self.spare_span(bot, view))
            .flatten();
        let span = span.or(spare.filter(|_| holding));
        bot.planned = span;
        // A Bridge Builder on one of its buttons lays a new one for it,
        // straight, if it is no longer than a builder goes: no metres of
        // its own.
        let builder = bot.laying.is_none()
            && holding
            && span.is_some_and(|(_, _, span)| span.length() <= BUILDER_METRES - 2.0);
        // What is left of one it is part way along; the whole of a new
        // one, and a little over, for the bends it will have.
        let short = span.filter(|_| !builder).map_or(0.0, |(_, _, span)| {
            let laid = bot.laying.map_or(0, |laying| laying.next.saturating_sub(1));
            let bends = if bot.laying.is_some() { 1.0 } else { MOST_BENT };
            let left = span.length() * bends * (1.0 - laid as f32 / span.planks() as f32);
            (left + 0.5 - bot.metres).max(0.0)
        });
        if let Some((from, to, span)) = span
            && short == 0.0
            && reach(here, span.from)
        {
            // (One it could do without, under whatever there is to be
            // done about its island and over going out for nothing.)
            offers.push((if scheming { 0.66 } else { 0.78 }, Goal::Lay(from, to)));
        }

        // What it would buy, in the order it would: the first it can pay
        // for now it goes to the shop for; what it cannot yet, it saves
        // for at its pad.
        // A team that means to build later arms itself first: both of its
        // two buy as the one that keeps the base does.
        let building_later = self.plans[team].opening == Opening::Late && now < self.plans[team].bridge_from;
        let buys_as = if building_later { Role::Keeper } else { bot.role };
        let list = shopping(
            bot.persona,
            &bot.loadout,
            buys_as,
            alone,
            now,
            &purse,
            &kit,
            &upgrades,
            short,
            bot.laying.is_some(),
            spare.is_some() && !holding,
        );
        let payable = |ware: Ware| price(ware, &kit, &upgrades).is_some_and(|(item, cost)| purse[item.index()] >= cost);
        if let Some(&ware) = list.iter().find(|&&ware| payable(ware)) {
            // (With no way home, at the nearest stand of another island
            // that it can get to: [`Squad::stand_for`].)
            let score = if on_home {
                0.72
            } else if can_go_home {
                0.55
            } else if self.stand_for(bot, nav, ware).is_some() {
                0.5
            } else {
                0.0
            };
            offers.push((score, Goal::Shop(ware)));
        }
        // What it would take from a pad: the price of the first few
        // things on its list, less what it holds.
        let mut wants = [0u16; 4];
        for &ware in list.iter().take(3) {
            if let Some((item, cost)) = price(ware, &kit, &upgrades) {
                wants[item.index()] = wants[item.index()].saturating_add(cost);
            }
        }
        for (item, wanted) in wants.iter_mut().enumerate() {
            *wanted = wanted.saturating_sub(purse[item]);
        }
        // Its team's first bridge comes before anything the keeper wants
        // silver for, from when the team means to build it.
        let plan = self.plans[team];
        let unbridged = !nav.joined(home, Region::Middle)
            && !(plan.opening == Opening::Sapphire
                && beside(team as u8).iter().any(|&island| nav.joined(home, Region::Sapphire(island))));
        let bridging = unbridged && now >= plan.bridge_from && now < plan.bridge_from + 100.0;
        if keeper && !alone && !with_person && bridging {
            wants[Item::Silver.index()] = 0;
        }
        bot.urgency = if short > 0.0 {
            3
        } else if kit.sword == 0 {
            2
        } else {
            1
        };
        let saving = wants[Item::Silver.index()] > 0 || wants[Item::Gold.index()] > 0;
        // On its pad for them. **Beside a person it is no keeper of the
        // pad**: it comes for what lies there once enough of it does,
        // and they are not there for it themselves, picks it up, and is
        // off again, to the shop, a bridge, another pad or a raid, and not
        // back for a while (since 2026-11-13; Hajun: "the bot stays on the
        // iron/gold generator for toooo long ... just come back to pick up
        // resource and leave again quickly not stay"; until then it stood
        // on the pad for as long as it was saving for anything, which was
        // nearly always).
        //
        // **And beside one of its own it is no keeper of the pad either**
        // (since 2026-11-15; Hajun: "the ai seem to be just in their
        // island for too long of the game like idle like both of them on
        // the generator which is not only inefficient but not humanlike").
        // What a pad makes lies on it until someone comes, so a bot comes
        // once enough of what it wants lies there to be worth the walk
        // ([`PAD_WORTH`]), or once the first thing it cannot pay for is
        // all but made; having taken it, it stays only while the rest of
        // that would be made within what it has the patience for this
        // time ([`PAD_WAIT`]), longer for a bridge it is waiting to lay;
        // and is off again, and not back for a while ([`PAD_AGAIN`]).
        // Until then it stood there whenever it was saving for anything,
        // which was nearly always: over half of every game.
        let at_pad = here.distance(spots.pad) < 1.6;
        if !at_pad {
            bot.pad_since = None;
        }
        if saving && can_go_home {
            let lying = [view.mint.held(team), view.mint.held(ARENA_TEAMS + team)];
            let stock = lying[0] + lying[1];
            if with_person {
                let worth = usize::from(wants[Item::Silver.index()] + wants[Item::Gold.index()]).clamp(1, 6);
                let there = bot.goal == Goal::Collect;
                if !bot.yields && now >= bot.pad_again && (there || stock >= worth) {
                    offers.push((0.56, Goal::Collect));
                }
            } else {
                // What lies there of what it would take; and how long the
                // pad takes to make what the first thing it cannot pay
                // for is still short of, with that taken.
                let wanted = [usize::from(wants[Item::Silver.index()]), usize::from(wants[Item::Gold.index()])];
                let takes = lying[0].min(wanted[0]) + lying[1].min(wanted[1]);
                let coming = list
                    .iter()
                    .find(|&&ware| !payable(ware))
                    .and_then(|&ware| price(ware, &kit, &upgrades))
                    .filter(|(item, _)| matches!(item, Item::Silver | Item::Gold))
                    .map_or(f32::INFINITY, |(item, cost)| {
                        let faster = FORGE_SPEED[usize::from(upgrades.forge).min(FORGE_SPEED.len() - 1)];
                        let short = usize::from(cost.saturating_sub(purse[item.index()])).saturating_sub(lying[item.index()]);
                        short as f32 * interval(item, 1).0 / faster
                    });
                // In a hurry for a bridge, or for its first sword, it has
                // more patience with the pad, and comes for less.
                let (patience, worth) = match bot.urgency {
                    3 => (bot.pad_wait * 2.5 + 4.0, 1),
                    2 => (bot.pad_wait * 1.5, usize::from(bot.pad_worth).min(3)),
                    _ => (bot.pad_wait, usize::from(bot.pad_worth)),
                };
                let since = at_pad.then(|| *bot.pad_since.get_or_insert(now));
                let stays = since.is_some_and(|since| takes > 0 || coming <= patience - (now - since));
                let due = !at_pad && (now >= bot.pad_again || bot.urgency == 3);
                let comes = due && (takes >= worth.min(wanted[0] + wanted[1]).max(1) || coming <= patience);
                if stays || comes {
                    offers.push((if takes > 0 { 0.52 } else { 0.48 }, Goal::Collect));
                }
            }
        }

        // Cut off from its own island, what lies on the pad of an island
        // it can get to does as well: silver for the planks to get back
        // by, and gold while it is there.
        if saving && !can_go_home {
            let all = self.spots(nav);
            for other in (0..ARENA_TEAMS).filter(|&other| other != team) {
                let lying = view.mint.held(other) + view.mint.held(ARENA_TEAMS + other);
                if lying > 0 && nav.connected(here, all[other].pad) {
                    offers.push((0.52, Goal::Fetch(all[other].pad, Item::Silver)));
                }
            }
        }

        // Out to a pad for what lies on it: the amethyst in the middle,
        // and the sapphire on the smaller islands. The keeper stays home,
        // unless it is alone, or its teammate is a person: then it goes
        // out for them as the one that goes out does, from early on and
        // with nobody of another team near its island.
        // Its own: the amethyst pad toward its island, and the two smaller
        // islands beside it. The rest are other teams' to go to, until
        // late.
        // (And since 2026-11-15 the one that keeps the base too, while it
        // is one for going out and nobody is near its island.)
        let quiet = nearest_foe > 25.0;
        let late = now >= bot.keeper_from;
        if runner || (with_person && quiet && now > 45.0) || (bot.out && quiet) {
            let base = 0.5;
            for (pad, &at) in AMETHYST_PADS.iter().enumerate() {
                let stock = view.mint.held(12 + pad);
                if stock > 0 && (pad == team || late) && reach(here, at) {
                    let score = base + 0.06 + 0.08 * stock.min(2) as f32 - here.distance(at) / 300.0;
                    offers.push((score, Goal::Fetch(at, Item::Amethyst)));
                }
            }
            let beside = beside(team as u8);
            for (pad, &at) in SAPPHIRE_PADS.iter().enumerate() {
                let stock = view.mint.held(8 + pad);
                if stock > 0 && (beside.contains(&(pad as u8)) || late) && reach(here, at) {
                    let score = base + 0.07 * stock.min(4) as f32 - here.distance(at) / 300.0;
                    offers.push((score, Goal::Fetch(at, Item::Sapphire)));
                }
            }
        }

        // What someone who died has dropped, lying where it came down on
        // ground that is staying: the nearest of it that is no further
        // off than it would go for it just now, and that it can get to,
        // it goes and takes, the sooner the nearer it is and the more of
        // it lies together; and from there the next nearest, until none
        // is left. Under a fight, a raid's landing and its own island's
        // defence, over its pads and its shopping.
        let lying = |piece: &&crate::bridge_wars::Piece| piece.dropped() && piece.lying(now) && nav.stays_at(piece.at);
        if view.mint.pieces().iter().any(|piece| lying(&piece)) {
            let within = *bot.spoils_within.get_or_insert_with(|| self.dice.around(SPOILS_WITHIN.0, SPOILS_WITHIN.1));
            let spilt = view
                .mint
                .pieces()
                .iter()
                .filter(lying)
                .map(|piece| (here.distance(piece.at), piece))
                .filter(|(far, _)| *far <= within)
                .min_by(|a, b| a.0.total_cmp(&b.0));
            match spilt {
                Some((far, piece)) if reach(here, piece.at) => {
                    let heap: u16 = view
                        .mint
                        .pieces()
                        .iter()
                        .filter(|other| other.dropped() && other.at.distance(piece.at) <= 3.0)
                        .map(|other| other.worth)
                        .sum();
                    let score = 0.6 + 0.015 * f32::from(heap.min(10)) + 0.14 * (1.0 - far / within);
                    offers.push((score, Goal::Fetch(piece.at, piece.item)));
                }
                Some(_) => {}
                None => bot.spoils_within = None,
            }
        }

        // The anchors gone, everyone goes for everyone: whoever of another
        // team is nearest that it can get at.
        let in_play = |team: usize| (0..ARENA_SEATS).any(|other| usize::from(arena_team(other)) == team && view.seats.get(other).is_some_and(|seen| seen.in_play));
        if now >= skill.hunt_from && foes.iter().any(|foe| reach(here, foe.at.xz())) {
            offers.push((0.8, Goal::Hunt));
        }

        // A raid: to another team's island, armed and whole, when its
        // turn comes. The one that goes out goes; the keeper with it,
        // late on; and a person's teammate by itself, a little after the
        // rest have begun and with nobody of another team near its own
        // island (since 2026-11-13, Hajun: "make it go out more ... or
        // raiding an island more"; never by itself until then).
        // Sooner or later than the rest, by who it is.
        // And since 2026-11-15 the one that keeps the base by itself too,
        // from a time of its own, while it is one for going out and
        // nobody is near its island: a team's two are not one at home
        // and one away all game.
        let rested = now >= bot.rest_until && me.health >= 0.7 && kit.sword > 0;
        let goes_out = runner
            || (with_person && quiet && now >= bot.raids_from.max(skill.raid_from) + 20.0)
            || (bot.out && quiet && now >= bot.keeper_from);
        if now >= bot.raids_from && now < skill.hunt_from && rested {
            let others = (0..ARENA_TEAMS).filter(|&other| other != team && in_play(other)).count();
            for victim in (0..ARENA_TEAMS).filter(|&victim| victim != team && in_play(victim)) {
                let mine = matches!(bot.goal, Goal::Raid(on) if usize::from(on) == victim);
                // (Or it has just built its own way onto their island:
                // in after it, whoever's turn it is.)
                let door = bot.door.is_some_and(|(to, until)| usize::from(to) == victim && now < until);
                // Along with its teammate, unless it is one for staying
                // at home; or, the one that goes out, by its own choice.
                let along = mate_raids == Some(victim) && (runner || (now >= bot.keeper_from && bot.persona.raids > -0.05));
                if !mine && !along && !goes_out && !door {
                    continue;
                }
                // One team's raid on a team at a time, a rest after each,
                // and a team with a person on it no more than its share.
                let theirs = self.raids[victim].iter().all(|&raider| usize::from(arena_team(raider)) == team);
                let people = (0..ARENA_SEATS).any(|other| usize::from(arena_team(other)) == victim && view.seats.get(other).is_some_and(|seen| seen.person && seen.in_play));
                let share = (self.raided.1 + 1) as f32 <= skill.people_share * (self.raided.0 + 1) as f32 + 1.0;
                let fair = !people || others < 2 || share;
                let allowed = mine || along || (door && fair) || (theirs && now >= self.spared_until[victim] && fair);
                let their_heart = self.spots(nav)[victim].heart;
                if !allowed || !reach(here, their_heart) {
                    continue;
                }
                let ramp = ((now - skill.raid_from) / (skill.hunt_from - skill.raid_from)).clamp(0.0, 1.0);
                // Which of them, by chance as much as by how far: thrown
                // once a minute for each, not again at every thought, and
                // differently in every game (until 2026-11-12 the same in
                // all of them: the first raids always fell on the same
                // two islands).
                let chance = ((seat * 7 + victim * 13 + (now / 60.0) as usize * 5 + self.salt) % 11) as f32 / 11.0;
                let score = 0.6 + 0.14 * ramp - here.distance(their_heart) / 2000.0 + chance * 0.05 + bot.persona.raids + if door { 0.12 } else { 0.0 };
                offers.push((score, Goal::Raid(victim as u8)));
            }
        }

        // Out to look about, with nothing it has to do (since 2026-11-15;
        // Hajun: "or atleast just going around the map like just a
        // scout"): one that is one for going out just now, whole, with a
        // way home, and not just back from the last time. Under anything
        // there is to be done; over standing about at home.
        match bot.goal {
            // (Gone to cut a bridge, it is not put off it by as little.)
            Goal::Scout(to) if reach(here, to) => {
                let score = if bot.cutting.is_some() { 0.6 + 0.25 * bot.persona.schemes } else { 0.3 + 0.12 * bot.temper.roams };
                offers.push((score, Goal::Scout(to)));
            }
            _ if bot.out && now >= bot.scout_again && me.health >= 0.6 && can_go_home => {
                if let Some(to) = self.lookout(bot, view) {
                    offers.push((0.3 + 0.12 * bot.temper.roams, Goal::Scout(to)));
                }
            }
            _ => {}
        }

        // **A way to cut** (since 2026-11-15, a stick of dynamite taking
        // the whole bridge with it): one with a stick on it, once the
        // raids are on, now and then goes to where another team's bridge
        // leaves the middle and cuts it there, with nobody on it: the
        // schemers and those who like to see people fall a good deal
        // oftener than the rest ([`Squad::way_to_cut`], which says whose
        // it may be). Thought of every so often, not at every thought,
        // and meant for a while once thought of: it goes when there is
        // nothing it would sooner do, a schemer sooner than on a raid.
        let holds = kit.held(Utility::Dynamite) > 0;
        if !holds || bot.means_to_cut.is_some_and(|until| now >= until) {
            bot.means_to_cut = None;
        }
        if holds && bot.means_to_cut.is_none() && bot.cutting.is_none() && now >= bot.cut_again && now >= bot.raids_from {
            bot.cut_again = now + self.dice.around(CUT_AGAIN.0, CUT_AGAIN.1);
            let keen = (CUT_KEEN[0] + CUT_KEEN[1] * bot.persona.schemes + CUT_KEEN[2] * (bot.temper.shoves - 1.0)).clamp(0.03, 0.9);
            if self.dice.roll() < keen {
                bot.means_to_cut = Some(now + self.dice.around(CUT_MEANT.0, CUT_MEANT.1));
            }
        }
        // The one that keeps the base, only when it is one for going out
        // just now, as it goes for anything.
        let goes = runner || (bot.out && now >= bot.keeper_from);
        if bot.means_to_cut.is_some()
            && bot.cutting.is_none()
            && goes
            && me.health >= 0.6
            && can_go_home
            && let Some((victim, stand, at)) = self.way_to_cut(bot, view)
        {
            bot.cutting = Some((victim, at));
            offers.push((0.6 + 0.25 * bot.persona.schemes, Goal::Scout(stand)));
        }

        // **Stranded**: with no way to its island, and nothing it can do
        // about that or about anything else where it is: on planks left
        // in the middle of nowhere, the rest of their bridge having come
        // down, or on land it was cut off on with nothing to buy planks
        // with and no stand to buy them at (since 2026-11-15). Nobody is
        // coming for it and nobody can get at it. After a while of that
        // it steps off the edge, to stand again on its own island; and
        // with no anchor to bring it back, after a good deal longer,
        // there being nothing to stay for: a game is not to hang on one
        // of them out of everyone's reach until its time is up.
        let means = offers.iter().any(|(score, goal)| *score > 0.0 && !matches!(goal, Goal::Idle | Goal::Scout(_)));
        let stranded = !can_go_home && !on_home && !means && bot.laying.is_none() && bot.body.free(nav) && nav.falls_in(here).is_none();
        if !stranded {
            bot.marooned = None;
        } else if bot.marooned.is_none() {
            let patience = if view.anchors[team] > 0.0 { self.dice.around(6.0, 16.0) } else { self.dice.around(40.0, 90.0) };
            bot.marooned = Some(now + patience);
        } else if bot.marooned.is_some_and(|until| now >= until) && bot.steps_off.is_none() {
            // Off the nearest edge.
            bot.steps_off = (0..16)
                .map(|spoke| Vec2::from_angle(spoke as f32 * std::f32::consts::TAU / 16.0))
                .filter_map(|way| (1..=60).map(|step| step as f32 * 0.5).find(|far| nav.ground(here + way * *far).is_none()).map(|far| (far, way)))
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, way)| way);
        }

        // The best of them, what it is on having a bonus, and what it has
        // given up on left out for a while.
        if bot.barred.is_some_and(|(_, until)| now >= until) {
            bot.barred = None;
        }
        let best = offers
            .into_iter()
            .filter(|(_, goal)| !bot.barred.is_some_and(|(barred, _)| barred.alike(goal)))
            .map(|(score, goal)| (if goal == bot.goal { score + skill.sticky } else { score }, goal))
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(Goal::Idle, |(_, goal)| goal);
        // One bot to a span.
        self.claims.retain(|claim| claim.2 != seat);
        if let Goal::Lay(from, to) = best {
            self.claims.push((from, to, seat));
        }
        // The raids' ledger: who is on whom.
        if let Goal::Raid(victim) = best {
            let victim = usize::from(victim);
            if !self.raids[victim].contains(&seat) {
                if self.raids[victim].is_empty() {
                    let people = (0..ARENA_SEATS).any(|other| usize::from(arena_team(other)) == victim && view.seats.get(other).is_some_and(|seen| seen.person && seen.in_play));
                    self.raided = (self.raided.0 + 1, self.raided.1 + u32::from(people));
                }
                // Off whoever else it was on.
                self.call_off(seat, now);
                self.raids[victim].push(seat);
            }
        } else if matches!(bot.goal, Goal::Raid(_)) {
            // Back from one: no other for a while, shorter for one that
            // is always for going out.
            self.call_off(seat, now);
            let rest = self.dice.around(skill.raid_rest.0, skill.raid_rest.1) * (1.3 - 0.5 * bot.persona.drive);
            bot.rest_until = bot.rest_until.max(now + rest);
            bot.raiding_since = None;
        }
        // Off its pad: not back for a while, sooner for one that keeps to
        // its island; and what brings it back, and how long it will stand
        // there, are thrown again for the next time.
        if bot.goal == Goal::Collect && best != Goal::Collect && !with_person {
            bot.pad_again = now + self.dice.around(PAD_AGAIN.0, PAD_AGAIN.1) * (1.3 - 0.6 * bot.temper.roams);
            bot.pad_worth = self.dice.around(PAD_WORTH.0, PAD_WORTH.1).round() as u16;
            bot.pad_wait = self.dice.around(PAD_WAIT.0, PAD_WAIT.1);
            bot.pad_since = None;
        }
        // Setting out to look about: to a place or two more after the
        // first, the more the more it roams.
        if matches!(best, Goal::Scout(_)) && !matches!(bot.goal, Goal::Scout(_)) {
            bot.stops = (self.dice.roll() * (1.0 + 2.5 * bot.temper.roams)) as u8;
            bot.looking = None;
        }
        // (Something else come up, the bridge it meant to cut is let be.)
        if !matches!(best, Goal::Scout(_)) {
            bot.cutting = None;
        }
        // What it takes from a pad: what it is saving for at its own,
        // whatever lies on one it has gone out to, and on a raid what
        // its next purchases need of what the raided have.
        bot.wants = match best {
            // (At another island's pad, cut off from its own: what it is
            // saving for, and silver for planks at the least.)
            Goal::Fetch(_, Item::Silver) => [wants[0].max(8u16.saturating_sub(purse[0])), wants[1], 0, 0],
            Goal::Fetch(_, item) => {
                let mut all = [0; 4];
                all[item.index()] = 9;
                all
            }
            Goal::Raid(_) => [20u16.saturating_sub(purse[0]), 7u16.saturating_sub(purse[1]), 0, 0],
            // Out looking about: whatever sapphire or amethyst lies where
            // it comes by.
            Goal::Scout(_) => [0, 0, 9, 9],
            _ => wants,
        };
        // Over a fright: the next one finds it a little braver, or less.
        if bot.goal == Goal::Retreat && best != Goal::Retreat {
            bot.nerve = self.dice.around(NERVE.0, NERVE.1);
        }
        bot.set_goal(best, now);
    }

    /// Keeps the bot's mind of whoever comes at its base ([`Vigil`]), as
    /// often as it thinks. `caller` is whoever is at it now, if anyone;
    /// `at_home` whether the bot is on its island. With someone at it: it
    /// has them in mind, and counts their coming if they had been away.
    /// With them stepped back: it takes its stand toward them, if it is
    /// home or was seeing them off, for a look or, having had enough, for
    /// good; keeps between them and what its team keeps as they move; and
    /// ends it once they are gone a good way off for a while, or it has
    /// been long enough, when it either goes out at them or lets them be.
    /// Left alone long enough, it forgets them, and how patient it is
    /// with the next is thrown again.
    fn keep_vigil(&mut self, bot: &mut Bot, view: &View, caller: Option<usize>, at_home: bool, spots: &Spots) {
        let (nav, now) = (view.nav, view.elapsed);
        let home = Region::Team(arena_team(bot.seat));
        if let Some(foe) = caller {
            let mut vigil = bot.vigil.unwrap_or(Vigil {
                foe,
                comings: 0,
                at_it: false,
                about: now,
                forget: self.dice.around(VIGIL_FORGET.0, VIGIL_FORGET.1),
                stand: None,
            });
            if !vigil.at_it {
                vigil.comings = vigil.comings.saturating_add(1);
            }
            vigil.foe = foe;
            vigil.at_it = true;
            vigil.about = now;
            vigil.stand = None;
            bot.vigil = Some(vigil);
            return;
        }
        let Some(mut vigil) = bot.vigil else {
            return;
        };
        let them = view.seats.get(vigil.foe).copied().unwrap_or_default();
        let there = them.at.xz();
        let seen = them.alive && !them.unseen;
        let here = bot.body.at.xz();
        if vigil.at_it {
            // They have just stepped back from it.
            vigil.at_it = false;
            if at_home || bot.goal == Goal::Defend {
                let resolved = vigil.comings >= bot.patience;
                let lasts = if resolved { VIGIL_STAND } else { VIGIL_LOOK };
                vigil.stand = Some(Stand {
                    spot: here,
                    until: now + self.dice.around(lasts.0, lasts.1),
                    resolved,
                    watch: self.dice.around(VIGIL_WATCH.0, VIGIL_WATCH.1) * bot.temper.bold,
                    gone: self.dice.around(VIGIL_GONE.0, VIGIL_GONE.1),
                    calm: self.dice.around(VIGIL_CALM.0, VIGIL_CALM.1),
                    gone_since: None,
                    off: self.dice.around(VIGIL_OFF.0, VIGIL_OFF.1),
                    inside: self.dice.between(VIGIL_INSIDE.0, VIGIL_INSIDE.1) * bot.temper.minds.clamp(0.6, 1.5),
                });
            }
        }
        let away = there.distance(spots.heart);
        if seen && away < VIGIL_GONE.1 + 8.0 {
            vigil.about = now;
        }
        if let Some(mut stand) = vigil.stand {
            let gone = !seen || away > stand.gone;
            let gone_for = if gone { now - *stand.gone_since.get_or_insert(now) } else { 0.0 };
            if !gone {
                stand.gone_since = None;
            }
            let over = now >= stand.until || (stand.resolved && gone_for >= stand.calm);
            vigil.stand = if !over {
                // Between them and what it keeps, as they move; the two of
                // a team a little apart.
                let side = if bot.seat < ARENA_TEAMS { 0.9 } else { -0.9 };
                if let Some(spot) = guard_spot(nav, home, spots.keep, there, stand.off, stand.inside) {
                    let across = (spots.keep - there).normalize_or_zero().perp() * side;
                    let apart = spot + across;
                    let stands = nav.grid().region(apart) == Some(home) && nav.grid().height(apart).is_some() && nav.walkable_at(apart);
                    stand.spot = if stands { apart } else { spot };
                } else if stand.spot == here {
                    stand.spot = spots.guard;
                }
                Some(stand)
            } else if stand.resolved && !gone && self.dice.roll() < (0.25 + 0.3 * bot.temper.pvp + 0.35 * (bot.temper.bold - 1.0)).clamp(0.1, 0.85) {
                // It has stood there long enough with them still about:
                // out at them, for as far as they are and a while.
                stand.watch = stand.spot.distance(there) + 6.0;
                stand.until = now + self.dice.around(6.0, 14.0);
                Some(stand)
            } else {
                // Or it lets them be: back to what it was doing, with
                // them still in mind.
                bot.think_at = bot.think_at.min(now + 0.2);
                None
            };
        }
        // Left alone long enough, it has forgotten them.
        if vigil.stand.is_none() && now - vigil.about > vigil.forget {
            bot.vigil = None;
            bot.patience = patience_of(bot.persona, &mut self.dice);
            return;
        }
        bot.vigil = Some(vigil);
    }

    /// Somewhere for a bot with nothing it has to do to go and look about
    /// from ([`Goal::Scout`]): where its own team's way comes ashore in
    /// the middle, the amethyst pad toward its island, the middle of the
    /// map, where another team's way comes ashore (the likelier the
    /// bolder it is), or one of the two smaller islands beside its own.
    /// One of those it can get to and is not standing at, by chance, and
    /// a little to one side of it, so that no two stand on one spot. None
    /// with nowhere to go: its island joined to nothing yet.
    fn lookout(&mut self, bot: &Bot, view: &View) -> Option<Vec2> {
        let nav = view.nav;
        let here = bot.body.at.xz();
        let team = arena_team(bot.seat);
        let spots = self.spots(nav);
        let in_play = |other: usize| {
            (0..ARENA_SEATS).any(|seat| usize::from(arena_team(seat)) == other && view.seats.get(seat).is_some_and(|seen| seen.in_play))
        };
        let mut places: Vec<(f32, Vec2)> = vec![
            (2.0, spots[usize::from(team) % ARENA_TEAMS].rim),
            (1.4, AMETHYST_PADS[usize::from(team) % AMETHYST_PADS.len()]),
            (0.8, nav.heart(Region::Middle)),
        ];
        for other in (0..ARENA_TEAMS).filter(|&other| other != usize::from(team) && in_play(other)) {
            places.push((0.9 * bot.temper.bold, spots[other].rim));
        }
        for island in beside(team) {
            places.push((1.4, SAPPHIRE_PADS[usize::from(island) % SAPPHIRE_PADS.len()]));
        }
        places.retain(|(_, at)| here.distance(*at) > 8.0 && Self::reachable(nav, here, *at));
        if places.is_empty() {
            return None;
        }
        let odds: Vec<f32> = places.iter().map(|(odd, _)| *odd).collect();
        let (_, at) = places[pick(&mut self.dice, &odds)];
        let aside = Vec2::from_angle(self.dice.roll() * std::f32::consts::TAU) * self.dice.between(1.0, 3.0);
        nav.nearest_walkable(at + aside)
            .filter(|to| Self::reachable(nav, here, *to))
            .or(Some(at))
    }

    /// What a bot stepping onto the island of the team `victim` on a raid
    /// is there for ([`Aim`]), thrown by who it is and how things stand.
    /// Their anchor, if it stands, anchors are fair game to this one by
    /// now ([`Bot::anchors_from`]) and nobody else of the computer's is at
    /// it: the likelier the more it goes for anchors and the more set on
    /// them it is, and less, for a while, once a couple are broken
    /// ([`Skill::anchors_spared`]). Whoever is home: the likelier the
    /// better it is in a fight and the bolder, and hardly at all with
    /// nobody there. And what lies on their pad: the likelier the more of
    /// what it wants lies there.
    fn take_aim(&mut self, bot: &Bot, view: &View, victim: usize) -> Aim {
        let (nav, now) = (view.nav, view.elapsed);
        let skill = self.skill;
        let victim = victim % ARENA_TEAMS;
        let broken = view.anchors.iter().filter(|anchor| **anchor <= 0.0).count();
        let spares = if now < bot.spares_until { 1.0 / (1.0 + skill.anchors_spared * broken as f32) } else { 1.0 };
        let taken = self.strikers[victim].is_some_and(|striker| striker != bot.seat);
        let home = Region::Team(victim as u8);
        let anyone_home = view.seats.iter().enumerate().any(|(other, seen)| {
            other < ARENA_SEATS
                && usize::from(arena_team(other)) == victim
                && seen.alive
                && !seen.unseen
                && nav.grid().region(seen.at.xz()) == Some(home)
        });
        let anchor = if view.anchors[victim] > 0.0 && now >= bot.anchors_from && !taken {
            skill.anchor_keen * bot.persona.anchors * (if anyone_home { 0.45 } else { 1.0 } + 1.6 * bot.temper.zeal) * spares
        } else {
            0.0
        };
        let players = (0.3 + 0.5 * bot.temper.pvp + 0.4 * (bot.temper.bold - 1.0)).max(0.05)
            * (1.0 - 0.8 * bot.temper.zeal)
            * if anyone_home { 1.0 } else { 0.1 };
        let lying = view.mint.held(victim) + view.mint.held(ARENA_TEAMS + victim);
        let wanted = bot.wants[Item::Silver.index()] + bot.wants[Item::Gold.index()];
        let loot = if lying > 0 && wanted > 0 { 0.2 + 0.03 * lying.min(12) as f32 } else { 0.04 };
        [Aim::Anchor, Aim::Players, Aim::Loot][pick(&mut self.dice, &[anchor.max(0.0), players, loot])]
    }

    /// The bridge the bot would lay next, if any, as its team means to
    /// open ([`Plan`]): from its island to the middle, if nothing joins
    /// them yet, or first, for a team that opens so, from its island
    /// straight to a smaller island beside it, with the middle left for
    /// as long as it leaves it; then from the middle to one of the two
    /// smaller islands beside its own, if neither is joined to its
    /// island; none before its team means to build. The one that goes out
    /// lays them; beside a person the bot does, once they have had the
    /// time to. Not one a teammate is laying. The line of it between the
    /// two lands is a little to one side or the other, and straight: it
    /// is bent as it is begun ([`Squad::lay`]).
    fn next_span(&mut self, bot: &Bot, view: &View, runner: bool, with_person: bool) -> Option<(Region, Region, Span)> {
        let (nav, now) = (view.nav, view.elapsed);
        let team = arena_team(bot.seat);
        let home = Region::Team(team);
        // The one it was laying, to be finished.
        if let Some(laying) = bot.laying {
            return Some((laying.from, laying.to, laying.span));
        }
        // **Cut off**: standing on land, not its own island, that nothing
        // joins to the middle or to its island any more, the bridge it
        // came over having come down behind it (since 2026-11-15, when a
        // stick of dynamite began to take the whole bridge). A way from
        // there to the middle, then, whichever of its team it is and
        // before anything else: mended where one stood, or new.
        let here = bot.body.at.xz();
        let land = nav.grid().region(here).filter(|land| nav.grid().height(here).is_some() && !matches!(land, Region::Middle | Region::Pocket) && *land != home);
        if let Some(land) = land
            && !nav.joined(land, Region::Middle)
            && !nav.joined(land, home)
        {
            return self.span_for(bot, view, (land, Region::Middle));
        }
        let plan = self.plans[usize::from(team) % ARENA_TEAMS];
        let beside = beside(team);
        let own = Region::Sapphire(beside[usize::from(plan.side) % 2]);
        let to_middle = nav.joined(home, Region::Middle);
        let to_sapphire = beside.iter().any(|&island| nav.joined(home, Region::Sapphire(island)));
        // A hole blown in its team's way to the middle is either of the
        // team's to mend, whichever of them goes out and whoever is
        // beside it, from whichever side of it they stand on: a few
        // planks, where a bridge is the one that goes out's to lay.
        if !to_middle && let Some(mend) = self.mend_for(bot, view, (home, Region::Middle)) {
            return Some(mend);
        }
        let wanted = if with_person {
            // Beside a person: the bridge to the middle once they have
            // had the time to build it, unless they are out over the void
            // building one now; and a little later one to a smaller
            // island. (After 40 s and two minutes until 2026-11-13, when
            // Hajun asked for the bot beside them to be out and about
            // more: "bridging or raiding an island more".)
            let mate = view.seats.get(mate_of(bot.seat)).copied().unwrap_or_default();
            let theirs = mate.alive && out_over_the_void(nav, mate.at.xz());
            if !to_middle && now > 30.0 && !theirs {
                (home, Region::Middle)
            } else if to_middle && !to_sapphire && now > 75.0 {
                (Region::Middle, own)
            } else {
                return None;
            }
        } else {
            if !runner || now < plan.bridge_from {
                return None;
            }
            let direct = (plan.opening == Opening::Sapphire && !to_sapphire && !to_middle).then_some((home, own));
            let middle = (!to_middle && now >= plan.middle_from).then_some((home, Region::Middle));
            let sapphire = (to_middle && !to_sapphire).then_some((Region::Middle, own));
            // Or, once the raids are on, to the island of a team that has
            // built no way to it: nobody sits a game out behind the void.
            let stranger = (to_middle && now >= self.skill.raid_from + 90.0)
                .then(|| {
                    (0..ARENA_TEAMS as u8).find(|&other| {
                        other != team
                            && !nav.joined(Region::Team(other), Region::Middle)
                            && (0..ARENA_SEATS)
                                .any(|seat| arena_team(seat) == other && view.seats.get(seat).is_some_and(|seen| seen.in_play))
                    })
                })
                .flatten()
                .map(|other| (Region::Middle, Region::Team(other)));
            direct.or(middle).or(sapphire).or(stranger)?
        };
        self.span_for(bot, view, wanted)
    }

    /// The bridge from the one land of `wanted` to the other, for `bot` to
    /// lay: none if a teammate is laying it, or a Bridge Builder is, its
    /// own or anyone's. Beside whatever was begun there and left; and not
    /// on the very line between the two lands' middles, but a little to
    /// one side of it or the other, by the bot and by the bridge.
    fn span_for(&mut self, bot: &Bot, view: &View, wanted: (Region, Region)) -> Option<(Region, Region, Span)> {
        let nav = view.nav;
        if self.claims.iter().any(|&(from, to, by)| (from, to) == wanted && by != bot.seat) {
            return None;
        }
        let being_built = |sent: Option<Sent>| sent.is_some_and(|sent| (sent.from, sent.to) == wanted);
        if being_built(bot.sent) || self.bots.iter().flatten().any(|other| being_built(other.sent)) {
            return None;
        }
        // A bridge there with one hole blown in it is mended, not built
        // again beside itself.
        if let Some(mend) = self.mend_for(bot, view, wanted) {
            return Some(mend);
        }
        let tried = self.begun.iter().find(|begun| (begun.0, begun.1) == wanted).map_or(0, |begun| begun.2);
        let off = ((bot.seat * 31 + usize::from(bot.chains) * 17 + self.salt * 7) % 13) as f32 / 6.0 - 1.0;
        let aside = [0.0, 2.9, -2.9][usize::from(tried) % 3] + off * 1.4;
        let span = nav.grid().span(wanted.0, wanted.1, aside).or_else(|| nav.grid().span(wanted.0, wanted.1, 0.0))?;
        // From its far end, for a bot that stands on that side of the
        // water and cannot get round to this one.
        let here = bot.body.at.xz();
        if !Self::reachable(nav, here, span.from) && Self::reachable(nav, here, span.to) {
            let back = nav.grid().span(wanted.1, wanted.0, -aside).or_else(|| nav.grid().span(wanted.1, wanted.0, 0.0))?;
            return Some((wanted.0, wanted.1, back));
        }
        Some((wanted.0, wanted.1, span))
    }

    /// The planks that would mend the way between the two lands of
    /// `wanted`, for `bot` to lay: across the one hole a blast has left
    /// in a bridge between them ([`Nav::crossing`]), from a little inside
    /// what stands on the side the bot can get to, to a little inside
    /// what stands on the other ([`MEND_INSET`]), straight. None if they
    /// are joined, if no one hole lies between them (a bridge blown in
    /// two places has a stretch of itself standing in the middle of
    /// nowhere: a new bridge, then), if the bot can get to neither side
    /// of it, or if a teammate is at it already.
    fn mend_for(&mut self, bot: &Bot, view: &View, wanted: (Region, Region)) -> Option<(Region, Region, Span)> {
        let nav = view.nav;
        if nav.gaps().is_empty() || self.claims.iter().any(|&(from, to, by)| (from, to) == wanted && by != bot.seat) {
            return None;
        }
        let here = bot.body.at.xz();
        let gap = nav.crossing(nav.heart(wanted.0), nav.heart(wanted.1), f32::INFINITY)?;
        let gap = [gap, gap.turned()].into_iter().find(|gap| nav.connected(here, gap.a_in))?;
        let inside = |end: Vec2, step_back: Vec2| end + (step_back - end).normalize_or_zero() * MEND_INSET;
        Some((wanted.0, wanted.1, Span::straight(inside(gap.a, gap.a_in), inside(gap.b, gap.b_in))))
    }

    /// A bridge the bot's team could do without, for one that is given
    /// to them ([`Persona::schemes`]; since 2026-11-15, Hajun: "they rly
    /// decide to only make necessary bridges rather than some creative
    /// attack strategy bridges - some ai personality can be like that").
    /// Thought of now and then, `free` to (the one of its team that goes
    /// out, armed and whole, with nothing it has to build), the likelier
    /// the more it is one for them and the fewer it has built; and kept to
    /// until it is laid, with metres of its own or by a Bridge Builder if
    /// it has one and the bridge is no longer than a builder goes. One of:
    ///
    /// * **a side door**, from a smaller island its team has reached
    ///   straight onto the island of the other team that island lies
    ///   beside, 76 m, to come at them from where no bridge of theirs
    ///   lands (until then only ever a Bridge Builder's:
    ///   [`Squad::spare_span`]);
    /// * **a second way in**, from the middle onto the island of a team
    ///   that has its way to the middle already, well to one side of that,
    ///   to land on their flank and not where they wait for whoever comes
    ///   over their own bridge;
    /// * **straight across**, from its own island to the island of a team
    ///   at its side, 164 m, never touching the middle: the rarest, and
    ///   for the keenest only.
    ///
    /// None over water where one has been begun already.
    fn scheme(&mut self, bot: &mut Bot, view: &View, free: bool) -> Option<(Region, Region, Span)> {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let team = arena_team(bot.seat);
        let home = Region::Team(team);
        let in_play = |other: u8| {
            (0..ARENA_SEATS).any(|seat| arena_team(seat) == other && view.seats.get(seat).is_some_and(|seen| seen.in_play))
        };
        let claimed = |wanted: (Region, Region)| self.claims.iter().any(|&(from, to, by)| (from, to) == wanted && by != bot.seat);
        // The one it has settled on, for as long as there is any laying
        // it and anyone on that island to lay it for.
        if let Some((from, to, span)) = bot.scheme {
            let lives = matches!(to, Region::Team(other) if in_play(other));
            if lives && free && !claimed((from, to)) && Self::reachable(nav, here, span.from) {
                return Some((from, to, span));
            }
            bot.scheme = None;
        }
        if !free || now < bot.scheme_at || now < bot.raids_from - 20.0 || bot.laying.is_some() || bot.sent.is_some() {
            return None;
        }
        bot.scheme_at = now + self.dice.around(20.0, 50.0);
        // (Some are given to them and most are not: one that is hardly
        // given to them at all never thinks of one.)
        if self.dice.roll() >= 0.9 * (bot.persona.schemes - 0.35) / (1.0 + f32::from(bot.schemed)) {
            return None;
        }
        let begun = |a: Region, b: Region| {
            self.begun
                .iter()
                .filter(|begun| (begun.0, begun.1) == (a, b) || (begun.0, begun.1) == (b, a))
                .map(|begun| u32::from(begun.2))
                .sum::<u32>()
        };
        // What there is to think of: how likely, from where, to where, and
        // how far to one side of the line between the two.
        let mut plans: Vec<(f32, Region, Region, f32)> = Vec::new();
        for island in beside(team) {
            let from = Region::Sapphire(island);
            if !nav.joined(home, from) {
                continue;
            }
            for other in (0..ARENA_TEAMS as u8).filter(|&other| other != team && in_play(other) && beside(other).contains(&island)) {
                if begun(from, Region::Team(other)) == 0 {
                    plans.push((1.0, from, Region::Team(other), 0.0));
                }
            }
        }
        if nav.joined(home, Region::Middle) {
            for other in (0..ARENA_TEAMS as u8).filter(|&other| other != team && in_play(other)) {
                let theirs = Region::Team(other);
                // (None where one of the computer's has laid a way from
                // the middle onto it already, a second or their first.)
                let laid = self.begun.iter().any(|begun| (begun.0, begun.1) == (Region::Middle, theirs));
                if nav.joined(theirs, Region::Middle) && !laid {
                    let side = if self.dice.roll() < 0.5 { 1.0 } else { -1.0 };
                    plans.push((0.8, Region::Middle, theirs, side * self.dice.around(6.5, 8.5)));
                }
            }
        }
        for other in [(team + 1) % ARENA_TEAMS as u8, (team + 3) % ARENA_TEAMS as u8] {
            if in_play(other) && begun(home, Region::Team(other)) == 0 {
                plans.push((0.45 * bot.persona.schemes, home, Region::Team(other), 0.0));
            }
        }
        plans.retain(|&(_, from, to, _)| !claimed((from, to)));
        if plans.is_empty() {
            return None;
        }
        let odds: Vec<f32> = plans.iter().map(|plan| plan.0).collect();
        let (_, from, to, aside) = plans[pick(&mut self.dice, &odds)];
        // A second way in lands well to one side of the first, or is not
        // one; the rest a little to one side of the line or the other.
        let span = if aside == 0.0 {
            let aside = self.dice.between(-1.2, 1.2);
            nav.grid().span(from, to, aside).or_else(|| nav.grid().span(from, to, 0.0))?
        } else {
            nav.grid().span(from, to, aside).or_else(|| nav.grid().span(from, to, aside * 0.8))?
        };
        if !Self::reachable(nav, here, span.from) {
            return None;
        }
        bot.scheme = Some((from, to, span));
        bot.scheme
    }

    /// A bridge worth a Bridge Builder, to a bot with none it has to
    /// build ([`Squad::next_span`]): one its team could do without, and
    /// which it lays only by sending a builder, never with metres of its
    /// own. From the middle to the other of the two smaller islands
    /// beside its own, for twice the sapphire; or, once the raids are on,
    /// **a side door**: from a smaller island its team has reached
    /// straight onto the island of the other team that island lies
    /// beside, to come at them from where no bridge of theirs lands. One
    /// that likes a raid goes for the side door first, and the rest for
    /// the sapphire. No side door twice over the same water.
    fn spare_span(&mut self, bot: &Bot, view: &View) -> Option<(Region, Region, Span)> {
        let (nav, now) = (view.nav, view.elapsed);
        let team = arena_team(bot.seat);
        let home = Region::Team(team);
        let own = beside(team);
        let reached = |island: u8| nav.joined(home, Region::Sapphire(island));
        let in_play = |other: u8| {
            (0..ARENA_SEATS).any(|seat| arena_team(seat) == other && view.seats.get(seat).is_some_and(|seen| seen.in_play))
        };
        let second = nav
            .joined(home, Region::Middle)
            .then(|| own.into_iter().find(|&island| !reached(island)))
            .flatten()
            .map(|island| (Region::Middle, Region::Sapphire(island)));
        let side_door = (now >= self.skill.raid_from)
            .then(|| {
                own.into_iter().filter(|&island| reached(island)).find_map(|island| {
                    let from = Region::Sapphire(island);
                    (0..ARENA_TEAMS as u8)
                        .find(|&other| {
                            other != team
                                && in_play(other)
                                && beside(other).contains(&island)
                                && !self.begun.iter().any(|begun| (begun.0, begun.1) == (from, Region::Team(other)))
                        })
                        .map(|other| (from, Region::Team(other)))
                })
            })
            .flatten();
        let wanted = if bot.persona.raids > 0.0 { side_door.or(second) } else { second.or(side_door) }?;
        self.span_for(bot, view, wanted).filter(|(_, _, span)| span.length() <= BUILDER_METRES - 2.0)
    }

    /// The fast layer: which way the bot walks this frame, in metres a
    /// second across the ground, and what it does. A knock that has just
    /// landed carries it for a moment and no more ([`Skill::reel`]); then,
    /// in the air over the void, it steers back for the nearest ground,
    /// and, carried toward an edge, against what carries it, each as hard
    /// as it steers this time ([`Skill::grip`]); otherwise it goes about
    /// what it was doing ([`Squad::intend`]), fighting whoever it fights
    /// all the while, and now and then, running, it jumps.
    fn act(&mut self, bot: &mut Bot, view: &View, dt: f32, order: &mut Order) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        bot.face = None;
        bot.body.leap = false;
        // With a leap potion on it, its jumps are five times as high.
        bot.body.high = view.outfits.kit(bot.seat).leaping(now);
        bot.awaits = false;
        if now < bot.reel_until {
            return Vec2::ZERO;
        }
        // Having had enough of being marooned: off the edge, and on out
        // over the void, steering back for nothing.
        if let Some(way) = bot.steps_off {
            bot.body.leap = true;
            return way * RUN_SPEED;
        }
        let ground = nav.ground(here);
        let in_air = bot.body.lift != 0.0 || ground.is_none_or(|ground| (bot.body.at.y - ground).abs() > 0.02);
        // Jumping a gap: at where it means to come down and nothing else,
        // off the edge with a jump ([`Body::leap`]), until it is down
        // again; or, still on the ground, has come to where it was going
        // with no gap on the way; or where it meant to come down is gone,
        // to a blast; or it has come to nothing. **In the air it keeps on
        // for where it means to come down, and stops over that very
        // spot**: until 2026-11-14 the jump was over as soon as it was
        // within 0.4 m of there, in the air or not, which over the void,
        // with the place to come down on the very edge of a hole, left it
        // to steer for the nearest ground from there, and it fell
        // ([`Nav::nearest_walkable_within`]); and going at its pace all
        // the way, it would have been to one side of the spot and the
        // other of it, frame by frame, as it came down.
        if let Some(mut leap) = bot.leaping {
            leap.aloft |= in_air;
            let down = leap.aloft && !in_air;
            let there = !in_air && here.distance(leap.to) < 0.4;
            let gone = in_air && nav.ground(leap.to).is_none();
            if down || there || gone || now >= leap.until {
                bot.leaping = None;
                bot.path.clear();
                bot.path_to = None;
                bot.progress = (here, now);
            } else {
                bot.leaping = Some(leap);
                bot.body.leap = true;
                let off = leap.to - here;
                return off.normalize_or_zero() * leap.pace.min(off.length() / dt.max(1e-4));
            }
        }
        if in_air && ground.is_none() {
            return nav
                .nearest_walkable_within(here, REACH_BACK)
                .map_or(Vec2::ZERO, |land| (land - here).normalize_or_zero() * RUN_SPEED * bot.grip);
        }
        if bot.body.shove != Vec2::ZERO {
            // How far what carries it has still to carry it: further in
            // the air than along the ground.
            let carried = here + bot.body.shove * if in_air { 0.5 } else { 0.11 };
            if nav.ground(carried).is_none() {
                return -bot.body.shove.normalize_or_zero() * RUN_SPEED * bot.grip;
            }
        }
        let walk = self.intend(bot, view, dt, order);
        // A jump it has just begun is next frame's; and a stick of
        // dynamite comes before wherever it was going.
        if bot.leaping.is_some() {
            return walk;
        }
        // What someone has just dropped beside it, before it goes on.
        let walk = self.mind_spoils(bot, view, walk, order);
        let walk = self.mind_sticks(bot, view, walk);
        // And a bridge coming down from under it before either.
        let walk = self.mind_falling(bot, view, walk, order);
        if bot.leaping.is_some() || bot.flight.is_some() {
            return walk;
        }
        // Running somewhere, a jump now and then, as people run: the more
        // the bouncier it is. Not with a plank in its arms or a potion at
        // its mouth, nor in a fight, which has jumps of its own.
        let running = walk.length() > RUN_SPEED * 0.6;
        if running && !in_air && bot.duel.is_none() && !order.carrying && bot.drinking.is_none() && now >= bot.hop_at {
            bot.hop_at = now + self.dice.around(0.7, 3.2) / (0.25 + bot.temper.bouncy);
            if self.dice.roll() < 0.6 {
                bot.body.jump(nav);
            }
        }
        walk
    }

    /// What the bot does this frame toward its goal, and which way it
    /// walks for it.
    fn intend(&mut self, bot: &mut Bot, view: &View, dt: f32, order: &mut Order) -> Vec2 {
        let skill = self.skill;
        let now = view.elapsed;
        let (seat, nav) = (bot.seat, view.nav);
        let team = usize::from(arena_team(seat));
        let spots = self.spots(nav)[team];
        let here = bot.body.at.xz();
        let kit = view.outfits.kit(seat);

        // In the pond, where a knock put it: to its rim, and over it.
        if nav.grid().region(here) == Some(Region::Pocket) && !nav.walkable_at(here) {
            let out = nav.grid().nearest_clear(here, 32.0).unwrap_or(spots.heart);
            let way = (out - here).normalize_or_zero();
            let ahead = here + way * 0.6;
            if nav.ground(ahead).is_none_or(|ground| ground - bot.body.at.y > STEP) && bot.body.free(nav) {
                bot.body.hop(way);
                return Vec2::ZERO;
            }
            return way * RUN_SPEED;
        }

        // A potion at its mouth: slowly, and nothing else, until it is
        // drunk.
        let mut pace = RUN_SPEED;
        if let Some((until, potion)) = bot.drinking {
            if now >= until {
                bot.drinking = None;
                bot.drank_at = now;
                // The one it put to its mouth, of however many kinds it
                // carries.
                order.drink = Some(potion);
            } else {
                order.drinking = true;
                pace *= DRINK_SLOW;
            }
        }

        // Faster while a speed potion lasts.
        if kit.fast(now) {
            pace *= FAST;
        }
        // A potion, of whichever kinds it carries, each in its own time
        // and with nobody near enough to spill it: an invisibility potion
        // as it comes up to the island it is raiding, to walk onto it
        // unseen; a speed potion as it sets out on a raid or a hunt, or
        // for home from far off when someone is on its island; and a
        // healing potion whenever it is hurt and has the room to drink it.
        // Not in the frame one has just been drunk: it is on its button
        // still, as this frame sees its kit, and for want of this a bot
        // put the same potion to its mouth a second time, for nothing.
        if bot.drinking.is_none() && order.drink.is_none() && bot.body.free(nav) {
            let nearest = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| seen.alive && arena_team(other) != arena_team(seat))
                .map(|(_, seen)| seen.at.xz().distance(here))
                .fold(f32::INFINITY, f32::min);
            let health = view.seats.get(seat).map_or(1.0, |me| me.health);
            let speed = kit.held(Utility::Speed) > 0 && !kit.fast(now);
            let unseen = kit.held(Utility::Invisibility) > 0 && !kit.unseen(now);
            // And a leap potion (since 2026-11-15): as it comes up to the
            // island it is raiding, to go in over the heads of whoever
            // waits where its bridge comes ashore; out hunting, with
            // someone it is after not far off; and, one that jumps about
            // a good deal, out and about with nothing much to do, now and
            // then, for the fun of it.
            let leap = kit.held(Utility::Leap) > 0 && !kit.leaping(now);
            let to_their = |victim: u8| here.distance(ARENA_ISLANDS[usize::from(victim) % ARENA_ISLANDS.len()]);
            let potion = match bot.goal {
                _ if health < 0.55 && kit.held(Utility::Healing) > 0 => Some(Utility::Healing),
                Goal::Raid(victim) if unseen && to_their(victim) < 62.0 => Some(Utility::Invisibility),
                Goal::Raid(victim) if leap && to_their(victim) < bot.leap_near + crate::ARENA_ISLAND_RADIUS => Some(Utility::Leap),
                Goal::Raid(_) | Goal::Hunt if speed => Some(Utility::Speed),
                Goal::Hunt if leap && nearest < 30.0 => Some(Utility::Leap),
                Goal::Defend if speed && here.distance(spots.heart) > 30.0 => Some(Utility::Speed),
                Goal::Scout(_) | Goal::Fetch(..) if leap && self.dice.roll() < dt * 0.03 * bot.temper.bouncy => Some(Utility::Leap),
                _ => None,
            };
            if let Some(potion) = potion
                && nearest > 9.0
            {
                bot.drinking = Some((now + DRINK_SECS, potion));
                if potion == Utility::Leap {
                    bot.leap_near = self.dice.around(16.0, 40.0);
                }
            }
        }

        // A fireball, at whoever of another team is a throw away and has
        // land under them: never at someone out on a bridge.
        if kit.held(Utility::Fireball) > 0 && now >= bot.throw_at && bot.drinking.is_none() {
            let target = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
                .map(|(_, seen)| seen.at)
                .filter(|at| (4.0..FIREBALL_RANGE - 1.5).contains(&at.xz().distance(here)) && (at.y - bot.body.at.y).abs() < 1.0)
                .filter(|at| nav.grid().height(at.xz()).is_some() && !nav.on_plank(at.xz()))
                .min_by(|a, b| a.xz().distance_squared(here).total_cmp(&b.xz().distance_squared(here)));
            if let Some(target) = target {
                let off = target.xz() - here;
                order.throw = Some((-off.x).atan2(-off.y));
                bot.throw_at = now + skill.throw_every;
            }
        }

        // A stick of dynamite, at whoever of another team is out on a
        // bridge a throw away: at where they will be by the time it
        // blows, if that is on a bridge still, and otherwise at where
        // they are, to take the bridge from under them or from in front
        // of them. From the land, or from out on a bridge itself if what
        // would go is well clear of where it stands; never where one of
        // its own team stands on the planks that would go, and not with a
        // plank in its arms. Not the moment it could, but soon.
        //
        // **Since 2026-11-15 the whole bridge goes with it**, plank by
        // plank from the hole, and a stick is two amethyst: so not the
        // bridge it stands on itself, nor one any of its own team does,
        // and its own island's only with its back to the wall
        // ([`Squad::worth_blowing`]); and having thought better of one,
        // it does not think of it again for a while. And one set on an
        // anchor, on an island none of its people are home on, now and
        // then burns the bridge they would come home over
        // ([`Squad::burn_behind`]).
        let ashore = nav.grid().height(here).is_some();
        if kit.held(Utility::Dynamite) > 0
            && now >= bot.blast_at
            && bot.drinking.is_none()
            && bot.laying.is_none()
            && bot.body.free(nav)
        {
            let team = arena_team(seat);
            let mates: Vec<Vec2> = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| other != seat && seen.alive && arena_team(other) == team)
                .map(|(_, seen)| seen.at.xz())
                .collect();
            let aim = view
                .seats
                .iter()
                .enumerate()
                .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != team)
                .filter(|(_, seen)| (seen.at.y - bot.body.at.y).abs() < 1.5 && out_over_the_void(nav, seen.at.xz()))
                .filter_map(|(other, seen)| {
                    let there = seen.at.xz();
                    let away = there.distance(here);
                    if !(3.5..=DYNAMITE_FURTHEST + 2.0).contains(&away) {
                        return None;
                    }
                    // In the air for about as long as it is far, and
                    // then most of its fuse.
                    let ahead = self.foresee(nav, other, there, 0.45 + 0.04 * away + 0.6 * DYNAMITE_FUSE);
                    let at = if out_over_the_void(nav, ahead) { ahead } else { there };
                    let far = at.distance(here);
                    let nearest = if ashore { DYNAMITE_NEAREST + 0.3 } else { DYNAMITE_RADIUS + 2.0 };
                    let reached = (nearest..=DYNAMITE_FURTHEST).contains(&far);
                    let spares = mates.iter().all(|mate| !nav.on_plank(*mate) || mate.distance(at) > DYNAMITE_RADIUS + 1.5);
                    (reached && spares).then_some((away, at))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, at)| at);
            let burn = match bot.goal {
                Goal::Raid(victim) if aim.is_none() && ashore && bot.burns && bot.aim == Aim::Anchor && bot.raiding_since.is_some() => {
                    self.burn_behind(bot, view, usize::from(victim))
                }
                _ => None,
            };
            // Or, out looking about with nobody to throw it at, another
            // team's bridge that it has come to the end of, to leave them
            // no way over ([`Squad::cut_off`]): now and then, by who it
            // is.
            let cut = match bot.goal {
                Goal::Scout(_) if aim.is_none() && ashore && bot.cuts && bot.looking.is_some() => {
                    // (The one it came to cut, if it did come to.)
                    let meant = bot.cutting.map(|(_, at)| at).filter(|at| (DYNAMITE_NEAREST + 0.3..=DYNAMITE_FURTHEST).contains(&at.distance(here)));
                    meant.or_else(|| self.cut_off(bot, view))
                }
                _ => None,
            };
            let unmanned = aim.is_none();
            if let Some(at) = aim.or(burn).or(cut)
                && self.dice.roll() < dt * 4.0
            {
                if self.worth_blowing(bot, view, at, unmanned) {
                    let off = at - here;
                    order.blast = Some(((-off.x).atan2(-off.y), off.length()));
                    bot.blast_at = now + self.dice.around(6.0, 12.0);
                    bot.means_to_cut = None;
                } else {
                    bot.blast_at = now + self.dice.around(2.5, 6.0);
                }
                bot.burns = false;
                bot.cuts = false;
                bot.cutting = None;
            }
        }

        // On a raid, stepping onto their island: what it is there for
        // this time, and how long it stays ([`Aim`]). Before whom it
        // fights, which goes by it.
        if let Goal::Raid(victim) = bot.goal {
            let on_island = nav.grid().region(here) == Some(Region::Team(victim)) && !nav.on_plank(here);
            if on_island && bot.raiding_since.is_none() {
                bot.aim = self.take_aim(bot, view, usize::from(victim));
                bot.raid_lasts = self.dice.around(skill.raid_secs.0, skill.raid_secs.1) * bot.persona.lingers;
                bot.aim_again = now + self.dice.around(4.0, 10.0);
                bot.raiding_since = Some(now);
                bot.burns = self.dice.roll() < 0.6 * bot.temper.zeal;
            }
        }

        // Whoever it fights, if anyone.
        if bot.shun.is_some_and(|(_, until)| now >= until) {
            bot.shun = None;
        }
        bot.foe = self.foe(bot, view);
        if let Some(foe) = bot.foe.filter(|_| bot.drinking.is_none()) {
            return self.fight(bot, view, foe, pace, dt, order);
        }
        bot.in_reach_since = None;
        bot.duel = None;

        match bot.goal {
            Goal::Idle => {
                // Nothing to do: about its island, from one thing on it
                // to another and not straight to any of them, as people
                // wander when there is nothing to do; the one that keeps
                // the base mostly by its anchor. (Until 2026-11-12 it
                // stood on one spot.)
                //
                // **Mostly at one of its two stands** (since 2026-11-15;
                // Hajun: "atleast they should be standing in the shop so
                // they look like theyre doing sth"): in the outline, as
                // someone with the shop's window open stands, turned to
                // the stand; or at the shore its bridges leave from,
                // looking out; the one that keeps the base by its anchor
                // now and then; and nobody on the pad, which makes what
                // it makes with nobody on it. (Until then the keeper was
                // by its anchor half the time and each of them on the pad
                // a fifth of it.)
                if bot.loiter.is_none_or(|(_, until)| now >= until) {
                    let places = [spots.guard, spots.heart, spots.lookout, spots.items, spots.upgrades];
                    let mate = view.seats.get(mate_of(seat)).copied().unwrap_or_default();
                    let odds = if mate.person && mate.in_play {
                        [2.0, 1.0, 1.5, 2.5, 2.5]
                    } else if bot.role == Role::Keeper {
                        [2.0, 0.5, 1.5, 3.0, 3.0]
                    } else {
                        [0.5, 0.5, 2.0, 3.0, 3.0]
                    };
                    let which = pick(&mut self.dice, &odds);
                    let at_a_stand = which >= 3;
                    let (far, lasts) = if at_a_stand { (0.45, BROWSE) } else { (1.6, (2.5, 8.0)) };
                    let about = Vec2::from_angle(self.dice.roll() * std::f32::consts::TAU) * self.dice.between(0.0, far);
                    let to = places[which] + about;
                    let to = nav.nearest_walkable(to).filter(|to| nav.connected(here, *to)).unwrap_or(spots.heart);
                    bot.loiter = Some((to, now + self.dice.around(lasts.0, lasts.1)));
                }
                let to = bot.loiter.map_or(spots.heart, |(to, _)| to);
                // Come to where it was going: turned to the stand it is
                // at, out from the middle of its island, or, at the
                // shore, to the middle of the map.
                if here.distance(to) <= ARRIVED + 0.2 {
                    let island = ARENA_ISLANDS[team % ARENA_ISLANDS.len()];
                    let stand = [spots.items, spots.upgrades].into_iter().find(|stand| stand.distance(to) < 0.8);
                    let toward = match stand {
                        Some(stand) => stand - island,
                        None if to.distance(spots.lookout) < 2.0 => -island,
                        None => Vec2::ZERO,
                    };
                    if toward.length() > 0.5 {
                        bot.face = Some((-toward.x).atan2(-toward.y));
                    }
                }
                // At a stroll about its island; at a run from further off,
                // as one comes home from having been out.
                let stroll = if here.distance(to) > 14.0 { 1.0 } else { 0.6 };
                self.walk_to(bot, view, to, pace * stroll, 2.0)
            }
            Goal::Collect => {
                // Beside a person, a visit and no more: once it has been
                // at the pad a moment, or there is nothing left lying
                // there, it is off, and not back for a while.
                let mate = view.seats.get(mate_of(seat)).copied().unwrap_or_default();
                if mate.person && mate.in_play {
                    let lying = view.mint.held(team) + view.mint.held(ARENA_TEAMS + team);
                    let since = (here.distance(spots.pad) < 1.3).then(|| *bot.pad_since.get_or_insert(now));
                    if since.is_some_and(|since| now - since > 1.4 || lying == 0) || bot.yields {
                        bot.pad_since = None;
                        bot.pad_again = now + self.dice.around(9.0, 16.0);
                        bot.think_at = now;
                        return Vec2::ZERO;
                    }
                }
                self.walk_to(bot, view, spots.pad, pace, 2.0)
            }
            Goal::Shop(ware) => {
                let Some(shop) = self.stand_for(bot, nav, ware) else {
                    bot.think_at = bot.think_at.min(now + 0.3);
                    return Vec2::ZERO;
                };
                if here.distance(shop) <= 1.2 {
                    // At the stand: a moment over it, as anyone takes,
                    // and a moment of its own for each thing bought.
                    let due = match bot.browsing {
                        Some((wanted, due)) if wanted == ware => due,
                        _ => {
                            let due = now + self.dice.around(SHOP_SECS.0, SHOP_SECS.1);
                            bot.browsing = Some((ware, due));
                            due
                        }
                    };
                    if now < due {
                        return Vec2::ZERO;
                    }
                    bot.browsing = None;
                    order.buy = Some(ware);
                    bot.tries += 1;
                    // Asked for and not had: something else, for a while.
                    if bot.tries > 2 {
                        bot.barred = Some((bot.goal, now + 6.0));
                    }
                    bot.think_at = now;
                    return Vec2::ZERO;
                }
                // Not there, or gone from it: what it was making up its
                // mind over is forgotten.
                bot.browsing = None;
                self.walk_to(bot, view, shop, pace, 2.0)
            }
            Goal::Fetch(pad, _) => {
                // Given long enough there, it thinks of something else.
                if now - bot.goal_since > 45.0 {
                    bot.barred = Some((bot.goal, now + 15.0));
                    bot.think_at = now;
                }
                self.walk_to(bot, view, pad, pace, 2.0)
            }
            Goal::Defend => self.walk_to(bot, view, spots.guard, pace, 1.0),
            Goal::Guard => {
                // Where it stands, between them and what its team keeps,
                // turned to them.
                let Some((spot, foe)) = bot.vigil.and_then(|vigil| vigil.stand.map(|stand| (stand.spot, vigil.foe))) else {
                    bot.think_at = bot.think_at.min(now);
                    return Vec2::ZERO;
                };
                if here.distance(spot) <= 1.5
                    && let Some(them) = view.seats.get(foe)
                {
                    let off = them.at.xz() - here;
                    if off.length() > 0.5 {
                        bot.face = Some((-off.x).atan2(-off.y));
                    }
                }
                self.walk_to(bot, view, spot, pace, 1.0)
            }
            Goal::Retreat => {
                // Home if there is a way, to its pad, the far side of its
                // island from anyone coming; and a healing potion there,
                // or anywhere nobody is near.
                let nearest = view
                    .seats
                    .iter()
                    .enumerate()
                    .filter(|&(other, seen)| seen.alive && arena_team(other) != arena_team(seat))
                    .map(|(_, seen)| seen.at.xz().distance(here))
                    .fold(f32::INFINITY, f32::min);
                if bot.drinking.is_none() && order.drink.is_none() && bot.body.free(nav) {
                    if kit.held(Utility::Healing) > 0 && nearest > 6.0 {
                        bot.drinking = Some((now + DRINK_SECS, Utility::Healing));
                    } else if kit.held(Utility::Speed) > 0 && !kit.fast(now) && (8.0..22.0).contains(&nearest) {
                        // Or a speed potion, to get away on, with whoever
                        // is after it far enough off for it to be drunk.
                        bot.drinking = Some((now + DRINK_SECS, Utility::Speed));
                    } else if kit.held(Utility::Leap) > 0 && !kit.leaping(now) && (6.0..18.0).contains(&nearest) {
                        // Or a leap potion, to get away over their heads.
                        bot.drinking = Some((now + DRINK_SECS, Utility::Leap));
                    }
                }
                // With a leap potion on it and someone all but on it, up
                // and away: soon, not on the instant.
                if bot.body.high && nearest < 5.5 && bot.body.free(nav) && bot.drinking.is_none() && self.dice.roll() < dt * 5.0 {
                    bot.body.jump(nav);
                }
                if nav.connected(here, spots.pad) {
                    self.walk_to(bot, view, spots.pad, pace, 2.0)
                } else {
                    Vec2::ZERO
                }
            }
            Goal::Lay(from, to) => self.lay(bot, view, from, to, pace, order, dt),
            Goal::Hunt => {
                // After whoever of another team is nearest that it can
                // get at, over a wider hole with a leap potion on it or on
                // its buttons; with nobody, it thinks of something else.
                let jump = Self::reach_of_a_jump(RUN_SPEED, bot.body.high || kit.held(Utility::Leap) > 0);
                let prey = view
                    .seats
                    .iter()
                    .enumerate()
                    .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
                    .map(|(_, seen)| seen.at.xz())
                    .filter(|&at| nav.connected(here, at) || nav.crossing(here, at, jump).is_some())
                    .min_by(|a, b| a.distance_squared(here).total_cmp(&b.distance_squared(here)));
                match prey {
                    Some(prey) => self.walk_to(bot, view, prey, pace, 1.0),
                    None => {
                        bot.think_at = bot.think_at.min(now + 0.5);
                        Vec2::ZERO
                    }
                }
            }
            Goal::Raid(victim) => {
                let victim = usize::from(victim);
                let theirs = self.spots(nav)[victim];
                let on_island = nav.grid().region(here) == Some(Region::Team(victim as u8)) && !nav.on_plank(here);
                if !on_island {
                    bot.raiding_since = None;
                    let walk = self.walk_to(bot, view, theirs.heart, pace, 2.0);
                    self.vault(bot, view, walk, dt);
                    return walk;
                }
                // There, for what it came for ([`Aim`], thrown as it
                // stepped ashore): no longer than a raid lasts, which is
                // not the same twice and for some is longer than for
                // others.
                let since = *bot.raiding_since.get_or_insert(now);
                // (One at an anchor that still stands stays the longer the
                // more set on it it is.)
                let at_it = bot.aim == Aim::Anchor && view.anchors[victim] > 0.0;
                if now - since > bot.raid_lasts * if at_it { 1.0 + bot.temper.zeal } else { 1.0 } {
                    let rest = self.dice.around(skill.raid_rest.0, skill.raid_rest.1);
                    bot.barred = Some((bot.goal, now + rest));
                    bot.think_at = now;
                    return Vec2::ZERO;
                }
                // What it came for is done, or there is nothing in it:
                // their anchor gone or another at it, nobody home, their
                // pad bare. Something else, then, thrown again: at once
                // for an anchor gone, and otherwise in a while.
                let home = Region::Team(victim as u8);
                let anyone_home = view.seats.iter().enumerate().any(|(other, seen)| {
                    other < ARENA_SEATS
                        && usize::from(arena_team(other)) == victim
                        && seen.alive
                        && !seen.unseen
                        && nav.grid().region(seen.at.xz()) == Some(home)
                });
                let taken = self.strikers[victim].is_some_and(|striker| striker != seat);
                let lying = view.mint.held(victim) + view.mint.held(ARENA_TEAMS + victim);
                let spent = match bot.aim {
                    Aim::Anchor => view.anchors[victim] <= 0.0 || taken,
                    Aim::Players => !anyone_home && now >= bot.aim_again,
                    Aim::Loot => (lying == 0 || bot.wants == [0; 4]) && here.distance(theirs.pad) < 1.6 && now >= bot.aim_again,
                };
                if spent {
                    bot.aim = self.take_aim(bot, view, victim);
                    bot.aim_again = now + self.dice.around(4.0, 10.0);
                }
                // Their anchor: one bot at it at a time, each punch in its
                // own time, quicker for one that is there for nothing
                // else.
                if bot.aim == Aim::Anchor && view.anchors[victim] > 0.0 && !taken {
                    self.strikers[victim] = Some(seat);
                    let anchor = ARENA_ANCHORS[victim];
                    if here.distance(anchor) <= REACH + ANCHOR_RADIUS - 0.2 {
                        let off = anchor - here;
                        bot.face = Some((-off.x).atan2(-off.y));
                        if now >= bot.strike_at {
                            let (least, most) = skill.anchor_every;
                            bot.strike_at = now + self.dice.around(least, most) * (1.0 - 0.15 * bot.temper.zeal);
                            order.strike = Some(victim as u8);
                        }
                        return Vec2::ZERO;
                    }
                    let walk = self.walk_to(bot, view, theirs.strike, pace, 2.0);
                    self.vault(bot, view, walk, dt);
                    return walk;
                }
                if self.strikers[victim] == Some(seat) {
                    self.strikers[victim] = None;
                }
                // Whoever is home is fought wherever they are
                // ([`Squad::foe`]); with nobody to fight just now, it
                // waits where they stand again, in the middle of their
                // island. And for what lies on their pad, onto the pad,
                // as all of them went until 2026-11-15.
                let to = if bot.aim == Aim::Players { theirs.heart } else { theirs.pad };
                self.walk_to(bot, view, to, pace, 2.0)
            }
            Goal::Scout(to) => {
                // Out to look about. Come to the place, it looks about a
                // while, turned to whoever of another team is in sight or
                // to the middle of the map, a little this way and that;
                // then on to another place, if it means to go on, or it
                // has seen enough, and stays in a while.
                let over = |squad: &mut Self, bot: &mut Bot| {
                    bot.looking = None;
                    bot.stops = 0;
                    bot.scout_again = now + squad.dice.around(SCOUT_AGAIN.0, SCOUT_AGAIN.1) * (1.5 - bot.temper.roams);
                    bot.set_goal(Goal::Idle, now);
                    bot.think_at = now;
                };
                if here.distance(to) <= 1.5 {
                    let until = match bot.looking {
                        Some(until) => until,
                        None => {
                            let until = now + self.dice.around(LOOK_ABOUT.0, LOOK_ABOUT.1);
                            bot.looking = Some(until);
                            // (And whether, with a stick of dynamite on
                            // it, it is one for cutting a bridge it finds
                            // itself at the end of: the schemers and
                            // those who like to see people fall.)
                            // And one that came here to cut one does.
                            let odds = 0.1 + 0.35 * bot.persona.schemes + 0.15 * (bot.temper.shoves - 1.0);
                            bot.cuts = bot.cutting.is_some() || self.dice.roll() < odds;
                            until
                        }
                    };
                    if now < until {
                        let seen = view
                            .seats
                            .iter()
                            .enumerate()
                            .filter(|&(other, seen)| seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
                            .map(|(_, seen)| seen.at.xz())
                            .filter(|at| at.distance(here) < 70.0)
                            .min_by(|a, b| a.distance_squared(here).total_cmp(&b.distance_squared(here)));
                        let toward = seen.map_or(-here, |at| at - here);
                        if toward.length() > 1.0 {
                            let swing = 0.7 * (now * 0.9 + seat as f32).sin();
                            bot.face = Some((-toward.x).atan2(-toward.y) + swing);
                        }
                        return Vec2::ZERO;
                    }
                    bot.looking = None;
                    if bot.stops > 0
                        && let Some(next) = self.lookout(bot, view)
                    {
                        bot.stops -= 1;
                        bot.set_goal(Goal::Scout(next), now);
                        return Vec2::ZERO;
                    }
                    over(self, bot);
                    return Vec2::ZERO;
                }
                // Given long enough on the way, it thinks of something
                // else.
                if now - bot.goal_since > 60.0 {
                    over(self, bot);
                    return Vec2::ZERO;
                }
                self.walk_to(bot, view, to, pace, 2.0)
            }
        }
    }

    /// A frame of the bot's fight with whoever is in `foe`: its punch,
    /// when they are in its reach and it is due, now and then with a jump;
    /// and where it goes, by how it is fighting them for now ([`Stance`]),
    /// which it thinks of again every few seconds and whenever either of
    /// them steps onto a bridge or off one. It faces them whichever way
    /// it walks.
    #[allow(clippy::too_many_arguments)]
    fn fight(&mut self, bot: &mut Bot, view: &View, foe: usize, pace: f32, dt: f32, order: &mut Order) -> Vec2 {
        let skill = self.skill;
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let there = view.seats[foe].at.xz();
        let off = there - here;
        let away = off.length();
        bot.face = Some((-off.x).atan2(-off.y));

        // How good it is in a fight ([`Temper::pvp`]): the better, the
        // sooner its first punch comes once it has someone in reach, and
        // the nearer its punches come to as fast as they count (a body is
        // past hurting for half a second after a blow that took).
        let pvp = bot.temper.pvp;
        let every = skill.punch_every - (skill.punch_every - 0.53).max(0.0) * pvp;
        let windup = skill.windup * (1.0 - 0.65 * pvp);
        // Coming in again from a step back timed to its next punch, that
        // punch is thrown as they come into its reach: it knew they would.
        let lunging = bot.duel.is_some_and(|duel| duel.foe == foe && duel.lunge);
        let mut struck = false;
        if away <= REACH + BODY - 0.1 {
            let since = *bot.in_reach_since.get_or_insert(now);
            if (lunging || now - since >= bot.windup) && now >= bot.punch_at {
                // Not to the beat: a little sooner, a little later; the
                // best of them nearly to it.
                bot.punch_at = now + self.dice.around(every * (0.75 + 0.2 * pvp), every * (1.3 - 0.24 * pvp));
                bot.windup = self.dice.around(windup * 0.5, windup * 1.6);
                order.punch = Some(foe);
                // On the run only if it really was running in.
                order.running = bot.speed > RUN_SPEED * 0.6;
                struck = true;
                // And now and then with a jump, as people throw theirs.
                if self.dice.roll() < 0.1 + 0.3 * bot.temper.bouncy {
                    bot.body.jump(nav);
                }
            }
        } else if away > REACH + BODY + 0.5 {
            bot.in_reach_since = None;
        }
        // A jump of no use at all, now and then, close in.
        if away < 6.0 && now >= bot.hop_at {
            bot.hop_at = now + self.dice.around(0.8, 3.4) / (0.3 + bot.temper.bouncy);
            if self.dice.roll() < 0.5 {
                bot.body.jump(nav);
            }
        }

        // How it goes about this one, for the next while.
        let (me_out, foe_out) = (out_over_the_void(nav, here), out_over_the_void(nav, there));
        let stale = bot
            .duel
            .is_none_or(|duel| duel.foe != foe || now >= duel.until || duel.me_out != me_out || duel.foe_out != foe_out);
        if stale {
            // Having stood off someone on the land from out on a bridge
            // long enough, now and then it gives them up for a while.
            let stood_off = bot
                .duel
                .is_some_and(|duel| duel.foe == foe && duel.stance == Stance::Wait && duel.me_out && !duel.foe_out && now >= duel.until);
            if stood_off && me_out && !foe_out && self.dice.roll() < 0.3 {
                bot.shun = Some((foe, now + self.dice.around(3.0, 8.0)));
                bot.duel = None;
                bot.think_at = now;
                return Vec2::ZERO;
            }
            bot.duel = Some(self.take_stance(bot, view, foe, me_out, foe_out));
        }
        let Some(mut duel) = bot.duel else {
            return Vec2::ZERO;
        };
        if struck {
            // In and out: back out of their reach until its next punch is
            // nearly due, to come in again on the run, where a punch
            // knocks further. Poking is that; and the better a bot is in a
            // fight, the oftener it does so however it is fighting, and
            // the closer to the moment it cuts it (Hajun, 2026-11-13:
            // "they seem to get knocked back more than they knock me
            // back": a bot stood where it was between its punches, and a
            // punch thrown standing knocks half as far as one thrown
            // running in).
            // (Back in from where it steps back to, to where they come
            // into its reach, as its next punch falls due.)
            let timed = (bot.punch_at - (POKE_BACK - (REACH + BODY - 0.1)) / pace.max(1.0)).max(now + 0.12);
            duel.lunge = false;
            match duel.stance {
                Stance::Poke if self.dice.roll() >= pvp => duel.back_until = now + self.dice.around(0.45, 1.0),
                // It has had its punch from where it lay in wait: at them.
                Stance::Ambush => {
                    duel.stance = Stance::Press;
                    duel.spot = None;
                    duel.until = now + self.dice.around(1.5, 3.5);
                }
                Stance::Poke => (duel.back_until, duel.lunge) = (timed, true),
                Stance::Press | Stance::Circle if self.dice.roll() < 0.9 * pvp => (duel.back_until, duel.lunge) = (timed, true),
                _ => {}
            }
            // Wherever it stood to punch, it is somewhere else next.
            duel.spot_until = now;
        }
        // Where it goes for them: at where they will be, not where they
        // are ([`Squad::meet`]).
        let aim = self.meet(nav, bot, foe, there, pace);
        let brisk = self.speed_of(foe) > 3.0;
        // Someone on their way over a gap to it, in the air over the void
        // or running at it with no straight way between them, is waited
        // for: it jumps no gap itself until they are down or have stopped
        // (seen in a game of bots on 2026-11-14: two of them after each
        // other across a hole a blast had left in a bridge jumped it at
        // the same moment, passing each other in the air, three times
        // running, and the third time one of them fell).
        let coming = !nav.line(here, there) && {
            let aloft = nav.ground(there).is_none_or(|ground| view.seats[foe].at.y - ground > 0.3);
            let closing = self
                .tracks
                .get(foe)
                .is_some_and(|track| track.known && track.velocity.dot(-off / away.max(1e-3)) > 3.0);
            aloft || closing
        };
        bot.awaits = coming;
        // A gap between it and them that it could jump, with no walking
        // straight to them and the way round far, or none: now and then
        // it jumps it, the likelier the bolder and the better it is, as
        // someone after you would (Hajun, 2026-11-13: "i jump across to
        // the other bridge, and instead of the ai also jumping across, it
        // goes all the way back to the island and then go on the other
        // bridge - so i can jump back and forth between the 2 bridge").
        // It stands where it is for the moment it takes to make up its
        // mind, rather than set off round and be out of reach of the jump
        // by then; and having thought better of it, it goes round, and
        // thinks of it again in a while.
        if bot.leaping.is_none()
            && !coming
            && pace >= RUN_SPEED * 0.9
            && now >= duel.round_until
            && bot.body.free(nav)
            && !nav.line(here, there)
            && let Some((_, down)) = nav.leap(here, there, Self::reach_of_a_jump(pace, bot.body.high))
        {
            let far = match duel.far {
                Some((far, stood)) if stood.distance(there) < 2.0 => far,
                _ => {
                    let round = nav.path(here, there).map(|path| {
                        let mut from = here;
                        path.iter().fold(0.0, |walked, &point| {
                            let step = from.distance(point);
                            from = point;
                            walked + step
                        })
                    });
                    let far = round.is_none_or(|round| round > 2.0 * away + 5.0);
                    duel.far = Some((far, there));
                    far
                }
            };
            if far && now < duel.leap_at {
                bot.duel = Some(duel);
                return Vec2::ZERO;
            }
            if far {
                let keen = (0.3 + 0.3 * bot.temper.bold + 0.35 * pvp).clamp(0.3, 0.95);
                if self.dice.roll() < keen {
                    bot.leaping = Some(Leap {
                        to: down,
                        pace,
                        until: now + if bot.body.high { 3.5 } else { 2.0 },
                        aloft: false,
                    });
                    bot.duel = Some(duel);
                    return (down - here).normalize_or_zero() * pace;
                }
                duel.round_until = now + self.dice.around(1.5, 3.0);
                duel.leap_at = duel.round_until + self.dice.between(0.1, 0.4);
                duel.far = None;
            }
        }
        // Straight to somewhere near, no further than it is in a frame;
        // round whatever stands between otherwise.
        let toward = |squad: &mut Self, bot: &mut Bot, to: Vec2, pace: f32| {
            let stretch = to - here;
            if stretch.length() <= 0.15 {
                Vec2::ZERO
            } else if nav.line(here, to) {
                bot.path.clear();
                bot.path_to = None;
                stretch.normalize_or_zero() * pace.min(stretch.length() / dt.max(1e-3))
            } else {
                squad.walk_to(bot, view, to, pace, 0.5)
            }
        };
        let walk = match duel.stance {
            Stance::Wait => {
                if away < HOLD_OFF - 1.4 {
                    // They have come: at them, then.
                    duel.stance = if self.dice.roll() < 0.5 { Stance::Press } else { Stance::Poke };
                    duel.spot = None;
                    duel.until = now + self.dice.around(1.5, 3.5);
                    Vec2::ZERO
                } else if away > HOLD_OFF + 1.2 {
                    toward(self, bot, aim, pace)
                } else {
                    Vec2::ZERO
                }
            }
            Stance::Ambush => match duel.spot {
                Some(lair) => toward(self, bot, lair, pace),
                None => Vec2::ZERO,
            },
            Stance::Press | Stance::Circle | Stance::Poke | Stance::Rush => {
                let backing = duel.stance != Stance::Rush && now < duel.back_until;
                if duel.spot.is_none() || now >= duel.spot_until {
                    duel.spot = if backing {
                        // Straight back from them, as far as there is
                        // ground to stand on that way.
                        let back = here - off.normalize_or_zero() * (POKE_BACK - away).max(0.0);
                        (nav.walkable_at(back) && flung(nav, there, back) == 0.0).then_some(back)
                    } else {
                        self.spot(bot, view, aim, &duel)
                    };
                    // Sooner again for someone on the move, who is not
                    // where they were by then; and, backing off, the
                    // moment that is over.
                    let again = now + if brisk { self.dice.between(0.1, 0.22) } else { self.dice.between(0.3, 0.7) };
                    duel.spot_until = if backing { again.min(duel.back_until) } else { again };
                }
                // Circling, a little slower; but not after someone who is
                // running, which would never catch them.
                let pace = if duel.stance == Stance::Circle && !brisk { pace * 0.85 } else { pace };
                match duel.spot {
                    Some(spot) => toward(self, bot, spot, pace),
                    // Nowhere to stand round them: up to them as it used
                    // to go, or, backing, where it is.
                    None if backing || away <= REACH * PRESS_IN => Vec2::ZERO,
                    None => toward(self, bot, aim, pace),
                }
            }
        };
        // Off the bridge and past them: on the land, it fights on as it
        // would there.
        if duel.stance == Stance::Rush && !me_out {
            duel.until = now;
        }
        // And with a leap potion on it, over their heads: up as it comes
        // at them, to come down on the land behind them.
        if duel.stance == Stance::Rush && me_out && bot.body.high && away < 5.0 && bot.body.free(nav) && self.dice.roll() < dt * 5.0 {
            bot.body.jump(nav);
        }
        bot.duel = Some(duel);
        walk
    }

    /// How the bot goes about whoever is in `foe` for the next while, by
    /// chance: weighted by where the two of them stand, on the land or
    /// out on a bridge, by how bold it is, and by who it is
    /// ([`Persona::fights`]). On the land it presses,
    /// circles or pokes; with them coming over a bridge it may lie in
    /// wait beside where the bridge comes ashore, or wait, or go to meet
    /// them; out on a bridge with them on the land ahead it may stop short
    /// of their reach, or run for the land past them, rather than stand
    /// on the last plank to be thrown off it; and with both of them out
    /// on a bridge it presses, pokes or waits. Having waited once, it does
    /// not wait again.
    fn take_stance(&mut self, bot: &Bot, view: &View, foe: usize, me_out: bool, foe_out: bool) -> Duel {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let there = view.seats[foe].at.xz();
        let bold = bot.temper.bold;
        let shy = 2.0 - bold;
        let side = if self.dice.roll() < 0.5 { 1.0 } else { -1.0 };
        // Press, circle, poke, wait, ambush, rush.
        let mut odds = match (me_out, foe_out) {
            (false, false) => [3.0 * bold, 3.0, 2.5 * bold, 0.4 * shy, 0.0, 0.0],
            (false, true) => [2.0 * bold, 0.0, 1.5 * bold, 1.5 * shy, 4.5 * shy, 0.0],
            (true, false) => [1.0 * bold, 0.0, 1.5, 3.5 * shy, 0.0, 4.0 * bold],
            (true, true) => [3.0 * bold, 0.0, 3.0, 2.0 * shy, 0.0, 0.0],
        };
        for (odd, leaning) in odds.iter_mut().zip(bot.persona.fights) {
            *odd *= leaning;
        }
        // With a leap potion on it, past them is over them: the likelier.
        if bot.body.high {
            odds[5] *= 2.5;
        }
        if bot.duel.is_some_and(|was| was.foe == foe && matches!(was.stance, Stance::Wait | Stance::Ambush)) {
            odds[3] = 0.0;
            odds[4] = 0.0;
        }
        let lair = (odds[4] > 0.0).then(|| lair(nav, here, there, side)).flatten();
        if lair.is_none() {
            odds[4] = 0.0;
        }
        let stances = [Stance::Press, Stance::Circle, Stance::Poke, Stance::Wait, Stance::Ambush, Stance::Rush];
        let stance = stances[pick(&mut self.dice, &odds)];
        let lasts = match stance {
            Stance::Press | Stance::Circle | Stance::Poke => self.dice.around(1.6, 4.2),
            Stance::Wait => self.dice.around(1.4, 3.6),
            Stance::Ambush => 5.0,
            Stance::Rush => 3.0,
        };
        Duel {
            foe,
            stance,
            until: now + lasts,
            side,
            spot: if stance == Stance::Ambush { lair } else { None },
            spot_until: 0.0,
            back_until: 0.0,
            me_out,
            foe_out,
            // A moment to take in that there is a gap between them.
            leap_at: now + self.dice.between(0.15, 0.5),
            round_until: 0.0,
            far: None,
            lunge: false,
        }
    }

    /// Where the bot stands to fight someone `there`: the best of twelve
    /// places round them a punch's length out, each a place a body can
    /// walk that it can get to. Better for being near; much better, by
    /// how much it goes for that, if a punch from there would carry them
    /// off the land ([`flung`]), and worse, by how much it minds, if one
    /// of theirs would carry it off; circling, better for being a little further round them the
    /// way it is going; and all of it a little by chance, so that it is
    /// not the same place twice. A bot running for the land past them
    /// takes only a place on the land. `None` with nowhere to stand: out
    /// on a bridge's middle line there are two places at the most.
    fn spot(&mut self, bot: &Bot, view: &View, there: Vec2, duel: &Duel) -> Option<Vec2> {
        let nav = view.nav;
        let here = bot.body.at.xz();
        let (shoves, minds) = (bot.temper.shoves, bot.temper.minds);
        let mine = (here - there).to_angle();
        let turn = self.dice.roll() * std::f32::consts::TAU / 12.0;
        let mut best: Option<(f32, Vec2)> = None;
        for place in 0..12 {
            let angle = turn + place as f32 * std::f32::consts::TAU / 12.0;
            let at = there + Vec2::from_angle(angle) * RING;
            if !nav.walkable_at(at) || !nav.connected(here, at) {
                continue;
            }
            if duel.stance == Stance::Rush && out_over_the_void(nav, at) {
                continue;
            }
            let mut score = -0.09 * here.distance(at) + shoves * flung(nav, at, there) - 1.1 * minds * flung(nav, there, at);
            if duel.stance == Stance::Circle {
                // How far round them from where it is, the way it goes:
                // best about a sixth of a turn on.
                let round = (angle - mine + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                score += 0.5 - (round * duel.side - 1.0).abs() * 0.35;
            }
            if !nav.line(here, at) {
                score -= 0.5;
            }
            score += 0.18 * self.dice.bell();
            if best.is_none_or(|(most, _)| score > most) {
                best = Some((score, at));
            }
        }
        best.map(|(_, at)| at)
    }

    /// Whom the bot fights this frame, if anyone: the nearest of another
    /// team it can see and get at, within how far it goes for people,
    /// which is further the bolder it is; but for whoever it has given up
    /// on for now, unless they have just hit it.
    fn foe(&self, bot: &Bot, view: &View) -> Option<usize> {
        let (seat, nav, now) = (bot.seat, view.nav, view.elapsed);
        let here = bot.body.at.xz();
        let me = view.seats[seat];
        let kit = view.outfits.kit(seat);
        let sight = if kit.blind { BLIND_SIGHT } else { HUNT_RADIUS };
        let stung = now - bot.hurt_at < bot.sting;
        let reach = match bot.goal {
            // Backing off, it fights only whoever is on top of it.
            Goal::Retreat => 2.6,
            Goal::Defend | Goal::Guard | Goal::Hunt => sight,
            // On the island it is raiding: whoever is home, if it is
            // there for them; whoever comes near, if it is there for what
            // lies on their pad; and there for their anchor, only whoever
            // is on top of it, the less the more set on the anchor it is,
            // a little further off once they have hit it ([`Aim`],
            // [`Temper::zeal`]: one as set on it as they come hits the
            // anchor whoever is hitting it).
            Goal::Raid(victim) if nav.grid().region(here) == Some(Region::Team(victim)) => match bot.aim {
                Aim::Players => sight,
                Aim::Loot => (self.skill.aggro_at(now) * bot.temper.bold * bot.keen).max(if stung { 12.0 } else { 6.0 }).min(sight),
                Aim::Anchor if view.anchors[usize::from(victim) % ARENA_TEAMS] > 0.0 => {
                    (1.0 - bot.temper.zeal) * if stung { 7.0 } else { 3.2 }
                }
                Aim::Anchor => sight,
            },
            // As near as it lets people come just now ([`KEEN`]).
            _ => (self.skill.aggro_at(now) * bot.temper.bold * bot.keen).max(if stung { 9.0 } else { 0.0 }).min(sight),
        };
        // Standing toward someone who has stepped back from its base, it
        // goes at them too, for as far from where it stands as it means
        // to this time: not from wherever it has got to after them, or it
        // would be after them for as long as they kept walking (as one
        // was, 30 m down its own bridge, the first time this was tried).
        let watched = bot
            .vigil
            .filter(|_| bot.goal == Goal::Guard)
            .and_then(|vigil| vigil.stand.map(|stand| (vigil.foe, stand.spot, stand.watch)));
        let home = Region::Team(arena_team(seat));
        let on_plank = out_over_the_void(nav, here);
        view.seats
            .iter()
            .enumerate()
            .filter(|&(other, seen)| other != seat && seen.alive && !seen.unseen && arena_team(other) != arena_team(seat))
            .filter(|&(other, _)| stung || !bot.shun.is_some_and(|(shunned, _)| shunned == other))
            .filter_map(|(other, seen)| {
                let there = seen.at.xz();
                let away = there.distance(here);
                // Whoever it is on already it keeps after a little
                // further than it would have gone for them.
                let reach = if bot.foe == Some(other) { (reach * KEEPS_ON).min(sight.max(reach)) } else { reach };
                if away > reach || (seen.at.y - bot.body.at.y).abs() > 3.0 {
                    return None;
                }
                // Seeing someone off its island, it meets them on the
                // land: it does not hold the end of a bridge against
                // them, but for whoever it stands toward.
                if matches!(bot.goal, Goal::Defend | Goal::Guard) && away > 2.6 && nav.grid().region(there) != Some(home) {
                    let at_them = watched.is_some_and(|(foe, spot, watch)| foe == other && there.distance(spot) <= watch);
                    if !at_them {
                        return None;
                    }
                }
                // Hurt and out on a bridge, it starts nothing.
                if on_plank && me.health < 0.5 && away > 3.0 {
                    return None;
                }
                // Someone it can walk to; or, with only the void between
                // them, someone a jump away, over a hole in a bridge or
                // from the one bridge to the next.
                let jump = Self::reach_of_a_jump(RUN_SPEED, bot.body.high);
                if !nav.connected(here, there) && nav.crossing(here, there, jump).is_none() && nav.leap(here, there, jump).is_none() {
                    return None;
                }
                // And minding its own business, not someone across the
                // void that it would have to go round half the map to get
                // at, on the bridge beside its own or the next land over:
                // only whoever it can walk straight at, or who stands on
                // the land it stands on, or who is a jump away. (Since
                // 2026-11-15, with second ways in laid beside the first:
                // one laying its bridge 7 m from another's stood on it
                // facing whoever was on the other, setting off for them the
                // long way round and coming back to its planks, over and
                // over.)
                //
                // Whoever it is on already it stays on, and whoever is on
                // top of it it turns on, straight walk or none: the
                // straight walk to someone on the land comes and goes with
                // every step taken at a bridge's end, and one going by
                // that alone went at them one frame and about its
                // business the next, and stood where it was (twice in
                // sixty games, three seconds and more each); and from the
                // very edge of a plank, where a punch leaves one, there
                // is no straight walk to anybody.
                let after = matches!(bot.goal, Goal::Hunt | Goal::Defend | Goal::Guard)
                    || matches!(bot.goal, Goal::Raid(_)) && reach >= sight
                    || bot.foe == Some(other)
                    || away <= ON_TOP_OF_IT;
                if !after {
                    let (from, to) = (nav.nearest_walkable(here).unwrap_or(here), nav.nearest_walkable(there).unwrap_or(there));
                    let grid = nav.grid();
                    let same_land = grid.height(here).is_some() && grid.height(there).is_some() && grid.region(here) == grid.region(there);
                    if !nav.line(from, to) && !same_land && nav.leap(here, there, jump).is_none() {
                        return None;
                    }
                }
                // Whoever it is on already, a little sooner than another.
                Some((if bot.foe == Some(other) { away * 0.8 } else { away }, other))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, other)| other)
    }

    /// Which way the bot walks, at `pace`, to get to `to`: along its way
    /// there, found again when where it is going has moved, every few
    /// times `again` seconds, and, having no way, as soon as a plank is
    /// laid anywhere; nowhere once it is there. A plank only ever adds
    /// ground, so a way it has stays good while bridges are built (until
    /// 2026-11-12 every bot found its way again at every plank anyone
    /// laid). Getting nowhere for a while, it thinks again. **It does not
    /// walk the way as a ruler would**: it weaves a little to either side
    /// as it goes, the more the wider it weaves by nature, less on a
    /// bridge and none at all close to where it is going (Hajun,
    /// 2026-11-12: "more nonlinear/curve/slightly random not efficient
    /// movements to make it seem like a real player").
    fn walk_to(&mut self, bot: &mut Bot, view: &View, to: Vec2, pace: f32, again: f32) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        // Getting nowhere is reckoned from when it set out: one that has
        // stood its ground for a while, or fought where it stood, and
        // only now walks on has not been getting nowhere all that time
        // (until 2026-11-14 it had, and gave up what it was about for
        // eight seconds as it took its first step).
        if now - bot.walked_at > 0.5 {
            bot.progress = (here, now);
        }
        bot.walked_at = now;
        if here.distance(to) <= ARRIVED {
            bot.path.clear();
            bot.progress = (here, now);
            return Vec2::ZERO;
        }
        // A way is found from where the bot stood then, and it is walked
        // with a weave, and fought on, and knocked off: from where the
        // bot has got to, the next point of it may lie past the corner of
        // a stand, or of the land's edge beside a bridge's end, and a body
        // walked straight at that comes to rest in the corner and stays
        // there, walking (Hajun, 2026-11-14: "sometime your ai teammate
        // get stuck on the edge between the bridge and the island so it
        // keeps trying to go but its stuck. it happened when the ai
        // teammate built a bridge -> got sapphire -> came back to the
        // island"; in whole games of bots, eight times a game on average,
        // three seconds and more each, at the ends of bridges, the
        // corners of the stands and the things that stand in the middle).
        // So a few times a second it looks along the straight walk to
        // that point, and if that cannot be walked, finds its way again
        // from where it is.
        //
        // **From where a body's middle walks**: one that has come to
        // stand on the very edge of a plank, or on the rim of the land,
        // where a body stands and none walks, looks from the nearest
        // place that one does, which is where its way is found from
        // ([`Nav::nearest_walkable`]). Until 2026-11-15 it did not look
        // at all from there; and with bridges that come ashore at a
        // slant, beside the shore, one that cut the corner between the
        // bridge and the land, or was knocked across to the next bridge,
        // stood on the plank's edge facing where it was going across the
        // void until it gave that up: five times in sixty games.
        let lost = now - bot.looked_at >= LOOK_EVERY && {
            bot.looked_at = now;
            let from = nav.nearest_walkable(here);
            bot.path.first().is_some_and(|&next| from.is_some_and(|from| !nav.line(from, next)))
        };
        // A plank laid only adds to the ways there are; a blast takes
        // some away, and a way found before one may lead over a hole now.
        let stale = bot.path_to.is_none_or(|was| was.distance(to) > 1.0)
            || (bot.path.is_empty() && (bot.path_version != nav.version || now - bot.path_at > 0.5))
            || now - bot.path_at > again.max(0.5) * 4.0
            || bot.path_cuts != nav.cuts
            || lost;
        if stale {
            bot.cross = None;
            bot.path = match nav.path(here, to) {
                Some(path) => path,
                // No way there: to the edge of a hole in a bridge that it
                // can jump, if one lies on the way, and over it; or of one
                // wider than that, if it has a leap potion to drink there
                // (since 2026-11-15). At the pace it will have once what
                // is at its mouth is drunk.
                None => {
                    let run = if bot.drinking.is_some() { pace / DRINK_SLOW } else { pace };
                    let kit = view.outfits.kit(bot.seat);
                    let jumped = nav.crossing(here, to, Self::reach_of_a_jump(run, bot.body.high));
                    let leapt = || nav.crossing(here, to, Self::reach_of_a_jump(run, true)).filter(|_| kit.held(Utility::Leap) > 0);
                    match jumped.or_else(leapt).filter(|_| run >= RUN_SPEED * 0.9) {
                        Some(gap) => {
                            bot.cross = Some(gap);
                            nav.path(here, gap.a_in).unwrap_or_default()
                        }
                        None => Vec::new(),
                    }
                }
            };
            bot.path_to = Some(to);
            bot.path_version = nav.version;
            bot.path_at = now;
            bot.path_cuts = nav.cuts;
        }
        if let Some(gap) = bot.cross
            && here.distance(gap.a_in) <= 0.6
            && bot.body.free(nav)
            && !bot.awaits
        {
            // Wider than a jump of its own crosses: with the leap potion
            // it has, which it drinks here first, standing; and not with
            // any potion at its mouth, at half its pace.
            let run = if bot.drinking.is_some() { pace / DRINK_SLOW } else { pace };
            if bot.drinking.is_some() || now - bot.drank_at < 0.25 {
                bot.progress = (here, now);
                return Vec2::ZERO;
            }
            if gap.length() > Self::reach_of_a_jump(run, bot.body.high) {
                if !bot.body.high && view.outfits.kit(bot.seat).held(Utility::Leap) > 0 {
                    bot.drinking = Some((now + DRINK_SECS, Utility::Leap));
                    bot.progress = (here, now);
                    return Vec2::ZERO;
                }
                // Nothing to cross it with after all: something else.
                bot.cross = None;
                bot.path.clear();
                bot.path_to = None;
                if !matches!(bot.goal, Goal::Idle) {
                    bot.barred = Some((bot.goal, now + 8.0));
                    bot.think_at = now;
                }
                return Vec2::ZERO;
            }
            bot.cross = None;
            bot.path.clear();
            bot.path_to = None;
            bot.progress = (here, now);
            bot.leaping = Some(Leap {
                to: gap.b_in,
                pace,
                until: now + if bot.body.high { 3.5 } else { 2.0 },
                aloft: false,
            });
            return (gap.b_in - here).normalize_or_zero() * pace;
        }
        while bot.path.first().is_some_and(|next| next.distance(here) <= ARRIVED) {
            bot.path.remove(0);
        }
        // Getting nowhere: something else.
        if here.distance(bot.progress.0) > 0.75 {
            bot.progress = (here, now);
        } else if now - bot.progress.1 > 3.0 {
            bot.progress = (here, now);
            if let Some(&next) = bot.path.first() {
                bot.stalls += 1;
                bot.stalled = Some((here, next));
                // Held fast, and not standing by choice: the very next
                // step of the straight walk there cannot be taken.
                if !Self::gets_on(nav, &bot.body, (next - here).normalize_or_zero() * pace) {
                    bot.pinned += 1;
                }
            }
            bot.path.clear();
            bot.path_to = None;
            if !matches!(bot.goal, Goal::Idle) {
                bot.barred = Some((bot.goal, now + 8.0));
                bot.think_at = now;
            }
        }
        match bot.path.first() {
            Some(&next) => {
                let off = next - here;
                let far = off.length();
                // To one side and the other of the straight way, slowly,
                // and not in time with anyone else.
                bot.weave += (1.0 / 60.0) * (1.9 + 0.6 * bot.temper.weave);
                let room = if out_over_the_void(nav, here) { 0.35 } else { 1.0 };
                let swing = (bot.weave.sin() + 0.5 * (bot.weave * 2.3).sin()) * 0.16 * bot.temper.weave * room * (far / 3.0).min(1.0);
                // No further than the point itself in a frame.
                let straight = off.normalize_or_zero() * pace.min(far / (1.0 / 60.0));
                let woven = Vec2::from_angle(swing).rotate(straight);
                // It does not weave into a wall: where the next step of
                // its weave gets it nowhere, it walks straight; and where
                // that gets it nowhere either, it looks for its way again
                // at once.
                if Self::gets_on(nav, &bot.body, woven) {
                    woven
                } else {
                    if !Self::gets_on(nav, &bot.body, straight) {
                        bot.looked_at = f32::MIN;
                    }
                    straight
                }
            }
            None => Vec2::ZERO,
        }
    }

    /// With a leap potion on it, a bot on its way somewhere at a run with
    /// someone of another team right in its way jumps: over their heads,
    /// and on along its way in the air, to come down past them (since
    /// 2026-11-15). Soon, not on the instant, and not every time.
    fn vault(&mut self, bot: &mut Bot, view: &View, walk: Vec2, dt: f32) {
        if !bot.body.high || walk.length() < RUN_SPEED * 0.6 || !bot.body.free(view.nav) {
            return;
        }
        let here = bot.body.at.xz();
        let heading = walk.normalize_or_zero();
        let in_the_way = view.seats.iter().enumerate().any(|(other, seen)| {
            let off = seen.at.xz() - here;
            seen.alive
                && !seen.unseen
                && arena_team(other) != arena_team(bot.seat)
                && (1.2..5.5).contains(&off.length())
                && heading.dot(off.normalize_or_zero()) > 0.5
        });
        if in_the_way && self.dice.roll() < dt * 4.0 {
            bot.body.jump(view.nav);
        }
    }

    /// Whether `body`, standing where one can walk, gets anywhere by
    /// walking `walk` for a frame: whole, or along either side of a wall
    /// it is against, onto ground no more than a step up, as
    /// [`Body::step`] moves one. A body knocked to where none walks gets
    /// on wherever there is ground, and one in the air is not on its
    /// legs: neither is asked about.
    fn gets_on(nav: &Nav, body: &Body, walk: Vec2) -> bool {
        let here = body.at.xz();
        if !nav.walkable_at(here) || !body.free(nav) {
            return true;
        }
        let whole = here + walk * (1.0 / 30.0);
        [whole, Vec2::new(whole.x, here.y), Vec2::new(here.x, whole.y)]
            .into_iter()
            .any(|to| to != here && nav.walkable_at(to) && nav.ground(to).is_some_and(|there| there - body.at.y <= STEP))
    }

    /// How a bridge of `chord` metres bends on its way, thrown as it is
    /// begun ([`Span::bend`]): a sweep to one side, a wave and a ripple,
    /// each as a bell rings it, and all of them by how straight this
    /// bridge is to be, which is thrown as a bell rings too: most a
    /// little wavy, a few all but straight, a few all over the place
    /// (Hajun, 2026-11-12: "add some random bell curve variation to the
    /// 'straightness' of their bridging. straight line bridge look too
    /// ai"). Never so bent that it leaves or meets the land at more than
    /// a slant of [`MOST_SLANT`], nor that it is more than [`MOST_BENT`]
    /// times as long as the straight one.
    fn bends(&mut self, span: Span) -> Span {
        let chord = span.chord();
        let wavy = (1.0 + 0.5 * self.dice.bell()).clamp(0.1, 2.4) * chord / 44.0;
        let mut bend = [1.3, 0.7, 0.35].map(|far| far * wavy * self.dice.bell());
        for _ in 0..12 {
            // How it slants at its two ends, as so much to the side for
            // each metre on.
            let turn = std::f32::consts::PI / chord.max(1.0);
            let slants = [bend[0] + 2.0 * bend[1] + 3.0 * bend[2], bend[0] - 2.0 * bend[1] + 3.0 * bend[2]].map(|sum| (sum * turn).abs());
            let bent = span.bent(bend);
            if slants[0] <= MOST_SLANT && slants[1] <= MOST_SLANT && bent.length() <= chord * MOST_BENT - 0.2 {
                return bent;
            }
            bend = bend.map(|far| far * 0.8);
        }
        span
    }

    /// Lays the bridge from `from` to `to`: to where it begins; then out
    /// along it, bends and all, at the pace anyone builds at, each plank
    /// laid ahead of its feet and paid for in metres; and it is done at
    /// the far end.
    #[allow(clippy::too_many_arguments)]
    fn lay(&mut self, bot: &mut Bot, view: &View, from: Region, to: Region, pace: f32, order: &mut Order, dt: f32) -> Vec2 {
        let (nav, now) = (view.nav, view.elapsed);
        let here = bot.body.at.xz();
        // A blast has taken planks since it last looked: if any were of
        // the bridge it is laying, behind it, there is no walking on along
        // it. It leaves it as it is and thinks again: of mending the hole,
        // or of another bridge.
        if let Some(laying) = bot.laying
            && laying.cuts != nav.cuts
        {
            let holed = (0..laying.next.saturating_sub(1)).any(|plank| nav.broken(bot.seat as u8, laying.chain, plank as u16));
            // Unless the hole is behind it and it has the metres to lay
            // what is left: then on with it, to the land ahead, which is
            // the only land it can get to now (since 2026-11-15, when a
            // hole became the whole of the bridge behind it).
            let span = laying.span;
            let planks = span.planks();
            let under = ((here - span.from).dot(span.way()) / (span.chord() / planks as f32).max(1e-3)).floor().max(0.0) as usize;
            let behind = (0..under.min(laying.next.saturating_sub(1))).any(|plank| nav.broken(bot.seat as u8, laying.chain, plank as u16));
            let ahead = (under..laying.next.saturating_sub(1)).any(|plank| nav.broken(bot.seat as u8, laying.chain, plank as u16));
            let left = span.length() * (1.0 - laying.next.saturating_sub(1) as f32 / planks as f32);
            let finishes = behind && !ahead && bot.metres + 0.5 >= left && out_over_the_void(nav, here);
            if holed && !finishes {
                bot.laying = None;
                bot.planned = None;
                self.claims.retain(|claim| claim.2 != bot.seat);
                bot.think_at = now;
                return Vec2::ZERO;
            }
            bot.laying = Some(Laying { cuts: nav.cuts, ..laying });
        }
        let laying = match bot.laying {
            Some(laying) => laying,
            None => {
                // The bridge its last thought settled on, if that is the
                // one it is here to lay. (Until 2026-11-13 this asked for
                // it again, as if for the one of two bots that goes out:
                // a person's teammate whose kind opens to a smaller
                // island, or late, was given that bridge, or none yet, in
                // place of the one to the middle it had the metres for.)
                let Some((_, _, span)) = bot.planned.filter(|planned| (planned.0, planned.1) == (from, to)) else {
                    bot.think_at = now;
                    return Vec2::ZERO;
                };
                // With a Bridge Builder on a button: to where the bridge
                // is to begin, and the builder is sent off from there to
                // lay it, straight, while the bot goes about something
                // else.
                if view.outfits.kit(bot.seat).held(Utility::Builder) > 0 && span.length() <= BUILDER_METRES - 2.0 {
                    let near = nav.nearest_walkable(span.from).unwrap_or(span.from);
                    if here.distance(near) > ARRIVED + 0.3 {
                        return self.walk_to(bot, view, near, pace, 2.0);
                    }
                    order.build = true;
                    bot.sent = Some(Sent {
                        span,
                        from,
                        to,
                        chain: bot.chains,
                        next: 0,
                        along: 0.0,
                    });
                    bot.chains += 1;
                    match self.begun.iter_mut().find(|begun| (begun.0, begun.1) == (from, to)) {
                        Some(begun) => begun.2 += 1,
                        None => self.begun.push((from, to, 1)),
                    }
                    self.claims.retain(|claim| claim.2 != bot.seat);
                    bot.planned = None;
                    // One it could do without is off its mind: the
                    // builder has it.
                    if bot.scheme.is_some_and(|scheme| (scheme.0, scheme.1) == (from, to)) {
                        bot.scheme = None;
                        bot.schemed = bot.schemed.saturating_add(1);
                    }
                    bot.think_at = now;
                    return Vec2::ZERO;
                }
                let laying = Laying {
                    // A hole mended is a few planks, laid straight.
                    span: if span.chord() < MEND { span } else { self.bends(span) },
                    from,
                    to,
                    chain: bot.chains,
                    next: 0,
                    cuts: nav.cuts,
                };
                bot.chains += 1;
                match self.begun.iter_mut().find(|begun| (begun.0, begun.1) == (from, to)) {
                    Some(begun) => begun.2 += 1,
                    None => self.begun.push((from, to, 1)),
                }
                bot.laying = Some(laying);
                laying
            }
        };
        let span = laying.span;
        let (way, planks, chord) = (span.way(), span.planks(), span.chord());
        // A rung to so much of the straight line, and so much bridge.
        let (step, plank) = (chord / planks as f32, span.length() / planks as f32);
        let along = (here - span.from).dot(way);
        let onto = span.point(along / chord.max(1e-3));
        let aside = here.distance(onto);
        // Not at its line yet: to the nearest a body can walk to where it
        // begins, or, taking it up again, to as far as it is laid.
        let reached = span.rung(laying.next.saturating_sub(1));
        let on_line = aside < 0.9 && along > -1.5 && along < chord + 1.0;
        if !on_line || (laying.next == 0 && here.distance(span.from) > 2.2) {
            let to = if laying.next == 0 { span.from } else { reached };
            let near = nav.nearest_walkable(to).unwrap_or(to);
            if here.distance(near) > ARRIVED + 0.3 {
                return self.walk_to(bot, view, near, pace, 2.0);
            }
        }
        // Its planks, a rung at a time, kept ahead of its feet: the first
        // rung is the bridge's root, and costs nothing.
        // The last rung goes down with the one before it: a bridge a
        // plank short of its end is one a bot may jump from, to get at
        // whoever it is after, and leave so (a hole mended all but its
        // last plank, in the test of it, once the squares walked along a
        // plank ended where the plank does).
        let mut next = laying.next;
        while next <= planks && ((next as f32 - 1.0) * step < along + LAID_AHEAD + step || next == planks) {
            let cost = if next == 0 { 0.0 } else { plank };
            if bot.metres + 1e-3 < cost {
                break;
            }
            bot.metres -= cost;
            order.rungs.push((laying.chain, next as u16, span.rung(next), span.heading_at(next)));
            next += 1;
        }
        bot.laying = Some(Laying { next, ..laying });
        order.carrying = true;
        // Laid to its end, and walked to it: done.
        if next > planks && along >= chord - ARRIVED {
            bot.laying = None;
            self.claims.retain(|claim| claim.2 != bot.seat);
            // One it could do without is done with; and having built its
            // way onto another team's island, it goes in after it, for a
            // while, whoever's turn it is.
            if bot.scheme.is_some_and(|scheme| (scheme.0, scheme.1) == (from, to)) {
                bot.scheme = None;
                bot.schemed = bot.schemed.saturating_add(1);
            }
            if let Region::Team(other) = to
                && other != arena_team(bot.seat)
            {
                bot.door = Some((other, now + self.dice.around(20.0, 45.0)));
            }
            bot.think_at = now;
            return Vec2::ZERO;
        }
        // Out along it, no further than it is laid.
        let laid = (next.saturating_sub(1)) as f32 * step;
        let limit = if next > planks { chord } else { (laid - 0.4).max(0.0) };
        // (At the pace its team builds at, quickened or not.)
        let speed = builder_speed(&view.outfits.team(bot.seat)).min((limit - along).max(0.0) / dt.max(1e-3));
        // Onto its middle line, and on along it as it bends.
        let sideways = onto - here;
        if speed <= 0.0 && sideways.length() < 0.05 {
            // Out of metres on the way: nothing for it but to go and get
            // more.
            if bot.metres + 1e-3 < plank && next <= planks {
                bot.think_at = bot.think_at.min(now + 0.2);
            }
            return Vec2::ZERO;
        }
        span.tangent(along / chord.max(1e-3)) * speed + sideways.clamp_length_max(1.0) * 3.0
    }
}

/// Moves on the Bridge Builder `bot` has sent off, if any: along its
/// bridge at the pace anyone builds at, each plank laid a rung or two
/// ahead of it, to the far shore, where it is done.
fn build_on(bot: &mut Bot, speed: f32, dt: f32, order: &mut Order) {
    let Some(mut sent) = bot.sent else {
        return;
    };
    let (planks, chord) = (sent.span.planks(), sent.span.chord());
    let step = chord / planks as f32;
    sent.along += speed * dt;
    while sent.next <= planks && (sent.next as f32 - 1.0) * step < sent.along + LAID_AHEAD + step {
        order
            .rungs
            .push((sent.chain, sent.next as u16, sent.span.rung(sent.next), sent.span.heading_at(sent.next)));
        sent.next += 1;
    }
    bot.sent = (sent.next <= planks).then_some(sent);
}

/// Whether `at` is over the plank between the rungs `a` and `b`: within
/// its width of its middle line, between the two or a step past either.
/// For telling who stands on planks that a blast would take
/// ([`Squad::worth_blowing`]).
fn over_plank(at: Vec2, a: Vec2, b: Vec2) -> bool {
    let along = b - a;
    let length = along.length();
    if length < 1e-4 {
        return false;
    }
    let way = along / length;
    let off = at - a;
    (-0.3..=length + 0.3).contains(&off.dot(way)) && off.perp_dot(way).abs() <= crate::nav::PLANK_HALF_WIDTH
}

/// Whether someone `at` is out on a bridge with nothing but the void
/// under it: on a plank, and over no land.
fn out_over_the_void(nav: &Nav, at: Vec2) -> bool {
    nav.grid().height(at).is_none() && nav.on_plank(at)
}

/// How surely a punch thrown from `from` carries whoever stands `at` off
/// the land and into the void, from 0 to 1: by whether there is ground
/// where a knock would put them down, as far on as one carries
/// ([`CARRY`]). What a bot goes by in choosing where to stand
/// ([`Squad::spot`]): where its punch throws them off, and theirs does
/// not throw it off.
fn flung(nav: &Nav, from: Vec2, at: Vec2) -> f32 {
    let away = (at - from).normalize_or_zero();
    if away == Vec2::ZERO {
        return 0.0;
    }
    CARRY.iter().filter(|far| nav.ground(at + away * **far).is_none()).count() as f32 / CARRY.len() as f32
}

/// Where a bot standing `here` on the land lies in wait for someone
/// coming over a bridge from `there`: beside where their bridge comes
/// ashore, on the way between the two of them, a little inland and well
/// to one side (`side`, or the other if there is no standing there), so
/// that a punch at whoever steps ashore throws them back over the side of
/// their bridge. `None` if they are far off, the landfall is far from the
/// bot, or there is nowhere to stand.
fn lair(nav: &Nav, here: Vec2, there: Vec2, side: f32) -> Option<Vec2> {
    let whole = here.distance(there);
    if !(2.0..=16.0).contains(&whole) {
        return None;
    }
    let way = (here - there) / whole;
    let landfall = (1..=(whole / 0.4) as usize)
        .map(|step| there + way * (step as f32 * 0.4))
        .find(|&at| nav.grid().clear(at))?;
    if landfall.distance(here) > 9.0 {
        return None;
    }
    [side, -side]
        .into_iter()
        .map(|side| landfall + way * 1.2 + Vec2::new(-way.y, way.x) * side * 1.9)
        .find(|&at| nav.walkable_at(at) && nav.grid().height(at).is_some())
}

/// The two smaller islands beside `team`'s own, by their pads' numbers.
fn beside(team: u8) -> [u8; 2] {
    let island = ARENA_ISLANDS[usize::from(team) % ARENA_ISLANDS.len()];
    let mut nearest: Vec<u8> = (0..SAPPHIRE_PADS.len() as u8).collect();
    nearest.sort_by(|&a, &b| {
        SAPPHIRE_PADS[usize::from(a)]
            .distance_squared(island)
            .total_cmp(&SAPPHIRE_PADS[usize::from(b)].distance_squared(island))
    });
    [nearest[0], nearest[1]]
}

/// What a bot would buy, in the order it would: by what it does on its
/// team, what it holds and has, how many metres of bridge it is `short`
/// of for the span it means to lay, **and who it is**
/// ([`Persona::spends`]): everything it might buy next has a place in the
/// order they all bought in until 2026-11-12 (the team's upgrades, the
/// heal pool first; the aura sword and shield; a sword; a shield;
/// something for the second button; a better shield), and its leaning to
/// each kind of thing moves that up or down. So one puts a fireball
/// before its sword, another its team's defences before its generator,
/// and a third buys as they all did. What goes on its buttons is what it
/// means to carry this life (`loadout`, [`Persona::loadout`]): each kind
/// in its turn, the first before the rest, as many of each as it keeps,
/// and a second or a third of a kind only after the things it has none
/// of. A bridge it is short of (`short`) is bought as metres, or, by one
/// that carries Bridge Builders and has the amethyst, as a builder to lay
/// it, unless it is part way along laying it itself (`laying`). And with
/// a bridge worth a builder to be had (`spare`, [`Squad::spare_span`]),
/// one that carries them buys one in its turn among the rest.
#[allow(clippy::too_many_arguments)]
fn shopping(
    persona: &Persona,
    loadout: &[(Utility, u8)],
    role: Role,
    alone: bool,
    elapsed: f32,
    purse: &Purse,
    kit: &Kit,
    team: &Upgrades,
    short: f32,
    laying: bool,
    spare: bool,
) -> Vec<Ware> {
    let mut list = Vec::new();
    let has = |item: Item| purse[item.index()];
    // Its bridge before anything: a builder to lay it, or the metres.
    let builder = Ware::Utility(Utility::Builder);
    let builds = loadout.iter().any(|(kind, _)| *kind == Utility::Builder)
        && has(Item::Amethyst) >= 1
        && kit.held(Utility::Builder) == 0
        && price(builder, kit, team).is_some();
    if short > 0.0 {
        if builds && !laying && short <= BUILDER_METRES - 2.0 {
            list.push(builder);
        } else {
            let buys = (short / BRIDGE_A_BUY).ceil() as usize;
            list.extend(std::iter::repeat_n(Ware::Bridge, buys.min(4)));
        }
    }
    let [sword, shield, button, generator, defences, swords] = persona.spends;
    let keeper = role == Role::Keeper && !alone;
    let mut wanted: Vec<(f32, Ware)> = Vec::new();
    // Sapphire is the team's: the heal pool, its generator, its swords,
    // its shields, a trap.
    if has(Item::Sapphire) > 0 {
        for (rank, leaning, ware) in [
            (1.0, defences, Ware::HealPool),
            (0.98, generator, Ware::Forge),
            (0.96, swords, Ware::StrongerSwords),
            (0.94, defences, Ware::BetterShields),
            (0.92, defences, Ware::Trap),
            // Its building quickened (since 2026-11-15; Hajun: "i think
            // the ai doesnt use the most recently added items like the
            // advanced bridger upgrade or leap potion"): early for one
            // given to bridges, and late, with sapphire to spare, for the
            // rest; each step after the first a little less wanted than
            // the last, as each costs twice as much.
            (
                0.95 * [1.0, 0.8, 0.6, 0.0][usize::from(team.build_speed).min(3)],
                0.35 + 1.1 * persona.schemes,
                Ware::BuildSpeed,
            ),
        ] {
            if price(ware, kit, team).is_some() {
                wanted.push((rank * leaning, ware));
            }
        }
    }
    // Amethyst is the best of its own arms.
    if has(Item::Amethyst) >= 4 && kit.sword < 3 {
        wanted.push((0.9 * sword, Ware::Sword(3)));
    }
    if has(Item::Amethyst) >= 6 && kit.shield < 3 {
        wanted.push((0.88 * shield, Ware::Shield(3)));
    }
    // A sword: the silver one once the gold for it is near, and until
    // then the basic one, which the keeper has first of all.
    if kit.sword < 2 && (has(Item::Gold) >= 4 || kit.sword == 1) {
        wanted.push((0.86 * sword, Ware::Sword(2)));
    }
    if kit.sword == 0 && has(Item::Gold) < 4 && (keeper || elapsed > 45.0) {
        wanted.push((0.84 * sword, Ware::Sword(1)));
    }
    if kit.shield == 0 && (keeper || kit.sword > 0 || shield > 1.25) {
        wanted.push((0.8 * shield, Ware::Shield(1)));
    }
    // Its buttons: each kind it means to carry, in its turn, once it is
    // armed, or at once if that is what it is like; a fireball late on and
    // with silver to spare, unless it is one for fireballs. A keeper with
    // nobody to fetch it amethyst makes do with a healing potion in place
    // of what amethyst buys. As many of a kind as it keeps, the second and
    // the third of one well after the first of the next.
    let keen = button > 1.2;
    let stuck = keeper && has(Item::Amethyst) == 0;
    // A builder for a bridge its team could do without, in its turn.
    if spare && builds && short == 0.0 {
        wanted.push((0.74 * button, builder));
    }
    let mut kinds: Vec<Utility> = Vec::new();
    for &(kind, keeps) in loadout {
        let kind = match kind {
            Utility::Speed | Utility::Invisibility | Utility::Builder | Utility::Dynamite | Utility::Leap if stuck => Utility::Healing,
            other => other,
        };
        if kinds.contains(&kind) {
            continue;
        }
        kinds.push(kind);
        let ready = match kind {
            Utility::Fireball => (elapsed > 300.0 && kit.sword > 0 && kit.shield > 0) || (keen && elapsed > 45.0),
            Utility::Healing => kit.sword > 0 || keen,
            Utility::Speed | Utility::Invisibility | Utility::Dynamite | Utility::Leap => {
                has(Item::Amethyst) > 0 || (keen && elapsed > 60.0)
            }
            // Bought for a bridge it is short of, before anything.
            Utility::Builder => false,
        };
        let held = kit.held(kind);
        if ready && held < keeps && price(Ware::Utility(kind), kit, team).is_some() {
            let turn = 0.76 - 0.05 * (kinds.len() - 1) as f32;
            let again = if held > 0 { 0.6 } else { 1.0 };
            wanted.push((turn * again * button, Ware::Utility(kind)));
        }
    }
    if kit.sword >= 2 && kit.shield < 2 {
        wanted.push((0.72 * shield, Ware::Shield(2)));
    }
    wanted.sort_by(|a, b| b.0.total_cmp(&a.0));
    list.extend(wanted.into_iter().map(|(_, ware)| ware));
    list
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::bridge_wars::BUILDER_SPEED;
    use crate::nav::Grid;

    /// An island 12 m across at the origin, its top 0.25 m up, `Team(0)`.
    fn island() -> Nav {
        let mut grid = Grid::blank(60, 60, Vec2::new(-15.0, -15.0));
        for row in 0..60 {
            for column in 0..60 {
                let away = grid.centre_of(column, row).length();
                if away <= 6.0 {
                    grid.set(column, row, Some(0.25), away <= 5.5, Some(Region::Team(0)));
                }
            }
        }
        grid.place_land(Region::Team(0), Vec2::ZERO);
        Nav::new(Arc::new(grid))
    }

    /// Two islands 12 m across, 30 m apart on the x axis, `Team(1)` at the
    /// origin and `Middle` at (30, 0), with a straight bridge between
    /// them: for trying a fight at a bridge's end.
    fn bridged() -> Nav {
        let (mut nav, span) = two_islands();
        for index in 0..=span.planks() {
            nav.hear(1, 0, index as u16, span.rung(index));
        }
        assert!(nav.joined(Region::Team(1), Region::Middle));
        nav
    }

    /// The two islands of [`bridged`] with nothing between them yet, and
    /// the straight bridge there would be.
    fn two_islands() -> (Nav, Span) {
        let mut grid = Grid::blank(100, 40, Vec2::new(-10.0, -10.0));
        for row in 0..40 {
            for column in 0..100 {
                let at = grid.centre_of(column, row);
                for (centre, region) in [(Vec2::ZERO, Region::Team(1)), (Vec2::new(30.0, 0.0), Region::Middle)] {
                    let away = at.distance(centre);
                    if away <= 6.0 {
                        grid.set(column, row, Some(0.25), away <= 5.5, Some(region));
                    }
                }
            }
        }
        grid.place_land(Region::Team(1), Vec2::ZERO);
        grid.place_land(Region::Middle, Vec2::new(30.0, 0.0));
        let span = grid.span(Region::Team(1), Region::Middle, 0.0).expect("a span between them");
        (Nav::new(Arc::new(grid)), span)
    }

    /// What became of a bot sent over the bridge at someone standing on
    /// the far land beside its end, who punches it whenever it is in
    /// their reach, twice a second, and is never moved themselves: whether
    /// it fell, when, whether it ever stood on the far land, and how it
    /// went about them.
    struct Crossing {
        fell: Option<f32>,
        landed: bool,
        stances: Vec<Stance>,
        knocks: u32,
    }

    fn cross_to_someone_waiting(seed: u64, who: Option<&str>, waits_at: Vec2, secs: f32) -> Crossing {
        let nav = bridged();
        let mut squad = Squad::new(seed, Skill::MODERATE);
        let seat = 5;
        squad.seat(seat, Vec3::new(4.0, 0.25, 0.0), 0.0);
        if let Some(who) = who {
            assert!(squad.cast(seat, who));
        }
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        let mut seen = vec![Seen::default(); ARENA_SEATS];
        seen[0] = Seen {
            at: Vec3::new(waits_at.x, 0.25, waits_at.y),
            alive: true,
            in_play: true,
            unseen: false,
            person: true,
            health: 1.0,
        };
        let mut told = Crossing {
            fell: None,
            landed: false,
            stances: Vec::new(),
            knocks: 0,
        };
        let dt = 1.0 / 60.0;
        let mut cooldown = 0.0f32;
        // Late in a game, when everyone hunts everyone.
        let mut now = Skill::MODERATE.hunt_from + 5.0;
        let until = now + secs;
        while now < until {
            now += dt;
            cooldown -= dt;
            let body = squad.bot(seat).expect("seated").body;
            seen[seat] = Seen {
                at: body.at,
                alive: true,
                in_play: true,
                unseen: false,
                person: false,
                health: 1.0,
            };
            if body.at.y < -5.0 {
                told.fell = Some(now - (until - secs));
                break;
            }
            let view = View {
                elapsed: now,
                nav: &nav,
                seats: &seen,
                purses: &purses,
                outfits: &outfits,
                mint: &mint,
                anchors: [120.0; ARENA_TEAMS],
                sticks: &[],
            };
            squad.step(seat, &view, dt);
            let bot = squad.bot(seat).expect("seated");
            if let Some(stance) = bot.stance()
                && told.stances.last() != Some(&stance)
            {
                told.stances.push(stance);
            }
            let at = bot.body.at;
            told.landed |= nav.grid().region(at.xz()) == Some(Region::Middle) && !nav.on_plank(at.xz()) && at.y <= 0.26;
            if cooldown <= 0.0 && at.xz().distance(waits_at) <= REACH + BODY && (at.y - 0.25).abs() < 2.0 {
                squad.knock(seat, waits_at, false);
                told.knocks += 1;
                cooldown = 0.5;
            }
        }
        told
    }

    /// For tuning by: `cargo test -p roundtown_net crossings -- --ignored
    /// --nocapture`. How a bot of each kind fares against someone waiting
    /// beside the far end of its bridge.
    #[test]
    #[ignore]
    fn crossings_to_someone_waiting_at_a_bridge_s_end() {
        // Beside where the bridge comes ashore, to one side: where a
        // punch at whoever arrives throws them off the side of it.
        let waits_at = Vec2::new(26.2, 1.9);
        let mut all = (0, 0, 0);
        for persona in &PERSONAS {
            let (mut fell, mut landed, mut knocks) = (0, 0, 0);
            let mut when = 0.0;
            let trials = 60;
            for seed in 0..trials {
                let told = cross_to_someone_waiting(seed, Some(persona.name), waits_at, 30.0);
                fell += u32::from(told.fell.is_some());
                when += told.fell.unwrap_or(30.0);
                landed += u32::from(told.landed);
                knocks += told.knocks;
            }
            println!(
                "{}: fell {fell} of {trials} (after {:.1} s on average), stood on the far land {landed}, knocked {:.1} times",
                persona.name,
                when / trials as f32,
                knocks as f32 / trials as f32
            );
            all = (all.0 + fell, all.1 + landed, all.2 + trials as u32);
        }
        println!("all: fell {} of {}, stood on the far land {}", all.0, all.2, all.1);
        let told = cross_to_someone_waiting(3, None, waits_at, 30.0);
        println!("one of them: fell {:?}, landed {}, {:?}, {} knocks", told.fell, told.landed, told.stances, told.knocks);
    }

    #[test]
    fn a_body_walks_where_it_can_and_never_off_an_edge() {
        let nav = island();
        let mut body = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        assert!(body.free(&nav));
        // East at a run for two seconds: it is stopped short of the edge,
        // on the last of the land a body walks.
        for _ in 0..120 {
            body.step(&nav, Vec2::X * RUN_SPEED, 1.0 / 60.0);
        }
        assert!(body.at.x > 5.0 && body.at.x < 5.6, "{}", body.at);
        assert_eq!(body.at.y, 0.25);
        assert!(body.free(&nav));
        // Facing the way it walked: east.
        assert!((body.yaw + std::f32::consts::FRAC_PI_2).abs() < 1e-3, "{}", body.yaw);
        // Walked at a slant into the edge, it slides along it.
        let before = body.at;
        for _ in 0..30 {
            body.step(&nav, Vec2::new(1.0, 1.0).normalize() * RUN_SPEED, 1.0 / 60.0);
        }
        assert!(body.at.z > before.z + 0.5 && nav.walkable_at(body.at.xz()), "{}", body.at);
    }

    #[test]
    fn a_knock_carries_a_body_and_off_the_edge_it_falls_unless_its_legs_bring_it_back() {
        let nav = island();
        let frame = 1.0 / 60.0;
        // Knocked from the west while standing in the middle: up and
        // east, down again on the island, and its own to move after.
        let mut body = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        body.knock(Vec2::new(-1.0, 0.0), false, 1.0);
        assert!(!body.free(&nav));
        assert_eq!((body.shove, body.lift), (Vec2::X * KNOCK_BACK, KNOCK_UP));
        let mut highest = 0.0f32;
        for _ in 0..180 {
            body.step(&nav, Vec2::ZERO, frame);
            highest = highest.max(body.at.y);
        }
        assert!(highest > 1.2, "it rose to {highest}");
        assert!(body.free(&nav), "{body:?}");
        assert!(body.at.x > 2.0 && body.at.x < 5.6 && body.at.y == 0.25, "{}", body.at);
        let carried = body.at.x;
        // Its legs are its own all the while: running back at whoever
        // knocked it, it is carried hardly at all.
        let mut back = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        back.knock(Vec2::new(-1.0, 0.0), false, 1.0);
        let mut furthest = 0.0f32;
        for _ in 0..50 {
            back.step(&nav, Vec2::NEG_X * RUN_SPEED, frame);
            furthest = furthest.max(back.at.x);
        }
        assert!(furthest < carried - 1.5, "carried {furthest} m against {carried}");
        assert!(back.at.x < 0.0, "and back past where it stood: {}", back.at);
        // On the run from near the edge, it goes over it and falls, and
        // is never its own to move again.
        let mut body = Body::standing(Vec3::new(4.0, 0.25, 0.0), 0.0);
        body.knock(Vec2::new(3.0, 0.0), true, 1.0);
        for _ in 0..180 {
            body.step(&nav, Vec2::ZERO, frame);
            assert!(!body.free(&nav) || body.at.y > 0.0);
        }
        assert!(body.at.x > 6.5 && body.at.y < -20.0, "{}", body.at);
        // A knock that takes it a little way out over the void: with its
        // legs still, it is gone; steering back for the land, it lands on
        // it. And in the air over the land, its legs do not take it out
        // over the void: only the knock does.
        let mut gone = Body::standing(Vec3::new(3.2, 0.25, 0.0), 0.0);
        gone.knock(Vec2::new(2.0, 0.0), false, 1.0);
        let mut saved = gone;
        let mut out = false;
        for _ in 0..180 {
            gone.step(&nav, Vec2::ZERO, frame);
            // Its legs still until it is out over the void; then back
            // for the land.
            out |= nav.ground(saved.at.xz()).is_none();
            saved.step(&nav, if out { Vec2::NEG_X * RUN_SPEED } else { Vec2::ZERO }, frame);
        }
        assert!(gone.at.y < -20.0, "{}", gone.at);
        assert!(out, "it was carried out over the void");
        assert!(saved.free(&nav) && saved.at.y == 0.25 && saved.at.x < 5.6, "{saved:?}");
        let mut jumper = Body::standing(Vec3::new(4.5, 0.25, 0.0), 0.0);
        assert!(jumper.jump(&nav) && !jumper.jump(&nav), "one jump from the ground");
        let mut highest = 0.0f32;
        for _ in 0..120 {
            jumper.step(&nav, Vec2::X * RUN_SPEED, frame);
            highest = highest.max(jumper.at.y);
        }
        assert!(highest > 1.6, "a jump as high as anyone's: {highest}");
        assert!(jumper.free(&nav) && jumper.at.x < 5.6, "it came down on the land at its edge: {}", jumper.at);
        // A zombie takes a share of it.
        let mut zombie = Body::standing(Vec3::new(0.0, 0.25, 0.0), 0.0);
        zombie.knock(Vec2::new(-1.0, 0.0), true, 0.3);
        assert!((zombie.shove.x - 6.0).abs() < 1e-4 && (zombie.lift - 2.4).abs() < 1e-4);
    }

    #[test]
    fn a_body_stood_under_the_island_s_top_is_put_onto_it_and_one_fallen_under_it_goes_on_falling() {
        let nav = island();
        // Where a device's arrival stands it: at the land's own height, a
        // quarter of a metre under the island's top. It is put onto it,
        // and is its own to move.
        let mut body = Body::standing(Vec3::new(1.0, 0.0, 1.0), 0.0);
        assert!(!body.free(&nav));
        body.step(&nav, Vec2::ZERO, 1.0 / 60.0);
        assert_eq!(body.at.y, 0.25);
        assert!(body.free(&nav));
        for _ in 0..30 {
            body.step(&nav, Vec2::X * RUN_SPEED, 1.0 / 60.0);
        }
        assert!(body.at.x > 4.0 && body.at.y == 0.25, "{}", body.at);
        // Knocked off the edge and carried back in under the island as it
        // falls, it goes on falling: it does not hang there.
        let mut fallen = Body::standing(Vec3::new(5.0, -3.0, 0.0), 0.0);
        fallen.lift = -6.0;
        fallen.shove = Vec2::new(-4.0, 0.0);
        for _ in 0..180 {
            fallen.step(&nav, Vec2::ZERO, 1.0 / 60.0);
            assert!(!fallen.free(&nav));
        }
        assert!(fallen.at.y < -30.0, "{}", fallen.at);
        // Just under the edge, it catches it.
        let mut caught = Body::standing(Vec3::new(5.0, 0.1, 0.0), 0.0);
        caught.lift = -1.0;
        caught.step(&nav, Vec2::ZERO, 1.0 / 60.0);
        assert_eq!(caught.at.y, 0.25);
    }

    /// Someone with no leaning to anything.
    fn plain() -> Persona {
        Persona {
            name: "plain",
            spends: [1.0; 6],
            ..PERSONAS[0]
        }
    }

    fn persona(name: &str) -> &'static Persona {
        PERSONAS.iter().find(|persona| persona.name == name).expect("one of them")
    }

    #[test]
    fn what_a_bot_buys_goes_by_what_it_holds_and_what_it_does_and_who_it_is() {
        let (kit, team) = (Kit::default(), Upgrades::default());
        let plain = plain();
        let healing = [(Utility::Healing, 1)];
        let list = |role, elapsed, purse: &Purse, kit: &Kit, short| {
            shopping(&plain, &healing, role, false, elapsed, purse, kit, &team, short, false, false)
        };
        // Short of 44 m of bridge: two buys of it before anything.
        assert_eq!(&list(Role::Runner, 10.0, &[8, 0, 0, 0], &kit, 44.6)[..2], &[Ware::Bridge, Ware::Bridge]);
        // The keeper: a basic sword, and a shield.
        assert_eq!(list(Role::Keeper, 10.0, &[10, 1, 0, 0], &kit, 0.0), vec![Ware::Sword(1), Ware::Shield(1)]);
        // The runner waits for the silver sword's gold, early on.
        assert!(list(Role::Runner, 10.0, &[10, 1, 0, 0], &kit, 0.0).is_empty());
        assert_eq!(list(Role::Runner, 60.0, &[10, 5, 0, 0], &kit, 0.0)[0], Ware::Sword(2));
        // Sapphire goes on the team, the heal pool first.
        let armed = Kit { sword: 2, shield: 1, ..kit };
        let bought = list(Role::Runner, 200.0, &[0, 0, 3, 0], &armed, 0.0);
        assert_eq!(&bought[..2], &[Ware::HealPool, Ware::Forge]);
        // With a sword, a healing potion for the second button.
        assert!(bought.contains(&Ware::Utility(Utility::Healing)) && bought.contains(&Ware::Shield(2)));

        // Who it is moves things up and down. The daredevil has its
        // fireball before its sword; the sentinel, by the book, its sword.
        let silver: Purse = [60, 0, 0, 0];
        let fireballs = [(Utility::Fireball, 3)];
        let daring = shopping(persona("ESTP"), &fireballs, Role::Runner, false, 60.0, &silver, &kit, &team, 0.0, false, false);
        assert_eq!(daring[0], Ware::Utility(Utility::Fireball), "{daring:?}");
        let steady = shopping(persona("ISTJ"), &fireballs, Role::Runner, false, 60.0, &silver, &kit, &team, 0.0, false, false);
        assert!(matches!(steady[0], Ware::Sword(1) | Ware::Shield(1)), "{steady:?}");
        assert!(!steady.contains(&Ware::Utility(Utility::Fireball)), "not before it is armed, and late: {steady:?}");
        // The defender spends its team's sapphire on defences, the
        // commander on swords.
        let sapphire: Purse = [0, 0, 9, 0];
        let safe = shopping(persona("ISFJ"), &healing, Role::Keeper, false, 200.0, &sapphire, &armed, &team, 0.0, false, false);
        assert_eq!(&safe[..3], &[Ware::HealPool, Ware::BetterShields, Ware::Trap], "{safe:?}");
        let bent_on = shopping(persona("ENTJ"), &healing, Role::Runner, false, 200.0, &sapphire, &armed, &team, 0.0, false, false);
        assert_eq!(bent_on[0], Ware::StrongerSwords, "{bent_on:?}");
        // The troll has an invisibility potion as soon as it has the
        // amethyst for it, before the best of swords; a keeper who
        // fancies one, with nobody to fetch amethyst, a healing potion.
        let amethyst: Purse = [0, 0, 0, 4];
        let unseen = [(Utility::Invisibility, 1)];
        let sly = shopping(persona("ESFP"), &unseen, Role::Runner, false, 200.0, &amethyst, &armed, &team, 0.0, false, false);
        assert_eq!(sly[0], Ware::Utility(Utility::Invisibility), "{sly:?}");
        let home = shopping(persona("ESFP"), &unseen, Role::Keeper, false, 200.0, &[0, 9, 0, 0], &armed, &team, 0.0, false, false);
        assert!(home.contains(&Ware::Utility(Utility::Healing)) && !home.contains(&Ware::Utility(Utility::Invisibility)));
        // Sixteen of them, each by its own name, and none with odds of
        // nothing at all.
        let mut names: Vec<&str> = PERSONAS.iter().map(|persona| persona.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 16);
        for persona in &PERSONAS {
            assert!(persona.opening.iter().sum::<f32>() > 0.0 && persona.carries.iter().sum::<f32>() > 0.0, "{}", persona.name);
            let [least, most] = persona.pockets;
            assert!(1 <= least && least <= most && usize::from(most) <= KINDS.len(), "{}", persona.name);
            assert!(persona.fights.iter().all(|leaning| *leaning > 0.0), "{}", persona.name);
        }
    }

    #[test]
    fn they_go_for_people_from_further_off_as_the_game_goes_on() {
        let skill = Skill::MODERATE;
        assert_eq!(skill.aggro_at(0.0), 6.0);
        assert_eq!(skill.aggro_at(150.0), 6.0);
        assert!((skill.aggro_at(375.0) - 15.0).abs() < 0.01);
        assert_eq!(skill.aggro_at(600.0), HUNT_RADIUS);
        assert_eq!(skill.aggro_at(1400.0), HUNT_RADIUS);
        // The same dice, the same throws.
        let (mut a, mut b) = (Dice::new(4), Dice::new(4));
        for _ in 0..50 {
            let roll = a.roll();
            assert!((0.0..1.0).contains(&roll));
            assert_eq!(roll, b.roll());
        }
        assert_ne!(Dice::new(5).roll(), Dice::new(6).roll());
    }

    #[test]
    fn what_each_carries_on_its_buttons_goes_by_who_it_is() {
        let place = |kind: Utility| KINDS.iter().position(|other| *other == kind).expect("one of the seven");
        let mut dice = Dice::new(11);
        let mut builders = std::collections::HashMap::new();
        let mut sticks = std::collections::HashMap::new();
        // Every kind there is to carry is one of them, each once: the
        // leap potion too, since 2026-11-15 (until then it was not theirs
        // to carry: none of them knew what to do with it).
        assert_eq!(KINDS.len(), Utility::ALL.len());
        assert!(Utility::ALL.iter().all(|kind| KINDS.contains(kind)));
        for persona in &PERSONAS {
            let [least, most] = persona.pockets;
            let (mut firsts, mut fullest) = ([0u32; KINDS.len()], 0);
            for _ in 0..600 {
                let loadout = persona.loadout(&mut dice);
                // As many kinds as its pockets have it, and no more than
                // there are buttons; none of them twice, none it has no
                // leaning to, some of each, and no more fireballs or
                // sticks of dynamite than a button holds.
                assert!((usize::from(least)..=usize::from(most)).contains(&loadout.len()), "{}: {loadout:?}", persona.name);
                assert!(loadout.len() <= crate::UTILITY_SLOTS, "{}: {loadout:?}", persona.name);
                for (turn, (kind, keeps)) in loadout.iter().enumerate() {
                    assert!(loadout[..turn].iter().all(|(other, _)| other != kind), "{}: {loadout:?}", persona.name);
                    assert!(persona.carries[place(*kind)] > 0.0, "{}: {loadout:?}", persona.name);
                    assert!(*keeps >= 1 && *keeps <= crate::bridge_wars::most_held(*kind).min(3), "{}: {loadout:?}", persona.name);
                    assert!(*keeps == 1 || matches!(kind, Utility::Fireball | Utility::Healing | Utility::Dynamite | Utility::Leap));
                }
                firsts[place(loadout[0].0)] += 1;
                fullest = fullest.max(loadout.len());
                *builders.entry(persona.name).or_insert(0u32) += u32::from(loadout.iter().any(|(kind, _)| *kind == Utility::Builder));
                *sticks.entry(persona.name).or_insert(0u32) += u32::from(loadout.iter().any(|(kind, _)| *kind == Utility::Dynamite));
            }
            // What it leans to most is what it mostly buys first (or as
            // good as, where it leans to two alike); and over enough
            // lives it fills as many buttons as it has pockets.
            let leaning_most = persona.carries.iter().copied().fold(0.0, f32::max);
            let most_often = (0..KINDS.len()).max_by_key(|kind| firsts[*kind]).unwrap();
            assert!(persona.carries[most_often] >= 0.75 * leaning_most, "{}: {firsts:?}", persona.name);
            assert_eq!(fullest, usize::from(most), "{}", persona.name);
        }
        // Some fill all five buttons, and some keep to one or two.
        assert!(PERSONAS.iter().any(|persona| persona.pockets[1] == 5));
        assert!(PERSONAS.iter().any(|persona| persona.pockets[1] <= 2));
        // The tinkerer has a Bridge Builder far oftener than the sentinel.
        assert!(builders["INTP"] > 3 * builders["ISTJ"], "{builders:?}");
        // And the trickster a stick of dynamite far oftener than the
        // captain; everyone has one now and then.
        assert!(sticks["ENTP"] > 3 * sticks["ENFJ"], "{sticks:?}");
        assert!(sticks.values().all(|lives| *lives > 0), "{sticks:?}");
        // The daredevil's fireballs come by the handful; the sentinel has
        // one, when it has any.
        let keeps = |name: &str, kind: Utility, seed: u64| {
            let mut dice = Dice::new(seed);
            (0..200).find_map(|_| persona(name).loadout(&mut dice).into_iter().find(|(carried, _)| *carried == kind)).map(|(_, keeps)| keeps)
        };
        assert_eq!(keeps("ESTP", Utility::Fireball, 3), Some(3));
        assert_eq!(keeps("ISTJ", Utility::Fireball, 3), Some(1));
        assert_eq!(keeps("ISFJ", Utility::Healing, 3), Some(2));
        // The defender never goes unseen: it has no leaning to that.
        assert_eq!(keeps("ISFJ", Utility::Invisibility, 3), None);
        // The free spirit, all over the map and jumping, has leap potions
        // by the pair; the sentinel, by the book, never one.
        assert_eq!(keeps("ENFP", Utility::Leap, 3), Some(2));
        assert_eq!(keeps("ISTJ", Utility::Leap, 3), None);
    }

    #[test]
    fn a_bot_buys_each_kind_it_carries_in_its_turn_and_a_builder_for_its_bridge() {
        let team = Upgrades::default();
        let armed = Kit {
            sword: 2,
            shield: 2,
            ..Kit::default()
        };
        let plain = plain();
        let carried = [(Utility::Fireball, 2), (Utility::Healing, 1), (Utility::Speed, 1), (Utility::Builder, 1)];
        let rich: Purse = [200, 20, 0, 5];
        let buy = |kit: &Kit, purse: &Purse, short: f32, laying: bool| {
            shopping(&plain, &carried, Role::Runner, false, 400.0, purse, kit, &team, short, laying, false)
        };
        let things = |list: &[Ware]| -> Vec<Utility> {
            list.iter()
                .filter_map(|ware| match ware {
                    Ware::Utility(kind) => Some(*kind),
                    _ => None,
                })
                .collect()
        };
        // Nothing on its buttons: each kind it carries, in its turn, and
        // no builder with no bridge to lay.
        assert_eq!(things(&buy(&armed, &rich, 0.0, false)), vec![Utility::Fireball, Utility::Healing, Utility::Speed]);
        // With one fireball, the second comes after the first of the rest.
        let one = armed.holding(Utility::Fireball, 1);
        assert_eq!(things(&buy(&one, &rich, 0.0, false)), vec![Utility::Healing, Utility::Speed, Utility::Fireball]);
        // As many of each as it keeps, and no more.
        let full = armed.holding(Utility::Fireball, 2).holding(Utility::Healing, 1).holding(Utility::Speed, 1);
        assert!(things(&buy(&full, &rich, 0.0, false)).is_empty());
        // Short of a bridge: a builder to lay it, with the amethyst for
        // one, in place of the metres; the metres without the amethyst,
        // part way along laying it itself, or with a builder in hand
        // that this bridge is not for.
        let short = buy(&armed, &rich, 44.6, false);
        assert_eq!(short[0], Ware::Utility(Utility::Builder), "{short:?}");
        assert!(!short.contains(&Ware::Bridge), "{short:?}");
        assert_eq!(&buy(&armed, &[200, 20, 0, 0], 44.6, false)[..2], &[Ware::Bridge, Ware::Bridge]);
        assert_eq!(buy(&armed, &rich, 20.0, true)[0], Ware::Bridge);
        assert_eq!(buy(&armed.holding(Utility::Builder, 1), &rich, 44.6, false)[0], Ware::Bridge);
        // One that carries none buys the metres, whatever amethyst it has.
        let none = shopping(&plain, &[(Utility::Healing, 1)], Role::Runner, false, 400.0, &rich, &armed, &team, 44.6, false, false);
        assert_eq!(&none[..2], &[Ware::Bridge, Ware::Bridge]);
        // A keeper with nobody to fetch it amethyst makes do with healing
        // potions in place of what amethyst buys, once.
        let home = shopping(&plain, &carried, Role::Keeper, false, 400.0, &[200, 20, 0, 0], &armed, &team, 0.0, false, false);
        assert_eq!(things(&home), vec![Utility::Fireball, Utility::Healing]);
        // With a bridge worth a builder to be had, one that carries them
        // buys one in its turn among the rest; with one in hand, with no
        // amethyst, or carrying none, it does not.
        let spare = |kit: &Kit, purse: &Purse, carried: &[(Utility, u8)]| {
            things(&shopping(&plain, carried, Role::Runner, false, 400.0, purse, kit, &team, 0.0, false, true))
        };
        assert_eq!(spare(&armed, &rich, &carried), vec![Utility::Fireball, Utility::Builder, Utility::Healing, Utility::Speed]);
        assert!(!spare(&armed.holding(Utility::Builder, 1), &rich, &carried).contains(&Utility::Builder));
        assert!(!spare(&armed, &[200, 20, 0, 0], &carried).contains(&Utility::Builder));
        assert!(!spare(&armed, &rich, &[(Utility::Healing, 1)]).contains(&Utility::Builder));
    }

    #[test]
    fn a_bot_with_a_builder_and_nothing_it_has_to_build_opens_a_way_its_team_could_do_without() {
        // Hajun's map, with the island at the bottom joined to the middle
        // and to the smaller island on its one side, as a team has them
        // a minute or two in.
        let mut nav = Nav::arena();
        let (home, middle) = (Region::Team(0), Region::Middle);
        let [first, other] = beside(0);
        let mut chain = 0;
        let mut lay = |nav: &mut Nav, from: Region, to: Region| {
            let span = nav.grid().span(from, to, 0.0).expect("a span between them");
            for index in 0..=span.planks() {
                nav.hear(0, chain, index as u16, span.rung(index));
            }
            chain += 1;
            assert!(nav.joined(from, to), "{from:?} to {to:?}");
            span
        };
        lay(&mut nav, home, middle);
        lay(&mut nav, middle, Region::Sapphire(first));
        let outfits = {
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            outfits
        };
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        let mut seen = vec![Seen::default(); ARENA_SEATS];
        for (seat, one) in seen.iter_mut().enumerate() {
            *one = Seen {
                at: Vec3::new(ARENA_ISLANDS[seat % ARENA_TEAMS].x, 0.25, ARENA_ISLANDS[seat % ARENA_TEAMS].y),
                alive: true,
                in_play: true,
                unseen: false,
                person: false,
                health: 1.0,
            };
        }
        let spare = |nav: &Nav, name: &str, elapsed: f32| {
            let mut squad = Squad::new(3, Skill::MODERATE);
            squad.seat(4, Vec3::new(0.0, 0.25, 130.0), 0.0);
            assert!(squad.cast(4, name));
            let bot = squad.bot(4).expect("seated").clone();
            let view = View {
                elapsed,
                nav,
                seats: &seen,
                purses: &purses,
                outfits: &outfits,
                mint: &mint,
                anchors: [120.0; ARENA_TEAMS],
                sticks: &[],
            };
            squad.spare_span(&bot, &view)
        };
        // Early on, whoever it is: the other of the two smaller islands
        // beside its own, from the middle. No side door before the raids.
        for name in ["ISFJ", "ESTP"] {
            let (from, to, span) = spare(&nav, name, 100.0).expect("a bridge worth a builder");
            assert_eq!((from, to), (middle, Region::Sapphire(other)), "{name}");
            assert!(span.length() <= BUILDER_METRES - 2.0);
        }
        // Once the raids are on, one that likes a raid opens a side door:
        // from the smaller island its team has reached onto the island of
        // the other team that island lies beside. One that does not goes
        // for the sapphire still.
        let raids_on = Skill::MODERATE.raid_from + 10.0;
        assert!(persona("ESTP").raids > 0.0 && persona("ISFJ").raids <= 0.0);
        let (from, to, door) = spare(&nav, "ESTP", raids_on).expect("a side door");
        let Region::Team(theirs) = to else {
            panic!("not onto a team's island: {to:?}");
        };
        assert_eq!(from, Region::Sapphire(first));
        assert!(theirs != 0 && beside(theirs).contains(&first));
        assert!(door.length() <= BUILDER_METRES - 2.0, "{} m", door.length());
        assert_eq!(spare(&nav, "ISFJ", raids_on).map(|(from, to, _)| (from, to)), Some((middle, Region::Sapphire(other))));
        // A builder's bridge over that water is one a body walks: laid
        // rung by rung it joins the two lands, though nothing else did.
        let mut alone = Nav::arena();
        for index in 0..=door.planks() {
            alone.hear(4, 0, index as u16, door.rung(index));
        }
        assert!(alone.joined(from, to), "the side door does not land");
        // With both smaller islands reached and no raids yet, there is
        // nothing worth a builder.
        lay(&mut nav, middle, Region::Sapphire(other));
        assert!(spare(&nav, "ISFJ", 100.0).is_none());
    }

    /// One frame after another of the bot in `seat` alone in `nav`, with
    /// its kit as `outfits` has it and its health `health`, until `secs`
    /// are up or `done` says so: what it is ordered to do is done as a
    /// game would do it (a builder off its buttons, its rungs onto the
    /// map, a potion drunk), and each frame's order is shown to `each`.
    #[allow(clippy::too_many_arguments)]
    fn alone(
        squad: &mut Squad,
        nav: &mut Nav,
        outfits: &mut Outfits,
        seat: usize,
        foe: Option<Vec3>,
        mut health: f32,
        from: f32,
        secs: f32,
        mut each: impl FnMut(f32, &Order, &Nav, &Bot) -> bool,
    ) {
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        let mut seen = vec![Seen::default(); ARENA_SEATS];
        if let Some(at) = foe {
            seen[0] = Seen {
                at,
                alive: true,
                in_play: true,
                unseen: false,
                person: true,
                health: 1.0,
            };
        }
        let dt = 1.0 / 60.0;
        let mut now = from;
        while now < from + secs {
            now += dt;
            seen[seat] = Seen {
                at: squad.bot(seat).expect("seated").body.at,
                alive: true,
                in_play: true,
                unseen: false,
                person: false,
                health,
            };
            let order = {
                let view = View {
                    elapsed: now,
                    nav,
                    seats: &seen,
                    purses: &purses,
                    outfits,
                    mint: &mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &[],
                };
                squad.step(seat, &view, dt)
            };
            if order.build && !outfits.spend(seat, Utility::Builder) {
                squad.unsend(seat);
            }
            for &(chain, index, at, _) in &order.rungs {
                nav.hear(seat as u8, chain, index, at);
            }
            if let Some(potion) = order.drink
                && outfits.drink(seat, potion, now) == Some(Utility::Healing)
            {
                health = (health + crate::bridge_wars::HEALING).min(1.0);
            }
            if each(now, &order, nav, squad.bot(seat).expect("seated")) {
                break;
            }
        }
    }

    #[test]
    fn a_bridge_builder_sent_off_lays_its_bridge_while_the_bot_stays_ashore() {
        let (mut nav, span) = two_islands();
        let mut squad = Squad::new(5, Skill::MODERATE);
        let seat = 5;
        squad.seat(seat, Vec3::new(0.0, 0.25, 0.0), 0.0);
        assert!(squad.cast(seat, "ISTJ"));
        squad.set_plan(1, Plan::PLAIN);
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        outfits.kits[seat] = Kit::default().holding(Utility::Builder, 1);
        let (mut sent, mut joined, mut rungs, mut carried, mut furthest) = (None, None, Vec::new(), false, 0.0f32);
        let mut walk: Vec<(Vec2, Vec2)> = Vec::new();
        alone(&mut squad, &mut nav, &mut outfits, seat, None, 1.0, 1.0, 40.0, |now, order, nav, bot| {
            if order.build {
                sent = Some(now);
            }
            walk.extend(bot.builder_at());
            carried |= order.carrying;
            rungs.extend(order.rungs.iter().map(|rung| (rung.1, rung.2)));
            furthest = furthest.max(bot.body.at.x);
            if sent.is_some() && nav.joined(Region::Team(1), Region::Middle) {
                joined = Some(now);
            }
            joined.is_some()
        });
        let (sent, joined) = (sent.expect("it sent its builder off"), joined.expect("its bridge reached the far land"));
        // Every rung of one straight bridge, in its order, from its own
        // shore to the far one, a little to one side of the line between
        // the two lands' middles or on it, as any bridge of a bot's is.
        let (first, last) = (rungs[0].1, rungs[rungs.len() - 1].1);
        let way = (last - first).normalize();
        assert!(rungs.len() >= 10 && rungs.len() <= span.planks() + 3, "{} rungs", rungs.len());
        for (index, (rung, at)) in rungs.iter().enumerate() {
            assert_eq!(usize::from(*rung), index);
            assert!(way.perp_dot(*at - first).abs() < 1e-3, "rung {index} is off the line");
            assert!(index == 0 || at.distance(rungs[index - 1].1) <= crate::nav::PLANK_SPACING + 1e-3);
        }
        assert!(first.x < 6.0 && last.x > 24.0 && first.y.abs() < 5.0 && (first.y - last.y).abs() < 0.5, "{first} to {last}");
        // The void between them at the pace anyone builds at, and no
        // quicker: the bridge goes up a rung at a time.
        let across = first.distance(last) / BUILDER_SPEED;
        assert!(joined - sent < across + 1.0 && joined - sent > across * 0.5, "{} s for {} m", joined - sent, first.distance(last));
        // The builder itself is to be seen walking it, for whoever runs
        // the game to show: from where the bridge begins, along its line
        // and the way it goes, never back, and never past the planks laid
        // ahead of it.
        assert!(walk.len() > 60, "seen for {} frames", walk.len());
        assert!(walk[0].0.distance(first) < 0.5, "it sets off from {}", walk[0].0);
        for (at, heading) in &walk {
            assert!(heading.distance(way) < 1e-3 && way.perp_dot(*at - first).abs() < 1e-3, "{at}, heading {heading}");
        }
        assert!(walk.windows(2).all(|pair| (pair[1].0 - pair[0].0).dot(way) >= -1e-4));
        let far = (walk[walk.len() - 1].0 - first).dot(way);
        assert!(far > first.distance(last) * 0.6 && far <= first.distance(last) + 1e-3, "{far} m along");
        // And nothing of it the bot's own: no plank in its arms, no
        // metres spent, the builder gone from its button, and its feet on
        // its own island.
        assert!(!carried);
        let bot = squad.bot(seat).expect("seated");
        assert_eq!(bot.metres(), 0.0);
        assert!(!bot.building());
        assert_eq!(outfits.kit(seat).held(Utility::Builder), 0);
        assert!(furthest < 6.01, "it walked out to {furthest}");
        // And it lays no second bridge beside the one being built: with
        // the two joined there is none to lay.
        assert!(nav.joined(Region::Team(1), Region::Middle));
    }

    #[test]
    fn it_drinks_the_potion_the_moment_calls_for_of_those_it_carries() {
        // Late in a game, when everyone hunts everyone, with someone to
        // go for on the far land, and a speed potion, a healing potion
        // and an invisibility potion on its buttons.
        let drunk = |health: f32| {
            let mut nav = bridged();
            let mut squad = Squad::new(9, Skill::MODERATE);
            let seat = 5;
            squad.seat(seat, Vec3::new(0.0, 0.25, 0.0), 0.0);
            assert!(squad.cast(seat, "ISTJ"));
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            outfits.kits[seat] = Kit::default()
                .holding(Utility::Invisibility, 1)
                .holding(Utility::Speed, 1)
                .holding(Utility::Healing, 1);
            let mut drinks = Vec::new();
            let foe = Some(Vec3::new(32.0, 0.25, 0.0));
            let from = Skill::MODERATE.hunt_from + 5.0;
            alone(&mut squad, &mut nav, &mut outfits, seat, foe, health, from, 4.5, |_, order, _, _| {
                drinks.extend(order.drink);
                false
            });
            (drinks, outfits.kit(seat))
        };
        // Whole: the speed potion for the chase, and nothing else. An
        // invisibility potion is for walking onto an island it raids.
        let (drinks, kit) = drunk(1.0);
        assert_eq!(drinks, vec![Utility::Speed]);
        assert_eq!((kit.held(Utility::Healing), kit.held(Utility::Invisibility)), (1, 1));
        // Hurt: the healing potion first, and then the speed potion.
        let (drinks, kit) = drunk(0.45);
        assert_eq!(drinks, vec![Utility::Healing, Utility::Speed]);
        assert_eq!(kit.held(Utility::Invisibility), 1);
    }

    /// A round island `radius` across from its middle at the origin, its
    /// top 0.25 m up, `Team(1)`: room to run round and round on.
    fn round_island(radius: f32) -> Nav {
        let side = ((radius + 4.0) * 4.0) as usize;
        let mut grid = Grid::blank(side, side, Vec2::splat(-(radius + 4.0)));
        for row in 0..side {
            for column in 0..side {
                let away = grid.centre_of(column, row).length();
                if away <= radius {
                    grid.set(column, row, Some(0.25), away <= radius - 0.5, Some(Region::Team(1)));
                }
            }
        }
        grid.place_land(Region::Team(1), Vec2::ZERO);
        Nav::new(Arc::new(grid))
    }

    /// One of the computer's players, in seat 5, with someone of another
    /// team in seat 0 and, if `mate` says where, one of its own in seat 1,
    /// on a map of their own, a frame at a time: where the someone is each
    /// frame is the caller's to say, and so is every stick of dynamite
    /// there is; the bot's rungs go onto the map as a game would lay them.
    struct Rig {
        nav: Nav,
        squad: Squad,
        outfits: Outfits,
        mint: Mint,
        seat: usize,
        now: f32,
        sticks: Vec<Stick>,
        mate: Option<Vec2>,
        rungs: usize,
    }

    impl Rig {
        const FRAME: f32 = 1.0 / 60.0;

        /// `who` standing `at` on `nav`, `now` seconds into a fight.
        fn new(nav: Nav, seed: u64, who: &str, at: Vec2, now: f32) -> Self {
            let mut squad = Squad::new(seed, Skill::MODERATE);
            let seat = 5;
            squad.seat(seat, Vec3::new(at.x, 0.25, at.y), 0.0);
            assert!(squad.cast(seat, who), "nobody is called {who}");
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            Self {
                nav,
                squad,
                outfits,
                mint: Mint::new(0.0, &mut || 0.5),
                seat,
                now,
                sticks: Vec::new(),
                mate: None,
                rungs: 0,
            }
        }

        /// Late in a game, when everyone goes for everyone they can get at.
        fn hunting(nav: Nav, seed: u64, who: &str, at: Vec2) -> Self {
            Self::new(nav, seed, who, at, Skill::MODERATE.hunt_from + 5.0)
        }

        fn bot(&self) -> &Bot {
            self.squad.bot(self.seat).expect("seated")
        }

        fn at(&self) -> Vec2 {
            self.bot().body.at.xz()
        }

        /// Whether the bot has fallen off everything.
        fn fallen(&self) -> bool {
            self.bot().body.at.y < -2.0
        }

        /// One frame, with the someone at `person` if they are there: what
        /// the bot is ordered to do.
        fn step(&mut self, person: Option<Vec2>) -> Order {
            self.now += Self::FRAME;
            let standing = |at: Vec2, person: bool| Seen {
                at: Vec3::new(at.x, 0.25, at.y),
                alive: true,
                in_play: true,
                unseen: false,
                person,
                health: 1.0,
            };
            let mut seen = vec![Seen::default(); ARENA_SEATS];
            if let Some(at) = person {
                seen[0] = standing(at, true);
            }
            if let Some(at) = self.mate {
                seen[1] = standing(at, false);
            }
            seen[self.seat] = Seen {
                at: self.bot().body.at,
                ..standing(Vec2::ZERO, false)
            };
            let purses = [[0u16; 4]; ARENA_SEATS];
            let order = {
                let view = View {
                    elapsed: self.now,
                    nav: &self.nav,
                    seats: &seen,
                    purses: &purses,
                    outfits: &self.outfits,
                    mint: &self.mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &self.sticks,
                };
                self.squad.step(self.seat, &view, Self::FRAME)
            };
            for &(chain, index, at, _) in &order.rungs {
                self.nav.hear(self.seat as u8, chain, index, at);
                self.rungs += 1;
            }
            for stick in &mut self.sticks {
                stick.left -= Self::FRAME;
            }
            order
        }
    }

    #[test]
    fn a_body_jumping_a_gap_goes_off_the_edge_it_would_never_walk_off_and_comes_down_past_it() {
        // A bridge with four planks blown out of it, 5.9 m of nothing: a
        // blast where two planks meet takes the two either side of it,
        // as one anywhere but on the very middle of a plank does.
        let mut nav = bridged();
        let span = two_islands().1;
        let taken = nav.blast(span.rung(6), 3.0);
        assert_eq!(taken.len(), 4, "{taken:?}");
        let gap = nav.gaps()[0];
        assert!(gap.length() < Squad::reach_of_a_jump(RUN_SPEED, false), "{} m", gap.length());
        let frame = 1.0 / 60.0;
        let run = (gap.b_in - gap.a_in).normalize() * RUN_SPEED;
        // Walked at it, a body stops at the end of what stands.
        let mut walker = Body::standing(Vec3::new(gap.a_in.x, PLANK, gap.a_in.y), 0.0);
        for _ in 0..90 {
            walker.step(&nav, run, frame);
        }
        assert!(walker.free(&nav) && walker.at.x <= gap.a.x + 0.1, "{}", walker.at);
        // Jumping it, it leaves the ground at the end of what stands, and
        // is down again on the far side well inside what stands there.
        let mut jumper = Body::standing(Vec3::new(gap.a_in.x, PLANK, gap.a_in.y), 0.0);
        jumper.leap = true;
        let (mut left_at, mut highest, mut down_at) = (None, 0.0f32, None);
        for _ in 0..90 {
            let before = jumper;
            jumper.step(&nav, run, frame);
            if left_at.is_none() && jumper.lift > 0.0 {
                left_at = Some(before.at.x);
            }
            highest = highest.max(jumper.at.y);
            if left_at.is_some() && down_at.is_none() && jumper.free(&nav) {
                down_at = Some(jumper.at.x);
                break;
            }
        }
        let (left_at, down_at) = (left_at.expect("it jumped"), down_at.expect("it came down"));
        assert!((left_at - gap.a.x).abs() < 0.3, "it left the ground at {left_at}, the edge being at {}", gap.a.x);
        assert!(highest > 1.6, "as high as anyone jumps: {highest}");
        // With what a bot keeps in hand ([`JUMP_SPARE`]), and not much
        // over.
        assert!(down_at - gap.b.x >= JUMP_SPARE, "down {} m past the far edge", down_at - gap.b.x);
        assert!(Squad::reach_of_a_jump(RUN_SPEED, false) - gap.length() < 0.5);
        // Five planks gone, 7.3 m, as a blast on the very middle of a
        // plank takes, a body at a run clears too, by a hand: a bot jumps
        // no gap it has not a good deal in hand over, and this is not one
        // it jumps.
        let mut wide = bridged();
        assert_eq!(wide.blast((span.rung(6) + span.rung(7)) * 0.5, 3.0).len(), 5);
        let hole = wide.gaps()[0];
        assert!(hole.length() > Squad::reach_of_a_jump(RUN_SPEED, false) + 1.0, "{} m", hole.length());
        let mut close = Body::standing(Vec3::new(hole.a_in.x, PLANK, hole.a_in.y), 0.0);
        close.leap = true;
        let mut down_at = None;
        for _ in 0..90 {
            close.step(&wide, run, frame);
            if close.at.x > hole.a.x + 1.0 && close.free(&wide) {
                down_at = Some(close.at.x);
                break;
            }
        }
        let down_at = down_at.expect("it clears it");
        assert!(down_at - hole.b.x < JUMP_SPARE * 0.5, "it came down {} m past the far edge", down_at - hole.b.x);
        // Six planks gone is further than a jump goes.
        let mut wider = bridged();
        assert_eq!(wider.blast(span.rung(7), 3.7).len(), 6);
        let hole = wider.gaps()[0];
        let mut gone = Body::standing(Vec3::new(hole.a_in.x, PLANK, hole.a_in.y), 0.0);
        gone.leap = true;
        for _ in 0..180 {
            gone.step(&wider, run, frame);
        }
        assert!(gone.at.y < -10.0, "{}", gone.at);
    }

    /// How high a plank's top is: where a body stands on a bridge.
    const PLANK: f32 = crate::nav::PLANK_TOP;

    /// How many punches a bot called `who` lands in `secs` on someone who
    /// runs round and round a ring `radius` across from the middle of an
    /// island and never hits back, and how near it gets to them at its
    /// nearest.
    fn punches_on_a_runner(who: &str, seed: u64, radius: f32, secs: f32) -> (u32, f32) {
        let mut rig = Rig::hunting(round_island(10.0), seed, who, Vec2::new(-7.0, 0.5));
        let (mut punches, mut nearest) = (0, f32::INFINITY);
        for frame in 0..(secs / Rig::FRAME) as usize {
            let angle = frame as f32 * Rig::FRAME * RUN_SPEED / radius;
            let person = Vec2::from_angle(angle) * radius;
            let order = rig.step(Some(person));
            let away = rig.at().distance(person);
            nearest = nearest.min(away);
            punches += u32::from(order.punch == Some(0) && away <= REACH + BODY + 0.5);
        }
        assert!(!rig.fallen());
        (punches, nearest)
    }

    #[test]
    fn someone_running_round_and_round_is_cut_off_and_not_followed_for_ever() {
        // Round a ring 3.5 m across from its middle and round one 6 m, at
        // a run, for ten seconds: every kind of bot lands punches on them,
        // a few seeds of each together, where one that ran at where they
        // were would follow them round for ever a step behind.
        for radius in [3.5, 6.0] {
            for persona in &PERSONAS {
                let landed: u32 = (0..4).map(|seed| punches_on_a_runner(persona.name, seed, radius, 10.0).0).sum();
                assert!(landed >= 12, "{} landed {landed} punches in four tries on someone running a ring of {radius} m", persona.name);
            }
        }
        // And the best of them land the most.
        let of = |who: &str| (0..6).map(|seed| punches_on_a_runner(who, seed, 3.5, 10.0).0).sum::<u32>();
        let (best, plain) = (of("ISTP"), of("INFP"));
        assert!(best > plain, "the lone wolf landed {best}, the dreamer {plain}");
    }

    #[test]
    fn the_better_a_bot_is_in_a_fight_the_faster_its_punches_come_and_the_more_of_them_on_the_run() {
        // Someone standing still on an island, never hitting back, for
        // twelve seconds: how many punches, and how many thrown running.
        let fight = |who: &str, seed: u64| {
            let mut rig = Rig::hunting(round_island(10.0), seed, who, Vec2::new(-6.0, 0.0));
            let (mut punches, mut running) = (0u32, 0u32);
            for _ in 0..(12.0 / Rig::FRAME) as usize {
                let order = rig.step(Some(Vec2::ZERO));
                if order.punch == Some(0) {
                    punches += 1;
                    running += u32::from(order.running);
                }
            }
            (punches, running)
        };
        let sum = |who: &str| (0..5).map(|seed| fight(who, seed)).fold((0, 0), |all, one| (all.0 + one.0, all.1 + one.1));
        let (best, plain) = (sum("ISTP"), sum("INFP"));
        assert!(persona("ISTP").pvp > 0.9 && persona("INFP").pvp < 0.1);
        // The dreamer as they all were: a punch every 0.8 s or so, once it
        // is there. The lone wolf nearly as fast as punches count, two a
        // second, and most of them thrown running in.
        assert!((45..=75).contains(&plain.0), "the dreamer threw {} in a minute", plain.0);
        assert!(best.0 as f32 > plain.0 as f32 * 1.3, "the lone wolf threw {} against {}", best.0, plain.0);
        assert!(best.1 as f32 > 0.6 * best.0 as f32, "{} of its {} on the run", best.1, best.0);
        assert!((best.1 as f32 / best.0 as f32) > (plain.1 as f32 / plain.0 as f32) + 0.2, "{best:?} against {plain:?}");
        // Every kind is somewhere between, by how good it is.
        assert!(PERSONAS.iter().all(|persona| (0.0..=1.0).contains(&persona.pvp)));
        assert!(PERSONAS.iter().filter(|persona| persona.pvp >= 0.6).count() >= 4, "some are good at it");
        assert!(PERSONAS.iter().filter(|persona| persona.pvp <= 0.25).count() >= 4, "and some are as they all were");
    }

    /// The two islands with two bridges between them, side by side, 7.8 m
    /// from the middle of the one to the middle of the other: 4.4 m of
    /// nothing between their edges. And how far to each side of the line
    /// between the islands' middles they lie.
    fn two_bridges() -> (Nav, f32) {
        let (mut nav, _) = two_islands();
        let aside = 3.9;
        for (chain, side) in [aside, -aside].into_iter().enumerate() {
            let span = nav.grid().span(Region::Team(1), Region::Middle, side).expect("a span to one side");
            for index in 0..=span.planks() {
                nav.hear(1, chain as u16, index as u16, span.rung(index));
            }
        }
        (nav, aside)
    }

    #[test]
    fn a_bot_jumps_from_one_bridge_to_the_next_after_someone_and_back_after_them() {
        let (nav, aside) = two_bridges();
        let across = 2.0 * aside - 2.0 * crate::nav::PLANK_HALF_WIDTH;
        assert!(across > 4.0 && across < Squad::reach_of_a_jump(RUN_SPEED, false));
        // Halfway out on the one bridge, with someone halfway out on the
        // other: 7 m off as the crow flies, and 30 m round by the island.
        let (mut jumped, mut went_round) = (0, 0);
        for seed in 0..6 {
            let mut rig = Rig::hunting(nav.clone(), seed, "ISTP", Vec2::new(15.0, aside));
            let mut side = -aside;
            let mut crossings = 0;
            for _ in 0..(12.0 / Rig::FRAME) as usize {
                rig.step(Some(Vec2::new(15.0, side)));
                assert!(!rig.fallen(), "seed {seed}: it fell");
                let at = rig.at();
                // On their bridge and down, having come over the gap and
                // not round by an island: they jump back to the other.
                if (at.y - side).abs() < 1.2 && rig.bot().body.free(&rig.nav) {
                    crossings += 1;
                    side = -side;
                }
                // Back on an island: it went round. It is put back out on
                // the bridge across from them, to try again.
                if rig.nav.grid().height(at).is_some() {
                    went_round += 1;
                    let now = rig.now;
                    rig.squad.stand(rig.seat, Vec3::new(15.0, PLANK, -side), 0.0, now);
                }
            }
            jumped += crossings;
        }
        // Over and back and over again, nearly every time: now and then
        // it thinks better of it and goes round.
        assert!(jumped >= 30, "{jumped} jumps in six tries of twelve seconds");
        assert!(went_round * 5 <= jumped, "round by the island {went_round} times, over the gap {jumped}");
    }

    #[test]
    fn a_bot_gets_out_from_under_a_stick_of_dynamite_and_waits_short_of_one_on_its_way() {
        let span = two_islands().1;
        let middle = span.rung(7);
        // Standing on a bridge, a stick comes down beside it: the quick
        // ones are clear of the blast by the time it blows, on what still
        // stands, whichever team threw it.
        for (who, team) in [("ISTP", 0), ("ESTP", 1), ("ENTJ", 0)] {
            for seed in 0..4 {
                let mut rig = Rig::new(bridged(), seed, who, middle, 200.0);
                rig.sticks.push(Stick {
                    at: middle + Vec2::X * 0.8,
                    left: 0.6 + DYNAMITE_FUSE,
                    team,
                });
                while rig.sticks[0].left > 0.0 {
                    rig.step(None);
                }
                let stick = rig.sticks.remove(0);
                let away = rig.at().distance(stick.at);
                assert!(away > DYNAMITE_RADIUS + BODY, "{who}, seed {seed}: {away} m from it as it blew");
                rig.nav.blast(stick.at, DYNAMITE_RADIUS);
                for _ in 0..60 {
                    rig.step(None);
                }
                assert!(!rig.fallen() && rig.nav.ground(rig.at()).is_some(), "{who}, seed {seed}: it went with the planks");
            }
        }
        // On the land, one of its own team's is nothing to it: it stays
        // where it is. Another team's it gets away from.
        for (team, moves) in [(1, false), (0, true)] {
            let mut rig = Rig::new(bridged(), 3, "ISTP", Vec2::new(1.0, 1.0), 200.0);
            let from = rig.at();
            rig.sticks.push(Stick {
                at: from + Vec2::new(0.8, 0.0),
                left: 1.6,
                team,
            });
            let mut furthest = 0.0f32;
            while rig.sticks[0].left > 0.0 {
                rig.step(None);
                furthest = furthest.max(rig.at().distance(rig.sticks[0].at));
            }
            assert_eq!(furthest > DYNAMITE_RADIUS + BODY, moves, "a stick of team {team}'s: {furthest} m at the furthest");
        }
        // How soon one is out of a blast goes by how good it is at a fight.
        // A stick of another team's thrown from close by to come down right
        // where it stands, in the open: the best of them is clear of it
        // every time, and the worst is still in it now and then, more
        // often than not when it comes down in the middle of a fight with
        // someone.
        let caught = |who: &str, fighting: bool| -> usize {
            (0..80)
                .filter(|&seed| {
                    let at = Vec2::new(-2.0, 1.0);
                    let mut rig = if fighting {
                        Rig::hunting(bridged(), seed, who, at)
                    } else {
                        Rig::new(bridged(), seed, who, at, 200.0)
                    };
                    let foe = fighting.then_some(at + Vec2::new(0.0, 1.6));
                    // (A moment of it first, to have the fight begun.)
                    for _ in 0..30 {
                        rig.step(foe);
                    }
                    rig.sticks.push(Stick {
                        at: rig.at(),
                        left: 0.55 + DYNAMITE_FUSE,
                        team: 0,
                    });
                    while rig.sticks[0].left > 0.0 {
                        rig.step(foe);
                    }
                    rig.at().distance(rig.sticks[0].at) <= DYNAMITE_RADIUS + BODY
                })
                .count()
        };
        assert_eq!((caught("ISTP", false), caught("ISTP", true)), (0, 0));
        assert_eq!(caught("ESTJ", false), 0);
        let (worst, worst_fighting) = (caught("INFP", false), caught("INFP", true));
        assert!((12..=48).contains(&worst), "the worst of them was caught {worst} times in eighty");
        assert!(worst_fighting > worst && worst_fighting >= 40, "and {worst_fighting} times in a fight");
        let middling = caught("ENFJ", true);
        assert!(middling > 0 && middling < worst_fighting, "one in the middle, in a fight: {middling} times in eighty");
        // (As it stood on 2026-11-13, of eighty, in the open and in a
        // fight: ISTP, ESTP, ENTJ and ESTJ never; ENTP 0 and 1, INTJ 0 and
        // 4, ESFP 0 and 1, ENFP 1 and 10, ENFJ 0 and 14, ISTJ 2 and 15,
        // INFJ 5 and 14, ISFP 12 and 30, INTP 12 and 36, ESFJ 14 and 31,
        // ISFJ 17 and 49, INFP 34 and 55.)
        // After someone on the far land, with a stick lying on the bridge
        // between: it waits short of the blast, and does not run into it.
        let far = Vec2::new(30.0, 0.5);
        let mut rig = Rig::hunting(bridged(), 2, "ENTJ", Vec2::new(4.0, 0.0));
        rig.sticks.push(Stick {
            at: span.rung(8),
            left: 0.7 + DYNAMITE_FUSE,
            team: 0,
        });
        while rig.sticks[0].left > 0.0 {
            rig.step(Some(far));
            assert!(rig.at().distance(rig.sticks[0].at) > DYNAMITE_RADIUS, "it ran into the blast: {}", rig.at());
        }
        assert!(rig.at().x < span.rung(8).x, "and it is on its own side of it");
    }

    /// Someone comes at the base of `who` and steps back onto their
    /// bridge, five times over: three seconds on its island, seven on the
    /// bridge three and a half metres out. How patient the bot was thrown;
    /// at which coming it first stood its ground; and whether, having not
    /// yet had enough of them, it ever went back to what it was doing with
    /// them still there.
    fn baited(who: &str, seed: u64) -> (u8, Option<u8>, bool) {
        let mut rig = Rig::new(bridged(), seed, who, Vec2::new(-3.0, 0.5), 200.0);
        let patience = rig.bot().patience();
        let (on, off) = (Vec2::new(3.5, 0.0), Vec2::new(9.5, 0.0));
        let mut went_back = false;
        for coming in 1..=5u8 {
            for _ in 0..180 {
                rig.step(Some(on));
                assert!(!rig.bot().stands_its_ground(), "they are on its island: it is at them");
            }
            assert_eq!(rig.bot().vigil().map(|vigil| vigil.comings), Some(coming), "{who}, seed {seed}");
            let mut stood = false;
            for _ in 0..420 {
                rig.step(Some(off));
                assert!(!rig.fallen(), "{who}, seed {seed}: it went off its island after them");
                stood |= rig.bot().stands_its_ground();
                went_back |= !stood && !matches!(rig.bot().goal(), Goal::Guard | Goal::Defend);
            }
            if stood {
                return (patience, Some(coming), went_back);
            }
        }
        (patience, None, went_back)
    }

    #[test]
    fn someone_who_keeps_coming_at_a_bot_s_base_is_stood_against_sooner_or_later_by_who_it_is() {
        // Where it stands: on its island, between what it keeps and them,
        // so far from them and so far in from where their bridge lands.
        let nav = bridged();
        let spot = guard_spot(&nav, Region::Team(1), Vec2::ZERO, Vec2::new(9.5, 0.0), 3.5, 1.0).expect("a place to stand");
        assert!((spot.x - 4.5).abs() < 0.35 && spot.y.abs() < 1e-3, "{spot}");
        assert_eq!(guard_spot(&nav, Region::Team(1), Vec2::ZERO, Vec2::new(0.6, 0.0), 3.5, 1.0), None);
        // Each of them stands its ground at the coming it was thrown the
        // patience for, the first, the second or the third, and not
        // before: until then it looks after them a moment and goes back
        // to what it was doing. Nobody's patience is the same every time,
        // and one kind is not as patient as another.
        let mut mean = std::collections::HashMap::new();
        for persona in PERSONAS.iter() {
            let mut stood_at = [0u32; 4];
            for seed in 0..24 {
                let (patience, stood, went_back) = baited(persona.name, seed);
                assert!((1..=3).contains(&patience));
                assert_eq!(stood, Some(patience), "{}, seed {seed}", persona.name);
                assert_eq!(went_back, patience > 1, "{}, seed {seed}: patient for {patience}", persona.name);
                stood_at[usize::from(patience)] += 1;
            }
            assert!(stood_at.iter().filter(|times| **times > 0).count() >= 2, "{}: always at the same coming, {stood_at:?}", persona.name);
            let comings = (stood_at[1] + 2 * stood_at[2] + 3 * stood_at[3]) as f32 / 24.0;
            assert!((comings - persona.patience).abs() < 0.5, "{}: at {comings} on average, for {}", persona.name, persona.patience);
            // (As it stood on 2026-11-14, of twenty-four: the defender at
            // the first coming 21 times and the second 3; the mastermind
            // 2, 21 and 1 at the first, the second and the third; the
            // free spirit at the second 8 times and the third 16.)
            mean.insert(persona.name, comings);
        }
        assert!(mean["ISFJ"] + 1.0 < mean["INFP"] && mean["INFJ"] + 1.0 < mean["ENFP"], "{mean:?}");
        assert!(PERSONAS.iter().all(|persona| (1.0..=3.0).contains(&persona.patience)));
    }

    #[test]
    fn a_bot_standing_its_ground_keeps_between_them_and_its_base_until_they_are_really_gone() {
        let (on, off) = (Vec2::new(3.5, 0.0), Vec2::new(9.5, 0.0));
        for (who, seed) in [("ISFJ", 1), ("INFJ", 4), ("ISTJ", 2), ("ESTJ", 7), ("ISFJ", 9)] {
            let mut rig = Rig::new(bridged(), seed, who, Vec2::new(-3.0, 0.5), 200.0);
            // Baited until it stands.
            let mut comings = 0;
            while !rig.bot().stands_its_ground() {
                comings += 1;
                assert!(comings <= 3, "{who}, seed {seed}");
                for _ in 0..180 {
                    rig.step(Some(on));
                }
                for _ in 0..120 {
                    rig.step(Some(off));
                }
            }
            // With them a good way down their bridge, further than it
            // goes at anyone from where it stands and nearer than gone,
            // it is on its island on their side of what it keeps, turned
            // to them; and it does not go back to anything else.
            let down = Vec2::new(22.5, 0.0);
            for frame in 0..(6.0 / Rig::FRAME) as usize {
                rig.step(Some(down));
                if frame > 150 && rig.bot().stands_its_ground() && rig.bot().foe.is_none() {
                    let at = rig.at();
                    assert!(at.x > 0.8 && at.x < 5.6 && at.y.abs() < 2.5, "{who}, seed {seed}: it stands at {at}");
                    assert_eq!(rig.bot().goal(), Goal::Guard);
                }
            }
            // They come back onto its island: it is at them at once, and
            // it has counted another coming.
            let mut struck = false;
            for _ in 0..(4.0 / Rig::FRAME) as usize {
                struck |= rig.step(Some(on)).punch == Some(0);
            }
            assert!(struck, "{who}, seed {seed}: they came back and it did nothing");
            assert_eq!(rig.bot().vigil().map(|vigil| vigil.comings), Some(comings + 1));
            // They step back, and it stands again, having had enough long
            // since; they go for good, and within the while it takes to
            // believe it, it stands there no longer. It has not forgotten
            // them: the next time they come and step back, it stands at
            // once, however patient it was thrown.
            for _ in 0..120 {
                rig.step(Some(off));
            }
            assert!(rig.bot().stands_its_ground(), "{who}, seed {seed}");
            let mut stood_down = None;
            for frame in 0..(12.0 / Rig::FRAME) as usize {
                rig.step(None);
                if !rig.bot().stands_its_ground() {
                    stood_down = Some(frame as f32 * Rig::FRAME);
                    break;
                }
            }
            let after = stood_down.expect("it never stood down");
            assert!(after > 1.0 && after < VIGIL_CALM.1 + 2.0, "{who}, seed {seed}: after {after} s");
            for _ in 0..180 {
                rig.step(Some(on));
            }
            for _ in 0..120 {
                rig.step(Some(off));
            }
            assert!(rig.bot().stands_its_ground(), "{who}, seed {seed}: it had forgotten them");
            // Out on their bridge near enough for it to go at them, and
            // then away to the far land: it goes no further after them
            // than it means to from where it stands, and is on its own
            // island again, whatever it was doing to them on the bridge.
            for _ in 0..(5.0 / Rig::FRAME) as usize {
                rig.step(Some(off));
                assert!(!rig.fallen(), "{who}, seed {seed}");
            }
            for _ in 0..(10.0 / Rig::FRAME) as usize {
                rig.step(Some(Vec2::new(28.0, 0.0)));
                assert!(!rig.fallen(), "{who}, seed {seed}");
            }
            assert!(rig.at().x < 6.3, "{who}, seed {seed}: it is after them still, at {}", rig.at());
        }
    }

    #[test]
    fn a_bot_does_not_stand_its_ground_for_ever_and_what_it_does_then_is_not_the_same_twice() {
        // Someone who has had a bot stand its ground, and then stays out
        // on their bridge where it does not go at them from where it
        // stands: for a minute and a half. Sooner or later it has stood
        // there long enough, and either goes out at them or lets them be
        // and goes back to what it was doing; which, and when, differs
        // from one time to the next, and the good at a fight go out at
        // them oftener than the rest.
        let (on, off, down) = (Vec2::new(3.5, 0.0), Vec2::new(9.5, 0.0), Vec2::new(22.5, 0.0));
        let outcomes = |who: &str| -> (u32, u32, Vec<f32>) {
            let (mut went_out, mut let_be, mut whens) = (0, 0, Vec::new());
            for seed in 0..30 {
                let mut rig = Rig::new(bridged(), seed, who, Vec2::new(-3.0, 0.5), 200.0);
                for _ in 0..4 {
                    for _ in 0..180 {
                        rig.step(Some(on));
                    }
                    for _ in 0..90 {
                        rig.step(Some(off));
                    }
                    if rig.bot().stands_its_ground() {
                        break;
                    }
                }
                assert!(rig.bot().stands_its_ground(), "{who}, seed {seed}");
                let (mut out, mut back) = (false, false);
                for frame in 0..(90.0 / Rig::FRAME) as usize {
                    rig.step(Some(down));
                    assert!(!rig.fallen(), "{who}, seed {seed}");
                    // Out on the bridge after them; or about its island
                    // again with nobody in mind to stand against.
                    let now_out = rig.at().x > 8.0;
                    let now_back = !rig.bot().stands_its_ground() && rig.bot().goal() != Goal::Guard;
                    if (now_out && !out) || (now_back && !back) {
                        whens.push(frame as f32 * Rig::FRAME);
                    }
                    out |= now_out;
                    back |= now_back;
                }
                assert!(out || back, "{who}, seed {seed}: it stood there for a minute and a half");
                went_out += u32::from(out);
                let_be += u32::from(back && !out);
            }
            (went_out, let_be, whens)
        };
        let (bold_out, bold_be, whens) = outcomes("ISTP");
        let (meek_out, meek_be, _) = outcomes("INFP");
        // (As it stood on 2026-11-14, of thirty: the lone wolf out at them
        // 19 times and letting them be 11, the dreamer 4 and 26.)
        assert!(bold_out > 0 && bold_be > 0, "the lone wolf: out at them {bold_out} times, let them be {bold_be}");
        assert!(meek_out > 0 && meek_be > 0, "the dreamer: out at them {meek_out} times, let them be {meek_be}");
        assert!(bold_out > meek_out, "{bold_out} against {meek_out}");
        // And not on the stroke of anything: the first thing it did came
        // anywhere from a quarter of a minute on to well over half of one.
        let (soonest, latest) = whens.iter().fold((f32::INFINITY, 0.0f32), |(low, high), when| (low.min(*when), high.max(*when)));
        assert!(soonest < 20.0 && latest > 30.0, "between {soonest} and {latest} s");
    }

    /// A floor 20 m square about the origin, with a wall across it and
    /// down it, an L whose corner is at the origin: along z = 0 from x =
    /// -5 to 0, and along x = 0 from z = 0 to 5.
    fn walled() -> Nav {
        let mut grid = Grid::blank(60, 60, Vec2::new(-15.0, -15.0));
        for row in 10..50 {
            for column in 10..50 {
                let wall = (row == 30 && (20..=30).contains(&column)) || (column == 30 && (30..=40).contains(&row));
                grid.set(column, row, Some(0.25), !wall, Some(Region::Team(1)));
            }
        }
        Nav::new(Arc::new(grid))
    }

    #[test]
    fn a_bot_off_its_way_in_a_corner_finds_the_way_again_and_is_not_held_there() {
        // In the crook of the L, against both walls, with a way it found
        // from somewhere else: the next point of it is on the far side of
        // the wall, up and to the right, and a body walked straight at
        // that stays in the corner. (What held Hajun's teammate at the
        // end of its bridge, and the computer's players eight times a
        // game.)
        let (corner, to) = (Vec2::new(-0.02, 0.52), Vec2::new(6.0, -6.0));
        for (seed, who) in [(1, "ESTP"), (2, "INFP"), (3, "ISTJ"), (4, "ENFP")] {
            let mut rig = Rig::new(walled(), seed, who, corner, 100.0);
            let nav = rig.nav.clone();
            assert!(nav.walkable_at(corner) && !nav.line(corner, to));
            // A body walked straight at it, as a bot was until it looked:
            // nowhere.
            let mut body = Body::standing(Vec3::new(corner.x, 0.25, corner.y), 0.0);
            for _ in 0..120 {
                let walk = (to - body.at.xz()).normalize_or_zero() * RUN_SPEED;
                body.step(&nav, walk, Rig::FRAME);
            }
            assert!(body.at.xz().distance(corner) < 0.1, "{}", body.at);
            // The bot, with that for its way.
            let mut bot = rig.squad.bots[rig.seat].take().expect("seated");
            bot.path = vec![to];
            bot.path_to = Some(to);
            bot.path_at = rig.now;
            bot.path_version = nav.version;
            bot.path_cuts = nav.cuts;
            bot.progress = (corner, rig.now);
            let seen = vec![Seen::default(); ARENA_SEATS];
            let purses = [[0u16; 4]; ARENA_SEATS];
            let (mut arrived, mut furthest_back) = (None, 0.0f32);
            for frame in 0..(8.0 / Rig::FRAME) as usize {
                rig.now += Rig::FRAME;
                let view = View {
                    elapsed: rig.now,
                    nav: &nav,
                    seats: &seen,
                    purses: &purses,
                    outfits: &rig.outfits,
                    mint: &rig.mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &rig.sticks,
                };
                let walk = rig.squad.walk_to(&mut bot, &view, to, RUN_SPEED, 1.0);
                bot.body.step(&nav, walk, Rig::FRAME);
                let here = bot.body.at.xz();
                assert!(nav.walkable_at(here), "{who}: it is in the wall, at {here}");
                furthest_back = furthest_back.max(here.distance(to) - corner.distance(to));
                // Out of the corner within the second.
                assert!(frame as f32 * Rig::FRAME < 1.0 || here.distance(corner) > 1.0, "{who}: still in the corner");
                if here.distance(to) <= ARRIVED + 0.05 {
                    arrived = Some(frame as f32 * Rig::FRAME);
                    break;
                }
            }
            // Round the end of one wall or the other: 17 m or so at a
            // run, weaving, and nothing holding it on the way.
            let took = arrived.unwrap_or_else(|| panic!("{who}: it never got there, and is at {}", bot.body.at));
            assert!(took < 4.5, "{who}: {took} s");
            assert_eq!(bot.pinned, 0, "{who}");
            assert_eq!(bot.stalls, 0, "{who}");
            rig.squad.bots[rig.seat] = Some(bot);
        }
    }

    #[test]
    fn a_bot_at_a_stand_takes_a_moment_over_each_thing_it_buys() {
        // On Hajun's map, at its own team's stand, with silver and gold
        // for a sword and a shield and nothing in its hand: how long it
        // stands there before each thing is asked for.
        let seat = 5;
        let stand = Nav::arena().grid().nearest_clear(ARENA_SHOPS[usize::from(arena_team(seat))], 4.0).expect("room at the stand");
        let mut waits = Vec::new();
        for seed in 0..10 {
            let mut rig = Rig::new(Nav::arena(), seed, "ESTJ", stand, 30.0);
            let mut purses = [[0u16; 4]; ARENA_SEATS];
            purses[seat] = [40, 20, 0, 0];
            let mut seen = vec![Seen::default(); ARENA_SEATS];
            let (mut arrived, mut bought) = (rig.now, 0);
            for _ in 0..(20.0 / Rig::FRAME) as usize {
                rig.now += Rig::FRAME;
                seen[seat] = Seen {
                    at: rig.bot().body.at,
                    alive: true,
                    in_play: true,
                    unseen: false,
                    person: false,
                    health: 1.0,
                };
                let order = {
                    let view = View {
                        elapsed: rig.now,
                        nav: &rig.nav,
                        seats: &seen,
                        purses: &purses,
                        outfits: &rig.outfits,
                        mint: &rig.mint,
                        anchors: [120.0; ARENA_TEAMS],
                        sticks: &rig.sticks,
                    };
                    rig.squad.step(seat, &view, Rig::FRAME)
                };
                // Away from the stand, on its way to it or off again, no
                // time is being taken over anything.
                if rig.at().distance(stand) > 1.2 || !matches!(rig.bot().goal(), Goal::Shop(_)) {
                    arrived = rig.now;
                }
                if let Some(ware) = order.buy {
                    waits.push(rig.now - arrived);
                    arrived = rig.now;
                    // Sold, as the game would sell it.
                    assert!(rig.outfits.buy(seat, ware, &mut purses[seat]), "seed {seed}: it asked for {ware:?} and could not have it");
                    rig.squad.bought(seat, ware, rig.now);
                    bought += 1;
                    if bought == 2 {
                        break;
                    }
                }
            }
            assert_eq!(bought, 2, "seed {seed}: it bought {bought} things in twenty seconds");
        }
        // Each within what anyone takes, a frame over at the most; not
        // all alike; and about a second on the whole.
        for wait in &waits {
            assert!((SHOP_SECS.0..=SHOP_SECS.1 + 2.0 * Rig::FRAME).contains(wait), "{wait} s at the stand: {waits:?}");
        }
        let (least, most) = waits.iter().fold((f32::MAX, 0.0f32), |(least, most), &wait| (least.min(wait), most.max(wait)));
        assert!(most - least > 0.3, "all of a length: {waits:?}");
        let mean = waits.iter().sum::<f32>() / waits.len() as f32;
        assert!((0.8..=1.2).contains(&mean), "{mean} s on average: {waits:?}");
    }

    #[test]
    fn a_bot_does_not_weave_into_the_wall_it_walks_along() {
        // Along the wall's upper side, an arm's length from it, to its
        // far end and round it: the straight way is clear, and the weave
        // would take it into the wall. It gets there as fast as one that
        // walks straight, near enough, and is never against the wall
        // getting nowhere.
        let (from, to) = (Vec2::new(-9.0, -0.3), Vec2::new(2.0, -0.3));
        for (seed, who) in [(5, "ENFP"), (6, "ESTP"), (7, "ISFP")] {
            let mut rig = Rig::new(walled(), seed, who, from, 100.0);
            let nav = rig.nav.clone();
            assert!(nav.line(from, to));
            let mut bot = rig.squad.bots[rig.seat].take().expect("seated");
            // (It has stood here since the game began, a hundred seconds
            // ago: setting out is not getting nowhere.)
            assert_eq!(bot.progress.1, 0.0);
            let seen = vec![Seen::default(); ARENA_SEATS];
            let purses = [[0u16; 4]; ARENA_SEATS];
            let (mut arrived, mut slowest) = (None, f32::MAX);
            let mut was = from;
            for frame in 0..(6.0 / Rig::FRAME) as usize {
                rig.now += Rig::FRAME;
                let view = View {
                    elapsed: rig.now,
                    nav: &nav,
                    seats: &seen,
                    purses: &purses,
                    outfits: &rig.outfits,
                    mint: &rig.mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &rig.sticks,
                };
                let walk = rig.squad.walk_to(&mut bot, &view, to, RUN_SPEED, 1.0);
                bot.body.step(&nav, walk, Rig::FRAME);
                let here = bot.body.at.xz();
                if here.distance(to) <= ARRIVED + 0.05 {
                    arrived = Some(frame as f32 * Rig::FRAME);
                    break;
                }
                // Every quarter of a second, how far it has come in it.
                if frame % 15 == 14 {
                    if here.distance(to) > 1.5 {
                        slowest = slowest.min(here.distance(was) / (15.0 * Rig::FRAME));
                    }
                    was = here;
                }
            }
            let took = arrived.unwrap_or_else(|| panic!("{who}: it never got there, and is at {}", bot.body.at));
            assert!(took < 11.0 / RUN_SPEED + 0.6, "{who}: {took} s for 11 m");
            assert!(slowest > RUN_SPEED * 0.6, "{who}: down to {slowest} m/s along the wall");
            assert_eq!((bot.stalls, bot.pinned), (0, 0), "{who}");
            rig.squad.bots[rig.seat] = Some(bot);
        }
    }

    #[test]
    fn a_hole_blown_in_a_bridge_is_jumped_if_it_is_short_and_mended_if_it_is_not() {
        let span = two_islands().1;
        let far = Vec2::new(30.0, 0.5);
        // Four planks gone, 5.9 m, as a blast takes all but always: after
        // someone on the far land, a bot runs up to the hole and jumps
        // it.
        let mut over = 0;
        for seed in 0..5 {
            let mut rig = Rig::hunting(bridged(), seed, "ESTJ", Vec2::new(2.0, 0.0));
            assert_eq!(rig.nav.blast(span.rung(7), DYNAMITE_RADIUS).len(), 4);
            let gap = rig.nav.gaps()[0];
            for _ in 0..(8.0 / Rig::FRAME) as usize {
                rig.step(Some(far));
                assert!(!rig.fallen(), "seed {seed}: it fell into the hole");
                if rig.at().x > gap.b.x + 1.0 && rig.bot().body.free(&rig.nav) {
                    over += 1;
                    break;
                }
            }
            assert_eq!(rig.rungs, 0, "it laid planks where it could jump");
        }
        assert_eq!(over, 5, "over the hole {over} times of five");
        // Five planks gone, 7.3 m, as a blast on the very middle of a
        // plank takes, is further than it jumps. With nobody it can get
        // at, and the metres for it, it lays planks across the hole, from
        // the end of what stands on its side to the end of what stands on
        // the other: a few planks, not a bridge.
        let mut rig = Rig::new(bridged(), 4, "ISTJ", Vec2::new(2.0, 0.0), 400.0);
        let hole = (span.rung(7) + span.rung(8)) * 0.5;
        assert_eq!(rig.nav.blast(hole, DYNAMITE_RADIUS).len(), 5);
        let gap = rig.nav.gaps()[0];
        assert!(!rig.nav.joined(Region::Team(1), Region::Middle));
        // Nothing to lay them with yet: it stays on its own side.
        for _ in 0..(4.0 / Rig::FRAME) as usize {
            rig.step(Some(far));
            assert!(!rig.fallen() && rig.at().x < gap.a.x + 0.3, "{}", rig.at());
        }
        assert_eq!(rig.rungs, 0);
        // 25 m, as the shop sells them.
        let now = rig.now;
        rig.squad.bought(rig.seat, Ware::Bridge, now);
        let mut mended = None;
        for _ in 0..(25.0 / Rig::FRAME) as usize {
            rig.step(Some(far));
            assert!(!rig.fallen(), "it fell mending the hole");
            if rig.nav.joined(Region::Team(1), Region::Middle) {
                mended = Some(rig.now - now);
                break;
            }
        }
        let took = mended.expect("the hole was not mended");
        assert!(took < 20.0, "{took} s");
        // (Joined as its last plank but one comes within a body's reach
        // of what stands: it lays the last, and walks to the end of it.)
        for _ in 0..(3.0 / Rig::FRAME) as usize {
            rig.step(Some(far));
        }
        assert!(!rig.fallen());
        // One rung to begin on and a plank's worth after it for each plank
        // of the hole, and one over for the ends: not the fifteen of a
        // bridge begun again.
        assert!((6..=9).contains(&rig.rungs), "{} rungs", rig.rungs);
        let left = rig.bot().metres();
        assert!(left > 14.0 && left < 19.5, "{left} m left of 25");
        // Its planks begin inside what stands on the near side and end
        // inside what stands on the far side, straight along the bridge.
        let laid = rig.nav.rungs(rig.seat as u8, 0);
        assert!(laid[0].x < gap.a.x && laid[0].x > gap.a.x - 1.0 && laid[0].y.abs() < 0.3, "{:?}", laid[0]);
        let last = laid[laid.len() - 1];
        assert!(last.x > gap.b.x && last.x < gap.b.x + 1.0 && last.y.abs() < 0.3, "{last:?}");
    }

    #[test]
    fn a_bot_comes_down_past_someone_on_the_edge_of_a_hole_and_waits_for_someone_coming_over_it() {
        let span = two_islands().1;
        let holed = || {
            let mut nav = bridged();
            assert_eq!(nav.blast(span.rung(7), DYNAMITE_RADIUS).len(), 4);
            let gap = nav.gaps()[0];
            (nav, gap)
        };
        // Someone standing on the very edge of what is left past a hole
        // of four planks. A bot after them jumps it and comes down on the
        // planks behind them, whoever it is, and never in the hole: for a
        // night the place to come down was where they stood, the edge
        // itself, the jump was over a step short of it, in the air, and
        // what it then steered for was a square with nothing under it.
        let mut over = 0;
        for who in ["ESTP", "ISTP", "ENTJ", "INFP"] {
            for seed in 0..6 {
                let (nav, gap) = holed();
                let mut rig = Rig::hunting(nav, seed, who, Vec2::new(2.0, 0.0));
                for _ in 0..(9.0 / Rig::FRAME) as usize {
                    rig.step(Some(gap.b));
                    assert!(!rig.fallen(), "{who}, seed {seed}: it fell into the hole");
                    if rig.at().x > gap.b.x - 0.1 && rig.bot().body.free(&rig.nav) {
                        over += 1;
                        break;
                    }
                }
            }
        }
        assert!(over >= 20, "over the hole {over} times of twenty-four");
        // Someone running at it from the far land, off the edge and over
        // the hole: it lets them come. It is not jumping the hole itself
        // while they are on their way, to pass them in the air.
        for seed in 0..10 {
            let (nav, gap) = holed();
            let mut rig = Rig::hunting(nav, seed, "ESTP", gap.a_in - Vec2::X * 2.0);
            // From the far land, 16 m past the hole, to a step in from the
            // edge on the bot's side: a little over two seconds of it.
            let from = gap.b.x + 16.0;
            let frames = ((from - gap.a.x + 0.8) / (RUN_SPEED * Rig::FRAME)) as usize;
            let mut was = false;
            for frame in 0..frames {
                let run = frame as f32 * RUN_SPEED * Rig::FRAME;
                rig.step(Some(Vec2::new(from - run, 0.0)));
                assert!(!rig.fallen(), "seed {seed}: it fell");
                // (For the first quarter of a second it cannot know which
                // way they are going.)
                let leaping = rig.bot().leaping();
                assert!(frame < 15 || was || !leaping, "seed {seed}: it jumped at someone on their way over, {run} m into their run");
                was = leaping;
            }
            // And they are met where it stood, on its own side.
            assert!(rig.at().x < gap.a.x + 0.1, "seed {seed}: it is at {}", rig.at());
        }
    }

    #[test]
    fn a_bot_on_the_land_throws_dynamite_at_whoever_comes_over_a_bridge_and_at_nobody_else() {
        // Someone coming over the bridge at a walk, 9 m off and nearer, a
        // few minutes into a game, with the bot minding its own island and
        // a stick of dynamite on a button.
        let thrown = |at: Vec2, mate: Option<Vec2>, holds: u8, seed: u64, walks: f32| {
            let mut rig = Rig::new(bridged(), seed, "INTP", at, 200.0);
            rig.outfits.kits[5] = Kit::default().holding(Utility::Dynamite, holds);
            rig.mate = mate;
            let mut person = Vec2::new(15.0, 0.0);
            for _ in 0..(2.5 / Rig::FRAME) as usize {
                person.x -= walks * Rig::FRAME;
                let order = rig.step(Some(person));
                if let Some((heading, far)) = order.blast {
                    let way = Vec2::new(-heading.sin(), -heading.cos());
                    return Some((rig.at() + way * far, far, person, rig.nav.clone()));
                }
            }
            None
        };
        // (Its own island's bridge, which goes with it, all of it, since
        // 2026-11-15: so by who it is, and not every time. The tinkerer,
        // who minds the edge, more often than not.)
        let home = Vec2::new(4.0, 0.3);
        let mut threw = 0;
        for seed in 0..16 {
            let Some((down, far, person, nav)) = thrown(home, None, 1, seed, 3.0) else {
                continue;
            };
            threw += 1;
            // Onto the bridge, at where they are going, no further than
            // one can be thrown.
            assert!((DYNAMITE_NEAREST..=DYNAMITE_FURTHEST).contains(&far), "{far} m");
            assert!(out_over_the_void(&nav, down), "it comes down off the bridge, at {down}");
            assert!(down.x <= person.x + 0.5 && down.x > 6.0, "at {down}, with them at {person}");
        }
        assert!((6..16).contains(&threw), "it threw it {threw} times of sixteen");
        // Not with none to throw; nor at where one of its own stands on
        // the planks that would go.
        assert!(thrown(home, None, 0, 1, 3.0).is_none());
        assert!(thrown(home, Some(Vec2::new(9.0, 0.0)), 1, 1, 3.0).is_none());
        // From out on a bridge itself, never at anyone on that bridge,
        // however far along it they stand: it would go down with them.
        // (Until 2026-11-15, when a stick took four planks and no more,
        // it threw at whoever stood well clear of it.)
        // If it throws it at all, it has walked back to its island
        // first.
        for (out, walks) in [(Vec2::new(8.0, 0.0), 0.0), (Vec2::new(11.0, 0.0), 2.0)] {
            for seed in 0..6 {
                let mut rig = Rig::new(bridged(), seed, "INTP", out, 200.0);
                rig.outfits.kits[5] = Kit::default().holding(Utility::Dynamite, 1);
                let mut person = Vec2::new(15.0, 0.0);
                for _ in 0..(2.5 / Rig::FRAME) as usize {
                    person.x -= walks * Rig::FRAME;
                    let order = rig.step(Some(person));
                    assert!(order.blast.is_none() || !out_over_the_void(&rig.nav, rig.at()), "seed {seed}: it threw it from {}", rig.at());
                }
            }
        }
    }

    #[test]
    fn a_body_with_a_leap_potion_on_it_clears_a_hole_no_jump_of_its_own_does() {
        // A bridge with six planks blown out of it, 9 m of nothing:
        // further than anyone jumps, as the test of a jump has it.
        let mut nav = bridged();
        let span = two_islands().1;
        assert_eq!(nav.blast(span.rung(7), 3.7).len(), 6);
        let hole = nav.gaps()[0];
        let across = hole.length();
        assert!(across > Squad::reach_of_a_jump(RUN_SPEED, false) + 2.0, "{across} m");
        assert!(across < Squad::reach_of_a_jump(RUN_SPEED, true) - 2.0, "{across} m");
        let frame = 1.0 / 60.0;
        let run = (hole.b_in - hole.a_in).normalize() * RUN_SPEED;
        let mut leaper = Body::standing(Vec3::new(hole.a_in.x, PLANK, hole.a_in.y), 0.0);
        leaper.leap = true;
        leaper.high = true;
        let (mut aloft, mut highest, mut down_at) = (0.0f32, 0.0f32, None);
        for _ in 0..240 {
            leaper.step(&nav, run, frame);
            highest = highest.max(leaper.at.y);
            if leaper.lift != 0.0 {
                aloft += frame;
            }
            if leaper.at.x > hole.a.x + 1.0 && leaper.free(&nav) {
                down_at = Some(leaper.at.x);
                break;
            }
        }
        let down_at = down_at.expect("it came down");
        // Five times as high as a jump of its own, 8.2 m, and over twice
        // as long in the air.
        assert!((highest - PLANK - 8.2).abs() < 0.4, "it rose to {highest}");
        assert!((aloft - LEAP_SECS_ALOFT).abs() < 0.15, "{aloft} s in the air");
        // Well past the far edge; and what a bot counts on crossing so is
        // no further than it does cross, with what it keeps in hand.
        assert!(down_at - hole.b.x >= LEAP_SPARE, "down {} m past the far edge", down_at - hole.b.x);
        let flown = down_at - hole.a.x;
        assert!(Squad::reach_of_a_jump(RUN_SPEED, true) + LEAP_SPARE <= flown + 0.3, "{flown} m flown");
        // And without one on it, the same body goes into the hole.
        let mut gone = Body::standing(Vec3::new(hole.a_in.x, PLANK, hole.a_in.y), 0.0);
        gone.leap = true;
        for _ in 0..180 {
            gone.step(&nav, run, frame);
        }
        assert!(gone.at.y < -10.0, "{}", gone.at);
    }

    #[test]
    fn a_bot_with_a_leap_potion_drinks_it_at_a_hole_too_wide_to_jump_and_goes_over() {
        let holed = || {
            let mut nav = bridged();
            let span = two_islands().1;
            assert_eq!(nav.blast(span.rung(7), 3.7).len(), 6);
            let hole = nav.gaps()[0];
            (nav, hole)
        };
        // After someone on the far land, late in a game. With nothing to
        // cross the hole with, it stays on its own side of it, and does
        // not go into it.
        let prey = Vec2::new(30.0, 0.0);
        let (nav, hole) = holed();
        let mut rig = Rig::hunting(nav, 2, "ISTP", Vec2::new(2.0, 0.0));
        for _ in 0..(8.0 / Rig::FRAME) as usize {
            rig.step(Some(prey));
            assert!(!rig.fallen(), "it fell into the hole");
        }
        assert!(rig.at().x <= hole.a.x + 0.1, "it is at {}", rig.at());
        // With a leap potion on its buttons: to the edge of the hole, the
        // potion to its mouth there, and over, to the land they stand on.
        let mut over = 0;
        for seed in 0..6 {
            let (nav, hole) = holed();
            let mut rig = Rig::hunting(nav, seed, "ISTP", Vec2::new(2.0, 0.0));
            rig.outfits.kits[rig.seat] = Kit::default().holding(Utility::Leap, 1);
            let (mut drank_at, mut highest) = (None, 0.0f32);
            for _ in 0..(14.0 / Rig::FRAME) as usize {
                let order = rig.step(Some(prey));
                if let Some(potion) = order.drink {
                    assert_eq!(potion, Utility::Leap, "seed {seed}");
                    rig.outfits.drink(rig.seat, potion, rig.now);
                    drank_at.get_or_insert(rig.at());
                }
                assert!(!rig.fallen(), "seed {seed}: it fell into the hole");
                highest = highest.max(rig.bot().body.at.y);
                if rig.at().x > hole.b.x + 0.5 && rig.bot().body.free(&rig.nav) {
                    over += 1;
                    break;
                }
            }
            let drank_at = drank_at.expect("it never drank it");
            assert!(drank_at.distance(hole.a_in) < 1.5, "seed {seed}: it drank it at {drank_at}");
            assert!(highest > 6.0, "seed {seed}: it rose to {highest}");
        }
        assert!(over >= 5, "over the hole {over} times of six");
    }

    /// Everyone standing in the middle of their own island on Hajun's
    /// map, in play: what a bot sees of a game nobody has moved in.
    fn everyone_at_home() -> Vec<Seen> {
        (0..ARENA_SEATS)
            .map(|seat| Seen {
                at: Vec3::new(ARENA_ISLANDS[seat % ARENA_TEAMS].x, 0.25, ARENA_ISLANDS[seat % ARENA_TEAMS].y),
                alive: true,
                in_play: true,
                unseen: false,
                person: false,
                health: 1.0,
            })
            .collect()
    }

    #[test]
    fn what_one_is_on_an_island_it_raids_for_goes_by_who_it_is_and_how_things_stand() {
        let nav = Nav::arena();
        let outfits = {
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            outfits
        };
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        // How often one of team 1's, stepping onto team 0's island, is
        // there for their anchor, for whoever is home, and for their pad.
        let shares = |name: &str, anchors: [f32; ARENA_TEAMS], home: bool, now: f32, taken: bool| {
            let mut squad = Squad::new(7, Skill::MODERATE);
            squad.seat(5, Vec3::new(-130.0, 0.25, 0.0), 0.0);
            assert!(squad.cast(5, name));
            if taken {
                squad.strikers[0] = Some(6);
            }
            let bot = squad.bot(5).expect("seated").clone();
            let mut seen = everyone_at_home();
            if !home {
                for seat in [0, 4] {
                    seen[seat].alive = false;
                }
            }
            let view = View {
                elapsed: now,
                nav: &nav,
                seats: &seen,
                purses: &purses,
                outfits: &outfits,
                mint: &mint,
                anchors,
                sticks: &[],
            };
            let mut counts = [0u32; 3];
            for _ in 0..600 {
                let which = match squad.take_aim(&bot, &view, 0) {
                    Aim::Anchor => 0,
                    Aim::Players => 1,
                    Aim::Loot => 2,
                };
                counts[which] += 1;
            }
            counts.map(|count| count as f32 / 600.0)
        };
        let whole = [120.0; ARENA_TEAMS];
        // Late enough for an anchor to be fair game to anyone.
        let late = Skill::MODERATE.anchors_from.1 + 10.0;
        // The commander is there for the anchor, whoever is home; the
        // troll for whoever is home, and hardly ever for the anchor.
        let [anchor, ..] = shares("ENTJ", whole, true, late, false);
        assert!(anchor > 0.7, "the commander, for the anchor {anchor}");
        let [anchor, players, _] = shares("ESFP", whole, true, late, false);
        assert!(anchor < 0.1 && players > 0.6, "the troll, for the anchor {anchor} and for them {players}");
        // With nobody home there is nobody to be there for: the anchor,
        // mostly, where with them home the lone wolf would have fought.
        let [empty, players, _] = shares("ISTP", whole, false, late, false);
        assert!(empty > 0.5 && players < 0.3, "nobody home: for the anchor {empty}, for them {players}");
        let [defended, players, _] = shares("ISTP", whole, true, late, false);
        assert!(defended < empty - 0.2 && players > 0.5, "them home: for the anchor {defended}, for them {players}");
        // No anchor before anchors are fair game to it; none that is
        // broken already; and none another of the computer's is at.
        assert_eq!(shares("ENTJ", whole, true, 100.0, false)[0], 0.0);
        assert_eq!(shares("ENTJ", [0.0, 120.0, 120.0, 120.0], true, late, false)[0], 0.0);
        assert_eq!(shares("ENTJ", whole, true, late, true)[0], 0.0);
        // And with two anchors gone already, not far into a game, another
        // is gone for a good deal less often.
        assert!(late < Skill::MODERATE.anchors_spared_until.0);
        let [spared, ..] = shares("ISTP", [120.0, 120.0, 0.0, 0.0], false, late, false);
        assert!(spared < 0.75 * empty, "two gone: for the anchor {spared}, where it was {empty}");
    }

    #[test]
    fn one_set_on_an_anchor_hits_it_whoever_stands_by_and_one_that_is_not_turns_on_them() {
        // Hajun's map. One of team 1's on team 0's island, on a raid, by
        // the shore; someone of team 0's standing a step from where their
        // anchor is hit from, all the while.
        let anchor = ARENA_ANCHORS[0];
        let by = anchor + Vec2::new(1.5, -1.7);
        let raid = |name: &str, zeal: f32, seed: u64| {
            let nav = Nav::arena();
            let from = nav.nearest_walkable(Vec2::new(3.0, 121.5)).expect("the shore of their island");
            let mut rig = Rig::new(nav, seed, name, from, 400.0);
            {
                let bot = rig.squad.bots[rig.seat].as_mut().expect("seated");
                bot.goal = Goal::Raid(0);
                bot.think_at = f32::MAX;
                bot.temper.zeal = zeal;
                bot.anchors_from = 0.0;
            }
            let (mut strikes, mut punches) = (0u32, 0u32);
            for frame in 0..(14.0 / Rig::FRAME) as usize {
                let order = rig.step(Some(by));
                // There for the anchor, whatever it threw as it stepped
                // ashore.
                if frame == 0 {
                    let bot = rig.squad.bots[rig.seat].as_mut().expect("seated");
                    bot.aim = Aim::Anchor;
                }
                strikes += u32::from(order.strike == Some(0));
                punches += u32::from(order.punch == Some(0));
            }
            (strikes, punches, rig.at())
        };
        // As set on it as they come: the anchor and nothing else, a punch
        // every second and a half or so, from beside it.
        for seed in 0..4 {
            let (strikes, punches, at) = raid("ENTJ", 1.0, seed);
            assert!(strikes >= 5 && punches == 0, "seed {seed}: {strikes} at the anchor, {punches} at them");
            assert!(at.distance(anchor) < 3.0, "seed {seed}: it is at {at}");
        }
        // Not set on it at all: whoever stands by has it at them.
        let mut turned = 0;
        for seed in 0..4 {
            let (_, punches, _) = raid("ISTP", 0.0, seed);
            turned += u32::from(punches > 0);
        }
        assert!(turned >= 3, "it turned on them {turned} times of four");
    }

    #[test]
    fn with_nothing_it_has_to_do_one_goes_out_to_look_about_and_another_stands_at_its_stands() {
        // Hajun's map, team 1's island joined to the middle and the middle
        // to a smaller island beside it, so that there is no bridge it
        // has to build; one of the computer's alone on it for two
        // minutes, with nothing in its purse and nothing lying on its pad.
        let seat = 5;
        let home = ARENA_ISLANDS[1];
        let watch = |name: &str, out: bool| {
            let mut nav = Nav::arena();
            for (chain, (from, to)) in [(Region::Team(1), Region::Middle), (Region::Middle, Region::Sapphire(beside(1)[0]))].into_iter().enumerate() {
                let span = nav.grid().span(from, to, 0.0).expect("a span between them");
                for index in 0..=span.planks() {
                    nav.hear(1, chain as u16, index as u16, span.rung(index));
                }
            }
            let mut squad = Squad::new(4, Skill::MODERATE);
            squad.seat(seat, Vec3::new(home.x, 0.25, home.y), 0.0);
            assert!(squad.cast(seat, name));
            {
                let bot = squad.bots[seat].as_mut().expect("seated");
                bot.out = out;
                bot.out_until = f32::MAX;
                bot.scout_again = 0.0;
            }
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            // Seconds of it: out looking about; off its island; standing
            // still off it; standing still at a stand; and by its pad.
            let mut spent = [0.0f32; 5];
            let mut was = home;
            let dt = 1.0 / 60.0;
            alone(&mut squad, &mut nav, &mut outfits, seat, None, 1.0, 60.0, 120.0, |_, _, nav, bot| {
                let at = bot.body.at.xz();
                let still = at.distance(was) < 0.3 * dt;
                was = at;
                let off = nav.grid().height(at).is_none() || nav.grid().region(at) != Some(Region::Team(1));
                let at_a_stand = [ARENA_SHOPS[1], ARENA_UPGRADE_SHOPS[1]].iter().any(|stand| stand.distance(at) < 1.6);
                for (which, so) in [matches!(bot.goal(), Goal::Scout(_)), off, off && still, at_a_stand && still, at.distance(ARENA_PADS[1]) < 2.0]
                    .into_iter()
                    .enumerate()
                {
                    if so {
                        spent[which] += dt;
                    }
                }
                false
            });
            spent
        };
        // One that is all over the map, and in a mood to go out: off over
        // its bridge, looking about a while where it comes to, and off
        // again.
        let [scouting, off, looking, _, pad] = watch("ENFP", true);
        assert!(scouting > 40.0 && off > 30.0, "out looking about {scouting} s, off its island {off} s");
        assert!(looking > 4.0, "stood looking about {looking} s");
        assert!(pad < 1.0, "by its pad {pad} s");
        // One that keeps to its island, and not in a mood to: about its
        // island, a good part of the time at one or the other of its
        // stands, in the outline; never out, and not on its pad.
        let [scouting, off, _, at_stands, pad] = watch("ISFJ", false);
        assert_eq!((scouting, off), (0.0, 0.0));
        assert!(at_stands > 25.0, "at its stands {at_stands} s");
        assert!(pad < 1.0, "by its pad {pad} s");
    }

    #[test]
    fn one_given_to_bridges_thinks_of_a_way_onto_another_team_s_island_that_its_team_could_do_without() {
        // Hajun's map, a few minutes in: team 0's island joined to the
        // middle and to the first of the two smaller islands beside it,
        // and team 1's to the middle by a bridge of its own.
        let mut nav = Nav::arena();
        let [first, _] = beside(0);
        for (chain, (from, to)) in [(Region::Team(0), Region::Middle), (Region::Middle, Region::Sapphire(first)), (Region::Team(1), Region::Middle)]
            .into_iter()
            .enumerate()
        {
            let span = nav.grid().span(from, to, 0.0).expect("a span between them");
            for index in 0..=span.planks() {
                nav.hear(0, chain as u16, index as u16, span.rung(index));
            }
        }
        let outfits = {
            let mut outfits = Outfits::default();
            outfits.seat(ARENA_SEATS);
            outfits
        };
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        let seen = everyone_at_home();
        // What the one of team 0's that goes out settles on, thinking of
        // it now and then from four minutes in to ten.
        let settled = |name: &str, seed: u64, free: bool| {
            let mut squad = Squad::new(seed, Skill::MODERATE);
            squad.seat(4, Vec3::new(0.0, 0.25, 130.0), 0.0);
            assert!(squad.cast(4, name));
            let mut bot = squad.bot(4).expect("seated").clone();
            let mut now = 240.0;
            while now < 600.0 {
                let view = View {
                    elapsed: now,
                    nav: &nav,
                    seats: &seen,
                    purses: &purses,
                    outfits: &outfits,
                    mint: &mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &[],
                };
                if let Some(scheme) = squad.scheme(&mut bot, &view, free) {
                    // Kept to, once settled on.
                    assert_eq!(squad.scheme(&mut bot, &view, free), Some(scheme));
                    return Some(scheme);
                }
                now += 4.0;
            }
            None
        };
        // The trickster, given to them: a side door from the smaller
        // island its team has reached, a second way in from the middle
        // onto the flank of an island that has its own way there, or its
        // own island straight to the one at its side.
        let (mut doors, mut seconds, mut straight) = (0, 0, 0);
        for seed in 0..30 {
            let (from, to, span) = settled("ENTP", seed, true).expect("it never thought of one");
            let Region::Team(theirs) = to else {
                panic!("not onto a team's island: {to:?}");
            };
            assert_ne!(theirs, 0);
            match from {
                Region::Sapphire(island) => {
                    assert!(island == first && beside(theirs).contains(&island), "from {island} to team {theirs}'s");
                    assert!((70.0..90.0).contains(&span.length()), "{} m", span.length());
                    doors += 1;
                }
                Region::Middle => {
                    // Team 1's, the one that has a way to the middle; and
                    // on its flank: their own bridge runs along the x
                    // axis.
                    assert_eq!(theirs, 1);
                    assert!(span.to.y.abs() > 5.0 && span.length() < 56.0, "it lands at {}, {} m", span.to, span.length());
                    seconds += 1;
                }
                Region::Team(0) => {
                    assert!(theirs == 1 || theirs == 3, "straight to team {theirs}'s");
                    assert!((155.0..180.0).contains(&span.length()), "{} m", span.length());
                    straight += 1;
                }
                other => panic!("from {other:?}"),
            }
        }
        assert!(doors >= 3 && seconds >= 3 && straight >= 1, "{doors} side doors, {seconds} second ways in, {straight} straight across");
        // Not while it has something it has to do; and the sentinel and
        // the enforcer, by the book and head on, never think of one.
        assert!(settled("ENTP", 1, false).is_none());
        for seed in 0..6 {
            assert!(settled("ISTJ", seed, true).is_none() && settled("ESTJ", seed, true).is_none(), "seed {seed}");
        }
    }

    #[test]
    fn its_team_s_building_is_quickened_and_a_leap_potion_carried_by_who_it_is() {
        let team = Upgrades::default();
        let armed = Kit {
            sword: 2,
            shield: 1,
            ..Kit::default()
        };
        let healing = [(Utility::Healing, 1)];
        let sapphire: Purse = [0, 0, 9, 0];
        let buys = |name: &str, team: &Upgrades| shopping(persona(name), &healing, Role::Runner, false, 200.0, &sapphire, &armed, team, 0.0, false, false);
        // One given to bridges has its team's building quickened before
        // anything else sapphire buys; one that is not, after all of it.
        assert_eq!(buys("ENTP", &team)[0], Ware::BuildSpeed);
        let plain = buys("ISTJ", &team);
        let place = |ware: Ware| plain.iter().position(|other| *other == ware).unwrap_or_else(|| panic!("no {ware:?} in {plain:?}"));
        for other in [Ware::HealPool, Ware::Forge, Ware::StrongerSwords, Ware::BetterShields, Ware::Trap] {
            assert!(place(Ware::BuildSpeed) > place(other), "{plain:?}");
        }
        // With it quickened all the way, there is no more of it to buy.
        let fastest = Upgrades {
            build_speed: crate::bridge_wars::BUILD_SPEED_MOST,
            ..team
        };
        assert!(!buys("ENTP", &fastest).contains(&Ware::BuildSpeed));
        // A leap potion, for one that means to carry one, once it has the
        // amethyst for it; and a keeper with nobody to fetch amethyst
        // makes do with a healing potion, as for anything amethyst buys.
        let leaps = [(Utility::Leap, 1)];
        let carried = |role, purse: &Purse| shopping(persona("ENFP"), &leaps, role, false, 200.0, purse, &armed, &team, 0.0, false, false);
        assert!(carried(Role::Runner, &[0, 0, 0, 1]).contains(&Ware::Utility(Utility::Leap)));
        let kept = carried(Role::Keeper, &[0, 9, 0, 0]);
        assert!(kept.contains(&Ware::Utility(Utility::Healing)) && !kept.contains(&Ware::Utility(Utility::Leap)), "{kept:?}");
    }

    /// A stick of dynamite come down `at` on `nav`'s bridge, as a game
    /// has it since 2026-11-15: the planks in its blast gone, and every
    /// other plank of the bridge doomed to fall, one after another from
    /// the hole outward. How many planks the blast itself took.
    fn blown(nav: &mut Nav, at: Vec2) -> usize {
        let taken = nav.blast(at, DYNAMITE_RADIUS);
        let falls = nav.cascade(&taken);
        nav.doom(&falls, 0);
        assert!(nav.crumbling());
        taken.len()
    }

    #[test]
    fn a_bot_on_a_bridge_that_is_coming_down_gets_off_it_and_one_on_the_land_keeps_off_it() {
        let span = two_islands().1;
        // Out on its island's bridge, a third of the way over, as the far
        // end of it is blown: the falling comes at it from there, slower
        // than it runs. Whoever it is, it is on its island before the
        // planks it stood on are gone, and it never goes down with them.
        for who in ["ISTP", "ESTP", "ISTJ", "INFP", "ENFP", "ESFJ"] {
            for seed in 0..6 {
                let mut rig = Rig::new(bridged(), seed, who, span.rung(4), 200.0);
                assert!(blown(&mut rig.nav, span.rung(11)) >= 3);
                assert!(rig.nav.falls_in(rig.at()).is_some(), "it does not stand on planks that are going");
                let mut ashore = None;
                while rig.nav.crumbling() {
                    rig.step(None);
                    rig.nav.crumble(Rig::FRAME);
                    assert!(!rig.fallen(), "{who}, seed {seed}: it went down with the bridge");
                    if ashore.is_none() && rig.nav.stays_at(rig.at()) {
                        ashore = Some(rig.now - 200.0);
                    }
                }
                let took = ashore.expect("it is not on ground that stays");
                assert!(took < 2.6, "{who}, seed {seed}: {took} s to get off");
                assert!(rig.nav.grid().region(rig.at()) == Some(Region::Team(1)) && rig.nav.stays_at(rig.at()), "{who}, seed {seed}: at {}", rig.at());
                assert!(!rig.bot().fleeing());
            }
        }
        // Standing where its bridge leaves its island, on the rim of the
        // land under the bridge's end, where a body is walked only for
        // the planks' being there: it steps back onto ground that stays,
        // well before they go. (Until this was so it stood there, on land,
        // with no way to anywhere, until the last plank had fallen.)
        for seed in 0..6 {
            let rim = Vec2::new(5.8, 0.0);
            let mut rig = Rig::new(bridged(), seed, "ISFJ", rim, 200.0);
            blown(&mut rig.nav, span.rung(11));
            assert!(rig.nav.grid().height(rim).is_some() && rig.nav.walkable_at(rim) && !rig.nav.stays_at(rim));
            let mut back = None;
            for _ in 0..(1.8 / Rig::FRAME) as usize {
                rig.step(None);
                rig.nav.crumble(Rig::FRAME);
                if back.is_none() && rig.nav.stays_at(rig.at()) {
                    back = Some(rig.now - 200.0);
                }
            }
            assert!(back.is_some(), "seed {seed}: still on the rim under the planks, at {}", rig.at());
        }
        // After someone on the far land, at a run for its bridge, as the
        // bridge is blown: it sets no foot on the planks while they are
        // coming down, and is on its island when they are gone.
        for seed in 0..8 {
            let far = Vec2::new(30.0, 0.5);
            let mut rig = Rig::hunting(bridged(), seed, "ESTP", Vec2::new(2.0, 0.0));
            for _ in 0..120 {
                rig.step(Some(far));
                if rig.at().x > 4.6 {
                    break;
                }
            }
            assert!(rig.at().x > 4.6 && rig.nav.grid().height(rig.at()).is_some(), "seed {seed}: it did not set out for the bridge, at {}", rig.at());
            blown(&mut rig.nav, span.rung(11));
            while rig.nav.crumbling() {
                rig.step(Some(far));
                rig.nav.crumble(Rig::FRAME);
                assert!(rig.nav.falls_in(rig.at()).is_none() && !rig.fallen(), "seed {seed}: on the falling planks, at {}", rig.at());
            }
            assert!(rig.nav.grid().region(rig.at()) == Some(Region::Team(1)) && rig.nav.grid().height(rig.at()).is_some());
        }
    }

    #[test]
    fn the_quick_get_off_a_bridge_blown_right_beside_them_and_the_slow_do_not_always() {
        // A plank from the edge of the hole as the stick blows: the plank
        // it stands on goes within a second, and the next a quarter of a
        // second after. How soon it takes that in is thrown, the sooner
        // the better it is in a fight ([`FALL_WAKES`]): the best of them
        // are off every time, and the worst now and then go down with it.
        let span = two_islands().1;
        let off = |who: &str| {
            (0..40)
                .filter(|&seed| {
                    let mut rig = Rig::new(bridged(), seed, who, span.rung(8), 200.0);
                    blown(&mut rig.nav, span.rung(11));
                    while rig.nav.crumbling() && !rig.fallen() {
                        rig.step(None);
                        rig.nav.crumble(Rig::FRAME);
                    }
                    !rig.fallen()
                })
                .count()
        };
        let (quick, slow) = (off("ISTP"), off("INFP"));
        println!("off a bridge blown beside them, of forty: the lone wolf {quick}, the dreamer {slow}");
        assert!(persona("ISTP").pvp > 0.9 && persona("INFP").pvp < 0.1);
        assert!(quick >= 38, "the lone wolf got off {quick} times of forty");
        assert!(slow < quick && slow >= 8, "the dreamer got off {slow} times of forty");
    }

    #[test]
    fn a_stick_of_dynamite_is_never_thrown_under_its_own_side_nor_to_strand_a_team_with_a_person_on_it() {
        // The island of team 1 at the origin, joined to the middle by one
        // bridge; whoever throws, and whoever stands where, case by case.
        let span = two_islands().1;
        let nav = bridged();
        let mint = Mint::new(0.0, &mut || 0.5);
        let purses = [[0u16; 4]; ARENA_SEATS];
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        let standing = |at: Vec2, person: bool| Seen {
            at: Vec3::new(at.x, 0.25, at.y),
            alive: true,
            in_play: true,
            unseen: false,
            person,
            health: 1.0,
        };
        // How many of forty, five kinds eight times each, standing `at`
        // in `seat`, think a stick worth throwing to come down `down`,
        // with everyone else as `others` has them: at someone on the
        // planks, or (`unmanned`) at nobody, to cut the bridge.
        let asked = |seat: usize, at: Vec2, down: Vec2, unmanned: bool, others: &[(usize, Vec2, bool)]| {
            let mut yes = 0;
            for who in ["ENTP", "ESFP", "ISTJ", "INTP", "ESTP"] {
                for seed in 0..8 {
                    let mut squad = Squad::new(seed, Skill::MODERATE);
                    squad.seat(seat, Vec3::new(at.x, 0.25, at.y), 0.0);
                    assert!(squad.cast(seat, who));
                    let bot = squad.bot(seat).expect("seated").clone();
                    let mut seen = vec![Seen::default(); ARENA_SEATS];
                    seen[seat] = standing(at, false);
                    for &(other, there, person) in others {
                        seen[other] = standing(there, person);
                    }
                    let view = View {
                        elapsed: 300.0,
                        nav: &nav,
                        seats: &seen,
                        purses: &purses,
                        outfits: &outfits,
                        mint: &mint,
                        anchors: [120.0; ARENA_TEAMS],
                        sticks: &[],
                    };
                    yes += u32::from(squad.worth_blowing(&bot, &view, down, unmanned));
                }
            }
            yes
        };
        let (home, middle) = (Vec2::new(3.0, 1.0), Vec2::new(27.0, 2.0));
        let out = |rung: usize| span.rung(rung);
        assert!(nav.grid().region(middle) == Some(Region::Middle) && nav.grid().height(middle).is_some());
        // **Never the bridge it stands on**, however far along it whoever
        // it would throw at stands: the whole of it goes.
        assert_eq!(asked(5, out(3), out(9), false, &[(0, out(9), true)]), 0);
        // **Nor one that one of its own team stands on.**
        assert_eq!(asked(6, middle, out(9), false, &[(1, out(9), false), (2, out(5), false)]), 0);
        // From the land, at someone of another team out on their own
        // bridge, with none of its own on it: every time, and a person
        // as soon as one of the computer's. A bridge someone is fighting
        // over is fair.
        assert_eq!(asked(6, middle, out(9), false, &[(1, out(9), true), (5, home, false)]), 40);
        assert_eq!(asked(6, middle, out(9), false, &[(1, out(9), false), (5, home, false)]), 40);
        // **With nobody on it**, to leave a team no way over: only the way
        // of a team still in the game, all of them the computer's.
        assert_eq!(asked(6, middle, out(10), true, &[(1, home, false), (5, home, false)]), 40);
        // Not with a person on that team, wherever they are: they would
        // be left with a bridge to lay again.
        assert_eq!(asked(6, middle, out(10), true, &[(1, home, true), (5, home, false)]), 0);
        assert_eq!(asked(6, middle, out(10), true, &[(1, middle + Vec2::Y, true), (5, home, false)]), 0);
        // Not a team that is out of the game: there is nobody to cut off.
        assert_eq!(asked(6, middle, out(10), true, &[]), 0);
        // And never its own island's.
        assert_eq!(asked(5, middle, out(10), true, &[(1, home, false)]), 0);
        // **Its own island's bridge, with someone coming over it**: by who
        // it is, not every time and not never; and never with a person of
        // its team away across it, who would have no way back.
        let own = asked(5, home, out(4), false, &[(0, out(4), true), (1, home + Vec2::Y, false)]);
        assert!((10..40).contains(&own), "its own island's bridge {own} times of forty");
        assert_eq!(asked(5, home, out(4), false, &[(0, out(4), true), (1, middle, true)]), 0);
        // (Less often with its teammate of the computer's away across it.)
        let alone = asked(5, home, out(4), false, &[(0, out(4), true), (1, middle, false)]);
        assert!(alone < own, "{alone} times with its teammate away, {own} with it home");
    }

    /// One of the computer's players of team 0 alone on the island of team
    /// 1, on Hajun's map with no bridge anywhere and nobody about, a
    /// minute into a game, with `silver` in its purse, a frame at a time
    /// for `secs` or until it has fallen: what it asks a stand for is sold
    /// to it as a game sells it, at whichever stand it stands at, and its
    /// planks go onto the map. What it bought and where; the first bridge
    /// it set about laying; when it stepped off the edge, if it did; and
    /// whether it fell.
    #[allow(clippy::type_complexity)]
    fn cut_off_on_another_island(silver: u16, secs: f32) -> (Vec<(Ware, Vec2)>, Option<(Region, Region)>, Option<f32>, bool, Nav) {
        let mut nav = Nav::arena();
        let seat = 4;
        let at = nav.heart(Region::Team(1));
        let mut squad = Squad::new(11, Skill::MODERATE);
        squad.seat(seat, Vec3::new(at.x, nav.ground(at).expect("land"), at.y), 0.0);
        assert!(squad.cast(seat, "ISTJ"));
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        let mint = Mint::new(0.0, &mut || 0.5);
        let mut purses = [[0u16; 4]; ARENA_SEATS];
        purses[seat][Item::Silver.index()] = silver;
        // Everyone else at home, but the two whose island it is on: down,
        // and not standing again while this lasts.
        let mut seen = everyone_at_home();
        for theirs in [1, 5] {
            seen[theirs].alive = false;
        }
        let (mut bought, mut laid, mut stepped, mut fell) = (Vec::new(), None, None, false);
        let (dt, from) = (1.0 / 60.0, 60.0);
        let mut now = from;
        while now < from + secs {
            now += dt;
            seen[seat].at = squad.bot(seat).expect("seated").body.at;
            let order = {
                let view = View {
                    elapsed: now,
                    nav: &nav,
                    seats: &seen,
                    purses: &purses,
                    outfits: &outfits,
                    mint: &mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &[],
                };
                squad.step(seat, &view, dt)
            };
            let here = squad.bot(seat).expect("seated").body.at;
            if let Some(ware) = order.buy
                && crate::bridge_wars::at_a_shop(here.xz(), crate::bridge_wars::sold_at(ware))
                && outfits.buy(seat, ware, &mut purses[seat])
            {
                squad.bought(seat, ware, now);
                bought.push((ware, here.xz()));
            }
            for &(chain, index, at, _) in &order.rungs {
                nav.hear(seat as u8, chain, index, at);
            }
            let bot = squad.bot(seat).expect("seated");
            if let Goal::Lay(from, to) = bot.goal() {
                laid.get_or_insert((from, to));
            }
            if bot.stepped_off() {
                stepped.get_or_insert(now - from);
            }
            if here.y < -2.0 {
                fell = true;
                break;
            }
        }
        (bought, laid, stepped, fell, nav)
    }

    #[test]
    fn one_cut_off_on_another_island_buys_planks_there_and_lays_a_way_off_it_and_with_nothing_steps_off() {
        // With silver on it: to that island's stand, whoever's it is, for
        // planks; and with them, a bridge from where it is to the middle,
        // which it lays, going back for more planks as it runs out; and
        // from the middle, its own island's bridge, from that end. It
        // does not stand there until the game is over, as one did, and
        // its teammate with it, the first night a stick of dynamite took
        // the whole bridge.
        let (bought, laid, stepped, fell, nav) = cut_off_on_another_island(200, 75.0);
        let planks: Vec<&(Ware, Vec2)> = bought.iter().filter(|(ware, _)| *ware == Ware::Bridge).collect();
        assert!(planks.len() >= 2, "it bought {bought:?}");
        assert_eq!(nav.grid().region(planks[0].1), Some(Region::Team(1)), "its first planks bought at {}", planks[0].1);
        assert_eq!(laid, Some((Region::Team(1), Region::Middle)));
        assert!(nav.joined(Region::Team(1), Region::Middle), "no way off the island it was cut off on");
        assert!(nav.joined(Region::Team(0), Region::Middle), "no way home");
        assert!(stepped.is_none() && !fell);
        // With nothing on it and nothing lying on that island's pad:
        // there is nothing it can do there, and nobody can get at it.
        // After a while of that, not at once, it steps off the edge, to
        // stand again on its own island.
        let (bought, laid, stepped, fell, _) = cut_off_on_another_island(0, 40.0);
        assert!(bought.is_empty() && laid.is_none(), "{bought:?}, {laid:?}");
        let after = stepped.expect("it stood there");
        assert!((5.0..22.0).contains(&after), "off the edge after {after} s");
        assert!(fell);
    }

    #[test]
    fn a_bot_on_the_very_edge_of_a_plank_with_its_way_across_the_void_finds_it_again() {
        // On the edge of a plank, where a body stands and none walks,
        // with a way found from somewhere else: the next point of it is on
        // the far land, off to one side, straight across the void from
        // where it stands. (Where the computer's players came to stand
        // five times in sixty games once bridges came ashore at a slant:
        // having cut the corner between the bridge and the shore, or been
        // knocked across to the next bridge.)
        let nav = bridged();
        let edge = (0..40)
            .map(|step| Vec2::new(15.0, step as f32 * 0.05))
            .filter(|at| nav.ground(*at).is_some() && !nav.walkable_at(*at))
            .last()
            .expect("an edge to stand on");
        let to = Vec2::new(28.0, 4.0);
        assert!(nav.walkable_at(to) && nav.connected(edge, to));
        for (seed, who) in [(1, "ESTP"), (2, "INFP"), (3, "ISTJ"), (4, "ENFP")] {
            let mut rig = Rig::new(bridged(), seed, who, edge, 100.0);
            let mut bot = rig.squad.bots[rig.seat].take().expect("seated");
            bot.path = vec![to];
            bot.path_to = Some(to);
            bot.path_at = rig.now;
            bot.path_version = nav.version;
            bot.path_cuts = nav.cuts;
            bot.progress = (edge, rig.now);
            let seen = vec![Seen::default(); ARENA_SEATS];
            let purses = [[0u16; 4]; ARENA_SEATS];
            let mut arrived = None;
            for _ in 0..(6.0 / Rig::FRAME) as usize {
                rig.now += Rig::FRAME;
                let view = View {
                    elapsed: rig.now,
                    nav: &nav,
                    seats: &seen,
                    purses: &purses,
                    outfits: &rig.outfits,
                    mint: &rig.mint,
                    anchors: [120.0; ARENA_TEAMS],
                    sticks: &rig.sticks,
                };
                let walk = rig.squad.walk_to(&mut bot, &view, to, RUN_SPEED, 1.0);
                bot.body.step(&nav, walk, Rig::FRAME);
                assert!(bot.body.at.y > -1.0, "{who}: it fell");
                if bot.body.at.xz().distance(to) <= ARRIVED + 0.3 {
                    arrived = Some(rig.now - 100.0);
                    break;
                }
            }
            let took = arrived.expect("it never got there");
            // 14 m at a run and a moment to look: not the three seconds
            // of getting nowhere first.
            assert!(took < 2.6, "{who}: {took} s");
            assert_eq!(bot.stalls, 0, "{who}");
        }
    }

    #[test]
    fn a_bot_goes_and_takes_what_someone_who_died_dropped_near_it_and_not_off_a_bridge_that_is_coming_down() {
        let span = two_islands().1;
        // What someone dropped, dying with their feet at `at`: five
        // sapphires and two amethysts, where the map has ground that is
        // staying, as a game drops them.
        let spill = |rig: &mut Rig, at: Vec2, seed: u64| {
            let mut dice = Dice::new(seed + 1);
            let mut held: Purse = [0, 0, 5, 2];
            let nav = &rig.nav;
            let ground = |at: Vec2| nav.stays_at(at).then(|| nav.ground(at)).flatten();
            let dropped = rig
                .mint
                .spill(&mut held, Vec3::new(at.x, 0.25, at.y), rig.now, &mut || dice.roll(), &ground)
                .expect("there is ground");
            assert_eq!(dropped.len(), 7);
        };
        // Each frame it has whatever of that is within its reach, as a
        // game gives it: how many it has had.
        let gather = |rig: &mut Rig| {
            let at = rig.bot().body.at;
            rig.mint.pick_up_where(at, rig.now, &mut |piece| piece.dropped()).len()
        };
        // With nothing it has to do, on its island, and someone dead a few
        // metres off: whoever it is, it is over there and has every piece
        // within a few seconds, a piece at a time as it walks into each.
        for who in ["ISTJ", "ENFP", "INFP", "ESTP"] {
            for seed in 0..5 {
                let mut rig = Rig::new(bridged(), seed, who, Vec2::new(-2.0, 1.0), 60.0);
                spill(&mut rig, Vec2::new(2.5, -2.0), seed);
                let (mut had, mut took) = (0, None);
                for _ in 0..(20.0 / Rig::FRAME) as usize {
                    rig.step(None);
                    had += gather(&mut rig);
                    assert!(!rig.fallen());
                    if had == 7 {
                        took = Some(rig.now - 60.0);
                        break;
                    }
                }
                let took = took.unwrap_or_else(|| panic!("{who}, seed {seed}: it had {had} of 7, at {}", rig.at()));
                assert!(took > crate::bridge_wars::SPILL_WAIT && took < 14.0, "{who}, seed {seed}: {took} s");
            }
        }
        // By as someone dies, and on its way after someone else far off:
        // most times it has what they dropped first, all of it, within a
        // few seconds of its coming down, and is then on its way again
        // ([`Squad::mind_spoils`]).
        let far = Vec2::new(30.0, 0.5);
        let mut quick = 0;
        for seed in 0..20 {
            let mut rig = Rig::hunting(bridged(), seed, "ESTP", Vec2::new(-2.0, 0.0));
            for _ in 0..20 {
                rig.step(Some(far));
            }
            let dead = rig.at() + Vec2::new(1.0, 2.2);
            spill(&mut rig, dead, seed);
            let began = rig.now;
            let mut had = 0;
            while had < 7 && rig.now - began < 5.0 {
                rig.step(Some(far));
                had += gather(&mut rig);
            }
            if had == 7 {
                quick += 1;
                // And on after whoever it was after.
                let from = rig.at();
                for _ in 0..(1.5 / Rig::FRAME) as usize {
                    rig.step(Some(far));
                }
                assert!(rig.at().x > from.x + 3.0, "seed {seed}: it stayed by where they died, at {}", rig.at());
            }
        }
        println!("by as someone dies, of twenty: {quick} had all they dropped within five seconds");
        assert!((14..=20).contains(&quick), "{quick} of twenty");
        // Dropped out on its bridge, which is then blown at its far end:
        // it lies on planks that are going, and the bot does not set foot
        // on them for it. It is on land when they have gone.
        for seed in 0..6 {
            let mut rig = Rig::new(bridged(), seed, "ESTP", Vec2::new(2.0, 0.0), 60.0);
            spill(&mut rig, span.rung(5), seed);
            blown(&mut rig.nav, span.rung(14));
            while rig.nav.crumbling() {
                rig.step(None);
                rig.nav.crumble(Rig::FRAME);
                assert!(rig.nav.falls_in(rig.at()).is_none() && !rig.fallen(), "seed {seed}: on the falling planks, at {}", rig.at());
            }
            assert!(rig.nav.grid().height(rig.at()).is_some());
        }
    }
}
