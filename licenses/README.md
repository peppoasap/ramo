# Supplemental upstream license texts

Some published crates omit their repository's top-level license file. The notice
generator uses these upstream texts for those crates; no upstream code is vendored.

- `objc2-MIT.txt`: https://raw.githubusercontent.com/madsmtm/objc2/master/LICENSE-MIT.txt — objc2, block2 and dispatch2.
- `alloc-BSD-3-Clause.txt`: https://raw.githubusercontent.com/dropbox/rust-alloc-no-stdlib/master/LICENSE — alloc-stdlib.
- `MPL-2.0.txt`: https://www.mozilla.org/media/MPL/2.0/index.815ca599c9df.txt — selectors.

Retrieved 2026-11-03. Exact dependency versions and source archives are listed in
the generated `THIRD_PARTY_NOTICES.txt`. Generate notices on each target platform.
