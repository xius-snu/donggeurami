//! Bridge Wars' map as the computer's players know it (since 2026-11-12;
//! Hajun: "the other bots dont do anything other than stay in their base. i
//! need the bots to be intelligent enough ... maybe treat islands as graph
//! nodes and every bridge as an edge and bots pathfind over the graph and
//! when no path exist, they buy meters for the straight line distance and
//! bridge it").
//!
//! The server has no map, and until now kept each of its bots to the disc
//! of its island. This is the map it was missing, and the same one for a
//! game played offline:
//!
//! * [`Grid`]: where a body can stand and walk on Hajun's `pvptownmap.glb`,
//!   half a metre to a square, baked out of the map by the game's own
//!   collision (`nav_bake`, a test in the game, which writes `arena.nav`
//!   and fails when the map has changed since) and built into both
//!   binaries. Each square has its floor's height, whether a body fits
//!   there and can walk to it ([`Grid::clear`]), and which piece of land it
//!   is ([`Region`]): one of the four teams' islands, one of the four
//!   sapphire islands, or the middle of the map. The map is a star: every
//!   island reaches every other through the middle, and through nothing
//!   else until someone builds it.
//! * [`Nav`]: that, with every bridge laid over it as its rungs come
//!   ([`Nav::hear`]): the squares along a plank's middle are walkable, so
//!   one search finds a way over land and planks alike ([`Nav::path`]), and
//!   which pieces of land have been joined is known ([`Nav::joined`]).
//! * [`Span`]: the straight line of rungs across one of the star's eight
//!   gaps ([`Grid::span`]), which is what a bot lays when there is no way.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Arc, LazyLock};

use bevy::math::Vec2;

use crate::{ARENA_ISLANDS, SAPPHIRE_PADS};

/// How far across a square of the grid is, in metres.
pub const CELL: f32 = 0.5;
/// Half of how wide a plank is, in metres: Hajun's `bridge` object's
/// measure (2.726 m across), which the game's `bridge` builds by.
pub const PLANK_HALF_WIDTH: f32 = 1.363;
/// How far apart a bridge's rungs are fixed, in metres: half a plank's
/// width, the boards of a dock.
pub const PLANK_SPACING: f32 = PLANK_HALF_WIDTH;
/// How high a plank's top is, in metres: a hair under the islands' tops.
pub const PLANK_TOP: f32 = 0.24;
/// How far out from its middle a body reaches, across the ground, in
/// metres: the game's `PLAYER_RADIUS`.
pub const BODY: f32 = 0.45;
/// How far either side of a plank's middle line a body's middle may walk:
/// the plank's half width less the body's.
const DECK_HALF: f32 = PLANK_HALF_WIDTH - BODY;
/// The longest a plank can be, in metres, to be taken for one: a fork's
/// first plank at its widest slant is under three. A rung further than
/// this from the one before it is somebody's device gone wrong, and no
/// bot walks out along it.
const MOST_PLANK: f32 = 6.0;
/// The most rungs a chain is kept to, and the most chains: the game's own
/// limits, and room.
const MOST_RUNGS: usize = 600;
const MOST_CHAINS: usize = 400;
/// How far from where it was asked for a square to walk on is looked for,
/// in metres: someone standing at the very edge of an island, on a pad's
/// corner or against a stand is beside one.
pub const BESIDE: f32 = 2.5;
/// How far inside each shore a span's first and last rungs are, in
/// metres: over the land, where the game's `bridge` leaves a root's
/// corners as they are, hidden under it, and takes an end for landed.
const SPAN_INSET: f32 = 0.5;

/// A piece of the map's land, apart from every other until a bridge joins
/// them: a team's island, by team; one of the four smaller islands on the
/// diagonals, by its pad's number ([`SAPPHIRE_PADS`]); the middle of the
/// map; and the pond in the middle of that, whose floor is standing room
/// nobody can walk into or out of, its rim being higher than a step.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Region {
    Team(u8),
    Sapphire(u8),
    Middle,
    Pocket,
}

impl Region {
    /// Every piece of land a body can walk about on.
    pub const LANDS: [Region; 9] = [
        Region::Team(0),
        Region::Team(1),
        Region::Team(2),
        Region::Team(3),
        Region::Sapphire(0),
        Region::Sapphire(1),
        Region::Sapphire(2),
        Region::Sapphire(3),
        Region::Middle,
    ];

    /// The number a square of the grid keeps it as.
    pub fn code(self) -> u8 {
        match self {
            Region::Team(team) => 1 + team,
            Region::Sapphire(island) => 5 + island,
            Region::Middle => 9,
            Region::Pocket => 15,
        }
    }

    /// Which of [`Region::LANDS`] it is, if it is one.
    fn slot(self) -> Option<usize> {
        match self {
            Region::Team(team) => Some(usize::from(team) % 4),
            Region::Sapphire(island) => Some(4 + usize::from(island) % 4),
            Region::Middle => Some(8),
            Region::Pocket => None,
        }
    }

    fn of(code: u8) -> Option<Region> {
        match code {
            1..=4 => Some(Region::Team(code - 1)),
            5..=8 => Some(Region::Sapphire(code - 5)),
            9 => Some(Region::Middle),
            15 => Some(Region::Pocket),
            _ => None,
        }
    }

    /// Its middle, across the ground.
    pub fn centre(self) -> Vec2 {
        match self {
            Region::Team(team) => ARENA_ISLANDS[usize::from(team) % ARENA_ISLANDS.len()],
            Region::Sapphire(island) => SAPPHIRE_PADS[usize::from(island) % SAPPHIRE_PADS.len()],
            Region::Middle | Region::Pocket => Vec2::ZERO,
        }
    }
}

/// No floor: a square over the void.
const NO_FLOOR: u8 = 255;
/// A floor's height is kept in whole centimetres over this.
const FLOOR_BASE: f32 = -0.5;
/// A body fits on the square and can walk to it.
const CLEAR: u8 = 1;
/// The square has a floor: it holds a body knocked onto it.
const GROUND: u8 = 2;
/// What a baked grid begins with.
const MAGIC: &[u8; 8] = b"RTNAV1\0\0";

/// Where a body can stand and walk on the map, in squares [`CELL`] across:
/// the map as it is with nothing built on it.
#[derive(Clone, PartialEq, Debug)]
pub struct Grid {
    width: usize,
    height: usize,
    /// The corner of the grid with the lowest x and z.
    corner: Vec2,
    /// Each square's floor, in centimetres over [`FLOOR_BASE`].
    floor: Vec<u8>,
    /// Each square's [`CLEAR`] and [`GROUND`], and in its upper half its
    /// region's code.
    flags: Vec<u8>,
    /// Where the middle of each land is ([`Region::LANDS`]): what a span
    /// between two of them is drawn through.
    lands: [Vec2; 9],
}

