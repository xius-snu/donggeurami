//! Friends, groups and messages, on the server (since 2026-11-13; Hajun: "I
//! want to add a friend system. First every account need a random number
//! between 1000~9999 that is gona serve as a #1234 tag for identification
//! when there is duplicate name ... you should be able to search the
//! username, and it will give u a list of everyone with that username and u
//! can choose which one to add ... they can accept or reject the add friend
//! request. and when youre in the group with ur friend, only the leader can
//! queue for a minigame, and everyone get queued at the same time in the
//! same team"). What a device asks and is told is `roundtown_net::social`;
//! this is what the server keeps and does about it.
//!
//! **All of it is held here, in memory** ([`Social`]): what every account is
//! called and its tag, who is whose friend, everyone's inbox and every
//! group. It is read from the database once, as the server starts
//! (`accounts::Snapshot`), and from then on the game's loop goes by its own
//! copy, so that nothing about friends waits on a query; whatever changes
//! that is to outlast the server is written down after the fact, in the
//! order it happened (`accounts::Keep`). Without a database it all lasts as
//! long as the server runs, which is what trying it out on one machine
//! needs. Groups, and being asked into one, are never written down: a group
//! is people who are here now.
//!
//! [`Social`] is plain: it knows nothing of the world, takes what is asked
//! ([`Social::act`]) and where everyone is ([`Social::saw`],
//! [`Social::left`]), and only says what is to be told and to whom, and what
//! is to be kept ([`Social::told`], [`Social::kept`]); three small systems
//! feed it and carry that out ([`hear`], [`track`], [`flush`]). So all of it
//! is tested as it stands, with no server running.
//!
//! * **A tag** ([`Social::enroll`], [`Social::rename`]): no two accounts
//!   have both the same name, whatever its capitals, and the same tag. An
//!   account that takes a name someone with its tag already has is given
//!   another tag, at random, that nobody of that name has (mine: Hajun asked
//!   only that the two never both be the same; the other way is to refuse
//!   the name, which would make a name something that can be taken).
//! * **Where everyone is** ([`track`]): looked at four times a second, each
//!   account that is connected and the room its body is in; whoever has
//!   come is told everything of theirs ([`Social::greet`]), and whoever's
//!   place has changed has it told to their friends who are here, and to
//!   their group.
//! * **A group** has a leader, whoever first asked someone into it, and up
//!   to [`GROUP_MOST`] in it. Being asked stands for [`INVITE_SECS`], and
//!   only while whoever asked is here. Whoever's connection goes is kept in
//!   their group for [`GROUP_GRACE`]; the leader leaving, the first of the
//!   rest who is here leads; one alone is no group. Going into a game is
//!   the games' own (`arena`, `builder`), which ask who goes with whom
//!   ([`Social::party`]).
//! * **A level** is what an account has earned playing ([`Social::award`]).

use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::social::*;
use roundtown_net::{Ask, Tell};

use crate::accounts::{Bank, Keep, Letter, Snapshot, free_tag};
use crate::arena::Arenas;
use crate::builder::Games;
use crate::players::{Body, Everyone, Plays, Who, tell};
use crate::rooms::Room;

/// How often where everyone is is looked at, in seconds.
const TRACK_EVERY: f32 = 0.25;

pub(crate) fn plugin(app: &mut App, snapshot: Snapshot, deleted: Option<Receiver<u64>>) {
    app.insert_resource(Social::new(snapshot))
        .insert_resource(Deleted(deleted.map(Mutex::new)))
        .add_systems(
            PreUpdate,
            hear.after(ServerSystems::Receive).run_if(in_state(ServerState::Running)),
        )
        .add_systems(
            Update,
            (forget_deleted, track, flush).chain().run_if(in_state(ServerState::Running)),
        );
}

/// One account, as far as its friends are concerned.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Person {
    pub name: String,
    pub tag: u16,
    /// What it has earned toward its level.
    pub xp: u32,
    /// When it was last here, in seconds since 1970; 0 if never, as far as
    /// is known.
    pub seen: u64,
}

/// Something waiting in an inbox.
#[derive(Clone, Debug, PartialEq)]
struct Waiting {
    id: u64,
    from: u64,
    what: Post,
    /// When it came, by the server's own seconds: what being asked into a
    /// group goes stale by.
    at: f64,
}

/// A group: who leads it, and everyone in it, in the order they came.
#[derive(Clone, Debug, PartialEq)]
struct Band {
    leader: u64,
    members: Vec<u64>,
}

/// Everything about friends, groups and messages.
#[derive(Resource)]
pub(crate) struct Social {
    people: HashMap<u64, Person>,
    /// Who goes by each name, in small letters.
    named: HashMap<String, Vec<u64>>,
    friends: HashMap<u64, BTreeSet<u64>>,
    /// Everyone's inbox, oldest first.
    inbox: HashMap<u64, Vec<Waiting>>,
    next_mail: u64,
    groups: HashMap<u32, Band>,
    /// The group each account in one is in.
    grouped: HashMap<u64, u32>,
    next_group: u32,
    /// Where everyone who is connected is, as their friends were last told.
    here: HashMap<u64, Presence>,
    /// Since when each member of a group whose connection has gone has
    /// been gone, by the server's own seconds.
    gone: HashMap<u64, f64>,
    /// The games already paid out for, each once.
    awarded: HashSet<(Minigame, u32)>,
    /// What is to be told, and to whom, until it has been ([`flush`]).
    told: Vec<(u64, News)>,
    /// What is to be written down, until it has been.
    kept: Vec<Keep>,
    dice: fastrand::Rng,
}

impl Social {
    /// As the database has it, or with nothing in it.
    pub(crate) fn new(snapshot: Snapshot) -> Self {
        Self::with(snapshot, fastrand::Rng::new())
    }

    fn with(snapshot: Snapshot, dice: fastrand::Rng) -> Self {
        let mut social = Self {
            people: HashMap::new(),
            named: HashMap::new(),
            friends: HashMap::new(),
            inbox: HashMap::new(),
            next_mail: 1,
            groups: HashMap::new(),
            grouped: HashMap::new(),
            next_group: 1,
            here: HashMap::new(),
            gone: HashMap::new(),
            awarded: HashSet::new(),
            told: Vec::new(),
            kept: Vec::new(),
            dice,
        };
        for person in snapshot.people {
            social.enroll(person.id, &person.name, Some(person.tag), person.xp);
            if let Some(known) = social.people.get_mut(&person.id) {
                known.seen = person.seen;
            }
        }
        for (a, b) in snapshot.friends {
            if a != b && social.people.contains_key(&a) && social.people.contains_key(&b) {
                social.friends.entry(a).or_default().insert(b);
                social.friends.entry(b).or_default().insert(a);
            }
        }
        // A message keeps the number it was written down under; whatever
        // comes after is numbered on from the last of them.
        social.next_mail = snapshot.letters.iter().map(|letter| letter.id + 1).max().unwrap_or(1);
        for (from, to) in snapshot.requests {
            if social.people.contains_key(&from) && social.people.contains_key(&to) {
                social.post(to, from, Post::Request, 0.0);
            }
        }
        for letter in snapshot.letters {
            if social.people.contains_key(&letter.from) && social.people.contains_key(&letter.to) {
                social.inbox.entry(letter.to).or_default().push(Waiting {
                    id: letter.id,
                    from: letter.from,
                    what: Post::Message(letter.text),
                    at: 0.0,
                });
            }
        }
        social
    }

    // ------------------------------------------------------ who is who

    /// What is known of the account `id`.
    pub(crate) fn person(&self, id: u64) -> Option<&Person> {
        self.people.get(&id)
    }

    /// Whether an account other than `but` goes by `name` with `tag`.
    fn taken(&self, name: &str, tag: u16, but: u64) -> bool {
        self.named.get(&name.to_lowercase()).is_some_and(|ids| {
            ids.iter()
                .any(|&id| id != but && self.people.get(&id).is_some_and(|person| person.tag == tag))
        })
    }

    /// A tag nobody called `name` has, `but` aside, at random; none, with
    /// all nine thousand taken.
    fn a_free_tag(&mut self, name: &str, but: u64) -> Option<u16> {
        let taken: HashSet<u16> = self
            .named
            .get(&name.to_lowercase())
            .into_iter()
            .flatten()
            .filter(|&&id| id != but)
            .filter_map(|id| self.people.get(id))
            .map(|person| person.tag)
            .collect();
        if taken.len() > usize::from(TAG_LAST - TAG_FIRST) {
            return None;
        }
        let dice = &mut self.dice;
        Some(free_tag(&taken, &mut || dice.u16(TAG_FIRST..=TAG_LAST)))
    }

