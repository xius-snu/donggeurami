//! Bridge Race at its tables, on the server (since 2026-11-18; Hajun: "Like
//! King of the Ladder but with bridge building ... Your goal is to reach the
//! golden island first compared to the other team - both of u need to make
//! it across cuz its 2v2"): what a table of it plays (`tables::Rules`), on
//! `bridgeracemap.glb`, two teams by seat (`roundtown_net::bridge_race::team`).
//!
//! Its seats are taken two by two onto a team ([`BridgeRace::seats_for`]),
//! so that people are teamed with each other first and a group goes in
//! together. The rest is for now a stand-in, so that a table can be played
//! from the join to the town again: everyone waits in the glass room over
//! the map and plays on their team's island, and after [`STAND_IN`] seconds
//! the game is over, seat 0 first and the rest in seat order. The game's
//! own rules (`roundtown_net::bridge_race`) take its place, in this file.

use bevy::prelude::*;
use roundtown_net::{Ask, Venue, arrival, bridge_race};

use crate::tables::{Ctx, Rules, Table, facing, yaw_of};

/// How long the stand-in's game lasts, in seconds.
const STAND_IN: f32 = 20.0;

/// A game of Bridge Race at its table.
pub(crate) struct BridgeRace;

impl BridgeRace {
    /// The game at a new table, numbered as `table` is, its dice thrown from
    /// `seed`.
    pub(crate) fn new(_table: &Table, _seed: u64) -> Self {
        Self
    }
}

impl Rules for BridgeRace {
    /// On a ring round the middle of the glass room, facing in.
    fn waiting_spot(&mut self, _table: &Table, seat: usize) -> (Vec3, f32) {
        let at = bridge_race::waiting_spot(seat);
        (at, facing(at.xz(), bridge_race::WAITING_ROOM))
    }

    /// Two by two onto a team, people with people first (Hajun: "if its 2
    /// players then its a 2v2 against bots"): each pair of a group onto a
    /// team with both its seats free, red first, and an odd one into the
    /// first seat free, in the order people take seats
    /// (`bridge_race::PEOPLE_FIRST`). One alone takes red's first seat,
    /// two red's both, three red's both and blue's first; and a group of
    /// two after someone alone is blue together. As Bridge Wars seats a
    /// group (`arena::Game::seats_for`).
    fn seats_for(&self, table: &Table, count: usize) -> Option<Vec<usize>> {
        let order = table.order();
        let mut seats = Vec::with_capacity(count);
        let mut teams = order.chunks(2);
        for _ in 0..count / 2 {
            let pair = teams.by_ref().find(|pair| pair.len() == 2 && pair.iter().all(|&seat| table.open(seat)))?;
            seats.extend_from_slice(pair);
        }
        if count % 2 == 1 {
            seats.push(order.iter().copied().find(|&seat| table.open(seat) && !seats.contains(&seat))?);
        }
        Some(seats)
    }

    fn begin(&mut self, ctx: &mut Ctx, table: &mut Table) {
        for seat in 0..table.seats.len() {
            let (at, turn) = arrival(Venue::BridgeRace, seat);
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
        // Fallen into the void: back where the seat stands, as a fresh
        // arrival there.
        if let Ask::Respawn = ask {
            let (at, yaw) = if table.playing() || table.over() {
                let (at, turn) = arrival(Venue::BridgeRace, seat);
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
