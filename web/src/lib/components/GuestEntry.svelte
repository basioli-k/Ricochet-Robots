<script lang="ts">
  import type { AppModel } from '../app.svelte';
  import { t } from '../i18n/locale.svelte';

  let { app }: { app: AppModel } = $props();
  let nickname = $state('');

  function submit(event: SubmitEvent) {
    event.preventDefault();
    void app.play(nickname.trim());
  }
</script>

<section>
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
