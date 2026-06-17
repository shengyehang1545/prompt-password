import { useRef, useEffect } from "preact/hooks";
import type { EntrySearchResult } from "../lib/tauri";
import { useClipboard } from "../hooks/useClipboard";

interface EntryItemProps {
  entry: EntrySearchResult;
  isHighlighted?: boolean;
  shouldCopy?: boolean;
  onCopied?: () => void;
  onEdit?: (entry: EntrySearchResult) => void;
  onDelete?: (entry: EntrySearchResult) => void;
}

export function EntryItem({
  entry,
  isHighlighted,
  shouldCopy,
  onCopied,
  onEdit,
  onDelete,
}: EntryItemProps) {
  const { copyState, copy } = useClipboard();
  const ref = useRef<HTMLDivElement>(null);
  const onCopiedRef = useRef(onCopied);
  onCopiedRef.current = onCopied;

  useEffect(() => {
    if (isHighlighted) {
      ref.current?.scrollIntoView({ block: "nearest" });
    }
  }, [isHighlighted]);

  useEffect(() => {
    if (shouldCopy) {
      copy(entry.id);
      onCopiedRef.current?.();
    }
  }, [shouldCopy]);

  return (
    <div
      ref={ref}
      class={`entry-item${isHighlighted ? " entry-item--highlighted" : ""}${copyState === "copied" ? " entry-item--copied" : ""}${copyState === "failed" ? " entry-item--failed" : ""}`}
      onClick={() => copy(entry.id)}
    >
      <div class="entry-item__icon">{entry.name.charAt(0)}</div>
      <div class="entry-item__info">
        <span class="entry-item__name">{entry.name}</span>
        {entry.account && <span class="entry-item__url">{entry.account}</span>}
        {entry.url && <span class="entry-item__url">{entry.url}</span>}
      </div>
      <span class="entry-item__password">{entry.password_preview}</span>
      <div class="entry-item__actions" onClick={(e) => e.stopPropagation()}>
        <button
          type="button"
          class="entry-item__action"
          onClick={() => onEdit?.(entry)}
          aria-label={`Edit ${entry.name}`}
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M9.8 3.2 12.8 6.2M2.8 10.7 9.9 3.6a2.1 2.1 0 0 1 3 3l-7.1 7.1-3.6.8.6-3.8Z" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <button
          type="button"
          class="entry-item__action entry-item__action--danger"
          onClick={() => onDelete?.(entry)}
          aria-label={`Delete ${entry.name}`}
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M3.5 4.5h9M6.5 2.5h3M5 4.5l.5 8.5h5l.5-8.5M7 7v4M9 7v4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </div>
      {copyState === "copied" && <span class="entry-item__copied-badge">Copied</span>}
      {copyState === "failed" && <span class="entry-item__failed-badge">Failed</span>}
    </div>
  );
}
