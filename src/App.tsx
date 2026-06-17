import { useState, useCallback, useEffect, useRef } from "preact/hooks";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  deleteEntry,
  exportVaultBackup,
  getAppSettings,
  importVaultBackup,
  lockVault,
  setAutoStartEnabled,
  setGlobalHotkey,
  type AppSettings,
  type EntrySearchResult,
  type ExportFormat,
} from "./lib/tauri";
import { inputGuards } from "./lib/inputGuards";
import { useSearch } from "./hooks/useSearch";
import { SearchInput } from "./components/SearchInput";
import { EntryList } from "./components/EntryList";
import { AddEntryForm } from "./components/AddEntryForm";
import { UnlockScreen } from "./components/UnlockScreen";
import { SettingsModal } from "./components/SettingsModal";
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
  const [editingEntry, setEditingEntry] = useState<EntrySearchResult | null>(null);
  const [unlocked, setUnlocked] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [appSettings, setAppSettings] = useState<AppSettings | null>(null);
  const [secureAction, setSecureAction] = useState<
    | { type: "export"; format: ExportFormat }
    | { type: "import"; backupJson: string; filename: string }
    | null
  >(null);
  const [securePassword, setSecurePassword] = useState("");
  const [secureSubmitting, setSecureSubmitting] = useState(false);
  const [secureError, setSecureError] = useState<string | null>(null);
  const importFileRef = useRef<HTMLInputElement>(null);

  const loadSettings = useCallback(async () => {
    try {
      setAppSettings(await getAppSettings());
    } catch (err) {
      window.alert(String(err));
    }
  }, []);

  useEffect(() => {
    if (unlocked) {
      loadSettings();
    }
  }, [unlocked, loadSettings]);

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
    setEditingEntry(null);
    refresh();
  }, [refresh]);

  const handleOpenAdd = useCallback(() => {
    setEditingEntry(null);
    setShowForm(true);
  }, []);

  const handleEdit = useCallback((entry: EntrySearchResult) => {
    setEditingEntry(entry);
    setShowForm(true);
  }, []);

  const handleDelete = useCallback(async (entry: EntrySearchResult) => {
    const confirmed = window.confirm(`Delete "${entry.name}"?`);
    if (!confirmed) return;

    try {
      await deleteEntry(entry.id);
      refresh();
    } catch (err) {
      window.alert(String(err));
    }
  }, [refresh]);

  const openExportDialog = useCallback((format: ExportFormat) => {
    setShowSettings(false);
    setSecureAction({ type: "export", format });
    setSecurePassword("");
    setSecureError(null);
  }, []);

  const handleImportClick = useCallback(() => {
    setShowSettings(false);
    importFileRef.current?.click();
  }, []);

  const handleImportFile = useCallback(async (event: Event) => {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;

    try {
      const backupJson = await file.text();
      setSecureAction({ type: "import", backupJson, filename: file.name });
      setSecurePassword("");
      setSecureError(null);
    } catch (err) {
      window.alert(String(err));
    }
  }, []);

  const closeSecureDialog = useCallback(() => {
    if (secureSubmitting) return;
    setSecureAction(null);
    setSecurePassword("");
    setSecureError(null);
  }, [secureSubmitting]);

  const handleSecureSubmit = useCallback(async (event: Event) => {
    event.preventDefault();
    if (!secureAction) return;
    if (!securePassword) {
      setSecureError("Master password is required.");
      return;
    }

    setSecureSubmitting(true);
    setSecureError(null);
    try {
      if (secureAction.type === "export") {
        const backup = await exportVaultBackup(securePassword, secureAction.format);
        downloadFile(backup.filename, backup.content, secureAction.format);
      } else {
        const summary = await importVaultBackup(securePassword, secureAction.backupJson);
        window.alert(
          `Imported entries: ${summary.imported_entries}\n` +
            `Skipped entries: ${summary.skipped_entries}\n` +
            `Imported templates: ${summary.imported_templates}\n` +
            `Skipped templates: ${summary.skipped_templates}`
        );
        refresh();
      }
      setSecureAction(null);
      setSecurePassword("");
    } catch (err) {
      setSecureError(String(err));
    } finally {
      setSecureSubmitting(false);
    }
  }, [secureAction, securePassword, refresh]);

  const handleLockVault = useCallback(async () => {
    await lockVault();
    setUnlocked(false);
    setShowSettings(false);
    setShowForm(false);
    setEditingEntry(null);
    setSecureAction(null);
    setSecurePassword("");
  }, []);

  const handleAutoStartChange = useCallback(async (enabled: boolean) => {
    const settings = await setAutoStartEnabled(enabled);
    setAppSettings(settings);
  }, []);

  const handleHotkeySave = useCallback(async (hotkey: string) => {
    const settings = await setGlobalHotkey(hotkey);
    setAppSettings(settings);
  }, []);

  if (!unlocked) {
    return <UnlockScreen onUnlocked={() => setUnlocked(true)} />;
  }

  return (
    <div class="app">
      <SearchInput
        value={query}
        onInput={handleInput}
        onKeyDown={handleKeyDown}
        loading={loading}
      >
        <input
          ref={importFileRef}
          type="file"
          accept=".json,application/json"
          onChange={handleImportFile}
          style={{ display: "none" }}
        />
        <button
          class="add-entry-btn"
          onClick={handleOpenAdd}
          type="button"
          aria-label="Add entry"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 3.5v9M3.5 8h9" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
        </button>
        <button
          class="add-entry-btn"
          onClick={() => setShowSettings(true)}
          type="button"
          aria-label="Settings"
          title="Settings"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M6.9 2.2h2.2l.4 1.5c.3.1.6.2.9.4l1.3-.8 1.6 1.6-.8 1.3c.2.3.3.6.4.9l1.5.4v2.2l-1.5.4c-.1.3-.2.6-.4.9l.8 1.3-1.6 1.6-1.3-.8c-.3.2-.6.3-.9.4l-.4 1.5H6.9l-.4-1.5c-.3-.1-.6-.2-.9-.4l-1.3.8-1.6-1.6.8-1.3c-.2-.3-.3-.6-.4-.9l-1.5-.4V7.5l1.5-.4c.1-.3.2-.6.4-.9l-.8-1.3 1.6-1.6 1.3.8c.3-.2.6-.3.9-.4l.4-1.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
            <circle cx="8" cy="8.6" r="2" stroke="currentColor" stroke-width="1.2" />
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
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      )}
      {showForm && (
        <AddEntryForm
          entry={editingEntry}
          onSuccess={handleAddSuccess}
          onClose={() => {
            setShowForm(false);
            setEditingEntry(null);
          }}
        />
      )}
      {showSettings && (
        <SettingsModal
          settings={appSettings}
          onClose={() => setShowSettings(false)}
          onLockVault={handleLockVault}
          onAutoStartChange={handleAutoStartChange}
          onHotkeySave={handleHotkeySave}
          onExport={openExportDialog}
          onImport={handleImportClick}
        />
      )}
      {secureAction && (
        <div class="modal-overlay" onClick={(e) => { if (e.target === e.currentTarget) closeSecureDialog(); }}>
          <div class="modal-panel secure-action-panel">
            <div class="modal-header">
              <h2 class="modal-title">
                {secureAction.type === "export" ? "Export Passwords" : "Import Backup"}
              </h2>
              <button class="modal-close" onClick={closeSecureDialog} type="button" aria-label="Close">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                  <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                </svg>
              </button>
            </div>
            <form class="modal-form" onSubmit={handleSecureSubmit}>
              {secureAction.type === "export" && secureAction.format !== "json" && (
                <div class="form-warning">
                  This export is plaintext. Store the file carefully.
                </div>
              )}
              {secureAction.type === "import" && (
                <div class="form-note">
                  Importing {secureAction.filename}. Existing entries and templates with the same id or name will be skipped.
                </div>
              )}
              <div class="form-field">
                <label class="form-label" for="secure-master-password">
                  Master Password
                </label>
                <input
                  id="secure-master-password"
                  class="form-input"
                  type="password"
                  value={securePassword}
                  onInput={(e) => setSecurePassword((e.target as HTMLInputElement).value)}
                  autocomplete="current-password"
                  {...inputGuards}
                />
              </div>
              {secureError && <div class="form-error">{secureError}</div>}
              <div class="modal-actions">
                <button type="button" class="btn btn-cancel" onClick={closeSecureDialog}>
                  Cancel
                </button>
                <button type="submit" class="btn btn-save" disabled={secureSubmitting}>
                  {secureSubmitting ? "Working..." : secureAction.type === "export" ? "Export" : "Import"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}

function downloadFile(filename: string, content: string, format: ExportFormat) {
  const mime = format === "json" ? "application/json" : format === "csv" ? "text/csv" : "text/plain";
  const blob = new Blob([content], { type: `${mime};charset=utf-8` });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}
