<script lang="ts">
  import { localeNames, isLocale } from '../i18n';
  import { i18n, setLocale, t } from '../i18n/locale.svelte';

  function choose(event: Event & { currentTarget: HTMLSelectElement }) {
    const value = event.currentTarget.value;
    if (isLocale(value)) setLocale(value);
  }
</script>

<label class="language">
  <span class="muted">{t('language')}</span>
  <select value={i18n.locale} onchange={choose}>
    {#each Object.entries(localeNames) as [code, name] (code)}
      <option value={code}>{name}</option>
    {/each}
  </select>
</label>

<style>
  .language {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-weight: 400;
  }
  select {
    padding: 0.35rem 0.6rem;
  }
</style>
