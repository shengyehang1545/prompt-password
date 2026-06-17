import { useRef, useEffect } from "preact/hooks";
import type { ComponentChildren } from "preact";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { inputGuards } from "../lib/inputGuards";

interface SearchInputProps {
  value: string;
  onInput: (value: string) => void;
  onKeyDown?: (e: KeyboardEvent) => void;
  loading?: boolean;
  children?: ComponentChildren;
}

export function SearchInput({ value, onInput, onKeyDown, loading, children }: SearchInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();

    const unlisten = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) inputRef.current?.focus();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div class="search-input-wrapper">
      <svg
        class="search-icon"
        width="16" height="16"
        viewBox="0 0 16 16" fill="none"
      >
        <path
          d="M7.333 12.667A5.333 5.333 0 1 0 7.333 2a5.333 5.333 0 0 0 0 10.667ZM11.333 11.333 14 14"
          stroke="currentColor" stroke-width="1.33" stroke-linecap="round" stroke-linejoin="round"
        />
      </svg>
      <input
        ref={inputRef}
        class="search-input"
        type="text"
        placeholder="Search entries..."
        value={value}
        onInput={(e) => onInput((e.target as HTMLInputElement).value)}
        onKeyDown={onKeyDown}
        autofocus
        {...inputGuards}
      />
      {loading && <span class="search-spinner" />}
      {children}
    </div>
  );
}
