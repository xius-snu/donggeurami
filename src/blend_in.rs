//! Blend In, on this device (since 2026-11-18; Hajun: "1 random hider
//! chosen as the person walking around pretending to be an NPC/AI. 1~3
//! other people trying to find who is the person, by punching with a aura
//! sword. This game min player count is 2, so if there 2 ppl, it will not
//! fill the 2 slot with 2 ai's but rather start game with just 2 real ppl.
//! There is a time limit, and incorrect guesses cost you time by
//! stunning/blinding etc. Both hider and seeker can Choose (must choose
//! within the timer of maybe 30s, autostart when both have chosen):
//! Guy/girl + skin of character, Name. Map = roundtownmap ... You swap until
//! every one has tried hiding atleast once ... (i think freecam is ok for
//! blend in mode, similar to main lobby camera)"): two to four people and
//! never the computer, each hiding once among a crowd of the town's people
//! while the others look for them, in the town as it stands with nobody of
//! its own in it (`island::Scenery`). Its rules are `roundtown_net::
//! blend_in`'s, and the server plays them: Blend In is online only, its
//! card on the selector blocked out offline (`games`), and its table is
//! `table`'s. This is what each device draws of it and sends of it.
//!
//! **What comes**: the room's [`BlendRound`] on the table's own entity
//! (which round, who hides, how far the round has got and until when by
//! the server's clock, everyone's points, who has chosen, the seekers'
//! bodies, and once a round is over who found the hider and which body was
//! theirs); and what the server tells this device alone ([`Told`]): what
//! you wear, that a name was no name, that you are stunned, that a seeker
//! struck one of the town's own. The crowd's bodies and everyone else's
//! come as everyone's do online, and are drawn as people are (`net`), each
//! with its name over its head (`tags`). What goes back: your look and name
//! for the coming hide, that you have chosen (`roundtown_net::blend_in::
//! Ask`), your moves, and your sword's swing as a punch on whoever it
//! landed on (`net::relay_hits`), which the server takes for a strike.
//!
//! **A round here**, as the game has it:
//!
//! * **Choosing**: a window (`choose`) says whether you hide or seek, with
//!   your look between two arrows, your name, a dice that throws one of the
//!   town's names, the time left and Ready; the players with a tick by each
//!   who has chosen. You stand still; the chat may be used.
//! * **The hide**: the hider is told across the screen to walk like the
//!   town's people and stop like them; a seeker's screen is black ("The
//!   hider is getting into place", the seconds, and what a wrong swing
//!   will cost) and they cannot move, stood facing out from the pond.
//! * **The seek**: the seekers run ([`BlendRound::pace`]), the aura sword in
//!   their hands for everyone to see and a faint glow on the ground round
//!   them, the punch's range drawn under the one who is you
//!   (`range::Shows`), the sword where the town has its fist. A wrong guess
//!   blinds the seeker but for a small circle round them (`gear`'s
//!   blindfold) and holds them where they stand, "Wrong!" counting down;
//!   everyone's chat says so. The chat and the quick chat are hushed from
//!   the hide to the end of the seek ([`Hushed`]).
//! * **Over**: a column of light over the hider where they stood, an arrow
//!   at the screen's edge toward it while it is off the screen, and who
//!   found them in how long, or that they stayed hidden, and who they went
//!   by; and at the end who wins, and the table's results.
//!
//! Top centre a plate says what is going on and the time; top right the
//! points; top left your name, and while you are dressed as someone else
//! that name, with "(you)" under it (`screen`). A dashed line on the
//! ground marks the ring past which nobody walks, a light rising off it as
//! you come near (`light`), and your body is kept inside it
//! ([`keep_inside`]).

mod choose;
mod light;
mod screen;

use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use bevy_replicon::prelude::Remote;
use roundtown_net::blend_in::{self, BlendRound, BlendStage, RING, WALK_SPEED};
use roundtown_net::{Ask as NetAsk, Member, Minigame, Venue};

use crate::arena::Wears;
use crate::chat::{self, Hushed};
use crate::clock::TownClock;
use crate::hud;
use crate::range::Shows;
use crate::signup::Profile;
use crate::table::{self, AtTable, InTable, TableView};
use crate::ui::{fill, ui};
use crate::{Frozen, Intent, PLAYER_RADIUS, Player, Team, WalkCycle};

/// Something the server says to this device alone in its game of Blend In
/// (`roundtown_net::blend_in::Tell`), handed on by `net`.
#[derive(Message, Clone, Debug)]
pub(crate) struct Told(pub roundtown_net::blend_in::Tell);

/// The team every seeker's body is on, on this device, from the hide to the
/// end of the seek (mine): a swing never lands on a body of your own team
/// (`start_punch`), so that a seeker's sword passes another seeker by, and
/// lands on whoever is behind them (the design: "a seeker is not hit by
/// another's swing"). Never shown: nothing of Blend In draws a team.
const SEEKERS: u8 = 0;

/// What the sword is that seekers hold: the aura sword, Bridge Wars' third
/// (Hajun: "by punching with a aura sword").
const AURA_SWORD: u8 = 3;

/// How far in from the ring your body is kept, in metres: a body's width,
/// so that nobody is ever seen past the line on the ground.
const KEPT_IN: f32 = PLAYER_RADIUS;

/// How long a headline is across the screen, in seconds: as the four games'
/// names are said (`table`), and longer for the end of a round, which the
/// round shows for its own while ([`blend_in::SHOWN`]); and the game's end,
/// shorter, gone by the time the table's results come up over it two
/// seconds after the end (mine: at three it showed through their
/// see-through window), and taken down as they come all the same
/// ([`clear_for_results`]).
const HEADLINE_SECS: f32 = 3.0;
const SHOWN_SECS: f32 = 4.5;
const END_SECS: f32 = 2.0;

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Told>()
        .init_resource::<Blend>()
        .add_systems(
            Update,
            (begin_afresh, hear, follow, hold, arm, dress_me)
                .chain()
                .after(table::TableSystems)
                .before(crate::read_player_input)
                .before(crate::island::show_where_i_am),
        )
        .add_systems(Update, clear_for_results.after(table::TableSystems).after(follow))
        .add_systems(Update, pace.after(crate::read_player_input).before(crate::move_bodies))
        .add_systems(Update, keep_inside.after(crate::move_bodies).before(crate::animate_walk));
    choose::plugin(app);
    screen::plugin(app);
    light::plugin(app);
}

