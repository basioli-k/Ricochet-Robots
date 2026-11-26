<script lang="ts">
  import type { Dir } from '../../protocol/Dir';
  import { robotName, ROBOTS } from '../game/board';
  import type { GameModel } from '../game/model.svelte';
  import { t } from '../i18n/locale.svelte';

  /**
   * The demonstrator's controls: an arrow pad and the robot to move. In the
   * side panel on wide screens; docked to the bottom of narrow ones, big
   * enough for thumbs, with the move count and give-up beside it.
   */
  let {
    game,
    docked = false,
    moves = 0,
    bid = 0,
  }: { game: GameModel; docked?: boolean; moves?: number; bid?: number } = $props();

  const directions: { dir: Dir; label: 'moveUp' | 'moveDown' | 'moveLeft' | 'moveRight'; arrow: string }[] = [
    { dir: 'up', label: 'moveUp', arrow: '▲' },
    { dir: 'left', label: 'moveLeft', arrow: '◀' },
    { dir: 'down', label: 'moveDown', arrow: '▼' },
    { dir: 'right', label: 'moveRight', arrow: '▶' },
  ];
</script>

<div class="controller" class:docked>
  {#if docked}
    <div class="side">
      <span class="count">
        <small>{t('movesMade')}</small>
        <strong>{moves} / {bid}</strong>
      </span>
      <button class="secondary" onclick={() => {
        game.concede();
      }}>{t('concede')}</button>
    </div>
  {/if}
  <div class="pickers">
    {#each ROBOTS as robot (robot)}
      <button
        class="pick {robot}"
        aria-label={t(robotName[robot])}
        aria-pressed={game.selected === robot}
        onclick={() => {
          game.select(robot);
        }}
      ></button>
    {/each}
  </div>
  <div class="pad">
    {#each directions as { dir, label, arrow } (dir)}
      <button class="dir {dir} {game.selected}" aria-label={t(label)} onclick={() => {
        game.move(dir);
      }}>{arrow}</button>
    {/each}
    <span class="hub {game.selected}" aria-hidden="true"></span>
  </div>
</div>

<style>
  .controller {
    --key: 2.75rem;
    display: flex;
    flex-wrap: wrap;
    gap: 1rem 1.5rem;
    align-items: center;
  }
  /* The arrows as a cross around a hub in the selected robot's colour. */
  .pad {
    display: grid;
    grid-template-columns: repeat(3, var(--key));
    grid-template-rows: repeat(3, var(--key));
    gap: 0.25rem;
  }
  .dir {
    padding: 0;
    font-size: calc(var(--key) * 0.36);
    touch-action: manipulation;
  }
  .dir.up {
    grid-area: 1 / 2;
    border-radius: 12px 12px 6px 6px;
  }
  .dir.left {
    grid-area: 2 / 1;
    border-radius: 12px 6px 6px 12px;
  }
  .dir.right {
    grid-area: 2 / 3;
    border-radius: 6px 12px 12px 6px;
  }
  .dir.down {
    grid-area: 3 / 2;
    border-radius: 6px 6px 12px 12px;
  }
  .hub {
    grid-area: 2 / 2;
    border-radius: 50%;
    margin: 18%;
    background: var(--robot);
    box-shadow: inset 0 -3px 4px rgb(0 0 0 / 25%);
  }
  .pickers {
    display: grid;
    grid-template-columns: repeat(2, 2.75rem);
    gap: 0.5rem;
  }
  .pick {
    width: 2.75rem;
    height: 2.75rem;
    padding: 0;
    border-radius: 50%;
    background: var(--robot);
    box-shadow: inset 0 -3px 4px rgb(0 0 0 / 20%);
    touch-action: manipulation;
  }
  .pick:hover:not(:disabled) {
    background: var(--robot);
  }
  .pick[aria-pressed='true'] {
    box-shadow:
      0 0 0 3px var(--surface),
      0 0 0 6px var(--robot);
  }

  /* Docked: fixed to the bottom on narrow screens, thumbs on the arrows. */
  .docked {
    --key: 3.2rem;
    position: fixed;
    inset: auto 0 0;
    z-index: 10;
    justify-content: space-between;
    flex-wrap: nowrap;
    gap: 0.6rem;
    padding: 0.75rem 0.9rem calc(0.75rem + env(safe-area-inset-bottom));
    background: var(--surface);
    border-top: 1px solid var(--line);
    box-shadow: 0 -6px 20px rgb(0 0 0 / 12%);
  }
  .docked .side button {
    white-space: nowrap;
    padding: 0.45rem 0.7rem;
    font-size: 0.9rem;
  }
  .docked .side {
    display: grid;
    gap: 0.5rem;
    justify-items: start;
  }
  .count {
    display: grid;
    line-height: 1.2;
  }
  .count strong {
    font-size: 1.4rem;
    font-variant-numeric: tabular-nums;
  }
  .docked .pickers {
    grid-template-columns: repeat(2, 2.4rem);
  }
  .docked .pick {
    width: 2.4rem;
    height: 2.4rem;
  }
  @media (max-width: 23rem) {
    .docked {
      --key: 2.8rem;
      gap: 0.4rem;
    }
    .docked .pickers {
      grid-template-columns: repeat(2, 2.2rem);
    }
    .docked .pick {
      width: 2.2rem;
      height: 2.2rem;
    }
  }
</style>
