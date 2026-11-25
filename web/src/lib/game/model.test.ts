import { describe, expect, it } from 'vitest';
import type { ServerMessage } from '../../protocol/ServerMessage';
import { GameModel, socketUrl, type Socket } from './model.svelte';

const ANA = 'a'.repeat(32);

class FakeSocket implements Socket {
  sent: unknown[] = [];
  closed = false;
  onopen: (() => void) | null = null;
  onmessage: ((event: { data: unknown }) => void) | null = null;
  onclose: (() => void) | null = null;

  send(data: string): void {
    this.sent.push(JSON.parse(data));
  }
  close(): void {
    this.closed = true;
  }
  server(message: ServerMessage): void {
    this.onmessage?.({ data: JSON.stringify(message) });
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
  socket.onopen?.();
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
    expect(game.error).toBe('errorBidZero');
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
    socket.server({ type: 'demonstration_started', player: ANA, bid: 2, until: 1 });
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
    sockets[0]?.onopen?.();
    sockets[0]?.onclose?.();
    expect(game.status).toBe('closed');
    // Nothing is sent on a closed socket.
    game.start();
    expect(sockets[0]?.sent).toEqual([]);
    game.connect();
    expect(game.status).toBe('connecting');
    sockets[1]?.onopen?.();
    expect(game.status).toBe('open');
  });

  it('leaves the room and closes for good', () => {
    const { game, socket } = connected();
    game.leave();
    expect(socket.sent).toEqual([{ type: 'leave' }]);
    expect(socket.closed).toBe(true);
    expect(game.status).toBe('closed');
    socket.onclose?.();
    expect(game.status).toBe('closed');
  });
});