/// The buttons round the screen at a table of Blend In, the town's less what
/// does not belong in a game (Hajun's town layout; no gamepad, no friends,
/// no car, no balance): top left your name on its plate, and under it, while
/// you are dressed as someone else, that name with "(you)" (`screen`); top
/// right the points, once the game is played; round the town's own jump
/// button the speech bubble's things to say and the chat's keyboard, where
/// the town has them; and where the town has its fist, the sword, shown
/// only while you seek (the hider has no swing).
pub(crate) fn spawn_bar(commands: &mut Commands, assets: &AssetServer, images: &mut Assets<Image>) {
    let (left, right) = hud::game_bar(commands);
    screen::plates(commands, left);
    screen::scores(commands, right);
    let round = commands.spawn(hud::round_the_jump()).id();
    let (quick_right, quick_bottom, quick_size) = hud::QUICK_BUTTON;
    commands.spawn((
        ChildOf(round),
        hud::QuickChatButton,
        ForTheChat,
        hud::round_button(assets.load("icons/social.png"), quick_size, hud::beside_the_jump(quick_right, quick_bottom)),
    ));
    // The chat's keyboard, while lines may be typed (not since 2026-11-27:
    // the speech bubble beside it is all there is to say, `quick`).
    if roundtown_net::chat::TYPED {
        commands.spawn((ChildOf(round), ForTheChat, hud::chat_button(images)));
    }
    let (right_of, bottom) = FIST;
    let sword = images.add(hud::sword());
    let reverse = images.add(hud::jump_arrow());
    commands.spawn((
        ChildOf(round),
        SwordButton,
        Visibility::Hidden,
        hud::punch_button(sword, reverse, hud::ACTION_BUTTON, hud::beside_the_jump(right_of, bottom)),
    ));
}

/// Where the town has its fist round the jump button, which is where the
/// seekers' sword is: from the box's right and bottom, in shares of the
/// short side (`hud`'s town bar).
const FIST: (f32, f32) = (24.3, 1.2);

/// The sword round the jump button, which swings it: shown while you seek.
#[derive(Component)]
struct SwordButton;

/// Something the game has said across the screen (`hud::announce`), while
/// it is up: the next takes its place.
#[derive(Component)]
struct Headline;

/// The speech bubble and the chat's keyboard on Blend In's bar, put away
/// while the chat is hushed (`screen`).
#[derive(Component)]
struct ForTheChat;

/// What its score is called among the results: the file's
/// (`words.blend_in.score`).
pub(crate) fn score_word() -> String {
    ui().words.blend_in.score.clone()
}

/// What the line under the results says of what the whole table made:
/// Blend In counts nothing together.
pub(crate) fn total_words(_total: u32) -> Option<String> {
    None
}

/// Blend In as this device has it, at a table of it: what the room was
/// last seen to be sent of the game, and what this device has been told of
/// itself in it.
#[derive(Resource, Default)]
struct Blend {
    /// Your seat at the table, while you are at one of Blend In.
    seat: Option<usize>,
    /// The round as the room was last sent it, once the play has begun.
    round: Option<BlendRound>,
    /// What the server last said you wear, or will from the coming hide
    /// (`Tell::Wear`): a name, and a look.
    offered: Option<(String, String)>,
    /// What your body is dressed as, since the first hide: the name on your
    /// plate and the look of your model (what everyone else sees you as).
    worn: Option<(String, String)>,
    /// Until when you are stunned, by the server's clock, if you have been.
    stunned: Option<f64>,
    /// The name you asked for was turned down, and nothing chosen since.
    refused: bool,
    /// Whether the sword and the seekers' team are on your body, put there
    /// by this game, to take off as it goes.
    me_armed: bool,
    /// The round whose choosing window you put away with its cross: it is
    /// not up again until the next round's. At this table only: a table
    /// begun afresh (Play again, another game) has none put away.
    shut: Option<u8>,
    /// How many of the choices you made this choosing (`Ask::Disguise`)
    /// the server has yet to answer, one answer to each (`Tell::Wear`):
    /// while any is still out, an answer is to a choice made before the
    /// last, and says what you have chosen over since. Mine: shown, it put
    /// the window back on an older choice, and a tap made then stepped on
    /// from that one and was lost.
    asked: u32,
    /// Whose round was given up just before this one, by the name they sit
    /// at the table under: said in this round's window (`choose`), which
    /// comes up as it is, and where a headline would stand over it.
    given_up: Option<String>,
    /// Whether you have been told the game is played inside the ring, as
    /// you first walked into it at this table ([`keep_inside`]).
    ringed: bool,
}

impl Blend {
    /// The round, while one is being played: not before the play, nor once
    /// the game is over.
    fn playing(&self) -> Option<&BlendRound> {
        self.round.as_ref().filter(|round| round.stage != BlendStage::Over)
    }

    /// Whether you hide this round.
    fn hiding(&self) -> bool {
        self.seat.zip(self.playing()).is_some_and(|(seat, round)| usize::from(round.hider) == seat)
    }

    /// Whether you seek this round, from the hide to the end of the seek:
    /// the aura sword in your hand.
    fn armed(&self) -> bool {
        self.playing()
            .is_some_and(|round| matches!(round.stage, BlendStage::Hide | BlendStage::Seek))
            && !self.hiding()
    }

    /// Whether you are stunned at `now`, by the server's clock: a wrong
    /// guess's, which ends with its countdown or with the seek.
    fn stunned_at(&self, now: f64) -> bool {
        self.stunned.is_some_and(|until| now < until)
            && self.playing().is_some_and(|round| round.stage == BlendStage::Seek)
            && !self.hiding()
    }
}

/// A table of Blend In begun on this device, or left: whatever was had of a
/// game before goes, and you are yourself again. The same table begun
/// afresh (your seat moved before the play, or this device put back in the
/// game it was in) keeps what it had of the game, which the room is sent
/// as it stands, with your seat as it now is.
fn begin_afresh(
    mut entered: MessageReader<table::Entered>,
    mut gone: MessageReader<table::Gone>,
    mut blend: ResMut<Blend>,
    me: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    let mut away = gone.read().count() > 0;
    let mut here = None;
    for at in entered.read() {
        if at.game == Minigame::BlendIn {
            here = Some((usize::from(at.seat), at.same_table));
        } else {
            here = None;
            away = true;
        }
    }
    let you = me.single().ok();
    match here {
        // (Its connection made again, or its seat moved: an answer to a
        // choice it made may have been lost on the way.)
        Some((seat, true)) if blend.seat.is_some() => {
            blend.seat = Some(seat);
            blend.asked = 0;
        }
        Some((seat, _)) => {
            let_go(&mut blend, you, &mut commands);
            blend.seat = Some(seat);
        }
        None if away && blend.seat.is_some() => let_go(&mut blend, you, &mut commands),
        None => {}
    }
}

