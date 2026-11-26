<script lang="ts">
  import { untrack } from 'svelte';
  import type { Robot } from '../../protocol/Robot';
  import type { AppModel } from '../app.svelte';
  import { robotName, shapeName, targetGlyph } from '../game/board';
  import { isTyping, keyAction } from '../game/keys';
  import { GameModel, socketUrl } from '../game/model.svelte';
  import { nameOf, standings } from '../game/state';
  import { t } from '../i18n/locale.svelte';
  import BoardView from './BoardView.svelte';
  import Controller from './Controller.svelte';

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
  /** Seconds left at which the countdown turns urgent. */
  const HURRY_S = 10;

  function onkeydown(event: KeyboardEvent) {
    const typing = isTyping(event.target instanceof Element ? event.target.tagName : undefined);
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

<div class="game" class:docked={game.demonstrating}>
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
      <p class="hint keys">{t('keyboardHint')}</p>
      <p class="hint touch">{t('touchHint')}</p>
    {:else}
      <div class="card loading"><p class="muted">{t('connecting')}</p></div>
    {/if}
  </div>

  <aside>
    <div class="room-bar">
      <span class="status-dot" class:live={game.status === 'open'} aria-hidden="true"></span>
      <span class="code">{t('roomCode', { code: id.slice(0, 6) })}</span>
      <button class="secondary small" onclick={leave}>{t('leaveRoom')}</button>
    </div>

    {#if game.error}
      <p class="error" role="alert">{t(game.error)}</p>
    {/if}
    {#if game.status === 'closed'}
      <p class="error" role="alert">
        {t('disconnected')}
        <button class="small" onclick={() => {
          game.connect();
        }}>{t('reconnect')}</button>
      </p>
    {/if}

    {#if view}
      <section class="card status" aria-live="polite">
        {#if !round}
          <p class="phase">{t('phaseWaiting')}</p>
          <button class="wide" onclick={() => {
            game.start();
          }}>{t('startGame')}</button>
        {:else}
          <div class="target-row">
            <span class="target-chip {round.target.kind === 'colored' ? round.target.robot : 'vortex'}"
              >{targetGlyph(round.target)}</span
            >
            <div>
              <h3>{t('target')}</h3>
              <p class="target-name">
                {#if round.target.kind === 'colored'}
                  {t(robotName[round.target.robot])} · {t(shapeName[round.target.shape])}
                {:else}
                  {t('vortex')}
                {/if}
              </p>
            </div>
          </div>

          {#if secondsLeft !== null}
            <!-- Ticks every second: kept out of the status announcements. -->
            <p class="clock" class:hurry={secondsLeft <= HURRY_S} aria-live="off">
              {t('timeLeft', { seconds: secondsLeft })}
            </p>
          {/if}

          {#if round.over}
            <p class="phase"><strong>{t('gameOver')}</strong></p>
          {:else if round.phase.kind === 'bidding'}
            <p class="phase">{t('phaseBidding')}</p>
            <form class="bid" onsubmit={placeBid}>
              <label>
                {t('bidLabel')}
                <input type="number" min="1" max="255" bind:value={bid} name="bid" required />
              </label>
              <button type="submit">{t('placeBid')}</button>
            </form>
          {:else if round.phase.kind === 'demonstrating'}
            {#if game.demonstrating}
              <p class="phase"><strong>{t('yourTurn')}</strong></p>
              <div class="wide-only">
                <Controller {game} />
                <button class="secondary" onclick={() => {
                  game.concede();
                }}>{t('concede')}</button>
              </div>
            {:else}
              <p class="phase">{t('demonstratingOther', { name: nameOf(view, round.phase.player) })}</p>
            {/if}
            <p class="moves">{t('movesMade')}: {round.phase.moves_made} / {round.phase.bid}</p>
          {:else if round.phase.kind === 'solved'}
            <p class="phase result"><strong>{t('roundSolved', { name: nameOf(view, round.phase.winner) })}</strong></p>
            <p class="moves">{t('movesMade')}: {round.phase.moves}</p>
            <p class="muted">{t('nextRound')}</p>
          {:else}
            <p class="phase result"><strong>{t('roundUnsolved')}</strong></p>
            <p class="muted">{t('nextRound')}</p>
          {/if}

          {#if round.phase.kind === 'bidding' || round.phase.kind === 'demonstrating'}
            <h3>{t('bidsTitle')}</h3>
            {#if round.bids.length === 0}
              <p class="muted">{t('noBidsYet')}</p>
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

      <section class="card">
        <h3>{t('scoresTitle')}</h3>
        <ul class="scores">
          {#each standings(view) as score (score.player)}
            <li>
              <span class="avatar" aria-hidden="true">{nameOf(view, score.player).slice(0, 1)}</span>
              <span class="name">
                {nameOf(view, score.player)}
                {#if score.player === view.you}<span class="badge">{t('you')}</span>{/if}
              </span>
              <strong class="tokens">{score.tokens}</strong>
            </li>
          {/each}
        </ul>
      </section>

      <section class="card chat">
        <h3>{t('chatTitle')}</h3>
        <ul>
          {#each view.chat as line, i (i)}
            <li class:mine={line.player === view.you}>
              <strong>{nameOf(view, line.player)}:</strong> {line.text}
            </li>
          {/each}
        </ul>
        <form class="send" onsubmit={sendChat}>
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
  </aside>
</div>

{#if game.demonstrating && round?.phase.kind === 'demonstrating'}
  <div class="narrow-only">
    <Controller {game} docked moves={round.phase.moves_made} bid={round.phase.bid} />
  </div>
{/if}

<style>
  .game {
    display: grid;
    gap: 1.25rem;
    grid-template-columns: minmax(0, 40rem) minmax(17rem, 24rem);
    justify-content: center;
    align-items: start;
  }
  @media (max-width: 62rem) {
    .game {
      grid-template-columns: minmax(0, 40rem);
    }
  }
  /* Grid items may shrink below their content's width on narrow screens. */
  .play,
  aside {
    min-width: 0;
  }
  .play {
    display: grid;
    gap: 0.5rem;
  }
  .hint {
    margin: 0;
    color: var(--muted);
    font-size: 0.85rem;
    text-align: center;
  }
  /* Touch screens without a hovering pointer get the tap hint instead. */
  .hint.touch {
    display: none;
  }
  @media (hover: none) {
    .hint.keys {
      display: none;
    }
    .hint.touch {
      display: block;
    }
  }
  .loading {
    aspect-ratio: 1;
    display: grid;
    place-items: center;
  }
  aside {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 0.9rem;
  }
  aside .error {
    margin: 0;
  }
  .room-bar {
    display: flex;
    gap: 0.6rem;
    align-items: center;
  }
  .code {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    flex: 1;
  }
  .status-dot {
    width: 0.6rem;
    height: 0.6rem;
    border-radius: 50%;
    background: var(--muted);
  }
  .status-dot.live {
    background: var(--ok);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--ok) 25%, transparent);
  }
  button.small {
    padding: 0.3rem 0.7rem;
    font-size: 0.85rem;
  }
  .card p {
    margin: 0;
  }
  .status {
    display: grid;
    gap: 0.85rem;
  }
  .phase {
    font-size: 1.05rem;
  }
  .result {
    font-size: 1.2rem;
  }
  .wide {
    width: 100%;
  }
  .target-row {
    display: flex;
    gap: 0.8rem;
    align-items: center;
  }
  .target-chip {
    display: grid;
    place-items: center;
    width: 3rem;
    height: 3rem;
    border-radius: 12px;
    font-size: 1.7rem;
    line-height: 1;
    color: #fff;
    background: var(--robot);
    flex: none;
  }
  .target-chip.vortex {
    color: var(--surface);
  }
  .target-name {
    font-weight: 600;
  }
  .clock {
    font-size: 1.6rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .clock.hurry {
    color: var(--danger-text);
  }
  .bid {
    display: flex;
    gap: 0.5rem;
    align-items: end;
  }
  .bid label {
    flex: 1;
  }
  .wide-only {
    display: grid;
    gap: 0.85rem;
    justify-items: start;
  }
  .narrow-only {
    display: none;
  }
  /* Phones get the docked controller instead, and room for it below. */
  @media (max-width: 62rem) {
    .wide-only {
      display: none;
    }
    .narrow-only {
      display: block;
    }
    .game.docked {
      padding-bottom: calc(12rem + env(safe-area-inset-bottom));
    }
  }
  .moves {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .bids {
    margin: 0;
    padding-left: 1.4rem;
    display: grid;
    gap: 0.2rem;
    font-variant-numeric: tabular-nums;
  }
  .bids li:first-child {
    font-weight: 700;
  }
  .scores {
    list-style: none;
    margin: 0.6rem 0 0;
    padding: 0;
    display: grid;
    gap: 0.5rem;
  }
  .scores li {
    display: flex;
    gap: 0.6rem;
    align-items: center;
  }
  .avatar {
    display: inline-grid;
    place-items: center;
    width: 1.9rem;
    height: 1.9rem;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent-strong);
    font-weight: 700;
    text-transform: uppercase;
    flex: none;
  }
  .name {
    flex: 1;
  }
  .tokens {
    font-size: 1.1rem;
    font-variant-numeric: tabular-nums;
  }
  .chat ul {
    list-style: none;
    margin: 0.6rem 0;
    padding: 0;
    display: grid;
    gap: 0.4rem;
    max-height: 14rem;
    overflow-y: auto;
    overflow-wrap: anywhere;
  }
  .chat li {
    background: var(--surface-2);
    border-radius: 12px 12px 12px 4px;
    padding: 0.4rem 0.7rem;
    justify-self: start;
    max-width: 90%;
  }
  .chat li.mine {
    background: var(--accent-soft);
    border-radius: 12px 12px 4px 12px;
    justify-self: end;
  }
  .send {
    display: flex;
    gap: 0.5rem;
  }
  .send input {
    flex: 1;
    min-width: 0;
  }
</style>