static ARENA: LazyLock<Arc<Grid>> = LazyLock::new(|| {
    Arc::new(Grid::decode(include_bytes!("arena.nav")).expect("arena.nav is a baked grid: bake it again (RT_BAKE_NAV=1)"))
});

impl Grid {
    /// The grid of Hajun's `pvptownmap.glb`, as last baked.
    pub fn arena() -> Arc<Grid> {
        ARENA.clone()
    }

    /// A grid of `width` by `height` squares from `corner`, all void.
    pub fn blank(width: usize, height: usize, corner: Vec2) -> Self {
        Self {
            width,
            height,
            corner,
            floor: vec![NO_FLOOR; width * height],
            flags: vec![0; width * height],
            lands: Region::LANDS.map(Region::centre),
        }
    }

    /// Says where the middle of the land `region` is, on a map that is
    /// not Hajun's.
    pub fn place_land(&mut self, region: Region, middle: Vec2) {
        if let Some(slot) = region.slot() {
            self.lands[slot] = middle;
        }
    }

    /// Where the middle of the land `region` is.
    pub fn land(&self, region: Region) -> Vec2 {
        region.slot().map_or(Vec2::ZERO, |slot| self.lands[slot])
    }

    /// Says what the square at `column`, `row` is: its floor's height, if
    /// it has one; whether a body can stand on it and walk there; and
    /// which piece of land it is part of.
    pub fn set(&mut self, column: usize, row: usize, floor: Option<f32>, clear: bool, region: Option<Region>) {
        let index = row * self.width + column;
        let Some(floor) = floor else {
            self.floor[index] = NO_FLOOR;
            self.flags[index] = 0;
            return;
        };
        self.floor[index] = (((floor - FLOOR_BASE) * 100.0).round().clamp(0.0, 254.0)) as u8;
        self.flags[index] = GROUND | if clear { CLEAR } else { 0 } | (region.map_or(0, Region::code) << 4);
    }

    /// The grid as bytes, for `arena.nav`: its size, its corner, and its
    /// squares row by row in runs of the same.
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = MAGIC.to_vec();
        bytes.extend((self.width as u16).to_le_bytes());
        bytes.extend((self.height as u16).to_le_bytes());
        bytes.extend(self.corner.x.to_le_bytes());
        bytes.extend(self.corner.y.to_le_bytes());
        for land in self.lands {
            bytes.extend(land.x.to_le_bytes());
            bytes.extend(land.y.to_le_bytes());
        }
        let mut index = 0;
        while index < self.floor.len() {
            let same = (self.floor[index], self.flags[index]);
            let mut run = 1;
            while run < usize::from(u16::MAX) && index + run < self.floor.len() && (self.floor[index + run], self.flags[index + run]) == same {
                run += 1;
            }
            bytes.extend((run as u16).to_le_bytes());
            bytes.push(same.0);
            bytes.push(same.1);
            index += run;
        }
        bytes
    }

    /// The grid `bytes` are ([`Grid::encode`]), if they are one.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let rest = bytes.strip_prefix(MAGIC.as_slice())?;
        let (head, rest) = rest.split_at_checked(12)?;
        let (lands, mut runs) = rest.split_at_checked(72)?;
        let number = |bytes: &[u8]| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let width = usize::from(u16::from_le_bytes([head[0], head[1]]));
        let height = usize::from(u16::from_le_bytes([head[2], head[3]]));
        let corner = Vec2::new(
            f32::from_le_bytes([head[4], head[5], head[6], head[7]]),
            f32::from_le_bytes([head[8], head[9], head[10], head[11]]),
        );
        let mut grid = Self::blank(width, height, corner);
        for (slot, land) in grid.lands.iter_mut().enumerate() {
            *land = Vec2::new(number(&lands[slot * 8..]), number(&lands[slot * 8 + 4..]));
        }
        let mut index = 0;
        while let Some((run, rest)) = runs.split_at_checked(4) {
            let count = usize::from(u16::from_le_bytes([run[0], run[1]]));
            if count == 0 || index + count > grid.floor.len() {
                return None;
            }
            grid.floor[index..index + count].fill(run[2]);
            grid.flags[index..index + count].fill(run[3]);
            index += count;
            runs = rest;
        }
        (index == grid.floor.len() && runs.is_empty()).then_some(grid)
    }

    /// How many squares across and down it is.
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// How many squares it has.
    pub fn squares(&self) -> usize {
        self.floor.len()
    }

    /// The square `at` is in, if it is on the grid.
    pub fn index(&self, at: Vec2) -> Option<usize> {
        let square = ((at - self.corner) / CELL).floor();
        (square.x >= 0.0 && square.y >= 0.0 && (square.x as usize) < self.width && (square.y as usize) < self.height)
            .then(|| square.y as usize * self.width + square.x as usize)
    }

    /// The middle of the square numbered `index`.
    pub fn centre(&self, index: usize) -> Vec2 {
        self.corner + Vec2::new((index % self.width) as f32 + 0.5, (index / self.width) as f32 + 0.5) * CELL
    }

    /// The middle of the square at `column`, `row`.
    pub fn centre_of(&self, column: usize, row: usize) -> Vec2 {
        self.corner + Vec2::new(column as f32 + 0.5, row as f32 + 0.5) * CELL
    }

    fn clear_index(&self, index: usize) -> bool {
        self.flags[index] & CLEAR != 0
    }

    /// Whether a body can stand `at` and walk there: its floor is there,
    /// nothing reaches into a body on it, and it can be walked to.
    pub fn clear(&self, at: Vec2) -> bool {
        self.index(at).is_some_and(|index| self.clear_index(index))
    }

    /// How high the floor `at` is, if there is one: land that holds a
    /// body, whether or not it can walk there.
    pub fn height(&self, at: Vec2) -> Option<f32> {
        let index = self.index(at)?;
        (self.flags[index] & GROUND != 0).then(|| f32::from(self.floor[index]) / 100.0 + FLOOR_BASE)
    }

    /// Which piece of land the floor `at` is part of, if there is one.
    pub fn region(&self, at: Vec2) -> Option<Region> {
        Region::of(self.flags[self.index(at)?] >> 4)
    }

    /// The square nearest `at`, no further than `within` from it, that
    /// `wanted` says yes to, by its number.
    fn nearest(&self, at: Vec2, within: f32, wanted: impl Fn(usize) -> bool) -> Option<usize> {
        if let Some(index) = self.index(at)
            && wanted(index)
        {
            return Some(index);
        }
        let square = ((at - self.corner) / CELL).floor();
        let (column, row) = (square.x as i32, square.y as i32);
        let reach = (within / CELL).ceil() as i32;
        let mut best: Option<(f32, usize)> = None;
        for ring in 1..=reach {
            // A ring further out than the best so far cannot hold a nearer
            // one.
            if best.is_some_and(|(away, _)| away < (ring - 1) as f32 * CELL) {
                break;
            }
            for dr in -ring..=ring {
                for dc in -ring..=ring {
                    if dr.abs() != ring && dc.abs() != ring {
                        continue;
                    }
                    let (c, r) = (column + dc, row + dr);
                    if c < 0 || r < 0 || c as usize >= self.width || r as usize >= self.height {
                        continue;
                    }
                    let index = r as usize * self.width + c as usize;
                    if !wanted(index) {
                        continue;
                    }
                    let away = self.centre(index).distance(at);
                    if away <= within && best.is_none_or(|(least, _)| away < least) {
                        best = Some((away, index));
                    }
                }
            }
        }
        best.map(|(_, index)| index)
    }

    /// The nearest place to `at`, no further than `within`, that a body
    /// can stand and walk ([`Grid::clear`]): the middle of its square.
    pub fn nearest_clear(&self, at: Vec2, within: f32) -> Option<Vec2> {
        self.nearest(at, within, |index| self.clear_index(index)).map(|index| self.centre(index))
    }

    /// The straight bridge from the land `from` to the land `to`, on the
    /// line between their middles moved `aside` to its right: from just
    /// inside the one's shore to just inside the other's ([`SPAN_INSET`]).
    /// `None` if the line does not leave the one and come to the other.
    pub fn span(&self, from: Region, to: Region, aside: f32) -> Option<Span> {
        let (a, b) = (self.land(from), self.land(to));
        let whole = a.distance(b);
        let along = (b - a) / whole.max(1e-3);
        let start = a + Vec2::new(-along.y, along.x) * aside;
        let step = 0.1;
        let steps = (whole / step).ceil() as usize;
        let region = |far: f32| self.region(start + along * far);
        // The last of the land it leaves, and then the first of the land
        // it is for.
        let mut left = None;
        let mut landed = None;
        for k in 0..=steps {
            let far = k as f32 * step;
            match region(far) {
                Some(on) if on == from && landed.is_none() => left = Some(far),
                Some(on) if on == to && left.is_some() => {
                    landed = Some(far);
                    break;
                }
                _ => {}
            }
        }
        let (left, landed) = (left?, landed?);
        let span = Span {
            from: start + along * (left - SPAN_INSET),
            to: start + along * (landed + SPAN_INSET),
        };
        (self.height(span.from).is_some() && self.height(span.to).is_some()).then_some(span)
    }
}

