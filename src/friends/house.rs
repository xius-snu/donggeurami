//! The House tab of the friends' panel (since 2026-11-27; Hajun: "in the
//! social tab, as the 3rd tab after the messages tab, it can be a house tab
//! with a house icon and it has basic info about your house at the top like
//! your house net worth, how many objects you have in your house, how many
//! total visitors you got till now, how many visitors you got in the past 7
//! days, total amount of ratings you got, your total average rating, and
//! then below it it can be a list of the ratings that ppl gave you where u
//! can see their username and the star rating they gave u and the
//! adjectives they chose"; and later the same message: "in the home tab, at
//! the top i actually want another tab selector with 2 tabs (user icon on
//! left, which is the thing i said before of stats and ratings of your own
//! house, and leader board icon on the right). and in the leaderboard icon
//! tab, u can see a global leaderboard of the best rated house that shows
//! the top 100, and if you are in the top 100 ur name is on there, but if
//! ur like top 101+ then it just shows u that ur place is top 101 on your
//! side at the bottom, but not to anyone else ... u can scroll and click
//! each one and u can see the stat & rating of that persons house, similar
//! to how u see your own stat").
//!
//! Everything in it is the server's (`roundtown_net::house`,
//! `server/src/house.rs`), asked for as it is shown ([`ask`]) and drawn as
//! it comes ([`HouseView`]): nothing offline. Along its top, under the
//! panel's own tabs, two more, a person and a leaderboard ([`Sub`]).
//! **The person**: your house's stats, six cards in a row ([`cards`]):
//! what stands on it is worth, at the shop's prices, how many things that
//! is, its visitors ever and in the past week, how many ratings and their
//! stars on average; and under them its ratings, the newest first, each
//! who gave it, its stars and its words. **The leaderboard**: the best
//! houses, a hundred at the most, each its place, whose, its stars on
//! average and how many ratings, your own lit; a press on one shows that
//! house as yours is shown, with a chevron back. Further down than the
//! hundred, your own place is on a row of its own under them, for you
//! alone. The look is the panel's (`window`); the sizes and the words in a
//! row of cards are mine.

use bevy::prelude::*;
use roundtown_net::house::{self, Ask as HouseAsk, Board, Ranked, Rating, Stats, Tell as HouseTell};
use roundtown_net::social::handle;
use roundtown_net::{Ask, Tell};

use super::{List, Panel, Tab, Tap, View, heading, list, note};
use crate::builder::Fill;
use crate::hud::{COIN_ICON, TakesPress};
use crate::net::Online;
use crate::ui::{fill, ui};
use crate::window::{self, HEADING, NAME, PAD, STATUS, picture, words};

/// The two tabs under the panel's own: how tall.
const SUB_HIGH: f32 = 6.6;
const SUB_ICON: f32 = 4.4;
/// The stats' cards: how tall, the room between them, and the size of
/// their numbers.
const CARD_HIGH: f32 = 9.6;
const CARD_GAP: f32 = 1.2;
const CARD_NUMBER: f32 = 3.4;
/// A row of a rating or of the board: how tall, its disc, its place, and
/// the small stars along it.
const ROW_HIGH: f32 = 8.8;
const DISC: f32 = 6.2;
const PLACE_WIDE: f32 = 9.0;
const SMALL_STAR: f32 = 2.9;

/// Which of the House tab's two is shown: your house, or the board.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(super) enum Sub {
    #[default]
    Mine,
    Board,
}

/// What a press on something of the House tab's does.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum HouseTap {
    Show(Sub),
    /// The house of the account `who`, from its row on the board.
    Look(u64),
    /// Back to the board from someone's house.
    Back,
}

/// What the server last said of houses, for the House tab: which of its
/// two is shown, and whose house from the board, if one is; your own
/// house's stats, the one looked at, and the board.
#[derive(Resource, Default, PartialEq, Debug)]
pub(super) struct HouseView {
    pub(super) sub: Sub,
    pub(super) looking: Option<u64>,
    mine: Option<Stats>,
    theirs: Option<Stats>,
    board: Option<Board>,
}

