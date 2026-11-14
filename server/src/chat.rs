//! What is said in a room (since 2026-11-14; Hajun: "I think there should be
//! a global chat in the Bridge Wars game ... This chat box should also be in
//! the main lobby ... everyone in the lobby should see the message with your
//! name tag followed by your text, like chat on a Minecraft server").
//!
//! A device says a line ([`Ask::Chat`]); if it is one ([`chat::line`]:
//! short, and of what the game writes, English, Korean and Vietnamese, as
//! `roundtown_net::text` has it) and its speaker has not said too many too fast
//! ([`chat::lines_left`]), everyone connected in the speaker's room is told
//! it, the speaker among them ([`Tell::Said`]): a channel of the town, a
//! game of Bridge Wars from its waiting room to its end, or a game of House
//! Builder. Nothing is kept of it, here or in the database, and what is
//! said is not written to the log. What happens in a game of Bridge Wars is
//! said in the same place, by the game itself (`arena`, [`Tell::Feed`]).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use roundtown_net::{Ask, Member, Tell, chat};

use crate::players::{Body, Plays, tell};

/// How many lines each body may still say, and when that was worked out, in
/// the server's seconds ([`chat::lines_left`]).
#[derive(Resource, Default)]
struct Talkers(HashMap<Entity, (f32, f64)>);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Talkers>().add_systems(
        PreUpdate,
        hear.after(ServerSystems::Receive).run_if(in_state(ServerState::Running)),
    );
}

/// Whether someone who had `talker` lines to say, as of then, may say one
/// at `now`, and what they have left from now: a line taken if so.
fn takes_a_line(talker: Option<(f32, f64)>, now: f64) -> (bool, (f32, f64)) {
    let (had, when) = talker.unwrap_or((chat::BURST, now));
    let left = chat::lines_left(had, (now - when) as f32);
    if left < 1.0 {
        (false, (left, now))
    } else {
        (true, (left - 1.0, now))
    }
}

/// Passes each line said on to everyone in its speaker's room.
fn hear(
    mut asks: MessageReader<FromClient<Ask>>,
    clients: Query<&Plays>,
    bodies: Query<(&Body, &Member)>,
    time: Res<Time>,
    mut talkers: ResMut<Talkers>,
    mut tells: MessageWriter<ToClients<Tell>>,
) {
    let now = time.elapsed_secs_f64();
    for ask in asks.read() {
        let Ask::Chat(text) = &ask.message else {
            continue;
        };
        let Some(client) = ask.client_id.entity() else {
            continue;
        };
        let Ok(&Plays(body)) = clients.get(client) else {
            continue;
        };
        let Ok((speaker, member)) = bodies.get(body) else {
            continue;
        };
        let Some(text) = chat::line(text) else {
            continue;
        };
        let (may, left) = takes_a_line(talkers.0.get(&body).copied(), now);
        talkers.0.insert(body, left);
        if !may {
            debug!("{} is saying too much too fast", member.id);
            continue;
        }
        let room = speaker.room;
        let mut heard = 0;
        for (other, _) in &bodies {
            if other.room == room
                && let Some(device) = other.client
            {
                tell(
                    &mut tells,
                    device,
                    Tell::Said {
                        who: member.id,
                        name: member.name.clone(),
                        text: text.clone(),
                    },
                );
                heard += 1;
            }
        }
        debug!("{} said a line in {room:?}, to {heard}", member.id);
    }
    // Whoever has gone is forgotten, once there are enough of them to be
    // worth the looking.
    if talkers.0.len() > 256 {
        talkers.0.retain(|body, _| bodies.contains(*body));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_few_lines_at_once_and_then_one_every_so_often() {
        // Nobody has said anything: a burst of lines, one after another.
        let mut talker = None;
        let mut said = 0;
        for _ in 0..10 {
            let (may, left) = takes_a_line(talker, 100.0);
            talker = Some(left);
            said += u32::from(may);
        }
        assert_eq!(said as f32, chat::BURST);
        // And no more until one has come round again.
        let (may, left) = takes_a_line(talker, 100.0 + f64::from(chat::EVERY) * 0.9);
        assert!(!may);
        let (may, left) = takes_a_line(Some(left), 100.0 + f64::from(chat::EVERY) * 1.05);
        assert!(may, "{left:?}");
        let (may, _) = takes_a_line(Some(left), 100.0 + f64::from(chat::EVERY) * 1.1);
        assert!(!may);
        // Left alone long enough, the whole burst is theirs again, and no
        // more than that.
        let mut talker = Some(left);
        let mut said = 0;
        for _ in 0..10 {
            let (may, left) = takes_a_line(talker, 10_000.0);
            talker = Some(left);
            said += u32::from(may);
        }
        assert_eq!(said as f32, chat::BURST);
    }
}
