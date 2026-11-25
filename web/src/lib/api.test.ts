import { describe, expect, it } from 'vitest';
import { Api, ApiError, type Fetch } from './api';
import { isValidNickname } from './nickname';

interface Call {
  path: string;
  init: RequestInit | undefined;
}

/** A fetch that answers each call with the next response and records it. */
function scripted(...responses: Response[]): Fetch & { calls: Call[] } {
  const calls: Call[] = [];
  const fake = (path: string, init?: RequestInit) => {
    calls.push({ path, init });
    const next = responses.shift();
    return next ? Promise.resolve(next) : Promise.reject(new TypeError('offline'));
  };
  return Object.assign(fake, { calls });
}

const json = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });

const header = (call: Call | undefined, name: string) =>
  (call?.init?.headers as Record<string, string> | undefined)?.[name];

const ana = { kind: 'guest', id: 'a'.repeat(32), nickname: 'Ana1' } as const;

describe('Api', () => {
  it('sends the session CSRF token, then the renewed one after starting', async () => {
    const fetch = scripted(
      json({ csrf: 'first', identity: null }),
      json({ identity: ana, csrf: 'second' }),
      json({ id: 'r'.repeat(32) }, 201),
    );
    const api = new Api(fetch);
    expect((await api.session()).identity).toBeNull();
    expect((await api.startGuest('Ana1')).identity).toEqual(ana);
    expect(await api.openRoom()).toEqual({ id: 'r'.repeat(32) });

    const [session, guest, room] = fetch.calls;
    expect(header(session, 'x-csrf-token')).toBeUndefined();
    expect(guest?.path).toBe('/api/guest');
    expect(header(guest, 'x-csrf-token')).toBe('first');
    expect(guest?.init?.body).toBe(JSON.stringify({ nickname: 'Ana1' }));
    expect(header(room, 'x-csrf-token')).toBe('second');
    expect(room?.init?.credentials).toBe('same-origin');
  });

  it('renews a stale token once and retries', async () => {
    const fetch = scripted(
      json({ csrf: 'old', identity: null }),
      json({ error: 'csrf_mismatch' }, 403),
      json({ csrf: 'new', identity: null }),
      json({ identity: ana, csrf: 'after' }),
    );
    const api = new Api(fetch);
    await api.session();
    expect((await api.startGuest('Ana1')).csrf).toBe('after');
    expect(fetch.calls.map((c) => c.path)).toEqual([
      '/api/session',
      '/api/guest',
      '/api/session',
      '/api/guest',
    ]);
    expect(header(fetch.calls[3], 'x-csrf-token')).toBe('new');
  });

  it('fetches a token first when it has none, and gives up after one retry', async () => {
    const fetch = scripted(
      json({ csrf: 'fresh', identity: null }),
      json({ error: 'csrf_mismatch' }, 403),
      json({ csrf: 'again', identity: null }),
      json({ error: 'csrf_mismatch' }, 403),
    );
    const api = new Api(fetch);
    await expect(api.openRoom()).rejects.toEqual(new ApiError('csrf_mismatch', 403));
    expect(header(fetch.calls[1], 'x-csrf-token')).toBe('fresh');
    expect(fetch.calls).toHaveLength(4);
  });

  it('lists rooms', async () => {
    const api = new Api(scripted(json([{ id: 'x', members: 2 }])));
    expect(await api.rooms()).toEqual([{ id: 'x', members: 2 }]);
  });

  it('turns refusals into their codes', async () => {
    const api = new Api(scripted(json({ error: 'nickname_taken' }, 409)));
    await expect(api.startGuest('ana')).rejects.toEqual(new ApiError('nickname_taken', 409));
  });

  it('reports a body without a code, and an unreachable server', async () => {
    const api = new Api(scripted(new Response('oops', { status: 502 })));
    await expect(api.rooms()).rejects.toEqual(new ApiError('unknown', 502));
    await expect(api.rooms()).rejects.toEqual(new ApiError('network', 0));
  });
});

describe('isValidNickname', () => {
  it('follows the server rule', () => {
    for (const ok of ['abc', 'Visitor1', 'a'.repeat(15)]) expect(isValidNickname(ok)).toBe(true);
    for (const bad of ['ab', 'a'.repeat(16), 'a b', 'ana!', 'Ćiro', '']) {
      expect(isValidNickname(bad)).toBe(false);
    }
  });
});
