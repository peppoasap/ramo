# ADR 0002: functional desktop implementation

Date: 2026-11-03. Status: accepted for local implementation.

The user clarified that the deliverable must be functional, fast, integrable, multi-platform and installable, rather than a prototype. This supersedes ADR 0001's demo-first roadmap and proposed stack status.

Use Tauri 2 with React/TypeScript, a small Rust backend and installed Git. The local macOS build and disposable Git fixtures validate this choice enough to continue. Keep one package and explicit UI/Git/process/AI boundaries. No browser HTTP backend is needed.

Provider integration uses user-installed official CLIs or loopback Ollama. Ramo does not import credentials. CLI discovery and an authenticated account do not establish inference eligibility. The initial Codex 0.139.0 live test failed. ADR 0003 records the later successful 0.166.0 contract/live check. A dedicated official OSS login remains a possible future route, not an implemented feature.

All three desktop platforms remain required product targets. Only macOS Apple Silicon has local build/runtime evidence. Packaging configuration and CI do not establish Windows/Linux support. No public release is authorized by this decision.
