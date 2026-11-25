import { fill, translate, type Locale } from './index';
import type { MessageKey } from './messages';
import { browserStore, loadLocale, saveLocale } from './preference';

/** The page's language; components read it through `t`, so they follow changes. */
export const i18n = $state({ locale: loadLocale(browserStore(), navigator.languages) });

export function setLocale(locale: Locale): void {
  i18n.locale = locale;
  saveLocale(browserStore(), locale);
}

export function t(key: MessageKey, values?: Readonly<Record<string, string | number>>): string {
  const text = translate(i18n.locale, key);
  return values ? fill(text, values) : text;
}
