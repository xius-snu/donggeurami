//! Land (the game's `land`, since 2026-11-20): the town's 48 plots, whose
//! each is, and what fills the ones nobody who is here has.
//!
//! **The plots** are the planes of the material `plot` in the town's map,
//! which the game reads as the map spawns; the server has no map, and goes
//! by their middles as they are written here ([`HOMES`]), **in the order
//! Hajun numbered them** (`plotorder.png`, 2026-11-23: "i numbered the
//! plots in priority order so 1 is priority"): the twelve round the pond
//! first, then the next ring, then the outer one, each number a quarter of
//! a turn on from the one before. A plot's number here is that number less
//! one, and **a seat of the town is a plot**: the seat the server gives
//! you ([`Tell::Arrive`](crate::Tell::Arrive)) is the number of your plot,
//! and you arrive in the middle of it, facing the fountain (Hajun,
//! 2026-11-20: "u need to spawn on your plot in the center, facing toward
//! the fountain").
//!
//! **All 48 are always filled** (Hajun, 2026-11-23: "i want to change it so
//! that at all times, there are 48 plots filled"), which is what an
//! [`Estate`] keeps for one channel of the town:
//!
//! * **Whoever is in the channel has one of the first sixteen**
//!   ([`FRONT`]): the lowest numbered that nobody of the channel has, and
//!   that nobody is standing on ("unless there is a real player standing
//!   on that plot, then replace the next number plot on the priority");
//!   or, if a plot among them is filled with their own land, with someone
//!   of the computer's walking it under their name, that one ("if the real
//!   player joins and there is an npc of them in that channel, then that
//!   real person should just take that npc's plot location"). **It is
//!   theirs for as long as they are in the channel**, wherever they go
//!   from it: into a game ("when u go to minigame, your house should
//!   remain there and u should count as someone who is on that channel"),
//!   and for [`GRACE`] seconds after their connection has gone, a phone's
//!   going whenever its app is put away.
//! * **The rest are filled with the land of people who are not here**,
//!   each as it stood when it was taken up ([`Lot`]): real people's first,
//!   the worthiest first, by what stands on it at the shop's prices
//!   ([`worth`]: "unempty plots first in order of higher land valuation
//!   (total sum or placed asset's buy price) calculated at the moment of
//!   first real user in the channel and remains fixed as long as 1 real
//!   player is online in that channel"), and then the made-up ones
//!   ([`crate::premade`]: "if there are still unfilled plots, fill with
//!   fake plots that are premade"), each plot its own.
//! * **Someone of the computer's walks each of the first sixteen that
//!   nobody of the channel has** ([`Holder::Minded`]), under its owner's
//!   name and in their look ("if < 16 players. for example, 3 players ->
//!   the 13 next plots will have an AI NPC spawn in each plot with that
//!   plot owner's name"): every device walks its own (the game's
//!   `resident`), the server only says which plots.
//!
//! A filling stays where it is put: someone coming into the channel takes
//! one plot, and what filled it goes to the first plot a made-up one
//! fills, so that real people's land is always shown before any made-up
//! one, and one thing moves.

use bevy::math::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::TOWN_SEATS;
use crate::bag::{Paint, Standing, thing};

/// How many plots the town has.
pub const PLOTS: usize = 48;

/// How many of them, the first by Hajun's numbers, are for whoever is in a
/// channel of the town: one each, a channel holding as many people
/// ([`TOWN_SEATS`]). Each of them that nobody of the channel has is walked
/// by someone of the computer's.
pub const FRONT: usize = TOWN_SEATS;

/// The middle of each plot across the ground (x and z), in metres, in
/// Hajun's order (`plotorder.png`): his plot 1 is the first. His picture is
/// Blender's top view, its +X to the right and its +Y up (read off the
/// navigation gizmo in its corner, whose ball toward the map is the bare
/// ring Blender draws for -X in a view down an axis), and a point of the
/// map (x, y) in Blender is (-x, y) across the game's ground: his plot 1,
/// right of the pond and just over the street that runs right from it, is
/// at (-90, 25) here. A test of the game's holds these to the map: after
/// an export of the town's map that moves a plot, they have to be written
/// again, and the server deployed.
pub const HOMES: [[f32; 2]; PLOTS] = [
    // 1 to 12: round the pond.
    [-90.0, 25.0],
    [25.0, 90.0],
    [90.0, -25.0],
    [-25.0, -90.0],
    [-25.0, 90.0],
    [90.0, 25.0],
    [25.0, -90.0],
    [-90.0, -25.0],
    [-66.0, 66.0],
    [66.0, 66.0],
    [66.0, -66.0],
    [-66.0, -66.0],
    // 13 to 28: the ring behind them.
    [-125.0, 25.0],
    [25.0, 125.0],
    [125.0, -25.0],
    [-25.0, -125.0],
    [-25.0, 125.0],
    [125.0, 25.0],
    [25.0, -125.0],
    [-125.0, -25.0],
    [-107.385, 64.895],
    [64.895, 107.385],
    [107.385, -64.895],
    [-64.895, -107.385],
    [-64.895, 107.385],
    [107.385, 64.895],
    [64.895, -107.385],
    [-107.385, -64.895],
    // 29 to 48: the outer ring.
    [-160.0, 25.0],
    [25.0, 160.0],
    [160.0, -25.0],
    [-25.0, -160.0],
    [-25.0, 160.0],
    [160.0, 25.0],
    [25.0, -160.0],
    [-160.0, -25.0],
    [-145.5, 80.5],
    [80.5, 145.5],
    [145.5, -80.5],
    [-80.5, -145.5],
    [-80.5, 145.5],
    [145.5, 80.5],
    [80.5, -145.5],
    [-145.5, -80.5],
    [-110.5, 110.5],
    [110.5, 110.5],
    [110.5, -110.5],
    [-110.5, -110.5],
];

