//! Talking to whoever is in the town: its AIs, and the House Builder at the
//! door of their house (Hajun, 2026-10-31).
//!
//! Walk up to any of them and a small round chat bubble, three dots in it,
//! comes up over their head: over one at a time, the nearest to you
//! whichever way you face ([`choose`]), so that there is never one to pick
//! from another and no aiming at it. The bubble is a thing in the world
//! since 2026-11-22 (`bubble`, [`show_bubble`]): as big there wherever the
//! camera is, where until then it was a button on the screen, as big on it
//! however far out the camera was. Tap it, or on desktop press E, and the
//! camera comes in to their face from your eyes (`CloseUp`) as they stop and
//! turn to you: what they say is along the bottom of the screen, and what you
//! can say back is over it, on the right. A tap anywhere, or on what you say
//! back, and you are done: the camera goes back out to where it was, and they
//! carry on.
//!
//! An AI says something at random (`small_talk` in `assets/ui.ron`, as the
//! colours and the rest of the words are, `ui`), and you answer in a word.
//! The House Builder asks whether you want to play their game
//! (`builder::pitch`): Play is House Builder's to deal with, and Later is
//! goodbye.

use bevy::prelude::*;

use roundtown_net::rules::ENTRY;

use crate::ai::Steering;
use crate::builder::{self, DIALOG_LAYER, Fill};
use crate::bubble::{self, Bubble, Looks, Says, Tapped};
use crate::hud::{self, DialogUp, TakesPress};
use crate::island::{Island, Islands, Venue};
use crate::lobby::Balance;
use crate::tags::{self, HidesNames, Tagged};
use crate::ui::ui;
use crate::{CloseUp, EYE_HEIGHT, Player, ThirdPersonCamera};

/// How near someone has to be for the bubble to come up over them, in
/// metres, whichever way you face (Hajun, 2026-11-02: "it should be
/// irrelevant where the player is facing, just their xy location"; until
/// then they had to be within 70° of straight ahead of you as well).
const REACH: f32 = 4.5;
/// How far off someone the bubble is over can go before it leaves them: a
/// little further than it takes to come up, so that it does not come and go
/// at the edge (and 80° round, while facing counted).
const KEEP_REACH: f32 = 5.0;
/// How much nearer someone else has to be to take the bubble from whoever
/// has it, in metres.
const NEARER_BY: f32 = 0.5;
/// How much higher or lower than you someone can stand and still be talked
/// to, in metres.
const HIGHER: f32 = 1.5;
/// How high over someone's feet their face is, in metres: the model's eyes
/// are 1.54 to 1.6 m up, the top of its head 1.71.
/// 1.38 since the evening of 2026-11-01, the middle of `playercharacter.glb`'s
/// round head, where its eyes are; 1.52, the old model's, until then.
const FACE: f32 = 1.38;
/// How high over someone's feet the bubble's point is, in metres: a little
/// over their head, where their name's foot is (`tags::OVER_HEAD`), so that
/// over a name it stands a fixed way over the name however near the camera
/// is. 1.95 until Hajun asked for it "slightly lower so its slightly more
/// closer to the nametag" (2026-11-01).
const OVER_HEAD: f32 = tags::OVER_HEAD;
/// How quickly whoever you talk to turns to you, and you to them: this share
/// of the turn left, a second.
const TURN_TO: f32 = 10.0;
/// What is said along the bottom of the screen: as wide as this at the most,
/// as shares of the short side of the screen, and the size of its words; and
/// the replies over it. A tenth bigger since the evening of 2026-10-31
/// (Hajun: "very slightly increase the size of both the text box and the
/// font size and the button"; 112, 4.2, 3.4, 4.0, 9.5, 30 and 4 until then).
const SAY_WIDTH: f32 = 122.0;
const SAY_TEXT: f32 = 4.6;
const NAME_TEXT: f32 = 3.7;
const REPLY_TEXT: f32 = 4.4;
const REPLY_HEIGHT: f32 = 10.5;
const REPLY_WIDTH: f32 = 33.0;
const COIN: f32 = 4.4;
/// How thick the outline of the bubble, what is said and the replies is, as
/// a share of the short side of the screen.
const OUTLINE: f32 = 0.45;
/// How far below the bottom of the screen what is said starts, to rise from,
/// as a share of its short side, and how long it takes, in seconds.
const RISE_FROM: f32 = 40.0;
const RISE_SECS: f32 = 0.3;

