//! Talking to whoever is in the town: its AIs, and the House Builder at the
//! door of their house (Hajun, 2026-10-31).
//!
//! Walk up to any of them and a small round chat bubble, three dots in it,
//! comes up over their head: over one at a time, the nearest in front of you
//! ([`choose`]), so that there is never one to pick from another and no aiming
//! at it. Tap it, or on desktop press E, and the camera comes in to their face
//! from your eyes (`CloseUp`) as they stop and turn to you: what they say is
//! along the bottom of the screen, and what you can say back is over it, on
//! the right. A tap anywhere, or on what you say back, and you are done: the
//! camera goes back out to where it was, and they carry on.
//!
//! An AI says something at random ([`SMALL_TALK`]), and you answer in a word.
//! The House Builder asks whether you want to play their game
//! (`builder::pitch`): Play is House Builder's to deal with, and Later is
//! goodbye.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use roundtown_net::rules::ENTRY;

use crate::ai::Steering;
use crate::builder::{self, DIALOG_LAYER, Fill};
use crate::hud::{self, DialogUp, TakesPress};
use crate::island::{Island, Islands, Venue};
use crate::lobby::Balance;
use crate::tags::{self, HidesNames, Tagged};
use crate::{CloseUp, EYE_HEIGHT, Player, ThirdPersonCamera};

/// How near someone has to be for the bubble to come up over them, in
/// metres, and how far round from straight ahead of you, as the cosine of
/// the angle: 70°.
const REACH: f32 = 4.5;
const IN_FRONT: f32 = 0.34;
/// How far off, and how far round, someone the bubble is over can go before
/// it leaves them: a little further than it takes to come up, so that it does
/// not come and go at the edge. 80° round.
const KEEP_REACH: f32 = 5.0;
const KEEP_IN_FRONT: f32 = 0.17;
/// How much nearer someone else has to be to take the bubble from whoever
/// has it, in metres.
const NEARER_BY: f32 = 0.5;
/// How much higher or lower than you someone can stand and still be talked
/// to, in metres.
const HIGHER: f32 = 1.5;
/// How high over someone's feet their face is, in metres: the model's eyes
/// are 1.54 to 1.6 m up, the top of its head 1.71.
const FACE: f32 = 1.52;
/// How high over someone's feet the bubble's point is, in metres: a little
/// over their head.
const OVER_HEAD: f32 = 1.95;
/// How wide the bubble is drawn, and how far round it a tap still lands on it,
/// as shares of the short side of the screen: being the only one, it can
/// take a tap well wide of it.
const BUBBLE: f32 = 8.0;
const BUBBLE_MARGIN: f32 = 3.0;
/// How long the bubble takes to pop up over someone, in seconds.
const POP_SECS: f32 = 0.18;
/// How quickly whoever you talk to turns to you, and you to them: this share
/// of the turn left, a second.
const TURN_TO: f32 = 10.0;
/// What is said along the bottom of the screen: as wide as this at the most,
/// as shares of the short side of the screen, and the size of its words.
const SAY_WIDTH: f32 = 112.0;
const SAY_TEXT: f32 = 4.2;
const NAME_TEXT: f32 = 3.4;
const REPLY_TEXT: f32 = 4.0;
const REPLY_HEIGHT: f32 = 9.5;
const REPLY_WIDTH: f32 = 30.0;
const COIN: f32 = 4.0;
/// How thick the outline of the bubble, what is said and the replies is, as
/// a share of the short side of the screen.
const OUTLINE: f32 = 0.45;
/// How far below the bottom of the screen what is said starts, to rise from,
/// as a share of its short side, and how long it takes, in seconds.
const RISE_FROM: f32 = 40.0;
const RISE_SECS: f32 = 0.3;

/// Black outlines and words on white: the bubble's look, which what is said
/// and the replies share. A reply lit up while pressed, and one that cannot
/// be chosen.
const INK: Color = Color::srgb(0.05, 0.05, 0.06);
const PAPER: Color = Color::srgba(1.0, 1.0, 1.0, 0.97);
const PAPER_LIT: Color = Color::srgba(0.86, 0.86, 0.87, 0.97);
const GREYED: Color = Color::srgb(0.62, 0.62, 0.64);

