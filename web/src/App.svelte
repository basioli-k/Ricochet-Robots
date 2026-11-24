<script lang="ts">
  import { pickLocale, translate } from './lib/i18n';

  const locale = pickLocale(navigator.languages);
  document.documentElement.lang = locale;
  document.title = translate(locale, 'appTitle');
</script>

<main>
  <h1>{translate(locale, 'appTitle')}</h1>
  <p>{translate(locale, 'comingSoon')}</p>
</main>
