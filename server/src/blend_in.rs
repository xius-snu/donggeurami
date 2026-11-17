//! Blend In at its tables, on the server (since 2026-11-18; Hajun: "1 random
//! hider chosen as the person walking around pretending to be an NPC/AI. 1~3
//! other people trying to find who is the person, by punching with a aura
//! sword"): what a table of it plays (`tables::Rules`), on the town's own
//! map with nobody of the town's in it.
//!
//! How long its table waits for people is its own already
//! ([`BlendIn::wait_from_join`]): the rest is for now a stand-in, so that a
//! table can be played from the join to the town again: everyone waits and
//! plays where they would arrive in the town, on the path round its pond,
//! and after [`STAND_IN`] seconds the game is over, seat 0 first and the
//! rest in seat order. The game's own rules (`roundtown_net::blend_in`)
//! take its place, in this file.

use bevy::prelude::*;
use roundtown_net::{Ask, Venue, arrival, blend_in};

use crate::tables::{Ctx, Rules, Table, yaw_of};

/// How long the stand-in's game lasts, in seconds.
const STAND_IN: f32 = 20.0;

/// The least a table waits from someone sitting down at it after the
/// second person, in seconds (mine, in the games' design: "The game starts
/// 15 s after the second person joins (each later join resets that to at
/// least 8 s), or at once with four"): a third does not have the two
/// already there wait the whole of it again, nor come too late to look
/// about them; a fourth fills the table, which counts down at once.
const LATER_WAIT: f32 = 8.0;

/// A game of Blend In at its table.
pub(crate) struct BlendIn;

impl BlendIn {
    /// The game at a new table, numbered as `table` is, its dice thrown from
    /// `seed`.
    pub(crate) fn new(_table: &Table, _seed: u64) -> Self {
        Self
    }
}

impl Rules for BlendIn {
    fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
        let (at, turn) = blend_in::arrival(seat);
        (at, yaw_of(turn))
    }

    /// The whole wait (15 s) from the second person, with whom the game can
    /// begin; from each after them, what was left of it, but never under
    /// [`LATER_WAIT`].
    fn wait_from_join(&self, table: &Table, left: Option<f32>) -> f32 {
        left.map_or(table.shape.wait, |left| left.max(LATER_WAIT))
    }

    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        for seat in 0..table.seats.len() {
            let (at, turn) = arrival(Venue::BlendIn, seat);
            ctx.place(table, seat, at, yaw_of(turn));
        }
    }

    fn step(&mut self, _ctx: &mut Ctx, table: &mut Table, _dt: f32) {
        if table.elapsed() >= STAND_IN {
            let seats = table.seats.len();
            table.finish((0..seats as u8).collect(), vec![0; seats], None);
        }
    }

    fn hear(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize, ask: &Ask) {
        // Fallen off the world: back where the seat stands, as a fresh
        // arrival there.
        if let Ask::Respawn = ask {
            let (at, yaw) = if table.playing() || table.over() {
                let (at, turn) = arrival(Venue::BlendIn, seat);
                (at, yaw_of(turn))
            } else {
                self.waiting_spot(table, seat)
            };
            ctx.place(table, seat, at, yaw);
        }
    }

    fn show(&mut self, _commands: &mut Commands, _table: &Table) {}

    fn close(&mut self, _commands: &mut Commands, _table: &Table) {}
}
