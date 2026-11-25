import { describe, expect, it } from 'vitest';
import { isTyping, keyAction } from './keys';

describe('isTyping', () => {
  it('is true in form fields only', () => {
    for (const tag of ['INPUT', 'textarea', 'SELECT']) expect(isTyping(tag)).toBe(true);
    for (const tag of ['BUTTON', 'BODY', 'DIV', undefined]) expect(isTyping(tag)).toBe(false);
  });
});

describe('keyAction', () => {
  it('moves with arrows and WASD in either case', () => {
    expect(keyAction('ArrowLeft')).toEqual({ move: 'left' });
    expect(keyAction('w')).toEqual({ move: 'up' });
    expect(keyAction('D')).toEqual({ move: 'right' });
  });

  it('picks robots by number or initial', () => {
    expect(keyAction('2')).toEqual({ select: 'blue' });
    expect(keyAction('Y')).toEqual({ select: 'yellow' });
  });

  it('ignores other keys, inherited names included', () => {
    for (const key of ['x', 'Enter', 'constructor', 'toString', ' ']) {
      expect(keyAction(key)).toBeNull();
    }
  });
});
