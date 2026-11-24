use Direction::{Left, Right, Up};
use RobotColor::{Blue, Green, Red};
use domain::{
    Board, BoardBuilder, Direction, Phase, PlayerId, Position, RobotColor, Robots, Round,
    RoundError, Symbol, Target, Timestamp,
};

const ANA: PlayerId = PlayerId::new(1);
const BOB: PlayerId = PlayerId::new(2);
const WINDOW: u64 = 60_000;
const RED_MOON: Target = Target::Colored(Red, Symbol::Moon);

fn pos(row: u8, col: u8) -> Position {
    Position::new(row, col).unwrap()
}

/// Open board: Red Moon at (0,5), vortex at (0,10). Red at (5,5) reaches the
/// moon with Up; Blue at (5,10) reaches the vortex with Up.
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

/// A round on `target` whose bidding has closed with the given bids.
fn demonstrating(target: Target, bids: &[(PlayerId, u8)]) -> Round {
    let mut round = Round::new(target, robots(), WINDOW);
    for (i, &(player, moves)) in (0..).zip(bids) {
        round.bid(player, moves, Timestamp(i)).unwrap();
    }
    round.close_bidding(Timestamp(WINDOW)).unwrap();
    round
}

#[test]
fn the_first_bid_opens_a_fixed_window() {
    let mut round = Round::new(RED_MOON, robots(), WINDOW);
    assert_eq!(round.bidding_closes_at(), None);
    round.bid(ANA, 5, Timestamp(1_000)).unwrap();
    assert_eq!(round.bidding_closes_at(), Some(Timestamp(61_000)));
    round.bid(BOB, 3, Timestamp(60_999)).unwrap();
    assert_eq!(
        round.bid(ANA, 2, Timestamp(61_000)),
        Err(RoundError::BiddingClosed)
    );
}

#[test]
fn bidding_closes_only_after_the_window() {
    let mut round = Round::new(RED_MOON, robots(), WINDOW);
    round.bid(ANA, 5, Timestamp(0)).unwrap();
    round.bid(BOB, 3, Timestamp(10)).unwrap();
    assert_eq!(
        round.close_bidding(Timestamp(WINDOW - 1)),
        Err(RoundError::BiddingStillOpen)
    );
    round.close_bidding(Timestamp(WINDOW)).unwrap();
    assert_eq!(
        round.phase(),
        Phase::Demonstrating {
            player: BOB,
            bid: 3,
            moves_made: 0
        }
    );
}

#[test]
fn a_round_without_bids_is_abandoned_not_closed() {
    let mut round = Round::new(RED_MOON, robots(), WINDOW);
    assert_eq!(
        round.close_bidding(Timestamp(u64::MAX)),
        Err(RoundError::NoBids)
    );
    round.abandon().unwrap();
    assert_eq!(round.phase(), Phase::Unsolved);

    let mut bid_on = Round::new(RED_MOON, robots(), WINDOW);
    bid_on.bid(ANA, 4, Timestamp(0)).unwrap();
    assert_eq!(bid_on.abandon(), Err(RoundError::WrongPhase));
}

#[test]
fn reaching_the_target_within_the_bid_wins_and_keeps_robots() {
    let mut round = demonstrating(RED_MOON, &[(ANA, 2)]);
    round.move_robot(&board(), ANA, Red, Up).unwrap();
    assert_eq!(
        round.phase(),
        Phase::Solved {
            winner: ANA,
            moves: 1
        }
    );
    assert_eq!(round.robots().position(Red), pos(0, 5));
}

#[test]
fn a_coloured_target_needs_its_own_robot() {
    // Blue lands on the red moon's row but only Red can complete it.
    let mut round = Round::new(
        RED_MOON,
        Robots::new([pos(9, 9), pos(5, 5), pos(10, 0), pos(15, 15)]).unwrap(),
        WINDOW,
    );
    round.bid(ANA, 1, Timestamp(0)).unwrap();
    round.close_bidding(Timestamp(WINDOW)).unwrap();
    round.move_robot(&board(), ANA, Blue, Up).unwrap();
    assert_eq!(round.phase(), Phase::Unsolved);
}

#[test]
fn any_robot_completes_the_vortex() {
    let mut round = demonstrating(Target::Vortex, &[(ANA, 1)]);
    round.move_robot(&board(), ANA, Blue, Up).unwrap();
    assert_eq!(
        round.phase(),
        Phase::Solved {
            winner: ANA,
            moves: 1
        }
    );
}