/// A straight bridge to be laid: from its first rung to its last, each
/// over land.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Span {
    pub from: Vec2,
    pub to: Vec2,
}

impl Span {
    /// How long it is, in metres: what it costs to lay.
    pub fn length(&self) -> f32 {
        self.from.distance(self.to)
    }

    /// How many planks it is: as few as are no longer than a plank is
    /// ([`PLANK_SPACING`]), all of a length.
    pub fn planks(&self) -> usize {
        ((self.length() / PLANK_SPACING).ceil() as usize).max(1)
    }

    /// Where its rung numbered `index` is: the first at its start, the
    /// last, numbered [`Span::planks`], at its end.
    pub fn rung(&self, index: usize) -> Vec2 {
        self.from.lerp(self.to, (index as f32 / self.planks() as f32).min(1.0))
    }

    /// Which way it runs, across the ground.
    pub fn way(&self) -> Vec2 {
        (self.to - self.from).normalize_or_zero()
    }

    /// Which way it runs as a plank heads, in radians: a turn of this
    /// about the upright points -Z along it.
    pub fn heading(&self) -> f32 {
        let way = self.way();
        (-way.x).atan2(-way.y)
    }
}

/// One bridge: whose it is, by seat, its number among theirs, and its
/// rungs in order; and those that have come ahead of the ones before them.
#[derive(Clone, Debug)]
struct Chain {
    seat: u8,
    id: u16,
    rungs: Vec<Vec2>,
    waiting: Vec<(u16, Vec2)>,
}

/// The map with every bridge laid on it so far: what the computer's
/// players walk by.
#[derive(Clone, Debug)]
pub struct Nav {
    grid: Arc<Grid>,
    chains: Vec<Chain>,
    /// Which squares a body may walk for being on a plank: those whose
    /// middle is within [`DECK_HALF`] of a plank's middle line.
    deck: Vec<bool>,
    /// Which piece of everything that can be walked each square was part
    /// of as it became walkable, land and planks together; 0 for a square
    /// that cannot be walked.
    label: Vec<u16>,
    /// Which pieces a plank has joined since: each piece's number, and
    /// through this the number of the piece it is part of now
    /// ([`Nav::piece`]). Two squares whose pieces come to the same one can
    /// be walked between. A plank only ever adds squares to walk, so
    /// laying one joins pieces and never parts them, and nothing of the
    /// map has to be gone over again for it (until 2026-11-12 all of it
    /// was, at every plank: 3 ms a plank on a laptop, a frame gone on a
    /// phone, where a game played offline lays a dozen a second as it
    /// begins).
    joined_to: Vec<u16>,
    /// Counts up each time a plank is laid.
    pub version: u32,
    /// Somewhere to stand on each land ([`Region::LANDS`]): the square a
    /// body can walk that is nearest its middle.
    hearts: [Option<usize>; 9],
}

impl Nav {
    /// The map `grid` with nothing built on it.
    pub fn new(grid: Arc<Grid>) -> Self {
        let squares = grid.squares();
        let mut nav = Self {
            grid,
            chains: Vec::new(),
            deck: vec![false; squares],
            label: vec![0; squares],
            joined_to: vec![0],
            version: 0,
            hearts: [None; 9],
        };
        for index in 0..squares {
            if !nav.grid.clear_index(index) {
                continue;
            }
            let Some(slot) = Region::of(nav.grid.flags[index] >> 4).and_then(Region::slot) else {
                continue;
            };
            let middle = nav.grid.lands[slot];
            let nearer = nav.hearts[slot].is_none_or(|best| {
                nav.grid.centre(index).distance_squared(middle) < nav.grid.centre(best).distance_squared(middle)
            });
            if nearer {
                nav.hearts[slot] = Some(index);
            }
        }
        nav.relabel();
        nav
    }

