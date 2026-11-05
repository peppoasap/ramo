# Windows build and acceptance

Windows CI produces installers only after its build and tests pass. Check the
successful workflow run or a published prerelease for an actual setup.exe; a
configured job is not proof of a completed build. GUI installation/runtime
acceptance is still outstanding. The source ZIP is a developer build kit, not a
double-click installer for end users.

Use Windows x64 with Node22.12+ (22.x),24.x or26+, npm, Git for Windows on PATH,
Rust1.95+ with the MSVC toolchain, Visual Studio C++ build tools and WebView2.
See the official [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).
The script does not install prerequisites, start logins, sign or publish anything.

Extract the source ZIP to a local folder. From PowerShell in that folder run:

```powershell
powershell -NoProfile -File scripts/package-windows-local.ps1
```

The script installs locked project dependencies via npm ci, runs version checks,
frontend and real-Git backend tests, builds an NSIS installer, then copies the
version-matching setup.exe and SHA256SUMS to releases/local/<version>/windows/.
Existing output is refused. Failed checks stop the process; do not distribute a
partial build. Internet access is needed for dependencies and build tooling.
No AI inference or remote Git operations are part of this packaging script.

Alternatively, in an authorized GitHub repository run the existing **Build and
check Ramo** workflow and download its ramo-Windows artifact. It performs the
Windows build and tests on windows-latest; it does not publish a GitHub release.
A configured workflow is not evidence that a Windows build has passed.

Before sharing a built installer, on a disposable Windows test machine:

1. Install and launch; verify the version and availability of installed Git.
2. Use a synthetic repository to open history, create/switch a branch, stage and
   commit; inspect/pop/stash tracked and untracked text.
3. Exercise a synthetic merge conflict through save/stage/complete and abort.
4. Close/reopen and confirm draft recovery; uninstall and reinstall.
5. Record actual results and installer SHA256. AI providers, SSH/HTTPS credentials
   and CLI wrappers require separate checks. Never use personal repositories for
   mutation acceptance.

Any CI installer is unsigned. Public signing, SmartScreen reputation and runtime
acceptance remain outstanding. Do not disable Windows security protections.
