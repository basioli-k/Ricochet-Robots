import { describe, expect, it, vi } from 'vitest';
import type { ServerMessage } from '../../protocol/ServerMessage';
import { GameModel, REJOIN_DELAY_MS, socketUrl, type Socket } from './model.svelte';

const ANA = 'a'.repeat(32);

class FakeSocket implements Socket {
  sent: unknown[] = [];
  closed = false;
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;

  send(data: string): void {
    this.sent.push(JSON.parse(data));
  }
  close(): void {
    this.closed = true;
  }
  server(message: ServerMessage): void {
    this.onmessage?.(new MessageEvent('message', { data: JSON.stringify(message) }));
  }
}

function connected(): { game: GameModel; socket: FakeSocket; urls: string[] } {
  const sockets: FakeSocket[] = [];
  const urls: string[] = [];
  const game = new GameModel('ws://x/api/rooms/r/ws', (url) => {
    urls.push(url);
    const socket = new FakeSocket();
    sockets.push(socket);
    return socket;
  });
  game.connect();
  const socket = sockets[0];
  if (!socket) throw new Error('no socket');
  socket.onopen?.(new Event('open'));
  return { game, socket, urls };
}

const snapshot: ServerMessage = {
  type: 'snapshot',
  you: ANA,
  room: {
    members: [{ id: ANA, name: 'Ana1', is_guest: true }],
    board: { size: 16, walls: [], blocked: [], targets: [] },
    game: null,
  },
};

describe('socketUrl', () => {
  it('follows the page scheme and host', () => {
    expect(socketUrl({ protocol: 'https:', host: 'rr.example' }, 'abc')).toBe(
      'wss://rr.example/api/rooms/abc/ws',
    );
    expect(socketUrl({ protocol: 'http:', host: 'localhost:5173' }, 'a/b')).toBe(
      'ws://localhost:5173/api/rooms/a%2Fb/ws',
    );
  });
});

describe('GameModel', () => {
  it('opens the socket and applies what the server sends', () => {
    const { game, socket, urls } = connected();
    expect(urls).toEqual(['ws://x/api/rooms/r/ws']);
    expect(game.status).toBe('open');
    socket.server(snapshot);
    expect(game.state?.you).toBe(ANA);
  });

  it('sends intents, the selected robot with each move', () => {
    const { game, socket } = connected();
    game.start();
    game.bid(4);
    game.select('blue');
    game.move('left');
    game.concede();
    game.chat('  ');
    game.chat('hi');
    expect(socket.sent).toEqual([
      { type: 'start' },
      { type: 'bid', moves: 4 },
      { type: 'move', robot: 'blue', dir: 'left' },
      { type: 'concede' },
      { type: 'chat', text: 'hi' },
    ]);
  });

  it('refuses an impossible bid locally', () => {
    const { game, socket } = connected();
    for (const bad of [0, 2.5, 256, Number.NaN]) game.bid(bad);
    expect(socket.sent).toEqual([]);
    expect(game.error).toBe('errorBidRange');
  });

  it('shows refusals and clears them on the next intent', () => {
    const { game, socket } = connected();
    socket.server({ type: 'error', code: 'not_your_turn' });
    expect(game.error).toBe('errorNotYourTurn');
    game.start();
    expect(game.error).toBeNull();
  });

  it('knows when this player is demonstrating', () => {
    const { game, socket } = connected();
    socket.server(snapshot);
    expect(game.demonstrating).toBe(false);
    socket.server({
      type: 'round_started',
      target: { kind: 'vortex' },
      robots: {
        red: { row: 0, col: 0 },
        blue: { row: 0, col: 1 },
        green: { row: 0, col: 2 },
        yellow: { row: 0, col: 3 },
      },
    });
    socket.server({
      type: 'demonstration_started',
      player: ANA,
      bid: 2,
      until: 1,
      robots: {
        red: { row: 0, col: 0 },
        blue: { row: 0, col: 1 },
        green: { row: 0, col: 2 },
        yellow: { row: 0, col: 3 },
      },
    });
    expect(game.demonstrating).toBe(true);
  });

  it('notices the socket closing and can reconnect', () => {
    const sockets: FakeSocket[] = [];
    const game = new GameModel('ws://x', () => {
      const socket = new FakeSocket();
      sockets.push(socket);
      return socket;
    });
    game.connect();
    sockets[0]?.onopen?.(new Event('open'));
    sockets[0]?.onclose?.(new Event('close') as CloseEvent);
    expect(game.status).toBe('closed');
    // Nothing is sent on a closed socket.
    game.start();
    expect(sockets[0]?.sent).toEqual([]);
    game.connect();
    expect(game.status).toBe('connecting');
    sockets[1]?.onopen?.(new Event('open'));
    expect(game.status).toBe('open');
  });

  it('reconnects by itself after falling behind, and ignores garbage', () => {
    vi.useFakeTimers();
    const sockets: FakeSocket[] = [];
    const game = new GameModel('ws://x', () => {
      const socket = new FakeSocket();
      sockets.push(socket);
      return socket;
    });
    game.connect();
    sockets[0]?.onopen?.(new Event('open'));
    sockets[0]?.onmessage?.(new MessageEvent('message', { data: 'not json' }));
    expect(game.error).toBeNull();
    sockets[0]?.server({ type: 'error', code: 'lagged' });
    sockets[0]?.onclose?.(new Event('close') as CloseEvent);
    expect(sockets).toHaveLength(1);
    vi.advanceTimersByTime(REJOIN_DELAY_MS);
    expect(sockets).toHaveLength(2);
    expect(game.status).toBe('connecting');
    sockets[1]?.onclose?.(new Event('close') as CloseEvent);
    vi.advanceTimersByTime(REJOIN_DELAY_MS);
    expect(sockets).toHaveLength(2);
    expect(game.status).toBe('closed');
    vi.useRealTimers();
  });

  it('keeps a chat message that could not be sent', () => {
    const { game, socket } = connected();
    socket.onclose?.(new Event('close') as CloseEvent);
    expect(game.chat('hi')).toBe(false);
    expect(socket.sent).toEqual([]);
  });

  it('leaves the room and closes for good', () => {
    const { game, socket } = connected();
    game.leave();
    expect(socket.sent).toEqual([{ type: 'leave' }]);
    expect(socket.closed).toBe(true);
    expect(game.status).toBe('closed');
    socket.onclose?.(new Event('close') as CloseEvent);
    expect(game.status).toBe('closed');
  });
});