    /// Takes `id` out of the list of who goes by `name`.
    fn unname(&mut self, id: u64, name: &str) {
        let key = name.to_lowercase();
        if let Some(ids) = self.named.get_mut(&key) {
            ids.retain(|&other| other != id);
            if ids.is_empty() {
                self.named.remove(&key);
            }
        }
    }

    /// The account `id` is called `name`, with `tag` after it, and has
    /// earned `xp`, as its account has it: as it connects, or as the
    /// server starts. With no tag, or one that is not one, or that
    /// someone of its name has, it is given one, and that is to be kept.
    /// The tag it has.
    pub(crate) fn enroll(&mut self, id: u64, name: &str, tag: Option<u16>, xp: u32) -> u16 {
        let was = self.people.get(&id).cloned();
        let good = tag.filter(|&tag| (TAG_FIRST..=TAG_LAST).contains(&tag) && !self.taken(name, tag, id));
        let tag = good.unwrap_or_else(|| {
            let tag = self.a_free_tag(name, id).unwrap_or(TAG_FIRST);
            self.kept.push(Keep::Tag { id, tag });
            tag
        });
        if let Some(was) = &was {
            self.unname(id, &was.name);
        }
        self.named.entry(name.to_lowercase()).or_default().push(id);
        self.people.insert(
            id,
            Person {
                name: name.to_string(),
                tag,
                xp,
                seen: was.map_or(0, |was| was.seen),
            },
        );
        tag
    }

    /// The account `id` has taken the name `name` (`Ask::Profile`, checked
    /// already): the tag that goes with it, its own unless someone of that
    /// name has it, and then another, which they and their friends are
    /// told. None if it cannot be: no such account, or every tag of that
    /// name taken.
    pub(crate) fn rename(&mut self, id: u64, name: &str) -> Option<u16> {
        let person = self.people.get(&id)?;
        let (was, mut tag) = (person.name.clone(), person.tag);
        if self.taken(name, tag, id) {
            tag = self.a_free_tag(name, id)?;
        }
        self.unname(id, &was);
        self.named.entry(name.to_lowercase()).or_default().push(id);
        let person = self.people.get_mut(&id)?;
        let changed = person.name != name || person.tag != tag;
        person.name = name.to_string();
        person.tag = tag;
        if changed {
            self.tell_me(id);
            self.announce(id);
        }
        Some(tag)
    }

    /// The account `id` is no more: deleted.
    pub(crate) fn forget(&mut self, id: u64) {
        self.leave_group(id);
        let Some(person) = self.people.remove(&id) else {
            return;
        };
        self.unname(id, &person.name);
        self.here.remove(&id);
        self.gone.remove(&id);
        for friend in self.friends.remove(&id).unwrap_or_default() {
            if let Some(theirs) = self.friends.get_mut(&friend) {
                theirs.remove(&id);
            }
            self.tell_friends(friend);
        }
        self.inbox.remove(&id);
        let written_to: Vec<u64> = self
            .inbox
            .iter()
            .filter(|(_, waiting)| waiting.iter().any(|mail| mail.from == id))
            .map(|(&to, _)| to)
            .collect();
        for to in written_to {
            if let Some(waiting) = self.inbox.get_mut(&to) {
                waiting.retain(|mail| mail.from != id);
            }
            self.tell_inbox(to);
        }
    }

    // ---------------------------------------------------- what is told

    /// What is to be told, and to whom, since this was last asked.
    pub(crate) fn told(&mut self) -> Vec<(u64, News)> {
        std::mem::take(&mut self.told)
    }

    /// What is to be written down, since this was last asked.
    pub(crate) fn kept(&mut self) -> Vec<Keep> {
        std::mem::take(&mut self.kept)
    }

    fn tell(&mut self, to: u64, news: News) {
        self.told.push((to, news));
    }

    fn say(&mut self, to: u64, said: Outcome) {
        self.tell(to, News::Said(said));
    }

    /// Where `id` is: as last seen, or not here, since when.
    fn presence(&self, id: u64) -> Presence {
        self.here.get(&id).copied().unwrap_or(Presence::Offline {
            since: self.people.get(&id).map_or(0, |person| person.seen),
        })
    }

    fn card(&self, id: u64) -> Option<Card> {
        let person = self.people.get(&id)?;
        Some(Card {
            id,
            name: person.name.clone(),
            tag: person.tag,
            level: level(person.xp),
            presence: self.presence(id),
        })
    }

    fn tell_me(&mut self, id: u64) {
        if let Some(person) = self.people.get(&id) {
            let me = News::Me {
                tag: person.tag,
                level: level(person.xp),
            };
            self.tell(id, me);
        }
    }

