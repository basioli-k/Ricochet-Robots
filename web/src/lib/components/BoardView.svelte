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
  const colorOf = (target: Target) => (target.kind === 'colored' ? target.robot : 'vortex');

  /** The blocked middle, where the current target is shown large. */
  const middle = $derived.by(() => {
    if (board.blocked.length === 0) return null;
    const rows = board.blocked.map((p) => p.row);
    const cols = board.blocked.map((p) => p.col);
    const top = Math.min(...rows);
    const left = Math.min(...cols);
    return {
      top,
      left,
      height: Math.max(...rows) - top + 1,
      width: Math.max(...cols) - left + 1,
    };
  });
</script>

<div class="frame">
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
            class="target {colorOf(cell.target)}"
            class:current={target !== null && sameTarget(cell.target, target)}
            aria-hidden="true">{targetGlyph(cell.target)}</span
          >
        {/if}
      </div>
    {/each}

    {#if middle}
      <div
        class="middle"
        style:top={pct(middle.top)}
        style:left={pct(middle.left)}
        style:width={pct(middle.width)}
        style:height={pct(middle.height)}
        aria-hidden="true"
      >
        {#if target}
          <span class="middle-target {colorOf(target)}">{targetGlyph(target)}</span>
        {/if}
      </div>
    {/if}

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
</div>

<style>
  .frame {
    background: var(--board-frame);
    border-radius: 16px;
    padding: 0.6rem;
    box-shadow: var(--shadow);
    width: min(100%, 40rem);
  }
  .board {
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--size), 1fr);
    aspect-ratio: 1;
    background: var(--board);
    border-radius: 6px;
    overflow: hidden;
  }
  .cell {
    position: relative;
    border: 0.5px solid var(--grid);
    display: grid;
    place-items: center;
  }
  /* Walls: a thick bar along the cell's side. */
  .wall-up {
    border-top: 4px solid var(--wall);
  }
  .wall-down {
    border-bottom: 4px solid var(--wall);
  }
  .wall-left {
    border-left: 4px solid var(--wall);
  }
  .wall-right {
    border-right: 4px solid var(--wall);
  }
  .blocked {
    background: var(--board-frame);
    border-color: var(--board-frame);
  }
  .target {
    display: grid;
    place-items: center;
    width: 78%;
    height: 78%;
    border-radius: 22%;
    font-size: clamp(0.55rem, 2.2vw, 1.15rem);
    line-height: 1;
    color: var(--robot);
    background: color-mix(in srgb, var(--robot) 14%, transparent);
  }
  .target.current {
    color: #fff;
    background: var(--robot);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--robot) 35%, transparent);
    animation: pulse 1.6s ease-in-out infinite;
  }
  .target.vortex.current {
    color: var(--board);
  }
  .middle {
    position: absolute;
    display: grid;
    place-items: center;
    pointer-events: none;
  }
  .middle-target {
    font-size: clamp(1.2rem, 5vw, 2.6rem);
    line-height: 1;
    color: var(--robot);
    filter: drop-shadow(0 0 6px color-mix(in srgb, var(--robot) 60%, transparent));
  }
  .robot {
    position: absolute;
    padding: 0;
    border: 0;
    border-radius: 50%;
    transform: scale(0.74);
    background: radial-gradient(
      circle at 35% 30%,
      color-mix(in srgb, var(--robot) 55%, white) 0 18%,
      var(--robot) 55%,
      color-mix(in srgb, var(--robot) 70%, black) 100%
    );
    box-shadow:
      0 2px 3px rgb(0 0 0 / 35%),
      inset 0 -2px 3px rgb(0 0 0 / 20%);
    /* The slide: robots glide to their new cell. */
    transition:
      top 0.28s cubic-bezier(0.3, 0.7, 0.4, 1),
      left 0.28s cubic-bezier(0.3, 0.7, 0.4, 1),
      transform 0.15s;
  }
  .robot:hover {
    background: radial-gradient(
      circle at 35% 30%,
      color-mix(in srgb, var(--robot) 55%, white) 0 18%,
      var(--robot) 55%,
      color-mix(in srgb, var(--robot) 70%, black) 100%
    );
    transform: scale(0.82);
  }
  .robot.selected {
    transform: scale(0.82);
    box-shadow:
      0 0 0 3px var(--board),
      0 0 0 6px var(--robot),
      0 3px 8px rgb(0 0 0 / 35%);
  }
  @keyframes pulse {
    50% {
      box-shadow: 0 0 0 6px color-mix(in srgb, var(--robot) 15%, transparent);
    }
  }
</style>
