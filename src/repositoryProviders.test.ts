import { expect, it } from "vitest";
import { providerUrl, suggestedDirectory } from "./repositoryProviders";
it("builds ordinary hosted and enterprise clone addresses", () => {
  expect(providerUrl("github", "meadow/app", "", false)).toBe(
    "https://github.com/meadow/app.git",
  );
  expect(providerUrl("gitlab", "group/subgroup/app", "", true)).toBe(
    "git@gitlab.com:group/subgroup/app.git",
  );
  expect(
    providerUrl("githubEnterprise", "meadow/app", "git.example.invalid", true),
  ).toBe("git@git.example.invalid:meadow/app.git");
  expect(providerUrl("bitbucket", "team/app.git", "", false)).toBe(
    "https://bitbucket.org/team/app.git",
  );
  expect(providerUrl("azure", "org/project/app", "", false)).toBe(
    "https://dev.azure.com/org/project/_git/app",
  );
  expect(providerUrl("azure", "org/project/app", "", true)).toBe(
    "git@ssh.dev.azure.com:v3/org/project/app",
  );
});
it("refuses path traversal, credentials and option-like shorthand", () => {
  for (const path of [
    "../app",
    "team/../app",
    "-team/app",
    "team/-app",
    "team/app?token=secret",
    "https://user:secret@host/repo",
  ])
    expect(() => providerUrl("github", path, "", false)).toThrow();
  for (const host of [
    "https://host",
    "user@host",
    "host/path",
    "host:443",
    "--option",
  ])
    expect(() => providerUrl("custom", "team/app", host, false)).toThrow();
  expect(() => providerUrl("azure", "org/app", "", false)).toThrow();
});
it("suggests folder names from paths without inventing unsafe filenames", () => {
  expect(suggestedDirectory("git@github.com:meadow/app.git")).toBe("app");
  expect(suggestedDirectory("https://git.example.invalid/group/app/")).toBe(
    "app",
  );
  expect(suggestedDirectory("../..")).toBe("");
  expect(suggestedDirectory("https://host/")).toBe("");
});
