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

<section class="lobby">
  <div class="head">
    <h2>{t('lobbyTitle')}</h2>
    <div class="actions">
      <button class="secondary" onclick={() => void app.refresh()} disabled={app.busy}
        >{t('refresh')}</button
      >
      <button onclick={() => void app.openRoom()} disabled={app.busy}>{t('openRoom')}</button>
    </div>
  </div>

  {#if app.rooms.length === 0}
    <div class="card empty">
      <p>{t('noRooms')}</p>
    </div>
  {:else}
    <ul class="rooms">
      {#each app.rooms as room (room.id)}
        <li class="card room">
          <span class="code">{t('room')} {room.id.slice(0, 6)}</span>
          <span class="muted">{t('players')}: {room.members}</span>
          <span class="seats" aria-hidden="true">
            {#each [...Array(Math.min(room.members, 8)).keys()] as seat (seat)}<span class="seat"></span>{/each}
          </span>
          <button onclick={join(room.id)}>{t('join')}</button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .lobby {
    display: grid;
    gap: 1rem;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: center;
    justify-content: space-between;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
  }
  .empty {
    text-align: center;
    color: var(--muted);
    padding-block: 2.5rem;
  }
  .empty p {
    margin: 0;
  }
  .rooms {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 1rem;
    grid-template-columns: repeat(auto-fill, minmax(15rem, 1fr));
  }
  .room {
    display: grid;
    gap: 0.4rem;
  }
  .code {
    font-weight: 700;
    font-size: 1.1rem;
    font-variant-numeric: tabular-nums;
  }
  .seats {
    display: flex;
    gap: 0.25rem;
    min-height: 0.75rem;
  }
  .seat {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 50%;
    background: var(--accent);
    opacity: 0.7;
  }
  .room button {
    justify-self: start;
    margin-top: 0.4rem;
  }
</style>
