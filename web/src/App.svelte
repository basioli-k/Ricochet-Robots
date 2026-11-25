<script lang="ts">
  import LanguagePicker from './lib/components/LanguagePicker.svelte';
  import { i18n, t } from './lib/i18n/locale.svelte';

  $effect(() => {
    document.documentElement.lang = i18n.locale;
    document.title = t('appTitle');
  });
</script>

<header>
  <h1>{t('appTitle')}</h1>
  <LanguagePicker />
</header>
<main>
  <p>{t('loading')}</p>
</main>
