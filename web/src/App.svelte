<script lang="ts">
  import { Api } from './lib/api';
  import { AppModel } from './lib/app.svelte';
  import GameRoom from './lib/components/GameRoom.svelte';
  import GuestEntry from './lib/components/GuestEntry.svelte';
  import LanguagePicker from './lib/components/LanguagePicker.svelte';
  import Lobby from './lib/components/Lobby.svelte';
  import Logo from './lib/components/Logo.svelte';
  import { i18n, t } from './lib/i18n/locale.svelte';

  const app = new AppModel(new Api((input, init) => fetch(input, init)));
  void app.start();

  $effect(() => {
    document.documentElement.lang = i18n.locale;
    document.title = t('appTitle');
  });
</script>

<header class="bar">
  <div class="page bar-inner">
    <div class="brand">
      <Logo />
      <h1>{t('appTitle')}</h1>
    </div>
    <div class="tools">
      {#if app.identity}
        <span class="who">
          <span class="avatar" aria-hidden="true">{app.identity.nickname.slice(0, 1)}</span>
          {app.identity.nickname}
          <span class="badge">{t('guestBadge')}</span>
        </span>
      {/if}
      <LanguagePicker />
    </div>
  </div>
</header>

<main class="page">
  {#if app.error}
    <p class="error narrow" role="alert">{t(app.error)}</p>
  {/if}
  {#if app.view.name === 'loading'}
    <p class="muted narrow">{t('loading')}</p>
  {:else if app.view.name === 'guest'}
    <GuestEntry {app} />
  {:else if app.view.name === 'lobby'}
    <Lobby {app} />
  {:else}
    {#key app.view.id}
      <GameRoom {app} id={app.view.id} />
    {/key}
  {/if}
</main>

<style>
  .bar {
    background: var(--surface);
    border-bottom: 1px solid var(--line);
    margin-bottom: 1.25rem;
  }
  .bar-inner {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1.5rem;
    align-items: center;
    justify-content: space-between;
    padding-block: 0.75rem;
  }
  .brand {
    display: flex;
    gap: 0.6rem;
    align-items: center;
  }
  h1 {
    font-size: 1.3rem;
    letter-spacing: -0.01em;
  }
  .tools {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1.25rem;
    align-items: center;
  }
  .who {
    display: inline-flex;
    gap: 0.45rem;
    align-items: center;
    font-weight: 600;
  }
  .avatar {
    display: inline-grid;
    place-items: center;
    width: 1.8rem;
    height: 1.8rem;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent-strong);
    font-weight: 700;
    text-transform: uppercase;
  }
</style>