/// Away from a game of Blend In: nothing of it stays on you.
fn let_go(blend: &mut Blend, you: Option<Entity>, commands: &mut Commands) {
    if let (Some(you), true) = (you, blend.me_armed) {
        commands.entity(you).try_remove::<(Wears, Shows, Team)>();
    }
    *blend = Blend::default();
}

/// What the server says to this device alone in the game: what you wear,
/// that a name was no name, that you are stunned, and, for the chat, that a
/// seeker struck one of the town's own.
fn hear(
    mut told: MessageReader<Told>,
    mut blend: ResMut<Blend>,
    view: Res<TableView>,
    mut log: ResMut<chat::Log>,
) {
    for Told(said) in told.read() {
        match said {
            blend_in::Tell::Wear { name, look, now } => {
                let wearing = (name.clone(), look.clone());
                blend.asked = blend.asked.saturating_sub(1);
                // Dressed as it now, from the hide to the end of the round
                // shown, as the server says: what everyone sees you as.
                // Otherwise what you will wear from the coming hide, and you
                // go on as you were until then. (In the first build the
                // device guessed which by the part of the round it had last
                // followed, and the next round's offer, which comes as the
                // round before is still shown, or given up in its seek, was
                // put on at once: your plate and your body the offer, and a
                // choice made in the window not worn until the hide.)
                let dressed = *now;
                if dressed {
                    blend.worn = Some(wearing.clone());
                }
                // While choosing, the answer to a choice you have made
                // another since: the window keeps the newer, which the
                // answer to it will say.
                if blend.asked > 0 && !dressed {
                    continue;
                }
                if blend.offered.as_ref() != Some(&wearing) {
                    blend.offered = Some(wearing);
                }
            }
            blend_in::Tell::Refused => blend.refused = true,
            blend_in::Tell::Stunned { until } => blend.stunned = Some(*until),
            blend_in::Tell::Struck { seeker } => {
                let file = ui();
                let colours = file.colours.chat;
                let name = name_of(&view, usize::from(*seeker));
                log.push(chat::spans(&file.words.blend_in.struck, colours.event, &[("name", &name, colours.name)]));
            }
        }
    }
}

/// The name the person in `seat` goes by at the table: their account's,
/// as the room is told who sits where.
fn name_of(view: &TableView, seat: usize) -> String {
    view.members.get(seat).map(|member| member.name.clone()).unwrap_or_default()
}

/// Whether everyone is dressed as they chose at `stage` of a round: from
/// its hide to the end of its showing.
fn dressed(stage: BlendStage) -> bool {
    matches!(stage, BlendStage::Hide | BlendStage::Seek | BlendStage::Shown)
}

/// Something that has come about in the game, from one round the room was
/// sent to the next ([`moments`]).
#[derive(Clone, PartialEq, Debug)]
enum Moment {
    /// A round's choosing has begun.
    Choose,
    /// The hide has begun.
    Hide,
    /// The seek has begun.
    Seek,
    /// The round is over: the hider found by the seat, so many seconds into
    /// the seek, or not found.
    Shown { found: Option<(usize, u16)> },
    /// The hider's round was given up, with no point: they left, or their
    /// device was gone too long.
    Void { hider: usize },
    /// The game is over.
    Over,
}

/// What came about from the round `before` (none, before the play) to
/// `now`, in the order it came about.
fn moments(before: Option<&BlendRound>, now: &BlendRound) -> Vec<Moment> {
    let mut came = Vec::new();
    let Some(before) = before else {
        // Seen first part way through (a device put back in the game): only
        // what is going on now.
        came.push(match now.stage {
            BlendStage::Choose => Moment::Choose,
            BlendStage::Hide => Moment::Hide,
            BlendStage::Seek => Moment::Seek,
            BlendStage::Shown => Moment::Shown { found: now.found.map(|(seat, after)| (usize::from(seat), after)) },
            BlendStage::Over => Moment::Over,
        });
        return came;
    };
    let new_round = now.round != before.round;
    if before.stage == now.stage && !new_round {
        return came;
    }
    // Gone on from the middle of a round without its being shown: given up.
    let cut_short = matches!(before.stage, BlendStage::Choose | BlendStage::Hide | BlendStage::Seek)
        && (new_round || now.stage == BlendStage::Over);
    if cut_short && new_round {
        came.push(Moment::Void { hider: usize::from(before.hider) });
    }
    match now.stage {
        BlendStage::Choose => came.push(Moment::Choose),
        BlendStage::Hide => came.push(Moment::Hide),
        BlendStage::Seek => came.push(Moment::Seek),
        BlendStage::Shown => came.push(Moment::Shown { found: now.found.map(|(seat, after)| (usize::from(seat), after)) }),
        BlendStage::Over => came.push(Moment::Over),
    }
    came
}

/// The headline at the end, from the place each seat came (0 first): one
/// who came first wins; several first share it; everyone first alike, a
/// draw.
fn winners_words(places: &[u8], names: &[String]) -> String {
    let file = ui();
    let words = &file.words.blend_in;
    let first: Vec<&str> = places
        .iter()
        .zip(names)
        .filter(|(place, _)| **place == 0)
        .map(|(_, name)| name.as_str())
        .collect();
    if first.len() == places.len() || first.is_empty() {
        words.draw.clone()
    } else if let [name] = first[..] {
        fill(&words.wins, &[("name", name)])
    } else {
        fill(&words.win_together, &[("names", &first.join(&words.and))])
    }
}

/// The places `points` would give, by seat, 0 first and those alike tied:
/// what the end is said by should the table's results not be here yet
/// (they come with it), which place whoever left after everyone who
/// stayed (`blend_in::Game::outcome`).
fn by_points(points: &[u8]) -> Vec<u8> {
    points
        .iter()
        .map(|&mine| points.iter().filter(|&&theirs| theirs > mine).count() as u8)
        .collect()
}

