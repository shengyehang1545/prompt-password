import { useRef, useEffect } from "preact/hooks";
import type { EntrySearchResult } from "../lib/tauri";
import { useClipboard } from "../hooks/useClipboard";

interface EntryItemProps {
  entry: EntrySearchResult;
  isHighlighted?: boolean;
  shouldCopy?: boolean;
  onCopied?: () => void;
}

export function EntryItem({ entry, isHighlighted, shouldCopy, onCopied }: EntryItemProps) {
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
      {copyState === "copied" && <span class="entry-item__copied-badge">Copied</span>}
      {copyState === "failed" && <span class="entry-item__failed-badge">Failed</span>}
    </div>
  );
}