/// How near the middle of a plot someone stands who has only just
/// arrived on one, and not walked off it yet, in metres.
pub const JUST_ARRIVED: f32 = 2.0;

/// How far from the middle of a plot someone may stand and be on it, as
/// far as taking it from under them goes, in metres: its corner, the plot
/// being 24 m square, however it is turned. No two plots are nearer than
/// twice this.
pub const ON_A_PLOT: f32 = 17.0;

/// How long a plot is kept for someone whose connection has gone, as they
/// left it, with nobody of the computer's on it, in seconds: a phone's
/// goes whenever its app is put away, and whoever is back within this
/// finds their land where it was, and so does everyone else. As long as
/// that, a channel nobody is left in stays as it was too. Mine.
pub const GRACE: f64 = 300.0;

/// Where whoever has the plot numbered `plot` stands on arriving in the
/// town, and which way they face: in the middle of it, facing the
/// fountain, which is the middle of the town. `None` past the town's plots.
pub fn arrival(plot: usize) -> Option<(Vec3, Quat)> {
    let [x, z] = *HOMES.get(plot)?;
    // A turn of this much about the upright points -Z at the middle
    // from where it puts +Z, as [`crate::arrival`] has it.
    Some((Vec3::new(x, 0.0, z), Quat::from_rotation_y(x.atan2(z))))
}

/// Whether someone standing `at`, across the ground, stands where a plot's
/// holder arrives, any plot's: a device that has only just opened, which
/// guessed at its plot before the server gave it one.
pub fn just_arrived(at: Vec2) -> bool {
    HOMES.iter().any(|&home| Vec2::from(home).distance(at) <= JUST_ARRIVED)
}

/// The plot someone standing `at`, across the ground, is on, if any
/// ([`ON_A_PLOT`]).
pub fn plot_at(at: Vec2) -> Option<usize> {
    HOMES.iter().position(|&home| Vec2::from(home).distance(at) <= ON_A_PLOT)
}

/// What a plot is worth: what stands on it, each thing at what the shop
/// sells one for (Hajun, 2026-11-23: "land valuation (total sum or placed
/// asset's buy price)"). Nothing, for nothing standing.
pub fn worth(stands: &[Standing]) -> u32 {
    stands
        .iter()
        .filter_map(|stood| thing(&stood.kind))
        .fold(0u32, |sum, thing| sum.saturating_add(thing.price))
}

/// The accounts the made-up lots go under ([`crate::premade`]): this and
/// the 47 after it, the plot's number added. No real account is one of
/// them: the database counts accounts up from 1, and a server run without
/// one takes the hash of a device's own bytes.
pub const PREMADE: u64 = 0x5052_454D_4144_0000;

/// Whether `who` is one of the made-up owners.
pub fn is_premade(who: u64) -> bool {
    (PREMADE..PREMADE + PLOTS as u64).contains(&who)
}

/// Someone's land as it stood when it was taken up to fill a plot: whose,
/// what they go by and how they look, for whoever of the computer's walks
/// it, and what stands on it, with what of that is painted.
#[derive(Clone, PartialEq, Debug)]
pub struct Lot {
    pub who: u64,
    pub name: String,
    pub look: String,
    pub stands: Vec<Standing>,
    pub paints: Vec<Paint>,
}

impl Lot {
    /// What it is worth ([`worth`]).
    pub fn worth(&self) -> u32 {
        worth(&self.stands)
    }

    /// Whether it is one of the made-up ones.
    pub fn premade(&self) -> bool {
        is_premade(self.who)
    }
}

