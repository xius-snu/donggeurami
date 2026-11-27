//! Residents: the computer's people, one to each plot of the town that
//! nobody of your channel has, in its owner's look (Hajun, 2026-11-23: "if
//! < 16 players. for example, 3 players -> the 13 next plots will have an
//! AI NPC spawn in each plot with that plot owner's name, and they should
//! just stay on their plot/house and do random things to make them seem
//! like a real person, and maybe sometime go over to the plot of the real
//! player and go inside and do random things again like sit/sleep and
//! leave and go back to their own plot, etc. random based on random
//! personality/mbti assigned to each bot, kind of like bridge wars bots").
//! **On every plot** since 2026-11-24, where it was the first sixteen
//! ("there should be a character for all 48 plot not just the first 16 ...
//! if theres just 1 real player online in a channel, then there should be
//! 47 npcs, each on their plot"), and **named for what they are**
//! ([`called`]): on a made-up owner's land that owner's name, and on the
//! land of someone real who is not here "{name}'s bot" (`words.land.bot`;
//! "if the npc is named after a real person, it should say like
//! "bunnihihi's bot". but if its a fake house owner's npc then just keep
//! their name like "Sagwa" like now"). They take the place of the town's
//! wanderers round the pond, of which there were enough to make 32 bodies.
//!
//! **Which plots have one is the server's to say** (`land::Deeds`,
//! `Holder::Minded`); offline, every one but yours.
//! **Each device walks its own**, as it always has the town's people
//! (Hajun's choice, 2026-10-08), and nothing of them is sent. But **what
//! each is doing is worked out from the town's clock and from who they
//! are** ([`plan`]), which every device has the same: at a given moment
//! the resident of a plot is on its sofa on everyone's screen, or on its
//! way to the same neighbour's, a step or two apart from one device to the
//! next.
//!
//! **Who one is** ([`Persona`]): four leanings, each one of two ways by
//! who they are and then by how much within that way, as the four letters
//! of a type have it: out among people or at home (how often a visit, how
//! much of the day out on the lawn), on their feet or settled, a napper or
//! a sitter, and steady or changeable (how long they keep at one thing,
//! how fast they walk). Nothing is a fixed rule: each is a share the dice
//! are thrown against, time after time.
//!
//! **The day** goes in blocks of [`BLOCK`] seconds, each resident's begun
//! at its own moment, and sessions of [`SESSION`] blocks. A session is a
//! visit or a session at home, by the dice. At home each block is one
//! thing: out on the lawn, at the front of the plot looking out, about
//! the house, sat on a chair or a sofa, or lying on a bed. A visit is to
//! the plot of someone real in your channel, you among them: there for
//! most of the session, about their house and on their chairs and beds as
//! at home, and back for the end of it. With more residents to each real
//! person than [`CALLERS`], each visits that much less often, so that
//! someone alone in a channel with 47 of them has about as many at theirs
//! as with fifteen: one or so at a time.
//!
//! **Getting there** ([`Search`]): over the town's own shape, what stands on
//! its plots with it, half a metre to a square, as a body could walk it:
//! through a door, round a table; looked for a little each frame, and what
//! is found of each square kept ([`Walkable`]). Between plots, out by the
//! plot's front, along the pavement round the ring road ([`RING`]) and in
//! by the other's front. One that gets nowhere for a while looks for
//! another way, and failing that is put where it was going. Sitting and
//! lying are `rest`'s, as yours are, and a place someone is on is left to
//! them.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::f32::consts::TAU;

use bevy::prelude::*;
use roundtown_net::Pose;
use roundtown_net::land::{Deed, Holder};

use crate::build::stand::yard_of;
use crate::build::{Bounds, Ghost, Mount, Piece};
use crate::clock::TownClock;
use crate::island::{Island, Islands, LATTICE, Venue};
use crate::land::{Deeds, Plot, Plots};
use crate::lobby::{self, Member, Seat};
use crate::rest::{self, Resting};
use crate::talk::{Listening, Speaker};
use crate::ui::{UiChanged, fill, ui};
use crate::{Intent, LAND_STEP_UP, PLAYER_HEIGHT, PLAYER_RADIUS, Player};

/// How long one thing is kept at, in seconds, and how many of those make a
/// session, which is a visit or not.
const BLOCK: f64 = 30.0;
const SESSION: i64 = 8;
/// How many blocks of a visit are spent at the other's plot, the walk
/// there among them: the rest is the way home, and home.
const STAY: i64 = 6;
/// How many residents to each real person of the channel visit as often
/// as each leans to ([`Persona::visits`]): with more of them to each than
/// that, every one visits that much less often.
const CALLERS: f32 = 15.0;
/// How far out from the middle of the town the pavement between its ring
/// road and its first plots is walked along, in metres: clear of the
/// street lamps that stand 72.4 m out and of the plots' fronts, 79 m out
/// at the nearest.
const RING: f32 = 75.0;
/// How far out from the middle of its front a plot's gate is, the point
/// its resident leaves it and comes back to it by, in metres.
const GATE_OUT: f32 = 2.0;
/// How far apart the points along the ring are, in radians.
const ARC_STEP: f32 = 0.2;
/// How big the squares a way is looked for over are, in metres: two of
/// those a body is tested against (`island::LATTICE`) to a side. And how
/// wide a body is taken to be in looking, in metres: less than it is
/// ([`PLAYER_RADIUS`]), and just less than half a square, so that a
/// doorway a metre wide always has a square in it that is a way; it is
/// kept out of the walls on the way by what keeps any body out of them.
const SQUARE: f32 = 2.0 * LATTICE;
const SLIM: f32 = 0.24;
/// How many squares a way is looked for over before it is given up; how
/// many squares nobody has looked at before may be looked at in a frame,
/// by all the residents together (looking at one is what takes the time:
/// what is found of it is kept, [`Walkable`]); and how near the end of a
/// way is there, in metres.
const MOST_SQUARES: usize = 2_500;
const FRESH_A_FRAME: usize = 40;
const THERE: f32 = 0.3;
/// How long a resident may get no nearer where it is going before it looks
/// for another way, in seconds, and how much nearer is nearer, in metres;
/// and how many times it looks before it is put there.
const STUCK_SECS: f32 = 1.6;
const NEARER: f32 = 0.25;
const TRIES: u8 = 2;
/// How long a resident waits before looking again for something to do
/// that it could not find, in seconds: a bed that is not there yet, its
/// model still loading.
const LOOK_AGAIN: f32 = 2.0;
/// How long one just made waits before it is about anything, in seconds:
/// what stands on its plot is told and loaded in that while, and it is
/// then put where it would have been by now.
const SETTLE_IN: f32 = 2.5;
/// How far from a place to rest a body may stand to be put on it, in
/// metres: a little less than you may from its bubble (`rest`'s 3.7 m),
/// the beds being 2.3 m and more across, so that the middle of one is
/// further than a step from anywhere beside it. And how near a body
/// resting a place is taken.
const REACH: f32 = 3.4;
const TAKEN: f32 = 0.45;
/// How long someone standing about stands before they do some small thing,
/// in seconds, the least and the most.
const FIDGET: (f32, f32) = (2.5, 7.0);
/// How far over the ground somewhere to stand has to be clear from, in
/// metres: over a doorstep, and under the top of a low table, which is no
/// place to stand in.
const UNDERFOOT: f32 = 0.15;

