//! Bridge Wars' Player Compass Tracker (Hajun's `bridgewarsgameevent.txt`,
//! 2026-11-12: "one time per life purchase - lost on death, which is a non
//! slot filling item that basically shows a small white arrow around your
//! character that is pointing at the direction of the nearest nonteammate
//! player + name (color of team) + '123m'").
//!
//! Bought at the item shop for 2 amethyst (`roundtown_net::Ware::Compass`;
//! whether you have it is your kit's, `arena::Outfit`), it is an arrow on
//! the screen a little way out from you, pointing the way whoever of
//! another team is nearest stands, with their name by it in their team's
//! colour and how far off they are in metres. Yours alone: nobody else
//! sees it, and nothing of it is sent. It points at nobody who is unseen
//! (an invisibility potion hides them from it, as from everything), dead,
//! or a zombie, which is on nobody's team and no player. Nobody to point
//! at, it is put away; and with whoever it points at close by, in plain
//! sight, the words are put away first and then the arrow (`WORDS_FROM`,
//! `ARROW_FROM`).
//!
//! The arrow and the words are two nodes over the world, under the
//! buttons, kept where they belong each frame once the camera is placed,
//! and written only as they change.

use bevy::prelude::*;

use crate::arena::{Fighting, Outfit, Wears, Zombie};
use crate::hud::{self, Picture, Pictures};
use crate::island::Venue;
use crate::ui::{UiChanged, ui};
use crate::{Dead, Player, Team, ThirdPersonCamera, WalkCycle};

/// Sizes, as shares of the short side of the screen: how big the arrow
/// is, how far out from you its middle is (about two metres of the map,
/// under the game's camera: clear of your body and of the punch's range
/// drawn in front of you), how far past the arrow's tip the words begin,
/// and how big they are, the names over heads' size. The words are put
/// past the arrow by however much of them lies that way ([`past`]): for
/// their first day their middle was a fixed 15.5 out, which up and down
/// the screen cleared the arrow and across it did not, the words being
/// much longer than they are tall ("Dori 76m" written over the arrow,
/// seen in a scripted run).
const ARROW: f32 = 4.2;
const ARROW_OUT: f32 = 10.0;
const WORDS_GAP: f32 = 1.0;
const WORDS: f32 = 2.6;
/// How high over your feet the point is that the arrow is drawn round, in
/// metres: about your middle.
const ROUND: f32 = 0.9;
/// How far off whoever it points at has to be, in metres, for the words to
/// be written and for the arrow to be drawn at all. Nearer than the first
/// they are on the screen with their name over their head, which the words
/// would lie over (seen in a scripted run: "Yeppi 2m" written across two
/// names); nearer than the second the arrow, two metres out from you, would
/// lie on them. Each comes back `SLACK` further off than it went, so that
/// neither flickers with someone at that very distance. Mine.
const WORDS_FROM: f32 = 6.0;
const ARROW_FROM: f32 = 3.0;
const SLACK: f32 = 0.75;

/// The arrow; the words by it; and of those, whose name it is, and how far
/// off they are.
#[derive(Component)]
struct Needle;
#[derive(Component)]
struct Words;
#[derive(Component)]
struct Whose;
#[derive(Component)]
struct HowFar;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, (point.after(crate::follow_camera), recolour));
}

fn setup(mut images: ResMut<Assets<Image>>, mut pictures: ResMut<Pictures>, mut commands: Commands) {
    let colour = ui().colours.arena.compass;
    commands.spawn((
        Needle,
        Name::new("Compass arrow"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::ZERO,
            top: Val::ZERO,
            width: Val::VMin(ARROW),
            height: Val::VMin(ARROW),
            ..default()
        },
        ImageNode::new(pictures.of(&mut images, Picture::Compass)).with_color(colour),
        UiTransform::default(),
        Visibility::Hidden,
        GlobalZIndex(-1),
        Pickable::IGNORE,
    ));
    commands
        .spawn((
            Words,
            Name::new("Compass words"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: Val::ZERO,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::VMin(0.9),
                ..default()
            },
            UiTransform::default(),
            Visibility::Hidden,
            GlobalZIndex(-1),
            Pickable::IGNORE,
        ))
        .with_children(|words| {
            words.spawn((Whose, hud::words("", WORDS, colour)));
            words.spawn((HowFar, hud::words("", WORDS, colour)));
        });
}

/// The arrow and the metres take the file's colour as it changes.
fn recolour(
    changed: Res<UiChanged>,
    mut needles: Query<&mut ImageNode, With<Needle>>,
    mut far: Query<&mut TextColor, With<HowFar>>,
) {
    if !changed.is_changed() || changed.is_added() {
        return;
    }
    let colour = ui().colours.arena.compass;
    for mut needle in &mut needles {
        needle.color = colour;
    }
    for mut ink in &mut far {
        ink.0 = colour;
    }
}