/// Follows the round as the room is sent it: what has come about said
/// across the screen and in the chat, as each part of a round begins and
/// ends.
#[allow(clippy::too_many_arguments)]
fn follow(
    at: Res<AtTable>,
    view: Res<TableView>,
    rounds: Query<&BlendRound, With<Remote>>,
    bodies: Query<&Member, With<Remote>>,
    mut blend: ResMut<Blend>,
    mut log: ResMut<chat::Log>,
    headlines: Query<Entity, With<Headline>>,
    mut commands: Commands,
) {
    if at.of(Minigame::BlendIn).is_none() || blend.seat.is_none() {
        return;
    }
    let Some(now) = view.shown.and_then(|shown| rounds.get(shown).ok()) else {
        return;
    };
    if blend.round.as_ref() == Some(now) {
        return;
    }
    let came = moments(blend.round.as_ref(), now);
    let before = blend.round.replace(now.clone());
    // Seen first part way through a round, this device begun afresh in the
    // game (its app closed and opened again): dressed as it was told it
    // wears, if that came first; should it come after, it dresses you
    // itself (`hear`).
    if before.is_none()
        && dressed(now.stage)
        && let Some(offered) = blend.offered.clone()
    {
        blend.worn = Some(offered);
    }
    let file = ui();
    let words = &file.words.blend_in;
    let colours = file.colours.chat;
    let seat = blend.seat.unwrap_or(0);
    let hider = usize::from(now.hider);
    let names: Vec<String> = (0..now.points.len()).map(|seat| name_of(&view, seat)).collect();
    let said = |template: &str, fills: &[(&str, &str)]| -> chat::Line {
        let fills: Vec<(&str, &str, Color)> = fills.iter().map(|&(key, words)| (key, words, colours.name)).collect();
        chat::spans(template, colours.event, &fills)
    };
    // Each headline of the game's in place of the one before, if that is
    // still up: two at once would be one over the other. None for a round
    // seen first part way through, on a device begun afresh in the game
    // (mine): the table says the game's name across the screen then, which
    // the game's own would stand over, and the plate top centre says what
    // is going on.
    let fresh = before.is_none();
    let mut up: Vec<Entity> = headlines.iter().collect();
    let mut announce = |commands: &mut Commands, caption: Option<&str>, title: &str, size: f32, secs: f32| {
        if fresh {
            return;
        }
        for old in up.drain(..) {
            commands.entity(old).try_despawn();
        }
        let new = hud::announce(commands, caption, title, size, secs).insert((InTable, Headline)).id();
        up.push(new);
    };
    let mut given_up = None;
    for moment in came {
        match moment {
            Moment::Choose => {
                blend.refused = false;
                blend.stunned = None;
                blend.asked = 0;
                // Said in the window this round's choosing comes up in,
                // if the round before was given up.
                blend.given_up = given_up.take();
            }
            Moment::Hide => {
                // What you chose is what you are dressed as from now, if
                // the word of it is here; it is told again as you are.
                blend.asked = 0;
                blend.given_up = None;
                if let Some(offered) = blend.offered.clone() {
                    blend.worn = Some(offered);
                }
                if hider == seat {
                    announce(&mut commands, Some(&words.hide_caption), &words.hide_title, hud::HEADLINE, HEADLINE_SECS);
                }
            }
            Moment::Seek => {
                let (title, caption) = if hider == seat {
                    (&words.hidden_title, &words.hidden_caption)
                } else {
                    (&words.seek_title, &words.seek_caption)
                };
                announce(&mut commands, Some(caption), title, hud::HEADLINE, HEADLINE_SECS);
            }
            Moment::Shown { found } => {
                let hidden = names.get(hider).cloned().unwrap_or_default();
                // Who they went by, so that a seeker who never saw them
                // has the answer (mine, since a play-tester found the end
                // of a round named only the account): their own word of
                // it, or the name over the body the room is told was
                // theirs.
                let guise = if hider == seat {
                    blend.worn.as_ref().map(|(name, _)| name.clone())
                } else {
                    now.unmasked.and_then(|id| bodies.iter().find(|body| body.id == id)).map(|body| body.name.clone())
                };
                let (headline, line) = match (found, &guise) {
                    (Some((by, after)), _) => {
                        let time = table::clock(f32::from(after));
                        let finder = names.get(by).cloned().unwrap_or_default();
                        let line = match &guise {
                            Some(guise) => said(
                                &words.found_as_line,
                                &[("seeker", &finder), ("hider", &hidden), ("guise", guise), ("time", &time)],
                            ),
                            None => said(&words.found_line, &[("seeker", &finder), ("hider", &hidden), ("time", &time)]),
                        };
                        (fill(&words.found, &[("name", &finder), ("time", &time)]), line)
                    }
                    (None, Some(guise)) => (
                        fill(&words.stayed, &[("name", &hidden)]),
                        said(&words.stayed_as_line, &[("name", &hidden), ("guise", guise)]),
                    ),
                    (None, None) => (fill(&words.stayed, &[("name", &hidden)]), said(&words.stayed_line, &[("name", &hidden)])),
                };
                let caption = guise.as_ref().map(|guise| fill(&words.hid_as, &[("name", &hidden), ("guise", guise)]));
                announce(&mut commands, caption.as_deref(), &headline, hud::HEADLINE, SHOWN_SECS);
                log.push(line);
            }
            // In the chat, and in the window of the round that comes up as
            // it is: never across the screen, where it stood over that
            // window (in the first build it did).
            Moment::Void { hider } => {
                let name = names.get(hider).cloned().unwrap_or_default();
                log.push(said(&words.void_line, &[("name", &name)]));
                given_up = Some(name);
            }
            Moment::Over => {
                // By the places the table pays by, which put whoever left
                // after everyone who stayed; they come with the end.
                let places = view.results().map_or_else(|| by_points(&now.points), |results| results.places.clone());
                let headline = winners_words(&places, &names);
                announce(&mut commands, None, &headline, hud::HEADLINE, END_SECS);
                info!("Blend In: over, points {:?}, places {places:?}", now.points);
            }
        }
    }
    if let Some(before) = before
        && before.stage != now.stage
    {
        debug!("Blend In: {:?} to {:?}, round {}", before.stage, now.stage, now.round);
    }
}

/// Takes down whatever the game has said across the screen as the table's
/// results come up (`table::ResultsWindow`): the window is see-through,
/// and the end's headline showed through it.
fn clear_for_results(
    results: Query<(), Added<table::ResultsWindow>>,
    headlines: Query<Entity, With<Headline>>,
    mut commands: Commands,
) {
    if results.is_empty() {
        return;
    }
    for headline in &headlines {
        commands.entity(headline).try_despawn();
    }
}