/// Who a resident is, in four leanings (the module's own account): each a
/// share from 0 to 1 that the dice are thrown against.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Persona {
    /// How likely a session is a visit, with no more residents to each
    /// real person than [`CALLERS`].
    visits: f32,
    /// Of the time on their feet at home, how much is out of doors.
    outdoors: f32,
    /// How much of the time they are on their feet rather than settled.
    restless: f32,
    /// Of the time settled, how much is lying down.
    naps: f32,
    /// How likely two blocks running are the same thing.
    steady: f32,
    /// How fast they walk, as a share of a body's full pace.
    pace: f32,
}

impl Persona {
    /// Who the resident of the land of `who` is: the same on every device.
    pub(crate) fn of(who: u64) -> Self {
        let lean = |salt: u64| mix(who, 0, salt) & 1 == 0;
        let within = |salt: u64, low: f32, high: f32| low + (high - low) * dice(who, 1, salt);
        let (out, restless, feeling, steady) = (lean(1), lean(2), lean(3), lean(4));
        Self {
            visits: if out { within(5, 0.07, 0.13) } else { within(5, 0.02, 0.05) },
            outdoors: if out { within(6, 0.5, 0.75) } else { within(6, 0.15, 0.35) },
            restless: if restless { within(7, 0.5, 0.72) } else { within(7, 0.25, 0.42) },
            naps: if feeling { within(8, 0.3, 0.5) } else { within(8, 0.08, 0.25) },
            steady: if steady { within(9, 0.55, 0.8) } else { within(9, 0.15, 0.4) },
            pace: if steady { within(10, 0.45, 0.58) } else { within(10, 0.58, 0.75) },
        }
    }
}

/// A number made of `seed`, `count` and `salt` that looks like chance and
/// is the same wherever it is worked out (splitmix64).
fn mix(seed: u64, count: i64, salt: u64) -> u64 {
    let mut z = seed
        .wrapping_add((count as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(salt.wrapping_mul(0xD1B5_4A32_D192_ED03));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The dice: a number from 0 up to 1, thrown for `seed` the `count`th
/// time for the purpose `salt`, the same on every device.
fn dice(seed: u64, count: i64, salt: u64) -> f32 {
    (mix(seed, count, salt) >> 40) as f32 / (1u64 << 24) as f32
}

/// What a resident is about for a block.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum What {
    /// Out on the plot's grass.
    Lawn,
    /// At the front of the plot, looking out.
    Gate,
    /// About the house.
    Indoors,
    /// Sat on a chair or a sofa.
    Sit,
    /// Lying on a bed.
    Lie,
}

/// What a resident is about just now, and on which plot, by its number:
/// with it, what tells one such thing from the next ([`Plan::throw`]), for
/// the dice that choose where exactly.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Plan {
    what: What,
    plot: usize,
    throw: i64,
}

/// What the resident who is `who`, of the plot `home`, is about at `now`
/// by the town's clock, `hosts` being the plots of everyone real in the
/// channel, in order. The same on every device.
pub(crate) fn plan(who: u64, persona: &Persona, home: usize, hosts: &[usize], now: f64) -> Plan {
    // Each begins its blocks at its own moment.
    let block = ((now + f64::from(dice(who, 2, 11)) * BLOCK) / BLOCK).floor() as i64;
    let session = block.div_euclid(SESSION);
    // Every plot nobody real has has a resident: so many of them to each
    // real person, and each visits the less often the more there are.
    let residents = roundtown_net::land::PLOTS.saturating_sub(hosts.len()).max(1) as f32;
    let often = persona.visits * (CALLERS * hosts.len() as f32 / residents).min(1.0);
    let visiting = !hosts.is_empty() && dice(who, session, 12) < often && block.rem_euclid(SESSION) < STAY;
    let plot = if visiting {
        hosts[((dice(who, session, 13) * hosts.len() as f32) as usize).min(hosts.len() - 1)]
    } else {
        home
    };
    // Steady, two blocks running are one thing.
    let throw = if dice(who, block.div_euclid(2), 14) < persona.steady { block.div_euclid(2) * 2 } else { block };
    let what = if dice(who, throw, 15) < persona.restless {
        // A guest keeps to the house more than its owner does.
        let outdoors = if visiting { persona.outdoors * 0.5 } else { persona.outdoors };
        if dice(who, throw, 16) >= outdoors {
            What::Indoors
        } else if !visiting && dice(who, throw, 17) < 0.25 {
            What::Gate
        } else {
            What::Lawn
        }
    } else if dice(who, throw, 18) < persona.naps {
        What::Lie
    } else {
        What::Sit
    };
    Plan { what, plot, throw }
}

/// Where a resident is to be for what it is about: standing somewhere, or
/// resting on a place of a piece, come up to from a point beside it.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Aim {
    Stand { at: Vec3, face: Option<Vec2> },
    Rest { piece: Entity, spot: usize, from: Vec3 },
}

impl Aim {
    /// Where it is walked to.
    fn at(&self) -> Vec3 {
        match *self {
            Aim::Stand { at, .. } => at,
            Aim::Rest { from, .. } => from,
        }
    }
}

/// What a resident rests on: the piece and how it stood as they came to
/// it, and where they stood before, which is where they get up to.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Sat {
    piece: Entity,
    stood: Transform,
    from: Vec3,
}

/// One of the computer's who walks a plot of the town under its owner's
/// name.
#[derive(Component)]
pub(crate) struct Resident {
    who: u64,
    /// Their own plot, by its number.
    home: usize,
    persona: Persona,
    /// What they are about, once they have found where.
    plan: Option<Plan>,
    aim: Option<Aim>,
    /// The points still to be gone by on the way there, the last first,
    /// and the way to the next of them, a step at a time.
    stages: Vec<Vec3>,
    way: Vec<Vec3>,
    /// A way being looked for, a little each frame.
    search: Option<Search>,
    /// How near the next point they have got, for how long they have got
    /// no nearer, and how many times they have looked for another way.
    nearest: f32,
    stalled: f32,
    tried: u8,
    /// What they rest on, while they do.
    sat: Option<Sat>,
    /// Seconds until they look again for something to do, having found
    /// nothing, or having only just been made, what stands on the plots
    /// still loading; and until the next small thing done standing about.
    again: f32,
    fidget: f32,
    /// Which way they look, standing about.
    look: Option<Vec2>,
    /// Just made: put where they would be by now, rather than seen to walk
    /// there from the middle of their plot.
    fresh: bool,
}

