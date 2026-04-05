import { useMemo } from "react";
import type { Config } from "@lichess-org/chessground/config";
import type { Color, Key } from "@lichess-org/chessground/types";
import { useChessground } from "../../hooks/useChessground";
import "@lichess-org/chessground/assets/chessground.base.css";
import "@lichess-org/chessground/assets/chessground.brown.css";
import "@lichess-org/chessground/assets/chessground.cburnett.css";
import "../../styles/board.css";

const ANIMATION_DURATION_MS = 200;

interface ChessBoardProps {
  /** FEN string for the current position. */
  fen: string;
  /** Which color is at the bottom of the board. */
  orientation: Color;
  /** Whether the player can make moves. */
  interactive: boolean;
  /** Last move to highlight, e.g. ["e2", "e4"]. */
  lastMove?: [string, string] | null;
  /** Called when the player makes a move. */
  onMove?: (from: string, to: string) => void;
}

export function ChessBoard({
  fen,
  orientation,
  interactive,
  lastMove,
  onMove,
}: ChessBoardProps) {
  const config: Partial<Config> = useMemo(
    () => ({
      fen,
      orientation,
      turnColor: fenToTurnColor(fen),
      lastMove: lastMove ? (lastMove as [Key, Key]) : undefined,
      movable: {
        // Server validates all moves; allow free dragging for Phase 1.
        // Phase 2+ will compute and pass `dests` for legal move hints.
        free: interactive,
        color: interactive ? orientation : undefined,
        events: {
          after: (orig: Key, dest: Key) => {
            onMove?.(orig, dest);
          },
        },
      },
      draggable: {
        enabled: interactive,
      },
      selectable: {
        enabled: interactive,
      },
      animation: {
        enabled: true,
        duration: ANIMATION_DURATION_MS,
      },
    }),
    [fen, orientation, interactive, lastMove, onMove]
  );

  const { containerRef } = useChessground(config);

  return <div ref={containerRef} className="chess-board" />;
}

/** Extract the turn color from a FEN string. */
function fenToTurnColor(fen: string): Color {
  const parts = fen.split(" ");
  return parts[1] === "b" ? "black" : "white";
}
