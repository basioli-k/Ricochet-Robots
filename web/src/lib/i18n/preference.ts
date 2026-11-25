import { isLocale, pickLocale, type Locale } from './index';

/** The part of `localStorage` used here, so tests can pass a fake. */
export type KeyValueStore = Pick<Storage, 'getItem' | 'setItem'>;

const KEY = 'rr.locale';

/**
 * The saved language, else the first supported browser language. Storage
 * can be missing or throw (private windows, blocked site data), so any
 * failure falls back to the browser's languages.
 */
export function loadLocale(store: KeyValueStore | null, browser: readonly string[]): Locale {
  try {
    const saved = store?.getItem(KEY);
    if (saved != null && isLocale(saved)) return saved;
  } catch {
    // Unreadable storage: use the browser's languages.
  }
  return pickLocale(browser);
}

/** Remembers the choice; returns whether it could be saved. */
export function saveLocale(store: KeyValueStore | null, locale: Locale): boolean {
  try {
    store?.setItem(KEY, locale);
    return store != null;
  } catch {
    return false;
  }
}

/** `localStorage`, or null where reading it throws. */
export function browserStore(): KeyValueStore | null {
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}
