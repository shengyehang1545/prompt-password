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

export interface CreateEntryPayload {
  name: string;
  url?: string;
  description?: string;
  alias?: string;
  account?: string;
  password: string;
  tags?: string;
}

export interface UpdateEntryPayload {
  id: string;
  name: string;
  url?: string;
  description?: string;
  alias?: string;
  account?: string;
  password?: string;
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

export interface UpdatePasswordTemplatePayload {
  id: string;
  name: string;
  description?: string;
  password?: string;
}

export interface UsedPasswordTemplate {
  password: string;
}

export type ExportFormat = "json" | "csv" | "txt";

export interface BackupFile {
  filename: string;
  content: string;
}

export interface ImportSummary {
  imported_entries: number;
  skipped_entries: number;
  imported_templates: number;
  skipped_templates: number;
}

export interface AppSettings {
  auto_start_enabled: boolean;
  hotkey: string;
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

export function createEntry(payload: CreateEntryPayload): Promise<EntrySearchResult> {
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

export function updateEntry(payload: UpdateEntryPayload): Promise<EntrySearchResult> {
  return invoke("update_entry", {
    id: payload.id,
    name: payload.name,
    url: payload.url ?? null,
    description: payload.description ?? null,
    alias: payload.alias ?? null,
    account: payload.account ?? null,
    password: payload.password ?? null,
    tags: payload.tags ?? null,
  });
}

export function deleteEntry(id: string): Promise<void> {
  return invoke("delete_entry", { id });
}

export function generatePassword(): Promise<string> {
  return invoke("generate_password");
}

export function exportVaultBackup(
  masterPassword: string,
  format: ExportFormat
): Promise<BackupFile> {
  return invoke("export_vault_backup", { masterPassword, format });
}

export function importVaultBackup(
  masterPassword: string,
  backupJson: string
): Promise<ImportSummary> {
  return invoke("import_vault_backup", { masterPassword, backupJson });
}

export function getAppSettings(): Promise<AppSettings> {
  return invoke("get_app_settings");
}

export function setAutoStartEnabled(enabled: boolean): Promise<AppSettings> {
  return invoke("set_auto_start_enabled", { enabled });
}

export function setGlobalHotkey(hotkey: string): Promise<AppSettings> {
  return invoke("set_global_hotkey", { hotkey });
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

export function updatePasswordTemplate(
  payload: UpdatePasswordTemplatePayload
): Promise<PasswordTemplateSearchResult> {
  return invoke("update_password_template", {
    id: payload.id,
    name: payload.name,
    description: payload.description ?? null,
    password: payload.password ?? null,
  });
}

export function deletePasswordTemplate(id: string): Promise<void> {
  return invoke("delete_password_template", { id });
}

export function usePasswordTemplate(id: string): Promise<UsedPasswordTemplate> {
  return invoke("use_password_template", { id });
}
