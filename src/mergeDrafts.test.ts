import { expect, it } from "vitest";
import {
  allowsMergeRecovery,
  mergeDraftKey,
  MergeDraftSession,
} from "./mergeDrafts";
const repo = "a".repeat(64),
  otherRepo = "b".repeat(64),
  oldState = "c".repeat(64),
  newState = "d".repeat(64);
function storage() {
  const values = new Map<string, string>();
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
    removeItem: (key: string) => {
      values.delete(key);
    },
  };
}
it("scopes recovery by canonical worktree and filename without exposing the filename in keys", async () => {
  const key = await mergeDraftKey(repo, "src/è.ts");
  expect(key).not.toContain("è");
  expect(key).not.toBe(await mergeDraftKey(otherRepo, "src/è.ts"));
  expect(key).not.toBe(await mergeDraftKey(repo, "src/other.ts"));
  expect(key).toBe(await mergeDraftKey(repo, "src/è.ts"));
});
it("restores exact Unicode, CRLF, empty results and deletion intent", () => {
  const disk = storage();
  for (const [text, remove] of [
    ["è 🌱\r\n\r\n", false],
    ["", false],
    ["", true],
  ] as const) {
    const session = new MergeDraftSession(disk, "key");
    session.load();
    session.save(oldState, text, remove);
    const reopened = new MergeDraftSession(disk, "key").load();
    expect(reopened).toMatchObject({
      fingerprint: oldState,
      content: text,
      remove,
    });
  }
});
it("retains old review identity rather than silently rebasing a recovered result", () => {
  const disk = storage(),
    session = new MergeDraftSession(disk, "key");
  session.load();
  session.save(oldState, "Old reviewed result", false);
  const draft = new MergeDraftSession(disk, "key").load();
  expect(draft?.fingerprint).not.toBe(newState);
  expect(draft?.content).toBe("Old reviewed result");
});
it("does not overwrite or clear a newer record detected from another editor", () => {
  const disk = storage(),
    first = new MergeDraftSession(disk, "key"),
    second = new MergeDraftSession(disk, "key");
  first.load();
  first.save(oldState, "first", false);
  second.load();
  second.save(oldState, "newer", false);
  expect(() => first.save(oldState, "stale", false)).toThrow("Another window");
  expect(() => first.clear()).toThrow("Another window");
  expect(new MergeDraftSession(disk, "key").load()?.content).toBe("newer");
});
it("clears only this recovery entry after success or explicit discard", () => {
  const disk = storage(),
    first = new MergeDraftSession(disk, "one"),
    other = new MergeDraftSession(disk, "two");
  first.load();
  other.load();
  first.save(oldState, "resolved", false);
  other.save(oldState, "keep", false);
  first.clear();
  expect(new MergeDraftSession(disk, "one").load()).toBeNull();
  expect(other.load()?.content).toBe("keep");
});
it("quota errors preserve the last saved record and reject oversized UTF-8 without truncation", () => {
  const disk = storage(),
    original = new MergeDraftSession(disk, "key");
  original.load();
  original.save(oldState, "keep", false);
  const full = new MergeDraftSession(
    {
      ...disk,
      setItem() {
        throw new Error("Quota exceeded");
      },
    },
    "key",
  );
  full.load();
  expect(() => full.save(oldState, "new", false)).toThrow("Quota");
  expect(() => original.save(oldState, "🌱".repeat(262145), false)).toThrow(
    "oversized",
  );
  expect(new MergeDraftSession(disk, "key").load()?.content).toBe("keep");
});
it("leaves corrupt or unsupported records untouched", () => {
  const disk = storage();
  disk.setItem("key", '{"version":99}');
  const session = new MergeDraftSession(disk, "key");
  expect(() => session.load()).toThrow();
  expect(() => session.clear()).toThrow("not ready");
  expect(disk.getItem("key")).toBe('{"version":99}');
});
it("disables automatic recovery for sensitive credential filenames", () => {
  for (const path of [
    ".env",
    "config/.env.local",
    "keys/private.pem",
    "credentials.json",
    "id_ed25519",
  ])
    expect(allowsMergeRecovery(path)).toBe(false);
  expect(allowsMergeRecovery("src/search.ts")).toBe(true);
});
