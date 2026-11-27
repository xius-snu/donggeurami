//! What 동그라미타운 and its server say to each other.
//!
//! The game (the root crate) and the server (`server/`) both build on this
//! crate, and both register it through [`protocol`], so they register the
//! same things in the same order. replicon hashes those registrations, and a
//! client whose hash differs from the server's is told it is out of date
//! instead of being let in.
//!
//! Three rules keep it cheap (`MULTIPLAYER.md`, "The one rule"):
//!
//! * A position is whole centimetres and a 16-bit turn ([`Spot`]): about ten
//!   bytes. Riding one of the town's cars, it is a place on the car, which
//!   moves with it ([`Spot::aboard`]), so that a rider is never seen beside
//!   it.
//! * Only what has changed is sent. replicon works off Bevy's change
//!   detection, the same idea as `set_if_neq` everywhere in the game, and a
//!   device sends its own position only while it changes.
//! * Who someone is, their name and their look ([`Member`]), goes once, when
//!   they come into your room, not with every move.
//!
//! What goes which way:
//!
//! * **Server to everyone in a room**, replicated: each player's [`Member`],
//!   [`Spot`] and [`Venue`]; and in a game of House Builder its [`Round`], its
//!   [`Seats`] and every piece put down, [`Built`].
//! * **Device to server**: where you are, [`Moved`], unreliable, since a lost
//!   one is replaced by the next; and everything you ask for, [`Ask`], in
//!   order.
//! * **Server to one device**: what concerns you alone, [`Tell`], in order.
//! * **Both ways, a few times as a device comes online and once a minute
//!   after**: what time it is, [`WhatTime`] and its answer [`TheTime`], for the
//!   town's clock, which the cars round the town, the fountain's jet and the
//!   day go by on every device. Nothing else about any of them is sent.

use std::f32::consts::TAU;

use bevy::math::{Quat, Vec2, Vec3};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::replication::registry::ctx::WriteCtx;
use bevy_replicon::shared::replication::registry::rule_fns::{
    RuleFns, default_deserialize, default_serialize,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// netcode's protocol id: which game a packet is for. A packet with another
/// one is dropped without a word, so this names the game and never changes;
/// versions are told apart by replicon's protocol hash instead, which a
/// client is told about.
pub const PROTOCOL_ID: u64 = u64::from_be_bytes(*b"DGRMTOWN");

/// Bumped whenever what is sent changes: a field added, taken out or turned
/// to mean something else. replicon's protocol hash is made of the names of
/// the types [`protocol`] registers and their order, and nothing of what is
/// in them (`ProtocolHasher` in bevy_replicon 0.44), so it notices a type
/// added or renamed, and nothing else.
///
/// 2 since 2026-10-23: [`Spot::aboard`], for riding the town's car.
///
/// 3 since 2026-10-24: [`WhatTime`] and [`TheTime`], for the town's clock;
/// and the town's cars are two buses, so a place on car 0 is a place on a
/// bus, which an app with the cars put somewhere else entirely.
///
/// 4 since 2026-10-30: the town is Hajun's new map, `roundtownmap.glb`, 420 m
/// across where the old one was 140, so a place in the town is somewhere
/// else entirely in an app with the old one; and everyone arrives in it on
/// the path round its pond ([`TOWN_ARRIVAL_RADIUS`]).
///
/// Not bumped for [`Ask::Profile`] (2026-10-31), added last: an app from
/// before never sends it, and draws a look it has not heard of as its first.
///
/// 5 since 2026-11-07: the town is Hajun's `pvptownmap.glb`, four islands
/// at the ends of bridges round a town with no sea round it, and everyone
/// arrives on one of the islands ([`ARENA_ISLANDS`], the town's that day); [`Ask::Respawn`], for
/// whoever falls off into the void; and [`Ask::Punch`] and [`Tell::Knocked`],
/// a punch landing on someone online. 6, that evening: the islands pulled
/// in to 130 m ([`ARENA_ISLANDS`]); an app with them at 200 m would arrive
/// over the void, fall, and have every move put back. 7, late that
/// evening: the first island the furnished one at +Z, its arrival at its
/// `spawn` marker. 8, at the day's end: every island's arrival at its own
/// marker, all four furnished.
///
/// 9 since the night of 2026-11-07: the town is `roundtownmap.glb` again, with the sea
/// round it and everyone arriving on the path round its pond
/// ([`TOWN_ARRIVAL_RADIUS`]), and the PvP map is a game of its own, Bridge
/// Wars, on [`Venue::Arena`] (Hajun: "i want the default map to open when
/// u open the app to be roundtownmap.glb ... when u press the game icon ui
/// button, it will open a popup game selector menu"); an app with the town
/// on the islands would arrive in the sea.
///
/// 10 since 2026-11-08: Bridge Wars is the server's online (Hajun: "the
/// bridge battle minigame doesnt allow queing with other players"): a game
/// is a room of its own with eight seats in four teams of two ([`ARENA_SEATS`],
/// [`arena_team`]), joined by [`Ask::Arena`] and followed through
/// [`ArenaRound`]; every punch, every hit on a respawn anchor
/// ([`Ask::HitAnchor`]) and every plank of a bridge ([`Ask::Rung`],
/// [`Plank`]) goes through the server, which keeps everyone's health.
///
/// Not bumped for the map of 2026-11-11, whose waiting room is 50 m higher
/// ([`bridge_wars::WAITING_FLOOR`]) and whose anchors stand up to 0.9 m from
/// where they did ([`ARENA_ANCHORS`]): an app with the map before is put
/// 50 m over its own room, which is open to the sky, and falls into it (the
/// server lets any fall go), and the anchors are well inside the slack a
/// hit on one is checked with.
///
/// 11 since the night of 2026-11-11: Bridge Wars has generators, a shop
/// and a timetable (Hajun: "i want to now make the generators functional
/// in the pvpmap ... these generators/items in map need to be server
/// synced"): everything a generator has made lies in the game's room as a
/// [`Loot`], what everyone holds is its [`Purses`], [`Ask::Buy`] and
/// [`Tell::Bought`] are the shop, a game lasts 25 minutes where it lasted
/// 10 ([`bridge_wars::PLAY_SECS`]), and past its eight seats a game may
/// seat zombies ([`bridge_wars::ZOMBIE_TEAM`]), whose seats an app from
/// before would take for a fifth pair of players.
///
/// 12 since late on 2026-11-11: [`Tell::Bought`] of a [`Ware::Bridge`] is 25 m of
/// bridge where it was 50 ([`bridge_wars::BRIDGE_A_BUY`]; Hajun: "make it
/// so 4 silver gets you 25m of bridge"), which an app from before would
/// still count as 50. With it, though no app from before would have
/// minded: a thing a generator makes drops onto its pad and cannot be
/// picked up until it has lain there a moment, and a generator's count to
/// its next thing runs on while it is full ([`bridge_wars::next_at`]), so
/// that every device can write the same count over it.
///
/// Not bumped on 2026-11-12, when a thing came to be picked up as it
/// drops, as soon as it is within reach ([`bridge_wars::REACH_OVER`]),
/// rather than once it has lain a moment: who has what, and when, is the
/// server's to say online, and an app from before shows it as the server
/// says it.
///
/// 13 since 2026-11-12: Bridge Wars' two shops sell swords, shields, things
/// to use and upgrades for a team (Hajun's `bridgewarsgameevent.txt`):
/// [`Ware`] has them all, what everyone has is the game's [`Kits`] and
/// what each team has bought its [`Upgraded`], a fireball in the air is a
/// [`Fireball`], and [`Ask::Use`] uses what is on the second button.
///
/// 14 since 2026-11-13: there are five buttons for things to use where
/// there was one (Hajun: "4 more buttons that are hidden by default until
/// either u press the ... toggle button or u buy an item that fills up
/// another slot ... so u can buy more than 1 item and hold it at once"): a
/// [`Kit`] has a [`Stack`] to each of its [`UTILITY_SLOTS`] where it had
/// one kind and a count, and [`Use::Potion`] says which potion was drunk.
///
/// 15 since 2026-11-13: friends, groups and messages ([`social`]; Hajun: "I
/// want to add a friend system ... every account need a random number
/// between 1000~9999 that is gona serve as a #1234 tag"): [`Ask::Social`]
/// and [`Tell::Social`], which an app from before would not read, and
/// [`Refusal::NotLeader`], as only the leader of a group starts a game.
///
/// And with it, the same day and under the same number, the two having
/// never been out apart: Bridge Wars' dynamite (Hajun's
/// `bridgewarsgameevent.md`: "it will throw an arched dynamite ... and
/// destroy that blast radius region of the bridge"). [`Utility::Dynamite`]
/// is a sixth thing for the buttons, which an app from before could not
/// read in a [`Kit`]; a stick in the air or lying is a [`Dynamite`] and
/// where one blew, with the planks it took, a [`Blasted`], both registered
/// after [`Fireball`]; [`Use::Dynamite`] throws one; and the healing
/// potion gives back half of your health where it gave a quarter
/// ([`bridge_wars::HEALING`]). Either of the two changes alone would have
/// needed the number raised: should one ever go out without the other,
/// raise it again.
///
/// 16 since 2026-11-14 (Hajun: "implement the new items in the shop listed
/// in the bridgewarsgameevent.md (leap potion and bridge speed upgrade)",
/// "I think there should be a global chat", "The bridge could also be a
/// little wider, maybe 1.25 times its current width"): [`Utility::Leap`] is
/// a seventh thing for the buttons and a [`Kit`] has until when its owner
/// leaps ([`Kit::leap_until`]); an [`Upgrades`] has how far its team's
/// building has been quickened ([`Upgrades::build_speed`],
/// [`Ware::BuildSpeed`]); what is said in a room, and what happens in a
/// game of Bridge Wars, is told to everyone in it ([`chat`]:
/// [`Ask::Chat`], [`Tell::Said`], [`Tell::Feed`]); and a plank is a quarter
/// as wide and as long again ([`nav::PLANK_HALF_WIDTH`]), which the
/// server's bots walk and the devices draw by the same measure.
///
/// Still 16 with a plank a tenth as wide again, not a quarter, since the
/// night of 2026-11-14, and with whoever kills someone getting their
/// sapphire and amethyst: a rung is sent as it was, its middle and its
/// heading, and a purse as it was, so a device from before plays on, and
/// draws every bridge a little wider than the server walks it.
///
/// 17 since 2026-11-15 (Hajun: "if u exit the app while ur playing and
/// reopen, u shouldnt go to the main lobby - u should rejoin the game as
/// long as there is atleast 1 real player in that game still", and of the
/// dynamite, "from that point it will destroy cascadingly the entire
/// bridge ... but it shouldnt destroy the whole bridge instantly"): a seat
/// in a game of Bridge Wars waits for whoever's connection has gone, and
/// they are put back in it ([`Tell::Back`], with what only their own
/// device knew of it, which it now says as it changes, [`Ask::Metres`]);
/// and a [`Blasted`] says when it blew, by the server's clock, and which
/// planks of the bridge fall after the ones the blast took, each how long
/// after ([`Blasted::later`], [`nav::Nav::cascade`]).
///
/// 18 since the night of 2026-11-15 (Hajun: "if its a land-hp kill where
/// they die on land, they should drop the sapphires and amethyst like
/// small scatter explosion from their dead body location ... and u only
/// get it if u pick it up"): a [`Loot`] says whether someone who died
/// dropped it, and if so from where, onto what, and how many of its item
/// it is ([`Spilt`]), which an app from before cannot read. (The dynamite
/// at 1 amethyst, and its falling quickening as it goes, of the same
/// night, changed nothing that is sent.)
///
/// 19 since 2026-11-16 (Hajun: "in the bridge wars minigame map, i want 2
/// buses to go 2x normal speed around the road. and when a person is
/// standing in front of the moving bus, the bus 'squishes' that person, so
/// the person is squashed on the road for 10 second"): two buses drive
/// round the middle of Bridge Wars' map by the clock
/// ([`bridge_wars::Bus`]), which an app from before does not draw, stand
/// on or get out of the way of; a game says who lies flat and until when
/// ([`Flattened`], registered after [`BuilderOf`]); a device says a bus has
/// run its own player over ([`Ask::RunOver`]); a place on one of those
/// buses is a place on a car numbered past the town's
/// ([`bridge_wars::FIRST_BUS`]); and the chat is told who a bus killed
/// ([`chat::Feed::RunOver`]).
///
/// 20 since the night of 2026-11-16 (Hajun, having looked on at a game
/// through its fifteenth minute: "i want it to be 20min instead of 25min
/// ... at 14:00 is when the spawn anchors break, and 15:00 is when the
/// zombies spawn all over the map, starting slow, and more and more spawn,
/// exponentially until like maybe max 50 zombies ... zombie should be
/// invincible, and maybe deal like 20 dmg per hit"): nothing that is sent
/// has changed its shape, but the rules both sides go by have. A game is
/// twenty minutes ([`bridge_wars::PLAY_SECS`]) and its anchors break at
/// fourteen ([`bridge_wars::TIMETABLE`]), which an app from before would
/// say at ten, with a clock five minutes out; and its zombies are up to
/// fifty, in seats an app from before has no place for past the twelfth
/// ([`bridge_wars::ZOMBIES`], [`bridge_wars::zombie_spot`]).
///
/// 21 since 2026-11-17 (Hajun: "i want 4 bus, one in each lane (there are
/// 4 lanes) at varying intervals, and the bus will stop before the
/// crosswalk if there is a person on the crosswalk until they cross it"):
/// a game's buses are four, one to each lane of the ring, and go no longer
/// by the clock alone but as the game has them going, which it sends
/// whenever that changes ([`bridge_wars::Buses`], registered after
/// [`Flattened`]); a place on one of them is a place on a car numbered up
/// to seven, past the town's four.
///
/// 22 since 2026-11-18 (Hajun: "add these minigames: Blend In, Knock Off,
/// Bridge Race, Last Stand"): four new games, each at a table ([`table`]).
/// Four venues, after [`Venue::Arena`] ([`Venue::BlendIn`] and the rest),
/// which an app from before cannot read; four [`Minigame`]s, which a
/// friend's presence sends ([`social::Presence::Playing`]), so that an app
/// from before could not read what it is told of its friends while one of
/// them sat at a table; the join ([`Ask::Join`], answered by
/// [`Tell::Entered`]) and each game's own asks and tells ([`Ask::BlendIn`],
/// [`Tell::BlendIn`] and the rest), appended after them; the board
/// everyone at a table is sent ([`table::Board`], registered after
/// [`bridge_wars::Buses`]); and each game's own components, which each
/// game registers after the board, in its own module (`blend_in::register`
/// and the rest, in that order).
///
/// 23 since the evening of 2026-11-18 (Hajun, after playing the four):
/// Last Stand's sword dropped by the dead and its treasure chests lying
/// about, each a component of its own registered after its spit
/// ([`last_stand::LyingSword`], [`last_stand::Chest`]), a chest opened
/// ([`last_stand::Ask::Open`]) and what came out of it told
/// ([`last_stand::Tell::Found`]); its map over the sea where it was over
/// the void, whose rules an app from before would not play by; Bridge Wars
/// with a fee ([`bridge_wars::FEE`]); and Blend In's seekers on the speed
/// potion's pace ([`blend_in::SEEKER_PACE`]).
///
/// 24 since later that evening (Hajun's next message): Last Stand's
/// treasure chests struck open ([`last_stand::Ask::Hit`], appended after
/// the bubble's `Open`; [`last_stand::Chest::hits`]); Bridge Race's round
/// saying what the next grant brings ([`bridge_race::RaceRound::next`],
/// [`bridge_race::RaceRound::next_sticks`]), its grants of a size thrown
/// for each and coming every 12 s, and its blasts bringing the rest of the
/// bridge down after them, which an app from before would not play by.
///
/// 25 since 2026-11-23 (Hajun: "i want to change it so that at all times,
/// there are 48 plots filled"): a seat of the town is one of its 48 plots
/// now, in his own order ([`land::HOMES`]), where it was one of sixteen
/// that each device worked a plot out for; so an app from before would
/// stand everyone's land on another plot than the server has them on.
/// And with it [`Tell::Plots`], whose every plot of the channel is, and a
/// group gathered into one channel ([`social::Outcome::Gathered`]).
///
/// 26 since 2026-11-27 (Hajun: "each game, a truly random shape of island
/// forms ... i dont want fixed starting positions"): Knock Off is played on
/// an island thrown for its table ([`knock_off::Island`], a component sent
/// its room, registered after [`knock_off::KnockRound`]), whose every slide
/// an app from before would play on the round island; its round says how big
/// the ice is as a share of the island ([`knock_off::KnockRound::size`]),
/// where it said how far it reached; and everyone begins where they stood.
///
/// 27 since later that day (Hajun: "im gona disable custom text chat
/// throughout the game in both minigames and in the lobby. there is only
/// gona be quickchat with the chatbubble"): what someone said is a line
/// typed or one of the things to say at a press, by its number
/// ([`Tell::Said`], [`chat::Said`]), where it was the words alone, and the
/// server passes on no line typed ([`chat::TYPED`]); with them, houses are
/// visited and rated ([`house`], [`Ask::House`], [`Tell::House`]).
pub const PROTOCOL_VERSION: u32 = 27;

/// The UDP port the game server listens on. 443, where QUIC goes, because a
/// network that lets any UDP through lets that through: school and office
/// networks often stop the rest.
pub const GAME_PORT: u16 = 443;

/// How many times a second the server sends what has changed, and how many
/// times a second a device sends its own position while it moves.
pub const TICK_HZ: f64 = 15.0;

/// How many players a town channel holds: one to each of the town's plots
/// ([`land::PLOTS`], 48) since 2026-11-24 (Hajun: "channel size for real
/// players should be increased from 16 to 48"); 16 until then
/// (`MULTIPLAYER.md`, "How many to a room": 16 to start).
pub const TOWN_SEATS: usize = land::PLOTS;

/// How many arrive on one ring round the middle of an island ([`arrival`]):
/// the seventeenth is on a ring further out. The town's own people arrive
/// on their plots ([`land::arrival`]); the path round its pond is where
/// Blend In's crowd is walked from.
pub const RING_SPOTS: usize = 16;

/// How many players a game of House Builder holds, bots included.
pub const GAME_SEATS: usize = 8;

/// How many players a game of Bridge Wars holds, bots included: two to each
/// of the four islands round the PvP map ([`ARENA_ISLANDS`]), a team to an
/// island ([`arena_team`]; since 2026-11-08, Hajun: "make it 8 player
/// rather than 4 player, so its 2v2v2v2"; one to an island until then).
pub const ARENA_SEATS: usize = 8;

/// How many teams a game of Bridge Wars has: one to an island.
pub const ARENA_TEAMS: usize = 4;

/// How many people the town has in it, the computer's among them: one to
/// each of its 48 plots ([`land::PLOTS`]), whoever is in your channel on
/// theirs and someone of the computer's on each of the rest, under its
/// owner's name (the game's `resident`; Hajun, 2026-11-24: "there should
/// be a character for all 48 plot not just the first 16 ... if theres just
/// 1 real player online in a channel, then there should be 47 npcs, each
/// on their plot"). Each device walks its own, as it always has (his
/// choice, 2026-10-08). Sixteen for the day before, one to each of the
/// first sixteen plots; 32 from 2026-11-01 until 2026-11-23, the
/// computer's wandering round the pond; 8 before. Not part of the
/// protocol: the server never counts them.
pub const TOWN_BODIES: usize = TOWN_SEATS;

/// The look a new account wears until its owner has chosen one of
/// [`PEOPLE_LOOKS`] (the game's `signup`, [`Ask::Profile`]), which the game
/// draws as the first of them. Every account from before there was a choice
/// (2026-10-31) wears it. Bots wear others.
pub const PLAYER_LOOK: &str = "blue";

/// The two sides a person chooses between on signing up, a boy and a girl:
/// the first look of [`BOY_LOOKS`] and of [`GIRL_LOOKS`], which were the
/// only two until 2026-11-02 (the game's `limbperson.glb` and
/// `limbpersongirl.glb` until 2026-11-01, then its `lobby::MODEL` and
/// `GIRL_MODEL`).
pub const PEOPLE_LOOKS: [&str; 2] = ["boy", "girl"];

/// Every look there is, by side: the boy and the girl, and Hajun's
/// recolourings of each (2026-11-02: "use variations of the colors. i made
/// various colors for both guy and girl for the ais"; and the same night,
/// for people too: "put some arrow thingy next to each guy and girl
/// character selection in the start so u can choose which color guy or
/// which color girl"). A person picks any of them on signing up
/// ([`profile::look`]); a bot is a boy or a girl by the toss of a coin and
/// then one of that side at random ([`ai_look`]), on the device for its own
/// AIs and on the server for House Builder's. The game maps each to its
/// model (`lobby::LOOKS`); an app that has not heard of one draws the boy.
pub const BOY_LOOKS: [&str; 3] = ["boy", "boy_blackbrown", "boy_whitejeans"];
pub const GIRL_LOOKS: [&str; 3] = ["girl", "girl_whiteblue", "girl_yellowpink"];

/// What the computer's players are called, the boys and the girls (Hajun,
/// 2026-11-02: "use the more manly ai names for the boy ais and girly cute
/// ai names for the girl ais from the names you already made"): the 48
/// names the game's AIs went by, split two ways. Players online go by the
/// same names with a number after (the server's).
pub const BOY_NAMES: [&str; 24] = [
    "Hodu", "Gamja", "Mandu", "Kimbap", "Maru", "Kong", "Hobak", "Bam", "Dori", "Bada", "Gureum",
    "Dongi", "Tori", "Jjang", "Pado", "Choco", "Kkami", "Ddol", "Ssal", "Bokki", "Tangi", "Sol",
    "Nuri", "Saem",
];
pub const GIRL_NAMES: [&str; 24] = [
    "Mochi", "Dubu", "Bori", "Yuja", "Nabi", "Podo", "Sagwa", "Gyul", "Dalgi", "Haru", "Byeol",
    "Bomi", "Nana", "Mint", "Coco", "Latte", "Bbori", "Yeppi", "Ggul", "Sseul", "Jelly", "Pudding",
    "Cookie", "Waffle",
];

/// One of the computer's looks: a boy if `boy`, else a girl, and of that
/// side the `pick`th, round and round. Both are the caller's to draw at
/// random, so that this crate needs no dice of its own.
pub fn ai_look(boy: bool, pick: usize) -> &'static str {
    let side: &[&str] = if boy { &BOY_LOOKS } else { &GIRL_LOOKS };
    side[pick % side.len()]
}

/// The names the computer's players of one side go by.
pub fn ai_names(boy: bool) -> &'static [&'static str] {
    if boy { &BOY_NAMES } else { &GIRL_NAMES }
}

/// What can be written: the letters and signs a name, a line of chat and a
/// message to a friend are made of, as both sides check them. Since
/// 2026-11-14 that is English, Korean and Vietnamese (Hajun: "i need korean
/// text to be able to work in the global chat not just english. same for
/// username and send message", and "also vietnamese text as well"); until
/// then plain ASCII, the game's font having no Hangul.
///
/// It is what the game's fonts draw (the game's `font`: Inter, and
/// Pretendard for Hangul), no more: what is on an English keyboard; the
/// accented letters of western Europe (Latin-1), which Vietnamese shares;
/// the rest of Vietnamese's letters, each vowel with each tone; the curly
/// quotes, dashes and ellipsis a phone's keyboard writes by itself; and
/// Hangul, every syllable and the letters they are made of, as they are
/// typed alone ("ㅋㅋ"). Nothing else: no emoji, no Hanja, no other
/// script.
///
/// **Lengths are counted in widths** ([`length`]): a Hangul syllable is
/// two, being about as wide as two Latin letters, so that a name or a line
/// is as long on the screen in either. A name of 16 is 16 Latin letters or
/// 8 syllables.
///
/// **A letter is the same letter however it was typed** ([`clean`]): some
/// keyboards send a Vietnamese vowel and its tone mark as two, or a Hangul
/// syllable as its two or three parts, and they are put together
/// (Unicode's NFC), so that a name looked for is found whichever keyboard
/// wrote it.
///
/// **Common swear words are starred out of what people say to each other**
/// ([`masked`], since 2026-11-14; Hajun: "add basic word filter for common
/// swear words so they turn to ****** of that word's length"): a line of
/// chat and a message to a friend, as both sides clean them, so that the
/// server does it to whatever it is sent. **And a name with one in it is no
/// name** ([`swears`], `profile::name`; Hajun, later that evening: "yes add
/// the username filter too"): stars would be no name to go by, so it is
/// refused, and another has to be picked.
pub mod text {
    use unicode_normalization::UnicodeNormalization;

    /// Whether `c` is something the game writes: see the module.
    pub fn allowed(c: char) -> bool {
        matches!(
            c,
            // What is on an English keyboard, and a space.
            ' '..='~'
            // Latin-1's letters and signs, but the soft hyphen.
            | '\u{A1}'..='\u{AC}' | '\u{AE}'..='\u{FF}'
            // Vietnamese's own: A with a breve, D with a stroke, I and U
            // with a tilde, O and U with a horn, capital and small; and
            // every vowel with every tone.
            | '\u{102}' | '\u{103}' | '\u{110}' | '\u{111}' | '\u{128}' | '\u{129}' | '\u{168}' | '\u{169}'
            | '\u{1A0}' | '\u{1A1}' | '\u{1AF}' | '\u{1B0}'
            | '\u{1EA0}'..='\u{1EF9}'
            // An en dash and an em dash, curly quotes, an ellipsis.
            | '\u{2013}' | '\u{2014}' | '\u{2018}' | '\u{2019}' | '\u{201C}' | '\u{201D}' | '\u{2026}'
            // Hangul: its letters as typed alone, and every syllable.
            | '\u{3131}'..='\u{3163}'
            | '\u{AC00}'..='\u{D7A3}'
        )
    }

    /// Whether `c` is a piece of a letter, as some keyboards send one: a
    /// mark that goes on the vowel before it (Vietnamese's five tones, and
    /// the circumflex, the breve and the horn), or one of the parts of a
    /// Hangul syllable. [`clean`] puts the pieces together; any left over
    /// is no letter, and is left out.
    pub fn piece(c: char) -> bool {
        matches!(c, '\u{300}'..='\u{303}' | '\u{306}' | '\u{309}' | '\u{31B}' | '\u{323}' | '\u{1100}'..='\u{11FF}')
    }

    /// Whether `c` may go into a box that is typed into: what is
    /// [`allowed`], and the [`piece`]s of it on their way to being put
    /// together.
    pub fn typed(c: char) -> bool {
        allowed(c) || piece(c)
    }

    /// How wide `c` is, in the widths lengths are counted in: a Hangul
    /// syllable or letter is two; a mark on the letter before it, and the
    /// second and third parts of a syllable typed in parts, none; anything
    /// else one.
    pub fn width(c: char) -> usize {
        match c {
            '\u{300}'..='\u{36F}' | '\u{1160}'..='\u{11FF}' => 0,
            '\u{1100}'..='\u{115F}' | '\u{3131}'..='\u{3163}' | '\u{AC00}'..='\u{D7A3}' => 2,
            _ => 1,
        }
    }

    /// How long `text` is, in those widths.
    pub fn length(text: &str) -> usize {
        text.chars().map(width).sum()
    }

    /// `text` as it is kept and sent: every letter put together from its
    /// pieces, a new line or a tab or any other space a plain space, what
    /// the game does not write left out, and the spaces round it trimmed.
    /// Cleaning what is clean changes nothing, so both sides can.
    pub fn clean(text: &str) -> String {
        let whole: String = text
            .nfc()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .filter(|c| allowed(*c))
            .collect();
        whole.trim().to_string()
    }

    /// `typed` with every letter put together from its pieces, and nothing
    /// else changed: what a box shows while it is typed into, before it is
    /// [`clean`]ed.
    pub fn whole(typed: &str) -> String {
        typed.nfc().collect()
    }

    /// As much of the front of `text` as is no longer than `most` widths.
    pub fn fitted(text: &str, most: usize) -> &str {
        let mut long = 0;
        for (at, c) in text.char_indices() {
            long += width(c);
            if long > most {
                return &text[..at];
            }
        }
        text
    }

    /// English swear words that are masked wherever they are in a word,
    /// the whole word with them ("motherfucker", "bullshit"), each written
    /// with no letter doubled, as a word is compared once its own doubled
    /// letters are taken out ("fuuuck", "shiiit"). Only ones no everyday
    /// word has in it.
    const STEMS: &[&str] = &["fuck", "shit", "bitch", "ashole", "arsehole", "bastard", "fagot", "whore"];
    /// Swear words that are masked only as a whole word, whatever their
    /// capitals: English ones that are part of everyday words ("class",
    /// "cockpit", "Dickens"), and Vietnamese ones, each a word of its own
    /// as Vietnamese is written, with the short forms they are typed in.
    /// Only as they are spelt with their marks: without them most are
    /// everyday words ("cac ban" is "các bạn").
    const WORDS: &[&str] = &[
        "ass", "asses", "arse", "dick", "dicks", "dickhead", "dickheads", "cock", "cocks", "cocksucker",
        "cocksuckers", "pussy", "pussies", "cunt", "cunts", "slut", "sluts", "twat", "twats", "wanker",
        "wankers", "nigger", "niggers", "nigga", "niggas", "fag", "fags",
        "địt", "đụ", "đù", "đéo", "lồn", "cặc", "buồi", "đĩ", "loz",
        "đm", "đmm", "dmm", "đcm", "dcm", "đkm", "vcl", "vkl", "vl", "clgt",
    ];
    /// Korean swear words, masked wherever they are in a run of Hangul,
    /// Korean being written with few spaces: themselves and no more of
    /// what they are in. With the letters alone they are shortened to.
    const HANGUL: &[&str] = &[
        "시발", "씨발", "씨빨", "씨팔", "시팔", "씨바", "ㅅㅂ", "ㅆㅂ", "병신", "ㅂㅅ", "좆", "좃", "존나", "ㅈㄴ",
        "개새끼", "개새", "개색", "미친새끼", "미친놈", "미친년", "씹새", "씹년", "지랄", "ㅈㄹ", "느금마", "니애미",
        "창녀",
    ];
    /// Everyday Korean that has one of those in it, and is left alone:
    /// "다시 발견" written without its space, a starting point, a first
    /// station.
    const HANGUL_FINE: &[&str] = &["다시발", "시발점", "시발역"];

    /// Whether `c` is Hangul, a syllable or a letter alone.
    fn hangul(c: char) -> bool {
        matches!(c, '\u{3131}'..='\u{3163}' | '\u{AC00}'..='\u{D7A3}')
    }

    /// `word` with each run of one letter over and over as that letter
    /// once.
    fn squeezed(word: &str) -> String {
        let mut out = String::new();
        for c in word.chars() {
            if !out.ends_with(c) {
                out.push(c);
            }
        }
        out
    }

    /// Where `needle` is in `run`, by where each begins.
    fn found<'a>(run: &'a [char], needle: &'a str) -> impl Iterator<Item = (usize, usize)> + 'a {
        let needle: Vec<char> = needle.chars().collect();
        let long = needle.len();
        (0..run.len().saturating_sub(long.saturating_sub(1)))
            .filter(move |&at| long > 0 && run[at..at + long] == needle[..])
            .map(move |at| (at, long))
    }

    /// `text` with the common swear words in it starred out, a star to
    /// each letter, so that it is as long as it was (in letters: a Hangul
    /// syllable is one star). **A basic filter**, for a chat anyone may
    /// read: the commonest words of English, Korean and Vietnamese, as
    /// they are plainly spelt. An English or a Vietnamese word is masked
    /// whole, if it is one of [`WORDS`] or has one of [`STEMS`] in it;
    /// Korean, which runs its words together, has each of [`HANGUL`]
    /// masked where it stands. It does not look for a word spelt with
    /// digits or signs, or with spaces between its letters, and whoever
    /// means to get a word past it will. What is masked stays masked:
    /// both sides do this, the server to whatever it is sent.
    pub fn masked(text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut hide = vec![false; chars.len()];
        let mut at = 0;
        while at < chars.len() {
            // A run of Hangul, or a run of other letters: a word.
            let korean = hangul(chars[at]);
            if !korean && !chars[at].is_alphabetic() {
                at += 1;
                continue;
            }
            let mut end = at;
            while end < chars.len() && hangul(chars[end]) == korean && (korean || chars[end].is_alphabetic()) {
                end += 1;
            }
            let run = &chars[at..end];
            if korean {
                let fine: Vec<(usize, usize)> = HANGUL_FINE.iter().flat_map(|fine| found(run, fine)).collect();
                for swear in HANGUL {
                    for (begins, long) in found(run, swear) {
                        let let_be = fine.iter().any(|&(from, span)| from <= begins && begins + long <= from + span);
                        if !let_be {
                            hide[at + begins..at + begins + long].fill(true);
                        }
                    }
                }
            } else if swear_word(run) {
                hide[at..end].fill(true);
            } else {
                // Words run together with a capital to begin each, as a
                // name often is ("BigDick"): each by itself.
                let mut from = 0;
                for to in 1..=run.len() {
                    let ends = to == run.len() || (run[to - 1].is_lowercase() && run[to].is_uppercase());
                    if ends {
                        if swear_word(&run[from..to]) {
                            hide[at + from..at + to].fill(true);
                        }
                        from = to;
                    }
                }
            }
            at = end;
        }
        chars.iter().zip(hide).map(|(c, hide)| if hide { '*' } else { *c }).collect()
    }

    /// Whether `word`, a run of letters that are not Hangul, is a swear
    /// word: one of [`WORDS`], whatever its capitals, or with one of
    /// [`STEMS`] in it once its doubled letters are taken out.
    fn swear_word(word: &[char]) -> bool {
        let word: String = word.iter().flat_map(|c| c.to_lowercase()).collect();
        WORDS.contains(&word.as_str()) || {
            let squeezed = squeezed(&word);
            STEMS.iter().any(|stem| squeezed.contains(stem))
        }
    }

    /// Whether `text` has a common swear word in it, which [`masked`]
    /// would star out.
    pub fn swears(text: &str) -> bool {
        masked(text) != text
    }
}

/// What a person says about themselves on signing up, as both sides check
/// it: a name to go by, and which of [`PEOPLE_LOOKS`] they wear.
pub mod profile {
    use super::text;

    /// How long a name may be, in widths ([`text::length`]): 2 to 16 Latin
    /// letters, or 1 to 8 Hangul syllables, or as much of both.
    pub const SHORTEST: usize = 2;
    pub const LONGEST: usize = 16;

    /// `text` as a name, if it is one: [`SHORTEST`] to [`LONGEST`] wide
    /// once it is cleaned ([`text::clean`]: its letters put together, what
    /// the game does not write left out, the spaces round it trimmed), and
    /// with no common swear word in it ([`text::swears`], since
    /// 2026-11-14). The server goes by this too, and takes no other.
    pub fn name(text: &str) -> Option<String> {
        let name = text::clean(text);
        ((SHORTEST..=LONGEST).contains(&text::length(&name)) && !text::swears(&name)).then_some(name)
    }

    /// Whether `text` would be a name but for a swear word in it: what
    /// whoever typed it is told, to pick another, where anything else that
    /// is no name is only too short or too long.
    pub fn rude(text: &str) -> bool {
        let name = text::clean(text);
        (SHORTEST..=LONGEST).contains(&text::length(&name)) && text::swears(&name)
    }

    /// Whether `c` may be in a name: what the game writes ([`text`]).
    /// English, Korean and Vietnamese since 2026-11-14; plain ASCII until
    /// then.
    pub fn allowed(c: char) -> bool {
        text::allowed(c)
    }

    /// Whether `look` is one a person may choose: any of either side's
    /// (since 2026-11-02; the first of each until then).
    pub fn look(look: &str) -> bool {
        super::BOY_LOOKS.contains(&look) || super::GIRL_LOOKS.contains(&look)
    }
}

/// Registers everything both sides send, in one order for both. Call it after
/// `RepliconPlugins` and before the renet client or server is made: its
/// channels are counted from what is registered here.
pub fn protocol(app: &mut App) {
    app.replicate::<Member>()
        .replicate::<Venue>()
        .replicate_with(RuleFns::new(default_serialize::<Spot>, deserialize_spot))
        .replicate::<Round>()
        .replicate::<Seats>()
        .replicate::<Built>()
        .replicate::<ArenaRound>()
        .replicate::<Plank>()
        .replicate::<Loot>()
        .replicate::<Purses>()
        .replicate::<Kits>()
        .replicate::<Upgraded>()
        .replicate::<Fireball>()
        .replicate::<Dynamite>()
        .replicate::<Blasted>()
        .replicate::<BuilderOf>()
        .replicate::<Flattened>()
        .replicate::<bridge_wars::Buses>()
        .replicate::<table::Board>();
    // The four games of 2026-11-18 register their own, each in its own
    // module, in this order.
    blend_in::register(app);
    knock_off::register(app);
    bridge_race::register(app);
    last_stand::register(app);
    app.add_client_message::<Moved>(Channel::Unreliable)
        .add_client_message::<Ask>(Channel::Ordered)
        .add_server_message::<Tell>(Channel::Ordered)
        .add_client_message::<WhatTime>(Channel::Unreliable)
        .add_server_message::<TheTime>(Channel::Unreliable)
        .make_message_independent::<TheTime>();
    app.world_mut()
        .resource_mut::<ProtocolHasher>()
        .add_custom(PROTOCOL_VERSION);
}

// ---------------------------------------------------------------- islands

/// Which island something is on. The game's `island::Venue` is this.
///
/// Every island stands at the origin, in the same place: what keeps them
/// apart is this, carried by everything that moves or collides.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Venue {
    /// `roundtownmap.glb`: Round Town, round a pond with a fountain in it,
    /// where everyone is.
    Town,
    /// Your home, where nobody else goes for now.
    Home,
    /// `lobbymap.glb`: where the players of a game of House Builder wait for
    /// it to fill.
    Lobby,
    /// One builder's own copy of `homemap.glb` in a game of House Builder, by
    /// their seat in the game.
    Plot(u8),
    /// `pvptownmap.glb`: Bridge Wars' map, Round Town's middle in the sky
    /// with four islands at the ends of its bridges ([`ARENA_ISLANDS`]),
    /// where a game of Bridge Wars is played (since the night of 2026-11-07; the town
    /// itself on 2026-11-07). The game is the device's own for now, played
    /// with bots: online you are in a room of your own there, as at home,
    /// and nothing of where you stand is sent.
    Arena,
    /// `roundtownmap.glb`, the town's own map, for a game of Blend In
    /// ([`blend_in`], since 2026-11-18): its scenery, none of its people. A
    /// venue of its own, so that the players of a game, and the crowd its
    /// hider passes among, meet nobody in the town, nor the town them.
    BlendIn,
    /// `iceisland.glb`: Knock Off's island of ice in the sea
    /// ([`knock_off`], since 2026-11-18).
    KnockOff,
    /// `bridgeracemap.glb`: Bridge Race's two team islands and the golden
    /// island, over the void ([`bridge_race`], since 2026-11-18).
    BridgeRace,
    /// `laststandmap.glb`: Last Stand's stone and sand round the anchor,
    /// over the void ([`last_stand`], since 2026-11-18).
    LastStand,
}

impl Venue {
    /// The game of the four (2026-11-18) played on this venue, if it is one
    /// of theirs: none for the town, home, House Builder's venues or Bridge
    /// Wars', whose games are not played at tables ([`Minigame::venue`]
    /// goes the other way, for all six).
    pub fn game(self) -> Option<Minigame> {
        match self {
            Venue::BlendIn => Some(Minigame::BlendIn),
            Venue::KnockOff => Some(Minigame::KnockOff),
            Venue::BridgeRace => Some(Minigame::BridgeRace),
            Venue::LastStand => Some(Minigame::LastStand),
            Venue::Town | Venue::Home | Venue::Lobby | Venue::Plot(_) | Venue::Arena => None,
        }
    }
}

/// How far from the middle everyone stands on arriving on an island, in
/// metres: well inside the grass of the home and the lobby, which reaches 20 m
/// out. The town has its own ([`TOWN_ARRIVAL_RADIUS`]).
pub const ARRIVAL_RADIUS: f32 = 6.0;

/// How far from the middle everyone stands on arriving in the town, in
/// metres: on the middle of the path, 30 to 33 m out, round the pond that
/// fills the middle of `roundtownmap.glb`, where the town's own name shows.
/// Since 2026-10-30, but for 2026-11-07, when the town was the PvP map and
/// everyone arrived on its islands ([`ARENA_ISLANDS`]).
pub const TOWN_ARRIVAL_RADIUS: f32 = 31.5;

/// The arrival points of the four islands at the ends of the PvP map's
/// bridges ([`Venue::Arena`]), in metres from the middle of the map, where
/// everyone arrives in a game of Bridge Wars (Hajun, 2026-11-07, of the map
/// as the town: "make the player start in the center of one of the 4
/// bridge connected island"): the islands 130 m out along each axis, each
/// 20.8 m across (the islands' object scaled to 0.65 across the ground in
/// their map, its array of four with it; 200 m out and 32 m across that
/// morning), with a bridge 3.5 m wide from each to the middle, 75 m out,
/// until 2026-11-08, since when every bridge is built by walking; the
/// map's four smaller islands, 24 m across (`SapphireIslands`), are 125 m
/// out between them, on the diagonals. Each island's point is its middle (since the night of 2026-11-07, Hajun:
/// "make the spawnpoint in the center of the island so that u dont need
/// to hide the purple rotated cube"; for the evening before, the `spawn`
/// marker each island has, a purple pillar 5 to 6 m toward the middle,
/// which the game hid and now draws as it is): first the island at the
/// game's +Z (the bottom of the screen; Blender's +Y), then the others
/// clockwise from it seen from above. The islands' object is an array of
/// four about the map's middle, 130 m out on each axis (its mesh reaches
/// 140.4 m, the islands being 20.8 m across). Since Hajun's map of
/// 2026-11-11 the middle of each island is open ground between two market
/// stands, 4.65 m either side of it across the island (a desk lay across
/// it until then, which shoved whoever arrived to its edge), with the
/// anchor's slab 5 m toward the town and a low pad, `gen1` to `gen4`, 5 m
/// the other way. The server, which has no map, goes by this too.
pub const ARENA_ISLANDS: [Vec2; 4] = [
    Vec2::new(0.0, 130.0),
    Vec2::new(-130.0, 0.0),
    Vec2::new(0.0, -130.0),
    Vec2::new(130.0, 0.0),
];

/// How far from its island's middle everyone on it but the first stands on
/// arriving, in metres: round the one in the middle, a few metres apart.
pub const ISLAND_RING: f32 = 4.0;

/// How far across the ground each of the [`ARENA_ISLANDS`] reaches from its
/// middle, in metres: Hajun's islands are discs 20.8 m across (their object
/// scaled to 0.65). The server keeps its bots on them with this.
pub const ARENA_ISLAND_RADIUS: f32 = 10.4;

/// Where each team's respawn anchor stands, in metres from the middle of the
/// map, by team ([`arena_team`]): the purple pillar in the material `spawn`
/// on each island, 6 m from the island's middle toward the town, as
/// Hajun's `pvptownmap.glb` has them (a test in the game reads them off the
/// map). While it stands, whoever on its team dies comes back; broken, they
/// are out for good once they die (since 2026-11-08, Hajun: "make the purple
/// rotated cube thingy on each island the respawn anchor (similar to bed in
/// bedwars)"). As the map of 2026-11-11 has them, each a quarter turn on
/// from the last about the map's middle (the objects `spawn` to
/// `spawn.003`); until then, placed one by one, they were (0, 124.92),
/// (-123.84, 0.03), (-0.16, -124) and (123.83, -0.13).
pub const ARENA_ANCHORS: [Vec2; ARENA_TEAMS] = [
    Vec2::new(0.16, 124.0),
    Vec2::new(-124.0, 0.16),
    Vec2::new(-0.16, -124.0),
    Vec2::new(124.0, -0.16),
];

/// How far out from its middle an anchor reaches, across the ground, in
/// metres: the pillar is two pyramids base to base, 1.41 m square where
/// they meet, 0.71 m out at its sides and 1 m at its corners (a cube 1.4 m
/// across turned on its corner until 2026-11-11, about as wide). A punch
/// lands on it as on a body this wide.
pub const ANCHOR_RADIUS: f32 = 0.85;

/// Where the generators' pads are, in metres from the middle of the map, as
/// Hajun's `pvptownmap.glb` of 2026-11-11 has them (a test in the game
/// reads them off the map): the stone pad at the back of each team's
/// island, by team (the objects `gen1` to `gen4`), where its silver and its
/// gold come up; the pad in the middle of each of the four smaller islands
/// on the diagonals (`sapphiregen` to `sapphiregen4`), for sapphire; and
/// the four in the middle of the map, 35.8 m out on the axes, by the team
/// whose island each is toward (`amethystgen` to `.003`), for amethyst
/// (Hajun's `bridgewarsgameevent.txt`: "Silver and Gold spawn in each
/// player's personal island's square pad. Sapphire spawn in the 4 corner
/// islands's square pad. Amethyst spawn in the center map's 4 square
/// pads."). Each pad is 2 m square with its top [`PAD_TOP`] up.
pub const ARENA_PADS: [Vec2; ARENA_TEAMS] = [
    Vec2::new(0.16, 135.92),
    Vec2::new(-135.92, 0.16),
    Vec2::new(-0.16, -135.92),
    Vec2::new(135.92, -0.16),
];
pub const SAPPHIRE_PADS: [Vec2; 4] = [
    Vec2::new(88.39, 88.39),
    Vec2::new(-88.39, 88.39),
    Vec2::new(-88.39, -88.39),
    Vec2::new(88.39, -88.39),
];
pub const AMETHYST_PADS: [Vec2; 4] = [
    Vec2::new(0.16, 35.82),
    Vec2::new(-35.82, 0.16),
    Vec2::new(-0.16, -35.82),
    Vec2::new(35.82, -0.16),
];

/// How high a pad's top is, in metres, and how far it reaches from its
/// middle each way across the ground: a slab 2 m square, 0.25 m over the
/// island it lies on.
pub const PAD_TOP: f32 = 0.5;
pub const PAD_HALF: f32 = 1.0;

/// How far each of the four smaller islands reaches from its middle, in
/// metres: 24 m across (the map's `SapphireIslands`), its pad in the
/// middle of it.
pub const SAPPHIRE_ISLAND_RADIUS: f32 = 12.0;

/// Where each team's shop is, in metres from the middle of the map, by
/// team: the middle of the outline on the ground in front of the
/// red-roofed market stand on its island (the map's `activation`
/// material), to the right of whoever arrives there, 5 m from the island's
/// middle. Standing in the outline opens the shop on the device (the
/// game's `market`, which reads the outlines off the map); the server,
/// which has no map, takes a purchase only from someone near one of these
/// ([`bridge_wars::SHOP_SLACK`]). A test in the game keeps them where the
/// map has them.
pub const ARENA_SHOPS: [Vec2; ARENA_TEAMS] = [
    Vec2::new(5.0, 130.13),
    Vec2::new(-130.13, 5.0),
    Vec2::new(-5.0, -130.13),
    Vec2::new(130.13, -5.0),
];

/// Where each team's upgrade shop is, as [`ARENA_SHOPS`] are: the middle
/// of the outline in front of the blue-roofed stand on its island (the
/// map's `bluemarket`), to the left of whoever arrives there, across the
/// island's middle from the red one.
pub const ARENA_UPGRADE_SHOPS: [Vec2; ARENA_TEAMS] = [
    Vec2::new(-5.0, 130.13),
    Vec2::new(-130.13, -5.0),
    Vec2::new(5.0, -130.13),
    Vec2::new(130.13, 5.0),
];

/// Which of the [`ARENA_ISLANDS`] the player in `seat` arrives on, and is
/// put back on when they fall off (the game's `lobby`): seat by seat round
/// the four, so that the first four seats of a game of Bridge Wars are on
/// four different islands (Hajun: "when there is 2 players+ you will not
/// spawn in the same bridge connected island"), and the next four beside
/// them, a team to an island.
pub fn arena_island(seat: usize) -> usize {
    seat % ARENA_ISLANDS.len()
}

/// Which team the player in `seat` is on: the island's (2v2v2v2 since
/// 2026-11-08). Teammates cannot hurt each other, nor their own anchor.
/// A seat past the eight players' is a zombie's (since 2026-11-11,
/// [`bridge_wars::ZOMBIE_TEAM`]): on nobody's side but the zombies'.
pub fn arena_team(seat: usize) -> u8 {
    if seat >= ARENA_SEATS {
        return bridge_wars::ZOMBIE_TEAM;
    }
    arena_island(seat) as u8
}

/// What Bridge Wars' generators make, and its shop is paid in (since
/// 2026-11-11; Hajun's four item models, `silver_item.glb` and the rest):
/// silver and gold on each team's own island, sapphire on the four smaller
/// islands between them, amethyst in the middle of the map.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    Silver,
    Gold,
    Sapphire,
    Amethyst,
}

impl Item {
    /// Every item, in the order they are counted in ([`Purse`]) and shown
    /// down the screen.
    pub const ALL: [Item; 4] = [Item::Silver, Item::Gold, Item::Sapphire, Item::Amethyst];

    /// Its place in a [`Purse`].
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// How many of each [`Item`] someone holds, by [`Item::index`].
pub type Purse = [u16; 4];

/// Something Bridge Wars' shops sell (Hajun's `bridgewarsgameevent.txt`;
/// only the bridge until 2026-11-12). The item shop, at the red stand,
/// sells what is yours alone; the upgrade shop, at the blue one, what is
/// your whole team's ([`bridge_wars::sold_at`]). What each costs, and
/// whether it can be bought at all, is [`bridge_wars::price`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ware {
    /// [`bridge_wars::BRIDGE_A_BUY`] metres of bridge to lay.
    Bridge,
    /// A sword, by its tier: 1 the basic one, 2 the silver, 3 the aura.
    /// Only one better than the one in hand; lost on dying.
    Sword(u8),
    /// A shield, by its tier, 1 to 3: a share of every blow kept off.
    /// Only one better than the one worn; kept for the game.
    Shield(u8),
    /// One more of something to use: onto the button that holds its kind,
    /// or, a new kind, onto the first empty one of the five
    /// ([`UTILITY_SLOTS`]).
    Utility(Utility),
    /// The next step of the team's own generator: faster, faster again,
    /// amethyst too, and fastest.
    Forge,
    /// Ten more damage on every sword the team swings. Once.
    StrongerSwords,
    /// The next step of the team's shields: so much more kept off, four
    /// times over.
    BetterShields,
    /// The team heals while it stands on its own island.
    HealPool,
    /// One more blindness trap waiting on the team's island, three at the
    /// most.
    Trap,
    /// The Player Compass Tracker (Hajun's file, 2026-11-12: "one time per
    /// life purchase - lost on death, which is a non slot filling item
    /// that basically shows a small white arrow around your character
    /// that is pointing at the direction of the nearest nonteammate
    /// player + name (color of team) + '123m'"): yours until you die,
    /// and nothing on the second button. The last of the wares, so that an
    /// app from before it, which never asks for it, reads the rest as it
    /// did.
    Compass,
    /// The next step of the team's building: everyone of the team walks
    /// faster carrying a plank, and the builders their Bridge Builders
    /// send walk as fast ([`bridge_wars::BUILD_PACE`]; since 2026-11-14,
    /// Hajun's file: "Advanced Builder ... Master Builder ... Grand
    /// Builder"). At the upgrade shop, three steps.
    BuildSpeed,
}

/// How many buttons Bridge Wars has for things to use ([`Utility`]): the
/// second button, beside the sword, and four more on an arc round those
/// (since 2026-11-13, Hajun's `new4morebuttonlayout.jpg`: "4 more buttons
/// that are hidden by default until either u press the ... toggle button
/// or u buy an item that fills up another slot"; one until then). Each
/// holds one kind of thing, however many of it, and they fill in the
/// order things are bought ("it fills up the slots according to buy order
/// so u can buy more than 1 item and hold it at once").
pub const UTILITY_SLOTS: usize = 5;

/// So many of one kind of thing to use, on one of the buttons
/// ([`Kit::slots`]). With none left it stays on its button for as long as
/// what was drunk from it lasts ([`bridge_wars::Outfits::tick`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stack {
    pub utility: Utility,
    pub count: u8,
}

/// Something to use in Bridge Wars, bought at the item shop and put on a
/// button of its own, however many of it ([`Stack`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Utility {
    /// Aimed and thrown: it flies straight and bursts at the end of its
    /// range, or on whoever it meets ([`bridge_wars::Flight`]).
    Fireball,
    /// Aimed and sent off: a builder who walks that way laying a bridge.
    Builder,
    /// Drunk: [`bridge_wars::FAST_SECS`] of running faster.
    Speed,
    /// Drunk: [`bridge_wars::UNSEEN_SECS`] of not being seen.
    Invisibility,
    /// Drunk: half of your health back, at once.
    Healing,
    /// Aimed and lobbed, as far as the drag is long: it comes down, lies
    /// there while its fuse burns, and blows up, taking the planks of
    /// every bridge in its blast with it ([`bridge_wars::Lob`]; since
    /// 2026-11-13).
    Dynamite,
    /// Drunk: [`bridge_wars::LEAP_SECS`] of jumping [`bridge_wars::LEAP`]
    /// times as high (since 2026-11-14; Hajun's file: "Leap Potion: 1
    /// amethyst (after drinking time 1.6s, you get 5x jump height jump
    /// boost for 45 sec, similar ui as speed potions)"). The last of the
    /// kinds, so that the rest are sent as they were.
    Leap,
}

impl Utility {
    pub const ALL: [Utility; 7] = [
        Utility::Fireball,
        Utility::Builder,
        Utility::Speed,
        Utility::Invisibility,
        Utility::Healing,
        Utility::Dynamite,
        Utility::Leap,
    ];

    /// Whether it is aimed, by holding its button and dragging, rather
    /// than pressed.
    pub const fn aimed(self) -> bool {
        matches!(self, Utility::Fireball | Utility::Builder | Utility::Dynamite)
    }

    /// Whether it takes [`bridge_wars::DRINK_SECS`] to drink before it
    /// does anything: every potion (the healing one too since Hajun's
    /// file of 2026-11-12, "after drinking time 1.6s, speed reduct, same
    /// as speed potion").
    pub const fn drunk(self) -> bool {
        !self.aimed()
    }

    /// Whether what it does lasts a while once it is drunk, with the time
    /// left written over the button.
    pub const fn lasts(self) -> bool {
        matches!(self, Utility::Speed | Utility::Invisibility | Utility::Leap)
    }
}

/// What someone in a game of Bridge Wars has, besides what is in their
/// [`Purse`]: the sword in their hand and the shield round them, by tier
/// (0 is none); what is on each of their buttons for things to use and how
/// many of it, button by button ([`UTILITY_SLOTS`]; one kind and a count
/// until 2026-11-13, when there was one button); until
/// when, in whole seconds into the fight, they run fast and go unseen (0
/// is not at all); and whether a trap has blinded them. And whether they
/// have the compass ([`Ware::Compass`]), **which is not sent**: only its
/// owner's device draws it, and is told it is theirs as they buy it
/// ([`Tell::Bought`]), so a kit goes over the network as it did before
/// there was one, and an app from before reads it as it did. Whoever runs
/// the game has it here, the server online and the device offline; online
/// a device keeps its own word of it and puts it back on its own kit as
/// the room's kits come.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Kit {
    pub sword: u8,
    pub shield: u8,
    pub slots: [Option<Stack>; UTILITY_SLOTS],
    pub fast_until: u16,
    pub unseen_until: u16,
    pub blind: bool,
    #[serde(skip)]
    pub compass: bool,
    /// Until when they jump higher, for having drunk the leap potion
    /// (since 2026-11-14): sent after the rest.
    pub leap_until: u16,
}

impl Kit {
    /// Which button `utility` is on, if any: with some of it, or with none
    /// left and what was drunk from it still lasting.
    pub fn slot_of(&self, utility: Utility) -> Option<usize> {
        self.slots.iter().position(|slot| slot.is_some_and(|stack| stack.utility == utility))
    }

    /// The first button with nothing on it: where a new kind of thing
    /// goes, so that the buttons fill in the order things are bought.
    pub fn free_slot(&self) -> Option<usize> {
        self.slots.iter().position(Option::is_none)
    }

    /// How many of `utility` they have.
    pub fn held(&self, utility: Utility) -> u8 {
        self.slot_of(utility).and_then(|slot| self.slots[slot]).map_or(0, |stack| stack.count)
    }

    /// The first potion they have one of, button by button.
    pub fn potion(&self) -> Option<Utility> {
        self.slots.iter().flatten().find(|stack| stack.count > 0 && stack.utility.drunk()).map(|stack| stack.utility)
    }

    /// This kit with `count` of `utility` on the button that kind is on,
    /// or, with none, on the first empty one; as it is, with every button
    /// taken by other kinds.
    pub fn holding(mut self, utility: Utility, count: u8) -> Self {
        if let Some(slot) = self.slot_of(utility).or(self.free_slot()) {
            self.slots[slot] = Some(Stack { utility, count });
        }
        self
    }


    /// Whether they are running fast, `elapsed` seconds into the fight.
    pub fn fast(&self, elapsed: f32) -> bool {
        f32::from(self.fast_until) > elapsed
    }

    /// Whether they are unseen then.
    pub fn unseen(&self, elapsed: f32) -> bool {
        f32::from(self.unseen_until) > elapsed
    }

    /// Whether they jump higher then, for the leap potion.
    pub fn leaping(&self, elapsed: f32) -> bool {
        f32::from(self.leap_until) > elapsed
    }

    /// Until when, in whole seconds into the fight, what was drunk of
    /// `potion` lasts: 0 for one that is over, or that does not last.
    pub fn lasts_until(&self, potion: Utility) -> u16 {
        match potion {
            Utility::Speed => self.fast_until,
            Utility::Invisibility => self.unseen_until,
            Utility::Leap => self.leap_until,
            _ => 0,
        }
    }
}

/// What a team has bought at the upgrade shop: how far its generator has
/// been raised (0 to [`bridge_wars::FORGE_MOST`]), whether its swords are
/// stronger, how far its shields have been bettered (0 to
/// [`bridge_wars::BETTER_SHIELDS_MOST`]), whether it has a heal pool, how
/// many blindness traps wait on its island, and how far its building has
/// been quickened (0 to [`bridge_wars::BUILD_SPEED_MOST`], since
/// 2026-11-14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Upgrades {
    pub forge: u8,
    pub swords: bool,
    pub shields: u8,
    pub heal_pool: bool,
    pub traps: u8,
    pub build_speed: u8,
}

pub mod bots;
pub mod horde;
pub mod nav;

// The town's plots, whose each is, and what fills those nobody who is
// here has (since 2026-11-23, a file of its own; in this one since
// 2026-11-20), and the made-up land that fills what is left.
pub mod land;
pub mod premade;

// Visiting someone's house, rating it, and the board of the best (since
// 2026-11-27).
pub mod house;

// The four games of 2026-11-18 (Hajun: "add these minigames: Blend In,
// Knock Off, Bridge Race, Last Stand"): what they have in common, and each
// one's own rules.
pub mod table;
pub mod blend_in;
pub mod knock_off;
pub mod bridge_race;
pub mod last_stand;

pub use social::Minigame;

/// Bridge Wars' rules, as the game plays them on the device (`arena`) and
/// the server does online (`server/src/arena.rs`): in one place, so that
/// the two never differ. The device's `assets/ui.ron` has the same numbers
/// for playing offline, where Hajun tunes them by hand; online the server's
/// rule.
pub mod bridge_wars {
    use super::{
        AMETHYST_PADS, ARENA_ISLAND_RADIUS, ARENA_ISLANDS, ARENA_PADS, ARENA_SEATS, ARENA_SHOPS, ARENA_TEAMS,
        ARENA_UPGRADE_SHOPS, Item, Kit, PAD_HALF, PAD_TOP, Purse, Quat, SAPPHIRE_ISLAND_RADIUS, SAPPHIRE_PADS, Stack, TAU,
        Upgrades, Utility, Vec2, Vec3, Ware, arena_team,
    };
    use serde::{Deserialize, Serialize};

    /// How long the waiting room waits for someone else to join before the
    /// computer fills the seats left, in seconds, counted from the last to
    /// join (Hajun, the night of 2026-11-07: "if there is no player joining for 10sec,
    /// fill it with bots").
    pub const BOT_WAIT: f32 = 10.0;
    /// How long after one of the computer's players has come into the
    /// waiting room the next does, in seconds, at the least and at the
    /// most, thrown anew for each (since 2026-11-16; Hajun: "make them
    /// join slightly delay one by one rather than all at once"): seven of
    /// them are all in after about five seconds. Until then they all came
    /// in the one frame.
    pub const BOT_GAP: (f32, f32) = (0.25, 1.1);
    /// How long a full room counts down before everyone is sent to their
    /// island, in seconds ("it will count down from 5").
    pub const COUNTDOWN: f32 = 5.0;
    /// How long a game lasts, in seconds, after which it is a draw and
    /// everyone is back in the town: 20 minutes since the night of
    /// 2026-11-16 (Hajun's `bridgewarsgameevent.md`: "20:00  Game ends in a
    /// draw"; "i want it to be 20min instead of 25min like i updted
    /// bridgewarsgaemeevent.md"), 25 from 2026-11-11 until then, and 10
    /// before that ("a (small) timer of 10 minute"). What happens on the
    /// way is [`TIMETABLE`].
    pub const PLAY_SECS: f32 = 1200.0;
    /// How long before the end everyone is told it is coming, in seconds
    /// ([`Happening::DrawSoon`]; Hajun: "at 19:30 just notifying that the
    /// game will end in a draw in 30 seconds").
    pub const DRAW_SOON: f32 = 30.0;
    /// When every anchor left breaks, and when the zombies begin to come,
    /// in seconds into the fight ([`TIMETABLE`]): 14:00 and 15:00 (the
    /// anchors broke at 10:00 until the night of 2026-11-16).
    pub const ANCHORS_BREAK: f32 = 840.0;
    pub const SUDDEN_DEATH: f32 = 900.0;
    /// How long the result is up at the end, in seconds, before everyone
    /// still there is sent back to the town.
    pub const RESULTS_SECS: f32 = 6.0;
    /// What a seat in a game costs, in coins, taken as you go in and given
    /// back to whoever leaves before the fight begins (since 2026-11-18;
    /// Hajun: "make the cost of bridgewars to be 50 coins as well"; free
    /// until then).
    pub const FEE: u32 = 50;
    /// What each person in a game is paid at its end, in coins, by how
    /// their team came, first to fourth: the team left standing first, and
    /// the rest by the order they went out, the last out second (mine, by
    /// Hajun's rule for every game with a fee: "Generally should be above
    /// 50% to win net positive coin, and if not, net negative coin"; Knock
    /// Off's four places, net +80, +10, -40 and -50: a team that beats two
    /// of the three others comes out ahead). Teams that share a place, out
    /// in the same moment or all still in at the draw, share what the
    /// places they take up pay ([`prizes`]); the computer is paid nothing.
    pub const PRIZES: [u32; ARENA_TEAMS] = [130, 60, 10, 0];

    /// Where each team came in a game that is over, from how far into the
    /// fight each had nobody left in play, `out_at[team]` (none: still in at
    /// the end): those still in first, together (one winner, or every team
    /// still in at the draw); then the rest by how long each lasted, the
    /// last out next; teams out in the very same moment sharing a place. A
    /// place is how many came before it. The server's and a device's
    /// offline, the same.
    pub fn places(out_at: &[Option<f32>; ARENA_TEAMS]) -> [u8; ARENA_TEAMS] {
        let lasted = out_at.map(|out| out.unwrap_or(f32::INFINITY));
        std::array::from_fn(|team| lasted.iter().filter(|other| **other > lasted[team]).count() as u8)
    }

    /// What each seat is paid at the end of a game, by seat: `places[team]`
    /// is where each team came, from 0, teams that tie on the same place;
    /// `people[seat]` whether the seat is a person's (the computer's are
    /// paid nothing). A team's place pays each of its people
    /// ([`PRIZES`], shared by the teams tied there, as the tables share
    /// theirs, [`crate::table::split`]).
    pub fn prizes(places: &[u8; ARENA_TEAMS], people: &[bool]) -> Vec<u32> {
        let by_team = crate::table::split(&PRIZES, places);
        (0..people.len())
            .map(|seat| if people[seat] { by_team[usize::from(arena_team(seat))] } else { 0 })
            .collect()
    }

    /// The health everyone starts with, what a punch takes off it, and
    /// what a respawn anchor starts with: ten punches' worth (Hajun,
    /// 2026-11-08: twenty, "make its default hp enough so that u need to
    /// hit it 20 times to break it"; then, 2026-11-09, "make the respawn
    /// anchor's hp to be 1/2 of now").
    pub const HEALTH: f32 = 100.0;
    pub const DAMAGE: f32 = 12.0;
    pub const ANCHOR_HEALTH: f32 = 120.0;
    /// How long after a punch lands on a body, or an anchor, no other
    /// counts, in seconds: Minecraft's ten ticks of hurt resistance, the
    /// game's `melee.hurt_time`. The server keeps it too (2026-11-09,
    /// Hajun: the bots "seem to get knocked back too much by me maybe cuz
    /// i can hit them multiple times before they process it"): every punch
    /// a device landed went through before.
    pub const HURT_TIME: f32 = 0.5;
    /// How long the dead stay dead before standing again, in seconds, and
    /// how long after the last hit health starts to come back, and how much
    /// a second.
    pub const RESPAWN_SECS: f32 = 4.0;
    pub const REGEN_AFTER: f32 = 5.0;
    pub const REGEN_PER_SEC: f32 = 4.0;
    /// How fast everyone runs there, in metres a second; how far a bot sees
    /// whoever it goes for, in metres; the least time between its punches,
    /// in seconds; under what share of its health it runs away; and how far
    /// from the puncher's middle a punch reaches.
    pub const RUN_SPEED: f32 = 10.0;
    pub const HUNT_RADIUS: f32 = 24.0;
    pub const BOT_PUNCH_EVERY: f32 = 0.8;
    pub const FLEE_BELOW: f32 = 0.2;
    pub const REACH: f32 = 2.0;
    /// How far below the map a body has fallen before it is dead, in
    /// metres.
    pub const FALL_TO_DIE: f32 = 30.0;
    /// How hard a punch knocks, in metres a second, away from the puncher,
    /// and that much more from a puncher on the move; and what is left of
    /// the knock each twentieth of a second on the ground: the game's
    /// `melee` numbers, Minecraft 1.8.9's, for the server's bots, which
    /// it knocks about itself.
    pub const KNOCK_BACK: f32 = 10.0;
    pub const SPRINT_BONUS: f32 = 10.0;
    pub const KNOCK_UP: f32 = 8.0;
    pub const GROUND_DRAG: f32 = 0.546;
    pub const AIR_DRAG: f32 = 0.91;

    /// The waiting room, Hajun's glass box over the middle of the map
    /// (`pvptownmap.glb`, the object `waiting room`): where its floor is,
    /// in metres up, and how far from its middle everyone stands in it,
    /// facing the middle. The game checks the map agrees. 150.5 since
    /// Hajun's map of 2026-11-11, which has the room 50 m higher; 100.5
    /// until then.
    pub const WAITING_FLOOR: f32 = 150.5;
    pub const WAIT_RING: f32 = 3.0;

    /// Where the player in `seat` waits for the game: on a ring round the
    /// middle of the waiting room, on its floor, facing in, each an eighth
    /// of the turn on from the last.
    pub fn waiting_spot(seat: usize) -> (Vec3, Quat) {
        let angle = seat as f32 * TAU / ARENA_SEATS as f32;
        let at = Vec3::new(angle.sin() * WAIT_RING, WAITING_FLOOR, angle.cos() * WAIT_RING);
        let toward = -at.with_y(0.0).normalize_or(Vec3::NEG_Z);
        (at, Quat::from_rotation_y((-toward.x).atan2(-toward.z)))
    }

    /// Whether a punch thrown from `from` could reach an anchor standing at
    /// `at`: within `reach` of the puncher's middle, counting the anchor's
    /// own width. The server checks a hit on an anchor with this, where
    /// the puncher last said they stood, with slack for the way since; the
    /// device aims one with its own range.
    pub fn anchor_in_reach(from: Vec2, at: Vec2, reach: f32) -> bool {
        from.distance(at) <= reach + super::ANCHOR_RADIUS
    }

    // ------------------------------------------------------- the timetable

    /// Something that happens at a set time into a game (Hajun's
    /// `bridgewarsgameevent.txt`, 2026-11-11), which every device says in a
    /// line across the upper half of the screen as it happens.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Happening {
        /// Every generator of `item` makes its things faster from now on:
        /// "Sapphire II".
        Tier { item: Item, tier: u8 },
        /// "Spawn Anchor destruction: all remaining spawn anchors break":
        /// from now on whoever dies is out.
        AnchorsBreak,
        /// "Sudden death: zombies spawn all over the map starting slow,
        /// and more spawn, exponentially": from now on they come
        /// ([`zombies_at`]).
        SuddenDeath,
        /// The game ends in a draw in [`DRAW_SOON`] seconds.
        DrawSoon,
    }

    /// What happens when, in seconds into the fight, in order: the file's
    /// timetable, from "2:00 Sapphire II" to "15:00 Sudden death", and
    /// the word that the end is coming. Its first line, both generators at
    /// their first tier, is how a game begins, and its last, "20:00 Game
    /// ends in a draw", is [`PLAY_SECS`].
    pub const TIMETABLE: [(f32, Happening); 7] = [
        (120.0, Happening::Tier { item: Item::Sapphire, tier: 2 }),
        (240.0, Happening::Tier { item: Item::Amethyst, tier: 2 }),
        (360.0, Happening::Tier { item: Item::Sapphire, tier: 3 }),
        (480.0, Happening::Tier { item: Item::Amethyst, tier: 3 }),
        (ANCHORS_BREAK, Happening::AnchorsBreak),
        (SUDDEN_DEATH, Happening::SuddenDeath),
        (PLAY_SECS - DRAW_SOON, Happening::DrawSoon),
    ];

    /// What happens later than `before` and no later than `now`, both in
    /// seconds into the fight: this frame's events, given last frame's
    /// time and this one's.
    pub fn happening_between(before: f32, now: f32) -> impl Iterator<Item = Happening> {
        TIMETABLE
            .iter()
            .filter(move |(at, _)| *at > before && *at <= now)
            .map(|&(_, event)| event)
    }

    /// Which tier `item`'s generators are at, `elapsed` seconds into the
    /// fight: 1 to begin, and as [`TIMETABLE`] has raised it since.
    pub fn tier(item: Item, elapsed: f32) -> u8 {
        TIMETABLE
            .iter()
            .filter(|(at, _)| *at <= elapsed)
            .filter_map(|&(_, event)| match event {
                Happening::Tier { item: raised, tier } if raised == item => Some(tier),
                _ => None,
            })
            .max()
            .unwrap_or(1)
    }

    // ------------------------------------------------------ the generators

    /// How many generators the map has: a silver one and a gold one on
    /// each team's pad, a sapphire one on each of the four smaller
    /// islands, and four amethyst ones in the middle ([`generator`]).
    pub const GENERATORS: usize = 16;

    /// What the generator numbered `index` makes, and the middle of the
    /// pad it makes it on. Past the map's own sixteen are the four a
    /// team's own generator becomes once it is raised to make amethyst
    /// too ([`FORGE_AMETHYST`]): on the team's pad, by team.
    pub fn generator(index: usize) -> (Item, Vec2) {
        match index {
            0..=3 => (Item::Silver, ARENA_PADS[index]),
            4..=7 => (Item::Gold, ARENA_PADS[index - 4]),
            8..=11 => (Item::Sapphire, SAPPHIRE_PADS[index - 8]),
            12..=15 => (Item::Amethyst, AMETHYST_PADS[index - 12]),
            _ => (Item::Amethyst, ARENA_PADS[(index - GENERATORS) % ARENA_TEAMS]),
        }
    }

    /// How many times as fast a team's own generator makes its silver and
    /// its gold, by how far the team has raised it at the upgrade shop
    /// (Hajun's file: "Better Generator ... (faster base generator 150%)
    /// -> Advanced Generator ... (even faster base generator 200%) ->
    /// Amethyst Generator ... (starts spawning amethyst too) -> Molten
    /// Generator ... (fastest 300%)"; that the third step keeps the
    /// second's pace is mine).
    pub const FORGE_SPEED: [f32; 5] = [1.0, 1.5, 2.0, 2.0, 3.0];
    /// From which step a team's generator makes amethyst too, how long each
    /// takes, in seconds, and how many lie on the pad at the most: as slow
    /// as the map's own amethyst generators begin, and as few.
    pub const FORGE_AMETHYST: u8 = 3;
    pub const FORGE_AMETHYST_SECS: f32 = 60.0;
    pub const FORGE_AMETHYST_CAP: usize = 2;

    /// How long a generator of `item` at `tier` takes to make one, in
    /// seconds, and how much more or less than that each one may take
    /// (Hajun's file: "Silver: baserate = 1 per ~2 sec (+- 0.5 sec variable
    /// interval)", "Gold: baserate = 1 per ~7.5sec (+-1 sec variable
    /// interval)", "Sapphire: base rate = 1 per 30 s (Tier I) ... Tier2 =
    /// 1per23s, Tier3 = 1per 12s", "Amethyst: base rate = 1 per 60 s (Tier
    /// I) ... Tier2 = 1 per 40s, Tier3 = 1 per 25s": 40, 30 and 20 until
    /// 2026-11-12).
    pub fn interval(item: Item, tier: u8) -> (f32, f32) {
        match (item, tier) {
            (Item::Silver, _) => (2.0, 0.5),
            (Item::Gold, _) => (7.5, 1.0),
            (Item::Sapphire, 0 | 1) => (30.0, 0.0),
            (Item::Sapphire, 2) => (23.0, 0.0),
            (Item::Sapphire, _) => (12.0, 0.0),
            (Item::Amethyst, 0 | 1) => (60.0, 0.0),
            (Item::Amethyst, 2) => (40.0, 0.0),
            (Item::Amethyst, _) => (25.0, 0.0),
        }
    }

    /// The most a generator of `item` holds on its pad before it stops
    /// making more until some are taken (the file's "max held on
    /// generator": 48, 6, 4 and 2).
    pub fn cap(item: Item) -> usize {
        match item {
            Item::Silver => 48,
            Item::Gold => 6,
            Item::Sapphire => 4,
            Item::Amethyst => 2,
        }
    }

    /// How far from the middle of its pad a thing may come up, in metres
    /// each way across the ground: well inside the pad's edge, the things
    /// being 0.7 m across.
    pub const PAD_SPREAD: f32 = 0.7;
    /// How many places on its pad a generator looks at for each thing it
    /// makes: it takes the one furthest from everything lying there, so
    /// that a pad fills evenly, and no two things come up in one spot
    /// while there is room (Hajun: "semi random location").
    const PLACES_TRIED: usize = 6;
    /// How near someone has to come to a thing to pick it up, in metres
    /// across the ground from their middle to its (Hajun: "the user can
    /// collect it by walking in it (short magnet)"): a body's own width
    /// and a little over a metre, so that standing at a pad's edge takes
    /// what lies along it, and walking across takes it all.
    pub const MAGNET: f32 = 1.5;
    /// How far over someone's feet a thing may be and still be picked up
    /// by them, in metres: half a metre over the top of their head, which
    /// is 1.64 m up, as Minecraft picks a thing up half a block over a
    /// player's; and how far under their feet: a jump's height over what
    /// lies on a pad, so that someone jumping across a pad takes what
    /// lies on it, and neither the waiting room nor a fall past the island
    /// does. A thing is where it is as it drops ([`Piece::height`]), so
    /// one still coming down is out of reach until it is that near, and
    /// someone who jumps for it has it sooner, in the air (Hajun,
    /// 2026-11-12: "the delay is a bit too much for the delay between the
    /// item dropping from generator and the user collecting - user should
    /// be able to collect in the air too maybe when they jump"). For an
    /// evening before that a thing could not be picked up until it had
    /// dropped and lain 0.3 s, 0.85 s after it was made, however near it
    /// came; standing on its pad it is yours 0.45 s after it is made now,
    /// and at the top of a jump 0.32 s after.
    pub const REACH_OVER: f32 = 2.14;
    pub const REACH_UNDER: f32 = 1.5;
    /// How far over its pad's top a thing's own middle hangs once it has
    /// dropped, in metres (the device bobs it about there, `loot`): the
    /// models reach 0.18 m under their middles at the most, so the lowest
    /// of them clears the pad by a hand.
    pub const HOVER: f32 = 0.42;
    /// How far over where it will hang a thing appears as it is made, to
    /// drop from there, in metres, and how long the drop takes, in
    /// seconds (Hajun, 2026-11-11: "i want it to be a drop from a certain
    /// height animation. so that even when player is standing on top of
    /// the generator, u can see the mineral for a bit before the player
    /// pick it up"): from over the head of anyone standing on the pad.
    /// Both numbers are mine. The device draws the drop (`loot`), and
    /// whoever runs the generators goes by it for who can reach the thing
    /// ([`Mint::pick_up`]).
    pub const DROP_FROM: f32 = 5.0;
    pub const DROP_SECS: f32 = 0.55;

    /// How far over where it hangs a thing is, `age` seconds after it was
    /// made, in metres: dropping from [`DROP_FROM`] for [`DROP_SECS`], as
    /// a thing let go falls, slowly and then fast, and there from then on.
    pub fn drop_height(age: f32) -> f32 {
        if age >= DROP_SECS {
            return 0.0;
        }
        let fallen = age.max(0.0) / DROP_SECS;
        DROP_FROM * (1.0 - fallen * fallen)
    }

    /// What someone who dies on land drops (since the night of
    /// 2026-11-15; Hajun: "right now you just get the sapphire and amethyst
    /// automatically into ur balance after killing, regardless of whether
    /// it was a fall off the world kill or a land-hp kill. i think if its a
    /// land-hp kill where they die on land, they should drop the sapphires
    /// and amethyst like small scatter explosion from their dead body
    /// location like mineraft dropping items on death, and u only get it if
    /// u pick it up. but items shouldnt scatter to location where they would
    /// fall off the map"; [`Mint::spill`]). The generator a thing that was
    /// dropped is said to be of, which is none; how near whoever died and
    /// how far from them a thing comes down, in metres across the ground;
    /// how long it is in the air, in seconds, and how far over the straight
    /// line from them to where it comes down it rises on the way
    /// ([`toss`]); how high over their feet it leaves them; the most
    /// things of one item that are dropped, each being more than one of it
    /// if they held more than that many ([`Piece::worth`]); how far over
    /// their feet the ground a thing comes down on may be, a step, and how
    /// far under them, which is further than anyone is ever over ground
    /// they are not falling past; and how many places are tried for each.
    /// All of them are mine.
    ///
    /// And how long a thing lies where it came down before anyone can
    /// have it, in seconds (Hajun, an hour on: "killing someone so ur
    /// already near them so u might pick it up right away too fast for it
    /// to be visible that they dropped sth - make sure that there is a
    /// very small delay before u can pick it up so u can atleast see very
    /// quickly before u pick it up rather than instantly picking it up"):
    /// whoever stands over the dead sees what they held fly out, come
    /// down and lie there, and then has what is within their reach,
    /// [`SPILL_WAIT`] after the death in all. Until then a thing was
    /// theirs the moment it came down. How long is mine.
    pub const SPILT: u8 = u8::MAX;
    pub const SPILL_NEAR: f32 = 0.8;
    pub const SPILL_FAR: f32 = 2.4;
    pub const TOSS_SECS: f32 = 0.5;
    pub const TOSS_RISE: f32 = 1.3;
    pub const SPILL_LIES: f32 = 0.3;
    pub const SPILL_WAIT: f32 = TOSS_SECS + SPILL_LIES;
    pub const SPILL_FROM: f32 = 1.0;
    pub const SPILL_MOST: u16 = 10;
    pub const SPILL_UP: f32 = 0.6;
    pub const SPILL_DOWN: f32 = 12.0;
    const SPILL_TRIES: usize = 8;

    /// Where a thing thrown from `from` to hang at `to` is, `age` seconds
    /// after it was: on its way for [`TOSS_SECS`], over the straight line
    /// between the two by as much as [`TOSS_RISE`] half way, and at `to`
    /// from then on. The device draws it (`loot`), and nobody has the thing
    /// until it has come down and lain a moment ([`Piece::lying`]).
    pub fn toss(from: Vec3, to: Vec3, age: f32) -> Vec3 {
        let done = (age / TOSS_SECS).clamp(0.0, 1.0);
        from.lerp(to, done) + Vec3::Y * (TOSS_RISE * 4.0 * done * (1.0 - done))
    }

    /// One thing lying in a game until someone picks it up: one a
    /// generator has made, dropping onto its pad or lying on it, or one
    /// that someone who died dropped ([`Mint::spill`]). A number of its
    /// own, which generator made it ([`SPILT`] for one that was dropped),
    /// what it is, where it lies across the ground, and how far into the
    /// fight it was made, in seconds; how high what it lies on is, a
    /// pad's top for a generator's; where it was thrown from, if it was
    /// dropped; and how many of its item it is, which is one, but for a
    /// thing dropped by someone who held more than [`SPILL_MOST`] of it.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Piece {
        pub id: u32,
        pub generator: u8,
        pub item: Item,
        pub at: Vec2,
        pub born: f32,
        pub ground: f32,
        pub from: Option<Vec3>,
        pub worth: u16,
    }

    impl Piece {
        /// Where its middle hangs once it has come down.
        pub fn rest(&self) -> Vec3 {
            Vec3::new(self.at.x, self.ground + HOVER, self.at.y)
        }

        /// How high its middle is, `elapsed` seconds into the fight, in
        /// the map's own heights: over what it lies on where it hangs,
        /// and as far over that as it has still to drop, or wherever it
        /// is on its way from whoever dropped it.
        pub fn height(&self, elapsed: f32) -> f32 {
            let age = elapsed - self.born;
            match self.from {
                None => self.ground + HOVER + drop_height(age),
                Some(from) => toss(from, self.rest(), age).y,
            }
        }

        /// Whether it is there to be picked up, `elapsed` seconds into
        /// the fight: one a generator made is, in the air as it drops;
        /// one that was dropped, once it has come down and lain a moment
        /// ([`SPILL_WAIT`]), so that whoever stands over the dead sees
        /// what they held fly out and lie there before any of it is
        /// theirs.
        pub fn lying(&self, elapsed: f32) -> bool {
            self.from.is_none() || elapsed - self.born >= SPILL_WAIT
        }

        /// Whether someone who died dropped it.
        pub fn dropped(&self) -> bool {
            self.from.is_some()
        }
    }

    /// Whether a generator of `item` makes its things to the second, with
    /// nothing more or less to each wait ([`interval`]): the sapphire and
    /// the amethyst ones, whose next thing every device can count down to
    /// by the time alone ([`next_at`]).
    pub fn steady(item: Item) -> bool {
        interval(item, 1).1 == 0.0
    }

    /// When a [`steady`] generator of `item` next comes to make one, later
    /// than `after`, both in seconds into the fight: a whole wait on from
    /// the last time it did, counted from the start of the fight, and
    /// sooner where a tier raised meanwhile has a wait of its own that
    /// ends sooner, counted from when it was raised (so the sapphire
    /// ones, every 30 s to 120, then every 23 s to 350, then at 372, the
    /// third tier having come at 360, and every 12 s from there). It
    /// comes round whether or not the generator is full, in which case
    /// nothing is made and the count starts over: so the count is the
    /// time's alone, and the same on every device, which is how each
    /// writes "Next Amethyst in 24sec..." over a generator with nothing
    /// sent for it (the time is the round's, to the second).
    pub fn next_at(item: Item, after: f32) -> f32 {
        let mut at = 0.0;
        loop {
            let next = following(item, at);
            if next > after {
                return next;
            }
            at = next;
        }
    }

    /// The time a [`steady`] generator of `item` comes to make one after
    /// the time at `at`.
    fn following(item: Item, at: f32) -> f32 {
        let mut next = at + interval(item, tier(item, at)).0;
        for &(raised_at, happening) in &TIMETABLE {
            if let Happening::Tier { item: raised, tier } = happening
                && raised == item
                && raised_at > at
                && raised_at < next
            {
                next = next.min(raised_at + interval(item, tier).0);
            }
        }
        next
    }

    /// Everything lying in a game for whoever walks into it, and the rules
    /// of its lying there: what a generator makes, dropping onto its pad,
    /// and what is thrown out round where someone died, or something was
    /// killed, coming down where there is ground for it; each picked up by
    /// whoever comes within reach of it, and gone with a plank it lay on.
    /// Taken out of [`Mint`] on 2026-11-18, which keeps its generators over
    /// one of these, for the generators of Last Stand
    /// ([`crate::last_stand`]) and the things its zombies drop: the same
    /// rules, the same numbers. It rolls no dice of its own, as the mint
    /// rolls none: whoever runs it hands it some, numbers from 0 up to 1.
    #[derive(Clone, Debug, Default)]
    pub struct Pile {
        pieces: Vec<Piece>,
        next: u32,
    }

    impl Pile {
        /// Nothing lying anywhere yet.
        pub fn new() -> Self {
            Self::default()
        }

        /// Everything lying there.
        pub fn pieces(&self) -> &[Piece] {
            &self.pieces
        }

        /// How many things the generator numbered `generator` has lying
        /// on its pad; of [`SPILT`], how many lie where they were
        /// dropped.
        pub fn held(&self, generator: u8) -> usize {
            self.pieces.iter().filter(|piece| piece.generator == generator).count()
        }

        /// One more `item` from the generator numbered `generator`,
        /// coming up on its pad, whose middle is `pad` across the ground,
        /// which reaches `half` each way from it, and whose top is `top`
        /// high: of a few places on it at random, the one furthest from
        /// everything already lying there, as far inside the pad's edge as
        /// on Bridge Wars' ([`PAD_SPREAD`] of [`PAD_HALF`]). `elapsed`
        /// seconds into the fight.
        #[allow(clippy::too_many_arguments)]
        pub fn make(
            &mut self,
            generator: u8,
            item: Item,
            pad: Vec2,
            half: f32,
            top: f32,
            elapsed: f32,
            dice: &mut dyn FnMut() -> f32,
        ) -> Piece {
            let piece = Piece {
                id: self.next,
                generator,
                item,
                at: self.place(pad, half, dice),
                born: elapsed,
                ground: top,
                from: None,
                worth: 1,
            };
            self.next = self.next.wrapping_add(1);
            self.pieces.push(piece);
            piece
        }

        /// Somewhere on the pad round `pad`, `half` each way, for a new
        /// thing: of a few places at random, the one furthest from
        /// everything already lying on that pad.
        fn place(&self, pad: Vec2, half: f32, dice: &mut dyn FnMut() -> f32) -> Vec2 {
            let spread = half * (PAD_SPREAD / PAD_HALF);
            let lying: Vec<Vec2> = self
                .pieces
                .iter()
                .map(|piece| piece.at)
                .filter(|at| (*at - pad).abs().max_element() <= half)
                .collect();
            let mut best = (f32::NEG_INFINITY, pad);
            for _ in 0..PLACES_TRIED {
                let off = Vec2::new(dice() * 2.0 - 1.0, dice() * 2.0 - 1.0) * spread;
                let at = pad + off;
                let room = lying
                    .iter()
                    .map(|other| other.distance_squared(at))
                    .fold(f32::INFINITY, f32::min);
                if room > best.0 {
                    best = (room, at);
                }
            }
            best.1
        }

        /// `things` thrown out from feet at `from`, `elapsed` seconds into
        /// the fight, each the item it is and as many of it as it is worth
        /// ([`Piece::worth`]): every way, each to come down [`SPILL_NEAR`]
        /// to [`SPILL_FAR`] from them, of a few places at random the one
        /// furthest from the rest of what is thrown, and to lie there for
        /// whoever walks into it once it has lain a moment
        /// ([`Piece::lying`]). What [`Mint::spill`] throws out of whoever
        /// died, and what Last Stand's zombies drop.
        ///
        /// **Only onto ground**: `ground` says how high what a thing may
        /// lie on is at a place, if there is anything there that a body
        /// can walk to. Nor onto anything more than a step over the feet,
        /// a stand's roof, nor further under them than [`SPILL_DOWN`]. A
        /// place none of the tries finds ground at is under the feet, if
        /// that is ground, or where another of the things lies.
        ///
        /// What was thrown, which is nothing with nothing to throw; or
        /// `None`, with nothing thrown, where there is nowhere for any of
        /// it to lie, the feet being out over the void (asked with nothing
        /// to throw too, so that whoever has nothing is told so as well).
        pub fn scatter(
            &mut self,
            things: &[(Item, u16)],
            from: Vec3,
            elapsed: f32,
            dice: &mut dyn FnMut() -> f32,
            ground: &dyn Fn(Vec2) -> Option<f32>,
        ) -> Option<Vec<Piece>> {
            self.scatter_within(things, from, SPILL_FAR, elapsed, dice, ground)
        }

        /// As [`Pile::scatter`], but each thing comes down from
        /// [`SPILL_NEAR`] to `far` from the feet (since 2026-11-18, for
        /// all that Last Stand's dead hold, which is hundreds of things
        /// at a time, and in [`SPILL_FAR`] lay in one heap).
        pub fn scatter_within(
            &mut self,
            things: &[(Item, u16)],
            from: Vec3,
            far: f32,
            elapsed: f32,
            dice: &mut dyn FnMut() -> f32,
            ground: &dyn Fn(Vec2) -> Option<f32>,
        ) -> Option<Vec<Piece>> {
            let widest = far.max(SPILL_NEAR);
            let lies = |at: Vec2| ground(at).filter(|top| (-SPILL_DOWN..=SPILL_UP).contains(&(top - from.y)));
            let feet = Vec2::new(from.x, from.z);
            let under = lies(feet).map(|top| (feet, top));
            // Where each comes down.
            let mut places: Vec<(Vec2, f32)> = Vec::new();
            for _ in 0..things.len().max(1) {
                let mut best: Option<(f32, Vec2, f32)> = None;
                for _ in 0..SPILL_TRIES {
                    let (turn, far) = (dice() * TAU, SPILL_NEAR + (widest - SPILL_NEAR) * dice());
                    let at = feet + Vec2::from_angle(turn) * far;
                    let Some(top) = lies(at) else {
                        continue;
                    };
                    let room = places.iter().map(|(other, _)| other.distance_squared(at)).fold(f32::INFINITY, f32::min);
                    if best.is_none_or(|(most, ..)| room > most) {
                        best = Some((room, at, top));
                    }
                }
                let place = best
                    .map(|(_, at, top)| (at, top))
                    .or(under)
                    .or_else(|| places.get(places.len().wrapping_sub(1)).copied());
                places.push(place?);
            }
            let thrown = from + Vec3::Y * SPILL_FROM;
            let mut dropped = Vec::with_capacity(things.len());
            for (&(item, worth), &(at, top)) in things.iter().zip(&places) {
                let piece = Piece {
                    id: self.next,
                    generator: SPILT,
                    item,
                    at,
                    born: elapsed,
                    ground: top,
                    from: Some(thrown),
                    worth,
                };
                self.next = self.next.wrapping_add(1);
                self.pieces.push(piece);
                dropped.push(piece);
            }
            Some(dropped)
        }

        /// Takes away whatever was dropped and lies where `stands` says
        /// there is nothing under it any more, a plank it lay on having
        /// gone: which they were. It falls with the plank, and is
        /// nobody's.
        pub fn shed(&mut self, stands: &dyn Fn(Vec2) -> bool) -> Vec<Piece> {
            if !self.pieces.iter().any(Piece::dropped) {
                return Vec::new();
            }
            let (gone, left): (Vec<Piece>, Vec<Piece>) = self.pieces.iter().copied().partition(|piece| piece.dropped() && !stands(piece.at));
            self.pieces = left;
            gone
        }

        /// Takes away whatever was dropped and has lain `most` seconds or
        /// more, `elapsed` seconds into the fight: which they were, gone
        /// for good (since 2026-11-18, Last Stand's: Hajun, "the items
        /// despawns after a very long time"). What a generator made stays
        /// on its pad until it is taken, as ever; Bridge Wars calls this
        /// never.
        pub fn lapse(&mut self, elapsed: f32, most: f32) -> Vec<Piece> {
            if !self.pieces.iter().any(|piece| piece.dropped() && elapsed - piece.born >= most) {
                return Vec::new();
            }
            let (gone, left): (Vec<Piece>, Vec<Piece>) =
                self.pieces.iter().copied().partition(|piece| piece.dropped() && elapsed - piece.born >= most);
            self.pieces = left;
            gone
        }

        /// Whoever's feet are at `at`, `elapsed` seconds into the fight,
        /// picks up everything within reach of them: within [`MAGNET`]
        /// across the ground, and, where the thing is now, dropping or
        /// lying ([`Piece::height`]), no further over their feet than
        /// [`REACH_OVER`] nor under them than [`REACH_UNDER`]; and of
        /// what was dropped, only what has come down and lain a moment
        /// ([`Piece::lying`]). What they took, no longer there for anyone
        /// else: each is as many of its item as it is worth
        /// ([`Piece::worth`]).
        pub fn pick_up(&mut self, at: Vec3, elapsed: f32) -> Vec<Piece> {
            self.pick_up_where(at, elapsed, &mut |_| true)
        }

        /// As [`Pile::pick_up`], but only what `wanted` says yes to, asked
        /// of each thing within reach in turn: the rest is left lying.
        pub fn pick_up_where(&mut self, at: Vec3, elapsed: f32, wanted: &mut dyn FnMut(&Piece) -> bool) -> Vec<Piece> {
            let (across, reach) = (Vec2::new(at.x, at.z), MAGNET * MAGNET);
            let (taken, left): (Vec<Piece>, Vec<Piece>) = self.pieces.iter().copied().partition(|piece| {
                let over = piece.height(elapsed) - at.y;
                piece.lying(elapsed)
                    && piece.at.distance_squared(across) <= reach
                    && (-REACH_UNDER..=REACH_OVER).contains(&over)
                    && wanted(piece)
            });
            self.pieces = left;
            taken
        }
    }

    /// Every generator of a game and everything they have made that still
    /// lies there: the server's online, and the device's own in a game
    /// played offline, run by the same rules either way. It rolls no dice
    /// of its own: whoever runs it hands it some, numbers from 0 up to 1.
    /// What lies about is its [`Pile`]'s (since 2026-11-18; its own until
    /// then, by the same rules).
    #[derive(Clone, Debug)]
    pub struct Mint {
        /// When each generator next comes to make one, in seconds into
        /// the fight. It comes round whether or not the generator is
        /// full ([`cap`]): full, nothing is made, and the count starts
        /// over.
        due: [f32; GENERATORS],
        /// When each team's own generator next makes an amethyst, once it
        /// has been raised to ([`FORGE_AMETHYST`]): none until then.
        forge_due: [Option<f32>; ARENA_TEAMS],
        pile: Pile,
    }

    impl Mint {
        /// Every generator as a fight begins, `elapsed` seconds into its
        /// timetable (none, but for trying the timetable out): nothing
        /// made yet, and the first of each a whole wait away, or, for the
        /// [`steady`] ones of a fight begun late, when the time says.
        pub fn new(elapsed: f32, dice: &mut dyn FnMut() -> f32) -> Self {
            let mut due = [0.0; GENERATORS];
            for (index, due) in due.iter_mut().enumerate() {
                let item = generator(index).0;
                *due = if steady(item) {
                    next_at(item, elapsed)
                } else {
                    elapsed + wait_for(item, tier(item, elapsed), dice)
                };
            }
            Self {
                due,
                forge_due: [None; ARENA_TEAMS],
                pile: Pile::new(),
            }
        }

        /// Everything lying on the pads.
        pub fn pieces(&self) -> &[Piece] {
            self.pile.pieces()
        }

        /// How many things the generator numbered `index` holds.
        pub fn held(&self, index: usize) -> usize {
            u8::try_from(index).map_or(0, |generator| self.pile.held(generator))
        }

        /// Runs every generator up to `elapsed` seconds into the fight
        /// (which says what tier each is at): what they made. A
        /// generator whose time has come makes one, unless it holds its
        /// [`cap`], and either way begins its next wait: full, it makes
        /// nothing until something has been taken and its time comes
        /// round again (until late on 2026-11-11 its count stood still while it
        /// was full, and the next thing was a whole wait on from when one
        /// was taken; the count is written over the sapphire and the
        /// amethyst ones now, and one that stood still at nothing would
        /// say so for as long as nobody came).
        ///
        /// `forge` is how far each team has raised its own generator
        /// ([`Upgrades::forge`]): its silver and its gold come that many
        /// times as fast ([`FORGE_SPEED`]), from its next wait on, and
        /// from [`FORGE_AMETHYST`] it makes amethyst on its pad too.
        pub fn run(&mut self, elapsed: f32, forge: &[u8; ARENA_TEAMS], dice: &mut dyn FnMut() -> f32) -> Vec<Piece> {
            let mut made = Vec::new();
            for index in 0..GENERATORS {
                let (item, pad) = generator(index);
                // One at the most a go: a frame is far shorter than a wait.
                if elapsed < self.due[index] {
                    continue;
                }
                self.due[index] = if steady(item) {
                    next_at(item, self.due[index])
                } else {
                    // A team's own, as fast as the team has made it; and
                    // never a wait that has already passed, should a go
                    // come late.
                    let speed = FORGE_SPEED[usize::from(forge[index % ARENA_TEAMS]).min(FORGE_SPEED.len() - 1)];
                    (self.due[index] + wait_for(item, tier(item, elapsed), dice) / speed).max(elapsed)
                };
                if self.held(index) >= cap(item) {
                    continue;
                }
                made.push(self.make(index, item, pad, elapsed, dice));
            }
            for team in 0..ARENA_TEAMS {
                if forge[team] < FORGE_AMETHYST {
                    continue;
                }
                // Its first a whole wait on from when it was raised.
                let due = *self.forge_due[team].get_or_insert(elapsed + FORGE_AMETHYST_SECS);
                if elapsed < due {
                    continue;
                }
                self.forge_due[team] = Some(due + FORGE_AMETHYST_SECS);
                let index = GENERATORS + team;
                if self.held(index) < FORGE_AMETHYST_CAP {
                    let (item, pad) = generator(index);
                    made.push(self.make(index, item, pad, elapsed, dice));
                }
            }
            made
        }

        /// One more thing from the generator numbered `index`, on its pad.
        fn make(&mut self, index: usize, item: Item, pad: Vec2, elapsed: f32, dice: &mut dyn FnMut() -> f32) -> Piece {
            self.pile.make(index as u8, item, pad, PAD_HALF, PAD_TOP, elapsed, dice)
        }

        /// Whoever has died with their feet at `from`, `elapsed` seconds
        /// into the fight, drops what they held of sapphire and amethyst
        /// ([`spoils`]), out of `held`: thrown out from them every way,
        /// each thing to come down [`SPILL_NEAR`] to [`SPILL_FAR`] from
        /// them, of a few places at random the one furthest from the
        /// rest of what they drop, and to lie there for whoever walks
        /// into it, as what a generator makes lies on its pad. One thing
        /// for each they held, up to [`SPILL_MOST`] of an item; of more
        /// than that, that many things, the count shared out among them
        /// ([`Piece::worth`]).
        ///
        /// **Only onto ground** ("items shouldnt scatter to location
        /// where they would fall off the map"): `ground` says how high
        /// what a thing may lie on is at a place, if there is anything
        /// there that a body can walk to: land, or a plank. Nor onto
        /// anything more than a step over their feet, a stand's roof, nor
        /// further under them than [`SPILL_DOWN`]. A place none of the
        /// tries finds ground at is where they died, if that is ground,
        /// or where another of the things lies.
        ///
        /// What was dropped, which may be nothing, if they held none of
        /// either; or `None` with nowhere for any of it to lie, whoever
        /// died being out over the void, with `held` as it was: that is
        /// no death on land, and what they held goes as it does with a
        /// fall.
        pub fn spill(
            &mut self,
            held: &mut Purse,
            from: Vec3,
            elapsed: f32,
            dice: &mut dyn FnMut() -> f32,
            ground: &dyn Fn(Vec2) -> Option<f32>,
        ) -> Option<Vec<Piece>> {
            // How many things, and how many of its item each is.
            let mut things: Vec<(Item, u16)> = Vec::new();
            for item in [Item::Sapphire, Item::Amethyst] {
                let count = held[item.index()];
                let pieces = count.min(SPILL_MOST);
                for piece in 0..pieces {
                    things.push((item, count / pieces + u16::from(piece < count % pieces)));
                }
            }
            // Thrown out where there is ground for them ([`Pile::scatter`]),
            // and out of the purse only then.
            let dropped = self.pile.scatter(&things, from, elapsed, dice, ground)?;
            spoils(held);
            Some(dropped)
        }

        /// Takes away whatever was dropped and lies where `stands` says
        /// there is nothing under it any more, a plank it lay on having
        /// gone: which they were. It falls with the plank, and is
        /// nobody's.
        pub fn shed(&mut self, stands: &dyn Fn(Vec2) -> bool) -> Vec<Piece> {
            self.pile.shed(stands)
        }

        /// Whoever's feet are at `at`, `elapsed` seconds into the fight,
        /// picks up everything within reach of them: within [`MAGNET`]
        /// across the ground, and, where the thing is now, dropping or
        /// lying ([`Piece::height`]), no further over their feet than
        /// [`REACH_OVER`] nor under them than [`REACH_UNDER`]; and of
        /// what someone who died dropped, only what has come down and
        /// lain a moment ([`Piece::lying`]). What they took, no longer there for anyone
        /// else: each is as many of its item as it is worth
        /// ([`Piece::worth`]).
        pub fn pick_up(&mut self, at: Vec3, elapsed: f32) -> Vec<Piece> {
            self.pile.pick_up(at, elapsed)
        }

        /// As [`Mint::pick_up`], but only what `wanted` says yes to, asked
        /// of each thing within reach in turn: the rest is left lying.
        /// How one of the computer's players takes from a pad no more
        /// than its next purchase needs (`bots`).
        pub fn pick_up_where(&mut self, at: Vec3, elapsed: f32, wanted: &mut dyn FnMut(&Piece) -> bool) -> Vec<Piece> {
            self.pile.pick_up_where(at, elapsed, wanted)
        }
    }

    /// How long a generator of `item` at `tier` waits before its next
    /// thing, in seconds: its [`interval`], more or less its spread.
    fn wait_for(item: Item, tier: u8, dice: &mut dyn FnMut() -> f32) -> f32 {
        let (every, spread) = interval(item, tier);
        every + (dice() * 2.0 - 1.0) * spread
    }

    // ------------------------------------------------------------ the shop

    /// Which of an island's two shops: the item shop at its red stand, or
    /// the upgrade shop at its blue one.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Shop {
        Items,
        Upgrades,
    }

    /// Which shop sells `ware`.
    pub fn sold_at(ware: Ware) -> Shop {
        match ware {
            Ware::Bridge | Ware::Sword(_) | Ware::Shield(_) | Ware::Utility(_) | Ware::Compass => Shop::Items,
            Ware::Forge
            | Ware::StrongerSwords
            | Ware::BetterShields
            | Ware::HealPool
            | Ware::Trap
            | Ware::BuildSpeed => Shop::Upgrades,
        }
    }

    /// How much more a punch takes with each sword in hand, by its tier
    /// (none, basic, silver, aura: Hajun's file, "+5 dmg", "+10 dmg", "+20
    /// dmg"), and what share of every blow each shield keeps off ("6% dmg
    /// reduction", 14% and 22%).
    pub const SWORD_DAMAGE: [f32; 4] = [0.0, 5.0, 10.0, 20.0];
    pub const SHIELD_SHARE: [f32; 4] = [0.0, 0.06, 0.14, 0.22];
    /// What the upgrade shop's Stronger Swords adds to every sword of the
    /// team ("permanent +5 dmg on all swords"), and each step of its
    /// Better Shields to every shield ("permanent +6% shield dmg
    /// reduction"), of which there are four. Both go by having a sword or
    /// a shield: bare hands hit no harder, and nothing is kept off someone
    /// with no shield (mine: the file says "on all swords" and "shield
    /// dmg reduction").
    pub const STRONGER_SWORDS: f32 = 5.0;
    pub const BETTER_SHIELDS: f32 = 0.06;
    pub const BETTER_SHIELDS_MOST: u8 = 4;
    /// How far a team's generator can be raised, and how many traps can
    /// wait on its island ("1st 2nd, 3rd queued").
    pub const FORGE_MOST: u8 = 4;
    pub const TRAPS_MOST: u8 = 3;
    /// How fast someone carrying a plank walks, laying a bridge, as a
    /// share of a run, by how far their team has quickened its building
    /// at the upgrade shop, and how many steps of that there are (Hajun's
    /// file, 2026-11-14: "Default = 50% bridge speed & 50% bridge speed
    /// for your bridge bot. One upgrade - Advanced Builder (cost = 4
    /// diamond) = 75% bridge speed & 75% bridge speed for your bridge
    /// bot. Master Builder (cost = 8 diamond) = 100% ... Grand Builder
    /// (cost = 16 diamond) = 125% bridge speed & 125% bridge speed"). The
    /// builder a Bridge Builder sends walks as fast ([`builder_speed`]).
    /// That it is the whole team's, as everything else at the upgrade
    /// shop is, is mine: the file says only "your bridge bot".
    pub const BUILD_PACE: [f32; 4] = [0.5, 0.75, 1.0, 1.25];
    pub const BUILD_SPEED_MOST: u8 = 3;

    /// What share of a run someone of a team with `team` walks at while
    /// they carry a plank ([`BUILD_PACE`]).
    pub fn build_pace(team: &Upgrades) -> f32 {
        BUILD_PACE[usize::from(team.build_speed.min(BUILD_SPEED_MOST))]
    }

    /// How fast the builder a Bridge Builder sends walks, laying its
    /// bridge, for a team with `team`, in metres a second; and how fast
    /// one of the computer's players lays its own.
    pub fn builder_speed(team: &Upgrades) -> f32 {
        RUN_SPEED * build_pace(team)
    }

    /// How many of `utility` its button holds: three fireballs
    /// (Hajun's file: "can buy multiple - cap at 3 stack"), three sticks
    /// of dynamite ("can stack up to 3 times max at once"), and nine of
    /// anything else (mine: one figure in the button's corner).
    pub const fn most_held(utility: Utility) -> u8 {
        match utility {
            Utility::Fireball | Utility::Dynamite => 3,
            _ => 9,
        }
    }

    /// What `ware` costs someone with `kit` on a team with `team`: so many
    /// of an item, from Hajun's file. Nothing if it is not to be had: a
    /// sword or a shield no better than the one they have ("you cannot buy
    /// a same/lower tier sword/shield, only greater"); something to use
    /// with as many of it on its button as that holds ([`most_held`]), or,
    /// a kind they have none of, with every button taken by another kind
    /// (since 2026-11-13 a kind takes the first empty button of five;
    /// until then there was one button, and "you can only buy if the 2nd
    /// ui button is either empty or its filled by the same item u already
    /// have"); or an upgrade the team has to its last step.
    pub fn price(ware: Ware, kit: &Kit, team: &Upgrades) -> Option<(Item, u16)> {
        priced(ware, kit, team)
    }

    /// Whether whoever is in the seat `killer` gets what whoever was in
    /// the seat `victim` held of sapphire and amethyst for having killed
    /// them ([`spoils`]): one of the eight killing one of another team,
    /// with a hand, a sword, a fireball or a stick of dynamite, or by
    /// knocking them off the world, as the chat says it ("Bob killed
    /// Hannah", "Bob knocked Hannah off the world"). Not for a teammate
    /// caught in their own team's blast, nor for themselves; a zombie
    /// holds nothing and takes nothing.
    pub fn takes_spoils(killer: usize, victim: usize) -> bool {
        killer < ARENA_SEATS && victim < ARENA_SEATS && killer != victim && arena_team(killer) != arena_team(victim)
    }

    /// What whoever kills someone gets of what they held (since the night
    /// of 2026-11-14; Hajun: "if you kill an enemy in bridgewars, you
    /// should automatically get their sapphire and amethyst (not gold or
    /// iron) if they had any - not drop but just auto get it in ur
    /// inven"): every sapphire and every amethyst, taken out of `held`,
    /// the purse of whoever died, and given back as a purse with nothing
    /// else in it. Their silver and their gold go as everything went
    /// before, with the death. As Bedwars gives a killer what the killed
    /// had on them.
    pub fn spoils(held: &mut Purse) -> Purse {
        let mut won = Purse::default();
        for item in [Item::Sapphire, Item::Amethyst] {
            won[item.index()] = std::mem::take(&mut held[item.index()]);
        }
        won
    }

    /// Puts `won` into `purse`: each count as high as it goes, and no
    /// higher.
    pub fn gain(purse: &mut Purse, won: Purse) {
        for (held, more) in purse.iter_mut().zip(won) {
            *held = held.saturating_add(more);
        }
    }

    /// [`price`], worked out.
    fn priced(ware: Ware, kit: &Kit, team: &Upgrades) -> Option<(Item, u16)> {
        let step = |prices: &[u16], taken: u8| prices.get(usize::from(taken)).map(|cost| (Item::Sapphire, *cost));
        match ware {
            Ware::Bridge => Some((Item::Silver, 4)),
            Ware::Sword(1) if kit.sword < 1 => Some((Item::Silver, 10)),
            Ware::Sword(2) if kit.sword < 2 => Some((Item::Gold, 7)),
            Ware::Sword(3) if kit.sword < 3 => Some((Item::Amethyst, 4)),
            Ware::Shield(1) if kit.shield < 1 => Some((Item::Silver, 24)),
            Ware::Shield(2) if kit.shield < 2 => Some((Item::Gold, 12)),
            Ware::Shield(3) if kit.shield < 3 => Some((Item::Amethyst, 6)),
            Ware::Utility(utility)
                if match kit.slot_of(utility) {
                    Some(_) => kit.held(utility) < most_held(utility),
                    None => kit.free_slot().is_some(),
                } =>
            {
                Some(match utility {
                    Utility::Fireball => (Item::Silver, 40),
                    Utility::Builder => (Item::Amethyst, 1),
                    Utility::Speed => (Item::Amethyst, 1),
                    Utility::Invisibility => (Item::Amethyst, 2),
                    Utility::Healing => (Item::Gold, 3),
                    // 1 again since the night of 2026-11-15 (Hajun:
                    // "change the price of dynamite from 2 amethyst to
                    // 1 amethyst"); 2 for the day before it, from when
                    // it began to take the whole bridge down; 1 until
                    // then.
                    Utility::Dynamite => (Item::Amethyst, 1),
                    Utility::Leap => (Item::Amethyst, 1),
                })
            }
            Ware::Forge => step(&[2, 4, 6, 8], team.forge),
            // "Advanced Builder (cost = 4 diamond) ... Master Builder
            // (cost = 8 diamond) ... Grand Builder (cost = 16 diamond)":
            // in sapphire, as everything at the upgrade shop is (the game
            // has no diamond; it was what sapphire was called at first).
            Ware::BuildSpeed => step(&[4, 8, 16], team.build_speed),
            Ware::StrongerSwords if !team.swords => Some((Item::Sapphire, 4)),
            Ware::BetterShields => step(&[2, 3, 4, 6], team.shields),
            Ware::HealPool if !team.heal_pool => Some((Item::Sapphire, 1)),
            Ware::Trap => step(&[1, 2, 4], team.traps),
            // Once a life: with one, there is none to buy.
            Ware::Compass if !kit.compass => Some((Item::Amethyst, 2)),
            _ => None,
        }
    }

    /// Sells `ware` to someone with `purse` and `kit` on a team with
    /// `team`, if it is to be had and they hold its [`price`], which comes
    /// out of the purse: whether it is theirs. A bridge is nothing in the
    /// kit: whoever sells it gives the metres ([`BRIDGE_A_BUY`]).
    pub fn buy(ware: Ware, purse: &mut Purse, kit: &mut Kit, team: &mut Upgrades) -> bool {
        let Some((item, cost)) = price(ware, kit, team) else {
            return false;
        };
        let held = &mut purse[item.index()];
        if *held < cost {
            return false;
        }
        *held -= cost;
        match ware {
            Ware::Bridge => {}
            Ware::Sword(tier) => kit.sword = tier,
            Ware::Shield(tier) => kit.shield = tier,
            Ware::Utility(utility) => {
                // Onto the button its kind is on, or the first empty one:
                // the price was only there with one of the two.
                if let Some(slot) = kit.slot_of(utility).or(kit.free_slot()) {
                    kit.slots[slot].get_or_insert(Stack { utility, count: 0 }).count += 1;
                }
            }
            Ware::Forge => team.forge += 1,
            Ware::StrongerSwords => team.swords = true,
            Ware::BetterShields => team.shields += 1,
            Ware::HealPool => team.heal_pool = true,
            Ware::Trap => team.traps += 1,
            Ware::Compass => kit.compass = true,
            Ware::BuildSpeed => team.build_speed += 1,
        }
        true
    }

    /// How many metres of bridge one [`Ware::Bridge`] is: 25 since
    /// late on 2026-11-11, 50 for the evening before.
    pub const BRIDGE_A_BUY: f32 = 25.0;

    /// What a punch takes, from `base` (a bare hand's, [`DAMAGE`]): more
    /// with a sword, and more again if the team's swords are stronger.
    pub fn punch_damage(base: f32, kit: &Kit, team: &Upgrades) -> f32 {
        if kit.sword == 0 {
            return base;
        }
        base + SWORD_DAMAGE[usize::from(kit.sword.min(3))] + if team.swords { STRONGER_SWORDS } else { 0.0 }
    }

    /// What share of a blow the shield of someone with `kit` keeps off: its
    /// own, and the team's Better Shields on top.
    pub fn kept_off(kit: &Kit, team: &Upgrades) -> f32 {
        if kit.shield == 0 {
            return 0.0;
        }
        SHIELD_SHARE[usize::from(kit.shield.min(3))] + BETTER_SHIELDS * f32::from(team.shields.min(BETTER_SHIELDS_MOST))
    }

    /// How long a potion takes to drink, in seconds, and how much of their
    /// pace someone keeps while they drink ("shows 1.6s timer running to 0
    /// in front of ur character while ur movement is slowed 50% while
    /// drinking"); how long the speed potion lasts and how much faster it
    /// makes them ("45 second 40% speed boost"); how long the invisibility
    /// potion lasts ("30second complete invisibility ... being hit or
    /// attacking reveals u", [`Outfits::reveal`]); and what share of
    /// their health the healing potion gives back once it is drunk
    /// ("instantly heal 50% of your health", since Hajun's file of
    /// 2026-11-13; 25% until then).
    pub const DRINK_SECS: f32 = 1.6;
    pub const DRINK_SLOW: f32 = 0.5;
    pub const FAST_SECS: f32 = 45.0;
    pub const FAST: f32 = 1.4;
    pub const UNSEEN_SECS: f32 = 30.0;
    pub const HEALING: f32 = 0.5;
    /// How long the leap potion lasts, in seconds, and how many times as
    /// high its drinker jumps meanwhile ("you get 5x jump height jump
    /// boost for 45 sec"): the same fall and the same gravity, so the
    /// jump leaves the ground the square root of that many times as fast
    /// ([`leap_speed`]), and is in the air about twice as long.
    pub const LEAP_SECS: f32 = 45.0;
    pub const LEAP: f32 = 5.0;

    /// How many times as fast as a jump a leap leaves the ground: as fast
    /// as takes it [`LEAP`] times as high.
    pub fn leap_speed() -> f32 {
        LEAP.sqrt()
    }
    /// What a team's heal pool gives each of the team on its own island,
    /// and how often, in seconds ("heals 5hp every 2.5sec").
    pub const HEAL_POOL: f32 = 5.0;
    pub const HEAL_POOL_EVERY: f32 = 2.5;
    /// How long a trap blinds whoever springs it, in seconds ("for fixed
    /// 8 seconds"; until Hajun's file of 2026-11-12 it was until they had
    /// been off that island five); and how far round themselves they see
    /// meanwhile, in metres ("a very small circle around their character":
    /// the number is mine).
    pub const BLIND_SECS: f32 = 8.0;
    pub const BLIND_SIGHT: f32 = 2.6;
    /// How far a builder lays its bridge, in metres ("creating a bridge in
    /// a straight line for 100m or until it reaches a island/land"), and
    /// how fast it walks, as anyone building does, half a run, until the
    /// team has quickened its building ([`builder_speed`]).
    pub const BUILDER_METRES: f32 = 100.0;
    pub const BUILDER_SPEED: f32 = 5.0;

    /// A fireball: what its blast takes off everyone it reaches ("20
    /// dmg"); how fast it flies and how far, in metres a second and
    /// metres; how big the ball is; how high over the thrower's feet it
    /// flies and how far in front of them it sets out; and how far its
    /// blast reaches from where it bursts, across the ground and up or
    /// down. All but the damage are mine: a range that ends about where
    /// the screen does under the camera over you, which shows 11 m up and
    /// down from you, so that whoever throws it sees it burst (20 m, for
    /// its first hours, burst off the screen); a ball that can be seen
    /// coming; a blast three bodies wide.
    pub const FIREBALL_DAMAGE: f32 = 20.0;
    pub const FIREBALL_SPEED: f32 = 22.0;
    pub const FIREBALL_RANGE: f32 = 12.0;
    pub const FIREBALL_RADIUS: f32 = 0.4;
    pub const FIREBALL_HEIGHT: f32 = 1.0;
    pub const FIREBALL_AHEAD: f32 = 0.9;
    pub const BLAST_RADIUS: f32 = 3.0;
    pub const BLAST_HEIGHT: f32 = 3.0;
    /// How wide a body is to a fireball, in metres from its middle: the
    /// game's bodies', at the shoulders.
    const BODY: f32 = 0.45;

    /// A fireball on its way: where it set out from, which way it flies, as
    /// a `Plank` heads (radians from the game's -Z, about the upright), and
    /// how far it has flown. The server flies the ones thrown online and
    /// the device its own offline, by the same rules: straight and level,
    /// through whatever stands on the map, the server having no map, until
    /// it [`Flight::meets`] someone or has flown its range, where it
    /// bursts.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Flight {
        pub from: Vec3,
        pub heading: f32,
        pub flown: f32,
    }

    impl Flight {
        /// One thrown by someone whose feet are at `feet`, that way.
        pub fn thrown(feet: Vec3, heading: f32) -> Self {
            let mut flight = Self {
                from: feet + Vec3::Y * FIREBALL_HEIGHT,
                heading,
                flown: 0.0,
            };
            flight.from += flight.way() * FIREBALL_AHEAD;
            flight
        }

        /// Which way it flies, across the ground.
        pub fn way(&self) -> Vec3 {
            Vec3::new(-self.heading.sin(), 0.0, -self.heading.cos())
        }

        /// Where it is.
        pub fn at(&self) -> Vec3 {
            self.from + self.way() * self.flown
        }

        /// Flies it on for `dt` seconds: whether it has come to the end of
        /// its range, where it stops.
        pub fn fly(&mut self, dt: f32) -> bool {
            self.flown = (self.flown + FIREBALL_SPEED * dt).min(FIREBALL_RANGE);
            self.flown >= FIREBALL_RANGE
        }

        /// Whether it has met a body whose feet are at `feet`.
        pub fn meets(&self, feet: Vec3) -> bool {
            let at = self.at();
            let across = Vec2::new(at.x - feet.x, at.z - feet.z).length();
            across <= FIREBALL_RADIUS + BODY && at.y >= feet.y - 0.2 && at.y <= feet.y + 2.0
        }
    }

    /// Whether a fireball bursting at `centre` reaches a body whose feet
    /// are at `feet`.
    pub fn in_blast(centre: Vec3, feet: Vec3) -> bool {
        let across = Vec2::new(centre.x - feet.x, centre.z - feet.z).length();
        across <= BLAST_RADIUS + BODY && (centre.y - (feet.y + 1.0)).abs() <= BLAST_HEIGHT
    }

    /// A stick of dynamite (since 2026-11-13; Hajun's file: "similar to
    /// default dynamite attack for dynamike in brawlstars, hold&drag&letgo
    /// mechanic like fireball ... with ability to change the total
    /// distance with the magnitude of the hold&drag&letgo, and when fired,
    /// it will throw an arched dynamite ... after landing on the
    /// ground/bridge, it will show a circular blast radius range around it
    /// and it will shine bright red 3 time before it explode, and when it
    /// explode, it will do 20 damage to player in that blast radius and
    /// destroy that blast radius region of the bridge"). What its blast
    /// takes off everyone of another team it reaches ("20 damage"), and
    /// how far it reaches across the ground, which is also how far from
    /// it a plank's middle may be for the plank to go with it
    /// ([`super::nav::plank_in_blast`]); the nearest and the furthest it
    /// can be thrown, from the thrower's middle to where it comes down,
    /// and how far a quick tap throws it; how long it lies before it
    /// blows, and how many times it shines in that while; how high over
    /// the thrower's feet it leaves the hand, and how far in front of
    /// them; how far up the map it is drawn for each metre it has risen
    /// over the straight line of its throw ([`Lob::track`]); and how
    /// near ground it has to come down to lie on it rather than fall
    /// past. All but the damage, the three and the price are mine: a blast
    /// as wide as the fireball's, which takes four or five planks out of
    /// a bridge, about as far as a running jump clears; a throw that ends
    /// on the screen under the camera over you, which shows 11 m up and
    /// down from you, so that whoever throws it sees where it comes down;
    /// and a fuse long enough to get out of the way of, as the three
    /// flashes are there to say.
    /// The rest of a bridge a stick of dynamite has blown a hole in falls
    /// after it, plank by plank from the hole outward (since 2026-11-15;
    /// [`super::nav::Nav::cascade`]): how long after the blast the first
    /// of them goes, in seconds, for whoever stands beside the hole to
    /// set off; how fast the falling sets out along the bridge, in metres
    /// a second, how much faster it gets with every second, and the
    /// fastest it gets ([`cascade_after`]); and how
    /// much of a plank's width two planks of different bridges have to
    /// touch along, or lie over each other by, to be one bridge, as a
    /// share of it (Hajun: "even adjacent connected part of the bridge
    /// that is connected by atleast a certain percentage in a way that
    /// makes sense"): a fork off a bridge's side, a bridge landed on
    /// another, two laid side by side with their edges together, two
    /// crossing; not two that only meet at a corner, nor two with a gap
    /// between them. The numbers are mine.
    ///
    /// The falling's pace, since the night of 2026-11-15. It was 6 m a
    /// second all the way, which anyone at a run (10) left behind, by
    /// three seconds at the far end of a bridge to the middle. Hajun:
    /// "increase the speed of the dynamite's bridge explosion cascading
    /// so that it matches player's full speed running so that u need to
    /// run full speed to barely or even fail to escape the falling
    /// bridge. i guess the destruction cascade can start at the current
    /// speed but accelerate slightly (but not too much). make it so its
    /// rare to fall off a bridge if you react right away and u start
    /// running toward safety, but if its a very long bridge then i guess
    /// even if u react right away and u run full speed and ur rllly far
    /// from land then ok u will still fail to reach land". So it sets
    /// out at the 6 it had, is at a run's pace two seconds on, 16 m from
    /// the hole, and at its fastest, a fifth faster than a run, a second
    /// after that, 27 m from it. Whoever stands right beside the hole
    /// and sets off within three quarters of a second is off a bridge
    /// from an island to the middle with the falling a few metres behind
    /// them all the way, which is what they had beside the hole before
    /// (the first plank goes when it did); further along the bridge
    /// there is a second and more to set off in, where there were three.
    /// And from 27 m on it gains two metres on a runner every second: 60
    /// m and more from the land, beside the hole, half a second is too
    /// long to take, and 85 m from it nobody gets there.
    pub const CASCADE_WAIT: f32 = 0.6;
    pub const CASCADE_SPEED: f32 = 6.0;
    pub const CASCADE_QUICKENS: f32 = 2.0;
    pub const CASCADE_TOP: f32 = 12.0;
    pub const CASCADE_JOIN: f32 = 0.3;
    /// The longest after a blast any plank of its bridge falls, in
    /// seconds: whatever is further from the hole than the falling gets
    /// in this long goes with it then, all together. A time is sent in
    /// thousandths of a second in a `u16` ([`super::Blasted::later`]),
    /// which holds 65 of them; and no bridge anyone has built is as long
    /// as this is at the falling's pace, 700 m.
    pub const CASCADE_LONGEST: f32 = 60.0;

    /// How long after a blast the plank whose middle is `far` metres
    /// along the bridge from the hole falls, in seconds
    /// ([`super::nav::Nav::cascade`]): [`CASCADE_WAIT`], and then as long
    /// as the falling takes to come that far, setting out at
    /// [`CASCADE_SPEED`], faster by [`CASCADE_QUICKENS`] with every
    /// second until it is going at [`CASCADE_TOP`], and at that from
    /// there on. No longer than [`CASCADE_LONGEST`].
    pub fn cascade_after(far: f32) -> f32 {
        let far = far.max(0.0);
        let (from, top, by) = (CASCADE_SPEED, CASCADE_TOP.max(CASCADE_SPEED), CASCADE_QUICKENS);
        // (Not quickening at all, it is the one pace all the way.)
        let quickening = if by > 0.0 { (top - from) / by } else { 0.0 };
        let covered = (from + top) * 0.5 * quickening;
        let taken = if far < covered {
            // far = from * t + by * t * t / 2, for t.
            ((from * from + 2.0 * by * far).sqrt() - from) / by
        } else {
            quickening + (far - covered) / if by > 0.0 { top } else { from }
        };
        (CASCADE_WAIT + taken).min(CASCADE_LONGEST)
    }
    pub const DYNAMITE_DAMAGE: f32 = 20.0;
    pub const DYNAMITE_RADIUS: f32 = 3.0;
    pub const DYNAMITE_NEAREST: f32 = 2.5;
    pub const DYNAMITE_FURTHEST: f32 = 10.0;
    pub const DYNAMITE_TAP: f32 = 4.5;
    pub const DYNAMITE_FUSE: f32 = 1.2;
    pub const DYNAMITE_FLASHES: u32 = 3;
    pub const DYNAMITE_HAND: f32 = 1.2;
    pub const DYNAMITE_AHEAD: f32 = 0.5;
    pub const DYNAMITE_LEAN: f32 = 1.0;
    /// Which way across the ground, in its (x, z), is up the map: north,
    /// the top of the screen under the camera fixed over you in a game.
    pub const UP_THE_MAP: Vec2 = Vec2::NEG_Y;
    pub const DYNAMITE_CATCH: f32 = 0.5;
    /// What a stick falls by on its way, in metres a second a second: the
    /// game's own gravity on the way up. With how long it flies, this is
    /// how high it goes ([`Lob::apex`]).
    const LOB_GRAVITY: f32 = 22.0;

    /// A stick of dynamite from the hand to the ground and until it blows:
    /// where it left the hand, where it comes down, and for how long it
    /// has been going. The server lobs the ones thrown online and the
    /// device its own offline, by the same rules: over whatever stands on
    /// the map, down in [`Lob::secs`], and blown [`DYNAMITE_FUSE`] after
    /// that, if there was ground for it to lie on; whoever runs it says
    /// whether there was.
    ///
    /// **It is thrown straight at where it comes down, in an arch, and
    /// drawn as an arch is on a map** ([`Lob::track`]): up the screen as
    /// it rises and back down it as it falls, by [`DYNAMITE_LEAN`] of
    /// however high it is over the straight line of its throw, as a game
    /// seen from a little to the south would show anything thrown. The
    /// camera of this game is straight over you, and from there a throw
    /// that only rose and fell would be seen going along a straight line,
    /// however high it went (Hajun: "instead of one straight rectangular
    /// hit range indicator its like an long arched curved rectangular",
    /// "it will throw an arched dynamite"). Thrown to the left or the
    /// right, its arch is seen whole; thrown up the screen or down it,
    /// it goes along a straight line, a little further on or behind.
    /// **Never to one side of its throw as such** (since 2026-11-13,
    /// Hajun: "i want the dynamites hit indicator's curved part to be
    /// curved only in the Y axis (up and down) not the sideways axis. its
    /// a arch projectile not a boomarang"): for its first day it swung out
    /// to the thrower's right and back, whichever way it was thrown,
    /// which thrown up the screen was a curve to the right and thrown to
    /// the right a sag down the screen.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Lob {
        pub from: Vec3,
        pub to: Vec3,
        pub age: f32,
    }

    impl Lob {
        /// One thrown by someone whose feet are at `feet`, the way
        /// `heading` says, as a `Plank` heads, to come down `far` from
        /// their middle, as near and as far as one can be thrown, on
        /// ground `floor` high.
        pub fn thrown(feet: Vec3, heading: f32, far: f32, floor: f32) -> Self {
            let way = Vec3::new(-heading.sin(), 0.0, -heading.cos());
            let far = far.clamp(DYNAMITE_NEAREST, DYNAMITE_FURTHEST);
            let down = feet + way * far;
            Self {
                from: feet + way * DYNAMITE_AHEAD + Vec3::Y * DYNAMITE_HAND,
                to: Vec3::new(down.x, floor, down.z),
                age: 0.0,
            }
        }

        /// How far it flies, across the ground, from the hand to where it
        /// comes down.
        pub fn chord(&self) -> f32 {
            Vec2::new(self.from.x, self.from.z).distance(Vec2::new(self.to.x, self.to.z))
        }

        /// How long it is in the air, in seconds: a little longer the
        /// further it is thrown, half a second close by and most of one
        /// at its furthest.
        pub fn secs(&self) -> f32 {
            0.45 + 0.04 * (self.chord() + DYNAMITE_AHEAD)
        }

        /// How high it rises over the straight line from the hand to
        /// where it comes down, in metres: as anything thrown for that
        /// long would.
        pub fn apex(&self) -> f32 {
            LOB_GRAVITY * self.secs().powi(2) / 8.0
        }

        /// How high it is over the straight line from the hand to where
        /// it comes down, `share` of the way through its flight, from 0
        /// to 1: nothing at either end, and [`Lob::apex`] at the middle.
        pub fn rise(&self, share: f32) -> f32 {
            let share = share.clamp(0.0, 1.0);
            4.0 * self.apex() * share * (1.0 - share)
        }

        /// Where over the ground it is drawn `share` of the way through
        /// its flight, from 0 to 1: on the straight line from the hand to
        /// where it comes down, and up the map from there by
        /// [`DYNAMITE_LEAN`] of how high it has risen ([`lob_track`]).
        pub fn track(&self, share: f32) -> Vec2 {
            lob_track(Vec2::new(self.from.x, self.from.z), Vec2::new(self.to.x, self.to.z), self.apex(), UP_THE_MAP, share)
        }

        /// Where it is drawn: on its way, or lying where it came down.
        pub fn at(&self) -> Vec3 {
            let share = (self.age / self.secs()).clamp(0.0, 1.0);
            let across = self.track(share);
            let height = self.from.y + (self.to.y - self.from.y) * share + self.rise(share);
            Vec3::new(across.x, height, across.y)
        }

        /// Moves it on by `dt` seconds.
        pub fn tick(&mut self, dt: f32) {
            self.age += dt;
        }

        /// Whether it has come down.
        pub fn down(&self) -> bool {
            self.age >= self.secs()
        }

        /// How long until it blows, in seconds, once it is down: its fuse.
        pub fn fuse_left(&self) -> f32 {
            (self.secs() + DYNAMITE_FUSE - self.age).clamp(0.0, DYNAMITE_FUSE)
        }

        /// Whether its fuse has burnt down: it blows.
        pub fn blown(&self) -> bool {
            self.age >= self.secs() + DYNAMITE_FUSE
        }

        /// How bright it shines, from 0 to 1, as it lies: three times
        /// before it blows ([`DYNAMITE_FLASHES`]), each a swell and a
        /// fade, the last at its brightest as it goes. Nothing in the air.
        pub fn shine(&self) -> f32 {
            if !self.down() {
                return 0.0;
            }
            let burnt = 1.0 - self.fuse_left() / DYNAMITE_FUSE;
            let beat = (burnt * DYNAMITE_FLASHES as f32).fract();
            (beat * std::f32::consts::PI).sin().max(0.0)
        }
    }

    /// Where over the ground a stick thrown from `from` to come down at
    /// `to`, rising `apex` over the straight line between them at the
    /// middle of its flight, is drawn `share` of the way through it
    /// ([`Lob::track`]): on that line, and `up` from it, the way the top
    /// of the screen is, by [`DYNAMITE_LEAN`] of how high it has risen.
    /// What the band that aims one is drawn along, too, in whatever space
    /// that is built in.
    pub fn lob_track(from: Vec2, to: Vec2, apex: f32, up: Vec2, share: f32) -> Vec2 {
        let share = share.clamp(0.0, 1.0);
        from + (to - from) * share + up * (DYNAMITE_LEAN * apex * 4.0 * share * (1.0 - share))
    }

    /// Whether a stick of dynamite coming down at `to`, across the ground,
    /// has ground to lie on, `ground` saying where there is any: under
    /// where it comes down, or within a hand's reach of it all round
    /// ([`DYNAMITE_CATCH`]), so that one thrown at the very edge of a
    /// bridge stays on it. With none it falls past, into the void, and
    /// never blows.
    pub fn caught(to: Vec2, ground: impl Fn(Vec2) -> bool) -> bool {
        ground(to) || (0..8).any(|spoke| ground(to + Vec2::from_angle(spoke as f32 * TAU / 8.0) * DYNAMITE_CATCH))
    }

    /// Whether a stick of dynamite blowing at `centre` reaches a body
    /// whose feet are at `feet`.
    pub fn in_dynamite(centre: Vec3, feet: Vec3) -> bool {
        let across = Vec2::new(centre.x - feet.x, centre.z - feet.z).length();
        across <= DYNAMITE_RADIUS + BODY && (centre.y - (feet.y + 1.0)).abs() <= BLAST_HEIGHT
    }

    /// What has happened in a go of [`Outfits::tick`]: who a heal pool has
    /// healed, by seat, and whose trap has blinded whom, by seat and the
    /// team whose island it was.
    #[derive(Clone, PartialEq, Debug, Default)]
    pub struct Ticked {
        pub healed: Vec<usize>,
        pub sprung: Vec<(usize, u8)>,
    }

    /// What everyone in a game has and what every team has bought, with
    /// the rules that change them: the server's online, and the device's
    /// own in a game played offline, as the [`Mint`] is.
    #[derive(Clone, PartialEq, Debug, Default)]
    pub struct Outfits {
        pub kits: Vec<Kit>,
        pub teams: [Upgrades; ARENA_TEAMS],
        /// How much longer each seat is blind, in seconds, and whose
        /// island it last sprang a trap on and has not stepped off since.
        blind_left: Vec<f32>,
        trapped_on: Vec<Option<u8>>,
        /// How long each team's heal pool has been filling.
        pool: [f32; ARENA_TEAMS],
    }

    impl Outfits {
        /// Room for everyone up to `seats`, with nothing.
        pub fn seat(&mut self, seats: usize) {
            if self.kits.len() < seats {
                self.kits.resize(seats, Kit::default());
                self.blind_left.resize(seats, 0.0);
                self.trapped_on.resize(seats, None);
            }
        }

        /// What the one in `seat` has: nothing, for a seat nobody is in.
        pub fn kit(&self, seat: usize) -> Kit {
            self.kits.get(seat).copied().unwrap_or_default()
        }

        /// What the team of the one in `seat` has bought: nothing, for a
        /// zombie, which is on nobody's team.
        pub fn team(&self, seat: usize) -> Upgrades {
            self.teams.get(usize::from(arena_team(seat))).copied().unwrap_or_default()
        }

        /// Sells `ware` to the one in `seat`, out of `purse` ([`buy`]).
        pub fn buy(&mut self, seat: usize, ware: Ware, purse: &mut Purse) -> bool {
            self.seat(seat + 1);
            let Some(team) = self.teams.get_mut(usize::from(arena_team(seat))) else {
                return false;
            };
            buy(ware, purse, &mut self.kits[seat], team)
        }

        /// The one in `seat` uses one `utility` off the button it is on:
        /// whether they had one. The button is empty once none is left
        /// and nothing drunk from it still lasts ([`Outfits::tick`]).
        pub fn spend(&mut self, seat: usize, utility: Utility) -> bool {
            let stack = self
                .kits
                .get_mut(seat)
                .and_then(|kit| kit.slots.iter_mut().flatten().find(|stack| stack.utility == utility && stack.count > 0));
            match stack {
                Some(stack) => {
                    stack.count -= 1;
                    true
                }
                None => false,
            }
        }

        /// The one in `seat` has drunk a `potion` off its button,
        /// `elapsed` seconds into the fight: which it was, if they had
        /// one. The speed and the invisibility last from now; the healing
        /// is the caller's to give ([`HEALING`]). Whoever asks has taken
        /// [`DRINK_SECS`] over it already: the drinking is the drinker's
        /// own device's.
        pub fn drink(&mut self, seat: usize, potion: Utility, elapsed: f32) -> Option<Utility> {
            if potion.aimed() || !self.spend(seat, potion) {
                return None;
            }
            let kit = &mut self.kits[seat];
            match potion {
                Utility::Speed => kit.fast_until = (elapsed + FAST_SECS).ceil() as u16,
                Utility::Invisibility => kit.unseen_until = (elapsed + UNSEEN_SECS).ceil() as u16,
                Utility::Leap => kit.leap_until = (elapsed + LEAP_SECS).ceil() as u16,
                _ => {}
            }
            Some(potion)
        }

        /// The one in `seat` has died: their sword, what was on their
        /// second button and whatever they had drunk are gone ("all items
        /// are lost upon death"), and they see again. Their shield stays
        /// ("sword is lost on death, shield carries on"), as what their
        /// team has bought does.
        pub fn die(&mut self, seat: usize) {
            let Some(kit) = self.kits.get_mut(seat) else {
                return;
            };
            *kit = Kit {
                shield: kit.shield,
                ..Kit::default()
            };
            self.blind_left[seat] = 0.0;
            self.trapped_on[seat] = None;
        }

        /// The one in `seat` has hit someone or something, or been hit:
        /// if they were unseen, they are seen again ("being hit or
        /// attacking reveals u"). Whether they were.
        pub fn reveal(&mut self, seat: usize) -> bool {
            match self.kits.get_mut(seat) {
                Some(kit) if kit.unseen_until != 0 => {
                    kit.unseen_until = 0;
                    true
                }
                _ => false,
            }
        }

        /// What a punch by the one in `hitter` takes off the one in
        /// `victim`, from a bare hand's `base`: more with a sword, less
        /// through a shield.
        pub fn punch(&self, base: f32, hitter: usize, victim: usize) -> f32 {
            self.through(victim, punch_damage(base, &self.kit(hitter), &self.team(hitter)))
        }

        /// What gets through to the one in `victim` of a blow of `damage`.
        pub fn through(&self, victim: usize, damage: f32) -> f32 {
            damage * (1.0 - kept_off(&self.kit(victim), &self.team(victim)))
        }

        /// Moves the potions, the traps and the heal pools on by `dt`
        /// seconds, `elapsed` seconds into the fight, with every living
        /// player where they stand, by seat (the computer's too; never a
        /// zombie): a potion that has run out is over, and the button
        /// empty with it once none is left; an enemy stepping onto the
        /// island of a team with a trap waiting is blinded for
        /// [`BLIND_SECS`], the trap spent, and springs no other there
        /// until they have stepped off it and on again (mine: the file
        /// says only that a trap is used up by one enemy); and every
        /// [`HEAL_POOL_EVERY`] a team's pool heals whoever of the team
        /// stands on its island.
        pub fn tick(&mut self, dt: f32, elapsed: f32, standing: &[(usize, Vec2)]) -> Ticked {
            let mut ticked = Ticked::default();
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
                // A button with none left on it is empty, unless what was
                // drunk from it still lasts: the speed potion's while they
                // run fast, the invisibility's while they are unseen, the
                // leap potion's while they leap.
                let (fast, unseen, leaping) = (kit.fast_until != 0, kit.unseen_until != 0, kit.leap_until != 0);
                for slot in &mut kit.slots {
                    let lasting = |stack: &Stack| match stack.utility {
                        Utility::Speed => fast,
                        Utility::Invisibility => unseen,
                        Utility::Leap => leaping,
                        _ => false,
                    };
                    if slot.is_some_and(|stack| stack.count == 0 && !lasting(&stack)) {
                        *slot = None;
                    }
                }
            }
            for (kit, left) in self.kits.iter_mut().zip(&mut self.blind_left) {
                if *left > 0.0 {
                    *left -= dt;
                    if *left <= 0.0 {
                        *left = 0.0;
                        kit.blind = false;
                    }
                }
            }
            let island_of = |at: Vec2| (0..ARENA_TEAMS).find(|&team| ARENA_ISLANDS[team].distance(at) <= ARENA_ISLAND_RADIUS);
            for &(seat, at) in standing {
                if seat >= ARENA_SEATS || seat >= self.kits.len() {
                    continue;
                }
                let on = island_of(at);
                if self.trapped_on[seat].is_some_and(|island| on != Some(usize::from(island))) {
                    self.trapped_on[seat] = None;
                }
                if let Some(island) = on
                    && island != usize::from(arena_team(seat))
                    && self.trapped_on[seat].is_none()
                    && self.teams[island].traps > 0
                {
                    self.teams[island].traps -= 1;
                    self.trapped_on[seat] = Some(island as u8);
                    self.blind_left[seat] = BLIND_SECS;
                    self.kits[seat].blind = true;
                    ticked.sprung.push((seat, island as u8));
                }
            }
            for team in 0..ARENA_TEAMS {
                if !self.teams[team].heal_pool {
                    continue;
                }
                self.pool[team] += dt;
                if self.pool[team] < HEAL_POOL_EVERY {
                    continue;
                }
                self.pool[team] -= HEAL_POOL_EVERY;
                ticked.healed.extend(
                    standing
                        .iter()
                        .filter(|(seat, at)| {
                            *seat < ARENA_SEATS && usize::from(arena_team(*seat)) == team && island_of(*at) == Some(team)
                        })
                        .map(|(seat, _)| *seat),
                );
            }
            ticked
        }
    }

    /// How far from a shop ([`ARENA_SHOPS`], [`ARENA_UPGRADE_SHOPS`])
    /// someone may be, where they last said they stood, for the server to
    /// sell to them, in metres: the outline in front of the stand reaches
    /// 1.5 m from its middle, and the rest is for the way they may have
    /// gone since.
    pub const SHOP_SLACK: f32 = 5.0;

    /// Whether someone standing at `at` across the ground is near enough
    /// to a `shop` to buy from it, as the server checks it. An island's
    /// two shops are 10 m apart: at either, someone is not at the other.
    pub fn at_a_shop(at: Vec2, shop: Shop) -> bool {
        let shops = match shop {
            Shop::Items => &ARENA_SHOPS,
            Shop::Upgrades => &ARENA_UPGRADE_SHOPS,
        };
        shops.iter().any(|shop| shop.distance(at) < SHOP_SLACK)
    }

    // --------------------------------------------------------- the zombies

    /// The team of the zombies of sudden death ([`Happening::SuddenDeath`]):
    /// nobody's, so everyone is theirs to go for and they are everyone's.
    /// They sit in the seats past the players' eight (`arena_team`).
    pub const ZOMBIE_TEAM: u8 = super::ARENA_TEAMS as u8;
    /// The most zombies there are (Hajun, 2026-11-16: "more and more
    /// spawn, exponentially until like maybe max 50 zombies"; twelve, all
    /// at once, until then). How many there are by when is [`zombies_at`].
    pub const ZOMBIES: usize = 50;
    /// How many zombies come as sudden death begins, and how long their
    /// number takes to double, in seconds: "starting slow, and more and
    /// more spawn, exponentially". Two at 15:00, a third 21 s later, four
    /// at 15:36, eight at 16:12, sixteen at 16:48, thirty two at 17:24,
    /// and all fifty by 17:47. The numbers are not Hajun's.
    pub const ZOMBIES_FIRST: f32 = 2.0;
    pub const ZOMBIES_DOUBLE: f32 = 36.0;
    /// What a zombie is called over its head, and the look it wears, which
    /// the game draws as the body everyone has, gone green.
    pub const ZOMBIE_NAME: &str = "Zombie";
    pub const ZOMBIE_LOOK: &str = "zombie";
    /// How long a zombie that has gone into the void is gone, in seconds,
    /// before it comes up again where it first did: the one way one is
    /// ever got rid of, and not for long.
    pub const ZOMBIE_RESPAWN_SECS: f32 = 12.0;
    /// How fast a zombie comes at whoever it goes for, in metres a second:
    /// slower than anyone runs ([`RUN_SPEED`]), so that it can be got away
    /// from, though not for long on an island.
    pub const ZOMBIE_SPEED: f32 = 6.0;
    /// What a zombie's blow takes off whoever it lands on, before their
    /// shield has taken its share (Hajun, 2026-11-16: "maybe deal like 20
    /// dmg per hit"; a bare fist's 12 until then).
    pub const ZOMBIE_DAMAGE: f32 = 20.0;
    /// How much of a punch's knock a zombie takes, as a share: it is not
    /// to be batted off an island as a player is. Knocked as anyone is,
    /// the one that rose on each team's island, 4 m from its middle, was
    /// over the edge at the first punch thrown on the run, which carries a
    /// body 7 m: all four of a game's were gone within three seconds of
    /// rising, and again each time they stood again (seen on a server on
    /// this PC, 2026-11-11). **Nothing hurts one** (since the night of
    /// 2026-11-16; Hajun: "zombie should be invincible"): a blow, a blast
    /// or a bus only moves it, or lays it flat.
    pub const ZOMBIE_KNOCK: f32 = 0.3;

    /// How many zombies there are `elapsed` seconds into the fight: none
    /// before sudden death ([`SUDDEN_DEATH`]), [`ZOMBIES_FIRST`] as it
    /// begins, twice as many every [`ZOMBIES_DOUBLE`] seconds after, and
    /// never more than [`ZOMBIES`]. They come one at a time as this goes
    /// up, each where [`zombie_spot`] has it.
    pub fn zombies_at(elapsed: f32) -> usize {
        let since = elapsed - SUDDEN_DEATH;
        if since < 0.0 {
            return 0;
        }
        let risen = ZOMBIES_FIRST * (since / ZOMBIES_DOUBLE).min(16.0).exp2();
        (risen.floor() as usize).min(ZOMBIES)
    }

    /// How many places the zombies come up at ([`zombie_spot`]): four in
    /// the middle of the map, one on each of the smaller islands and one
    /// on each team's, each used again by the twelfth zombie after.
    const ZOMBIE_PLACES: usize = 12;

    /// The ground the zombie numbered `index` comes up on, as the middle
    /// of a disc and how far it reaches: the middle of the map, out to the
    /// edge of its paving, one of the smaller islands, or a team's. It
    /// goes wherever it can walk from there (the game's `horde`): until
    /// the night of 2026-11-16 each kept to this.
    pub fn zombie_home(index: usize) -> (Vec2, f32) {
        let (round, place) = (index % ZOMBIES / ZOMBIE_PLACES, index % ZOMBIES % ZOMBIE_PLACES);
        // Which of the four of its kind: one on from the round before,
        // so that no one team's island has each round's first.
        let which = (place % 4 + round) % 4;
        match place / 4 {
            0 => (Vec2::ZERO, TOWN_RADIUS),
            1 => (SAPPHIRE_PADS[which], SAPPHIRE_ISLAND_RADIUS),
            _ => (ARENA_ISLANDS[which], ARENA_ISLAND_RADIUS),
        }
    }

    /// How far the middle of the map reaches for a zombie, in metres: the
    /// paving ends 76 m out.
    pub const TOWN_RADIUS: f32 = 74.0;

    /// Where the zombie numbered `index` comes up, and comes up again from
    /// the void: "all over the map". The first four in the middle, on the
    /// path round the pond, between the roads out; the next four on the
    /// smaller islands, 4 m from each one's pad, toward the middle of the
    /// map; the next on the teams' islands, 4 m to one side of each one's
    /// middle; and so on round again, each round a little further along
    /// the path, or a fifth of the way round the pad or the island's
    /// middle, so that no two come up in one place.
    pub fn zombie_spot(index: usize) -> Vec2 {
        let index = index % ZOMBIES;
        let (round, place) = (index / ZOMBIE_PLACES, index % ZOMBIE_PLACES);
        let which = (place % 4 + round) % 4;
        let (home, _) = zombie_home(index);
        let turned = Vec2::from_angle(round as f32 * TAU / 5.0);
        match place / 4 {
            0 => {
                let angle = which as f32 * TAU / 4.0 + TAU / 8.0 + (round as f32 - 2.0) * TAU / 30.0;
                Vec2::from_angle(angle) * super::TOWN_ARRIVAL_RADIUS
            }
            1 => home + turned.rotate(-home.normalize() * 4.0),
            _ => home + turned.rotate(Vec2::new(home.y, -home.x).normalize() * -4.0),
        }
    }

    // ----------------------------------------------------------- the buses

    /// How many buses drive round the middle of the map: one to each lane
    /// of the ring round the pond ([`LANES`]; since 2026-11-17, Hajun: "rn
    /// the bus in the bridge wars minigame is just 2 buses that goes
    /// around the outmost lane, i want 4 bus, one in each lane (there are
    /// 4 lanes) at varying intervals"; two on the outside lane, half a lap
    /// apart, from 2026-11-16, when Hajun asked for them: "in the bridge
    /// wars minigame map, i want 2 buses to go 2x normal speed around the
    /// road. and when a person is standing in front of the moving bus, the
    /// bus 'squishes' that person"). The town's own bus (the game's
    /// `car`).
    pub const BUSES: usize = 4;
    /// The number the first of them goes by as a car someone is riding
    /// ([`Spot::aboard`](super::Spot::aboard)), the next the one after:
    /// past the town's four (the game's `car::CARS`), so that a number is
    /// one car on every device and to the server.
    pub const FIRST_BUS: u8 = 4;

    /// One of the lanes round the ring: how far out from the middle of the
    /// map the middle of a bus goes round it, in metres, and whether it
    /// goes round anticlockwise seen from above, or clockwise.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Lane {
        pub out: f32,
        pub anticlockwise: bool,
    }

    impl Lane {
        /// How far it is round, in metres.
        pub fn length(&self) -> f32 {
            TAU * self.out
        }

        /// 1 going anticlockwise, -1 clockwise.
        fn turn(&self) -> f32 {
            if self.anticlockwise { 1.0 } else { -1.0 }
        }
    }

    /// The lanes, a bus to each, the outermost first. The ring round the
    /// pond is two halves, road from 44 to 56 m out and from 59 to 71, with
    /// a strip down the middle between them and a dashed line down the
    /// middle of each, 50 and 65 m out; each half is two lanes, one either
    /// side of its line (the game's `roads`, which the game's tests hold
    /// these to). Keeping right, as Korea drives, the outer half goes
    /// anticlockwise seen from above and the inner half clockwise, as on
    /// any road with a strip down its middle: **mine** (Hajun said four
    /// lanes, not which way each goes; the bus of the day before went
    /// anticlockwise on the outside lane, the first here). Each the middle
    /// of a bus, which is a little inside the line its wheels go round:
    /// 67.87 m out for wheels on 68, as the town's own planner has a bus
    /// on the outside lane.
    pub const LANES: [Lane; BUSES] = [
        Lane { out: 67.87, anticlockwise: true },
        Lane { out: 61.88, anticlockwise: true },
        Lane { out: 52.83, anticlockwise: false },
        Lane { out: 46.84, anticlockwise: false },
    ];

    /// How fast a bus goes, in metres a second, with nothing in its way:
    /// twice the 7 the town's buses went at when it was asked for ("2x
    /// normal speed"), as the two of 2026-11-16 went, and left as it is
    /// since the town's own were slowed to three quarters on 2026-11-21
    /// ([`cars::TOP_SPEED`](super::cars::TOP_SPEED), which was for the
    /// town's alone). The same in every lane, so that a bus
    /// round an inner lane, which is shorter, comes round sooner: every 21
    /// s round the innermost, 30 round the outermost ("at varying
    /// intervals", with where each begins thrown for every game,
    /// [`Buses::new`]).
    pub const BUS_SPEED: f32 = 14.2;
    /// How hard a bus slows for a crossing someone is on, in metres a
    /// second each second: the gentlest it brakes, beginning where this
    /// stops it short of the crossing, 17 m before it from full speed, in
    /// 2.4 s; harder only for someone who steps on later, as hard as it
    /// takes. And how quickly it gathers speed again once nobody is on it:
    /// to full speed in 3.6 s, over 25 m. Mine.
    pub const BUS_BRAKE: f32 = 6.0;
    pub const BUS_GATHERS: f32 = 4.0;
    /// How far short of a crossing someone is on a bus stops, its front,
    /// in metres: as far as the town's stop short of theirs (the game's
    /// `roads::STOP_SHORT`).
    pub const BUS_STOP_SHORT: f32 = 1.5;

    /// The crossings painted across the ring, one at each of its corners
    /// across both its halves (Hajun's: twelve stripes to each, 8 m long,
    /// lying along the road), which a bus stops short of while anyone is
    /// on one (since 2026-11-17; Hajun: "the bus will stop before the
    /// crosswalk if there is a person on the crosswalk until they cross it
    /// - there are 4 crosswalks, if no player on that crosswalk, just same
    /// speed pass through crosswalk. It will not stop for ppl who are not
    /// on the crosswalk but rather on the road and go over them like
    /// now"): how far round each is from east, anticlockwise seen from
    /// above, in radians; how far out its middle is, in metres; and how
    /// far it goes each way from its middle, along the road and across it.
    /// The game's tests hold these to the map's paint.
    pub const CROSSINGS: [f32; 4] = [TAU / 8.0, 3.0 * TAU / 8.0, 5.0 * TAU / 8.0, 7.0 * TAU / 8.0];
    pub const CROSSING_OUT: f32 = 57.51;
    pub const CROSSING_ALONG: f32 = 4.0;
    pub const CROSSING_ACROSS: f32 = 12.15;
    /// How far outside the paint the middle of someone's body can be and
    /// they still be on it, in metres: their feet on its edge (a body's
    /// legs are 0.42 m across). Mine.
    pub const CROSSING_GRACE: f32 = 0.2;

    /// Half the length and half the width of a bus, in metres, and how
    /// high the road it drives is: Hajun's `bus.glb` is 12.5 m long and
    /// 3.7 wide, made about the middle of its footprint, facing the
    /// game's -Z, its wheels' bottoms at its own height 0 (the game's
    /// tests hold these to the model, and the road's height to the map).
    pub const BUS_HALF_LENGTH: f32 = 6.25;
    pub const BUS_HALF_WIDTH: f32 = 1.848;
    pub const BUS_ROAD: f32 = 0.0;
    /// How high over the road the feet of someone a bus's front still
    /// hits can be, in metres: its roof is 3.3 m up, and someone higher
    /// than this comes down on it. A jump (1.64 m) is nowhere near.
    pub const BUS_TALL: f32 = 3.0;
    /// How long someone a bus has run over lies flat on the road, in
    /// seconds, and what it takes off their health as it does ("the
    /// person is squashed on the road for 10 second. they cannot move for
    /// 10second ... and they also take an initial -20hp dmg"): through
    /// their shield, as a fireball's 20 is.
    pub const SQUASH_SECS: f32 = 10.0;
    pub const SQUASH_DAMAGE: f32 = 20.0;
    /// How far outside a bus's side the middle of a body may be and still
    /// be run over by its front, in metres: within the bus's width and a
    /// little, so that someone it only clips with a corner is shoved
    /// aside, not flattened. And how far behind its front face a body may
    /// have been a moment before, in metres, and still have been in front
    /// of it: pressed up against its nose. Someone at its side, however
    /// near its front, is further back than that: its front doorway is
    /// from 4.9 to 6.1 m ahead of its middle, and a body in it no further
    /// ahead than 5.65 ("obviously u shouldnt be squished when u are on
    /// the side of the bus trying to go in the bus from the side, only the
    /// front of the bus should squish").
    pub const BUS_GRAZE: f32 = 0.15;
    pub const BUS_NOSE: f32 = 0.25;

    /// Which crossing someone whose feet are at `at` is on, if any: the
    /// middle of their body on its paint, or within [`CROSSING_GRACE`] of
    /// it, and their feet no higher over the road than a bus hits
    /// ([`BUS_TALL`]), nor fallen away under it. Whoever is on one, a bus
    /// stops for ([`Buses::drive`]) and none runs over.
    pub fn crossing_at(at: Vec3) -> Option<usize> {
        if at.y > BUS_ROAD + BUS_TALL || at.y < BUS_ROAD - 1.0 {
            return None;
        }
        CROSSINGS.iter().position(|&round| {
            let (out, along) = (Vec2::new(round.cos(), -round.sin()), Vec2::new(-round.sin(), -round.cos()));
            let off = Vec2::new(at.x, at.z) - out * CROSSING_OUT;
            off.dot(along).abs() <= CROSSING_ALONG + CROSSING_GRACE
                && off.dot(out).abs() <= CROSSING_ACROSS + CROSSING_GRACE
        })
    }

    /// Where a bus in `lane` stands by a crossing, as turns round the
    /// ring from the crossing's middle line the way the bus goes, in
    /// radians: stopped short of it, its front [`BUS_STOP_SHORT`] short of
    /// the crossing's near edge, all of its front; and past it, all of its
    /// front over the far edge. The crossing's edges are lines across the
    /// road either side of its middle line ([`CROSSING_ALONG`]); a bus is
    /// a box on the lane, its front face [`BUS_HALF_LENGTH`] ahead of its
    /// middle, and its corner nearest the middle of the map is the first
    /// of it to reach a crossing, the other the last to leave one.
    fn by_a_crossing(lane: &Lane) -> (f32, f32) {
        let out = lane.out;
        // How far past the crossing's middle line, the way the bus goes,
        // the furthest and the least far corner of its front are, with
        // its middle `turned` round from the line.
        let ahead = |turned: f32, side: f32| out * turned.sin() + side * BUS_HALF_WIDTH * turned.sin().abs() + BUS_HALF_LENGTH * turned.cos();
        let solve = |side: f32, past: f32| {
            let (mut low, mut high) = (-1.0f32, 1.0f32);
            for _ in 0..60 {
                let half = (low + high) * 0.5;
                if ahead(half, side) < past {
                    low = half;
                } else {
                    high = half;
                }
            }
            (low + high) * 0.5
        };
        (solve(1.0, -(CROSSING_ALONG + BUS_STOP_SHORT)), solve(-1.0, CROSSING_ALONG))
    }

    /// The next crossing a bus in `lane`, `round` metres round it, comes
    /// to: the nearest ahead of it whose far edge its front has not passed;
    /// and how far round its lane it stands stopped short of that, in the
    /// same metres as `round`, which is behind `round` once its front is
    /// past where it would stop, at the crossing or on it.
    fn next_crossing(lane: &Lane, round: f64) -> (usize, f64) {
        let (short, past) = by_a_crossing(lane);
        let (out, length) = (f64::from(lane.out), f64::from(lane.length()));
        let mut next = (0, f64::INFINITY);
        for (crossing, &angle) in CROSSINGS.iter().enumerate() {
            let line = f64::from(lane.turn()) * f64::from(angle) * out;
            let ahead = (line + f64::from(past) * out - round).rem_euclid(length);
            if ahead < next.1 {
                next = (crossing, ahead);
            }
        }
        let (crossing, ahead) = next;
        (crossing, round + ahead - f64::from(past - short) * out)
    }

    /// Where one of the buses is: the middle of its footprint, across the
    /// ground, and which way it faces and goes, a direction across the
    /// ground one long. Where each is, is how it is going ([`Buses`]).
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Bus {
        pub at: Vec2,
        pub way: Vec2,
    }

    impl Bus {
        /// The bus in `lane`, `round` metres round it the way it goes,
        /// from east: heading north there going anticlockwise, south
        /// going clockwise.
        pub fn on(lane: &Lane, round: f64) -> Bus {
            let turned = (f64::from(lane.turn()) * round / f64::from(lane.out)).rem_euclid(std::f64::consts::TAU) as f32;
            let (sin, cos) = turned.sin_cos();
            Bus {
                at: Vec2::new(cos, -sin) * lane.out,
                way: Vec2::new(-sin, -cos) * lane.turn(),
            }
        }

        /// Which way its right is, across the ground: the kerb's side,
        /// where its doorways are, which is away from the pond in the
        /// outer half and toward it in the inner.
        pub fn right(&self) -> Vec2 {
            Vec2::new(-self.way.y, self.way.x)
        }

        /// Where `point`, across the ground, is from the bus: how far to
        /// its right of the line down its middle, and how far ahead of
        /// its middle.
        pub fn local(&self, point: Vec2) -> Vec2 {
            let off = point - self.at;
            Vec2::new(off.dot(self.right()), off.dot(self.way))
        }

        /// Where a point that far to its right and that far ahead of its
        /// middle is, across the ground.
        pub fn world(&self, local: Vec2) -> Vec2 {
            self.at + self.right() * local.x + self.way * local.y
        }

        /// The turn about the upright that has the game's -Z face the
        /// way it goes, in radians: how its model is turned, which faces
        /// -Z.
        pub fn yaw(&self) -> f32 {
            (-self.way.x).atan2(-self.way.y)
        }

        /// Where its model stands and how it is turned: its footprint's
        /// middle on the lane, its wheels on the road. What a place on it
        /// is a place from ([`Spot::aboard`](super::Spot::aboard),
        /// [`Bus::carries`]).
        pub fn stands(&self) -> (Vec3, Quat) {
            (Vec3::new(self.at.x, BUS_ROAD, self.at.y), Quat::from_rotation_y(self.yaw()))
        }

        /// Where someone riding it at `on`, a place in its own space, is
        /// in the world.
        pub fn carries(&self, on: Vec3) -> Vec3 {
            let (at, turn) = self.stands();
            at + turn * on
        }

        /// Whether the bus, come from where it was a moment ago,
        /// `before`, to here, has run over a body `radius` wide that had
        /// its feet at `was` then and has them at `now`: **its front has
        /// come to someone who was in front of it.** The body was ahead
        /// of its front face, or pressed against it ([`BUS_NOSE`]); the
        /// face has now reached the body; and the body's middle is within
        /// the bus's width ([`BUS_GRAZE`]), its feet no higher than a bus
        /// hits ([`BUS_TALL`]) nor fallen away under the road. Whoever
        /// stands in its way, steps in front of it, or is knocked or
        /// comes down there.
        ///
        /// Never someone at its side: they were not ahead of its front,
        /// whether they walk into its flank, into one of its doorways or
        /// alongside it, nor anyone it is carrying, inside it or on its
        /// roof. Never by a bus standing still: it is the front coming on
        /// that does it. (Nor, but that is for whoever asks to see to,
        /// anyone of the players on a crossing ([`crossing_at`]), whom a
        /// bus stops for.)
        pub fn runs_over(&self, before: &Bus, was: Vec3, now: Vec3, radius: f32) -> bool {
            if now.y > BUS_ROAD + BUS_TALL || now.y < BUS_ROAD - 1.0 {
                return false;
            }
            let (then, here) = (before.local(Vec2::new(was.x, was.z)), self.local(Vec2::new(now.x, now.z)));
            // How far the bus itself has come on since: a bus that has
            // not is no more than a wall to walk into.
            let come_on = (self.at - before.at).dot(self.way);
            come_on > 1e-4
                && then.y >= BUS_HALF_LENGTH - BUS_NOSE
                && here.y <= BUS_HALF_LENGTH + radius
                && here.x.abs() <= BUS_HALF_WIDTH + BUS_GRAZE
        }

        /// Where a body `radius` wide with its middle at `at`, which the
        /// bus is to be solid to, is put so that none of it is in the
        /// bus: out at the side it is nearer, level with where it was
        /// along the bus; or nowhere, clear of it already. For the
        /// computer's players, which have no model of the bus to bump
        /// into (the game's own bodies bump into every triangle of it).
        /// Not for someone its front has just come to
        /// ([`Bus::runs_over`], asked first).
        pub fn beside(&self, at: Vec2, radius: f32) -> Option<Vec2> {
            let local = self.local(at);
            let out = BUS_HALF_WIDTH + radius;
            if local.y.abs() >= BUS_HALF_LENGTH || local.x.abs() >= out {
                return None;
            }
            let side = if local.x < 0.0 { -1.0 } else { 1.0 };
            Some(self.world(Vec2::new(side * out, local.y)))
        }

        /// Which lane's band of road `at`, across the ground, is in, a
        /// body `radius` wide counted: where standing is being in the way
        /// of that lane's bus. The bands are apart: between them is the
        /// strip down the ring's middle and, 0.8 m wide for a body of the
        /// computer's players, each half's line.
        pub fn sweeps(at: Vec2, radius: f32) -> Option<usize> {
            let out = at.length();
            LANES.iter().position(|lane| (out - lane.out).abs() < BUS_HALF_WIDTH + BUS_GRAZE + radius)
        }
    }

    /// How a bus goes from one moment on (since 2026-11-17, when they began
    /// to stop for whoever is on a crossing, and so went no longer by the
    /// clock alone): from when, by the clock the buses go by, in seconds
    /// since 1970 (the game's `clock`, which online is the server's); how
    /// far round its lane it had come then, in metres from east the way it
    /// goes, counted on and on; how fast it was going, in metres a second;
    /// and how quickly it was gathering speed, in metres a second each
    /// second, or, less than nothing, slowing, up to [`BUS_SPEED`] and
    /// down to standing, and then going on as that. What a game changes
    /// as a bus slows for a crossing, stops short of it and goes on
    /// ([`Buses::drive`]); in between, the bus is where this has it on
    /// every device and on the server alike, with nothing sent.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
    pub struct Leg {
        pub from: f64,
        pub round: f64,
        pub speed: f32,
        pub gathers: f32,
    }

    impl Leg {
        /// How far round its lane the bus is `clock`, and how fast it is
        /// going: as it was going from when this began, never before.
        pub fn at(&self, clock: f64) -> (f64, f32) {
            let secs = (clock - self.from).max(0.0);
            let (top, speed, gathers) = (f64::from(BUS_SPEED), f64::from(self.speed.clamp(0.0, BUS_SPEED)), f64::from(self.gathers));
            // Changing until it is at full speed or standing, and then
            // going on as it is.
            let until = match gathers {
                more if more > 0.0 => (top - speed) / more,
                less if less < 0.0 => speed / -less,
                _ => f64::INFINITY,
            };
            let changing = secs.min(until);
            // (Exactly at full speed or standing, once it has got there.)
            let reached = match () {
                _ if secs < until => (speed + gathers * changing).clamp(0.0, top),
                _ if gathers > 0.0 => top,
                _ => 0.0,
            };
            let round = self.round + speed * changing + 0.5 * gathers * changing * changing + reached * (secs - changing);
            (round, reached as f32)
        }

        /// How far round its lane it comes to stand, slowing as it is;
        /// none if it is not.
        fn stands_at(&self) -> Option<f64> {
            (self.gathers < 0.0).then(|| {
                let speed = f64::from(self.speed.clamp(0.0, BUS_SPEED));
                self.round + speed * speed / (2.0 * f64::from(-self.gathers))
            })
        }
    }

    /// How a game's buses are going, each by its number, from 0: the last
    /// change to how each goes ([`Leg`]). On the game's own entity beside
    /// its [`ArenaRound`](super::ArenaRound), and sent to the room whenever
    /// one changes: a bus slowing for someone on a crossing, stopping, or
    /// going on. Offline a device has its own.
    #[derive(bevy::prelude::Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
    pub struct Buses(pub [Leg; BUSES]);

    impl Buses {
        /// A game's buses as it begins, `clock`: each going at full speed
        /// from a place on its lane thrown for it (`throw` gives 0 to 1),
        /// so that they are neither abreast nor evenly apart, and no two
        /// games' alike ("at varying intervals").
        pub fn new(clock: f64, throw: &mut impl FnMut() -> f32) -> Buses {
            Buses(std::array::from_fn(|number| Leg {
                from: clock,
                round: f64::from(throw().clamp(0.0, 1.0) * LANES[number].length()),
                speed: BUS_SPEED,
                gathers: 0.0,
            }))
        }

        /// Where the bus numbered `number` is `clock`.
        pub fn at(&self, number: usize, clock: f64) -> Bus {
            let number = number % BUSES;
            Bus::on(&LANES[number], self.0[number].at(clock).0)
        }

        /// How far round its lane the bus numbered `number` has come
        /// `clock`, in metres, counted on and on, and how fast it is going.
        pub fn going(&self, number: usize, clock: f64) -> (f64, f32) {
            self.0[number % BUSES].at(clock)
        }

        /// In how many seconds the front of the bus numbered `number` comes
        /// level with `at`, going round as it goes, `clock`: by how far
        /// round its lane from its front `at` is, wherever across the road;
        /// and as though it went on from now, gathering speed as fast as
        /// it does, whatever it is doing (one standing at a crossing, or
        /// slowing for one, goes on the moment nobody is on it). For
        /// someone about to cross.
        pub fn reaches_in(&self, number: usize, clock: f64, at: Vec2) -> f32 {
            let lane = &LANES[number % BUSES];
            let (_, speed) = self.going(number, clock);
            let front = self.at(number, clock).world(Vec2::new(0.0, BUS_HALF_LENGTH));
            // How far round from the bus's front `at` is, the way it goes:
            // anticlockwise seen from above is a turn from x toward -z.
            let angle = |point: Vec2| (-point.y).atan2(point.x);
            let metres = (lane.turn() * (angle(at) - angle(front))).rem_euclid(TAU) * lane.out;
            let gathering = (BUS_SPEED * BUS_SPEED - speed * speed) / (2.0 * BUS_GATHERS);
            if metres <= gathering {
                (-speed + (speed * speed + 2.0 * BUS_GATHERS * metres).sqrt()) / BUS_GATHERS
            } else {
                (BUS_SPEED - speed) / BUS_GATHERS + (metres - gathering) / BUS_SPEED
            }
        }

        /// One look at the crossings, `clock`, with whether anyone is on
        /// each (`crowded`, by [`crossing_at`]): **every bus stops short of
        /// the next crossing ahead of it while anyone is on it**, slowing
        /// as gently as stops it there with its front [`BUS_STOP_SHORT`]
        /// short ([`BUS_BRAKE`]; harder for someone who has stepped on
        /// late, and at once if its front is past where it would stop
        /// already, at the crossing or on it, so that none is ever driven
        /// onto anyone on one); stands while anyone is on it; and goes on,
        /// gathering speed ([`BUS_GATHERS`]), once nobody is. A crossing
        /// nobody is on it goes over at full speed. Whether how any of
        /// them is going has changed, which is what is then sent.
        pub fn drive(&mut self, clock: f64, crowded: [bool; 4]) -> bool {
            let mut changed = false;
            for (number, leg) in self.0.iter_mut().enumerate() {
                let (round, speed) = leg.at(clock);
                let (crossing, stop) = next_crossing(&LANES[number], round);
                let stand = Leg { from: clock, round, speed: 0.0, gathers: 0.0 };
                let new = if crowded[crossing] {
                    let short = stop - round;
                    if leg.stands_at().is_some_and(|stands| stands <= stop + 1e-3) || (speed <= 0.0 && leg.gathers <= 0.0) {
                        // Stopping there already, or standing.
                        None
                    } else if short <= 1e-3 || speed <= 0.0 {
                        // At the line or past it, or about to go on from
                        // standing: stopped, there.
                        Some(stand)
                    } else {
                        // As gently as stops it there, once that is as
                        // gentle as it brakes; harder, on its way to it
                        // already and not stopping in time.
                        let needed = (f64::from(speed) * f64::from(speed) / (2.0 * short)) as f32;
                        (needed >= BUS_BRAKE || leg.gathers < 0.0).then_some(Leg { from: clock, round, speed, gathers: -needed })
                    }
                } else if leg.gathers < 0.0 || (leg.gathers == 0.0 && speed < BUS_SPEED) {
                    // Nobody on it: on it goes.
                    Some(Leg { from: clock, round, speed, gathers: BUS_GATHERS })
                } else {
                    None
                };
                if let Some(new) = new {
                    *leg = new;
                    changed = true;
                }
            }
            changed
        }
    }

    /// The buses as a game has them at one moment, for the computer's
    /// players to mind ([`bots`](super::bots)): how each is going, and the
    /// clock.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Traffic {
        pub buses: Buses,
        pub clock: f64,
    }

    impl Traffic {
        /// Where the bus numbered `number` is.
        pub fn at(&self, number: usize) -> Bus {
            self.buses.at(number, self.clock)
        }

        /// How fast the bus numbered `number` is going.
        pub fn speed(&self, number: usize) -> f32 {
            self.buses.going(number, self.clock).1
        }

        /// In how many seconds its front comes level with `at`
        /// ([`Buses::reaches_in`]).
        pub fn reaches_in(&self, number: usize, at: Vec2) -> f32 {
            self.buses.reaches_in(number, self.clock, at)
        }
    }

    /// A game's buses as they have gone, a little way back as well as now
    /// (since 2026-11-17): each bus's last few ways of going ([`Leg`]),
    /// the newest last, for where one was a moment ago: a rider's place on
    /// it, as they said it, and someone's word that one ran them over.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct BusLog {
        legs: [Vec<Leg>; BUSES],
    }

    impl BusLog {
        /// How many of each bus's ways of going are kept: seconds' worth.
        const KEPT: usize = 8;

        /// Takes in how the buses are going now: each one's, if it is not
        /// the last kept. One that began before that is another game's,
        /// and that bus's are begun again from it.
        pub fn take(&mut self, buses: &Buses) {
            for (kept, leg) in self.legs.iter_mut().zip(buses.0) {
                if kept.last() == Some(&leg) {
                    continue;
                }
                if kept.last().is_some_and(|last| leg.from < last.from) {
                    kept.clear();
                }
                kept.push(leg);
                if kept.len() > Self::KEPT {
                    kept.remove(0);
                }
            }
        }

        /// How the buses are going now, once any have been taken in.
        pub fn now(&self) -> Option<Buses> {
            let mut buses = Buses([Leg::default(); BUSES]);
            for (leg, kept) in buses.0.iter_mut().zip(&self.legs) {
                *leg = *kept.last()?;
            }
            Some(buses)
        }

        /// Where the bus numbered `number` was `clock`: by how it was
        /// going then, or by the first way of going kept, before that.
        pub fn at(&self, number: usize, clock: f64) -> Option<Bus> {
            Some(Bus::on(&LANES[number], self.going(number, clock)?.0))
        }

        /// How far round its lane the bus numbered `number` had come
        /// `clock`, in metres, counted on and on, and how fast it was
        /// going ([`BusLog::at`]).
        pub fn going(&self, number: usize, clock: f64) -> Option<(f64, f32)> {
            let kept = self.legs.get(number)?;
            let leg = kept.iter().rev().find(|leg| leg.from <= clock).or(kept.first())?;
            Some(leg.at(clock))
        }
    }
}

/// Where the player in `seat` stands on arriving on `venue`, and which way
/// they face. On the PvP map, on their island ([`arena_island`]): the first
/// on each island at its marker, and the rest round that [`ISLAND_RING`]
/// out, every one facing the middle, down the bridge. Anywhere else, on a
/// ring round the middle, facing in — round the pond, in the town
/// ([`TOWN_ARRIVAL_RADIUS`]) — the first eight seats each an eighth of the
/// turn on from the one before, the next eight between them, and past
/// sixteen a ring further out. The server works out arrivals with this too,
/// so that everyone sees you arrive where you do. Each of the four games of
/// 2026-11-18 has its own, in its own module: Blend In's is the town's.
pub fn arrival(venue: Venue, seat: usize) -> (Vec3, Quat) {
    match venue {
        Venue::BlendIn => return blend_in::arrival(seat),
        Venue::KnockOff => return knock_off::arrival(seat),
        Venue::BridgeRace => return bridge_race::arrival(seat),
        Venue::LastStand => return last_stand::arrival(seat),
        Venue::Town | Venue::Home | Venue::Lobby | Venue::Plot(_) | Venue::Arena => {}
    }
    if venue == Venue::Arena {
        let island = ARENA_ISLANDS[arena_island(seat)];
        let round = seat / ARENA_ISLANDS.len();
        // Seven round the one in the middle, and past them round again, a
        // little further out and turned on by half a place.
        let places = 7;
        let out = (round + places - 1) / places;
        let at = if round == 0 {
            island
        } else {
            let place = (round - 1) % places;
            let angle = (place as f32 + 0.5 * (out - 1) as f32) * TAU / places as f32;
            island + Vec2::new(angle.cos(), angle.sin()) * (ISLAND_RING + 2.0 * (out - 1) as f32)
        };
        let toward = -island.normalize();
        // A turn of `yaw` about the upright points -Z at (-sin yaw, -cos yaw).
        let yaw = (-toward.x).atan2(-toward.y);
        return (Vec3::new(at.x, 0.0, at.y), Quat::from_rotation_y(yaw));
    }
    let eighth = TAU / 8.0;
    let ring = seat / 16;
    let within = seat % 16;
    let angle = (within % 8) as f32 * eighth
        + if within >= 8 { eighth * 0.5 } else { 0.0 }
        + ring as f32 * eighth * 0.25;
    let first = if venue == Venue::Town { TOWN_ARRIVAL_RADIUS } else { ARRIVAL_RADIUS };
    let radius = first + ring as f32 * 2.0;
    let turn = Quat::from_rotation_y(angle);
    // Facing in: a turn of `angle` about the upright points -Z at the middle
    // from where it puts +Z.
    (turn * Vec3::new(0.0, 0.0, radius), turn)
}

/// Where the person in `seat` stands on arriving on `venue`: in the town,
/// where a seat is a plot ([`land::HOMES`]), in the middle of it
/// ([`land::arrival`]); anywhere else, and in the town past its plots,
/// where [`arrival`] has the seat. The town's
/// own crowd, which each device runs for itself, arrives round the pond as
/// it always has ([`arrival`]).
pub fn arrival_of_a_person(venue: Venue, seat: usize) -> (Vec3, Quat) {
    match venue {
        Venue::Town => land::arrival(seat).unwrap_or_else(|| arrival(venue, seat)),
        _ => arrival(venue, seat),
    }
}

// ---------------------------------------------------------------- players

/// One player in a room, as the server describes them: sent once, when they
/// come into your room. Flat and primitive-only: a look is a name, never an
/// asset path.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[component(immutable)]
pub struct Member {
    /// Unique and stable for as long as they are connected, so that whatever
    /// is said about them can say who it is about.
    pub id: u64,
    /// What everyone else sees them called: what the game writes
    /// ([`text`]).
    pub name: String,
    /// Which of the game's looks they wear.
    pub look: String,
    /// The computer is playing this seat, not a person.
    pub ai: bool,
}

/// Where a body stands and which way it faces: whole centimetres, and a turn
/// in 65,536ths of a circle. About twelve bytes once postcard has packed it.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Default, Debug)]
pub struct Spot {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub yaw: u16,
    /// The car the body is riding, by its number, if it is: on it, or in the
    /// air off it and carried along with it until it comes down on anything
    /// else. The place and the turn are then in the car's own space, which
    /// moves with it, and not the island's: someone watching puts the body
    /// on the car as they see it, wherever that is (the game's `car`), so
    /// that a rider is never seen beside it, nor behind it by the time the
    /// update takes to come.
    pub aboard: Option<u8>,
    /// The body has come to a stop here. Someone watching draws it standing
    /// here from then on, rather than carrying it on the way it was going
    /// while they wait for the next update, which is not coming.
    pub still: bool,
    /// On a device, the server tick of the update that brought it, so that
    /// updates can be laid out in the time they were sent rather than the
    /// time they happened to arrive. Never sent.
    #[serde(skip)]
    pub tick: u32,
}

impl Spot {
    /// Where a body at `translation`, turned by `rotation`, stands.
    pub fn of(translation: Vec3, rotation: Quat) -> Self {
        let forward = rotation * Vec3::NEG_Z;
        let yaw = (-forward.x).atan2(-forward.z);
        Self {
            x: centimetres(translation.x),
            y: centimetres(translation.y),
            z: centimetres(translation.z),
            yaw: ((yaw / TAU).rem_euclid(1.0) * 65_536.0).round() as u32 as u16,
            aboard: None,
            still: false,
            tick: 0,
        }
    }

    /// The same, as a place on the car numbered `car`, in its own space, or,
    /// with none, on the island.
    pub fn aboard(self, car: Option<u8>) -> Self {
        Self { aboard: car, ..self }
    }

    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32) / 100.0
    }

    /// The turn about the upright, in radians.
    pub fn yaw(&self) -> f32 {
        f32::from(self.yaw) / 65_536.0 * TAU
    }

    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw())
    }

    /// The same place, the same way and as still, on the same car or none,
    /// whenever it was sent.
    pub fn same(&self, other: &Self) -> bool {
        (self.x, self.y, self.z, self.yaw, self.aboard, self.still)
            == (other.x, other.y, other.z, other.yaw, other.aboard, other.still)
    }
}

fn centimetres(metres: f32) -> i32 {
    (metres * 100.0).round() as i32
}

/// The cars that drive round the town, two buses since 2026-10-24 (the
/// game's `car`), as far as the server checks a rider's moves against them:
/// it cannot tell where they are, having no islands, but it knows how fast a
/// car can take anyone.
pub mod cars {
    /// The fastest a car goes, in metres a second, along the straight and
    /// gentle stretches of the town's roads, and twice it along an open
    /// road (the game's `roads::OPEN_ROAD`): three quarters of what it was
    /// (Hajun, 2026-11-21: "decrease the bus speed to 75% for the main
    /// lobby donnbgeurami town buses"), a little under a walk, so that
    /// someone on foot gets on with ease. 7 until then, as fast as a body
    /// walks, so that someone beside it kept up and no more. Each device
    /// works out where the buses are from this and the town's clock: an app
    /// built before the change has them elsewhere on their rounds, and
    /// draws whoever rides one on the bus as it has it.
    pub const TOP_SPEED: f32 = 5.25;
    /// How far from a car's own middle a body riding it can be, in metres:
    /// on it, which is 12.5 m long, its corners 6.5 m out, or in the air off
    /// it, carried along with it until it comes down, which a run takes
    /// another 5 m or so.
    pub const REACH: f32 = 15.0;
    /// The fastest the car anyone can drive themselves goes, in metres a
    /// second (the game's `drive`, since 2026-11-05): boosted, or reversing.
    /// The server lets every body go this fast across the ground
    /// (`server/src/players.rs`), in a car or not: nothing said a body was
    /// in one until 2026-11-21 ([`Ask::Drive`](super::Ask::Drive)), and an
    /// app from before still does not say. The car's own speeds are in the game's
    /// `assets/ui.ron`, and a test there keeps them no faster than this:
    /// 40 since 2026-11-06 (Hajun: "change the max speed to 40m/s"), 20
    /// until then.
    pub const DRIVEN_TOP_SPEED: f32 = 40.0;
    /// How fast a body can go across the ground, in metres a second, as far
    /// as the server lets it: that car, boosted, with some slack for the
    /// odd update arriving early. Every body is allowed as much, whether it
    /// has said it is in the car or not; and a device takes anyone shown
    /// going faster to have been put where they are, not to have gone
    /// there (the game's `net::leaps`).
    pub const MOST_SPEED: f32 = DRIVEN_TOP_SPEED * 1.3;
}

/// A [`Spot`] as it arrives, stamped with the tick of the update it came in.
fn deserialize_spot(ctx: &mut WriteCtx, message: &mut Bytes) -> Result<Spot> {
    let mut spot: Spot = default_deserialize(ctx, message)?;
    spot.tick = ctx.message_tick.get();
    Ok(spot)
}

// ------------------------------------------------------------ House Builder

/// House Builder's rules, which the server plays by online and the game by
/// itself offline: in one place, so that the two never differ.
pub mod rules {
    /// What every player starts with, in coins.
    pub const STARTING_BALANCE: u32 = 1000;
    /// What a game costs to join, in coins.
    pub const ENTRY: u32 = 100;
    /// How long the lobby waits for people before the computer starts taking
    /// the empty seats, in seconds: online only (Hajun's choice, 2026-10-08).
    /// Offline there is nobody to wait for.
    pub const PEOPLE_WAIT: f32 = 10.0;
    /// How long the computer takes to fill the seats left, in seconds: one at
    /// a time, evenly over it, the last as it runs out.
    pub const QUEUE_FILL: f32 = 5.0;
    /// How long a full room shows 8/8 before everyone leaves for their plot.
    pub const FULL_HOLD: f32 = 2.0;
    /// How long everyone has to build, in seconds.
    pub const BUILD_SECS: f32 = 300.0;
    /// How long everyone spends at each house rating it, in seconds.
    pub const VISIT_SECS: f32 = 15.0;
    /// The most stars a house can be given.
    pub const MOST_STARS: u8 = 5;
    /// What each place pays, in coins, first to last (Hajun's, 2026-10-08).
    pub const PRIZES: [u32; super::GAME_SEATS] = [500, 400, 300, 200, 100, 50, 25, 0];
    /// How long the results are up, in seconds, before everyone who has not
    /// asked to play again is sent back to the town.
    pub const RESULTS_SECS: f32 = 15.0;
}

/// Every plot in the order it came, first to last: by its stars, most first,
/// a tie settled by `lot`. The seats are shuffled by the lot before they are
/// put in order of their stars, and the order keeps those with the same stars
/// as the shuffle left them.
///
/// The server's lot is its game's [`Round::theme`], drawn at random when the
/// game opens and sent to everyone in it, so that every device works out the
/// same places from the stars as the server pays out on, without being told
/// them.
pub fn standings(stars: &[u32; GAME_SEATS], lot: u8) -> [u8; GAME_SEATS] {
    let mut order: [u8; GAME_SEATS] = std::array::from_fn(|seat| seat as u8);
    // Fisher and Yates' shuffle, drawing on xorshift32 seeded from the lot:
    // odd, so never the zero xorshift cannot leave.
    let mut state = u32::from(lot).wrapping_mul(0x9E37_79B9) | 1;
    for last in (1..GAME_SEATS).rev() {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        order.swap(last, (state % (last as u32 + 1)) as usize);
    }
    // A stable sort: those with the same stars stay as the shuffle put them.
    order.sort_by_key(|&plot| std::cmp::Reverse(stars[usize::from(plot)]));
    order
}

/// A game of House Builder as everyone in it sees it: how far it has got, how
/// long is left of that, and the theme.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Round {
    pub stage: Stage,
    /// Whole seconds left of this stage, rounded up: what the board shows.
    /// Once the game is over, how long the results are up for.
    pub left: u16,
    /// Which of the game's themes, by its place in `builder::THEMES`, going
    /// round again past the last. Also the lot its ties are settled by
    /// ([`standings`]).
    pub theme: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Stage {
    /// In the lobby, waiting for the room to fill.
    Queue,
    /// Full, and about to go.
    Full,
    /// Everyone on their own plot, building.
    Build,
    /// Everyone at the house on this plot, rating it.
    Visit(u8),
    /// Done: the house on `winner` came first, and these are every house's
    /// stars, by plot, which the rest of the places are worked out from
    /// ([`standings`]).
    Over { winner: u8, stars: [u32; GAME_SEATS] },
}

/// Everyone in a game of House Builder, in seat order: a builder's plot is
/// their seat.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Seats(pub Vec<Member>);

/// A piece put down on a plot in a game of House Builder: a house, a door, a
/// window, a bed. Sent once it is down, so that everyone visiting sees it;
/// never while it is being dragged.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[component(immutable)]
pub struct Built {
    /// Whose plot it is on.
    pub plot: u8,
    /// What it is: `build::PIECES`' name for it, never an asset path.
    pub kind: String,
    pub at: At,
}

/// Where a piece is on its plot.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum At {
    /// Standing on the plot: where its origin is, and its turn about the
    /// upright, in radians.
    Ground { x: f32, y: f32, z: f32, yaw: f32 },
    /// In or on a wall of the plot's house: which of its walls, counted in
    /// order of where they stand in the house, how far along it and up it,
    /// and which face, 1 or -1, for what hangs on one.
    Wall {
        wall: u8,
        along: f32,
        up: f32,
        face: i8,
    },
}

// ------------------------------------------------------------- Bridge Wars

/// A game of Bridge Wars as everyone in it sees it (since 2026-11-08, when
/// the server took the game over so that people could play it together):
/// how far it has got, how long is left of that, everyone's health by
/// seat, each team's respawn anchor, and who is out for good. Sent to the
/// room whenever any of it changes: a few dozen bytes, a few times a second
/// while anyone's health is coming back.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ArenaRound {
    pub stage: ArenaStage,
    /// Whole seconds left of this stage, rounded up: what the board shows.
    pub left: u16,
    /// Everyone's health by seat, in whole points of [`bridge_wars::HEALTH`];
    /// 0 is dead, until they stand again.
    pub health: Vec<u16>,
    /// Each team's respawn anchor, in whole points of
    /// [`bridge_wars::ANCHOR_HEALTH`]; 0 is broken.
    pub anchors: [u16; ARENA_TEAMS],
    /// The seats that are out for good, a bit to a seat: dead with their
    /// anchor broken.
    pub out: u8,
}

impl ArenaRound {
    /// Whether the player in `seat` is out for good.
    pub fn is_out(&self, seat: usize) -> bool {
        seat < 8 && self.out & (1 << seat) != 0
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum ArenaStage {
    /// In the waiting room, waiting for the room to fill.
    Waiting,
    /// Full, counting down to go.
    Countdown,
    /// Everyone on their island, fighting.
    Playing,
    /// Done: `winner` is the team left standing, or none, the time having
    /// run out with more than one.
    Over { winner: Option<u8> },
}

/// One rung of a bridge someone has built in a game of Bridge Wars (the
/// game's `bridge`): whose, which of their chains of planks, which rung of
/// it, where it lies across the ground and which way it heads. Sent once
/// as it is laid, so that everyone in the game sees the plank between it
/// and the rung before; never moved.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[component(immutable)]
pub struct Plank {
    pub seat: u8,
    pub chain: u16,
    pub index: u16,
    pub x: f32,
    pub z: f32,
    /// Which way the rung heads, across the ground, in radians from the
    /// game's -Z, as a turn about the upright.
    pub heading: f32,
}

/// One thing a generator has made in a game of Bridge Wars, lying on its
/// pad ([`bridge_wars::Mint`], since 2026-11-11): what it is, and where it
/// lies across the ground, on a pad's top ([`PAD_TOP`]). Sent once as it
/// comes up, to everyone in the game, and gone for everyone the moment
/// the server has someone pick it up (Hajun: "other player taking the
/// item from the generators make it so you cant take it too"); never
/// moved.
///
/// Or one that someone who died on land dropped (since the night of
/// 2026-11-15, [`bridge_wars::Mint::spill`]), which says so ([`Spilt`]).
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[component(immutable)]
pub struct Loot {
    pub item: Item,
    pub x: f32,
    pub z: f32,
    pub spilt: Option<Spilt>,
}

/// What is said of a thing that someone who died dropped, for every
/// device to draw it flying out from them and lying where it came down:
/// how high what it lies on is, where it was thrown from, and how many of
/// its item it is ([`bridge_wars::Piece`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Spilt {
    pub ground: f32,
    pub from_x: f32,
    pub from_y: f32,
    pub from_z: f32,
    pub worth: u16,
}

impl Loot {
    /// What the room is sent of `piece`.
    pub fn of(piece: &bridge_wars::Piece) -> Self {
        Self {
            item: piece.item,
            x: piece.at.x,
            z: piece.at.y,
            spilt: piece.from.map(|from| Spilt {
                ground: piece.ground,
                from_x: from.x,
                from_y: from.y,
                from_z: from.z,
                worth: piece.worth,
            }),
        }
    }
}

/// What everyone in a game of Bridge Wars holds, by seat: how many of each
/// [`Item`]. On the game's own entity beside its [`ArenaRound`], and sent
/// to the room whenever anyone's changes: something picked up, or
/// something bought.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Purses(pub Vec<Purse>);

/// What everyone in a game of Bridge Wars has, by seat ([`Kit`]): beside
/// the [`Purses`], and sent to the room whenever anyone's changes, so that
/// every device draws the sword in everyone's hand and the shield round
/// them, and hides whoever is unseen.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Kits(pub Vec<Kit>);

/// What each team of a game of Bridge Wars has bought at the upgrade
/// shop, by team: beside the [`Kits`].
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub struct Upgraded(pub [Upgrades; ARENA_TEAMS]);

/// A fireball in the air in a game of Bridge Wars: who threw it, by seat,
/// where it set out from, and which way it flies, as a [`Plank`] heads.
/// Sent once as it is thrown; every device flies it on from there at
/// [`bridge_wars::FIREBALL_SPEED`], and it is gone, for everyone, the
/// moment the server has it burst, where each device then has it.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[component(immutable)]
pub struct Fireball {
    pub seat: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub heading: f32,
}

/// A stick of dynamite on its way, or lying with its fuse burning, in a
/// game of Bridge Wars (since 2026-11-13): who threw it, by seat, where it
/// left the hand, and where it comes down ([`bridge_wars::Lob`]). Sent
/// once as it is thrown; every device lobs it on from there by the same
/// rules, and it is gone, for everyone, the moment the server has it blow
/// ([`Blasted`]) or fall past the ground into the void.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[component(immutable)]
pub struct Dynamite {
    pub seat: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub to_x: f32,
    pub to_y: f32,
    pub to_z: f32,
}

/// A stick of dynamite has blown up in a game of Bridge Wars: where, and
/// which planks went with it, each as whose bridge it was part of, by
/// seat, which of their bridges, and which plank of that one (the one
/// between its rung of that number and the next). Sent once as it blows
/// and kept to the end of the game, so that every device has every plank
/// that is gone, in whatever order the rest of the bridge reaches it.
/// The server says which planks: a device's own word of where a bridge
/// ends may be a rung ahead of the server's.
///
/// **And the rest of the bridge after them** (since 2026-11-15; Hajun:
/// "from that point it will destroy cascadingly the entire bridge like the
/// truly connected bridge ... but it shouldnt destroy the whole bridge
/// instantly but rather cascadingly from that initial blast point so it
/// gives ppl time to run away to safety"): every plank joined to the ones
/// the blast took, through however many others, falls in its turn, the
/// nearest first ([`nav::Nav::cascade`]). `later` is each of them as
/// `planks` has them, with how long after the blast it falls, in
/// thousandths of a second; `at` is when the stick blew, by the server's
/// clock, in seconds since 1970, which every device has (the game's
/// `clock`): so that a device shows each plank about to go and takes it
/// at its time, and one that comes into the game later, or again, has the
/// planks whose time has passed gone at once, with no blast drawn for a
/// stick that blew before it was there.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[component(immutable)]
pub struct Blasted {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub planks: Vec<(u8, u16, u16)>,
    pub at: f64,
    pub later: Vec<(u8, u16, u16, u16)>,
}

impl Blasted {
    /// The planks that fall after the blast, as [`nav::Nav::cascade`]
    /// has them, each with how long after, written as they are sent: in
    /// whole thousandths of a second, and no more than a `u16` of them.
    pub fn later_of(cascade: &[((u8, u16, u16), f32)]) -> Vec<(u8, u16, u16, u16)> {
        cascade
            .iter()
            .map(|&((seat, chain, plank), secs)| (seat, chain, plank, (secs.max(0.0) * 1000.0).round().min(f32::from(u16::MAX)) as u16))
            .collect()
    }

    /// The planks that fall after the blast, each with how many seconds
    /// after.
    pub fn later(&self) -> impl Iterator<Item = ((u8, u16, u16), f32)> + '_ {
        self.later.iter().map(|&(seat, chain, plank, after)| ((seat, chain, plank), f32::from(after) / 1000.0))
    }
}

/// The builder a Bridge Builder has sent off, walking and laying its bridge
/// (since 2026-11-14; Hajun: "right now, the bridge bot is not visible to
/// other players, just the bridge that it creates. make your bridgebot
/// visible to other players too"): whose it is, by seat. On a body the
/// server sends the game's room, with a [`Member`] in its owner's name and
/// look and a [`Spot`] that the owner's device moves ([`Ask::Builder`]), or
/// the server itself, for one of the computer's players'; gone as the
/// builder is. A device draws it as a body carrying a plank, in nobody's
/// way, and is never sent its own, which it draws itself.
#[derive(Component, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[component(immutable)]
pub struct BuilderOf(pub u8);

/// Who in a game of Bridge Wars a bus has run over and lies flat on the
/// road (since 2026-11-16, [`bridge_wars::Bus`]): each by their seat, a
/// zombie's past the players' eight, with until when, by the server's
/// clock, in seconds since 1970, which every device has (the game's
/// `clock`). On the game's own entity beside its [`ArenaRound`], and sent
/// to the room whenever it changes: someone run over, or up again, or dead
/// where they lay. A device draws each of them flat with the seconds left
/// over them, and stands them up as the server does; its own player it
/// lays flat itself the moment its own bus reaches them
/// ([`Ask::RunOver`]), and reads here only that the server has it so.
#[derive(Component, Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Flattened(pub Vec<(u8, f64)>);

impl Flattened {
    /// Until when whoever is in `seat` lies flat, if they do.
    pub fn until(&self, seat: usize) -> Option<f64> {
        self.0.iter().find(|(flat, _)| usize::from(*flat) == seat).map(|&(_, until)| until)
    }
}

/// Using what is on one of the buttons in Bridge Wars ([`Ask::Use`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Use {
    /// Throw a fireball this way, as a [`Plank`] heads.
    Fireball { heading: f32 },
    /// A builder has been sent off: its bridge comes as [`Ask::Rung`]s.
    Builder,
    /// This potion has been drunk (which, since 2026-11-13: there may be
    /// several kinds on the buttons at once).
    Potion(Utility),
    /// Throw a stick of dynamite this way, as a [`Plank`] heads, to come
    /// down this far from me, in metres (since 2026-11-13).
    Dynamite { heading: f32, far: f32 },
}

// ---------------------------------------------------------------- friends

/// Friends, groups, and what is said between them (since 2026-11-13; Hajun:
/// "I want to add a friend system. First every account need a random
/// number between 1000~9999 that is gona serve as a #1234 tag ... you should
/// be able to search the username ... they can accept or reject the add
/// friend request ... when youre in the group with ur friend, only the
/// leader can queue for a minigame, and everyone get queued at the same
/// time in the same team"). All of it is the server's, kept with the
/// accounts (`server/src/social.rs`); a device asks ([`Act`], in
/// [`Ask::Social`](super::Ask::Social)) and is told ([`News`], in
/// [`Tell::Social`](super::Tell::Social)), and shows what it was last told
/// (the game's `friends`). Offline there is none of it.
///
/// * **A tag**: every account has a number from [`TAG_FIRST`] to
///   [`TAG_LAST`] after its name, `Hajooon#1323` ([`handle`]), so that two
///   people of one name can be told apart. No two accounts have both the
///   same name, whatever its capitals, and the same tag: an account that
///   takes a name someone with its tag already has is given another tag.
/// * **Friends**: you ask someone by their account ([`Act::Befriend`]),
///   found by their name ([`Act::Find`]); they are told so in their inbox
///   ([`Post::Request`]) and say yes or no ([`Act::Answer`]). Up to
///   [`FRIENDS_MOST`].
/// * **A group**: a friend who is online can be asked into yours
///   ([`Act::Invite`], [`Post::Invite`]); whoever asks first leads it. Only
///   its leader starts a game ([`Refusal::NotLeader`](super::Refusal)), and
///   everyone of it who is free to goes in with them: into the same game,
///   and in Bridge Wars onto the same team, two by two. Up to
///   [`GROUP_MOST`]. Kept in the server's memory alone: a group is over
///   when the server restarts, or when all but one have left it.
/// * **A message** to a friend ([`Act::Write`], [`Post::Message`]), short
///   ([`MESSAGE_LONGEST`]) and of what the game writes ([`text`](super::text)),
///   kept for them until they have seen it off.
/// * **A level**, from what an account has earned playing ([`level`]).
pub mod social {
    use serde::{Deserialize, Serialize};

    use super::{Venue, profile, text};

    /// The tags there are, the number after a name: `#1000` to `#9999`.
    pub const TAG_FIRST: u16 = 1000;
    pub const TAG_LAST: u16 = 9999;
    /// The most friends one account has, and the most who can be waiting
    /// on its answer at once.
    pub const FRIENDS_MOST: usize = 100;
    pub const REQUESTS_MOST: usize = 50;
    /// The most in a group, its leader among them: two teams of Bridge
    /// Wars. Mine, Hajun not having said.
    pub const GROUP_MOST: usize = 4;
    /// How long a message may be, in widths ([`text::length`]: 80 Latin
    /// letters, or 40 Hangul syllables), and how many wait for one account
    /// at the most.
    pub const MESSAGE_LONGEST: usize = 80;
    pub const LETTERS_MOST: usize = 30;
    /// The most players a search answers with, and the longest it reads,
    /// in widths: a name with its tag after it.
    pub const FOUND_MOST: usize = 20;
    pub const SOUGHT_LONGEST: usize = profile::LONGEST + 5;

    /// What is typed to look for a player by, as it is looked for: cleaned
    /// as a name is ([`text::clean`]), so that a name is found however its
    /// letters were typed, and no longer than [`SOUGHT_LONGEST`].
    pub fn sought(typed: &str) -> String {
        let clean = text::clean(typed);
        text::fitted(&clean, SOUGHT_LONGEST).trim().to_string()
    }
    /// How long being asked into a group stands, in seconds, and how long
    /// someone whose connection has gone is kept in theirs: a phone closes
    /// its connection whenever the app is put in the background.
    pub const INVITE_SECS: f64 = 600.0;
    pub const GROUP_GRACE: f64 = 120.0;

    /// What a game earns an account toward its [`level`]: for having played
    /// it to its end, for winning it (in House Builder, for coming first,
    /// second and third), and how much makes a level. All mine: Hajun's
    /// sketch of the friends' list has a level by each name, and nothing
    /// else says what a level is.
    pub const XP_PLAYED: u32 = 20;
    pub const XP_WON: u32 = 50;
    pub const XP_PLACED: [u32; 3] = [50, 30, 15];
    pub const XP_A_LEVEL: u32 = 100;
    /// The highest level shown.
    pub const LEVEL_MOST: u16 = 999;

    /// The level of an account that has earned `xp`: 1 to start, and one
    /// more for every [`XP_A_LEVEL`].
    pub fn level(xp: u32) -> u16 {
        (1 + xp / XP_A_LEVEL).min(u32::from(LEVEL_MOST)) as u16
    }

    /// A name with its tag after it, as a friend is told who you are:
    /// `Hajooon#1323`.
    pub fn handle(name: &str, tag: u16) -> String {
        format!("{name}#{tag}")
    }

    /// `text` as a name with a tag after it, if it is one: `Hajooon#1323`
    /// is `Hajooon` and 1323. A name may have a `#` in it, so whoever
    /// looks for a player by this looks for the whole of `text` as a name
    /// as well.
    pub fn split_handle(text: &str) -> Option<(&str, u16)> {
        let (name, tag) = text.trim().rsplit_once('#')?;
        let tag: u16 = (tag.len() == 4 && tag.bytes().all(|digit| digit.is_ascii_digit()))
            .then(|| tag.parse().ok())
            .flatten()?;
        ((TAG_FIRST..=TAG_LAST).contains(&tag) && profile::name(name).is_some_and(|whole| whole == name))
            .then_some((name, tag))
    }

    /// `typed` as a message, if it is one: something, once it is cleaned
    /// ([`text::clean`]), and no longer than [`MESSAGE_LONGEST`]; with the
    /// common swear words in it starred out ([`text::masked`]).
    pub fn message(typed: &str) -> Option<String> {
        let message = text::clean(typed);
        (!message.is_empty() && text::length(&message) <= MESSAGE_LONGEST).then(|| text::masked(&message))
    }

    /// One of the games there are to be in: Bridge Wars and House Builder,
    /// and since 2026-11-18 the four Hajun asked for that day ("add these
    /// minigames: Blend In, Knock Off, Bridge Race, Last Stand"), each
    /// played at a table ([`crate::table`]), appended after the two, as a
    /// friend's presence sends it by its number ([`Presence::Playing`]).
    /// Re-exported at the crate's root.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Minigame {
        BridgeWars,
        HouseBuilder,
        /// Hide and seek in the town, the hider passing for one of its
        /// people ([`crate::blend_in`]).
        BlendIn,
        /// Aim on the ice, then everyone slides at once
        /// ([`crate::knock_off`]).
        KnockOff,
        /// Two teams bridging to the golden island ([`crate::bridge_race`]).
        BridgeRace,
        /// Wave after wave of zombies round an anchor ([`crate::last_stand`]).
        LastStand,
    }

    impl Minigame {
        /// The four games played at a table ([`crate::table`]), in the
        /// selector's order.
        pub const TABLES: [Minigame; 4] = [Minigame::BlendIn, Minigame::KnockOff, Minigame::BridgeRace, Minigame::LastStand];

        /// Whether it is one of [`TABLES`](Self::TABLES).
        pub fn at_a_table(self) -> bool {
            Self::TABLES.contains(&self)
        }

        /// The venue it is played on: Bridge Wars' map for Bridge Wars,
        /// House Builder's lobby for House Builder, where its players wait
        /// (each builder's plot is a venue of its own, [`Venue::Plot`]),
        /// and each of the four its own ([`Venue::game`] the other way
        /// round).
        pub fn venue(self) -> Venue {
            match self {
                Minigame::BridgeWars => Venue::Arena,
                Minigame::HouseBuilder => Venue::Lobby,
                Minigame::BlendIn => Venue::BlendIn,
                Minigame::KnockOff => Venue::KnockOff,
                Minigame::BridgeRace => Venue::BridgeRace,
                Minigame::LastStand => Venue::LastStand,
            }
        }
    }

    /// Where someone is, as their friends are told.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Presence {
        /// Not connected, since this time, in seconds since 1970 by the
        /// server's clock; 0 if it does not know.
        Offline { since: u64 },
        /// In the town, in this channel of it, counted from 0.
        Town { channel: u16 },
        /// At home.
        Home,
        /// In a game: still waiting for it to fill, or playing it.
        Playing { game: Minigame, queued: bool },
    }

    impl Presence {
        pub fn online(&self) -> bool {
            !matches!(self, Self::Offline { .. })
        }
    }

    /// Someone, as their friends and their group see them: who they are,
    /// what they go by, and where they are now.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct Card {
        /// Their account: [`Member::id`](super::Member::id) while they are
        /// connected, and what everything asked about them says.
        pub id: u64,
        pub name: String,
        pub tag: u16,
        pub level: u16,
        pub presence: Presence,
    }

    impl Card {
        /// `Hajooon#1323`.
        pub fn handle(&self) -> String {
            handle(&self.name, self.tag)
        }
    }

    /// What someone found by their name is to you already.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Tie {
        /// Nothing yet.
        Stranger,
        /// Your friend.
        Friend,
        /// You have asked them, and they have not answered.
        Asked,
        /// They have asked you: it is in your inbox.
        AskedYou,
    }

    /// Someone a search turned up.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct Found {
        pub card: Card,
        pub tie: Tie,
    }

    /// Something in your inbox: who it is from, and what it is.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct Mail {
        /// The server's number for it, which an answer names
        /// ([`Act::Answer`]).
        pub id: u64,
        pub from: Card,
        pub what: Post,
    }

    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub enum Post {
        /// They ask to be your friend.
        Request,
        /// They ask you into their group.
        Invite,
        /// They wrote this to you.
        Message(String),
    }

    /// A group, as everyone in it is told: who leads it, and everyone in
    /// it, the leader among them, in the order they came into it.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct Group {
        pub leader: u64,
        pub members: Vec<Card>,
    }

    /// Something asked of the server about friends ([`Ask::Social`](super::Ask::Social)).
    /// Each is answered by what it changes ([`News`]), and where it could
    /// not be done, or leaves nothing to show, by a word ([`News::Said`]).
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub enum Act {
        /// Who goes by this name, or this name and tag? [`News::Found`]
        /// answers.
        Find(String),
        /// Ask this account to be my friend; or, if they have asked me
        /// already, yes.
        Befriend(u64),
        /// No longer my friend; or, of someone I have asked who has not
        /// answered, never mind.
        Unfriend(u64),
        /// My answer to something in my inbox: yes or no to being
        /// someone's friend or going into their group, and of a message,
        /// either way, that I have seen it.
        Answer { mail: u64, yes: bool },
        /// Ask this friend into my group: mine to lead, if I am in none.
        Invite(u64),
        /// Out of my group.
        LeaveGroup,
        /// Put this account out of the group I lead.
        Kick(u64),
        /// Write this to a friend.
        Write { to: u64, text: String },
    }

    /// What the server tells a device about its friends ([`Tell::Social`](super::Tell::Social)).
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub enum News {
        /// The tag after your name, and your level: on being let in, and
        /// whenever either changes.
        Me { tag: u16, level: u16 },
        /// Your friends, all of them: on being let in, and whenever one is
        /// made or lost.
        Friends(Vec<Card>),
        /// One of your friends as they are now: gone somewhere else, or by
        /// another name.
        Friend(Card),
        /// Your inbox, all of it: on being let in, and whenever it changes.
        Inbox(Vec<Mail>),
        /// Your group, or that you are in none: on being let in, and
        /// whenever it changes.
        Group(Option<Group>),
        /// Who goes by what you looked for.
        Found { sought: String, people: Vec<Found> },
        /// What became of something you asked.
        Said(Outcome),
    }

    /// What became of something asked ([`Act`]), in a word, for the device
    /// to say in its own.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Outcome {
        /// They have been asked to be your friend.
        Asked,
        /// You are friends now.
        Befriended,
        /// No longer your friend, or no longer asked.
        Unfriended,
        /// There was nothing to do: your friend already, asked already, in
        /// your group already.
        Already,
        /// You have as many friends as an account has.
        Full,
        /// They have, or as many waiting on their answer.
        TheirsFull,
        /// There is no such player, or it is you.
        NoSuch,
        /// They are not your friend.
        NotFriend,
        /// They are not online.
        Offline,
        /// They have been asked into your group.
        Invited,
        /// Only the leader of a group can do that.
        NotLeader,
        /// The group has as many in it as a group has.
        GroupFull,
        /// What was answered is no longer there to answer: taken back, or
        /// too old.
        Stale,
        /// Your message has gone to them.
        Sent,
        /// They have as many messages waiting as an account keeps.
        InboxFull,
        /// That is no message: nothing, too long, or not plain letters.
        NotAMessage,
        /// Your group has gone into a game without you: you were not free
        /// to go, or could not pay for it.
        LeftBehind,
        /// You have been moved to the channel of the town your group is
        /// in, your land with you (since 2026-11-23; Hajun: "if u invite
        /// someone to ur group, they should move to the same channel").
        Gathered,
    }
}

// ------------------------------------------------------------------- chat

/// What is said in a room, and what a game of Bridge Wars tells everyone in
/// it as it goes (since 2026-11-14; Hajun: "I think there should be a
/// global chat in the Bridge Wars game, with notifications that pop up
/// based on game events: When anyone dies: 'Bob fell off the world.' When
/// someone kills someone: 'Bob killed Hannah with a Stone Sword,' or 'Bob
/// knocked Hannah off the world.' Team-specific notifications, like 'Bob
/// purchased Heal Pool for your team.' ... This chat box should also be in
/// the main lobby ... everyone in the lobby should see the message with
/// your name tag followed by your text, like chat on a Minecraft server").
///
/// * **A line** ([`Ask::Chat`](super::Ask::Chat)) is short and of what the
///   game writes ([`line`], [`text`](super::text): English, Korean and
///   Vietnamese); the server passes it on to everyone in the asker's room, a
///   channel of the town or a game, the asker among them
///   ([`Tell::Said`](super::Tell::Said)), no oftener than [`EVERY`] once
///   [`BURST`] have gone out together. Nothing is kept of it anywhere: a
///   device shows what it heard while it was there (the game's `chat`).
/// * **What happens in a game of Bridge Wars** ([`Feed`]) is the server's
///   to say online ([`Tell::Feed`](super::Tell::Feed)), by seat, and the
///   device's own in a game played offline; each device writes it out in
///   its own words, with every name in its team's colour.
pub mod chat {
    use serde::{Deserialize, Serialize};

    use super::{Ware, text};

    /// The longest line someone may say, in widths ([`text::length`]): 80
    /// Latin letters, or 40 Hangul syllables.
    pub const LONGEST: usize = 80;
    /// How many lines someone may say one after another, and how long each
    /// one more takes to be theirs to say, in seconds: for someone leaning
    /// on a key, not for a conversation.
    pub const BURST: f32 = 4.0;
    pub const EVERY: f32 = 1.5;
    /// How long after someone was last hit by another their fall is that
    /// other's doing, in seconds ([`Feed::Knocked`]): Minecraft's five
    /// seconds of being in a fight.
    pub const CREDIT_SECS: f32 = 5.0;

    /// Whether a line may be typed and said (since 2026-11-27, not; Hajun:
    /// "im gona disable custom text chat throughout the game in both
    /// minigames and in the lobby. there is only gona be quickchat with the
    /// chatbubble"): the server passes on no line typed
    /// ([`Ask::Chat`](super::Ask::Chat)), and the game puts up no box to
    /// type one into, nor the keyboard that took it up. All of it is still
    /// there: this one word brings it back, on both sides.
    pub const TYPED: bool = false;

    /// The most things to say at a press one list has ([`Quick`]): what a
    /// device may ask the server to say by its number.
    pub const QUICK_MOST: u8 = 16;

    /// Which list of things to say at a press a line is from (the game's
    /// `quick`): the town's, Hajun's, or the one for the games.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Lines {
        Town,
        Games,
    }

    /// One of the things to say at a press: which list, and which of it by
    /// its number there (since 2026-11-27, when they became all there is to
    /// say). The words are each device's own (`words.chat.quick` and
    /// `quick_games` in its `assets/ui.ron`), so the server passes on the
    /// number alone, which nobody can put words of their own into.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Quick {
        pub lines: Lines,
        pub line: u8,
    }

    impl Quick {
        /// Whether it is one a list could have ([`QUICK_MOST`]).
        pub fn sane(&self) -> bool {
            self.line < QUICK_MOST
        }
    }

    /// What someone said, as everyone in their room is told
    /// ([`Tell::Said`](super::Tell::Said)): a line typed, while that may be
    /// ([`TYPED`]), or one of the things to say at a press.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub enum Said {
        Typed(String),
        Quick(Quick),
    }

    /// `typed` as a line to say, if it is one: something, once it is
    /// cleaned ([`text::clean`]: its letters put together, what the game
    /// does not write left out, the spaces round it trimmed), and no longer
    /// than [`LONGEST`]; with the common swear words in it starred out
    /// ([`text::masked`]), which the server does to every line it passes
    /// on.
    pub fn line(typed: &str) -> Option<String> {
        let line = text::clean(typed);
        (!line.is_empty() && text::length(&line) <= LONGEST).then(|| text::masked(&line))
    }

    /// How many lines someone may say now, who had `had` to say `since`
    /// seconds ago: one more for every [`EVERY`] gone by, up to
    /// [`BURST`]. Saying one takes one of them.
    pub fn lines_left(had: f32, since: f32) -> f32 {
        (had + since.max(0.0) / EVERY).min(BURST)
    }

    /// What someone was killed with.
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Weapon {
        /// A bare hand.
        Fists,
        /// A sword, by its tier, 1 to 3.
        Sword(u8),
        Fireball,
        Dynamite,
    }

    /// Something that has happened in a game of Bridge Wars, for the chat:
    /// who, by seat (a zombie's is past the players' eight).
    #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Feed {
        /// They fell off the world, by nobody's doing.
        Fell { seat: u8 },
        /// They fell off the world, `by` having hit them a moment before
        /// ([`CREDIT_SECS`]).
        Knocked { seat: u8, by: u8 },
        /// `by` killed them, with this.
        Killed { seat: u8, by: u8, with: Weapon },
        /// `by` bought this at the upgrade shop, for their team, which is
        /// who is told: `step` is how far the team has it now, for what
        /// goes by steps.
        Bought { by: u8, ware: Ware, step: u8 },
        /// `by` broke `team`'s respawn anchor.
        AnchorBroken { team: u8, by: u8 },
        /// A bus ran them over, and what it took was the last of their
        /// health (since 2026-11-16; last, so that the rest are sent as
        /// they were).
        RunOver { seat: u8 },
    }

    impl Feed {
        /// Which team alone is told of it, if it is not for everyone.
        pub fn only_for(&self) -> Option<u8> {
            match *self {
                Feed::Bought { by, .. } => Some(super::arena_team(usize::from(by))),
                _ => None,
            }
        }
    }
}

// ---------------------------------------------------------------- the bag

/// What a person owns, and what of it stands on their plot of the town (the
/// game's `bag`, since 2026-11-21; Hajun: "im gonna change the emote button
/// in the lobby to a inventory/shop panel ... the first tab is a inventory
/// tab and the second tab is the shop tab").
///
/// **Things** ([`THINGS`]) are what there is to own: every piece the game
/// stands on a plot (the game's `build::PIECES`: House Builder's nineteen,
/// and since 2026-11-25 Hajun's cabinet, bathtub and end table, 56
/// downloaded things and seventeen more houses), by its name there, and
/// the cars anyone drives, 23 since 2026-11-25. Each is of a [`Group`],
/// which the panel sorts them by, and of a [`Rarity`], which is the colour
/// its name is written in; and each has its price, which for now is its
/// rarity's (Hajun: "rarity is tied to the color, but is not tied to the
/// price. but for now, set the price for each rarity items to this"), in
/// coins, or for those his list marks "(gem)" in gems ([`Money`]). Sold
/// back, a thing fetches half of it ("sell price should be 50% of buy
/// price"), in what it is sold for.
///
/// **A bag** ([`Bag`]) is one person's: their gems, how many of each thing
/// they own, and which of those stand on their plot and where. Everyone
/// begins with ten gems, one car and one street lamp ([`Bag::new`]; Hajun:
/// "make the default gem count to be 10 ... make every player own 1 car and
/// 1 street lamp by default"). Nothing earns a gem yet; a thing sold for
/// gems spends them.
///
/// **A thing that stands on the plot is out of the bag** until it is taken
/// back ([`Bag::free`]): it cannot be sold, nor stood there twice. Where it
/// stands is in the plot's own space ([`Standing`]), so that it stands the
/// same on whichever of the town's plots its owner's seat has that day.
///
/// The rules are here for both sides: the server plays by them online, with
/// the account's coins, and the game by itself offline. What is said is
/// [`Ask`] and [`Tell`], inside [`super::Ask::Bag`] and [`super::Tell::Bag`],
/// which were appended last with no new protocol: an app from before says
/// none of it and is told none of it.
pub mod bag {
    use serde::{Deserialize, Serialize};

    use super::At;

    /// How rare a thing is, the commonest first: the colour its name is
    /// written in (the game's `colours.bag`), and for now what it costs
    /// ([`Rarity::price`], or in gems [`Rarity::gems`]). Transcendent, over
    /// Mythic, since 2026-11-25 (Hajun's list: the ten-floor houses and the
    /// Yang, "i think yang car should be highest rarity").
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum Rarity {
        Common,
        Uncommon,
        Rare,
        Epic,
        Legendary,
        Mythic,
        Transcendent,
    }

    impl Rarity {
        pub const ALL: [Rarity; 7] = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
            Rarity::Mythic,
            Rarity::Transcendent,
        ];

        /// What a thing of this rarity costs for now, in coins (Hajun's
        /// numbers, 2026-11-21, and Transcendent's 2026-11-25). A thing's
        /// price is its own ([`Thing::price`]) and only begins as this.
        pub const fn price(self) -> u32 {
            match self {
                Rarity::Common => 50,
                Rarity::Uncommon => 150,
                Rarity::Rare => 400,
                Rarity::Epic => 1_000,
                Rarity::Legendary => 2_500,
                Rarity::Mythic => 15_000,
                Rarity::Transcendent => 100_000,
            }
        }

        /// What a thing of this rarity costs for now if it is sold for
        /// gems ([`Money::Gems`]; Hajun's list, 2026-11-25: "$1,000 or 50
        /// gem" for Epic, and so on).
        pub const fn gems(self) -> u32 {
            match self {
                Rarity::Common => 3,
                Rarity::Uncommon => 8,
                Rarity::Rare => 20,
                Rarity::Epic => 50,
                Rarity::Legendary => 125,
                Rarity::Mythic => 750,
                Rarity::Transcendent => 5_000,
            }
        }
    }

    /// What a thing is sold for: coins, as most things are, or gems, as
    /// those marked "(gem)" in Hajun's list are (since 2026-11-25: the
    /// houses of five, seven and nine floors).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Money {
        Coins,
        Gems,
    }

    /// What kind of thing something is: what the panel's list down its left
    /// sorts by. Nothing is a utility or a cosmetic yet.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Group {
        Buildings,
        Vehicles,
        Utilities,
        Cosmetics,
    }

    impl Group {
        pub const ALL: [Group; 4] = [Group::Buildings, Group::Vehicles, Group::Utilities, Group::Cosmetics];
    }

    /// One thing there is to own.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Thing {
        /// What it goes by over the wire and where it is kept: for a
        /// building, its piece's name in the game's `build::PIECES`. Never
        /// an asset path, and never changed once anyone owns one.
        pub kind: &'static str,
        pub group: Group,
        pub rarity: Rarity,
        /// What the shop sells one for: so many of `money`.
        pub price: u32,
        pub money: Money,
    }

    impl Thing {
        /// What the shop gives for one sold back: half its price, in the
        /// money it is sold for.
        pub const fn resale(&self) -> u32 {
            self.price / 2
        }

        /// Whether it is something that stands on a plot.
        pub const fn stands(&self) -> bool {
            matches!(self.group, Group::Buildings)
        }

        /// What one is worth in coins, whatever it is sold for: its price,
        /// or for one sold for gems what its rarity costs in coins. What a
        /// plot is ranked by (`land::worth`).
        pub const fn worth(&self) -> u32 {
            match self.money {
                Money::Coins => self.price,
                Money::Gems => self.rarity.price(),
            }
        }
    }

    const fn building(kind: &'static str, rarity: Rarity) -> Thing {
        Thing {
            kind,
            group: Group::Buildings,
            rarity,
            price: rarity.price(),
            money: Money::Coins,
        }
    }

    /// A building sold for gems, at its rarity's price in them.
    const fn gem_building(kind: &'static str, rarity: Rarity) -> Thing {
        Thing {
            price: rarity.gems(),
            money: Money::Gems,
            ..building(kind, rarity)
        }
    }

    const fn vehicle(kind: &'static str, rarity: Rarity) -> Thing {
        Thing {
            kind,
            group: Group::Vehicles,
            rarity,
            price: rarity.price(),
            money: Money::Coins,
        }
    }

    /// The street lamp, and the car anyone drives (the game's `drive`):
    /// what everyone begins with one of.
    pub const LAMP: &str = "streetlamp";
    pub const CAR: &str = "red_sports_car";

    /// Everything there is to own, in the order Hajun listed it with its
    /// rarity (`assets/itemrarities.txt`: 2026-11-21, and as he had it on
    /// 2026-11-25, with his new furniture, the 56 downloaded things, the
    /// houses of three to ten floors, the Wide Roof House and the pack's
    /// cars). The game's bag window shows them kind by kind and by rarity,
    /// in this order only among things alike (its `bag::panel::shown`,
    /// since 2026-11-25).
    pub const THINGS: &[Thing] = &[
        building(LAMP, Rarity::Legendary),
        building("singlebed", Rarity::Uncommon),
        building("doublebed", Rarity::Rare),
        building("cafechair", Rarity::Common),
        building("circletable", Rarity::Common),
        building("cafedeskedge", Rarity::Common),
        building("cafedeskcorner", Rarity::Common),
        building("softcouch", Rarity::Rare),
        building("end_table", Rarity::Common),
        building("cabinet", Rarity::Uncommon),
        building("bathtub", Rarity::Rare),
        building("classic_door", Rarity::Common),
        building("classic_door_with_window", Rarity::Common),
        building("classic_double_door", Rarity::Common),
        building("glass_door", Rarity::Common),
        building("glass_double_door", Rarity::Common),
        building("square_window", Rarity::Common),
        building("wide_window", Rarity::Common),
        // The 56 things downloaded on 2026-11-24 (the game's
        // `mybuilds/newones`), the commonest first.
        building("alarm_clock", Rarity::Common),
        building("chair", Rarity::Common),
        building("closed_umbrella", Rarity::Common),
        building("computer_mouse", Rarity::Common),
        building("cup_of_tea", Rarity::Common),
        building("dumbbell", Rarity::Common),
        building("empty_box", Rarity::Common),
        building("frying_pan", Rarity::Common),
        building("hair_dryer", Rarity::Common),
        building("hamburger", Rarity::Common),
        building("hand_rake", Rarity::Common),
        building("mug_with_office_tool", Rarity::Common),
        building("screwdriver", Rarity::Common),
        building("stapler", Rarity::Common),
        building("stool", Rarity::Common),
        building("table", Rarity::Common),
        building("tissue_box", Rarity::Common),
        building("trash_bin", Rarity::Common),
        building("watering_can", Rarity::Common),
        building("wine_glass", Rarity::Common),
        building("armchair", Rarity::Uncommon),
        building("bathroom_sink", Rarity::Uncommon),
        building("coat_rack", Rarity::Uncommon),
        building("desk", Rarity::Uncommon),
        building("desk_fan", Rarity::Uncommon),
        building("doll", Rarity::Uncommon),
        building("dresser", Rarity::Uncommon),
        building("flowers", Rarity::Uncommon),
        building("fruit_bowl", Rarity::Uncommon),
        building("globe", Rarity::Uncommon),
        building("headphones", Rarity::Uncommon),
        building("lamp_with_shade", Rarity::Uncommon),
        building("mailbox", Rarity::Uncommon),
        building("monitor", Rarity::Uncommon),
        building("painting", Rarity::Uncommon),
        building("piggy_bank", Rarity::Uncommon),
        building("potted_plant", Rarity::Uncommon),
        building("present", Rarity::Uncommon),
        building("printer", Rarity::Uncommon),
        building("propane_tank", Rarity::Uncommon),
        building("rubber_duck", Rarity::Uncommon),
        building("table_large_circular", Rarity::Uncommon),
        building("toilet", Rarity::Uncommon),
        building("barbecue", Rarity::Rare),
        building("bed", Rarity::Rare),
        building("ceiling_fan", Rarity::Rare),
        building("computer", Rarity::Rare),
        building("couch", Rarity::Rare),
        building("refrigerator", Rarity::Rare),
        building("rocking_chair", Rarity::Rare),
        building("stove", Rarity::Rare),
        building("treadmill", Rarity::Rare),
        building("grandfathers_clock", Rarity::Epic),
        building("safe", Rarity::Epic),
        building("telescope", Rarity::Epic),
        building("chandelier", Rarity::Legendary),
        // The houses: the Square House, the Flat House (Epic until
        // 2026-11-25, when his list had it Common) and the Wide Roof
        // House; and the Square House and the Roof Terrace House of two
        // floors to ten, those of five, seven and nine sold for gems.
        building("square_house", Rarity::Common),
        building("square_house_roofless", Rarity::Common),
        building("wide_roof_house", Rarity::Common),
        building("square_house_two_floor", Rarity::Uncommon),
        building("square_house_three_floor", Rarity::Rare),
        building("square_house_four_floor", Rarity::Epic),
        gem_building("square_house_five_floor", Rarity::Epic),
        building("square_house_six_floor", Rarity::Legendary),
        gem_building("square_house_seven_floor", Rarity::Legendary),
        building("square_house_eight_floor", Rarity::Mythic),
        gem_building("square_house_nine_floor", Rarity::Mythic),
        building("square_house_ten_floor", Rarity::Transcendent),
        building("square_house_roofless_two_floor", Rarity::Uncommon),
        building("square_house_roofless_three_floor", Rarity::Rare),
        building("square_house_roofless_four_floor", Rarity::Epic),
        gem_building("square_house_roofless_five_floor", Rarity::Epic),
        building("square_house_roofless_six_floor", Rarity::Legendary),
        gem_building("square_house_roofless_seven_floor", Rarity::Legendary),
        building("square_house_roofless_eight_floor", Rarity::Mythic),
        gem_building("square_house_roofless_nine_floor", Rarity::Mythic),
        building("square_house_roofless_ten_floor", Rarity::Transcendent),
        // The car everyone begins with (the pack's `italia`), and the
        // pack's other 22 (the game's `assets/cars`).
        vehicle(CAR, Rarity::Mythic),
        vehicle("gen", Rarity::Rare),
        vehicle("sedan", Rarity::Rare),
        vehicle("van", Rarity::Rare),
        vehicle("coupe", Rarity::Rare),
        vehicle("suv", Rarity::Rare),
        vehicle("jeep", Rarity::Epic),
        vehicle("mobil", Rarity::Epic),
        vehicle("police", Rarity::Epic),
        vehicle("rally", Rarity::Epic),
        vehicle("tronco", Rarity::Epic),
        vehicle("armor", Rarity::Legendary),
        vehicle("cilo", Rarity::Legendary),
        vehicle("kamaro", Rarity::Legendary),
        vehicle("ponche", Rarity::Legendary),
        vehicle("ghini", Rarity::Legendary),
        vehicle("lamb", Rarity::Legendary),
        vehicle("poly", Rarity::Legendary),
        vehicle("sportrs", Rarity::Legendary),
        vehicle("chiron", Rarity::Mythic),
        vehicle("fenyr", Rarity::Mythic),
        vehicle("imola", Rarity::Mythic),
        vehicle("yang", Rarity::Transcendent),
    ];

    /// The thing that goes by `kind`, if there is one.
    pub fn thing(kind: &str) -> Option<&'static Thing> {
        THINGS.iter().find(|thing| thing.kind == kind)
    }

    /// How many gems everyone begins with.
    pub const GEMS: u32 = 10;
    /// The most of one kind of thing anyone owns, and the most things that
    /// stand on one plot: what a plot is told of in one message.
    pub const MOST_OF_A_KIND: u16 = 99;
    pub const MOST_STANDING: usize = 100;
    /// How far from the middle of its plot a thing's own middle may stand,
    /// across the ground, in metres: half a plot, 24 m square, and a little
    /// (the game keeps the whole of it inside the plot; this only keeps
    /// what is said sane); and how far under and over the plot's ground:
    /// over the roof terrace of a house of ten floors, 45 m up (since
    /// 2026-11-25; 40 m until then).
    pub const REACH: f32 = 12.5;
    pub const DEEP: f32 = 2.0;
    pub const HIGH: f32 = 60.0;

    /// Something standing on its owner's plot: `id` is its owner's number
    /// for it, the same for as long as it stands; `kind` what it is
    /// ([`Thing::kind`]); and `at` where, **in the plot's own space**: its
    /// origin the middle of the plot on its ground, x to the right of
    /// someone standing there looking out over the plot's front (the side
    /// its sign stands on), y up, and -z out over that front, so that a turn
    /// of nothing faces a thing's front that way. A door or a window is in
    /// a wall of the plot's house, as in House Builder ([`At::Wall`]).
    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    pub struct Standing {
        pub id: u32,
        pub kind: String,
        pub at: At,
    }

    /// Why something asked of a bag was not done.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum No {
        /// There is no such thing.
        Unknown,
        /// Not the coins for it.
        Coins,
        /// Not the gems for it, sold for gems.
        Gems,
        /// As many of it as anyone may own, or as many things as may stand
        /// on a plot.
        Full,
        /// None of it in the bag: none owned, or every one standing.
        None,
        /// Not something that stands on a plot, or nowhere a thing may
        /// stand.
        Place,
        /// Not food, for something only food goes in ([`Keeps::food_only`]).
        Food,
        /// A slot with something else in it ([`stow`]).
        Taken,
        /// A piggy bank with coins in it, which is not taken back into the
        /// bag until they are taken out ([`take_back`]).
        Banked,
    }

    /// Whether `at` is somewhere a thing may be said to stand: numbers, and
    /// within reach of its plot.
    pub fn sound(at: &At) -> bool {
        let level = |y: f32| (-DEEP..=HIGH).contains(&y);
        match *at {
            At::Ground { x, y, z, yaw } => {
                [x, y, z, yaw].iter().all(|n| n.is_finite()) && x.abs() <= REACH && z.abs() <= REACH && level(y)
            }
            At::Wall { along, up, face, .. } => {
                along.is_finite() && up.is_finite() && along.abs() <= 2.0 * REACH && level(up) && (face == 1 || face == -1)
            }
        }
    }

    /// The most parts that are painted on one plot, and the longest name a
    /// part goes by, in bytes.
    pub const MOST_PAINTED: usize = 400;
    pub const PART_NAME: usize = 40;

    /// A colour a part of something standing on a plot has been painted
    /// (since 2026-11-22; Hajun: "u can select a color there and then u can
    /// tap objects and it will paint it to that color (so if u tap the
    /// blanket of the red bed, it will just change that red color to the
    /// color that u chose, or if u tap your house wall, it will just change
    /// the color of the walls"). `id` is the standing thing's
    /// ([`Standing::id`]); `part` which part of it, by the name its material
    /// has in its model ("blanket", "plain_wall"): everything of the thing
    /// that is drawn with that material is that part; and `colour` its red,
    /// green and blue as a screen has them (sRGB). A part that is not
    /// painted is as its model has it, and so is every part of a thing
    /// taken back into the bag and stood again ("color gets reset if u
    /// trash that item and place it back").
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct Paint {
        pub id: u32,
        pub part: String,
        pub colour: [u8; 3],
    }

    /// Whether `part` is a name a part may go by: something, no longer than
    /// [`PART_NAME`], and nothing a line cannot hold.
    pub fn part_named(part: &str) -> bool {
        !part.is_empty() && part.len() <= PART_NAME && !part.chars().any(char::is_control)
    }

    /// The colours on one plot: what its owner has painted of what stands
    /// there. Kept beside the bag rather than in it ([`Bag`] is said over
    /// the wire as it was before there were colours, to apps from then).
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
    pub struct Paints(pub Vec<Paint>);

    impl Paints {
        /// The colour `part` of the thing standing under `id` is painted,
        /// if it is.
        pub fn colour(&self, id: u32, part: &str) -> Option<[u8; 3]> {
            self.0.iter().find(|paint| paint.id == id && paint.part == part).map(|paint| paint.colour)
        }

        /// What is painted of the thing standing under `id`: each part and
        /// its colour.
        pub fn of(&self, id: u32) -> impl Iterator<Item = (&str, [u8; 3])> {
            self.0.iter().filter(move |paint| paint.id == id).map(|paint| (paint.part.as_str(), paint.colour))
        }

        /// Paints `part` of the thing standing under `id` among `stands`
        /// this colour, over whatever colour it was.
        pub fn set(&mut self, stands: &[Standing], id: u32, part: &str, colour: [u8; 3]) -> Result<(), No> {
            if !part_named(part) {
                return Err(No::Unknown);
            }
            if stands.iter().all(|stood| stood.id != id) {
                return Err(No::None);
            }
            if let Some(paint) = self.0.iter_mut().find(|paint| paint.id == id && paint.part == part) {
                paint.colour = colour;
                return Ok(());
            }
            if self.0.len() >= MOST_PAINTED {
                return Err(No::Full);
            }
            self.0.push(Paint {
                id,
                part: part.to_string(),
                colour,
            });
            Ok(())
        }

        /// Takes the paint off `part` of the thing standing under `id`: it
        /// is as its model has it again. Whether it was painted.
        pub fn clear(&mut self, id: u32, part: &str) -> bool {
            let before = self.0.len();
            self.0.retain(|paint| !(paint.id == id && paint.part == part));
            self.0.len() != before
        }

        /// Lets go of the colours of whatever no longer stands among
        /// `stands`. Whether any went.
        pub fn fit(&mut self, stands: &[Standing]) -> bool {
            let before = self.0.len();
            self.0.retain(|paint| stands.iter().any(|stood| stood.id == paint.id));
            self.0.len() != before
        }

        /// The colours as they may be, with whatever was kept wrongly put
        /// right: only of what stands among `stands`, one to a part, each
        /// under a name a part may go by, and no more than a plot has.
        pub fn sane(self, stands: &[Standing]) -> Self {
            let mut sane = Paints::default();
            for paint in self.0 {
                if sane.colour(paint.id, &paint.part).is_none() {
                    let _ = sane.set(stands, paint.id, &paint.part, paint.colour);
                }
            }
            sane
        }
    }

    // ------------------------------------------- what works (2026-11-27)

    /// What something that keeps things in it keeps, as a chest does (since
    /// 2026-11-27; Hajun: "cabinets can have storage inventory that is like a
    /// chest - u can put items in there, any anyone can open it and take it
    /// out or put sth there"): how many slots it has, each holding so many
    /// of one kind of thing, and whether only food goes in it.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Keeps {
        pub slots: u8,
        pub food_only: bool,
    }

    /// What a thing of `kind` keeps, if it keeps anything: as Hajun has it,
    /// "cabinet = 8 square slots (4 on left, 4 on right) barbeque = 3 square
    /// slots but food only (1 row of 3) end table = 3 square slots (1 row of
    /// 3) safe = 9 square slots (3 rows of 3, like a 3x3 square) dresser = 12
    /// square slots (6 on left, 6 on right)". How the slots are laid out is
    /// the game's to draw.
    pub fn keeps(kind: &str) -> Option<Keeps> {
        let (slots, food_only) = match kind {
            "cabinet" => (8, false),
            "barbecue" => (3, true),
            "end_table" => (3, false),
            "safe" => (9, false),
            "dresser" => (12, false),
            _ => return None,
        };
        Some(Keeps { slots, food_only })
    }

    /// What there is to own that is food, for the barbecue: what is eaten
    /// and what is drunk (mine, 2026-11-27: Hajun's list has no food of its
    /// own, and these four are the things on it that are).
    pub const FOOD: [&str; 4] = ["hamburger", "fruit_bowl", "cup_of_tea", "wine_glass"];

    /// Whether the thing that goes by `kind` is food ([`FOOD`]).
    pub fn is_food(kind: &str) -> bool {
        FOOD.contains(&kind)
    }

    /// The lights that are switched on and off by hand, and not by the
    /// time of day (since 2026-11-27; Hajun: "lamp with shade (toggle light
    /// on off rather than day/night) chandelier (toggle light on off rather
    /// than day/night)"): off until someone switches one on.
    pub const SWITCHED: [&str; 2] = ["lamp_with_shade", "chandelier"];

    /// Whether a thing of `kind` is a light switched by hand ([`SWITCHED`]).
    pub fn switched(kind: &str) -> bool {
        SWITCHED.contains(&kind)
    }

    /// The sink, whose tap is run (since 2026-11-27; Hajun: "turn faucet on
    /// for 5 second, water comes down the faucet, and water in the sink goes
    /// up and up until its full, and then the faucet turns off
    /// automatically"); and how long it is from its tap being run until it
    /// is empty again and can be run once more, in seconds: five filling,
    /// the rest full and draining away (the game's `working` draws it so).
    pub const SINK: &str = "bathroom_sink";
    pub const SINK_SECS: f64 = 12.0;
    /// The bathtub, whose tap is run as the sink's is (later on 2026-11-27,
    /// where it had been lain in; Hajun: "for the bathtub, instead of being
    /// a lie down proximity button, make it similar to the sink -> it poors
    /// water until the tub is full, and it stops pooring when its full, and
    /// then it drains after a while"), and how long from its tap being run
    /// until it can be run again: eight filling, eight full, four draining
    /// (mine).
    pub const BATH: &str = "bathtub";
    pub const BATH_SECS: f64 = 20.0;

    /// How long from the tap of a thing of `kind` being run until it can be
    /// run again, if it has a tap: the sink's and the bathtub's.
    pub fn runs(kind: &str) -> Option<f64> {
        match kind {
            SINK => Some(SINK_SECS),
            BATH => Some(BATH_SECS),
            _ => None,
        }
    }

    /// The piggy bank, which keeps coins (since later on 2026-11-27; Hajun:
    /// "for the piggybank, make a proximity thing similar to the
    /// cabinet/storage, but this time instead of items its coin. so u
    /// choose how much coin to put in the piggy bank and anyone can take it
    /// out or put more"), and the most one keeps: a billion, so that taken
    /// out with a balance besides, it never runs past what a balance holds.
    pub const PIGGY: &str = "piggy_bank";
    pub const PIGGY_MOST: u32 = 1_000_000_000;

    /// The coins in each piggy bank standing on one plot, by its number
    /// ([`Standing::id`]): none, for one that has never had any. Kept beside
    /// what works there, and by the server in a table of its own
    /// (`piggy_banks`), each change one transaction with the balance of
    /// whoever made it. Anyone in the channel may put coins in and take
    /// them out, as with what is kept in a cabinet.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
    pub struct Banked(pub Vec<(u32, u32)>);

    impl Banked {
        /// The coins in the piggy bank standing under `id`.
        pub fn of(&self, id: u32) -> u32 {
            self.0.iter().find(|(bank, _)| *bank == id).map_or(0, |(_, coins)| *coins)
        }

        /// The piggy bank under `id` holds `coins` now: with none, nothing
        /// is kept of it.
        pub fn set(&mut self, id: u32, coins: u32) {
            self.0.retain(|(bank, _)| *bank != id);
            if coins > 0 {
                self.0.push((id, coins));
                self.0.sort_unstable();
            }
        }

        /// Whether no piggy bank of these has any coins in it.
        pub fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// The coins as they may be: only in piggy banks among `stands`, one
        /// entry each, none empty, none over the most.
        pub fn sane(self, stands: &[Standing]) -> Self {
            let mut sane = Banked::default();
            for (id, coins) in self.0 {
                let banks = stands.iter().any(|stood| stood.id == id && stood.kind == PIGGY);
                if banks && sane.of(id) == 0 {
                    sane.set(id, coins.min(PIGGY_MOST));
                }
            }
            sane
        }
    }

    /// Whether the thing standing under `id` among `stands` is a piggy bank.
    fn piggy(stands: &[Standing], id: u32) -> Result<(), No> {
        let stood = stands.iter().find(|stood| stood.id == id).ok_or(No::None)?;
        if stood.kind == PIGGY { Ok(()) } else { Err(No::Place) }
    }

    /// Puts `coins` of `balance` into the piggy bank standing under `id`
    /// among `stands`, whose plot's coins are `banked`: what is left of the
    /// balance. Not more than there is of it, nor so much that the piggy
    /// bank would hold more than the most.
    pub fn deposit(balance: u32, banked: &mut Banked, stands: &[Standing], id: u32, coins: u32) -> Result<u32, No> {
        piggy(stands, id)?;
        if coins == 0 {
            return Err(No::None);
        }
        let left = balance.checked_sub(coins).ok_or(No::Coins)?;
        let now = banked.of(id).checked_add(coins).filter(|&now| now <= PIGGY_MOST).ok_or(No::Full)?;
        banked.set(id, now);
        Ok(left)
    }

    /// Takes `coins` out of the piggy bank standing under `id` into
    /// `balance`: what the balance is then. Not more than it holds.
    pub fn withdraw(balance: u32, banked: &mut Banked, stands: &[Standing], id: u32, coins: u32) -> Result<u32, No> {
        piggy(stands, id)?;
        let now = banked.of(id).checked_sub(coins).filter(|_| coins > 0).ok_or(No::None)?;
        let after = balance.checked_add(coins).ok_or(No::Full)?;
        banked.set(id, now);
        Ok(after)
    }

    /// So many of one kind of thing, in one slot of something that keeps
    /// things.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct Stack {
        pub kind: String,
        pub count: u16,
    }

    /// What is kept in one thing standing on a plot: the thing, by its
    /// number ([`Standing::id`]), and each of its slots in order, empty or
    /// with a [`Stack`] in it.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
    pub struct Held {
        pub id: u32,
        pub slots: Vec<Option<Stack>>,
    }

    /// What is at work among what stands on one plot: what is kept in each
    /// thing that keeps things ([`keeps`]), and which of its lights are
    /// switched on ([`SWITCHED`]), each by its number. **Anyone in the
    /// channel may put things in, take them out and switch the lights**, as
    /// Hajun has it of the first; what is kept belongs to nobody while it is
    /// kept, and comes into the bag of whoever takes it. Kept beside the bag
    /// and not in it ([`Bag`] and [`Paints`] are said over the wire as they
    /// were before, to apps from then), and by the server in a column of its
    /// own, which it writes for an owner who is not here.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
    pub struct Workings {
        pub held: Vec<Held>,
        pub lit: Vec<u32>,
    }

    impl Workings {
        /// The slots of the thing standing under `id`: none, for one that
        /// has never had anything in it.
        pub fn slots(&self, id: u32) -> &[Option<Stack>] {
            self.held.iter().find(|held| held.id == id).map_or(&[], |held| held.slots.as_slice())
        }

        /// Whether anything is kept in the thing standing under `id`.
        pub fn holds_anything(&self, id: u32) -> bool {
            self.slots(id).iter().any(Option::is_some)
        }

        /// Whether the light standing under `id` is switched on.
        pub fn is_lit(&self, id: u32) -> bool {
            self.lit.contains(&id)
        }

        /// Whether nothing is kept anywhere and no light is on.
        pub fn is_empty(&self) -> bool {
            self.lit.is_empty() && self.held.iter().all(|held| held.slots.iter().all(Option::is_none))
        }

        /// The slots of the thing standing under `id`, `slots` of them, as
        /// many empty ones made as it has never had.
        fn slots_mut(&mut self, id: u32, slots: usize) -> &mut Vec<Option<Stack>> {
            let at = match self.held.iter().position(|held| held.id == id) {
                Some(at) => at,
                None => {
                    self.held.push(Held { id, slots: Vec::new() });
                    self.held.len() - 1
                }
            };
            let held = &mut self.held[at].slots;
            if held.len() < slots {
                held.resize(slots, None);
            }
            held
        }

        /// Switches the light standing under `id` among `stands` on or off.
        /// Whether that changed it.
        pub fn switch(&mut self, stands: &[Standing], id: u32, on: bool) -> Result<bool, No> {
            let stood = stands.iter().find(|stood| stood.id == id).ok_or(No::None)?;
            if !switched(&stood.kind) {
                return Err(No::Place);
            }
            if on == self.is_lit(id) {
                return Ok(false);
            }
            if on {
                self.lit.push(id);
                self.lit.sort_unstable();
            } else {
                self.lit.retain(|&lit| lit != id);
            }
            Ok(true)
        }

        /// Lets go of whatever of the thing standing under `id` there is
        /// here, it being taken back into the bag: whether its light was
        /// on, and what was kept in it, which goes with it.
        pub fn let_go(&mut self, id: u32) -> Vec<Stack> {
            self.lit.retain(|&lit| lit != id);
            let Some(at) = self.held.iter().position(|held| held.id == id) else {
                return Vec::new();
            };
            self.held.remove(at).slots.into_iter().flatten().collect()
        }

        /// What is at work as it may be, with whatever was kept wrongly put
        /// right: only of what stands among `stands` and keeps things, or
        /// is a light switched by hand; each thing once, with its own number
        /// of slots, what is in a slot past them moved into an empty one if
        /// there is one; and in each slot so many of a thing there is, as
        /// many as one may own at the most, food where only food goes.
        pub fn sane(self, stands: &[Standing]) -> Self {
            let mut sane = Workings::default();
            for held in self.held {
                let Some(keeps) = stands.iter().find(|stood| stood.id == held.id).and_then(|stood| keeps(&stood.kind)) else {
                    continue;
                };
                if sane.held.iter().any(|other| other.id == held.id) {
                    continue;
                }
                let fits = |stack: &Stack| {
                    thing(&stack.kind).is_some() && stack.count > 0 && (!keeps.food_only || is_food(&stack.kind))
                };
                let mut slots: Vec<Option<Stack>> = vec![None; usize::from(keeps.slots)];
                let mut over = Vec::new();
                for (at, stack) in held.slots.into_iter().enumerate() {
                    let Some(mut stack) = stack.filter(fits) else {
                        continue;
                    };
                    stack.count = stack.count.min(MOST_OF_A_KIND);
                    match slots.get_mut(at) {
                        Some(slot) => *slot = Some(stack),
                        None => over.push(stack),
                    }
                }
                for stack in over {
                    if let Some(empty) = slots.iter_mut().find(|slot| slot.is_none()) {
                        *empty = Some(stack);
                    }
                }
                if slots.iter().any(Option::is_some) {
                    sane.held.push(Held { id: held.id, slots });
                }
            }
            sane.lit = self
                .lit
                .into_iter()
                .filter(|&id| stands.iter().any(|stood| stood.id == id && switched(&stood.kind)))
                .collect();
            sane.lit.sort_unstable();
            sane.lit.dedup();
            sane
        }
    }

    /// What the thing standing under `id` among `stands` keeps, if it is
    /// there and keeps anything.
    fn keeping(stands: &[Standing], id: u32) -> Result<Keeps, No> {
        let stood = stands.iter().find(|stood| stood.id == id).ok_or(No::None)?;
        keeps(&stood.kind).ok_or(No::Place)
    }

    /// Puts `count` of `kind` out of `bag` into slot `slot` of the thing
    /// standing under `id` among `stands`, whose plot's workings are
    /// `workings`: into an empty slot, or onto as many of the same there
    /// already, up to as many as one may own; and food alone into what only
    /// food goes in. The bag has to have them in it, not standing. Anyone's
    /// bag, into anyone's thing (Hajun: "anyone can open it and take it out
    /// or put sth there").
    pub fn stow(bag: &mut Bag, workings: &mut Workings, stands: &[Standing], id: u32, slot: usize, kind: &str, count: u16) -> Result<(), No> {
        thing(kind).ok_or(No::Unknown)?;
        let keeps = keeping(stands, id)?;
        if slot >= usize::from(keeps.slots) {
            return Err(No::Place);
        }
        if keeps.food_only && !is_food(kind) {
            return Err(No::Food);
        }
        if count == 0 || bag.free(kind) < count {
            return Err(No::None);
        }
        let there = workings.slots(id).get(slot).cloned().flatten();
        let now = match there {
            None => count,
            Some(stack) if stack.kind == kind => {
                let now = u32::from(stack.count) + u32::from(count);
                if now > u32::from(MOST_OF_A_KIND) {
                    return Err(No::Full);
                }
                now as u16
            }
            Some(_) => return Err(No::Taken),
        };
        bag.give_up(kind, count)?;
        workings.slots_mut(id, usize::from(keeps.slots))[slot] = Some(Stack {
            kind: kind.to_string(),
            count: now,
        });
        Ok(())
    }

    /// Takes what is in slot `slot` of the thing standing under `id` among
    /// `stands` into `bag`, as much of it as the bag has room for, which may
    /// be all of it: what came into the bag. Anyone may.
    pub fn unstow(bag: &mut Bag, workings: &mut Workings, stands: &[Standing], id: u32, slot: usize) -> Result<Stack, No> {
        let keeps = keeping(stands, id)?;
        let Some(stack) = workings.slots(id).get(slot).cloned().flatten() else {
            return Err(No::None);
        };
        let came = stack.count.min(bag.room_for(&stack.kind));
        if came == 0 {
            return Err(No::Full);
        }
        bag.take_in(&stack.kind, came)?;
        let left = stack.count - came;
        workings.slots_mut(id, usize::from(keeps.slots))[slot] = (left > 0).then(|| Stack {
            kind: stack.kind.clone(),
            count: left,
        });
        if !workings.holds_anything(id) {
            workings.held.retain(|held| held.id != id);
        }
        Ok(Stack {
            kind: stack.kind,
            count: came,
        })
    }

    /// Takes the thing standing under `id` back off the plot into `bag`
    /// ([`Bag::take`]), and whatever is kept in it with it, if the bag has
    /// room for all of that; its light, if it is one, goes out. Refused
    /// ([`No::Full`]) with too much of a kind in it to fit, and nothing is
    /// changed: what was kept in it is never lost. A piggy bank with coins
    /// in it, by `banked`, is refused too ([`No::Banked`]), until they are
    /// taken out (mine: rather than its coins coming into the balance of
    /// whoever took it back, which would be a transaction of its own).
    pub fn take_back(bag: &mut Bag, workings: &mut Workings, banked: &Banked, id: u32) -> Result<(), No> {
        if banked.of(id) > 0 {
            return Err(No::Banked);
        }
        let mut kept: Vec<(&str, u32)> = Vec::new();
        for stack in workings.slots(id).iter().flatten() {
            match kept.iter_mut().find(|(kind, _)| *kind == stack.kind) {
                Some((_, count)) => *count += u32::from(stack.count),
                None => kept.push((stack.kind.as_str(), u32::from(stack.count))),
            }
        }
        if kept.iter().any(|&(kind, count)| thing(kind).is_some() && u32::from(bag.room_for(kind)) < count) {
            return Err(No::Full);
        }
        bag.take(id)?;
        for stack in workings.let_go(id) {
            // A kind there is no longer is let go of, as the bag would.
            let _ = bag.take_in(&stack.kind, stack.count);
        }
        Ok(())
    }

    /// One person's bag: their gems, how many of each kind of thing they
    /// own, those standing on their plot among them, and what stands there.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    pub struct Bag {
        pub gems: u32,
        pub owned: Vec<(String, u16)>,
        pub stands: Vec<Standing>,
    }

    impl Default for Bag {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Bag {
        /// What everyone begins with: ten gems, one car and one street
        /// lamp, and nothing on their plot.
        pub fn new() -> Self {
            Self {
                gems: GEMS,
                owned: vec![(LAMP.to_string(), 1), (CAR.to_string(), 1)],
                stands: Vec::new(),
            }
        }

        /// How many of `kind` are owned, those standing among them.
        pub fn count(&self, kind: &str) -> u16 {
            self.owned.iter().find(|(owned, _)| owned == kind).map_or(0, |(_, count)| *count)
        }

        /// How many of `kind` stand on the plot.
        pub fn standing(&self, kind: &str) -> u16 {
            self.stands.iter().filter(|stood| stood.kind == kind).count().min(usize::from(u16::MAX)) as u16
        }

        /// How many of `kind` are in the bag: owned, and not standing on
        /// the plot. What the inventory shows, and what may be sold or
        /// stood there.
        pub fn free(&self, kind: &str) -> u16 {
            self.count(kind).saturating_sub(self.standing(kind))
        }

        /// Whether a car is owned, any of them, to drive.
        pub fn has_car(&self) -> bool {
            self.cars().next().is_some()
        }

        /// The cars owned, in the order of the things there are.
        pub fn cars(&self) -> impl Iterator<Item = &'static str> + '_ {
            THINGS
                .iter()
                .filter(|thing| thing.group == Group::Vehicles && self.count(thing.kind) > 0)
                .map(|thing| thing.kind)
        }

        fn set(&mut self, kind: &str, count: u16) {
            self.owned.retain(|(owned, _)| owned != kind);
            if count > 0 {
                self.owned.push((kind.to_string(), count));
                // In the order of the things there are, whatever was
                // bought when.
                let place = |kind: &str| THINGS.iter().position(|thing| thing.kind == kind).unwrap_or(usize::MAX);
                self.owned.sort_by_key(|(kind, _)| place(kind));
            }
        }

        /// Buys one of `kind` with `coins` in hand: what is left of them.
        /// One sold for gems is paid for out of the bag's own gems, and
        /// leaves the coins as they were.
        pub fn buy(&mut self, kind: &str, coins: u32) -> Result<u32, No> {
            let thing = thing(kind).ok_or(No::Unknown)?;
            let count = self.count(kind);
            if count >= MOST_OF_A_KIND {
                return Err(No::Full);
            }
            let left = match thing.money {
                Money::Coins => coins.checked_sub(thing.price).ok_or(No::Coins)?,
                Money::Gems => {
                    self.gems = self.gems.checked_sub(thing.price).ok_or(No::Gems)?;
                    coins
                }
            };
            self.set(kind, count + 1);
            Ok(left)
        }

        /// Sells one of `kind` out of the bag, with `coins` in hand: what
        /// there is of them after. One that stands on the plot is not in
        /// the bag to be sold. One sold for gems fetches gems, half its
        /// price of them, into the bag (mine, 2026-11-25: what it was
        /// bought with).
        pub fn sell(&mut self, kind: &str, coins: u32) -> Result<u32, No> {
            let thing = thing(kind).ok_or(No::Unknown)?;
            if self.free(kind) == 0 {
                return Err(No::None);
            }
            self.set(kind, self.count(kind) - 1);
            match thing.money {
                Money::Coins => Ok(coins.saturating_add(thing.resale())),
                Money::Gems => {
                    self.gems = self.gems.saturating_add(thing.resale());
                    Ok(coins)
                }
            }
        }

        /// Stands one of `kind` on the plot at `at`, under the number `id`;
        /// or, if what stands under that number is one of that kind, moves
        /// it there.
        pub fn put(&mut self, id: u32, kind: &str, at: At) -> Result<(), No> {
            let thing = thing(kind).ok_or(No::Unknown)?;
            if !thing.stands() || !sound(&at) {
                return Err(No::Place);
            }
            if let Some(stood) = self.stands.iter_mut().find(|stood| stood.id == id) {
                if stood.kind != kind {
                    return Err(No::Place);
                }
                stood.at = at;
                return Ok(());
            }
            if self.free(kind) == 0 {
                return Err(No::None);
            }
            if self.stands.len() >= MOST_STANDING {
                return Err(No::Full);
            }
            self.stands.push(Standing {
                id,
                kind: kind.to_string(),
                at,
            });
            Ok(())
        }

        /// Takes what stands under the number `id` back into the bag.
        pub fn take(&mut self, id: u32) -> Result<(), No> {
            let before = self.stands.len();
            self.stands.retain(|stood| stood.id != id);
            if self.stands.len() == before { Err(No::None) } else { Ok(()) }
        }

        /// Gives up `count` of `kind` out of the bag, put away in something
        /// that keeps things ([`stow`]): it has to have that many in it, not
        /// standing.
        pub fn give_up(&mut self, kind: &str, count: u16) -> Result<(), No> {
            if self.free(kind) < count {
                return Err(No::None);
            }
            self.set(kind, self.count(kind) - count);
            Ok(())
        }

        /// How many more of `kind` the bag has room for: as many as one may
        /// own at the most, less those owned; none of a kind there is not.
        pub fn room_for(&self, kind: &str) -> u16 {
            if thing(kind).is_none() {
                return 0;
            }
            MOST_OF_A_KIND.saturating_sub(self.count(kind))
        }

        /// Takes `count` of `kind` into the bag, taken out of something that
        /// keeps things ([`unstow`]), if it has room for them all.
        pub fn take_in(&mut self, kind: &str, count: u16) -> Result<(), No> {
            thing(kind).ok_or(No::Unknown)?;
            if self.room_for(kind) < count {
                return Err(No::Full);
            }
            self.set(kind, self.count(kind) + count);
            Ok(())
        }

        /// A number for the next thing to stand on the plot: one more than
        /// the highest there.
        pub fn next_id(&self) -> u32 {
            self.stands.iter().map(|stood| stood.id).max().map_or(1, |most| most.wrapping_add(1).max(1))
        }

        /// The bag as it may be: as it was kept, with whatever was kept
        /// wrongly put right, so that nothing read out of a file or a table
        /// breaks the rules above. Kinds there are no longer are let go of,
        /// with whatever of them stood; no more of a kind than may be owned;
        /// nothing standing that is not owned, that stands nowhere, or
        /// twice under one number.
        pub fn sane(mut self) -> Self {
            let mut owned: Vec<(String, u16)> = Vec::new();
            for thing in THINGS {
                let count: u32 = self.owned.iter().filter(|(kind, _)| kind == thing.kind).map(|(_, count)| u32::from(*count)).sum();
                if count > 0 {
                    owned.push((thing.kind.to_string(), count.min(u32::from(MOST_OF_A_KIND)) as u16));
                }
            }
            self.owned = owned;
            let stood = std::mem::take(&mut self.stands);
            for standing in stood {
                if self.stands.iter().all(|other| other.id != standing.id) {
                    let _ = self.put(standing.id, &standing.kind, standing.at);
                }
            }
            self
        }
    }

    /// Something asked about your bag.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    pub enum Ask {
        /// Tell me of my bag, and of what stands on the plots of my channel
        /// of the town, now and as either changes. Said once, on coming
        /// online, by an app that knows how to read the answers.
        Watch,
        /// Sell me one of this, for its price out of my coins.
        Buy(String),
        /// Buy one of this back off me, out of my bag, for half its price.
        Sell(String),
        /// Stand this thing of mine on my plot, here: `id` is mine to
        /// choose, and says the same thing again if it is moved.
        Put { id: u32, kind: String, at: At },
        /// Take this back off my plot, into my bag.
        Take { id: u32 },
        /// Tell me of the colours things are painted too, on my plot and on
        /// the plots of my channel ([`Tell::Painted`]). Said once, on
        /// coming online, after [`Ask::Watch`], by an app that knows how to
        /// read them (since 2026-11-22: one from before never says it, and
        /// is never told).
        Colours,
        /// Paint this part of this thing of mine, which stands on my plot,
        /// this colour ([`Paint`]).
        Paint { id: u32, part: String, colour: [u8; 3] },
        /// Take the paint off this part of this thing of mine: as its model
        /// has it again (since later on 2026-11-22; Hajun: "if u paint sth
        /// with the clear, it will clear that color to its original").
        Unpaint { id: u32, part: String },
        /// Tell me what is at work on my plot and on the plots of my channel
        /// ([`Tell::Workings`]): what is kept in what keeps things, and which
        /// lights are on. Said once, on coming online, after [`Ask::Colours`],
        /// by an app that knows how to show it (since 2026-11-27: one from
        /// before never says it, and is never told).
        Workings,
        /// Put `count` of `kind` out of my bag into slot `slot` of the thing
        /// standing under `id` on `who`'s plot, which keeps things
        /// ([`stow`]): mine, or anyone's in my channel.
        Stow { who: u64, id: u32, slot: u8, kind: String, count: u16 },
        /// Take what is in slot `slot` of that thing into my bag
        /// ([`unstow`]).
        Unstow { who: u64, id: u32, slot: u8 },
        /// Switch the light standing under `id` on `who`'s plot on or off
        /// ([`Workings::switch`]).
        Switch { who: u64, id: u32, on: bool },
        /// Run the tap of the sink standing under `id` on `who`'s plot
        /// ([`SINK`]): everyone in the channel is to see it fill
        /// ([`Tell::Ran`]).
        Run { who: u64, id: u32 },
        /// Tell me what is in the piggy banks on my plot and on the plots of
        /// my channel ([`Tell::Banked`]). Said once, on coming online, after
        /// [`Ask::Workings`], by an app that knows how to show it (since
        /// later on 2026-11-27).
        Banked,
        /// Put this many of my coins into the piggy bank standing under `id`
        /// on `who`'s plot ([`deposit`]).
        Deposit { who: u64, id: u32, coins: u32 },
        /// Take this many coins out of it, into my balance ([`withdraw`]).
        Withdraw { who: u64, id: u32, coins: u32 },
    }

    /// Something told about a bag.
    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    pub enum Tell {
        /// Your bag, as it stands: said as you sit down in the town, having
        /// asked ([`Ask::Watch`]), and once in answer to everything you ask
        /// of it after, done or not. `asked` is how many of this
        /// connection's asks about the bag the server has answered by then,
        /// that one among them: with more of your own on their way, a newer
        /// answer is coming.
        Yours { asked: u32, bag: Bag },
        /// What stands on the plot of someone else in your channel of the
        /// town, by their [`Member::id`](super::Member::id): all of it, as
        /// you come into the channel or they do, and whenever it changes.
        /// Which plot is theirs goes by their seat
        /// ([`Tell::Channel`](super::Tell::Channel)).
        Theirs { who: u64, stands: Vec<Standing> },
        /// The colours on what stands on `who`'s plot, all of them
        /// ([`Paints`]). Only to an app that asked ([`Ask::Colours`]). Of
        /// your own plot, where `who` is you, it comes after each
        /// [`Tell::Yours`] and `asked` is that one's; of the plot of anyone
        /// else in your channel, after what stands there
        /// ([`Tell::Theirs`]) and whenever they paint, and `asked` is
        /// nothing.
        Painted { who: u64, asked: u32, paints: Vec<Paint> },
        /// What is at work on `who`'s plot, you or anyone else in your
        /// channel, all of it ([`Workings`]): after what stands there and its
        /// colours, as either is told, and whenever anyone in the channel
        /// puts something into one of its things, takes something out or
        /// switches a light. The server's word, whoever's plot it is. Only to
        /// an app that asked ([`Ask::Workings`]).
        Workings { who: u64, workings: Workings },
        /// So many of a kind of thing have come into your bag, taken out of
        /// something that keeps things ([`Ask::Unstow`]): for the game to say
        /// so. After the [`Tell::Yours`] that has them in it.
        Got(Stack),
        /// The tap of the sink standing under `id` on `who`'s plot has been
        /// run, by you or anyone else in your channel ([`Ask::Run`]): it
        /// fills from now.
        Ran { who: u64, id: u32 },
        /// The coins in the piggy banks on `who`'s plot, you or anyone else
        /// in your channel, all of them ([`Banked`]): after what works
        /// there, as that is told, and whenever anyone puts coins in or takes
        /// them out. Only to an app that asked ([`Ask::Banked`]).
        Banked { who: u64, banked: Banked },
        /// So many coins have come into your balance out of a piggy bank, or,
        /// less than none, gone into one ([`Ask::Deposit`],
        /// [`Ask::Withdraw`]): for the game to say so. After the balance.
        Coins(i64),
    }

    #[cfg(test)]
    mod working_tests {
        use super::*;

        fn ground(x: f32) -> At {
            At::Ground { x, y: 0.0, z: 0.0, yaw: 0.0 }
        }

        /// A bag with these things in it, these of them standing under the
        /// numbers 1, 2, 3 and so on.
        fn bag_with(owned: &[(&str, u16)], standing: &[&str]) -> Bag {
            let mut bag = Bag { gems: 0, owned: Vec::new(), stands: Vec::new() };
            for &(kind, count) in owned {
                bag.take_in(kind, count).unwrap();
            }
            for (n, kind) in standing.iter().enumerate() {
                bag.put(n as u32 + 1, kind, ground(n as f32)).unwrap();
            }
            bag
        }

        #[test]
        fn what_keeps_things_keeps_as_many_as_hajun_said_and_the_barbecue_food_alone() {
            let slots = |kind: &str| keeps(kind).map(|keeps| (keeps.slots, keeps.food_only));
            assert_eq!(slots("cabinet"), Some((8, false)));
            assert_eq!(slots("barbecue"), Some((3, true)));
            assert_eq!(slots("end_table"), Some((3, false)));
            assert_eq!(slots("safe"), Some((9, false)));
            assert_eq!(slots("dresser"), Some((12, false)));
            assert_eq!(slots("desk"), None);
            // Every one of them, the food, the lights and the sink are things
            // there are to own.
            for kind in ["cabinet", "barbecue", "end_table", "safe", "dresser", SINK].into_iter().chain(FOOD).chain(SWITCHED) {
                assert!(thing(kind).is_some(), "{kind}");
            }
            assert!(is_food("hamburger") && !is_food("streetlamp"));
            assert!(switched("chandelier") && switched("lamp_with_shade") && !switched(LAMP));
        }

        #[test]
        fn anyone_puts_things_in_and_takes_them_out_and_nothing_is_made_or_lost() {
            let owner = bag_with(&[("cabinet", 1), ("barbecue", 1)], &["cabinet", "barbecue"]);
            let stands = owner.stands.clone();
            let mut works = Workings::default();
            let mut guest = bag_with(&[("hamburger", 5), ("doll", 2)], &[]);
            // Three hamburgers into the cabinet's second slot: out of the bag,
            // into the slot.
            stow(&mut guest, &mut works, &stands, 1, 1, "hamburger", 3).unwrap();
            assert_eq!((guest.count("hamburger"), works.slots(1).len()), (2, 8));
            assert_eq!(works.slots(1)[1], Some(Stack { kind: "hamburger".into(), count: 3 }));
            // Onto them, two more; but not a doll, nor more than the bag has.
            stow(&mut guest, &mut works, &stands, 1, 1, "hamburger", 2).unwrap();
            assert_eq!(works.slots(1)[1].as_ref().map(|stack| stack.count), Some(5));
            assert_eq!(stow(&mut guest, &mut works, &stands, 1, 1, "doll", 1), Err(No::Taken));
            assert_eq!(stow(&mut guest, &mut works, &stands, 1, 0, "doll", 3), Err(No::None));
            assert_eq!(stow(&mut guest, &mut works, &stands, 1, 0, "doll", 0), Err(No::None));
            // Nowhere past its slots, in nothing that keeps things, and in
            // nothing standing.
            assert_eq!(stow(&mut guest, &mut works, &stands, 1, 8, "doll", 1), Err(No::Place));
            assert_eq!(stow(&mut guest, &mut works, &stands, 7, 0, "doll", 1), Err(No::None));
            // The barbecue takes food alone.
            assert_eq!(stow(&mut guest, &mut works, &stands, 2, 0, "doll", 1), Err(No::Food));
            assert_eq!(guest.count("doll"), 2);
            // Someone else takes the five out: into their bag, the slot empty
            // and the cabinet with nothing in it.
            let mut taker = bag_with(&[], &[]);
            assert_eq!(unstow(&mut taker, &mut works, &stands, 1, 1), Ok(Stack { kind: "hamburger".into(), count: 5 }));
            assert_eq!(taker.count("hamburger"), 5);
            assert!(!works.holds_anything(1) && works.is_empty());
            assert_eq!(unstow(&mut taker, &mut works, &stands, 1, 1), Err(No::None));
            // With room for some of it only, they take what fits and leave
            // the rest.
            let mut full = bag_with(&[("doll", MOST_OF_A_KIND - 1)], &[]);
            stow(&mut guest, &mut works, &stands, 1, 4, "doll", 2).unwrap();
            assert_eq!(unstow(&mut full, &mut works, &stands, 1, 4), Ok(Stack { kind: "doll".into(), count: 1 }));
            assert_eq!(works.slots(1)[4], Some(Stack { kind: "doll".into(), count: 1 }));
            assert_eq!(unstow(&mut full, &mut works, &stands, 1, 4), Err(No::Full));
        }

        #[test]
        fn a_thing_taken_back_takes_what_is_kept_in_it_and_is_refused_where_that_would_not_fit() {
            let mut owner = bag_with(&[("dresser", 1), ("chandelier", 1), ("cup_of_tea", 3)], &["dresser", "chandelier"]);
            let stands = owner.stands.clone();
            let mut works = Workings::default();
            stow(&mut owner, &mut works, &stands, 1, 11, "cup_of_tea", 3).unwrap();
            assert!(works.switch(&stands, 2, true).unwrap());
            assert_eq!(owner.count("cup_of_tea"), 0);
            take_back(&mut owner, &mut works, &Banked::default(), 1).unwrap();
            assert_eq!((owner.count("cup_of_tea"), owner.free("dresser")), (3, 1));
            assert!(works.held.is_empty() && works.is_lit(2));
            take_back(&mut owner, &mut works, &Banked::default(), 2).unwrap();
            assert!(works.is_empty());
            // A safe with four cups in it and a bag with as many cups as
            // may be owned: refused, and nothing changed.
            let mut owner = bag_with(&[("safe", 1), ("cup_of_tea", MOST_OF_A_KIND)], &["safe"]);
            let stands = owner.stands.clone();
            let mut works = Workings::default();
            stow(&mut owner, &mut works, &stands, 1, 0, "cup_of_tea", 4).unwrap();
            owner.take_in("cup_of_tea", 4).unwrap();
            let (before, kept) = (owner.clone(), works.clone());
            assert_eq!(take_back(&mut owner, &mut works, &Banked::default(), 1), Err(No::Full));
            assert_eq!((owner, works), (before, kept));
            // A piggy bank with coins in it: refused until they are out.
            let mut owner = bag_with(&[(PIGGY, 1)], &[PIGGY]);
            let mut banked = Banked::default();
            deposit(10, &mut banked, &owner.stands.clone(), 1, 4).unwrap();
            let mut works = Workings::default();
            assert_eq!(take_back(&mut owner, &mut works, &banked, 1), Err(No::Banked));
            assert_eq!(owner.stands.len(), 1);
            withdraw(6, &mut banked, &owner.stands.clone(), 1, 4).unwrap();
            take_back(&mut owner, &mut works, &banked, 1).unwrap();
            assert_eq!((owner.stands.len(), owner.free(PIGGY)), (0, 1));
        }

        #[test]
        fn a_light_is_switched_by_hand_and_only_a_light_that_is() {
            let bag = bag_with(&[("lamp_with_shade", 1), ("chandelier", 1), (LAMP, 1)], &["lamp_with_shade", "chandelier", LAMP]);
            let mut works = Workings::default();
            assert_eq!(works.switch(&bag.stands, 1, true), Ok(true));
            assert_eq!(works.switch(&bag.stands, 1, true), Ok(false));
            assert_eq!(works.switch(&bag.stands, 2, true), Ok(true));
            assert_eq!(works.lit, [1, 2]);
            assert_eq!(works.switch(&bag.stands, 1, false), Ok(true));
            assert!(!works.is_lit(1) && works.is_lit(2));
            // The street lamp goes by the time of day, and nothing stands
            // under 9.
            assert_eq!(works.switch(&bag.stands, 3, true), Err(No::Place));
            assert_eq!(works.switch(&bag.stands, 9, true), Err(No::None));
        }

        #[test]
        fn what_is_at_work_is_put_right_as_it_is_read() {
            let bag = bag_with(&[("cabinet", 1), ("barbecue", 1), ("lamp_with_shade", 1)], &["cabinet", "barbecue", "lamp_with_shade"]);
            let stack = |kind: &str, count| Some(Stack { kind: kind.into(), count });
            let mut slots = vec![None; 10];
            slots[0] = stack("doll", 200);
            slots[1] = stack("nothing_there_is", 2);
            slots[9] = stack("globe", 1);
            let kept = Workings {
                held: vec![
                    Held { id: 1, slots },
                    Held { id: 1, slots: vec![stack("doll", 1)] },
                    Held { id: 2, slots: vec![stack("doll", 1), stack("hamburger", 2)] },
                    Held { id: 9, slots: vec![stack("doll", 1)] },
                    Held { id: 3, slots: vec![stack("doll", 1)] },
                ],
                lit: vec![3, 3, 1, 9],
            };
            let sane = kept.sane(&bag.stands);
            // The cabinet's eight slots: as many dolls as may be owned in the
            // first, nothing of a kind there is not, and the globe from past
            // its slots in the first empty one; the second cabinet entry
            // gone. The barbecue's hamburgers and not its doll. Nothing of
            // what does not stand, nor in the lamp, and only the lamp lit.
            assert_eq!(sane.held.len(), 2);
            assert_eq!(sane.slots(1).len(), 8);
            assert_eq!(sane.slots(1)[0], stack("doll", MOST_OF_A_KIND));
            assert_eq!(sane.slots(1)[1], stack("globe", 1));
            assert_eq!(sane.slots(2), &[None, stack("hamburger", 2), None]);
            assert_eq!(sane.lit, [3]);
            // And as it was, as it is.
            assert_eq!(sane.clone().sane(&bag.stands), sane);
        }

        #[test]
        fn coins_go_into_a_piggy_bank_and_come_out_of_it_and_none_are_made_or_lost() {
            let bag = bag_with(&[(PIGGY, 1), ("cabinet", 1)], &[PIGGY, "cabinet"]);
            let mut banked = Banked::default();
            // A hundred of a thousand in: nine hundred left, a hundred in it.
            assert_eq!(deposit(1_000, &mut banked, &bag.stands, 1, 100), Ok(900));
            assert_eq!(banked.of(1), 100);
            // Anyone takes thirty out: their balance thirty more.
            assert_eq!(withdraw(5, &mut banked, &bag.stands, 1, 30), Ok(35));
            assert_eq!((banked.of(1), banked.clone().0), (70, vec![(1, 70)]));
            // Not more than there is, either way; nothing into a cabinet or
            // a piggy bank that is not there; nothing of nothing.
            assert_eq!(deposit(50, &mut banked, &bag.stands, 1, 51), Err(No::Coins));
            assert_eq!(withdraw(0, &mut banked, &bag.stands, 1, 71), Err(No::None));
            assert_eq!(deposit(50, &mut banked, &bag.stands, 2, 10), Err(No::Place));
            assert_eq!(deposit(50, &mut banked, &bag.stands, 9, 10), Err(No::None));
            assert_eq!(deposit(50, &mut banked, &bag.stands, 1, 0), Err(No::None));
            assert_eq!(withdraw(50, &mut banked, &bag.stands, 1, 0), Err(No::None));
            // Not past the most, and taken out, never past what a balance
            // holds.
            assert_eq!(deposit(u32::MAX, &mut banked, &bag.stands, 1, PIGGY_MOST), Err(No::Full));
            assert_eq!(withdraw(u32::MAX, &mut banked, &bag.stands, 1, 70), Err(No::Full));
            assert_eq!(banked.of(1), 70);
            // All of it out: nothing kept of it.
            assert_eq!(withdraw(0, &mut banked, &bag.stands, 1, 70), Ok(70));
            assert!(banked.0.is_empty());
            // Kept wrongly, put right: only standing piggy banks, none twice,
            // none empty, none past the most.
            let kept = Banked(vec![(1, 5), (1, 9), (2, 40), (7, 3), (1, 0)]);
            assert_eq!(kept.sane(&bag.stands), Banked(vec![(1, 5)]));
            assert_eq!(Banked(vec![(1, u32::MAX)]).sane(&bag.stands).of(1), PIGGY_MOST);
            assert!(thing(PIGGY).is_some());
        }

        #[test]
        fn what_works_goes_over_the_wire_after_everything_an_app_from_before_knows() {
            let number = |ask: Ask| postcard::to_allocvec(&ask).unwrap()[0];
            assert_eq!(number(Ask::Unpaint { id: 1, part: "x".into() }), 7);
            assert_eq!(number(Ask::Workings), 8);
            assert_eq!(number(Ask::Stow { who: 1, id: 2, slot: 3, kind: "doll".into(), count: 4 }), 9);
            assert_eq!(number(Ask::Unstow { who: 1, id: 2, slot: 3 }), 10);
            assert_eq!(number(Ask::Switch { who: 1, id: 2, on: true }), 11);
            assert_eq!(number(Ask::Run { who: 1, id: 2 }), 12);
            assert_eq!(number(Ask::Banked), 13);
            assert_eq!(number(Ask::Deposit { who: 1, id: 2, coins: 3 }), 14);
            assert_eq!(number(Ask::Withdraw { who: 1, id: 2, coins: 3 }), 15);
            let told = |tell: Tell| postcard::to_allocvec(&tell).unwrap()[0];
            assert_eq!(told(Tell::Painted { who: 1, asked: 0, paints: Vec::new() }), 2);
            let workings = Workings {
                held: vec![Held { id: 4, slots: vec![None, Some(Stack { kind: "hamburger".into(), count: 3 })] }],
                lit: vec![7],
            };
            let tell = Tell::Workings { who: u64::MAX, workings };
            assert_eq!(told(tell.clone()), 3);
            assert_eq!(postcard::from_bytes::<Tell>(&postcard::to_allocvec(&tell).unwrap()).unwrap(), tell);
            assert_eq!(told(Tell::Got(Stack { kind: "doll".into(), count: 1 })), 4);
            assert_eq!(told(Tell::Ran { who: 1, id: 2 }), 5);
            assert_eq!(told(Tell::Banked { who: 1, banked: Banked(vec![(2, 3)]) }), 6);
            assert_eq!(told(Tell::Coins(-50)), 7);
            let banked = Tell::Banked { who: u64::MAX, banked: Banked(vec![(1, PIGGY_MOST), (9, 1)]) };
            assert_eq!(postcard::from_bytes::<Tell>(&postcard::to_allocvec(&banked).unwrap()).unwrap(), banked);
            for ask in [
                Ask::Workings,
                Ask::Stow { who: u64::MAX, id: 2, slot: 11, kind: "square_house_ten_floor".into(), count: 99 },
                Ask::Unstow { who: 5, id: 2, slot: 0 },
                Ask::Switch { who: 5, id: 3, on: false },
                Ask::Run { who: 5, id: 9 },
            ] {
                let back: Ask = postcard::from_bytes(&postcard::to_allocvec(&ask).unwrap()).unwrap();
                assert_eq!(back, ask);
            }
            // A plot with a hundred dressers, every slot full, is still one
            // message, if a long one.
            let full = Workings {
                held: (0..MOST_STANDING as u32)
                    .map(|id| Held { id, slots: vec![Some(Stack { kind: "square_house_roofless_ten_floor".into(), count: 99 }); 12] })
                    .collect(),
                lit: Vec::new(),
            };
            assert!(postcard::to_allocvec(&Tell::Workings { who: 1, workings: full }).unwrap().len() < 48_000);
        }
    }
}

// --------------------------------------------------------------- messages

/// Where you are now: sent while it changes, 15 times a second, and nothing
/// while you stand still. Unreliable: a lost one is replaced by the next.
#[derive(Message, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Moved {
    /// How many times the server has moved you, as of this position. One
    /// sent before the server last moved you is from where you were before,
    /// and is dropped rather than taken for a leap.
    pub arrival: u8,
    pub spot: Spot,
}

/// Something you ask of the server. In order, and never lost.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub enum Ask {
    /// The first thing a device says: where it already is, so that it is let
    /// in there rather than moved.
    Hello { venue: Venue, spot: Spot },
    /// Take me home, or back to the town.
    Travel(Venue),
    /// Pay, and join a game of House Builder: from the House Builder's offer,
    /// or after one game for another.
    Play,
    /// Back to the town after a game.
    Leave,
    /// Give the house on `plot` this many stars.
    Rate { plot: u8, stars: u8 },
    /// End the wait early: for trying the game out, and only allowed by a
    /// server that says so.
    Skip,
    /// This piece is down on my plot, here. `id` is mine to choose, and says
    /// the same piece again if it is picked up and put down again.
    Put { id: u32, kind: String, at: At },
    /// This piece of mine is no longer down: picked back up, or thrown away.
    Take { id: u32 },
    /// Who I am, as chosen on signing up (the game's `signup`): the name to
    /// go by, and which of [`PEOPLE_LOOKS`] I wear. The server keeps both
    /// with the account and tells everyone in the room. Sent on coming online
    /// if the account has them otherwise, and whenever they change. One that
    /// is no name ([`profile::name`]) or no look is ignored. Last, since
    /// 2026-10-31: an app from before never sends it.
    Profile { name: String, look: String },
    /// I have fallen off the town into the void (since 2026-11-07, when the
    /// sea round it went): put me back where my seat arrives, as a fresh
    /// arrival ([`Tell::Arrive`]), so that the leap back is not taken for
    /// a move I could not have made.
    Respawn,
    /// My punch landed on the player whose [`Member::id`] this is: they are
    /// to be knocked back from me ([`Tell::Knocked`]). `running`, I was on
    /// the move as I hit them, which knocks them further, as a sprinting
    /// hit does in Minecraft. In a game of Bridge Wars the server takes the
    /// punch's damage off them too, if they are not on my team.
    Punch { who: u64, running: bool },
    /// Join a game of Bridge Wars: from the town, or from a game that is
    /// over for me, for another ([`Tell::Seated`] answers).
    Arena,
    /// My punch landed on `team`'s respawn anchor ([`ARENA_ANCHORS`]): the
    /// server takes the damage off it, if it is not my own.
    HitAnchor { team: u8 },
    /// I have laid this rung of a bridge, as [`Plank`] has it: everyone in
    /// the game is to see the plank.
    Rung {
        chain: u16,
        index: u16,
        x: f32,
        z: f32,
        heading: f32,
    },
    /// Sell me this, from Bridge Wars' shop: its price comes out of what I
    /// hold ([`bridge_wars::price`]), and [`Tell::Bought`] says it is
    /// mine; with too little, or nowhere near a shop, nothing happens.
    Buy(Ware),
    /// I have used what is on my second button in Bridge Wars: one fewer
    /// of it, and what it does is done (since 2026-11-12).
    Use(Use),
    /// Something about my friends, my group or my inbox ([`social::Act`],
    /// since 2026-11-13).
    Social(social::Act),
    /// Say this to everyone in my room ([`chat`], since 2026-11-14): the
    /// town's channel I am in, or my game. One that is no line
    /// ([`chat::line`]), or one too many too fast, is dropped.
    Chat(String),
    /// Where the builder I sent off is now ([`BuilderOf`], since
    /// 2026-11-14), for everyone else in the game to see it: `id` is my
    /// number for it, as I may have several out; with no spot, it is
    /// gone. Sent while it walks, as my own moves are.
    Builder { id: u8, spot: Option<Spot> },
    /// How many metres of bridge I have left to lay in my game of Bridge
    /// Wars (since 2026-11-15): they are my own device's to count, bought
    /// 25 at a time and laid a plank at a time, and the server only keeps
    /// what it was last told, to give back to me if I come back into the
    /// game on a device that has forgotten ([`Tell::Back`]). Said as it
    /// changes, no oftener than once a second.
    Metres(f32),
    /// One of the buses that drive round Bridge Wars' map has run me over
    /// (since 2026-11-16, [`bridge_wars::Bus::runs_over`]): which, from 0.
    /// My own device says so, as it says I have fallen ([`Ask::Respawn`]):
    /// it is where my body and the bus are at the very moment, and has
    /// laid me flat already. The server takes the bus's damage off me and
    /// has me flat for everyone ([`Flattened`]), if I am standing in the
    /// fight, not flat already, and where I last said I stood is anywhere
    /// near that bus's front.
    RunOver { bus: u8 },
    /// Join a game of one of the four (since 2026-11-18, [`table`]): from
    /// the town, or from a table that is over for me, for another
    /// ([`Tell::Entered`] answers). Refused for the two older games, which
    /// have their own ([`Ask::Arena`], [`Ask::Play`]).
    Join(Minigame),
    /// Something asked in my game of Blend In ([`blend_in::Ask`]).
    BlendIn(blend_in::Ask),
    /// Something asked in my game of Knock Off ([`knock_off::Ask`]).
    KnockOff(knock_off::Ask),
    /// Something asked in my game of Bridge Race ([`bridge_race::Ask`]).
    BridgeRace(bridge_race::Ask),
    /// Something asked in my game of Last Stand ([`last_stand::Ask`]).
    LastStand(last_stand::Ask),
    /// Tell me of my channel of the town ([`Tell::Channel`]): who sits in
    /// which of its seats, and so whose plot is whose (the game's `land`),
    /// and who is in their own car (`drive`), now and whenever either
    /// changes. Said once, on coming online, by an app that knows how to
    /// read the answer. Last but one, since 2026-11-21, with no new
    /// protocol: an app from before never says it and is never told, and a
    /// server from before ignores it, as either ignores what it cannot
    /// read.
    Channel,
    /// I am in my own car now (`true`), or out of it: for everyone else in
    /// my channel of the town to see the car, and not me walking where it
    /// goes ([`Tell::Channel`]). Last but one, since 2026-11-21, as
    /// [`Ask::Channel`] is. Since the bag came, later that day, the server
    /// takes it only from someone who owns the car ([`bag::Bag::has_car`]),
    /// which everyone does who has not sold theirs.
    Drive(bool),
    /// Something about my bag: what I own, buying and selling, and what of
    /// it stands on my plot of the town ([`bag::Ask`], since 2026-11-21).
    /// With no new protocol, as [`Ask::Channel`] is: an app from
    /// before never says it, and a server from before ignores it.
    Bag(bag::Ask),
    /// Tell me who in my channel of the town sits or lies on something
    /// ([`Tell::Resting`]), now and whenever that changes. Said once, on
    /// coming online, by an app that knows how to draw it. Last but one,
    /// since 2026-11-22, with no new protocol, as [`Ask::Channel`] is.
    Resting,
    /// I have sat or lain down on something in the town, like this
    /// ([`Pose`]), or am up again (`None`): for everyone else in my
    /// channel to see me so, where my body is. Since 2026-11-22, as
    /// [`Ask::Resting`] is.
    Rest(Option<Pose>),
    /// Tell me which car each of those in my channel of the town who are in
    /// theirs is in ([`Tell::Cars`]), now and whenever that changes. Said
    /// once, on coming online, by an app that knows the cars there are
    /// (since 2026-11-25, when the shop came to sell 22 more). With no new
    /// protocol, as [`Ask::Channel`] is: an app from before never says it,
    /// and is never told.
    Cars,
    /// The car I am in, or am getting into: one of mine, by its kind
    /// ([`bag::THINGS`], a vehicle). Said before [`Ask::Drive`]`(true)`, and
    /// as I come online in it. Last, since 2026-11-25. Until it is said, and
    /// from an app from before, which never says it, the car is the one
    /// everyone begins with ([`bag::CAR`]).
    Car(String),
    /// Say this, one of the things to say at a press, to everyone in my
    /// room ([`chat::Quick`], since 2026-11-27): as [`Ask::Chat`] says a
    /// line typed, by its number alone. One that is no line a list could
    /// have, or one too many too fast, is dropped.
    Quick(chat::Quick),
    /// Something about houses: visiting, rating, the board ([`house::Ask`],
    /// since 2026-11-27).
    House(house::Ask),
}

/// Someone in a channel of the town who is in their own car, as
/// [`Tell::Cars`] has them: who, by their [`Member::id`], and which car, by
/// its kind ([`bag::THINGS`]).
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Driver {
    pub who: u64,
    pub car: String,
}

/// How someone rests on a chair, a sofa or a bed in the town (the game's
/// `rest`, since 2026-11-22): what everyone else has to know of it to draw
/// them so, where their body is and facing the way it faces.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Pose {
    /// Sat, the knees bent over the front of the seat.
    Sit,
    /// Sat back, the legs out along the seat, as on a sofa.
    Lounge,
    /// Lying on the back, the head a little raised, as on a pillow.
    Lie,
}

/// Someone in a channel of the town who rests, as [`Tell::Resting`] has
/// them: who, by their [`Member::id`], and how.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Posed {
    pub who: u64,
    pub pose: Pose,
}

/// One person in a channel of the town, as [`Tell::Channel`] has them: who,
/// by their [`Member::id`]; which of the channel's seats they sit in, which
/// says which plot of the town is theirs ([`land`]); and whether they are
/// in their own car ([`Ask::Drive`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Seated {
    pub who: u64,
    pub seat: u8,
    pub driving: bool,
}

/// What the server tells you alone. In order, and never lost.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub enum Tell {
    /// You are in, and this is who you are to everyone else.
    Welcome { me: Member },
    /// The server has put you on `venue`, where `seat` arrives there, or, with
    /// a `spot`, where you already stood. `arrival` is what to send with your
    /// moves from now on.
    Arrive {
        venue: Venue,
        seat: u8,
        spot: Option<Spot>,
        arrival: u8,
    },
    /// The last move was not one a body could make: back to here.
    PutBack { arrival: u8, spot: Spot },
    /// What you have, now that it has changed.
    Balance(u32),
    /// You are in a game of House Builder, in this seat: your plot.
    Joined { seat: u8 },
    /// You are in a game of Bridge Wars, in this seat: your island is
    /// [`arena_island`] of it, your team [`arena_team`].
    Seated { seat: u8 },
    /// The player whose [`Member::id`] is `by` punched you, standing at
    /// `from` (in the island's space, or on a bus, in the bus's): your own
    /// device knocks your body back from there, and sends where it goes as
    /// it does any move. `running`, they hit you on the move.
    Knocked { by: u64, from: Spot, running: bool },
    /// What you asked for cannot be done.
    Refused(Refusal),
    /// The shop has sold you this ([`Ask::Buy`]): its price is out of what
    /// you hold, as the room's [`Purses`] will show.
    Bought(Ware),
    /// Something about your friends, your group or your inbox
    /// ([`social::News`], since 2026-11-13).
    Social(social::News),
    /// Someone in your room has said this ([`Ask::Chat`], since
    /// 2026-11-14; [`Ask::Quick`] since 2026-11-27), you among them: who, by
    /// their [`Member::id`], the name they go by, and what they said: a line
    /// typed, or one of the things to say at a press by its number
    /// ([`chat::Said`], since 2026-11-27, protocol 27; the line typed alone
    /// until then).
    Said { who: u64, name: String, said: chat::Said },
    /// Something has happened in your game of Bridge Wars ([`chat::Feed`]).
    Feed(chat::Feed),
    /// You are back in the game of Bridge Wars you were in when your
    /// connection went, in the seat you had (since 2026-11-15; Hajun: "if
    /// u exit the app while ur playing and reopen, u shouldnt go to the
    /// main lobby - u should rejoin the game as long as there is atleast 1
    /// real player in that game still"): the game is begun here afresh,
    /// as [`Tell::Seated`] begins one, and everything in it comes as it
    /// stands, your own bridges with the rest. With it, what only your
    /// own device knew: the metres of bridge you had left to lay
    /// ([`Ask::Metres`]) and whether you had the compass. An
    /// [`Tell::Arrive`] follows, standing you where your body stood all
    /// the while. A device that never left the game, its app only put
    /// away for a moment, is told none of this: it is let in where it
    /// stands and carries on.
    Back { seat: u8, metres: f32, compass: bool },
    /// You are at a table of `game`, in this seat ([`table`], since
    /// 2026-11-18): the game is begun afresh here. Sent as you are seated,
    /// and again to a device that comes back to a game it was in.
    Entered { game: Minigame, seat: u8 },
    /// Something for you alone in your game of Blend In ([`blend_in::Tell`]).
    BlendIn(blend_in::Tell),
    /// Something for you alone in your game of Knock Off ([`knock_off::Tell`]).
    KnockOff(knock_off::Tell),
    /// Something for you alone in your game of Bridge Race
    /// ([`bridge_race::Tell`]).
    BridgeRace(bridge_race::Tell),
    /// Something for you alone in your game of Last Stand
    /// ([`last_stand::Tell`]).
    LastStand(last_stand::Tell),
    /// Everyone in your channel of the town, you among them, as they sit
    /// and as they go ([`Seated`]): all of them, as you come into the
    /// channel and whenever any of it changes (someone comes or goes, gets
    /// into their car or out of it). Only to an app that asked
    /// ([`Ask::Channel`]). Last but one, since 2026-11-21, with no new
    /// protocol.
    Channel(Vec<Seated>),
    /// Something about a bag: your own, or what stands on the plot of
    /// someone else in your channel of the town ([`bag::Tell`], since
    /// 2026-11-21). Only to an app that asked ([`bag::Ask::Watch`]). With
    /// no new protocol.
    Bag(bag::Tell),
    /// Everyone in your channel of the town who sits or lies on something,
    /// you among them, and how ([`Posed`]): all of them, as you come into
    /// the channel and whenever any of it changes. Only to an app that
    /// asked ([`Ask::Resting`]). Last but one, since 2026-11-22, with no
    /// new protocol.
    Resting(Vec<Posed>),
    /// Whose every plot of your channel of the town is ([`land::Deed`],
    /// since 2026-11-23): all 48, your own among them, as you come into
    /// the channel and whenever any of it changes: someone comes or goes,
    /// into a game or back, takes another name, or a plot is filled with
    /// something else. What stands on each comes as what stands on
    /// anyone's does ([`bag::Tell::Theirs`], by the deed's `who`), after
    /// this. Only to an app that asked of its channel ([`Ask::Channel`]).
    Plots(Vec<land::Deed>),
    /// Which car everyone in your channel of the town who is in theirs is
    /// in ([`Driver`]), you among them: all of them, as you come into the
    /// channel and whenever any of it changes. Whoever is in theirs and not
    /// here is in the one everyone begins with. Only to an app that asked
    /// ([`Ask::Cars`]). Last, since 2026-11-25, with no new protocol.
    Cars(Vec<Driver>),
    /// Something about houses ([`house::Tell`], since 2026-11-27): only to
    /// an app that asked ([`house::Ask::Watch`]).
    House(house::Tell),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// You have not got the coins.
    Coins,
    /// You are in the middle of something else.
    Busy,
    /// You are in a group someone else leads, and only its leader starts a
    /// game ([`social`], since 2026-11-13).
    NotLeader,
}

/// What time it is, asked of the server to set the town's clock by (the
/// game's `clock`): when it was asked, by the device's own steady clock,
/// which comes back with the answer, so that the device can tell how long the
/// answer took. Unreliable: one that is lost is never answered, and the next
/// is asked in its place.
#[derive(Message, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct WhatTime {
    pub asked: f64,
}

/// The server's answer to [`WhatTime`], written the moment it is asked: the
/// time by its clock as it answers, in seconds since 1970, and when it was
/// asked, as the device said. Independent of replication, as nothing in it
/// waits on anything replicated: it goes out at once, rather than with the
/// next tick, which would hold it up to a fifteenth of a second, and put
/// the town's clock out by half that.
#[derive(Message, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct TheTime {
    pub asked: f64,
    pub now: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_wars_costs_fifty_and_a_team_that_beats_two_of_the_three_others_comes_out_ahead() {
        use bridge_wars::{FEE, PRIZES, prizes};
        assert_eq!(FEE, 50);
        // Hajun's rule: more than half beaten pays more than the fee, the
        // rest less.
        assert!(PRIZES[0] > FEE && PRIZES[1] > FEE && PRIZES[2] < FEE && PRIZES[3] < FEE);
        // By seat: seats 0 and 4 are team 0, 1 and 5 team 1, and so on; the
        // computer's seats (here 4 to 7) are paid nothing.
        let people = [true, true, true, true, false, false, false, false];
        assert_eq!(prizes(&[0, 1, 2, 3], &people), vec![130, 60, 10, 0, 0, 0, 0, 0]);
        assert_eq!(prizes(&[3, 0, 2, 1], &people), vec![0, 130, 10, 60, 0, 0, 0, 0]);
        // Two people on the winning team: each is paid.
        let two = [true, false, false, false, true, false, false, false];
        assert_eq!(prizes(&[0, 1, 2, 3], &two), vec![130, 0, 0, 0, 130, 0, 0, 0]);
        // Out in the same moment, or both still in at the draw: the places
        // they take up, shared; all four in at the draw, the fee back.
        assert_eq!(prizes(&[0, 0, 2, 3], &people)[..4], [95, 95, 10, 0]);
        assert_eq!(prizes(&[0, 0, 0, 0], &people)[..4], [50, 50, 50, 50]);
        assert_eq!(prizes(&[0, 1, 2, 2], &people)[..4], [130, 60, 5, 5]);
        // Places by how long each team lasted: the one left in first, the
        // last out second; out together, together; all still in, all first.
        use bridge_wars::places;
        assert_eq!(places(&[Some(300.0), None, Some(900.0), Some(50.0)]), [2, 0, 1, 3]);
        assert_eq!(places(&[Some(300.0), Some(300.0), None, Some(50.0)]), [1, 1, 0, 3]);
        assert_eq!(places(&[None, None, Some(900.0), None]), [0, 0, 3, 0]);
        // Everyone out at once (the last two fallen together): first, both.
        assert_eq!(places(&[Some(1000.0), Some(1000.0), Some(10.0), Some(20.0)]), [0, 0, 3, 2]);
    }

    #[test]
    fn a_handle_is_a_name_and_its_tag() {
        use social::{handle, split_handle};
        assert_eq!(handle("Hajooon", 1323), "Hajooon#1323");
        assert_eq!(split_handle("Hajooon#1323"), Some(("Hajooon", 1323)));
        assert_eq!(split_handle("  Mochi 482#9999 "), Some(("Mochi 482", 9999)));
        // A name may have a # in it: the tag is what comes after the last.
        assert_eq!(split_handle("a#b#1000"), Some(("a#b", 1000)));
        // Four digits, from 1000 up, after something that is a name.
        for not in ["Hajooon", "Hajooon#", "Hajooon#123", "Hajooon#12345", "Hajooon#0999", "Hajooon#12a4", "#1234", "a#1234", "Hajooon #1234"] {
            assert_eq!(split_handle(not), None, "{not:?}");
        }
        // Every tag there is reads back.
        for tag in [social::TAG_FIRST, 5000, social::TAG_LAST] {
            assert_eq!(split_handle(&handle("Bob", tag)), Some(("Bob", tag)));
        }
    }

    #[test]
    fn a_message_is_short_plain_and_something() {
        use social::{MESSAGE_LONGEST, message};
        assert_eq!(message("  play at 9?  ").as_deref(), Some("play at 9?"));
        assert_eq!(message(""), None);
        assert_eq!(message("   "), None);
        // Korean and Vietnamese, as English; a new line is a space; what
        // the game does not write is left out, and nothing left is none.
        assert_eq!(message(" 안녕 ㅋㅋ ").as_deref(), Some("안녕 ㅋㅋ"));
        assert_eq!(message("Chào bạn!").as_deref(), Some("Chào bạn!"));
        assert_eq!(message("two\nlines").as_deref(), Some("two lines"));
        assert_eq!(message("gg \u{1F600}").as_deref(), Some("gg"));
        assert_eq!(message("\u{1F600}"), None);
        let longest = "x".repeat(MESSAGE_LONGEST);
        assert_eq!(message(&longest).as_deref(), Some(longest.as_str()));
        assert_eq!(message(&"x".repeat(MESSAGE_LONGEST + 1)), None);
        // A Hangul syllable is as long as two letters.
        let longest = "가".repeat(MESSAGE_LONGEST / 2);
        assert_eq!(message(&longest).as_deref(), Some(longest.as_str()));
        assert_eq!(message(&"가".repeat(MESSAGE_LONGEST / 2 + 1)), None);
    }

    #[test]
    fn common_swear_words_are_starred_out_letter_for_letter_and_everyday_words_left_alone() {
        use text::masked;
        // English: the word, whatever its capitals, and whatever it is
        // part of or drawn out to; the rest of the line as it was.
        assert_eq!(masked("what the fuck"), "what the ****");
        assert_eq!(masked("FUCK you, Shithead!"), "**** you, ********!");
        assert_eq!(masked("fuuuuck this bullshit"), "******* this ********");
        assert_eq!(masked("you motherfucker, biiitch"), "you ************, *******");
        assert_eq!(masked("kick ass, asshole"), "kick ***, *******");
        assert_eq!(masked("fuck123 fuck_you"), "****123 ****_you");
        assert_eq!(masked("dick, cock and pussy"), "****, **** and *****");
        // Everyday words with those letters in them are not swear words.
        for fine in [
            "class pass assassin grass bass", "cockpit peacock cocktail Hancock", "Dickens dickson", "Scunthorpe",
            "snigger", "as is", "shut the door", "duck luck", "I pushed it", "hello world gg wp", "analysis",
            "cac ban oi", "di choi", "lon coca", "lol", "dm me", "cc", "",
        ] {
            assert_eq!(masked(fine), fine);
        }
        // Korean: where it stands in a run of Hangul, itself and no more,
        // and the letters alone it is shortened to.
        assert_eq!(masked("아 시발 진짜"), "아 ** 진짜");
        assert_eq!(masked("아씨발진짜뭐야"), "아**진짜뭐야");
        assert_eq!(masked("ㅅㅂ ㅋㅋㅋ"), "** ㅋㅋㅋ");
        assert_eq!(masked("이 개새끼야 병신아"), "이 ***야 **아");
        assert_eq!(masked("좆같네 지랄하네"), "*같네 **하네");
        for fine in ["안녕하세요 ㅋㅋ ㅎㅎ", "다시발견했다", "시발점", "보지 마 자지 마", "새끼 고양이", "미친 듯이 달려"] {
            assert_eq!(masked(fine), fine);
        }
        // Vietnamese: a word of its own, as it is spelt with its marks.
        assert_eq!(masked("Địt mẹ mày"), "*** mẹ mày");
        assert_eq!(masked("ngu vl, đéo hiểu"), "ngu **, *** hiểu");
        assert_eq!(masked("vcl ĐM"), "*** **");
        for fine in ["Xin chào các bạn", "tôi đi chơi", "đẹp quá", "Việt Nam"] {
            assert_eq!(masked(fine), fine);
        }
        // As long as it was, letter for letter; and done twice it is as
        // done once.
        for said in ["what the fuck", "아씨발진짜", "Địt mẹ", "ok"] {
            assert_eq!(masked(said).chars().count(), said.chars().count());
            assert_eq!(masked(&masked(said)), masked(said));
        }
        // Words run together with a capital to each, as names are: each
        // by itself, and the rest left.
        assert_eq!(masked("BigDick and fuckYou"), "Big**** and *******");
        assert_eq!(masked("fUcK"), "****");
        assert_eq!(masked("McDonald DeAndre iPhone"), "McDonald DeAndre iPhone");
        // What is said and what is written to a friend are masked, by
        // whoever cleans them.
        assert_eq!(chat::line("  fuck this  ").as_deref(), Some("**** this"));
        assert_eq!(social::message("씨발 뭐야").as_deref(), Some("** 뭐야"));
        assert!(text::swears("oh shit") && !text::swears("oh ship") && !text::swears(""));
    }

    #[test]
    fn a_name_with_a_swear_word_in_it_is_no_name_and_is_said_to_be_rude() {
        // Refused, in any of the three languages, however it is dressed
        // up with capitals, digits and signs round it.
        for rude in ["asshole", "xXfuckXx", "BigDick", "Shit Head", "bitch_99", "씨발놈", "개새끼", "ㅅㅂ", "Địt mẹ", "fuuuck"] {
            assert_eq!(profile::name(rude), None, "{rude}");
            assert!(profile::rude(rude), "{rude}");
        }
        // Names that only have such letters in them are names.
        for fine in [
            "Dickens", "Hancock", "Cassidy", "Bassist", "Scunthorpe", "Mochi 482", "Hajun", "하준", "민", "동그라미<3",
            "다시발견", "Nguyễn Văn Hải", "iLikeQuynhAnh", "hihihohohaha", "xd", "Glass Bass1", "Peacock",
        ] {
            assert_eq!(profile::name(fine).as_deref(), Some(fine), "{fine}");
            assert!(!profile::rude(fine), "{fine}");
        }
        // What is no name for its length is not rude, only no name: there
        // is nothing to pick another of.
        assert!(!profile::rude("a") && !profile::rude("") && !profile::rude("a name far too long to be one"));
        assert!(!profile::rude("fuck this is far too long to be a name"));
        // Nobody the computer plays is called anything rude.
        for name in BOY_NAMES.iter().chain(GIRL_NAMES.iter()) {
            assert!(!text::swears(name), "{name}");
        }
        // A name with its tag is found by its tag only if it is a name.
        assert_eq!(social::split_handle("Dickens#1234"), Some(("Dickens", 1234)));
        assert_eq!(social::split_handle("asshole#1234"), None);
    }

    #[test]
    fn what_is_written_is_english_korean_and_vietnamese_put_together_from_its_pieces() {
        use text::{allowed, clean, fitted, length, piece, typed, width};
        // English, as ever; Korean, syllables and the letters alone;
        // Vietnamese, every vowel with every tone, capital and small.
        for c in "Az09 !~<3#".chars().chain("한글ㅋㅎㅏ".chars()).chain("ăâđêôơưĂÂĐÊÔƠƯàáảãạằắẳẵặầấẩẫậèéẻẽẹềếểễệìíỉĩịòóỏõọồốổỗộờớởỡợùúủũụừứửữựỳýỷỹỵỲÝỶỸỴ".chars()) {
            assert!(allowed(c), "{c:?}");
        }
        // And what a phone's keyboard writes by itself.
        for c in "\u{2018}\u{2019}\u{201C}\u{201D}\u{2026}\u{2013}\u{2014}".chars() {
            assert!(allowed(c), "{c:?}");
        }
        // Not what the fonts cannot draw: emoji, Hanja, kana, Cyrillic,
        // the letters of other languages, a new line, a soft hyphen.
        for c in "\u{1F600}漢かЖłİ\n\t\u{AD}\u{A0}\u{3164}".chars() {
            assert!(!allowed(c), "{c:?}");
        }
        // The pieces a letter may come in are typed, and are no letter by
        // themselves.
        for c in "\u{300}\u{301}\u{303}\u{309}\u{323}\u{302}\u{306}\u{31B}\u{1100}\u{1161}\u{11AB}".chars() {
            assert!(piece(c) && typed(c) && !allowed(c), "{c:?}");
        }
        assert!(typed('a') && typed('한') && !typed('\u{1F600}'));
        // Put together: a vowel and its marks, in whatever order a
        // keyboard sent them; a syllable from its parts.
        assert_eq!(clean("Vie\u{302}\u{323}t"), "Việt");
        assert_eq!(clean("Vi\u{EA}\u{323}t"), "Việt");
        assert_eq!(clean("Việt"), "Việt");
        assert_eq!(clean("\u{1112}\u{1161}\u{11AB}\u{1100}\u{1173}\u{11AF}"), "한글");
        // A piece with nothing to go on, or on a letter that takes none,
        // is left out.
        assert_eq!(clean("q\u{301}x"), "qx");
        assert_eq!(clean("\u{1100}"), "");
        // Spaces of every kind are a space, and those round it trimmed.
        assert_eq!(clean("\ta\u{A0}b\u{3000}c\n"), "a b c");
        // As it is shown while it is typed: put together, nothing left
        // out.
        assert_eq!(text::whole(" Vie\u{302}\u{323}t\u{1F600} "), " Việt\u{1F600} ");
        // Clean twice is clean once.
        for raw in ["  Vie\u{302}\u{323}t  Nam ", "한\u{1F600}글", "plain", ""] {
            assert_eq!(clean(&clean(raw)), clean(raw), "{raw:?}");
        }
        // Widths: a Latin letter one, Hangul two, a mark none.
        assert_eq!((width('a'), width('ệ'), width('한'), width('ㅋ'), width('\u{301}')), (1, 1, 2, 2, 0));
        assert_eq!(length("Việt Nam"), 8);
        assert_eq!(length("한글 ab"), 7);
        assert_eq!(length("Vie\u{302}\u{323}t"), 4);
        assert_eq!(length("\u{1112}\u{1161}\u{11AB}"), 2);
        // The front of a text that fits, never cut inside a letter.
        assert_eq!(fitted("한글abc", 5), "한글a");
        assert_eq!(fitted("한글abc", 3), "한");
        assert_eq!(fitted("abc", 10), "abc");
        assert_eq!(fitted("e\u{301}x", 1), "e\u{301}");
        assert_eq!(fitted("", 4), "");
    }

    #[test]
    fn a_level_is_one_to_start_and_one_more_for_every_hundred() {
        use social::{LEVEL_MOST, XP_A_LEVEL, level};
        assert_eq!(level(0), 1);
        assert_eq!(level(XP_A_LEVEL - 1), 1);
        assert_eq!(level(XP_A_LEVEL), 2);
        assert_eq!(level(52 * XP_A_LEVEL + 7), 53);
        assert_eq!(level(u32::MAX), LEVEL_MOST);
    }

    #[test]
    fn the_leap_potion_lasts_its_while_and_a_team_s_building_quickens_step_by_step() {
        use bridge_wars::{
            BUILD_PACE, BUILD_SPEED_MOST, BUILDER_SPEED, LEAP, LEAP_SECS, Outfits, Shop, build_pace, builder_speed, buy,
            leap_speed, price, sold_at,
        };
        // The leap potion: 1 amethyst at the item shop, drunk, and lasting
        // its while from the moment it is.
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        let mut purse: Purse = [0, 0, 0, 2];
        assert_eq!(sold_at(Ware::Utility(Utility::Leap)), Shop::Items);
        assert!(outfits.buy(0, Ware::Utility(Utility::Leap), &mut purse));
        assert_eq!(purse, [0, 0, 0, 1]);
        assert!(Utility::Leap.drunk() && Utility::Leap.lasts() && !Utility::Leap.aimed());
        assert!(!outfits.kit(0).leaping(100.0));
        assert_eq!(outfits.drink(0, Utility::Leap, 100.2), Some(Utility::Leap));
        let kit = outfits.kit(0);
        assert_eq!((LEAP_SECS, kit.leap_until, kit.lasts_until(Utility::Leap)), (45.0, 146, 146));
        assert!(kit.leaping(100.2) && kit.leaping(145.9) && !kit.leaping(146.0));
        assert_eq!((kit.lasts_until(Utility::Speed), kit.lasts_until(Utility::Healing)), (0, 0));
        // Its button is its own while it lasts, with none left on it, and
        // empty once it is over.
        outfits.tick(0.1, 120.0, &[]);
        assert_eq!(outfits.kit(0).slots[0], Some(Stack { utility: Utility::Leap, count: 0 }));
        outfits.tick(0.1, 146.5, &[]);
        assert_eq!((outfits.kit(0).slots[0], outfits.kit(0).leap_until), (None, 0));
        // Five times as high is the square root of five times as fast off
        // the ground, the fall being what it is.
        assert_eq!(LEAP, 5.0);
        assert!((leap_speed() * leap_speed() - LEAP).abs() < 1e-5);
        // Dying, it is gone with the rest.
        assert!(outfits.buy(0, Ware::Utility(Utility::Leap), &mut purse));
        assert_eq!(outfits.drink(0, Utility::Leap, 200.0), Some(Utility::Leap));
        outfits.die(0);
        assert!(!outfits.kit(0).leaping(201.0));
        // It goes over the wire with the rest of a kit.
        let kit = Kit { leap_until: 300, ..Kit::default() }.holding(Utility::Leap, 2);
        let back: Kit = postcard::from_bytes(&postcard::to_allocvec(&kit).unwrap()).unwrap();
        assert_eq!(back, kit);

        // A team's building: half a run to begin with, and three steps at
        // the upgrade shop, for 4, 8 and 16 sapphire, to three quarters of
        // one, all of one, and a quarter over.
        let (mut kit, mut team) = (Kit::default(), Upgrades::default());
        assert_eq!(sold_at(Ware::BuildSpeed), Shop::Upgrades);
        assert_eq!((build_pace(&team), builder_speed(&team)), (0.5, BUILDER_SPEED));
        let mut purse: Purse = [0, 0, 40, 0];
        let mut steps = Vec::new();
        while let Some((item, cost)) = price(Ware::BuildSpeed, &kit, &team) {
            assert_eq!(item, Item::Sapphire);
            assert!(buy(Ware::BuildSpeed, &mut purse, &mut kit, &mut team));
            steps.push((cost, build_pace(&team)));
        }
        assert_eq!(steps, vec![(4, 0.75), (8, 1.0), (16, 1.25)]);
        assert_eq!((team.build_speed, purse[2]), (BUILD_SPEED_MOST, 40 - 28));
        assert!(!buy(Ware::BuildSpeed, &mut purse, &mut kit, &mut team));
        assert_eq!(builder_speed(&team), 12.5);
        assert_eq!(BUILD_PACE.len(), usize::from(BUILD_SPEED_MOST) + 1);
        let back: Upgrades = postcard::from_bytes(&postcard::to_allocvec(&team).unwrap()).unwrap();
        assert_eq!(back, team);
    }

    #[test]
    fn a_line_of_chat_is_short_and_plain_and_what_a_game_says_goes_over_the_wire() {
        use chat::{BURST, CREDIT_SECS, EVERY, Feed, LONGEST, Weapon, line, lines_left};
        // A line: something, cleaned, no longer than it may be; English,
        // Korean or Vietnamese.
        assert_eq!(line("  gg wp  ").as_deref(), Some("gg wp"));
        assert_eq!(line(""), None);
        assert_eq!(line("   "), None);
        assert_eq!(line("two\nlines").as_deref(), Some("two lines"));
        assert_eq!(line("\u{c548}\u{b155} ㅎㅎ").as_deref(), Some("안녕 ㅎㅎ"));
        assert_eq!(line("xin cha\u{300}o").as_deref(), Some("xin chào"));
        assert_eq!(line("\u{1F600}\u{1F600}"), None, "nothing the game writes");
        let longest = "x".repeat(LONGEST);
        assert_eq!(line(&longest).as_deref(), Some(longest.as_str()));
        assert_eq!(line(&"x".repeat(LONGEST + 1)), None);
        let longest = "글".repeat(LONGEST / 2);
        assert_eq!(line(&longest).as_deref(), Some(longest.as_str()));
        assert_eq!(line(&"글".repeat(LONGEST / 2 + 1)), None);
        // So many at once, and one more for every while gone by.
        assert_eq!(lines_left(BURST, 1000.0), BURST);
        assert_eq!(lines_left(0.0, 0.0), 0.0);
        assert!((lines_left(0.0, EVERY) - 1.0).abs() < 1e-6);
        assert!(lines_left(0.5, -3.0) == 0.5, "time never runs back");
        assert!(CREDIT_SECS > 2.0 && CREDIT_SECS < 15.0);
        // What a game says, and what someone says, there and back.
        for feed in [
            Feed::Fell { seat: 3 },
            Feed::Knocked { seat: 3, by: 6 },
            Feed::Killed { seat: 0, by: 11, with: Weapon::Sword(2) },
            Feed::Killed { seat: 0, by: 1, with: Weapon::Dynamite },
            Feed::Bought { by: 4, ware: Ware::BuildSpeed, step: 2 },
            Feed::AnchorBroken { team: 2, by: 5 },
        ] {
            let bytes = postcard::to_allocvec(&Tell::Feed(feed)).unwrap();
            let Tell::Feed(back) = postcard::from_bytes(&bytes).unwrap() else {
                panic!("not what was sent");
            };
            assert_eq!(back, feed);
            assert!(bytes.len() <= 6, "{feed:?} is {} bytes", bytes.len());
        }
        // What a team bought is for that team alone; the rest for all.
        assert_eq!(Feed::Bought { by: 4, ware: Ware::HealPool, step: 1 }.only_for(), Some(arena_team(4)));
        assert_eq!(Feed::Fell { seat: 4 }.only_for(), None);
        let typed = chat::Said::Typed("hello".into());
        let said = Tell::Said { who: 9, name: "Bob".into(), said: typed.clone() };
        let Tell::Said { who, name, said } = postcard::from_bytes(&postcard::to_allocvec(&said).unwrap()).unwrap() else {
            panic!("not what was sent");
        };
        assert_eq!((who, name.as_str(), said), (9, "Bob", typed));
        // One of the things to say at a press, by its number alone: a few
        // bytes, whatever its words.
        let quick = chat::Quick { lines: chat::Lines::Town, line: 7 };
        assert!(quick.sane() && !chat::Quick { line: chat::QUICK_MOST, ..quick }.sane());
        let bytes = postcard::to_allocvec(&Ask::Quick(quick)).unwrap();
        assert!(bytes.len() <= 4, "{} bytes", bytes.len());
        let Ask::Quick(back) = postcard::from_bytes(&bytes).unwrap() else {
            panic!("not what was sent");
        };
        assert_eq!(back, quick);
        let Ask::Chat(back) = postcard::from_bytes(&postcard::to_allocvec(&Ask::Chat("hello".into())).unwrap()).unwrap()
        else {
            panic!("not what was sent");
        };
        assert_eq!(back, "hello");
        // And where a builder is, or that it is gone.
        let spot = Spot::of(Vec3::new(12.5, 0.24, -40.0), Quat::from_rotation_y(1.0));
        for ask in [Ask::Builder { id: 3, spot: Some(spot) }, Ask::Builder { id: 3, spot: None }] {
            let Ask::Builder { id, spot: back } = postcard::from_bytes(&postcard::to_allocvec(&ask).unwrap()).unwrap() else {
                panic!("not what was sent");
            };
            let Ask::Builder { spot: sent, .. } = ask else { unreachable!() };
            assert_eq!(id, 3);
            assert_eq!(back.is_some(), sent.is_some());
            assert!(back.zip(sent).is_none_or(|(back, sent)| back.same(&sent)));
        }
    }

    #[test]
    fn what_is_said_about_friends_goes_over_the_wire_and_back() {
        use social::*;
        let card = Card {
            id: 7,
            name: "BobTheBuilder".into(),
            tag: 1234,
            level: 123,
            presence: Presence::Playing {
                game: Minigame::BridgeWars,
                queued: true,
            },
        };
        assert_eq!(card.handle(), "BobTheBuilder#1234");
        assert!(card.presence.online());
        assert!(!Presence::Offline { since: 0 }.online());
        let news = News::Inbox(vec![
            Mail {
                id: 3,
                from: card.clone(),
                what: Post::Request,
            },
            Mail {
                id: 4,
                from: card.clone(),
                what: Post::Message("play at 9?".into()),
            },
        ]);
        let bytes = postcard::to_allocvec(&Tell::Social(news.clone())).unwrap();
        let Tell::Social(back) = postcard::from_bytes(&bytes).unwrap() else {
            panic!("not what was sent");
        };
        assert_eq!(back, news);
        let act = Act::Write {
            to: 7,
            text: "ok".into(),
        };
        let bytes = postcard::to_allocvec(&Ask::Social(act.clone())).unwrap();
        let Ask::Social(back) = postcard::from_bytes(&bytes).unwrap() else {
            panic!("not what was sent");
        };
        assert_eq!(back, act);
        // A friend standing in the town is a dozen bytes and their name.
        let town = Card {
            presence: Presence::Town { channel: 0 },
            ..card
        };
        assert!(postcard::to_allocvec(&town).unwrap().len() < 24, "a card is small");
    }

    #[test]
    fn a_spot_keeps_where_and_which_way() {
        for (at, yaw) in [
            (Vec3::new(5.55, 0.0, -3.21), 0.0),
            (Vec3::new(-580.0, 12.34, 580.0), 1.0),
            (Vec3::new(0.004, -0.7, 0.0), -2.5),
            (Vec3::ZERO, std::f32::consts::PI),
        ] {
            let rotation = Quat::from_rotation_y(yaw);
            let spot = Spot::of(at, rotation);
            assert!(spot.translation().distance(at) < 0.01, "{at} came back {}", spot.translation());
            let back = spot.rotation() * Vec3::NEG_Z;
            let was = rotation * Vec3::NEG_Z;
            assert!(back.distance(was) < 1e-3, "{yaw}: {back} for {was}");
        }
    }

    #[test]
    fn the_first_eight_arrive_where_they_did_offline() {
        let venue = Venue::Lobby;
        for seat in 0..8 {
            let (at, turn) = arrival(venue, seat);
            let was = Quat::from_rotation_y(seat as f32 * TAU / 8.0) * Vec3::new(0.0, 0.0, ARRIVAL_RADIUS);
            assert!(at.distance(was) < 1e-4, "seat {seat}: {at} for {was}");
            let facing = turn * Vec3::NEG_Z;
            assert!(facing.distance(-was.normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
    }

    #[test]
    fn the_first_four_of_a_game_of_bridge_wars_arrive_on_four_islands_facing_the_middle() {
        for seat in 0..ARENA_TEAMS {
            let (at, turn) = arrival(Venue::Arena, seat);
            let island = ARENA_ISLANDS[seat];
            assert!(at.xz().distance(island) < 1e-4, "seat {seat}: {at} for {island}");
            assert_eq!(at.y, 0.0);
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(-island.normalize()) < 1e-4, "seat {seat} faces {facing}");
            assert_eq!(arena_island(seat), seat);
        }
        // Past them, on the islands too, near the middle, each on the
        // island its seat says: the second of each team beside the first.
        for seat in ARENA_TEAMS..RING_SPOTS * 2 {
            let (at, _) = arrival(Venue::Arena, seat);
            let island = ARENA_ISLANDS[arena_island(seat)];
            let out = at.xz().distance(island);
            assert!(out > 3.0 && out < 8.0, "seat {seat}: {out} m from its island's middle");
            assert!(out < ARENA_ISLAND_RADIUS - 2.0, "seat {seat}: off the island");
        }
        // Eight seats, two to a team, a team to an island.
        assert_eq!(ARENA_SEATS, 2 * ARENA_TEAMS);
        for seat in 0..ARENA_SEATS {
            assert_eq!(arena_team(seat), arena_island(seat) as u8);
            assert_eq!(arena_team(seat), arena_team((seat + ARENA_TEAMS) % ARENA_SEATS));
            assert_ne!(arena_team(seat), arena_team((seat + 1) % ARENA_SEATS));
        }
        // Each team's anchor stands on its island, toward the town.
        for team in 0..ARENA_TEAMS {
            let (anchor, island) = (ARENA_ANCHORS[team], ARENA_ISLANDS[team]);
            let out = anchor.distance(island);
            assert!(out > 4.0 && out < 7.0, "team {team}: anchor {out} m from its island's middle");
            assert!(anchor.length() < island.length(), "team {team}: anchor beyond its island");
            assert!(bridge_wars::anchor_in_reach(anchor + Vec2::X * 2.5, anchor, bridge_wars::REACH));
            assert!(!bridge_wars::anchor_in_reach(anchor + Vec2::X * 4.0, anchor, bridge_wars::REACH));
        }
    }

    #[test]
    fn everyone_waits_in_the_room_facing_its_middle_and_apart() {
        let spots: Vec<(Vec3, Quat)> = (0..ARENA_SEATS).map(bridge_wars::waiting_spot).collect();
        for (seat, &(at, turn)) in spots.iter().enumerate() {
            assert_eq!(at.y, bridge_wars::WAITING_FLOOR);
            assert!((at.xz().length() - bridge_wars::WAIT_RING).abs() < 1e-4, "seat {seat} at {at}");
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(-at.xz().normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
        for (a, &(at, _)) in spots.iter().enumerate() {
            for &(other, _) in &spots[a + 1..] {
                assert!(at.distance(other) > 1.0, "{at} and {other}");
            }
        }
        let round = ArenaRound {
            stage: ArenaStage::Playing,
            left: 10,
            health: vec![100; ARENA_SEATS],
            anchors: [240; ARENA_TEAMS],
            out: 0b0000_0101,
        };
        assert!(round.is_out(0) && round.is_out(2) && !round.is_out(1) && !round.is_out(9));
    }

    #[test]
    fn in_the_town_everyone_arrives_on_the_path_round_the_pond_facing_in() {
        for seat in 0..RING_SPOTS * 2 {
            let (at, turn) = arrival(Venue::Town, seat);
            let out = at.xz().length();
            let ring = if seat < 16 { TOWN_ARRIVAL_RADIUS } else { TOWN_ARRIVAL_RADIUS + 2.0 };
            assert!((out - ring).abs() < 1e-3, "seat {seat}: {out} m out");
            assert_eq!(at.y, 0.0);
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(-at.xz().normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
    }

    #[test]
    fn a_full_town_arrives_apart() {
        for venue in [Venue::Town, Venue::Lobby, Venue::Arena] {
            let spots: Vec<Vec3> = (0..RING_SPOTS * 2).map(|seat| arrival(venue, seat).0).collect();
            for (i, a) in spots.iter().enumerate() {
                for b in &spots[i + 1..] {
                    assert!(a.distance(*b) > 2.0, "{venue:?}: {a} and {b}");
                }
            }
        }
    }

    #[test]
    fn the_most_stars_come_first() {
        let stars = [12, 30, 7, 25, 18, 3, 22, 9];
        for lot in 0..=u8::MAX {
            assert_eq!(standings(&stars, lot), [1, 3, 6, 4, 0, 7, 2, 5], "lot {lot}");
        }
    }

    #[test]
    fn a_tie_goes_either_way_by_lot() {
        let mut stars = [5; GAME_SEATS];
        stars[1] = 20;
        stars[6] = 20;
        let firsts: Vec<u8> = (0..=u8::MAX).map(|lot| standings(&stars, lot)[0]).collect();
        assert!(firsts.iter().all(|&first| first == 1 || first == 6));
        assert!(firsts.contains(&1) && firsts.contains(&6));
        // All eight tied: every one of them comes first by some lot, and last
        // by another.
        let level = [10; GAME_SEATS];
        for plot in 0..GAME_SEATS as u8 {
            assert!((0..=u8::MAX).any(|lot| standings(&level, lot)[0] == plot), "{plot} first");
            assert!((0..=u8::MAX).any(|lot| standings(&level, lot)[7] == plot), "{plot} last");
        }
    }

    #[test]
    fn every_plot_has_a_place() {
        for lot in 0..=u8::MAX {
            let mut order = standings(&[4, 4, 9, 0, 4, 9, 1, 4], lot);
            order.sort_unstable();
            assert_eq!(order, [0, 1, 2, 3, 4, 5, 6, 7], "lot {lot}");
        }
    }

    #[test]
    fn a_name_is_two_to_sixteen_plain_characters() {
        assert_eq!(profile::name("  Hajun  ").as_deref(), Some("Hajun"));
        assert_eq!(profile::name("Mochi 482").as_deref(), Some("Mochi 482"));
        assert_eq!(profile::name("ab").as_deref(), Some("ab"));
        assert_eq!(profile::name("sixteen chars ok").as_deref(), Some("sixteen chars ok"));
        assert_eq!(profile::name("a"), None);
        assert_eq!(profile::name(" a "), None);
        assert_eq!(profile::name("seventeen chars!!"), None);
        // Korean, a syllable as long as two letters: one to eight of them,
        // or fewer with letters beside them.
        assert_eq!(profile::name("하준").as_deref(), Some("하준"));
        assert_eq!(profile::name("민").as_deref(), Some("민"));
        assert_eq!(profile::name("동그라미타운주민").as_deref(), Some("동그라미타운주민"));
        assert_eq!(profile::name("동그라미타운주민들"), None);
        assert_eq!(profile::name("동그라미<3").as_deref(), Some("동그라미<3"));
        assert_eq!(profile::name("동그라미타운 abc").as_deref(), Some("동그라미타운 abc"));
        assert_eq!(profile::name("동그라미타운 abcd"), None);
        // Vietnamese, each letter one whatever marks it has, and the same
        // name however its letters were typed.
        assert_eq!(profile::name("Nguyễn Văn Hải").as_deref(), Some("Nguyễn Văn Hải"));
        assert_eq!(profile::name("Ha\u{309}i"), profile::name("Hải"));
        assert_eq!(profile::name("Nguyễn Thị Minh Hà"), None, "eighteen letters");
        // What the game does not write is left out of it, and a tab is a
        // space.
        assert_eq!(profile::name("Bob\u{1F600}").as_deref(), Some("Bob"));
        assert_eq!(profile::name("\u{1F600}\u{1F600}"), None);
        assert_eq!(profile::name("tab\there").as_deref(), Some("tab here"));
        assert!(profile::look("boy") && profile::look("girl"));
        assert!(!profile::look("blue") && !profile::look("red") && !profile::look(""));
    }

    #[test]
    fn a_spot_is_about_ten_bytes() {
        let spot = Spot::of(Vec3::new(-234.56, 1.2, 345.67), Quat::from_rotation_y(2.0));
        let bytes = postcard::to_allocvec(&spot).unwrap();
        assert!(bytes.len() <= 13, "{} bytes", bytes.len());
        // On the car, a place on it, which is never far from its middle.
        let on = Spot::of(Vec3::new(-1.2, 1.0, 2.5), Quat::from_rotation_y(2.0)).aboard(Some(0));
        let bytes = postcard::to_allocvec(&on).unwrap();
        assert!(bytes.len() <= 12, "{} bytes", bytes.len());
    }

    #[test]
    fn a_spot_on_the_car_is_not_the_same_place_off_it() {
        let off = Spot::of(Vec3::new(1.0, 1.0, 2.0), Quat::IDENTITY);
        let on = off.aboard(Some(0));
        assert!(off.same(&off) && on.same(&on));
        assert!(!off.same(&on));
        let back: Spot = postcard::from_bytes(&postcard::to_allocvec(&on).unwrap()).unwrap();
        assert!(back.same(&on), "{back:?}");
    }

    /// Dice that always come up the same: 0.5 is every wait at its
    /// middle and every thing in the middle of its pad.
    fn steady(value: f32) -> impl FnMut() -> f32 {
        move || value
    }

    /// Dice that go round a few values, for places apart on a pad.
    fn rolling() -> impl FnMut() -> f32 {
        let mut state = 0x2545_F491u32;
        move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state >> 8) as f32 / (1u32 << 24) as f32
        }
    }

    #[test]
    fn the_generators_make_what_the_file_says_as_fast_as_it_says_and_hold_no_more_than_it_says() {
        use bridge_wars::{DROP_SECS, GENERATORS, Mint, PAD_SPREAD, WAITING_FLOOR, cap, generator};
        let mut dice = steady(0.5);
        let mut mint = Mint::new(0.0, &mut dice);
        // A silver every 2 s and a gold every 7.5 s from each team's pad,
        // a sapphire every 30 s and an amethyst every 40 s: after 29 s,
        // 14 silver and 3 gold a pad, and neither of the others yet.
        let (dt, mut elapsed) = (1.0 / 60.0, 0.0);
        let mut made = Vec::new();
        while elapsed < 29.0 {
            elapsed += dt;
            made.extend(mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice));
        }
        let count = |made: &[bridge_wars::Piece], item: Item| made.iter().filter(|piece| piece.item == item).count();
        assert_eq!(count(&made, Item::Silver), 14 * 4);
        assert_eq!(count(&made, Item::Gold), 3 * 4);
        assert_eq!(count(&made, Item::Sapphire) + count(&made, Item::Amethyst), 0);
        // Each lies on its own generator's pad, well inside its edge, at
        // a number nobody else has.
        for piece in &made {
            let (item, pad) = generator(usize::from(piece.generator));
            assert_eq!(item, piece.item);
            assert!((piece.at - pad).abs().max_element() <= PAD_SPREAD + 1e-4, "{piece:?} off its pad at {pad}");
        }
        let mut ids: Vec<u32> = made.iter().map(|piece| piece.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), made.len());
        // Left alone for two minutes, every generator holds what it may
        // and no more: 48, 6, 4 and 2.
        while elapsed < 125.0 {
            elapsed += dt;
            mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice);
        }
        for index in 0..GENERATORS {
            let (item, _) = generator(index);
            assert_eq!(mint.held(index), cap(item), "generator {index}, {item:?}");
        }
        assert_eq!(mint.pieces().len(), 4 * (48 + 6 + 4 + 2));
        // Full, a generator makes nothing however long it is left; and
        // emptied, its next thing comes as its time next comes round,
        // within a wait, and not all at once.
        while elapsed < 140.0 {
            elapsed += dt;
            assert!(mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice).is_empty(), "something made on a full pad, {elapsed} s in");
        }
        let pad = ARENA_PADS[0];
        let taken = mint.pick_up(on(pad), elapsed);
        assert_eq!(taken.len(), 48 + 6, "everything on a team's pad is within reach of its middle");
        let mut waited = 0.0;
        let next = loop {
            waited += dt;
            let made = mint.run(elapsed + waited, &[0; ARENA_TEAMS], &mut dice);
            assert!(made.len() <= 1, "{} things at once on an emptied pad", made.len());
            if let Some(piece) = made.first() {
                break *piece;
            }
            assert!(waited < 2.1, "nothing after {waited} s");
        };
        assert_eq!(next.item, Item::Silver);
        // And that one, dropping from high over the pad, is nobody's
        // while it is out of reach over their head. It is theirs as it
        // comes down to them, before it has landed, and sooner for
        // someone at the top of a jump, who has it in the air.
        elapsed += waited;
        let standing = on(pad);
        let jumping = standing + Vec3::Y * 1.6;
        let had = |by: Vec3, after: f32| mint.clone().pick_up(by, elapsed + after);
        assert!(had(standing, 0.0).is_empty() && had(jumping, 0.0).is_empty());
        assert!(had(standing, 0.25).is_empty() && had(jumping, 0.25).is_empty());
        assert!(had(standing, 0.35).is_empty());
        assert_eq!(had(jumping, 0.35), vec![next]);
        assert!(had(standing, 0.42).is_empty());
        assert_eq!(had(standing, 0.47), vec![next]);
        assert!(0.47 < DROP_SECS, "it is still dropping then");
        // Landed, it is to be had at once, from the pad and from a jump
        // over it; and not from far under the island, nor from the
        // waiting room over the map.
        let landed = DROP_SECS + 0.01;
        assert_eq!(had(standing, landed), vec![next]);
        assert_eq!(had(jumping, landed), vec![next]);
        assert!(had(standing - Vec3::Y * 8.0, landed).is_empty());
        assert!(had(Vec3::new(pad.x, WAITING_FLOOR, pad.y), landed).is_empty());
    }

    /// Standing on the pad at `pad`: where someone's feet are there.
    fn on(pad: Vec2) -> Vec3 {
        Vec3::new(pad.x, PAD_TOP, pad.y)
    }

    #[test]
    fn a_sapphire_and_an_amethyst_generator_s_next_thing_is_the_time_s_alone() {
        use bridge_wars::{GENERATORS, Mint, PLAY_SECS, generator, next_at, steady as to_the_second};
        assert!(to_the_second(Item::Sapphire) && to_the_second(Item::Amethyst));
        assert!(!to_the_second(Item::Silver) && !to_the_second(Item::Gold));
        // Every time one comes round, up to `until`.
        let times = |item: Item, until: f32| {
            let (mut at, mut times) = (0.0, Vec::new());
            loop {
                at = next_at(item, at);
                if at > until {
                    return times;
                }
                times.push(at);
            }
        };
        // A sapphire every 30 s to two minutes, every 23 s from then, and
        // from six minutes, when the wait in hand had 13 s to go, every
        // 12 s.
        let sapphire = times(Item::Sapphire, 400.0);
        assert_eq!(sapphire[..6], [30.0, 60.0, 90.0, 120.0, 143.0, 166.0]);
        assert_eq!(sapphire[sapphire.len() - 4..], [350.0, 372.0, 384.0, 396.0]);
        // An amethyst every 60 s to four minutes, every 40 s to eight,
        // and every 25 s after.
        let amethyst = times(Item::Amethyst, 530.0);
        assert_eq!(amethyst[..3], [60.0, 120.0, 180.0]);
        assert_eq!(amethyst[3..6], [240.0, 280.0, 320.0]);
        assert_eq!(amethyst[amethyst.len() - 4..], [440.0, 480.0, 505.0, 530.0]);
        // What is written over a generator: the next is always ahead,
        // never further than a wait, and at the moment one comes it is
        // the one after.
        assert_eq!(next_at(Item::Sapphire, 0.0), 30.0);
        assert_eq!(next_at(Item::Sapphire, 119.0), 120.0);
        assert_eq!(next_at(Item::Sapphire, 120.0), 143.0);
        assert_eq!(next_at(Item::Sapphire, 360.0), 372.0);
        assert_eq!(next_at(Item::Amethyst, 239.5), 240.0);
        assert_eq!(next_at(Item::Amethyst, 240.0), 280.0);
        assert_eq!(next_at(Item::Amethyst, 480.0), 505.0);
        for item in [Item::Sapphire, Item::Amethyst] {
            let mut at = 0.0;
            while at < PLAY_SECS {
                let next = next_at(item, at);
                assert!(next > at && next - at <= 60.0, "{item:?}: {next} after {at}");
                at += 0.37;
            }
        }
        // The generators keep those times, to the frame, taken from or
        // not: one emptied as each thing comes, and one left to fill.
        let mut dice = steady(0.5);
        let mut mint = Mint::new(0.0, &mut dice);
        let (dt, mut elapsed) = (1.0 / 30.0, 0.0);
        let mut made_at = Vec::new();
        let mut left_alone = 0;
        while elapsed < 400.0 {
            elapsed += dt;
            for piece in mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice) {
                if piece.generator == 8 {
                    made_at.push(elapsed);
                }
                left_alone += usize::from(piece.generator == 9);
            }
            mint.pick_up(on(SAPPHIRE_PADS[0]), elapsed);
        }
        assert_eq!(made_at.len(), sapphire.len(), "{made_at:?}");
        for (made, due) in made_at.iter().zip(&sapphire) {
            assert!(*made >= *due && made - due < 2.0 * dt, "made {made} s in, due at {due}");
        }
        assert_eq!(left_alone, 4, "a full generator makes nothing as its time comes round");
        // A fight begun part way through its timetable keeps them too.
        let mut late = Mint::new(595.0, &mut dice);
        let mut elapsed = 595.0;
        let mut first = [None; GENERATORS];
        while elapsed < 625.0 {
            elapsed += dt;
            for piece in late.run(elapsed, &[0; ARENA_TEAMS], &mut dice) {
                first[usize::from(piece.generator)].get_or_insert(elapsed);
            }
        }
        for (index, first) in first.iter().enumerate() {
            let (item, _) = generator(index);
            let first = first.unwrap_or_else(|| panic!("generator {index} made nothing in 30 s"));
            if to_the_second(item) {
                let due = next_at(item, 595.0);
                assert!(first >= due && first - due < 2.0 * dt, "{item:?}: first {first} s in, due at {due}");
            }
        }
    }

    #[test]
    fn the_generators_speed_up_as_the_timetable_says() {
        use bridge_wars::{TIMETABLE, Happening, Mint, PLAY_SECS, happening_between, interval, tier};
        assert_eq!(tier(Item::Sapphire, 0.0), 1);
        assert_eq!(tier(Item::Sapphire, 119.9), 1);
        assert_eq!(tier(Item::Sapphire, 120.0), 2);
        assert_eq!(tier(Item::Amethyst, 239.0), 1);
        assert_eq!(tier(Item::Amethyst, 240.0), 2);
        assert_eq!(tier(Item::Sapphire, 360.0), 3);
        assert_eq!(tier(Item::Amethyst, 480.0), 3);
        assert_eq!(tier(Item::Sapphire, PLAY_SECS), 3);
        assert_eq!(tier(Item::Silver, PLAY_SECS), 1);
        assert_eq!(tier(Item::Gold, PLAY_SECS), 1);
        assert_eq!(interval(Item::Sapphire, 1).0, 30.0);
        assert_eq!(interval(Item::Sapphire, 2).0, 23.0);
        assert_eq!(interval(Item::Sapphire, 3).0, 12.0);
        assert_eq!(interval(Item::Amethyst, 1).0, 60.0);
        assert_eq!(interval(Item::Amethyst, 2).0, 40.0);
        assert_eq!(interval(Item::Amethyst, 3).0, 25.0);
        // In order, each once, all before the draw; and an event is this
        // frame's only in the frame its time passes.
        assert!(TIMETABLE.windows(2).all(|pair| pair[0].0 < pair[1].0));
        assert!(TIMETABLE.iter().all(|(at, _)| *at > 0.0 && *at < PLAY_SECS));
        assert_eq!(
            happening_between(119.99, 120.01).collect::<Vec<_>>(),
            vec![Happening::Tier { item: Item::Sapphire, tier: 2 }]
        );
        assert_eq!(happening_between(120.01, 120.03).count(), 0);
        assert_eq!(happening_between(0.0, PLAY_SECS).count(), TIMETABLE.len());
        // The anchors at 14:00, the zombies from 15:00, the word that
        // the end is coming at 19:30, and the end at 20:00, as Hajun's
        // file has them since the night of 2026-11-16.
        assert_eq!(happening_between(839.0, 841.0).collect::<Vec<_>>(), vec![Happening::AnchorsBreak]);
        assert_eq!(happening_between(599.0, 601.0).count(), 0);
        assert_eq!(happening_between(899.0, 901.0).collect::<Vec<_>>(), vec![Happening::SuddenDeath]);
        assert_eq!(happening_between(1169.0, 1171.0).collect::<Vec<_>>(), vec![Happening::DrawSoon]);
        assert_eq!(PLAY_SECS, 1200.0);
        // The zombies: none before 15:00, two as it comes, twice as many
        // every 36 s, never fewer than there were, and fifty at the most,
        // well before the end.
        use bridge_wars::{ZOMBIES, zombies_at};
        assert_eq!((zombies_at(0.0), zombies_at(899.9), zombies_at(900.0), zombies_at(920.0)), (0, 0, 2, 2));
        assert_eq!((zombies_at(922.0), zombies_at(936.0), zombies_at(972.0), zombies_at(1008.0), zombies_at(1044.0)), (3, 4, 8, 16, 32));
        assert_eq!((zombies_at(1068.0), zombies_at(PLAY_SECS), zombies_at(f32::MAX)), (ZOMBIES, ZOMBIES, ZOMBIES));
        let mut risen = 0;
        for second in 0..=PLAY_SECS as u32 {
            let now = zombies_at(second as f32);
            assert!(now >= risen && now <= ZOMBIES);
            risen = now;
        }
        // Someone standing on a sapphire pad the whole game, taking each
        // as it comes: 4 in the first two minutes, one every 23 s for the
        // next four, and one every 12 s after.
        let mut dice = steady(0.5);
        let mut mint = Mint::new(0.0, &mut dice);
        let (dt, mut elapsed) = (1.0 / 30.0, 0.0);
        let mut taken = [0usize; 3];
        while elapsed < 600.0 {
            elapsed += dt;
            mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice);
            let got = mint.pick_up(on(SAPPHIRE_PADS[0]), elapsed);
            assert!(got.iter().all(|piece| piece.item == Item::Sapphire));
            // Each is to be had about half a second after it is made, as
            // it comes down.
            let stretch = if elapsed <= 121.5 {
                0
            } else if elapsed <= 361.5 {
                1
            } else {
                2
            };
            taken[stretch] += got.len();
        }
        assert_eq!(taken, [4, 10, 19]);
    }

    #[test]
    fn a_thing_is_picked_up_from_near_it_once_and_things_come_up_apart() {
        use bridge_wars::{MAGNET, Mint, generator};
        let mut dice = rolling();
        let mut mint = Mint::new(0.0, &mut dice);
        let (dt, mut elapsed) = (0.05, 0.0);
        while elapsed < 20.0 {
            elapsed += dt;
            mint.run(elapsed, &[0; ARENA_TEAMS], &mut dice);
        }
        // A second on, the last of them has dropped.
        let elapsed = elapsed + 1.0;
        // Ten silver or so and two gold on each team's pad by now, no two
        // of a pad's in one spot.
        let pad = ARENA_PADS[2];
        let lying: Vec<Vec2> = mint
            .pieces()
            .iter()
            .filter(|piece| generator(usize::from(piece.generator)).1 == pad)
            .map(|piece| piece.at)
            .collect();
        assert!(lying.len() >= 10, "{} things", lying.len());
        for (k, a) in lying.iter().enumerate() {
            for b in &lying[k + 1..] {
                assert!(a.distance(*b) > 0.02, "two things at {a}");
            }
        }
        // From 3 m off the pad nothing is in reach; from its middle all
        // of it is, once: a second go, or someone else's, finds nothing.
        let all = mint.pieces().len();
        assert!(mint.pick_up(on(pad + Vec2::new(MAGNET + 1.0 + 0.5, 0.0)), elapsed).is_empty());
        assert_eq!(mint.pieces().len(), all);
        let taken = mint.pick_up(on(pad), elapsed);
        assert_eq!(taken.len(), lying.len());
        assert!(mint.pick_up(on(pad), elapsed).is_empty());
        assert_eq!(mint.pieces().len(), all - taken.len());
        // Another team's pad is untouched by it.
        assert!(!mint.pick_up(on(ARENA_PADS[0]), elapsed).is_empty());
    }

    #[test]
    fn the_shops_sell_what_the_file_says_for_what_it_says() {
        use bridge_wars::{Shop, buy, price, sold_at};
        let (bare, none) = (Kit::default(), Upgrades::default());
        // The item shop: the bridge, three swords, three shields and six
        // things for the buttons.
        for (ware, cost) in [
            (Ware::Utility(Utility::Dynamite), (Item::Amethyst, 1)),
            (Ware::Bridge, (Item::Silver, 4)),
            (Ware::Sword(1), (Item::Silver, 10)),
            (Ware::Sword(2), (Item::Gold, 7)),
            (Ware::Sword(3), (Item::Amethyst, 4)),
            (Ware::Shield(1), (Item::Silver, 24)),
            (Ware::Shield(2), (Item::Gold, 12)),
            (Ware::Shield(3), (Item::Amethyst, 6)),
            (Ware::Utility(Utility::Fireball), (Item::Silver, 40)),
            (Ware::Utility(Utility::Builder), (Item::Amethyst, 1)),
            (Ware::Utility(Utility::Speed), (Item::Amethyst, 1)),
            (Ware::Utility(Utility::Invisibility), (Item::Amethyst, 2)),
            (Ware::Utility(Utility::Healing), (Item::Gold, 3)),
        ] {
            assert_eq!(price(ware, &bare, &none), Some(cost), "{ware:?}");
            assert_eq!(sold_at(ware), Shop::Items);
        }
        assert_eq!(price(Ware::Sword(4), &bare, &none), None);
        assert_eq!(price(Ware::Shield(0), &bare, &none), None);
        // Only a better sword or shield than the one in hand; any tier
        // from none.
        let silver = Kit { sword: 2, shield: 1, ..bare };
        assert_eq!(price(Ware::Sword(1), &silver, &none), None);
        assert_eq!(price(Ware::Sword(2), &silver, &none), None);
        assert!(price(Ware::Sword(3), &silver, &none).is_some());
        assert_eq!(price(Ware::Shield(1), &silver, &none), None);
        assert!(price(Ware::Shield(2), &silver, &none).is_some() && price(Ware::Shield(3), &silver, &none).is_some());
        // A button holds one kind of thing: three fireballs at the most,
        // and nine of anything else. Another kind is to be had beside
        // it, on a button of its own.
        let fire = bare.holding(Utility::Fireball, 2);
        assert!(price(Ware::Utility(Utility::Fireball), &fire, &none).is_some());
        assert!(price(Ware::Utility(Utility::Speed), &fire, &none).is_some());
        let full = fire.holding(Utility::Fireball, 3);
        assert_eq!(price(Ware::Utility(Utility::Fireball), &full, &none), None);
        assert!(price(Ware::Utility(Utility::Speed), &full, &none).is_some());
        let potions = bare.holding(Utility::Healing, 8);
        assert!(price(Ware::Utility(Utility::Healing), &potions, &none).is_some());
        assert_eq!(price(Ware::Utility(Utility::Healing), &potions.holding(Utility::Healing, 9), &none), None);
        // And three sticks of dynamite ("can stack up to 3 times max at
        // once").
        let sticks = bare.holding(Utility::Dynamite, 2);
        assert!(price(Ware::Utility(Utility::Dynamite), &sticks, &none).is_some());
        assert_eq!(price(Ware::Utility(Utility::Dynamite), &sticks.holding(Utility::Dynamite, 3), &none), None);
        assert_eq!(bridge_wars::most_held(Utility::Dynamite), 3);
        // What is aimed and what is drunk; and of what is drunk, what
        // lasts.
        for utility in Utility::ALL {
            assert_ne!(utility.aimed(), utility.drunk(), "{utility:?}");
            assert!(!utility.lasts() || utility.drunk());
        }
        assert!(Utility::Fireball.aimed() && Utility::Builder.aimed() && Utility::Dynamite.aimed() && Utility::Healing.drunk());
        // The healing potion gives half of one's health back ("instantly
        // heal 50% of your health").
        assert_eq!(bridge_wars::HEALING, 0.5);
        assert!(Utility::Speed.lasts() && Utility::Invisibility.lasts() && !Utility::Healing.lasts());
        // The upgrade shop: each step dearer than the last, to its last.
        let steps = |ware: Ware, raise: &dyn Fn(&mut Upgrades)| {
            let (mut team, mut costs) = (Upgrades::default(), Vec::new());
            while let Some((item, cost)) = price(ware, &bare, &team) {
                assert_eq!((item, sold_at(ware)), (Item::Sapphire, Shop::Upgrades));
                costs.push(cost);
                raise(&mut team);
            }
            costs
        };
        assert_eq!(steps(Ware::Forge, &|team| team.forge += 1), [2, 4, 6, 8]);
        assert_eq!(steps(Ware::BetterShields, &|team| team.shields += 1), [2, 3, 4, 6]);
        assert_eq!(steps(Ware::Trap, &|team| team.traps += 1), [1, 2, 4]);
        assert_eq!(steps(Ware::StrongerSwords, &|team| team.swords = true), [4]);
        assert_eq!(steps(Ware::HealPool, &|team| team.heal_pool = true), [1]);
        // Buying takes the price and gives the thing; without the price,
        // nothing changes.
        let (mut purse, mut kit, mut team): (Purse, Kit, Upgrades) = ([9, 7, 3, 0], bare, none);
        assert!(!buy(Ware::Sword(1), &mut purse, &mut kit, &mut team), "9 silver is not 10");
        assert_eq!((purse, kit), ([9, 7, 3, 0], bare));
        assert!(buy(Ware::Sword(2), &mut purse, &mut kit, &mut team));
        assert_eq!((purse, kit.sword), ([9, 0, 3, 0], 2));
        assert!(!buy(Ware::Sword(2), &mut purse, &mut kit, &mut team));
        assert!(buy(Ware::Forge, &mut purse, &mut kit, &mut team) && buy(Ware::Trap, &mut purse, &mut kit, &mut team));
        assert_eq!((purse, team.forge, team.traps), ([9, 0, 0, 0], 1, 1));
        purse[0] = 100;
        assert!(buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team));
        assert!(buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team));
        assert_eq!((purse[0], kit.slots[0]), (20, Some(Stack { utility: Utility::Fireball, count: 2 })));
        assert_eq!(kit.held(Utility::Fireball), 2);
    }

    #[test]
    fn the_buttons_fill_in_the_order_things_are_bought_one_kind_to_each() {
        use bridge_wars::{FAST_SECS, Outfits, buy, most_held, price};
        let none = Upgrades::default();
        let stack = |utility, count| Some(Stack { utility, count });
        // Five kinds bought in turn take the five buttons in that order,
        // and more of a kind goes onto the button it is on.
        let order = [Utility::Healing, Utility::Fireball, Utility::Speed, Utility::Builder, Utility::Invisibility];
        let (mut purse, mut kit, mut team): (Purse, Kit, Upgrades) = ([200; 4], Kit::default(), none);
        for (slot, utility) in order.into_iter().enumerate() {
            assert_eq!((kit.free_slot(), kit.slot_of(utility), kit.held(utility)), (Some(slot), None, 0));
            assert!(buy(Ware::Utility(utility), &mut purse, &mut kit, &mut team));
            assert_eq!((kit.slot_of(utility), kit.held(utility)), (Some(slot), 1));
        }
        assert_eq!(kit.free_slot(), None);
        // There are seven kinds and five buttons: with five kinds held,
        // another is not to be had until a button is empty.
        assert_eq!(Utility::ALL.len(), UTILITY_SLOTS + 2);
        assert_eq!(price(Ware::Utility(Utility::Dynamite), &kit, &none), None);
        assert_eq!(price(Ware::Utility(Utility::Leap), &kit, &none), None);
        let held = purse;
        assert!(!buy(Ware::Utility(Utility::Dynamite), &mut purse, &mut kit, &mut team));
        assert_eq!(purse, held, "and nothing is paid for it");
        assert!(buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team));
        assert!(buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team));
        assert!(!buy(Ware::Utility(Utility::Fireball), &mut purse, &mut kit, &mut team), "three to a button");
        assert_eq!(kit.slots[1], stack(Utility::Fireball, most_held(Utility::Fireball)));
        assert_eq!(kit.slots.map(|slot| slot.map(|stack| stack.utility)), order.map(Some));
        assert_eq!(kit.potion(), Some(Utility::Healing), "the first potion, button by button");
        // With every button taken by another kind, a kind they have none
        // of is not to be had; with one free, it is, and goes there.
        let crowded = Kit { slots: [stack(Utility::Fireball, 1); UTILITY_SLOTS], ..Kit::default() };
        assert_eq!(price(Ware::Utility(Utility::Healing), &crowded, &none), None);
        assert_eq!(crowded.holding(Utility::Healing, 2), crowded);

        // Used up, a button is empty, the rest staying where they are; and
        // the next kind bought goes onto the first empty one.
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        outfits.kits[0] = kit;
        assert_eq!(outfits.drink(0, Utility::Healing, 10.0), Some(Utility::Healing));
        assert_eq!(outfits.drink(0, Utility::Healing, 10.0), None, "there was only one");
        assert!(outfits.spend(0, Utility::Builder) && !outfits.spend(0, Utility::Builder));
        outfits.tick(0.1, 10.1, &[]);
        let after = outfits.kits[0];
        assert_eq!(after.slots[0], None);
        assert_eq!((after.slots[1], after.slots[2]), (stack(Utility::Fireball, 3), stack(Utility::Speed, 1)));
        assert_eq!((after.slots[3], after.slots[4]), (None, stack(Utility::Invisibility, 1)));
        let mut purse: Purse = [200; 4];
        assert!(outfits.buy(0, Ware::Utility(Utility::Builder), &mut purse));
        assert!(outfits.buy(0, Ware::Utility(Utility::Healing), &mut purse));
        assert_eq!((outfits.kits[0].slot_of(Utility::Builder), outfits.kits[0].slot_of(Utility::Healing)), (Some(0), Some(3)));
        // Both potions that last, drunk: each keeps its own button, with
        // none left on it, for as long as it lasts and no longer.
        assert_eq!(outfits.drink(0, Utility::Speed, 20.0), Some(Utility::Speed));
        assert_eq!(outfits.drink(0, Utility::Invisibility, 20.0), Some(Utility::Invisibility));
        outfits.tick(0.1, 21.0, &[]);
        let kit = outfits.kits[0];
        assert_eq!((kit.slots[2], kit.slots[4]), (stack(Utility::Speed, 0), stack(Utility::Invisibility, 0)));
        assert!(kit.fast(21.0) && kit.unseen(21.0) && kit.potion() == Some(Utility::Healing));
        // Seen again by a hit, the invisibility's button is empty and the
        // speed's is not; and that one is once the speed has run out.
        assert!(outfits.reveal(0));
        outfits.tick(0.1, 22.0, &[]);
        assert_eq!((outfits.kits[0].slots[2], outfits.kits[0].slots[4]), (stack(Utility::Speed, 0), None));
        outfits.tick(0.1, 20.0 + FAST_SECS + 0.1, &[]);
        assert_eq!(outfits.kits[0].slots[2], None);
        // Dying empties every button.
        outfits.die(0);
        assert_eq!(outfits.kits[0], Kit::default());
        // And a kit is sent button by button.
        let sent = postcard::to_allocvec(&Kit::default().holding(Utility::Speed, 2)).unwrap();
        assert_eq!(postcard::from_bytes::<Kit>(&sent).unwrap().slots[0], stack(Utility::Speed, 2));
    }

    #[test]
    fn a_sword_hits_harder_and_a_shield_keeps_some_of_it_off() {
        use bridge_wars::{DAMAGE, Outfits, kept_off, punch_damage};
        let (bare, none) = (Kit::default(), Upgrades::default());
        // A bare hand's 12, and 5, 10 and 20 more by the sword.
        assert_eq!(punch_damage(DAMAGE, &bare, &none), 12.0);
        for (tier, more) in [(1, 5.0), (2, 10.0), (3, 20.0)] {
            assert_eq!(punch_damage(DAMAGE, &Kit { sword: tier, ..bare }, &none), 12.0 + more);
        }
        // Stronger Swords is five more on a sword, and nothing on a hand.
        let strong = Upgrades { swords: true, ..none };
        assert_eq!(punch_damage(DAMAGE, &Kit { sword: 3, ..bare }, &strong), 37.0);
        assert_eq!(punch_damage(DAMAGE, &bare, &strong), 12.0);
        // 6%, 14% and 22% kept off by the shield, and 6% more for each of
        // the team's four Better Shields; nothing with no shield.
        for (tier, share) in [(1, 0.06), (2, 0.14), (3, 0.22)] {
            assert!((kept_off(&Kit { shield: tier, ..bare }, &none) - share).abs() < 1e-6);
        }
        let better = Upgrades { shields: 4, ..none };
        assert!((kept_off(&Kit { shield: 3, ..bare }, &better) - 0.46).abs() < 1e-6);
        assert_eq!(kept_off(&bare, &better), 0.0);
        // In a game: seat 0, with the aura sword on a team with stronger
        // swords, hits seat 1, with the basic shield.
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        outfits.kits[0].sword = 3;
        outfits.kits[1].shield = 1;
        outfits.teams[0].swords = true;
        assert!((outfits.punch(DAMAGE, 0, 1) - 37.0 * 0.94).abs() < 1e-4);
        assert_eq!(outfits.punch(DAMAGE, 1, 0), 12.0);
        // A zombie, in a seat past the players', has nothing.
        assert_eq!(outfits.punch(DAMAGE, ARENA_SEATS, 2), 12.0);
        assert!((outfits.through(1, 20.0) - 18.8).abs() < 1e-4);
        // Dying loses the sword and what was on the buttons, and keeps
        // the shield.
        outfits.kits[1] = Kit {
            sword: 2,
            shield: 1,
            fast_until: 90,
            unseen_until: 0,
            blind: true,
            compass: true,
            ..Kit::default()
        }
        .holding(Utility::Speed, 2)
        .holding(Utility::Fireball, 1);
        outfits.die(1);
        assert_eq!(outfits.kits[1], Kit { shield: 1, ..Kit::default() });
    }

    #[test]
    fn the_compass_is_bought_once_a_life_at_the_item_shop_and_is_not_sent() {
        use bridge_wars::{Outfits, Shop, price, sold_at};
        let none = Upgrades::default();
        // 2 amethyst at the red stand, and nothing on the second button.
        assert_eq!(sold_at(Ware::Compass), Shop::Items);
        assert_eq!(price(Ware::Compass, &Kit::default(), &none), Some((Item::Amethyst, 2)));
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        let mut purse: Purse = [0, 0, 0, 5];
        assert!(!outfits.buy(0, Ware::Compass, &mut [0, 0, 0, 1]), "one amethyst is not its price");
        assert!(outfits.buy(0, Ware::Compass, &mut purse));
        assert_eq!(purse, [0, 0, 0, 3]);
        let kit = outfits.kit(0);
        assert!(kit.compass && kit.slots == [None; UTILITY_SLOTS]);
        // Once a life: not again while it is theirs.
        assert_eq!(price(Ware::Compass, &kit, &none), None);
        assert!(!outfits.buy(0, Ware::Compass, &mut purse));
        // A potion goes on the second button beside it all the same.
        assert!(outfits.buy(0, Ware::Utility(Utility::Speed), &mut purse));
        // Lost on dying, and to be bought again.
        outfits.die(0);
        assert!(!outfits.kit(0).compass);
        assert!(outfits.buy(0, Ware::Compass, &mut [0, 0, 0, 2]));
        // A kit goes over the network as it did before there was one: the
        // same bytes with it and without, and read back without it.
        let with = Kit { sword: 2, compass: true, ..Kit::default() };
        let without = Kit { sword: 2, ..Kit::default() };
        let (sent, plain) = (postcard::to_allocvec(&with).unwrap(), postcard::to_allocvec(&without).unwrap());
        assert_eq!(sent, plain);
        assert_eq!(postcard::from_bytes::<Kit>(&sent).unwrap(), without);
        // And the wares before it are numbered as they were.
        assert_eq!(postcard::to_allocvec(&Ware::Trap).unwrap(), vec![8]);
        assert_eq!(postcard::to_allocvec(&Ware::Compass).unwrap(), vec![9]);
    }

    #[test]
    fn a_team_s_generator_is_as_fast_as_the_team_has_made_it() {
        use bridge_wars::{FORGE_AMETHYST, GENERATORS, Mint, generator};
        // Three minutes of every generator, team 0's raised to each step:
        // its silver half as fast again, twice, and three times as fast as
        // team 1's, which is left as it was; and from the third step
        // amethyst on its own pad too.
        let made = |forge: u8| {
            let mut dice = steady(0.5);
            let mut mint = Mint::new(0.0, &mut dice);
            let (dt, mut elapsed) = (1.0 / 30.0, 0.0);
            let mut counts = [0usize; 3];
            while elapsed < 180.0 {
                elapsed += dt;
                for piece in mint.run(elapsed, &[forge, 0, 0, 0], &mut dice) {
                    match usize::from(piece.generator) {
                        0 => counts[0] += 1,
                        1 => counts[1] += 1,
                        index if index == GENERATORS => {
                            assert_eq!((piece.item, generator(index).1), (Item::Amethyst, ARENA_PADS[0]));
                            counts[2] += 1;
                        }
                        _ => {}
                    }
                }
                // Everything taken as it comes, so that nothing fills.
                mint.pick_up(on(ARENA_PADS[0]), elapsed);
                mint.pick_up(on(ARENA_PADS[1]), elapsed);
            }
            counts
        };
        assert_eq!(made(0), [90, 90, 0]);
        let better = made(1);
        assert!((133..=136).contains(&better[0]) && better[1] == 90 && better[2] == 0, "{better:?}");
        let advanced = made(2);
        assert!((178..=180).contains(&advanced[0]) && advanced[2] == 0, "{advanced:?}");
        // An amethyst a minute from when it was raised: at 60, 120 and 180.
        let amethyst = made(FORGE_AMETHYST);
        assert!((178..=180).contains(&amethyst[0]) && (2..=3).contains(&amethyst[2]), "{amethyst:?}");
        let molten = made(4);
        assert!((266..=270).contains(&molten[0]) && (2..=3).contains(&molten[2]), "{molten:?}");
    }

    #[test]
    fn a_fireball_flies_straight_to_the_end_of_its_range_or_whoever_it_meets() {
        use bridge_wars::{BLAST_RADIUS, FIREBALL_AHEAD, FIREBALL_HEIGHT, FIREBALL_RANGE, FIREBALL_SPEED, Flight, in_blast};
        // Thrown north from the middle of an island: it sets out in front
        // of the thrower, at their chest, and flies north, level.
        let feet = Vec3::new(0.0, 0.25, 130.0);
        let mut ball = Flight::thrown(feet, 0.0);
        assert!(ball.at().distance(feet + Vec3::new(0.0, FIREBALL_HEIGHT, -FIREBALL_AHEAD)) < 1e-5);
        assert!(ball.way().distance(Vec3::NEG_Z) < 1e-6);
        // Heading a quarter turn to the left of north is west, as a plank
        // heads.
        assert!(Flight::thrown(feet, std::f32::consts::FRAC_PI_2).way().distance(Vec3::NEG_X) < 1e-6);
        // Someone 8 m north is met on the way; someone 8 m north and 2 m
        // to the side is not, nor someone far under it.
        let (ahead, beside) = (feet + Vec3::new(0.0, 0.0, -8.0), feet + Vec3::new(2.0, 0.0, -8.0));
        let (dt, mut met, mut seconds) = (1.0 / 60.0, None, 0.0);
        while !ball.fly(dt) {
            seconds += dt;
            assert!(!ball.meets(beside) && !ball.meets(ahead - Vec3::Y * 6.0));
            if met.is_none() && ball.meets(ahead) {
                met = Some(ball.flown);
            }
        }
        let met = met.expect("it passed through someone in its way");
        assert!(met > 6.0 && met < 7.5, "met {met} m out");
        // Left to fly, it stops at the end of its range, in the time that
        // takes.
        assert_eq!(ball.flown, FIREBALL_RANGE);
        assert!((seconds - FIREBALL_RANGE / FIREBALL_SPEED).abs() < 0.05);
        assert!(ball.at().distance(feet + Vec3::new(0.0, FIREBALL_HEIGHT, -FIREBALL_AHEAD - FIREBALL_RANGE)) < 1e-3);
        // Its blast reaches whoever is within three metres of it across
        // the ground, at about its height, and nobody further.
        let burst = ball.at();
        let under = burst - Vec3::Y * FIREBALL_HEIGHT;
        assert!(in_blast(burst, under) && in_blast(burst, under + Vec3::X * BLAST_RADIUS));
        assert!(!in_blast(burst, under + Vec3::X * (BLAST_RADIUS + 0.6)));
        assert!(!in_blast(burst, under - Vec3::Y * 8.0));
    }

    #[test]
    fn a_stick_of_dynamite_is_lobbed_in_an_arch_to_where_it_was_aimed_and_blows_after_its_fuse() {
        use bridge_wars::{
            DYNAMITE_AHEAD, DYNAMITE_FLASHES, DYNAMITE_FURTHEST, DYNAMITE_FUSE, DYNAMITE_HAND, DYNAMITE_LEAN, DYNAMITE_NEAREST,
            DYNAMITE_RADIUS, DYNAMITE_TAP, Lob, UP_THE_MAP, in_dynamite, lob_track,
        };
        // Thrown north from an island, 8 m: out of the hand, in front of
        // the thrower and over their feet, to come down 8 m north of them
        // on the ground it was thrown at.
        let feet = Vec3::new(3.0, 0.25, 130.0);
        let mut lob = Lob::thrown(feet, 0.0, 8.0, 0.24);
        assert!(lob.from.distance(feet + Vec3::new(0.0, DYNAMITE_HAND, -DYNAMITE_AHEAD)) < 1e-5);
        assert!(lob.to.distance(Vec3::new(3.0, 0.24, 122.0)) < 1e-4);
        assert_eq!(lob.at(), lob.from);
        // A throw is no nearer and no further than one can be.
        let across = |lob: Lob| Vec2::new(lob.to.x - feet.x, lob.to.z - feet.z).length();
        assert!((across(Lob::thrown(feet, 1.0, 0.3, 0.25)) - DYNAMITE_NEAREST).abs() < 1e-4);
        assert!((across(Lob::thrown(feet, 1.0, 99.0, 0.25)) - DYNAMITE_FURTHEST).abs() < 1e-4);
        assert!(DYNAMITE_NEAREST < DYNAMITE_TAP && DYNAMITE_TAP < DYNAMITE_FURTHEST);
        // Further takes longer, and goes higher; none of them long.
        let (near, far) = (Lob::thrown(feet, 0.0, DYNAMITE_NEAREST, 0.25), Lob::thrown(feet, 0.0, DYNAMITE_FURTHEST, 0.25));
        assert!(near.secs() < far.secs() && near.secs() > 0.4 && far.secs() < 1.0, "{} and {}", near.secs(), far.secs());
        assert!(near.apex() < far.apex() && far.apex() > 1.5 && far.apex() < 3.0, "{} and {}", near.apex(), far.apex());
        // On its way it rises and comes down again. Thrown up the map,
        // north, it is drawn along the straight line of its throw, to
        // neither side of it, and always on northward.
        let frame = 1.0 / 60.0;
        let (mut highest, mut flown) = (lob.from.y, 0.0);
        let mut last = lob.at();
        while !lob.down() {
            lob.tick(frame);
            flown += frame;
            let at = lob.at();
            assert!(at.z <= last.z + 1e-4, "it flies on north");
            assert!((at.x - feet.x).abs() < 1e-4, "to neither side of its throw: {}", at.x - feet.x);
            highest = highest.max(at.y);
            assert!(lob.down() || lob.shine() == 0.0, "it does not shine in the air");
            last = at;
        }
        assert!((flown - lob.secs()).abs() <= frame + 1e-4);
        assert!((highest - (DYNAMITE_HAND + 0.25) - lob.apex()).abs() < 0.6, "it rose to {highest}");
        assert!(lob.at().distance(lob.to) < 1e-4, "and it lies where it was aimed");
        assert_eq!((lob.rise(0.0), lob.rise(1.0)), (0.0, 0.0));
        assert!((lob.rise(0.5) - lob.apex()).abs() < 1e-5);
        // Thrown to the east or the west it is drawn as an arch is on a
        // map: up it as it rises and back down it as it falls, by as much
        // as it has risen, the same whichever of the two ways it goes, and
        // never down the map from the line of its throw. The band that
        // aims one is drawn along the same way.
        for heading in [std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2] {
            let side = Lob::thrown(feet, heading, 8.0, 0.25);
            let (from, to) = (Vec2::new(side.from.x, side.from.z), Vec2::new(side.to.x, side.to.z));
            assert!((side.to.x - feet.x).abs() > 7.9 && (side.to.z - feet.z).abs() < 1e-4);
            assert_eq!(side.track(0.0), from);
            assert!(side.track(1.0).distance(to) < 1e-5);
            let top = side.track(0.5);
            assert!(top.distance((from + to) * 0.5 + UP_THE_MAP * DYNAMITE_LEAN * side.apex()) < 1e-4, "{top:?}");
            assert!(top.y < feet.z - 1.0, "its arch is seen: {top:?}");
            for step in 0..=20 {
                let share = step as f32 / 20.0;
                let at = side.track(share);
                assert!(at.y <= feet.z + 1e-4, "never down the map from its throw");
                assert!((at.x - (from.x + (to.x - from.x) * share)).abs() < 1e-4, "nor ahead of itself or behind");
                assert_eq!(at, lob_track(from, to, side.apex(), UP_THE_MAP, share));
            }
        }
        // Thrown down the map, south, it is on the line of its throw too.
        let south = Lob::thrown(feet, std::f32::consts::PI, 8.0, 0.25);
        assert!((0..=20).all(|step| (south.track(step as f32 / 20.0).x - feet.x).abs() < 1e-4));
        // Down, it lies there for its fuse, shining three times, each a
        // swell and a fade; and then it blows.
        let (mut lain, mut peaks, mut rising, mut before) = (0.0, 0, false, 0.0);
        while !lob.blown() {
            assert!(lob.at().distance(lob.to) < 1e-4);
            let shine = lob.shine();
            assert!((0.0..=1.0).contains(&shine));
            if rising && shine < before {
                peaks += 1;
            }
            rising = shine > before;
            before = shine;
            lob.tick(frame);
            lain += frame;
        }
        assert!((lain - DYNAMITE_FUSE).abs() <= 2.0 * frame, "{lain} s");
        assert_eq!(peaks, DYNAMITE_FLASHES, "it shone {peaks} times");
        assert_eq!(lob.fuse_left(), 0.0);
        // Its blast reaches whoever stands within its reach of it, across
        // the ground, on about its level; and nobody further off, or far
        // under it.
        assert!(in_dynamite(lob.to, lob.to) && in_dynamite(lob.to, lob.to + Vec3::X * DYNAMITE_RADIUS));
        assert!(!in_dynamite(lob.to, lob.to + Vec3::X * (DYNAMITE_RADIUS + 0.6)));
        assert!(!in_dynamite(lob.to, lob.to - Vec3::Y * 8.0));
        // What the room is sent of one, and of its blast, is small.
        let thrown = Dynamite {
            seat: 3,
            x: lob.from.x,
            y: lob.from.y,
            z: lob.from.z,
            to_x: lob.to.x,
            to_y: lob.to.y,
            to_z: lob.to.z,
        };
        assert!(postcard::to_allocvec(&thrown).unwrap().len() <= 25);
        let blast = Blasted {
            x: lob.to.x,
            y: lob.to.y,
            z: lob.to.z,
            planks: vec![(3, 1, 4), (3, 1, 5), (3, 1, 6), (3, 1, 7), (3, 1, 8)],
            at: 1_794_000_000.25,
            later: Vec::new(),
        };
        let sent = postcard::to_allocvec(&blast).unwrap();
        assert!(sent.len() <= 41, "{} bytes", sent.len());
        assert_eq!(postcard::from_bytes::<Blasted>(&sent).unwrap(), blast);
        // With the rest of a bridge from an island to the middle falling
        // after them, thirty planks at a quarter of a second apart: six
        // bytes a plank at the most, and each time as it was to the
        // thousandth of a second.
        let cascade: Vec<((u8, u16, u16), f32)> = (0..30u16).map(|plank| ((3, 1, 9 + plank), 0.6 + 0.25 * f32::from(plank))).collect();
        let long = Blasted {
            later: Blasted::later_of(&cascade),
            ..blast.clone()
        };
        let sent = postcard::to_allocvec(&long).unwrap();
        assert!(sent.len() <= 41 + 30 * 6, "{} bytes", sent.len());
        let back = postcard::from_bytes::<Blasted>(&sent).unwrap();
        assert_eq!(back, long);
        for (was, is) in cascade.iter().zip(back.later()) {
            assert!(was.0 == is.0 && (was.1 - is.1).abs() < 1e-3, "{was:?} {is:?}");
        }
        // A time past what can be sent is the longest that can.
        assert_eq!(Blasted::later_of(&[((0, 0, 0), 500.0)]), vec![(0, 0, 0, u16::MAX)]);
        let used = postcard::to_allocvec(&Use::Dynamite { heading: 1.0, far: 6.0 }).unwrap();
        assert_eq!(postcard::from_bytes::<Use>(&used).unwrap(), Use::Dynamite { heading: 1.0, far: 6.0 });
    }

    #[test]
    fn potions_wear_off_traps_blind_and_pools_heal() {
        use bridge_wars::{BLIND_SECS, FAST_SECS, HEAL_POOL_EVERY, Outfits, UNSEEN_SECS};
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        // A speed potion: nothing until it is drunk; drunk, 45 s of being
        // fast, the button kept for as long, and empty after.
        let mut purse: Purse = [0, 3, 0, 6];
        assert!(outfits.buy(0, Ware::Utility(Utility::Speed), &mut purse));
        assert!(!outfits.spend(0, Utility::Fireball) && !outfits.kits[0].fast(10.0));
        assert_eq!(outfits.drink(0, Utility::Healing, 10.0), None, "not the potion they have");
        assert_eq!(outfits.drink(0, Utility::Speed, 10.0), Some(Utility::Speed));
        assert_eq!(outfits.drink(0, Utility::Speed, 10.0), None, "there was only one");
        assert!(outfits.kits[0].fast(10.0) && outfits.kits[0].fast(10.0 + FAST_SECS - 0.5));
        outfits.tick(0.1, 30.0, &[]);
        assert_eq!(outfits.kits[0].slots[0], Some(Stack { utility: Utility::Speed, count: 0 }));
        outfits.tick(0.1, 10.0 + FAST_SECS + 0.1, &[]);
        assert_eq!(outfits.kits[0], Kit::default());
        // The invisibility lasts its 30 s, or until they hit or are hit;
        // the healing is at once, and the button empty the moment the
        // last is gone.
        assert!(outfits.buy(0, Ware::Utility(Utility::Invisibility), &mut purse));
        assert_eq!(outfits.drink(0, Utility::Invisibility, 100.0), Some(Utility::Invisibility));
        assert!(outfits.kits[0].unseen(100.0 + UNSEEN_SECS - 0.5) && !outfits.kits[0].unseen(100.0 + UNSEEN_SECS));
        outfits.tick(0.1, 200.0, &[]);
        assert_eq!(outfits.kits[0], Kit::default());
        assert!(outfits.buy(0, Ware::Utility(Utility::Invisibility), &mut purse));
        assert_eq!(outfits.drink(0, Utility::Invisibility, 150.0), Some(Utility::Invisibility));
        assert!(outfits.kits[0].unseen(151.0) && !outfits.reveal(1), "seat 1 was in plain sight");
        assert!(outfits.reveal(0) && !outfits.kits[0].unseen(151.0) && !outfits.reveal(0));
        outfits.tick(0.1, 151.0, &[]);
        assert_eq!(outfits.kits[0], Kit::default(), "and the button is empty again");
        assert!(outfits.buy(0, Ware::Utility(Utility::Healing), &mut purse));
        assert_eq!(outfits.drink(0, Utility::Healing, 200.0), Some(Utility::Healing));
        outfits.tick(0.1, 200.1, &[]);
        assert_eq!(outfits.kits[0].slots, [None; UTILITY_SLOTS]);
        // A fireball is not drunk.
        outfits.kits[0] = Kit::default().holding(Utility::Fireball, 1);
        assert_eq!(outfits.drink(0, Utility::Fireball, 200.0), None);
        assert!(outfits.spend(0, Utility::Fireball) && !outfits.spend(0, Utility::Fireball));

        // Team 1 has two traps waiting and a heal pool. Seat 0, of team 0,
        // steps onto its island: blinded, one trap spent; its own seat 1
        // there is not, nor a zombie; and seat 2, coming after, takes the
        // second.
        outfits.teams[1] = Upgrades { traps: 2, heal_pool: true, ..Upgrades::default() };
        let (theirs, mine) = (ARENA_ISLANDS[1], ARENA_ISLANDS[0]);
        let ticked = outfits.tick(0.1, 300.0, &[(0, theirs), (1, theirs + Vec2::X), (ARENA_SEATS, theirs)]);
        assert_eq!(ticked.sprung, vec![(0, 1)]);
        assert!(outfits.kits[0].blind && !outfits.kits[1].blind && outfits.teams[1].traps == 1);
        let ticked = outfits.tick(0.1, 300.1, &[(0, theirs), (2, theirs)]);
        assert_eq!(ticked.sprung, vec![(2, 1)]);
        assert_eq!(outfits.teams[1].traps, 0);
        // With none left, seat 3 walks on unharmed.
        assert!(outfits.tick(0.1, 300.2, &[(3, theirs)]).sprung.is_empty() && !outfits.kits[3].blind);
        // Blind for eight seconds, wherever they go: seat 0 for 7.8 more
        // off the island, and then no longer.
        let off = theirs + Vec2::new(40.0, 0.0);
        for _ in 0..77 {
            outfits.tick(0.1, 301.0, &[(0, off)]);
        }
        assert!(outfits.kits[0].blind, "7.9 seconds in");
        for _ in 0..2 {
            outfits.tick(0.1, 308.0, &[(0, off)]);
        }
        assert!(!outfits.kits[0].blind, "{BLIND_SECS} seconds in");
        // Dying, seat 2 sees again at once.
        outfits.die(2);
        assert!(!outfits.kits[2].blind);
        // A trap springs on someone stepping onto the island, not on
        // someone who has stood on it since the last one: seat 0 stays on
        // team 1's island through a trap bought under their feet, and
        // springs it only once they have stepped off and on again.
        outfits.teams[1].traps = 1;
        outfits.tick(0.1, 320.0, &[(0, theirs)]);
        assert!(outfits.kits[0].blind && outfits.teams[1].traps == 0);
        for _ in 0..90 {
            outfits.tick(0.1, 321.0, &[(0, theirs)]);
        }
        assert!(!outfits.kits[0].blind);
        outfits.teams[1].traps = 1;
        assert!(outfits.tick(0.1, 330.0, &[(0, theirs)]).sprung.is_empty(), "still standing on it");
        outfits.tick(0.1, 330.1, &[(0, off)]);
        assert_eq!(outfits.tick(0.1, 330.2, &[(0, theirs)]).sprung, vec![(0, 1)]);
        assert!(outfits.kits[0].blind && outfits.teams[1].traps == 0);
        // The pool heals whoever of team 1 is on its island every two and
        // a half seconds: seat 1 there, not seat 5 away from it, nor seat
        // 0 of another team standing on it.
        let mut outfits = Outfits::default();
        outfits.seat(ARENA_SEATS);
        outfits.teams[1].heal_pool = true;
        let mut healed = Vec::new();
        for _ in 0..((HEAL_POOL_EVERY * 2.0 / 0.05) as usize + 2) {
            healed.extend(outfits.tick(0.05, 0.0, &[(1, theirs), (5, mine), (0, theirs)]).healed);
        }
        assert_eq!(healed, vec![1, 1]);
    }

    #[test]
    fn the_pads_the_shops_and_the_zombies_are_where_the_map_has_room_for_them() {
        use bridge_wars::{
            BRIDGE_A_BUY, SHOP_SLACK, Shop, TOWN_RADIUS, ZOMBIE_TEAM, ZOMBIES, at_a_shop, price, zombie_home,
            zombie_spot,
        };
        for team in 0..ARENA_TEAMS {
            let island = ARENA_ISLANDS[team];
            // Each team's pad is on its island, behind its middle, the
            // anchor being before it; and its shop beside its middle.
            let pad = ARENA_PADS[team];
            assert!(pad.distance(island) + PAD_HALF * 1.5 < ARENA_ISLAND_RADIUS, "team {team}'s pad");
            assert!(pad.length() > island.length() && ARENA_ANCHORS[team].length() < island.length());
            let shop = ARENA_SHOPS[team];
            assert!((shop.distance(island) - 5.0).abs() < 0.1, "team {team}'s shop");
            // Bought from at the shop and a few steps off it, and not
            // from the pad, nor from the anchor.
            assert!(at_a_shop(shop, Shop::Items) && at_a_shop(shop + Vec2::new(SHOP_SLACK - 0.1, 0.0), Shop::Items));
            assert!(!at_a_shop(pad, Shop::Items) && !at_a_shop(ARENA_ANCHORS[team], Shop::Items), "team {team}");
            // Its upgrade shop is across the island's middle from it, as
            // far out, and at either nobody is at the other.
            let upgrades = ARENA_UPGRADE_SHOPS[team];
            assert!((upgrades.distance(island) - 5.0).abs() < 0.1 && (upgrades.distance(shop) - 10.0).abs() < 0.1);
            assert!(at_a_shop(upgrades, Shop::Upgrades) && !at_a_shop(upgrades, Shop::Items));
            assert!(!at_a_shop(shop, Shop::Upgrades) && !at_a_shop(pad, Shop::Upgrades));
            // Its amethyst pad is the one toward it, in the middle.
            assert!(AMETHYST_PADS[team].normalize().dot(island.normalize()) > 0.99);
            assert!(AMETHYST_PADS[team].length() < 40.0);
        }
        for pad in SAPPHIRE_PADS {
            assert!((pad.length() - 125.0).abs() < 0.1, "{pad}");
            assert!(ARENA_ISLANDS.iter().all(|island| island.distance(pad) > ARENA_ISLAND_RADIUS + SAPPHIRE_ISLAND_RADIUS));
        }
        assert_eq!(price(Ware::Bridge, &Kit::default(), &Upgrades::default()), Some((Item::Silver, 4)));
        assert_eq!(BRIDGE_A_BUY, 25.0);
        // The seats past the players' are the zombies', on nobody's team
        // but their own; each comes up inside the ground it keeps to,
        // with room round it, and no two in one place.
        for seat in 0..ARENA_SEATS {
            assert!(arena_team(seat) < ZOMBIE_TEAM);
        }
        assert_eq!(arena_team(ARENA_SEATS), ZOMBIE_TEAM);
        assert_eq!(arena_team(ARENA_SEATS + ZOMBIES - 1), ZOMBIE_TEAM);
        let spots: Vec<Vec2> = (0..ZOMBIES).map(zombie_spot).collect();
        for (index, spot) in spots.iter().enumerate() {
            let (home, reach) = zombie_home(index);
            assert!(spot.distance(home) < reach - 2.0, "zombie {index} at {spot}, home {home} reaching {reach}");
            assert!(spots.iter().filter(|other| other.distance(*spot) < 3.0).count() == 1, "zombie {index}");
        }
        // All over the map: of every twelve, four in the middle, on the
        // path round the pond and clear of the amethyst pads, four on the
        // smaller islands and four on the teams'; and the first of them
        // in the middle, where they are nobody's own trouble.
        for index in 0..ZOMBIES {
            let (home, reach) = zombie_home(index);
            match index % 12 / 4 {
                0 => {
                    assert_eq!((home, reach), (Vec2::ZERO, TOWN_RADIUS), "zombie {index}");
                    assert!(AMETHYST_PADS.iter().all(|pad| pad.distance(spots[index]) > 10.0), "zombie {index}");
                }
                1 => assert!(SAPPHIRE_PADS.contains(&home) && reach == SAPPHIRE_ISLAND_RADIUS, "zombie {index}"),
                _ => assert!(ARENA_ISLANDS.contains(&home) && reach == ARENA_ISLAND_RADIUS, "zombie {index}"),
            }
        }
        // No team's island has the first of every round of them.
        let firsts: Vec<Vec2> = (0..4).map(|round| zombie_home(round * 12 + 8).0).collect();
        assert!(ARENA_ISLANDS.iter().all(|island| firsts.contains(island)), "{firsts:?}");
    }

    #[test]
    fn a_thing_on_a_pad_is_a_few_bytes_and_a_purse_is_four_counts() {
        let loot = Loot {
            item: Item::Sapphire,
            x: -88.39,
            z: 88.39,
            spilt: None,
        };
        let bytes = postcard::to_allocvec(&loot).unwrap();
        assert!(bytes.len() <= 10, "{} bytes", bytes.len());
        let back: Loot = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, loot);
        // One that someone who died dropped says from where, onto what
        // and how many it is, in a few bytes more.
        let dropped = Loot {
            spilt: Some(Spilt {
                ground: 0.25,
                from_x: -87.0,
                from_y: 1.25,
                from_z: 87.5,
                worth: 3,
            }),
            ..loot
        };
        let bytes = postcard::to_allocvec(&dropped).unwrap();
        assert!(bytes.len() <= 28, "{} bytes", bytes.len());
        assert_eq!(postcard::from_bytes::<Loot>(&bytes).unwrap(), dropped);
        let mut purse: Purse = [0; 4];
        purse[Item::Silver.index()] = 7;
        purse[Item::Amethyst.index()] = 1;
        assert_eq!(purse, [7, 0, 0, 1]);
        assert_eq!(Item::ALL.map(Item::index), [0, 1, 2, 3]);
        // What a killer gets of a purse is its sapphire and its amethyst,
        // all of each, and what is left to go with the dead is the silver
        // and the gold.
        let mut held: Purse = [30, 20, 7, 5];
        let won = bridge_wars::spoils(&mut held);
        assert_eq!((won, held), ([0, 0, 7, 5], [30, 20, 0, 0]));
        assert_eq!(bridge_wars::spoils(&mut held), Purse::default(), "nothing twice");
        let mut killer: Purse = [1, 2, 3, u16::MAX - 2];
        bridge_wars::gain(&mut killer, won);
        assert_eq!(killer, [1, 2, 10, u16::MAX]);
        // Someone of another team, of the eight: not a teammate (seats 0
        // and 4 are a team), not themselves, and no zombie either way.
        assert!(bridge_wars::takes_spoils(0, 1) && bridge_wars::takes_spoils(7, 4));
        assert!(!bridge_wars::takes_spoils(0, 4) && !bridge_wars::takes_spoils(3, 3));
        assert!(!bridge_wars::takes_spoils(ARENA_SEATS, 1) && !bridge_wars::takes_spoils(1, ARENA_SEATS + 2));
    }

    #[test]
    fn a_bridge_s_falling_sets_out_slower_than_a_run_and_ends_a_little_faster_than_one() {
        use bridge_wars::{CASCADE_LONGEST, CASCADE_QUICKENS, CASCADE_SPEED, CASCADE_TOP, CASCADE_WAIT, RUN_SPEED, build_pace, cascade_after};
        // The bridge from an island to the middle, on Hajun's map.
        let span = nav::Nav::arena().grid().span(nav::Region::Team(0), nav::Region::Middle, 0.0).expect("a span");
        let (planks, each) = (span.planks(), span.length() / span.planks() as f32);
        assert!((26..=32).contains(&planks) && (each - nav::PLANK_SPACING).abs() < 0.06, "{planks} planks of {each}");
        // A beat, and then the first plank as soon after it as it went
        // while the falling had one pace.
        assert_eq!(cascade_after(0.0), CASCADE_WAIT);
        assert!((cascade_after(each) - (CASCADE_WAIT + each / CASCADE_SPEED)).abs() < 0.02);
        // Each plank sooner after the one before it than that one was
        // after the one before, down to what its fastest makes it, and
        // never sooner than that.
        let gaps: Vec<f32> = (1..120)
            .map(|plank| cascade_after(plank as f32 * each) - cascade_after((plank - 1) as f32 * each))
            .collect();
        assert!(gaps.windows(2).all(|pair| pair[1] <= pair[0] + 1e-4), "{gaps:?}");
        assert!((gaps[0] - each / CASCADE_SPEED).abs() < 0.02 && (gaps[gaps.len() - 1] - each / CASCADE_TOP).abs() < 1e-3);
        // Slower than a run as it sets out, and in the end faster than
        // one, by a little ("accelerate slightly (but not too much)").
        assert!(CASCADE_SPEED < RUN_SPEED && CASCADE_TOP > RUN_SPEED && CASCADE_TOP <= RUN_SPEED * 1.25);
        assert!(CASCADE_QUICKENS > 0.0 && CASCADE_QUICKENS <= 3.0);
        // How long whoever stands on the middle of the plank `stood`
        // planks from the hole may take to set off, at `pace`, and still
        // be off a bridge whose shore is `shore` planks from the hole:
        // off every plank on their way before it goes. Less than
        // nothing, and they do not get there.
        let spare = |stood: usize, shore: usize, pace: f32| {
            (stood..=shore)
                .map(|plank| cascade_after(plank as f32 * each) - (plank as f32 + 0.5 - stood as f32) * each / pace)
                .fold(f32::INFINITY, f32::min)
        };
        // Blown at one end, which takes four planks or five: from right
        // beside the hole there is three quarters of a second to set off
        // in ("its rare to fall off a bridge if you react right away and u
        // start running toward safety"), which is what the plank they
        // stand on gave them before; and they are at the far shore with
        // the falling under a second behind them ("u need to run full
        // speed to barely or even fail to escape").
        let shore = planks - 5;
        let beside = spare(1, shore, RUN_SPEED);
        assert!(beside > 0.7 && beside < 0.8, "{beside} s beside the hole");
        let ashore = cascade_after(shore as f32 * each) - (shore as f32 - 0.5) * each / RUN_SPEED;
        assert!(ashore > 0.7 && ashore < 1.0, "{ashore} s behind them at the shore");
        // Further along it there is more, and it is never less for being
        // further from the hole: a second and more from the third plank
        // on, where there were three and more.
        let along: Vec<f32> = (1..shore).map(|stood| spare(stood, shore, RUN_SPEED)).collect();
        assert!(along.windows(2).all(|pair| pair[1] >= pair[0] - 1e-4), "{along:?}");
        assert!(along[1] > 0.85 && along[2] > 1.0 && along[shore / 2] > 2.0, "{along:?}");
        // Carrying a plank, at a builder's pace, nobody beside the hole
        // gets there: it takes a run.
        assert!(spare(1, shore, RUN_SPEED * build_pace(&Upgrades::default())) < 0.0);
        // And a very long bridge ("even if u react right away and u run
        // full speed and ur rllly far from land then ok u will still fail
        // to reach land"): beside the hole with 60 m to the shore, half a
        // second is too long to take; with 95, no time at all is short
        // enough.
        let planks_in = |metres: f32| (metres / each).round() as usize;
        let long = spare(1, planks_in(60.0), RUN_SPEED);
        assert!(long > 0.3 && long < 0.5, "{long}");
        assert!(spare(1, planks_in(95.0), RUN_SPEED) < 0.0);
        // Half way along a bridge as long as that, there is still time.
        assert!(spare(planks_in(47.0), planks_in(95.0), RUN_SPEED) > 1.0);
        // However far, no later than a time can be sent.
        assert_eq!(cascade_after(1.0e6), CASCADE_LONGEST);
        assert!(CASCADE_LONGEST * 1000.0 < f32::from(u16::MAX));
    }

    #[test]
    fn someone_who_dies_on_land_drops_their_sapphire_and_amethyst_where_there_is_ground_for_them() {
        use bridge_wars::{
            HOVER, MAGNET, Mint, Piece, SPILL_FAR, SPILL_FROM, SPILL_LIES, SPILL_MOST, SPILL_NEAR, SPILL_WAIT, SPILT, TOSS_RISE, TOSS_SECS, toss,
        };
        let mut dice = rolling();
        let mut mint = Mint::new(0.0, &mut steady(0.5));
        let across = |piece: &Piece, feet: Vec3| piece.at.distance(feet.xz());
        // On open ground, a quarter of a metre up: one thing for each
        // sapphire and each amethyst they held, thrown out round them,
        // and their silver and gold left to go with them.
        let flat = |_: Vec2| Some(0.25);
        let feet = Vec3::new(10.0, 0.25, -4.0);
        let mut held: Purse = [30, 20, 7, 3];
        let dropped = mint.spill(&mut held, feet, 100.0, &mut dice, &flat).expect("there is ground");
        assert_eq!(held, [30, 20, 0, 0]);
        let of = |item: Item| dropped.iter().filter(|piece| piece.item == item).count();
        assert_eq!((dropped.len(), of(Item::Sapphire), of(Item::Amethyst)), (10, 7, 3));
        for piece in &dropped {
            assert!(piece.dropped() && piece.generator == SPILT && piece.worth == 1 && piece.ground == 0.25, "{piece:?}");
            assert_eq!(piece.from, Some(feet + Vec3::Y * SPILL_FROM));
            let far = across(piece, feet);
            assert!((SPILL_NEAR - 1e-3..=SPILL_FAR + 1e-3).contains(&far), "{far} m from them");
        }
        // Round them, and not in a heap: on every side, and no two in
        // one place.
        let sides = |side: fn(Vec2) -> bool| dropped.iter().any(|piece| side(piece.at - Vec2::new(10.0, -4.0)));
        assert!(sides(|off| off.x > 0.3) && sides(|off| off.x < -0.3) && sides(|off| off.y > 0.3) && sides(|off| off.y < -0.3));
        for (index, piece) in dropped.iter().enumerate() {
            assert!(dropped[..index].iter().all(|other| other.at.distance(piece.at) > 0.25), "{dropped:?}");
        }
        // They lie with what the generators have made, and are none of
        // any generator's.
        assert_eq!(mint.pieces().len(), 10);
        assert!((0..bridge_wars::GENERATORS + ARENA_TEAMS).all(|index| mint.held(index) == 0));
        // Each flies out from them and comes down where it lies, and
        // lies there a moment: nobody has it on the way nor as it comes
        // down, standing right there, so that it is seen to have been
        // dropped; whoever walks into it after that moment does, and
        // nobody twice.
        let piece = dropped[0];
        assert_eq!(toss(piece.from.unwrap(), piece.rest(), 0.0), feet + Vec3::Y * SPILL_FROM);
        assert_eq!(toss(piece.from.unwrap(), piece.rest(), TOSS_SECS), piece.rest());
        assert!(toss(piece.from.unwrap(), piece.rest(), TOSS_SECS * 0.5).y > feet.y + SPILL_FROM * 0.5 + TOSS_RISE * 0.9);
        assert!((piece.height(100.0 + TOSS_SECS) - (0.25 + HOVER)).abs() < 1e-5 && !piece.lying(100.0 + TOSS_SECS + 0.02));
        assert!(SPILL_LIES >= 0.2 && SPILL_LIES <= 0.5 && SPILL_WAIT == TOSS_SECS + SPILL_LIES && SPILL_WAIT < 1.0);
        let on = |piece: &Piece| Vec3::new(piece.at.x, 0.25, piece.at.y);
        assert!(mint.clone().pick_up(on(&piece), 100.0 + TOSS_SECS - 0.02).is_empty(), "not in the air");
        assert!(mint.clone().pick_up(on(&piece), 100.0 + SPILL_WAIT - 0.02).is_empty(), "nor the moment it is down");
        assert!(!mint.clone().pick_up(on(&piece), 100.0 + SPILL_WAIT).is_empty() && piece.lying(100.0 + SPILL_WAIT));
        let mut walked = mint.clone();
        let mut got = [0u16; 4];
        for piece in &dropped {
            for taken in walked.pick_up(on(piece), 100.0 + SPILL_WAIT) {
                assert!(taken.at.distance(piece.at) <= MAGNET + 1e-3);
                got[taken.item.index()] += taken.worth;
            }
        }
        assert_eq!((got, walked.pieces().len()), ([0, 0, 7, 3], 0));
        // Standing where they died, only what came down within reach.
        let near = mint.clone().pick_up(feet, 101.0);
        assert!(near.len() < dropped.len() && near.iter().all(|taken| across(taken, feet) <= MAGNET + 1e-3));
        assert_eq!(near.len(), dropped.iter().filter(|piece| across(piece, feet) <= MAGNET).count());
        // More than ten of one are ten things, the count shared out among
        // them.
        let mut held: Purse = [0, 0, 23, 1];
        let many = Mint::new(0.0, &mut steady(0.5)).spill(&mut held, feet, 5.0, &mut dice, &flat).expect("ground");
        let worths: Vec<u16> = many.iter().filter(|piece| piece.item == Item::Sapphire).map(|piece| piece.worth).collect();
        assert_eq!((many.len(), worths.len(), worths.iter().sum::<u16>()), (11, usize::from(SPILL_MOST), 23));
        assert!(worths.iter().all(|worth| (2..=3).contains(worth)), "{worths:?}");
        // Only onto ground. On a bridge, a plank's width of it: nothing
        // comes down off its sides.
        let bridge = |at: Vec2| (at.y.abs() <= 1.0).then_some(0.24);
        let mut held: Purse = [0, 0, 9, 9];
        let on_bridge = Mint::new(0.0, &mut steady(0.5))
            .spill(&mut held, Vec3::new(40.0, 0.24, 0.3), 5.0, &mut dice, &bridge)
            .expect("the bridge");
        assert_eq!(on_bridge.len(), 18);
        assert!(on_bridge.iter().all(|piece| piece.at.y.abs() <= 1.0 && piece.ground == 0.24), "{on_bridge:?}");
        // At the very edge of an island: on the island.
        let shore = |at: Vec2| (at.x <= 0.0).then_some(0.25);
        let mut held: Purse = [0, 0, 10, 10];
        let ashore = Mint::new(0.0, &mut steady(0.5))
            .spill(&mut held, Vec3::new(-0.2, 0.25, 7.0), 5.0, &mut dice, &shore)
            .expect("the island");
        assert!(ashore.len() == 20 && ashore.iter().all(|piece| piece.at.x <= 0.0), "{ashore:?}");
        // Beside a stand: not onto its roof, which is no step up.
        let stand = |at: Vec2| Some(if at.x > 1.0 { 3.0 } else { 0.25 });
        let mut held: Purse = [0, 0, 10, 0];
        let by_stand = Mint::new(0.0, &mut steady(0.5))
            .spill(&mut held, Vec3::new(0.5, 0.25, 0.0), 5.0, &mut dice, &stand)
            .expect("the ground");
        assert!(by_stand.iter().all(|piece| piece.at.x <= 1.0 && piece.ground == 0.25), "{by_stand:?}");
        // Killed at the top of a jump, they drop onto the ground under
        // them all the same.
        let mut held: Purse = [0, 0, 2, 0];
        let jumping = Mint::new(0.0, &mut steady(0.5)).spill(&mut held, feet + Vec3::Y * 1.6, 5.0, &mut dice, &flat);
        assert!(jumping.is_some_and(|dropped| dropped.len() == 2 && dropped[0].ground == 0.25));
        // Out over the void there is nowhere for any of it to lie:
        // nothing is dropped, and they hold what they held, to go as it
        // does with a fall. Nor from under an island, falling past it.
        let mut held: Purse = [4, 4, 4, 4];
        let mut empty = Mint::new(0.0, &mut steady(0.5));
        assert_eq!(empty.spill(&mut held, feet, 5.0, &mut dice, &|_| None), None);
        assert_eq!(empty.spill(&mut held, feet - Vec3::Y * 3.0, 5.0, &mut dice, &flat), None);
        assert_eq!((held, empty.pieces().len()), ([4, 4, 4, 4], 0));
        // Holding neither, on land: nothing dropped, and nothing to hand
        // anyone.
        let mut held: Purse = [9, 9, 0, 0];
        assert_eq!(empty.spill(&mut held, feet, 5.0, &mut dice, &flat), Some(Vec::new()));
        assert_eq!(held, [9, 9, 0, 0]);
        // What lies on a plank goes with the plank; what lies on land
        // stays, and so does whatever a generator made, wherever.
        let mut both = Mint::new(0.0, &mut steady(0.5));
        both.run(1000.0, &[0; ARENA_TEAMS], &mut steady(0.5));
        let made = both.pieces().len();
        assert!(made > 0);
        let mut held: Purse = [0, 0, 6, 0];
        let lying = both.spill(&mut held, Vec3::new(40.0, 0.24, 0.0), 1000.0, &mut dice, &bridge).expect("the bridge");
        assert_eq!(both.shed(&|_| true), Vec::new());
        let gone = both.shed(&|at| at.x < 40.0);
        assert_eq!(gone.len(), lying.iter().filter(|piece| piece.at.x >= 40.0).count());
        assert!(!gone.is_empty() && gone.len() < lying.len() && gone.iter().all(Piece::dropped));
        assert_eq!(both.shed(&|_| false).len(), lying.len() - gone.len());
        assert_eq!(both.pieces().len(), made);
    }

    #[test]
    fn four_buses_go_round_the_four_lanes_the_outer_half_one_way_and_the_inner_the_other() {
        use bridge_wars::{BUS_HALF_LENGTH, BUS_HALF_WIDTH, BUS_SPEED, BUSES, Bus, Buses, LANES, Leg};
        // "i want 4 bus, one in each lane (there are 4 lanes)": the two
        // lanes of each half of the ring, either side of its line (50 and
        // 65 m out), each bus wholly on its half (44 to 56, and 59 to 71).
        assert_eq!(BUSES, 4);
        for (number, lane) in LANES.iter().enumerate() {
            let (line, (inner, outer)) = if lane.out > 57.5 { (65.0, (59.0, 71.0)) } else { (50.0, (44.0, 56.0)) };
            assert!(((lane.out - line).abs() - 3.0).abs() < 0.25, "lane {number}, {} m out", lane.out);
            assert!(lane.out - BUS_HALF_WIDTH > inner + 0.5 && lane.out + BUS_HALF_WIDTH < outer - 0.5, "lane {number}");
            // Keeping right: the outer half anticlockwise seen from above,
            // the inner clockwise.
            assert_eq!(lane.anticlockwise, lane.out > 57.5, "lane {number}");
        }
        // "2x normal speed": twice the 7 the town's buses went at then,
        // whatever the town's go at since.
        assert!((BUS_SPEED - 14.0).abs() < 0.3, "{BUS_SPEED} m/s");
        // The town's own: three quarters of that 7.
        assert_eq!(cars::TOP_SPEED, 7.0 * 0.75);
        // Years after 1970, to the last of a second: always on its lane,
        // facing along it, the kerb on its right, going at full speed the
        // way its lane goes.
        let clock = 1.79e9 + 0.123;
        let buses = Buses(std::array::from_fn(|number| Leg {
            from: clock - 7.0,
            round: 100.0 * number as f64,
            speed: BUS_SPEED,
            gathers: 0.0,
        }));
        for (number, lane) in LANES.iter().enumerate() {
            let mut round = 0.0;
            for step in 0..3000 {
                let bus = buses.at(number, clock + f64::from(step) * 0.01);
                assert!((bus.at.length() - lane.out).abs() < 1e-3 && (bus.way.length() - 1.0).abs() < 1e-5);
                assert!(bus.way.dot(bus.at).abs() < 1e-3, "along the lane");
                let kerb = if lane.anticlockwise { 1.0 } else { -1.0 };
                assert!(bus.right().dot(bus.at) * kerb > 0.0, "keeping right, lane {number}");
                let next = buses.at(number, clock + f64::from(step + 1) * 0.01);
                let went = next.at - bus.at;
                assert!((went.length() / 0.01 - BUS_SPEED).abs() < 0.05, "{} m/s", went.length() / 0.01);
                assert!(went.dot(bus.way) > 0.0);
                round += bus.way.perp_dot(next.way);
            }
            // Anticlockwise from above, its way turns toward -Z from +X.
            assert_eq!(round < 0.0, lane.anticlockwise, "lane {number}");
        }
        // From east, north going anticlockwise and south the other way.
        let east = Bus::on(&LANES[0], 0.0);
        assert!(east.at.distance(Vec2::new(LANES[0].out, 0.0)) < 1e-4 && east.way.distance(Vec2::NEG_Y) < 1e-5);
        let inner = Bus::on(&LANES[3], 0.0);
        assert!(inner.at.distance(Vec2::new(LANES[3].out, 0.0)) < 1e-4 && inner.way.distance(Vec2::Y) < 1e-5);
        // The same place a lap on, however far it has come, and half a
        // lap on the far side of the ring.
        for lane in &LANES {
            let length = f64::from(lane.length());
            let (now, later, half) = (Bus::on(lane, 37.0), Bus::on(lane, 37.0 + 40.0 * length), Bus::on(lane, 37.0 + length / 2.0));
            assert!(now.at.distance(later.at) < 0.01 && (now.at + half.at).length() < 0.01);
        }
        // Its own space and back; its model faces the game's -Z.
        let now = buses.at(1, clock);
        let point = Vec2::new(40.0, -31.0);
        assert!(now.world(now.local(point)).distance(point) < 1e-3);
        assert!(now.local(now.at + now.way * 3.0 + now.right() * 1.5).distance(Vec2::new(1.5, 3.0)) < 1e-4);
        let (at, turn) = east.stands();
        assert!(at.distance(Vec3::new(LANES[0].out, 0.0, 0.0)) < 1e-4);
        assert!((turn * Vec3::NEG_Z).distance(Vec3::NEG_Z) < 1e-5 && (turn * Vec3::X).distance(Vec3::X) < 1e-5);
        // Someone standing 2 m back from its windscreen, inside, a metre
        // to its right: where they are in the world.
        let rider = now.carries(Vec3::new(1.0, 0.47, -4.0));
        assert!(now.local(Vec2::new(rider.x, rider.z)).distance(Vec2::new(1.0, 4.0)) < 1e-3 && (rider.y - 0.47).abs() < 1e-5);
        // How long until its front is level with somewhere: nothing for
        // where its front is, a quarter of a lap for a quarter of the way
        // round ahead of it, nearly a lap for just behind it; and for one
        // standing, as though it went on from now, gathering speed.
        let cruising = Buses([Leg { from: 0.0, round: 0.0, speed: BUS_SPEED, gathers: 0.0 }; BUSES]);
        let front = east.world(Vec2::new(0.0, BUS_HALF_LENGTH));
        let reaches = cruising.reaches_in(0, 0.0, front);
        assert!(reaches < 0.01 || reaches > LANES[0].length() / BUS_SPEED - 0.01, "{reaches}");
        let quarter = Vec2::new(0.0, -LANES[0].out) * 0.9;
        let lead = (BUS_HALF_LENGTH / LANES[0].out).atan() * LANES[0].out / BUS_SPEED;
        let lap = LANES[0].length() / BUS_SPEED;
        assert!((cruising.reaches_in(0, 0.0, quarter) - (lap / 4.0 - lead)).abs() < 0.05, "{}", cruising.reaches_in(0, 0.0, quarter));
        assert!(cruising.reaches_in(0, 0.0, Vec2::new(LANES[0].out, 30.0)) > lap - 3.0);
        let standing = Buses([Leg { from: 0.0, round: 0.0, speed: 0.0, gathers: 0.0 }; BUSES]);
        let ahead = east.world(Vec2::new(0.0, BUS_HALF_LENGTH + 8.0));
        let from_rest = (2.0 * 8.0 / bridge_wars::BUS_GATHERS).sqrt();
        assert!((standing.reaches_in(0, 0.0, ahead) - from_rest).abs() < 0.05, "{}", standing.reaches_in(0, 0.0, ahead));
        // A game's buses begin each at its thrown place round its own
        // lane, at full speed.
        let mut throws = [0.0, 0.25, 0.5, 0.99].into_iter();
        let begun = Buses::new(50.0, &mut || throws.next().unwrap());
        for (number, leg) in begun.0.iter().enumerate() {
            assert_eq!((leg.from, leg.speed, leg.gathers), (50.0, BUS_SPEED, 0.0));
            let share = [0.0, 0.25, 0.5, 0.99][number];
            assert!((leg.round - f64::from(share * LANES[number].length())).abs() < 1e-3);
        }
    }

    #[test]
    fn a_bus_goes_as_its_leg_has_it_slowing_to_stand_and_gathering_to_full_speed() {
        use bridge_wars::{BUS_SPEED, BusLog, Buses, Leg};
        // Going steadily: so far for so long; never before it began.
        let steady = Leg { from: 10.0, round: 5.0, speed: 10.0, gathers: 0.0 };
        assert_eq!(steady.at(12.0), (25.0, 10.0));
        assert_eq!(steady.at(3.0), (5.0, 10.0));
        // Slowing at 5 m/s each second from 10 m/s: standing 2 s later,
        // 10 m on, and standing there for good.
        let slowing = Leg { from: 0.0, round: 0.0, speed: 10.0, gathers: -5.0 };
        let (round, speed) = slowing.at(1.0);
        assert!((round - 7.5).abs() < 1e-9 && (speed - 5.0).abs() < 1e-6);
        assert_eq!(slowing.at(2.0), (10.0, 0.0));
        assert_eq!(slowing.at(300.0), (10.0, 0.0));
        // Gathering speed from standing: to full speed, and on at that.
        let gathering = Leg { from: 0.0, round: 0.0, speed: 0.0, gathers: 4.0 };
        let full = f64::from(BUS_SPEED) / 4.0;
        let (round, speed) = gathering.at(full);
        assert!((round - 2.0 * full * full).abs() < 1e-6 && (speed - BUS_SPEED).abs() < 1e-5);
        let (later, speed) = gathering.at(full + 10.0);
        assert!((later - round - 10.0 * f64::from(BUS_SPEED)).abs() < 1e-4 && speed == BUS_SPEED);
        // What is kept of how they went: where each was a moment ago, by
        // how it was going then.
        let mut log = BusLog::default();
        assert_eq!(log.now(), None);
        let first = Buses([steady; 4]);
        log.take(&first);
        let mut second = first;
        second.0[2] = Leg { from: 14.0, round: 45.0, speed: 10.0, gathers: -5.0 };
        log.take(&second);
        log.take(&second);
        assert_eq!(log.now(), Some(second));
        let at = |log: &BusLog, number: usize, clock: f64| log.at(number, clock).unwrap().at;
        assert!(at(&log, 2, 13.0).distance(first.at(2, 13.0).at) < 1e-4, "before it began to slow");
        assert!(at(&log, 2, 15.0).distance(second.at(2, 15.0).at) < 1e-4);
        assert!(at(&log, 0, 15.0).distance(first.at(0, 15.0).at) < 1e-4);
        // Another game's, begun before the last kept: begun again.
        let mut again = second;
        again.0[1] = Leg { from: 1.0, round: 0.0, speed: BUS_SPEED, gathers: 0.0 };
        log.take(&again);
        assert!(at(&log, 1, 13.0).distance(again.at(1, 13.0).at) < 1e-4);
    }

    #[test]
    fn a_bus_stops_short_of_a_crossing_anyone_is_on_until_nobody_is_and_goes_over_one_nobody_is_on() {
        use bridge_wars::{
            BUS_BRAKE, BUS_GATHERS, BUS_HALF_LENGTH, BUS_HALF_WIDTH, BUS_SPEED, BUS_STOP_SHORT, BUSES, Bus, Buses, CROSSING_ACROSS,
            CROSSING_ALONG, CROSSING_GRACE, CROSSING_OUT, CROSSINGS, LANES, Leg, crossing_at,
        };
        // On a crossing: on its paint, or with your feet on its edge; on
        // the road beside it, in the air over it higher than a bus hits,
        // or anywhere else, not.
        for (index, &round) in CROSSINGS.iter().enumerate() {
            let (out, along) = (Vec2::new(round.cos(), -round.sin()), Vec2::new(-round.sin(), -round.cos()));
            let place = |across: f32, ahead: f32, up: f32| {
                let at = out * (CROSSING_OUT + across) + along * ahead;
                Vec3::new(at.x, up, at.y)
            };
            assert_eq!(crossing_at(place(0.0, 0.0, 0.02)), Some(index));
            assert_eq!(crossing_at(place(CROSSING_ACROSS, -CROSSING_ALONG - CROSSING_GRACE + 0.01, 1.0)), Some(index));
            assert_eq!(crossing_at(place(-CROSSING_ACROSS + 1.0, CROSSING_ALONG + CROSSING_GRACE + 0.05, 0.0)), None);
            assert_eq!(crossing_at(place(-CROSSING_ACROSS - CROSSING_GRACE - 0.05, 0.0, 0.0)), None);
            assert_eq!(crossing_at(place(0.0, 0.0, 3.5)), None);
        }
        assert_eq!(crossing_at(Vec3::new(LANES[0].out, 0.0, 0.0)), None);
        // How far past a crossing's middle line, the way the bus goes, the
        // furthest corner of its front is, and the hindmost.
        let reach = |lane: usize, bus: &Bus, crossing: usize| {
            let line = Vec2::new(CROSSINGS[crossing].cos(), -CROSSINGS[crossing].sin());
            let along = Vec2::new(-CROSSINGS[crossing].sin(), -CROSSINGS[crossing].cos()) * if LANES[lane].anticlockwise { 1.0 } else { -1.0 };
            let corners = [-1.0, 1.0].map(|side| (bus.world(Vec2::new(side * BUS_HALF_WIDTH, BUS_HALF_LENGTH)) - line * CROSSING_OUT).dot(along));
            (corners[0].max(corners[1]), corners[0].min(corners[1]))
        };
        let frame = 1.0 / 60.0;
        for lane in 0..BUSES {
            // A minute's drive short of the first crossing, at full speed.
            let length = f64::from(LANES[lane].length());
            let line = f64::from(if LANES[lane].anticlockwise { 1.0 } else { -1.0 } * CROSSINGS[0]) * f64::from(LANES[lane].out);
            let start = (line - 80.0).rem_euclid(length);
            let mut buses = Buses([Leg { from: 0.0, round: start, speed: BUS_SPEED, gathers: 0.0 }; BUSES]);
            // Somebody on it from the start: it slows, no harder than it
            // gently brakes, stands with its front short of the crossing,
            // and never on it, and stays as long as they are there.
            let (mut clock, mut changes, mut hardest) = (0.0f64, 0, 0.0f32);
            let mut stood_at = None;
            for _ in 0..(30.0 / frame) as usize {
                clock += frame;
                if buses.drive(clock, [true, false, false, false]) {
                    changes += 1;
                }
                hardest = hardest.max(-buses.0[lane].gathers);
                let bus = buses.at(lane, clock);
                let (front, _) = reach(lane, &bus, 0);
                assert!(front < -CROSSING_ALONG, "lane {lane}: its front {front} m past the crossing's middle");
                if buses.going(lane, clock).1 == 0.0 && stood_at.is_none() {
                    stood_at = Some(front);
                }
            }
            let front = stood_at.expect("it stopped");
            assert!((front + CROSSING_ALONG + BUS_STOP_SHORT).abs() < 0.05, "lane {lane}: stood with its front {front}");
            assert!(hardest <= BUS_BRAKE + 0.2, "lane {lane}: braked at {hardest}");
            // Each bus was told to brake once, all in the same look or
            // nearly: nothing more was sent while they stood.
            assert!(changes <= BUSES, "{changes} changes");
            // Nobody on it: it goes on, gathering speed, over the crossing
            // and to full speed.
            let gone_at = clock;
            for _ in 0..(10.0 / frame) as usize {
                clock += frame;
                buses.drive(clock, [false; 4]);
            }
            assert_eq!(buses.going(lane, clock).1, BUS_SPEED);
            let (_, hindmost) = reach(lane, &buses.at(lane, clock), 0);
            assert!(hindmost > CROSSING_ALONG, "lane {lane}: past it");
            assert!(buses.going(lane, gone_at + f64::from(BUS_SPEED / BUS_GATHERS) - 0.05).1 < BUS_SPEED);
        }
        // Nobody on any crossing: a whole lap of every lane at full speed,
        // nothing changed, nothing sent.
        let mut buses = Buses(std::array::from_fn(|lane| Leg { from: 0.0, round: 3.0 * lane as f64, speed: BUS_SPEED, gathers: 0.0 }));
        let (mut clock, start) = (0.0f64, buses);
        for _ in 0..(32.0 / frame) as usize {
            clock += frame;
            assert!(!buses.drive(clock, [false; 4]));
        }
        assert_eq!(buses, start);
        // Somebody stepping on late, with the bus's front 3 m short of
        // where it would stop: harder, and short of it all the same.
        // Somebody stepping on in front of it once its front is on the
        // crossing: it stops at once, there.
        for (late, on_it) in [(3.0f64, false), (-3.5, true)] {
            for lane in 0..BUSES {
                let mut buses = Buses([Leg { from: 0.0, round: 0.0, speed: BUS_SPEED, gathers: 0.0 }; BUSES]);
                let mut clock = 0.0f64;
                // Drive up to where it is wanted, nobody on the crossing.
                let near = Vec2::new(CROSSINGS[2].cos(), -CROSSINGS[2].sin()) * LANES[lane].out;
                loop {
                    let bus = buses.at(lane, clock);
                    let (front, _) = reach(lane, &bus, 2);
                    if bus.at.distance(near) < 15.0 && front >= -CROSSING_ALONG - BUS_STOP_SHORT - late as f32 {
                        break;
                    }
                    clock += frame;
                    buses.drive(clock, [false; 4]);
                    assert!(clock < 60.0, "lane {lane} never got there");
                }
                let mut most = f32::MIN;
                for _ in 0..(5.0 / frame) as usize {
                    clock += frame;
                    buses.drive(clock, [false, false, true, false]);
                    most = most.max(reach(lane, &buses.at(lane, clock), 2).0);
                }
                assert_eq!(buses.going(lane, clock).1, 0.0, "lane {lane} still going");
                if on_it {
                    assert!(most < -CROSSING_ALONG + 4.0 + BUS_SPEED * frame as f32 + 0.1, "lane {lane}: {most}");
                } else {
                    assert!(most < -CROSSING_ALONG, "lane {lane}: onto the crossing, {most}");
                }
            }
        }
        // Somebody on a crossing a bus has passed, its front over the far
        // edge, or on another lane's other crossing: nothing to it.
        let lane = 0;
        let line = f64::from(CROSSINGS[1]) * f64::from(LANES[lane].out);
        let mut buses = Buses([Leg { from: 0.0, round: line + 6.0, speed: BUS_SPEED, gathers: 0.0 }; BUSES]);
        assert!(reach(lane, &buses.at(lane, 0.0), 1).1 > CROSSING_ALONG);
        let going = buses.0[lane];
        buses.drive(0.1, [true, true, false, true]);
        assert_eq!(buses.0[lane], going);
    }

    #[test]
    fn a_bus_runs_over_whoever_its_front_comes_to_and_nobody_at_its_side_in_it_or_over_it() {
        use bridge_wars::{BUS_HALF_LENGTH, BUS_HALF_WIDTH, BUS_ROAD, BUS_SPEED, BUS_TALL, Bus, LANES};
        let radius = 0.45;
        // A step of a 120th of a second, of a 15th, and of a slow frame,
        // on the outside lane and on the innermost, going the other way.
        for lane in [0, 3] {
            for dt in [1.0 / 120.0, 1.0 / 15.0, 0.25] {
                let (before, bus) = (Bus::on(&LANES[lane], 1000.0), Bus::on(&LANES[lane], 1000.0 + f64::from(BUS_SPEED) * dt));
                let came_on = f64::from(BUS_SPEED) * dt;
                let on_the_road = |local: Vec2| {
                    let at = bus.world(local);
                    Vec3::new(at.x, BUS_ROAD, at.y)
                };
                // Standing in the middle of the lane where the front has
                // just got to: run over, in that step and no sooner.
                let reached = on_the_road(Vec2::new(0.0, BUS_HALF_LENGTH + radius - 0.01));
                assert!(bus.runs_over(&before, reached, reached, radius), "dt {dt}");
                let not_yet = on_the_road(Vec2::new(0.0, BUS_HALF_LENGTH + radius + 0.05));
                assert!(!bus.runs_over(&before, not_yet, not_yet, radius), "dt {dt}");
                // Anywhere across its width, and a little outside it; not
                // someone it only clips with a corner.
                for across in [-BUS_HALF_WIDTH, -0.9, 0.9, BUS_HALF_WIDTH + 0.1] {
                    let at = on_the_road(Vec2::new(across, BUS_HALF_LENGTH + 0.1));
                    assert!(bus.runs_over(&before, at, at, radius), "{across} m across, dt {dt}");
                }
                for across in [-(BUS_HALF_WIDTH + 0.3), BUS_HALF_WIDTH + 0.45] {
                    let at = on_the_road(Vec2::new(across, BUS_HALF_LENGTH + 0.1));
                    assert!(!bus.runs_over(&before, at, at, radius), "{across} m across, dt {dt}");
                }
                // In the air in front of it, off a jump: run over. Higher
                // than a bus hits, coming down onto its roof: not.
                let jumping = reached + Vec3::Y * 1.6;
                assert!(bus.runs_over(&before, jumping, jumping, radius));
                let leaping = reached + Vec3::Y * (BUS_TALL + 0.3);
                assert!(!bus.runs_over(&before, leaping, leaping, radius));
                // At its side, walking into its flank, at its front doorway
                // (4.9 to 6.1 m ahead of its middle) or its middle one, from
                // the kerb's side or the other: never, however far in.
                for ahead in [-6.0, -3.0, 0.5, 3.0, 5.35, 5.65] {
                    for side in [-1.0, 1.0] {
                        for inside in [0.5, 0.0, -0.6, -1.5] {
                            let was = before.world(Vec2::new(side * (BUS_HALF_WIDTH + radius + 0.3), ahead));
                            let now = bus.world(Vec2::new(side * (BUS_HALF_WIDTH + radius + inside - 0.5), ahead));
                            let (was, now) = (Vec3::new(was.x, 0.0, was.y), Vec3::new(now.x, 0.0, now.y));
                            assert!(!bus.runs_over(&before, was, now, radius), "{ahead} m ahead, side {side}, in {inside}, dt {dt}");
                        }
                    }
                }
                // Riding it, standing just inside its windscreen: not.
                let aboard = Vec2::new(0.4, BUS_HALF_LENGTH - 0.5);
                let (was, now) = (before.world(aboard), bus.world(aboard));
                assert!(!bus.runs_over(&before, Vec3::new(was.x, 0.47, was.y), Vec3::new(now.x, 0.47, now.y), radius));
                // Stepping in front of it from the side at the last moment,
                // level with its nose: run over.
                let was = before.world(Vec2::new(BUS_HALF_WIDTH + radius + 0.2, BUS_HALF_LENGTH + came_on as f32 + 0.3));
                let now = bus.world(Vec2::new(BUS_HALF_WIDTH - 0.3, BUS_HALF_LENGTH + 0.2));
                assert!(bus.runs_over(&before, Vec3::new(was.x, 0.0, was.y), Vec3::new(now.x, 0.0, now.y), radius), "dt {dt}");
                // Behind it, and fallen away under the road: not.
                let behind = on_the_road(Vec2::new(0.0, -BUS_HALF_LENGTH - 1.0));
                assert!(!bus.runs_over(&before, behind, behind, radius));
                assert!(!bus.runs_over(&before, reached - Vec3::Y * 3.0, reached - Vec3::Y * 3.0, radius));
                // A bus that has not moved runs nobody over: it is a wall.
                let walked_into = (on_the_road(Vec2::new(0.0, BUS_HALF_LENGTH + 1.0)), on_the_road(Vec2::new(0.0, BUS_HALF_LENGTH + 0.2)));
                assert!(!bus.runs_over(&bus, walked_into.0, walked_into.1, radius));
            }
        }
        // To the computer's players it is solid at its sides: one found
        // in it is out at the side it was nearer, level with where it
        // was; one clear of it, or past either end, is left.
        let bus = Bus::on(&LANES[1], 400.0);
        let within = bus.world(Vec2::new(0.6, 2.0));
        let out = bus.beside(within, radius).expect("in the bus");
        assert!(bus.local(out).distance(Vec2::new(BUS_HALF_WIDTH + radius, 2.0)) < 1e-3);
        let out = bus.beside(bus.world(Vec2::new(-1.7, -5.0)), radius).expect("in the bus");
        assert!(bus.local(out).distance(Vec2::new(-(BUS_HALF_WIDTH + radius), -5.0)) < 1e-3);
        assert_eq!(bus.beside(bus.world(Vec2::new(BUS_HALF_WIDTH + radius + 0.01, 0.0)), radius), None);
        assert_eq!(bus.beside(bus.world(Vec2::new(0.0, BUS_HALF_LENGTH + 0.2)), radius), None);
        assert_eq!(bus.beside(bus.world(Vec2::new(0.0, -BUS_HALF_LENGTH - 0.2)), radius), None);
        // And where one sweeps is a band of the road either side of its
        // lane's middle; between them, the strip down the ring's middle
        // and each half's line, none.
        for (number, lane) in LANES.iter().enumerate() {
            assert_eq!(Bus::sweeps(Vec2::new(0.0, lane.out + 2.0), radius), Some(number));
            assert_eq!(Bus::sweeps(Vec2::new(lane.out - 2.0, 0.0), radius), Some(number));
            assert_eq!(Bus::sweeps(Vec2::new(0.0, lane.out + 2.5), radius), None);
        }
        for between in [50.0, 57.5, 65.0, 72.0, 40.0] {
            assert_eq!(Bus::sweeps(Vec2::new(0.0, between), radius), None, "{between} m out");
        }
    }

    #[test]
    fn who_lies_flat_is_said_by_seat_with_until_when() {
        let flat = Flattened(vec![(3, 100.5), (9, 104.0)]);
        assert_eq!(flat.until(3), Some(100.5));
        assert_eq!(flat.until(9), Some(104.0));
        assert_eq!(flat.until(0), None);
        assert_eq!(Flattened::default().until(3), None);
    }

    // ---------------------------------------- the four games of 2026-11-18

    #[test]
    fn each_of_the_four_is_played_on_its_own_venue_and_at_a_table() {
        for game in Minigame::TABLES {
            assert!(game.at_a_table(), "{game:?}");
            assert_eq!(game.venue().game(), Some(game), "{game:?}");
            assert!(table::shape(game).is_some(), "{game:?}");
        }
        // The selector's order.
        assert_eq!(Minigame::TABLES, [Minigame::BlendIn, Minigame::KnockOff, Minigame::BridgeRace, Minigame::LastStand]);
        // The two older games are not, and no other venue is any of theirs.
        for game in [Minigame::BridgeWars, Minigame::HouseBuilder] {
            assert!(!game.at_a_table(), "{game:?}");
            assert_eq!(game.venue().game(), None, "{game:?}");
        }
        assert_eq!(Minigame::BridgeWars.venue(), Venue::Arena);
        assert_eq!(Minigame::HouseBuilder.venue(), Venue::Lobby);
        for venue in [Venue::Town, Venue::Home, Venue::Lobby, Venue::Plot(3), Venue::Arena] {
            assert_eq!(venue.game(), None, "{venue:?}");
        }
    }

    #[test]
    fn what_was_sent_before_the_four_games_is_sent_as_it_was() {
        // The new venues, games, asks and tells come after all that was
        // there, which keeps the number it was sent as.
        fn sent<T: Serialize>(what: &T) -> Vec<u8> {
            postcard::to_allocvec(what).unwrap()
        }
        assert_eq!(sent(&Venue::Arena), [4]);
        assert_eq!(sent(&Venue::Plot(2)), [3, 2]);
        assert_eq!(sent(&Venue::BlendIn), [5]);
        assert_eq!(sent(&Venue::LastStand), [8]);
        assert_eq!(sent(&Minigame::HouseBuilder), [1]);
        assert_eq!(sent(&Minigame::BlendIn), [2]);
        assert_eq!(sent(&Ask::RunOver { bus: 2 }), [20, 2]);
        assert_eq!(sent(&Ask::Join(Minigame::LastStand)), [21, 5]);
        assert_eq!(sent(&Tell::Back { seat: 1, metres: 0.0, compass: false })[0], 12);
        assert_eq!(sent(&Tell::Entered { game: Minigame::KnockOff, seat: 3 }), [13, 3, 3]);
        // And each game's own come back as they went.
        let asks = [
            Ask::Join(Minigame::BridgeRace),
            Ask::BlendIn(blend_in::Ask::Disguise { name: "Mochi".into(), look: "girl".into() }),
            Ask::KnockOff(knock_off::Ask::Aim { x: 0.6, z: -0.8, power: 7 }),
            Ask::BridgeRace(bridge_race::Ask::Metres { metres: 12.5, grants: 3 }),
            Ask::LastStand(last_stand::Ask::Buy(last_stand::Ware::Sharpness)),
        ];
        for ask in asks {
            let bytes = postcard::to_allocvec(&ask).unwrap();
            let back: Ask = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(format!("{back:?}"), format!("{ask:?}"));
        }
        let tells = [
            Tell::Entered { game: Minigame::LastStand, seat: 2 },
            Tell::BlendIn(blend_in::Tell::Stunned { until: 1_800_000_000.25 }),
            Tell::KnockOff(knock_off::Tell::Aimed { x: 0.0, z: -1.0, power: 5 }),
            Tell::BridgeRace(bridge_race::Tell::Metres { metres: 12.5, grants: 3 }),
            Tell::LastStand(last_stand::Tell::Upgraded { by: 1, ware: last_stand::Ware::Beam, level: 3 }),
        ];
        for tell in tells {
            let bytes = postcard::to_allocvec(&tell).unwrap();
            let back: Tell = postcard::from_bytes(&bytes).unwrap();
            assert_eq!(format!("{back:?}"), format!("{tell:?}"));
        }
        // A friend at a table of one of them.
        let playing = social::Presence::Playing { game: Minigame::KnockOff, queued: true };
        let back: social::Presence = postcard::from_bytes(&postcard::to_allocvec(&playing).unwrap()).unwrap();
        assert_eq!(back, playing);
    }

    #[test]
    fn blend_in_is_arrived_at_as_the_town_is() {
        for seat in 0..4 {
            assert_eq!(arrival(Venue::BlendIn, seat), arrival(Venue::Town, seat), "seat {seat}");
            assert_eq!(blend_in::arrival(seat), arrival(Venue::Town, seat), "seat {seat}");
        }
    }

    #[test]
    fn knock_off_is_arrived_at_on_the_diagonals_facing_the_middle() {
        // Where a seat stands before its table's island is known (since
        // 2026-11-27 the game stands it on the island as it comes,
        // `knock_off::waiting_spot`): 7 m out, as on the disc of 12 m it was.
        let spots: Vec<(Vec3, Quat)> = (0..4).map(|seat| arrival(Venue::KnockOff, seat)).collect();
        let (ring, ice) = (7.0, knock_off::RADIUS);
        assert!(ring < ice - 2.0, "{ring} m out, the ice {ice}");
        for (seat, &(at, turn)) in spots.iter().enumerate() {
            assert_eq!(arrival(Venue::KnockOff, seat), knock_off::arrival(seat));
            assert!((at.xz().length() - ring).abs() < 1e-4, "seat {seat} at {at}");
            assert!(at.xz().normalize().distance(knock_off::side(seat)) < 1e-5, "seat {seat} at {at}");
            assert_eq!(at.y, 0.0);
            // On a diagonal: as far across as along.
            assert!((at.x.abs() - at.z.abs()).abs() < 1e-4, "seat {seat} at {at}");
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(-at.xz().normalize()) < 1e-4, "seat {seat} faces {facing}");
        }
        // Each on a diagonal of its own.
        for (a, &(at, _)) in spots.iter().enumerate() {
            for &(other, _) in &spots[a + 1..] {
                assert!(at.distance(other) > 9.0, "{at} and {other}");
            }
        }
    }

    #[test]
    fn two_people_at_bridge_race_are_on_one_team_one_on_each_or_one_against_the_other_a_third_each() {
        use bridge_race::{Pairing, pairing};
        assert_eq!(pairing(0.0), Pairing::Together);
        assert_eq!(pairing(0.34), Pairing::Apart);
        assert_eq!(pairing(0.99), Pairing::Alone);
        // A third of the way each, over the rolls.
        let mut ways = [0u32; 3];
        for step in 0..3000 {
            ways[match pairing(step as f32 / 3000.0) {
                Pairing::Together => 0,
                Pairing::Apart => 1,
                Pairing::Alone => 2,
            }] += 1;
        }
        assert!(ways.iter().all(|way| (990..=1010).contains(way)), "{ways:?}");
    }

    #[test]
    fn bridge_race_seats_two_teams_on_their_islands_facing_the_golden_one() {
        use bridge_race::{GOLDEN, ISLAND_RADIUS, ISLANDS, LIFT, PEOPLE_FIRST, team};
        // Seat by seat, red and blue in turn.
        assert_eq!((0..4).map(team).collect::<Vec<u8>>(), [0, 1, 0, 1]);
        for seat in 0..4 {
            let (at, turn) = arrival(Venue::BridgeRace, seat);
            assert_eq!((at, turn), bridge_race::arrival(seat));
            let island = ISLANDS[usize::from(team(seat))];
            let out = at.xz().distance(island);
            let expected = if seat < 2 { 0.0 } else { 3.0 };
            assert!((out - expected).abs() < 1e-4, "seat {seat}: {out} m from its island's middle");
            assert!(out < ISLAND_RADIUS - 2.0, "seat {seat}: near the edge");
            // Toward the golden island, and facing it.
            assert!(at.xz().distance(GOLDEN) <= island.distance(GOLDEN) + 1e-4, "seat {seat}");
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance((GOLDEN - at.xz()).normalize()) < 1e-4, "seat {seat} faces {facing}");
            assert_eq!(at.y, LIFT);
        }
        // The two teams' islands, and the golden island as far from each.
        // (32 m apart since Hajun's export of 2026-11-18; 42 until then.)
        assert!((ISLANDS[0].distance(ISLANDS[1]) - 32.0).abs() < 1e-3);
        assert!((ISLANDS[0].distance(GOLDEN) - ISLANDS[1].distance(GOLDEN)).abs() < 1e-3);
        // People take every seat once, the first two on one team.
        let mut taken = PEOPLE_FIRST;
        taken.sort_unstable();
        assert_eq!(taken, [0, 1, 2, 3]);
        assert_eq!(team(PEOPLE_FIRST[0]), team(PEOPLE_FIRST[1]));
        assert_ne!(team(PEOPLE_FIRST[0]), team(PEOPLE_FIRST[2]));
        // One person against the computer is seats 0 and 1, one a side.
        assert_eq!(bridge_race::seats_for(1), 2);
        assert_ne!(team(PEOPLE_FIRST[0]), team(1));
        // Everyone waits on the room's floor, apart.
        let waiting: Vec<Vec3> = (0..4).map(bridge_race::waiting_spot).collect();
        for (seat, at) in waiting.iter().enumerate() {
            assert_eq!(at.y, bridge_race::WAITING_FLOOR);
            assert!((at.xz().distance(bridge_race::WAITING_ROOM) - bridge_wars::WAIT_RING).abs() < 1e-4, "seat {seat} at {at}");
            for other in &waiting[seat + 1..] {
                assert!(at.distance(*other) > 3.0, "{at} and {other}");
            }
        }
    }

    #[test]
    fn last_stand_seats_everyone_round_the_anchor_facing_out() {
        use last_stand::{ANCHOR, LIFT};
        let spots: Vec<(Vec3, Quat)> = (0..4).map(|seat| arrival(Venue::LastStand, seat)).collect();
        for (seat, &(at, turn)) in spots.iter().enumerate() {
            assert_eq!((at, turn), last_stand::arrival(seat));
            let out = at.xz() - ANCHOR;
            assert!((out.length() - 3.2).abs() < 1e-4, "seat {seat}: {} m from the anchor", out.length());
            // On a diagonal, clear of the anchor's slab, 2 m across.
            assert!((out.x.abs() - out.y.abs()).abs() < 1e-4, "seat {seat}: {out}");
            assert!(out.x.abs().min(out.y.abs()) > 1.0 + 0.9, "seat {seat}: on the slab");
            let facing = (turn * Vec3::NEG_Z).xz();
            assert!(facing.distance(out.normalize()) < 1e-4, "seat {seat} faces {facing}");
            assert_eq!(at.y, LIFT);
        }
        for (a, &(at, _)) in spots.iter().enumerate() {
            for &(other, _) in &spots[a + 1..] {
                assert!(at.distance(other) > 4.0, "{at} and {other}");
            }
        }
        let waiting: Vec<Vec3> = (0..4).map(last_stand::waiting_spot).collect();
        for (seat, at) in waiting.iter().enumerate() {
            assert_eq!(at.y, last_stand::WAITING_FLOOR);
            assert!((at.xz().distance(last_stand::WAITING_ROOM) - bridge_wars::WAIT_RING).abs() < 1e-4, "seat {seat} at {at}");
            for other in &waiting[seat + 1..] {
                assert!(at.distance(*other) > 3.0, "{at} and {other}");
            }
        }
    }

    #[test]
    fn a_thing_costs_what_its_rarity_does_for_now_and_sells_back_for_half() {
        use bag::{Group, Money, Rarity, THINGS, thing};
        // Hajun's numbers, 2026-11-21, and Transcendent's and those in gems
        // with his list of 2026-11-25.
        let prices = [50, 150, 400, 1_000, 2_500, 15_000, 100_000];
        let gems = [3, 8, 20, 50, 125, 750, 5_000];
        for ((rarity, price), gem) in Rarity::ALL.into_iter().zip(prices).zip(gems) {
            assert_eq!((rarity.price(), rarity.gems()), (price, gem), "{rarity:?}");
        }
        for thing in THINGS {
            let own = match thing.money {
                Money::Coins => thing.rarity.price(),
                Money::Gems => thing.rarity.gems(),
            };
            assert_eq!(thing.price, own, "{}", thing.kind);
            assert_eq!(thing.resale(), thing.price / 2, "{}", thing.kind);
            assert_eq!(thing.worth(), thing.rarity.price(), "{}", thing.kind);
            assert_eq!(bag::thing(thing.kind), Some(thing));
            assert_eq!(THINGS.iter().filter(|other| other.kind == thing.kind).count(), 1, "{} twice", thing.kind);
            // A kind is said over the wire and kept: plain, short, lower case.
            assert!(thing.kind.len() <= 40 && thing.kind.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'), "{}", thing.kind);
        }
        // As their file has them (`assets/itemrarities.txt`; the game's
        // `bag::panel` holds every line of it to these).
        let rare = |kind: &str| thing(kind).map(|thing| (thing.rarity, thing.money));
        assert_eq!(rare("streetlamp"), Some((Rarity::Legendary, Money::Coins)));
        assert_eq!(rare("singlebed"), Some((Rarity::Uncommon, Money::Coins)));
        assert_eq!(rare("square_house_roofless"), Some((Rarity::Common, Money::Coins)));
        assert_eq!(rare("square_house_five_floor"), Some((Rarity::Epic, Money::Gems)));
        assert_eq!(rare("square_house_roofless_nine_floor"), Some((Rarity::Mythic, Money::Gems)));
        assert_eq!(rare("square_house_ten_floor"), Some((Rarity::Transcendent, Money::Coins)));
        assert_eq!(rare("chandelier"), Some((Rarity::Legendary, Money::Coins)));
        assert_eq!(rare("red_sports_car"), Some((Rarity::Mythic, Money::Coins)));
        assert_eq!(rare("gen"), Some((Rarity::Rare, Money::Coins)));
        assert_eq!(rare("yang"), Some((Rarity::Transcendent, Money::Coins)));
        let sold_for_gems = THINGS.iter().filter(|thing| thing.money == Money::Gems).count();
        assert_eq!((THINGS.len(), sold_for_gems), (118, 6));
        // 95 buildings, which stand on a plot, and 23 cars, which do not.
        assert_eq!(THINGS.iter().filter(|thing| thing.group == Group::Buildings && thing.stands()).count(), 95);
        assert_eq!(THINGS.iter().filter(|thing| thing.group == Group::Vehicles && !thing.stands()).count(), 23);
        assert_eq!(thing("a_castle"), None);
    }

    #[test]
    fn a_thing_sold_for_gems_is_paid_for_in_gems_and_sells_back_for_half_of_them() {
        use bag::{Bag, No};
        let house = "square_house_five_floor";
        // Ten gems, as everyone has: not enough for 50, whatever the coins.
        let mut bag = Bag::new();
        assert_eq!(bag.buy(house, 1_000_000), Err(No::Gems));
        assert_eq!((bag.gems, bag.count(house)), (10, 0));
        // With 60: bought for 50 of them, and the coins left as they were.
        bag.gems = 60;
        assert_eq!(bag.buy(house, 7), Ok(7));
        assert_eq!((bag.gems, bag.count(house)), (10, 1));
        // Sold back for 25 gems; the coins as they were again.
        assert_eq!(bag.sell(house, 7), Ok(7));
        assert_eq!((bag.gems, bag.count(house)), (35, 0));
        // Half of 125 is 62.
        let mut bag = Bag { gems: 125, ..Bag::new() };
        bag.buy("square_house_seven_floor", 0).unwrap();
        assert_eq!(bag.gems, 0);
        assert_eq!(bag.sell("square_house_seven_floor", 0), Ok(0));
        assert_eq!(bag.gems, 62);
        // And what is sold for coins spends no gems.
        let mut bag = Bag::new();
        assert_eq!(bag.buy("yang", 100_000), Ok(0));
        assert_eq!(bag.gems, 10);
        assert_eq!(bag.cars().collect::<Vec<_>>(), [bag::CAR, "yang"]);
    }

    #[test]
    fn everyone_begins_with_ten_gems_a_car_and_a_street_lamp() {
        use bag::{Bag, CAR, GEMS, LAMP};
        let bag = Bag::new();
        assert_eq!((bag.gems, GEMS), (10, 10));
        assert_eq!((bag.count(CAR), bag.count(LAMP), bag.owned.len()), (1, 1, 2));
        assert!(bag.has_car() && bag.stands.is_empty());
        assert_eq!(bag.free(LAMP), 1);
        assert_eq!(bag.count("singlebed"), 0);
    }

    #[test]
    fn buying_takes_the_price_and_selling_gives_half_of_it_back() {
        use bag::{Bag, CAR, LAMP, MOST_OF_A_KIND, No};
        let mut bag = Bag::new();
        // A bed for 150 out of 1,000; then no coins for a lamp at 2,500.
        assert_eq!(bag.buy("singlebed", 1_000), Ok(850));
        assert_eq!(bag.count("singlebed"), 1);
        assert_eq!(bag.buy(LAMP, 850), Err(No::Coins));
        assert_eq!(bag.count(LAMP), 1);
        assert_eq!(bag.buy("a_castle", 1_000_000), Err(No::Unknown));
        // Exactly the price is enough.
        assert_eq!(bag.buy("cafechair", 50), Ok(0));
        // Kept in the order of the things there are, whatever was bought
        // when.
        let kinds: Vec<&str> = bag.owned.iter().map(|(kind, _)| kind.as_str()).collect();
        assert_eq!(kinds, [LAMP, "singlebed", "cafechair", CAR]);
        // Sold back for half: the bed for 75, the lamp for 1,250, the car
        // for 7,500; and then there is none to sell.
        assert_eq!(bag.sell("singlebed", 0), Ok(75));
        assert_eq!(bag.sell("singlebed", 75), Err(No::None));
        assert_eq!(bag.sell(LAMP, 75), Ok(1_325));
        assert_eq!(bag.sell(CAR, 0), Ok(7_500));
        assert!(!bag.has_car());
        assert_eq!(bag.sell("a_castle", 0), Err(No::Unknown));
        // No more of one kind than anyone may own.
        let mut many = Bag::new();
        for _ in 0..MOST_OF_A_KIND {
            many.buy("cafechair", 50).unwrap();
        }
        assert_eq!(many.buy("cafechair", 50), Err(No::Full));
        assert_eq!(many.count("cafechair"), MOST_OF_A_KIND);
    }

    #[test]
    fn a_thing_stood_on_the_plot_is_out_of_the_bag_until_it_is_taken_back() {
        use bag::{Bag, CAR, LAMP, MOST_STANDING, No};
        let here = At::Ground { x: 3.0, y: 0.0, z: -4.5, yaw: 1.5 };
        let there = At::Ground { x: -2.0, y: 0.0, z: 6.0, yaw: 0.0 };
        let mut bag = Bag::new();
        assert_eq!(bag.next_id(), 1);
        assert_eq!(bag.put(1, LAMP, here), Ok(()));
        assert_eq!((bag.count(LAMP), bag.standing(LAMP), bag.free(LAMP)), (1, 1, 0));
        assert_eq!(bag.next_id(), 2);
        // The one lamp stands: there is none to stand again, nor to sell.
        assert_eq!(bag.put(2, LAMP, there), Err(No::None));
        assert_eq!(bag.sell(LAMP, 0), Err(No::None));
        // The same number again moves it.
        assert_eq!(bag.put(1, LAMP, there), Ok(()));
        assert_eq!(bag.stands.len(), 1);
        assert_eq!(bag.stands[0].at, there);
        // But not as something else.
        assert_eq!(bag.put(1, "singlebed", here), Err(No::Place));
        // The car stands on no plot, nor anything nowhere, nor what is not
        // owned.
        assert_eq!(bag.put(5, CAR, here), Err(No::Place));
        assert_eq!(bag.put(5, "singlebed", here), Err(No::None));
        let far = At::Ground { x: 40.0, y: 0.0, z: 0.0, yaw: 0.0 };
        let nowhere = At::Ground { x: f32::NAN, y: 0.0, z: 0.0, yaw: 0.0 };
        bag.buy("singlebed", 150).unwrap();
        assert_eq!(bag.put(5, "singlebed", far), Err(No::Place));
        assert_eq!(bag.put(5, "singlebed", nowhere), Err(No::Place));
        let in_a_wall = At::Wall { wall: 2, along: 1.5, up: 0.0, face: 1 };
        assert_eq!(bag.put(5, "singlebed", in_a_wall), Ok(()));
        assert_eq!(bag.put(6, "singlebed", At::Wall { wall: 2, along: 1.5, up: 0.0, face: 3 }), Err(No::Place));
        // Taken back, it is in the bag again, to sell.
        assert_eq!(bag.take(1), Ok(()));
        assert_eq!(bag.take(1), Err(No::None));
        assert_eq!(bag.free(LAMP), 1);
        assert_eq!(bag.sell(LAMP, 0), Ok(1_250));
        // No more on a plot than a plot is told of at once.
        let mut full = Bag::new();
        for _ in 0..2 {
            for _ in 0..60 {
                full.buy("cafechair", 50).unwrap_or_default();
            }
        }
        for _ in 0..60 {
            full.buy("circletable", 50).unwrap();
        }
        let mut stood = 0;
        for id in 1..=150 {
            let kind = if id <= 90 { "cafechair" } else { "circletable" };
            if full.put(id, kind, here).is_ok() {
                stood += 1;
            }
        }
        assert_eq!((stood, full.stands.len()), (MOST_STANDING, MOST_STANDING));
        assert_eq!(full.put(999, "circletable", here), Err(No::Full));
    }

    #[test]
    fn a_bag_kept_wrongly_is_put_right_as_it_is_read() {
        use bag::{Bag, CAR, LAMP, MOST_OF_A_KIND, Standing};
        let here = At::Ground { x: 1.0, y: 0.0, z: 1.0, yaw: 0.0 };
        let kept = Bag {
            gems: 3,
            owned: vec![
                (CAR.into(), 1),
                ("a_castle".into(), 4),
                ("cafechair".into(), 500),
                (LAMP.into(), 1),
                (LAMP.into(), 1),
            ],
            stands: vec![
                Standing { id: 7, kind: LAMP.into(), at: here },
                // The same number twice, a thing there is no longer, one
                // that is not owned, and one standing nowhere.
                Standing { id: 7, kind: "cafechair".into(), at: here },
                Standing { id: 8, kind: "a_castle".into(), at: here },
                Standing { id: 9, kind: "singlebed".into(), at: here },
                Standing { id: 10, kind: "cafechair".into(), at: At::Ground { x: 99.0, y: 0.0, z: 0.0, yaw: 0.0 } },
                Standing { id: 11, kind: "cafechair".into(), at: here },
            ],
        };
        let bag = kept.sane();
        assert_eq!(bag.gems, 3);
        assert_eq!(bag.owned, vec![(LAMP.to_string(), 2), ("cafechair".to_string(), MOST_OF_A_KIND), (CAR.to_string(), 1)]);
        let stood: Vec<(u32, &str)> = bag.stands.iter().map(|stood| (stood.id, stood.kind.as_str())).collect();
        assert_eq!(stood, [(7, LAMP), (11, "cafechair")]);
        // One that is right already is left as it is.
        assert_eq!(bag.clone().sane(), bag);
        assert_eq!(Bag::new().sane(), Bag::new());
    }

    #[test]
    fn a_part_of_what_stands_is_painted_and_is_as_it_was_made_once_taken_back() {
        use bag::{Bag, LAMP, No, Paint, Paints};
        let here = At::Ground { x: 1.0, y: 0.0, z: -2.0, yaw: 0.0 };
        let mut bag = Bag::new();
        bag.buy("singlebed", 1_000).unwrap();
        bag.put(1, LAMP, here).unwrap();
        bag.put(2, "singlebed", here).unwrap();
        let mut paints = Paints::default();
        // The bed's blanket, blue; then green over it: one colour to a part.
        assert_eq!(paints.set(&bag.stands, 2, "blanket", [0, 0, 255]), Ok(()));
        assert_eq!(paints.set(&bag.stands, 2, "blanket", [0, 255, 0]), Ok(()));
        assert_eq!(paints.set(&bag.stands, 2, "bedframe", [10, 20, 30]), Ok(()));
        assert_eq!(paints.set(&bag.stands, 1, "lampostbody", [255, 255, 255]), Ok(()));
        assert_eq!(paints.0.len(), 3);
        assert_eq!(paints.colour(2, "blanket"), Some([0, 255, 0]));
        assert_eq!(paints.colour(2, "mattress"), None);
        assert_eq!(paints.of(2).collect::<Vec<_>>(), [("blanket", [0, 255, 0]), ("bedframe", [10, 20, 30])]);
        // Nothing that does not stand, and no part with no name or one too
        // long to be one.
        assert_eq!(paints.set(&bag.stands, 9, "blanket", [1, 2, 3]), Err(No::None));
        assert_eq!(paints.set(&bag.stands, 2, "", [1, 2, 3]), Err(No::Unknown));
        assert_eq!(paints.set(&bag.stands, 2, &"x".repeat(bag::PART_NAME + 1), [1, 2, 3]), Err(No::Unknown));
        assert_eq!(paints.set(&bag.stands, 2, "two\nlines", [1, 2, 3]), Err(No::Unknown));
        assert_eq!(paints.0.len(), 3);
        // The paint taken off a part: as its model has it, and the rest as
        // they were; and nothing to take off a part that is not painted.
        assert!(paints.clear(2, "bedframe"));
        assert_eq!(paints.colour(2, "bedframe"), None);
        assert!(!paints.clear(2, "bedframe"));
        assert!(!paints.clear(1, "blanket"));
        assert_eq!(paints.colour(2, "blanket"), Some([0, 255, 0]));
        assert_eq!(paints.set(&bag.stands, 2, "bedframe", [10, 20, 30]), Ok(()));
        // Moved, the bed keeps its colours: it is the same thing standing.
        bag.put(2, "singlebed", At::Ground { x: 4.0, y: 0.0, z: 4.0, yaw: 1.0 }).unwrap();
        assert!(!paints.fit(&bag.stands));
        assert_eq!(paints.colour(2, "blanket"), Some([0, 255, 0]));
        // Taken back into the bag, its colours go with it, and stood again
        // it is as it was made (Hajun: "color gets reset if u trash that
        // item and place it back, but stays if u leave it").
        bag.take(2).unwrap();
        assert!(paints.fit(&bag.stands));
        assert_eq!(paints.0, [Paint { id: 1, part: "lampostbody".into(), colour: [255, 255, 255] }]);
        bag.put(2, "singlebed", here).unwrap();
        assert_eq!(paints.colour(2, "blanket"), None);
        // No more than a plot has.
        let mut full = Paints::default();
        for n in 0..bag::MOST_PAINTED {
            assert_eq!(full.set(&bag.stands, 1, &format!("part{n}"), [0; 3]), Ok(()));
        }
        assert_eq!(full.set(&bag.stands, 1, "one_more", [0; 3]), Err(No::Full));
        assert_eq!(full.set(&bag.stands, 1, "part7", [9; 3]), Ok(()));
        // Kept wrongly, it is put right as it is read: nothing of what does
        // not stand, and one colour to a part, the first.
        let wrong = Paints(vec![
            Paint { id: 1, part: "lampostbody".into(), colour: [1, 1, 1] },
            Paint { id: 1, part: "lampostbody".into(), colour: [2, 2, 2] },
            Paint { id: 44, part: "door".into(), colour: [3, 3, 3] },
            Paint { id: 1, part: String::new(), colour: [4, 4, 4] },
        ]);
        assert_eq!(wrong.sane(&bag.stands).0, [Paint { id: 1, part: "lampostbody".into(), colour: [1, 1, 1] }]);
    }

    #[test]
    fn who_rests_goes_over_the_wire_after_everything_an_app_from_before_knows() {
        // Appended after the bag's, which was last: a variant's number is
        // all postcard writes of which it is, so nothing an app from
        // before sends or is sent reads as anything else.
        assert_eq!(postcard::to_allocvec(&Ask::Bag(bag::Ask::Watch)).unwrap()[0], 28);
        assert_eq!(postcard::to_allocvec(&Ask::Resting).unwrap(), [29]);
        for (pose, number) in [(Pose::Sit, 0), (Pose::Lounge, 1), (Pose::Lie, 2)] {
            let bytes = postcard::to_allocvec(&Ask::Rest(Some(pose))).unwrap();
            assert_eq!(bytes, [30, 1, number]);
            assert!(matches!(postcard::from_bytes(&bytes).unwrap(), Ask::Rest(Some(back)) if back == pose));
        }
        assert_eq!(postcard::to_allocvec(&Ask::Rest(None)).unwrap(), [30, 0]);
        let bag_tell = postcard::to_allocvec(&Tell::Bag(bag::Tell::Theirs { who: 1, stands: Vec::new() })).unwrap()[0];
        let resting = Tell::Resting(vec![Posed { who: 7, pose: Pose::Lie }, Posed { who: u64::MAX, pose: Pose::Sit }]);
        let bytes = postcard::to_allocvec(&resting).unwrap();
        assert_eq!(bytes[0], bag_tell + 1, "after the bag's, the last an app from before is told");
        let Tell::Resting(back) = postcard::from_bytes(&bytes).unwrap() else {
            panic!("who rests came back as something else");
        };
        assert_eq!(back, [Posed { who: 7, pose: Pose::Lie }, Posed { who: u64::MAX, pose: Pose::Sit }]);
        // A whole channel of sixteen resting is a few bytes each.
        let all: Vec<Posed> = (0..16).map(|n| Posed { who: u64::MAX - n, pose: Pose::Lounge }).collect();
        assert!(postcard::to_allocvec(&Tell::Resting(all)).unwrap().len() < 200);
    }

    #[test]
    fn which_car_goes_over_the_wire_after_everything_an_app_from_before_knows() {
        // Appended after the last of those, which was who rests: nothing
        // an app from before sends or is sent reads as anything else.
        assert_eq!(postcard::to_allocvec(&Ask::Rest(None)).unwrap()[0], 30);
        assert_eq!(postcard::to_allocvec(&Ask::Cars).unwrap(), [31]);
        let bytes = postcard::to_allocvec(&Ask::Car("yang".into())).unwrap();
        assert_eq!(bytes[0], 32);
        assert!(matches!(postcard::from_bytes(&bytes).unwrap(), Ask::Car(car) if car == "yang"));
        let plots = postcard::to_allocvec(&Tell::Plots(Vec::new())).unwrap()[0];
        let cars = Tell::Cars(vec![Driver { who: 7, car: "yang".into() }, Driver { who: u64::MAX, car: bag::CAR.into() }]);
        let bytes = postcard::to_allocvec(&cars).unwrap();
        assert_eq!(bytes[0], plots + 1, "after whose the plots are, the last an app from before is told");
        let Tell::Cars(back) = postcard::from_bytes(&bytes).unwrap() else {
            panic!("which car came back as something else");
        };
        assert_eq!(back, [Driver { who: 7, car: "yang".into() }, Driver { who: u64::MAX, car: bag::CAR.into() }]);
        // A whole channel of 48 in their cars is under two kilobytes.
        let all: Vec<Driver> = (0..48).map(|n| Driver { who: u64::MAX - n, car: "square_house_roofless_nine".into() }).collect();
        assert!(postcard::to_allocvec(&Tell::Cars(all)).unwrap().len() < 2_000);
    }

    #[test]
    fn what_is_said_of_a_bag_goes_over_the_wire_last_of_all() {
        use bag::{Bag, LAMP, Standing};
        let mut bag = Bag::new();
        bag.put(1, LAMP, At::Ground { x: 3.25, y: 0.0, z: -4.5, yaw: 0.785 }).unwrap();
        // Appended last, after everything an app from before knows: a
        // variant's number is all postcard writes of which it is.
        let ask = Ask::Bag(bag::Ask::Put { id: 1, kind: LAMP.into(), at: bag.stands[0].at });
        let bytes = postcard::to_allocvec(&ask).unwrap();
        assert_eq!(bytes[0], 28, "Ask::Bag is the twenty-ninth thing a device asks");
        assert_eq!(postcard::to_allocvec(&Ask::Drive(true)).unwrap()[0], 27);
        assert_eq!(postcard::to_allocvec(&Ask::Channel).unwrap()[0], 26);
        let Ask::Bag(back) = postcard::from_bytes(&bytes).unwrap() else {
            panic!("an ask about a bag came back as something else");
        };
        assert_eq!(back, bag::Ask::Put { id: 1, kind: LAMP.into(), at: bag.stands[0].at });
        let paint = bag::Ask::Paint { id: 9, part: "blanket".into(), colour: [12, 200, 255] };
        for ask in [bag::Ask::Watch, bag::Ask::Buy("singlebed".into()), bag::Ask::Sell(LAMP.into()), bag::Ask::Take { id: 9 }, bag::Ask::Colours, paint] {
            let bytes = postcard::to_allocvec(&Ask::Bag(ask.clone())).unwrap();
            let Ask::Bag(back) = postcard::from_bytes(&bytes).unwrap() else {
                panic!("{ask:?} came back as something else");
            };
            assert_eq!(back, ask);
        }
        // What an app from before colours asks is numbered as it was, and
        // what came with them comes after: nothing an app from before
        // sends reads as anything else, and it is never sent the new.
        let number = |ask: bag::Ask| postcard::to_allocvec(&ask).unwrap()[0];
        assert_eq!(number(bag::Ask::Watch), 0);
        assert_eq!(number(bag::Ask::Take { id: 9 }), 4);
        assert_eq!(number(bag::Ask::Colours), 5);
        assert_eq!(number(bag::Ask::Paint { id: 9, part: "door".into(), colour: [0; 3] }), 6);
        assert_eq!(number(bag::Ask::Unpaint { id: 9, part: "door".into() }), 7);
        let unpaint = bag::Ask::Unpaint { id: 9, part: "door".into() };
        let bytes = postcard::to_allocvec(&Ask::Bag(unpaint.clone())).unwrap();
        assert!(matches!(postcard::from_bytes(&bytes).unwrap(), Ask::Bag(back) if back == unpaint));
        let painted = bag::Tell::Painted {
            who: 77,
            asked: 4,
            paints: vec![bag::Paint { id: 1, part: "lampostbody".into(), colour: [255, 0, 128] }],
        };
        let bytes = postcard::to_allocvec(&painted).unwrap();
        assert_eq!(bytes[0], 2, "after Yours and Theirs");
        assert_eq!(postcard::from_bytes::<bag::Tell>(&bytes).unwrap(), painted);
        assert_eq!(postcard::to_allocvec(&bag::Tell::Theirs { who: 1, stands: Vec::new() }).unwrap()[0], 1);
        // A plot painted as much as it may be is a few kilobytes too.
        let most: Vec<bag::Paint> = (0..bag::MOST_PAINTED as u32)
            .map(|n| bag::Paint { id: n + 1_000, part: "desktopedge".into(), colour: [200, 100, 50] })
            .collect();
        let bytes = postcard::to_allocvec(&Tell::Bag(bag::Tell::Painted { who: u64::MAX, asked: 0, paints: most })).unwrap();
        assert!(bytes.len() < 8_000, "{} bytes", bytes.len());
        let tell = Tell::Bag(bag::Tell::Yours { asked: 3, bag: bag.clone() });
        let bytes = postcard::to_allocvec(&tell).unwrap();
        assert_eq!(bytes[0], 19, "Tell::Bag is the twentieth thing a device is told");
        assert_eq!(postcard::to_allocvec(&Tell::Channel(Vec::new())).unwrap()[0], 18);
        let Tell::Bag(bag::Tell::Yours { asked, bag: back }) = postcard::from_bytes(&bytes).unwrap() else {
            panic!("a bag came back as something else");
        };
        assert_eq!((asked, &back), (3, &bag));
        let theirs = Tell::Bag(bag::Tell::Theirs { who: 77, stands: bag.stands.clone() });
        let Tell::Bag(bag::Tell::Theirs { who, stands }) = postcard::from_bytes(&postcard::to_allocvec(&theirs).unwrap()).unwrap() else {
            panic!("a plot came back as something else");
        };
        assert_eq!((who, stands), (77, bag.stands.clone()));
        // A plot as full as it may be is a few kilobytes: one message.
        let full: Vec<Standing> = (0..bag::MOST_STANDING as u32)
            .map(|id| Standing {
                id: id + 1_000,
                kind: "square_house_roofless_two_floor".into(),
                at: At::Ground { x: -11.5, y: 0.25, z: 11.5, yaw: 5.5 },
            })
            .collect();
        let bytes = postcard::to_allocvec(&Tell::Bag(bag::Tell::Theirs { who: u64::MAX, stands: full })).unwrap();
        assert!(bytes.len() < 6_000, "{} bytes", bytes.len());
    }
}