/// Who is on a plot, as a device is told of it ([`Deed`]).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Holder {
    /// Its owner, who is in your channel of the town and in the town now.
    Here,
    /// Its owner, who is in your channel and away from the town: in a
    /// game, at home, or gone a moment ago ([`GRACE`]). Nobody walks it.
    Away,
    /// Nobody who is here: someone of the computer's walks it under its
    /// owner's name, each device its own (the game's `resident`).
    Minded,
    /// Nobody at all: what stands on it, and its sign.
    Nobody,
}

/// One plot of your channel of the town, as the server says whose it is
/// ([`Tell::Plots`](crate::Tell::Plots)): its number ([`HOMES`]); whose, by
/// their [`Member::id`](crate::Member::id), or by a made-up owner's
/// ([`PREMADE`]); the name on its sign, and the look of whoever walks it;
/// and who is on it. What stands on it comes as what stands on anyone's
/// plot does ([`bag::Tell::Theirs`](crate::bag::Tell::Theirs), by `who`).
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Deed {
    pub plot: u8,
    pub who: u64,
    pub name: String,
    pub look: String,
    pub holder: Holder,
}

/// The plots this device fills by itself, with no server to say: every one
/// but `mine`, each with its own made-up lot ([`crate::premade::lot`]),
/// and someone walking each of the first sixteen.
pub fn offline_deeds(mine: usize) -> Vec<Deed> {
    (0..PLOTS)
        .filter(|&plot| plot != mine)
        .map(|plot| {
            let lot = crate::premade::lot(plot);
            Deed {
                plot: plot as u8,
                who: lot.who,
                name: lot.name,
                look: lot.look,
                holder: if plot < FRONT { Holder::Minded } else { Holder::Nobody },
            }
        })
        .collect()
}

/// What is on one plot of a channel.
#[derive(Clone, PartialEq, Debug)]
enum Slot {
    /// Someone's who is in the channel: in the town, or `away` from it in
    /// a game or at home.
    Held { who: u64, away: bool },
    /// Someone's whose connection went at `since`, as they left it.
    Left { lot: Lot, since: f64 },
    /// Nobody's who is in the channel: filled with this.
    Filled(Lot),
}

/// The plots of one channel of the town: who of the channel has which, and
/// what fills the rest (the module's own account says how). It knows
/// nothing of the world: whoever keeps one says who comes and goes, and
/// when ([`Estate::join`], [`Estate::away`], [`Estate::leave`],
/// [`Estate::tick`]), and asks whose each plot is ([`Estate::deeds`]) and
/// what stands on those nobody here has ([`Estate::lot`]).
#[derive(Clone, PartialEq, Debug)]
pub struct Estate {
    slots: Vec<Slot>,
    /// Real people's land that fills no plot just now, the worthiest
    /// first.
    spare: Vec<Lot>,
    /// The plots that have been filled with something else since that was
    /// last asked ([`Estate::take_news`]).
    news: Vec<usize>,
}

impl Estate {
    /// A channel as `first` comes into it, with nobody else in it: theirs
    /// is the first plot, and the rest are filled with `lots`, real
    /// people's land that has anything standing on it, the worthiest
    /// first, and then with the made-up ones. Worked out now, and as it is
    /// for as long as anyone is in the channel.
    pub fn open(first: u64, lots: Vec<Lot>) -> Self {
        let mut spare: Vec<Lot> = Vec::new();
        for lot in lots {
            let known = spare.iter().any(|other| other.who == lot.who);
            if lot.who != first && !lot.premade() && !lot.stands.is_empty() && !known {
                spare.push(lot);
            }
        }
        spare.sort_by_key(rank);
        let mut filling = spare.into_iter();
        let slots = (0..PLOTS)
            .map(|plot| match plot {
                0 => Slot::Held { who: first, away: false },
                _ => Slot::Filled(filling.next().unwrap_or_else(|| crate::premade::lot(plot))),
            })
            .collect();
        Self {
            slots,
            spare: filling.collect(),
            news: Vec::new(),
        }
    }

    /// The plot `who` has, in the channel or gone from it a moment ago.
    pub fn plot_of(&self, who: u64) -> Option<usize> {
        self.slots.iter().position(|slot| match slot {
            Slot::Held { who: holder, .. } => *holder == who,
            Slot::Left { lot, .. } => lot.who == who,
            Slot::Filled(_) => false,
        })
    }

    /// How many are in the channel, those gone a moment ago among them.
    pub fn count(&self) -> usize {
        self.slots.iter().filter(|slot| !matches!(slot, Slot::Filled(_))).count()
    }

    /// How many more it has room for.
    pub fn room(&self) -> usize {
        FRONT.saturating_sub(self.count())
    }

    /// Whether nobody is left in it, nor anyone it is kept for: a channel
    /// to be worked out afresh by whoever comes into it next.
    pub fn deserted(&self) -> bool {
        self.count() == 0
    }

