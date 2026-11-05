# Contributing

Ramo has a functional local desktop implementation. Read README.md for prerequisites and build/test commands; STATUS.md distinguishes verified behavior from remaining platform and AI work.

## Resume the project

1. Read README.md, STATUS.md, AGENTS.md and the spec for the area you are changing.
2. Choose one small task from STATUS.md or ROADMAP.md.
3. State the observable result and how it will be checked.
4. Implement the smallest complete slice; preserve other work.
5. Run checks relevant to the change and record the actual results.
6. Update STATUS.md so another contributor can continue without this conversation.

Use English for public documentation, code, identifiers and issues. Initial UI terminology is English. Centralizing strings for localization remains a follow-up task.

## Changes and evidence

A contribution should explain the problem, resulting behavior, checks performed and remaining limits. Include screenshots for visible changes and temporary Git fixture scenarios for command or graph changes.

Do not add tests that merely repeat implementation details. Meaningful examples include merge topology, Unicode filenames, staged versus unstaged changes, concurrent worktrees and stale AI responses.

A live provider connection test must be deliberate, use synthetic data, and record the tested provider/version. CI must not require a subscription, account or paid inference.

Use the Node version range declared in package.json (verified locally with Node 22.13.0). Run `npm ci`, `npm test`, `npm run build` and `npm run audit:dependencies` when changing the JavaScript toolchain. The audit command checks known registry advisories; an empty report does not replace code or release review.

Dependencies need a reason, a compatible license and required attribution. Lockfiles and reproducible setup commands belong in the implementation slice that introduces the toolchain.

## Project conduct

Be specific, respectful and evidence-driven. Review code and behavior rather than the author. Do not include private repository content or credentials in issues, screenshots or logs.

## Licensing

Contributions intentionally submitted for inclusion are under Apache-2.0, subject to the license's contribution provisions. Preserve existing copyright and attribution notices. The repository is maintained at https://github.com/peppoasap/ramo. Use issues for non-sensitive bugs and proposals, pull requests for reviewable changes, and SECURITY.md for sensitive findings. No formal support SLA or broader governance model is established yet.

