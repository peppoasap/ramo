import { useEffect, useState } from "react";
import type { AiConfig, Provider } from "../domain";
import { call } from "../bridge";
import { Dialog } from "./Dialog";
export function ProviderDialog({
  config,
  onSave,
  onClose,
}: {
  config: AiConfig;
  onSave: (config: AiConfig) => void;
  onClose: () => void;
}) {
  const [draft, setDraft] = useState(config);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [codexPath, setCodexPath] = useState<string | null>(null);
  const [choosing, setChoosing] = useState(false);
  const [discovering, setDiscovering] = useState(false);
  const [models, setModels] = useState<string[]>([]);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    let current = true;
    call<string | null>("codex_cli_path")
      .then((path) => {
        if (current) setCodexPath(path);
      })
      .catch(() => {});
    call<Provider[]>("ai_providers")
      .then((v) => {
        if (current) setProviders(v);
      })
      .catch((e) => {
        if (current) setError(String(e));
      })
      .finally(() => {
        if (current) setLoading(false);
      });
    return () => {
      current = false;
    };
  }, []);
  async function chooseCodex() {
    setChoosing(true);
    setError("");
    try {
      const path = await call<string | null>("choose_codex_cli");
      if (path) {
        setCodexPath(path);
        setProviders(await call<Provider[]>("ai_providers"));
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setChoosing(false);
    }
  }
  async function findModels() {
    setError("");
    setDiscovering(true);
    try {
      setModels(await call<string[]>("ollama_models"));
    } catch (e) {
      setError(String(e));
    } finally {
      setDiscovering(false);
    }
  }
  return (
    <Dialog
      title="AI settings"
      className="provider-dialog"
      locked={choosing || discovering}
      onClose={onClose}
    >
      <p>
        Choose the tool you already use. Git works independently; no Ramo
        account is required.
      </p>
      {error && (
        <p role="alert" className="warning-text">
          {error}
        </p>
      )}
      {loading && <p role="status">Detecting installed providers…</p>}
      <div className="provider-options">
        {providers.map((provider) => (
          <label
            key={provider.id}
            className={draft.provider === provider.id ? "chosen" : ""}
          >
            <input
              disabled={choosing || discovering}
              type="radio"
              name="provider"
              checked={draft.provider === provider.id}
              onChange={() => {
                setDraft({ provider: provider.id, model: "" });
                setModels([]);
                setError("");
              }}
            />
            <span>
              <strong>{provider.name}</strong>
              <em>
                {provider.available
                  ? provider.id === "ollama"
                    ? "Local service required"
                    : "CLI detected · compatibility checked before use"
                  : "Not detected in app environment"}
              </em>
            </span>
          </label>
        ))}
      </div>
      <p className="provider-requirements">
        {providers.find((provider) => provider.id === draft.provider)?.detail}
      </p>
      <label>
        Model {draft.provider !== "ollama" && "(optional)"}
        <input
          disabled={choosing || discovering}
          value={draft.model}
          maxLength={200}
          onChange={(e) => setDraft({ ...draft, model: e.target.value })}
          placeholder={
            draft.provider === "ollama"
              ? "e.g. qwen2.5-coder:7b"
              : draft.provider === "codex"
                ? "Use your Codex model preference"
                : "Use provider default"
          }
          list="ai-models"
        />
      </label>
      <datalist id="ai-models">
        {models.map((model) => (
          <option key={model} value={model} />
        ))}
      </datalist>
      {draft.provider === "ollama" && (
        <button disabled={discovering} onClick={findModels}>
          {discovering ? "Finding models…" : "Find local models"}
        </button>
      )}
      {draft.provider === "codex" && (
        <details className="provider-advanced">
          <summary>Codex installation</summary>
          <button disabled={choosing} onClick={chooseCodex}>
            {choosing ? "Checking CLI…" : "Choose installed Codex CLI…"}
          </button>
          <p>{codexPath || "Uses Codex from the app environment."}</p>
          <p>
            CLI selection is saved when chosen. Requires Codex 0.166.0. Sign in
            with its own <code>codex login</code> first; Ramo never imports
            credentials.
          </p>
        </details>
      )}
      <div className="dialog-actions">
        <button disabled={choosing || discovering} onClick={onClose}>
          Cancel
        </button>
        <button
          className="primary"
          disabled={
            choosing ||
            discovering ||
            !providers.length ||
            (draft.provider === "ollama" && !draft.model.trim())
          }
          onClick={() => onSave(draft)}
        >
          Save settings
        </button>
      </div>
    </Dialog>
  );
}
