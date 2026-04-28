import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { BuildOrder, StepChangedPayload } from "../lib/types";

export function useBuildOrder() {
  const [buildOrder, setBuildOrder] = useState<BuildOrder | null>(null);
  const [currentStep, setCurrentStep] = useState(0);
  const [totalSteps, setTotalSteps] = useState(0);

  useEffect(() => {
    const unlisten = listen<StepChangedPayload>("step-changed", (event) => {
      setCurrentStep(event.payload.index);
      setTotalSteps(event.payload.total);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const loadBuildOrder = useCallback(async (path: string) => {
    try {
      const bo = await invoke<BuildOrder>("load_build_order_cmd", { path });
      setBuildOrder(bo);
      setCurrentStep(0);
      setTotalSteps(bo.steps.length);
    } catch (e) {
      console.error("Failed to load build order:", e);
    }
  }, []);

  const advance = useCallback(async () => {
    try { await invoke("advance_step"); } catch (e) { console.error("Failed to advance step:", e); }
  }, []);

  const previous = useCallback(async () => {
    try { await invoke("previous_step"); } catch (e) { console.error("Failed to go to previous step:", e); }
  }, []);

  const reset = useCallback(async () => {
    try { await invoke("reset_steps"); } catch (e) { console.error("Failed to reset steps:", e); }
  }, []);

  return { buildOrder, currentStep, totalSteps, loadBuildOrder, advance, previous, reset };
}