    fn friends_of(&self, id: u64) -> impl Iterator<Item = u64> + '_ {
        self.friends.get(&id).into_iter().flatten().copied()
    }

    fn are_friends(&self, a: u64, b: u64) -> bool {
        self.friends.get(&a).is_some_and(|friends| friends.contains(&b))
    }

    fn tell_friends(&mut self, id: u64) {
        let cards = self.friends_of(id).filter_map(|friend| self.card(friend)).collect();
        self.tell(id, News::Friends(cards));
    }

    fn tell_inbox(&mut self, id: u64) {
        let mail = self
            .inbox
            .get(&id)
            .into_iter()
            .flatten()
            .filter_map(|waiting| {
                Some(Mail {
                    id: waiting.id,
                    from: self.card(waiting.from)?,
                    what: waiting.what.clone(),
                })
            })
            .collect();
        self.tell(id, News::Inbox(mail));
    }

    /// Tells everyone in `group` who is in it.
    fn tell_group(&mut self, group: u32) {
        let Some(band) = self.groups.get(&group) else {
            return;
        };
        let shown = Group {
            leader: band.leader,
            members: band.members.iter().filter_map(|&member| self.card(member)).collect(),
        };
        for member in band.members.clone() {
            self.tell(member, News::Group(Some(shown.clone())));
        }
    }

    /// `id` is not as their friends and their group were last told: by
    /// another name, at another level, or somewhere else. Their friends
    /// who are here are told, and their group.
    fn announce(&mut self, id: u64) {
        let Some(card) = self.card(id) else {
            return;
        };
        let friends: Vec<u64> = self.friends_of(id).filter(|friend| self.here.contains_key(friend)).collect();
        for friend in friends {
            self.tell(friend, News::Friend(card.clone()));
        }
        if let Some(&group) = self.grouped.get(&id) {
            self.tell_group(group);
        }
    }

    /// Tells `id` everything of theirs: as they come.
    fn greet(&mut self, id: u64) {
        self.tell_me(id);
        self.tell_friends(id);
        self.tell_inbox(id);
        match self.grouped.get(&id).copied() {
            Some(group) => self.tell_group(group),
            None => self.tell(id, News::Group(None)),
        }
    }

    // ------------------------------------------------- where everyone is

    /// `id` is connected, and `presence` is where: whoever has just come
    /// is told everything of theirs, and whoever is somewhere else than
    /// their friends were last told has them told.
    pub(crate) fn saw(&mut self, id: u64, presence: Presence) {
        if !self.people.contains_key(&id) {
            return;
        }
        match self.here.insert(id, presence) {
            Some(was) if was == presence => {}
            Some(_) => self.announce(id),
            None => {
                self.gone.remove(&id);
                self.greet(id);
                self.announce(id);
            }
        }
    }

    /// A device has just been let in as `id`, who was here already as far
    /// as the server knew: a phone back before the connection it left had
    /// been given up on, the app put away and opened again. That device
    /// knows nothing, whatever the one before it was told, so the next look
    /// at who is here ([`Social::saw`]) takes them as newly come and tells
    /// them everything. Nothing else of theirs changes: they never left.
    pub(crate) fn connected(&mut self, id: u64) {
        self.here.remove(&id);
    }

    /// `id`'s connection has gone, at `now` by the server's own seconds and
    /// `unix` since 1970: their friends are told, whoever they asked into a
    /// group is no longer asked, and their group keeps them a while.
    pub(crate) fn left(&mut self, id: u64, now: f64, unix: u64) {
        if self.here.remove(&id).is_none() {
            return;
        }
        if let Some(person) = self.people.get_mut(&id) {
            person.seen = unix;
            self.kept.push(Keep::Seen(id));
        }
        self.unpost(|_, mail| mail.from == id && mail.what == Post::Invite);
        if self.grouped.contains_key(&id) {
            self.gone.insert(id, now);
        }
        self.announce(id);
    }

    /// Who is connected, as last seen.
    pub(crate) fn here(&self) -> Vec<u64> {
        self.here.keys().copied().collect()
    }

    /// Whatever has stood too long, at `now`: being asked into a group, and
    /// a place in one kept for someone whose connection has gone.
    pub(crate) fn tick(&mut self, now: f64) {
        self.unpost(|_, mail| mail.what == Post::Invite && now - mail.at > INVITE_SECS);
        let overdue: Vec<u64> = self
            .gone
            .iter()
            .filter(|&(_, &since)| now - since > GROUP_GRACE)
            .map(|(&id, _)| id)
            .collect();
        for id in overdue {
            self.leave_group(id);
        }
    }

    // ------------------------------------------------------- what is asked

    /// Does what `id` asks, at `now` by the server's own seconds.
    pub(crate) fn act(&mut self, id: u64, act: Act, now: f64) {
        if !self.people.contains_key(&id) {
            return;
        }
        let said = match act {
            Act::Find(text) => {
                self.find(id, &text);
                None
            }
            Act::Befriend(them) => self.befriend(id, them, now),
            Act::Unfriend(them) => Some(self.unfriend(id, them)),
            Act::Answer { mail, yes } => self.answer(id, mail, yes),
            Act::Invite(them) => Some(self.invite(id, them, now)),
            Act::LeaveGroup => {
                self.leave_group(id);
                None
            }
            Act::Kick(them) => self.kick(id, them),
            Act::Write { to, text } => Some(self.write(id, to, &text, now)),
        };
        if let Some(said) = said {
            self.say(id, said);
        }
    }

    /// Who goes by `text`, as `id` is told: by all of it as a name, and, if
    /// it ends in a tag, by the name before the tag with that tag. Whoever
    /// is here first, then whoever was here last; never `id` themselves.
    fn find(&mut self, id: u64, text: &str) {
        let sought: String = text.trim().chars().take(SOUGHT_LONGEST).collect();
        let mut ids: Vec<u64> = self.named.get(&sought.to_lowercase()).cloned().unwrap_or_default();
        if let Some((name, tag)) = split_handle(&sought) {
            let tagged = self
                .named
                .get(&name.to_lowercase())
                .into_iter()
                .flatten()
                .copied()
                .filter(|other| self.people.get(other).is_some_and(|person| person.tag == tag));
            ids.extend(tagged);
        }
        ids.retain(|&other| other != id);
        ids.sort_unstable();
        ids.dedup();
        ids.sort_by_key(|&other| {
            let seen = self.people.get(&other).map_or(0, |person| person.seen);
            (!self.here.contains_key(&other), Reverse(seen), other)
        });
        ids.truncate(FOUND_MOST);
        let people = ids
            .into_iter()
            .filter_map(|other| {
                Some(Found {
                    card: self.card(other)?,
                    tie: self.tie(id, other),
                })
            })
            .collect();
        self.tell(id, News::Found { sought, people });
    }

    /// The request `from` has waiting with `to`, if any: its number.
    fn request(&self, from: u64, to: u64) -> Option<u64> {
        self.inbox
            .get(&to)?
            .iter()
            .find(|mail| mail.from == from && mail.what == Post::Request)
            .map(|mail| mail.id)
    }

    /// What `them` is to `me` already.
    fn tie(&self, me: u64, them: u64) -> Tie {
        if self.are_friends(me, them) {
            Tie::Friend
        } else if self.request(me, them).is_some() {
            Tie::Asked
        } else if self.request(them, me).is_some() {
            Tie::AskedYou
        } else {
            Tie::Stranger
        }
    }

    /// Puts `what`, from `from`, in `to`'s inbox: its number.
    fn post(&mut self, to: u64, from: u64, what: Post, now: f64) -> u64 {
        let id = self.next_mail;
        self.next_mail += 1;
        self.inbox.entry(to).or_default().push(Waiting { id, from, what, at: now });
        id
    }

    /// Takes out of every inbox whatever `gone(whose, mail)` says has gone,
    /// and tells whoever's inbox it was in.
    fn unpost(&mut self, gone: impl Fn(u64, &Waiting) -> bool) {
        let mut changed = Vec::new();
        for (&to, waiting) in &mut self.inbox {
            let before = waiting.len();
            waiting.retain(|mail| !gone(to, mail));
            if waiting.len() != before {
                changed.push(to);
            }
        }
        self.inbox.retain(|_, waiting| !waiting.is_empty());
        for to in changed {
            self.tell_inbox(to);
        }
    }

    fn waiting(&self, id: u64, kind: impl Fn(&Post) -> bool) -> usize {
        self.inbox
            .get(&id)
            .map_or(0, |waiting| waiting.iter().filter(|mail| kind(&mail.what)).count())
    }

    /// `me` asks `them` to be friends: or, `them` having asked already,
    /// says yes.
    fn befriend(&mut self, me: u64, them: u64, now: f64) -> Option<Outcome> {
        if me == them || !self.people.contains_key(&them) {
            return Some(Outcome::NoSuch);
        }
        match self.tie(me, them) {
            Tie::Friend | Tie::Asked => return Some(Outcome::Already),
            Tie::AskedYou => {
                let mail = self.request(them, me)?;
                return self.answer(me, mail, true);
            }
            Tie::Stranger => {}
        }
        if self.friends_of(me).count() >= FRIENDS_MOST {
            return Some(Outcome::Full);
        }
        let asked = self.waiting(them, |what| *what == Post::Request);
        if self.friends_of(them).count() >= FRIENDS_MOST || asked >= REQUESTS_MOST {
            return Some(Outcome::TheirsFull);
        }
        self.post(them, me, Post::Request, now);
        self.kept.push(Keep::Request { from: me, to: them });
        self.tell_inbox(them);
        Some(Outcome::Asked)
    }

    /// `me` answers what is numbered `mail` in their inbox.
    fn answer(&mut self, me: u64, mail: u64, yes: bool) -> Option<Outcome> {
        let at = self
            .inbox
            .get(&me)
            .and_then(|waiting| waiting.iter().position(|each| each.id == mail));
        let Some(at) = at else {
            return Some(Outcome::Stale);
        };
        let mail = self.inbox.get_mut(&me)?.remove(at);
        let said = match mail.what {
            Post::Request => {
                self.kept.push(Keep::Unrequest { from: mail.from, to: me });
                if !yes {
                    None
                } else if !self.people.contains_key(&mail.from) {
                    Some(Outcome::NoSuch)
                } else if self.friends_of(me).count() >= FRIENDS_MOST {
                    Some(Outcome::Full)
                } else if self.friends_of(mail.from).count() >= FRIENDS_MOST {
                    Some(Outcome::TheirsFull)
                } else {
                    self.friends.entry(me).or_default().insert(mail.from);
                    self.friends.entry(mail.from).or_default().insert(me);
                    self.kept.push(Keep::Friends { a: me, b: mail.from });
                    self.tell_friends(me);
                    self.tell_friends(mail.from);
                    Some(Outcome::Befriended)
                }
            }
            Post::Invite => {
                if yes {
                    self.join(me, mail.from)
                } else {
                    None
                }
            }
            Post::Message(_) => {
                self.kept.push(Keep::Unletter(mail.id));
                None
            }
        };
        self.tell_inbox(me);
        said
    }

    /// `them` is no longer `me`'s friend; or, asked by `me` and not yet
    /// answered, no longer asked.
    fn unfriend(&mut self, me: u64, them: u64) -> Outcome {
        if self.are_friends(me, them) {
            for (one, other) in [(me, them), (them, me)] {
                if let Some(friends) = self.friends.get_mut(&one) {
                    friends.remove(&other);
                }
            }
            self.kept.push(Keep::Unfriend { a: me, b: them });
            self.tell_friends(me);
            self.tell_friends(them);
            Outcome::Unfriended
        } else if self.request(me, them).is_some() {
            self.unpost(|to, mail| to == them && mail.from == me && mail.what == Post::Request);
            self.kept.push(Keep::Unrequest { from: me, to: them });
            Outcome::Unfriended
        } else {
            Outcome::NotFriend
        }
    }

    /// `me` writes `text` to their friend `to`.
    fn write(&mut self, me: u64, to: u64, text: &str, now: f64) -> Outcome {
        if !self.are_friends(me, to) {
            return Outcome::NotFriend;
        }
        let Some(text) = message(text) else {
            return Outcome::NotAMessage;
        };
        if self.waiting(to, |what| matches!(what, Post::Message(_))) >= LETTERS_MOST {
            return Outcome::InboxFull;
        }
        let id = self.post(to, me, Post::Message(text.clone()), now);
        self.kept.push(Keep::Letter(Letter { id, from: me, to, text }));
        self.tell_inbox(to);
        Outcome::Sent
    }

    // -------------------------------------------------------------- groups

    fn band_of(&self, id: u64) -> Option<&Band> {
        self.groups.get(self.grouped.get(&id)?)
    }

    /// `me` asks their friend `them` into their group, which is theirs to
    /// lead if they are in none.
    fn invite(&mut self, me: u64, them: u64, now: f64) -> Outcome {
        if !self.are_friends(me, them) {
            return Outcome::NotFriend;
        }
        if !self.here.contains_key(&them) {
            return Outcome::Offline;
        }
        if let Some(band) = self.band_of(me) {
            if band.leader != me {
                return Outcome::NotLeader;
            }
            if band.members.contains(&them) {
                return Outcome::Already;
            }
            if band.members.len() >= GROUP_MOST {
                return Outcome::GroupFull;
            }
        }
        // Asked already: asked afresh, from now.
        let asked = self
            .inbox
            .get_mut(&them)
            .and_then(|waiting| waiting.iter_mut().find(|mail| mail.from == me && mail.what == Post::Invite));
        match asked {
            Some(asked) => asked.at = now,
            None => {
                self.post(them, me, Post::Invite, now);
                self.tell_inbox(them);
            }
        }
        Outcome::Invited
    }

    /// `me` goes into the group of `asker`, who asked them: a new one of
    /// the two of them, `asker` leading, if `asker` is in none. Out of
    /// whatever group `me` was in first.
    fn join(&mut self, me: u64, asker: u64) -> Option<Outcome> {
        if !self.here.contains_key(&asker) {
            return Some(Outcome::Offline);
        }
        match self.grouped.get(&asker).copied() {
            Some(group) => {
                let band = self.groups.get(&group)?;
                if band.leader != asker {
                    return Some(Outcome::Stale);
                }
                if band.members.contains(&me) {
                    return Some(Outcome::Already);
                }
                if band.members.len() >= GROUP_MOST {
                    return Some(Outcome::GroupFull);
                }
                self.leave_group(me);
                self.groups.get_mut(&group)?.members.push(me);
                self.grouped.insert(me, group);
                self.tell_group(group);
            }
            None => {
                self.leave_group(me);
                let group = self.next_group;
                self.next_group += 1;
                self.groups.insert(
                    group,
                    Band {
                        leader: asker,
                        members: vec![asker, me],
                    },
                );
                self.grouped.insert(asker, group);
                self.grouped.insert(me, group);
                self.tell_group(group);
            }
        }
        None
    }

    /// `id` is out of whatever group they were in. The leader gone, the
    /// first of the rest who is here leads; one alone is no group.
    fn leave_group(&mut self, id: u64) {
        let Some(group) = self.grouped.remove(&id) else {
            return;
        };
        self.gone.remove(&id);
        self.tell(id, News::Group(None));
        let Some(band) = self.groups.get_mut(&group) else {
            return;
        };
        band.members.retain(|&member| member != id);
        if band.members.len() < 2 {
            let last = self.groups.remove(&group).map(|band| band.members).unwrap_or_default();
            for id in last {
                self.grouped.remove(&id);
                self.gone.remove(&id);
                self.tell(id, News::Group(None));
            }
            return;
        }
        if band.leader == id {
            let here = &self.here;
            band.leader = band
                .members
                .iter()
                .copied()
                .find(|member| here.contains_key(member))
                .unwrap_or(band.members[0]);
        }
        self.tell_group(group);
    }

    /// `me`, leading their group, puts `them` out of it.
    fn kick(&mut self, me: u64, them: u64) -> Option<Outcome> {
        let band = self.band_of(me)?;
        if band.leader != me {
            return Some(Outcome::NotLeader);
        }
        if them == me || !band.members.contains(&them) {
            return Some(Outcome::NoSuch);
        }
        self.leave_group(them);
        None
    }

    /// Who goes into a game with `id` when they ask for one, themselves
    /// first: nobody else, if they are in no group; everyone of their
    /// group, if they lead it; and nobody at all, not even them, if
    /// someone else leads it (`Refusal::NotLeader`).
    pub(crate) fn party(&self, id: u64) -> Option<Vec<u64>> {
        match self.band_of(id) {
            None => Some(vec![id]),
            Some(band) if band.leader == id => {
                let mut party = vec![id];
                party.extend(band.members.iter().copied().filter(|&member| member != id));
                Some(party)
            }
            Some(_) => None,
        }
    }

    /// `id`'s group has gone into a game without them: they are told so.
    pub(crate) fn left_behind(&mut self, id: u64) {
        self.say(id, Outcome::LeftBehind);
    }

    // -------------------------------------------------------------- levels

    /// Whether the game numbered `id` is paid out for now for the first
    /// time: each is once.
    pub(crate) fn first_time(&mut self, game: Minigame, id: u32) -> bool {
        self.awarded.insert((game, id))
    }

    /// `id` has earned `xp` more toward their level: kept, and at a new
    /// level, told to them, their friends and their group.
    pub(crate) fn award(&mut self, id: u64, xp: u32) {
        let Some(person) = self.people.get_mut(&id) else {
            return;
        };
        if xp == 0 {
            return;
        }
        let was = level(person.xp);
        person.xp = person.xp.saturating_add(xp);
        let now = level(person.xp);
        self.kept.push(Keep::Xp { id, amount: xp });
        if now != was {
            self.tell_me(id);
            self.announce(id);
        }
    }
}