impl Resident {
    fn new(who: u64, home: usize) -> Self {
        Self {
            who,
            home,
            persona: Persona::of(who),
            plan: None,
            aim: None,
            stages: Vec::new(),
            way: Vec::new(),
            search: None,
            nearest: f32::INFINITY,
            stalled: 0.0,
            tried: 0,
            sat: None,
            again: SETTLE_IN,
            fidget: FIDGET.1,
            look: None,
            fresh: true,
        }
    }

    /// Lets go of wherever they were going.
    fn forget_the_way(&mut self) {
        self.aim = None;
        self.stages.clear();
        self.way.clear();
        self.search = None;
        self.nearest = f32::INFINITY;
        self.stalled = 0.0;
        self.tried = 0;
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (keep, mind)
            .chain()
            .after(crate::land::DeedSystems)
            .after(crate::read_player_input)
            .before(crate::move_bodies),
    );
}

/// What the resident of the land `deed` says of goes by: a made-up owner's
/// own name, the land being theirs and they nobody else; on the land of
/// someone real who is not here, "{name}'s bot" (`words.land.bot`), which
/// nobody takes for that person come online (Hajun, 2026-11-24; the plain
/// name for the day before). The sign says whose the land is either way.
fn called(deed: &Deed) -> String {
    if roundtown_net::land::is_premade(deed.who) {
        deed.name.clone()
    } else {
        fill(&ui().words.land.bot, &[("name", &deed.name)])
    }
}

/// Keeps a resident on every plot that is to have one (`land::Deeds`), as
/// whose each plot is changes, and takes away those of plots that are not:
/// someone real has come to theirs, or the land that fills it is someone
/// else's now, or what they go by is other words.
fn keep(
    deeds: Res<Deeds>,
    plots: Res<Plots>,
    words: Res<UiChanged>,
    residents: Query<(Entity, &Resident, &Name)>,
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    if !(deeds.is_changed() || plots.is_changed() || words.is_changed()) || plots.all.is_empty() {
        return;
    }
    let wanted: Vec<(&Deed, String)> = deeds
        .0
        .iter()
        .filter(|deed| deed.holder == Holder::Minded)
        .map(|deed| (deed, called(deed)))
        .collect();
    let is = |(deed, called): &(&Deed, String), resident: &Resident, name: &Name| {
        deed.who == resident.who && usize::from(deed.plot) == resident.home && called.as_str() == name.as_str()
    };
    for (body, resident, name) in &residents {
        if !wanted.iter().any(|one| is(one, resident, name)) {
            commands.entity(body).despawn();
        }
    }
    for one in &wanted {
        if residents.iter().any(|(_, resident, name)| is(one, resident, name)) {
            continue;
        }
        let (deed, called) = one;
        let home = usize::from(deed.plot);
        let Some(plot) = plots.of(home) else {
            continue;
        };
        let member = Member {
            id: deed.who,
            name: called.clone(),
            look: deed.look.clone(),
            ai: true,
        };
        // In the middle of their plot, facing out over its front, until
        // they have somewhere to be.
        let at = Transform::from_xyz(plot.middle.x, plot.ground, plot.middle.y)
            .looking_to(Vec3::new(plot.front.x, 0.0, plot.front.y), Vec3::Y);
        let body = lobby::spawn_resident(&mut commands, &assets, &member, home, at);
        commands.entity(body).insert(Resident::new(deed.who, home));
    }
}

/// A piece that stands in the town, as a resident looks at it.
type Pieces<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Piece, &'static Transform, &'static Bounds, &'static Venue),
    (Without<Ghost>, Without<Resident>, Without<ChildOf>),
>;

