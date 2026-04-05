import { useEffect, useRef } from "react";
import { Chessground } from "@lichess-org/chessground";
import type { Api } from "@lichess-org/chessground/api";
import type { Config } from "@lichess-org/chessground/config";

/**
 * React hook that manages a Chessground instance lifecycle.
 *
 * Returns a ref to attach to a container div and the chessground API.
 * The instance is created once when the container mounts and destroyed on unmount.
 * Config updates are applied via `api.set()` without recreating the instance.
 */
export function useChessground(config: Partial<Config>): {
  containerRef: React.RefObject<HTMLDivElement | null>;
  api: Api | null;
} {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const apiRef = useRef<Api | null>(null);

  // Create the Chessground instance once on mount.
  useEffect(() => {
    if (!containerRef.current) return;

    const api = Chessground(containerRef.current, config);
    apiRef.current = api;

    return () => {
      api.destroy();
      apiRef.current = null;
    };
    // Only run on mount/unmount — config updates go through api.set() below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Apply config changes without recreating the instance.
  useEffect(() => {
    if (apiRef.current) {
      apiRef.current.set(config);
    }
  }, [config]);

  // Note: `apiRef.current` is null on first render (before mount effect runs)
  // and ref mutations don't trigger re-render. If callers need the API object,
  // this should be changed to useState. Currently no callers use the api return.
  return { containerRef, api: apiRef.current };
}
