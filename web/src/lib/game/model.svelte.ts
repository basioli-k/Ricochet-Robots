import type { ClientMessage } from '../../protocol/ClientMessage';
import type { Dir } from '../../protocol/Dir';
import type { Robot } from '../../protocol/Robot';
import type { ServerMessage } from '../../protocol/ServerMessage';
import { errorKey } from '../i18n/errors';
import type { MessageKey } from '../i18n/messages';
import { apply, type GameState } from './state';

/** The part of `WebSocket` used here, so tests can pass a fake. */
export interface Socket {
  send(data: string): void;
  close(): void;
  onopen: (() => void) | null;
  onmessage: ((event: { data: unknown }) => void) | null;
  onclose: (() => void) | null;
}

export type SocketFactory = (url: string) => Socket;

/** The game socket's URL for room `id`, on the page's own host. */
export function socketUrl(location: Pick<Location, 'protocol' | 'host'>, id: string): string {
  const scheme = location.protocol === 'https:' ? 'wss' : 'ws';
  return `${scheme}://${location.host}/api/rooms/${encodeURIComponent(id)}/ws`;
}

/**
 * One room's game as this player sees it: the socket, the state the server
 * describes, the robot picked for moving, and the last refusal. Every
 * decision is the server's; this only sends intents and shows answers.
 */
export class GameModel {
  state = $state<GameState | null>(null);
  error = $state<MessageKey | null>(null);
  status = $state<'connecting' | 'open' | 'closed'>('connecting');
  selected = $state<Robot>('red');
  private socket: Socket | null = null;

  constructor(
    private readonly url: string,
    private readonly makeSocket: SocketFactory,
  ) {}

  connect(): void {
    this.socket?.close();
    this.status = 'connecting';
    const socket = this.makeSocket(this.url);
    socket.onopen = () => {
      this.status = 'open';
      this.error = null;
    };
    socket.onmessage = (event) => {
      if (typeof event.data === 'string') this.receive(JSON.parse(event.data) as ServerMessage);
    };
    socket.onclose = () => {
      if (this.socket === socket) {
        this.socket = null;
        this.status = 'closed';
      }
    };
    this.socket = socket;
  }

  receive(message: ServerMessage): void {
    if (message.type === 'error') this.error = errorKey(message.code);
    else this.state = apply(this.state, message);
  }

  /** Whether the server expects this player's moves right now. */
  get demonstrating(): boolean {
    const phase = this.state?.round?.phase;
    return phase?.kind === 'demonstrating' && phase.player === this.state?.you;
  }

  start(): void {
    this.send({ type: 'start' });
  }

  bid(moves: number): void {
    if (Number.isInteger(moves) && moves >= 1 && moves <= 255) this.send({ type: 'bid', moves });
    else this.error = 'errorBidZero';
  }

  select(robot: Robot): void {
    this.selected = robot;
  }

  move(dir: Dir): void {
    this.send({ type: 'move', robot: this.selected, dir });
  }

  concede(): void {
    this.send({ type: 'concede' });
  }

  chat(text: string): void {
    if (text.trim() !== '') this.send({ type: 'chat', text });
  }

  /** Leaves the room and closes the socket for good. */
  leave(): void {
    this.send({ type: 'leave' });
    const socket = this.socket;
    this.socket = null;
    this.status = 'closed';
    socket?.close();
  }

  private send(message: ClientMessage): void {
    this.error = null;
    if (this.status === 'open') this.socket?.send(JSON.stringify(message));
  }
}