/// Whom the compass points at, of `others` (where each stands, their team
/// and whether they are to be pointed at at all), for someone of `team`
/// standing `here`: the nearest across the ground of another team, and how
/// far off. Which of them, by their place in `others`.
pub(crate) fn nearest(here: Vec3, team: Option<Team>, others: impl IntoIterator<Item = (Vec3, Team, bool)>) -> Option<(usize, f32)> {
    others
        .into_iter()
        .enumerate()
        .filter(|(_, (_, theirs, tracked))| *tracked && Some(*theirs) != team)
        .map(|(which, (at, ..))| (which, at.xz().distance(here.xz())))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// What is written of how far off they are: whole metres.
pub(crate) fn metres(away: f32) -> String {
    format!("{}m", away.round().max(0.0) as u32)
}

/// How far from the middle of a box of `size` its edge is, going `way`
/// from it, as the words' box lies along the arrow's way: half its width
/// across the screen, half its height up and down it, and between the
/// two at a slant. What keeps the words clear of the arrow whichever way
/// it points.
fn past(way: Vec2, size: Vec2) -> f32 {
    0.5 * (way.x.abs() * size.x + way.y.abs() * size.y)
}

/// Whether what is shown only of someone `from` metres off or further is
/// shown of someone `away`: from there while it is `shown`, and from
/// [`SLACK`] further once it is not.
fn far_enough(away: f32, from: f32, shown: bool) -> bool {
    away >= if shown { from } else { from + SLACK }
}

/// Keeps the arrow pointing at whoever of another team is nearest, a
/// little way out from you that way, with their name and how far off they
/// are just beyond it; and puts both away when you have no compass, are
/// dead or out of a game, or there is nobody to point at.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn point(
    fighting: Res<Fighting>,
    outfit: Res<Outfit>,
    me: Query<(&Transform, &Venue, Option<&Team>, Has<Dead>), With<Player>>,
    others: Query<(&Transform, &Venue, &Team, &Name, Has<Dead>, Has<Zombie>, Option<&Wears>), (With<WalkCycle>, Without<Player>)>,
    cameras: Query<(&Camera, &Transform), With<ThirdPersonCamera>>,
    windows: Query<&Window>,
    mut needles: Query<(&mut Visibility, &mut UiTransform, &ComputedNode), (With<Needle>, Without<Words>)>,
    mut words: Query<(&mut Visibility, &mut UiTransform, &ComputedNode), (With<Words>, Without<Needle>)>,
    mut whose: Query<(&mut Text, &mut TextColor), (With<Whose>, Without<HowFar>)>,
    mut far: Query<&mut Text, (With<HowFar>, Without<Whose>)>,
) {
    let (Ok((mut needle_shown, mut needle_at, needle_node)), Ok((mut words_shown, mut words_at, words_node))) =
        (needles.single_mut(), words.single_mut())
    else {
        return;
    };
    let mut put_away = || {
        needle_shown.set_if_neq(Visibility::Hidden);
        words_shown.set_if_neq(Visibility::Hidden);
    };
    if !fighting.playing || !outfit.kit.compass {
        return put_away();
    }
    let (Ok((you, venue, team, dead)), Ok((camera, eye)), Ok(window)) = (me.single(), cameras.single(), windows.single()) else {
        return put_away();
    };
    if *venue != Venue::Arena || dead {
        return put_away();
    }
    let here = you.translation;
    let seen: Vec<(Vec3, Team, bool, &Name)> = others
        .iter()
        .filter(|(_, venue, ..)| **venue == Venue::Arena)
        .map(|(at, _, team, name, dead, zombie, wears)| {
            let tracked = !dead && !zombie && !wears.is_some_and(|wears| wears.unseen);
            (at.translation, *team, tracked, name)
        })
        .collect();
    let Some((which, away)) = nearest(here, team.copied(), seen.iter().map(|(at, team, tracked, _)| (*at, *team, *tracked))) else {
        return put_away();
    };
    let (there, theirs, _, name) = seen[which];
    // Which way that is on the screen, whichever way the camera is
    // turned: from where you are drawn to where a metre that way is.
    let eye = GlobalTransform::from(*eye);
    let middle = here + Vec3::Y * ROUND;
    let toward = (there - here).with_y(0.0).normalize_or(Vec3::NEG_Z);
    let (Ok(from), Ok(to)) = (camera.world_to_viewport(&eye, middle), camera.world_to_viewport(&eye, middle + toward)) else {
        return put_away();
    };
    let way = (to - from).normalize_or(Vec2::NEG_Y);
    let vmin = window.width().min(window.height()) / 100.0;
    // Close by, they are in plain sight: no words over their name, and,
    // closer still, no arrow lying on them.
    let arrow_on = far_enough(away, ARROW_FROM, *needle_shown != Visibility::Hidden);
    let words_on = far_enough(away, WORDS_FROM, *words_shown != Visibility::Hidden);
    // The arrow's picture points up the screen: turned to point that way.
    let needle_size = needle_node.size() * needle_node.inverse_scale_factor();
    let needle_middle = from + way * ARROW_OUT * vmin;
    needle_at.set_if_neq(UiTransform {
        translation: Val2::px(needle_middle.x - needle_size.x * 0.5, needle_middle.y - needle_size.y * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::radians(way.x.atan2(-way.y)),
    });
    needle_shown.set_if_neq(if arrow_on { Visibility::Inherited } else { Visibility::Hidden });
    // Their name, in their team's colour, and how far off they are.
    if let Ok((mut text, mut ink)) = whose.single_mut() {
        if text.0 != name.as_str() {
            text.0 = name.as_str().to_string();
        }
        let colour = ui().colours.arena.team(theirs.0);
        if ink.0 != colour {
            ink.0 = colour;
        }
    }
    if let Ok(mut text) = far.single_mut() {
        let written = metres(away);
        if text.0 != written {
            text.0 = written;
        }
    }
    let words_size = words_node.size() * words_node.inverse_scale_factor();
    let words_middle = from + way * ((ARROW_OUT + ARROW * 0.5 + WORDS_GAP) * vmin + past(way, words_size));
    words_at.set_if_neq(UiTransform {
        translation: Val2::px(words_middle.x - words_size.x * 0.5, words_middle.y - words_size.y * 0.5),
        scale: Vec2::ONE,
        rotation: Rot2::IDENTITY,
    });
    words_shown.set_if_neq(if words_on { Visibility::Inherited } else { Visibility::Hidden });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_compass_points_at_the_nearest_of_another_team_who_is_to_be_seen() {
        let here = Vec3::new(0.0, 0.25, 130.0);
        let mine = Some(Team(0));
        let others = [
            // A teammate beside you; someone unseen, close by; a zombie's
            // like, not tracked; and two of other teams further off.
            (Vec3::new(2.0, 0.25, 130.0), Team(0), true),
            (Vec3::new(0.0, 0.25, 125.0), Team(2), false),
            (Vec3::new(0.0, 0.25, 90.0), Team(1), true),
            (Vec3::new(30.0, 40.0, 130.0), Team(3), true),
        ];
        // The nearest across the ground: how high they are is no matter.
        assert_eq!(nearest(here, mine, others), Some((3, 30.0)));
        // With them gone, the next.
        assert_eq!(nearest(here, mine, others[..3].iter().copied()), Some((2, 40.0)));
        // Nobody but your own team, or nobody to be seen: nobody.
        assert_eq!(nearest(here, mine, others[..2].iter().copied()), None);
        assert_eq!(nearest(here, mine, []), None);
        // On no team, everyone to be seen is another's.
        assert_eq!(nearest(here, None, others), Some((0, 2.0)));
        // Whole metres.
        assert_eq!(metres(122.6), "123m");
        assert_eq!(metres(0.2), "0m");
    }

    #[test]
    fn close_by_the_words_and_then_the_arrow_are_put_away_and_come_back_a_little_further_off() {
        // Far off, both; within the words' distance, the arrow alone;
        // within the arrow's, neither.
        assert!(far_enough(40.0, WORDS_FROM, true) && far_enough(40.0, ARROW_FROM, true));
        assert!(!far_enough(WORDS_FROM - 0.1, WORDS_FROM, true) && far_enough(WORDS_FROM - 0.1, ARROW_FROM, true));
        assert!(!far_enough(ARROW_FROM - 0.1, ARROW_FROM, true));
        // Put away, each comes back only that much further off.
        assert!(!far_enough(WORDS_FROM + 0.1, WORDS_FROM, false));
        assert!(far_enough(WORDS_FROM + SLACK, WORDS_FROM, false));
        // The words go before the arrow does, and the arrow goes before it
        // would lie on a body beside you.
        const { assert!(WORDS_FROM > ARROW_FROM + SLACK && ARROW_FROM > 2.0) };
    }

    #[test]
    fn the_words_are_clear_of_the_arrow_whichever_way_it_points() {
        // Words eight times as long as they are tall, as a name and a
        // distance are: their near edge is the gap past the arrow's tip
        // across the screen and up and down it alike, where a fixed
        // distance out had them over the arrow across it.
        let size = Vec2::new(160.0, 20.0);
        assert_eq!(past(Vec2::X, size), 80.0);
        assert_eq!(past(Vec2::NEG_X, size), 80.0);
        assert_eq!(past(Vec2::Y, size), 10.0);
        assert_eq!(past(Vec2::NEG_Y, size), 10.0);
        let slant = Vec2::new(1.0, -1.0).normalize();
        assert!(past(slant, size) > 10.0 && past(slant, size) < 80.0);
        // And nothing of nothing: the frame before they are laid out.
        assert_eq!(past(Vec2::X, Vec2::ZERO), 0.0);
    }
}