/// Holds you and the chat as the game is: you stand still while choosing,
/// as a seeker through the hide, and while a wrong guess has you stunned
/// (blinded too, `screen`); and the chat is hushed from the hide to the end
/// of the seek. Each written only as it changes. Blend In takes none of
/// Bridge Wars' fight state (`arena::Fighting`, `Outfit`): nothing of the
/// kit's is had in it but the sword in a seeker's hand and the punch's
/// range under your feet, each on a body of its own (`arm`).
fn hold(
    blend: Res<Blend>,
    town: Res<TownClock>,
    mut hushed: ResMut<Hushed>,
    me: Query<(Entity, Option<&Frozen>), With<Player>>,
    mut commands: Commands,
) {
    let Some(round) = blend.playing() else {
        // (Over, or away: the table lets go of the chat, your body and the
        // camera as it ends; over, they are let go of here, once.)
        if blend.round.is_some() {
            if let Ok((you, Some(_))) = me.single() {
                commands.entity(you).try_remove::<Frozen>();
            }
            hushed.set_if_neq(Hushed(false));
        }
        return;
    };
    let stunned = blend.stunned_at(town.now());
    let held = match round.stage {
        BlendStage::Choose => true,
        BlendStage::Hide => !blend.hiding(),
        BlendStage::Seek => stunned,
        BlendStage::Shown | BlendStage::Over => false,
    };
    if let Ok((you, frozen)) = me.single() {
        match (held, frozen) {
            (true, None) => {
                commands.entity(you).insert(Frozen { placed: false });
            }
            (false, Some(_)) => {
                commands.entity(you).try_remove::<Frozen>();
            }
            _ => {}
        }
    }
    hushed.set_if_neq(Hushed(matches!(round.stage, BlendStage::Hide | BlendStage::Seek)));
}

/// Puts the aura sword in every seeker's hand and the punch's range under
/// your feet while you seek, and on every seeker's body the seekers' team,
/// which a swing passes by ([`SEEKERS`]): yours as you seek, and every body
/// the room is told is a seeker's ([`BlendRound::swords`]); and takes them
/// off everyone else. Each written only as it changes.
#[allow(clippy::type_complexity)]
fn arm(
    mut blend: ResMut<Blend>,
    me: Query<(Entity, Option<&Wears>, Option<&Team>, Has<Shows>), With<Player>>,
    others: Query<(Entity, &Member, Option<&Wears>, Option<&Team>), (With<Remote>, With<WalkCycle>)>,
    mut swords: Query<&mut Visibility, With<SwordButton>>,
    mut commands: Commands,
) {
    // Away from Blend In nothing is touched: anything on a body is another
    // game's own.
    if blend.seat.is_none() {
        return;
    }
    let swords_out: Vec<u64> = blend
        .playing()
        .filter(|round| matches!(round.stage, BlendStage::Hide | BlendStage::Seek))
        .map(|round| round.swords.iter().copied().filter(|&id| id != 0).collect())
        .unwrap_or_default();
    let armed = blend.armed();
    let seek = blend.playing().is_some_and(|round| round.stage == BlendStage::Seek);
    let seeking = armed && seek;
    if let Ok((you, wears, team, shows)) = me.single() {
        put(&mut commands, you, armed, seek, wears, team);
        if seeking && !shows {
            commands.entity(you).insert(Shows);
        } else if !seeking && shows {
            commands.entity(you).try_remove::<Shows>();
        }
        if blend.me_armed != armed {
            blend.me_armed = armed;
        }
    }
    // The sword's button from the seek alone: in the hide, on the black
    // screen, a swing does nothing (the server takes strikes in the seek
    // only), and a button that does nothing is no button (mine, since a
    // play-tester pressed it there). The sword is in your hand from the
    // hide all the same, for the hider to see.
    let shown = if seeking { Visibility::Inherited } else { Visibility::Hidden };
    for mut button in &mut swords {
        button.set_if_neq(shown);
    }
    for (body, member, wears, team) in &others {
        put(&mut commands, body, swords_out.contains(&member.id), seek, wears, team);
    }
}

/// Puts the aura sword and the seekers' team on `body` if it is `armed`,
/// and in the `seek` the speed potion's effect, which leaves its blue
/// sparks behind them (`trail`; since 2026-11-18, Hajun: "the seeker should
/// get speed boost potion effect"; their pace is [`pace`]'s); and takes off
/// whatever of them it has if not: only what is not so already.
fn put(commands: &mut Commands, body: Entity, armed: bool, seek: bool, wears: Option<&Wears>, team: Option<&Team>) {
    let armed_with = Wears {
        sword: AURA_SWORD,
        fast: seek,
        ..default()
    };
    if armed {
        if wears != Some(&armed_with) {
            commands.entity(body).insert(armed_with);
        }
        if team != Some(&Team(SEEKERS)) {
            commands.entity(body).insert(Team(SEEKERS));
        }
    } else {
        if wears.is_some() {
            commands.entity(body).try_remove::<Wears>();
        }
        if team.is_some() {
            commands.entity(body).try_remove::<Team>();
        }
    }
}

/// Dresses your own body as you are seen: from the first hide on, in the
/// look you wear (everyone else is sent it, and sees you so); and in your
/// own once the game is over, or you are away from it, once. Your name
/// over nothing changes: it is never over your own head (`tags`), and your
/// plate says who you are dressed as (`screen`).
fn dress_me(
    blend: Res<Blend>,
    profile: Res<Profile>,
    assets: Res<AssetServer>,
    mut me: Query<&mut WorldAssetRoot, With<Player>>,
    mut was: Local<Option<String>>,
) {
    let wanted = blend
        .worn
        .as_ref()
        .filter(|_| blend.playing().is_some())
        .map(|(_, look)| look.clone());
    if *was == wanted {
        return;
    }
    let look = wanted.clone().unwrap_or_else(|| crate::lobby::me(&profile).look);
    if let Ok(mut body) = me.single_mut() {
        body.set_if_neq(crate::lobby::dressed_as(&assets, &look));
    }
    *was = wanted;
}

