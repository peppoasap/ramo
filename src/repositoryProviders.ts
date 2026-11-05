export type ProviderId =
  | "url"
  | "github"
  | "githubEnterprise"
  | "gitlab"
  | "bitbucket"
  | "azure"
  | "custom"
  | "local";
export const repositoryProviders: {
  id: ProviderId;
  name: string;
  host?: string;
}[] = [
  { id: "url", name: "Paste a link" },
  { id: "github", name: "GitHub", host: "github.com" },
  { id: "githubEnterprise", name: "GitHub Enterprise" },
  { id: "gitlab", name: "GitLab", host: "gitlab.com" },
  { id: "bitbucket", name: "Bitbucket", host: "bitbucket.org" },
  { id: "azure", name: "Azure DevOps", host: "dev.azure.com" },
  { id: "custom", name: "Other server" },
  { id: "local", name: "On this computer" },
];
export function suggestedDirectory(value: string): string {
  let path = value.trim();
  if (path.includes("://")) {
    try {
      path = new URL(path).pathname;
    } catch {
      return "";
    }
  }
  const name =
    path
      .replace(/\/+$/, "")
      .split(/[/:]/)
      .pop()
      ?.replace(/\.git$/, "") ?? "";
  return /^[A-Za-z0-9][A-Za-z0-9._-]{0,99}$/.test(name) ? name : "";
}
export function providerUrl(
  id: ProviderId,
  repository: string,
  host: string,
  ssh: boolean,
): string {
  if (id === "url") return repository.trim();
  if (id === "local") return "";
  const selected = repositoryProviders.find((p) => p.id === id);
  const server =
    id === "custom" || id === "githubEnterprise" ? host.trim() : selected?.host;
  if (
    !server ||
    !/^[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?$/i.test(server) ||
    server.includes("..")
  )
    throw new Error("Enter a hostname without a scheme, path or credentials.");
  const path = repository.trim().replace(/\.git$/, "");
  if (
    !/^[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)+$/.test(path) ||
    path.split("/").some((p) => p === "." || p === ".." || p.startsWith("-"))
  )
    throw new Error("Enter the repository path, for example team/project.");
  if (id === "azure") {
    const parts = path.split("/");
    if (parts.length !== 3)
      throw new Error("Use organization/project/repository for Azure DevOps.");
    return ssh
      ? `git@ssh.dev.azure.com:v3/${path}`
      : `https://dev.azure.com/${parts[0]}/${parts[1]}/_git/${parts[2]}`;
  }
  return ssh ? `git@${server}:${path}.git` : `https://${server}/${path}.git`;
}
