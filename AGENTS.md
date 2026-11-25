# Open Camera Mouse — agent instructions

Canonical instructions for any AI agent (Claude Code, Codex, or other) working in this
repository. Keep this file short; link to the docs instead of duplicating them.

## The project

A hands-free pointer: a webcam tracks a point on the user's face and drives the OS cursor.
Rust workspace, egui desktop app, macOS / Windows / Linux (X11). v0.4.0 is the first Rust
release (the earlier Go app is in git history, tag `v0.3.0`). The project website is a
separate static Astro site in `website/` (ADR-0023); see "Website tasks" below.

| Read | For |
|---|---|
| `README.md` | Install, use, build, package, release |
| `docs/PRODUCT.md` | What it does, defaults, UI design rules and tokens |
| `docs/BEHAVIOR.md` | The R-* requirements code is held to |
| `docs/ARCHITECTURE.md` | Crates, runtime threads, adapters, platform risks P1–P20, dependencies, packaging |
| `docs/TESTING.md` | Gates, where tests live, manual QA, diagnostics |
| `docs/ROADMAP.md` | Open items and what was deferred |
| `docs/architecture/decisions/` | ADRs: why things are the way they are |
| `docs/COMMANDS.md` | Shortcut commands (`rhmo`, `h`, `cp`, `mc`, …) |
| `.agents/PLANNING.md` | Planning and tracking: active roadmap + handoff, work items, statuses, leftovers, reconciliation |
| `.agents/WORKFLOW.md`, `.agents/MODELS.md` | Orchestration: roles, delegation, modes, checkpoints; model per role and runtime |
| `docs/design-lab.html` | Approved visual reference (open in a browser) |
| `docs/WEBSITE.md` | Website: scope, content rules, pages, design and tokens, stack, CI and Pages deployment |
| `docs/V0.4.1_HANDOFF.md`, `docs/V0.4.1_ROADMAP.md` | v0.4.1 release: temporary, on `fix/v0.4.1-stabilization` only. Read the handoff first, keep it current per `.agents/PLANNING.md` "Handoff"; both deleted before merge |

## Workflow: one prompt = one task

Tasks arrive as prompts from the user. There are no task files or session logs; the only
planning files are the temporary roadmap + handoff of a multi-session initiative the user
approved (`.agents/PLANNING.md`, ADR-0025).
A prompt that is only a shortcut word (`rhmo`, `h`, `hcp`, `cp`, `v`, `r`, `lr`, `mc`, `resume`)
is a command: resolve it through `docs/COMMANDS.md` first.

**Orchestration:** you are the main agent (accountable lead). Read `.agents/WORKFLOW.md` at
the start of every session and run the steps below through it. Single-agent execution is the
default: delegate (models in `.agents/MODELS.md`) only when it clearly pays, and get
independent review when the risk warrants it; spike and main-context (`mc`) modes are
defined there (ADR-0024). Use the fewest agents, the least context and the cheapest suitable
model that deliver a correct, verified result.

1. **Understand.** `git status` and `git log -5`; read the docs and ADRs for the area; read
   the code. Don't guess architecture; if a doc and the code disagree, the code wins, fix the
   doc.
2. **Clarify** only what you can't decide from the code and docs (product behaviour, defaults,
   anything user-visible). Ask before changing a default or adding behaviour.
3. **Measure first** for bugs and tuning: reproduce with a test, a metric or `OCM_PERF`
   before changing anything.
4. **Change** in small steps, with a test that fails before and passes after. A
   non-trivial architectural choice gets an ADR *before* the code (template `0000`); a
   changed default gets an amendment to the ADR that owns it.
5. **Verify** with the gates below (scoped, one heavy command at a time) and, for
   device-only behaviour, say exactly what the user should check by hand.
6. **Finish:** update the docs the change made stale (the active roadmap's statuses, and
   `docs/ROADMAP.md` if the user moved something there (`.agents/PLANNING.md`)), report what was verified and what wasn't, then **ask whether to commit**
   (and separately whether to push). At each subtask checkpoint, offer a commit and push even
   if checks failed or work is unfinished; record their actual status. A checkpoint does not
   waive the verification or release gates.

Larger work: plan it with the `engineering-planning` skill; do one item (or a slice the user
names) per session; multi-session initiatives per `.agents/PLANNING.md`.

## Principles

- Start from Camera Mouse 2018 behaviour, improve on it; new behaviour is an option, off by
  default, unless the user approves a new default (ADR-0006).
- UI: monochrome chrome, colour preview, neon-green ring marker, yellow primary buttons,
  Carbon/Paper themes, two fixed windows (`docs/PRODUCT.md`).
- Hotkeys are fixed (F10 macOS / F11 elsewhere, F12); don't make them configurable without
  asking.