/// Has every resident be about what it is about ([`plan`]): finds where,
/// walks it there a step at a time, sits or lays it down when it is there,
/// and gets it up again when it is time for something else. One you are
/// talking to stands and listens (`talk`). While you are anywhere but the
/// town they stand where they are: nobody is looking.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn mind(
    time: Res<Time>,
    clock: Res<TownClock>,
    islands: Res<Islands>,
    plots: Res<Plots>,
    deeds: Res<Deeds>,
    me: Query<(&Seat, &Venue), With<Player>>,
    mut residents: Query<(Entity, &mut Resident, &mut Transform, &mut Intent, Has<Listening>)>,
    pieces: Pieces,
    others: Query<&Transform, (With<Resting>, Without<Resident>)>,
    mut walkable: Local<Walkable>,
    mut commands: Commands,
) {
    let Ok((seat, &here)) = me.single() else {
        return;
    };
    // What was found of the town's squares holds until its shape changes:
    // something put down on a plot, or taken off one.
    if islands.is_changed() {
        walkable.known.clear();
    }
    walkable.fresh = 0;
    let (Some(island), true) = (islands.get(Venue::Town), here == Venue::Town) else {
        for (.., mut intent, _) in &mut residents {
            *intent = Intent::default();
        }
        return;
    };
    let dt = time.delta_secs();
    let now = clock.now();
    // Everyone real in the channel, you among them, by their plots.
    let mut hosts: Vec<usize> = deeds
        .0
        .iter()
        .filter(|deed| matches!(deed.holder, Holder::Here | Holder::Away))
        .map(|deed| usize::from(deed.plot))
        .chain([seat.0])
        .filter(|&plot| plots.of(plot).is_some())
        .collect();
    hosts.sort_unstable();
    hosts.dedup();
    // Where anyone rests already: those places are theirs.
    let mut taken: Vec<Vec3> = others.iter().map(|body| body.translation).collect();
    taken.extend(residents.iter().filter(|(_, resident, ..)| resident.sat.is_some()).map(|(_, _, at, ..)| at.translation));
    for (body, mut resident, mut place, mut intent, listening) in &mut residents {
        *intent = Intent::default();
        if listening {
            continue;
        }
        let wanted = plan(resident.who, &resident.persona, resident.home, &hosts, now);
        // Up from what they rest on, when it is time for something else,
        // or it has been moved from under them.
        if let Some(sat) = resident.sat {
            let stands = pieces.get(sat.piece).is_ok_and(|(_, _, stood, ..)| *stood == sat.stood);
            if resident.plan == Some(wanted) && stands {
                continue;
            }
            commands.entity(body).remove::<Resting>().insert(Speaker::Townsfolk);
            place.translation = sat.from;
            resident.sat = None;
            resident.forget_the_way();
        }
        // Something else to be about, or nowhere found for it yet.
        if resident.plan != Some(wanted) || resident.aim.is_none() {
            if resident.aim.is_none() && resident.again > 0.0 {
                resident.again -= dt;
                continue;
            }
            resident.plan = Some(wanted);
            resident.look = None;
            resident.forget_the_way();
            let Some(aim) = find(&wanted, resident.who, island, &plots, &pieces, &taken) else {
                resident.again = LOOK_AGAIN;
                continue;
            };
            resident.aim = Some(aim);
            if resident.fresh {
                // As if they had been there a while.
                resident.fresh = false;
                place.translation = aim.at();
            } else {
                resident.stages = stages(&plots, place.translation, &wanted, aim.at(), island);
            }
        }
        resident.fresh = false;
        let Some(aim) = resident.aim else {
            continue;
        };
        // On the way: the next point of it, and the way there.
        let to = match resident.way.last().or(resident.stages.last()) {
            Some(&next) => next,
            None => aim.at(),
        };
        let off = (to - place.translation).xz();
        if off.length() > THERE {
            if resident.way.is_empty() {
                let stage = resident.stages.last().copied().unwrap_or(aim.at());
                // Straight there if a body can walk it; otherwise a way is
                // looked for, a little a frame, and they stand meanwhile.
                if resident.search.is_none() {
                    if walkable.fresh >= FRESH_A_FRAME {
                        continue;
                    }
                    if straight(island, &mut walkable, place.translation, stage) {
                        resident.way = vec![stage];
                    } else {
                        resident.search = Some(Search::new(place.translation, stage));
                    }
                }
                if let Some(search) = resident.search.as_mut() {
                    match search.go(island, &mut walkable, FRESH_A_FRAME) {
                        None => continue,
                        // The last first, as the stages are.
                        Some(Some(mut found)) => {
                            found.reverse();
                            resident.way = found;
                        }
                        // No way found: straight at it, and whatever keeps
                        // them from getting there is met below.
                        Some(None) => resident.way = vec![stage],
                    }
                    resident.search = None;
                }
            }
            // Nearer, or not: stalled long enough, another way is looked
            // for, and after a few of those they are put there.
            let far = off.length();
            if far < resident.nearest - NEARER {
                (resident.nearest, resident.stalled) = (far, 0.0);
            } else {
                resident.stalled += dt;
            }
            if resident.stalled > STUCK_SECS {
                (resident.nearest, resident.stalled) = (f32::INFINITY, 0.0);
                resident.way.clear();
                resident.tried += 1;
                if resident.tried > TRIES {
                    resident.tried = 0;
                    place.translation = resident.stages.pop().unwrap_or(aim.at());
                }
                continue;
            }
            let pace = if resident.stages.len() > 1 { 0.9 } else { resident.persona.pace };
            intent.walk = Vec3::new(off.x, 0.0, off.y).normalize_or_zero() * pace;
            continue;
        }
        // There: the next point, or the place itself.
        (resident.nearest, resident.stalled, resident.tried) = (f32::INFINITY, 0.0, 0);
        if resident.way.pop().is_some() && !resident.way.is_empty() {
            continue;
        }
        if resident.stages.pop().is_some() {
            continue;
        }
        match aim {
            Aim::Rest { piece, spot, .. } => {
                let Ok((_, of, stood, ..)) = pieces.get(piece) else {
                    resident.forget_the_way();
                    continue;
                };
                let Some(on) = rest::spots(of.def.kind).get(spot) else {
                    resident.forget_the_way();
                    continue;
                };
                let rests = rest::rests_at(stood, on);
                if taken.iter().any(|other| other.distance(rests.translation) <= TAKEN) {
                    // Someone got there first: somewhere else of the kind
                    // is looked for, from where they stand.
                    resident.forget_the_way();
                    resident.again = 0.0;
                    continue;
                }
                taken.push(rests.translation);
                resident.sat = Some(Sat {
                    piece,
                    stood: *stood,
                    from: place.translation,
                });
                *place = rests;
                // Nobody is talked to lying down.
                commands.entity(body).insert(Resting::begun(on.pose)).remove::<Speaker>();
            }
            Aim::Stand { face, .. } => {
                // Standing about: looking the way they came to look, and
                // now and then another way, or a hop. These small things
                // are each device's own to throw.
                if resident.look.is_none() {
                    resident.look = face;
                }
                resident.fidget -= dt;
                if resident.fidget <= 0.0 {
                    resident.fidget = FIDGET.0 + fastrand::f32() * (FIDGET.1 - FIDGET.0);
                    let turn = fastrand::f32() * TAU;
                    resident.look = face.filter(|_| fastrand::f32() < 0.5).or(Some(Vec2::new(turn.cos(), turn.sin())));
                    intent.jump = fastrand::f32() < 0.12 * resident.persona.restless;
                }
                intent.face = resident.look.map(|look| Vec3::new(look.x, 0.0, look.y));
            }
        }
    }
}

/// Where the resident who is `who` is to be for `plan`: somewhere of the
/// kind on its plot, chosen by the dice of who they are and which such
/// thing this is, so that it is the same place on every device. None if
/// the plot is not there, or there is nowhere to stand on it: its things
/// still loading, say.
fn find(plan: &Plan, who: u64, island: &Island, plots: &Plots, pieces: &Pieces, taken: &[Vec3]) -> Option<Aim> {
    let plot = plots.of(plan.plot)?;
    let yard = yard_of(plot);
    // What stands on the plot, in an order every device has it in: by
    // where each stands, to the centimetre.
    let mut on_it: Vec<(Entity, &Piece, &Transform, &Bounds)> = pieces
        .iter()
        .filter(|(_, _, at, _, venue)| **venue == Venue::Town && yard.holds(at.translation.xz(), 0.0))
        .map(|(piece, of, at, bounds, _)| (piece, of, at, bounds))
        .collect();
    let cm = |at: &Transform| (at.translation * 100.0).round().as_ivec3().to_array();
    on_it.sort_by_key(|(_, _, at, _)| cm(at));
    let house = on_it.iter().find(|(_, of, ..)| of.def.mount == Mount::House).map(|&(_, _, at, bounds)| (*at, *bounds));
    let throw = |salt: u64, nth: usize| dice(who, plan.throw, salt + 100 * nth as u64 + plan.plot as u64 * 10_000);
    let resting = |lying: bool| -> Option<Aim> {
        // Every place of the kind on the ground floor, a point to come up
        // to it from, and nobody on it.
        let places: Vec<(Entity, usize, Vec3)> = on_it
            .iter()
            .filter(|(_, of, ..)| rest::residents_sit_on(of.def.kind))
            .flat_map(|&(piece, of, at, _)| {
                rest::spots(of.def.kind)
                    .iter()
                    .enumerate()
                    .filter(move |(_, spot)| (spot.pose == Pose::Lie) == lying)
                    .map(move |(nth, spot)| (piece, nth, rest::rests_at(at, spot).translation))
            })
            .filter(|(.., rests)| rests.y - yard.ground < 2.2 && taken.iter().all(|other| other.distance(*rests) > TAKEN))
            .collect();
        if places.is_empty() {
            return None;
        }
        let first = (throw(21, 0) * places.len() as f32) as usize;
        (0..places.len()).map(|step| places[(first + step) % places.len()]).find_map(|(piece, spot, rests)| {
            let from = beside(island, rests, yard.ground)?;
            Some(Aim::Rest { piece, spot, from })
        })
    };
    let indoors = || -> Option<Aim> {
        let (at, bounds) = house?;
        let room = (bounds.high - bounds.low).xz() - Vec2::splat(2.0);
        if room.min_element() <= 0.0 {
            return None;
        }
        (0..8).find_map(|nth| {
            let own = Vec3::new(
                bounds.low.x + 1.0 + room.x * throw(22, nth),
                bounds.low.y,
                bounds.low.z + 1.0 + room.y * throw(23, nth),
            );
            let at = stand_at(island, at.transform_point(own), yard.ground)?;
            Some(Aim::Stand { at, face: None })
        })
    };
    let lawn = || -> Option<Aim> {
        let reach = yard.half - 1.5;
        let right = Vec2::new(-yard.front.y, yard.front.x);
        (0..8).find_map(|nth| {
            let across = yard.middle + yard.front * reach * (2.0 * throw(24, nth) - 1.0) + right * reach * (2.0 * throw(25, nth) - 1.0);
            // Not in the house: that is being indoors.
            let inside = house.is_some_and(|(at, bounds)| {
                let own = at.compute_affine().inverse().transform_point3(Vec3::new(across.x, yard.ground, across.y));
                own.x > bounds.low.x && own.x < bounds.high.x && own.z > bounds.low.z && own.z < bounds.high.z
            });
            let at = (!inside).then(|| stand_at(island, Vec3::new(across.x, yard.ground, across.y), yard.ground)).flatten()?;
            Some(Aim::Stand { at, face: None })
        })
    };
    let gate = || -> Option<Aim> {
        let right = Vec2::new(-yard.front.y, yard.front.x);
        (0..6).find_map(|nth| {
            let across = yard.middle + yard.front * (yard.half - 1.2) + right * 4.0 * (2.0 * throw(26, nth) - 1.0);
            let at = stand_at(island, Vec3::new(across.x, yard.ground, across.y), yard.ground)?;
            Some(Aim::Stand { at, face: Some(yard.front) })
        })
    };
    // Each thing, and what is done in its place where there is none of it:
    // no bed, a seat; no seat, about the house; no house, the lawn.
    match plan.what {
        What::Lie => resting(true).or_else(|| resting(false)).or_else(indoors).or_else(lawn),
        What::Sit => resting(false).or_else(indoors).or_else(lawn),
        What::Indoors => indoors().or_else(lawn),
        What::Gate => gate().or_else(lawn),
        What::Lawn => lawn(),
    }
}