    /// `who` comes into the channel, or back to it: the plot they have.
    /// The one they had, if they never left, or left a moment ago; else
    /// the one of the first sixteen their own land fills, if it does; else
    /// the first of them that nobody of the channel has and nobody stands
    /// on (`busy`), or, with someone on every one, the first nobody has.
    /// What filled it goes to the first plot a made-up lot fills. `None`
    /// with the channel full.
    pub fn join(&mut self, who: u64, busy: impl Fn(usize) -> bool) -> Option<usize> {
        if let Some(plot) = self.plot_of(who) {
            self.slots[plot] = Slot::Held { who, away: false };
            return Some(plot);
        }
        if self.room() == 0 {
            return None;
        }
        let theirs = |slot: &Slot| matches!(slot, Slot::Filled(lot) if lot.who == who);
        let own = self.slots.iter().position(theirs);
        let filled = |plot: &usize| matches!(self.slots[*plot], Slot::Filled(_));
        let plot = match own {
            Some(plot) if plot < FRONT => plot,
            _ => (0..FRONT)
                .filter(filled)
                .find(|&plot| !busy(plot))
                .or_else(|| (0..FRONT).find(filled))?,
        };
        // Their own land fills nothing while they are here themselves.
        self.spare.retain(|lot| lot.who != who);
        let mut bare = Vec::new();
        if let Some(further) = own.filter(|&further| further != plot) {
            bare.push(further);
        }
        let was = std::mem::replace(&mut self.slots[plot], Slot::Held { who, away: false });
        if let Slot::Filled(lot) = was
            && lot.who != who
        {
            self.set_aside(lot);
        }
        self.settle(bare);
        Some(plot)
    }

    /// `who`, of the channel, has gone from the town into a game or home
    /// (`away`), or is back in it: their plot is theirs all the while.
    /// Whether that changed anything.
    pub fn away(&mut self, who: u64, away: bool) -> bool {
        for slot in &mut self.slots {
            if let Slot::Held { who: holder, away: was } = slot
                && *holder == who
                && *was != away
            {
                *was = away;
                return true;
            }
        }
        false
    }

    /// `who` is gone from the channel at `now`, their land as they leave
    /// it being `lot`. With `kept`, their connection has gone: the plot is
    /// kept as it is for [`GRACE`] ([`Estate::tick`]). Without, they are
    /// in another channel now: the plot is filled with their land at once,
    /// if anything stands on it, and with something else if not. The plot
    /// it was.
    pub fn leave(&mut self, who: u64, lot: Option<Lot>, now: f64, kept: bool) -> Option<usize> {
        let plot = self.slots.iter().position(|slot| matches!(slot, Slot::Held { who: holder, .. } if *holder == who))?;
        match lot {
            Some(lot) if kept => self.slots[plot] = Slot::Left { lot, since: now },
            Some(lot) if !lot.stands.is_empty() => {
                self.slots[plot] = Slot::Filled(lot);
                self.news.push(plot);
            }
            _ => self.settle(vec![plot]),
        }
        Some(plot)
    }

    /// Lets go, at `now`, of every plot kept longer than [`GRACE`] for
    /// someone whose connection went: filled with their land from here on,
    /// with someone of the computer's on it, or, with nothing standing on
    /// it, with something else. Whether any was.
    pub fn tick(&mut self, now: f64) -> bool {
        let mut bare = Vec::new();
        let mut changed = false;
        for (plot, slot) in self.slots.iter_mut().enumerate() {
            let Slot::Left { lot, since } = slot else {
                continue;
            };
            if now - *since <= GRACE {
                continue;
            }
            changed = true;
            if lot.stands.is_empty() {
                bare.push(plot);
            } else {
                *slot = Slot::Filled(lot.clone());
            }
        }
        // Still theirs to look at until something else is put there.
        for &plot in &bare {
            self.slots[plot] = Slot::Filled(crate::premade::lot(plot));
        }
        self.settle(bare);
        changed
    }

    /// The account `who` is no more: its land fills nothing from here on.
    pub fn forget(&mut self, who: u64) {
        self.spare.retain(|lot| lot.who != who);
        let bare: Vec<usize> = (0..self.slots.len())
            .filter(|&plot| match &self.slots[plot] {
                Slot::Left { lot, .. } | Slot::Filled(lot) => lot.who == who,
                Slot::Held { .. } => false,
            })
            .collect();
        for &plot in &bare {
            self.slots[plot] = Slot::Filled(crate::premade::lot(plot));
        }
        self.settle(bare);
    }

