import { create } from "zustand";
import type { Color, Role } from "../../protocol/messages";
import { wsClient } from "../ws/client";

const ERROR_DISPLAY_DURATION_MS = 3000;

interface GameState {
  /** FEN string of the current board position. */
  fen: string;
  /** Which color this player is. */
  myColor: Color | null;
  /** Whose turn it is. */
  turnColor: Color;
  /** Whether we're connected to the server. */
  connected: boolean;
  /** Last move made (for highlighting). */
  lastMove: [string, string] | null;
  /** Error message from server, if any. */
  error: string | null;
  /** Whether the game is over. */
  gameOver: boolean;
  /** Game result description. */
  resultText: string | null;

  // Actions
  connect: () => void;
  disconnect: () => void;
  sendMove: (from: string, to: string, promotion?: string) => void;
}

const INITIAL_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/** Stored unsubscribe functions to prevent handler accumulation. */
let unsubscribers: (() => void)[] = [];

export const useGameStore = create<GameState>((set, get) => ({
  fen: INITIAL_FEN,
  myColor: null,
  turnColor: "white",
  connected: false,
  lastMove: null,
  error: null,
  gameOver: false,
  resultText: null,

  connect: () => {
    // Clean up any existing handlers first (prevents duplicates on re-mount).
    for (const unsub of unsubscribers) {
      unsub();
    }
    unsubscribers = [];

    unsubscribers.push(
      wsClient.on("game_start", (msg) => {
        set({
          myColor: msg.color,
          fen: msg.fen,
          turnColor: "white",
          connected: true,
          lastMove: null,
          error: null,
          gameOver: false,
          resultText: null,
        });
      })
    );

    unsubscribers.push(
      wsClient.on("move", (msg) => {
        set({
          fen: msg.fen,
          turnColor: msg.color === "white" ? "black" : "white",
          lastMove: [msg.from, msg.to],
        });
      })
    );

    unsubscribers.push(
      wsClient.on("drop", (msg) => {
        set({
          fen: msg.fen,
          turnColor: msg.color === "white" ? "black" : "white",
          lastMove: null,
        });
      })
    );

    unsubscribers.push(
      wsClient.on("game_over", (msg) => {
        const winner = msg.result.winner;
        let resultText: string;
        if (winner === null) {
          resultText = `Draw by ${msg.result.termination}`;
        } else {
          const myColor = get().myColor;
          resultText =
            winner === myColor
              ? `You won! (${msg.result.termination})`
              : `You lost. (${msg.result.termination})`;
        }
        set({ gameOver: true, resultText });
      })
    );

    unsubscribers.push(
      wsClient.on("error", (msg) => {
        set({ error: msg.message });
        setTimeout(() => set({ error: null }), ERROR_DISPLAY_DURATION_MS);
      })
    );

    wsClient.connect();
  },

  disconnect: () => {
    for (const unsub of unsubscribers) {
      unsub();
    }
    unsubscribers = [];
    wsClient.disconnect();
    set({ connected: false });
  },

  sendMove: (from: string, to: string, promotion?: string) => {
    const state = get();
    if (state.gameOver) return;

    wsClient.send({
      type: "move",
      from,
      to,
      promotion: promotion as Role | undefined,
    });
  },
}));