/// How high the ground is where a body could stand at `at`, if it could:
/// dry land within a step of `near`, and nothing of the town in it from
/// its knees to its head, as wide as it is.
fn stand_at(island: &Island, at: Vec3, near: f32) -> Option<Vec3> {
    // The ground a body walking there would be on: the floor, and not the
    // top of the bed that stands on it.
    let ground = crate::dry_ground(island, at.with_y(near))?;
    let clear = island
        .walls_round(at.xz(), &[(PLAYER_RADIUS, ground + UNDERFOOT, ground + PLAYER_HEIGHT)])
        .is_empty();
    (clear && (ground - near).abs() <= LAND_STEP_UP).then_some(at.with_y(ground))
}

/// Somewhere to stand beside a place to rest at `rests`, near enough to be
/// put on it ([`REACH`]) and in sight of it, as you have to be of its
/// bubble (`rest`): the nearest there is, looked for round it. Not through
/// a wall, from the lawn outside.
fn beside(island: &Island, rests: Vec3, ground: f32) -> Option<Vec3> {
    [0.7f32, 0.95, 1.2, 1.5, 1.8, 2.2, 2.6, 3.0, REACH]
        .into_iter()
        .flat_map(|out| (0..16).map(move |nth| (out, nth as f32 * TAU / 16.0)))
        .filter_map(|(out, turn)| stand_at(island, rests + Vec3::new(turn.cos(), 0.0, turn.sin()) * out, ground))
        .find(|from| island.sweep(*from + Vec3::Y * crate::EYE_HEIGHT, rests + Vec3::Y * 0.6, 0.05).is_none())
}

/// A plot's gate: the point out by the middle of its front that its
/// resident leaves it and comes back to it by.
fn gate_of(plot: &Plot) -> Vec2 {
    plot.middle + plot.front * (yard_of(plot).half + GATE_OUT)
}

/// The points to go by from `from` to `to` on the plot `plan` is for, the
/// last first: none but `to`, on the plot one is on already; otherwise out
/// by the gate of the plot one is on, along the ring the shorter way
/// round, and in by the other's gate.
fn stages(plots: &Plots, from: Vec3, plan: &Plan, to: Vec3, island: &Island) -> Vec<Vec3> {
    let on = roundtown_net::land::plot_at(from.xz());
    let Some(there) = plots.of(plan.plot) else {
        return vec![to];
    };
    if on == Some(plan.plot) || yard_of(there).holds(from.xz(), GATE_OUT) {
        return vec![to];
    }
    let grounded = |at: Vec2| {
        let ground = island.floor(at, from.y.max(to.y) + 2.0).unwrap_or(from.y);
        Vec3::new(at.x, ground, at.y)
    };
    let mut by: Vec<Vec2> = Vec::new();
    // Out by the gate of the plot they are on, if they are on one.
    let leaving = on.and_then(|plot| plots.of(plot)).map(gate_of);
    by.extend(leaving);
    let start = leaving.unwrap_or(from.xz());
    let arriving = gate_of(there);
    // Round the ring from where the one is nearest it to where the other
    // is, the shorter way.
    let (a, b) = (start.y.atan2(start.x), arriving.y.atan2(arriving.x));
    let turn = (b - a + std::f32::consts::PI).rem_euclid(TAU) - std::f32::consts::PI;
    let steps = (turn.abs() / ARC_STEP).ceil().max(1.0) as usize;
    for step in 0..=steps {
        let at = a + turn * step as f32 / steps as f32;
        by.push(Vec2::new(at.cos(), at.sin()) * RING);
    }
    by.push(arriving);
    let mut stages: Vec<Vec3> = by.into_iter().map(grounded).collect();
    stages.push(to);
    stages.reverse();
    stages
}

/// Whether a body could stand at `at`, coming to it from ground `from`
/// high: how high its ground is, if so. Dry land no further up or down
/// than a step, and nothing of the town in it from its knees to its head,
/// [`SLIM`] wide.
fn passable(island: &Island, at: Vec2, from: f32) -> Option<f32> {
    let ground = crate::dry_ground(island, Vec3::new(at.x, from, at.y))?;
    if from - ground > LAND_STEP_UP {
        return None;
    }
    island
        .walls_round(at, &[(SLIM, ground + LAND_STEP_UP, ground + PLAYER_HEIGHT)])
        .is_empty()
        .then_some(ground)
}

/// The square `at` is in, and the middle of a square.
fn square_of(at: Vec2) -> IVec2 {
    (at / SQUARE).floor().as_ivec2()
}

fn middle_of(square: IVec2) -> Vec2 {
    (square.as_vec2() + 0.5) * SQUARE
}

