import type { BidView } from '../../protocol/BidView';
import type { Board } from '../../protocol/Board';
import type { Phase } from '../../protocol/Phase';
import type { Player } from '../../protocol/Player';
import type { Robots } from '../../protocol/Robots';
import type { Score } from '../../protocol/Score';
import type { ServerMessage } from '../../protocol/ServerMessage';
import type { Target } from '../../protocol/Target';

export interface Round {
  target: Target;
  robots: Robots;
  phase: Phase;
  /** Best first. */
  bids: BidView[];
  /** When the current stage ends on its own (server epoch ms), if known. */
  deadline: number | null;
  scores: Score[];
  over: boolean;
}

export interface ChatLine {
  player: string;
  text: string;
}

/** What the game page draws: the room as the server last described it. */
export interface GameState {
  you: string;
  members: Player[];
  board: Board;
  round: Round | null;
  chat: ChatLine[];
}

/** Chat lines kept on screen; older ones scroll away for good. */
export const CHAT_LINES = 100;

/**
 * Applies one server message. Pure, and idempotent where the server may
 * repeat itself: an event can also be reflected in the snapshot taken just
 * before it, so applying it again must not change anything.
 */
export function apply(state: GameState | null, message: ServerMessage): GameState | null {
  if (message.type === 'snapshot') {
    const { room } = message;
    return {
      you: message.you,
      members: room.members,
      board: room.board,
      round: room.game && {
        target: room.game.target,
        robots: room.game.robots,
        phase: room.game.phase,
        bids: room.game.bids,
        deadline: room.game.deadline,
        scores: room.game.scores,
        over: room.game.over,
      },
      chat: state?.chat ?? [],
    };
  }
  if (!state) return null;
  const round = state.round;
  switch (message.type) {
    case 'joined':
      if (state.members.some((m) => m.id === message.player.id)) return state;
      return { ...state, members: [...state.members, message.player] };
    case 'left':
      return { ...state, members: state.members.filter((m) => m.id !== message.player) };
    case 'round_started':
      return {
        ...state,
        round: {
          target: message.target,
          robots: message.robots,
          phase: { kind: 'bidding' },
          bids: [],
          deadline: null,
          scores: round?.scores ?? [],
          over: false,
        },
      };
    case 'bid_placed':
      if (!round) return state;
      return {
        ...state,
        round: {
          ...round,
          bids: withBid(round.bids, { player: message.player, moves: message.moves }),
          deadline: message.closes_at,
        },
      };
    case 'demonstration_started':
      if (!round) return state;
      return {
        ...state,
        round: {
          ...round,
          robots: message.robots,
          phase: {
            kind: 'demonstrating',
            player: message.player,
            bid: message.bid,
            moves_made: 0,
          },
          deadline: message.until,
        },
      };
    case 'robot_moved': {
      // A real move always changes the robots (the server refuses others),
      // so the same positions mean this move is already counted.
      if (!round || sameRobots(round.robots, message.robots)) return state;
      const phase =
        round.phase.kind === 'demonstrating'
          ? { ...round.phase, moves_made: round.phase.moves_made + 1 }
          : round.phase;
      return { ...state, round: { ...round, robots: message.robots, phase } };
    }
    case 'round_resolved':
      if (!round || samePhase(round.phase, message.phase)) return state;
      return {
        ...state,
        round: {
          ...round,
          phase: message.phase,
          robots: message.robots,
          deadline: null,
          scores:
            message.phase.kind === 'solved'
              ? addToken(round.scores, message.phase.winner)
              : round.scores,
        },
      };
    case 'game_over':
      if (!round) return state;
      return { ...state, round: { ...round, scores: message.scores, deadline: null, over: true } };
    case 'chat':
      return {
        ...state,
        chat: [...state.chat, { player: message.player, text: message.text }].slice(-CHAT_LINES),
      };
    case 'error':
      return state;
  }
}

/** The player's new bid replaces their old one; fewest moves first, ties in order made. */
function withBid(bids: BidView[], bid: BidView): BidView[] {
  return [...bids.filter((b) => b.player !== bid.player), bid].sort((a, b) => a.moves - b.moves);
}

function addToken(scores: Score[], player: string): Score[] {
  if (!scores.some((s) => s.player === player)) return [...scores, { player, tokens: 1 }];
  return scores.map((s) => (s.player === player ? { ...s, tokens: s.tokens + 1 } : s));
}

function sameRobots(a: Robots, b: Robots): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

function samePhase(a: Phase, b: Phase): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** A member's display name, or a short form of an unknown ID. */
export function nameOf(state: GameState, player: string): string {
  return state.members.find((m) => m.id === player)?.name ?? player.slice(0, 6);
}

/** Tokens per player, highest first, including members who have none. */
export function standings(state: GameState): Score[] {
  const scores = state.round?.scores ?? [];
  const tokens = (player: string) => scores.find((s) => s.player === player)?.tokens ?? 0;
  const players = [...new Set([...state.members.map((m) => m.id), ...scores.map((s) => s.player)])];
  return players
    .map((player) => ({ player, tokens: tokens(player) }))
    .sort((a, b) => b.tokens - a.tokens);
}
