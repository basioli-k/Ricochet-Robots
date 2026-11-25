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
  onopen: ((event: Event) => void) | null;
  onmessage: ((event: MessageEvent) => void) | null;
  onclose: ((event: CloseEvent) => void) | null;
}

export type SocketFactory = (url: string) => Socket;

/** Pause before reconnecting a socket the server closed for falling behind. */
export const REJOIN_DELAY_MS = 1_000;

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
  /** Reconnect when the current socket closes (it fell behind). */
  private rejoin = false;
  /** The pending reconnect; leaving or connecting again cancels it. */
  private rejoinTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(
    private readonly url: string,
    private readonly makeSocket: SocketFactory,
  ) {}

  connect(): void {
    this.cancelRejoin();
    this.socket?.close();
    this.status = 'connecting';
    const socket = this.makeSocket(this.url);
    socket.onopen = () => {
      this.status = 'open';
      this.error = null;
      this.rejoin = false;
    };
    socket.onmessage = (event) => {
      if (typeof event.data !== 'string') return;
      let message: ServerMessage;
      try {
        message = JSON.parse(event.data) as ServerMessage;
      } catch {
        return; // Not ours to understand; the server only sends JSON.
      }
      this.receive(message);
    };
    socket.onclose = () => {
      if (this.socket !== socket) return;
      this.socket = null;
      this.status = 'closed';
      // The server closes a socket that fell behind; catch up after a
      // pause, so a client that keeps falling behind doesn't spin.
      if (this.rejoin) {
        this.rejoin = false;
        this.rejoinTimer = setTimeout(() => {
          this.rejoinTimer = null;
          this.connect();
        }, REJOIN_DELAY_MS);
      }
    };
    this.socket = socket;
  }

  receive(message: ServerMessage): void {
    if (message.type === 'error') {
      this.error = errorKey(message.code);
      if (message.code === 'lagged') this.rejoin = true;
    } else {
      this.state = apply(this.state, message);
    }
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
    // A bid must fit the protocol's byte; the server judges everything else.
    if (Number.isInteger(moves) && moves >= 1 && moves <= 255) this.send({ type: 'bid', moves });
    else this.error = 'errorBidRange';
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

  /** Sends a chat message; false if it couldn't go (empty, or no connection). */
  chat(text: string): boolean {
    if (text.trim() === '' || this.status !== 'open') return false;
    this.send({ type: 'chat', text });
    return true;
  }

  /** Leaves the room and closes the socket for good. */
  leave(): void {
    this.send({ type: 'leave' });
    this.rejoin = false;
    this.cancelRejoin();
    const socket = this.socket;
    this.socket = null;
    this.status = 'closed';
    socket?.close();
  }

  private cancelRejoin(): void {
    if (this.rejoinTimer !== null) clearTimeout(this.rejoinTimer);
    this.rejoinTimer = null;
  }

  private send(message: ClientMessage): void {
    this.error = null;
    if (this.status === 'open') this.socket?.send(JSON.stringify(message));
  }
}
