import { describe, expect, it } from 'vitest';
import { errorKey } from './errors';
import { en } from './messages';
import { loadLocale, saveLocale, type KeyValueStore } from './preference';

function memory(): KeyValueStore & { data: Map<string, string> } {
  const data = new Map<string, string>();
  return {
    data,
    getItem: (key) => data.get(key) ?? null,
    setItem: (key, value) => {
      data.set(key, value);
    },
  };
}

const broken: KeyValueStore = {
  getItem: () => {
    throw new Error('blocked');
  },
  setItem: () => {
    throw new Error('blocked');
  },
};

describe('language preference', () => {
  it('prefers the saved language over the browser', () => {
    const store = memory();
    expect(saveLocale(store, 'hr')).toBe(true);
    expect(loadLocale(store, ['en-US'])).toBe('hr');
  });

  it('falls back to the browser without a valid saved language', () => {
    const store = memory();
    expect(loadLocale(store, ['hr-HR'])).toBe('hr');
    store.data.set('rr.locale', 'de');
    expect(loadLocale(store, ['en'])).toBe('en');
    store.data.set('rr.locale', 'constructor');
    expect(loadLocale(store, ['en'])).toBe('en');
  });

  it('survives missing or failing storage', () => {
    expect(loadLocale(null, ['hr'])).toBe('hr');
    expect(loadLocale(broken, ['hr'])).toBe('hr');
    expect(saveLocale(null, 'hr')).toBe(false);
    expect(saveLocale(broken, 'hr')).toBe(false);
  });
});

describe('error messages', () => {
  it('has a message for each server code the pages can see', () => {
    for (const code of [
      'nickname_invalid',
      'nickname_taken',
      'rate_limited',
      'room_limit',
      'too_many_rooms',
      'not_signed_in',
      'csrf_mismatch',
      'network',
    ]) {
      expect(errorKey(code)).not.toBe('errorUnknown');
      expect(en[errorKey(code)]).toBeTruthy();
    }
  });

  it('shows a generic message for unknown codes', () => {
    expect(errorKey('no_such_code')).toBe('errorUnknown');
    expect(errorKey('toString')).toBe('errorUnknown');
  });
});