// Black outlines and words on white, the bubble's look, which what is said
// and the replies share, a reply lit up while pressed and one that cannot
// be chosen; what the House Builder is called as they talk to you; and what
// the town's AIs say, and what you say back, one of them at random: all
// `assets/ui.ron`'s (`ui`).

/// Something the bubble comes up over that is opened rather than talked to,
/// on any island (since 2026-11-18, Last Stand's treasure chests; Hajun:
/// "u need to go close to it and it will show the proximitychatbubble and
/// u press that and the chest pops/vanishes"): the bubble's tap, or E,
/// says so ([`Opened`]), for its game to open it, and nothing is talked.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Opens;

/// The bubble over something that [`Opens`] was tapped: this one.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Opened(pub(crate) Entity);

/// Someone you can talk to, and what about.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Speaker {
    /// One of the town's AIs, who says something at random.
    Townsfolk,
    /// The House Builder, who asks whether you want to play their game.
    HouseBuilder,
}

/// Someone you are talking to: stopped where they stand, turned to you, and
/// the way they faced before.
#[derive(Component)]
pub(crate) struct Listening {
    rest: Quat,
}

/// The House Builder after you have talked, turning back to face the way they
/// stood.
#[derive(Component)]
struct TurningBack(Quat);

/// Who the bubble is over, if anyone.
#[derive(Resource, Default)]
struct Prompt {
    over: Option<Entity>,
}

/// The bubble, a thing in the world (`bubble`): over this one.
#[derive(Component)]
struct PromptBubble(Entity);

/// Who you are talking to, if anyone; what a key says back to them; and the
/// last thing an AI said, so that the next is something else.
#[derive(Resource, Default)]
struct Conversation {
    with: Option<Entity>,
    first: Option<Reply>,
    said: Option<usize>,
}

/// Everything on the screen while you talk, to go when you are done.
#[derive(Component)]
struct TalkUi;

/// What is said and the replies, rising from the bottom of the screen, for
/// this long so far.
#[derive(Component)]
struct Rising(f32);

/// What a press on part of a conversation says: a reply, or, anywhere else,
/// goodbye.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Reply {
    /// That is all: back to what you were doing.
    Bye,
    /// Play the House Builder's game.
    Play,
}

/// Something asked of talking, by a tap, a click or a key.
#[derive(Message, Clone, Copy, PartialEq, Eq, Debug)]
enum Ask {
    /// Talk to whoever the bubble is over.
    Start,
    /// Say this back, and be done.
    Say(Reply),
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Ask>()
        .add_message::<Opened>()
        .init_resource::<Prompt>()
        .init_resource::<Conversation>()
        .add_observer(press)
        .add_systems(
            Update,
            (
                // Ahead of House Builder, which Play goes to, and of your own
                // walking, which stops while you talk.
                (keys, begin, answer, keep_up)
                    .chain()
                    .before(builder::GameSystems),
                (turn_to_each_other, turn_back, point_camera)
                    .after(crate::move_bodies)
                    .before(crate::follow_camera),
                // With everyone where they are this frame, and the camera;
                // the bubble where it is to be before it is turned to the
                // camera.
                (choose, show_bubble.before(bubble::BubbleSystems), rise)
                    .chain()
                    .after(crate::follow_camera),
                tapped.after(bubble::Taps),
            ),
        );
}

// ------------------------------------------------------------- the bubble

