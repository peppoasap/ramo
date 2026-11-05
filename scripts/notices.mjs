import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";

// Include a conservative superset: native target dependencies and runtime JS
// dependencies. Never copy machine-specific paths into distribution files.
const target =
  process.env.TAURI_ENV_TARGET_TRIPLE ||
  execFileSync("rustc", ["-vV"], { encoding: "utf8" }).match(
    /^host: (.+)$/m,
  )[1];
const metadata = JSON.parse(
  execFileSync(
    "cargo",
    [
      "metadata",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--format-version",
      "1",
      "--locked",
      "--offline",
      "--filter-platform",
      target,
    ],
    { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  ),
);
const sections = [];
const missing = [];
function licenseTexts(dir, extra) {
  const names = readdirSync(dir).filter((n) =>
    /^(licen[sc]e|copying|notice)([-_.]|$)/i.test(n),
  );
  if (extra && existsSync(join(dir, extra)) && !names.includes(extra))
    names.push(extra);
  return names.flatMap((n) => {
    try {
      return [`--- ${n} ---\n${readFileSync(join(dir, n), "utf8")}`];
    } catch {
      return [];
    }
  });
}
for (const p of metadata.packages
  .filter((p) => p.source)
  .sort((a, b) => a.name.localeCompare(b.name))) {
  let texts = licenseTexts(dirname(p.manifest_path), p.license_file);
  if (!texts.length) {
    const fallback =
      p.name.startsWith("objc2") || ["block2", "dispatch2"].includes(p.name)
        ? "objc2-MIT.txt"
        : p.name === "alloc-stdlib"
          ? "alloc-BSD-3-Clause.txt"
          : p.name === "selectors"
            ? "MPL-2.0.txt"
            : null;
    if (fallback) texts = [readFileSync(join("licenses", fallback), "utf8")];
  }
  if (!texts.length) missing.push(`${p.name}@${p.version}`);
  sections.push(
    `${p.name} ${p.version}\nLicense: ${p.license || "see text"}\nSource: https://crates.io/api/v1/crates/${p.name}/${p.version}/download\n${texts.join("\n\n")}`,
  );
}
const lock = JSON.parse(readFileSync("package-lock.json", "utf8"));
const dirs = readdirSync("node_modules")
  .filter((n) => !n.startsWith("."))
  .flatMap((n) =>
    n.startsWith("@")
      ? readdirSync(join("node_modules", n)).map((s) =>
          join("node_modules", n, s),
        )
      : [join("node_modules", n)],
  );
for (const dir of dirs.sort()) {
  if (lock.packages[dir.replaceAll("\\", "/")]?.dev) continue;
  if (!existsSync(join(dir, "package.json"))) continue;
  const p = JSON.parse(readFileSync(join(dir, "package.json"), "utf8"));
  const texts = licenseTexts(dir);
  if (!texts.length) missing.push(`${p.name}@${p.version}`);
  sections.push(
    `${p.name} ${p.version}\nLicense: ${typeof p.license === "string" ? p.license : JSON.stringify(p.license)}\nSource: https://www.npmjs.com/package/${p.name}/v/${p.version}\n${texts.join("\n\n")}`,
  );
}
if (missing.length)
  throw new Error(`Missing license texts: ${missing.join(", ")}`);
writeFileSync(
  "THIRD_PARTY_NOTICES.txt",
  `Ramo third-party notices\nTarget: ${target}\n\nOriginal Ramo code is Apache-2.0. Dependencies retain their own licenses.\nThis inventory includes native build dependencies as a conservative superset; JS runtime dependencies only.\nUnmodified MPL-covered dependency source is available at each exact source URL\nbelow and identified by Cargo.lock; Ramo does not relicense that source.\n\n${sections.join("\n\n" + "=".repeat(72) + "\n\n")}\n`,
);
console.log(`Collected ${sections.length} dependency notices for ${target}.`);
