<script lang="ts">
  import type { AppModel } from '../app.svelte';
  import { t } from '../i18n/locale.svelte';

  let { app, id }: { app: AppModel; id: string } = $props();
</script>

<section>
  <h2>{t('room')} {id.slice(0, 6)}</h2>
  <p>{t('gameComingSoon')}</p>
  <button onclick={() => void app.backToLobby()}>{t('backToLobby')}</button>
</section>