/// Who the bubble is over: the nearest of those you can talk to, whichever
/// way you face, in the town, while nothing else is up and the camera is
/// your own; or, on any island, the nearest thing there that [`Opens`].
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn choose(
    me: Query<(&Transform, &Venue, Has<crate::rest::Resting>), (With<Player>, Without<crate::Dead>)>,
    speakers: Query<(Entity, &GlobalTransform, &Venue, &InheritedVisibility, Has<Opens>), Or<(With<Speaker>, With<Opens>)>>,
    islands: Res<Islands>,
    up: Res<DialogUp>,
    close_up: Res<CloseUp>,
    conversation: Res<Conversation>,
    driving: Res<crate::drive::Driving>,
    mut prompt: ResMut<Prompt>,
) {
    // Nobody is spoken to from the car (`drive`), nor by the dead, nor
    // from a chair or a bed (`rest`), which talking would turn you on.
    let over = me
        .single()
        .ok()
        .filter(|&(_, _, resting)| !resting && !up.0 && !close_up.busy() && conversation.with.is_none() && !driving.on())
        .and_then(|(you, &here, _)| {
            let island = islands.get(here)?;
            let them = speakers
                .iter()
                .filter(|&(_, _, &on, shown, opens)| on == here && shown.get() && (opens || here == Venue::Town))
                .map(|(who, at, ..)| (who, at.translation()));
            nearest_to(you, them, prompt.over, |from, to| in_sight(island, from, to))
        });
    if prompt.over != over {
        prompt.over = over;
    }
}

/// Of `them`, by where their feet are, the one the bubble goes over for
/// someone standing at `you`: the nearest within [`REACH`] along the ground,
/// whichever way you face, and in sight (`clear`, from eyes to face); unless
/// whoever has it `now` still has it by the looser [`KEEP_REACH`], and nobody
/// is [`NEARER_BY`] nearer.
fn nearest_to(
    you: &Transform,
    them: impl Iterator<Item = (Entity, Vec3)>,
    now: Option<Entity>,
    clear: impl Fn(Vec3, Vec3) -> bool,
) -> Option<Entity> {
    let eyes = you.translation + Vec3::Y * EYE_HEIGHT;
    let mut nearest: Option<(f32, Entity)> = None;
    let mut kept = None;
    for (who, feet) in them {
        let holds = now == Some(who);
        let reach = if holds { KEEP_REACH } else { REACH };
        let far = (feet.xz() - you.translation.xz()).length();
        if far > reach || (feet.y - you.translation.y).abs() > HIGHER {
            continue;
        }
        if !clear(eyes, feet + Vec3::Y * FACE) {
            continue;
        }
        if holds {
            kept = Some(far);
        }
        if nearest.is_none_or(|(near, _)| far < near) {
            nearest = Some((far, who));
        }
    }
    match (kept, nearest) {
        (Some(kept), Some((far, _))) if far > kept - NEARER_BY => now,
        (_, nearest) => nearest.map(|(_, who)| who),
    }
}

/// Whether nothing of `island` is between `from` and `to`: no wall between
/// you and someone on the other side of it.
fn in_sight(island: &Island, from: Vec3, to: Vec3) -> bool {
    island.sweep(from, to, 0.05).is_none()
}

/// Keeps the bubble over the head of whoever it is over, its point on
/// them, and over their name if they have one: a thing in the world
/// (`bubble`), as big there wherever the camera is, put up as someone
/// comes within reach and taken away as they go, or another does.
///
/// A name is written on the screen, as big on it however far off its owner
/// is, so over one the bubble's point is a name and a half's height over
/// the name's foot as the screen has it ([`over_a_name`]): further over
/// their head, in the world, the further out the camera is.
fn show_bubble(
    prompt: Res<Prompt>,
    speakers: Query<(&GlobalTransform, &Venue, Has<Tagged>)>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    looks: Option<Res<Looks>>,
    mut bubbles: Query<(Entity, &PromptBubble, &mut Bubble)>,
    mut commands: Commands,
) {
    let over = prompt.over.and_then(|over| {
        let (at, &venue, tagged) = speakers.get(over).ok()?;
        let head = at.translation() + Vec3::Y * OVER_HEAD;
        let tip = match (tagged, cameras.single(), windows.single()) {
            // The camera has no parent, so its transform is its global one,
            // and `follow_camera` has just set it for this frame.
            (true, Ok((camera, eye)), Ok(window)) => over_a_name(camera, &GlobalTransform::from(*eye), window, head).unwrap_or(head),
            _ => head,
        };
        Some((over, tip, venue))
    });
    let mut up = false;
    for (entity, of, mut bubble) in &mut bubbles {
        match over {
            Some((over, tip, _)) if of.0 == over => {
                up = true;
                if bubble.tip != tip {
                    bubble.tip = tip;
                }
            }
            _ => commands.entity(entity).despawn(),
        }
    }
    if let (false, Some((over, tip, venue)), Some(looks)) = (up, over, looks) {
        commands.spawn((PromptBubble(over), looks.bubble(Says::Dots, tip, venue)));
    }
}

