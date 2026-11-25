<script lang="ts">
  import type { AppModel } from '../app.svelte';
  import { t } from '../i18n/locale.svelte';

  let { app }: { app: AppModel } = $props();

  /** A click handler that enters room `id`. */
  function join(id: string): () => void {
    return () => {
      app.enter(id);
    };
  }

  /** How often the open lobby re-reads the room list. */
  const REFRESH_MS = 5_000;

  $effect(() => {
    const timer = setInterval(() => void app.poll(), REFRESH_MS);
    return () => {
      clearInterval(timer);
    };
  });
</script>

<section>
  {#if app.identity}
    <p class="who">
      {t('playingAs')} <strong>{app.identity.nickname}</strong>
      <span class="badge">{t('guestBadge')}</span>
    </p>
  {/if}
  <h2>{t('lobbyTitle')}</h2>
  <div class="actions">
    <button onclick={() => void app.openRoom()} disabled={app.busy}>{t('openRoom')}</button>
    <button onclick={() => void app.refresh()} disabled={app.busy}>{t('refresh')}</button>
  </div>
  {#if app.rooms.length === 0}
    <p>{t('noRooms')}</p>
  {:else}
    <ul class="rooms">
      {#each app.rooms as room (room.id)}
        <li>
          <span>{t('room')} {room.id.slice(0, 6)}</span>
          <span>{t('players')}: {room.members}</span>
          <button onclick={join(room.id)}>{t('join')}</button>
        </li>
      {/each}
    </ul>
  {/if}
</section>
