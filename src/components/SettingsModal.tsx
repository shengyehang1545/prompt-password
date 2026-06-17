import { useEffect, useState } from "preact/hooks";
import { inputGuards } from "../lib/inputGuards";
import type { AppSettings, ExportFormat } from "../lib/tauri";

interface SettingsModalProps {
  settings: AppSettings | null;
  onClose: () => void;
  onLockVault: () => Promise<void>;
  onAutoStartChange: (enabled: boolean) => Promise<void>;
  onHotkeySave: (hotkey: string) => Promise<void>;
  onExport: (format: ExportFormat) => void;
  onImport: () => void;
}

export function SettingsModal({
  settings,
  onClose,
  onLockVault,
  onAutoStartChange,
  onHotkeySave,
  onExport,
  onImport,
}: SettingsModalProps) {
  const [hotkeyDraft, setHotkeyDraft] = useState(settings?.hotkey ?? "");
  const [pending, setPending] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setHotkeyDraft(settings?.hotkey ?? "");
  }, [settings?.hotkey]);

  const run = async (label: string, action: () => Promise<void>, success: string) => {
    setPending(label);
    setError(null);
    setMessage(null);
    try {
      await action();
      setMessage(success);
    } catch (err) {
      setError(String(err));
    } finally {
      setPending(null);
    }
  };

  return (
    <div class="modal-overlay" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div class="modal-panel settings-panel">
        <div class="modal-header">
          <h2 class="modal-title">Settings</h2>
          <button class="modal-close" onClick={onClose} type="button" aria-label="Close">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </div>

        <div class="settings-content">
          <section class="settings-section">
            <h3 class="settings-section-title">Startup</h3>
            <label class="settings-row">
              <span>
                <span class="settings-row-title">Launch at login</span>
                <span class="settings-row-detail">Start Prompt Password in the background.</span>
              </span>
              <input
                type="checkbox"
                checked={settings?.auto_start_enabled ?? false}
                disabled={!settings || pending === "auto-start"}
                onChange={(e) => {
                  const enabled = (e.target as HTMLInputElement).checked;
                  run("auto-start", () => onAutoStartChange(enabled), "Startup setting saved.");
                }}
              />
            </label>
          </section>

          <section class="settings-section">
            <h3 class="settings-section-title">Hotkey</h3>
            <div class="settings-inline-form">
              <input
                class="form-input"
                type="text"
                value={hotkeyDraft}
                onInput={(e) => setHotkeyDraft((e.target as HTMLInputElement).value)}
                placeholder="Cmd+Shift+K"
                autocomplete="off"
                {...inputGuards}
              />
              <button
                type="button"
                class="btn btn-save"
                disabled={!settings || pending === "hotkey"}
                onClick={() => run("hotkey", () => onHotkeySave(hotkeyDraft), "Hotkey saved.")}
              >
                Save
              </button>
            </div>
          </section>

          <section class="settings-section">
            <h3 class="settings-section-title">Backup</h3>
            <div class="settings-button-row">
              <button type="button" class="btn btn-cancel" onClick={onImport}>
                Import JSON
              </button>
              <button type="button" class="btn btn-cancel" onClick={() => onExport("json")}>
                Export JSON
              </button>
              <button type="button" class="btn btn-cancel" onClick={() => onExport("csv")}>
                Export CSV
              </button>
              <button type="button" class="btn btn-cancel" onClick={() => onExport("txt")}>
                Export TXT
              </button>
            </div>
          </section>

          <section class="settings-section">
            <h3 class="settings-section-title">Security</h3>
            <button
              type="button"
              class="btn btn-save settings-lock-btn"
              disabled={pending === "lock"}
              onClick={() => run("lock", onLockVault, "Vault locked.")}
            >
              Lock Vault
            </button>
          </section>

          {message && <div class="form-note">{message}</div>}
          {error && <div class="form-error">{error}</div>}
        </div>
      </div>
    </div>
  );
}
