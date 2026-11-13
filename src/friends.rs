//! Friends: the panel behind the speech bubble under the gift, down the
//! right edge of the screen (`hud::FriendsButton`; since 2026-11-13, Hajun,
//! with four sketches of it, `friendtabdesign1.png` to `4`: "The friend
//! panel popup will open when u tap the top right corner chat bubblt icon
//! under the gift icon ... you should be able to search the username, and
//! it will give u a list of everyone with that username and u can choose
//! which one to add, and when u add the other person, will get a small 1
//! badge next to the chat icon, and the "message" tab inside the panel,
//! andthey can accept or reject the add friend request ... inviting to
//! group should also land in the same panel's messaging icon tab with the
//! small 1 circular badge in gold background as a subscript of the chat
//! icon").
//!
//! Everything about friends is the server's (`roundtown_net::social`,
//! `server/src/social.rs`): this shows what it last said ([`Friends`]) and
//! asks it for whatever is pressed. Offline there are no friends, and the
//! panel says so.
//!
//! **The panel** ([`draw`]) is dark and see-through, as sketched, with two
//! tabs along its top. **Friends**: your group, if you are in one, with
//! Leave; and your friends, whoever is here first, each with where they
//! are, their level, a button that asks them into your group (the sketch's
//! question comes first, "Would you like to invite ... to your group?"),
//! and a gear with Message and Remove Friend under it. The person with a
//! plus by the heading opens **Add Friend**: a name is typed and looked
//! for, everyone of that name comes back with their tags, and "Add friend"
//! beside one asks them; under them is your own name and tag, with a
//! button that copies it. **The envelope** is your inbox: who has asked to
//! be your friend, who has asked you into their group, and what your
//! friends have written, each with its answer beside it; how many wait
//! there is counted in gold on its corner, and on the speech bubble's
//! ([`hang_badge`]).
//!
//! It is a dialog (`hud::DialogUp`): you stand still, and presses are its
//! own; the cross, a tap outside it, or Escape shuts it. It is drawn again
//! whole whenever what it shows changes ([`redraw`]), as the settings'
//! window is, which keeps what is typed (`signup::NameBox`) and how far a
//! list is scrolled ([`Scrolled`]).
//!
//! **A group's leader starts its games** (`games`): everyone of it who is
//! free to goes in with them, and is told so across the screen
//! ([`hear`]); anyone else in a group finds the game selector's cards
//! blocked out.

use bevy::clipboard::Clipboard;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;
use bevy::window::PrimaryWindow;
use roundtown_net::social::{
    Act, Card, Found, GROUP_MOST, Group, MESSAGE_LONGEST, Mail, Minigame, News, Outcome, Post, Presence, SOUGHT_LONGEST, Tie,
    handle, message,
};
use roundtown_net::{Ask, Refusal, Tell, profile};

use crate::CloseUp;
use crate::builder::{self, DIALOG_LAYER, Fill};
use crate::clock::TownClock;
use crate::hud::{self, DialogUp, FriendsButton, TakesPress};
use crate::net::Online;
use crate::signup::{self, LiftsForKeyboard, NameBox};
use crate::ui::{UiChanged, fill, ui};

/// Sizes, as shares of the short side of the screen, off Hajun's sketches
/// (874 by 402, so a share is four of their pixels): the panel and its
/// padding; a tab, its picture and the line under it; the cross; a
/// heading; a row of the lists, the disc with someone's picture, their
/// name and the line under it, and the round buttons at its end; the
/// boxes that come up over the panel and what is in them; the gear's menu;
/// the count on a corner; and the line that says what came of something.
const WIDTH: f32 = 112.0;
const HEIGHT: f32 = 66.0;
const PAD: f32 = 3.6;
const ROUNDED: f32 = 3.0;
const TAB_WIDTH: f32 = 19.0;
const TAB_HEIGHT: f32 = 8.2;
const TAB_ICON: f32 = 5.2;
const LINE: f32 = 0.4;
const CLOSE: f32 = 7.0;
const HEADING: f32 = 3.1;
const ROW: f32 = 10.4;
const AVATAR: f32 = 7.6;
const NAME: f32 = 3.3;
const STATUS: f32 = 2.3;
const ROW_BUTTON: f32 = 7.6;
const SMALL_BUTTON: f32 = 5.6;
const POPUP_TEXT: f32 = 3.2;
const POPUP_TITLE: f32 = 3.6;
const CHOICE_WIDTH: f32 = 18.0;
const CHOICE_HEIGHT: f32 = 6.4;
const CHOICE_TEXT: f32 = 3.0;
const FIELD_HEIGHT: f32 = 7.4;
const FIELD_TEXT: f32 = 3.0;
/// Add Friend's box: wide enough for the longest name there is with its
/// tag after it, beside "Add friend".
const ADD_WIDTH: f32 = 76.0;
const FOUND_ROW: f32 = 6.8;
const FOUND_AVATAR: f32 = 5.2;
const LINK_TEXT: f32 = 2.8;
const MENU_WIDTH: f32 = 27.0;
const MENU_ROW: f32 = 6.8;
const MENU_TEXT: f32 = 3.0;
const BADGE: f32 = 4.6;
const NOTICE_TEXT: f32 = 2.8;
/// How long the line that says what came of something stays, in seconds.
const NOTICE_SECS: f32 = 3.5;
/// How far a press may be dragged and still be a tap on what it began on,
/// rather than a drag to scroll a list.
const TAP_SLOP: f32 = 16.0;
/// A line said across the screen about your group ([`toast`]): how big,
/// how far down the screen, as a share of its height, and for how long, in
/// seconds.
const TOAST_TEXT: f32 = 4.0;
const TOAST_DOWN: f32 = 66.0;
const TOAST_SECS: f32 = 3.5;
/// How often the list of friends is drawn again with nothing else changed,
/// in seconds, while a friend who is gone is on it: how long they have been
/// gone is written in whole minutes.
const AGAIN_EVERY: f32 = 60.0;

/// What the server last said of your friends, your inbox and your group:
/// nothing, offline.
#[derive(Resource, Default, Debug, PartialEq)]
pub(crate) struct Friends {
    /// The tag after your name, and your level.
    me: Option<(u16, u16)>,
    friends: Vec<Card>,
    inbox: Vec<Mail>,
    group: Option<Group>,
    /// What was last looked for, and who goes by it.
    found: Option<(String, Vec<Found>)>,
}

impl Friends {
    /// Whether you, whose account is `me`, are in a group someone else
    /// leads: its games are theirs to start (`games`).
    pub(crate) fn follows(&self, me: u64) -> bool {
        self.group.as_ref().is_some_and(|group| group.leader != me)
    }

    /// Your friends as the panel lists them: whoever is here first, and
    /// then by name.
    fn listed(&self) -> Vec<&Card> {
        let mut listed: Vec<&Card> = self.friends.iter().collect();
        listed.sort_by_key(|card| (!card.presence.online(), card.name.to_lowercase(), card.id));
        listed
    }

    /// Whether `card`, a friend, can be asked into your group just now by
    /// you, whose account is `me`: they are here, and are not in it; and
    /// either you are in none, or you lead yours and it has room.
    fn can_invite(&self, me: Option<u64>, card: &Card) -> bool {
        card.presence.online()
            && self.group.as_ref().is_none_or(|group| {
                Some(group.leader) == me
                    && group.members.len() < GROUP_MOST
                    && group.members.iter().all(|member| member.id != card.id)
            })
    }
}

