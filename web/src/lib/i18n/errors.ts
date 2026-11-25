import type { MessageKey } from './messages';

/**
 * The server's stable error codes, by the message that explains each. A
 * code the client doesn't know (or a failure with no code) shows
 * `errorUnknown`.
 */
const byCode: Readonly<Record<string, MessageKey>> = {
  nickname_invalid: 'errorNicknameInvalid',
  nickname_taken: 'errorNicknameTaken',
  rate_limited: 'errorRateLimited',
  room_limit: 'errorRoomLimit',
  too_many_rooms: 'errorTooManyRooms',
  not_signed_in: 'errorNotSignedIn',
  csrf_mismatch: 'errorCsrfMismatch',
  network: 'errorNetwork',
  room_not_found: 'errorRoomNotFound',
  too_many_connections: 'errorTooManyConnections',
  room_full: 'errorRoomFull',
  name_taken: 'errorNameTaken',
  already_started: 'errorAlreadyStarted',
  not_started: 'errorNotStarted',
  wrong_phase: 'errorWrongPhase',
  bidding_closed: 'errorBiddingClosed',
  bid_zero: 'errorBidZero',
  bid_not_lower: 'errorBidNotLower',
  not_your_turn: 'errorNotYourTurn',
  robot_did_not_move: 'errorRobotDidNotMove',
  bad_chat: 'errorBadChat',
  lagged: 'errorLagged',
  room_closed: 'errorRoomClosed',
  round_in_progress: 'errorRoundInProgress',
  game_over: 'errorGameOver',
  bidding_open: 'errorBiddingOpen',
  no_bids: 'errorNoBids',
  bad_message: 'errorBadMessage',
  not_member: 'errorNotMember',
};

export function errorKey(code: string): MessageKey {
  return Object.hasOwn(byCode, code) ? (byCode[code] ?? 'errorUnknown') : 'errorUnknown';
}
