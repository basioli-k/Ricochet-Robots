import type { Board } from '../../protocol/Board';
import type { Dir } from '../../protocol/Dir';
import type { Robot } from '../../protocol/Robot';
import type { Shape } from '../../protocol/Shape';
import type { Target } from '../../protocol/Target';
import type { MessageKey } from '../i18n/messages';

export const ROBOTS: readonly Robot[] = ['red', 'blue', 'green', 'yellow'];

export const robotName: Readonly<Record<Robot, MessageKey>> = {
  red: 'robotRed',
  blue: 'robotBlue',
  green: 'robotGreen',
  yellow: 'robotYellow',
};

export const shapeName: Readonly<Record<Shape, MessageKey>> = {
  star: 'shapeStar',
  planet: 'shapePlanet',
  gear: 'shapeGear',
  moon: 'shapeMoon',
};

/** Symbols drawn on the board; icons, not text, so they need no translation. */
export const shapeGlyph: Readonly<Record<Shape, string>> = {
  star: '★',
  planet: '♁',
  gear: '⚙',
  moon: '☾',
};
export const VORTEX_GLYPH = '✺';

export function targetGlyph(target: Target): string {
  return target.kind === 'vortex' ? VORTEX_GLYPH : shapeGlyph[target.shape];
}

export function sameTarget(a: Target, b: Target): boolean {
  if (a.kind === 'vortex' || b.kind === 'vortex') return a.kind === b.kind;
  return a.robot === b.robot && a.shape === b.shape;
}

export interface Cell {
  row: number;
  col: number;
  walls: Set<Dir>;
  blocked: boolean;
  target: Target | null;
}

/** The board as rows of cells, each knowing its walls, block and target. */
export function cells(board: Board): Cell[] {
  const key = (row: number, col: number) => row * board.size + col;
  const all: Cell[] = [];
  for (let row = 0; row < board.size; row++) {
    for (let col = 0; col < board.size; col++) {
      all.push({ row, col, walls: new Set(), blocked: false, target: null });
    }
  }
  const at = (row: number, col: number) => all[key(row, col)];
  for (const wall of board.walls) at(wall.pos.row, wall.pos.col)?.walls.add(wall.side);
  for (const pos of board.blocked) {
    const cell = at(pos.row, pos.col);
    if (cell) cell.blocked = true;
  }
  for (const { pos, target } of board.targets) {
    const cell = at(pos.row, pos.col);
    if (cell) cell.target = target;
  }
  return all;
}
