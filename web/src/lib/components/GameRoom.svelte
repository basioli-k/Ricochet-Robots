<script lang="ts">
  import { untrack } from 'svelte';
  import type { Dir } from '../../protocol/Dir';
  import type { Robot } from '../../protocol/Robot';
  import type { AppModel } from '../app.svelte';
  import { robotName, ROBOTS, shapeName, targetGlyph } from '../game/board';
  import { keyAction } from '../game/keys';
  import { GameModel, socketUrl } from '../game/model.svelte';
  import { nameOf, standings } from '../game/state';
  import { t } from '../i18n/locale.svelte';
  import BoardView from './BoardView.svelte';

  let { app, id }: { app: AppModel; id: string } = $props();

  // The page is keyed by room, so the ID never changes under it.
  const game = new GameModel(socketUrl(window.location, untrack(() => id)), (url) => new WebSocket(url));
  let bid = $state('');
  let message = $state('');
  let now = $state(Date.now());

  $effect(() => {
    game.connect();
    const clock = setInterval(() => {
      now = Date.now();
    }, 250);
    return () => {
      clearInterval(clock);
      game.leave();
    };
  });

  const view = $derived(game.state);
  const round = $derived(view?.round ?? null);
  const secondsLeft = $derived(
    round?.deadline == null ? null : Math.max(0, Math.ceil((round.deadline - now) / 1000)),
  );
  const directions: { dir: Dir; label: 'moveUp' | 'moveDown' | 'moveLeft' | 'moveRight'; arrow: string }[] = [
    { dir: 'up', label: 'moveUp', arrow: '↑' },
    { dir: 'left', label: 'moveLeft', arrow: '←' },
    { dir: 'down', label: 'moveDown', arrow: '↓' },
    { dir: 'right', label: 'moveRight', arrow: '→' },
  ];

  function onkeydown(event: KeyboardEvent) {
    const typing =
      event.target instanceof HTMLInputElement ||
      event.target instanceof HTMLTextAreaElement ||
      event.target instanceof HTMLSelectElement;
    if (typing || event.ctrlKey || event.metaKey || event.altKey) return;
    const action = keyAction(event.key);
    if (!action) return;
    if ('select' in action) game.select(action.select);
    else if (game.demonstrating) game.move(action.move);
    else return;
    event.preventDefault();
  }

  function placeBid(event: SubmitEvent) {
    event.preventDefault();
    game.bid(Number(bid));
  }

  function sendChat(event: SubmitEvent) {
    event.preventDefault();
    if (game.chat(message)) message = '';
  }

  function leave() {
    game.leave();
    void app.backToLobby();
  }
</script>

<svelte:window {onkeydown} />

