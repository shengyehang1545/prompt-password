import { useState, useEffect, useRef, useCallback } from "preact/hooks";
import {
  createEntry,
  createPasswordTemplate,
  deletePasswordTemplate,
  generatePassword,
  searchEntries,
  searchPasswordTemplates,
  updateEntry,
  updatePasswordTemplate,
  usePasswordTemplate,
  type EntrySearchResult,
  type PasswordTemplateSearchResult,
} from "../lib/tauri";
import { inputGuards } from "../lib/inputGuards";

interface AddEntryFormProps {
  entry?: EntrySearchResult | null;
  onSuccess: () => void;
  onClose: () => void;
}

export function AddEntryForm({ entry, onSuccess, onClose }: AddEntryFormProps) {
  const isEditing = Boolean(entry);
  const [name, setName] = useState(entry?.name ?? "");
  const [url, setUrl] = useState(entry?.url ?? "");
  const [account, setAccount] = useState(entry?.account ?? "");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Template search
  const [templateQuery, setTemplateQuery] = useState("");
  const [templateResults, setTemplateResults] = useState<PasswordTemplateSearchResult[]>([]);
  const [templateOpen, setTemplateOpen] = useState(false);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string | null>(null);
  const [showTemplateCreate, setShowTemplateCreate] = useState(false);
  const [templateName, setTemplateName] = useState("");
  const [templateDescription, setTemplateDescription] = useState("");
  const [templatePassword, setTemplatePassword] = useState("");
  const [showTemplatePassword, setShowTemplatePassword] = useState(false);
  const [editingTemplateId, setEditingTemplateId] = useState<string | null>(null);
  const [templateSubmitting, setTemplateSubmitting] = useState(false);
  const templateTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const templateRef = useRef<HTMLDivElement>(null);

  // Tags
  const [selectedTags, setSelectedTags] = useState<string[]>(
    entry?.tags ? entry.tags.split(",").map((tag) => tag.trim()).filter(Boolean) : []
  );
  const [tagInput, setTagInput] = useState("");
  const [allTags, setAllTags] = useState<string[]>([]);
  const [tagSuggestions, setTagSuggestions] = useState<string[]>([]);
  const [tagDropdownOpen, setTagDropdownOpen] = useState(false);
  const tagRef = useRef<HTMLDivElement>(null);

  const nameRef = useRef<HTMLInputElement>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  // Focus name field on mount
  useEffect(() => {
    nameRef.current?.focus();
  }, []);

  // Fetch existing tags on mount
  useEffect(() => {
    let cancelled = false;
    searchEntries("").then((entries) => {
      if (cancelled) return;
      const tagSet = new Set<string>();
      for (const e of entries) {
        if (e.tags) {
          e.tags.split(",").forEach((t) => {
            const trimmed = t.trim();
            if (trimmed) tagSet.add(trimmed);
          });
        }
      }
      setAllTags(Array.from(tagSet).sort());
    }).catch(() => {});
    return () => { cancelled = true; };
  }, []);

  // Cleanup debounce timer on unmount
  useEffect(() => {
    return () => {
      if (templateTimerRef.current) clearTimeout(templateTimerRef.current);
    };
  }, []);

  // Escape to close (using ref to avoid re-registration)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onCloseRef.current();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Close dropdowns on outside click
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (templateRef.current && !templateRef.current.contains(e.target as Node)) {
        setTemplateOpen(false);
      }
      if (tagRef.current && !tagRef.current.contains(e.target as Node)) {
        setTagDropdownOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  // Template search with debounce
  const searchTemplates = useCallback((q: string) => {
    if (templateTimerRef.current) clearTimeout(templateTimerRef.current);

    const trimmed = q.trim();
    templateTimerRef.current = setTimeout(async () => {
      try {
        const results = await searchPasswordTemplates(trimmed);
        setTemplateResults(results);
        setTemplateOpen(results.length > 0);
      } catch {
        setTemplateResults([]);
      }
    }, 200);
  }, []);

  // Filter tag suggestions
  useEffect(() => {
    const q = tagInput.trim().toLowerCase();
    if (!q) {
      setTagSuggestions(allTags.filter((t) => !selectedTags.includes(t)));
    } else {
      setTagSuggestions(
        allTags.filter(
          (t) => t.toLowerCase().includes(q) && !selectedTags.includes(t)
        )
      );
    }
  }, [tagInput, allTags, selectedTags]);

  const handleTemplateInput = (value: string) => {
    setTemplateQuery(value);
    setSelectedTemplateId(null);
    searchTemplates(value);
  };

  const resetTemplateEditor = () => {
    setEditingTemplateId(null);
    setTemplateName("");
    setTemplateDescription("");
    setTemplatePassword("");
    setShowTemplatePassword(false);
  };

  const toggleNewTemplate = () => {
    if (showTemplateCreate) {
      setShowTemplateCreate(false);
      resetTemplateEditor();
      return;
    }

    resetTemplateEditor();
    setShowTemplateCreate(true);
  };

  const handleSelectTemplate = async (entry: PasswordTemplateSearchResult) => {
    setTemplateQuery(entry.name);
    setSelectedTemplateId(entry.id);
    setTemplateOpen(false);
    setError(null);

    try {
      const usedTemplate = await usePasswordTemplate(entry.id);
      setPassword(usedTemplate.password);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleSaveTemplate = async () => {
    const trimmedName = templateName.trim();
    if (!trimmedName) {
      setError("Template name is required.");
      return;
    }
    if (!editingTemplateId && !templatePassword) {
      setError("Template password is required.");
      return;
    }

    setTemplateSubmitting(true);
    setError(null);
    try {
      const saved = editingTemplateId
        ? await updatePasswordTemplate({
            id: editingTemplateId,
            name: trimmedName,
            description: templateDescription.trim() || undefined,
            password: templatePassword || undefined,
          })
        : await createPasswordTemplate({
            name: trimmedName,
            description: templateDescription.trim() || undefined,
            password: templatePassword,
          });

      setTemplateResults((prev) => {
        const withoutSaved = prev.filter((item) => item.id !== saved.id);
        return [saved, ...withoutSaved];
      });
      if (selectedTemplateId === saved.id) {
        setTemplateQuery(saved.name);
      }
      setShowTemplateCreate(false);
      resetTemplateEditor();
      if (!editingTemplateId) {
        await handleSelectTemplate(saved);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setTemplateSubmitting(false);
    }
  };

  const handleEditTemplate = (entry: PasswordTemplateSearchResult) => {
    setEditingTemplateId(entry.id);
    setTemplateName(entry.name);
    setTemplateDescription(entry.description);
    setTemplatePassword("");
    setShowTemplatePassword(false);
    setShowTemplateCreate(true);
    setTemplateOpen(false);
  };

  const handleDeleteTemplate = async (entry: PasswordTemplateSearchResult) => {
    const confirmed = window.confirm(`Delete template "${entry.name}"?`);
    if (!confirmed) return;

    setError(null);
    try {
      await deletePasswordTemplate(entry.id);
      setTemplateResults((prev) => prev.filter((item) => item.id !== entry.id));
      if (selectedTemplateId === entry.id) {
        setSelectedTemplateId(null);
        setTemplateQuery("");
      }
      if (editingTemplateId === entry.id) {
        setShowTemplateCreate(false);
        resetTemplateEditor();
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleGeneratePassword = async () => {
    setGenerating(true);
    setError(null);
    try {
      const generated = await generatePassword();
      setPassword(generated);
      setShowPassword(true);
    } catch (err) {
      setError(String(err));
    } finally {
      setGenerating(false);
    }
  };

  const handleAddTag = (tag: string) => {
    const trimmed = tag.trim();
    if (trimmed && !selectedTags.includes(trimmed)) {
      setSelectedTags((prev) => [...prev, trimmed]);
    }
    setTagInput("");
    setTagDropdownOpen(false);
  };

  const handleRemoveTag = (tag: string) => {
    setSelectedTags((prev) => prev.filter((t) => t !== tag));
  };

  const handleTagKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      if (tagInput.trim()) handleAddTag(tagInput);
    } else if (e.key === "Backspace" && !tagInput && selectedTags.length > 0) {
      setSelectedTags((prev) => prev.slice(0, -1));
    } else if (e.key === "Escape") {
      e.stopPropagation();
      setTagDropdownOpen(false);
    }
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) {
      setError("Name is required.");
      return;
    }
    if (!isEditing && !password) {
      setError("Password is required.");
      return;
    }

    setSubmitting(true);
    setError(null);
    try {
      const payload = {
        name: trimmedName,
        url: url.trim() || undefined,
        account: account.trim() || undefined,
        tags: selectedTags.length > 0 ? selectedTags.join(",") : undefined,
      };

      if (entry) {
        await updateEntry({
          id: entry.id,
          ...payload,
          password: password ? password : undefined,
        });
      } else {
        await createEntry({
          ...payload,
          password,
        });
      }
      onSuccess();
    } catch (err) {
      setError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  const handleOverlayClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) onClose();
  };

  return (
    <div class="modal-overlay" onClick={handleOverlayClick}>
      <div class="modal-panel">
        <div class="modal-header">
          <h2 class="modal-title">{isEditing ? "Edit Entry" : "Add Entry"}</h2>
          <button class="modal-close" onClick={onClose} type="button" aria-label="Close">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </div>

        <form class="modal-form" onSubmit={handleSubmit}>
          <div class="form-field">
            <label class="form-label" for="ae-name">
              Name <span class="form-required">*</span>
            </label>
            <input
              ref={nameRef}
              id="ae-name"
              class="form-input"
              type="text"
              value={name}
              onInput={(e) => setName((e.target as HTMLInputElement).value)}
              placeholder="e.g. GitHub"
              autocomplete="off"
              {...inputGuards}
            />
          </div>

          <div class="form-field">
            <label class="form-label" for="ae-url">URL</label>
            <input
              id="ae-url"
              class="form-input"
              type="text"
              value={url}
              onInput={(e) => setUrl((e.target as HTMLInputElement).value)}
              placeholder="e.g. https://github.com"
              autocomplete="off"
              {...inputGuards}
            />
          </div>

          <div class="form-field">
            <label class="form-label" for="ae-account">Account</label>
            <input
              id="ae-account"
              class="form-input"
              type="text"
              value={account}
              onInput={(e) => setAccount((e.target as HTMLInputElement).value)}
              placeholder="e.g. octocat@example.com"
              autocomplete="off"
              {...inputGuards}
            />
          </div>

          <div class="form-field" ref={tagRef}>
            <label class="form-label" for="ae-tags">Tags</label>
            <div class="tag-input-wrapper" onClick={() => setTagDropdownOpen(true)}>
              {selectedTags.map((tag) => (
                <span class="tag-chip" key={tag}>
                  {tag}
                  <button
                    type="button"
                    class="tag-chip-remove"
                    onClick={(e) => { e.stopPropagation(); handleRemoveTag(tag); }}
                    aria-label={`Remove ${tag}`}
                  >
                    <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                      <path d="M2 2l6 6M8 2l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                    </svg>
                  </button>
                </span>
              ))}
              <input
                id="ae-tags"
                class="tag-input"
                type="text"
                value={tagInput}
                onInput={(e) => {
                  setTagInput((e.target as HTMLInputElement).value);
                  setTagDropdownOpen(true);
                }}
                onFocus={() => setTagDropdownOpen(true)}
                onKeyDown={handleTagKeyDown}
                placeholder={selectedTags.length === 0 ? "Search or type new tag..." : ""}
                autocomplete="off"
                {...inputGuards}
              />
            </div>
            {tagDropdownOpen && tagSuggestions.length > 0 && (
              <div class="tag-dropdown">
                {tagSuggestions.map((tag) => (
                  <button
                    type="button"
                    class="tag-dropdown-item"
                    key={tag}
                    onClick={() => handleAddTag(tag)}
                  >
                    {tag}
                  </button>
                ))}
                {tagInput.trim() && !allTags.includes(tagInput.trim()) && !selectedTags.includes(tagInput.trim()) && (
                  <button
                    type="button"
                    class="tag-dropdown-item tag-dropdown-item--new"
                    onClick={() => handleAddTag(tagInput)}
                  >
                    Create "{tagInput.trim()}"
                  </button>
                )}
              </div>
            )}
          </div>

          <div class="form-field" ref={templateRef}>
            <div class="form-label-row">
              <label class="form-label" for="ae-template">Password Template</label>
              <button
                type="button"
                class="inline-action-btn"
                onClick={toggleNewTemplate}
              >
                {showTemplateCreate ? "Cancel" : "New"}
              </button>
            </div>
            <input
              id="ae-template"
              class="form-input"
              type="text"
              value={templateQuery}
              onInput={(e) => handleTemplateInput((e.target as HTMLInputElement).value)}
              onFocus={() => {
                if (templateResults.length > 0) {
                  setTemplateOpen(true);
                } else {
                  searchTemplates(templateQuery);
                }
              }}
              placeholder="Search templates..."
              autocomplete="off"
              {...inputGuards}
            />
            {templateOpen && templateResults.length > 0 && (
              <div class="template-dropdown">
                {templateResults.map((entry) => (
                  <div
                    class={`template-dropdown-item${entry.id === selectedTemplateId ? " template-dropdown-item--selected" : ""}`}
                    key={entry.id}
                  >
                    <button
                      type="button"
                      class="template-item-select"
                      onClick={() => handleSelectTemplate(entry)}
                    >
                      <span class="template-item-name">{entry.name}</span>
                      <span class="template-item-pass">{entry.password_preview}</span>
                    </button>
                    <div class="template-item-actions">
                      <button
                        type="button"
                        class="template-item-action"
                        onClick={() => handleEditTemplate(entry)}
                        aria-label={`Edit template ${entry.name}`}
                      >
                        <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
                          <path d="M9.8 3.2 12.8 6.2M2.8 10.7 9.9 3.6a2.1 2.1 0 0 1 3 3l-7.1 7.1-3.6.8.6-3.8Z" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                      </button>
                      <button
                        type="button"
                        class="template-item-action template-item-action--danger"
                        onClick={() => handleDeleteTemplate(entry)}
                        aria-label={`Delete template ${entry.name}`}
                      >
                        <svg width="13" height="13" viewBox="0 0 16 16" fill="none">
                          <path d="M3.5 4.5h9M6.5 2.5h3M5 4.5l.5 8.5h5l.5-8.5M7 7v4M9 7v4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
            {showTemplateCreate && (
              <div class="template-create-panel">
                <input
                  class="form-input"
                  type="text"
                  value={templateName}
                  onInput={(e) => setTemplateName((e.target as HTMLInputElement).value)}
                  placeholder="Template name"
                  autocomplete="off"
                  {...inputGuards}
                />
                <input
                  class="form-input"
                  type="text"
                  value={templateDescription}
                  onInput={(e) => setTemplateDescription((e.target as HTMLInputElement).value)}
                  placeholder="Description"
                  autocomplete="off"
                  {...inputGuards}
                />
                <div class="form-password-wrapper">
                  <input
                    class="form-input form-password-input"
                    type={showTemplatePassword ? "text" : "password"}
                    value={templatePassword}
                    onInput={(e) => setTemplatePassword((e.target as HTMLInputElement).value)}
                    placeholder={editingTemplateId ? "Leave blank to keep template password" : "Template password"}
                    autocomplete="off"
                    {...inputGuards}
                  />
                  <button
                    type="button"
                    class="form-password-toggle"
                    onClick={() => setShowTemplatePassword((v) => !v)}
                    aria-label={showTemplatePassword ? "Hide template password" : "Show template password"}
                  >
                    {showTemplatePassword ? (
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path d="M8 3C4.5 3 1.5 5.5 0.5 8c1 2.5 4 5 7.5 5s6.5-2.5 7.5-5c-1-2.5-4-5-7.5-5z" stroke="currentColor" stroke-width="1.2" />
                        <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.2" />
                        <path d="M2 2l12 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                      </svg>
                    ) : (
                      <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                        <path d="M8 3C4.5 3 1.5 5.5 0.5 8c1 2.5 4 5 7.5 5s6.5-2.5 7.5-5c-1-2.5-4-5-7.5-5z" stroke="currentColor" stroke-width="1.2" />
                        <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.2" />
                      </svg>
                    )}
                  </button>
                </div>
                <button
                  type="button"
                  class="btn btn-save template-create-save"
                  onClick={handleSaveTemplate}
                  disabled={templateSubmitting}
                >
                  {templateSubmitting ? "Saving..." : editingTemplateId ? "Update Template" : "Save Template"}
                </button>
              </div>
            )}
          </div>

          <div class="form-field">
            <div class="form-label-row">
              <label class="form-label" for="ae-password">
                {isEditing ? "New Password" : "Password"} {!isEditing && <span class="form-required">*</span>}
              </label>
              <button
                type="button"
                class="inline-action-btn"
                onClick={handleGeneratePassword}
                disabled={generating}
              >
                {generating ? "Generating..." : "Generate"}
              </button>
            </div>
            <div class="form-password-wrapper">
              <input
                id="ae-password"
                class="form-input form-password-input"
                type={showPassword ? "text" : "password"}
                value={password}
                onInput={(e) => setPassword((e.target as HTMLInputElement).value)}
                placeholder={isEditing ? "Leave blank to keep current password" : "Enter password"}
                autocomplete="off"
                {...inputGuards}
              />
              <button
                type="button"
                class="form-password-toggle"
                onClick={() => setShowPassword((v) => !v)}
                aria-label={showPassword ? "Hide password" : "Show password"}
              >
                {showPassword ? (
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path d="M8 3C4.5 3 1.5 5.5 0.5 8c1 2.5 4 5 7.5 5s6.5-2.5 7.5-5c-1-2.5-4-5-7.5-5z" stroke="currentColor" stroke-width="1.2" />
                    <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.2" />
                    <path d="M2 2l12 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                  </svg>
                ) : (
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                    <path d="M8 3C4.5 3 1.5 5.5 0.5 8c1 2.5 4 5 7.5 5s6.5-2.5 7.5-5c-1-2.5-4-5-7.5-5z" stroke="currentColor" stroke-width="1.2" />
                    <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.2" />
                  </svg>
                )}
              </button>
            </div>
          </div>

          {error && <div class="form-error">{error}</div>}

          <div class="modal-actions">
            <button type="button" class="btn btn-cancel" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" class="btn btn-save" disabled={submitting}>
              {submitting ? "Saving..." : isEditing ? "Update" : "Save"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
