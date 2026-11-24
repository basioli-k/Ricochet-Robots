import { en, hr, type MessageKey, type Messages } from './messages';

export const locales = { en, hr } satisfies Record<string, Messages>;
export type Locale = keyof typeof locales;

const isLocale = (value: string): value is Locale => value in locales;

/**
 * Picks the first supported locale from the user's preferences (e.g.
 * `navigator.languages`), matching on the primary subtag, else English.
 */
export function pickLocale(preferred: readonly string[]): Locale {
  for (const tag of preferred) {
    const primary = tag.toLowerCase().split('-')[0] ?? '';
    if (isLocale(primary)) return primary;
  }
  return 'en';
}

export function translate(locale: Locale, key: MessageKey): string {
  return locales[locale][key];
}