/// Where the point of a bubble is over someone whose name is written over
/// `head`: a name and a half's height (`tags::TAG_TEXT`) up the screen from
/// there, as far off as `head` is. Nowhere, with `head` out of the
/// camera's sight.
fn over_a_name(camera: &Camera, eye: &GlobalTransform, window: &Window, head: Vec3) -> Option<Vec3> {
    let vmin = window.width().min(window.height()) / 100.0;
    let foot = camera.world_to_viewport(eye, head).ok()?;
    let ray = camera.viewport_to_world(eye, foot - Vec2::Y * (tags::TAG_TEXT * 1.5 * vmin)).ok()?;
    let ahead = eye.forward();
    let (far, along) = ((head - ray.origin).dot(*ahead), ray.direction.dot(*ahead));
    (along > 1e-4 && far > 0.0).then(|| ray.origin + *ray.direction * (far / along))
}

/// A tap on the bubble, in the world (`bubble::tap`): as E is.
fn tapped(mut taps: MessageReader<Tapped>, bubbles: Query<(), With<PromptBubble>>, mut asks: MessageWriter<Ask>) {
    if taps.read().any(|&Tapped(bubble)| bubbles.contains(bubble)) {
        asks.write(Ask::Start);
    }
}

// ------------------------------------------------------------ asking

/// A tap or a click on any part of a conversation: what it says, or,
/// anywhere but a reply, goodbye. Only what the press landed on, not what
/// the click passed up through on its way out. (The bubble is in the
/// world: [`tapped`].)
fn press(click: On<Pointer<Click>>, replies: Query<&Reply>, mut asks: MessageWriter<Ask>) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if let Ok(&reply) = replies.get(click.entity) {
        asks.write(Ask::Say(reply));
    }
}

/// Keys, for desktop: E to talk to whoever the bubble is over; and talking,
/// Enter for the first reply, Escape or E to say goodbye.
fn keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    prompt: Res<Prompt>,
    conversation: Res<Conversation>,
    mut asks: MessageWriter<Ask>,
) {
    if conversation.with.is_none() {
        if prompt.over.is_some() && keyboard.just_pressed(KeyCode::KeyE) {
            asks.write(Ask::Start);
        }
        return;
    }
    if keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter])
        && let Some(reply) = conversation.first
    {
        asks.write(Ask::Say(reply));
    } else if keyboard.any_just_pressed([KeyCode::Escape, KeyCode::KeyE]) {
        asks.write(Ask::Say(Reply::Bye));
    }
}

// -------------------------------------------------------- talking

