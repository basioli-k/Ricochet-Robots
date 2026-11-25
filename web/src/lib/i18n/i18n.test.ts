import { describe, expect, it } from 'vitest';
import { en, hr } from './messages';
import { fill, pickLocale, translate } from './index';

describe('pickLocale', () => {
  it('matches on the primary language subtag', () => {
    expect(pickLocale(['hr-HR', 'en-US'])).toBe('hr');
  });

  it('takes the first supported preference', () => {
    expect(pickLocale(['de-DE', 'en-GB', 'hr'])).toBe('en');
  });

  it('ignores names inherited from Object.prototype', () => {
    expect(pickLocale(['constructor', '__proto__'])).toBe('en');
  });

  it('falls back to English', () => {
    expect(pickLocale(['de', 'fr'])).toBe('en');
    expect(pickLocale([])).toBe('en');
  });
});

describe('fill', () => {
  it('replaces known placeholders and leaves others', () => {
    expect(fill('{name} has {n} – {other}', { name: 'Ana', n: 3 })).toBe('Ana has 3 – {other}');
    expect(fill('{toString}', {})).toBe('{toString}');
  });
});

describe('messages', () => {
  it('has a non-empty Croatian string for every English key', () => {
    for (const key of Object.keys(en) as (keyof typeof en)[]) {
      expect(hr[key].trim()).not.toBe('');
    }
  });

  it('translates by locale', () => {
    expect(translate('hr', 'play')).toBe(hr.play);
  });
});