/// What the House Builder is called, as they talk to you.
const HOUSE_BUILDER: &str = "House Builder";

/// What the town's AIs say, and what you say back: one of these, at random.
/// The built-in font is plain ASCII, so these are too.
const SMALL_TALK: &[(&str, &str)] = &[
    ("Hi there! Isn't Donggeurami Town lovely today?", "It really is!"),
    (
        "Did you know the fountain throws you up in the air if you stand in its top bowl?",
        "I'll try it!",
    ),
    ("The buses go all the way round town. Hop on one sometime!", "Sounds fun!"),
    (
        "I heard the House Builder is looking for builders. Have you played?",
        "Maybe later!",
    ),
    ("I love walking round the pond. It's so peaceful here.", "Me too!"),
    ("When the street lamps come on at night, the whole town glows.", "I'll stay up for it!"),
    ("Have you seen the sunset? The sky goes all pink and orange.", "Not yet!"),
    ("I'm just out for a stroll. Lovely weather for it!", "Enjoy your walk!"),
    ("Someday I want a house with a roof terrace.", "Good luck!"),
    ("Can you hear the music? I could listen to it all day.", "It's catchy!"),
    ("I tried to jump onto a bus roof once. Way too high!", "Nice try!"),
    ("Every house in town has a door that opens. I checked them all!", "That's dedication!"),
    ("Careful by the pond. It's shallow, but my shoes still got wet!", "I'll be careful!"),
    ("Hello! Have we met before? You look familiar.", "Nice to meet you!"),
    ("I'm saving up my coins for something special.", "Good idea!"),
    ("The stars come out at night. Have you ever looked up?", "I'll look tonight!"),
    ("Sometimes I just stand here and watch the buses go by.", "Sounds relaxing!"),
];

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

/// Who the bubble is over, if anyone, and how long it has been over them.
#[derive(Resource, Default)]
struct Prompt {
    over: Option<Entity>,
    shown: f32,
}

/// The bubble.
#[derive(Component)]
struct PromptBubble;

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
        .init_resource::<Prompt>()
        .init_resource::<Conversation>()
        .add_observer(press)
        .add_systems(Startup, spawn_bubble)
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
                // With everyone where they are this frame, and the camera.
                (choose, place_bubble, rise)
                    .chain()
                    .after(crate::follow_camera),
            ),
        );
}

// ------------------------------------------------------------- the bubble

/// The bubble, put away until there is someone to talk to. It is laid out top
/// left, and moved over their head without being laid out again; a little
/// over everything else that is not a dialog, so that the stick and the jump
/// button are never over it.
fn spawn_bubble(mut images: ResMut<Assets<Image>>, mut commands: Commands) {
    let image = images.add(bubble_image());
    commands.spawn((
        PromptBubble,
        TakesPress,
        Button,
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            padding: UiRect::all(Val::VMin(BUBBLE_MARGIN)),
            ..default()
        },
        UiTransform::default(),
        GlobalZIndex(1),
        Visibility::Hidden,
        children![(
            ImageNode::new(image),
            Node {
                width: Val::VMin(BUBBLE),
                height: Val::VMin(BUBBLE * BUBBLE_TALL),
                ..default()
            },
            Pickable::IGNORE,
        )],
    ));
}

/// Who the bubble is over: the nearest of those you can talk to in front of
/// you, in the town, while nothing else is up and the camera is your own.
#[allow(clippy::too_many_arguments)]
fn choose(
    time: Res<Time>,
    me: Query<(&Transform, &Venue), With<Player>>,
    speakers: Query<(Entity, &GlobalTransform, &Venue, &InheritedVisibility), With<Speaker>>,
    islands: Res<Islands>,
    up: Res<DialogUp>,
    close_up: Res<CloseUp>,
    conversation: Res<Conversation>,
    mut prompt: ResMut<Prompt>,
) {
    let over = me
        .single()
        .ok()
        .filter(|&(_, &here)| here == Venue::Town && !up.0 && !close_up.busy() && conversation.with.is_none())
        .and_then(|(you, &here)| {
            let island = islands.get(here)?;
            let them = speakers
                .iter()
                .filter(|&(_, _, &on, shown)| on == here && shown.get())
                .map(|(who, at, ..)| (who, at.translation()));
            nearest_in_front(you, them, prompt.over, |from, to| in_sight(island, from, to))
        });
    if prompt.over != over {
        prompt.over = over;
        prompt.shown = 0.0;
    } else if over.is_some() {
        prompt.shown += time.delta_secs();
    }
}

