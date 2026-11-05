import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
const read = (path) =>
  readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const pkg = JSON.parse(read("package.json"));
assert.match(pkg.version, /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/);
assert.equal(pkg.license, "Apache-2.0");
const lock = JSON.parse(read("package-lock.json"));
assert.equal(lock.version, pkg.version);
assert.equal(lock.packages[""].version, pkg.version);
assert.equal(
  JSON.parse(read("src-tauri/tauri.conf.json")).version,
  pkg.version,
);
const cargo = read("src-tauri/Cargo.toml");
assert.equal(cargo.match(/^version = "([^"]+)"/m)?.[1], pkg.version);
assert.match(cargo, /^license = "Apache-2.0"/m);
assert.equal(
  read("src-tauri/Cargo.lock").match(/name = "ramo"\nversion = "([^"]+)"/)?.[1],
  pkg.version,
);
assert.ok(
  read("CHANGELOG.md").includes(`## [${pkg.version}]`),
  "Missing changelog entry",
);
assert.ok(read("LICENSE").includes("Apache License"));
for (const file of [
  "NOTICE",
  "SECURITY.md",
  "docs/RELEASING.md",
  "docs/BETA-CHECKLIST.md",
])
  assert.ok(read(file).trim());
console.log(
  `Release metadata consistent: ${pkg.version} (no publication performed)`,
);
