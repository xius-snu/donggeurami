//! Bridge Race played on this device, offline (`super`): the rules' own
//! game (`roundtown_net::bridge_race::Game`), the very one the server plays
//! online, stepped here once its table is played, fed what you do and
//! carried out on this device as the server carries it out for the room
//! (`server/src/bridge_race.rs`). The same game either way: the wrappers
//! differ, the rules do not.
//!
//! * **In**: where you stand, each frame; every rung of your own bridges as
//!   they are fixed (`bridge::Bridges`, which lays them under your walk as
//!   in Bridge Wars); a punch of yours that landed on one of the
//!   computer's players (`crate::Landed`); a stick of dynamite thrown at
//!   its button (`super::use_sticks`); and your fall into the void
//!   (`super::reap`).
//! * **Out**: the computer's players' bodies where the game has them, each
//!   with a plank in its arms while it lays its bridge, hidden while it is
//!   dead; their bridges, a rung at a time, into the bridges as the
//!   server's rungs go (`Bridges::lay_remote`); every stick drawn as it is
//!   thrown (`dynamite::throw`), and blown where the game blows it, the
//!   planks it reaches gone and no more, with its ball of fire; a knock on
//!   you, carrying you off as a punch does; standing you on your island
//!   again after a fall; and the chat's lines (`super::feed_line`). The
//!   round, the kits and the end the game says are followed as the
//!   server's are (`super::follow`).

use std::collections::HashMap;

use bevy::prelude::*;
use roundtown_net::bridge_race::{self as rules, Event, Game};
use roundtown_net::bridge_wars::{DYNAMITE_RADIUS, Lob};

use super::{Racing, feed_line};
use crate::bridge::{self, Bridges, Rung};
use crate::chat;
use crate::dynamite::{self, Lies, Stick};
use crate::fire;
use crate::island::Venue;
use crate::lobby::{self, Whereabouts};
use crate::table::{ComputerSeat, TableView};
use crate::{Carrying, Dead, Intent, Landed, OrbitCamera, Player, Punching};

/// A game of Bridge Race this device plays itself: the game, from the
/// moment its table is played; each stick of dynamite drawn, by the game's
/// number for it; and the bridges' count as last read, the rungs of your
/// own fixed since then being told to the game.
pub(super) struct Played {
    pub(super) game: Game,
    drawn: HashMap<u32, Entity>,
    heard: Option<u32>,
}

impl Played {
    /// The game at a table whose seats are `people`'s (false for the
    /// computer's), begun, its dice thrown from `seed`.
    pub(super) fn new(people: &[bool], seed: u64) -> Self {
        let mut game = Game::new(people.len(), people, seed);
        game.begin();
        Self {
            game,
            drawn: HashMap::new(),
            heard: None,
        }
    }
}

/// The turn about the upright a body facing as `turn` has, as the game
/// keeps a yaw: -Z turned by it is the way it faces.
fn yaw_of(turn: Quat) -> f32 {
    let ahead = turn * Vec3::NEG_Z;
    (-ahead.x).atan2(-ahead.z)
}

