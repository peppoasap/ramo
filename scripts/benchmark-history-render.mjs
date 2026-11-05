// Actual History component, static server rendering only; no UI/frame-rate claim.
import { createServer } from "vite";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { performance } from "node:perf_hooks";
const server = await createServer({
  server: { middlewareMode: true, ws: false, hmr: false, watch: null },
  optimizeDeps: { noDiscovery: true, include: [] },
  appType: "custom",
});
try {
  const { History } = await server.ssrLoadModule("/src/components/History.tsx");
  const measurements = [];
  for (const count of [300, 5000, 50000]) {
    const commits = Array.from({ length: count }, (_, i) => ({
      oid: i.toString(16).padStart(40, "0"),
      parents: i + 1 < count ? [(i + 1).toString(16).padStart(40, "0")] : [],
      author: "Synthetic Contributor",
      subject: `Synthetic change ${i}`,
      date: "2026-01-01T00:00:00Z",
      refs: "",
    }));
    const props = {
      commits,
      branches: [],
      currentBranch: "main",
      selected: commits[0].oid,
      filter: "",
      onSelect: () => {},
      onFilter: () => {},
      onBranchContext: () => {},
      onCommitContext: () => {},
      onMore: () => {},
      hasMore: false,
      busy: false,
    };
    let html = "";
    const times = [];
    for (let pass = 0; pass < 8; pass++) {
      const start = performance.now();
      html = renderToStaticMarkup(React.createElement(History, props));
      const elapsed = performance.now() - start;
      if (pass) times.push(Number(elapsed.toFixed(2)));
    }
    const renderedRows = (html.match(/data-history-index=/g) || []).length;
    if (renderedRows > 21)
      throw new Error(`Unbounded history rows: ${renderedRows}`);
    measurements.push({
      loadedCommits: count,
      renderedRows,
      medianStaticRenderMs: [...times].sort((a, b) => a - b)[3],
      samplesMs: times,
    });
  }
  console.log(
    JSON.stringify(
      {
        date: new Date().toISOString(),
        node: process.version,
        scope:
          "Actual History component static server render with 600px initial viewport. Includes complete graph layout. Synthetic data; no Git calls, browser timing, frame-rate or baseline speed-up claim.",
        measurements,
      },
      null,
      2,
    ),
  );
} finally {
  await server.close();
}
