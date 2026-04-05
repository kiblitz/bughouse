/**
 * Shared WebSocket message protocol between frontend and backend.
 * This file is the source of truth — the Rust backend mirrors these types via serde.
 */

// ─── Common Types ────────────────────────────────────────────────────────────

export type Color = "white" | "black";
export type BoardId = "a" | "b";
export type Square = string; // e.g. "e4", "d7"
export type Role = "pawn" | "knight" | "bishop" | "rook" | "queen" | "king";

export interface Reserve {
  pawn: number;
  knight: number;
  bishop: number;
  rook: number;
  queen: number;
}

export interface GameResult {
  /** The color that won, or null for a draw. */
  winner: Color | null;
  termination: "checkmate" | "timeout" | "resignation" | "abandoned" | "draw";
}

// ─── Client → Server Messages ────────────────────────────────────────────────

export type ClientMessage =
  | { type: "move"; from: Square; to: Square; promotion?: Role }
  | { type: "drop"; role: Role; to: Square }
  | { type: "ping" };

// ─── Server → Client Messages ────────────────────────────────────────────────

export type ServerMessage =
  | { type: "game_start"; color: Color; fen: string }
  | {
      type: "move";
      color: Color;
      from: Square;
      to: Square;
      promotion?: Role;
      fen: string;
    }
  | { type: "drop"; color: Color; role: Role; to: Square; fen: string }
  | { type: "reserve_update"; white: Reserve; black: Reserve }
  | { type: "game_over"; result: GameResult }
  | { type: "error"; message: string }
  | { type: "pong" };
