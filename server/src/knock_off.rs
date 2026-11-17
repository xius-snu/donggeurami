//! Knock Off at its tables, on the server (since 2026-11-18; Hajun: "You
//! spawn on a tiny icy circular island with 2~4 other ppl (fill with ai like
//! bridgewars and its deathmatch so 1v1v1v1) ... Everyone is frozen in
//! place, and before every certain time interval like 10 second, you choose
//! the direction and intensity that you wanna slide"): what a table of it
//! plays (`tables::Rules`), on `iceisland.glb`.
//!
//! For now a stand-in, so that a table can be played from the join to the
//! town again: everyone waits and plays on their spot of the ice, 7 m out
//! on the diagonals, and after [`STAND_IN`] seconds the game is over, seat
//! 0 first and the rest in seat order. The game's own rules
//! (`roundtown_net::knock_off`) take its place, in this file.

use bevy::prelude::*;
use roundtown_net::{Ask, Venue, arrival, knock_off};

use crate::tables::{Ctx, Rules, Table, yaw_of};

/// How long the stand-in's game lasts, in seconds.
const STAND_IN: f32 = 20.0;

/// A game of Knock Off at its table.
pub(crate) struct KnockOff;

impl KnockOff {
    /// The game at a new table, numbered as `table` is, its dice thrown from
    /// `seed`.
    pub(crate) fn new(_table: &Table, _seed: u64) -> Self {
        Self
    }
}

impl Rules for KnockOff {
    fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
        let (at, turn) = knock_off::arrival(seat);
        (at, yaw_of(turn))
    }

    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        for seat in 0..table.seats.len() {
            let (at, turn) = arrival(Venue::KnockOff, seat);
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
        // Fallen off the ice: back where the seat stands, as a fresh arrival
        // there.
        if let Ask::Respawn = ask {
            let (at, yaw) = if table.playing() || table.over() {
                let (at, turn) = arrival(Venue::KnockOff, seat);
                (at, yaw_of(turn))
            } else {
                self.waiting_spot(table, seat)
            };
            ctx.place(table, seat, at, yaw);
        }
    }

    fn show(&mut self, _commands: &mut Commands, _table: &Table) {}

    fn close(&mut self, _commands: &mut Commands) {}
}
