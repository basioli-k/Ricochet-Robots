use Direction::Up;
use RobotColor::{Blue, Red};
use domain::{
    Board, BoardBuilder, Direction, Game, GameError, Phase, PlayerId, Position, RobotColor, Robots,
    RoundError, Shuffle, Symbol, Target, Timestamp, fixed_board,
};

const ANA: PlayerId = PlayerId::new(1);
const BOB: PlayerId = PlayerId::new(2);
const WINDOW: u64 = 60_000;
const RED_MOON: Target = Target::Colored(Red, Symbol::Moon);

/// Keeps the deck in board placement order.
struct InOrder;
impl Shuffle for InOrder {
    fn shuffle(&mut self, _deck: &mut [Target]) {}
}

/// Reverses the deck, to show the shuffle decides the order.
struct Reversed;
impl Shuffle for Reversed {
    fn shuffle(&mut self, deck: &mut [Target]) {
        deck.reverse();
    }
}

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

/// Red Moon at (0,5), then the vortex at (0,10); see tests/round.rs.
fn board() -> Board {
    BoardBuilder::new()
        .target(pos(0, 5), RED_MOON)
        .and_then(|b| b.target(pos(0, 10), Target::Vortex))
        .unwrap()
        .build()
        .unwrap()
}

fn robots() -> Robots {
    Robots::new([pos(5, 5), pos(5, 10), pos(10, 0), pos(15, 15)]).unwrap()
}

fn new_game(shuffle: &mut impl Shuffle) -> Game {
    Game::new(board(), robots(), WINDOW, shuffle).unwrap()
}

/// `player` bids one move and solves the current round with `robot` Up.
fn solve(game: &mut Game, player: PlayerId, robot: RobotColor) {
    game.bid(player, 1, Timestamp(0)).unwrap();
    game.close_bidding(Timestamp(WINDOW)).unwrap();
    game.move_robot(player, robot, Up).unwrap();
}

#[test]
fn plays_every_target_of_the_fixed_board() {
    let game = Game::new(
        fixed_board::board(),
        fixed_board::starting_robots(),
        WINDOW,
        &mut InOrder,
    )
    .unwrap();
    assert_eq!(game.targets_left(), 16);
}

#[test]
fn the_shuffle_decides_the_order() {
    assert_eq!(new_game(&mut InOrder).round().target(), RED_MOON);
    assert_eq!(new_game(&mut Reversed).round().target(), Target::Vortex);
}

#[test]
fn a_board_without_targets_has_no_game() {
    let empty = BoardBuilder::new().build().unwrap();
    assert!(Game::new(empty, robots(), WINDOW, &mut InOrder).is_none());
}

#[test]
fn a_round_in_play_cannot_be_skipped() {
    let mut game = new_game(&mut InOrder);
    assert_eq!(game.next_round(), Err(GameError::RoundInProgress));
    game.bid(ANA, 3, Timestamp(0)).unwrap();
    game.close_bidding(Timestamp(WINDOW)).unwrap();
    assert_eq!(game.next_round(), Err(GameError::RoundInProgress));
}

#[test]
fn a_solved_round_scores_and_robots_stay() {
    let mut game = new_game(&mut InOrder);
    solve(&mut game, ANA, Red);
    game.next_round().unwrap();
    assert_eq!(game.scores().get(&ANA), Some(&1));
    assert_eq!(game.round().target(), Target::Vortex);
    assert_eq!(game.round().phase(), Phase::Bidding);
    assert_eq!(game.round().robots().position(Red), pos(0, 5));
}

#[test]
fn an_unsolved_round_scores_nothing_and_robots_reset() {
    let mut game = new_game(&mut InOrder);
    game.bid(ANA, 1, Timestamp(0)).unwrap();
    game.close_bidding(Timestamp(WINDOW)).unwrap();
    game.move_robot(ANA, Blue, Up).unwrap(); // Blue can't complete the red moon
    game.next_round().unwrap();
    assert!(game.scores().is_empty());
    assert_eq!(*game.round().robots(), robots());
}

#[test]
fn the_game_ends_when_the_deck_is_empty() {
    let mut game = new_game(&mut InOrder);
    solve(&mut game, ANA, Red);
    game.next_round().unwrap();
    solve(&mut game, BOB, Blue);
    assert!(!game.is_over());
    game.next_round().unwrap();
    assert!(game.is_over());
    assert_eq!(game.next_round(), Err(GameError::GameOver));
    assert_eq!(game.leaders(), [ANA, BOB]);
}

#[test]
fn leaders_are_the_top_scorers_or_nobody() {
    let mut game = new_game(&mut InOrder);
    game.abandon_round().unwrap();
    game.next_round().unwrap();
    assert!(game.leaders().is_empty());
    solve(&mut game, BOB, Blue);
    game.next_round().unwrap();
    assert_eq!(game.leaders(), [BOB]);
}

