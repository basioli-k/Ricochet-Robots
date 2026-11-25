<script lang="ts">
  import type { Board } from '../../protocol/Board';
  import type { Robot } from '../../protocol/Robot';
  import type { Robots } from '../../protocol/Robots';
  import type { Target } from '../../protocol/Target';
  import { cells, robotName, ROBOTS, sameTarget, targetGlyph } from '../game/board';
  import { t } from '../i18n/locale.svelte';

  let {
    board,
    robots,
    target,
    selected,
    onselect,
  }: {
    board: Board;
    robots: Robots | null;
    target: Target | null;
    selected: Robot;
    onselect: (robot: Robot) => void;
  } = $props();

  const grid = $derived(cells(board));
  const pct = (n: number) => `${String((n * 100) / board.size)}%`;
</script>

<div class="board" style:--size={board.size}>
  {#each grid as cell (cell.row * board.size + cell.col)}
    <div
      class="cell"
      class:blocked={cell.blocked}
      class:wall-up={cell.walls.has('up')}
      class:wall-down={cell.walls.has('down')}
      class:wall-left={cell.walls.has('left')}
      class:wall-right={cell.walls.has('right')}
    >
      {#if cell.target}
        <span
          class="target {cell.target.kind === 'colored' ? cell.target.robot : 'vortex'}"
          class:current={target !== null && sameTarget(cell.target, target)}
          aria-hidden="true">{targetGlyph(cell.target)}</span
        >
      {/if}
    </div>
  {/each}
  {#if robots}
    {#each ROBOTS as robot (robot)}
      <button
        class="robot {robot}"
        class:selected={robot === selected}
        style:top={pct(robots[robot].row)}
        style:left={pct(robots[robot].col)}
        style:width={pct(1)}
        style:height={pct(1)}
        aria-label={t(robotName[robot])}
        aria-pressed={robot === selected}
        onclick={() => {
          onselect(robot);
        }}
      ></button>
    {/each}
  {/if}
</div>

<style>
  .board {
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--size), 1fr);
    width: min(100%, 36rem);
    aspect-ratio: 1;
    border: 3px solid var(--wall);
    background: var(--board);
  }
  .cell {
    position: relative;
    border: 1px solid var(--grid);
    display: grid;
    place-items: center;
  }
  .wall-up {
    border-top: 3px solid var(--wall);
  }
  .wall-down {
    border-bottom: 3px solid var(--wall);
  }
  .wall-left {
    border-left: 3px solid var(--wall);
  }
  .wall-right {
    border-right: 3px solid var(--wall);
  }
  .blocked {
    background: var(--wall);
  }
  .target {
    font-size: clamp(0.6rem, 2.4vw, 1.2rem);
    line-height: 1;
    opacity: 0.55;
  }
  .target.current {
    opacity: 1;
    outline: 2px solid currentColor;
    border-radius: 0.2rem;
  }
  .robot {
    position: absolute;
    padding: 0;
    border-radius: 50%;
    border: 2px solid var(--wall);
    transform: scale(0.75);
    /* The slide: robots glide to their new cell. */
    transition:
      top 0.25s ease-out,
      left 0.25s ease-out;
  }
  .robot.selected {
    outline: 3px solid var(--text);
    outline-offset: 1px;
  }
  .red {
    color: var(--red);
    background: var(--red);
  }
  .blue {
    color: var(--blue);
    background: var(--blue);
  }
  .green {
    color: var(--green);
    background: var(--green);
  }
  .yellow {
    color: var(--yellow);
    background: var(--yellow);
  }
  .target.red,
  .target.blue,
  .target.green,
  .target.yellow,
  .target.vortex {
    background: none;
  }
  .vortex {
    color: var(--text);
  }
  @media (prefers-reduced-motion: reduce) {
    .robot {
      transition: none;
    }
  }
</style>