/// What has been found of the town's squares: whether a body could stand
/// in the middle of each, coming to it from about so high, and how high
/// its ground is ([`passable`]). Looking at a square is what looking for a
/// way costs, the town's shape being gone through for each; found once, it
/// holds until that shape changes. How many were looked at this frame is
/// counted, for the looking to be shared out over frames.
#[derive(Default)]
pub(crate) struct Walkable {
    known: HashMap<(IVec2, i16), Option<f32>>,
    fresh: usize,
}

impl Walkable {
    fn at(&mut self, island: &Island, square: IVec2, from: f32) -> Option<f32> {
        // From about the same height: to a quarter of a metre.
        let key = (square, (from * 4.0).round() as i16);
        if let Some(&known) = self.known.get(&key) {
            return known;
        }
        self.fresh += 1;
        let found = passable(island, middle_of(square), from);
        self.known.insert(key, found);
        found
    }
}

/// Whether a body could walk straight from `from` to `to`: every square
/// the line goes through can be stood in, each from the one before.
fn straight(island: &Island, walkable: &mut Walkable, from: Vec3, to: Vec3) -> bool {
    let across = (to - from).xz();
    let steps = (across.length() / LATTICE).ceil().max(1.0) as usize;
    let mut ground = from.y;
    let mut last = square_of(from.xz());
    for step in 1..=steps {
        let square = square_of(from.xz() + across * (step as f32 / steps as f32));
        if square == last {
            continue;
        }
        last = square;
        match walkable.at(island, square, ground) {
            Some(next) => ground = next,
            None => return false,
        }
    }
    true
}

/// A way being looked for from one place to another over the island's
/// squares, [`SQUARE`] to a side: from one to any of the eight round it
/// (to a corner only with both squares beside it free), the nearest to
/// the end first (A*).
pub(crate) struct Search {
    from: Vec3,
    to: Vec3,
    /// Each square reached: how high its ground is, how far it is by the
    /// way found, and the square it was come to from.
    reached: HashMap<IVec2, (f32, f32, IVec2)>,
    open: BinaryHeap<(Reverse<u32>, [i32; 2])>,
    looked: usize,
}

impl Search {
    fn new(from: Vec3, to: Vec3) -> Self {
        let first = square_of(from.xz());
        let mut search = Self {
            from,
            to,
            reached: HashMap::from([(first, (from.y, 0.0, first))]),
            open: BinaryHeap::new(),
            looked: 0,
        };
        search.open.push((Self::key(search.far(first)), first.to_array()));
        search
    }

    fn key(cost: f32) -> Reverse<u32> {
        Reverse((cost * 1024.0) as u32)
    }

    fn far(&self, square: IVec2) -> f32 {
        middle_of(square).distance(self.to.xz())
    }

    /// Goes on looking, until the way is found, there is found to be none
    /// within [`MOST_SQUARES`], or as many squares nobody had looked at
    /// have been looked at this frame as may be (`most_fresh`, counted by
    /// `walkable`): then `None`, to go on with next frame. A way is as few
    /// points as it can be gone by in straight lines, the first to be gone
    /// to first and the end the last.
    fn go(&mut self, island: &Island, walkable: &mut Walkable, most_fresh: usize) -> Option<Option<Vec<Vec3>>> {
        const ROUND: [IVec2; 8] = [
            IVec2::X,
            IVec2::NEG_X,
            IVec2::Y,
            IVec2::NEG_Y,
            IVec2::new(1, 1),
            IVec2::new(1, -1),
            IVec2::new(-1, 1),
            IVec2::new(-1, -1),
        ];
        let last = square_of(self.to.xz());
        loop {
            if walkable.fresh >= most_fresh {
                return None;
            }
            let Some((_, at)) = self.open.pop() else {
                return Some(None);
            };
            let at = IVec2::from_array(at);
            if at == last || self.far(at) <= THERE {
                return Some(Some(self.way_to(island, walkable, at)));
            }
            self.looked += 1;
            if self.looked > MOST_SQUARES {
                return Some(None);
            }
            let (ground, gone, _) = self.reached[&at];
            // Which of the four beside it can be stood on, for the corners.
            let beside = [0, 1, 2, 3].map(|nth| walkable.at(island, at + ROUND[nth], ground));
            for (nth, step) in ROUND.into_iter().enumerate() {
                let next = at + step;
                let height = match nth {
                    0..4 => beside[nth],
                    _ => {
                        let (x, y) = (if step.x > 0 { 0 } else { 1 }, if step.y > 0 { 2 } else { 3 });
                        (beside[x].is_some() && beside[y].is_some()).then(|| walkable.at(island, next, ground)).flatten()
                    }
                };
                let Some(height) = height else {
                    continue;
                };
                let gone = gone + step.as_vec2().length() * SQUARE;
                if self.reached.get(&next).is_some_and(|&(_, before, _)| before <= gone) {
                    continue;
                }
                self.reached.insert(next, (height, gone, at));
                self.open.push((Self::key(gone + self.far(next)), next.to_array()));
            }
        }
    }

    /// The way found to the square `end`: back from it to the beginning,
    /// and then only the points a body has to turn at.
    fn way_to(&self, island: &Island, walkable: &mut Walkable, end: IVec2) -> Vec<Vec3> {
        let mut squares = vec![end];
        while let Some(&(_, _, before)) = squares.last().and_then(|square| self.reached.get(square)) {
            if Some(&before) == squares.last() {
                break;
            }
            squares.push(before);
        }
        squares.reverse();
        let mut points: Vec<Vec3> = squares.iter().map(|square| middle_of(*square).extend(self.reached[square].0).xzy()).collect();
        points[0] = self.from;
        points.push(self.to);
        let mut way = Vec::new();
        let mut at = 0;
        while at + 1 < points.len() {
            let next = (at + 2..points.len()).rev().find(|&next| straight(island, walkable, points[at], points[next])).unwrap_or(at + 1);
            way.push(points[next]);
            at = next;
        }
        way
    }
}