    /// Hajun's map with nothing built on it.
    pub fn arena() -> Self {
        Self::new(Grid::arena())
    }

    /// The map itself.
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// Takes in a rung someone has laid: the `index`th of the bridge
    /// numbered `chain` of whoever is in `seat`, `at` across the ground.
    /// As the game's `bridge` takes them: in order, one that comes early
    /// waiting for those before it, one that has come already left out.
    /// Whether a plank was laid by it.
    pub fn hear(&mut self, seat: u8, chain: u16, index: u16, at: Vec2) -> bool {
        if !at.is_finite() {
            return false;
        }
        let found = match self.chains.iter().position(|kept| kept.seat == seat && kept.id == chain) {
            Some(found) => found,
            None => {
                if self.chains.len() >= MOST_CHAINS {
                    return false;
                }
                self.chains.push(Chain {
                    seat,
                    id: chain,
                    rungs: Vec::new(),
                    waiting: Vec::new(),
                });
                self.chains.len() - 1
            }
        };
        let kept = &mut self.chains[found];
        if usize::from(index) < kept.rungs.len() || kept.rungs.len() >= MOST_RUNGS || kept.waiting.len() >= MOST_RUNGS {
            return false;
        }
        kept.waiting.push((index, at));
        kept.waiting.sort_by_key(|(index, _)| *index);
        let mut laid = Vec::new();
        while kept.waiting.first().is_some_and(|(index, _)| usize::from(*index) == kept.rungs.len()) {
            let (_, rung) = kept.waiting.remove(0);
            if let Some(&last) = kept.rungs.last() {
                laid.push((last, rung));
            }
            kept.rungs.push(rung);
        }
        let mut any = false;
        for (a, b) in laid {
            any |= self.lay_deck(a, b);
        }
        if any {
            self.version += 1;
        }
        any
    }

    /// The rungs of the bridge numbered `chain` of whoever is in `seat`,
    /// as far as they have come in order.
    pub fn rungs(&self, seat: u8, chain: u16) -> &[Vec2] {
        self.chains
            .iter()
            .find(|kept| kept.seat == seat && kept.id == chain)
            .map_or(&[], |kept| kept.rungs.as_slice())
    }

