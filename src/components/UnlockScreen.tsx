import { useEffect, useRef, useState } from "preact/hooks";
import { unlockVault } from "../lib/tauri";

interface UnlockScreenProps {
  onUnlocked: () => void;
}

export function UnlockScreen({ onUnlocked }: UnlockScreenProps) {
  const [masterPassword, setMasterPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSubmit = async (event: Event) => {
    event.preventDefault();
    if (!masterPassword) {
      setError("Master password is required.");
      return;
    }

    setSubmitting(true);
    setError(null);
    try {
      await unlockVault(masterPassword);
      setMasterPassword("");
      onUnlocked();
    } catch (err) {
      setError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div class="unlock-screen">
      <form class="unlock-panel" onSubmit={handleSubmit}>
        <div class="unlock-header">
          <h1 class="unlock-title">Prompt Password</h1>
          <span class="unlock-subtitle">Vault locked</span>
        </div>
        <div class="form-field">
          <label class="form-label" for="unlock-master-password">
            Master Password
          </label>
          <input
            ref={inputRef}
            id="unlock-master-password"
            class="form-input"
            type="password"
            value={masterPassword}
            onInput={(e) => setMasterPassword((e.target as HTMLInputElement).value)}
            autocomplete="current-password"
          />
        </div>
        {error && <div class="form-error">{error}</div>}
        <button type="submit" class="btn btn-save unlock-submit" disabled={submitting}>
          {submitting ? "Unlocking..." : "Unlock"}
        </button>
      </form>
    </div>
  );
}
