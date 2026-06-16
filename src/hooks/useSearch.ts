import { useEffect, useState, useRef } from "preact/hooks";
import { type EntrySearchResult, searchEntries } from "../lib/tauri";

export function useSearch() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<EntrySearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [version, setVersion] = useState(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }

    const trimmed = query.trim();
    if (!trimmed) {
      setResults([]);
      setLoading(false);
      setError(null);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setError(null);

    timerRef.current = setTimeout(async () => {
      try {
        const data = await searchEntries(trimmed);
        if (!cancelled) {
          setResults(data);
          setLoading(false);
        }
      } catch {
        if (!cancelled) {
          setResults([]);
          setError("Search failed. Please try again.");
          setLoading(false);
        }
      }
    }, 200);

    return () => {
      cancelled = true;
    };
  }, [query, version]);

  const refresh = () => setVersion((v) => v + 1);

  return { query, setQuery, results, loading, error, refresh };
}
