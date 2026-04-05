# Bughouse Chess

A real-time bughouse chess platform. Bughouse is a 2v2 chess variant where captured pieces transfer to your partner's reserve for dropping onto their board.

## Architecture

- **Backend** — Rust/Axum server with WebSocket support, using [shakmaty](https://crates.io/crates/shakmaty) for game logic
- **Frontend** — TypeScript/React with [Chessground](https://github.com/lichess-org/chessground) for board rendering
- **Protocol** — Shared WebSocket message types in `protocol/messages.ts`

## Development

### Backend

```bash
cd backend
cargo run          # Starts server on :3000
cargo test         # Run tests
cargo fmt          # Format code
```

### Frontend

```bash
cd frontend
npm install        # Install dependencies
npm run dev        # Start dev server with HMR
npm run build      # Production build
```

### Playing locally

1. Start the backend: `cd backend && cargo run`
2. Start the frontend: `cd frontend && npm run dev`
3. Open two browser tabs to the frontend URL
4. First tab plays as white, second as black

## Deployment

- **Frontend** — Automatically deployed to GitHub Pages on push to `main`
- **Backend** — Build with `cargo build --release` and deploy to your VPS behind nginx

Set `VITE_API_URL` and `VITE_WS_URL` as GitHub Actions variables for production.

## License

See [LICENSE](LICENSE).