/// While you seek, you run ([`BlendRound::pace`], the shared rule:
/// [`SEEKER_PACE`](blend_in::SEEKER_PACE)), as fast as Bridge Wars' speed
/// potion runs you (since 2026-11-18; a plain run until then): your walk
/// this frame that much faster, and your legs with it. The hider walks as
/// the crowd does, and everyone else in any other part of a round.
fn pace(blend: Res<Blend>, mut me: Query<(&mut Intent, &mut WalkCycle), With<Player>>) {
    let (Some(seat), Some(round)) = (blend.seat, blend.playing()) else {
        return;
    };
    let quicker = round.pace(seat) / WALK_SPEED;
    if quicker == 1.0 {
        return;
    }
    let Ok((mut intent, mut walk)) = me.single_mut() else {
        return;
    };
    if intent.walk == Vec3::ZERO || intent.pace.is_some() {
        return;
    }
    intent.sprint *= quicker;
    walk.speed *= quicker;
}

/// Keeps your body inside the ring ([`RING`]): nobody walks past it, and
/// the server would put you back anyway. Walking out at it, you stand at
/// it, a body's width inside the line on the ground, and the first time at
/// a table your chat says why (mine: a wall nobody can see is no wall; the
/// light rising off the line as you come near it shows where, `light`).
/// Written only if you were past it.
fn keep_inside(
    mut blend: ResMut<Blend>,
    mut log: ResMut<chat::Log>,
    mut me: Query<(&mut Transform, &Venue), With<Player>>,
) {
    if blend.seat.is_none() {
        return;
    }
    let Ok((mut body, &here)) = me.single_mut() else {
        return;
    };
    if here != Venue::BlendIn {
        return;
    }
    let across = body.translation.xz();
    let most = RING - KEPT_IN;
    if across.length() <= most {
        return;
    }
    let inside = across.clamp_length_max(most);
    body.translation.x = inside.x;
    body.translation.z = inside.y;
    if !blend.ringed {
        blend.ringed = true;
        let file = ui();
        log.push(chat::spans(&file.words.blend_in.ring_line, file.colours.chat.event, &[]));
    }
}

/// What a wrong swing costs a seeker, in the file's words
/// (`words.blend_in.stun_rule`): the stun of a round's first wrong guess,
/// its second and its third, in seconds, as the rules have them
/// ([`STUN_FIRST`](blend_in::STUN_FIRST), [`STUN_MORE`](blend_in::STUN_MORE)),
/// so that the words never say other than the game does. On a seeker's
/// window and on their black screen through the hide (mine, since a
/// play-tester found a first-time seeker's first word of it was "Wrong!
/// 4", and nothing said the next would be 7).
fn stun_rule() -> String {
    let nth = |guess: u8| (blend_in::STUN_FIRST + blend_in::STUN_MORE * f32::from(guess)).round().to_string();
    fill(&ui().words.blend_in.stun_rule, &[("first", &nth(0)), ("second", &nth(1)), ("third", &nth(2))])
}

/// Asks the server something of the game, as this device's own word
/// (`net`).
fn ask(asks: &mut MessageWriter<NetAsk>, ask: blend_in::Ask) {
    asks.write(NetAsk::BlendIn(ask));
}