    /// Puts `lot`, real land that fills nothing now, among the spare, the
    /// worthiest first.
    fn set_aside(&mut self, lot: Lot) {
        if lot.premade() || self.spare.iter().any(|other| other.who == lot.who) {
            return;
        }
        let at = self.spare.partition_point(|other| rank(other) <= rank(&lot));
        self.spare.insert(at, lot);
    }

    /// Fills each of the `bare` plots, with the worthiest spare land or,
    /// with none, the plot's own made-up lot; then has real people's land
    /// shown before any made-up one, the worthiest spare going to the
    /// first plot a made-up one fills; and, with no made-up one left to
    /// give way, has the worthiest shown, a spare one taking the place of
    /// the least of those that fill a plot if it is worth more.
    fn settle(&mut self, mut bare: Vec<usize>) {
        bare.sort_unstable();
        for plot in bare {
            let lot = if self.spare.is_empty() { crate::premade::lot(plot) } else { self.spare.remove(0) };
            self.slots[plot] = Slot::Filled(lot);
            self.news.push(plot);
        }
        while !self.spare.is_empty() {
            let made_up = self.slots.iter().position(|slot| matches!(slot, Slot::Filled(lot) if lot.premade()));
            let Some(plot) = made_up else {
                break;
            };
            self.slots[plot] = Slot::Filled(self.spare.remove(0));
            self.news.push(plot);
        }
        loop {
            let least = self
                .slots
                .iter()
                .enumerate()
                .filter_map(|(plot, slot)| match slot {
                    Slot::Filled(lot) => Some((plot, lot.worth())),
                    _ => None,
                })
                .min_by_key(|&(plot, worth)| (worth, std::cmp::Reverse(plot)));
            let (Some(best), Some((plot, worth))) = (self.spare.first(), least) else {
                break;
            };
            if best.worth() <= worth {
                break;
            }
            let was = std::mem::replace(&mut self.slots[plot], Slot::Filled(self.spare.remove(0)));
            if let Slot::Filled(lot) = was {
                self.set_aside(lot);
            }
            self.news.push(plot);
        }
        self.news.sort_unstable();
        self.news.dedup();
    }

    /// The plots filled with something else since this was last asked,
    /// each once, in order: whoever is in the channel is to be told what
    /// stands on each now ([`Estate::lot`]).
    pub fn take_news(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.news)
    }

    /// Whether any plot has been filled with something else since that was
    /// last asked.
    pub fn has_news(&self) -> bool {
        !self.news.is_empty()
    }

    /// The land that fills `plot`, or is kept on it for someone who went a
    /// moment ago: none on a plot someone of the channel has, whose own
    /// bag says what stands there.
    pub fn lot(&self, plot: usize) -> Option<&Lot> {
        match self.slots.get(plot)? {
            Slot::Left { lot, .. } | Slot::Filled(lot) => Some(lot),
            Slot::Held { .. } => None,
        }
    }

    /// Everyone of the channel who has a plot and has not gone: each
    /// plot, whose, and whether they are away from the town.
    pub fn held(&self) -> impl Iterator<Item = (usize, u64, bool)> + '_ {
        self.slots.iter().enumerate().filter_map(|(plot, slot)| match slot {
            Slot::Held { who, away } => Some((plot, *who, *away)),
            _ => None,
        })
    }

    /// Whose every plot is, as a device is told ([`Deed`]), given the name
    /// and the look of someone of the channel (`known`: whoever is not is
    /// left out until they are).
    pub fn deeds(&self, known: impl Fn(u64) -> Option<(String, String)>) -> Vec<Deed> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(plot, slot)| {
                let (who, (name, look), holder) = match slot {
                    Slot::Held { who, away } => (*who, known(*who)?, if *away { Holder::Away } else { Holder::Here }),
                    Slot::Left { lot, .. } => (lot.who, (lot.name.clone(), lot.look.clone()), Holder::Away),
                    Slot::Filled(lot) => {
                        let holder = if plot < FRONT { Holder::Minded } else { Holder::Nobody };
                        (lot.who, (lot.name.clone(), lot.look.clone()), holder)
                    }
                };
                Some(Deed {
                    plot: plot as u8,
                    who,
                    name,
                    look,
                    holder,
                })
            })
            .collect()
    }
}

