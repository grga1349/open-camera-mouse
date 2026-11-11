# Migration plan — preserve old app, establish clean Rust rewrite

## Requested intent

Create a fresh Git branch, tentatively `rewrite/rust` (the user described it as “open camera mouse rust rewrite”). Move old working implementation under `legacy/` to keep it as a regression and research reference. Old `CLAUDE.md`, `AGENTS.md` and AI files should no longer govern the new project; **archive first, don't delete blindly**.

## Safety and sequencing

1. `git status --short`, `git branch --show-current`, find tracked vs generated files, gitignore, license, submodules, symlinks and uncommitted work.
2. Preserve uncommitted changes with an explicit safe path; no `git clean -fdx`, no blind mass `git mv`, no unreviewed destructive script.
3. Create/switch to `rewrite/rust` only after confirming safe state. Keep default branch untouched.
4. Classify files into old app source/build/config/docs, still-relevant root metadata, generated/untracked/secrets and staging dump.
5. Move tracked old app contents safely to `legacy/` (possibly `legacy/ai-config/` for historical AI instructions). Do **not** move `.git/`, the staging `dump/` or new root docs. Preserve license notices, and update root ignore rules intentionally.
6. Verify `git diff --stat`, `git status`, no untracked data loss, source readable and comparable to prior commit.
7. Record old application startup/build instructions in future repo docs; legacy may be broken as a build after moving, so document restoration if needed.

## Proposed eventual repo layout (not created by seed)

```text
open-camera-mouse/
├── dump/                         # staging context; can be archived later
├── legacy/                       # old application + archived old AI instructions
├── crates/
│   ├── tracking-core/
│   ├── pointer-core/
│   └── platform/
├── apps/desktop/
├── tests/fixtures/
├── benches/
├── docs/
│   ├── product/
│   ├── research/
│   ├── architecture/decisions/
│   ├── design/
│   └── testing/
├── tasks/{active,done,blocked}/
├── agent/{templates}/
├── .agents/skills/
├── .claude/
├── .codex/
├── AGENTS.md
├── CLAUDE.md -> AGENTS.md       # verify client behavior before relying on link
├── Cargo.toml
└── README.md
```

Do not create empty directories just to satisfy this sketch; meaningful content and tooling determine final layout.