/// Accounts the login has deleted since it was last asked, if there is a
/// login.
#[derive(Resource)]
struct Deleted(Option<Mutex<Receiver<u64>>>);

/// Seconds since 1970, by this machine's clock.
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

/// What devices ask about their friends.
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<&Plays>,
    bodies: Query<&Body>,
    time: Res<Time>,
    mut social: ResMut<Social>,
) {
    for ask in asks.read() {
        let Ask::Social(act) = &ask.message else {
            continue;
        };
        let Some(client) = ask.client_id.entity() else {
            continue;
        };
        let Ok(&Plays(body)) = clients.get(client) else {
            continue;
        };
        let Ok(body) = bodies.get(body) else {
            continue;
        };
        social.act(body.who.0, act.clone(), time.elapsed_secs_f64());
    }
}

/// Where the body in `room` is, as its friends are told.
fn presence_in(room: Room, games: &Games, arenas: &Arenas) -> Presence {
    match room {
        Room::Town(channel) => Presence::Town { channel },
        Room::Home(_) => Presence::Home,
        Room::Game(_) => Presence::Playing {
            game: Minigame::HouseBuilder,
            queued: games.queued(room),
        },
        Room::Arena(_) => Presence::Playing {
            game: Minigame::BridgeWars,
            queued: arenas.queued(room),
        },
    }
}

/// Looks at where everyone connected is, [`TRACK_EVERY`] seconds: whoever
/// has come, gone, or gone somewhere else is what [`Social`] then tells
/// their friends; and lets go of whatever has stood too long.
fn track(
    time: Res<Time>,
    mut since: Local<f32>,
    everyone: Res<Everyone>,
    bodies: Query<&Body>,
    games: Res<Games>,
    arenas: Res<Arenas>,
    mut social: ResMut<Social>,
) {
    *since += time.delta_secs();
    if *since < TRACK_EVERY {
        return;
    }
    *since = 0.0;
    let now = time.elapsed_secs_f64();
    let mut here = HashSet::new();
    for (who, &entity) in &everyone.0 {
        // A body kept for a device that has gone, in the middle of a game
        // of House Builder, is nobody who is here.
        let Ok(body) = bodies.get(entity) else {
            continue;
        };
        if body.client.is_none() {
            continue;
        }
        here.insert(who.0);
        social.saw(who.0, presence_in(body.room, &games, &arenas));
    }
    let unix = unix_now();
    for id in social.here() {
        if !here.contains(&id) {
            social.left(id, now, unix);
        }
    }
    social.tick(now);
}

/// The accounts the login has deleted are nobody's friends any more.
fn forget_deleted(deleted: Res<Deleted>, mut social: ResMut<Social>) {
    let Some(deleted) = &deleted.0 else {
        return;
    };
    let Ok(deleted) = deleted.lock() else {
        return;
    };
    while let Ok(id) = deleted.try_recv() {
        info!("account {id} was deleted: nobody's friend any more");
        social.forget(id);
    }
}