#[test]
fn a_failed_demonstration_resets_robots_and_passes_on() {
    let mut round = demonstrating(RED_MOON, &[(ANA, 1), (BOB, 2)]);
    round.move_robot(&board(), ANA, Blue, Up).unwrap();
    assert_eq!(
        round.phase(),
        Phase::Demonstrating {
            player: BOB,
            bid: 2,
            moves_made: 0
        }
    );
    assert_eq!(*round.robots(), robots());
    round.move_robot(&board(), BOB, Red, Up).unwrap();
    assert_eq!(
        round.phase(),
        Phase::Solved {
            winner: BOB,
            moves: 1
        }
    );
}

#[test]
fn when_everyone_fails_the_round_is_unsolved_with_robots_reset() {
    let mut round = demonstrating(RED_MOON, &[(ANA, 1), (BOB, 1)]);
    round.move_robot(&board(), ANA, Blue, Up).unwrap();
    round.concede(BOB).unwrap();
    assert_eq!(round.phase(), Phase::Unsolved);
    assert_eq!(*round.robots(), robots());
}

#[test]
fn only_the_demonstrator_may_act() {
    let mut round = demonstrating(RED_MOON, &[(ANA, 2), (BOB, 3)]);
    let before = round.clone();
    assert_eq!(
        round.move_robot(&board(), BOB, Red, Up),
        Err(RoundError::NotYourTurn)
    );
    assert_eq!(round.concede(BOB), Err(RoundError::NotYourTurn));
    assert_eq!(round, before, "refused actions must not change the round");
}

/// Regression (PR #11 review): every move counts against the bid, not just
/// the first, so a bid of several moves is used up move by move.
#[test]
fn a_multi_move_bid_is_used_up_move_by_move() {
    let mut round = demonstrating(RED_MOON, &[(ANA, 2)]);
    round.move_robot(&board(), ANA, Green, Up).unwrap();
    assert_eq!(
        round.phase(),
        Phase::Demonstrating {
            player: ANA,
            bid: 2,
            moves_made: 1
        }
    );
    round.move_robot(&board(), ANA, Green, Right).unwrap();
    assert_eq!(round.phase(), Phase::Unsolved);
    assert_eq!(*round.robots(), robots());
}

#[test]
fn solving_on_a_later_move_reports_the_moves_used() {
    let mut round = demonstrating(RED_MOON, &[(ANA, 3)]);
    round.move_robot(&board(), ANA, Blue, Up).unwrap();
    round.move_robot(&board(), ANA, Red, Up).unwrap();
    assert_eq!(
        round.phase(),
        Phase::Solved {
            winner: ANA,
            moves: 2
        }
    );
}

#[test]
fn a_move_that_goes_nowhere_does_not_count() {
    let mut round = demonstrating(RED_MOON, &[(ANA, 1)]);
    assert_eq!(
        round.move_robot(&board(), ANA, Green, Left),
        Err(RoundError::RobotDidNotMove)
    );
    assert_eq!(
        round.phase(),
        Phase::Demonstrating {
            player: ANA,
            bid: 1,
            moves_made: 0
        }
    );
}

#[test]
fn actions_outside_their_phase_are_refused() {
    let mut bidding = Round::new(RED_MOON, robots(), WINDOW);
    let before = bidding.clone();
    assert_eq!(
        bidding.move_robot(&board(), ANA, Red, Up),
        Err(RoundError::WrongPhase)
    );
    assert_eq!(bidding.concede(ANA), Err(RoundError::WrongPhase));
    assert_eq!(bidding, before, "refused actions must not change the round");

    let mut solved = demonstrating(RED_MOON, &[(ANA, 2)]);
    assert_eq!(
        solved.bid(BOB, 1, Timestamp(0)),
        Err(RoundError::WrongPhase)
    );
    solved.move_robot(&board(), ANA, Red, Up).unwrap();
    assert_eq!(
        solved.move_robot(&board(), ANA, Red, Left),
        Err(RoundError::WrongPhase)
    );
    assert_eq!(
        solved.close_bidding(Timestamp(u64::MAX)),
        Err(RoundError::WrongPhase)
    );
    assert_eq!(solved.abandon(), Err(RoundError::WrongPhase));
}