/// Words that wrap at the edge of the node they are in, from the left: for
/// a line on a window too long for one line.
fn wrapped(text: impl Into<String>, size: f32, colour: Color) -> impl Bundle {
    (
        Text::new(text),
        crate::font::sized(size),
        TextColor(colour),
        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
        Pickable::IGNORE,
    )
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    fn round(round: u8, stage: BlendStage) -> BlendRound {
        BlendRound {
            round,
            rounds: 3,
            hider: round,
            stage,
            until: 0.0,
            points: vec![0; 3],
            ready: vec![false; 3],
            found: None,
            unmasked: None,
            swords: Vec::new(),
        }
    }

    #[test]
    fn each_part_of_a_round_comes_about_once_in_the_order_it_does() {
        let choose = round(0, BlendStage::Choose);
        assert_eq!(moments(None, &choose), vec![Moment::Choose]);
        assert_eq!(moments(Some(&choose), &choose), vec![], "nothing new");
        let mut readier = choose.clone();
        readier.ready[1] = true;
        assert_eq!(moments(Some(&choose), &readier), vec![], "a tick is no new part");
        let hide = round(0, BlendStage::Hide);
        assert_eq!(moments(Some(&choose), &hide), vec![Moment::Hide]);
        let seek = round(0, BlendStage::Seek);
        assert_eq!(moments(Some(&hide), &seek), vec![Moment::Seek]);
        let mut found = round(0, BlendStage::Shown);
        found.found = Some((2, 72));
        assert_eq!(moments(Some(&seek), &found), vec![Moment::Shown { found: Some((2, 72)) }]);
        let next = round(1, BlendStage::Choose);
        assert_eq!(moments(Some(&found), &next), vec![Moment::Choose]);
        // A hider gone in the middle of the seek: their round given up, and
        // the next begins.
        assert_eq!(moments(Some(&seek), &next), vec![Moment::Void { hider: 0 }, Moment::Choose]);
        // Too few left: over.
        assert_eq!(moments(Some(&seek), &round(0, BlendStage::Over)), vec![Moment::Over]);
        let last = round(2, BlendStage::Shown);
        assert_eq!(moments(Some(&last), &round(2, BlendStage::Over)), vec![Moment::Over]);
        // Put back part way through: only what is going on.
        assert_eq!(moments(None, &seek), vec![Moment::Seek]);
    }

    #[test]
    fn the_end_names_whoever_came_first_or_says_it_is_a_draw() {
        let names: Vec<String> = ["Dubu", "Mochi", "Hodu"].iter().map(|name| name.to_string()).collect();
        let file = ui();
        let words = &file.words.blend_in;
        assert_eq!(winners_words(&by_points(&[2, 1, 0]), &names), fill(&words.wins, &[("name", "Dubu")]));
        assert_eq!(winners_words(&by_points(&[1, 1, 1]), &names), words.draw);
        assert_eq!(winners_words(&by_points(&[0, 0]), &names[..2]), words.draw, "neither found the other");
        let two = winners_words(&by_points(&[1, 1, 0]), &names);
        assert_eq!(two, fill(&words.win_together, &[("names", &format!("Dubu{}Mochi", words.and))]));
        // At none each, the one who left placed after the one who stayed:
        // no draw.
        assert_eq!(by_points(&[0, 0]), vec![0, 0]);
        assert_eq!(winners_words(&[1, 0], &names[..2]), fill(&words.wins, &[("name", "Mochi")]));
    }

    /// A device at a table of Blend In, online, in seat 0 of two, with the
    /// game's systems that hear what it is told and follow the round.
    fn at_the_table() -> World {
        let mut world = World::new();
        world.init_resource::<Blend>();
        world.init_resource::<TableView>();
        world.init_resource::<chat::Log>();
        world.init_resource::<Messages<Told>>();
        world.init_resource::<Messages<table::Entered>>();
        world.init_resource::<Messages<table::Gone>>();
        world.insert_resource(AtTable(Some(table::Sitting {
            game: Minigame::BlendIn,
            seat: 0,
            online: true,
            id: 3,
        })));
        let members = ["Dubu", "Mochi"]
            .iter()
            .enumerate()
            .map(|(seat, name)| Member {
                id: seat as u64 + 1,
                name: (*name).into(),
                look: "boy".into(),
                ai: false,
            })
            .collect();
        world.resource_mut::<TableView>().members = members;
        world.resource_mut::<Blend>().seat = Some(0);
        world
    }

    /// The room's round, as it comes to the device: `stage` of round 0, seat 1 hiding.
    fn sent(world: &mut World, stage: BlendStage) {
        let mut now = round(0, stage);
        now.hider = 1;
        let shown = world.resource::<TableView>().shown;
        match shown {
            Some(entity) => {
                world.entity_mut(entity).insert(now);
            }
            None => {
                let entity = world.spawn((Remote, now)).id();
                world.resource_mut::<TableView>().shown = Some(entity);
            }
        }
    }

    fn told(world: &mut World, said: blend_in::Tell) {
        world.write_message(Told(said));
        world.run_system_once(hear).expect("heard");
        world.resource_mut::<Messages<Told>>().clear();
    }

    /// What the server says you will wear from the coming hide: as the
    /// choosing begins, and in answer to each choice.
    fn wear(name: &str, look: &str) -> blend_in::Tell {
        blend_in::Tell::Wear {
            name: name.into(),
            look: look.into(),
            now: false,
        }
    }

    /// What the server says you are dressed in now: as the hide begins, or
    /// to a device back from the hide to the end of the round shown.
    fn worn(name: &str, look: &str) -> blend_in::Tell {
        blend_in::Tell::Wear {
            name: name.into(),
            look: look.into(),
            now: true,
        }
    }

    #[test]
    fn a_device_opened_again_part_way_through_a_round_is_dressed_as_it_wears_whichever_word_comes_first() {
        // What it wears told before the round comes.
        let mut world = at_the_table();
        told(&mut world, worn("Bori", "girl"));
        sent(&mut world, BlendStage::Seek);
        world.run_system_once(follow).expect("followed");
        let blend = world.resource::<Blend>();
        assert_eq!(blend.worn, Some(("Bori".into(), "girl".into())));
        // And after it.
        let mut world = at_the_table();
        sent(&mut world, BlendStage::Shown);
        world.run_system_once(follow).expect("followed");
        assert_eq!(world.resource::<Blend>().worn, None);
        told(&mut world, worn("Bori", "girl"));
        assert_eq!(world.resource::<Blend>().worn, Some(("Bori".into(), "girl".into())));
        // Seen first while everyone chooses, nobody is dressed yet.
        let mut world = at_the_table();
        told(&mut world, wear("Bori", "girl"));
        sent(&mut world, BlendStage::Choose);
        world.run_system_once(follow).expect("followed");
        assert_eq!(world.resource::<Blend>().worn, None);
    }

    #[test]
    fn an_answer_to_a_choice_made_before_the_last_is_not_shown_over_the_last() {
        let mut world = at_the_table();
        sent(&mut world, BlendStage::Choose);
        world.run_system_once(follow).expect("followed");
        told(&mut world, wear("Dubu", "boy"));
        // Two quick taps on an arrow, each sent as made (`choose::act`).
        {
            let mut blend = world.resource_mut::<Blend>();
            blend.offered = Some(("Dubu".into(), "boy_blackbrown".into()));
            blend.asked += 1;
            blend.offered = Some(("Dubu".into(), "boy_whiteblue".into()));
            blend.asked += 1;
        }
        // The answer to the first: the second stays.
        told(&mut world, wear("Dubu", "boy_blackbrown"));
        assert_eq!(world.resource::<Blend>().offered, Some(("Dubu".into(), "boy_whiteblue".into())));
        // The answer to the second, whatever it says, is what is chosen.
        told(&mut world, wear("Dubu", "boy_whiteblue"));
        assert_eq!(world.resource::<Blend>().offered, Some(("Dubu".into(), "boy_whiteblue".into())));
        // A choice turned down, nothing sent since: what the server has.
        world.resource_mut::<Blend>().offered = Some(("Mochi".into(), "boy_whiteblue".into()));
        world.resource_mut::<Blend>().asked += 1;
        told(&mut world, wear("Dubu", "boy_whiteblue"));
        assert_eq!(world.resource::<Blend>().offered, Some(("Dubu".into(), "boy_whiteblue".into())));
        // Whatever is out as the hide begins, what it is told it wears.
        world.resource_mut::<Blend>().asked = 2;
        sent(&mut world, BlendStage::Hide);
        world.run_system_once(follow).expect("followed");
        told(&mut world, worn("Kong", "boy"));
        let blend = world.resource::<Blend>();
        assert_eq!((blend.offered.clone(), blend.worn.clone()), (Some(("Kong".into(), "boy".into())), Some(("Kong".into(), "boy".into()))));
    }

    #[test]
    fn what_the_next_round_offers_is_not_worn_until_its_hide() {
        let mut world = at_the_table();
        sent(&mut world, BlendStage::Seek);
        world.run_system_once(follow).expect("followed");
        told(&mut world, worn("Bori", "girl"));
        assert_eq!(world.resource::<Blend>().worn, Some(("Bori".into(), "girl".into())));
        // The round shown, and the next round's offer come before the room
        // says its choosing has begun: offered, and not worn.
        sent(&mut world, BlendStage::Shown);
        world.run_system_once(follow).expect("followed");
        told(&mut world, wear("Kong", "boy"));
        let blend = world.resource::<Blend>();
        assert_eq!(blend.worn, Some(("Bori".into(), "girl".into())), "still as everyone sees you");
        assert_eq!(blend.offered, Some(("Kong".into(), "boy".into())), "the window's");
        // A round given up in the middle of its seek: the next round's offer
        // comes while this device still follows the seek (as a play-tester's
        // run of three found it).
        let mut world = at_the_table();
        sent(&mut world, BlendStage::Seek);
        world.run_system_once(follow).expect("followed");
        told(&mut world, worn("Bori", "girl"));
        told(&mut world, wear("Kong", "boy"));
        let blend = world.resource::<Blend>();
        assert_eq!(blend.worn, Some(("Bori".into(), "girl".into())), "the offer is not worn");
        assert_eq!(blend.offered, Some(("Kong".into(), "boy".into())));
        // At the hide, what it is dressed in, whichever comes first, the
        // word or the round.
        told(&mut world, worn("Kong", "boy"));
        assert_eq!(world.resource::<Blend>().worn, Some(("Kong".into(), "boy".into())));
    }

    #[test]
    fn a_round_given_up_is_said_in_the_next_round_s_window_and_never_across_it() {
        let mut world = at_the_table();
        sent(&mut world, BlendStage::Seek);
        world.run_system_once(follow).expect("followed");
        // The hider (Mochi, seat 1) gone in the seek: the next round's
        // choosing comes at once.
        let shown = world.resource::<TableView>().shown.unwrap();
        let mut next = round(1, BlendStage::Choose);
        next.hider = 0;
        world.entity_mut(shown).insert(next);
        world.run_system_once(follow).expect("followed");
        assert_eq!(world.resource::<Blend>().given_up, Some("Mochi".to_owned()));
        let mut up = world.query_filtered::<(), With<Headline>>();
        assert_eq!(up.iter(&world).count(), 0, "nothing across the window");
        // Gone as the hide begins.
        let mut hide = round(1, BlendStage::Hide);
        hide.hider = 0;
        world.entity_mut(shown).insert(hide);
        world.run_system_once(follow).expect("followed");
        assert_eq!(world.resource::<Blend>().given_up, None);
    }

    #[test]
    fn the_end_of_a_round_says_who_the_hider_went_by() {
        let mut world = at_the_table();
        sent(&mut world, BlendStage::Seek);
        world.run_system_once(follow).expect("followed");
        // Mochi (seat 1) hid in the body that went by 777, as Latte.
        world.spawn((
            Remote,
            Member {
                id: 777,
                name: "Latte".into(),
                look: "girl".into(),
                ai: false,
            },
        ));
        let shown = world.resource::<TableView>().shown.unwrap();
        let mut over = round(0, BlendStage::Shown);
        over.hider = 1;
        over.unmasked = Some(777);
        world.entity_mut(shown).insert(over);
        world.run_system_once(follow).expect("followed");
        let file = ui();
        let said = fill(&file.words.blend_in.hid_as, &[("name", "Mochi"), ("guise", "Latte")]);
        let mut texts = world.query::<&Text>();
        assert!(texts.iter(&world).any(|text| text.0 == said), "{said:?} across the screen");
        assert!(texts.iter(&world).any(|text| text.0 == fill(&file.words.blend_in.stayed, &[("name", "Mochi")])));
    }

    #[test]
    fn what_a_wrong_swing_costs_is_said_as_the_rules_have_it() {
        let rule = stun_rule();
        for seconds in [4, 7, 10] {
            assert!(rule.contains(&seconds.to_string()), "{rule}");
        }
        assert!(!rule.contains('{'), "{rule}");
        assert!(rule.chars().all(roundtown_net::text::allowed));
    }

    #[test]
    fn a_window_put_away_by_its_cross_is_up_again_at_the_next_table() {
        let mut world = at_the_table();
        sent(&mut world, BlendStage::Choose);
        world.run_system_once(follow).expect("followed");
        assert_eq!(choose::choosing(&world.resource::<Blend>()), Some(0));
        // Its cross, in round 1 of this table: not up again this round.
        world.resource_mut::<Blend>().shut = Some(0);
        assert_eq!(choose::choosing(&world.resource::<Blend>()), None);
        // The same table begun afresh on this device: still put away.
        world.write_message(table::Entered {
            game: Minigame::BlendIn,
            seat: 0,
            online: true,
            same_table: true,
        });
        world.run_system_once(begin_afresh).expect("begun");
        assert_eq!(choose::choosing(&world.resource::<Blend>()), None);
        world.resource_mut::<Messages<table::Entered>>().clear();
        // Play again: a new table, its first round the window's.
        world.write_message(table::Entered {
            game: Minigame::BlendIn,
            seat: 1,
            online: true,
            same_table: false,
        });
        world.run_system_once(begin_afresh).expect("begun");
        assert_eq!(world.resource::<Blend>().shut, None);
        let entity = world.spawn((Remote, round(0, BlendStage::Choose))).id();
        world.resource_mut::<TableView>().shown = Some(entity);
        world.run_system_once(follow).expect("followed");
        assert_eq!(choose::choosing(&world.resource::<Blend>()), Some(0));
    }

    #[test]
    fn a_seeker_runs_only_through_the_seek() {
        let mut blend = Blend {
            seat: Some(1),
            round: Some(round(0, BlendStage::Hide)),
            ..default()
        };
        assert!(blend.armed() && !blend.hiding());
        blend.round = Some(round(0, BlendStage::Seek));
        assert_eq!(blend.playing().unwrap().pace(1), blend_in::SEEKER_PACE);
        assert!(blend.armed());
        blend.round = Some(round(0, BlendStage::Shown));
        assert!(!blend.armed());
        // The hider never.
        blend.seat = Some(0);
        blend.round = Some(round(0, BlendStage::Seek));
        assert!(blend.hiding() && !blend.armed());
        // Stunned only in the seek, and only until its countdown is up.
        blend.seat = Some(1);
        blend.stunned = Some(10.0);
        assert!(blend.stunned_at(9.0) && !blend.stunned_at(10.0));
        blend.round = Some(round(0, BlendStage::Shown));
        assert!(!blend.stunned_at(9.0));
    }

    #[test]
    fn the_sword_is_where_the_town_has_its_fist_clear_of_the_other_buttons() {
        let (right, bottom, size) = hud::CHAT_BUTTON;
        let (quick_right, quick_bottom, quick_size) = hud::QUICK_BUTTON;
        hud::clear_of_each_other(&[
            ((0.0, 0.0), crate::JUMP_SIZE_VMIN),
            ((right, bottom), size),
            ((quick_right, quick_bottom), quick_size),
            (FIST, hud::ACTION_BUTTON),
        ])
        .unwrap();
    }
}
