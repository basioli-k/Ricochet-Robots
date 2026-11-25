import { describe, expect, it } from 'vitest';
import { cells, sameTarget, targetGlyph } from './board';

describe('cells', () => {
  it('places walls, blocks and targets on their cells', () => {
    const grid = cells({
      size: 16,
      walls: [
        { pos: { row: 3, col: 4 }, side: 'right' },
        { pos: { row: 3, col: 5 }, side: 'left' },
      ],
      blocked: [{ row: 7, col: 7 }],
      targets: [{ pos: { row: 0, col: 1 }, target: { kind: 'vortex' } }],
    });
    expect(grid).toHaveLength(256);
    const at = (row: number, col: number) => grid[row * 16 + col];
    expect([...(at(3, 4)?.walls ?? [])]).toEqual(['right']);
    expect([...(at(3, 5)?.walls ?? [])]).toEqual(['left']);
    expect(at(7, 7)?.blocked).toBe(true);
    expect(at(0, 1)?.target).toEqual({ kind: 'vortex' });
    expect(at(0, 0)?.target).toBeNull();
  });
});

describe('targets', () => {
  it('compare by robot and shape and have glyphs', () => {
    const redMoon = { kind: 'colored', robot: 'red', shape: 'moon' } as const;
    expect(sameTarget(redMoon, { ...redMoon })).toBe(true);
    expect(sameTarget(redMoon, { ...redMoon, robot: 'blue' })).toBe(false);
    expect(sameTarget(redMoon, { kind: 'vortex' })).toBe(false);
    expect(sameTarget({ kind: 'vortex' }, { kind: 'vortex' })).toBe(true);
    expect(targetGlyph(redMoon)).toBe('☾');
    expect(targetGlyph({ kind: 'vortex' })).toBe('✺');
  });
});