/// A way a body could walk from `from` to `to` over the island, looked
/// for all at once ([`Search`]): none if there is none.
#[cfg(test)]
fn path(island: &Island, from: Vec3, to: Vec3) -> Option<Vec<Vec3>> {
    let mut walkable = Walkable::default();
    Search::new(from, to).go(island, &mut walkable, usize::MAX).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A slab of ground `half` to each side of the middle, at no height,
    /// and whatever boxes stand on it: each its low and high corners.
    fn ground(half: f32, boxes: &[(Vec3, Vec3)]) -> Island {
        let quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| [[a, b, c], [a, c, d]];
        let mut faces = Vec::new();
        // Its top, looking up.
        faces.extend(quad(
            Vec3::new(-half, 0.0, -half),
            Vec3::new(-half, 0.0, half),
            Vec3::new(half, 0.0, half),
            Vec3::new(half, 0.0, -half),
        ));
        let mut parts = vec![faces];
        for &(low, high) in boxes {
            let corner = |x: bool, y: bool, z: bool| Vec3::new(if x { high.x } else { low.x }, if y { high.y } else { low.y }, if z { high.z } else { low.z });
            let mut sides = Vec::new();
            sides.extend(quad(corner(false, true, false), corner(false, true, true), corner(true, true, true), corner(true, true, false)));
            sides.extend(quad(corner(false, false, false), corner(true, false, false), corner(true, false, true), corner(false, false, true)));
            sides.extend(quad(corner(false, false, false), corner(false, true, false), corner(true, true, false), corner(true, false, false)));
            sides.extend(quad(corner(false, false, true), corner(true, false, true), corner(true, true, true), corner(false, true, true)));
            sides.extend(quad(corner(false, false, false), corner(false, false, true), corner(false, true, true), corner(false, true, false)));
            sides.extend(quad(corner(true, false, false), corner(true, true, false), corner(true, true, true), corner(true, false, true)));
            // Each wound to look away from the middle of the box, as a
            // solid's faces have to.
            let middle = (low + high) * 0.5;
            for [a, b, c] in &mut sides {
                if (*b - *a).cross(*c - *a).dot((*a + *b + *c) / 3.0 - middle) < 0.0 {
                    std::mem::swap(b, c);
                }
            }
            parts.push(sides);
        }
        let parts: Vec<&[[Vec3; 3]]> = parts.iter().map(Vec::as_slice).collect();
        Island::of_parts_for_tests(&parts)
    }

    #[test]
    fn who_a_resident_is_is_the_same_wherever_it_is_worked_out_and_no_two_are_alike() {
        let all: Vec<Persona> = (0..48u64).map(|n| Persona::of(roundtown_net::land::PREMADE + n)).collect();
        assert_eq!(all[3], Persona::of(roundtown_net::land::PREMADE + 3));
        for (nth, one) in all.iter().enumerate() {
            assert!(all[nth + 1..].iter().all(|other| other != one));
            // Each a share, and nobody visits most of the time.
            for share in [one.visits, one.outdoors, one.restless, one.naps, one.steady, one.pace] {
                assert!((0.0..=1.0).contains(&share), "{one:?}");
            }
            assert!(one.visits <= 0.13 && one.pace >= 0.45);
        }
        // Both ways of each leaning are there among them.
        assert!(all.iter().any(|one| one.visits > 0.06) && all.iter().any(|one| one.visits < 0.06));
        assert!(all.iter().any(|one| one.restless > 0.45) && all.iter().any(|one| one.restless < 0.45));
        // The dice are numbers from nothing up to one, evenly.
        let thrown: Vec<f32> = (0..2_000).map(|count| dice(7, count, 3)).collect();
        assert!(thrown.iter().all(|throw| (0.0..1.0).contains(throw)));
        let mean = thrown.iter().sum::<f32>() / thrown.len() as f32;
        assert!((mean - 0.5).abs() < 0.03, "{mean}");
    }

    #[test]
    fn a_day_is_mostly_at_home_with_a_visit_now_and_then_and_the_same_on_every_device() {
        let who = roundtown_net::land::PREMADE + 4;
        let persona = Persona::of(who);
        let hosts = [0, 2];
        let day: Vec<Plan> = (0..2_880).map(|block| plan(who, &persona, 4, &hosts, f64::from(block) * BLOCK)).collect();
        // The same asked again, and anywhere within a block.
        assert_eq!(plan(who, &persona, 4, &hosts, 300.0), plan(who, &persona, 4, &hosts, 300.0));
        let offset = f64::from(dice(who, 2, 11)) * BLOCK;
        let begins = (10.0 * BLOCK - offset).max(0.0) + 0.01;
        assert_eq!(plan(who, &persona, 4, &hosts, begins), plan(who, &persona, 4, &hosts, begins + BLOCK - 0.02));
        // At home most of the day, at one of the two now and then, and
        // never anywhere else.
        let away = day.iter().filter(|plan| plan.plot != 4).count();
        assert!(day.iter().all(|plan| plan.plot == 4 || hosts.contains(&plan.plot)));
        assert!(away > 0 && away < day.len() / 5, "{away} blocks of {} away", day.len());
        // A visit is one stretch at one plot, and then home.
        for session in day.chunks(SESSION as usize) {
            let plots: Vec<usize> = session.iter().map(|plan| plan.plot).collect();
            if plots.iter().any(|&plot| plot != 4) {
                let host = *plots.iter().find(|&&plot| plot != 4).unwrap();
                assert!(plots.iter().all(|&plot| plot == host || plot == 4), "{plots:?}");
            }
        }
        // Of the 47 who are about one person alone in a channel, one or so
        // is at theirs at a time, taken over a day, and seldom more than a
        // few at once.
        let people: Vec<(u64, Persona)> = (1..48u64).map(|n| (roundtown_net::land::PREMADE + n, Persona::of(roundtown_net::land::PREMADE + n))).collect();
        let at_yours = |block: i32| {
            people
                .iter()
                .enumerate()
                .filter(|(n, (who, persona))| plan(*who, persona, n + 1, &[0], f64::from(block) * BLOCK).plot == 0)
                .count()
        };
        let guests: Vec<usize> = (0..2_880).map(at_yours).collect();
        let mean = guests.iter().sum::<usize>() as f32 / guests.len() as f32;
        let crowded = guests.iter().filter(|&&at_once| at_once > 3).count();
        assert!((0.3..1.3).contains(&mean), "{mean} guests at a time");
        assert!(crowded < guests.len() / 20, "more than three at once in {crowded} blocks of {}", guests.len());
        // And as many at each of sixteen people's, were there so many: no
        // more than fifteen residents to each, everyone visits as often as
        // they lean to.
        let hosts: Vec<usize> = (0..16).collect();
        let about: Vec<(usize, u64, Persona)> = (16..48usize).map(|n| (n, roundtown_net::land::PREMADE + n as u64, Persona::of(roundtown_net::land::PREMADE + n as u64))).collect();
        let out = |block: i32| about.iter().filter(|(home, who, persona)| plan(*who, persona, *home, &hosts, f64::from(block) * BLOCK).plot != *home).count();
        let mean = (0..2_880).map(out).sum::<usize>() as f32 / 2_880.0 / 16.0;
        assert!((0.03..0.2).contains(&mean), "{mean} guests at a time at each of sixteen");
        let hosts = [0, 2];
        // With nobody real about, never away.
        assert!((0..2_880).all(|block| plan(who, &persona, 4, &[], f64::from(block) * BLOCK).plot == 4));
        // Everything there is to be about is, in a day, by someone.
        for what in [What::Lawn, What::Gate, What::Indoors, What::Sit, What::Lie] {
            let anyone = (0..16u64).any(|n| {
                let who = roundtown_net::land::PREMADE + n;
                let persona = Persona::of(who);
                (0..2_880).any(|block| plan(who, &persona, n as usize, &hosts, f64::from(block) * BLOCK).what == what)
            });
            assert!(anyone, "nobody is ever about {what:?}");
        }
    }

    #[test]
    fn a_way_goes_round_what_is_in_it_and_through_a_gap_a_body_fits() {
        // Open ground: straight there, one point.
        let open = ground(20.0, &[]);
        let (from, to) = (Vec3::new(-6.0, 0.0, 0.1), Vec3::new(6.0, 0.0, 0.1));
        assert!(straight(&open, &mut Walkable::default(), from, to));
        assert_eq!(path(&open, from, to), Some(vec![to]));
        // A wall across the way with a doorway 1.2 m wide in it, off to
        // one side: through the doorway, in a few straight lines.
        let wall = [
            (Vec3::new(-0.15, 0.0, -8.0), Vec3::new(0.15, 2.5, 3.0)),
            (Vec3::new(-0.15, 0.0, 4.2), Vec3::new(0.15, 2.5, 8.0)),
        ];
        let walled = ground(20.0, &wall);
        assert!(!straight(&walled, &mut Walkable::default(), from, to));
        let way = path(&walled, from, to).expect("a way through the doorway");
        assert_eq!(way.last(), Some(&to));
        assert!(way.len() >= 2 && way.len() <= 6, "{way:?}");
        // It crosses the wall's line in the doorway, and every stretch of
        // it can be walked.
        let mut at = from;
        for &next in &way {
            assert!(straight(&walled, &mut Walkable::default(), at, next), "{at} to {next}");
            if (at.x < 0.0) != (next.x < 0.0) {
                let across = at.lerp(next, at.x.abs() / (at.x - next.x).abs());
                assert!((3.0..4.2).contains(&across.z), "crossed at {across}");
            }
            at = next;
        }
        // Walled in all round, there is no way.
        let pen = [
            (Vec3::new(-8.0, 0.0, -8.0), Vec3::new(-7.7, 2.5, 8.0)),
            (Vec3::new(-4.3, 0.0, -8.0), Vec3::new(-4.0, 2.5, 8.0)),
            (Vec3::new(-8.0, 0.0, -8.0), Vec3::new(-4.0, 2.5, -7.7)),
            (Vec3::new(-8.0, 0.0, 7.7), Vec3::new(-4.0, 2.5, 8.0)),
        ];
        assert_eq!(path(&ground(20.0, &pen), from, to), None);
        // Looked for a little at a time, it is the same way: so many
        // squares nobody has looked at a frame, and no more; and looked for
        // again, nothing is looked at twice.
        let mut walkable = Walkable::default();
        let mut search = Search::new(from, to);
        let mut frames = 0;
        let slowly = loop {
            walkable.fresh = 0;
            frames += 1;
            match search.go(&walled, &mut walkable, 20) {
                Some(found) => break found,
                None => assert!(walkable.fresh >= 20 && walkable.fresh < 30, "{}", walkable.fresh),
            }
        };
        assert!(frames > 3, "{frames} frames");
        assert_eq!(slowly, Some(way.clone()));
        walkable.fresh = 0;
        assert_eq!(Search::new(from, to).go(&walled, &mut walkable, 1), Some(Some(way.clone())));
        assert_eq!(walkable.fresh, 0);
        // A bed in the room is nowhere to stand: neither in it nor on it.
        let bedded = ground(20.0, &[(Vec3::new(4.0, 0.0, -1.0), Vec3::new(6.3, 0.95, 3.5))]);
        assert_eq!(stand_at(&bedded, Vec3::new(5.0, 0.0, 1.0), 0.0), None);
        assert_eq!(stand_at(&bedded, Vec3::new(7.2, 0.0, 1.0), 0.0), Some(Vec3::new(7.2, 0.0, 1.0)));
        // Where a body can stand, and beside a place to rest.
        assert_eq!(stand_at(&walled, Vec3::new(3.0, 0.4, 0.0), 0.0), Some(Vec3::new(3.0, 0.0, 0.0)));
        assert_eq!(stand_at(&walled, Vec3::new(0.0, 0.0, 0.0), 0.0), None);
        let by = beside(&walled, Vec3::new(0.3, 0.9, 0.0), 0.0).expect("somewhere beside it");
        assert!(by.xz().distance(Vec2::new(0.3, 0.0)) <= 0.75 && by.x > 0.5, "{by}");
        // Not from the far side of a wall, though that is nearer.
        let behind = [(Vec3::new(2.0, 0.0, -3.0), Vec3::new(2.3, 2.5, 3.0))];
        let by = beside(&ground(20.0, &behind), Vec3::new(2.9, 0.9, 0.0), 0.0).expect("somewhere on its own side");
        assert!(by.x > 2.3, "{by}");
        // A bed is wider than a step: beside it all the same, within reach.
        let by = beside(&bedded, Vec3::new(5.15, 0.95, 2.2), 0.0).expect("somewhere beside the bed");
        assert!(by.xz().distance(Vec2::new(5.15, 2.2)) <= REACH + 1e-3, "{by}");
    }

    #[test]
    fn between_plots_the_way_is_out_by_one_gate_round_the_ring_and_in_by_the_other() {
        let plot = |middle: Vec2, front: Vec2| Plot {
            middle,
            ground: 0.27,
            corners: [middle; 4].map(|corner| corner + Vec2::splat(12.0)),
            ring: 0,
            front,
            street: true,
            sign: middle,
        };
        // Hajun's plots 2 and 6, round the pond from each other.
        let all = vec![plot(Vec2::new(25.0, 90.0), Vec2::new(0.0, -1.0)), plot(Vec2::new(90.0, 25.0), Vec2::new(-1.0, 0.0))];
        let plots = Plots { all, of_seat: vec![0, 0, 0, 0, 0, 1] };
        let island = ground(200.0, &[]);
        let there = Plan { what: What::Lawn, plot: 5, throw: 0 };
        let to = Vec3::new(88.0, 0.0, 27.0);
        // On the plot already: only there.
        assert_eq!(stages(&plots, Vec3::new(92.0, 0.0, 20.0), &there, to, &island), vec![to]);
        // From plot 2: its gate, the ring, plot 6's gate, and there; the
        // last first.
        let from = Vec3::new(25.0, 0.0, 90.0);
        let by = stages(&plots, from, &there, to, &island);
        assert_eq!(by.first(), Some(&to));
        let gates = (Vec2::new(25.0, 76.0), Vec2::new(76.0, 25.0));
        assert!(by.last().unwrap().xz().distance(gates.0) < 1e-3, "{by:?}");
        assert!(by[1].xz().distance(gates.1) < 1e-3, "{by:?}");
        let ring: Vec<Vec2> = by[2..by.len() - 1].iter().map(|at| at.xz()).collect();
        assert!(ring.len() >= 4 && ring.iter().all(|at| (at.length() - RING).abs() < 1e-2), "{ring:?}");
        // The shorter way round: never to the far side of the pond.
        assert!(ring.iter().all(|at| at.x > 0.0 && at.y > 0.0), "{ring:?}");
        // No two points further apart than a way is soon found over.
        for pair in by.windows(2) {
            assert!(pair[0].distance(pair[1]) < 20.0, "{pair:?}");
        }
    }
}