/// The panel: whether it is up, the tab it shows, and what is up over it.
#[derive(Resource, Default)]
struct Panel {
    open: bool,
    tab: Tab,
    over: Option<Over>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
enum Tab {
    #[default]
    Friends,
    Mail,
}

/// What is up over the panel.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Over {
    /// Add Friend: a name to look for, and who goes by it.
    Add,
    /// "Would you like to invite ... to your group?"
    Invite(u64),
    /// The gear's menu beside this friend, where on the screen it was
    /// pressed.
    Menu(u64, Vec2),
    /// "Remove ... from your friends?"
    Remove(u64),
    /// A message being written to this friend.
    Write(u64),
}

/// What a press on something of the panel's does.
#[derive(Component, Clone, Copy, PartialEq, Debug)]
enum Tap {
    /// Nothing: a button that cannot be pressed just now.
    Nothing,
    Close,
    /// Away with what is up over the panel.
    Back,
    Show(Tab),
    Add,
    Search,
    Befriend(u64),
    Copy,
    AskInvite(u64),
    Invite(u64),
    Menu(u64),
    Write(u64),
    Send,
    AskRemove(u64),
    Remove(u64),
    Answer(u64, bool),
    Leave,
    Kick(u64),
}

/// The presses on the panel since they were last acted on ([`act`]), each
/// with where on the screen it landed; and that the speech bubble was
/// pressed.
#[derive(Resource, Default)]
struct Pressed {
    taps: Vec<(Tap, Vec2)>,
    open: bool,
    /// How far the press now down has been dragged, so that a drag to
    /// scroll a list is not also a tap on what it began on.
    dragged: f32,
}

/// The line along the panel's bottom that says what came of something, and
/// how long it has been up.
#[derive(Resource, Default)]
struct Notice(Option<(String, f32)>);

/// How far each list was scrolled, to draw it there again.
#[derive(Resource, Default)]
struct Scrolled {
    friends: f32,
    mail: f32,
    found: f32,
}

/// The lists that scroll.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum List {
    Friends,
    Mail,
    Found,
}

/// The root of the panel, over the whole screen.
#[derive(Component)]
struct Root;

/// The line that says what came of something.
#[derive(Component)]
struct NoticeLine;

/// A line across the screen, and how long it has been up.
#[derive(Component)]
struct Toast(f32);

/// The count on the speech bubble's corner, and the number in it.
#[derive(Component)]
struct HudBadge;
#[derive(Component)]
struct BadgeCount;

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Friends>()
        .init_resource::<Panel>()
        .init_resource::<Pressed>()
        .init_resource::<Notice>()
        .init_resource::<Scrolled>()
        .add_observer(press)
        .add_observer(start_press)
        .add_systems(
            Update,
            (hear, forget_offline, act, redraw, show_notice, remember_scroll, hang_badge, count_badge, clear_toasts).chain(),
        );
}

// ------------------------------------------------------------ what is heard

/// What the server says about your friends, into [`Friends`]; what came of
/// something asked, onto the panel's bottom line; and, taken into a game by
/// your group's leader, the panel put away and the game named across the
/// screen.
#[allow(clippy::too_many_arguments)]
fn hear(
    mut tells: MessageReader<Tell>,
    mut friends: ResMut<Friends>,
    mut panel: ResMut<Panel>,
    mut notice: ResMut<Notice>,
    mut up: ResMut<DialogUp>,
    mut name_box: ResMut<NameBox>,
    online: Res<Online>,
    mut commands: Commands,
) {
    for tell in tells.read() {
        let ui = ui();
        let words = &ui.words.friends;
        match tell {
            Tell::Social(news) => match news {
                News::Me { tag, level } => friends.me = Some((*tag, *level)),
                News::Friends(cards) => friends.friends.clone_from(cards),
                News::Friend(card) => {
                    if let Some(known) = friends.friends.iter_mut().find(|known| known.id == card.id) {
                        *known = card.clone();
                    }
                }
                News::Inbox(mail) => friends.inbox.clone_from(mail),
                News::Group(group) => friends.group.clone_from(group),
                News::Found { sought, people } => friends.found = Some((sought.clone(), people.clone())),
                // Left behind, the panel is as likely shut: said across the
                // screen.
                News::Said(Outcome::LeftBehind) => toast(&mut commands, &words.said.left_behind),
                News::Said(said) => notice.0 = Some((said_in_words(*said), 0.0)),
            },
            // Seated in a game without having asked: the leader of your
            // group did.
            Tell::Seated { .. } | Tell::Joined { .. } => {
                let follows = online.me().is_some_and(|me| friends.follows(me.id));
                if follows {
                    let game = match tell {
                        Tell::Seated { .. } => &ui.words.games.bridge_wars,
                        _ => &ui.words.games.home_builders,
                    };
                    toast(&mut commands, &fill(&words.summoned, &[("game", game)]));
                }
                if panel.open {
                    shut(&mut panel, &mut up, &mut name_box);
                }
            }
            Tell::Refused(Refusal::NotLeader) => toast(&mut commands, &words.only_leader),
            _ => {}
        }
    }
}

/// A line across the lower middle of the screen for a few seconds, over
/// whatever else is up: what is said to you about your group while the
/// panel is as likely shut. Lower than a headline, which a game puts its
/// own name in as you arrive in it, and clear of you in the middle.
fn toast(commands: &mut Commands, text: &str) {
    commands.spawn((
        Toast(0.0),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            right: Val::ZERO,
            top: Val::Vh(TOAST_DOWN),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(DIALOG_LAYER + 6),
        Pickable::IGNORE,
        children![hud::words(text, TOAST_TEXT, ui().colours.text)],
    ));
}

/// Takes each line across the screen away once it has been up its while.
fn clear_toasts(time: Res<Time>, mut toasts: Query<(Entity, &mut Toast)>, mut commands: Commands) {
    for (entity, mut toast) in &mut toasts {
        toast.0 += time.delta_secs();
        if toast.0 > TOAST_SECS {
            commands.entity(entity).despawn();
        }
    }
}

/// What the server made of something asked, in the file's words.
fn said_in_words(said: Outcome) -> String {
    let ui = ui();
    let words = &ui.words.friends.said;
    match said {
        Outcome::Asked => &words.asked,
        Outcome::Befriended => &words.befriended,
        Outcome::Unfriended => &words.unfriended,
        Outcome::Already => &words.already,
        Outcome::Full => &words.full,
        Outcome::TheirsFull => &words.theirs_full,
        Outcome::NoSuch => &words.no_such,
        Outcome::NotFriend => &words.not_friend,
        Outcome::Offline => &words.offline,
        Outcome::Invited => &words.invited,
        Outcome::NotLeader => &words.not_leader,
        Outcome::GroupFull => &words.group_full,
        Outcome::Stale => &words.stale,
        Outcome::Sent => &words.sent,
        Outcome::InboxFull => &words.inbox_full,
        Outcome::NotAMessage => &words.not_a_message,
        Outcome::LeftBehind => &words.left_behind,
    }
    .clone()
}

/// Offline, there are no friends: whatever the server last said is
/// forgotten, until it says it again.
fn forget_offline(online: Res<Online>, mut friends: ResMut<Friends>) {
    if !online.is() && *friends != Friends::default() {
        *friends = Friends::default();
    }
}

// ------------------------------------------------------------ what is pressed

/// A new press: nothing dragged yet.
fn start_press(_press: On<Pointer<Press>>, mut pressed: ResMut<Pressed>) {
    pressed.dragged = 0.0;
}

/// Whatever of the panel's was tapped or clicked, or the speech bubble, to
/// be acted on ([`act`]). Only what the press landed on, not what the
/// click passed up through on its way out, so that a tap on a row is not
/// also a tap outside the panel; and not a press dragged to scroll a list.
fn press(
    click: On<Pointer<Click>>,
    taps: Query<&Tap>,
    bubbles: Query<(), With<FriendsButton>>,
    mut pressed: ResMut<Pressed>,
) {
    if click.button != PointerButton::Primary || click.original_event_target() != click.entity {
        return;
    }
    if bubbles.contains(click.entity) {
        pressed.open = true;
    }
    if let Ok(&tap) = taps.get(click.entity)
        && pressed.dragged <= TAP_SLOP
    {
        let at = click.pointer_location.position;
        pressed.taps.push((tap, at));
    }
}