<div class="game">
  <div class="play">
    {#if view}
      <BoardView
        board={view.board}
        robots={round?.robots ?? null}
        target={round?.target ?? null}
        selected={game.selected}
        onselect={(robot: Robot) => {
          game.select(robot);
        }}
      />
    {:else}
      <p>{t('connecting')}</p>
    {/if}
  </div>

  <aside>
    <p class="code">{t('roomCode', { code: id.slice(0, 6) })}</p>
    {#if game.error}
      <p class="error" role="alert">{t(game.error)}</p>
    {/if}
    {#if game.status === 'closed'}
      <p class="error" role="alert">
        {t('disconnected')}
        <button onclick={() => {
          game.connect();
        }}>{t('reconnect')}</button>
      </p>
    {/if}

    {#if view}
      <section class="status" aria-live="polite">
        {#if !round}
          <p>{t('phaseWaiting')}</p>
          <button onclick={() => {
            game.start();
          }}>{t('startGame')}</button>
        {:else}
          <p class="target">
            {t('target')}:
            <span class="glyph {round.target.kind === 'colored' ? round.target.robot : 'vortex'}"
              >{targetGlyph(round.target)}</span
            >
            {#if round.target.kind === 'colored'}
              {t(robotName[round.target.robot])} · {t(shapeName[round.target.shape])}
            {:else}
              {t('vortex')}
            {/if}
          </p>
          {#if round.over}
            <p><strong>{t('gameOver')}</strong></p>
          {:else if round.phase.kind === 'bidding'}
            <p>{t('phaseBidding')}</p>
            <form class="inline" onsubmit={placeBid}>
              <label>
                {t('bidLabel')}
                <input type="number" min="1" max="255" bind:value={bid} name="bid" required />
              </label>
              <button type="submit">{t('placeBid')}</button>
            </form>
          {:else if round.phase.kind === 'demonstrating'}
            {#if game.demonstrating}
              <p><strong>{t('yourTurn')}</strong></p>
              <div class="pad">
                {#each directions as { dir, label, arrow } (dir)}
                  <button aria-label={t(label)} onclick={() => {
                    game.move(dir);
                  }}>{arrow}</button>
                {/each}
              </div>
              <p class="robots">
                {#each ROBOTS as robot (robot)}
                  <button
                    class="pick {robot}"
                    aria-pressed={game.selected === robot}
                    onclick={() => {
                      game.select(robot);
                    }}>{t(robotName[robot])}</button
                  >
                {/each}
              </p>
              <button onclick={() => {
                game.concede();
              }}>{t('concede')}</button>
            {:else}
              <p>{t('demonstratingOther', { name: nameOf(view, round.phase.player) })}</p>
            {/if}
            <p>{t('movesMade')}: {round.phase.moves_made} / {round.phase.bid}</p>
          {:else if round.phase.kind === 'solved'}
            <p><strong>{t('roundSolved', { name: nameOf(view, round.phase.winner) })}</strong></p>
            <p>{t('movesMade')}: {round.phase.moves}</p>
            <p>{t('nextRound')}</p>
          {:else}
            <p><strong>{t('roundUnsolved')}</strong></p>
            <p>{t('nextRound')}</p>
          {/if}
          {#if secondsLeft !== null}
            <p class="clock">{t('timeLeft', { seconds: secondsLeft })}</p>
          {/if}
          {#if round.phase.kind === 'bidding' || round.phase.kind === 'demonstrating'}
            <h3>{t('bidsTitle')}</h3>
            {#if round.bids.length === 0}
              <p>{t('noBidsYet')}</p>
            {:else}
              <ol class="bids">
                {#each round.bids as b (b.player)}
                  <li>{nameOf(view, b.player)}: {b.moves}</li>
                {/each}
              </ol>
            {/if}
          {/if}
        {/if}
      </section>

      <section>
        <h3>{t('scoresTitle')}</h3>
        <ul class="scores">
          {#each standings(view) as score (score.player)}
            <li>
              <span>
                {nameOf(view, score.player)}
                {#if score.player === view.you}<span class="badge">{t('you')}</span>{/if}
              </span>
              <strong>{score.tokens}</strong>
            </li>
          {/each}
        </ul>
      </section>

      <section class="chat">
        <h3>{t('chatTitle')}</h3>
        <ul>
          {#each view.chat as line, i (i)}
            <li><strong>{nameOf(view, line.player)}:</strong> {line.text}</li>
          {/each}
        </ul>
        <form class="inline" onsubmit={sendChat}>
          <input
            bind:value={message}
            name="chat"
            maxlength="200"
            placeholder={t('chatPlaceholder')}
            aria-label={t('chatTitle')}
          />
          <button type="submit">{t('send')}</button>
        </form>
      </section>
    {/if}

    <p class="hint">{t('keyboardHint')}</p>
    <button onclick={leave}>{t('leaveRoom')}</button>
  </aside>
</div>

<style>
  .game {
    display: grid;
    gap: 1rem;
    grid-template-columns: minmax(0, 2fr) minmax(15rem, 1fr);
    align-items: start;
  }
  @media (max-width: 52rem) {
    .game {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  aside {
    display: grid;
    gap: 0.75rem;
  }
  h3 {
    margin: 0 0 0.5rem;
    font-size: 1rem;
  }
  .inline {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: end;
  }
  .inline input {
    flex: 1;
    min-width: 5rem;
  }
  .pad {
    display: grid;
    grid-template-columns: repeat(4, 2.5rem);
    gap: 0.25rem;
  }
  .robots {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }
  .pick[aria-pressed='true'] {
    outline: 3px solid var(--text);
  }
  .pick.red {
    background: var(--red);
  }
  .pick.blue {
    background: var(--blue);
  }
  .pick.green {
    background: var(--green);
  }
  .pick.yellow {
    background: var(--yellow);
    color: #222;
  }
  .glyph.red {
    color: var(--red);
  }
  .glyph.blue {
    color: var(--blue);
  }
  .glyph.green {
    color: var(--green);
  }
  .glyph.yellow {
    color: var(--yellow);
  }
  .bids,
  .scores,
  .chat ul {
    margin: 0;
    padding-left: 1.25rem;
  }
  .scores {
    list-style: none;
    padding: 0;
  }
  .scores li {
    display: flex;
    gap: 0.5rem;
    justify-content: space-between;
  }
  .chat ul {
    list-style: none;
    padding: 0;
    max-height: 12rem;
    overflow-y: auto;
    overflow-wrap: anywhere;
  }
  .hint {
    color: var(--muted);
    font-size: 0.85rem;
  }
</style>
