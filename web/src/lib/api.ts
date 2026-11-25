import type { ErrorBody } from '../protocol/ErrorBody';
import type { RoomCreated } from '../protocol/RoomCreated';
import type { RoomSummary } from '../protocol/RoomSummary';
import type { Session } from '../protocol/Session';
import type { StartGuest } from '../protocol/StartGuest';
import type { Started } from '../protocol/Started';

/** A refused or failed request, by the server's stable code (`network` when unreachable). */
export class ApiError extends Error {
  constructor(
    readonly code: string,
    readonly status: number,
  ) {
    super(code);
    this.name = 'ApiError';
  }
}

export type Fetch = (input: string, init?: RequestInit) => Promise<Response>;

/**
 * The server's HTTP API. It keeps the session's CSRF token and sends it on
 * every state-changing request; the session cookie travels on its own.
 */
export class Api {
  private csrf = '';

  constructor(private readonly fetch: Fetch) {}

  async session(): Promise<Session> {
    const session = await this.request<Session>('GET', '/api/session');
    this.csrf = session.csrf;
    return session;
  }

  async startGuest(nickname: string): Promise<Started> {
    const body: StartGuest = { nickname };
    const started = await this.post<Started>('/api/guest', body);
    this.csrf = started.csrf;
    return started;
  }

  rooms(): Promise<RoomSummary[]> {
    return this.request<RoomSummary[]>('GET', '/api/rooms');
  }

  openRoom(): Promise<RoomCreated> {
    return this.post<RoomCreated>('/api/rooms');
  }

  /**
   * A state-changing request. The token belongs to one session, so when
   * there is none yet (the first load failed) or the server says it no
   * longer matches (the session expired and a new one began), a fresh
   * token is fetched and the request sent once more.
   */
  private async post<T>(path: string, body?: unknown): Promise<T> {
    if (this.csrf === '') await this.session();
    try {
      return await this.request<T>('POST', path, body);
    } catch (e) {
      if (!(e instanceof ApiError) || e.code !== 'csrf_mismatch') throw e;
      await this.session();
      return this.request<T>('POST', path, body);
    }
  }

  private async request<T>(method: 'GET' | 'POST', path: string, body?: unknown): Promise<T> {
    const headers: Record<string, string> = {};
    if (method === 'POST') headers['x-csrf-token'] = this.csrf;
    if (body !== undefined) headers['content-type'] = 'application/json';
    let response: Response;
    try {
      response = await this.fetch(path, {
        method,
        headers,
        credentials: 'same-origin',
        body: body === undefined ? undefined : JSON.stringify(body),
      });
    } catch {
      throw new ApiError('network', 0);
    }
    if (!response.ok) throw new ApiError(await errorCode(response), response.status);
    return (await response.json()) as T;
  }
}

async function errorCode(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as Partial<ErrorBody>;
    return typeof body.error === 'string' ? body.error : 'unknown';
  } catch {
    return 'unknown';
  }
}
