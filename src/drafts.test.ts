import { describe, expect, it } from "vitest";
import { loadDraft, saveDraft, clearCommittedDraft } from "./drafts";

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
const main = "a".repeat(64);
const worktree = "b".repeat(64);
describe("commit draft recovery", () => {
  it("restores exact multiline text and keeps worktrees separate", () => {
    const disk = storage();
    const text = "Preserve normalization\n\nUnicode: è 🌱\n";
    saveDraft(disk, main, text);
    saveDraft(disk, worktree, "Independent worktree draft");
    expect(loadDraft(disk, main)).toBe(text);
    expect(loadDraft(disk, worktree)).toBe("Independent worktree draft");
  });
  it("removes a successfully committed draft without clearing another worktree", () => {
    const disk = storage();
    saveDraft(disk, main, "Committed");
    saveDraft(disk, worktree, "Still editing");
    clearCommittedDraft(disk, main, "Committed");
    expect(loadDraft(disk, main)).toBe("");
    expect(loadDraft(disk, worktree)).toBe("Still editing");
  });
  it("does not erase a newer saved draft when an earlier commit finishes", () => {
    const disk = storage();
    saveDraft(disk, main, "Submitted earlier");
    saveDraft(disk, main, "Next commit in progress");
    clearCommittedDraft(disk, main, "Submitted earlier");
    expect(loadDraft(disk, main)).toBe("Next commit in progress");
  });
  it("reports storage failure and preserves the previous saved record", () => {
    const disk = storage();
    saveDraft(disk, main, "Previous draft");
    const full = {
      ...disk,
      setItem() {
        throw new Error("Quota exceeded");
      },
    };
    expect(() => saveDraft(full, main, "New text")).toThrow("Quota exceeded");
    expect(loadDraft(disk, main)).toBe("Previous draft");
  });
  it("rejects oversized text without truncating or overwriting prior work", () => {
    const disk = storage();
    saveDraft(disk, main, "Keep this");
    expect(() => saveDraft(disk, main, "x".repeat(32769))).toThrow();
    expect(loadDraft(disk, main)).toBe("Keep this");
  });
});
