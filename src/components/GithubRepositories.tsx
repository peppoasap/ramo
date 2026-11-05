import { useState } from "react";
import { MagnifyingGlass, ArrowClockwise } from "@phosphor-icons/react";
import { call } from "../bridge";
interface HostedRepository {
  name: string;
  https: string;
  ssh: string;
  private: boolean;
}
export function GithubRepositories({
  host,
  disabled,
  selected,
}: {
  host: string;
  disabled: boolean;
  selected: (name: string) => void;
}) {
  const [organization, setOrganization] = useState("");
  const [repositories, setRepositories] = useState<HostedRepository[]>([]);
  const [filter, setFilter] = useState("");
  const [loaded, setLoaded] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [page, setPage] = useState(0);
  const [more, setMore] = useState(false);
  async function load(next: number) {
    setBusy(true);
    setError("");
    if (next === 1) {
      setRepositories([]);
      setLoaded(false);
    }
    try {
      const rows = await call<HostedRepository[]>("github_repositories", {
        host,
        organization: organization.trim(),
        page: next,
      });
      setRepositories((previous) =>
        next === 1
          ? rows
          : [
              ...previous,
              ...rows.filter(
                (row) => !previous.some((old) => old.name === row.name),
              ),
            ],
      );
      setPage(next);
      setMore(rows.length === 100 && next < 20);
      setLoaded(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <details className="hosted-catalog" aria-label="GitHub repository catalog">
      <summary>
        <MagnifyingGlass size={15} aria-hidden="true" />
        Browse GitHub repositories
      </summary>
      <div className="catalog-body">
        <div className="catalog-controls">
          <input
            aria-label="GitHub organization"
            placeholder="Organization (optional)"
            value={organization}
            disabled={busy || disabled}
            onChange={(e) => {
              setOrganization(e.target.value);
              setLoaded(false);
              setRepositories([]);
              setMore(false);
            }}
          />
          <button
            type="button"
            disabled={busy || disabled || !host}
            onClick={() => load(1)}
          >
            <ArrowClockwise
              size={15}
              aria-hidden="true"
              className={busy ? "catalog-loading" : undefined}
            />
            {busy ? "Loading…" : "Load repositories"}
          </button>
        </div>
        <p className="small">
          Uses the account signed in to GitHub CLI on {host || "your server"}.
          Leave organization empty to browse all repositories you can access.
        </p>
        {error && (
          <p role="alert" className="resolution-error">
            {error}
          </p>
        )}
        {loaded && (
          <>
            <input
              aria-label="Filter GitHub repositories"
              placeholder="Find a repository…"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              disabled={busy || disabled}
            />
            <div
              className="hosted-results"
              aria-label="Available GitHub repositories"
            >
              {repositories
                .filter((r) =>
                  r.name.toLowerCase().includes(filter.toLowerCase()),
                )
                .map((r) => (
                  <button
                    type="button"
                    key={r.name}
                    disabled={busy || disabled}
                    onClick={() => selected(r.name)}
                  >
                    <span>{r.name}</span>
                    <small>{r.private ? "Private" : "Public"}</small>
                  </button>
                ))}
            </div>
            {!repositories.length && (
              <p className="small">
                No repositories returned for this account or organization.
              </p>
            )}
            <p className="small">
              Filtering {repositories.length} loaded repositories. Catalog
              access and Git clone authentication are separate.
            </p>
            {more && (
              <button
                type="button"
                disabled={busy || disabled}
                onClick={() => load(page + 1)}
              >
                Load next 100
              </button>
            )}
          </>
        )}
      </div>
    </details>
  );
}