/// Scrolls a list with a finger, or a mouse held down, dragged over it.
fn drag_list(drag: On<Pointer<Drag>>, mut lists: Query<(&ComputedNode, &mut ScrollPosition), With<List>>, mut pressed: ResMut<Pressed>) {
    pressed.dragged = pressed.dragged.max(drag.distance.length());
    let Ok((node, mut scroll)) = lists.get_mut(drag.entity) else {
        return;
    };
    let shown = node.size().y * node.inverse_scale_factor();
    let whole = node.content_size().y * node.inverse_scale_factor();
    let most = (whole - shown).max(0.0);
    let to = (scroll.y - drag.delta.y).clamp(0.0, most);
    if scroll.y != to {
        scroll.y = to;
    }
}

/// Puts the panel away.
fn shut(panel: &mut Panel, up: &mut DialogUp, name_box: &mut NameBox) {
    if matches!(panel.over, Some(Over::Add | Over::Write(_))) {
        name_box.close();
    }
    panel.open = false;
    panel.over = None;
    if up.0 {
        up.0 = false;
    }
}

/// Does what was pressed: the speech bubble opens the panel, and the
/// panel's own buttons go on from there, asking the server for whatever is
/// its to do. On desktop `Escape` backs out of what is up over the panel,
/// or shuts it, and `Enter` looks for the name typed, or sends the message.
#[allow(clippy::too_many_arguments)]
fn act(
    mut pressed: ResMut<Pressed>,
    mut panel: ResMut<Panel>,
    mut friends: ResMut<Friends>,
    mut name_box: ResMut<NameBox>,
    mut up: ResMut<DialogUp>,
    (online, close_up, keyboard): (Res<Online>, Res<CloseUp>, Res<ButtonInput<KeyCode>>),
    mut clipboard: ResMut<Clipboard>,
    mut notice: ResMut<Notice>,
    mut asks: MessageWriter<Ask>,
) {
    if pressed.open {
        pressed.open = false;
        // With nothing else up, and not while the camera is on someone
        // being talked to.
        if !panel.open && !up.0 && !close_up.busy() {
            panel.open = true;
            panel.over = None;
            up.0 = true;
        }
    }
    let mut taps: Vec<(Tap, Vec2)> = if pressed.taps.is_empty() { Vec::new() } else { std::mem::take(&mut pressed.taps) };
    if !panel.open {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        taps.push((if panel.over.is_some() { Tap::Back } else { Tap::Close }, Vec2::ZERO));
    }
    let typing = matches!(panel.over, Some(Over::Add | Over::Write(_)));
    if typing && (name_box.entered() || keyboard.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter])) {
        let tap = if panel.over == Some(Over::Add) { Tap::Search } else { Tap::Send };
        taps.push((tap, Vec2::ZERO));
    }
    let mut ask = |act: Act| {
        asks.write(Ask::Social(act));
    };
    for (tap, at) in taps {
        // Whatever is typed into goes with the box it was typed in.
        let leaves_typing = !matches!(tap, Tap::Search | Tap::Send | Tap::Befriend(_) | Tap::Copy | Tap::Nothing);
        if leaves_typing && matches!(panel.over, Some(Over::Add | Over::Write(_))) {
            name_box.close();
        }
        match tap {
            Tap::Nothing => {}
            Tap::Close => shut(&mut panel, &mut up, &mut name_box),
            Tap::Back => panel.over = None,
            Tap::Show(tab) => {
                panel.tab = tab;
                panel.over = None;
            }
            Tap::Add => {
                panel.over = Some(Over::Add);
                name_box.open_for("", SOUGHT_LONGEST);
                friends.found = None;
            }
            Tap::Search => {
                let sought = name_box.text().trim().to_string();
                if sought.chars().count() >= profile::SHORTEST {
                    ask(Act::Find(sought));
                }
            }
            // Asked, and looked for again, so that what stands beside them
            // is what they are to you now.
            Tap::Befriend(id) => {
                ask(Act::Befriend(id));
                if let Some((sought, _)) = &friends.found {
                    ask(Act::Find(sought.clone()));
                }
            }
            Tap::Copy => {
                if let (Some(me), Some((tag, _))) = (online.me(), friends.me)
                    && copy(&handle(&me.name, tag), &mut clipboard)
                {
                    notice.0 = Some((ui().words.friends.copied.clone(), 0.0));
                }
            }
            Tap::AskInvite(id) => panel.over = Some(Over::Invite(id)),
            Tap::Invite(id) => {
                ask(Act::Invite(id));
                panel.over = None;
            }
            Tap::Menu(id) => panel.over = Some(Over::Menu(id, at)),
            Tap::Write(id) => {
                panel.over = Some(Over::Write(id));
                name_box.open_for("", MESSAGE_LONGEST);
            }
            Tap::Send => {
                if let (Some(Over::Write(to)), Some(text)) = (panel.over, message(name_box.text())) {
                    ask(Act::Write { to, text });
                    name_box.close();
                    panel.over = None;
                }
            }
            Tap::AskRemove(id) => panel.over = Some(Over::Remove(id)),
            Tap::Remove(id) => {
                ask(Act::Unfriend(id));
                panel.over = None;
            }
            Tap::Answer(mail, yes) => ask(Act::Answer { mail, yes }),
            Tap::Leave => ask(Act::LeaveGroup),
            Tap::Kick(id) => ask(Act::Kick(id)),
        }
    }
}

/// What is to be copied to an Android phone's clipboard, until its
/// activity has taken it. Bevy's own clipboard reaches the system's on
/// desktop and nowhere on a phone, so `MainActivity` looks here a few times
/// a second (`clipLength`, `clipByte`, `clipTaken`), by numbers alone:
/// nothing of Java's is touched from this side, as the keyboard's height is
/// handed over (`signup::IME_INSET`).
#[cfg(target_os = "android")]
static TO_COPY: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());

/// `MainActivity.clipLength()`: how many bytes wait to be copied, or none.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_clipLength(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) -> i32 {
    TO_COPY.lock().map_or(0, |text| text.len() as i32)
}

/// `MainActivity.clipByte(int)`: the byte at `index` of what waits, or -1.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_clipByte(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    index: i32,
) -> i32 {
    let Ok(text) = TO_COPY.lock() else {
        return -1;
    };
    usize::try_from(index)
        .ok()
        .and_then(|index| text.get(index))
        .map_or(-1, |&byte| i32::from(byte))
}

/// `MainActivity.clipTaken()`: what waited has been copied.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_clipTaken(_env: *mut std::ffi::c_void, _class: *mut std::ffi::c_void) {
    if let Ok(mut text) = TO_COPY.lock() {
        text.clear();
    }
}

/// Copies `text` to the clipboard: the system's on desktop, and on Android
/// by way of its activity. Whether it could be.
fn copy(text: &str, clipboard: &mut Clipboard) -> bool {
    #[cfg(target_os = "android")]
    if let Ok(mut waiting) = TO_COPY.lock() {
        *waiting = text.as_bytes().to_vec();
    }
    clipboard.set_text(text.to_string()).is_ok()
}

// --------------------------------------------------------------- the panel

/// Everything [`draw`] draws the panel from.
struct View<'a> {
    panel: &'a Panel,
    friends: &'a Friends,
    /// Your account and the name you go by, online.
    me: Option<(u64, &'a str)>,
    /// The town's time, in seconds since 1970, for how long someone has
    /// been gone.
    now: f64,
    /// What is typed so far, if anything is being.
    typed: &'a str,
    scrolled: &'a Scrolled,
    /// The window, in logical pixels.
    window: Vec2,
}

