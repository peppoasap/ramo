import { renderToStaticMarkup } from "react-dom/server";
import { expect, it, vi } from "vitest";
import type { Snapshot } from "../domain";
import { OperationGuide } from "./OperationGuide";
function guide(operation: string | null, conflicts = true) {
  const snapshot = {
    operation,
    repo: { name: "Synthetic repository" },
    files: conflicts
      ? [{ path: "README.md", index: "U", worktree: "U", original: null }]
      : [],
  } as Snapshot;
  return renderToStaticMarkup(
    <OperationGuide
      snapshot={snapshot}
      busy={false}
      review={vi.fn()}
      confirm={vi.fn()}
    />,
  );
}
it("guides unmerged files with no active operation through ordinary review and commit", () => {
  const html = guide(null);
  expect(html).toContain("create a normal commit");
  expect(html).toContain("Any retained stash stays saved");
  expect(html).not.toContain("using Git outside Ramo");
  expect(html).not.toContain("Abort merge");
  expect(html).not.toContain("Continue cherry-pick");
});
it("does not offer an ordinary commit as a substitute for rebase or an unknown sequence", () => {
  for (const operation of ["Rebase in progress", "Sequence in progress"]) {
    const html = guide(operation);
    expect(html).toContain("using Git outside Ramo");
    expect(html).not.toContain("create a normal commit");
  }
});
it("preserves the dedicated merge and sequencer continuation controls", () => {
  expect(guide("Merge in progress")).toContain("Complete merge");
  expect(guide("Merge in progress")).toContain("Abort merge");
  expect(guide("Cherry-pick in progress")).toContain("Continue cherry-pick");
  expect(guide("Revert in progress")).toContain("Continue revert");
});
it("removes the operation guide when unmerged files are resolved and no sequence is active", () => {
  expect(guide(null, false)).toBe("");
});
