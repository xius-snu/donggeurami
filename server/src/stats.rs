//! What the server logs once a minute while anyone is on: how many are
//! playing and where, and the bytes going out and coming in, all told and per
//! player, to set against the estimates in `MULTIPLAYER.md` ("Bandwidth is the
//! bill": 2 KB/s each in a room of eight, 3.2 in one of sixteen).

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_replicon::prelude::*;

use crate::players::Body;
use crate::rooms::{Room, Towns};

const EVERY: Duration = Duration::from_secs(60);

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, report.run_if(on_timer(EVERY)));
}

fn report(clients: Query<&ConnectedClientStats>, bodies: Query<&Body>, towns: Res<Towns>) {
    let connected = clients.iter().count();
    if connected == 0 {
        return;
    }
    let (out, came_in) = clients.iter().fold((0.0, 0.0), |(out, came_in), stats| {
        (out + stats.sent_bps, came_in + stats.received_bps)
    });
    let homes = bodies
        .iter()
        .filter(|body| matches!(body.room, Room::Home(_)))
        .count();
    let gaming = bodies
        .iter()
        .filter(|body| body.client.is_some() && matches!(body.room, Room::Game(_)))
        .count();
    let each = out / connected as f64;
    info!(
        "{connected} connected: town channels {:?}, {homes} at home, {gaming} in House Builder; \
         out {:.1} KB/s ({:.2} KB/s, {:.1} MB an hour, each), in {:.1} KB/s",
        towns.counts(),
        out / 1000.0,
        each / 1000.0,
        each * 3600.0 / 1e6,
        came_in / 1000.0,
    );
}