- Every fix gets an automated check (CI test, replay metric or `OCM_PERF`). Recordings are for
  diagnosis only: never commit them or numbers from them; turn findings into synthetic tests.
- Crate boundaries per ADR-0007: pure `tracking-core` / `pointer-core`, `app-core` owns
  state and ports, `platform` owns OS code, `apps/desktop` the UI. No OpenCV, ever.
- Cross-platform: macOS is the dev machine, but Windows and Linux ship too. Don't assume
  POSIX or macOS behaviour (no symlinks, ADR-0002); say when something is untested on an OS.
- Treat Camera Mouse 2018 findings as behavioural evidence; never paste decompiled or
  proprietary code here.

## Quality gates

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked        # locally: scope with -p / a filter
scripts/check-core-deps.sh             # when touching core crate dependencies
```

CI (`.github/workflows/ci.yml`) runs them on macOS, Windows and Linux plus `cargo deny`. Read
a failed job's annotations without auth at
`https://api.github.com/repos/grga1349/open-camera-mouse/check-runs/<job id>/annotations`
(no `gh` CLI here). Release builds and packaging are slow: ask before running them.

## Website tasks

Any change under `website/` or to `.github/workflows/website.yml`: first read
`docs/WEBSITE.md` (and its section in `docs/ROADMAP.md`), and the
`astro-development` and `web-accessibility` skills (plus `website-design` for anything visual or textual). The
workflow above applies unchanged (understand, clarify, change, verify, ask before commit);
new packages need an ADR-0023 amendment, and site copy may only state what `README.md` and
`docs/PRODUCT.md` support. Gates, from `website/` (Bun, no Node needed):

```sh
bun install --frozen-lockfile
bun run check                          # astro check, 0 errors/warnings
bun run test                           # bun test (src/lib/*.test.ts)
bun run build                          # astro build + base-path/dead-link check of dist/
```

Website-only changes don't need the Rust gates (and don't trigger the Rust CI); a change
touching both runs both. Report by-hand checks (keyboard, both themes, 320px, screen reader)
as done or not done.

## Skills

Project skills live in `.agents/skills/<name>/SKILL.md`. Codex discovers them; **Claude Code
does not**, so `Read` the matching `SKILL.md` before starting (ADR-0005). Load only the
skills the task needs, and a skill's `references/` files only when its `SKILL.md` says so;
a delegation names the skills its subagent must read (`.agents/WORKFLOW.md`).

| Skill | Use for |
|---|---|
| `rust-engineering` | Idiomatic Rust: ownership, API/types, errors, allocation, hot paths, threads, unsafe/FFI (refs lazy) |
| `rust-architecture` | Crate boundaries, dependency direction, ADRs |
| `engineering-planning` | Plans: discovery, slicing, dependencies, risk-first order, acceptance criteria, verification, plan review |
| `rust-refactoring` | Behaviour-preserving restructuring of existing code; read-only code-health audits (evidence first) |
| `rust-testing` | Unit / integration / property / UI test design |
| `computer-vision` | Pyramids, interpolation, optical flow maths |
| `tracking-validation` | Synthetic fixtures, ground truth, tracker and pointer metrics |
| `benchmarking` | Performance measurement, regressions |
| `egui-development` | eframe/egui 0.37: immediate mode, viewports, interaction/sliders, repaint, `egui_kittest`, AccessKit |
| `platform-integration` | Camera / cursor / hotkey / permission adapters per OS |
| `desktop-input` | OS input ↔ winit ↔ egui: synthetic vs physical events, drags, focus, feedback loops, coordinates |
| `systematic-debugging` | Bugs: reproduce, hypotheses, evidence, root cause across layers, device-only failures |
| `code-review` | Independent review: correctness, failure modes, evidence |
| `release` | Version bump (SemVer patch/minor/major), gates, commit, tag, draft release |
| `astro-development` | Website: Astro components, Tailwind v4, base-path URLs, zero-JS, build gates |
| `website-design` | Website: brand, tokens, typography, responsive layout, content rules |
| `web-accessibility` | Website: WCAG 2.2 AA, keyboard, focus, screen readers, contrast, motion |

## Instruction files

`CLAUDE.md` is a plain file (not a symlink, ADR-0002) that imports this one with
`@AGENTS.md`; Codex reads `AGENTS.md` directly. Orchestration lives only in `.agents/`
(`WORKFLOW.md`, `MODELS.md`); no `.claude/agents/` or `.codex/agents/` copies (ADR-0024).

## Never

- Commit or push without the user's yes; force-push; rewrite published history.
- Add OpenCV (or any non-permissive licence; `deny.toml` is the policy).
- Add a crate, a cross-layer dependency or a website package without an ADR.
- Change a default, a hotkey or the visual direction without the user's OK.
- Commit camera recordings or anything derived from them.
