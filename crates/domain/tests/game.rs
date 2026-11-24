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