/// What land is ranked by: the worthiest first, and of two worth the
/// same, the older account.
fn rank(lot: &Lot) -> (std::cmp::Reverse<u32>, u64) {
    (std::cmp::Reverse(lot.worth()), lot.who)
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3Swizzles;

    use super::*;
    use crate::At;

    /// Real land of `who` with `chairs` cafe chairs on it, fifty coins
    /// each.
    fn land(who: u64, chairs: usize) -> Lot {
        Lot {
            who,
            name: format!("Person {who}"),
            look: "girl".into(),
            stands: (0..chairs)
                .map(|n| Standing {
                    id: n as u32 + 1,
                    kind: "cafechair".into(),
                    at: At::Ground {
                        x: n as f32,
                        y: 0.0,
                        z: 0.0,
                        yaw: 0.0,
                    },
                })
                .collect(),
            paints: Vec::new(),
        }
    }

    fn whose(estate: &Estate) -> Vec<u64> {
        estate.deeds(|who| Some((format!("Person {who}"), "boy".into()))).iter().map(|deed| deed.who).collect()
    }

    fn nobody_standing(_: usize) -> bool {
        false
    }

    #[test]
    fn the_plots_are_in_hajun_s_order_each_a_quarter_turn_on_from_the_one_before() {
        // Forty-eight, no two the same, none nearer another than someone
        // standing on one could be on both.
        for (a, home) in HOMES.iter().enumerate() {
            for other in &HOMES[a + 1..] {
                assert!(Vec2::from(*home).distance(Vec2::from(*other)) >= 2.0 * ON_A_PLOT, "{home:?} and {other:?}");
            }
        }
        // His numbers go round in fours: 1, 2, 3, 4 are one plot turned a
        // quarter each time, and so on.
        for four in HOMES.chunks(4) {
            for pair in four.windows(2) {
                assert!(Vec2::from(pair[1]).distance(Vec2::new(pair[0][1], -pair[0][0])) < 1e-3, "{pair:?}");
            }
        }
        // The twelve round the pond first, then the next ring, then the
        // outer one.
        let out = |plot: usize| Vec2::from(HOMES[plot]).length();
        assert!((0..12).all(|plot| out(plot) < 95.0));
        assert!((12..28).all(|plot| (120.0..130.0).contains(&out(plot))));
        assert!((28..48).all(|plot| out(plot) > 150.0));
        // His plot 1, and where its holder arrives: in its middle, facing
        // the fountain.
        assert_eq!(HOMES[0], [-90.0, 25.0]);
        let (at, turn) = arrival(0).unwrap();
        assert_eq!(at, Vec3::new(-90.0, 0.0, 25.0));
        assert!((turn * Vec3::NEG_Z).xz().distance(-Vec2::new(-90.0, 25.0).normalize()) < 1e-4);
        assert!(arrival(PLOTS).is_none());
        assert!(just_arrived(Vec2::new(-89.0, 25.5)) && !just_arrived(Vec2::new(-80.0, 25.0)));
        assert_eq!(plot_at(Vec2::new(-80.0, 30.0)), Some(0));
        assert_eq!(plot_at(Vec2::new(25.0, 100.0)), Some(1));
        assert_eq!(plot_at(Vec2::ZERO), None);
    }

    #[test]
    fn land_is_worth_what_stands_on_it_at_the_shop_s_prices() {
        assert_eq!(worth(&[]), 0);
        assert_eq!(land(1, 3).worth(), 150);
        let mut lamp = land(2, 1);
        lamp.stands[0].kind = crate::bag::LAMP.into();
        assert_eq!(lamp.worth(), 2_500);
        // What there is no such thing as is worth nothing.
        lamp.stands[0].kind = "a_castle".into();
        assert_eq!(lamp.worth(), 0);
    }

    #[test]
    fn a_channel_opens_with_its_first_on_plot_one_and_the_worthiest_land_after() {
        // Land with nothing on it fills nothing, nor the newcomer's own.
        let lots = vec![land(5, 1), land(6, 4), land(7, 0), land(8, 2), land(9, 4), land(1, 30)];
        let estate = Estate::open(1, lots);
        let whose = whose(&estate);
        assert_eq!(whose.len(), PLOTS);
        // The worthiest first, the older account of two worth the same.
        assert_eq!(&whose[..5], &[1, 6, 9, 8, 5]);
        // Then the made-up ones, each plot its own.
        for (plot, who) in whose.iter().enumerate().skip(5) {
            assert_eq!(*who, PREMADE + plot as u64);
        }
        assert_eq!((estate.count(), estate.room()), (1, FRONT - 1));
        // Someone walks each of the first sixteen that nobody here has, and
        // none of the rest.
        let deeds = estate.deeds(|_| Some(("Hajun".into(), "boy".into())));
        assert_eq!(deeds[0].holder, Holder::Here);
        assert!(deeds[1..FRONT].iter().all(|deed| deed.holder == Holder::Minded));
        assert!(deeds[FRONT..].iter().all(|deed| deed.holder == Holder::Nobody));
        assert_eq!((deeds[1].name.as_str(), deeds[1].look.as_str()), ("Person 6", "girl"));
        // Someone of the channel whose name is not known yet is left out.
        assert_eq!(estate.deeds(|_| None).len(), PLOTS - 1);
        // What stands on a plot nobody here has is the land's own.
        assert_eq!(estate.lot(1).map(Lot::worth), Some(200));
        assert!(estate.lot(0).is_none());
    }

    #[test]
    fn whoever_comes_takes_the_first_plot_nobody_has_and_what_filled_it_moves_once() {
        let mut estate = Estate::open(1, vec![land(6, 4), land(8, 2)]);
        assert_eq!(&whose(&estate)[..4], &[1, 6, 8, PREMADE + 3]);
        // The second: plot two, and the land that filled it goes to the
        // first plot a made-up one fills. Everyone is told of that plot.
        assert_eq!(estate.join(2, nobody_standing), Some(1));
        assert_eq!(&whose(&estate)[..5], &[1, 2, 8, 6, PREMADE + 4]);
        assert_eq!(estate.take_news(), [3]);
        assert!(!estate.has_news());
        // Come again, they have what they had.
        assert_eq!(estate.join(2, nobody_standing), Some(1));
        assert!(estate.take_news().is_empty());
        // With someone standing on the next, the one after it.
        assert_eq!(estate.join(3, |plot| plot == 2), Some(3));
        assert_eq!(&whose(&estate)[..6], &[1, 2, 8, 3, 6, PREMADE + 5]);
        // With someone on every one of them, the first all the same.
        assert_eq!(estate.join(4, |_| true), Some(2));
        assert_eq!(&whose(&estate)[..6], &[1, 2, 4, 3, 6, 8]);
    }

    #[test]
    fn someone_whose_own_land_fills_a_plot_takes_that_plot() {
        // Among the first sixteen, where someone walks it under their name.
        let mut estate = Estate::open(1, vec![land(6, 4), land(8, 2), land(9, 1)]);
        assert_eq!(estate.join(8, nobody_standing), Some(2));
        assert_eq!(&whose(&estate)[..5], &[1, 6, 8, 9, PREMADE + 4]);
        assert!(estate.take_news().is_empty());
        // Further out, nobody walks it: they come to the front like anyone,
        // and the plot it filled is filled with something else.
        let lots: Vec<Lot> = (0..20).map(|n| land(100 + n, 30 - n as usize)).collect();
        let mut estate = Estate::open(1, lots);
        assert_eq!(whose(&estate)[18], 117);
        assert_eq!(estate.join(117, nobody_standing), Some(1));
        let whose = whose(&estate);
        assert_eq!(whose.iter().filter(|who| **who == 117).count(), 1);
        // The land they took the place of went where theirs stood.
        assert_eq!((whose[1], whose[18]), (117, 100));
    }

    #[test]
    fn a_plot_is_its_holder_s_in_a_game_and_for_a_while_after_their_connection_goes() {
        let mut estate = Estate::open(1, vec![land(6, 4)]);
        estate.join(2, nobody_standing);
        estate.take_news();
        let holder = |estate: &Estate, plot: usize| estate.deeds(|_| Some(("x".into(), "boy".into())))[plot].holder;
        // Into a game: theirs, nobody walking it.
        assert!(estate.away(2, true));
        assert!(!estate.away(2, true));
        assert_eq!(holder(&estate, 1), Holder::Away);
        assert_eq!((estate.count(), estate.plot_of(2)), (2, Some(1)));
        // Back: here.
        assert_eq!(estate.join(2, nobody_standing), Some(1));
        assert_eq!(holder(&estate, 1), Holder::Here);
        // Their connection gone: kept as they left it, nobody on it, and
        // counted in the channel.
        assert_eq!(estate.leave(2, Some(land(2, 3)), 100.0, true), Some(1));
        assert_eq!(holder(&estate, 1), Holder::Away);
        assert_eq!((estate.count(), estate.lot(1).map(Lot::worth)), (2, Some(150)));
        assert!(!estate.tick(100.0 + GRACE));
        // Back within that: theirs again, where it was.
        assert_eq!(estate.join(2, nobody_standing), Some(1));
        assert_eq!((holder(&estate, 1), estate.lot(1)), (Holder::Here, None));
        // Gone for longer: their land fills the plot from here on, and
        // someone walks it under their name.
        estate.leave(2, Some(land(2, 3)), 200.0, true);
        assert!(estate.tick(200.1 + GRACE));
        assert_eq!((holder(&estate, 1), estate.count()), (Holder::Minded, 1));
        assert_eq!(whose(&estate)[1], 2);
        // And back after that, they take it where it stands.
        assert_eq!(estate.join(2, |_| true), Some(1));
        // With nothing standing on it, the plot goes to something else.
        estate.leave(2, Some(land(2, 0)), 900.0, true);
        assert!(estate.tick(901.0 + GRACE));
        assert_eq!(whose(&estate)[1], PREMADE + 1);
        assert_eq!(estate.take_news(), [1]);
        // The last of them gone, and the while over, nobody is left.
        estate.leave(1, None, 2_000.0, true);
        assert!(estate.deserted());
    }

    #[test]
    fn someone_gone_to_another_channel_leaves_their_land_filling_the_plot_at_once() {
        let mut estate = Estate::open(1, Vec::new());
        estate.join(2, nobody_standing);
        estate.take_news();
        estate.leave(2, Some(land(2, 2)), 0.0, false);
        assert_eq!((whose(&estate)[1], estate.count()), (2, 1));
        assert_eq!(estate.take_news(), [1]);
        // Whoever comes next has that plot all the same, the first nobody
        // of the channel has, and the land that filled it moves along.
        estate.join(3, nobody_standing);
        assert_eq!((estate.plot_of(3), whose(&estate)[2]), (Some(1), 2));
        // With nothing on it, the plot's own made-up lot.
        estate.leave(3, Some(land(3, 0)), 0.0, false);
        assert_eq!(&whose(&estate)[..3], &[1, PREMADE + 1, 2]);
    }

    #[test]
    fn a_channel_holds_sixteen_and_all_of_them_at_the_front() {
        let mut estate = Estate::open(1, Vec::new());
        for who in 2..=16 {
            assert_eq!(estate.join(who, nobody_standing), Some(who as usize - 1));
        }
        assert_eq!((estate.count(), estate.room()), (FRONT, 0));
        assert_eq!(estate.join(17, nobody_standing), None);
        // Nobody of the computer's walks anywhere.
        let deeds = estate.deeds(|_| Some(("x".into(), "boy".into())));
        assert!(deeds.iter().all(|deed| deed.holder != Holder::Minded));
        // One in a game is still one of the sixteen.
        estate.away(5, true);
        assert_eq!(estate.join(17, nobody_standing), None);
        // One gone to another channel makes room.
        estate.leave(5, None, 0.0, false);
        assert_eq!(estate.join(17, nobody_standing), Some(4));
    }

    #[test]
    fn real_land_is_shown_before_made_up_and_the_worthiest_of_it() {
        // More real land than plots to fill: the least of it is spare, and
        // comes when a plot is free.
        let lots: Vec<Lot> = (0..50).map(|n| land(100 + n, 60 - n as usize)).collect();
        let mut estate = Estate::open(1, lots);
        let shown = whose(&estate);
        assert!(shown.iter().all(|who| !is_premade(*who)));
        assert_eq!((shown[1], shown[47]), (100, 146));
        // Someone comes: what they took the place of, worth more than the
        // least shown, takes its place.
        estate.join(2, nobody_standing);
        let shown = whose(&estate);
        assert_eq!((shown[1], shown[47]), (2, 100));
        assert!(!shown.contains(&146));
        // An account that is no more fills nothing.
        estate.forget(100);
        let shown = whose(&estate);
        assert!(!shown.contains(&100) && shown.contains(&146));
    }

    #[test]
    fn whose_each_plot_is_goes_last_of_what_is_told_and_all_of_them_in_a_couple_of_kilobytes() {
        use crate::Tell;
        use crate::social::Outcome;
        let deeds = Estate::open(1, vec![land(6, 4)]).deeds(|_| Some(("Hajooon".into(), "boy_whitejeans".into())));
        let bytes = postcard::to_allocvec(&Tell::Plots(deeds.clone())).unwrap();
        // After everything an app from before this was told: who rests
        // was the twenty-first, numbered 20.
        assert_eq!(bytes[0], 21);
        assert!(bytes.len() < 2_048, "{} bytes", bytes.len());
        let Tell::Plots(back) = postcard::from_bytes(&bytes).unwrap() else {
            panic!("not the plots");
        };
        assert_eq!(back, deeds);
        // And what a group gathered into one channel is told, the last of
        // its kind.
        assert_eq!(postcard::to_allocvec(&Outcome::Gathered).unwrap(), [17]);
        assert_eq!(postcard::to_allocvec(&[Holder::Here, Holder::Away, Holder::Minded, Holder::Nobody]).unwrap(), [0, 1, 2, 3]);
    }

    #[test]
    fn offline_every_plot_but_yours_has_its_made_up_owner() {
        let deeds = offline_deeds(0);
        assert_eq!(deeds.len(), PLOTS - 1);
        assert!(deeds.iter().all(|deed| deed.plot != 0 && is_premade(deed.who)));
        assert_eq!(deeds.iter().filter(|deed| deed.holder == Holder::Minded).count(), FRONT - 1);
        let elsewhere = offline_deeds(5);
        assert!(elsewhere.iter().any(|deed| deed.plot == 0) && elsewhere.iter().all(|deed| deed.plot != 5));
        // No two of them go by one name.
        let mut names: Vec<&str> = deeds.iter().map(|deed| deed.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), PLOTS - 1);
    }
}