/// Starts talking to whoever the bubble is over: they stop where they stand,
/// and what they say comes up.
#[allow(clippy::too_many_arguments)]
fn begin(
    mut asks: MessageReader<Ask>,
    mut prompt: ResMut<Prompt>,
    mut conversation: ResMut<Conversation>,
    mut up: ResMut<DialogUp>,
    mut speakers: Query<(&Speaker, Option<&Name>, &Transform, Option<&TurningBack>, Option<&mut Steering>)>,
    opens: Query<(), With<Opens>>,
    mut opened: MessageWriter<Opened>,
    me: Query<&Balance, With<Player>>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    for &ask in asks.read() {
        if ask != Ask::Start || conversation.with.is_some() || up.0 {
            continue;
        }
        let Some(who) = prompt.over.take() else {
            continue;
        };
        // A thing that opens is opened, by its game, and nothing is said.
        if opens.contains(who) {
            opened.write(Opened(who));
            continue;
        }
        let Ok((&speaker, name, place, turning, steering)) = speakers.get_mut(who) else {
            continue;
        };
        if let Some(mut steering) = steering {
            steering.stop();
        }
        // Turning back from the last time, they face the way they stood
        // before that.
        let rest = turning.map_or(place.rotation, |turning| turning.0);
        commands
            .entity(who)
            .try_remove::<TurningBack>()
            .try_insert(Listening { rest });
        let ui = ui();
        let words = &ui.words;
        let (name, line, replies) = match speaker {
            Speaker::Townsfolk => {
                let line = small_talk(conversation.said, words.talk.small_talk.len());
                conversation.said = Some(line);
                let (says, back) = &words.talk.small_talk[line];
                let name = name.map_or(words.talk.someone.as_str(), Name::as_str);
                (name.to_owned(), says.clone(), vec![(Reply::Bye, back.as_str(), true)])
            }
            Speaker::HouseBuilder => {
                let affordable = me.single().is_ok_and(|balance| balance.0 >= ENTRY);
                (
                    words.talk.house_builder.clone(),
                    builder::pitch(affordable),
                    vec![
                        (Reply::Play, words.house_builder.play.as_str(), affordable),
                        (Reply::Bye, words.house_builder.later.as_str(), true),
                    ],
                )
            }
        };
        conversation.with = Some(who);
        conversation.first = replies.first().map(|&(reply, ..)| reply);
        up.set_if_neq(DialogUp(true));
        spawn_talk(&mut commands, &assets, &name, &line, &replies);
    }
}

/// Which of the `count` lines of small talk to say: any but `last`.
fn small_talk(last: Option<usize>, count: usize) -> usize {
    let Some(last) = last.filter(|&last| last < count && count > 1) else {
        return fastrand::usize(..count);
    };
    let line = fastrand::usize(..count - 1);
    if line >= last { line + 1 } else { line }
}

/// What you say back, which is the end of it: Play goes to House Builder, and
/// only once you can pay for it. Anything else is goodbye.
#[allow(clippy::too_many_arguments)]
fn answer(
    mut asks: MessageReader<Ask>,
    mut conversation: ResMut<Conversation>,
    mut up: ResMut<DialogUp>,
    mut close_up: ResMut<CloseUp>,
    talks: Query<Entity, With<TalkUi>>,
    speakers: Query<(&Speaker, &Listening)>,
    me: Query<&Balance, With<Player>>,
    mut plays: MessageWriter<builder::Ask>,
    mut commands: Commands,
) {
    for &ask in asks.read() {
        let Ask::Say(reply) = ask else {
            continue;
        };
        let Some(who) = conversation.with else {
            continue;
        };
        if reply == Reply::Play && !me.single().is_ok_and(|balance| balance.0 >= ENTRY) {
            continue;
        }
        stop_talking(who, &speakers, &talks, &mut conversation, &mut up, &mut commands);
        if reply == Reply::Play {
            // Off to the lobby: the camera goes with you rather than back out
            // from a face in the town.
            close_up.cut();
            plays.write(builder::Ask::Join);
        }
    }
}

/// Stops talking to whoever you are talking to if they have gone, or either of
/// you is somewhere else now: sent there by the server, say.
fn keep_up(
    mut conversation: ResMut<Conversation>,
    mut up: ResMut<DialogUp>,
    mut close_up: ResMut<CloseUp>,
    me: Query<&Venue, With<Player>>,
    venues: Query<&Venue>,
    talks: Query<Entity, With<TalkUi>>,
    speakers: Query<(&Speaker, &Listening)>,
    mut commands: Commands,
) {
    let Some(who) = conversation.with else {
        return;
    };
    let here = me.single().ok();
    if here.is_some() && here == venues.get(who).ok() && here == Some(&Venue::Town) {
        return;
    }
    stop_talking(who, &speakers, &talks, &mut conversation, &mut up, &mut commands);
    close_up.cut();
}

