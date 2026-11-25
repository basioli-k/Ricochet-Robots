import { en, hr, type MessageKey, type Messages } from './messages';

export const locales = { en, hr } satisfies Record<string, Messages>;
export type Locale = keyof typeof locales;

/** Each language's own name, the same in every locale. */
export const localeNames: Readonly<Record<Locale, string>> = { en: 'English', hr: 'Hrvatski' };

// Own keys only: `in` would also accept inherited names such as `constructor`.
export const isLocale = (value: string): value is Locale => Object.hasOwn(locales, value);

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

/**
 * Fills `{name}` placeholders, so each language orders a sentence its own
 * way. Unknown placeholders stay as they are.
 */
export function fill(text: string, values: Readonly<Record<string, string | number>>): string {
  return text.replace(/\{(\w+)\}/g, (whole, name: string) =>
    Object.hasOwn(values, name) ? String(values[name]) : whole,
  );
}