impl HouseView {
    /// What a press does to it.
    pub(super) fn tap(&mut self, tap: HouseTap) {
        match tap {
            HouseTap::Show(sub) => {
                self.sub = sub;
                self.looking = None;
            }
            HouseTap::Look(who) => {
                if self.looking != Some(who) {
                    self.theirs = None;
                }
                self.sub = Sub::Board;
                self.looking = Some(who);
            }
            HouseTap::Back => self.looking = None,
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<HouseView>()
        .add_systems(Update, (hear, ask).chain().before(super::redraw));
}

/// What the server says of houses that is the tab's: a house's stats, and
/// the board. Nothing is kept offline.
fn hear(mut tells: MessageReader<Tell>, online: Res<Online>, mut view: ResMut<HouseView>) {
    let me = online.me().map(|me| me.id);
    for tell in tells.read() {
        let Tell::House(told) = tell else {
            continue;
        };
        match told {
            HouseTell::Stats(stats) if Some(stats.who) == me => view.mine = Some(stats.clone()),
            HouseTell::Stats(stats) if view.looking == Some(stats.who) => view.theirs = Some(stats.clone()),
            HouseTell::Board(board) => view.board = Some(board.clone()),
            _ => {}
        }
    }
    if me.is_none() && (view.mine.is_some() || view.theirs.is_some() || view.board.is_some()) {
        view.mine = None;
        view.theirs = None;
        view.board = None;
    }
}

/// Asks the server for what the tab shows, as it comes to be shown: the
/// panel opened on it, the tab pressed, one of its two, a house on the
/// board, or back.
fn ask(panel: Res<Panel>, view: Res<HouseView>, online: Res<Online>, mut last: Local<Option<(Sub, Option<u64>)>>, mut asks: MessageWriter<Ask>) {
    let showing = (panel.open && panel.tab == Tab::House && online.is()).then_some((view.sub, view.looking));
    if showing == *last {
        return;
    }
    *last = showing;
    let asked = match showing {
        None => return,
        Some((_, Some(who))) => HouseAsk::Stats(Some(who)),
        Some((Sub::Mine, None)) => HouseAsk::Stats(None),
        Some((Sub::Board, None)) => HouseAsk::Board,
    };
    asks.write(Ask::House(asked));
}

/// `count` as the cards write it: with commas to a hundred thousand, and
/// in thousands and millions past that.
fn counted(count: u32) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", f64::from(count) / 1e6)
    } else if count >= 100_000 {
        format!("{}K", count / 1_000)
    } else {
        let digits = count.to_string();
        let mut written = String::new();
        for (index, digit) in digits.chars().enumerate() {
            if index > 0 && (digits.len() - index).is_multiple_of(3) {
                written.push(',');
            }
            written.push(digit);
        }
        written
    }
}

/// What a house is worth as its card writes it (Hajun, 2026-11-27: "number
/// in the net worth thing, make it like 28.2K or 29.3M"): as it is under a
/// thousand, and past that in thousands, millions or billions to a tenth,
/// in whichever keeps it under a thousand once rounded.
fn worth(coins: u32) -> String {
    let coins = f64::from(coins);
    if coins < 1_000.0 {
        return format!("{coins}");
    }
    for (unit, size) in [("K", 1e3), ("M", 1e6)] {
        let tenths = (coins / size * 10.0).round() / 10.0;
        if tenths < 1_000.0 {
            return format!("{tenths:.1}{unit}");
        }
    }
    format!("{:.1}B", (coins / 1e9 * 10.0).round() / 10.0)
}

/// Stars on average as the tab writes them: one figure after the point, a
/// dash with none.
fn stars(count: u32, sum: u32) -> String {
    if count == 0 { "-".into() } else { format!("{:.1}", house::average(count, sum)) }
}

/// The House tab, under the panel's own tabs.
pub(super) fn tab(commands: &mut Commands, assets: &AssetServer, panel: Entity, view: &View) {
    let ui = ui();
    let said = &ui.words.friends;
    let house = view.house;
    sub_tabs(commands, assets, panel, house.sub);
    let Some((me, _)) = view.me else {
        let list = list(commands, panel, List::House, 0.0);
        note(commands, list, &said.not_online);
        return;
    };
    match (house.sub, house.looking) {
        (Sub::Mine, _) => stats_page(commands, assets, panel, view, house.mine.as_ref(), false),
        (Sub::Board, Some(who)) => {
            let stats = if who == me { house.mine.as_ref() } else { house.theirs.as_ref() };
            stats_page(commands, assets, panel, view, stats, true);
        }
        (Sub::Board, None) => board_page(commands, assets, panel, view, me),
    }
}

