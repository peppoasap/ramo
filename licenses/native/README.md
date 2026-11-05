# Native UI dependency notices

The native experiment has its own locked Cargo graph. Run
`python3 scripts/native-notices.py` after fetching Cargo sources. The generator
fails on missing texts instead of emitting an incomplete inventory. Build outputs
are target-specific and ignored; original licenses remain their own.

`manifest.json` maps exact crate versions to supplemental upstream texts, their
source revision and SHA-256. Revisions come from the published `.cargo_vcs_info`
files. Workspace crates often omit the workspace-level license from their archive.
GPU allocator terms live under the upstream `license/` folder. Existing objc2 /
block2 / dispatch2 terms use Ramo's existing supplemental
MIT notice in the parent directory. No dependency source is vendored here.

Four older crate versions need a documented canonical-term fallback:

- block0.1.6, dispatch0.2.0 and malloc_buf0.0.6 declare MIT and Steven Sheldon as
  author in their published manifest, but their archive and corresponding upstream
  revision have no license file. The inventory explicitly identifies that fact,
  adds author attribution and supplies the canonical SPDX MIT permission terms.
  This is not presented as a recovered upstream license file.
- hexf-parse0.2.1 declares CC0-1.0 without a bundled text; its entry includes the
  canonical SPDX CC0 terms and says where those terms came from.

The standard texts' `.source.json` files retain retrieval URL, date and hash.
Phosphor icon-font attribution is included separately from the egui wrapper's
license, following its README. Egui's bundled default fonts also retain their
own Hack, Ubuntu, Noto and emoji font texts; their Rust crate license does not
replace those terms. The conservative inventory may include unused build inputs;
it is not a legal review or a statement of publisher signing/runtime acceptance.