/// A frame of the game played here, before anyone moves: your punch that
/// landed this frame, where you stand and your new rungs into it; a step of
/// it; what came of it carried out; and the computer's players where it has
/// them.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn play(
    time: Res<Time>,
    mut racing: ResMut<Racing>,
    view: Res<TableView>,
    mut landed: MessageReader<Landed>,
    me: Query<(Entity, &Intent, Has<Dead>), With<Player>>,
    computers: Query<(Entity, &ComputerSeat, Has<Dead>, Has<Carrying>)>,
    mut bodies: Whereabouts,
    mut sticks: Query<&mut Stick>,
    mut bridges: ResMut<Bridges>,
    mut orbit: ResMut<OrbitCamera>,
    mut log: ResMut<chat::Log>,
    mut commands: Commands,
) {
    let Some(race) = racing.0.as_mut().filter(|race| !race.online && race.begun) else {
        landed.clear();
        return;
    };
    let mine = race.seat;
    let Some(played) = race.played.as_mut() else {
        landed.clear();
        return;
    };
    let Ok((you, intent, dead)) = me.single() else {
        landed.clear();
        return;
    };
    let seat_of: HashMap<Entity, usize> = computers.iter().map(|(body, seat, ..)| (body, usize::from(seat.0))).collect();
    let body_of = |seat: usize| seat_of.iter().find(|(_, of)| **of == seat).map(|(body, _)| *body);
    // Your punches that landed this frame on one of the computer's players
    // (`start_punch`): the game's to take, or not, and to knock it by.
    let running = intent.walk.length() >= 0.5;
    for hit in landed.read() {
        if hit.by == you
            && let Some(&victim) = seat_of.get(&hit.victim)
        {
            played.game.punch(mine, victim, running);
        }
    }
    // Where you stand, and every rung of yours fixed since the bridges were
    // last read (one the game has already is nothing new to it).
    if let Ok((_, at, &venue, ..)) = bodies.get(you)
        && venue == Venue::BridgeRace
    {
        played.game.sync(mine, at.translation, yaw_of(at.rotation));
    }
    if played.heard != Some(bridges.version) {
        played.heard = Some(bridges.version);
        let yours: Vec<(u16, u16, Vec2)> = bridges
            .laid()
            .filter(|&(seat, ..)| usize::from(seat) == mine)
            .map(|(_, chain, index, at)| (chain, index, at))
            .collect();
        for (chain, index, at) in yours {
            played.game.rung(mine, chain, index, at, 0.0);
        }
    }
    for event in played.game.step(time.delta_secs()) {
        if let Some(said) = rules::feed(&event) {
            feed_line(&mut log, &view, said);
        }
        match event {
            // The computer's rungs, as the server's come online; yours the
            // bridges have already.
            Event::Plank(plank) if usize::from(plank.seat) != mine => {
                let rung = Rung {
                    p: Vec2::new(plank.x, plank.z),
                    d: bridge::heading_of(plank.heading),
                };
                bridges.lay_remote(plank.seat, plank.chain, plank.index, rung);
            }
            Event::Knocked { victim, by, from, running } => {
                if victim == mine
                    && let Ok((_, at, _, mut jump, .., Some(mut shove))) = bodies.get_mut(you)
                {
                    let here = at.translation.xz();
                    shove.knock(&mut jump, from.xz(), here, running);
                }
                // One of the computer's punches, from where it stands: its
                // arm swings.
                if by != mine
                    && played.game.at(by).0.xz().distance(from.xz()) < 0.1
                    && let Some(body) = body_of(by)
                {
                    commands.entity(body).insert(Punching::next());
                }
            }
            Event::Thrown { stick, shown } => {
                let lob = Lob {
                    from: Vec3::new(shown.x, shown.y, shown.z),
                    to: Vec3::new(shown.to_x, shown.to_y, shown.to_z),
                    age: 0.0,
                };
                let drawn = dynamite::throw(&mut commands, lob, usize::from(shown.seat), None, Venue::BridgeRace);
                played.drawn.insert(stick, drawn);
            }
            // Come down where the game has no ground for it: on into the
            // void, whatever this device makes of where it came down.
            Event::Lost { stick } => {
                if let Some(drawn) = played.drawn.remove(&stick)
                    && let Ok(mut lying) = sticks.get_mut(drawn)
                {
                    match lying.lies {
                        Lies::Lying => lying.lies = Lies::Falling(2.0),
                        Lies::Flying => lying.served = true,
                        Lies::Falling(_) => {}
                    }
                }
            }
            Event::Blew { stick, at, planks, .. } => {
                bridges.break_planks(&planks);
                bridges.blast_tip(at.xz(), DYNAMITE_RADIUS);
                fire::burst(&mut commands, at + Vec3::Y * 0.3, Venue::BridgeRace);
                if let Some(drawn) = played.drawn.remove(&stick) {
                    commands.entity(drawn).try_despawn();
                }
            }
            Event::Back { seat, at, yaw } => {
                let stood = Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw));
                if seat == mine {
                    lobby::put(&mut bodies, &mut orbit, you, Venue::BridgeRace, stood);
                    commands.entity(you).remove::<Dead>();
                } else if let Some(body) = body_of(seat) {
                    lobby::put(&mut bodies, &mut orbit, body, Venue::BridgeRace, stood);
                }
            }
            Event::Plank(_) | Event::Granted { .. } | Event::Fell { .. } | Event::Holding { .. } | Event::Over { .. } => {}
        }
    }
    // Over with you still in the void: the game stands nobody up after its
    // end, and you stand on your island's middle at once.
    if dead && played.game.result().is_some() {
        let (at, yaw) = rules::home(mine);
        let stood = Transform::from_translation(at).with_rotation(Quat::from_rotation_y(yaw));
        lobby::put(&mut bodies, &mut orbit, you, Venue::BridgeRace, stood);
        commands.entity(you).remove::<Dead>();
    }
    for (body, seat, dead, carrying) in &computers {
        let seat = usize::from(seat.0);
        let (at, yaw) = played.game.at(seat);
        if let Ok((_, mut transform, _, mut jump, .., shove)) = bodies.get_mut(body) {
            // What a punch did to it here is the game's to carry it by
            // (`start_punch` knocks every body this device moves): undone,
            // and it reels no longer than the game has it.
            if let Some(mut shove) = shove
                && (shove.across != Vec2::ZERO || shove.hurt != 0.0)
            {
                shove.across = Vec2::ZERO;
                shove.hurt = 0.0;
            }
            if jump.velocity_y != 0.0 {
                jump.velocity_y = 0.0;
            }
            if transform.translation != at {
                transform.translation = at;
            }
            let turn = Quat::from_rotation_y(yaw);
            if transform.rotation != turn {
                transform.rotation = turn;
            }
        }
        let standing = played.game.standing(seat);
        if !standing && !dead {
            commands.entity(body).insert(Dead { left: f32::INFINITY });
        } else if standing && dead {
            commands.entity(body).remove::<Dead>();
        }
        let laying = played.game.carrying(seat) && standing;
        if laying != carrying {
            if laying {
                commands.entity(body).insert(Carrying);
            } else {
                commands.entity(body).remove::<Carrying>();
            }
        }
    }
}
