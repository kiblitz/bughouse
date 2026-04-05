# Bughouse Chess Website

## Current State (Phase 1)

Single Crazyhouse board playable over WebSocket between two browser tabs.

### What exists now
- **Backend:** Axum server with single `/ws` WebSocket endpoint and `/health` REST route
- **Game engine:** Single `Board` struct wrapping `shakmaty::Crazyhouse` with bughouse-ready pocket management
- **Frontend:** React + Vite with Chessground board, Zustand state, WS client with auto-reconnect
- **Protocol:** Shared types in `protocol/messages.ts`, mirrored in Rust serde types
- **CI:** GitHub Actions for backend (fmt/clippy/test) and frontend (tsc/build)
- **Deployment:** GitHub Pages deploy workflow for frontend

### What does NOT exist yet
- `BughouseGame` coordinator (two-board orchestration) — Phase 2
- Database (SQLite/sqlx) — Phase 3
- Auth (JWT) — Phase 3
- REST API beyond `/health` — Phase 3
- React Router — Phase 3
- Rooms, lobby, matchmaking — Phase 3
- Chat, spectating — Phase 4
- Piece drops via chessground `dragNewPiece` — Phase 2

## Architecture

**Monorepo** with three top-level directories:
- `backend/` — Rust/Axum server (VPS hosted)
- `frontend/` — TypeScript/React app via Vite (GitHub Pages hosted)
- `protocol/` — Shared WebSocket message type definitions (TS source of truth)

### Backend (Rust)
- **Framework:** Axum with tokio async runtime
- **Real-time:** Single `/ws` WebSocket endpoint, message-type routing
- **Concurrency:** `DashMap` for player senders, bounded channels per client

### Frontend (TypeScript/React)
- **Build:** Vite
- **State management:** Zustand
- **Board rendering:** `@lichess-org/chessground` — used directly via custom `useChessground` hook

### Communication
- WebSocket for real-time game state
- Protocol types defined in `protocol/messages.ts`, mirrored in Rust via serde

## Key Design Decisions

- **Pocket management for bughouse:** `Board.make_move()` intercepts Crazyhouse's automatic pocket addition on capture and restores pre-capture pocket state. This allows the future `BughouseGame` coordinator to route captured pieces to the partner's board. Position is reconstructed via `from_setup` with `ignore_too_much_material` since bughouse pocket counts can exceed normal material limits.
- **Server-side move validation:** Frontend sets `movable.free: true` and sends moves to the server. Server validates via shakmaty and broadcasts the result. Phase 2+ will add client-side legal move hints via `movable.dests`.
- **GitHub Pages SPA:** 404.html redirect trick for client-side routing.

## Development

```bash
# Backend
cd backend && cargo build && cargo test && cargo fmt

# Frontend
cd frontend && npm install && npm run dev && npm run build
```

## Phased Roadmap

1. **Foundation** (current) — Single board over WebSocket (two tabs see moves)
2. **Bughouse game logic** — `BughouseGame` coordinator, two boards, reserves, drops, clocks, game-over
3. **Auth + Lobby** — JWT, SQLite, rooms, matchmaking, REST API, React Router
4. **Chat + Spectating**
5. **Matchmaking + History**
6. **Polish** — Sound, animations, mobile, reconnection