    /// Every plank laid, as the two rungs it is between.
    pub fn planks(&self) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        self.chains
            .iter()
            .flat_map(|kept| kept.rungs.windows(2).map(|pair| (pair[0], pair[1])))
            .filter(|(a, b)| a.distance(*b) <= MOST_PLANK)
    }

    /// Marks the squares along the plank from `a` to `b` as walkable, and
    /// joins whatever pieces they lie between: whether it was a plank.
    fn lay_deck(&mut self, a: Vec2, b: Vec2) -> bool {
        if a.distance(b) > MOST_PLANK {
            return false;
        }
        let grid = self.grid.clone();
        let low = ((a.min(b) - DECK_HALF - grid.corner) / CELL).floor().max(Vec2::ZERO);
        let high = ((a.max(b) + DECK_HALF - grid.corner) / CELL).floor();
        let (width, height) = grid.size();
        let (first, last) = (low.as_uvec2(), high.as_uvec2().min(bevy::math::UVec2::new(width as u32 - 1, height as u32 - 1)));
        if high.x < 0.0 || high.y < 0.0 {
            return true;
        }
        // The squares it makes walkable that were not.
        let mut fresh = Vec::new();
        for row in first.y..=last.y {
            for column in first.x..=last.x {
                let at = grid.centre_of(column as usize, row as usize);
                let index = row as usize * width + column as usize;
                if away_from(at, a, b) <= DECK_HALF && !self.deck[index] {
                    if !grid.clear_index(index) {
                        fresh.push(index);
                    }
                    self.deck[index] = true;
                }
            }
        }
        // Each is part of whatever is walked beside it, which it joins
        // into one piece; with nothing beside it, a piece of its own. (A
        // step cornerwise needs both squares beside the corner, so the
        // four beside a square are all that joining goes by.)
        for index in fresh {
            let (column, row) = ((index % width) as i32, (index / width) as i32);
            let mut mine = 0u16;
            for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (c, r) = (column + dc, row + dr);
                if c < 0 || r < 0 || c as usize >= width || r as usize >= height {
                    continue;
                }
                let beside = self.piece(self.label[r as usize * width + c as usize]);
                if beside == 0 || beside == mine {
                    continue;
                }
                if mine == 0 {
                    mine = beside;
                } else {
                    let (kept, gone) = (mine.min(beside), mine.max(beside));
                    self.joined_to[usize::from(gone)] = kept;
                    mine = kept;
                }
            }
            if mine == 0 {
                // Out of numbers, after tens of thousands of planks laid
                // apart from everything: all of it gone over afresh.
                if self.joined_to.len() >= usize::from(u16::MAX) {
                    self.relabel();
                    return true;
                }
                mine = self.joined_to.len() as u16;
                self.joined_to.push(mine);
            }
            self.label[index] = mine;
        }
        true
    }

    /// The piece that the piece numbered `label` is part of now, pieces
    /// having been joined by planks since it was numbered; 0 for none.
    fn piece(&self, label: u16) -> u16 {
        let mut piece = label;
        while let Some(&next) = self.joined_to.get(usize::from(piece)) {
            if next == piece {
                break;
            }
            piece = next;
        }
        piece
    }

    fn walkable(&self, index: usize) -> bool {
        self.deck[index] || self.grid.clear_index(index)
    }

    /// Whether a body's middle may be `at`: on land it can walk, or along
    /// the middle of a plank.
    pub fn walkable_at(&self, at: Vec2) -> bool {
        self.grid.index(at).is_some_and(|index| self.walkable(index))
    }

    /// Whether `at` is over a plank: within its width of one's middle
    /// line, between its rungs.
    pub fn on_plank(&self, at: Vec2) -> bool {
        self.planks().any(|(a, b)| {
            let along = b - a;
            let length = along.length();
            if length < 1e-4 {
                return false;
            }
            let way = along / length;
            let off = at - a;
            let far = off.dot(way);
            (-0.05..=length + 0.05).contains(&far) && off.perp_dot(way).abs() <= PLANK_HALF_WIDTH
        })
    }

    /// How high the ground `at` is, if there is any: a plank's top, or the
    /// land's floor, whichever is higher. `None` over the void.
    pub fn ground(&self, at: Vec2) -> Option<f32> {
        let land = self.grid.height(at);
        if self.on_plank(at) {
            return Some(land.map_or(PLANK_TOP, |land| land.max(PLANK_TOP)));
        }
        land
    }

    /// The numbers of the squares round the one numbered `index` that a
    /// body walking from it can step to, and how far each is: the four
    /// beside it, and each of the four cornerwise if both squares beside
    /// that corner can be walked too, so that no corner is cut.
    fn steps(&self, index: usize) -> impl Iterator<Item = (usize, f32)> + '_ {
        let (width, height) = self.grid.size();
        let (column, row) = ((index % width) as i32, (index / width) as i32);
        const ROUND: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)];
        ROUND.into_iter().filter_map(move |(dc, dr)| {
            let (c, r) = (column + dc, row + dr);
            if c < 0 || r < 0 || c as usize >= width || r as usize >= height {
                return None;
            }
            let next = r as usize * width + c as usize;
            if !self.walkable(next) {
                return None;
            }
            if dc != 0 && dr != 0 {
                let beside = (row as usize * width + c as usize, r as usize * width + column as usize);
                if !self.walkable(beside.0) || !self.walkable(beside.1) {
                    return None;
                }
                return Some((next, CELL * std::f32::consts::SQRT_2));
            }
            Some((next, CELL))
        })
    }

    /// Works out afresh which squares can be walked between
    /// ([`Nav::label`]), every square of the map gone over: as the map is
    /// first read. After that each plank joins what it joins as it is laid
    /// ([`Nav::lay_deck`]).
    fn relabel(&mut self) {
        let squares = self.grid.squares();
        let mut label = vec![0u16; squares];
        let mut next = 1u16;
        let mut stack = Vec::new();
        for seed in 0..squares {
            if label[seed] != 0 || !self.walkable(seed) {
                continue;
            }
            label[seed] = next;
            stack.push(seed);
            while let Some(index) = stack.pop() {
                for (beside, _) in self.steps(index) {
                    if label[beside] == 0 {
                        label[beside] = next;
                        stack.push(beside);
                    }
                }
            }
            next = next.saturating_add(1);
        }
        self.label = label;
        self.joined_to = (0..next).collect();
    }

    /// The square to walk on nearest `at`, no further than [`BESIDE`].
    fn foothold(&self, at: Vec2) -> Option<usize> {
        self.grid.nearest(at, BESIDE, |index| self.walkable(index))
    }

    /// The place to walk on nearest `at`, no further than [`BESIDE`]: the
    /// middle of its square, or `at` itself if it can be walked.
    pub fn nearest_walkable(&self, at: Vec2) -> Option<Vec2> {
        if self.walkable_at(at) {
            return Some(at);
        }
        self.foothold(at).map(|index| self.grid.centre(index))
    }

    /// Whether there is a way to walk from `a` to `b`, over land and
    /// planks.
    pub fn connected(&self, a: Vec2, b: Vec2) -> bool {
        match (self.foothold(a), self.foothold(b)) {
            (Some(a), Some(b)) => self.piece(self.label[a]) == self.piece(self.label[b]),
            _ => false,
        }
    }

    /// Whether the lands `a` and `b` have been joined: by one bridge, or
    /// by several through other land.
    pub fn joined(&self, a: Region, b: Region) -> bool {
        a == b || self.connected(self.heart(a), self.heart(b))
    }

    /// Somewhere to stand on the land `region`: the place a body can walk
    /// that is nearest its middle (for the middle of the map, whose own
    /// middle is the pond, on the path round that).
    pub fn heart(&self, region: Region) -> Vec2 {
        region
            .slot()
            .and_then(|slot| self.hearts[slot])
            .map_or_else(|| self.grid.land(region), |index| self.grid.centre(index))
    }

    /// Whether a body can walk the straight line from `a` to `b`: every
    /// step of it on land it can walk or along a plank.
    pub fn line(&self, a: Vec2, b: Vec2) -> bool {
        let whole = a.distance(b);
        let steps = (whole / 0.2).ceil().max(1.0) as usize;
        (0..=steps).all(|k| self.walkable_at(a.lerp(b, k as f32 / steps as f32)))
    }

    /// A way to walk from `from` to `to`, over land and planks, as the
    /// points to walk to in turn, the last of them `to` itself or as near
    /// it as can be walked; or `None` if there is no way. A short one
    /// over the squares (A*, [`GREED`]), with every point of it that can
    /// be walked straight past left out.
    pub fn path(&self, from: Vec2, to: Vec2) -> Option<Vec<Vec2>> {
        self.path_weighing(from, to, GREED)
    }

    /// [`Nav::path`], weighing what is left of a way `greed` times what
    /// is done of it.
    fn path_weighing(&self, from: Vec2, to: Vec2, greed: f32) -> Option<Vec<Vec2>> {
        let (start, goal) = (self.foothold(from)?, self.foothold(to)?);
        if self.piece(self.label[start]) != self.piece(self.label[goal]) {
            return None;
        }
        let (width, _) = self.grid.size();
        let place = |index: usize| ((index % width) as f32, (index / width) as f32);
        let (gx, gy) = place(goal);
        // As the crow flies over the squares, straight and cornerwise.
        let guess = |index: usize| {
            let (x, y) = place(index);
            let (dx, dy) = ((x - gx).abs(), (y - gy).abs());
            (dx.max(dy) + (std::f32::consts::SQRT_2 - 1.0) * dx.min(dy)) * CELL * greed
        };
        let cells = SCRATCH.with_borrow_mut(|scratch| {
            scratch.begin(self.grid.squares());
            let mut open = BinaryHeap::new();
            scratch.reach(start, 0.0, start as u32);
            open.push(Open {
                total: guess(start),
                index: start as u32,
            });
            while let Some(Open { index, .. }) = open.pop() {
                let index = index as usize;
                if index == goal {
                    break;
                }
                // Gone on from once, a square is done with: an older,
                // dearer way to it left in the heap is passed over.
                if !scratch.close(index) {
                    continue;
                }
                let so_far = scratch.cost[index];
                for (next, step) in self.steps(index) {
                    let cost = so_far + step;
                    if scratch.reach(next, cost, index as u32) {
                        open.push(Open {
                            total: cost + guess(next),
                            index: next as u32,
                        });
                    }
                }
            }
            if !scratch.seen(goal) {
                return None;
            }
            let mut cells = vec![goal];
            while *cells.last()? != start {
                cells.push(scratch.from[*cells.last()?] as usize);
            }
            cells.reverse();
            Some(cells)
        })?;
        let mut points: Vec<Vec2> = cells.into_iter().map(|index| self.grid.centre(index)).collect();
        // From where it is asked from and to where it is asked for, if
        // those can be walked.
        if self.walkable_at(from) {
            points.insert(0, from);
        }
        if self.walkable_at(to) {
            points.push(to);
        }
        Some(self.straighten(points))
    }

    /// `points` with every one left out that can be walked straight past:
    /// from each kept, the furthest ahead that a straight walk reaches, no
    /// further off than a while's walk.
    fn straighten(&self, points: Vec<Vec2>) -> Vec<Vec2> {
        const AHEAD: f32 = 40.0;
        let mut kept = Vec::new();
        let mut at = 0;
        while at + 1 < points.len() {
            // The furthest first: on open ground that is the one, and the
            // rest are never tried.
            let reach = (at + 2..points.len())
                .take_while(|&further| points[at].distance(points[further]) <= AHEAD)
                .last()
                .unwrap_or(at + 1);
            let next = (at + 2..=reach)
                .rev()
                .find(|&further| self.line(points[at], points[further]))
                .unwrap_or(at + 1);
            kept.push(points[next]);
            at = next;
        }
        if kept.is_empty()
            && let Some(&only) = points.last()
        {
            kept.push(only);
        }
        kept
    }
}

