import { invoke } from "@tauri-apps/api/core";

export interface EntrySearchResult {
  id: string;
  name: string;
  url: string;
  description: string;
  alias: string;
  tags: string;
  password_enc: string;
}

export interface Entry {
  id: string;
  name: string;
  url: string;
  description: string;
  alias: string;
  password_enc: string;
  password_nonce: string;
  tags: string;
  created_at: string;
  updated_at: string;
}

export interface CreateEntryPayload {
  name: string;
  url?: string;
  description?: string;
  alias?: string;
  password: string;
  tags?: string;
}

export function searchEntries(query: string): Promise<EntrySearchResult[]> {
  return invoke("search_entries", { query });
}

export function getEntry(id: string): Promise<Entry> {
  return invoke("get_entry", { id });
}

export function createEntry(payload: CreateEntryPayload): Promise<Entry> {
  return invoke("create_entry", {
    name: payload.name,
    url: payload.url ?? null,
    description: payload.description ?? null,
    alias: payload.alias ?? null,
    password: payload.password,
    tags: payload.tags ?? null,
  });
}

export function copyToClipboard(text: string): Promise<void> {
  return invoke("copy_to_clipboard", { text });
}
