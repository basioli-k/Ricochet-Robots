import { describe, expect, it } from 'vitest';
import { en, hr } from './messages';
import { pickLocale, translate } from './index';

describe('pickLocale', () => {
  it('matches on the primary language subtag', () => {
    expect(pickLocale(['hr-HR', 'en-US'])).toBe('hr');
  });

  it('takes the first supported preference', () => {
    expect(pickLocale(['de-DE', 'en-GB', 'hr'])).toBe('en');
  });

  it('ignores names inherited from Object.prototype', () => {
    expect(pickLocale(['constructor', '__proto__', 'toString-HR'])).toBe('en');
  });

  it('falls back to English', () => {
    expect(pickLocale(['de', 'fr'])).toBe('en');
    expect(pickLocale([])).toBe('en');
  });
});

describe('messages', () => {
  it('has a non-empty Croatian string for every English key', () => {
    for (const key of Object.keys(en) as (keyof typeof en)[]) {
      expect(hr[key].trim()).not.toBe('');
    }
  });

  it('translates by locale', () => {
    expect(translate('hr', 'comingSoon')).toBe(hr.comingSoon);
  });
});