#[test]
fn round_errors_pass_through() {
    let mut game = new_game(&mut InOrder);
    assert_eq!(
        game.move_robot(ANA, Red, Up),
        Err(GameError::Round(RoundError::WrongPhase))
    );
}

const BLUE_STAR: Target = Target::Colored(Blue, Symbol::Star);

/// `board()` plus a Blue Star at (15,10), which Blue reaches with Down.
fn board3() -> Board {
    BoardBuilder::new()
        .target(pos(0, 5), RED_MOON)
        .and_then(|b| b.target(pos(0, 10), Target::Vortex))
        .and_then(|b| b.target(pos(15, 10), BLUE_STAR))
        .unwrap()
        .build()
        .unwrap()
}

fn close(game: &mut Game, bids: &[(PlayerId, u8)]) {
    for &(player, moves) in bids {
        game.bid(player, moves, Timestamp(0)).unwrap();
    }
    game.close_bidding(Timestamp(WINDOW)).unwrap();
}

/// Robots carry over, so a target can come up already completed; it must
/// not be winnable by any single move.
#[test]
fn a_target_already_completed_is_put_back_not_played() {
    let mut game = Game::new(board3(), robots(), WINDOW, &mut InOrder).unwrap();
    close(&mut game, &[(ANA, 2)]);
    game.move_robot(ANA, Blue, Up).unwrap(); // Blue parks on the vortex
    game.move_robot(ANA, Red, Up).unwrap(); // Red completes the red moon
    game.next_round().unwrap();
    // The vortex is already completed by Blue, so the blue star comes first.
    assert_eq!(game.round().target(), BLUE_STAR);
    game.abandon_round().unwrap();
    game.next_round().unwrap();
    // Only the vortex is left, still completed: nothing playable remains.
    assert!(game.is_over());
    assert_eq!(game.targets_left(), 0);
    assert_eq!(game.leaders(), [ANA]);
}

#[test]
fn the_first_target_is_skipped_if_already_completed() {
    let red_on_moon = Robots::new([pos(0, 5), pos(5, 10), pos(10, 0), pos(15, 15)]).unwrap();
    let game = Game::new(board(), red_on_moon, WINDOW, &mut InOrder).unwrap();
    assert_eq!(game.round().target(), Target::Vortex);
}

#[test]
fn later_rounds_keep_the_bidding_window() {
    let mut game = new_game(&mut InOrder);
    solve(&mut game, ANA, Red);
    game.next_round().unwrap();
    game.bid(BOB, 4, Timestamp(5_000)).unwrap();
    assert_eq!(
        game.round().bidding_closes_at(),
        Some(Timestamp(5_000 + WINDOW))
    );
}

#[test]
fn the_leader_has_strictly_more_tokens() {
    let mut game = Game::new(board3(), robots(), WINDOW, &mut InOrder).unwrap();
    solve(&mut game, ANA, Red); // red moon
    game.next_round().unwrap();
    solve(&mut game, ANA, Blue); // vortex
    game.next_round().unwrap();
    close(&mut game, &[(BOB, 1)]);
    game.move_robot(BOB, Blue, Direction::Down).unwrap(); // blue star
    game.next_round().unwrap();
    assert_eq!(game.scores().get(&ANA), Some(&2));
    assert_eq!(game.scores().get(&BOB), Some(&1));
    assert_eq!(game.leaders(), [ANA]);
}

/// Overwrites the deck instead of reordering it.
struct Corrupting;
impl Shuffle for Corrupting {
    fn shuffle(&mut self, deck: &mut [Target]) {
        deck.fill(Target::Vortex);
    }
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "Shuffle must only reorder the deck")]
fn a_shuffle_that_corrupts_the_deck_is_caught_in_debug_builds() {
    let _ = Game::new(board(), robots(), WINDOW, &mut Corrupting);
}

/// A target put back because it was completed is dealt again once the
/// robots have moved off it.
#[test]
fn a_put_back_target_is_dealt_once_playable_again() {
    let mut game = Game::new(board3(), robots(), WINDOW, &mut InOrder).unwrap();
    close(&mut game, &[(ANA, 2)]);
    game.move_robot(ANA, Blue, Up).unwrap(); // Blue parks on the vortex
    game.move_robot(ANA, Red, Up).unwrap();
    game.next_round().unwrap();
    assert_eq!(game.round().target(), BLUE_STAR);
    close(&mut game, &[(BOB, 1)]);
    game.move_robot(BOB, Blue, Direction::Down).unwrap(); // Blue leaves the vortex
    game.next_round().unwrap();
    assert_eq!(game.round().target(), Target::Vortex);
}
