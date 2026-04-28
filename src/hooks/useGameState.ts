import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { GameState } from "../lib/types";

export function useGameState() {
  const [gameState, setGameState] = useState<GameState | null>(null);
  useEffect(() => {
    const unlisten = listen<GameState>("game-state", (event) => {
      setGameState(event.payload);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);
  return gameState;
}
