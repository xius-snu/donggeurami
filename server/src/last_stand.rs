//! Last Stand at its tables, on the server (since 2026-11-18; Hajun: "Coop
//! (2~4 players) Last Stand (Endless Zombie survival game). You are in a
//! circular map, with a spawn anchor crystal in the center, and 2 generators
//! and item shops/upgrade shops"): what a table of it plays
//! (`tables::Rules`), on `laststandmap.glb`, everyone on one side.
//!
//! For now a stand-in, so that a table can be played from the join to the
//! town again: everyone waits in the glass room over the map and plays
//! round the anchor, and after [`STAND_IN`] seconds the game is over, seat
//! 0 first and the rest in seat order, two waves cleared. The game's own
//! rules (`roundtown_net::last_stand`) take its place, in this file.

use bevy::prelude::*;
use roundtown_net::{Ask, Venue, arrival, last_stand};

use crate::tables::{Ctx, Rules, Table, facing, yaw_of};

/// How long the stand-in's game lasts, in seconds, and how many waves it
/// says were cleared.
const STAND_IN: f32 = 20.0;
const STAND_IN_WAVES: u32 = 2;

/// A game of Last Stand at its table.
pub(crate) struct LastStand;

impl LastStand {
    /// The game at a new table, numbered as `table` is, its dice thrown from
    /// `seed`.
    pub(crate) fn new(_table: &Table, _seed: u64) -> Self {
        Self
    }
}

impl Rules for LastStand {
    /// On a ring round the middle of the glass room, facing in.
    fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
        let at = last_stand::waiting_spot(seat);
        (at, facing(at.xz(), last_stand::WAITING_ROOM))
    }

    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        for seat in 0..table.seats.len() {
            let (at, turn) = arrival(Venue::LastStand, seat);
            ctx.place(table, seat, at, yaw_of(turn));
        }
    }

    fn step(&mut self, _ctx: &mut Ctx, table: &mut Table, _dt: f32) {
        if table.elapsed() >= STAND_IN {
            let seats = table.seats.len();
            table.finish((0..seats as u8).collect(), vec![0; seats], Some(STAND_IN_WAVES));
        }
    }

    fn hear(&mut self, ctx: &mut Ctx, table: &mut Table, seat: usize, ask: &Ask) {
        // Fallen into the void: back where the seat stands, as a fresh
        // arrival there.
        if let Ask::Respawn = ask {
            let (at, yaw) = if table.playing() || table.over() {
                let (at, turn) = arrival(Venue::LastStand, seat);
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