/// The end of talking to `who`: what was said goes, they carry on, and the
/// House Builder turns back to the way they stood.
fn stop_talking(
    who: Entity,
    speakers: &Query<(&Speaker, &Listening)>,
    talks: &Query<Entity, With<TalkUi>>,
    conversation: &mut Conversation,
    up: &mut DialogUp,
    commands: &mut Commands,
) {
    for talk in talks {
        commands.entity(talk).despawn();
    }
    // Quietly: they may be going this same frame, an AI making way for
    // someone online.
    if let Ok((&speaker, listening)) = speakers.get(who) {
        let mut them = commands.entity(who);
        them.try_remove::<Listening>();
        if speaker == Speaker::HouseBuilder {
            them.try_insert(TurningBack(listening.rest));
        }
    }
    conversation.with = None;
    conversation.first = None;
    if up.0 {
        up.0 = false;
    }
}

/// Turns whoever you are talking to round to you, and you to them, their
/// feet where they stand.
fn turn_to_each_other(
    time: Res<Time>,
    conversation: Res<Conversation>,
    me: Query<Entity, With<Player>>,
    mut bodies: Query<&mut Transform>,
) {
    let (Some(who), Ok(you)) = (conversation.with, me.single()) else {
        return;
    };
    let Ok([mut them, mut yours]) = bodies.get_many_mut([who, you]) else {
        return;
    };
    let share = 1.0 - (-TURN_TO * time.delta_secs()).exp();
    let (there, here) = (them.translation, yours.translation);
    turn_toward(&mut them, here, share);
    turn_toward(&mut yours, there, share);
}

/// Turns `body` `share` of the way round to face `at`, about the upright.
fn turn_toward(body: &mut Mut<Transform>, at: Vec3, share: f32) {
    let toward = (at - body.translation).with_y(0.0);
    if toward.length_squared() < 1e-6 {
        return;
    }
    let facing = Transform::default().looking_to(toward, Vec3::Y).rotation;
    let turned = body.rotation.slerp(facing, share);
    if body.rotation.angle_between(facing) > 1e-4 {
        body.rotation = turned;
    }
}

/// Turns the House Builder back to the way they stood, after you have
/// talked.
fn turn_back(
    time: Res<Time>,
    mut turning: Query<(Entity, &TurningBack, &mut Transform)>,
    mut commands: Commands,
) {
    let share = 1.0 - (-TURN_TO * time.delta_secs()).exp();
    for (who, back, mut place) in &mut turning {
        if place.rotation.angle_between(back.0) < 1e-3 {
            place.rotation = back.0;
            commands.entity(who).remove::<TurningBack>();
        } else {
            place.rotation = place.rotation.slerp(back.0, share);
        }
    }
}

/// Puts the camera on the face of whoever you are talking to, or takes it off
/// once you are done.
fn point_camera(
    conversation: Res<Conversation>,
    speakers: Query<&GlobalTransform>,
    mut close_up: ResMut<CloseUp>,
) {
    let on = conversation
        .with
        .and_then(|who| speakers.get(who).ok())
        .map(|at| at.translation() + Vec3::Y * FACE);
    if close_up.on != on {
        close_up.on = on;
    }
}

// ---------------------------------------------------- what is said

