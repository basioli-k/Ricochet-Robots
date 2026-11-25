import type { Dir } from '../../protocol/Dir';
import type { Robot } from '../../protocol/Robot';

export type KeyAction = { move: Dir } | { select: Robot } | null;

const moves: Readonly<Record<string, Dir>> = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
  w: 'up',
  s: 'down',
  a: 'left',
  d: 'right',
};

const robots: Readonly<Record<string, Robot>> = {
  '1': 'red',
  '2': 'blue',
  '3': 'green',
  '4': 'yellow',
  r: 'red',
  b: 'blue',
  g: 'green',
  y: 'yellow',
};

/** What a key press means on the board: arrows or WASD move, 1–4 or R/B/G/Y pick a robot. */
export function keyAction(key: string): KeyAction {
  const k = key.length === 1 ? key.toLowerCase() : key;
  if (Object.hasOwn(moves, k)) return { move: moves[k] ?? 'up' };
  if (Object.hasOwn(robots, k)) return { select: robots[k] ?? 'red' };
  return null;
}
