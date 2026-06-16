import { useState, useCallback, useRef } from "preact/hooks";
import { copyToClipboard } from "../lib/tauri";

type CopyState = "idle" | "copied" | "failed";

export function useClipboard() {
  const [copyState, setCopyState] = useState<CopyState>("idle");
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const copy = useCallback(async (text: string) => {
    if (timerRef.current) clearTimeout(timerRef.current);

    try {
      await copyToClipboard(text);
      setCopyState("copied");
    } catch {
      setCopyState("failed");
    }

    timerRef.current = setTimeout(() => setCopyState("idle"), 1500);
  }, []);

  return { copyState, copy };
}
