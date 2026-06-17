import { invoke } from "@tauri-apps/api/core";

export interface EntrySearchResult {
  id: string;
  name: string;
  url: string;
  description: string;
  alias: string;
  account: string;
  tags: string;
  password_preview: string;
}

export interface Entry {
  id: string;
  name: string;
  url: string;
  description: string;
  alias: string;
  account: string;
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
  account?: string;
  password: string;
  tags?: string;
}

export interface PasswordTemplateSearchResult {
  id: string;
  name: string;
  description: string;
  password_preview: string;
}

export interface CreatePasswordTemplatePayload {
  name: string;
  description?: string;
  password: string;
}

export interface UsedPasswordTemplate {
  password: string;
}

export function unlockVault(masterPassword: string): Promise<boolean> {
  return invoke("unlock_vault", { masterPassword });
}

export function lockVault(): Promise<void> {
  return invoke("lock_vault");
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
    account: payload.account ?? null,
    password: payload.password,
    tags: payload.tags ?? null,
  });
}

export function copyToClipboard(text: string): Promise<void> {
  return invoke("copy_to_clipboard", { text });
}

export function copyEntryPassword(id: string): Promise<void> {
  return invoke("copy_entry_password", { id });
}

export function searchPasswordTemplates(
  query: string
): Promise<PasswordTemplateSearchResult[]> {
  return invoke("search_password_templates", { query });
}

export function createPasswordTemplate(
  payload: CreatePasswordTemplatePayload
): Promise<PasswordTemplateSearchResult> {
  return invoke("create_password_template", {
    name: payload.name,
    description: payload.description ?? null,
    password: payload.password,
  });
}

export function usePasswordTemplate(id: string): Promise<UsedPasswordTemplate> {
  return invoke("use_password_template", { id });
}
