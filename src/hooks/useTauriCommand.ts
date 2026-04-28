import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useTauriCommand<T>(command: string) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = useCallback(
    async (args?: Record<string, unknown>): Promise<T | null> => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<T>(command, args);
        return result;
      } catch (e) {
        const message = typeof e === "string" ? e : String(e);
        setError(message);
        return null;
      } finally {
        setLoading(false);
      }
    },
    [command]
  );

  return { execute, loading, error };
}
