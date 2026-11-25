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
};

export function errorKey(code: string): MessageKey {
  return Object.hasOwn(byCode, code) ? (byCode[code] ?? 'errorUnknown') : 'errorUnknown';
}
