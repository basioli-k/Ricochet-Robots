import { ApiError, type Api } from './api';
import { errorKey } from './i18n/errors';
import type { MessageKey } from './i18n/messages';
import { isValidNickname } from './nickname';
import type { Identity } from '../protocol/Identity';
import type { RoomSummary } from '../protocol/RoomSummary';

export type View =
  | { name: 'loading' }
  | { name: 'guest' }
  | { name: 'lobby' }
  | { name: 'room'; id: string };

/** What the pages need from the server; `Api` provides it, tests fake it. */
export type LobbyApi = Pick<Api, 'session' | 'startGuest' | 'rooms' | 'openRoom'>;

/**
 * The app's flow: who is playing, which page shows, the lobby's rooms and
 * the last error (as a message key). Pages render it and call its actions.
 */
export class AppModel {
  view = $state<View>({ name: 'loading' });
  identity = $state<Identity | null>(null);
  rooms = $state<RoomSummary[]>([]);
  error = $state<MessageKey | null>(null);
  busy = $state(false);

  constructor(private readonly api: LobbyApi) {}

  /** Asks the server who this session belongs to. */
  async start(): Promise<void> {
    await this.run(async () => {
      const session = await this.api.session();
      this.identity = session.identity;
      if (session.identity) await this.showLobby();
      else this.view = { name: 'guest' };
    });
    if (this.view.name === 'loading') this.view = { name: 'guest' };
  }

  async play(nickname: string): Promise<void> {
    if (!isValidNickname(nickname)) {
      this.error = 'errorNicknameInvalid';
      return;
    }
    await this.run(async () => {
      const started = await this.api.startGuest(nickname);
      this.identity = started.identity;
      await this.showLobby();
    });
  }

  async refresh(): Promise<void> {
    await this.run(async () => {
      this.rooms = await this.api.rooms();
    });
  }

  /**
   * The lobby's background re-read: it never blocks the player's own
   * actions and stays quiet on failure, except that an expired session
   * still returns to the guest entry.
   */
  async poll(): Promise<void> {
    if (this.busy) return;
    // Not marked busy: an action may start meanwhile, and either may set
    // the rooms last; both are fresh lists.
    try {
      this.rooms = await this.api.rooms();
    } catch (e) {
      if (e instanceof ApiError && e.code === 'not_signed_in') this.signedOut();
    }
  }

  async openRoom(): Promise<void> {
    await this.run(async () => {
      const room = await this.api.openRoom();
      this.enter(room.id);
    });
  }

  enter(id: string): void {
    this.error = null;
    this.view = { name: 'room', id };
  }

  async backToLobby(): Promise<void> {
    await this.run(() => this.showLobby());
  }

  private signedOut(): void {
    this.identity = null;
    this.view = { name: 'guest' };
  }

  private async showLobby(): Promise<void> {
    this.view = { name: 'lobby' };
    this.rooms = await this.api.rooms();
  }

  /** Runs one action at a time, turning failures into a message. */
  private async run(action: () => Promise<void>): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    this.error = null;
    try {
      await action();
    } catch (e) {
      const code = e instanceof ApiError ? e.code : 'unknown';
      this.error = errorKey(code);
      if (code === 'not_signed_in') this.signedOut();
    } finally {
      this.busy = false;
    }
  }
}