/// What `name` says, `line`, along the bottom of the screen, and what you can
/// say back over it on the right, `replies`: each with whether it can be
/// chosen. Over everything else, and taking every press there is: a tap on
/// anything but a reply is goodbye.
fn spawn_talk(
    commands: &mut Commands,
    assets: &AssetServer,
    name: &str,
    line: &str,
    replies: &[(Reply, &str, bool)],
) {
    let colours = ui().colours;
    let screen = commands
        .spawn((
            TalkUi,
            Reply::Bye,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let column = commands
        .spawn((
            ChildOf(screen),
            Rising(0.0),
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                right: Val::ZERO,
                bottom: Val::VMin(hud::EDGE),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::VMin(2.6),
                padding: UiRect::horizontal(Val::VMin(hud::EDGE)),
                ..default()
            },
            UiTransform::from_translation(Val2::new(Val::ZERO, Val::VMin(RISE_FROM))),
            Pickable::IGNORE,
        ))
        .id();
    commands
        .spawn((
            ChildOf(column),
            Node {
                align_self: AlignSelf::Stretch,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::VMin(1.8),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|column| {
            for &(reply, words, able) in replies {
                column.spawn(reply_button(assets, reply, words, able));
            }
        });
    commands.spawn((
        ChildOf(column),
        Reply::Bye,
        HidesNames,
        Node {
            width: Val::VMin(SAY_WIDTH),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::new(Val::VMin(4.5), Val::VMin(4.5), Val::VMin(4.2), Val::VMin(3.4)),
            border: UiRect::all(Val::VMin(OUTLINE)),
            border_radius: BorderRadius::all(Val::VMin(4.0)),
            ..default()
        },
        BackgroundColor(colours.talk.paper),
        BorderColor::all(colours.talk.ink),
        children![
            // Who is talking, on a tab over the top edge.
            (
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::VMin(-NAME_TEXT * 0.9),
                    left: Val::VMin(3.6),
                    padding: UiRect::axes(Val::VMin(2.6), Val::VMin(0.7)),
                    border_radius: BorderRadius::all(Val::VMin(NAME_TEXT)),
                    ..default()
                },
                BackgroundColor(colours.talk.ink),
                Pickable::IGNORE,
                children![builder::label(name, NAME_TEXT, colours.text)],
            ),
            (
                Text::new(line),
                crate::font::sized(SAY_TEXT),
                TextColor(colours.talk.ink),
                TextLayout::justify(Justify::Left),
                Pickable::IGNORE,
            ),
        ],
    ));
}

/// One of the replies: white with a black outline, as the bubble is, and
/// lit up while pressed. Play has its price beside it, and is greyed out
/// while you cannot pay it.
fn reply_button(assets: &AssetServer, reply: Reply, words: &str, able: bool) -> impl Bundle {
    let colours = ui().colours;
    let (ink, lit) = if able {
        (colours.talk.ink, colours.talk.paper_lit)
    } else {
        (colours.talk.greyed, colours.talk.paper)
    };
    let price = (reply == Reply::Play).then(|| {
        (
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::VMin(0.8),
                margin: UiRect::left(Val::VMin(2.0)),
                ..default()
            },
            Pickable::IGNORE,
            children![
                (
                    ImageNode::new(assets.load(hud::COIN_ICON)).with_color(if able {
                        colours.text
                    } else {
                        colours.text.with_alpha(colours.text.alpha() * 0.5)
                    }),
                    Node {
                        width: Val::VMin(COIN),
                        height: Val::VMin(COIN),
                        ..default()
                    },
                    Pickable::IGNORE,
                ),
                builder::label(hud::money(ENTRY), REPLY_TEXT, ink),
            ],
        )
    });
    (
        reply,
        TakesPress,
        HidesNames,
        Button,
        Node {
            min_width: Val::VMin(REPLY_WIDTH),
            height: Val::VMin(REPLY_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::VMin(4.0)),
            border: UiRect::all(Val::VMin(OUTLINE)),
            border_radius: BorderRadius::all(Val::VMin(REPLY_HEIGHT * 0.5)),
            ..default()
        },
        BackgroundColor(colours.talk.paper),
        BorderColor::all(ink),
        Fill {
            idle: colours.talk.paper,
            lit,
        },
        Children::spawn((
            Spawn(builder::label(words, REPLY_TEXT, ink)),
            SpawnIter(price.into_iter()),
        )),
    )
}

/// Slides what is said and the replies up from below the screen as they come
/// up, slowing as they arrive. Once up, they are left alone.
fn rise(
    time: Res<Time>,
    mut rising: Query<(Entity, &mut Rising, &mut UiTransform)>,
    mut commands: Commands,
) {
    for (column, mut risen, mut place) in &mut rising {
        risen.0 += time.delta_secs();
        let done = (risen.0 / RISE_SECS).min(1.0);
        let below = RISE_FROM * (1.0 - done).powi(3);
        place.set_if_neq(UiTransform::from_translation(Val2::new(Val::ZERO, Val::VMin(below))));
        if done >= 1.0 {
            commands.entity(column).remove::<Rising>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3::new(x, 0.0, z)
    }

    /// Someone standing at the origin, facing -z, as `Transform` faces.
    fn you() -> Transform {
        Transform::default()
    }

    #[test]
    fn the_bubble_goes_over_the_nearest_to_you_whichever_way_you_face() {
        let mut world = World::new();
        let [near, far, behind, aside, off] = [(); 5].map(|_| world.spawn_empty().id());
        let them = [
            (far, at(0.5, -4.0)),
            (near, at(-0.8, -2.5)),
            // Behind you and to your side, nearer than either; and out of reach.
            (behind, at(0.0, 1.0)),
            (aside, at(2.0, -0.2)),
            (off, at(0.0, -6.0)),
        ];
        let open = |_: Vec3, _: Vec3| true;
        // The one behind you is the nearest, and has it whichever way you face.
        for face in [Vec3::NEG_Z, Vec3::Z, Vec3::NEG_X] {
            let you = you().looking_to(face, Vec3::Y);
            assert_eq!(nearest_to(&you, them.into_iter(), None, open), Some(behind));
        }
        // Without them, the one to your side; and nobody within reach, no bubble.
        let rest = [them[0], them[1], them[3], them[4]];
        assert_eq!(nearest_to(&you(), rest.into_iter(), None, open), Some(aside));
        assert_eq!(nearest_to(&you(), [them[4]].into_iter(), None, open), None);
    }

    #[test]
    fn a_wall_between_you_keeps_the_bubble_off_them() {
        let mut world = World::new();
        let [near, far] = [(); 2].map(|_| world.spawn_empty().id());
        let them = [(near, at(0.0, -2.0)), (far, at(0.0, -4.0))];
        // Only the further one is in sight.
        let sight = |_: Vec3, to: Vec3| to.z < -3.0;
        assert_eq!(nearest_to(&you(), them.into_iter(), None, sight), Some(far));
    }

    #[test]
    fn the_bubble_stays_put_until_someone_is_clearly_nearer() {
        let mut world = World::new();
        let [held, other] = [(); 2].map(|_| world.spawn_empty().id());
        let open = |_: Vec3, _: Vec3| true;
        // A little nearer is not enough to take it.
        let them = [(held, at(0.0, -3.0)), (other, at(0.3, -2.7))];
        assert_eq!(nearest_to(&you(), them.into_iter(), Some(held), open), Some(held));
        // Well nearer is.
        let them = [(held, at(0.0, -3.0)), (other, at(0.3, -2.0))];
        assert_eq!(nearest_to(&you(), them.into_iter(), Some(held), open), Some(other));
        // Once it has them, they keep it a little past where it came up.
        let them = [(held, at(0.0, -4.8))];
        assert_eq!(nearest_to(&you(), them.into_iter(), Some(held), open), Some(held));
        assert_eq!(nearest_to(&you(), them.into_iter(), None, open), None);
    }

    #[test]
    fn nobody_says_the_same_thing_twice_running() {
        let count = ui().words.talk.small_talk.len();
        for _ in 0..500 {
            let last = fastrand::usize(..count);
            let next = small_talk(Some(last), count);
            assert_ne!(next, last);
            assert!(next < count);
        }
        // Everything is said, sooner or later.
        let mut said = vec![false; count];
        for _ in 0..2000 {
            said[small_talk(None, count)] = true;
        }
        assert!(said.iter().all(|&said| said));
        // A file with one line, or fewer lines than before, does not break.
        assert_eq!(small_talk(Some(0), 1), 0);
        assert!(small_talk(Some(40), 3) < 3);
    }

    #[test]
    fn everything_said_is_of_what_the_fonts_draw() {
        // Plain ASCII until 2026-11-14, the font having nothing else;
        // Korean and Vietnamese besides since.
        let drawn = |words: &str| words.chars().all(roundtown_net::text::allowed);
        for (says, back) in &ui().words.talk.small_talk {
            assert!(drawn(says) && drawn(back), "{says} / {back}");
        }
        assert!(drawn(&builder::pitch(false)));
        assert!(builder::pitch(false).contains("100 coins"));
    }
}
