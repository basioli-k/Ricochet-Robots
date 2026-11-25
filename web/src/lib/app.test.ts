import { describe, expect, it } from 'vitest';
import { ApiError } from './api';
import { AppModel, type LobbyApi } from './app.svelte';
import type { Identity } from '../protocol/Identity';

const ana: Identity = { kind: 'guest', id: 'a'.repeat(32), nickname: 'Ana1' };
const room = { id: 'r'.repeat(32), members: 1 };

/** An API whose answers each test sets; it records the nicknames sent. */
function fakeApi(overrides: Partial<LobbyApi> = {}): LobbyApi & { nicknames: string[] } {
  const nicknames: string[] = [];
  return {
    nicknames,
    session: () => Promise.resolve({ csrf: 'c', identity: null }),
    startGuest: (nickname) => {
      nicknames.push(nickname);
      return Promise.resolve({ identity: ana, csrf: 'd' });
    },
    rooms: () => Promise.resolve([room]),
    openRoom: () => Promise.resolve({ id: room.id }),
    ...overrides,
  };
}

describe('AppModel', () => {
  it('asks a new visitor for a nickname, then shows the lobby', async () => {
    const api = fakeApi();
    const app = new AppModel(api);
    await app.start();
    expect(app.view).toEqual({ name: 'guest' });
    await app.play('Ana1');
    expect(api.nicknames).toEqual(['Ana1']);
    expect(app.identity).toEqual(ana);
    expect(app.view).toEqual({ name: 'lobby' });
    expect(app.rooms).toEqual([room]);
  });

  it('takes a returning player straight to the lobby', async () => {
    const app = new AppModel(fakeApi({ session: () => Promise.resolve({ csrf: 'c', identity: ana }) }));
    await app.start();
    expect(app.view).toEqual({ name: 'lobby' });
  });

  it('checks the nickname before asking the server', async () => {
    const api = fakeApi();
    const app = new AppModel(api);
    await app.play('a b');
    expect(app.error).toBe('errorNicknameInvalid');
    expect(api.nicknames).toEqual([]);
  });

  it('shows the server refusal as a message', async () => {
    const app = new AppModel(
      fakeApi({ startGuest: () => Promise.reject(new ApiError('nickname_taken', 409)) }),
    );
    await app.start();
    await app.play('Ana1');
    expect(app.error).toBe('errorNicknameTaken');
    expect(app.view).toEqual({ name: 'guest' });
    expect(app.busy).toBe(false);
  });

  it('opens a room and goes back to the lobby', async () => {
    const app = new AppModel(fakeApi({ session: () => Promise.resolve({ csrf: 'c', identity: ana }) }));
    await app.start();
    await app.openRoom();
    expect(app.view).toEqual({ name: 'room', id: room.id });
    await app.backToLobby();
    expect(app.view).toEqual({ name: 'lobby' });
  });

  it('returns to the guest page when the session has expired', async () => {
    const app = new AppModel(
      fakeApi({
        session: () => Promise.resolve({ csrf: 'c', identity: ana }),
        openRoom: () => Promise.reject(new ApiError('not_signed_in', 401)),
      }),
    );
    await app.start();
    await app.openRoom();
    expect(app.view).toEqual({ name: 'guest' });
    expect(app.identity).toBeNull();
    expect(app.error).toBe('errorNotSignedIn');
  });

  it('shows the guest page with a message when the server is unreachable', async () => {
    const app = new AppModel(fakeApi({ session: () => Promise.reject(new ApiError('network', 0)) }));
    await app.start();
    expect(app.view).toEqual({ name: 'guest' });
    expect(app.error).toBe('errorNetwork');
  });

  it('polls the lobby quietly and never blocks the player', async () => {
    let failing = false;
    const app = new AppModel(
      fakeApi({
        session: () => Promise.resolve({ csrf: 'c', identity: ana }),
        rooms: () => (failing ? Promise.reject(new ApiError('network', 0)) : Promise.resolve([room])),
      }),
    );
    await app.start();
    app.error = 'errorRoomLimit';
    failing = true;
    await app.poll();
    // A failed poll neither clears the message nor adds its own.
    expect(app.error).toBe('errorRoomLimit');
    expect(app.busy).toBe(false);
  });

  it('leaves the lobby from a poll when the session has expired', async () => {
    let expired = false;
    const app = new AppModel(
      fakeApi({
        session: () => Promise.resolve({ csrf: 'c', identity: ana }),
        rooms: () =>
          expired ? Promise.reject(new ApiError('not_signed_in', 401)) : Promise.resolve([room]),
      }),
    );
    await app.start();
    expired = true;
    await app.poll();
    expect(app.view).toEqual({ name: 'guest' });
  });

  it('runs one action at a time', async () => {
    let release = () => {};
    const app = new AppModel(
      fakeApi({
        rooms: () =>
          new Promise((resolve) => {
            release = () => {
              resolve([]);
            };
          }),
      }),
    );
    const first = app.refresh();
    expect(app.busy).toBe(true);
    await app.refresh();
    release();
    await first;
    expect(app.busy).toBe(false);
  });
});
