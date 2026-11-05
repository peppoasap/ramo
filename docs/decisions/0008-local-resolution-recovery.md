# 0008 — Local recovery of unsaved resolution buffers

Status: accepted, local implementation (2026-11-03).

Persist edited result buffers synchronously in the app's local WebView storage. Use a canonical worktree identity plus a SHA-256 filename digest as the key. Each versioned record contains the reviewed Git fingerprint, result text, deletion intent and timestamp. Do not persist provider inputs, original source panes, prompts, accounts or credentials. Known sensitive credential filenames disable automatic recovery and show that limitation.

A matching fingerprint can restore the local buffer automatically; this never writes Git. A changed fingerprint requires reviewing the saved result and explicitly restoring or discarding it before Save becomes available. Recovery failures keep the in-memory result and show a warning instead of claiming it was saved.

Compare the stored record before updating or clearing it. A detected newer editor's record must be retained. This is optimistic protection, not an atomic cross-window transaction. Clear an owned copy after successful save-and-stage or explicit discard. Keep draft and close is a separate option. Storage failures and the 1 MiB UTF-8 result limit never cause truncation or silent replacement of the previous copy.

Recovery is on this device only and depends on WebView storage retention. Normal restart recovery is a separate native acceptance gate; neither local storage nor tests establish survival of every OS crash or disk failure. Source-line selections are transient until applied to the result.
