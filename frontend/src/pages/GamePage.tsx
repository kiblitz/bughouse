import { useEffect, useCallback } from "react";
import type { Color } from "@lichess-org/chessground/types";
import { ChessBoard } from "../components/board/ChessBoard";
import { useGameStore } from "../store/game";
import "../styles/board.css";

export function GamePage() {
  const {
    fen,
    myColor,
    turnColor,
    connected,
    lastMove,
    error,
    gameOver,
    resultText,
    connect,
    disconnect,
    sendMove,
  } = useGameStore();

  useEffect(() => {
    connect();
    return () => disconnect();
  }, [connect, disconnect]);

  const handleMove = useCallback(
    (from: string, to: string) => {
      sendMove(from, to);
    },
    [sendMove]
  );

  const orientation: Color = myColor === "black" ? "black" : "white";
  const isMyTurn = myColor === turnColor;

  return (
    <div className="game-page">
      <h1>Bughouse Chess</h1>

      <div className="game-info">
        {myColor ? (
          <span className="color-badge">
            Playing as {myColor}
          </span>
        ) : (
          <span>Waiting for game...</span>
        )}
        <span className="turn-indicator">
          {gameOver
            ? "Game over"
            : isMyTurn
            ? "Your turn"
            : "Opponent's turn"}
        </span>
        {error && <span className="error-message">{error}</span>}
      </div>

      <div className="board-container">
        <ChessBoard
          fen={fen}
          orientation={orientation}
          interactive={!gameOver && isMyTurn}
          lastMove={lastMove}
          onMove={handleMove}
        />
        {gameOver && resultText && (
          <div className="game-over-banner">{resultText}</div>
        )}
      </div>

      <div className="status-bar">
        <span>{connected ? "Connected" : "Disconnected"}</span>
        <span>Turn: {turnColor}</span>
      </div>
    </div>
  );
}