/// Puts the panel up while it is open, and draws it again whenever what it
/// shows has changed: what the server said, what is up over it, the file's
/// colours and words. Takes it down once it is shut.
#[allow(clippy::too_many_arguments)]
fn redraw(
    panel: Res<Panel>,
    friends: Res<Friends>,
    changed: Res<UiChanged>,
    online: Res<Online>,
    clock: Res<TownClock>,
    name_box: Res<NameBox>,
    scrolled: Res<Scrolled>,
    windows: Query<&Window, With<PrimaryWindow>>,
    roots: Query<Entity, With<Root>>,
    assets: Res<AssetServer>,
    time: Res<Time>,
    (mut was_online, mut drawn_for): (Local<bool>, Local<f32>),
    mut commands: Commands,
) {
    if !panel.open {
        for root in &roots {
            commands.entity(root).despawn();
        }
        return;
    }
    let came_or_went = std::mem::replace(&mut *was_online, online.is()) != online.is();
    // How long a friend has been gone is written in whole minutes, and
    // would be left standing with nothing else changing.
    *drawn_for += time.delta_secs();
    let aged = *drawn_for >= AGAIN_EVERY
        && panel.tab == Tab::Friends
        && panel.over.is_none()
        && friends.friends.iter().any(|card| matches!(card.presence, Presence::Offline { since } if since != 0));
    let stale = panel.is_changed() || friends.is_changed() || (changed.is_changed() && !changed.is_added()) || came_or_went || aged;
    if !stale && !roots.is_empty() {
        return;
    }
    *drawn_for = 0.0;
    for root in &roots {
        commands.entity(root).despawn();
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let view = View {
        panel: &panel,
        friends: &friends,
        me: online.me().map(|me| (me.id, me.name.as_str())),
        now: clock.now(),
        typed: if name_box.up() { name_box.text() } else { "" },
        scrolled: &scrolled,
        window: Vec2::new(window.width(), window.height()),
    };
    draw(&mut commands, &assets, &view);
}

/// Words on the panel, which has a colour of its own behind them and needs
/// no shadow: on one line, from the left.
fn words(text: impl Into<String>, size: f32, colour: Color) -> impl Bundle {
    (
        Text::new(text),
        crate::font::sized(size),
        TextColor(colour),
        TextLayout::no_wrap(),
        Pickable::IGNORE,
    )
}

/// A picture `size` across, in `tint`.
fn picture(image: Handle<Image>, size: Val, tint: Color) -> impl Bundle {
    (
        ImageNode::new(image).with_color(tint),
        Node {
            width: size,
            height: size,
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// The disc with someone's picture on it, `size` across, and the dot on
/// its corner that says whether they are here.
fn avatar(person: Handle<Image>, size: f32, here: bool) -> impl Bundle {
    let colours = ui().colours.friends;
    (
        Node {
            width: Val::VMin(size),
            height: Val::VMin(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(colours.avatar),
        Pickable::IGNORE,
        children![
            picture(person, Val::Percent(64.0), colours.avatar_mark),
            (
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::VMin(-size * 0.03),
                    bottom: Val::VMin(size * 0.02),
                    width: Val::VMin(size * 0.31),
                    height: Val::VMin(size * 0.31),
                    border: UiRect::all(Val::VMin(size * 0.045)),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(if here { colours.online } else { colours.offline }),
                BorderColor::all(colours.panel.with_alpha(1.0)),
                Pickable::IGNORE,
            ),
        ],
    )
}

/// A round button with `image` on it, `size` across, that does `tap`; or,
/// if it cannot be pressed just now, that does nothing and shows so.
fn round(image: Handle<Image>, size: f32, tap: Tap, able: bool) -> impl Bundle {
    let colours = ui().colours.friends;
    (
        if able { tap } else { Tap::Nothing },
        TakesPress,
        Button,
        Node {
            width: Val::VMin(size),
            height: Val::VMin(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(colours.button),
        Fill {
            idle: colours.button,
            lit: if able { colours.button_lit } else { colours.button },
        },
        children![picture(image, Val::Percent(58.0), if able { colours.button_mark } else { colours.button_off })],
    )
}

/// A button with a picture on it and nothing behind it, lit under the
/// pointer: the cross, the chevron back, the glass, the two sheets.
fn bare(image: Handle<Image>, size: f32, tap: Tap) -> impl Bundle {
    let colours = ui().colours.friends;
    (
        tap,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(size),
            height: Val::VMin(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::NONE),
        Fill {
            idle: Color::NONE,
            lit: colours.button_lit,
        },
        children![picture(image, Val::Percent(72.0), colours.button_mark)],
    )
}

/// How a button with words on it looks: green for yes and dark red for
/// no, as sketched, and the dark of the round buttons for one that only
/// puts something away.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Look {
    Yes,
    No,
    Plain,
}

/// A button with words on it, in `look`.
fn choice(text: impl Into<String>, tap: Tap, look: Look) -> impl Bundle {
    let colours = ui().colours.friends;
    let (idle, lit) = match look {
        Look::Yes => (colours.yes, colours.yes_lit),
        Look::No => (colours.no, colours.no_lit),
        Look::Plain => (colours.button, colours.button_lit),
    };
    (
        tap,
        TakesPress,
        Button,
        Node {
            width: Val::VMin(CHOICE_WIDTH),
            height: Val::VMin(CHOICE_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(0.8)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(idle),
        Fill { idle, lit },
        children![builder::label(text, CHOICE_TEXT, colours.on_choice)],
    )
}

/// Words that are pressed, or, when there is nothing to press, that only
/// say: "Add friend", "Requested".
fn link(text: impl Into<String>, tap: Tap, able: bool) -> impl Bundle {
    let colours = ui().colours.friends;
    (
        if able { tap } else { Tap::Nothing },
        TakesPress,
        Button,
        Node {
            padding: UiRect::axes(Val::VMin(1.6), Val::VMin(1.2)),
            border_radius: BorderRadius::all(Val::VMin(1.2)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(Color::NONE),
        Fill {
            idle: Color::NONE,
            lit: if able { colours.button_lit } else { Color::NONE },
        },
        children![builder::label(text, LINK_TEXT, if able { colours.link } else { colours.link_off })],
    )
}

/// How many wait, in gold, hung off the corner of whatever it is spawned
/// under: the envelope, the speech bubble.
fn badge(count: usize, marker: impl Bundle) -> impl Bundle {
    let colours = ui().colours.friends;
    (
        marker,
        Node {
            position_type: PositionType::Absolute,
            right: Val::VMin(-BADGE * 0.4),
            bottom: Val::VMin(-BADGE * 0.3),
            min_width: Val::VMin(BADGE),
            height: Val::VMin(BADGE),
            padding: UiRect::horizontal(Val::VMin(BADGE * 0.22)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::VMin(BADGE * 0.5)),
            ..default()
        },
        BackgroundColor(colours.badge),
        Pickable::IGNORE,
        children![(BadgeCount, builder::label(counted(count), BADGE * 0.66, colours.badge_text))],
    )
}

/// A count as a badge writes it: past nine, "9+".
fn counted(count: usize) -> String {
    if count > 9 { "9+".into() } else { count.to_string() }
}

/// A line that only says something, in a list with nothing else to show.
fn note(commands: &mut Commands, list: Entity, text: &str) {
    let colours = ui().colours.friends;
    commands.spawn((
        ChildOf(list),
        Node {
            padding: UiRect::axes(Val::VMin(0.4), Val::VMin(2.4)),
            ..default()
        },
        Pickable::IGNORE,
        children![words(text, HEADING, colours.status)],
    ));
}

/// Where a friend is, as the line under their name says it, at `now` by
/// the town's clock.
fn whereabouts(presence: &Presence, now: f64) -> String {
    let ui = ui();
    let words = &ui.words.friends;
    let game = |game: &Minigame| match game {
        Minigame::BridgeWars => &ui.words.games.bridge_wars,
        Minigame::HouseBuilder => &ui.words.games.home_builders,
    };
    match presence {
        Presence::Town { channel } => fill(&words.in_town, &[("channel", &(u32::from(*channel) + 1).to_string())]),
        Presence::Home => words.at_home.clone(),
        Presence::Playing { game: which, queued: true } => fill(&words.in_queue, &[("game", game(which))]),
        Presence::Playing { game: which, queued: false } => fill(&words.in_game, &[("game", game(which))]),
        Presence::Offline { since: 0 } => words.offline.clone(),
        Presence::Offline { since } => {
            let gone = (now - *since as f64).max(0.0);
            fill(&words.offline_for, &[("time", &how_long(gone))])
        }
    }
}

/// `seconds` as the file's words have it: a moment, so many minutes, hours
/// or days, whole ones, counted down.
fn how_long(seconds: f64) -> String {
    let ui = ui();
    let words = &ui.words.friends;
    let (count, unit) = if seconds < 60.0 {
        return words.moments.clone();
    } else if seconds < 3_600.0 {
        (seconds / 60.0, &words.minutes)
    } else if seconds < 86_400.0 {
        (seconds / 3_600.0, &words.hours)
    } else {
        (seconds / 86_400.0, &words.days)
    };
    let count = count as u64;
    fill(&unit[usize::from(count != 1)], &[("n", &count.to_string())])
}

/// The panel as things stand: its root, over the whole screen, which a tap
/// on shuts it.
fn draw(commands: &mut Commands, assets: &AssetServer, view: &View) -> Entity {
    let colours = ui().colours.friends;
    let root = commands
        .spawn((
            Root,
            Tap::Close,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(DIALOG_LAYER),
        ))
        .id();
    let panel = commands
        .spawn((
            ChildOf(root),
            Node {
                width: Val::VMin(WIDTH),
                height: Val::VMin(HEIGHT),
                flex_direction: FlexDirection::Column,
                // More at the bottom, where the line that says what came of
                // something goes.
                padding: UiRect {
                    bottom: Val::VMin(PAD + NOTICE_TEXT),
                    ..UiRect::all(Val::VMin(PAD))
                },
                border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
                ..default()
            },
            BackgroundColor(colours.panel),
        ))
        .id();
    header(commands, assets, panel, view);
    match view.panel.tab {
        Tab::Friends => friends_tab(commands, assets, panel, view),
        Tab::Mail => mail_tab(commands, assets, panel, view),
    }
    match view.panel.over {
        Some(Over::Add) => add_friend(commands, assets, panel, view),
        Some(Over::Invite(id)) => question(commands, panel, view, id, true),
        Some(Over::Remove(id)) => question(commands, panel, view, id, false),
        Some(Over::Write(id)) => writing(commands, panel, view, id),
        Some(Over::Menu(id, at)) => menu(commands, root, view, id, at),
        None => {}
    }
    // What came of something, along the bottom, over whatever else is up.
    commands.spawn((
        ChildOf(panel),
        NoticeLine,
        Visibility::Hidden,
        builder::label("", NOTICE_TEXT, colours.notice),
        Node {
            position_type: PositionType::Absolute,
            left: Val::VMin(PAD),
            right: Val::VMin(PAD),
            bottom: Val::VMin(1.4),
            ..default()
        },
    ));
    root
}

/// The panel's top: its two tabs, who you are, and the cross.
fn header(commands: &mut Commands, assets: &AssetServer, panel: Entity, view: &View) {
    let ui = ui();
    let (colours, said) = (ui.colours.friends, &ui.words.friends);
    let row = commands
        .spawn((
            ChildOf(panel),
            Node {
                height: Val::VMin(TAB_HEIGHT),
                column_gap: Val::VMin(1.0),
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                // Clear of the cross on the corner.
                padding: UiRect::right(Val::VMin(CLOSE + 1.0)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for (tab, image, waiting) in [(Tab::Friends, "friends", 0), (Tab::Mail, "mail", view.friends.inbox.len())] {
        let shown = view.panel.tab == tab;
        let button = commands
            .spawn((
                ChildOf(row),
                Tap::Show(tab),
                TakesPress,
                Button,
                Node {
                    width: Val::VMin(TAB_WIDTH),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::bottom(Val::VMin(LINE)),
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(if shown { colours.line } else { Color::NONE }),
            ))
            .id();
        let mark = commands
            .spawn((
                ChildOf(button),
                Node {
                    width: Val::VMin(TAB_ICON),
                    height: Val::VMin(TAB_ICON),
                    ..default()
                },
                Pickable::IGNORE,
                children![picture(
                    assets.load(format!("icons/{image}.png")),
                    Val::Percent(100.0),
                    if shown { colours.line } else { colours.tab_off },
                )],
            ))
            .id();
        if waiting > 0 {
            commands.spawn((ChildOf(mark), badge(waiting, ())));
        }
    }
    // Who you are to your friends, at the right: your name and tag, and
    // your level.
    if let (Some((_, name)), Some((tag, level))) = (view.me, view.friends.me) {
        commands.spawn((
            ChildOf(row),
            Node {
                flex_grow: 1.0,
                justify_content: JustifyContent::FlexEnd,
                column_gap: Val::VMin(2.4),
                ..default()
            },
            Pickable::IGNORE,
            children![
                words(handle(name, tag), STATUS + 0.5, colours.status),
                words(fill(&said.level, &[("level", &level.to_string())]), STATUS + 0.5, colours.status),
            ],
        ));
    }
    commands.spawn((
        ChildOf(panel),
        Node {
            position_type: PositionType::Absolute,
            top: Val::VMin(PAD - 0.4),
            right: Val::VMin(PAD - 0.8),
            ..default()
        },
        Pickable::IGNORE,
        children![bare(assets.load("icons/close.png"), CLOSE, Tap::Close)],
    ));
}

/// A list that scrolls, filling what is left of whatever it is spawned
/// under, where it was last scrolled to.
fn list(commands: &mut Commands, parent: Entity, which: List, scrolled: f32) -> Entity {
    commands
        .spawn((
            ChildOf(parent),
            which,
            ScrollArea,
            Node {
                flex_grow: 1.0,
                min_height: Val::ZERO,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                margin: UiRect::top(Val::VMin(1.2)),
                ..default()
            },
            ScrollPosition(Vec2::new(0.0, scrolled)),
        ))
        .observe(drag_list)
        .id()
}

/// A heading in a list, with whatever goes beside it spawned under what
/// this gives back.
fn heading(commands: &mut Commands, list: Entity, text: String) -> Entity {
    let colours = ui().colours.friends;
    commands
        .spawn((
            ChildOf(list),
            Node {
                height: Val::VMin(SMALL_BUTTON + 0.8),
                align_items: AlignItems::Center,
                column_gap: Val::VMin(1.6),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
            children![words(text, HEADING, colours.title)],
        ))
        .id()
}

/// The Friends tab: your group, if you are in one, and your friends.
fn friends_tab(commands: &mut Commands, assets: &AssetServer, panel: Entity, view: &View) {
    let ui = ui();
    let said = &ui.words.friends;
    let list = list(commands, panel, List::Friends, view.scrolled.friends);
    let Some((me, _)) = view.me else {
        note(commands, list, &said.not_online);
        return;
    };
    if let Some(group) = &view.friends.group {
        let count = [("count", group.members.len().to_string()), ("most", GROUP_MOST.to_string())];
        let count: Vec<(&str, &str)> = count.iter().map(|(name, value)| (*name, value.as_str())).collect();
        let head = heading(commands, list, fill(&said.group, &count));
        commands.spawn((ChildOf(head), link(said.leave.clone(), Tap::Leave, true)));
        for member in &group.members {
            let mut name = member.name.clone();
            for (is, word) in [(member.id == group.leader, &said.leader), (member.id == me, &said.you)] {
                if is {
                    name.push_str(&format!("  ({word})"));
                }
            }
            let row = row(commands, assets, list, member, name, view.now);
            // Whoever leads puts the others out.
            if group.leader == me && member.id != me {
                commands.spawn((ChildOf(row), round(assets.load("icons/close.png"), ROW_BUTTON, Tap::Kick(member.id), true)));
            }
        }
    }
    let here = view.friends.friends.iter().filter(|card| card.presence.online()).count();
    let count = [("online", here.to_string()), ("all", view.friends.friends.len().to_string())];
    let count: Vec<(&str, &str)> = count.iter().map(|(name, value)| (*name, value.as_str())).collect();
    let head = heading(commands, list, fill(&said.friends, &count));
    commands.spawn((ChildOf(head), bare(assets.load("icons/add_friend.png"), SMALL_BUTTON, Tap::Add)));
    if view.friends.friends.is_empty() {
        note(commands, list, &said.no_friends);
    }
    for card in view.friends.listed() {
        let row = row(commands, assets, list, card, card.name.clone(), view.now);
        let can_invite = view.friends.can_invite(Some(me), card);
        commands.spawn((ChildOf(row), round(assets.load("icons/groups.png"), ROW_BUTTON, Tap::AskInvite(card.id), can_invite)));
        commands.spawn((ChildOf(row), round(assets.load("icons/settings.png"), ROW_BUTTON, Tap::Menu(card.id), true)));
    }
}

/// One of a list's rows: someone's picture, their `name` over where they
/// are, and their level; whatever buttons go at its end are spawned under
/// what this gives back.
fn row(commands: &mut Commands, assets: &AssetServer, list: Entity, card: &Card, name: String, now: f64) -> Entity {
    let ui = ui();
    let (colours, said) = (ui.colours.friends, &ui.words.friends);
    let here = card.presence.online();
    let ink = if here { colours.name } else { colours.name_off };
    commands
        .spawn((
            ChildOf(list),
            Node {
                height: Val::VMin(ROW),
                align_items: AlignItems::Center,
                column_gap: Val::VMin(2.2),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
            children![
                avatar(assets.load("icons/person.png"), AVATAR, here),
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        min_width: Val::ZERO,
                        row_gap: Val::VMin(0.3),
                        overflow: Overflow::clip_x(),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![words(name, NAME, ink), words(whereabouts(&card.presence, now), STATUS, colours.status)],
                ),
                (
                    Node {
                        margin: UiRect::right(Val::VMin(1.0)),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![words(fill(&said.level, &[("level", &card.level.to_string())]), NAME, ink)],
                ),
            ],
        ))
        .id()
}

/// The envelope's tab: who has asked to be your friend, who has asked you
/// into their group, and what your friends have written, the newest first,
/// each with its answer beside it.
fn mail_tab(commands: &mut Commands, assets: &AssetServer, panel: Entity, view: &View) {
    let ui = ui();
    let (colours, said) = (ui.colours.friends, &ui.words.friends);
    let list = list(commands, panel, List::Mail, view.scrolled.mail);
    if view.me.is_none() {
        note(commands, list, &said.not_online);
        return;
    }
    if view.friends.inbox.is_empty() {
        note(commands, list, &said.no_mail);
    }
    for mail in view.friends.inbox.iter().rev() {
        let line = match &mail.what {
            Post::Request => said.wants_friend.clone(),
            Post::Invite => said.wants_group.clone(),
            Post::Message(text) => text.clone(),
        };
        let row = commands
            .spawn((
                ChildOf(list),
                Node {
                    min_height: Val::VMin(ROW),
                    align_items: AlignItems::Center,
                    column_gap: Val::VMin(2.2),
                    padding: UiRect::vertical(Val::VMin(0.8)),
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
                children![
                    avatar(assets.load("icons/person.png"), AVATAR, mail.from.presence.online()),
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            flex_basis: Val::ZERO,
                            min_width: Val::ZERO,
                            row_gap: Val::VMin(0.3),
                            ..default()
                        },
                        Pickable::IGNORE,
                        children![
                            words(mail.from.handle(), NAME - 0.2, colours.name),
                            // What a friend wrote may run to a second line.
                            (
                                Text::new(line),
                                crate::font::sized(STATUS + 0.4),
                                TextColor(colours.status),
                                TextLayout::justify(Justify::Left),
                                Pickable::IGNORE,
                            ),
                        ],
                    ),
                ],
            ))
            .id();
        match mail.what {
            Post::Request => {
                commands.spawn((ChildOf(row), choice(said.accept.clone(), Tap::Answer(mail.id, true), Look::Yes)));
                commands.spawn((ChildOf(row), choice(said.reject.clone(), Tap::Answer(mail.id, false), Look::No)));
            }
            Post::Invite => {
                commands.spawn((ChildOf(row), choice(said.join.clone(), Tap::Answer(mail.id, true), Look::Yes)));
                commands.spawn((ChildOf(row), choice(said.reject.clone(), Tap::Answer(mail.id, false), Look::No)));
            }
            Post::Message(_) => {
                commands.spawn((ChildOf(row), choice(said.reply.clone(), Tap::Write(mail.from.id), Look::Yes)));
                commands.spawn((ChildOf(row), choice(said.seen.clone(), Tap::Answer(mail.id, true), Look::Plain)));
            }
        }
    }
}

/// A box up over the panel, the panel darkened behind it, which a tap on
/// the dark takes away: what goes in it is spawned under what this gives
/// back. One that is typed into slides up out from under a phone's
/// keyboard.
fn popup(commands: &mut Commands, panel: Entity, width: f32, typed_into: bool) -> Entity {
    let colours = ui().colours.friends;
    let dark = commands
        .spawn((
            ChildOf(panel),
            Tap::Back,
            TakesPress,
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
                ..default()
            },
            BackgroundColor(colours.scrim),
        ))
        .id();
    let mut within = commands.spawn((
        ChildOf(dark),
        Node {
            width: Val::VMin(width),
            flex_direction: FlexDirection::Column,
            row_gap: Val::VMin(2.0),
            padding: UiRect::all(Val::VMin(3.0)),
            border_radius: BorderRadius::all(Val::VMin(ROUNDED)),
            ..default()
        },
        BackgroundColor(colours.popup),
    ));
    if typed_into {
        within.insert(LiftsForKeyboard::default());
    }
    within.id()
}

/// Add Friend: a name to look for, everyone who goes by it, and your own
/// name and tag to give to someone looking for you.
fn add_friend(commands: &mut Commands, assets: &AssetServer, panel: Entity, view: &View) {
    let ui = ui();
    let (colours, said) = (ui.colours.friends, &ui.words.friends);
    let within = popup(commands, panel, ADD_WIDTH, true);
    commands.spawn((
        ChildOf(within),
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::VMin(1.4),
            ..default()
        },
        Pickable::IGNORE,
        children![
            bare(assets.load("icons/back.png"), SMALL_BUTTON, Tap::Back),
            words(said.add_friend.clone(), POPUP_TITLE, colours.title),
        ],
    ));
    // The name typed, in a pill, with the glass at its end.
    commands.spawn((
        ChildOf(within),
        Node {
            height: Val::VMin(FIELD_HEIGHT),
            align_items: AlignItems::Center,
            padding: UiRect::new(Val::VMin(3.2), Val::VMin(0.9), Val::ZERO, Val::ZERO),
            border_radius: BorderRadius::all(Val::VMin(FIELD_HEIGHT * 0.5)),
            ..default()
        },
        BackgroundColor(colours.field),
        Pickable::IGNORE,
        children![
            signup::field(
                view.typed,
                Node {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                FIELD_TEXT,
                Color::NONE,
                colours.name,
            ),
            bare(assets.load("icons/search.png"), SMALL_BUTTON, Tap::Search),
        ],
    ));
    // Who goes by it.
    let found = commands
        .spawn((
            ChildOf(within),
            List::Found,
            ScrollArea,
            Node {
                height: Val::VMin(FOUND_ROW * 3.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition(Vec2::new(0.0, view.scrolled.found)),
        ))
        .observe(drag_list)
        .id();
    match &view.friends.found {
        None => note(commands, found, &said.search_first),
        Some((_, people)) if people.is_empty() => note(commands, found, &said.nobody),
        Some((_, people)) => {
            for person in people {
                let (text, able) = match person.tie {
                    Tie::Stranger => (&said.add, true),
                    Tie::Asked => (&said.asked, false),
                    Tie::Friend => (&said.is_friend, false),
                    Tie::AskedYou => (&said.accept, true),
                };
                commands.spawn((
                    ChildOf(found),
                    Node {
                        height: Val::VMin(FOUND_ROW),
                        align_items: AlignItems::Center,
                        column_gap: Val::VMin(2.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![
                        avatar(assets.load("icons/person.png"), FOUND_AVATAR, person.card.presence.online()),
                        (
                            Node {
                                flex_grow: 1.0,
                                min_width: Val::ZERO,
                                overflow: Overflow::clip_x(),
                                ..default()
                            },
                            Pickable::IGNORE,
                            children![words(person.card.handle(), FIELD_TEXT, colours.name)],
                        ),
                        link(text.clone(), Tap::Befriend(person.card.id), able),
                    ],
                ));
            }
        }
    }
    // Your own, to give to someone looking for you, and the two sheets
    // that copy it. An iPhone's clipboard is out of the game's reach: no
    // sheets there.
    // The tag fainter than the name, as sketched.
    let (mine, tag) = match (view.me, view.friends.me) {
        (Some((_, name)), Some((tag, _))) => (name.to_string(), format!("#{tag}")),
        _ => default(),
    };
    let bar = commands
        .spawn((
            ChildOf(within),
            Node {
                height: Val::VMin(FIELD_HEIGHT),
                align_items: AlignItems::Center,
                column_gap: Val::VMin(2.6),
                padding: UiRect::new(Val::VMin(2.4), Val::VMin(0.9), Val::ZERO, Val::ZERO),
                border_radius: BorderRadius::all(Val::VMin(0.8)),
                ..default()
            },
            BackgroundColor(colours.field),
            Pickable::IGNORE,
            children![
                words(said.my_id.clone(), FIELD_TEXT, colours.status),
                (
                    Node {
                        flex_grow: 1.0,
                        min_width: Val::ZERO,
                        overflow: Overflow::clip_x(),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![words(mine, FIELD_TEXT, colours.name), words(tag, FIELD_TEXT, colours.name_off)],
                ),
            ],
        ))
        .id();
    if cfg!(not(target_os = "ios")) {
        commands.spawn((ChildOf(bar), bare(assets.load("icons/copy.png"), SMALL_BUTTON, Tap::Copy)));
    }
}

/// What the friend whose account is `id` goes by, as far as is known.
fn name_of(view: &View, id: u64) -> String {
    let known = view.friends.friends.iter().find(|card| card.id == id);
    let written = view.friends.inbox.iter().map(|mail| &mail.from).find(|card| card.id == id);
    known.or(written).map_or_else(String::new, |card| card.name.clone())
}

/// One of the two questions, as sketched: whether to ask this friend into
/// your group, or whether to have them your friend no longer; Cancel, and
/// Yes.
fn question(commands: &mut Commands, panel: Entity, view: &View, id: u64, invite: bool) {
    let ui = ui();
    let (colours, said) = (ui.colours.friends, &ui.words.friends);
    let (asked, yes) = if invite { (&said.invite_ask, Tap::Invite(id)) } else { (&said.remove_ask, Tap::Remove(id)) };
    let within = popup(commands, panel, 48.0, false);
    commands.spawn((
        ChildOf(within),
        builder::label(fill(asked, &[("name", &name_of(view, id))]), POPUP_TEXT, colours.title),
    ));
    commands.spawn((
        ChildOf(within),
        Node {
            justify_content: JustifyContent::Center,
            column_gap: Val::VMin(3.0),
            margin: UiRect::top(Val::VMin(0.6)),
            ..default()
        },
        Pickable::IGNORE,
        children![choice(said.cancel.clone(), Tap::Back, Look::No), choice(said.yes.clone(), yes, Look::Yes)],
    ));
}

/// A message being written to the friend whose account is `id`: what is
/// typed so far, Cancel, and Send.
fn writing(commands: &mut Commands, panel: Entity, view: &View, id: u64) {
    let ui = ui();
    let (colours, said) = (ui.colours.friends, &ui.words.friends);
    let within = popup(commands, panel, 80.0, true);
    commands.spawn((
        ChildOf(within),
        words(fill(&said.message_to, &[("name", &name_of(view, id))]), POPUP_TITLE, colours.title),
    ));
    commands.spawn((
        ChildOf(within),
        signup::field(
            view.typed,
            Node {
                width: Val::Percent(100.0),
                min_height: Val::VMin(FIELD_TEXT * 1.3 * 3.0 + 3.0),
                padding: UiRect::axes(Val::VMin(2.4), Val::VMin(1.5)),
                align_items: AlignItems::FlexStart,
                border_radius: BorderRadius::all(Val::VMin(0.8)),
                ..default()
            },
            FIELD_TEXT,
            colours.field,
            colours.name,
        ),
    ));
    commands.spawn((
        ChildOf(within),
        Node {
            justify_content: JustifyContent::Center,
            column_gap: Val::VMin(3.0),
            ..default()
        },
        Pickable::IGNORE,
        children![choice(said.cancel.clone(), Tap::Back, Look::No), choice(said.send.clone(), Tap::Send, Look::Yes)],
    ));
}

/// The gear's menu beside the friend whose account is `id`, where the gear
/// was pressed at `at`: Message, and Remove Friend. A tap anywhere else
/// takes it away.
fn menu(commands: &mut Commands, root: Entity, view: &View, id: u64, at: Vec2) {
    let ui = ui();
    let (colours, said) = (ui.colours.friends, &ui.words.friends);
    commands.spawn((
        ChildOf(root),
        Tap::Back,
        TakesPress,
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    // Hung from where the gear was pressed, to its left, and kept on the
    // screen.
    let share = view.window.min_element() / 100.0;
    let (wide, tall) = (MENU_WIDTH * share, 2.0 * MENU_ROW * share);
    let left = (at.x - wide + 2.0 * share).clamp(0.0, (view.window.x - wide).max(0.0));
    let top = (at.y + 1.0 * share).clamp(0.0, (view.window.y - tall).max(0.0));
    let entry = |text: &str, tap: Tap| {
        (
            tap,
            TakesPress,
            Button,
            Node {
                height: Val::VMin(MENU_ROW),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::VMin(2.2)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(colours.menu),
            BorderColor::all(colours.menu_rim),
            Fill {
                idle: colours.menu,
                lit: colours.menu_lit,
            },
            children![words(text.to_string(), MENU_TEXT, colours.name)],
        )
    };
    commands.spawn((
        ChildOf(root),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(left),
            top: Val::Px(top),
            width: Val::VMin(MENU_WIDTH),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Pickable::IGNORE,
        children![entry(&said.message, Tap::Write(id)), entry(&said.remove, Tap::AskRemove(id))],
    ));
}

// ------------------------------------------------------ what stays in step

/// Keeps the line along the panel's bottom saying what came of something,
/// for a while, and then nothing.
fn show_notice(time: Res<Time>, mut notice: ResMut<Notice>, mut lines: Query<(&mut Text, &mut Visibility), With<NoticeLine>>) {
    if let Some((_, age)) = notice.bypass_change_detection().0.as_mut() {
        *age += time.delta_secs();
        if *age > NOTICE_SECS {
            notice.0 = None;
        }
    }
    let (said, shown) = match &notice.0 {
        Some((said, _)) => (said.as_str(), Visibility::Inherited),
        None => ("", Visibility::Hidden),
    };
    for (mut text, mut visibility) in &mut lines {
        if text.0 != said {
            text.0 = said.to_string();
        }
        visibility.set_if_neq(shown);
    }
}

/// Remembers how far each list is scrolled, for when the panel is drawn
/// again.
fn remember_scroll(lists: Query<(&List, &ScrollPosition), Changed<ScrollPosition>>, mut scrolled: ResMut<Scrolled>) {
    for (list, at) in &lists {
        let kept = match list {
            List::Friends => &mut scrolled.friends,
            List::Mail => &mut scrolled.mail,
            List::Found => &mut scrolled.found,
        };
        *kept = at.y;
    }
}

/// Hangs the count on the speech bubble's corner as the bubble comes up:
/// as the app opens, and whenever the bar is put up afresh.
fn hang_badge(bubbles: Query<Entity, Added<FriendsButton>>, mut commands: Commands) {
    for bubble in &bubbles {
        commands.spawn((ChildOf(bubble), badge(0, (HudBadge, Visibility::Hidden))));
    }
}

/// Keeps the count on the speech bubble's corner what waits in your inbox,
/// and away with nothing waiting.
fn count_badge(
    friends: Res<Friends>,
    hung: Query<(), Added<HudBadge>>,
    mut badges: Query<(&mut Visibility, &Children), With<HudBadge>>,
    mut counts: Query<&mut Text, With<BadgeCount>>,
) {
    if !friends.is_changed() && hung.is_empty() {
        return;
    }
    let waiting = friends.inbox.len();
    for (mut visibility, children) in &mut badges {
        visibility.set_if_neq(if waiting > 0 { Visibility::Inherited } else { Visibility::Hidden });
        let mut numbers = counts.iter_many_mut(children);
        while let Some(mut text) = numbers.fetch_next() {
            let count = counted(waiting);
            if text.0 != count {
                text.0 = count;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(id: u64, name: &str, presence: Presence) -> Card {
        Card {
            id,
            name: name.into(),
            tag: 1000 + id as u16,
            level: 1,
            presence,
        }
    }

    const HERE: Presence = Presence::Town { channel: 0 };
    const GONE: Presence = Presence::Offline { since: 1_000_000 };

    #[test]
    fn friends_are_listed_whoever_is_here_first_and_then_by_name() {
        let friends = Friends {
            friends: vec![card(1, "bobby", GONE), card(2, "Zed", HERE), card(3, "Alba", GONE), card(4, "alex", HERE)],
            ..default()
        };
        let listed: Vec<u64> = friends.listed().iter().map(|card| card.id).collect();
        assert_eq!(listed, vec![4, 2, 3, 1]);
    }

    #[test]
    fn only_someone_here_and_not_in_your_group_can_be_asked_into_it_and_only_by_its_leader() {
        let (here, gone) = (card(2, "Bobby", HERE), card(3, "Dongi", GONE));
        // In no group: anyone who is here.
        let mut friends = Friends::default();
        assert!(friends.can_invite(Some(1), &here));
        assert!(!friends.can_invite(Some(1), &gone));
        assert!(!friends.follows(1));
        // Leading one: whoever is not in it, while it has room.
        friends.group = Some(Group {
            leader: 1,
            members: vec![card(1, "Hajooon", HERE), here.clone()],
        });
        assert!(!friends.can_invite(Some(1), &here), "in it already");
        assert!(friends.can_invite(Some(1), &card(4, "Mochi", HERE)));
        assert!(!friends.follows(1));
        // Following one: nobody, and its games are the leader's to start.
        assert!(!friends.can_invite(Some(2), &card(4, "Mochi", HERE)));
        assert!(friends.follows(2));
        // Full: nobody.
        let full = (1..=GROUP_MOST as u64).map(|id| card(id, "P", HERE)).collect();
        friends.group = Some(Group { leader: 1, members: full });
        assert!(!friends.can_invite(Some(1), &card(9, "Mochi", HERE)));
    }

    #[test]
    fn where_a_friend_is_reads_as_the_sketch_has_it() {
        assert_eq!(whereabouts(&Presence::Town { channel: 0 }, 0.0), "In Main Lobby #1");
        assert_eq!(whereabouts(&Presence::Home, 0.0), "At home");
        let queue = Presence::Playing {
            game: Minigame::BridgeWars,
            queued: true,
        };
        assert_eq!(whereabouts(&queue, 0.0), "Bridge Wars: In Queue");
        let game = Presence::Playing {
            game: Minigame::HouseBuilder,
            queued: false,
        };
        assert_eq!(whereabouts(&game, 0.0), "Home Builders: In Game");
        // Gone: for how long, in whole minutes, hours and days, one of
        // each said as one; and with no word of when, only that they are.
        let since = 1_700_000_000u64;
        let at = |after: f64| whereabouts(&Presence::Offline { since }, since as f64 + after);
        assert_eq!(at(5.0), "Offline for a moment");
        assert_eq!(at(60.0), "Offline for 1 minute");
        assert_eq!(at(59.0 * 60.0 + 59.0), "Offline for 59 minutes");
        assert_eq!(at(3_600.0), "Offline for 1 hour");
        assert_eq!(at(15.0 * 3_600.0 + 120.0), "Offline for 15 hours");
        assert_eq!(at(86_400.0), "Offline for 1 day");
        assert_eq!(at(3.0 * 86_400.0), "Offline for 3 days");
        assert_eq!(at(-30.0), "Offline for a moment", "a clock a little behind");
        assert_eq!(whereabouts(&Presence::Offline { since: 0 }, 5.0), "Offline");
    }

    #[test]
    fn a_count_past_nine_is_nine_and_more_and_every_outcome_has_its_words() {
        assert_eq!(counted(1), "1");
        assert_eq!(counted(9), "9");
        assert_eq!(counted(10), "9+");
        for said in [
            Outcome::Asked,
            Outcome::Befriended,
            Outcome::Unfriended,
            Outcome::Already,
            Outcome::Full,
            Outcome::TheirsFull,
            Outcome::NoSuch,
            Outcome::NotFriend,
            Outcome::Offline,
            Outcome::Invited,
            Outcome::NotLeader,
            Outcome::GroupFull,
            Outcome::Stale,
            Outcome::Sent,
            Outcome::InboxFull,
            Outcome::NotAMessage,
            Outcome::LeftBehind,
        ] {
            assert!(!said_in_words(said).is_empty(), "{said:?}");
        }
    }

    #[test]
    fn the_panel_fits_a_phone_lying_on_its_side_and_a_tablet() {
        // The short side is 100 shares: a phone's long side is over 200, a
        // 4:3 tablet's 133. The panel, and the widest box over it, fit
        // both with room round them.
        assert!(WIDTH < 133.0 - 2.0 * hud::EDGE);
        assert!(HEIGHT < 100.0 - 2.0 * hud::EDGE);
        assert!(80.0 < WIDTH - 2.0 * PAD, "the box a message is written in");
        // A row holds a picture, and two round buttons beside the words.
        assert!(AVATAR < ROW && ROW_BUTTON < ROW);
    }
}
