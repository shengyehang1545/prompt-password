import type { EntrySearchResult } from "../lib/tauri";
import { EntryItem } from "./EntryItem";

interface EntryListProps {
  entries: EntrySearchResult[];
  highlightedIndex: number;
  enterCopyIndex: number;
  onCopyHandled: () => void;
}

export function EntryList({ entries, highlightedIndex, enterCopyIndex, onCopyHandled }: EntryListProps) {
  if (entries.length === 0) {
    return (
      <div class="entry-list__empty">
        <span class="entry-list__empty-text">No entries found</span>
      </div>
    );
  }

  return (
    <div class="entry-list">
      {entries.map((entry, index) => (
        <EntryItem
          key={entry.id}
          entry={entry}
          isHighlighted={index === highlightedIndex}
          shouldCopy={index === enterCopyIndex}
          onCopied={onCopyHandled}
        />
      ))}
    </div>
  );
}
