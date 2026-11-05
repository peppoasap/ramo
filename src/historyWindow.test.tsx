import { describe, it, expect } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { History } from "./components/History";
import {
  historyWindow,
  historyIndices,
  historyScrollTarget,
  historyNavigationIndex,
  HISTORY_ROW_HEIGHT,
} from "./historyWindow";
import type { Commit } from "./domain";
function commits(count: number): Commit[] {
  return Array.from({ length: count }, (_, index) => ({
    oid: String(index).padStart(40, "0"),
    parents: index + 1 < count ? [String(index + 1).padStart(40, "0")] : [],
    subject: `Synthetic change ${index}`,
    author: "Synthetic",
    date: "2026-01-01T00:00:00Z",
    refs: "",
  }));
}
describe("Virtual commit history", () => {
  it("bounds rows independently of the loaded history size and preserves absolute positions", () => {
    for (const count of [0, 1, 300, 5000, 100000])
      for (const top of [0, 48, 9999, count * 48 + 500])
        for (const height of [1, 480, 801]) {
          const window = historyWindow(count, top, height);
          const indices = historyIndices(window, count, []);
          expect(indices.length).toBeLessThanOrEqual(
            Math.ceil(height / 48) + 17,
          );
          expect(window.totalHeight).toBe(count * HISTORY_ROW_HEIGHT);
          expect(indices.every((i) => i >= 0 && i < count)).toBe(true);
          expect(new Set(indices).size).toBe(indices.length);
          if (count) expect(indices.length).toBeGreaterThan(0);
        }
    const window = historyWindow(5000, 48000, 480);
    expect(window).toEqual({ start: 992, end: 1018, totalHeight: 240000 });
  });
  it("retains focused/pending rows without filling all intervening history", () => {
    const window = historyWindow(5000, 48000, 480);
    const indices = historyIndices(window, 5000, [
      0,
      4999,
      4999,
      -1,
      6000,
      null,
    ]);
    expect(indices.length).toBe(28);
    expect(indices[0]).toBe(0);
    expect(indices.at(-1)).toBe(4999);
    // A refresh can replace a paginated list before the next layout measurement.
    expect(historyIndices(window, 10, [0, 999])).toEqual([0]);
  });
  it("reveals only when needed and keyboard movement crosses unmounted windows", () => {
    expect(historyScrollTarget(8, 240, 480)).toBe(240);
    expect(historyScrollTarget(0, 240, 480)).toBe(0);
    expect(historyScrollTarget(4999, 240, 480)).toBe(239520);
    expect(historyNavigationIndex("ArrowDown", 299, 5000, 480)).toBe(300);
    expect(historyNavigationIndex("PageDown", 299, 5000, 480)).toBe(309);
    expect(historyNavigationIndex("End", 0, 5000, 480)).toBe(4999);
    expect(historyNavigationIndex("Home", 4999, 5000, 480)).toBe(0);
    expect(historyNavigationIndex("ArrowDown", 4999, 5000, 480)).toBe(4999);
    expect(historyNavigationIndex("ArrowUp", 0, 5000, 480)).toBe(0);
    expect(historyNavigationIndex("ArrowDown", 0, 0, 480)).toBeNull();
    expect(historyNavigationIndex("ContextMenu", 1, 5000, 480)).toBeNull();
  });
  it("renders a bounded actual component with branch controls, logical positions and full graph topology", () => {
    const data = commits(5000);
    data[0].parents.push(data[4999].oid);
    data[20].subject = "Needle match";
    const html = renderToStaticMarkup(
      <History
        commits={data}
        branches={[
          { name: "work/synthetic", oid: data[10].oid, remote: false },
        ]}
        currentBranch="work/synthetic"
        selected={data[0].oid}
        onSelect={() => {}}
        filter="Needle match"
        onFilter={() => {}}
        onBranchContext={() => {}}
        onCommitContext={() => {}}
        onMore={() => {}}
        hasMore={true}
        busy={false}
      />,
    );
    expect((html.match(/data-history-index=/g) || []).length).toBe(21);
    expect(html).toContain("height:240000px");
    expect(html).toContain("top:960px");
    expect(html).toContain('aria-setsize="5000"');
    expect(html).toContain('aria-posinset="21"');
    expect(html).toContain("Branch: work/synthetic");
    expect(html.replace(/<!--.*?-->/g, "")).toContain(
      "1 matches in loaded commits",
    );
    expect(html).toContain("Load older commits");
    expect(html).toContain("M 40 0 C 40 40");
    expect((html.match(/dimmed/g) || []).length).toBe(20);
    expect(html).not.toContain('data-history-index="4999"');
  });
});
