/** The server's name rule: 3–15 ASCII letters or digits (no lookalike Unicode). */
export function isValidNickname(nickname: string): boolean {
  return /^[A-Za-z0-9]{3,15}$/.test(nickname);
}