/// Of `them`, by where their feet are, the one the bubble goes over for
/// someone standing at `you`: the nearest within [`REACH`], in front of them,
/// and in sight (`clear`, from eyes to face); unless whoever has it `now` still
/// has it by the looser [`KEEP_REACH`], and nobody is [`NEARER_BY`] nearer.
fn nearest_in_front(
    you: &Transform,
    them: impl Iterator<Item = (Entity, Vec3)>,
    now: Option<Entity>,
    clear: impl Fn(Vec3, Vec3) -> bool,
) -> Option<Entity> {
    let ahead = you.forward().xz().normalize_or_zero();
    let eyes = you.translation + Vec3::Y * EYE_HEIGHT;
    let mut nearest: Option<(f32, Entity)> = None;
    let mut kept = None;
    for (who, feet) in them {
        let holds = now == Some(who);
        let (reach, wide) = if holds {
            (KEEP_REACH, KEEP_IN_FRONT)
        } else {
            (REACH, IN_FRONT)
        };
        let across = feet.xz() - you.translation.xz();
        let far = across.length();
        if far > reach || (feet.y - you.translation.y).abs() > HIGHER {
            continue;
        }
        if far > 1e-3 && across.dot(ahead) / far < wide {
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

/// Keeps the bubble over the head of whoever it is over, wherever the camera
/// goes, its point on them, and over their name if they have one. Never off
/// the screen, so that it can always be tapped: with the camera pulled in
/// close, their head can be over the top of it. It pops up as it comes.
fn place_bubble(
    prompt: Res<Prompt>,
    speakers: Query<(&GlobalTransform, Has<Tagged>)>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    mut bubbles: Query<(&mut Visibility, &mut UiTransform, &ComputedNode), With<PromptBubble>>,
) {
    let Ok((mut shown, mut place, node)) = bubbles.single_mut() else {
        return;
    };
    let on_screen = prompt.over.and_then(|over| {
        let (at, tagged) = speakers.get(over).ok()?;
        let (camera, eye) = cameras.single().ok()?;
        // The camera has no parent, so its transform is its global one, and
        // `follow_camera` has just set it for this frame.
        let point = camera
            .world_to_viewport(&GlobalTransform::from(*eye), at.translation() + Vec3::Y * OVER_HEAD)
            .ok()?;
        Some((point, tagged))
    });
    let (Some((point, tagged)), Ok(window)) = (on_screen, windows.single()) else {
        shown.set_if_neq(Visibility::Hidden);
        return;
    };
    let vmin = window.width().min(window.height()) / 100.0;
    let size = node.size() * node.inverse_scale_factor();
    let lift = if tagged { tags::TAG_TEXT * 1.5 * vmin } else { 0.0 };
    let tip = Vec2::splat(BUBBLE_MARGIN * vmin) + BUBBLE_TIP * Vec2::new(BUBBLE, BUBBLE * BUBBLE_TALL) * vmin;
    let lowest = Vec2::splat(hud::EDGE * vmin);
    let highest = (window.size() - size - lowest).max(lowest);
    let corner = (point - Vec2::Y * lift - tip).clamp(lowest, highest);
    let popped = (prompt.shown / POP_SECS).min(1.0);
    // Overshooting a little before it settles.
    let scale = 0.6 + 0.4 * (1.0 + 2.7 * (popped - 1.0).powi(3) + 1.7 * (popped - 1.0).powi(2));
    place.set_if_neq(UiTransform {
        translation: Val2::px(corner.x, corner.y),
        scale: Vec2::splat(scale),
        ..default()
    });
    shown.set_if_neq(Visibility::Inherited);
}

// ------------------------------------------------------------ asking

/// A tap or a click on the bubble, or on any part of a conversation: what it
/// says, or, anywhere but a reply, goodbye. Only what the press landed on,
/// not what the click passed up through on its way out.
fn press(
    click: On<Pointer<Click>>,
    bubbles: Query<(), With<PromptBubble>>,
    replies: Query<&Reply>,
    mut asks: MessageWriter<Ask>,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if bubbles.contains(click.entity) {
        asks.write(Ask::Start);
    } else if let Ok(&reply) = replies.get(click.entity) {
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
        let (name, line, replies) = match speaker {
            Speaker::Townsfolk => {
                let line = small_talk(conversation.said);
                conversation.said = Some(line);
                let (says, back) = SMALL_TALK[line];
                let name = name.map_or("Someone", Name::as_str);
                (name.to_owned(), says.to_owned(), vec![(Reply::Bye, back, true)])
            }
            Speaker::HouseBuilder => {
                let affordable = me.single().is_ok_and(|balance| balance.0 >= ENTRY);
                (
                    HOUSE_BUILDER.to_owned(),
                    builder::pitch(affordable),
                    vec![(Reply::Play, "Play", affordable), (Reply::Bye, "Later", true)],
                )
            }
        };
        conversation.with = Some(who);
        conversation.first = replies.first().map(|&(reply, ..)| reply);
        up.set_if_neq(DialogUp(true));
        spawn_talk(&mut commands, &assets, &name, &line, &replies);
    }
}

/// Which of [`SMALL_TALK`] to say: any but `last`.
fn small_talk(last: Option<usize>) -> usize {
    let Some(last) = last else {
        return fastrand::usize(..SMALL_TALK.len());
    };
    let line = fastrand::usize(..SMALL_TALK.len() - 1);
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
        BackgroundColor(PAPER),
        BorderColor::all(INK),
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
                BackgroundColor(INK),
                Pickable::IGNORE,
                children![builder::label(name, NAME_TEXT, Color::WHITE)],
            ),
            (
                Text::new(line),
                TextFont {
                    font_size: FontSize::VMin(SAY_TEXT),
                    ..default()
                },
                TextColor(INK),
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
    let (ink, lit) = if able { (INK, PAPER_LIT) } else { (GREYED, PAPER) };
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
                        Color::WHITE
                    } else {
                        Color::WHITE.with_alpha(0.5)
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
        BackgroundColor(PAPER),
        BorderColor::all(ink),
        Fill { idle: PAPER, lit },
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

// ------------------------------------------------------------- the picture

/// How tall the bubble's picture is, as a share of how wide.
const BUBBLE_TALL: f32 = (BUBBLE_TOP - BUBBLE_BOTTOM) / (BUBBLE_RIGHT - BUBBLE_LEFT);
/// Where the bubble's point is in its picture, as shares of its width from
/// the left and of its height from the top.
const BUBBLE_TIP: Vec2 = Vec2::new(
    (TAIL_TIP.x - BUBBLE_LEFT) / (BUBBLE_RIGHT - BUBBLE_LEFT),
    (BUBBLE_TOP - TAIL_TIP.y) / (BUBBLE_TOP - BUBBLE_BOTTOM),
);
/// The bubble, in shares of the radius of its circle, from the middle of it,
/// y up: the edges of its picture, and its tail's point, down and to the left,
/// and the two corners where it meets the circle, inside it.
const BUBBLE_LEFT: f32 = -1.06;
const BUBBLE_RIGHT: f32 = 1.06;
const BUBBLE_TOP: f32 = 1.06;
const BUBBLE_BOTTOM: f32 = -1.4;
const TAIL_TIP: Vec2 = Vec2::new(-0.74, -1.3);
const TAIL_FROM: Vec2 = Vec2::new(-0.75, -0.35);
const TAIL_TO: Vec2 = Vec2::new(-0.12, -0.8);
/// How much the tail's point is rounded off, how smoothly it runs into the
/// circle, how thick the outline is, and the three dots: how far apart and
/// how big.
const TAIL_ROUND: f32 = 0.06;
const TAIL_JOIN: f32 = 0.22;
const BUBBLE_OUTLINE: f32 = 0.1;
const DOT_APART: f32 = 0.42;
const DOT: f32 = 0.13;

/// The bubble: a white circle with a black outline, a short tail down and to
/// the left, and three black dots across the middle (Hajun, 2026-10-31: "small
/// minimalistic black outline, white inner, circular chatbubble with 3 black
/// dots"). Drawn, as the rating star is, and edged by how far each pixel is
/// from each edge, so that it is smooth at whatever size a screen shows it.
fn bubble_image() -> Image {
    /// How many pixels across the circle's radius is.
    const RADIUS: f32 = 64.0;
    let width = ((BUBBLE_RIGHT - BUBBLE_LEFT) * RADIUS).round() as u32;
    let height = ((BUBBLE_TOP - BUBBLE_BOTTOM) * RADIUS).round() as u32;
    let pixel = 1.0 / RADIUS;
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        for column in 0..width {
            let at = Vec2::new(
                BUBBLE_LEFT + (column as f32 + 0.5) * pixel,
                BUBBLE_TOP - (row as f32 + 0.5) * pixel,
            );
            let edge = bubble_distance(at);
            let dot = [-DOT_APART, 0.0, DOT_APART]
                .into_iter()
                .map(|x| at.distance(Vec2::new(x, 0.0)) - DOT)
                .fold(f32::INFINITY, f32::min);
            let cover = |distance: f32| (0.5 - distance / pixel).clamp(0.0, 1.0);
            let white = cover(edge + BUBBLE_OUTLINE) * (1.0 - cover(dot));
            let shade = INK.to_srgba().red + (1.0 - INK.to_srgba().red) * white;
            let value = (shade * 255.0).round() as u8;
            data.extend_from_slice(&[value, value, value, (cover(edge) * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// How far `at` is from the edge of the bubble, its circle and its tail, in
/// shares of the circle's radius: less than nothing inside it.
fn bubble_distance(at: Vec2) -> f32 {
    let circle = at.length() - 1.0;
    let tail = hud::triangle_distance(at, TAIL_FROM, TAIL_TIP, TAIL_TO) - TAIL_ROUND;
    // The two run into each other smoothly, rather than meeting at a corner.
    let h = (0.5 + 0.5 * (tail - circle) / TAIL_JOIN).clamp(0.0, 1.0);
    tail + (circle - tail) * h - TAIL_JOIN * h * (1.0 - h)
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
    fn the_bubble_goes_over_the_nearest_in_front_of_you() {
        let mut world = World::new();
        let [near, far, behind, aside, off] = [(); 5].map(|_| world.spawn_empty().id());
        let them = [
            (far, at(0.5, -4.0)),
            (near, at(-0.8, -2.5)),
            // Nearer, but behind you, to your side, and out of reach.
            (behind, at(0.0, 1.0)),
            (aside, at(2.0, -0.2)),
            (off, at(0.0, -6.0)),
        ];
        let open = |_: Vec3, _: Vec3| true;
        assert_eq!(nearest_in_front(&you(), them.into_iter(), None, open), Some(near));
        // Turned round, the one behind you is the one in front.
        let turned = you().looking_to(Vec3::Z, Vec3::Y);
        assert_eq!(nearest_in_front(&turned, them.into_iter(), None, open), Some(behind));
        // Nobody in front: no bubble.
        let away = you().looking_to(Vec3::NEG_X, Vec3::Y);
        assert_eq!(nearest_in_front(&away, [(aside, at(2.0, 0.0))].into_iter(), None, open), None);
    }

    #[test]
    fn a_wall_between_you_keeps_the_bubble_off_them() {
        let mut world = World::new();
        let [near, far] = [(); 2].map(|_| world.spawn_empty().id());
        let them = [(near, at(0.0, -2.0)), (far, at(0.0, -4.0))];
        // Only the further one is in sight.
        let sight = |_: Vec3, to: Vec3| to.z < -3.0;
        assert_eq!(nearest_in_front(&you(), them.into_iter(), None, sight), Some(far));
    }

    #[test]
    fn the_bubble_stays_put_until_someone_is_clearly_nearer() {
        let mut world = World::new();
        let [held, other] = [(); 2].map(|_| world.spawn_empty().id());
        let open = |_: Vec3, _: Vec3| true;
        // A little nearer is not enough to take it.
        let them = [(held, at(0.0, -3.0)), (other, at(0.3, -2.7))];
        assert_eq!(nearest_in_front(&you(), them.into_iter(), Some(held), open), Some(held));
        // Well nearer is.
        let them = [(held, at(0.0, -3.0)), (other, at(0.3, -2.0))];
        assert_eq!(nearest_in_front(&you(), them.into_iter(), Some(held), open), Some(other));
        // Once it has them, they keep it a little past where it came up:
        // further off, and further round.
        let them = [(held, at(0.0, -4.8))];
        assert_eq!(nearest_in_front(&you(), them.into_iter(), Some(held), open), Some(held));
        assert_eq!(nearest_in_front(&you(), them.into_iter(), None, open), None);
        let round = 75f32.to_radians();
        let them = [(held, Vec3::new(round.sin(), 0.0, -round.cos()) * 3.0)];
        assert_eq!(nearest_in_front(&you(), them.into_iter(), Some(held), open), Some(held));
        assert_eq!(nearest_in_front(&you(), them.into_iter(), None, open), None);
    }

    #[test]
    fn nobody_says_the_same_thing_twice_running() {
        for _ in 0..500 {
            let last = fastrand::usize(..SMALL_TALK.len());
            let next = small_talk(Some(last));
            assert_ne!(next, last);
            assert!(next < SMALL_TALK.len());
        }
        // Everything is said, sooner or later.
        let mut said = vec![false; SMALL_TALK.len()];
        for _ in 0..2000 {
            said[small_talk(None)] = true;
        }
        assert!(said.iter().all(|&said| said));
    }

    #[test]
    fn everything_said_is_plain_ascii() {
        // The built-in font has nothing else.
        for (says, back) in SMALL_TALK {
            assert!(says.is_ascii() && back.is_ascii(), "{says} / {back}");
        }
        assert!(builder::pitch(false).is_ascii());
    }

    #[test]
    fn the_bubble_is_a_white_circle_outlined_in_black_with_three_dots() {
        let image = bubble_image();
        let (width, height) = (image.width(), image.height());
        let pixel = |x: f32, y: f32| {
            // From shares of the circle's radius, y up, to the pixel there.
            let column = ((x - BUBBLE_LEFT) / (BUBBLE_RIGHT - BUBBLE_LEFT) * width as f32) as u32;
            let row = ((BUBBLE_TOP - y) / (BUBBLE_TOP - BUBBLE_BOTTOM) * height as f32) as u32;
            let i = ((row.min(height - 1) * width + column.min(width - 1)) * 4) as usize;
            let data = image.data.as_ref().unwrap();
            [data[i], data[i + 1], data[i + 2], data[i + 3]]
        };
        // White inside, between the dots.
        assert_eq!(pixel(0.0, 0.5), [255, 255, 255, 255]);
        assert_eq!(pixel(-0.21, 0.0), [255, 255, 255, 255]);
        // Black on the dots and round the edge.
        for x in [-DOT_APART, 0.0, DOT_APART] {
            assert!(pixel(x, 0.0)[0] < 30, "{x}");
        }
        let rim = pixel(0.0, 1.0 - BUBBLE_OUTLINE * 0.5);
        assert!(rim[0] < 30 && rim[3] == 255, "{rim:?}");
        // Nothing in the corners, nor right of the tail.
        assert_eq!(pixel(0.95, 0.95)[3], 0);
        assert_eq!(pixel(0.95, -1.3)[3], 0);
        // The tail reaches down to its point, and no further.
        assert_eq!(pixel(TAIL_TIP.x + 0.02, TAIL_TIP.y + 0.08)[3], 255);
        assert_eq!(pixel(TAIL_TIP.x, TAIL_TIP.y - 0.08)[3], 0);
        // Its point is where the bubble is put, inside the picture.
        assert!(BUBBLE_TIP.x > 0.0 && BUBBLE_TIP.x < 0.5 && BUBBLE_TIP.y > 0.9 && BUBBLE_TIP.y < 1.0);
    }
}
