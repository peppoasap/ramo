// Uses the existing Phosphor icon asset; no copied product branding.
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { GitBranch } from "@phosphor-icons/react";
import { Resvg } from "@resvg/resvg-js";
import { mkdirSync, writeFileSync } from "node:fs";
const svg = renderToStaticMarkup(
  React.createElement(GitBranch, {
    size: 1024,
    weight: "regular",
    color: "#7bd8ac",
  }),
);
mkdirSync("assets", { recursive: true });
writeFileSync("assets/icon.svg", svg);
writeFileSync(
  "assets/icon.png",
  new Resvg(svg, {
    background: "#171b20",
    fitTo: { mode: "width", value: 1024 },
  })
    .render()
    .asPng(),
);
