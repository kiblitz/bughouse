import type { ClientMessage, ServerMessage } from "../../protocol/messages";
import { WS_URL } from "../env";

type MessageHandler = (msg: ServerMessage) => void;

/**
 * WebSocket client with auto-reconnect and typed message handling.
 *
 * Usage:
 *   const ws = new WsClient();
 *   ws.on("game_start", (msg) => { ... });
 *   ws.connect();
 */
export class WsClient {
  private ws: WebSocket | null = null;
  private handlers = new Map<string, MessageHandler[]>();
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private shouldReconnect = true;

  private static readonly MAX_RECONNECT_ATTEMPTS = 5;
  private static readonly PING_INTERVAL_MS = 30_000;
  private static readonly RECONNECT_BASE_DELAY_MS = 1_000;
  private static readonly MAX_RECONNECT_DELAY_MS = 30_000;

  connect(): void {
    this.shouldReconnect = true;
    this.reconnectAttempts = 0;
    this.createConnection();
  }

  disconnect(): void {
    this.shouldReconnect = false;
    this.cleanup();
  }

  send(msg: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }

  /** Register a handler for a specific message type. */
  on<T extends ServerMessage["type"]>(
    type: T,
    handler: (msg: Extract<ServerMessage, { type: T }>) => void
  ): () => void {
    const handlers = this.handlers.get(type) ?? [];
    handlers.push(handler as MessageHandler);
    this.handlers.set(type, handlers);

    // Return unsubscribe function.
    return () => {
      const current = this.handlers.get(type);
      if (current) {
        this.handlers.set(
          type,
          current.filter((h) => h !== handler)
        );
      }
    };
  }

  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private createConnection(): void {
    this.cleanup();

    const ws = new WebSocket(WS_URL);

    ws.onopen = () => {
      console.log("[ws] connected");
      this.reconnectAttempts = 0;
      this.startPing();
    };

    ws.onmessage = (event) => {
      try {
        const msg: ServerMessage = JSON.parse(event.data);
        const handlers = this.handlers.get(msg.type);
        if (handlers) {
          for (const handler of handlers) {
            handler(msg);
          }
        }
      } catch (e) {
        console.error("[ws] failed to parse message:", e);
      }
    };

    ws.onclose = () => {
      console.log("[ws] disconnected");
      this.stopPing();
      if (this.shouldReconnect) {
        this.scheduleReconnect();
      }
    };

    ws.onerror = (e) => {
      console.error("[ws] error:", e);
    };

    this.ws = ws;
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= WsClient.MAX_RECONNECT_ATTEMPTS) {
      console.log("[ws] max reconnect attempts reached");
      return;
    }

    const delay = Math.min(
      WsClient.RECONNECT_BASE_DELAY_MS * 2 ** this.reconnectAttempts,
      WsClient.MAX_RECONNECT_DELAY_MS
    );
    console.log(`[ws] reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1})`);

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      this.createConnection();
    }, delay);
  }

  private startPing(): void {
    this.pingTimer = setInterval(() => {
      this.send({ type: "ping" });
    }, WsClient.PING_INTERVAL_MS);
  }

  private stopPing(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  private cleanup(): void {
    this.stopPing();
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.onclose = null;
      this.ws.close();
      this.ws = null;
    }
  }
}

/** Singleton WS client instance. */
export const wsClient = new WsClient();
