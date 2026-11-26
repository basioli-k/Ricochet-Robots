<script lang="ts">
  import type { AppModel } from '../app.svelte';
  import { t } from '../i18n/locale.svelte';

  let { app }: { app: AppModel } = $props();
  let nickname = $state('');

  function submit(event: SubmitEvent) {
    event.preventDefault();
    void app.play(nickname.trim());
  }

  const steps = ['howBid', 'howShow', 'howWin'] as const;
</script>

<div class="narrow stack">
  <section class="card hero">
    <p class="tagline">{t('tagline')}</p>
    <h2>{t('guestTitle')}</h2>
    <form onsubmit={submit}>
      <label>
        {t('nicknameLabel')}
        <input
          bind:value={nickname}
          name="nickname"
          autocomplete="nickname"
          maxlength="15"
          required
          aria-describedby="nickname-hint"
        />
      </label>
      <small id="nickname-hint">{t('nicknameHint')}</small>
      <button type="submit" disabled={app.busy}>{t('play')}</button>
    </form>
  </section>

  <section class="card">
    <h3>{t('howTitle')}</h3>
    <ol class="steps">
      {#each steps as step, i (step)}
        <li><span class="num" aria-hidden="true">{i + 1}</span>{t(step)}</li>
      {/each}
    </ol>
  </section>
</div>

<style>
  .stack {
    display: grid;
    gap: 1rem;
  }
  .hero {
    display: grid;
    gap: 0.9rem;
  }
  .tagline {
    margin: 0;
    color: var(--muted);
    font-size: 1.05rem;
  }
  form {
    gap: 0.6rem;
  }
  button {
    justify-self: start;
    min-width: 8rem;
  }
  .steps {
    list-style: none;
    margin: 0.75rem 0 0;
    padding: 0;
    display: grid;
    gap: 0.7rem;
  }
  .steps li {
    display: flex;
    gap: 0.7rem;
    align-items: baseline;
  }
  .num {
    flex: none;
    display: inline-grid;
    place-items: center;
    width: 1.6rem;
    height: 1.6rem;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent-strong);
    font-weight: 700;
    font-size: 0.85rem;
  }
</style>