/// How much a search for a way weighs what is left of it over what is done
/// of it ([`Nav::path`]): at 1 it finds the shortest way over the squares,
/// having tried everything nearer; over 1 it goes for the far end sooner,
/// and the way it finds may be that many times the shortest at the most,
/// before it is straightened. At 2, measured on the map on a laptop
/// (2026-11-12, `timing_of_the_arena_s_map`): from a team's island round
/// the pond to the far side of the middle, 0.09 ms where 1 took 2.1, and
/// to the island across the map 0.35 ms where 1 took 4.8, the ways found,
/// once straightened, a metre longer in 274 and 3.6 m shorter in 161. On a
/// phone, where a game played offline finds its bots' ways between two
/// frames 8 ms apart, that is the difference between a frame late and
/// not.
const GREED: f32 = 2.0;

/// How far `at` is from the stretch from `a` to `b`.
fn away_from(at: Vec2, a: Vec2, b: Vec2) -> f32 {
    let along = b - a;
    let whole = along.length_squared();
    if whole < 1e-8 {
        return at.distance(a);
    }
    let share = ((at - a).dot(along) / whole).clamp(0.0, 1.0);
    at.distance(a + along * share)
}

/// A square a search has come to and not yet gone on from, and what the
/// whole way through it comes to at the least.
struct Open {
    total: f32,
    index: u32,
}

impl PartialEq for Open {
    fn eq(&self, other: &Self) -> bool {
        self.total == other.total && self.index == other.index
    }
}

impl Eq for Open {}

impl PartialOrd for Open {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Open {
    // The least first, out of a heap that gives the greatest.
    fn cmp(&self, other: &Self) -> Ordering {
        other.total.total_cmp(&self.total).then_with(|| other.index.cmp(&self.index))
    }
}

/// What a search keeps of each square, kept from one search to the next so
/// that none of it is made again: which search last came to the square,
/// which last went on from it, what the way to it cost, and which square
/// it came from.
#[derive(Default)]
struct Scratch {
    round: u32,
    stamp: Vec<u32>,
    closed: Vec<u32>,
    cost: Vec<f32>,
    from: Vec<u32>,
}

impl Scratch {
    fn begin(&mut self, squares: usize) {
        if self.stamp.len() != squares {
            self.stamp = vec![0; squares];
            self.closed = vec![0; squares];
            self.cost = vec![0.0; squares];
            self.from = vec![0; squares];
            self.round = 0;
        }
        self.round += 1;
    }

    fn seen(&self, index: usize) -> bool {
        self.stamp[index] == self.round
    }

    /// The search goes on from the square numbered `index`: whether it
    /// had not already.
    fn close(&mut self, index: usize) -> bool {
        if self.closed[index] == self.round {
            return false;
        }
        self.closed[index] = self.round;
        true
    }

