import { useState } from "react";
import {
  ArrowRight,
  Buildings,
  Cloud,
  FolderOpen,
  GitBranch,
  GithubLogo,
  GitlabLogo,
  LinkSimple,
  GearSix,
} from "@phosphor-icons/react";
const providerIcons = {
  url: LinkSimple,
  github: GithubLogo,
  githubEnterprise: GithubLogo,
  gitlab: GitlabLogo,
  bitbucket: GitBranch,
  azure: Cloud,
  custom: Buildings,
  local: FolderOpen,
};
import { call } from "../bridge";
import type { Snapshot } from "../domain";
import {
  repositoryProviders,
  providerUrl,
  suggestedDirectory,
  type ProviderId,
} from "../repositoryProviders";
import { GithubRepositories } from "./GithubRepositories";
import { Dialog } from "./Dialog";
interface Grant {
  id: string;
  path: string;
}
interface Plan {
  id: string;
  source: string;
  destination: string;
  network: boolean;
}
export function CloneDialog({
  close,
  opened,
}: {
  close: () => void;
  opened: (snapshot: Snapshot) => Promise<void>;
}) {
  const [provider, setProvider] = useState<ProviderId>("url");
  const [url, setUrl] = useState("");
  const [host, setHost] = useState("");
  const [ssh, setSsh] = useState(false);
  const [folderEdited, setFolderEdited] = useState(false);
  const local = provider === "local";
  let resolvedUrl = "",
    sourceError = "";
  try {
    resolvedUrl = providerUrl(provider, url, host, ssh);
  } catch (e) {
    sourceError = String(e).replace(/^Error: /, "");
  }
  function repository(value: string) {
    setUrl(value);
    if (!folderEdited) setDirectory(suggestedDirectory(value));
  }
  const [source, setSource] = useState<Grant | null>(null),
    [parent, setParent] = useState<Grant | null>(null);
  const [directory, setDirectory] = useState(""),
    [plan, setPlan] = useState<Plan | null>(null);
  const [trusted, setTrusted] = useState(false),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  async function pick(isSource: boolean) {
    setBusy(true);
    setError("");
    try {
      const selected = await call<Grant | null>("choose_clone_folder", {
        source: isSource,
      });
      if (selected) {
        (isSource ? setSource : setParent)(selected);
        if (isSource && !folderEdited)
          setDirectory(suggestedDirectory(selected.path));
      }
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  async function review() {
    if (!parent) return;
    setBusy(true);
    setError("");
    try {
      setPlan(
        await call<Plan>("prepare_clone", {
          parentId: parent.id,
          directory,
          source: local
            ? { kind: "local", id: source?.id }
            : { kind: "url", url: resolvedUrl },
        }),
      );
      setTrusted(false);
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  async function clone() {
    if (!plan || !trusted) return;
    setBusy(true);
    setError("");
    const current = plan;
    setPlan(null);
    setTrusted(false);
    try {
      const snapshot = await call<Snapshot>("clone_repository", {
        planId: current.id,
      });
      await opened(snapshot);
      close();
    } catch (error) {
      setError(String(error));
    } finally {
      setBusy(false);
    }
  }
  return (
    <Dialog
      title="Clone a repository"
      locked={busy}
      onClose={close}
      wide
      className="clone-dialog"
    >
      {error && (
        <p role="alert" className="resolution-error">
          {error}
        </p>
      )}
      {busy && (
        <p role="status" className="clone-progress">
          {plan ? "Preparing your clone…" : "Working…"} Keep this window open.
          Large repositories can take a while; cancellation is not available
          yet.
        </p>
      )}
      {plan ? (
        <section aria-label="Review clone" className="clone-review">
          <p>Create a local copy of this repository.</p>
          <dl>
            <dt>Repository</dt>
            <dd>{plan.source}</dd>
            <dt>Save to</dt>
            <dd>{plan.destination}</dd>
          </dl>
          <p className="small">
            {plan.network
              ? "Your existing Git sign-in or SSH key will be used."
              : "The original repository stays in place."}{" "}
            Only clone sources you trust: Git may run their configured tools.
            Submodules are not included.
          </p>
          <label className="merge-inline-check">
            <input
              type="checkbox"
              checked={trusted}
              disabled={busy}
              onChange={(e) => setTrusted(e.target.checked)}
            />
            I trust this source and want to create this copy.
          </label>
          <div className="dialog-actions">
            <button disabled={busy} onClick={() => setPlan(null)}>
              Back
            </button>
            <button
              className="primary"
              disabled={!trusted || busy}
              onClick={clone}
            >
              Clone and open <ArrowRight size={15} aria-hidden="true" />
            </button>
          </div>
        </section>
      ) : (
        <div className="clone-layout">
          <nav className="clone-providers" aria-label="Repository providers">
            {repositoryProviders.map((p) => {
              const Icon = providerIcons[p.id];
              return (
                <button
                  type="button"
                  key={p.id}
                  aria-pressed={provider === p.id}
                  disabled={busy}
                  onClick={() => {
                    setProvider(p.id);
                    setUrl("");
                    if (!folderEdited) setDirectory("");
                    setError("");
                  }}
                >
                  <Icon size={17} aria-hidden="true" />
                  <span>{p.name}</span>
                </button>
              );
            })}
          </nav>
          <form
            className="clone-form"
            onSubmit={(e) => {
              e.preventDefault();
              void review();
            }}
          >
            <div className="clone-intro">
              <h3>
                {local ? "Copy from your computer" : "Choose your repository"}
              </h3>
              <p>Make a local copy to start working.</p>
            </div>
            {(provider === "custom" || provider === "githubEnterprise") && (
              <label>
                Server
                <input
                  aria-label="Git server hostname"
                  placeholder="git.example.com"
                  value={host}
                  disabled={busy}
                  onChange={(e) => setHost(e.target.value)}
                />
              </label>
            )}
            {local ? (
              <div className="clone-field">
                <span id="clone-source-label">Repository folder</span>
                <button
                  type="button"
                  className="folder-choice"
                  aria-labelledby="clone-source-label"
                  title={source?.path ?? "Choose repository folder"}
                  disabled={busy}
                  onClick={() => pick(true)}
                >
                  <FolderOpen size={18} aria-hidden="true" />
                  <span>{source?.path ?? "Choose repository…"}</span>
                </button>
              </div>
            ) : (
              <label>
                {provider === "url"
                  ? "Repository address"
                  : provider === "azure"
                    ? "Organization / project / repository"
                    : "Repository"}
                <input
                  aria-label="Repository URL"
                  required
                  maxLength={2048}
                  disabled={busy}
                  spellCheck={false}
                  value={url}
                  onChange={(e) => repository(e.target.value)}
                  placeholder={
                    provider === "url"
                      ? "Paste an HTTPS or SSH address"
                      : provider === "azure"
                        ? "organization/project/repository"
                        : "owner/repository"
                  }
                />
              </label>
            )}
            {!local && url && sourceError && (
              <p role="alert" className="resolution-error">
                {sourceError}
              </p>
            )}
            {(provider === "github" || provider === "githubEnterprise") && (
              <GithubRepositories
                key={`${provider}:${host}`}
                host={provider === "github" ? "github.com" : host.trim()}
                disabled={busy}
                selected={repository}
              />
            )}
            <div className="clone-destination">
              <div className="clone-field">
                <span id="clone-parent-label">Save in</span>
                <button
                  type="button"
                  className="folder-choice"
                  aria-labelledby="clone-parent-label"
                  disabled={busy}
                  title={parent?.path ?? "Choose destination folder"}
                  onClick={() => pick(false)}
                >
                  <FolderOpen size={18} aria-hidden="true" />
                  <span>{parent?.path ?? "Choose folder…"}</span>
                </button>
              </div>
              <label>
                Folder name
                <input
                  aria-label="Clone folder name"
                  required
                  maxLength={100}
                  value={directory}
                  disabled={busy}
                  placeholder="repository-name"
                  onChange={(e) => {
                    setFolderEdited(true);
                    setDirectory(e.target.value);
                  }}
                />
              </label>
            </div>
            <p className="clone-hint">
              A new folder will be created here. Existing files are kept.
            </p>
            {!local && provider !== "url" && (
              <details className="clone-options">
                <summary>
                  <GearSix size={15} aria-hidden="true" />
                  Connection options<span>{ssh ? "SSH" : "HTTPS"}</span>
                </summary>
                <label>
                  Connect using
                  <select
                    aria-label="Clone transport"
                    value={ssh ? "ssh" : "https"}
                    disabled={busy}
                    onChange={(e) => setSsh(e.target.value === "ssh")}
                  >
                    <option value="https">HTTPS (standard)</option>
                    <option value="ssh">SSH</option>
                  </select>
                </label>
                {resolvedUrl && <p className="clone-address">{resolvedUrl}</p>}
                <p className="small">
                  Uses your existing Git sign-in or SSH key. Do not include
                  passwords or tokens in addresses.
                </p>
              </details>
            )}
            <div className="dialog-actions">
              <button type="button" disabled={busy} onClick={close}>
                Cancel
              </button>
              <button
                className="primary"
                type="submit"
                disabled={
                  busy ||
                  !parent ||
                  !directory.trim() ||
                  (local ? !source : !resolvedUrl.trim() || !!sourceError)
                }
              >
                Review and clone <ArrowRight size={15} aria-hidden="true" />
              </button>
            </div>
          </form>
        </div>
      )}
    </Dialog>
  );
}
