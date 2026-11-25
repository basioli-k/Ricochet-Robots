<script lang="ts">
  import { Api } from './lib/api';
  import { AppModel } from './lib/app.svelte';
  import GuestEntry from './lib/components/GuestEntry.svelte';
  import LanguagePicker from './lib/components/LanguagePicker.svelte';
  import Lobby from './lib/components/Lobby.svelte';
  import GameRoom from './lib/components/GameRoom.svelte';
  import { i18n, t } from './lib/i18n/locale.svelte';

  const app = new AppModel(new Api((input, init) => fetch(input, init)));
  void app.start();

  $effect(() => {
    document.documentElement.lang = i18n.locale;
    document.title = t('appTitle');
  });
</script>

<header>
  <h1>{t('appTitle')}</h1>
  <LanguagePicker />
</header>
<main>
  {#if app.error}
    <p class="error" role="alert">{t(app.error)}</p>
  {/if}
  {#if app.view.name === 'loading'}
    <p>{t('loading')}</p>
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
