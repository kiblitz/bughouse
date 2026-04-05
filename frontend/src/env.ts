/** Backend connection URLs, configured via Vite env vars. */

const DEFAULT_API_URL = "http://localhost:3000";
const DEFAULT_WS_URL = "ws://localhost:3000/ws";

export const API_BASE_URL =
  import.meta.env.VITE_API_URL ?? DEFAULT_API_URL;

export const WS_URL =
  import.meta.env.VITE_WS_URL ?? DEFAULT_WS_URL;
