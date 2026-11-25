import { describe, expect, it } from 'vitest';
import type { Robots } from '../../protocol/Robots';
import type { ServerMessage } from '../../protocol/ServerMessage';
import { apply, CHAT_LINES, nameOf, standings, type GameState } from './state';

const ANA = 'a'.repeat(32);
const BOB = 'b'.repeat(32);
const at = (row: number, col: number) => ({ row, col });
const robots: Robots = { red: at(1, 1), blue: at(2, 2), green: at(3, 3), yellow: at(4, 4) };
const moved: Robots = { ...robots, red: at(0, 1) };
const board = { size: 16, walls: [], blocked: [], targets: [] };
const target = { kind: 'vortex' } as const;

const snapshot: ServerMessage = {
  type: 'snapshot',
  you: ANA,
  room: { members: [{ id: ANA, name: 'Ana1', is_guest: true }], board, game: null },
};

function play(...messages: ServerMessage[]): GameState {
  let state: GameState | null = null;
  for (const message of messages) state = apply(state, message);
  if (!state) throw new Error('no snapshot');
  return state;
}

const bob = { type: 'joined', player: { id: BOB, name: 'Bob1', is_guest: true } } as const;
const started = { type: 'round_started', target, robots } as const;

describe('apply', () => {
  it('ignores events before the first snapshot', () => {
    expect(apply(null, bob)).toBeNull();
  });

  it('tracks members idempotently', () => {
    const state = play(snapshot, bob, bob);
    expect(state.members.map((m) => m.name)).toEqual(['Ana1', 'Bob1']);
    expect(play(snapshot, bob, { type: 'left', player: BOB }).members).toHaveLength(1);
  });

  it('plays a round from bids to a solve', () => {
    const state = play(
      snapshot,
      bob,
      started,
      { type: 'bid_placed', player: ANA, moves: 5, closes_at: 61_000 },
      { type: 'bid_placed', player: BOB, moves: 3, closes_at: 61_000 },
      { type: 'bid_placed', player: ANA, moves: 2, closes_at: 61_000 },
      { type: 'demonstration_started', player: ANA, bid: 2, until: 121_000, robots },
      { type: 'robot_moved', robot: 'red', robots: moved },
      { type: 'robot_moved', robot: 'red', robots: moved },
    );
    expect(state.round?.bids).toEqual([
      { player: ANA, moves: 2 },
      { player: BOB, moves: 3 },
    ]);
    // The repeated move (as after a snapshot) counts once.
    expect(state.round?.phase).toEqual({ kind: 'demonstrating', player: ANA, bid: 2, moves_made: 1 });
    expect(state.round?.deadline).toBe(121_000);
    expect(state.round?.robots).toEqual(moved);

    const solved = { type: 'round_resolved', phase: { kind: 'solved', winner: ANA, moves: 1 }, robots: moved } as const;
    const after = play(snapshot, bob, started, solved, solved);
    expect(after.round?.scores).toEqual([{ player: ANA, tokens: 1 }]);
    expect(after.round?.deadline).toBeNull();
  });

  it('keeps scores into the next round and takes the final ones at the end', () => {
    const solved = { type: 'round_resolved', phase: { kind: 'solved', winner: BOB, moves: 1 }, robots } as const;
    const state = play(snapshot, bob, started, solved, started);
    expect(state.round?.scores).toEqual([{ player: BOB, tokens: 1 }]);
    expect(state.round?.phase).toEqual({ kind: 'bidding' });
    const over = apply(state, {
      type: 'game_over',
      leaders: [BOB],
      scores: [{ player: BOB, tokens: 2 }],
    });
    expect(over?.round?.over).toBe(true);
    expect(over?.round?.scores).toEqual([{ player: BOB, tokens: 2 }]);
  });

  it('starts from a snapshot of a running game and keeps chat across snapshots', () => {
    const running: ServerMessage = {
      type: 'snapshot',
      you: BOB,
      room: {
        members: [],
        board,
        game: {
          target,
          robots,
          phase: { kind: 'bidding' },
          bids: [],
          deadline: null,
          scores: [],
          targets_left: 3,
          over: false,
        },
      },
    };
    const state = play(snapshot, { type: 'chat', player: ANA, text: 'hi' }, running);
    expect(state.you).toBe(BOB);
    expect(state.round?.target).toEqual(target);
    expect(state.chat).toEqual([{ player: ANA, text: 'hi' }]);
  });

  it('keeps the last chat lines and ignores errors', () => {
    let state = play(snapshot);
    for (let i = 0; i <= CHAT_LINES; i++) {
      state = apply(state, { type: 'chat', player: ANA, text: `line ${String(i)}` }) ?? state;
    }
    expect(state.chat).toHaveLength(CHAT_LINES);
    expect(state.chat[0]?.text).toBe('line 1');
    expect(apply(state, { type: 'error', code: 'rate_limited' })).toBe(state);
  });
});

describe('robot positions', () => {
  it('come from the server when a demonstration starts or a round ends', () => {
    const state = play(
      snapshot,
      started,
      { type: 'demonstration_started', player: ANA, bid: 2, until: 1, robots },
      { type: 'robot_moved', robot: 'red', robots: moved },
      { type: 'demonstration_started', player: BOB, bid: 3, until: 2, robots },
    );
    expect(state.round?.robots).toEqual(robots);
    const ended = apply(state, { type: 'round_resolved', phase: { kind: 'unsolved' }, robots: moved });
    expect(ended?.round?.robots).toEqual(moved);
  });

  it('keep equal bids in the order made', () => {
    const state = play(
      snapshot,
      started,
      { type: 'bid_placed', player: BOB, moves: 4, closes_at: 1 },
      { type: 'bid_placed', player: ANA, moves: 4, closes_at: 1 },
    );
    expect(state.round?.bids.map((b) => b.player)).toEqual([BOB, ANA]);
  });
});

describe('helpers', () => {
  it('names players and ranks them, members without tokens included', () => {
    const solved = { type: 'round_resolved', phase: { kind: 'solved', winner: BOB, moves: 1 }, robots } as const;
    const state = play(snapshot, bob, started, solved);
    expect(nameOf(state, BOB)).toBe('Bob1');
    expect(nameOf(state, 'c'.repeat(32))).toBe('cccccc');
    expect(standings(state)).toEqual([
      { player: BOB, tokens: 1 },
      { player: ANA, tokens: 0 },
    ]);
  });
});