    /// Comes to the square numbered `index` at `cost` from the one
    /// numbered `from`: whether that is the cheapest way to it yet, the
    /// search not having gone on from it already.
    fn reach(&mut self, index: usize, cost: f32, from: u32) -> bool {
        if self.closed[index] == self.round || (self.seen(index) && self.cost[index] <= cost) {
            return false;
        }
        self.stamp[index] = self.round;
        self.cost[index] = cost;
        self.from[index] = from;
        true
    }
}

thread_local! {
    static SCRATCH: RefCell<Scratch> = RefCell::new(Scratch::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two round islands 6 m across, 20 m apart on the x axis, on a grid
    /// 40 m by 20: region `Team(0)` at the origin and `Middle` at (20, 0),
    /// each clear to half a metre inside its edge.
    fn two_islands() -> Grid {
        let mut grid = Grid::blank(80, 40, Vec2::new(-10.0, -10.0));
        for row in 0..40 {
            for column in 0..80 {
                let at = grid.centre_of(column, row);
                for (centre, region) in [(Vec2::ZERO, Region::Team(0)), (Vec2::new(20.0, 0.0), Region::Middle)] {
                    let away = at.distance(centre);
                    if away <= 3.0 {
                        grid.set(column, row, Some(0.25), away <= 2.5, Some(region));
                    }
                }
            }
        }
        grid.place_land(Region::Team(0), Vec2::ZERO);
        grid.place_land(Region::Middle, Vec2::new(20.0, 0.0));
        grid
    }

    #[test]
    #[ignore]
    fn timing_of_the_arena_s_map() {
        let started = std::time::Instant::now();
        let mut nav = Nav::arena();
        println!("Nav::arena(): {:?}, {} squares", started.elapsed(), nav.grid().squares());
        let started = std::time::Instant::now();
        for _ in 0..50 {
            nav.relabel();
        }
        println!("relabel: {:?} each", started.elapsed() / 50);
        let span = nav.grid().span(Region::Team(0), Region::Middle, 0.0).unwrap();
        let started = std::time::Instant::now();
        for index in 0..=span.planks() {
            nav.hear(4, 0, index as u16, span.rung(index));
        }
        println!("a span of {} planks heard: {:?}", span.planks(), started.elapsed());
        // Ways long and short, at several weights: how long each takes
        // to find, and how long it is to walk.
        let across = grid_span(&mut nav);
        let walks = [
            ("an island to the middle's heart", nav.heart(Region::Team(0)), nav.heart(Region::Middle)),
            ("an island to the island across", nav.heart(Region::Team(0)), across),
            ("an island to its bridge's end", nav.heart(Region::Team(0)), Vec2::new(0.0, 80.0)),
            ("across its own island", nav.heart(Region::Team(0)), Vec2::new(0.16, 135.92)),
        ];
        for (name, from, to) in walks {
            for greed in [1.0, 1.1, 1.3, 1.6, 2.0, 3.0] {
                let started = std::time::Instant::now();
                let mut way = None;
                for _ in 0..20 {
                    way = nav.path_weighing(from, to, greed);
                }
                let each = started.elapsed() / 20;
                let way = way.expect("a way");
                let mut walked = 0.0;
                let mut at = from;
                for &point in &way {
                    walked += at.distance(point);
                    at = point;
                }
                println!("{name}, greed {greed}: {each:?}, {walked:.1} m in {} stretches", way.len());
            }
        }
        let started = std::time::Instant::now();
        let mut on = 0;
        for k in 0..100_000 {
            on += usize::from(nav.on_plank(Vec2::new(0.0, 80.0 + (k % 40) as f32)));
        }
        println!("on_plank: {:?} each ({on})", started.elapsed() / 100_000);
    }

    /// Lays the span from the middle to the island across from team 0's,
    /// and gives back that island's heart.
    fn grid_span(nav: &mut Nav) -> Vec2 {
        let span = nav.grid().span(Region::Middle, Region::Team(2), 0.0).unwrap();
        for index in 0..=span.planks() {
            nav.hear(6, 0, index as u16, span.rung(index));
        }
        nav.heart(Region::Team(2))
    }

    #[test]
    fn a_grid_is_the_same_baked_and_read_back() {
        let grid = two_islands();
        let bytes = grid.encode();
        assert!(bytes.len() < 2_000, "{} bytes", bytes.len());
        assert_eq!(Grid::decode(&bytes).as_ref(), Some(&grid));
        assert_eq!(Grid::decode(&bytes[..bytes.len() - 1]), None);
        assert_eq!(Grid::decode(b"nothing"), None);
        // What it says of a place.
        assert!(grid.clear(Vec2::ZERO) && grid.clear(Vec2::new(2.2, 0.0)));
        assert!(!grid.clear(Vec2::new(2.8, 0.0)), "the edge holds a body but is not walked");
        assert_eq!(grid.height(Vec2::new(2.8, 0.0)), Some(0.25));
        assert_eq!(grid.height(Vec2::new(5.0, 0.0)), None);
        assert_eq!(grid.region(Vec2::new(19.0, 1.0)), Some(Region::Middle));
        assert_eq!(grid.region(Vec2::new(10.0, 0.0)), None);
        assert_eq!(grid.index(Vec2::new(-11.0, 0.0)), None);
        let near = grid.nearest_clear(Vec2::new(4.0, 0.0), 3.0).unwrap();
        assert!(near.distance(Vec2::new(2.25, 0.25)) < 0.6, "{near}");
        assert_eq!(grid.nearest_clear(Vec2::new(10.0, 0.0), 3.0), None);
    }

    #[test]
    fn the_span_between_two_lands_runs_from_inside_the_one_shore_to_inside_the_other() {
        let grid = two_islands();
        let span = grid.span(Region::Team(0), Region::Middle, 0.0).unwrap();
        // The shores are 3 m from each middle, to the grid's half metre.
        assert!((span.from.x - 2.5).abs() <= 0.35 && span.from.y.abs() < 0.01, "{:?}", span.from);
        assert!((span.to.x - 17.5).abs() <= 0.35 && span.to.y.abs() < 0.01, "{:?}", span.to);
        assert!(grid.height(span.from).is_some() && grid.height(span.to).is_some());
        // Planks all of a length, none longer than a plank is.
        let planks = span.planks();
        assert_eq!(planks, (span.length() / PLANK_SPACING).ceil() as usize);
        let each = span.rung(0).distance(span.rung(1));
        assert!(each <= PLANK_SPACING + 1e-4 && each > PLANK_SPACING * 0.8);
        assert_eq!(span.rung(0), span.from);
        assert!(span.rung(planks).distance(span.to) < 1e-4);
        assert_eq!(span.way(), Vec2::X);
        // Heading east, as a plank heads: a quarter turn clockwise from -Z.
        assert!((span.heading() + std::f32::consts::FRAC_PI_2).abs() < 1e-4, "{}", span.heading());
        // The other way, and to one side.
        let back = grid.span(Region::Middle, Region::Team(0), 1.0).unwrap();
        assert!(back.from.x > 17.0 && back.to.x < 3.0 && (back.from.y + 1.0).abs() < 0.01, "{back:?}");
        // No span to land that is not there.
        assert_eq!(grid.span(Region::Team(0), Region::Sapphire(1), 0.0), None);
    }

    #[test]
    fn a_bridge_heard_rung_by_rung_joins_what_it_reaches_and_is_walked() {
        let grid = Arc::new(two_islands());
        let mut nav = Nav::new(grid.clone());
        let (here, there) = (Vec2::new(-1.0, 0.5), Vec2::new(21.0, -1.0));
        assert!(!nav.joined(Region::Team(0), Region::Middle));
        assert!(!nav.connected(here, there));
        assert_eq!(nav.path(here, there), None);
        // On an island a walk is straight there.
        assert_eq!(nav.path(here, Vec2::new(1.5, -1.0)), Some(vec![Vec2::new(1.5, -1.0)]));
        // The span is laid rung by rung: the first is no plank, and until
        // the last the far land is not reached.
        let span = grid.span(Region::Team(0), Region::Middle, 0.0).unwrap();
        let planks = span.planks();
        assert!(!nav.hear(4, 0, 0, span.rung(0)));
        for index in 1..planks {
            assert!(nav.hear(4, 0, index as u16, span.rung(index)));
            assert!(!nav.joined(Region::Team(0), Region::Middle), "joined at rung {index} of {planks}");
        }
        let short = span.rung(planks - 1);
        assert!(nav.on_plank(short - Vec2::X * 0.5) && nav.walkable_at(short - Vec2::X * 0.5));
        assert_eq!(nav.ground(short - Vec2::X * 0.5), Some(PLANK_TOP));
        assert!(nav.connected(here, short - Vec2::X * 0.5));
        // Off its side there is nothing.
        assert!(!nav.on_plank(short + Vec2::new(-0.5, PLANK_HALF_WIDTH + 0.1)));
        assert_eq!(nav.ground(short + Vec2::new(-0.5, PLANK_HALF_WIDTH + 0.1)), None);
        // The middle of the plank is walked, its edge only stood on.
        assert!(!nav.walkable_at(short + Vec2::new(-0.5, PLANK_HALF_WIDTH - 0.1)));
        assert!(nav.on_plank(short + Vec2::new(-0.5, PLANK_HALF_WIDTH - 0.1)));
        // The last rung lands it.
        let version = nav.version;
        assert!(nav.hear(4, 0, planks as u16, span.rung(planks)));
        assert!(nav.version > version);
        assert!(nav.joined(Region::Team(0), Region::Middle) && nav.connected(here, there));
        // A rung heard twice is nothing new.
        assert!(!nav.hear(4, 0, 3, span.rung(3)));
        // The way across: along the bridge's middle, every stretch of it
        // walkable, and no longer than the straight line and a little.
        let path = nav.path(here, there).unwrap();
        assert_eq!(path.last(), Some(&there));
        let mut from = here;
        let mut walked = 0.0;
        for &point in &path {
            assert!(nav.line(from, point), "{from} to {point}");
            walked += from.distance(point);
            from = point;
        }
        assert!(walked < here.distance(there) * 1.1, "{walked} m for {} m", here.distance(there));
        assert!(path.len() <= 4, "{path:?}");
        // Land is higher than a plank where both are.
        assert_eq!(nav.ground(span.from), Some(0.25));
    }

    #[test]
    fn rungs_out_of_order_wait_and_a_plank_too_long_is_not_walked() {
        let grid = Arc::new(two_islands());
        let mut nav = Nav::new(grid.clone());
        let span = grid.span(Region::Team(0), Region::Middle, 0.0).unwrap();
        // The third comes before the second: nothing is laid until it has.
        assert!(!nav.hear(1, 7, 0, span.rung(0)));
        assert!(!nav.hear(1, 7, 2, span.rung(2)));
        assert_eq!(nav.rungs(1, 7).len(), 1);
        assert!(nav.hear(1, 7, 1, span.rung(1)));
        assert_eq!(nav.rungs(1, 7).len(), 3);
        assert_eq!(nav.planks().count(), 2);
        // Another's chain of the same number is its own.
        assert!(nav.rungs(2, 7).is_empty());
        // A rung far from the one before it is kept, and is no plank.
        assert!(!nav.hear(1, 7, 3, span.to));
        assert_eq!(nav.rungs(1, 7).len(), 4);
        assert!(!nav.joined(Region::Team(0), Region::Middle));
        assert!(!nav.on_plank(span.rung(8)));
        // Nor is one that is not a place at all.
        assert!(!nav.hear(1, 8, 0, Vec2::new(f32::NAN, 0.0)));
    }

    #[test]
    fn planks_join_pieces_as_they_are_laid_as_going_over_the_whole_map_would() {
        // Bridges laid every which way between and beside two islands,
        // some of them whole, some left short, some crossing: after each
        // plank, which places can be walked between is what going over
        // the whole map afresh makes it.
        let grid = Arc::new(two_islands());
        let mut nav = Nav::new(grid.clone());
        let places: Vec<Vec2> = (0..16)
            .flat_map(|x| [-3.0, 0.0, 1.5, 3.5].map(|y: f32| Vec2::new(-2.0 + x as f32 * 1.6, y)))
            .collect();
        let chains = [
            (Vec2::new(2.5, 0.0), Vec2::new(9.0, 0.5)),
            (Vec2::new(17.5, -0.5), Vec2::new(11.0, -3.0)),
            (Vec2::new(6.0, 3.5), Vec2::new(14.0, 3.5)),
            (Vec2::new(9.0, 0.5), Vec2::new(12.0, 3.5)),
            (Vec2::new(11.0, -3.0), Vec2::new(8.5, 0.4)),
            (Vec2::new(2.0, 1.5), Vec2::new(6.0, 3.5)),
        ];
        let mut joined_at = None;
        for (chain, &(from, to)) in chains.iter().enumerate() {
            let planks = (from.distance(to) / PLANK_SPACING).ceil() as usize;
            for index in 0..=planks {
                nav.hear(2, chain as u16, index as u16, from.lerp(to, index as f32 / planks as f32));
                let mut afresh = nav.clone();
                afresh.relabel();
                for (k, &a) in places.iter().enumerate() {
                    for &b in &places[k + 1..] {
                        assert_eq!(nav.connected(a, b), afresh.connected(a, b), "{a} and {b}, chain {chain} rung {index}");
                        assert_eq!(nav.path(a, b).is_some(), afresh.connected(a, b), "a way from {a} to {b}");
                    }
                }
                if joined_at.is_none() && nav.joined(Region::Team(0), Region::Middle) {
                    joined_at = Some(chain);
                }
            }
        }
        // The two islands were joined by the fifth, which closed the gap
        // between the first two bridges' ends.
        assert_eq!(joined_at, Some(4));
    }

    #[test]
    fn a_way_goes_round_what_stands_in_it() {
        // A wall across the island with a gap at its far end.
        let mut grid = two_islands();
        for row in 0..40 {
            for column in 0..80 {
                let at = grid.centre_of(column, row);
                if at.x.abs() < 0.3 && at.y > -1.6 && at.distance(Vec2::ZERO) <= 3.0 {
                    grid.set(column, row, Some(0.25), false, Some(Region::Team(0)));
                }
            }
        }
        let nav = Nav::new(Arc::new(grid));
        let (from, to) = (Vec2::new(-1.5, 1.0), Vec2::new(1.5, 1.0));
        assert!(!nav.line(from, to));
        let path = nav.path(from, to).unwrap();
        assert!(path.len() >= 2, "{path:?}");
        let mut at = from;
        let mut walked = 0.0;
        for &point in &path {
            assert!(nav.line(at, point), "{at} to {point}");
            walked += at.distance(point);
            at = point;
        }
        assert_eq!(at, to);
        // Down to the gap and back up: about 2.9 m each way.
        assert!(walked > 5.0 && walked < 8.5, "{walked}");
        // Asked for somewhere that cannot be walked, the nearest that can.
        let beside = nav.path(from, Vec2::new(0.0, 0.5)).unwrap();
        assert!(beside.last().unwrap().distance(Vec2::new(0.0, 0.5)) < 1.0);
    }
}