/// The two tabs along the top of the House tab, each half its width: the
/// person, your house; the leaderboard, the board.
fn sub_tabs(commands: &mut Commands, assets: &AssetServer, panel: Entity, shown: Sub) {
    let colours = ui().colours.window;
    let row = commands
        .spawn((
            ChildOf(panel),
            Node {
                height: Val::VMin(SUB_HIGH),
                flex_shrink: 0.0,
                margin: UiRect::top(Val::VMin(0.8)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for (sub, image) in [(Sub::Mine, "person"), (Sub::Board, "leaderboard")] {
        let lit = sub == shown;
        commands.spawn((
            ChildOf(row),
            Tap::House(HouseTap::Show(sub)),
            TakesPress,
            Button,
            Node {
                flex_grow: 1.0,
                flex_basis: Val::ZERO,
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::bottom(Val::VMin(window::LINE)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(if lit { colours.line } else { colours.tab_off.with_alpha(colours.tab_off.alpha() * 0.35) }),
            children![picture(
                assets.load(format!("icons/{image}.png")),
                Val::VMin(SUB_ICON),
                window::tab_ink(lit),
            )],
        ));
    }
}

/// A house's stats and its ratings: yours, or, `from_the_board`, someone's
/// pressed on it, under a chevron back and their house's name.
fn stats_page(commands: &mut Commands, assets: &AssetServer, panel: Entity, view: &View, stats: Option<&Stats>, from_the_board: bool) {
    let ui = ui();
    let (colours, said) = (ui.colours.window, &ui.words.house);
    if from_the_board {
        let title = stats.map_or_else(String::new, |stats| fill(&said.whose, &[("name", &stats.name)]));
        commands.spawn((
            ChildOf(panel),
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::VMin(1.4),
                flex_shrink: 0.0,
                margin: UiRect::top(Val::VMin(1.0)),
                ..default()
            },
            Pickable::IGNORE,
            children![
                (Tap::House(HouseTap::Back), window::bare(assets.load("icons/back.png"), window::SMALL_BUTTON, true)),
                words(title, window::TITLE, colours.title),
            ],
        ));
    }
    // Over your own house's stats, as a list has its heading (Hajun,
    // 2026-11-27: "in my house stat page, i need text above the stat boxes
    // "My House:""); someone else's has its name and the chevron instead.
    if !from_the_board {
        let head = heading(commands, panel, said.my_house.clone());
        commands.entity(head).insert(Node {
            height: Val::VMin(window::SMALL_BUTTON + 0.8),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            margin: UiRect::top(Val::VMin(0.8)),
            ..default()
        });
    }
    let Some(stats) = stats else {
        let list = list(commands, panel, List::House, 0.0);
        note(commands, list, &said.waiting);
        return;
    };
    cards(commands, assets, panel, view, stats, if from_the_board { 1.4 } else { 0.2 });
    let list = list(commands, panel, List::House, view.scrolled.house);
    heading(commands, list, fill(&said.ratings_heading, &[("count", &stats.count.to_string())]));
    if stats.ratings.is_empty() {
        note(commands, list, &said.no_ratings);
    }
    for rating in &stats.ratings {
        rating_row(commands, assets, list, view, rating);
    }
}

/// A house's six stats, each a card of its own in a row: a number over what
/// it is.
fn cards(commands: &mut Commands, assets: &AssetServer, panel: Entity, view: &View, stats: &Stats, above: f32) {
    let ui = ui();
    let (colours, said) = (ui.colours.window, &ui.words.house);
    let row = commands
        .spawn((
            ChildOf(panel),
            Node {
                column_gap: Val::VMin(CARD_GAP),
                flex_shrink: 0.0,
                margin: UiRect::top(Val::VMin(above)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    let coin = assets.load(COIN_ICON);
    let star = view.star.clone();
    let shown: [(Option<Handle<Image>>, String, &str); 6] = [
        (Some(coin), worth(stats.worth), &said.worth),
        (None, counted(stats.things), &said.things),
        (None, counted(stats.visits), &said.visitors),
        (None, counted(stats.visits_week), &said.visitors_week),
        (None, counted(stats.count), &said.ratings),
        (star, stars(stats.count, stats.sum), &said.average),
    ];
    let star_colour = ui.colours.house.star;
    for (icon, number, what) in shown {
        let card = commands
            .spawn((
                ChildOf(row),
                Node {
                    flex_grow: 1.0,
                    flex_basis: Val::ZERO,
                    height: Val::VMin(CARD_HIGH),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::VMin(0.5),
                    border_radius: BorderRadius::all(Val::VMin(window::CARD_ROUNDED)),
                    ..default()
                },
                BackgroundColor(colours.card),
                Pickable::IGNORE,
            ))
            .id();
        let top = commands
            .spawn((
                ChildOf(card),
                Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::VMin(0.6),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        if let Some(icon) = icon {
            // The coin as it is, the star in its gold.
            let tint = if what == said.average { star_colour } else { Color::WHITE };
            commands.spawn((ChildOf(top), picture(icon, Val::VMin(CARD_NUMBER), tint)));
        }
        commands.spawn((ChildOf(top), words(number, CARD_NUMBER, colours.name)));
        commands.spawn((ChildOf(card), words(what.to_string(), STATUS, colours.status)));
    }
}

/// The disc with someone's picture on it, `size` across, with no dot.
fn disc(assets: &AssetServer, size: f32) -> impl Bundle {
    let colours = ui().colours.window;
    (
        Node {
            width: Val::VMin(size),
            height: Val::VMin(size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(colours.avatar),
        Pickable::IGNORE,
        children![picture(assets.load("icons/person.png"), Val::Percent(64.0), colours.avatar_mark)],
    )
}

/// Five small stars, `halves` of them given, under `parent`.
fn small_stars(commands: &mut Commands, parent: Entity, view: &View, halves: u8) {
    let Some(star) = &view.star else {
        return;
    };
    let row = commands
        .spawn((
            ChildOf(parent),
            Node {
                column_gap: Val::VMin(0.3),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    for number in 0..5u8 {
        crate::rating::spawn_star(commands, row, star, SMALL_STAR, crate::rating::filled(number, halves), None);
    }
}

/// One rating, as a house's list has it: who gave it, its words under their
/// name, and its stars.
fn rating_row(commands: &mut Commands, assets: &AssetServer, list: Entity, view: &View, rating: &Rating) {
    let ui = ui();
    let colours = ui.colours.window;
    let adjectives: Vec<&str> = rating
        .words
        .iter()
        .filter_map(|&word| ui.words.house.adjectives.get(usize::from(word)).map(String::as_str))
        .collect();
    let row = commands
        .spawn((
            ChildOf(list),
            Node {
                min_height: Val::VMin(ROW_HIGH),
                align_items: AlignItems::Center,
                column_gap: Val::VMin(2.0),
                padding: UiRect::vertical(Val::VMin(0.6)),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
            children![
                disc(assets, DISC),
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        flex_basis: Val::ZERO,
                        min_width: Val::ZERO,
                        row_gap: Val::VMin(0.3),
                        overflow: Overflow::clip_x(),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![
                        words(handle(&rating.name, rating.tag), NAME - 0.2, colours.name),
                        words(adjectives.join(" · "), STATUS + 0.2, colours.status),
                    ],
                ),
            ],
        ))
        .id();
    small_stars(commands, row, view, rating.halves);
    commands.spawn((
        ChildOf(row),
        Node {
            width: Val::VMin(6.0),
            margin: UiRect::right(Val::VMin(1.0)),
            justify_content: JustifyContent::FlexEnd,
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
        children![words(format!("{:.1}", f32::from(rating.halves) / 2.0), NAME - 0.2, colours.name)],
    ));
}

/// The board: its heading, the best houses, and under them your own
/// place, if it is further down, or that your house has not been rated.
fn board_page(commands: &mut Commands, assets: &AssetServer, panel: Entity, view: &View, me: u64) {
    let ui = ui();
    let said = &ui.words.house;
    let Some(board) = &view.house.board else {
        let list = list(commands, panel, List::Board, 0.0);
        note(commands, list, &said.waiting);
        return;
    };
    let list = list(commands, panel, List::Board, view.scrolled.board);
    heading(commands, list, fill(&said.board_heading, &[("count", &house::BOARD.to_string())]));
    if board.top.is_empty() {
        note(commands, list, &said.no_board);
    }
    for ranked in &board.top {
        board_row(commands, assets, list, view, ranked, ranked.who == me);
    }
    // Under the list, yours: your place further down, for you alone; or
    // that your house has not been rated yet, with it nowhere on the board.
    if let Some(mine) = &board.mine {
        let foot = foot(commands, panel);
        board_row(commands, assets, foot, view, mine, true);
    } else if !board.top.iter().any(|ranked| ranked.who == me) && view.house.mine.as_ref().is_none_or(|stats| stats.count == 0) {
        let foot = foot(commands, panel);
        note(commands, foot, &said.unrated);
    }
}

/// The room under the board's list where your own place stands.
fn foot(commands: &mut Commands, panel: Entity) -> Entity {
    let colours = ui().colours.window;
    commands
        .spawn((
            ChildOf(panel),
            Node {
                flex_direction: FlexDirection::Column,
                flex_shrink: 0.0,
                margin: UiRect::top(Val::VMin(0.6)),
                border: UiRect::top(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(colours.tab_off.with_alpha(colours.tab_off.alpha() * 0.5)),
            Pickable::IGNORE,
        ))
        .id()
}

/// One house on the board: its place, whose, its stars on average and how
/// many ratings; yours lit. A press shows it.
fn board_row(commands: &mut Commands, assets: &AssetServer, list: Entity, view: &View, ranked: &Ranked, yours: bool) {
    let ui = ui();
    let (colours, said) = (ui.colours.window, &ui.words.house);
    let back = if yours { colours.row_mine } else { Color::NONE };
    let name = if yours {
        format!("{}  ({})", handle(&ranked.name, ranked.tag), said.your_place)
    } else {
        handle(&ranked.name, ranked.tag)
    };
    let row = commands
        .spawn((
            ChildOf(list),
            Tap::House(HouseTap::Look(ranked.who)),
            TakesPress,
            Button,
            Node {
                height: Val::VMin(ROW_HIGH),
                align_items: AlignItems::Center,
                column_gap: Val::VMin(1.8),
                padding: UiRect::horizontal(Val::VMin(0.8)),
                border_radius: BorderRadius::all(Val::VMin(1.2)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(back),
            Fill {
                idle: back,
                lit: colours.button_lit,
            },
            children![
                (
                    Node {
                        width: Val::VMin(PLACE_WIDE),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![words(format!("#{}", ranked.place), NAME, colours.name)],
                ),
                disc(assets, DISC),
                (
                    Node {
                        flex_grow: 1.0,
                        flex_basis: Val::ZERO,
                        min_width: Val::ZERO,
                        overflow: Overflow::clip_x(),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![words(name, NAME, colours.name)],
                ),
            ],
        ))
        .id();
    // Its stars, by halves to the nearest, and the average and how many.
    let halves = if ranked.count == 0 { 0 } else { ((ranked.sum as f32 / ranked.count as f32).round() as u8).min(house::HALVES_MOST) };
    small_stars(commands, row, view, halves);
    commands.spawn((
        ChildOf(row),
        Node {
            width: Val::VMin(15.0),
            margin: UiRect::right(Val::VMin(PAD * 0.3)),
            justify_content: JustifyContent::FlexEnd,
            flex_shrink: 0.0,
            ..default()
        },
        Pickable::IGNORE,
        children![words(
            format!("{}  ({})", stars(ranked.count, ranked.sum), counted(ranked.count)),
            HEADING,
            colours.status,
        )],
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_written_short_and_stars_to_a_figure() {
        assert_eq!(counted(0), "0");
        assert_eq!(counted(950), "950");
        assert_eq!(counted(1_250), "1,250");
        assert_eq!(counted(99_999), "99,999");
        assert_eq!(counted(125_000), "125K");
        assert_eq!(counted(2_500_000), "2.5M");
        // Worth in thousands and millions to a tenth.
        assert_eq!(worth(0), "0");
        assert_eq!(worth(950), "950");
        assert_eq!(worth(1_000), "1.0K");
        assert_eq!(worth(28_200), "28.2K");
        assert_eq!(worth(28_249), "28.2K");
        assert_eq!(worth(999_949), "999.9K");
        assert_eq!(worth(999_950), "1.0M", "not 1000.0K");
        assert_eq!(worth(29_300_000), "29.3M");
        assert_eq!(worth(u32::MAX), "4.3B");
        assert_eq!(stars(0, 0), "-");
        assert_eq!(stars(2, 15), "3.8");
        assert_eq!(stars(1, 10), "5.0");
    }

    #[test]
    fn a_house_on_the_board_is_looked_at_and_back_is_the_board() {
        let mut view = HouseView::default();
        assert_eq!(view.sub, Sub::Mine);
        view.tap(HouseTap::Show(Sub::Board));
        view.tap(HouseTap::Look(7));
        assert_eq!((view.sub, view.looking), (Sub::Board, Some(7)));
        view.tap(HouseTap::Back);
        assert_eq!((view.sub, view.looking), (Sub::Board, None));
        view.tap(HouseTap::Look(8));
        view.tap(HouseTap::Show(Sub::Mine));
        assert_eq!((view.sub, view.looking), (Sub::Mine, None), "the person shows your own");
    }
}