/// Tells every device what [`Social`] has for it, and has the database
/// write down what is to be kept.
fn flush(
    mut social: ResMut<Social>,
    everyone: Res<Everyone>,
    bodies: Query<&Body>,
    bank: Res<Bank>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    if social.told.is_empty() && social.kept.is_empty() {
        return;
    }
    for (to, news) in social.told() {
        let device = everyone
            .0
            .get(&Who(to))
            .and_then(|&body| bodies.get(body).ok())
            .and_then(|body| body.client);
        // Nobody connected as them: they are told everything when they come.
        if let Some(device) = device {
            tell(&mut tells, device, Tell::Social(news));
        }
    }
    for keep in social.kept() {
        bank.keep(keep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::Kept;

    const TOWN: Presence = Presence::Town { channel: 0 };

    /// A server with these accounts, numbered from 1 in this order, all of
    /// them here in the town and told everything; nothing waiting to be
    /// told or kept.
    fn with(names: &[&str]) -> Social {
        let mut social = Social::with(Snapshot::default(), fastrand::Rng::with_seed(7));
        for (n, name) in names.iter().enumerate() {
            let id = n as u64 + 1;
            social.enroll(id, name, None, 0);
            social.saw(id, TOWN);
        }
        social.told();
        social.kept();
        social
    }

    /// What `id` has been told since it was last asked, leaving the rest.
    fn told(social: &mut Social, id: u64) -> Vec<News> {
        let (theirs, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut social.told).into_iter().partition(|(to, _)| *to == id);
        social.told = rest;
        theirs.into_iter().map(|(_, news)| news).collect()
    }

    fn said(news: &[News]) -> Vec<Outcome> {
        news.iter()
            .filter_map(|news| match news {
                News::Said(said) => Some(*said),
                _ => None,
            })
            .collect()
    }

    /// The inbox `id` was last told, if they were told one.
    fn inbox(news: &[News]) -> Option<Vec<Mail>> {
        news.iter().rev().find_map(|news| match news {
            News::Inbox(mail) => Some(mail.clone()),
            _ => None,
        })
    }

    fn friends(news: &[News]) -> Option<Vec<u64>> {
        news.iter().rev().find_map(|news| match news {
            News::Friends(cards) => Some(cards.iter().map(|card| card.id).collect()),
            _ => None,
        })
    }

    fn group(news: &[News]) -> Option<Option<(u64, Vec<u64>)>> {
        news.iter().rev().find_map(|news| match news {
            News::Group(group) => Some(
                group
                    .as_ref()
                    .map(|group| (group.leader, group.members.iter().map(|card| card.id).collect())),
            ),
            _ => None,
        })
    }

    /// Makes `a` and `b` friends, with nothing left to be told or kept.
    fn befriended(social: &mut Social, a: u64, b: u64) {
        social.act(a, Act::Befriend(b), 0.0);
        let mail = social.request(a, b).expect("asked");
        social.act(b, Act::Answer { mail, yes: true }, 0.0);
        assert!(social.are_friends(a, b) && social.are_friends(b, a));
        social.told();
        social.kept();
    }

    /// Asks `them` into `leader`'s group and has them say yes.
    fn grouped(social: &mut Social, leader: u64, them: u64) {
        social.act(leader, Act::Invite(them), 0.0);
        let mail = social
            .inbox
            .get(&them)
            .and_then(|waiting| waiting.iter().find(|mail| mail.from == leader && mail.what == Post::Invite))
            .map(|mail| mail.id)
            .expect("asked into the group");
        social.act(them, Act::Answer { mail, yes: true }, 0.0);
        social.told();
    }

    #[test]
    fn no_two_accounts_have_both_one_name_and_one_tag() {
        let mut social = with(&[]);
        // A thousand of one name, whatever its capitals: a thousand tags.
        let mut tags = HashSet::new();
        for id in 1..=1000u64 {
            let name = if id % 2 == 0 { "Mochi" } else { "mochi" };
            let tag = social.enroll(id, name, None, 0);
            assert!((TAG_FIRST..=TAG_LAST).contains(&tag));
            assert!(tags.insert(tag), "{tag} twice");
        }
        // Each given here is to be kept.
        assert_eq!(social.kept().len(), 1000);
        // As the account has it, kept as it is; one that someone of the
        // name has, or that is no tag, not.
        assert_eq!(social.enroll(2000, "Dubu", Some(4321), 0), 4321);
        assert!(social.kept().is_empty());
        let other = social.enroll(2001, "DUBU", Some(4321), 0);
        assert_ne!(other, 4321);
        assert_eq!(social.kept(), vec![Keep::Tag { id: 2001, tag: other }]);
        let given = social.enroll(2002, "Bori", Some(0), 0);
        assert!((TAG_FIRST..=TAG_LAST).contains(&given));
        // The same name and tag again for the account that has them is no
        // clash with itself.
        assert_eq!(social.enroll(2000, "Dubu", Some(4321), 0), 4321);
    }

    #[test]
    fn taking_a_name_someone_with_your_tag_has_gives_you_another_tag() {
        let mut social = with(&[]);
        social.enroll(1, "Hajooon", Some(1323), 0);
        social.enroll(2, "Bobby", Some(1323), 0);
        social.enroll(3, "Bobby", Some(5000), 0);
        for id in 1..=3 {
            social.saw(id, TOWN);
        }
        befriended(&mut social, 2, 3);
        // Another name nobody with the tag has: the tag stays, and they and
        // their friend are told who they are now.
        assert_eq!(social.rename(2, "Bob"), Some(1323));
        assert_eq!(told(&mut social, 2), vec![News::Me { tag: 1323, level: 1 }]);
        let news = told(&mut social, 3);
        assert!(matches!(&news[..], [News::Friend(card)] if card.name == "Bob" && card.tag == 1323), "{news:?}");
        // The name of someone with the same tag, in other capitals even:
        // another tag, which nobody of that name has.
        let tag = social.rename(2, "hajooon").expect("a tag");
        assert_ne!(tag, 1323);
        assert_eq!(social.person(2).map(|person| (person.name.as_str(), person.tag)), Some(("hajooon", tag)));
        assert_eq!(told(&mut social, 2), vec![News::Me { tag, level: 1 }]);
        // The first keeps theirs, and is found by it; both by the name.
        assert_eq!(social.person(1).map(|person| person.tag), Some(1323));
        social.act(3, Act::Find("HAJOOON".into()), 0.0);
        let found = told(&mut social, 3);
        let News::Found { people, .. } = found.last().expect("an answer") else {
            panic!("{found:?}");
        };
        assert_eq!(people.len(), 2);
        // The same name again changes nothing and tells nobody.
        assert_eq!(social.rename(2, "hajooon"), Some(tag));
        assert!(social.told().is_empty());
        // Nobody is left under the old name.
        social.act(3, Act::Find("Bob".into()), 0.0);
        assert!(matches!(social.told().last(), Some((3, News::Found { people, .. })) if people.is_empty()));
    }

    #[test]
    fn a_search_finds_everyone_of_a_name_and_one_by_their_tag() {
        let mut social = with(&["Hajooon", "Dongi", "dongi", "DONGI", "Dongi#5"]);
        let tag = |social: &Social, id: u64| social.person(id).unwrap().tag;
        // Offline, and seen at different times.
        social.left(3, 0.0, 1_000);
        social.left(4, 0.0, 2_000);
        social.told();
        social.act(1, Act::Find(" dongi ".into()), 0.0);
        let news = told(&mut social, 1);
        let [News::Found { sought, people }] = &news[..] else {
            panic!("{news:?}");
        };
        assert_eq!(sought, "dongi");
        // Whoever is here first, then whoever was here last.
        assert_eq!(people.iter().map(|found| found.card.id).collect::<Vec<_>>(), vec![2, 4, 3]);
        assert!(people.iter().all(|found| found.tie == Tie::Stranger));
        assert_eq!(people[1].card.presence, Presence::Offline { since: 2_000 });
        // By name and tag: the one.
        social.act(1, Act::Find(handle("Dongi", tag(&social, 3))), 0.0);
        let news = told(&mut social, 1);
        assert!(matches!(&news[..], [News::Found { people, .. }] if people.len() == 1 && people[0].card.id == 3), "{news:?}");
        // A name with a # in it is still found whole.
        social.act(1, Act::Find("Dongi#5".into()), 0.0);
        let news = told(&mut social, 1);
        assert!(matches!(&news[..], [News::Found { people, .. }] if people.len() == 1 && people[0].card.id == 5), "{news:?}");
        // Never yourself; and nobody is nobody.
        social.act(2, Act::Find("Dongi".into()), 0.0);
        let news = told(&mut social, 2);
        assert!(matches!(&news[..], [News::Found { people, .. }] if people.iter().all(|found| found.card.id != 2) && people.len() == 2));
        social.act(1, Act::Find("Nobody".into()), 0.0);
        assert!(matches!(&told(&mut social, 1)[..], [News::Found { people, .. }] if people.is_empty()));
    }

    #[test]
    fn asking_and_being_answered_makes_friends_or_not() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi"]);
        // Asked: it is in their inbox, and is to be kept.
        social.act(1, Act::Befriend(2), 0.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Asked]);
        let mail = inbox(&told(&mut social, 2)).expect("their inbox");
        assert_eq!(mail.len(), 1);
        assert_eq!((mail[0].from.id, &mail[0].what), (1, &Post::Request));
        assert_eq!(mail[0].from.handle(), handle("Hajooon", social.person(1).unwrap().tag));
        assert_eq!(social.kept(), vec![Keep::Request { from: 1, to: 2 }]);
        // Asked again: nothing more.
        social.act(1, Act::Befriend(2), 0.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Already]);
        assert!(told(&mut social, 2).is_empty());
        // Found now, each is what they are to the other.
        assert_eq!((social.tie(1, 2), social.tie(2, 1), social.tie(1, 3)), (Tie::Asked, Tie::AskedYou, Tie::Stranger));
        // No: gone from the inbox, and nobody's friend.
        social.act(2, Act::Answer { mail: mail[0].id, yes: false }, 0.0);
        let news = told(&mut social, 2);
        assert_eq!(inbox(&news), Some(vec![]));
        assert!(said(&news).is_empty());
        assert!(!social.are_friends(1, 2));
        assert_eq!(social.kept(), vec![Keep::Unrequest { from: 1, to: 2 }]);
        assert!(told(&mut social, 1).is_empty(), "nobody is told they were turned down");
        // Asked again, and yes: both are told their friends.
        social.act(1, Act::Befriend(2), 0.0);
        let mail = inbox(&told(&mut social, 2)).expect("their inbox")[0].id;
        social.told();
        social.kept();
        social.act(2, Act::Answer { mail, yes: true }, 0.0);
        let news = told(&mut social, 2);
        assert_eq!(said(&news), vec![Outcome::Befriended]);
        assert_eq!(friends(&news), Some(vec![1]));
        assert_eq!(inbox(&news), Some(vec![]));
        assert_eq!(friends(&told(&mut social, 1)), Some(vec![2]));
        assert_eq!(social.kept(), vec![Keep::Unrequest { from: 1, to: 2 }, Keep::Friends { a: 2, b: 1 }]);
        // An answer to what is no longer there.
        social.act(2, Act::Answer { mail, yes: true }, 0.0);
        assert_eq!(said(&told(&mut social, 2)), vec![Outcome::Stale]);
        // Each asking the other: the second asking is the yes.
        social.act(3, Act::Befriend(1), 0.0);
        social.told();
        social.act(1, Act::Befriend(3), 0.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Befriended]);
        assert!(social.are_friends(1, 3) && social.are_friends(3, 1));
        assert!(social.inbox.get(&1).is_none_or(|waiting| waiting.is_empty()));
        // Yourself, and nobody.
        social.act(1, Act::Befriend(1), 0.0);
        social.act(1, Act::Befriend(99), 0.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::NoSuch, Outcome::NoSuch]);
    }

    #[test]
    fn a_friend_is_removed_and_a_request_taken_back() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi"]);
        befriended(&mut social, 1, 2);
        social.act(1, Act::Unfriend(2), 0.0);
        let news = told(&mut social, 1);
        assert_eq!(said(&news), vec![Outcome::Unfriended]);
        assert_eq!(friends(&news), Some(vec![]));
        assert_eq!(friends(&told(&mut social, 2)), Some(vec![]));
        assert_eq!(social.kept(), vec![Keep::Unfriend { a: 1, b: 2 }]);
        assert!(!social.are_friends(2, 1));
        // A request taken back is gone from their inbox.
        social.act(1, Act::Befriend(3), 0.0);
        social.told();
        social.kept();
        social.act(1, Act::Unfriend(3), 0.0);
        assert_eq!(inbox(&told(&mut social, 3)), Some(vec![]));
        assert_eq!(social.kept(), vec![Keep::Unrequest { from: 1, to: 3 }]);
        assert_eq!(social.tie(1, 3), Tie::Stranger);
        // Neither a friend nor asked.
        social.act(1, Act::Unfriend(3), 0.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Unfriended, Outcome::NotFriend]);
    }

    #[test]
    fn a_list_and_an_inbox_hold_only_so_many() {
        let names: Vec<String> = (0..FRIENDS_MOST + 3).map(|n| format!("P{n}")).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut social = with(&names);
        let last = names.len() as u64;
        for other in 2..=FRIENDS_MOST as u64 + 1 {
            befriended(&mut social, 1, other);
        }
        assert_eq!(social.friends_of(1).count(), FRIENDS_MOST);
        // Full, asking; and theirs full, being asked.
        social.act(1, Act::Befriend(last), 0.0);
        social.act(last, Act::Befriend(1), 0.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Full]);
        assert_eq!(said(&told(&mut social, last)), vec![Outcome::TheirsFull]);
        // Asked while there was room, and answered once there is none.
        social.act(1, Act::Unfriend(2), 0.0);
        social.act(last, Act::Befriend(1), 0.0);
        befriended(&mut social, 1, 2);
        let mail = social.request(last, 1).expect("still waiting");
        social.told();
        social.act(1, Act::Answer { mail, yes: true }, 0.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Full]);
        assert!(!social.are_friends(1, last));
        // As many waiting on one answer as an inbox takes.
        let mut social = with(&names);
        for other in 2..=REQUESTS_MOST as u64 + 1 {
            social.act(other, Act::Befriend(1), 0.0);
        }
        social.told();
        social.act(last, Act::Befriend(1), 0.0);
        assert_eq!(said(&told(&mut social, last)), vec![Outcome::TheirsFull]);
    }

    #[test]
    fn a_message_goes_to_a_friend_and_waits_until_it_is_seen_off() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi"]);
        befriended(&mut social, 1, 2);
        social.act(1, Act::Write { to: 2, text: "  play at 9?  ".into() }, 5.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Sent]);
        let mail = inbox(&told(&mut social, 2)).expect("their inbox");
        assert_eq!(mail.len(), 1);
        assert_eq!(mail[0].what, Post::Message("play at 9?".into()));
        assert_eq!(social.kept(), vec![Keep::Letter(Letter { id: mail[0].id, from: 1, to: 2, text: "play at 9?".into() })]);
        // Not to someone who is no friend; nor nothing, nor Hangul, which
        // the font has not.
        social.act(1, Act::Write { to: 3, text: "hi".into() }, 5.0);
        social.act(1, Act::Write { to: 2, text: "   ".into() }, 5.0);
        social.act(1, Act::Write { to: 2, text: "안녕".into() }, 5.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::NotFriend, Outcome::NotAMessage, Outcome::NotAMessage]);
        assert!(told(&mut social, 2).is_empty() && told(&mut social, 3).is_empty());
        // Seen off, either way: gone, here and where it was kept.
        social.kept();
        social.act(2, Act::Answer { mail: mail[0].id, yes: false }, 6.0);
        assert_eq!(inbox(&told(&mut social, 2)), Some(vec![]));
        assert_eq!(social.kept(), vec![Keep::Unletter(mail[0].id)]);
        // Only so many wait.
        for _ in 0..LETTERS_MOST {
            social.act(1, Act::Write { to: 2, text: "again".into() }, 7.0);
        }
        social.told();
        social.act(1, Act::Write { to: 2, text: "one more".into() }, 7.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::InboxFull]);
        // To someone who is not here it waits all the same.
        befriended(&mut social, 1, 3);
        social.left(3, 8.0, 1_700_000_000);
        social.told();
        social.act(1, Act::Write { to: 3, text: "see you tomorrow".into() }, 9.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Sent]);
        social.told();
        social.saw(3, TOWN);
        let news = told(&mut social, 3);
        assert_eq!(inbox(&news).map(|mail| mail.len()), Some(1), "told as they come back");
    }

    #[test]
    fn whoever_comes_is_told_everything_and_their_friends_where_they_are() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi"]);
        befriended(&mut social, 1, 2);
        befriended(&mut social, 1, 3);
        // Gone: their friends are told since when, and it is to be kept.
        social.left(2, 10.0, 1_700_000_000);
        let news = told(&mut social, 1);
        assert!(
            matches!(&news[..], [News::Friend(card)] if card.id == 2 && card.presence == Presence::Offline { since: 1_700_000_000 }),
            "{news:?}"
        );
        assert!(told(&mut social, 3).is_empty(), "no friend of theirs");
        assert_eq!(social.kept(), vec![Keep::Seen(2)]);
        // Gone twice is gone once.
        social.left(2, 11.0, 1_700_000_001);
        assert!(social.told().is_empty() && social.kept().is_empty());
        // Somewhere else: told once, and not again while they stay.
        let queue = Presence::Playing {
            game: Minigame::BridgeWars,
            queued: true,
        };
        social.saw(3, queue);
        social.saw(3, queue);
        let news = told(&mut social, 1);
        assert!(matches!(&news[..], [News::Friend(card)] if card.id == 3 && card.presence == queue), "{news:?}");
        // Back: told who they are, their friends, their inbox and that they
        // are in no group; and their friend, that they are here.
        social.act(3, Act::Befriend(2), 12.0);
        social.told();
        social.saw(2, TOWN);
        let news = told(&mut social, 2);
        assert!(matches!(news[0], News::Me { level: 1, .. }), "{news:?}");
        assert_eq!(friends(&news), Some(vec![1]));
        assert_eq!(inbox(&news).map(|mail| mail.iter().map(|mail| mail.from.id).collect::<Vec<_>>()), Some(vec![3]));
        assert_eq!(group(&news), Some(None));
        let news = told(&mut social, 1);
        assert!(matches!(&news[..], [News::Friend(card)] if card.id == 2 && card.presence == TOWN), "{news:?}");
        // Someone nobody has heard of is nobody.
        social.saw(77, TOWN);
        social.act(77, Act::Find("Bobby".into()), 0.0);
        assert!(social.told().is_empty());
    }

    #[test]
    fn a_device_let_in_for_someone_who_never_left_is_told_everything_too() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi"]);
        befriended(&mut social, 1, 2);
        social.act(3, Act::Befriend(1), 1.0);
        grouped(&mut social, 1, 2);
        social.told();
        social.kept();
        // Seen where they were, again and again: nothing to say.
        social.saw(1, TOWN);
        assert!(social.told().is_empty());
        // Another device let in as them, the one before never seen to go:
        // at the next look it is told who they are, their friends, their
        // inbox and their group, as if they had just come.
        social.connected(1);
        social.saw(1, TOWN);
        let news = told(&mut social, 1);
        assert!(matches!(news[0], News::Me { level: 1, .. }), "{news:?}");
        assert_eq!(friends(&news), Some(vec![2]));
        assert_eq!(inbox(&news).map(|mail| mail.iter().map(|mail| mail.from.id).collect::<Vec<_>>()), Some(vec![3]));
        assert_eq!(group(&news), Some(Some((1, vec![1, 2]))));
        // They never left: nothing of theirs is written down as gone, their
        // place in their group is theirs, and what they asked stands.
        assert!(social.kept().is_empty());
        assert!(social.gone.is_empty());
        assert_eq!(social.party(1), Some(vec![1, 2]));
        // And once is enough.
        social.told();
        social.saw(1, TOWN);
        assert!(social.told().is_empty());
        // For someone who was not here it changes nothing.
        social.left(3, 5.0, 1_700_000_000);
        social.told();
        social.connected(3);
        assert!(social.told().is_empty());
    }

    #[test]
    fn a_group_is_whoever_its_leader_asked_and_said_yes() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi", "Mochi", "Dubu", "Bori"]);
        for other in 2..=6 {
            befriended(&mut social, 1, other);
        }
        // Alone, you go into a game alone.
        assert_eq!(social.party(1), Some(vec![1]));
        // Asked: in their inbox, and nothing is kept, a group being who is
        // here now.
        social.act(1, Act::Invite(2), 0.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Invited]);
        let mail = inbox(&told(&mut social, 2)).expect("their inbox");
        assert_eq!((mail[0].from.id, &mail[0].what), (1, &Post::Invite));
        assert!(social.kept().is_empty());
        // Asked again: asked once.
        social.act(1, Act::Invite(2), 1.0);
        assert!(told(&mut social, 2).is_empty());
        assert_eq!(social.waiting(2, |what| *what == Post::Invite), 1);
        // No: nothing comes of it.
        social.act(2, Act::Answer { mail: mail[0].id, yes: false }, 2.0);
        assert_eq!(group(&told(&mut social, 2)), None);
        assert_eq!(social.party(1), Some(vec![1]));
        // Yes: a group of the two, the one who asked leading.
        social.told();
        grouped(&mut social, 1, 2);
        assert_eq!(social.band_of(1), Some(&Band { leader: 1, members: vec![1, 2] }));
        assert_eq!(social.party(1), Some(vec![1, 2]));
        assert_eq!(social.party(2), None, "only the leader starts a game");
        // Only the leader asks more in; whoever is in is not asked again;
        // nor a stranger, nor someone who is not here.
        befriended(&mut social, 2, 3);
        social.act(2, Act::Invite(3), 3.0);
        social.act(1, Act::Invite(2), 3.0);
        social.left(6, 3.0, 1_700_000_000);
        social.act(1, Act::Invite(6), 3.0);
        social.act(3, Act::Invite(4), 3.0);
        assert_eq!(said(&told(&mut social, 2)), vec![Outcome::NotLeader]);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::Already, Outcome::Offline]);
        assert_eq!(said(&told(&mut social, 3)), vec![Outcome::NotFriend]);
        // Up to as many as a group has, each told who is in it as it grows.
        social.told();
        social.act(1, Act::Invite(3), 4.0);
        let mail = inbox(&told(&mut social, 3)).unwrap()[0].id;
        social.act(3, Act::Answer { mail, yes: true }, 4.0);
        for member in [1, 2, 3] {
            assert_eq!(group(&told(&mut social, member)), Some(Some((1, vec![1, 2, 3]))), "{member}");
        }
        grouped(&mut social, 1, 4);
        assert_eq!(social.party(1), Some(vec![1, 2, 3, 4]));
        social.act(1, Act::Invite(5), 5.0);
        assert_eq!(said(&told(&mut social, 1)), vec![Outcome::GroupFull]);
    }

    #[test]
    fn leaving_a_group_passes_it_on_and_one_alone_is_no_group() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi", "Mochi"]);
        for other in 2..=4 {
            befriended(&mut social, 1, other);
            grouped(&mut social, 1, other);
        }
        // A member leaves: out, and the rest are told who is left.
        social.act(3, Act::LeaveGroup, 0.0);
        assert_eq!(group(&told(&mut social, 3)), Some(None));
        assert_eq!(group(&told(&mut social, 1)), Some(Some((1, vec![1, 2, 4]))));
        assert_eq!(social.party(3), Some(vec![3]));
        // The leader leaves: the first of the rest who is here leads.
        social.left(2, 1.0, 1_700_000_000);
        social.told();
        social.act(1, Act::LeaveGroup, 2.0);
        assert_eq!(group(&told(&mut social, 4)), Some(Some((4, vec![2, 4]))));
        assert_eq!(social.party(4), Some(vec![4, 2]));
        // The leader puts someone out; nobody else can; and with one left
        // there is no group.
        social.saw(2, TOWN);
        social.told();
        social.act(2, Act::Kick(4), 3.0);
        assert_eq!(said(&told(&mut social, 2)), vec![Outcome::NotLeader]);
        social.act(4, Act::Kick(2), 3.0);
        assert_eq!(group(&told(&mut social, 2)), Some(None));
        assert_eq!(group(&told(&mut social, 4)), Some(None));
        assert!(social.groups.is_empty() && social.grouped.is_empty());
        // Leaving no group is nothing.
        social.act(4, Act::LeaveGroup, 4.0);
        assert!(social.told().is_empty());
    }

    #[test]
    fn saying_yes_to_another_group_leaves_the_one_you_were_in() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi", "Mochi", "Dubu"]);
        befriended(&mut social, 1, 2);
        befriended(&mut social, 1, 3);
        befriended(&mut social, 4, 3);
        befriended(&mut social, 4, 5);
        grouped(&mut social, 1, 2);
        grouped(&mut social, 1, 3);
        grouped(&mut social, 4, 5);
        // Out of the one, into the other: both are told.
        social.act(4, Act::Invite(3), 0.0);
        let mail = inbox(&told(&mut social, 3)).unwrap()[0].id;
        social.told();
        social.act(3, Act::Answer { mail, yes: true }, 0.0);
        assert_eq!(group(&told(&mut social, 1)), Some(Some((1, vec![1, 2]))));
        assert_eq!(group(&told(&mut social, 4)), Some(Some((4, vec![4, 5, 3]))));
        assert_eq!(group(&told(&mut social, 3)), Some(Some((4, vec![4, 5, 3]))));
        // Asked by someone who has since gone into another's group: stale.
        befriended(&mut social, 2, 5);
        social.act(2, Act::LeaveGroup, 1.0);
        social.act(2, Act::Invite(5), 1.0);
        let mail = social.inbox[&5].iter().find(|mail| mail.what == Post::Invite).unwrap().id;
        befriended(&mut social, 1, 5);
        grouped(&mut social, 1, 2);
        social.told();
        social.act(5, Act::Answer { mail, yes: true }, 2.0);
        assert_eq!(said(&told(&mut social, 5)), vec![Outcome::Stale]);
        assert_eq!(social.band_of(5).map(|band| band.leader), Some(4), "still where they were");
    }

    #[test]
    fn being_asked_into_a_group_stands_only_so_long_and_only_while_the_asker_is_here() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi"]);
        befriended(&mut social, 1, 2);
        befriended(&mut social, 1, 3);
        social.act(1, Act::Invite(2), 100.0);
        social.act(1, Act::Invite(3), 100.0);
        social.told();
        // Still good a little before it goes stale; gone after.
        social.tick(100.0 + INVITE_SECS - 1.0);
        assert!(social.told().is_empty());
        social.tick(100.0 + INVITE_SECS + 1.0);
        assert_eq!(inbox(&told(&mut social, 2)), Some(vec![]));
        assert_eq!(inbox(&told(&mut social, 3)), Some(vec![]));
        // The asker gone: what they asked goes with them, but not what
        // they wrote, nor a request to be friends.
        social.act(1, Act::Invite(2), 1000.0);
        social.act(1, Act::Write { to: 2, text: "join me".into() }, 1000.0);
        social.told();
        social.left(1, 1001.0, 1_700_000_000);
        let news = told(&mut social, 2);
        assert_eq!(inbox(&news).map(|mail| mail.into_iter().map(|mail| mail.what).collect::<Vec<_>>()), Some(vec![Post::Message("join me".into())]));
    }

    #[test]
    fn a_group_keeps_whoever_dropped_for_a_while() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi"]);
        befriended(&mut social, 1, 2);
        befriended(&mut social, 1, 3);
        grouped(&mut social, 1, 2);
        grouped(&mut social, 1, 3);
        // Dropped: still in it, and shown as gone to the rest.
        social.left(2, 50.0, 1_700_000_000);
        let news = told(&mut social, 1);
        let shown = news.iter().rev().find_map(|news| match news {
            News::Group(Some(group)) => Some(group.clone()),
            _ => None,
        });
        let shown = shown.expect("the group, told again");
        assert_eq!(shown.members.iter().map(|card| card.id).collect::<Vec<_>>(), vec![1, 2, 3]);
        assert!(!shown.members[1].presence.online());
        // Back in time: as if never gone, and told their group.
        social.tick(50.0 + GROUP_GRACE - 1.0);
        social.told();
        social.saw(2, TOWN);
        assert_eq!(group(&told(&mut social, 2)), Some(Some((1, vec![1, 2, 3]))));
        social.tick(50.0 + GROUP_GRACE + 100.0);
        assert_eq!(social.party(1), Some(vec![1, 2, 3]));
        // Gone too long: out.
        social.left(3, 200.0, 1_700_000_100);
        social.told();
        social.tick(200.0 + GROUP_GRACE + 1.0);
        assert_eq!(group(&told(&mut social, 1)), Some(Some((1, vec![1, 2]))));
        assert_eq!(social.party(1), Some(vec![1, 2]));
        // The leader gone too long: the next leads.
        social.left(1, 400.0, 1_700_000_200);
        social.tick(400.0 + GROUP_GRACE + 1.0);
        assert!(social.groups.is_empty(), "one alone is no group");
        assert_eq!(social.party(2), Some(vec![2]));
    }

    #[test]
    fn playing_earns_a_level_and_their_friends_are_told() {
        let mut social = with(&["Hajooon", "Bobby"]);
        befriended(&mut social, 1, 2);
        // Not yet a level: kept, and nobody told.
        social.award(1, XP_A_LEVEL - 1);
        assert_eq!(social.kept(), vec![Keep::Xp { id: 1, amount: XP_A_LEVEL - 1 }]);
        assert!(social.told().is_empty());
        // A level: they are told, and their friend.
        social.award(1, 1);
        let tag = social.person(1).unwrap().tag;
        assert_eq!(told(&mut social, 1), vec![News::Me { tag, level: 2 }]);
        let news = told(&mut social, 2);
        assert!(matches!(&news[..], [News::Friend(card)] if card.id == 1 && card.level == 2), "{news:?}");
        // Nothing earned, and nobody known: nothing.
        social.award(1, 0);
        social.award(99, 500);
        assert!(social.told().is_empty());
        // A game pays out once.
        assert!(social.first_time(Minigame::BridgeWars, 3));
        assert!(!social.first_time(Minigame::BridgeWars, 3));
        assert!(social.first_time(Minigame::HouseBuilder, 3));
    }

    #[test]
    fn an_account_deleted_is_nobody_s_friend_any_more() {
        let mut social = with(&["Hajooon", "Bobby", "Dongi"]);
        befriended(&mut social, 1, 2);
        befriended(&mut social, 2, 3);
        grouped(&mut social, 2, 1);
        social.act(2, Act::Write { to: 3, text: "bye".into() }, 0.0);
        social.act(2, Act::Befriend(1), 0.0);
        social.told();
        social.forget(2);
        assert_eq!(friends(&told(&mut social, 1)), Some(vec![]));
        let news = told(&mut social, 3);
        assert_eq!(friends(&news), Some(vec![]));
        assert_eq!(inbox(&news), Some(vec![]));
        assert!(social.person(2).is_none() && social.groups.is_empty());
        social.act(1, Act::Find("Bobby".into()), 0.0);
        assert!(matches!(social.told().last(), Some((1, News::Found { people, .. })) if people.is_empty()));
    }

    #[test]
    fn what_was_kept_is_what_the_server_starts_with() {
        let kept = |id: u64, name: &str, tag: u16| Kept {
            id,
            name: name.into(),
            tag,
            xp: 250,
            seen: 1_700_000_000 + id,
        };
        let snapshot = Snapshot {
            people: vec![kept(1, "Hajooon", 1323), kept(2, "Bobby", 2000), kept(3, "Dongi", 3000), kept(4, "Mochi", 0)],
            friends: vec![(1, 2), (2, 9)],
            requests: vec![(3, 1), (9, 1)],
            letters: vec![
                Letter { id: 40, from: 2, to: 1, text: "play at 9?".into() },
                Letter { id: 41, from: 9, to: 1, text: "from nobody".into() },
            ],
        };
        let mut social = Social::with(snapshot, fastrand::Rng::with_seed(1));
        // An account with no tag is given one, to be kept.
        let given = social.person(4).unwrap().tag;
        assert!((TAG_FIRST..=TAG_LAST).contains(&given));
        assert_eq!(social.kept(), vec![Keep::Tag { id: 4, tag: given }]);
        // Friends both ways; what names an account that is not there is
        // left out.
        assert!(social.are_friends(1, 2) && social.are_friends(2, 1));
        assert!(!social.are_friends(2, 9));
        // Coming, they are told their level, their friends, and an inbox
        // with the request and the message in it, each to be answered by a
        // number of its own.
        social.saw(1, TOWN);
        let news = told(&mut social, 1);
        assert_eq!(news[0], News::Me { tag: 1323, level: 3 });
        assert_eq!(friends(&news), Some(vec![2]));
        let mail = inbox(&news).expect("their inbox");
        assert_eq!(mail.iter().map(|mail| (mail.from.id, mail.what.clone())).collect::<Vec<_>>(), vec![(3, Post::Request), (2, Post::Message("play at 9?".into()))]);
        assert_ne!(mail[0].id, mail[1].id);
        assert_eq!(mail[1].id, 40, "a message keeps the number it was kept under");
        assert_eq!(mail[0].from.presence, Presence::Offline { since: 1_700_000_003 });
        // And what comes next is numbered on past both.
        social.act(1, Act::Befriend(4), 0.0);
        assert!(social.request(1, 4).is_some_and(|id| id > 41));
    }
}
