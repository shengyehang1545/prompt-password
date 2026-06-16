import { useState, useCallback } from "preact/hooks";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSearch } from "./hooks/useSearch";
import { SearchInput } from "./components/SearchInput";
import { EntryList } from "./components/EntryList";
import { AddEntryForm } from "./components/AddEntryForm";
import "./App.css";
import "./components/SearchInput.css";
import "./components/EntryItem.css";
import "./components/EntryList.css";
import "./components/AddEntryForm.css";

export function App() {
  const { query, setQuery, results, loading, error, refresh } = useSearch();
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const [enterCopyIndex, setEnterCopyIndex] = useState(-1);
  const [showForm, setShowForm] = useState(false);

  const handleInput = useCallback((value: string) => {
    setQuery(value);
    setHighlightedIndex(-1);
  }, []);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (results.length === 0) {
        if (e.key === "Escape") {
          e.preventDefault();
          getCurrentWindow().hide();
        }
        return;
      }

      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setHighlightedIndex((prev) =>
            prev < results.length - 1 ? prev + 1 : 0
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setHighlightedIndex((prev) =>
            prev > 0 ? prev - 1 : results.length - 1
          );
          break;
        case "Enter":
          e.preventDefault();
          if (highlightedIndex >= 0 && highlightedIndex < results.length) {
            setEnterCopyIndex(highlightedIndex);
          }
          break;
        case "Escape":
          e.preventDefault();
          getCurrentWindow().hide();
          break;
      }
    },
    [results, highlightedIndex]
  );

  const handleAddSuccess = useCallback(() => {
    setShowForm(false);
    refresh();
  }, [refresh]);

  return (
    <div class="app">
      <SearchInput
        value={query}
        onInput={handleInput}
        onKeyDown={handleKeyDown}
        loading={loading}
      >
        <button
          class="add-entry-btn"
          onClick={() => setShowForm(true)}
          type="button"
          aria-label="Add entry"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 3.5v9M3.5 8h9" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
        </button>
      </SearchInput>
      {error ? (
        <div class="search-error">{error}</div>
      ) : (
        <EntryList
          entries={results}
          highlightedIndex={highlightedIndex}
          enterCopyIndex={enterCopyIndex}
          onCopyHandled={() => setEnterCopyIndex(-1)}
        />
      )}
      {showForm && (
        <AddEntryForm
          onSuccess={handleAddSuccess}
          onClose={() => setShowForm(false)}
        />
      )}
    </div>
  );
}
