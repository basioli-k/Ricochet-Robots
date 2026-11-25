<script lang="ts">
  import { localeNames, isLocale } from '../i18n';
  import { i18n, setLocale, t } from '../i18n/locale.svelte';

  function choose(event: Event & { currentTarget: HTMLSelectElement }) {
    const value = event.currentTarget.value;
    if (isLocale(value)) setLocale(value);
  }
</script>

<label class="language">
  {t('language')}
  <select value={i18n.locale} onchange={choose}>
    {#each Object.entries(localeNames) as [code, name] (code)}
      <option value={code}>{name}</option>
    {/each}
  </select>
</label>
